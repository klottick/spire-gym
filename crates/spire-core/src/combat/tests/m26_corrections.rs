//! M26 final-review v1 corrections (I1-I3, M1, M3), every character under the complete overlay.
//! Each test fails on the candidate-v1 source (review probes in
//! `m26-independent-senior-review-20261101/notes/`).
use super::super::*;
use super::m26_support::CHARACTERS;
use crate::{content::{CardId, Character, EncounterId}, items::CombatItemContext};
use crate::combat::model::OwnedCard;
use CardId::*;

fn card(id: CardId, instance: u64, upgrades: u32) -> LiveCard {
    complete_live_card(id, CardInstanceId(instance), upgrades)
        .unwrap_or_else(|error| panic!("complete card {id:?}+{upgrades}: {error:?}"))
}

fn items(character: Character) -> Option<CombatItemContext> {
    let items = CombatItemContext::new(vec![], vec![None; 3]);
    let items = match character {
        Character::Ironclad => items,
        Character::Silent => items.with_m15_profile(),
        Character::Defect => items.with_m20_profile(),
        Character::Watcher => items.with_m25_profile(),
    };
    Some(items.with_complete_cards())
}

/// Jaw Worm, HP 80/80, energy 10. `hand` is the opening hand in order; `draw` is the draw
/// pile with its first element on top.
fn fight(character: Character, hand: &[(CardId, u32)], draw: &[(CardId, u32)]) -> Combat {
    let mut rng = crate::run::RunRng::new([41; 32], rng::Partition::Test, 0);
    let mut entry = enemies::setup(EncounterId::JawWorm, 1, 0, &mut rng, enemies::SetupContext {
        player: PlayerState { hp: 80, max_hp: 80, block: 0, energy: 10, max_energy: 10, draw_per_turn: 5, powers: vec![] },
        player_powers: vec![], character,
        cards: vec![permanent_live_card(Defend, CardInstanceId(99), 0)], opening_draw: 1, encounter_visit: 0,
    }).unwrap_or_else(|error| panic!("Jaw Worm setup: {error:?}"));
    let all = hand.iter().chain(draw).enumerate()
        .map(|(index, (id, upgrades))| card(*id, index as u64 + 1, *upgrades)).collect::<Vec<_>>();
    entry.cards = all;
    entry.opening_draw = 1;
    let mut combat = Combat::enter_with_item_context(entry, &mut items(character))
        .unwrap_or_else(|error| panic!("{character:?} complete-card combat entry: {error:?}"));
    arrange(&mut combat, hand.len());
    combat
}

/// Puts instances 1..=hand in hand in order and the rest in the draw pile, first on top.
fn arrange(combat: &mut Combat, hand: usize) {
    let mut cards = combat.state.piles.hand.drain(..).chain(combat.state.piles.draw.drain(..)).collect::<Vec<_>>();
    cards.sort_by_key(|owned| owned.full().map(|live| live.instance.0));
    let rest = cards.split_off(hand);
    combat.state.piles.hand = cards;
    combat.state.piles.draw = rest.into_iter().rev().collect();
}

fn power(combat: &Combat, id: &str) -> i32 {
    combat.state.player_full_powers.iter().find(|power| power.id.0 == id).map_or(0, |power| power.state.amount)
}

fn step(combat: &mut Combat, action: CombatAction, what: &str) {
    combat.step_full(&action).unwrap_or_else(|error| panic!("{what}: {error:?}; fault={:?}", combat.fault_record()));
}

fn in_hand(combat: &Combat, instance: u64) -> bool {
    combat.state.piles.hand.iter().any(|owned| owned.handle() == card_state::handle(CardInstanceId(instance)))
}

fn card_played(combat: &Combat) -> usize {
    combat.state.hook_events.iter().filter(|event| event.hook == Hook::CardPlayed).count()
}

