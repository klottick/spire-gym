//! M34s-c c2: synthetic positions `spire-gym-combat-position-v1` (design §4.8; c0 `task0-v1.md` §3–§4): enemy fields at
//! setup before the opening roll, `Combat::apply_position` at the native opening's first main decision (overwrite table),
//! the gym-owned draw order and the public-knowledge reset.
use super::super::position::{EnemyFields, Position, PositionCard, PositionError, PositionPiles};
use super::super::*;
use crate::content::{CardId, Character, EncounterId, RelicId};
use crate::run::{AuthoredRunStart, CardInstance, Run, RunRng, StandardCombatStart};

fn deck(cards: &[(CardId, u32)]) -> Vec<CardInstance> {
    let mut cards = cards.to_vec();
    cards.sort_by_key(|(id, u)| (id.as_str(), *u));
    cards.iter().enumerate().map(|(i, (id, u))| CardInstance { id: *id, instance: i as u64 + 1, upgrades: *u, misc: 0 }).collect()
}
fn start(character: Character, ascension: u8, encounter: EncounterId, cards: &[(CardId, u32)], relics: Vec<RelicId>, position: Option<Position>) -> StandardCombatStart {
    StandardCombatStart { character, ascension, encounter, burning: None, bottled_cards: vec![], position,
        resources: AuthoredRunStart { hp: 70, max_hp: 80, gold: 50, deck: deck(cards), relics, potions: vec![None; if ascension >= 11 { 2 } else { 3 }], relic_counters: Default::default() } }
}
fn enter(start: StandardCombatStart, seed: u8) -> Result<Run, PositionError> {
    let mut run = Run::prepare_standard_combat(start, RunRng::new([seed; 32], rng::Partition::Test, 0)).map_err(|_| PositionError::Input("prepare"))?;
    run.enter_standard_combat_position().map_err(|e| match e { crate::run::StandardStartError::Position(p) => p, crate::run::StandardStartError::Run(_) => PositionError::Input("run") })?;
    Ok(run)
}
fn state(run: &Run) -> &model::State { &run.state().combat.as_ref().unwrap().state }
fn card(id: CardId) -> PositionCard { PositionCard { id, upgrades: 0, misc: 0, generated: false } }
fn generated(id: CardId) -> PositionCard { PositionCard { id, upgrades: 0, misc: 0, generated: true } }
fn power(name: &str, amount: i32) -> (FullPowerId, i32) { (super::super::position::power_id(name).unwrap(), amount) }
fn amount(powers: &[FullPowerEntry], name: &str) -> i32 { powers.iter().find(|p| p.id.0 == name).map_or(0, |p| p.state.amount) }
fn ids(pile: &[model::OwnedCard]) -> Vec<CardId> { pile.iter().map(|c| c.full().unwrap().id).collect() }
const IRONCLAD: &[(CardId, u32)] = &[(CardId::Strike, 0), (CardId::Strike, 0), (CardId::Strike, 0), (CardId::Strike, 0), (CardId::Defend, 0),
    (CardId::Defend, 0), (CardId::Defend, 0), (CardId::Bash, 0), (CardId::TrueGrit, 1), (CardId::ShrugItOff, 0)];
const SILENT: &[(CardId, u32)] = &[(CardId::StrikeGreen, 0), (CardId::StrikeGreen, 0), (CardId::DefendGreen, 0), (CardId::DefendGreen, 0),
    (CardId::Neutralize, 0), (CardId::DeadlyPoison, 0), (CardId::BouncingFlask, 0), (CardId::Catalyst, 1), (CardId::Footwork, 0)];
/// Every Ironclad deck card, split hand (5) / draw_top (1) / draw (2) / discard (1) / exhaust (1).
fn ironclad_piles() -> PositionPiles {
    PositionPiles { hand: vec![card(CardId::Bash), card(CardId::TrueGrit).up(), card(CardId::ShrugItOff), card(CardId::Strike), card(CardId::Defend)],
        draw_top: vec![card(CardId::Strike)], draw: vec![card(CardId::Strike), card(CardId::Defend)],
        discard: vec![card(CardId::Strike)], exhaust: vec![card(CardId::Defend)] }
}
trait Up { fn up(self) -> Self; }
impl Up for PositionCard { fn up(mut self) -> Self { self.upgrades = 1; self } }

