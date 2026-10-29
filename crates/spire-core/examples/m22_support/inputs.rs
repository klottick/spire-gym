//! Fixed canonical inputs and parse-only validation; no rollout.
use spire_core::{combat::rng::Partition,content::{Character,EncounterId},run::*};
pub const ACTION_LIMIT_MAX:usize=32768;
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Scenario{Guardian,Hexaghost,SlimeBoss}
pub const SCENARIOS:[Scenario;3]=[Scenario::Guardian,Scenario::Hexaghost,Scenario::SlimeBoss];
impl Scenario{
    pub fn name(self)->&'static str{match self{Self::Guardian=>"guardian",Self::Hexaghost=>"hexaghost",Self::SlimeBoss=>"slime-boss"}}
    pub fn encounter(self)->EncounterId{match self{Self::Guardian=>EncounterId::Guardian,Self::Hexaghost=>EncounterId::Hexaghost,Self::SlimeBoss=>EncounterId::SlimeBoss}}
}
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct Options{pub seed:u64,pub budget:usize,pub scenarios:Vec<Scenario>,pub nonattacking:bool,pub prepare_only:bool}
pub fn parse_args(args:impl IntoIterator<Item=String>)->Result<Options,String>{
    let mut o=Options{seed:7,budget:ACTION_LIMIT_MAX,scenarios:SCENARIOS.to_vec(),nonattacking:false,prepare_only:false};let mut args=args.into_iter();
    while let Some(flag)=args.next(){if flag=="--prepare-only"{o.prepare_only=true;continue}let value=args.next().ok_or_else(||format!("missing value for {flag}"))?;
        match flag.as_str(){"--seed"=>o.seed=value.parse().map_err(|_|"invalid seed")?,"--budget"=>o.budget=value.parse().map_err(|_|"invalid budget")?,"--scenario"=>{o.nonattacking=value=="death";o.scenarios=match value.as_str(){"all"=>SCENARIOS.to_vec(),"guardian"|"death"=>vec![Scenario::Guardian],"hexaghost"=>vec![Scenario::Hexaghost],"slime-boss"=>vec![Scenario::SlimeBoss],_=>return Err("unsupported M22 scenario".into())};},_=>return Err(format!("unsupported flag {flag}"))}
    }
    if ![7,11].contains(&o.seed){return Err("M22 seed must be7 or11".into())}if o.budget>ACTION_LIMIT_MAX{return Err("M22 budget must be0..=32768".into())}Ok(o)
}
pub fn make_run(seed:u64,scenario:Scenario)->Result<Run,String>{
    let mut p=CreationProfile::experienced();p.neow_eligible=false;p.seen_bosses[0]=match scenario{Scenario::Guardian=>vec!["Hexaghost","SlimeBoss"],Scenario::Hexaghost=>vec!["Guardian","SlimeBoss"],Scenario::SlimeBoss=>vec!["Guardian","Hexaghost"]};
    let mut root=[0;32];root[..8].copy_from_slice(&seed.to_le_bytes());
    Run::new_room_scenario(RunConfig{character:Character::Watcher,ascension:0},p,RunRng::new(root,Partition::Test,0),RoomScenario::M22WatcherAct1).map_err(|e|format!("{e:?}"))
}
