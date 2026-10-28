use spire_env::{defect_combat_observation_spec,encode_defect_combat,RunCapacities,RunSessionError};
use spire_core::{run::*,content::{CardId,EncounterId},combat::{OrbKind,rng::Partition}};
fn canonical()->Run{Run::from_m16_defect_combat(M16DefectCombatStart::Canonical,RunRng::new([7;32],Partition::Test,0)).unwrap()}
#[test]
fn m16_defect_observation_public_values_and_holes(){
    let caps=RunCapacities::m10_default();let spec=defect_combat_observation_spec(&caps).unwrap().descriptor();
    assert_eq!(spec["schema_id"],"spire-gym-m16-defect-combat-observation-v1");assert_eq!(spec["profile_id"],"M16DefectCombat");
    assert_eq!(spec["action_space"]["n"],caps.actions);
    let mut run=canonical();run.enter_combat(EncounterId::Cultist).unwrap();let mut d=run.decision().unwrap();
    let obs=encode_defect_combat(&d,&caps).unwrap();assert_eq!(obs.action_mask.len(),caps.actions);
    assert!(obs.action_mask[..d.candidates.len()].iter().all(|v|*v==1));assert!(obs.action_mask[d.candidates.len()..].iter().all(|v|*v==0));
    assert_eq!(obs.group("actions").unwrap().shape,vec![caps.actions,8]);
    for (i,candidate) in d.candidates.iter().enumerate(){
        let actions=obs.group("actions").unwrap();let row=&actions.values[i*8..i*8+8];
        if let GameAction::Combat(spire_core::combat::CombatAction::PlayCard{instance,target})=&candidate.action{
            let hand=&d.combat.as_ref().unwrap().hand;let index=hand.iter().position(|c|c.instance.0==*instance).unwrap();
            assert_eq!((row[0],row[1],row[2]),(8,4,index as i64));
            if let Some(target)=target{assert_eq!((row[3],row[4]),(5,target.0 as i64-1));}
        }
    }
    let c=d.combat.as_mut().unwrap();c.character.orbs=vec![None,Some(OrbKind::Dark),None,Some(OrbKind::Frost)];c.character.orb_dark_damage=vec![None,Some(12),None,None];
    let obs=encode_defect_combat(&d,&caps).unwrap();let orbs=obs.group("orbs").unwrap();
    assert_eq!(orbs.shape,vec![10,2]);assert_eq!(&orbs.values[..8],&[0,0,3,12,0,0,2,0]);assert_eq!(&orbs.known[..10],&[1,0,1,1,1,0,1,0,0,0]);
    assert_eq!(&orbs.occupied.as_ref().unwrap()[..5],&[0,1,0,1,0]);
    assert!(encode_defect_combat(&d,&RunCapacities{orbs:3,..caps.clone()}).is_err());
    let saved=obs.clone();d.combat.as_mut().unwrap().character.orb_dark_damage[1]=Some(99);assert_eq!(obs,saved);
    let mut bad=d.clone();bad.objects[0].id=CardId::Unknown("not_in_vocabulary");assert!(encode_defect_combat(&bad,&caps).is_err());
}
#[test]
fn m16_defect_observation_capacity_privacy_and_compatibility(){
    let caps=RunCapacities::m10_default();let mut run=canonical();run.enter_combat(EncounterId::Cultist).unwrap();let mut d=run.decision().unwrap();
    d.combat.as_mut().unwrap().character.orbs=vec![None;10];d.combat.as_mut().unwrap().character.orb_dark_damage=vec![None;10];
    assert!(matches!(encode_defect_combat(&d,&RunCapacities{orbs:9,..caps.clone()}),Err(RunSessionError::Capacity{group:"orbs",required:10,capacity:9})));
    let mut wrong=d.clone();wrong.player.character=spire_core::content::Character::Ironclad;assert!(encode_defect_combat(&wrong,&caps).is_err());
    // Existing run_m10_observation/run_m15_schema tests own frozen predecessor fingerprint sentinels.
}

