//! M34s-a public knowledge `spire-gym-public-knowledge-v1` (design §4.5): the exact draw-pile posterior (§4.5.1) and the
//! pre-roll consistent sets (§4.5.2). Both are functions of public history only (card classes and law values, never
//! handles or RNG), are cloned with the combat, and are never read by rules: no RNG draw, no gameplay state.
//!
//! The draw knowledge is maintained incrementally: every draw-pile path of the a0(ix) map calls [`note`] with its
//! operation. The tracker keeps the physical (instance, class) sequence of its last update only to check that each
//! operation explains the physical change and to re-key in-place class changes; that bookkeeping is not part of the
//! knowledge value, its equality or its digest. A change no operation explains (an unmapped path, a non-uniform in-place
//! change, an over-bound mixture) makes the knowledge `Unrepresentable` until the next shuffle or an empty pile.
//!
//! Normal form (R-M1): equal representations up to its rules compare and print equal; two structurally different
//! mixtures with the same distribution may stay unequal (a missed merge). Equality is sound: every normalisation rule
//! (merging identical components, gcd scaling, sorting, a single-class bag as known cards, a bag whose only non-excluded
//! class is `x` as `Known(x)` + the rest) preserves the distribution over class sequences, so equal normal forms always
//! mean equal distributions (`knowledge_equality_is_sound`).
//!
//! Pre-roll sets narrow only at published decision/selection boundaries (`engine::resolve_from`), on the published
//! intent value (M34s-b b0 D2), never at execution or at an unpublished mid-resolution refresh (P1).
use super::model::{OwnedCard, PublicKnowledge, State};
use super::view::CardClass;
use super::*;
use std::collections::BTreeMap;

/// a0(ix) lane 3 proposal (contract minimum 1,024): more components make the knowledge `Unrepresentable`.
pub const N_COMPONENTS: usize = 65_536;
/// M34s-b (final review I2): the retained heap of a knowledge ([`DrawKnowledge::heap_bytes`]) is capped; a larger knowledge
/// is `Unrepresentable` (fail closed), so a root or search branch never retains more. 2 MiB keeps the contract minimum of
/// 1,024 components for a 60-card pile (about 1.5 KiB per component).
pub const KNOWLEDGE_BYTES_CAP: usize = 2 * 1024 * 1024;
/// Per-allocation overhead counted by [`DrawKnowledge::heap_bytes`] (a conservative allocator header).
const ALLOCATION_OVERHEAD: usize = 16;

/// The draw knowledge exceeds the exact model (a bound, an unmapped path or a fail-closed removal; design §4.5.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Unrepresentable;

/// One bag class: `count` copies, `excluded` of them top-excluded (the bag's first card is not one of them).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct BagEntry { pub class: u16, pub count: u16, pub excluded: u16 }

/// A known card class, or a bag: a multiset in uniformly random order whose first card is not top-excluded.
/// Class indices refer to [`DrawKnowledge::classes`].
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Segment { Known(u16), Bag(Vec<BagEntry>) }

/// A top-to-bottom segment sequence with an exact weight (proportional to its probability).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Component { pub weight: u128, pub segments: Vec<Segment> }

/// The exact draw-pile posterior over class sequences, in normal form (review recheck R-M1): classes sorted and
/// all used; bag entries sorted by class; bags of one class or with one non-excluded class split into known
/// segments; identical components merged, weights reduced by their gcd, components sorted by segment sequence.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DrawKnowledge { classes: Vec<CardClass>, components: Vec<Component> }

impl DrawKnowledge {
    pub fn classes(&self) -> &[CardClass] { &self.classes }
    pub fn components(&self) -> &[Component] { &self.components }
    pub fn class(&self, index: u16) -> &CardClass { &self.classes[index as usize] }
    /// Sum of the component weights (checked to fit at every update).
    pub fn total_weight(&self) -> u128 { self.components.iter().map(|c| c.weight).sum() }
    /// Whether the class sequence `classes_top_to_bottom` has positive probability (the a6 fuzz support check).
    pub fn supports(&self, classes_top_to_bottom: &[CardClass]) -> bool {
        self.components.iter().any(|component| {
            let mut at = 0;
            component.segments.iter().all(|segment| match segment {
                Segment::Known(c) => { at += 1; classes_top_to_bottom.get(at - 1) == Some(self.class(*c)) }
                Segment::Bag(entries) => {
                    let size = bag_size(entries) as usize;
                    let Some(part) = classes_top_to_bottom.get(at..at + size) else { return false };
                    at += size;
                    let count = |e: &BagEntry| part.iter().filter(|c| *c == self.class(e.class)).count();
                    entries.iter().any(|e| self.class(e.class) == &part[0] && e.count > e.excluded)
                        && entries.iter().all(|e| count(e) == e.count as usize)
                }
            }) && at == classes_top_to_bottom.len()
        })
    }
    /// The heap this knowledge retains: every vector's capacity in bytes plus [`ALLOCATION_OVERHEAD`] per allocation
    /// (a function of the knowledge built from public operations; at most [`KNOWLEDGE_BYTES_CAP`]).
    pub fn heap_bytes(&self) -> usize {
        fn vec<T>(v: &Vec<T>) -> usize { if v.capacity() == 0 { 0 } else { v.capacity() * std::mem::size_of::<T>() + ALLOCATION_OVERHEAD } }
        vec(&self.classes) + vec(&self.components) + self.components.iter().map(|c| vec(&c.segments)
            + c.segments.iter().map(|s| match s { Segment::Known(_) => 0, Segment::Bag(e) => vec(e) }).sum::<usize>()).sum::<usize>()
    }
    fn empty() -> Self { Self { classes: vec![], components: vec![Component { weight: 1, segments: vec![] }] } }
    /// Every arrangement of `top_to_bottom` known (Frozen Eye, a full reveal).
    fn known(top_to_bottom: impl IntoIterator<Item = CardClass>) -> Result<Self, Unrepresentable> {
        let mut classes = vec![];
        let segments = top_to_bottom.into_iter().map(|class| Segment::Known(intern(&mut classes, class))).collect();
        let mut acc = Acc::default();
        acc.push(Ratio::ONE, segments)?;
        finish(classes, acc)
    }
    fn shuffled(classes_in: impl IntoIterator<Item = CardClass>) -> Result<Self, Unrepresentable> {
        let mut classes = vec![];
        let bag = bag_of(&mut classes, classes_in);
        let mut acc = Acc::default();
        acc.push(Ratio::ONE, vec![Segment::Bag(bag)])?;
        finish(classes, acc)
    }
    fn class_counts(&self) -> Option<BTreeMap<CardClass, usize>> {
        let mut seen: Option<BTreeMap<CardClass, usize>> = None;
        for component in &self.components {
            let mut counts = BTreeMap::new();
            for segment in &component.segments {
                match segment {
                    Segment::Known(c) => *counts.entry(self.classes[*c as usize]).or_insert(0) += 1,
                    Segment::Bag(entries) => for e in entries { *counts.entry(self.classes[e.class as usize]).or_insert(0) += e.count as usize },
                }
            }
            match &seen { Some(previous) if *previous != counts => return None, Some(_) => {}, None => seen = Some(counts) }
        }
        seen
    }
}

