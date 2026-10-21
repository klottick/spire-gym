//! A pool-private persistent scheduler; effective-one is genuine direct iteration.
use rayon::prelude::*;
use crate::EnvError;
use crate::error::{reserved, setup};
use crate::slot::Slot;

pub(crate) enum Scheduler { Serial, Parallel(rayon::ThreadPool) }
impl Scheduler {
    pub(crate) fn new(workers:usize, batch:usize) -> Result<Self,EnvError> {
        let effective=workers.min(batch);
        if effective==1 { return Ok(Self::Serial); }
        rayon::ThreadPoolBuilder::new().num_threads(effective).build()
            .map(Self::Parallel).map_err(|e|setup(format!("worker pool construction: {e}")))
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
