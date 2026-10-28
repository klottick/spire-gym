mod m18_support;
use m18_support::*;
use spire_env::*;
#[test]
fn m18_schema_composes_shared_defect_groups() {
    let caps=RunCapacities::m10_default();let spec=m18_run_observation_spec(&caps).unwrap().descriptor();
    assert_eq!(spec["schema_id"],"spire-gym-m18-defect-act2-observation-v1");
    assert_eq!(spec["profile_id"],"M18DefectAct2");
    let (d,map)=combat_diagnostic();let o=encode_run_m18(&d,map.as_ref(),&caps).unwrap();
    assert_eq!(o.group("permanent_cards").unwrap().shape[1],3);
    assert_eq!(o.group("combat_cards").unwrap().shape[1],19);
    assert_eq!(o.group("orbs").unwrap().shape[1],2);
}

#[test]
fn m18_foreign_public_inventory_cannot_hide_behind_combat_inventory() {
    use spire_core::content::{RelicId,PotionId};
    let caps=RunCapacities::m10_default();let (d,map)=combat_diagnostic();
    let mut bad=d.clone();bad.player.relics.push(RelicId::BurningBlood);
    assert!(encode_run_m18(&bad,map.as_ref(),&caps).is_err(),"foreign Run inventory must reject even while combat has its own public inventory");
    let mut bad=d.clone();bad.player.potions[0]=Some(PotionId::DexterityPotion);
    assert!(encode_run_m18(&bad,map.as_ref(),&caps).is_err());
}

#[test]
fn m18_all_permanent_values_items_and_statuses_use_actual_public_values() {
    use spire_core::{combat::*,content::*,run::*};
    let caps=RunCapacities::m10_default();let descriptor=m18_run_observation_spec(&caps).unwrap().descriptor();
    assert_eq!((M18_DEFECT_ACT2_CARDS.len(),M18_DEFECT_ACT2_RELICS.len(),M18_DEFECT_ACT2_POTIONS.len()),(29,14,5));
    assert_eq!(descriptor["vocabularies"]["card"].as_array().unwrap().len(),34);
    for (kind,id) in M18_DEFECT_ACT2_CARDS.iter().copied().enumerate() { for upgrades in 0..=1 {
        let misc=if id==CardId::GeneticAlgorithm {7} else {0};
        let card=permanent_live_card_with_misc(id,CardInstanceId(701),upgrades,misc);
        let combat=supplied_combat(EncounterId::SphericGuardian,vec![card]);
        let d=project_combat(&combat,vec![PublicObject{instance:701,id,upgrades,misc}]);
        let c=&d.combat.as_ref().unwrap().hand[0];let values=c.values.as_ref().unwrap();
        let o=encode_run_m18(&d,None,&caps).unwrap();let row=&o.group("combat_cards").unwrap().values[..19];
        assert_eq!(&o.group("permanent_cards").unwrap().values[..3],&[kind as i64+1,upgrades as i64,misc as i64]);
        assert_eq!(&row[..8],&[1,0,kind as i64+1,upgrades as i64,c.effective_cost.unwrap_or(0) as i64,0,-1,0]);
        for (column,value) in [(8,values.base_damage),(9,values.base_block),(10,values.magic_number),(11,values.damage_bonus)] {
            let group=o.group("combat_cards").unwrap();assert_eq!(group.known[column],i8::from(value.is_some()));
            if let Some(value)=value {assert_eq!(row[column],value as i64,"{id:?}+{upgrades} column{column}");}
        }
        assert_eq!(row[18],misc as i64);
        let mut bad=d.clone();bad.objects[0].upgrades=2;assert!(encode_run_m18(&bad,None,&caps).is_err());
        let mut bad=d.clone();bad.combat.as_mut().unwrap().hand[0].upgrades=2;assert!(encode_run_m18(&bad,None,&caps).is_err());
    }}
    let mut d=diagnostic_run().decision().unwrap();
    for (row,id) in M18_DEFECT_ACT2_RELICS.iter().copied().enumerate() {
        d.player.relics=vec![id];assert_eq!(encode_run_m18(&d,None,&caps).unwrap().group("relics").unwrap().values[0],row as i64+1);
    }
    for (row,id) in M18_DEFECT_ACT2_POTIONS.iter().copied().enumerate() {
        d.player.potions=vec![Some(id),None,None];let o=encode_run_m18(&d,None,&caps).unwrap();let p=o.group("potions").unwrap();
        assert_eq!(p.values[0],row as i64+1);assert_eq!(&p.known[..4],&[1,1,1,0]);assert_eq!(&p.occupied.as_ref().unwrap()[..4],&[1,0,0,0]);
    }
    for id in [CardId::Dazed,CardId::Burn,CardId::Slimed,CardId::Wound] {for upgrades in 0..=u32::from(id==CardId::Burn) {
        // Actual shared setup computes the generated status public values.
        let mut card=permanent_live_card(id,CardInstanceId(991),upgrades);card.origin=CombatCardRef::CombatLocal(991);
        let combat=supplied_combat(EncounterId::SphericGuardian,vec![card]);let d=project_combat(&combat,vec![]);
        assert!(encode_run_m18(&d,None,&caps).is_ok(),"generated {id:?}+{upgrades}");
        let mut bad=d.clone();bad.combat.as_mut().unwrap().hand[0].upgrades=2;assert!(encode_run_m18(&bad,None,&caps).is_err());
        let mut bad=d.clone();bad.objects=vec![PublicObject{instance:991,id,upgrades,misc:0}];assert!(encode_run_m18(&bad,None,&caps).is_err());
    }}
    for id in [CardId::Shiv,CardId::Void,CardId::Leap,CardId::StrikeGreen,CardId::Unknown("foreign")] {
        let mut bad=d.clone();bad.objects[0].id=id;assert!(encode_run_m18(&bad,None,&caps).is_err());
    }
}

