//! Consume released, already recorded M24 actions. This executable contains no policy.
//! Mirrors the accepted M23 same-choices consumer: scalar session, strict typed/JSON
//! record, portable replay and full homogeneous batches (workers 1/2) per case, plus a
//! bounded interleaved M24/M23/M15/M20 coexistence prefix from retained recorded inputs.
//! `released-dry` consumes only labelled non-outcome receipts (budget controls under the
//! all-zero sentinel freeze) to validate this consumer before the M24 freeze.
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
use spire_core::{combat::rng::Partition,content::Character,run::{GameAction,PublicDecision,RunTerminal}};
use spire_env::*;
use std::{collections::BTreeSet,path::{Path,PathBuf}};
type Result<T>=std::result::Result<T,String>;
fn check(ok:bool,message:&str)->Result<()>{if ok{Ok(())}else{Err(message.into())}}
fn native<T>(r:std::result::Result<T,impl std::fmt::Debug>)->Result<T>{r.map_err(|e|format!("{e:?}"))}
fn hash(bytes:&[u8])->String{format!("{:x}",Sha256::digest(bytes))}
fn text(v:&Value)->Result<&str>{v.as_str().ok_or_else(||"expected string".into())}
fn read(path:&Path,expected:&str)->Result<Value>{let bytes=std::fs::read(path).map_err(|e|format!("{}: {e}",path.display()))?;check(hash(&bytes)==expected,"released input hash mismatch")?;serde_json::from_slice(&bytes).map_err(|e|e.to_string())}
fn act1(name:&str)->Result<RunAct1BossTarget>{Ok(match name{"guardian"=>RunAct1BossTarget::Guardian,"hexaghost"=>RunAct1BossTarget::Hexaghost,"slime_boss"=>RunAct1BossTarget::SlimeBoss,_=>return Err("unknown Act1 boss".into())})}
fn act2(name:&str)->Result<RunAct2BossTarget>{Ok(match name{"champ"=>RunAct2BossTarget::Champ,"bronze_automaton"=>RunAct2BossTarget::BronzeAutomaton,"collector"=>RunAct2BossTarget::Collector,_=>return Err("unknown Act2 boss".into())})}
fn act3(name:&str)->Result<RunAct3BossTarget>{Ok(match name{"awakened_one"=>RunAct3BossTarget::AwakenedOne,"time_eater"=>RunAct3BossTarget::TimeEater,"donu_and_deca"=>RunAct3BossTarget::DonuAndDeca,_=>return Err("unknown Act3 boss".into())})}
fn config(character:Character,boss_history:RunBossHistory)->RunSessionConfig{RunSessionConfig{character,ascension:0,partition:Partition::Test,boss_history,capacities:RunCapacities::m10_default(),record_replay:true}}
fn m23_config(a:&str,b:&str)->Result<RunSessionConfig>{Ok(config(Character::Watcher,RunBossHistory::M23WatcherAct2{act1:act1(a)?,act2:act2(b)?}))}
fn m24_config(a:&str,b:&str,c:&str)->Result<RunSessionConfig>{Ok(config(Character::Watcher,RunBossHistory::M24WatcherAct3{act1:act1(a)?,act2:act2(b)?,act3:act3(c)?}))}
/// The six approved triples: the release row is the independent expected identity.
const CASES:[(u64,&str,&str,&str);6]=[(7,"guardian","champ","awakened_one"),(7,"hexaghost","bronze_automaton","time_eater"),(7,"slime_boss","collector","donu_and_deca"),(11,"guardian","bronze_automaton","donu_and_deca"),(11,"hexaghost","collector","awakened_one"),(11,"slime_boss","champ","time_eater")];
const DRY_FREEZE:&str="0000000000000000000000000000000000000000000000000000000000000000";
struct Case{value:Value,seed:u64,config:RunSessionConfig,receipt_sha256:String}
fn case(base:&Path,row:&Value,freeze:&str,dry:bool)->Result<Case>{
 let seed=row["seed"].as_u64().ok_or("missing released seed")?;let (a,b,c)=(text(&row["act1_boss"])?,text(&row["act2_boss"])?,text(&row["act3_boss"])?);
 check(CASES.contains(&(seed,a,b,c)),"unreleased seed/boss triple")?;check(row["partition"]=="test"&&row["ordinal"]==0,"normal frozen envelope requires test/ordinal0")?;
 let receipt_sha256=text(&row["sha256"])?.to_owned();let value=read(&base.join(text(&row["path"])?),&receipt_sha256)?;
 check(value["schema_id"]=="m24-watcher-act3-caller-attempt-v1"&&value["caller_revision"]=="m24-public-caller-v1","foreign caller receipt")?;
 if dry{check(freeze==DRY_FREEZE&&value["source_freeze_sha256"]==DRY_FREEZE&&value["provenance"]["kind"]=="budget-control"&&value["action_budget"].as_u64().is_some_and(|b|b<32768),"dry input must be a labelled non-outcome budget control under the sentinel freeze")?;}
 else{check(freeze!=DRY_FREEZE&&value["source_freeze_sha256"]==freeze&&value["provenance"]["kind"]=="normal"&&value["action_budget"]==32768,"receipt provenance/freeze/budget mismatch")?;}
 check(value["seed"]==seed&&value["scenario"]["act1_boss"]==a&&value["scenario"]["act2_boss"]==b&&value["scenario"]["act3_boss"]==c,"released case identity mismatch")?;
 check(value["accepted_action_count"].as_u64()==Some(value["steps"].as_array().ok_or("missing steps")?.len() as u64),"accepted count mismatch")?;
Ok(Case{config:m24_config(a,b,c)?,value,seed,receipt_sha256})
}
/// Decode through the existing strict profile codec, never through candidate indexes.
fn action(template:&RunEpisodeRecord,value:&Value)->Result<GameAction>{let mut v:Value=serde_json::from_slice(&native(template.to_json())?).map_err(|e|e.to_string())?;v["steps"]=json!([{"action":value,"reward":0,"terminal":null}]);v["terminal"]=Value::Null;v["fault"]=Value::Null;v["closed"]=json!(false);let r=native(RunEpisodeRecord::from_json(&serde_json::to_vec(&v).map_err(|e|e.to_string())?))?;Ok(r.steps[0].action.clone())}
fn public(rep:&RunReproduction)->Result<&PublicDecision>{rep.latest_decision.as_ref().ok_or("missing Run")?.as_ref().map_err(|e|format!("{e:?}"))}
fn snapshot(rep:&RunReproduction,row:&Value)->Result<()>{
 match public(rep){Ok(d)=>check(row["decision_debug"].as_str()==Some(format!("{d:?}").as_str()),"public decision/resources/records/candidates differ")?,Err(error)=>check(row["decision_error"].as_str()==Some(error.as_str()),"public core fault differs")?,}
 check(row["map_debug"].as_str()==Some(format!("{:?}",rep.latest_map).as_str()),"public map differs")
}
fn index(d:&PublicDecision,a:&GameAction,crosscheck:&Value)->Result<RunActionIndex>{let i=d.candidates.iter().position(|c|&c.action==a).ok_or("recorded semantic action is not currently offered")?;if !crosscheck.is_null(){check(crosscheck.as_u64()==Some(i as u64),"recorded candidate order mismatch")?;}Ok(RunActionIndex(i as i64))}
fn terminal(v:&Value)->Result<Option<RunTerminal>>{Ok(match text(v)?{"None"=>None,"Some(Death)"=>Some(RunTerminal::Death),"Some(EmptyDeck)"=>Some(RunTerminal::EmptyDeck),"Some(Act3NoKeys)"=>Some(RunTerminal::Act3NoKeys),other=>return Err(format!("unexpected recorded terminal {other}"))})}
const CUTOFF:&str="M24 stops before the Act4 door";
fn scalar(case:&Case)->Result<(RunEpisodeRecord,Vec<Option<RunStepResult>>)>{
 let mut s=native(RunSession::new_m24(case.config.clone()))?;native(s.reset(Some(case.seed)))?;let template=native(s.record())?.ok_or("record disabled")?;
 check(template.seed==case.seed&&template.ordinal==0&&template.config.partition==Partition::Test,"initial record identity mismatch")?;
 snapshot(&native(s.reproduction())?,&case.value["provenance"]["initial"])?;
 let mut results=Vec::new();
 for (i,row) in case.value["steps"].as_array().unwrap().iter().enumerate(){
  check(row["index"].as_u64()==Some(i as u64),"step order mismatch")?;let rep=native(s.reproduction())?;snapshot(&rep,&row["pre"])?;
  let a=action(&template,&row["action"])?;check(row["action_debug"].as_str()==Some(format!("{a:?}").as_str()),"semantic action debug mismatch")?;
  let i_action=index(public(&rep)?,&a,&row["candidate_index"])?;let b=native(s.decision())?;
  let step=match s.step(b.token,i_action){Ok(step)=>Some(step),Err(error)=>{let retained=native(s.record())?.ok_or("lost fault record")?;check(retained.steps.len()==i+1&&retained.fault.as_ref().is_some_and(|f|f.committed&&f.gameplay_recorded),"failed step did not retain accepted core transition")?;check(matches!(error,RunSessionError::Core(_)),"publication fault layer differs")?;None}};
  let retained=native(s.record())?.ok_or("record disabled")?;let transition=retained.steps.last().ok_or("missing accepted transition")?;
  check(row["transition"]["reward"].as_i64()==Some(i64::from(transition.reward))&&terminal(&row["transition"]["terminal"])?==transition.terminal,"reward or terminal mismatch")?;
  if let Some(step)=&step{check(!step.truncated&&step.terminated==step.boundary.terminal.is_some(),"terminal/truncation mismatch")?;}
  snapshot(&native(s.reproduction())?,&row["post"])?;
  results.push(step);
 }
 let failed=&case.value["failed_action"];
 if !failed.is_null(){let a=action(&template,&failed["action"])?;let rep=native(s.reproduction())?;let d=public(&rep)?;
  if let Ok(i)=index(d,&a,&failed["candidate_index"]){let token=native(s.decision())?.token;let error=s.step(token,i).err().ok_or("original failed action now succeeds")?;let RunSessionError::Core(error)=error else{return Err("original failure has different layer".into())};check(failed["error"].as_str()==Some(format!("{error:?}").as_str()),"original core fault checkpoint differs")?;}}
 snapshot(&native(s.reproduction())?,&case.value["endpoint"])?;
 let record=native(s.record())?.ok_or("missing final record")?;let parsed=native(RunEpisodeRecord::from_json(&native(record.to_json())?))?;check(parsed==record,"strict typed/JSON round trip differs")?;
 let report=native(replay_run(&parsed,case.config.capacities.clone()))?;
 check(report.accepted_count==record.steps.len()&&report.reported_fault==record.fault,"portable count/fault differs")?;
 let rep=native(s.reproduction())?;check(report.final_map==rep.latest_map,"portable final map differs")?;
 match rep.latest_decision.as_ref().unwrap(){Ok(d)=>check(&report.final_decision==d,"portable final public state differs")?,Err(error)=>check(report.final_decision_error.as_ref()==Some(error),"portable core fault differs")?,}
 check(report.rewards==record.steps.iter().map(|r|r.reward).collect::<Vec<_>>()&&report.terminals==record.steps.iter().map(|r|r.terminal).collect::<Vec<_>>(),"portable transition vectors differ")?;
 let before=native(s.reproduction())?;
 if record.terminal.is_some(){let b=native(s.decision())?;check(s.step(b.token,RunActionIndex(0))==Err(RunSessionError::Terminal),"terminal step accepted")?;}
 else if public(&before).is_ok_and(|d|d.candidates.is_empty()){let b=native(s.decision())?;check(s.step(b.token,RunActionIndex(0))==Err(RunSessionError::UnsupportedBoundary(CUTOFF)),"cutoff step not rejected")?;}
 check(native(s.reproduction())?==before,"invalid endpoint step changed owner")?;native(s.close())?;check(native(s.record())?.unwrap().closed,"close lost record")?;
 Ok((record,results))
}
fn batch(case:&Case,record:&RunEpisodeRecord,expected:&[Option<RunStepResult>],workers:usize)->Result<()>{
 let mut b=native(RunBatch::new_m24(vec![case.config.clone();2],workers))?;let mut bounds=native(b.reset(&[Some(case.seed);2]))?;
 for (i,row) in case.value["steps"].as_array().unwrap().iter().enumerate(){
  let a=action(record,&row["action"])?;let mut indices=Vec::new();
  for slot in 0..2{let rep=native(b.reproduction(slot))?;snapshot(&rep,&row["pre"])?;indices.push(index(public(&rep)?,&a,&row["candidate_index"])?);}
  match b.step(&bounds.iter().map(|b|b.token).collect::<Vec<_>>(),&indices){
   Ok(steps)=>{let want=expected[i].as_ref().ok_or("batch succeeded where scalar faulted")?;for s in &steps{check(s.reward==want.reward&&s.terminated==want.terminated&&s.truncated==want.truncated&&s.boundary.terminal==want.boundary.terminal,"batch step differs from scalar")?;}bounds=steps.into_iter().map(|s|s.boundary).collect();}
   Err(_)=>{check(expected[i].is_none(),"batch faulted where scalar succeeded")?;break}
  }
 }
 for slot in 0..2{check(native(b.record(slot))?.as_ref()==Some(record),"paired final retained record differs")?;snapshot(&native(b.reproduction(slot))?,&case.value["endpoint"])?;}
 let before=[native(b.reproduction(0))?,native(b.reproduction(1))?];check(b.step(&[bounds[0].token],&[RunActionIndex(0)]).is_err(),"partial batch step accepted")?;check([native(b.reproduction(0))?,native(b.reproduction(1))?]==before,"invalid batch call changed peers")?;
 native(b.close())?;Ok(())
}
fn consume_case(case:&Case)->Result<Value>{
 let (record,steps)=scalar(case)?;batch(case,&record,&steps,1)?;batch(case,&record,&steps,2)?;
 Ok(json!({"seed":case.seed,"act1":case.value["scenario"]["act1_boss"],"act2":case.value["scenario"]["act2_boss"],"act3":case.value["scenario"]["act3_boss"],"receipt_sha256":case.receipt_sha256,"accepted":record.steps.len(),"outcome":case.value["outcome"],"error":case.value["error"],"scalar_strictjson_portable_workers1_workers2":"matched","record":serde_json::from_slice::<Value>(&native(record.to_json())?).map_err(|e|e.to_string())?}))
}