/// An exact non-negative rational used inside one update.
#[derive(Clone, Copy, Debug)]
struct Ratio { num: u128, den: u128 }
impl Ratio {
    const ONE: Self = Self { num: 1, den: 1 };
    fn times(self, num: u128, den: u128) -> Result<Self, Unrepresentable> {
        if den == 0 { return Err(Unrepresentable) }
        let (num, den) = (self.num.checked_mul(num).ok_or(Unrepresentable)?, self.den.checked_mul(den).ok_or(Unrepresentable)?);
        let g = gcd(num, den).max(1);
        Ok(Self { num: num / g, den: den / g })
    }
    fn plus(self, other: Self) -> Result<Self, Unrepresentable> {
        let g = gcd(self.den, other.den);
        let num = self.num.checked_mul(other.den / g).and_then(|a| other.num.checked_mul(self.den / g).and_then(|b| a.checked_add(b)));
        let den = (self.den / g).checked_mul(other.den).ok_or(Unrepresentable)?;
        let num = num.ok_or(Unrepresentable)?;
        let g = gcd(num, den).max(1);
        Ok(Self { num: num / g, den: den / g })
    }
}
fn gcd(mut a: u128, mut b: u128) -> u128 { while b != 0 { (a, b) = (b, a % b); } a }

/// The components of one update, merged as they are produced (segments in normal form) and bounded by `N_COMPONENTS`.
#[derive(Default)]
struct Acc { map: BTreeMap<Vec<Segment>, Ratio> }
impl Acc {
    fn push(&mut self, weight: Ratio, segments: Vec<Segment>) -> Result<(), Unrepresentable> {
        if weight.num == 0 { return Ok(()) }
        let mut out = Vec::with_capacity(segments.len());
        for segment in segments {
            match segment {
                Segment::Known(c) => out.push(Segment::Known(c)),
                Segment::Bag(entries) => if !push_bag(&mut out, entries) { return Ok(()) },
            }
        }
        match self.map.get_mut(&out) {
            Some(w) => *w = w.plus(weight)?,
            None => { if self.map.len() == N_COMPONENTS { return Err(Unrepresentable) } self.map.insert(out, weight); }
        }
        Ok(())
    }
    fn of(knowledge: &DrawKnowledge) -> Self {
        Self { map: knowledge.components.iter().map(|c| (c.segments.clone(), Ratio { num: c.weight, den: 1 })).collect() }
    }
}

fn intern(classes: &mut Vec<CardClass>, class: CardClass) -> u16 {
    if let Some(i) = classes.iter().position(|c| *c == class) { return i as u16 }
    classes.push(class);
    (classes.len() - 1) as u16
}
fn bag_of(classes: &mut Vec<CardClass>, cards: impl IntoIterator<Item = CardClass>) -> Vec<BagEntry> {
    let mut entries: Vec<BagEntry> = vec![];
    for class in cards {
        let class = intern(classes, class);
        match entries.iter_mut().find(|e| e.class == class) { Some(e) => e.count += 1, None => entries.push(BagEntry { class, count: 1, excluded: 0 }) }
    }
    entries
}
fn bag_free(entries: &[BagEntry]) -> u128 { entries.iter().map(|e| u128::from(e.count - e.excluded)).sum() }
fn bag_size(entries: &[BagEntry]) -> u128 { entries.iter().map(|e| u128::from(e.count)).sum() }
fn segment_len(segment: &Segment) -> usize { match segment { Segment::Known(_) => 1, Segment::Bag(e) => bag_size(e) as usize } }

/// Normal form (R-M1): a sorted class table of the used classes (equal classes after a re-key share one index),
/// segments re-normalized under it, integer weights over the least common denominator reduced by their gcd.
fn finish(classes: Vec<CardClass>, acc: Acc) -> Result<DrawKnowledge, Unrepresentable> {
    let mut seen = vec![false; classes.len()];
    for segment in acc.map.keys().flatten() {
        match segment { Segment::Known(k) => seen[*k as usize] = true, Segment::Bag(e) => for e in e { seen[e.class as usize] = true } }
    }
    let mut used = classes.iter().zip(&seen).filter(|(_, u)| **u).map(|(c, _)| *c).collect::<Vec<_>>();
    used.sort();
    used.dedup();
    let remap = |i: u16| used.binary_search(&classes[i as usize]).map_or(u16::MAX, |j| j as u16);
    let mut normal = Acc::default();
    for (segments, weight) in acc.map {
        normal.push(weight, segments.into_iter().map(|s| match s {
            Segment::Known(k) => Segment::Known(remap(k)),
            Segment::Bag(e) => Segment::Bag(e.into_iter().map(|e| BagEntry { class: remap(e.class), ..e }).collect()),
        }).collect())?;
    }
    if normal.map.is_empty() { return Err(Unrepresentable) }
    let mut lcm = 1u128;
    for w in normal.map.values() { lcm = (lcm / gcd(lcm, w.den)).checked_mul(w.den).ok_or(Unrepresentable)?; }
    let weights = normal.map.values().map(|w| w.num.checked_mul(lcm / w.den).ok_or(Unrepresentable)).collect::<Result<Vec<_>, _>>()?;
    let g = weights.iter().fold(0, |g, w| gcd(g, *w)).max(1);
    weights.iter().try_fold(0u128, |t, w| t.checked_add(w / g)).ok_or(Unrepresentable)?;
    let components = normal.map.into_keys().zip(weights).map(|(segments, weight)| Component { weight: weight / g, segments }).collect();
    let knowledge = DrawKnowledge { classes: used, components };
    if knowledge.heap_bytes() > KNOWLEDGE_BYTES_CAP { return Err(Unrepresentable) }
    Ok(knowledge)
}

/// Push a bag in normal form: known segments while its first card is forced (one class, or one class not
/// excluded), then the remaining bag sorted by class. False when no first card is possible (zero weight).
fn push_bag(out: &mut Vec<Segment>, mut entries: Vec<BagEntry>) -> bool {
    loop {
        entries.retain(|e| e.count > 0);
        for e in &mut entries { e.excluded = e.excluded.min(e.count); }
        match entries.as_slice() {
            [] => return true,
            [only] => { out.extend(std::iter::repeat_n(Segment::Known(only.class), only.count as usize)); return true }
            _ => {}
        }
        let free = entries.iter().filter(|e| e.count > e.excluded).map(|e| e.class).collect::<Vec<_>>();
        match free.as_slice() {
            [] => return false,
            [first] => {
                out.push(Segment::Known(*first));
                for e in &mut entries { e.excluded = 0; if e.class == *first { e.count -= 1; } }
            }
            _ => { entries.sort(); out.push(Segment::Bag(entries)); return true }
        }
    }
}

