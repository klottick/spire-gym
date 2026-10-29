//! Finite Silent A0-through-Heart vocabulary on the shared native Run schema.
use crate::{
    run_schema as v1, run_schema_m10 as m10, run_schema_m13 as m13,
    run_schema_m14 as m14, RunCapacities, RunObservationSpec, RunSessionError,
};
use serde_json::{json, Value};
use spire_core::{
    combat::{SelectionDisposition::*, SelectionOperation::*},
    content::*,
    run::*,
};
use std::sync::OnceLock;

pub const RUN_OBSERVATION_M15_SCHEMA_ID: &str =
    "spire-gym-m15-silent-heart-observation-v1";

const M15_COLORLESS_CARDS: &[CardId] = &[
    CardId::Apotheosis,
    CardId::Blind,
    CardId::Chrysalis,
    CardId::DarkShackles,
    CardId::DeepBreath,
    CardId::Discovery,
    CardId::DramaticEntrance,
    CardId::Enlightenment,
    CardId::Finesse,
    CardId::FlashOfSteel,
    CardId::Forethought,
    CardId::GoodInstincts,
    CardId::HandOfGreed,
    CardId::Impatience,
    CardId::JackOfAllTrades,
    CardId::Madness,
    CardId::Magnetism,
    CardId::MasterOfStrategy,
    CardId::Mayhem,
    CardId::Metamorphosis,
    CardId::MindBlast,
    CardId::Panacea,
    CardId::Panache,
    CardId::PanicButton,
    CardId::Purity,
    CardId::SadisticNature,
    CardId::SecretTechnique,
    CardId::SecretWeapon,
    CardId::SwiftStrike,
    CardId::TheBomb,
    CardId::ThinkingAhead,
    CardId::Transmutation,
    CardId::Trip,
    CardId::Violence,
];

const M15_GENERATED_AND_STATUS_CARDS: &[CardId] = &[
    CardId::Shiv,
    CardId::Dazed,
    CardId::Burn,
    CardId::Slimed,
    CardId::Wound,
    CardId::Void,
];

const M15_ACT4_ENCOUNTERS: &[EncounterId] = &[
    EncounterId::ShieldAndSpear,
    EncounterId::CorruptHeart,
];

const M15_ACT4_INTENTS: &[&str] = &[
    "shield_bash",
    "shield_fortify",
    "shield_smash",
    "spear_burn_strike",
    "spear_skewer",
    "spear_strength",
    "heart_debilitate",
    "heart_blood_shots",
    "heart_echo",
    "heart_buff",
];

const M15_ACT4_POWERS: &[&str] = &[
    "beat_of_death",
    "invincible",
    "surrounded",
    "back_attack",
];

const M15_SILENT_POWERS: &[&str] = &[
    "a_thousand_cuts",
    "after_image",
    "blur",
    "burst",
    "choked",
    "corpse_explosion",
    "double_damage",
    "draw_card_next_turn",
    "energized",
    "envenom",
    "infinite_blades",
    "intangible_player",
    "magnetism",
    "mayhem",
    "next_turn_block",
    "no_block",
    "no_draw",
    "panache",
    "phantasmal",
    "sadistic_nature",
    "tools_of_the_trade",
    "wraith_form",
    "lose_strength",
    "lose_dexterity",
    "plated_armor",
    "duplication",
    "panache_countdown",
    "the_bomb",
    "the_bomb_plus",
];

const M15_SELECTION_OPERATIONS: &[&str] = &[
    "move_hand",
    "move_discard",
    "move_exhaust",
    "move_draw_top",
    "upgrade",
    "topdeck",
    "return_from_exhaust",
    "copy_to_hand_1",
    "copy_to_hand_2",
    "copy_to_hand_3",
    "select_permanent",
    "nightmare_capture",
    "retain",
    "setup",
    "bottomdeck",
    "retrieve_memories",
];

fn append_unique<T: PartialEq + Copy>(values: &mut Vec<T>, additions: impl IntoIterator<Item = T>) {
    for value in additions {
        if !values.contains(&value) {
            values.push(value);
        }
    }
}

