//! Pure batch packaging helpers (no PyO3): stacking per-slot observations into `[B, ...]` columns, and the per-slot
//! status fields of `BatchFaultError.slots`. Compiled without the extension feature so `cargo test -p spire-python`
//! covers them.
use std::{mem::MaybeUninit, sync::{Mutex, atomic::{AtomicBool, AtomicUsize, Ordering}}};

use spire_env::{BatchFaultStage, ObservationTarget, RunObservation, RunSessionError, SlotDelivery, SlotReport};

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

/// One slot's observation for [`stack_slots`]: owned (the copying job drops it, so the frees also run in parallel) or a
/// shared row (the zero template).
pub(crate) enum SlotSource<'a> { Owned(RunObservation), Shared(&'a RunObservation) }
impl SlotSource<'_> {
    fn get(&self) -> &RunObservation { match self { Self::Owned(o) => o, Self::Shared(o) => o } }
}
/// One slot's share of the stacked columns: its destination chunk in every tensor, in [`stack`]'s tensor order.
pub(crate) struct SlotCopy<'a> {
    source: SlotSource<'a>,
    done: &'a AtomicUsize,
    wide: Vec<&'a mut [MaybeUninit<i64>]>,
    narrow: Vec<&'a mut [MaybeUninit<i8>]>,
}
impl SlotCopy<'_> {
    /// Copy this slot's rows into its chunks (lengths were checked by [`stack_slots`]), then drop an owned source.
    pub(crate) fn run(self) {
        fn copy<T: Copy>(target: &mut [MaybeUninit<T>], source: &[T]) {
            assert_eq!(target.len(), source.len(), "validated stacking chunk");
            for (t, s) in target.iter_mut().zip(source) { t.write(*s); }
        }
        let SlotCopy { source, done, mut wide, mut narrow } = self;
        let o = source.get();
        let (mut w, mut n) = (wide.iter_mut(), narrow.iter_mut());
        for g in &o.groups {
            copy(w.next().expect("validated stacking plan"), &g.values);
            copy(n.next().expect("validated stacking plan"), &g.known);
            if let Some(occupied) = &g.occupied { copy(n.next().expect("validated stacking plan"), occupied); }
        }
        copy(n.next().expect("validated stacking plan"), &o.action_mask);
        done.fetch_add(1, Ordering::Release);
    }
}

