//! Semantic standalone replay through the existing combat session owner.
use crate::*;
use serde_json::Value;
use spire_core::{combat::{rng::Partition,Outcome},run::PublicDecision};
pub const M21_WATCHER_COMBAT_REPLAY_SCHEMA_ID:&str="spire-gym-m21-watcher-combat-replay-v1";
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct CombatEpisodeRecord{
 pub schema_id:String,pub profile_id:String,pub rules_id:String,pub content_id:String,pub observation_schema_id:String,pub seed_adapter_id:String,pub rng_id:String,pub reward_id:String,
 pub observation_fingerprint:[u8;32],pub capacities:RunCapacities,pub catalog_descriptor:Value,pub catalog_fingerprint:[u8;32],pub entry_id:String,pub selected_entry:Value,
 pub partition:Partition,pub seed:u64,pub ordinal:u64,pub steps:Vec<RunRecordedStep>,pub fault:Option<RunRecordedFault>,pub outcome:Option<Outcome>,pub closed:bool,
}
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct CombatReplayReport{
 pub final_decision:Option<PublicDecision>,pub final_decision_error:Option<RunSessionError>,pub final_boundary:Option<CombatPolicyBoundary>,pub accepted_count:usize,pub rewards:Vec<i32>,pub outcome:Option<Outcome>,pub reported_fault:Option<RunRecordedFault>,
}
use crate::run_replay::{self as codec,exact_fields,text,u,flag,array,incompatible};
use spire_core::{combat::CombatAction,run::{GameAction,RunTerminal}};
use serde_json::json;
pub(crate) fn new_record(catalog:&CombatTrainingCatalog,caps:&RunCapacities,partition:Partition,seed:u64,ordinal:u64,id:&str)->Result<CombatEpisodeRecord,RunSessionError>{Ok(CombatEpisodeRecord{
 schema_id:M21_WATCHER_COMBAT_REPLAY_SCHEMA_ID.into(),profile_id:M21_WATCHER_COMBAT_PROFILE_ID.into(),rules_id:M21_WATCHER_COMBAT_RULES_ID.into(),content_id:M21_WATCHER_COMBAT_CONTENT_ID.into(),observation_schema_id:M21_WATCHER_COMBAT_OBSERVATION_SCHEMA_ID.into(),seed_adapter_id:RUN_SEED_ADAPTER_ID.into(),rng_id:spire_core::combat::rng::RNG_CONTRACT_ID.into(),reward_id:COMBAT_TRAINING_REWARD_ID.into(),
 observation_fingerprint:watcher_combat_observation_spec(caps)?.fingerprint(),capacities:caps.clone(),catalog_descriptor:catalog.descriptor(),catalog_fingerprint:catalog.fingerprint(),entry_id:id.into(),selected_entry:catalog.m21_selected_entry(id)?,partition,seed,ordinal,steps:vec![],fault:None,outcome:None,closed:false,
})}
fn action_value(a:&GameAction)->Result<Value,RunSessionError>{
 let GameAction::Combat(a)=a else{return Err(incompatible())};
 if matches!(a,CombatAction::ChooseCardOffer{..}|CombatAction::SkipCardOffer){return Err(incompatible())}
 let value=codec::combat_action_value(a,true);if value.is_null(){return Err(incompatible())}Ok(value)
}
fn parse_action(v:&Value)->Result<GameAction,RunSessionError>{codec::exact_action_fields_for(v,true)?;let action=GameAction::Combat(codec::parse_combat_action(v,true)?);action_value(&action)?;Ok(action)}
fn outcome_value(o:Option<Outcome>)->Value{o.map(|o|json!(match o{Outcome::Victory=>"victory",Outcome::Death=>"death",Outcome::Escaped=>"escaped"})).unwrap_or(Value::Null)}
fn parse_outcome(v:&Value)->Result<Option<Outcome>,RunSessionError>{if v.is_null(){return Ok(None)}Ok(Some(match text(v)?{"victory"=>Outcome::Victory,"death"=>Outcome::Death,"escaped"=>Outcome::Escaped,_=>return Err(incompatible())}))}
fn fingerprint(v:&Value)->Result<[u8;32],RunSessionError>{array(v)?.iter().map(|n|u8::try_from(u(n)?).map_err(|_|incompatible())).collect::<Result<Vec<_>,_>>()?.try_into().map_err(|_|incompatible())}
fn validate(r:&CombatEpisodeRecord)->Result<CombatTrainingCatalog,RunSessionError>{
 if r.schema_id!=M21_WATCHER_COMBAT_REPLAY_SCHEMA_ID||r.profile_id!=M21_WATCHER_COMBAT_PROFILE_ID||r.rules_id!=M21_WATCHER_COMBAT_RULES_ID||r.content_id!=M21_WATCHER_COMBAT_CONTENT_ID||r.observation_schema_id!=M21_WATCHER_COMBAT_OBSERVATION_SCHEMA_ID||r.seed_adapter_id!=RUN_SEED_ADAPTER_ID||r.rng_id!=spire_core::combat::rng::RNG_CONTRACT_ID||r.reward_id!=COMBAT_TRAINING_REWARD_ID{return Err(RunSessionError::IncompatibleReplay("M21 replay identity mismatch"))}
 let spec=watcher_combat_observation_spec(&r.capacities).map_err(|_|incompatible())?;if spec.fingerprint()!=r.observation_fingerprint{return Err(RunSessionError::IncompatibleReplay("M21 observation fingerprint mismatch"))}
 let catalog=CombatTrainingCatalog::m21_from_descriptor(&r.catalog_descriptor)?;
 if catalog.fingerprint()!=r.catalog_fingerprint{return Err(RunSessionError::IncompatibleReplay("M21 whole catalog fingerprint mismatch"))}
 if catalog.m21_selected_entry(&r.entry_id)?!=r.selected_entry{return Err(RunSessionError::IncompatibleReplay("M21 selected entry receipt mismatch"))}
 codec::validate_recorded_fault(r.fault.as_ref(),&r.steps)?;
 for (i,step) in r.steps.iter().enumerate(){
  action_value(&step.action)?;
  if !(-1..=1).contains(&step.reward)||!matches!(step.terminal,None|Some(RunTerminal::Death))||step.reward==0&&step.terminal.is_some()||step.reward!=0&&i+1!=r.steps.len(){return Err(incompatible())}
 }
 let last=r.steps.last();let expected=match r.outcome{None=>0,Some(Outcome::Victory)=>1,Some(Outcome::Death|Outcome::Escaped)=>-1};
 if last.map_or(0,|s|s.reward)!=expected||last.is_some_and(|s|s.terminal.is_some())!=(r.outcome==Some(Outcome::Death)){return Err(incompatible())}
 if let Some(f)=&r.fault{
  if let Some(a)=&f.action{action_value(a)?;}
  if let Some(cap)=&f.error.capacity{
   let owner=match cap.group.as_str(){"known_draw"=>"combat_cards",name=>name};let caps=codec::capacities(&r.capacities);
   if caps.get(owner).is_none()||cap.required<=cap.capacity||caps[owner].as_u64()!=Some(cap.capacity as u64){return Err(incompatible())}
  }
 }Ok(catalog)
}
impl CombatEpisodeRecord{
 pub fn to_json(&self)->Result<Vec<u8>,RunSessionError>{
  validate(self)?;
  let steps=self.steps.iter().map(|s|Ok(json!({"action":action_value(&s.action)?,"reward":s.reward,"terminal":codec::term(s.terminal)}))).collect::<Result<Vec<_>,RunSessionError>>()?;
  let fault=self.fault.as_ref().map(|f|Ok(json!({"action":f.action.as_ref().map(action_value).transpose()?,"committed":f.committed,"gameplay_recorded":f.gameplay_recorded,"error":f.error}))).transpose()?;
  crate::canonical_json(&json!({"schema_id":self.schema_id,"profile_id":self.profile_id,"rules_id":self.rules_id,"content_id":self.content_id,"observation_schema_id":self.observation_schema_id,"seed_adapter_id":self.seed_adapter_id,"rng_id":self.rng_id,"reward_id":self.reward_id,"observation_fingerprint":self.observation_fingerprint,"capacities":codec::capacities(&self.capacities),"catalog_descriptor":self.catalog_descriptor,"catalog_fingerprint":self.catalog_fingerprint,"entry_id":self.entry_id,"selected_entry":self.selected_entry,"partition":self.partition.as_str(),"seed":self.seed,"ordinal":self.ordinal,"steps":steps,"fault":fault,"outcome":outcome_value(self.outcome),"closed":self.closed})).map_err(|_|RunSessionError::DeliveryFault("combat record export failed"))
 }
 pub fn from_json(bytes:&[u8])->Result<Self,RunSessionError>{
  let v=codec::parse_unique_json(bytes)?;
  exact_fields(&v,&["schema_id","profile_id","rules_id","content_id","observation_schema_id","seed_adapter_id","rng_id","reward_id","observation_fingerprint","capacities","catalog_descriptor","catalog_fingerprint","entry_id","selected_entry","partition","seed","ordinal","steps","fault","outcome","closed"])?;
  exact_fields(&v["capacities"],&["map_nodes","map_edges","permanent_cards","combat_cards","enemies","powers","relics","potions","offers","selection","orbs","actions"])?;
  let mut steps=Vec::new();for s in array(&v["steps"])?{exact_fields(s,&["action","reward","terminal"])?;steps.push(RunRecordedStep{action:parse_action(&s["action"])?,reward:i32::try_from(s["reward"].as_i64().ok_or_else(incompatible)?).map_err(|_|incompatible())?,terminal:codec::parse_term(&s["terminal"])?});}
  codec::strict_recorded_fault_shape(&v["fault"],true)?;
  let fault=if v["fault"].is_null(){None}else{let f=&v["fault"];exact_fields(f,&["action","committed","gameplay_recorded","error"])?;Some(RunRecordedFault{action:if f["action"].is_null(){None}else{Some(parse_action(&f["action"])?)},committed:flag(&f["committed"])?,gameplay_recorded:flag(&f["gameplay_recorded"])?,error:serde_json::from_value(f["error"].clone()).map_err(|_|incompatible())?})};
  let r=Self{schema_id:text(&v["schema_id"])?.into(),profile_id:text(&v["profile_id"])?.into(),rules_id:text(&v["rules_id"])?.into(),content_id:text(&v["content_id"])?.into(),observation_schema_id:text(&v["observation_schema_id"])?.into(),seed_adapter_id:text(&v["seed_adapter_id"])?.into(),rng_id:text(&v["rng_id"])?.into(),reward_id:text(&v["reward_id"])?.into(),observation_fingerprint:fingerprint(&v["observation_fingerprint"])?,capacities:codec::parse_capacities(&v["capacities"])?,catalog_descriptor:v["catalog_descriptor"].clone(),catalog_fingerprint:fingerprint(&v["catalog_fingerprint"])?,entry_id:text(&v["entry_id"])?.into(),selected_entry:v["selected_entry"].clone(),partition:match text(&v["partition"])?{"test"=>Partition::Test,"train"=>Partition::Train,"selection"=>Partition::Selection,"final"=>Partition::Final,_=>return Err(incompatible())},seed:u(&v["seed"])?,ordinal:u(&v["ordinal"])?,steps,fault,outcome:parse_outcome(&v["outcome"])?,closed:flag(&v["closed"])?};
  validate(&r)?;Ok(r)
 }
}
pub fn replay_m21_combat(record:&CombatEpisodeRecord,caps:RunCapacities)->Result<CombatReplayReport,RunSessionError>{
 let catalog=validate(record)?;
 if caps!=record.capacities||watcher_combat_observation_spec(&caps).map_err(|_|incompatible())?.fingerprint()!=record.observation_fingerprint{return Err(RunSessionError::IncompatibleReplay("M21 replay capacities mismatch"))}
 // Every identity, whole catalog and complete explicit selection receipt passed
 // before constructing a session or entering the one shared episode initializer.
 let mut session=CombatTrainingSession::new_m21(catalog,record.partition,caps)?;
 let initial=session.initialize_episode(record.seed,record.ordinal,Some(&record.entry_id));
 let mut rewards=Vec::new();
 if initial.is_err(){
  if !record.steps.is_empty()||record.fault.as_ref()!=session.record()?.as_ref().and_then(|r|r.fault.as_ref()){return Err(RunSessionError::IncompatibleReplay("M21 reset fault mismatch"))}
  return Ok(session.replay_report(rewards,record.fault.clone()))
 }
 for (i,step) in record.steps.iter().enumerate(){
  let index=session.semantic_index(&step.action)?;let token=session.decision()?.token;let result=session.step(token,index);
  let actual=session.record()?.ok_or_else(incompatible)?;
  if actual.steps.last()!=Some(step){return Err(RunSessionError::IncompatibleReplay("M21 replay reward or terminal mismatch"))}
  rewards.push(step.reward);
  if result.is_err()&&(i+1!=record.steps.len()||actual.fault!=record.fault){return Err(RunSessionError::IncompatibleReplay("M21 step fault mismatch"))}
 }
 if let Some(f)=&record.fault{
  if f.committed&&!f.gameplay_recorded{
   let a=f.action.as_ref().ok_or_else(incompatible)?;let index=session.semantic_index(a)?;let token=session.decision()?.token;
   if !matches!(session.step(token,index),Err(RunSessionError::Core(spire_core::run::RunError::Faulted(_))))||session.record()?.as_ref().and_then(|r|r.fault.as_ref())!=Some(f){return Err(RunSessionError::IncompatibleReplay("recorded combat core fault no longer occurs"))}
  }else if matches!(f.error.kind,RunRecordErrorKind::Core|RunRecordErrorKind::Capacity|RunRecordErrorKind::UnsupportedBoundary)&&session.record()?.as_ref().and_then(|r|r.fault.as_ref())!=Some(f){return Err(RunSessionError::IncompatibleReplay("recorded combat fault no longer occurs"))}
  // External delivery-only diagnostics are reported, never injected into core.
 }
 let report=session.replay_report(rewards,record.fault.clone());if report.outcome!=record.outcome{return Err(RunSessionError::IncompatibleReplay("M21 replay outcome mismatch"))}Ok(report)
}
