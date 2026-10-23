//! Authored persistent combat checkpoints, not normal Neow/map/reward runs.
mod m3_support;
use m3_support::*;
use spire_core::{combat::{CombatAction,CombatCardRef,TargetId},content::{CardId,EncounterId,RelicId},run::*};

fn show(run:&Run)->Result<(),DemoError> {
    let view=run.decision()?;
    println!("hp={}/{} gold={} inventory={:?}/{:?} progress={:?} last={:?} terminal={:?}",
        view.player.hp,view.player.max_hp,view.player.gold,view.player.relics,view.player.potions,
        view.combat_progress,view.last_combat,view.terminal);
    println!("  permanent_deck={:?}",view.objects);
    if let Some(combat)=view.combat {
        println!("  turn={} energy={}/{} Block={} powers={:?} enemies={:?}",combat.turn,
            combat.player.energy,combat.player.max_energy,combat.player.block,combat.player.powers,combat.enemies);
        println!("  hand={:?} draw_count={} discard={} exhaust={} selection={:?}",
            combat.hand,combat.draw_count,combat.discard.len(),combat.exhaust.len(),combat.selection);
    }
    println!("  menu={:?}",view.candidates.iter().map(|c|&c.action).collect::<Vec<_>>());
    Ok(())
}

fn enter(run:&mut Run,encounter:EncounterId)->Result<(),DemoError> {
    println!("enter authored encounter={encounter:?}");
    let transition=run.enter_combat(encounter)?;
    println!("  committed_events={:?}",transition.committed_events);
    show(run)
}
fn take(caller:&mut Caller,run:&mut Run,action:CombatAction)->Result<(),DemoError> {
    println!("action={action:?}");
    let transition=caller.step(run,action)?;
    println!("  committed_events={:?}",transition.committed_events);
    show(run)
}
fn play(caller:&mut Caller,run:&mut Run,instance:u64)->Result<(),DemoError> {
    take(caller,run,CombatAction::PlayCard {instance,target:if instance==4 {None} else {Some(TargetId(1))}})
}
fn potion(caller:&mut Caller,run:&mut Run,slot:usize,target:Option<TargetId>)->Result<(),DemoError> {
    take(caller,run,CombatAction::UsePotion {slot,target})
}

fn consecutive(seed:u64)->Result<(),DemoError> {
    let mut run=Run::from_authored_combat(config(),start(),rng(seed))?;
    let mut caller=Caller::new(128);
    show(&run)?;
    enter(&mut run,EncounterId::JawWorm)?;
    play(&mut caller,&mut run,5)?; play(&mut caller,&mut run,4)?;
    potion(&mut caller,&mut run,0,None)?;
    for instance in [3,2,1] {play(&mut caller,&mut run,instance)?;}
    potion(&mut caller,&mut run,1,Some(TargetId(1)))?;
    enter(&mut run,EncounterId::Cultist)?;
    potion(&mut caller,&mut run,2,None)?;
    for instance in [5,4,3,2,1] {play(&mut caller,&mut run,instance)?;}
    take(&mut caller,&mut run,CombatAction::EndTurn)?;
    let copy=run.decision()?.combat.ok_or(RunError::InvalidAction)?.hand.iter()
        .find(|c|c.id==CardId::Anger && matches!(c.origin,CombatCardRef::CombatLocal(_)))
        .map(|c|c.instance.0).ok_or(RunError::InvalidAction)?;
    for instance in [3,copy,2] {play(&mut caller,&mut run,instance)?;}
    let view=run.decision()?;
    println!("scenario=consecutive fights={} hp={} max_hp={} gold={} terminal={:?} accepted_actions={}",
        view.combat_progress.settled,view.player.hp,view.player.max_hp,view.player.gold,view.terminal,caller.accepted());
    Ok(())
}

fn death(seed:u64)->Result<(),DemoError> {
    let mut initial=start();initial.hp=11;initial.relics=vec![RelicId::BurningBlood];initial.potions=vec![None,None,None];
    let mut run=Run::from_authored_combat(config(),initial,rng(seed))?;
    let mut caller=Caller::new(128);
    enter(&mut run,EncounterId::JawWorm)?;
    take(&mut caller,&mut run,CombatAction::EndTurn)?;
    let view=run.decision()?;
    println!("scenario=death actual_terminal={:?} hp={} accepted_actions={}",view.terminal,view.player.hp,caller.accepted());
    Ok(())
}

fn main()->Result<(),String> {
    let mut scenario="all".to_owned();let mut seed=7_u64;
    let mut args=std::env::args().skip(1);
    while let Some(arg)=args.next() {
        match arg.as_str() {
            "--scenario"=>scenario=args.next().ok_or("--scenario requires a value")?,
            "--seed"=>seed=args.next().ok_or("--seed requires a u64")?.parse().map_err(|_|"--seed requires a u64")?,
            "--help"|"-h"=>{println!("m3_run --scenario all|consecutive|death --seed <u64> (default7)");return Ok(());}
            _=>return Err(format!("unknown argument {arg}")),
        }
    }
    if !["all","consecutive","death"].contains(&scenario.as_str()) {return Err(format!("unknown scenario {scenario}"));}
    println!("M3 authored Ironclad A0 persistent combat checkpoints, seed={seed}; not a normal run");
    if scenario=="all"||scenario=="consecutive" {consecutive(seed).map_err(|e|e.to_string())?;}
    if scenario=="all"||scenario=="death" {death(seed).map_err(|e|e.to_string())?;}
    Ok(())
}
