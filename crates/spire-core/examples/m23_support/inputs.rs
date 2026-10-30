//! Fixed canonical M23 inputs and parse-only validation; no rollout, no seed/case lookup in the chooser.
use spire_core::{combat::rng::Partition,content::{Character,EncounterId},run::*};
pub const ACTION_LIMIT_MAX:usize=32768;
pub const SEEDS:[u64;2]=[7,11];
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Scenario{Champ,BronzeAutomaton,Collector}
pub const SCENARIOS:[Scenario;3]=[Scenario::Champ,Scenario::BronzeAutomaton,Scenario::Collector];
impl Scenario{
    pub fn name(self)->&'static str{match self{Self::Champ=>"champ",Self::BronzeAutomaton=>"bronze-automaton",Self::Collector=>"collector"}}
    pub fn act2(self)->EncounterId{match self{Self::Champ=>EncounterId::Champ,Self::BronzeAutomaton=>EncounterId::BronzeAutomaton,Self::Collector=>EncounterId::Collector}}
}
/// Exact approved (seed, Act2) -> Act1 pairing; `death` is the seed7 Guardian->Champ control.
pub fn act1(seed:u64,scenario:Scenario,death:bool)->Result<EncounterId,String>{
    if death{return if seed==7&&scenario==Scenario::Champ{Ok(EncounterId::Guardian)}else{Err("M23 death control is seed7 Guardian->Champ only".into())}}
    Ok(match (seed,scenario){
        (7,Scenario::Champ)=>EncounterId::Guardian,(7,Scenario::BronzeAutomaton)=>EncounterId::Hexaghost,(7,Scenario::Collector)=>EncounterId::SlimeBoss,
        (11,Scenario::Champ)=>EncounterId::SlimeBoss,(11,Scenario::BronzeAutomaton)=>EncounterId::Guardian,(11,Scenario::Collector)=>EncounterId::Hexaghost,
        _=>return Err("M23 seed must be 7 or 11".into()),
    })
}
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct Options{pub seed:u64,pub budget:usize,pub scenarios:Vec<Scenario>,pub nonattacking:bool,pub prepare_only:bool}
pub fn parse_args(args:impl IntoIterator<Item=String>)->Result<Options,String>{
    let mut o=Options{seed:7,budget:ACTION_LIMIT_MAX,scenarios:SCENARIOS.to_vec(),nonattacking:false,prepare_only:false};let mut args=args.into_iter();
    while let Some(flag)=args.next(){if flag=="--prepare-only"{o.prepare_only=true;continue}let value=args.next().ok_or_else(||format!("missing value for {flag}"))?;
        match flag.as_str(){
            "--seed"=>o.seed=value.parse().map_err(|_|"invalid seed")?,
            "--budget"=>o.budget=value.parse().map_err(|_|"invalid budget")?,
            "--scenario"=>{o.nonattacking=value=="death";o.scenarios=match value.as_str(){"all"=>SCENARIOS.to_vec(),"champ"|"death"=>vec![Scenario::Champ],"bronze-automaton"=>vec![Scenario::BronzeAutomaton],"collector"=>vec![Scenario::Collector],_=>return Err("unsupported M23 scenario".into())};},
            _=>return Err(format!("unsupported flag {flag}")),
        }
    }
    if !SEEDS.contains(&o.seed){return Err("M23 seed must be 7 or 11".into())}
    if o.budget>ACTION_LIMIT_MAX{return Err("M23 budget must be 0..=32768".into())}
    for s in &o.scenarios{act1(o.seed,*s,o.nonattacking)?;}
    Ok(o)
}
/// Exact matched creation history: Act1 and Act2 unseen targets; Act3 stays experienced().
pub fn creation(seed:u64,scenario:Scenario,death:bool)->Result<CreationProfile,String>{
    let mut p=CreationProfile::experienced();p.neow_eligible=false;
    p.seen_bosses[0]=match act1(seed,scenario,death)?{EncounterId::Guardian=>vec!["Hexaghost","SlimeBoss"],EncounterId::Hexaghost=>vec!["Guardian","SlimeBoss"],_=>vec!["Guardian","Hexaghost"]};
    p.seen_bosses[1]=match scenario{Scenario::Champ=>vec!["Automaton","Collector"],Scenario::BronzeAutomaton=>vec!["Champ","Collector"],Scenario::Collector=>vec!["Champ","Automaton"]};Ok(p)
}
pub fn make_run(seed:u64,scenario:Scenario,death:bool)->Result<Run,String>{
    let p=creation(seed,scenario,death)?;let mut root=[0;32];root[..8].copy_from_slice(&seed.to_le_bytes());
    Run::new_room_scenario(RunConfig{character:Character::Watcher,ascension:0},p,RunRng::new(root,Partition::Test,0),RoomScenario::M23WatcherAct2).map_err(|e|format!("{e:?}"))
}
