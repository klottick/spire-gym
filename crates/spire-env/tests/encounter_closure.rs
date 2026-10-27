use serde_json::{json, Value};
use spire_core::{combat::enemies::A0_ENCOUNTER_SETUPS, combat::rng::Partition};
use spire_env::*;

const ACTION_WIDTH: usize = 8;
const ENEMY_WIDTH: usize = 12;

fn entry(
    encounter: &str,
    gold: Option<Value>,
    hp: u32,
    max_hp: u32,
    deck: &[(&str, u32)],
    relics: &[&str],
    potions: [Option<&str>; 3],
) -> Value {
    let mut value = json!({
        "id": "case",
        "encounter": encounter,
        "hp": hp,
        "max_hp": max_hp,
        "deck": deck.iter().map(|(card, upgrades)| {
            json!({"card": card, "upgrades": upgrades})
        }).collect::<Vec<_>>(),
        "relics": relics,
        "potions": potions.map(|potion| potion.map(Value::from)),
    });
    if let Some(gold) = gold {
        value.as_object_mut().unwrap().insert("gold".to_owned(), gold);
    }
    value
}

fn catalog_json(profile_id: Option<&str>, entries: Vec<Value>) -> Vec<u8> {
    let mut value = json!({"entries": entries});
    if let Some(profile_id) = profile_id {
        value["profile_id"] = json!(profile_id);
    }
    serde_json::to_vec(&value).unwrap()
}

fn v2_catalog(entries: Vec<Value>) -> CombatTrainingCatalog {
    CombatTrainingCatalog::from_json(&catalog_json(
        Some(COMBAT_TRAINING_PROFILE_V2_ID),
        entries,
    ))
    .unwrap()
}

fn support_ids(value: &Value) -> Vec<String> {
    value["encounters"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| row["id"].as_str().unwrap().to_owned())
        .collect()
}

fn player(boundary: &CombatPolicyBoundary) -> &[i64] {
    &boundary.observation.group("player").unwrap().values
}

fn enemy_row(boundary: &CombatPolicyBoundary, name: &str) -> Option<usize> {
    let vocab = combat_policy_observation_spec_v2().unwrap().descriptor();
    let enemy_id = vocab["vocabularies"]["enemy"]
        .as_array()
        .unwrap()
        .iter()
        .position(|value| value == name)? as i64;
    let group = boundary.observation.group("enemies").unwrap();
    group
        .values
        .chunks(ENEMY_WIDTH)
        .enumerate()
        .find_map(|(row_index, row)| {
            let occupied = group
                .occupied
                .as_ref()
                .is_some_and(|rows| rows[row_index] == 1);
            (occupied && row[0] == enemy_id).then_some(row_index)
        })
}

fn step_matching(
    session: &mut CombatTrainingSession,
    boundary: &CombatPolicyBoundary,
    predicate: impl Fn(&[i64]) -> bool,
) -> CombatTrainingStep {
    let actions = boundary.observation.group("actions").unwrap();
    let index = actions
        .values
        .chunks(ACTION_WIDTH)
        .position(predicate)
        .expect("requested action is legal");
    session
        .step(boundary.token, RunActionIndex(index as i64))
        .unwrap()
}

fn end_turn(
    session: &mut CombatTrainingSession,
    boundary: &CombatPolicyBoundary,
) -> CombatTrainingStep {
    step_matching(session, boundary, |row| row[0] == 10)
}

fn play_card_at(
    session: &mut CombatTrainingSession,
    boundary: &CombatPolicyBoundary,
    target_row: usize,
) -> CombatTrainingStep {
    step_matching(session, boundary, |row| {
        row[0] == 8 && row[1] == 4 && row[3] == 5 && row[4] == target_row as i64
    })
}

fn play_aoe_card(
    session: &mut CombatTrainingSession,
    boundary: &CombatPolicyBoundary,
) -> CombatTrainingStep {
    step_matching(session, boundary, |row| row[0] == 8 && row[1] == 4 && row[3] != 5)
}

fn use_potion_on(
    session: &mut CombatTrainingSession,
    boundary: &CombatPolicyBoundary,
    slot: usize,
    target_row: usize,
) -> CombatTrainingStep {
    step_matching(session, boundary, |row| {
        row[0] == 6
            && row[6] == slot as i64
            && row[3] == 5
            && row[4] == target_row as i64
    })
}

fn no_relics() -> &'static [&'static str] {
    &[]
}

