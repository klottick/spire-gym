//! Retain native initial artifacts, then replay the core caller's recorded
//! semantic choices. This consumer never invokes a policy or selects an action.
#[path="../../spire-core/examples/m21_support/mod.rs"]mod support;
use support::{Scenario,SCENARIOS};
use spire_env::*;
use spire_core::combat::rng::Partition;
use serde_json::{Value,json};
use sha2::{Digest,Sha256};
fn sha(s:&str)->String{Sha256::digest(s.as_bytes()).iter().map(|b|format!("{b:02x}")).collect()}
fn catalog()->Result<CombatTrainingCatalog,String>{CombatTrainingCatalog::m21(SCENARIOS.into_iter().map(|s|M21WatcherCombatEntry{id:s.name().into(),encounter:s.encounter(),start:s.start()}).collect()).map_err(|e|format!("catalog: {e:?}"))}
fn prepare(seed:u64,scenario:Scenario)->Result<Value,String>{
 let mut session=CombatTrainingSession::new_m21(catalog()?,Partition::Test,combat_policy_capacities()).map_err(|e|format!("session: {e:?}"))?;
 let boundary=session.reset(Some(seed),Some(scenario.name())).map_err(|e|format!("initial native boundary: {e:?}"))?;
 let record=session.record().map_err(|e|format!("initial record: {e:?}"))?.ok_or("missing initial record")?;
 Ok(json!({"kind":"native_initial","scenario":scenario.name(),"seed":seed,"catalog_descriptor":session.catalog_descriptor(),"environment_spec":session.environment_spec().map_err(|e|format!("environment: {e:?}"))?,"observation_spec":session.observation_spec().map_err(|e|format!("descriptor: {e:?}"))?.descriptor(),"record":serde_json::from_slice::<Value>(&record.to_json().map_err(|e|format!("record codec: {e:?}"))?).map_err(|e|e.to_string())?,"initial":{"info":boundary.info,"action_mask":boundary.observation.action_mask,"groups":boundary.observation.groups.iter().map(|g|json!({"key":g.key,"shape":g.shape,"values":g.values,"known":g.known,"occupied":g.occupied})).collect::<Vec<_>>()}}))
}
fn steps(value:&Value)->Result<Vec<Value>,String>{value.as_array().ok_or("missing core steps")?.iter().map(|s|{
 let o=s.as_object().ok_or("step must be object")?;
 if o.len()!=4||!["action","reward","terminal","core_reward"].iter().all(|k|o.contains_key(*k)){return Err("unexpected core step fields".into())}
 Ok(json!({"action":s["action"],"reward":s["reward"],"terminal":s["terminal"]}))
}).collect()}
fn replay(template:&[u8],transcript:&str)->Result<Value,String>{
 let initial:Value=serde_json::from_slice(template).map_err(|e|e.to_string())?;
 let events=transcript.lines().filter(|s|!s.is_empty()).map(serde_json::from_str::<Value>).collect::<Result<Vec<_>,_>>().map_err(|e|format!("core raw JSON: {e}"))?;
 let config=events.iter().find(|v|v["kind"]=="configuration").ok_or("missing configuration")?;
 let result=events.iter().find(|v|v["kind"]=="result").ok_or("missing raw result; partial transcript retained")?;
 if initial["scenario"]!=config["scenario"]||initial["seed"]!=config["seed"]||result["scenario"]!=config["scenario"]||result["seed"]!=config["seed"]{return Err("case identity mismatch".into())}
 let mut record=initial["record"].clone();record["steps"]=json!(steps(&result["steps"])?);record["outcome"]=result["outcome"].clone();
 if result["status"]=="fault"||result["status"]=="startup_fault"{
  // Raw core failures remain authoritative. A transcript error string is not
  // a native structured fault receipt; do not fabricate one after execution.
  record["outcome"]=Value::Null;
 }
 let bytes=serde_json::to_vec(&record).map_err(|e|e.to_string())?;
 let record=CombatEpisodeRecord::from_json(&bytes).map_err(|e|format!("shared record import: {e:?}"))?;
 let report=replay_m21_combat(&record,combat_policy_capacities()).map_err(|e|format!("same-choice semantic replay: {e:?}"))?;
 let decision=report.final_decision.as_ref().ok_or("no readable replay decision")?;
 let actual=sha(&format!("{decision:?}"));let expected=result["public_sha256"].as_str();let full=matches!(result["status"].as_str(),Some("settled"|"budget"));
 if full&&(Some(actual.as_str())!=expected||report.accepted_count!=record.steps.len()||format!("{:?}",report.outcome)!=format!("{:?}",record.outcome)){return Err("same-choice final public/count/outcome mismatch".into())}
 Ok(json!({"kind":"same_choice_replay","scenario":config["scenario"],"seed":config["seed"],"status":if full{"matched"}else{"accepted_prefix_only_core_fault_requires_disposition"},"accepted_steps":report.accepted_count,"rewards":report.rewards,"outcome":result["outcome"],"public_sha256":actual,"core_public_sha256":result["public_sha256"],"full_public_match":full,"fault":result["fault"],"record":serde_json::from_slice::<Value>(&record.to_json().map_err(|e|format!("record export: {e:?}"))?).map_err(|e|e.to_string())?}))
}
fn execute(args:Vec<String>)->Result<Value,String>{match args.as_slice(){
 [mode,seed,scenario] if mode=="prepare"=>prepare(seed.parse().map_err(|_|"invalid seed")?,Scenario::parse(scenario)?),
 [mode,template,transcript] if mode=="replay"=>replay(&std::fs::read(template).map_err(|e|e.to_string())?,&std::fs::read_to_string(transcript).map_err(|e|e.to_string())?),
 _=>Err("usage: m21_same_choices prepare SEED SCENARIO | replay INITIAL_JSON CORE_JSONL".into()),
}}
fn main(){match execute(std::env::args().skip(1).collect()){Ok(v)=>println!("{v}"),Err(e)=>{eprintln!("{e}");std::process::exit(1)}}}
#[cfg(test)]mod tests{use super::*;#[test]fn recorded_steps_use_shared_codec_shape_without_core_reward(){assert_eq!(steps(&json!([{"action":{"verb":"combat.end_turn"},"reward":0,"terminal":null,"core_reward":0}])).unwrap(),vec![json!({"action":{"verb":"combat.end_turn"},"reward":0,"terminal":null})]);assert!(steps(&json!([{"action":{},"reward":0,"terminal":null}])).is_err());}#[test]fn invalid_arguments_do_not_initialize(){assert!(execute(vec![]).is_err());assert!(execute(vec!["prepare".into(),"7".into(),"bad-scenario".into()]).is_err());}}
