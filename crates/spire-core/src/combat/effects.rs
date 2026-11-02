//! Exhaustive routing to the finite synchronous payload owners.
use super::*;
use super::model::{OwnedCard, PendingSelection, State};

fn amount(powers: &[PowerEntry], id: PowerId) -> i32 {
    powers.iter().find(|p| p.id == id).map_or(0, |p| p.state.amount)
}

/// Canonical output used by both refresh and ready-check validation. Keeping
/// the power projection here prevents full-only power state from being applied
/// during refresh but omitted by the invariant.
pub(super) fn refreshed_damage(base: i32, enemy_powers: &[PowerEntry],
    enemy_full_powers: &[super::FullPowerEntry], player_powers: &[PowerEntry],
    player_full_powers: &[super::FullPowerEntry], wrath: bool, full_runtime: bool, back_attack:bool) -> Result<i32, InputError>
{
    let strength = amount(enemy_powers, PowerId::Strength)
        + enemy_full_powers.iter().find(|p| p.id.0 == "strength").map_or(0, |p| p.state.amount);
    let vulnerable = amount(player_powers, PowerId::Vulnerable) > 0
        || player_full_powers.iter().any(|p| p.id.0 == "vulnerable" && p.state.amount > 0);
    let weak = enemy_full_powers.iter().any(|p| p.id.0 == "weak" && p.state.amount > 0);
    if full_runtime {
        rules::full_normal_damage_with_back_attack(base,strength,weak,wrath,vulnerable,back_attack)
    }
    else { rules::normal_damage_modifiers(base, strength, weak, wrath, vulnerable) }
}

pub(super) fn refreshed_damage_with_paper_krane(base: i32, enemy_powers: &[PowerEntry],
    enemy_full_powers: &[super::FullPowerEntry], player_powers: &[PowerEntry],
    player_full_powers: &[super::FullPowerEntry], wrath: bool, full_runtime: bool,
    back_attack:bool,paper_krane:bool) -> Result<i32, InputError>
{
    if !paper_krane {
        return refreshed_damage(base,enemy_powers,enemy_full_powers,player_powers,
            player_full_powers,wrath,full_runtime,back_attack);
    }
    let strength = amount(enemy_powers, PowerId::Strength)
        + enemy_full_powers.iter().find(|p| p.id.0 == "strength").map_or(0, |p| p.state.amount);
    let vulnerable = amount(player_powers, PowerId::Vulnerable) > 0
        || player_full_powers.iter().any(|p| p.id.0 == "vulnerable" && p.state.amount > 0);
    let weak = enemy_full_powers.iter().any(|p| p.id.0 == "weak" && p.state.amount > 0);
    if full_runtime {
        rules::full_normal_damage_with_back_attack_weak_multiplier(base,strength,weak,
            0.6f32,wrath,vulnerable,back_attack)
    }
    else { rules::normal_damage_modifiers(base, strength, weak, wrath, vulnerable) }
}

pub(super) fn refreshed_damage_with_vulnerable_factor(base:i32,enemy_powers:&[PowerEntry],
    enemy_full_powers:&[super::FullPowerEntry],player_powers:&[PowerEntry],
    player_full_powers:&[super::FullPowerEntry],wrath:bool,full_runtime:bool,
    back_attack:bool,paper_krane:bool,vulnerable_factor:f32)->Result<i32,InputError>{
    if !full_runtime||vulnerable_factor==1.5_f32 {
        return refreshed_damage_with_paper_krane(base,enemy_powers,enemy_full_powers,
            player_powers,player_full_powers,wrath,full_runtime,back_attack,paper_krane);
    }
    let strength=amount(enemy_powers,PowerId::Strength)
        +enemy_full_powers.iter().find(|power|power.id.0=="strength").map_or(0,|power|power.state.amount);
    let vulnerable=amount(player_powers,PowerId::Vulnerable)>0
        ||player_full_powers.iter().any(|power|power.id.0=="vulnerable"&&power.state.amount>0);
    let weak=enemy_full_powers.iter().any(|power|power.id.0=="weak"&&power.state.amount>0);
    let weak_multiplier=if paper_krane{0.6_f32}else{0.75_f32};
    let modified=rules::full_normal_damage_given_received_with_weak_multiplier(base,strength,weak,
        weak_multiplier,wrath,false,[if vulnerable{vulnerable_factor}else{1.0_f32}])?;
    if back_attack {
        let after=(modified as f32*1.5_f32).trunc() as i32;
        if after>30_012{return Err(InputError::Unsupported("full Back Attack damage outside supported range".into()))}
        Ok(after.max(0))
    }else{Ok(modified)}
}

pub(super) fn player_vulnerable_factor(state:&State)->f32{
    state.item_context.as_ref().map_or(1.5_f32,|context|{
        // M27 seam: player-vulnerable-factor
        crate::items::laws::player_vulnerable_factor(context)
    })
}

pub(super) fn refresh_enemy_values(state: &mut State) -> Result<(), InputError> {
    let wrath = state.character.stance == super::characters::Stance::Wrath;
    let player_powers = &state.player.powers;
    let player_full_powers = &state.player_full_powers;
    let paper_krane=state.item_context.as_ref().is_some_and(|items|
        (items.m15_profile()||items.complete_items())&&items.owns_relic(crate::content::RelicId::PaperKrane));
    let vulnerable_factor=player_vulnerable_factor(state);
    let back_attack=state.enemies.iter().map(|enemy|(enemy.instance_id.clone(),back_attack_active(state,enemy))).collect::<std::collections::BTreeMap<_,_>>();
    for enemy in &mut state.enemies {
        if state.full_runtime{super::enemies::act3::elites::refresh_actor(enemy);}
        let behind=back_attack.get(&enemy.instance_id).copied().unwrap_or(false);
        for entry in enemy.damage_entries.values_mut().chain(enemy.full_damage_entries.values_mut()) {
            // M27 seam: player-vulnerable-factor; the default follows the exact old path.
            entry.output=refreshed_damage_with_vulnerable_factor(entry.base,&enemy.powers,&enemy.full_powers,
                player_powers,player_full_powers,wrath,state.full_runtime,behind,paper_krane,vulnerable_factor)?;
        }
        enemy.intent_damage = match &enemy.intent_id {
            ActorIntentId::Legacy(id) => enemy.damage_entries.get(id).map(|d| d.output),
            ActorIntentId::Full(id) => enemy.full_damage_entries.get(id).map(|d| d.output),
        };
    }
    Ok(())
}

pub(super) fn back_attack_active(state:&State,enemy:&super::EnemyState)->bool{
    let slot=state.actor_slots.get(&enemy.instance_id).copied().unwrap_or(0);
    state.player_full_powers.iter().any(|p|p.id.0=="surrounded"&&p.state.amount>0)
        &&matches!((state.facing,slot),(Some(super::PlayerFacing::Left),1)|(Some(super::PlayerFacing::Right),0))
}

fn powers_mut<'a>(state: &'a mut State, target: &Actor) -> &'a mut Vec<PowerEntry> {
    match target {
        Actor::Player => &mut state.player.powers,
        Actor::Enemy(id) => &mut state.enemies.iter_mut().find(|e| &e.instance_id == id).expect("validated actor").powers,
    }
}

fn is_enemy_poison_tick(state:&State,effect:&Effect)->bool {
    let (Source::Actor(Actor::Enemy(source)),Some(Actor::Enemy(target)),
        EffectKind::Damage(DamagePayload::PoisonTickHpLoss{owner,amount}))=
        (&effect.source,&effect.target,&effect.kind) else { return false; };
    source==target&&source==owner&&state.full_runtime&&*amount>0&&state.enemies.iter().any(|enemy|
        &enemy.instance_id==owner&&enemy.life==super::ActorLife::Active&&enemy.hp>0)
}

/// Queue one relic phase through the shared resolver, preserving its source side.
pub(super) fn queue_relic_work(state:&mut State,work:crate::items::RelicHookWork)->Result<(),EffectError>{
    queue_relic_work_with_startup_order(state,work,false)
}

fn queue_relic_work_with_startup_order(state:&mut State,work:crate::items::RelicHookWork,
    preserve_finite_startup_order:bool)->Result<(),EffectError>{
    if work.is_empty(){return Ok(())}
    for effect in work.top.iter().chain(&work.bottom){super::engine::validate_effect(effect,state)?;}
    for effect in work.top.into_iter().rev(){state.pending.push_front(effect);}
    if preserve_finite_startup_order||state.item_context.as_ref().is_some_and(|items|items.complete_items()) {
        for effect in work.bottom{super::enemies::act2::bottom(state,effect);}
    } else {
        // Bottom work joins the current queue ahead of an already queued player decision
        // (M20/M21/complete-card entry queues the opening Decision before CombatStarted).
        let mut at=state.pending.iter().position(|e|e.kind==EffectKind::Decision).unwrap_or(state.pending.len());
        for effect in work.bottom{state.pending.insert(at,effect);at+=1;}
    }
    Ok(())
}

pub(super) fn phase_card_facts(state:&State,source:&Source)->Option<crate::items::dispatch::PhaseCard>{
    let Source::Card(owner)=source else{return None};
    let card=super::piles::owned_full_cards(state).find(|card|super::card_state::handle(card.instance)==*owner)?;
    let card_type=crate::content::Catalog::standard().card(card.id)?.card_type;
    let context=state.play_context.as_ref().filter(|context|context.card==*owner);
    let cost_paid=context.map_or(0,|context|if card.local.x_cost{context.energy_on_use}else{context.paid});
    Some(crate::items::dispatch::PhaseCard{id:card.id,card_type,cost_paid})
}

