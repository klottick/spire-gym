//! Review-fix regressions for fuzz chunk A: the shared selection suspension
//! record (C1/I1), origin-tagged manual-discard callbacks (C2), and autoplayed
//! X-cost capture for a whole burst without any energy drain (I2).
use super::super::*;
use crate::{combat::{card_state, model::OwnedCard, CardInstanceId, CombatAction, Hook},
    content::{CardId, Character, EncounterId, PotionId, RelicId}, items::CombatItemContext, run::RunRng};
use CardId::*;

fn fixture(character: Character, cards: &[(CardId, u32)], relics: Vec<RelicId>, potions: Vec<Option<PotionId>>, energy: i32) -> Combat {
    let mut entry = enemies::setup(EncounterId::Cultist, 1, 0, &mut RunRng::new([0x41; 32], rng::Partition::Test, 0),
        enemies::SetupContext {
            player: PlayerState { hp: 80, max_hp: 80, block: 0, energy, max_energy: energy, draw_per_turn: 5, powers: vec![] },
            player_powers: vec![], character, cards: vec![permanent_live_card(Defend, CardInstanceId(99), 0)],
            opening_draw: cards.len().min(10), encounter_visit: 0 }).unwrap();
    entry.cards = cards.iter().enumerate()
        .map(|(index, (id, upgrades))| complete_live_card(*id, CardInstanceId(index as u64 + 1), *upgrades).unwrap()).collect();
    entry.opening_draw = cards.len().min(10);
    let mut items = Some(CombatItemContext::new(relics, potions).with_complete_cards().with_complete_items());
    Combat::enter_with_item_context(entry, &mut items).unwrap()
}

fn take(combat: &mut Combat, instance: u64) -> OwnedCard {
    let handle = card_state::handle(CardInstanceId(instance));
    for pile in [&mut combat.state.piles.hand, &mut combat.state.piles.draw, &mut combat.state.piles.discard, &mut combat.state.piles.exhaust] {
        if let Some(at) = pile.iter().position(|owned| owned.handle() == handle) { return pile.remove(at); }
    }
    panic!("card {instance} is not in a pile")
}

/// Hand, draw (top is last) and discard, by fixture instance.
fn arrange(combat: &mut Combat, hand: &[u64], draw: &[u64], discard: &[u64]) {
    for instance in hand { let card = take(combat, *instance); combat.state.piles.hand.push(card); }
    for instance in draw { let card = take(combat, *instance); combat.state.piles.draw.push(card); }
    for instance in discard { let card = take(combat, *instance); combat.state.piles.discard.push(card); }
    combat.state.public.known_draw.clear();
    combat.state.public.known_draw_bottom.clear();
    piles::refresh_hand_values(&mut combat.state);
}

fn step(combat: &mut Combat, action: CombatAction, context: &str) {
    combat.step_full(&action).unwrap_or_else(|error| panic!("{context}: {error:?}; fault={:?}", combat.state.fault));
    assert!(combat.state.fault.is_none(), "{context}: {:?}", combat.state.fault);
}

fn manual_discards(combat: &Combat) -> usize {
    combat.state.hook_events.iter().filter(|event| event.hook == Hook::CardManuallyDiscarded).count()
}

/// A Melange Scry is open, owns its suspended continuation, and is offered.
fn assert_melange_scry_open(combat: &Combat, resume: Phase) {
    let selection = combat.state.pending_selection.as_ref().expect("Melange Scry is open");
    assert!(matches!(&selection.parent, SelectionParent::Power { id, .. } if id.0 == "melange"));
    assert_eq!(selection.operation, SelectionOperation::Scry);
    assert_eq!(selection.resume.phase, resume);
    assert_eq!(combat.state.phase, Phase::Selection);
    assert_eq!(combat.validate_runtime_for_test(), Ok(()));
    assert!(combat.legal_actions_full().unwrap().contains(&CombatAction::ConfirmSelection));
}

