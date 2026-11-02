//! M28 standard A0 Run owners (design §6–§7). Reached only from `RoomScenario::StandardA0`,
#![cfg_attr(not(any(test,feature="m28-diagnostics")),allow(dead_code))]
//! which every finite scenario helper excludes; the standard branch is checked first.
use super::*;
pub(in crate::run) mod neow;
pub(in crate::run) mod rewards;
pub(in crate::run) mod shop;
pub(in crate::run) mod treasure;
pub(in crate::run) mod map;


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
        let choices=neow::screen(&run)?;
        run.state.room_loop=Some(RoomLoop{scenario:RoomScenario::StandardA0,act1:None,act2:None,act3:None,act4:None,m10_elite_relic_cursor:0,
            screen:RoomView{kind:RoomViewKind::Neow,choices},purge_cost:75,reward_parent:None,active_rewards:None,boss:None,transition:None,
            act2_boss_order:None,act3_boss_order:None,joust:None,joust_visits:0,act3_event:None,act3_end:None});
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

/// Per-act standard state rebuild at each act start (design §7). The ordinary and shrine event
/// lists land in M28c; question-room ramps reset with each generated map.
pub(in crate::run) fn on_act_start(run:&mut Run){ let _=run; }
