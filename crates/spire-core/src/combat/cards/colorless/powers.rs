//! Executable Colorless powers whose continuations share combat owners.
use super::*;
use crate::combat::model::{BombTimer, ColorlessState};

pub(super) fn starting_cost(id:crate::content::CardId,upgrades:u32)->Option<i32>{
    use crate::content::CardId::*;
    Some(match id{
        Mayhem=>if upgrades>0{1}else{2},Panache|SadisticNature=>0,TheBomb=>2,_=>return None,
    })
}

pub(super) fn payload(state:&State,card:&LiveCard,target:Option<u64>)
    ->Result<Option<(i32,Vec<(Option<PayloadTarget>,EffectKind)>)>,crate::run::RunError>{
    use crate::content::CardId::*;
    if !matches!(card.id,Mayhem|Panache|SadisticNature|TheBomb){return Ok(None)}
    if target.is_some(){return Err(crate::run::RunError::InvalidAction)}
    let upgraded=card.upgrades>0;
    let mut effects=Vec::new();
    let cost=match card.id{
        Mayhem=>{effects.push((Some(PayloadTarget::Player),EffectKind::ApplyFullPower{
            id:FullPowerId("mayhem"),amount:1}));if upgraded{1}else{2}},
        Panache=>{effects.push((Some(PayloadTarget::Player),EffectKind::ApplyFullPower{
            id:FullPowerId("panache"),amount:if upgraded{14}else{10}}));0},
        SadisticNature=>{effects.push((Some(PayloadTarget::Player),EffectKind::ApplyFullPower{
            id:FullPowerId("sadistic_nature"),amount:if upgraded{7}else{5}}));0},
        TheBomb=>{effects.push((Some(PayloadTarget::Player),EffectKind::Colorless(
            ColorlessEffect::BombInstall{damage:if upgraded{50}else{40}})));2},
        _=>unreachable!(),
    };
    let _=state;
    Ok(Some((cost,effects)))
}

pub(super) fn public_values(card:&LiveCard)->Option<CardPublicValues>{
    use crate::content::CardId::*;
    let upgraded=card.upgrades>0;
    let magic=match card.id{
        Mayhem=>Some(1),Panache=>Some(if upgraded{14}else{10}),
        SadisticNature=>Some(if upgraded{7}else{5}),TheBomb=>Some(if upgraded{50}else{40}),
        _=>return None,
    };
    Some(CardPublicValues{base_damage:None,base_block:None,magic_number:magic,damage_bonus:None,
        cost_kind:CardCostKind::Fixed,free_play:card.local.free_play,
        exhaust_on_finish:card.local.exhaust_on_finish,ethereal:card.local.ethereal,
        innate:card.local.innate,temporary_cost:card.local.temporary_cost})
}

pub(super) fn validate_power_application(state:&State,effect:&Effect,id:FullPowerId,amount:i32)
    ->Result<(),InputError>{
    let Source::Card(owner)=&effect.source else{return Err(InputError::Invariant("Colorless power application source absent".into()))};
    let card=state.in_play.as_ref().filter(|card|card.handle()==*owner).and_then(OwnedCard::full)
        .ok_or_else(||InputError::Invariant("Colorless power card owner absent".into()))?;
    let expected=match (card.id,id.0){
        (crate::content::CardId::Mayhem,"mayhem")=>1,
        (crate::content::CardId::Panache,"panache")=>if card.upgrades>0{14}else{10},
        (crate::content::CardId::SadisticNature,"sadistic_nature")=>if card.upgrades>0{7}else{5},
        _=>return Err(InputError::Invariant("Colorless power application card/id mismatch".into())),
    };
    if effect.target!=Some(Actor::Player)||amount!=expected{
        return Err(InputError::Invariant("Colorless power application target/payload mismatch".into()))
    }
    Ok(())
}