fn confirm_scry_discarding_first(combat: &mut Combat, context: &str) {
    let candidate = combat.state.pending_selection.as_ref().unwrap().candidates[0].as_str().parse().unwrap();
    step(combat, CombatAction::SelectCard { instance: candidate }, context);
    step(combat, CombatAction::ConfirmSelection, context);
}

#[test]
fn fuzzfix_a_melange_scry_inside_a_distilled_chaos_child_suspends_and_resumes_the_child() {
    let mut combat = fixture(Character::Watcher, &[(PommelStrike, 0), (StrikePurple, 0), (DefendPurple, 0), (DefendPurple, 0), (StrikePurple, 0), (DefendPurple, 0), (StrikePurple, 0)],
        vec![RelicId::PureWater, RelicId::Melange], vec![Some(PotionId::DistilledChaos), None, None], 10);
    arrange(&mut combat, &[4], &[3, 2, 1], &[5, 6, 7]);
    // Distilled Chaos takes all three top cards before using any; Pommel Strike's draw then
    // shuffles the discard and Melange opens its Scry inside the potion-owned child.
    step(&mut combat, CombatAction::UsePotion { slot: 0, target: None }, "Distilled Chaos + Melange");
    assert_melange_scry_open(&combat, Phase::PlayerAction);
    let frame = combat.state.pending_selection.as_ref().unwrap().resume.frame.clone().expect("child frame recorded");
    assert!(matches!(frame.1, model::PlayFrameParentOwner::Potion { id: PotionId::DistilledChaos, .. }));
    confirm_scry_discarding_first(&mut combat, "Melange Scry confirm inside Distilled Chaos");
    assert_eq!(combat.state.phase, Phase::Decision);
    assert!(combat.state.play_frames.is_empty() && combat.state.in_play.is_none());
    assert_eq!(combat.validate_runtime_for_test(), Ok(()));
}

#[test]
fn fuzzfix_a_melange_scry_inside_burning_pact_post_selection_draw_resumes_the_card() {
    let mut combat = fixture(Character::Watcher, &[(BurningPact, 0), (StrikePurple, 0), (DefendPurple, 0), (StrikePurple, 0),
        (DefendPurple, 0), (StrikePurple, 0), (DefendPurple, 0), (StrikePurple, 0)],
        vec![RelicId::PureWater, RelicId::Melange], vec![None; 3], 10);
    arrange(&mut combat, &[1, 2, 3], &[], &[4, 5, 6, 7, 8]);
    step(&mut combat, CombatAction::PlayCard { instance: 1, target: None }, "Burning Pact");
    // The exhaust commit resumes the card in PlayerAction; its Draw 2 shuffles and Melange suspends it.
    step(&mut combat, CombatAction::SelectCard { instance: 2 }, "Burning Pact exhaust choice");
    assert_melange_scry_open(&combat, Phase::PlayerAction);
    assert_eq!(combat.state.pending_selection.as_ref().unwrap().resume.in_play, Some(card_state::handle(CardInstanceId(1))));
    confirm_scry_discarding_first(&mut combat, "Melange Scry confirm inside Burning Pact");
    assert_eq!(combat.state.phase, Phase::Decision);
    assert!(combat.state.in_play.is_none());
    assert_eq!(combat.validate_runtime_for_test(), Ok(()));
}