#[test]
fn m18_sparse_orbs_signed_values_capacities_and_identity_privacy() {
    use spire_core::{combat::*,content::*,run::*};
    let caps=RunCapacities::m10_default();let (mut d,map)=combat_diagnostic();
    d.combat.as_mut().unwrap().character.orbs=vec![Some(OrbKind::Lightning),None,Some(OrbKind::Frost),Some(OrbKind::Dark),Some(OrbKind::Plasma),None,None,None,None,None];
    d.combat.as_mut().unwrap().character.orb_dark_damage=vec![None,None,None,Some(i32::MAX),None,None,None,None,None,None];
    d.combat.as_mut().unwrap().character.focus=-3;d.objects[0].misc=i32::MIN;
    let o=encode_run_m18(&d,map.as_ref(),&caps).unwrap();let orbs=o.group("orbs").unwrap();
    assert_eq!(&orbs.values[..10],&[1,0,0,0,2,0,3,i32::MAX as i64,4,0]);
    assert_eq!(orbs.known.iter().step_by(2).sum::<i8>(),10);assert_eq!(&orbs.occupied.as_ref().unwrap()[..5],&[1,0,1,1,1]);
    assert_eq!(o.group("player").unwrap().values[9],-3);assert_eq!(o.group("permanent_cards").unwrap().values[2],i32::MIN as i64);
    let mut sparse=d.clone();sparse.combat.as_mut().unwrap().character.orbs.truncate(5);sparse.combat.as_mut().unwrap().character.orb_dark_damage.truncate(5);
    let padded=encode_run_m18(&sparse,None,&caps).unwrap();assert_eq!(&padded.group("orbs").unwrap().known[10..],&[0;10]);
    assert!(matches!(encode_run_m18(&d,None,&RunCapacities{orbs:9,..caps.clone()}),Err(RunSessionError::Capacity{group:"orbs",required:10,capacity:9})));
    for small in [RunCapacities{actions:d.candidates.len()-1,..caps.clone()},RunCapacities{permanent_cards:9,..caps.clone()},RunCapacities{combat_cards:1,..caps.clone()},RunCapacities{enemies:0,..caps.clone()}] {
        assert!(encode_run_m18(&d,None,&small).is_err());
    }
    let mut bad=d.clone();bad.combat.as_mut().unwrap().character.orb_dark_damage.pop();assert!(encode_run_m18(&bad,None,&caps).is_err());
    let mut bad=d.clone();bad.combat.as_mut().unwrap().character.orb_dark_damage[0]=Some(4);assert!(encode_run_m18(&bad,None,&caps).is_err());
    let mut bad=d.clone();bad.content_profile=Some(RoomScenario::M17DefectAct1);assert!(encode_run_m18(&bad,None,&caps).is_err());
    let mut bad=d.clone();bad.player.character=Character::Silent;assert!(encode_run_m18(&bad,None,&caps).is_err());
    let mut bad=d.clone();bad.terminal=Some(RunTerminal::HeartVictory);assert!(encode_run_m18(&bad,None,&caps).is_err());
    // Public resolver identities are consumed only as row references.
    let mut a=diagnostic_run().decision().unwrap();a.candidates.clear();let before=encode_run_m18(&a,None,&caps).unwrap();
    for c in &mut a.objects {c.instance+=100000;}assert_eq!(encode_run_m18(&a,None,&caps).unwrap(),before);
    let descriptor=m18_run_observation_spec(&caps).unwrap().descriptor();
    for forbidden in ["copy_family","applied_mutation_ordinal","rng_key","enemy_ai","draw_order","future_offer"] {assert!(!descriptor.to_string().contains(forbidden));}
    for (key,field) in [("permanent_cards","misc"),("combat_cards","misc"),("orbs","dark_damage")] {
        let tensor=descriptor["tensors"].as_array().unwrap().iter().find(|t|t["key"]==key).unwrap();let column=tensor["columns"].as_array().unwrap().iter().find(|c|c["field"]==field).unwrap();assert_eq!(column["low"],i32::MIN);assert_eq!(column["high"],i32::MAX);
    }
}

