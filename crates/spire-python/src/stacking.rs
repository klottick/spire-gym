//! Pure batch packaging helpers (no PyO3): stacking per-slot observations into `[B, ...]` columns, and the per-slot
//! status fields of `BatchFaultError.slots`. Compiled without the extension feature so `cargo test -p spire-python`
//! covers them.
use std::{collections::HashMap, mem::MaybeUninit, sync::{Mutex, atomic::{AtomicBool, AtomicUsize, Ordering}}};

use spire_env::{BatchFaultStage, ObservationChunk, ObservationTarget, RunObservation, RunSessionError, SlotDelivery, SlotReport};

/// Dense tensor buffers used by the batch packager.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Column { I64(Vec<i64>), I8(Vec<i8>) }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DType { I64, I32, I16, I8 }
impl DType {
    fn bytes(self) -> usize { match self { Self::I64 => 8, Self::I32 => 4, Self::I16 => 2, Self::I8 => 1 } }
}
pub(crate) type DTypeMap = HashMap<String, DType>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Buffer { I64(Vec<i64>), I32(Vec<i32>), I16(Vec<i16>), I8(Vec<i8>) }
impl Buffer {
    fn dtype(&self) -> DType { match self { Self::I64(_) => DType::I64, Self::I32(_) => DType::I32, Self::I16(_) => DType::I16, Self::I8(_) => DType::I8 } }
    fn capacity_bytes(&self) -> usize { match self { Self::I64(v) => v.capacity() * 8, Self::I32(v) => v.capacity() * 4, Self::I16(v) => v.capacity() * 2, Self::I8(v) => v.capacity() } }
    fn capacity(&self) -> usize { match self { Self::I64(v) => v.capacity(), Self::I32(v) => v.capacity(), Self::I16(v) => v.capacity(), Self::I8(v) => v.capacity() } }
    fn clear(&mut self) { match self { Self::I64(v) => v.clear(), Self::I32(v) => v.clear(), Self::I16(v) => v.clear(), Self::I8(v) => v.clear() } }
    fn set_len(&mut self, len: usize) { match self {
        Self::I64(v) => set_initialized_len(v, len), Self::I32(v) => set_initialized_len(v, len),
        Self::I16(v) => set_initialized_len(v, len), Self::I8(v) => set_initialized_len(v, len),
    } }
}
fn set_initialized_len<T>(values: &mut Vec<T>, len: usize) {
    assert!(values.is_empty(), "output buffers must have length zero before writing");
    assert!(values.capacity() >= len, "output buffer capacity must cover its initialized length");
    // SAFETY: This helper is called only after every element in `0..len` has been written. The callers
    // (`stack_slots_with_dtypes` and `Slab::finish`) verify successful copy completion and initialized native rows;
    // copy paths write exact-sized chunks. Buffers are reserved to at least `len` or taken with exact capacity above.
    unsafe { values.set_len(len) }
}

/// One stacked tensor: the scalar observation dict's key, with a leading batch dimension.
/// Dense tensors are stored in `column`; compact value tensors are stored in `typed`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Stacked { pub key: String, pub shape: Vec<usize>, pub column: Column, pub typed: Option<Buffer> }
#[derive(Clone)]
struct Tensor { key: String, shape: Vec<usize>, dtype: DType, width: usize }

/// The stacking plan of `count` rows laid out like `first`: key, shape, dtype and width per tensor.
fn tensor_plan(first: &RunObservation, count: usize, dtypes: &DTypeMap) -> Vec<Tensor> {
    let mut plan = Vec::new();
    for group in &first.groups {
        let mut shape = Vec::with_capacity(group.shape.len() + 1);
        shape.push(count);
        shape.extend_from_slice(&group.shape);
        plan.push(Tensor { key: group.key.to_owned(), shape: shape.clone(), dtype: dtypes.get(group.key).copied().unwrap_or(DType::I64), width: group.values.len() });
        plan.push(Tensor { key: format!("{}_known", group.key), shape: shape.clone(), dtype: DType::I8, width: group.known.len() });
        if let Some(occupied) = &group.occupied { plan.push(Tensor { key: format!("{}_occupied", group.key), shape: vec![count, occupied.len()], dtype: DType::I8, width: occupied.len() }); }
    }
    plan.push(Tensor { key: "action_mask".to_owned(), shape: vec![count, first.action_mask.len()], dtype: DType::I8, width: first.action_mask.len() });
    plan
}
fn reserve_buffer(dtype: DType, len: usize) -> Result<Buffer, &'static str> {
    macro_rules! reserve { ($type:ty, $variant:ident) => {{
        let mut values = Vec::<$type>::new();
        values.try_reserve_exact(len).map_err(|_| "observation stacking allocation failed")?;
        Buffer::$variant(values)
    }} }
    Ok(match dtype { DType::I64 => reserve!(i64, I64), DType::I32 => reserve!(i32, I32), DType::I16 => reserve!(i16, I16), DType::I8 => reserve!(i8, I8) })
}
fn value_dtype(group: &spire_env::RunFeatureGroup, dtypes: &DTypeMap) -> DType { dtypes.get(group.key).copied().unwrap_or(DType::I64) }

