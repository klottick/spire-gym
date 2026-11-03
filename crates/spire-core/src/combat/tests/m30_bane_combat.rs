//! M30 direct Ascender's Bane combat lifecycle coverage. Expected literals follow the M30 source check.
use super::super::*;
use crate::combat::{CardInstanceId, Combat, CombatAction, FullPowerEntry, FullPowerId, PowerState, TargetId};
use crate::combat::model::OwnedCard;
use crate::content::{CardId, Character, EncounterId, PotionId, RelicId};
use crate::items::tests::m27_support;
use crate::run::{CardInstance, Run};
use std::collections::BTreeMap;

fn battle(ids: &[CardId], relics: Vec<RelicId>, character: Character) -> Run {
    let mut run = m27_support::complete_run(character, relics, vec![Some(PotionId::FirePotion); 3],
        BTreeMap::new(), EncounterId::Cultist);
    for slot in 0..3 {
        if run.state().combat.is_none() { break; }
        run.choose_combat(CombatAction::UsePotion { slot, target: Some(TargetId(1)) })
            .unwrap_or_else(|error| panic!("settle fixture combat with Fire Potion {slot}: {error:?}"));
    }
    assert!(run.state().combat.is_none(), "the fixture combat settles before installing the Bane deck");
    {
        let state = run.state_mut_for_test();
        state.deck = ids.iter().enumerate().map(|(index, id)| CardInstance {
            instance: index as u64 + 1, id: *id, upgrades: 0, misc: 0,
        }).collect();
        state.next_instance = ids.len() as u64 + 1;
        state.potions = vec![None; 3];
    }
    m27_support::next_complete_combat(&mut run, EncounterId::Cultist)
        .unwrap_or_else(|error| panic!("enter complete combat with Bane deck: {error:?}"));
    run
}

fn combat(run: &mut Run) -> &mut Combat {
    run.state_mut_for_test().combat.as_mut().expect("complete Run owns a live combat")
}

fn move_to_hand(run: &mut Run, instance: u64) {
    let state = &mut combat(run).state;
    let handle = card_state::handle(CardInstanceId(instance));
    if state.piles.hand.iter().any(|owned| owned.handle() == handle) { return; }
    let take = |pile: &mut Vec<OwnedCard>| pile.iter().position(|owned| owned.handle() == handle)
        .map(|index| pile.remove(index));
    let owned = take(&mut state.piles.draw).or_else(|| take(&mut state.piles.discard))
        .or_else(|| take(&mut state.piles.exhaust)).expect("Bane deck instance has one combat-pile owner");
    state.piles.hand.push(owned);
    piles::refresh_hand_values(state);
}

fn keep_only_in_hand(run: &mut Run, instance: u64) {
    move_to_hand(run, instance);
    let state = &mut combat(run).state;
    let handle = card_state::handle(CardInstanceId(instance));
    let mut keep = Vec::new();
    for owned in std::mem::take(&mut state.piles.hand) {
        if owned.handle() == handle { keep.push(owned); } else { state.piles.draw.push(owned); }
    }
    state.piles.hand = keep;
    piles::refresh_hand_values(state);
}

fn power(id: &'static str, amount: i32) -> FullPowerEntry {
    FullPowerEntry { id: FullPowerId(id), state: PowerState { amount, skip_first: false, monster_source_grace: false } }
}

fn has_card(pile: &[OwnedCard], instance: u64, id: CardId) -> bool {
    pile.iter().any(|owned| owned.handle() == card_state::handle(CardInstanceId(instance))
        && owned.full().is_some_and(|live| live.id == id))
}

fn power_amount(run: &mut Run, id: &'static str) -> i32 {
    combat(run).state.player_full_powers.iter().find(|entry| entry.id.0 == id)
        .map_or(0, |entry| entry.state.amount)
}

fn play(run: &mut Run, instance: u64) {
    combat(run).step_full(&CombatAction::PlayCard { instance, target: None })
        .unwrap_or_else(|error| panic!("play {instance}: {error:?}"));
}