/// I1: Doubt's Weak and Shame's Frail are just-applied (`WeakPower`/`FrailPower(p, 1, true)`), so
/// they skip the first round-end decrement and last through the player's next turn, then
/// expire at the next round end. The same law keeps a monster's Weak (its non-curse user).
#[test]
fn m26_end_turn_curse_debuffs_last_through_the_next_turn() {
    for character in CHARACTERS {
        let mut combat = fight(character, &[(Doubt, 0), (Shame, 0)], &[(Defend, 0); 10]);
        step(&mut combat, CombatAction::EndTurn, "end turn 1");
        assert_eq!((combat.state.turn, power(&combat, "weak"), power(&combat, "frail")), (2, 1, 1),
            "{character:?}: Doubt's Weak and Shame's Frail survive the first round end");
        assert!(!in_hand(&combat, 1) && !in_hand(&combat, 2), "{character:?}: the curses were discarded");
        step(&mut combat, CombatAction::EndTurn, "end turn 2");
        assert_eq!((power(&combat, "weak"), power(&combat, "frail")), (0, 0),
            "{character:?}: both expire at the second round end");
    }
    // Non-curse user of the law: a monster-sourced Weak keeps its grace for every character.
    for character in CHARACTERS {
        let mut combat = fight(character, &[(Defend, 0)], &[(Defend, 0); 10]);
        combat.state.pending.push_back(Effect {
            source: Source::Actor(Actor::Enemy(combat.state.enemies[0].instance_id.clone())),
            target: Some(Actor::Player), kind: EffectKind::ApplyFullPower { id: FullPowerId("weak"), amount: 1 } });
        combat.state.pending.push_back(Effect { source: Source::None, target: None, kind: EffectKind::Decision });
        engine::resolve(&mut combat.state);
        assert_eq!(power(&combat, "weak"), 1, "{character:?}: monster Weak applied");
        step(&mut combat, CombatAction::EndTurn, "end turn with a monster Weak");
        assert_eq!(power(&combat, "weak"), 1, "{character:?}: a monster's Weak still skips its first decrement");
    }
}

/// I2: every card that plays itself at end of turn discards itself even when retained
/// (M25 R4 generalized; source wiki pages and `implementation-notes.json` BURN and REGRET).
#[test]
fn m26_end_turn_self_playing_cards_discard_when_retained() {
    for character in CHARACTERS {
        for id in [Burn, Decay, Doubt, Shame, Regret] {
            let mut combat = fight(character, &[(id, 0), (Defend, 0)], &[(Defend, 0); 10]);
            combat.state.player_full_powers.push(FullPowerEntry { id: FullPowerId("retain_cards"),
                state: PowerState { amount: 1, skip_first: false, monster_source_grace: false } });
            step(&mut combat, CombatAction::EndTurn, "end turn");
            step(&mut combat, CombatAction::SelectCard { instance: 1 }, "retain the card");
            step(&mut combat, CombatAction::ConfirmSelection, "confirm retain");
            assert_eq!(combat.state.turn, 2, "{character:?} {id:?}: next player turn");
            assert!(!in_hand(&combat, 1), "{character:?} {id:?}: a retained self-playing card leaves the hand");
            assert!(combat.state.piles.discard.iter().chain(&combat.state.piles.draw)
                .any(|owned| owned.handle() == card_state::handle(CardInstanceId(1))),
                "{character:?} {id:?}: it discards itself");
            assert!(combat.fault_record().is_none(), "{character:?} {id:?}: no fault");
        }
    }
}

/// I3: the hit that downs the last Darkling is Fatal (source `Darkling.damage` clears
/// `halfDead` and kills every Darkling), for every Fatal card and character. The first two
/// kills stay non-Fatal.
#[test]
fn m26_fatal_fires_on_the_final_darkling() {
    for character in CHARACTERS {
        for id in [Feed, HandOfGreed, LessonLearned, RitualDagger] {
            let mut entry = super::m8_shapes::entry(EncounterId::ThreeDarklings);
            entry.setup.character = character;
            entry.cards = vec![card(id, 1, 0), card(id, 2, 0), card(id, 3, 0), card(Defend, 4, 0)];
            entry.opening_draw = 4;
            let mut combat = Combat::enter_with_item_context(entry, &mut items(character))
                .unwrap_or_else(|error| panic!("{character:?} Three Darklings entry: {error:?}"));
            combat.state.player.energy = 10;
            for enemy in combat.state.enemies.iter_mut() { enemy.hp = 1; enemy.block = 0; }
            let fatal = |combat: &Combat| (combat.state.player.max_hp,
                enemies::act1::thieves::current_gold(&combat.state),
                combat.state.watcher.permanent_upgrades.len(),
                combat.state.defect.permanent_mutations.len());
            let start = fatal(&combat);
            for (instance, target) in [(1u64, 1u64), (2, 2)] {
                step(&mut combat, CombatAction::PlayCard { instance, target: Some(TargetId(target)) }, "non-final kill");
            }
            assert_eq!(fatal(&combat), start, "{character:?} {id:?}: non-final Darkling kills are not Fatal");
            step(&mut combat, CombatAction::PlayCard { instance: 3, target: Some(TargetId(3)) }, "final kill");
            let end = fatal(&combat);
            let expected = match id {
                Feed => (start.0 + 3, start.1, start.2, start.3),
                HandOfGreed => (start.0, start.1.map(|gold| gold + 20), start.2, start.3),
                LessonLearned => (start.0, start.1, start.2 + 1, start.3),
                _ => (start.0, start.1, start.2, start.3 + 1),
            };
            assert_eq!(end, expected, "{character:?} {id:?}: the final Darkling kill is Fatal");
            assert_eq!(combat.state.outcome, Some(Outcome::Victory), "{character:?} {id:?}: the group is defeated");
            assert!(combat.fault_record().is_none(), "{character:?} {id:?}: no fault");
        }
    }
}

