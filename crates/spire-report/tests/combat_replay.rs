//! Protect native replay parity, pre-action card identity, privacy and sink isolation.
use spire_env::*;
use spire_report::*;
use serde_json::{json,Value};
use std::{sync::Arc,io::{self,Write}};

fn episode(complete:bool)->(CombatPool,EpisodeRecord) {
    let p=CombatPool::new(Arc::new(ScenarioCatalog::starter()),PoolConfig { record_replay:true,..PoolConfig::default() }).unwrap();
    let mut tokens=p.reset_batch(&[0],&[ResetRequest {seed:Some(IntegerSeed::from_be_bytes(&[7]).unwrap()),scenario_id:None}]).unwrap().decisions.tokens;
    tokens=p.step_batch(&[0],&tokens).unwrap().current.tokens;
    for _ in 0..128 {
        let rows=p.read_rows(&[0]).unwrap();
        let TensorValues::I8(mask)=&rows.observations.tensors.iter().find(|t|t.key=="action_mask").unwrap().values else {panic!("mask")};
        let index=mask[1..32].iter().position(|&v|v==1).map_or(0,|i|i as i64+1);
        let result=p.step_batch(&[index],&tokens).unwrap();tokens=result.current.tokens;
        if !complete||result.terminated[0] {let record=p.reproduction(0).unwrap().unwrap();return (p,record);}
    }
    panic!("finite supplied combat must end")
}
fn header()->Value { json!({"record":"header","schemaId":"spire-report-v1","schemaVersion":1,
    "source":{"kind":"combat_episode","profile":"supplied combat","producerRevision":"5b1cd19+viewer","contentId":spire_core::combat::CONTENT_ID,"evidenceGrade":"source-grounded"},
    "run":{"character":"ironclad","ascension":0,"seed":{"encoding":"unsigned decimal","value":"7"},"caller":null}}) }
fn capture(record:&EpisodeRecord)->(Value,ReplayResult) {
    let mut writer=ReportWriter::new(Vec::new(),header()).unwrap();
    let result=capture_combat(record,&mut writer).unwrap();
    let report=read_report(&writer.into_inner()[..],64*1024*1024).unwrap();
    (report,result)
}

#[test]
fn combat_replay_actual() {
    let (pool,record)=episode(true);
    let (report,result)=capture(&record);
    assert_eq!(result,replay(&record).unwrap());
    assert_eq!(result.final_snapshot,pool.diagnostics().unwrap().slots[0].snapshot.clone().unwrap());
    let frames=report["frames"].as_array().unwrap();
    assert_eq!(frames.len(),record.actions.len()+1);
    assert_eq!(frames.iter().filter(|f|f["kind"]=="action").count(),record.actions.len());
    let play=frames.iter().find(|f|f["action"]["type"]=="play_card").unwrap();
    assert!(play["action"]["card"]["ref"].as_str().unwrap().contains("/card/"));
    assert!(play["effects"].as_array().unwrap().iter().any(|e|e["kind"]=="damage"||e["kind"]=="block"));
}
#[test]
fn combat_replay_terminal() {
    let (_,record)=episode(true);let(report,_)=capture(&record);
    let last=report["frames"].as_array().unwrap().last().unwrap();
    assert!(last["action"]["card"].is_object());
    assert_eq!(last["state"]["combat"]["enemies"][0]["hp"],0);
    assert_eq!(report["game"]["outcome"],"combat_victory");
    assert!(last["effects"].as_array().unwrap().iter().any(|e|e["kind"]=="heal"));
}
#[test]
fn combat_replay_prefix() {
    let (_,mut record)=episode(false);
    for (status,expected) in [(RecordStatus::Active,"unfinished"),(RecordStatus::Abandoned,"abandoned"),(RecordStatus::Faulted,"faulted")] {
        record.status=status;
        record.fault=(status==RecordStatus::Faulted).then(||SlotFault {slot:None,stage:FaultStage::Export,message:"external export failure".into(),core_fault:None});
        let (report,_)=capture(&record);assert_eq!(report["capture"]["status"],expected);assert!(report["game"]["outcome"].is_null());
    }
    record.attempted_action=Some(0);
    let mut writer=ReportWriter::new(Vec::new(),header()).unwrap();
    assert!(capture_combat(&record,&mut writer).is_err());
    let report=read_report(&writer.into_inner()[..],64*1024*1024).unwrap();
    assert_eq!(report["frames"].as_array().unwrap().len(),record.actions.len()+1,"successful external fault attempt never becomes accepted frame");
    assert_eq!(report["capture"]["status"],"interrupted");
}
#[test]
fn combat_replay_privacy() {
    let(_,record)=episode(false);let(report,_)=capture(&record);
    fn inspect(v:&Value) {match v {
        Value::Object(m)=>for(k,v)in m {assert!(!["root","key","cursor","pending","revealed_order"].contains(&k.as_str()));inspect(v)},
        Value::Array(a)=>for v in a {inspect(v)},_=>(),
    }}
    inspect(&report);
    assert_eq!(report["frames"][0]["state"]["combat"]["piles"]["draw"]["knownCards"],Value::Null);
    assert_eq!(report["frames"][0]["state"]["combat"]["piles"]["draw"]["orderKnown"],false);
    assert!(report["frames"][0]["state"]["inventory"].is_null());
}
#[test]
fn combat_replay_visitor_failure() {
    struct Broken;
    impl Write for Broken {fn write(&mut self,_:&[u8])->io::Result<usize>{Err(io::Error::other("sink full"))}fn flush(&mut self)->io::Result<()>{Ok(())}}
    let(pool,record)=episode(false);let before=pool.diagnostics().unwrap();
    assert!(replay_observed(&record,|_|Err("visitor stopped".into())).is_err());
    assert_eq!(pool.diagnostics().unwrap(),before);
    assert_eq!(pool.reproduction(0).unwrap().unwrap(),record);
    assert_eq!(replay(&record).unwrap().boundaries.len(),record.actions.len()+1);
    assert!(ReportWriter::new(Broken,header()).is_err());
    struct Sink {bytes:Vec<u8>,fail:std::rc::Rc<std::cell::Cell<bool>>}
    impl Write for Sink {fn write(&mut self,b:&[u8])->io::Result<usize>{if self.fail.get(){Err(io::Error::other("sink full"))}else{self.bytes.extend(b);Ok(b.len())}}fn flush(&mut self)->io::Result<()>{Ok(())}}
    let fail=std::rc::Rc::new(std::cell::Cell::new(false));
    let mut writer=ReportWriter::new(Sink {bytes:vec![],fail:fail.clone()},header()).unwrap();
    fail.set(true);
    assert!(capture_combat(&record,&mut writer).is_err());
    assert_eq!(pool.diagnostics().unwrap(),before);
    assert_eq!(pool.reproduction(0).unwrap().unwrap(),record);
}
