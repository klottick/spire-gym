use crate::{combat::rng::Partition, content::{Character,RelicId},run::*};
fn profile()->CreationProfile {let mut p=CreationProfile::experienced();p.neow_eligible=false;p}
pub(super) fn run()->Run {Run::new_room_scenario(RunConfig{character:Character::Ironclad,ascension:0},profile(),RunRng::new([7;32],Partition::Test,0),RoomScenario::M4Act1Prefix).unwrap()}
pub(super) fn choose(r:&mut Run,kind:RoomChoiceKind)->RunTransition {
    let d=r.decision().unwrap();let id=d.room.unwrap().choices.into_iter().find(|c|c.kind==kind).unwrap().id;
    r.step(&GameAction::Choose(id)).unwrap()
}
#[test]
fn normal_resources_short_neow_and_real_map_choice() {
    let mut r=run();let d=r.decision().unwrap();
    assert_eq!((d.player.hp,d.player.max_hp,d.player.gold,d.objects.len()),(80,80,99,10));
    assert_eq!(d.player.relics,vec![RelicId::BurningBlood]);assert_eq!(d.player.potions,vec![None;3]);
    assert!(r.public_map().is_none());
    choose(&mut r,RoomChoiceKind::NeowMaxHp{amount:8});
    let d=r.decision().unwrap();assert_eq!((d.player.hp,d.player.max_hp,d.reward_total),(88,88,0));
    assert_eq!(d.candidates.iter().map(|c|c.action.clone()).collect::<Vec<_>>(),vec![GameAction::Move(NodeId(2)),GameAction::Move(NodeId(4))]);
    let before=r.clone();assert!(r.step(&GameAction::Move(NodeId(24))).is_err());assert!(r==before);
    let t=r.step(&GameAction::Move(NodeId(2))).unwrap();assert_eq!(t.reward,1);
    let d=r.decision().unwrap();assert!(d.combat.is_some());assert_eq!(d.floor,1);assert_eq!(d.reward_total,1);
    assert!(r.remove_card(d.objects[0].instance).is_err());
}
#[test]
fn captured_neow_heals_only_earned_amount_and_invalid_profiles_reject() {
    let mut r=run();r.state.hp=40;choose(&mut r,RoomChoiceKind::NeowMaxHp{amount:8});
    assert_eq!((r.state.hp,r.state.max_hp),(48,88));
    assert!(Run::new_room_scenario(RunConfig{character:Character::Ironclad,ascension:0},CreationProfile::experienced(),RunRng::new([7;32],Partition::Test,0),RoomScenario::M4Act1Prefix).is_err());
}

#[test]
fn lament_starts_every_initial_actor_at_one_hp_and_keeps_maximum() {
    let mut r=run();choose(&mut r,RoomChoiceKind::NeowLament);
    assert_eq!(r.state.relic_counters.get(&RelicId::NeowsLament),Some(&3));
    // A valid native Two Louse entry supplements the normal-resource public route.
    r.enter_owned(crate::content::EncounterId::TwoLouse,super::super::CombatOrigin::MapRoom{node:NodeId(45),kind:MapNodeKind::Monster}).unwrap();
    let d=r.decision().unwrap();let c=d.combat.unwrap();
    assert_eq!(c.enemies.len(),2);assert!(c.enemies.iter().all(|e|e.hp==1&&e.max_hp>1));
    assert!(r.state.relic_counters.is_empty(),"active inventory has one counter owner");
}

#[test]
fn lament_counter_returns_expires_and_potion_use_never_resets_it() {
    use crate::{combat::{CombatAction,TargetId},content::{CardId,EncounterId,PotionId}};
    let mut r=run();choose(&mut r,RoomChoiceKind::NeowLament);
    r.step(&GameAction::Move(NodeId(2))).unwrap();
    for expected in [2,1,-2,-2] {
        if r.state.combat.is_none(){r.enter_owned(EncounterId::Cultist,super::super::CombatOrigin::MapRoom{node:NodeId(2),kind:MapNodeKind::Monster}).unwrap();}
        let d=r.decision().unwrap();assert_eq!(d.player.relic_counters[0].value,expected);
        if expected==-2&&r.state.combat_progress.entered==4{assert!(d.combat.unwrap().enemies[0].hp>1);break}
        assert_eq!(d.combat.as_ref().unwrap().enemies[0].hp,1);
        if expected==1 {
            r.choose_combat(CombatAction::UsePotion{slot:0,target:Some(TargetId(1))}).unwrap();
        }else{
            let id=d.combat.unwrap().hand.iter().find(|c|c.id==CardId::Strike||c.id==CardId::Bash).unwrap().instance.0;
            r.choose_combat(CombatAction::PlayCard{instance:id,target:Some(TargetId(1))}).unwrap();
        }
        assert_eq!(r.state.relic_counters.get(&RelicId::NeowsLament),Some(&expected));
        choose(&mut r,RoomChoiceKind::Potion(PotionId::FirePotion));choose(&mut r,RoomChoiceKind::Proceed);
    }
}

#[test]
fn accepted_startup_fault_keeps_lament_counter_and_move_reward() {
    let mut r=run();choose(&mut r,RoomChoiceKind::NeowLament);r.state.relics.push(RelicId::PureWater);
    r.state.deck=(1..=999).map(|instance|CardInstance{ misc:0,instance,id:crate::content::CardId::Strike,upgrades:0}).collect();
    let error=r.step(&GameAction::Move(NodeId(2))).unwrap_err();assert!(matches!(error,RunError::Faulted(_)));
    assert_eq!(r.state.reward_ledger.total,1);assert!(r.state.relic_counters.is_empty());
    let items=r.state.combat.as_ref().unwrap().item_context().unwrap();assert_eq!(items.counters().get(&RelicId::NeowsLament),Some(&2));
    let before=r.clone();assert_eq!(r.step(&GameAction::Move(NodeId(2))),Err(error));assert!(r==before);
}

#[test]
fn death_settlement_overflow_is_retained_operational_fault_not_retryable_loss() {
    use crate::combat::CombatAction;
    let mut r=run();choose(&mut r,RoomChoiceKind::NeowMaxHp{amount:8});r.step(&GameAction::Move(NodeId(4))).unwrap();
    r.state.reward_ledger.total=i32::MIN;
    let error=loop {match r.choose_combat(CombatAction::EndTurn){Ok(t)=>assert!(t.terminal.is_none()),Err(e)=>break e}};
    assert!(matches!(error,RunError::Faulted(_)));assert_eq!(r.state.terminal,None);assert!(r.state.combat.is_some());
    let before=r.clone();assert_eq!(r.choose_combat(CombatAction::EndTurn),Err(error));assert!(r==before);
}
