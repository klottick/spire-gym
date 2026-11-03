//! Native-boundary regressions for Learn-reported faults (spire-tsw/spire-bhl at accepted
//! source 533dad1c, spire-14g at a86df922). Each test replays the consumer's retained public
//! reset/action prefix on the same session owner the Gym facades call and checks the formerly
//! faulting action.
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

// spire-14g: standalone `ironclad-combat-v1` B16, Train partition, slot 7's final episode
// (seed 533, Donu and Deca). Two Battle Trances stacked No Draw to 2; legal action 0 then plays
// Burning Pact+ (free under Corruption) and its exhaust selection faulted "full player powers".
const N14G_ENTRY:&str=r#"{"entries":[{"deck":[{"card":"anger","upgrades":0},{"card":"bash","upgrades":1},{"card":"battle_trance","upgrades":0},{"card":"battle_trance","upgrades":1},{"card":"burning_pact","upgrades":0},{"card":"burning_pact","upgrades":1},{"card":"burning_pact","upgrades":1},{"card":"burning_pact","upgrades":1},{"card":"cleave","upgrades":1},{"card":"corruption","upgrades":0},{"card":"defend","upgrades":1},{"card":"defend","upgrades":1},{"card":"defend","upgrades":1},{"card":"defend","upgrades":1},{"card":"dropkick","upgrades":1},{"card":"headbutt","upgrades":0},{"card":"inflame","upgrades":0},{"card":"pommel_strike","upgrades":0},{"card":"seeing_red","upgrades":1},{"card":"strike","upgrades":0},{"card":"strike","upgrades":1},{"card":"strike","upgrades":1},{"card":"strike","upgrades":1},{"card":"strike","upgrades":1},{"card":"twin_strike","upgrades":0}],"encounter":"donu_and_deca","hp":57,"id":"s-c24249242ba4a6a8a23dd30a5129803a575bea82a0acedbb","max_hp":95,"potions":[null,null,null],"relic_counters":{},"relics":["brimstone","burning_blood","mark_of_bloom","sacred_bark"]}]}"#;
const N14G_SLOT_7:[i64;12]=[5,5,4,2,0,6,3,1,1,0,0,0];

#[test]
fn n14g_standalone_second_battle_trance_then_burning_pact_selection_is_accepted_not_faulted(){
    let catalog=CombatTrainingCatalog::from_json(N14G_ENTRY.as_bytes()).unwrap();
    let mut session=CombatTrainingSession::new(catalog,Partition::Train).unwrap();
    let mut token=session.reset(Some(533),Some("s-c24249242ba4a6a8a23dd30a5129803a575bea82a0acedbb")).unwrap().token;
    for (i,index) in N14G_SLOT_7.iter().enumerate(){
        let step=session.step(token,RunActionIndex(*index)).unwrap_or_else(|e|panic!("public prefix action {i}: {e:?}"));
        assert!(!step.terminated);token=step.boundary.token;
    }
    let legal=|b:&CombatPolicyBoundary|b.observation.action_mask.iter().filter(|m|**m!=0).count();
    let boundary=session.decision().unwrap();
    assert_eq!(legal(&boundary),2,"Burning Pact+ or End Turn");
    assert_eq!(boundary.observation.group("player").unwrap().values[0],35);
    let step=session.step(token,RunActionIndex(0)).expect("Burning Pact's exhaust selection must not fault");
    assert!(!step.terminated);
    assert_eq!(legal(&step.boundary),2,"exhaust Strike+ or Headbutt");
    let step=session.step(step.boundary.token,RunActionIndex(0)).expect("the exhaust choice resolves");
    assert!(!step.terminated);
    assert_eq!(session.diagnostic()["fault"],serde_json::Value::Null);
    assert_eq!(session.diagnostic()["accepted_steps"],14);
}

// spire-14g soak (Learn qualification shard-0, same standalone config): Bronze Automaton, seed
// 31009. A topdeck leaves card 2 known on top of the draw pile, a Bronze Orb's Stasis then takes it, and the
// End Turn that kills the player faulted publishing the stale known-draw handle.
const N14G_STASIS_ENTRY:&str=r#"{"entries":[{"deck":[{"card":"anger","upgrades":0},{"card":"anger","upgrades":1},{"card":"bash","upgrades":0},{"card":"body_slam","upgrades":1},{"card":"cleave","upgrades":0},{"card":"clothesline","upgrades":1},{"card":"defend","upgrades":0},{"card":"defend","upgrades":0},{"card":"defend","upgrades":1},{"card":"demon_form","upgrades":0},{"card":"headbutt","upgrades":1},{"card":"inflame","upgrades":1},{"card":"perfected_strike","upgrades":0},{"card":"pummel","upgrades":0},{"card":"searing_blow","upgrades":0},{"card":"shrug_it_off","upgrades":1},{"card":"strike","upgrades":0},{"card":"strike","upgrades":1},{"card":"strike","upgrades":1},{"card":"warcry","upgrades":1}],"encounter":"bronze_automaton","hp":26,"id":"s-b87b8351d4b874e428db0e39e1e3557e4030f8fbc140c359","max_hp":88,"potions":["fire_potion",null,null],"relic_counters":{},"relics":["bag_of_preparation","burning_blood"]}]}"#;
const N14G_STASIS_31009:[i64;14]=[6,2,0,2,0,0,0,0,9,5,9,0,5,2];