#[test]
fn m16_defect_observation_supplied_actual_mutations_and_signed_columns(){
    use spire_core::combat::{CombatAction,TargetId};
    let caps=RunCapacities::m10_default();
    let start=AuthoredRunStart{hp:75,max_hp:75,gold:99,deck:[CardId::GeneticAlgorithm,CardId::SteamBarrier,CardId::Streamline].into_iter().enumerate().map(|(i,id)|CardInstance{id,instance:i as u64+1,upgrades:0,misc:if id==CardId::GeneticAlgorithm{1}else{0}}).collect(),relics:vec![],potions:vec![None;3],relic_counters:Default::default()};
    let mut run=Run::from_m16_defect_combat(M16DefectCombatStart::Supplied(start),RunRng::new([7;32],Partition::Test,0)).unwrap();
    run.enter_combat(EncounterId::Cultist).unwrap();
    for action in [CombatAction::PlayCard{instance:1,target:None},CombatAction::PlayCard{instance:2,target:None},CombatAction::PlayCard{instance:3,target:Some(TargetId(1))}]{run.choose_combat(action).unwrap();}
    let d=run.decision().unwrap();let obs=encode_defect_combat(&d,&caps).unwrap();
    let permanent=obs.group("permanent_cards").unwrap();assert_eq!(permanent.values[2],3);
    let cards=obs.group("combat_cards").unwrap();let rows=cards.values.chunks_exact(19).filter(|r|r[0]!=0).collect::<Vec<_>>();
    for card in d.combat.as_ref().unwrap().discard.iter().chain(&d.combat.as_ref().unwrap().exhaust){
        let row=rows.iter().find(|r|r[5]==card.instance.0 as i64-1).unwrap();
        assert_eq!(row[18],i64::from(card.misc));assert_eq!(row[4],i64::from(card.effective_cost.unwrap()));
        match card.id{CardId::GeneticAlgorithm=>assert_eq!(row[18],3),CardId::SteamBarrier=>assert_eq!(row[9],5),CardId::Streamline=>assert_eq!(row[4],1),_=>unreachable!()}
    }
    let actions=obs.group("actions").unwrap();
    for (i,candidate) in d.candidates.iter().enumerate(){assert_eq!(obs.action_mask[i],1);if matches!(candidate.action,GameAction::Combat(CombatAction::EndTurn)){assert_eq!(actions.values[i*8],10);}}
    assert!(obs.action_mask[d.candidates.len()..].iter().all(|v|*v==0));
    let descriptor=defect_combat_observation_spec(&caps).unwrap().descriptor();
    for (key,field) in [("permanent_cards","misc"),("combat_cards","misc"),("orbs","dark_damage")]{
        let tensor=descriptor["tensors"].as_array().unwrap().iter().find(|t|t["key"]==key).unwrap();let column=tensor["columns"].as_array().unwrap().iter().find(|c|c["field"]==field).unwrap();
        assert_eq!(column["low"],i32::MIN);assert_eq!(column["high"],i32::MAX);
    }
    assert!(!format!("{descriptor}").contains("copy_family"));assert!(!format!("{descriptor}").contains("applied_mutation"));
}

#[test]
fn m16_defect_observation_real_capacity_and_setup_projection(){
    use spire_core::combat::CombatAction;
    let start=AuthoredRunStart{hp:75,max_hp:75,gold:99,deck:(1..=3).map(|instance|CardInstance{id:CardId::Capacitor,instance,upgrades:1,misc:0}).collect(),relics:vec![],potions:vec![None;3],relic_counters:Default::default()};
    let mut run=Run::from_m16_defect_combat(M16DefectCombatStart::Supplied(start),RunRng::new([7;32],Partition::Test,0)).unwrap();
    run.enter_combat(EncounterId::Lagavulin).unwrap();for instance in 1..=3{run.choose_combat(CombatAction::PlayCard{instance,target:None}).unwrap();}
    let d=run.decision().unwrap();assert_eq!(d.combat.as_ref().unwrap().character.orbs.len(),10);
    let caps=RunCapacities::m10_default();let encoded=encode_defect_combat(&d,&caps).unwrap();assert_eq!(encoded.group("orbs").unwrap().known.iter().step_by(2).sum::<i8>(),10);
    assert!(matches!(encode_defect_combat(&d,&RunCapacities{orbs:9,..caps}),Err(RunSessionError::Capacity{group:"orbs",required:10,capacity:9})));
}