/// M37b: [`stack`]'s tensors (same keys, shapes, order and bytes) with the per-slot copies handed to `run`, which the
/// batch transport runs on the batch's workers outside the GIL. Layouts are checked first, in [`stack`]'s order and
/// with its messages; rows whose value or known lengths differ from the first row's are also rejected ("observation
/// layouts disagree"), where [`stack`] would build a column that does not match its shape.
pub(crate) fn stack_slots<E>(rows: Vec<SlotSource<'_>>, run: impl FnOnce(Vec<SlotCopy<'_>>) -> Result<(), E>)
    -> Result<Vec<Stacked>, &'static str> {
    let first = rows.first().ok_or("empty observation batch")?.get();
    let count = rows.len();
    for (index, group) in first.groups.iter().enumerate() {
        for row in &rows {
            let other = row.get().groups.get(index).ok_or("observation groups disagree")?;
            if other.key != group.key || other.shape != group.shape { return Err("observation layouts disagree") }
            if other.values.len() != group.values.len() || other.known.len() != group.known.len() { return Err("observation layouts disagree") }
            match (&group.occupied, &other.occupied) {
                (Some(target), Some(source)) if source.len() == target.len() => {}
                (None, None) => {}
                _ => return Err("occupancy layouts disagree"),
            }
        }
    }
    if rows.iter().any(|r| r.get().groups.len() != first.groups.len() || r.get().action_mask.len() != first.action_mask.len()) {
        return Err("observation layouts disagree")
    }
    let plan = plan(first, count);
    let mut wide = plan.iter().filter(|t| t.2).map(|t| reserve::<i64>(t.3 * count)).collect::<Result<Vec<_>, _>>()?;
    let mut narrow = plan.iter().filter(|t| !t.2).map(|t| reserve::<i8>(t.3 * count)).collect::<Result<Vec<_>, _>>()?;
    let widths = |wide: bool| plan.iter().filter(move |t| t.2 == wide).map(|t| t.3).collect::<Vec<_>>();
    let (wide_widths, narrow_widths) = (widths(true), widths(false));
    let done = AtomicUsize::new(0);
    {
        let mut jobs = rows.into_iter().map(|source| SlotCopy { source, done: &done, wide: Vec::new(), narrow: Vec::new() }).collect::<Vec<_>>();
        for (column, width) in wide.iter_mut().zip(&wide_widths) {
            let chunks = &mut column.spare_capacity_mut()[..width * count];
            for (job, chunk) in jobs.iter_mut().zip(split(chunks, *width, count)) { job.wide.push(chunk); }
        }
        for (column, width) in narrow.iter_mut().zip(&narrow_widths) {
            let chunks = &mut column.spare_capacity_mut()[..width * count];
            for (job, chunk) in jobs.iter_mut().zip(split(chunks, *width, count)) { job.narrow.push(chunk); }
        }
        run(jobs).map_err(|_| "observation stacking allocation failed")?;
    }
    // Every job must have finished its copies before any length is set (a `run` that skips jobs is an error, not UB).
    if done.load(Ordering::Acquire) != count { return Err("observation stacking did not copy every slot") }
    for (column, width) in wide.iter_mut().zip(&wide_widths) {
        // SAFETY: all `count` jobs completed (`done` above), and each wrote its whole chunk; the chunks partition the
        // first `width * count` reserved elements. A panicking job unwinds before this line, leaving the length at zero.
        unsafe { column.set_len(width * count) }
    }
    for (column, width) in narrow.iter_mut().zip(&narrow_widths) {
        // SAFETY: as above.
        unsafe { column.set_len(width * count) }
    }
    let (mut wide, mut narrow) = (wide.into_iter(), narrow.into_iter());
    Ok(plan.into_iter().map(|(key, shape, is_wide, _)| Stacked { key, shape, column: if is_wide {
        Column::I64(wide.next().expect("one column per wide tensor"))
    } else {
        Column::I8(narrow.next().expect("one column per narrow tensor"))
    } }).collect())
}
/// The stacking plan of `count` rows laid out like `first`: (key, shape, wide, width) per tensor, i64 tensors `wide`.
fn plan(first: &RunObservation, count: usize) -> Vec<(String, Vec<usize>, bool, usize)> {
    let mut plan = Vec::new();
    for group in &first.groups {
        let mut shape = Vec::with_capacity(group.shape.len() + 1);
        shape.push(count);
        shape.extend_from_slice(&group.shape);
        plan.push((group.key.to_owned(), shape.clone(), true, group.values.len()));
        plan.push((format!("{}_known", group.key), shape, false, group.known.len()));
        if let Some(occupied) = &group.occupied { plan.push((format!("{}_occupied", group.key), vec![count, occupied.len()], false, occupied.len())); }
    }
    plan.push(("action_mask".to_owned(), vec![count, first.action_mask.len()], false, first.action_mask.len()));
    plan
}
fn reserve<T>(len: usize) -> Result<Vec<T>, &'static str> {
    let mut v = Vec::new();
    v.try_reserve_exact(len).map_err(|_| "observation stacking allocation failed")?;
    Ok(v)
}

/// M38b fix 3(a): a bounded pool of column buffers. The binding returns a stacked `obs` column here when NumPy releases
/// the last array that uses it, and the next step's [`Slab`] takes it instead of allocating. Only memory no live array
/// can reach is reused, and every reused element is written again before it is read, so outputs are unchanged.
pub(crate) struct ColumnPool { state: Mutex<PoolState> }
#[derive(Default)]
struct PoolState { wide: Vec<Vec<i64>>, narrow: Vec<Vec<i8>>, bytes: usize, limit: usize }
impl Default for ColumnPool { fn default() -> Self { Self { state: Mutex::new(PoolState::default()) } } }
impl ColumnPool {
    fn lock(&self) -> std::sync::MutexGuard<'_, PoolState> { self.state.lock().unwrap_or_else(|p| p.into_inner()) }
    /// Keep a returned buffer if it fits the bound, else free it.
    pub(crate) fn put(&self, column: Column) {
        let mut state = self.lock();
        let bytes = match &column { Column::I64(v) => v.capacity() * 8, Column::I8(v) => v.capacity() };
        if bytes == 0 || state.bytes + bytes > state.limit { return }
        state.bytes += bytes;
        match column { Column::I64(v) => state.wide.push(v), Column::I8(v) => state.narrow.push(v) }
    }
    /// Free every pooled buffer (batch close).
    pub(crate) fn clear(&self) { let mut state = self.lock(); *state = PoolState { limit: state.limit, ..PoolState::default() }; }
    /// Raise the bound to `bytes` (two calls' columns, so a caller that keeps one call's arrays while the next runs
    /// recycles steadily).
    fn reserve_limit(&self, bytes: usize) { let mut state = self.lock(); state.limit = state.limit.max(bytes); }
    fn take_wide(&self, len: usize) -> Result<Vec<i64>, &'static str> {
        let mut state = self.lock();
        match state.wide.iter().position(|v| v.capacity() == len) {
            Some(i) => { let mut v = state.wide.swap_remove(i); state.bytes -= len * 8; v.clear(); Ok(v) }
            None => { drop(state); reserve(len) }
        }
    }
    fn take_narrow(&self, len: usize) -> Result<Vec<i8>, &'static str> {
        let mut state = self.lock();
        match state.narrow.iter().position(|v| v.capacity() == len) {
            Some(i) => { let mut v = state.narrow.swap_remove(i); state.bytes -= len; v.clear(); Ok(v) }
            None => { drop(state); reserve(len) }
        }
    }
    #[cfg(test)]
    pub(crate) fn pooled_bytes(&self) -> usize { self.lock().bytes }
}

