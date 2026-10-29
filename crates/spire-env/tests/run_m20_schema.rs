mod m20_support;
use m20_support::*;
use spire_env::*;
use spire_core::{combat::*,content::*,run::*};

#[test]
fn m20_schema_composes_full_blue_with_shared_offer_and_defect_groups() {
    let caps=RunCapacities::m10_default();
    let spec=m20_run_observation_spec(&caps).unwrap().descriptor();
    assert_eq!(spec["schema_id"],RUN_OBSERVATION_M20_SCHEMA_ID);
    assert_eq!(spec["profile_id"],"M20DefectHeart");
    assert_eq!((M20_DEFECT_HEART_CARDS.len(),M20_DEFECT_HEART_RELICS.len(),M20_DEFECT_HEART_POTIONS.len()),(76,26,33));
    for (key,count) in [("card",115),("relic",26),("potion",33)] {
        let rows=spec["vocabularies"][key].as_array().unwrap();assert_eq!(rows.len(),count+1);
        assert_eq!(rows.iter().map(|x|x.as_str().unwrap()).collect::<std::collections::BTreeSet<_>>().len(),count+1);
    }
    for foreign in ["strike_green","shiv","searing_blow","unknown"] {assert!(!spec["vocabularies"]["card"].as_array().unwrap().iter().any(|x|x==foreign));}
    let (d,map)=combat_diagnostic();let o=encode_run_m20(&d,map.as_ref(),&caps).unwrap();
    for (key,width) in [("permanent_cards",3),("combat_cards",19),("orbs",2),("combat_card_offers",2)] {assert_eq!(o.group(key).unwrap().shape[1],width);}
}

#[test]
fn m20_all_permanent_base_plus_and_current_public_values_are_lossless() {
    let caps=RunCapacities::m10_default();
    for (kind,id) in M20_DEFECT_HEART_CARDS.iter().copied().enumerate() {for upgrades in 0..=u32::from(id!=CardId::Parasite) {
        let misc=if id==CardId::GeneticAlgorithm {17}else{0};
        let card=permanent_live_card_with_misc(id,CardInstanceId(701),upgrades,misc);
        let combat=supplied_combat(EncounterId::SphericGuardian,vec![card]);
        let d=project_combat(&combat,vec![PublicObject{instance:701,id,upgrades,misc}]);
        let o=encode_run_m20(&d,None,&caps).unwrap();let group=o.group("combat_cards").unwrap();let row=&group.values[..19];
        let c=&d.combat.as_ref().unwrap().hand[0];let v=c.values.as_ref();if id!=CardId::Parasite{assert!(v.is_some(),"every admitted blue kind has public values: {id:?}");}
        assert_eq!(&o.group("permanent_cards").unwrap().values[..3],&[kind as i64+1,upgrades as i64,misc as i64]);
        assert_eq!(&row[..8],&[1,0,kind as i64+1,upgrades as i64,c.effective_cost.unwrap_or(0) as i64,0,-1,0]);
        for (column,value) in [(8,v.and_then(|v|v.base_damage)),(9,v.and_then(|v|v.base_block)),(10,v.and_then(|v|v.magic_number)),(11,v.and_then(|v|v.damage_bonus))] {
            assert_eq!(group.known[column],i8::from(value.is_some()));if let Some(value)=value {assert_eq!(row[column],i64::from(value),"{id:?}+{upgrades} column{column}");}
        }
        assert_eq!(row[18],i64::from(misc));
        if id!=CardId::Parasite {let mut mutated=d.clone();let live=&mut mutated.combat.as_mut().unwrap().hand[0];live.effective_cost=Some(99);
        let values=live.values.as_mut().unwrap();values.free_play=true;values.temporary_cost=-9;values.base_block=Some(-7);
        let encoded=encode_run_m20(&mutated,None,&caps).unwrap();let row=&encoded.group("combat_cards").unwrap().values[..19];
        assert_eq!((row[4],row[9],row[13],row[17]),(99,-7,1,-9));}
        let mut bad=d.clone();bad.objects[0].upgrades=2;assert!(encode_run_m20(&bad,None,&caps).is_err());
    }}
}

