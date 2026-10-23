use super::{catalog, canonical_hash, combat_action_value, entry_value, game_action_value, m1, m5, m6};
use serde_json::{json, Value};
use spire_core::{combat::{Combat, CombatAction, CombatPublic, Outcome}, content::{EnemyId, EncounterId, EventId}, run::*};
use std::hint::black_box;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CaseId { Combat, Act1, Act2Prefix }
impl CaseId {
    pub const ALL: [Self;3] = [Self::Combat, Self::Act1, Self::Act2Prefix];
    pub fn as_str(self) -> &'static str { match self {
        Self::Combat=>"combat.ironclad-a0.jaw-worm", Self::Act1=>"run.ironclad-a0.act1-guardian",
        Self::Act2Prefix=>"run.ironclad-a0.act2-prefix-champ"
    }}
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Boundary { Native, Public }
impl Boundary { pub fn as_str(self)->&'static str { match self { Self::Native=>"native-transitions", Self::Public=>"public-decision-loop" } } }

#[derive(Clone)]
pub enum BenchAction { Combat(CombatAction), Run(GameAction) }
impl BenchAction { pub fn value(&self)->Value { match self { Self::Combat(a)=>combat_action_value(a), Self::Run(a)=>game_action_value(a) } } }

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Checkpoint { Combat(CombatPublic, Vec<CombatAction>), Run(PublicDecision, Option<map::PublicMap>) }

pub struct PreparedCase {
    pub id: CaseId, pub contract: Value, pub input: Value, pub actions: Vec<BenchAction>,
    pub checkpoints: Vec<Checkpoint>, pub rewards: Vec<i32>, pub expected: Value,
}
impl PreparedCase {
    pub fn action_values(&self)->Vec<Value> { self.actions.iter().map(BenchAction::value).collect() }
    pub fn actions_sha256(&self)->String { canonical_hash(&json!(self.action_values())) }
}

pub enum BenchState { Combat(Box<Combat>), Run(Box<Run>) }
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ReplayTotals { pub accepted: usize, pub reward: i32 }

fn error(e: impl std::fmt::Debug) -> String { format!("{e:?}") }

impl BenchState {
    pub fn checkpoint(&self)->Result<Checkpoint,String> { match self {
        Self::Combat(c)=>Ok(Checkpoint::Combat(c.public_full().map_err(error)?,c.legal_actions_full().map_err(error)?)),
        Self::Run(r)=>Ok(Checkpoint::Run(r.decision().map_err(error)?,r.public_map())),
    }}
    fn public_boundary(&self)->Result<(),String> { match self {
        Self::Combat(c)=> { black_box(c.public_full().map_err(error)?); black_box(c.legal_actions_full().map_err(error)?); }
        Self::Run(r)=> { let d=r.decision().map_err(error)?; if d.room.as_ref().is_some_and(|r|r.kind==RoomViewKind::Map) { black_box(r.public_map()); } black_box(d); }
    } Ok(()) }
    pub fn step(&mut self, action:&BenchAction, boundary:Boundary)->Result<i32,String> {
        if boundary==Boundary::Public { self.public_boundary()?; }
        match (self,action) {
            (Self::Combat(c),BenchAction::Combat(a))=> { c.step_full(black_box(a)).map_err(error)?; Ok(0) }
            (Self::Run(r),BenchAction::Run(a))=> { let t=r.step(black_box(a)).map_err(error)?; black_box(&t); Ok(t.reward) }
            _=>Err("action/state mismatch".into()),
        }
    }
    pub fn replay(&mut self, case:&PreparedCase, boundary:Boundary)->Result<ReplayTotals,String> {
        let mut totals=ReplayTotals::default();
        for action in &case.actions { totals.reward+=self.step(action,boundary)?; totals.accepted+=1; }
        Ok(totals)
    }
    pub fn endpoint(&self, totals:ReplayTotals)->Result<Value,String> { match self {
        Self::Combat(c)=> { let v=c.public_full().map_err(error)?; Ok(json!({
            "outcome":match v.outcome {Some(Outcome::Victory)=>"victory",Some(Outcome::Death)=>"death",None=>"unfinished"},
            "hp":v.player.hp,"enemy_hp":v.enemies.iter().map(|e|e.hp).sum::<i32>(),
            "candidates":c.legal_actions_full().map_err(error)?.len(),"accepted_actions":totals.accepted,"reward":totals.reward })) }
        Self::Run(r)=> { let d=r.decision().map_err(error)?; let mut value=run_resources(&d);
            value["outcome"]=json!(match d.room.as_ref().map(|r|r.kind) { Some(RoomViewKind::Act2Ready)=>"Act2Ready",Some(RoomViewKind::Act2PrefixComplete)=>"Act2PrefixComplete",_=>"unfinished" });
            value["act"]=json!(d.act); value["floor"]=json!(d.floor); value["reward"]=json!(d.reward_total);
            value["transition_reward"]=json!(totals.reward);value["accepted_actions"]=json!(totals.accepted);
            value["candidates"]=json!(d.candidates.len()); value["terminal"]=terminal_value(d.terminal);
            Ok(value) }
    }}
}

fn terminal_value(t:Option<RunTerminal>)->Value { match t {None=>Value::Null,Some(t)=>json!(match t {
    RunTerminal::HeartVictory=>"heart-victory",RunTerminal::Death=>"death",RunTerminal::EmptyDeck=>"empty-deck",RunTerminal::Act3NoKeys=>"act3-no-keys"
})} }

fn run_resources(d:&PublicDecision)->Value { json!({"hp":d.player.hp,"max_hp":d.player.max_hp,"gold":d.player.gold,
    "deck":d.objects.iter().map(|c|json!([c.instance,c.id.as_str(),c.upgrades])).collect::<Vec<_>>(),
    "relics":d.player.relics.iter().map(|r|r.as_str()).collect::<Vec<_>>(),
    "potions":d.player.potions.iter().map(|p|p.map(|p|p.as_str())).collect::<Vec<_>>(),"keys":d.player.keys,
    "combats_entered":d.combat_progress.entered,"combats_settled":d.combat_progress.settled}) }

fn map_value(map:Option<map::PublicMap>)->Value { match map {None=>Value::Null,Some(m)=>json!({
    "boss":m.boss.map(|b|b.as_str()),"act":m.act,"current":m.current.map(|n|n.0),"winged_moves":m.winged_moves,
    "nodes":m.nodes.iter().map(|n|json!({"id":n.id.0,"floor":n.floor,"x":n.x,"y":n.y,"kind":node_kind(n.kind),
       "resolved":n.resolved_room.map(node_kind),"edges":n.edges.iter().map(|n|n.0).collect::<Vec<_>>(),
       "visited":n.visited,"revealed":n.revealed,"burning":n.burning})).collect::<Vec<_>>()
})} }
fn node_kind(k:MapNodeKind)->&'static str { match k { MapNodeKind::Monster=>"monster",MapNodeKind::Elite=>"elite",MapNodeKind::Shop=>"shop",MapNodeKind::Rest=>"rest",MapNodeKind::Question=>"question",MapNodeKind::Treasure=>"treasure",MapNodeKind::Boss=>"boss",MapNodeKind::Event=>"event" } }

pub fn fresh(case:&PreparedCase)->Result<BenchState,String> { match case.id {
    CaseId::Combat=>Ok(BenchState::Combat(Box::new(Combat::enter(m1::fixture(EnemyId::JawWorm,7)).map_err(error)?))),
    CaseId::Act1=>Ok(BenchState::Run(Box::new(m5::run(7,m5::Scenario::Guardian)))),
    CaseId::Act2Prefix=>Ok(BenchState::Run(Box::new(m6::start(7,vec!["Automaton","Collector"])?))),
} }

pub fn prepare(id:CaseId)->Result<PreparedCase,String> {
    let contract=catalog()["cases"].as_array().unwrap().iter().find(|c|c["id"]==id.as_str()).unwrap().clone();
    let mut case=PreparedCase{id,contract,input:Value::Null,actions:vec![],checkpoints:vec![],rewards:vec![],expected:Value::Null};
    let reference:Vec<Checkpoint>;
    match id {
        CaseId::Combat=> {
            let entry=m1::fixture(EnemyId::JawWorm,7);case.input=json!({"entry":entry_value(&entry)?});
            let mut c=Combat::enter(entry).map_err(error)?;let mut observations=vec![];
            m1::run_fight(&mut c,true,4096,|view,menu,action| {
                observations.push(Checkpoint::Combat(view.clone(),menu.to_vec()));
                if let Some(a)=action {case.actions.push(BenchAction::Combat(a.clone()));case.rewards.push(0);}
            }).map_err(error)?;
            reference=observations;
        }
        CaseId::Act1=> {
            let mut r=m5::run(7,m5::Scenario::Guardian);let initial=Checkpoint::Run(r.decision().map_err(error)?,r.public_map());
            let mut caller=m5::Caller::new(4096);m5::rollout(&mut r,&mut caller,m5::Scenario::Guardian)?;
            let mut observations=vec![initial];
            for (a,t,d) in caller.trace {case.actions.push(BenchAction::Run(a));case.rewards.push(t.reward);observations.push(Checkpoint::Run(d,None));}
            reference=observations;
        }
        CaseId::Act2Prefix=> {
            let r=m6::prefix(7,vec!["Automaton","Collector"],4096)?;
            let act2:Vec<_>=r.combats.iter().filter(|(act,_)|*act==2).map(|(_,r)|r).collect();
            if !act2.iter().any(|r|r.encounter==EncounterId::GremlinLeader&&r.outcome==Outcome::Victory) ||
               act2.iter().filter(|r|matches!(r.encounter,EncounterId::SphericGuardian|EncounterId::ThreeByrds|EncounterId::CenturionAndHealer|EncounterId::Snecko|EncounterId::ChosenAndByrds|EncounterId::ThreeCultists)&&r.outcome==Outcome::Victory).count()<2 ||
               !r.events.contains(&EventId::AncientWriting) { return Err("missing actual Act2 encounters/event".into()); }
            let start=m6::start(7,vec!["Automaton","Collector"])?;
            let mut observations=vec![Checkpoint::Run(start.decision().map_err(error)?,start.public_map())];
            for s in r.trace {case.actions.push(BenchAction::Run(s.action));case.rewards.push(s.reward);observations.push(Checkpoint::Run(s.decision,s.map));}
            reference=observations;
        }
    }
    if case.actions.is_empty() || case.actions.len()>4096 { return Err("unfinished/invalid script length".into()); }
    let mut state=fresh(&case)?;
    if id!=CaseId::Combat {
        let BenchState::Run(r)=&state else {unreachable!()};let d=r.decision().map_err(error)?;
        let mut p=CreationProfile::experienced();p.neow_eligible=false;p.seen_bosses[0]=vec!["Hexaghost","SlimeBoss"];
        if id==CaseId::Act2Prefix {p.seen_bosses[1]=vec!["Automaton","Collector"];}
        let mut root=[0u8;32];root[..8].copy_from_slice(&7u64.to_le_bytes());
        case.input=json!({"config":{"character":"ironclad","ascension":0},"profile":{
            "neow_eligible":p.neow_eligible,"seen_bosses":p.seen_bosses,"stored_note":[p.stored_note.0.as_str(),p.stored_note.1],
            "fully_unlocked":p.fully_unlocked,"act4_enabled":p.act4_enabled},
            "room_scenario":case.contract["profile"],"root":root,"partition":"test","ordinal":0,
            "initial_resources":run_resources(&d),"initial_map":map_value(r.public_map())});
    }
    let mut totals=ReplayTotals::default();
    for index in 0..=case.actions.len() {
        let observed=state.checkpoint()?;
        let matches=match (&observed,&reference[index]) {
            (Checkpoint::Run(d,_),Checkpoint::Run(expected,_)) if id==CaseId::Act1=>d==expected,
            _=>observed==reference[index],
        };
        if !matches {return Err(format!("public policy replay diverged at {index}"));}
        case.checkpoints.push(observed);
        if index<case.actions.len() {let reward=state.step(&case.actions[index],Boundary::Native)?;
            if reward!=case.rewards[index] {return Err("policy reward mismatch".into());}totals.accepted+=1;totals.reward+=reward;}
    }
    case.expected=state.endpoint(totals)?;
    check_expected(&case)?;
    Ok(case)
}

fn check_expected(case:&PreparedCase)->Result<(),String> {
    for (field,expected) in case.contract["expected"].as_object().unwrap() {
        if case.expected.get(field)!=Some(expected) {return Err(format!("behavior changed: {field}, got {}, expected {expected}",case.expected[field]));}
    } Ok(())
}

pub fn preflight(case:&PreparedCase)->Result<Value,String> {
    if case.actions.len()+1!=case.checkpoints.len() || case.rewards.len()!=case.actions.len() { return Err("script/checkpoint length mismatch".into()); }
    for boundary in [Boundary::Native,Boundary::Public] {
        let mut state=fresh(case)?;let mut totals=ReplayTotals::default();
        if state.checkpoint()?!=case.checkpoints[0] {return Err("initial state changed".into());}
        for (i,action) in case.actions.iter().enumerate() {
            let reward=state.step(action,boundary)?;
            if reward!=case.rewards[i] || state.checkpoint()?!=case.checkpoints[i+1] {return Err(format!("preflight divergence at action{i}"));}
            totals.accepted+=1;totals.reward+=reward;
        }
        if state.endpoint(totals)?!=case.expected {return Err("endpoint changed".into());}
    }
    check_expected(case)?;
    Ok(case.expected.clone())
}
