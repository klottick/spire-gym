//! Finite standalone Watcher vocabulary over the shared public Run encoder.
use crate::{run_schema as v1,run_schema_m10 as m10,RunCapacities,RunObservationSpec,RunSessionError};
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
use spire_core::{content::*,combat::{SelectionOperation,SelectionView}};
use std::sync::OnceLock;
pub const M21_WATCHER_COMBAT_OBSERVATION_SCHEMA_ID:&str="spire-gym-m21-watcher-combat-observation-v1";
pub(crate) const GROUPS:&[&str]=&["player","permanent_cards","combat_cards","enemies","powers","relics","potions","selection","selection_counts","actions","combat_state","known_draw","stance_choices"];
fn vocabulary()->Value{
 let mut v=crate::combat_policy::combat_policy_observation_spec_v2().expect("fixed shared vocabulary").descriptor()["vocabularies"].clone();
 v["card"]=json!(std::iter::once("padding").chain(watcher::M21_WATCHER_CARDS.iter().chain(watcher::M21_WATCHER_GENERATED).map(|id|id.as_str())).chain(["dazed","slimed","wound","burn","void","parasite"]).collect::<Vec<_>>());
 v["relic"]=json!(std::iter::once("padding").chain(spire_core::run::M21_WATCHER_RELICS.iter().map(|id|id.as_str())).collect::<Vec<_>>());
 v["potion"]=json!(std::iter::once("padding").chain(spire_core::run::M21_WATCHER_POTIONS.iter().map(|id|id.as_str())).collect::<Vec<_>>());
 for name in ["mantra","devotion","foresight","nirvana","establishment","master_reality","battle_hymn","mental_fortress","rushdown","like_water"]{if !v["power"].as_array().unwrap().contains(&json!(name)){v["power"].as_array_mut().unwrap().push(json!(name));}}
 v["stance"]=json!(["neutral","wrath","calm","divinity"]);v["profile"]=json!(["padding","M21WatcherCombat"]);
 v["selection_operation"].as_array_mut().unwrap().extend([json!("scry"),json!("meditate_return")]);
 v["verb"].as_array_mut().unwrap().extend([json!("choose_stance"),json!("combat_deselect_card")]);v
}
fn vocab()->&'static Value{static V:OnceLock<Value>=OnceLock::new();V.get_or_init(vocabulary)}
fn named(key:&str,name:&str)->Result<i64,RunSessionError>{vocab()[key].as_array().and_then(|a|a.iter().position(|v|v.as_str()==Some(name))).map(|n|n as i64).ok_or(RunSessionError::Input("unsupported M21 public identity"))}
pub(crate) fn card(id:CardId)->Result<i64,RunSessionError>{if matches!(id,CardId::Unknown(_)){return Err(RunSessionError::Input("unknown M21 card"))}named("card",id.as_str())}
pub(crate) fn relic(id:RelicId)->Result<i64,RunSessionError>{if matches!(id,RelicId::Unknown(_)){return Err(RunSessionError::Input("unknown M21 relic"))}named("relic",id.as_str())}
pub(crate) fn potion(id:PotionId)->Result<i64,RunSessionError>{named("potion",id.as_str())}
pub(crate) fn power(_:bool,name:&str)->Result<i64,RunSessionError>{named("power",name)}
pub(crate) fn operation(s:&SelectionView)->Result<i64,RunSessionError>{match s.operation{SelectionOperation::Scry=>named("selection_operation","scry"),SelectionOperation::MeditateReturn=>named("selection_operation","meditate_return"),_=>Err(RunSessionError::UnsupportedBoundary("selection outside M21 profile"))}}
pub(crate) fn verb(name:&str)->Result<i64,RunSessionError>{named("verb",name)}
pub(crate) fn groups(c:&RunCapacities)->Vec<v1::GroupSpec>{
 let mut groups=crate::run_schema_m16::groups(c);
 for g in &mut groups{match g.key{
 "player"=>{g.fields="hp,max_hp,gold,block,energy,max_energy,turn,draw_count,stance,focus,hp_before_healing,hp_after_cleanup,mantra_gained";g.units="hp,hp,gold,block,energy,energy,turn,cards,category,power,hp,hp,signed_int32";},
 "combat_cards"=>{g.fields="zone,zone_position,card_kind,upgrades,effective_cost,permanent_row,stasis_enemy_row,selected,base_damage,base_block,magic_number,damage_bonus,cost_kind,free_play,exhaust_on_finish,ethereal,innate,temporary_cost,misc,combat_cost,turn_cost,self_retain,retain";g.units="category,position,category,upgrades,energy,permanent_card_row,enemy_row,flag,damage,block,magic,damage,category,flag,flag,flag,flag,energy,signed_int32,energy,energy,flag,flag";},
 "selection"=>{g.fields="source_group,source_row,eligible,selected,reveal_position,selected_ordinal";g.units="category,source_row,flag,flag,position,cards";},
 "actions"=>{g.fields="verb,source_group,source_row,target_group,target_row,choice_kind,scalar0,scalar1,stance_choice_ref";g.units="category,category,source_row,category,target_row,category,verb_specific,verb_specific,stance_choice_row";g.references=&[2,4,8];},
 _=>()}}
 groups.push(v1::GroupSpec{key:"stance_choices",rows:Some(2),fields:"stance",units:"category",references:&[]});groups
}
pub(crate) fn validate(c:&RunCapacities)->Result<(),RunSessionError>{m10::validate_groups(&groups(c),c)}
pub fn watcher_combat_observation_spec(c:&RunCapacities)->Result<RunObservationSpec,RunSessionError>{
 let mut d=m10::spec_from_vocabulary(c,M21_WATCHER_COMBAT_OBSERVATION_SCHEMA_ID,"m21-watcher-combat-v1",vocab().clone(),json!({
 "scope":"finite native Watcher A0 standalone authored combat; canonical resources bypass progression; supplied resources are diagnostic",
 "padding":"zero,known0,occupied0; unavailable references minus_one,known0",
 "actions":"row i is native candidate i; no filtering, paging or reordering",
 "public_order":"Scry candidates top-first; selected ordinal is one-based append/remove order; hidden draw remainder, RNG, families and cursors absent",
 "stances":"Neutral0 Wrath1 Calm2 Divinity3; stance choices separate from card offers",
 "amounts":"signed int32 public fields transported as native int64; no artificial gameplay clamp"
 }),groups(c),b"spire-gym/m21-watcher-combat/observation/v1\0")?.descriptor();
 d["tensors"].as_array_mut().unwrap().retain(|t|{let key=t["key"].as_str().unwrap();key=="action_mask"||GROUPS.iter().any(|g|key==*g||key==format!("{g}_known")||key==format!("{g}_occupied"))});
 for t in d["tensors"].as_array_mut().unwrap(){if t["key"]=="actions"{t["low"][8]=json!(-1);t["high"][8]=json!(1);t["columns"][8]=json!({"field":"stance_choice_ref","unit":"stance_choice_row","low":-1,"high":1,"reference":"stance_choices","unavailable":-1});}}
 let mut h=Sha256::new();h.update(b"spire-gym/m21-watcher-combat/observation/v1\0");h.update(crate::canonical_json(&d).map_err(|_|RunSessionError::Input("invalid M21 descriptor"))?);Ok(RunObservationSpec{descriptor:d,fingerprint:h.finalize().into()})
}