#[test]
fn m20_all_inventory_and_generated_status_rows_have_exact_admission() {
    let caps=RunCapacities::m10_default();let d=diagnostic_run().decision().unwrap();
    for (row,id) in M20_DEFECT_HEART_RELICS.iter().copied().enumerate() {
        let mut one=d.clone();one.player.relics=vec![id];one.player.relic_counters.clear();
        assert_eq!(encode_run_m20(&one,None,&caps).unwrap().group("relics").unwrap().values[0],row as i64+1);
    }
    for (row,id) in M20_DEFECT_HEART_POTIONS.iter().copied().enumerate() {
        let mut one=d.clone();one.player.potions=vec![Some(id),None,None];let o=encode_run_m20(&one,None,&caps).unwrap();let p=o.group("potions").unwrap();
        assert_eq!(p.values[0],row as i64+1);assert_eq!(&p.known[..4],&[1,1,1,0]);assert_eq!(&p.occupied.as_ref().unwrap()[..4],&[1,0,0,0]);
    }
    for id in [CardId::Dazed,CardId::Burn,CardId::Slimed,CardId::Wound,CardId::Void] {for upgrades in 0..=u32::from(id==CardId::Burn) {
        let mut card=permanent_live_card(id,CardInstanceId(991),upgrades);card.origin=CombatCardRef::CombatLocal(991);
        let combat=supplied_combat(EncounterId::SphericGuardian,vec![card]);let mut one=project_combat(&combat,vec![]);
        assert!(encode_run_m20(&one,None,&caps).is_ok());
        one.objects=vec![PublicObject{instance:991,id,upgrades,misc:0}];assert!(encode_run_m20(&one,None,&caps).is_err());
    }}
    for id in [CardId::StrikeGreen,CardId::SearingBlow,CardId::Shiv,CardId::Unknown("foreign")] {
        let mut bad=d.clone();bad.objects[0].id=id;assert!(encode_run_m20(&bad,None,&caps).is_err());
    }
    let mut bad=d.clone();bad.player.relics.push(RelicId::BurningBlood);assert!(encode_run_m20(&bad,None,&caps).is_err());
    let mut bad=d.clone();bad.player.potions[0]=Some(PotionId::GhostInAJar);assert!(encode_run_m20(&bad,None,&caps).is_err());
    let mut bad=d.clone();bad.content_profile=Some(RoomScenario::M19DefectAct3);assert!(encode_run_m20(&bad,None,&caps).is_err());
    let mut bad=d.clone();bad.player.character=Character::Silent;assert!(encode_run_m20(&bad,None,&caps).is_err());
}

#[test]
fn m20_orb_empty_padding_signed_values_and_capacity_are_distinct() {
    let caps=RunCapacities::m10_default();let (mut d,_)=combat_diagnostic();let c=d.combat.as_mut().unwrap();
    c.character.orbs=vec![Some(OrbKind::Lightning),None,Some(OrbKind::Frost),Some(OrbKind::Dark),Some(OrbKind::Plasma),None,None,None,None,None];
    c.character.orb_dark_damage=vec![None,None,None,Some(i32::MIN),None,None,None,None,None,None];c.character.focus=-999;d.objects[0].misc=i32::MAX;
    let o=encode_run_m20(&d,None,&caps).unwrap();let orbs=o.group("orbs").unwrap();
    assert_eq!(&orbs.values[..10],&[1,0,0,0,2,0,3,i32::MIN as i64,4,0]);assert_eq!(orbs.known.iter().step_by(2).sum::<i8>(),10);
    assert_eq!(&orbs.occupied.as_ref().unwrap()[..5],&[1,0,1,1,1]);assert_eq!(o.group("player").unwrap().values[9],-999);
    assert_eq!(o.group("permanent_cards").unwrap().values[2],i32::MAX as i64);
    let mut sparse=d.clone();sparse.combat.as_mut().unwrap().character.orbs.truncate(5);sparse.combat.as_mut().unwrap().character.orb_dark_damage.truncate(5);
    assert_eq!(&encode_run_m20(&sparse,None,&caps).unwrap().group("orbs").unwrap().known[10..],&[0;10]);
    assert!(matches!(encode_run_m20(&d,None,&RunCapacities{orbs:9,..caps.clone()}),Err(RunSessionError::Capacity{group:"orbs",required:10,capacity:9})));
    let mut bad=d.clone();bad.combat.as_mut().unwrap().character.orb_dark_damage[0]=Some(2);assert!(encode_run_m20(&bad,None,&caps).is_err());
    let mut bad=d.clone();bad.combat.as_mut().unwrap().character.orbs.push(None);bad.combat.as_mut().unwrap().character.orb_dark_damage.push(None);
    assert!(encode_run_m20(&bad,None,&RunCapacities{orbs:11,..caps.clone()}).is_err());
}

