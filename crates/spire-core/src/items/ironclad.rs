//! Five finite Ironclad relic listeners. Shared combat owns effect commits.
use crate::{combat::{Actor,Effect,EffectKind,Hook,Source,FullPowerId,DamagePayload},content::RelicId};
use super::{CombatItemContext,RelicHookWork};

const RED_ACTIVE:RelicId=RelicId::RedSkull;
const CLAY_PENDING:RelicId=RelicId::SelfFormingClay;

/// Combat phase work of the accepted Ironclad relics (bodies unchanged).
pub(super) fn phase(id:RelicId,context:&mut CombatItemContext,phase:super::dispatch::ItemPhase,facts:&super::dispatch::PhaseFacts<'_>,work:&mut RelicHookWork)->Result<bool,crate::combat::InputError>{
    use super::dispatch::ItemPhase;
    let hook=facts.hook;
    match id{
        RelicId::RedSkull=>{
            // atBattleStart: clear the latch and queue the bloodied check at the bottom
            // (source-check-turnstart-hooks-v1), after any top-queued battle-start heal.
            if hook==Some(Hook::CombatStarted){
                context.set_counter(RED_ACTIVE,0);
                work.bottom.push(Effect{source:Source::None,target:Some(Actor::Player),
                    kind:EffectKind::Relic(crate::combat::RelicEffect::RedSkullCheck)});
            }else if hook==Some(Hook::CombatSettled){context.set_counter(RED_ACTIVE,0);}
        }
        RelicId::SelfFormingClay=>{
            if hook==Some(Hook::TurnStartedPreDraw){
                let pending=context.counter(CLAY_PENDING);
                if pending>0{context.set_counter(CLAY_PENDING,0);work.bottom.push(Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::GainBlock(pending)});}
            }
            if hook==Some(Hook::CombatSettled){context.set_counter(CLAY_PENDING,0);}
        }
        RelicId::CharonsAshes=>if phase==ItemPhase::Exhaust{
            work.top.extend(facts.live_enemies.iter().cloned().map(|enemy|Effect{source:Source::None,target:Some(Actor::Enemy(enemy)),kind:EffectKind::Damage(DamagePayload::Thorns(3))}));
        },
        RelicId::PaperPhrog|RelicId::ChampionsBelt=>{}
        _=>return Ok(false),
    }
    Ok(true)
}

/// Immediate HP-state work (`laws::hp_changed`): Self-Forming Clay's pending Block on a
/// positive player HP loss, then Red Skull's threshold.
pub(super) fn hp_changed(id:RelicId,context:&mut CombatItemContext,hp_lost:i32,hp:i32,max_hp:i32,work:&mut Vec<Effect>)->Result<bool,crate::combat::InputError>{
    match id{
        RelicId::SelfFormingClay=>{if hp_lost>0{let next=context.counter(CLAY_PENDING).checked_add(3).filter(|next|*next<=999).ok_or_else(||crate::combat::InputError::Unsupported("Self-Forming Clay pending Block exceeded supported bound".into()))?;context.set_counter(CLAY_PENDING,next);}}
        RelicId::RedSkull=>work.extend(on_hp_state(context,hp_lost,hp,max_hp)),
        _=>return Ok(false),
    }
    Ok(true)
}

fn on_hp_state(context:&mut CombatItemContext,hp_lost:i32,hp:i32,max_hp:i32)->Vec<Effect>{
    let mut work=Vec::new();
    // Existing source closure does not prove threshold callbacks after lethal
    // transitions. Keep the counted latch for settlement cleanup instead.
    if hp<=0{return work}
    if context.owns_relic(RelicId::RedSkull){
        let active=context.counter(RED_ACTIVE)>0;
        let bloodied=i64::from(hp)*2<=i64::from(max_hp);
        // M30 G5: only damage reaches `onBloodied`; a heal calls `onNotBloodied` alone, on crossing above
        // half (AbstractCreature.java:441; RedSkull.java). A heal that leaves the player bloodied (Burning
        // Blood's victory heal after the settlement reset) must not re-arm the latch.
        if bloodied&&!active&&hp_lost>0{context.set_counter(RED_ACTIVE,1);work.push(power(3));}
        else if !bloodied&&active{context.set_counter(RED_ACTIVE,0);work.push(power(-3));}
    }
    work
}

fn power(amount:i32)->Effect{Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::ApplyFullPower{id:FullPowerId("strength"),amount}}}
