//! Bounded work operations for the single shared card/effect resolver.
use super::*;
use crate::combat::{Actor,Effect,EffectError,Hook,InputError,model::{FinishDisposition,OwnedCard,PlayContext,PlayFrame,PlayFrameParentOwner,State}};
use crate::content::{CardType,Catalog};

pub(in crate::combat) fn prepare_child_work(state:&State,child:&LiveCard,target:Option<&Actor>,source:&crate::combat::CardHandle,energy_on_use:i32)
    ->Result<Option<Vec<Effect>>,EffectError>{
    match super::super::prepare_autoplay(state,child,target,energy_on_use,source){
        Ok(prepared)=>Ok(prepared),
        Err(crate::run::RunError::InvalidAction)=>Ok(None),
        Err(crate::run::RunError::Unsupported(message))=>Err(InputError::Unsupported(format!("autoplay child handler unavailable: {message}")).into()),
        Err(error)=>Err(InputError::Unsupported(format!("autoplay child preparation failed: {error:?}")).into()),
    }
}

/// PlayTopCardAction (Havoc, Mayhem, Distilled Chaos). An empty draw pile is shuffled
/// from the discard and `retry` runs again; otherwise the top card is revealed, held in
/// queued-autoplay custody and refreshed (applyPowers) before it is queued
/// (source-check-outofhand-turnorder-v1). `None` when no card is taken now.
pub(in crate::combat) fn take_top_card(state:&mut State,retry:&Effect)->Result<Option<crate::combat::CardHandle>,EffectError>{
    if state.piles.draw.is_empty(){
        if state.piles.discard.is_empty(){return Ok(None)}
        state.pending.push_front(retry.clone());
        state.pending.push_front(Effect{source:retry.source.clone(),target:Some(Actor::Player),kind:EffectKind::Recycle});
        return Ok(None)
    }
    let child=state.piles.draw.pop().unwrap();let handle=child.handle();
    crate::combat::piles::drawn_knowledge(state,&child);
    if state.public.known_draw.first()==Some(&handle){state.public.known_draw.remove(0);}else{state.public.known_draw.clear();}
    state.public.known_draw_bottom.retain(|known|known!=&handle);state.public.revealed_order.push(handle.clone());
    if state.queued_autoplay.insert(handle.clone(),child).is_some(){
        return Err(InputError::Invariant("top card already has queued ownership".into()).into())
    }
    crate::combat::piles::refresh_hand_values(state);
    crate::combat::piles::refresh_queued_values(state,&handle);
    Ok(Some(handle))
}

fn capture_autoplay_energy(state:&State,requested:i32)->Result<i32,EffectError>{
    let captured=if requested==-1{state.player.energy}else{requested};
    if !(0..=999).contains(&captured){return Err(InputError::Unsupported("autoplay energy_on_use outside supported energy surface".into()).into())}
    Ok(captured)
}

