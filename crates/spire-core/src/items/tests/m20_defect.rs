use super::super::*;
#[test]
fn m20_defect_items_profile_is_additive_not_silent_identity(){
    let old=CombatItemContext::default();assert!(!old.m15_profile());assert!(!old.m20_profile());assert!(!old.shared_generated_profile());
    let blue=old.clone().with_m20_profile();assert!(blue.m20_profile());assert!(!blue.m15_profile());assert!(blue.shared_generated_profile());
    let silent=old.with_m15_profile();assert!(silent.m15_profile());assert!(!silent.m20_profile());assert!(silent.shared_generated_profile());
}

use crate::{combat::{Actor,EffectKind,CombatAction},content::{Character,PotionId,RelicId},run::*};
fn blue_items(relics:Vec<RelicId>,potions:Vec<Option<PotionId>>)->CombatItemContext{
    CombatItemContext::new(relics,potions).with_m20_profile()
}
fn blue_run()->Run{
    let mut profile=CreationProfile::experienced();profile.neow_eligible=false;
    Run::new_room_scenario(RunConfig{character:Character::Defect,ascension:0},profile,
        RunRng::new([7;32],crate::combat::rng::Partition::Test,0),RoomScenario::M20DefectHeart).unwrap()
}
#[test]
fn m20_defect_items_shared_potion_preparation_uses_blue_capability_without_silent_fallback(){
    let targets=[crate::combat::EnemyHandle::parse("enemy-1").unwrap()];let facts=super::combat_potion_facts(&targets);
    for potion in [PotionId::FlexPotion,PotionId::RegenPotion,PotionId::BlessingOfTheForge,PotionId::SneckoOil,PotionId::AttackPotion,PotionId::ColorlessPotion,PotionId::EntropicBrew,PotionId::SmokeBomb]{
        let items=blue_items(vec![],vec![Some(potion),None,None]);
        let prepared=potion_use_effects_for_character(&items,0,None,&facts,Character::Defect).unwrap_or_else(|e|panic!("shared actual Defect {potion:?}: {e:?}"));
        assert_eq!(prepared.potions,vec![None;3]);assert!(!prepared.effects.is_empty());
        assert_eq!(items.potions(),&[Some(potion),None,None],"pure preparation does not consume the live owner");
        let old=CombatItemContext::new(vec![],vec![Some(potion),None,None]);
        assert!(potion_use_effects_for_character(&old,0,None,&facts,Character::Defect).is_err());
    }
    for potion in [PotionId::PoisonPotion,PotionId::CunningPotion,PotionId::GhostInAJar]{
        let items=blue_items(vec![],vec![Some(potion),None,None]);
        assert!(potion_use_effects_for_character(&items,0,if potion==PotionId::PoisonPotion{Some(targets[0].clone())}else{None},&facts,Character::Defect).is_err());
    }
}
#[test]
fn m20_defect_items_fairy_rescue_consumes_one_live_slot_with_bark_and_bloom(){
    for (relics,want) in [(vec![],Some((0,15))),(vec![RelicId::SacredBark],Some((0,30))),(vec![RelicId::MarkOfBloom],None)]{
        let mut items=blue_items(relics,vec![Some(PotionId::FairyPotion),Some(PotionId::FairyPotion),None]);
        assert_eq!(items.consume_fairy_for_rescue(50),want);
        assert_eq!(items.potions()[0],if want.is_some(){None}else{Some(PotionId::FairyPotion)});assert_eq!(items.potions()[1],Some(PotionId::FairyPotion));
    }
}
#[test]
fn m20_defect_items_frozen_replaces_named_starter_slot_once_without_rebuilding_inventory(){
    let mut run=blue_run();let state=run.state_mut_for_test();state.relics.clear();
    for id in [RelicId::Mango,RelicId::CrackedCore,RelicId::Anchor,RelicId::NeowsLament]{acquire_relic(state,id).unwrap();}
    // A stale starter-only counter is removed; another actually owned counter survives.
    state.relic_counters.insert(RelicId::CrackedCore,2);
    acquire_relic(state,RelicId::FrozenCore).unwrap();
    assert_eq!(state.relics,vec![RelicId::Mango,RelicId::FrozenCore,RelicId::Anchor,RelicId::NeowsLament]);
    assert!(!state.relic_counters.contains_key(&RelicId::CrackedCore));assert_eq!(state.relic_counters[&RelicId::NeowsLament],3);
    let before=state.clone();assert!(acquire_relic(state,RelicId::FrozenCore).is_err());assert_eq!(*state,before);
    let mut absent=blue_run();absent.state_mut_for_test().relics.clear();let before=absent.state().clone();
    assert!(acquire_relic(absent.state_mut_for_test(),RelicId::FrozenCore).is_err());assert_eq!(*absent.state(),before);
}
#[test]
fn m20_defect_items_new_relics_require_blue_profile_and_actual_character(){
    for id in [RelicId::SymbioticVirus,RelicId::FrozenCore,RelicId::Inserter,RelicId::NuclearBattery,RelicId::RunicCapacitor]{
        let items=blue_items(vec![id],vec![None;3]);assert_eq!(validate_combat_inventory(&items,Character::Defect),Ok(()),"new M20 item {id:?}");
        assert!(validate_combat_inventory(&items,Character::Silent).is_err());
        assert!(validate_combat_inventory(&CombatItemContext::new(vec![id],vec![None;3]),Character::Defect).is_err());
    }
}
#[test]
fn m20_defect_items_essence_is_a_real_admitted_potion_before_live_dispatch(){
    let mut owner=Some(blue_items(vec![],vec![Some(PotionId::EssenceOfDarkness),None,None]));
    let mut c=crate::combat::Combat::enter_with_item_context(super::full_entry(Character::Defect,1,10),&mut owner).unwrap();
    c.step_full(&CombatAction::UsePotion{slot:0,target:None}).unwrap();
    let p=c.public_full().unwrap();assert_eq!(p.character.orbs,vec![Some(crate::combat::OrbKind::Dark);3]);
    assert_eq!(p.character.orb_dark_damage,vec![Some(6);3]);assert_eq!(c.item_context().unwrap().potions(),&[None;3]);
}

