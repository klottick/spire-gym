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
/// M36 (design §3.5, spire-dmm M5): `Weak` handles per worker count plus one strong hot slot, the most recently used
/// pool. A pool lives while a search holds it or while it is hot, so residency is the live searches' counts plus one.
struct SharedPools { handles: Vec<(usize,std::sync::Weak<rayon::ThreadPool>)>, hot: Option<std::sync::Arc<rayon::ThreadPool>> }
static SHARED_POOLS: std::sync::Mutex<SharedPools> = std::sync::Mutex::new(SharedPools { handles: Vec::new(), hot: None });
impl Scheduler {
    pub(crate) fn new(workers:usize, batch:usize) -> Result<Self,EnvError> {
        let effective=workers.min(batch);
        if effective==1 { return Ok(Self::Serial); }
        build_pool(effective).map(|pool|Self::Parallel(std::sync::Arc::new(pool)))
    }
    /// M35 (design §3.6) shared pools for callers created per decision or nested (`PublicSearch::step_many`); `workers <= 1`
    /// touches no pool. M36: a live pool of that count is reused, otherwise a new one is built; dead handles are pruned
    /// and the returned pool becomes hot (this replaces M35 P13's per-count residency). Batches keep their own pool.
    pub(crate) fn shared(workers:usize) -> Result<Self,EnvError> {
        if workers<=1 { return Ok(Self::Serial); }
        let mut pools=SHARED_POOLS.lock().unwrap_or_else(|p|p.into_inner());
        pools.handles.retain(|(_,pool)|pool.strong_count()>0);
        let pool=match pools.handles.iter().find_map(|(n,pool)|if *n==workers { pool.upgrade() } else { None }) {
            Some(pool) => pool,
            None => {
                let pool=std::sync::Arc::new(build_pool(workers)?);
                pools.handles.try_reserve(1).map_err(|_|setup("worker pool registry allocation failed"))?;
                pools.handles.push((workers,std::sync::Arc::downgrade(&pool)));
                pool
            }
        };
        pools.hot=Some(pool.clone());
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
    /// M38b fix 4: [`Self::map_mut`] with a home range per worker. With `w` workers and `n` slots, worker `i` first runs
    /// the slots `i*n/w .. (i+1)*n/w` in order, so a slot's Run and session state normally stay with one worker thread
    /// between calls; a worker that finishes its range then takes the still unclaimed slots of the other ranges, so one
    /// long job (a combat setup, an autoreset) does not hold up the call. Every job runs exactly once, results keep slot
    /// order and never depend on which worker ran them, and a panic escaping `f` reaches the caller after every worker
    /// returns, as with `map_mut`.
    pub(crate) fn map_mut_sticky<T:Send,J:Send,R:Send>(&self, values:&mut [T], jobs:Vec<J>,
        f:impl Fn(usize,&mut T,J)->R+Send+Sync) -> Result<Vec<R>,EnvError> {
        use std::sync::{Mutex,atomic::{AtomicUsize,Ordering}};
        let Self::Parallel(pool) = self else { return self.map_mut(values, jobs, f) };
        assert_eq!(values.len(),jobs.len());
        let count=values.len();
        let workers=pool.current_num_threads().clamp(1,count.max(1));
        let mut cells=reserved(count)?;
        cells.extend(values.iter_mut().zip(jobs).map(|(value,job)|Mutex::new((value,Some(job),None::<R>))));
        let bound=|range:usize|(range*count/workers,(range+1)*count/workers);
        let next=(0..workers).map(|range|AtomicUsize::new(bound(range).0)).collect::<Vec<_>>();
        pool.broadcast(|ctx| {
            let home=ctx.index();
            if home>=workers { return }
            for offset in 0..workers {
                let range=(home+offset)%workers;
                loop {
                    let slot=next[range].fetch_add(1,Ordering::Relaxed);
                    if slot>=bound(range).1 { break }
                    let mut cell=cells[slot].lock().unwrap_or_else(|p|p.into_inner());
                    let (value,job,out)=&mut *cell;
                    let job=job.take().expect("each slot is claimed once");
                    *out=Some(f(slot,value,job));
                }
            }
        });
        let mut results=reserved(count)?;
        for cell in cells { results.push(cell.into_inner().unwrap_or_else(|p|p.into_inner()).2.expect("every slot ran")); }
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
/// TEST-ONLY (M36): whether the registry holds a live pool for `workers` (read only for counts a test owns).
#[cfg(test)]
pub(crate) fn registry_contains_for_test(workers:usize) -> bool {
    SHARED_POOLS.lock().unwrap_or_else(|p|p.into_inner()).handles.iter().any(|(n,pool)|*n==workers && pool.strong_count()>0)
}
