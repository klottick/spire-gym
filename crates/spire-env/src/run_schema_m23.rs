//! Finite Watcher Acts1–2 catalog: accepted M22 Watcher groups, M13 Act2 actor/room vocabulary and Wound.
//! Producer rows and the Act2 capture domain come from the authoritative core owners.
use crate::{run_schema as v1,run_schema_m10 as m10,run_schema_m12 as act1,run_schema_m13 as act2,run_schema_m21 as watcher,run_schema_m22 as m22,RunCapacities,RunObservationSpec,RunSessionError};
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
use spire_core::{combat::{CombatCardRef,LiveCardView,Outcome,SelectionView},content::*,run::{*,m22_content as producers}};
use std::sync::OnceLock;
pub const RUN_OBSERVATION_M23_SCHEMA_ID:&str="spire-gym-m23-watcher-act2-observation-v1";
const DOMAIN:&[u8]=b"spire-gym/run-schema/m23-watcher-act2/v1\0";
// Act2 enemy-installed public powers (design union), deduplicated against M22.
const ACT2_POWERS:&[&str]=&["artifact","barricade","flight","hex","confused","minion","angry","entangled","painful_stabs","stasis"];
fn padded(names:impl IntoIterator<Item=&'static str>)->Value{json!(std::iter::once("padding").chain(names).collect::<Vec<_>>())}
fn vocabulary()->Value{
 let mut v=m22::m22_run_observation_spec(&RunCapacities::m10_default()).expect("fixed M22 vocabulary").descriptor()["vocabularies"].clone();
 let a=act2::vocabularies();
 for key in ["enemy","encounter","event","room","choice","intent"]{v[key]=a[key].clone();}
 v["card"]=padded(M23_WATCHER_ACT2_CARDS.iter().chain(m23_content::GENERATED_ONLY).chain(m23_content::STATUSES).map(|id|id.as_str()));
 v["relic"]=padded(M23_WATCHER_ACT2_RELICS.iter().map(|id|id.as_str()));v["potion"]=padded(M23_WATCHER_ACT2_POTIONS.iter().map(|id|id.as_str()));v["profile"]=json!(["padding","M23WatcherAct2"]);
 for name in ACT2_POWERS{let value=json!(name);assert!(a["power"].as_array().unwrap().contains(&value));if !v["power"].as_array().unwrap().contains(&value){v["power"].as_array_mut().unwrap().push(value);}}
 v
}
fn vocab()->&'static Value{static V:OnceLock<Value>=OnceLock::new();V.get_or_init(vocabulary)}
fn named(key:&str,name:&str)->Result<i64,RunSessionError>{vocab()[key].as_array().and_then(|a|a.iter().position(|v|v.as_str()==Some(name))).filter(|n|*n>0).map(|n|n as i64).ok_or(RunSessionError::Input("unsupported M23 public identity"))}
pub(crate) fn validate(c:&RunCapacities)->Result<(),RunSessionError>{m10::validate_groups(&watcher::groups(c),c)}
pub(crate) fn finite_content()->Value{
 let cards=|ids:&[CardId]|ids.iter().map(|id|id.as_str()).collect::<Vec<_>>();let relics=|ids:&[RelicId]|ids.iter().map(|id|id.as_str()).collect::<Vec<_>>();let potions=|ids:&[PotionId]|ids.iter().map(|id|id.as_str()).collect::<Vec<_>>();
 let mut capture=m22::finite_content()["capture_law"].clone();capture["scope"]=json!({"act1":producers::RELIC_CAPTURE_SCOPE_NAME,"act2":m23_content::ACT2_RELIC_CAPTURE_SCOPE_NAME});
 json!({"permanent_cards":cards(M23_WATCHER_ACT2_CARDS),"generated_cards":cards(m23_content::GENERATED_ONLY),"status_cards":cards(m23_content::STATUSES),
 "relics":relics(M23_WATCHER_ACT2_RELICS),"potions":potions(M23_WATCHER_ACT2_POTIONS),
 "playable_encounters":act2::M13_PLAYABLE_ENCOUNTERS.iter().map(|id|id.as_str()).collect::<Vec<_>>(),"playable_enemies":act2::M13_PLAYABLE_ENEMIES.iter().map(|id|id.as_str()).collect::<Vec<_>>(),"map_only_boss_icons":act2::M13_MAP_ONLY_BOSS_ICONS.iter().map(|id|id.as_str()).collect::<Vec<_>>(),
 "card_taxonomy":M23_WATCHER_ACT2_CARDS.iter().map(|id|json!({"id":id.as_str(),"rarity":format!("{:?}",Catalog::standard().card(*id).expect("finite card metadata").rarity)})).collect::<Vec<_>>(),"relic_taxonomy":producers::RELIC_TAXONOMY.iter().map(|(id,tier)|json!({"id":id.as_str(),"tier":tier})).collect::<Vec<_>>(),"capture_law":capture,
 "producers":m22::finite_content()["producers"].clone(),
 "continuation":"Act1 is the exact M22 prefix; committed (1,2,17) continues to the Act2 Map instead of Act2Ready"})
}
pub fn m23_run_observation_spec(c:&RunCapacities)->Result<RunObservationSpec,RunSessionError>{
 validate(c)?;
 let mut d=m10::spec_from_vocabulary(c,RUN_OBSERVATION_M23_SCHEMA_ID,"M23WatcherAct2",vocab().clone(),json!({
 "finite_content":finite_content(),"padding":"zero,known0,occupied0; unavailable references minus_one,known0",
 "actions":"row i is native candidate i; no filtering, truncation or reordering",
 "public_order":"Scry reveal top-first; selected ordinal one-based append/remove order; unrevealed draw, RNG, AI, families and cursors absent",
 "stances":"Neutral0 Wrath1 Calm2 Divinity3; current Mantra separate from gained history; real Wrath/Calm modal rows",
 "amounts":"current public costs, signed stats, Retain and installed powers; held/playing/Stasis custody",
 "cutoff":"Act3Ready act3 floor34 is healthy nonterminal zero actions after actual Act2 boss settlement; Act3 map icons only; no readiness bonus"
 }),watcher::groups(c),DOMAIN)?.descriptor();
 watcher::finalize_stance_descriptor(&mut d);
 let mut h=Sha256::new();h.update(DOMAIN);h.update(crate::canonical_json(&d).map_err(|_|RunSessionError::Input("invalid M23 descriptor"))?);Ok(RunObservationSpec{descriptor:d,fingerprint:h.finalize().into()})
}
pub(crate) fn card(id:CardId)->Result<i64,RunSessionError>{if matches!(id,CardId::Unknown(_)){return Err(RunSessionError::Input("unknown M23 card"))}named("card",id.as_str())}
pub(crate) fn relic(id:RelicId)->Result<i64,RunSessionError>{if matches!(id,RelicId::Unknown(_)){return Err(RunSessionError::Input("unknown M23 relic"))}named("relic",id.as_str())}
pub(crate) fn potion(id:PotionId)->Result<i64,RunSessionError>{named("potion",id.as_str())}
pub(crate) fn enemy(id:EnemyId)->Result<i64,RunSessionError>{named("enemy",id.as_str())}
pub(crate) fn encounter(id:EncounterId)->Result<i64,RunSessionError>{named("encounter",id.as_str())}
pub(crate) fn event(id:EventId)->Result<i64,RunSessionError>{named("event",id.as_str())}
pub(crate) fn room(id:RoomViewKind)->Result<i64,RunSessionError>{match id{RoomViewKind::Act3Ready=>v1::room(id),RoomViewKind::Act2Ready=>Err(RunSessionError::UnsupportedBoundary("M23 continues past Act2Ready")),_=>act1::room(id)}}
pub(crate) fn zone(name:&str)->Result<i64,RunSessionError>{named("zone",name)}
pub(crate) fn intent(name:&str)->Result<i64,RunSessionError>{named("intent",name)}
pub(crate) fn power(_:bool,name:&str)->Result<i64,RunSessionError>{named("power",name)}
pub(crate) fn profile(p:RoomScenario)->Result<i64,RunSessionError>{if p==RoomScenario::M23WatcherAct2{Ok(1)}else{Err(RunSessionError::Input("wrong M23 profile"))}}
pub(crate) fn choice(c:&RoomChoiceKind)->Result<i64,RunSessionError>{if matches!(c,RoomChoiceKind::Act3Continue{..}){return Err(RunSessionError::Input("unsupported M23 RoomChoiceKind"))}if let RoomChoiceKind::Card{id,upgrades}=c{permanent(*id,*upgrades)?;}Ok(m10::choice(c))}
pub(crate) fn operation(s:&SelectionView)->Result<i64,RunSessionError>{let n=watcher::operation(s)?;let d=watcher::vocabulary();named("selection_operation",d["selection_operation"][n as usize].as_str().unwrap())}
pub(crate) fn select_permanent_operation()->i64{named("selection_operation","select_permanent").expect("shared operation")}
fn permanent(id:CardId,upgrades:u32)->Result<(),RunSessionError>{if !M23_WATCHER_ACT2_CARDS.contains(&id)||upgrades>1{return Err(RunSessionError::Input("unsupported M23 permanent card"))}Ok(())}
fn visible(c:&LiveCardView)->Result<(),RunSessionError>{if matches!(c.origin,CombatCardRef::Permanent(_)){permanent(c.id,c.upgrades)}else{card(c.id)?;if c.upgrades>u32::from(!matches!(c.id,CardId::Dazed|CardId::Slimed|CardId::Wound)){return Err(RunSessionError::Input("unsupported M23 generated upgrade"))}Ok(())}}
pub(crate) fn validate_public_decision(d:&PublicDecision)->Result<(),RunSessionError>{
 if d.content_profile!=Some(RoomScenario::M23WatcherAct2)||d.player.character!=Character::Watcher{return Err(RunSessionError::Input("M23 encoder requires Watcher Acts1-2"))}
 let ready=d.room.as_ref().is_some_and(|r|r.kind==RoomViewKind::Act3Ready);
 let floors=match d.act{1=>d.floor<=17,2=>(17..=34).contains(&d.floor),3=>d.floor==34&&ready,_=>false};
 if !floors||d.room.as_ref().is_some_and(|r|r.kind==RoomViewKind::Act2Ready)||(ready&&!(d.act==3&&d.terminal.is_none()&&d.candidates.is_empty()))||matches!(d.terminal,Some(RunTerminal::Act3NoKeys|RunTerminal::HeartVictory)){return Err(RunSessionError::UnsupportedBoundary("outside M23 Acts1-2 boundary"))}
 for c in d.objects.iter().chain(d.selection.iter().flat_map(|s|&s.eligible)).chain(d.candidates.iter().filter_map(|c|c.object.as_ref())){permanent(c.id,c.upgrades)?;}
 if let Some(last)=&d.last_combat{if !act2::M13_PLAYABLE_ENCOUNTERS.contains(&last.encounter)||last.outcome==Outcome::Escaped{return Err(RunSessionError::Input("M23 map-only icon is not playable"))}}
 if let Some(c)=&d.combat{
  if c.stance_choice.is_some()&&(c.selection.is_some()||c.card_offer.is_some()||d.selection.is_some()){return Err(RunSessionError::Input("ambiguous M23 modal owners"))}
  if c.outcome==Some(Outcome::Escaped){return Err(RunSessionError::Input("unsupported M23 combat"))}
  for card in c.hand.iter().chain(&c.discard).chain(&c.exhaust).chain(&c.played_powers).chain(&c.playing_cards).chain(c.stasis.iter().map(|s|&s.card)).chain(c.known_draw.iter().map(|s|&s.card)).chain(c.selection.iter().flat_map(|s|&s.candidates)){visible(card)?;}
 }
 Ok(())
}

#[cfg(test)]
pub(crate) fn diagnostic_spec(c:&RunCapacities,fixture:spire_core::run::diagnostics::M23Diagnostic)->Result<RunObservationSpec,RunSessionError>{
 let mut d=m23_run_observation_spec(c)?.descriptor();d["semantics"]["diagnostic"]=json!({"fixture":format!("{fixture:?}"),"provenance":"constructed/supplied actual core owner (constructed (1,2,17) predecessor, visit0 boss record); internal SAME fixed initializer only; not normal portable replay"});
 let mut h=Sha256::new();h.update(b"spire-gym/run-schema/m23-watcher-act2/diagnostic/v1\0");h.update(crate::canonical_json(&d).map_err(|_|RunSessionError::Input("invalid diagnostic descriptor"))?);Ok(RunObservationSpec{descriptor:d,fingerprint:h.finalize().into()})
}