/// Design §4.8 (1), review I6: the enemy's setup HP feeds its opening roll (Champ chooses Anger below half HP before its RNG),
/// with no damage event implied.
#[test]
fn enemy_fields_apply_at_setup_before_the_opening_roll() {
    let at = |hp: Option<i32>| {
        let p = Position { enemies: vec![EnemyFields { slot: 0, hp, block: None, powers: vec![] }], ..Position::default() };
        enter(start(Character::Ironclad, 0, EncounterId::Champ, IRONCLAD, vec![], Some(p)), 4).unwrap()
    };
    let (low, full) = (at(Some(209)), at(None));
    let champ = |run: &Run| state(run).enemies[0].clone();
    assert_eq!(champ(&low).intent_id.as_str(), "champ_anger");
    assert_ne!(champ(&full).intent_id.as_str(), "champ_anger", "control: a full-HP Champ rolls a normal opening");
    assert_eq!((champ(&low).hp, champ(&low).max_hp), (209, champ(&full).max_hp));
    assert_eq!(state(&low).damage_events_this_combat, 0, "no damage event is implied");
    // Time Eater: Haste below half HP, same law.
    let p = Position { enemies: vec![EnemyFields { slot: 0, hp: Some(227), block: None, powers: vec![] }], ..Position::default() };
    let te = enter(start(Character::Ironclad, 0, EncounterId::TimeEater, IRONCLAD, vec![], Some(p)), 4).unwrap();
    assert_eq!(state(&te).enemies[0].intent_id.as_str(), "time_haste");
}

/// Named fields are set; everything else is the native opening's.
#[test]
fn position_sets_named_fields_and_keeps_the_native_rest() {
    let native = enter(start(Character::Ironclad, 0, EncounterId::JawWorm, IRONCLAD, vec![RelicId::Anchor], None), 9).unwrap();
    let p = Position { powers: vec![power("feel_no_pain", 3)], energy: Some(5), ..Position::default() };
    let run = enter(start(Character::Ironclad, 0, EncounterId::JawWorm, IRONCLAD, vec![RelicId::Anchor], Some(p)), 9).unwrap();
    let (n, s) = (state(&native), state(&run));
    assert_eq!((amount(&s.player_full_powers, "feel_no_pain"), s.player.energy), (3, 5));
    assert_eq!(s.player.block, n.player.block, "Anchor's native block is kept when block is not listed");
    assert_eq!(ids(&s.piles.hand), ids(&n.piles.hand), "piles absent: the native piles are kept");
    assert_eq!((s.enemies[0].hp, s.enemies[0].intent_id.as_str()), (n.enemies[0].hp, n.enemies[0].intent_id.as_str()));
    let p = Position { block: Some(0), ..Position::default() };
    assert_eq!(state(&enter(start(Character::Ironclad, 0, EncounterId::JawWorm, IRONCLAD, vec![RelicId::Anchor], Some(p)), 9).unwrap()).player.block, 0);
}

/// Overwrite table row 1: Ninja Scroll's opening Shivs are discarded and de-issued; listed generated cards are issued anew.
#[test]
fn opening_generated_cards_are_discarded_and_deissued() {
    let native = enter(start(Character::Silent, 0, EncounterId::Cultist, SILENT, vec![RelicId::NinjaScroll], None), 2).unwrap();
    assert!(ids(&state(&native).piles.hand).contains(&CardId::Shiv), "control: the native opening has Ninja Scroll's Shivs");
    let all = |extra: Vec<PositionCard>| PositionPiles { hand: [vec![card(CardId::Neutralize), card(CardId::DeadlyPoison)], extra].concat(),
        draw: SILENT.iter().filter(|(id, _)| !matches!(id, CardId::Neutralize | CardId::DeadlyPoison)).map(|(id, u)| PositionCard { upgrades: *u, ..card(*id) }).collect(),
        ..PositionPiles::default() };
    let run = enter(start(Character::Silent, 0, EncounterId::Cultist, SILENT, vec![RelicId::NinjaScroll], Some(Position { piles: Some(all(vec![])), ..Position::default() })), 2).unwrap();
    let s = state(&run);
    assert!(!s.piles.hand.iter().chain(&s.piles.draw).any(|c| c.full().unwrap().id == CardId::Shiv));
    assert_eq!(s.card_ids_issued.len(), SILENT.len(), "only deck instances stay issued");
    let run = enter(start(Character::Silent, 0, EncounterId::Cultist, SILENT, vec![RelicId::NinjaScroll], Some(Position { piles: Some(all(vec![generated(CardId::Shiv)])), ..Position::default() })), 2).unwrap();
    let shiv = state(&run).piles.hand.iter().filter_map(|c| c.full()).find(|c| c.id == CardId::Shiv).unwrap();
    assert_eq!(shiv.instance.0, SILENT.len() as u64 + 1, "the listed Shiv takes the next issued id");
}

