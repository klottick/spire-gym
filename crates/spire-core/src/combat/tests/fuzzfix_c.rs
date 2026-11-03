//! Regressions for fuzz chunk C1 (spire-oto.22): the player's end of turn in source order
//! (source-check-endturn-v1). callEndOfTurnActions runs the onPlayerEndTurn relics, then the
//! pre-card powers in power-list order, then the orb pass and the hand cards' end-turn plays;
//! AbstractRoom.endTurn then runs the player's atEndOfTurn powers in list order before the discard.
use super::super::*;
use super::fuzzfix_b::{arrange, fixture, fixture_in, player_power};
use crate::combat::model::OwnedCard;
use crate::combat::observe::{ResolvedEffect, ResolvedEffectKind};
use crate::{combat::CombatAction, content::{CardId, Character, EncounterId, RelicId}};
use CardId::*;

const ALL: [Character; 4] = [Character::Ironclad, Character::Silent, Character::Defect, Character::Watcher];

fn power(id: &'static str, amount: i32) -> FullPowerEntry {
    FullPowerEntry { id: FullPowerId(id), state: PowerState { amount, skip_first: false, monster_source_grace: false } }
}

fn end_turn(combat: &mut Combat) -> Vec<ResolvedEffect> {
    let observed = combat.step_full_observed(&CombatAction::EndTurn);
    observed.result.unwrap_or_else(|error| panic!("EndTurn: {error:?}; fault={:?}", combat.state.fault));
    assert!(combat.state.fault.is_none(), "EndTurn fault: {:?}", combat.state.fault);
    observed.effects
}

fn at(effects: &[ResolvedEffect], what: &str, matches: impl Fn(&ResolvedEffect) -> bool) -> u32 {
    effects.iter().find(|event| matches(event)).unwrap_or_else(|| panic!("no {what} in {effects:?}")).sequence
}

fn block(before: i32, after: i32) -> impl Fn(&ResolvedEffect) -> bool {
    move |event| event.kind == ResolvedEffectKind::Block { before, after }
}

fn enemy_damage(event: &ResolvedEffect) -> bool {
    matches!(event.target, Some(Actor::Enemy(_))) && matches!(event.kind, ResolvedEffectKind::Damage { .. })
}

fn player_damage(event: &ResolvedEffect) -> bool {
    event.target == Some(Actor::Player) && matches!(event.kind, ResolvedEffectKind::Damage { .. })
}

fn power_change(id: &'static str) -> impl Fn(&ResolvedEffect) -> bool {
    move |event| matches!(&event.kind, ResolvedEffectKind::Power { id: changed, .. } if changed.0 == id)
}

fn with_stone_calendar_due(combat: &mut Combat) {
    combat.state.item_context.as_mut().unwrap().set_local(RelicId::StoneCalendar, 7);
    combat.state.enemies[0].hp = 30;
}

/// The orb pass is queued before the hand cards' end-turn plays, so a Frost passive's Block
/// takes Burn's hit (base: Burn resolved first, then the orbs after the discard).
#[test]
fn fuzzfix_c_frost_orb_block_precedes_burn() {
    let mut combat = fixture(Character::Defect, &[(Burn, 0)], vec![], vec![None; 3], 3);
    combat.enqueue_native_effects(vec![Effect { source: Source::None, target: Some(Actor::Player),
        kind: EffectKind::Character(super::super::characters::CharacterEffect::Channel(OrbKind::Frost)) }], None).unwrap();
    let hp = combat.state.player.hp;
    let effects = end_turn(&mut combat);
    assert!(at(&effects, "Frost Block", block(0, 2)) < at(&effects, "Burn damage", player_damage), "{effects:?}");
    assert_eq!(combat.state.player.hp, hp, "Frost's 2 Block absorbs Burn's 2 (Cultist incants on turn 1)");
}

/// Stone Calendar is an onPlayerEndTurn relic: its 52 damage resolves inside the player's end
/// of turn, before the monster turn (base: its bottom work landed after every enemy move).
#[test]
fn fuzzfix_c_stone_calendar_resolves_before_the_monster_turn() {
    for character in ALL {
        let mut combat = fixture(character, &[(Strike, 0)], vec![RelicId::StoneCalendar], vec![None; 3], 3);
        with_stone_calendar_due(&mut combat);
        end_turn(&mut combat);
        assert!(!combat.state.hook_events.iter().any(|event| event.hook == Hook::EnemyTurnStarted),
            "{character:?}: the Cultist died at the end of the player's turn and never acted");
        assert_eq!(combat.state.enemies[0].hp, 0, "{character:?}");
    }
}