pub(super) fn fill_combat_entry_facts(facts:&mut crate::items::dispatch::PhaseFacts<'_>,state:&State){
    if let Some(items)=state.item_context.as_ref(){
        facts.elite=items.battle.elite_trigger;facts.boss=items.battle.boss;
    }
    facts.deck_curses=state.piles.hand.iter().chain(&state.piles.draw).chain(&state.piles.discard).chain(&state.piles.exhaust)
        .filter_map(OwnedCard::full).filter(|card|crate::content::Catalog::standard().card(card.id)
            .is_some_and(|definition|definition.color==crate::content::CardColor::Curse)).count();
}

pub(super) fn apply_effect(state: &mut State, effect: &Effect) -> Result<(), EffectError> {
    apply_effect_with_result(state,effect).map(|_|())
}

pub(super) fn apply_effect_with_result(state:&mut State,effect:&Effect)->Result<Option<super::model::DamageResult>,EffectError>{
    let (result,damage_result)=apply_effect_with_receipt(state,effect);
    result?;
    Ok(damage_result)
}

/// Reaper must retain a committed shared hit even if a later lifecycle callback
/// faults. Ordinary callers keep the original Result API above; this narrow
/// internal helper separates the post-commit receipt from callback success.
pub(super) fn apply_effect_with_receipt(state:&mut State,effect:&Effect)
    ->(Result<(),EffectError>,Option<super::model::DamageResult>){
    let before=super::observe::mark(state);
    let mut damage_result=None;
    let result=apply_effect_inner(state,effect,&mut damage_result);
    super::observe::changes(state,before,&effect.source);
    (result,damage_result)
}

/// Queued damage at an enemy that left combat without a damage death does nothing, as the damage
/// death branch above already does for its corpse. DamageAction cancels on a dead or escaped target,
/// and DamageAllEnemiesAction skips dying, half-dead and escaped monsters when it resolves
/// (source-check-half-dead-hits-v1, source-check-damage-all-enemies-v1). Called by every non-damage
/// departure: suicide, escape, split replacement, Instant Kill. It drops only work that admission
/// would reject: a qualified Dark orb hit at a half-dead target is kept. Admission stays strict.
pub(in crate::combat) fn cancel_queued_hits_at(state:&mut State,target:&Actor){
    let half_dead=matches!(target,Actor::Enemy(owner) if state.enemies.iter().any(|enemy|&enemy.instance_id==owner&&enemy.life==ActorLife::HalfDead));
    state.pending.retain(|queued|queued.target.as_ref()!=Some(target)||!matches!(queued.kind,EffectKind::Damage(_))
        ||half_dead&&matches!(queued.kind,EffectKind::Damage(DamagePayload::Orb(_))));
}

pub(in crate::combat) fn instant_kill(state:&mut State,effect:&Effect)->Result<(),EffectError>{
    let Some(Actor::Enemy(owner))=&effect.target else{return Err(InputError::Invariant("Instant Kill target absent".into()).into())};
    let enemy=state.enemies.iter_mut().find(|enemy|&enemy.instance_id==owner)
        .ok_or_else(||InputError::Invariant("Instant Kill target custody absent".into()))?;
    enemy.hp=0;
    let downed=super::enemies::act3::darklings::down(state,owner,&effect.source)
        ||super::enemies::act3::awakened::down(state,owner,&effect.source);
    cancel_queued_hits_at(state,&Actor::Enemy(owner.clone()));
    if downed{return Ok(())}
    if super::enemies::act2::combat_victory_ready(state)&&state.player.hp>0{
        super::enemies::act2::queue_victory(state,&effect.source);
    }
    state.pending.push_front(Effect{source:effect.source.clone(),target:effect.target.clone(),kind:EffectKind::Hook(Hook::Death)});
    Ok(())
}

