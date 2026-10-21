//! Checked transport metadata. Never part of a gameplay seed or episode summary.
use std::sync::atomic::{AtomicU64, Ordering};
use crate::EnvError;
use crate::error::setup;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DecisionToken {
    pub pool_instance_id: u64,
    pub live_decision_serial: u64,
}
impl From<DecisionToken> for [u64;2] {
    fn from(t: DecisionToken) -> Self { [t.pool_instance_id,t.live_decision_serial] }
}
impl From<[u64;2]> for DecisionToken {
    fn from(t: [u64;2]) -> Self { Self { pool_instance_id:t[0],live_decision_serial:t[1] } }
}

pub(crate) struct PoolIdAllocator { next: AtomicU64 }
impl PoolIdAllocator {
    pub(crate) const fn new(first: u64) -> Self { Self { next:AtomicU64::new(first) } }
    pub(crate) fn allocate(&self) -> Result<u64, EnvError> {
        let mut current = self.next.load(Ordering::Relaxed);
        loop {
            if current == 0 { return Err(setup("pool instance identity exhausted")); }
            match self.next.compare_exchange_weak(current,current.checked_add(1).unwrap_or(0),
                Ordering::Relaxed,Ordering::Relaxed) {
                Ok(id) => return Ok(id),
                Err(actual) => current = actual,
            }
        }
    }
}
static POOL_IDS: PoolIdAllocator = PoolIdAllocator::new(1);
pub(crate) fn pool_id() -> Result<u64, EnvError> { POOL_IDS.allocate() }

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CheckedCounter { next: Option<u64> }
impl CheckedCounter {
    pub(crate) fn new(first: u64) -> Self { Self { next:Some(first) } }
    pub(crate) fn next(&self) -> Option<u64> { self.next }
    pub(crate) fn check(&self, count: usize) -> Result<(), EnvError> {
        if count == 0 { return Ok(()); }
        let offset = u64::try_from(count-1).map_err(|_| setup("decision serial capacity"))?;
        self.next().and_then(|n| n.checked_add(offset)).ok_or_else(|| setup("decision serial exhausted"))?;
        Ok(())
    }
    pub(crate) fn take(&mut self) -> Result<u64, EnvError> {
        self.check(1)?;
        let n = self.next.expect("checked counter");
        self.next = n.checked_add(1);
        Ok(n)
    }
    #[cfg(test)]
    pub(crate) fn exhausted() -> Self { Self { next:None } }
}
