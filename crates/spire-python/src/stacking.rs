//! Pure batch packaging helpers (no PyO3): stacking per-slot observations into `[B, ...]` columns, and the per-slot
//! status fields of `BatchFaultError.slots`. Compiled without the extension feature so `cargo test -p spire-python`
//! covers them.
use spire_env::{BatchFaultStage, RunObservation, RunSessionError, SlotDelivery, SlotReport};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Column { I64(Vec<i64>), I8(Vec<i8>) }
/// One stacked tensor: the scalar observation dict's key, with a leading batch dimension.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Stacked { pub key: String, pub shape: Vec<usize>, pub column: Column }

/// Stack rows that share one layout, in the scalar dict's key order (`<group>`, `<group>_known`, `<group>_occupied`
/// per group, then `action_mask`). One owned copy per tensor.
pub(crate) fn stack(rows: &[&RunObservation]) -> Result<Vec<Stacked>, &'static str> {
    let first = rows.first().ok_or("empty observation batch")?;
    let count = rows.len();
    let mut out = Vec::new();
    for (index, group) in first.groups.iter().enumerate() {
        let mut shape = Vec::with_capacity(group.shape.len() + 1);
        shape.push(count);
        shape.extend_from_slice(&group.shape);
        let mut values = Vec::new();
        let mut known = Vec::new();
        let mut occupied = group.occupied.as_ref().map(|_| Vec::new());
        values.try_reserve_exact(group.values.len() * count).map_err(|_| "observation stacking allocation failed")?;
        known.try_reserve_exact(group.known.len() * count).map_err(|_| "observation stacking allocation failed")?;
        for row in rows {
            let other = row.groups.get(index).ok_or("observation groups disagree")?;
            if other.key != group.key || other.shape != group.shape { return Err("observation layouts disagree") }
            values.extend_from_slice(&other.values);
            known.extend_from_slice(&other.known);
            match (&mut occupied, &other.occupied) {
                (Some(target), Some(source)) if source.len() == group.occupied.as_ref().map_or(0, Vec::len) => target.extend_from_slice(source),
                (None, None) => {}
                _ => return Err("occupancy layouts disagree"),
            }
        }
        out.push(Stacked { key: group.key.to_owned(), shape: shape.clone(), column: Column::I64(values) });
        out.push(Stacked { key: format!("{}_known", group.key), shape, column: Column::I8(known) });
        if let Some(occupied) = occupied {
            let width = group.occupied.as_ref().map_or(0, Vec::len);
            out.push(Stacked { key: format!("{}_occupied", group.key), shape: vec![count, width], column: Column::I8(occupied) });
        }
    }
    if rows.iter().any(|r| r.groups.len() != first.groups.len() || r.action_mask.len() != first.action_mask.len()) {
        return Err("observation layouts disagree")
    }
    let mut mask = Vec::new();
    mask.try_reserve_exact(first.action_mask.len() * count).map_err(|_| "observation stacking allocation failed")?;
    for row in rows { mask.extend_from_slice(&row.action_mask); }
    out.push(Stacked { key: "action_mask".to_owned(), shape: vec![count, first.action_mask.len()], column: Column::I8(mask) });
    Ok(out)
}

/// The Python-facing message of a session error (the scalar `RunEnv` texts; core faults stay redacted).
pub(crate) fn error_message(error: &RunSessionError, combat: bool) -> String {
    match error {
        RunSessionError::Input(s) | RunSessionError::UnsupportedBoundary(s) | RunSessionError::DeliveryFault(s)
        | RunSessionError::IncompatibleReplay(s) | RunSessionError::Budget(s) => (*s).to_owned(),
        RunSessionError::StaleToken => "stale Run decision token".to_owned(),
        RunSessionError::NotReset => "Run has not been reset".to_owned(),
        RunSessionError::Terminal => "Run is terminal".to_owned(),
        RunSessionError::Closed => "Run is closed".to_owned(),
        RunSessionError::Capacity { group, required, capacity } => format!("Run capacity {group}: {required} > {capacity}"),
        RunSessionError::Core(_) if combat => "native combat fault; inspect privileged reproduction".to_owned(),
        RunSessionError::Core(_) => "native Run fault; inspect privileged reproduction".to_owned(),
        RunSessionError::BranchFaulted => "search branch is faulted".to_owned(),
    }
}

/// One `BatchFaultError.slots` entry (design §3.3): `status`, `stage`, `committed`, `recoverable`, `message`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SlotStatus {
    pub status: &'static str, pub stage: Option<&'static str>, pub committed: Option<bool>, pub recoverable: Option<bool>,
    pub message: Option<String>,
}
pub(crate) fn slot_status(report: &SlotReport, combat: bool) -> SlotStatus {
    let message = report.error.as_ref().map(|e| error_message(e, combat));
    let (stage, committed, recoverable) = match report.delivery {
        SlotDelivery::CommittedUndelivered { recoverable } => (None, Some(true), Some(recoverable)),
        SlotDelivery::NotCommitted => (None, Some(false), None),
        SlotDelivery::Faulted { stage, committed, .. } =>
            (Some(stage.label()), Some(committed), Some(committed && stage == BatchFaultStage::Autoreset)),
        SlotDelivery::Delivered | SlotDelivery::Unaffected => (None, None, None),
    };
    SlotStatus { status: report.delivery.label(), stage, committed, recoverable, message }
}
