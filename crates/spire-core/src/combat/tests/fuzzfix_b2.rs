//! Regressions for fuzz chunk B2 (spire-oto.10, .12, .15 and .21).
use super::super::*;
use super::fuzzfix_b::{arrange, fixture, fixture_in, in_pile, step, used_target};
use crate::{combat::{model::OwnedCard, CombatAction}, content::{CardId, Character, EncounterId, PotionId, RelicId}};
use CardId::*;

fn hand_ids(combat: &Combat) -> Vec<CardId> {
    combat.state.piles.hand.iter().filter_map(OwnedCard::full).map(|card| card.id).collect()
}

fn in_hand(combat: &Combat, instance: u64) -> bool { in_pile(&combat.state.piles.hand, instance) }

/// spire-oto.15 (F16): RetainCardPower.atEndOfTurn retains nothing when Equilibrium already
/// applies (source-check-turnstart-hooks-v1, atEndOfTurn table), in either power order, so no
/// Well-Laid Plans selection opens. The recorded fuzz failure was `InvalidAction` on the
/// published `ConfirmSelection` of that selection.
#[test]
fn fuzzfix_b2_well_laid_plans_opens_no_selection_under_equilibrium() {
    for (first, second) in [(1, 2), (2, 1)] {
        let mut combat = fixture(Character::Silent, &[(WellLaidPlans, 0), (Equilibrium, 0), (StrikeGreen, 0), (DefendGreen, 0)],
            vec![], vec![None; 3], 5);
        arrange(&mut combat, &[1, 2, 3, 4], &[], &[]);
        step(&mut combat, CombatAction::PlayCard { instance: first, target: None }, "first power");
        step(&mut combat, CombatAction::PlayCard { instance: second, target: None }, "second power");
        step(&mut combat, CombatAction::EndTurn, "end turn under Equilibrium");
        assert_eq!(combat.state.phase, Phase::Decision, "order {first},{second}: no retain selection opens");
        assert_eq!(combat.state.turn, 2);
        assert!(in_hand(&combat, 3) && in_hand(&combat, 4), "order {first},{second}: Equilibrium retained the hand");
    }
}

/// spire-oto.15: the same source check skips Well-Laid Plans when Runic Pyramid applies.
#[test]
fn fuzzfix_b2_well_laid_plans_opens_no_selection_under_runic_pyramid() {
    let mut combat = fixture(Character::Silent, &[(WellLaidPlans, 0), (StrikeGreen, 0), (DefendGreen, 0)],
        vec![RelicId::RunicPyramid], vec![None; 3], 3);
    arrange(&mut combat, &[1, 2, 3], &[], &[]);
    step(&mut combat, CombatAction::PlayCard { instance: 1, target: None }, "Well-Laid Plans");
    step(&mut combat, CombatAction::EndTurn, "end turn under Runic Pyramid");
    assert_eq!(combat.state.phase, Phase::Decision, "no retain selection opens");
    assert!(in_hand(&combat, 2) && in_hand(&combat, 3), "Runic Pyramid retained the hand");
}

/// spire-oto.15: every ConfirmSelection the menu publishes is accepted. A retain choice made
/// while another owner already retains cards this turn adds to that set.
#[test]
fn fuzzfix_b2_published_retain_confirm_is_accepted_and_merges_the_existing_set() {
    let mut combat = fixture(Character::Silent, &[(WellLaidPlans, 0), (StrikeGreen, 0), (DefendGreen, 0), (Neutralize, 0),
        (Survivor, 0), (Survivor, 0), (Survivor, 0), (Survivor, 0), (Survivor, 0)], vec![], vec![None; 3], 3);
    // Five draw-pile cards: the turn-2 draw does not reshuffle the discard.
    arrange(&mut combat, &[1, 2, 3, 4], &[5, 6, 7, 8, 9], &[]);
    step(&mut combat, CombatAction::PlayCard { instance: 1, target: None }, "Well-Laid Plans");
    step(&mut combat, CombatAction::EndTurn, "end turn opens the retain selection");
    assert_eq!(combat.state.phase, Phase::Selection);
    let turn = combat.state.turn;
    let other = card_state::handle(crate::combat::CardInstanceId(4));
    combat.state.temporary_retain = Some(model::TemporaryRetain { turn, cards: [other].into_iter().collect() });
    step(&mut combat, CombatAction::SelectCard { instance: 2 }, "retain Strike");
    let menu = combat.legal_actions_full().unwrap();
    assert!(menu.contains(&CombatAction::ConfirmSelection), "the menu publishes ConfirmSelection");
    step(&mut combat, CombatAction::ConfirmSelection, "the published confirm is accepted");
    assert!(in_hand(&combat, 2) && in_hand(&combat, 4), "both owners' cards stay in hand");
    assert!(in_pile(&combat.state.piles.discard, 3), "the unretained Defend is discarded");
}

