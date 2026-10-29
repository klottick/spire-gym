//! Additive full-blue Heart vocabulary on the shared Run descriptor and encoder.
use crate::{run_schema as v1, run_schema_m10 as m10, run_schema_m15 as m15,
    run_schema_m16 as defect, RunCapacities, RunObservationSpec, RunSessionError};
use serde_json::{json, Value};
use spire_core::{combat::{CombatCardRef, LiveCardView}, content::*, run::*};
use std::sync::OnceLock;

pub const RUN_OBSERVATION_M20_SCHEMA_ID: &str = "spire-gym-m20-defect-heart-observation-v1";
fn append(v: &mut Value, key: &str, names: impl IntoIterator<Item = &'static str>) {
    let rows = v[key].as_array_mut().expect("shared vocabulary");
    for name in names { if !rows.iter().any(|row| row.as_str() == Some(name)) { rows.push(json!(name)); } }
}
fn shared_descriptor()->&'static Value{
    static SHARED:OnceLock<Value>=OnceLock::new();
    SHARED.get_or_init(||m15::m15_run_observation_spec(&RunCapacities::m10_default()).expect("fixed M15 descriptor").descriptor())
}
fn vocabulary() -> Value {
    let caps = RunCapacities::m10_default();
    let mut v = shared_descriptor()["vocabularies"].clone();
    let shared = defect::defect_combat_observation_spec(&caps).expect("fixed Defect descriptor").descriptor()["vocabularies"].clone();
    for key in ["power", "intent"] {
        let rows = v[key].as_array_mut().expect("shared names");
        for row in shared[key].as_array().expect("Defect names") { if !rows.contains(row) { rows.push(row.clone()); } }
    }
    let m15_cards = v["card"].as_array().expect("M15 cards");
    let extras = shared_descriptor()["semantics"]["entitlements"]["generated_and_status_cards"].clone();
    let cards: Vec<Value> = std::iter::once(json!("padding"))
        .chain(M20_DEFECT_HEART_CARDS.iter().map(|id| json!(id.as_str())))
        .chain(extras.as_array().expect("generated closure").iter().filter(|id| id.as_str() != Some("shiv") && m15_cards.contains(id)).cloned()).collect();
    v["card"] = json!(cards);
    v["relic"] = json!(std::iter::once("padding").chain(M20_DEFECT_HEART_RELICS.iter().map(|id|id.as_str())).collect::<Vec<_>>());
    v["potion"] = json!(std::iter::once("padding").chain(M20_DEFECT_HEART_POTIONS.iter().map(|id|id.as_str())).collect::<Vec<_>>());
    v["orb"] = shared["orb"].clone();
    v["profile"] = json!(["padding", "M20DefectHeart"]);
    // Keep shared enemy/potion/colorless powers, excluding historical class-only rows.
    const FOREIGN_POWERS:&[&str]=&["berserk","brutality","combust","corruption","dark_embrace","demon_form","double_tap","evolve","feel_no_pain","fire_breathing","flame_barrier","juggernaut","no_draw","rage","rupture","a_thousand_cuts","after_image","blur","burst","choked","corpse_explosion","double_damage","draw_card_next_turn","energized","envenom","infinite_blades","intangible_player","next_turn_block","phantasmal","tools_of_the_trade","wraith_form"];
    v["power"].as_array_mut().unwrap().retain(|name|!FOREIGN_POWERS.contains(&name.as_str().unwrap()));
    v["selection_operation"].as_array_mut().unwrap().retain(|name|!matches!(name.as_str(),Some("nightmare_capture"|"retain"|"setup")));
    append(&mut v,"selection_operation",["hologram_retrieve","seek_retrieve","recycle"]);
    append(&mut v, "power", ["buffer", "repair", "draw", "energized_blue", "hello_world", "creative_ai", "rebound", "equilibrium"]);
    v
}
fn vocab() -> &'static Value { static V: OnceLock<Value> = OnceLock::new(); V.get_or_init(vocabulary) }
fn named(key: &str, name: &str) -> Result<i64, RunSessionError> {
    vocab()[key].as_array().and_then(|rows|rows.iter().position(|row|row.as_str()==Some(name)))
        .filter(|index|*index>0).map(|index|index as i64).ok_or(RunSessionError::Input("unsupported M20 public identity"))
}
pub(crate) fn groups(c: &RunCapacities) -> Vec<v1::GroupSpec> {
    let mut rows = defect::groups(c);
    rows.extend(m15::groups(c).into_iter().filter(|row|row.key == "combat_card_offers")); rows
}
pub(crate) fn validate(c: &RunCapacities) -> Result<(), RunSessionError> { m10::validate_groups(&groups(c), c) }
pub(crate) fn finite_content() -> Value {
    let d = shared_descriptor();
    json!({"permanent_cards":M20_DEFECT_HEART_CARDS.iter().map(|id|id.as_str()).collect::<Vec<_>>(),
        "generated_and_status_cards":vocab()["card"].as_array().unwrap()[1+M20_DEFECT_HEART_CARDS.len()..],
        "relics":M20_DEFECT_HEART_RELICS.iter().map(|id|id.as_str()).collect::<Vec<_>>(),
        "potions":M20_DEFECT_HEART_POTIONS.iter().map(|id|id.as_str()).collect::<Vec<_>>(),
        "playable_enemies":d["semantics"]["enemy_species"]["cumulative_through_act4"],
        "playable_encounters":d["semantics"]["encounters"]["playable"],
        "act4_encounters":d["semantics"]["encounters"]["act4"],"map_only_boss_icons":[]})
}
pub fn m20_run_observation_spec(c: &RunCapacities) -> Result<RunObservationSpec, RunSessionError> {
    validate(c)?;
    m10::spec_from_vocabulary(c, RUN_OBSERVATION_M20_SCHEMA_ID, "M20DefectHeart", vocab().clone(), json!({
        "padding":"zero,known0,occupied0", "unknown_reference":"minus_one,known0 in occupied rows",
        "counts":"only published public counts; unknown draw cards have no rows",
        "cards":"public signed misc, costs, damage, Block, free play and temporary costs",
        "orbs":"logical slots through ten; known empty differs from padding; signed Dark damage known only for Dark",
        "offers":"current public card offers only, with zero-based indexes",
        "actions":"row i is PublicDecision.candidates[i]; no filtering, truncation or paging",
        "finite_content":finite_content(), "ending":"HeartVictory only after actual final Proceed",
        "scope":"finite Defect A0 Acts1–4; native source only"
    }), groups(c), b"spire-gym/run-schema/m20-defect-heart/v1\0")
}
pub(crate) fn card(id: CardId) -> Result<i64, RunSessionError> {
    if matches!(id,CardId::Unknown(_)){return Err(RunSessionError::Input("unknown M20 card"))} named("card",id.as_str())
}
pub(crate) fn relic(id: RelicId) -> Result<i64, RunSessionError> {
    if matches!(id,RelicId::Unknown(_)){return Err(RunSessionError::Input("unknown M20 relic"))} named("relic",id.as_str())
}
pub(crate) fn potion(id: PotionId) -> Result<i64, RunSessionError> { named("potion",id.as_str()) }
pub(crate) fn enemy(id: EnemyId) -> Result<i64, RunSessionError> { m15::enemy(id) }
pub(crate) fn encounter(id: EncounterId) -> Result<i64, RunSessionError> { m15::encounter(id) }
pub(crate) fn event(id: EventId) -> Result<i64, RunSessionError> { m15::event(id) }
pub(crate) fn room(id: RoomViewKind) -> Result<i64, RunSessionError> { m15::room(id) }
pub(crate) fn intent(name: &str) -> Result<i64, RunSessionError> { named("intent",name) }
pub(crate) fn zone(name: &str) -> Result<i64, RunSessionError> { m15::zone(name) }
pub(crate) fn power(_: bool, name: &str) -> Result<i64, RunSessionError> { named("power",name) }
pub(crate) fn operation(s: &spire_core::combat::SelectionView) -> Result<i64, RunSessionError> {
    use spire_core::combat::SelectionOperation;
    let name=match s.operation{
        SelectionOperation::HologramRetrieve=>"hologram_retrieve",
        SelectionOperation::SeekRetrieve=>"seek_retrieve",
        SelectionOperation::Recycle=>"recycle",
        SelectionOperation::NightmareCapture|SelectionOperation::Retain|SelectionOperation::Setup=>return Err(RunSessionError::Input("unsupported M20 selection operation")),
        _=>{
            let index=m15::operation(s)? as usize;
            let descriptor=shared_descriptor();
            return named("selection_operation",descriptor["vocabularies"]["selection_operation"][index].as_str().expect("shared operation name"));
        }
    };
    named("selection_operation",name)
}
pub(crate) fn select_permanent_operation() -> i64 { named("selection_operation","select_permanent").expect("M20 permanent selection") }
pub(crate) fn profile(p: RoomScenario) -> Result<i64, RunSessionError> {
    if p == RoomScenario::M20DefectHeart { Ok(1) } else { Err(RunSessionError::Input("profile is not representable in the M20 schema")) }
}
fn permanent(id: CardId, upgrades: u32) -> Result<(), RunSessionError> {
    if !M20_DEFECT_HEART_CARDS.contains(&id) || upgrades > u32::from(id != CardId::Parasite) {
        return Err(RunSessionError::Input("unsupported M20 permanent card"))
    } Ok(())
}
pub(crate) fn choice(c: &RoomChoiceKind) -> Result<i64, RunSessionError> {
    if let RoomChoiceKind::Card{id,upgrades}=c {permanent(*id,*upgrades)?;} m15::choice(c)
}
fn visible(c: &LiveCardView) -> Result<(), RunSessionError> {
    if matches!(c.origin, CombatCardRef::Permanent(_)) { return permanent(c.id,c.upgrades) }
    card(c.id)?;
    if c.id == CardId::Parasite || c.upgrades > u32::from(!matches!(c.id,CardId::Dazed|CardId::Slimed|CardId::Wound|CardId::Void)) {
        return Err(RunSessionError::Input("unsupported M20 generated card"))
    } Ok(())
}
pub(crate) fn validate_public_decision(d: &PublicDecision) -> Result<(), RunSessionError> {
    validate_decision(d,permanent,visible)
}
fn validate_decision(d:&PublicDecision, permanent:fn(CardId,u32)->Result<(),RunSessionError>,visible:fn(&LiveCardView)->Result<(),RunSessionError>)->Result<(),RunSessionError>{
    if d.content_profile != Some(RoomScenario::M20DefectHeart) || d.player.character != Character::Defect {
        return Err(RunSessionError::Input("M20 encoder requires the Defect Heart profile"))
    }
    if d.act>4 || d.floor>55 || d.room.as_ref().is_some_and(|r|matches!(r.kind,RoomViewKind::Act2Ready|RoomViewKind::Act3Ready)) {
        return Err(RunSessionError::UnsupportedBoundary("M20 requires the continuing Acts1–4 profile"))
    }
    if d.terminal.is_none() && d.candidates.is_empty(){
        return Err(RunSessionError::UnsupportedBoundary("nonterminal M20 has no legal actions"))
    }
    for c in d.objects.iter().chain(d.candidates.iter().filter_map(|c|c.object.as_ref())).chain(d.selection.iter().flat_map(|s|&s.eligible)) {permanent(c.id,c.upgrades)?;}
    for id in &d.player.relics {relic(*id)?;} for c in &d.player.relic_counters {relic(c.relic)?;}
    for id in d.player.potions.iter().flatten() {potion(*id)?;}
    if let Some(r) = &d.last_combat {encounter(r.encounter)?;}
    if let Some(c) = &d.combat {
        const PLAYER_POWERS:&[&str]=&["strength","dexterity","weak","vulnerable","frail","artifact","ritual","hex","confused","entangled","regeneration","thorns","constricted","draw_reduction","surrounded",
            "magnetism","mayhem","no_block","panache","sadistic_nature","lose_strength","lose_dexterity","plated_armor","duplication","panache_countdown","the_bomb","the_bomb_plus",
            "focus","bias","loop","electrodynamics","static_discharge","storm","heatsinks","amplify","echo_form","buffer","repair","draw","hello_world","creative_ai","energized_blue","equilibrium","rebound"];
        for p in &c.player.powers{if !PLAYER_POWERS.contains(&p.id.0){return Err(RunSessionError::Input("unsupported M20 player power"))}}
        if c.character.orbs.len()>10 {return Err(RunSessionError::Input("M20 orb slots exceed gameplay cap10"))}
        if let Some(i)=&c.inventory {
            for id in &i.relics {relic(*id)?;} for counter in &i.relic_counters {relic(counter.relic)?;}
            for id in i.potions.iter().flatten() {potion(*id)?;}
        }
        for card in c.hand.iter().chain(&c.discard).chain(&c.exhaust).chain(&c.played_powers).chain(&c.playing_cards)
            .chain(c.stasis.iter().map(|s|&s.card)).chain(c.known_draw.iter().map(|k|&k.card)).chain(c.selection.iter().flat_map(|s|&s.candidates)) {visible(card)?;}
    } Ok(())
}

