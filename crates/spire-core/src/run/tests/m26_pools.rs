use super::super::*;
use crate::combat::{self, rng::Partition, CardInstanceId, CombatAction};
use crate::content::{CardId, Character, EncounterId};
use crate::items::CombatItemContext;

#[derive(Clone, Copy, Debug)]
enum Caller {
    M15Reward,
    M15Merchant,
    M20Reward,
    M20Merchant,
    M25Reward,
    M25Merchant,
}

struct CaptureRow {
    caller: Caller,
    seed: u8,
    key: &'static [u64],
    cards: &'static [&'static str],
}

const SAMPLER_CAPTURES: &[CaptureRow] = &[
    CaptureRow { caller: Caller::M15Reward, seed: 1, key: &[1, 2, 10, 0, 0], cards: &["sucker_punch", "accuracy", "riddle_with_holes"] },
    CaptureRow { caller: Caller::M15Merchant, seed: 1, key: &[1, 4, 23], cards: &["acrobatics", "masterful_stab", "glass_knife"] },
    CaptureRow { caller: Caller::M20Reward, seed: 1, key: &[1, 2, 10, 0, 0], cards: &["leap", "chill", "double_energy"] },
    CaptureRow { caller: Caller::M20Merchant, seed: 1, key: &[1, 4, 23], cards: &["hologram", "melter", "fission"] },
    CaptureRow { caller: Caller::M25Merchant, seed: 1, key: &[4, 52, 2751463506], cards: &["flying_sleeves", "foreign_influence", "omniscience"] },
    CaptureRow { caller: Caller::M25Reward, seed: 1, key: &[4, 53, 2751463507, 3, 1], cards: &["halt", "simmering_fury", "signature_move"] },
    CaptureRow { caller: Caller::M15Reward, seed: 2, key: &[1, 2, 10, 0, 0], cards: &["backflip", "blur", "caltrops"] },
    CaptureRow { caller: Caller::M15Merchant, seed: 2, key: &[1, 3, 15], cards: &["dagger_throw", "all_out_attack", "a_thousand_cuts"] },
    CaptureRow { caller: Caller::M20Reward, seed: 2, key: &[1, 2, 10, 0, 0], cards: &["charge_battery", "recycle", "overclock"] },
    CaptureRow { caller: Caller::M20Merchant, seed: 2, key: &[1, 3, 15], cards: &["stack", "tempest", "biased_cognition"] },
    CaptureRow { caller: Caller::M25Merchant, seed: 2, key: &[4, 52, 2751463506], cards: &["cut_through_fate", "carve_reality", "wish"] },
    CaptureRow { caller: Caller::M25Reward, seed: 2, key: &[4, 53, 2751463507, 3, 1], cards: &["consecrate", "fasting", "pray"] },
    CaptureRow { caller: Caller::M15Reward, seed: 3, key: &[1, 2, 8, 0, 0], cards: &["prepared", "caltrops", "distraction"] },
    CaptureRow { caller: Caller::M15Merchant, seed: 3, key: &[1, 2, 9], cards: &["dagger_throw", "flechettes", "bullet_time"] },
    CaptureRow { caller: Caller::M20Reward, seed: 3, key: &[1, 2, 8, 0, 0], cards: &["recursion", "defragment", "chaos"] },
    CaptureRow { caller: Caller::M20Merchant, seed: 3, key: &[1, 2, 9], cards: &["barrage", "double_energy", "echo_form"] },
    CaptureRow { caller: Caller::M25Merchant, seed: 3, key: &[4, 52, 2751463506], cards: &["bowling_bash", "sands_of_time", "conjure_blade"] },
    CaptureRow { caller: Caller::M25Reward, seed: 3, key: &[4, 53, 2751463507, 3, 1], cards: &["follow_up", "sanctity", "tantrum"] },
    CaptureRow { caller: Caller::M15Reward, seed: 4, key: &[1, 2, 7, 0, 0], cards: &["deadly_poison", "tactician", "calculated_gamble"] },
    CaptureRow { caller: Caller::M15Merchant, seed: 4, key: &[1, 3, 17], cards: &["bane", "masterful_stab", "envenom"] },
    CaptureRow { caller: Caller::M20Reward, seed: 4, key: &[1, 2, 7, 0, 0], cards: &["compile_driver", "melter", "hello_world"] },
    CaptureRow { caller: Caller::M20Merchant, seed: 4, key: &[1, 3, 17], cards: &["go_for_the_eyes", "consume", "machine_learning"] },
    CaptureRow { caller: Caller::M25Merchant, seed: 4, key: &[4, 52, 2751463506], cards: &["halt", "mental_fortress", "deus_ex_machina"] },
    CaptureRow { caller: Caller::M25Reward, seed: 4, key: &[4, 53, 2751463507, 3, 1], cards: &["follow_up", "wheel_kick", "swivel"] },
    CaptureRow { caller: Caller::M15Reward, seed: 5, key: &[1, 2, 7, 0, 0], cards: &["deadly_poison", "blur", "flechettes"] },
    CaptureRow { caller: Caller::M15Merchant, seed: 5, key: &[1, 8, 51], cards: &["flying_knee", "distraction", "glass_knife"] },
    CaptureRow { caller: Caller::M20Reward, seed: 5, key: &[1, 2, 7, 0, 0], cards: &["sweeping_beam", "chaos", "auto_shields"] },
    CaptureRow { caller: Caller::M20Merchant, seed: 5, key: &[1, 8, 51], cards: &["compile_driver", "chaos", "machine_learning"] },
    CaptureRow { caller: Caller::M25Merchant, seed: 5, key: &[4, 52, 2751463506], cards: &["bowling_bash", "empty_mind", "deva_form"] },
    CaptureRow { caller: Caller::M25Reward, seed: 5, key: &[4, 53, 2751463507, 3, 1], cards: &["sash_whip", "swivel", "deceive_reality"] },
    CaptureRow { caller: Caller::M15Reward, seed: 6, key: &[1, 2, 8, 0, 0], cards: &["acrobatics", "endless_agony", "terror"] },
    CaptureRow { caller: Caller::M15Merchant, seed: 6, key: &[1, 3, 18], cards: &["dagger_spray", "caltrops", "tools_of_the_trade"] },
    CaptureRow { caller: Caller::M20Reward, seed: 6, key: &[1, 2, 8, 0, 0], cards: &["go_for_the_eyes", "bullseye", "boot_sequence"] },
    CaptureRow { caller: Caller::M20Merchant, seed: 6, key: &[1, 3, 18], cards: &["sweeping_beam", "recycle", "hyperbeam"] },
    CaptureRow { caller: Caller::M25Merchant, seed: 6, key: &[4, 52, 2751463506], cards: &["empty_fist", "nirvana", "judgment"] },
    CaptureRow { caller: Caller::M25Reward, seed: 6, key: &[4, 53, 2751463507, 3, 1], cards: &["flurry_of_blows", "wheel_kick", "swivel"] },
    CaptureRow { caller: Caller::M15Reward, seed: 7, key: &[1, 2, 10, 0, 0], cards: &["outmaneuver", "blur", "footwork"] },
    CaptureRow { caller: Caller::M15Merchant, seed: 7, key: &[1, 6, 40], cards: &["poisoned_stab", "catalyst", "adrenaline"] },
    CaptureRow { caller: Caller::M20Reward, seed: 7, key: &[1, 2, 10, 0, 0], cards: &["streamline", "consume", "blizzard"] },
    CaptureRow { caller: Caller::M20Merchant, seed: 7, key: &[1, 6, 40], cards: &["compile_driver", "doom_and_gloom", "rainbow"] },
    CaptureRow { caller: Caller::M25Merchant, seed: 7, key: &[4, 52, 2751463506], cards: &["follow_up", "talk_to_the_hand", "deva_form"] },
    CaptureRow { caller: Caller::M25Reward, seed: 7, key: &[4, 53, 2751463507, 3, 1], cards: &["empty_fist", "foresight", "battle_hymn"] },
    CaptureRow { caller: Caller::M15Reward, seed: 8, key: &[1, 2, 8, 0, 0], cards: &["outmaneuver", "backstab", "bouncing_flask"] },
    CaptureRow { caller: Caller::M15Merchant, seed: 8, key: &[1, 3, 20], cards: &["sneaky_strike", "infinite_blades", "envenom"] },
    CaptureRow { caller: Caller::M20Reward, seed: 8, key: &[1, 2, 8, 0, 0], cards: &["coolheaded", "bullseye", "double_energy"] },
    CaptureRow { caller: Caller::M20Merchant, seed: 8, key: &[1, 3, 20], cards: &["go_for_the_eyes", "ftl", "core_surge"] },
    CaptureRow { caller: Caller::M25Merchant, seed: 8, key: &[4, 52, 2751463506], cards: &["flurry_of_blows", "perseverance", "conjure_blade"] },
    CaptureRow { caller: Caller::M25Reward, seed: 8, key: &[4, 53, 2751463507, 3, 1], cards: &["evaluate", "inner_peace", "tantrum"] },
];

