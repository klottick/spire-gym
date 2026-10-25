//! Five finite Ironclad relic listeners. Shared combat owns effect commits.
use crate::{combat::{Actor,Effect,EffectKind,Hook,Source,FullPowerId,DamagePayload},content::RelicId};
use super::{CombatItemContext,RelicHookWork};

const RED_ACTIVE:RelicId=RelicId::RedSkull;
const CLAY_PENDING:RelicId=RelicId::SelfFormingClay;

pub(super) fn on_hook(context:&mut CombatItemContext,hook:Hook,hp:i32,max_hp:i32)->RelicHookWork{
    let mut work=RelicHookWork::default();
    if context.owns_relic(RelicId::RedSkull){
        if hook==Hook::CombatStarted{
            let active=hp>0&&i64::from(hp)*2<=i64::from(max_hp);
            context.set_counter(RED_ACTIVE,i32::from(active));
            if active{work.direct_strength=3;}
        }else if hook==Hook::CombatSettled{context.set_counter(RED_ACTIVE,0);}
    }
    if hook==Hook::TurnStartedPreDraw&&context.owns_relic(CLAY_PENDING){
        let pending=context.counter(CLAY_PENDING);
        if pending>0{context.set_counter(CLAY_PENDING,0);work.top.push(Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::GainBlock(pending)});}
    }
    if hook==Hook::CombatSettled&&context.owns_relic(CLAY_PENDING){context.set_counter(CLAY_PENDING,0);}
    work
}

pub(super) fn on_damage(context:&mut CombatItemContext,target:&Actor,hp_lost:i32,hp_after:i32,max_hp:i32)->Result<Vec<Effect>,crate::combat::InputError>{
    if target!=&Actor::Player||hp_lost<=0{return Ok(Vec::new())}
    let mut work=Vec::new();
    if context.owns_relic(CLAY_PENDING){let next=context.counter(CLAY_PENDING).checked_add(3).filter(|next|*next<=999).ok_or_else(||crate::combat::InputError::Unsupported("Self-Forming Clay pending Block exceeded supported bound".into()))?;context.set_counter(CLAY_PENDING,next);}
    work.extend(on_hp_state(context,hp_after,max_hp));
    Ok(work)
}

pub(super) fn on_hp_state(context:&mut CombatItemContext,hp:i32,max_hp:i32)->Vec<Effect>{
    let mut work=Vec::new();
    // Existing source closure does not prove threshold callbacks after lethal
    // transitions. Keep the counted latch for settlement cleanup instead.
    if hp<=0{return work}
    if context.owns_relic(RelicId::RedSkull){
        let active=context.counter(RED_ACTIVE)>0;
        let bloodied=i64::from(hp)*2<=i64::from(max_hp);
        if bloodied&&!active{context.set_counter(RED_ACTIVE,1);work.push(power(3));}
        else if !bloodied&&active{context.set_counter(RED_ACTIVE,0);work.push(power(-3));}
    }
    work
}

pub(super) fn on_exhaust(context:&CombatItemContext,_source:&Source,enemies:&[crate::combat::EnemyHandle])->Vec<Effect>{
    if !context.owns_relic(RelicId::CharonsAshes){return Vec::new()}
    enemies.iter().cloned().map(|enemy|Effect{source:Source::None,target:Some(Actor::Enemy(enemy)),kind:EffectKind::Damage(DamagePayload::Thorns(3))}).collect()
}

fn power(amount:i32)->Effect{Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::ApplyFullPower{id:FullPowerId("strength"),amount}}}
