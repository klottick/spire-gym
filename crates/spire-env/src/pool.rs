//! Validate first, dispatch disjoint slots, quiesce, then publish in slot order.
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::{Arc, Mutex, MutexGuard, TryLockError};
use std::sync::atomic::{AtomicBool, Ordering};
use spire_core::combat::rng::Partition;
use crate::{DecisionToken, EnvError, FaultStage, InputReason, IntegerSeed, ObservationBatch,
    ObservationRow, ScenarioCatalog, SlotFault, SlotStatus, StateReason};
use crate::episode::{EpisodeSequence, PreparedEpisode};
use crate::error::{input, reserved, setup};
use crate::identity::{CheckedCounter, pool_id};
use crate::slot::Slot;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AutoReset { Manual, SameStep }
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PoolConfig {
    pub batch_size: usize,
    pub workers: usize,
    pub partition: Partition,
    pub autoreset: AutoReset,
    pub record_replay: bool,
}
impl Default for PoolConfig {
    fn default() -> Self { Self { batch_size:1,workers:1,partition:Partition::Train,
        autoreset:AutoReset::Manual,record_replay:false } }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PoolStatus { Uninitialized, Active, Faulted, Closed }
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ResetRequest { pub seed:Option<IntegerSeed>, pub scenario_id:Option<String> }
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EpisodeInfo {
    pub episode_id:String, pub scenario_id:String, pub episode_steps:u64, pub outcome:i32,
    pub hp_before_healing:Option<i32>, pub hp_after_cleanup:Option<i32>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DecisionRows {
    pub slot_indices:Vec<usize>, pub observations:ObservationBatch,
    pub tokens:Vec<[u64;2]>, pub infos:Vec<EpisodeInfo>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResetRows { pub decisions:DecisionRows, pub effective_seeds:Vec<Option<IntegerSeed>> }
#[derive(Clone, Debug, PartialEq)]
pub struct StepBatch {
    pub current:DecisionRows, pub rewards:Vec<f32>, pub terminated:Vec<bool>,
    pub truncated:Vec<bool>, pub finals:DecisionRows,
}

pub struct CombatPool {
    pub(crate) catalog:Arc<ScenarioCatalog>,
    pub(crate) config:PoolConfig,
    pub(crate) instance_id:u64,
    pub(crate) state:Mutex<PoolState>,
    busy:AtomicBool,
    record_descriptor:Option<Arc<serde_json::Value>>,
}
pub(crate) struct PoolState {
    pub(crate) scheduler:crate::workers::Scheduler,
    pub(crate) slots:Vec<Slot>,
    pub(crate) serials:CheckedCounter,
    pub(crate) status:PoolStatus,
    pub(crate) faults:Vec<SlotFault>,
    pub(crate) records:Option<Vec<Arc<crate::replay::Trace>>>,
}

/// Metadata-only exclusive operation lease. Keep alive through attached export;
/// no native state guard survives any of the methods below.
pub struct PoolOperation<'a> {
    pool:&'a CombatPool,
    // Only this native call's newly retired records belong to its delivery.
    // Lease release or the next native call discards this metadata, not records.
    delivery_start:Option<usize>,
}
impl PoolOperation<'_> {
    pub fn reset_batch(&mut self, indices:&[usize], requests:&[ResetRequest]) -> Result<ResetRows,EnvError> {
        self.delivery_start=None;
        let pool=self.pool;
        pool.reset_inner(indices,requests,false,&mut self.delivery_start,|sequence,request| sequence.prepare_reset(
            request.seed.as_ref(),request.scenario_id.as_deref(),&pool.catalog,pool.config.partition))
    }
    /// Vector partial reset: preflight unselected initialization, then pack all
    /// rows before installation without starting a second delivery transaction.
    pub fn reset_batch_return_all(&mut self, indices:&[usize], requests:&[ResetRequest]) -> Result<ResetRows,EnvError> {
        self.delivery_start=None;
        let pool=self.pool;
        pool.reset_inner(indices,requests,true,&mut self.delivery_start,|sequence,request| sequence.prepare_reset(
            request.seed.as_ref(),request.scenario_id.as_deref(),&pool.catalog,pool.config.partition))
    }
    pub fn step_batch(&mut self, indices:&[i64], tokens:&[[u64;2]]) -> Result<StepBatch,EnvError> {
        self.delivery_start=None;
        self.pool.step_inner(indices,tokens,&mut self.delivery_start)
    }
    pub fn read_rows(&mut self, indices:&[usize]) -> Result<DecisionRows,EnvError> {
        self.delivery_start=None;
        self.pool.read_inner(indices)
    }
    pub fn diagnostics(&mut self) -> Result<crate::PoolDiagnostics,EnvError> {
        self.delivery_start=None;
        self.pool.diagnostics_inner()
    }
    pub fn reproduction(&mut self, slot:usize) -> Result<Option<crate::EpisodeRecord>,EnvError> {
        self.delivery_start=None;
        self.pool.reproduction_inner(slot)
    }
    pub fn take_completed_records(&mut self) -> Result<Vec<crate::EpisodeRecord>,EnvError> {
        self.delivery_start=None;
        self.pool.take_records_inner()
    }
    pub fn close(&mut self) -> Result<(),EnvError> {
        self.delivery_start=None;
        self.pool.close_inner()
    }
    /// A post-execution export failure must never permit a repeated action.
    /// Retain the entire original diagnostic set if native work already faulted.
    pub fn mark_export_fault(&mut self, message:&str) {
        self.pool.mark_fault(FaultStage::Export,message,self.delivery_start);
    }
}
impl Drop for PoolOperation<'_> {
    fn drop(&mut self) {
        if std::thread::panicking() {
            self.pool.mark_fault(FaultStage::Panic,"operation lease unwound",self.delivery_start);
        }
        self.pool.busy.store(false,Ordering::Release);
    }
}