fn run_rng_counter(rng: &RunRng, scope_debug: &str, key: &[u64]) -> u64 {
    let marker = format!("({scope_debug}, {key:?}): ");
    let debug = format!("{rng:?}");
    let Some(start) = debug.find(&marker) else { return 0; };
    let digits = debug[start + marker.len()..]
        .chars()
        .take_while(char::is_ascii_digit)
        .collect::<String>();
    digits.parse().expect("RunRng Debug counter is decimal")
}

#[test]
fn m26_pools_sampler_generalization_is_rng_identical() {
    use crate::content::CardRarity::{Common, Rare, Uncommon};
    for row in SAMPLER_CAPTURES {
        let rarities = match row.caller {
            Caller::M15Reward | Caller::M20Reward | Caller::M25Reward =>
                [Common, Uncommon, Uncommon],
            Caller::M15Merchant | Caller::M20Merchant | Caller::M25Merchant =>
                [Common, Uncommon, Rare],
        };
        let mut rng = RunRng::new([row.seed; 32], Partition::Test, 0);
        let scope_debug = match row.caller {
            Caller::M15Reward | Caller::M15Merchant => "M15CardOffer",
            Caller::M20Reward | Caller::M20Merchant => "M20CardOffer",
            Caller::M25Reward | Caller::M25Merchant => "M25CardOffer",
        };
        for slot in 0..3 {
            let mut key = row.key.to_vec();
            key.push(slot);
            assert_eq!(run_rng_counter(&rng, scope_debug, &key), 0,
                "{:?} seed {} slot {slot} counter before", row.caller, row.seed);
        }
        let actual = match row.caller {
            Caller::M15Reward | Caller::M15Merchant =>
                rooms::m15_content::sample_capture_row(&mut rng, row.key, &rarities),
            Caller::M20Reward | Caller::M20Merchant =>
                rooms::m20_content::sample_capture_row(&mut rng, row.key, &rarities),
            Caller::M25Reward | Caller::M25Merchant =>
                rooms::m25_content::sample_capture_row(&mut rng, row.key, &rarities),
        };
        let actual = actual.unwrap_or_else(|error| panic!("{:?} seed {} key {:?}: {error:?}", row.caller, row.seed, row.key));
        let actual = actual.iter().map(|id| id.as_str()).collect::<Vec<_>>();
        assert_eq!(actual, row.cards.to_vec(), "{:?} seed {} key {:?}", row.caller, row.seed, row.key);
        for slot in 0..3 {
            let mut key = row.key.to_vec();
            key.push(slot);
            assert_eq!(run_rng_counter(&rng, scope_debug, &key), 1,
                "{:?} seed {} slot {slot} counter after", row.caller, row.seed);
        }
    }
}

