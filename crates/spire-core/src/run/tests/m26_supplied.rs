//! M26 Task B2 supplied mixed decks. The fixed legal-action policy and all fixture
//! card identities are literal so this lane exercises the real Run and combat owners.
use crate::{
    combat::{CombatAction, Outcome},
    content::{CardId, Catalog, Character, EncounterId},
    run::{diagnostics, AuthoredRunStart, CardInstance, GameAction, Run},
};
use std::collections::BTreeMap;

use CardId::*;

fn card(instance: u64, id: CardId, misc: i32) -> CardInstance {
    CardInstance { instance, id, upgrades: 0, misc }
}

fn supplied(deck: Vec<CardInstance>) -> AuthoredRunStart {
    AuthoredRunStart {
        hp: 80,
        max_hp: 80,
        gold: 99,
        deck,
        relics: vec![],
        potions: vec![None; 3],
        relic_counters: Default::default(),
    }
}

/// The Run contract exposes its legal actions through decision candidates in legal order.
/// A confirm wins when present; selections are always the first candidate, never a deselect.
fn next_action(run: &Run) -> CombatAction {
    let decision = run.decision().expect("active supplied combat has a decision");
    let legal = decision.candidates.into_iter().filter_map(|candidate| match candidate.action {
        GameAction::Combat(action) => Some(action),
        _ => None,
    }).collect::<Vec<_>>();

    if let Some(confirm) = legal.iter().find(|action| matches!(action, CombatAction::ConfirmSelection)) {
        return confirm.clone();
    }
    legal.into_iter()
        .find(|action| !matches!(action, CombatAction::EndTurn | CombatAction::DeselectCard { .. }))
        .unwrap_or(CombatAction::EndTurn)
}

#[derive(Default)]
struct JournalSeen {
    mutations: usize,
    upgrades: usize,
    notes: Vec<String>,
}

/// Copy only changes observed in the active combat's journals before Run settlement.
fn observe_journals(run: &Run, expected: &mut [CardInstance], seen: &mut JournalSeen) {
    let combat = run.state.combat.as_ref().expect("journal observation precedes settlement");
    let mutations = combat.permanent_card_mutations();
    for mutation in mutations.iter().skip(seen.mutations) {
        let owner = expected.iter_mut().find(|card| card.instance == mutation.instance.0)
            .unwrap_or_else(|| panic!("journal owner instance {} is in the supplied deck", mutation.instance.0));
        assert_eq!(owner.id, mutation.id, "permanent mutation keeps its supplied card kind");
        owner.misc = mutation.misc;
        seen.notes.push(format!("{}#{} misc={}", mutation.id.as_str(), mutation.instance.0, mutation.misc));
    }
    seen.mutations = mutations.len();

    let upgrades = combat.permanent_card_upgrades();
    for upgrade in upgrades.iter().skip(seen.upgrades) {
        let owner = expected.iter_mut().find(|card| card.instance == upgrade.instance.0)
            .unwrap_or_else(|| panic!("upgrade journal owner instance {} is in the supplied deck", upgrade.instance.0));
        assert_eq!(owner.id, upgrade.id, "permanent upgrade keeps its supplied card kind");
        owner.upgrades = owner.upgrades.checked_add(1).expect("supplied upgrade count fits");
        seen.notes.push(format!("{}#{} upgrade={}", upgrade.id.as_str(), upgrade.instance.0, owner.upgrades));
    }
    seen.upgrades = upgrades.len();
}

fn deck_by_instance(deck: &[CardInstance]) -> BTreeMap<u64, CardInstance> {
    deck.iter().cloned().map(|card| (card.instance, card)).collect()
}