pub(in crate::combat) fn start_queued_child(state:&mut State,child_handle:&crate::combat::CardHandle,target:Option<Actor>,purge:bool,
    imposed_exhaust:bool,energy_on_use:i32,parent_owner:PlayFrameParentOwner)->Result<(),EffectError>{
    if state.in_play.is_some() || state.play_context.is_some() || !state.play_frames.is_empty() {
        return Err(InputError::Invariant("queued card dispatched before the active play finished".into()).into());
    }
    match &parent_owner {
        PlayFrameParentOwner::Card(parent) if parent != child_handle && state.card_ids_issued.contains(parent) => (),
        PlayFrameParentOwner::Potion{slot,id} if *id == crate::content::PotionId::DistilledChaos
            && crate::combat::engine::valid_potion_slot(state,*slot) => (),
        _ => return Err(InputError::Invariant("queued card has an invalid producer".into()).into()),
    }
    let energy_on_use=capture_autoplay_energy(state,energy_on_use)?;
    let mut preview=state.queued_autoplay.get(child_handle).cloned()
        .ok_or_else(||InputError::Invariant("queued autoplay child custody absent".into()))?;
    let preview_live=preview.full_mut().ok_or_else(||InputError::Invariant("queued autoplay child is not a full card".into()))?;
    preview_live.local.free_play=true;preview_live.local.purge=purge;
    let live=preview.full().unwrap().clone();
    Catalog::standard().card(live.id).ok_or_else(||InputError::Unsupported("autoplay child definition absent".into()))?;
    let mut prepared=if super::super::suppress_autoplay_card(&live){None}
        else{prepare_child_work(state,&live,target.as_ref(),child_handle,energy_on_use)?};
    if let Some(effects)=prepared.as_mut(){super::super::capture_silent_targets(state,effects)?;capture_havoc_targets(state,effects)?;}
    // The queue owns the child through every fallible preparation step. The
    // producer remains in its ordinary zone, identified by the typed source.
    let mut child=state.queued_autoplay.remove(child_handle).unwrap();
    let live=child.full_mut().unwrap();live.local.free_play=true;live.local.purge=purge;
    let disposition=if purge{FinishDisposition::Purge}else if imposed_exhaust{FinishDisposition::Exhaust}
        else{super::exhaust::per_use_disposition(state,live)};
    state.play_frames.push(PlayFrame{parent_owner,child:child_handle.clone(),target:target.clone(),
        resume_phase:state.phase,captured_energy:energy_on_use,paid:0,disposition});
    state.in_play=Some(child);state.phase=crate::combat::Phase::PlayerAction;
    state.play_context=Some(PlayContext{card:child_handle.clone(),target:target.clone(),energy_on_use,paid:0,disposition});
    let source=crate::combat::Source::Card(child_handle.clone());
    let Some(effects)=prepared else{
        state.pending.push_front(Effect{source,target:None,kind:EffectKind::Ironclad(IroncladEffect::FinishSuppressed)});
        return Ok(())
    };
    let mut work=Vec::with_capacity(effects.len()+2);
    work.push(Effect{source:source.clone(),target:target.clone(),kind:EffectKind::Hook(Hook::CardPlayed)});
    work.extend(effects);work.push(Effect{source:source.clone(),target:None,kind:EffectKind::Hook(Hook::CardAfterUse)});work.push(Effect{source,target:None,kind:EffectKind::FinishCard});
    for next in work.into_iter().rev(){state.pending.push_front(next)}Ok(())
}

pub(in crate::combat) fn queued_child_is_playable(state:&State,child_handle:&crate::combat::CardHandle,target:Option<Actor>,energy_on_use:i32,purge:bool)
    ->Result<bool,EffectError>{
    let energy_on_use=capture_autoplay_energy(state,energy_on_use)?;
    let mut preview=state.queued_autoplay.get(child_handle).cloned()
        .ok_or_else(||InputError::Invariant("queued autoplay preview custody absent".into()))?;
    let live=preview.full_mut().ok_or_else(||InputError::Invariant("queued autoplay preview is not full".into()))?;
    live.local.free_play=true;live.local.purge=purge;
    Catalog::standard().card(live.id).ok_or_else(||InputError::Unsupported("autoplay child definition absent".into()))?;
    if super::super::suppress_autoplay_card(preview.full().unwrap()){return Ok(false)}
    Ok(prepare_child_work(state,preview.full().unwrap(),target.as_ref(),child_handle,energy_on_use)?.is_some())
}

/// A cancelled card-queue item whose card is not otherwise owned (Mayhem): discarded unused.
pub(in crate::combat) fn discard_cancelled_queued_card(state:&mut State,child:&crate::combat::CardHandle)->Result<(),EffectError>{
    let card=state.queued_autoplay.remove(child).ok_or_else(||InputError::Invariant("cancelled queued card custody absent".into()))?;
    dispose_unplayed(state,card,FinishDisposition::Natural)
}

/// A cancelled card-queue item retires through the use-card cleanup without its use (the original's early end:
/// `UseCardAction` with `dontTriggerOnUseCard`): purge cards vanish, a Power leaves the piles, an exhaust card
/// exhausts (Strange Spoon may save it), others are discarded (spire-d8g D3).
pub(in crate::combat) fn dispose_unplayed(state:&mut State,card:OwnedCard,disposition:FinishDisposition)->Result<(),EffectError>{
    super::super::super::piles::finish_played_card(state,card,disposition)?;
    Ok(())
}