/// Where a removed draw card was published (`known_draw` from the top, `known_draw_bottom` from the bottom).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Published { Top(usize), Bottom(usize), No }

/// How the engine chose the removed copy (a0(ix) grid handle rules, recheck R-I1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::combat) enum Removal {
    /// Uniform over the eligible copies (Violence, the Stasis tie-break): the uniform-removal row.
    Uniform,
    /// A player-chosen candidate of an order-hiding grid (Seek, Secret Technique/Weapon, Omniscience; `piles::sort_draw_grid`,
    /// opened by `piles::open_draw_grid`). Which copy of the class the pick removed is not public (final review I1), so the
    /// update reads only public facts: a class with a published copy and another copy fails closed; one published copy is
    /// that copy; otherwise the unpublished copies must be interchangeable, or fail closed.
    Grid,
    /// A card chosen by its published position (Scry discards: the revealed top cards in position order).
    Revealed,
    /// The opening's Innate/bottled placement (first such card in physical order): exact when every such card is
    /// dealt (`all_dealt`), otherwise fail closed.
    Opening { all_dealt: bool },
}

/// One draw-pile operation (design §4.5.1 table), noted right after its physical change.
#[derive(Clone, Copy, Debug)]
pub(in crate::combat) enum PileOp {
    /// The whole draw pile was uniformly shuffled (entry, Deep Breath, Reboot's final shuffle, any reshuffle).
    Shuffled,
    /// The top `n` cards are a fresh uniform shuffle placed over the previous pile (discard recycle).
    ShuffledOnTop(usize),
    /// The top card left the pile, revealing `class` (draw, Havoc/Mayhem/Distilled Chaos, the opening deal).
    Drawn { instance: u64, class: CardClass },
    /// A known card was placed on top / at the bottom.
    TopDecked,
    BottomDecked,
    /// A card was inserted uniformly below the top card; `index` is the physical slot (checked, never modelled).
    Inserted(usize),
    /// A card left the middle of the pile under `law`, revealing `class`.
    Removed { instance: u64, class: CardClass, law: Removal },
    /// The top `n` cards were revealed in order and stay (Scry).
    Revealed(usize),
    /// No change (a read).
    Sync,
}

/// The draw knowledge with its physical bookkeeping; equality and the digest use the knowledge only.
#[derive(Clone, Debug)]
pub(in crate::combat) struct KnowledgeTracker {
    /// PM ruling (final review M7): maintained only in a combat whose profile publishes public knowledge (the Standard Run,
    /// `standard-combat-v1`), set once at entry (`Combat::enter_with_public_knowledge`); everywhere else every hook returns
    /// before any work and every read answers "no knowledge".
    pub(in crate::combat) maintained: bool,
    pub draw: Result<DrawKnowledge, Unrepresentable>,
    tracked: Option<Vec<(u64, CardClass)>>,
    pub prerolls: Prerolls,
}
impl Default for KnowledgeTracker {
    fn default() -> Self { Self { maintained: false, draw: Ok(DrawKnowledge::empty()), tracked: Some(vec![]), prerolls: Prerolls::default() } }
}
impl PartialEq for KnowledgeTracker { fn eq(&self, other: &Self) -> bool { self.maintained == other.maintained && self.draw == other.draw && self.prerolls == other.prerolls } }
impl Eq for KnowledgeTracker {}

pub(in crate::combat) fn class_of(state: &State, card: &OwnedCard) -> Option<CardClass> {
    card.full().map(|live| super::view::card_class(&super::view::live_card_view(state, live)))
}
/// Physical (instance, class) draw sequence, bottom to top; `None` with a legacy card.
fn draw_classes(state: &State) -> Option<Vec<(u64, CardClass)>> {
    state.piles.draw.iter().map(|card| Some((card.full()?.instance.0, class_of(state, card)?))).collect()
}

/// Note one draw-pile operation (call right after its physical change, before `known_draw` forgets a removed card).
pub(in crate::combat) fn note(state: &mut State, op: PileOp) {
    if !state.knowledge.maintained { return }
    #[cfg(test)]
    let start = std::time::Instant::now();
    let now = draw_classes(state);
    let mut tracker = std::mem::take(&mut state.knowledge);
    tracker.update(&state.public, now, op);
    state.knowledge = tracker;
    #[cfg(test)]
    MAINTENANCE.with(|m| { let (calls, nanos) = m.get(); m.set((calls + 1, nanos + start.elapsed().as_nanos())); });
}

/// Test-only accounting of knowledge maintenance (calls, nanoseconds) on this thread: every `note` and every pre-roll
/// observation (review M7 measurement; absent from non-test builds).
#[cfg(test)]
thread_local! { pub(crate) static MAINTENANCE: std::cell::Cell<(u64, u128)> = const { std::cell::Cell::new((0, 0)) }; }

impl KnowledgeTracker {
    fn update(&mut self, public: &PublicKnowledge, now: Option<Vec<(u64, CardClass)>>, op: PileOp) {
        let before = std::mem::replace(&mut self.tracked, now.clone());
        let Some(now) = now else { self.draw = Err(Unrepresentable); return };
        let full_shuffle = matches!(op, PileOp::Shuffled) || matches!(op, PileOp::ShuffledOnTop(n) if n == now.len());
        if now.is_empty() { self.draw = Ok(DrawKnowledge::empty()); return }
        if full_shuffle { self.draw = DrawKnowledge::shuffled(now.iter().map(|(_, c)| *c)); return }
        let (Some(before), Ok(knowledge)) = (before, &self.draw) else { self.draw = Err(Unrepresentable); return };
        self.draw = step(knowledge, &before, &now, public, op);
    }
}

/// The pre-operation (instance, class) sequence implied by `now` and `op`: the physical check and the re-key input.
fn pre_sequence(before: &[(u64, CardClass)], now: &[(u64, CardClass)], op: PileOp) -> Option<Vec<(u64, CardClass)>> {
    let mut pre = now.to_vec();
    match op {
        PileOp::Shuffled => return None,
        PileOp::ShuffledOnTop(n) => pre.truncate(now.len().checked_sub(n)?),
        PileOp::Drawn { instance, class } => pre.push((instance, class)),
        PileOp::TopDecked => { pre.pop()?; }
        PileOp::BottomDecked => { if pre.is_empty() { return None } pre.remove(0); }
        PileOp::Inserted(index) => { if index >= pre.len() { return None } pre.remove(index); }
        PileOp::Removed { instance, class, .. } => {
            let at = before.iter().position(|(i, _)| *i == instance)?;
            if at > pre.len() { return None }
            pre.insert(at, (instance, class));
        }
        PileOp::Revealed(_) | PileOp::Sync => {}
    }
    (pre.len() == before.len() && pre.iter().zip(before).all(|(a, b)| a.0 == b.0)).then_some(pre)
}

