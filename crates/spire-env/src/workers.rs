//! A pool-private persistent scheduler; effective-one is genuine direct iteration.
use rayon::prelude::*;
use crate::EnvError;
use crate::error::{reserved, setup};
use crate::slot::Slot;

/// M35 (design §3.1, P13): every worker runs on an 8 MiB stack, the main-thread default the scalar Python sessions run
/// on, so a deep resolution cannot overflow only in a batch.
pub(crate) const WORKER_STACK_BYTES: usize = 8 * 1024 * 1024;
pub(crate) enum Scheduler { Serial, Parallel(std::sync::Arc<rayon::ThreadPool>) }
fn build_pool(threads:usize) -> Result<rayon::ThreadPool,EnvError> {
    rayon::ThreadPoolBuilder::new().num_threads(threads).stack_size(WORKER_STACK_BYTES).build()
        .map_err(|e|setup(format!("worker pool construction: {e}")))
}
static SHARED_POOLS: std::sync::Mutex<Vec<(usize,std::sync::Arc<rayon::ThreadPool>)>> = std::sync::Mutex::new(Vec::new());
impl Scheduler {
    pub(crate) fn new(workers:usize, batch:usize) -> Result<Self,EnvError> {
        let effective=workers.min(batch);
        if effective==1 { return Ok(Self::Serial); }
        build_pool(effective).map(|pool|Self::Parallel(std::sync::Arc::new(pool)))
    }
    /// M35 (design §3.6, P13): one lazily created process-wide pool per worker count, for callers created per decision or
    /// nested (`PublicSearch::step_many`); `workers <= 1` touches no pool. Batches keep their own private pool.
    pub(crate) fn shared(workers:usize) -> Result<Self,EnvError> {
        if workers<=1 { return Ok(Self::Serial); }
        let mut pools=SHARED_POOLS.lock().unwrap_or_else(|p|p.into_inner());
        if let Some((_,pool))=pools.iter().find(|(n,_)|*n==workers) { return Ok(Self::Parallel(pool.clone())); }
        let pool=std::sync::Arc::new(build_pool(workers)?);
        pools.try_reserve(1).map_err(|_|setup("worker pool registry allocation failed"))?;
        pools.push((workers,pool.clone()));
        Ok(Self::Parallel(pool))
    }
    pub(crate) fn map<T:Send,R:Send>(&self, jobs:Vec<T>, f:impl Fn(T)->R+Send+Sync)
        -> Result<Vec<R>,EnvError> {
        let mut results=reserved(jobs.len())?;
        match self {
            Self::Serial => results.extend(jobs.into_iter().map(f)),
            Self::Parallel(pool) => pool.install(|| jobs.into_par_iter().map(f).collect_into_vec(&mut results)),
        }
        Ok(results)
    }
    /// Map owned jobs over disjoint mutable values, retaining those values in
    /// their original ordered slice. Result storage is reserved before dispatch.
    pub(crate) fn map_mut<T:Send,J:Send,R:Send>(&self, values:&mut [T], jobs:Vec<J>,
        f:impl Fn(usize,&mut T,J)->R+Send+Sync) -> Result<Vec<R>,EnvError> {
        assert_eq!(values.len(),jobs.len());
        let mut results=reserved(jobs.len())?;
        match self {
            Self::Serial => results.extend(values.iter_mut().zip(jobs).enumerate()
                .map(|(i,(value,job))|f(i,value,job))),
            Self::Parallel(pool) => pool.install(|| values.par_iter_mut().zip(jobs.into_par_iter())
                .enumerate().map(|(i,(value,job))|f(i,value,job)).collect_into_vec(&mut results)),
        }
        Ok(results)
    }
    pub(crate) fn slots<T:Send,R:Send>(&self, slots:&mut [Slot], jobs:Vec<T>,
        f:impl Fn(usize,&mut Slot,T)->R+Send+Sync) -> Result<Vec<R>,EnvError> {
        assert_eq!(slots.len(),jobs.len());
        let mut results=reserved(jobs.len())?;
        match self {
            Self::Serial => results.extend(slots.iter_mut().zip(jobs).enumerate().map(|(i,(s,j))|f(i,s,j))),
            Self::Parallel(pool) => pool.install(|| slots.par_iter_mut().zip(jobs.into_par_iter())
                .enumerate().map(|(i,(s,j))|f(i,s,j)).collect_into_vec(&mut results)),
        }
        Ok(results)
    }
}
