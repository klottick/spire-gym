use super::super::*;
use crate::{combat::{CombatAction,rng::Partition},content::{Character,CardId,RelicId}};

fn profile()->CreationProfile{let mut p=CreationProfile::experienced();p.neow_eligible=false;p.seen_bosses[0]=vec!["Hexaghost","SlimeBoss"];p}
fn run()->Run{Run::new_room_scenario(RunConfig{character:Character::Silent,ascension:0},profile(),RunRng::new([7;32],Partition::Test,0),RoomScenario::M12SilentAct1).unwrap()}
fn choose(run:&mut Run,kind:RoomChoiceKind)->RunTransition{
    let room=run.decision().unwrap().room.unwrap();
    let row=room.choices.iter().find(|c|c.kind==kind&&c.status==RoomChoiceStatus::Offered)
        .unwrap_or_else(||panic!("expected offered {kind:?}, actual {room:?}"));
    run.step(&GameAction::Choose(row.id)).unwrap()
}

#[test]
fn m12_normal_start_selects_actual_short_neow_and_enters_shared_generated_map_combat(){
    let mut r=run();let initial=r.decision().unwrap();
    assert_eq!((initial.player.character,initial.player.hp,initial.player.max_hp,initial.player.gold),(Character::Silent,70,70,99));
    assert_eq!(initial.player.relics,vec![RelicId::RingOfTheSnake]);assert_eq!(initial.player.potions,vec![None;3]);assert_eq!(initial.player.keys,[false;3]);
    assert_eq!(initial.objects.iter().filter(|c|c.id==CardId::StrikeGreen).count(),5);assert_eq!(initial.objects.iter().filter(|c|c.id==CardId::DefendGreen).count(),5);assert_eq!(initial.objects.len(),12);
    assert!(r.state.selected_neow.is_none());assert!(r.public_map().is_none());
    choose(&mut r,RoomChoiceKind::NeowMaxHp{amount:7});assert_eq!((r.state.hp,r.state.max_hp),(77,77));assert!(r.state.selected_neow.is_some());
    let map=r.public_map().unwrap();assert_eq!(map.boss,Some(crate::content::EncounterId::Guardian));assert!(map.nodes.len()>50);
    let d=r.decision().unwrap();let a=d.candidates.iter().find(|c|matches!(c.action,GameAction::Move(_))).unwrap().action.clone();r.step(&a).unwrap();
    let d=r.decision().unwrap();let c=d.combat.as_ref().unwrap();assert_eq!(c.hand.len(),7);assert_eq!(d.player.relics,vec![RelicId::RingOfTheSnake]);assert!(r.state.relics.is_empty());assert!(r.state.potions.is_empty());
    assert!(d.candidates.iter().any(|c|matches!(c.action,GameAction::Combat(CombatAction::PlayCard{..}))));
    assert_eq!(d.content_profile,Some(RoomScenario::M12SilentAct1));
}

#[test]
fn m12_profile_validation_preserves_old_character_and_neow_boundaries(){
    for character in Character::ALL{for scenario in [RoomScenario::M4Act1Prefix,RoomScenario::M5Act1Captured,RoomScenario::M6Act2Prefix,RoomScenario::M7Act2Captured,RoomScenario::M8Act3Prefix,RoomScenario::M9IroncladAct3,RoomScenario::M10IroncladHeart,RoomScenario::M12SilentAct1]{
        let accepted=Run::new_room_scenario(RunConfig{character,ascension:0},profile(),RunRng::new([1;32],Partition::Test,0),scenario).is_ok();
        assert_eq!(accepted,character==if scenario==RoomScenario::M12SilentAct1{Character::Silent}else{Character::Ironclad});
    }}
    for ascension in [1,20]{assert!(Run::new_room_scenario(RunConfig{character:Character::Silent,ascension},profile(),RunRng::new([1;32],Partition::Test,0),RoomScenario::M12SilentAct1).is_err());}
    assert!(Run::new_room_scenario(RunConfig{character:Character::Silent,ascension:0},CreationProfile::experienced(),RunRng::new([1;32],Partition::Test,0),RoomScenario::M12SilentAct1).is_err());
}

