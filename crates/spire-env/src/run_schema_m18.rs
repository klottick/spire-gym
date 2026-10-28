//! Finite Defect Acts 1–2 vocabulary, composed from the shared Run and M16 owners.
use crate::{
    run_schema as v1, run_schema_m10 as m10, run_schema_m13 as act2,
    run_schema_m16 as defect, RunCapacities, RunObservationSpec, RunSessionError,
};
use serde_json::{json, Value};
use spire_core::{combat::{Outcome, SelectionOperation}, content::*, run::*};

pub const RUN_OBSERVATION_M18_SCHEMA_ID: &str = "spire-gym-m18-defect-act2-observation-v1";
const STATUSES: &[CardId] = &[CardId::Dazed, CardId::Burn, CardId::Slimed, CardId::Wound];

pub(crate) fn groups(c: &RunCapacities) -> Vec<v1::GroupSpec> { defect::groups(c) }
pub(crate) fn validate(c: &RunCapacities) -> Result<(), RunSessionError> { defect::validate(c) }
fn padded(values: impl Iterator<Item = &'static str>) -> Vec<&'static str> {
    std::iter::once("padding").chain(values).collect()
}
fn vocabulary(c: &RunCapacities) -> Result<Value, RunSessionError> {
    // Reuse room/map/actor/intent vocabulary. Character admission and dynamic
    // groups come from Defect, independently of the Silent profile's cards.
    let mut vocab = act2::vocabularies();
    let defect_vocab = defect::defect_combat_observation_spec(c)?.descriptor()["vocabularies"].clone();
    vocab["card"] = json!(padded(M18_DEFECT_ACT2_CARDS.iter().chain(STATUSES).map(|id|id.as_str())));
    vocab["relic"] = json!(padded(M18_DEFECT_ACT2_RELICS.iter().map(|id|id.as_str())));
    vocab["potion"] = json!(padded(M18_DEFECT_ACT2_POTIONS.iter().map(|id|id.as_str())));
    vocab["profile"] = json!(["padding", "M18DefectAct2"]);
    for key in ["power", "orb", "selection_operation"] {
        vocab[key] = defect_vocab.get(key).cloned().ok_or(RunSessionError::Input("missing shared Defect vocabulary"))?;
    }
    // Keep the accepted Defect indices and append only shared Act2 additions.
    let act2_powers = act2::vocabularies()["power"].as_array().cloned()
        .ok_or(RunSessionError::Input("missing shared Act2 powers"))?;
    let powers = vocab["power"].as_array_mut().ok_or(RunSessionError::Input("missing shared Defect powers"))?;
    for power in act2_powers {
        if !matches!(power.as_str(), Some("noxious_fumes"|"accuracy"|"retain_cards")) && !powers.contains(&power) {
            powers.push(power);
        }
    }
    Ok(vocab)
}
pub fn m18_run_observation_spec(c: &RunCapacities) -> Result<RunObservationSpec, RunSessionError> {
    validate(c)?;
    m10::spec_from_vocabulary(c, RUN_OBSERVATION_M18_SCHEMA_ID, "M18DefectAct2", vocabulary(c)?, json!({
        "padding":"zero,known0,occupied0",
        "unknown_reference":"minus_one,known0 in occupied rows",
        "counts":"only published public counts; no hidden draw rows",
        "identity":"finite content IDs and public row references; no raw resolver identities",
        "cards":{
            "permanent":M18_DEFECT_ACT2_CARDS.iter().map(|id|id.as_str()).collect::<Vec<_>>(),
            "generated_statuses":STATUSES.iter().map(|id|id.as_str()).collect::<Vec<_>>(),
            "burn_plus":"Burn uses the Burn card ID and its published upgrade flag"
        },
        "relics":M18_DEFECT_ACT2_RELICS.iter().map(|id|id.as_str()).collect::<Vec<_>>(),
        "potions":M18_DEFECT_ACT2_POTIONS.iter().map(|id|id.as_str()).collect::<Vec<_>>(),
        "encounters":{
            "playable":act2::M13_PLAYABLE_ENCOUNTERS.iter().map(|id|id.as_str()).collect::<Vec<_>>(),
            "playable_actors":act2::M13_PLAYABLE_ENEMIES.iter().map(|id|id.as_str()).collect::<Vec<_>>(),
            "map_only_boss_icons":act2::M13_MAP_ONLY_BOSS_ICONS.iter().map(|id|id.as_str()).collect::<Vec<_>>()
        },
        "orbs":"M16 logical slots through ten; known empty differs from padding; signed Dark damage is known only for Dark",
        "selection":"public counts and entitled combat card rows, preserving legal candidate order",
        "actions":"row i is PublicDecision.candidates[i]; no filtering, truncation or paging",
        "terminal":"only ongoing, death and empty_deck",
        "cutoff":"healthy Act3Ready at act3/floor34, nonterminal with zero actions and no readiness reward",
        "scope":"finite Defect A0 normal Acts1–2; native source only"
    }), groups(c), b"spire-gym/run-schema/m18-defect-act2/v1\0")
}
fn index<T: PartialEq>(id: T, values: &[T], error: &'static str) -> Result<i64, RunSessionError> {
    values.iter().position(|value|*value==id).map(|i|i as i64+1).ok_or(RunSessionError::Input(error))
}
pub(crate) fn card(id: CardId) -> Result<i64, RunSessionError> {
    let cards=M18_DEFECT_ACT2_CARDS.iter().chain(STATUSES).copied().collect::<Vec<_>>();
    index(id,&cards,"unsupported M18 CardId")
}
pub(crate) fn relic(id: RelicId) -> Result<i64, RunSessionError> { index(id,M18_DEFECT_ACT2_RELICS,"unsupported M18 RelicId") }
pub(crate) fn potion(id: PotionId) -> Result<i64, RunSessionError> { index(id,M18_DEFECT_ACT2_POTIONS,"unsupported M18 PotionId") }
pub(crate) fn enemy(id: EnemyId) -> Result<i64, RunSessionError> { act2::enemy(id) }
pub(crate) fn encounter(id: EncounterId) -> Result<i64, RunSessionError> { act2::encounter(id) }
pub(crate) fn event(id: EventId) -> Result<i64, RunSessionError> { act2::event(id) }
pub(crate) fn room(kind: RoomViewKind) -> Result<i64, RunSessionError> { act2::room(kind) }
pub(crate) fn intent(name: &str) -> Result<i64, RunSessionError> { act2::intent(name) }
pub(crate) fn zone(name: &str) -> Result<i64, RunSessionError> { act2::zone(name) }
pub(crate) fn profile(value: RoomScenario) -> Result<i64, RunSessionError> {
    if value==RoomScenario::M18DefectAct2 { Ok(1) } else { Err(RunSessionError::Input("profile is not representable in the M18 schema")) }
}
pub(crate) fn power(is_player: bool, name: &str) -> Result<i64, RunSessionError> {
    // Shared Act2 power names already belong to the accepted M16 vocabulary;
    // do not map a missing/foreign name to a padding zero.
    defect::power(is_player,name)
}
pub(crate) fn choice(value: &RoomChoiceKind) -> Result<i64, RunSessionError> {
    if let RoomChoiceKind::Card{id,upgrades}=value { permanent_card(*id,*upgrades)?; }
    act2::choice(value)
}
pub(crate) fn operation(value: &spire_core::combat::SelectionView) -> Result<i64, RunSessionError> {
    if matches!(value.operation,SelectionOperation::NightmareCapture|SelectionOperation::Retain) {
        return Err(RunSessionError::Input("unsupported M18 selection operation"));
    }
    act2::operation(value)
}
pub(crate) fn select_permanent_operation() -> i64 { act2::select_permanent_operation() }
fn permanent_card(id: CardId, upgrades: u32) -> Result<(), RunSessionError> {
    if !M18_DEFECT_ACT2_CARDS.contains(&id) || upgrades>1 {
        return Err(RunSessionError::Input("unsupported M18 permanent card"));
    }
    Ok(())
}
fn visible_card(card: &spire_core::combat::LiveCardView) -> Result<(), RunSessionError> {
    if M18_DEFECT_ACT2_CARDS.contains(&card.id) { return permanent_card(card.id,card.upgrades); }
    if !STATUSES.contains(&card.id) || card.upgrades > u32::from(card.id==CardId::Burn) {
        return Err(RunSessionError::Input("unsupported M18 generated card"));
    }
    if matches!(card.origin,spire_core::combat::CombatCardRef::Permanent(_)) {
        return Err(RunSessionError::Input("M18 status cannot be permanent"));
    }
    Ok(())
}
pub(crate) fn validate_public_decision(d: &PublicDecision) -> Result<(), RunSessionError> {
    if d.content_profile!=Some(RoomScenario::M18DefectAct2) || d.player.character!=Character::Defect {
        return Err(RunSessionError::Input("M18 encoder requires the Defect Act2 profile"));
    }
    if !matches!(d.terminal,None|Some(RunTerminal::Death|RunTerminal::EmptyDeck)) {
        return Err(RunSessionError::UnsupportedBoundary("M18 cannot encode a terminal outside its Act2 profile"));
    }
    if d.room.as_ref().is_some_and(|room|room.kind==RoomViewKind::Act3Ready)
        && (d.act!=3 || d.floor!=34 || d.player.hp==0 || d.terminal.is_some()
            || !d.candidates.is_empty() || d.combat.is_some()
            || d.room.as_ref().is_some_and(|room|!room.choices.is_empty())) {
        return Err(RunSessionError::UnsupportedBoundary("M18 Act3Ready must be healthy at 3/34 with no actions"));
    }
    for card in d.objects.iter().chain(d.candidates.iter().filter_map(|c|c.object.as_ref()))
        .chain(d.selection.iter().flat_map(|s|&s.eligible)) {
        permanent_card(card.id,card.upgrades)?;
    }
    for id in &d.player.relics { relic(*id)?; }
    for counter in &d.player.relic_counters { relic(counter.relic)?; }
    for id in d.player.potions.iter().flatten() { potion(*id)?; }
    if let Some(record)=&d.last_combat {
        if !act2::M13_PLAYABLE_ENCOUNTERS.contains(&record.encounter) {
            return Err(RunSessionError::Input("M18 map-only icon is not a played encounter"));
        }
        if record.outcome==Outcome::Escaped { return Err(RunSessionError::Input("unsupported M18 escape")); }
    }
    if let Some(combat)=&d.combat {
        if combat.character.orbs.len()>10 { return Err(RunSessionError::Input("M18 orb slots exceed gameplay cap10")); }
        if let Some(inventory)=&combat.inventory {
            for counter in &inventory.relic_counters { relic(counter.relic)?; }
        }
        if combat.outcome==Some(Outcome::Escaped) { return Err(RunSessionError::Input("unsupported M18 escape")); }
        for card in combat.hand.iter().chain(&combat.discard).chain(&combat.exhaust).chain(&combat.played_powers).chain(&combat.playing_cards)
            .chain(combat.stasis.iter().map(|s|&s.card)).chain(combat.known_draw.iter().map(|k|&k.card))
            .chain(combat.selection.iter().flat_map(|s|&s.candidates)) {
            visible_card(card)?;
        }
    }
    Ok(())
}