#[test]
fn m26_pools_rng_scopes() {
    let existing = [
        ("run.neow.category0", RunRngScope::NeowCategory0),
        ("run.neow.category1", RunRngScope::NeowCategory1),
        ("run.neow.drawback", RunRngScope::NeowDrawback),
        ("run.neow.category2", RunRngScope::NeowCategory2),
        ("run.map.topology", RunRngScope::MapTopology),
        ("run.map.room_bucket", RunRngScope::MapRoomBucket),
        ("run.map.burning_elite", RunRngScope::MapBurningElite),
        ("run.map.encounter", RunRngScope::MapEncounter),
        ("run.map.question_room", RunRngScope::MapQuestionRoom),
        ("run.combat.enemy.setup", RunRngScope::CombatEnemySetup),
        ("run.reward.gold", RunRngScope::CombatGoldReward),
        ("run.act1.boss_order", RunRngScope::Act1BossOrder),
        ("run.combat.burning_elite.buff", RunRngScope::BurningEliteBuff),
        ("run.act2.boss_order", RunRngScope::Act2BossOrder),
        ("run.act2.joust", RunRngScope::Act2Joust),
        ("run.act3.boss_order", RunRngScope::Act3BossOrder),
        ("run.act3.falling", RunRngScope::Act3Falling),
        ("run.m10.card.offer", RunRngScope::M10CardOffer),
        ("run.m15.potion.generate", RunRngScope::M15PotionGeneration),
        ("run.m15.card.offer", RunRngScope::M15CardOffer),
        ("run.m20.card.offer", RunRngScope::M20CardOffer),
        ("run.m22.relic.capture", RunRngScope::M22RelicCapture),
        ("run.m23.relic.capture", RunRngScope::M23RelicCapture),
        ("run.m24.relic.capture", RunRngScope::M24RelicCapture),
        ("run.m25.card.offer", RunRngScope::M25CardOffer),
        ("run.m25.relic.capture", RunRngScope::M25RelicCapture),
    ];
    for (spelling, scope) in existing {
        assert_eq!(RunRngScope::parse(spelling), Ok(scope), "pre-M26 scope {spelling}");
    }

    let added = [
        ("run.m26.card.offer", RunRngScope::M26CardOffer, 12_210_395_265_854_424_081u64),
        ("run.m26.card.transform", RunRngScope::M26CardTransform, 15_641_866_729_566_123_980u64),
        ("run.m26.card.curse", RunRngScope::M26CardCurse, 16_334_826_230_206_982_708u64),
    ];
    for (spelling, scope, expected_word) in added {
        assert_eq!(RunRngScope::parse(spelling), Ok(scope), "{spelling} parses");
        let mut rng = RunRng::new([31; 32], Partition::Test, 5);
        assert_eq!(rng.draw_scoped(scope, &[9]), expected_word,
            "{spelling} is the exact scope string used by the RNG encoder");
    }
    assert!(RunRngScope::M25RelicCapture < RunRngScope::M26CardOffer);
    assert!(RunRngScope::M26CardOffer < RunRngScope::M26CardTransform);
    assert!(RunRngScope::M26CardTransform < RunRngScope::M26CardCurse);
}

