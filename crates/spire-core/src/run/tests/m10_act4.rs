use crate::{combat::Outcome, content::{CardRarity,EncounterId,RelicId}, run::*};
use super::{m9_boss, rooms::choose};

fn real_m10_heart_source(all_keys:bool)->Run{
    let (mut run,node)=m9_boss::source(EncounterId::TimeEater);
    run.state.room_loop.as_mut().unwrap().scenario=RoomScenario::M10IroncladHeart;
    run.state.act4_enabled=true;
    if all_keys{
        for key in [Key::Ruby,Key::Sapphire,Key::Emerald]{
            let event=super::super::mutations::acquire_key(&mut run.state,key).unwrap();
            assert_eq!(super::super::reward::commit(&mut run.state,&[event]).unwrap(),5);
        }
    }
    run.step(&GameAction::Move(node)).unwrap();
    m9_boss::hit(&mut run,0,456);
    assert_eq!(run.state.combat.as_ref().unwrap().public_full().unwrap().outcome,Some(Outcome::Victory));
    run.settle_combat().unwrap();
    choose(&mut run,RoomChoiceKind::Proceed);
    for stage in 1..=3 { choose(&mut run,RoomChoiceKind::Act3Continue{stage}); }
    run
}
fn select_card(run:&mut Run,instance:u64)->GameAction{
    run.decision().unwrap().candidates.iter().find(|candidate|candidate.object.as_ref().is_some_and(|object|object.instance==instance)).unwrap().action.clone()
}
fn choose_rest(run:&mut Run){
    let row=run.decision().unwrap().room.unwrap().choices.into_iter().find(|row|matches!(row.kind,RoomChoiceKind::Rest{..})).unwrap();
    run.step(&GameAction::Choose(row.id)).unwrap();
}

#[test]
fn m10_fourth_all_key_choice_enters_the_real_act4_route(){
    let mut run=real_m10_heart_source(true);
    let fourth=run.decision().unwrap().room.unwrap();
    assert_eq!(fourth.kind,RoomViewKind::Act3Ending);
    assert_eq!(fourth.choices.iter().map(|row|row.kind.clone()).collect::<Vec<_>>(),vec![RoomChoiceKind::Act3Continue{stage:4}]);
    let fourth_id=fourth.choices[0].id;
    let result=choose(&mut run,RoomChoiceKind::Act3Continue{stage:4});
    assert_eq!(result.reward,0);
    assert_eq!(result.terminal,None);
    assert!(result.committed_events.contains(&CommittedEvent::ActAdvanced{from:3,to:4}));
    assert_eq!((run.state.act,run.state.floor,run.state.terminal),(4,50,None));
    assert_eq!(run.decision().unwrap().room.unwrap().kind,RoomViewKind::Map);
    let map=run.public_map().unwrap();
    assert_eq!(map.act,4);
    assert_eq!(map.nodes.iter().map(|node|(node.floor,node.kind)).collect::<Vec<_>>(),vec![
        (51,MapNodeKind::Rest),(52,MapNodeKind::Shop),(53,MapNodeKind::Elite),(54,MapNodeKind::Boss),
    ]);
    assert_eq!(map.nodes[0].edges,vec![map.nodes[1].id]);
    assert_eq!(map.nodes[1].edges,vec![map.nodes[2].id]);
    assert_eq!(map.nodes[2].edges,vec![map.nodes[3].id]);
    assert!(map.nodes[3].edges.is_empty());
    assert_eq!(run.state.history.events.iter().filter(|event|matches!(event,CommittedEvent::ActAdvanced{from:3,to:4})).count(),1);
    let after_entry=run.clone();
    assert_eq!(run.step(&GameAction::Choose(fourth_id)),Err(RunError::InvalidAction));
    assert_eq!(run,after_entry,"the accepted fourth choice cannot repeat its heal, ActAdvanced event, or route initialization");
}

#[test]
fn m10_fourth_choice_rejects_malformed_hp_and_changed_source_before_acceptance(){
    for malformed_hp in [true,false]{
        let mut run=real_m10_heart_source(true);
        let choice=run.decision().unwrap().room.unwrap().choices[0].id;
        if malformed_hp{run.state.hp=run.state.max_hp+1}else{run.state.last_combat=None;}
        let before=run.clone();
        assert_eq!(run.step(&GameAction::Choose(choice)),Err(RunError::InvalidAction));
        assert_eq!(run,before,"bad source/HP facts must be rejected before accepting the choice or changing its phase");
        assert!(run.state.room_loop.as_ref().unwrap().act4.is_none());
    }
}

