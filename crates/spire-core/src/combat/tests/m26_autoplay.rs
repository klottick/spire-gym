//! M26 Task M3 Normality blocked-autoplay checks (Review Focus 2, seam F9).
use super::super::*;
use crate::{
    combat::{card_state, model::OwnedCard, rng::Partition, CardInstanceId, CombatAction, Hook, Source, TargetId},
    content::{CardId, Character, EncounterId, EnemyId},
    items::CombatItemContext,
};

use CardId::*;

fn card(id: CardId, instance: u64, upgrades: u32) -> LiveCard {
    complete_live_card(id, CardInstanceId(instance), upgrades)
        .unwrap_or_else(|error| panic!("complete card {id:?}+{upgrades}: {error:?}"))
}

fn fight(encounter: EncounterId, act: u8, cards: &[(CardId, u32)]) -> Combat {
    fight_as(Character::Ironclad, encounter, act, cards)
}

fn fight_as(character: Character, encounter: EncounterId, act: u8, cards: &[(CardId, u32)]) -> Combat {
    let mut rng = crate::run::RunRng::new([41; 32], Partition::Test, 0);
    let mut entry = enemies::setup(encounter, act, 0, &mut rng, enemies::SetupContext {
        player: PlayerState {
            hp: 80, max_hp: 80, block: 0, energy: 10, max_energy: 10,
            draw_per_turn: 5, powers: vec![],
        },
        player_powers: vec![],
        character,
        cards: vec![permanent_live_card(Defend, CardInstanceId(99), 0)],
        opening_draw: cards.len(),
        encounter_visit: 0,
    }).unwrap_or_else(|error| panic!("{encounter:?} setup: {error:?}"));
    entry.cards = cards.iter().enumerate().map(|(index, (id, upgrades))|
        card(*id, index as u64 + 1, *upgrades)).collect();
    entry.opening_draw = cards.len();
    let mut items = Some(CombatItemContext::new(vec![], vec![None; 3]).with_complete_cards());
    Combat::enter_with_item_context(entry, &mut items)
        .unwrap_or_else(|error| panic!("complete-card {encounter:?} entry: {error:?}"))
}

fn take_card(combat: &mut Combat, instance: u64) -> OwnedCard {
    let handle = card_state::handle(CardInstanceId(instance));
    if let Some(at) = combat.state.piles.hand.iter().position(|owned| owned.handle() == handle) {
        return combat.state.piles.hand.remove(at);
    }
    if let Some(at) = combat.state.piles.draw.iter().position(|owned| owned.handle() == handle) {
        return combat.state.piles.draw.remove(at);
    }
    if let Some(at) = combat.state.piles.discard.iter().position(|owned| owned.handle() == handle) {
        return combat.state.piles.discard.remove(at);
    }
    if let Some(at) = combat.state.piles.exhaust.iter().position(|owned| owned.handle() == handle) {
        return combat.state.piles.exhaust.remove(at);
    }
    panic!("fixture card {instance} has no pile owner")
}

fn arrange(combat: &mut Combat, hand: &[u64], draw: &[u64]) {
    for instance in hand {
        let owned = take_card(combat, *instance);
        combat.state.piles.hand.push(owned);
    }
    for instance in draw {
        let owned = take_card(combat, *instance);
        combat.state.piles.draw.push(owned);
    }
    piles::refresh_hand_values(&mut combat.state);
}

fn play(combat: &mut Combat, instance: u64, target: Option<u64>) {
    let action = CombatAction::PlayCard { instance, target: target.map(TargetId) };
    combat.step_full(&action)
        .unwrap_or_else(|error| panic!("{action:?}: {error:?}; fault={:?}", combat.fault_record()));
}

fn card_played_count(combat: &Combat, instance: u64) -> usize {
    let source = Source::Card(card_state::handle(CardInstanceId(instance)));
    combat.state.hook_events.iter()
        .filter(|event| event.hook == Hook::CardPlayed && event.source == source)
        .count()
}

fn all_card_played_count(combat: &Combat) -> usize {
    combat.state.hook_events.iter().filter(|event| event.hook == Hook::CardPlayed).count()
}