/// These continuations stand for source cardQueue dispatch, after ordinary
/// actions. Their retained card ownership is separate from a fresh action.
pub(in crate::combat) fn is_queued_card_dispatch(kind:&EffectKind)->bool{
    matches!(kind,EffectKind::Ironclad(IroncladEffect::HavocQueued{..}|IroncladEffect::DoubleTapRepeat{..})
        |EffectKind::Defect(super::super::super::cards::DefectEffect::Repeat{..})
        |EffectKind::Silent(super::super::super::cards::SilentEffect::BurstRepeat{..})
        |EffectKind::Watcher(super::super::super::cards::watcher::WatcherEffect::OmnisciencePlay{..}|super::super::super::cards::watcher::WatcherEffect::OmniscienceRepeat{..})
        |EffectKind::Potion(crate::combat::PotionEffect::DuplicationRepeat{..}|crate::combat::PotionEffect::DistilledChaosQueuedChild{..})
        |EffectKind::Colorless(super::super::super::cards::ColorlessEffect::MayhemAutoplay{..}))
}

pub(in crate::combat) fn queue_immediate_repeat(state:&mut State,parent:&crate::combat::Source,work:Effect)->Result<(),EffectError>{
    if !state.pending.iter().any(|pending| pending.source == *parent && pending.kind == EffectKind::FinishCard) {
        return Err(InputError::Invariant("autoplay parent has no retained finish continuation".into()).into());
    }
    // The current use is already out of the card queue. An immediate repeat
    // goes at its front, after ordinary actions and before earlier card entries.
    crate::combat::enemies::act2::bottom(state, work);
    Ok(())
}

pub(in crate::combat) fn cancel_pending_card_queue(state:&mut State){
    for pending in &mut state.pending{
        match &mut pending.kind{
            EffectKind::Ironclad(IroncladEffect::HavocQueued{cancelled,..}|IroncladEffect::DoubleTapRepeat{cancelled,..})=>*cancelled=true,
            EffectKind::Defect(super::super::super::cards::DefectEffect::Repeat{cancelled,..})=>*cancelled=true,
            EffectKind::Silent(super::super::super::cards::SilentEffect::BurstRepeat{cancelled,..})=>*cancelled=true,
            EffectKind::Colorless(super::super::super::cards::ColorlessEffect::MayhemAutoplay{cancelled,..})=>*cancelled=true,
            EffectKind::Potion(crate::combat::PotionEffect::DistilledChaosQueuedChild{cancelled,..}
                |crate::combat::PotionEffect::DuplicationRepeat{cancelled,..})=>*cancelled=true,
            EffectKind::Watcher(super::super::super::cards::watcher::WatcherEffect::OmnisciencePlay{cancelled,..}
                |super::super::super::cards::watcher::WatcherEffect::OmniscienceRepeat{cancelled,..})=>*cancelled=true,
            _=>(),
        }
    }
}

pub(in crate::combat) fn capture_random_enemy_target(state:&mut State)->Result<Option<crate::combat::EnemyHandle>,InputError>{
    capture_random_enemy_target_with(state,false).map_err(|error|match error{
        EffectError::Invalid(error)=>error,
        EffectError::Random(error)=>InputError::Unsupported(format!("Havoc target RNG unavailable: {error:?}")),
    })
}

/// Blue random-hit actions call the source random(0,0) path even for a singleton.
/// The accepted legacy selector and uniform_below(bound1) contracts stay frozen.
pub(in crate::combat) fn capture_blue_random_enemy_target(state:&mut State)->Result<Option<crate::combat::EnemyHandle>,EffectError>{
    capture_random_enemy_target_with(state,true)
}
fn capture_random_enemy_target_with(state:&mut State,consume_singleton:bool)->Result<Option<crate::combat::EnemyHandle>,EffectError>{
    let mut living=state.enemies.iter().filter(|enemy|enemy.life==ActorLife::Active&&enemy.hp>0)
        .map(|enemy|enemy.instance_id.clone()).collect::<Vec<_>>();
    living.sort_by_key(|owner|state.actor_slots.get(owner).copied().unwrap_or(u8::MAX));
    if living.is_empty(){return Ok(None)}
    let stream=state.card_rng.as_ref().ok_or_else(||InputError::Invariant("Havoc card-random stream absent".into()))?;
    let (index,next)=if consume_singleton&&living.len()==1{let (_,next)=super::super::super::rng::draw_u64(stream)?;(0,next)}
        else{super::super::super::rng::uniform_below(stream,living.len() as u128)?};
    state.card_rng=Some(next);Ok(Some(living[index as usize].clone()))
}

