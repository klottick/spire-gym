//! Finite Silent Act1 vocabulary on the shared versioned Run schema owner.
use crate::{run_schema as v1, run_schema_m10 as m10, RunCapacities, RunObservationSpec, RunSessionError};
use serde_json::{json, Value};
use spire_core::{content::*, run::*};

pub const RUN_OBSERVATION_M12_SCHEMA_ID: &str = "spire-gym-m12-silent-act1-observation-v1";

const GENERATED_CARDS: &[CardId] = &[CardId::Shiv, CardId::Dazed, CardId::Burn, CardId::Slimed];
const M12_POWERS: &[&str] = &["noxious_fumes", "accuracy", "retain_cards"];
const M12_SELECTION_OPERATIONS: &[&str] = &[
    "move_hand", "move_discard", "move_exhaust", "move_draw_top", "upgrade", "topdeck",
    "return_from_exhaust", "copy_to_hand_1", "copy_to_hand_2", "copy_to_hand_3",
    "select_permanent", "nightmare_capture", "retain",
];
pub(crate) const M12_PLAYABLE_ENCOUNTERS: &[EncounterId] = &[
    EncounterId::Cultist, EncounterId::JawWorm, EncounterId::TwoLouse,
    EncounterId::LargeSlime, EncounterId::ThreeLouse, EncounterId::LotsOfSlimes,
    EncounterId::GremlinNob, EncounterId::Lagavulin, EncounterId::ThreeSentries,
    EncounterId::Guardian, EncounterId::Hexaghost, EncounterId::SlimeBoss,
];
pub(crate) const M12_MAP_ONLY_BOSS_ICONS: &[EncounterId] = &[
    EncounterId::Champ, EncounterId::BronzeAutomaton, EncounterId::Collector,
];
const M12_ENCOUNTERS: &[EncounterId] = &[
    EncounterId::Cultist, EncounterId::JawWorm, EncounterId::TwoLouse,
    EncounterId::LargeSlime, EncounterId::ThreeLouse, EncounterId::LotsOfSlimes,
    EncounterId::GremlinNob, EncounterId::Lagavulin, EncounterId::ThreeSentries,
    EncounterId::Guardian, EncounterId::Hexaghost, EncounterId::SlimeBoss,
    EncounterId::Champ, EncounterId::BronzeAutomaton, EncounterId::Collector,
];
pub(crate) const M12_PLAYABLE_ENEMIES: &[EnemyId] = &[
    EnemyId::Cultist, EnemyId::JawWorm, EnemyId::GremlinNob, EnemyId::Lagavulin,
    EnemyId::Sentry, EnemyId::Guardian, EnemyId::Hexaghost, EnemyId::LouseRed,
    EnemyId::LouseGreen, EnemyId::AcidSlimeS, EnemyId::AcidSlimeM,
    EnemyId::AcidSlimeL, EnemyId::SpikeSlimeS, EnemyId::SpikeSlimeM,
    EnemyId::SpikeSlimeL, EnemyId::SlimeBoss,
];

fn cards() -> impl Iterator<Item = CardId> {
    M12_SILENT_ACT1_CARDS.iter().copied().chain(GENERATED_CARDS.iter().copied())
}