/// spire-oto.10 (F11): a Meditate autoplayed by Distilled Chaos ends the turn at the potion
/// action's boundary, with and without its return selection. The recorded fuzz failure was
/// `Invariant("Meditate lost its parent finish/decision checkpoint")`.
#[test]
fn fuzzfix_b2_distilled_chaos_meditate_ends_the_turn() {
    for discard in [&[4u64][..], &[4, 5]] {
        let mut combat = fixture(Character::Watcher, &[(Meditate, 0), (DefendPurple, 0), (DefendPurple, 0), (StrikePurple, 0), (StrikePurple, 0)],
            vec![], vec![Some(PotionId::DistilledChaos), None, None], 3);
        arrange(&mut combat, &[], &[3, 2, 1], discard);
        combat.step_full(&CombatAction::UsePotion { slot: 0, target: None })
            .unwrap_or_else(|error| panic!("discard {discard:?}: {error:?}; fault={:?}", combat.state.fault));
        if discard.len() > 1 {
            assert_eq!(combat.state.phase, Phase::Selection, "Meditate chooses one of two discards");
            step(&mut combat, CombatAction::SelectCard { instance: 4 }, "Meditate returns Strike");
            step(&mut combat, CombatAction::ConfirmSelection, "Meditate confirms its return");
        }
        assert!(combat.state.fault.is_none(), "discard {discard:?}: {:?}", combat.state.fault);
        assert_eq!((combat.state.turn, combat.state.phase), (2, Phase::Decision), "discard {discard:?}: the turn ended once");
        assert!(in_hand(&combat, 4), "discard {discard:?}: the returned Strike was retained");
        assert_eq!(used_target(&combat, 2), None, "discard {discard:?}: the end cancels the rest of the burst");
    }
}

/// spire-oto.12 (F13): a Distilled Chaos burst survives a potion obtained into its own emptied
/// slot by an earlier burst card (Alchemize). The queued work belongs to the consumed use, not
/// to the slot's current contents. The recorded fuzz failure was
/// `Invariant("Distilled Chaos top action retains its captured target proposal")`.
#[test]
fn fuzzfix_b2_distilled_chaos_burst_survives_its_slot_refill() {
    let mut combat = fixture(Character::Silent, &[(Alchemize, 0), (DefendGreen, 0), (DefendGreen, 0), (StrikeGreen, 0)],
        vec![], vec![Some(PotionId::DistilledChaos), None, None], 3);
    arrange(&mut combat, &[4], &[3, 2, 1], &[]);
    step(&mut combat, CombatAction::UsePotion { slot: 0, target: None }, "Distilled Chaos with Alchemize on top");
    let potions = combat.state.item_context.as_ref().unwrap().potions().to_vec();
    assert!(potions[0].is_some_and(|id| id != PotionId::DistilledChaos), "Alchemize refilled slot 0: {potions:?}");
    assert_eq!(combat.state.player.block, 10, "both Defends after Alchemize were used");
    assert_eq!(hand_ids(&combat), vec![StrikeGreen]);
}

/// Turn-2 start for a Defect holding the given powers, played in this order, from one
/// fixed card-random state. Returns the generated cards in hand order (they precede the draw).
fn turn_two_generated(powers: &[CardId], rng: &Option<rng::StreamState>) -> Vec<CardId> {
    let mut cards = powers.iter().map(|id| (*id, 0)).collect::<Vec<_>>();
    cards.extend([(Defend, 0); 6]);
    let mut combat = fixture(Character::Defect, &cards, vec![], vec![None; 3], 10);
    let count = powers.len() as u64;
    arrange(&mut combat, &(1..=count).collect::<Vec<_>>(), &(count + 1..=count + 6).collect::<Vec<_>>(), &[]);
    for instance in 1..=count { step(&mut combat, CombatAction::PlayCard { instance, target: None }, "install power"); }
    combat.state.card_rng = rng.clone();
    step(&mut combat, CombatAction::EndTurn, "end turn 1");
    assert_eq!(combat.state.turn, 2);
    hand_ids(&combat).into_iter().take(powers.len()).collect()
}