fn m26_infernal_combat(seed: u8) -> combat::Combat {
    let mut profile = CreationProfile::experienced();
    profile.neow_eligible = false;
    let mut source = Run::new_room_scenario(
        RunConfig { character: Character::Ironclad, ascension: 0 },
        profile,
        RunRng::new([seed; 32], Partition::Test, 0),
        RoomScenario::M10IroncladHeart,
    ).expect("accepted M10 Ironclad A0 scenario constructor");
    let infernal_instance = mutations::obtain_card(&mut source.state, CardId::InfernalBlade, 0)
        .expect("M10 admits Infernal Blade");
    let remove_instance = source.state.deck.iter().find(|card| card.id == CardId::Defend)
        .expect("M10 Ironclad starter Defend").instance;
    source.state.deck.retain(|card| card.instance != remove_instance);
    assert_eq!(source.state.deck.len(), 10);
    assert!(source.state.deck.iter().any(|card| card.instance == infernal_instance));
    let cards = source.state.deck.iter().map(|card|
        combat::permanent_live_card(card.id, CardInstanceId(card.instance), card.upgrades)
    ).collect::<Vec<_>>();
    let context = combat::enemies::SetupContext {
        player: combat::PlayerState {
            hp: source.state.hp as i32, max_hp: source.state.max_hp as i32, block: 0,
            energy: 3, max_energy: 3, draw_per_turn: 5, powers: vec![],
        },
        player_powers: vec![],
        character: Character::Ironclad,
        opening_draw: cards.len(),
        cards,
        encounter_visit: source.state.combat_progress.entered,
    };
    let entry = combat::enemies::setup(
        EncounterId::Cultist, 1, 0, &mut source.state.rng, context,
    ).expect("frozen M10 A0 Ironclad Act1 setup");
    let mut inventory = Some(CombatItemContext::new(
        source.state.relics.clone(), source.state.potions.clone(),
    ));
    let combat = combat::Combat::enter_with_item_context(entry, &mut inventory)
        .expect("frozen full-runtime M10 combat entry");
    assert!(inventory.is_none());
    assert_eq!(combat.native_generation_character(), None);
    combat
}

