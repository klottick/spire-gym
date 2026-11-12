//! Thin Run transport; native ownership spans attached array/tuple packaging.
use std::{collections::HashMap,sync::{Mutex,atomic::{AtomicBool,AtomicI64,Ordering}}};
use pyo3::{prelude::*,create_exception,exceptions::{PyValueError,PyRuntimeError,PyNotImplementedError,PyTypeError},types::{PyAny,PyDict,PyInt,PyBool,PyString,PyTuple,PyBytes,PyList}};
use spire_core::{content::Character,combat::rng::Partition,run::{map::PublicMap,PublicDecision,RunTerminal}};
use spire_env::{RunSession,RunSessionConfig,RunSessionError,RunCapacities,RunBoundary,RunObservation,RunActionIndex,RunDecisionToken,M9BossHistory,RunEpisodeRecord,StandardCreation,StandardRunConfig,RunObservationLayout,RUN_OBSERVATION_SCHEMA_ID,RUN_OBSERVATION_M10_SCHEMA_ID,RUN_RECORD_SCHEMA_ID,RUN_RECORD_SCHEMA_ID_M10,PublicRoot,PublicSearch,SearchConfig,SearchLimits,BranchId,BranchView,BranchStepRequest};
use crate::{numeric,stacking};

create_exception!(_native,RunInputError,PyValueError);
create_exception!(_native,RunStateError,PyRuntimeError);
create_exception!(_native,RunFaultError,PyRuntimeError);
create_exception!(_native,RunUnsupportedBoundaryError,PyNotImplementedError);
create_exception!(_native,RunBudgetError,RunStateError);
pub(crate) fn integer<T:for<'a,'py> FromPyObject<'a,'py>>(v:&Bound<'_,PyAny>)->PyResult<T>{
    if !v.is_exact_instance_of::<PyInt>(){return Err(RunInputError::new_err("expected built-in integer, not bool"))}
    v.extract().map_err(|_|RunInputError::new_err("integer outside native range"))
}
pub(crate) fn label(v:&Bound<'_,PyAny>)->PyResult<String>{if !v.is_exact_instance_of::<PyString>(){return Err(RunInputError::new_err("expected string"))}v.extract().map_err(Into::into)}
pub(crate) fn boolean(v:&Bound<'_,PyAny>)->PyResult<bool>{if !v.is_exact_instance_of::<PyBool>(){return Err(RunInputError::new_err("expected bool"))}v.extract().map_err(Into::into)}
pub(crate) fn error(e:RunSessionError)->PyErr{match e{
    RunSessionError::Input(s)=>RunInputError::new_err(s),RunSessionError::StaleToken=>RunStateError::new_err("stale Run decision token"),RunSessionError::NotReset=>RunStateError::new_err("Run has not been reset"),RunSessionError::Terminal=>RunStateError::new_err("Run is terminal"),RunSessionError::Closed=>RunStateError::new_err("Run is closed"),
    RunSessionError::UnsupportedBoundary(s)=>RunUnsupportedBoundaryError::new_err(s),RunSessionError::DeliveryFault(s)=>RunFaultError::new_err(s),RunSessionError::Capacity{group,required,capacity}=>RunFaultError::new_err(format!("Run capacity {group}: {required} > {capacity}")),RunSessionError::IncompatibleReplay(s)=>RunInputError::new_err(s),RunSessionError::Core(_)=>RunFaultError::new_err("native Run fault; inspect privileged reproduction"),
    RunSessionError::Budget(s)=>RunBudgetError::new_err(s),RunSessionError::BranchFaulted=>RunStateError::new_err("search branch is faulted"),
}}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub(crate) enum RunProfile{M9IroncladAct3,M10IroncladHeart,M10IroncladPolicy,Standard}
impl RunProfile{
    pub(crate) fn parse(value:&str)->PyResult<Self>{match value{
        "m9-ironclad-act3"=>Ok(Self::M9IroncladAct3),
        "m10-ironclad-heart"=>Ok(Self::M10IroncladHeart),
        "m10-ironclad-policy"=>Ok(Self::M10IroncladPolicy),
        "standard"=>Ok(Self::Standard),
        _=>Err(RunInputError::new_err("unsupported Run profile")),
    }}
    pub(crate) fn schema_id(self)->&'static str{match self{
        Self::M9IroncladAct3=>RUN_OBSERVATION_SCHEMA_ID,
        Self::M10IroncladHeart=>RUN_OBSERVATION_M10_SCHEMA_ID,Self::M10IroncladPolicy=>spire_env::RUN_OBSERVATION_POLICY_SCHEMA_ID,Self::Standard=>spire_env::RUN_OBSERVATION_STANDARD_SCHEMA_ID,
    }}
    pub(crate) fn new_session(self,config:RunSessionConfig,layout:RunObservationLayout)->Result<RunSession,RunSessionError>{match self{
        Self::M9IroncladAct3=>RunSession::new(config),
        Self::M10IroncladHeart=>RunSession::new_m10(config),Self::M10IroncladPolicy=>RunSession::new_m10_policy(config),
        Self::Standard=>{let M9BossHistory::Standard(creation)=config.boss_history else{return Err(RunSessionError::Input("Standard requires creation and rejects boss_history"))};RunSession::new_standard_with_layout(StandardRunConfig{character:config.character,ascension:config.ascension,partition:config.partition,creation,capacities:config.capacities,record_replay:config.record_replay},layout)},
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
pub(crate) fn capacities(v:Option<&Bound<'_,PyAny>>,profile:RunProfile,layout:RunObservationLayout)->PyResult<RunCapacities>{
    let mut c=match (profile,layout){(RunProfile::M9IroncladAct3,_)=>RunCapacities::m9_default(),(RunProfile::M10IroncladHeart|RunProfile::M10IroncladPolicy,_)=>RunCapacities::m10_default(),(RunProfile::Standard,RunObservationLayout::Dense)=>RunCapacities::standard_default(),(RunProfile::Standard,RunObservationLayout::Compact)=>RunCapacities::standard_compact(),_=>RunCapacities::standard_default()};let Some(v)=v.filter(|v|!v.is_none())else{return Ok(c)};
    let d=v.cast::<PyDict>().map_err(|_|RunInputError::new_err("capacities must be a dictionary"))?;
    for (key,value) in d.iter(){let target=match label(&key)?.as_str(){"map_nodes"=>&mut c.map_nodes,"map_edges"=>&mut c.map_edges,"permanent_cards"=>&mut c.permanent_cards,"combat_cards"=>&mut c.combat_cards,"enemies"=>&mut c.enemies,"powers"=>&mut c.powers,"relics"=>&mut c.relics,"potions"=>&mut c.potions,"offers"=>&mut c.offers,"selection"=>&mut c.selection,"orbs"=>&mut c.orbs,"actions"=>&mut c.actions,_=>return Err(RunInputError::new_err("unknown capacity"))};*target=integer(&value)?;}
    if layout==RunObservationLayout::Compact { spire_env::standard_compact_run_observation_spec(&c).map_err(error)?; }
    Ok(c)
}
/// The value dtypes of `layout`. Dense (every profile) is an empty map: every value tensor defaults to int64, so no
/// descriptor is built. Compact dtypes depend only on the group, so they are read once per process from the
/// `standard-compact-v1` descriptor (review I1: no descriptor build per session, fork or record).
pub(crate) fn layout_dtypes(layout:RunObservationLayout)->PyResult<stacking::DTypeMap>{
    if layout!=RunObservationLayout::Compact{return Ok(stacking::DTypeMap::new())}
    static COMPACT:std::sync::OnceLock<Result<stacking::DTypeMap,&'static str>>=std::sync::OnceLock::new();
    COMPACT.get_or_init(||{
        let spec=spire_env::standard_compact_run_observation_spec(&RunCapacities::standard_compact()).map_err(|_|"compact observation descriptor failed")?;
        let mut result=HashMap::new();
        for tensor in spec.descriptor()["tensors"].as_array().ok_or("observation descriptor tensors are invalid")?{
            let key=tensor["key"].as_str().ok_or("observation descriptor tensor has no key")?;
            let dtype=match tensor["dtype"].as_str(){Some("int64")=>stacking::DType::I64,Some("int32")=>stacking::DType::I32,Some("int16")=>stacking::DType::I16,Some("int8")=>continue,_=>return Err("invalid observation descriptor dtype")};
            result.insert(key.to_owned(),dtype);
        }
        Ok(result)
    }).clone().map_err(RunFaultError::new_err)
}
pub(crate) fn observation_layout(value:&str)->PyResult<RunObservationLayout>{match value{"dense"=>Ok(RunObservationLayout::Dense),"compact"=>Ok(RunObservationLayout::Compact),_=>Err(RunInputError::new_err("observation must be 'dense' or 'compact'"))}}
pub(crate) fn fault_cause_value<'py>(py:Python<'py>,cause:Option<RunSessionError>)->PyResult<Option<Bound<'py,PyAny>>>{
    cause.map(|cause|{let record=stacking::fault_cause_record(&cause);numeric::descriptor(py,record.diagnostic_json().map_err(error)?)}).transpose()
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
pub(crate) fn terminal(t:Option<RunTerminal>)->Option<&'static str>{t.map(|t|match t{RunTerminal::Death=>"death",RunTerminal::EmptyDeck=>"empty_deck",RunTerminal::Act3NoKeys=>"act3_no_keys",RunTerminal::HeartVictory=>"heart_victory"})}
pub(crate) fn observation<'py>(py:Python<'py>,o:RunObservation,dtypes:&stacking::DTypeMap)->PyResult<Bound<'py,PyDict>>{
    let d=PyDict::new(py);
    for g in o.groups{
        let values=match dtypes.get(g.key){
            Some(stacking::DType::I16)=>numeric::array(py,narrow_values::<i16>(&g.values)?,&g.shape)?,
            Some(stacking::DType::I32)=>numeric::array(py,narrow_values::<i32>(&g.values)?,&g.shape)?,
            _=>numeric::array(py,g.values,&g.shape)?,
        };
        d.set_item(g.key,values)?;
        d.set_item(format!("{}_known",g.key),numeric::array(py,g.known,&g.shape)?)?;
        if let Some(mask)=g.occupied{let count=mask.len();d.set_item(format!("{}_occupied",g.key),numeric::array(py,mask,&[count])?)?;}
    }
    let count=o.action_mask.len();d.set_item("action_mask",numeric::array(py,o.action_mask,&[count])?)?;Ok(d)
}
fn narrow_values<T:TryFrom<i64>>(values:&[i64])->PyResult<Vec<T>>{values.iter().copied().map(T::try_from).collect::<Result<Vec<_>,_>>().map_err(|_|RunFaultError::new_err("observation narrowing failed"))}
pub(crate) fn specification<'py>(py:Python<'py>,spec:spire_env::RunObservationSpec)->PyResult<Bound<'py,PyAny>>{
    let d=numeric::descriptor(py,spec.descriptor().to_string())?;
    d.set_item("fingerprint",spec.fingerprint().iter().map(|v|format!("{v:02x}")).collect::<String>())?;Ok(d)
}
pub(crate) fn policy_boundary<'py>(py:Python<'py>,b:spire_env::CombatPolicyBoundary)->PyResult<Bound<'py,PyDict>>{
    let d=PyDict::new(py);d.set_item("obs",observation(py,b.observation,&stacking::DTypeMap::new())?)?;
    let t:[u64;2]=b.token.into();d.set_item("token",(t[0],t[1]))?;
    d.set_item("info",numeric::descriptor(py,b.info.to_string())?)?;Ok(d)
}
fn boundary<'py>(py:Python<'py>,b:RunBoundary,profile:RunProfile,dtypes:&stacking::DTypeMap)->PyResult<Bound<'py,PyDict>>{
    let d=PyDict::new(py);d.set_item("obs",observation(py,b.observation,dtypes)?)?;let token:[u64;2]=b.token.into();d.set_item("token",(token[0],token[1]))?;
    let info=PyDict::new(py);info.set_item("schema_id",profile.schema_id())?;info.set_item("terminal_reason",terminal(b.terminal))?;d.set_item("info",info)?;Ok(d)
}
pub(crate) fn reward_events<'py>(py:Python<'py>,items:&[spire_core::run::RewardItem])->PyResult<Bound<'py,PyList>>{
    use spire_core::run::RewardItemKind as K;
    let list=PyList::empty(py);
    for item in items{let row=PyDict::new(py);let kind=match item.kind{K::NewFloor{act,floor}=>{row.set_item("act",act)?;row.set_item("floor",floor)?;"new_floor"},K::NewAct4Floor{floor}=>{row.set_item("floor",floor)?;"new_act4_floor"},K::FirstKey(key)=>{row.set_item("key",match key{spire_core::run::Key::Ruby=>"ruby",spire_core::run::Key::Sapphire=>"sapphire",spire_core::run::Key::Emerald=>"emerald"})?;"first_key"},K::HeartVictory=>"heart_victory",K::Death=>"death",K::EmptyDeck=>"empty_deck"};row.set_item("kind",kind)?;row.set_item("amount",item.amount)?;list.append(row)?;}
    Ok(list)
}
pub(crate) fn token(v:&Bound<'_,PyAny>)->PyResult<RunDecisionToken>{
    let t=v.cast::<PyTuple>().map_err(|_|RunInputError::new_err("token must be a two-integer tuple"))?;
    if t.len()!=2{return Err(RunInputError::new_err("token must have length two"))}Ok([integer(&t.get_item(0)?)?,integer(&t.get_item(1)?)?].into())
}