#[test]
fn fuzzfix_a_direct_manual_discard_during_a_retain_commit_runs_its_relic_callback() {
    let mut combat = fixture(Character::Silent, &[(WellLaidPlans, 0), (Mayhem, 0), (AllOutAttack, 0), (DefendGreen, 0), (StrikeGreen, 0),
        (DefendGreen, 0), (StrikeGreen, 0), (DefendGreen, 0), (StrikeGreen, 0), (DefendGreen, 0)],
        vec![RelicId::Tingsha], vec![None; 3], 10);
    arrange(&mut combat, &[1, 2, 4, 5], &[3, 6, 10, 9, 8, 7], &[]);
    step(&mut combat, CombatAction::PlayCard { instance: 1, target: None }, "Well-Laid Plans");
    step(&mut combat, CombatAction::PlayCard { instance: 2, target: None }, "Mayhem");
    step(&mut combat, CombatAction::EndTurn, "end turn into retain");
    assert!(combat.state.pending_selection.as_ref().is_some_and(|selection| selection.operation == SelectionOperation::Retain));
    step(&mut combat, CombatAction::SelectCard { instance: 4 }, "retain choice");
    // After the next turn draws five cards, Mayhem plays the sixth card, All-Out Attack,
    // which discards directly while the retain commit is still resolving.
    step(&mut combat, CombatAction::ConfirmSelection, "retain confirm through Mayhem All-Out Attack");
    assert_eq!(combat.state.turn, 2);
    assert!(combat.state.hook_events.iter().any(|event| event.hook == Hook::CardPlayed
        && event.source == Source::Card(card_state::handle(CardInstanceId(3)))), "Mayhem played All-Out Attack");
    assert_eq!(manual_discards(&combat), 1);
}

#[test]
fn fuzzfix_a_storm_of_steel_discards_under_an_omniscience_commit_run_their_relic_callbacks() {
    for relic in [RelicId::Tingsha, RelicId::ToughBandages] {
        let mut combat = fixture(Character::Silent, &[(Omniscience, 0), (StormOfSteel, 0), (DefendGreen, 0), (DefendGreen, 0), (StrikeGreen, 0)],
            vec![relic], vec![None; 3], 10);
        arrange(&mut combat, &[1, 3, 4], &[5, 2], &[]);
        step(&mut combat, CombatAction::PlayCard { instance: 1, target: None }, "Omniscience");
        // The Omniscience choice commits; its children play Storm of Steel twice while that commit resolves.
        step(&mut combat, CombatAction::SelectCard { instance: 2 }, "Omniscience selects Storm of Steel");
        assert_eq!(manual_discards(&combat), 2, "{relic:?}");
    }
}

#[test]
fn fuzzfix_a_tools_of_the_trade_no_choice_discard_runs_its_relic_callback() {
    let mut combat = fixture(Character::Silent, &[(ToolsOfTheTrade, 0), (StrikeGreen, 0)], vec![RelicId::Tingsha], vec![None; 3], 10);
    arrange(&mut combat, &[1], &[], &[2]);
    step(&mut combat, CombatAction::PlayCard { instance: 1, target: None }, "Tools of the Trade");
    // Turn two draws the only card and Tools draws nothing more, so it discards that card directly at PlayerStart.
    step(&mut combat, CombatAction::EndTurn, "Tools of the Trade turn start");
    assert!(combat.state.pending_selection.is_none());
    assert_eq!(manual_discards(&combat), 1);
}

fn whirlwind_damage(combat: &Combat, hp: i32) -> i32 { hp - combat.state.enemies[0].hp }

#[test]
fn fuzzfix_a_distilled_chaos_burst_x_cards_share_pre_burst_energy_and_spend_nothing() {
    let mut combat = fixture(Character::Ironclad, &[(Whirlwind, 0), (Whirlwind, 0), (Defend, 0), (Defend, 0)],
        vec![], vec![Some(PotionId::DistilledChaos), None, None], 3);
    arrange(&mut combat, &[4], &[3, 2, 1], &[]);
    let hp = combat.state.enemies[0].hp;
    step(&mut combat, CombatAction::UsePotion { slot: 0, target: None }, "Distilled Chaos burst");
    assert_eq!(whirlwind_damage(&combat, hp), 30, "each Whirlwind has X=3 at 5 per hit");
    assert_eq!(combat.state.player.energy, 3, "autoplayed X cards are free to play once");
}

