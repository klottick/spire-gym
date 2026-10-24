//! Opt-in numeric traces and explicit from-start reproduction. No snapshot import.
//!
//! Queues have no retention cap: callers must explicitly drain them. Normal play
//! stores indices and shared immutable setup only; descriptor materialization and
//! snapshots occur solely on privileged export/replay. Consumers persisting an
//! export must create fresh exclusive files and retain failed evidence.
use std::sync::Arc;
use serde_json::Value;
use sha2::{Digest, Sha256};
use spire_core::combat::{CONTENT_ID, Snapshot};
use spire_core::combat::rng::RNG_CONTRACT_ID;
use crate::{EnvError, EpisodeInfo, EpisodeJob, InputReason, ObservationRow, SlotFault,
    ScenarioCatalog, SEED_ADAPTER_ID, OBSERVATION_SCHEMA_ID, observation_spec, canonical_json};
use crate::error::{input, reserved, setup};
use crate::slot::Slot;

pub const RECORD_SCHEMA_ID:&str="spire-lab-combat-reproduction-v1";
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecordStatus { Active, Completed, Abandoned, Faulted }
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EpisodeRecord {
    pub record_schema_id:String,
    pub job:EpisodeJob,
    pub catalog_descriptor:Value,
    pub content_id:String,
    pub schema_id:String,
    pub schema_fingerprint:[u8;32],
    pub seed_adapter_id:String,
    pub rng_contract_id:String,
    pub actions:Vec<i64>,
    pub status:RecordStatus,
    pub attempted_action:Option<i64>,
    pub fault:Option<SlotFault>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct ReplayBoundary { pub observation:ObservationRow, pub info:EpisodeInfo, pub reward:f32 }
#[derive(Clone, Debug, PartialEq)]
pub struct ReplayResult {
    pub boundaries:Vec<ReplayBoundary>, pub final_snapshot:Snapshot, pub fault:Option<SlotFault>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReplayVisitKind { Initial, AcceptedAction, FaultAttempt }
#[derive(Clone, Debug, PartialEq)]
pub struct ReplayVisit {
    pub index:usize, pub kind:ReplayVisitKind, pub action:Option<spire_core::combat::Action>,
    pub public:Option<spire_core::combat::PublicView>, pub effects:Vec<spire_core::combat::ResolvedEffect>,
    pub boundary:Option<ReplayBoundary>,
}
pub fn replay_observed<F>(record:&EpisodeRecord,mut visitor:F)->Result<ReplayResult,EnvError>
where F:FnMut(ReplayVisit)->Result<(),String> { replay_visiting(record,|_|{},Some(&mut visitor)) }

// Completed records share immutable ownership with their former live slot, so
// completion moves the action buffer without copying it. Only exceptional fault
// annotation can require copy-on-write; export always creates an owned public value.
#[derive(Clone, Debug)]
pub(crate) struct Trace {
    job:EpisodeJob, descriptor:Arc<Value>, pub(crate) actions:Vec<i64>,
    pub(crate) status:RecordStatus, pub(crate) attempted_action:Option<i64>,
    pub(crate) fault:Option<SlotFault>,
}
impl Trace {
    pub(crate) fn new(job:EpisodeJob, descriptor:Arc<Value>) -> Arc<Self> {
        Arc::new(Self { job,descriptor,actions:vec![],status:RecordStatus::Active,
            attempted_action:None,fault:None })
    }
    pub(crate) fn reserve(&mut self) -> Result<(),EnvError> {
        self.actions.try_reserve(1).map_err(|_|setup("trace append capacity"))
    }
    pub(crate) fn export(&self) -> EpisodeRecord {
        self.clone().into_export()
    }
    pub(crate) fn export_owned(trace:Arc<Self>) -> EpisodeRecord {
        match Arc::try_unwrap(trace) {
            Ok(trace)=>trace.into_export(),
            Err(trace)=>trace.export(),
        }
    }
    fn into_export(self) -> EpisodeRecord {
        EpisodeRecord { record_schema_id:RECORD_SCHEMA_ID.into(),job:self.job,
            catalog_descriptor:(*self.descriptor).clone(),content_id:CONTENT_ID.into(),
            schema_id:OBSERVATION_SCHEMA_ID.into(),schema_fingerprint:observation_spec().fingerprint(),
            seed_adapter_id:SEED_ADAPTER_ID.into(),rng_contract_id:RNG_CONTRACT_ID.into(),
            actions:self.actions,status:self.status,attempted_action:self.attempted_action,
            fault:self.fault }
    }
}

fn validate(record:&EpisodeRecord) -> Result<(),EnvError> {
    let invalid=||input(InputReason::Scenario,"incompatible reproduction record");
    if record.record_schema_id!=RECORD_SCHEMA_ID || record.content_id!=CONTENT_ID
        || record.schema_id!=OBSERVATION_SCHEMA_ID
        || record.schema_fingerprint!=observation_spec().fingerprint()
        || record.seed_adapter_id!=SEED_ADAPTER_ID || record.rng_contract_id!=RNG_CONTRACT_ID {
        return Err(invalid());
    }
    let mut hash=Sha256::new();hash.update(b"spire-lab/env-catalog/v1\0");
    hash.update(canonical_json(&record.catalog_descriptor)?);
    let fingerprint:[u8;32]=hash.finalize().into();
    if fingerprint!=record.job.catalog_fingerprint() {return Err(invalid());}
    // Compare the exact normalized selected template, not merely its ID or a
    // claimed catalog hash. Job internals remain immutable and cannot be imported.
    let selected=ScenarioCatalog::new(vec![record.job.template().clone()])?.descriptor();
    let templates=record.catalog_descriptor.get("templates").and_then(Value::as_array).ok_or_else(invalid)?;
    if record.catalog_descriptor.get("schema_id").and_then(Value::as_str)!=Some("spire-lab-scenario-catalog-v1")
        || !templates.iter().any(|t|t==&selected["templates"][0]) {return Err(invalid());}
    if record.actions.iter().chain(record.attempted_action.iter()).any(|&a|!(0..32).contains(&a)) {
        return Err(input(InputReason::IllegalAction,"recorded candidate index out of range"));
    }
    if (record.status==RecordStatus::Faulted)!=record.fault.is_some()
        || (record.attempted_action.is_some() && record.status!=RecordStatus::Faulted) {
        return Err(invalid());
    }
    Ok(())
}

pub(crate) fn fresh_slot(job:&EpisodeJob) -> Result<Slot,EnvError> {
    Slot::build(crate::episode::PreparedEpisode { job:job.clone(),
        sequence:crate::episode::EpisodeSequence::default(),effective_seed:None })
}

/// Re-execute the accepted prefix through the normal manual slot/menu transition.
/// External failures (allocation, packaging, injected faults) are not recreated by
/// a record. `fault` reports only a failure actually encountered in this execution.
pub fn replay(record:&EpisodeRecord) -> Result<ReplayResult,EnvError> {
    replay_inner(record, |_|{})
}
fn replay_inner(record:&EpisodeRecord, prepare:impl FnOnce(&mut Slot)) -> Result<ReplayResult,EnvError> {
    replay_visiting(record,prepare,None)
}
fn replay_visiting(record:&EpisodeRecord,prepare:impl FnOnce(&mut Slot),
    mut visitor:Option<&mut dyn FnMut(ReplayVisit)->Result<(),String>>)->Result<ReplayResult,EnvError> {
    validate(record)?;
    let mut slot=fresh_slot(&record.job)?;
    prepare(&mut slot);
    let count=record.actions.len().checked_add(1).ok_or_else(||setup("replay length overflow"))?;
    let mut boundaries=reserved(count)?;
    let l=slot.live.as_mut().expect("fresh episode");
    boundaries.push(boundary(l));
    if let Some(v)=visitor.as_mut() {v(ReplayVisit {index:0,kind:ReplayVisitKind::Initial,action:None,
        public:l.combat.public_view().ok(),effects:vec![],boundary:Some(boundary(l))}).map_err(|e|setup(format!("replay visitor: {e}")))?;}
    for (offset,&index) in record.actions.iter().enumerate() {
        if l.info.outcome!=0 {return Err(input(InputReason::IllegalAction,"record continues beyond terminal boundary"));}
        let action=l.menu.resolve(index)?.clone();
        let effects=if visitor.is_some() {
            let observed=l.advance_observed(&action);observed.result?;observed.effects
        } else {l.advance(&action)?;Vec::new()};
        boundaries.push(boundary(l));
        if let Some(v)=visitor.as_mut() {v(ReplayVisit {index:offset+1,kind:ReplayVisitKind::AcceptedAction,
            action:Some(action.core),public:l.combat.public_view().ok(),effects,boundary:Some(boundary(l))})
            .map_err(|e|setup(format!("replay visitor: {e}")))?;}
    }
    if (record.status==RecordStatus::Completed && l.info.outcome==0)
        || (matches!(record.status,RecordStatus::Active|RecordStatus::Abandoned) && l.info.outcome!=0) {
        return Err(input(InputReason::Scenario,"record status disagrees with reproduced prefix"));
    }
    let mut fault=None;
    if let Some(index)=record.attempted_action {
        if l.info.outcome!=0 {return Err(input(InputReason::IllegalAction,"fault attempt after terminal boundary"));}
        let action=l.menu.resolve(index)?.clone();
        #[cfg(test)]
        let injected=slot.injection.as_ref().map_or(Ok(()),|i|i.at(crate::FaultStage::Resolution));
        #[cfg(not(test))]
        let injected:Result<(),EnvError>=Ok(());
        let mut effects=Vec::new();
        let attempted=injected.and_then(|_| {
            if visitor.is_some() {let observed=l.advance_observed(&action);effects=observed.effects;observed.result}
            else {l.advance(&action)}
        });
        match attempted {
            Err(EnvError::Fault { records })=> {
                fault=records.into_iter().next();
                if let Some(v)=visitor.as_mut() {v(ReplayVisit {index:record.actions.len()+1,kind:ReplayVisitKind::FaultAttempt,
                    action:Some(action.core),public:l.combat.public_view().ok(),effects,boundary:None})
                    .map_err(|e|setup(format!("replay visitor: {e}")))?;}
            },
            Err(e)=>return Err(e),
            // A completed attempted action is not part of the recorded prefix.
            // Do not fabricate the original external failure or silently include
            // this extra boundary/snapshot in a prefix reproduction.
            Ok(())=>return Err(input(InputReason::Scenario,"recorded fault attempt did not fault during replay")),
        }
    }
    Ok(ReplayResult { boundaries,final_snapshot:l.combat.snapshot(),fault })
}
fn boundary(l:&crate::slot::LiveEpisode) -> ReplayBoundary {
    ReplayBoundary { observation:l.row.clone(),info:l.info.clone(),
        reward:match l.info.outcome {1=>1.0,2=>-1.0,_=>0.0} }
}

#[cfg(test)]
pub(crate) fn replay_injected(record:&EpisodeRecord, injection:crate::tests::workers_faults::Injection)
    -> Result<ReplayResult,EnvError> {
    replay_inner(record,|slot|slot.injection=Some(injection))
}