fn step(knowledge: &DrawKnowledge, before: &[(u64, CardClass)], now: &[(u64, CardClass)], public: &PublicKnowledge, op: PileOp)
    -> Result<DrawKnowledge, Unrepresentable> {
    let pre = pre_sequence(before, now, op).ok_or(Unrepresentable)?;
    // Re-key in-place changes: every card of a class changes the same way, or fail closed.
    let mut rekey: BTreeMap<CardClass, CardClass> = BTreeMap::new();
    for ((_, old), (_, new)) in before.iter().zip(&pre) {
        if *rekey.entry(*old).or_insert(*new) != *new { return Err(Unrepresentable) }
    }
    let rekeyed;
    let knowledge = if rekey.iter().any(|(a, b)| a != b) {
        let classes = knowledge.classes.iter().map(|c| *rekey.get(c).unwrap_or(c)).collect();
        rekeyed = finish(classes, Acc::of(knowledge))?;
        &rekeyed
    } else { knowledge };
    // Safety net: every component holds the pre-operation class multiset.
    let mut expected = BTreeMap::new();
    for (_, c) in &pre { *expected.entry(*c).or_insert(0usize) += 1; }
    if knowledge.class_counts().as_ref() != Some(&expected) { return Err(Unrepresentable) }
    let mut classes = knowledge.classes.clone();
    let mut out = Acc::default();
    let components = knowledge.components.iter().map(|c| (Ratio { num: c.weight, den: 1 }, &c.segments));
    match op {
        PileOp::Shuffled => unreachable!("handled by the tracker"),
        PileOp::Sync => out = Acc::of(knowledge),
        PileOp::ShuffledOnTop(n) => {
            let bag = bag_of(&mut classes, now[now.len() - n..].iter().map(|(_, c)| *c));
            for (w, s) in components { out.push(w, std::iter::once(Segment::Bag(bag.clone())).chain(s.iter().cloned()).collect())?; }
        }
        PileOp::Drawn { class, .. } => {
            let c = intern(&mut classes, class);
            for (w, s) in components { if let Some((w, s)) = take_top(w, s.clone(), c)? { out.push(w, s)?; } }
        }
        PileOp::Revealed(n) => {
            let top = now.iter().rev().take(n).map(|(_, c)| intern(&mut classes, *c)).collect::<Vec<_>>();
            for (w, s) in components {
                let mut kept = Some((w, s.clone()));
                for c in &top { kept = match kept { Some((w, s)) => take_top(w, s, *c)?, None => None }; }
                if let Some((w, rest)) = kept { out.push(w, top.iter().map(|c| Segment::Known(*c)).chain(rest).collect())?; }
            }
        }
        PileOp::TopDecked => {
            let c = intern(&mut classes, now[now.len() - 1].1);
            for (w, s) in components { out.push(w, std::iter::once(Segment::Known(c)).chain(s.iter().cloned()).collect())?; }
        }
        PileOp::BottomDecked => {
            let c = intern(&mut classes, now[0].1);
            for (w, s) in components { out.push(w, s.iter().cloned().chain(std::iter::once(Segment::Known(c))).collect())?; }
        }
        PileOp::Inserted(index) => {
            let c = intern(&mut classes, now[index].1);
            for (w, s) in components { insert_below_top(&mut out, w, s, c)?; }
        }
        PileOp::Removed { instance, class, law } => {
            let c = intern(&mut classes, class);
            let law = variant_law(law);
            let published = if law == Removal::Grid && !hidden_grid_rule() { grid_published(public, &pre, class)? } else {
                let handle = super::card_state::handle(CardInstanceId(instance));
                if let Some(i) = public.known_draw.iter().position(|h| *h == handle) { Published::Top(i) }
                else if let Some(i) = public.known_draw_bottom.iter().position(|h| *h == handle) { Published::Bottom(i) } else { Published::No }
            };
            if law == Removal::Revealed && published == Published::No { return Err(Unrepresentable) }
            for (w, s) in components {
                match law {
                    Removal::Uniform => remove_uniform(&mut out, w, s, c)?,
                    Removal::Grid | Removal::Revealed | Removal::Opening { all_dealt: true } =>
                        remove_chosen(&mut out, w, s, c, published, public.known_draw.len(), public.known_draw_bottom.len())?,
                    Removal::Opening { all_dealt: false } => return Err(Unrepresentable),
                }
            }
        }
    }
    finish(classes, out)
}

/// Final review I1: where an order-hiding grid's pick of class `class` was published, from public facts only (the
/// classes at the published positions and the class count; never the removed handle). A class with a published copy and
/// another copy is ambiguous: `Unrepresentable` (`piles::open_draw_grid` forgets such positions before the grid opens).
fn grid_published(public: &PublicKnowledge, pre: &[(u64, CardClass)], class: CardClass) -> Result<Published, Unrepresentable> {
    let class_of = |h: &CardHandle| pre.iter().find(|(i, _)| super::card_state::handle(CardInstanceId(*i)) == *h).map(|(_, c)| *c);
    let top = public.known_draw.iter().enumerate().filter(|(_, h)| class_of(h) == Some(class)).map(|(i, _)| i).collect::<Vec<_>>();
    let bottom = public.known_draw_bottom.iter().enumerate().filter(|(_, h)| class_of(h) == Some(class)).map(|(i, _)| i).collect::<Vec<_>>();
    let copies = pre.iter().filter(|(_, c)| *c == class).count();
    match (top.as_slice(), bottom.as_slice()) {
        ([], []) => Ok(Published::No),
        ([i], []) if copies == 1 => Ok(Published::Top(*i)),
        ([], [i]) if copies == 1 => Ok(Published::Bottom(*i)),
        _ => Err(Unrepresentable),
    }
}

/// Observe the top card's class `c` and pop it; `None` when this component cannot show `c`.
fn take_top(w: Ratio, mut s: Vec<Segment>, c: u16) -> Result<Option<(Ratio, Vec<Segment>)>, Unrepresentable> {
    match s.first_mut() {
        None => Ok(None),
        Some(Segment::Known(k)) => { if *k != c { return Ok(None) } s.remove(0); Ok(Some((w, s))) }
        Some(Segment::Bag(entries)) => {
            let total_free = bag_free(entries);
            let Some(e) = entries.iter_mut().find(|e| e.class == c && e.count > e.excluded) else { return Ok(None) };
            let w = w.times(u128::from(e.count - e.excluded), total_free)?;
            e.count -= 1;
            for e in entries.iter_mut() { e.excluded = 0; }
            Ok(Some((w, s)))
        }
    }
}