/// Dense-compatible entry point retained for existing internal callers and tests.
pub(crate) fn stack(rows: &[&RunObservation]) -> Result<Vec<Stacked>, &'static str> { stack_with_dtypes(rows, &DTypeMap::new()) }

/// Stack rows that share one layout, in the scalar dict's key order (`<group>`, `<group>_known`, `<group>_occupied`
/// per group, then `action_mask`). One owned copy per tensor; value tensor dtypes come from the batch descriptor.
pub(crate) fn stack_with_dtypes(rows: &[&RunObservation], dtypes: &DTypeMap) -> Result<Vec<Stacked>, &'static str> {
    let first = rows.first().ok_or("empty observation batch")?;
    let count = rows.len();
    let plan = tensor_plan(first, count, dtypes);
    let mut buffers = plan.iter().map(|t| reserve_buffer(t.dtype, t.width * count)).collect::<Result<Vec<_>, _>>()?;
    for row in rows {
        if row.groups.len() != first.groups.len() { return Err("observation groups disagree") }
        if row.action_mask.len() != first.action_mask.len() { return Err("observation layouts disagree") }
        let mut index = 0;
        for (group_index, group) in first.groups.iter().enumerate() {
            let other = &row.groups[group_index];
            if other.key != group.key || other.shape != group.shape { return Err("observation layouts disagree") }
            let values = match value_dtype(group, dtypes) {
                DType::I64 => Buffer::I64(other.values.clone()), DType::I32 => Buffer::I32(narrow::<i32>(&other.values)?),
                DType::I16 => Buffer::I16(narrow::<i16>(&other.values)?), DType::I8 => return Err("invalid observation dtype"),
            };
            append(&mut buffers[index], values); index += 1;
            append(&mut buffers[index], Buffer::I8(other.known.clone())); index += 1;
            match (&group.occupied, &other.occupied) {
                (Some(expected), Some(actual)) if actual.len() == expected.len() => { append(&mut buffers[index], Buffer::I8(actual.clone())); index += 1; }
                (None, None) => {}
                _ => return Err("occupancy layouts disagree"),
            }
        }
        append(&mut buffers[index], Buffer::I8(row.action_mask.clone()));
    }
    Ok(plan.into_iter().zip(buffers).map(|(t, b)| stacked(t, b)).collect())
}
fn narrow<T: TryFrom<i64>>(values: &[i64]) -> Result<Vec<T>, &'static str> {
    values.iter().copied().map(T::try_from).collect::<Result<Vec<_>, _>>().map_err(|_| "observation narrowing failed")
}
fn append(destination: &mut Buffer, source: Buffer) {
    match (destination, source) {
        (Buffer::I64(a), Buffer::I64(mut b)) => a.append(&mut b), (Buffer::I32(a), Buffer::I32(mut b)) => a.append(&mut b),
        (Buffer::I16(a), Buffer::I16(mut b)) => a.append(&mut b), (Buffer::I8(a), Buffer::I8(mut b)) => a.append(&mut b),
        _ => unreachable!("dtype plan"),
    }
}
fn stacked(t: Tensor, buffer: Buffer) -> Stacked {
    let (column, typed) = match buffer {
        Buffer::I64(v) => (Column::I64(v), None), Buffer::I8(v) => (Column::I8(v), None),
        buffer @ Buffer::I32(_) | buffer @ Buffer::I16(_) => (Column::I64(Vec::new()), Some(buffer)),
    };
    Stacked { key: t.key, shape: t.shape, column, typed }
}

