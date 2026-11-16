//! spire-d8g E1–E3, E5: Standard Run faults found by the 2026-11-16 stress campaign (lane S r1b), replayed from their
//! saved action prefixes. Supplied start as in M36 (public seed world plus `supply_hp_and_keys`), here HP 600 and all
//! keys (E5 also 5,000 gold); supplied, never a strategy claim. The last action of each prefix used to fault; each case
//! asserts the original.
use super::m38e_compact::standard_config;
use crate::run_session::{create_run, RunProfile};
use crate::{standard_run_observation_spec, standard_vocab_manifest, RunActionIndex, RunCapacities, RunObservation, RunSession};
use spire_core::{content::{Character, RelicId}, run::{diagnostics::{supply_gold, supply_hp_and_keys}, Run}};

const SUPPLIED_HP: u32 = 600;

/// Replays the prefix; returns the Run and observation before the last action, the observation after it, and the session.
fn replay(character: Character, ascension: u8, seed: u64, prefix: &str) -> (Run, RunObservation, RunObservation, RunSession) {
    replay_with_gold(character, ascension, seed, prefix, None)
}
fn replay_with_gold(character: Character, ascension: u8, seed: u64, prefix: &str, gold: Option<u32>)
    -> (Run, RunObservation, RunObservation, RunSession) {
    let config = standard_config(character, ascension, RunCapacities::standard_default(), true);
    let mut start = create_run(RunProfile::Standard, &config.session_config(), seed, 0).unwrap();
    supply_hp_and_keys(&mut start, SUPPLIED_HP, true);
    if let Some(gold) = gold { supply_gold(&mut start, gold); }
    let mut session = RunSession::from_standard_supplied(config, start, seed).unwrap();
    let actions: Vec<i64> = serde_json::from_str(prefix).unwrap();
    let mut boundary = session.decision().unwrap();
    let mut before = None;
    for (step, action) in actions.iter().enumerate() {
        if step + 1 == actions.len() { before = Some((session.run_for_test().unwrap().clone(), boundary.observation.clone())); }
        session.step(boundary.token, RunActionIndex(*action))
            .unwrap_or_else(|error| panic!("{character:?} A{ascension} s={seed} step {step}: {error:?}"));
        boundary = session.decision().unwrap();
    }
    let (run, observation) = before.unwrap();
    (run, observation, boundary.observation, session)
}

fn view(run: &Run) -> spire_core::run::PublicDecision { run.decision().unwrap() }
fn field(group: &str, name: &str) -> usize {
    let d = standard_run_observation_spec(&RunCapacities::standard_default()).unwrap().descriptor();
    let t = d["tensors"].as_array().unwrap().iter().find(|t| t["key"] == group).unwrap();
    t["fields"].as_array().unwrap().iter().position(|f| f == name).unwrap()
}
/// `(field value, row)` of every occupied row of `group`.
fn rows(o: &RunObservation, group: &str, name: &str) -> Vec<i64> {
    let g = o.group(group).unwrap();
    let width = *g.shape.last().unwrap();
    let occupied = g.occupied.as_ref().unwrap();
    (0..g.shape[0]).filter(|row| occupied[*row] == 1).map(|row| g.values[row * width + field(group, name)]).collect()
}
fn vocab(family: &str, name: &str) -> i64 {
    standard_vocab_manifest()["families"][family].as_array().unwrap().iter().position(|v| v == name).unwrap() as i64
}
fn count(o: &RunObservation, card: &str) -> usize { rows(o, "permanent_cards", "card_kind").into_iter().filter(|k| *k == vocab("card", card)).count() }

/// E1: Omamori negating a transformed curse. The original removes the selected card, transforms it, and obtains the
/// output through `ShowCardAndObtainEffect`, whose constructor spends an Omamori charge on a curse; the new curse is
/// not added (lane D Q4). The transform used to fault (`a negated card obtain needs an *_or_negated owner`). Omamori's
/// charge is not public in the Standard observation; `spire-core` `run::tests::d8g_findings` asserts it.
fn e1(character: Character, ascension: u8, seed: u64, prefix: &str, input: &str, output: &str) {
    let (run, before, after, _) = replay(character, ascension, seed, prefix);
    assert!(view(&run).player.relics.contains(&RelicId::Omamori));
    assert_eq!(rows(&after, "permanent_cards", "card_kind").len(), rows(&before, "permanent_cards", "card_kind").len() - 1,
        "the input is removed and nothing is added");
    assert_eq!(count(&after, input), count(&before, input) - 1, "{input} removed");
    assert_eq!(count(&after, output), count(&before, output), "{output} not added");
}