/// M1: Master Reality upgrades every created card that can still upgrade, so a Dual Wield
/// copy of Searing Blow+3 is +4 (source `upgrade()` has no once-guard for Searing Blow).
#[test]
fn m26_master_reality_upgrades_a_copied_searing_blow() {
    for character in CHARACTERS {
        let mut combat = fight(character, &[(DualWield, 0), (SearingBlow, 3)], &[(Defend, 0); 5]);
        combat.state.player_full_powers.push(FullPowerEntry { id: FullPowerId("master_reality"),
            state: PowerState { amount: -1, skip_first: false, monster_source_grace: false } });
        step(&mut combat, CombatAction::PlayCard { instance: 1, target: None }, "Dual Wield");
        if combat.state.pending_selection.is_some() {
            step(&mut combat, CombatAction::SelectCard { instance: 2 }, "choose Searing Blow");
            step(&mut combat, CombatAction::ConfirmSelection, "confirm");
        }
        let blows = combat.state.piles.hand.iter().filter_map(OwnedCard::full)
            .filter(|live| live.id == SearingBlow).map(|live| (live.instance.0 == 2, live.upgrades)).collect::<Vec<_>>();
        assert_eq!(blows.len(), 2, "{character:?}: original and one copy");
        assert!(blows.contains(&(true, 3)) && blows.contains(&(false, 4)), "{character:?}: copy is +4: {blows:?}");
    }
}

/// M3 (F10): Pain loses 1 HP for every other CardPlayed, autoplayed (Havoc) and duplicated
/// (Double Tap) plays included, and not for end-of-turn auto-use by statuses and curses.
#[test]
fn m26_pain_autoplay_duplicate_and_end_of_turn() {
    for character in CHARACTERS {
        let mut havoc = fight(character, &[(Pain, 0), (Havoc, 0)], &[(Defend, 0)]);
        let hp = havoc.state.player.hp;
        step(&mut havoc, CombatAction::PlayCard { instance: 2, target: None }, "Havoc");
        assert_eq!((card_played(&havoc), hp - havoc.state.player.hp), (2, 2),
            "{character:?}: Havoc and its autoplayed Defend each trigger Pain");

        let mut double_tap = fight(character, &[(Pain, 0), (DoubleTap, 0), (Strike, 0)], &[(Defend, 0); 5]);
        let hp = double_tap.state.player.hp;
        step(&mut double_tap, CombatAction::PlayCard { instance: 2, target: None }, "Double Tap");
        step(&mut double_tap, CombatAction::PlayCard { instance: 3, target: Some(TargetId(1)) }, "Strike");
        assert_eq!((card_played(&double_tap), hp - double_tap.state.player.hp), (3, 3),
            "{character:?}: Double Tap, Strike and its duplicate each trigger Pain");

        let mut end = fight(character, &[(Pain, 0), (Burn, 0), (Decay, 0), (Doubt, 0), (Shame, 0)], &[(Defend, 0); 10]);
        end.state.player.block = 999;
        let hp = end.state.player.hp;
        step(&mut end, CombatAction::EndTurn, "end turn");
        assert_eq!((card_played(&end), end.state.player.hp), (0, hp),
            "{character:?}: end-of-turn auto-use is not a CardPlayed and does not trigger Pain");
        assert_eq!(power(&end, "weak"), 1, "{character:?}: the curses still resolved");
    }
}
