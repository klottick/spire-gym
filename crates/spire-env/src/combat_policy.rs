//! Reusable public combat features; no resolver state or hidden identities.
use crate::{RunObservation,RunObservationSpec,RunCapacities,RunSessionError,RunDecisionToken};
use spire_core::{run::*,combat::{SelectionParent,Outcome}};
use serde_json::{Value,json};
use sha2::{Digest,Sha256};
pub const COMBAT_POLICY_VIEW_ID:&str="spire-gym-combat-policy-view-v1";
pub const COMBAT_POLICY_VIEW_V2_ID:&str="spire-gym-combat-policy-view-v2";
const CLOSURE_ENEMIES:&[spire_core::content::EnemyId]=&[
    spire_core::content::EnemyId::FungiBeast,spire_core::content::EnemyId::Looter,
    spire_core::content::EnemyId::Mugger,spire_core::content::EnemyId::ShelledParasite,
    spire_core::content::EnemyId::SnakePlant];
const CLOSURE_POWERS:&[&str]=&["spore_cloud","thievery","plated_armor"];
// Intent suffix is populated from the named source-qualified actor owners.
const CLOSURE_INTENTS:&[&str]=&["looter_mug","looter_lunge","looter_smoke_bomb","looter_escape","mugger_mug","mugger_lunge","mugger_smoke_bomb","mugger_escape","shelled_parasite_double_strike","shelled_parasite_life_suck","shelled_parasite_fell","shelled_parasite_stunned","snake_plant_chomp_chomp","snake_plant_enfeebling_spores"];
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
pub fn combat_policy_observation_spec_v2()->Result<RunObservationSpec,RunSessionError>{
    let mut d=combat_policy_observation_spec()?.descriptor();
    d["schema_id"]=json!(COMBAT_POLICY_VIEW_V2_ID);d["profile_id"]=json!("IroncladCombatPolicyV2");
    let enemies=d["vocabularies"]["enemy"].as_array_mut().unwrap();enemies.extend(CLOSURE_ENEMIES.iter().map(|id|json!(id.as_str())));let enemy_high=enemies.len()-1;
    let powers=d["vocabularies"]["power"].as_array_mut().unwrap();powers.extend(CLOSURE_POWERS.iter().map(|id|json!(id)));let power_high=powers.len()-1;
    let intents=d["vocabularies"]["intent"].as_array_mut().unwrap();intents.extend(CLOSURE_INTENTS.iter().map(|id|json!(id)));let intent_high=intents.len()-1;
    for tensor in d["tensors"].as_array_mut().unwrap(){
        match tensor["key"].as_str().unwrap(){
            "enemies"=>{tensor["high"][0]=json!(enemy_high);tensor["columns"][0]["high"]=json!(enemy_high);tensor["high"][4]=json!(intent_high);tensor["columns"][4]["high"]=json!(intent_high);},
            "powers"=>{tensor["high"][2]=json!(power_high);tensor["columns"][2]["high"]=json!(power_high);},
            _=>{}
        }
    }
    d["semantics"]["vocabulary_extension"]=json!("current standard encounter actors, intents and powers are appended after the frozen M10 and combat-policy v1 vocabularies");
    d["semantics"]["stolen_gold"]=json!("player gold is current committed currency; terminal last_exit stolen_gold_rewards are earned unclaimed death receipts, never an immediate refund; all_enemies_escaped and mugged distinguish escape");
    let mut h=Sha256::new();h.update(b"spire-gym/combat-policy-view/v2\0");h.update(crate::canonical_json(&d).map_err(|_|RunSessionError::Input("invalid combat policy descriptor"))?);
    Ok(RunObservationSpec{descriptor:d,fingerprint:h.finalize().into()})
}
pub(crate) fn encode_combat_policy(d:&PublicDecision,checkpoint:Option<&CombatCheckpoint>)->Result<RunObservation,RunSessionError>{
    encode_combat_policy_versioned(d,checkpoint,false)
}
pub(crate) fn encode_combat_policy_v2(d:&PublicDecision,checkpoint:Option<&CombatCheckpoint>)->Result<RunObservation,RunSessionError>{
    encode_combat_policy_versioned(d,checkpoint,true)
}
fn encode_combat_policy_versioned(d:&PublicDecision,checkpoint:Option<&CombatCheckpoint>,v2:bool)->Result<RunObservation,RunSessionError>{
    let mut d=d.clone();d.room=None;d.content_profile=None;d.offers.clear();
    if d.combat.is_none(){d.candidates.clear();d.selection=None;}
    let mut obs=if v2 {
        crate::run_observation::encode_run_with_schema(&d,None,&combat_policy_capacities(),crate::run_observation::RunEncodingProfile::CombatV2,policy_power_v2)?
    }else{
        crate::run_observation::encode_run_with_power(&d,None,&combat_policy_capacities(),true,policy_power)?
    };
    obs.groups.retain(|g|GROUPS.contains(&g.key));
    if d.combat.is_none(){if let Some(cp)=checkpoint{let p=obs.groups.iter_mut().find(|g|g.key=="player").unwrap();p.values[10]=i64::from(cp.hp_before_healing);p.values[11]=i64::from(cp.hp_after_cleanup);p.known[10]=1;p.known[11]=1;}}
    Ok(obs)
}
pub(crate) fn lifecycle(d:&PublicDecision,checkpoint:Option<&CombatCheckpoint>)->Value{
    lifecycle_for_view(d,checkpoint,COMBAT_POLICY_VIEW_ID)
}
pub(crate) fn lifecycle_v2(d:&PublicDecision,checkpoint:Option<&CombatCheckpoint>)->Value{
    let mut info=lifecycle_for_view(d,checkpoint,COMBAT_POLICY_VIEW_V2_ID);
    if let Some(checkpoint)=checkpoint{
        info["last_exit"]["stolen_gold_rewards"]=json!(checkpoint.stolen_gold_rewards);
        info["last_exit"]["all_enemies_escaped"]=json!(checkpoint.all_enemies_escaped);
        info["last_exit"]["mugged"]=json!(checkpoint.mugged);
    }
    info
}
fn lifecycle_for_view(d:&PublicDecision,checkpoint:Option<&CombatCheckpoint>,view_id:&str)->Value{
    let combat=d.combat.as_ref();let selection=combat.and_then(|c|c.selection.as_ref());
    let owner=selection.map(|s|match s.parent{SelectionParent::Card(_)=>"card",SelectionParent::Potion{..}=>"potion",SelectionParent::EndTurn{..}=>"end_turn",SelectionParent::Power{..}=>"power"});
    let routing=if combat.is_none(){"noncombat"}else if selection.is_some(){"combat_selection"}else{"combat"};
    let exit=checkpoint.map(|c|json!({"visit":c.record.visit,"encounter":c.record.encounter.as_str(),"outcome":match c.record.outcome{Outcome::Victory=>"victory",Outcome::Death=>"death",Outcome::Escaped=>"escaped"},"hp_before_healing":c.hp_before_healing,"hp_after_cleanup":c.hp_after_cleanup}));
    json!({"combat_view_id":view_id,"active":combat.is_some(),"routing":routing,"selection_owner":owner,
        "active_visit":combat.map(|_|d.combat_progress.entered-1),"combats_entered":d.combat_progress.entered,"combats_settled":d.combat_progress.settled,
        "last_exit":exit,"resource_checkpoint":"current; snapshot on settled-count change"})
}
pub(crate) fn hex(bytes:&[u8])->String{bytes.iter().map(|x|format!("{x:02x}")).collect()}