fn player_power(combat: &Combat, name: &'static str) -> i32 {
    combat.state.player_full_powers.iter().find(|power| power.id.0 == name)
        .map_or(0, |power| power.state.amount)
}

fn owned_instance(combat: &Combat, instance: u64) -> Option<&LiveCard> {
    combat.state.piles.all(combat.state.in_play.as_ref()).filter_map(OwnedCard::full)
        .find(|card| card.instance.0 == instance)
}

fn assert_no_fault(combat: &Combat, context: &str) {
    assert!(combat.fault_record().is_none(), "{context}: {:?}", combat.fault_record());
    assert!(combat.state.fault.is_none(), "{context}: {:?}", combat.state.fault);
}

#[test]
fn m26_burst_replay_preserves_collect_captured_x() {
    let mut combat = fight_as(Character::Silent, EncounterId::Cultist, 1, &[(Burst, 0), (Collect, 1)]);
    play(&mut combat, 1, None);
    play(&mut combat, 2, None);
    assert_no_fault(&combat, "Burst+Collect repeat");
    assert_eq!(player_power(&combat, "collect"), 20, "both Collect+ uses use the captured nine energy plus one");
    engine::validate_runtime(&combat.state).unwrap();
}

#[test]
fn m26_mayhem_collect_uses_turn_start_energy_without_spending_it() {
    let mut combat = fight_as(Character::Watcher, EncounterId::Cultist, 1, &[(Mayhem, 0), (Collect, 0)]);
    arrange(&mut combat, &[1], &[2]);
    combat.state.player.energy = 3;
    combat.state.player.max_energy = 3;
    play(&mut combat, 1, None);
    assert_eq!(combat.state.player.energy, 1, "Mayhem's ordinary cost is paid before turn start");
    combat.step_full(&CombatAction::EndTurn)
        .unwrap_or_else(|error| panic!("Mayhem turn-start Collect: {error:?}; fault={:?}", combat.fault_record()));
    assert_no_fault(&combat, "Mayhem X-cost Collect");
    assert_eq!(player_power(&combat, "collect"), 3, "Collect uses energy captured when Mayhem queues its child");
    assert_eq!(combat.state.player.energy, 3, "an autoplayed X card is free to play once and spends no energy");
}

#[test]
fn m26_havoc_whirlwind_uses_queued_energy_without_spending_it() {
    let mut combat = fight(EncounterId::Cultist, 1, &[(Havoc, 0), (Whirlwind, 0)]);
    arrange(&mut combat, &[1], &[2]);
    combat.state.player.energy = 4;
    let hp = combat.state.enemies[0].hp;
    play(&mut combat, 1, None);
    assert_no_fault(&combat, "Havoc X-cost Whirlwind");
    assert_eq!(combat.state.enemies[0].hp, hp - 15, "Havoc child uses the three energy left after Havoc's payment");
    assert_eq!(combat.state.player.energy, 3, "an autoplayed Whirlwind is free to play once and drains nothing");
}

#[test]
fn m26_zero_energy_autoplay_skips_cost_check_and_uses_positive_x_upgrade() {
    let mut fixed = fight(EncounterId::Cultist, 1, &[(Havoc, 0), (Strike, 0)]);
    arrange(&mut fixed, &[1], &[2]);
    fixed.state.player.energy = 0;
    fixed.state.piles.hand.iter_mut().filter_map(OwnedCard::full_mut)
        .find(|card| card.instance == CardInstanceId(1)).unwrap().local.free_play = true;
    let hp = fixed.state.enemies[0].hp;
    play(&mut fixed, 1, None);
    assert_eq!(fixed.state.enemies[0].hp, hp - 6, "autoplay child ignores its unaffordable fixed cost");
    assert_eq!(fixed.state.player.energy, 0);

    let mut x = fight_as(Character::Watcher, EncounterId::Cultist, 1, &[(Havoc, 0), (Collect, 1)]);
    arrange(&mut x, &[1], &[2]);
    x.state.player.energy = 0;
    x.state.piles.hand.iter_mut().filter_map(OwnedCard::full_mut)
        .find(|card| card.instance == CardInstanceId(1)).unwrap().local.free_play = true;
    x.step_full(&CombatAction::PlayCard { instance: 1, target: None })
        .unwrap_or_else(|error| panic!("zero-energy Havoc+Collect+: {error:?}; fault={:?}", x.fault_record()));
    assert_no_fault(&x, "zero-energy Collect+ autoplay");
    assert_eq!(player_power(&x, "collect"), 1, "Collect+ upgrade makes its zero-energy X effect positive");
    assert_eq!(x.state.player.energy, 0, "zero-energy X has no energy to spend");
}