#[test]
fn m10_fourth_choice_preserves_all_keyed_resources_and_uses_ordinary_heal_prevention(){
    for prevent_heal in [false,true]{
        let mut run=real_m10_heart_source(true);
        run.state.hp=40;
        if prevent_heal{crate::items::acquire_relic(&mut run.state,RelicId::MarkOfBloom).unwrap();}
        let (max_hp,gold,deck,relics,potions,keys,visits,rng)=(run.state.max_hp,run.state.gold,run.state.deck.clone(),run.state.relics.clone(),run.state.potions.clone(),run.state.keys,run.state.combat_progress,run.state.rng.clone());
        let before_total=run.state.reward_ledger.total;
        let transition=choose(&mut run,RoomChoiceKind::Act3Continue{stage:4});
        assert_eq!(transition.reward,0);
        assert_eq!(run.state.hp,if prevent_heal{40}else{max_hp},"the ordinary healing owner and Mark of Bloom decide the Act4 transition heal");
        assert_eq!((run.state.max_hp,run.state.gold,run.state.deck.clone(),run.state.relics.clone(),run.state.potions.clone(),run.state.keys,run.state.combat_progress),
            (max_hp,gold,deck,relics,potions,keys,visits));
        assert_eq!(run.state.rng,rng,"the fixed route does not reroll or reseed the Run");
        assert_eq!(run.state.reward_ledger.total,before_total,"ActAdvanced and the door choice do not award floor reward");
        let progress=run.state.room_loop.as_ref().unwrap().act4.as_ref().unwrap();
        assert_eq!(progress.phase,super::super::rooms::act4::Act4TransitionPhase::Published);
        assert_eq!(progress.source_record.outcome,Outcome::Victory);
    }
}

#[test]
fn m10_missing_or_disabled_act4_keeps_the_fourth_no_key_choice_without_transition(){
    let mut missing=real_m10_heart_source(false);
    let before=(missing.state.hp,missing.state.gold,missing.state.keys,missing.state.rng.clone(),missing.state.reward_ledger.total);
    let result=choose(&mut missing,RoomChoiceKind::Act3Continue{stage:4});
    assert_eq!(result.terminal,Some(RunTerminal::Act3NoKeys));
    assert_eq!((missing.state.act,missing.state.floor),(3,50));
    assert_eq!((missing.state.hp,missing.state.gold,missing.state.keys,missing.state.rng.clone(),missing.state.reward_ledger.total),before);
    assert!(!result.committed_events.iter().any(|event|matches!(event,CommittedEvent::ActAdvanced{..}|CommittedEvent::FloorEntered{act:4,..})));

    let mut disabled=real_m10_heart_source(true);disabled.state.act4_enabled=false;
    let before=(disabled.state.hp,disabled.state.keys,disabled.state.rng.clone(),disabled.state.reward_ledger.total);
    let result=choose(&mut disabled,RoomChoiceKind::Act3Continue{stage:4});
    assert_eq!(result.terminal,Some(RunTerminal::Act3NoKeys));
    assert_eq!((disabled.state.act,disabled.state.floor,disabled.state.hp,disabled.state.keys,disabled.state.rng.clone(),disabled.state.reward_ledger.total),
        (3,50,before.0,before.1,before.2,before.3));
}