/// Overwrite table: relic-start powers are kept unless listed; a listed power is set; amount 0 removes (player and enemies).
#[test]
fn relic_start_powers_kept_unless_listed() {
    let relics = || vec![RelicId::Vajra, RelicId::BagOfMarbles];
    let run = enter(start(Character::Ironclad, 0, EncounterId::JawWorm, IRONCLAD, relics(), Some(Position::default())), 3).unwrap();
    assert_eq!((amount(&state(&run).player_full_powers, "strength"), amount(&state(&run).enemies[0].full_powers, "vulnerable")), (1, 1));
    let p = Position { powers: vec![power("strength", 0)], enemies: vec![EnemyFields { slot: 0, hp: None, block: None, powers: vec![power("vulnerable", 3)] }], ..Position::default() };
    let run = enter(start(Character::Ironclad, 0, EncounterId::JawWorm, IRONCLAD, relics(), Some(p)), 3).unwrap();
    assert_eq!((amount(&state(&run).player_full_powers, "strength"), amount(&state(&run).enemies[0].full_powers, "vulnerable")), (0, 3));
    let p = Position { enemies: vec![EnemyFields { slot: 0, hp: None, block: Some(7), powers: vec![power("vulnerable", 0)] }], ..Position::default() };
    let run = enter(start(Character::Ironclad, 0, EncounterId::JawWorm, IRONCLAD, relics(), Some(p)), 3).unwrap();
    assert_eq!((amount(&state(&run).enemies[0].full_powers, "vulnerable"), state(&run).enemies[0].block), (0, 7));
}

/// Draw-time effects: under Snecko Eye (Confused) each hand card's cost is rolled on the session card stream, in hand order,
/// exactly as the shared draw callback does.
#[test]
fn snecko_hand_costs_are_rolled_with_the_card_stream() {
    let piles = || Some(ironclad_piles());
    let snecko = |seed| enter(start(Character::Ironclad, 0, EncounterId::JawWorm, IRONCLAD, vec![RelicId::SneckoEye], Some(Position { piles: piles(), ..Position::default() })), seed).unwrap();
    let costs = |run: &Run| state(run).piles.hand.iter().map(|c| c.full().unwrap().local.randomized_cost).collect::<Vec<_>>();
    let a = snecko(5);
    assert!(costs(&a).iter().all(|c| c.is_some_and(|c| (0..=3).contains(&c))), "every costed hand card is rolled");
    assert_eq!(costs(&a), costs(&snecko(5)), "same seed, same rolls");
    assert!((6..30).any(|seed| costs(&snecko(seed)) != costs(&a)), "the rolls follow the session's card stream");
    let plain = enter(start(Character::Ironclad, 0, EncounterId::JawWorm, IRONCLAD, vec![], Some(Position { piles: piles(), ..Position::default() })), 5).unwrap();
    assert!(costs(&plain).iter().all(Option::is_none), "control: no Confused, no roll");
}

/// Relic counters and armed flags are never set by a position: they are the native opening's (Lantern's energy, Pen Nib's counter).
#[test]
fn armed_relic_state_is_native() {
    let relics = || vec![RelicId::Lantern, RelicId::PenNib];
    let native = enter(start(Character::Ironclad, 0, EncounterId::JawWorm, IRONCLAD, relics(), None), 7).unwrap();
    let run = enter(start(Character::Ironclad, 0, EncounterId::JawWorm, IRONCLAD, relics(), Some(Position { piles: Some(ironclad_piles()), ..Position::default() })), 7).unwrap();
    assert_eq!(state(&run).player.energy, state(&native).player.energy, "Lantern's opening energy is native");
    assert_eq!(state(&run).item_context, state(&native).item_context, "item counters and armed flags are native");
}