#[test]
fn m20_offer_rows_candidate_order_and_exact_capacity_use_shared_references() {
    let caps=RunCapacities::m10_default();let (mut d,_)=combat_diagnostic();
    let c=d.combat.as_mut().unwrap();c.card_offer=Some(CombatCardOfferView{cards:vec![CardId::Buffer,CardId::Seek],optional:true});c.selection=None;
    d.candidates=[CombatAction::ChooseCardOffer{index:1},CombatAction::SkipCardOffer,CombatAction::ChooseCardOffer{index:0}].into_iter().enumerate().map(|(i,a)|Candidate{id:ChoiceId(i as u64),action:GameAction::Combat(a),label:"projection diagnostic",neow_offer:None,object:None}).collect();
    let o=encode_run_m20(&d,None,&caps).unwrap();assert_eq!(&o.action_mask[..4],&[1,1,1,0]);
    let actions=o.group("actions").unwrap();assert_eq!(&actions.values[..3],&[12,11,1]);assert_eq!(actions.values[8],13);assert_eq!(&actions.values[16..19],&[12,11,0]);
    let exact=RunCapacities{actions:3,offers:2,..caps.clone()};assert!(encode_run_m20(&d,None,&exact).is_ok());
    assert!(matches!(encode_run_m20(&d,None,&RunCapacities{actions:2,..caps.clone()}),Err(RunSessionError::Capacity{group:"actions",required:3,capacity:2})));
    assert!(matches!(encode_run_m20(&d,None,&RunCapacities{offers:1,..caps.clone()}),Err(RunSessionError::Capacity{group:"combat_card_offers",required:2,capacity:1})));
    d.combat.as_mut().unwrap().card_offer.as_mut().unwrap().optional=false;assert!(encode_run_m20(&d,None,&caps).is_err());
}

#[test]
fn m20_actual_act4_setups_and_all_shared_ending_rows_are_representable() {
    let caps=RunCapacities::m10_default();let spec=m20_run_observation_spec(&caps).unwrap().descriptor();
    for id in [EncounterId::ShieldAndSpear,EncounterId::CorruptHeart] {
        let combat=supplied_combat(id,vec![permanent_live_card(CardId::DefendBlue,CardInstanceId(71),0)]);
        let mut d=project_combat(&combat,vec![PublicObject{instance:71,id:CardId::DefendBlue,upgrades:0,misc:0}]);d.act=4;d.floor=if id==EncounterId::CorruptHeart{55}else{53};
        let o=encode_run_m20(&d,None,&caps).unwrap();
        for (i,e) in d.combat.as_ref().unwrap().enemies.iter().enumerate() {assert_eq!(spec["vocabularies"]["enemy"][o.group("enemies").unwrap().values[i*12] as usize],e.enemy_id.as_str());}
        d.last_combat=Some(CombatRecord{visit:1,encounter:id,outcome:Outcome::Victory});assert!(encode_run_m20(&d,None,&caps).is_ok());
    }
    let mut d=diagnostic_run().decision().unwrap();
    for terminal in [RunTerminal::Death,RunTerminal::EmptyDeck,RunTerminal::Act3NoKeys,RunTerminal::HeartVictory] {
        d.terminal=Some(terminal);d.candidates.clear();assert!(encode_run_m20(&d,None,&caps).is_ok());
    }
}