#[test]
fn m10_act4_rest_and_shop_are_real_rooms_before_native_shield_spear_entry(){
    let mut run=real_m10_heart_source(true);
    choose(&mut run,RoomChoiceKind::Act3Continue{stage:4});
    let route=run.public_map().unwrap();
    let rest=route.nodes[0].id;
    let moved=run.step(&GameAction::Move(rest)).unwrap();
    assert_eq!((run.state.act,run.state.floor,moved.reward),(4,51,10));
    assert_eq!(run.decision().unwrap().room.unwrap().kind,RoomViewKind::Campfire);
    assert!(run.decision().unwrap().room.unwrap().choices.iter().any(|row|row.kind==RoomChoiceKind::Smith&&row.status==RoomChoiceStatus::Offered));
    assert!(run.decision().unwrap().room.unwrap().choices.iter().any(|row|row.kind==RoomChoiceKind::Recall&&row.status==RoomChoiceStatus::Disabled));
    run.state.hp=40;
    let rest_heal=(run.state.max_hp as f32*0.3_f32) as u32;
    choose(&mut run,RoomChoiceKind::Rest{heal:rest_heal});
    assert_eq!(run.state.hp,(40+rest_heal).min(run.state.max_hp),"the actual Campfire Rest choice heals by its offered nominal amount through the ordinary owner");
    assert_eq!(run.decision().unwrap().room.unwrap().kind,RoomViewKind::Map);
    let total=run.state.reward_ledger.total;let old_floor=run.state.floor;
    let shop=run.public_map().unwrap().nodes[1].id;
    let mut expected_shop_rng=run.state.rng.clone();
    super::super::rooms::m10_content::sample_capture_row(&mut expected_shop_rng,&[4,52,shop.0],&[CardRarity::Common,CardRarity::Uncommon,CardRarity::Rare]).unwrap();
    let entered=run.step(&GameAction::Move(shop)).unwrap();
    assert_eq!((run.state.floor,entered.reward),(52,10));assert_eq!(run.state.reward_ledger.total,total+10);
    assert_eq!(run.state.rng,expected_shop_rng,"the shop capture uses its actual Act4 floor/node destination, not the previous room key");
    assert_eq!(run.decision().unwrap().room.unwrap().kind,RoomViewKind::Shop);
    let leave=run.decision().unwrap().room.unwrap().choices.into_iter().find(|row|row.kind==RoomChoiceKind::Leave).unwrap().id;
    assert_eq!(run.step(&GameAction::Choose(leave)).unwrap().reward,0);
    assert_eq!(run.decision().unwrap().room.unwrap().kind,RoomViewKind::Map);
    let previous_visits=run.state.combat_progress.entered;
    let elite=run.public_map().unwrap().nodes[2].id;let entered=run.step(&GameAction::Move(elite)).unwrap();
    assert_eq!((run.state.floor,entered.reward),(53,10));
    assert_eq!(run.state.active_combat.as_ref().unwrap().encounter,EncounterId::ShieldAndSpear);
    assert_eq!(run.state.combat.as_ref().unwrap().public_full().unwrap().enemies.len(),2);
    assert_eq!(run.state.combat_progress.entered,previous_visits+1);
    assert!(run.state.combat.as_ref().unwrap().item_context().unwrap().battle.elite_trigger);
    assert!(!run.state.map.as_ref().unwrap().nodes.iter().find(|node|node.id==elite).unwrap().burning);
    assert_eq!(old_floor,51);
}