/// A coexistence lane replays retained recorded choices only (semantic actions, or
/// recorded candidate indexes where the retained receipt has no semantic field).
#[derive(Clone)]
enum Choice{Semantic(GameAction),Index(i64)}
#[derive(Clone)]
struct Lane{name:&'static str,config:RunSessionConfig,seed:u64,steps:Vec<(Choice,i32)>}
fn session(l:&Lane)->Result<RunSession>{native(match l.name{"m24"=>RunSession::new_m24(l.config.clone()),"m23"=>RunSession::new_m23(l.config.clone()),"m15"=>RunSession::new_m15(l.config.clone()),_=>RunSession::new_m20(l.config.clone())})}
fn batch_of(l:&Lane,workers:usize)->Result<RunBatch>{native(match l.name{"m24"=>RunBatch::new_m24(vec![l.config.clone();2],workers),"m23"=>RunBatch::new_m23(vec![l.config.clone();2],workers),"m15"=>RunBatch::new_m15(vec![l.config.clone();2],workers),_=>RunBatch::new_m20(vec![l.config.clone();2],workers)})}
fn resolve(rep:&RunReproduction,c:&Choice)->Result<RunActionIndex>{match c{Choice::Semantic(a)=>index(public(rep)?,a,&Value::Null),Choice::Index(i)=>{check(*i>=0&&(*i as usize)<public(rep)?.candidates.len(),"recorded index not offered")?;Ok(RunActionIndex(*i))}}}
fn dispatch(s:&mut RunSession,step:&(Choice,i32))->Result<RunStepResult>{let rep=native(s.reproduction())?;let i=resolve(&rep,&step.0)?;let token=native(s.decision())?.token;let actual=native(s.step(token,i))?;check(actual.reward==step.1,"prefix reward mismatch")?;Ok(actual)}
fn receipt_lane(name:&'static str,base:&Path,row:&Value,config:RunSessionConfig,semantic:bool)->Result<Lane>{
 let v=read(&base.join(text(&row["path"])?),text(&row["sha256"])?)?;let seed=v["seed"].as_u64().ok_or("lane seed")?;check(Some(seed)==row["seed"].as_u64(),"lane seed identity")?;
 let mut s=if semantic{Some((session(&Lane{name,config:config.clone(),seed,steps:vec![]})?,()))}else{None};
 let template=if let Some((s,_))=&mut s{native(s.reset(Some(seed)))?;native(s.record())?}else{None};
 let steps=v["steps"].as_array().ok_or("lane steps")?.iter().take(32).map(|r|{let reward=i32::try_from(r["transition"]["reward"].as_i64().ok_or("lane reward")?).map_err(|_|"reward range")?;
  let c=if semantic{Choice::Semantic(action(template.as_ref().unwrap(),&r["action"])?)}else{Choice::Index(r["candidate_index"].as_i64().ok_or("lane candidate index")?)};Ok((c,reward))}).collect::<Result<Vec<_>>>()?;
 Ok(Lane{name,config,seed,steps})
}
/// Nominated static M15 prefix (same input and checks as accepted M22).
fn m15_lane(base:&Path,row:&Value)->Result<Lane>{
 const NOMINATED:&str="b98de6a89b2b07d5d3f37dc582f3b8954baa9dd01ff31b31be9163655a376477";
 check(row["sha256"]==NOMINATED,"M15 input is not root's nominated static prefix")?;let v=read(&base.join(text(&row["path"])?),NOMINATED)?;
 check(v["schema_id"]=="m22-static-m15-coexistence-prefix-input-v1"&&v["prefix_length"]==32,"wrong static prefix schema/length")?;
 let config=config(Character::Silent,RunBossHistory::M15SilentHeart{act1:RunAct1BossTarget::Guardian,act2:RunAct2BossTarget::Champ,act3:RunAct3BossTarget::AwakenedOne});
 let mut s=native(RunSession::new_m15(config.clone()))?;native(s.reset(Some(7)))?;let template=native(s.record())?.unwrap();
 let rows=v["actions"].as_array().ok_or("M15 actions")?;let rewards=v["source_transition_rewards"].as_array().ok_or("M15 rewards")?;
 let steps=(0..32).map(|i|Ok((Choice::Semantic(action(&template,&rows[i])?),i32::try_from(rewards[i].as_i64().ok_or("M15 reward")?).map_err(|_|"reward range")?))).collect::<Result<Vec<_>>>()?;
 Ok(Lane{name:"m15",config,seed:7,steps})
}
fn coexistence(lanes:&[Lane])->Result<Value>{
 let mut live=lanes.iter().map(session).collect::<Result<Vec<_>>>()?;let mut isolated=lanes.iter().map(session).collect::<Result<Vec<_>>>()?;
 for (s,l) in live.iter_mut().zip(lanes){native(s.reset(Some(l.seed)))?;}for (s,l) in isolated.iter_mut().zip(lanes){native(s.reset(Some(l.seed)))?;for step in &l.steps{dispatch(s,step)?;}}
 let mut combat=vec![false;lanes.len()];let n=lanes.iter().map(|l|l.steps.len()).max().unwrap_or(0);
 for i in 0..n{for (k,(s,l)) in live.iter_mut().zip(lanes).enumerate(){if let Some(step)=l.steps.get(i){let r=dispatch(s,step)?;check(!r.truncated,"prefix silently truncated")?;combat[k]|=public(&native(s.reproduction())?).is_ok_and(|d|d.combat.is_some());}}}
 for (s,i) in live.iter().zip(&isolated){check(native(s.reproduction())?==native(i.reproduction())?,"interleaved prefix differs from isolated owner")?;}
 for l in lanes{for workers in [1,2]{let mut b=batch_of(l,workers)?;let mut bounds=native(b.reset(&[Some(l.seed);2]))?;
  for step in &l.steps{let rep=native(b.reproduction(0))?;let i=resolve(&rep,&step.0)?;let out=native(b.step(&bounds.iter().map(|r|r.token).collect::<Vec<_>>(),&[i;2]))?;for o in &out{check(o.reward==step.1,"batch prefix reward mismatch")?;}bounds=out.into_iter().map(|o|o.boundary).collect();}
  let mut single=session(l)?;native(single.reset(Some(l.seed)))?;for step in &l.steps{dispatch(&mut single,step)?;}
  for slot in 0..2{check(native(b.reproduction(slot))?.latest_decision==native(single.reproduction())?.latest_decision,"homogeneous batch differs from single")?;}native(b.close())?;}}
 Ok(json!({"lanes":lanes.iter().zip(&combat).map(|(l,c)|json!({"profile":l.name,"seed":l.seed,"accepted":l.steps.len(),"actual_combat_boundary":c})).collect::<Vec<_>>(),"grade":"bounded interleaved coexistence of retained recorded prefixes plus separate homogeneous workers1/2; not full-history proof; a lane without combat is reported honestly"}))
}
fn release(path:&Path,expected:&str,dry:bool)->Result<Value>{
 let release=read(path,expected)?;check(release["schema_id"]==if dry{"m24-native-receipt-dry-v1"}else{"m24-native-receipt-release-v1"},"foreign release envelope")?;
 let freeze=text(&release["freeze_manifest_sha256"])?;check(freeze.len()==64&&freeze.bytes().all(|b|b.is_ascii_hexdigit()),"invalid freeze hash")?;
 let base=path.parent().unwrap_or(Path::new("."));let rows=release["cases"].as_array().ok_or("missing released cases")?;
 if dry{check(!rows.is_empty(),"dry release needs at least one labelled control")?;}else{check(rows.len()==6,"release must bind all six originals")?;}
 let mut cases=Vec::new();let mut seen=BTreeSet::new();for row in rows{let c=case(base,row,freeze,dry)?;check(seen.insert((c.seed,text(&c.value["scenario"]["act3_boss"])?.to_owned(),text(&row["path"])?.to_owned())),"duplicate case")?;cases.push(c);}
 let mut reports=Vec::new();for c in &cases{reports.push(consume_case(c)?);}
 let co=&release["coexistence"];let m23=&co["m23"];let m20=&co["m20"];
 let mut m24_lane=receipt_lane("m24",base,&json!({"path":rows[0]["path"],"sha256":rows[0]["sha256"],"seed":rows[0]["seed"]}),cases[0].config.clone(),true)?;m24_lane.steps.truncate(32);
 let m23_lane=receipt_lane("m23",base,m23,m23_config(text(&m23["act1_boss"])?,text(&m23["act2_boss"])?)?,true)?;
 let m20_lane=receipt_lane("m20",base,m20,config(Character::Defect,RunBossHistory::M20DefectHeart{act1:act1(text(&m20["act1_boss"])?)?,act2:act2(text(&m20["act2_boss"])?)?,act3:act3(text(&m20["act3_boss"])?)?}),false)?;
 let prefixes=coexistence(&[m24_lane,m23_lane,m15_lane(base,&co["m15"])?,m20_lane])?;
 Ok(json!({"schema_id":if dry{"m24-native-same-choice-dry-result-v1"}else{"m24-native-same-choice-result-v1"},"freeze_manifest_sha256":freeze,"release_sha256":expected,"label":if dry{"pre-freeze consumer validation on labelled non-outcome budget controls; not outcome evidence"}else{"released frozen outcomes"},"cases":reports,"coexistence":prefixes}))
}
fn execute(args:Vec<String>)->Result<Value>{match args.as_slice(){[mode,path,sha] if mode=="released"=>release(&PathBuf::from(path),sha,false),[mode,path,sha] if mode=="released-dry"=>release(&PathBuf::from(path),sha,true),[mode] if mode=="descriptor"=>{let spec=native(m24_run_observation_spec(&RunCapacities::m10_default()))?;Ok(json!({"schema_id":"m24-native-descriptor-export-v1","capacities":"RunCapacities::m10_default()","fingerprint":spec.fingerprint().iter().map(|b|format!("{b:02x}")).collect::<String>(),"descriptor":spec.descriptor()}))},_=>Err("HELD: usage m23_same_choices released TRUSTED_RELEASE_JSON EXPECTED_RELEASE_SHA256".into())}}
fn main(){match execute(std::env::args().skip(1).collect()){Ok(value)=>println!("{value}"),Err(error)=>{eprintln!("{error}");std::process::exit(1)}}}