#[test]
fn m12_finite_green_acquisition_and_upgrades_use_existing_permanent_owner(){
    let mut r=run();
    for &id in M12_SILENT_ACT1_CARDS{
        let instance=mutations::obtain_card(&mut r.state,id,0).unwrap();
        let before=r.state.deck.iter().find(|c|c.instance==instance).unwrap().clone();
        assert!(mutations::can_upgrade(&r.state,&before));
        mutations::upgrade_card(&mut r.state,instance).unwrap();
        let after=r.state.deck.iter().find(|c|c.instance==instance).unwrap();assert_eq!((after.instance,after.id,after.upgrades),(instance,id,1));
        assert!(!mutations::can_upgrade(&r.state,after));
    }
    for id in [CardId::Strike,CardId::IronWave,CardId::SearingBlow,CardId::Shiv]{let before=r.clone();assert!(mutations::obtain_card(&mut r.state,id,0).is_err());assert!(r==before);}
    let mut old=super::m5_act1::run();let before=old.clone();assert!(mutations::obtain_card(&mut old.state,CardId::DeadlyPoison,0).is_err());assert!(old==before);
}

fn diagnostic_room(r:&mut Run,kind:MapNodeKind){
    // A supplied valid predecessor boundary for focused room custody tests.
    // This is not normal-start traversal evidence; the public example supplies that.
    let map=r.state.map.as_mut().unwrap();
    let target=map.nodes.iter().find(|n|n.kind==kind&&!n.visited&&map.nodes.iter().any(|previous|previous.edges.contains(&n.id))).unwrap().id;
    let previous=map.nodes.iter_mut().find(|n|n.edges.contains(&target)).unwrap();
    previous.visited=true;r.state.floor=previous.floor;map.current=Some(previous.id);
    r.step(&GameAction::Move(target)).unwrap();
}
fn resolve_lament_fight(r:&mut Run){
    for _ in 0..128{
        let d=r.decision().unwrap();if d.combat.is_none(){return}
        let action=d.candidates.iter().find(|c|matches!(c.action,GameAction::Combat(CombatAction::PlayCard{target:Some(_),..})))
            .map(|c|c.action.clone()).unwrap_or(GameAction::Combat(CombatAction::EndTurn));
        r.step(&action).unwrap();
    }panic!("bounded real Lament combat did not settle")
}
fn boss_chest(boss:crate::content::EncounterId)->Run{
    use crate::content::EncounterId::*;
    let mut p=profile();p.seen_bosses[0]=match boss{Guardian=>vec!["Hexaghost","SlimeBoss"],Hexaghost=>vec!["Guardian","SlimeBoss"],SlimeBoss=>vec!["Guardian","Hexaghost"],_=>unreachable!()};
    let mut r=Run::new_room_scenario(RunConfig{character:Character::Silent,ascension:0},p,RunRng::new([7;32],Partition::Test,0),RoomScenario::M12SilentAct1).unwrap();
    choose(&mut r,RoomChoiceKind::NeowLament);diagnostic_room(&mut r,MapNodeKind::Boss);resolve_lament_fight(&mut r);
    assert_eq!(r.state.last_combat.as_ref().unwrap().encounter,boss);
    choose(&mut r,RoomChoiceKind::CardReward);
    let rows=r.decision().unwrap().room.unwrap().choices;
    assert_eq!(rows.iter().filter_map(|c|match c.kind{RoomChoiceKind::Card{id,..}=>Some(id),_=>None}).collect::<Vec<_>>(),vec![CardId::Nightmare]);
    choose(&mut r,RoomChoiceKind::Card{id:CardId::Nightmare,upgrades:0});
    choose(&mut r,RoomChoiceKind::Proceed);assert_eq!((r.state.act,r.state.floor),(1,17));choose(&mut r,RoomChoiceKind::OpenChest);r
}