/// Review M9: position-set Weak/Vulnerable/Frail clear `skip_first` and `monster_source_grace`; the first end of round
/// decrements them.
#[test]
fn position_set_weak_vulnerable_frail_have_cleared_flags() {
    let p = Position { powers: vec![power("weak", 2), power("vulnerable", 2), power("frail", 2)], ..Position::default() };
    let mut run = enter(start(Character::Ironclad, 0, EncounterId::Cultist, IRONCLAD, vec![], Some(p)), 1).unwrap();
    for name in ["weak", "vulnerable", "frail"] {
        let p = state(&run).player_full_powers.iter().find(|p| p.id.0 == name).unwrap().state.clone();
        assert_eq!((p.amount, p.skip_first, p.monster_source_grace), (2, false, false), "{name}");
    }
    // An already-present debuff with its flags set (as a monster-applied one would have) is overwritten with both flags cleared.
    let native = enter(start(Character::Ironclad, 0, EncounterId::Cultist, IRONCLAD, vec![], None), 1).unwrap();
    let mut combat = native.state().combat.as_ref().unwrap().clone();
    for name in ["weak", "vulnerable", "frail"] {
        combat.state.player_full_powers.push(FullPowerEntry { id: FullPowerId(super::super::position::power_id(name).unwrap().0), state: PowerState { amount: 1, skip_first: true, monster_source_grace: true } });
    }
    let p = Position { powers: vec![power("weak", 2), power("vulnerable", 2), power("frail", 2)], ..Position::default() };
    let applied = combat.apply_position(&p, &deck(IRONCLAD)).unwrap();
    for name in ["weak", "vulnerable", "frail"] {
        let p = applied.state.player_full_powers.iter().find(|p| p.id.0 == name).unwrap().state.clone();
        assert_eq!((p.amount, p.skip_first, p.monster_source_grace), (2, false, false), "{name} overwritten");
    }
    let end = run.decision().unwrap().candidates.iter().find(|c| matches!(c.action, crate::run::GameAction::Combat(CombatAction::EndTurn))).unwrap().action.clone();
    run.step(&end).unwrap();
    for name in ["weak", "vulnerable", "frail"] { assert_eq!(amount(&state(&run).player_full_powers, name), 1, "{name} decremented at the first end of round"); }
}

/// The draw multiset's order is gym-owned: shuffled on the session shuffle stream (same seed ⇒ same order; seeds differ ⇒
/// orders differ); `draw_top` is placed on top, in order, and is known.
#[test]
fn position_draw_order_is_gym_owned_and_seeded_and_draw_top_is_known() {
    let many = PositionPiles { hand: vec![card(CardId::Bash)], draw_top: vec![card(CardId::ShrugItOff), card(CardId::TrueGrit).up()],
        draw: IRONCLAD.iter().filter(|(id, _)| !matches!(id, CardId::Bash | CardId::ShrugItOff | CardId::TrueGrit)).map(|(id, _)| card(*id)).collect(), ..PositionPiles::default() };
    let at = |seed| enter(start(Character::Ironclad, 0, EncounterId::Cultist, IRONCLAD, vec![], Some(Position { piles: Some(many.clone()), ..Position::default() })), seed).unwrap();
    let order = |run: &Run| state(run).piles.draw.iter().map(|c| c.handle()).collect::<Vec<_>>();
    assert_eq!(order(&at(1)), order(&at(1)));
    assert!((2..20).any(|seed| order(&at(seed)) != order(&at(1))), "the bag below the known prefix is shuffled by seed");
    let run = at(3);
    let s = state(&run);
    let top = s.piles.draw.iter().rev().take(2).map(|c| c.full().unwrap().id).collect::<Vec<_>>();
    assert_eq!(top, vec![CardId::ShrugItOff, CardId::TrueGrit]);
    assert_eq!(s.public.known_draw, s.piles.draw.iter().rev().take(2).map(|c| c.handle()).collect::<Vec<_>>());
}

