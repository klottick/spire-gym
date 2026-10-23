mod m6_support;
use spire_core::{combat::Outcome,run::{RoomViewKind,RunTerminal}};
fn show(label:&str,r:m6_support::PrefixReport,death:bool)->bool{
    let mut last=None;let mut last_record=None;for s in &r.trace{
        let d=&s.decision;if last!=Some((d.act,d.floor)){println!("{label}: act{} floor{} hp{}/{} gold{} cards{} reward{}",d.act,d.floor,d.player.hp,d.player.max_hp,d.player.gold,d.objects.len(),d.reward_total);last=Some((d.act,d.floor));}
        if let Some(c)=&d.last_combat{if last_record!=Some(c.visit){println!("  settled {:?} {:?}",c.encounter,c.outcome);last_record=Some(c.visit);}}
        if s.reward!=0{println!("  accepted reward delta {}",s.reward);}
        if let Some(room)=&d.room{if d.combat.is_none()&&!room.choices.is_empty(){println!("  {:?} offered {:?}",room.kind,room.choices.iter().filter(|c|c.status==spire_core::run::RoomChoiceStatus::Offered).map(|c|&c.kind).collect::<Vec<_>>());}}
    }
    let d=r.final_decision;let complete=if death{d.terminal==Some(RunTerminal::Death)}else{d.room.as_ref().is_some_and(|room|room.kind==RoomViewKind::Act2PrefixComplete)&&d.terminal.is_none()};
    println!("{label}: {} actions{} act{} floor{} hp{} reward{} terminal{:?}",if complete{"complete scoped result"}else{"unfinished or lost"},r.trace.len(),d.act,d.floor,d.player.hp,d.reward_total,d.terminal);
    if !complete{eprintln!("  final permanent deck {:?}",d.objects.iter().map(|c|(c.id,c.upgrades)).collect::<Vec<_>>());let mut turn=None;for s in r.trace.iter().filter(|s|s.decision.act==d.act&&s.decision.floor==d.floor){if let Some(c)=&s.decision.combat{if turn!=Some(c.turn){eprintln!("  diagnostic turn{} hp{} hand{:?} enemies{:?}",c.turn,c.player.hp,c.hand.iter().map(|c|c.id).collect::<Vec<_>>(),c.enemies.iter().map(|e|(e.enemy_id,e.hp,e.block,e.intent.as_str(),e.intent_damage)).collect::<Vec<_>>());turn=Some(c.turn);}}}}complete
}
fn main(){if let Err(e)=execute(){eprintln!("{e}");std::process::exit(1)}}
fn execute()->Result<(),String>{
    let mut seed=7_u64;let mut budget=4096_usize;let mut scenario="all".to_owned();let mut args=std::env::args().skip(1);while let Some(arg)=args.next(){match arg.as_str(){"--seed"=>seed=args.next().ok_or("missing seed")?.parse().map_err(|_|"invalid seed")?,"--budget"=>budget=args.next().ok_or("missing budget")?.parse().map_err(|_|"invalid budget")?,"--scenario"=>scenario=args.next().ok_or("missing scenario")?,_=>return Err(format!("unknown argument {arg}"))}}
    if budget>4096{return Err("budget must be <=4096".into())}if !matches!(scenario.as_str(),"prefix"|"encounters"|"death"|"all"){return Err("scenario must be prefix|encounters|death|all".into())}let mut complete=true;
    if matches!(scenario.as_str(),"prefix"|"all"){for (label,seen) in [("normal prefix: Champ icon",vec!["Automaton","Collector"]),("normal prefix: Automaton icon",vec!["Champ","Collector"]),("normal prefix: Collector icon",vec!["Champ","Automaton"]) ]{complete&=show(label,m6_support::prefix(seed,seen,budget)?,false);}}
    if matches!(scenario.as_str(),"encounters"|"all"){for d in m6_support::encounters(seed,budget)?{println!("supplied-resource diagnostic {:?}: {:?} hp{} accepted_actions{} (not map progression)",d.encounter,d.exit.outcome,d.exit.hp,d.actions.len());complete&=d.exit.outcome==Outcome::Victory;}}
    if matches!(scenario.as_str(),"death"|"all"){complete&=show("normal-start deliberate nonattacking Death",m6_support::death(seed,budget)?,true);}
    if complete{Ok(())}else{Err("unfinished/unsuccessful requested scenario; no full-act success implied".into())}
}