/// One slot's observation for [`stack_slots`]: owned (the copying job drops it, so the frees also run in parallel) or a
/// shared row (the zero template).
pub(crate) enum SlotSource<'a> { Owned(RunObservation), Shared(&'a RunObservation) }
impl SlotSource<'_> { fn get(&self) -> &RunObservation { match self { Self::Owned(o) => o, Self::Shared(o) => o } } }
enum ValueChunk<'a> { I64(&'a mut [MaybeUninit<i64>]), I32(&'a mut [MaybeUninit<i32>]), I16(&'a mut [MaybeUninit<i16>]) }

/// One slot's share of the stacked columns: its destination chunk in every tensor, in [`stack`]'s tensor order.
pub(crate) struct SlotCopy<'a> {
    source: SlotSource<'a>, done: &'a AtomicUsize, failed: &'a AtomicBool,
    values: Vec<ValueChunk<'a>>, narrow: Vec<&'a mut [MaybeUninit<i8>]>,
}
impl SlotCopy<'_> {
    /// Copy this slot's rows into its chunks (lengths were checked by [`stack_slots`] or [`Slab::finish`]), then drop
    /// an owned source.
    pub(crate) fn run(self) {
        fn write<T: Copy>(target: &mut [MaybeUninit<T>], source: &[T]) {
            assert_eq!(target.len(), source.len(), "validated stacking chunk");
            for (t, s) in target.iter_mut().zip(source) { t.write(*s); }
        }
        fn write_checked<T: Copy + TryFrom<i64>>(target: &mut [MaybeUninit<T>], source: &[i64], failed: &AtomicBool) {
            assert_eq!(target.len(), source.len(), "validated stacking chunk");
            for (t, s) in target.iter_mut().zip(source) {
                let value = match T::try_from(*s) {
                    Ok(value) => value,
                    Err(_) => { failed.store(true, Ordering::Release); T::try_from(0_i64).ok().expect("zero fits") }
                };
                t.write(value);
            }
        }
        let Self { source, done, failed, mut values, mut narrow } = self;
        let observation = source.get();
        let (mut value_chunks, mut narrow_chunks) = (values.iter_mut(), narrow.iter_mut());
        for group in &observation.groups {
            match value_chunks.next().expect("value plan") {
                ValueChunk::I64(target) => write(target, &group.values),
                ValueChunk::I32(target) => write_checked(target, &group.values, failed),
                ValueChunk::I16(target) => write_checked(target, &group.values, failed),
            }
            write(narrow_chunks.next().expect("known plan"), &group.known);
            if let Some(occupied) = &group.occupied { write(narrow_chunks.next().expect("occupied plan"), occupied); }
        }
        write(narrow_chunks.next().expect("mask plan"), &observation.action_mask);
        done.fetch_add(1, Ordering::Release);
    }
}

/// M37b: [`stack`]'s tensors (same keys, shapes, order and dense bytes) with the per-slot copies handed to `run`, which
/// the batch transport runs on the batch's workers outside the GIL. Layouts are checked first, in [`stack`]'s order
/// and with its messages; rows whose value or known lengths differ from the first row's are also rejected
/// ("observation layouts disagree"), where [`stack`] would build a column that does not match its shape.
#[cfg(test)]
pub(crate) fn stack_slots<E>(rows: Vec<SlotSource<'_>>, run: impl FnOnce(Vec<SlotCopy<'_>>) -> Result<(), E>)
    -> Result<Vec<Stacked>, &'static str> { stack_slots_with_dtypes(rows, &DTypeMap::new(), run) }