pub(super) fn power_applied(state:&mut State,actor:&Actor,id:FullPowerId,application:bool,previous:i32){
    if matches!(actor,Actor::Player)&&application&&id.0=="panache"&&previous==0{
        state.colorless.panache_remaining=Some(5);
    }
}

pub(super) fn power_removed(state:&mut State,actor:&Actor,id:FullPowerId){
    if matches!(actor,Actor::Player)&&id.0=="panache"{
        state.colorless.panache_remaining=None;
    }
}

fn active_targets(state:&State)->Vec<crate::combat::EnemyHandle>{
    let mut targets=state.enemies.iter().filter(|enemy|
        enemy.life==ActorLife::Active&&enemy.hp>0).map(|enemy|enemy.instance_id.clone()).collect::<Vec<_>>();
    targets.sort_by_key(|owner|state.actor_slots.get(owner).copied().unwrap_or(u8::MAX));
    targets
}

/// Count one use, then place the non-Attack reaction behind that card's own
/// payload and before its after-use checkpoint.
pub(super) fn card_played(state:&mut State,source:&Source)->Result<(),EffectError>{
    if !state.full_runtime||!matches!(source,Source::Card(_)){return Ok(())}
    let damage=state.player_full_powers.iter().find(|power|power.id.0=="panache")
        .map_or(0,|power|power.state.amount);
    if damage<=0{return Ok(())}
    let remaining=state.colorless.panache_remaining.unwrap_or(5);
    if !(1..=5).contains(&remaining){return Err(InputError::Invariant("Panache remaining-use state bounds".into()).into())}
    if remaining>1{
        state.colorless.panache_remaining=Some(remaining-1);
        return Ok(())
    }
    state.colorless.panache_remaining=Some(5);
    let at=state.pending.iter().position(|work|work.kind==EffectKind::Hook(Hook::CardAfterUse))
        .unwrap_or_else(||state.pending.iter().position(|work|work.kind==EffectKind::FinishCard).unwrap_or(state.pending.len()));
    state.pending.insert(at,Effect{source:Source::Actor(Actor::Player),target:None,
        kind:EffectKind::Colorless(ColorlessEffect::PanacheDamage{damage})});
    Ok(())
}

pub(super) fn power_hook(state:&mut State,hook:Hook,target:Option<&Actor>,work:&mut Vec<Effect>)
    ->Result<(),EffectError>{
    if !state.full_runtime{return Ok(())}
    if hook==Hook::TurnStartedPreDraw&&target==Some(&Actor::Player){
        if state.player_full_powers.iter().any(|power|power.id.0=="panache"&&power.state.amount>0){
            state.colorless.panache_remaining=Some(5);
        }
        let amount=state.player_full_powers.iter().find(|power|power.id.0=="mayhem")
            .map_or(0,|power|power.state.amount);
        if amount>0&& !active_targets(state).is_empty(){
            let owner=state.played_powers.iter().filter_map(OwnedCard::full)
                .find(|card|card.id==crate::content::CardId::Mayhem).map(|card|crate::combat::card_state::handle(card.instance))
                .ok_or_else(||InputError::Invariant("Mayhem power card owner absent".into()))?;
            for _ in 0..amount{
                work.push(Effect{source:Source::Card(owner.clone()),target:None,
                    kind:EffectKind::Colorless(ColorlessEffect::MayhemPlayTopCard)});
            }
        }
    }
    if hook==Hook::PlayerPreEndTurnCards&&target==Some(&Actor::Player)&&!active_targets(state).is_empty(){
        for bomb in state.colorless.bombs.clone(){
            work.push(Effect{source:Source::Card(bomb.owner),target:Some(Actor::Player),
                kind:EffectKind::Colorless(ColorlessEffect::BombTick{id:bomb.id})});
        }
    }
    Ok(())
}

