//! Finite Watcher Act1 catalog. Producer rows come from the authoritative core owner.
use crate::{run_schema_m10 as m10,run_schema_m12 as act1,run_schema_m21 as watcher,RunCapacities,RunObservationSpec,RunSessionError};
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
use spire_core::{combat::{CombatCardRef,LiveCardView,Outcome,SelectionView},content::*,run::{*,m22_content as producers}};
use std::sync::OnceLock;
pub const RUN_OBSERVATION_M22_SCHEMA_ID:&str="spire-gym-m22-watcher-act1-observation-v1";
const DOMAIN:&[u8]=b"spire-gym/run-schema/m22-watcher-act1/v1\0";
const STATUSES:&[CardId]=&[CardId::Dazed,CardId::Burn,CardId::Slimed];
fn padded(names:impl IntoIterator<Item=&'static str>)->Value{json!(std::iter::once("padding").chain(names).collect::<Vec<_>>())}
fn vocabulary()->Value{
 let mut v=watcher::vocabulary();let a=act1::m12_run_observation_spec(&RunCapacities::m10_default()).expect("fixed Act1 vocabulary").descriptor()["vocabularies"].clone();
 for key in ["enemy","encounter","event","room","choice","intent"]{v[key]=a[key].clone();}
 v["card"]=padded(M22_WATCHER_ACT1_CARDS.iter().chain(spire_core::content::watcher::M21_WATCHER_GENERATED).chain(STATUSES).map(|id|id.as_str()));
 v["relic"]=padded(M22_WATCHER_ACT1_RELICS.iter().map(|id|id.as_str()));v["potion"]=padded(M22_WATCHER_ACT1_POTIONS.iter().map(|id|id.as_str()));v["profile"]=json!(["padding","M22WatcherAct1"]);
 // Act1 enemy/item powers plus the finite Watcher powers, without foreign class rows.
 let shared=v["power"].clone();v["power"]=json!(["padding","strength","dexterity","weak","vulnerable","frail","artifact","ritual","curl_up","split","enrage","metallicize","mode_shift","sharp_hide"]);
 for name in ["mantra","devotion","foresight","nirvana","establishment","master_reality","battle_hymn","mental_fortress","rushdown","like_water"]{let value=json!(name);assert!(shared.as_array().unwrap().contains(&value));if !v["power"].as_array().unwrap().contains(&value){v["power"].as_array_mut().unwrap().push(value);}}
 v
}
fn vocab()->&'static Value{static V:OnceLock<Value>=OnceLock::new();V.get_or_init(vocabulary)}
fn named(key:&str,name:&str)->Result<i64,RunSessionError>{vocab()[key].as_array().and_then(|a|a.iter().position(|v|v.as_str()==Some(name))).filter(|n|*n>0).map(|n|n as i64).ok_or(RunSessionError::Input("unsupported M22 public identity"))}
pub(crate) fn validate(c:&RunCapacities)->Result<(),RunSessionError>{m10::validate_groups(&watcher::groups(c),c)}
pub(crate) fn finite_content()->Value{
 let cards=|ids:&[CardId]|ids.iter().map(|id|id.as_str()).collect::<Vec<_>>();let relics=|ids:&[RelicId]|ids.iter().map(|id|id.as_str()).collect::<Vec<_>>();let potions=|ids:&[PotionId]|ids.iter().map(|id|id.as_str()).collect::<Vec<_>>();
 json!({"permanent_cards":cards(M22_WATCHER_ACT1_CARDS),"generated_cards":cards(spire_core::content::watcher::M21_WATCHER_GENERATED),"status_cards":cards(STATUSES),
 "relics":relics(M22_WATCHER_ACT1_RELICS),"potions":potions(M22_WATCHER_ACT1_POTIONS),
 "playable_encounters":act1::M12_PLAYABLE_ENCOUNTERS.iter().map(|id|id.as_str()).collect::<Vec<_>>(),"playable_enemies":act1::M12_PLAYABLE_ENEMIES.iter().map(|id|id.as_str()).collect::<Vec<_>>(),"map_only_boss_icons":act1::M12_MAP_ONLY_BOSS_ICONS.iter().map(|id|id.as_str()).collect::<Vec<_>>(),
 "card_taxonomy":M22_WATCHER_ACT1_CARDS.iter().map(|id|json!({"id":id.as_str(),"rarity":format!("{:?}",Catalog::standard().card(*id).expect("finite card metadata").rarity)})).collect::<Vec<_>>(),"relic_taxonomy":producers::RELIC_TAXONOMY.iter().map(|(id,tier)|json!({"id":id.as_str(),"tier":tier})).collect::<Vec<_>>(),"capture_law":{"scope":producers::RELIC_CAPTURE_SCOPE_NAME,"key_fields":producers::RELIC_CAPTURE_KEY_FIELDS,"elite_source_tag":producers::ELITE_SOURCE_TAG,"chest_source_tag":producers::CHEST_SOURCE_TAG,"elite_visit_identity":producers::ELITE_VISIT_IDENTITY,"chest_visit_identity":producers::CHEST_VISIT_IDENTITY,"filter_owned_before_sampling":producers::RELIC_CAPTURE_FILTER_OWNED,"empty_requests_rng":producers::RELIC_CAPTURE_EMPTY_REQUEST,"singleton_requests_rng":producers::RELIC_CAPTURE_SINGLETON_REQUEST,"draw_law":producers::RELIC_CAPTURE_DRAW_LAW},
 "producers":{"starter_cards":cards(Catalog::standard().starter_deck(Character::Watcher).expect("canonical Watcher starter")),"starter_relic":Catalog::standard().starter_relic(Character::Watcher).expect("canonical Watcher relic").as_str(),"starter_and_neow_relics":relics(&M22_WATCHER_ACT1_RELICS[..2]),"shop_relic_price":producers::SHOP_RELIC_PRICE,"shop_potion_price":producers::SHOP_POTION_PRICE,"owner":"spire_core::run::m22_content","nonboss_reward_rows":producers::NONBOSS_REWARD_ROWS.iter().map(|r|cards(r)).collect::<Vec<_>>(),"boss_card_rewards":cards(&producers::BOSS_CARD_REWARDS),"shop_card_rows":producers::SHOP_CARD_ROWS.iter().map(|r|cards(r)).collect::<Vec<_>>(),"shop_card_prices":producers::SHOP_CARD_PRICES,"elite_relics":relics(&producers::ELITE_RELICS),"chest_relics":relics(&producers::CHEST_RELICS),"boss_relics":relics(&producers::BOSS_RELICS),"potion_cycle":potions(&producers::POTION_CYCLE),"shop_relics":relics(&producers::SHOP_RELICS),"shop_potions":potions(&producers::SHOP_POTIONS)}})
}
pub fn m22_run_observation_spec(c:&RunCapacities)->Result<RunObservationSpec,RunSessionError>{
 validate(c)?;
 let mut d=m10::spec_from_vocabulary(c,RUN_OBSERVATION_M22_SCHEMA_ID,"M22WatcherAct1",vocab().clone(),json!({
 "finite_content":finite_content(),"padding":"zero,known0,occupied0; unavailable references minus_one,known0",
 "actions":"row i is native candidate i; no filtering, truncation or reordering",
 "public_order":"Scry reveal top-first; selected ordinal one-based append/remove order; unrevealed draw, RNG, AI, families and cursors absent",
 "stances":"Neutral0 Wrath1 Calm2 Divinity3; current Mantra separate from gained history; real Wrath/Calm modal rows",
 "amounts":"current public costs, signed stats, Retain and installed powers; held/playing custody",
 "cutoff":"Act2Ready act2 floor17 is healthy nonterminal zero actions; no readiness bonus"
 }),watcher::groups(c),DOMAIN)?.descriptor();
 watcher::finalize_stance_descriptor(&mut d);
 let mut h=Sha256::new();h.update(DOMAIN);h.update(crate::canonical_json(&d).map_err(|_|RunSessionError::Input("invalid M22 descriptor"))?);Ok(RunObservationSpec{descriptor:d,fingerprint:h.finalize().into()})
}
pub(crate) fn card(id:CardId)->Result<i64,RunSessionError>{if matches!(id,CardId::Unknown(_)){return Err(RunSessionError::Input("unknown M22 card"))}named("card",id.as_str())}
pub(crate) fn relic(id:RelicId)->Result<i64,RunSessionError>{if matches!(id,RelicId::Unknown(_)){return Err(RunSessionError::Input("unknown M22 relic"))}named("relic",id.as_str())}
pub(crate) fn potion(id:PotionId)->Result<i64,RunSessionError>{named("potion",id.as_str())}
pub(crate) fn enemy(id:EnemyId)->Result<i64,RunSessionError>{named("enemy",id.as_str())}
pub(crate) fn encounter(id:EncounterId)->Result<i64,RunSessionError>{named("encounter",id.as_str())}
pub(crate) fn event(id:EventId)->Result<i64,RunSessionError>{named("event",id.as_str())}
pub(crate) fn room(id:RoomViewKind)->Result<i64,RunSessionError>{act1::room(id)}
pub(crate) fn zone(name:&str)->Result<i64,RunSessionError>{named("zone",name)}
pub(crate) fn intent(name:&str)->Result<i64,RunSessionError>{named("intent",name)}
pub(crate) fn power(_:bool,name:&str)->Result<i64,RunSessionError>{named("power",name)}
pub(crate) fn profile(p:RoomScenario)->Result<i64,RunSessionError>{if p==RoomScenario::M22WatcherAct1{Ok(1)}else{Err(RunSessionError::Input("wrong M22 profile"))}}
pub(crate) fn choice(c:&RoomChoiceKind)->Result<i64,RunSessionError>{if let RoomChoiceKind::Card{id,upgrades}=c{permanent(*id,*upgrades)?;}Ok(m10::choice(c))}
pub(crate) fn operation(s:&SelectionView)->Result<i64,RunSessionError>{let n=watcher::operation(s)?;let d=watcher::vocabulary();named("selection_operation",d["selection_operation"][n as usize].as_str().unwrap())}
pub(crate) fn select_permanent_operation()->i64{named("selection_operation","select_permanent").expect("shared operation")}
fn permanent(id:CardId,upgrades:u32)->Result<(),RunSessionError>{if !M22_WATCHER_ACT1_CARDS.contains(&id)||upgrades>1{return Err(RunSessionError::Input("unsupported M22 permanent card"))}Ok(())}
fn visible(c:&LiveCardView)->Result<(),RunSessionError>{if matches!(c.origin,CombatCardRef::Permanent(_)){permanent(c.id,c.upgrades)}else{card(c.id)?;if c.upgrades>u32::from(!matches!(c.id,CardId::Dazed|CardId::Slimed)){return Err(RunSessionError::Input("unsupported M22 generated upgrade"))}Ok(())}}
pub(crate) fn validate_public_decision(d:&PublicDecision)->Result<(),RunSessionError>{
 if d.content_profile!=Some(RoomScenario::M22WatcherAct1)||d.player.character!=Character::Watcher{return Err(RunSessionError::Input("M22 encoder requires Watcher Act1"))}
 if d.floor>17||(!matches!(d.act,1|2))||(d.room.as_ref().is_some_and(|r|r.kind==RoomViewKind::Act2Ready)&&!(d.act==2&&d.floor==17&&d.terminal.is_none()&&d.candidates.is_empty()))||(d.act==2&&!d.room.as_ref().is_some_and(|r|r.kind==RoomViewKind::Act2Ready))||matches!(d.terminal,Some(RunTerminal::Act3NoKeys|RunTerminal::HeartVictory)){return Err(RunSessionError::UnsupportedBoundary("outside M22 Act1 boundary"))}
 for c in d.objects.iter().chain(d.selection.iter().flat_map(|s|&s.eligible)).chain(d.candidates.iter().filter_map(|c|c.object.as_ref())){permanent(c.id,c.upgrades)?;}
 if let Some(last)=&d.last_combat{if !act1::M12_PLAYABLE_ENCOUNTERS.contains(&last.encounter)||last.outcome==Outcome::Escaped{return Err(RunSessionError::Input("M22 map-only icon is not playable"))}}
 if let Some(c)=&d.combat{
  if c.stance_choice.is_some()&&(c.selection.is_some()||c.card_offer.is_some()||d.selection.is_some()){return Err(RunSessionError::Input("ambiguous M22 modal owners"))}
  if c.outcome==Some(Outcome::Escaped){return Err(RunSessionError::Input("unsupported M22 combat"))}
  for card in c.hand.iter().chain(&c.discard).chain(&c.exhaust).chain(&c.played_powers).chain(&c.playing_cards).chain(c.stasis.iter().map(|s|&s.card)).chain(c.known_draw.iter().map(|s|&s.card)).chain(c.selection.iter().flat_map(|s|&s.candidates)){visible(card)?;}
 }
 Ok(())
}

#[cfg(test)]
pub(crate) fn diagnostic_spec(c:&RunCapacities,fixture:spire_core::run::diagnostics::M22Diagnostic)->Result<RunObservationSpec,RunSessionError>{
 let mut d=m22_run_observation_spec(c)?.descriptor();d["semantics"]["diagnostic"]=json!({"fixture":format!("{fixture:?}"),"provenance":"constructed/supplied actual core owner; internal SAME fixed initializer only; not normal portable replay"});
 let mut h=Sha256::new();h.update(b"spire-gym/run-schema/m22-watcher-act1/diagnostic/v1\0");h.update(crate::canonical_json(&d).map_err(|_|RunSessionError::Input("invalid diagnostic descriptor"))?);Ok(RunObservationSpec{descriptor:d,fingerprint:h.finalize().into()})
}
