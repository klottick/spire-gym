//! M34s-a a2: public knowledge (design §4.5). The draw knowledge against a brute-force posterior that enumerates every
//! history at the handle level and reproduces the engine's real handle choices (the grid rule `piles::sort_draw_grid`,
//! the instance order of copies, Stasis' uniform tie-break); the engine itself runs every operation through its real
//! paths (CreateCard, ShuffleDraw, DrawOne, Recycle, Scry and its commit, Stasis capture, the Seek grid removal).
use super::super::*;
use super::super::knowledge::{ConsistentSet, DrawKnowledge, PrerollKind, Removal, Segment, Unrepresentable, Variant, VARIANT};
use super::super::model::{OwnedCard, State};
use crate::{content::{CardId, Character, EncounterId, EnemyId}, items::CombatItemContext, run::RunRng};
use std::collections::BTreeMap;
use CardId::*;

// ---------- exact rationals ----------
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Q(u128, u128);
fn gcd(a: u128, b: u128) -> u128 { if b == 0 { a } else { gcd(b, a % b) } }
impl Q {
    fn new(n: u128, d: u128) -> Q { let g = gcd(n, d).max(1); Q(n / g, d / g) }
    fn add(self, o: Q) -> Q { let g = gcd(self.1, o.1); Q::new(self.0 * (o.1 / g) + o.0 * (self.1 / g), self.1 / g * o.1) }
    fn mul(self, o: Q) -> Q { Q::new(self.0 * o.0, self.1 * o.1) }
}
type Dist = BTreeMap<Vec<CardClass>, Q>;

/// The knowledge's distribution over class sequences (top to bottom).
fn expand(k: &DrawKnowledge) -> Dist {
    let total = k.total_weight();
    let mut out = Dist::new();
    for component in k.components() {
        let mut partial: Vec<(Vec<CardClass>, Q)> = vec![(vec![], Q::new(component.weight, total))];
        for segment in &component.segments {
            let mut next = vec![];
            for (seq, p) in partial {
                match segment {
                    Segment::Known(c) => { let mut s = seq.clone(); s.push(*k.class(*c)); next.push((s, p)); }
                    Segment::Bag(entries) => {
                        let counts = entries.iter().map(|e| (*k.class(e.class), e.count as u128, e.excluded as u128)).collect::<Vec<_>>();
                        for (tail, q) in bag_sequences(&counts, true) { let mut s = seq.clone(); s.extend(tail); next.push((s, p.mul(q))); }
                    }
                }
            }
            partial = next;
        }
        for (seq, p) in partial { let slot = out.entry(seq).or_insert(Q(0, 1)); *slot = slot.add(p); }
    }
    out
}
fn bag_sequences(counts: &[(CardClass, u128, u128)], first: bool) -> Vec<(Vec<CardClass>, Q)> {
    let free: u128 = counts.iter().map(|(_, n, e)| if first { n - e } else { *n }).sum();
    if counts.iter().all(|(_, n, _)| *n == 0) { return vec![(vec![], Q(1, 1))] }
    let mut out = vec![];
    for (i, (c, n, e)) in counts.iter().enumerate() {
        let weight = if first { n - e } else { *n };
        if weight == 0 { continue }
        let mut rest = counts.to_vec();
        rest[i].1 -= 1;
        for (tail, q) in bag_sequences(&rest, false) {
            let mut s = vec![*c]; s.extend(tail);
            out.push((s, Q::new(weight, free).mul(q)));
        }
    }
    out
}

// ---------- the engine world ----------
fn fixture(deck: &[CardId], seed: u8) -> Combat {
    let mut entry = enemies::setup(EncounterId::Cultist, 1, 0, &mut RunRng::new([seed; 32], rng::Partition::Test, 0), enemies::SetupContext {
        player: PlayerState { hp: 80, max_hp: 80, block: 0, energy: 3, max_energy: 3, draw_per_turn: 5, powers: vec![] }, player_powers: vec![],
        character: Character::Ironclad, cards: vec![permanent_live_card(Defend, CardInstanceId(99), 0)], opening_draw: 0, encounter_visit: 0 }).unwrap();
    entry.cards = deck.iter().enumerate().map(|(i, id)| complete_live_card(*id, CardInstanceId(i as u64 + 1), 0).unwrap()).collect();
    entry.opening_draw = 0;
    let mut items = Some(CombatItemContext::new(vec![], vec![None, None, None]).with_complete_cards().with_complete_items());
    Combat::enter_with_public_knowledge(entry, &mut items, true).unwrap()
}
fn player_effect(kind: EffectKind) -> Effect { Effect { source: Source::Actor(Actor::Player), target: Some(Actor::Player), kind } }
fn create(c: &mut Combat, id: CardId, upgrades: u32, destination: CardDestination) -> u64 {
    piles::apply_effect(&mut c.state, &player_effect(EffectKind::CreateCard { id, destination, upgrades })).unwrap();
    c.state.card_ids_issued.iter().map(|h| h.as_str().parse::<u64>().unwrap()).max().unwrap()
}
fn class_map(state: &State) -> BTreeMap<u64, CardClass> {
    piles::owned_full_cards(state).map(|card| (card.instance.0, card_class(&view::live_card_view(state, card)))).collect()
}
fn top_to_bottom(state: &State) -> Vec<u64> { state.piles.draw.iter().rev().map(|c| c.full().unwrap().instance.0).collect() }
fn handle(i: u64) -> CardHandle { card_state::handle(CardInstanceId(i)) }

