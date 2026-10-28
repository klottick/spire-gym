//! Versioned semantic Run replay; no snapshots or simulator implementation here.
use spire_core::{combat::{CombatAction,TargetId,rng::Partition},content::Character,run::{*,map::PublicMap}};
use crate::{RunEpisodeRecord,RunSessionError,RunCapacities,RunRecordedFault,RunRecordedStep,RunSessionConfig,RunBossHistory,RunRecordErrorKind};
use crate::run_session::{create_run,validate_config,RunProfile,RUN_SEED_ADAPTER_ID,RUN_REWARD_ID};
use serde::{de::{self, MapAccess, SeqAccess, Visitor}, Deserialize, Deserializer};
use serde_json::{Map, Number, Value, json};
use std::fmt;
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct RunReplayReport{
    /// Latest readable boundary; final_decision_error discloses a faulted current owner.
    pub final_decision:PublicDecision,pub final_decision_error:Option<RunError>,pub final_map:Option<PublicMap>,
    /// Completed successful semantic steps; a partial fault attempt is separate.
    pub accepted_count:usize,pub rewards:Vec<i32>,pub terminals:Vec<Option<RunTerminal>>,pub reported_fault:Option<RunRecordedFault>,
}
fn incompatible()->RunSessionError{RunSessionError::IncompatibleReplay("invalid Run replay value")}