/// M38b fix 1: the stacked `obs` columns of one step, allocated before the step (from the pool where it can), so each
/// Standard slot is encoded straight into its chunks ([`ObservationTarget`]) on the batch's workers. Rows that were
/// not encoded into their chunks (faults, isolated slots) are copied in afterwards, as [`stack_slots`] copies every row.
/// Same tensors, keys, shapes, order and bytes as [`stack`].
pub(crate) struct Slab<'p> {
    plan: Vec<(String, Vec<usize>, bool, usize)>, count: usize, wide: Vec<Vec<i64>>, narrow: Vec<Vec<i8>>,
    initialized: Vec<AtomicBool>, pool: &'p ColumnPool,
}
impl<'p> Slab<'p> {
    /// Columns for `count` rows laid out like `layout` (the batch's zero template).
    pub(crate) fn new(layout: &RunObservation, count: usize, pool: &'p ColumnPool) -> Result<Self, &'static str> {
        if count == 0 { return Err("empty observation batch") }
        let plan = plan(layout, count);
        pool.reserve_limit(2 * count * plan.iter().map(|t| if t.2 { t.3 * 8 } else { t.3 }).sum::<usize>());
        let mut slab = Self { plan: Vec::new(), count, wide: Vec::new(), narrow: Vec::new(), initialized: Vec::new(), pool };
        for t in &plan {
            if t.2 { slab.wide.push(pool.take_wide(t.3 * count)?) } else { slab.narrow.push(pool.take_narrow(t.3 * count)?) }
        }
        slab.plan = plan;
        slab.initialized = (0..count).map(|_| AtomicBool::new(false)).collect();
        Ok(slab)
    }
    fn widths(&self, wide: bool) -> Vec<usize> { widths(&self.plan, wide) }
    /// One target per slot, for `SessionBatch::step_into`.
    pub(crate) fn targets(&mut self) -> Vec<ObservationTarget<'_>> {
        let (wide, narrow) = chunks(&self.plan, self.count, &mut self.wide, &mut self.narrow);
        wide.into_iter().zip(narrow).zip(&self.initialized).map(|((w, n), flag)| ObservationTarget::new(w, n, flag)).collect()
    }
    /// The finished tensors. `fill[slot]` is `None` for a row encoded into its chunks (its target must be initialized)
    /// and the row's observation otherwise; those rows are copied by `run` (on the batch's workers).
    pub(crate) fn finish<E>(mut self, fill: Vec<Option<SlotSource<'_>>>, run: impl FnOnce(Vec<SlotCopy<'_>>) -> Result<(), E>)
        -> Result<Vec<Stacked>, &'static str> {
        if fill.len() != self.count { return Err("observation stacking did not copy every slot") }
        for (source, flag) in fill.iter().zip(&self.initialized) {
            match source {
                None if !flag.load(Ordering::Acquire) => return Err("observation stacking did not copy every slot"),
                None => {}
                Some(source) => {
                    let o = source.get();
                    if o.groups.len() * 2 + o.groups.iter().filter(|g| g.occupied.is_some()).count() + 1 != self.plan.len() {
                        return Err("observation layouts disagree")
                    }
                    let mut tensors = self.plan.iter();
                    for g in &o.groups {
                        let mut next = |len: usize| tensors.next().is_some_and(|t| t.3 == len);
                        if !next(g.values.len()) || !next(g.known.len()) || g.occupied.as_ref().is_some_and(|occ| !next(occ.len())) {
                            return Err("observation layouts disagree")
                        }
                    }
                    if !tensors.next().is_some_and(|t| t.3 == o.action_mask.len()) { return Err("observation layouts disagree") }
                }
            }
        }
        let done = AtomicUsize::new(0);
        let copies = fill.iter().filter(|f| f.is_some()).count();
        {
            let (wide, narrow) = chunks(&self.plan, self.count, &mut self.wide, &mut self.narrow);
            let jobs = fill.into_iter().zip(wide.into_iter().zip(narrow)).filter_map(|(source, (wide, narrow))|
                source.map(|source| SlotCopy { source, done: &done, wide, narrow })).collect::<Vec<_>>();
            if !jobs.is_empty() { run(jobs).map_err(|_| "observation stacking allocation failed")?; }
        }
        if done.load(Ordering::Acquire) != copies { return Err("observation stacking did not copy every slot") }
        let (wide_widths, narrow_widths) = (self.widths(true), self.widths(false));
        let count = self.count;
        for (column, width) in self.wide.iter_mut().zip(&wide_widths) {
            // SAFETY: every chunk of the first `width * count` elements was written: by the zero fill of an initialized
            // target (`None` rows, checked above) or by a completed copy job (`done`); the chunks partition them.
            unsafe { column.set_len(width * count) }
        }
        for (column, width) in self.narrow.iter_mut().zip(&narrow_widths) {
            // SAFETY: as above.
            unsafe { column.set_len(width * count) }
        }
        let (mut wide, mut narrow) = (std::mem::take(&mut self.wide).into_iter(), std::mem::take(&mut self.narrow).into_iter());
        Ok(std::mem::take(&mut self.plan).into_iter().map(|(key, shape, is_wide, _)| Stacked { key, shape, column: if is_wide {
            Column::I64(wide.next().expect("one column per wide tensor"))
        } else {
            Column::I8(narrow.next().expect("one column per narrow tensor"))
        } }).collect())
    }
}
fn widths(plan: &[(String, Vec<usize>, bool, usize)], wide: bool) -> Vec<usize> { plan.iter().filter(|t| t.2 == wide).map(|t| t.3).collect() }
/// Every slot's chunks of the columns, in slot order.
type Chunks<'a> = (Vec<Vec<&'a mut [MaybeUninit<i64>]>>, Vec<Vec<&'a mut [MaybeUninit<i8>]>>);
fn chunks<'a>(plan: &[(String, Vec<usize>, bool, usize)], count: usize, wide_columns: &'a mut [Vec<i64>], narrow_columns: &'a mut [Vec<i8>]) -> Chunks<'a> {
    let (wide_widths, narrow_widths) = (widths(plan, true), widths(plan, false));
    let mut wide = (0..count).map(|_| Vec::with_capacity(wide_widths.len())).collect::<Vec<_>>();
    let mut narrow = (0..count).map(|_| Vec::with_capacity(narrow_widths.len())).collect::<Vec<_>>();
    for (column, width) in wide_columns.iter_mut().zip(&wide_widths) {
        for (slot, chunk) in wide.iter_mut().zip(split(&mut column.spare_capacity_mut()[..width * count], *width, count)) { slot.push(chunk); }
    }
    for (column, width) in narrow_columns.iter_mut().zip(&narrow_widths) {
        for (slot, chunk) in narrow.iter_mut().zip(split(&mut column.spare_capacity_mut()[..width * count], *width, count)) { slot.push(chunk); }
    }
    (wide, narrow)
}
impl Drop for Slab<'_> {
    /// Unfinished columns (a failed step or packaging) go back to the pool.
    fn drop(&mut self) {
        for v in self.wide.drain(..) { self.pool.put(Column::I64(v)); }
        for v in self.narrow.drain(..) { self.pool.put(Column::I8(v)); }
    }
}

/// `count` consecutive chunks of `width` (zero-width tensors give empty chunks).
fn split<T>(chunks: &mut [T], width: usize, count: usize) -> Vec<&mut [T]> {
    if width == 0 { return (0..count).map(|_| <&mut [T]>::default()).collect() }
    chunks.chunks_exact_mut(width).collect()
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
