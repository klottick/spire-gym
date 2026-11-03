//! Typed combat lifecycle checkpoints consumed by the shared effect resolver.
use super::{Effect, EffectError, EffectKind, PowerId, PowerEntry, Source};
use super::model::{Actor, State};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hook {
    CardAcquired, CardRemoved, RoomEntered, CombatStarted, TurnStartedPreDraw, TurnStarted, TurnEnded,
    PlayerPreEndTurnCards, PlayerEndTurn, EnemyTurnStarted,
    CardPlayed, CardAfterUse, CardDiscarded, CardManuallyDiscarded, CardExhausted, CardDrawn, Damage, Death, CombatSettled,
    CombatStartedPreDraw,
    ChestOpened{boss:bool},ChestOpenedAfter{boss:bool},Rested,HalfDeath,CardAfterPlayed,
    AttackResolved, DrawPileShuffled,
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
    // Defect relic turn-start and end-of-turn work runs in the generic relic phase dispatch
    // (`items::relic_phase_effects`), whose top work precedes this hook's work.
    if hook==Hook::Death {
        work.extend(super::cards::silent_poison_death_work(state,target)?);
        super::enemies::act2::stasis::on_death(state,target);
        work.extend(super::enemies::act1::spore_cloud_work(state,target));
        work.extend(super::enemies::act1::thieves::death_reward_work(state,target));
        // Monster power callbacks precede player relic callbacks. Capture the
        // dying owner's Poison here; the transfer chooses its live target later.
        if let (Some(items),Some(Actor::Enemy(owner)))=(state.item_context.as_mut(),target) {
            if let Some(enemy)=state.enemies.iter().find(|enemy|enemy.instance_id==*owner&&enemy.hp<=0) {
                let poison=enemy.full_powers.iter().find(|power|power.id.0=="poison").map(|power|power.state.amount);
                let live=state.enemies.iter().filter(|enemy|enemy.life==super::ActorLife::Active&&enemy.hp>0).count();
                // The one EnemyDied relic dispatch (M27), at the existing relic callback point.
                let mut facts=crate::items::dispatch::PhaseFacts::new(Some(Hook::Death),state.player.hp,state.player.max_hp,state.player.block);
                facts.target=target;facts.dead_enemy=Some((owner,poison,live));
                let relic=crate::items::dispatch::combat_phase(items,crate::items::dispatch::ItemPhase::EnemyDied,&facts)?;
                // Top work joins this hook's work; bottom work (Gremlin Horn's addToBot) is queued
                // at the bottom, so at end of turn it follows the hand discard.
                for effect in &relic.bottom{super::engine::validate_effect(effect,state)?;}
                work.extend(relic.top);super::effects::queue_relic_bottom(state,relic.bottom,false);
            }
        }
    }
    if hook == Hook::CardPlayed {
        if let Source::Card(owner)=source {
            if let Some(card)=state.in_play.as_ref().filter(|card|card.handle()==*owner).and_then(|c|c.full()) {
                state.silent.history_enabled|=super::cards::requires_draw_history(card.id);
                state.silent.played_this_turn.push(card.id);
                if let Some(row)=crate::content::Catalog::standard().card(card.id){
                    state.played_card_types.push(row.card_type);
                    // FreeAttackPower.onUseCard: a qualifying non-purge Attack use spends one
                    // entitlement, whatever its cost or route (watcher.md:123).
                    if row.card_type==crate::content::CardType::Attack&&!card.local.purge
                        &&state.player_full_powers.iter().any(|power|power.id.0=="free_attack"&&power.state.amount>0){
                        work.push(Effect{source:source.clone(),target:Some(Actor::Player),
                            kind:EffectKind::ReduceFullPower{id:super::FullPowerId("free_attack"),amount:1}});
                    }
                    if row.card_type==crate::content::CardType::Power{
                    state.silent.powers_played_this_combat=state.silent.powers_played_this_combat.checked_add(1)
                        .ok_or_else(||super::InputError::Unsupported("native combat Power history exhausted".into()))?;
                    }
                }
            }
        }
        super::cards::ironclad::reactive::card_played(state,source)?;
        work.extend(super::enemies::gremlin_nob_card_played_work(state, source)?);
        super::enemies::guardian::card_played(state,source)?;
        super::enemies::act2::chosen_snecko::card_played(state,source)?;
        super::enemies::act3::awakened::card_played(state,source)?;
        super::cards::ironclad::autoplay::card_played(state,source,target,&mut work)?;
        super::cards::defect::card_played(state,source,target)?;
        super::cards::silent_power_on_use(state)?;
        super::cards::colorless_power_card_played(state,source)?;
        if let Source::Card(owner)=source{
            if let Some(card)=state.in_play.as_ref().filter(|card|card.handle()==*owner).and_then(|c|c.full()){
                super::cards::curses::on_card_played(state,card,&mut work);
            }
        }
        if state.player_full_powers.iter().any(|p|p.id.0=="a_thousand_cuts"&&p.state.amount>0){
            work.push(Effect{source:source.clone(),target:None,kind:EffectKind::Hook(Hook::CardAfterPlayed)});
        }
    }
    if hook==Hook::CardAfterPlayed{super::cards::silent_power_after_played(state);}
    if hook==Hook::CardAfterUse {
        if !matches!(source,Source::Card(_)){return Err(super::InputError::Invariant("after-use checkpoint requires the actual card owner".into()).into())}
        let duality_attack=if let Source::Card(owner)=source {
            state.in_play.as_ref().filter(|card|card.handle()==*owner).and_then(|card|card.full())
                .is_some_and(|card|crate::content::Catalog::standard().card(card.id)
                    .is_some_and(|row|row.card_type==crate::content::CardType::Attack))
                &&state.item_context.as_ref().is_some_and(|items|items.owns_relic(crate::content::RelicId::Duality))
        }else{false};
        if duality_attack {
            work.push(Effect{source:source.clone(),target:Some(Actor::Player),kind:EffectKind::Hook(Hook::AttackResolved)});
        }
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
    // AbstractRoom.applyEndOfTurnPreCardPowers and the player's applyEndOfTurnTriggers each
    // loop once over the player's power list (source-check-endturn-v1), so each callback runs
    // in list order whichever engine module owns it (spire-oto.22).
    if target==Some(&Actor::Player)&&matches!(hook,Hook::PlayerPreEndTurnCards|Hook::PlayerEndTurn){
        let (mut top,mut bombs)=(vec![],0);
        for power in state.player_full_powers.clone(){
            if power.id==super::powers::BOMB_TIMER{
                if hook==Hook::PlayerEndTurn{super::cards::colorless_end_turn_bomb(state,bombs,&mut work);}
                bombs+=1;continue;
            }
            player_end_power(state,hook,&power,&mut top,&mut work)?;
        }
        // RegenPower queues its heal at the top, ahead of the pass's bottom work.
        for effect in top.into_iter().rev(){work.insert(0,effect);}
    }
    // AbstractCreature.applyStartOfTurnPowers and applyStartOfTurnPostDrawPowers each loop once
    // over the player's power list, so callbacks run in list order whichever engine module owns
    // them (spire-oto.21), Night Terror included. The stance hook precedes them; the orb pass
    // follows them.
    if target==Some(&Actor::Player)&&matches!(hook,Hook::TurnStartedPreDraw|Hook::TurnStarted){
        if hook==Hook::TurnStartedPreDraw{super::cards::watcher::stance_start(state,&mut work);}
        let mut night_terror=0;
        for power in state.player_full_powers.clone(){
            if power.id==super::powers::NIGHT_TERROR{
                if hook==Hook::TurnStartedPreDraw{work.extend(super::cards::silent_nightmare_pre_draw(state,night_terror));}
                night_terror+=1;continue;
            }
            player_start_power(state,hook,&power,&mut work)?;
        }
        if hook==Hook::TurnStartedPreDraw{super::cards::defect::orbs_start(state,&mut work)?;}
    }
    // MonsterGroup.applyEndOfTurnPowers runs the monsters' end-turn powers before every
    // atEndOfRound callback, the player's included.
    if hook==Hook::TurnEnded{monster_end_turn(state,&mut work);}
    super::cards::defect::power_hook(state,hook,target,&mut work)?;
    super::cards::watcher::hook(state,hook,target,&mut work)?;
    super::cards::silent_power_hook(state,hook,target,&mut work)?;
    // Later turns recharge at the turn draw's construction; turn 1's initial energy action adds
    // the starting energy (set at entry) and dispatches the same onEnergyRecharge callbacks.
    if target==Some(&Actor::Player)&&hook==Hook::TurnStartedPreDraw {
        if state.turn>1 {super::turns::recharge(state,&mut work)?;} else {super::turns::on_energy_recharge(state,&mut work)?;}
    }
    if hook == Hook::TurnEnded {
        round_end_work(&mut state.player.powers, Actor::Player, &mut work);
        round_end_full_work(&mut state.player_full_powers, Actor::Player, &mut work);
        for index in slot_order(state) {
            let enemy = &mut state.enemies[index];
            if enemy.life != super::ActorLife::Active || enemy.hp <= 0 { continue; }
            let actor = Actor::Enemy(enemy.instance_id.clone());
            round_end_work(&mut enemy.powers, actor.clone(), &mut work);
            round_end_full_work(&mut enemy.full_powers, actor, &mut work);
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
        for effect in work { super::enemies::act2::bottom(state,effect); }
    } else if hook==Hook::TurnStartedPreDraw&&target==Some(&Actor::Player)&&state.turn<=1 {
        // The battle-start block: card, power and orb work follows the battle-start and
        // turn-start relic work already queued (`effects::player_turn_start`).
        for effect in work { super::enemies::act2::bottom(state,effect); }
        for effect in exhaust_top.into_iter().rev() { state.pending.push_front(effect); }
    } else if hook==Hook::EnemyTurnStarted {
        // MonsterGroup.applyPreTurnLogic calls every monster's start-of-turn powers in one loop;
        // their addToBot work (Poison captures its amount now) queues in monster order ahead of
        // the first move (source-check-monsterturn-v3 Q10).
        for effect in work { super::enemies::act2::bottom(state,effect); }
        for effect in exhaust_top.into_iter().rev() { state.pending.push_front(effect); }
    } else {
        for effect in work.into_iter().rev() { state.pending.push_front(effect); }
        for effect in exhaust_top.into_iter().rev() { state.pending.push_front(effect); }
    }
    for effect in after_use_bottom{super::enemies::act2::bottom(state,effect);}
    Ok(())
}

/// One player power's callback in a start slot. Each module still owns its powers' work.
fn player_start_power(state:&mut State,hook:Hook,power:&super::FullPowerEntry,work:&mut Vec<Effect>)->Result<(),EffectError>{
    let player=|kind:EffectKind|Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind};
    let amount=power.state.amount;
    match (hook,power.id.0) {
        (Hook::TurnStartedPreDraw,"berserk") if amount>0=>work.push(player(EffectKind::GainEnergy(amount))),
        // EnergyDownPower.atStartOfTurn queues the loss (G7: the power persists).
        (Hook::TurnStartedPreDraw,"energy_down") if amount>0=>work.push(player(EffectKind::LoseEnergy(amount))),
        (Hook::TurnStartedPreDraw,"flame_barrier")=>work.push(player(EffectKind::RemoveFullPower(super::FullPowerId("flame_barrier")))),
        (Hook::TurnStarted,"brutality") if amount>0=>{
            work.push(player(EffectKind::Draw(amount)));
            work.push(player(EffectKind::Damage(super::DamagePayload::HpLoss(amount))));
        }
        (Hook::TurnStarted,"demon_form") if amount>0=>work.push(player(EffectKind::ApplyFullPower{id:super::FullPowerId("strength"),amount})),
        // The player-owned callback emits one ordinary application per eligible live enemy.
        // The shared power owner still handles Artifact and relics.
        (Hook::TurnStarted,"noxious_fumes") if amount>0=>{
            for enemy in state.enemies.iter().filter(|enemy|enemy.life==super::ActorLife::Active&&enemy.hp>0) {
                work.push(Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Enemy(enemy.instance_id.clone())),
                    kind:EffectKind::ApplyFullPower{id:super::FullPowerId("poison"),amount}});
            }
        }
        _=>(),
    }
    if hook==Hook::TurnStartedPreDraw{
        super::cards::defect::start_power(state,power,work)?;
        super::generation::start_power(state,power,work)?;
    }
    super::cards::watcher::start_power(state,hook,power,work)?;
    super::cards::silent_start_power(state,hook,power,work);
    super::cards::colorless_start_power(state,hook,power,work)
}

