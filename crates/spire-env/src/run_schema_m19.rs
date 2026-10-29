//! Finite M19 vocabulary composed from the accepted Defect and shared Act3 owners.
use crate::{run_schema as v1,run_schema_m10 as m10,run_schema_m13 as prefix,
    run_schema_m14 as act3,run_schema_m16 as defect,run_schema_m18 as m18,
    RunCapacities,RunObservationSpec,RunSessionError};
use serde_json::{json,Value};
use spire_core::{combat::{Outcome,CombatCardRef,LiveCardView},content::*,run::*};
pub const RUN_OBSERVATION_M19_SCHEMA_ID:&str="spire-gym-m19-defect-act3-observation-v1";
const STATUSES:&[CardId]=&[CardId::Dazed,CardId::Burn,CardId::Slimed,CardId::Wound,CardId::Void];
pub(crate) fn groups(c:&RunCapacities)->Vec<v1::GroupSpec>{defect::groups(c)}
pub(crate) fn validate(c:&RunCapacities)->Result<(),RunSessionError>{defect::validate(c)}
fn padded(names:impl Iterator<Item=&'static str>)->Value{json!(std::iter::once("padding").chain(names).collect::<Vec<_>>())}
fn vocabulary(c:&RunCapacities)->Result<Value,RunSessionError>{
    let mut vocab=m18::m18_run_observation_spec(c)?.descriptor()["vocabularies"].clone();
    let shared=act3::m14_run_observation_spec(c)?.descriptor()["vocabularies"].clone();
    for key in ["enemy","encounter","event","intent","choice","room","terminal"] {vocab[key]=shared[key].clone();}
    vocab["card"]=padded(M19_DEFECT_ACT3_CARDS.iter().chain(STATUSES).map(|id|id.as_str()));
    vocab["relic"]=padded(M19_DEFECT_ACT3_RELICS.iter().map(|id|id.as_str()));
    vocab["potion"]=padded(M19_DEFECT_ACT3_POTIONS.iter().map(|id|id.as_str()));
    vocab["profile"]=json!(["padding","M19DefectAct3"]);Ok(vocab)
}
pub(crate) fn finite_content()->Value{
    let mut enemies=prefix::M13_PLAYABLE_ENEMIES.to_vec();
    for id in act3::M14_ACT3_ENEMIES {if !enemies.contains(id){enemies.push(*id);}}
    let mut encounters=prefix::M13_PLAYABLE_ENCOUNTERS.to_vec();
    for id in act3::M14_ACT3_ENCOUNTERS {if !encounters.contains(id){encounters.push(*id);}}
    json!({"permanent_cards":M19_DEFECT_ACT3_CARDS.iter().map(|id|id.as_str()).collect::<Vec<_>>(),
        "generated_and_status_cards":STATUSES.iter().map(|id|id.as_str()).collect::<Vec<_>>(),
        "relics":M19_DEFECT_ACT3_RELICS.iter().map(|id|id.as_str()).collect::<Vec<_>>(),
        "potions":M19_DEFECT_ACT3_POTIONS.iter().map(|id|id.as_str()).collect::<Vec<_>>(),
        "playable_enemies":enemies.iter().map(|id|id.as_str()).collect::<Vec<_>>(),
        "playable_encounters":encounters.iter().map(|id|id.as_str()).collect::<Vec<_>>(),
        "act3_encounters":act3::M14_ACT3_ENCOUNTERS.iter().map(|id|id.as_str()).collect::<Vec<_>>(),
        "act3_enemy_species":act3::M14_ACT3_ENEMIES.iter().map(|id|id.as_str()).collect::<Vec<_>>(),"map_only_boss_icons":[]})
}
pub fn m19_run_observation_spec(c:&RunCapacities)->Result<RunObservationSpec,RunSessionError>{
    validate(c)?;
    m10::spec_from_vocabulary(c,RUN_OBSERVATION_M19_SCHEMA_ID,"M19DefectAct3",vocabulary(c)?,json!({
        "padding":"zero,known0,occupied0","unknown_reference":"minus_one,known0 in occupied rows",
        "counts":"only published public counts; no hidden draw rows",
        "cards":"public signed misc, combat cost, damage and Block through shared M16 groups",
        "orbs":"logical slots through ten; known empty differs from padding; signed Dark damage is known only for Dark",
        "actions":"row i is PublicDecision.candidates[i]; no filtering, truncation or paging",
        "finite_content":finite_content(),"sludge":"awa_sludge creates Void at BottomDraw",
        "terminal":"ongoing, death, empty_deck and act3_no_keys; no Heart victory",
        "cutoff":"healthy nonterminal Act4Ready at act3/floor50 with zero actions, before the Act4 door; supplied diagnostic only",
        "scope":"finite Defect A0 Acts1–3; native source only"
    }),groups(c),b"spire-gym/run-schema/m19-defect-act3/v1\0")
}
fn index<T:PartialEq>(id:T,mut values:impl Iterator<Item=T>)->Result<i64,RunSessionError>{
    values.position(|v|v==id).map(|i|i as i64+1).ok_or(RunSessionError::Input("unsupported M19 content identity"))
}
pub(crate) fn card(id:CardId)->Result<i64,RunSessionError>{index(id,M19_DEFECT_ACT3_CARDS.iter().chain(STATUSES).copied())}
pub(crate) fn relic(id:RelicId)->Result<i64,RunSessionError>{index(id,M19_DEFECT_ACT3_RELICS.iter().copied())}
pub(crate) fn potion(id:PotionId)->Result<i64,RunSessionError>{index(id,M19_DEFECT_ACT3_POTIONS.iter().copied())}
pub(crate) fn enemy(id:EnemyId)->Result<i64,RunSessionError>{act3::enemy(id)}
pub(crate) fn encounter(id:EncounterId)->Result<i64,RunSessionError>{act3::encounter(id)}
pub(crate) fn event(id:EventId)->Result<i64,RunSessionError>{act3::event(id)}
pub(crate) fn room(id:RoomViewKind)->Result<i64,RunSessionError>{act3::room(id)}
pub(crate) fn intent(name:&str)->Result<i64,RunSessionError>{act3::intent(name)}
pub(crate) fn zone(name:&str)->Result<i64,RunSessionError>{m18::zone(name)}
pub(crate) fn power(player:bool,name:&str)->Result<i64,RunSessionError>{m18::power(player,name)}
pub(crate) fn operation(s:&spire_core::combat::SelectionView)->Result<i64,RunSessionError>{m18::operation(s)}
pub(crate) fn select_permanent_operation()->i64{m18::select_permanent_operation()}
pub(crate) fn profile(p:RoomScenario)->Result<i64,RunSessionError>{
    if p==RoomScenario::M19DefectAct3{Ok(1)}else{Err(RunSessionError::Input("profile is not representable in the M19 schema"))}
}
fn permanent(id:CardId,upgrades:u32)->Result<(),RunSessionError>{
    if !M19_DEFECT_ACT3_CARDS.contains(&id)||upgrades>u32::from(id!=CardId::Parasite){return Err(RunSessionError::Input("unsupported M19 permanent card"))}Ok(())
}
pub(crate) fn choice(c:&RoomChoiceKind)->Result<i64,RunSessionError>{
    if let RoomChoiceKind::Card{id,upgrades}=c{permanent(*id,*upgrades)?;}act3::choice(c)
}
fn visible(c:&LiveCardView)->Result<(),RunSessionError>{
    if M19_DEFECT_ACT3_CARDS.contains(&c.id){return permanent(c.id,c.upgrades)}
    if !STATUSES.contains(&c.id)||c.upgrades>u32::from(c.id==CardId::Burn)||matches!(c.origin,CombatCardRef::Permanent(_)){
        return Err(RunSessionError::Input("unsupported M19 generated card"))
    }Ok(())
}
pub(crate) fn validate_public_decision(d:&PublicDecision)->Result<(),RunSessionError>{
    if d.content_profile!=Some(RoomScenario::M19DefectAct3)||d.player.character!=Character::Defect{return Err(RunSessionError::Input("M19 encoder requires the Defect Act3 profile"))}
    if d.act>3||d.floor>50||!matches!(d.terminal,None|Some(RunTerminal::Death|RunTerminal::EmptyDeck|RunTerminal::Act3NoKeys)){
        return Err(RunSessionError::UnsupportedBoundary("M19 cannot encode Act4 gameplay"))
    }
    if let Some(r)=&d.room{
        if matches!(r.kind,RoomViewKind::Act4Ready) && (d.act!=3||d.floor!=50||d.player.hp==0||d.terminal.is_some()||!d.candidates.is_empty()||d.combat.is_some()||!r.choices.is_empty()){
            return Err(RunSessionError::UnsupportedBoundary("M19 Act4Ready must be healthy at 3/50 with no actions"))
        }
        if matches!(r.kind,RoomViewKind::Act2Ready|RoomViewKind::Act3Ready){return Err(RunSessionError::UnsupportedBoundary("M19 continues through generated Acts1–3"))}
    }
    for c in d.objects.iter().chain(d.candidates.iter().filter_map(|c|c.object.as_ref())).chain(d.selection.iter().flat_map(|s|&s.eligible)){permanent(c.id,c.upgrades)?;}
    for id in &d.player.relics{relic(*id)?;}for c in &d.player.relic_counters{relic(c.relic)?;}for id in d.player.potions.iter().flatten(){potion(*id)?;}
    if let Some(r)=&d.last_combat{
        if !(prefix::M13_PLAYABLE_ENCOUNTERS.contains(&r.encounter)||act3::M14_ACT3_ENCOUNTERS.contains(&r.encounter))||r.outcome==Outcome::Escaped{return Err(RunSessionError::Input("unsupported M19 combat record"))}
    }
    if let Some(c)=&d.combat{
        if c.character.orbs.len()>10{return Err(RunSessionError::Input("M19 orb slots exceed gameplay cap10"))}
        if c.outcome==Some(Outcome::Escaped){return Err(RunSessionError::Input("unsupported M19 escape"))}
        if let Some(i)=&c.inventory{for counter in &i.relic_counters{relic(counter.relic)?;}}
        for card in c.hand.iter().chain(&c.discard).chain(&c.exhaust).chain(&c.played_powers).chain(&c.playing_cards)
            .chain(c.stasis.iter().map(|s|&s.card)).chain(c.known_draw.iter().map(|k|&k.card)).chain(c.selection.iter().flat_map(|s|&s.candidates)){visible(card)?;}
    }Ok(())
}
