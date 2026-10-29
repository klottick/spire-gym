//! Opt-in M10 numeric Run schema. The v1 tables and fingerprint stay frozen.
use crate::{run_schema as v1, RunCapacities, RunObservationSpec, RunSessionError};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use spire_core::{content::*, run::*};

pub const RUN_OBSERVATION_M10_SCHEMA_ID: &str = "spire-gym-run-observation-v2";

const NEW_CARDS: &[CardId] = &[
    CardId::Armaments, CardId::SearingBlow, CardId::BodySlam, CardId::Carnage, CardId::Clash,
    CardId::Hemokinesis, CardId::Pummel, CardId::SwordBoomerang, CardId::Uppercut, CardId::WildStrike,
    CardId::Immolate, CardId::Feed, CardId::Reaper, CardId::InfernalBlade, CardId::Flex,
    CardId::LimitBreak, CardId::BloodForBlood, CardId::Rampage, CardId::Dropkick, CardId::Entrench,
    CardId::GhostlyArmor, CardId::Intimidate, CardId::PowerThrough, CardId::RecklessCharge,
    CardId::SeeingRed, CardId::Shockwave, CardId::SpotWeakness, CardId::TrueGrit, CardId::Headbutt,
    CardId::Warcry, CardId::BattleTrance, CardId::Exhume, CardId::DualWield, CardId::Havoc,
    CardId::DoubleTap, CardId::Whirlwind,
    CardId::Evolve, CardId::FireBreathing, CardId::FlameBarrier, CardId::Juggernaut, CardId::Rage,
    CardId::Rupture, CardId::Corruption,
    CardId::DarkEmbrace, CardId::FeelNoPain, CardId::Sentinel, CardId::SecondWind, CardId::SeverSoul,
    CardId::FiendFire, CardId::Barricade, CardId::Berserk, CardId::Brutality, CardId::DemonForm,
    CardId::Metallicize, CardId::Combust,
];
pub(crate) const NEW_ENEMIES: &[EnemyId] = &[EnemyId::SpireShield, EnemyId::SpireSpear, EnemyId::CorruptHeart];
const NEW_ENCOUNTERS: &[EncounterId] = &[EncounterId::ShieldAndSpear, EncounterId::CorruptHeart];
const NEW_RELICS: &[RelicId] = &[
    RelicId::RedSkull, RelicId::PaperPhrog, RelicId::ChampionsBelt, RelicId::CharonsAshes,
    RelicId::SelfFormingClay, RelicId::Orichalcum, RelicId::Brimstone, RelicId::MarkOfPain,
    RelicId::RunicCube,
];
const NEW_POTIONS: &[PotionId] = &[PotionId::Elixir, PotionId::HeartOfIron];
pub(crate) const NEW_POWERS: &[&str] = &[
    "berserk", "brutality", "combust", "corruption", "dark_embrace", "demon_form", "double_tap",
    "evolve", "feel_no_pain", "fire_breathing", "flame_barrier", "juggernaut", "no_draw", "rage", "rupture",
    "beat_of_death", "invincible", "surrounded", "back_attack",
];
const NEW_INTENTS: &[&str] = &[
    "shield_bash", "shield_fortify", "shield_smash", "spear_burn_strike", "spear_skewer",
    "spear_strength", "heart_debilitate", "heart_blood_shots", "heart_echo", "heart_buff",
];
const ZONES: &[&str] = &["known_draw_top", "known_draw_bottom", "playing"];
const SELECTION_OPERATIONS: &[&str] = &[
    "move_hand", "move_discard", "move_exhaust", "move_draw_top", "upgrade", "topdeck",
    "return_from_exhaust", "copy_to_hand_1", "copy_to_hand_2", "copy_to_hand_3", "select_permanent",
];

pub(crate) fn groups(c: &RunCapacities) -> Vec<v1::GroupSpec> {
    let mut groups = v1::groups(c);
    for group in &mut groups {
        match group.key {
            "combat_cards" => {
                group.fields = "zone,zone_position,card_kind,upgrades,effective_cost,permanent_row,stasis_enemy_row,selected,base_damage,base_block,magic_number,damage_bonus,cost_kind,free_play,exhaust_on_finish,ethereal,innate,temporary_cost";
                group.units = "category,position,category,upgrades,energy,permanent_card_row,enemy_row,flag,damage,block,magic,damage,category,flag,flag,flag,flag,energy";
            }
            "enemies" => {
                group.fields = "enemy_kind,hp,max_hp,block,intent_kind,intent_damage,replaced,escaped,half_dead,logical_slot,side,intent_hits";
                group.units = "category,hp,hp,block,category,hp,flag,flag,flag,slot,category,hits";
            }
            "selection_counts" => {
                group.fields = "required,selected_count,minimum,maximum,confirm_available,operation";
                group.units = "cards,cards,cards,cards,flag,category";
            }
            _ => {}
        }
    }
    groups.push(v1::GroupSpec {
        key: "combat_state",
        rows: None,
        fields: "facing,surrounded,heart_damage_remaining,heart_cap",
        units: "category,flag,allowance,allowance",
        references: &[],
    });
    groups.push(v1::GroupSpec {
        key: "known_draw",
        rows: Some(c.combat_cards),
        fields: "from_top,offset,combat_card_row",
        units: "flag,cards,combat_card_row",
        references: &[2],
    });
    groups
}