/// Stack rows using the descriptor's value dtypes, preserving the dense [`stack_slots`] layout and ordering.
pub(crate) fn stack_slots_with_dtypes<E>(rows: Vec<SlotSource<'_>>, dtypes: &DTypeMap,
    run: impl FnOnce(Vec<SlotCopy<'_>>) -> Result<(), E>) -> Result<Vec<Stacked>, &'static str> {
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
        return Err("observation layouts disagree");
    }
    let plan = tensor_plan(first, count, dtypes);
    let mut buffers = plan.iter().map(|t| reserve_buffer(t.dtype, t.width * count)).collect::<Result<Vec<_>, _>>()?;
    let mut slots = (0..count).map(|_| (Vec::new(), Vec::new())).collect::<SlotChunks<'_>>();
    let failed = AtomicBool::new(false);
    let done = AtomicUsize::new(0);
    for (tensor, buffer) in plan.iter().zip(buffers.iter_mut()) {
        let slot_chunks = match buffer {
            Buffer::I64(v) => split(&mut v.spare_capacity_mut()[..tensor.width * count], tensor.width, count).into_iter().map(ValueChunk::I64).collect::<Vec<_>>(),
            Buffer::I32(v) => split(&mut v.spare_capacity_mut()[..tensor.width * count], tensor.width, count).into_iter().map(ValueChunk::I32).collect(),
            Buffer::I16(v) => split(&mut v.spare_capacity_mut()[..tensor.width * count], tensor.width, count).into_iter().map(ValueChunk::I16).collect(),
            Buffer::I8(v) => {
                let chunks = split(&mut v.spare_capacity_mut()[..tensor.width * count], tensor.width, count);
                for (slot, chunk) in slots.iter_mut().zip(chunks) { slot.1.push(chunk); }
                Vec::new()
            }
        };
        if !slot_chunks.is_empty() { for (slot, chunk) in slots.iter_mut().zip(slot_chunks) { slot.0.push(chunk); } }
    }
    let jobs = rows.into_iter().zip(slots).map(|(source, (values, narrow))| SlotCopy {
        source, done: &done, failed: &failed, values, narrow,
    }).collect();
    run(jobs).map_err(|_| "observation stacking allocation failed")?;
    if failed.load(Ordering::Acquire) { return Err("observation narrowing failed") }
    if done.load(Ordering::Acquire) != count { return Err("observation stacking did not copy every slot") }
    for (tensor, buffer) in plan.iter().zip(buffers.iter_mut()) { buffer.set_len(tensor.width * count); }
    Ok(plan.into_iter().zip(buffers).map(|(t, b)| stacked(t, b)).collect())
}

/// Every slot's value and narrow chunks of the columns, in slot order.
type SlotChunks<'a> = Vec<(Vec<ValueChunk<'a>>, Vec<&'a mut [MaybeUninit<i8>]>)>;
fn matches_plan(observation: &RunObservation, plan: &[Tensor], count: usize) -> bool {
    if observation.groups.len() * 2 + observation.groups.iter().filter(|g| g.occupied.is_some()).count() + 1 != plan.len() { return false }
    let mut tensors = plan.iter();
    for group in &observation.groups {
        let mut shape = Vec::with_capacity(group.shape.len() + 1);
        shape.push(count);
        shape.extend_from_slice(&group.shape);
        let Some(values) = tensors.next() else { return false };
        if values.key != group.key || values.shape != shape || values.width != group.values.len() { return false }
        let Some(known) = tensors.next() else { return false };
        if known.key != format!("{}_known", group.key) || known.shape != shape || known.width != group.known.len() { return false }
        if let Some(occupied) = &group.occupied {
            let Some(tensor) = tensors.next() else { return false };
            if tensor.key != format!("{}_occupied", group.key) || tensor.shape != [count, occupied.len()] || tensor.width != occupied.len() { return false }
        }
    }
    let Some(mask) = tensors.next() else { return false };
    mask.key == "action_mask" && mask.shape == [count, observation.action_mask.len()]
        && mask.width == observation.action_mask.len() && tensors.next().is_none()
}

#[derive(Default)]
struct PoolState { buffers: Vec<Buffer>, bytes: usize, limit: usize }
/// M38b fix 3(a): a bounded pool of column buffers. The binding returns a stacked `obs` column here when NumPy releases
/// the last array that uses it, and the next step's [`Slab`] takes it instead of allocating. Only memory no live array
/// can reach is reused, and every reused element is written again before it is read, so outputs are unchanged. The byte
/// bound accounts for each buffer's dtype width.
pub(crate) struct ColumnPool { state: Mutex<PoolState> }
impl Default for ColumnPool { fn default() -> Self { Self { state: Mutex::new(PoolState::default()) } } }
impl ColumnPool {
    fn lock(&self) -> std::sync::MutexGuard<'_, PoolState> { self.state.lock().unwrap_or_else(|p| p.into_inner()) }
    #[cfg(test)]
    pub(crate) fn put(&self, column: Column) {
        let buffer = match column { Column::I64(v) => Buffer::I64(v), Column::I8(v) => Buffer::I8(v) };
        self.put_buffer(buffer);
    }
    /// Keep a returned buffer if it fits the bound, else free it.
    pub(crate) fn put_buffer(&self, buffer: Buffer) {
        let mut state = self.lock();
        let bytes = buffer.capacity_bytes();
        if bytes == 0 || state.bytes.saturating_add(bytes) > state.limit { return }
        state.bytes += bytes;
        state.buffers.push(buffer);
    }
    /// Free every pooled buffer (batch close).
    pub(crate) fn clear(&self) { let mut state = self.lock(); *state = PoolState { limit: state.limit, ..PoolState::default() }; }
    /// Raise the bound to `bytes` (two calls' columns, so a caller that keeps one call's arrays while the next runs
    /// recycles steadily).
    fn reserve_limit(&self, bytes: usize) { let mut state = self.lock(); state.limit = state.limit.max(bytes); }
    fn take(&self, dtype: DType, len: usize) -> Result<Buffer, &'static str> {
        let mut state = self.lock();
        match state.buffers.iter().position(|b| b.dtype() == dtype && b.capacity() == len) {
            Some(index) => { let mut b = state.buffers.swap_remove(index); state.bytes -= b.capacity_bytes(); b.clear(); Ok(b) }
            None => { drop(state); reserve_buffer(dtype, len) }
        }
    }
    #[cfg(test)]
    pub(crate) fn pooled_bytes(&self) -> usize { self.lock().bytes }
}