// Source: m30-source-check-v1.md §C; verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/cards/curses/AscendersBane.java:28–40; m30-engine-audit-v1.md §4 (shared exhaust hooks and cleanup).
#[test]
fn m30_bane_combat_ethereal_exhaust_fires_exhaust_hooks() {
    let ids = [CardId::AscendersBane, CardId::Defend, CardId::Defend, CardId::Defend,
        CardId::Defend, CardId::Defend, CardId::Defend];
    let mut run = battle(&ids, vec![], Character::Ironclad);
    keep_only_in_hand(&mut run, 1);
    combat(&mut run).state.player_full_powers.extend([
        power("feel_no_pain", 3), power("dark_embrace", 1), power("barricade", 1),
    ]);
    let bane = card_state::handle(CardInstanceId(1));

    combat(&mut run).step_full(&CombatAction::EndTurn).expect("end turn resolves");

    let state = &combat(&mut run).state;
    assert!(has_card(&state.piles.exhaust, 1, CardId::AscendersBane), "Ethereal Bane exhausts");
    assert!(!has_card(&state.piles.discard, 1, CardId::AscendersBane), "Bane is not discarded");
    assert_eq!(state.hook_events.iter().filter(|event|
        event.hook == Hook::CardExhausted && event.source == Source::Card(bane.clone())).count(), 1,
        "one exhaust notification fires for Bane");
    assert_eq!(state.player.block, 3, "Feel No Pain grants 3 Block for Bane's exhaust");
    assert_eq!(state.piles.hand.len(), 6,
        "Dark Embrace draws one for the exhaust, in addition to the next turn's five-card draw");
    assert_eq!(power_amount(&mut run, "feel_no_pain"), 3);
    assert_eq!(power_amount(&mut run, "dark_embrace"), 1);
}

// Source: verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/relics/BlueCandle.java:40–49; verification/2026-11-02-m28e-c-v1/decompiled/com/megacrit/cardcrawl/cards/AbstractCard.java:839–875, 895–909.
#[test]
fn m30_bane_combat_blue_candle_plays_it() {
    let mut ordinary = battle(&[CardId::AscendersBane], vec![], Character::Ironclad);
    move_to_hand(&mut ordinary, 1);
    assert!(!combat(&mut ordinary).legal_actions_full().unwrap().iter().any(|action|
        matches!(action, CombatAction::PlayCard { instance: 1, .. })), "Bane is not playable without Blue Candle");

    let mut candle = battle(&[CardId::AscendersBane], vec![RelicId::BlueCandle], Character::Ironclad);
    move_to_hand(&mut candle, 1);
    assert!(combat(&mut candle).legal_actions_full().unwrap().iter().any(|action|
        matches!(action, CombatAction::PlayCard { instance: 1, .. })), "Blue Candle makes the Curse playable");
    let hp = combat(&mut candle).state.player.hp;
    play(&mut candle, 1);
    assert_eq!(combat(&mut candle).state.player.hp, hp - 1, "Blue Candle costs 1 HP");
    assert!(has_card(&combat(&mut candle).state.piles.exhaust, 1, CardId::AscendersBane), "Blue Candle exhausts the played Curse");
}

// Source: verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/relics/MedicalKit.java:30–37; verification/2026-11-02-m28e-c-v1/decompiled/com/megacrit/cardcrawl/cards/AbstractCard.java:895–909.
#[test]
fn m30_bane_combat_medical_kit_does_not_enable() {
    let mut run = battle(&[CardId::AscendersBane], vec![RelicId::MedicalKit], Character::Ironclad);
    move_to_hand(&mut run, 1);
    assert!(!combat(&mut run).legal_actions_full().unwrap().iter().any(|action|
        matches!(action, CombatAction::PlayCard { instance: 1, .. })), "Medical Kit permits Status cards, not Curses");
    assert!(combat(&mut run).step_full(&CombatAction::PlayCard { instance: 1, target: None }).is_err(),
        "Medical Kit rejects Bane as a Curse");
    assert!(has_card(&combat(&mut run).state.piles.hand, 1, CardId::AscendersBane), "rejected play leaves Bane in hand");
}

// Source: verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/relics/RunicPyramid.java:15–35; verification/2026-11-03-endturn-v1/src/com/megacrit/cardcrawl/actions/common/DiscardAtEndOfTurnAction.java:26–48.
#[test]
fn m30_bane_combat_runic_pyramid_still_exhausts() {
    let mut run = battle(&[CardId::AscendersBane], vec![RelicId::RunicPyramid], Character::Ironclad);
    move_to_hand(&mut run, 1);
    combat(&mut run).step_full(&CombatAction::EndTurn).expect("Runic Pyramid end turn resolves");
    assert!(has_card(&combat(&mut run).state.piles.exhaust, 1, CardId::AscendersBane),
        "Runic Pyramid does not retain an Ethereal Curse past end turn");
    assert!(!has_card(&combat(&mut run).state.piles.hand, 1, CardId::AscendersBane));
}