#[test]
fn m26_mayhem_tools_of_the_trade_turn_start_selection_is_accepted() {
    let mut combat = fight_as(Character::Silent, EncounterId::Cultist, 1, &[
        (Mayhem, 0), (ToolsOfTheTrade, 0), (Strike, 0), (Defend, 0), (DodgeAndRoll, 0),
    ]);
    arrange(&mut combat, &[1], &[5, 4, 3, 2]);
    play(&mut combat, 1, None);
    combat.step_full(&CombatAction::EndTurn)
        .unwrap_or_else(|error| panic!("Mayhem + Tools start turn: {error:?}; fault={:?}", combat.fault_record()));
    assert_no_fault(&combat, "Mayhem + Tools start turn");
    let selection = combat.state.pending_selection.as_ref()
        .expect("Tools of the Trade opens its turn-start discard selection");
    assert!(matches!(selection.parent,
        crate::combat::SelectionParent::Power { id: FullPowerId("tools_of_the_trade"), .. }));
    let candidate = selection.candidates[0].clone();
    let instance = combat.state.piles.hand.iter().find(|card| card.handle() == candidate)
        .and_then(OwnedCard::full).expect("Tools selection candidate remains in hand").instance.0;
    combat.step_full(&CombatAction::SelectCard { instance })
        .unwrap_or_else(|error| panic!("select Tools discard: {error:?}; fault={:?}", combat.fault_record()));
    assert!(combat.state.pending_selection.is_none(), "Tools selection is accepted and resolved");
    assert_no_fault(&combat, "Mayhem + Tools discard selection");
}