/// Relics run in relic-list order: Cloak Clasp's and Stone Calendar's addToBot work follows
/// the owner order, whichever is first (Cloak Clasp is a Watcher relic).
#[test]
fn fuzzfix_c_end_turn_relics_follow_relic_order_both_ways() {
    for character in [Character::Watcher] {
        for clasp_first in [true, false] {
            let relics = if clasp_first { vec![RelicId::CloakClasp, RelicId::StoneCalendar] }
                else { vec![RelicId::StoneCalendar, RelicId::CloakClasp] };
            let mut combat = fixture(character, &[(Strike, 0), (Strike, 0)], relics, vec![None; 3], 3);
            with_stone_calendar_due(&mut combat);
            // The Cultist survives the 52, so neither relic's work is cleared by combat end.
            (combat.state.enemies[0].max_hp, combat.state.enemies[0].hp) = (90, 90);
            let effects = end_turn(&mut combat);
            let clasp = at(&effects, "Clasp Block", block(0, 2));
            let calendar = at(&effects, "Calendar damage", enemy_damage);
            assert_eq!(clasp < calendar, clasp_first, "{character:?} clasp_first={clasp_first}: {effects:?}");
        }
    }
}

/// Orichalcum reads the Block at the relic callback, before the pre-card powers: with
/// Metallicize it still grants its 6, for every character (base: no grant without Cloak Clasp
/// or the complete profile, since its check followed Metallicize).
#[test]
fn fuzzfix_c_orichalcum_reads_block_before_metallicize() {
    for character in ALL {
        let mut combat = fixture(character, &[(Strike, 0)], vec![RelicId::Orichalcum], vec![None; 3], 3);
        combat.state.player_full_powers.push(power("metallicize", 3));
        let effects = end_turn(&mut combat);
        assert!(at(&effects, "Orichalcum Block", block(0, 6)) < at(&effects, "Metallicize Block", block(6, 9)), "{character:?}: {effects:?}");
    }
}

/// Orichalcum is addToTop and Cloak Clasp addToBot: Orichalcum resolves first in both relic
/// orders, then Clasp, then the pre-card Metallicize.
#[test]
fn fuzzfix_c_orichalcum_precedes_clasp_and_metallicize_both_ways() {
    for character in [Character::Watcher] {
        for orichalcum_first in [true, false] {
            let relics = if orichalcum_first { vec![RelicId::Orichalcum, RelicId::CloakClasp] }
                else { vec![RelicId::CloakClasp, RelicId::Orichalcum] };
            let mut combat = fixture(character, &[(Strike, 0)], relics, vec![None; 3], 3);
            combat.state.player_full_powers.push(power("metallicize", 3));
            let effects = end_turn(&mut combat);
            let orichalcum = at(&effects, "Orichalcum Block", block(0, 6));
            let clasp = at(&effects, "Clasp Block", block(6, 7));
            let metallicize = at(&effects, "Metallicize Block", block(7, 10));
            assert!(orichalcum < clasp && clasp < metallicize, "{character:?} orichalcum_first={orichalcum_first}: {effects:?}");
        }
    }
}

/// The pre-card powers run in power-list order (Metallicize and Plated Armor, both ways).
#[test]
fn fuzzfix_c_pre_card_powers_follow_power_list_order_both_ways() {
    for character in ALL {
        for metallicize_first in [true, false] {
            let mut combat = fixture(character, &[(Strike, 0)], vec![], vec![None; 3], 3);
            let (metallicize, plated) = (power("metallicize", 3), power("plated_armor", 5));
            combat.state.player_full_powers.extend(if metallicize_first { [metallicize, plated] } else { [plated, metallicize] });
            let effects = end_turn(&mut combat);
            let first = effects.iter().find(|event| matches!(event.kind, ResolvedEffectKind::Block { before: 0, .. })).unwrap();
            let expected = if metallicize_first { 3 } else { 5 };
            assert_eq!(first.kind, ResolvedEffectKind::Block { before: 0, after: expected }, "{character:?}: {effects:?}");
        }
    }
}

/// The player's atEndOfTurn powers run in one list-ordered pass (Omega and Wraith Form, both
/// ways); base ran Wraith Form at the pre-card hook and Omega later.
#[test]
fn fuzzfix_c_end_turn_powers_follow_power_list_order_both_ways() {
    for character in ALL {
        for omega_first in [true, false] {
            let mut combat = fixture(character, &[(Strike, 0)], vec![], vec![None; 3], 3);
            let (omega, wraith) = (power("omega", 4), power("wraith_form", -1));
            combat.state.player_full_powers.extend(if omega_first { [omega, wraith] } else { [wraith, omega] });
            let effects = end_turn(&mut combat);
            let omega = at(&effects, "Omega damage", enemy_damage);
            let wraith = at(&effects, "Wraith Form Dexterity", power_change("dexterity"));
            assert_eq!(omega < wraith, omega_first, "{character:?}: {effects:?}");
        }
    }
}

