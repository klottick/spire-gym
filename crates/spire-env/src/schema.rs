//! Owned native schema/space data. JSON is constructed only on explicit schema reads.
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use crate::canonical_json;
use spire_core::combat::CONTENT_ID;

pub const OBSERVATION_SCHEMA_ID: &str = "spire-lab-combat-observation-v1";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObservationSpec {
    descriptor: Value,
    fingerprint: [u8;32],
}
impl ObservationSpec {
    pub fn descriptor(&self) -> Value { self.descriptor.clone() }
    pub fn fingerprint(&self) -> [u8;32] { self.fingerprint }
}

fn tensor(key: &str, dtype: &str, shape: &[usize], high: Value, fields: &str,
    units: &str, axes: &[&str], category: Value, mask: Option<&str>,
    padding: i32, low: i32, semantics: &str) -> Value {
    json!({"key":key,"dtype":dtype,"shape":shape,"low":low,"high":high,
        "fields":fields.split(',').collect::<Vec<_>>(),
        "units":units.split(',').collect::<Vec<_>>(),"axes":axes,
        "category":category,"mask":mask,"padding":padding,"semantics":semantics,
        "space":if dtype == "int8" { "MultiBinary" } else { "Box" }})
}

pub fn observation_spec() -> ObservationSpec {
    let max = json!(i32::MAX);
    // Order is normative and shared by the single and batch numeric interfaces.
    let tensors = vec![
        tensor("player","int32",&[7],max.clone(),"hp,max_hp,block,energy,energy_per_turn,draw_per_turn,turn",
            "hit_points,hit_points,block_points,energy,energy,cards,turns",&["field"],Value::Null,None,0,0,"raw_public_values"),
        tensor("player_powers","int32",&[3],max.clone(),"strength,ritual,vulnerable",
            "power_amount,power_amount,power_amount",&["power"],Value::Null,None,0,0,"raw_public_values"),
        tensor("hand_kind","int32",&[10],json!(3),"kind","category",&["hand"],json!("card"),Some("hand_mask"),0,0,"raw_public_values"),
        tensor("hand_upgraded","int8",&[10],json!(1),"upgraded","flag",&["hand"],Value::Null,Some("hand_mask"),0,0,"raw_public_values"),
        tensor("hand_features","int32",&[10,4],max.clone(),"cost,printed_damage,printed_block,printed_vulnerable",
            "energy,hit_points,block_points,power_amount",&["hand","field"],Value::Null,Some("hand_mask"),0,0,"core_printed_stats_not_damage_prediction"),
        tensor("hand_mask","int8",&[10],json!(1),"occupied","flag",&["hand"],Value::Null,None,0,0,"occupied_public_hand_rows"),
        tensor("hand_playable","int8",&[10],json!(1),"playable","flag",&["hand"],Value::Null,Some("hand_mask"),0,0,"card_occurs_in_core_legal_menu_zero_at_terminal"),
        tensor("enemy_kind","int32",&[1],json!(2),"kind","category",&["enemy"],json!("enemy"),None,0,0,"raw_public_values"),
        tensor("enemy","int32",&[1,4],max.clone(),"hp,max_hp,block,intent_damage",
            "hit_points,hit_points,block_points,hit_points",&["enemy","field"],Value::Null,Some("enemy_damage_mask:intent_damage"),0,0,"published_intent_damage_zero_if_unavailable"),
        tensor("enemy_powers","int32",&[1,3],max.clone(),"strength,ritual,vulnerable",
            "power_amount,power_amount,power_amount",&["enemy","power"],Value::Null,None,0,0,"raw_public_values"),
        tensor("enemy_intent","int32",&[1],json!(5),"intent","category",&["enemy"],json!("intent"),None,0,0,"raw_public_values"),
        tensor("enemy_damage_mask","int8",&[1],json!(1),"damage_available","flag",&["enemy"],Value::Null,None,0,0,"published_intent_damage_is_available"),
        tensor("enemy_history","int32",&[1,2],json!(5),"previous_intent","category",&["enemy","previous_intent"],json!("intent"),Some("enemy_history_mask"),0,0,"newest_first_excluding_current_shift_once_per_completed_nonterminal_end_turn"),
        tensor("enemy_history_mask","int8",&[1,2],json!(1),"observed","flag",&["enemy","previous_intent"],Value::Null,None,0,0,"observed_previous_intents"),
        tensor("deck_counts","int32",&[3,2],max.clone(),"count","cards",&["count_kind","upgrade"],Value::Null,None,0,0,"constant_declared_permanent_deck_multiset"),
        tensor("pile_counts","int32",&[3,3,2],max.clone(),"count","cards",&["pile","count_kind","upgrade"],Value::Null,None,0,0,"draw_equals_deck_minus_hand_discard_exhaust_in_play"),
        tensor("in_play_counts","int32",&[3,2],max.clone(),"count","cards",&["count_kind","upgrade"],Value::Null,None,0,0,"accepted_terminal_play_card_only_otherwise_zero"),
        tensor("relic_presence","int8",&[1],json!(1),"present","flag",&["relic"],Value::Null,None,0,0,"burning_blood_always_one"),
        tensor("context","int32",&[4],json!([1,0,1,2]),"character,ascension,act,next_encounter",
            "category,ascension_level,act_number,category",&["field"],json!(["character",null,null,"enemy"]),Some("context_mask"),0,0,"declared_public_context_only_default_1_0_0_0"),
        tensor("context_mask","int8",&[4],json!(1),"character,ascension,act,next_encounter",
            "flag,flag,flag,flag",&["field"],Value::Null,None,0,0,"default_1_1_0_0_optional_fields_known_only_if_declared"),
        tensor("outcome","int32",&[1],json!(2),"outcome","category",&["outcome"],json!("outcome"),None,0,0,"raw_public_values"),
        tensor("action_kind","int32",&[32],json!(2),"kind","category",&["candidate"],json!("action"),Some("action_mask"),0,0,"end_turn_then_public_hand_row_then_public_enemy_row"),
        tensor("action_source","int32",&[32],json!(9),"hand_row","row_index",&["candidate"],Value::Null,Some("action_mask"),-1,-1,"minus_one_for_end_turn_or_padding"),
        tensor("action_target","int32",&[32],json!(0),"enemy_row","row_index",&["candidate"],Value::Null,Some("action_mask"),-1,-1,"minus_one_for_untargeted_or_padding"),
        tensor("action_features","int32",&[32,4],max,"cost,printed_damage,printed_block,printed_vulnerable",
            "energy,hit_points,block_points,power_amount",&["candidate","field"],Value::Null,Some("action_mask"),0,0,"core_printed_stats_zero_for_end_turn"),
        tensor("action_mask","int8",&[32],json!(1),"legal","flag",&["candidate"],Value::Null,None,0,0,"core_legal_candidates_empty_at_terminal"),
    ];
    let descriptor = json!({
        "schema_id":OBSERVATION_SCHEMA_ID,"content_id":CONTENT_ID,
        "capacities":{"hand":10,"enemy":1,"previous_intents":2,"candidates":32},
        "action_space":{"type":"Discrete","n":32,"start":0},
        "array_layout":"owned_c_contiguous_native_endian_row_major_batch_prefix",
        "history_order":"newest_first_excluding_current_terminal_retains_last_announced",
        "power_absence":"known_zero",
        "vocabularies":{
            "card":["padding","strike","defend","bash"],
            "count_kind":["strike","defend","bash"],"upgrade":["base","upgraded"],
            "enemy":["unknown_or_padding","cultist","jaw_worm"],
            "intent":["unavailable","incantation","attack","chomp","thrash","bellow"],
            "character":["unknown","ironclad"],"outcome":["ongoing","victory","death"],
            "action":["padding","end_turn","play_card"],"power":["strength","ritual","vulnerable"],
            "pile":["draw","discard","exhaust"],"relic":["burning_blood"]},
        "tensors":tensors,
    });
    let mut hash = Sha256::new();
    hash.update(b"spire-lab/env-schema/v1\0");
    hash.update(canonical_json(&descriptor).expect("integer-only static schema descriptor"));
    ObservationSpec { descriptor, fingerprint: hash.finalize().into() }
}