fn validate_owned_mayhem(state:&State,source:&Source)->Result<crate::combat::CardHandle,InputError>{
    let Source::Card(owner)=source else{return Err(InputError::Invariant("Mayhem autoplay source absent".into()))};
    let exists=state.played_powers.iter().any(|owned|owned.handle()==*owner
        &&owned.full().is_some_and(|card|card.id==crate::content::CardId::Mayhem));
    if !exists||!state.player_full_powers.iter().any(|power|power.id.0=="mayhem"&&power.state.amount>0){
        return Err(InputError::Invariant("Mayhem autoplay power owner missing".into()))
    }
    Ok(owner.clone())
}

pub(super) fn validate_work(state:&State,effect:&Effect,work:&ColorlessEffect)->Result<(),InputError>{
    match work{
        ColorlessEffect::MayhemPlayTopCard=>{
            if effect.target.is_some(){return Err(InputError::Invariant("Mayhem top-card target shape".into()))}
            validate_owned_mayhem(state,&effect.source).map(|_|())
        }
        ColorlessEffect::MayhemAutoplay{child,target}=>{
            validate_owned_mayhem(state,&effect.source)?;
            let child_owned=state.queued_autoplay.get(child).and_then(OwnedCard::full);
            let target_valid=matches!(target.as_ref(),Some(Actor::Enemy(owner))
                if state.enemies.iter().any(|enemy|enemy.instance_id==*owner&&enemy.life==ActorLife::Active&&enemy.hp>0));
            if effect.target.is_some()||child_owned.is_none()||!target_valid{
                return Err(InputError::Invariant("Mayhem autoplay child/target custody".into()))
            }
            Ok(())
        }
        ColorlessEffect::BombInstall{damage}=>{
            let Source::Card(owner)=&effect.source else{return Err(InputError::Invariant("Bomb install source absent".into()))};
            let card=state.in_play.as_ref().filter(|card|card.handle()==*owner).and_then(OwnedCard::full)
                .ok_or_else(||InputError::Invariant("Bomb install card owner absent".into()))?;
            let expected=if card.upgrades>0{50}else{40};
            if card.id!=crate::content::CardId::TheBomb||effect.target!=Some(Actor::Player)||*damage!=expected{
                return Err(InputError::Invariant("Bomb install card/target/payload mismatch".into()))
            }
            if state.colorless.next_bomb_id==0||state.colorless.next_bomb_id==u64::MAX{
                return Err(InputError::Unsupported("Bomb cast identity exhausted".into()))
            }
            Ok(())
        }
        ColorlessEffect::BombTick{id}=>{
            let bomb=state.colorless.bombs.iter().find(|bomb|bomb.id==*id)
                .ok_or_else(||InputError::Invariant("Bomb tick owner absent".into()))?;
            if effect.target!=Some(Actor::Player)||effect.source!=Source::Card(bomb.owner.clone()){
                return Err(InputError::Invariant("Bomb tick source/target mismatch".into()))
            }
            Ok(())
        }
        ColorlessEffect::PanacheDamage{damage}=>{
            if effect.target.is_some()||effect.source!=Source::Actor(Actor::Player)||!(1..=999).contains(damage){
                return Err(InputError::Invariant("Panache damage target/payload".into()))
            }
            Ok(())
        }
        _=>Err(InputError::Invariant("Impatience work reached power validator".into())),
    }
}

