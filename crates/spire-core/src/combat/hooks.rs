//! Typed combat lifecycle checkpoints consumed by the shared effect resolver.
use super::{Effect, EffectError, EffectKind, PowerId, PowerEntry, Source};
use super::model::{Actor, State};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hook {
    CardAcquired, CardRemoved, RoomEntered, CombatStarted, TurnStartedPreDraw, TurnStarted, TurnEnded,
    PlayerPreEndTurnCards, PlayerEndTurn, EnemyTurnStarted,
    CardPlayed, CardAfterUse, CardDiscarded, CardManuallyDiscarded, CardExhausted, CardDrawn, Damage, Death, CombatSettled,
    ChestOpened{boss:bool},ChestOpenedAfter{boss:bool},Rested,HalfDeath,CardAfterPlayed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HookEvent { pub hook: Hook, pub source: Source, pub target: Option<Actor> }

/// Execute the finite native behaviors owned by the combat foundation.
/// Producers enqueue this as ordinary effect work; failures leave it as the
/// resolver's current effect and therefore preserve first-fault provenance.
pub(super) fn dispatch(state: &mut State, hook: Hook, source: &Source,target:Option<&Actor>) -> Result<(), EffectError> {
    if !state.full_runtime { return Ok(()); }

    let mut work = Vec::new();
    let mut after_use_bottom=Vec::new();
    let mut exhaust_top = Vec::new();
    if hook==Hook::Death {
        work.extend(super::cards::silent_poison_death_work(state,target)?);
        super::enemies::act2::stasis::on_death(state,target);
        work.extend(super::enemies::act1::spore_cloud_work(state,target));
        work.extend(super::enemies::act1::thieves::death_reward_work(state,target));
        // Monster power callbacks precede player relic callbacks. Capture the
        // dying owner's Poison here; the transfer chooses its live target later.
        if let (Some(items),Some(Actor::Enemy(owner)))=(state.item_context.as_ref(),target) {
            if let Some(enemy)=state.enemies.iter().find(|enemy|enemy.instance_id==*owner&&enemy.hp<=0) {
                let poison=enemy.full_powers.iter().find(|power|power.id.0=="poison").map(|power|power.state.amount);
                let live=state.enemies.iter().filter(|enemy|enemy.life==super::ActorLife::Active&&enemy.hp>0).count();
                work.extend(crate::items::silent_death_work(items,owner.clone(),poison,live));
            }
        }
    }
    if hook==Hook::TurnStartedPreDraw&&target==Some(&Actor::Player){
        super::cards::ironclad::reactive::turn_started(state,&mut work);
        work.extend(super::cards::silent_nightmare_pre_draw(state));
    }
    if hook == Hook::CardPlayed {
        if let Source::Card(owner)=source {
            if let Some(card)=state.in_play.as_ref().filter(|card|card.handle()==*owner).and_then(|c|c.full()) {
                state.silent.history_enabled|=super::cards::requires_draw_history(card.id);
                state.silent.played_this_turn.push(card.id);
            }
        }
        super::cards::ironclad::reactive::card_played(state,source)?;
        work.extend(super::enemies::gremlin_nob_card_played_work(state, source)?);
        super::enemies::guardian::card_played(state,source)?;
        super::enemies::act2::chosen_snecko::card_played(state,source)?;
        super::enemies::act3::awakened::card_played(state,source)?;
        super::cards::ironclad::autoplay::card_played(state,source,target,&mut work)?;
        super::cards::silent_power_on_use(state)?;
        super::cards::colorless_power_card_played(state,source)?;
        if state.player_full_powers.iter().any(|p|p.id.0=="a_thousand_cuts"&&p.state.amount>0){
            work.push(Effect{source:source.clone(),target:None,kind:EffectKind::Hook(Hook::CardAfterPlayed)});
        }
    }
    if hook==Hook::CardAfterPlayed{super::cards::silent_power_after_played(state);}
    if hook==Hook::CardAfterUse {
        if !matches!(source,Source::Card(_)){return Err(super::InputError::Invariant("after-use checkpoint requires the actual card owner".into()).into())}
        for enemy in state.enemies.iter().filter(|enemy|enemy.enemy_id==super::ActorEnemyId::Full(crate::content::EnemyId::CorruptHeart)&&enemy.hp>0&&enemy.life==super::ActorLife::Active){
            if let Some(power)=enemy.full_powers.iter().find(|power|power.id.0=="beat_of_death"&&power.state.amount>0){
                after_use_bottom.push(Effect{source:Source::Actor(Actor::Enemy(enemy.instance_id.clone())),target:Some(Actor::Player),kind:EffectKind::Damage(super::DamagePayload::Thorns(power.state.amount))});
            }
        }
    }
    if hook==Hook::CardExhausted {
        let (top,tail)=super::exhaust::dispatch(state,source)?;
        exhaust_top=top;
        work.extend(tail);
    }
    if hook == Hook::CardDrawn && target == Some(&Actor::Player) {
        let accuracy = state.player_full_powers.iter()
            .find(|power| power.id.0 == "accuracy" && power.state.amount > 0)
            .map(|power| power.state.amount);
        if let Some(amount) = accuracy {
            super::cards::rewrite_accuracy_hand_shivs(state, amount);
        }
    }
    if hook==Hook::EnemyTurnStarted {
        let Some(Actor::Enemy(owner))=target else {
            return Err(super::InputError::Invariant("enemy start-turn hook owner absent".into()).into());
        };
        if !super::enemies::act2::combat_victory_ready(state) {
            if let Some(enemy)=state.enemies.iter().find(|enemy|&enemy.instance_id==owner
                &&enemy.life==super::ActorLife::Active&&enemy.hp>0) {
                if let Some(poison)=enemy.full_powers.iter().find(|power|power.id.0=="poison")
                    .map(|power|power.state.amount).filter(|amount|*amount>0) {
                    let actor=Actor::Enemy(owner.clone());
                    work.push(Effect{source:Source::Actor(actor.clone()),target:Some(actor.clone()),
                        kind:EffectKind::Damage(super::DamagePayload::PoisonTickHpLoss {
                            owner: owner.clone(), amount: poison })});
                    work.push(Effect{source:Source::Actor(actor.clone()),target:Some(actor.clone()),
                        kind:EffectKind::ReduceFullPower{id:super::FullPowerId("poison"),amount:1}});
                    work.push(Effect{source:Source::Actor(actor.clone()),target:Some(actor),
                        kind:EffectKind::PoisonTickComplete});
                }
            }
        }
    }
    if let Some(Actor::Player)=target {
        match hook {
            Hook::TurnStartedPreDraw=>{
                for power in &state.player_full_powers {
                    if power.id.0=="berserk"&&power.state.amount>0 {
                        work.push(Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::GainEnergy(power.state.amount)});
                    }
                }
            }
            Hook::TurnStarted=>{
                for power in &state.player_full_powers {
                    match power.id.0 {
                        "brutality" if power.state.amount>0=>{
                            work.push(Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::Draw(power.state.amount)});
                            work.push(Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::Damage(super::DamagePayload::HpLoss(power.state.amount))});
                        }
                        "demon_form" if power.state.amount>0=>work.push(Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::ApplyFullPower{id:super::FullPowerId("strength"),amount:power.state.amount}}),
                        "noxious_fumes" if power.state.amount>0=>{
                            // The player-owned callback emits one ordinary
                            // application per eligible live enemy. The shared
                            // power owner still handles Artifact and relics.
                            for enemy in state.enemies.iter().filter(|enemy|
                                enemy.life==super::ActorLife::Active&&enemy.hp>0) {
                                work.push(Effect{source:Source::Actor(Actor::Player),
                                    target:Some(Actor::Enemy(enemy.instance_id.clone())),
                                    kind:EffectKind::ApplyFullPower{id:super::FullPowerId("poison"),amount:power.state.amount}});
                            }
                        }
                        _=>(),
                    }
                }
            }
            Hook::PlayerPreEndTurnCards=>{
                if let Some(power)=state.player_full_powers.iter().find(|power|power.id.0=="metallicize"&&power.state.amount>0){
                    work.push(Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::GainBlock(power.state.amount)});
                }
                if let Some(power)=state.player_full_powers.iter().find(|power|power.id.0=="retain_cards"&&power.state.amount>0){
                    work.push(Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),
                        kind:EffectKind::OpenRetainSelection{limit:power.state.amount}});
                }
            }
            Hook::PlayerEndTurn=>{
                let copies=state.combust.as_ref().map_or(0,|power|i32::from(power.copies));
                if copies>0&&state.enemies.iter().any(|enemy|enemy.life==super::ActorLife::Active&&enemy.hp>0){
                    let damage=state.player_full_powers.iter().find(|power|power.id.0=="combust").map_or(0,|power|power.state.amount);
                    if damage>0 {
                        work.push(Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::Damage(super::DamagePayload::HpLoss(copies))});
                        for enemy in state.enemies.iter().filter(|enemy|enemy.life==super::ActorLife::Active&&enemy.hp>0){
                            work.push(Effect{source:Source::None,target:Some(Actor::Enemy(enemy.instance_id.clone())),kind:EffectKind::Damage(super::DamagePayload::Thorns(damage))});
                        }
                    }
                }
                super::cards::ironclad::reactive::turn_ended(state,&mut work);
            }
            _=>(),
        }
    }
    super::generation::start_turn(state,hook,target,&mut work);
    super::cards::silent_power_hook(state,hook,target,&mut work)?;
    super::cards::colorless_power_hook(state,hook,target,&mut work)?;
    let m15_player=state.item_context.as_ref().is_some_and(|items|items.m15_profile());
    if hook==Hook::PlayerPreEndTurnCards&&target==Some(&Actor::Player)&&m15_player {
        for power in &state.player_full_powers {
            if power.id.0=="ritual"&&power.state.amount>0 {
                work.push(Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),
                    kind:EffectKind::ApplyFullPower{id:super::FullPowerId("strength"),amount:power.state.amount}});
            }
            if power.id.0=="regeneration"&&power.state.amount>0&&state.player.hp>0 {
                work.push(Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::Heal(power.state.amount)});
                work.push(Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::ReduceFullPower{id:power.id,amount:1}});
            }
        }
        stat_loss_work(state,&mut work);
    }
    if hook==Hook::PlayerPreEndTurnCards&&target==Some(&Actor::Player)&&m15_player {
        if let Some(power)=state.player_full_powers.iter().find(|power|power.id.0=="plated_armor"&&power.state.amount>0) {
            work.push(Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::GainBlock(power.state.amount)});
        }
    }

    if hook == Hook::TurnEnded {
        // The existing Transient continuation removes its own Shackled earlier.
        // Other accepted applications share this end-round restoration owner.
        for enemy in state.enemies.iter().filter(|enemy|state.silent.history_enabled&&enemy.life==super::ActorLife::Active&&enemy.hp>0){
            if let Some(power)=enemy.full_powers.iter().find(|power|power.id.0=="shackled"&&power.state.amount>0){
                let actor=Actor::Enemy(enemy.instance_id.clone());
                work.push(Effect{source:Source::Actor(actor.clone()),target:Some(actor.clone()),kind:EffectKind::ApplyFullPower{id:super::FullPowerId("strength"),amount:power.state.amount}});
                work.push(Effect{source:Source::Actor(actor.clone()),target:Some(actor),kind:EffectKind::RemoveFullPower(power.id)});
            }
        }
        if !m15_player { stat_loss_work(state,&mut work); }
        round_end_work(&mut state.player.powers, Actor::Player, &mut work);
        round_end_full_work(&mut state.player_full_powers, Actor::Player, &mut work,m15_player);
        for enemy in state.enemies.iter_mut().filter(|enemy|
            enemy.life == super::ActorLife::Active && enemy.hp > 0) {
            let actor = Actor::Enemy(enemy.instance_id.clone());
            round_end_work(&mut enemy.powers, actor.clone(), &mut work);
            round_end_full_work(&mut enemy.full_powers, actor, &mut work,false);
        }

    }
    // CardGroup.moveToExhaustPile invokes listeners immediately, but their
    // addToBot child actions wait behind the caller's already queued top work.
    // Keep notification synchronous while placing those children at the tail.
    let exhaust_batch_in_flight=hook==Hook::CardExhausted&&state.pending.iter().any(|effect|
        matches!(effect.kind,EffectKind::ExhaustBatchContinue(_)));
    if exhaust_batch_in_flight {
        let continuation=state.pending.iter().position(|pending|matches!(pending.kind,EffectKind::ExhaustBatchContinue(_))).unwrap_or(0);
        for effect in exhaust_top.into_iter().rev() {
            state.pending.insert(continuation,effect);
        }
        for effect in work {
            let decision=state.pending.iter().position(|pending|pending.kind==EffectKind::Decision).unwrap_or(state.pending.len());
            state.pending.insert(decision,effect);
        }
    } else {
        for effect in work.into_iter().rev() { state.pending.push_front(effect); }
        for effect in exhaust_top.into_iter().rev() { state.pending.push_front(effect); }
    }
    for effect in after_use_bottom{super::enemies::act2::bottom(state,effect);}
    Ok(())
}