#[test]
fn n14g_standalone_stasis_taking_a_known_topdeck_ends_in_normal_death_not_fault(){
    let catalog=CombatTrainingCatalog::from_json(N14G_STASIS_ENTRY.as_bytes()).unwrap();
    let mut session=CombatTrainingSession::new(catalog,Partition::Train).unwrap();
    let mut token=session.reset(Some(31009),Some("s-b87b8351d4b874e428db0e39e1e3557e4030f8fbc140c359")).unwrap().token;
    for (i,index) in N14G_STASIS_31009.iter().enumerate(){
        let step=session.step(token,RunActionIndex(*index)).unwrap_or_else(|e|panic!("soak prefix action {i}: {e:?}"));
        assert!(!step.terminated);token=step.boundary.token;
    }
    let boundary=session.decision().unwrap();
    assert_eq!(boundary.observation.action_mask.iter().filter(|m|**m!=0).count(),1,"only End Turn");
    let step=session.step(token,RunActionIndex(0)).expect("Stasis taking a known topdeck must not fault");
    assert!(step.terminated);assert_eq!(step.reward,-1,"a normal death stays a valid terminal");
    assert_eq!(step.boundary.info["outcome"],"death");
    assert_eq!(session.diagnostic()["fault"],serde_json::Value::Null);
}

// spire-14g soak (Learn randomized shard-1, same standalone config): Chosen and Byrds with
// Charon's Ashes, seed 6058. Sever Soul targets byrd-0-0 (7 HP); the first exhaust's Charon's
// Ashes kills it and Sever Soul's own hit at the corpse faulted the damage admission check.
const N14G_ASHES_ENTRY:&str=r#"{"entries":[{"deck":[{"card":"bash","upgrades":1},{"card":"blood_for_blood","upgrades":0},{"card":"clash","upgrades":1},{"card":"clothesline","upgrades":0},{"card":"defend","upgrades":0},{"card":"defend","upgrades":0},{"card":"defend","upgrades":1},{"card":"double_tap","upgrades":0},{"card":"heavy_blade","upgrades":1},{"card":"hemokinesis","upgrades":0},{"card":"perfected_strike","upgrades":0},{"card":"rage","upgrades":1},{"card":"seeing_red","upgrades":1},{"card":"sever_soul","upgrades":0},{"card":"shockwave","upgrades":0},{"card":"spot_weakness","upgrades":0},{"card":"spot_weakness","upgrades":1},{"card":"strike","upgrades":0},{"card":"strike","upgrades":0},{"card":"strike","upgrades":0},{"card":"twin_strike","upgrades":0},{"card":"warcry","upgrades":0},{"card":"warcry","upgrades":1},{"card":"whirlwind","upgrades":0}],"encounter":"chosen_and_byrds","hp":79,"id":"s-5682e012395ef33e17e60d4a4302e33593ce70049de965d0","max_hp":93,"potions":[null,null,null],"relic_counters":{},"relics":["blood_vial","burning_blood","charons_ashes","red_skull"]}]}"#;
const N14G_ASHES_6058:[i64;17]=[6,0,2,0,6,2,1,0,1,3,2,0,3,0,4,1,0];