/// Burn plays (the hand cards' end-turn plays) before the atEndOfTurn pass: Omega's damage
/// follows Burn's. A characterization test: base already ordered these two this way.
#[test]
fn fuzzfix_c_hand_card_plays_precede_the_end_turn_powers() {
    for character in ALL {
        let mut combat = fixture(character, &[(Burn, 0)], vec![], vec![None; 3], 3);
        combat.state.player_full_powers.push(power("omega", 4));
        let effects = end_turn(&mut combat);
        assert!(at(&effects, "Burn damage", player_damage) < at(&effects, "Omega damage", enemy_damage), "{character:?}: {effects:?}");
    }
}

/// RegenPower.atEndOfTurn queues its heal at the top: it resolves before an earlier-listed
/// power's addToBot work, and the player's Regeneration heals under every profile.
#[test]
fn fuzzfix_c_regeneration_heals_ahead_of_the_pass() {
    for character in ALL {
        let mut combat = fixture(character, &[(Strike, 0)], vec![], vec![None; 3], 3);
        combat.state.player.hp = 50;
        combat.state.player_full_powers.extend([power("omega", 4), power("regeneration", 3)]);
        let effects = end_turn(&mut combat);
        let heal = at(&effects, "Regeneration heal", |event| matches!(event.kind, ResolvedEffectKind::Heal { .. }));
        assert!(heal < at(&effects, "Omega damage", enemy_damage), "{character:?}: {effects:?}");
        assert_eq!((combat.state.player.hp, player_power(&combat, "regeneration")), (53, 2), "{character:?}");
    }
}

/// The player's Ritual is RitualPower.atEndOfTurn(isPlayer): Strength at the end of every
/// player turn, with no first-turn skip (atEndOfRound's skip is the monster branch).
#[test]
fn fuzzfix_c_player_ritual_has_no_first_turn_skip() {
    for character in ALL {
        let mut combat = fixture(character, &[(Strike, 0)], vec![], vec![None; 3], 3);
        combat.state.player_full_powers.push(FullPowerEntry { id: FullPowerId("ritual"),
            state: PowerState { amount: 2, skip_first: true, monster_source_grace: false } });
        end_turn(&mut combat);
        assert_eq!(player_power(&combat, "strength"), 2, "{character:?}");
    }
}

/// EstablishmentPowerAction discounts cards marked retained when it runs; Equilibrium marks the
/// hand immediately in its callback, so either list order discounts (base: never).
#[test]
fn fuzzfix_c_establishment_discounts_equilibrium_retained_cards_both_ways() {
    for character in ALL {
        for establishment_first in [true, false] {
            let mut combat = fixture(character, &[(DefendPurple, 0)], vec![], vec![None; 3], 3);
            let (establishment, equilibrium) = (power("establishment", 1), power("equilibrium", 1));
            combat.state.player_full_powers.extend(if establishment_first { [establishment, equilibrium] } else { [equilibrium, establishment] });
            end_turn(&mut combat);
            let defend = combat.state.piles.hand.iter().find_map(|owned| owned.full().filter(|card| card.id == DefendPurple)).expect("retained Defend");
            assert_eq!(card_state::effective_cost(defend, 1).unwrap(), 0, "{character:?} establishment_first={establishment_first}");
        }
    }
}

fn hand_handles(combat: &Combat) -> Vec<CardHandle> { combat.state.piles.hand.iter().map(OwnedCard::handle).collect() }

/// DiscardAtEndOfTurnAction queues each Ethereal exhaust at the top, so it resolves before the
/// discards (source-check-endturn-v2); base moved the leftmost Dazed after the rightmost Strike.
#[test]
fn fuzzfix_c_ethereal_exhaust_resolves_before_the_discard() {
    for character in ALL {
        let mut combat = fixture(character, &[(Dazed, 0), (Strike, 0)], vec![], vec![None; 3], 3);
        let effects = end_turn(&mut combat);
        let moved = |after: PublicZone| at(&effects, "hand move", |event| matches!(&event.kind,
            ResolvedEffectKind::CardMove { before: PublicZone::Hand, after: zone, .. } if *zone == after));
        assert!(moved(PublicZone::Exhaust) < moved(PublicZone::Discard), "{character:?}: {effects:?}");
        let hooks = &combat.state.hook_events;
        let exhausted = hooks.iter().position(|event| event.hook == Hook::CardExhausted).expect("Dazed exhausts");
        let discarded = hooks.iter().position(|event| event.hook == Hook::CardDiscarded).expect("Strike discards");
        assert!(exhausted < discarded, "{character:?}: {hooks:?}");
    }
}

