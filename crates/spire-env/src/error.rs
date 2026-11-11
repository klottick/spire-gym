//! Typed input/setup errors and retained runtime fault provenance.
use spire_core::combat::FaultRecord;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputReason {
    Shape, Dtype, Range, Scenario, Seed, Token, IllegalAction, Capacity,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StateReason { Uninitialized, Terminal, Busy, Closed, Faulted }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FaultStage { Resolution, Encoding, Reset, Panic, Export }
impl FaultStage {
    pub fn label(self) -> &'static str {
        match self { Self::Resolution => "resolution", Self::Encoding => "encoding", Self::Reset => "reset",
            Self::Panic => "panic", Self::Export => "export" }
    }
}

/// A slot's first fault. Pool-wide export faults use `slot: None`.
/// Coordinators preserve first records and sort Some(slot) ascending, then None.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SlotFault {
    pub slot: Option<usize>,
    pub stage: FaultStage,
    pub message: String,
    pub core_fault: Option<FaultRecord>,
}

/// Delivery of one combat-v1 pool slot when a whole batch operation faults.
/// The pool has no committed-result recovery, so committed peers remain informational only.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PoolSlotStatus {
    pub status: &'static str,
    pub stage: Option<FaultStage>,
    pub committed: Option<bool>,
    pub recoverable: Option<bool>,
    pub message: Option<String>,
}
impl PoolSlotStatus {
    pub(crate) fn faulted(fault: &SlotFault, committed: bool) -> Self {
        Self { status:"faulted", stage:Some(fault.stage), committed:Some(committed), recoverable:Some(false),
            message:Some(fault.message.clone()) }
    }
    pub(crate) fn committed_undelivered() -> Self {
        Self { status:"committed_undelivered", stage:None, committed:Some(true), recoverable:Some(false), message:None }
    }
    pub(crate) fn not_committed() -> Self {
        Self { status:"not_committed", stage:None, committed:Some(false), recoverable:None, message:None }
    }
    pub(crate) fn unaffected() -> Self {
        Self { status:"unaffected", stage:None, committed:None, recoverable:None, message:None }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EnvError {
    Input { slot: Option<usize>, reason: InputReason, message: String },
    State { slot: Option<usize>, reason: StateReason },
    Setup { slot: Option<usize>, message: String },
    Fault { records: Vec<SlotFault>, slots: Vec<PoolSlotStatus> },
}

impl std::fmt::Display for EnvError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Input { slot, reason, message } => write!(f, "input {reason:?} at {slot:?}: {message}"),
            Self::State { slot, reason } => write!(f, "state {reason:?} at {slot:?}"),
            Self::Setup { slot, message } => write!(f, "setup at {slot:?}: {message}"),
            Self::Fault { records, .. } => write!(f, "environment fault: {records:?}"),
        }
    }
}
impl std::error::Error for EnvError {}

pub(crate) fn input(reason: InputReason, message: impl Into<String>) -> EnvError {
    EnvError::Input { slot: None, reason, message: message.into() }
}
pub(crate) fn setup(message: impl Into<String>) -> EnvError {
    EnvError::Setup { slot: None, message: message.into() }
}

/// Check representable capacity and fallible preallocation before filling a buffer.
/// This does not claim recovery from process aborts or all allocator failures.
pub(crate) fn reserved<T>(len: usize) -> Result<Vec<T>, EnvError> {
    #[cfg(test)]
    if FAIL_RESERVED_AFTER.with(|n| match n.get() { Some(0) => { n.set(None); true }, Some(k) => { n.set(Some(k - 1)); false }, None => false }) {
        return Err(setup("setup allocation exceeds available capacity"));
    }
    let mut values = Vec::new();
    values.try_reserve_exact(len).map_err(|_| setup("setup allocation exceeds available capacity"))?;
    Ok(values)
}

// Private env unit-test support only: fail the Nth subsequent `reserved` call on
// the current thread with the ordinary setup error. No production failure backend.
#[cfg(test)]
thread_local!{pub(crate) static FAIL_RESERVED_AFTER:std::cell::Cell<Option<usize>>=const{std::cell::Cell::new(None)};}