/// Uniform insertion of `c` into one of the slots below each card (never on top): a known card's slot gives `c`
/// known directly below it (weight 1); a bag of size `s` gives the bag with `c` added and top-excluded (weight `s`).
fn insert_below_top(out: &mut Acc, w: Ratio, s: &[Segment], c: u16) -> Result<(), Unrepresentable> {
    #[cfg(test)]
    if variant() == Some(Variant::V1PrefixTruncation) { return insert_v1(out, w, s, c) }
    for (j, segment) in s.iter().enumerate() {
        let mut next = s.to_vec();
        match segment {
            Segment::Known(_) => { next.insert(j + 1, Segment::Known(c)); out.push(w, next)?; }
            Segment::Bag(entries) => {
                let size = bag_size(entries);
                let excluded = top_exclusion();
                let Segment::Bag(bag) = &mut next[j] else { unreachable!() };
                match bag.iter_mut().find(|e| e.class == c) {
                    Some(e) => { e.count += 1; e.excluded += excluded; }
                    None => bag.push(BagEntry { class: c, count: 1, excluded }),
                }
                out.push(w.times(size, 1)?, next)?;
            }
        }
    }
    Ok(())
}

/// Remove one copy of class `c` from bag segment `j`: an excluded copy keeps the first card; a non-excluded copy was
/// the first card with probability 1 / (non-excluded size). `copies` weights the copies (uniform law) or is 1.
fn remove_from_bag(out: &mut Acc, w: Ratio, s: &[Segment], j: usize, c: u16, excluded_copy: bool, copies: u128)
    -> Result<(), Unrepresentable> {
    let Segment::Bag(entries) = &s[j] else { unreachable!() };
    let free = bag_free(entries);
    let mut with = |first: bool, weight: Ratio| {
        let mut next = s.to_vec();
        let Segment::Bag(bag) = &mut next[j] else { unreachable!() };
        for e in bag.iter_mut() {
            if e.class == c { e.count -= 1; if excluded_copy { e.excluded -= 1; } }
            if first { e.excluded = 0; }
        }
        out.push(weight, next)
    };
    if excluded_copy { return with(false, w.times(copies, 1)?) }
    with(true, w.times(copies, free)?)?;
    if free > 1 { with(false, w.times(copies * (free - 1), free)?)?; }
    Ok(())
}

/// Uniform random removal of a copy of `c`: split over every copy (design §4.5.1, recheck R-I1 row 1).
fn remove_uniform(out: &mut Acc, w: Ratio, s: &[Segment], c: u16) -> Result<(), Unrepresentable> {
    for (j, segment) in s.iter().enumerate() {
        match segment {
            Segment::Known(k) if *k == c => { let mut next = s.to_vec(); next.remove(j); out.push(w, next)?; }
            Segment::Known(_) => {}
            Segment::Bag(entries) => if let Some(e) = entries.iter().find(|e| e.class == c) {
                if e.excluded > 0 { remove_from_bag(out, w, s, j, c, true, u128::from(e.excluded))?; }
                if e.count > e.excluded { remove_from_bag(out, w, s, j, c, false, u128::from(e.count - e.excluded))?; }
            },
        }
    }
    Ok(())
}

/// A player-chosen removal of a copy of `c` under a deterministic handle rule (recheck R-I1 row 2): a published card
/// removes its known segment; otherwise exact only when the unpublished copies are interchangeable (one known copy,
/// or one bag whose copies are all non-excluded or all excluded); otherwise fail closed.
fn remove_chosen(out: &mut Acc, w: Ratio, s: &[Segment], c: u16, published: Published, top: usize, bottom: usize)
    -> Result<(), Unrepresentable> {
    // Segment index of a position known in this component (published positions are known in every component).
    let total = s.iter().map(segment_len).sum::<usize>();
    let known_at = |position: usize| {
        let mut depth = 0;
        for (j, segment) in s.iter().enumerate() {
            if depth == position { return matches!(segment, Segment::Known(_)).then_some(j) }
            depth += segment_len(segment);
            if depth > position { return None }
        }
        None
    };
    let at = match published {
        Published::Top(i) => Some(known_at(i).ok_or(Unrepresentable)?),
        Published::Bottom(i) => Some(known_at(total.checked_sub(i + 1).ok_or(Unrepresentable)?).ok_or(Unrepresentable)?),
        Published::No => None,
    };
    if let Some(j) = at {
        if s[j] != Segment::Known(c) { return Err(Unrepresentable) }
        let mut next = s.to_vec();
        next.remove(j);
        return out.push(w, next)
    }
    let published_segments = (0..top.min(total)).chain(total.saturating_sub(bottom)..total)
        .map(|p| known_at(p).ok_or(Unrepresentable)).collect::<Result<Vec<_>, _>>()?;
    let holders = s.iter().enumerate().filter(|(j, segment)| !published_segments.contains(j) && match segment {
        Segment::Known(k) => *k == c,
        Segment::Bag(e) => e.iter().any(|e| e.class == c),
    }).map(|(j, _)| j).collect::<Vec<_>>();
    let [j] = holders.as_slice() else { return Err(Unrepresentable) };
    match &s[*j] {
        Segment::Known(_) => { let mut next = s.to_vec(); next.remove(*j); out.push(w, next) }
        Segment::Bag(entries) => {
            let e = entries.iter().find(|e| e.class == c).unwrap();
            if e.excluded == 0 { remove_from_bag(out, w, s, *j, c, false, 1) }
            else if e.excluded == e.count { remove_from_bag(out, w, s, *j, c, true, 1) }
            else { Err(Unrepresentable) }
        }
    }
}

/// Test-only knowledge variants for the negative controls of design §4.10 (d) and recheck R-I1.
#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::combat) enum Variant { NoTopExclusion, V1PrefixTruncation, UniformGrid,
    /// Final review I1 negative control: part a's grid rule (no forgetting at the grid opening; the update branches on
    /// whether the TRUE removed handle was published).
    HiddenGridRule,
    /// Recheck R1 negative control: no forgetting before a hidden (uniform random) pick; the published positions are
    /// pruned by the TRUE taken handle.
    HiddenPickRule }