fn apply_effect_inner(state: &mut State, effect: &Effect, damage_result_out:&mut Option<super::model::DamageResult>) -> Result<(), EffectError> {
    super::engine::validate_effect(effect, state)?;
    if state.full_runtime && matches!(effect.kind, EffectKind::ApplyPower { .. }
        | EffectKind::ReducePower { .. } | EffectKind::RemovePower(_)
        | EffectKind::ApplyFullPower { .. } | EffectKind::ReduceFullPower { .. } | EffectKind::RemoveFullPower(_)) {
        return super::powers::apply_effect(state, effect).map_err(Into::into);
    }
    match &effect.kind {
        EffectKind::Damage(payload) => {
            let poison_tick=is_enemy_poison_tick(state,effect);
            let already_half_dead=match &effect.target{Some(Actor::Enemy(owner))=>state.enemies.iter().any(|e|&e.instance_id==owner&&e.life==ActorLife::HalfDead),_=>false};
            // The validated target is needed while calculating typed payloads as
            // well as for the shared Block/HP commit below.
            let target = effect.target.as_ref().unwrap();
            let damage = match payload {
                DamagePayload::Fixed(n) | DamagePayload::Thorns(n)
                | DamagePayload::HpLoss(n) => *n,
                DamagePayload::Orb(n) => {
                    let lock_on=if let Actor::Enemy(owner)=target{state.enemies.iter().find(|e|&e.instance_id==owner).is_some_and(|e|e.full_powers.iter().any(|p|p.id.0=="lock_on"&&p.state.amount!=0))}else{false};
                    (if lock_on{((*n as f32)*1.5_f32) as i32}else{*n}).max(0)
                },
                DamagePayload::PoisonTickHpLoss { amount, .. } | DamagePayload::CorpseExplosion {amount,..} => *amount,
                DamagePayload::EnemyMove(id) => {
                    let source = match &effect.source { Source::Actor(Actor::Enemy(id)) => id, _ => unreachable!("validated enemy source") };
                    state.enemies.iter().find(|e| &e.instance_id == source).expect("validated enemy").damage_entries[id].output
                },
                DamagePayload::FullEnemyMove(id) => {
                    let source = match &effect.source { Source::Actor(Actor::Enemy(id)) => id, _ => unreachable!("validated full enemy source") };
                    state.enemies.iter().find(|e| &e.instance_id == source).expect("validated full enemy").full_damage_entries[id].output
                }
            };
            let full_runtime = state.full_runtime;
            let mut damage=if full_runtime{super::enemies::act2::modify_incoming_damage(state,target,payload,damage)?}else{damage};
            if full_runtime&&*target==Actor::Player&&state.player_full_powers.iter().any(|p|p.id.0=="intangible_player"&&p.state.amount>0){damage=damage.min(1);}
            // Invincible caps committed HP loss after Block, while preserving the
            // actual damage-type path and charging only HP damage that occurred.
            let invincible=if full_runtime {if let Actor::Enemy(owner)=target {
                state.enemies.iter().find(|enemy|&enemy.instance_id==owner&&enemy.enemy_id==ActorEnemyId::Full(crate::content::EnemyId::CorruptHeart))
                    .and_then(|enemy|enemy.full_powers.iter().find(|power|power.id.0=="invincible").map(|power|power.state.amount.max(0)))
            }else{None}}else{None};
            let block_at_entry=match target{Actor::Player=>state.player.block,Actor::Enemy(owner)=>state.enemies.iter().find(|enemy|&enemy.instance_id==owner).expect("validated actor").block};
            if full_runtime&&*target==Actor::Player&&state.player_full_powers.iter().any(|power|power.id.0=="buffer"&&power.state.amount>0){
                let ignores_block=matches!(payload,DamagePayload::HpLoss(_)|DamagePayload::PoisonTickHpLoss{..});
                let incoming=if ignores_block{damage}else{damage.saturating_sub(block_at_entry)};
                if incoming>0 {
                    state.pending.push_front(Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::ReduceFullPower{id:FullPowerId("buffer"),amount:1}});
                    damage=if ignores_block{0}else{damage.min(block_at_entry)};
                }
            }
            // AbstractPlayer.damage passes the post-change-damage local to
            // onAttacked. Buffer's zero prevents Discharge, independently of HP.
            if full_runtime&&*target==Actor::Player {super::cards::defect::on_attacked(state,effect,payload,damage.saturating_sub(block_at_entry).max(0));}
            let damage_event_count=state.damage_events_this_combat;
            if let Some(quota)=invincible {let permitted=if matches!(payload,DamagePayload::HpLoss(_)|DamagePayload::PoisonTickHpLoss{..}){quota}else{block_at_entry.saturating_add(quota)};damage=damage.min(permitted);}
            if full_runtime {
                let normal=matches!(payload,DamagePayload::Fixed(_)|DamagePayload::EnemyMove(_)|DamagePayload::FullEnemyMove(_));
                if normal&&matches!(target,Actor::Enemy(_))
                    &&matches!(effect.source,Source::Card(_)|Source::Actor(Actor::Player)) {
                    if let Some(context)=state.item_context.as_ref(){
                        let unblocked=damage.saturating_sub(block_at_entry);
                        // M27 seam: unblocked-outgoing-damage
                        let adjusted=crate::items::laws::unblocked_outgoing_damage(context,target,unblocked);
                        damage=block_at_entry.checked_add(adjusted)
                            .ok_or_else(||InputError::Unsupported("unblocked outgoing damage overflow".into()))?;
                    }
                }
                if normal&&*target==Actor::Player&&matches!(effect.source,Source::Actor(Actor::Enemy(_))){
                    if let Some(context)=state.item_context.as_ref(){
                        let unblocked=damage.saturating_sub(block_at_entry);
                        let attacker=match &effect.source{Source::Actor(actor)=>Some(actor),_=>None};
                        // M27 seam: incoming-damage
                        let adjusted=crate::items::laws::incoming_damage(context,attacker,unblocked);
                        damage=block_at_entry.checked_add(adjusted)
                            .ok_or_else(||InputError::Unsupported("incoming damage overflow".into()))?;
                    }
                }
                if *target==Actor::Player {
                    let ignores_block=matches!(payload,DamagePayload::HpLoss(_)|DamagePayload::PoisonTickHpLoss{..});
                    let hp_loss=if ignores_block{damage}else{damage.saturating_sub(block_at_entry)};
                    if hp_loss>0 {
                        if let Some(context)=state.item_context.as_ref(){
                            // M27 seam: hp-loss
                            let adjusted=crate::items::laws::hp_loss(context,hp_loss);
                            damage=if ignores_block{adjusted}else{block_at_entry.checked_add(adjusted)
                                .ok_or_else(||InputError::Unsupported("player HP loss overflow".into()))?};
                        }
                    }
                }
            }
            let wide_mutable_attack=super::cards::defect::valid_mutable_attack_source(state,&effect.source)||super::cards::watcher::valid_mutable_attack_source(state,&effect.source);
            let (hp_before,block_before,next_hp,next_block)={
                let (hp, block) = match target {
                    Actor::Player => (&mut state.player.hp, &mut state.player.block),
                    Actor::Enemy(id) => {let enemy=state.enemies.iter_mut().find(|e|&e.instance_id==id).expect("validated actor");(&mut enemy.hp,&mut enemy.block)}
                };
                let hp_before=*hp;let block_before=*block;
                let (next_hp,next_block)=if matches!(payload,DamagePayload::HpLoss(_)|DamagePayload::PoisonTickHpLoss{..}){
                    ((i64::from(hp_before)-i64::from(damage)).max(0) as i32,*block)
                }else if full_runtime&&(matches!(payload,DamagePayload::Orb(_))||matches!(payload,DamagePayload::Fixed(_))&&wide_mutable_attack){
                    rules::full_wide_hp_and_block_after_damage(hp_before,*block,damage)?
                }else if full_runtime{rules::full_hp_and_block_after_damage(hp_before,*block,damage)?}
                else{rules::hp_and_block_after_damage(hp_before,*block,damage)?};
                if full_runtime&&*target==Actor::Player&&hp_before>next_hp{
                    damage_event_count.checked_add(1).ok_or_else(||InputError::Unsupported("player damage event history exhausted".into()))?;
                }
                *hp=next_hp;*block=next_block;(hp_before,block_before,next_hp,next_block)
            };
            if let (Some(_),Actor::Enemy(owner))=(invincible,target) {
                // The source callback receives post-Block damage and spends its
                // capped value before creature HP clamps overkill to zero.
                let spent=if matches!(payload,DamagePayload::HpLoss(_)|DamagePayload::PoisonTickHpLoss{..}){damage}else{damage.saturating_sub(block_at_entry)};
                if spent>0 {if let Some(power)=state.enemies.iter_mut().find(|enemy|&enemy.instance_id==owner).and_then(|enemy|enemy.full_powers.iter_mut().find(|power|power.id.0=="invincible")){
                    power.state.amount=power.state.amount.checked_sub(spent).ok_or_else(||InputError::Invariant("Invincible quota underflow".into()))?;
                }}
            }
            let next_damage_event_count=if full_runtime&&target==&Actor::Player&&hp_before>next_hp {Some(damage_event_count+1)}else{None};
            super::observe::emit(state, &effect.source, Some(target), || ResolvedEffectKind::Damage {
                incoming:damage, absorbed_block:block_before-next_block, hp_before, hp_after:next_hp,
                hp_lost:hp_before-next_hp,
                overkill:Some((damage-(block_before-next_block)-(hp_before-next_hp)).max(0)),
                category:match payload { DamagePayload::Orb(_)=>DamageCategory::Orb,
                    DamagePayload::Thorns(_)|DamagePayload::CorpseExplosion{..}=>DamageCategory::Thorns,
                    DamagePayload::HpLoss(_)|DamagePayload::PoisonTickHpLoss{..}=>DamageCategory::HpLoss,
                    _=>DamageCategory::Normal },
            });
            if full_runtime&&matches!(target,Actor::Enemy(_))
                &&!matches!(payload,DamagePayload::HpLoss(_)|DamagePayload::PoisonTickHpLoss{..})
                &&block_before>0&&next_block==0 {
                // M27 seam: block-broken
                let work=if let Some(items)=state.item_context.as_mut(){
                    let (hp,max_hp,block)=(state.player.hp,state.player.max_hp,state.player.block);
                    let mut facts=crate::items::dispatch::PhaseFacts::new(Some(Hook::Damage),hp,max_hp,block);
                    facts.target=Some(target);facts.source=Some(&effect.source);
                    Some(crate::items::dispatch::combat_phase(items,crate::items::dispatch::ItemPhase::BlockBroken,&facts)?)
                }else{None};
                if let Some(work)=work{queue_relic_work(state,work)?;}
            }
            let damage_result=super::model::DamageResult{source:effect.source.clone(),target:target.clone(),normal:matches!(payload,DamagePayload::Fixed(_)|DamagePayload::EnemyMove(_)|DamagePayload::FullEnemyMove(_)),hp_loss:matches!(payload,DamagePayload::HpLoss(_)|DamagePayload::PoisonTickHpLoss{..}),thorns:matches!(payload,DamagePayload::Thorns(_)|DamagePayload::Orb(_)|DamagePayload::CorpseExplosion{..}),attempted:damage,blocked:block_before-next_block,hp_lost:hp_before-next_hp,hp_after:next_hp};
            *damage_result_out=Some(damage_result.clone());
            if full_runtime {
                // Lightning's native Orb lane retains original THORNS callback
                // exclusions; it is not a fourth original DamageType.
                if let Some(count)=next_damage_event_count {state.damage_events_this_combat=count;}
                if damage_result.target==Actor::Player&&damage_result.hp_lost>0 {
                    if let Some(items)=state.item_context.as_mut().filter(|items|items.owns_relic(crate::content::RelicId::EmotionChip)){items.emotion_chip_armed=true;}
                    let reduce=|owned:&mut super::model::OwnedCard| { if let Some(card)=owned.full_mut().filter(|card|card.id==crate::content::CardId::BloodForBlood) {super::card_state::reduce_blood_for_blood_cost(card);}};
                    for owned in state.piles.hand.iter_mut().chain(&mut state.piles.draw).chain(&mut state.piles.discard){reduce(owned);}
                    super::cards::silent_player_hp_lost(state)?;
                }
                super::enemies::act2::after_damage(state,&damage_result)?;
                super::powers::player_thorns_after_damage(state,&damage_result);
                super::cards::silent_power_after_damage(state,&damage_result);
                if let Some(context)=state.item_context.as_mut().filter(|_|damage_result.target==Actor::Player&&damage_result.hp_lost>0){
                    let work=crate::items::laws::hp_changed(context,damage_result.hp_lost,damage_result.hp_after,state.player.max_hp)?;
                    for child in work.into_iter().rev(){super::engine::validate_effect(&child,state)?;state.pending.push_front(child);}
                }
                super::cards::ironclad::reactive::after_damage(state,&damage_result)?;
                super::powers::enemy_block_return_after_damage(state,&damage_result);
                super::cards::watcher::after_damage(state,&damage_result)?;
                if damage_result.target==Actor::Player {
                    // The HpLost relic phase (Runic Cube), at its existing callback point.
                    let (hp,max_hp,block)=(state.player.hp,state.player.max_hp,state.player.block);
                    let work=if let Some(items)=state.item_context.as_mut() {
                        let mut facts=crate::items::dispatch::PhaseFacts::new(Some(Hook::Damage),hp,max_hp,block);
                        facts.hp_lost=damage_result.hp_lost;facts.target=Some(&damage_result.target);facts.source=Some(&effect.source);
                        Some(crate::items::dispatch::combat_phase(items,crate::items::dispatch::ItemPhase::HpLost,&facts)?)
                    }else{None};
                    if let Some(work)=work{queue_relic_work(state,work)?;}
                }
            }
            if let Actor::Enemy(id) = target {
                // Only these native payload kinds are NORMAL for Curl Up.
                // Orb and typed THORNS damage retain their distinct exclusions.
                let normal_damage = matches!(payload,
                    DamagePayload::Fixed(_) | DamagePayload::EnemyMove(_) | DamagePayload::FullEnemyMove(_));
                super::enemies::on_hit(state, id, &effect.source, normal_damage, hp_before, hp_before - next_hp);
            }
            if state.full_runtime&&*target==Actor::Player&&state.player.hp==0 {
                let rescue=state.item_context.as_mut().and_then(|items|items.consume_fairy_for_rescue(state.player.max_hp));
                if let Some((slot,amount))=rescue {
                    let heal=Effect{source:Source::Potion{slot,id:crate::content::PotionId::FairyPotion},
                        target:Some(Actor::Player),kind:EffectKind::Heal(amount)};
                    heal_player(state,&heal,amount,true)?;
                }
                if state.player.hp==0 {
                    // M27 seam: would-die
                    let revive=state.item_context.as_mut().and_then(|items|
                        crate::items::laws::would_die(items,state.player.max_hp));
                    if let Some(amount)=revive {
                        let heal=Effect{source:Source::None,target:Some(Actor::Player),kind:EffectKind::Heal(amount)};
                        heal_player(state,&heal,amount,true)?;
                    }
                }
            }
            let half_death=already_half_dead||(state.full_runtime&&next_hp==0&&match target{Actor::Enemy(owner)=>super::enemies::act3::darklings::down(state,owner,&effect.source)||super::enemies::act3::awakened::down(state,owner,&effect.source),_=>false});
            // Feed's Fatal checkpoint observes the real post-hit lifecycle
            // state (including half-death/revival), before queued retaliation.
            if full_runtime { super::cards::ironclad::fatal::after_damage(state,&damage_result)?; }
            if next_hp == 0 {
                if state.full_runtime&&hp_before>0&&matches!(target,Actor::Enemy(_)){
                    // Accepted multi-hit work stops on this actual corpse even
                    // when other actors keep combat live. Keep unrelated hits,
                    // earned healing and FinishCard; never relax stale admission.
                    state.pending.retain(|queued|queued.target.as_ref()!=Some(target)||!matches!(queued.kind,EffectKind::Damage(_)));
                    // The original damage action does nothing once its owner is dying or
                    // half-dead; THORNS-type damage is exempt (source-settled by static
                    // analysis, source-check-half-dead-hits-v1, and the 2026-11-01 user ruling;
                    // partial repo coverage: docs/research/2026-10-19-act1-content-silent.md:632,
                    // data/enemies/m8-static-findings.md:333-335). So a downed attacker,
                    // dead or half-dead, drops its own remaining queued hits: Byrd pecks,
                    // Darkling Chomp, phase-one Soul Strike. HP loss is not a damage hit.
                    // Book of Stabbing and The Maw queue later hits as owner continuations.
                    state.pending.retain(|queued|!(matches!(&queued.source,Source::Actor(source) if source==target)
                        &&match &queued.kind{
                            EffectKind::Damage(payload)=>!matches!(payload,DamagePayload::Thorns(_)|DamagePayload::HpLoss(_)|DamagePayload::PoisonTickHpLoss{..}),
                            EffectKind::BookStab{..}|EffectKind::MawNom{..}=>queued.target.as_ref()==Some(target),
                            _=>false,
                        }));
                }
                let outcome = match target {
                    _ if half_death=>None,
                    Actor::Player if state.player.hp>0=>None,
                    Actor::Player => Some(Outcome::Death),
                    Actor::Enemy(_) if super::enemies::act2::combat_victory_ready(state) => Some(Outcome::Victory),
                    Actor::Enemy(_) => None,
                };
                if let Some(outcome) = outcome.filter(|_| !poison_tick) {
                    if state.full_runtime&&outcome==Outcome::Victory {
                        super::enemies::act2::queue_victory(state,&effect.source);
                    }else{
                    state.pending.push_front(Effect { source: effect.source.clone(), target: None,
                        kind: EffectKind::Terminal(outcome) });
                    if state.full_runtime {
                        state.pending.push_front(Effect { source: effect.source.clone(), target: None,
                            kind: EffectKind::Hook(Hook::CombatSettled) });
                    }
                    }
                }
                if state.full_runtime&&!half_death&&!poison_tick {
                    state.pending.push_front(Effect { source: effect.source.clone(), target: effect.target.clone(),
                        kind: EffectKind::Hook(Hook::Death) });
                }
            }
            if state.full_runtime {
                state.pending.push_front(Effect { source: effect.source.clone(), target: effect.target.clone(),
                    kind: EffectKind::Hook(Hook::Damage) });
                super::enemies::act2::ordinary_closure::vampire_damage_complete(state,&damage_result,payload);
            }
        }
        EffectKind::PoisonTickComplete => {
            let Some(Actor::Enemy(owner))=effect.target.as_ref() else { unreachable!("validated Poison completion actor") };
            let died=state.enemies.iter().any(|enemy|&enemy.instance_id==owner
                &&enemy.life==super::ActorLife::Active&&enemy.hp==0);
            if died {
                // The shared HP_LOSS and current-stack reduction are ahead of
                // this completion marker, so Death and settlement now follow
                // their source-qualified local order.
                if super::enemies::act2::combat_victory_ready(state) {
                    super::enemies::act2::queue_victory(state,&effect.source);
                }
                state.pending.push_front(Effect{source:effect.source.clone(),
                    target:Some(Actor::Enemy(owner.clone())),kind:EffectKind::Hook(Hook::Death)});
            }
        }
        EffectKind::Catalyst => super::cards::apply_silent_catalyst(state, effect)?,
        EffectKind::JuggernautRandomDamage(damage) => {
            let candidates=state.enemies.iter().filter(|enemy|enemy.life==super::ActorLife::Active&&enemy.hp>0).map(|enemy|enemy.instance_id.clone()).collect::<Vec<_>>();
            if candidates.is_empty(){return Ok(())}
            let stream=state.card_rng.as_ref().ok_or_else(||InputError::Invariant("Juggernaut card-random stream absent".into()))?;
            let (index,next)=super::rng::uniform_below(stream,candidates.len() as u128)?;
            state.card_rng=Some(next);
            state.pending.push_front(Effect{source:effect.source.clone(),target:Some(Actor::Enemy(candidates[index as usize].clone())),kind:EffectKind::Damage(DamagePayload::Thorns(*damage))});
        }
        EffectKind::CardOffer{..}|EffectKind::DeliverCardOffer{..}=>super::card_offers::apply(state,effect)?,
        EffectKind::Relic(RelicEffect::UpgradeRandomHandCard)=>{
            let candidates=state.piles.hand.iter().filter_map(OwnedCard::full)
                .filter(|card|crate::content::Catalog::standard().card(card.id)
                    .is_some_and(|definition|definition.card_type!=crate::content::CardType::Status)
                    &&super::cards::ironclad::rules::can_upgrade(card.id,card.upgrades))
                .map(|card|card.instance).collect::<Vec<_>>();
            if candidates.is_empty(){return Ok(())}
            let stream=state.card_rng.as_ref().ok_or_else(||InputError::Invariant("random upgrade card stream absent".into()))?;
            let (index,next)=super::rng::uniform_below(stream,candidates.len() as u128)?;state.card_rng=Some(next);
            let instance=candidates[index as usize];
            let card=state.piles.hand.iter().filter_map(OwnedCard::full).find(|card|card.instance==instance)
                .ok_or_else(||InputError::Invariant("random upgrade hand owner absent".into()))?.clone();
            let upgraded=super::card_state::upgrade_live_card(state,&card)
                .map_err(|error|InputError::Unsupported(format!("relic hand upgrade failed: {error:?}")))?;
            *state.piles.hand.iter_mut().filter_map(OwnedCard::full_mut).find(|card|card.instance==instance)
                .ok_or_else(||InputError::Invariant("random upgrade hand slot absent".into()))?=upgraded;
            super::piles::refresh_hand_values(state);
        }
        EffectKind::Relic(RelicEffect::CapEnemyHp{percent})=>{
            for enemy in state.enemies.iter_mut().filter(|enemy|enemy.targetable()){
                let cap=(i64::from(enemy.max_hp)*i64::from(*percent)/100) as i32;
                enemy.hp=enemy.hp.min(cap);
            }
        }
        EffectKind::Relic(RelicEffect::SetRandomHandCardCostZeroThisTurn)=>{
            let mut unavailable=std::collections::BTreeSet::new();
            if let Some(card)=state.in_play.as_ref(){unavailable.insert(card.handle());}
            if let Some(context)=state.play_context.as_ref(){unavailable.insert(context.card.clone());}
            for (owner,card) in &state.queued_autoplay { unavailable.insert(owner.clone());unavailable.insert(card.handle()); }
            for frame in &state.play_frames {
                unavailable.insert(frame.child.clone());
                if let Some(card)=frame.parent.as_ref(){unavailable.insert(card.handle());}
                if let super::model::PlayFrameParentOwner::Card(owner)=&frame.parent_owner{unavailable.insert(owner.clone());}
            }
            let candidates=state.piles.hand.iter().filter_map(OwnedCard::full).filter_map(|card|{
                let base=super::cards::base_cost(state,card)?;
                let current=super::card_state::effective_cost(card,base).ok()?;
                (!unavailable.contains(&super::card_state::handle(card.instance))
                    &&base>0&&current>0&&!card.local.x_cost&&!card.local.free_to_play_once
                    &&!card.local.free_play).then_some(card.instance)
            }).collect::<Vec<_>>();
            if candidates.is_empty(){return Ok(())}
            let stream=state.card_rng.as_ref().ok_or_else(||InputError::Invariant("Mummified Hand card stream absent".into()))?;
            let (index,next)=super::rng::uniform_below(stream,candidates.len() as u128)?;state.card_rng=Some(next);
            let instance=candidates[index as usize];
            let card=state.piles.hand.iter_mut().filter_map(OwnedCard::full_mut)
                .find(|card|card.instance==instance)
                .ok_or_else(||InputError::Invariant("Mummified Hand selected card absent".into()))?;
            card.local.turn_cost=Some(0);
            super::piles::refresh_hand_values(state);
        }
        EffectKind::Relic(RelicEffect::RemovePlayerDebuffs)=>{
            let mut remove=std::collections::BTreeSet::new();
            for power in &state.player_full_powers {
                if super::powers::cleanse_debuff(power)==Some(true){remove.insert(power.id.0);}
            }
            for power in &state.player.powers {
                let full=super::FullPowerEntry{id:super::FullPowerId(power.id.as_str()),state:power.state.clone()};
                if super::powers::cleanse_debuff(&full)==Some(true){remove.insert(full.id.0);}
            }
            for id in remove.into_iter().rev(){
                state.pending.push_front(Effect{source:effect.source.clone(),target:Some(Actor::Player),
                    kind:EffectKind::RemoveFullPower(super::FullPowerId(id))});
            }
        }
        EffectKind::Relic(RelicEffect::GamblingChipSelect)=>{
            let Source::Relic(crate::content::RelicId::GamblingChip)=&effect.source else{
                return Err(InputError::Invariant("Gambling Chip selection lost its relic parent".into()).into())};
            // M27 seam: the owned relic opens the existing optional manual-discard selection.
            let candidates=state.piles.hand.iter().map(|card|card.handle()).collect::<Vec<_>>();
            if !candidates.is_empty(){begin_selection_range(state,super::selection::SelectionParent::Relic(
                crate::content::RelicId::GamblingChip),super::selection::SelectionSourceZone::Hand,
                candidates.clone(),0,candidates.len(),super::selection::SelectionDisposition::Discard,
                super::selection::EmptySelection::Continue)?;}
        }
        EffectKind::Relic(RelicEffect::RetainHand)=>{
            // M27 seam: Runic Pyramid shares Equilibrium's non-Ethereal retain set.
            let mut retained=state.piles.hand.iter().filter(|card|card.full().is_some_and(|live|!live.local.ethereal))
                .map(|card|card.handle()).collect::<std::collections::BTreeSet<_>>();
            if let Some(existing)=state.temporary_retain.as_ref().filter(|retain|retain.turn==state.turn){
                retained.extend(existing.cards.iter().cloned());
            }
            state.temporary_retain=Some(super::model::TemporaryRetain{turn:state.turn,cards:retained});
        }
        EffectKind::Potion(super::PotionEffect::UpgradeHand) => {
            for index in 0..state.piles.hand.len() {
                if let Some(card)=state.piles.hand[index].full().filter(|card|super::cards::ironclad::rules::can_upgrade(card.id,card.upgrades)).cloned() {
                    let upgraded=super::card_state::upgrade_live_card(state,&card)
                        .map_err(|error|InputError::Unsupported(format!("accepted hand upgrade failed: {error:?}")))?;
                    if let Some(slot)=state.piles.hand[index].full_mut(){*slot=upgraded;}
                }
            }
            super::piles::refresh_hand_values(state);
        }
        EffectKind::Potion(super::PotionEffect::RandomizeHandCosts) => {
            let eligible=state.piles.hand.iter().filter_map(|owned|owned.full()).filter_map(|card|{
                let base=super::cards::base_cost(state,card)?;
                let cost=card.local.mutable_combat_cost.or(card.local.randomized_cost).unwrap_or(base);
                (!card.local.x_cost&&cost>=0).then_some((card.instance,cost))
            }).collect::<Vec<_>>();
            for (instance,old) in eligible {
                let stream=state.card_rng.as_ref().ok_or_else(||InputError::Invariant("Snecko Oil card stream absent".into()))?;
                let (cost,next)=super::rng::uniform_below(stream,4)?;
                state.card_rng=Some(next);
                let card=state.piles.hand.iter_mut().filter_map(super::model::OwnedCard::full_mut)
                    .find(|card|card.instance==instance).ok_or_else(||InputError::Invariant("Snecko Oil hand owner absent".into()))?;
                // Source RandomizeHandCostAction clears freeToPlayOnce for every eligible card.
                card.local.free_play=false;card.local.free_to_play_once=false;
                if cost as i32!=old {
                    card.local.randomized_cost=Some(cost as i32);card.local.turn_cost=Some(cost as i32);
                    if card.local.mutable_combat_cost.is_some(){card.local.mutable_combat_cost=Some(cost as i32);}
                    card.local.temporary_cost=0;
                }
            }
        }
        EffectKind::Potion(super::PotionEffect::GamblersBrewSelectDiscard) => {
            let Source::Potion{slot,id:crate::content::PotionId::GamblersBrew}=&effect.source else{
                return Err(InputError::Invariant("Gambler's Brew selection lost its potion parent".into()).into())};
            let candidates=state.piles.hand.iter().map(|card|card.handle()).collect::<Vec<_>>();
            if !candidates.is_empty(){begin_selection_range(state,super::selection::SelectionParent::Potion{
                slot:*slot,id:crate::content::PotionId::GamblersBrew},super::selection::SelectionSourceZone::Hand,
                candidates.clone(),0,candidates.len(),super::selection::SelectionDisposition::Discard,
                super::selection::EmptySelection::Continue)?;}
        }
        EffectKind::Potion(super::PotionEffect::LiquidMemories{requested}) => {
            let Source::Potion{slot,id:crate::content::PotionId::LiquidMemories}=&effect.source else{
                return Err(InputError::Invariant("Liquid Memories action lost its potion parent".into()).into())};
            let candidates=state.piles.discard.iter().map(|card|card.handle()).collect::<Vec<_>>();
            let requested=usize::from(*requested);
            if requested==0||requested>2{return Err(InputError::Invariant("Liquid Memories request is outside its potion potency".into()).into())}
            if candidates.len()>requested {
                begin_selection_range(state,super::selection::SelectionParent::Potion{
                    slot:*slot,id:crate::content::PotionId::LiquidMemories},super::selection::SelectionSourceZone::Discard,
                    candidates,requested,requested,super::selection::SelectionDisposition::Hand,
                    super::selection::EmptySelection::Continue)?;
                state.pending_selection.as_mut().unwrap().operation=super::selection::SelectionOperation::RetrieveMemories;
            }else{
                for card in candidates.into_iter().rev(){state.pending.push_front(Effect{source:effect.source.clone(),target:Some(Actor::Player),
                    kind:EffectKind::Potion(super::PotionEffect::LiquidMemoriesRetrieve{card})});}
            }
        }
        EffectKind::Potion(super::PotionEffect::LiquidMemoriesRetrieve{card})=>{
            super::piles::retrieve_from_discard(state,card)?;
        }
        EffectKind::Potion(work @ (super::PotionEffect::DistilledChaosTopCard{..}
            |super::PotionEffect::DistilledChaosQueuedChild{..}|super::PotionEffect::DuplicationRepeat{..}))=>
            super::cards::ironclad::autoplay::apply_potion_work(state,effect,work)?,
        EffectKind::Potion(super::PotionEffect::EntropicBrew { character, count }) => {
            let stream = state.card_rng.as_ref().ok_or_else(||
                InputError::Invariant("Entropic Brew card-random stream absent".into()))?;
            let (proposals, next) = crate::items::potion_generation::sample_many(
                *character, true, *count, stream)?;
            state.card_rng = Some(next);
            for id in proposals.into_iter().rev() {
                state.pending.push_front(Effect {
                    source: effect.source.clone(),
                    target: None,
                    kind: EffectKind::ObtainPotion { id },
                });
            }
        }
        EffectKind::ObtainPotion { id } => {
            let Some(items) = state.item_context.as_mut() else {
                return Err(InputError::Invariant("generated potion obtain input invalid".into()).into());
            };
            if items.owns_relic(crate::content::RelicId::Sozu) {
                items.validate_potion_obtain_input(*id)?;
            } else {
                items.obtain_potion(*id)?;
            }
        }
        EffectKind::SilentItem(super::SilentItemEffect::SpecimenPoisonTransfer { amount, .. }) => {
            // The Death callback only captures Poison and the fact that a live
            // recipient existed. Like the source random action, this chooses
            // from the then-current living set when its queued action executes.
            let candidates=state.enemies.iter().filter(|enemy|
                enemy.life==super::ActorLife::Active&&enemy.hp>0)
                .map(|enemy|enemy.instance_id.clone()).collect::<Vec<_>>();
            if candidates.is_empty(){return Ok(())}
            let stream=state.card_rng.as_ref().ok_or_else(||InputError::Invariant(
                "Specimen card-random stream absent".into()))?;
            let (index,next)=super::rng::uniform_below(stream,candidates.len() as u128)?;
            state.card_rng=Some(next);
            state.pending.push_front(Effect{source:Source::Actor(Actor::Player),
                target:Some(Actor::Enemy(candidates[index as usize].clone())),
                kind:EffectKind::ApplyFullPower{id:super::FullPowerId("poison"),amount:*amount}});
        }
        EffectKind::GainBlock(n) => {
            // Deca and Shield publish their full group, not a living-only
            // snapshot. Already queued Block survives source death but skips a
            // dead target. HalfDead is not ordinary death.
            if state.full_runtime
                &&matches!(&effect.source,Source::Actor(Actor::Enemy(owner)) if state.enemies.iter().any(|e|&e.instance_id==owner&&matches!(e.enemy_id,ActorEnemyId::Full(crate::content::EnemyId::Deca|crate::content::EnemyId::SpireShield))))
                &&matches!(&effect.target,Some(Actor::Enemy(target)) if state.enemies.iter().any(|e|&e.instance_id==target&&e.hp==0&&e.life!=ActorLife::HalfDead)){return Ok(())}
            let full_runtime = state.full_runtime;
            let block_target=effect.target.as_ref().unwrap().clone();
            let block = match &block_target {
                Actor::Player => &mut state.player.block,
                Actor::Enemy(id) => &mut state.enemies.iter_mut().find(|e| &e.instance_id == id).expect("validated actor").block,
            };
            let before=*block;
            *block = if full_runtime {
                rules::full_block_after_add(*block, *n)?
            } else {
                rules::block_after_add(*block, *n)?
            };
            let after=*block;
            if full_runtime{
                super::cards::ironclad::reactive::after_block(state,&block_target,*n,before,after)?;
                super::powers::player_block_gained(state,&block_target,after-before);
            }
        }
        EffectKind::Heal(n) => {
            if let Some(Actor::Enemy(owner))=&effect.target {
                let enemy=state.enemies.iter_mut().find(|e|&e.instance_id==owner).expect("validated heal owner");
                let before=enemy.hp;
                if enemy.life==super::ActorLife::Active&&enemy.hp>0 {enemy.hp=(i64::from(enemy.hp)+i64::from(*n)).min(i64::from(enemy.max_hp)) as i32;}
                let after=enemy.hp;
                super::observe::emit(state, &effect.source, effect.target.as_ref(), ||
                    ResolvedEffectKind::Heal { hp_before:before, hp_after:after, amount:after-before });
                return Ok(());
            }
            heal_player(state,effect,*n,false)?;
        }
        EffectKind::NeowsLament=>{
            for enemy in state.enemies.iter_mut().filter(|e|e.hp>0&&e.life==super::ActorLife::Active){enemy.hp=1;}
        }
        EffectKind::IncreaseMaxHpThenHeal(n) => {
            // Commit the max-HP child first, then schedule the ordinary Heal
            // as the next queue item with the same effect owner.
            let max_hp = state.player.max_hp.checked_add(*n)
                .ok_or_else(|| InputError::Unsupported("player maximum HP overflow".into()))?;
            state.player.max_hp = max_hp;
            if let Some(context)=state.item_context.as_mut(){
                let work=crate::items::laws::hp_changed(context,0,state.player.hp,state.player.max_hp)?;
                for child in work.into_iter().rev(){super::engine::validate_effect(&child,state)?;state.pending.push_front(child);}
            }
            state.pending.push_front(Effect { source: effect.source.clone(),
                target: Some(Actor::Player), kind: EffectKind::Heal(*n) });
        }
        EffectKind::GainEnergy(n) => {
            gain_energy(state,*n,super::cards::defect::capped_energy_source(state,&effect.source,*n)||super::cards::watcher::capped_energy_source(state,&effect.source,*n))?;
        }
        EffectKind::LoseEnergy(n)=>{state.player.energy=(state.player.energy-*n).max(0);},
        EffectKind::ApplyPower { id, amount: delta } => {
            let grace = matches!(effect.source, Source::Actor(Actor::Enemy(_)));
            let powers = powers_mut(state, effect.target.as_ref().unwrap());
            let previous = amount(powers, *id);
            let total = i64::from(previous).checked_add(i64::from(*delta))
                .ok_or_else(|| InputError::Invariant("power sum overflow".into()))?;
            let next = if *id == PowerId::Strength { total.min(999) } else { total };
            let next = i32::try_from(next).map_err(|_| InputError::Invariant("power amount overflow".into()))?;
            if let Some(entry) = powers.iter_mut().find(|p| p.id == *id) {
                entry.state.amount = next;
            } else {
                powers.push(PowerEntry { id: *id, state: PowerState { amount: next,
                    skip_first: *id == PowerId::Ritual,
                    monster_source_grace: *id == PowerId::Vulnerable && grace } });
            }
            refresh_enemy_values(state)?;
        }
        EffectKind::ReducePower { id, amount: delta } => {
            let powers = powers_mut(state, effect.target.as_ref().unwrap());
            if let Some(entry) = powers.iter_mut().find(|p| p.id == *id) {
                entry.state.amount = entry.state.amount.checked_sub(*delta)
                    .ok_or_else(|| InputError::Invariant("power reduction overflow".into()))?.max(0);
                if entry.state.amount == 0 {
                    state.pending.push_front(Effect { source: effect.source.clone(), target: effect.target.clone(),
                        kind: EffectKind::RemovePower(*id) });
                }
            }
            refresh_enemy_values(state)?;
        }
        EffectKind::RemovePower(id) => {
            powers_mut(state, effect.target.as_ref().unwrap()).retain(|p| p.id != *id);
            refresh_enemy_values(state)?;
        }
        EffectKind::ApplyFullPower { .. } | EffectKind::ReduceFullPower { .. } | EffectKind::RemoveFullPower(_) =>
            return Err(InputError::Invariant("full power outside full runtime".into()).into()),
        EffectKind::Character(action) => super::characters::apply(state, &effect.source, *action)?,
        EffectKind::ShieldBashFocus => super::enemies::act4::shield_spear::apply_shield_bash_focus(state,effect)?,
        EffectKind::CardContinuation(continuation) =>
            super::cards::apply_defect_continuation(state, effect, continuation)?,
        EffectKind::ReplaceEnemy { parent, children } =>
            super::enemies::apply_replacement(state, parent, children)?,
        EffectKind::GuardianChange(change)=>super::enemies::guardian::apply(state,effect,*change)?,
        EffectKind::HexaghostBurn(gate)=>super::enemies::hexaghost::apply(state,effect,*gate)?,
        EffectKind::GroundByrd=>super::enemies::act2::flight::ground(state,effect)?,
        EffectKind::SpawnEnemy(request)=>super::enemies::act2::spawn::apply(state,request)?,
        EffectKind::CompleteSpawn=>{
            let before=state.enemies.len();
            super::enemies::act2::spawn::complete(state)?;
            for handle in state.enemies.iter().skip(before).map(|enemy|enemy.instance_id.clone()).collect::<Vec<_>>(){
                super::enemies::dispatch_spawned(state,&handle)?;
            }
        },
        EffectKind::CaptureStasis{orb}=>super::enemies::act2::stasis::capture(state,orb)?,
        EffectKind::ReleaseStasis{orb}=>super::enemies::act2::stasis::release(state,orb)?,
        EffectKind::ChampCleanse=>{
            let Some(Actor::Enemy(owner))=&effect.target else{return Err(InputError::Invariant("Champ cleanse owner absent".into()).into())};super::enemies::act2::champ::cleanse(state,owner)?;
        },
        EffectKind::CollectorSuicide|EffectKind::AutomatonSuicide=>{
            let Some(Actor::Enemy(owner))=&effect.target else{return Err(InputError::Invariant("boss suicide target absent".into()).into())};super::enemies::act2::suicide(state,owner);
        },
        EffectKind::ShapeSuicide=>{
            let Some(Actor::Enemy(owner))=&effect.target else{return Err(InputError::Invariant("shape suicide owner absent".into()).into())};super::enemies::act2::suicide(state,owner);
        },
        EffectKind::ShapeDamageComplete=>super::enemies::act3::shapes::complete_damage(state,&effect.source),
        EffectKind::RevivalHeal(_)|EffectKind::DarklingCollapse|EffectKind::DarklingRevive|EffectKind::DarklingsDefeat=>super::enemies::act3::darklings::apply(state,effect)?,
        EffectKind::TransientFade|EffectKind::TransientRecover|EffectKind::MawNom{..}=>super::enemies::act3::ordinary::apply(state,effect)?,
        EffectKind::ObtainParasite=>super::enemies::act3::writhing::obtain(state,&effect.source)?,
        EffectKind::AwakenedCollapse|EffectKind::AwakenedRebirth|EffectKind::AwakenedCanLose=>super::enemies::act3::awakened::apply(state,effect)?,
        EffectKind::TimeWarpEndTurn=>super::enemies::act3::time_eater::force_end(state,effect)?,
        EffectKind::TimeEaterCleanse=>{let Some(Actor::Enemy(owner))=&effect.target else{return Err(InputError::Invariant("Haste cleanse target absent".into()).into())};super::enemies::act3::time_eater::cleanse(state,owner)?;},
        EffectKind::CorruptHeartBuff=>super::enemies::act4::heart::apply_buff(state,effect)?,
        EffectKind::ReptomancerSuicide=>{let Some(Actor::Enemy(owner))=&effect.target else{return Err(InputError::Invariant("Reptomancer suicide target absent".into()).into())};super::enemies::act3::reptomancer::cleanup_suicide(state,owner);},
        EffectKind::ShelledArmorBreak=>{let Some(Actor::Enemy(owner))=&effect.target else{return Err(InputError::Invariant("armor break owner absent".into()).into())};super::enemies::act2::ordinary_closure::armor_break(state,owner)?;},
        EffectKind::Thief(kind)=>super::enemies::act1::thieves::apply_effect(state,effect,kind)?,
        EffectKind::ResetMalleable=>{let Some(Actor::Enemy(owner))=&effect.target else{return Err(InputError::Invariant("Malleable reset owner absent".into()).into())};if state.enemies.iter().any(|enemy|&enemy.instance_id==owner&&enemy.enemy_id==ActorEnemyId::Full(crate::content::EnemyId::SnakePlant)){
                super::enemies::act2::ordinary_closure::reset_malleable(state,owner);
            }else{super::enemies::act3::writhing::reset_malleable(state,owner);}},
        EffectKind::BookStab{remaining}=>{
            let Some(Actor::Enemy(owner))=&effect.target else{return Err(InputError::Invariant("Book stab owner absent".into()).into())};super::enemies::act2::book::stab(state,owner,*remaining);
        },
        EffectKind::RallyGremlins{..}|EffectKind::ProtectGremlin|EffectKind::SetEnemyMove{..}|EffectKind::EscapeEnemy=>{
            let Some(Actor::Enemy(owner))=&effect.target else{return Err(InputError::Invariant("Gremlin continuation owner absent".into()).into())};
            match effect.kind {
                EffectKind::RallyGremlins{remaining}=>super::enemies::act2::gremlin_leader::rally(state,owner,remaining)?,
                EffectKind::ProtectGremlin=>super::enemies::act2::gremlin_leader::protect(state,owner)?,
                EffectKind::SetEnemyMove{intent}=>super::enemies::act2::gremlin_leader::set_move(state,owner,intent)?,
                EffectKind::EscapeEnemy=>super::enemies::act2::gremlin_leader::escape(state,owner),_=>unreachable!(),
            }
        },
        EffectKind::Decision => {
            if !state.pending.is_empty() || state.in_play.is_some() || state.player.hp <= 0
                || state.enemies.iter().all(|e|e.hp<=0&&e.life!=ActorLife::HalfDead) || state.outcome.is_some() {
                return Err(InputError::Invariant("decision before quiescence".into()).into());
            }
            refresh_enemy_values(state)?;
            state.phase = Phase::Decision;
        }
        EffectKind::FinishCard => {
            let Source::Card(card) = &effect.source else {
                return Err(InputError::Invariant("FinishCard requires card source".into()).into());
            };
            if state.full_runtime{super::enemies::act3::elites::after_card(state)?;}
            if state.full_runtime{super::enemies::act3::time_eater::after_card(state)?;}
            if state.in_play.as_ref().map(OwnedCard::handle).as_ref()!=Some(card){return Err(InputError::Invariant("FinishCard in-play owner missing".into()).into())}
            if let Some(live)=state.in_play.as_mut().and_then(OwnedCard::full_mut){live.local.free_to_play_once=false;}
            if state.full_runtime&&super::cards::ironclad::autoplay::defer_parent_finish(state,card)?{return Ok(())}
            let disposition=state.play_context.as_ref().filter(|context|context.card==*card)
                .map_or(super::model::FinishDisposition::Natural,|context|context.disposition);
            let owned=state.in_play.take().ok_or_else(||InputError::Invariant("finished card owner missing".into()))?;
            super::piles::finish_played_card(state,owned,disposition)?;
            if state.full_runtime{super::cards::ironclad::autoplay::resume_parent(state,card)?;}
        }
        EffectKind::Terminal(outcome) => {
            if !state.full_runtime || state.hp_before_healing.is_none() {state.hp_before_healing = Some(state.player.hp);}
            let before=state.player.hp;
            if *outcome == Outcome::Victory && !state.full_runtime {
                state.player.hp = i32::try_from((i64::from(state.player.hp) + 6)
                    .min(i64::from(state.player.max_hp)))
                    .map_err(|_| InputError::Invariant("terminal HP overflow".into()))?;
                super::observe::emit(state, &Source::None, Some(&Actor::Player), ||
                    ResolvedEffectKind::Heal { hp_before:before, hp_after:state.player.hp, amount:state.player.hp-before });
            }
            state.pending.clear();
            state.nightmare_templates.clear();
            state.colorless=super::model::ColorlessState::default();
            state.pending_time_warp=None;
            // Do not run FinishCard or reset diagnostic piles/block. dispatch_one
            // releases current after this synchronous terminal payload completes.
            state.phase = Phase::Terminal;
            state.outcome = Some(*outcome);
            state.hp_after_cleanup = Some(state.player.hp);
        }
        EffectKind::SelectCard { candidates, empty } => {
            if candidates.is_empty() {
                match empty { EmptySelection::Continue => (), }
            } else {
                let Source::Card(parent) = &effect.source else {
                    return Err(InputError::Invariant("selection parent is not a card".into()).into());
                };
                begin_selection(state, parent.clone(), candidates.clone(), 1,
                    super::selection::SelectionDisposition::Hand, *empty)?;
            }
        }
        EffectKind::SelectNightmareCard => {
            let Source::Card(parent) = &effect.source else {
                return Err(InputError::Invariant("Nightmare selection parent is not a card".into()).into());
            };
            if state.piles.hand.is_empty() { return Ok(()); }
            let candidates = state.piles.hand.iter().map(|card| card.handle()).collect::<Vec<_>>();
            begin_selection(state, parent.clone(), candidates, 1,
                super::selection::SelectionDisposition::Hand, super::selection::EmptySelection::Continue)?;
            state.pending_selection.as_mut().expect("nonempty Nightmare hand creates selection").operation =
                super::selection::SelectionOperation::NightmareCapture;
        }
        EffectKind::NightmareCopy { owner, installation_id } =>
            super::cards::apply_silent_nightmare_copy(state,effect,owner,*installation_id)?,
        EffectKind::SelectHandCardsAfterDraw { required, disposition, empty } => {
            let parent=match &effect.source {
                Source::Card(parent)=>super::selection::SelectionParent::Card(parent.clone()),
                Source::Potion{slot,id:crate::content::PotionId::Elixir}
                    if super::engine::valid_potion_slot(state,*slot)&&*required==0&&*disposition==super::selection::SelectionDisposition::Exhaust
                        &&effect.target.as_ref()==Some(&Actor::Player)=>
                    super::selection::SelectionParent::Potion{slot:*slot,id:crate::content::PotionId::Elixir},
                _=>return Err(InputError::Invariant("selection parent is not an owned card or finite Elixir use".into()).into()),
            };
            let candidates = state.piles.hand.iter().map(|card| card.handle()).collect::<Vec<_>>();
            if candidates.is_empty() {
                match empty { EmptySelection::Continue => return Ok(()), }
            }
            if *required > candidates.len() {
                return Err(InputError::Invariant("invalid captured hand selection count".into()).into());
            }
            if *required==0 {
                if !candidates.is_empty(){begin_selection_range(state,parent,
                    super::selection::SelectionSourceZone::Hand,candidates,0,state.piles.hand.len(),*disposition,*empty)?;}
            }else{let super::selection::SelectionParent::Card(parent)=parent else{return Err(InputError::Invariant("required potion selection is not supported".into()).into())};begin_selection(state, parent, candidates, *required, *disposition, *empty)?;
                if *disposition==super::selection::SelectionDisposition::DrawTop{
                    state.pending_selection.as_mut().unwrap().operation=super::selection::SelectionOperation::Topdeck;
                }
            }
        }
        EffectKind::SelectCards { candidates, required, disposition, empty } => {
            let Source::Card(parent) = &effect.source else {
                return Err(InputError::Invariant("selection parent is not a card".into()).into());
            };
            if candidates.is_empty() {
                match empty {
                    EmptySelection::Continue => return Ok(()),
                }
            }
            if *required > candidates.len() { return Err(InputError::Invariant("selection count exceeds candidates".into()).into()); }
            if *required==0{begin_selection_range(state,super::selection::SelectionParent::Card(parent.clone()),infer_selection_zone(state,candidates)?,
                candidates.clone(),0,candidates.len(),*disposition,*empty)?;}else{begin_selection(state, parent.clone(), candidates.clone(), *required, *disposition, *empty)?;}
        }
        EffectKind::Silent(work)=>super::cards::apply_silent(state,effect,work)?,
        EffectKind::Alchemize{proposal}=>{
            let id=proposal.ok_or_else(||InputError::Invariant("Alchemize use-time proposal absent".into()))?;
            state.pending.push_front(Effect{source:effect.source.clone(),target:None,kind:EffectKind::ObtainPotion{id}});
        }
        EffectKind::SmokeEscape=>{
            state.silent.smoked=true;
            // Smoke Bomb only starts the escape; work already queued (Toy Ornithopter's potion-used
            // Heal) still resolves before the escape settles (source check 2026-11-01, M27).
            state.pending.push_back(Effect{source:effect.source.clone(),target:None,kind:EffectKind::Hook(Hook::CombatSettled)});
            state.pending.push_back(Effect{source:effect.source.clone(),target:None,kind:EffectKind::Terminal(Outcome::Escaped)});
        }
        EffectKind::Generate(work)=>super::generation::apply(state,effect,work)?,
        EffectKind::Defect(work)=>super::cards::defect::apply(state,effect,work)?,
        EffectKind::Colorless(work)=>super::cards::apply_colorless(state,effect,work)?,
        EffectKind::OpenRetainSelection{limit}=>super::cards::apply_silent_retain_selection(state,effect,*limit)?,
        EffectKind::ManualDiscardCallback{..} => apply_hook(state,effect,&Hook::CardManuallyDiscarded)?,
        EffectKind::Hook(hook) => apply_hook(state,effect,hook)?,
        EffectKind::OpeningHand{count}=>{
            let count=super::piles::opening_hand_count(state,*count);
            if count>0{let callback=super::piles::deal_opening_one(state);if count>1{state.pending.push_front(Effect{source:effect.source.clone(),target:Some(Actor::Player),kind:EffectKind::OpeningHand{count:count-1}});}if let Some(callback)=callback{state.pending.push_front(callback);}}
        },
        EffectKind::Watcher(work)=>super::cards::watcher::apply(state,effect,work)?,
        EffectKind::Ironclad(work)=>super::cards::ironclad::apply(state,effect,work)?,
        EffectKind::ExhaustBatchStart{filter,random,finish}=>super::exhaust::begin_batch(state,effect,*filter,*random,finish.clone())?,
        EffectKind::ExhaustBatchContinue(batch)=>super::exhaust::continue_batch(state,effect,batch)?,
        EffectKind::Draw(_) | EffectKind::DrawOne | EffectKind::AfterCardDrawn{..} | EffectKind::Recycle | EffectKind::ShuffleDraw | EffectKind::DiscardHand
        | EffectKind::CreateCard { .. } | EffectKind::CopyCard { .. } | EffectKind::ExhaustHandCard { .. } => {
            super::piles::apply_effect(state, effect)?;
        }
        EffectKind::EndPlayerTurn |
        EffectKind::MonsterStart | EffectKind::EnemyMove | EffectKind::RollMove | EffectKind::EndRound => {
            super::turns::apply_effect(state, effect)?;
        }
        EffectKind::StartNextPlayer => super::turns::apply_effect(state, effect)?,
    }
    Ok(())
}