#[test]
fn m18_actual_act2_setups_vocabulary_and_map_icon_admission() {
    use spire_core::{combat::*,content::*,run::*};
    let caps=RunCapacities::m10_default();let descriptor=m18_run_observation_spec(&caps).unwrap().descriptor();
    let encounters=[EncounterId::SphericGuardian,EncounterId::ThreeByrds,EncounterId::CenturionAndHealer,EncounterId::Snecko,EncounterId::ChosenAndByrds,EncounterId::ThreeCultists,EncounterId::GremlinLeader,EncounterId::Slavers,EncounterId::BookOfStabbing,EncounterId::Champ,EncounterId::BronzeAutomaton,EncounterId::Collector];
    let mut species=std::collections::BTreeSet::new();
    for encounter in encounters {
        let mut combat=supplied_combat(encounter,vec![permanent_live_card(CardId::DefendBlue,CardInstanceId(71),0)]);
        if matches!(encounter,EncounterId::BronzeAutomaton|EncounterId::Collector) { combat.step_full(&CombatAction::EndTurn).unwrap(); }
        let mut d=project_combat(&combat,vec![PublicObject{instance:71,id:CardId::DefendBlue,upgrades:0,misc:0}]);
        let o=encode_run_m18(&d,None,&caps).unwrap();
        for (row,enemy) in d.combat.as_ref().unwrap().enemies.iter().enumerate() {
            species.insert(enemy.enemy_id.as_str());let encoded=o.group("enemies").unwrap().values[row*12] as usize;assert_eq!(descriptor["vocabularies"]["enemy"][encoded],enemy.enemy_id.as_str());
        }
        d.last_combat=Some(CombatRecord{visit:1,encounter,outcome:Outcome::Victory});assert!(encode_run_m18(&d,None,&caps).is_ok());
    }
    assert!(species.contains("bronze_orb"));assert!(species.contains("torch_head"));
    let mut d=diagnostic_run().decision().unwrap();
    for event in [EventId::Neow,EventId::UpgradeShrine,EventId::Purifier,EventId::AncientWriting,EventId::Joust] {d.room.as_mut().unwrap().kind=RoomViewKind::Event(event);assert!(encode_run_m18(&d,None,&caps).is_ok());}
    d.room.as_mut().unwrap().kind=RoomViewKind::Event(EventId::Falling);assert!(encode_run_m18(&d,None,&caps).is_err());d.room.as_mut().unwrap().kind=RoomViewKind::Neow;
    // Icons may be encoded on a public map, but cannot become played records.
    for icon in [EncounterId::AwakenedOne,EncounterId::TimeEater,EncounterId::DonuAndDeca] {
        let map=map::PublicMap{boss:Some(icon),act:2,nodes:vec![],current:None,winged_moves:0};assert!(encode_run_m18(&d,Some(&map),&caps).is_ok());
        d.last_combat=Some(CombatRecord{visit:0,encounter:icon,outcome:Outcome::Victory});assert!(encode_run_m18(&d,None,&caps).is_err());d.last_combat=None;
    }
}