pub(crate) fn validate(c: &RunCapacities) -> Result<(), RunSessionError> {
    validate_groups(&groups(c), c)
}

pub fn m10_run_observation_spec(c: &RunCapacities) -> Result<RunObservationSpec, RunSessionError> {
    let vocab = vocabularies();
    spec_from_vocabulary(
        c,
        RUN_OBSERVATION_M10_SCHEMA_ID,
        "M10IroncladHeart",
        vocab,
        json!({
            "padding":"zero,known0,occupied0","unknown_reference":"minus_one,known0 in occupied rows",
            "counts":"only published public counts; no hidden draw rows","identity":"explicit versioned vocabulary IDs and row references only; no raw resolver IDs",
            "power_absence":"no row; listed signed amounts are public","potions":"belt slot rows; empty slot known1/occupied0/kind0; unused transport padding known0",
            "orbs":"N/A for Ironclad; known empty slot differs from padding/N/A",
            "selection":"Run selections expose counts without selected identities; entitled combat selections expose selected card rows and an explicit public operation",
            "known_draw":"only entitled known cards; references point to corresponding combat_cards rows; unknown draw cards have no rows",
            "playing_cards":"accepted public play owners use a dedicated playing zone",
            "actions":"legal candidate order; no paging, truncation or policy",
            "scalar0":"potion belt slot for potion verbs; otherwise absent","scalar1":"reserved absent",
            "heart":"remaining Invincible allowance is public only when the Heart/power view supplies it; cap is source-qualified A0 value 300",
            "offer_amount":"Neow HP/gold/heal/Joust result/Falling type/stage per choice_kind"
        }),
        groups(c),
        b"spire-gym/run-schema/v2\0",
    )
}

/// Shared descriptor assembly for finite Run vocabularies. The profile supplies
/// only its vocabulary, groups and identity; row layout and domain construction
/// stay with the existing M10-compatible owner.
pub(crate) fn spec_from_vocabulary(
    c: &RunCapacities,
    schema_id: &str,
    profile_id: &str,
    vocab: Value,
    semantics: Value,
    groups: Vec<v1::GroupSpec>,
    fingerprint_domain: &[u8],
) -> Result<RunObservationSpec, RunSessionError> {
    validate_groups(&groups, c)?;
    let mut tensors = Vec::new();
    for group in groups {
        let columns = group.fields.split(',').zip(group.units.split(','))
            .map(|(field, unit)| domain(group.key, field, unit, c, &vocab)).collect::<Vec<_>>();
        let shape = group.rows.map_or_else(|| vec![columns.len()], |rows| vec![rows, columns.len()]);
        let low = columns.iter().map(|column| column["low"].clone()).collect::<Vec<_>>();
        let high = columns.iter().map(|column| column["high"].clone()).collect::<Vec<_>>();
        tensors.push(json!({"key":group.key,"dtype":"int64","shape":shape,"low":low,"high":high,
            "space":"Box","fields":group.fields.split(',').collect::<Vec<_>>(),
            "units":group.units.split(',').collect::<Vec<_>>(),"columns":columns,"padding":0,
            "known":format!("{}_known",group.key),"reference_columns":group.references}));
        tensors.push(json!({"key":format!("{}_known",group.key),"dtype":"int8","shape":shape,
            "low":0,"high":1,"space":"MultiBinary","padding":0}));
        if let Some(rows) = group.rows {
            tensors.push(json!({"key":format!("{}_occupied",group.key),"dtype":"int8","shape":[rows],
                "low":0,"high":1,"space":"MultiBinary","padding":0}));
        }
    }
    tensors.push(json!({"key":"action_mask","dtype":"int8","shape":[c.actions],"low":0,"high":1,
        "space":"MultiBinary","padding":0}));
    let descriptor = json!({
        "schema_id":schema_id,
        "profile_id":profile_id,
        "array_layout":"owned_c_contiguous_native_endian_row_major",
        "capacities":{"map_nodes":c.map_nodes,"map_edges":c.map_edges,"permanent_cards":c.permanent_cards,
            "combat_cards":c.combat_cards,"enemies":c.enemies,"powers":c.powers,"relics":c.relics,
            "potions":c.potions,"offers":c.offers,"selection":c.selection,"orbs":c.orbs,"actions":c.actions},
        "action_space":{"type":"Discrete","n":c.actions,"start":0},
        "tensors":tensors,"vocabularies":vocab,
        "semantics":semantics
    });
    let canonical = crate::canonical_json(&descriptor).map_err(|_| RunSessionError::Input("invalid Run descriptor"))?;
    let mut hash = Sha256::new();
    hash.update(fingerprint_domain);
    hash.update(canonical);
    Ok(RunObservationSpec { descriptor, fingerprint: hash.finalize().into() })
}