#[test]
fn m26_normality_blocks_autoplay() {
    // Disclosed decision F9 (seam map §7): the child would be play four, so it is suppressed.
    // Main ruling v1.1: prepare_child_work -> None -> FinishSuppressed -> finish_owned_card
    // applies the frame disposition: Havoc's imposed Exhaust, Mayhem's Discard, and purge
    // for Double Tap/Burst copies. No child emits CardPlayed or spends energy.
    let mut havoc = fight(EncounterId::JawWorm, 1, &[
        (Normality, 0), (Defend, 0), (Defend, 0), (Havoc, 0), (Strike, 0), (Defend, 0),
    ]);
    arrange(&mut havoc, &[1, 2, 3, 4, 6], &[5]);
    play(&mut havoc, 2, None);
    play(&mut havoc, 3, None);
    assert_eq!(havoc.state.silent.played_this_turn.len(), 2);
    let energy_before_havoc = havoc.state.player.energy;
    play(&mut havoc, 4, None);
    assert_eq!(havoc.state.silent.played_this_turn.len(), 3, "Havoc itself is play three");
    assert_eq!(havoc.state.player.energy, energy_before_havoc - 1, "Havoc's own cost only; blocked child costs no energy");
    assert_eq!(all_card_played_count(&havoc), 3, "Havoc child emitted no CardPlayed");
    assert_eq!(card_played_count(&havoc, 4), 1, "Havoc parent is counted once");
    assert_eq!(card_played_count(&havoc, 5), 0, "blocked top Strike is not played");
    // The accepted M10 status disposition check uses Dazed made unplayable by Confused:
    // m10_havoc_recycles_without_draw_callback_and_suppresses_unplayable_status.
    assert!(havoc.state.piles.exhaust.iter().any(|owned| owned.handle() == card_state::handle(CardInstanceId(5))),
        "Havoc imposes Exhaust on its Normality-blocked child, matching the accepted M10 status case");
    assert!(!havoc.legal_actions_full().unwrap().iter()
        .any(|action| matches!(action, CombatAction::PlayCard { instance: 6, .. })),
        "fourth manual play is absent from legal actions");
    assert_no_fault(&havoc, "Normality-blocked Havoc");

    // RetainCards keeps Normality in hand through EndTurn; the accepted natural-turn
    // fixture is m26_normality_counts_autoplay_and_late_draw (m26_cards.rs:272-284).
    // Mayhem runs at TurnStartedPreDraw (colorless/powers.rs:108-120). Stacking four
    // triggers makes the fourth known top Defend the first child blocked by Normality.
    let mut mayhem = fight(EncounterId::JawWorm, 1, &[
        (Normality, 0), (Mayhem, 0), (Defend, 0), (Defend, 0), (Defend, 0), (Defend, 0), (Defend, 0),
    ]);
    arrange(&mut mayhem, &[1, 2], &[7, 6, 5, 4, 3]);
    // Leave the blocked fourth child in Discard after Mayhem's pre-draw hook; the
    // fifth known card remains available for a manual fourth-play legality check.
    mayhem.state.player.draw_per_turn = 1;
    mayhem.state.player_full_powers.push(FullPowerEntry { id: FullPowerId("retain_cards"),
        state: PowerState { amount: 1, skip_first: false, monster_source_grace: false } });
    play(&mut mayhem, 2, None);
    let mayhem_power = mayhem.state.player_full_powers.iter_mut()
        .find(|power| power.id.0 == "mayhem").expect("played Mayhem installs its power");
    mayhem_power.state.amount = 4;
    assert_eq!(mayhem.state.silent.played_this_turn.len(), 1, "Mayhem was the only play on the prior turn");
    let card_played_before_next_turn = all_card_played_count(&mayhem);
    mayhem.step_full(&CombatAction::EndTurn)
        .unwrap_or_else(|error| panic!("end turn with retained Normality: {error:?}"));
    assert!(mayhem.state.pending_selection.as_ref().unwrap().candidates
        .contains(&card_state::handle(CardInstanceId(1))), "retain selection includes Normality");
    mayhem.step_full(&CombatAction::SelectCard { instance: 1 }).unwrap();
    mayhem.step_full(&CombatAction::ConfirmSelection)
        .unwrap_or_else(|error| panic!("confirm retained Normality and resolve Mayhem: {error:?}"));
    assert!(mayhem.state.piles.hand.iter().any(|owned|
        owned.handle() == card_state::handle(CardInstanceId(1))), "Normality remains in hand");
    assert_eq!(mayhem.state.silent.played_this_turn.len(), 3, "only the first three Mayhem children count");
    assert_eq!(all_card_played_count(&mayhem), card_played_before_next_turn + 3,
        "the first three children emit CardPlayed; the blocked fourth does not");
    for instance in [3, 4, 5] {
        assert_eq!(card_played_count(&mayhem, instance), 1, "Mayhem child {instance} plays normally");
    }
    assert_eq!(card_played_count(&mayhem, 6), 0, "fourth Mayhem child is not played");
    assert_eq!(mayhem.state.player.energy, 10, "Mayhem children cost no energy after start-turn recharge");
    assert!(mayhem.state.piles.discard.iter().any(|owned|
        owned.handle() == card_state::handle(CardInstanceId(6))), "Mayhem sends its blocked child to Discard");
    assert!(!mayhem.legal_actions_full().unwrap().iter()
        .any(|action| matches!(action, CombatAction::PlayCard { instance: 7, .. })),
        "fourth manual play is absent from legal actions");
    assert_no_fault(&mayhem, "Normality-blocked natural Mayhem");

    let mut double_tap = fight(EncounterId::JawWorm, 1, &[
        (Normality, 0), (DoubleTap, 0), (Defend, 0), (Strike, 0), (Defend, 0),
    ]);
    arrange(&mut double_tap, &[1, 2, 3, 4, 5], &[]);
    play(&mut double_tap, 2, None);
    play(&mut double_tap, 3, None);
    let issued_before = double_tap.state.card_ids_issued.clone();
    let energy_before = double_tap.state.player.energy;
    play(&mut double_tap, 4, Some(1));
    assert_eq!(double_tap.state.silent.played_this_turn.len(), 3, "Double Tap original is the third play");
    assert_eq!(double_tap.state.player.energy, energy_before - 1, "repeat spends no energy");
    assert_eq!(all_card_played_count(&double_tap), 3, "Double Tap repeat emits no CardPlayed");
    assert_eq!(card_played_count(&double_tap, 4), 1, "original Attack counts normally");
    assert_eq!(player_power(&double_tap, "double_tap"), 0, "blocked Double Tap charge is consumed");
    let spawned = double_tap.state.card_ids_issued.difference(&issued_before).cloned().collect::<Vec<_>>();
    assert_eq!(spawned.len(), 1, "Double Tap constructs one repeat copy");
    let purged = double_tap.state.inactive_cards.iter().filter_map(OwnedCard::full)
        .find(|card| card.instance.0 == spawned[0].as_str().parse::<u64>().unwrap())
        .expect("Double Tap repeat is retired as the purged owned copy");
    assert!(purged.local.purge, "Double Tap repeat carries its purge disposition");
    assert!(!double_tap.state.piles.all(double_tap.state.in_play.as_ref()).any(|owned| owned.handle() == spawned[0]),
        "blocked Double Tap copy is purged");
    assert!(!double_tap.legal_actions_full().unwrap().iter()
        .any(|action| matches!(action, CombatAction::PlayCard { instance: 5, .. })),
        "fourth manual play is absent from legal actions");
    assert_no_fault(&double_tap, "Normality-blocked Double Tap repeat");

    let mut burst = fight(EncounterId::JawWorm, 1, &[
        (Normality, 0), (Burst, 0), (Defend, 0), (Defend, 0), (Defend, 0),
    ]);
    arrange(&mut burst, &[1, 2, 3, 4, 5], &[]);
    play(&mut burst, 3, None);
    play(&mut burst, 2, None);
    let issued_before = burst.state.card_ids_issued.clone();
    let energy_before = burst.state.player.energy;
    play(&mut burst, 4, None);
    assert_eq!(burst.state.silent.played_this_turn.len(), 3, "Burst original Skill is the third play and its repeat is blocked");
    assert_eq!(burst.state.player.energy, energy_before - 1, "repeat spends no energy");
    assert_eq!(all_card_played_count(&burst), 3, "Burst repeat emits no CardPlayed");
    assert_eq!(card_played_count(&burst, 4), 1, "original Skill counts normally");
    assert_eq!(player_power(&burst, "burst"), 0, "blocked Burst charge is consumed");
    let spawned = burst.state.card_ids_issued.difference(&issued_before).cloned().collect::<Vec<_>>();
    assert_eq!(spawned.len(), 1, "Burst constructs one repeat copy");
    let purged = burst.state.inactive_cards.iter().filter_map(OwnedCard::full)
        .find(|card| card.instance.0 == spawned[0].as_str().parse::<u64>().unwrap())
        .expect("Burst repeat is retired as the purged owned copy");
    assert!(purged.local.purge, "Burst repeat carries its purge disposition");
    assert!(!burst.state.piles.all(burst.state.in_play.as_ref()).any(|owned| owned.handle() == spawned[0]),
        "blocked Burst copy is purged");
    assert!(!burst.legal_actions_full().unwrap().iter()
        .any(|action| matches!(action, CombatAction::PlayCard { instance: 5, .. })),
        "fourth manual play is absent from legal actions");
    assert_no_fault(&burst, "Normality-blocked Burst repeat");
}