#[test]
fn v2_catalog_is_the_complete_additive_57_encounter_profile() {
    let canonical = A0_ENCOUNTER_SETUPS
        .iter()
        .map(|(encounter, _, _)| encounter.as_str().to_owned())
        .collect::<Vec<_>>();
    let v1 = support_ids(&combat_training_support());
    let v2 = support_ids(&combat_training_support_v2());

    assert_eq!(canonical.len(), 57);
    assert_eq!(v1.len(), 43);
    assert_eq!(v2.len(), 57);
    assert_eq!(v1, canonical[..43]);
    assert_eq!(v2, canonical);

    let additions = [
        "blue_slaver",
        "two_fungi_beasts",
        "red_slaver",
        "looter",
        "gremlin_gang",
        "exordium_thugs",
        "exordium_wildlife",
        "chosen",
        "shelled_parasite",
        "two_thieves",
        "sentry_and_sphere",
        "snake_plant",
        "cultist_and_chosen",
        "shelled_parasite_and_fungi",
    ];
    let additions_owned = additions.iter().map(|id| (*id).to_owned()).collect::<Vec<_>>();
    assert_eq!(&v2[43..], additions_owned.as_slice());

    for encounter in additions.iter().copied() {
        let input = catalog_json(
            Some(COMBAT_TRAINING_PROFILE_ID),
            vec![entry(
                encounter,
                None,
                80,
                80,
                &[("strike", 0)],
                no_relics(),
                [None; 3],
            )],
        );
        assert!(
            CombatTrainingCatalog::from_json(&input).is_err(),
            "v1 must reject additive encounter {encounter}"
        );
    }
}

#[test]
fn v2_gold_defaults_and_validates_without_changing_v1_inputs() {
    let defaulted = v2_catalog(vec![entry(
        "looter",
        None,
        80,
        80,
        &[("strike", 0)],
        no_relics(),
        [None; 3],
    )]);
    assert_eq!(defaulted.descriptor()["entries"][0]["gold"], 0);

    let owned = v2_catalog(vec![entry(
        "looter",
        Some(json!(20)),
        80,
        80,
        &[("strike", 0)],
        no_relics(),
        [None; 3],
    )]);
    assert_eq!(owned.descriptor()["entries"][0]["gold"], 20);

    for invalid_gold in [Value::Null, json!(true), json!(-1)] {
        let input = catalog_json(
            Some(COMBAT_TRAINING_PROFILE_V2_ID),
            vec![entry(
                "looter",
                Some(invalid_gold.clone()),
                80,
                80,
                &[("strike", 0)],
                no_relics(),
                [None; 3],
            )],
        );
        assert!(
            CombatTrainingCatalog::from_json(&input).is_err(),
            "v2 must reject gold value {invalid_gold}"
        );
    }

    for profile_id in [None, Some(COMBAT_TRAINING_PROFILE_ID)] {
        let input = catalog_json(
            profile_id,
            vec![entry(
                "cultist",
                Some(json!(0)),
                80,
                80,
                &[("strike", 0)],
                no_relics(),
                [None; 3],
            )],
        );
        assert!(
            CombatTrainingCatalog::from_json(&input).is_err(),
            "frozen v1 must reject the gold field even when it is zero"
        );
    }
}

#[test]
fn every_v2_supported_encounter_resets_and_publishes_a_numeric_turn() {
    for row in combat_training_support_v2()["encounters"].as_array().unwrap() {
        let encounter = row["id"].as_str().unwrap();
        let catalog = v2_catalog(vec![entry(
            encounter,
            None,
            80,
            80,
            &[("strike", 0)],
            no_relics(),
            [None; 3],
        )]);
        let mut session = CombatTrainingSession::new(catalog, Partition::Test).unwrap();
        let boundary = session
            .reset(Some(11), None)
            .unwrap_or_else(|error| panic!("{encounter} reset: {error:?}"));
        assert_eq!(boundary.info["profile_id"], COMBAT_TRAINING_PROFILE_V2_ID);
        assert_eq!(player(&boundary)[2], 0, "{encounter} starting gold");
        assert!(boundary
            .observation
            .group("enemies")
            .unwrap()
            .occupied
            .as_ref()
            .unwrap()
            .iter()
            .any(|occupied| *occupied == 1), "{encounter} has no published actor");

        let step = end_turn(&mut session, &boundary);
        assert_eq!(step.boundary.info["profile_id"], COMBAT_TRAINING_PROFILE_V2_ID);
        assert_eq!(player(&step.boundary).len(), 12, "{encounter} player row");
        assert_eq!(step.boundary.observation.group("actions").unwrap().values.len(), 1024 * ACTION_WIDTH);
        assert!(step
            .boundary
            .observation
            .action_mask
            .iter()
            .all(|value| matches!(*value, 0 | 1)));
    }
}