pub(super) fn apply(state:&mut State,effect:&Effect,work:&ColorlessEffect)->Result<(),EffectError>{
    match work{
        ColorlessEffect::MayhemPlayTopCard=>{
            validate_owned_mayhem(state,&effect.source)?;
            if state.piles.draw.is_empty(){return Ok(())}
            let Some(target)=super::super::ironclad::autoplay::capture_random_enemy_target(state)? else{return Ok(())};
            let child=state.piles.draw.pop().unwrap();let child_handle=child.handle();
            crate::combat::piles::refresh_hand_values(state);
            if state.public.known_draw.first()==Some(&child_handle){state.public.known_draw.remove(0);}
            else{state.public.known_draw.clear()}
            state.public.known_draw_bottom.retain(|known|known!=&child_handle);
            state.public.revealed_order.push(child_handle.clone());
            state.queued_autoplay.insert(child_handle.clone(),child);
            state.pending.push_front(Effect{source:effect.source.clone(),target:None,
                kind:EffectKind::Colorless(ColorlessEffect::MayhemAutoplay{child:child_handle,target:Some(Actor::Enemy(target))})});
            Ok(())
        }
        ColorlessEffect::MayhemAutoplay{child,target}=>{
            let parent=validate_owned_mayhem(state,&effect.source)?;
            super::super::ironclad::autoplay::start_queued_child(state,child,target.clone(),false,false,-1,crate::combat::model::PlayFrameParentOwner::Card(parent))
        }
        ColorlessEffect::BombInstall{damage}=>{
            let Source::Card(owner)=&effect.source else{return Err(InputError::Invariant("Bomb install source absent".into()).into())};
            let id=state.colorless.next_bomb_id;
            let next=id.checked_add(1).filter(|next|*next>id)
                .ok_or_else(||InputError::Unsupported("Bomb cast identity exhausted".into()))?;
            state.colorless.bombs.push(BombTimer{id,owner:owner.clone(),remaining:3,damage:*damage});
            state.colorless.next_bomb_id=next;
            Ok(())
        }
        ColorlessEffect::BombTick{id}=>{
            let index=state.colorless.bombs.iter().position(|bomb|bomb.id==*id)
                .ok_or_else(||InputError::Invariant("Bomb tick owner absent".into()))?;
            if state.colorless.bombs[index].remaining>1{
                state.colorless.bombs[index].remaining-=1;
                return Ok(())
            }
            let bomb=state.colorless.bombs.remove(index);
            for owner in active_targets(state).into_iter().rev(){
                state.pending.push_front(Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Enemy(owner)),
                    kind:EffectKind::Damage(DamagePayload::Thorns(bomb.damage))});
            }
            Ok(())
        }
        ColorlessEffect::PanacheDamage{damage}=>{
            if effect.target.is_some(){return Err(InputError::Invariant("Panache damage target shape".into()).into())}
            for owner in active_targets(state).into_iter().rev(){
                state.pending.push_front(Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Enemy(owner)),
                    kind:EffectKind::Damage(DamagePayload::Thorns(*damage))});
            }
            Ok(())
        }
        _=>Err(InputError::Invariant("Impatience work reached power validator".into()).into()),
    }
}

pub(super) fn validate_state(state:&State)->Result<(),InputError>{
    let colorless=&state.colorless;
    if !state.full_runtime&&*colorless!=ColorlessState::default(){
        return Err(InputError::Invariant("legacy combat cannot carry Colorless power state".into()))
    }
    if colorless.panache_remaining.is_some_and(|remaining|!(1..=5).contains(&remaining))
        ||(colorless.panache_remaining.is_some()
            &&!state.player_full_powers.iter().any(|power|power.id.0=="panache"&&power.state.amount>0)){
        return Err(InputError::Invariant("Panache counter/power custody".into()))
    }
    let mut ids=std::collections::BTreeSet::new();
    if colorless.next_bomb_id==0||colorless.bombs.iter().any(|bomb|bomb.id==0||bomb.id>=colorless.next_bomb_id
        ||!(1..=3).contains(&bomb.remaining)||!matches!(bomb.damage,40|50)||!ids.insert(bomb.id)
        ||!state.card_ids_issued.contains(&bomb.owner)
        ||!state.piles.all(state.in_play.as_ref()).chain(state.stasis.values()).chain(&state.played_powers)
            .chain(&state.inactive_cards).chain(state.play_frames.iter().filter_map(|frame|frame.parent.as_ref()))
            .chain(state.queued_autoplay.values()).any(|card|card.handle()==bomb.owner
                &&card.full().is_some_and(|live|live.id==crate::content::CardId::TheBomb))){
        return Err(InputError::Invariant("Bomb timer identity/owner/payload bounds".into()))
    }
    Ok(())
}