pub(crate) fn validate_groups(groups: &[v1::GroupSpec], c: &RunCapacities) -> Result<(), RunSessionError> {
    let mut bytes = 0usize;
    for group in groups {
        if group.rows == Some(0) {
            return Err(RunSessionError::Input("capacities must be positive"));
        }
        let count = group.rows.unwrap_or(1)
            .checked_mul(group.fields.split(',').count())
            .ok_or(RunSessionError::Input("Run shape overflow"))?;
        bytes = bytes.checked_add(count.checked_mul(9).ok_or(RunSessionError::Input("Run shape overflow"))?)
            .ok_or(RunSessionError::Input("Run shape overflow"))?;
    }
    if bytes > isize::MAX as usize {
        return Err(RunSessionError::Input("Run shape overflow"));
    }
    // Keep capacity references in the common schema builder's range checks.
    let _ = c;
    Ok(())
}

fn append<T: PartialEq>(id: T, base: &[T], extra: &[T], message: &'static str) -> Result<i64, RunSessionError> {
    if let Some(index) = base.iter().position(|value| *value == id) {
        return Ok(index as i64 + 1);
    }
    extra.iter().position(|value| *value == id)
        .map(|index| (base.len() + index + 1) as i64)
        .ok_or(RunSessionError::Input(message))
}

pub(crate) fn card(id: CardId) -> Result<i64, RunSessionError> { append(id, v1::CARDS, NEW_CARDS, "unsupported CardId") }
pub(crate) fn enemy(id: EnemyId) -> Result<i64, RunSessionError> { append(id, v1::ENEMIES, NEW_ENEMIES, "unsupported EnemyId") }
pub(crate) fn encounter(id: EncounterId) -> Result<i64, RunSessionError> { append(id, v1::ENCOUNTERS, NEW_ENCOUNTERS, "unsupported EncounterId") }
pub(crate) fn relic(id: RelicId) -> Result<i64, RunSessionError> { append(id, v1::RELICS, NEW_RELICS, "unsupported RelicId") }
pub(crate) fn potion(id: PotionId) -> Result<i64, RunSessionError> { append(id, v1::POTIONS, NEW_POTIONS, "unsupported PotionId") }

pub(crate) fn event(id: EventId) -> Result<i64, RunSessionError> { v1::event(id) }
pub(crate) fn profile(profile: RoomScenario) -> Result<i64, RunSessionError> {
    if profile == RoomScenario::M10IroncladHeart { Ok(7) } else { v1::profile(profile) }
}
pub(crate) fn room(kind: RoomViewKind) -> Result<i64, RunSessionError> {
    match kind {
        RoomViewKind::HeartVictory => Ok(22),
        RoomViewKind::TrueVictory => Ok(23),
        _ => v1::room(kind),
    }
}
pub(crate) fn choice(choice: &RoomChoiceKind) -> i64 { v1::choice(choice) }

pub(crate) fn power(name: &str) -> Result<i64, RunSessionError> {
    if let Some(index) = v1::POWERS.iter().position(|value| *value == name).filter(|index| *index > 0) {
        return Ok(index as i64);
    }
    NEW_POWERS.iter().position(|value| *value == name)
        .map(|index| (v1::POWERS.len() + index) as i64)
        .ok_or(RunSessionError::Input("unsupported public power or intent"))
}

pub(crate) fn intent(name: &str) -> Result<i64, RunSessionError> {
    if let Ok(value) = v1::intent(name) { return Ok(value); }
    NEW_INTENTS.iter().position(|value| *value == name)
        .map(|index| (v1::INTENTS.len() + v1::ACT1_INTENTS.len() + index) as i64)
        .ok_or(RunSessionError::Input("unsupported public intent"))
}

pub(crate) fn zone(name: &str) -> Result<i64, RunSessionError> {
    let zones = ["padding", "hand", "discard", "exhaust", "played_power", "stasis", "selection_only"];
    zones.iter().position(|value| *value == name).filter(|index| *index > 0).map(|index| index as i64)
        .or_else(|| ZONES.iter().position(|value| *value == name).map(|index| (zones.len() + index) as i64))
        .ok_or(RunSessionError::Input("unsupported public card zone"))
}

