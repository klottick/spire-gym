//! Native-boundary regressions for the two Learn-reported faults at accepted source
//! 533dad1c. Each test replays the consumer's retained public reset/action prefix on the
//! same session owner the Gym facades call and checks the formerly faulting action.
use spire_core::{combat::rng::Partition,content::Character};
use spire_env::*;

// spire-tsw: Ironclad A0 RunEnv `m10-ironclad-policy`, Train partition, run seed 484. The
// 109th action plays Infernal Blade; at 533dad1c it generated Clash beside a non-Attack and
// faulted "Infernal Blade generated attack cost unavailable" (fixed by M26 3b7f5ef).
const TSW_RUN_484:[i64;108]=[0,2,1,0,0,2,0,1,0,0,0,1,1,2,1,2,1,0,5,3,4,0,0,0,1,1,0,0,1,2,3,1,0,1,0,1,0,6,2,0,2,2,1,2,0,1,0,4,1,1,0,3,2,0,0,3,1,3,3,0,2,3,2,0,0,3,3,0,1,4,0,0,2,1,0,0,6,12,3,0,5,2,4,0,3,3,0,1,3,1,1,0,0,2,0,0,2,0,4,1,0,3,2,1,1,1,1,0];

#[test]
fn tsw_policy_run_infernal_blade_clash_roll_is_accepted_not_faulted(){
    let config=RunSessionConfig{character:Character::Ironclad,ascension:0,partition:Partition::Train,boss_history:M9BossHistory::AwakenedUnseen,capacities:RunCapacities::m10_default(),record_replay:true};
    let mut session=RunSession::new_m10_policy(config).unwrap();
    let mut token=session.reset(Some(484)).unwrap().token;
    for (i,index) in TSW_RUN_484.iter().enumerate(){
        let step=session.step(token,RunActionIndex(*index)).unwrap_or_else(|e|panic!("public prefix action {i}: {e:?}"));
        assert!(!step.terminated,"public prefix action {i} was not terminal");token=step.boundary.token;
    }
    let decision=session.reproduction().unwrap().latest_decision.unwrap().unwrap();
    assert_eq!(decision.candidates.len(),5);
    assert_eq!(format!("{:?}",decision.candidates[1].action),"Combat(PlayCard { instance: 15, target: None })");
    let step=session.step(token,RunActionIndex(1)).expect("Infernal Blade generating Clash must not fault");
    assert!(!step.terminated);
    assert!(session.reproduction().unwrap().fault.is_none());
}

// spire-bhl: standalone `ironclad-combat-v1`, Train partition, slot 15's final episode
// (seed 101). Its only legal action ends the turn; Bronze Scales THORNS kills byrd-0-0 on
// the first of five pecks and the next peck faulted "unhandled full enemy damage intent".
const BHL_ENTRY:&str=r#"{"entries":[{"deck":[{"card":"defend","upgrades":0},{"card":"defend","upgrades":0},{"card":"defend","upgrades":1},{"card":"fire_breathing","upgrades":0},{"card":"havoc","upgrades":1},{"card":"offering","upgrades":0},{"card":"power_through","upgrades":0},{"card":"rupture","upgrades":1},{"card":"second_wind","upgrades":1},{"card":"shockwave","upgrades":0},{"card":"shrug_it_off","upgrades":0},{"card":"strike","upgrades":0},{"card":"strike","upgrades":0},{"card":"strike","upgrades":0},{"card":"strike","upgrades":0},{"card":"strike","upgrades":0}],"encounter":"three_byrds","hp":94,"id":"s-29d46653301c12a90c36d347a6ebff776e179a0ce0b72890","max_hp":118,"potions":[null,"fire_potion",null],"relic_counters":{},"relics":["blood_vial","bronze_scales","burning_blood","vajra"]}]}"#;
const BHL_SLOT_15:[i64;6]=[5,9,5,3,4,0];

#[test]
fn bhl_standalone_thorns_killed_byrd_stops_its_pecks_without_fault(){
    let catalog=CombatTrainingCatalog::from_json(BHL_ENTRY.as_bytes()).unwrap();
    let mut session=CombatTrainingSession::new(catalog,Partition::Train).unwrap();
    let mut token=session.reset(Some(101),Some("s-29d46653301c12a90c36d347a6ebff776e179a0ce0b72890")).unwrap().token;
    for (i,index) in BHL_SLOT_15.iter().enumerate(){
        let step=session.step(token,RunActionIndex(*index)).unwrap_or_else(|e|panic!("public prefix action {i}: {e:?}"));
        assert!(!step.terminated);token=step.boundary.token;
    }
    let boundary=session.decision().unwrap();
    assert_eq!(boundary.observation.action_mask.iter().filter(|m|**m!=0).count(),1,"the batch-94 slot had one legal action");
    // Enemy rows: [kind,hp,max_hp,block,intent_kind,intent_damage,..,intent_hits]; player [hp,max_hp,gold,block,..].
    let rows=|b:&CombatPolicyBoundary|b.observation.group("enemies").unwrap().values.chunks(12).filter(|r|r[0]!=0).map(|r|r.to_vec()).collect::<Vec<_>>();
    let (player,before)=(boundary.observation.group("player").unwrap().values.clone(),rows(&boundary));
    // Retained state: byrd-0-0 (3 HP, Weak) intends 0-damage pecks; partners intend 1-damage pecks; the
    // player has 96 HP and 14 Block, which absorbs every partner peck (8 land), so HP stays exact.
    assert_eq!((before[0][1],before[0][5],before[1][5],before[2][5]),(3,0,1,1));assert_eq!((player[0],player[3]),(96,14));
    let step=session.step(token,RunActionIndex(0)).expect("THORNS-lethal peck must not fault the session");
    let after=rows(&step.boundary);
    assert_eq!(after[0][1],0,"Bronze Scales THORNS killed byrd-0-0");
    assert_eq!(step.boundary.observation.group("player").unwrap().values[0],96,"no HP loss: one 0-damage peck from byrd-0-0, partners blocked");
    assert_eq!((after[1][1],after[2][1]),(0,11),"byrd-0-1 (8 HP) dies to THORNS on peck 3 of 5 (its last two dropped); byrd-0-2 takes 5x3");
    assert_eq!(session.diagnostic()["fault"],serde_json::Value::Null);
    assert_eq!(session.diagnostic()["accepted_steps"],7);
    assert!(!step.terminated,"two Byrds remain");
}