/// spire-oto.21: atStartOfTurn power callbacks run in the player's power-list order, whatever
/// engine module owns them. Creative AI and Magnetism each take one card-random draw when
/// their callback runs, so the first-applied power takes the first draw, in either order.
#[test]
fn fuzzfix_b2_start_of_turn_powers_draw_in_power_list_order() {
    let rng = fixture(Character::Defect, &[(Defend, 0)], vec![], vec![None; 3], 3).state.card_rng.clone();
    let alone_power = turn_two_generated(&[CreativeAI], &rng)[0];
    let alone_colorless = turn_two_generated(&[Magnetism], &rng)[0];
    let ai_first = turn_two_generated(&[CreativeAI, Magnetism], &rng);
    assert_eq!(ai_first[0], alone_power, "Creative AI applied first takes the first draw: {ai_first:?}");
    assert_eq!(crate::content::Catalog::standard().card(ai_first[1]).unwrap().color, crate::content::CardColor::Colorless);
    let magnetism_first = turn_two_generated(&[Magnetism, CreativeAI], &rng);
    assert_eq!(magnetism_first[0], alone_colorless, "Magnetism applied first takes the first draw: {magnetism_first:?}");
    assert_eq!(crate::content::Catalog::standard().card(magnetism_first[1]).unwrap().color, crate::content::CardColor::Blue);
}

fn power_order(powers: &[FullPowerEntry], ids: &[&str]) -> Vec<&'static str> {
    powers.iter().map(|power| power.id.0).filter(|id| ids.contains(id)).collect()
}

/// spire-oto.21 (source-check-power-priority-v1): ApplyPowerAction inserts a new power by a
/// stable ascending-priority sort. Weak (99) follows Poison (5) on a monster, and Noxious
/// Fumes (5) precedes Tools of the Trade (25) on the player, in either application order.
#[test]
fn fuzzfix_b2_applied_powers_take_their_priority_position() {
    for (first, second) in [(1, 2), (2, 1)] {
        let mut combat = fixture(Character::Silent, &[(Neutralize, 0), (DeadlyPoison, 0), (ToolsOfTheTrade, 0), (NoxiousFumes, 0)],
            vec![], vec![None; 3], 10);
        arrange(&mut combat, &[1, 2, 3, 4], &[], &[]);
        let enemy = Some(TargetId(1));
        for instance in [first, second] { step(&mut combat, CombatAction::PlayCard { instance, target: enemy }, "debuff"); }
        assert_eq!(power_order(&combat.state.enemies[0].full_powers, &["poison", "weak"]), vec!["poison", "weak"], "order {first},{second}");
        for instance in [first + 2, second + 2] { step(&mut combat, CombatAction::PlayCard { instance, target: None }, "power"); }
        assert_eq!(power_order(&combat.state.player_full_powers, &["noxious_fumes", "tools_of_the_trade"]),
            vec!["noxious_fumes", "tools_of_the_trade"], "order {first},{second}");
    }
}

/// spire-oto.21: equal priorities keep their application order (the sort is stable).
#[test]
fn fuzzfix_b2_equal_priority_powers_keep_application_order() {
    for (first, second, expected) in [(1, 2, ["noxious_fumes", "infinite_blades"]), (2, 1, ["infinite_blades", "noxious_fumes"])] {
        let mut combat = fixture(Character::Silent, &[(NoxiousFumes, 0), (InfiniteBlades, 0)], vec![], vec![None; 3], 10);
        arrange(&mut combat, &[1, 2], &[], &[]);
        for instance in [first, second] { step(&mut combat, CombatAction::PlayCard { instance, target: None }, "power"); }
        assert_eq!(power_order(&combat.state.player_full_powers, &expected), expected.to_vec());
    }
}