#[test]
fn looter_steals_through_anchor_block_and_keeps_the_reward_unclaimed_at_victory() {
    let catalog = v2_catalog(vec![entry(
        "looter",
        Some(json!(50)),
        80,
        80,
        &[("bash", 0)],
        &["anchor"],
        [Some("fire_potion"), Some("fire_potion"), None],
    )]);
    let mut session = CombatTrainingSession::new(catalog, Partition::Test).unwrap();
    let start = session.reset(Some(11), None).unwrap();
    assert_eq!(player(&start)[2], 50);
    assert_eq!(player(&start)[3], 10, "Anchor grants its opening Block");
    let initial_hp = player(&start)[0];
    let looter = enemy_row(&start, "looter").unwrap();

    let after_bash = play_card_at(&mut session, &start, looter).boundary;
    let after_theft = end_turn(&mut session, &after_bash).boundary;
    assert_eq!(player(&after_theft)[0], initial_hp, "Block prevents the Looter hit");
    assert_eq!(player(&after_theft)[2], 35, "theft ignores player Block");
    assert!(after_theft.info["last_exit"].is_null());

    let looter = enemy_row(&after_theft, "looter").unwrap();
    let first = use_potion_on(&mut session, &after_theft, 0, looter).boundary;
    let looter = enemy_row(&first, "looter").unwrap();
    let won = use_potion_on(&mut session, &first, 1, looter);

    assert!(won.terminated);
    assert_eq!(won.reward, 1);
    assert_eq!(won.boundary.info["outcome"], "victory");
    assert_eq!(player(&won.boundary)[2], 35, "the theft is not refunded");
    assert_eq!(won.boundary.info["last_exit"]["stolen_gold_rewards"], json!([15]));
    assert_eq!(won.boundary.info["last_exit"]["all_enemies_escaped"], false);
}

#[test]
fn two_thieves_debit_sequentially_and_settle_independent_receipts() {
    let catalog = v2_catalog(vec![entry(
        "two_thieves",
        Some(json!(20)),
        999,
        999,
        &[("immolate", 1); 10],
        no_relics(),
        [Some("fire_potion"); 3],
    )]);
    let mut session = CombatTrainingSession::new(catalog, Partition::Test).unwrap();
    let start = session.reset(Some(11), None).unwrap();
    assert_eq!(player(&start)[2], 20);
    let after_theft = end_turn(&mut session, &start).boundary;
    assert_eq!(player(&after_theft)[2], 0);
    assert!(enemy_row(&after_theft, "looter").is_some());
    assert!(enemy_row(&after_theft, "mugger").is_some());

    let mut boundary = play_aoe_card(&mut session, &after_theft).boundary;
    for slot in 0..3 {
        if boundary.info["last_exit"].is_object() {
            break;
        }
        let target = enemy_row(&boundary, if slot == 0 { "looter" } else { "mugger" })
            .or_else(|| enemy_row(&boundary, "looter"))
            .expect("at least one thief remains until settlement");
        boundary = use_potion_on(&mut session, &boundary, slot, target).boundary;
    }

    assert_eq!(boundary.info["outcome"], "victory");
    assert_eq!(player(&boundary)[2], 0);
    assert_eq!(boundary.info["last_exit"]["stolen_gold_rewards"], json!([15, 5]));
    assert_eq!(boundary.info["last_exit"]["all_enemies_escaped"], false);
}

#[test]
fn looter_theft_remains_debited_after_a_real_player_death() {
    let catalog = v2_catalog(vec![entry(
        "looter",
        Some(json!(50)),
        1,
        80,
        &[("defend", 0)],
        no_relics(),
        [None; 3],
    )]);
    let mut session = CombatTrainingSession::new(catalog, Partition::Test).unwrap();
    let start = session.reset(Some(11), None).unwrap();
    let lost = end_turn(&mut session, &start);

    assert!(lost.terminated);
    assert_eq!(lost.reward, -1);
    assert_eq!(lost.boundary.info["outcome"], "death");
    assert_eq!(player(&lost.boundary)[0], 0);
    assert_eq!(player(&lost.boundary)[2], 35);
    assert_eq!(lost.boundary.info["last_exit"]["stolen_gold_rewards"], json!([]),
        "the surviving thief has not died and therefore has earned no recovery receipt");
}

#[test]
fn escaped_thieves_keep_stolen_gold_without_creating_earned_receipts_or_player_death() {
    let catalog = v2_catalog(vec![entry(
        "two_thieves",
        Some(json!(20)),
        999,
        999,
        &[("defend", 0); 10],
        no_relics(),
        [None; 3],
    )]);
    let mut session = CombatTrainingSession::new(catalog, Partition::Test).unwrap();
    let mut boundary = session.reset(Some(11), None).unwrap();
    let mut terminated = false;
    for _ in 0..24 {
        let step = end_turn(&mut session, &boundary);
        boundary = step.boundary;
        if step.terminated {
            terminated = true;
            break;
        }
    }

    assert!(terminated, "both thieves should escape on real combat turns");
    assert_eq!(boundary.info["last_exit"]["all_enemies_escaped"], true);
    assert_eq!(boundary.info["last_exit"]["mugged"], true);
    assert_eq!(boundary.info["last_exit"]["stolen_gold_rewards"], json!([]));
    assert_ne!(boundary.info["outcome"], "death");
    assert!(player(&boundary)[0] > 0, "the player did not die from escape");
    assert_eq!(player(&boundary)[2], 0, "stolen gold is not refunded on escape");
}
