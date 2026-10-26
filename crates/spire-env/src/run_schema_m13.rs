//! Finite Silent Acts 1–2 vocabulary and Act3Ready map boundary.
use crate::{
    run_schema as v1, run_schema_m10 as m10, run_schema_m12 as m12,
    RunCapacities, RunObservationSpec, RunSessionError,
};
use serde_json::{json, Value};
use spire_core::{content::*, run::*};

pub const RUN_OBSERVATION_M13_SCHEMA_ID: &str =
    "spire-gym-m13-silent-act2-observation-v1";

const GENERATED_CARDS: &[CardId] = &[
    CardId::Shiv,
    CardId::Dazed,
    CardId::Burn,
    CardId::Slimed,
    CardId::Wound,
];

const ACT3_MAP_ONLY_BOSS_ICONS: &[EncounterId] = &[
    EncounterId::AwakenedOne,
    EncounterId::TimeEater,
    EncounterId::DonuAndDeca,
];

const ACT2_POWERS: &[&str] = &[
    "artifact",
    "barricade",
    "flight",
    "hex",
    "confused",
    "minion",
    "angry",
    "entangled",
    "painful_stabs",
    "stasis",
];

fn cards() -> impl Iterator<Item = CardId> {
    M12_SILENT_ACT1_CARDS.iter().copied().chain(GENERATED_CARDS.iter().copied())
}