/// spire-oto.21: AbstractCreature.addPower appends without sorting. Red Skull's direct
/// Strength follows a higher-priority power applied earlier, and Philosopher's Stone's battle-start
/// Strength follows a Byrd's Flight (50).
#[test]
fn fuzzfix_b2_direct_add_paths_append_unsorted() {
    let mut combat = fixture(Character::Silent, &[(ToolsOfTheTrade, 0), (DefendGreen, 0)], vec![], vec![None; 3], 10);
    arrange(&mut combat, &[1, 2], &[], &[]);
    step(&mut combat, CombatAction::PlayCard { instance: 1, target: None }, "Tools of the Trade");
    powers::add_direct_strength(&mut combat.state, 3).unwrap();
    assert_eq!(power_order(&combat.state.player_full_powers, &["strength", "tools_of_the_trade"]), vec!["tools_of_the_trade", "strength"]);
    let byrds = fixture_in(EncounterId::ThreeByrds, 2, Character::Silent, &[(DefendGreen, 0)],
        vec![RelicId::PhilosophersStone], vec![None; 3], 3);
    for enemy in &byrds.state.enemies {
        assert_eq!(power_order(&enemy.full_powers, &["flight", "strength"]), vec!["flight", "strength"]);
    }
}

/// spire-oto.21 review I1: ApplyPowerAction adds, then stable-sorts the whole list, so a sorted
/// insert also moves an entry an earlier unsorted addPower appended. A Byrd with Philosopher's
/// Stone holds [flight, strength]; Deadly Poison then gives [strength, poison, flight]. On the
/// player, Weak then a direct (Red Skull) Strength, then Noxious Fumes, gives
/// [strength, noxious_fumes, weak].
#[test]
fn fuzzfix_b2_sorted_insert_resorts_an_appended_list() {
    let ids = ["flight", "strength", "poison"];
    let mut byrds = fixture_in(EncounterId::ThreeByrds, 2, Character::Silent, &[(DeadlyPoison, 0), (DefendGreen, 0)],
        vec![RelicId::PhilosophersStone], vec![None; 3], 3);
    arrange(&mut byrds, &[1, 2], &[], &[]);
    assert_eq!(power_order(&byrds.state.enemies[0].full_powers, &ids), vec!["flight", "strength"]);
    step(&mut byrds, CombatAction::PlayCard { instance: 1, target: Some(TargetId(1)) }, "Deadly Poison");
    assert_eq!(power_order(&byrds.state.enemies[0].full_powers, &ids), vec!["strength", "poison", "flight"]);
    let public = byrds.public_full().unwrap().enemies[0].powers.iter().map(|power| power.id.0)
        .filter(|id| ids.contains(id)).collect::<Vec<_>>();
    assert_eq!(public, vec!["strength", "poison", "flight"], "the public monster list shows the source order");
    assert_eq!(power_order(&byrds.state.enemies[1].full_powers, &ids), vec!["flight", "strength"], "untouched Byrd keeps its append");

    let ids = ["strength", "noxious_fumes", "weak"];
    let mut combat = fixture(Character::Silent, &[(NoxiousFumes, 0), (DefendGreen, 0)], vec![], vec![None; 3], 10);
    arrange(&mut combat, &[1, 2], &[], &[]);
    let enemy = Actor::Enemy(combat.state.enemies[0].instance_id.clone());
    combat.enqueue_native_effects(vec![Effect { source: Source::Actor(enemy), target: Some(Actor::Player),
        kind: EffectKind::ApplyFullPower { id: FullPowerId("weak"), amount: 2 } }], None).unwrap();
    powers::add_direct_strength(&mut combat.state, 3).unwrap();
    assert_eq!(power_order(&combat.state.player_full_powers, &ids), vec!["weak", "strength"]);
    step(&mut combat, CombatAction::PlayCard { instance: 1, target: None }, "Noxious Fumes");
    assert_eq!(power_order(&combat.state.player_full_powers, &ids), vec!["strength", "noxious_fumes", "weak"]);
}