/// Parse raw JSON while rejecting duplicate object keys before serde_json::Value
/// can collapse them. This is shared across profiles because a duplicate field
/// has no unambiguous semantic replay meaning.
struct UniqueJson(Value);
struct UniqueJsonVisitor;
impl<'de> Deserialize<'de> for UniqueJson {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(UniqueJsonVisitor)
    }
}
impl<'de> Visitor<'de> for UniqueJsonVisitor {
    type Value = UniqueJson;
    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a JSON value with unique object keys")
    }
    fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(UniqueJson(Value::Bool(value)))
    }
    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(UniqueJson(Value::Number(value.into())))
    }
    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(UniqueJson(Value::Number(value.into())))
    }
    fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Number::from_f64(value)
            .map(|number| UniqueJson(Value::Number(number)))
            .ok_or_else(|| E::custom("non-finite JSON number"))
    }
    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(UniqueJson(Value::String(value.to_owned())))
    }
    fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(UniqueJson(Value::String(value)))
    }
    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(UniqueJson(Value::Null))
    }
    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(UniqueJson(Value::Null))
    }
    fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut values = Vec::new();
        while let Some(UniqueJson(value)) = sequence.next_element()? {
            values.push(value);
        }
        Ok(UniqueJson(Value::Array(values)))
    }
    fn visit_map<A>(self, mut object: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut values = Map::new();
        while let Some(key) = object.next_key::<String>()? {
            if values.contains_key(&key) {
                return Err(de::Error::custom("duplicate JSON object key"));
            }
            let UniqueJson(value) = object.next_value()?;
            values.insert(key, value);
        }
        Ok(UniqueJson(Value::Object(values)))
    }
}
pub(crate) fn parse_unique_json(bytes: &[u8]) -> Result<Value, RunSessionError> {
    let mut deserializer = serde_json::Deserializer::from_slice(bytes);
    let UniqueJson(value) = UniqueJson::deserialize(&mut deserializer).map_err(|_| incompatible())?;
    deserializer.end().map_err(|_| incompatible())?;
    Ok(value)
}
fn exact_fields(value: &Value, expected: &[&str]) -> Result<(), RunSessionError> {
    let object = value.as_object().ok_or_else(incompatible)?;
    if object.len() != expected.len() || expected.iter().any(|field| !object.contains_key(*field)) {
        return Err(incompatible());
    }
    Ok(())
}
fn exact_action_fields(action: &Value) -> Result<(), RunSessionError> {
    let fields: &[&str] = match text(&action["verb"])? {
        "move" => &["verb", "node"],
        "choose" => &["verb", "choice"],
        "select" => &["verb", "selection"],
        "confirm" | "cancel" | "combat.skip_card_offer" | "combat.confirm_selection" | "combat.end_turn" => &["verb"],
        "use_potion" | "combat.use_potion" => &["verb", "slot", "target"],
        "discard_potion" => &["verb", "slot"],
        "combat.choose_card_offer" => &["verb", "index"],
        "combat.play_card" => &["verb", "instance", "target"],
        "combat.select_card" => &["verb", "instance"],
        _ => return Err(incompatible()),
    };
    exact_fields(action, fields)
}
fn strict_profile_json_shape(value: &Value,history_key:&str,acts:&[&str]) -> Result<(), RunSessionError> {
    exact_fields(value, &[
        "schema_id", "profile_id", "rules_id", "content_id", "seed_adapter_id",
        "rng_id", "reward_id", "observation_fingerprint", "config", "seed",
        "ordinal", "steps", "terminal", "fault", "closed",
    ])?;
    let config = &value["config"];
    exact_fields(config, &[
        "character", "ascension", "partition", "boss_history", "capacities", "record_replay",
    ])?;
    exact_fields(&config["capacities"], &[
        "map_nodes", "map_edges", "permanent_cards", "combat_cards", "enemies", "powers",
        "relics", "potions", "offers", "selection", "orbs", "actions",
    ])?;
    exact_fields(&config["boss_history"], &[history_key])?;
    exact_fields(&config["boss_history"][history_key], acts)?;
    for step in array(&value["steps"])? {
        exact_fields(step, &["action", "reward", "terminal"])?;
        exact_action_fields(&step["action"])?;
    }
    if !value["fault"].is_null() {
        let fault = &value["fault"];
        exact_fields(fault, &["action", "committed", "gameplay_recorded", "error"])?;
        if !fault["action"].is_null() {
            exact_action_fields(&fault["action"])?;
        }
        let error = &fault["error"];
        exact_fields(error, &["kind", "message", "capacity"])?;
        if !error["capacity"].is_null() {
            let capacity = &error["capacity"];
            exact_fields(capacity, &["group", "required", "capacity"])?;
            // Encoder-derived rows share the configured capacity of their
            // owning group. Diagnostics retain their emitted row-group name;
            // this codec check does not claim that a synthetic fault is reachable
            // from a default-capacity session (or rerun gameplay to prove it).
            let capacity_owner=match text(&capacity["group"])? {
                "combat_card_offers"=>"offers",
                "known_draw"=>"combat_cards",
                group=>group,
            };
            if !matches!(
                capacity_owner ,
                "map_nodes" | "map_edges" | "permanent_cards" | "combat_cards" | "enemies"
                    | "powers" | "relics" | "potions" | "offers" | "selection" | "orbs" | "actions"
            ) || u(&capacity["required"])? <= u(&capacity["capacity"])? {
                return Err(incompatible());
            }
        }
    }
    Ok(())
}
fn u(v:&Value)->Result<u64,RunSessionError>{v.as_u64().ok_or_else(incompatible)}
fn size(v:&Value)->Result<usize,RunSessionError>{usize::try_from(u(v)?).map_err(|_|incompatible())}
fn text(v:&Value)->Result<&str,RunSessionError>{v.as_str().ok_or_else(incompatible)}
fn flag(v:&Value)->Result<bool,RunSessionError>{v.as_bool().ok_or_else(incompatible)}
fn array(v:&Value)->Result<&Vec<Value>,RunSessionError>{v.as_array().ok_or_else(incompatible)}
fn term(t:Option<RunTerminal>)->Value{match t{None=>Value::Null,Some(t)=>json!(match t{RunTerminal::Death=>"death",RunTerminal::EmptyDeck=>"empty_deck",RunTerminal::Act3NoKeys=>"act3_no_keys",RunTerminal::HeartVictory=>"heart_victory"})}}
fn parse_term(v:&Value)->Result<Option<RunTerminal>,RunSessionError>{if v.is_null(){return Ok(None)}Ok(Some(match text(v)?{"death"=>RunTerminal::Death,"empty_deck"=>RunTerminal::EmptyDeck,"act3_no_keys"=>RunTerminal::Act3NoKeys,"heart_victory"=>RunTerminal::HeartVictory,_=>return Err(incompatible())}))}
fn action(a:&GameAction,profile:RunProfile)->Value{match a{
    GameAction::Move(id)=>json!({"verb":"move","node":id.0}),GameAction::Choose(id)=>json!({"verb":"choose","choice":id.0}),GameAction::Select(id)=>json!({"verb":"select","selection":id.0}),GameAction::Confirm=>json!({"verb":"confirm"}),GameAction::Cancel=>json!({"verb":"cancel"}),
    GameAction::UsePotion{slot,target}=>json!({"verb":"use_potion","slot":slot,"target":target.map(|t|t.0)}),GameAction::DiscardPotion{slot}=>json!({"verb":"discard_potion","slot":slot}),
      GameAction::Combat(a)=>match a{CombatAction::ChooseCardOffer{index}=>json!({"verb":"combat.choose_card_offer","index":index}),CombatAction::SkipCardOffer=>json!({"verb":"combat.skip_card_offer"}),CombatAction::PlayCard{instance,target}=>json!({"verb":"combat.play_card","instance":instance,"target":target.map(|t|t.0)}),CombatAction::SelectCard{instance}=>json!({"verb":"combat.select_card","instance":instance}),CombatAction::ConfirmSelection if profile!=RunProfile::M9IroncladAct3=>json!({"verb":"combat.confirm_selection"}),CombatAction::ConfirmSelection=>Value::Null,CombatAction::UsePotion{slot,target}=>json!({"verb":"combat.use_potion","slot":slot,"target":target.map(|t|t.0)}),CombatAction::EndTurn=>json!({"verb":"combat.end_turn"})},
 }}
