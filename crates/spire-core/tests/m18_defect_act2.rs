#[path="../examples/m18_support/mod.rs"]
#[allow(dead_code)]
mod m18_support;
use m18_support::*;
use spire_core::{combat::Outcome,content::EncounterId,run::*};
fn receipts()->Vec<serde_json::Value>{
    let root=std::env::var_os("M18_CASE_RECEIPTS").map(std::path::PathBuf::from).unwrap_or_else(||std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/m18"));
    FIXED_CASES.iter().map(|(seed,s)|serde_json::from_slice(&std::fs::read(root.join(format!("{seed}-{}.json",s.label()))).expect("retained original M18 fixed-case receipt")).unwrap()).collect()
}
#[test]
#[ignore = "strategy outcome gate (normal-start policy must reach/beat bosses); out of repo scope per primary scope clarification 2026-10-30; run with --ignored"]
fn m18_from_start_fixed_pairs_require_actual_boss_entry_and_completion(){
    let values=receipts();assert_eq!(values.len(),6);let mut act2_boss_records=vec![];let mut wins=0;let mut ordinary=false;let mut elite=false;
    for ((seed,scenario),value) in FIXED_CASES.iter().zip(values){
        let (run,caller)=replay_receipt(&value).unwrap();let d=run.decision().unwrap();let first=&caller.trace[0].decision;
        assert_eq!((first.player.hp,first.player.max_hp,first.player.gold,first.objects.len()),(75,75,99,10));assert_eq!(first.room.as_ref().unwrap().kind,RoomViewKind::Neow);assert!(caller.trace[0].map.is_none());
        eprintln!("M18 original seed={seed} pair={} result={} actions={} final={:?}",scenario.label(),value["result"],caller.accepted(),d.terminal);
        let mut entered=vec![];let mut first_boss_record=None;let mut act2_boss_record=None;
        for step in &caller.trace{for event in &step.transition.committed_events{match event{
            CommittedEvent::CombatEntered{visit,encounter}=>{entered.push((*visit,*encounter));if step.decision.act==2 {if [EncounterId::GremlinLeader,EncounterId::Slavers,EncounterId::BookOfStabbing].contains(encounter){elite=true}else if *encounter!=scenario.act2_boss{ordinary=true}}},
            CommittedEvent::CombatFinished(record)=>{assert!(entered.contains(&(record.visit,record.encounter)),"record must match actual entered visit");if record.encounter==scenario.act1_boss{first_boss_record=Some(record.clone())}if record.encounter==scenario.act2_boss{act2_boss_record=Some(record.clone());act2_boss_records.push(record.encounter)}},
            CommittedEvent::ActAdvanced{from:1,to:2}=>{assert_eq!(first_boss_record.as_ref().map(|r|r.outcome),Some(Outcome::Victory));},_=>()
        }}}
        if d.room.as_ref().is_some_and(|r|r.kind==RoomViewKind::Act3Ready){assert_eq!((d.act,d.floor,d.terminal),(3,34,None));assert!(d.candidates.is_empty());assert_eq!(d.player.hp,d.player.max_hp);assert_eq!(act2_boss_record.as_ref().map(|r|r.outcome),Some(Outcome::Victory));wins+=1;}
        if d.terminal==Some(RunTerminal::Death){assert_eq!(value["result"],"Ok(Death)");}
    }
    assert!(ordinary&&elite,"M18 original normal gate unmet: actual Act2 ordinary={ordinary}, elite={elite}");
    assert!([EncounterId::Champ,EncounterId::BronzeAutomaton,EncounterId::Collector].iter().all(|id|act2_boss_records.contains(id)),"M18 normal gate unmet: entered/matching Act2 boss records {act2_boss_records:?}");
    assert!(wins>=1,"M18 normal gate unmet: no canonical Act3Ready completion");
}
#[test]
fn m18_death_budget_and_invalid_actions_are_distinct(){
    for seed in [7,11]{
        let scenario=FIXED_CASES.iter().find(|(s,_)|*s==seed).unwrap().1;
        let mut r=make_run(seed,scenario).unwrap();let mut c=Caller::nonattacking(10000);assert_eq!(rollout(&mut r,&mut c,scenario).unwrap(),CallerOutcome::Death);assert_eq!(r.decision().unwrap().terminal,Some(RunTerminal::Death));
        for budget in [0,1]{let mut r=make_run(seed,scenario).unwrap();let mut c=Caller::new(budget);assert_eq!(rollout(&mut r,&mut c,scenario).unwrap(),CallerOutcome::BudgetExhausted);assert_eq!(c.accepted(),budget);assert_eq!(r.decision().unwrap().terminal,None);}
    }
    for args in [vec!["--seed","8"],vec!["--scenario","other"],vec!["--budget","10001"],vec!["--budget","-1"]]{assert!(parse_args(args.into_iter().map(String::from)).is_err());}
    let mut r=make_run(7,FIXED_CASES[0].1).unwrap();let d=r.decision().unwrap();let map=r.public_map();assert!(r.step(&GameAction::Confirm).is_err());assert_eq!(r.decision().unwrap(),d);assert_eq!(r.public_map(),map);
    let mut c=Caller::new(0);assert_eq!(rollout(&mut r,&mut c,FIXED_CASES[0].1),Ok(CallerOutcome::BudgetExhausted));let receipt=action_receipt(7,FIXED_CASES[0].1,&c,&Ok(CallerOutcome::BudgetExhausted));assert!(replay_receipt(&receipt).is_ok());
}