/// The real ShuffleDraw (Deep Breath's combined shuffle), with its live card source in play.
fn shuffle(c: &mut Combat) {
    let instance = piles::next_generated_instance(&c.state).unwrap();
    let breath = OwnedCard::Full(complete_live_card(DeepBreath, instance, 0).unwrap());
    let source = Source::Card(breath.handle());
    c.state.card_ids_issued.insert(breath.handle());
    c.state.in_play = Some(breath);
    piles::apply_effect(&mut c.state, &Effect { source, target: None, kind: EffectKind::ShuffleDraw }).unwrap();
    c.state.in_play = None;
    c.state.pending.clear();
}
/// Real Scry of the top `k` with the real selection commit discarding the cards at `discard` depths.
fn scry(c: &mut Combat, k: usize, discard: &[usize]) {
    let eye = OwnedCard::Full(complete_live_card(ThirdEye, CardInstanceId(990), 0).unwrap());
    let parent = eye.handle();
    c.state.in_play = Some(eye);
    // The card's play continuation the Scry selection suspends.
    let phase = std::mem::replace(&mut c.state.phase, Phase::PlayerAction);
    c.state.pending.push_back(Effect { source: Source::Card(parent.clone()), target: None, kind: EffectKind::FinishCard });
    let effect = Effect { source: Source::Card(parent), target: Some(Actor::Player), kind: EffectKind::Draw(0) };
    cards::watcher::scry(&mut c.state, &effect, k as i32, None).unwrap();
    let mut selection = c.state.pending_selection.take().unwrap();
    selection.selected = discard.iter().map(|d| selection.candidates[*d].clone()).collect();
    for next in 0..=selection.selected.len() { cards::watcher::commit_selection(&mut c.state, &effect, &selection, next).unwrap(); }
    c.state.in_play = None;
    c.state.pending.clear();
    c.state.phase = phase;
}
/// Real Stasis capture (uniform over the highest-rarity tier; every test card shares one tier).
fn stasis(c: &mut Combat, n: usize) -> u64 {
    let owner = EnemyHandle::parse(&format!("bronze-orb-test-{n}")).unwrap();
    enemies::act2::stasis::capture(&mut c.state, &owner).unwrap();
    c.state.pending.clear();
    c.state.stasis[&owner].full().unwrap().instance.0
}
/// The Seek grid (shared `sort_draw_grid` order); the player takes the first candidate of `class`.
fn seek(c: &mut Combat, class: CardClass) -> u64 {
    let mut candidates = c.state.piles.draw.iter().map(OwnedCard::handle).collect::<Vec<_>>();
    piles::sort_draw_grid(&c.state, &mut candidates);
    let classes = class_map(&c.state);
    let chosen = candidates.into_iter().find(|h| classes[&h.as_str().parse::<u64>().unwrap()] == class).unwrap();
    piles::move_draw_card(&mut c.state, &chosen, CardDestination::Hand, Removal::Grid).unwrap();
    chosen.as_str().parse().unwrap()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Status { Top(usize), Bottom(usize), No }
#[derive(Clone, Debug)]
enum Op { Shuffle, Draw, Recycle(usize), Scry(usize, Vec<usize>), TopDeck(CardId, u32), BottomDeck(CardId, u32), Insert(CardId, u32), Stasis, Seek(usize), UpgradeBurns }

/// The brute force: every handle arrangement (top to bottom) with its number of histories.
struct Brute { worlds: BTreeMap<Vec<u64>, u128> }
fn permutations(items: &[u64]) -> Vec<Vec<u64>> {
    if items.len() <= 1 { return vec![items.to_vec()] }
    let mut out = vec![];
    for i in 0..items.len() {
        let mut rest = items.to_vec(); let x = rest.remove(i);
        for mut p in permutations(&rest) { p.insert(0, x); out.push(p); }
    }
    out
}
impl Brute {
    fn map(&mut self, mut f: impl FnMut(&Vec<u64>, u128, &mut Vec<(Vec<u64>, u128)>)) {
        let mut next: BTreeMap<Vec<u64>, u128> = BTreeMap::new();
        let mut out = vec![];
        for (arrangement, w) in &self.worlds { f(arrangement, *w, &mut out); }
        for (a, w) in out { *next.entry(a).or_insert(0) += w; }
        let g = next.values().fold(0, |g, w| gcd(g, *w)).max(1);
        self.worlds = next.into_iter().map(|(a, w)| (a, w / g)).collect();
    }
    fn dist(&self, classes: &BTreeMap<u64, CardClass>) -> Dist {
        let total: u128 = self.worlds.values().sum();
        let mut out = Dist::new();
        for (a, w) in &self.worlds {
            let slot = out.entry(a.iter().map(|i| classes[i]).collect()).or_insert(Q(0, 1));
            *slot = slot.add(Q::new(*w, total));
        }
        out
    }
}

/// Apply `op` to the engine and to the brute force (conditioning on its public observation).
fn apply(c: &mut Combat, brute: &mut Brute, op: &Op, n: usize) {
    let classes = class_map(&c.state);
    let (kt, kb) = (c.state.public.known_draw.len(), c.state.public.known_draw_bottom.len());
    match op {
        Op::Shuffle => {
            shuffle(c);
            brute.map(|a, w, out| for p in permutations(a) { out.push((p, w)); });
        }
        Op::Draw => {
            let seen = classes[&top_to_bottom(&c.state)[0]];
            piles::apply_effect(&mut c.state, &player_effect(EffectKind::DrawOne)).unwrap();
            brute.map(|a, w, out| if classes[&a[0]] == seen { out.push((a[1..].to_vec(), w)); });
        }
        Op::Recycle(k) => {
            // Only fresh cards recycle (earlier Scry discards differ by world); retire the rest to exhaust.
            let old = std::mem::take(&mut c.state.piles.discard);
            c.state.piles.exhaust.extend(old);
            let fresh = (0..*k).map(|i| create(c, if i % 2 == 0 { Wound } else { Dazed }, 0, CardDestination::Discard)).collect::<Vec<_>>();
            piles::apply_effect(&mut c.state, &player_effect(EffectKind::Recycle)).unwrap();
            brute.map(|a, w, out| for p in permutations(&fresh) { out.push((p.into_iter().chain(a.iter().copied()).collect(), w)); });
        }
        Op::Scry(k, discard) => {
            let seen = top_to_bottom(&c.state).iter().take(*k).map(|i| classes[i]).collect::<Vec<_>>();
            scry(c, *k, discard);
            brute.map(|a, w, out| if a.iter().take(*k).map(|i| classes[i]).eq(seen.iter().copied()) {
                out.push((a.iter().enumerate().filter(|(d, _)| !discard.contains(d)).map(|(_, i)| *i).collect(), w));
            });
        }
        Op::TopDeck(id, up) => { let h = create(c, *id, *up, CardDestination::Draw); brute.map(|a, w, out| out.push((std::iter::once(h).chain(a.iter().copied()).collect(), w))); }
        Op::BottomDeck(id, up) => { let h = create(c, *id, *up, CardDestination::BottomDraw); brute.map(|a, w, out| out.push((a.iter().copied().chain(std::iter::once(h)).collect(), w))); }
        Op::Insert(id, up) => {
            let h = create(c, *id, *up, CardDestination::ShuffledDraw);
            brute.map(|a, w, out| if a.is_empty() { out.push((vec![h], w)) } else {
                for p in 1..=a.len() { let mut b = a.clone(); b.insert(p, h); out.push((b, w)); }
            });
        }
        Op::Stasis => {
            let taken = stasis(c, n);
            let seen = classes[&taken];
            brute.map(|a, w, out| for i in 0..a.len() { if classes[&a[i]] == seen { let mut b = a.clone(); b.remove(i); out.push((b, w)); } });
        }
        Op::Seek(pick) => {
            let mut present = c.state.piles.draw.iter().map(|o| classes[&o.full().unwrap().instance.0]).collect::<Vec<_>>();
            present.sort(); present.dedup();
            let class = present[pick % present.len()];
            let known = c.state.public.clone();
            let chosen = seek(c, class);
            let status = if let Some(i) = known.known_draw.iter().position(|h| *h == handle(chosen)) { Status::Top(i) }
                else if let Some(i) = known.known_draw_bottom.iter().position(|h| *h == handle(chosen)) { Status::Bottom(i) } else { Status::No };
            let id = |i: u64| (classes[&i].id().as_str(), i);
            brute.map(|a, w, out| {
                // The engine's grid rule in this world: (card id, instance) order, first candidate of `class`.
                let mut sorted = a.clone(); sorted.sort_by_key(|i| id(*i));
                let h = *sorted.iter().find(|i| classes[*i] == class).unwrap();
                let p = a.iter().position(|i| *i == h).unwrap();
                let here = if p < kt { Status::Top(p) } else if a.len() - 1 - p < kb { Status::Bottom(a.len() - 1 - p) } else { Status::No };
                if here == status { let mut b = a.clone(); b.remove(p); out.push((b, w)); }
            });
        }
        Op::UpgradeBurns => {
            // Every card of a kind changes the same way (as Hexaghost's Inferno or Apotheosis): all Burns become Burn+.
            piles::for_each_owned_card_mut(&mut c.state, |card| if card.id == Burn && card.upgrades == 0 { card.upgrades = 1; });
        }
    }
}

struct Rng(u64);
impl Rng { fn next(&mut self, n: usize) -> usize { self.0 ^= self.0 << 13; self.0 ^= self.0 >> 7; self.0 ^= self.0 << 17; (self.0 % n as u64) as usize } }
const KINDS: [(CardId, u32); 4] = [(Wound, 0), (Dazed, 0), (Burn, 0), (Burn, 1)];

/// One generated scenario: a shuffled permanent deck of 1–3 cards, then up to 8 operations (piles stay at most 6).
fn scenario(seed: u64) -> (Combat, Vec<Op>) {
    let mut r = Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1);
    let deck = [Strike, Defend, Bash][..1 + r.next(3)].to_vec();
    let c = fixture(&deck, (seed % 251) as u8);
    let mut len = deck.len();
    let mut ops = vec![];
    while ops.len() < 8 {
        let kind = KINDS[r.next(KINDS.len())];
        let op = match r.next(10) {
            0 => Op::Shuffle, 1 => Op::Draw, 2 => Op::Recycle(1 + r.next(2)),
            3 => { let k = 1 + r.next(3); let d = (0..k.min(len)).filter(|_| r.next(3) == 0).collect(); Op::Scry(k, d) }
            4 => Op::TopDeck(kind.0, kind.1), 5 => Op::BottomDeck(kind.0, kind.1), 6 | 7 => Op::Insert(kind.0, kind.1),
            8 => if r.next(2) == 0 { Op::Stasis } else { Op::UpgradeBurns }, _ => Op::Seek(r.next(6)),
        };
        let delta: isize = match &op {
            Op::Draw | Op::Stasis | Op::Seek(_) => -1, Op::Recycle(k) => *k as isize,
            Op::Scry(_, d) => -(d.len() as isize), Op::TopDeck(..) | Op::BottomDeck(..) | Op::Insert(..) => 1, _ => 0,
        };
        let needs_cards = matches!(op, Op::Draw | Op::Stasis | Op::Seek(_) | Op::Scry(..));
        if (needs_cards && len == 0) || len as isize + delta > 6 { continue }
        len = (len as isize + delta) as usize;
        ops.push(op);
    }
    (c, ops)
}

