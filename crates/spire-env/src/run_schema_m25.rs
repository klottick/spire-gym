//! Finite Watcher A0 Acts 1–4 and Heart vocabulary, composed from accepted Run builders and M25 owners.
use crate::{
    run_schema as v1, run_schema_m10 as m10, run_schema_m15 as m15,
    run_schema_m20 as m20, run_schema_m21 as m21, run_schema_m24 as m24,
    RunCapacities, RunObservationSpec, RunSessionError,
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use spire_core::{
    combat::{CombatCardRef, LiveCardView, Outcome, SelectionOperation, SelectionView},
    content::{CardId, Character, EncounterId, EnemyId, EventId, PotionId, RelicId},
    run::{m25_content, PublicDecision, RoomChoiceKind, RoomScenario, RoomViewKind},
};
use std::sync::OnceLock;

pub const RUN_OBSERVATION_M25_SCHEMA_ID: &str = "spire-gym-m25-watcher-heart-observation-v1";
const DOMAIN: &[u8] = b"spire-gym/run-schema/m25-watcher-heart/v1\0";

// These non-Watcher attacks suspend a player choice in the shared resolver:
// Headbutt chooses a discard-pile card to topdeck; Dagger Throw chooses a
// hand card to discard after its draw. All-Out Attack discards randomly, so it
// creates no selection operation.
const FOREIGN_INFLUENCE_SELECTION_OPERATIONS: &[(&str, &str)] = &[
    ("topdeck", "Headbutt"),
    ("move_discard", "Dagger Throw"),
];

fn append(vocab: &mut Value, key: &str, rows: impl IntoIterator<Item = Value>) {
    let target = vocab[key].as_array_mut().expect("shared vocabulary array");
    for row in rows {
        if !target.contains(&row) {
            target.push(row);
        }
    }
}

fn append_accepted(vocab: &mut Value, key: &str, accepted: &Value, names: &[Value]) {
    let source = accepted[key].as_array().expect("accepted vocabulary array");
    for name in names {
        assert!(source.contains(name), "{key} row {name} is not in its accepted descriptor source");
    }
    append(vocab, key, names.iter().cloned());
}

fn rows_from<'a>(value: &'a Value, key: &str) -> Vec<Value> {
    value[key]
        .as_array()
        .expect("semantic owner array")
        .iter()
        .cloned()
        .collect()
}

fn padded_names(values: impl IntoIterator<Item = String>) -> Vec<Value> {
    std::iter::once(json!("padding"))
        .chain(values.into_iter().map(|name| json!(name)))
        .collect()
}

