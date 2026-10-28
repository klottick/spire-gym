//! Finite Defect Act1 vocabulary on the shared M16 Run projection groups.
use crate::{
    run_schema as v1, run_schema_m10 as m10, run_schema_m12 as m12, run_schema_m16 as m16,
    RunCapacities, RunObservationSpec, RunSessionError,
};
use serde_json::{json, Value};
use spire_core::{combat::Outcome, content::*, run::*};

pub const RUN_OBSERVATION_M17_SCHEMA_ID: &str =
    "spire-gym-m17-defect-act1-observation-v1";

const GENERATED_STATUSES: &[CardId] = &[CardId::Dazed, CardId::Burn, CardId::Slimed];
const M17_EVENTS: &[EventId] = &[EventId::Neow, EventId::UpgradeShrine, EventId::Purifier];

fn cards() -> impl Iterator<Item = CardId> {
    M17_DEFECT_ACT1_CARDS
        .iter()
        .copied()
        .chain(GENERATED_STATUSES.iter().copied())
}

fn padded(values: impl Iterator<Item = &'static str>) -> Vec<&'static str> {
    std::iter::once("padding").chain(values).collect()
}

pub(crate) fn groups(c: &RunCapacities) -> Vec<v1::GroupSpec> {
    m16::groups(c)
}

pub(crate) fn validate(c: &RunCapacities) -> Result<(), RunSessionError> {
    m16::validate(c)
}

fn descriptor_vocabularies(spec: RunObservationSpec) -> Result<Value, RunSessionError> {
    spec.descriptor()
        .get("vocabularies")
        .cloned()
        .ok_or(RunSessionError::Input("shared Run descriptor has no vocabularies"))
}

fn vocabularies(c: &RunCapacities) -> Result<Value, RunSessionError> {
    let mut vocab = descriptor_vocabularies(m12::m12_run_observation_spec(c)?)?;
    let m16_vocab = descriptor_vocabularies(m16::defect_combat_observation_spec(c)?)?;

    vocab["card"] = json!(padded(cards().map(CardId::as_str)));
    vocab["enemy"] = json!(padded(m12::M12_PLAYABLE_ENEMIES.iter().map(|id|id.as_str())));
    vocab["encounter"] = json!(padded(
        m12::M12_PLAYABLE_ENCOUNTERS
            .iter()
            .chain(m12::M12_MAP_ONLY_BOSS_ICONS.iter())
            .map(|id|id.as_str())
    ));
    vocab["relic"] = json!(padded(M17_DEFECT_ACT1_RELICS.iter().map(|id|id.as_str())));
    vocab["potion"] = json!(padded(M17_DEFECT_ACT1_POTIONS.iter().map(|id|id.as_str())));
    vocab["event"] = json!(padded(M17_EVENTS.iter().map(|id|id.as_str())));
    vocab["profile"] = json!(["padding", "M17DefectAct1"]);

    let powers = m16_vocab["power"]
        .as_array()
        .ok_or(RunSessionError::Input("M16 descriptor has no power vocabulary"))?;
    vocab["power"] = Value::Array(
        powers
            .iter()
            .filter(|power| power.as_str() != Some("echo_form"))
            .cloned()
            .collect(),
    );
    vocab["orb"] = m16_vocab
        .get("orb")
        .cloned()
        .ok_or(RunSessionError::Input("M16 descriptor has no orb vocabulary"))?;
    vocab["selection_operation"] = m16_vocab
        .get("selection_operation")
        .cloned()
        .ok_or(RunSessionError::Input("M16 descriptor has no selection vocabulary"))?;
    Ok(vocab)
}

pub fn m17_run_observation_spec(
    c: &RunCapacities,
) -> Result<RunObservationSpec, RunSessionError> {
    validate(c)?;
    let vocab = vocabularies(c)?;
    m10::spec_from_vocabulary(
        c,
        RUN_OBSERVATION_M17_SCHEMA_ID,
        "M17DefectAct1",
        vocab,
        json!({
            "padding":"zero,known0,occupied0",
            "unknown_reference":"minus_one,known0 in occupied rows",
            "counts":"only published public counts; no hidden draw rows",
            "identity":"finite M17 content IDs and row references only; no raw resolver IDs",
            "power_absence":"no row; listed signed amounts are public",
            "cards":{
                "permanent":M17_DEFECT_ACT1_CARDS.iter().map(|id|id.as_str()).collect::<Vec<_>>(),
                "generated_statuses":GENERATED_STATUSES.iter().map(|id|id.as_str()).collect::<Vec<_>>(),
                "burn_plus":"Burn uses the Burn card ID and its published upgrade flag"
            },
            "relics":M17_DEFECT_ACT1_RELICS.iter().map(|id|id.as_str()).collect::<Vec<_>>(),
            "potions":M17_DEFECT_ACT1_POTIONS.iter().map(|id|id.as_str()).collect::<Vec<_>>(),
            "encounters":{
                "playable":m12::M12_PLAYABLE_ENCOUNTERS.iter().map(|id|id.as_str()).collect::<Vec<_>>(),
                "playable_actors":m12::M12_PLAYABLE_ENEMIES.iter().map(|id|id.as_str()).collect::<Vec<_>>(),
                "map_only_boss_icons":m12::M12_MAP_ONLY_BOSS_ICONS.iter().map(|id|id.as_str()).collect::<Vec<_>>()
            },
            "powers":"M16 Defect public projection vocabulary, excluding echo_form",
            "orbs":"M16 empty/lightning/frost/dark/plasma IDs; known empty slots differ from padding and Dark damage is known only for Dark",
            "selection":"Run selections expose counts without selected identities; entitled combat selections expose selected card rows and operations",
            "actions":"legal candidate order; no paging, truncation or policy",
            "terminal":"only ongoing, death and empty_deck are admitted",
            "cutoff":"Act2Ready is a nonterminal zero-action boundary after actual Act1 settlement; it is not victory and receives no completion reward",
            "scope":"finite Defect A0 Act1 Run; foreign profile/content and post-cutoff terminals are rejected"
        }),
        groups(c),
        b"spire-gym/run-schema/m17-defect-act1/v1\0",
    )
}

fn append<T: PartialEq>(
    id: T,
    values: &[T],
    message: &'static str,
) -> Result<i64, RunSessionError> {
    values
        .iter()
        .position(|value| *value == id)
        .map(|index| index as i64 + 1)
        .ok_or(RunSessionError::Input(message))
}

pub(crate) fn card(id: CardId) -> Result<i64, RunSessionError> {
    append(
        id,
        &cards().collect::<Vec<_>>(),
        "unsupported M17 CardId",
    )
}

pub(crate) fn enemy(id: EnemyId) -> Result<i64, RunSessionError> {
    append(id, m12::M12_PLAYABLE_ENEMIES, "unsupported M17 EnemyId")
}

pub(crate) fn encounter(id: EncounterId) -> Result<i64, RunSessionError> {
    let encounters = m12::M12_PLAYABLE_ENCOUNTERS
        .iter()
        .chain(m12::M12_MAP_ONLY_BOSS_ICONS.iter())
        .copied()
        .collect::<Vec<_>>();
    append(id, &encounters, "unsupported M17 EncounterId")
}

pub(crate) fn relic(id: RelicId) -> Result<i64, RunSessionError> {
    append(id, M17_DEFECT_ACT1_RELICS, "unsupported M17 RelicId")
}

pub(crate) fn potion(id: PotionId) -> Result<i64, RunSessionError> {
    append(id, M17_DEFECT_ACT1_POTIONS, "unsupported M17 PotionId")
}

pub(crate) fn event(id: EventId) -> Result<i64, RunSessionError> {
    if M17_EVENTS.contains(&id) {
        m12::event(id)
    } else {
        Err(RunSessionError::Input("unsupported M17 EventId"))
    }
}

pub(crate) fn profile(profile: RoomScenario) -> Result<i64, RunSessionError> {
    if profile == RoomScenario::M17DefectAct1 {
        Ok(1)
    } else {
        Err(RunSessionError::Input(
            "profile is not representable in the M17 schema",
        ))
    }
}

pub(crate) fn room(kind: RoomViewKind) -> Result<i64, RunSessionError> {
    m12::room(kind)
}

pub(crate) fn terminal(value: Option<RunTerminal>) -> Result<i64, RunSessionError> {
    match value {
        None => Ok(0),
        Some(RunTerminal::Death) => Ok(1),
        Some(RunTerminal::EmptyDeck) => Ok(2),
        Some(_) => Err(RunSessionError::UnsupportedBoundary(
            "M17 cannot encode a terminal outside its Act1 profile",
        )),
    }
}

pub(crate) fn outcome(value: Option<Outcome>) -> Result<i64, RunSessionError> {
    match value {
        None => Ok(0),
        Some(Outcome::Victory) => Ok(1),
        Some(Outcome::Death) => Ok(2),
        Some(Outcome::Escaped) => Err(RunSessionError::UnsupportedBoundary(
            "M17 cannot encode an escaped combat outcome",
        )),
    }
}

pub(crate) fn validate_public_decision(
    decision: &PublicDecision,
) -> Result<(), RunSessionError> {
    if decision.content_profile != Some(RoomScenario::M17DefectAct1)
        || decision.player.character != Character::Defect
    {
        return Err(RunSessionError::Input(
            "M17 encoder requires the Defect Act1 profile",
        ));
    }
    for card in &decision.objects {
        if !M17_DEFECT_ACT1_CARDS.contains(&card.id)||card.upgrades>1 {
            return Err(RunSessionError::Input("unsupported M17 permanent card"));
        }
    }
    terminal(decision.terminal)?;
    if let Some(combat) = &decision.combat {
        outcome(combat.outcome)?;
    }
    if let Some(last_combat) = &decision.last_combat {
        if !m12::M12_PLAYABLE_ENCOUNTERS.contains(&last_combat.encounter) {
            return Err(RunSessionError::Input("M17 map-only icon is not a played encounter"));
        }
        outcome(Some(last_combat.outcome))?;
    }
    Ok(())
}

pub(crate) fn intent(name: &str) -> Result<i64, RunSessionError> {
    m12::intent(name)
}

pub(crate) fn power(is_player: bool, name: &str) -> Result<i64, RunSessionError> {
    if name == "echo_form" {
        return Err(RunSessionError::Input("unsupported M17 public power"));
    }
    m16::power(is_player, name)
}

pub(crate) fn choice(choice: &RoomChoiceKind) -> Result<i64, RunSessionError> {
    if matches!(choice,RoomChoiceKind::Card{id,upgrades} if !M17_DEFECT_ACT1_CARDS.contains(id)||*upgrades>1){
        return Err(RunSessionError::Input("unsupported M17 permanent card offer"));
    }
    m12::choice(choice)
}

pub(crate) fn zone(name: &str) -> Result<i64, RunSessionError> {
    m12::zone(name)
}

pub(crate) fn operation(
    selection: &spire_core::combat::SelectionView,
) -> Result<i64, RunSessionError> {
    use spire_core::combat::SelectionOperation::{NightmareCapture, Retain};
    if matches!(selection.operation, NightmareCapture | Retain) {
        return Err(RunSessionError::Input(
            "unsupported M17 public selection operation",
        ));
    }
    m12::operation(selection)
}

pub(crate) fn select_permanent_operation() -> i64 {
    m12::select_permanent_operation()
}