fn padded_ids(values: impl IntoIterator<Item = &'static str>) -> Vec<&'static str> {
    std::iter::once("padding").chain(values).collect()
}

fn card_ids() -> Vec<CardId> {
    M15_SILENT_HEART_CARDS
        .iter()
        .copied()
        .chain(M15_COLORLESS_CARDS.iter().copied())
        .chain(M15_GENERATED_AND_STATUS_CARDS.iter().copied())
        .collect()
}

fn enemy_ids() -> Vec<EnemyId> {
    let mut ids = m13::M13_PLAYABLE_ENEMIES.to_vec();
    append_unique(&mut ids, m14::M14_ACT3_ENEMIES.iter().copied());
    append_unique(&mut ids, m10::NEW_ENEMIES.iter().copied());
    ids
}

fn encounter_ids() -> Vec<EncounterId> {
    let mut ids = m13::M13_ENCOUNTERS.to_vec();
    append_unique(&mut ids, m14::M14_ACT3_ENCOUNTERS.iter().copied());
    append_unique(&mut ids, M15_ACT4_ENCOUNTERS.iter().copied());
    ids
}

fn playable_encounters() -> Vec<EncounterId> {
    let mut ids = m13::M13_PLAYABLE_ENCOUNTERS.to_vec();
    append_unique(&mut ids, m14::M14_ACT3_ENCOUNTERS.iter().copied());
    append_unique(&mut ids, M15_ACT4_ENCOUNTERS.iter().copied());
    ids
}

fn padded_names(values: impl IntoIterator<Item = &'static str>) -> Vec<&'static str> {
    std::iter::once("padding").chain(values).collect()
}

fn intent_names() -> &'static [String] {
    static NAMES: OnceLock<Vec<String>> = OnceLock::new();
    NAMES.get_or_init(|| {
        let mut names = m13::vocabularies()["intent"]
            .as_array()
            .expect("M13 intent vocabulary")
            .iter()
            .map(|value| value.as_str().expect("intent name").to_owned())
            .collect::<Vec<_>>();
        for name in M15_ACT4_INTENTS {
            if !names.iter().any(|existing| existing == name) {
                names.push((*name).to_owned());
            }
        }
        names
    })
}

fn power_names() -> &'static [String] {
    static NAMES: OnceLock<Vec<String>> = OnceLock::new();
    NAMES.get_or_init(|| {
        let mut names = m13::vocabularies()["power"]
            .as_array()
            .expect("M13 power vocabulary")
            .iter()
            .map(|value| value.as_str().expect("power name").to_owned())
            .collect::<Vec<_>>();
        for name in M15_ACT4_POWERS.iter().chain(M15_SILENT_POWERS) {
            if !names.iter().any(|existing| existing == name) {
                names.push((*name).to_owned());
            }
        }
        names
    })
}

pub(crate) fn groups(c: &RunCapacities) -> Vec<v1::GroupSpec> {
    let mut groups = m10::groups(c);
    groups.push(v1::GroupSpec {
        key: "combat_card_offers",
        rows: Some(c.offers),
        fields: "card_kind,offer_index",
        units: "category,offer_index",
        references: &[],
    });
    groups
}

pub(crate) fn validate(c: &RunCapacities) -> Result<(), RunSessionError> {
    m10::validate_groups(&groups(c), c)
}