#[cfg(test)]
pub(crate) const DIAGNOSTIC_SCHEMA_ID:&str="spire-gym-m20-defect-heart-diagnostic-observation-v1";
#[cfg(test)]
pub(crate) fn diagnostic_spec(c:&RunCapacities)->Result<RunObservationSpec,RunSessionError>{
    validate(c)?;
    let mut v=vocab().clone();v["card"].as_array_mut().unwrap().push(json!(CardId::SearingBlow.as_str()));
    m10::spec_from_vocabulary(c,DIAGNOSTIC_SCHEMA_ID,"M20DefectHeart",v,json!({
        "initialization":"fixed core diagnostic constructor only; synthetic prior source and keys",
        "replay":"internal same-initializer replay; ordinary portable replay cannot reconstruct supplied start",
        "normal_schema":RUN_OBSERVATION_M20_SCHEMA_ID
    }),groups(c),b"spire-gym/run-schema/m20-defect-heart/diagnostic/v1\0")
}
#[cfg(test)]
pub(crate) fn diagnostic_card(id:CardId)->Result<i64,RunSessionError>{
    if id==CardId::SearingBlow {Ok(vocab()["card"].as_array().unwrap().len() as i64)} else {card(id)}
}
#[cfg(test)]
fn diagnostic_permanent(id:CardId,upgrades:u32)->Result<(),RunSessionError>{
    if id==CardId::SearingBlow {Ok(())} else {permanent(id,upgrades)}
}
#[cfg(test)]
fn diagnostic_visible(c:&LiveCardView)->Result<(),RunSessionError>{
    if c.id==CardId::SearingBlow {diagnostic_permanent(c.id,c.upgrades)} else {visible(c)}
}
#[cfg(test)]
pub(crate) fn validate_diagnostic_decision(d:&PublicDecision)->Result<(),RunSessionError>{
    validate_decision(d,diagnostic_permanent,diagnostic_visible)
}