pub(in crate::combat) fn capture_havoc_target(state:&mut State)->Result<Option<crate::combat::EnemyHandle>,InputError>{
    capture_random_enemy_target(state)
}

pub(in crate::combat) fn capture_havoc_targets(state:&mut State,effects:&mut [Effect])->Result<(),InputError>{
    for effect in effects{
        if let EffectKind::Ironclad(IroncladEffect::HavocTop{target})=&mut effect.kind{
            *target=capture_havoc_target(state)?;
        }
    }
    Ok(())
}

fn finish_suppressed(state:&mut State,source:&crate::combat::Source)->Result<(),EffectError>{
    let crate::combat::Source::Card(owner)=source else{return Err(InputError::Invariant("suppressed autoplay source is not a card".into()).into())};
    if state.in_play.as_ref().map(OwnedCard::handle).as_ref()!=Some(owner){return Err(InputError::Invariant("suppressed autoplay child custody mismatch".into()).into())}
    let child=state.in_play.take().unwrap();
    let disposition=state.play_context.as_ref().filter(|context|context.card==*owner)
        .map_or_else(||state.play_frames.last().filter(|frame|frame.child==*owner).map_or(FinishDisposition::Natural,|frame|frame.disposition),|context|context.disposition);
    // A queued card that fails canUse is retired by the same use-card cleanup (lane D Q8; spire-d8g D3).
    dispose_unplayed(state,child,disposition)?;
    finish_play(state,owner)?;
    Ok(())
}

pub(in crate::combat) fn finish_play(state:&mut State,owner:&crate::combat::CardHandle)->Result<(),InputError>{
    if let Some(frame)=state.play_frames.last(){
        if frame.child!=*owner{return Err(InputError::Invariant("finished autoplay child differs from active frame".into()))}
        let frame=state.play_frames.pop().unwrap();
        if state.outcome.is_none(){
            state.phase=if matches!(frame.parent_owner,PlayFrameParentOwner::Potion{..})
                &&frame.resume_phase==Phase::PlayerAction{Phase::Decision}else{frame.resume_phase};
        }
    }
    if state.play_context.as_ref().is_some_and(|context|context.card==*owner){state.play_context=None;}
    Ok(())
}

pub(in crate::combat) fn apply_potion_work(state:&mut State,effect:&Effect,work:&crate::combat::PotionEffect)->Result<(),EffectError>{
    match work {
        crate::combat::PotionEffect::DistilledChaosTopCard{target,energy_on_use}=>{
            let crate::combat::Source::Potion{id:crate::content::PotionId::DistilledChaos,..}=&effect.source else{
                return Err(InputError::Invariant("Distilled Chaos top action lost its potion owner".into()).into())};
            if !(0..=999).contains(energy_on_use){
                return Err(InputError::Invariant("Distilled Chaos top action lost its captured burst energy".into()).into())}
            let Some(child_handle)=take_top_card(state,effect)? else{return Ok(())};
            // Every top-card action of the burst runs before the first queued card is used.
            crate::combat::enemies::act2::queue_card(state,Effect{source:effect.source.clone(),target:None,
                kind:EffectKind::Potion(crate::combat::PotionEffect::DistilledChaosQueuedChild{
                    child:child_handle,target:target.clone(),energy_on_use:*energy_on_use,cancelled:false})});
            Ok(())
        }
        crate::combat::PotionEffect::DistilledChaosQueuedChild{child,target,energy_on_use,cancelled}=>{
            let crate::combat::Source::Potion{slot,id:crate::content::PotionId::DistilledChaos}=&effect.source else{
                return Err(InputError::Invariant("Distilled Chaos queued child lost its potion owner".into()).into())};
            if *cancelled{
                let card=state.queued_autoplay.remove(child)
                    .ok_or_else(||InputError::Invariant("queued Distilled Chaos child custody absent".into()))?;
                dispose_unplayed(state,card,FinishDisposition::Natural)?;
                return Ok(())
            }
            start_queued_child(state,child,target.clone().map(Actor::Enemy),false,false,*energy_on_use,
                PlayFrameParentOwner::Potion{slot:*slot,id:crate::content::PotionId::DistilledChaos})
        }
        crate::combat::PotionEffect::DuplicationRepeat{child,target,energy_on_use,cancelled}=>{
            let crate::combat::Source::Card(parent)=&effect.source else{
                return Err(InputError::Invariant("Duplication repeat source is not its actual card".into()).into())};
            if *cancelled{
                let card=state.queued_autoplay.remove(child)
                    .ok_or_else(||InputError::Invariant("queued Duplication child custody absent".into()))?;
                dispose_unplayed(state,card,FinishDisposition::Purge)?;
                return Ok(())
            }
            start_queued_child(state,child,target.clone(),true,false,*energy_on_use,PlayFrameParentOwner::Card(parent.clone()))
        }
        _=>Err(InputError::Invariant("non-scheduler potion work reached autoplay owner".into()).into()),
    }
}