fn vocabularies() -> Value {
    static M24: OnceLock<Value> = OnceLock::new();
    static M15: OnceLock<Value> = OnceLock::new();
    static M20: OnceLock<Value> = OnceLock::new();
    let caps = RunCapacities::m10_default();
    let m24_descriptor = M24.get_or_init(|| {
        m24::m24_run_observation_spec(&caps)
            .expect("fixed accepted M24 descriptor")
            .descriptor()
    });
    let m15_descriptor = M15.get_or_init(|| {
        m15::m15_run_observation_spec(&caps)
            .expect("fixed accepted M15 descriptor")
            .descriptor()
    });
    let m20_descriptor = M20.get_or_init(|| {
        m20::m20_run_observation_spec(&caps)
            .expect("fixed accepted M20 descriptor")
            .descriptor()
    });

    let mut vocab = m24_descriptor["vocabularies"].clone();
    let mut cards = m25_content::M25_WATCHER_HEART_CARDS
        .iter()
        .chain(m25_content::GENERATED_ONLY)
        .chain(m25_content::STATUSES)
        .map(|id| id.as_str().to_owned())
        .collect::<Vec<_>>();
    for id in spire_core::content::watcher::foreign_influence_support() {
        if !cards.iter().any(|name| name == id.as_str()) {
            cards.push(id.as_str().to_owned());
        }
    }
    assert_eq!(m25_content::M25_WATCHER_HEART_CARDS.len(), 76);
    assert_eq!(m25_content::GENERATED_ONLY.len(), 8);
    assert_eq!(m25_content::CHOICES.len(), 3);
    assert_eq!(m25_content::STATUSES.len(), 5);
    assert_eq!(spire_core::content::watcher::foreign_influence_support().len(), 106);
    assert_eq!(cards.len(), 171);
    vocab["card"] = json!(padded_names(cards));
    vocab["relic"] = json!(std::iter::once("padding")
        .chain(m25_content::M25_WATCHER_HEART_RELICS.iter().map(|id| id.as_str()))
        .collect::<Vec<_>>());
    vocab["potion"] = json!(std::iter::once("padding")
        .chain(m25_content::M25_WATCHER_HEART_POTIONS.iter().map(|id| id.as_str()))
        .collect::<Vec<_>>());
    vocab["profile"] = json!(["padding", "M25WatcherHeart"]);
    vocab["wish_choice"] = json!(padded_names(
        m25_content::CHOICES.iter().map(|id| id.as_str().to_owned())
    ));

    let act4 = &m15_descriptor["semantics"]["act4"];
    let m20_vocab = &m20_descriptor["vocabularies"];
    for (key, names) in [
        ("encounter", rows_from(act4, "encounters")),
        ("enemy", rows_from(act4, "enemies")),
    ] {
        append_accepted(&mut vocab, key, m20_vocab, &names);
    }

    // M20's accepted descriptor carries the two Act4 rooms and Heart terminal not in M24.
    for key in ["room", "terminal"] {
        let old_rows = vocab[key].as_array().expect("M24 rows");
        let additions = m20_vocab[key]
            .as_array()
            .expect("M20 rows")
            .iter()
            .filter(|row| !old_rows.contains(row))
            .cloned()
            .collect::<Vec<_>>();
        append_accepted(&mut vocab, key, m20_vocab, &additions);
    }

    for (name, _card) in FOREIGN_INFLUENCE_SELECTION_OPERATIONS {
        let row = json!(name);
        assert!(m15_descriptor["vocabularies"]["selection_operation"].as_array().unwrap().contains(&row),
            "Foreign Influence selection operation {name} must come from accepted M15");
        append(&mut vocab, "selection_operation", [row]);
    }
    // HC0 added this operation to the shared core enum. Its public spelling is the new enum variant.
    append(&mut vocab, "selection_operation", [json!("omniscience")]);
    vocab
}

fn vocab() -> &'static Value {
    static V: OnceLock<Value> = OnceLock::new();
    V.get_or_init(vocabularies)
}

fn named(key: &str, name: &str) -> Result<i64, RunSessionError> {
    vocab()[key]
        .as_array()
        .and_then(|rows| rows.iter().position(|row| row.as_str() == Some(name)))
        .filter(|index| *index > 0)
        .map(|index| index as i64)
        .ok_or(RunSessionError::Input("unsupported M25 public identity"))
}

pub(crate) fn groups(caps: &RunCapacities) -> Vec<v1::GroupSpec> {
    let mut groups = m21::groups(caps);
    if let Some(state) = groups.iter_mut().find(|row| row.key == "combat_state") {
        state.fields = "facing,surrounded,heart_damage_remaining,heart_cap,previous_card_type,phase,selection_resume_phase";
        state.units = "category,flag,allowance,allowance,category,category,category";
    }
    groups.extend(m15::groups(caps).into_iter().filter(|row| row.key == "combat_card_offers"));
    if let Some(offers) = groups.iter_mut().find(|row| row.key == "combat_card_offers") {
        offers.fields = "card_kind_or_wish_choice,offer_index";
        offers.units = "category_or_choice_id,offer_index";
    }
    groups
}

pub(crate) fn validate(caps: &RunCapacities) -> Result<(), RunSessionError> {
    m10::validate_groups(&groups(caps), caps)
}