fn heal_player(state:&mut State,effect:&Effect,amount:i32,rescue:bool)->Result<(),EffectError> {
    let before=state.player.hp;
    if state.player.hp > 0 || rescue && state.player.hp==0 {
        let amount = if state.full_runtime {
            crate::items::ordinary_heal_amount_in_room(
                state.item_context.as_ref().map_or(&[], |context| context.relics()), amount,
                state.item_context.as_ref().map_or(
                    crate::items::NativeHealRoom::default(), |context| context.heal_room()))
        } else { amount };
        let amount=if state.full_runtime&&!rescue {
            // M27 seam: heal-amount
            state.item_context.as_ref().map_or(amount,|context|crate::items::laws::heal_amount(context,amount))
        }else{amount};
        state.player.hp = i32::try_from((i64::from(state.player.hp) + i64::from(amount))
            .min(i64::from(state.player.max_hp)))
            .map_err(|_| InputError::Invariant("healing overflow".into()))?;
    }
    if state.full_runtime { if let Some(context)=state.item_context.as_mut(){
        let work=crate::items::laws::hp_changed(context,0,state.player.hp,state.player.max_hp)?;
        for child in work.into_iter().rev(){super::engine::validate_effect(&child,state)?;state.pending.push_front(child);}
    }}
    super::observe::emit(state, &effect.source, Some(&Actor::Player), ||
        ResolvedEffectKind::Heal { hp_before:before, hp_after:state.player.hp, amount:state.player.hp-before });
    Ok(())
}