pub(in crate::combat) fn card_played(state:&mut State,source:&Source,target:Option<&Actor>,work:&mut Vec<Effect>)->Result<(),EffectError>{
    let Source::Card(owner)=source else{return Ok(())};
    let Some((card_id,purge,x_cost))=state.in_play.as_ref().filter(|owned|owned.handle()==*owner).and_then(OwnedCard::full)
        .map(|card|(card.id,card.local.purge,card.local.x_cost))else{return Ok(())};
    let Some(definition)=Catalog::standard().card(card_id)else{return Ok(())};
    if purge||state.play_frames.last().is_some_and(|frame|frame.child==*owner&&frame.disposition==FinishDisposition::Purge){return Ok(())}
    let (energy_on_use,paid)=state.play_context.as_ref().filter(|context|context.card==*owner)
        .map_or((state.player.energy,0),|context|(context.energy_on_use,context.paid));
    let target=target.cloned();

    // UseCardAction calls player powers in list order and then relics. Every
    // listener that copies inserts at the front, so the last one dispatches first.
    let listeners:Vec<_>=state.player_full_powers.iter().filter_map(|power|
        (power.state.amount>0&&match power.id.0{
            "duplication"|"echo_form"=>true,
            "double_tap"=>definition.card_type==CardType::Attack,
            "burst"=>definition.card_type==CardType::Skill,
            "amplify"=>definition.card_type==CardType::Power,
            _=>false,
        }).then_some(power.id)).collect();
    for id in listeners {
        let index=state.player_full_powers.iter().position(|power|power.id==id)
            .ok_or_else(||InputError::Invariant("repeat power left the on-use pass".into()))?;
        if id.0=="echo_form" {
            let position=i64::try_from(state.silent.played_this_turn.len())
                .map_err(|_|InputError::Unsupported("Echo turn position exhausted".into()))?;
            if position-i64::from(state.defect.echo_doubled_this_turn)>i64::from(state.player_full_powers[index].state.amount) {
                continue;
            }
            let next=state.defect.echo_doubled_this_turn.checked_add(1)
                .ok_or_else(||InputError::Unsupported("Echo counter exhausted".into()))?;
            queue_shared_repeat(state,source,target.clone(),energy_on_use,false)?;
            state.defect.echo_doubled_this_turn=next;
            continue;
        }

        state.player_full_powers[index].state.amount-=1;
        if state.player_full_powers[index].state.amount==0 {
            if id.0=="amplify" {
                state.player_full_powers.remove(index);
            } else {
                let source=if id.0=="double_tap"{source.clone()}else{Source::Actor(Actor::Player)};
                work.push(Effect{source,target:Some(Actor::Player),kind:EffectKind::RemoveFullPower(id)});
            }
        }
        let child=allocate_repeat(state,owner,false)?;
        let target=target.clone();
        let kind=match id.0 {
            "duplication"=>EffectKind::Potion(crate::combat::PotionEffect::DuplicationRepeat{child,target,energy_on_use,cancelled:false}),
            "double_tap"=>EffectKind::Ironclad(IroncladEffect::DoubleTapRepeat{child,target,energy_on_use,cancelled:false}),
            "burst"=>EffectKind::Silent(crate::combat::cards::SilentEffect::BurstRepeat{child,target,energy_on_use,cancelled:false}),
            "amplify"=>EffectKind::Defect(crate::combat::cards::DefectEffect::Repeat{child,target,energy_on_use,cancelled:false}),
            _=>unreachable!("filtered repeat listeners"),
        };
        queue_immediate_repeat(state,source,Effect{source:source.clone(),target:None,kind})?;
    }
    if let Some(items)=state.item_context.as_mut(){
        let extra=crate::items::laws::repeat_play(items,crate::items::dispatch::PhaseCard{
            id:card_id,card_type:definition.card_type,cost_paid:if x_cost{energy_on_use}else{paid}});
        for _ in 0..extra{queue_shared_repeat(state,source,target.clone(),energy_on_use,true)?;}
    }
    Ok(())
}

