//! Explicit privileged owned snapshots, never produced in the normal step loop.
use spire_core::combat::Snapshot;
use crate::{PoolStatus, SlotFault, SlotStatus};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PoolDiagnostics {
    pub status:PoolStatus,
    pub faults:Vec<SlotFault>,
    pub slots:Vec<SlotDiagnostic>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SlotDiagnostic {
    pub slot:usize,
    pub status:SlotStatus,
    pub snapshot:Option<Snapshot>,
    pub episode_id:Option<String>,
    pub next_ordinal:Option<u64>,
    pub token:[u64;2],
}