#[test]
fn m10_act4_smith_and_purge_keep_their_mandatory_owned_selection_children(){
    let mut run=real_m10_heart_source(true);choose(&mut run,RoomChoiceKind::Act3Continue{stage:4});
    let rest=run.public_map().unwrap().nodes[0].id;run.step(&GameAction::Move(rest)).unwrap();
    let bash=run.state.deck.iter().find(|card|card.id==crate::content::CardId::Bash).unwrap().clone();
    choose(&mut run,RoomChoiceKind::Smith);
    assert_eq!(run.state.pending.as_ref().unwrap().selection.required,1);
    let smith_parent=run.state.pending.as_ref().unwrap().parent.clone();
    let pending_smith=run.clone();
    assert_eq!(run.step(&GameAction::Move(run.public_map().unwrap().nodes[1].id)),Err(RunError::InvalidAction));
    assert_eq!(run,pending_smith,"the mandatory Smith child keeps its source room open until selection resolves");
    let smith_action=select_card(&mut run,bash.instance);run.step(&smith_action).unwrap();
    assert_eq!(run.state.deck.iter().find(|card|card.instance==bash.instance).unwrap().upgrades,1);
    assert_eq!(run.state.last_resumed_parent,Some(match smith_parent{ContinuationTarget::Room{choice,..}=>choice,_=>panic!("Smith owns a room selection")}));
    assert_eq!(run.decision().unwrap().room.unwrap().kind,RoomViewKind::Map);

    let shop=run.public_map().unwrap().nodes[1].id;run.step(&GameAction::Move(shop)).unwrap();
    let gold=run.state.gold;let purge=run.decision().unwrap().room.unwrap().choices.into_iter().find(|row|row.kind==RoomChoiceKind::Purge).unwrap();
    assert_eq!(purge.price,Some(75));run.step(&GameAction::Choose(purge.id)).unwrap();
    assert_eq!(run.state.gold,gold,"opening the owned selector does not charge before its required child is accepted");
    let pending_purge=run.clone();
    let leave=run.state.room_loop.as_ref().unwrap().screen.choices.iter().find(|row|row.kind==RoomChoiceKind::Leave).unwrap().id;
    assert_eq!(run.step(&GameAction::Choose(leave)),Err(RunError::InvalidAction));
    assert_eq!(run,pending_purge,"the mandatory paid-purge child cannot be bypassed through Leave");
    let removal=run.state.deck.iter().find(|card|card.id==crate::content::CardId::Strike).unwrap().instance;
    let purge_action=select_card(&mut run,removal);run.step(&purge_action).unwrap();
    assert!(!run.state.deck.iter().any(|card|card.instance==removal));
    assert_eq!((run.state.gold,run.state.room_loop.as_ref().unwrap().purge_cost),(gold-75,100));
    assert_eq!(run.decision().unwrap().room.unwrap().kind,RoomViewKind::Shop);
    choose(&mut run,RoomChoiceKind::Leave);
    assert_eq!(run.decision().unwrap().room.unwrap().kind,RoomViewKind::Map);
}

#[test]
fn m10_act4_wrong_source_and_duplicate_room_actions_reject_without_new_floor_credit(){
    let mut run=real_m10_heart_source(true);
    let choice=run.decision().unwrap().room.unwrap().choices[0].id;
    let mut invalid=run.clone();invalid.state.map.as_mut().unwrap().current=None;
    let before=invalid.clone();assert_eq!(invalid.step(&GameAction::Choose(choice)),Err(RunError::InvalidAction));assert_eq!(invalid,before);
    choose(&mut run,RoomChoiceKind::Act3Continue{stage:4});
    let rest=run.public_map().unwrap().nodes[0].id;let first=run.step(&GameAction::Move(rest)).unwrap();assert_eq!(first.reward,10);
    let reward_total=run.state.reward_ledger.total;let screen=run.decision().unwrap().room.unwrap();
    let old_move=GameAction::Move(rest);let stale=run.clone();assert_eq!(run.step(&old_move),Err(RunError::InvalidAction));assert_eq!(run,stale);
    assert_eq!(run.state.reward_ledger.total,reward_total);assert_eq!(run.decision().unwrap().room.unwrap(),screen);
}

#[test]
fn m10_act4_shop_offer_overflow_rejects_before_committing_the_actual_shop_move(){
    let mut run=real_m10_heart_source(true);choose(&mut run,RoomChoiceKind::Act3Continue{stage:4});
    let rest=run.public_map().unwrap().nodes[0].id;run.step(&GameAction::Move(rest)).unwrap();choose_rest(&mut run);
    let shop=run.public_map().unwrap().nodes[1].id;
    run.next_choice=u64::MAX-2;
    let before=run.clone();
    assert_eq!(run.step(&GameAction::Move(shop)),Err(RunError::Unsupported("choice identity exhausted")));
    assert_eq!(run,before,"shop rows and RNG are prepared on detached cursors before the actual room move commits");
}

