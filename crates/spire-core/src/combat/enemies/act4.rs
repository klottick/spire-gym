//! Act4 actors on the ordinary resolver.
use super::*;
pub(crate) mod shield_spear;
pub(crate) mod heart;

pub(crate) fn supports(id:EnemyId)->bool{matches!(id,EnemyId::SpireShield|EnemyId::SpireSpear)}
pub(crate) fn select_move(id:EnemyId,history:&[FullIntentId],memory:&EnemyMemory,stream:&StreamState)->Result<MoveSelection,SelectError>{
    shield_spear::select_move(id,history,memory,stream)
}
pub(in crate::combat) fn move_work(state:&mut State,owner:&EnemyHandle)->Result<Vec<Effect>,super::super::EffectError>{
    shield_spear::move_work(state,owner)
}