fn finite_content() -> Value {
    let mut finite = m24::finite_content();
    finite["permanent_cards"] = json!(m25_content::M25_WATCHER_HEART_CARDS.iter().map(|id| id.as_str()).collect::<Vec<_>>());
    finite["generated_cards"] = json!(m25_content::GENERATED_ONLY.iter().map(|id| id.as_str()).collect::<Vec<_>>());
    finite["choice_cards"] = json!(m25_content::CHOICES.iter().map(|id| id.as_str()).collect::<Vec<_>>());
    finite["status_cards"] = json!(m25_content::STATUSES.iter().map(|id| id.as_str()).collect::<Vec<_>>());
    finite["relics"] = json!(m25_content::M25_WATCHER_HEART_RELICS.iter().map(|id| id.as_str()).collect::<Vec<_>>());
    finite["potions"] = json!(m25_content::M25_WATCHER_HEART_POTIONS.iter().map(|id| id.as_str()).collect::<Vec<_>>());

    let m15 = m15::m15_run_observation_spec(&RunCapacities::m10_default())
        .expect("fixed accepted M15 descriptor").descriptor();
    let act4 = &m15["semantics"]["act4"];
    let mut encounters = finite["playable_encounters"].as_array().unwrap().clone();
    for row in rows_from(act4, "encounters") {
        if !encounters.contains(&row) { encounters.push(row); }
    }
    finite["playable_encounters"] = json!(encounters);
    let mut enemies = finite["playable_enemies"].as_array().unwrap().clone();
    for row in rows_from(act4, "enemies") {
        if !enemies.contains(&row) { enemies.push(row); }
    }
    finite["playable_enemies"] = json!(enemies);
    finite["act4_encounters"] = act4["encounters"].clone();
    finite["act4_enemy_species"] = act4["enemies"].clone();

    let taxonomy = finite["relic_taxonomy"].as_array_mut().unwrap();
    for (id, tier) in m25_content::RELIC_TAXONOMY_ADDITIONS {
        taxonomy.push(json!({"id":id.as_str(),"tier":tier}));
    }
    finite["capture_law"]["scope"]["act4"] = json!(m25_content::ACT4_RELIC_CAPTURE_SCOPE_NAME);
    finite
}

pub fn m25_run_observation_spec(caps: &RunCapacities) -> Result<RunObservationSpec, RunSessionError> {
    validate(caps)?;
    let mut semantics = m24::m24_run_observation_spec(caps)?.descriptor()["semantics"].clone();
    if let Some(object) = semantics.as_object_mut() {
        object.remove("cutoff");
    }
    semantics["finite_content"] = finite_content();
    semantics["terminal"] = json!("ongoing, death, empty_deck, act3_no_keys and heart_victory; HeartVictory follows the final Proceed");
    semantics["ending"] = json!("HeartVictory only after the actual final Act4 Proceed; TrueVictory is the terminal room");
    semantics["act4"] = json!("the accepted M20 Act4 map, ShieldAndSpear and CorruptHeart rooms, and Heart ending");
    semantics["combat_phases"] = json!("combat_state.phase uses combat_phase codes; selection_resume_phase is the saved continuation phase, or padding when no selection is paused");
    semantics["wish_choice_offer_encoding"] = json!("combat_card_offers first-column positive IDs reference card rows; negative one-based IDs reference wish_choice rows, in the owner-provided order");

    let mut spec = m10::spec_from_vocabulary(
        caps,
        RUN_OBSERVATION_M25_SCHEMA_ID,
        "M25WatcherHeart",
        vocab().clone(),
        semantics,
        groups(caps),
        DOMAIN,
    )?;
    if let Some(offers) = spec.descriptor["tensors"].as_array_mut().unwrap()
        .iter_mut().find(|tensor| tensor["key"] == "combat_card_offers") {
        let choice_count = vocab()["wish_choice"].as_array().unwrap().len() as i64 - 1;
        let card_high = vocab()["card"].as_array().unwrap().len() as i64 - 1;
        offers["columns"][0] = json!({
            "field":"card_kind_or_wish_choice", "unit":"category_or_choice_id",
            "low":-choice_count, "high":card_high,
            "card_vocabulary":"card", "negative_choice_vocabulary":"wish_choice",
            "negative_encoding":"negative one-based row"
        });
        offers["low"][0] = json!(-choice_count);
        offers["high"][0] = json!(card_high);
    }
    if let Some(state) = spec.descriptor["tensors"].as_array_mut().unwrap()
        .iter_mut().find(|tensor| tensor["key"] == "combat_state") {
        let card_type_high = vocab()["card_type"].as_array().unwrap().len() as i64 - 1;
        state["columns"][4] = json!({
            "field":"previous_card_type", "unit":"category", "low":0,
            "high":card_type_high, "vocabulary":"card_type"
        });
        state["low"][4] = json!(0);
        state["high"][4] = json!(card_type_high);
    }
    m21::finalize_stance_descriptor(&mut spec.descriptor);
    let mut hash = Sha256::new();
    hash.update(DOMAIN);
    hash.update(crate::canonical_json(&spec.descriptor).map_err(|_| RunSessionError::Input("invalid M25 descriptor"))?);
    spec.fingerprint = hash.finalize().into();
    Ok(spec)
}