fn parse_action(v:&Value,profile:RunProfile)->Result<GameAction,RunSessionError>{
    let verb = text(&v["verb"])?;
    if verb == "combat.confirm_selection" {
        return if profile != RunProfile::M9IroncladAct3 {
            Ok(GameAction::Combat(CombatAction::ConfirmSelection))
        } else {
            Err(RunSessionError::IncompatibleReplay("ConfirmSelection is not representable in replay v1"))
        };
    }
    if verb == "combat.choose_card_offer" {
        return if profile == RunProfile::M15SilentHeart {
            Ok(GameAction::Combat(CombatAction::ChooseCardOffer {
                index: u32::try_from(u(&v["index"])?).map_err(|_| incompatible())?,
            }))
        } else {
            Err(incompatible())
        };
    }
    if verb == "combat.skip_card_offer" {
        return if profile == RunProfile::M15SilentHeart {
            Ok(GameAction::Combat(CombatAction::SkipCardOffer))
        } else {
            Err(incompatible())
        };
    }
    let target=||if v["target"].is_null(){Ok(None)}else{Ok(Some(TargetId(u(&v["target"])?)))};
    Ok(match verb {
        "move"=>GameAction::Move(NodeId(u(&v["node"])?)),"choose"=>GameAction::Choose(ChoiceId(u(&v["choice"])?)),"select"=>GameAction::Select(SelectionId(u(&v["selection"])?)),"confirm"=>GameAction::Confirm,"cancel"=>GameAction::Cancel,
        "use_potion"=>GameAction::UsePotion{slot:u32::try_from(u(&v["slot"])?).map_err(|_|incompatible())?,target:target()?},"discard_potion"=>GameAction::DiscardPotion{slot:u32::try_from(u(&v["slot"])?).map_err(|_|incompatible())?},
        "combat.play_card"=>GameAction::Combat(CombatAction::PlayCard{instance:u(&v["instance"])? ,target:target()?}),"combat.select_card"=>GameAction::Combat(CombatAction::SelectCard{instance:u(&v["instance"])?}),"combat.use_potion"=>GameAction::Combat(CombatAction::UsePotion{slot:size(&v["slot"])? ,target:target()?}),"combat.end_turn"=>GameAction::Combat(CombatAction::EndTurn),_=>return Err(incompatible()),
    })
}
fn capacities(c:&RunCapacities)->Value{json!({"map_nodes":c.map_nodes,"map_edges":c.map_edges,"permanent_cards":c.permanent_cards,"combat_cards":c.combat_cards,"enemies":c.enemies,"powers":c.powers,"relics":c.relics,"potions":c.potions,"offers":c.offers,"selection":c.selection,"orbs":c.orbs,"actions":c.actions})}
fn parse_capacities(v:&Value)->Result<RunCapacities,RunSessionError>{Ok(RunCapacities{map_nodes:size(&v["map_nodes"])? ,map_edges:size(&v["map_edges"])? ,permanent_cards:size(&v["permanent_cards"])? ,combat_cards:size(&v["combat_cards"])? ,enemies:size(&v["enemies"])? ,powers:size(&v["powers"])? ,relics:size(&v["relics"])? ,potions:size(&v["potions"])? ,offers:size(&v["offers"])? ,selection:size(&v["selection"])? ,orbs:size(&v["orbs"])? ,actions:size(&v["actions"])?})}
fn boss_history(history: RunBossHistory) -> Value {
    match history {
        RunBossHistory::M18DefectAct2 { act1,act2 } => json!({"m18-defect-act2":{"act1":act1_history_name(act1),"act2":act2_history_name(act2)}}),
        RunBossHistory::M17DefectAct1 { act1 } => json!({"m17-defect-act1":{"act1":act1_history_name(act1)}}),
        RunBossHistory::AwakenedUnseen => json!("awakened-unseen"),
        RunBossHistory::TimeEaterUnseen => json!("time-eater-unseen"),
        RunBossHistory::DonuDecaUnseen => json!("donu-deca-unseen"),
        RunBossHistory::Act1GuardianUnseen => json!("act1-guardian-unseen"),
        RunBossHistory::Act1HexaghostUnseen => json!("act1-hexaghost-unseen"),
        RunBossHistory::Act1SlimeBossUnseen => json!("act1-slime-boss-unseen"),
        RunBossHistory::M13SilentAct2 { act1, act2 } => {
            let act1 = act1_history_name(act1);
            let act2 = act2_history_name(act2);
            json!({"m13-silent-act2":{"act1":act1,"act2":act2}})
        }
        RunBossHistory::M14SilentAct3 { act1, act2, act3 } => json!({
            "m14-silent-act3":{
                "act1":act1_history_name(act1),
                "act2":act2_history_name(act2),
                "act3":act3_history_name(act3)
            }
        }),
        RunBossHistory::M15SilentHeart { act1, act2, act3 } => json!({
            "m15-silent-heart":{
                "act1":act1_history_name(act1),
                "act2":act2_history_name(act2),
                "act3":act3_history_name(act3)
            }
        }),
    }
}
fn act1_history_name(target: crate::RunAct1BossTarget) -> &'static str {
    match target {
        crate::RunAct1BossTarget::Guardian => "guardian",
        crate::RunAct1BossTarget::Hexaghost => "hexaghost",
        crate::RunAct1BossTarget::SlimeBoss => "slime_boss",
    }
}
fn act2_history_name(target: crate::RunAct2BossTarget) -> &'static str {
    match target {
        crate::RunAct2BossTarget::Champ => "champ",
        crate::RunAct2BossTarget::BronzeAutomaton => "bronze_automaton",
        crate::RunAct2BossTarget::Collector => "collector",
    }
}
fn act3_history_name(target: crate::RunAct3BossTarget) -> &'static str {
    match target {
        crate::RunAct3BossTarget::AwakenedOne => "awakened-one",
        crate::RunAct3BossTarget::TimeEater => "time-eater",
        crate::RunAct3BossTarget::DonuAndDeca => "donu-and-deca",
    }
}
fn parse_boss_history(value: &Value) -> Result<RunBossHistory, RunSessionError> {
    if let Some(value) = value.as_str() {
        return Ok(match value {
            "awakened-unseen" => RunBossHistory::AwakenedUnseen,
            "time-eater-unseen" => RunBossHistory::TimeEaterUnseen,
            "donu-deca-unseen" => RunBossHistory::DonuDecaUnseen,
            "act1-guardian-unseen" => RunBossHistory::Act1GuardianUnseen,
            "act1-hexaghost-unseen" => RunBossHistory::Act1HexaghostUnseen,
            "act1-slime-boss-unseen" => RunBossHistory::Act1SlimeBossUnseen,
            _ => return Err(incompatible()),
        });
    }
    let Some(outer) = value.as_object() else { return Err(incompatible()) };
    if outer.len() != 1 { return Err(incompatible()) }
    if let Some(pair)=outer.get("m18-defect-act2").and_then(Value::as_object){
        if pair.len()!=2 || !pair.contains_key("act1") || !pair.contains_key("act2"){return Err(incompatible())}
        return Ok(RunBossHistory::M18DefectAct2{act1:parse_act1_history(&pair["act1"])?,act2:parse_act2_history(&pair["act2"])?});
    }
    if let Some(single)=outer.get("m17-defect-act1").and_then(Value::as_object){
        if single.len()!=1||!single.contains_key("act1"){return Err(incompatible())}
        return Ok(RunBossHistory::M17DefectAct1{act1:parse_act1_history(&single["act1"])?});
    }
    if let Some(pair) = outer.get("m13-silent-act2").and_then(Value::as_object) {
        if pair.len() != 2 || !pair.contains_key("act1") || !pair.contains_key("act2") {
            return Err(incompatible());
        }
        let act1 = parse_act1_history(pair.get("act1").ok_or_else(incompatible)?)?;
        let act2 = parse_act2_history(pair.get("act2").ok_or_else(incompatible)?)?;
        return Ok(RunBossHistory::M13SilentAct2 { act1, act2 });
    }
    if let Some(triple) = outer.get("m14-silent-act3").and_then(Value::as_object) {
        if triple.len() != 3 || !triple.contains_key("act1") || !triple.contains_key("act2") || !triple.contains_key("act3") {
            return Err(incompatible());
        }
        let act1 = parse_act1_history(triple.get("act1").ok_or_else(incompatible)?)?;
        let act2 = parse_act2_history(triple.get("act2").ok_or_else(incompatible)?)?;
        let act3 = parse_act3_history(triple.get("act3").ok_or_else(incompatible)?)?;
        return Ok(RunBossHistory::M14SilentAct3 { act1, act2, act3 });
    }
    if let Some(triple) = outer.get("m15-silent-heart").and_then(Value::as_object) {
        if triple.len() != 3 || !triple.contains_key("act1") || !triple.contains_key("act2") || !triple.contains_key("act3") {
            return Err(incompatible());
        }
        let act1 = parse_act1_history(triple.get("act1").ok_or_else(incompatible)?)?;
        let act2 = parse_act2_history(triple.get("act2").ok_or_else(incompatible)?)?;
        let act3 = parse_act3_history(triple.get("act3").ok_or_else(incompatible)?)?;
        return Ok(RunBossHistory::M15SilentHeart { act1, act2, act3 });
    }
    Err(incompatible())
}
fn parse_act3_history(value: &Value) -> Result<crate::RunAct3BossTarget, RunSessionError> {
    match text(value)? {
        "awakened-one" => Ok(crate::RunAct3BossTarget::AwakenedOne),
        "time-eater" => Ok(crate::RunAct3BossTarget::TimeEater),
        "donu-and-deca" => Ok(crate::RunAct3BossTarget::DonuAndDeca),
        _ => Err(incompatible()),
    }
}
fn parse_act1_history(value: &Value) -> Result<crate::RunAct1BossTarget, RunSessionError> {
    match text(value)? {
        "guardian" => Ok(crate::RunAct1BossTarget::Guardian),
        "hexaghost" => Ok(crate::RunAct1BossTarget::Hexaghost),
        "slime_boss" => Ok(crate::RunAct1BossTarget::SlimeBoss),
        _ => Err(incompatible()),
    }
}
fn parse_act2_history(value: &Value) -> Result<crate::RunAct2BossTarget, RunSessionError> {
    match text(value)? {
        "champ" => Ok(crate::RunAct2BossTarget::Champ),
        "bronze_automaton" => Ok(crate::RunAct2BossTarget::BronzeAutomaton),
        "collector" => Ok(crate::RunAct2BossTarget::Collector),
        _ => Err(incompatible()),
    }
}
fn config(c:&RunSessionConfig)->Value{json!({"character":c.character.as_str(),"ascension":c.ascension,"partition":c.partition.as_str(),"boss_history":boss_history(c.boss_history),"capacities":capacities(&c.capacities),"record_replay":c.record_replay})}
fn parse_config(v:&Value)->Result<RunSessionConfig,RunSessionError>{Ok(RunSessionConfig{
    character:match text(&v["character"])?{"ironclad"=>Character::Ironclad,"silent"=>Character::Silent,"defect"=>Character::Defect,"watcher"=>Character::Watcher,_=>return Err(incompatible())},ascension:u8::try_from(u(&v["ascension"])?).map_err(|_|incompatible())?,
    partition:match text(&v["partition"])?{"train"=>Partition::Train,"selection"=>Partition::Selection,"final"=>Partition::Final,"test"=>Partition::Test,_=>return Err(incompatible())},
    boss_history:parse_boss_history(&v["boss_history"])? ,capacities:parse_capacities(&v["capacities"])? ,record_replay:flag(&v["record_replay"])? ,
})}
fn validate(record:&RunEpisodeRecord)->Result<RunProfile,RunSessionError>{
    let profile=RunProfile::from_record(&record.schema_id,&record.profile_id)?;
    if record.rules_id!=profile.rules_id()||record.content_id!=profile.content_id()||record.seed_adapter_id!=RUN_SEED_ADAPTER_ID||record.rng_id!=spire_core::combat::rng::RNG_CONTRACT_ID||record.reward_id!=RUN_REWARD_ID{return Err(RunSessionError::IncompatibleReplay("Run identity mismatch"))}
    validate_config(profile,&record.config).map_err(|_|incompatible())?;
    if record.observation_fingerprint!=profile.observation_spec(&record.config.capacities)?.fingerprint(){return Err(RunSessionError::IncompatibleReplay("Run observation fingerprint mismatch"))}
    if let Some(f)=&record.fault{
        if f.gameplay_recorded&&(!f.committed||f.action.is_none()||f.action.as_ref()!=record.steps.last().map(|s|&s.action))||f.committed&&f.action.is_none(){return Err(incompatible())}
        if (f.error.kind==RunRecordErrorKind::Capacity)!=f.error.capacity.is_some(){return Err(incompatible())}
        if f.committed&&!f.gameplay_recorded&&f.error.kind!=RunRecordErrorKind::Core{return Err(incompatible())}
    }Ok(profile)
}
impl RunEpisodeRecord{
    pub fn to_json(&self)->Result<Vec<u8>,RunSessionError>{
        let profile=validate(self)?;
        let m15_action=|action:&GameAction|matches!(action,GameAction::Combat(CombatAction::ChooseCardOffer{..}|CombatAction::SkipCardOffer));
        if profile != RunProfile::M15SilentHeart && (self.steps.iter().any(|step|m15_action(&step.action))
            ||self.fault.as_ref().and_then(|fault|fault.action.as_ref()).is_some_and(m15_action)) {
            return Err(RunSessionError::UnsupportedBoundary("generated offer actions require M15 replay profile"));
        }
        if profile==RunProfile::M9IroncladAct3&&(self.steps.iter().any(|s|matches!(s.action,GameAction::Combat(CombatAction::ConfirmSelection)))
            ||self.fault.as_ref().and_then(|f|f.action.as_ref()).is_some_and(|a|matches!(a,GameAction::Combat(CombatAction::ConfirmSelection)))){
            return Err(RunSessionError::UnsupportedBoundary("ConfirmSelection is not representable in replay v1"));
        }
        let steps=self.steps.iter().map(|s|json!({"action":action(&s.action,profile),"reward":s.reward,"terminal":term(s.terminal)})).collect::<Vec<_>>();
        let fault=self.fault.as_ref().map(|f|json!({"action":f.action.as_ref().map(|a|action(a,profile)),"committed":f.committed,"gameplay_recorded":f.gameplay_recorded,"error":f.error}));
        crate::canonical_json(&json!({"schema_id":self.schema_id,"profile_id":self.profile_id,"rules_id":self.rules_id,"content_id":self.content_id,"seed_adapter_id":self.seed_adapter_id,"rng_id":self.rng_id,"reward_id":self.reward_id,"observation_fingerprint":self.observation_fingerprint,"config":config(&self.config),"seed":self.seed,"ordinal":self.ordinal,"steps":steps,"terminal":term(self.terminal),"fault":fault,"closed":self.closed})).map_err(|_|RunSessionError::DeliveryFault("Run record JSON export failed"))
    }
    pub fn from_json(bytes:&[u8])->Result<Self,RunSessionError>{
        let v = parse_unique_json(bytes)?;
        let profile=RunProfile::from_record(text(&v["schema_id"])? ,text(&v["profile_id"])? )?;
        if profile == RunProfile::M15SilentHeart {
            strict_profile_json_shape(&v,"m15-silent-heart",&["act1","act2","act3"])?;
        }
        if profile == RunProfile::M18DefectAct2 { strict_profile_json_shape(&v,"m18-defect-act2",&["act1","act2"])?; }
        if profile == RunProfile::M17DefectAct1 { strict_profile_json_shape(&v,"m17-defect-act1",&["act1"])?; }
        let rows=array(&v["steps"])?;
        let mut steps=Vec::new();
        steps.try_reserve_exact(rows.len()).map_err(|_|RunSessionError::DeliveryFault("Run record import allocation failed"))?;
        for s in rows{
            steps.push(RunRecordedStep{
                action:parse_action(&s["action"],profile)? ,
                reward:i32::try_from(s["reward"].as_i64().ok_or_else(incompatible)?).map_err(|_|incompatible())? ,
                terminal:parse_term(&s["terminal"])?
            });
        }
        let fault=if v["fault"].is_null(){
            None
        }else{
            let f=&v["fault"];
            Some(RunRecordedFault{
                action:if f["action"].is_null(){None}else{Some(parse_action(&f["action"],profile)? )},
                committed:flag(&f["committed"])? ,
                gameplay_recorded:flag(&f["gameplay_recorded"])? ,
                error:serde_json::from_value(f["error"].clone()).map_err(|_|incompatible())?
            })
        };
        let fingerprint=array(&v["observation_fingerprint"])? .iter()
            .map(|v|u8::try_from(u(v)?).map_err(|_|incompatible()))
            .collect::<Result<Vec<_>,_>>()?;
        let record=Self{
            schema_id:text(&v["schema_id"])? .into(),
            profile_id:text(&v["profile_id"])? .into(),
            rules_id:text(&v["rules_id"])? .into(),
            content_id:text(&v["content_id"])? .into(),
            seed_adapter_id:text(&v["seed_adapter_id"])? .into(),
            rng_id:text(&v["rng_id"])? .into(),
            reward_id:text(&v["reward_id"])? .into(),
            observation_fingerprint:fingerprint.try_into().map_err(|_|incompatible())?,
            config:parse_config(&v["config"])? ,
            seed:u(&v["seed"])? ,
            ordinal:u(&v["ordinal"])? ,
            steps,
            terminal:parse_term(&v["terminal"])? ,
            fault,
            closed:flag(&v["closed"])?
        };
        validate(&record)?;
        Ok(record)
    }
}
pub fn replay_run(record:&RunEpisodeRecord,caps:RunCapacities)->Result<RunReplayReport,RunSessionError>{
    let profile=validate(record)?;
    match profile{RunProfile::M9IroncladAct3=>crate::run_schema::validate(&caps)?,RunProfile::M10IroncladHeart|RunProfile::M10IroncladPolicy=>crate::run_schema_m10::validate(&caps)?,RunProfile::M12SilentAct1=>crate::run_schema_m12::validate(&caps)?,RunProfile::M13SilentAct2=>crate::run_schema_m13::validate(&caps)?,RunProfile::M14SilentAct3=>crate::run_schema_m14::validate(&caps)?,RunProfile::M15SilentHeart=>crate::run_schema_m15::validate(&caps)?,RunProfile::M17DefectAct1=>crate::run_schema_m17::validate(&caps)?,RunProfile::M18DefectAct2=>crate::run_schema_m18::validate(&caps)?}
    if profile==RunProfile::M9IroncladAct3&&(record.steps.iter().any(|s|matches!(s.action,GameAction::Combat(CombatAction::ConfirmSelection)))
        ||record.fault.as_ref().and_then(|f|f.action.as_ref()).is_some_and(|a|matches!(a,GameAction::Combat(CombatAction::ConfirmSelection)))){
        return Err(RunSessionError::IncompatibleReplay("ConfirmSelection is not representable in replay v1"));
    }
    let mut run=create_run(profile,&record.config,record.seed,record.ordinal)?;
    let mut decision=run.decision().map_err(RunSessionError::Core)?;
    match profile{RunProfile::M9IroncladAct3=>crate::encode_run(&decision,run.public_map().as_ref(),&caps)?,RunProfile::M10IroncladHeart=>crate::encode_run_m10(&decision,run.public_map().as_ref(),&caps)?,RunProfile::M10IroncladPolicy=>crate::encode_run_policy(&decision,run.public_map().as_ref(),&caps)?,RunProfile::M12SilentAct1=>crate::encode_run_m12(&decision,run.public_map().as_ref(),&caps)?,RunProfile::M13SilentAct2=>crate::encode_run_m13(&decision,run.public_map().as_ref(),&caps)?,RunProfile::M14SilentAct3=>crate::encode_run_m14(&decision,run.public_map().as_ref(),&caps)?,RunProfile::M15SilentHeart=>crate::encode_run_m15(&decision,run.public_map().as_ref(),&caps)?,RunProfile::M17DefectAct1=>crate::encode_run_m17(&decision,run.public_map().as_ref(),&caps)?,RunProfile::M18DefectAct2=>crate::encode_run_m18(&decision,run.public_map().as_ref(),&caps)?};
    let mut rewards=Vec::new();let mut terminals=Vec::new();
    rewards.try_reserve_exact(record.steps.len()).map_err(|_|RunSessionError::DeliveryFault("Run replay allocation failed"))?;
    terminals.try_reserve_exact(record.steps.len()).map_err(|_|RunSessionError::DeliveryFault("Run replay allocation failed"))?;
    for step in &record.steps{
        if !decision.candidates.iter().any(|c|c.action==step.action){return Err(RunSessionError::IncompatibleReplay("Run action not currently offered"))}
        let t=run.step(&step.action).map_err(RunSessionError::Core)?;
        if (t.reward,t.terminal)!=(step.reward,step.terminal){return Err(RunSessionError::IncompatibleReplay("Run reward or terminal mismatch"))}
        decision=run.decision().map_err(RunSessionError::Core)?;
        match profile{RunProfile::M9IroncladAct3=>crate::encode_run(&decision,run.public_map().as_ref(),&caps)?,RunProfile::M10IroncladHeart=>crate::encode_run_m10(&decision,run.public_map().as_ref(),&caps)?,RunProfile::M10IroncladPolicy=>crate::encode_run_policy(&decision,run.public_map().as_ref(),&caps)?,RunProfile::M12SilentAct1=>crate::encode_run_m12(&decision,run.public_map().as_ref(),&caps)?,RunProfile::M13SilentAct2=>crate::encode_run_m13(&decision,run.public_map().as_ref(),&caps)?,RunProfile::M14SilentAct3=>crate::encode_run_m14(&decision,run.public_map().as_ref(),&caps)?,RunProfile::M15SilentHeart=>crate::encode_run_m15(&decision,run.public_map().as_ref(),&caps)?,RunProfile::M17DefectAct1=>crate::encode_run_m17(&decision,run.public_map().as_ref(),&caps)?,RunProfile::M18DefectAct2=>crate::encode_run_m18(&decision,run.public_map().as_ref(),&caps)?};
        rewards.push(t.reward);terminals.push(t.terminal);
    }
    if decision.terminal!=record.terminal{return Err(RunSessionError::IncompatibleReplay("Run final terminal mismatch"))}
    let mut final_decision_error=None;
    if let Some(f)=&record.fault{if f.committed&&!f.gameplay_recorded{
        let action=f.action.as_ref().ok_or_else(incompatible)?;
        if !decision.candidates.iter().any(|c|&c.action==action){return Err(incompatible())}
        if !matches!(run.step(action),Err(RunError::Faulted(_))){return Err(RunSessionError::IncompatibleReplay("recorded core fault no longer occurs"))}
        final_decision_error=run.decision().err();
    }}
    Ok(RunReplayReport{final_decision:decision,final_decision_error,final_map:run.public_map(),accepted_count:record.steps.len(),rewards,terminals,reported_fault:record.fault.clone()})
}