/// M38b fix 1: the stacked `obs` columns of one step, allocated before the step (from the pool where it can), so each
/// Standard slot is encoded straight into its chunks ([`ObservationTarget`]) on the batch's workers. Rows that were
/// not encoded into their chunks (faults, isolated slots) are copied in afterwards, as [`stack_slots`] copies every row.
/// Same tensors, keys, shapes and order as [`stack`]; dense tensors retain the same bytes.
pub(crate) struct Slab<'p> {
    plan: Vec<Tensor>, count: usize, buffers: Vec<Buffer>, initialized: Vec<AtomicBool>, pool: &'p ColumnPool,
}
impl<'p> Slab<'p> {
    /// Columns for `count` rows laid out like `layout` (the batch's zero template).
    #[cfg(test)]
    pub(crate) fn new(layout: &RunObservation, count: usize, pool: &'p ColumnPool) -> Result<Self, &'static str> {
        Self::new_with_dtypes(layout, count, pool, &DTypeMap::new())
    }
    /// Columns with value dtypes from the batch's observation descriptor.
    pub(crate) fn new_with_dtypes(layout: &RunObservation, count: usize, pool: &'p ColumnPool, dtypes: &DTypeMap) -> Result<Self, &'static str> {
        if count == 0 { return Err("empty observation batch") }
        let plan = tensor_plan(layout, count, dtypes);
        let limit = 2 * plan.iter().map(|t| t.width * count * t.dtype.bytes()).sum::<usize>();
        pool.reserve_limit(limit);
        let buffers = plan.iter().map(|t| pool.take(t.dtype, t.width * count)).collect::<Result<Vec<_>, _>>()?;
        Ok(Self { plan, count, buffers, initialized: (0..count).map(|_| AtomicBool::new(false)).collect(), pool })
    }
    /// One target per slot, for `SessionBatch::step_into`.
    pub(crate) fn targets(&mut self) -> Vec<ObservationTarget<'_>> {
        let mut values = (0..self.count).map(|_| Vec::new()).collect::<Vec<Vec<ObservationChunk<'_>>>>();
        let mut narrow = (0..self.count).map(|_| Vec::new()).collect::<Vec<Vec<&mut [MaybeUninit<i8>]>>>();
        for (t, b) in self.plan.iter().zip(self.buffers.iter_mut()) {
            match b {
                Buffer::I64(v) => for (slot, chunk) in values.iter_mut().zip(split(&mut v.spare_capacity_mut()[..t.width * self.count], t.width, self.count)) { slot.push(ObservationChunk::I64(chunk)); },
                Buffer::I32(v) => for (slot, chunk) in values.iter_mut().zip(split(&mut v.spare_capacity_mut()[..t.width * self.count], t.width, self.count)) { slot.push(ObservationChunk::I32(chunk)); },
                Buffer::I16(v) => for (slot, chunk) in values.iter_mut().zip(split(&mut v.spare_capacity_mut()[..t.width * self.count], t.width, self.count)) { slot.push(ObservationChunk::I16(chunk)); },
                Buffer::I8(v) => for (slot, chunk) in narrow.iter_mut().zip(split(&mut v.spare_capacity_mut()[..t.width * self.count], t.width, self.count)) { slot.push(chunk); },
            }
        }
        values.into_iter().zip(narrow).zip(&self.initialized)
            .map(|((v, n), initialized)| ObservationTarget::typed(v, n, initialized)).collect()
    }
    /// The finished tensors. `fill[slot]` is `None` for a row encoded into its chunks (its target must be initialized)
    /// and the row's observation otherwise; those rows are copied by `run` (on the batch's workers).
    pub(crate) fn finish<E>(mut self, fill: Vec<Option<SlotSource<'_>>>, run: impl FnOnce(Vec<SlotCopy<'_>>) -> Result<(), E>)
        -> Result<Vec<Stacked>, &'static str> {
        if fill.len() != self.count { return Err("observation stacking did not copy every slot") }
        for (source, initialized) in fill.iter().zip(&self.initialized) {
            match source {
                None if !initialized.load(Ordering::Acquire) => return Err("observation stacking did not copy every slot"),
                None => {}
                Some(source) if !matches_plan(source.get(), &self.plan, self.count) => return Err("observation layouts disagree"),
                Some(_) => {}
            }
        }
        let mut slots = (0..self.count).map(|_| (Vec::new(), Vec::new())).collect::<SlotChunks<'_>>();
        for (t, b) in self.plan.iter().zip(self.buffers.iter_mut()) {
            match b {
                Buffer::I64(v) => for (slot, chunk) in slots.iter_mut().zip(split(&mut v.spare_capacity_mut()[..t.width * self.count], t.width, self.count)) { slot.0.push(ValueChunk::I64(chunk)); },
                Buffer::I32(v) => for (slot, chunk) in slots.iter_mut().zip(split(&mut v.spare_capacity_mut()[..t.width * self.count], t.width, self.count)) { slot.0.push(ValueChunk::I32(chunk)); },
                Buffer::I16(v) => for (slot, chunk) in slots.iter_mut().zip(split(&mut v.spare_capacity_mut()[..t.width * self.count], t.width, self.count)) { slot.0.push(ValueChunk::I16(chunk)); },
                Buffer::I8(v) => for (slot, chunk) in slots.iter_mut().zip(split(&mut v.spare_capacity_mut()[..t.width * self.count], t.width, self.count)) { slot.1.push(chunk); },
            }
        }
        let failed = AtomicBool::new(false);
        let done = AtomicUsize::new(0);
        let jobs = fill.into_iter().zip(slots).filter_map(|(source, (values, narrow))| source.map(|source| SlotCopy {
            source, done: &done, failed: &failed, values, narrow,
        })).collect::<Vec<_>>();
        let copies = jobs.len();
        if copies > 0 { run(jobs).map_err(|_| "observation stacking allocation failed")?; }
        if failed.load(Ordering::Acquire) { return Err("observation narrowing failed") }
        if done.load(Ordering::Acquire) != copies { return Err("observation stacking did not copy every slot") }
        for (t, b) in self.plan.iter().zip(self.buffers.iter_mut()) { b.set_len(t.width * self.count); }
        Ok(self.plan.iter().cloned().zip(std::mem::take(&mut self.buffers)).map(|(t, b)| stacked(t, b)).collect())
    }
}
impl Drop for Slab<'_> {
    /// Unfinished columns (a failed step or packaging) go back to the pool.
    fn drop(&mut self) { for b in self.buffers.drain(..) { self.pool.put_buffer(b); } }
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
        SlotDelivery::Faulted { stage, committed, .. } => (Some(stage.label()), Some(committed), Some(committed && stage == BatchFaultStage::Autoreset)),
        SlotDelivery::Delivered | SlotDelivery::Unaffected => (None, None, None),
    };
    SlotStatus { status: report.delivery.label(), stage, committed, recoverable, message }
}
pub(crate) fn fault_cause_record(error: &RunSessionError) -> spire_env::RunRecordError {
    let mut record = spire_env::RunRecordError::from_error(error);
    if matches!(record.kind, spire_env::RunRecordErrorKind::Core) {
        record.message = "native Run fault; inspect privileged reproduction".to_owned();
        record.capacity = None;
    }
    record
}

