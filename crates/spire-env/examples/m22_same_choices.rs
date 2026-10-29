//! Consume released, already recorded actions. This executable contains no policy.
//! Originals and heterogeneous prefixes stay HELD until main releases their exact envelope.
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
use spire_core::{combat::rng::Partition,content::Character,run::{GameAction,PublicDecision}};
use spire_env::*;
use std::{collections::BTreeSet,path::{Path,PathBuf}};
type Result<T>=std::result::Result<T,String>;
fn check(ok:bool,message:&str)->Result<()>{if ok{Ok(())}else{Err(message.into())}}
fn native<T>(r:std::result::Result<T,impl std::fmt::Debug>)->Result<T>{r.map_err(|e|format!("{e:?}"))}
fn hash(bytes:&[u8])->String{format!("{:x}",Sha256::digest(bytes))}
fn text(v:&Value)->Result<&str>{v.as_str().ok_or_else(||"expected string".into())}
fn read(path:&Path,expected:&str)->Result<Value>{let bytes=std::fs::read(path).map_err(|e|e.to_string())?;check(hash(&bytes)==expected,"released receipt hash mismatch")?;serde_json::from_slice(&bytes).map_err(|e|e.to_string())}
fn config(boss:&str)->Result<RunSessionConfig>{Ok(RunSessionConfig{character:Character::Watcher,ascension:0,partition:Partition::Test,boss_history:match boss{"guardian"=>RunBossHistory::Act1GuardianUnseen,"hexaghost"=>RunBossHistory::Act1HexaghostUnseen,"slime_boss"=>RunBossHistory::Act1SlimeBossUnseen,_=>return Err("foreign M22 boss history".into())},capacities:RunCapacities::m10_default(),record_replay:true})}
struct Case{value:Value,seed:u64,config:RunSessionConfig,receipt_sha256:String}
fn case(base:&Path,row:&Value,freeze:&str)->Result<Case>{
 // The release row is the independent expected identity, checked before any Run exists.
 let expected_seed=row["seed"].as_u64().ok_or("missing released seed")?;let expected_boss=text(&row["act1_boss"])?;
 check([7,11].contains(&expected_seed),"unreleased seed")?;check(row["partition"]=="test"&&row["ordinal"]==0,"normal frozen envelope requires test/ordinal0")?;
 let receipt_sha256=text(&row["sha256"])?.to_owned();let path=base.join(text(&row["path"])?);let value=read(&path,&receipt_sha256)?;
 check(value["schema_id"]=="m22-watcher-act1-caller-attempt-v1"&&value["caller_revision"]=="m22-public-caller-v1","foreign caller receipt")?;
 check(value["source_freeze_sha256"]==freeze&&value["provenance"]["kind"]=="normal"&&value["action_budget"]==32768,"receipt provenance/freeze/budget mismatch")?;
 check(value["seed"]==expected_seed&&value["scenario"]["act1_boss"]==expected_boss,"released case identity mismatch")?;
 check(value["accepted_action_count"].as_u64()==Some(value["steps"].as_array().ok_or("missing steps")?.len() as u64),"accepted count mismatch")?;
 Ok(Case{config:config(expected_boss)?,value,seed:expected_seed,receipt_sha256})
}
/// Decode through the existing strict profile codec, never through candidate indexes.
fn action(template:&RunEpisodeRecord,value:&Value)->Result<GameAction>{let mut v:Value=serde_json::from_slice(&native(template.to_json())?).map_err(|e|e.to_string())?;v["steps"]=json!([{"action":value,"reward":0,"terminal":null}]);v["terminal"]=Value::Null;v["fault"]=Value::Null;let decoded=native(RunEpisodeRecord::from_json(&serde_json::to_vec(&v).map_err(|e|e.to_string())?))?;Ok(decoded.steps[0].action.clone())}
fn public(rep:&RunReproduction)->Result<&PublicDecision>{rep.latest_decision.as_ref().ok_or("missing Run")?.as_ref().map_err(|e|format!("{e:?}"))}
fn snapshot(rep:&RunReproduction,row:&Value,_prefix:bool)->Result<()>{
 let decision=public(rep);let expected=&row["decision_debug"];
 match decision{Ok(d)=>check(expected.as_str()==Some(format!("{d:?}").as_str()),"public decision/resources/records/candidates differ")?,Err(error)=>check(row["decision_error"].as_str()==Some(error.as_str()),"public core fault differs")?,}
 check(row["map_debug"].as_str()==Some(format!("{:?}",rep.latest_map).as_str()),"public map differs")
}
fn index(d:&PublicDecision,a:&GameAction,crosscheck:&Value)->Result<RunActionIndex>{let i=d.candidates.iter().position(|c|&c.action==a).ok_or("recorded semantic action is not currently offered")?;if !crosscheck.is_null(){check(crosscheck.as_u64()==Some(i as u64),"recorded candidate order mismatch")?;}Ok(RunActionIndex(i as i64))}
fn scalar(case:&Case)->Result<(RunEpisodeRecord,Vec<Option<RunStepResult>>)>{
 let mut s=native(RunSession::new_m22(case.config.clone()))?;native(s.reset(Some(case.seed)))?;let template=native(s.record())?.ok_or("record disabled")?;
 check(template.seed==case.seed&&template.ordinal==0&&template.config.partition==Partition::Test,"initial record identity mismatch")?;
 snapshot(&native(s.reproduction())?,&case.value["provenance"]["initial"],true)?;
 let mut results=Vec::new();
 for (i,row) in case.value["steps"].as_array().unwrap().iter().enumerate(){
  check(row["index"].as_u64()==Some(i as u64),"step order mismatch")?;let rep=native(s.reproduction())?;snapshot(&rep,&row["pre"],true)?;
  let a=action(&template,&row["action"])?;check(row["action_debug"].as_str()==Some(format!("{a:?}").as_str()),"semantic action debug mismatch")?;
  let action_index=index(public(&rep)?,&a,&row["candidate_index"])?;let b=native(s.decision())?;let step=match s.step(b.token,action_index){Ok(step)=>Some(step),Err(error)=>{let retained=native(s.record())?.ok_or("lost fault record")?;check(retained.steps.len()==i+1&&retained.fault.as_ref().is_some_and(|f|f.committed&&f.gameplay_recorded),"failed step did not retain accepted core transition")?;check(i+1==case.value["steps"].as_array().unwrap().len(),"fault before original endpoint")?;let _=error;None}};
  let retained=native(s.record())?.ok_or("record disabled")?;let transition=retained.steps.last().ok_or("missing accepted transition")?;
  check(row["transition"]["reward"].as_i64()==Some(i64::from(transition.reward))&&row["transition"]["terminal"].as_str()==Some(format!("{:?}",transition.terminal).as_str()),"reward or terminal mismatch")?;if let Some(step)=&step{check(!step.truncated&&step.terminated==step.boundary.terminal.is_some(),"terminal/truncation mismatch")?;}
  snapshot(&native(s.reproduction())?,&row["post"],false)?;results.push(step);
 }
 let failed=&case.value["failed_action"];
 if !failed.is_null(){
  let a=action(&template,&failed["action"])?;let rep=native(s.reproduction())?;let d=public(&rep)?;
  if let Ok(i)=index(d,&a,&failed["candidate_index"]){let token=native(s.decision())?.token;let error=s.step(token,i).err().ok_or("original failed action now succeeds")?;let RunSessionError::Core(error)=error else{return Err("original failure has different layer".into())};check(failed["error"].as_str()==Some(format!("{error:?}").as_str()),"original core fault checkpoint differs")?;}
 }
 snapshot(&native(s.reproduction())?,&case.value["endpoint"],false)?;
 let record=native(s.record())?.ok_or("missing final record")?;let parsed=native(RunEpisodeRecord::from_json(&native(record.to_json())?))?;check(parsed==record,"strict typed/JSON round trip differs")?;
 let report=native(replay_run(&parsed,case.config.capacities.clone()))?;
 check(report.accepted_count==record.steps.len()&&report.reported_fault==record.fault,"portable count/fault differs")?;
 let rep=native(s.reproduction())?;check(report.final_map==rep.latest_map,"portable final map differs")?;
 match rep.latest_decision.as_ref().unwrap(){Ok(d)=>check(&report.final_decision==d,"portable final public state differs")?,Err(error)=>check(report.final_decision_error.as_ref()==Some(error),"portable core fault differs")?,}
 check(report.rewards==record.steps.iter().map(|r|r.reward).collect::<Vec<_>>()&&report.terminals==record.steps.iter().map(|r|r.terminal).collect::<Vec<_>>(),"portable transition vectors differ")?;
 let before=native(s.reproduction())?;if record.terminal.is_some(){let b=native(s.decision())?;check(s.step(b.token,RunActionIndex(0))==Err(RunSessionError::Terminal),"terminal step accepted")?;}else if public(&before).is_ok_and(|d|d.candidates.is_empty()){let b=native(s.decision())?;check(s.step(b.token,RunActionIndex(0))==Err(RunSessionError::UnsupportedBoundary("M22 stops at Act2Ready")),"cutoff step accepted")?;}
 check(native(s.reproduction())?==before,"invalid endpoint step changed owner")?;native(s.close())?;check(native(s.record())?.unwrap().closed,"close lost record")?;
 Ok((record,results))
}
fn batch(case:&Case,record:&RunEpisodeRecord,expected:&[Option<RunStepResult>],workers:usize)->Result<()>{
 let mut b=native(RunBatch::new_m22(vec![case.config.clone();2],workers))?;let mut bounds=native(b.reset(&[Some(case.seed);2]))?;
 for (i,row) in case.value["steps"].as_array().unwrap().iter().enumerate(){let a=action(record,&row["action"])?;let mut indices=Vec::new();for slot in 0..2{let rep=native(b.reproduction(slot))?;snapshot(&rep,&row["pre"],true)?;indices.push(index(public(&rep)?,&a,&row["candidate_index"])?);}match b.step(&bounds.iter().map(|b|b.token).collect::<Vec<_>>(),&indices){Ok(steps)=>{let expected=expected[i].as_ref().ok_or("batch delivered a scalar fault row")?;for slot in 0..2{check(steps[slot].boundary.observation==expected.boundary.observation&&steps[slot].reward==expected.reward&&steps[slot].terminated==expected.terminated&&!steps[slot].truncated,"paired full-boundary scalar disagreement")?;snapshot(&native(b.reproduction(slot))?,&row["post"],false)?;}bounds=steps.into_iter().map(|s|s.boundary).collect();},Err(error)=>{check(expected[i].is_none()&&i+1==expected.len(),"unexpected batch execution fault")?;let RunBatchError::Fault{slots}=error else{return Err("preflight failed on accepted original action".into())};check(slots.iter().all(|f|f.committed&&!f.delivery_lost),"paired accepted fault retention mismatch")?;}}}
 let failed=&case.value["failed_action"];if !failed.is_null(){let a=action(record,&failed["action"])?;let rep=native(b.reproduction(0))?;if let Ok(i)=index(public(&rep)?,&a,&failed["candidate_index"]){check(b.step(&bounds.iter().map(|r|r.token).collect::<Vec<_>>(),&[i;2]).is_err(),"paired original failed action now succeeds")?;}}
 for slot in 0..2{check(native(b.record(slot))?.as_ref()==Some(record),"paired final retained record differs")?;snapshot(&native(b.reproduction(slot))?,&case.value["endpoint"],false)?;}
 let before=[native(b.reproduction(0))?,native(b.reproduction(1))?];check(b.step(&[bounds[0].token],&[RunActionIndex(0)]).is_err(),"partial batch step accepted")?;check([native(b.reproduction(0))?,native(b.reproduction(1))?]==before,"invalid batch call changed peers")?;
 native(b.close())?;Ok(())
}
fn run_release(path:&Path,expected:&str)->Result<Value>{
 let release=read(path,expected)?;check(release["schema_id"]=="m22-native-receipt-release-v1","foreign release envelope")?;let freeze=text(&release["freeze_manifest_sha256"])?;check(freeze.len()==64&&freeze.bytes().all(|b|b.is_ascii_hexdigit()),"invalid freeze hash")?;
 let rows=release["cases"].as_array().ok_or("missing released cases")?;check(rows.len()==6,"release must bind all six originals")?;let mut cases=Vec::new();let mut seen=BTreeSet::new();for row in rows{let c=case(path.parent().unwrap_or(Path::new(".")),row,freeze)?;check(seen.insert((c.seed,text(&c.value["scenario"]["act1_boss"])?.to_owned())),"duplicate original case")?;cases.push(c);}
 let m15=m15_prefix(path.parent().unwrap_or(Path::new(".")),&release["m15_prefix"])?;
 let mut prefixes=Vec::new();for case in &cases{prefixes.push(coexistence(case,&m15)?);}
 let mut reports=Vec::new();for case in &cases{reports.push(consume_case(case)?);}
 Ok(json!({"schema_id":"m22-native-same-choice-result-v1","freeze_manifest_sha256":freeze,"release_sha256":expected,"cases":reports,"coexistence_prefixes":prefixes,"m15_derived_current_prefix_record":serde_json::from_slice::<Value>(&native(m15.to_json())?).map_err(|e|e.to_string())?,"m15_grade":"new current typed/strict derivation from nominated static original32 actions; no historical portable-export or full191 claim","grade":"same released normal actions; no new policy/outcome run"}))
}
fn consume_case(case:&Case)->Result<Value>{
 let (record,steps)=scalar(case)?;batch(case,&record,&steps,1)?;batch(case,&record,&steps,2)?;
 Ok(json!({"seed":case.seed,"boss":case.value["scenario"]["act1_boss"],"receipt_sha256":case.receipt_sha256,"accepted":record.steps.len(),"outcome":case.value["outcome"],"fault":case.value["error"],"scalar_portable_workers1_workers2":"matched","record":serde_json::from_slice::<Value>(&native(record.to_json())?).map_err(|e|e.to_string())?}))
}
fn corrective_row(release:&Value)->Result<&Value>{
 check(release["schema_id"]=="m22-native-corrective-release-v1","foreign corrective release envelope")?;
 check(release["authority_sha256"]=="65ee85836ad0c4a7a3baee855c3fdb177ea53bce2f20f610e02990bca7369a88"&&release["original_fault_receipt_sha256"]=="6d040a8c086749c15815d4b368a856708cf834cf080b14d57de8338e39816611","wrong corrective authority/counterexample")?;
 let rows=release["cases"].as_array().ok_or("missing corrective cases")?;check(rows.len()==1,"correction admits exactly one case")?;
 let row=&rows[0];check(row["seed"]==11&&row["act1_boss"]=="slime_boss"&&row["partition"]=="test"&&row["ordinal"]==0,"foreign corrective case identity")?;Ok(row)
}
fn run_corrective_release(path:&Path,expected:&str)->Result<Value>{
 let release=read(path,expected)?;let row=corrective_row(&release)?;
 let freeze=text(&release["freeze_manifest_sha256"])?;check(freeze.len()==64&&freeze.bytes().all(|b|b.is_ascii_hexdigit()),"invalid corrective freeze hash")?;
 let case=case(path.parent().unwrap_or(Path::new(".")),row,freeze)?;
 Ok(json!({"schema_id":"m22-native-corrective-same-choice-result-v1","freeze_manifest_sha256":freeze,"release_sha256":expected,"case":consume_case(&case)?,"grade":"separately recorded seed11 SlimeBoss correction only; same choices scalar/strictJSON/portable/full homogeneous workers1/2; no original replacement, other five originals, M15 prefix or policy invocation"}))
}
fn execute(args:Vec<String>)->Result<Value>{match args.as_slice(){[mode,path,sha]if mode=="released"=>run_release(&PathBuf::from(path),sha),[mode,path,sha]if mode=="corrective-released"=>run_corrective_release(&PathBuf::from(path),sha),_=>Err("HELD: usage m22_same_choices released|corrective-released TRUSTED_RELEASE_JSON EXPECTED_RELEASE_SHA256".into())}}
#[cfg(test)]
mod corrective_tests{
 use super::*;
 fn envelope()->Value{json!({"schema_id":"m22-native-corrective-release-v1","authority_sha256":"65ee85836ad0c4a7a3baee855c3fdb177ea53bce2f20f610e02990bca7369a88","original_fault_receipt_sha256":"6d040a8c086749c15815d4b368a856708cf834cf080b14d57de8338e39816611","cases":[{"seed":11,"act1_boss":"slime_boss","partition":"test","ordinal":0}]})}
 #[test]
 fn corrective_preflight_rejects_foreign_cases_duplicates_and_original_envelope_before_startup(){
  let valid=envelope();assert!(corrective_row(&valid).is_ok());
  for (key,value) in [("seed",json!(7)),("act1_boss",json!("guardian")),("act1_boss",json!("hexaghost")),("partition",json!("train")),("ordinal",json!(1))]{let mut v=valid.clone();v["cases"][0][key]=value;assert!(corrective_row(&v).is_err());}
  for rows in [json!([]),json!([valid["cases"][0].clone(),valid["cases"][0].clone()])]{let mut v=valid.clone();v["cases"]=rows;assert!(corrective_row(&v).is_err());}
  for key in ["schema_id","authority_sha256","original_fault_receipt_sha256"]{let mut v=valid.clone();v[key]=json!("foreign");assert!(corrective_row(&v).is_err());}
  let mut original=valid;original["schema_id"]=json!("m22-native-receipt-release-v1");assert!(corrective_row(&original).is_err());
 }
}
fn main(){match execute(std::env::args().skip(1).collect()){Ok(value)=>println!("{value}"),Err(error)=>{eprintln!("{error}");std::process::exit(1)}}}