#[test]
fn m20_defect_potion_factory_rarity_boundaries_and_rejections_keep_full_proposals(){
    for (roll,proposal,want) in [(0,0,PotionId::FocusPotion),(64,3,PotionId::BlockPotion),(65,1,PotionId::PotionOfCapacity),(89,1,PotionId::PotionOfCapacity),(90,2,PotionId::EssenceOfDarkness),(99,28,PotionId::FruitJuice)]{
        let (actual,bounds,used)=potion_generation::sample_scripted(Character::Defect,false,&[roll,proposal]);assert_eq!(actual,want);assert_eq!(bounds,vec![100,33]);assert_eq!(used,2);
    }
    let (actual,bounds,used)=potion_generation::sample_scripted(Character::Defect,true,&[90,0,28,3,2]);
    assert_eq!(actual,PotionId::EssenceOfDarkness);assert_eq!(bounds,vec![100,33,33,33,33]);assert_eq!(used,5);
    let mut draws=vec![95,0];draws.extend(std::iter::repeat_n(3,130));draws.push(2);
    let (actual,bounds,used)=potion_generation::sample_scripted(Character::Defect,true,&draws);assert_eq!(actual,PotionId::EssenceOfDarkness);assert_eq!(used,133);assert_eq!(bounds.len(),133);
    for id in [PotionId::PoisonPotion,PotionId::CunningPotion,PotionId::GhostInAJar]{assert!(!potion_generation::candidates(Character::Defect,false).unwrap().contains(&id));}
}
#[test]
fn m20_defect_items_opening_first_turn_relics_use_distinct_real_phases(){
    let relics=vec![RelicId::CrackedCore,RelicId::SymbioticVirus,RelicId::NuclearBattery,RelicId::RunicCapacitor];
    let mut owner=Some(blue_items(relics,vec![None;3]));
    let mut c=crate::combat::Combat::enter_with_item_context(super::full_entry(Character::Defect,1,10),&mut owner).unwrap();
    assert_eq!(c.public_full().unwrap().character.orbs,vec![Some(crate::combat::OrbKind::Dark),Some(crate::combat::OrbKind::Plasma),Some(crate::combat::OrbKind::Lightning),None,None,None]);
    assert!(owner.is_none());
    c.step_full(&CombatAction::EndTurn).unwrap();
    assert_eq!(c.public_full().unwrap().character.orbs.len(),6,"Runic Capacitor is one first-turn callback, not each turn");
}
#[test]
fn m20_defect_items_inserter_persistent_counter_uses_each_real_first_turn_even_at_slot_cap(){
    for (initial,want_counter,want_slots) in [(-1,1,3),(0,1,3),(1,0,4)]{
        let counters=std::collections::BTreeMap::from([(RelicId::Inserter,initial)]);
        let mut owner=Some(blue_items(vec![RelicId::Inserter],vec![None;3]).with_counters(counters));
        let mut c=crate::combat::Combat::enter_with_item_context(super::full_entry(Character::Defect,1,10),&mut owner).unwrap();
        assert_eq!((c.item_context().unwrap().counter(RelicId::Inserter),c.public_full().unwrap().character.orbs.len()),(want_counter,want_slots));
        if initial==0{
            set_slots(&mut c,10);
            c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(c.public_full().unwrap().character.orbs.len(),10);assert_eq!(c.item_context().unwrap().counter(RelicId::Inserter),0,"counter resets even when the ordinary capacity action caps");
        }
    }
    for invalid in [-2,2,99]{
        let mut owner=Some(blue_items(vec![RelicId::Inserter],vec![None;3]).with_counters(std::collections::BTreeMap::from([(RelicId::Inserter,invalid)])));
        let before=owner.clone();assert!(crate::combat::Combat::enter_with_item_context(super::full_entry(Character::Defect,1,10),&mut owner).is_err());assert_eq!(owner,before);
    }
}
#[test]
fn m20_defect_items_callback_primitive_keeps_prebattle_turn_and_end_work_separate(){
    use crate::combat::{CharacterEffect as C,OrbKind as O};
    let mut items=blue_items(vec![RelicId::CrackedCore,RelicId::SymbioticVirus,RelicId::NuclearBattery,RelicId::RunicCapacitor,RelicId::Inserter],vec![None;3]);
    // Primitive callback facts; this intentionally does not claim joined engine execution.
    let pre=defect_pre_battle_effects(&mut items).unwrap();
    assert_eq!(pre.iter().map(|e|e.kind.clone()).collect::<Vec<_>>(),vec![EffectKind::Character(C::Channel(O::Dark)),EffectKind::Character(C::Channel(O::Plasma))]);
    assert!(pre.iter().all(|e|e.source==crate::combat::Source::None&&e.target==Some(Actor::Player)));
    let first=defect_turn_start_effects(&mut items).unwrap();assert_eq!(first.iter().map(|e|e.kind.clone()).collect::<Vec<_>>(),vec![EffectKind::Character(C::Slots(3))]);assert_eq!(items.counter(RelicId::Inserter),1);
    let second=defect_turn_start_effects(&mut items).unwrap();assert_eq!(second.iter().map(|e|e.kind.clone()).collect::<Vec<_>>(),vec![EffectKind::Character(C::Slots(1))]);assert_eq!(items.counter(RelicId::Inserter),0);
    let frozen=blue_items(vec![RelicId::FrozenCore],vec![None;3]);
    assert!(defect_end_turn_work(&frozen,false).is_empty());assert_eq!(defect_end_turn_work(&frozen,true).iter().map(|e|e.kind.clone()).collect::<Vec<_>>(),vec![EffectKind::Character(C::Channel(O::Frost))]);
    let mut old=CombatItemContext::new(vec![RelicId::RunicCapacitor,RelicId::NuclearBattery,RelicId::FrozenCore],vec![None;3]);
    assert!(defect_pre_battle_effects(&mut old).unwrap().is_empty());assert!(defect_turn_start_effects(&mut old).unwrap().is_empty());assert!(defect_end_turn_work(&old,true).is_empty());
}
#[test]
fn m20_defect_items_callback_primitive_rejects_malformed_counter_before_advancing_any_relic(){
    let mut items=blue_items(vec![RelicId::RunicCapacitor,RelicId::Inserter],vec![None;3]);
    defect_pre_battle_effects(&mut items).unwrap();items.set_counter(RelicId::Inserter,99);let before=items.clone();
    assert!(defect_turn_start_effects(&mut items).is_err());assert_eq!(items,before);
    let mut run=blue_run();acquire_relic(run.state_mut_for_test(),RelicId::Inserter).unwrap();
    assert_eq!(visible_counters(&run.state().relic_counters),vec![RelicCounterView{relic:RelicId::Inserter,value:0}]);
}
#[test]
fn m20_defect_potion_factory_outside_brew_uses_live_belt_and_sozu_before_rng(){
    for (slots,belt,sozu) in [(3,false,false),(5,true,false),(3,false,true),(5,true,true)]{
        let mut run=blue_run();let s=run.state_mut_for_test();s.hp=30;s.max_hp=50;s.relics.push(RelicId::ToyOrnithopter);
        if belt{s.relics.push(RelicId::PotionBelt);}if sozu{s.relics.push(RelicId::Sozu);}
        s.potions=vec![None;slots];s.potions[slots-1]=Some(PotionId::EntropicBrew);let rng=s.rng.clone();
        assert!(can_use_run_potion(s,slots-1,false,false),"M20 outside Brew must be offered on live belt");
        use_run_potion(s,slots-1,false,false).unwrap();assert_eq!(s.hp,35,"one use callback regardless of generated count/Sozu");
        if sozu{assert!(s.potions.iter().all(Option::is_none));assert_eq!(s.rng,rng);}else{
            assert!(s.potions.iter().all(Option::is_some));assert_ne!(s.rng,rng);
            assert!(s.potions.iter().flatten().all(|id|!matches!(id,PotionId::PoisonPotion|PotionId::CunningPotion|PotionId::GhostInAJar)));
        }
    }
}
#[test]
fn m20_defect_potion_factory_accepted_obtain_retains_prefix_and_first_fault_even_with_sozu(){
    let mut run=blue_run();let s=run.state_mut_for_test();s.potions=vec![None;3];
    assert_eq!(apply_accepted_run_potion_obtain(s,PotionId::EssenceOfDarkness),Ok(true));s.relics.push(RelicId::Sozu);
    assert_eq!(apply_accepted_run_potion_obtain(s,PotionId::FocusPotion),Ok(false));
    let Err(RunError::Faulted(first))=apply_accepted_run_potion_obtain(s,PotionId::Unknown("bad"))else{panic!("accepted invalid obtain must fault before Sozu no-op")};
    let before=s.clone();assert_eq!(apply_accepted_run_potion_obtain(s,PotionId::WeakPotion),Err(RunError::Faulted(first)));assert_eq!(*s,before);assert_eq!(s.potions,vec![Some(PotionId::EssenceOfDarkness),None,None]);
}
#[test]
fn m20_defect_potion_factory_all33_have_real_use_or_passive_rescue_without_green_outputs(){
    let mut failures=vec![];
    for id in potion_generation::candidates(Character::Defect,false).unwrap(){
        let mut owner=Some(blue_items(vec![],vec![Some(*id),None,None]));
        let mut c=crate::combat::Combat::enter_with_item_context(super::full_entry(Character::Defect,1,10),&mut owner).unwrap();
        if *id==PotionId::FairyPotion{
            assert!(c.legal_actions_full().unwrap().iter().all(|a|!matches!(a,CombatAction::UsePotion{slot:0,..})));
            let mut items=c.item_context().unwrap().clone();assert_eq!(items.consume_fairy_for_rescue(50),Some((0,15)));continue;
        }
        let target=matches!(id,PotionId::FirePotion|PotionId::WeakPotion|PotionId::FearPotion).then_some(crate::combat::TargetId(1));
        match c.step_full(&CombatAction::UsePotion{slot:0,target}){
            Ok(_)=>{if *id==PotionId::EntropicBrew{assert!(c.item_context().unwrap().potions().iter().all(Option::is_some),"Brew may legitimately obtain another Brew in its vacated slot");}else{assert_eq!(c.item_context().unwrap().potions()[0],None,"accepted source vacated once: {id:?}");}
                if let Some(offer)=c.public_full().unwrap().card_offer{assert_eq!(offer.cards.len(),3);assert!(offer.cards.iter().all(|id|!matches!(id,crate::content::CardId::Shiv|crate::content::CardId::StrikeGreen|crate::content::CardId::PoisonedStab)));}
            },
            Err(e)=>failures.push(format!("{id:?}: {e:?}")),
        }
    }
    assert!(failures.is_empty(),"actual M20 shared source joins unresolved: {failures:?}");
}
fn durable_blue_entry()->crate::combat::CombatEntry{
    let mut e=super::full_entry(Character::Defect,1,10);e.setup.enemies[0].hp=500;e.setup.enemies[0].max_hp=500;e
}
// Literal supplied unit setup at the core test owner, before the real potion
// action. It does not impersonate a card/potion/relic Slots request.
fn set_slots(c:&mut crate::combat::Combat,slots:i32){
    crate::combat::tests::m16_support::m20_item_set_orbs(c,&vec![None;slots as usize]);
}
fn fill_orbs(c:&mut crate::combat::Combat,count:usize,orb:crate::combat::OrbKind){
    c.enqueue_native_effects((0..count).map(|_|crate::combat::Effect{source:crate::combat::Source::None,target:Some(Actor::Player),kind:EffectKind::Character(crate::combat::CharacterEffect::Channel(orb))}).collect(),None).unwrap();
}
#[test]
fn m20_defect_items_essence_reads_live_capacity_and_ordinary_evictions_for_sparse_and_full_slots(){
    use crate::combat::OrbKind;
    for (slots,filled,bark,want_hp) in [(0,0,false,500),(0,0,true,500),(1,1,false,492),(1,1,true,486),(3,0,false,500),(3,0,true,482),(3,1,false,492),(3,1,true,474),(3,3,false,476),(3,3,true,458),(10,0,false,500),(10,0,true,440),(10,5,false,460),(10,5,true,400),(10,10,false,420),(10,10,true,360)]{
        let relics=if bark{vec![RelicId::SacredBark]}else{vec![]};let mut owner=Some(blue_items(relics,vec![Some(PotionId::EssenceOfDarkness),None,None]));
        let mut c=crate::combat::Combat::enter_with_item_context(durable_blue_entry(),&mut owner).unwrap();set_slots(&mut c,slots);fill_orbs(&mut c,filled,OrbKind::Lightning);
        c.step_full(&CombatAction::UsePotion{slot:0,target:None}).unwrap();let p=c.public_full().unwrap();
        assert_eq!(p.character.orbs,vec![Some(OrbKind::Dark);slots as usize],"live {slots} filled{filled} Bark{bark}");
        assert_eq!(p.character.orb_dark_damage,vec![Some(6);slots as usize]);assert_eq!(p.enemies[0].hp,want_hp,"each ordinary old orb eviction executes; no vector replacement");assert_eq!(c.item_context().unwrap().potions(),&[None;3]);
    }
}
#[test]
fn m20_defect_items_essence_fault_after_first_eviction_keeps_consumption_and_channel_prefix(){
    use crate::combat::{OrbKind,rng::StreamState};
    let mut e=durable_blue_entry();let r=&e.setup.card_rng;e.setup.card_rng=StreamState::new(r.id().clone(),*r.key(),(1u128<<64)-1).unwrap();
    let mut owner=Some(blue_items(vec![],vec![Some(PotionId::EssenceOfDarkness),None,None]));
    let mut c=crate::combat::Combat::enter_with_item_context(e,&mut owner).unwrap();fill_orbs(&mut c,3,OrbKind::Lightning);
    let error=c.step_full(&CombatAction::UsePotion{slot:0,target:None}).unwrap_err();assert!(matches!(error,RunError::Faulted(_)));
    assert_eq!(c.item_context().unwrap().potions(),&[None;3]);let facts=crate::combat::tests::m16_support::m20_item_fault_facts(&c);assert_eq!(facts,(492,1));
    let before=c.clone();assert_eq!(c.step_full(&CombatAction::UsePotion{slot:0,target:None}),Err(error));assert_eq!(c,before);
}
#[test]
fn m20_defect_items_frozen_callback_precedes_real_natural_passives_and_cables(){
    use crate::combat::{FullIntentId,OrbKind};
    for (slots,filled,want_hp,want_frost) in [(0,0,34,0),(1,0,38,1),(3,0,38,1),(3,3,40,3)]{
        let mut e=durable_blue_entry();e.setup.enemies[0].intent=FullIntentId::Attack;
        let mut owner=Some(blue_items(vec![RelicId::FrozenCore,RelicId::GoldPlatedCables],vec![None;3]));
        let mut c=crate::combat::Combat::enter_with_item_context(e,&mut owner).unwrap();set_slots(&mut c,slots);fill_orbs(&mut c,filled,OrbKind::Frost);
        c.step_full(&CombatAction::EndTurn).unwrap();let p=c.public_full().unwrap();assert_eq!(p.player.hp,want_hp,"new Frost must join natural passives before Cables, capacity{slots}/filled{filled}");
        assert_eq!(p.character.orbs.iter().filter(|orb|**orb==Some(OrbKind::Frost)).count(),want_frost);
    }
}
#[test]
fn m20_defect_potion_factory_full_domains_have_source_rarities_and_each_proposal_is_reachable(){
    use crate::content::{Catalog,PotionRarity};use std::collections::HashSet;
    let full=potion_generation::candidates(Character::Defect,false).unwrap();let limited=potion_generation::candidates(Character::Defect,true).unwrap();
    assert_eq!(full.len(),33);assert_eq!(full.iter().collect::<HashSet<_>>().len(),33);assert_eq!(limited.len(),32);assert!(!limited.contains(&PotionId::FruitJuice));
    for (rarity,count,limited_count) in [(PotionRarity::Common,17,17),(PotionRarity::Uncommon,9,9),(PotionRarity::Rare,7,6)]{
        assert_eq!(full.iter().filter(|id|Catalog::standard().potion_rarity(**id)==Some(rarity)).count(),count);
        assert_eq!(limited.iter().filter(|id|Catalog::standard().potion_rarity(**id)==Some(rarity)).count(),limited_count);
    }
    for (index,id) in full.iter().enumerate(){
        let roll=match Catalog::standard().potion_rarity(*id).unwrap(){PotionRarity::Common=>0,PotionRarity::Uncommon=>65,PotionRarity::Rare=>90};
        let (actual,bounds,used)=potion_generation::sample_scripted(Character::Defect,false,&[roll,index]);assert_eq!(actual,*id);assert_eq!(used,2);assert_eq!(bounds,vec![100,33]);
    }
}
#[test]
fn m20_defect_items_existing_focus_capacity_use_base_bark_and_real_slot_cap(){
    for (bark,want_focus,want_slots) in [(false,2,5),(true,4,7)]{
        let mut owner=Some(blue_items(if bark{vec![RelicId::SacredBark]}else{vec![]},vec![Some(PotionId::FocusPotion),Some(PotionId::PotionOfCapacity),None]));
        let mut c=crate::combat::Combat::enter_with_item_context(durable_blue_entry(),&mut owner).unwrap();
        c.step_full(&CombatAction::UsePotion{slot:0,target:None}).unwrap();c.step_full(&CombatAction::UsePotion{slot:1,target:None}).unwrap();
        let p=c.public_full().unwrap();assert_eq!(p.character.focus,want_focus);assert_eq!(p.character.orbs.len(),want_slots);assert_eq!(c.item_context().unwrap().potions(),&[None;3]);
        let mut owner=Some(blue_items(if bark{vec![RelicId::SacredBark]}else{vec![]},vec![Some(PotionId::PotionOfCapacity),None,None]));let mut capped=crate::combat::Combat::enter_with_item_context(durable_blue_entry(),&mut owner).unwrap();set_slots(&mut capped,10);
        capped.step_full(&CombatAction::UsePotion{slot:0,target:None}).unwrap();assert_eq!(capped.public_full().unwrap().character.orbs.len(),10);
    }
}
#[test]
fn m20_defect_items_existing_fairy_actual_damage_rescue_is_not_manual_potion_use(){
    use crate::combat::{Effect,Source,DamagePayload};
    for (bark,bloom,want_hp,want_consumed) in [(false,false,15,true),(true,false,30,true),(false,true,0,false)]{
        let mut relics=vec![RelicId::ToyOrnithopter];if bark{relics.push(RelicId::SacredBark);}if bloom{relics.push(RelicId::MarkOfBloom);}
        let mut owner=Some(blue_items(relics,vec![Some(PotionId::FairyPotion),None,None]));let mut c=crate::combat::Combat::enter_with_item_context(durable_blue_entry(),&mut owner).unwrap();
        c.enqueue_native_effects(vec![Effect{source:Source::None,target:Some(Actor::Player),kind:EffectKind::Damage(DamagePayload::Fixed(50))}],None).unwrap();
        let p=c.public_full().unwrap();assert_eq!(p.player.hp,want_hp,"rescue applies30/60percent and no manual Toy heal");assert_eq!(c.item_context().unwrap().potions()[0].is_none(),want_consumed);
        assert_eq!(p.outcome==Some(crate::combat::Outcome::Death),bloom);
    }
}

