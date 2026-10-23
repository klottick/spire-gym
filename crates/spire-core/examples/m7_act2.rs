mod m7_support;
use spire_core::run::{RoomViewKind,RunTerminal};

fn show(label:&str,r:m7_support::Act2Report,death:bool)->bool{
    let mut last=None;let mut record=None;
    for (index,s) in r.trace.iter().enumerate() {
        let d=&s.decision;
        if last!=Some((d.act,d.floor)){println!("{label}: act{} floor{} HP{}/{} gold{} cards{} reward{}",d.act,d.floor,d.player.hp,d.player.max_hp,d.player.gold,d.objects.len(),d.reward_total);last=Some((d.act,d.floor));}
        if let Some(c)=&d.last_combat{if record!=Some(c.visit){println!("  settled {:?} {:?}",c.encounter,c.outcome);record=Some(c.visit);}}
        if s.reward!=0{println!("  accepted reward delta {}",s.reward);}
        if let spire_core::run::GameAction::Combat(spire_core::combat::CombatAction::UsePotion{slot,..})=&s.action {
            if let Some(before)=index.checked_sub(1).and_then(|i|r.trace.get(i)).and_then(|s|s.decision.combat.as_ref()) {
                println!("  actual potion use {:?} act{} floor{} enemies{:?}",before.inventory.as_ref().and_then(|v|v.potions.get(*slot)).copied().flatten(),d.act,d.floor,before.enemies.iter().filter(|e|e.hp>0).map(|e|e.enemy_id).collect::<Vec<_>>());
            }
        }
        if let Some(room)=&d.room{if d.combat.is_none()&&!room.choices.is_empty(){println!("  {:?} offered {:?}",room.kind,room.choices.iter().filter(|c|c.status==spire_core::run::RoomChoiceStatus::Offered).map(|c|&c.kind).collect::<Vec<_>>());}}
    }
    let d=&r.final_decision;
    let died=d.terminal==Some(RunTerminal::Death);
    let ready=d.terminal.is_none()&&d.room.as_ref().is_some_and(|r|r.kind==RoomViewKind::Act3Ready);
    let valid=if death{died}else{died||ready};
    println!("{label}: {} actions{} act{} floor{} HP{} reward{} terminal{:?}",if died{if death{"real Death diagnostic"}else{"valid gameplay Death (not Act3 completion)"}}else if ready{"scoped nonterminal Act3Ready (not Heart victory)"}else{"unfinished"},r.trace.len(),d.act,d.floor,d.player.hp,d.reward_total,d.terminal);
    if !valid {
        eprintln!("final deck {:?}",d.objects.iter().map(|c|(c.id,c.upgrades)).collect::<Vec<_>>());
        let turns=r.trace.iter().filter(|s|s.decision.act==d.act&&s.decision.floor==d.floor).collect::<Vec<_>>();
        for s in turns.iter().skip(turns.len().saturating_sub(16)){if let Some(c)=&s.decision.combat{eprintln!("turn{} HP{} block{} energy{} hand{:?} enemies{:?}",c.turn,c.player.hp,c.player.block,c.player.energy,c.hand.iter().map(|c|(c.id,c.effective_cost)).collect::<Vec<_>>(),c.enemies.iter().map(|e|(e.enemy_id,e.hp,e.block,e.intent.as_str(),e.intent_damage)).collect::<Vec<_>>());}}
    }valid
}

fn main(){if let Err(e)=execute(){eprintln!("{e}");std::process::exit(1)}}
fn execute()->Result<(),String>{
    let mut seed=7_u64;let mut budget=4096_usize;let mut scenario="all".to_owned();let mut args=std::env::args().skip(1);
    while let Some(arg)=args.next(){match arg.as_str(){"--seed"=>seed=args.next().ok_or("missing seed")?.parse().map_err(|_|"invalid seed")?,"--budget"=>budget=args.next().ok_or("missing budget")?.parse().map_err(|_|"invalid budget")?,"--scenario"=>scenario=args.next().ok_or("missing scenario")?,_=>return Err(format!("unknown argument {arg}"))}}
    if budget>4096{return Err("budget must be <=4096".into())}if !matches!(scenario.as_str(),"complete"|"death"|"all"){return Err("scenario must be complete|death|all".into())}
    let mut complete=true;
    if matches!(scenario.as_str(),"complete"|"all"){for (label,seen) in [("Champ normal start",vec!["Automaton","Collector"]),("Automaton normal start",vec!["Champ","Collector"]),("Collector normal start",vec!["Champ","Automaton"]) ]{complete&=show(label,m7_support::complete(seed,seen,budget)?,false);}}
    if matches!(scenario.as_str(),"death"|"all"){complete&=show("nonattacking normal start",m7_support::death(seed,budget)?,true);}
    if complete{Ok(())}else{Err("unfinished requested episode; no completion or truncation fabricated".into())}
}

#[cfg(test)]
mod tests {
    #[test]
    fn natural_champ_death_is_a_valid_executed_episode_not_act3_completion() {
        let r=super::m7_support::complete(7,vec!["Automaton","Collector"],4096).unwrap();
        assert_eq!(r.final_decision.terminal,Some(spire_core::run::RunTerminal::Death));
        assert_ne!(r.final_decision.room.as_ref().unwrap().kind,spire_core::run::RoomViewKind::Act3Ready);
        assert!(super::show("natural Champ loss",r,false));
    }
}