pub(crate) fn operation(selection: &spire_core::combat::SelectionView) -> i64 {
    use spire_core::combat::{SelectionDisposition::*, SelectionOperation::*};
    let name = match selection.operation {
        HologramRetrieve|SeekRetrieve|Recycle=>panic!("M20 selection requires M20 native profile"),
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
        CopyToHand { .. } => "copy_to_hand_3",
        Setup|Bottomdeck|RetrieveMemories=>panic!("M10 v2 selection schema cannot encode M15 choices"),
        NightmareCapture => panic!("M10 v2 selection schema cannot encode M11 Nightmare selections"),
        Retain => panic!("M10 v2 selection schema cannot encode M11 retain choices"),
    };
    SELECTION_OPERATIONS.iter().position(|value| *value == name).expect("declared M10 selection operation") as i64 + 1
}

pub(crate) fn select_permanent_operation() -> i64 {
    SELECTION_OPERATIONS.iter().position(|value| *value == "select_permanent").expect("declared M10 run-selection operation") as i64 + 1
}

pub(crate) fn vocabularies() -> Value {
    let mut vocab = v1::vocabularies();
    append_strings(&mut vocab, "card", &NEW_CARDS.iter().map(|id| id.as_str()).collect::<Vec<_>>());
    append_strings(&mut vocab, "enemy", &NEW_ENEMIES.iter().map(|id| id.as_str()).collect::<Vec<_>>());
    append_strings(&mut vocab, "encounter", &NEW_ENCOUNTERS.iter().map(|id| id.as_str()).collect::<Vec<_>>());
    append_strings(&mut vocab, "relic", &NEW_RELICS.iter().map(|id| id.as_str()).collect::<Vec<_>>());
    append_strings(&mut vocab, "potion", &NEW_POTIONS.iter().map(|id| id.as_str()).collect::<Vec<_>>());
    append_strings(&mut vocab, "power", NEW_POWERS);
    append_strings(&mut vocab, "intent", NEW_INTENTS);
    append_strings(&mut vocab, "profile", &["M10IroncladHeart"]);
    append_strings(&mut vocab, "room", &["heart_victory", "true_victory"]);
    append_strings(&mut vocab, "zone", ZONES);
    append_strings(&mut vocab, "verb", &["confirm_selection"]);
    append_strings(&mut vocab, "row_group", &["combat_state", "known_draw"]);
    vocab["encounter_side"] = json!(["padding", "left", "right"]);
    vocab["facing"] = json!(["padding", "left", "right"]);
    vocab["card_cost_kind"] = json!(["padding", "fixed", "x", "unplayable"]);
    vocab["selection_operation"] = json!(["padding", "move_hand", "move_discard", "move_exhaust", "move_draw_top",
        "upgrade", "topdeck", "return_from_exhaust", "copy_to_hand_1", "copy_to_hand_2", "copy_to_hand_3", "select_permanent"]);
    vocab
}

fn append_strings(vocab: &mut Value, key: &str, values: &[&str]) {
    let table = vocab[key].as_array_mut().expect("declared M10 vocabulary");
    table.extend(values.iter().map(|value| Value::String((*value).to_owned())));
}

fn domain(group: &str, field: &str, unit: &str, c: &RunCapacities, vocab: &Value) -> Value {
    // Only the appended M15 offer group uses this field. Existing profile
    // descriptors retain the same domains and serialized bytes.
    if group == "combat_card_offers" && field == "offer_index" {
        return json!({"field":field,"unit":unit,"low":0,"high":c.offers-1});
    }
    let category = match (group, field) {
        ("combat_cards", "cost_kind") => Some("card_cost_kind"),
        ("enemies", "side") => Some("encounter_side"),
        ("combat_state", "facing") => Some("facing"),
        ("selection_counts", "operation") => Some("selection_operation"),
        _ => None,
    };
    if let Some(category) = category {
        return json!({"field":field,"unit":unit,"low":0,"high":vocab[category].as_array().unwrap().len()-1,"vocabulary":category});
    }
    if group == "known_draw" && field == "combat_card_row" {
        return json!({"field":field,"unit":unit,"low":-1,"high":c.combat_cards-1,"reference":"combat_cards","unavailable":-1});
    }
    if group == "combat_state" && matches!(field,"heart_damage_remaining"|"heart_cap") {
        return json!({"field":field,"unit":unit,"low":0,"high":300});
    }
    if group == "enemies" && field == "intent_hits" {
        return json!({"field":field,"unit":unit,"low":0,"high":12});
    }
    if group == "known_draw" && field == "offset" {
        return json!({"field":field,"unit":unit,"low":0,"high":u32::MAX});
    }
    if matches!(unit, "damage" | "block" | "magic" | "allowance" | "signed_int32") {
        return json!({"field":field,"unit":unit,"low":i32::MIN,"high":i32::MAX});
    }
    v1::domain(group, field, unit, c, vocab)
}
