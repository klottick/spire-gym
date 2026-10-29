//! Standalone full-engine combat owner. Validation is separate from accepted
//! effects; faults retain the same Run and cannot reset into a fabricated loss.
use crate::{RunSessionError as Error,RunObservationSpec,RunDecisionToken,RunActionIndex,CombatPolicyBoundary};
use crate::combat_policy::{encode_combat_policy,lifecycle,hex};
use spire_core::{run::*,content::*,combat::{rng::Partition,Outcome}};
use serde::Deserialize;
use serde_json::{Value,json};
use sha2::{Digest,Sha256};
pub const COMBAT_TRAINING_PROFILE_ID:&str="ironclad-combat-v1";
pub const COMBAT_TRAINING_PROFILE_V2_ID:&str="ironclad-combat-v2";
pub const COMBAT_TRAINING_CONTENT_ID:&str="spire-gym-ironclad-combat-content-v1";
pub const COMBAT_TRAINING_CONTENT_V2_ID:&str="spire-gym-ironclad-combat-content-v2";
pub const COMBAT_TRAINING_REWARD_ID:&str="spire-gym-combat-outcome-reward-v1";
fn default_combat_training_profile_id()->String{COMBAT_TRAINING_PROFILE_ID.to_owned()}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
enum CombatTrainingProfile{V1,V2,M21}
impl CombatTrainingProfile{
    fn id(self)->&'static str{match self{Self::V1=>COMBAT_TRAINING_PROFILE_ID,Self::M21=>M21_WATCHER_COMBAT_PROFILE_ID,Self::V2=>COMBAT_TRAINING_PROFILE_V2_ID}}
    fn content_id(self)->&'static str{match self{Self::V1=>COMBAT_TRAINING_CONTENT_ID,Self::M21=>M21_WATCHER_COMBAT_CONTENT_ID,Self::V2=>COMBAT_TRAINING_CONTENT_V2_ID}}
    fn view_id(self)->&'static str{match self{Self::V1=>crate::COMBAT_POLICY_VIEW_ID,Self::M21=>crate::M21_WATCHER_COMBAT_OBSERVATION_SCHEMA_ID,Self::V2=>crate::COMBAT_POLICY_VIEW_V2_ID}}
    fn encounters(self)->&'static [(EncounterId,u8,spire_core::combat::enemies::EncounterKind)]{
        match self{Self::V1=>TRAINING_ENCOUNTERS,Self::V2|Self::M21=>TRAINING_ENCOUNTERS_V2}
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawCatalog{#[serde(default="default_combat_training_profile_id")]profile_id:String,entries:Vec<RawEntry>}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawCard{card:String,upgrades:u32}
fn owned_gold<'de,D:serde::Deserializer<'de>>(deserializer:D)->Result<Option<u32>,D::Error>{u32::deserialize(deserializer).map(Some)}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawEntry{#[serde(default,deserialize_with="owned_gold")]gold:Option<u32>,id:String,encounter:String,hp:u32,max_hp:u32,deck:Vec<RawCard>,relics:Vec<String>,potions:Vec<Option<String>>,#[serde(default)]relic_counters:std::collections::BTreeMap<String,i32>}
#[derive(Clone,Debug)]
pub struct CombatTrainingCatalog{profile:CombatTrainingProfile,entries:Vec<(String,TrainingEntry)>,descriptor:Value,fingerprint:[u8;32]}
fn lookup<T:Copy>(s:&str,ids:&[T],name:impl Fn(T)->&'static str)->Result<T,Error>{ids.iter().copied().find(|id|name(*id)==s).ok_or(Error::Input("unsupported combat training content"))}
impl CombatTrainingCatalog{
    pub fn from_json(bytes:&[u8])->Result<Self,Error>{
        if bytes.len()>1_048_576{return Err(Error::Input("combat catalog exceeds 1 MiB"))}
        let raw:RawCatalog=serde_json::from_slice(bytes).map_err(|_|Error::Input("invalid combat catalog fields or types"))?;
        let profile=match raw.profile_id.as_str(){
            COMBAT_TRAINING_PROFILE_ID=>CombatTrainingProfile::V1,
            COMBAT_TRAINING_PROFILE_V2_ID=>CombatTrainingProfile::V2,
            _=>return Err(Error::Input("unsupported combat training profile_id")),
        };
        if !(1..=256).contains(&raw.entries.len()){return Err(Error::Input("catalog requires 1..256 entries"))}
        let mut entries=Vec::new();let mut normalized=Vec::new();let mut ids=std::collections::BTreeSet::new();
        for r in raw.entries{
            if profile==CombatTrainingProfile::V1&&r.gold.is_some(){return Err(Error::Input("owned gold requires combat training v2"))}
            let gold=r.gold.unwrap_or(0);
            if r.id.is_empty()||r.id.len()>64||!r.id.bytes().all(|b|b.is_ascii_alphanumeric()||b==b'_'||b==b'-')||!ids.insert(r.id.clone()){
                return Err(Error::Input("invalid or duplicate combat entry ID"))
            }
            let encounter=profile.encounters().iter().find(|(e,_,_)|e.as_str()==r.encounter).map(|x|x.0).ok_or(Error::Input("unsupported combat encounter"))?;
            let mut deck=Vec::new();for c in r.deck{deck.push((lookup(&c.card,TRAINING_CARDS,CardId::as_str)?,c.upgrades));}
            let mut relics=Vec::new();for relic in r.relics{relics.push(lookup(&relic,TRAINING_RELICS,RelicId::as_str)?);}
            let mut potions=Vec::new();for potion in r.potions{potions.push(potion.map(|p|lookup(&p,TRAINING_POTIONS,PotionId::as_str)).transpose()?);}
            if r.relic_counters.keys().any(|k|k!="neows_lament"){return Err(Error::Input("unsupported initial relic counter"))}
            let mut entry=CombatTrainingEntry{encounter,hp:r.hp,max_hp:r.max_hp,deck,relics,potions,neows_lament_remaining:r.relic_counters.get("neows_lament").copied()};
            match profile{
                CombatTrainingProfile::V1=>entry.validate(),
                CombatTrainingProfile::V2=>entry.validate_v2(),CombatTrainingProfile::M21=>unreachable!("typed-only profile"),
            }.map_err(|_|Error::Input("invalid combat entry resources"))?;
            entry.deck.sort_by_key(|(id,u)|(id.as_str(),*u));entry.relics.sort_by_key(|id|id.as_str());
            let counters=if entry.relics.contains(&RelicId::NeowsLament){json!({"neows_lament":entry.neows_lament_remaining.unwrap_or(3)})}else{json!({})};
            let mut normalized_entry=json!({"id":r.id,"encounter":encounter.as_str(),"hp":entry.hp,"max_hp":entry.max_hp,
                "deck":entry.deck.iter().map(|(c,u)|json!({"card":c.as_str(),"upgrades":u})).collect::<Vec<_>>(),
                "relics":entry.relics.iter().map(|r|r.as_str()).collect::<Vec<_>>(),"potions":entry.potions.iter().map(|p|p.map(PotionId::as_str)).collect::<Vec<_>>(),"relic_counters":counters});
            if profile==CombatTrainingProfile::V2{normalized_entry["gold"]=json!(gold);}
            normalized.push(normalized_entry);entries.push((r.id,TrainingEntry::Legacy{entry,gold}));
        }
        let (descriptor,domain)=match profile{
            CombatTrainingProfile::V1=>(json!({"catalog_schema_id":"spire-gym-combat-entry-catalog-v1","entries":normalized}),b"spire-gym/combat-entry-catalog/v1\0".as_slice()),
            CombatTrainingProfile::M21=>unreachable!("typed-only profile"),
            CombatTrainingProfile::V2=>(json!({"catalog_schema_id":"spire-gym-combat-entry-catalog-v2","profile_id":COMBAT_TRAINING_PROFILE_V2_ID,"entries":normalized}),b"spire-gym/combat-entry-catalog/v2\0".as_slice()),
        };
        let mut h=Sha256::new();h.update(domain);h.update(crate::canonical_json(&descriptor).map_err(|_|Error::Input("invalid catalog"))?);
        Ok(Self{profile,entries,descriptor,fingerprint:h.finalize().into()})
    }
    pub fn descriptor(&self)->Value{self.descriptor.clone()}
    pub fn fingerprint(&self)->[u8;32]{self.fingerprint}
    fn entry(&self,id:Option<&str>)->Result<&(String,TrainingEntry),Error>{match id{None=>Ok(&self.entries[0]),Some(id)=>self.entries.iter().find(|(key,_)|key==id).ok_or(Error::Input("unknown combat entry ID"))}}
}
/// Actual finite input support, separate from broad registry metadata.
pub fn combat_training_support()->Value{combat_training_support_for(CombatTrainingProfile::V1)}
pub fn combat_training_support_v2()->Value{combat_training_support_for(CombatTrainingProfile::V2)}
fn combat_training_support_for(profile:CombatTrainingProfile)->Value{
let encounters:&[(EncounterId,u8,spire_core::combat::enemies::EncounterKind)]=profile.encounters();
let mut support=json!({
    "character":"ironclad","ascension":0,"hp":{"minimum":1,"maximum":999},"deck":{"minimum":1,"maximum":128,"ordering":"unordered canonical multiset"},
    "cards":TRAINING_CARDS.iter().map(|c|json!({"id":c.as_str(),"maximum_upgrades":if *c==CardId::SearingBlow{10}else{1}})).collect::<Vec<_>>(),
    "encounters":encounters.iter().map(|(e,a,k)|json!({"id":e.as_str(),"act":a,"kind":k.label()})).collect::<Vec<_>>(),
    "relics":TRAINING_RELICS.iter().map(|r|r.as_str()).collect::<Vec<_>>(),"potions":TRAINING_POTIONS.iter().map(|p|p.as_str()).collect::<Vec<_>>(),"potion_slots":3,
    "relic_counters":{"neows_lament":{"default":3,"allowed":[-2,1,2,3],"requires_equipped":true}},
    "resource_semantics":"already owned; acquisition HP/deck effects included by caller, not repeated", "maximum_catalog_entries":256,"maximum_catalog_bytes":1048576,
    "unsupported":"other characters/ascensions, hidden state/order/RNG, burning elites, unlisted cards/items/encounters; no full standard pool or original-runtime parity"});
    if profile==CombatTrainingProfile::V2{support["unsupported"]=json!("other characters/ascensions, hidden state/order/RNG, burning elites, unlisted cards/items; no complete standard Run/reward pools or original-runtime parity");support["gold"]=json!({"minimum":0,"maximum":u32::MAX,"default":0,"semantics":"already owned; theft removes current gold; killed thief earns an unclaimed stolen-gold receipt"});}
    support}
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct CombatTrainingStep{pub boundary:CombatPolicyBoundary,pub reward:i32,pub terminated:bool,pub truncated:bool}
pub struct CombatTrainingSession{
    catalog:CombatTrainingCatalog,caps:crate::RunCapacities,partition:Partition,id:u64,serial:u64,ordinal:u64,seed:Option<u64>,entry_id:Option<String>,
    record:Option<crate::CombatEpisodeRecord>,latest_public:Option<PublicDecision>,run:Option<Run>,published:Option<CombatPolicyBoundary>,actions:Vec<GameAction>,fault:Option<Error>,closed:bool,accepted_steps:u64,last_step_committed:bool,
}
impl CombatTrainingSession{
    pub fn new(catalog:CombatTrainingCatalog,partition:Partition)->Result<Self,Error>{
        let id=crate::run_session::allocate_run_session_id()?;
        Ok(Self{catalog,caps:crate::combat_policy_capacities(),partition,id,serial:0,ordinal:0,seed:None,entry_id:None,record:None,latest_public:None,run:None,published:None,actions:vec![],fault:None,closed:false,accepted_steps:0,last_step_committed:false})
    }
    pub fn observation_spec(&self)->Result<RunObservationSpec,Error>{match self.catalog.profile{
        CombatTrainingProfile::V1=>crate::combat_policy_observation_spec(),
        CombatTrainingProfile::V2=>crate::combat_policy_observation_spec_v2(),CombatTrainingProfile::M21=>crate::watcher_combat_observation_spec(&self.caps),
    }}
    pub fn catalog_descriptor(&self)->Value{self.catalog.descriptor()}
    pub fn environment_spec(&self)->Result<Value,Error>{
        let spec=self.observation_spec()?;
        let support=match self.catalog.profile{CombatTrainingProfile::V1=>combat_training_support(),CombatTrainingProfile::V2=>combat_training_support_v2(),CombatTrainingProfile::M21=>m21_support()};
        Ok(json!({"profile_id":self.catalog.profile.id(),"schema_id":self.catalog.profile.view_id(),"observation_fingerprint":hex(&spec.fingerprint()),
            "content_id":self.catalog.profile.content_id(),"rules_id":if self.catalog.profile==CombatTrainingProfile::M21{M21_WATCHER_COMBAT_RULES_ID}else{crate::RUN_RULES_ID_M10},"reward_id":COMBAT_TRAINING_REWARD_ID,
            "rng_id":spire_core::combat::rng::RNG_CONTRACT_ID,"seed_adapter_id":crate::RUN_SEED_ADAPTER_ID,"combat_view_id":self.catalog.profile.view_id(),
            "mode":"standalone_combat","partition":self.partition.as_str(),"entry_catalog_fingerprint":hex(&self.catalog.fingerprint()),"autoreset":"disabled","seed_domain":"u64; explicit seed resets episode ordinal to zero; implicit reset advances ordinal",
            "support":support}))
    }
    fn usable(&self)->Result<(),Error>{if self.closed{return Err(Error::Closed)}if let Some(e)=&self.fault{return Err(e.clone())}Ok(())}
    fn freeze(&mut self,e:Error)->Error{self.freeze_action(e,None,false,false)}
    fn freeze_action(&mut self,e:Error,action:Option<GameAction>,committed:bool,gameplay_recorded:bool)->Error{
        if self.fault.is_none(){self.fault=Some(e.clone());if let Some(record)=&mut self.record{record.fault=Some(crate::RunRecordedFault{action,committed,gameplay_recorded,error:crate::RunRecordError::from_error(&e)});}}
        self.fault.clone().unwrap()
    }
    pub fn effective_seed(&self)->Option<u64>{self.seed}
    pub fn preflight_reset(&self,seed:Option<u64>,entry_id:Option<&str>)->Result<(),Error>{
        self.usable()?;self.catalog.entry(entry_id)?;
        self.serial.checked_add(1).ok_or(Error::Input("combat decision serial exhausted"))?;
        if seed.is_none()&&self.seed.is_some(){self.ordinal.checked_add(1).ok_or(Error::Input("combat episode ordinal exhausted"))?;}Ok(())
    }
    pub fn reset(&mut self,seed:Option<u64>,entry_id:Option<&str>)->Result<CombatPolicyBoundary,Error>{
        self.preflight_reset(seed,entry_id)?;
        let (effective,ordinal)=if let Some(seed)=seed{(seed,0)}else if let Some(seed)=self.seed{(seed,self.ordinal+1)}else{
            let mut bytes=[0;8];getrandom::fill(&mut bytes).map_err(|_|Error::Input("combat seed entropy unavailable"))?;(u64::from_le_bytes(bytes),0)
        };
        self.initialize_episode(effective,ordinal,entry_id)
    }
    /// One entry/startup owner for ordinary reset and semantic replay.
    pub(crate) fn initialize_episode(&mut self,effective:u64,ordinal:u64,entry_id:Option<&str>)->Result<CombatPolicyBoundary,Error>{
        self.usable()?;
        self.serial.checked_add(1).ok_or(Error::Input("combat decision serial exhausted"))?;
        let (id,entry)=self.catalog.entry(entry_id)?.clone();
        #[cfg(test)]
        INITIALIZER_CALLS.with(|calls|calls.set(calls.get()+1));
        self.record=if self.catalog.profile==CombatTrainingProfile::M21{Some(crate::combat_replay::new_record(&self.catalog,&self.caps,self.partition,effective,ordinal,&id)?)}else{None};
        let mut root=[0;32];root[..8].copy_from_slice(&effective.to_le_bytes());
        let rng=RunRng::new(root,self.partition,ordinal);
        let encounter=entry.encounter();
        let run=match entry{
            TrainingEntry::Legacy{entry,gold}=>if self.catalog.profile==CombatTrainingProfile::V2{Run::prepare_combat_training_v2_with_gold(entry,gold,rng)}else{Run::prepare_combat_training(entry,rng)},
            TrainingEntry::M21{start,..}=>Run::prepare_m21_watcher_combat(start,encounter,rng),
        }.map_err(Error::Core)?;
        self.run=Some(run);self.seed=Some(effective);self.ordinal=ordinal;self.entry_id=Some(id);self.accepted_steps=0;self.last_step_committed=false;self.serial+=1;self.published=None;self.latest_public=None;self.actions.clear();
        if let Err(e)=self.run.as_mut().unwrap().enter_training_combat(encounter){return Err(self.freeze(Error::Core(e)))}
        let result=self.publish();result.map_err(|e|self.freeze(e))
    }
    fn publish(&mut self)->Result<CombatPolicyBoundary,Error>{
        let d=match self.run.as_ref().unwrap().decision(){Ok(d)=>d,Err(e)=>return Err(Error::Core(e))};
        if self.catalog.profile==CombatTrainingProfile::M21{self.latest_public=Some(d.clone());}
        let cp=self.run.as_ref().unwrap().combat_checkpoint();
        let v2=self.catalog.profile==CombatTrainingProfile::V2;
        let mut observation=match if self.catalog.profile==CombatTrainingProfile::M21{crate::encode_watcher_combat(&d,&self.caps)}else if v2{crate::combat_policy::encode_combat_policy_v2(&d,cp.as_ref())}else{encode_combat_policy(&d,cp.as_ref())}{Ok(o)=>o,Err(e)=>return Err(e)};
        if self.catalog.profile==CombatTrainingProfile::M21&&d.combat.is_none(){if let Some(cp)=&cp{let p=observation.groups.iter_mut().find(|g|g.key=="player").unwrap();p.values[10]=i64::from(cp.hp_before_healing);p.values[11]=i64::from(cp.hp_after_cleanup);p.known[10]=1;p.known[11]=1;}}
        let mut info=if self.catalog.profile==CombatTrainingProfile::M21{crate::combat_policy::lifecycle_for_view(&d,cp.as_ref(),crate::M21_WATCHER_COMBAT_OBSERVATION_SCHEMA_ID)}else if v2{crate::combat_policy::lifecycle_v2(&d,cp.as_ref())}else{lifecycle(&d,cp.as_ref())};info["profile_id"]=json!(self.catalog.profile.id());info["entry_id"]=json!(self.entry_id);info["episode_ordinal"]=json!(self.ordinal);
        info["effective_seed"]=json!(self.seed);info["outcome"]=cp.as_ref().map(|c|json!(match c.record.outcome{Outcome::Victory=>"victory",Outcome::Death=>"death",Outcome::Escaped=>"escaped"})).unwrap_or(Value::Null);
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
        if let Some(record)=&mut self.record{if record.steps.try_reserve(1).is_err(){return Err(self.freeze_action(Error::DeliveryFault("combat recorder allocation failed"),Some(action),false,false))}}
        if let Err(e)=self.run.as_mut().unwrap().step(&action){
            self.last_step_committed=matches!(e,RunError::Faulted(_));
            if self.last_step_committed{self.accepted_steps+=1;self.serial+=1;}
            return Err(self.freeze_action(Error::Core(e),Some(action),self.last_step_committed,false))
        }
        self.last_step_committed=true;
        self.accepted_steps+=1;self.serial+=1;
        let cp=self.run.as_ref().unwrap().combat_checkpoint();let terminated=cp.is_some();
        let reward=cp.map_or(0,|c|if c.record.outcome==Outcome::Victory{1}else{-1});
        if let Some(record)=&mut self.record{record.steps.push(crate::RunRecordedStep{action:action.clone(),reward,terminal:self.run.as_ref().unwrap().decision().ok().and_then(|d|d.terminal)});record.outcome=self.run.as_ref().unwrap().combat_checkpoint().map(|c|c.record.outcome);}
        let boundary=self.publish().map_err(|e|self.freeze_action(e,Some(action),true,true))?;Ok(CombatTrainingStep{boundary,reward,terminated,truncated:false})
    }
    pub fn fail_delivery(&mut self,reason:&'static str){self.freeze(Error::DeliveryFault(reason));}
    pub fn diagnostic(&self)->Value{json!({"fault":self.fault.as_ref().map(|e|match e{
        Error::Core(_)=>format!("Core({})",crate::RunRecordError::from_error(e).message.replace(" (full owner retained in reproduction)","")),
        _=>format!("{e:?}")
    }),"accepted_steps":self.accepted_steps,"last_step_committed":self.last_step_committed,"closed":self.closed,"entry_id":self.entry_id,"effective_seed":self.seed,"episode_ordinal":self.ordinal})}
    pub fn close(&mut self)->Result<(),Error>{self.closed=true;if let Some(record)=&mut self.record{record.closed=true;}Ok(())}
}

/// Typed-only finite Watcher entry; no hidden combat state or pile order.
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct M21WatcherCombatEntry{pub id:String,pub encounter:EncounterId,pub start:M21WatcherCombatStart}
impl CombatTrainingCatalog{
 pub fn m21(entries:Vec<M21WatcherCombatEntry>)->Result<Self,Error>{
  if !(1..=256).contains(&entries.len()){return Err(Error::Input("catalog requires 1..256 entries"))}
  let mut ids=std::collections::BTreeSet::new();let mut owned=Vec::new();let mut normalized=Vec::new();
  for e in entries{
   if e.id.is_empty()||e.id.len()>64||!e.id.bytes().all(|b|b.is_ascii_alphanumeric()||b==b'_'||b==b'-')||!ids.insert(e.id.clone()){return Err(Error::Input("invalid or duplicate combat entry ID"))}
   if m21_watcher_combat_encounter_metadata(e.encounter).is_none(){return Err(Error::Input("unsupported M21 encounter"))}
   let start=e.start.normalized().map_err(Error::Core)?;
   normalized.push(json!({"id":e.id,"encounter":e.encounter.as_str(),"start":m21_start_descriptor(&start)}));
   owned.push((e.id,TrainingEntry::M21{encounter:e.encounter,start}));
  }
  let descriptor=json!({"catalog_schema_id":M21_WATCHER_COMBAT_CATALOG_SCHEMA_ID,"profile_id":M21_WATCHER_COMBAT_PROFILE_ID,"entries":normalized});
  let bytes=crate::canonical_json(&descriptor).map_err(|_|Error::Input("invalid M21 catalog"))?;
  if bytes.len()>1_048_576{return Err(Error::Input("combat catalog exceeds 1 MiB"))}
  let mut h=Sha256::new();h.update(b"spire-gym/m21-watcher-combat-catalog/v1\0");h.update(bytes);
  Ok(Self{profile:CombatTrainingProfile::M21,entries:owned,descriptor,fingerprint:h.finalize().into()})
 }
 pub(crate) fn m21_from_descriptor(d:&Value)->Result<Self,Error>{
  let raw:RawM21Catalog=serde_json::from_value(d.clone()).map_err(|_|Error::IncompatibleReplay("invalid M21 catalog descriptor"))?;
  if raw.catalog_schema_id!=M21_WATCHER_COMBAT_CATALOG_SCHEMA_ID||raw.profile_id!=M21_WATCHER_COMBAT_PROFILE_ID{return Err(Error::IncompatibleReplay("M21 catalog identity mismatch"))}
  let entries=raw.entries.into_iter().map(|e|{
   let encounter=lookup(&e.encounter,TRAINING_ENCOUNTERS_V2.iter().map(|x|x.0).collect::<Vec<_>>().as_slice(),EncounterId::as_str)?;
   let start=match e.start{RawM21Start::Canonical=>M21WatcherCombatStart::Canonical,RawM21Start::Supplied{resources:r}=>{
    let deck=r.deck.into_iter().map(|c|Ok(CardInstance{instance:c.instance,id:lookup(&c.card,watcher::M21_WATCHER_CARDS,CardId::as_str)?,upgrades:c.upgrades,misc:c.misc})).collect::<Result<Vec<_>,Error>>()?;
    let relics=r.relics.iter().map(|s|lookup(s,M21_WATCHER_RELICS,RelicId::as_str)).collect::<Result<Vec<_>,_>>()?;
    let potions=r.potions.iter().map(|p|p.as_ref().map(|s|lookup(s,M21_WATCHER_POTIONS,PotionId::as_str)).transpose()).collect::<Result<Vec<_>,_>>()?;
    let relic_counters=r.relic_counters.iter().map(|(s,n)|Ok((lookup(s,M21_WATCHER_RELICS,RelicId::as_str)?,*n))).collect::<Result<_,Error>>()?;
    M21WatcherCombatStart::Supplied(AuthoredRunStart{hp:r.hp,max_hp:r.max_hp,gold:r.gold,deck,relics,potions,relic_counters})
   }};Ok(M21WatcherCombatEntry{id:e.id,encounter,start})
  }).collect::<Result<Vec<_>,Error>>().map_err(|_|Error::IncompatibleReplay("invalid M21 catalog content"))?;
  let rebuilt=Self::m21(entries).map_err(|_|Error::IncompatibleReplay("invalid M21 catalog resources"))?;
  if rebuilt.descriptor!=*d{return Err(Error::IncompatibleReplay("M21 catalog is not normalized"))}Ok(rebuilt)
 }
 pub(crate) fn m21_selected_entry(&self,id:&str)->Result<Value,Error>{
  self.entry(Some(id)).map_err(|_|Error::IncompatibleReplay("unknown recorded M21 entry ID"))?;
  self.descriptor["entries"].as_array().unwrap().iter().find(|e|e["id"].as_str()==Some(id)).cloned().ok_or(Error::IncompatibleReplay("missing recorded M21 entry"))
 }
}

pub const M21_WATCHER_COMBAT_PROFILE_ID:&str="m21-watcher-combat-v1";
pub const M21_WATCHER_COMBAT_CONTENT_ID:&str="spire-gym-m21-watcher-combat-content-v1";
pub const M21_WATCHER_COMBAT_RULES_ID:&str="spire-gym-native-m21-watcher-combat-rules-v1";
pub const M21_WATCHER_COMBAT_CATALOG_SCHEMA_ID:&str="spire-gym-m21-watcher-combat-catalog-v1";
#[derive(Clone,Debug)]
enum TrainingEntry{Legacy{entry:CombatTrainingEntry,gold:u32},M21{encounter:EncounterId,start:M21WatcherCombatStart}}
impl TrainingEntry{fn encounter(&self)->EncounterId{match self{Self::Legacy{entry,..}=>entry.encounter,Self::M21{encounter,..}=>*encounter}}}
fn m21_start_descriptor(s:&M21WatcherCombatStart)->Value{match s{
 M21WatcherCombatStart::Canonical=>json!({"kind":"canonical"}),
 M21WatcherCombatStart::Supplied(r)=>json!({"kind":"supplied","resources":{"hp":r.hp,"max_hp":r.max_hp,"gold":r.gold,
 "deck":r.deck.iter().map(|c|json!({"instance":c.instance,"card":c.id.as_str(),"upgrades":c.upgrades,"misc":c.misc})).collect::<Vec<_>>(),
 "relics":r.relics.iter().map(|id|id.as_str()).collect::<Vec<_>>(),"potions":r.potions.iter().map(|p|p.map(PotionId::as_str)).collect::<Vec<_>>(),
 "relic_counters":r.relic_counters.iter().map(|(id,n)|(id.as_str(),*n)).collect::<std::collections::BTreeMap<_,_>>()}})
}}
#[derive(Deserialize)]#[serde(deny_unknown_fields)]
struct RawM21Catalog{catalog_schema_id:String,profile_id:String,entries:Vec<RawM21Entry>}
#[derive(Deserialize)]#[serde(deny_unknown_fields)]
struct RawM21Entry{id:String,encounter:String,start:RawM21Start}
#[derive(Deserialize)]#[serde(tag="kind",rename_all="snake_case",deny_unknown_fields)]
enum RawM21Start{Canonical,Supplied{resources:RawM21Resources}}
#[derive(Deserialize)]#[serde(deny_unknown_fields)]
struct RawM21Resources{hp:u32,max_hp:u32,gold:u32,deck:Vec<RawM21Card>,relics:Vec<String>,potions:Vec<Option<String>>,relic_counters:std::collections::BTreeMap<String,i32>}
#[derive(Deserialize)]#[serde(deny_unknown_fields)]
struct RawM21Card{instance:u64,card:String,upgrades:u32,misc:i32}
fn m21_support()->Value{json!({"character":"watcher","ascension":0,"cards":watcher::M21_WATCHER_CARDS.iter().map(|id|id.as_str()).collect::<Vec<_>>(),"generated_only":watcher::M21_WATCHER_GENERATED.iter().map(|id|id.as_str()).collect::<Vec<_>>(),"relics":M21_WATCHER_RELICS.iter().map(|id|id.as_str()).collect::<Vec<_>>(),"potions":M21_WATCHER_POTIONS.iter().map(|id|id.as_str()).collect::<Vec<_>>(),"encounters":TRAINING_ENCOUNTERS_V2.iter().map(|(id,act,kind)|json!({"id":id.as_str(),"act":act,"kind":kind.label()})).collect::<Vec<_>>(),"resources":"canonical bypasses normal progression; supplied diagnostic already-owned resources; no hidden order or RNG"})}
impl CombatTrainingSession{
 pub fn new_m21(catalog:CombatTrainingCatalog,partition:Partition,caps:crate::RunCapacities)->Result<Self,Error>{
  if catalog.profile!=CombatTrainingProfile::M21{return Err(Error::Input("new_m21 requires M21 catalog"))}
  crate::run_schema_m21::validate(&caps)?;let mut s=Self::new(catalog,partition)?;s.caps=caps;Ok(s)
 }
}
#[cfg(test)]
thread_local!{static INITIALIZER_CALLS:std::cell::Cell<usize>=const{std::cell::Cell::new(0)};}
impl CombatTrainingSession{
 pub fn record(&self)->Result<Option<crate::CombatEpisodeRecord>,Error>{Ok(self.record.clone())}
 pub(crate) fn semantic_index(&self,action:&GameAction)->Result<RunActionIndex,Error>{self.actions.iter().position(|a|a==action).map(|i|RunActionIndex(i as i64)).ok_or(Error::IncompatibleReplay("combat action is not currently offered"))}
 pub(crate) fn replay_report(&self,rewards:Vec<i32>,reported_fault:Option<crate::RunRecordedFault>)->crate::CombatReplayReport{
  crate::CombatReplayReport{final_decision:self.run.as_ref().and_then(|r|r.decision().ok()).or_else(||self.latest_public.clone()),final_decision_error:self.fault.clone(),final_boundary:self.published.clone(),accepted_count:usize::try_from(self.accepted_steps).expect("accepted count is bounded by native record allocation"),rewards,outcome:self.run.as_ref().and_then(|r|r.combat_checkpoint()).map(|c|c.record.outcome),reported_fault}
 }
}
#[cfg(test)]
mod m21_replay_preflight_tests{
 use super::*;
 fn catalog()->CombatTrainingCatalog{CombatTrainingCatalog::m21(vec![M21WatcherCombatEntry{id:"A".into(),encounter:EncounterId::JawWorm,start:M21WatcherCombatStart::Canonical},M21WatcherCombatEntry{id:"B".into(),encounter:EncounterId::Cultist,start:M21WatcherCombatStart::Supplied(AuthoredRunStart{hp:60,max_hp:72,gold:7,deck:vec![CardInstance{id:CardId::DefendPurple,instance:31,upgrades:0,misc:0}],relics:vec![],potions:vec![None;3],relic_counters:Default::default()})}]).unwrap()}
 #[test]
 fn m21_watcher_replay_checks_catalog_before_any_initializer(){
  let mut s=CombatTrainingSession::new(catalog(),Partition::Test).unwrap();s.reset(Some(7),Some("A")).unwrap();s.reset(None,Some("B")).unwrap();let r=s.record().unwrap().unwrap();
  assert_eq!(r.ordinal,1);let mut bad=Vec::new();
  let mut v=r.clone();v.catalog_descriptor["entries"][0]["encounter"]=json!("cultist");bad.push(v);
  let mut v=r.clone();v.catalog_descriptor["entries"].as_array_mut().unwrap().reverse();bad.push(v);
  let mut v=r.clone();v.catalog_descriptor["entries"][1]["id"]=json!("A");bad.push(v);
  let mut v=r.clone();v.catalog_descriptor["entries"][0]["start"]["hidden_draw"]=json!([]);bad.push(v);
  let mut v=r.clone();v.catalog_descriptor["catalog_schema_id"]=json!("wrong");bad.push(v);
  let mut v=r.clone();v.entry_id="A".into();bad.push(v);let mut v=r.clone();v.entry_id="missing".into();bad.push(v);
  let mut v=r.clone();v.selected_entry["start"]["resources"]["deck"][0]["instance"]=json!(32);bad.push(v);
  let mut v=r.clone();v.profile_id="wrong".into();bad.push(v);let mut v=r.clone();v.observation_fingerprint=[0;32];bad.push(v);
  let mut v=r.clone();v.capacities.actions=0;bad.push(v);
  for v in bad{INITIALIZER_CALLS.with(|n|n.set(0));assert!(matches!(crate::replay_m21_combat(&v,crate::combat_policy_capacities()),Err(Error::IncompatibleReplay(_))));assert_eq!(INITIALIZER_CALLS.with(|n|n.get()),0);assert!(v.to_json().is_err());}
  let mut json:Value=serde_json::from_slice(&r.to_json().unwrap()).unwrap();json["catalog_descriptor"]["entries"][0]["encounter"]=json!("cultist");INITIALIZER_CALLS.with(|n|n.set(0));assert!(crate::CombatEpisodeRecord::from_json(&serde_json::to_vec(&json).unwrap()).is_err());assert_eq!(INITIALIZER_CALLS.with(|n|n.get()),0);
  INITIALIZER_CALLS.with(|n|n.set(0));let report=crate::replay_m21_combat(&r,crate::combat_policy_capacities()).unwrap();assert_eq!(INITIALIZER_CALLS.with(|n|n.get()),1);assert_eq!(report.final_boundary.unwrap().info["episode_ordinal"],1);
 }
 #[test]
 fn m21_watcher_capacity_fault_keeps_owner_and_record_after_close(){
  let mut caps=crate::combat_policy_capacities();caps.actions=1;let mut s=CombatTrainingSession::new_m21(catalog(),Partition::Test,caps.clone()).unwrap();
  let e=s.reset(Some(7),Some("A")).unwrap_err();assert!(matches!(e,Error::Capacity{group:"actions",..}));assert!(s.run.is_some());assert_eq!(s.run.as_ref().unwrap().decision().unwrap().combat_progress.entered,1);
  let before=s.run.as_ref().unwrap().decision().unwrap();let record=s.record().unwrap().unwrap();assert_eq!(record.steps.len(),0);assert!(record.outcome.is_none());assert!(!record.fault.as_ref().unwrap().committed);
  assert!(s.reset(Some(8),Some("B")).is_err());assert_eq!(s.run.as_ref().unwrap().decision().unwrap(),before);assert_eq!(s.serial,1);assert_eq!(s.seed,Some(7));
  s.close().unwrap();let r=s.record().unwrap().unwrap();assert!(r.closed);assert!(crate::CombatEpisodeRecord::from_json(&r.to_json().unwrap()).is_ok());let replay=crate::replay_m21_combat(&r,caps).unwrap();assert_eq!(replay.final_decision,Some(before));assert_eq!(replay.reported_fault,r.fault);
 }
}