#[test]
fn m18_actual_confused_stasis_and_summon_rows_project_public_identity() {
    use spire_core::{combat::*,content::*,run::*};
    let caps=RunCapacities::m10_default();
    let cards=(1..=10).map(|id|permanent_live_card(CardId::DefendBlue,CardInstanceId(id),0)).collect();
    let mut snecko=supplied_combat(EncounterId::Snecko,cards);
    for _ in 0..2 {snecko.step_full(&CombatAction::EndTurn).unwrap();}
    let objects=(1..=10).map(|instance|PublicObject{instance,id:CardId::DefendBlue,upgrades:0,misc:0}).collect();
    let d=project_combat(&snecko,objects);let c=d.combat.as_ref().unwrap();assert!(c.player.powers.iter().any(|p|p.id.0=="confused"));
    let o=encode_run_m18(&d,None,&caps).unwrap();
    for (i,card) in c.hand.iter().enumerate() {assert!((0..=3).contains(&card.effective_cost.unwrap()));assert_eq!(o.group("combat_cards").unwrap().values[i*19+4],card.effective_cost.unwrap() as i64);}
    let cards=(1..=10).map(|id|permanent_live_card_with_misc(CardId::GeneticAlgorithm,CardInstanceId(id),0,3)).collect();
    let objects=(1..=10).map(|instance|PublicObject{instance,id:CardId::GeneticAlgorithm,upgrades:0,misc:3}).collect::<Vec<_>>();
    let mut automaton=supplied_combat(EncounterId::BronzeAutomaton,cards);
    for _ in 0..3 {if !automaton.public_full().unwrap().stasis.is_empty(){break}automaton.step_full(&CombatAction::EndTurn).unwrap();}
    let d=project_combat(&automaton,objects);let c=d.combat.as_ref().unwrap();assert!(!c.stasis.is_empty(),"actual BronzeOrb Stasis must have captured a card");
    let o=encode_run_m18(&d,None,&caps).unwrap();let rows=o.group("combat_cards").unwrap();
    for stasis in &c.stasis {
        let owner=c.enemies.iter().position(|e|e.instance_id==stasis.orb).unwrap();
        let row=rows.values.chunks_exact(19).find(|row|row[0]==5&&row[5]==stasis.card.instance.0 as i64-1).unwrap();
        assert_eq!(row[6],owner as i64);assert_eq!(row[18],3);assert_eq!(row[9],3);
    }
    let mut bad=d.clone();bad.combat.as_mut().unwrap().stasis[0].orb=EnemyHandle::parse("999999").unwrap();assert!(encode_run_m18(&bad,None,&caps).is_err());
    // Actor replacement and logical positions are public; numeric rows keep no
    // opaque enemy handle. The shared owner performs the real summon itself.
    let mut leader=supplied_combat(EncounterId::GremlinLeader,vec![permanent_live_card(CardId::DefendBlue,CardInstanceId(9),0)]);
    leader.step_full(&CombatAction::EndTurn).unwrap();let d=project_combat(&leader,vec![PublicObject{instance:9,id:CardId::DefendBlue,upgrades:0,misc:0}]);
    let o=encode_run_m18(&d,None,&caps).unwrap();for (i,e) in d.combat.as_ref().unwrap().enemies.iter().enumerate(){assert_eq!(o.group("enemies").unwrap().values[i*12+9],e.logical_slot as i64);assert_eq!(o.group("enemies").unwrap().values[i*12+6],i64::from(e.replaced));}
}

