//! Pure M23 choices: the frozen M22 policy with per-act public route progress.
#[path="../m22_support/policy.rs"]mod m22;
pub use m22::PublicCallerProgress;
use spire_core::run::{GameAction,PublicDecision,map::PublicMap};
pub fn choose(d:&PublicDecision,map:Option<&PublicMap>,progress:&PublicCallerProgress)->Result<GameAction,String>{
    // Only Act2-reachable AncientWriting differs: take Simplicity when actually offered.
    if let Some(room)=d.room.as_ref().filter(|r|r.kind==RoomViewKind::Event(spire_core::content::EventId::AncientWriting)){
        if let Some(row)=room.choices.iter().find(|c|c.kind==RoomChoiceKind::AncientSimplicity&&c.status==spire_core::run::RoomChoiceStatus::Offered&&c.price.is_none()&&d.candidates.iter().any(|x|x.action==GameAction::Choose(c.id))){return Ok(GameAction::Choose(row.id))}
    }
    m22::choose(d,map,progress)
}
use spire_core::run::{CommittedEvent,RoomChoiceKind,RoomViewKind,RunTransition};
/// Public route witness from accepted steps (decision, action, transition).
pub fn act_progress<'a>(steps:impl IntoIterator<Item=(&'a PublicDecision,&'a GameAction,&'a RunTransition)>,act:u8)->PublicCallerProgress{
    let mut p=PublicCallerProgress::default();
    for (d,action,t) in steps{
        // Route witnesses are per act; purchases stay keyed by (act,floor).
        if d.act==act{for event in &t.committed_events{if let CommittedEvent::CombatEntered{encounter,..}=event{
            if let Some((_,_,kind))=spire_core::combat::enemies::A0_ENCOUNTER_SETUPS.iter().find(|(id,_,_)|id==encounter){
                p.ordinary_reached|=*kind==spire_core::combat::enemies::EncounterKind::Monster;p.elite_reached|=*kind==spire_core::combat::enemies::EncounterKind::Elite;
            }
        }}}
        if d.room.as_ref().is_some_and(|room|room.kind==RoomViewKind::Shop&&room.choices.iter().any(|row|*action==GameAction::Choose(row.id)&&matches!(row.kind,RoomChoiceKind::Card{..})&&row.price.is_some())){p.purchased_shops.insert((d.act,d.floor));}
    }p
}