pub(crate) fn card(id: CardId) -> Result<i64, RunSessionError> {
    if matches!(id, CardId::Unknown(_)) { return Err(RunSessionError::Input("unknown M25 card")); }
    named("card", id.as_str())
}
pub(crate) fn relic(id: RelicId) -> Result<i64, RunSessionError> {
    if matches!(id, RelicId::Unknown(_)) { return Err(RunSessionError::Input("unknown M25 relic")); }
    named("relic", id.as_str())
}
pub(crate) fn potion(id: PotionId) -> Result<i64, RunSessionError> { named("potion", id.as_str()) }
pub(crate) fn enemy(id: EnemyId) -> Result<i64, RunSessionError> { named("enemy", id.as_str()) }
pub(crate) fn encounter(id: EncounterId) -> Result<i64, RunSessionError> { named("encounter", id.as_str()) }
pub(crate) fn event(id: EventId) -> Result<i64, RunSessionError> { named("event", id.as_str()) }
pub(crate) fn zone(name: &str) -> Result<i64, RunSessionError> { named("zone", name) }
pub(crate) fn offer_card(id: CardId) -> Result<i64, RunSessionError> {
    if let Some(row) = m25_content::CHOICES.iter().position(|choice| *choice == id) {
        return Ok(-((row as i64) + 1));
    }
    card(id)
}

pub(crate) fn room(id: RoomViewKind) -> Result<i64, RunSessionError> {
    let m24_rows = m24::m24_run_observation_spec(&RunCapacities::m10_default())?.descriptor();
    if let Ok(index) = m24::room(id) {
        let name = m24_rows["vocabularies"]["room"][index as usize].as_str()
            .ok_or(RunSessionError::Input("invalid M24 room identity"))?;
        return named("room", name);
    }
    let m20_rows = m20::m20_run_observation_spec(&RunCapacities::m10_default())?.descriptor();
    let index = m20::room(id)?;
    let name = m20_rows["vocabularies"]["room"][index as usize].as_str()
        .ok_or(RunSessionError::Input("invalid M20 room identity"))?;
    named("room", name)
}

pub(crate) fn profile(profile: RoomScenario) -> Result<i64, RunSessionError> {
    if profile == RoomScenario::M25WatcherHeart { Ok(1) } else { Err(RunSessionError::Input("wrong M25 profile")) }
}
pub(crate) fn choice(choice: &RoomChoiceKind) -> Result<i64, RunSessionError> {
    if let RoomChoiceKind::Card { id, upgrades } = choice { permanent(*id, *upgrades)?; }
    v1::choice(choice)
}
pub(crate) fn operation(selection: &SelectionView) -> Result<i64, RunSessionError> {
    let name = match selection.operation {
        SelectionOperation::Scry | SelectionOperation::MeditateReturn => {
            let index = m24::operation(selection)? as usize;
            m24::m24_run_observation_spec(&RunCapacities::m10_default())?.descriptor()["vocabularies"]["selection_operation"][index]
                .as_str().ok_or(RunSessionError::Input("invalid M24 selection operation"))?.to_owned()
        }
        SelectionOperation::Omniscience => "omniscience".to_owned(),
        SelectionOperation::HologramRetrieve | SelectionOperation::SeekRetrieve | SelectionOperation::Recycle => {
            let index = m20::operation(selection)? as usize;
            m20::m20_run_observation_spec(&RunCapacities::m10_default())?.descriptor()["vocabularies"]["selection_operation"][index]
                .as_str().ok_or(RunSessionError::Input("invalid M20 selection operation"))?.to_owned()
        }
        _ => {
            let index = m15::operation(selection)? as usize;
            m15::m15_run_observation_spec(&RunCapacities::m10_default())?.descriptor()["vocabularies"]["selection_operation"][index]
                .as_str().ok_or(RunSessionError::Input("invalid M15 selection operation"))?.to_owned()
        }
    };
    named("selection_operation", &name)
}
pub(crate) fn select_permanent_operation() -> i64 {
    named("selection_operation", "select_permanent").expect("shared M25 permanent selection")
}