#[test]
fn m20_defect_potion_factory_combat_brew_uses_live_belt_sozu_and_one_toy_callback() {
    for (slots, sozu, bark) in [(3, false, false), (5, false, true), (3, true, true), (5, true, false)] {
        let mut relics = vec![RelicId::ToyOrnithopter];
        if slots == 5 { relics.push(RelicId::PotionBelt); }
        if sozu { relics.push(RelicId::Sozu); }
        if bark { relics.push(RelicId::SacredBark); }
        let mut potions = vec![None; slots];
        potions[slots - 1] = Some(PotionId::EntropicBrew);
        let mut owner = Some(blue_items(relics, potions));
        let mut c = crate::combat::Combat::enter_with_item_context(durable_blue_entry(), &mut owner).unwrap();
        let rng_before = crate::combat::tests::m16_support::m20_item_card_rng_counter(&c);
        c.step_full(&CombatAction::UsePotion { slot: slots - 1, target: None }).unwrap();
        assert_eq!(c.public_full().unwrap().player.hp, 45, "Toy heals once, including Sozu; Bark does not multiply generation or Toy");
        let belt = c.item_context().unwrap().potions();
        if sozu {
            assert!(belt.iter().all(Option::is_none));
            assert!(crate::combat::tests::m16_support::m20_item_card_rng_counter(&c) > rng_before, "combat Brew samples before individually Sozu-blocked obtains; outside Brew skips sampling");
        } else {
            assert!(belt.iter().all(Option::is_some));
            assert!(crate::combat::tests::m16_support::m20_item_card_rng_counter(&c) > rng_before);
            assert!(belt.iter().flatten().all(|id| potion_generation::candidates(Character::Defect, false).unwrap().contains(id)));
        }
    }
}

