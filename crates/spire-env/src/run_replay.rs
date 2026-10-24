//! Versioned semantic Run replay; no snapshots or simulator implementation here.
use spire_core::{combat::{CombatAction,TargetId,rng::Partition},content::Character,run::{*,map::PublicMap}};
use crate::{RunEpisodeRecord,RunSessionError,RunCapacities,RunRecordedFault,RunRecordedStep,RunSessionConfig,M9BossHistory,RunRecordErrorKind};
use crate::run_session::{create_run,validate_config,RUN_RECORD_SCHEMA_ID,RUN_SEED_ADAPTER_ID,RUN_RULES_ID,RUN_CONTENT_ID,RUN_REWARD_ID};
use serde_json::{Value,json};
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct RunReplayReport{
    /// Latest readable boundary; final_decision_error discloses a faulted current owner.
    pub final_decision:PublicDecision,pub final_decision_error:Option<RunError>,pub final_map:Option<PublicMap>,
    /// Completed successful semantic steps; a partial fault attempt is separate.
    pub accepted_count:usize,pub rewards:Vec<i32>,pub terminals:Vec<Option<RunTerminal>>,pub reported_fault:Option<RunRecordedFault>,
}
fn incompatible()->RunSessionError{RunSessionError::IncompatibleReplay("invalid Run replay value")}
fn u(v:&Value)->Result<u64,RunSessionError>{v.as_u64().ok_or_else(incompatible)}
fn size(v:&Value)->Result<usize,RunSessionError>{usize::try_from(u(v)?).map_err(|_|incompatible())}
fn text(v:&Value)->Result<&str,RunSessionError>{v.as_str().ok_or_else(incompatible)}
fn flag(v:&Value)->Result<bool,RunSessionError>{v.as_bool().ok_or_else(incompatible)}
fn array(v:&Value)->Result<&Vec<Value>,RunSessionError>{v.as_array().ok_or_else(incompatible)}
fn term(t:Option<RunTerminal>)->Value{match t{None=>Value::Null,Some(t)=>json!(match t{RunTerminal::Death=>"death",RunTerminal::EmptyDeck=>"empty_deck",RunTerminal::Act3NoKeys=>"act3_no_keys",RunTerminal::HeartVictory=>"heart_victory"})}}
fn parse_term(v:&Value)->Result<Option<RunTerminal>,RunSessionError>{if v.is_null(){return Ok(None)}Ok(Some(match text(v)?{"death"=>RunTerminal::Death,"empty_deck"=>RunTerminal::EmptyDeck,"act3_no_keys"=>RunTerminal::Act3NoKeys,"heart_victory"=>RunTerminal::HeartVictory,_=>return Err(incompatible())}))}
fn action(a:&GameAction)->Value{match a{
    GameAction::Move(id)=>json!({"verb":"move","node":id.0}),GameAction::Choose(id)=>json!({"verb":"choose","choice":id.0}),GameAction::Select(id)=>json!({"verb":"select","selection":id.0}),GameAction::Confirm=>json!({"verb":"confirm"}),GameAction::Cancel=>json!({"verb":"cancel"}),
    GameAction::UsePotion{slot,target}=>json!({"verb":"use_potion","slot":slot,"target":target.map(|t|t.0)}),GameAction::DiscardPotion{slot}=>json!({"verb":"discard_potion","slot":slot}),
    GameAction::Combat(a)=>match a{CombatAction::PlayCard{instance,target}=>json!({"verb":"combat.play_card","instance":instance,"target":target.map(|t|t.0)}),CombatAction::SelectCard{instance}=>json!({"verb":"combat.select_card","instance":instance}),CombatAction::UsePotion{slot,target}=>json!({"verb":"combat.use_potion","slot":slot,"target":target.map(|t|t.0)}),CombatAction::EndTurn=>json!({"verb":"combat.end_turn"})},
}}
fn parse_action(v:&Value)->Result<GameAction,RunSessionError>{
    let target=||if v["target"].is_null(){Ok(None)}else{Ok(Some(TargetId(u(&v["target"])?)))};
    Ok(match text(&v["verb"])?{
        "move"=>GameAction::Move(NodeId(u(&v["node"])?)),"choose"=>GameAction::Choose(ChoiceId(u(&v["choice"])?)),"select"=>GameAction::Select(SelectionId(u(&v["selection"])?)),"confirm"=>GameAction::Confirm,"cancel"=>GameAction::Cancel,
        "use_potion"=>GameAction::UsePotion{slot:u32::try_from(u(&v["slot"])?).map_err(|_|incompatible())?,target:target()?},"discard_potion"=>GameAction::DiscardPotion{slot:u32::try_from(u(&v["slot"])?).map_err(|_|incompatible())?},
        "combat.play_card"=>GameAction::Combat(CombatAction::PlayCard{instance:u(&v["instance"])? ,target:target()?}),"combat.select_card"=>GameAction::Combat(CombatAction::SelectCard{instance:u(&v["instance"])?}),"combat.use_potion"=>GameAction::Combat(CombatAction::UsePotion{slot:size(&v["slot"])? ,target:target()?}),"combat.end_turn"=>GameAction::Combat(CombatAction::EndTurn),_=>return Err(incompatible()),
    })
}
fn capacities(c:&RunCapacities)->Value{json!({"map_nodes":c.map_nodes,"map_edges":c.map_edges,"permanent_cards":c.permanent_cards,"combat_cards":c.combat_cards,"enemies":c.enemies,"powers":c.powers,"relics":c.relics,"potions":c.potions,"offers":c.offers,"selection":c.selection,"orbs":c.orbs,"actions":c.actions})}
fn parse_capacities(v:&Value)->Result<RunCapacities,RunSessionError>{Ok(RunCapacities{map_nodes:size(&v["map_nodes"])? ,map_edges:size(&v["map_edges"])? ,permanent_cards:size(&v["permanent_cards"])? ,combat_cards:size(&v["combat_cards"])? ,enemies:size(&v["enemies"])? ,powers:size(&v["powers"])? ,relics:size(&v["relics"])? ,potions:size(&v["potions"])? ,offers:size(&v["offers"])? ,selection:size(&v["selection"])? ,orbs:size(&v["orbs"])? ,actions:size(&v["actions"])?})}
fn config(c:&RunSessionConfig)->Value{json!({"character":c.character.as_str(),"ascension":c.ascension,"partition":c.partition.as_str(),"boss_history":match c.boss_history{M9BossHistory::AwakenedUnseen=>"awakened-unseen",M9BossHistory::TimeEaterUnseen=>"time-eater-unseen",M9BossHistory::DonuDecaUnseen=>"donu-deca-unseen"},"capacities":capacities(&c.capacities),"record_replay":c.record_replay})}
fn parse_config(v:&Value)->Result<RunSessionConfig,RunSessionError>{Ok(RunSessionConfig{
    character:match text(&v["character"])?{"ironclad"=>Character::Ironclad,"silent"=>Character::Silent,"defect"=>Character::Defect,"watcher"=>Character::Watcher,_=>return Err(incompatible())},ascension:u8::try_from(u(&v["ascension"])?).map_err(|_|incompatible())?,
    partition:match text(&v["partition"])?{"train"=>Partition::Train,"selection"=>Partition::Selection,"final"=>Partition::Final,"test"=>Partition::Test,_=>return Err(incompatible())},
    boss_history:match text(&v["boss_history"])?{"awakened-unseen"=>M9BossHistory::AwakenedUnseen,"time-eater-unseen"=>M9BossHistory::TimeEaterUnseen,"donu-deca-unseen"=>M9BossHistory::DonuDecaUnseen,_=>return Err(incompatible())},capacities:parse_capacities(&v["capacities"])? ,record_replay:flag(&v["record_replay"])? ,
})}
fn validate(record:&RunEpisodeRecord)->Result<(),RunSessionError>{
    if record.schema_id!=RUN_RECORD_SCHEMA_ID||record.profile_id!="M9IroncladAct3"||record.rules_id!=RUN_RULES_ID||record.content_id!=RUN_CONTENT_ID||record.seed_adapter_id!=RUN_SEED_ADAPTER_ID||record.rng_id!=spire_core::combat::rng::RNG_CONTRACT_ID||record.reward_id!=RUN_REWARD_ID{return Err(RunSessionError::IncompatibleReplay("Run identity mismatch"))}
    validate_config(&record.config).map_err(|_|incompatible())?;
    if record.observation_fingerprint!=crate::run_observation_spec(&record.config.capacities)?.fingerprint(){return Err(RunSessionError::IncompatibleReplay("Run observation fingerprint mismatch"))}
    if let Some(f)=&record.fault{
        if f.gameplay_recorded&&(!f.committed||f.action.is_none()||f.action.as_ref()!=record.steps.last().map(|s|&s.action))||f.committed&&f.action.is_none(){return Err(incompatible())}
        if (f.error.kind==RunRecordErrorKind::Capacity)!=f.error.capacity.is_some(){return Err(incompatible())}
        if f.committed&&!f.gameplay_recorded&&f.error.kind!=RunRecordErrorKind::Core{return Err(incompatible())}
    }Ok(())
}
impl RunEpisodeRecord{
    pub fn to_json(&self)->Result<Vec<u8>,RunSessionError>{
        validate(self)?;
        let steps=self.steps.iter().map(|s|json!({"action":action(&s.action),"reward":s.reward,"terminal":term(s.terminal)})).collect::<Vec<_>>();
        let fault=self.fault.as_ref().map(|f|json!({"action":f.action.as_ref().map(action),"committed":f.committed,"gameplay_recorded":f.gameplay_recorded,"error":f.error}));
        crate::canonical_json(&json!({"schema_id":self.schema_id,"profile_id":self.profile_id,"rules_id":self.rules_id,"content_id":self.content_id,"seed_adapter_id":self.seed_adapter_id,"rng_id":self.rng_id,"reward_id":self.reward_id,"observation_fingerprint":self.observation_fingerprint,"config":config(&self.config),"seed":self.seed,"ordinal":self.ordinal,"steps":steps,"terminal":term(self.terminal),"fault":fault,"closed":self.closed})).map_err(|_|RunSessionError::DeliveryFault("Run record JSON export failed"))
    }
    pub fn from_json(bytes:&[u8])->Result<Self,RunSessionError>{
        let v:Value=serde_json::from_slice(bytes).map_err(|_|incompatible())?;let rows=array(&v["steps"])?;
        let mut steps=Vec::new();steps.try_reserve_exact(rows.len()).map_err(|_|RunSessionError::DeliveryFault("Run record import allocation failed"))?;
        for s in rows{steps.push(RunRecordedStep{action:parse_action(&s["action"])? ,reward:i32::try_from(s["reward"].as_i64().ok_or_else(incompatible)?).map_err(|_|incompatible())?,terminal:parse_term(&s["terminal"])?});}
        let fault=if v["fault"].is_null(){None}else{let f=&v["fault"];Some(RunRecordedFault{action:if f["action"].is_null(){None}else{Some(parse_action(&f["action"])? )},committed:flag(&f["committed"])? ,gameplay_recorded:flag(&f["gameplay_recorded"])? ,error:serde_json::from_value(f["error"].clone()).map_err(|_|incompatible())?})};
        let fingerprint=array(&v["observation_fingerprint"])? .iter().map(|v|u8::try_from(u(v)?).map_err(|_|incompatible())).collect::<Result<Vec<_>,_>>()?;
        let record=Self{schema_id:text(&v["schema_id"])? .into(),profile_id:text(&v["profile_id"])? .into(),rules_id:text(&v["rules_id"])? .into(),content_id:text(&v["content_id"])? .into(),seed_adapter_id:text(&v["seed_adapter_id"])? .into(),rng_id:text(&v["rng_id"])? .into(),reward_id:text(&v["reward_id"])? .into(),observation_fingerprint:fingerprint.try_into().map_err(|_|incompatible())?,config:parse_config(&v["config"])? ,seed:u(&v["seed"])? ,ordinal:u(&v["ordinal"])? ,steps,terminal:parse_term(&v["terminal"])? ,fault,closed:flag(&v["closed"])?};
        validate(&record)?;Ok(record)
    }
}
pub fn replay_run(record:&RunEpisodeRecord,caps:RunCapacities)->Result<RunReplayReport,RunSessionError>{
    validate(record)?;crate::run_schema::validate(&caps)?;
    let mut run=create_run(&record.config,record.seed,record.ordinal)?;
    let mut decision=run.decision().map_err(RunSessionError::Core)?;
    crate::encode_run(&decision,run.public_map().as_ref(),&caps)?;
    let mut rewards=Vec::new();let mut terminals=Vec::new();
    rewards.try_reserve_exact(record.steps.len()).map_err(|_|RunSessionError::DeliveryFault("Run replay allocation failed"))?;
    terminals.try_reserve_exact(record.steps.len()).map_err(|_|RunSessionError::DeliveryFault("Run replay allocation failed"))?;
    for step in &record.steps{
        if !decision.candidates.iter().any(|c|c.action==step.action){return Err(RunSessionError::IncompatibleReplay("Run action not currently offered"))}
        let t=run.step(&step.action).map_err(RunSessionError::Core)?;
        if (t.reward,t.terminal)!=(step.reward,step.terminal){return Err(RunSessionError::IncompatibleReplay("Run reward or terminal mismatch"))}
        decision=run.decision().map_err(RunSessionError::Core)?;crate::encode_run(&decision,run.public_map().as_ref(),&caps)?;rewards.push(t.reward);terminals.push(t.terminal);
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