#[test]
fn m10_act4_accepted_elite_entry_fault_retains_room_and_first_fault_without_fake_combat(){
    let mut run=real_m10_heart_source(true);choose(&mut run,RoomChoiceKind::Act3Continue{stage:4});
    let rest=run.public_map().unwrap().nodes[0].id;run.step(&GameAction::Move(rest)).unwrap();choose_rest(&mut run);
    let shop=run.public_map().unwrap().nodes[1].id;run.step(&GameAction::Move(shop)).unwrap();choose(&mut run,RoomChoiceKind::Leave);
    let before=run.clone();let elite=run.public_map().unwrap().nodes[2].id;
    run.state.combat_progress.entered=u64::MAX;
    let before_move=run.clone();
    let error=run.step(&GameAction::Move(elite)).unwrap_err();
    let RunError::Faulted(first_fault)=error else{panic!("the accepted map entry must retain the owner failure")};
    assert_eq!(first_fault.checkpoint,"run.m10.act4.encounter");
    assert_eq!(first_fault.kind,RunFaultKind::Invariant("accepted Act4 encounter entry failed"));
    assert_eq!(run.state.floor,53);
    assert_eq!(run.state.map.as_ref().unwrap().current,Some(elite));
    let node=run.state.map.as_ref().unwrap().nodes.iter().find(|node|node.id==elite).unwrap();
    assert!(node.visited);assert_eq!(node.encounter,Some(EncounterId::ShieldAndSpear));
    assert_eq!(run.state.history.rooms.len(),before_move.state.history.rooms.len()+1);
    assert_eq!(run.state.history.events.iter().filter(|event|matches!(event,CommittedEvent::FloorEntered{act:4,floor:53})).count(),1);
    assert_eq!(run.state.reward_ledger.total,before_move.state.reward_ledger.total+10);
    assert_eq!(run.state.combat_progress.entered,u64::MAX);
    assert_eq!((run.state.hp,run.state.max_hp,run.state.gold,run.state.deck.clone(),run.state.relics.clone(),run.state.potions.clone(),run.state.keys,run.state.relic_counters.clone(),run.state.room_loop.as_ref().unwrap().m10_elite_relic_cursor),
        (before_move.state.hp,before_move.state.max_hp,before_move.state.gold,before_move.state.deck.clone(),before_move.state.relics.clone(),before_move.state.potions.clone(),before_move.state.keys,before_move.state.relic_counters.clone(),before_move.state.room_loop.as_ref().unwrap().m10_elite_relic_cursor));
    assert!(run.state.combat.is_none());assert!(run.state.active_combat.is_none());
    assert!(!run.state.history.events.iter().any(|event|matches!(event,CommittedEvent::CombatEntered{encounter:EncounterId::ShieldAndSpear,..})));
    assert_eq!(run.state.rng,before_move.state.rng);
    let progress=run.state.room_loop.as_ref().unwrap().act4.as_ref().unwrap();
    assert_eq!(progress.phase,super::super::rooms::act4::Act4TransitionPhase::Published);
    assert_eq!(progress.source_record,before.state.room_loop.as_ref().unwrap().act4.as_ref().unwrap().source_record);
    assert_eq!(progress.accepted_choice,before.state.room_loop.as_ref().unwrap().act4.as_ref().unwrap().accepted_choice);
    let retained=run.clone();
    assert_eq!(run.step(&GameAction::Move(elite)),Err(RunError::Faulted(first_fault)));
    assert_eq!(run,retained,"retry returns the exact first fault without healing, resetting, rerolling, or repeating room credit");
}

#[test]
fn m10_act4_fixed_heart_node_enters_the_actual_corrupt_heart_combat(){
    // Supplied accepted-map checkpoint for the boss entry seam; this is not a
    // claim that the Act4 route was traversed in a normal-start episode.
    let mut run=real_m10_heart_source(true);choose(&mut run,RoomChoiceKind::Act3Continue{stage:4});
    super::m10_settlement::complete_elite_rewards_for_supplied_heart_entry(&mut run);
    let heart={
        let map=run.state.map.as_mut().unwrap();
        map.current=Some(map.nodes[2].id);map.nodes[3].id
    };
    run.state.floor=53;run.state.room="Map";
    let entered=run.step(&GameAction::Move(heart)).unwrap();
    assert_eq!((run.state.act,run.state.floor,entered.reward),(4,54,10));
    assert_eq!(run.state.active_combat.as_ref().unwrap().encounter,EncounterId::CorruptHeart);
    let view=run.decision().unwrap().combat.unwrap();
    assert_eq!(view.enemies.len(),1);
    assert_eq!(view.enemies[0].enemy_id,crate::content::EnemyId::CorruptHeart);
    assert!(run.state.combat.as_ref().unwrap().item_context().unwrap().battle.boss);
    assert!(!run.state.room_loop.as_ref().unwrap().screen.choices.iter().any(|row|row.kind==RoomChoiceKind::Proceed));
}