/// One player power's end-of-turn callback: atEndOfTurnPreEndTurnCards at
/// `PlayerPreEndTurnCards`, atEndOfTurn(isPlayer) at `PlayerEndTurn` (source-check-endturn-v1).
/// `top` holds addToTop work, which resolves ahead of the pass's addToBot work.
fn player_end_power(state:&mut State,hook:Hook,power:&super::FullPowerEntry,top:&mut Vec<Effect>,work:&mut Vec<Effect>)->Result<(),EffectError>{
    let player=|kind:EffectKind|Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind};
    let amount=power.state.amount;
    let enemies_alive=state.enemies.iter().any(|enemy|enemy.life==super::ActorLife::Active&&enemy.hp>0);
    match (hook,power.id.0) {
        (Hook::PlayerPreEndTurnCards,"metallicize"|"plated_armor") if amount>0=>work.push(player(EffectKind::GainBlock(amount))),
        (Hook::PlayerPreEndTurnCards,"like_water") if state.character.stance==super::Stance::Calm=>work.push(player(EffectKind::GainBlock(amount))),
        (Hook::PlayerEndTurn,"amplify"|"burst"|"double_tap"|"entangled"|"no_draw"|"rage"|"cannot_change_stance")=>
            work.push(player(EffectKind::RemoveFullPower(power.id))),
        (Hook::PlayerEndTurn,"rebound")=>{super::cards::defect::end_turn_rebound(state);}
        (Hook::PlayerEndTurn,"combust") if enemies_alive=>{
            let copies=state.combust.as_ref().map_or(0,|power|i32::from(power.copies));
            if copies>0&&amount>0 {
                work.push(player(EffectKind::Damage(super::DamagePayload::HpLoss(copies))));
                for enemy in state.enemies.iter().filter(|enemy|enemy.targetable()){
                    work.push(Effect{source:Source::None,target:Some(Actor::Enemy(enemy.instance_id.clone())),kind:EffectKind::Damage(super::DamagePayload::Thorns(amount))});
                }
            }
        }
        (Hook::PlayerEndTurn,"constricted") if amount>0=>work.extend(super::enemies::act3::ordinary::player_end_work(state)),
        (Hook::PlayerEndTurn,"equilibrium")=>super::cards::defect::end_turn_equilibrium(state),
        (Hook::PlayerEndTurn,"lose_strength"|"lose_dexterity")=>{
            let stat=if power.id.0=="lose_strength"{"strength"}else{"dexterity"};
            work.push(player(EffectKind::ApplyFullPower{id:super::FullPowerId(stat),amount:-amount}));
            work.push(player(EffectKind::RemoveFullPower(power.id)));
        }
        (Hook::PlayerEndTurn,"regeneration") if amount>0&&state.player.hp>0=>{
            top.push(player(EffectKind::Heal(amount)));
            top.push(player(EffectKind::ReduceFullPower{id:power.id,amount:1}));
        }
        // RetainCardPower retains nothing when Runic Pyramid or Equilibrium already applies.
        (Hook::PlayerEndTurn,"retain_cards") if amount>0&&!state.piles.hand.is_empty()
            &&!state.item_context.as_ref().is_some_and(|items|items.owns_relic(crate::content::RelicId::RunicPyramid))
            &&!state.player_full_powers.iter().any(|power|power.id.0=="equilibrium")=>
            work.push(player(EffectKind::OpenRetainSelection{limit:amount})),
        (Hook::PlayerEndTurn,"ritual") if amount>0=>work.push(player(EffectKind::ApplyFullPower{id:super::FullPowerId("strength"),amount})),
        (Hook::PlayerEndTurn,"wraith_form")=>work.push(player(EffectKind::ApplyFullPower{id:super::FullPowerId("dexterity"),amount})),
        (Hook::PlayerEndTurn,"establishment")=>work.push(player(EffectKind::Watcher(super::cards::watcher::WatcherEffect::EstablishmentDiscount{amount}))),
        (Hook::PlayerEndTurn,"study"|"omega")=>super::cards::watcher::end_turn_power(state,power,work)?,
        _=>(),
    }
    Ok(())
}