#[cfg(test)]
mod m38e_tests {
    use super::*;

    #[test]
    fn typed_value_columns_and_narrowing_are_checked() {
        let row = RunObservation {
            groups: vec![spire_env::RunFeatureGroup {
                key: "player",
                shape: vec![1],
                values: vec![123],
                known: vec![1],
                occupied: None,
            }],
            action_mask: vec![1],
        };
        let mut dtypes = DTypeMap::new();
        dtypes.insert("player".into(), DType::I16);
        let columns = stack_with_dtypes(&[&row], &dtypes).unwrap();
        assert!(matches!(&columns[0].typed, Some(Buffer::I16(values)) if values == &vec![123]));

        let mut bad = row.clone();
        bad.groups[0].values[0] = i64::from(i16::MAX) + 1;
        assert_eq!(
            stack_with_dtypes(&[&bad], &dtypes).unwrap_err(),
            "observation narrowing failed"
        );
    }

    #[test]
    fn typed_slot_copy_writes_compact_columns_and_pool_is_bounded() {
        let row = RunObservation {
            groups: vec![spire_env::RunFeatureGroup {
                key: "player",
                shape: vec![1],
                values: vec![9],
                known: vec![1],
                occupied: None,
            }],
            action_mask: vec![1],
        };
        let mut dtypes = DTypeMap::new();
        dtypes.insert("player".into(), DType::I32);
        fn serial(jobs: Vec<SlotCopy<'_>>) -> Result<(), ()> {
            jobs.into_iter().for_each(SlotCopy::run);
            Ok(())
        }
        let columns = stack_slots_with_dtypes(vec![SlotSource::Owned(row.clone())], &dtypes, serial).unwrap();
        assert!(matches!(&columns[0].typed, Some(Buffer::I32(values)) if values == &vec![9]));
        // A copied row whose value does not fit the column dtype fails; it is never written as a silent 0.
        let mut bad = row.clone();
        bad.groups[0].values[0] = i64::from(i32::MAX) + 1;
        assert_eq!(stack_slots_with_dtypes(vec![SlotSource::Owned(bad)], &dtypes, serial).err(), Some("observation narrowing failed"));

        let pool = ColumnPool::default();
        pool.reserve_limit(16);
        let mut values = Vec::<i16>::with_capacity(8);
        values.push(1);
        pool.put_buffer(Buffer::I16(values));
        assert_eq!(pool.pooled_bytes(), 16);
        let _ = pool.take(DType::I16, 8).unwrap();
        assert_eq!(pool.pooled_bytes(), 0);
        pool.put_buffer(Buffer::I32(Vec::with_capacity(8)));
        assert_eq!(pool.pooled_bytes(), 0);
        pool.clear();
        assert_eq!(pool.pooled_bytes(), 0);
    }