#[test]
fn m26_mayhem_suppresses_data_driven_unplayable_children() {
    for child in [Reflex, Tactician, DeusExMachina] {
        let mut combat = fight(EncounterId::Cultist, 1, &[(Mayhem, 0), (child, 0)]);
        combat.state.character = characters::CharacterState::for_character(Character::Silent);
        arrange(&mut combat, &[1], &[2]);
        play(&mut combat, 1, None);
        combat.step_full(&CombatAction::EndTurn)
            .unwrap_or_else(|error| panic!("Mayhem resolving {child:?}: {error:?}; fault={:?}", combat.fault_record()));
        assert_no_fault(&combat, "Mayhem unplayable child");
        assert_eq!(card_played_count(&combat, 2), 0, "{child:?} no-op use does not run card-play hooks");
        assert!(piles::owned_full_cards(&combat.state)
            .any(|card| card.instance == CardInstanceId(2)), "{child:?} completes under its ordinary disposition");
        assert!(engine::validate_runtime(&combat.state).is_ok());
    }
}

#[test]
fn m26_ritual_dagger_combat_exclusion_frontier() {
    // Combat-level H2′ probe. All three exclusions are also in TRAINING_ENCOUNTERS_V2,
    // but complete_combat is absent from this snapshot, so this fixture permits a direct
    // H2 runtime check without stubbing the behavior under test.
    for (encounter, act) in [
        (EncounterId::Reptomancer, 3),
        (EncounterId::AwakenedOne, 3),
        (EncounterId::ThreeDarklings, 3),
    ] {
        let mut combat = fight(encounter, act, &[(RitualDagger, 0)]);
        let target = match encounter {
            EncounterId::Reptomancer => combat.state.enemies.iter().position(|enemy|
                enemy.full_powers.iter().any(|power| power.id.0 == "minion")).unwrap(),
            EncounterId::AwakenedOne => combat.state.enemies.iter().position(|enemy|
                enemy.enemy_id == crate::combat::ActorEnemyId::Full(EnemyId::AwakenedOne)).unwrap(),
            EncounterId::ThreeDarklings => 0,
            _ => unreachable!(),
        };
        if encounter == EncounterId::ThreeDarklings {
            assert!(combat.state.enemies.iter().skip(1).all(|enemy| enemy.hp > 0));
        }
        combat.state.enemies[target].hp = 15;
        combat.state.enemies[target].block = 0;
        play(&mut combat, 1, Some(target as u64 + 1));
        assert_eq!(owned_instance(&combat, 1).unwrap().misc, 15,
            "excluded Fatal on {encounter:?} does not grow Ritual Dagger");
        assert!(combat.state.defect.permanent_mutations.is_empty(), "no journal for excluded Fatal on {encounter:?}");
        assert_no_fault(&combat, "Ritual Dagger exclusion");
    }
}