/// Turn-2 hand after playing Infinite Blades (1) and Nightmare (2, copying Defend 3) in `order`.
fn night_terror_turn_two_hand(order: [u64; 2]) -> Vec<CardId> {
    let mut combat = fixture(Character::Silent, &[(InfiniteBlades, 0), (Nightmare, 0), (DefendGreen, 0),
        (StrikeGreen, 0), (StrikeGreen, 0), (StrikeGreen, 0), (StrikeGreen, 0), (StrikeGreen, 0)], vec![], vec![None; 3], 10);
    arrange(&mut combat, &[1, 2, 3], &[4, 5, 6, 7, 8], &[]);
    for instance in order {
        step(&mut combat, CombatAction::PlayCard { instance, target: None }, "power");
        if instance == 2 {
            step(&mut combat, CombatAction::SelectCard { instance: 3 }, "Nightmare copies Defend");
            if combat.state.phase == Phase::Selection { step(&mut combat, CombatAction::ConfirmSelection, "confirm Nightmare"); }
        }
    }
    step(&mut combat, CombatAction::EndTurn, "end turn 1");
    assert_eq!((combat.state.turn, combat.state.phase), (2, Phase::Decision));
    hand_ids(&combat)
}

/// spire-oto.21 review I2: Night Terror (NightmarePower) is a priority-5 list entry whose
/// atStartOfTurn runs in list order with the other start powers, in either application order.
#[test]
fn fuzzfix_b2_night_terror_runs_in_power_list_order() {
    let blades_first = night_terror_turn_two_hand([1, 2]);
    assert_eq!(blades_first[..4], [Shiv, DefendGreen, DefendGreen, DefendGreen], "{blades_first:?}");
    let nightmare_first = night_terror_turn_two_hand([2, 1]);
    assert_eq!(nightmare_first[..4], [DefendGreen, DefendGreen, DefendGreen, Shiv], "{nightmare_first:?}");
    assert_eq!(blades_first.len(), 9, "four generated cards, then the five-card draw");
}

/// spire-oto.12 review M1: Distilled Chaos custody is the recorded use. Mid-burst work
/// re-sourced to a slot where no Distilled Chaos was used fails runtime validation.
#[test]
fn fuzzfix_b2_distilled_chaos_custody_is_the_recorded_use() {
    use crate::content::PotionId::DistilledChaos;
    let mut combat = fixture(Character::Watcher, &[(Meditate, 0), (DefendPurple, 0), (DefendPurple, 0), (StrikePurple, 0), (StrikePurple, 0)],
        vec![], vec![Some(DistilledChaos), None, None], 3);
    arrange(&mut combat, &[], &[3, 2, 1], &[4, 5]);
    step(&mut combat, CombatAction::UsePotion { slot: 0, target: None }, "Distilled Chaos");
    assert_eq!(combat.state.phase, Phase::Selection, "Meditate suspends the burst");
    assert_eq!(combat.state.item_context.as_ref().unwrap().potions(), [None, None, None], "the use emptied slot 0");
    combat.validate_runtime_for_test().expect("the real burst validates");
    let mut forged = combat.clone();
    for effect in forged.state.pending.iter_mut() {
        if let Source::Potion { slot, id: DistilledChaos } = &mut effect.source { *slot = 1; }
    }
    assert_eq!(forged.validate_runtime_for_test(),
        Err(InputError::Invariant("Distilled Chaos materialized child retains typed potion custody".into())));
}

/// spire-oto.15 review M2: the menu publishes ConfirmSelection whenever the selected count
/// allows it. A confirm the commit preflight rejects returns an error and changes no state.
#[test]
fn fuzzfix_b2_rejected_published_confirm_changes_no_state() {
    let mut combat = fixture(Character::Silent, &[(WellLaidPlans, 0), (StrikeGreen, 0), (DefendGreen, 0),
        (Survivor, 0), (Survivor, 0), (Survivor, 0), (Survivor, 0), (Survivor, 0)], vec![], vec![None; 3], 3);
    arrange(&mut combat, &[1, 2, 3], &[4, 5, 6, 7, 8], &[]);
    step(&mut combat, CombatAction::PlayCard { instance: 1, target: None }, "Well-Laid Plans");
    step(&mut combat, CombatAction::EndTurn, "end turn opens the retain selection");
    step(&mut combat, CombatAction::SelectCard { instance: 2 }, "retain Strike");
    let turn = combat.state.turn;
    // A selection-independent preflight defect: the retain parent names another turn.
    combat.state.pending_selection.as_mut().unwrap().parent = selection::SelectionParent::EndTurn { turn: turn + 1 };
    assert!(combat.legal_actions_full().unwrap().contains(&CombatAction::ConfirmSelection), "Confirm stays published");
    let before = format!("{:?}", combat.state);
    assert!(combat.step_full(&CombatAction::ConfirmSelection).is_err(), "the commit preflight rejects it");
    assert_eq!(format!("{:?}", combat.state), before, "a rejected confirm changes no state");
}