/// MonsterGroup.applyEndOfTurnPowers' first loop, after every monster's move: each live monster
/// in group order walks its power list, atEndOfTurnPreEndTurnCards then atEndOfTurn(false) per
/// entry (source-check-endturn-v1 Q2/Q3, source-check-monsterturn-v3). All of it is addToBot
/// except Intangible's justApplied skip, which clears at once.
fn monster_end_turn(state:&mut State,work:&mut Vec<Effect>){
    // Only dying or escaping monsters are skipped (source-check-monsterturn-v3 Q2).
    for index in slot_order(state){
        let enemy=&mut state.enemies[index];
        if !(enemy.life==super::ActorLife::HalfDead||enemy.life==super::ActorLife::Active&&enemy.hp>0){continue}
        let actor=Actor::Enemy(enemy.instance_id.clone());
        let live=enemy.life==super::ActorLife::Active;
        let malleable=matches!(enemy.enemy_id,super::ActorEnemyId::Full(crate::content::EnemyId::SnakePlant|crate::content::EnemyId::WrithingMass));
        for power in &mut enemy.full_powers {
            let amount=power.state.amount;
            let kinds=match power.id.0 {
                "metallicize"|"plated_armor" if amount>0=>vec![EffectKind::GainBlock(amount)],
                "intangible" if power.state.skip_first=>{power.state.skip_first=false;vec![]},
                "intangible"=>vec![EffectKind::ReduceFullPower{id:power.id,amount:1}],
                // Shackled is GainStrengthPower: it restores the Strength, then removes itself.
                "shackled" if amount>0=>vec![EffectKind::ApplyFullPower{id:super::FullPowerId("strength"),amount},EffectKind::RemoveFullPower(power.id)],
                // RegenerateMonsterPower (the Awakened One's and the burning-elite buff) skips a
                // half-dead owner.
                "regenerate" if amount>0&&live=>vec![EffectKind::Heal(amount)],
                // MalleablePower resets to its base amount; nothing between here and the queued
                // reset can hit the monster.
                "malleable" if malleable=>vec![EffectKind::ResetMalleable],
                _=>vec![],
            };
            work.extend(kinds.into_iter().map(|kind|Effect{source:Source::Actor(actor.clone()),target:Some(actor.clone()),kind}));
        }
    }
}