/// The unseeded source shuffle of the remaining hand is modelled on a stream keyed by the
/// combat's shuffle key and the turn: deterministic, turn-dependent, and it reads no cursor.
#[test]
fn fuzzfix_c_end_turn_hand_shuffle_is_keyed_by_combat_and_turn() {
    let cards = [(Dazed, 0), (Dazed, 0), (Dazed, 0), (Dazed, 0), (Dazed, 0), (Dazed, 0)];
    let mut combat = fixture(Character::Silent, &cards, vec![], vec![None; 3], 3);
    let hand = hand_handles(&combat);
    let (shuffle, card_rng) = (combat.state.rng.clone(), combat.state.card_rng.clone());
    let first = piles::end_turn_visit_order(&combat.state, &hand);
    assert_eq!(piles::end_turn_visit_order(&combat.state, &hand), first, "same combat and turn, same order");
    assert_eq!((combat.state.rng.clone(), combat.state.card_rng.clone()), (shuffle, card_rng), "no stream advances");
    let orders = (2..10).map(|turn| { combat.state.turn = turn; piles::end_turn_visit_order(&combat.state, &hand) }).collect::<Vec<_>>();
    assert!(orders.iter().any(|order| *order != first), "the order is keyed by the turn: {first:?} {orders:?}");
    let mut sorted = first.clone();
    sorted.sort();
    let mut expected = hand.clone();
    expected.sort();
    assert_eq!(sorted, expected, "a permutation of the remaining hand");
}

/// The Ethereal exhausts follow the reverse of the keyed visit order (each is queued at the
/// top), and the exhaust pile records them in that order.
#[test]
fn fuzzfix_c_ethereal_exhausts_follow_the_keyed_visit_order() {
    for character in ALL {
        let cards = [(Dazed, 0), (Strike, 0), (Dazed, 0), (Dazed, 0), (Strike, 0)];
        let mut combat = fixture(character, &cards, vec![], vec![None; 3], 3);
        let hand = hand_handles(&combat);
        let dazed = |handle: &CardHandle| combat.state.piles.hand.iter().any(|card| card.handle() == *handle
            && card.full().is_some_and(|live| live.id == Dazed));
        let expected = piles::end_turn_visit_order(&combat.state, &hand).into_iter().rev().filter(dazed).collect::<Vec<_>>();
        end_turn(&mut combat);
        let exhausted = combat.state.piles.exhaust.iter().map(OwnedCard::handle).collect::<Vec<_>>();
        assert_eq!(exhausted, expected, "{character:?}");
    }
}

fn with_combust(combat: &mut Combat) {
    combat.state.player_full_powers.push(power("combust", 5));
    combat.state.combust = Some(model::CombustState { copies: 1 });
}

fn card_moved(card: u64, before: PublicZone, after: PublicZone) -> impl Fn(&ResolvedEffect) -> bool {
    move |event| matches!(&event.kind, ResolvedEffectKind::CardMove { card: moved, before: from, after: to }
        if *moved == card_state::handle(CardInstanceId(card)) && *from == before && *to == after)
}

fn in_hand(combat: &Combat, card: u64) -> bool {
    combat.state.piles.hand.iter().any(|owned| owned.handle() == card_state::handle(CardInstanceId(card)))
}

