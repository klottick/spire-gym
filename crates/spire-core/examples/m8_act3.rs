mod m8_support;
use spire_core::run::{RoomViewKind,RunTerminal};

fn show(label:&str,r:m8_support::Act3Report,death:bool)->bool{
    let d=&r.final_decision;let died=d.terminal==Some(RunTerminal::Death);let complete=d.terminal.is_none()&&d.room.as_ref().is_some_and(|r|r.kind==RoomViewKind::Act3PrefixComplete);
    println!("{label}: {} actions{} act{} floor{} HP{}/{} gold{} cards{} reward{} terminal{:?}",if died{if death{"real nonattacking Death"}else{"valid gameplay Death (NOT prefix completion)"}}else if complete{"scoped Act3 prefix complete (NOT full Act3/Heart)"}else{"unfinished"},r.trace.len(),d.act,d.floor,d.player.hp,d.player.max_hp,d.player.gold,d.objects.len(),d.reward_total,d.terminal);
    println!("  Act3 combats {:?}; persistent event screens {:?}",r.combats.iter().filter(|(a,_)|*a==3).map(|(_,c)|(c.encounter,c.outcome)).collect::<Vec<_>>(),r.events.iter().filter(|e|matches!(e,spire_core::content::EventId::Falling|spire_core::content::EventId::TombOfLordRedMask)).collect::<Vec<_>>());
    if death{died}else{died||complete}
}
fn execute()->Result<(),String>{
    let mut seed=11_u64;let mut budget=8192_usize;let mut scenario="all".to_owned();let mut args=std::env::args().skip(1);
    while let Some(arg)=args.next(){match arg.as_str(){"--seed"=>seed=args.next().ok_or("missing seed")?.parse().map_err(|_|"invalid seed")?,"--budget"=>budget=args.next().ok_or("missing budget")?.parse().map_err(|_|"invalid budget")?,"--scenario"=>scenario=args.next().ok_or("missing scenario")?,_=>return Err(format!("unknown argument {arg}"))}}
    if budget>8192{return Err("budget must be <=8192".into())}if !matches!(scenario.as_str(),"prefix"|"diagnostics"|"death"|"all"){return Err("scenario must be prefix|diagnostics|death|all".into())}
    let mut valid=true;
    if matches!(scenario.as_str(),"prefix"|"all"){for (label,seen) in [("AwakenedOne icon normal start",vec![]),("TimeEater icon normal start",vec!["AwakenedOne","DonuAndDeca"]),("DonuAndDeca icon normal start",vec!["AwakenedOne"])]{valid&=show(label,m8_support::prefix(seed,seen,budget)?,false);}}
    if matches!(scenario.as_str(),"diagnostics"|"all"){for r in m8_support::diagnostics(seed,budget)?{println!("supplied-resource native {:?}: {:?} actions{} turn{} HP{} observedHalfDead{} DarklingReturn{} (NOT normal-start evidence)",r.encounter,r.view.outcome,r.actions,r.view.turn,r.view.player.hp,r.saw_half_dead,r.saw_darkling_return);}}
    if matches!(scenario.as_str(),"death"|"all"){valid&=show("normal-start nonattacking",m8_support::death(seed,budget)?,true);}
    if valid{Ok(())}else{Err("unfinished requested episode; no completion or truncation fabricated".into())}
}
fn main(){if let Err(error)=execute(){eprintln!("{error}");std::process::exit(1)}}
