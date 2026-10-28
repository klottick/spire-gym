//! Opt-in finite native Defect schema assembled by the shared Run descriptor owner.
use crate::{run_schema as v1,run_schema_m10 as m10,RunCapacities,RunObservationSpec,RunSessionError};
use serde_json::{json,Value};
use spire_core::content::*;
pub const M16_DEFECT_COMBAT_OBSERVATION_SCHEMA_ID:&str="spire-gym-m16-defect-combat-observation-v1";
const POWERS:&[&str]=&["focus","bias","loop","electrodynamics","lock_on","static_discharge","storm","heatsinks","amplify","echo_form"];
fn append(vocab:&mut Value,key:&str,names:impl IntoIterator<Item=&'static str>){
    let values=vocab[key].as_array_mut().expect("shared vocabulary");
    for name in names {if !values.iter().any(|v|v.as_str()==Some(name)){values.push(json!(name));}}
}
fn vocabulary()->Value{
    let mut vocab=crate::combat_policy::combat_policy_observation_spec_v2().expect("fixed shared descriptor").descriptor()["vocabularies"].clone();
    append(&mut vocab,"card",spire_core::content::defect::M16_DEFECT_CARDS.iter().map(|id|id.as_str()));
    append(&mut vocab,"relic",[RelicId::DataDisk,RelicId::GoldPlatedCables,RelicId::EmotionChip].into_iter().map(RelicId::as_str));
    append(&mut vocab,"potion",[PotionId::FocusPotion,PotionId::PotionOfCapacity].into_iter().map(PotionId::as_str));
    append(&mut vocab,"power",POWERS.iter().copied());
    vocab["orb"]=json!(["empty","lightning","frost","dark","plasma"]);
    vocab["profile"]=json!(["padding","M16DefectCombat"]);vocab
}
fn named(key:&str,name:&str)->Result<i64,RunSessionError>{
    use std::sync::OnceLock;
    static VOCAB:OnceLock<Value>=OnceLock::new();
    let vocab=VOCAB.get_or_init(vocabulary);
    vocab[key].as_array().and_then(|v|v.iter().position(|v|v.as_str()==Some(name))).filter(|index|*index>0)
        .map(|index|index as i64).ok_or(RunSessionError::Input("unsupported M16 vocabulary identity"))
}
pub(crate) fn card(id:CardId)->Result<i64,RunSessionError>{if matches!(id,CardId::Unknown(_)){return Err(RunSessionError::Input("unknown M16 card"))}named("card",id.as_str())}
pub(crate) fn enemy(id:EnemyId)->Result<i64,RunSessionError>{if matches!(id,EnemyId::Unknown(_)){return Err(RunSessionError::Input("unknown M16 enemy"))}named("enemy",id.as_str())}
pub(crate) fn encounter(id:EncounterId)->Result<i64,RunSessionError>{if matches!(id,EncounterId::Unknown(_)){return Err(RunSessionError::Input("unknown M16 encounter"))}named("encounter",id.as_str())}
pub(crate) fn relic(id:RelicId)->Result<i64,RunSessionError>{if matches!(id,RelicId::Unknown(_)){return Err(RunSessionError::Input("unknown M16 relic"))}named("relic",id.as_str())}
pub(crate) fn potion(id:PotionId)->Result<i64,RunSessionError>{named("potion",id.as_str())}
pub(crate) fn power(_:bool,name:&str)->Result<i64,RunSessionError>{named("power",name)}
pub(crate) fn intent(name:&str)->Result<i64,RunSessionError>{named("intent",name)}
pub(crate) fn groups(c:&RunCapacities)->Vec<v1::GroupSpec>{
    let mut groups=m10::groups(c);
    for group in &mut groups {
        match group.key {
            "permanent_cards"=>{group.fields="card_kind,upgrades,misc";group.units="category,upgrades,signed_int32";}
            "combat_cards"=>{group.fields="zone,zone_position,card_kind,upgrades,effective_cost,permanent_row,stasis_enemy_row,selected,base_damage,base_block,magic_number,damage_bonus,cost_kind,free_play,exhaust_on_finish,ethereal,innate,temporary_cost,misc";group.units="category,position,category,upgrades,energy,permanent_card_row,enemy_row,flag,damage,block,magic,damage,category,flag,flag,flag,flag,energy,signed_int32";}
            "orbs"=>{group.fields="kind,dark_damage";group.units="category,signed_int32";}
            _=>(),
        }
    }
    groups
}
pub(crate) fn validate(c:&RunCapacities)->Result<(),RunSessionError>{m10::validate_groups(&groups(c),c)}
pub fn defect_combat_observation_spec(c:&RunCapacities)->Result<RunObservationSpec,RunSessionError>{
    m10::spec_from_vocabulary(c,M16_DEFECT_COMBAT_OBSERVATION_SCHEMA_ID,"M16DefectCombat",vocabulary(),json!({
        "padding":"zero,known0,occupied0","orbs":"logical slot positions; known empty differs from padding; Dark damage known only for Dark",
        "cards":"public misc and combat cost/Block; hidden draw order, copy families and mutation ordinals are absent",
        "actions":"row i is original PublicDecision.candidates[i]; action mask is one exactly at candidate rows; no filtering, reordering or paging",
        "scope":"finite authored Defect A0 combat; supplied resources are diagnostic; no normal Run progression or Python delivery",
        "setup_metadata_base":spire_core::run::M16_DEFECT_COMBAT_SETUP_BASE,
        "setup_metadata_prefix_len":spire_core::run::M16_DEFECT_COMBAT_SETUP_PREFIX_LEN,
        "admitted_encounters":spire_core::combat::enemies::A0_ENCOUNTER_SETUPS.iter().take(spire_core::run::M16_DEFECT_COMBAT_SETUP_PREFIX_LEN).filter(|(_,act,_)|*act==1).map(|(id,_,_)|id.as_str()).collect::<Vec<_>>()
    }),groups(c),b"spire-gym/m16-defect-combat/observation/v1\0")
}
