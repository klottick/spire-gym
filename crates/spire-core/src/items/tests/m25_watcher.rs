//! M25 Watcher exclusive-relic admission and accepted-item preservation.
use super::*;
use crate::combat::{CombatAction, FullPowerEntry, FullPowerId, PowerState, PublicZone,
    ResolvedEffectKind, Stance, TargetId};
use crate::content::{CardId, Character, PotionId, RelicId};

fn m25_entry(deck: &[(CardId, u32)], opening_draw: usize) -> CombatEntry {
    let mut entry = full_entry(Character::Watcher, opening_draw, 0);
    entry.cards = deck.iter().enumerate().map(|(index, (id, upgrades))|
        crate::combat::m25_watcher_live_card(*id, crate::combat::CardInstanceId(index as u64 + 1), *upgrades).unwrap()
    ).collect();
    entry.opening_draw = opening_draw.min(deck.len());
    entry
}

fn m25_combat(entry: CombatEntry, relics: Vec<RelicId>, potions: Vec<Option<PotionId>>) -> Combat {
    enter_with_items(entry, CombatItemContext::new(relics, potions).with_m25_profile())
}

fn hand_count(combat: &Combat, id: CardId) -> usize {
    combat.public_full().unwrap().hand.iter().filter(|card| card.id == id).count()
}

fn player_power(combat: &Combat, id: &str) -> Option<i32> {
    combat.public_full().unwrap().player.powers.iter().find(|power| power.id.0 == id).map(|power| power.amount)
}

fn full_power(id: &'static str, amount: i32) -> FullPowerEntry {
    FullPowerEntry { id: FullPowerId(id), state: PowerState { amount, skip_first: false, monster_source_grace: false } }
}

fn hand_instance(combat: &Combat, id: CardId) -> u64 {
    combat.public_full().unwrap().hand.iter().find(|card| card.id == id).unwrap().instance.0
}

fn melange_combat(relics: Vec<RelicId>) -> Combat {
    // Ten cards guarantee Weave is in the discard before the third turn's real reshuffle.
    let mut deck = vec![(CardId::Weave, 0)];
    deck.extend(vec![(CardId::DefendPurple, 0); 9]);
    let mut entry = m25_entry(&deck, 5);
    entry.setup.player.draw_per_turn = 5;
    entry.setup.player_powers.push(full_power("nirvana", 3));
    m25_combat(entry, relics, vec![None; 3])
}

fn select_first_scry_card(combat: &mut Combat) {
    let instance = combat.public_full().unwrap().selection.unwrap().candidates[0]
        .instance.0;
    combat.step_full(&CombatAction::SelectCard { instance }).unwrap();
    combat.step_full(&CombatAction::ConfirmSelection).unwrap();
}