pub const RUN_OBSERVATION_POLICY_SCHEMA_ID:&str="spire-gym-run-observation-v3";
fn policy_power(m10:bool,name:&str)->Result<i64,RunSessionError>{
    if name=="lose_strength"{return Ok((crate::run_schema::POWERS.len()+crate::run_schema_m10::NEW_POWERS.len()) as i64)}
    if m10{crate::run_schema_m10::power(name)}else{crate::run_schema::named(crate::run_schema::POWERS,name)}
}
pub(crate) fn closure_enemy(id:spire_core::content::EnemyId)->Result<i64,RunSessionError>{
    if let Some(index)=CLOSURE_ENEMIES.iter().position(|candidate|*candidate==id){
        return Ok((crate::run_schema::ENEMIES.len()+crate::run_schema_m10::NEW_ENEMIES.len()+1+index) as i64)
    }
    crate::run_schema_m10::enemy(id)
}
pub(crate) fn closure_intent(name:&str)->Result<i64,RunSessionError>{
    if let Some(index)=CLOSURE_INTENTS.iter().position(|id|*id==name){
        return Ok(crate::run_schema_m10::intent("heart_buff")?+1+index as i64)
    }
    crate::run_schema_m10::intent(name)
}
fn policy_power_v2(m10:bool,name:&str)->Result<i64,RunSessionError>{
    if let Some(index)=CLOSURE_POWERS.iter().position(|id|*id==name){return Ok((crate::run_schema::POWERS.len()+crate::run_schema_m10::NEW_POWERS.len()+1+index) as i64)}
    policy_power(m10,name)
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

pub fn defect_combat_observation_spec(caps:&RunCapacities)->Result<RunObservationSpec,RunSessionError>{
    crate::run_schema_m16::defect_combat_observation_spec(caps)
}
pub fn encode_defect_combat(decision:&PublicDecision,caps:&RunCapacities)->Result<RunObservation,RunSessionError>{
    if decision.player.character!=spire_core::content::Character::Defect||decision.act!=1||decision.content_profile.is_some()
        ||decision.room.as_ref().is_some_and(|room|room.kind!=RoomViewKind::Combat)
        ||decision.candidates.iter().any(|candidate|!matches!(candidate.action,GameAction::Combat(_))){
        return Err(RunSessionError::Input("M16 encoder requires native authored Defect combat"))
    }
    crate::run_observation::encode_run_with_schema(decision,None,caps,crate::run_observation::RunEncodingProfile::M16Defect,crate::run_schema_m16::power)
}