pub(crate) fn search_limits(live:&Bound<'_,PyAny>,branches:&Bound<'_,PyAny>,per_branch:&Bound<'_,PyAny>,total:&Bound<'_,PyAny>)->PyResult<SearchLimits>{
    Ok(SearchLimits{max_live_branches:integer(live)?,max_branches:integer(branches)?,max_steps_per_branch:integer(per_branch)?,max_total_steps:integer(total)?})
}
fn search_error()->PyErr{RunStateError::new_err("public search is closed")}
/// `workers=None` is one worker (the M34s default).
pub(crate) fn one_worker<'py>(py:Python<'py>,workers:Option<&Bound<'py,PyAny>>)->Bound<'py,PyAny>{workers.filter(|w|!w.is_none()).cloned().unwrap_or_else(||1_i64.into_pyobject(py).unwrap().into_any())}
pub(crate) fn public_search(root:PublicRoot,research_seed:u64,limits:SearchLimits,workers:&Bound<'_,PyAny>)->PyResult<NativePublicSearch>{
    let workers=integer::<usize>(workers)?;
    PublicSearch::new(root,SearchConfig{research_seed,limits}).and_then(|s|s.with_workers(workers)).map(|inner|NativePublicSearch{inner:Some(inner)}).map_err(error)
}
/// Thread-movable (M35): a search may be created on one Python thread and stepped on another; PyO3's borrow check
/// still rejects concurrent use of one search.
#[pyclass(module="spire_lab._native")]
pub(crate) struct NativePublicSearch{inner:Option<PublicSearch>}
#[pymethods]
impl NativePublicSearch{
    fn __repr__(&self)->String{match &self.inner{Some(s)=>{let c=s.counters();format!("<NativePublicSearch live_branches={} branches_created={} total_steps={}>",c["live_branches"],c["branches_created"],c["total_steps"])},None=>"<NativePublicSearch closed>".to_owned()}}
    fn __reduce__(&self)->PyResult<()>{Err(PyTypeError::new_err("PublicSearch is in-memory only and cannot be pickled"))}
    #[pyo3(signature=(sample=None))]
    fn branch(&mut self,sample:Option<&Bound<'_,PyAny>>)->PyResult<u64>{let sample=sample.filter(|v|!v.is_none()).map(integer::<u64>).transpose()?;self.inner.as_mut().ok_or_else(search_error)?.branch(sample).map(|b|b.0).map_err(error)}
    fn decision<'py>(&self,py:Python<'py>,branch:&Bound<'_,PyAny>)->PyResult<Bound<'py,PyDict>>{let b=BranchId(integer(branch)?);let d=self.inner.as_ref().ok_or_else(search_error)?.decision(b).map_err(error)?;let out=PyDict::new(py);out.set_item("obs",observation(py,d.observation,&stacking::DTypeMap::new())?)?;let t:[u64;2]=d.token.into();out.set_item("token",(t[0],t[1]))?;out.set_item("info",numeric::descriptor(py,d.info.to_string())?)?;Ok(out)}
    #[pyo3(signature=(branch,token,action,view="full".to_owned()))]
    fn step<'py>(&mut self,py:Python<'py>,branch:&Bound<'_,PyAny>,token:&Bound<'_,PyAny>,action:&Bound<'_,PyAny>,#[pyo3(from_py_with=label)]view:String)->PyResult<Bound<'py,PyDict>>{
        let view=match view.as_str(){"full"=>BranchView::Full,"mask"=>BranchView::Mask,_=>return Err(RunInputError::new_err("view must be 'full' or 'mask'"))};
        let result=self.inner.as_mut().ok_or_else(search_error)?.step(BranchId(integer(branch)?),crate::run_env::token(token)?,RunActionIndex(integer(action)?),view).map_err(error)?;
        let out=PyDict::new(py);let obs=match result.observation{Some(o)=>Some(observation(py,o,&stacking::DTypeMap::new())?),None=>None};out.set_item("obs",obs)?;let count=result.action_mask.len();out.set_item("action_mask",numeric::array(py,result.action_mask,&[count])?)?;let t:[u64;2]=result.token.into();out.set_item("token",(t[0],t[1]))?;out.set_item("reward",result.reward)?;out.set_item("terminated",result.terminated)?;out.set_item("truncated",result.truncated)?;out.set_item("info",numeric::descriptor(py,result.info.to_string())?)?;Ok(out)
    }
    fn fork(&mut self,branch:&Bound<'_,PyAny>)->PyResult<u64>{self.inner.as_mut().ok_or_else(search_error)?.fork(BranchId(integer(branch)?)).map(|b|b.0).map_err(error)}
    fn actions(&self,branch:&Bound<'_,PyAny>)->PyResult<Vec<i64>>{self.inner.as_ref().ok_or_else(search_error)?.actions(BranchId(integer(branch)?)).map_err(error)}
    #[pyo3(signature=(branch=None))]
    fn close(&mut self,branch:Option<&Bound<'_,PyAny>>)->PyResult<()>{if let Some(branch)=branch{self.inner.as_mut().ok_or_else(search_error)?.close(BranchId(integer(branch)?)).map_err(error)}else{self.inner=None;Ok(())}}
    #[pyo3(signature=(branch,research_seed,*,max_live_branches,max_branches,max_steps_per_branch,max_total_steps,workers=None))]
    #[allow(clippy::too_many_arguments)]
    fn public_search<'py>(&self,py:Python<'py>,branch:&Bound<'_,PyAny>,research_seed:&Bound<'_,PyAny>,max_live_branches:&Bound<'_,PyAny>,max_branches:&Bound<'_,PyAny>,max_steps_per_branch:&Bound<'_,PyAny>,max_total_steps:&Bound<'_,PyAny>,workers:Option<&Bound<'py,PyAny>>)->PyResult<Self>{let root=self.inner.as_ref().ok_or_else(search_error)?.public_root(BranchId(integer(branch)?)).map_err(error)?;public_search(root,integer(research_seed)?,search_limits(max_live_branches,max_branches,max_steps_per_branch,max_total_steps)?,&one_worker(py,workers))}
    /// M35 batched branch stepping (design §3.6): one action per listed branch, outcomes in input order; the GIL is
    /// released for the native part. Preflight errors change nothing.
    #[pyo3(signature=(branches,tokens,actions,*,view="full".to_owned()))]
    fn step_many<'py>(&mut self,py:Python<'py>,branches:&Bound<'py,PyAny>,tokens:&Bound<'py,PyAny>,actions:&Bound<'py,PyAny>,#[pyo3(from_py_with=label)]view:String)->PyResult<Bound<'py,PyDict>>{
        let view=match view.as_str(){"full"=>BranchView::Full,"mask"=>BranchView::Mask,_=>return Err(RunInputError::new_err("view must be 'full' or 'mask'"))};
        let ids=branches.cast::<PyList>().map_err(|_|RunInputError::new_err("branches must be a list of branch ids"))?.iter().map(|b|integer::<u64>(&b).map(BranchId)).collect::<PyResult<Vec<_>>>()?;
        if ids.is_empty(){return Err(RunInputError::new_err("step_many needs at least one branch"))}
        let (actions,tokens)=numeric::step_inputs(actions,tokens,ids.len()).map_err(|e|RunInputError::new_err(e.value(py).to_string()))?;
        let requests=ids.iter().zip(tokens).zip(actions).map(|((b,t),a)|BranchStepRequest{branch:*b,token:t.into(),index:RunActionIndex(a)}).collect::<Vec<_>>();
        let search=self.inner.as_mut().ok_or_else(search_error)?;
        let outcomes=py.detach(||search.step_many(&requests,view)).map_err(error)?;
        crate::batch::branch_outcomes(py,outcomes)
    }
    fn counters<'py>(&self,py:Python<'py>)->PyResult<Bound<'py,PyAny>>{let v=self.inner.as_ref().ok_or_else(search_error)?.counters();numeric::descriptor(py,v.to_string())}
    fn spec<'py>(&self,py:Python<'py>)->PyResult<Bound<'py,PyAny>>{let v=self.inner.as_ref().ok_or_else(search_error)?.spec();numeric::descriptor(py,v.to_string())}
}
/// M34c privileged exact checkpoint: opaque, in-memory only (pickling/copying raise), redacted repr, identity equality.
#[pyclass(frozen,module="spire_lab._native",name="ExactCheckpoint")]
pub(crate) struct PyExactCheckpoint{inner:spire_env::ExactCheckpoint,profile:RunProfile,episode_return:i64}
#[pymethods]
impl PyExactCheckpoint{
    fn __repr__(&self)->&'static str{"<ExactCheckpoint privileged exact state (redacted)>"}
    fn __reduce__(&self)->PyResult<()>{Err(PyTypeError::new_err("ExactCheckpoint is in-memory only and cannot be pickled or copied; record() is the portable form"))}
}
pub(crate) fn reproduction<'py>(py:Python<'py>,r:spire_env::RunReproduction)->PyResult<Bound<'py,PyDict>>{
    let d=PyDict::new(py);d.set_item("seed",r.seed)?;d.set_item("ordinal",r.ordinal)?;d.set_item("closed",r.closed)?;d.set_item("fault",r.fault.map(|f|format!("{f:?}")))?;d.set_item("last_accepted_action",r.last_accepted_step.map(|s|format!("{:?}",s.action)))?;
    if let Some(record)=r.record{d.set_item("record",PyBytes::new(py,&record.to_json().map_err(error)?))?;}else{d.set_item("record",py.None())?;}Ok(d)
}
/// One session's config from the Python constructor arguments (shared by `NativeRunSession` and `NativeRunBatch`).
#[allow(clippy::too_many_arguments)]
pub(crate) fn session_config(py:Python<'_>,profile:RunProfile,character:&Bound<'_,PyAny>,ascension:&Bound<'_,PyAny>,partition:&str,boss_history:Option<&Bound<'_,PyAny>>,creation:Option<&Bound<'_,PyAny>>,capacities:RunCapacities,record_replay:bool)->PyResult<RunSessionConfig>{
    let character=match (profile,label(character)?.as_str()){
        (RunProfile::Standard,"ironclad")=>Character::Ironclad,(RunProfile::Standard,"silent")=>Character::Silent,(RunProfile::Standard,"defect")=>Character::Defect,(RunProfile::Standard,"watcher")=>Character::Watcher,
        (RunProfile::M9IroncladAct3|RunProfile::M10IroncladHeart|RunProfile::M10IroncladPolicy,"ironclad")=>Character::Ironclad,
        (RunProfile::Standard,_)=>return Err(RunInputError::new_err("Standard character must be ironclad, silent, defect, or watcher")),
        (RunProfile::M9IroncladAct3,_)=>return Err(RunInputError::new_err("M9 supports ironclad only")),
        (RunProfile::M10IroncladHeart|RunProfile::M10IroncladPolicy,_)=>return Err(RunInputError::new_err("M10 supports ironclad only")),
    };
    let partition=match partition{"train"=>Partition::Train,"selection"=>Partition::Selection,"final"=>Partition::Final,"test"=>Partition::Test,_=>return Err(RunInputError::new_err("unknown partition"))};
    let creation=if profile==RunProfile::Standard{self::creation(py,creation)?}else{if creation.is_some_and(|v|!v.is_none()){return Err(RunInputError::new_err("captured profiles do not accept creation"))}StandardCreation::experienced()};
    let boss_history=if profile==RunProfile::Standard{
        if boss_history.is_some_and(|v|!v.is_none()){return Err(RunInputError::new_err("Standard does not accept boss_history"))}
        M9BossHistory::Standard(creation)
    }else{
        let value=match boss_history.filter(|v|!v.is_none()){Some(v)=>label(v)?,None=>"awakened-unseen".to_owned()};
        match value.as_str(){"awakened-unseen"=>M9BossHistory::AwakenedUnseen,"time-eater-unseen"=>M9BossHistory::TimeEaterUnseen,"donu-deca-unseen"=>M9BossHistory::DonuDecaUnseen,_=>return Err(RunInputError::new_err("unknown boss history"))}
    };
    Ok(RunSessionConfig{character,ascension:integer(ascension)?,partition,boss_history,capacities,record_replay})
}
#[pyclass(frozen,module="spire_lab._native")]
pub(crate) struct NativeRunSession{session:Mutex<RunSession>,profile:RunProfile,dtypes:stacking::DTypeMap,busy:AtomicBool,episode_return:AtomicI64}
struct Lease<'a>(&'a NativeRunSession);
impl Drop for Lease<'_>{fn drop(&mut self){self.0.busy.store(false,Ordering::Release);}}
impl NativeRunSession{
    fn wrap(_py:Python<'_>,session:RunSession,profile:RunProfile,episode_return:i64)->PyResult<Self>{let dtypes=layout_dtypes(session.observation_layout())?;Ok(Self{session:Mutex::new(session),profile,dtypes,busy:AtomicBool::new(false),episode_return:AtomicI64::new(episode_return)})}
    /// Current boundary payload (reset-shaped: effective seed and, on Standard, the creation profile).
    fn current<'py>(&self,py:Python<'py>,b:RunBoundary,effective:Option<u64>,creation:Option<StandardCreation>)->PyResult<Bound<'py,PyDict>>{
        let raw=boundary(py,b,self.profile,&self.dtypes)?;raw.set_item("effective_seed",effective)?;
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
        let result=(||{let raw=boundary(py,b,self.profile,&self.dtypes)?;raw.set_item("effective_seed",effective)?;if let Some(c)=creation{raw.get_item("info")?.unwrap().set_item("creation",numeric::descriptor(py,c.to_json().to_string())?)?;}package(raw)})();self.deliver(result)
    }
    fn step_transport<'py,T>(&self,py:Python<'py>,index:&Bound<'py,PyAny>,t:&Bound<'py,PyAny>,package:impl FnOnce(Bound<'py,PyDict>)->PyResult<T>)->PyResult<T>{
        let _lease=self.begin()?;let index=RunActionIndex(integer(index)?);let t=token(t)?;
        let step=py.detach(||self.session.lock().unwrap_or_else(|p|p.into_inner()).step(t,index)).map_err(error)?;
        let running=self.episode_return.fetch_add(i64::from(step.reward),Ordering::AcqRel)+i64::from(step.reward);
        let heart_victory=step.boundary.terminal==Some(RunTerminal::HeartVictory);
        let result=(||{let raw=boundary(py,step.boundary,self.profile,&self.dtypes)?;raw.set_item("reward",step.reward)?;raw.set_item("terminated",step.terminated)?;raw.set_item("truncated",step.truncated)?;if self.profile==RunProfile::Standard{let info=raw.get_item("info")?.unwrap();info.set_item("reward_events",reward_events(py,&step.reward_events)?)?;info.set_item("episode_return",running)?;info.set_item("heart_victory",heart_victory)?;}package(raw)})();self.deliver(result)
    }
}
#[pymethods]
impl NativeRunSession{
    #[new]
    #[pyo3(signature=(character,ascension,*,profile="m9-ironclad-act3".to_owned(),partition="train".to_owned(),boss_history=None,creation=None,capacities=None,record_replay=false,observation="dense".to_owned()))]
    fn new(py:Python<'_>,character:&Bound<'_,PyAny>,ascension:&Bound<'_,PyAny>,#[pyo3(from_py_with=label)]profile:String,#[pyo3(from_py_with=label)]partition:String,boss_history:Option<&Bound<'_,PyAny>>,creation:Option<&Bound<'_,PyAny>>,capacities:Option<&Bound<'_,PyAny>>,#[pyo3(from_py_with=boolean)]record_replay:bool,#[pyo3(from_py_with=label)]observation:String)->PyResult<Self>{
        let layout=observation_layout(&observation)?;let profile=RunProfile::parse(&profile)?;
        if layout==RunObservationLayout::Compact&&profile!=RunProfile::Standard{return Err(RunInputError::new_err("observation='compact' requires the standard profile"))}
        let capacities=crate::run_env::capacities(capacities,profile,layout)?;
        let cfg=session_config(py,profile,character,ascension,&partition,boss_history,creation,capacities,record_replay)?;
        let session=py.detach(||profile.new_session(cfg,layout)).map_err(error)?;Self::wrap(py,session,profile,0)
    }
    /// Owned current boundary (after fork or from_record the caller reads its first token here).
    fn decision<'py>(&self,py:Python<'py>)->PyResult<Bound<'py,PyDict>>{
        let _lease=self.begin()?;
        let (b,effective,creation)=py.detach(||{let s=self.session.lock().unwrap_or_else(|p|p.into_inner());Ok::<_,RunSessionError>((s.decision()?,s.effective_seed(),s.standard_creation()))}).map_err(error)?;
        self.deliver(self.current(py,b,effective,creation))
    }
    #[pyo3(signature=(research_seed,*,max_live_branches,max_branches,max_steps_per_branch,max_total_steps,workers=None))]
    #[allow(clippy::too_many_arguments)]
    fn public_search<'py>(&self,py:Python<'py>,research_seed:&Bound<'_,PyAny>,max_live_branches:&Bound<'_,PyAny>,max_branches:&Bound<'_,PyAny>,max_steps_per_branch:&Bound<'_,PyAny>,max_total_steps:&Bound<'_,PyAny>,workers:Option<&Bound<'py,PyAny>>)->PyResult<NativePublicSearch>{
        let limits=search_limits(max_live_branches,max_branches,max_steps_per_branch,max_total_steps)?;
        let _lease=self.begin()?;
        let root=self.session.lock().unwrap_or_else(|p|p.into_inner()).public_root().map_err(error)?;
        public_search(root,integer(research_seed)?,limits,&one_worker(py,workers))
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
        Self::wrap(py,session,cp.profile,cp.episode_return)
    }
    /// Replay to a decision: a live session after the first `steps` accepted actions of `record` (checked as replay()).
    #[staticmethod]
    #[pyo3(signature=(record,steps,capacities=None))]
    fn from_record(py:Python<'_>,record:&Bound<'_,PyBytes>,steps:&Bound<'_,PyAny>,capacities:Option<&Bound<'_,PyAny>>)->PyResult<Self>{
        let steps:usize=integer(steps)?;let bytes=record.as_bytes().to_vec();
        let parsed=RunEpisodeRecord::from_json(&bytes).map_err(error)?;let profile=RunProfile::from_record(&parsed).map_err(error)?;let layout=spire_env::record_observation_layout(&parsed).map_err(error)?;
        let caps=crate::run_env::capacities(capacities,profile,layout)?;
        let episode_return=parsed.steps.iter().take(steps).map(|s|i64::from(s.reward)).sum();
        let session=py.detach(move||RunSession::from_record(&parsed,steps,caps)).map_err(error)?;
        Self::wrap(py,session,profile,episode_return)
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
    fn fault_cause<'py>(&self,py:Python<'py>)->PyResult<Option<Bound<'py,PyAny>>>{
        let _lease=self.begin()?;let cause=self.session.lock().unwrap_or_else(|p|p.into_inner()).fault_cause();fault_cause_value(py,cause)
    }
    fn reproduction<'py>(&self,py:Python<'py>)->PyResult<Bound<'py,PyDict>>{
        let _lease=self.begin()?;let r=py.detach(||self.session.lock().unwrap_or_else(|p|p.into_inner()).reproduction()).map_err(error)?;
        reproduction(py,r)
    }
    fn close(&self,py:Python<'_>)->PyResult<()>{let _lease=self.begin()?;py.detach(||self.session.lock().unwrap_or_else(|p|p.into_inner()).close()).map_err(error)}
    #[staticmethod]
    #[pyo3(signature=(record,capacities=None))]
    fn replay<'py>(py:Python<'py>,record:&Bound<'py,PyBytes>,capacities:Option<&Bound<'py,PyAny>>)->PyResult<Bound<'py,PyDict>>{
        let bytes=record.as_bytes().to_vec();let parsed=RunEpisodeRecord::from_json(&bytes).map_err(error)?;let profile=RunProfile::from_record(&parsed).map_err(error)?;let layout=spire_env::record_observation_layout(&parsed).map_err(error)?;let ascension=parsed.config.ascension;
        let caps=crate::run_env::capacities(capacities,profile,layout)?;let export_caps=caps.clone();let dtypes=layout_dtypes(layout)?;let report=py.detach(move||spire_env::replay_run(&parsed,caps)).map_err(error)?;
        let d=PyDict::new(py);let o=if profile==RunProfile::Standard{spire_env::encode_run_standard_decision_with_layout(&report.final_decision,report.final_map.as_ref(),&export_caps,ascension,report.final_room_selection,layout)}else{profile.encode(&report.final_decision,report.final_map.as_ref(),&export_caps,ascension,report.final_room_selection)}.map_err(error)?;d.set_item("obs",observation(py,o,&dtypes)?)?;d.set_item("terminal_reason",terminal(report.final_decision.terminal))?;d.set_item("rewards",report.rewards)?;d.set_item("terminals",report.terminals.into_iter().map(terminal).collect::<Vec<_>>())?;d.set_item("accepted_count",report.accepted_count)?;
        if let Some(f)=report.reported_fault{d.set_item("reported_fault",numeric::descriptor(py,f.error.diagnostic_json().map_err(error)?)?)?;}else{d.set_item("reported_fault",py.None())?;}
        d.set_item("final_decision_error",report.final_decision_error.map(|_|"native core fault; final obs is the last readable boundary"))?;Ok(d)
    }
}
pub(crate) fn register(m:&Bound<'_,PyModule>)->PyResult<()>{let py=m.py();m.add_class::<NativePublicSearch>()?;m.add("RunInputError",py.get_type::<RunInputError>())?;m.add("RunStateError",py.get_type::<RunStateError>())?;m.add("RunFaultError",py.get_type::<RunFaultError>())?;m.add("RunUnsupportedBoundaryError",py.get_type::<RunUnsupportedBoundaryError>())?;m.add("RunBudgetError",py.get_type::<RunBudgetError>())?;Ok(())}
