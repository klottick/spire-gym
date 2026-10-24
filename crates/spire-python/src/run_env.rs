//! Thin Run transport; native ownership spans attached array/tuple packaging.
use std::sync::{Mutex,atomic::{AtomicBool,Ordering}};
use pyo3::{prelude::*,create_exception,exceptions::{PyValueError,PyRuntimeError,PyNotImplementedError},types::{PyAny,PyDict,PyInt,PyBool,PyString,PyTuple,PyBytes}};
use spire_core::{content::Character,combat::rng::Partition,run::RunTerminal};
use spire_env::{RunSession,RunSessionConfig,RunSessionError,RunCapacities,RunBoundary,RunObservation,RunActionIndex,RunDecisionToken,M9BossHistory};
use crate::numeric;

create_exception!(_native,RunInputError,PyValueError);
create_exception!(_native,RunStateError,PyRuntimeError);
create_exception!(_native,RunFaultError,PyRuntimeError);
create_exception!(_native,RunUnsupportedBoundaryError,PyNotImplementedError);
fn integer<T:for<'a,'py> FromPyObject<'a,'py>>(v:&Bound<'_,PyAny>)->PyResult<T>{
    if !v.is_exact_instance_of::<PyInt>(){return Err(RunInputError::new_err("expected built-in integer, not bool"))}
    v.extract().map_err(|_|RunInputError::new_err("integer outside native range"))
}
fn label(v:&Bound<'_,PyAny>)->PyResult<String>{if !v.is_exact_instance_of::<PyString>(){return Err(RunInputError::new_err("expected string"))}v.extract().map_err(Into::into)}
fn boolean(v:&Bound<'_,PyAny>)->PyResult<bool>{if !v.is_exact_instance_of::<PyBool>(){return Err(RunInputError::new_err("expected bool"))}v.extract().map_err(Into::into)}
fn error(e:RunSessionError)->PyErr{match e{
    RunSessionError::Input(s)=>RunInputError::new_err(s),RunSessionError::StaleToken=>RunStateError::new_err("stale Run decision token"),RunSessionError::NotReset=>RunStateError::new_err("Run has not been reset"),RunSessionError::Terminal=>RunStateError::new_err("Run is terminal"),RunSessionError::Closed=>RunStateError::new_err("Run is closed"),
    RunSessionError::UnsupportedBoundary(s)=>RunUnsupportedBoundaryError::new_err(s),RunSessionError::DeliveryFault(s)=>RunFaultError::new_err(s),RunSessionError::Capacity{group,required,capacity}=>RunFaultError::new_err(format!("Run capacity {group}: {required} > {capacity}")),RunSessionError::IncompatibleReplay(s)=>RunInputError::new_err(s),RunSessionError::Core(_)=>RunFaultError::new_err("native Run fault; inspect privileged reproduction"),
}}
fn capacities(v:Option<&Bound<'_,PyAny>>)->PyResult<RunCapacities>{
    let mut c=RunCapacities::m9_default();let Some(v)=v.filter(|v|!v.is_none())else{return Ok(c)};
    let d=v.cast::<PyDict>().map_err(|_|RunInputError::new_err("capacities must be a dictionary"))?;
    for (key,value) in d.iter(){let target=match label(&key)?.as_str(){"map_nodes"=>&mut c.map_nodes,"map_edges"=>&mut c.map_edges,"permanent_cards"=>&mut c.permanent_cards,"combat_cards"=>&mut c.combat_cards,"enemies"=>&mut c.enemies,"powers"=>&mut c.powers,"relics"=>&mut c.relics,"potions"=>&mut c.potions,"offers"=>&mut c.offers,"selection"=>&mut c.selection,"orbs"=>&mut c.orbs,"actions"=>&mut c.actions,_=>return Err(RunInputError::new_err("unknown capacity"))};*target=integer(&value)?;}
    Ok(c)
}
fn terminal(t:Option<RunTerminal>)->Option<&'static str>{t.map(|t|match t{RunTerminal::Death=>"death",RunTerminal::EmptyDeck=>"empty_deck",RunTerminal::Act3NoKeys=>"act3_no_keys",RunTerminal::HeartVictory=>"heart_victory"})}
fn observation<'py>(py:Python<'py>,o:RunObservation)->PyResult<Bound<'py,PyDict>>{
    let d=PyDict::new(py);
    for g in o.groups{
        d.set_item(g.key,numeric::array(py,g.values,&g.shape)?)?;
        d.set_item(format!("{}_known",g.key),numeric::array(py,g.known,&g.shape)?)?;
        if let Some(mask)=g.occupied{let count=mask.len();d.set_item(format!("{}_occupied",g.key),numeric::array(py,mask,&[count])?)?;}
    }
    let count=o.action_mask.len();d.set_item("action_mask",numeric::array(py,o.action_mask,&[count])?)?;Ok(d)
}
fn boundary<'py>(py:Python<'py>,b:RunBoundary)->PyResult<Bound<'py,PyDict>>{
    let d=PyDict::new(py);d.set_item("obs",observation(py,b.observation)?)?;let token:[u64;2]=b.token.into();d.set_item("token",(token[0],token[1]))?;
    let info=PyDict::new(py);info.set_item("schema_id",spire_env::RUN_OBSERVATION_SCHEMA_ID)?;info.set_item("terminal_reason",terminal(b.terminal))?;d.set_item("info",info)?;Ok(d)
}
fn token(v:&Bound<'_,PyAny>)->PyResult<RunDecisionToken>{
    let t=v.cast::<PyTuple>().map_err(|_|RunInputError::new_err("token must be a two-integer tuple"))?;
    if t.len()!=2{return Err(RunInputError::new_err("token must have length two"))}Ok([integer(&t.get_item(0)?)?,integer(&t.get_item(1)?)?].into())
}
#[pyclass(frozen,module="spire_lab._native")]
pub(crate) struct NativeRunSession{session:Mutex<RunSession>,caps:RunCapacities,busy:AtomicBool}
struct Lease<'a>(&'a NativeRunSession);
impl Drop for Lease<'_>{fn drop(&mut self){self.0.busy.store(false,Ordering::Release);}}
impl NativeRunSession{
    fn begin(&self)->PyResult<Lease<'_>>{self.busy.compare_exchange(false,true,Ordering::Acquire,Ordering::Relaxed).map_err(|_|RunStateError::new_err("Run operation is busy"))?;Ok(Lease(self))}
    fn deliver<T>(&self,result:PyResult<T>)->PyResult<T>{if result.is_err(){let _=self.session.lock().unwrap_or_else(|p|p.into_inner()).fail_delivery("Run transport packaging failed");}result}
    fn reset_transport<'py,T>(&self,py:Python<'py>,seed:Option<&Bound<'py,PyAny>>,package:impl FnOnce(Bound<'py,PyDict>)->PyResult<T>)->PyResult<T>{
        let _lease=self.begin()?;let seed=seed.filter(|v|!v.is_none()).map(integer::<u64>).transpose()?;
        let (b,effective)=py.detach(||{let mut s=self.session.lock().unwrap_or_else(|p|p.into_inner());let b=s.reset(seed)?;Ok::<_,RunSessionError>((b,s.effective_seed().expect("successful reset owns seed")))}).map_err(error)?;
        let result=(||{let raw=boundary(py,b)?;raw.set_item("effective_seed",effective)?;package(raw)})();self.deliver(result)
    }
    fn step_transport<'py,T>(&self,py:Python<'py>,index:&Bound<'py,PyAny>,t:&Bound<'py,PyAny>,package:impl FnOnce(Bound<'py,PyDict>)->PyResult<T>)->PyResult<T>{
        let _lease=self.begin()?;let index=RunActionIndex(integer(index)?);let t=token(t)?;
        let step=py.detach(||self.session.lock().unwrap_or_else(|p|p.into_inner()).step(t,index)).map_err(error)?;
        let result=(||{let raw=boundary(py,step.boundary)?;raw.set_item("reward",step.reward)?;raw.set_item("terminated",step.terminated)?;raw.set_item("truncated",step.truncated)?;package(raw)})();self.deliver(result)
    }
}
#[pymethods]
impl NativeRunSession{
    #[new]
    #[pyo3(signature=(character,ascension,*,partition="train".to_owned(),boss_history="awakened-unseen".to_owned(),capacities=None,record_replay=false))]
    fn new(py:Python<'_>,character:&Bound<'_,PyAny>,ascension:&Bound<'_,PyAny>,#[pyo3(from_py_with=label)]partition:String,#[pyo3(from_py_with=label)]boss_history:String,capacities:Option<&Bound<'_,PyAny>>,#[pyo3(from_py_with=boolean)]record_replay:bool)->PyResult<Self>{
        let character=match label(character)?.as_str(){"ironclad"=>Character::Ironclad,_=>return Err(RunInputError::new_err("M9 supports ironclad only"))};
        let partition=match partition.as_str(){"train"=>Partition::Train,"selection"=>Partition::Selection,"final"=>Partition::Final,"test"=>Partition::Test,_=>return Err(RunInputError::new_err("unknown partition"))};
        let boss_history=match boss_history.as_str(){"awakened-unseen"=>M9BossHistory::AwakenedUnseen,"time-eater-unseen"=>M9BossHistory::TimeEaterUnseen,"donu-deca-unseen"=>M9BossHistory::DonuDecaUnseen,_=>return Err(RunInputError::new_err("unknown boss history"))};
        let caps=crate::run_env::capacities(capacities)?;let cfg=RunSessionConfig{character,ascension:integer(ascension)?,partition,boss_history,capacities:caps.clone(),record_replay};
        let session=py.detach(||RunSession::new(cfg)).map_err(error)?;Ok(Self{session:Mutex::new(session),caps,busy:AtomicBool::new(false)})
    }
    fn observation_spec<'py>(&self,py:Python<'py>)->PyResult<Bound<'py,PyAny>>{
        let spec=spire_env::run_observation_spec(&self.caps).map_err(error)?;let d=numeric::descriptor(py,spec.descriptor().to_string())?;
        d.set_item("fingerprint",spec.fingerprint().iter().map(|v|format!("{v:02x}")).collect::<String>())?;Ok(d)
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
        let caps=crate::run_env::capacities(capacities)?;let export_caps=caps.clone();let bytes=record.as_bytes().to_vec();let report=py.detach(||{let r=spire_env::RunEpisodeRecord::from_json(&bytes)?;spire_env::replay_run(&r,caps)}).map_err(error)?;
        let d=PyDict::new(py);let o=spire_env::encode_run(&report.final_decision,report.final_map.as_ref(),&export_caps).map_err(error)?;d.set_item("obs",observation(py,o)?)?;d.set_item("terminal_reason",terminal(report.final_decision.terminal))?;d.set_item("rewards",report.rewards)?;d.set_item("terminals",report.terminals.into_iter().map(terminal).collect::<Vec<_>>())?;d.set_item("accepted_count",report.accepted_count)?;
        if let Some(f)=report.reported_fault{d.set_item("reported_fault",numeric::descriptor(py,f.error.diagnostic_json().map_err(error)?)?)?;}else{d.set_item("reported_fault",py.None())?;}
        d.set_item("final_decision_error",report.final_decision_error.map(|_|"native core fault; final obs is the last readable boundary"))?;Ok(d)
    }
}
pub(crate) fn register(m:&Bound<'_,PyModule>)->PyResult<()>{let py=m.py();m.add("RunInputError",py.get_type::<RunInputError>())?;m.add("RunStateError",py.get_type::<RunStateError>())?;m.add("RunFaultError",py.get_type::<RunFaultError>())?;m.add("RunUnsupportedBoundaryError",py.get_type::<RunUnsupportedBoundaryError>())?;Ok(())}
