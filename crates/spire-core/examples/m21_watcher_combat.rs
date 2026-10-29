//! Fixed public-data M21 caller. No strategy search or hidden-state input.
mod m21_support;
use m21_support::*;
use spire_core::{combat::{CombatAction,Outcome,Stance,rng::Partition},run::*};
use serde_json::{Value,json};
use sha2::{Digest,Sha256};
use std::io::Write;
#[derive(Debug,PartialEq,Eq)]
struct Options{seed:u64,budget:usize,scenario:Option<Scenario>,prepare_only:bool}
fn options(args:impl IntoIterator<Item=String>)->Result<Options,String>{
    let mut result=Options{seed:7,budget:8192,scenario:None,prepare_only:false};let mut args=args.into_iter();
    while let Some(flag)=args.next(){if flag=="--prepare-only"{result.prepare_only=true;continue}
        let value=args.next().ok_or_else(||format!("missing value for {flag}"))?;
        match flag.as_str(){"--seed"=>result.seed=value.parse().map_err(|_|"invalid seed")?,"--budget"=>{result.budget=value.parse().map_err(|_|"invalid budget")?;if result.budget>8192{return Err("budget exceeds frozen8192".into())}},"--scenario"=>result.scenario=if value=="all"{None}else{Some(Scenario::parse(&value)?)},_=>return Err(format!("unsupported option {flag}"))}
    }Ok(result)
}
fn emit(v:Value){println!("{v}");std::io::stdout().flush().expect("retain output");}
fn sha(s:&str)->String{Sha256::digest(s.as_bytes()).iter().map(|b|format!("{b:02x}")).collect()}
fn outcome(value:Option<Outcome>)->Option<&'static str>{value.map(|v|match v{Outcome::Victory=>"victory",Outcome::Death=>"death",Outcome::Escaped=>"escaped"})}
#[derive(Default)]struct Witness{stances:std::collections::BTreeSet<String>,history:i32,powers:std::collections::BTreeSet<String>,played:std::collections::BTreeSet<String>,children:std::collections::BTreeSet<String>,potions:std::collections::BTreeSet<usize>,scry:bool,meditate:bool,retained_growth:bool,sands_discount:bool,calm_choice:bool}
impl Witness{
 fn inspect(&mut self,d:&PublicDecision){if let Some(c)=&d.combat{
    self.stances.insert(format!("{:?}",c.character.stance));self.history=self.history.max(c.mantra_gained.unwrap_or(0));self.powers.extend(c.player.powers.iter().map(|p|p.id.0.to_owned()));
    if let Some(s)=&c.selection{self.scry|=s.operation==spire_core::combat::SelectionOperation::Scry;self.meditate|=s.operation==spire_core::combat::SelectionOperation::MeditateReturn;}
    for card in c.hand.iter().chain(&c.discard).chain(&c.exhaust).chain(&c.playing_cards).chain(&c.played_powers){
      if matches!(card.origin,spire_core::combat::CombatCardRef::CombatLocal(_)){self.children.insert(card.id.as_str().to_owned());}
      if let Some(v)=&card.values{self.retained_growth|=card.id==spire_core::content::CardId::WindmillStrike&&v.base_damage.is_some_and(|n|n>7)||card.id==spire_core::content::CardId::Perseverance&&v.base_block.is_some_and(|n|n>5);}
      self.sands_discount|=card.id==spire_core::content::CardId::SandsOfTime&&card.watcher.as_ref().and_then(|v|v.combat_cost).is_some_and(|n|n<4);
    }
 }}
 fn action(&mut self,d:&PublicDecision,a:&CombatAction){match a{
    CombatAction::PlayCard{instance,..}=>if let Some(card)=d.combat.as_ref().and_then(|c|c.hand.iter().find(|c|c.instance.0==*instance)){self.played.insert(card.id.as_str().to_owned());},
    CombatAction::UsePotion{slot,..}=>{self.potions.insert(*slot);},CombatAction::ChooseStance{stance:Stance::Calm}=>self.calm_choice=true,_=>(),
 }}
 fn value(&self,scenario:Scenario,end:Option<Outcome>)->Value{
    let requirements:Vec<(&str,bool)>=match scenario{
      Scenario::Starter=>vec![("canonical_start",true)],Scenario::Stance=>vec![("Calm",self.stances.contains("Calm")),("Wrath",self.stances.contains("Wrath")),("mental_fortress",self.powers.contains("mental_fortress")),("rushdown",self.powers.contains("rushdown")),("like_water",self.powers.contains("like_water"))],
      Scenario::Mantra=>vec![("history_positive",self.history>0),("Divinity",self.stances.contains("Divinity")),("brilliance_played",self.played.contains("brilliance"))],Scenario::Scry=>vec![("scry_choice",self.scry),("weave_played",self.played.contains("weave"))],
      Scenario::Retain=>vec![("growth",self.retained_growth),("sands_discount",self.sands_discount),("meditate_choice",self.meditate)],Scenario::Creation=>vec![("smite",self.children.contains("smite")),("safety",self.children.contains("safety")),("miracle",self.children.contains("miracle")),("master_reality",self.powers.contains("master_reality"))],
      Scenario::Potions=>vec![("all_three_potions",self.potions.len()==3),("Calm_choice",self.calm_choice),("Divinity",self.stances.contains("Divinity"))],Scenario::Death=>vec![("genuine_death",end==Some(Outcome::Death))],
    };
    json!({"stances":self.stances,"max_public_mantra_history":self.history,"powers_seen":self.powers,"kinds_played":self.played,"children_seen":self.children,"potion_slots_used":self.potions,"scry_choice":self.scry,"meditate_choice":self.meditate,"retained_growth":self.retained_growth,"sands_discount":self.sands_discount,"requirements":requirements.iter().map(|(name,present)|json!({"name":name,"present":present})).collect::<Vec<_>>(),"missing":requirements.iter().filter(|(_,present)|!present).map(|(name,_)|*name).collect::<Vec<_>>()})
 }
}
fn resources(d:&PublicDecision)->Value{json!({"hp":d.player.hp,"max_hp":d.player.max_hp,"gold":d.player.gold,"relics":d.player.relics.iter().map(|r|r.as_str()).collect::<Vec<_>>(),"potions":d.player.potions.iter().map(|p|p.map(|p|p.as_str())).collect::<Vec<_>>(),"deck":d.objects.iter().map(|c|json!({"instance":c.instance,"card":c.id.as_str(),"upgrades":c.upgrades,"misc":c.misc})).collect::<Vec<_>>()})}
fn execute(o:&Options,s:Scenario)->Result<(),String>{
    let mut root=[0;32];root[..8].copy_from_slice(&o.seed.to_le_bytes());let rng=RunRng::new(root,Partition::Test,0);
    let mut run=Run::prepare_m21_watcher_combat(s.start(),s.encounter(),rng.clone()).map_err(|e|format!("prepare: {e:?}"))?;
    let before=run.decision().map_err(|e|format!("owned resources: {e:?}"))?;
    emit(json!({"kind":"configuration","scenario":s.name(),"seed":o.seed,"budget":o.budget,"encounter":s.encounter().as_str(),"input_rng":format!("{rng:?}"),"resources":resources(&before),"source_manifest":std::env::var("M21_SOURCE_MANIFEST_SHA256").ok(),"grade":if s==Scenario::Starter{"canonical authored combat"}else{"supplied diagnostic resources"}}));
    if let Err(error)=run.enter_training_combat(s.encounter()){emit(json!({"kind":"result","scenario":s.name(),"seed":o.seed,"status":"startup_fault","error":format!("{error:?}"),"steps":[]}));return Err("startup failed; owner retained in raw error".into())}
    let initial=run.decision().map_err(|e|format!("initial decision: {e:?}"))?;
    emit(json!({"kind":"initial","scenario":s.name(),"seed":o.seed,"public_debug":format!("{initial:?}"),"public_sha256":sha(&format!("{initial:?}"))}));
    if o.prepare_only{return Ok(())}
    let mut steps=Vec::new();let mut witness=Witness::default();
    for n in 0..=o.budget{
        let decision=run.decision().map_err(|e|format!("public boundary: {e:?}"))?;witness.inspect(&decision);
        let settled=decision.last_combat.as_ref().map(|r|r.outcome);
        if decision.combat.is_none()||n==o.budget{
            emit(json!({"kind":"result","scenario":s.name(),"seed":o.seed,"status":if settled.is_some(){"settled"}else{"budget"},"accepted_steps":steps.len(),"outcome":outcome(settled),"steps":steps,"resources":resources(&decision),"settlement":format!("{:?}",decision.last_combat),"public_sha256":sha(&format!("{decision:?}")),"witness":witness.value(s,settled)}));
            return if settled.is_some(){Ok(())}else{Err("budget unfinished; no fabricated outcome".into())}
        }
        let action=pick(&decision,s)?;witness.action(&decision,&action);let value=action_value(&action);
        emit(json!({"kind":"action","scenario":s.name(),"seed":o.seed,"ordinal":n,"public_debug":format!("{decision:?}"),"public_sha256":sha(&format!("{decision:?}")),"action":value}));
        match run.choose_combat(action){Ok(t)=>{
            let next=run.decision().map_err(|e|format!("post-step public boundary: {e:?}"))?;
            let end=next.last_combat.as_ref().map(|r|r.outcome);
            steps.push(json!({"action":value,"reward":end.map_or(0,|o|if o==Outcome::Victory{1}else{-1}),"terminal":if t.terminal==Some(RunTerminal::Death){Some("death")}else{None},"core_reward":t.reward}));
        },Err(error)=>{
            emit(json!({"kind":"result","scenario":s.name(),"seed":o.seed,"status":"fault","accepted_steps":steps.len(),"steps":steps,"fault":{"action":value,"error":format!("{error:?}")},"outcome":null,"witness":witness.value(s,None)}));return Err(format!("accepted action error: {error:?}"))
        }}
    }unreachable!()
}
fn main(){let options=match options(std::env::args().skip(1)){Ok(v)=>v,Err(e)=>{eprintln!("INPUT: {e}");std::process::exit(2)}};let cases=options.scenario.map_or_else(||SCENARIOS.to_vec(),|s|vec![s]);let mut failed=false;for s in cases{if let Err(e)=execute(&options,s){eprintln!("{}: {e}",s.name());failed=true;}}if failed{std::process::exit(1)}}
#[cfg(test)]mod tests{use super::*;#[test]fn m21_caller_options_accept_zero_one_and_freeze_maximum(){assert_eq!(options(Vec::new()).unwrap(),Options{seed:7,budget:8192,scenario:None,prepare_only:false});for budget in ["0","1"]{assert_eq!(options(["--budget",budget,"--scenario","starter"].map(str::to_owned)).unwrap().budget,budget.parse::<usize>().unwrap());}assert!(options(["--budget","8193"].map(str::to_owned)).is_err());assert!(options(["--scenario","unknown"].map(str::to_owned)).is_err());}}