    #[test]
    fn slab_rejects_short_fallback_rows_before_setting_buffer_lengths() {
        let layout = RunObservation {
            groups: vec![spire_env::RunFeatureGroup {
                key: "player",
                shape: vec![1],
                values: vec![9],
                known: vec![1],
                occupied: None,
            }],
            action_mask: vec![1],
        };
        let mut short = layout.clone();
        short.groups[0].values.clear();
        let pool = ColumnPool::default();
        let slab = Slab::new(&layout, 1, &pool).unwrap();
        fn serial(jobs: Vec<SlotCopy<'_>>) -> Result<(), ()> {
            jobs.into_iter().for_each(SlotCopy::run);
            Ok(())
        }

        assert_eq!(
            slab.finish(vec![Some(SlotSource::Owned(short))], serial).unwrap_err(),
            "observation layouts disagree"
        );
        assert_eq!(pool.pooled_bytes(), 10);
    }

    #[test]
    fn python_fault_cause_redacts_core_but_retains_other_causes() {
        let core = RunSessionError::Core(spire_core::run::RunError::Faulted(
            spire_core::run::RunFault {
                checkpoint: "test",
                kind: spire_core::run::RunFaultKind::Invariant("private"),
                committed_events: vec![],
            },
        ));
        let redacted = fault_cause_record(&core);
        assert_eq!(
            redacted.message,
            "native Run fault; inspect privileged reproduction"
        );
        assert!(redacted.capacity.is_none());

        let capacity = RunSessionError::Capacity {
            group: "player",
            required: 4,
            capacity: 3,
        };
        let full = fault_cause_record(&capacity);
        assert_eq!(full.kind, spire_env::RunRecordErrorKind::Capacity);
        assert_eq!(full.capacity.unwrap().group, "player");
    }
}