fn next_real_reward(r:&mut Run){
    for _ in 0..128{
        let d=r.decision().unwrap();
        if d.combat.is_some(){resolve_lament_fight(r);continue}
        let room=d.room.as_ref().unwrap();
        if room.kind==RoomViewKind::Rewards{return}
        let action=if room.kind==RoomViewKind::Map{
            d.candidates.iter().find(|c|matches!(c.action,GameAction::Move(_))).unwrap().action.clone()
        }else{
            let row=room.choices.iter().find(|c|c.status==RoomChoiceStatus::Offered&&matches!(c.kind,RoomChoiceKind::Leave|RoomChoiceKind::Proceed)).unwrap();
            GameAction::Choose(row.id)
        };
        r.step(&action).unwrap();
    }panic!("three actual reward captures not reached within bounded legal route")
}

#[test]
fn m12_same_topology_encounters_and_first_three_real_reward_rows_keep_green_custody(){
    let mut r=run();let mut old=Run::new_room_scenario(RunConfig{character:Character::Ironclad,ascension:0},profile(),RunRng::new([7;32],Partition::Test,0),RoomScenario::M5Act1Captured).unwrap();
    choose(&mut r,RoomChoiceKind::NeowLament);choose(&mut old,RoomChoiceKind::NeowLament);assert_eq!(r.public_map(),old.public_map());
    let expected=[[CardId::PoisonedStab,CardId::Backflip,CardId::BladeDance],[CardId::DeadlyPoison,CardId::CloakAndDagger,CardId::DaggerThrow],[CardId::NoxiousFumes,CardId::Accuracy,CardId::WellLaidPlans]];
    for row in expected{
        next_real_reward(&mut r);
        let settled=r.state.combat_progress;assert_eq!(settled.entered,settled.settled);
        let originals=r.state.deck.clone();choose(&mut r,RoomChoiceKind::CardReward);
        let screen=r.decision().unwrap().room.unwrap();assert_eq!(screen.choices.iter().filter_map(|c|match c.kind{RoomChoiceKind::Card{id,..}=>Some(id),_=>None}).collect::<Vec<_>>(),row);
        let id=screen.choices[0].id;let next=r.state.next_instance;
        choose(&mut r,RoomChoiceKind::Card{id:row[0],upgrades:0});assert_eq!(&r.state.deck[..originals.len()],originals.as_slice());assert_eq!(r.state.deck.last().unwrap(),&CardInstance{id:row[0],instance:next,upgrades:0});
        let before=r.clone();assert!(r.step(&GameAction::Choose(id)).is_err());assert!(r==before);
        choose(&mut r,RoomChoiceKind::Proceed);
    }
    assert_eq!(r.state.combat_progress,CombatProgress{entered:3,settled:3});assert_eq!(r.state.relics.iter().filter(|&&id|id==RelicId::RingOfTheSnake).count(),1);
}