#[test]
fn m26_ritual_dagger_growth_tracks_instance_family() {
    // Design §8.6: Fatal grows the played physical instance and its same-instance family;
    // the stat-equivalent copy used by Dual Wield belongs to another family.
    let mut combat = fight(EncounterId::JawWorm, 1, &[(RitualDagger, 0)]);
    let source = combat.state.piles.hand.iter().filter_map(OwnedCard::full)
        .find(|card| card.instance.0 == 1).expect("Ritual Dagger in opening hand").clone();
    let family = source.same_instance_family_copy(CardInstanceId(90));
    let stat_equivalent = source.stat_equivalent_copy(CardInstanceId(91));
    combat.state.card_ids_issued.insert(card_state::handle(CardInstanceId(90)));
    combat.state.card_ids_issued.insert(card_state::handle(CardInstanceId(91)));
    combat.state.piles.draw.push(OwnedCard::Full(family));
    combat.state.piles.discard.push(OwnedCard::Full(stat_equivalent));
    piles::refresh_hand_values(&mut combat.state);
    combat.state.enemies[0].hp = 15;
    combat.step_full(&CombatAction::PlayCard { instance: 1, target: Some(TargetId(1)) }).unwrap();

    let owned = |instance| combat.state.piles.all(combat.state.in_play.as_ref())
        .filter_map(OwnedCard::full).find(|card| card.instance.0 == instance).unwrap();
    assert_eq!(owned(1).misc, 18, "played Ritual Dagger gains +3");
    assert_eq!(owned(90).misc, 18, "same-instance-family copy follows +3 growth");
    assert_eq!(owned(91).misc, 15, "Dual Wield stat-equivalent copy keeps its base misc");
    assert_eq!(combat.permanent_card_mutations().len(), 1, "one permanent growth journal entry");
    assert_no_fault(&combat, "Ritual Dagger family growth");
}