/// spire-oto.21 review-v2 I3: the Night Terror entry stays out of every public channel. A
/// Nightmare installed and delivered through `step_full_observed` emits no `night_terror`
/// event; the only power event is the Cultist's Ritual, as before the entry existed.
#[test]
fn fuzzfix_b2_night_terror_stays_out_of_the_observed_stream() {
    let mut combat = fixture(Character::Silent, &[(Nightmare, 0), (DefendGreen, 0), (StrikeGreen, 0), (StrikeGreen, 0),
        (StrikeGreen, 0), (StrikeGreen, 0), (StrikeGreen, 0)], vec![], vec![None; 3], 10);
    arrange(&mut combat, &[1, 2], &[3, 4, 5, 6, 7], &[]);
    let mut stream = vec![];
    let mut actions = vec![CombatAction::PlayCard { instance: 1, target: None }, CombatAction::SelectCard { instance: 2 }];
    for _ in 0..3 {
        for action in actions.drain(..) {
            let observed = combat.step_full_observed(&action);
            assert!(observed.result.is_ok(), "{action:?}: {:?}", observed.result);
            stream.extend(observed.effects.iter().map(|effect| format!("{effect:?}")));
        }
        actions.push(if combat.state.phase == Phase::Selection { CombatAction::ConfirmSelection } else { CombatAction::EndTurn });
        if combat.state.turn == 2 { break; }
    }
    assert_eq!(combat.state.turn, 2);
    assert_eq!(hand_ids(&combat)[..3], [DefendGreen, DefendGreen, DefendGreen], "the copies were delivered");
    println!("observed stream: {stream:#?}");
    assert!(stream.iter().all(|effect| !effect.contains("night_terror")), "no Night Terror event");
    let powers = stream.iter().filter(|effect| effect.contains("kind: Power")).collect::<Vec<_>>();
    assert_eq!(powers.len(), 1, "{powers:#?}");
    assert!(powers[0].contains("\"ritual\"") && powers[0].contains("after: Some(3)"), "{powers:#?}");
}

/// spire-oto.21 review-v2 M6: with two Nightmares around Infinite Blades, the k-th Night Terror
/// entry delivers the k-th template in list order: A's copies, the Shiv, then B's copies.
#[test]
fn fuzzfix_b2_two_nightmares_deliver_in_list_order() {
    let mut combat = fixture(Character::Silent, &[(Nightmare, 0), (InfiniteBlades, 0), (Nightmare, 0), (DefendGreen, 0), (Neutralize, 0),
        (StrikeGreen, 0), (StrikeGreen, 0), (StrikeGreen, 0), (StrikeGreen, 0), (StrikeGreen, 0)], vec![], vec![None; 3], 10);
    arrange(&mut combat, &[1, 2, 3, 4, 5], &[6, 7, 8, 9, 10], &[]);
    for (nightmare, copied) in [(1, 4), (3, 5)] {
        if nightmare == 3 { step(&mut combat, CombatAction::PlayCard { instance: 2, target: None }, "Infinite Blades"); }
        step(&mut combat, CombatAction::PlayCard { instance: nightmare, target: None }, "Nightmare");
        step(&mut combat, CombatAction::SelectCard { instance: copied }, "Nightmare copies");
        if combat.state.phase == Phase::Selection { step(&mut combat, CombatAction::ConfirmSelection, "confirm Nightmare"); }
    }
    let ids = ["night_terror", "infinite_blades"];
    assert_eq!(power_order(&combat.state.player_full_powers, &ids), vec!["night_terror", "infinite_blades", "night_terror"]);
    step(&mut combat, CombatAction::EndTurn, "end turn 1");
    assert_eq!(hand_ids(&combat)[..7], [DefendGreen, DefendGreen, DefendGreen, Shiv, Neutralize, Neutralize, Neutralize]);
    assert_eq!(power_order(&combat.state.player_full_powers, &ids), vec!["infinite_blades"]);
    assert!(combat.state.nightmare_templates.is_empty());
}