// Source: m30-source-check-v1.md §C; verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/cards/curses/AscendersBane.java:28–40; crates/spire-core/src/combat/tests/m10_selection.rs:430–435.
#[test]
fn m30_bane_combat_exhume_returns_and_exhausts_again() {
    let mut run = battle(&[CardId::AscendersBane, CardId::Exhume], vec![], Character::Ironclad);
    move_to_hand(&mut run, 1);
    move_to_hand(&mut run, 2);
    combat(&mut run).step_full(&CombatAction::EndTurn).expect("first end turn exhausts Bane");
    assert!(has_card(&combat(&mut run).state.piles.exhaust, 1, CardId::AscendersBane));
    assert!(has_card(&combat(&mut run).state.piles.hand, 2, CardId::Exhume), "Exhume is drawn on the following player turn");

    play(&mut run, 2);
    assert!(has_card(&combat(&mut run).state.piles.hand, 1, CardId::AscendersBane), "Exhume returns the same Bane instance");
    assert!(!has_card(&combat(&mut run).state.piles.exhaust, 1, CardId::AscendersBane));
    combat(&mut run).step_full(&CombatAction::EndTurn).expect("second end turn exhausts returned Bane");
    assert!(has_card(&combat(&mut run).state.piles.exhaust, 1, CardId::AscendersBane));
    let bane = card_state::handle(CardInstanceId(1));
    assert_eq!(combat(&mut run).state.hook_events.iter().filter(|event|
        event.hook == Hook::CardExhausted && event.source == Source::Card(bane.clone())).count(), 2,
        "the original and returned Bane each exhaust once");
}

// Source: verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/relics/DuVuDoll.java:53–71.
#[test]
fn m30_bane_combat_du_vu_doll_counts_it() {
    let mut run = battle(&[CardId::AscendersBane], vec![RelicId::DuVuDoll], Character::Ironclad);
    assert_eq!(power_amount(&mut run, "strength"), 1, "Bane is the only Curse and grants Du-Vu Doll 1 Strength");
}

// Source: m30-source-check-v1.md §C; RetainCardsAction.java:34; EquilibriumPower.java:41; crates/spire-core/src/combat/effects.rs:584–592.
#[test]
fn m30_bane_combat_retain_effects_do_not_save_bane() {
    // The hand select screen offers every hand card, Ethereal ones included (RetainCardsAction.java:27);
    // only a non-Ethereal pick is marked retain (:34). Picking the Bane does not save it. (Corrected by
    // the main senior: the v1 brief wrongly expected the Bane to be filtered out of the choice.)
    for pick in [2_u64, 3] {
        let mut plans = battle(&[CardId::WellLaidPlans, CardId::AscendersBane, CardId::Defend], vec![], Character::Silent);
        for instance in 1..=3 { move_to_hand(&mut plans, instance); }
        play(&mut plans, 1);
        combat(&mut plans).step_full(&CombatAction::EndTurn).expect("Well-Laid Plans reaches its retain choice");
        let selection = combat(&mut plans).state.pending_selection.clone().expect("retain choice is open");
        assert!(selection.candidates.contains(&card_state::handle(CardInstanceId(2))), "the Bane is a candidate, as in the source");
        assert!(selection.candidates.contains(&card_state::handle(CardInstanceId(3))), "Defend is a candidate");
        combat(&mut plans).step_full(&CombatAction::SelectCard { instance: pick }).expect("pick");
        combat(&mut plans).step_full(&CombatAction::ConfirmSelection).expect("finish retain selection");
        assert!(has_card(&combat(&mut plans).state.piles.exhaust, 2, CardId::AscendersBane), "pick {pick}: Well-Laid Plans cannot save the Bane");
    }

    let mut equilibrium = battle(&[CardId::Equilibrium, CardId::AscendersBane, CardId::Defend], vec![], Character::Defect);
    for instance in 1..=3 { move_to_hand(&mut equilibrium, instance); }
    play(&mut equilibrium, 1);
    assert_eq!(power_amount(&mut equilibrium, "equilibrium"), 1, "Equilibrium power is active");
    combat(&mut equilibrium).step_full(&CombatAction::EndTurn).expect("Equilibrium end turn resolves");
    assert!(has_card(&combat(&mut equilibrium).state.piles.exhaust, 2, CardId::AscendersBane), "Equilibrium cannot save Ethereal Bane");
    assert!(has_card(&combat(&mut equilibrium).state.piles.hand, 3, CardId::Defend), "Equilibrium still retains a non-Ethereal card");
}
