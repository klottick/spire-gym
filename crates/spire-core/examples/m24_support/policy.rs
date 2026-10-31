//! Pure M24 choices: the frozen M23 policy (itself the frozen M22 policy) plus one
//! Campfire legality rule (design §11). No seed/case lookup, hidden state or simulation.
#[path="../m23_support/policy.rs"]mod m23;
pub use m23::{PublicCallerProgress,act_progress};
use spire_core::run::{GameAction,PublicDecision,RoomChoiceKind,RoomChoiceStatus,RoomViewKind,map::PublicMap};
pub fn choose(d:&PublicDecision,map:Option<&PublicMap>,progress:&PublicCallerProgress)->Result<GameAction,String>{
    // The M22 rule rests only below half HP and otherwise needs Smith; with nothing
    // upgradable Smith is disabled, so take the always-offered Rest instead of erroring.
    if let Some(room)=d.room.as_ref().filter(|r|r.kind==RoomViewKind::Campfire){
        let offered=|k:&dyn Fn(&RoomChoiceKind)->bool|room.choices.iter().find(|c|c.status==RoomChoiceStatus::Offered&&k(&c.kind)&&d.candidates.iter().any(|x|x.action==GameAction::Choose(c.id)));
        if offered(&|k|*k==RoomChoiceKind::Smith).is_none()&&2*u64::from(d.player.hp)>=u64::from(d.player.max_hp){
            if let Some(row)=offered(&|k|matches!(k,RoomChoiceKind::Rest{..})){return Ok(GameAction::Choose(row.id))}
        }
    }
    m23::choose(d,map,progress)
}