/// AbstractRoom.endTurn runs the atEndOfTurn pass synchronously, then queues the discard at the
/// bottom, so a bottom child of the pass's work lands after the discard: Gremlin Horn's addToBot
/// Energy and Draw, raised when an atEndOfTurn power kills one of two monsters, follow the
/// discard and the drawn card is kept (base agreed; v1 drew it before the discard).
#[test]
fn fuzzfix_c_gremlin_horn_work_from_the_end_turn_pass_follows_the_discard() {
    let cards = [(Strike, 0), (Strike, 0), (Defend, 0), (Defend, 0), (Defend, 0), (Defend, 0), (Defend, 0), (Defend, 0)];
    for (killer, character) in ALL.into_iter().map(|character| ("combust", character)).chain(ALL.into_iter().map(|character| ("omega", character))) {
        let mut combat = fixture_in(EncounterId::SmallSlimes, 1, character, &cards, vec![RelicId::GremlinHorn], vec![None; 3], 3);
        arrange(&mut combat, &[1, 2], &[3, 4, 5, 6, 7, 8], &[]);
        combat.state.enemies[0].hp = 1;
        if killer == "combust" { with_combust(&mut combat); } else { combat.state.player_full_powers.push(power("omega", 5)); }
        let effects = end_turn(&mut combat);
        let discarded = at(&effects, "discard", card_moved(1, PublicZone::Hand, PublicZone::Discard));
        let energy = at(&effects, "Horn energy", |event| matches!(event.kind, ResolvedEffectKind::Energy { before, after } if after == before + 1));
        assert!(discarded < energy, "{killer} {character:?}: Horn's energy follows the discard: {effects:?}");
        assert!(discarded < at(&effects, "Horn draw", card_moved(8, PublicZone::Unknown, PublicZone::Hand)), "{killer} {character:?}");
        assert!(in_hand(&combat, 8) && combat.state.piles.hand.len() == 6, "{killer} {character:?}: Horn's card is kept");
    }
}

/// CentennialPuzzle.wasHPLost queues its Draw 3 at the top: Combust's HP loss draws the three
/// cards before the discard, so they are discarded with the hand (base drew them after it).
#[test]
fn fuzzfix_c_centennial_puzzle_draw_is_top_work() {
    let cards = [(Strike, 0), (Strike, 0), (Defend, 0), (Defend, 0), (Defend, 0), (Defend, 0), (Defend, 0), (Defend, 0)];
    for character in ALL {
        let mut combat = fixture(character, &cards, vec![RelicId::CentennialPuzzle], vec![None; 3], 3);
        arrange(&mut combat, &[1, 2], &[3, 4, 5, 6, 7, 8], &[]);
        with_combust(&mut combat);
        let effects = end_turn(&mut combat);
        assert!(at(&effects, "Puzzle draw", card_moved(8, PublicZone::Unknown, PublicZone::Hand))
            < at(&effects, "discard", card_moved(1, PublicZone::Hand, PublicZone::Discard)), "{character:?}: {effects:?}");
        assert!(at(&effects, "drawn card discarded", card_moved(8, PublicZone::Hand, PublicZone::Discard))
            < at(&effects, "monster turn", |event| matches!(event.kind, ResolvedEffectKind::EnemyMove { .. })), "{character:?}");
        assert_eq!(combat.state.piles.hand.len(), 5, "{character:?}");
    }
}

/// Ghostly Armor's and Entrench's triggerOnEndOfPlayerTurn queue only ExhaustAllEtherealAction
/// at the top (no super call; Entrench's is unconditional). It queues each Ethereal card still in
/// hand, in hand order, at the top, so those exhaust in reverse hand order. Source queue model
/// over the keyed visit order: each visited card pushes its action; the stack then pops.
#[test]
fn fuzzfix_c_exhaust_all_ethereal_takes_the_remaining_ethereal_in_reverse_hand_order() {
    for (cards, character) in [[(Dazed, 0), (Dazed, 0), (GhostlyArmor, 0)], [(Dazed, 0), (Entrench, 0), (Dazed, 0)]]
        .into_iter().flat_map(|cards| ALL.into_iter().map(move |character| (cards, character))) {
        for turn in 1..=12 {
            let mut combat = fixture(character, &cards, vec![], vec![None; 3], 3);
            combat.state.turn = turn;
            let hand = hand_handles(&combat);
            let id = |handle: &CardHandle| combat.state.piles.hand.iter().find(|card| card.handle() == *handle)
                .and_then(|card| card.full()).map(|live| (live.id, live.local.ethereal)).unwrap();
            let mut stack = piles::end_turn_visit_order(&combat.state, &hand).into_iter().filter_map(|handle| match id(&handle) {
                (GhostlyArmor | Entrench, _) => Some(None), (_, true) => Some(Some(handle)), _ => None }).collect::<Vec<_>>();
            let mut expected = Vec::new();
            while let Some(top) = stack.pop() {
                match top {
                    Some(handle) => if !expected.contains(&handle) { expected.push(handle) },
                    None => stack.extend(hand.iter().filter(|handle| id(handle).1 && !expected.contains(handle)).cloned().map(Some)),
                }
            }
            end_turn(&mut combat);
            let exhausted = combat.state.piles.exhaust.iter().map(OwnedCard::handle).collect::<Vec<_>>();
            assert_eq!(exhausted, expected, "{character:?} turn {turn} {cards:?}");
        }
    }
}