fn combat_card_rng_word(combat: &combat::Combat) -> u64 {
    let debug = format!("{combat:?}");
    let card_stream = debug.split_once("card_rng: Some(")
        .map(|(_, stream)| stream).expect("M10 combat has a card RNG stream");
    let digits = card_stream.split_once("word_index: ")
        .map(|(_, suffix)| suffix).expect("StreamState Debug has word_index")
        .chars().take_while(char::is_ascii_digit).collect::<String>();
    digits.parse().expect("combat card RNG word index is decimal")
}

#[test]
fn m26_pools_infernal_blade_m10_replay() {
    let expected = [
        (1, "iron_wave", 0, 1), (2, "thunderclap", 0, 1),
        (3, "searing_blow", 0, 1), (4, "perfected_strike", 0, 1),
        (5, "clothesline", 0, 1), (6, "uppercut", 0, 1),
        (7, "pommel_strike", 0, 1), (8, "dropkick", 0, 1),
        (9, "bludgeon", 0, 1), (10, "dropkick", 0, 1),
        (11, "body_slam", 0, 1), (12, "cleave", 0, 1),
        (13, "body_slam", 0, 1), (14, "wild_strike", 0, 1),
        (15, "fiend_fire", 0, 1), (16, "blood_for_blood", 0, 1),
    ];
    for (seed, expected_id, expected_before, expected_after) in expected {
        let mut combat = m26_infernal_combat(seed);
        let before = combat_card_rng_word(&combat);
        let parent = combat.public_full().unwrap().hand.into_iter()
            .find(|card| card.id == CardId::InfernalBlade)
            .expect("opening hand contains Infernal Blade").instance.0;
        combat.step_full(&CombatAction::PlayCard { instance: parent, target: None })
            .expect("real Infernal Blade play");
        let after = combat_card_rng_word(&combat);
        let generated = combat.public_full().unwrap().hand.into_iter()
            .find(|card| card.instance.0 != parent
                && rooms::m10_content::infernal_candidates().contains(&card.id))
            .expect("Infernal Blade creates an M10-supported Attack in hand");
        assert_eq!(generated.id.as_str(), expected_id, "seed {seed}");
        assert_eq!(before, expected_before, "seed {seed} word index before");
        assert_eq!(after, expected_after, "seed {seed} word index after");
    }
}