#[derive(Default, Debug)]
struct Stats { exact: usize, unrepresentable: usize, mismatches: Vec<String>, exact_ops: BTreeMap<&'static str, usize>,
    seen: BTreeMap<String, Dist>, knowledges: Vec<(DrawKnowledge, Dist)>, unsound: Vec<String> }
/// Record a normal form (its Debug text is the digest input) with the brute-force truth; false on a false merge.
fn record(seen: &mut BTreeMap<String, Dist>, k: &DrawKnowledge, truth: Dist) -> bool {
    let key = format!("{k:?}");
    match seen.get(&key) { Some(previous) => *previous == truth, None => { seen.insert(key, truth); true } }
}
fn op_name(op: &Op) -> &'static str {
    match op { Op::Shuffle => "shuffle", Op::Draw => "draw", Op::Recycle(_) => "recycle", Op::Scry(..) => "scry", Op::TopDeck(..) => "topdeck",
        Op::BottomDeck(..) => "bottomdeck", Op::Insert(..) => "insert", Op::Stasis => "uniform_removal", Op::Seek(_) => "grid_removal", Op::UpgradeBurns => "rekey" }
}
/// Check the published known cards at their positions in every component (`knowledge_contains_known_draw_cards`).
fn known_cards_hold(c: &Combat, k: &DrawKnowledge) -> bool {
    let classes = class_map(&c.state);
    let class = |h: &CardHandle| classes[&h.as_str().parse::<u64>().unwrap()];
    expand(k).keys().all(|seq| c.state.public.known_draw.iter().enumerate().all(|(i, h)| seq.get(i) == Some(&class(h)))
        && c.state.public.known_draw_bottom.iter().enumerate().all(|(i, h)| seq.len() > i && seq[seq.len() - 1 - i] == class(h)))
}
fn run_oracle(variant: Option<Variant>, scenarios: u64) -> Stats {
    VARIANT.with(|v| v.set(variant));
    let mut stats = Stats::default();
    for seed in 0..scenarios {
        let (mut c, ops) = scenario(seed);
        let mut brute = Brute { worlds: BTreeMap::new() };
        brute.worlds.insert(top_to_bottom(&c.state), 1);
        brute.map(|a, w, out| for p in permutations(a) { out.push((p, w)); });
        for (n, op) in ops.iter().enumerate() {
            apply(&mut c, &mut brute, op, n);
            match c.draw_knowledge() {
                Ok(k) => {
                    let (want, got) = (brute.dist(&class_map(&c.state)), expand(&k));
                    let fresh = !stats.seen.contains_key(&format!("{k:?}"));
                    if !record(&mut stats.seen, &k, want.clone()) { stats.unsound.push(format!("seed {seed} after {:?}", &ops[..=n])); }
                    if fresh { stats.knowledges.push((k.clone(), want.clone())); }
                    if want != got || !known_cards_hold(&c, &k) {
                        stats.mismatches.push(format!("seed {seed} after {:?}: brute {} vs knowledge {} outcomes", &ops[..=n], want.len(), got.len()));
                        break;
                    }
                    stats.exact += 1;
                    *stats.exact_ops.entry(op_name(op)).or_insert(0) += 1;
                }
                Err(Unrepresentable) => stats.unrepresentable += 1,
            }
        }
    }
    VARIANT.with(|v| v.set(None));
    stats
}

#[test]
fn draw_knowledge_matches_brute_force_posterior() {
    let stats = run_oracle(None, 600);
    assert!(stats.mismatches.is_empty(), "{:#?}", stats.mismatches);
    println!("exact {} unrepresentable {} per op {:?}", stats.exact, stats.unrepresentable, stats.exact_ops);
    assert!(stats.exact > 4 * stats.unrepresentable, "{stats:?}");
    for op in ["shuffle", "draw", "recycle", "scry", "topdeck", "bottomdeck", "insert", "uniform_removal", "grid_removal", "rekey"] {
        assert!(stats.exact_ops.get(op).copied().unwrap_or(0) >= 20, "operation {op} is rarely exact: {stats:?}");
    }
}
#[test]
fn knowledge_without_top_exclusion_fails_the_oracle() {
    assert!(!run_oracle(Some(Variant::NoTopExclusion), 300).mismatches.is_empty());
}
#[test]
fn knowledge_with_v1_prefix_truncation_fails_the_oracle() {
    assert!(!run_oracle(Some(Variant::V1PrefixTruncation), 300).mismatches.is_empty());
}