pub(super) fn ordinary_heal_now(state:&mut State,source:Source,amount:i32)->Result<(),EffectError>{
    apply_effect(state,&Effect{source,target:Some(Actor::Player),kind:EffectKind::Heal(amount)})
}

pub(in crate::combat) fn begin_selection(state: &mut State, parent: super::model::CardHandle,
    candidates: Vec<super::model::CardHandle>, required: usize,
    disposition: super::selection::SelectionDisposition,
    empty: super::selection::EmptySelection) -> Result<(), EffectError> {
    let source_zone=infer_selection_zone(state,&candidates)?;
    begin_selection_range(state,super::selection::SelectionParent::Card(parent),source_zone,candidates,required,required,disposition,empty)
}

fn infer_selection_zone(state:&State,candidates:&[super::model::CardHandle])->Result<super::selection::SelectionSourceZone,EffectError>{
    let zones=[(&state.piles.hand,super::selection::SelectionSourceZone::Hand),(&state.piles.draw,super::selection::SelectionSourceZone::Draw),
        (&state.piles.discard,super::selection::SelectionSourceZone::Discard),(&state.piles.exhaust,super::selection::SelectionSourceZone::Exhaust)];
    let mut found=None;
    for candidate in candidates{
        let zone=zones.iter().find_map(|(pile,zone)|pile.iter().any(|card|card.handle()==*candidate).then_some(*zone))
            .ok_or_else(||InputError::Invariant("selection candidate is not in a selectable zone".into()))?;
        if found.is_some_and(|prior|prior!=zone){return Err(InputError::Invariant("selection candidates span multiple source zones".into()).into())}
        found=Some(zone);
    }
    Ok(found.unwrap_or(super::selection::SelectionSourceZone::Hand))
}

