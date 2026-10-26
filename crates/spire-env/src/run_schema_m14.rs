//! Finite Silent Acts 1–3 vocabulary on the shared Run observation owner.
use crate::{
    run_schema as v1, run_schema_m10 as m10,
    run_schema_m13 as m13, RunCapacities, RunObservationSpec, RunSessionError,
};
use serde_json::{json, Value};
use spire_core::{content::*, run::*};

pub const RUN_OBSERVATION_M14_SCHEMA_ID: &str =
    "spire-gym-m14-silent-act3-observation-v1";

pub(crate) const M14_ACT3_ENCOUNTERS: &[EncounterId] = &[
    EncounterId::OrbWalker,
    EncounterId::ThreeShapes,
    EncounterId::SpireGrowth,
    EncounterId::Transient,
    EncounterId::FourShapes,
    EncounterId::Maw,
    EncounterId::SphereAndTwoShapes,
    EncounterId::JawWormHorde,
    EncounterId::ThreeDarklings,
    EncounterId::WrithingMass,
    EncounterId::GiantHead,
    EncounterId::Nemesis,
    EncounterId::Reptomancer,
    EncounterId::AwakenedOne,
    EncounterId::TimeEater,
    EncounterId::DonuAndDeca,
];

pub(crate) const M14_ACT3_ENEMIES: &[EnemyId] = &[
    EnemyId::SphericGuardian,
    EnemyId::JawWorm,
    EnemyId::Cultist,
    EnemyId::OrbWalker,
    EnemyId::Repulsor,
    EnemyId::Exploder,
    EnemyId::Spiker,
    EnemyId::Darkling,
    EnemyId::SpireGrowth,
    EnemyId::Transient,
    EnemyId::Maw,
    EnemyId::WrithingMass,
    EnemyId::GiantHead,
    EnemyId::Nemesis,
    EnemyId::Reptomancer,
    EnemyId::Dagger,
    EnemyId::AwakenedOne,
    EnemyId::TimeEater,
    EnemyId::Deca,
    EnemyId::Donu,
];

const M14_NEW_ACT3_ENEMIES: &[EnemyId] = &[
    EnemyId::OrbWalker,
    EnemyId::Repulsor,
    EnemyId::Exploder,
    EnemyId::Spiker,
    EnemyId::Darkling,
    EnemyId::SpireGrowth,
    EnemyId::Transient,
    EnemyId::Maw,
    EnemyId::WrithingMass,
    EnemyId::GiantHead,
    EnemyId::Nemesis,
    EnemyId::Reptomancer,
    EnemyId::Dagger,
    EnemyId::AwakenedOne,
    EnemyId::TimeEater,
    EnemyId::Deca,
    EnemyId::Donu,
];

const M14_NEW_ACT3_ENCOUNTERS: &[EncounterId] = &[
    EncounterId::OrbWalker,
    EncounterId::ThreeShapes,
    EncounterId::SpireGrowth,
    EncounterId::Transient,
    EncounterId::FourShapes,
    EncounterId::Maw,
    EncounterId::SphereAndTwoShapes,
    EncounterId::JawWormHorde,
    EncounterId::ThreeDarklings,
    EncounterId::WrithingMass,
    EncounterId::GiantHead,
    EncounterId::Nemesis,
    EncounterId::Reptomancer,
];

const M14_NEW_RELICS: &[RelicId] = &[
    RelicId::Vajra,
    RelicId::OddlySmoothStone,
    RelicId::BagOfMarbles,
    RelicId::BronzeScales,
    RelicId::RedMask,
];

const M14_NEW_CARDS: &[CardId] = &[CardId::Parasite, CardId::Void];
const M14_NEW_EVENTS: &[EventId] = &[EventId::Falling, EventId::TombOfLordRedMask];

pub(crate) fn groups(c: &RunCapacities) -> Vec<v1::GroupSpec> {
    m13::groups(c)
}

pub(crate) fn validate(c: &RunCapacities) -> Result<(), RunSessionError> {
    m13::validate(c)
}