fn permanent(id: CardId, upgrades: u32) -> Result<(), RunSessionError> {
    if !m25_content::M25_WATCHER_HEART_CARDS.contains(&id)
        || upgrades > u32::from(id != CardId::Parasite)
    {
        return Err(RunSessionError::Input("unsupported M25 permanent card"));
    }
    Ok(())
}

fn visible(card: &LiveCardView) -> Result<(), RunSessionError> {
    if matches!(card.origin, CombatCardRef::Permanent(_)) {
        return permanent(card.id, card.upgrades);
    }
    self::card(card.id)?;
    let status = m25_content::STATUSES.contains(&card.id);
    if (card.id == CardId::Parasite || status) && card.upgrades != 0 || card.upgrades > 1 {
        return Err(RunSessionError::Input("unsupported M25 generated card"));
    }
    Ok(())
}

pub(crate) fn validate_public_decision(d: &PublicDecision) -> Result<(), RunSessionError> {
    if d.content_profile != Some(RoomScenario::M25WatcherHeart) || d.player.character != Character::Watcher {
        return Err(RunSessionError::Input("M25 encoder requires Watcher Heart profile"));
    }
    if d.act > 4 || d.floor > 55 || d.room.as_ref().is_some_and(|room| matches!(room.kind, RoomViewKind::Act2Ready | RoomViewKind::Act3Ready)) {
        return Err(RunSessionError::UnsupportedBoundary("M25 requires the continuing Acts1–4 profile"));
    }
    if d.terminal.is_none() && d.candidates.is_empty() {
        return Err(RunSessionError::UnsupportedBoundary("nonterminal M25 has no legal actions"));
    }
    for object in d.objects.iter()
        .chain(d.candidates.iter().filter_map(|candidate| candidate.object.as_ref()))
        .chain(d.selection.iter().flat_map(|selection| &selection.eligible))
    {
        permanent(object.id, object.upgrades)?;
    }
    for id in &d.player.relics { relic(*id)?; }
    for counter in &d.player.relic_counters { relic(counter.relic)?; }
    for id in d.player.potions.iter().flatten() { potion(*id)?; }
    if let Some(record) = &d.last_combat {
        if !vocab()["encounter"].as_array().unwrap().contains(&json!(record.encounter.as_str())) || record.outcome == Outcome::Escaped {
            return Err(RunSessionError::Input("unsupported M25 combat record"));
        }
    }
    if let Some(combat) = &d.combat {
        for card in combat.hand.iter().chain(&combat.discard).chain(&combat.exhaust)
            .chain(&combat.played_powers).chain(&combat.playing_cards)
            .chain(combat.stasis.iter().map(|entry| &entry.card))
            .chain(combat.known_draw.iter().map(|entry| &entry.card))
            .chain(combat.selection.iter().flat_map(|selection| &selection.candidates))
        { visible(card)?; }
    }
    Ok(())
}

#[cfg(test)]
pub(crate) fn diagnostic_spec(
    caps: &RunCapacities,
    fixture: spire_core::run::diagnostics::M25Diagnostic,
) -> Result<RunObservationSpec, RunSessionError> {
    let mut descriptor = m25_run_observation_spec(caps)?.descriptor;
    descriptor["semantics"]["diagnostic"] = json!({
        "fixture": format!("{fixture:?}"),
        "provenance": "constructed actual core owner (spire_core::run::diagnostics m25); internal same fixed initializer only; not normal portable replay"
    });
    let mut hash = Sha256::new();
    hash.update(b"spire-gym/run-schema/m25-watcher-heart/diagnostic/v1\0");
    hash.update(crate::canonical_json(&descriptor).map_err(|_| RunSessionError::Input("invalid M25 diagnostic descriptor"))?);
    Ok(RunObservationSpec { descriptor, fingerprint: hash.finalize().into() })
}
