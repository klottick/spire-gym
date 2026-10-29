#[path="../examples/m19_support/mod.rs"]
#[allow(dead_code)]
mod m19_support;
use m19_support::*;
use spire_core::{combat::Outcome,content::EncounterId,run::*};
#[test]
fn m19_death_budget_and_fault_remain_distinct(){
    let scenario=FIXED_CASES[0].1;
    for budget in [0,1]{let mut r=make_run(7,scenario).unwrap();let mut c=Caller::new(budget);assert_eq!(rollout(&mut r,&mut c,scenario),Ok(CallerOutcome::BudgetExhausted));assert_eq!(c.accepted(),budget);assert_eq!(r.decision().unwrap().terminal,None);}
    for args in [vec!["--seed","8"],vec!["--scenario","other"],vec!["--budget","10001"],vec!["--budget","-1"]]{assert!(parse_args(args.into_iter().map(String::from)).is_err());}
    let mut r=make_run(7,scenario).unwrap();let before=r.clone();assert!(r.step(&GameAction::Confirm).is_err());assert_eq!(r,before);
}
#[test]
fn m19_six_fixed_triples_require_actual_boss_records_and_normal_no_keys(){
    let root=std::env::var_os("M19_CASE_RECEIPTS").map(std::path::PathBuf::from).unwrap_or_else(||std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/m19"));
    // Load and preserve all attempts before any aggregate gate can fail.
    let values=FIXED_CASES.iter().map(|(seed,s)|serde_json::from_slice::<serde_json::Value>(&std::fs::read(root.join(format!("{seed}-{}.json",s.label()))).expect("original receipt required")).unwrap()).collect::<Vec<_>>();
    let mut bosses=vec![];let mut wins=0;
    for ((seed,scenario),value) in FIXED_CASES.iter().zip(values){
        let (run,caller)=replay_receipt(&value).unwrap();let first=&caller.trace[0];assert_eq!((first.decision.player.hp,first.decision.player.max_hp,first.decision.player.gold,first.decision.objects.len()),(75,75,99,10));assert_eq!(first.decision.room.as_ref().unwrap().kind,RoomViewKind::Neow);assert!(first.map.is_none());
        eprintln!("M19 original {seed} {} result={} actions={}",scenario.label(),value["result"],caller.accepted());
        let mut entries=vec![];let mut records=vec![];let mut ordinary=[false;3];let mut elite=[false;3];
        for step in &caller.trace{
            if let Some(map)=&step.map{let expected=match step.decision.act{1=>scenario.act1_boss,2=>scenario.act2_boss,3=>scenario.act3_boss,_=>panic!("outside finite M19")};assert_eq!(map.boss,Some(expected),"actual generated icon for reached act");}
            for e in &step.transition.committed_events{match e{
            CommittedEvent::CombatEntered{visit,encounter}=>{entries.push((*visit,*encounter));let act=step.decision.act as usize-1;let node=step.action.clone();if let GameAction::Move(id)=node{let kind=step.map.as_ref().unwrap().nodes.iter().find(|n|n.id==id).unwrap().kind;if kind==MapNodeKind::Elite{elite[act]=true}else if kind!=MapNodeKind::Boss{ordinary[act]=true}}},
            CommittedEvent::CombatFinished(record)=>{assert!(entries.contains(&(record.visit,record.encounter)));if step.decision.act==3&&record.encounter==scenario.act3_boss{bosses.push(record.encounter)}records.push(record.clone());},
            CommittedEvent::ActAdvanced{from,to}=>{assert!(ordinary[*from as usize-1]&&elite[*from as usize-1]);let boss=if *from==1{scenario.act1_boss}else{scenario.act2_boss};assert!(records.iter().any(|r|r.encounter==boss&&r.outcome==Outcome::Victory));assert_eq!(*to,*from+1);},_=>()
        }}}
        let d=run.decision().unwrap();if d.terminal==Some(RunTerminal::Act3NoKeys){assert!(ordinary[2]&&elite[2]);assert_eq!((d.act,d.floor),(3,50));assert!(records.iter().any(|r|r.encounter==scenario.act3_boss&&r.outcome==Outcome::Victory));wins+=1;}
        if d.terminal==Some(RunTerminal::Death){assert_eq!(value["result"],"Ok(Death)");}
    }
    assert!([EncounterId::AwakenedOne,EncounterId::TimeEater,EncounterId::DonuAndDeca].iter().all(|id|bosses.contains(id)),"M19 normal gate unmet: actual Act3 entries/matching records {bosses:?}");assert!(wins>0,"M19 normal gate unmet: no canonical Act3NoKeys");
}