#[test]
fn m20_defect_items_shared_power_potions_commit_literal_base_and_bark_amounts() {
    for (potion, powers) in [
        (PotionId::FlexPotion, vec![("strength", 5), ("lose_strength", 5)]),
        (PotionId::SpeedPotion, vec![("dexterity", 5), ("lose_dexterity", 5)]),
        (PotionId::RegenPotion, vec![("regeneration", 5)]),
        (PotionId::LiquidBronze, vec![("thorns", 3)]),
        (PotionId::EssenceOfSteel, vec![("plated_armor", 4)]),
        (PotionId::CultistPotion, vec![("ritual", 1)]),
    ] {
        for factor in [1, 2] {
            let relics = if factor == 2 { vec![RelicId::SacredBark] } else { vec![] };
            let mut owner = Some(blue_items(relics, vec![Some(potion), None, None]));
            let mut c = crate::combat::Combat::enter_with_item_context(durable_blue_entry(), &mut owner).unwrap();
            c.step_full(&CombatAction::UsePotion { slot: 0, target: None }).unwrap();
            let p = c.public_full().unwrap();
            for (id, base) in &powers {
                assert_eq!(p.player.powers.iter().find(|power| power.id.0 == *id).map(|power| power.amount), Some(base * factor), "{potion:?} Bark factor {factor}");
            }
        }
    }
}