/// A position resets public knowledge: the draw knowledge is the known prefix over one bag of the rest; pre-roll sets are
/// re-observed at the position's own boundary (never narrowed by the unpublished native opening).
#[test]
fn position_resets_public_knowledge() {
    let piles = PositionPiles { hand: vec![card(CardId::Bash)], draw_top: vec![card(CardId::ShrugItOff)],
        draw: IRONCLAD.iter().filter(|(id, _)| !matches!(id, CardId::Bash | CardId::ShrugItOff)).map(|(id, u)| PositionCard { upgrades: *u, ..card(*id) }).collect(), ..PositionPiles::default() };
    let run = enter(start(Character::Ironclad, 0, EncounterId::TwoLouse, IRONCLAD, vec![], Some(Position { piles: Some(piles), powers: vec![power("weak", 1)], ..Position::default() })), 6).unwrap();
    let combat = run.state().combat.as_ref().unwrap();
    let k = combat.draw_knowledge().unwrap();
    assert_eq!(k.components().len(), 1);
    let segments = &k.components()[0].segments;
    assert!(matches!(segments.first(), Some(knowledge::Segment::Known(_))) && matches!(segments.get(1), Some(knowledge::Segment::Bag(_))) && segments.len() == 2);
    assert_eq!(state(&run).public.revealed_order, state(&run).piles.hand.iter().map(|c| c.handle()).collect::<Vec<_>>());
    // Pre-roll sets equal a fresh observation of the published display, never narrowed by the unpublished native opening:
    // Darkling Nip 8 and 9 both show 6 under Weak (A0 law 7..=11), while the native opening showed them apart.
    let weak = (0..3).map(|slot| EnemyFields { slot, hp: None, block: None, powers: vec![power("weak", 1)] }).collect::<Vec<_>>();
    let mut mattered = false;
    for seed in 0..24 {
        let run = enter(start(Character::Ironclad, 0, EncounterId::ThreeDarklings, IRONCLAD, vec![], Some(Position { enemies: weak.clone(), ..Position::default() })), seed).unwrap();
        let native = enter(start(Character::Ironclad, 0, EncounterId::ThreeDarklings, IRONCLAD, vec![], None), seed).unwrap();
        let combat = run.state().combat.as_ref().unwrap();
        let mut fresh = combat.clone();
        fresh.state.knowledge.prerolls = Default::default();
        knowledge::observe_prerolls(&mut fresh.state);
        assert_eq!(combat.preroll_sets(), fresh.preroll_sets(), "seed {seed}");
        mattered |= native.state().combat.as_ref().unwrap().preroll_sets() != combat.preroll_sets();
    }
    assert!(mattered, "control: some native opening narrowed a set the position's display does not");
    // Final review I3: Philosopher's Stone gives every enemy Strength 1 during the native opening; the position re-asserts
    // Strength 0 (and Weak) at its own boundary, so the published Nip display is coarser than the unpublished native one.
    let shifted = (0..3).map(|slot| EnemyFields { slot, hp: None, block: None, powers: vec![power("strength", 0), power("weak", 1)] }).collect::<Vec<_>>();
    let stone = vec![RelicId::PhilosophersStone];
    let mut mattered = false;
    for seed in 0..96 {
        let run = enter(start(Character::Ironclad, 0, EncounterId::ThreeDarklings, IRONCLAD, stone.clone(), Some(Position { enemies: shifted.clone(), ..Position::default() })), seed).unwrap();
        let native = enter(start(Character::Ironclad, 0, EncounterId::ThreeDarklings, IRONCLAD, stone.clone(), None), seed).unwrap();
        let combat = run.state().combat.as_ref().unwrap();
        let mut fresh = combat.clone();
        fresh.state.knowledge.prerolls = Default::default();
        knowledge::observe_prerolls(&mut fresh.state);
        assert_eq!(combat.preroll_sets(), fresh.preroll_sets(), "Philosopher's Stone seed {seed}");
        mattered |= native.state().combat.as_ref().unwrap().preroll_sets() != combat.preroll_sets();
    }
    assert!(mattered, "control: the native opening under Strength 1 pinned a set the published display does not");
}

/// Ledgers are rewritten to a turn start consistent with the hand; listed orbs set the channel totals (PM ruling c0 (5)).
#[test]
fn position_ledgers_are_turn_start_consistent() {
    let run = enter(start(Character::Silent, 0, EncounterId::Cultist, SILENT, vec![], Some(Position { piles: Some(PositionPiles {
        hand: vec![card(CardId::Neutralize), card(CardId::Footwork)], draw: SILENT.iter().filter(|(id, _)| !matches!(id, CardId::Neutralize | CardId::Footwork))
            .map(|(id, u)| PositionCard { upgrades: *u, ..card(*id) }).collect(), ..PositionPiles::default() }), ..Position::default() })), 2).unwrap();
    let s = state(&run);
    assert_eq!(s.silent.drawn_cards.iter().map(|d| d.id).collect::<Vec<_>>(), vec![CardId::Neutralize, CardId::Footwork]);
    assert!(s.silent.played_this_turn.is_empty() && s.silent.discarded_this_turn == 0 && s.played_card_types.is_empty());
    let defect: &[(CardId, u32)] = &[(CardId::StrikeBlue, 0), (CardId::StrikeBlue, 0), (CardId::DefendBlue, 0), (CardId::Zap, 0), (CardId::Dualcast, 0)];
    let p = Position { orb_slots: Some(3), orbs: Some(vec![OrbKind::Frost, OrbKind::Frost, OrbKind::Lightning]), ..Position::default() };
    let run = enter(start(Character::Defect, 0, EncounterId::Cultist, defect, vec![RelicId::CrackedCore], Some(p)), 2).unwrap();
    let s = state(&run);
    assert_eq!((s.defect.frost_channeled, s.defect.lightning_channeled), (2, 1));
    assert_eq!(s.character.orb_slots.iter().map(|o| o.map(|o| o.kind)).collect::<Vec<_>>(), vec![Some(OrbKind::Frost), Some(OrbKind::Frost), Some(OrbKind::Lightning)]);
}

