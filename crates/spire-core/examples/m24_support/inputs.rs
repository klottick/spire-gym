//! Fixed canonical M24 inputs and parse-only validation; no seed/case lookup in the chooser.
use spire_core::{combat::rng::Partition,content::{Character,EncounterId},run::*};
pub const ACTION_LIMIT_MAX:usize=32768;
pub const SEEDS:[u64;2]=[7,11];
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Scenario{AwakenedOne,TimeEater,DonuAndDeca}
pub const SCENARIOS:[Scenario;3]=[Scenario::AwakenedOne,Scenario::TimeEater,Scenario::DonuAndDeca];
impl Scenario{
    pub fn name(self)->&'static str{match self{Self::AwakenedOne=>"awakened-one",Self::TimeEater=>"time-eater",Self::DonuAndDeca=>"donu-and-deca"}}
    pub fn act3(self)->EncounterId{match self{Self::AwakenedOne=>EncounterId::AwakenedOne,Self::TimeEater=>EncounterId::TimeEater,Self::DonuAndDeca=>EncounterId::DonuAndDeca}}
}
/// Design §11 triples: (seed, Act3 target) -> (Act1, Act2). `death` is the seed-7
/// Guardian/Champ/AwakenedOne nonattacking control.
pub fn triple(seed:u64,scenario:Scenario,death:bool)->Result<(EncounterId,EncounterId),String>{
    use EncounterId as E;
    if death{return if seed==7&&scenario==Scenario::AwakenedOne{Ok((E::Guardian,E::Champ))}else{Err("M24 death control is seed7 Guardian/Champ/AwakenedOne only".into())}}
    Ok(match (seed,scenario){
        (7,Scenario::AwakenedOne)=>(E::Guardian,E::Champ),(7,Scenario::TimeEater)=>(E::Hexaghost,E::BronzeAutomaton),(7,Scenario::DonuAndDeca)=>(E::SlimeBoss,E::Collector),
        (11,Scenario::DonuAndDeca)=>(E::Guardian,E::BronzeAutomaton),(11,Scenario::AwakenedOne)=>(E::Hexaghost,E::Collector),(11,Scenario::TimeEater)=>(E::SlimeBoss,E::Champ),
        _=>return Err("M24 seed must be 7 or 11".into()),
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
            "--scenario"=>{o.nonattacking=value=="death";o.scenarios=match value.as_str(){"all"=>SCENARIOS.to_vec(),"awakened-one"|"death"=>vec![Scenario::AwakenedOne],"time-eater"=>vec![Scenario::TimeEater],"donu-and-deca"=>vec![Scenario::DonuAndDeca],_=>return Err("unsupported M24 scenario".into())};},
            _=>return Err(format!("unsupported flag {flag}")),
        }
    }
    if !SEEDS.contains(&o.seed){return Err("M24 seed must be 7 or 11".into())}
    if o.budget>ACTION_LIMIT_MAX{return Err("M24 budget must be 0..=32768".into())}
    for s in &o.scenarios{triple(o.seed,*s,o.nonattacking)?;}
    Ok(o)
}
/// Exact creation history: all three unseen targets installed before construction.
pub fn creation(seed:u64,scenario:Scenario,death:bool)->Result<CreationProfile,String>{
    let (a1,a2)=triple(seed,scenario,death)?;let mut p=CreationProfile::experienced();p.neow_eligible=false;
    p.seen_bosses[0]=match a1{EncounterId::Guardian=>vec!["Hexaghost","SlimeBoss"],EncounterId::Hexaghost=>vec!["Guardian","SlimeBoss"],_=>vec!["Guardian","Hexaghost"]};
    p.seen_bosses[1]=match a2{EncounterId::Champ=>vec!["Automaton","Collector"],EncounterId::BronzeAutomaton=>vec!["Champ","Collector"],_=>vec!["Champ","Automaton"]};
    p.seen_bosses[2]=match scenario{Scenario::AwakenedOne=>vec!["TimeEater","DonuAndDeca"],Scenario::TimeEater=>vec!["AwakenedOne","DonuAndDeca"],Scenario::DonuAndDeca=>vec!["AwakenedOne","TimeEater"]};Ok(p)
}
pub fn make_run(seed:u64,scenario:Scenario,death:bool)->Result<Run,String>{
    let p=creation(seed,scenario,death)?;let mut root=[0;32];root[..8].copy_from_slice(&seed.to_le_bytes());
    Run::new_room_scenario(RunConfig{character:Character::Watcher,ascension:0},p,RunRng::new(root,Partition::Test,0),RoomScenario::M24WatcherAct3).map_err(|e|format!("{e:?}"))
}