/// Seek on {W_old, Strike, W_new}: `W_new` is shuffled in under an unknown top, the first Wound candidate is `W_old`.
fn seek_case(variant: Option<Variant>) -> (Result<DrawKnowledge, Unrepresentable>, Dist, Combat) {
    VARIANT.with(|v| v.set(variant));
    let mut c = fixture(&[Strike], 7);
    let mut brute = Brute { worlds: BTreeMap::from([(top_to_bottom(&c.state), 1)]) };
    apply(&mut c, &mut brute, &Op::TopDeck(Wound, 0), 0);
    apply(&mut c, &mut brute, &Op::Shuffle, 1);
    apply(&mut c, &mut brute, &Op::Insert(Wound, 0), 2);
    let wound = class_map(&c.state).into_iter().find(|(_, k)| k.id() == Wound).unwrap().1;
    let classes = class_map(&c.state);
    let mut present = c.state.piles.draw.iter().map(|o| classes[&o.full().unwrap().instance.0]).collect::<Vec<_>>();
    present.sort(); present.dedup();
    let pick = present.iter().position(|k| *k == wound).unwrap();
    let old = c.state.piles.draw.iter().map(|o| o.full().unwrap().instance.0).filter(|i| classes[i] == wound).min().unwrap();
    apply(&mut c, &mut brute, &Op::Seek(pick), 3);
    assert!(!c.state.piles.draw.iter().any(|o| o.full().unwrap().instance.0 == old), "the grid removes W_old");
    let knowledge = c.draw_knowledge();
    VARIANT.with(|v| v.set(None));
    (knowledge, brute.dist(&class_map(&c.state)), c)
}
#[test]
fn grid_removal_of_a_top_excluded_class_matches_the_engine() {
    let (knowledge, truth, mut c) = seek_case(None);
    let strike = |seq: &Vec<CardClass>| seq[0].id() == Strike;
    let strike_top = truth.iter().filter(|(s, _)| strike(s)).fold(Q(0, 1), |a, (_, q)| a.add(*q));
    // Engine-differential truth: W_old (lowest instance) leaves; W_new was never on top.
    assert_eq!(strike_top, Q(3, 4), "{truth:?}");
    assert_eq!(knowledge, Err(Unrepresentable), "a top-excluded class under a deterministic grid rule fails closed");
    // Still unrepresentable until the next shuffle.
    piles::apply_effect(&mut c.state, &player_effect(EffectKind::DrawOne)).unwrap();
    assert_eq!(c.draw_knowledge(), Err(Unrepresentable));
    create(&mut c, Dazed, 0, CardDestination::Discard);
    shuffle(&mut c);
    assert!(c.draw_knowledge().is_ok());
}
#[test]
fn knowledge_with_uniform_grid_removal_fails_the_seek_case() {
    let (knowledge, truth, _) = seek_case(Some(Variant::UniformGrid));
    assert_ne!(expand(&knowledge.expect("the uniform variant claims an exact answer")), truth);
}
#[test]
fn uniform_removal_of_an_excluded_class_splits_by_counts() {
    // {Wound, Strike} shuffled, a second Wound inserted (top-excluded), then a uniform Stasis removal reveals a Wound:
    // the excluded copy (weight 1) keeps the first card; the non-excluded copy (weight 1) was first with probability 1/2.
    for seed in 0..40 {
        let mut c = fixture(&[Strike], seed);
        let mut brute = Brute { worlds: BTreeMap::from([(top_to_bottom(&c.state), 1)]) };
        for (n, op) in [Op::TopDeck(Wound, 0), Op::Shuffle, Op::Insert(Wound, 0), Op::Stasis].iter().enumerate() { apply(&mut c, &mut brute, op, n); }
        let truth = brute.dist(&class_map(&c.state));
        let knowledge = c.draw_knowledge().expect("uniform removal of an excluded class is exact");
        assert_eq!(expand(&knowledge), truth, "seed {seed}");
        if c.state.stasis.values().next().unwrap().full().unwrap().id == Wound {
            let strike_top = truth.iter().filter(|(s, _)| s[0].id() == Strike).fold(Q(0, 1), |a, (_, q)| a.add(*q));
            assert_eq!(strike_top, Q(5, 8), "excluded copy 1/2·1/2 + non-excluded 1/2·(1/2·1/2 + 1/2·1)");
            return;
        }
    }
    panic!("no seed removed a Wound");
}
#[test]
fn interchangeable_grid_removal_stays_exact() {
    for seed in 0..12 {
        // All copies in one bag, not excluded.
        let mut c = fixture(&[Strike, Defend], seed);
        let mut brute = Brute { worlds: BTreeMap::from([(top_to_bottom(&c.state), 1)]) };
        brute.map(|a, w, out| for p in permutations(a) { out.push((p, w)); });
        for (n, op) in [Op::TopDeck(Wound, 0), Op::TopDeck(Wound, 0), Op::Shuffle].iter().enumerate() { apply(&mut c, &mut brute, op, n); }
        let wound = class_map(&c.state).into_values().filter(|k| k.id() == Wound).next().unwrap();
        let classes = class_map(&c.state);
        let mut present = c.state.piles.draw.iter().map(|o| classes[&o.full().unwrap().instance.0]).collect::<Vec<_>>();
        present.sort(); present.dedup();
        apply(&mut c, &mut brute, &Op::Seek(present.iter().position(|k| *k == wound).unwrap()), 3);
        assert_eq!(expand(&c.draw_knowledge().unwrap()), brute.dist(&class_map(&c.state)), "seed {seed}");
        // All copies excluded: two Wounds inserted into a shuffled pile.
        let mut c = fixture(&[Strike, Defend], seed);
        let mut brute = Brute { worlds: BTreeMap::from([(top_to_bottom(&c.state), 1)]) };
        brute.map(|a, w, out| for p in permutations(a) { out.push((p, w)); });
        for (n, op) in [Op::Insert(Wound, 0), Op::Insert(Wound, 0)].iter().enumerate() { apply(&mut c, &mut brute, op, n); }
        let classes = class_map(&c.state);
        let mut present = c.state.piles.draw.iter().map(|o| classes[&o.full().unwrap().instance.0]).collect::<Vec<_>>();
        present.sort(); present.dedup();
        apply(&mut c, &mut brute, &Op::Seek(present.iter().position(|k| k.id() == Wound).unwrap()), 2);
        assert_eq!(expand(&c.draw_knowledge().unwrap()), brute.dist(&class_map(&c.state)), "all-excluded seed {seed}");
    }
}
#[test]
fn knowledge_normal_form_is_canonical() {
    // Uniform removal of a bag's card splits into two components that merge: equal to a direct shuffle of the rest.
    let mut a = fixture(&[Strike, Defend, Bash], 3);
    stasis(&mut a, 0);
    let removed = a.state.stasis.values().next().unwrap().full().unwrap().id;
    let rest = [Strike, Defend, Bash].into_iter().filter(|id| *id != removed).collect::<Vec<_>>();
    let mut b = fixture(&[Strike, Defend, Bash], 4);
    let gone = b.state.piles.draw.iter().position(|o| o.full().unwrap().id == removed).unwrap();
    let card = b.state.piles.draw.remove(gone);
    b.state.piles.discard.push(card);
    shuffle(&mut b);
    let (ka, kb) = (a.draw_knowledge().unwrap(), b.draw_knowledge().unwrap());
    assert_eq!(ka.components().len(), 1);
    assert_eq!((ka.classes().len(), rest.len()), (2, 2));
    assert_eq!(ka, kb);
    assert_eq!(format!("{ka:?}"), format!("{kb:?}"), "equal digest input");
    // A re-key that merges classes, with different interning orders and weight scales.
    let mut c = fixture(&[Strike], 5);
    create(&mut c, Burn, 0, CardDestination::Draw);
    create(&mut c, Burn, 1, CardDestination::BottomDraw);
    shuffle(&mut c);
    create(&mut c, Wound, 0, CardDestination::Discard);
    piles::apply_effect(&mut c.state, &player_effect(EffectKind::DrawOne)).unwrap();
    piles::for_each_owned_card_mut(&mut c.state, |card| if card.id == Burn { card.upgrades = 1; });
    let mut d = fixture(&[Strike], 6);
    create(&mut d, Burn, 1, CardDestination::BottomDraw);
    create(&mut d, Burn, 1, CardDestination::Draw);
    shuffle(&mut d);
    piles::apply_effect(&mut d.state, &player_effect(EffectKind::DrawOne)).unwrap();
    let drawn = |x: &Combat| x.state.piles.hand.last().unwrap().full().unwrap().id;
    if drawn(&c) == drawn(&d) {
        assert_eq!(c.draw_knowledge().unwrap(), d.draw_knowledge().unwrap());
        // Core equality compares the synced normal form only (the physical bookkeeping differs).
        knowledge::note(&mut c.state, knowledge::PileOp::Sync);
        knowledge::note(&mut d.state, knowledge::PileOp::Sync);
        assert_eq!(c.state.knowledge, d.state.knowledge);
    }
    // An insertion into a two-card bag has raw weight 2 in its only component; the normal form reduces it to 1.
    let mut e = fixture(&[Strike, Defend], 8);
    create(&mut e, Wound, 0, CardDestination::ShuffledDraw);
    let ke = e.draw_knowledge().unwrap();
    assert_eq!(ke.components().iter().map(|c| c.weight).collect::<Vec<_>>(), vec![1]);
    // Each knowledge is in normal form: sorted classes, gcd-reduced weights, sorted distinct components.
    for k in [&ka, &ke, &c.draw_knowledge().unwrap()] {
        assert!(k.classes().windows(2).all(|w| w[0] < w[1]));
        assert_eq!(k.components().iter().fold(0, |g, c| gcd(g, c.weight)), 1);
        assert!(k.components().windows(2).all(|w| w[0].segments < w[1].segments));
    }
}
#[test]
fn inserted_card_is_never_sampled_on_top() {
    let mut c = fixture(&[Strike, Defend], 9);
    create(&mut c, Wound, 0, CardDestination::ShuffledDraw);
    let dist = expand(&c.draw_knowledge().unwrap());
    assert!(dist.keys().all(|seq| seq[0].id() != Wound), "{dist:?}");
    assert_eq!(dist.len(), 4);
    assert!(dist.values().all(|q| *q == Q(1, 4)));
}
#[test]
fn insertion_into_known_prefix_keeps_exact_support() {
    let mut c = fixture(&[Strike, Defend, Bash, Anger, Clothesline], 11);
    let mut brute = Brute { worlds: BTreeMap::from([(top_to_bottom(&c.state), 1)]) };
    brute.map(|a, w, out| for p in permutations(a) { out.push((p, w)); });
    apply(&mut c, &mut brute, &Op::Scry(3, vec![]), 0);
    let second = class_map(&c.state)[&top_to_bottom(&c.state)[1]];
    apply(&mut c, &mut brute, &Op::Insert(Dazed, 0), 1);
    let dist = expand(&c.draw_knowledge().unwrap());
    assert_eq!(dist, brute.dist(&class_map(&c.state)));
    assert!(dist.keys().all(|seq| seq[1] == second || seq[2] == second), "the second known card is only at depth 2 or 3");
    assert!(dist.keys().any(|seq| seq[2] == second));
    assert_eq!(c.state.public.known_draw.len(), 1, "the published prefix is truncated (L6: the knowledge is finer)");
}
#[test]
fn knowledge_contains_known_draw_cards() {
    // The oracle checks the published known cards at every exact step; here the published prefix and suffix are long.
    let mut c = fixture(&[Strike, Defend, Bash, Anger], 13);
    scry(&mut c, 2, &[]);
    create(&mut c, Wound, 0, CardDestination::BottomDraw);
    create(&mut c, Dazed, 0, CardDestination::Draw);
    let k = c.draw_knowledge().unwrap();
    assert_eq!((c.state.public.known_draw.len(), c.state.public.known_draw_bottom.len()), (3, 1));
    assert!(known_cards_hold(&c, &k));
    let stats = run_oracle(None, 120);
    assert!(stats.mismatches.is_empty() && stats.exact > 0);
}
#[test]
fn knowledge_is_handle_free() {
    // Different true arrangements and instance labels under the same public history give equal knowledge.
    let ops = |c: &mut Combat| {
        create(c, Wound, 0, CardDestination::ShuffledDraw);
        create(c, Dazed, 0, CardDestination::Draw);
        create(c, Wound, 0, CardDestination::ShuffledDraw);
        create(c, Burn, 0, CardDestination::BottomDraw);
        piles::for_each_owned_card_mut(&mut c.state, |card| if card.id == Burn { card.upgrades = 1; });
    };
    let mut seen = BTreeMap::new();
    for seed in 0..8 {
        let mut c = fixture(&[Strike, Defend, Bash], seed);
        ops(&mut c);
        seen.insert(top_to_bottom(&c.state), c.draw_knowledge().unwrap());
    }
    assert!(seen.len() > 1, "the true arrangements differ");
    assert!(seen.values().all(|k| *k == *seen.values().next().unwrap()));
}
#[test]
fn knowledge_over_bound_is_unrepresentable_until_shuffle() {
    let mut c = fixture(&[Strike, Defend], 17);
    for _ in 0..40 { create(&mut c, Wound, 0, CardDestination::Draw); }
    let components = |c: &Combat| c.draw_knowledge().map(|k| k.components().len());
    for _ in 0..3 { create(&mut c, Dazed, 0, CardDestination::ShuffledDraw); }
    assert!(components(&c).unwrap() > 10_000, "{:?}", components(&c));
    create(&mut c, Dazed, 0, CardDestination::ShuffledDraw);
    assert_eq!(components(&c), Err(Unrepresentable), "over N_COMPONENTS = {}", knowledge::N_COMPONENTS);
    piles::apply_effect(&mut c.state, &player_effect(EffectKind::DrawOne)).unwrap();
    assert_eq!(components(&c), Err(Unrepresentable), "until the next shuffle");
    shuffle(&mut c);
    assert_eq!(components(&c), Ok(1));
}
#[test]
fn unmapped_draw_pile_change_is_unrepresentable() {
    let mut c = fixture(&[Strike, Defend, Bash], 19);
    c.state.piles.draw.swap(0, 2);
    assert_eq!(c.draw_knowledge(), Err(Unrepresentable), "a physical change no operation explains");
    let mut c = fixture(&[Strike], 19);
    create(&mut c, Burn, 0, CardDestination::Draw);
    create(&mut c, Burn, 0, CardDestination::Draw);
    let first = c.state.piles.draw.iter_mut().find(|o| o.full().is_some_and(|l| l.id == Burn)).unwrap();
    first.full_mut().unwrap().upgrades = 1;
    assert_eq!(c.draw_knowledge(), Err(Unrepresentable), "copies of one class changing differently");
}