#[test]
fn m25_items_new_relics_through_real_combat() {
    // Relic facts: Damaru 1 Mantra/turn and its ordinary 10-Mantra threshold
    // (data/relics/relics.json:2353-2359); Duality's Attack trigger and after-resolution
    // ordering (2755-2762); Cloak Clasp's one Block gain after Orichalcum and before
    // Ethereal/discard (1971-1978); Melange's Scry 3 after a real reshuffle, with Golden
    // Eye raising it to 5 and insertion excluded (5175-5182).
    // Damaru 1 Mantra at each player turn start (data/relics/relics.json:2353-2359).
    let mut mantra_entry = m25_entry(&[(CardId::DefendPurple, 0); 10], 5);
    mantra_entry.setup.player_powers.push(full_power("mantra", 8));
    let mut damaru = m25_combat(mantra_entry.clone(), vec![RelicId::Damaru], vec![None; 3]);
    assert_eq!(player_power(&damaru, "mantra"), Some(9), "Damaru adds the first turn's Mantra at combat start");
    assert_eq!(damaru.public_full().unwrap().character.stance, Stance::Neutral,
        "nine Mantra have not yet crossed the threshold");
    damaru.step_full(&CombatAction::EndTurn).unwrap();
    assert_eq!(damaru.public_full().unwrap().character.stance, Stance::Divinity,
        "the second turn's tenth Mantra enters Divinity through the shared power owner");
    assert_eq!(player_power(&damaru, "mantra").unwrap_or(0), 0, "the shared threshold consumes ten Mantra");
    let mut no_damaru = m25_combat(mantra_entry, vec![], vec![None; 3]);
    assert_eq!((no_damaru.public_full().unwrap().character.stance, player_power(&no_damaru, "mantra")),
        (Stance::Neutral, Some(8)), "the first turn receives no Mantra without Damaru");
    no_damaru.step_full(&CombatAction::EndTurn).unwrap();
    assert_eq!((no_damaru.public_full().unwrap().character.stance, player_power(&no_damaru, "mantra")),
        (Stance::Neutral, Some(8)), "the next turn also receives no Mantra without Damaru");

    // Disclosed project decision (contract Binding 9), F6: the relic record says an
    // Attack resolves its own Block before Duality grants Dexterity, so bind the
    // gain to the shared after-use checkpoint, after the Attack payload settles.
    // This is once per actual Attack (data/relics/relics.json:2755-2762); Skills do not trigger it.
    let duality_deck = [(CardId::Prostrate, 0), (CardId::JustLucky, 0), (CardId::StrikePurple, 0),
        (CardId::DefendPurple, 0), (CardId::DefendPurple, 0)];
    let mut duality_entry = m25_entry(&duality_deck, 5);
    duality_entry.setup.player.energy = 9;
    duality_entry.setup.player.max_energy = 9;
    let mut duality = m25_combat(duality_entry.clone(), vec![RelicId::Duality], vec![None; 3]);
    let skill = hand_instance(&duality, CardId::Prostrate);
    duality.step_full(&CombatAction::PlayCard { instance: skill, target: None }).unwrap();
    assert_eq!(player_power(&duality, "dexterity"), None, "a Skill does not trigger Duality");
    let just_lucky = hand_instance(&duality, CardId::JustLucky);
    let just_lucky_observed = duality.step_full_observed(&CombatAction::PlayCard { instance: just_lucky, target: Some(TargetId(1)) });
    just_lucky_observed.result.unwrap();
    assert_eq!((player_power(&duality, "dexterity"), player_power(&duality, "lose_dexterity")),
        (Some(1), Some(1)), "one Attack grants one temporary Dexterity");
    let last_payload = just_lucky_observed.effects.iter().rposition(|effect|
        matches!(effect.kind, ResolvedEffectKind::Damage { .. } | ResolvedEffectKind::Block { .. })).unwrap();
    let first_duality_power = just_lucky_observed.effects.iter().position(|effect|
        matches!(&effect.kind, ResolvedEffectKind::Power { id, after: Some(1), .. } if id.0 == "dexterity")).unwrap();
    assert!(last_payload < first_duality_power, "Duality resolves after Just Lucky's own damage and Block");
    let strike = hand_instance(&duality, CardId::StrikePurple);
    duality.step_full(&CombatAction::PlayCard { instance: strike, target: Some(TargetId(1)) }).unwrap();
    assert_eq!((player_power(&duality, "dexterity"), player_power(&duality, "lose_dexterity")),
        (Some(2), Some(2)), "each further Attack adds one temporary Dexterity");
    duality.step_full(&CombatAction::EndTurn).unwrap();
    assert_eq!((player_power(&duality, "dexterity"), player_power(&duality, "lose_dexterity")),
        (None, None), "the shared end-of-turn loss owner removes the temporary Dexterity");
    let mut no_duality = m25_combat(duality_entry.clone(), vec![], vec![None; 3]);
    let strike = hand_instance(&no_duality, CardId::StrikePurple);
    no_duality.step_full(&CombatAction::PlayCard { instance: strike, target: Some(TargetId(1)) }).unwrap();
    assert_eq!(player_power(&no_duality, "dexterity"), None, "an Attack grants no Dexterity without Duality");
    let mut artifact_entry = duality_entry.clone();
    artifact_entry.setup.player_powers.push(full_power("artifact", 1));
    let mut artifact_duality = m25_combat(artifact_entry, vec![RelicId::Duality], vec![None; 3]);
    let just_lucky = hand_instance(&artifact_duality, CardId::JustLucky);
    artifact_duality.step_full(&CombatAction::PlayCard { instance: just_lucky, target: Some(TargetId(1)) }).unwrap();
    assert_eq!((player_power(&artifact_duality, "dexterity"), player_power(&artifact_duality, "lose_dexterity"),
        player_power(&artifact_duality, "artifact").unwrap_or(0)), (Some(1), None, 0),
        "Artifact blocks the source-defined Dexterity Down debuff, keeping Duality's Dexterity");

    // Disclosed project decision (contract Binding 9), F7: use one hand-count Block
    // event at the source-named checkpoint after Orichalcum and before Ethereal
    // cards leave or the remaining hand is discarded (data/relics/relics.json:1971-1978).
    let clasp_deck = [(CardId::DevaForm, 0), (CardId::DefendPurple, 0), (CardId::DefendPurple, 0),
        (CardId::DefendPurple, 0), (CardId::DefendPurple, 0)];
    // Orichalcum is not part of the accepted M25 Watcher inventory, so pin the
    // shared callback order directly while the end-turn/cleanup placement below
    // is exercised through a real M25 combat.
    let mut orichalcum_clasp = CombatItemContext::new(vec![RelicId::Orichalcum, RelicId::CloakClasp], vec![]).with_m25_profile();
    // spire-oto.22 (source-check-endturn-v1): both are onPlayerEndTurn relics at the
    // callEndOfTurnActions checkpoint; Orichalcum is addToTop and Cloak Clasp addToBot.
    let ordered = crate::items::relic_hook_effects(&mut orichalcum_clasp,
        crate::combat::Hook::PlayerPreEndTurnCards, 40, 50, 0);
    assert_eq!(ordered.top.iter().map(|effect| effect.kind.clone()).collect::<Vec<_>>(),
        vec![crate::combat::EffectKind::GainBlock(6)],
        "Orichalcum's one event precedes Cloak Clasp's hand-size event");
    let mut clasp = m25_combat(m25_entry(&clasp_deck, 5), vec![RelicId::CloakClasp], vec![None; 3]);
    let ethereal = hand_instance(&clasp, CardId::DevaForm);
    let end = clasp.step_full_observed(&CombatAction::EndTurn);
    end.result.unwrap();
    let block_gains = end.effects.iter().enumerate().filter_map(|(index, effect)| match effect.kind {
        ResolvedEffectKind::Block { before, after } if after > before => Some((index, before, after)), _ => None,
    }).collect::<Vec<_>>();
    assert_eq!(block_gains.iter().map(|(_, before, after)| (*before, *after)).collect::<Vec<_>>(),
        vec![(0, 5)], "Cloak Clasp emits one five-Block event for the five-card hand");
    let ethereal_move = end.effects.iter().position(|effect| matches!(&effect.kind,
        ResolvedEffectKind::CardMove { card, before: PublicZone::Hand, after: PublicZone::Exhaust }
            if card.as_str().parse::<u64>().ok() == Some(ethereal))).unwrap();
    assert!(block_gains[0].0 < ethereal_move, "Clasp resolves before Ethereal leaves the hand");
    let mut no_clasp = m25_combat(m25_entry(&clasp_deck, 5), vec![], vec![None; 3]);
    let no_clasp_end = no_clasp.step_full_observed(&CombatAction::EndTurn);
    no_clasp_end.result.unwrap();
    assert_eq!(no_clasp.public_full().unwrap().player.block, 0, "hand size grants no Block without Cloak Clasp");
    assert!(!no_clasp_end.effects.iter().any(|effect| matches!(effect.kind, ResolvedEffectKind::Block { .. })),
        "no Clasp Block event is emitted when the relic is absent");

    // Disclosed project decision (contract Binding 9), F5: queue Scry after the
    // draw action that caused a real shuffle finishes; insertion is not a shuffle.
    // Golden Eye raises Scry 3 to 5, and the shared Scry owner returns Weave and
    // grants Nirvana Block (data/relics/relics.json:5175-5182).
    let mut melange = melange_combat(vec![RelicId::Melange]);
    melange.step_full(&CombatAction::EndTurn).unwrap();
    melange.step_full(&CombatAction::EndTurn).unwrap();
    let scry3 = melange.public_full().unwrap().selection.expect("Melange owns Scry after a real shuffle");
    assert_eq!(scry3.candidates.len(), 3, "Melange Scry amount is three");
    select_first_scry_card(&mut melange);
    assert_eq!(melange.public_full().unwrap().player.block, 3, "Nirvana responds to Melange Scry");
    assert_eq!(hand_count(&melange, CardId::Weave), 1, "the shared Scry callback returns discarded Weave");

    let mut golden_melange = melange_combat(vec![RelicId::Melange, RelicId::GoldenEye]);
    golden_melange.step_full(&CombatAction::EndTurn).unwrap();
    golden_melange.step_full(&CombatAction::EndTurn).unwrap();
    assert_eq!(golden_melange.public_full().unwrap().selection.unwrap().candidates.len(), 5,
        "Golden Eye raises Melange Scry from three to five");
    let mut foresight_deck = vec![(CardId::Foresight, 0)];
    foresight_deck.extend(vec![(CardId::DefendPurple, 0); 9]);
    let mut foresight_entry = m25_entry(&foresight_deck, 10);
    foresight_entry.setup.player.draw_per_turn = 5;
    let mut foresight_melange = m25_combat(foresight_entry, vec![RelicId::Melange], vec![None; 3]);
    let foresight = hand_instance(&foresight_melange, CardId::Foresight);
    foresight_melange.step_full(&CombatAction::PlayCard { instance: foresight, target: None }).unwrap();
    foresight_melange.step_full(&CombatAction::EndTurn).unwrap();
    assert_eq!(foresight_melange.public_full().unwrap().selection.unwrap().parent,
        crate::combat::SelectionParent::Power { turn: 2, id: FullPowerId("foresight") },
        "Foresight keeps its own parent when Melange is also equipped");
    select_first_scry_card(&mut foresight_melange);
    let relic_scry = foresight_melange.public_full().unwrap().selection.expect("Melange follows the turn-start draw");
    assert_eq!(relic_scry.parent, crate::combat::SelectionParent::Power { turn: 2, id: FullPowerId("melange") });
    assert_eq!(relic_scry.candidates.len(), 3);
    let mut no_melange = melange_combat(vec![]);
    no_melange.step_full(&CombatAction::EndTurn).unwrap();
    no_melange.step_full(&CombatAction::EndTurn).unwrap();
    assert!(no_melange.public_full().unwrap().selection.is_none(), "a reshuffle alone does not Scry without Melange");
    assert_eq!(no_melange.public_full().unwrap().player.block, 0, "no Scry means no Nirvana Block");
}