#[test]
fn m12_shared_shop_purge_smith_and_eligible_relic_rows_keep_source_ownership(){
    let mut r=run();choose(&mut r,RoomChoiceKind::NeowLament);diagnostic_room(&mut r,MapNodeKind::Shop);
    let shop=r.decision().unwrap().room.unwrap();let cards=shop.choices.iter().filter_map(|c|match c.kind{RoomChoiceKind::Card{id,..}=>Some(id),_=>None}).collect::<Vec<_>>();assert_eq!(cards,vec![CardId::QuickSlash,CardId::Reflex,CardId::Catalyst]);
    assert!(shop.choices.iter().any(|c|c.kind==RoomChoiceKind::Relic(RelicId::SneckoSkull)&&c.status==RoomChoiceStatus::Disabled));
    let before=r.clone();let unaffordable=shop.choices.iter().find(|c|c.kind==RoomChoiceKind::Relic(RelicId::SneckoSkull)).unwrap().id;assert!(r.step(&GameAction::Choose(unaffordable)).is_err());assert!(r==before);
    choose(&mut r,RoomChoiceKind::Card{id:CardId::Reflex,upgrades:0});assert_eq!(r.state.gold,24);assert_eq!(r.state.deck.last().unwrap().id,CardId::Reflex);
    choose(&mut r,RoomChoiceKind::Leave);diagnostic_room(&mut r,MapNodeKind::Rest);choose(&mut r,RoomChoiceKind::Smith);
    let selected=r.decision().unwrap().candidates.iter().find(|c|c.object.as_ref().is_some_and(|c|c.id==CardId::Neutralize)).unwrap().clone();
    let before=r.state.deck.clone();r.step(&selected.action).unwrap();let after=r.state.deck.iter().find(|c|c.instance==selected.object.as_ref().unwrap().instance).unwrap();assert_eq!((after.id,after.upgrades),(CardId::Neutralize,1));assert_eq!(r.state.deck.len(),before.len());
    let mut later=run();choose(&mut later,RoomChoiceKind::NeowLament);later.state.room_loop.as_mut().unwrap().act1.as_mut().unwrap().shop_visits=1;
    let stock=super::super::rooms::shop::screen(&later,&mut later.next_choice.clone()).unwrap();assert_eq!(stock.choices.iter().filter_map(|c|match c.kind{RoomChoiceKind::Card{id,..}=>Some(id),_=>None}).collect::<Vec<_>>(),vec![CardId::Deflect,CardId::Tactician,CardId::Nightmare]);assert!(stock.choices.iter().any(|c|c.kind==RoomChoiceKind::Relic(RelicId::BagOfPreparation)));
}

#[test]
fn m12_real_elite_alternatives_and_finite_character_eligible_stock(){
    use crate::content::EncounterId::*;
    for (seen,expected) in [(vec!["Hexaghost","SlimeBoss"],GremlinNob),(vec!["Guardian","SlimeBoss"],Lagavulin),(vec!["Guardian","Hexaghost"],ThreeSentries)]{
        let mut p=profile();p.seen_bosses[0]=seen;let mut r=Run::new_room_scenario(RunConfig{character:Character::Silent,ascension:0},p,RunRng::new([7;32],Partition::Test,0),RoomScenario::M12SilentAct1).unwrap();choose(&mut r,RoomChoiceKind::NeowLament);
        // Keep the supplied diagnostic elite ordinary rather than applying a
        // random burning buff; actual marked routes remain the shared owner.
        for n in &mut r.state.map.as_mut().unwrap().nodes{n.burning=false;}
        diagnostic_room(&mut r,MapNodeKind::Elite);assert_eq!(r.state.active_combat.as_ref().unwrap().encounter,expected);resolve_lament_fight(&mut r);
        assert_eq!(r.state.last_combat.as_ref().unwrap().outcome,crate::combat::Outcome::Victory);let cursor=r.state.room_loop.as_ref().unwrap().act1.as_ref().unwrap().elite_relic_cursor;assert_eq!(cursor,1);
        choose(&mut r,RoomChoiceKind::Relic(RelicId::Lantern));assert!(r.state.relics.contains(&RelicId::Lantern));let before=r.clone();let stale=r.decision().unwrap().room.unwrap().choices.iter().find(|c|c.kind==RoomChoiceKind::Relic(RelicId::Lantern)).unwrap().id;assert!(r.step(&GameAction::Choose(stale)).is_err());assert!(r==before);
    }
    let mut cursor=0;let mut offered=vec![];for _ in 0..8{let (relic,next)=super::super::rooms::m12_content::elite_relic(cursor,&[]).unwrap();assert!(!offered.contains(&relic));assert!(M12_SILENT_ACT1_RELICS.contains(&relic));offered.push(relic);cursor=next;}
    assert!(offered.contains(&RelicId::SneckoSkull));assert!(!offered.contains(&RelicId::MagicFlower));assert!(super::super::rooms::m12_content::elite_relic(cursor,&[]).is_err());
    assert_eq!(super::super::rooms::m12_content::elite_relic(0,&[RelicId::Lantern]).unwrap(),(RelicId::Pear,2));
}