impl CombatPool {
    pub fn new(catalog:Arc<ScenarioCatalog>, config:PoolConfig) -> Result<Self,EnvError> {
        if config.batch_size == 0 || config.workers == 0 {
            return Err(input(InputReason::Range,"batch size and workers must be positive"));
        }
        let record_descriptor=config.record_replay.then(||Arc::new(catalog.descriptor()));
        let records=config.record_replay.then(Vec::new);
        let mut slots = reserved(config.batch_size)?;
        slots.resize_with(config.batch_size,Slot::default);
        let scheduler=crate::workers::Scheduler::new(config.workers,config.batch_size)?;
        Ok(Self { catalog,config,record_descriptor,busy:AtomicBool::new(false),instance_id:pool_id()?,state:Mutex::new(PoolState {
            scheduler,slots,records,serials:CheckedCounter::new(1),status:PoolStatus::Uninitialized,faults:vec![] }) })
    }

    pub fn begin_operation(&self) -> Result<PoolOperation<'_>,EnvError> {
        self.busy.compare_exchange(false,true,Ordering::Acquire,Ordering::Relaxed)
            .map_err(|_|EnvError::State { slot:None,reason:StateReason::Busy })?;
        Ok(PoolOperation { pool:self,delivery_start:None })
    }

    // Operation-local state lock only; the atomic lease also covers owned export.
    pub(crate) fn lock(&self) -> Result<MutexGuard<'_,PoolState>,EnvError> {
        self.lock_for_delivery(None)
    }
    fn lock_for_delivery(&self, delivery_start:Option<usize>) -> Result<MutexGuard<'_,PoolState>,EnvError> {
        match self.state.try_lock() {
            Ok(s) => Ok(s),
            Err(TryLockError::WouldBlock) => Err(EnvError::State { slot:None,reason:StateReason::Busy }),
            Err(TryLockError::Poisoned(p)) => {
                let mut s = p.into_inner();
                if !matches!(s.status,PoolStatus::Closed|PoolStatus::Faulted) {
                    s.retain_fault(SlotFault { slot:None,stage:FaultStage::Panic,
                        message:"pool operation unwound while holding native state".into(),core_fault:None },delivery_start);
                }
                Ok(s)
            }
        }
    }

    pub fn reset_batch(&self, indices:&[usize], requests:&[ResetRequest]) -> Result<ResetRows,EnvError> {
        self.begin_operation()?.reset_batch(indices,requests)
    }

    // Setup preparation is separate from installation; the public entry point always
    // supplies the OS-backed EpisodeSequence implementation, never caller callbacks.
    #[cfg(test)]
    pub(crate) fn reset_with(&self, indices:&[usize], requests:&[ResetRequest],
        prepare:impl FnMut(&EpisodeSequence,&ResetRequest) -> Result<PreparedEpisode,EnvError>)
        -> Result<ResetRows,EnvError> {
        let mut operation=self.begin_operation()?;
        self.reset_inner(indices,requests,false,&mut operation.delivery_start,prepare)
    }
    fn reset_inner(&self, indices:&[usize], requests:&[ResetRequest],
        return_all:bool,
        delivery_start:&mut Option<usize>,
        mut prepare:impl FnMut(&EpisodeSequence,&ResetRequest) -> Result<PreparedEpisode,EnvError>)
        -> Result<ResetRows,EnvError> {
        let mut s = self.lock()?;
        s.check_open()?;
        if indices.len() != requests.len() { return Err(input(InputReason::Shape,"reset request count")); }
        s.validate_indices(indices,true)?;
        if return_all {
            for (index,slot) in s.slots.iter().enumerate() {
                if !indices.contains(&index) && slot.live.is_none() {
                    return Err(EnvError::State { slot:Some(index),reason:StateReason::Uninitialized });
                }
            }
        }
        s.serials.check(indices.len())?;
        // Pure staged jobs first: a malformed last request cannot install earlier rows.
        let mut prepared = reserved(indices.len())?;
        let mut effective_seeds = reserved(indices.len())?;
        for (&index,request) in indices.iter().zip(requests) {
            let p = prepare(&s.slots[index].sequence,request).map_err(|e| at_slot(e,index))?;
            effective_seeds.push(p.effective_seed.clone());
            prepared.push(p);
        }
        let mut staged = reserved(indices.len())?;
        let results=s.scheduler.map(prepared,|p|catch_unwind(AssertUnwindSafe(|| Slot::build(p))))?;
        // Every staged construction has quiesced before inspecting any error.
        let mut reset_error=None;
        for (&index,result) in indices.iter().zip(results) {
            match result {
                Ok(Ok(slot)) => staged.push(slot),
                Ok(Err(e @ EnvError::Fault { .. })) => { s.runtime_fault(index,FaultStage::Reset,e); },
                Ok(Err(e)) => { if reset_error.is_none() { reset_error=Some(at_slot(e,index)); } },
                Err(_) => { s.runtime_fault(index,FaultStage::Panic,setup("reset construction panicked")); },
            }
        }
        if s.status==PoolStatus::Faulted { return Err(EnvError::Fault { records:s.faults.clone() }); }
        if let Some(e)=reset_error { return Err(e); }
        // Allocate only in staged metadata; failed output packing cannot consume a serial.
        let mut serials = s.serials.clone();
        let mut order = reserved(indices.len())?;
        order.extend(0..indices.len()); order.sort_unstable_by_key(|&p|indices[p]);
        for pos in order {
            staged[pos].live.as_mut().expect("staged live episode").token = DecisionToken {
                pool_instance_id:self.instance_id,live_decision_serial:serials.take()? };
        }
        let decisions = if return_all {
            let mut rows = reserved(s.slots.len())?;
            rows.extend(s.slots.iter());
            for (&index,slot) in indices.iter().zip(&staged) { rows[index]=slot; }
            collect_rows(rows.into_iter().enumerate())?
        } else { collect_rows(indices.iter().copied().zip(staged.iter()))? };
        s.reserve_records(indices.len())?;
        *delivery_start=s.records.as_ref().map(Vec::len);
        for (&index,mut slot) in indices.iter().zip(staged) {
            if let Some(descriptor)=&self.record_descriptor {
                let live=slot.live.as_mut().expect("staged live");
                live.trace=Some(crate::replay::Trace::new(live.job.clone(),descriptor.clone()));
            }
            s.abandon(index);
            s.slots[index] = slot;
        }
        s.serials = serials;
        if !indices.is_empty() { s.status = PoolStatus::Active; }
        Ok(ResetRows { decisions,effective_seeds })
    }

    pub fn read_rows(&self, indices:&[usize]) -> Result<DecisionRows,EnvError> {
        self.begin_operation()?.read_rows(indices)
    }
    fn read_inner(&self, indices:&[usize]) -> Result<DecisionRows,EnvError> {
        let s = self.lock()?; s.check_open()?; s.validate_indices(indices,false)?;
        collect_rows(indices.iter().map(|&i|(i,&s.slots[i])))
    }

    pub fn step_batch(&self, indices:&[i64], tokens:&[[u64;2]]) -> Result<StepBatch,EnvError> {
        self.begin_operation()?.step_batch(indices,tokens)
    }
    fn step_inner(&self, indices:&[i64], tokens:&[[u64;2]], delivery_start:&mut Option<usize>) -> Result<StepBatch,EnvError> {
        let mut s = self.lock()?; s.check_open()?;
        let count = self.config.batch_size;
        if indices.len() != count || tokens.len() != count {
            return Err(input(InputReason::Shape,"step requires one action and token per slot"));
        }
        let mut actions = reserved(count)?;
        let mut replacements: Vec<Option<PreparedEpisode>> = reserved(count)?;
        // Check maximum publication capacity without consuming metadata or constructing combats.
        s.serials.check(count)?;
        for (i,slot) in s.slots.iter().enumerate() {
            match slot.status() {
                SlotStatus::Uninitialized => return Err(EnvError::State { slot:Some(i),reason:StateReason::Uninitialized }),
                SlotStatus::Terminal => return Err(EnvError::State { slot:Some(i),reason:StateReason::Terminal }),
                SlotStatus::Ready => (),
            }
            let live = slot.live.as_ref().expect("ready live episode");
            if <[u64;2]>::from(live.token) != tokens[i] {
                return Err(at_slot(input(InputReason::Token,"obsolete or foreign decision"),i));
            }
            actions.push(live.menu.resolve(indices[i]).map_err(|e| at_slot(e,i))?.clone());
            live.steps.checked_add(1).ok_or_else(|| at_slot(setup("accepted step count exhausted"),i))?;
            replacements.push(if self.config.autoreset == AutoReset::SameStep {
                Some(slot.sequence.prepare_reset(None,None,&self.catalog,self.config.partition).map_err(|e| at_slot(e,i))?)
            } else { None });
        }
        let mut rewards = reserved(count)?;
        let mut terminated = reserved(count)?;
        let mut truncated = reserved(count)?;
        let mut final_rows = reserved(count)?;
        let mut faults = reserved(count)?;
        // Finish all checked reservations before dispatch; then every slot reports,
        // including after a peer fault. Scheduler reserves its result vector first.
        let mut jobs=reserved(count)?;
        s.reserve_records(count.checked_mul(2).ok_or_else(||setup("record queue count overflow"))?)?;
        for slot in &mut s.slots {
            if let Some(trace)=slot.live.as_mut().and_then(|l|l.trace.as_mut()) {
                Arc::get_mut(trace).expect("ready trace is exclusively owned").reserve()?;
            }
        }
        *delivery_start=s.records.as_ref().map(Vec::len);
        jobs.extend(actions.into_iter().zip(replacements).zip(indices.iter().copied()));
        let PoolState { scheduler,slots,.. }=&mut *s;
        let results=scheduler.slots(slots,jobs,|i,slot,((action,replacement),index)| {
            #[cfg(test)]
            let injection=slot.injection.clone();
            let result = catch_unwind(AssertUnwindSafe(|| {
                if let Some(trace)=slot.live.as_mut().and_then(|l|l.trace.as_mut()) {
                    Arc::get_mut(trace).expect("ready trace").attempted_action=Some(index);
                }
                #[cfg(test)]
                if let Some(test)=&injection {test.enter();test.at(FaultStage::Resolution)?;}
                let live = slot.live.as_mut().expect("preflight ready");
                live.advance(&action)?;
                if let Some(trace)=&mut live.trace {
                    let trace=Arc::get_mut(trace).expect("ready trace");
                    trace.actions.push(index);trace.attempted_action=None;
                    if live.info.outcome!=0 {trace.status=crate::RecordStatus::Completed;}
                }
                if live.info.outcome!=0 {
                    slot.retired=live.trace.clone();live.trace_queued=live.trace.is_some();
                }
                #[cfg(test)]
                if let Some(test)=&injection {test.at(FaultStage::Encoding)?;}
                let outcome = live.info.outcome;
                let final_row=if outcome != 0 { Some((i,live.row.clone(),live.info.clone())) } else { None };
                if outcome != 0 {
                    if let Some(prepared) = replacement {
                        #[cfg(test)]
                        if let Some(test)=&injection {test.at(FaultStage::Reset)?;}
                        let mut next=Slot::build(prepared).map_err(|e| fault_error(FaultStage::Reset,e))?;
                        if let Some(descriptor)=&self.record_descriptor {
                            let live=next.live.as_mut().expect("replacement live");
                            live.trace=Some(crate::replay::Trace::new(live.job.clone(),descriptor.clone()));
                        }
                        next.retired=slot.retired.take();
                        *slot=next;
                    }
                }
                Ok::<_,EnvError>((outcome,final_row))
            }));
            #[cfg(test)]
            if let Some(test)=&injection {test.complete();}
            result
        })?;
        // Transfer all completed buffers after quiescence, even if a peer failed.
        for i in 0..count {
            if let Some(record)=s.slots[i].retired.take() {
                s.records.as_mut().expect("recording enabled").push(record);
            }
        }
        for (i,result) in results.into_iter().enumerate() {
            match result {
                Ok(Ok((outcome,final_row))) => {
                    rewards.push(match outcome { 1=>1.0,2=>-1.0,_=>0.0 });
                    terminated.push(outcome!=0);truncated.push(false);
                    if let Some(row)=final_row { final_rows.push(row); }
                },
                Ok(Err(e)) => faults.push(s.fault_record(i,FaultStage::Encoding,e)),
                Err(_) => faults.push(s.fault_record(i,FaultStage::Panic,setup("slot execution panicked"))),
            }
        }
        if !faults.is_empty() {
            for fault in faults { s.retain_fault(fault,None); }
            for i in 0..count {s.abandon(i);}
            return Err(EnvError::Fault { records:s.faults.clone() });
        }
        let PoolState { slots,serials,.. } = &mut *s;
        for slot in slots {
            if slot.status() == SlotStatus::Ready {
                slot.live.as_mut().expect("ready episode").token = DecisionToken {
                    pool_instance_id:self.instance_id,live_decision_serial:serials.take().expect("preflight serial capacity") };
            }
        }
        let export = (|| {
            let current = collect_rows(s.slots.iter().enumerate())?;
            let finals = pack_rows(final_rows.into_iter().map(|(i,row,info)|(i,row,[0,0],info)))?;
            Ok::<_,EnvError>(StepBatch { current,rewards,terminated,truncated,finals })
        })();
        match export {
            Ok(batch) => Ok(batch),
            Err(e) => {
                s.retain_fault(SlotFault { slot:None,stage:FaultStage::Export,message:e.to_string(),core_fault:None },*delivery_start);
                Err(EnvError::Fault { records:s.faults.clone() })
            }
        }
    }

    pub fn close(&self) -> Result<(),EnvError> {
        self.begin_operation()?.close()
    }
    fn close_inner(&self) -> Result<(),EnvError> {
        let mut s = self.lock()?;
        let count=s.slots.len();
        s.reserve_records(count)?;
        for i in 0..s.slots.len() { s.abandon(i); }
        s.slots.clear(); s.status = PoolStatus::Closed;
        s.scheduler=crate::workers::Scheduler::Serial;
        Ok(())
    }

    pub fn diagnostics(&self) -> Result<crate::PoolDiagnostics,EnvError> {
        self.begin_operation()?.diagnostics()
    }
    /// Explicit privileged copy; available after fault. No trace exists by default.
    pub fn reproduction(&self, slot:usize) -> Result<Option<crate::EpisodeRecord>,EnvError> {
        self.begin_operation()?.reproduction(slot)
    }
    fn reproduction_inner(&self, slot:usize) -> Result<Option<crate::EpisodeRecord>,EnvError> {
        let s=self.lock()?;s.validate_indices(&[slot],false)?;
        Ok(s.slots[slot].live.as_ref().and_then(|l|l.trace.as_ref()).map(|t|t.export()))
    }
    /// Drain the opt-in unbounded retired-record queue explicitly, including
    /// retained faulted evidence. Close preserves this queue for a final drain.
    pub fn take_completed_records(&self) -> Result<Vec<crate::EpisodeRecord>,EnvError> {
        self.begin_operation()?.take_completed_records()
    }
    fn take_records_inner(&self) -> Result<Vec<crate::EpisodeRecord>,EnvError> {
        let mut s=self.lock()?;
        let Some(records)=s.records.as_mut() else {return Ok(Vec::new());};
        let mut exported=reserved(records.len())?;
        exported.extend(records.drain(..).map(crate::replay::Trace::export_owned));
        Ok(exported)
    }
    fn diagnostics_inner(&self) -> Result<crate::PoolDiagnostics,EnvError> {
        let s=self.lock()?;
        let mut slots=reserved(s.slots.len())?;
        for (slot,value) in s.slots.iter().enumerate() {
            slots.push(crate::SlotDiagnostic { slot,status:value.status(),
                snapshot:value.live.as_ref().map(|l|l.combat.snapshot()),
                episode_id:value.live.as_ref().map(|l|l.job.episode_id()),
                next_ordinal:value.sequence.next_ordinal(),
                token:value.live.as_ref().map_or([0,0],|l|l.token.into()) });
        }
        Ok(crate::PoolDiagnostics { status:s.status,faults:s.faults.clone(),slots })
    }
    fn mark_fault(&self, stage:FaultStage, message:&str, delivery_start:Option<usize>) {
        // The lease guarantees exclusivity, including during attached export.
        if let Ok(mut s)=self.lock_for_delivery(delivery_start) {
            if !matches!(s.status,PoolStatus::Faulted|PoolStatus::Closed) {
                s.retain_fault(SlotFault { slot:None,stage,message:message.into(),core_fault:None },delivery_start);
            }
        }
    }
}