#[cfg(test)]
mod independent_review_capacity_codec {
    use super::*;
    #[test]
    fn independent_review_capacity_group_roundtrip() {
        let config=RunSessionConfig{character:Character::Silent,ascension:0,partition:Partition::Test,boss_history:RunBossHistory::M15SilentHeart{act1:crate::RunAct1BossTarget::Guardian,act2:crate::RunAct2BossTarget::Champ,act3:crate::RunAct3BossTarget::AwakenedOne},capacities:RunCapacities::m10_default(),record_replay:true};
        let mut session=crate::RunSession::new_m15(config).unwrap();
        session.reset(Some(7)).unwrap();
        let mut record=session.record().unwrap().unwrap();
        record.fault=Some(RunRecordedFault{action:None,committed:false,gameplay_recorded:false,error:crate::RunRecordError::from_error(&RunSessionError::Capacity{group:"combat_card_offers",required:3,capacity:2})});
        let wire=record.to_json().unwrap();
        let decoded=RunEpisodeRecord::from_json(&wire);
        println!("exported capacity group=combat_card_offers; import={decoded:?}");
        assert_eq!(decoded,Ok(record),"typed valid capacity diagnostic must roundtrip through M15 JSON");
    }
}


#[cfg(test)]
mod m15_review_capacity_groups {
    use super::*;
    #[test]
    fn m15_review_derived_capacity_codec_roundtrips_and_rejects_malformed() {
        let config=RunSessionConfig{character:Character::Silent,ascension:0,partition:Partition::Test,boss_history:RunBossHistory::M15SilentHeart{act1:crate::RunAct1BossTarget::Guardian,act2:crate::RunAct2BossTarget::Champ,act3:crate::RunAct3BossTarget::AwakenedOne},capacities:RunCapacities::m10_default(),record_replay:true};
        let mut session=crate::RunSession::new_m15(config).unwrap();session.reset(Some(7)).unwrap();
        let base=session.record().unwrap().unwrap();
        for group in ["combat_card_offers","known_draw"] {
            let mut record=base.clone();
            record.fault=Some(RunRecordedFault{action:None,committed:false,gameplay_recorded:false,error:crate::RunRecordError::from_error(&RunSessionError::Capacity{group,required:3,capacity:2})});
            let wire=record.to_json().unwrap();assert_eq!(RunEpisodeRecord::from_json(&wire),Ok(record));
            let value:serde_json::Value=serde_json::from_slice(&wire).unwrap();
            for (key,bad) in [("group",json!("unknown_capacity_group")),("required",json!(2)),("required",json!(-1)),("capacity",json!(2.5))] {
                let mut bad_value=value.clone();bad_value["fault"]["error"]["capacity"][key]=bad;
                assert!(RunEpisodeRecord::from_json(&serde_json::to_vec(&bad_value).unwrap()).is_err(),"{group} malformed {key}");
            }
        }
    }
}