#[test]
fn m12_real_boss_rewards_neutral_chest_children_and_act2_readiness_share_transition(){
    use crate::content::EncounterId::*;
    for boss in [Guardian,Hexaghost,SlimeBoss]{
        let mut r=boss_chest(boss);let chest=r.decision().unwrap().room.unwrap();assert_eq!(chest.choices.iter().filter_map(|c|match c.kind{RoomChoiceKind::Relic(id)=>Some(id),_=>None}).collect::<Vec<_>>(),vec![RelicId::EmptyCage,RelicId::SlaversCollar]);
        choose(&mut r,RoomChoiceKind::Skip);choose(&mut r,RoomChoiceKind::CancelSkip);assert_eq!(r.decision().unwrap().room.unwrap(),chest);
        let originals=r.state.deck.clone();choose(&mut r,RoomChoiceKind::Relic(RelicId::EmptyCage));assert_eq!(r.state.pending.as_ref().unwrap().selection.required,2);
        let first=r.state.pending.as_ref().unwrap().selection.candidates[0].id;r.step(&GameAction::Select(first)).unwrap();assert_eq!(r.state.deck,originals);let before=r.clone();assert!(r.step(&GameAction::Select(first)).is_err());assert!(r==before);
        let second=r.state.pending.as_ref().unwrap().selection.candidates[0].id;r.step(&GameAction::Select(second)).unwrap();assert_eq!(&r.state.deck[..],&originals[2..]);assert_eq!(r.state.act,1);
        // A damaged, otherwise legal supplied transition boundary exercises
        // ordinary healing. It does not assert a from-start outcome.
        r.state.hp=41;let reward=r.state.reward_ledger.total;let t=choose(&mut r,RoomChoiceKind::Proceed);assert_eq!(t.reward,0);assert_eq!(r.state.reward_ledger.total,reward);assert_eq!((r.state.act,r.state.floor,r.state.hp),(2,17,r.state.max_hp));
        let d=r.decision().unwrap();assert_eq!(d.room.unwrap().kind,RoomViewKind::Act2Ready);assert!(d.candidates.is_empty());assert_eq!(d.terminal,None);assert!(r.state.deck.iter().any(|c|c.id==CardId::Nightmare));assert_eq!(r.state.relics.iter().filter(|&&id|id==RelicId::RingOfTheSnake).count(),1);
        let before=r.clone();assert!(r.step(&GameAction::Confirm).is_err());assert!(r==before);
    }
}

#[test]
fn m12_accepted_transition_fault_retains_reset_and_does_not_heal_or_credit_twice(){
    let mut r=boss_chest(crate::content::EncounterId::Guardian);choose(&mut r,RoomChoiceKind::Relic(RelicId::SlaversCollar));r.state.hp=41;
    // Corrupt a retained internal frame after actual source acquisition, using
    // the existing transition driver's invariant seam, never a public override.
    r.state.room_loop.as_mut().unwrap().transition=Some(ActTransition{from:1,to:3,floor:17,reset_applied:false,heal_applied:false,initialized:false});
    let result=super::super::rooms::transition::drive(&mut r);assert!(matches!(result,Err(RunError::Faulted(_))));assert_eq!(r.state.act,3);assert_eq!(r.state.hp,r.state.max_hp);assert_eq!(r.state.terminal,None);
    let frame=r.state.room_loop.as_ref().unwrap().transition.as_ref().unwrap();assert!(frame.reset_applied&&frame.heal_applied&&!frame.initialized);
    let frozen=r.clone();assert!(r.step(&GameAction::Confirm).is_err());assert!(r==frozen);
}

