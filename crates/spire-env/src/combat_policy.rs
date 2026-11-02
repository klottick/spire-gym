//! Reusable public combat features; no resolver state or hidden identities.
use crate::{RunObservation,RunObservationSpec,RunCapacities,RunSessionError,RunDecisionToken};
use spire_core::{run::*,combat::{Phase,SelectionParent,Outcome}};
use serde_json::{Value,json};
use sha2::{Digest,Sha256};
pub const COMBAT_POLICY_VIEW_ID:&str="spire-gym-combat-policy-view-v1";
pub const COMBAT_POLICY_VIEW_V2_ID:&str="spire-gym-combat-policy-view-v2";
const CLOSURE_ENEMIES:&[spire_core::content::EnemyId]=&[
    spire_core::content::EnemyId::FungiBeast,spire_core::content::EnemyId::Looter,
    spire_core::content::EnemyId::Mugger,spire_core::content::EnemyId::ShelledParasite,
    spire_core::content::EnemyId::SnakePlant];
const GROUPS:&[&str]=&["player","permanent_cards","combat_cards","enemies","powers","relics","potions","selection","selection_counts","actions","combat_state","known_draw"];
// Visible combat cards grow without bound in a stall (Repulsor/Heart status cards); keep
// the policy row bound equal to the shared Run default so the views share one overflow edge.
pub fn combat_policy_capacities()->RunCapacities{RunCapacities{map_nodes:1,map_edges:1,permanent_cards:128,combat_cards:RunCapacities::m9_default().combat_cards,enemies:16,powers:512,relics:32,potions:3,offers:1,selection:128,orbs:1,actions:1024}}
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct CombatPolicyBoundary{pub observation:RunObservation,pub token:RunDecisionToken,pub info:Value}
pub fn combat_policy_observation_spec()->Result<RunObservationSpec,RunSessionError>{
    let caps=combat_policy_capacities();let mut d=policy_run_observation_spec(&caps)?.descriptor();
    d["schema_id"]=json!(COMBAT_POLICY_VIEW_ID);d["profile_id"]=json!("IroncladCombatPolicy");
    d["tensors"].as_array_mut().unwrap().retain(|t|{let key=t["key"].as_str().unwrap();key=="action_mask"||GROUPS.iter().any(|g|key==*g||key==format!("{g}_known")||key==format!("{g}_occupied"))});
    d["semantics"]["dispatch_index"]=json!("actions row is original native decision index; never filter/reorder legal rows");
    d["semantics"]["lifecycle"]=json!("routing, visit, token, outcome, current combat phase and selection resume phase are separate metadata; noncombat mask is empty; resources are current, snapshot when settled count changes");
    d["semantics"]["terminal_resources"]=json!("settlement clears combat actors; permanent resources remain; HP checkpoints describe last exit, not later noncombat resource changes");
    let mut h=Sha256::new();h.update(b"spire-gym/combat-policy-view/v1\0");h.update(crate::canonical_json(&d).map_err(|_|RunSessionError::Input("invalid combat policy descriptor"))?);
    Ok(RunObservationSpec{descriptor:d,fingerprint:h.finalize().into()})
}
pub fn combat_policy_observation_spec_v2()->Result<RunObservationSpec,RunSessionError>{
    let mut d=combat_policy_observation_spec()?.descriptor();
    d["schema_id"]=json!(COMBAT_POLICY_VIEW_V2_ID);d["profile_id"]=json!("IroncladCombatPolicyV2");
    let enemies=d["vocabularies"]["enemy"].as_array_mut().unwrap();enemies.extend(CLOSURE_ENEMIES.iter().map(|id|json!(id.as_str())));let enemy_high=enemies.len()-1;
    for tensor in d["tensors"].as_array_mut().unwrap(){
        match tensor["key"].as_str().unwrap(){
            "enemies"=>{tensor["high"][0]=json!(enemy_high);tensor["columns"][0]["high"]=json!(enemy_high);},
            _=>{}
        }
    }
    d["semantics"]["vocabulary_extension"]=json!("current standard encounter actors are appended after the M10 and combat-policy v1 enemy vocabulary; powers and intents use the shared engine-registry vocabulary");
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
        crate::run_observation::encode_run_with_schema(&d,None,&combat_policy_capacities(),crate::run_observation::RunEncodingProfile::CombatV2)?
    }else{
        crate::run_observation::encode_run_versioned(&d,None,&combat_policy_capacities(),true)?
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
pub(crate) fn lifecycle_for_view(d:&PublicDecision,checkpoint:Option<&CombatCheckpoint>,view_id:&str)->Value{
    let combat=d.combat.as_ref();let selection=combat.and_then(|c|c.selection.as_ref());
    let owner=selection.map(|s|match s.parent{SelectionParent::Card(_)=>"card",SelectionParent::Potion{..}=>"potion",
        SelectionParent::Relic(_)=>"relic",SelectionParent::EndTurn{..}=>"end_turn",SelectionParent::Power{..}=>"power"});
    let routing=if combat.is_none(){"noncombat"}else if selection.is_some(){"combat_selection"}else{"combat"};
    let phase=combat.map(|c|phase_label(c.phase));
    let selection_resume_phase=combat.and_then(|c|c.selection_resume_phase.map(phase_label));
    let exit=checkpoint.map(|c|json!({"visit":c.record.visit,"encounter":c.record.encounter.as_str(),"outcome":match c.record.outcome{Outcome::Victory=>"victory",Outcome::Death=>"death",Outcome::Escaped=>"escaped"},"hp_before_healing":c.hp_before_healing,"hp_after_cleanup":c.hp_after_cleanup}));
    json!({"combat_view_id":view_id,"active":combat.is_some(),"routing":routing,"selection_owner":owner,
        "phase":phase,"selection_resume_phase":selection_resume_phase,
        "active_visit":combat.map(|_|d.combat_progress.entered-1),"combats_entered":d.combat_progress.entered,"combats_settled":d.combat_progress.settled,
        "last_exit":exit,"resource_checkpoint":"current; snapshot on settled-count change"})
}
fn phase_label(phase:Phase)->&'static str{match phase{Phase::Decision=>"decision",Phase::Selection=>"selection",Phase::PlayerAction=>"player_action",Phase::EnemyTurn=>"enemy_turn",Phase::RoundEnd=>"round_end",Phase::PlayerStart=>"player_start",Phase::Terminal=>"terminal"}}
pub(crate) fn hex(bytes:&[u8])->String{bytes.iter().map(|x|format!("{x:02x}")).collect()}

pub const RUN_OBSERVATION_POLICY_SCHEMA_ID:&str="spire-gym-run-observation-v3";
pub(crate) fn closure_enemy(id:spire_core::content::EnemyId)->Result<i64,RunSessionError>{
    if let Some(index)=CLOSURE_ENEMIES.iter().position(|candidate|*candidate==id){
        return Ok((crate::run_schema::ENEMIES.len()+crate::run_schema_m10::NEW_ENEMIES.len()+1+index) as i64)
    }
    crate::run_schema_m10::enemy(id)
}
/// Additive vocabulary extension. Existing v1/v2 descriptors remain untouched.
pub fn policy_run_observation_spec(caps:&RunCapacities)->Result<RunObservationSpec,RunSessionError>{
    let mut d=crate::m10_run_observation_spec(caps)?.descriptor();
    d["schema_id"]=json!(RUN_OBSERVATION_POLICY_SCHEMA_ID);d["profile_id"]=json!("M10IroncladPolicy");
    d["semantics"]["temporary_strength"]=json!("lose_strength is the public end-turn expiry debuff in the shared engine-registry power vocabulary");
    let mut h=Sha256::new();h.update(b"spire-gym/run-schema/v3\0");h.update(crate::canonical_json(&d).map_err(|_|RunSessionError::Input("invalid policy Run descriptor"))?);
    Ok(RunObservationSpec{descriptor:d,fingerprint:h.finalize().into()})
}
pub fn encode_run_policy(d:&PublicDecision,map:Option<&spire_core::run::map::PublicMap>,caps:&RunCapacities)->Result<RunObservation,RunSessionError>{
    if d.content_profile!=Some(RoomScenario::M10IroncladHeart)||d.player.character!=spire_core::content::Character::Ironclad{return Err(RunSessionError::Input("policy encoder requires M10 Ironclad core profile"))}
    crate::run_observation::encode_run_versioned(d,map,caps,true)
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
    crate::run_observation::encode_run_with_schema(decision,None,caps,crate::run_observation::RunEncodingProfile::M16Defect)
}

/// Opt-in public Watcher authored-combat projection.
pub fn watcher_combat_observation_spec(caps:&RunCapacities)->Result<RunObservationSpec,RunSessionError>{
    crate::run_schema_m21::watcher_combat_observation_spec(caps)
}
pub fn encode_watcher_combat(d:&PublicDecision,caps:&RunCapacities)->Result<RunObservation,RunSessionError>{
    if d.player.character!=spire_core::content::Character::Watcher||d.content_profile.is_some()||!(1..=4).contains(&d.act){return Err(RunSessionError::Input("M21 requires standalone Watcher combat"))}
    if let Some(c)=&d.combat{
        if c.stance_choice.is_some()&&(c.selection.is_some()||c.card_offer.is_some()) {return Err(RunSessionError::Input("ambiguous Watcher modal owners"))}
        if c.stance_choice.as_ref().is_some_and(|choice|choice.options!=[spire_core::combat::Stance::Wrath,spire_core::combat::Stance::Calm]){return Err(RunSessionError::Input("unsupported Watcher stance options"))}
    }
    let mut d=d.clone();d.room=None;d.offers.clear();if d.combat.is_none(){d.candidates.clear();d.selection=None;}
    let mut o=crate::run_observation::encode_run_with_schema(&d,None,caps,crate::run_observation::RunEncodingProfile::M21Watcher)?;
    o.groups.retain(|g|crate::run_schema_m21::GROUPS.contains(&g.key));Ok(o)
}