pub(in crate::combat) fn apply_burst_repeat(state:&mut State,effect:&Effect,work:&super::super::super::cards::SilentEffect)
    ->Result<(),EffectError>{
    let super::super::super::cards::SilentEffect::BurstRepeat{child,target,energy_on_use,cancelled}=work else {
        return Err(InputError::Invariant("non-Burst work reached shared repeat dispatcher".into()).into());
    };
    let crate::combat::Source::Card(parent)=&effect.source else {
        return Err(InputError::Invariant("Burst repeat source is not a card".into()).into());
    };
    if *cancelled {
        let copy=state.queued_autoplay.remove(child)
            .ok_or_else(||InputError::Invariant("queued Burst child custody absent".into()))?;
        dispose_unplayed(state,copy,FinishDisposition::Purge)?;
        return Ok(())
    }
    start_queued_child(state,child,target.clone(),true,false,*energy_on_use,PlayFrameParentOwner::Card(parent.clone()))
}

pub(super) fn apply(state:&mut State,effect:&Effect,work:&IroncladEffect)->Result<(),EffectError>{
    match work{
        IroncladEffect::WhirlwindRound{remaining,hits,spend_current}=>{
            if *remaining==0{return Ok(())}
            if state.player.energy<0{return Err(InputError::Invariant("Whirlwind energy checkpoint is negative".into()).into())}
            if *spend_current{state.player.energy=0;}
            let source=effect.source.clone();
            let mut batch=Vec::with_capacity(hits.len()+1);
            for (owner,damage) in hits{
                if state.enemies.iter().any(|enemy|enemy.instance_id==*owner&&enemy.targetable()){
                    batch.push(Effect{source:source.clone(),target:Some(Actor::Enemy(owner.clone())),kind:EffectKind::Damage(DamagePayload::Fixed(*damage))});
                }
            }
            if *remaining>1{batch.push(Effect{source:source.clone(),target:None,kind:EffectKind::Ironclad(IroncladEffect::WhirlwindRound{remaining:remaining-1,hits:hits.clone(),spend_current:false})});}
            for item in batch.into_iter().rev(){state.pending.push_front(item);}
            Ok(())
        }
        IroncladEffect::WhirlwindSpend=>{if state.player.energy<0{return Err(InputError::Invariant("Whirlwind spend checkpoint is negative".into()).into())}state.player.energy=0;Ok(())},
        IroncladEffect::HavocTop{target}=>{
            state.pending.push_front(Effect{source:effect.source.clone(),target:None,kind:EffectKind::Ironclad(IroncladEffect::HavocExtract{target:target.clone()})});Ok(())
        }
        IroncladEffect::HavocExtract{target}=>{
            let Some(child_handle)=take_top_card(state,effect)? else{return Ok(())};
            let energy_on_use=state.player.energy;
            let queued=Effect{source:effect.source.clone(),target:None,kind:EffectKind::Ironclad(IroncladEffect::HavocQueued{child:child_handle,target:target.clone(),energy_on_use,cancelled:false})};
            // Like Mayhem and Distilled Chaos, revealing a top card appends it
            // behind all cards that were already queued.
            crate::combat::enemies::act2::queue_card(state, queued);
            Ok(())
        }
        IroncladEffect::DoubleTapRepeat{child,target,energy_on_use,cancelled}=>{
            let crate::combat::Source::Card(owner)=&effect.source else{return Err(InputError::Invariant("Double Tap source is not a card".into()).into())};
            if *cancelled{
                let copy=state.queued_autoplay.remove(child).ok_or_else(||InputError::Invariant("queued Double Tap child custody absent".into()))?;
                return dispose_unplayed(state,copy,FinishDisposition::Purge)
            }
            start_queued_child(state,child,target.clone(),true,false,*energy_on_use,PlayFrameParentOwner::Card(owner.clone()))
        }
        IroncladEffect::HavocQueued{child,target,energy_on_use,cancelled}=>{
            let crate::combat::Source::Card(owner)=&effect.source else{return Err(InputError::Invariant("Havoc source is not a card".into()).into())};
            let target_actor=target.clone().map(Actor::Enemy);
            if *cancelled{
                let child=state.queued_autoplay.remove(child).ok_or_else(||InputError::Invariant("queued Havoc child custody absent".into()))?;
                return dispose_unplayed(state,child,FinishDisposition::Exhaust)
            }
            start_queued_child(state,child,target_actor,false,true,*energy_on_use,PlayFrameParentOwner::Card(owner.clone()))
        }
        IroncladEffect::FinishSuppressed=>finish_suppressed(state,&effect.source),
        _=>Err(InputError::Invariant("autoplay resolver received unrelated Ironclad work".into()).into()),
    }
}

