//! M28 standard A0 Run owners (design §6–§7). Reached only from `RoomScenario::StandardA0`,
#![cfg_attr(not(any(test,feature="m28-diagnostics")),allow(dead_code))]
//! which every finite scenario helper excludes; the standard branch is checked first.
use super::*;
pub(in crate::run) mod neow;
pub(in crate::run) mod rewards;
pub(in crate::run) mod shop;
pub(in crate::run) mod treasure;
pub(in crate::run) mod map;
pub(in crate::run) mod event_pool;
pub(in crate::run) mod event_frame;
pub(in crate::run) mod event_exec;
pub(in crate::run) mod events;


/// `Run::new_room_scenario(.., RoomScenario::StandardA0)`: any character at A0, full or short
/// Neow by `CreationProfile::neow_eligible`, complete card and item admission (design §7).
/// Diagnostics-only until M28f (PM ruling 2026-11-02).
pub(in crate::run) fn create(config:RunConfig,profile:CreationProfile,rng:RunRng)->Result<Run,RunError>{
    #[cfg(not(any(test,feature="m28-diagnostics")))]
    { let _=(config,profile,rng); Err(RunError::Unsupported("StandardA0 is diagnostics-only until M28f")) }
    #[cfg(any(test,feature="m28-diagnostics"))]
    {
        if config.ascension!=0{return Err(RunError::Unsupported("StandardA0 is A0 only"))}
        let mut run=Run::new(config,profile,rng)?;
        let admission=RunAdmission::new(CardAdmission::Complete,ItemAdmission::Complete)?;
        run.state.card_admission=admission.cards;
        run.state.item_admission=admission.items;
        run.state.relic_pools=Some(crate::items::relic_pools::build(&run.state,&[])?);
        run.state.card_rarity_offset=Some(rewards::OFFSET_START);
        run.state.event_pools=Some(event_pool::EventPools::new(1));
        let choices=neow::screen(&run)?;
        run.state.room_loop=Some(RoomLoop{scenario:RoomScenario::StandardA0,act1:None,act2:None,act3:None,act4:None,m10_elite_relic_cursor:0,
            screen:RoomView{kind:RoomViewKind::Neow,choices},purge_cost:75,reward_parent:None,active_rewards:None,boss:None,transition:None,
            act2_boss_order:None,act3_boss_order:None,joust:None,joust_visits:0,act3_event:None,act3_end:None,standard_event:None});
        Ok(run)
    }
}

/// Generate the current act's map with the boss from the standard selector (design §7).
pub(in crate::run) fn start_act(run:&mut Run)->Result<(),RunError>{
    let mut rng=run.state.rng.clone();
    let mut map=crate::run::map::generate(run.state.act,&mut rng,crate::run::map::MapEligibility::from_run(&run.state))?;
    run.state.rng=rng;
    map.boss_icon=Some(crate::run::world::act_boss(&mut run.state)?);
    run.state.map=Some(map);
    Ok(())
}

/// Per-act standard state rebuild at each act start (design §5.2, §7): the ordinary and shrine
/// event lists; one-time rows and history persist. Question-room ramps reset with each map.
pub(in crate::run) fn on_act_start(run:&mut Run){
    let act=run.state.act;
    if let Some(pools)=run.state.event_pools.as_mut(){pools.rebuild(act);}
}

/// M28c diagnostics (chunk plan §4 M28c item 16): a standard A0 Run past the short Neow, on
/// `act`'s generated map (global floors), whose event lists hold only `event`, and the first-row
/// node made an Event room. Moving there runs the selection law, which can only pick `event`.
#[cfg(any(test,feature="m28-diagnostics"))]
pub(in crate::run) fn diagnostic_event_run(character:crate::content::Character,act:u8,event:EventId,rng:RunRng)->Result<(Run,NodeId),RunError>{
    if !(1..=3).contains(&act){return Err(RunError::InvalidAction)}
    let profile=CreationProfile{neow_eligible:false,..CreationProfile::experienced()};
    let mut run=create(RunConfig{character,ascension:0},profile,rng)?;
    let id=public(&run)?.choices.get(1).ok_or(RunError::InvalidAction)?.id;
    run.step(&GameAction::Choose(id))?;
    if act>1{
        run.state.act=act;
        start_act(&mut run)?;
        let offset=[0,0,17,34][usize::from(act)];
        for node in &mut run.state.map.as_mut().ok_or(RunError::InvalidAction)?.nodes{node.floor+=offset;}
        run.state.floor=offset;
    }
    let pool=crate::content::Catalog::standard().event(event).ok_or(RunError::InvalidAction)?.pool;
    let list=vec![event];
    run.state.event_pools=Some(match pool{
        crate::content::EventPool::Ordinary=>event_pool::EventPools{ordinary:list,shrines:vec![],one_time:vec![],seen:vec![]},
        crate::content::EventPool::Shrine=>event_pool::EventPools{ordinary:vec![],shrines:list,one_time:vec![],seen:vec![]},
        crate::content::EventPool::OneTime=>event_pool::EventPools{ordinary:vec![],shrines:vec![],one_time:list,seen:vec![]},
        crate::content::EventPool::Start=>return Err(RunError::InvalidAction),
    });
    let map=run.state.map.as_mut().ok_or(RunError::InvalidAction)?;
    let node=map.nodes.iter_mut().find(|n|n.y==0).ok_or(RunError::InvalidAction)?;
    node.kind=MapNodeKind::Event;
    let id=node.id;
    Ok((run,id))
}