#[test]
fn fuzzfix_a_distilled_chaos_x_adds_chemical_x() {
    let mut combat = fixture(Character::Ironclad, &[(Whirlwind, 0), (Defend, 0), (Defend, 0), (Defend, 0)],
        vec![RelicId::ChemicalX], vec![Some(PotionId::DistilledChaos), None, None], 3);
    arrange(&mut combat, &[4], &[3, 2, 1], &[]);
    let hp = combat.state.enemies[0].hp;
    step(&mut combat, CombatAction::UsePotion { slot: 0, target: None }, "Distilled Chaos + Chemical X");
    assert_eq!(whirlwind_damage(&combat, hp), 25, "X = 3 energy + 2 Chemical X");
    assert_eq!(combat.state.player.energy, 3);
}

#[test]
fn fuzzfix_a_distilled_chaos_skewer_uses_captured_energy() {
    let mut combat = fixture(Character::Silent, &[(Skewer, 0), (DefendGreen, 0), (DefendGreen, 0), (DefendGreen, 0)],
        vec![], vec![Some(PotionId::DistilledChaos), None, None], 3);
    arrange(&mut combat, &[4], &[3, 2, 1], &[]);
    let hp = combat.state.enemies[0].hp;
    step(&mut combat, CombatAction::UsePotion { slot: 0, target: None }, "Distilled Chaos Skewer");
    assert_eq!(whirlwind_damage(&combat, hp), 21, "Skewer X=3 at 7 per hit");
    assert_eq!(combat.state.player.energy, 3);
}

#[test]
fn fuzzfix_a_two_stack_mayhem_burst_x_cards_share_turn_start_energy_and_spend_nothing() {
    let mut combat = fixture(Character::Ironclad, &[(Mayhem, 0), (Mayhem, 0), (Whirlwind, 0), (Whirlwind, 0),
        (Defend, 0), (Defend, 0), (Defend, 0), (Defend, 0), (Defend, 0)], vec![], vec![None; 3], 3);
    arrange(&mut combat, &[1, 2], &[4, 3, 5, 6, 7, 8, 9], &[]);
    combat.state.player.energy = 4;
    step(&mut combat, CombatAction::PlayCard { instance: 1, target: None }, "first Mayhem");
    step(&mut combat, CombatAction::PlayCard { instance: 2, target: None }, "second Mayhem");
    let hp = combat.state.enemies[0].hp;
    step(&mut combat, CombatAction::EndTurn, "two-stack Mayhem turn start");
    assert_eq!(combat.state.turn, 2);
    assert_eq!(whirlwind_damage(&combat, hp), 30, "both sixth/seventh cards use post-recharge X=3");
    assert_eq!(combat.state.player.energy, 3);
}

#[test]
fn fuzzfix_a_melange_scry_after_the_after_draw_relic_phase_does_not_redispatch_it() {
    let cards = (0..12).map(|index| if index % 2 == 0 { (StrikePurple, 0) } else { (DefendPurple, 0) }).collect::<Vec<_>>();
    let mut combat = fixture(Character::Watcher, &cards, vec![RelicId::Pocketwatch, RelicId::Melange], vec![None; 3], 3);
    arrange(&mut combat, &[1, 2, 3], &[4, 5, 6, 7, 8, 9], &[10, 11, 12]);
    // Turn two draws five of six; Pocketwatch's after-draw Draw 3 shuffles and Melange suspends PlayerStart.
    step(&mut combat, CombatAction::EndTurn, "end turn into Pocketwatch draw");
    assert_melange_scry_open(&combat, Phase::PlayerStart);
    // The after-draw relic phase is the TurnStarted checkpoint's own work, raised once.
    step(&mut combat, CombatAction::ConfirmSelection, "Melange Scry confirm at PlayerStart");
    assert_eq!(combat.state.phase, Phase::Decision);
    assert_eq!(combat.state.piles.hand.len(), 8, "five turn draws and one Pocketwatch Draw 3; the after-draw phase ran once");
    assert_eq!(combat.validate_runtime_for_test(), Ok(()));
}
