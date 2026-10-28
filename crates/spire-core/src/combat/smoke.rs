//! Native deterministic escape checkpoint; no frame-timer parity is implied.
use super::*;
use super::model::{State,ActorEnemyId};
pub(super) fn allowed(state:&State)->bool{
    state.full_runtime&&state.player.hp>0&&state.enemies.iter().any(|enemy|enemy.hp>0&&enemy.life==ActorLife::Active)
        &&!state.enemies.iter().any(|enemy|enemy.full_powers.iter().any(|power|power.id.0=="back_attack")||matches!(enemy.enemy_id,
            ActorEnemyId::Full(crate::content::EnemyId::SlimeBoss|crate::content::EnemyId::Guardian|crate::content::EnemyId::Hexaghost|crate::content::EnemyId::BronzeAutomaton|crate::content::EnemyId::Champ|crate::content::EnemyId::Collector|crate::content::EnemyId::AwakenedOne|crate::content::EnemyId::Deca|crate::content::EnemyId::Donu|crate::content::EnemyId::TimeEater|crate::content::EnemyId::CorruptHeart)))
}
impl Combat{pub(crate) fn native_smoke_allowed(&self)->bool{allowed(&self.state)}}
