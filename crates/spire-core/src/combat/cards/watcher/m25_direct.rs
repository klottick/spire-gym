//! M25 direct and stance cards (plan Task C2) on the shared owners.
//! Values come from the M25 card initializer (`card.local`), costs from `m25_cost`;
//! damage, Block, draw and stance go through the shared payload owners. Consecrate,
//! Empty Body and Empty Mind keep their generic handlers (`Ok(None)` here).
use super::*;
use crate::content::CardId;
use crate::run::RunError;

type Payload=(i32,Vec<(Option<PayloadTarget>,EffectKind)>);

fn living(enemy:&&super::super::super::model::EnemyState)->bool{enemy.life==ActorLife::Active&&enemy.hp>0}

/// Disclosed project decision (contract Binding 9). Fear No Evil requests Calm when the
/// target's intent is ATTACK, ATTACK_BUFF, ATTACK_DEBUFF or ATTACK_DEFEND. The native
/// `FullIntentId` carries no intent category, so the current intent is classified as one of
/// those four exactly when the engine holds a damage entry for it with base >= 0: the same
/// intent-damage data the accepted Spot Weakness and Go for the Eyes owners read. Every
/// attack-category move has such an entry and no other move does (Exploder's explode has
/// damage but an UNKNOWN intent and no entry).
fn intends_attack(state:&State,target:u64)->bool{
    state.enemies.get((target as usize).wrapping_sub(1)).is_some_and(|enemy|match &enemy.intent_id{
        ActorIntentId::Legacy(intent)=>enemy.damage_entries.get(intent).map(|entry|entry.base),
        ActorIntentId::Full(intent)=>enemy.full_damage_entries.get(intent).map(|entry|entry.base),
    }.is_some_and(|base|base>=0))
}

pub(in crate::combat) fn payload(state:&State,card:&LiveCard,target:Option<u64>)->Result<Option<Payload>,RunError>{
    use CardId::*;
    if !matches!(card.id,BowlingBash|Crescendo|FlyingSleeves|Halt|Protect|Tranquility|FearNoEvil|Indignation|InnerPeace|WheelKick|Ragnarok|Scrawl|SpiritShield){
        return Ok(None)
    }
    let targeted=matches!(card.id,BowlingBash|FlyingSleeves|FearNoEvil|WheelKick);
    if targeted!=target.is_some(){return Err(RunError::InvalidAction)}
    let cost=super::m25::m25_cost(card.id,card.upgrades>0).ok_or(RunError::InvalidAction)?;
    let damage=|selected:u64|->Result<(Option<PayloadTarget>,EffectKind),RunError>{
        let base=card.local.base_damage.ok_or(RunError::InvalidAction)?;
        Ok((Some(PayloadTarget::SelectedEnemy),EffectKind::Damage(DamagePayload::Fixed(super::super::modified_mutable_damage(state,selected,base)?))))
    };
    let magic=||card.local.magic_number.ok_or(RunError::InvalidAction);
    let block=|base:i32|->Result<(Option<PayloadTarget>,EffectKind),RunError>{Ok((Some(PayloadTarget::Player),EffectKind::GainBlock(super::super::modified_mutable_block(state,base)?)))};
    let stance=|next:Stance|(Some(PayloadTarget::Player),EffectKind::Character(CharacterEffect::Stance(next)));
    let current=state.character.stance;
    let effects=match card.id{
        // One hit on the selected target per enemy not dead or escaped when counted at use.
        BowlingBash=>{let hit=damage(target.unwrap())?;vec![hit;state.enemies.iter().filter(living).count()]},
        FlyingSleeves=>{let hit=damage(target.unwrap())?;vec![hit.clone(),hit]},
        WheelKick=>vec![damage(target.unwrap())?,(Some(PayloadTarget::Player),EffectKind::Draw(magic()?))],
        // The hit is queued ahead of the conditional Calm request; the intent is read for
        // this resolution before the hit.
        FearNoEvil=>{
            let selected=target.unwrap();
            let mut effects=vec![damage(selected)?];
            if intends_attack(state,selected){effects.push(stance(Stance::Calm));}
            effects
        },
        Crescendo=>vec![stance(Stance::Wrath)],
        Tranquility=>vec![stance(Stance::Calm)],
        Protect=>vec![block(card.local.base_block.ok_or(RunError::InvalidAction)?)?],
        // Each component is modified and floored separately; one Block gain.
        Halt=>{
            let mut amount=super::super::modified_mutable_block(state,card.local.base_block.ok_or(RunError::InvalidAction)?)?;
            if current==Stance::Wrath{
                amount=amount.checked_add(super::super::modified_mutable_block(state,magic()?)?).ok_or(RunError::Unsupported("Halt Block int32 exhausted"))?;
            }
            vec![(Some(PayloadTarget::Player),EffectKind::GainBlock(amount))]
        },
        // Mutually exclusive branches chosen by the stance at this resolution.
        Indignation=>if current==Stance::Wrath{
            let amount=magic()?;
            state.enemies.iter().filter(living).map(|enemy|(Some(PayloadTarget::Enemy(enemy.instance_id.clone())),EffectKind::ApplyFullPower{id:FullPowerId("vulnerable"),amount})).collect()
        }else{vec![stance(Stance::Wrath)]},
        InnerPeace=>if current==Stance::Calm{vec![(Some(PayloadTarget::Player),EffectKind::Draw(magic()?))]}else{vec![stance(Stance::Calm)]},
        // Each hit chooses its target at that hit through the shared blue random-enemy
        // owner (source random(0,n-1) including n=1), recalculating damage per target.
        Ragnarok=>{
            let base=card.local.base_damage.ok_or(RunError::InvalidAction)?;
            let remaining=u32::try_from(magic()?).map_err(|_|RunError::InvalidAction)?;
            vec![(None,EffectKind::Defect(DefectEffect::Direct(super::super::defect::DirectEffect::RandomHit{base,remaining})))]
        },
        // The shared ExpertiseAction owner: at resolution, draw up to a 10-card hand.
        Scrawl=>vec![(None,EffectKind::Silent(SilentEffect::Expertise{size:10}))],
        // Count the other cards in hand at use, multiply, then modify Block once.
        SpiritShield=>{
            let own=crate::combat::card_state::handle(card.instance);
            let others=i32::try_from(state.piles.hand.iter().filter(|owned|owned.handle()!=own).count()).map_err(|_|RunError::InvalidAction)?;
            vec![block(others.checked_mul(magic()?).ok_or(RunError::Unsupported("Spirit Shield Block int32 exhausted"))?)?]
        },
        _=>unreachable!("filtered above"),
    };
    Ok(Some((cost,effects)))
}
