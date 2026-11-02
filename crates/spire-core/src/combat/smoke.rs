//! Native deterministic escape checkpoint; no frame-timer parity is implied.
use super::*;
use super::model::{State,ActorEnemyId};
pub(super) fn allowed(state:&State)->bool{
    state.full_runtime&&state.player.hp>0&&state.enemies.iter().any(|enemy|enemy.hp>0&&enemy.life==ActorLife::Active)
        &&!state.enemies.iter().any(|enemy|enemy.full_powers.iter().any(|power|power.id.0=="back_attack")
            ||matches!(enemy.enemy_id,ActorEnemyId::Full(id) if super::enemies::is_boss_type(id)))
}
impl Combat{pub(crate) fn native_smoke_allowed(&self)->bool{allowed(&self.state)}}
