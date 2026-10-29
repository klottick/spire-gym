//! Caller boundary checks; prescribed gameplay is separately released/ignored.
#[path="../examples/m13_support/mod.rs"]
mod shared;
use spire_core::{combat::rng::Partition,content::Character,run::*};
fn run()->Run {
    let mut p=CreationProfile::experienced();p.neow_eligible=false;
    Run::new_room_scenario(RunConfig{character:Character::Defect,ascension:0},p,
        RunRng::new([7;32],Partition::Test,0),RoomScenario::M20DefectHeart).unwrap()
}
fn first(d:&PublicDecision,_:Option<&map::PublicMap>,_:&shared::Caller)->Result<GameAction,String>{
    Ok(d.candidates.first().ok_or("no candidate")?.action.clone())
}
#[test]
fn m20_caller_uses_shared_trace_and_budget_boundary(){
    assert!(shared::Caller::new_m20(8193).is_err());
    let mut r=run();let original=r.clone();let mut caller=shared::Caller::new_m20(0).unwrap();
    assert_eq!(shared::rollout_m20(&mut r,&mut caller,|_,_,_|panic!("zero budget must not invoke policy")).unwrap(),shared::M20CallerOutcome::BudgetExhausted);
    assert_eq!(r,original);assert!(caller.trace.is_empty());assert_eq!(caller.final_decision,Some(r.decision().unwrap()));
    let mut caller=shared::Caller::new_m20(1).unwrap();
    assert_eq!(shared::rollout_m20(&mut r,&mut caller,first).unwrap(),shared::M20CallerOutcome::BudgetExhausted);
    assert_eq!(caller.trace.len(),1);assert_eq!(caller.trace[0].decision,original.decision().unwrap());
    assert_eq!(caller.final_decision,Some(r.decision().unwrap()));assert!(r.public_map().is_some());
}
#[test]
fn m20_caller_rejects_wrong_owner_and_illegal_public_action(){
    let mut r=run();let original=r.clone();let mut wrong=shared::Caller::new_m15(1).unwrap();
    assert!(shared::rollout_m20(&mut r,&mut wrong,first).is_err());assert_eq!(r,original);
    let mut caller=shared::Caller::new_m20(1).unwrap();
    assert!(shared::rollout_m20(&mut r,&mut caller,|_,_,_|Ok(GameAction::Choose(ChoiceId(u64::MAX-100)))).is_err());
    assert_eq!(r,original);assert!(caller.trace.is_empty());
}
