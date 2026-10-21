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

/// A slot's first fault. Pool-wide export faults use `slot: None`.
/// Coordinators preserve first records and sort Some(slot) ascending, then None.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SlotFault {
    pub slot: Option<usize>,
    pub stage: FaultStage,
    pub message: String,
    pub core_fault: Option<FaultRecord>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EnvError {
    Input { slot: Option<usize>, reason: InputReason, message: String },
    State { slot: Option<usize>, reason: StateReason },
    Setup { slot: Option<usize>, message: String },
    Fault { records: Vec<SlotFault> },
}

impl std::fmt::Display for EnvError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Input { slot, reason, message } => write!(f, "input {reason:?} at {slot:?}: {message}"),
            Self::State { slot, reason } => write!(f, "state {reason:?} at {slot:?}"),
            Self::Setup { slot, message } => write!(f, "setup at {slot:?}: {message}"),
            Self::Fault { records } => write!(f, "environment fault: {records:?}"),
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
    let mut values = Vec::new();
    values.try_reserve_exact(len).map_err(|_| setup("setup allocation exceeds available capacity"))?;
    Ok(values)
}