fn stat_loss_work(state:&State,work:&mut Vec<Effect>) {
    for (loss_id,stat) in [("lose_strength","strength"),("lose_dexterity","dexterity")] {
        if let Some(loss)=state.player_full_powers.iter().find(|power|power.id.0==loss_id) {
            work.push(Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::ApplyFullPower{id:super::FullPowerId(stat),amount:-loss.state.amount}});
            work.push(Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::RemoveFullPower(loss.id)});
        }
    }
}

fn round_end_full_work(powers: &mut [super::FullPowerEntry], actor: Actor, work: &mut Vec<Effect>,m15_player:bool) {
    for power in powers {
        let kind = match power.id.0 {
            "ritual" if m15_player => None,
            "ritual" if power.state.skip_first => { power.state.skip_first = false; None }
            "ritual" => Some(EffectKind::ApplyFullPower { id: super::FullPowerId("strength"), amount: power.state.amount }),
            "strength_up" if matches!(actor,Actor::Enemy(_))=>Some(EffectKind::ApplyFullPower{id:super::FullPowerId("strength"),amount:power.state.amount}),
            "metallicize" if matches!(actor,Actor::Enemy(_))=>Some(EffectKind::GainBlock(power.state.amount)),
            "slow"=>{power.state.amount=0;None},
            "double_tap" if matches!(actor,Actor::Player)=>Some(EffectKind::RemoveFullPower(power.id)),
            "duplication" if matches!(actor,Actor::Player)&&power.state.amount==0=>Some(EffectKind::RemoveFullPower(power.id)),
            "duplication" if matches!(actor,Actor::Player)=>Some(EffectKind::ReduceFullPower{id:power.id,amount:1}),
            "intangible" if matches!(actor,Actor::Enemy(_))&&power.state.skip_first=>{power.state.skip_first=false;None},
            "intangible" if matches!(actor,Actor::Enemy(_))=>Some(EffectKind::ReduceFullPower{id:power.id,amount:1}),
            "draw_reduction" if power.state.skip_first=>{power.state.skip_first=false;None},
            "draw_reduction"=>Some(EffectKind::ReduceFullPower{id:power.id,amount:1}),
            "no_block" if matches!(actor,Actor::Player)=>Some(EffectKind::ReduceFullPower{id:power.id,amount:1}),
            "regeneration" if matches!(actor,Actor::Enemy(_))=>Some(EffectKind::Heal(power.state.amount)),
            "regenerate" if matches!(actor,Actor::Enemy(_))=>Some(EffectKind::Heal(power.state.amount)),
            "vulnerable" | "weak" | "frail" if power.state.monster_source_grace => { power.state.monster_source_grace = false; None }
            "vulnerable" | "weak" | "frail" => Some(EffectKind::ReduceFullPower { id: power.id, amount: 1 }),
            _ => None,
        };
        if let Some(kind) = kind {
            work.push(Effect { source: Source::Actor(actor.clone()), target: Some(actor.clone()), kind });
        }
    }
}

fn round_end_work(powers: &mut [PowerEntry], actor: Actor, work: &mut Vec<Effect>) {
    for power in powers {
        let kind = match power.id {
            PowerId::Ritual if power.state.skip_first => {
                power.state.skip_first = false;
                None
            }
            PowerId::Ritual => Some(EffectKind::ApplyPower { id: PowerId::Strength, amount: power.state.amount }),
            PowerId::Vulnerable if power.state.monster_source_grace => {
                power.state.monster_source_grace = false;
                None
            }
            PowerId::Vulnerable => Some(EffectKind::ReducePower { id: PowerId::Vulnerable, amount: 1 }),
            PowerId::Strength => None,
        };
        if let Some(kind) = kind {
            work.push(Effect { source: Source::Actor(actor.clone()), target: Some(actor.clone()), kind });
        }
    }
}