impl PoolState {
    fn reserve_records(&mut self, count:usize) -> Result<(),EnvError> {
        if let Some(records)=&mut self.records {
            records.try_reserve(count).map_err(|_|setup("completed record queue capacity"))?;
        }
        Ok(())
    }
    fn abandon(&mut self, index:usize) {
        if let Some(live)=&mut self.slots[index].live {
            let Some(trace)=&mut live.trace else {return;};
            if trace.status==crate::RecordStatus::Active {
                Arc::get_mut(trace).expect("active trace is exclusive").status=crate::RecordStatus::Abandoned;
                self.records.as_mut().expect("recording enabled").push(trace.clone());
                live.trace_queued=true;
            } else if trace.status==crate::RecordStatus::Faulted && !live.trace_queued {
                // Faulted prefixes stay retrievable even after explicit close.
                self.records.as_mut().expect("recording enabled").push(trace.clone());
                live.trace_queued=true;
            }
        }
    }
    fn check_open(&self) -> Result<(),EnvError> {
        match self.status {
            PoolStatus::Faulted => Err(EnvError::State { slot:None,reason:StateReason::Faulted }),
            PoolStatus::Closed => Err(EnvError::State { slot:None,reason:StateReason::Closed }),
            _ => Ok(()),
        }
    }
    fn validate_indices(&self, indices:&[usize], unique:bool) -> Result<(),EnvError> {
        let mut seen = reserved(indices.len())?;
        for &i in indices {
            if i >= self.slots.len() { return Err(at_slot(input(InputReason::Range,"slot index out of range"),i)); }
            seen.push(i);
        }
        seen.sort_unstable();
        if unique && seen.windows(2).any(|p|p[0] == p[1]) { return Err(input(InputReason::Range,"duplicate slot index")); }
        Ok(())
    }
    fn retain_fault(&mut self, fault:SlotFault, delivery_start:Option<usize>) {
        // A fault invalidates continuation of every installed episode. Keep the
        // per-slot first fault/attempt, and annotate delivery failures without
        // inventing another gameplay input. Update any shared queued owner too.
        for (index,slot) in self.slots.iter_mut().enumerate() {
            if let Some(trace)=slot.live.as_mut().and_then(|l|l.trace.as_mut()) {
                if trace.fault.is_none() && (fault.slot.is_none() || fault.slot==Some(index)) {
                    let old=trace.clone();
                    let value=Arc::make_mut(trace);
                    value.status=crate::RecordStatus::Faulted;value.fault=Some(fault.clone());
                    if let Some(records)=&mut self.records {
                        for r in records {if Arc::ptr_eq(r,&old) {*r=trace.clone();}}
                    }
                }
            }
        }
        if fault.slot.is_none() {
            if let (Some(records),Some(start))=(&mut self.records,delivery_start) {
                for trace in records.iter_mut().skip(start) {
                    if trace.fault.is_none() {
                        let value=Arc::make_mut(trace);
                        value.status=crate::RecordStatus::Faulted;value.fault=Some(fault.clone());
                    }
                }
            }
        }
        if !self.faults.iter().any(|f|f.slot == fault.slot) { self.faults.push(fault); }
        self.faults.sort_by_key(|f|(f.slot.is_none(),f.slot));
        self.status = PoolStatus::Faulted;
    }
    fn fault_record(&self, index:usize, stage:FaultStage, error:EnvError) -> SlotFault {
        let mut fault = match error {
            EnvError::Fault { records } => records.into_iter().next().unwrap_or(SlotFault {
                slot:Some(index),stage,message:"runtime failure without fault detail".into(),core_fault:None }),
            e => SlotFault { slot:Some(index),stage,message:e.to_string(),core_fault:None },
        };
        fault.slot = Some(index);
        if let Some(first) = self.slots[index].live.as_ref().and_then(|l|l.combat.fault_record()) {
            fault.core_fault = Some(first);
        }
        fault
    }
    fn runtime_fault(&mut self, index:usize, stage:FaultStage, error:EnvError) -> EnvError {
        let fault = self.fault_record(index,stage,error); self.retain_fault(fault,None);
        EnvError::Fault { records:self.faults.clone() }
    }
}