pub(in crate::combat) fn begin_selection_range(state:&mut State,parent:super::selection::SelectionParent,
    source_zone:super::selection::SelectionSourceZone,candidates:Vec<super::model::CardHandle>,minimum:usize,maximum:usize,
    disposition:super::selection::SelectionDisposition,empty:super::selection::EmptySelection)->Result<(),EffectError>{
    let mut seen = std::collections::BTreeSet::new();
    if candidates.iter().any(|id| !seen.insert(id.clone()))
        || candidates.iter().any(|id| !state.piles.all(None).any(|card| card.handle() == *id)) {
        return Err(InputError::Invariant("selection candidates are duplicate or unowned".into()).into());
    }
    match &parent{
        super::selection::SelectionParent::Card(card) if state.in_play.as_ref().map(OwnedCard::handle).as_ref()==Some(card)=>(),
        super::selection::SelectionParent::Potion{slot,..} if state.in_play.is_none()&&super::engine::valid_potion_slot(state,*slot)=>(),
        super::selection::SelectionParent::Relic(crate::content::RelicId::GamblingChip)
            if state.full_runtime&&state.phase==Phase::PlayerStart
                &&state.pending.iter().any(|effect|effect.kind==EffectKind::Decision)
                &&state.item_context.as_ref().is_some_and(|items|items.complete_items()
                    &&items.owns_relic(crate::content::RelicId::GamblingChip)
                    &&items.local(crate::content::RelicId::GamblingChip)==1)=>(),
        super::selection::SelectionParent::Power{turn,id} if state.full_runtime&&*turn==state.turn&&(
            (id.0=="tools_of_the_trade"||(super::cards::watcher::cards_active(state)&&id.0=="foresight"))
                &&state.phase==Phase::PlayerStart&&state.in_play.is_none()
                &&state.player_full_powers.iter().any(|p|p.id==*id&&p.state.amount>0)
            ||id.0=="melange"&&super::cards::watcher::melange_scry_source(state))=>(),
        super::selection::SelectionParent::EndTurn{turn} if state.full_runtime&&*turn==state.turn
            &&state.phase==super::Phase::EnemyTurn&&state.in_play.is_none()
            &&state.pending.iter().filter(|effect|effect.kind==EffectKind::DiscardHand).count()==1=>(),
        _=>return Err(InputError::Invariant("selection parent is not owned by the active continuation".into()).into()),
    }
    if minimum>maximum||maximum==0||maximum>candidates.len(){
        return Err(InputError::Invariant("invalid selection minimum/maximum".into()).into());
    }
    let candidate_zone=infer_selection_zone(state,&candidates)?;
    if candidate_zone!=source_zone{return Err(InputError::Invariant("selection candidates do not match declared source zone".into()).into())}
    // Any selection suspends whatever continuation is resolving; the shared
    // record is the only owner of where every commit path resumes.
    let resume=super::selection::suspend(state);
    super::selection::validate_resume(state,&resume)?;
    state.pending_selection = Some(PendingSelection { parent,source_zone,resume,candidates,selected: Vec::new(),operation:SelectionOperation::Move(disposition),
        minimum,maximum, disposition, empty });
    state.phase = Phase::Selection;
    Ok(())
}