/// MonsterGroup order: a summoned minion takes its slot, not the end of the enemy list (as the
/// MonsterStart queue does).
fn slot_order(state:&State)->Vec<usize>{
    let mut order=(0..state.enemies.len()).collect::<Vec<_>>();
    order.sort_by_key(|&index|state.actor_slots.get(&state.enemies[index].instance_id).copied().unwrap_or(0));
    order
}

fn round_end_full_work(powers: &mut [super::FullPowerEntry], actor: Actor, work: &mut Vec<Effect>) {
    for power in powers {
        let kind = match power.id.0 {
            // RitualPower.atEndOfRound acts only off the player; the player's is atEndOfTurn.
            "ritual" if matches!(actor,Actor::Player) => None,
            "ritual" if power.state.skip_first => { power.state.skip_first = false; None }
            "ritual" => Some(EffectKind::ApplyFullPower { id: super::FullPowerId("strength"), amount: power.state.amount }),
            "strength_up" if matches!(actor,Actor::Enemy(_))=>Some(EffectKind::ApplyFullPower{id:super::FullPowerId("strength"),amount:power.state.amount}),
            "slow"=>{power.state.amount=0;None},
            "duplication" if matches!(actor,Actor::Player)&&power.state.amount==0=>Some(EffectKind::RemoveFullPower(power.id)),
            "duplication" if matches!(actor,Actor::Player)=>Some(EffectKind::ReduceFullPower{id:power.id,amount:1}),
            "draw_reduction" if power.state.skip_first=>{power.state.skip_first=false;None},
            "draw_reduction"=>Some(EffectKind::ReduceFullPower{id:power.id,amount:1}),
            "no_block" if matches!(actor,Actor::Player)=>Some(EffectKind::ReduceFullPower{id:power.id,amount:1}),
            "wave_of_the_hand" if matches!(actor,Actor::Player)=>Some(EffectKind::RemoveFullPower(power.id)),
            "vulnerable" | "weak" | "frail" if power.state.monster_source_grace => { power.state.monster_source_grace = false; None }
            "lock_on" => Some(EffectKind::ReduceFullPower { id: power.id, amount: 1 }),
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