/// A retained M15 action prefix is an independent input, never a fresh policy series.
fn m15_prefix(base:&Path,row:&Value)->Result<RunEpisodeRecord>{
 const NOMINATED:&str="b98de6a89b2b07d5d3f37dc582f3b8954baa9dd01ff31b31be9163655a376477";
 check(row["format"]=="m22_static_m15_prefix"&&row["sha256"]==NOMINATED,"M15 input is not root's nominated static prefix")?;
 let v=read(&base.join(text(&row["path"])?),NOMINATED)?;
 check(v["schema_id"]=="m22-static-m15-coexistence-prefix-input-v1"&&v["prefix_length"]==32,"wrong static prefix schema/length")?;
 let identity=&v["expected_external_identity"];
 check(identity["profile"]=="M15SilentHeart"&&identity["seed"]==7&&identity["partition"]=="test"&&identity["ordinal"]==0,"wrong independent M15 identity")?;
 check(identity["scenario"]==json!({"act1_boss":"Guardian","act2_boss":"Champ","act3_boss":"AwakenedOne"})&&identity["capacities"]=="RunCapacities::m10_default()","wrong M15 scenario/capacities")?;
 let rows=v["actions"].as_array().ok_or("missing static M15 actions")?;let rewards=v["source_transition_rewards"].as_array().ok_or("missing original rewards")?;let terminals=v["source_terminals"].as_array().ok_or("missing original terminals")?;let indices=v["source_step_indexes"].as_array().ok_or("missing original indexes")?;
 check(rows.len()==32&&rewards.len()==32&&terminals.len()==32&&indices.len()==32,"wrong static prefix vectors")?;
 for i in 0..32{check(indices[i]==i&&terminals[i].is_null(),"M15 prefix order/terminal mismatch")?;}
 let cfg=RunSessionConfig{character:Character::Silent,ascension:0,partition:Partition::Test,boss_history:RunBossHistory::M15SilentHeart{act1:RunAct1BossTarget::Guardian,act2:RunAct2BossTarget::Champ,act3:RunAct3BossTarget::AwakenedOne},capacities:RunCapacities::m10_default(),record_replay:true};
 let mut s=native(RunSession::new_m15(cfg))?;native(s.reset(Some(7)))?;let template=native(s.record())?.unwrap();
 for i in 0..32{let a=action(&template,&rows[i])?;let rep=native(s.reproduction())?;let index=index(public(&rep)?,&a,&Value::Null)?;let token=native(s.decision())?.token;let step=native(s.step(token,index))?;check(rewards[i].as_i64()==Some(i64::from(step.reward))&&step.boundary.terminal.is_none(),"M15 retained prefix reward/terminal divergence")?;}
 // Newly derived current typed/strict representation, never a claimed historical export.
 let record=native(s.record())?.ok_or("missing derived M15 prefix record")?;let parsed=native(RunEpisodeRecord::from_json(&native(record.to_json())?))?;check(parsed==record,"M15 derived strict record mismatch")?;Ok(record)
}
fn dispatch_recorded(s:&mut RunSession,step:&RunRecordedStep)->Result<RunStepResult>{let rep=native(s.reproduction())?;let i=index(public(&rep)?,&step.action,&Value::Null)?;let token=native(s.decision())?.token;let actual=native(s.step(token,i))?;check((actual.reward,actual.boundary.terminal)==(step.reward,step.terminal),"prefix reward/terminal mismatch")?;Ok(actual)}
fn coexistence(case:&Case,m15:&RunEpisodeRecord)->Result<Value>{
 check(m15.profile_id=="M15SilentHeart"&&m15.ordinal==0&&m15.config.partition==Partition::Test&&m15.steps.len()>=32,"invalid retained M15 prefix")?;
 let n=32.min(case.value["steps"].as_array().unwrap().len());
 let mut watcher=native(RunSession::new_m22(case.config.clone()))?;let mut silent=native(RunSession::new_m15(m15.config.clone()))?;native(watcher.reset(Some(case.seed)))?;native(silent.reset(Some(m15.seed)))?;
 let template=native(watcher.record())?.unwrap();let mut wsteps=Vec::new();
 // Decode only actual original choices. The step reward/terminal are from its original receipt.
 for row in case.value["steps"].as_array().unwrap().iter().take(n){wsteps.push(RunRecordedStep{action:action(&template,&row["action"])? ,reward:i32::try_from(row["transition"]["reward"].as_i64().ok_or("invalid reward")?).map_err(|_|"reward outside int32")?,terminal:match text(&row["transition"]["terminal"])?{"None"=>None,"Some(Death)"=>Some(spire_core::run::RunTerminal::Death),"Some(EmptyDeck)"=>Some(spire_core::run::RunTerminal::EmptyDeck),_=>return Err("foreign M22 prefix terminal".into())}});}
 let mut expected_w=native(RunSession::new_m22(case.config.clone()))?;let mut expected_s=native(RunSession::new_m15(m15.config.clone()))?;native(expected_w.reset(Some(case.seed)))?;native(expected_s.reset(Some(m15.seed)))?;
 for step in &wsteps{dispatch_recorded(&mut expected_w,step)?;}for step in m15.steps.iter().take(n){dispatch_recorded(&mut expected_s,step)?;}
 let mut saw_combat=false;
 for i in 0..n{let w=dispatch_recorded(&mut watcher,&wsteps[i])?;let old=dispatch_recorded(&mut silent,&m15.steps[i])?;saw_combat|=public(&native(watcher.reproduction())?).is_ok_and(|d|d.combat.is_some());check(!w.truncated&&!old.truncated,"prefix silently truncated")?;}
 check(native(watcher.reproduction())?==native(expected_w.reproduction())?&&native(silent.reproduction())?==native(expected_s.reproduction())?,"interleaved prefix differs from isolated owners")?;
 for workers in [1,2]{for old in [false,true]{let cfg=if old{m15.config.clone()}else{case.config.clone()};let seed=if old{m15.seed}else{case.seed};let mut b=if old{native(RunBatch::new_m15(vec![cfg;2],workers))?}else{native(RunBatch::new_m22(vec![cfg;2],workers))?};let mut bounds=native(b.reset(&[Some(seed);2]))?;for i in 0..n{let step=if old{&m15.steps[i]}else{&wsteps[i]};let rep=native(b.reproduction(0))?;let i=index(public(&rep)?,&step.action,&Value::Null)?;let next=native(b.step(&bounds.iter().map(|r|r.token).collect::<Vec<_>>(),&[i;2]))?;bounds=next.into_iter().map(|r|r.boundary).collect();}let expected=if old{native(expected_s.reproduction())?}else{native(expected_w.reproduction())?};for slot in 0..2{check(native(b.reproduction(slot))?==expected,"separate homogeneous prefix batch differs")?;}}}
 check(n<32||saw_combat,"32-action coexistence prefix lacks an actual M22 combat boundary")?;
 Ok(json!({"seed":case.seed,"boss":case.value["scenario"]["act1_boss"],"accepted_per_lane":n,"actual_watcher_combat":saw_combat,"grade":"bounded interleaved coexistence plus separate homogeneous workers1/2; not full-history proof"}))
}