/// Shared gain owner; source-specific producers decide the qualified local cap.
pub(in crate::combat) fn gain_energy(state:&mut State,amount:i32,capped:bool)->Result<(),InputError>{
    state.player.energy=if capped{state.player.energy.saturating_add(amount).min(999)}
        else{state.player.energy.checked_add(amount).filter(|energy|*energy<=999).ok_or_else(||InputError::Unsupported("player energy overflow".into()))?};
    Ok(())
}

fn apply_hook(state:&mut State,effect:&Effect,hook:&Hook)->Result<(),EffectError>{
    if state.full_runtime {
        // Darkling downing calls the same source death callbacks without
        // becoming final actor death or combat settlement.
        let callback=if *hook==Hook::HalfDeath{Hook::Death}else{*hook};
        if *hook==Hook::CombatSettled{state.hp_before_healing=Some(state.player.hp);}
        super::hooks::dispatch(state, callback, &effect.source,effect.target.as_ref())?;
        let hp = state.player.hp;

        if *hook==Hook::CombatSettled&&hp>0 {
            if let Some(items)=state.item_context.as_mut().filter(|items|items.collar_activated){
                state.player.max_energy=state.player.max_energy.checked_sub(1).filter(|n|*n>0).ok_or_else(||InputError::Invariant("Collar cleanup master energy invalid".into()))?;
                items.collar_activated=false;
            }
        }
        // The one relic phase dispatch for this checkpoint (M27 design §6.2). Battle-start
        // enemy debuffs and Brimstone's monster Strength are their relics' own work (G4).
        // Every relic all-enemy producer reads live_enemies, the shared targetable() set.
        let enemies=state.enemies.iter().map(|e|e.instance_id.clone()).collect::<Vec<_>>();
        let live_enemies=state.enemies.iter().filter(|enemy|enemy.targetable())
            .map(|enemy|enemy.instance_id.clone()).collect::<Vec<_>>();
        let has_empty_orb=state.character.orb_slots.iter().any(Option::is_none);
        let mut facts=crate::items::dispatch::PhaseFacts::new(Some(callback),hp,state.player.max_hp,state.player.block);
        facts.target=effect.target.as_ref();facts.source=Some(&effect.source);
        facts.enemies=&enemies;facts.live_enemies=&live_enemies;facts.has_empty_orb=has_empty_orb;
        if matches!(callback,Hook::CardDrawn|Hook::CardPlayed|Hook::CardAfterUse|Hook::CardManuallyDiscarded){
            facts.card=phase_card_facts(state,&effect.source);
        }
        if matches!(callback,Hook::CombatStarted|Hook::CombatStartedPreDraw){fill_combat_entry_facts(&mut facts,state);}
        let preserve_finite_startup_order=callback==Hook::CombatStarted
            &&state.item_context.as_ref().is_some_and(|items|items.m20_profile()||items.m21_profile());
        let mut work = if let Some(context) = state.item_context.as_mut() {
            crate::items::relic_phase_effects(context,&facts)?
        } else {
            crate::items::relic_phase_effects(&mut crate::items::CombatItemContext::default(),&facts)?
        };
        super::cards::watcher::capture(state,&mut work.top)?;
        super::cards::watcher::capture(state,&mut work.bottom)?;
        if work.direct_strength!=0{super::powers::add_direct_strength(state,work.direct_strength)?;}
        queue_relic_work_with_startup_order(state,work,preserve_finite_startup_order)?;
        state.hook_events.push(HookEvent { hook: *hook,
            source: effect.source.clone(), target: effect.target.clone() });
    }
    Ok(())
}