/// Derived values are refreshed: a listed enemy Strength shows in the intent damage; a listed Accuracy in the Shivs' values.
#[test]
fn position_refreshes_intent_and_card_values() {
    let native = enter(start(Character::Ironclad, 0, EncounterId::JawWorm, IRONCLAD, vec![], None), 8).unwrap();
    let base = state(&native).enemies[0].intent_damage;
    let p = Position { enemies: vec![EnemyFields { slot: 0, hp: None, block: None, powers: vec![power("strength", 4)] }], powers: vec![power("vulnerable", 1)], ..Position::default() };
    let run = enter(start(Character::Ironclad, 0, EncounterId::JawWorm, IRONCLAD, vec![], Some(p)), 8).unwrap();
    assert!(base.is_some_and(|b| b > 0), "control: Jaw Worm opens with an attack");
    // Enemy Strength (set at setup) and player Vulnerable (applied after the opening): the refresh shows (base + 4) × 1.5.
    assert_eq!(state(&run).enemies[0].intent_damage, base.map(|d| (d + 4) * 3 / 2));
    let piles = PositionPiles { hand: vec![generated(CardId::Shiv), card(CardId::Neutralize)], discard: vec![generated(CardId::Shiv)],
        draw: SILENT.iter().filter(|(id, _)| *id != CardId::Neutralize).map(|(id, u)| PositionCard { upgrades: *u, ..card(*id) }).collect(), ..PositionPiles::default() };
    let run = enter(start(Character::Silent, 0, EncounterId::JawWorm, SILENT, vec![], Some(Position { powers: vec![power("accuracy", 4)], piles: Some(piles), ..Position::default() })), 8).unwrap();
    let view = run.state().combat.as_ref().unwrap().public_full().unwrap();
    for shiv in view.hand.iter().chain(&view.discard).filter(|c| c.id == CardId::Shiv) { assert_eq!(shiv.values.as_ref().unwrap().base_damage, Some(8), "Shiv 4 + Accuracy 4"); }
}

/// The opening must reach the main decision: an opening selection (Toolbox's offer, Gambling Chip's discard) is
/// `UnsupportedBoundary`, never a guessed post-selection state.
#[test]
fn position_requires_the_main_opening_decision() {
    for relic in [RelicId::Toolbox, RelicId::GamblingChip] {
        let r = enter(start(Character::Ironclad, 0, EncounterId::JawWorm, IRONCLAD, vec![relic], Some(Position::default())), 1);
        assert!(matches!(r, Err(PositionError::Unsupported(_))), "{relic:?}");
    }
}

/// PM ruling c0 (6): an owned Bottled relic names its bottled deck card, which is innate as native.
#[test]
fn standard_combat_bottled_card_is_named_and_innate() {
    let mut s = start(Character::Ironclad, 0, EncounterId::JawWorm, IRONCLAD, vec![RelicId::BottledFlame], None);
    assert!(Run::prepare_standard_combat(s.clone(), RunRng::new([1; 32], rng::Partition::Test, 0)).is_err(), "an owned bottle without its card is rejected");
    let bash = s.resources.deck.iter().find(|c| c.id == CardId::Bash).unwrap().instance;
    let defend = s.resources.deck.iter().find(|c| c.id == CardId::Defend).unwrap().instance;
    s.bottled_cards = vec![(RelicId::BottledFlame, Some(defend))];
    assert!(Run::prepare_standard_combat(s.clone(), RunRng::new([1; 32], rng::Partition::Test, 0)).is_err(), "Bottled Flame takes an Attack");
    // Final review m1: an owned bottle without a card is named explicitly (unlinked: no card is innate).
    s.bottled_cards = vec![(RelicId::BottledFlame, None)];
    let hands = (0..8).map(|seed| state(&enter(s.clone(), seed).unwrap()).piles.hand.iter().any(|c| c.full().unwrap().instance.0 == bash)).collect::<Vec<_>>();
    assert!(hands.contains(&false), "unlinked: Bash is not innate");
    s.bottled_cards = vec![(RelicId::BottledFlame, Some(bash))];
    for seed in 0..8 {
        let run = enter(s.clone(), seed).unwrap();
        assert!(state(&run).piles.hand.iter().any(|c| c.full().unwrap().instance.0 == bash), "the bottled Bash is in every opening hand");
    }
    // A position must keep the bottled card in its hand.
    let mut p = ironclad_piles();
    p.hand.retain(|c| c.id != CardId::Bash);
    p.draw.push(card(CardId::Bash));
    s.position = Some(Position { piles: Some(p), ..Position::default() });
    assert!(matches!(enter(s, 1), Err(PositionError::Input(_))));
}