#[test]
fn m20_candidate_order_masks_and_targets_are_shared_row_references() {
    use spire_core::combat::*;
    let caps=RunCapacities::m10_default();let (d,map)=combat_diagnostic();let o=encode_run_m20(&d,map.as_ref(),&caps).unwrap();
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
    let mut reordered=d.clone();reordered.candidates.reverse();let reversed=encode_run_m20(&reordered,map.as_ref(),&caps).unwrap();
    for i in 0..d.candidates.len() {assert_eq!(&reversed.group("actions").unwrap().values[i*8..i*8+8],&actions.values[(d.candidates.len()-i-1)*8..(d.candidates.len()-i)*8]);}
    let mut extra=d.clone();extra.combat.as_mut().unwrap().character.orbs=vec![None;11];extra.combat.as_mut().unwrap().character.orb_dark_damage=vec![None;11];
    assert!(encode_run_m20(&extra,None,&RunCapacities{orbs:11,..caps.clone()}).is_err(),"transport capacity cannot expand gameplay slot cap10");
}


#[test]
fn m20_addition_keeps_frozen_predecessor_descriptors_and_rejects_m9_admission() {
    let caps=RunCapacities::m10_default();
    for (spec,expected) in [
        (run_observation_spec(&caps).unwrap(),"0893bce86e4b3abf11aa384c501503a91ae37f688ec605ab40b814ad365cb999"),
        (m10_run_observation_spec(&caps).unwrap(),"7664874a7a483d1f2e0439ea7b319d1a4be4b9d276d0cb0f908654feda9393db"),
        (m12_run_observation_spec(&caps).unwrap(),"366c572af8141146995e849acc737a33484623790e2b3d661e290fadfb6c9fac"),
        (m13_run_observation_spec(&caps).unwrap(),"3f1e21d50106b8ca71d7cd67a2de7bfa4074455dbbb459629d8e2ac24cf39cf8"),
        (m17_run_observation_spec(&caps).unwrap(),"2176f835eca1384b2070f3d7d001158ef1fead23be5d6d7eea138b57f6bff999"),
        (m18_run_observation_spec(&caps).unwrap(),"e097a8202f7cf7433f0ce57a39807bb74d370fba75a9aa84af65d726269564bb"),
    ] {assert_eq!(spec.fingerprint().iter().map(|b|format!("{b:02x}")).collect::<String>(),expected);}
    let d=diagnostic_run().decision().unwrap();assert_eq!(encode_run(&d,None,&caps),Err(RunSessionError::Input("M20 profile is not representable in the M9 v1 observation schema")));
    println!("M20_DESCRIPTOR_SHA256 {}",m20_run_observation_spec(&caps).unwrap().fingerprint().iter().map(|b|format!("{b:02x}")).collect::<String>());
}


#[test]
fn m20_hidden_draw_order_and_rng_do_not_change_entitled_projection(){
    let caps=RunCapacities::m10_default();let cfg=default_config();
    let a=diagnostic_run_for_config(&cfg,7).decision().unwrap();let b=diagnostic_run_for_config(&cfg,11).decision().unwrap();
    assert_eq!(encode_run_m20(&a,None,&caps),encode_run_m20(&b,None,&caps));
    let mut cards=(1..=12).map(|i|permanent_live_card(if i%2==0{CardId::StrikeBlue}else{CardId::DefendBlue},CardInstanceId(i),0)).collect::<Vec<_>>();
    let objects=cards.iter().map(|c|PublicObject{instance:c.instance.0,id:c.id,upgrades:0,misc:0}).collect::<Vec<_>>();
    let a=supplied_combat_with_opening(EncounterId::SphericGuardian,cards.clone(),0);cards.reverse();
    let b=supplied_combat_with_opening(EncounterId::SphericGuardian,cards,0);
    let a=project_combat(&a,objects.clone());let b=project_combat(&b,objects);
    assert!(a.combat.as_ref().unwrap().hand.is_empty());assert_eq!(a.combat.as_ref().unwrap().draw_count,12);
    let o=encode_run_m20(&a,None,&caps).unwrap();assert_eq!(o,encode_run_m20(&b,None,&caps).unwrap());
    assert!(o.group("combat_cards").unwrap().occupied.as_ref().unwrap().iter().all(|x|*x==0));
    assert!(o.group("known_draw").unwrap().occupied.as_ref().unwrap().iter().all(|x|*x==0));
}

