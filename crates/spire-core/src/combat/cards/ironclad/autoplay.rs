//! Bounded work operations for the single shared card/effect resolver.
use super::*;
use crate::combat::{Actor,Effect,EffectError,Hook,InputError,model::{FinishDisposition,OwnedCard,PlayContext,PlayFrame,PlayFrameParentOwner,State}};
use crate::content::{CardType,Catalog};

fn prepare_child_work(state:&State,child:&LiveCard,target:Option<&Actor>,source:&crate::combat::CardHandle,energy_on_use:i32)
    ->Result<Option<Vec<Effect>>,EffectError>{
    match super::super::prepare_autoplay(state,child,target,energy_on_use,source){
        Ok(prepared)=>Ok(prepared),
        Err(crate::run::RunError::InvalidAction)=>Ok(None),
        Err(crate::run::RunError::Unsupported(message))=>Err(InputError::Unsupported(format!("autoplay child handler unavailable: {message}")).into()),
        Err(error)=>Err(InputError::Unsupported(format!("autoplay child preparation failed: {error:?}")).into()),
    }
}

pub(in crate::combat) fn start_queued_child(state:&mut State,child_handle:&crate::combat::CardHandle,target:Option<Actor>,purge:bool,
    imposed_exhaust:bool,energy_on_use:i32,parent_owner:PlayFrameParentOwner)->Result<(),EffectError>{
    let mut preview=state.queued_autoplay.get(child_handle).cloned()
        .ok_or_else(||InputError::Invariant("queued autoplay child custody absent".into()))?;
    let preview_live=preview.full_mut().ok_or_else(||InputError::Invariant("queued autoplay child is not a full card".into()))?;
    preview_live.local.free_play=true;preview_live.local.purge=purge;
    let live=preview.full().unwrap().clone();
    let definition=Catalog::standard().card(live.id).ok_or_else(||InputError::Unsupported("autoplay child definition absent".into()))?;
    let mut prepared=if matches!(definition.card_type,CardType::Status|CardType::Curse){None}
        else{prepare_child_work(state,&live,target.as_ref(),child_handle,energy_on_use)?};
    if let Some(effects)=prepared.as_mut(){super::super::capture_silent_targets(state,effects)?;capture_havoc_targets(state,effects)?;}
    // Keep the queued owner and any active parent/context in State until every
    // fallible legality and handler check has completed.
    let child=state.queued_autoplay.remove(child_handle).unwrap();
    let mut child=child;
    let live=child.full_mut().unwrap();live.local.free_play=true;live.local.purge=purge;
    let (parent,parent_context)=match &parent_owner {
        PlayFrameParentOwner::Card(parent_handle)=>{
            let parent=state.in_play.take();let parent_context=state.play_context.take();
            let identity_matches=parent.as_ref().is_some_and(|card|card.handle()==*parent_handle)
                &&parent_context.as_ref().is_some_and(|context|context.card==*parent_handle);
            if parent.is_some()!=parent_context.is_some()||parent.is_some()&&!identity_matches {
                state.queued_autoplay.insert(child_handle.clone(),child);state.in_play=parent;state.play_context=parent_context;
                return Err(InputError::Invariant("autoplay parent/context custody mismatch".into()).into())
            }
            if parent_handle==child_handle{
                state.queued_autoplay.insert(child_handle.clone(),child);state.in_play=parent;state.play_context=parent_context;
                return Err(InputError::Invariant("autoplay parent and child identity collision".into()).into())
            }
            (parent,parent_context)
        }
        PlayFrameParentOwner::Potion{slot,id}=>{
            if state.in_play.is_some()||state.play_context.is_some()||!crate::combat::engine::valid_potion_slot(state,*slot)||*id!=crate::content::PotionId::DistilledChaos {
                state.queued_autoplay.insert(child_handle.clone(),child);
                return Err(InputError::Invariant("potion autoplay parent is not a live Distilled Chaos owner".into()).into())
            }
            (None,None)
        }
    };
    let live=child.full().unwrap();
    let disposition=if purge{FinishDisposition::Purge}else if imposed_exhaust{FinishDisposition::Exhaust}
        else{super::exhaust::per_use_disposition(state,live)};
    let already_deferred=matches!(&parent_owner,PlayFrameParentOwner::Card(_))
        &&state.play_frames.last().is_some_and(|frame|frame.child==*child_handle&&frame.parent_finish_checkpoint_done);
    if already_deferred{
        if parent.is_some()||parent_context.is_some(){return Err(InputError::Invariant("deferred autoplay unexpectedly has a live parent".into()).into())}
    }else{
        state.play_frames.push(PlayFrame{parent,parent_owner,parent_context,child:child_handle.clone(),target:target.clone(),
            captured_energy:energy_on_use,paid:0,disposition,parent_finish_checkpoint_done:false});
    }
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

fn dispose_unplayed(state:&mut State,card:OwnedCard,disposition:FinishDisposition)->Result<(),EffectError>{
    super::super::super::piles::finish_owned_card(state,card,disposition,true)?;
    Ok(())
}

fn queue_after_finish(state:&mut State,parent:&crate::combat::Source,work:Effect)->Result<(),EffectError>{
    let at=state.pending.iter().position(|pending|pending.source==*parent&&pending.kind==EffectKind::FinishCard)
        .ok_or_else(||InputError::Invariant("autoplay parent has no retained finish continuation".into()))?;
    state.pending.insert(at+1,work);Ok(())
}

pub(in crate::combat) fn cancel_pending_card_queue(state:&mut State){
    for pending in &mut state.pending{
        match &mut pending.kind{
            EffectKind::Ironclad(IroncladEffect::HavocQueued{cancelled,..}|IroncladEffect::DoubleTapRepeat{cancelled,..})=>*cancelled=true,
            EffectKind::Silent(super::super::super::cards::SilentEffect::BurstRepeat{cancelled,..})=>*cancelled=true,
            EffectKind::Potion(crate::combat::PotionEffect::DistilledChaosQueuedChild{cancelled,..}
                |crate::combat::PotionEffect::DuplicationRepeat{cancelled,..})=>*cancelled=true,
            _=>(),
        }
    }
}

pub(in crate::combat) fn defer_parent_finish(state:&mut State,owner:&crate::combat::CardHandle)->Result<bool,InputError>{
    let Some(queued)=state.pending.iter().find(|effect|effect.source==crate::combat::Source::Card(owner.clone())&&matches!(effect.kind,
        EffectKind::Ironclad(IroncladEffect::HavocQueued{..}|IroncladEffect::DoubleTapRepeat{..})
        |EffectKind::Silent(super::super::super::cards::SilentEffect::BurstRepeat{..})
        |EffectKind::Potion(crate::combat::PotionEffect::DuplicationRepeat{..}))) else{return Ok(false)};
    let (child,target,energy_on_use,purge,imposed_exhaust)=match &queued.kind{
        EffectKind::Ironclad(IroncladEffect::HavocQueued{child,target,energy_on_use,..})=>(child.clone(),target.as_ref().map(|id|Actor::Enemy(id.clone())),*energy_on_use,false,true),
        EffectKind::Ironclad(IroncladEffect::DoubleTapRepeat{child,target,energy_on_use,..})=>(child.clone(),target.clone(),*energy_on_use,true,false),
        EffectKind::Silent(super::super::super::cards::SilentEffect::BurstRepeat{child,target,energy_on_use,..})
            =>(child.clone(),target.clone(),*energy_on_use,true,false),
        EffectKind::Potion(crate::combat::PotionEffect::DuplicationRepeat{child,target,energy_on_use,..})
            =>(child.clone(),target.clone(),*energy_on_use,true,false),
        _=>unreachable!(),
    };
    let _parent=state.in_play.as_ref().filter(|parent|parent.handle()==*owner)
        .ok_or_else(||InputError::Invariant("deferred autoplay parent is not in play".into()))?;
    let parent_context=state.play_context.as_ref().filter(|context|context.card==*owner).cloned()
        .ok_or_else(||InputError::Invariant("deferred autoplay parent context absent".into()))?;
    let parent=state.in_play.take().unwrap();state.play_context=None;
    let disposition=if purge{FinishDisposition::Purge}else if imposed_exhaust{FinishDisposition::Exhaust}else{FinishDisposition::Natural};
    state.play_frames.push(PlayFrame{parent:Some(parent),parent_owner:PlayFrameParentOwner::Card(owner.clone()),parent_context:Some(parent_context),child,target,
        captured_energy:energy_on_use,paid:0,disposition,parent_finish_checkpoint_done:true});
    Ok(true)
}

pub(in crate::combat) fn capture_random_enemy_target(state:&mut State)->Result<Option<crate::combat::EnemyHandle>,InputError>{
    let mut living=state.enemies.iter().filter(|enemy|enemy.life==ActorLife::Active&&enemy.hp>0)
        .map(|enemy|enemy.instance_id.clone()).collect::<Vec<_>>();
    living.sort_by_key(|owner|state.actor_slots.get(owner).copied().unwrap_or(u8::MAX));
    if living.is_empty(){return Ok(None)}
    let stream=state.card_rng.as_ref().ok_or_else(||InputError::Invariant("Havoc card-random stream absent".into()))?;
    let (index,next)=super::super::super::rng::uniform_below(stream,living.len() as u128)
        .map_err(|error|InputError::Unsupported(format!("Havoc target RNG unavailable: {error:?}")))?;
    state.card_rng=Some(next);
    Ok(Some(living[index as usize].clone()))
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
    super::super::super::piles::finish_owned_card(state,child,disposition,true)?;
    resume_parent(state,owner)?;
    Ok(())
}

pub(in crate::combat) fn resume_parent(state:&mut State,owner:&crate::combat::CardHandle)->Result<bool,InputError>{
    let Some(frame)=state.play_frames.last()else{
        if state.play_context.as_ref().is_some_and(|context|context.card==*owner){state.play_context=None;}
        return Ok(false)
    };
    if frame.child!=*owner{return Err(InputError::Invariant("finished autoplay child differs from top frame".into()))}
    let frame=state.play_frames.pop().unwrap();
    if frame.parent_finish_checkpoint_done{
        state.play_context=None;
        let parent=frame.parent.ok_or_else(||InputError::Invariant("deferred finished parent owner absent".into()))?;
        let handle=parent.handle();
        let disposition=frame.parent_context.map_or(FinishDisposition::Natural,|context|context.disposition);
        super::super::super::piles::finish_played_card(state,parent,disposition)?;
        // If the just-retired card was itself an autoplay child, finish that
        // containing continuation without replaying its already-fired card
        // checkpoint.
        if state.play_frames.last().is_some_and(|outer|outer.child==handle){
            resume_parent(state,&handle)?;
        }
        Ok(true)
    }else{
        match frame.parent_owner {
            PlayFrameParentOwner::Card(_)=>{state.in_play=frame.parent;state.play_context=frame.parent_context;},
            PlayFrameParentOwner::Potion{..}=>{
                if frame.parent.is_some()||frame.parent_context.is_some(){return Err(InputError::Invariant("potion autoplay frame acquired card parent custody".into()))}
                state.in_play=None;state.play_context=None;state.phase=Phase::Decision;
            }
        }
        Ok(true)
    }
}

pub(in crate::combat) fn apply_potion_work(state:&mut State,effect:&Effect,work:&crate::combat::PotionEffect)->Result<(),EffectError>{
    match work {
        crate::combat::PotionEffect::DistilledChaosTopCard{target}=>{
            let crate::combat::Source::Potion{id:crate::content::PotionId::DistilledChaos,..}=&effect.source else{
                return Err(InputError::Invariant("Distilled Chaos top action lost its potion owner".into()).into())};
            let Some(child)=state.piles.draw.pop() else{return Ok(())};
            let child_handle=child.handle();
            super::super::super::piles::refresh_hand_values(state);
            if state.public.known_draw.first()==Some(&child_handle){state.public.known_draw.remove(0);}else{state.public.known_draw.clear();}
            state.public.known_draw_bottom.retain(|known|known!=&child_handle);
            state.public.revealed_order.push(child_handle.clone());
            if state.queued_autoplay.insert(child_handle.clone(),child).is_some(){
                return Err(InputError::Invariant("Distilled Chaos top card already has queued ownership".into()).into())
            }
            state.pending.push_front(Effect{source:effect.source.clone(),target:None,
                kind:EffectKind::Potion(crate::combat::PotionEffect::DistilledChaosQueuedChild{
                    child:child_handle,target:target.clone(),energy_on_use:-1,cancelled:false})});
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
                resume_parent(state,child)?;
                return Ok(())
            }
            start_queued_child(state,child,target.clone(),true,false,*energy_on_use,PlayFrameParentOwner::Card(parent.clone()))
        }
        _=>Err(InputError::Invariant("non-scheduler potion work reached autoplay owner".into()).into()),
    }
}