/// Every lesson/smoke shape continues with normal actions to the combat's terminal with the runtime invariant at every step.
#[test]
fn position_continues_with_normal_actions_to_combat_end() {
    let p = Position { powers: vec![power("feel_no_pain", 3), power("dark_embrace", 1)], piles: Some(ironclad_piles()), ..Position::default() };
    let mut run = enter(start(Character::Ironclad, 0, EncounterId::GremlinNob, IRONCLAD, vec![], Some(p)), 1).unwrap();
    assert_eq!(state(&run).enemies[0].intent_id.as_str(), "bellow", "the Nob lesson: its opening move is Bellow, before Enrage");
    assert_eq!(amount(&state(&run).enemies[0].full_powers, "enrage"), 0);
    let mut i = 0u64;
    while run.combat_scope_terminal().is_none() {
        run.state().combat.as_ref().unwrap().validate_runtime_for_test().unwrap();
        let d = run.decision().unwrap();
        i = i.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let a = d.candidates[(i >> 33) as usize % d.candidates.len()].action.clone();
        run.step(&a).unwrap();
    }
}

fn with_enemies(enemies: Vec<EnemyFields>) -> Option<Position> { Some(Position { enemies, ..Position::default() }) }
fn fields(slot: u8, hp: Option<i32>, block: Option<i32>, powers: Vec<(FullPowerId, i32)>) -> EnemyFields { EnemyFields { slot, hp, block, powers } }
fn species(s: &model::State, i: usize) -> crate::content::EnemyId { match s.enemies[i].enemy_id { ActorEnemyId::Full(id) => id, _ => panic!("legacy actor") } }

/// Final review I1 (census): enemy edits are admitted only where every slot's species is seed-independent; the shapes
/// encounters re-choose each slot's species at setup and reject `enemies` at validation, naming the field.
#[test]
fn enemy_edits_need_seed_independent_species() {
    for encounter in [EncounterId::ThreeShapes, EncounterId::FourShapes, EncounterId::SphereAndTwoShapes] {
        let s = start(Character::Ironclad, 7, encounter, IRONCLAD, vec![], with_enemies(vec![fields(0, Some(31), None, vec![])]));
        assert!(matches!(s.validate_position(), Err(PositionError::Input(m)) if m.starts_with("enemies")), "{encounter:?}");
    }
    // Every Standard encounter that admits enemy edits sets up the formation's species on every seed.
    let mut admitted = 0;
    for (encounter, _, _) in super::super::enemies::A0_ENCOUNTER_SETUPS.iter().copied().filter(|(e, _, _)| crate::run::standard_combat_encounter(*e).is_some()) {
        let Some(formation) = super::super::enemies::position_formation(encounter) else { continue };
        admitted += 1;
        for seed in 0..12 {
            let run = enter(start(Character::Ironclad, 0, encounter, IRONCLAD, vec![], None), seed).unwrap();
            let got = (0..formation.len()).map(|i| species(state(&run), i)).collect::<Vec<_>>();
            assert_eq!(got, formation, "{encounter:?} seed {seed}");
        }
    }
    assert!(admitted > 40, "{admitted}");
}

/// Final review I1 (defence in depth): a setup override the rolled formation cannot take is a position rejection, never a
/// Run fault (the session is not frozen).
#[test]
fn setup_override_failure_is_a_position_rejection() {
    let p = Position { enemies: vec![fields(0, Some(31), None, vec![])], ..Position::default() };
    let mut rejected = 0;
    for seed in 0..=255u8 {
        let mut run = Run::prepare_standard_combat(start(Character::Ironclad, 7, EncounterId::ThreeShapes, IRONCLAD, vec![], None), RunRng::new([seed; 32], rng::Partition::Test, 0)).unwrap();
        run.set_combat_position_for_test(p.clone());
        match run.enter_standard_combat_position() {
            Ok(_) => {}
            Err(crate::run::StandardStartError::Position(PositionError::Input(m))) => { assert!(m.starts_with("enemies"), "{m}"); rejected += 1; }
            Err(e) => panic!("seed {seed}: {e:?}"),
        }
    }
    assert!(rejected > 0, "control: some seed rolls a 30-HP Exploder in slot 0");
}