#[test]
fn m18_candidate_order_masks_and_targets_are_shared_row_references() {
    use spire_core::combat::*;
    let caps=RunCapacities::m10_default();let (d,map)=combat_diagnostic();let o=encode_run_m18(&d,map.as_ref(),&caps).unwrap();
    assert!(o.action_mask[..d.candidates.len()].iter().all(|x|*x==1));assert!(o.action_mask[d.candidates.len()..].iter().all(|x|*x==0));
    let actions=o.group("actions").unwrap();
    for (i,candidate) in d.candidates.iter().enumerate() {
        let row=&actions.values[i*8..i*8+8];
        match &candidate.action {
            spire_core::run::GameAction::Combat(CombatAction::PlayCard{instance,target})=>{
                let position=d.combat.as_ref().unwrap().hand.iter().position(|c|c.instance.0==*instance).unwrap();assert_eq!(&row[..3],&[8,4,position as i64]);
                if let Some(target)=target {assert_eq!(&row[3..5],&[5,target.0 as i64-1]);}
            },
            spire_core::run::GameAction::Combat(CombatAction::EndTurn)=>assert_eq!(row[0],10),
            _=>(),
        }
    }
    let mut reordered=d.clone();reordered.candidates.reverse();let reversed=encode_run_m18(&reordered,map.as_ref(),&caps).unwrap();
    for i in 0..d.candidates.len() {assert_eq!(&reversed.group("actions").unwrap().values[i*8..i*8+8],&actions.values[(d.candidates.len()-i-1)*8..(d.candidates.len()-i)*8]);}
    let mut extra=d.clone();extra.combat.as_mut().unwrap().character.orbs=vec![None;11];extra.combat.as_mut().unwrap().character.orb_dark_damage=vec![None;11];
    assert!(encode_run_m18(&extra,None,&RunCapacities{orbs:11,..caps}).is_err(),"transport capacity cannot expand gameplay slot cap10");
}

#[test]
fn m18_addition_keeps_frozen_predecessor_descriptors_and_rejects_m9_admission() {
    let caps=RunCapacities::m10_default();
    for (spec,expected) in [
        (run_observation_spec(&caps).unwrap(),"0893bce86e4b3abf11aa384c501503a91ae37f688ec605ab40b814ad365cb999"),
        (m10_run_observation_spec(&caps).unwrap(),"7664874a7a483d1f2e0439ea7b319d1a4be4b9d276d0cb0f908654feda9393db"),
        (m12_run_observation_spec(&caps).unwrap(),"366c572af8141146995e849acc737a33484623790e2b3d661e290fadfb6c9fac"),
        (m13_run_observation_spec(&caps).unwrap(),"3f1e21d50106b8ca71d7cd67a2de7bfa4074455dbbb459629d8e2ac24cf39cf8"),
        (m17_run_observation_spec(&caps).unwrap(),"2176f835eca1384b2070f3d7d001158ef1fead23be5d6d7eea138b57f6bff999"),
    ] {assert_eq!(spec.fingerprint().iter().map(|b|format!("{b:02x}")).collect::<String>(),expected);}
    let d=diagnostic_run().decision().unwrap();assert_eq!(encode_run(&d,None,&caps),Err(RunSessionError::Input("M18 profile is not representable in the M9 v1 observation schema")));
    println!("M18_DESCRIPTOR_SHA256 {}",m18_run_observation_spec(&caps).unwrap().fingerprint().iter().map(|b|format!("{b:02x}")).collect::<String>());
}

#[test]
fn m18_rejects_postcutoff_and_malformed_ready_public_views() {
    use spire_core::run::*;
    // Negative codec probes only. These invented invalid public values are
    // never installed into a session and establish no actual cutoff evidence.
    let caps=RunCapacities::m10_default();let initial=diagnostic_run().decision().unwrap();
    for (act,floor,terminal,hp) in [(3,33,None,75),(3,35,None,75),(3,34,Some(RunTerminal::Death),0),(3,34,None,0),(4,34,None,75)] {
        let mut bad=initial.clone();bad.act=act;bad.floor=floor;bad.terminal=terminal;bad.player.hp=hp;bad.candidates.clear();bad.room.as_mut().unwrap().kind=RoomViewKind::Act3Ready;bad.room.as_mut().unwrap().choices.clear();
        assert!(encode_run_m18(&bad,None,&caps).is_err(),"invalid ready view act{act} floor{floor} terminal{terminal:?} hp{hp}");
    }
}
