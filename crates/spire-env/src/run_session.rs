//! One authoritative native Run, publication tokens, reset and retained fault/record ownership.
#[derive(Clone,Debug,PartialEq,Eq)]
pub enum RunSessionError {
    Input(&'static str),StaleToken,NotReset,Terminal,Closed,
    Core(spire_core::run::RunError),
    Capacity{group:&'static str,required:usize,capacity:usize},
    UnsupportedBoundary(&'static str),DeliveryFault(&'static str),IncompatibleReplay(&'static str),
}
use spire_core::{combat::rng::Partition,content::Character,run::*};
use crate::{RunCapacities,RunObservation};
pub const RUN_RECORD_SCHEMA_ID:&str="spire-gym-run-replay-v1";
pub const RUN_SEED_ADAPTER_ID:&str="spire-gym-run-u64-le-v1";
pub(crate) const RUN_RULES_ID:&str="spire-gym-native-m9-rules-v1";
pub(crate) const RUN_CONTENT_ID:&str="spire-gym-m9-ironclad-a0-captured-v1";
pub(crate) const RUN_REWARD_ID:&str="spire-gym-floor-key-terminal-reward-v1";
static SESSION_IDS:crate::identity::PoolIdAllocator=crate::identity::PoolIdAllocator::new(1);
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum M9BossHistory {AwakenedUnseen,TimeEaterUnseen,DonuDecaUnseen}
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct RunSessionConfig{pub character:Character,pub ascension:u8,pub partition:Partition,pub boss_history:M9BossHistory,pub capacities:RunCapacities,pub record_replay:bool}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct RunDecisionToken(pub(crate) [u64;2]);
impl From<RunDecisionToken> for [u64;2]{fn from(t:RunDecisionToken)->Self{t.0}}
impl From<[u64;2]> for RunDecisionToken{fn from(value:[u64;2])->Self{Self(value)}}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct RunActionIndex(pub i64);
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct RunBoundary{pub observation:RunObservation,pub token:RunDecisionToken,pub terminal:Option<RunTerminal>}
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct RunStepResult{pub boundary:RunBoundary,pub reward:i32,pub terminated:bool,pub truncated:bool}
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct RunRecordedStep{pub action:GameAction,pub reward:i32,pub terminal:Option<RunTerminal>}
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct RunRecordedFault{pub action:Option<GameAction>,pub committed:bool,pub gameplay_recorded:bool,pub error:RunRecordError}
#[derive(Clone,Debug,PartialEq,Eq,serde::Serialize,serde::Deserialize)]
#[serde(rename_all="snake_case")]
pub enum RunRecordErrorKind{Input,StaleToken,NotReset,Terminal,Closed,Core,Capacity,UnsupportedBoundary,DeliveryFault,IncompatibleReplay}
#[derive(Clone,Debug,PartialEq,Eq,serde::Serialize,serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunCapacityFailure{pub group:String,pub required:usize,pub capacity:usize}
#[derive(Clone,Debug,PartialEq,Eq,serde::Serialize,serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunRecordError{pub kind:RunRecordErrorKind,pub message:String,pub capacity:Option<RunCapacityFailure>}
impl RunRecordError{
    pub fn diagnostic_json(&self)->Result<String,RunSessionError>{serde_json::to_string(self).map_err(|_|RunSessionError::DeliveryFault("Run diagnostic export failed"))}
    fn from_error(error:&RunSessionError)->Self{
        use RunRecordErrorKind as K;
        let (kind,message,capacity)=match error{
            RunSessionError::Input(s)=>(K::Input,(*s).to_owned(),None),RunSessionError::StaleToken=>(K::StaleToken,"stale token".into(),None),
            RunSessionError::NotReset=>(K::NotReset,"not reset".into(),None),RunSessionError::Terminal=>(K::Terminal,"terminal".into(),None),RunSessionError::Closed=>(K::Closed,"closed".into(),None),
            RunSessionError::Core(e)=>{let message=match e{RunError::Faulted(f)=>format!("{}: {}",f.checkpoint,match &f.kind{RunFaultKind::UnsupportedBoundary(s)|RunFaultKind::Invariant(s)=>*s,RunFaultKind::Combat(_)=>"combat fault (full owner retained in reproduction)"}),_=>format!("{e:?}")};(K::Core,message,None)},
            RunSessionError::Capacity{group,required,capacity}=>(K::Capacity,"Run capacity exceeded".into(),Some(RunCapacityFailure{group:(*group).into(),required:*required,capacity:*capacity})),
            RunSessionError::UnsupportedBoundary(s)=>(K::UnsupportedBoundary,(*s).into(),None),RunSessionError::DeliveryFault(s)=>(K::DeliveryFault,(*s).into(),None),RunSessionError::IncompatibleReplay(s)=>(K::IncompatibleReplay,(*s).into(),None),
        };Self{kind,message,capacity}
    }
}
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct RunEpisodeRecord{
    pub schema_id:String,pub profile_id:String,pub rules_id:String,pub content_id:String,pub seed_adapter_id:String,pub rng_id:String,pub reward_id:String,
    pub observation_fingerprint:[u8;32],pub config:RunSessionConfig,pub seed:u64,pub ordinal:u64,
    pub steps:Vec<RunRecordedStep>,pub terminal:Option<RunTerminal>,pub fault:Option<RunRecordedFault>,pub closed:bool,
}
/// Explicitly privileged diagnostics, never ordinary observation/info fields.
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct RunReproduction{
    pub config:RunSessionConfig,pub seed:Option<u64>,pub ordinal:u64,
    pub latest_decision:Option<Result<PublicDecision,RunError>>,pub latest_map:Option<spire_core::run::map::PublicMap>,
    pub last_accepted_step:Option<RunRecordedStep>,pub fault:Option<RunSessionError>,pub record:Option<RunEpisodeRecord>,pub closed:bool,
}
pub struct RunSession{
    config:RunSessionConfig,id:u64,serial:u64,run:Option<Run>,published:Option<RunBoundary>,
    actions:Vec<GameAction>,seed:Option<u64>,ordinal:u64,record:Option<RunEpisodeRecord>,
    fault:Option<RunSessionError>,last_step:Option<RunRecordedStep>,closed:bool,
}
pub(crate) fn create_run(config:&RunSessionConfig,seed:u64,ordinal:u64)->Result<Run,RunSessionError>{
    let mut root=[0;32];root[..8].copy_from_slice(&seed.to_le_bytes());
    let mut profile=CreationProfile::experienced();profile.neow_eligible=false;
    profile.seen_bosses[0]=vec!["Hexaghost","SlimeBoss"];profile.seen_bosses[1]=vec!["Champ","Collector"];
    profile.seen_bosses[2]=match config.boss_history{
        M9BossHistory::AwakenedUnseen=>vec!["TimeEater","DonuAndDeca"],
        M9BossHistory::TimeEaterUnseen=>vec!["AwakenedOne","DonuAndDeca"],
        M9BossHistory::DonuDecaUnseen=>vec!["AwakenedOne","TimeEater"],
    };
    Run::new_room_scenario(RunConfig{character:config.character,ascension:config.ascension},profile,RunRng::new(root,config.partition,ordinal),RoomScenario::M9IroncladAct3).map_err(RunSessionError::Core)
}
pub(crate) fn validate_config(config:&RunSessionConfig)->Result<(),RunSessionError>{
    if config.character!=Character::Ironclad||config.ascension!=0{return Err(RunSessionError::Input("M9 requires Ironclad A0"))}
    crate::run_schema::validate(&config.capacities)
}
fn check_boundary(d:&PublicDecision)->Result<(),RunSessionError>{
    if d.room.as_ref().is_some_and(|r|r.kind==RoomViewKind::Act4Ready){return Err(RunSessionError::UnsupportedBoundary("M9 stops before the Act4 door"))}
    if d.terminal.is_none()&&d.candidates.is_empty(){return Err(RunSessionError::UnsupportedBoundary("nonterminal Run has no legal actions"))}Ok(())
}
fn publish(run:&Run,config:&RunSessionConfig,token:RunDecisionToken)->Result<(RunBoundary,Vec<GameAction>),RunSessionError>{
    let d=run.decision().map_err(RunSessionError::Core)?;check_boundary(&d)?;
    let observation=crate::encode_run(&d,run.public_map().as_ref(),&config.capacities)?;
    let mut actions=Vec::new();actions.try_reserve_exact(d.candidates.len()).map_err(|_|RunSessionError::DeliveryFault("Run candidate allocation failed"))?;
    actions.extend(d.candidates.into_iter().map(|c|c.action));
    Ok((RunBoundary{observation,token,terminal:d.terminal},actions))
}
fn new_record(config:&RunSessionConfig,seed:u64,ordinal:u64)->Result<RunEpisodeRecord,RunSessionError>{
    Ok(RunEpisodeRecord{schema_id:RUN_RECORD_SCHEMA_ID.into(),profile_id:"M9IroncladAct3".into(),rules_id:RUN_RULES_ID.into(),content_id:RUN_CONTENT_ID.into(),seed_adapter_id:RUN_SEED_ADAPTER_ID.into(),rng_id:spire_core::combat::rng::RNG_CONTRACT_ID.into(),reward_id:RUN_REWARD_ID.into(),observation_fingerprint:crate::run_observation_spec(&config.capacities)?.fingerprint(),config:config.clone(),seed,ordinal,steps:vec![],terminal:None,fault:None,closed:false})
}
impl RunSession{
    /// Transport-only Gym seeding metadata, never an observation feature.
    pub fn effective_seed(&self)->Option<u64>{self.seed}
    pub fn new(config:RunSessionConfig)->Result<Self,RunSessionError>{
        validate_config(&config)?;
        let id=SESSION_IDS.allocate().map_err(|_|RunSessionError::DeliveryFault("Run session identity exhausted"))?;
        Ok(Self{config,id,serial:0,run:None,published:None,actions:vec![],seed:None,ordinal:0,record:None,fault:None,last_step:None,closed:false})
    }
    fn usable(&self)->Result<(),RunSessionError>{if self.closed{return Err(RunSessionError::Closed)}if let Some(error)=&self.fault{return Err(error.clone())}Ok(())}
    fn freeze(&mut self,error:RunSessionError,cause:RunSessionError,action:Option<GameAction>,committed:bool,gameplay_recorded:bool)->RunSessionError{
        if self.fault.is_none(){self.fault=Some(error);if let Some(record)=&mut self.record{record.fault=Some(RunRecordedFault{action,committed,gameplay_recorded,error:RunRecordError::from_error(&cause)});}}
        self.fault.clone().unwrap()
    }
    pub fn reset(&mut self,seed:Option<u64>)->Result<RunBoundary,RunSessionError>{
        self.usable()?;
        let (seed,ordinal)=match (seed,self.seed){(Some(seed),_)=>(seed,0),(None,Some(seed))=>(seed,self.ordinal.checked_add(1).ok_or(RunSessionError::Input("Run episode ordinal exhausted"))?),(None,None)=>(0,0)};
        let serial=self.serial.checked_add(1).ok_or(RunSessionError::Input("Run decision identity exhausted"))?;
        // Healthy reset intentionally abandons the old episode, never a fault.
        let record=if self.config.record_replay{Some(new_record(&self.config,seed,ordinal)?)}else{None};
        self.record=record;self.seed=Some(seed);self.ordinal=ordinal;self.last_step=None;self.published=None;self.actions.clear();
        let run=match create_run(&self.config,seed,ordinal){Ok(run)=>run,Err(error)=>{self.run=None;return Err(self.freeze(error.clone(),error,None,false,false))}};
        self.run=Some(run);self.serial=serial;
        let (boundary,actions)=match publish(self.run.as_ref().unwrap(),&self.config,RunDecisionToken([self.id,serial])){
            Ok(result)=>result,Err(error)=>return Err(self.freeze(error.clone(),error,None,false,false)),
        };
        let returned=match boundary.owned(){Ok(b)=>b,Err(error)=>return Err(self.freeze(error.clone(),error,None,false,false))};
        self.published=Some(boundary);self.actions=actions;Ok(returned)
    }
    pub fn decision(&self)->Result<RunBoundary,RunSessionError>{self.usable()?;self.published.as_ref().ok_or(RunSessionError::NotReset)?.owned()}
    pub fn step(&mut self,token:RunDecisionToken,index:RunActionIndex)->Result<RunStepResult,RunSessionError>{
        self.usable()?;let published=self.published.as_ref().ok_or(RunSessionError::NotReset)?;
        if token!=published.token{return Err(RunSessionError::StaleToken)}
        if published.terminal.is_some(){return Err(RunSessionError::Terminal)}
        let index=usize::try_from(index.0).ok().filter(|i|*i<self.actions.len()).ok_or(RunSessionError::Input("action index is not legal"))?;
        let action=self.actions[index].clone();let serial=self.serial.checked_add(1).ok_or(RunSessionError::Input("Run decision identity exhausted"))?;
        if let Some(record)=&mut self.record{if record.steps.try_reserve(1).is_err(){return Err(self.freeze(RunSessionError::DeliveryFault("Run recorder allocation failed"),RunSessionError::DeliveryFault("Run recorder allocation failed"),Some(action),false,false))}}
        let transition=match self.run.as_mut().unwrap().step(&action){
            Ok(t)=>t,Err(error)=>{let committed=matches!(error,RunError::Faulted(_));let error=RunSessionError::Core(error);return Err(self.freeze(error.clone(),error,Some(action),committed,false))},
        };
        self.serial=serial;let step=RunRecordedStep{action:action.clone(),reward:transition.reward,terminal:transition.terminal};self.last_step=Some(step.clone());
        if let Some(record)=&mut self.record{record.steps.push(step);record.terminal=transition.terminal;}
        let (boundary,actions)=match publish(self.run.as_ref().unwrap(),&self.config,RunDecisionToken([self.id,serial])){
            Ok(result)=>result,
            Err(error)=>{let returned=if matches!(error,RunSessionError::UnsupportedBoundary(_)){error.clone()}else{RunSessionError::DeliveryFault("post-step Run encoding failed")};return Err(self.freeze(returned,error,Some(action),true,true))},
        };
        let returned=match boundary.owned(){Ok(b)=>b,Err(error)=>return Err(self.freeze(error.clone(),error,Some(action),true,true))};
        self.actions=actions;self.published=Some(boundary);
        Ok(RunStepResult{boundary:returned,reward:transition.reward,terminated:transition.terminal.is_some(),truncated:false})
    }
    pub fn record(&self)->Result<Option<RunEpisodeRecord>,RunSessionError>{
        let Some(record)=&self.record else{return Ok(None)};
        let mut steps=Vec::new();steps.try_reserve_exact(record.steps.len()).map_err(|_|RunSessionError::DeliveryFault("Run record export allocation failed"))?;steps.extend(record.steps.iter().cloned());
        let mut owned=new_record(&record.config,record.seed,record.ordinal)?;owned.steps=steps;owned.terminal=record.terminal;owned.fault=record.fault.clone();owned.closed=record.closed;Ok(Some(owned))
    }
    pub fn reproduction(&self)->Result<RunReproduction,RunSessionError>{
        Ok(RunReproduction{config:self.config.clone(),seed:self.seed,ordinal:self.ordinal,latest_decision:self.run.as_ref().map(Run::decision),latest_map:self.run.as_ref().and_then(Run::public_map),last_accepted_step:self.last_step.clone(),fault:self.fault.clone(),record:self.record()?,closed:self.closed})
    }
    pub fn close(&mut self)->Result<(),RunSessionError>{self.closed=true;if let Some(record)=&mut self.record{record.closed=true;}Ok(())}
    /// Transport acknowledgement failure, not a gameplay callback. Keep the
    /// accepted native action; never rerun it because Python packaging failed.
    pub fn fail_delivery(&mut self,reason:&'static str)->Result<(),RunSessionError>{
        if self.run.is_none(){return Err(RunSessionError::NotReset)}
        let action=self.last_step.as_ref().map(|s|s.action.clone());let committed=action.is_some();
        self.freeze(RunSessionError::DeliveryFault(reason),RunSessionError::DeliveryFault(reason),action,committed,committed);Ok(())
    }
}
impl RunBoundary{
    fn owned(&self)->Result<Self,RunSessionError>{Ok(Self{observation:self.observation.owned()?,token:self.token,terminal:self.terminal})}
}
#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn run_boundary_distinguishes_no_key_empty_deck_and_pre_door_without_fake_terminal(){
        let config=RunSessionConfig{character:Character::Ironclad,ascension:0,partition:Partition::Test,boss_history:M9BossHistory::AwakenedUnseen,capacities:RunCapacities::m9_default(),record_replay:false};
        let mut d=create_run(&config,11,0).unwrap().decision().unwrap();d.candidates.clear();
        for terminal in [RunTerminal::Act3NoKeys,RunTerminal::EmptyDeck,RunTerminal::Death]{d.terminal=Some(terminal);assert_eq!(check_boundary(&d),Ok(()));}
        d.terminal=None;d.room.as_mut().unwrap().kind=RoomViewKind::Act4Ready;assert_eq!(check_boundary(&d),Err(RunSessionError::UnsupportedBoundary("M9 stops before the Act4 door")));
    }
    #[test]
    fn run_session_counter_exhaustion_is_preflight_not_wrap_or_reset(){
        let config=RunSessionConfig{character:Character::Ironclad,ascension:0,partition:Partition::Test,boss_history:M9BossHistory::AwakenedUnseen,capacities:RunCapacities::m9_default(),record_replay:true};
        let mut s=RunSession::new(config).unwrap();let b=s.reset(Some(11)).unwrap();s.serial=u64::MAX;let before=s.record().unwrap();
        assert!(matches!(s.step(b.token,RunActionIndex(0)),Err(RunSessionError::Input(_))));assert_eq!(s.record().unwrap(),before);assert_eq!(s.decision().unwrap(),b);
        s.serial=1;s.ordinal=u64::MAX;assert!(matches!(s.reset(None),Err(RunSessionError::Input(_))));assert_eq!(s.record().unwrap(),before);assert_eq!(s.decision().unwrap(),b);
    }
}