pub(in crate::combat) fn card_played(state:&mut State,source:&crate::combat::Source,target:Option<&Actor>,work:&mut Vec<Effect>)->Result<(),EffectError>{
    let crate::combat::Source::Card(owner)=source else{return Ok(())};
    let Some((card_id,purge))=state.in_play.as_ref().filter(|owned|owned.handle()==*owner).and_then(OwnedCard::full)
        .map(|card|(card.id,card.local.purge))else{return Ok(())};
    let Some(definition)=Catalog::standard().card(card_id)else{return Ok(())};
    if purge||state.play_frames.last().is_some_and(|frame|frame.child==*owner&&frame.disposition==FinishDisposition::Purge){return Ok(())}
    let energy_on_use=state.play_context.as_ref().filter(|context|context.card==*owner).map_or(state.player.energy,|context|context.energy_on_use);
    let duplication=state.player_full_powers.iter().find(|power|power.id.0=="duplication").map_or(0,|power|power.state.amount);
    if duplication>0 {
        let charge=state.player_full_powers.iter_mut().find(|power|power.id.0=="duplication")
            .ok_or_else(||InputError::Invariant("Duplication charge disappeared at CardPlayed".into()))?;
        charge.state.amount-=1;
        if charge.state.amount==0 {
            work.push(Effect{source:crate::combat::Source::Actor(Actor::Player),target:Some(Actor::Player),
                kind:EffectKind::RemoveFullPower(super::super::super::FullPowerId("duplication"))});
        }
        let instance=super::super::super::piles::next_generated_instance(state)?;
        let copied=super::super::super::piles::make_same_instance_family_copy_from_owner(state,owner,instance)?;
        let mut child=OwnedCard::Full(copied);
        if let Some(live)=child.full_mut(){live.local.free_play=true;live.local.purge=true;}
        let child_handle=child.handle();
        state.card_ids_issued.insert(child_handle.clone());state.queued_autoplay.insert(child_handle.clone(),child);
        queue_after_finish(state,source,Effect{source:source.clone(),target:None,
            kind:EffectKind::Potion(crate::combat::PotionEffect::DuplicationRepeat{
                child:child_handle,target:target.cloned(),energy_on_use,cancelled:false})})?;
    }
    if definition.card_type==CardType::Attack {
        let charges=state.player_full_powers.iter().find(|power|power.id.0=="double_tap").map_or(0,|power|power.state.amount);
        if charges<=0{return Ok(())}
        let charge=state.player_full_powers.iter_mut().find(|power|power.id.0=="double_tap")
            .ok_or_else(||InputError::Invariant("Double Tap charge disappeared at CardPlayed".into()))?;
        charge.state.amount-=1;
        if charge.state.amount==0{
            // Only removal is deferred; the last charge itself is already consumed
            // at the source's repeat-queue checkpoint.
            work.push(Effect{source:source.clone(),target:Some(Actor::Player),kind:EffectKind::RemoveFullPower(super::super::super::FullPowerId("double_tap"))});
        }
        let instance=super::super::super::piles::next_generated_instance(state)?;
        let copied=super::super::super::piles::make_same_instance_family_copy_from_owner(state,owner,instance)?;
        let mut child=OwnedCard::Full(copied);
        if let Some(live)=child.full_mut(){live.local.free_play=true;live.local.purge=true;}
        let child_handle=child.handle();
        state.card_ids_issued.insert(child_handle.clone());state.queued_autoplay.insert(child_handle.clone(),child);
        let repeat=Effect{source:source.clone(),target:None,kind:EffectKind::Ironclad(IroncladEffect::DoubleTapRepeat{child:child_handle,target:target.cloned(),energy_on_use,cancelled:false})};
        queue_after_finish(state,source,repeat)?;
        return Ok(());
    }
    if definition.card_type!=CardType::Skill{return Ok(())}
    let charges=state.player_full_powers.iter().find(|power|power.id.0=="burst").map_or(0,|power|power.state.amount);
    if charges<=0{return Ok(())}
    let charge=state.player_full_powers.iter_mut().find(|power|power.id.0=="burst")
        .ok_or_else(||InputError::Invariant("Burst charge disappeared at CardPlayed".into()))?;
    charge.state.amount-=1;
    if charge.state.amount==0 {
        work.push(Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),
            kind:EffectKind::RemoveFullPower(super::super::super::FullPowerId("burst"))});
    }
    let instance=super::super::super::piles::next_generated_instance(state)?;
    let copied=super::super::super::piles::make_same_instance_family_copy_from_owner(state,owner,instance)?;
    let mut child=OwnedCard::Full(copied);
    if let Some(live)=child.full_mut(){live.local.free_play=true;live.local.purge=true;}
    let child_handle=child.handle();
    state.card_ids_issued.insert(child_handle.clone());state.queued_autoplay.insert(child_handle.clone(),child);
    let repeat=Effect{source:source.clone(),target:None,kind:EffectKind::Silent(
        super::super::super::cards::SilentEffect::BurstRepeat{child:child_handle,target:target.cloned(),energy_on_use,cancelled:false})};
    queue_after_finish(state,source,repeat)?;
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
        resume_parent(state,child)?;
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
                if state.enemies.iter().any(|enemy|enemy.instance_id==*owner&&enemy.life==ActorLife::Active&&enemy.hp>0){
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
            if state.piles.draw.is_empty(){
                if state.piles.discard.is_empty(){return Ok(())}
                state.pending.push_front(Effect{source:effect.source.clone(),target:None,kind:EffectKind::Ironclad(IroncladEffect::HavocExtract{target:target.clone()})});
                state.pending.push_front(Effect{source:effect.source.clone(),target:Some(Actor::Player),kind:EffectKind::Recycle});return Ok(())
            }
            let child=state.piles.draw.pop().unwrap();let handle=child.handle();
            crate::combat::piles::refresh_hand_values(state);
            if state.public.known_draw.first()==Some(&handle){state.public.known_draw.remove(0);}else{state.public.known_draw.clear();}
            state.public.known_draw_bottom.retain(|known|known!=&handle);state.public.revealed_order.push(handle);
            let energy_on_use=-1;
            let child_handle=child.handle();state.queued_autoplay.insert(child_handle.clone(),child);
            let queued=Effect{source:effect.source.clone(),target:None,kind:EffectKind::Ironclad(IroncladEffect::HavocQueued{child:child_handle,target:target.clone(),energy_on_use,cancelled:false})};
            queue_after_finish(state,&effect.source,queued)
        }
        IroncladEffect::DoubleTapRepeat{child,target,energy_on_use,cancelled}=>{
            let crate::combat::Source::Card(owner)=&effect.source else{return Err(InputError::Invariant("Double Tap source is not a card".into()).into())};
            if *cancelled{
                let copy=state.queued_autoplay.remove(child).ok_or_else(||InputError::Invariant("queued Double Tap child custody absent".into()))?;
                dispose_unplayed(state,copy,FinishDisposition::Purge)?;resume_parent(state,child)?;return Ok(())
            }
            start_queued_child(state,child,target.clone(),true,false,*energy_on_use,PlayFrameParentOwner::Card(owner.clone()))
        }
        IroncladEffect::HavocQueued{child,target,energy_on_use,cancelled}=>{
            let crate::combat::Source::Card(owner)=&effect.source else{return Err(InputError::Invariant("Havoc source is not a card".into()).into())};
            let target_actor=target.clone().map(Actor::Enemy);
            if *cancelled{
                let child=state.queued_autoplay.remove(child).ok_or_else(||InputError::Invariant("queued Havoc child custody absent".into()))?;
                let handle=child.handle();dispose_unplayed(state,child,FinishDisposition::Exhaust)?;resume_parent(state,&handle)?;return Ok(())
            }
            start_queued_child(state,child,target_actor,false,true,*energy_on_use,PlayFrameParentOwner::Card(owner.clone()))
        }
        IroncladEffect::FinishSuppressed=>finish_suppressed(state,&effect.source),
        _=>Err(InputError::Invariant("autoplay resolver received unrelated Ironclad work".into()).into()),
    }
}
