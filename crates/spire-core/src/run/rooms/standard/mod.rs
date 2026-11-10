//! M28 standard A0 Run owners (design §6–§7). Reached only from `RoomScenario::Standard`,
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


/// The numbers a Standard room displays (M34a a3/a4); `None` outside `RoomScenario::Standard`.
pub(in crate::run) fn room_params(run:&Run,room:Option<&RoomView>)->Option<RoomParams>{
    if run.state.room_loop.as_ref().map(|r|r.scenario)!=Some(RoomScenario::Standard){return None}
    let mut params=RoomParams::default();
    let Some(room)=room else {return Some(params)};
    if room.kind==RoomViewKind::Treasure&&room.choices.iter().any(|c|c.kind==RoomChoiceKind::OpenChest&&c.status==RoomChoiceStatus::Offered){
        params.stage[0]=map::chest_key(run).ok().and_then(|key|treasure::preview_size(&run.state,&key).ok());
    }
    if matches!(room.kind,RoomViewKind::Event(_)){
        params.rows=room.choices.iter().filter_map(|c|event_frame::row_params(run,&c.kind).map(|values|(c.id,values))).collect();
    }
    Some(params)
}

/// The highest ascension the standard Run admits (M29–M33 design §6.1): the game's A20.
pub const STANDARD_MAX_ASCENSION:u8=20;

/// `Run::new_room_scenario(.., RoomScenario::Standard)`: any character at A0–A20, full or short
/// Neow by `CreationProfile::neow_eligible`, complete card and item admission (design §7).
/// Public in the core crate since M28f2 (design §13, PM ruling Q1); no spire-env profile.
/// A1–A5 are public in every build since M29 close (M29f, folded into M29d); A6–A10 since M30;
/// A11–A15 since M31; A16–A19 since M32; A20 (the Act 3 double boss) since M33.
pub(in crate::run) fn create(config:RunConfig,profile:CreationProfile,rng:RunRng)->Result<Run,RunError>{
    build(config,profile,rng)
}

/// The standard Run at any admitted ascension: `create`, in-crate tests and the M29 diagnostics builders.
pub(in crate::run) fn build(config:RunConfig,profile:CreationProfile,rng:RunRng)->Result<Run,RunError>{
    if config.ascension>STANDARD_MAX_ASCENSION{return Err(RunError::Unsupported("Standard supports A0–A20"))}
    let mut run=Run::new(config,profile,rng)?;
    let admission=RunAdmission::new(CardAdmission::Complete,ItemAdmission::Complete)?;
    run.state.card_admission=admission.cards;
    run.state.item_admission=admission.items;
    // The public contract (review m8): Note For Yourself's stored card must be obtainable by its
    // take row, checked by the same obtain on a scratch state.
    let (note,upgrades)=run.state.stored_note;
    if mutations::obtain_stored_card(&mut run.state.clone(),note,upgrades).is_err(){return Err(RunError::Unsupported("Standard stored note must be a storable card with legal upgrades"))}
    run.state.relic_pools=Some(crate::items::relic_pools::build(&run.state,&[])?);
    run.state.card_rarity_offset=Some(rewards::OFFSET_START);
    run.state.event_pools=Some(event_pool::EventPools::new(1));
    let choices=neow::screen(&run)?;
    run.state.room_loop=Some(RoomLoop{scenario:RoomScenario::Standard,act1:None,act2:None,act3:None,act4:None,m10_elite_relic_cursor:0,
        screen:RoomView{kind:RoomViewKind::Neow,choices},purge_cost:75,reward_parent:None,active_rewards:None,boss:None,transition:None,
        act2_boss_order:None,act3_boss_order:None,joust:None,joust_visits:0,act3_event:None,act3_end:None,standard_event:None});
    Ok(run)
}

/// Generate the current act's map with the boss from the standard selector (design §7).
pub(in crate::run) fn start_act(run:&mut Run)->Result<(),RunError>{
    let mut rng=run.state.rng.clone();
    let mut map=crate::run::map::generate(run.state.act,&mut rng,crate::run::map::MapEligibility::from_run(&run.state))?;
    run.state.rng=rng;
    let order=crate::run::world::act_boss_order(&mut run.state)?;
    map.boss_icon=Some(order[0]);
    run.state.map=Some(map);
    // A diagnostic Act 3 start stores only a two-boss order (A20, all seen; M33 review M5): a one-boss
    // order stays implied by the icon, as before M33.
    if order.len()==2{run.state.room_loop.as_mut().ok_or(RunError::InvalidAction)?.act3_boss_order=Some(order);}
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