fn padded(values: impl Iterator<Item = &'static str>) -> Vec<&'static str> {
    std::iter::once("padding").chain(values).collect()
}

pub(crate) const M13_PLAYABLE_ENCOUNTERS: &[EncounterId] = &[
    EncounterId::Cultist,
    EncounterId::JawWorm,
    EncounterId::TwoLouse,
    EncounterId::LargeSlime,
    EncounterId::ThreeLouse,
    EncounterId::LotsOfSlimes,
    EncounterId::GremlinNob,
    EncounterId::Lagavulin,
    EncounterId::ThreeSentries,
    EncounterId::Guardian,
    EncounterId::Hexaghost,
    EncounterId::SlimeBoss,
    EncounterId::SphericGuardian,
    EncounterId::ThreeByrds,
    EncounterId::CenturionAndHealer,
    EncounterId::Snecko,
    EncounterId::ChosenAndByrds,
    EncounterId::ThreeCultists,
    EncounterId::GremlinLeader,
    EncounterId::Slavers,
    EncounterId::BookOfStabbing,
    EncounterId::Champ,
    EncounterId::Collector,
    EncounterId::BronzeAutomaton,
];

pub(crate) const M13_MAP_ONLY_BOSS_ICONS: &[EncounterId] = ACT3_MAP_ONLY_BOSS_ICONS;

pub(crate) const M13_ENCOUNTERS: &[EncounterId] = &[
    EncounterId::Cultist,
    EncounterId::JawWorm,
    EncounterId::TwoLouse,
    EncounterId::LargeSlime,
    EncounterId::ThreeLouse,
    EncounterId::LotsOfSlimes,
    EncounterId::GremlinNob,
    EncounterId::Lagavulin,
    EncounterId::ThreeSentries,
    EncounterId::Guardian,
    EncounterId::Hexaghost,
    EncounterId::SlimeBoss,
    EncounterId::Champ,
    EncounterId::BronzeAutomaton,
    EncounterId::Collector,
    EncounterId::SphericGuardian,
    EncounterId::ThreeByrds,
    EncounterId::CenturionAndHealer,
    EncounterId::Snecko,
    EncounterId::ChosenAndByrds,
    EncounterId::ThreeCultists,
    EncounterId::GremlinLeader,
    EncounterId::Slavers,
    EncounterId::BookOfStabbing,
    EncounterId::AwakenedOne,
    EncounterId::TimeEater,
    EncounterId::DonuAndDeca,
];

pub(crate) const M13_PLAYABLE_ENEMIES: &[EnemyId] = &[
    EnemyId::Cultist,
    EnemyId::JawWorm,
    EnemyId::GremlinNob,
    EnemyId::Lagavulin,
    EnemyId::Sentry,
    EnemyId::Guardian,
    EnemyId::Hexaghost,
    EnemyId::LouseRed,
    EnemyId::LouseGreen,
    EnemyId::AcidSlimeS,
    EnemyId::AcidSlimeM,
    EnemyId::AcidSlimeL,
    EnemyId::SpikeSlimeS,
    EnemyId::SpikeSlimeM,
    EnemyId::SpikeSlimeL,
    EnemyId::SlimeBoss,
    EnemyId::SphericGuardian,
    EnemyId::Byrd,
    EnemyId::Chosen,
    EnemyId::Snecko,
    EnemyId::Centurion,
    EnemyId::Mystic,
    EnemyId::GremlinLeader,
    EnemyId::GremlinMad,
    EnemyId::GremlinSneaky,
    EnemyId::GremlinFat,
    EnemyId::GremlinShield,
    EnemyId::GremlinWizard,
    EnemyId::SlaverBlue,
    EnemyId::Taskmaster,
    EnemyId::SlaverRed,
    EnemyId::BookOfStabbing,
    EnemyId::Champ,
    EnemyId::Collector,
    EnemyId::TorchHead,
    EnemyId::BronzeAutomaton,
    EnemyId::BronzeOrb,
];

pub(crate) fn groups(c: &RunCapacities) -> Vec<v1::GroupSpec> {
    m12::groups(c)
}

pub(crate) fn validate(c: &RunCapacities) -> Result<(), RunSessionError> {
    m10::validate_groups(&groups(c), c)
}

pub fn m13_run_observation_spec(
    c: &RunCapacities,
) -> Result<RunObservationSpec, RunSessionError> {
    validate(c)?;
    let vocab = vocabularies();
    m10::spec_from_vocabulary(
        c,
        RUN_OBSERVATION_M13_SCHEMA_ID,
        "M13SilentAct2",
        vocab,
        json!({
            "padding":"zero,known0,occupied0",
            "unknown_reference":"minus_one,known0 in occupied rows",
            "counts":"only published public counts; no hidden draw rows",
            "identity":"finite M13 content IDs and row references only; no raw resolver IDs",
            "power_absence":"no row; listed signed amounts are public",
            "potions":"belt slot rows; empty slot known1/occupied0/kind0; unused transport padding known0",
            "selection":"Run selections expose counts without selected identities; entitled Nightmare and retain combat selections expose selected card rows and operations",
            "known_draw":"only entitled known cards; references point to corresponding combat_cards rows; unknown draw cards have no rows",
            "playing_cards":"accepted public play owners use a dedicated playing zone",
            "actions":"legal candidate order; no paging, truncation or policy",
            "scalar0":"potion belt slot for potion verbs; otherwise absent",
            "scalar1":"reserved absent",
            "offer_amount":"Neow HP/gold/heal/Joust result/Falling type/stage per choice_kind",
            "encounters":{
                "playable":M13_PLAYABLE_ENCOUNTERS.iter().map(|id|id.as_str()).collect::<Vec<_>>(),
                "map_only_boss_icons":M13_MAP_ONLY_BOSS_ICONS.iter().map(|id|id.as_str()).collect::<Vec<_>>()
            },
            "cutoff":"Act3Ready is a nonterminal zero-action boundary after actual Act2 boss settlement; it is not victory and receives no completion reward"
        }),
        groups(c),
        b"spire-gym/run-schema/m13-silent-act2/v1\0",
    )
}

fn append<T: PartialEq>(
    id: T,
    values: &[T],
    message: &'static str,
) -> Result<i64, RunSessionError> {
    values.iter().position(|value| *value == id)
        .map(|index| index as i64 + 1)
        .ok_or(RunSessionError::Input(message))
}

pub(crate) fn card(id: CardId) -> Result<i64, RunSessionError> {
    append(id, &cards().collect::<Vec<_>>(), "unsupported M13 CardId")
}
pub(crate) fn enemy(id: EnemyId) -> Result<i64, RunSessionError> {
    append(id, M13_PLAYABLE_ENEMIES, "unsupported M13 EnemyId")
}
pub(crate) fn encounter(id: EncounterId) -> Result<i64, RunSessionError> {
    append(id, M13_ENCOUNTERS, "unsupported M13 EncounterId")
}
pub(crate) fn relic(id: RelicId) -> Result<i64, RunSessionError> {
    append(id, M12_SILENT_ACT1_RELICS, "unsupported M13 RelicId")
}
pub(crate) fn potion(id: PotionId) -> Result<i64, RunSessionError> {
    append(id, M12_SILENT_ACT1_POTIONS, "unsupported M13 PotionId")
}
pub(crate) fn event(id: EventId) -> Result<i64, RunSessionError> {
    match id {
        EventId::Neow | EventId::UpgradeShrine | EventId::Purifier |
        EventId::AncientWriting | EventId::Joust => v1::event(id),
        _ => Err(RunSessionError::Input("unsupported M13 EventId")),
    }
}
pub(crate) fn profile(profile: RoomScenario) -> Result<i64, RunSessionError> {
    if profile == RoomScenario::M13SilentAct2 {
        Ok(1)
    } else {
        Err(RunSessionError::Input("profile is not representable in the M13 schema"))
    }
}
pub(crate) fn room(kind: RoomViewKind) -> Result<i64, RunSessionError> {
    if kind == RoomViewKind::Act3Ready {
        return v1::room(kind);
    }
    m12::room(kind)
}
pub(crate) fn intent(name: &str) -> Result<i64, RunSessionError> {
    m12::intent(name)
}
pub(crate) fn power(name: &str) -> Result<i64, RunSessionError> {
    if let Ok(value) = m12::power(name) {
        return Ok(value);
    }
    ACT2_POWERS.iter().position(|value| *value == name)
        .map(|index| (v1::POWERS.len() + 3 + index) as i64)
        .ok_or(RunSessionError::Input("unsupported M13 public power"))
}
pub(crate) fn choice(choice: &RoomChoiceKind) -> Result<i64, RunSessionError> {
    m12::choice(choice)
}
pub(crate) fn zone(name: &str) -> Result<i64, RunSessionError> {
    m12::zone(name)
}
pub(crate) fn operation(selection: &spire_core::combat::SelectionView) -> Result<i64, RunSessionError> {
    m12::operation(selection)
}
pub(crate) fn select_permanent_operation() -> i64 {
    m12::select_permanent_operation()
}

pub(crate) fn vocabularies() -> Value {
    let mut vocab = m10::vocabularies();
    vocab["card"] = json!(padded(cards().map(CardId::as_str)));
    vocab["enemy"] = json!(padded(M13_PLAYABLE_ENEMIES.iter().map(|id| id.as_str())));
    vocab["encounter"] = json!(padded(M13_ENCOUNTERS.iter().map(|id| id.as_str())));
    vocab["relic"] = json!(padded(M12_SILENT_ACT1_RELICS.iter().map(|id| id.as_str())));
    vocab["potion"] = json!(padded(M12_SILENT_ACT1_POTIONS.iter().map(|id| id.as_str())));
    vocab["event"] = json!(padded([
        EventId::Neow,
        EventId::UpgradeShrine,
        EventId::Purifier,
        EventId::AncientWriting,
        EventId::Joust,
    ].into_iter().map(EventId::as_str)));
    vocab["profile"] = json!(["padding", "M13SilentAct2"]);
    vocab["room"] = json!(v1::ROOMS[..=17].to_vec());
    vocab["choice"] = json!(v1::CHOICES[..31].to_vec());
    vocab["terminal"] = json!(["ongoing", "death", "empty_deck"]);
    vocab["intent"] = json!(v1::INTENTS.iter().chain(v1::ACT1_INTENTS).copied().collect::<Vec<_>>());
    vocab["power"] = json!(
        v1::POWERS.iter().copied()
            .chain(["noxious_fumes", "accuracy", "retain_cards"])
            .chain(ACT2_POWERS.iter().copied().filter(|name| !v1::POWERS.contains(name)))
            .collect::<Vec<_>>()
    );
    vocab["zone"] = json!([
        "padding", "hand", "discard", "exhaust", "played_power", "stasis", "selection_only",
        "known_draw_top", "known_draw_bottom", "playing"
    ]);
    vocab["selection_operation"] = json!([
        "padding", "move_hand", "move_discard", "move_exhaust", "move_draw_top", "upgrade",
        "topdeck", "return_from_exhaust", "copy_to_hand_1", "copy_to_hand_2",
        "copy_to_hand_3", "select_permanent", "nightmare_capture", "retain"
    ]);
    vocab["row_group"] = json!([
        "none", "player", "map_nodes", "permanent_cards", "combat_cards", "enemies",
        "potions", "offers", "selection", "known_draw"
    ]);
    vocab["verb"] = json!([
        "padding", "move", "choose", "select", "confirm", "cancel", "use_potion",
        "discard_potion", "play_card", "combat_select_card", "end_turn", "confirm_selection"
    ]);
    vocab
}