#[cfg(test)]
thread_local! { pub(in crate::combat) static VARIANT: std::cell::Cell<Option<Variant>> = const { std::cell::Cell::new(None) }; }
#[cfg(test)]
fn variant() -> Option<Variant> { VARIANT.with(|v| v.get()) }
#[cfg(test)]
fn top_exclusion() -> u16 { u16::from(variant() != Some(Variant::NoTopExclusion)) }
#[cfg(not(test))]
fn top_exclusion() -> u16 { 1 }
/// Whether the I1 negative control (part a's leaky grid rule) is active (always false outside tests).
pub(in crate::combat) fn hidden_grid_rule() -> bool {
    #[cfg(test)]
    { variant() == Some(Variant::HiddenGridRule) }
    #[cfg(not(test))]
    { false }
}
/// Whether the R1 negative control (no forgetting before a hidden pick) is active (always false outside tests).
pub(in crate::combat) fn hidden_pick_rule() -> bool {
    #[cfg(test)]
    { variant() == Some(Variant::HiddenPickRule) }
    #[cfg(not(test))]
    { false }
}
fn variant_law(law: Removal) -> Removal {
    #[cfg(test)]
    if law == Removal::Grid && variant() == Some(Variant::UniformGrid) { return Removal::Uniform }
    law
}
/// The v1 coarse model: after an insertion only the top known card stays known; the rest is one uniform bag.
#[cfg(test)]
fn insert_v1(out: &mut Acc, w: Ratio, s: &[Segment], c: u16) -> Result<(), Unrepresentable> {
    let (head, rest) = match s.first() { Some(Segment::Known(k)) => (vec![Segment::Known(*k)], &s[1..]), _ => (vec![], s) };
    let mut bag = vec![BagEntry { class: c, count: 1, excluded: u16::from(head.is_empty()) }];
    for segment in rest {
        let cards = match segment { Segment::Known(k) => vec![BagEntry { class: *k, count: 1, excluded: 0 }], Segment::Bag(e) => e.iter().map(|e| BagEntry { excluded: 0, ..*e }).collect() };
        for e in cards { match bag.iter_mut().find(|b| b.class == e.class) { Some(b) => b.count += e.count, None => bag.push(e) } }
    }
    out.push(w, head.into_iter().chain(std::iter::once(Segment::Bag(bag))).collect())
}

/// Fail-closed tie guard (PM item-5 verdicts): an unknown draw card with `purge` or `rebound_on_finish`, a shared live
/// copy family, or two unknown draw cards of one class with different `m21` values make the knowledge unrepresentable.
fn tie_guard(state: &State) -> bool {
    let known = state.public.known_draw.iter().chain(&state.public.known_draw_bottom).collect::<std::collections::BTreeSet<_>>();
    let mut families: BTreeMap<_, usize> = BTreeMap::new();
    for card in super::piles::owned_full_cards(state) { *families.entry(card.copy_family).or_insert(0) += 1; }
    let mut m21: BTreeMap<CardClass, _> = BTreeMap::new();
    for owned in &state.piles.draw {
        let Some(card) = owned.full() else { return false };
        if known.contains(&owned.handle()) { continue }
        if card.local.purge || card.local.rebound_on_finish || families.get(&card.copy_family).is_some_and(|n| *n > 1) { return false }
        let class = class_of(state, owned).unwrap();
        if *m21.entry(class).or_insert(card.local.m21) != card.local.m21 { return false }
    }
    true
}

impl Combat {
    /// M34s-b final review I1: whether selecting card `instance` in the open selection would remove a draw-pile copy chosen
    /// by hidden instance order: two or more candidates share its card class and not all of them sit at published
    /// positions (an order-hiding grid; a Scry selection's candidates are all published). Public: the candidates' classes
    /// and published positions. The search arena refuses such a pick in a branch, whose copy labels are not resampled.
    pub fn pick_depends_on_hidden_order(&self, instance: u64) -> bool {
        let state = &self.state;
        let Some(selection) = &state.pending_selection else { return false };
        if selection.source_zone != SelectionSourceZone::Draw { return false }
        let class = |h: &CardHandle| state.piles.draw.iter().find(|c| c.handle() == *h).and_then(|c| class_of(state, c));
        let chosen = super::card_state::handle(CardInstanceId(instance));
        if !selection.candidates.contains(&chosen) { return false }
        let target = class(&chosen);
        let copies = selection.candidates.iter().filter(|h| target.is_none() || class(h) == target).collect::<Vec<_>>();
        let published = |h: &&CardHandle| state.public.known_draw.contains(h) || state.public.known_draw_bottom.contains(h);
        copies.len() >= 2 && !copies.iter().all(published)
    }
    /// M34s-a public draw knowledge (design §4.5.1): the exact posterior over the draw pile's class sequence given
    /// public history, in normal form, or `Unrepresentable` (L1). With Frozen Eye the true order is public.
    /// Without maintenance (`maintains_public_knowledge` false) there is no knowledge: always `Unrepresentable`, never stale.
    pub fn draw_knowledge(&self) -> Result<DrawKnowledge, Unrepresentable> {
        let state = &self.state;
        if !state.knowledge.maintained { return Err(Unrepresentable) }
        let now = draw_classes(state);
        let mut tracker = state.knowledge.clone();
        tracker.update(&state.public, now.clone(), PileOp::Sync);
        if !tie_guard(state) { return Err(Unrepresentable) }
        if super::view::public_rules(state).reveal_draw_order {
            return DrawKnowledge::known(now.ok_or(Unrepresentable)?.into_iter().rev().map(|(_, c)| c))
        }
        let knowledge = tracker.draw?;
        // Invariant: the published known cards are known at their positions in every component.
        let class = |handle: &CardHandle| state.piles.draw.iter().find(|c| c.handle() == *handle).and_then(|c| class_of(state, c));
        for component in &knowledge.components {
            let mut depths = vec![];
            for segment in &component.segments { depths.extend(std::iter::repeat_n(segment, segment_len(segment))); }
            let at = |depth: usize, handle: &CardHandle| matches!(depths.get(depth), Some(Segment::Known(k)) if Some(*knowledge.class(*k)) == class(handle));
            let len = depths.len();
            if !state.public.known_draw.iter().enumerate().all(|(i, h)| at(i, h))
                || !state.public.known_draw_bottom.iter().enumerate().all(|(i, h)| len > i && at(len - 1 - i, h)) { return Err(Unrepresentable) }
        }
        Ok(knowledge)
    }
}

/// A hidden pre-rolled value of the a0(i) census (design §4.5.2).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum PrerollKind { LouseBite, DarklingNip }

/// `C_v ⊆ L_v`: bit `i` set iff `low + i` is consistent with every observation; the posterior is uniform on it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ConsistentSet { pub low: i32, pub mask: u32 }
impl ConsistentSet {
    pub fn values(&self) -> impl Iterator<Item = i32> + '_ { (0..32).filter(|i| self.mask & (1 << i) != 0).map(|i| self.low + i) }
    pub fn contains(&self, value: i32) -> bool { value >= self.low && value - self.low < 32 && self.mask & (1 << (value - self.low)) != 0 }
}