fn append_names(vocab: &mut Value, key: &str, names: impl IntoIterator<Item = &'static str>) {
    let rows = vocab[key].as_array_mut().expect("M13 schema vocabulary array");
    for name in names {
        let value = Value::String(name.to_owned());
        if !rows.contains(&value) {
            rows.push(value);
        }
    }
}

fn vocabularies() -> Value {
    let mut vocab = m13::vocabularies();
    append_names(&mut vocab, "card", M14_NEW_CARDS.iter().map(|id| id.as_str()));
    append_names(&mut vocab, "enemy", M14_NEW_ACT3_ENEMIES.iter().map(|id| id.as_str()));
    append_names(&mut vocab, "encounter", M14_NEW_ACT3_ENCOUNTERS.iter().map(|id| id.as_str()));
    append_names(&mut vocab, "relic", M14_NEW_RELICS.iter().map(|id| id.as_str()));
    append_names(&mut vocab, "event", M14_NEW_EVENTS.iter().map(|id| id.as_str()));

    // M14 admits the actual Act3 ending choice and a supplied-only healthy
    // Act4Ready diagnostic. It still has no Act4 gameplay or Heart victory.
    vocab["choice"] = json!(v1::CHOICES);
    vocab["room"] = json!(v1::ROOMS[..=21]);
    vocab["profile"] = json!(["padding", "M14SilentAct3"]);
    vocab["terminal"] = json!(["ongoing", "death", "empty_deck", "act3_no_keys"]);
    vocab
}

pub fn m14_run_observation_spec(
    c: &RunCapacities,
) -> Result<RunObservationSpec, RunSessionError> {
    validate(c)?;
    let vocab = vocabularies();
    let mut playable_encounters = m13::M13_PLAYABLE_ENCOUNTERS.to_vec();
    playable_encounters.extend(M14_ACT3_ENCOUNTERS.iter().copied());
    let mut cumulative_enemies = m13::M13_PLAYABLE_ENEMIES.to_vec();
    cumulative_enemies.extend(M14_NEW_ACT3_ENEMIES.iter().copied());
    let semantics = json!({
        "padding":"zero,known0,occupied0",
        "unknown_reference":"minus_one,known0 in occupied rows",
        "counts":"only published public counts; no hidden draw rows",
        "identity":"finite M14 content IDs and row references only; no raw resolver IDs",
        "power_absence":"no row; listed signed amounts are public",
        "potions":"belt slot rows; empty slot known1/occupied0/kind0; unused transport padding known0",
        "selection":"Run selections expose counts without selected identities; entitled Nightmare and retain combat selections expose selected card rows and operations",
        "known_draw":"only entitled known cards; references point to corresponding combat_cards rows; unknown draw cards have no rows",
        "playing_cards":"accepted public play owners use a dedicated playing zone",
        "actions":"legal candidate order; no paging, truncation or policy",
        "scalar0":"potion belt slot for potion verbs; otherwise absent",
        "scalar1":"reserved absent",
        "offer_amount":"Neow HP/gold/heal/Joust result/Falling type/stage per choice_kind",
        "entitlements":{
            "permanent_cards":spire_core::run::M14_SILENT_ACT3_CARDS.iter().map(|id|id.as_str()).collect::<Vec<_>>(),
            "generated_and_status_cards":["shiv","dazed","burn","slimed","wound","void"],
            "parasite":"permanent curse from actual Writhing Mass custody; not an ordinary reward or shop card",
            "relics":spire_core::run::M14_SILENT_ACT3_RELICS.iter().map(|id|id.as_str()).collect::<Vec<_>>(),
            "potions":spire_core::run::M14_SILENT_ACT3_POTIONS.iter().map(|id|id.as_str()).collect::<Vec<_>>()
        },
        "encounters":{
            "playable":playable_encounters.iter().map(|id|id.as_str()).collect::<Vec<_>>(),
            "map_only_boss_icons":[],
            "act3":M14_ACT3_ENCOUNTERS.iter().map(|id|id.as_str()).collect::<Vec<_>>()
        },
        "act3_enemy_species":{
            "species":M14_ACT3_ENEMIES.iter().map(|id|id.as_str()).collect::<Vec<_>>(),
            "new_to_m13":M14_NEW_ACT3_ENEMIES.iter().map(|id|id.as_str()).collect::<Vec<_>>(),
            "cumulative_through_act3":cumulative_enemies.iter().map(|id|id.as_str()).collect::<Vec<_>>()
        },
        "sludge":"awa_sludge creates Void at BottomDraw; there is no Sludge CardId",
        "endings":{
            "act3_no_keys":"actual Act3 boss Victory and shared Proceed/dialog completion; no Heart reward",
            "act4_ready":"supplied diagnostic only; healthy nonterminal zero-action cutoff before the Act4 door"
        }
    });
    m10::spec_from_vocabulary(
        c,
        RUN_OBSERVATION_M14_SCHEMA_ID,
        "M14SilentAct3",
        vocab,
        semantics,
        groups(c),
        b"spire-gym/run-schema/m14-silent-act3/v1\0",
    )
}