#[test]
fn n14g_standalone_charons_ashes_killing_sever_souls_target_is_accepted_not_faulted(){
    let catalog=CombatTrainingCatalog::from_json(N14G_ASHES_ENTRY.as_bytes()).unwrap();
    let mut session=CombatTrainingSession::new(catalog,Partition::Train).unwrap();
    let mut token=session.reset(Some(6058),Some("s-5682e012395ef33e17e60d4a4302e33593ce70049de965d0")).unwrap().token;
    for (i,index) in N14G_ASHES_6058.iter().enumerate(){
        let step=session.step(token,RunActionIndex(*index)).unwrap_or_else(|e|panic!("soak prefix action {i}: {e:?}"));
        assert!(!step.terminated);token=step.boundary.token;
    }
    let rows=|b:&CombatPolicyBoundary|b.observation.group("enemies").unwrap().values.chunks(12).filter(|r|r[0]!=0).map(|r|r[1]).collect::<Vec<_>>();
    assert_eq!(rows(&session.decision().unwrap())[0],7,"byrd-0-0 has 7 HP");
    let step=session.step(token,RunActionIndex(0)).expect("Sever Soul at a target Charon's Ashes killed must not fault");
    assert_eq!(rows(&step.boundary)[0],0,"Charon's Ashes killed byrd-0-0");
    assert!(!step.terminated,"the Chosen and another Byrd remain");
    assert_eq!(session.diagnostic()["fault"],serde_json::Value::Null);
}

// spire-14g soak (Learn randomized shard-2, same standalone config): Large Slime with Bronze
// Scales, seed 13006. THORNS halves the Large Slime during its own attack; its queued roll then
// faulted "history/identity mismatch". Now the roll is consumed and Split stays the next move.
const N14G_SLIME_ENTRY:&str=r#"{"entries":[{"deck":[{"card":"bash","upgrades":0},{"card":"defend","upgrades":0},{"card":"defend","upgrades":0},{"card":"defend","upgrades":0},{"card":"defend","upgrades":1},{"card":"feel_no_pain","upgrades":0},{"card":"hemokinesis","upgrades":0},{"card":"iron_wave","upgrades":1},{"card":"strike","upgrades":0},{"card":"strike","upgrades":0},{"card":"strike","upgrades":0},{"card":"strike","upgrades":0},{"card":"strike","upgrades":0}],"encounter":"large_slime","hp":65,"id":"s-17f196b587ef621dde737579767857af04652b2aff588b6a","max_hp":117,"potions":[null,null,null],"relic_counters":{},"relics":["bronze_scales","burning_blood","pear","sacred_bark"]}]}"#;
const N14G_SLIME_13006:[i64;12]=[0,2,0,2,4,4,1,1,0,3,0,2];

#[test]
fn n14g_standalone_large_slime_halved_by_thorns_keeps_split_not_fault(){
    let catalog=CombatTrainingCatalog::from_json(N14G_SLIME_ENTRY.as_bytes()).unwrap();
    let mut session=CombatTrainingSession::new(catalog,Partition::Train).unwrap();
    let mut token=session.reset(Some(13006),Some("s-17f196b587ef621dde737579767857af04652b2aff588b6a")).unwrap().token;
    for (i,index) in N14G_SLIME_13006.iter().enumerate(){
        let step=session.step(token,RunActionIndex(*index)).unwrap_or_else(|e|panic!("soak prefix action {i}: {e:?}"));
        assert!(!step.terminated);token=step.boundary.token;
    }
    let step=session.step(token,RunActionIndex(0)).expect("a Large Slime halved by THORNS in its own attack must not fault");
    assert!(!step.terminated);
    assert_eq!(session.diagnostic()["fault"],serde_json::Value::Null);
    let enemies=|b:&CombatPolicyBoundary|b.observation.group("enemies").unwrap().values.chunks(12).filter(|r|r[0]!=0&&r[1]>0).count();
    assert_eq!(enemies(&step.boundary),1,"the halved Large Slime has not split yet");
    // Its next enemy turn performs the split into two Medium Slimes, without a fault.
    let (mut boundary,mut most)=(step.boundary,1);
    for _ in 0..40{
        let step=session.step(boundary.token,RunActionIndex(0)).expect("the split turn resolves");
        if step.terminated{break}boundary=step.boundary;most=most.max(enemies(&boundary));
    }
    assert_eq!(most,2,"the Split selected during its own attack replaced it with two children");
    assert_eq!(session.diagnostic()["fault"],serde_json::Value::Null);
}

