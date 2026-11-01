//! M25 stateful Watcher card payloads on shared history, damage, selection and fatal owners.
use super::*;
use crate::content::{CardId,CardType};
use crate::run::RunError;

type Payload=(i32,Vec<(Option<PayloadTarget>,EffectKind)>);

pub(in crate::combat) fn payload(state:&State,card:&LiveCard,target:Option<u64>)->Result<Option<Payload>,RunError>{
    use CardId::*;
    if !matches!(card.id,FollowUp|SashWhip|CrushJoints|Sanctity|SignatureMove|Wallop|Tantrum|Judgment|Omniscience|LessonLearned){return Ok(None)}
    let cost=super::m25::m25_cost(card.id,card.upgrades>0).ok_or(RunError::InvalidAction)?;
    let previous=state.played_card_types.last().copied();
    let enemy=||->Result<(u64,crate::combat::EnemyHandle),RunError>{
        let id=target.ok_or(RunError::InvalidAction)?;
        let owner=id.checked_sub(1).and_then(|index|usize::try_from(index).ok()).and_then(|index|state.enemies.get(index))
            .filter(|enemy|enemy.life==ActorLife::Active&&enemy.hp>0)
            .map(|enemy|enemy.instance_id.clone()).ok_or(RunError::InvalidAction)?;
        Ok((id,owner))
    };
    let hit=|id:u64,base:i32|->Result<(Option<PayloadTarget>,EffectKind),RunError>{
        Ok((Some(PayloadTarget::SelectedEnemy),EffectKind::Damage(DamagePayload::Fixed(super::super::modified_mutable_damage(state,id,base)?))))
    };
    let effects=match card.id{
        FollowUp|SashWhip|CrushJoints|SignatureMove|Wallop|Tantrum|Judgment|LessonLearned=>{
            let (id,owner)=enemy()?;
            match card.id{
                FollowUp=>{
                    let mut work=vec![hit(id,card.local.base_damage.ok_or(RunError::InvalidAction)?)?];
                    if previous==Some(CardType::Attack){work.push((Some(PayloadTarget::Player),EffectKind::GainEnergy(1)));}
                    work
                },
                SashWhip=>{
                    let mut work=vec![hit(id,card.local.base_damage.ok_or(RunError::InvalidAction)?)?];
                    if previous==Some(CardType::Attack){work.push((Some(PayloadTarget::Enemy(owner)),EffectKind::ApplyFullPower{id:FullPowerId("weak"),amount:card.local.magic_number.ok_or(RunError::InvalidAction)?}));}
                    work
                },
                CrushJoints=>{
                    let mut work=vec![hit(id,card.local.base_damage.ok_or(RunError::InvalidAction)?)?];
                    if previous==Some(CardType::Skill){work.push((Some(PayloadTarget::Enemy(owner)),EffectKind::ApplyFullPower{id:FullPowerId("vulnerable"),amount:card.local.magic_number.ok_or(RunError::InvalidAction)?}));}
                    work
                },
                SignatureMove=>vec![hit(id,card.local.base_damage.ok_or(RunError::InvalidAction)?)?],
                Wallop=>vec![hit(id,card.local.base_damage.ok_or(RunError::InvalidAction)?)?],
                Tantrum=>{
                    let base=card.local.base_damage.ok_or(RunError::InvalidAction)?;
                    let hits=card.local.magic_number.ok_or(RunError::InvalidAction)?;
                    let attack=hit(id,base)?;
                    let mut work=vec![attack;usize::try_from(hits).map_err(|_|RunError::InvalidAction)?];
                    work.push((Some(PayloadTarget::Player),EffectKind::Character(CharacterEffect::Stance(Stance::Wrath))));
                    work
                },
                // JudgementAction reads the target's HP when it resolves, not at play.
                Judgment=>{let _=id;vec![(Some(PayloadTarget::Enemy(owner)),EffectKind::Watcher(WatcherEffect::JudgmentKill{threshold:card.local.magic_number.ok_or(RunError::InvalidAction)?}))]},
                LessonLearned=>vec![hit(id,card.local.base_damage.ok_or(RunError::InvalidAction)?)?],
                _=>unreachable!(),
            }
        },
        Sanctity=>{
            if target.is_some(){return Err(RunError::InvalidAction)}
            let block=card.local.base_block.ok_or(RunError::InvalidAction)?;
            let mut work=vec![(Some(PayloadTarget::Player),EffectKind::GainBlock(super::super::modified_mutable_block(state,block)?))];
            if previous==Some(CardType::Skill){work.push((Some(PayloadTarget::Player),EffectKind::Draw(card.local.magic_number.ok_or(RunError::InvalidAction)?)));}
            work
        },
        Omniscience=>{
            if target.is_some(){return Err(RunError::InvalidAction)}
            vec![(Some(PayloadTarget::Player),EffectKind::Watcher(WatcherEffect::OpenOmniscience))]
        },
        _=>unreachable!(),
    };
    Ok(Some((cost,effects)))
}