#[test]
fn m20_defect_items_typed_potion_offers_keep_blue_pools_and_bark_copies() {
    use crate::content::{CardColor, CardType, Catalog};
    for (potion, card_type) in [(PotionId::AttackPotion, CardType::Attack), (PotionId::SkillPotion, CardType::Skill), (PotionId::PowerPotion, CardType::Power)] {
        for copies in [1, 2] {
            let mut owner = Some(blue_items(if copies == 2 { vec![RelicId::SacredBark] } else { vec![] }, vec![Some(potion), None, None]));
            let mut c = crate::combat::Combat::enter_with_item_context(durable_blue_entry(), &mut owner).unwrap();
            c.step_full(&CombatAction::UsePotion { slot: 0, target: None }).unwrap();
            let offer = c.public_full().unwrap().card_offer.unwrap();
            assert_eq!(offer.cards.len(), 3);
            let hand_before = c.public_full().unwrap().hand.len();
            for id in &offer.cards { let definition = Catalog::standard().card(*id).unwrap(); assert_eq!(definition.color, CardColor::Blue); assert_eq!(definition.card_type, card_type); }
            c.step_full(&CombatAction::ChooseCardOffer { index: 0 }).unwrap();
            let delivered = c.public_full().unwrap();
            assert_eq!(delivered.hand.len(), hand_before + copies);
            assert_eq!(delivered.hand.iter().filter(|card| card.id == offer.cards[0]).count(), copies);
        }
    }
}