// spire-14g wide soak (Learn randomized shard-0, same standalone config): Four Shapes with Fire
// Breathing+ and Dark Embrace, seed 254129. Fire Breathing's per-target hits were queued while
// the Exploder lived; its suicide then left one hit at the corpse, which faulted admission.
const N14G_SHAPES_ENTRY:&str=r#"{"entries":[{"deck":[{"card":"bash","upgrades":0},{"card":"cleave","upgrades":1},{"card":"clothesline","upgrades":1},{"card":"corruption","upgrades":0},{"card":"dark_embrace","upgrades":0},{"card":"dark_embrace","upgrades":0},{"card":"dark_embrace","upgrades":1},{"card":"dark_embrace","upgrades":1},{"card":"dark_embrace","upgrades":1},{"card":"defend","upgrades":0},{"card":"defend","upgrades":1},{"card":"defend","upgrades":1},{"card":"disarm","upgrades":1},{"card":"feel_no_pain","upgrades":0},{"card":"feel_no_pain","upgrades":0},{"card":"feel_no_pain","upgrades":1},{"card":"feel_no_pain","upgrades":1},{"card":"feel_no_pain","upgrades":1},{"card":"feel_no_pain","upgrades":1},{"card":"fire_breathing","upgrades":1},{"card":"headbutt","upgrades":0},{"card":"headbutt","upgrades":1},{"card":"headbutt","upgrades":1},{"card":"hemokinesis","upgrades":1},{"card":"strike","upgrades":1},{"card":"strike","upgrades":1},{"card":"sword_boomerang","upgrades":1},{"card":"thunderclap","upgrades":1},{"card":"wild_strike","upgrades":1}],"encounter":"four_shapes","hp":88,"id":"s-d74f44d2305cb67a87e811517e10755a9a57f0411ce14029","max_hp":115,"potions":[null,"elixir","elixir"],"relic_counters":{},"relics":["burning_blood"]}]}"#;
const N14G_SHAPES_254129:[i64;17]=[6,4,3,3,1,1,1,0,2,1,1,1,1,2,0,0,10];

#[test]
fn n14g_standalone_fire_breathing_hit_at_a_suicided_exploder_is_skipped_not_fault(){
    let catalog=CombatTrainingCatalog::from_json(N14G_SHAPES_ENTRY.as_bytes()).unwrap();
    let mut session=CombatTrainingSession::new(catalog,Partition::Train).unwrap();
    let mut token=session.reset(Some(254129),Some("s-d74f44d2305cb67a87e811517e10755a9a57f0411ce14029")).unwrap().token;
    for (i,index) in N14G_SHAPES_254129.iter().enumerate(){
        let step=session.step(token,RunActionIndex(*index)).unwrap_or_else(|e|panic!("soak prefix action {i}: {e:?}"));
        assert!(!step.terminated);token=step.boundary.token;
    }
    session.step(token,RunActionIndex(7)).expect("a Fire Breathing hit at a suicided Exploder must not fault");
    assert_eq!(session.diagnostic()["fault"],serde_json::Value::Null);
}

// spire-85d: frozen encoders lacked engine-producible powers. Native fuzz first-legal prefixes
// (evidence native-fuzz-20261101 priority-repros); the final action formerly committed and then
// returned DeliveryFault("post-step Run encoding failed").
fn step_prefix(session:&mut RunSession,seed:u64,prefix:&[i64])->RunBoundary{
    let mut boundary=session.reset(Some(seed)).unwrap();
    for (i,index) in prefix.iter().enumerate(){
        let step=session.step(boundary.token,RunActionIndex(*index)).unwrap_or_else(|e|panic!("public prefix action {i}: {e:?}"));
        assert!(!step.terminated,"public prefix action {i} was not terminal");boundary=step.boundary;
    }
    assert!(session.reproduction().unwrap().fault.is_none());
    boundary
}
fn has_power(session:&RunSession,player:bool,name:&str)->bool{
    let decision=session.reproduction().unwrap().latest_decision.unwrap().unwrap();let combat=decision.combat.unwrap();
    if player{combat.player.powers.iter().any(|p|p.id.0==name)}else{combat.enemies.iter().any(|e|e.powers.iter().any(|p|p.id.0==name))}
}

// M10 Ironclad `RunSession::new_m10`, Test partition: the 52nd first-legal action plays Flex,
// which publishes the player power `lose_strength`.
#[test]
fn m10_flex_lose_strength_steps_cleanly(){
    let config=RunSessionConfig{character:Character::Ironclad,ascension:0,partition:Partition::Test,boss_history:M9BossHistory::AwakenedUnseen,capacities:RunCapacities::m10_default(),record_replay:true};
    let mut session=RunSession::new_m10(config).unwrap();
    step_prefix(&mut session,1_537_092_705_410_732_568,&[0;52]);
    assert!(has_power(&session,true,"lose_strength"));
    session.decision().expect("published decision after Flex");
}