// ---------- fail-closed tie guard (PM item-5 verdicts) ----------
fn tie_case(mutate: impl Fn(&mut State)) -> Result<DrawKnowledge, Unrepresentable> {
    let mut c = fixture(&[Strike, Defend, Bash], 23);
    create(&mut c, Wound, 0, CardDestination::ShuffledDraw);
    create(&mut c, Wound, 0, CardDestination::ShuffledDraw);
    assert!(c.draw_knowledge().is_ok());
    mutate(&mut c.state);
    c.draw_knowledge()
}
fn unknown_wounds(state: &mut State) -> Vec<&mut card_state::LiveCard> {
    state.piles.draw.iter_mut().filter_map(OwnedCard::full_mut).filter(|l| l.id == Wound).collect()
}
#[test]
fn tie_guard_fails_closed_on_purge_or_rebound() {
    assert_eq!(tie_case(|s| unknown_wounds(s)[0].local.purge = true), Err(Unrepresentable));
    assert_eq!(tie_case(|s| unknown_wounds(s)[0].local.rebound_on_finish = true), Err(Unrepresentable));
}
#[test]
fn tie_guard_fails_closed_on_a_shared_copy_family() {
    assert_eq!(tie_case(|s| { let mut w = unknown_wounds(s); let f = w[0].copy_family; w[1].copy_family = f; }), Err(Unrepresentable));
    assert_eq!(tie_case(|s| { let f = s.piles.draw.iter().find_map(|o| o.full().filter(|l| l.id == Strike)).unwrap().copy_family;
        s.piles.hand.push(OwnedCard::Full(card_state::generated(Wound, CardInstanceId(970), 0).map(|mut l| { l.copy_family = f; l }).unwrap())); }),
        Err(Unrepresentable), "a family shared with a card outside the draw pile");
}
#[test]
fn tie_guard_fails_closed_on_an_m21_difference_within_a_class() {
    assert_eq!(tie_case(|s| unknown_wounds(s)[0].local.m21 = true), Err(Unrepresentable));
    assert!(tie_case(|s| for w in unknown_wounds(s) { w.local.m21 = true; }).is_ok(), "equal m21 within the class");
}