/// Per-actor consistent sets; `Unrepresentable` when the true value lies outside its law or the Darkling stores disagree.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(in crate::combat) struct Prerolls { sets: BTreeMap<(EnemyHandle, PrerollKind), Result<ConsistentSet, Unrepresentable>> }

fn preroll_of(enemy: &EnemyState) -> Option<(PrerollKind, FullIntentId)> {
    match enemy.enemy_id {
        ActorEnemyId::Full(crate::content::EnemyId::LouseRed | crate::content::EnemyId::LouseGreen) => Some((PrerollKind::LouseBite, FullIntentId::Bite)),
        ActorEnemyId::Full(crate::content::EnemyId::Darkling) => Some((PrerollKind::DarklingNip, FullIntentId::from_name("darkling_nip"))),
        _ => None,
    }
}
fn preroll_law(kind: PrerollKind, ascension: u8) -> (i32, i32) {
    match kind {
        PrerollKind::LouseBite => super::enemies::ascension::louse_bite_range(ascension),
        PrerollKind::DarklingNip => super::enemies::ascension::darkling_nip_range(ascension),
    }
}

/// Intersect the consistent sets with the intent value the view publishes at a published decision or selection boundary
/// (`engine::resolve_from`): `C_v := {x ∈ C_v : display(x, modifiers now) = published value}` (M34s-b b0 D2, final review
/// M2: narrow on the published display, never on a value recomputed at execution, which the player may not see — a
/// blocked hit, Torii, Buffer, or modifiers changed during the enemy turn). A set is created at the actor's first
/// observation as its setup law `L_v`. Nothing happens while intents are hidden (Runic Dome, L2). Invariant: the true
/// base maps to the published value (the display is fresh at every boundary); otherwise the set fails closed.
pub(in crate::combat) fn observe_prerolls(state: &mut State) {
    if !state.knowledge.maintained { return }
    #[cfg(test)]
    let start = std::time::Instant::now();
    observe_prerolls_inner(state);
    #[cfg(test)]
    MAINTENANCE.with(|m| { let (calls, nanos) = m.get(); m.set((calls + 1, nanos + start.elapsed().as_nanos())); });
}
fn observe_prerolls_inner(state: &mut State) {
    if !state.full_runtime || super::view::public_rules(state).hide_intents { return }
    let mut updates = vec![];
    for enemy in &state.enemies {
        let Some((kind, intent)) = preroll_of(enemy) else { continue };
        let Some(entry) = enemy.full_damage_entries.get(&intent) else { continue };
        let key = (enemy.instance_id.clone(), kind);
        let mut set = state.knowledge.prerolls.sets.get(&key).copied().unwrap_or_else(|| {
            let (low, high) = preroll_law(kind, state.ascension);
            let width = high - low + 1;
            if !(1..=32).contains(&width) { return Err(Unrepresentable) }
            Ok(ConsistentSet { low, mask: if width == 32 { u32::MAX } else { (1 << width) - 1 } })
        });
        // Runtime invariant: Darkling Nip is written to both stores together (M8; enemies/mod.rs setup).
        let stores_agree = kind != PrerollKind::DarklingNip
            || matches!(enemy.full_memory, Some(EnemyMemory::Darkling { nip, .. }) if nip == entry.base);
        if !stores_agree || set.is_ok_and(|s| !s.contains(entry.base)) { set = Err(Unrepresentable); }
        if let (Ok(s), ActorIntentId::Full(shown_intent), Some(output)) = (&mut set, &enemy.intent_id, enemy.intent_damage) {
            if *shown_intent == intent {
                let published = super::effects::displayed_intent_damage(state, enemy, output);
                for x in s.values().collect::<Vec<_>>() {
                    if super::effects::displayed_damage_of_base(state, enemy, x).ok() != Some(published) { s.mask &= !(1 << (x - s.low)); }
                }
                if !s.contains(entry.base) { set = Err(Unrepresentable); }
            }
        }
        updates.push((key, set));
    }
    state.knowledge.prerolls.sets.extend(updates);
}

impl Combat {
    /// Whether this combat maintains public knowledge (set at entry from its profile; final review M7).
    pub fn maintains_public_knowledge(&self) -> bool { self.state.knowledge.maintained }
    /// Fuzz/test oracle only (never public): the true draw pile's classes, top to bottom.
    #[cfg(any(test, feature = "fuzz-diagnostics"))]
    pub fn true_draw_classes(&self) -> Vec<CardClass> {
        self.state.piles.draw.iter().rev().filter_map(|card| class_of(&self.state, card)).collect()
    }
    /// M34s-a pre-roll consistent sets (design §4.5.2): each pre-rolled value's candidates consistent with public history.
    /// Empty without maintenance (`maintains_public_knowledge` false).
    pub fn preroll_sets(&self) -> Vec<(EnemyHandle, PrerollKind, Result<ConsistentSet, Unrepresentable>)> {
        if !self.state.knowledge.maintained { return vec![] }
        self.state.knowledge.prerolls.sets.iter().map(|((enemy, kind), set)| (enemy.clone(), *kind, *set)).collect()
    }
}

/// M34s-b: one hidden pre-rolled value of an actor with its consistent candidates (ascending; the setup law until the
/// first observation). The belief keeps a value with one candidate and re-samples one with several (design §4.6.2 R3).
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PrerollCandidates { pub enemy: EnemyHandle, pub slot: u8, pub kind: PrerollKind, pub intent: FullIntentId, pub values: Vec<i32> }

/// Every actor's pre-rolled values with their candidates; `Unrepresentable` if any set is (a value outside its law,
/// disagreeing Darkling stores).
pub(in crate::combat) fn preroll_candidates(state: &State) -> Result<Vec<PrerollCandidates>, Unrepresentable> {
    let mut out = vec![];
    for enemy in &state.enemies {
        let Some((kind, intent)) = preroll_of(enemy) else { continue };
        if !enemy.full_damage_entries.contains_key(&intent) { continue }
        let set = match state.knowledge.prerolls.sets.get(&(enemy.instance_id.clone(), kind)) {
            Some(set) => (*set)?,
            None => {
                let (low, high) = preroll_law(kind, state.ascension);
                let width = high - low + 1;
                if !(1..=32).contains(&width) { return Err(Unrepresentable) }
                ConsistentSet { low, mask: if width == 32 { u32::MAX } else { (1 << width) - 1 } }
            }
        };
        let slot = *state.actor_slots.get(&enemy.instance_id).ok_or(Unrepresentable)?;
        out.push(PrerollCandidates { enemy: enemy.instance_id.clone(), slot, kind, intent, values: set.values().collect() });
    }
    Ok(out)
}
/// Write a pre-rolled value to every place it is stored (the move's damage base; Darkling's memory too).
pub(in crate::combat) fn set_preroll(state: &mut State, p: &PrerollCandidates, value: i32) {
    let Some(enemy) = state.enemies.iter_mut().find(|e| e.instance_id == p.enemy) else { return };
    if let Some(entry) = enemy.full_damage_entries.get_mut(&p.intent) { entry.base = value; }
    if let Some(EnemyMemory::Darkling { nip, .. }) = enemy.full_memory.as_mut() { *nip = value; }
}
/// Resynchronise the physical bookkeeping with a rearranged draw pile (the knowledge value is unchanged).
pub(in crate::combat) fn resync(state: &mut State) { state.knowledge.tracked = draw_classes(state); }