/// Final review I2: `slot` is the formation index at validation, setup and the opening decision alike (Reptomancer's
/// logical slots are Dagger 1, Reptomancer 2, Dagger 4); single-boss formations accept edits on their boss.
#[test]
fn formation_slots_name_the_same_enemy_at_setup_and_opening() {
    let run = enter(start(Character::Ironclad, 0, EncounterId::Reptomancer, IRONCLAD, vec![], with_enemies(vec![fields(1, None, Some(10), vec![power("strength", 2)])])), 1).unwrap();
    let s = state(&run);
    for i in 0..3 {
        let edited = species(s, i) == crate::content::EnemyId::Reptomancer;
        assert_eq!((s.enemies[i].block, amount(&s.enemies[i].full_powers, "strength")), if edited { (10, 2) } else { (0, 0) }, "enemy {i}");
    }
    let run = enter(start(Character::Ironclad, 0, EncounterId::Reptomancer, IRONCLAD, vec![], with_enemies(vec![fields(0, Some(20), Some(5), vec![])])), 1).unwrap();
    let s = state(&run);
    let first = (0..3).find(|&i| s.enemies[i].instance_id.as_str().ends_with("-0")).unwrap();
    assert_eq!((species(s, first), s.enemies[first].hp, s.enemies[first].block), (crate::content::EnemyId::Dagger, 20, 5));
    assert_eq!((0..3).filter(|&i| s.enemies[i].block == 5).count(), 1);
    let run = enter(start(Character::Ironclad, 0, EncounterId::BronzeAutomaton, IRONCLAD, vec![], with_enemies(vec![fields(0, None, Some(7), vec![power("artifact", 5)])])), 1).unwrap();
    let boss = (0..state(&run).enemies.len()).find(|&i| species(state(&run), i) == crate::content::EnemyId::BronzeAutomaton).unwrap();
    assert_eq!((state(&run).enemies[boss].block, amount(&state(&run).enemies[boss].full_powers, "artifact")), (7, 5));
    let run = enter(start(Character::Ironclad, 0, EncounterId::Collector, IRONCLAD, vec![], with_enemies(vec![fields(0, None, Some(4), vec![power("strength", 1)])])), 1).unwrap();
    let boss = (0..state(&run).enemies.len()).find(|&i| species(state(&run), i) == crate::content::EnemyId::Collector).unwrap();
    assert_eq!((state(&run).enemies[boss].block, amount(&state(&run).enemies[boss].full_powers, "strength")), (4, 1));
}

/// Final review m2: Unceasing Top refills an empty hand at the engine's hand-emptied seam before any decision is published,
/// so an empty hand with cards to draw is never a native decision.
#[test]
fn empty_hand_under_unceasing_top_is_rejected() {
    let all = |hand: Vec<PositionCard>, draw: Vec<PositionCard>, exhaust: Vec<PositionCard>| Some(Position { piles: Some(PositionPiles { hand, draw, exhaust, ..PositionPiles::default() }), ..Position::default() });
    let deck = || IRONCLAD.iter().map(|(id, u)| PositionCard { upgrades: *u, ..card(*id) }).collect::<Vec<_>>();
    let top = vec![RelicId::UnceasingTop];
    assert!(matches!(enter(start(Character::Ironclad, 0, EncounterId::JawWorm, IRONCLAD, top.clone(), all(vec![], deck(), vec![])), 1), Err(PositionError::Input(m)) if m.starts_with("piles.hand")));
    assert!(enter(start(Character::Ironclad, 0, EncounterId::JawWorm, IRONCLAD, vec![], all(vec![], deck(), vec![])), 1).is_ok(), "control: no Unceasing Top");
    assert!(enter(start(Character::Ironclad, 0, EncounterId::JawWorm, IRONCLAD, top, all(vec![], vec![], deck())), 1).is_ok(), "nothing to draw: the seam does nothing");
}

/// Final review m3: burning buffs are admitted as the native start applies them — the entry's declared buff, on every
/// member of the elite formation, with one amount.
#[test]
fn enemy_burning_powers_follow_the_native_buff() {
    let elite = |encounter, burning, enemies| { let mut s = start(Character::Ironclad, 0, encounter, IRONCLAD, vec![], with_enemies(enemies)); s.burning = burning; s.validate_position() };
    use crate::run::BurningEliteBuff::{Metallicize, Regeneration};
    let m = |slot, a| fields(slot, None, None, vec![power("metallicize", a)]);
    assert!(elite(EncounterId::GremlinNob, None, vec![m(0, 4)]).is_err(), "no burning buff declared");
    assert!(elite(EncounterId::GremlinNob, Some(Metallicize), vec![m(0, 4)]).is_ok());
    assert!(elite(EncounterId::GremlinNob, Some(Regeneration), vec![m(0, 4)]).is_err(), "another buff declared");
    assert!(elite(EncounterId::GremlinNob, Some(Metallicize), vec![fields(0, None, None, vec![power("metallicize", 4), power("regenerate", 4)])]).is_err(), "two buffs");
    assert!(elite(EncounterId::Slavers, Some(Metallicize), vec![m(0, 4)]).is_err(), "one member of three");
    assert!(elite(EncounterId::Slavers, Some(Metallicize), vec![m(0, 4), m(1, 4), m(2, 5)]).is_err(), "unequal amounts");
    assert!(elite(EncounterId::Slavers, Some(Metallicize), vec![m(0, 4), m(1, 4), m(2, 4)]).is_ok());
}