fn padded(values: impl Iterator<Item = &'static str>) -> Vec<&'static str> {
    std::iter::once("padding").chain(values).collect()
}

pub(crate) fn groups(c: &RunCapacities) -> Vec<v1::GroupSpec> {
    m10::groups(c).into_iter().filter(|group| group.key != "combat_state").collect()
}

pub(crate) fn validate(c: &RunCapacities) -> Result<(), RunSessionError> {
    m10::validate_groups(&groups(c), c)
}

pub fn m12_run_observation_spec(c: &RunCapacities) -> Result<RunObservationSpec, RunSessionError> {
    validate(c)?;
    let vocab = vocabularies();
    m10::spec_from_vocabulary(
        c,
        RUN_OBSERVATION_M12_SCHEMA_ID,
        "M12SilentAct1",
        vocab,
        json!({
            "padding":"zero,known0,occupied0","unknown_reference":"minus_one,known0 in occupied rows",
            "counts":"only published public counts; no hidden draw rows","identity":"finite M12 content IDs and row references only; no raw resolver IDs",
            "power_absence":"no row; listed signed amounts are public","potions":"belt slot rows; empty slot known1/occupied0/kind0; unused transport padding known0",
            "selection":"Run selections expose counts without selected identities; entitled Nightmare and retain combat selections expose selected card rows and operations",
            "known_draw":"only entitled known cards; references point to corresponding combat_cards rows; unknown draw cards have no rows",
            "playing_cards":"accepted public play owners use a dedicated playing zone",
            "actions":"legal candidate order; no paging, truncation or policy",
            "scalar0":"potion belt slot for potion verbs; otherwise absent","scalar1":"reserved absent",
            "offer_amount":"Neow HP/gold/heal/Joust result/Falling type/stage per choice_kind",
            "encounters":{
                "playable":M12_PLAYABLE_ENCOUNTERS.iter().map(|id|id.as_str()).collect::<Vec<_>>(),
                "map_only_boss_icons":M12_MAP_ONLY_BOSS_ICONS.iter().map(|id|id.as_str()).collect::<Vec<_>>()
            },
            "cutoff":"Act2Ready is a nonterminal zero-action boundary after actual Act1 settlement; it is not victory and receives no completion reward"
        }),
        groups(c),
        b"spire-gym/run-schema/m12-silent-act1/v1\0",
    )
}

fn append<T: PartialEq>(id: T, values: &[T], message: &'static str) -> Result<i64, RunSessionError> {
    values.iter().position(|value| *value == id)
        .map(|index| index as i64 + 1)
        .ok_or(RunSessionError::Input(message))
}

pub(crate) fn card(id: CardId) -> Result<i64, RunSessionError> {
    append(id, &cards().collect::<Vec<_>>(), "unsupported M12 CardId")
}
pub(crate) fn enemy(id: EnemyId) -> Result<i64, RunSessionError> {
    append(id, M12_PLAYABLE_ENEMIES, "unsupported M12 EnemyId")
}
pub(crate) fn encounter(id: EncounterId) -> Result<i64, RunSessionError> {
    append(id, M12_ENCOUNTERS, "unsupported M12 EncounterId")
}
pub(crate) fn relic(id: RelicId) -> Result<i64, RunSessionError> {
    append(id, M12_SILENT_ACT1_RELICS, "unsupported M12 RelicId")
}
pub(crate) fn potion(id: PotionId) -> Result<i64, RunSessionError> {
    append(id, M12_SILENT_ACT1_POTIONS, "unsupported M12 PotionId")
}
pub(crate) fn event(id: EventId) -> Result<i64, RunSessionError> {
    match id {
        EventId::Neow | EventId::UpgradeShrine | EventId::Purifier => v1::event(id),
        _ => Err(RunSessionError::Input("unsupported M12 EventId")),
    }
}
pub(crate) fn profile(profile: RoomScenario) -> Result<i64, RunSessionError> {
    if profile == RoomScenario::M12SilentAct1 { Ok(1) }
    else { Err(RunSessionError::Input("profile is not representable in the M12 schema")) }
}
pub(crate) fn room(kind: RoomViewKind) -> Result<i64, RunSessionError> {
    match kind {
        RoomViewKind::Neow | RoomViewKind::Map | RoomViewKind::Combat | RoomViewKind::Rewards |
        RoomViewKind::CardReward | RoomViewKind::Shop | RoomViewKind::Campfire | RoomViewKind::Event(_) |
        RoomViewKind::Treasure | RoomViewKind::CheckpointComplete | RoomViewKind::BossTreasure |
        RoomViewKind::BossRelicChoice | RoomViewKind::BossRelicChild | RoomViewKind::ProceedWithoutRelic |
        RoomViewKind::Act2Ready => v1::room(kind),
        _ => Err(RunSessionError::UnsupportedBoundary("M12 observation cannot encode a room after the Act2Ready cutoff")),
    }
}
pub(crate) fn intent(name: &str) -> Result<i64, RunSessionError> { v1::intent(name) }
pub(crate) fn power(name: &str) -> Result<i64, RunSessionError> {
    if let Ok(value) = v1::named(v1::POWERS, name) { return Ok(value); }
    let base_len = v1::POWERS.len();
    M12_POWERS.iter().position(|value| *value == name)
        .map(|index| (base_len + index) as i64)
        .ok_or(RunSessionError::Input("unsupported M12 public power"))
}
pub(crate) fn choice(choice: &RoomChoiceKind) -> Result<i64, RunSessionError> {
    if matches!(choice, RoomChoiceKind::Act3Continue { .. }) {
        return Err(RunSessionError::Input("unsupported M12 RoomChoiceKind"));
    }
    Ok(v1::choice(choice))
}
pub(crate) fn zone(name: &str) -> Result<i64, RunSessionError> { m10::zone(name) }
pub(crate) fn operation(selection: &spire_core::combat::SelectionView) -> Result<i64, RunSessionError> {
    use spire_core::combat::{SelectionDisposition::*, SelectionOperation::*};
    let name = match selection.operation {
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
        CopyToHand { .. } => return Err(RunSessionError::Input("unsupported M12 selection copy count")),
        NightmareCapture => "nightmare_capture",
        Retain => "retain",
    };
    M12_SELECTION_OPERATIONS.iter().position(|value| *value == name)
        .map(|index| index as i64 + 1)
        .ok_or(RunSessionError::Input("unsupported M12 selection operation"))
}
pub(crate) fn select_permanent_operation() -> i64 {
    M12_SELECTION_OPERATIONS.iter().position(|value| *value == "select_permanent")
        .expect("declared M12 permanent selection operation") as i64 + 1
}

fn vocabularies() -> Value {
    let mut vocab = m10::vocabularies();
    vocab["card"] = json!(padded(cards().map(CardId::as_str)));
    vocab["enemy"] = json!(padded(M12_PLAYABLE_ENEMIES.iter().map(|id| id.as_str())));
    vocab["encounter"] = json!(padded(M12_ENCOUNTERS.iter().map(|id| id.as_str())));
    vocab["relic"] = json!(padded(M12_SILENT_ACT1_RELICS.iter().map(|id| id.as_str())));
    vocab["potion"] = json!(padded(M12_SILENT_ACT1_POTIONS.iter().map(|id| id.as_str())));
    vocab["event"] = json!(padded([EventId::Neow, EventId::UpgradeShrine, EventId::Purifier].into_iter().map(EventId::as_str)));
    vocab["profile"] = json!(["padding", "M12SilentAct1"]);
    vocab["room"] = json!(v1::ROOMS[..=15].to_vec());
    vocab["choice"] = json!(v1::CHOICES[..31].to_vec());
    vocab["terminal"] = json!(["ongoing", "death", "empty_deck"]);
    vocab["intent"] = json!(v1::INTENTS.iter().chain(v1::ACT1_INTENTS).copied().collect::<Vec<_>>());
    // The M12 encoder refuses Shield/Spear/Heart even though those names remain
    // in the shared M10 owner. M12's table contains the shared power vocabulary
    // plus the three actual Silent card powers.
    vocab["power"] = json!(v1::POWERS.iter().copied().chain(M12_POWERS.iter().copied().filter(|name|
        !v1::POWERS.contains(name))).collect::<Vec<_>>());
    vocab["zone"] = json!(["padding", "hand", "discard", "exhaust", "played_power", "stasis", "selection_only",
        "known_draw_top", "known_draw_bottom", "playing"]);
    vocab["selection_operation"] = json!(std::iter::once("padding").chain(M12_SELECTION_OPERATIONS.iter().copied()).collect::<Vec<_>>());
    vocab["row_group"] = json!(["none", "player", "map_nodes", "permanent_cards", "combat_cards", "enemies", "potions", "offers", "selection", "known_draw"]);
    vocab["verb"] = json!(["padding", "move", "choose", "select", "confirm", "cancel", "use_potion", "discard_potion",
        "play_card", "combat_select_card", "end_turn", "confirm_selection"]);
    vocab["encounter_side"] = json!(["padding", "left", "right"]);
    vocab["facing"] = json!(["padding", "left", "right"]);
    vocab["card_cost_kind"] = json!(["padding", "fixed", "x", "unplayable"]);
    vocab
}
