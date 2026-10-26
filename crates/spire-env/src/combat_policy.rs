//! Reusable public combat features; no resolver state or hidden identities.
use crate::{RunObservation,RunObservationSpec,RunCapacities,RunSessionError,RunDecisionToken};
use spire_core::{run::*,combat::{SelectionParent,Outcome}};
use serde_json::{Value,json};
use sha2::{Digest,Sha256};
pub const COMBAT_POLICY_VIEW_ID:&str="spire-gym-combat-policy-view-v1";
const GROUPS:&[&str]=&["player","permanent_cards","combat_cards","enemies","powers","relics","potions","selection","selection_counts","actions","combat_state","known_draw"];
pub fn combat_policy_capacities()->RunCapacities{RunCapacities{map_nodes:1,map_edges:1,permanent_cards:128,combat_cards:512,enemies:16,powers:512,relics:32,potions:3,offers:1,selection:128,orbs:1,actions:1024}}
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct CombatPolicyBoundary{pub observation:RunObservation,pub token:RunDecisionToken,pub info:Value}
pub fn combat_policy_observation_spec()->Result<RunObservationSpec,RunSessionError>{
    let caps=combat_policy_capacities();let mut d=policy_run_observation_spec(&caps)?.descriptor();
    d["schema_id"]=json!(COMBAT_POLICY_VIEW_ID);d["profile_id"]=json!("IroncladCombatPolicy");
    d["tensors"].as_array_mut().unwrap().retain(|t|{let key=t["key"].as_str().unwrap();key=="action_mask"||GROUPS.iter().any(|g|key==*g||key==format!("{g}_known")||key==format!("{g}_occupied"))});
    d["semantics"]["dispatch_index"]=json!("actions row is original native decision index; never filter/reorder legal rows");
    d["semantics"]["lifecycle"]=json!("routing, visit, token and outcome are separate metadata; noncombat mask is empty; resources are current, snapshot when settled count changes");
    d["semantics"]["terminal_resources"]=json!("settlement clears combat actors; permanent resources remain; HP checkpoints describe last exit, not later noncombat resource changes");
    let mut h=Sha256::new();h.update(b"spire-gym/combat-policy-view/v1\0");h.update(crate::canonical_json(&d).map_err(|_|RunSessionError::Input("invalid combat policy descriptor"))?);
    Ok(RunObservationSpec{descriptor:d,fingerprint:h.finalize().into()})
}
pub(crate) fn encode_combat_policy(d:&PublicDecision,checkpoint:Option<&CombatCheckpoint>)->Result<RunObservation,RunSessionError>{
    let mut d=d.clone();d.room=None;d.content_profile=None;d.offers.clear();
    if d.combat.is_none(){d.candidates.clear();d.selection=None;}
    let mut obs=crate::run_observation::encode_run_with_power(&d,None,&combat_policy_capacities(),true,policy_power)?;
    obs.groups.retain(|g|GROUPS.contains(&g.key));
    if d.combat.is_none(){if let Some(cp)=checkpoint{let p=obs.groups.iter_mut().find(|g|g.key=="player").unwrap();p.values[10]=i64::from(cp.hp_before_healing);p.values[11]=i64::from(cp.hp_after_cleanup);p.known[10]=1;p.known[11]=1;}}
    Ok(obs)
}
pub(crate) fn lifecycle(d:&PublicDecision,checkpoint:Option<&CombatCheckpoint>)->Value{
    let combat=d.combat.as_ref();let selection=combat.and_then(|c|c.selection.as_ref());
    let owner=selection.map(|s|match s.parent{SelectionParent::Card(_)=>"card",SelectionParent::Potion{..}=>"potion",SelectionParent::EndTurn{..}=>"end_turn"});
    let routing=if combat.is_none(){"noncombat"}else if selection.is_some(){"combat_selection"}else{"combat"};
    let exit=checkpoint.map(|c|json!({"visit":c.record.visit,"encounter":c.record.encounter.as_str(),"outcome":match c.record.outcome{Outcome::Victory=>"victory",Outcome::Death=>"death"},"hp_before_healing":c.hp_before_healing,"hp_after_cleanup":c.hp_after_cleanup}));
    json!({"combat_view_id":COMBAT_POLICY_VIEW_ID,"active":combat.is_some(),"routing":routing,"selection_owner":owner,
        "active_visit":combat.map(|_|d.combat_progress.entered-1),"combats_entered":d.combat_progress.entered,"combats_settled":d.combat_progress.settled,
        "last_exit":exit,"resource_checkpoint":"current; snapshot on settled-count change"})
}
pub(crate) fn hex(bytes:&[u8])->String{bytes.iter().map(|x|format!("{x:02x}")).collect()}

pub const RUN_OBSERVATION_POLICY_SCHEMA_ID:&str="spire-gym-run-observation-v3";
fn policy_power(m10:bool,name:&str)->Result<i64,RunSessionError>{
    if name=="lose_strength"{return Ok((crate::run_schema::POWERS.len()+crate::run_schema_m10::NEW_POWERS.len()) as i64)}
    if m10{crate::run_schema_m10::power(name)}else{crate::run_schema::named(crate::run_schema::POWERS,name)}
}
/// Additive vocabulary extension. Existing v1/v2 descriptors remain untouched.
pub fn policy_run_observation_spec(caps:&RunCapacities)->Result<RunObservationSpec,RunSessionError>{
    let mut d=crate::m10_run_observation_spec(caps)?.descriptor();
    d["schema_id"]=json!(RUN_OBSERVATION_POLICY_SCHEMA_ID);d["profile_id"]=json!("M10IroncladPolicy");
    let powers=d["vocabularies"]["power"].as_array_mut().unwrap();powers.push(json!("lose_strength"));let high=powers.len()-1;
    for t in d["tensors"].as_array_mut().unwrap(){if t["key"]=="powers"{t["high"][2]=json!(high);t["columns"][2]["high"]=json!(high);}}
    d["semantics"]["temporary_strength"]=json!("lose_strength is the public end-turn expiry debuff; appended vocabulary, frozen M10 v2 unchanged");
    let mut h=Sha256::new();h.update(b"spire-gym/run-schema/v3\0");h.update(crate::canonical_json(&d).map_err(|_|RunSessionError::Input("invalid policy Run descriptor"))?);
    Ok(RunObservationSpec{descriptor:d,fingerprint:h.finalize().into()})
}
pub fn encode_run_policy(d:&PublicDecision,map:Option<&spire_core::run::map::PublicMap>,caps:&RunCapacities)->Result<RunObservation,RunSessionError>{
    if d.content_profile!=Some(RoomScenario::M10IroncladHeart)||d.player.character!=spire_core::content::Character::Ironclad{return Err(RunSessionError::Input("policy encoder requires M10 Ironclad core profile"))}
    crate::run_observation::encode_run_with_power(d,map,caps,true,policy_power)
}