fn play_supplied(character: Character, supplied_deck: Vec<CardInstance>) {
    assert_eq!(supplied_deck.len(), 15, "B2 supplied deck has exactly 15 literal cards");
    let mut run = diagnostics::complete_combat(character, supplied(supplied_deck.clone()), EncounterId::Cultist)
        .unwrap_or_else(|error| panic!("complete {character:?}/Cultist combat: {error:?}"));
    let mut expected_deck = supplied_deck;
    let mut journal = JournalSeen::default();
    let mut actions = 0usize;
    let mut turns = 0u32;
    let mut played_by_kind = BTreeMap::<String, u32>::new();
    let mut played_by_id = BTreeMap::<String, u32>::new();
    let mut offcolour_stance_entered = false;
    let mut offcolour_orb_channelled = false;

    while run.state.combat.is_some() && actions < 400 {
        observe_journals(&run, &mut expected_deck, &mut journal);
        let combat = run.state.combat.as_ref().expect("loop is active");
        assert!(combat.fault_record().is_none(), "{character:?} combat has no fault before action {actions}");
        let before = combat.public_full().expect("full combat view");
        turns = turns.max(before.turn);
        let action = next_action(&run);
        let played_id = if let CombatAction::PlayCard { instance, .. } = &action {
            let played = before.hand.iter().find(|card| card.instance.0 == *instance)
                .unwrap_or_else(|| panic!("legal PlayCard instance {instance} is in the current hand"));
            let kind = Catalog::standard().card(played.id).expect("played card has catalog row").card_type;
            *played_by_kind.entry(format!("{kind:?}")).or_default() += 1;
            *played_by_id.entry(played.id.as_str().to_owned()).or_default() += 1;
            Some(played.id)
        } else {
            None
        };
        let before_stance = before.character.stance;
        let before_orbs = before.character.orbs.clone();
        run.choose_combat(action.clone())
            .unwrap_or_else(|error| panic!("{character:?} action {actions} {action:?}: {error:?}"));
        actions += 1;
        assert!(run.state.fault.is_none(), "{character:?} Run has no fault after action {actions}");

        if let Some(active) = run.state.combat.as_ref() {
            assert!(active.fault_record().is_none(), "{character:?} combat has no fault after action {actions}");
            let after = active.public_full().expect("full combat view after accepted action");
            turns = turns.max(after.turn);
            if played_id == Some(Crescendo) && character != Character::Watcher
                && before_stance != after.character.stance {
                offcolour_stance_entered = true;
            }
            if played_id == Some(ColdSnap) && character != Character::Defect
                && before_orbs != after.character.orbs {
                offcolour_orb_channelled = true;
            }
            observe_journals(&run, &mut expected_deck, &mut journal);
        } else if matches!(run.state.last_combat.as_ref().map(|record| record.outcome), Some(Outcome::Victory)) {
            // Main (B2 review): a Ritual Dagger Fatal that kills the last Cultist settles at once,
            // so its journal is not observable first; expect the literal growth instead
            // (RitualDagger base Fatal +3, design §4.2; supplied decks hold it unupgraded).
            if played_id == Some(RitualDagger) {
                let dagger = expected_deck.iter_mut().find(|card| card.id == RitualDagger).expect("supplied Ritual Dagger");
                assert_eq!(dagger.upgrades, 0, "supplied Ritual Dagger is unupgraded");
                dagger.misc += 3;
                journal.notes.push("final Fatal: RitualDagger +3 (literal)".to_owned());
            }
        }
    }

    assert!(run.state.combat.is_none(), "{character:?} reached the 400-action bound while combat remained active");
    assert_eq!(run.state.combat_progress.entered, 1, "{character:?} entered one supplied combat");
    assert_eq!(run.state.combat_progress.settled, 1, "{character:?} settlement passed once");
    assert!(run.state.fault.is_none(), "{character:?} Run has no fault after settlement");
    let outcome = run.state.last_combat.as_ref().expect("settled combat record").outcome;
    assert!(matches!(outcome, Outcome::Victory | Outcome::Death), "{character:?} ends in Victory or Death: {outcome:?}");
    assert_eq!(
        deck_by_instance(&run.state.deck),
        deck_by_instance(&expected_deck),
        "{character:?} Run deck is the supplied literal plus its observed combat journals",
    );

    println!(
        "Supplied played {character:?}: encounter=Cultist outcome={outcome:?} actions={actions} turns={turns} cards_by_kind={played_by_kind:?} cards_by_id={played_by_id:?} offcolour_stance_entered={offcolour_stance_entered} offcolour_orb_channelled={offcolour_orb_channelled} journal={:?}",
        journal.notes,
    );
}

#[test]
fn m26_supplied_mixed_deck_ironclad() {
    play_supplied(Character::Ironclad, vec![
        card(1, RitualDagger, 15), card(2, Bite, 0), card(3, FlashOfSteel, 0),
        card(4, BandageUp, 0), card(5, CloakAndDagger, 0), card(6, ColdSnap, 0),
        card(7, Crescendo, 0), card(8, Pain, 0), card(9, Decay, 0),
        card(10, Strike, 0), card(11, Strike, 0), card(12, Strike, 0),
        card(13, Defend, 0), card(14, Defend, 0), card(15, Bash, 0),
    ]);
}

#[test]
fn m26_supplied_mixed_deck_silent() {
    play_supplied(Character::Silent, vec![
        card(1, RitualDagger, 15), card(2, Bite, 0), card(3, FlashOfSteel, 0),
        card(4, BandageUp, 0), card(5, TrueGrit, 0), card(6, ColdSnap, 0),
        card(7, Crescendo, 0), card(8, Doubt, 0), card(9, Regret, 0),
        card(10, StrikeGreen, 0), card(11, StrikeGreen, 0), card(12, StrikeGreen, 0),
        card(13, DefendGreen, 0), card(14, DefendGreen, 0), card(15, Neutralize, 0),
    ]);
}

#[test]
fn m26_supplied_mixed_deck_defect() {
    play_supplied(Character::Defect, vec![
        card(1, RitualDagger, 15), card(2, Bite, 0), card(3, Trip, 0),
        card(4, BandageUp, 0), card(5, TrueGrit, 0), card(6, CloakAndDagger, 0),
        card(7, Crescendo, 0), card(8, Normality, 0), card(9, Writhe, 0),
        card(10, StrikeBlue, 0), card(11, DefendBlue, 0), card(12, DefendBlue, 0),
        card(13, DefendBlue, 0), card(14, DefendBlue, 0), card(15, Zap, 0),
    ]);
}

#[test]
fn m26_supplied_mixed_deck_watcher() {
    play_supplied(Character::Watcher, vec![
        card(1, RitualDagger, 15), card(2, Bite, 0), card(3, Trip, 0),
        card(4, BandageUp, 0), card(5, TrueGrit, 0), card(6, CloakAndDagger, 0),
        card(7, ColdSnap, 0), card(8, Shame, 0), card(9, Clumsy, 0),
        card(10, StrikePurple, 0), card(11, DefendPurple, 0), card(12, DefendPurple, 0),
        card(13, DefendPurple, 0), card(14, DefendPurple, 0), card(15, Vigilance, 0),
    ]);
}
