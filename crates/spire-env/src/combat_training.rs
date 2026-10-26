//! Standalone full-engine combat owner. Validation is separate from accepted
//! effects; faults retain the same Run and cannot reset into a fabricated loss.
use crate::{RunSessionError as Error,RunObservationSpec,RunDecisionToken,RunActionIndex,CombatPolicyBoundary};
use crate::combat_policy::{encode_combat_policy,lifecycle,hex};
use spire_core::{run::*,content::*,combat::{rng::Partition,Outcome}};
use serde::Deserialize;
use serde_json::{Value,json};
use sha2::{Digest,Sha256};
pub const COMBAT_TRAINING_PROFILE_ID:&str="ironclad-combat-v1";
pub const COMBAT_TRAINING_CONTENT_ID:&str="spire-gym-ironclad-combat-content-v1";
pub const COMBAT_TRAINING_REWARD_ID:&str="spire-gym-combat-outcome-reward-v1";
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawCatalog{entries:Vec<RawEntry>}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawCard{card:String,upgrades:u32}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawEntry{id:String,encounter:String,hp:u32,max_hp:u32,deck:Vec<RawCard>,relics:Vec<String>,potions:Vec<Option<String>>,#[serde(default)]relic_counters:std::collections::BTreeMap<String,i32>}
#[derive(Clone,Debug)]
pub struct CombatTrainingCatalog{entries:Vec<(String,CombatTrainingEntry)>,descriptor:Value,fingerprint:[u8;32]}
fn lookup<T:Copy>(s:&str,ids:&[T],name:impl Fn(T)->&'static str)->Result<T,Error>{ids.iter().copied().find(|id|name(*id)==s).ok_or(Error::Input("unsupported combat training content"))}
impl CombatTrainingCatalog{
    pub fn from_json(bytes:&[u8])->Result<Self,Error>{
        if bytes.len()>1_048_576{return Err(Error::Input("combat catalog exceeds 1 MiB"))}
        let raw:RawCatalog=serde_json::from_slice(bytes).map_err(|_|Error::Input("invalid combat catalog fields or types"))?;
        if !(1..=256).contains(&raw.entries.len()){return Err(Error::Input("catalog requires 1..256 entries"))}
        let mut entries=Vec::new();let mut normalized=Vec::new();let mut ids=std::collections::BTreeSet::new();
        for r in raw.entries{
            if r.id.is_empty()||r.id.len()>64||!r.id.bytes().all(|b|b.is_ascii_alphanumeric()||b==b'_'||b==b'-')||!ids.insert(r.id.clone()){
                return Err(Error::Input("invalid or duplicate combat entry ID"))
            }
            let encounter=TRAINING_ENCOUNTERS.iter().find(|(e,_,_)|e.as_str()==r.encounter).map(|x|x.0).ok_or(Error::Input("unsupported combat encounter"))?;
            let mut deck=Vec::new();for c in r.deck{deck.push((lookup(&c.card,TRAINING_CARDS,CardId::as_str)?,c.upgrades));}
            let mut relics=Vec::new();for relic in r.relics{relics.push(lookup(&relic,TRAINING_RELICS,RelicId::as_str)?);}
            let mut potions=Vec::new();for potion in r.potions{potions.push(potion.map(|p|lookup(&p,TRAINING_POTIONS,PotionId::as_str)).transpose()?);}
            if r.relic_counters.keys().any(|k|k!="neows_lament"){return Err(Error::Input("unsupported initial relic counter"))}
            let mut entry=CombatTrainingEntry{encounter,hp:r.hp,max_hp:r.max_hp,deck,relics,potions,neows_lament_remaining:r.relic_counters.get("neows_lament").copied()};
            entry.validate().map_err(|_|Error::Input("invalid combat entry resources"))?;
            entry.deck.sort_by_key(|(id,u)|(id.as_str(),*u));entry.relics.sort_by_key(|id|id.as_str());
            let counters=if entry.relics.contains(&RelicId::NeowsLament){json!({"neows_lament":entry.neows_lament_remaining.unwrap_or(3)})}else{json!({})};
            normalized.push(json!({"id":r.id,"encounter":encounter.as_str(),"hp":entry.hp,"max_hp":entry.max_hp,
                "deck":entry.deck.iter().map(|(c,u)|json!({"card":c.as_str(),"upgrades":u})).collect::<Vec<_>>(),
                "relics":entry.relics.iter().map(|r|r.as_str()).collect::<Vec<_>>(),"potions":entry.potions.iter().map(|p|p.map(PotionId::as_str)).collect::<Vec<_>>(),"relic_counters":counters}));
            entries.push((r.id,entry));
        }
        let descriptor=json!({"catalog_schema_id":"spire-gym-combat-entry-catalog-v1","entries":normalized});
        let mut h=Sha256::new();h.update(b"spire-gym/combat-entry-catalog/v1\0");h.update(crate::canonical_json(&descriptor).map_err(|_|Error::Input("invalid catalog"))?);
        Ok(Self{entries,descriptor,fingerprint:h.finalize().into()})
    }
    pub fn descriptor(&self)->Value{self.descriptor.clone()}
    pub fn fingerprint(&self)->[u8;32]{self.fingerprint}
    fn entry(&self,id:Option<&str>)->Result<&(String,CombatTrainingEntry),Error>{match id{None=>Ok(&self.entries[0]),Some(id)=>self.entries.iter().find(|(key,_)|key==id).ok_or(Error::Input("unknown combat entry ID"))}}
}
/// Actual finite input support, separate from broad registry metadata.
pub fn combat_training_support()->Value{json!({
    "character":"ironclad","ascension":0,"hp":{"minimum":1,"maximum":999},"deck":{"minimum":1,"maximum":128,"ordering":"unordered canonical multiset"},
    "cards":TRAINING_CARDS.iter().map(|c|json!({"id":c.as_str(),"maximum_upgrades":if *c==CardId::SearingBlow{10}else{1}})).collect::<Vec<_>>(),
    "encounters":TRAINING_ENCOUNTERS.iter().map(|(e,a,k)|json!({"id":e.as_str(),"act":a,"kind":k.label()})).collect::<Vec<_>>(),
    "relics":TRAINING_RELICS.iter().map(|r|r.as_str()).collect::<Vec<_>>(),"potions":TRAINING_POTIONS.iter().map(|p|p.as_str()).collect::<Vec<_>>(),"potion_slots":3,
    "relic_counters":{"neows_lament":{"default":3,"allowed":[-2,1,2,3],"requires_equipped":true}},
    "resource_semantics":"already owned; acquisition HP/deck effects included by caller, not repeated", "maximum_catalog_entries":256,"maximum_catalog_bytes":1048576,
    "unsupported":"other characters/ascensions, hidden state/order/RNG, burning elites, unlisted cards/items/encounters; no full standard pool or original-runtime parity"})}
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct CombatTrainingStep{pub boundary:CombatPolicyBoundary,pub reward:i32,pub terminated:bool,pub truncated:bool}
pub struct CombatTrainingSession{
    catalog:CombatTrainingCatalog,partition:Partition,id:u64,serial:u64,ordinal:u64,seed:Option<u64>,entry_id:Option<String>,
    run:Option<Run>,published:Option<CombatPolicyBoundary>,actions:Vec<GameAction>,fault:Option<Error>,closed:bool,accepted_steps:u64,last_step_committed:bool,
}
impl CombatTrainingSession{
    pub fn new(catalog:CombatTrainingCatalog,partition:Partition)->Result<Self,Error>{
        let id=crate::run_session::allocate_run_session_id()?;
        Ok(Self{catalog,partition,id,serial:0,ordinal:0,seed:None,entry_id:None,run:None,published:None,actions:vec![],fault:None,closed:false,accepted_steps:0,last_step_committed:false})
    }
    pub fn observation_spec(&self)->Result<RunObservationSpec,Error>{crate::combat_policy_observation_spec()}
    pub fn catalog_descriptor(&self)->Value{self.catalog.descriptor()}
    pub fn environment_spec(&self)->Result<Value,Error>{
        let spec=self.observation_spec()?;
        Ok(json!({"profile_id":COMBAT_TRAINING_PROFILE_ID,"schema_id":crate::COMBAT_POLICY_VIEW_ID,"observation_fingerprint":hex(&spec.fingerprint()),
            "content_id":COMBAT_TRAINING_CONTENT_ID,"rules_id":crate::RUN_RULES_ID_M10,"reward_id":COMBAT_TRAINING_REWARD_ID,
            "rng_id":spire_core::combat::rng::RNG_CONTRACT_ID,"seed_adapter_id":crate::RUN_SEED_ADAPTER_ID,"combat_view_id":crate::COMBAT_POLICY_VIEW_ID,
            "mode":"standalone_combat","partition":self.partition.as_str(),"entry_catalog_fingerprint":hex(&self.catalog.fingerprint()),"autoreset":"disabled","seed_domain":"u64; explicit seed resets episode ordinal to zero; implicit reset advances ordinal",
            "support":combat_training_support()}))
    }
    fn usable(&self)->Result<(),Error>{if self.closed{return Err(Error::Closed)}if let Some(e)=&self.fault{return Err(e.clone())}Ok(())}
    fn freeze(&mut self,e:Error)->Error{if self.fault.is_none(){self.fault=Some(e.clone());}self.fault.clone().unwrap()}
    pub fn effective_seed(&self)->Option<u64>{self.seed}
    pub fn preflight_reset(&self,seed:Option<u64>,entry_id:Option<&str>)->Result<(),Error>{
        self.usable()?;self.catalog.entry(entry_id)?;
        self.serial.checked_add(1).ok_or(Error::Input("combat decision serial exhausted"))?;
        if seed.is_none()&&self.seed.is_some(){self.ordinal.checked_add(1).ok_or(Error::Input("combat episode ordinal exhausted"))?;}Ok(())
    }
    pub fn reset(&mut self,seed:Option<u64>,entry_id:Option<&str>)->Result<CombatPolicyBoundary,Error>{
        self.preflight_reset(seed,entry_id)?;
        let (id,entry)=self.catalog.entry(entry_id)?.clone();
        let (effective,ordinal)=if let Some(seed)=seed{(seed,0)}else if let Some(seed)=self.seed{(seed,self.ordinal+1)}else{
            let mut bytes=[0;8];getrandom::fill(&mut bytes).map_err(|_|Error::Input("combat seed entropy unavailable"))?;(u64::from_le_bytes(bytes),0)
        };
        let mut root=[0;32];root[..8].copy_from_slice(&effective.to_le_bytes());
        let run=Run::prepare_combat_training(entry.clone(),RunRng::new(root,self.partition,ordinal)).map_err(Error::Core)?;
        // From this point this session retains the accepted entry even if its
        // startup callbacks or numeric publication fail.
        self.run=Some(run);self.seed=Some(effective);self.ordinal=ordinal;self.entry_id=Some(id);self.accepted_steps=0;self.last_step_committed=false;self.serial+=1;self.published=None;self.actions.clear();
        if let Err(e)=self.run.as_mut().unwrap().enter_training_combat(entry.encounter){return Err(self.freeze(Error::Core(e)))}
        self.publish()
    }
    fn publish(&mut self)->Result<CombatPolicyBoundary,Error>{
        let d=match self.run.as_ref().unwrap().decision(){Ok(d)=>d,Err(e)=>return Err(self.freeze(Error::Core(e)))};
        let cp=self.run.as_ref().unwrap().combat_checkpoint();
        let observation=match encode_combat_policy(&d,cp.as_ref()){Ok(o)=>o,Err(e)=>return Err(self.freeze(e))};
        let mut info=lifecycle(&d,cp.as_ref());info["profile_id"]=json!(COMBAT_TRAINING_PROFILE_ID);info["entry_id"]=json!(self.entry_id);info["episode_ordinal"]=json!(self.ordinal);
        info["effective_seed"]=json!(self.seed);info["outcome"]=cp.as_ref().map(|c|json!(match c.record.outcome{Outcome::Victory=>"victory",Outcome::Death=>"death"})).unwrap_or(Value::Null);
        let boundary=CombatPolicyBoundary{observation,token:[self.id,self.serial].into(),info};
        self.actions=d.candidates.into_iter().map(|c|c.action).collect();
        self.published=Some(boundary);self.decision()
    }
    pub fn decision(&self)->Result<CombatPolicyBoundary,Error>{
        self.usable()?;let b=self.published.as_ref().ok_or(Error::NotReset)?;
        Ok(CombatPolicyBoundary{observation:b.observation.owned()?,token:b.token,info:b.info.clone()})
    }
    pub fn preflight_step(&self,token:RunDecisionToken,index:RunActionIndex)->Result<(),Error>{
        self.usable()?;let b=self.published.as_ref().ok_or(Error::NotReset)?;
        if token!=b.token{return Err(Error::StaleToken)}
        if self.run.as_ref().unwrap().combat_checkpoint().is_some(){return Err(Error::Terminal)}
        if usize::try_from(index.0).ok().is_none_or(|i|i>=self.actions.len()){return Err(Error::Input("combat action index is not legal"))}
        self.serial.checked_add(1).ok_or(Error::Input("combat decision serial exhausted"))?;self.accepted_steps.checked_add(1).ok_or(Error::Input("combat step count exhausted"))?;Ok(())
    }
    pub fn step(&mut self,token:RunDecisionToken,index:RunActionIndex)->Result<CombatTrainingStep,Error>{
        self.preflight_step(token,index)?;
        let action=self.actions[index.0 as usize].clone();
        if let Err(e)=self.run.as_mut().unwrap().step(&action){
            self.last_step_committed=matches!(e,RunError::Faulted(_));
            if self.last_step_committed{self.accepted_steps+=1;self.serial+=1;}
            return Err(self.freeze(Error::Core(e)))
        }
        self.last_step_committed=true;
        self.accepted_steps+=1;self.serial+=1;
        let cp=self.run.as_ref().unwrap().combat_checkpoint();let terminated=cp.is_some();
        let reward=cp.map_or(0,|c|if c.record.outcome==Outcome::Victory{1}else{-1});
        let boundary=self.publish()?;Ok(CombatTrainingStep{boundary,reward,terminated,truncated:false})
    }
    pub fn fail_delivery(&mut self,reason:&'static str){self.freeze(Error::DeliveryFault(reason));}
    pub fn diagnostic(&self)->Value{json!({"fault":self.fault.as_ref().map(|e|match e{
        Error::Core(_)=>format!("Core({})",crate::RunRecordError::from_error(e).message.replace(" (full owner retained in reproduction)","")),
        _=>format!("{e:?}")
    }),"accepted_steps":self.accepted_steps,"last_step_committed":self.last_step_committed,"closed":self.closed,"entry_id":self.entry_id,"effective_seed":self.seed,"episode_ordinal":self.ordinal})}
    pub fn close(&mut self)->Result<(),Error>{self.closed=true;Ok(())}
}
