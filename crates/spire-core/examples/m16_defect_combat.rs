//! Fixed authored native M16 callers. Supplied fixtures are diagnostic resources.
use spire_core::{combat::{CombatAction,OrbKind,Outcome,rng::Partition},content::{CardId,EncounterId,RelicId},run::*};
const LABELS:[&str;5]=["starter-canonical-Cultist","supplied-orbs-Lagavulin","supplied-callbacks-Lagavulin","supplied-genetic-two-fights-Cultist","supplied-death-control-Cultist"];
#[derive(Debug,PartialEq,Eq)]
struct Options{seed:u64,budget:usize,scenario:String}
fn parse_args(args:impl IntoIterator<Item=String>)->Result<Options,String>{
    let mut args=args.into_iter();let mut seed=None;let mut budget=8192;let mut scenario="all".to_owned();
    while let Some(flag)=args.next(){let value=args.next().ok_or_else(||format!("missing value for {flag}"))?;
        match flag.as_str(){"--seed"=>seed=Some(value.parse().map_err(|_|"seed must be u64")?),"--budget"=>budget=value.parse().map_err(|_|"budget must be an integer")?,"--scenario"=>scenario=value,_=>return Err(format!("unknown argument {flag}"))}
    }
    if !(1..=8192).contains(&budget){return Err("budget must be in 1..=8192".into())}
    if !matches!(scenario.as_str(),"starter"|"supplied"|"all"){return Err("scenario must be starter|supplied|all".into())}
    Ok(Options{seed:seed.ok_or("missing --seed <u64>")?,budget,scenario})
}
fn rng(seed:u64)->RunRng{let mut bytes=[0;32];bytes[..8].copy_from_slice(&seed.to_le_bytes());RunRng::new(bytes,Partition::Test,0)}
fn supplied(seed:u64,hp:u32,ids:&[(CardId,u32)],relics:Vec<RelicId>)->Result<Run,String>{
    let deck=ids.iter().enumerate().map(|(i,(id,upgrades))|CardInstance{id:*id,instance:i as u64+1,upgrades:*upgrades,misc:if *id==CardId::GeneticAlgorithm{1}else{0}}).collect();
    Run::from_m16_defect_combat(M16DefectCombatStart::Supplied(AuthoredRunStart{hp,max_hp:75,gold:99,deck,relics,potions:vec![None;3],relic_counters:Default::default()}),rng(seed)).map_err(|e|format!("supplied constructor: {e:?}"))
}
#[derive(Default,Debug)]
struct Witness{cap10:bool,four:bool,barrage:bool,focus:i32,storm:bool,heat:bool,echo:bool,lightning:bool,misc:i32}
fn inspect(d:&PublicDecision,w:&mut Witness){
    w.misc=d.objects.iter().find(|c|c.id==CardId::GeneticAlgorithm).map_or(w.misc,|c|c.misc);
    if let Some(c)=&d.combat{
        w.cap10|=c.character.orbs.len()==10;
        w.four|=[OrbKind::Lightning,OrbKind::Frost,OrbKind::Dark,OrbKind::Plasma].iter().all(|k|c.character.orbs.contains(&Some(*k)));
        w.focus=w.focus.max(c.character.focus);w.lightning|=c.character.orbs.contains(&Some(OrbKind::Lightning));
        for p in &c.player.powers{match p.id.0{"storm"=>w.storm=true,"heatsinks"=>w.heat=true,"echo_form"=>w.echo=true,_=>()}}
    }
}
fn pick(d:&PublicDecision,kind:usize,w:&Witness)->Result<CombatAction,String>{
    use CardId::*;
    let c=d.combat.as_ref().ok_or("expected active combat")?;
    let priority:&[CardId]=match kind{0=>&[Zap,Dualcast,StrikeBlue,DefendBlue],1=>&[Capacitor,Darkness,Fusion,Coolheaded,Zap,Barrage,Claw],2=>&[Storm,Heatsinks,EchoForm,Amplify,Defragment],3=>&[GeneticAlgorithm,Claw],4=>&[],_=>unreachable!()};
    for id in priority{
        if kind==1 {
            let orb=match id{Darkness=>Some(OrbKind::Dark),Fusion=>Some(OrbKind::Plasma),Coolheaded=>Some(OrbKind::Frost),Zap=>Some(OrbKind::Lightning),_=>None};
            if orb.is_some_and(|orb|c.character.orbs.contains(&Some(orb))){continue}
            if *id==Barrage&&!(c.character.orbs.len()==10&&w.four&&c.character.orbs.iter().flatten().count()>3){continue}
            if *id==Claw&&!w.barrage{continue}
        }
        for candidate in &d.candidates{
            if let GameAction::Combat(action @ CombatAction::PlayCard{instance,..})=&candidate.action{
                if c.hand.iter().any(|card|card.instance.0==*instance&&card.id==*id){return Ok(action.clone())}
            }
        }
    }
    d.candidates.iter().find_map(|candidate|match &candidate.action{GameAction::Combat(action @ CombatAction::EndTurn)=>Some(action.clone()),_=>None}).ok_or_else(||"no permitted legal semantic action".into())
}
fn fight(run:&mut Run,encounter:EncounterId,kind:usize,budget:usize)->Result<(Outcome,Witness),String>{
    run.enter_combat(encounter).map_err(|e|format!("{} enter: {e:?}",LABELS[kind]))?;
    if kind==3 {
        let d=run.decision().map_err(|e|format!("GA entry: {e:?}"))?;
        require(d.combat.as_ref().unwrap().hand.iter().filter(|c|c.id==CardId::Claw).all(|c|c.values.as_ref().unwrap().base_damage==Some(5)),"Claw combat reset to5")?;
    }
    let mut replay=run.clone();let mut witness=Witness::default();
    for n in 0..=budget{
        let d=run.decision().map_err(|e|format!("{} decision: {e:?}",LABELS[kind]))?;
        if d!=replay.decision().map_err(|e|format!("replay decision: {e:?}"))?{return Err("public replay diverged".into())}
        inspect(&d,&mut witness);
        if d.combat.is_none(){
            let record=d.last_combat.ok_or("combat ended without shared settlement record")?;
            println!("RESULT label={} actions={n} outcome={:?} hp={} terminal={:?} witness={witness:?}",LABELS[kind],record.outcome,d.player.hp,d.terminal);
            return Ok((record.outcome,witness))
        }
        if n==budget{return Err(format!("BUDGET label={} accepted_actions={n}",LABELS[kind]))}
        let action=pick(&d,kind,&witness)?;
        if let CombatAction::PlayCard{instance,..}=&action{
            if d.combat.as_ref().unwrap().hand.iter().any(|c|c.instance.0==*instance&&c.id==CardId::Barrage){witness.barrage=true;}
        }
        let c=d.combat.as_ref().unwrap();
        println!("ACTION label={} n={n} turn={} hp={} energy={} slots={} orbs={:?} dark={:?} focus={} misc={} action={action:?}",LABELS[kind],c.turn,c.player.hp,c.player.energy,c.character.orbs.len(),c.character.orbs,c.character.orb_dark_damage,c.character.focus,witness.misc);
        let result=run.choose_combat(action.clone());let repeated=replay.choose_combat(action);
        if result!=repeated||*run!=replay{return Err(format!("{} native replay/resource divergence at{n}",LABELS[kind]))}
        result.map_err(|e|format!("FAULT_OR_UNSUPPORTED label={} action={n} error={e:?}",LABELS[kind]))?;
    }unreachable!()
}
fn require(ok:bool,message:&str)->Result<(),String>{if ok{Ok(())}else{Err(format!("missing required witness: {message}"))}}
fn execute(options:Options)->Result<(),String>{
    use CardId::*;
    println!("M16 native authored combat; accepted_base={} metadata_prefix={} source_manifest={} supplied resources are diagnostic; no normal progression/Python claim",M16_DEFECT_COMBAT_SETUP_BASE,M16_DEFECT_COMBAT_SETUP_PREFIX_LEN,std::env::var("M16_SOURCE_MANIFEST_SHA256").unwrap_or_else(|_|"see retained candidate receipt".into()));
    if options.scenario!="supplied"{
        let mut run=Run::from_m16_defect_combat(M16DefectCombatStart::Canonical,rng(options.seed)).map_err(|e|format!("canonical constructor: {e:?}"))?;
        let d=run.decision().map_err(|e|format!("canonical resources: {e:?}"))?;
        require(d.player.hp==75&&d.player.gold==99&&d.objects.len()==10&&d.player.relics==[RelicId::CrackedCore]&&d.player.potions==[None;3]&&d.player.keys==[false;3],"canonical resources")?;
        fight(&mut run,EncounterId::Cultist,0,options.budget)?;
    }
    if options.scenario!="starter"{
        let mut orbs=supplied(options.seed,75,&[(Capacitor,1),(Capacitor,1),(Capacitor,1),(Darkness,0),(Fusion,1),(Coolheaded,1),(Zap,1),(Barrage,1),(Claw,1)],vec![RelicId::DataDisk,RelicId::GoldPlatedCables])?;
        let (_,w)=fight(&mut orbs,EncounterId::Lagavulin,1,options.budget)?;require(w.cap10&&w.four&&w.barrage,"cap10/allfour/occupied>3 actual Barrage")?;
        let mut callbacks=supplied(options.seed,75,&[(Storm,1),(Heatsinks,1),(Amplify,1),(Defragment,1),(EchoForm,1)],vec![RelicId::EmotionChip])?;
        let (_,w)=fight(&mut callbacks,EncounterId::Lagavulin,2,options.budget)?;require(w.storm&&w.heat&&w.echo&&w.focus>=4&&w.lightning,"actual Power listeners/repeats/Lightning")?;
        let mut genetic=supplied(options.seed,75,&[(GeneticAlgorithm,1),(Claw,1),(Claw,1),(Claw,1),(Claw,1)],vec![])?;
        let (outcome,w)=fight(&mut genetic,EncounterId::Cultist,3,options.budget)?;require(outcome==Outcome::Victory&&w.misc==4,"first fight GA misc4/Victory")?;
        // A new shared entry reconstructs local Claw fields; learned misc is permanent.
        let (_,w)=fight(&mut genetic,EncounterId::Cultist,3,options.budget)?;require(w.misc==7,"second fight GA misc7")?;
        let mut death=supplied(options.seed,1,&[(DefendBlue,0)],vec![])?;let (outcome,_)=fight(&mut death,EncounterId::Cultist,4,options.budget)?;
        require(outcome==Outcome::Death&&death.decision().map_err(|e|format!("Death decision: {e:?}"))?.terminal==Some(RunTerminal::Death),"genuine Death")?;
    }Ok(())
}
fn main(){let result=parse_args(std::env::args().skip(1)).and_then(execute);if let Err(error)=result{eprintln!("{error}");std::process::exit(1);}}
#[cfg(test)]
mod tests{
    use super::*;
    #[test]fn arguments_and_fixed_labels(){
        assert_eq!(parse_args(["--seed","7"].map(str::to_owned)).unwrap(),Options{seed:7,budget:8192,scenario:"all".into()});
        assert!(parse_args(["--seed","7","--scenario","normal-run"].map(str::to_owned)).is_err());
        assert!(parse_args(["--seed","7","--budget","0"].map(str::to_owned)).is_err());
        assert!(parse_args(["--seed","7","--budget","8193"].map(str::to_owned)).is_err());
        assert!(parse_args(Vec::<String>::new()).is_err());
    }
}
