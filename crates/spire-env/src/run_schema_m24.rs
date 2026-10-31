//! Finite Watcher Acts1–3 catalog: the accepted M23 Watcher schema plus the accepted Act3 vocabulary of M19.
//! Producer rows and the Act1–3 capture domains come from the authoritative core content owners.
use crate::{run_schema as v1,run_schema_m10 as m10,run_schema_m13 as act2,run_schema_m14 as act3,run_schema_m19 as m19,run_schema_m21 as watcher,run_schema_m22 as m22,run_schema_m23 as m23,RunCapacities,RunObservationSpec,RunSessionError};
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
use spire_core::{combat::{CombatCardRef,LiveCardView,Outcome,SelectionView},content::*,run::{*,m22_content as producers}};
use std::sync::OnceLock;
pub const RUN_OBSERVATION_M24_SCHEMA_ID:&str="spire-gym-m24-watcher-act3-observation-v1";
const DOMAIN:&[u8]=b"spire-gym/run-schema/m24-watcher-act3/v1\0";
// Act3 enemy-installed public powers (design union), deduplicated against M23; each must exist in the M19 Act3 vocabulary.
const ACT3_POWERS:&[&str]=&["regeneration","thorns","explosive","life_link","fading","shifting","shackled","constricted","reactive","malleable","slow","intangible","regenerate","curiosity","unawakened","time_warp","draw_reduction","strength_up"];
fn padded(names:impl IntoIterator<Item=&'static str>)->Value{json!(std::iter::once("padding").chain(names).collect::<Vec<_>>())}
fn vocabulary()->Value{
 let caps=RunCapacities::m10_default();
 let mut v=m23::m23_run_observation_spec(&caps).expect("fixed M23 vocabulary").descriptor()["vocabularies"].clone();
 let a=m19::m19_run_observation_spec(&caps).expect("fixed M19 vocabulary").descriptor()["vocabularies"].clone();
 for key in ["enemy","encounter","event","intent","choice","room","terminal"]{v[key]=a[key].clone();}
 v["card"]=padded(M24_WATCHER_ACT3_CARDS.iter().chain(m24_content::GENERATED_ONLY).chain(m24_content::STATUSES).map(|id|id.as_str()));
 v["relic"]=padded(M24_WATCHER_ACT3_RELICS.iter().map(|id|id.as_str()));v["potion"]=padded(M24_WATCHER_ACT3_POTIONS.iter().map(|id|id.as_str()));v["profile"]=json!(["padding","M24WatcherAct3"]);
 for name in ACT3_POWERS{let value=json!(name);assert!(a["power"].as_array().unwrap().contains(&value),"{name} is an accepted Act3 power");if !v["power"].as_array().unwrap().contains(&value){v["power"].as_array_mut().unwrap().push(value);}}
 v
}
fn vocab()->&'static Value{static V:OnceLock<Value>=OnceLock::new();V.get_or_init(vocabulary)}
fn named(key:&str,name:&str)->Result<i64,RunSessionError>{vocab()[key].as_array().and_then(|a|a.iter().position(|v|v.as_str()==Some(name))).filter(|n|*n>0).map(|n|n as i64).ok_or(RunSessionError::Input("unsupported M24 public identity"))}
pub(crate) fn validate(c:&RunCapacities)->Result<(),RunSessionError>{m10::validate_groups(&watcher::groups(c),c)}
fn playable_encounters()->Vec<EncounterId>{let mut e=act2::M13_PLAYABLE_ENCOUNTERS.to_vec();for id in act3::M14_ACT3_ENCOUNTERS{if !e.contains(id){e.push(*id);}}e}
fn playable_enemies()->Vec<EnemyId>{let mut e=act2::M13_PLAYABLE_ENEMIES.to_vec();for id in act3::M14_ACT3_ENEMIES{if !e.contains(id){e.push(*id);}}e}
pub(crate) fn finite_content()->Value{
 let cards=|ids:&[CardId]|ids.iter().map(|id|id.as_str()).collect::<Vec<_>>();let relics=|ids:&[RelicId]|ids.iter().map(|id|id.as_str()).collect::<Vec<_>>();let potions=|ids:&[PotionId]|ids.iter().map(|id|id.as_str()).collect::<Vec<_>>();
 let mut capture=m22::finite_content()["capture_law"].clone();capture["scope"]=json!({"act1":producers::RELIC_CAPTURE_SCOPE_NAME,"act2":m23_content::ACT2_RELIC_CAPTURE_SCOPE_NAME,"act3":m24_content::ACT3_RELIC_CAPTURE_SCOPE_NAME});
 json!({"permanent_cards":cards(M24_WATCHER_ACT3_CARDS),"generated_cards":cards(m24_content::GENERATED_ONLY),"status_cards":cards(m24_content::STATUSES),
 "relics":relics(M24_WATCHER_ACT3_RELICS),"potions":potions(M24_WATCHER_ACT3_POTIONS),
 "playable_encounters":playable_encounters().iter().map(|id|id.as_str()).collect::<Vec<_>>(),"playable_enemies":playable_enemies().iter().map(|id|id.as_str()).collect::<Vec<_>>(),"map_only_boss_icons":[],
 "act3_encounters":act3::M14_ACT3_ENCOUNTERS.iter().map(|id|id.as_str()).collect::<Vec<_>>(),"act3_enemy_species":act3::M14_ACT3_ENEMIES.iter().map(|id|id.as_str()).collect::<Vec<_>>(),
 "card_taxonomy":M24_WATCHER_ACT3_CARDS.iter().map(|id|json!({"id":id.as_str(),"rarity":format!("{:?}",Catalog::standard().card(*id).expect("finite card metadata").rarity)})).collect::<Vec<_>>(),
 "relic_taxonomy":producers::RELIC_TAXONOMY.iter().chain(m24_content::RELIC_TAXONOMY_ADDITIONS).map(|(id,tier)|json!({"id":id.as_str(),"tier":tier})).collect::<Vec<_>>(),"capture_law":capture,
 "producers":m22::finite_content()["producers"].clone(),
 "parasite":"permanent Curse only from actual Writhing Mass Implant; never offered by a reward, shop or event",
 "continuation":"Acts1–2 are the exact M23 prefix; committed (2,3,34) continues to the Act3 Map instead of Act3Ready"})
}
pub fn m24_run_observation_spec(c:&RunCapacities)->Result<RunObservationSpec,RunSessionError>{
 validate(c)?;
 let mut d=m10::spec_from_vocabulary(c,RUN_OBSERVATION_M24_SCHEMA_ID,"M24WatcherAct3",vocab().clone(),json!({
 "finite_content":finite_content(),"padding":"zero,known0,occupied0; unavailable references minus_one,known0",
 "actions":"row i is native candidate i; no filtering, truncation or reordering",
 "public_order":"Scry reveal top-first; selected ordinal one-based append/remove order; unrevealed draw, RNG, AI, families and cursors absent",
 "stances":"Neutral0 Wrath1 Calm2 Divinity3; current Mantra separate from gained history; real Wrath/Calm modal rows",
 "amounts":"current public costs, signed stats, Retain and installed powers (including Slow, Time Warp, Curiosity, Artifact and Intangible); held/playing/Stasis custody",
 "actors":"actual half-dead and summoned enemy rows only; no hidden revive or summon slots",
 "sludge":"awa_sludge creates Void at BottomDraw",
 "terminal":"ongoing, death, empty_deck and act3_no_keys; no Heart victory",
 "cutoff":"Act4Ready act3 floor50 is healthy nonterminal zero actions after the actual Act3 ending with all keys; M24 stops before the Act4 door; no readiness bonus"
 }),watcher::groups(c),DOMAIN)?.descriptor();
 watcher::finalize_stance_descriptor(&mut d);
 let mut h=Sha256::new();h.update(DOMAIN);h.update(crate::canonical_json(&d).map_err(|_|RunSessionError::Input("invalid M24 descriptor"))?);Ok(RunObservationSpec{descriptor:d,fingerprint:h.finalize().into()})
}
pub(crate) fn card(id:CardId)->Result<i64,RunSessionError>{if matches!(id,CardId::Unknown(_)){return Err(RunSessionError::Input("unknown M24 card"))}named("card",id.as_str())}
pub(crate) fn relic(id:RelicId)->Result<i64,RunSessionError>{if matches!(id,RelicId::Unknown(_)){return Err(RunSessionError::Input("unknown M24 relic"))}named("relic",id.as_str())}
pub(crate) fn potion(id:PotionId)->Result<i64,RunSessionError>{named("potion",id.as_str())}
pub(crate) fn enemy(id:EnemyId)->Result<i64,RunSessionError>{named("enemy",id.as_str())}
pub(crate) fn encounter(id:EncounterId)->Result<i64,RunSessionError>{named("encounter",id.as_str())}
pub(crate) fn event(id:EventId)->Result<i64,RunSessionError>{named("event",id.as_str())}
pub(crate) fn room(id:RoomViewKind)->Result<i64,RunSessionError>{
 if matches!(id,RoomViewKind::Act2Ready|RoomViewKind::Act3Ready){return Err(RunSessionError::UnsupportedBoundary("M24 continues through Acts1-3"))}
 let n=v1::room(id)?;if n<1||n as usize>=vocab()["room"].as_array().unwrap().len(){return Err(RunSessionError::UnsupportedBoundary("M24 cannot encode Act4 rooms"))}Ok(n)
}
pub(crate) fn zone(name:&str)->Result<i64,RunSessionError>{named("zone",name)}
pub(crate) fn intent(name:&str)->Result<i64,RunSessionError>{named("intent",name)}
pub(crate) fn power(_:bool,name:&str)->Result<i64,RunSessionError>{named("power",name)}
pub(crate) fn profile(p:RoomScenario)->Result<i64,RunSessionError>{if p==RoomScenario::M24WatcherAct3{Ok(1)}else{Err(RunSessionError::Input("wrong M24 profile"))}}
pub(crate) fn choice(c:&RoomChoiceKind)->Result<i64,RunSessionError>{if let RoomChoiceKind::Card{id,upgrades}=c{permanent(*id,*upgrades)?;}Ok(v1::choice(c))}
pub(crate) fn operation(s:&SelectionView)->Result<i64,RunSessionError>{let n=watcher::operation(s)?;let d=watcher::vocabulary();named("selection_operation",d["selection_operation"][n as usize].as_str().unwrap())}
pub(crate) fn select_permanent_operation()->i64{named("selection_operation","select_permanent").expect("shared operation")}
fn permanent(id:CardId,upgrades:u32)->Result<(),RunSessionError>{if !M24_WATCHER_ACT3_CARDS.contains(&id)||upgrades>u32::from(id!=CardId::Parasite){return Err(RunSessionError::Input("unsupported M24 permanent card"))}Ok(())}
fn visible(c:&LiveCardView)->Result<(),RunSessionError>{if matches!(c.origin,CombatCardRef::Permanent(_)){permanent(c.id,c.upgrades)}else{card(c.id)?;if c.upgrades>u32::from(!matches!(c.id,CardId::Dazed|CardId::Slimed|CardId::Wound|CardId::Void|CardId::Parasite)){return Err(RunSessionError::Input("unsupported M24 generated card"))}Ok(())}}
pub(crate) fn validate_public_decision(d:&PublicDecision)->Result<(),RunSessionError>{
 if d.content_profile!=Some(RoomScenario::M24WatcherAct3)||d.player.character!=Character::Watcher{return Err(RunSessionError::Input("M24 encoder requires Watcher Acts1-3"))}
 let floors=match d.act{1=>d.floor<=17,2=>(17..=34).contains(&d.floor),3=>(34..=50).contains(&d.floor),_=>false};
 if !floors||matches!(d.terminal,Some(RunTerminal::HeartVictory)){return Err(RunSessionError::UnsupportedBoundary("outside M24 Acts1-3 boundary"))}
 if let Some(r)=&d.room{
  if matches!(r.kind,RoomViewKind::Act2Ready|RoomViewKind::Act3Ready){return Err(RunSessionError::UnsupportedBoundary("M24 continues through Acts1-3"))}
  if r.kind==RoomViewKind::Act4Ready&&(d.act!=3||d.floor!=50||d.player.hp==0||d.terminal.is_some()||!d.candidates.is_empty()||d.combat.is_some()||!r.choices.is_empty()){return Err(RunSessionError::UnsupportedBoundary("M24 Act4Ready must be healthy at 3/50 with no actions"))}
 }
 for c in d.objects.iter().chain(d.selection.iter().flat_map(|s|&s.eligible)).chain(d.candidates.iter().filter_map(|c|c.object.as_ref())){permanent(c.id,c.upgrades)?;}
 if let Some(last)=&d.last_combat{if !playable_encounters().contains(&last.encounter)||last.outcome==Outcome::Escaped{return Err(RunSessionError::Input("unsupported M24 combat record"))}}
 if let Some(c)=&d.combat{
  if c.stance_choice.is_some()&&(c.selection.is_some()||c.card_offer.is_some()||d.selection.is_some()){return Err(RunSessionError::Input("ambiguous M24 modal owners"))}
  if c.outcome==Some(Outcome::Escaped){return Err(RunSessionError::Input("unsupported M24 combat"))}
  for card in c.hand.iter().chain(&c.discard).chain(&c.exhaust).chain(&c.played_powers).chain(&c.playing_cards).chain(c.stasis.iter().map(|s|&s.card)).chain(c.known_draw.iter().map(|s|&s.card)).chain(c.selection.iter().flat_map(|s|&s.candidates)){visible(card)?;}
 }
 Ok(())
}

#[cfg(test)]
pub(crate) fn diagnostic_spec(c:&RunCapacities,start:crate::run_session::M24DiagnosticStart)->Result<RunObservationSpec,RunSessionError>{
 let mut d=m24_run_observation_spec(c)?.descriptor();d["semantics"]["diagnostic"]=json!({"fixture":format!("{start:?}"),"provenance":"constructed/supplied actual core owner (spire_core::run::diagnostics m24); internal SAME fixed initializer only; not normal portable replay"});
 let mut h=Sha256::new();h.update(b"spire-gym/run-schema/m24-watcher-act3/diagnostic/v1\0");h.update(crate::canonical_json(&d).map_err(|_|RunSessionError::Input("invalid diagnostic descriptor"))?);Ok(RunObservationSpec{descriptor:d,fingerprint:h.finalize().into()})
}