// ---------- pre-roll consistent sets (design §4.5.2) ----------
/// A Two Louse fight whose first louse has the wanted colour; both bite with `base`; `powers` start on the first louse.
fn louse_combat(red: bool, base: i32, ascension: u8, powers: &[(&'static str, i32)], player_powers: &[&'static str]) -> Combat {
    let want = if red { EnemyId::LouseRed } else { EnemyId::LouseGreen };
    let power = |id: &'static str, amount| FullPowerEntry { id: FullPowerId(id), state: PowerState { amount, skip_first: false, monster_source_grace: false } };
    for seed in 0..=255u8 {
        let mut entry = enemies::setup(EncounterId::TwoLouse, 1, ascension, &mut RunRng::new([seed; 32], rng::Partition::Test, 0), enemies::SetupContext {
            player: PlayerState { hp: 80, max_hp: 80, block: 0, energy: 3, max_energy: 3, draw_per_turn: 5, powers: vec![] },
            player_powers: player_powers.iter().map(|id| power(id, 1)).collect(),
            character: Character::Ironclad, cards: vec![permanent_live_card(Defend, CardInstanceId(99), 0)], opening_draw: 0, encounter_visit: 0 }).unwrap();
        if entry.setup.enemies[0].enemy_id != want { continue }
        entry.cards = vec![complete_live_card(Strike, CardInstanceId(1), 0).unwrap()];
        for enemy in &mut entry.setup.enemies { enemy.intent = FullIntentId::Bite; }
        entry.setup.enemies[0].powers.extend(powers.iter().map(|(id, n)| power(id, *n)));
        for (_, value) in entry.setup.enemy_damage_bases.iter_mut().filter(|(k, _)| k.1 == FullIntentId::Bite) { *value = base; }
        let mut items = Some(CombatItemContext::new(vec![], vec![None, None, None]).with_complete_cards().with_complete_items());
        return Combat::enter_with_public_knowledge(entry, &mut items, true).unwrap()
    }
    panic!("no Two Louse seed with the wanted colour first")
}
fn set_of(c: &Combat, kind: PrerollKind) -> Result<ConsistentSet, Unrepresentable> {
    let first = &c.state.enemies[0].instance_id;
    c.preroll_sets().into_iter().find(|(h, k, _)| *k == kind && h == first).unwrap().2
}
fn values(c: &Combat, kind: PrerollKind) -> Vec<i32> { set_of(c, kind).unwrap().values().collect() }
fn enemy_power(c: &mut Combat, id: &'static str, amount: i32) {
    let enemy = &mut c.state.enemies[0];
    enemy.full_powers.retain(|p| p.id.0 != id);
    enemy.full_powers.push(FullPowerEntry { id: FullPowerId(id), state: PowerState { amount, skip_first: false, monster_source_grace: false } });
}
fn refresh(c: &mut Combat) { effects::refresh_enemy_values(&mut c.state).unwrap(); }
fn shown(c: &Combat) -> i32 { c.public_full().unwrap().enemies[0].intent_damage.unwrap() }

#[test]
fn unique_display_pins_the_base() {
    for base in 5..=7 {
        let c = louse_combat(false, base, 0, &[], &[]);
        assert_eq!(shown(&c), base);
        assert_eq!(values(&c, PrerollKind::LouseBite), vec![base]);
    }
}
#[test]
fn seen_bite_under_weak_collision_keeps_the_consistent_set() {
    for base in [6, 7] {
        let c = louse_combat(false, base, 0, &[("strength", 2), ("weak", 1)], &[]);
        assert_eq!(shown(&c), 6, "Bite {base} with Strength 2 and Weak");
        assert_eq!(values(&c, PrerollKind::LouseBite), vec![6, 7]);
    }
}
#[test]
fn red_louse_grow_and_weak_collision() {
    // After one Red Louse Grow (+3 Strength at A0) and under Weak: Bite 5 and 6 both show 6; Bite 7 shows 7.
    for (base, want) in [(5, vec![5, 6]), (6, vec![5, 6]), (7, vec![7])] {
        let c = louse_combat(true, base, 0, &[("strength", 3), ("weak", 2)], &[]);
        assert_eq!(values(&c, PrerollKind::LouseBite), want, "base {base}");
    }
}
#[test]
fn intangible_display_keeps_the_full_law() {
    for ascension in [0, 2] {
        let c = louse_combat(false, 7, ascension, &[], &["intangible_player"]);
        assert_eq!(shown(&c), 1);
        let (lo, hi) = enemies::ascension::louse_bite_range(ascension);
        assert_eq!(values(&c, PrerollKind::LouseBite), (lo..=hi).collect::<Vec<_>>());
    }
}
/// ASSUMPTION (final review M2, carried to M34s-b b0): this test writes Strength WITHOUT the refresh every engine power
/// application performs (`powers.rs`), so the hit deals the stale stored `output` while execution-time narrowing uses the
/// display recomputed under the modifiers now in force. In real play output is fresh at execution, so the two agree; b0 must
/// either narrow on the dealt `output` (or assert it is fresh) and fix this premise before b samples from `C_v`, and must
/// treat hits whose observable is masked (Torii, Buffer) as non-revealing.
#[test]
fn consistent_set_updates_at_execution_after_an_ally_buff() {
    let mut c = louse_combat(false, 7, 0, &[("strength", 2), ("weak", 1)], &[]);
    assert_eq!(values(&c, PrerollKind::LouseBite), vec![6, 7]);
    // An ally's buff lands after the last refresh; the hit runs with the modifiers then in force (Strength 3: 6 vs 7).
    enemy_power(&mut c, "strength", 3);
    assert_eq!(values(&c, PrerollKind::LouseBite), vec![6, 7], "no refresh yet");
    let louse = c.state.enemies[0].instance_id.clone();
    let hit = Effect { source: Source::Actor(Actor::Enemy(louse)), target: Some(Actor::Player), kind: EffectKind::Damage(DamagePayload::FullEnemyMove(FullIntentId::Bite)) };
    effects::apply_effect(&mut c.state, &hit).unwrap();
    assert_eq!(values(&c, PrerollKind::LouseBite), vec![7]);
}
#[test]
fn nip_consistent_set_and_both_stores_agree() {
    let nip = FullIntentId::from_name("darkling_nip");
    let mut e = enemies::setup(EncounterId::ThreeDarklings, 3, 0, &mut RunRng::new([0x61; 32], rng::Partition::Test, 0), enemies::SetupContext {
        player: PlayerState { hp: 300, max_hp: 300, block: 0, energy: 3, max_energy: 3, draw_per_turn: 5, powers: vec![] }, player_powers: vec![],
        character: Character::Ironclad, cards: vec![permanent_live_card(Defend, CardInstanceId(99), 0)], opening_draw: 0, encounter_visit: 0 }).unwrap();
    for enemy in &mut e.setup.enemies { enemy.intent = nip; }
    let mut c = Combat::enter_with_public_knowledge(e, &mut None, true).unwrap_or_else(|_| panic!("entry"));
    for enemy in &c.state.enemies {
        let Some(EnemyMemory::Darkling { nip: stored, .. }) = enemy.full_memory else { panic!("Darkling memory") };
        assert_eq!(stored, enemy.full_damage_entries[&nip].base, "both stores agree");
    }
    let sets = c.preroll_sets();
    assert_eq!(sets.len(), 3);
    for (handle, kind, set) in &sets {
        let base = c.state.enemies.iter().find(|e| e.instance_id == *handle).unwrap().full_damage_entries[&nip].base;
        assert_eq!((*kind, set.unwrap().values().collect::<Vec<_>>()), (PrerollKind::DarklingNip, vec![base]));
    }
    // The runtime invariant: stores that disagree make that actor's set unrepresentable.
    if let Some(EnemyMemory::Darkling { nip: stored, .. }) = &mut c.state.enemies[0].full_memory { *stored += 1; }
    assert_eq!(engine::resolve(&mut c.state).status, TransitionStatus::Decision, "the next published boundary");
    let first = c.state.enemies[0].instance_id.clone();
    assert!(c.preroll_sets().iter().all(|(h, _, s)| (*h == first) == s.is_err()));
}

// ---------- real play: the knowledge never excludes the true pile ----------
#[test]
fn knowledge_support_contains_the_true_pile_in_play() {
    let deck = [Havoc, Warcry, Headbutt, WildStrike, RecklessCharge, Anger, PommelStrike, Seek, ThinkingAhead, Forethought,
        SecretWeapon, Violence, DeepBreath, Strike, Strike, Defend, Defend, Bash];
    let (mut exact, mut steps, mut generated) = (0, 0, 0);
    for seed in 0..96u8 {
        let mut c = fixture(&deck, seed);
        let mut r = Rng(u64::from(seed) * 7919 + 1);
        for _ in 0..200 {
            let Ok(actions) = c.legal_actions_full() else { break };
            if actions.is_empty() || c.state.outcome.is_some() { break }
            let action = actions[r.next(actions.len())].clone();
            if c.step_full(&action).is_err() { break }
            steps += 1;
            if let Ok(k) = c.draw_knowledge() {
                exact += 1;
                let classes = class_map(&c.state);
                let truth = top_to_bottom(&c.state).iter().map(|i| classes[i]).collect::<Vec<_>>();
                assert_eq!(c.true_draw_classes(), truth);
                assert!(k.supports(&truth), "seed {seed}: the true pile is outside the knowledge after {action:?}");
                generated += usize::from(c.state.piles.draw.iter().any(|o| o.full().is_some_and(|l| matches!(l.origin, CombatCardRef::CombatLocal(_)))));
            }
        }
    }
    println!("real play: {exact} exact of {steps} decisions, {generated} with a generated card in the draw pile");
    assert!(steps > 1000 && exact * 10 > steps * 8 && generated > 100, "{exact} of {steps}, {generated}");
}

/// Main's ruling (a2 v2): `C_v` is a function of the published decision views and the executions only. A display that is
/// computed mid-resolution and never published (here Strength 3 for one effect, which would separate Bite 6 from 7) must
/// not narrow the set.
#[test]
fn preroll_sets_ignore_unpublished_refreshes() {
    let strength = |louse: &EnemyHandle, amount: i32| Effect { source: Source::Actor(Actor::Enemy(louse.clone())), target: Some(Actor::Enemy(louse.clone())),
        kind: EffectKind::ApplyFullPower { id: FullPowerId("strength"), amount } };
    let mut published = vec![];
    for swing in [false, true] {
        let mut c = louse_combat(false, 7, 0, &[("strength", 2), ("weak", 1)], &[]);
        let louse = c.state.enemies[0].instance_id.clone();
        if swing { c.state.pending.extend([strength(&louse, 1), strength(&louse, -1)]); }
            let transition = engine::resolve(&mut c.state);
        assert_eq!(transition.status, TransitionStatus::Decision);
        published.push((c.public_full().unwrap().enemies, values(&c, PrerollKind::LouseBite)));
    }
    assert_eq!(published[0].0, published[1].0, "identical published decision views");
    assert_eq!(published[0].1, vec![6, 7]);
    assert_eq!(published[1].1, published[0].1, "an unpublished mid-resolution display does not narrow C_v");
}

/// PM ruling on R-M1: equal normal forms (and equal digest input) only for equal distributions. Over the oracle's states
/// and constructed pairs (the bag-local rules included), every equal normal form has one brute-force distribution.
#[test]
fn knowledge_equality_is_sound() {
    let stats = run_oracle(None, 300);
    assert!(stats.unsound.is_empty(), "{:#?}", stats.unsound);
    // Core equality (`==`) between distinct normal forms must also imply equal distributions.
    for (i, (a, da)) in stats.knowledges.iter().enumerate() {
        for (b, db) in &stats.knowledges[i + 1..] { assert!(a != b || da == db, "false merge by ==: {a:?} vs {b:?}"); }
    }
    let mut seen = stats.seen;
    let mut equal_pairs = 0;
    let mut check = |c: &Combat, brute: &Brute| {
        let k = c.draw_knowledge().unwrap();
        let fresh = !seen.contains_key(&format!("{k:?}"));
        assert!(record(&mut seen, &k, brute.dist(&class_map(&c.state))), "false merge: {k:?}");
        equal_pairs += usize::from(!fresh);
        k
    };
    let start = |deck: &[CardId], seed: u8| {
        let c = fixture(deck, seed);
        let mut brute = Brute { worlds: BTreeMap::from([(top_to_bottom(&c.state), 1)]) };
        brute.map(|a, w, out| for p in permutations(a) { out.push((p, w)); });
        (c, brute)
    };
    let seek_class = |c: &Combat, id: CardId| {
        let classes = class_map(&c.state);
        let mut present = c.state.piles.draw.iter().map(|o| classes[&o.full().unwrap().instance.0]).collect::<Vec<_>>();
        present.sort(); present.dedup();
        present.iter().position(|k| k.id() == id).unwrap()
    };
    // Single-class bag ⇒ known cards: a recycled {Wound, Wound} equals two topdecked Wounds.
    for recycled in [true, false] {
        let (mut c, mut brute) = start(&[Strike], 31);
        apply(&mut c, &mut brute, &Op::Draw, 0);
        if recycled {
            let fresh = [create(&mut c, Wound, 0, CardDestination::Discard), create(&mut c, Wound, 0, CardDestination::Discard)];
            piles::apply_effect(&mut c.state, &player_effect(EffectKind::Recycle)).unwrap();
            assert!(c.state.piles.discard.is_empty() && c.state.piles.draw.len() == 2, "a shuffled bag of two Wounds");
            brute.map(|a, w, out| for p in permutations(&fresh) { out.push((p.into_iter().chain(a.iter().copied()).collect(), w)); });
        }
        else { apply(&mut c, &mut brute, &Op::TopDeck(Wound, 0), 1); apply(&mut c, &mut brute, &Op::TopDeck(Wound, 0), 2); }
        let k = check(&c, &brute);
        if !recycled { assert!(k.components()[0].segments.iter().all(|s| matches!(s, Segment::Known(_)))); }
    }
    // Only one non-excluded class ⇒ Known + bag: {Strike, Defend} + inserted Wound, then the grid takes Defend
    // (the not-first branch leaves {Strike, Wound} with Wound excluded ⇒ [Strike, Wound]); two seeds, same normal form.
    for seed in [41, 42] {
        let (mut c, mut brute) = start(&[Strike, Defend], seed);
        apply(&mut c, &mut brute, &Op::Insert(Wound, 0), 0);
        let pick = seek_class(&c, Defend);
        apply(&mut c, &mut brute, &Op::Seek(pick), 1);
        let k = check(&c, &brute);
        assert!(k.components().iter().any(|c| c.segments.iter().all(|s| matches!(s, Segment::Known(_)))), "{k:?}");
    }
    // An inserted card (top-excluded) and a shuffled-in card have equal multisets and different distributions.
    let (mut c, mut brute) = start(&[Strike, Defend], 43);
    apply(&mut c, &mut brute, &Op::Insert(Wound, 0), 0);
    let inserted = check(&c, &brute);
    let (mut c, mut brute) = start(&[Strike, Defend], 44);
    apply(&mut c, &mut brute, &Op::TopDeck(Wound, 0), 0);
    apply(&mut c, &mut brute, &Op::Shuffle, 1);
    let shuffled = check(&c, &brute);
    assert_ne!(inserted, shuffled);
    assert!(equal_pairs >= 2, "the constructed pairs met equal normal forms");
}
#[test]
fn supports_is_positive_probability() {
    let mut c = fixture(&[Strike, Defend], 9);
    create(&mut c, Wound, 0, CardDestination::ShuffledDraw);
    let k = c.draw_knowledge().unwrap();
    let dist = expand(&k);
    let classes = k.classes().to_vec();
    // Every ordering of the three classes: supported iff it has positive probability.
    for p in permutations(&[0, 1, 2]) {
        let seq = p.iter().map(|i| classes[*i as usize]).collect::<Vec<_>>();
        assert_eq!(k.supports(&seq), dist.contains_key(&seq), "{seq:?}");
    }
    assert!(k.supports(&c.true_draw_classes()));
    assert!(!k.supports(&c.true_draw_classes()[1..]), "a shorter sequence");
    assert_eq!(k.components().len(), 1, "component count is components().len()");
}

/// Final review M7 (PM ruling): knowledge maintenance is gated by one entry flag. The same combat entered with and without it
/// plays identically (every public view apart from the published draw multiset, and every legal-action list, along a fixed
/// policy are equal), and without it every read answers "no knowledge" (never stale) and no maintenance work runs.
#[test]
fn knowledge_maintenance_flag_changes_nothing_but_the_knowledge() {
    let deck = [Havoc, Warcry, Headbutt, WildStrike, Anger, PommelStrike, Seek, ThinkingAhead, Forethought, Strike, Defend, Bash];
    for seed in 1..=6u8 {
        let enter = |maintained: bool| {
            let mut c = fixture(&deck, seed);
            if !maintained {
                let mut entry = enemies::setup(EncounterId::Cultist, 1, 0, &mut RunRng::new([seed; 32], rng::Partition::Test, 0), enemies::SetupContext {
                    player: PlayerState { hp: 80, max_hp: 80, block: 0, energy: 3, max_energy: 3, draw_per_turn: 5, powers: vec![] }, player_powers: vec![],
                    character: Character::Ironclad, cards: vec![permanent_live_card(Defend, CardInstanceId(99), 0)], opening_draw: 0, encounter_visit: 0 }).unwrap();
                entry.cards = deck.iter().enumerate().map(|(i, id)| complete_live_card(*id, CardInstanceId(i as u64 + 1), 0).unwrap()).collect();
                entry.opening_draw = 0;
                let mut items = Some(CombatItemContext::new(vec![], vec![None, None, None]).with_complete_cards().with_complete_items());
                c = Combat::enter_with_item_context(entry, &mut items).unwrap();
            }
            c
        };
        let (mut on, mut off) = (enter(true), enter(false));
        assert!(on.maintains_public_knowledge() && !off.maintains_public_knowledge());
        knowledge::MAINTENANCE.with(|m| m.set((0, 0)));
        for step in 0..60 {
            // Identical public view, except the published draw multiset, which only a maintaining combat builds.
            let (mut view_on, view_off) = (on.public_full().unwrap(), off.public_full().unwrap());
            assert!(view_off.draw_unknown.is_empty(), "no published draw multiset without maintenance");
            if view_on.known_draw.is_empty() { assert_eq!(view_on.draw_unknown.len(), view_on.draw_count); }
            view_on.draw_unknown.clear();
            assert_eq!(view_on, view_off, "seed {seed} step {step}: identical public view");
            let actions = on.legal_actions_full().unwrap();
            assert_eq!(actions, off.legal_actions_full().unwrap());
            assert_eq!(off.draw_knowledge(), Err(Unrepresentable), "no knowledge without maintenance");
            assert!(off.preroll_sets().is_empty());
            let Some(action) = actions.get((seed as usize + step) % actions.len().max(1)).cloned() else { break };
            let before = knowledge::MAINTENANCE.with(|m| m.get());
            let off_result = off.step_full(&action).is_ok();
            assert_eq!(knowledge::MAINTENANCE.with(|m| m.get()), before, "no maintenance work without the flag");
            assert_eq!(on.step_full(&action).is_ok(), off_result);
            if on.public_full().unwrap().outcome.is_some() { break }
        }
        assert!(knowledge::MAINTENANCE.with(|m| m.get()).0 > 0, "the maintained combat did the work");
    }
}