#[test]
fn m12_actual_combat_rewards_rotate_potions_and_preserve_full_belt_claim(){
    use crate::content::PotionId::*;
    let mut r=run();choose(&mut r,RoomChoiceKind::NeowLament);
    for (visit,potion) in [FirePotion,StrengthPotion,WeakPotion].into_iter().enumerate(){
        next_real_reward(&mut r);assert_eq!(r.state.last_combat.as_ref().unwrap().visit,visit as u64);
        let screen=r.decision().unwrap().room.unwrap();
        let row=screen.choices.iter().find(|c|matches!(c.kind,RoomChoiceKind::Potion(_))).unwrap();
        assert_eq!(row.kind,RoomChoiceKind::Potion(potion));assert_eq!(row.status,RoomChoiceStatus::Offered);
        let token=row.id;choose(&mut r,RoomChoiceKind::Potion(potion));assert_eq!(r.state.potions[visit],Some(potion));
        assert_eq!(r.decision().unwrap().room.unwrap().choices.iter().find(|c|c.id==token).unwrap().status,RoomChoiceStatus::Claimed);
        let frozen=r.clone();assert!(r.step(&GameAction::Choose(token)).is_err());assert!(r==frozen);choose(&mut r,RoomChoiceKind::Proceed);
    }
    assert_eq!(r.state.potions,vec![Some(FirePotion),Some(StrengthPotion),Some(WeakPotion)]);
    // Supplied full-belt inventory diagnostic; the preceding loop is the real
    // normal-start reward/claim proof. No completion is inferred from this row.
    let mut full=run();choose(&mut full,RoomChoiceKind::NeowLament);full.state.potions=vec![Some(StrengthPotion);3];
    next_real_reward(&mut full);let screen=full.decision().unwrap().room.unwrap();let row=screen.choices.iter().find(|c|c.kind==RoomChoiceKind::Potion(FirePotion)).unwrap();let token=row.id;
    assert_eq!(row.status,RoomChoiceStatus::Disabled);let frozen=full.clone();assert!(full.step(&GameAction::Choose(token)).is_err());assert!(full==frozen);
    full.step(&GameAction::DiscardPotion{slot:0}).unwrap();choose(&mut full,RoomChoiceKind::Potion(FirePotion));assert_eq!(full.state.potions,vec![Some(FirePotion),Some(StrengthPotion),Some(StrengthPotion)]);
    let frozen=full.clone();assert!(full.step(&GameAction::Choose(token)).is_err());assert!(full==frozen);
}

#[test]
fn m12_shared_events_purge_and_chest_keep_exact_child_identity_and_exclusions(){
    let mut r=run();choose(&mut r,RoomChoiceKind::NeowLament);
    diagnostic_room(&mut r,MapNodeKind::Question);assert_eq!(r.decision().unwrap().room.unwrap().kind,RoomViewKind::Event(crate::content::EventId::UpgradeShrine));choose(&mut r,RoomChoiceKind::ShrineUpgrade);
    let selected=r.decision().unwrap().candidates.into_iter().find(|c|c.object.as_ref().is_some_and(|c|c.id==CardId::Survivor)).unwrap();let instance=selected.object.unwrap().instance;r.step(&selected.action).unwrap();assert_eq!(r.state.deck.iter().find(|c|c.instance==instance).unwrap().upgrades,1);
    diagnostic_room(&mut r,MapNodeKind::Question);assert_eq!(r.decision().unwrap().room.unwrap().kind,RoomViewKind::Event(crate::content::EventId::Purifier));choose(&mut r,RoomChoiceKind::ShrineRemove);
    let selected=r.decision().unwrap().candidates[0].clone();let removed=selected.object.unwrap().instance;r.step(&selected.action).unwrap();assert!(!r.state.deck.iter().any(|c|c.instance==removed));
    diagnostic_room(&mut r,MapNodeKind::Shop);choose(&mut r,RoomChoiceKind::Purge);let before=r.state.deck.clone();let gold=r.state.gold;r.step(&GameAction::Cancel).unwrap();assert_eq!(r.state.deck,before);assert_eq!(r.state.gold,gold);
    choose(&mut r,RoomChoiceKind::Purge);let selected=r.decision().unwrap().candidates.iter().find(|c|c.object.is_some()).unwrap().clone();let removed=selected.object.unwrap().instance;let token=selected.action;r.step(&token).unwrap();assert!(!r.state.deck.iter().any(|c|c.instance==removed));assert_eq!(r.state.gold,gold-75);assert_eq!(r.state.room_loop.as_ref().unwrap().purge_cost,100);let frozen=r.clone();assert!(r.step(&token).is_err());assert!(r==frozen);
    choose(&mut r,RoomChoiceKind::Leave);diagnostic_room(&mut r,MapNodeKind::Treasure);choose(&mut r,RoomChoiceKind::OpenChest);let chest=r.decision().unwrap().room.unwrap();let relic=chest.choices.iter().find(|c|matches!(c.kind,RoomChoiceKind::Relic(_))).unwrap().clone();assert_eq!(relic.kind,RoomChoiceKind::Relic(RelicId::Strawberry));
    choose(&mut r,RoomChoiceKind::Sapphire);assert_eq!(r.state.keys,[false,true,false]);assert!(!r.state.relics.contains(&RelicId::Strawberry));assert_eq!(r.decision().unwrap().room.unwrap().choices.iter().find(|c|c.id==relic.id).unwrap().status,RoomChoiceStatus::Skipped);
    let frozen=r.clone();assert!(r.step(&GameAction::Choose(relic.id)).is_err());assert!(r==frozen);
}