#[test]
fn m20_public_delayed_power_rows_preserve_each_timer_and_named_capacity(){
    let caps=RunCapacities::m10_default();let (mut d,_)=combat_diagnostic();
    let c=d.combat.as_mut().unwrap();c.player.powers=vec![
        FullPowerView{id:FullPowerId("panache_countdown"),amount:2},FullPowerView{id:FullPowerId("the_bomb"),amount:1},
        FullPowerView{id:FullPowerId("the_bomb_plus"),amount:3},FullPowerView{id:FullPowerId("the_bomb"),amount:2},FullPowerView{id:FullPowerId("focus"),amount:-999},
    ];
    let required=c.player.powers.len()+c.enemies.iter().map(|e|e.powers.len()).sum::<usize>();
    let o=encode_run_m20(&d,None,&RunCapacities{powers:required,..caps.clone()}).unwrap();let rows=o.group("powers").unwrap();
    assert_eq!(rows.values.chunks_exact(4).take(5).map(|r|r[3]).collect::<Vec<_>>(),vec![2,1,3,2,-999]);
    assert_eq!(rows.values[6],rows.values[14],"independent equal-kind Bomb timers retain separate rows");
    assert!(matches!(encode_run_m20(&d,None,&RunCapacities{powers:required-1,..caps}),Err(RunSessionError::Capacity{group:"powers",required:r,capacity:c}) if r==required && c==required-1));
}

#[test]
fn m20_selection_rows_append_operations_and_old_encoders_reject_before_dispatch(){
    let caps=RunCapacities::m10_default();let (d,_)=combat_diagnostic();
    let descriptor=m20_run_observation_spec(&caps).unwrap().descriptor();
    for (operation,name,zone,disposition) in [
        (SelectionOperation::HologramRetrieve,"hologram_retrieve",SelectionSourceZone::Discard,SelectionDisposition::Hand),
        (SelectionOperation::SeekRetrieve,"seek_retrieve",SelectionSourceZone::Draw,SelectionDisposition::Hand),
        (SelectionOperation::Recycle,"recycle",SelectionSourceZone::Hand,SelectionDisposition::Exhaust),
    ]{
        // Static public selection projection, not a second gameplay initializer.
        let mut one=d.clone();let card=one.combat.as_ref().unwrap().hand[0].clone();let instance=card.instance.0;
        one.combat.as_mut().unwrap().selection=Some(SelectionView{parent:SelectionParent::Card(CardHandle::parse("1").unwrap()),operation,candidates:vec![card],required:1,selected:vec![],disposition,source_zone:zone,minimum:1,maximum:1});
        one.candidates=vec![Candidate{id:ChoiceId(1),action:GameAction::Combat(CombatAction::SelectCard{instance}),label:"projection diagnostic",neow_offer:None,object:None}];
        let encoded=encode_run_m20(&one,None,&caps).unwrap();let op=encoded.group("selection_counts").unwrap().values[5] as usize;
        assert_eq!(descriptor["vocabularies"]["selection_operation"][op],name);
        let mut old=one.clone();old.content_profile=Some(RoomScenario::M15SilentHeart);old.player.character=Character::Silent;
        assert_eq!(encode_run_m15(&old,None,&caps),Err(RunSessionError::UnsupportedBoundary("M20 selection requires M20 native profile")));
    }
}

#[test]
fn m20_foreign_player_powers_reject_without_removing_shared_enemy_vocabulary(){
    let caps=RunCapacities::m10_default();let (d,_)=combat_diagnostic();
    for name in ["wraith_form","after_image","barricade","corruption","poison"]{
        let mut bad=d.clone();bad.combat.as_mut().unwrap().player.powers.push(FullPowerView{id:FullPowerId(name),amount:1});
        assert_eq!(encode_run_m20(&bad,None,&caps),Err(RunSessionError::Input("unsupported M20 player power")));
    }
    let descriptor=m20_run_observation_spec(&caps).unwrap().descriptor();
    assert!(descriptor["vocabularies"]["power"].as_array().unwrap().iter().any(|x|x=="barricade"),"Spheric Guardian retains its shared enemy power");
}