#[test]
fn m25_items_accepted_relics_and_potions_preserved() {
    // The accepted startup and stance values are pinned by M22/M23/M24 Watcher tests:
    // HolyWater's three Miracles (m23_watcher_act2.rs:398-400), Calm entry and
    // VioletLotus Calm-exit energy (m24_watcher_act3.rs:253-259), and the M21 potion
    // actions (m21_watcher_creation.rs:19-45). These are real M25-context combats.
    let pure = m25_combat(m25_entry(&[(CardId::DefendPurple, 0); 10], 5), vec![RelicId::PureWater], vec![None; 3]);
    assert_eq!(hand_count(&pure, CardId::Miracle), 1, "Pure Water creates one actual Miracle");

    let holy = m25_combat(m25_entry(&[(CardId::DefendPurple, 0); 10], 5), vec![RelicId::HolyWater], vec![None; 3]);
    assert_eq!(hand_count(&holy, CardId::Miracle), 3, "Holy Water creates three actual Miracles");

    let teardrop = m25_combat(m25_entry(&[(CardId::DefendPurple, 0); 10], 5), vec![RelicId::TeardropLocket], vec![None; 3]);
    assert_eq!(teardrop.public_full().unwrap().character.stance, Stance::Calm, "Teardrop starts combat in Calm");

    let mut violet = m25_combat(
        m25_entry(&[(CardId::Tranquility, 0), (CardId::Eruption, 1)], 2),
        vec![RelicId::VioletLotus], vec![None; 3]);
    violet.step_full(&CombatAction::PlayCard { instance: 1, target: None }).unwrap();
    violet.step_full(&CombatAction::PlayCard { instance: 2, target: Some(TargetId(1)) }).unwrap();
    assert_eq!((violet.public_full().unwrap().character.stance, violet.public_full().unwrap().player.energy),
        (Stance::Wrath, 4), "leaving Calm for Wrath grants the accepted 2 + 1 energy");

    let mut golden_deck = vec![(CardId::ThirdEye, 0); 10];
    golden_deck.extend(vec![(CardId::DefendPurple, 0); 5]);
    let mut golden = m25_combat(m25_entry(&golden_deck, 10), vec![RelicId::GoldenEye], vec![None; 3]);
    let third_eye = golden.public_full().unwrap().hand.iter().find(|card| card.id == CardId::ThirdEye).unwrap().instance.0;
    golden.step_full(&CombatAction::PlayCard { instance: third_eye, target: None }).unwrap();
    assert_eq!(golden.public_full().unwrap().selection.unwrap().candidates.len(), 5,
        "Golden Eye adds two cards to Third Eye's base Scry 3");

    let mut potions = m25_combat(m25_entry(&[(CardId::DefendPurple, 0); 10], 5), vec![], vec![
        Some(PotionId::BottledMiracle), Some(PotionId::StancePotion), Some(PotionId::Ambrosia)]);
    potions.step_full(&CombatAction::UsePotion { slot: 0, target: None }).unwrap();
    assert_eq!(hand_count(&potions, CardId::Miracle), 2, "Bottled Miracle uses its accepted potency");
    potions.step_full(&CombatAction::UsePotion { slot: 1, target: None }).unwrap();
    potions.step_full(&CombatAction::ChooseStance { stance: Stance::Calm }).unwrap();
    assert_eq!(potions.public_full().unwrap().character.stance, Stance::Calm, "Stance Potion keeps its offered Calm choice");
    potions.step_full(&CombatAction::UsePotion { slot: 2, target: None }).unwrap();
    assert_eq!(potions.public_full().unwrap().character.stance, Stance::Divinity, "Ambrosia enters Divinity");
    assert_eq!(potions.item_context().unwrap().potions(), &[None, None, None], "each accepted potion is consumed once");
}