#[test]
fn m12_green_gain_overflow_and_invalid_source_proceed_are_atomic(){
    let mut r=run();choose(&mut r,RoomChoiceKind::NeowLament);next_real_reward(&mut r);choose(&mut r,RoomChoiceKind::CardReward);r.state.next_instance=u64::MAX;
    let row=r.decision().unwrap().room.unwrap().choices.iter().find(|c|matches!(c.kind,RoomChoiceKind::Card{..})).unwrap().clone();let frozen=r.clone();assert!(r.step(&GameAction::Choose(row.id)).is_err());assert!(r==frozen);
    let mut r=boss_chest(crate::content::EncounterId::Guardian);choose(&mut r,RoomChoiceKind::Relic(RelicId::SlaversCollar));r.state.hp=0;
    let token=r.decision().unwrap().room.unwrap().choices.iter().find(|c|c.kind==RoomChoiceKind::Proceed).unwrap().id;let frozen=r.clone();assert!(r.step(&GameAction::Choose(token)).is_err());assert!(r==frozen);assert!(r.state.room_loop.as_ref().unwrap().transition.is_none());
}

#[test]
fn m12_shop_acquired_snecko_skull_uses_existing_poison_listener(){
    for (skull,want) in [(false,5),(true,6)]{
        let mut r=run();choose(&mut r,RoomChoiceKind::NeowMaxHp{amount:7});
        // Supplied inventory/economy diagnostic. Relic acquisition is still the
        // actual shop owner, and Poison still comes from an actual card play.
        r.state.gold=150;diagnostic_room(&mut r,MapNodeKind::Shop);
        if skull{choose(&mut r,RoomChoiceKind::Relic(RelicId::SneckoSkull));assert_eq!(r.state.gold,0);}
        choose(&mut r,RoomChoiceKind::Leave);mutations::obtain_card(&mut r.state,CardId::DeadlyPoison,0).unwrap();diagnostic_room(&mut r,MapNodeKind::Monster);
        assert!(r.state.relics.is_empty());
        let mut played=false;
        for _ in 0..8{
            let d=r.decision().unwrap();let combat=d.combat.as_ref().unwrap();
            if let Some(card)=combat.hand.iter().find(|c|c.id==CardId::DeadlyPoison){
                let action=d.candidates.iter().find(|a|matches!(a.action,GameAction::Combat(CombatAction::PlayCard{instance,..}) if instance==card.instance.0)).unwrap().action.clone();r.step(&action).unwrap();played=true;break
            }
            r.step(&GameAction::Combat(CombatAction::EndTurn)).unwrap();
        }
        assert!(played);let d=r.decision().unwrap();let c=d.combat.unwrap();assert_eq!(c.enemies[0].powers.iter().find(|p|p.id.0=="poison").unwrap().amount,want);assert_eq!(d.player.relics.iter().filter(|&&id|id==RelicId::SneckoSkull).count(),usize::from(skull));
    }
}
