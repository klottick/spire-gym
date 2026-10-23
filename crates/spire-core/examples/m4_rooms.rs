mod m4_support;
use m4_support::{Caller,Scenario};
fn main(){if let Err(e)=execute(){eprintln!("{e}");std::process::exit(1)}}
fn execute()->Result<(),String>{
    let mut scenario="all".to_owned();let mut seed=7;let mut args=std::env::args().skip(1);
    while let Some(arg)=args.next(){match arg.as_str(){"--scenario"=>scenario=args.next().ok_or("missing scenario")?,"--seed"=>seed=args.next().ok_or("missing seed")?.parse().map_err(|_|"invalid seed")?,_=>return Err(format!("unknown argument {arg}"))}}
    let scenarios:Vec<_>=match scenario.as_str(){"all"=>vec![Scenario::Route,Scenario::Purge,Scenario::Keys,Scenario::Death],"route"=>vec![Scenario::Route],"purge"=>vec![Scenario::Purge],"keys"=>vec![Scenario::Keys],"death"=>vec![Scenario::Death],_=>return Err("expected all|route|purge|keys|death".into())};
    for scenario in scenarios {
        let mut run=m4_support::run(seed);let mut caller=Caller::new(512);m4_support::rollout(&mut run,&mut caller,scenario)?;
        for (action,t,d) in &caller.trace {
            if !matches!(action,spire_core::run::GameAction::Combat(_))||!t.committed_events.is_empty(){println!("{scenario:?} floor={} action={action:?} room={:?} hp={}/{} gold={} deck={} counters={:?} delta={} total={} events={:?}",d.floor,d.room.as_ref().map(|r|r.kind),d.player.hp,d.player.max_hp,d.player.gold,d.objects.len(),d.player.relic_counters,t.reward,d.reward_total,t.committed_events);}
        }
        let d=run.decision().map_err(|e|format!("{e:?}"))?;
        println!("{scenario:?} public map: {:?}",run.public_map());
        for (_,_,decision) in caller.trace.iter().filter(|(_,_,d)|d.room.as_ref().is_some_and(|r|matches!(r.kind,spire_core::run::RoomViewKind::Shop|spire_core::run::RoomViewKind::CardReward|spire_core::run::RoomViewKind::Treasure))){
            println!("{scenario:?} floor={} public offers: {:?}",decision.floor,decision.room.as_ref().unwrap().choices);
        }
        println!("{scenario:?} complete: actions={} floor={} hp={}/{} gold={} cards={} fights={} reward={} terminal={:?} checkpoint={:?}",caller.accepted(),d.floor,d.player.hp,d.player.max_hp,d.player.gold,d.objects.len(),d.combat_progress.settled,d.reward_total,d.terminal,d.room.map(|r|r.kind));
    }Ok(())
}
