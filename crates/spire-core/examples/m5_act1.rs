mod m5_support;
use m5_support::{Caller,Scenario};
fn main(){if let Err(e)=execute(){eprintln!("{e}");std::process::exit(1)}}
fn execute()->Result<(),String>{
    let mut scenario="all".to_owned();let mut seed=7;let mut budget=4096;let mut args=std::env::args().skip(1);
    while let Some(arg)=args.next(){match arg.as_str(){"--scenario"=>scenario=args.next().ok_or("missing scenario")?,"--seed"=>seed=args.next().ok_or("missing seed")?.parse().map_err(|_|"invalid seed")?,"--budget"=>budget=args.next().ok_or("missing budget")?.parse().map_err(|_|"invalid budget")?,_=>return Err(format!("unknown argument {arg}"))}}
    let scenarios=match scenario.as_str(){"all"=>vec![Scenario::Guardian,Scenario::Hexaghost,Scenario::Slime,Scenario::Death],"guardian"=>vec![Scenario::Guardian],"hexaghost"=>vec![Scenario::Hexaghost],"slime"=>vec![Scenario::Slime],"death"=>vec![Scenario::Death],_=>return Err("expected all|guardian|hexaghost|slime|death".into())};
    for scenario in scenarios {
        let mut r=m5_support::run(seed,scenario);let mut caller=Caller::new(budget);let result=m5_support::rollout(&mut r,&mut caller,scenario);
        for (action,t,d) in &caller.trace{if !matches!(action,spire_core::run::GameAction::Combat(_))||!t.committed_events.is_empty(){println!("{scenario:?} floor={} action={action:?} room={:?} hp={}/{} gold={} deck={} counters={:?} delta={} total={} events={:?}",d.floor,d.room.as_ref().map(|r|r.kind),d.player.hp,d.player.max_hp,d.player.gold,d.objects.len(),d.player.relic_counters,t.reward,d.reward_total,t.committed_events);}}
        for (_,_,d) in &caller.trace{if let Some(room)=d.room.as_ref().filter(|r|matches!(r.kind,spire_core::run::RoomViewKind::CardReward|spire_core::run::RoomViewKind::Shop|spire_core::run::RoomViewKind::Treasure|spire_core::run::RoomViewKind::BossRelicChoice)){println!("{scenario:?} floor={} public offers: {:?}",d.floor,room.choices);}}
        let d=r.decision().map_err(|e|format!("{e:?}"))?;println!("{scenario:?} result: actions={} elites={} act={} floor={} hp={}/{} gold={} deck={} reward={} terminal={:?} checkpoint={:?}",caller.accepted(),caller.elites,d.act,d.floor,d.player.hp,d.player.max_hp,d.player.gold,d.objects.len(),d.reward_total,d.terminal,d.room.map(|r|r.kind));
        result?;
    }Ok(())
}