/// M40a: whether the target a repeat copies from the play of `parent` is the player's own choice. A hand play runs with no
/// play frame and its target is a public action. A nested play (Havoc, Mayhem, Distilled Chaos, Omniscience child) holds a
/// target drawn from the card stream, so its repeat does not qualify unless that child is itself recorded; any other
/// frame shape fails closed.
fn copied_target_is_public(state:&State,parent:&crate::combat::CardHandle)->bool{
    match state.play_frames.last(){
        None=>true,
        Some(frame)=>frame.child==*parent&&state.public_target_plays.contains(&frame.child),
    }
}

/// Capture the physical copy at enqueue. Necronomicon re-applies powers to its copy.
fn allocate_repeat(state:&mut State,parent:&crate::combat::CardHandle,refresh:bool)->Result<crate::combat::CardHandle,EffectError>{
    let instance=crate::combat::piles::next_generated_instance(state)?;
    let mut copied=crate::combat::piles::make_same_instance_family_copy_from_owner(state,parent,instance)?;
    copied.local.free_play=true;copied.local.purge=true;
    let child=OwnedCard::Full(copied);let handle=child.handle();
    state.card_ids_issued.insert(handle.clone());state.queued_autoplay.insert(handle.clone(),child);
    if copied_target_is_public(state,parent){state.public_target_plays.insert(handle.clone());}
    if refresh{crate::combat::piles::refresh_queued_values(state,&handle);}
    Ok(handle)
}

fn queue_shared_repeat(state:&mut State,source:&Source,target:Option<Actor>,energy_on_use:i32,refresh:bool)->Result<(),EffectError>{
    let Source::Card(owner)=source else{return Err(InputError::Invariant("repeat parent source".into()).into())};
    let child=allocate_repeat(state,owner,refresh)?;
    queue_immediate_repeat(state,source,Effect{source:source.clone(),target:None,
        kind:EffectKind::Defect(crate::combat::cards::DefectEffect::Repeat{child,target,energy_on_use,cancelled:false})})
}

pub(in crate::combat) fn apply_shared_repeat(state:&mut State,effect:&Effect,child:&crate::combat::CardHandle,target:&Option<Actor>,energy_on_use:i32,cancelled:bool)->Result<(),EffectError>{
    let crate::combat::Source::Card(parent)=&effect.source else{return Err(InputError::Invariant("repeat parent source".into()).into())};
    if cancelled {
        let card=state.queued_autoplay.remove(child).ok_or_else(||InputError::Invariant("queued repeat custody absent".into()))?;
        return dispose_unplayed(state,card,FinishDisposition::Purge)
    }
    start_queued_child(state,child,target.clone(),true,false,energy_on_use,PlayFrameParentOwner::Card(parent.clone()))
}
