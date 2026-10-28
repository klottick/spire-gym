#[path="../examples/m17_support/mod.rs"]
#[allow(dead_code)]
mod m17_support;
use m17_support::*;
use spire_core::{combat::Outcome,content::EncounterId,run::*};
fn receipts()->Vec<serde_json::Value>{
    let root=std::env::var_os("M17_CASE_RECEIPTS").map(std::path::PathBuf::from).unwrap_or_else(||std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/m17"));
    FIXED_CASES.iter().map(|(seed,s)|serde_json::from_slice(&std::fs::read(root.join(format!("{seed}-{}.json",s.label()))).expect("retained original fixed-case receipt")).unwrap()).collect()
}
#[test]
fn m17_normal_cases_require_all_actual_boss_records_and_one_completion(){
    let mut entered=vec![];let mut finished=vec![];let mut wins=0;let mut ordinary=false;let mut elite=false;
    for value in receipts(){
        // Every case was already retained by the original source-bound driver.
        let (run,caller)=replay_receipt(&value).unwrap();let d=run.decision().unwrap();eprintln!("M17 retained seed={} scenario={} result={} actions={}",value["seed"],value["scenario"],value["result"],caller.accepted());
        for step in &caller.trace{for event in &step.transition.committed_events{match event{
            CommittedEvent::CombatEntered{visit,encounter}=>{
                if [EncounterId::Guardian,EncounterId::Hexaghost,EncounterId::SlimeBoss].contains(encounter){entered.push((*visit,*encounter));}
                else if [EncounterId::GremlinNob,EncounterId::Lagavulin,EncounterId::ThreeSentries].contains(encounter){elite=true}else{ordinary=true}
            },
            CommittedEvent::CombatFinished(record)=>{if entered.contains(&(record.visit,record.encounter)){finished.push(record.encounter);if record.outcome==Outcome::Victory&&d.room.as_ref().is_some_and(|r|r.kind==RoomViewKind::Act2Ready){wins+=1}}},_=>()
        }}}
        if d.room.as_ref().is_some_and(|r|r.kind==RoomViewKind::Act2Ready){assert_eq!((d.act,d.floor,d.terminal),(2,17,None));assert!(d.candidates.is_empty());assert_eq!(d.player.hp,d.player.max_hp);}
    }
    assert!(ordinary&&elite,"actual ordinary/elite coverage unmet");
    assert!([EncounterId::Guardian,EncounterId::Hexaghost,EncounterId::SlimeBoss].iter().all(|id|finished.contains(id)),"M17 normal evidence gate unmet: matching boss records {finished:?}");
    assert!(wins>=1,"M17 normal evidence gate unmet: no canonical Act2Ready completion");
}
#[test]
fn m17_death_budget_and_invalid_controls_remain_distinct(){
    for seed in [7,11]{
        let mut r=make_run(seed,Scenario::Death).unwrap();let mut c=Caller::new(10000);assert_eq!(rollout(&mut r,&mut c,Scenario::Death).unwrap(),CallerOutcome::Death);assert_eq!(r.decision().unwrap().terminal,Some(RunTerminal::Death));
        for budget in [0,1]{let mut r=make_run(seed,Scenario::Guardian).unwrap();let mut c=Caller::new(budget);assert_eq!(rollout(&mut r,&mut c,Scenario::Guardian).unwrap(),CallerOutcome::BudgetExhausted);assert_eq!(c.accepted(),budget);assert_eq!(r.decision().unwrap().terminal,None);}
    }
    for args in [vec!["--seed","8"],vec!["--scenario","other"],vec!["--budget","10001"],vec!["--budget","-1"]]{assert!(parse_args(args.into_iter().map(String::from)).is_err());}
    let mut r=make_run(7,Scenario::Guardian).unwrap();let d=r.decision().unwrap();let map=r.public_map();assert!(r.step(&GameAction::Confirm).is_err());assert_eq!(r.decision().unwrap(),d);assert_eq!(r.public_map(),map);
}