fn at_slot(error:EnvError, index:usize) -> EnvError {
    match error {
        EnvError::Input { reason,message,.. } => EnvError::Input { slot:Some(index),reason,message },
        EnvError::Setup { message,.. } => EnvError::Setup { slot:Some(index),message },
        EnvError::State { reason,.. } => EnvError::State { slot:Some(index),reason },
        e => e,
    }
}
fn fault_error(stage:FaultStage, error:EnvError) -> EnvError {
    match error {
        e @ EnvError::Fault { .. } => e,
        e => EnvError::Fault { records:vec![SlotFault { slot:None,stage,message:e.to_string(),core_fault:None }] },
    }
}
fn collect_rows<'a>(slots:impl ExactSizeIterator<Item=(usize,&'a Slot)>) -> Result<DecisionRows,EnvError> {
    let mut rows = reserved(slots.len())?;
    for (i,slot) in slots {
        let l = slot.live.as_ref().ok_or(EnvError::State { slot:Some(i),reason:StateReason::Uninitialized })?;
        rows.push((i,l.row.clone(),l.token.into(),l.info.clone()));
    }
    pack_rows(rows.into_iter())
}
fn pack_rows(rows:impl ExactSizeIterator<Item=(usize,ObservationRow,[u64;2],EpisodeInfo)>) -> Result<DecisionRows,EnvError> {
    let count = rows.len();
    let mut slot_indices = reserved(count)?; let mut observations = reserved(count)?;
    let mut tokens = reserved(count)?; let mut infos = reserved(count)?;
    for (i,row,token,info) in rows {
        slot_indices.push(i);observations.push(row);tokens.push(token);infos.push(info);
    }
    Ok(DecisionRows { slot_indices,observations:ObservationBatch::from_rows(&observations)?,tokens,infos })
}
