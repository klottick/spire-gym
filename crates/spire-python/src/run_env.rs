//! Thin Run transport; native ownership spans attached array/tuple packaging.
use std::sync::{Mutex,atomic::{AtomicBool,AtomicI64,Ordering}};
use pyo3::{prelude::*,create_exception,exceptions::{PyValueError,PyRuntimeError,PyNotImplementedError,PyTypeError},types::{PyAny,PyDict,PyInt,PyBool,PyString,PyTuple,PyBytes,PyList}};
use spire_core::{content::Character,combat::rng::Partition,run::{map::PublicMap,PublicDecision,RunTerminal}};
use spire_env::{RunSession,RunSessionConfig,RunSessionError,RunCapacities,RunBoundary,RunObservation,RunActionIndex,RunDecisionToken,M9BossHistory,RunEpisodeRecord,StandardCreation,StandardRunConfig,RUN_OBSERVATION_SCHEMA_ID,RUN_OBSERVATION_M10_SCHEMA_ID,RUN_RECORD_SCHEMA_ID,RUN_RECORD_SCHEMA_ID_M10};
use crate::numeric;

create_exception!(_native,RunInputError,PyValueError);
create_exception!(_native,RunStateError,PyRuntimeError);
create_exception!(_native,RunFaultError,PyRuntimeError);
create_exception!(_native,RunUnsupportedBoundaryError,PyNotImplementedError);
pub(crate) fn integer<T:for<'a,'py> FromPyObject<'a,'py>>(v:&Bound<'_,PyAny>)->PyResult<T>{
    if !v.is_exact_instance_of::<PyInt>(){return Err(RunInputError::new_err("expected built-in integer, not bool"))}
    v.extract().map_err(|_|RunInputError::new_err("integer outside native range"))
}
pub(crate) fn label(v:&Bound<'_,PyAny>)->PyResult<String>{if !v.is_exact_instance_of::<PyString>(){return Err(RunInputError::new_err("expected string"))}v.extract().map_err(Into::into)}
pub(crate) fn boolean(v:&Bound<'_,PyAny>)->PyResult<bool>{if !v.is_exact_instance_of::<PyBool>(){return Err(RunInputError::new_err("expected bool"))}v.extract().map_err(Into::into)}
pub(crate) fn error(e:RunSessionError)->PyErr{match e{
    RunSessionError::Input(s)=>RunInputError::new_err(s),RunSessionError::StaleToken=>RunStateError::new_err("stale Run decision token"),RunSessionError::NotReset=>RunStateError::new_err("Run has not been reset"),RunSessionError::Terminal=>RunStateError::new_err("Run is terminal"),RunSessionError::Closed=>RunStateError::new_err("Run is closed"),
    RunSessionError::UnsupportedBoundary(s)=>RunUnsupportedBoundaryError::new_err(s),RunSessionError::DeliveryFault(s)=>RunFaultError::new_err(s),RunSessionError::Capacity{group,required,capacity}=>RunFaultError::new_err(format!("Run capacity {group}: {required} > {capacity}")),RunSessionError::IncompatibleReplay(s)=>RunInputError::new_err(s),RunSessionError::Core(_)=>RunFaultError::new_err("native Run fault; inspect privileged reproduction"),
}}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub(crate) enum RunProfile{M9IroncladAct3,M10IroncladHeart,M10IroncladPolicy,Standard}
impl RunProfile{
    fn parse(value:&str)->PyResult<Self>{match value{
        "m9-ironclad-act3"=>Ok(Self::M9IroncladAct3),
        "m10-ironclad-heart"=>Ok(Self::M10IroncladHeart),
        "m10-ironclad-policy"=>Ok(Self::M10IroncladPolicy),
        "standard"=>Ok(Self::Standard),
        _=>Err(RunInputError::new_err("unsupported Run profile")),
    }}
    fn schema_id(self)->&'static str{match self{
        Self::M9IroncladAct3=>RUN_OBSERVATION_SCHEMA_ID,
        Self::M10IroncladHeart=>RUN_OBSERVATION_M10_SCHEMA_ID,Self::M10IroncladPolicy=>spire_env::RUN_OBSERVATION_POLICY_SCHEMA_ID,Self::Standard=>spire_env::RUN_OBSERVATION_STANDARD_SCHEMA_ID,
    }}
    fn new_session(self,config:RunSessionConfig)->Result<RunSession,RunSessionError>{match self{
        Self::M9IroncladAct3=>RunSession::new(config),
        Self::M10IroncladHeart=>RunSession::new_m10(config),Self::M10IroncladPolicy=>RunSession::new_m10_policy(config),
        Self::Standard=>{let M9BossHistory::Standard(creation)=config.boss_history else{return Err(RunSessionError::Input("Standard requires creation and rejects boss_history"))};RunSession::new_standard(StandardRunConfig{character:config.character,ascension:config.ascension,partition:config.partition,creation,capacities:config.capacities,record_replay:config.record_replay})},
    }}
    fn from_record(record:&RunEpisodeRecord)->Result<Self,RunSessionError>{match (record.schema_id.as_str(),record.profile_id.as_str()){
        (RUN_RECORD_SCHEMA_ID,"M9IroncladAct3")=>Ok(Self::M9IroncladAct3),
        (RUN_RECORD_SCHEMA_ID_M10,"M10IroncladHeart")=>Ok(Self::M10IroncladHeart),
        (spire_env::RUN_RECORD_SCHEMA_ID_POLICY,"M10IroncladPolicy")=>Ok(Self::M10IroncladPolicy),
        (spire_env::RUN_RECORD_SCHEMA_ID_STANDARD,"Standard")=>Ok(Self::Standard),
        _=>Err(RunSessionError::IncompatibleReplay("Run identity mismatch")),
    }}
    fn encode(self,decision:&PublicDecision,map:Option<&PublicMap>,caps:&RunCapacities,ascension:u8,selection:Option<spire_core::run::RoomSelectionOperation>)->Result<RunObservation,RunSessionError>{match self{
        Self::M9IroncladAct3=>spire_env::encode_run(decision,map,caps),
        Self::M10IroncladHeart=>spire_env::encode_run_m10(decision,map,caps),Self::M10IroncladPolicy=>spire_env::encode_run_policy(decision,map,caps),Self::Standard=>spire_env::encode_run_standard_decision(decision,map,caps,ascension,selection),
    }}
}
pub(crate) fn capacities(v:Option<&Bound<'_,PyAny>>,profile:RunProfile)->PyResult<RunCapacities>{
    let mut c=match profile{RunProfile::M9IroncladAct3=>RunCapacities::m9_default(),RunProfile::M10IroncladHeart|RunProfile::M10IroncladPolicy=>RunCapacities::m10_default(),RunProfile::Standard=>RunCapacities::standard_default()};let Some(v)=v.filter(|v|!v.is_none())else{return Ok(c)};
    let d=v.cast::<PyDict>().map_err(|_|RunInputError::new_err("capacities must be a dictionary"))?;
    for (key,value) in d.iter(){let target=match label(&key)?.as_str(){"map_nodes"=>&mut c.map_nodes,"map_edges"=>&mut c.map_edges,"permanent_cards"=>&mut c.permanent_cards,"combat_cards"=>&mut c.combat_cards,"enemies"=>&mut c.enemies,"powers"=>&mut c.powers,"relics"=>&mut c.relics,"potions"=>&mut c.potions,"offers"=>&mut c.offers,"selection"=>&mut c.selection,"orbs"=>&mut c.orbs,"actions"=>&mut c.actions,_=>return Err(RunInputError::new_err("unknown capacity"))};*target=integer(&value)?;}
    Ok(c)
}
fn creation(py:Python<'_>,value:Option<&Bound<'_,PyAny>>)->PyResult<StandardCreation>{
    let Some(value)=value.filter(|v|!v.is_none()) else{return Ok(StandardCreation::experienced())};
    let dict=value.cast::<PyDict>().map_err(|_|RunInputError::new_err("creation must be a dictionary"))?;
    for (key,_) in dict.iter(){if !matches!(label(&key)?.as_str(),"neow"|"seen_bosses"|"note"){return Err(RunInputError::new_err("unknown Standard creation key"))}}
    let json_module=py.import("json")?;
    let normalized=json_module.call_method1("loads",(StandardCreation::experienced().to_json_string(),))?;
    let normalized=normalized.cast::<PyDict>()?;
    if let Some(v)=dict.get_item("neow")?{let neow=label(&v)?;if neow!="full"&&neow!="short"{return Err(RunInputError::new_err("invalid Standard Neow choice"))}normalized.set_item("neow",neow)?;}
    if let Some(v)=dict.get_item("seen_bosses")?{
        let acts=v.cast::<PyList>().map_err(|_|RunInputError::new_err("seen_bosses must be a list of three boss lists"))?;
        if acts.len()!=3{return Err(RunInputError::new_err("seen_bosses must contain three acts"))}
        for act in acts.iter(){let bosses=act.cast::<PyList>().map_err(|_|RunInputError::new_err("each seen_bosses act must be a list"))?;for boss in bosses.iter(){label(&boss)?;}}
        normalized.set_item("seen_bosses",v)?;
    }
    if let Some(v)=dict.get_item("note")?{
        let note=v.cast::<PyDict>().map_err(|_|RunInputError::new_err("note must be a dictionary"))?;
        for (key,_) in note.iter(){if !matches!(label(&key)?.as_str(),"card"|"upgrades"){return Err(RunInputError::new_err("unknown Standard note key"))}}
        let card=note.get_item("card")?.ok_or_else(||RunInputError::new_err("note requires card and upgrades"))?;
        let upgrades=note.get_item("upgrades")?.ok_or_else(||RunInputError::new_err("note requires card and upgrades"))?;
        let stored=PyDict::new(py);stored.set_item("card",label(&card)?)?;stored.set_item("upgrades",integer::<u32>(&upgrades)?)?;normalized.set_item("stored_note",stored)?;
    }
    let text:String=json_module.call_method1("dumps",(normalized,))?.extract()?;
    StandardCreation::from_json_str(&text).map_err(error)
}
fn terminal(t:Option<RunTerminal>)->Option<&'static str>{t.map(|t|match t{RunTerminal::Death=>"death",RunTerminal::EmptyDeck=>"empty_deck",RunTerminal::Act3NoKeys=>"act3_no_keys",RunTerminal::HeartVictory=>"heart_victory"})}
pub(crate) fn observation<'py>(py:Python<'py>,o:RunObservation)->PyResult<Bound<'py,PyDict>>{
    let d=PyDict::new(py);
    for g in o.groups{
        d.set_item(g.key,numeric::array(py,g.values,&g.shape)?)?;
        d.set_item(format!("{}_known",g.key),numeric::array(py,g.known,&g.shape)?)?;
        if let Some(mask)=g.occupied{let count=mask.len();d.set_item(format!("{}_occupied",g.key),numeric::array(py,mask,&[count])?)?;}
    }
    let count=o.action_mask.len();d.set_item("action_mask",numeric::array(py,o.action_mask,&[count])?)?;Ok(d)
}
pub(crate) fn specification<'py>(py:Python<'py>,spec:spire_env::RunObservationSpec)->PyResult<Bound<'py,PyAny>>{
    let d=numeric::descriptor(py,spec.descriptor().to_string())?;
    d.set_item("fingerprint",spec.fingerprint().iter().map(|v|format!("{v:02x}")).collect::<String>())?;Ok(d)
}
pub(crate) fn policy_boundary<'py>(py:Python<'py>,b:spire_env::CombatPolicyBoundary)->PyResult<Bound<'py,PyDict>>{
    let d=PyDict::new(py);d.set_item("obs",observation(py,b.observation)?)?;
    let t:[u64;2]=b.token.into();d.set_item("token",(t[0],t[1]))?;
    d.set_item("info",numeric::descriptor(py,b.info.to_string())?)?;Ok(d)
}
fn boundary<'py>(py:Python<'py>,b:RunBoundary,profile:RunProfile)->PyResult<Bound<'py,PyDict>>{
    let d=PyDict::new(py);d.set_item("obs",observation(py,b.observation)?)?;let token:[u64;2]=b.token.into();d.set_item("token",(token[0],token[1]))?;
    let info=PyDict::new(py);info.set_item("schema_id",profile.schema_id())?;info.set_item("terminal_reason",terminal(b.terminal))?;d.set_item("info",info)?;Ok(d)
}
fn reward_events<'py>(py:Python<'py>,items:&[spire_core::run::RewardItem])->PyResult<Bound<'py,PyList>>{
    use spire_core::run::RewardItemKind as K;
    let list=PyList::empty(py);
    for item in items{let row=PyDict::new(py);let kind=match item.kind{K::NewFloor{act,floor}=>{row.set_item("act",act)?;row.set_item("floor",floor)?;"new_floor"},K::NewAct4Floor{floor}=>{row.set_item("floor",floor)?;"new_act4_floor"},K::FirstKey(key)=>{row.set_item("key",match key{spire_core::run::Key::Ruby=>"ruby",spire_core::run::Key::Sapphire=>"sapphire",spire_core::run::Key::Emerald=>"emerald"})?;"first_key"},K::HeartVictory=>"heart_victory",K::Death=>"death",K::EmptyDeck=>"empty_deck"};row.set_item("kind",kind)?;row.set_item("amount",item.amount)?;list.append(row)?;}
    Ok(list)
}
pub(crate) fn token(v:&Bound<'_,PyAny>)->PyResult<RunDecisionToken>{
    let t=v.cast::<PyTuple>().map_err(|_|RunInputError::new_err("token must be a two-integer tuple"))?;
    if t.len()!=2{return Err(RunInputError::new_err("token must have length two"))}Ok([integer(&t.get_item(0)?)?,integer(&t.get_item(1)?)?].into())
}
/// M34c privileged exact checkpoint: opaque, in-memory only (pickling/copying raise), redacted repr, identity equality.
#[pyclass(frozen,module="spire_lab._native",name="ExactCheckpoint")]
pub(crate) struct PyExactCheckpoint{inner:spire_env::ExactCheckpoint,profile:RunProfile,episode_return:i64}
#[pymethods]
impl PyExactCheckpoint{
    fn __repr__(&self)->&'static str{"<ExactCheckpoint privileged exact state (redacted)>"}
    fn __reduce__(&self)->PyResult<()>{Err(PyTypeError::new_err("ExactCheckpoint is in-memory only and cannot be pickled or copied; record() is the portable form"))}
}
#[pyclass(frozen,module="spire_lab._native")]
pub(crate) struct NativeRunSession{session:Mutex<RunSession>,profile:RunProfile,busy:AtomicBool,episode_return:AtomicI64}
struct Lease<'a>(&'a NativeRunSession);
impl Drop for Lease<'_>{fn drop(&mut self){self.0.busy.store(false,Ordering::Release);}}
impl NativeRunSession{
    fn wrap(session:RunSession,profile:RunProfile,episode_return:i64)->Self{Self{session:Mutex::new(session),profile,busy:AtomicBool::new(false),episode_return:AtomicI64::new(episode_return)}}
    /// Current boundary payload (reset-shaped: effective seed and, on Standard, the creation profile).
    fn current<'py>(&self,py:Python<'py>,b:RunBoundary,effective:Option<u64>,creation:Option<StandardCreation>)->PyResult<Bound<'py,PyDict>>{
        let raw=boundary(py,b,self.profile)?;raw.set_item("effective_seed",effective)?;
        if let Some(c)=creation{raw.get_item("info")?.unwrap().set_item("creation",numeric::descriptor(py,c.to_json().to_string())?)?;}
        if self.profile==RunProfile::Standard{raw.get_item("info")?.unwrap().set_item("episode_return",self.episode_return.load(Ordering::Acquire))?;}
        Ok(raw)
    }
    fn begin(&self)->PyResult<Lease<'_>>{self.busy.compare_exchange(false,true,Ordering::Acquire,Ordering::Relaxed).map_err(|_|RunStateError::new_err("Run operation is busy"))?;Ok(Lease(self))}
    fn deliver<T>(&self,result:PyResult<T>)->PyResult<T>{if result.is_err(){let _=self.session.lock().unwrap_or_else(|p|p.into_inner()).fail_delivery("Run transport packaging failed");}result}
    fn reset_transport<'py,T>(&self,py:Python<'py>,seed:Option<&Bound<'py,PyAny>>,package:impl FnOnce(Bound<'py,PyDict>)->PyResult<T>)->PyResult<T>{
        let _lease=self.begin()?;let seed=seed.filter(|v|!v.is_none()).map(integer::<u64>).transpose()?;
        let (b,effective,creation)=py.detach(||{let mut s=self.session.lock().unwrap_or_else(|p|p.into_inner());let b=s.reset(seed)?;Ok::<_,RunSessionError>((b,s.effective_seed().expect("successful reset owns seed"),s.standard_creation()))}).map_err(error)?;
        self.episode_return.store(0,Ordering::Release);
        let result=(||{let raw=boundary(py,b,self.profile)?;raw.set_item("effective_seed",effective)?;if let Some(c)=creation{raw.get_item("info")?.unwrap().set_item("creation",numeric::descriptor(py,c.to_json().to_string())?)?;}package(raw)})();self.deliver(result)
    }
    fn step_transport<'py,T>(&self,py:Python<'py>,index:&Bound<'py,PyAny>,t:&Bound<'py,PyAny>,package:impl FnOnce(Bound<'py,PyDict>)->PyResult<T>)->PyResult<T>{
        let _lease=self.begin()?;let index=RunActionIndex(integer(index)?);let t=token(t)?;
        let step=py.detach(||self.session.lock().unwrap_or_else(|p|p.into_inner()).step(t,index)).map_err(error)?;
        let running=self.episode_return.fetch_add(i64::from(step.reward),Ordering::AcqRel)+i64::from(step.reward);
        let heart_victory=step.boundary.terminal==Some(RunTerminal::HeartVictory);
        let result=(||{let raw=boundary(py,step.boundary,self.profile)?;raw.set_item("reward",step.reward)?;raw.set_item("terminated",step.terminated)?;raw.set_item("truncated",step.truncated)?;if self.profile==RunProfile::Standard{let info=raw.get_item("info")?.unwrap();info.set_item("reward_events",reward_events(py,&step.reward_events)?)?;info.set_item("episode_return",running)?;info.set_item("heart_victory",heart_victory)?;}package(raw)})();self.deliver(result)
    }
}
#[pymethods]
impl NativeRunSession{
    #[new]
    #[pyo3(signature=(character,ascension,*,profile="m9-ironclad-act3".to_owned(),partition="train".to_owned(),boss_history=None,creation=None,capacities=None,record_replay=false))]
    fn new(py:Python<'_>,character:&Bound<'_,PyAny>,ascension:&Bound<'_,PyAny>,#[pyo3(from_py_with=label)]profile:String,#[pyo3(from_py_with=label)]partition:String,boss_history:Option<&Bound<'_,PyAny>>,creation:Option<&Bound<'_,PyAny>>,capacities:Option<&Bound<'_,PyAny>>,#[pyo3(from_py_with=boolean)]record_replay:bool)->PyResult<Self>{
        let profile=RunProfile::parse(&profile)?;
        let character=match (profile,label(character)?.as_str()){
            (RunProfile::Standard,"ironclad")=>Character::Ironclad,(RunProfile::Standard,"silent")=>Character::Silent,(RunProfile::Standard,"defect")=>Character::Defect,(RunProfile::Standard,"watcher")=>Character::Watcher,
            (RunProfile::M9IroncladAct3|RunProfile::M10IroncladHeart|RunProfile::M10IroncladPolicy,"ironclad")=>Character::Ironclad,
            (RunProfile::Standard,_)=>return Err(RunInputError::new_err("Standard character must be ironclad, silent, defect, or watcher")),
            (RunProfile::M9IroncladAct3,_)=>return Err(RunInputError::new_err("M9 supports ironclad only")),
            (RunProfile::M10IroncladHeart|RunProfile::M10IroncladPolicy,_)=>return Err(RunInputError::new_err("M10 supports ironclad only")),
        };
        let partition=match partition.as_str(){"train"=>Partition::Train,"selection"=>Partition::Selection,"final"=>Partition::Final,"test"=>Partition::Test,_=>return Err(RunInputError::new_err("unknown partition"))};
        let creation=if profile==RunProfile::Standard{self::creation(py,creation)?}else{if creation.is_some_and(|v|!v.is_none()){return Err(RunInputError::new_err("captured profiles do not accept creation"))}StandardCreation::experienced()};
        let boss_history=if profile==RunProfile::Standard{
            if boss_history.is_some_and(|v|!v.is_none()){return Err(RunInputError::new_err("Standard does not accept boss_history"))}
            M9BossHistory::Standard(creation)
        }else{
            let value=match boss_history.filter(|v|!v.is_none()){Some(v)=>label(v)?,None=>"awakened-unseen".to_owned()};
            match value.as_str(){"awakened-unseen"=>M9BossHistory::AwakenedUnseen,"time-eater-unseen"=>M9BossHistory::TimeEaterUnseen,"donu-deca-unseen"=>M9BossHistory::DonuDecaUnseen,_=>return Err(RunInputError::new_err("unknown boss history"))}
        };
        let capacities=crate::run_env::capacities(capacities,profile)?;let cfg=RunSessionConfig{character,ascension:integer(ascension)?,partition,boss_history,capacities,record_replay};
        let session=py.detach(||profile.new_session(cfg)).map_err(error)?;Ok(Self::wrap(session,profile,0))
    }
    /// Owned current boundary (after fork or from_record the caller reads its first token here).
    fn decision<'py>(&self,py:Python<'py>)->PyResult<Bound<'py,PyDict>>{
        let _lease=self.begin()?;
        let (b,effective,creation)=py.detach(||{let s=self.session.lock().unwrap_or_else(|p|p.into_inner());Ok::<_,RunSessionError>((s.decision()?,s.effective_seed(),s.standard_creation()))}).map_err(error)?;
        self.deliver(self.current(py,b,effective,creation))
    }
    /// M34c privileged exact checkpoint (oracle: restoring replays the same hidden future).
    fn exact_checkpoint(&self,py:Python<'_>)->PyResult<PyExactCheckpoint>{
        let _lease=self.begin()?;let inner=py.detach(||self.session.lock().unwrap_or_else(|p|p.into_inner()).exact_checkpoint()).map_err(error)?;
        Ok(PyExactCheckpoint{inner,profile:self.profile,episode_return:self.episode_return.load(Ordering::Acquire)})
    }
    /// Abandon the current episode and continue from `checkpoint` (same profile and configuration); earlier tokens are stale.
    fn exact_restore<'py>(&self,py:Python<'py>,checkpoint:&Bound<'py,PyExactCheckpoint>)->PyResult<Bound<'py,PyDict>>{
        let _lease=self.begin()?;let cp=checkpoint.get();
        if cp.profile!=self.profile{return Err(RunInputError::new_err("exact checkpoint belongs to another session configuration"))}
        let (b,effective,creation)=py.detach(||{let mut s=self.session.lock().unwrap_or_else(|p|p.into_inner());let b=s.exact_restore(&cp.inner)?;Ok::<_,RunSessionError>((b,s.effective_seed(),s.standard_creation()))}).map_err(error)?;
        self.episode_return.store(cp.episode_return,Ordering::Release);
        self.deliver(self.current(py,b,effective,creation))
    }
    /// A new independent session at `checkpoint` (allowed after the source froze or closed; never inherits its fault).
    #[staticmethod]
    fn exact_fork(py:Python<'_>,checkpoint:&Bound<'_,PyExactCheckpoint>)->PyResult<Self>{
        let cp=checkpoint.get();let session=py.detach(||RunSession::exact_fork(&cp.inner)).map_err(error)?;
        Ok(Self::wrap(session,cp.profile,cp.episode_return))
    }
    /// Replay to a decision: a live session after the first `steps` accepted actions of `record` (checked as replay()).
    #[staticmethod]
    #[pyo3(signature=(record,steps,capacities=None))]
    fn from_record(py:Python<'_>,record:&Bound<'_,PyBytes>,steps:&Bound<'_,PyAny>,capacities:Option<&Bound<'_,PyAny>>)->PyResult<Self>{
        let steps:usize=integer(steps)?;let bytes=record.as_bytes().to_vec();
        let parsed=RunEpisodeRecord::from_json(&bytes).map_err(error)?;let profile=RunProfile::from_record(&parsed).map_err(error)?;
        let caps=crate::run_env::capacities(capacities,profile)?;
        let episode_return=parsed.steps.iter().take(steps).map(|s|i64::from(s.reward)).sum();
        let session=py.detach(move||RunSession::from_record(&parsed,steps,caps)).map_err(error)?;
        Ok(Self::wrap(session,profile,episode_return))
    }
    pub(crate) fn observation_spec<'py>(&self,py:Python<'py>)->PyResult<Bound<'py,PyAny>>{
        specification(py,self.session.lock().unwrap_or_else(|p|p.into_inner()).observation_spec().map_err(error)?)
    }
    fn environment_spec<'py>(&self,py:Python<'py>)->PyResult<Bound<'py,PyAny>>{
        let _lease=self.begin()?;let d=self.session.lock().unwrap_or_else(|p|p.into_inner()).environment_spec().map_err(error)?;
        numeric::descriptor(py,d.to_string())
    }
    fn combat_observation_spec<'py>(&self,py:Python<'py>)->PyResult<Bound<'py,PyAny>>{
        let _lease=self.begin()?;specification(py,self.session.lock().unwrap_or_else(|p|p.into_inner()).combat_observation_spec().map_err(error)?)
    }
    fn combat_view<'py>(&self,py:Python<'py>)->PyResult<Bound<'py,PyDict>>{
        let _lease=self.begin()?;let b=py.detach(||self.session.lock().unwrap_or_else(|p|p.into_inner()).combat_view()).map_err(error)?;
        self.deliver(policy_boundary(py,b))
    }
    #[pyo3(signature=(seed=None))]
    fn reset<'py>(&self,py:Python<'py>,seed:Option<&Bound<'py,PyAny>>)->PyResult<Bound<'py,PyDict>>{self.reset_transport(py,seed,Ok)}
    fn step<'py>(&self,py:Python<'py>,index:&Bound<'py,PyAny>,token:&Bound<'py,PyAny>)->PyResult<Bound<'py,PyDict>>{self.step_transport(py,index,token,Ok)}
    #[pyo3(signature=(package,*,seed=None))]
    fn _reset_deliver<'py>(&self,py:Python<'py>,package:&Bound<'py,PyAny>,seed:Option<&Bound<'py,PyAny>>)->PyResult<Bound<'py,PyAny>>{
        if !package.is_callable(){return Err(RunInputError::new_err("package must be callable"))}self.reset_transport(py,seed,|raw|package.call1((raw,)))
    }
    fn _step_deliver<'py>(&self,py:Python<'py>,index:&Bound<'py,PyAny>,token:&Bound<'py,PyAny>,package:&Bound<'py,PyAny>)->PyResult<Bound<'py,PyAny>>{
        if !package.is_callable(){return Err(RunInputError::new_err("package must be callable"))}self.step_transport(py,index,token,|raw|package.call1((raw,)))
    }
    fn record<'py>(&self,py:Python<'py>)->PyResult<Bound<'py,PyAny>>{
        let _lease=self.begin()?;let bytes=py.detach(||self.session.lock().unwrap_or_else(|p|p.into_inner()).record()?.map(|r|r.to_json()).transpose()).map_err(error)?;
        Ok(bytes.map_or_else(||py.None().into_bound(py),|v|PyBytes::new(py,&v).into_any()))
    }
    fn reproduction<'py>(&self,py:Python<'py>)->PyResult<Bound<'py,PyDict>>{
        let _lease=self.begin()?;let r=py.detach(||self.session.lock().unwrap_or_else(|p|p.into_inner()).reproduction()).map_err(error)?;
        let d=PyDict::new(py);d.set_item("seed",r.seed)?;d.set_item("ordinal",r.ordinal)?;d.set_item("closed",r.closed)?;d.set_item("fault",r.fault.map(|f|format!("{f:?}")))?;d.set_item("last_accepted_action",r.last_accepted_step.map(|s|format!("{:?}",s.action)))?;
        if let Some(record)=r.record{d.set_item("record",PyBytes::new(py,&record.to_json().map_err(error)?))?;}else{d.set_item("record",py.None())?;}Ok(d)
    }
    fn close(&self,py:Python<'_>)->PyResult<()>{let _lease=self.begin()?;py.detach(||self.session.lock().unwrap_or_else(|p|p.into_inner()).close()).map_err(error)}
    #[staticmethod]
    #[pyo3(signature=(record,capacities=None))]
    fn replay<'py>(py:Python<'py>,record:&Bound<'py,PyBytes>,capacities:Option<&Bound<'py,PyAny>>)->PyResult<Bound<'py,PyDict>>{
        let bytes=record.as_bytes().to_vec();let parsed=RunEpisodeRecord::from_json(&bytes).map_err(error)?;let profile=RunProfile::from_record(&parsed).map_err(error)?;let ascension=parsed.config.ascension;
        let caps=crate::run_env::capacities(capacities,profile)?;let export_caps=caps.clone();let report=py.detach(move||spire_env::replay_run(&parsed,caps)).map_err(error)?;
        let d=PyDict::new(py);let o=profile.encode(&report.final_decision,report.final_map.as_ref(),&export_caps,ascension,report.final_room_selection).map_err(error)?;d.set_item("obs",observation(py,o)?)?;d.set_item("terminal_reason",terminal(report.final_decision.terminal))?;d.set_item("rewards",report.rewards)?;d.set_item("terminals",report.terminals.into_iter().map(terminal).collect::<Vec<_>>())?;d.set_item("accepted_count",report.accepted_count)?;
        if let Some(f)=report.reported_fault{d.set_item("reported_fault",numeric::descriptor(py,f.error.diagnostic_json().map_err(error)?)?)?;}else{d.set_item("reported_fault",py.None())?;}
        d.set_item("final_decision_error",report.final_decision_error.map(|_|"native core fault; final obs is the last readable boundary"))?;Ok(d)
    }
}
pub(crate) fn register(m:&Bound<'_,PyModule>)->PyResult<()>{let py=m.py();m.add("RunInputError",py.get_type::<RunInputError>())?;m.add("RunStateError",py.get_type::<RunStateError>())?;m.add("RunFaultError",py.get_type::<RunFaultError>())?;m.add("RunUnsupportedBoundaryError",py.get_type::<RunUnsupportedBoundaryError>())?;Ok(())}