// M22 Watcher `RunSession::new_m22`, Test partition: the 98th action moves into the burning
// elite Gremlin Nob, which publishes the enemy power `regenerate` (RegenerateMonsterPower; it was
// `regeneration`, the player's RegenPower id, before spire-oto.27).
const M22_BURNING_NOB:[i64;98]=[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,3,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,2,0,0,0,0,0,0,0,3,0,0,0,0,0,0,0,0,0,0,2,0,0,3,0,0,0,0,0,0,0,0,0,0,0];
#[test]
fn m22_burning_elite_nob_regeneration_steps_cleanly(){
    let config=RunSessionConfig{character:Character::Watcher,ascension:0,partition:Partition::Test,boss_history:M9BossHistory::Act1GuardianUnseen,capacities:RunCapacities::m10_default(),record_replay:true};
    let mut session=RunSession::new_m22(config).unwrap();
    step_prefix(&mut session,3_958_066_649_157_562_556,&M22_BURNING_NOB);
    assert!(has_power(&session,false,"regenerate"));
    session.decision().expect("published decision in the burning elite fight");
}

// spire-oto.2 (fuzz F03): generated CatalogCombat entry, Test partition, seed 8832734076836322328. The
// choose-none prefix stalls three_shapes until Repulsor's Dazed fill the visible zones; at
// 3312c052 the last action needed 515 combat_cards rows against the 512-row policy capacity.
const F03_THREE_SHAPES_ENTRY:&str=r#"{"profile_id":"ironclad-combat-v2","entries":[{"deck":[{"card":"reckless_charge","upgrades":0},{"card":"cleave","upgrades":0},{"card":"pommel_strike","upgrades":0},{"card":"spot_weakness","upgrades":0},{"card":"corruption","upgrades":0},{"card":"feel_no_pain","upgrades":0},{"card":"bloodletting","upgrades":0},{"card":"fiend_fire","upgrades":0},{"card":"true_grit","upgrades":0},{"card":"flame_barrier","upgrades":0}],"encounter":"three_shapes","gold":99,"hp":80,"id":"encounter-27-three_shapes","max_hp":80,"potions":["dexterity_potion",null,null],"relic_counters":{},"relics":["magic_flower"]}]}"#;
const F03_THREE_SHAPES_SEED:u64=8832734076836322328;
const F03_THREE_SHAPES_ACTIONS:[i64;234]=[2,0,0,0,0,1,0,0,0,1,1,0,0,0,0,0,0,2,0,0,2,0,0,1,0,0,0,2,0,0,0,1,2,0,0,0,0,0,1,1,0,1,1,0,0,0,0,1,0,0,0,0,1,0,0,1,1,1,0,0,0,0,0,0,1,1,1,1,1,0,0,0,0,0,0,0,0,1,0,0,1,0,0,1,0,0,1,0,0,0,0,0,0,0,0,1,0,0,0,0,1,1,1,0,0,0,0,1,0,0,1,0,0,1,1,1,1,0,0,1,1,0,0,1,1,1,1,0,0,0,0,1,0,0,0,0,0,0,1,0,0,0,0,0,0,1,1,0,0,1,0,0,1,1,1,1,0,0,1,1,0,0,1,1,0,0,1,0,0,1,0,0,1,0,0,1,1,0,0,0,0,1,0,0,1,1,1,1,1,0,0,1,1,0,0,0,0,1,1,0,0,0,0,0,0,0,0,1,1,1,1,0,0,1,0,0,0,0,0,0,1,0,0,1,0,0,1,0,0,1,0,0,1,0];

#[test]
fn oto2_long_three_shapes_stall_fits_the_policy_combat_card_capacity(){
    let catalog=CombatTrainingCatalog::from_json(F03_THREE_SHAPES_ENTRY.as_bytes()).unwrap();
    let mut session=CombatTrainingSession::new(catalog,Partition::Test).unwrap();
    let mut token=session.reset(Some(F03_THREE_SHAPES_SEED),Some("encounter-27-three_shapes")).unwrap().token;
    let mut rows=0;
    for (i,index) in F03_THREE_SHAPES_ACTIONS.iter().enumerate(){
        let step=session.step(token,RunActionIndex(*index)).unwrap_or_else(|e|panic!("public prefix action {i}: {e:?}"));
        assert!(!step.terminated,"public prefix action {i} was not terminal");token=step.boundary.token;
        rows=step.boundary.observation.group("combat_cards").unwrap().occupied.as_ref().unwrap().iter().filter(|o|**o!=0).count();
    }
    assert_eq!(rows,511);
    // The recorded failing action: the policy's next index 0 (End Turn) adds the last rows.
    let step=session.step(token,RunActionIndex(0)).expect("the 515-row boundary must encode, not fault on capacity");
    assert!(!step.terminated);
    rows=step.boundary.observation.group("combat_cards").unwrap().occupied.as_ref().unwrap().iter().filter(|o|**o!=0).count();
    assert_eq!(rows,515,"the stall exceeds the former 512-row capacity");
    assert!(rows<=combat_policy_capacities().combat_cards);
    assert_eq!(session.diagnostic()["fault"],serde_json::Value::Null);
}