pub(crate) fn card(id: CardId) -> Result<i64, RunSessionError> {
    if let Ok(row) = m13::card(id) {
        return Ok(row);
    }
    M14_NEW_CARDS.iter().position(|value| *value == id)
        .map(|index| (M12_SILENT_ACT1_CARDS.len() + 5 + index + 1) as i64)
        .ok_or(RunSessionError::Input("unsupported M14 CardId"))
}

pub(crate) fn enemy(id: EnemyId) -> Result<i64, RunSessionError> {
    if let Ok(row) = m13::enemy(id) {
        return Ok(row);
    }
    M14_NEW_ACT3_ENEMIES.iter().position(|value| *value == id)
        .map(|index| (m13::M13_PLAYABLE_ENEMIES.len() + index + 1) as i64)
        .ok_or(RunSessionError::Input("unsupported M14 EnemyId"))
}

pub(crate) fn encounter(id: EncounterId) -> Result<i64, RunSessionError> {
    if let Ok(row) = m13::encounter(id) {
        return Ok(row);
    }
    M14_NEW_ACT3_ENCOUNTERS.iter().position(|value| *value == id)
        .map(|index| (m13::M13_ENCOUNTERS.len() + index + 1) as i64)
        .ok_or(RunSessionError::Input("unsupported M14 EncounterId"))
}

pub(crate) fn relic(id: RelicId) -> Result<i64, RunSessionError> {
    if let Ok(row) = m13::relic(id) {
        return Ok(row);
    }
    M14_NEW_RELICS.iter().position(|value| *value == id)
        .map(|index| (M12_SILENT_ACT1_RELICS.len() + index + 1) as i64)
        .ok_or(RunSessionError::Input("unsupported M14 RelicId"))
}

pub(crate) fn potion(id: PotionId) -> Result<i64, RunSessionError> {
    m13::potion(id)
}

pub(crate) fn event(id: EventId) -> Result<i64, RunSessionError> {
    match id {
        EventId::Falling | EventId::TombOfLordRedMask => v1::event(id),
        _ => m13::event(id),
    }
}

pub(crate) fn profile(profile: RoomScenario) -> Result<i64, RunSessionError> {
    if profile == RoomScenario::M14SilentAct3 {
        Ok(1)
    } else {
        Err(RunSessionError::Input("profile is not representable in the M14 schema"))
    }
}

pub(crate) fn room(kind: RoomViewKind) -> Result<i64, RunSessionError> {
    v1::room(kind)
}

pub(crate) fn intent(name: &str) -> Result<i64, RunSessionError> {
    m13::intent(name)
}

pub(crate) fn power(name: &str) -> Result<i64, RunSessionError> {
    m13::power(name)
}

pub(crate) fn choice(choice: &RoomChoiceKind) -> Result<i64, RunSessionError> {
    if matches!(choice, RoomChoiceKind::Act3Continue { .. }) {
        return Ok(v1::choice(choice));
    }
    m13::choice(choice)
}

pub(crate) fn zone(name: &str) -> Result<i64, RunSessionError> {
    m13::zone(name)
}

pub(crate) fn operation(selection: &spire_core::combat::SelectionView) -> Result<i64, RunSessionError> {
    m13::operation(selection)
}

pub(crate) fn select_permanent_operation() -> i64 {
    m13::select_permanent_operation()
}