#[test]
fn d8g_e1_living_wall_change_omamori_negates_the_transformed_curse() {
    e1(Character::Ironclad, 12, 1_568_801_021, include_str!("fixtures/d8g/e1-ironclad-a12.json"), "regret", "doubt");
    e1(Character::Silent, 1, 1_568_802_870, include_str!("fixtures/d8g/e1-silent-a1.json"), "doubt", "parasite");
}

#[test]
fn d8g_e1_drug_dealer_transform_omamori_negates_the_transformed_curse() {
    e1(Character::Ironclad, 3, 1_568_807_653, include_str!("fixtures/d8g/e1-ironclad-a3.json"), "doubt", "shame");
}

/// E2: Corpse Explosion kills the Stasis Bronze Orb while that orb's poison tick is queued. The original's death guards
/// (`isDying`; poison's `currentHealth > 0`) run its death callbacks once, so the held card returns once (lane D Q5).
/// The orb used to receive a second Death hook and fault (`Stasis return death/custody`).
#[test]
fn d8g_e2_corpse_explosion_kills_the_stasis_orb_once() {
    for (ascension, seed, prefix, held) in [
        (1, 1_568_802_070, include_str!("fixtures/d8g/e2-silent-a1.json"), "secret_technique"),
        (0, 1_568_806_869, include_str!("fixtures/d8g/e2-silent-a0.json"), "wraith_form"),
    ] {
        let (_, _, after, _) = replay(Character::Silent, ascension, seed, prefix);
        let in_hand = rows(&after, "combat_cards", "card_kind").into_iter().zip(rows(&after, "combat_cards", "zone"))
            .filter(|(kind, zone)| *kind == vocab("card", held) && *zone == vocab("zone", "hand")).count();
        assert_eq!(in_hand, 1, "A{ascension}: {held} returned to hand once");
    }
}

/// E3: Tomb of Lord Red Mask's gold with Ectoplasm. The original's `gainGold` returns before changing gold, so the
/// 222 gold is blocked entirely and nothing else changes (lane D Q6). The event's frontier check used to expect the
/// gold and fault (`accepted Act3 event mutation frontier changed`).
#[test]
fn d8g_e3_tomb_of_red_mask_gold_is_blocked_by_ectoplasm() {
    let (before, _, _, session) = replay(Character::Watcher, 5, 1_568_802_194, include_str!("fixtures/d8g/e3-watcher-a5.json"));
    let after = session.run_for_test().unwrap();
    let (before, after) = (view(&before), view(after));
    assert!(before.player.relics.contains(&RelicId::Ectoplasm) && before.player.relics.contains(&RelicId::RedMask));
    assert_eq!(after.player.gold, before.player.gold, "Ectoplasm blocks the 222 gold");
    assert_eq!((after.player.relics, after.player.hp, after.player.max_hp), (before.player.relics, before.player.hp, before.player.max_hp));
    assert_eq!(after.objects, before.objects);
}

/// E5: The Boot raised a 2-damage hit (Clothesline capped by Invincible 2) to 5 after the cap, so the Heart's
/// Invincible went to −3 and the next selection boundary faulted the validator. The original applies The Boot before
/// Invincible's cap (lane D Q10). Ironclad A20, 5,000 gold supplied; the last action plays Dual Wield+ at the Heart.
#[test]
fn d8g_e5_heart_invincible_stays_within_its_allowance_with_the_boot() {
    let (before, _, after, _) = replay_with_gold(Character::Ironclad, 20, 1_568_801_736,
        include_str!("fixtures/d8g/e5-ironclad-a20.json"), Some(5_000));
    assert!(view(&before).player.relics.contains(&RelicId::TheBoot));
    let heart = rows(&after, "enemies", "enemy_kind").iter().position(|kind| *kind == vocab("enemy", "corrupt_heart")).expect("the Heart fight");
    assert!(rows(&after, "enemies", "hp")[heart] > 0);
}