pub fn m15_run_observation_spec(
    c: &RunCapacities,
) -> Result<RunObservationSpec, RunSessionError> {
    validate(c)?;
    let vocab = vocabularies();
    let encounters = playable_encounters();
    let enemies = enemy_ids();
    let generated_and_status = M15_COLORLESS_CARDS
        .iter()
        .chain(M15_GENERATED_AND_STATUS_CARDS)
        .map(|id| id.as_str())
        .collect::<Vec<_>>();
    let semantics = json!({
        "padding":"zero,known0,occupied0",
        "unknown_reference":"minus_one,known0 in occupied rows",
        "counts":"only published public counts; no hidden draw rows",
        "identity":"finite M15 content IDs and row references only; no raw resolver IDs",
        "power_absence":"no row; listed signed amounts are public",
        "potions":"belt slot rows; empty slot known1/occupied0/kind0; unused transport padding known0",
        "selection":"Run selections expose counts without selected identities; entitled Nightmare, retain, Setup, Bottomdeck, and RetrieveMemories combat selections expose public card rows and operations",
        "known_draw":"only entitled known cards; references point to corresponding combat_cards rows; unknown draw cards have no rows",
        "playing_cards":"accepted public play owners use a dedicated playing zone",
        "card_offers":"only current publicly offered card IDs and zero-based offer indexes; no private template handles or future offers",
        "actions":"legal candidate order; no paging, truncation or policy",
        "scalar0":"potion belt slot for potion verbs; otherwise absent",
        "scalar1":"reserved absent",
        "offer_amount":"Neow HP/gold/heal/Joust result/Falling type/stage per choice_kind",
        "combat_policy_view":{"supported":false,"combat_view_id":null},
        "power_countdowns":{
            "panache_countdown":{"minimum":1,"maximum":5,"damage_power":"panache","amount":"remaining card plays"},
            "the_bomb":{"minimum":1,"maximum":3,"damage":40,"amount":"remaining turns"},
            "the_bomb_plus":{"minimum":1,"maximum":3,"damage":50,"amount":"remaining turns"},
            "bomb_rows":"one row per independent timer, in existing creation/callback order; repeated power IDs do not aggregate or sort by damage",
            "projection_order":"ordinary power rows retain name order, then panache_countdown, then independent Bomb timers"
        },
        "entitlements":{
            "permanent_cards":spire_core::run::M15_SILENT_HEART_CARDS.iter().map(|id|id.as_str()).collect::<Vec<_>>(),
            "generated_and_status_cards":generated_and_status,
            "parasite":"permanent curse from actual Writhing Mass custody; not an ordinary reward or shop card",
            "relics":spire_core::run::M15_SILENT_HEART_RELICS.iter().map(|id|id.as_str()).collect::<Vec<_>>(),
            "potions":spire_core::run::M15_SILENT_HEART_POTIONS.iter().map(|id|id.as_str()).collect::<Vec<_>>()
        },
        "encounters":{
            "playable":encounters.iter().map(|id|id.as_str()).collect::<Vec<_>>(),
            "map_only_boss_icons":[],
            "act4":M15_ACT4_ENCOUNTERS.iter().map(|id|id.as_str()).collect::<Vec<_>>()
        },
        "enemy_species":{
            "species":enemies.iter().map(|id|id.as_str()).collect::<Vec<_>>(),
            "act4":m10::NEW_ENEMIES.iter().map(|id|id.as_str()).collect::<Vec<_>>(),
            "cumulative_through_act4":enemies.iter().map(|id|id.as_str()).collect::<Vec<_>>()
        },
        "act4":{
            "encounters":M15_ACT4_ENCOUNTERS.iter().map(|id|id.as_str()).collect::<Vec<_>>(),
            "enemies":m10::NEW_ENEMIES.iter().map(|id|id.as_str()).collect::<Vec<_>>(),
            "intents":M15_ACT4_INTENTS,
            "powers":M15_ACT4_POWERS
        },
        "endings":{
            "act3_no_keys":"actual Act3 boss Victory and shared Proceed/dialog completion; no Heart reward",
            "heart_victory":"actual Heart Victory and the legal final Proceed action",
            "act4_ready":"M14's supplied cutoff remains a nonterminal boundary; M15 adds actual Act4 gameplay"
        }
    });
    m10::spec_from_vocabulary(
        c,
        RUN_OBSERVATION_M15_SCHEMA_ID,
        "M15SilentHeart",
        vocab,
        semantics,
        groups(c),
        b"spire-gym/run-schema/m15-silent-heart/v1\0",
    )
}

fn vocabularies() -> Value {
    let mut vocab = m13::vocabularies();
    vocab["card"] = json!(padded_ids(card_ids().into_iter().map(CardId::as_str)));
    vocab["enemy"] = json!(padded_ids(enemy_ids().into_iter().map(EnemyId::as_str)));
    vocab["encounter"] = json!(padded_ids(encounter_ids().into_iter().map(EncounterId::as_str)));
    vocab["relic"] = json!(padded_ids(
        spire_core::run::M15_SILENT_HEART_RELICS.iter().copied().map(RelicId::as_str)
    ));
    vocab["potion"] = json!(padded_ids(
        spire_core::run::M15_SILENT_HEART_POTIONS.iter().copied().map(PotionId::as_str)
    ));
    vocab["event"] = json!(padded_names([
        EventId::Neow.as_str(),
        EventId::UpgradeShrine.as_str(),
        EventId::Purifier.as_str(),
        EventId::AncientWriting.as_str(),
        EventId::Joust.as_str(),
        EventId::Falling.as_str(),
        EventId::TombOfLordRedMask.as_str(),
    ]));
    vocab["profile"] = json!(["padding", "M15SilentHeart"]);
    vocab["room"] = m10::vocabularies()["room"].clone();
    vocab["terminal"] = m10::vocabularies()["terminal"].clone();
    vocab["outcome"] = json!(["ongoing", "victory", "death", "escaped"]);
    vocab["choice"] = json!(v1::CHOICES);
    vocab["intent"] = json!(intent_names());
    vocab["power"] = json!(power_names());
    vocab["selection_operation"] = json!(std::iter::once("padding")
        .chain(M15_SELECTION_OPERATIONS.iter().copied()).collect::<Vec<_>>());

    let mut row_groups = m10::vocabularies()["row_group"].as_array().unwrap().clone();
    row_groups.push(Value::String("combat_card_offers".into()));
    vocab["row_group"] = Value::Array(row_groups);
    let mut verbs = m10::vocabularies()["verb"].as_array().unwrap().clone();
    verbs.extend([Value::String("choose_card_offer".into()), Value::String("skip_card_offer".into())]);
    vocab["verb"] = Value::Array(verbs);
    vocab
}