#[test]
fn m20_defect_items_invalid_potion_requests_preserve_live_owner_and_rng() {
    for id in [PotionId::EssenceOfDarkness, PotionId::EntropicBrew, PotionId::FocusPotion, PotionId::AttackPotion] {
        let mut owner = Some(blue_items(vec![], vec![Some(id), None, None]));
        let mut c = crate::combat::Combat::enter_with_item_context(durable_blue_entry(), &mut owner).unwrap();
        let before = c.clone();
        assert!(c.step_full(&CombatAction::UsePotion { slot: 0, target: Some(crate::combat::TargetId(1)) }).is_err());
        assert_eq!(c, before);
        assert!(c.step_full(&CombatAction::UsePotion { slot: 3, target: None }).is_err());
        assert_eq!(c, before);
    }
}

#[test]
fn m20_defect_items_new_acquisition_commits_once_and_rejects_foreign_character() {
    for id in [RelicId::SymbioticVirus, RelicId::Inserter, RelicId::NuclearBattery, RelicId::RunicCapacitor] {
        let mut run = blue_run();
        let s = run.state_mut_for_test();
        let prior = s.relics.clone();
        acquire_relic(s, id).unwrap();
        assert_eq!(&s.relics[..prior.len()], &prior);
        assert_eq!(s.relics.last(), Some(&id));
        let committed = s.clone();
        assert!(acquire_relic(s, id).is_err()); assert_eq!(*s, committed);
        let mut foreign = blue_run(); let s = foreign.state_mut_for_test(); s.character = Character::Silent;
        let before = s.clone(); assert!(acquire_relic(s, id).is_err()); assert_eq!(*s, before);
    }
}