/// P6: the tracker's fields, classified (b0): `maintained` profile flag, `draw` and `prerolls` public knowledge,
/// `tracked` physical bookkeeping (excluded from equality; the belief resynchronises it with every arrangement).
#[cfg(test)]
#[test]
fn tracker_census_classifies_every_field() {
    let KnowledgeTracker { maintained: _, draw: _, tracked: _, prerolls: _ } = KnowledgeTracker::default();
}

/// M34s-b final review I1 diagnostics (env P1 duplicate-copy families; never a gameplay path).
#[cfg(any(test, feature = "m34-diagnostics"))]
impl Combat {
    /// Create a card through the shared generated-card constructor and insertion owner (the `CreateCard` arm's pile,
    /// knowledge and public-history owners, without its effect-producer whitelist, so a fixture can create e.g. Seek);
    /// returns its instance.
    pub fn diagnostic_create_card(&mut self, id: crate::content::CardId, upgrades: u32, destination: CardDestination) -> Option<u64> {
        let state = &mut self.state;
        let instance = super::piles::next_generated_instance(state).ok()?;
        let history = super::card_state::CardConstructionHistory::from_state(state);
        let card = super::card_state::generated_with_history(id, instance, upgrades, history)?;
        super::piles::insert_generated(state, card, destination).ok()?;
        Some(instance.0)
    }
    /// Apply one player effect through the shared pile owner (e.g. `Recycle`).
    pub fn diagnostic_apply(&mut self, kind: EffectKind) -> Option<()> {
        super::piles::apply_effect(&mut self.state, &Effect { source: Source::Actor(Actor::Player), target: Some(Actor::Player), kind }).ok()?;
        Some(())
    }
    /// The discard pile's size.
    pub fn diagnostic_discard_len(&self) -> usize { self.state.piles.discard.len() }
    /// Oracle (never public): the true draw pile's classes, top to bottom (proves a family's true worlds differ).
    pub fn diagnostic_draw_classes(&self) -> Vec<Option<CardClass>> { self.state.piles.draw.iter().rev().map(|c| class_of(&self.state, c)).collect() }
    /// A supplied public upgrade of the discard pile's card at `index` (as a visible in-pile upgrade).
    pub fn diagnostic_upgrade_discard(&mut self, index: usize) -> Option<()> {
        self.state.piles.discard.get_mut(index)?.full_mut()?.upgrades += 1;
        Some(())
    }
    /// Pairs of combat-local cards in the hand, draw or discard pile with equal card classes (the copies whose labels a
    /// public-equivalent world may exchange).
    pub fn diagnostic_identical_copies(&self) -> Vec<(u64, u64)> {
        let state = &self.state;
        let cards = state.piles.hand.iter().chain(&state.piles.draw).chain(&state.piles.discard)
            .filter_map(|o| o.full().filter(|l| matches!(l.origin, CombatCardRef::CombatLocal(_))).map(|l| (l.instance.0, class_of(state, o))))
            .collect::<Vec<_>>();
        let mut out = vec![];
        for (i, (a, ca)) in cards.iter().enumerate() { for (b, cb) in &cards[i + 1..] { if ca.is_some() && ca == cb { out.push((*a, *b)); } } }
        out
    }
    /// The public-equivalent world in which copies `a` and `b` (equal classes) have exchanged places: each takes the
    /// other's pile slot, and a published draw position follows the card now there. False when not identical copies.
    pub fn diagnostic_swap_identical_copies(&mut self, a: u64, b: u64) -> bool {
        if !self.diagnostic_identical_copies().contains(&(a.min(b), a.max(b))) && !self.diagnostic_identical_copies().contains(&(a, b)) { return false }
        let (ha, hb) = (super::card_state::handle(CardInstanceId(a)), super::card_state::handle(CardInstanceId(b)));
        let state = &mut self.state;
        let locate = |piles: &super::model::OwnedPiles, h: &CardHandle| -> Option<(u8, usize)> {
            for (z, pile) in [&piles.hand, &piles.draw, &piles.discard].into_iter().enumerate() {
                if let Some(i) = pile.iter().position(|c| c.handle() == *h) { return Some((z as u8, i)) }
            }
            None
        };
        let (Some(pa), Some(pb)) = (locate(&state.piles, &ha), locate(&state.piles, &hb)) else { return false };
        let take = |piles: &mut super::model::OwnedPiles, (z, i): (u8, usize)| -> OwnedCard {
            let pile = match z { 0 => &mut piles.hand, 1 => &mut piles.draw, _ => &mut piles.discard };
            pile[i].clone()
        };
        let (ca, cb) = (take(&mut state.piles, pa), take(&mut state.piles, pb));
        let mut put = |(z, i): (u8, usize), card: OwnedCard| { let pile = match z { 0 => &mut state.piles.hand, 1 => &mut state.piles.draw, _ => &mut state.piles.discard }; pile[i] = card; };
        put(pa, cb);
        put(pb, ca);
        for known in state.public.known_draw.iter_mut().chain(state.public.known_draw_bottom.iter_mut()) {
            if *known == ha { *known = hb.clone() } else if *known == hb { *known = ha.clone() }
        }
        resync(state);
        true
    }
    /// Recheck R1 pairs: advance the hidden card-random stream by `draws` draws (a world with an equal public history and
    /// another hidden RNG state). None without the stream.
    pub fn diagnostic_advance_card_rng(&mut self, draws: usize) -> Option<()> {
        for _ in 0..draws {
            let (_, next) = super::rng::uniform_below(self.state.card_rng.as_ref()?, 2).ok()?;
            self.state.card_rng = Some(next);
        }
        Some(())
    }
    /// Oracle (never public): the hand and the Stasis custody as (instance, class), in order (which copies a hidden pick
    /// took).
    pub fn diagnostic_hand_and_stasis(&self) -> [Vec<(u64, Option<CardClass>)>; 2] {
        let state = &self.state;
        let row = |o: &OwnedCard| (o.full().map_or(0, |l| l.instance.0), class_of(state, o));
        [state.piles.hand.iter().map(row).collect(), state.stasis.values().map(row).collect()]
    }
}