fn append_id<T: PartialEq>(id: T, values: &[T], message: &'static str) -> Result<i64, RunSessionError> {
    values.iter()
        .position(|value| *value == id)
        .map(|index| index as i64 + 1)
        .ok_or(RunSessionError::Input(message))
}

fn named(name: &str, values: &[String], message: &'static str) -> Result<i64, RunSessionError> {
    values.iter()
        .position(|value| value == name)
        .filter(|index| *index > 0)
        .map(|index| index as i64)
        .ok_or(RunSessionError::Input(message))
}

pub(crate) fn card(id: CardId) -> Result<i64, RunSessionError> {
    append_id(id, &card_ids(), "unsupported M15 CardId")
}

pub(crate) fn enemy(id: EnemyId) -> Result<i64, RunSessionError> {
    append_id(id, &enemy_ids(), "unsupported M15 EnemyId")
}

pub(crate) fn encounter(id: EncounterId) -> Result<i64, RunSessionError> {
    append_id(id, &encounter_ids(), "unsupported M15 EncounterId")
}

pub(crate) fn relic(id: RelicId) -> Result<i64, RunSessionError> {
    append_id(id, spire_core::run::M15_SILENT_HEART_RELICS, "unsupported M15 RelicId")
}

pub(crate) fn potion(id: PotionId) -> Result<i64, RunSessionError> {
    append_id(id, spire_core::run::M15_SILENT_HEART_POTIONS, "unsupported M15 PotionId")
}

pub(crate) fn event(id: EventId) -> Result<i64, RunSessionError> {
    m14::event(id)
}

pub(crate) fn profile(profile: RoomScenario) -> Result<i64, RunSessionError> {
    if profile == RoomScenario::M15SilentHeart {
        Ok(1)
    } else {
        Err(RunSessionError::Input("profile is not representable in the M15 schema"))
    }
}

pub(crate) fn room(kind: RoomViewKind) -> Result<i64, RunSessionError> {
    m10::room(kind)
}

pub(crate) fn intent(name: &str) -> Result<i64, RunSessionError> {
    named(name, intent_names(), "unsupported M15 public intent")
}

pub(crate) fn power(name: &str) -> Result<i64, RunSessionError> {
    named(name, power_names(), "unsupported M15 public power")
}

pub(crate) fn choice(choice: &RoomChoiceKind) -> Result<i64, RunSessionError> {
    Ok(v1::choice(choice))
}

pub(crate) fn zone(name: &str) -> Result<i64, RunSessionError> {
    m10::zone(name)
}

pub(crate) fn operation(selection: &spire_core::combat::SelectionView) -> Result<i64, RunSessionError> {
    let name = match selection.operation {
        Scry|MeditateReturn=>return Err(RunSessionError::UnsupportedBoundary("Watcher selection requires M21 native profile")),
        HologramRetrieve|SeekRetrieve|Recycle=>return Err(RunSessionError::UnsupportedBoundary("M20 selection requires M20 native profile")),
        Move(Hand) => "move_hand",
        Move(Discard) => "move_discard",
        Move(Exhaust) => "move_exhaust",
        Move(DrawTop) => "move_draw_top",
        Upgrade => "upgrade",
        Topdeck => "topdeck",
        ReturnFromExhaust => "return_from_exhaust",
        CopyToHand { copies: 1 } => "copy_to_hand_1",
        CopyToHand { copies: 2 } => "copy_to_hand_2",
        CopyToHand { copies: 3 } => "copy_to_hand_3",
        CopyToHand { .. } => return Err(RunSessionError::Input("unsupported M15 selection copy count")),
        NightmareCapture => "nightmare_capture",
        Retain => "retain",
        Setup => "setup",
        Bottomdeck => "bottomdeck",
        RetrieveMemories => "retrieve_memories",
    };
    M15_SELECTION_OPERATIONS.iter()
        .position(|value| *value == name)
        .map(|index| index as i64 + 1)
        .ok_or(RunSessionError::Input("unsupported M15 selection operation"))
}

pub(crate) fn select_permanent_operation() -> i64 {
    M15_SELECTION_OPERATIONS.iter()
        .position(|value| *value == "select_permanent")
        .expect("M15 schema declares permanent selection") as i64 + 1
}