#[test]
fn m20_defect_items_real_exit_returns_inserter_counter_but_next_entry_resets_slots_and_rearms_runic() {
    let mut run = blue_run();
    let s = run.state_mut_for_test();
    acquire_relic(s, RelicId::Inserter).unwrap();
    acquire_relic(s, RelicId::RunicCapacitor).unwrap();
    s.potions = vec![Some(PotionId::PotionOfCapacity), Some(PotionId::FirePotion), None];
    let items = CombatItemContext::take_from_run(s).with_m20_profile();
    assert!(s.relics.is_empty() && s.potions.is_empty() && s.relic_counters.is_empty());
    let mut entry = super::full_entry(Character::Defect, 1, 10);
    // Literal short item-lifecycle unit fixture, never a milestone caller run.
    entry.setup.enemies[0].hp = 10;
    let mut owner = Some(items);
    let mut c = crate::combat::Combat::enter_with_item_context(entry, &mut owner).unwrap();
    assert!(owner.is_none());
    assert_eq!(c.item_context().unwrap().counter(RelicId::Inserter), 1);
    c.step_full(&CombatAction::UsePotion { slot: 0, target: None }).unwrap();
    assert_eq!(c.public_full().unwrap().character.orbs.len(), 8);
    c.step_full(&CombatAction::UsePotion { slot: 1, target: Some(crate::combat::TargetId(1)) }).unwrap();
    let exit = c.exit().unwrap();
    drop(c);
    exit.item_context.unwrap().return_to_run(s).unwrap();
    assert_eq!(s.relic_counters[&RelicId::Inserter], 1);
    assert_eq!(s.potions, vec![None; 3]);
    let mut owner = Some(CombatItemContext::take_from_run(s).with_m20_profile());
    let next = crate::combat::Combat::enter_with_item_context(super::full_entry(Character::Defect, 1, 10), &mut owner).unwrap();
    assert_eq!(next.item_context().unwrap().counter(RelicId::Inserter), 0);
    let p = next.public_full().unwrap();
    assert_eq!(p.character.orbs.len(), 7, "fresh three slots + first-turn Runic3 + carried Inserter1; temporary Capacity2 did not persist");
    assert_eq!(p.character.orbs.iter().flatten().count(), 1, "Cracked Core channels once into freshly reset orb state");
    assert!(s.relics.is_empty() && s.potions.is_empty() && s.relic_counters.is_empty());
}
