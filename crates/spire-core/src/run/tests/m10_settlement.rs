use crate::{combat::{CombatAction,Outcome},content::{CardId,CardRarity,Catalog,EncounterId,RelicId},run::*};
use super::{m9_boss,rooms::choose};

fn run_at_act4()->Run{
    let (mut run,node)=m9_boss::source(EncounterId::TimeEater);
    run.state.room_loop.as_mut().unwrap().scenario=RoomScenario::M10IroncladHeart;
    run.state.act4_enabled=true;
    for key in [Key::Ruby,Key::Sapphire,Key::Emerald]{
        let event=super::super::mutations::acquire_key(&mut run.state,key).unwrap();
        super::super::reward::commit(&mut run.state,&[event]).unwrap();
    }
    run.step(&GameAction::Move(node)).unwrap();m9_boss::hit(&mut run,0,456);run.settle_combat().unwrap();
    choose(&mut run,RoomChoiceKind::Proceed);
    for stage in 1..=4{choose(&mut run,RoomChoiceKind::Act3Continue{stage});}
    run
}

fn enter_elite(run:&mut Run){
    let rest=run.public_map().unwrap().nodes[0].id;run.step(&GameAction::Move(rest)).unwrap();
    let heal=run.decision().unwrap().room.unwrap().choices.into_iter().find_map(|row|match row.kind{RoomChoiceKind::Rest{heal}=>Some((row.id,heal)),_=>None}).unwrap();
    run.step(&GameAction::Choose(heal.0)).unwrap();
    let shop=run.public_map().unwrap().nodes[1].id;run.step(&GameAction::Move(shop)).unwrap();
    let leave=run.decision().unwrap().room.unwrap().choices.into_iter().find(|row|row.kind==RoomChoiceKind::Leave).unwrap().id;
    run.step(&GameAction::Choose(leave)).unwrap();
    let elite=run.public_map().unwrap().nodes[2].id;run.step(&GameAction::Move(elite)).unwrap();
}

fn enter_elite_at_hp(run:&mut Run,hp:u32){
    let rest=run.public_map().unwrap().nodes[0].id;run.step(&GameAction::Move(rest)).unwrap();
    let heal=run.decision().unwrap().room.unwrap().choices.into_iter().find_map(|row|match row.kind{RoomChoiceKind::Rest{heal}=>Some((row.id,heal)),_=>None}).unwrap();
    run.step(&GameAction::Choose(heal.0)).unwrap();
    let shop=run.public_map().unwrap().nodes[1].id;run.step(&GameAction::Move(shop)).unwrap();
    let leave=run.decision().unwrap().room.unwrap().choices.into_iter().find(|row|row.kind==RoomChoiceKind::Leave).unwrap().id;
    run.step(&GameAction::Choose(leave)).unwrap();run.state.hp=hp;
    let elite=run.public_map().unwrap().nodes[2].id;run.step(&GameAction::Move(elite)).unwrap();
}

fn defeat_pair(run:&mut Run){
    let count=run.decision().unwrap().combat.unwrap().enemies.len();assert_eq!(count,2);
    for slot in 0..count{m9_boss::hit(run,slot,10_000);}
    assert_eq!(run.state.combat.as_ref().unwrap().public_full().unwrap().outcome,Some(Outcome::Victory));
}

pub(super) fn complete_elite_rewards_for_supplied_heart_entry(run:&mut Run){
    enter_elite(run);defeat_pair(run);run.settle_combat().unwrap();choose(run,RoomChoiceKind::Proceed);
}

fn enter_heart(run:&mut Run){
    complete_elite_rewards_for_supplied_heart_entry(run);
    let heart=run.public_map().unwrap().nodes[3].id;
    run.state.max_hp=999;run.state.hp=999;
    run.step(&GameAction::Move(heart)).unwrap();
}

fn kill_heart(run:&mut Run){
    for _ in 0..4{
        m9_boss::hit(run,0,1_000);
        if run.state.combat.as_ref().unwrap().public_full().unwrap().outcome==Some(Outcome::Victory){break}
        run.choose_combat(CombatAction::EndTurn).unwrap();
    }
    assert_eq!(run.state.combat.as_ref().unwrap().public_full().unwrap().outcome,Some(Outcome::Victory));
}

#[test]
fn m10_act4_elite_reward_must_close_before_actual_heart_admission(){
    let mut run=run_at_act4();enter_elite(&mut run);
    assert_eq!((run.state.act,run.state.floor),(4,53));
    let act4_map=run.state.map.as_ref().unwrap();
    let elite_node=act4_map.nodes.iter().find(|node|Some(node.id)==act4_map.current).unwrap();
    assert_eq!((act4_map.act,elite_node.floor,elite_node.kind,elite_node.encounter,elite_node.burning),
        (4,53,MapNodeKind::Elite,Some(EncounterId::ShieldAndSpear),false));
    assert_eq!(run.state.active_combat.as_ref().unwrap().origin,CombatOrigin::MapRoom{node:run.public_map().unwrap().current.unwrap(),kind:MapNodeKind::Elite});
    assert!(run.state.combat.as_ref().unwrap().item_context().unwrap().battle.elite_trigger);
    assert!(run.state.combat.as_ref().unwrap().item_context().unwrap().relics().contains(&RelicId::SlaversCollar));
    let heart=run.public_map().unwrap().nodes[3].id;let frozen=run.clone();
    assert_eq!(run.step(&GameAction::Move(heart)),Err(RunError::InvalidAction));assert_eq!(run,frozen);
    let mut one_partner=run.decision().unwrap().combat.unwrap();
    let first_live=one_partner.enemies.iter().position(|enemy|enemy.hp>0).unwrap();
    m9_boss::hit(&mut run,first_live,10_000);
    let decision=run.decision().unwrap();
    assert_eq!(decision.combat.as_ref().unwrap().outcome,None,"one defeated partner does not settle the pair source or publish its rewards");
    assert_eq!(decision.combat.as_ref().unwrap().enemies.iter().filter(|enemy|enemy.hp>0).count(),1);
    assert_eq!(decision.room.unwrap().kind,RoomViewKind::Combat);
    let half_resolved=run.clone();assert_eq!(run.step(&GameAction::Move(heart)),Err(RunError::InvalidAction));assert_eq!(run,half_resolved);
    one_partner=run.decision().unwrap().combat.unwrap();
    let survivor=one_partner.enemies.iter().position(|enemy|enemy.hp>0).unwrap();m9_boss::hit(&mut run,survivor,10_000);
    assert_eq!(run.state.combat.as_ref().unwrap().public_full().unwrap().outcome,Some(Outcome::Victory));
    let settlement=run.settle_combat().unwrap();
    assert_eq!(settlement.terminal,None);
    assert_eq!(run.state.combat_progress.settled,run.state.combat_progress.entered);
    let reward=run.decision().unwrap().room.unwrap();assert_eq!(reward.kind,RoomViewKind::Rewards);
    let captured_cards=&run.state.room_loop.as_ref().unwrap().active_rewards.as_ref().unwrap().cards;
    let card_capture=captured_cards.choices.iter().filter_map(|row|match row.kind{
        RoomChoiceKind::Card{id,..}=>Some((id,Catalog::standard().card(id).unwrap().rarity)),_=>None,
    }).collect::<Vec<_>>();
    assert_eq!(card_capture,vec![
        (CardId::TrueGrit,CardRarity::Common),
        (CardId::RecklessCharge,CardRarity::Uncommon),
        (CardId::FireBreathing,CardRarity::Uncommon),
    ],"these are the exact captured rows for this deterministic supplied Act4 source, not a standard reward-pool parity claim");
    let gold=reward.choices.iter().find_map(|row|if let RoomChoiceKind::Gold(value)=row.kind{Some(value)}else{None}).unwrap();
    assert!((25..=35).contains(&gold),"Act4 Shield/Spear uses the source-qualified elite gold range");
    let relic_row=reward.choices.iter().find(|row|row.kind==RoomChoiceKind::Relic(RelicId::Lantern)).unwrap();
    assert_eq!(relic_row.status,RoomChoiceStatus::Offered,"the actual finite-stock relic capture is claimable and unowned");
    assert!(reward.choices.iter().any(|row|matches!(row.kind,RoomChoiceKind::Potion(_))&&row.status==RoomChoiceStatus::Disabled),"a full belt does not publish a claimable fourth potion");
    assert!(!reward.choices.iter().any(|row|row.kind==RoomChoiceKind::Emerald),"non-burning Act4 Shield/Spear does not offer the Emerald key");
    assert!(run.state.room_loop.as_ref().unwrap().active_rewards.is_some());
    let cursor=run.state.room_loop.as_ref().unwrap().m10_elite_relic_cursor;
    assert_eq!(cursor,1,"the first real M10 Act4 elite capture offers owned-eligible stock at sequence index zero and advances its publication cursor once");
    let potion_row=reward.choices.iter().find(|row|matches!(row.kind,RoomChoiceKind::Potion(_))).unwrap();
    let RoomChoiceKind::Potion(potion)=potion_row.kind else{unreachable!()};
    assert_eq!(potion,crate::content::PotionId::BloodPotion,"the first actual Act4 combat visit is captured as the first M10 potion offer");
    assert_eq!(potion_row.status,RoomChoiceStatus::Disabled);
    let gold_before=run.state.gold;choose(&mut run,RoomChoiceKind::Gold(gold));assert_eq!(run.state.gold,gold_before+gold);
    let parent_proceed=run.decision().unwrap().room.unwrap().choices.iter().find(|row|row.kind==RoomChoiceKind::Proceed).unwrap().id;
    choose(&mut run,RoomChoiceKind::CardReward);
    let card_child=run.decision().unwrap().room.unwrap();assert_eq!(card_child.kind,RoomViewKind::CardReward);
    assert_eq!(card_child.choices.iter().map(|row|row.kind.clone()).collect::<Vec<_>>(),vec![
        RoomChoiceKind::Card{id:CardId::TrueGrit,upgrades:0},
        RoomChoiceKind::Card{id:CardId::RecklessCharge,upgrades:0},
        RoomChoiceKind::Card{id:CardId::FireBreathing,upgrades:0},
        RoomChoiceKind::Skip,
    ],"the actual child exposes the captured Act4 elite identities, in capture order");
    let child_state=run.clone();assert_eq!(run.step(&GameAction::Choose(parent_proceed)),Err(RunError::InvalidAction));assert_eq!(run,child_state);
    choose(&mut run,RoomChoiceKind::Skip);
    run.step(&GameAction::DiscardPotion{slot:0}).unwrap();
    let available=run.decision().unwrap().room.unwrap().choices.into_iter().find(|row|row.kind==RoomChoiceKind::Potion(potion)).unwrap();
    assert_eq!(available.status,RoomChoiceStatus::Offered);
    choose(&mut run,RoomChoiceKind::Potion(potion));
    let after_claim=run.decision().unwrap().room.unwrap();
    assert_eq!(after_claim.choices.iter().find(|row|row.kind==RoomChoiceKind::Potion(potion)).unwrap().status,RoomChoiceStatus::Claimed);
    assert_eq!(run.state.potions.iter().filter(|slot|slot.is_some()).count(),3,"discard then claim restores one real potion in the owned Run inventory");
    assert!(run.state.potions.iter().flatten().any(|id|*id==potion));
    choose(&mut run,RoomChoiceKind::Proceed);
    assert_eq!(run.decision().unwrap().room.unwrap().kind,RoomViewKind::Map);
    assert!(run.state.combat.is_none()&&run.state.active_combat.is_none());
    assert_eq!(run.public_map().unwrap().current,Some(run.public_map().unwrap().nodes[2].id));
    assert_eq!(run.state.room_loop.as_ref().unwrap().m10_elite_relic_cursor,cursor,"an offered relic stock advances once when the reward owner publishes, even when the optional offer is skipped");
    assert!(run.state.relics.contains(&RelicId::SlaversCollar));
}

#[test]
fn m10_act4_invalid_accepted_pair_source_retains_real_combat_and_first_fault(){
    let mut run=run_at_act4();enter_elite(&mut run);defeat_pair(&mut run);
    let node=run.public_map().unwrap().current.unwrap();
    run.state.map.as_mut().unwrap().nodes.iter_mut().find(|entry|entry.id==node).unwrap().burning=true;
    let original=run.clone();let error=run.settle_combat().unwrap_err();
    let RunError::Faulted(first)=error else{panic!("accepted combat source corruption must be retained")};
    assert_eq!(first.checkpoint,"run.m10.act4.settlement");
    assert_eq!(run.state.fault,Some(first.clone()));assert_eq!(run.state.floor,original.state.floor);
    assert!(run.state.combat.is_some()&&run.state.active_combat.is_some());
    assert_eq!(run.state.combat.as_ref().unwrap().public_full().unwrap().outcome,Some(Outcome::Victory));
    assert_eq!(run.state.combat.as_ref().unwrap().item_context().unwrap().relics().contains(&RelicId::SlaversCollar),true);
    assert_eq!(run.state.combat_progress.settled,original.state.combat_progress.settled);
    assert!(run.state.room_loop.as_ref().unwrap().active_rewards.is_none());
    let frozen=run.clone();assert_eq!(run.settle_combat(),Err(RunError::Faulted(first)));assert_eq!(run,frozen);
}

#[test]
fn m10_act4_failed_census_preparation_keeps_the_actual_victory_combat_and_items(){
    let mut run=run_at_act4();enter_elite(&mut run);defeat_pair(&mut run);
    run.state.deck.pop();let damaged=run.clone();
    let error=run.settle_combat().unwrap_err();
    let RunError::Faulted(first)=error else{panic!("invalid live-card census after an accepted victory must retain a Run fault")};
    assert_eq!(first.checkpoint,"run.combat.settlement");
    assert_eq!(run.state.combat,damaged.state.combat);assert_eq!(run.state.active_combat,damaged.state.active_combat);
    assert!(run.state.combat.as_ref().unwrap().item_context().is_some());
    assert_eq!(run.state.combat_progress.settled,damaged.state.combat_progress.settled);
    let frozen=run.clone();assert_eq!(run.settle_combat(),Err(RunError::Faulted(first)));assert_eq!(run,frozen);
}

#[test]
fn m10_act4_native_entry_fault_retains_actual_combat_and_owned_inventory(){
    let mut run=run_at_act4();
    let rest=run.public_map().unwrap().nodes[0].id;run.step(&GameAction::Move(rest)).unwrap();
    let rest_choice=run.decision().unwrap().room.unwrap().choices.into_iter().find(|row|matches!(row.kind,RoomChoiceKind::Rest{..})).unwrap().id;
    run.step(&GameAction::Choose(rest_choice)).unwrap();
    let shop=run.public_map().unwrap().nodes[1].id;run.step(&GameAction::Move(shop)).unwrap();
    let leave=run.decision().unwrap().room.unwrap().choices.into_iter().find(|row|row.kind==RoomChoiceKind::Leave).unwrap().id;
    run.step(&GameAction::Choose(leave)).unwrap();
    crate::items::acquire_relic(&mut run.state,RelicId::MarkOfPain).unwrap();
    run.state.deck=(1..=998).map(|instance|CardInstance{instance,id:crate::content::CardId::Strike,upgrades:0}).collect();
    run.state.next_instance=999;
    assert_eq!(run.state.deck.len(),998);
    assert_eq!(run.state.deck.iter().map(|card|card.instance).collect::<std::collections::BTreeSet<_>>().len(),998);
    assert_eq!(run.state.deck.iter().map(|card|card.instance).max(),Some(998));
    assert!(run.state.relics.contains(&RelicId::MarkOfPain));
    let elite=run.public_map().unwrap().nodes[2].id;
    let error=run.step(&GameAction::Move(elite)).unwrap_err();
    let RunError::Faulted(first)=error else{panic!("the real native setup frontier must remain an accepted fault")};
    match &first.kind {
        RunFaultKind::Combat(fault)=>{
            assert_eq!(fault.record.message,"native generated-card support/count/target","the accepted native fault is Mark of Pain's real temporary Wound at the full issued-ID frontier");
            assert!(matches!(fault.current.as_ref().map(|effect|&effect.kind),Some(crate::combat::EffectKind::CreateCard{id:crate::content::CardId::Wound,destination:crate::combat::CardDestination::ShuffledDraw,upgrades:0})),"the retained source frame is Mark of Pain's qualified post-opening temporary Wound creation");
        },
        other=>panic!("expected Mark of Pain's owned combat generation fault, got {other:?}"),
    }
    assert!(run.state.combat.is_some()&&run.state.active_combat.is_some());
    assert!(run.state.combat.as_ref().unwrap().item_context().unwrap().relics().contains(&RelicId::MarkOfPain));
    assert_eq!(run.state.deck.len(),998,"temporary Mark of Pain Wounds do not change the permanent source census");
    assert_eq!(run.state.deck.iter().map(|card|card.instance).collect::<std::collections::BTreeSet<_>>().len(),998);
    let frozen=run.clone();assert_eq!(run.step(&GameAction::Move(elite)),Err(RunError::Faulted(first)));assert_eq!(run,frozen);
}

#[test]
fn m10_heart_victory_keeps_proceed_and_true_victory_room_as_separate_real_transitions(){
    let mut run=run_at_act4();enter_heart(&mut run);
    assert_eq!(run.state.active_combat.as_ref().unwrap().origin,CombatOrigin::MapRoom{node:run.public_map().unwrap().current.unwrap(),kind:MapNodeKind::Boss});
    let deck_before=run.state.deck.clone();let relics_before=run.state.combat.as_ref().unwrap().item_context().unwrap().relics().to_vec();
    for _ in 0..4{
        m9_boss::hit(&mut run,0,1_000);
        if run.state.combat.as_ref().unwrap().public_full().unwrap().outcome==Some(Outcome::Victory){break}
        run.choose_combat(CombatAction::EndTurn).unwrap();
    }
    assert_eq!(run.state.combat.as_ref().unwrap().public_full().unwrap().outcome,Some(Outcome::Victory));
    let finished=run.settle_combat().unwrap();
    assert_eq!(finished.terminal,None);
    assert_eq!(run.state.terminal,None,"Heart death bookkeeping is not the Run terminal until the explicit Proceed transition");
    assert_eq!(run.state.floor,54);assert_eq!(run.state.room,"Boss");
    assert_eq!(run.state.deck,deck_before);assert_eq!(run.state.relics,relics_before);
    assert_eq!(run.state.combat_progress.settled,run.state.combat_progress.entered);
    assert_eq!(run.state.reward_ledger.total,run.decision().unwrap().reward_total);
    let screen=run.decision().unwrap().room.unwrap();assert_eq!(screen.kind,RoomViewKind::HeartVictory);
    assert_eq!(screen.choices.iter().map(|row|row.kind.clone()).collect::<Vec<_>>(),vec![RoomChoiceKind::Proceed]);
    let proceed=screen.choices[0].id;let before=run.clone();
    assert_eq!(run.step(&GameAction::Choose(ChoiceId(proceed.0+1))),Err(RunError::InvalidAction));assert_eq!(run,before,"only the actual published Heart Proceed token may enter TrueVictoryRoom");
    let result=run.step(&GameAction::Choose(proceed)).unwrap();
    assert_eq!((result.reward,result.terminal),(110,Some(RunTerminal::HeartVictory)));
    assert_eq!(run.state.floor,55);assert_eq!(run.state.room,"TrueVictoryRoom");
    assert_eq!(run.decision().unwrap().room.unwrap(),RoomView{kind:RoomViewKind::TrueVictory,choices:vec![]});
    assert_eq!(run.state.reward_ledger.total,before.state.reward_ledger.total+110);
    assert_eq!(run.state.history.events.iter().filter(|event|matches!(event,CommittedEvent::FloorEntered{act:4,floor:55})).count(),1);
    assert_eq!(run.state.history.events.iter().filter(|event|matches!(event,CommittedEvent::Ended(RunTerminal::HeartVictory))).count(),1);
    assert_eq!(run.state.room_loop.as_ref().unwrap().act4.as_ref().unwrap().heart_proceed_phase,Some(super::super::rooms::act4::HeartProceedPhase::Accepted));
    let terminal=run.clone();assert_eq!(run.step(&GameAction::Choose(proceed)),Err(RunError::Terminal));assert_eq!(run,terminal);
}

#[test]
fn m10_heart_feed_max_hp_gain_survives_run_settlement_and_proceed(){
    let mut run=run_at_act4();complete_elite_rewards_for_supplied_heart_entry(&mut run);
    run.state.deck=vec![CardInstance{instance:1,id:crate::content::CardId::Feed,upgrades:0}];run.state.next_instance=2;
    run.state.max_hp=300;run.state.hp=300;
    let heart=run.public_map().unwrap().nodes[3].id;run.step(&GameAction::Move(heart)).unwrap();
    m9_boss::hit(&mut run,0,1_000);run.choose_combat(CombatAction::EndTurn).unwrap();
    m9_boss::hit(&mut run,0,1_000);run.choose_combat(CombatAction::EndTurn).unwrap();
    m9_boss::hit(&mut run,0,147);
    let feed=run.decision().unwrap().candidates.iter().find_map(|candidate|match &candidate.action{
        GameAction::Combat(action @ CombatAction::PlayCard{instance:1,target:Some(crate::combat::TargetId(1))})=>Some(action.clone()),_=>None
    }).expect("the actual Feed instance is legally offered against the live Heart");
    let permanent_before=run.state.deck.clone();let next_instance_before=run.state.next_instance;
    run.choose_combat(feed).unwrap();
    assert_eq!(run.state.terminal,None);assert_eq!(run.state.floor,54);
    assert_eq!(run.state.max_hp,303);
    assert!(run.state.hp>0&&run.state.hp<=run.state.max_hp,"the actual Feed/Heart/Burning Blood settlement leaves a valid same-Run HP value; direct exact Feed heal is covered by the focused fatal-Feed resolver regression");
    assert_eq!(run.state.deck,permanent_before,"Feed does not manufacture a new permanent deck identity");
    assert_eq!(run.state.next_instance,next_instance_before);
    assert_eq!(run.state.active_combat,None);assert_eq!(run.state.combat,None);
    let result=choose(&mut run,RoomChoiceKind::Proceed);
    assert_eq!(result.terminal,Some(RunTerminal::HeartVictory));
    assert_eq!(run.state.max_hp,303,"the Heart Proceed transition cannot replay or discard the accepted Feed max-HP gain");
}

#[test]
fn m10_true_victory_reward_preflight_rejects_overflow_without_accepting_proceed(){
    let mut run=run_at_act4();enter_heart(&mut run);
    kill_heart(&mut run);
    run.settle_combat().unwrap();
    run.state.reward_ledger.total=i32::MAX;
    let proceed=run.decision().unwrap().room.unwrap().choices[0].id;let before=run.clone();
    assert_eq!(run.step(&GameAction::Choose(proceed)),Err(RunError::Unsupported("reward total overflow")));
    assert_eq!(run,before,"failed final-output preflight is before the accepted proceed boundary");
    assert_eq!(run.state.room_loop.as_ref().unwrap().act4.as_ref().unwrap().heart_proceed_phase,Some(super::super::rooms::act4::HeartProceedPhase::AwaitingProceed));
}

#[test]
fn m10_heart_proceed_rejects_a_real_but_wrong_act4_combat_record_atomically(){
    let mut run=run_at_act4();enter_heart(&mut run);kill_heart(&mut run);run.settle_combat().unwrap();
    let prior_pair=run.state.room_loop.as_ref().unwrap().act4.as_ref().unwrap().elite_record.clone().unwrap();
    assert_eq!((prior_pair.encounter,prior_pair.outcome),(EncounterId::ShieldAndSpear,Outcome::Victory));
    let current_visit=run.state.room_loop.as_ref().unwrap().act4.as_ref().unwrap().heart_record.as_ref().unwrap().visit;
    let mut wrong_source=prior_pair.clone();wrong_source.visit=current_visit;
    assert_eq!(wrong_source.visit,current_visit,"keep the accepted Heart visit correct so the source identity is the discriminating defect");
    assert_eq!(wrong_source.visit.checked_add(1),Some(run.state.combat_progress.settled));
    let matching_spoof_event=CommittedEvent::CombatFinished(wrong_source.clone());
    assert!(!run.state.history.events.contains(&matching_spoof_event));
    // Deliberately inject a matching duplicate event to satisfy exact membership;
    // this is corrupted-state discrimination, not a normal-play history claim.
    run.state.history.events.push(matching_spoof_event.clone());
    run.state.room_loop.as_mut().unwrap().act4.as_mut().unwrap().heart_record=Some(wrong_source.clone());
    run.state.last_combat=Some(wrong_source);
    let progress=run.state.room_loop.as_ref().unwrap().act4.as_ref().unwrap();
    assert!(run.state.history.events.contains(&CommittedEvent::CombatFinished(prior_pair)),"the real Shield/Spear event remains to satisfy the independent elite-source guard");
    assert!(run.state.history.events.contains(&matching_spoof_event),"the deliberately corrupted history contains the exact spoof payload");
    assert_eq!(progress.heart_record.as_ref(),run.state.last_combat.as_ref());
    assert_eq!(progress.phase,super::super::rooms::act4::Act4TransitionPhase::Published);
    assert_eq!(progress.combat_visits,2);
    assert!(progress.elite_rewards_closed);
    assert_eq!(progress.source_record.encounter,EncounterId::TimeEater);
    assert_eq!(progress.source_record.outcome,Outcome::Victory);
    assert!(run.state.history.events.contains(&CommittedEvent::CombatFinished(progress.source_record.clone())));
    assert!(run.state.history.events.contains(&CommittedEvent::ActAdvanced{from:3,to:4}));
    assert_eq!(progress.heart_proceed_phase,Some(super::super::rooms::act4::HeartProceedPhase::AwaitingProceed));
    let map=run.state.map.as_ref().unwrap();
    assert_eq!((map.act,map.current,map.boss_icon),(4,Some(progress.route[3]),Some(EncounterId::CorruptHeart)));
    assert!(map.nodes.iter().any(|node|node.id==progress.route[3]&&node.floor==54&&node.visited&&node.kind==MapNodeKind::Boss&&node.encounter==Some(EncounterId::CorruptHeart)));
    assert_eq!((run.state.act,run.state.floor,run.state.act4_enabled),(4,54,true));
    assert_eq!(run.state.terminal,None);
    assert_eq!(run.state.fault,None);
    assert_eq!(run.state.room_loop.as_ref().unwrap().scenario,RoomScenario::M10IroncladHeart);
    assert!(run.state.combat.is_none()&&run.state.active_combat.is_none()&&run.state.pending.is_none());
    let heart_screen=run.decision().unwrap().room.unwrap();
    assert_eq!(heart_screen.kind,RoomViewKind::HeartVictory);
    assert_eq!(heart_screen.choices.len(),1);
    assert_eq!(heart_screen.choices[0].kind,RoomChoiceKind::Proceed);
    assert_eq!(heart_screen.choices[0].status,RoomChoiceStatus::Offered);
    assert_eq!(progress.heart_proceed,Some(heart_screen.choices[0].id));
    let proceed=run.decision().unwrap().room.unwrap().choices[0].id;let before=run.clone();
    assert_eq!(run.step(&GameAction::Choose(proceed)),Err(RunError::InvalidAction));
    assert_eq!(run,before,"a real earlier Shield/Spear victory cannot impersonate the actual Heart source or publish floor55/HeartVictory");
}

#[test]
fn m10_heart_proceed_preparation_overflow_retains_the_terminal_combat_and_inventory(){
    let mut run=run_at_act4();enter_heart(&mut run);kill_heart(&mut run);
    run.next_choice=u64::MAX;
    let before=run.clone();let settled=run.state.combat_progress.settled;
    let error=run.settle_combat().unwrap_err();
    let RunError::Faulted(first)=error else{panic!("failed detached Heart Proceed preparation after a real victory must retain a fault")};
    assert_eq!(first.checkpoint,"run.m10.heart.victory");assert_eq!(first.committed_events,before.state.history.events,"the retained fault owns only the existing prefix, with no fabricated Heart combat publication");
    assert_eq!(run.state.fault,Some(first.clone()));assert_eq!(run.next_choice,u64::MAX);
    assert_eq!(run.state.combat,before.state.combat);assert_eq!(run.state.active_combat,before.state.active_combat);
    assert!(run.state.combat.as_ref().unwrap().item_context().is_some());
    assert_eq!(run.state.combat_progress.settled,settled);
    assert!(!run.state.history.events.iter().any(|event|matches!(event,CommittedEvent::CombatFinished(record) if record.encounter==EncounterId::CorruptHeart)));
    let frozen=run.clone();assert_eq!(run.settle_combat(),Err(RunError::Faulted(first)));assert_eq!(run,frozen);
}

#[test]
fn m10_act4_real_pair_death_is_one_run_death_without_elite_reward_or_heart_admission(){
    let mut run=run_at_act4();enter_elite_at_hp(&mut run,1);
    let deck=run.state.deck.clone();let relics=run.state.combat.as_ref().unwrap().item_context().unwrap().relics().to_vec();
    let reward_before=run.state.reward_ledger.total;
    for _ in 0..8{if run.state.terminal.is_some(){break}run.choose_combat(CombatAction::EndTurn).unwrap();}
    assert_eq!(run.state.terminal,Some(RunTerminal::Death));
    assert_eq!(run.state.reward_ledger.total,reward_before-1);
    assert_eq!(run.state.history.events.iter().filter(|event|matches!(event,CommittedEvent::Ended(RunTerminal::Death))).count(),1);
    assert_eq!(run.state.deck,deck);assert_eq!(run.state.relics,relics);
    assert!(run.state.room_loop.as_ref().unwrap().active_rewards.is_none());
    assert!(run.state.room_loop.as_ref().unwrap().act4.as_ref().unwrap().elite_record.is_none());
    assert_eq!(run.state.room_loop.as_ref().unwrap().act4.as_ref().unwrap().elite_rewards_closed,false);
}

#[test]
fn m10_heart_native_death_is_not_promoted_to_heart_victory(){
    let mut run=run_at_act4();
    // Complete the genuine pair route first, then enter the actual Heart with a
    // supplied low-HP checkpoint; no public reset/resource profile is changed.
    complete_elite_rewards_for_supplied_heart_entry(&mut run);
    run.state.hp=1;let deck=run.state.deck.clone();let relics=run.state.relics.clone();let potions=run.state.potions.clone();
    let heart=run.public_map().unwrap().nodes[3].id;run.step(&GameAction::Move(heart)).unwrap();
    for _ in 0..8{if run.state.terminal.is_some(){break}run.choose_combat(CombatAction::EndTurn).unwrap();}
    assert_eq!(run.state.terminal,Some(RunTerminal::Death));
    assert_eq!(run.state.floor,54);assert_eq!(run.state.deck,deck);assert_eq!(run.state.relics,relics);assert_eq!(run.state.potions,potions);
    assert_eq!(run.state.room_loop.as_ref().unwrap().screen.kind,RoomViewKind::Map);
    assert!(run.state.room_loop.as_ref().unwrap().act4.as_ref().unwrap().heart_record.is_none());
    assert!(!run.state.history.events.iter().any(|event|matches!(event,CommittedEvent::Ended(RunTerminal::HeartVictory))));
    assert!(!run.state.history.events.iter().any(|event|matches!(event,CommittedEvent::FloorEntered{act:4,floor:55})));
}

fn supplied_m10_chain_start()->Run{
    let mut profile=CreationProfile::experienced();profile.neow_eligible=false;
    profile.seen_bosses[0]=vec!["Hexaghost","SlimeBoss"];
    profile.seen_bosses[1]=vec!["Champ","Collector"];
    profile.seen_bosses[2]=vec!["AwakenedOne","DonuAndDeca"];
    Run::new_room_scenario(
        RunConfig{character:crate::content::Character::Ironclad,ascension:0},profile,
        RunRng::new([0x18;32],crate::combat::rng::Partition::Test,0),RoomScenario::M10IroncladHeart,
    ).unwrap()
}

fn supplied_move_to(run:&mut Run,node:NodeId){
    super::m7_rooms::before(run,node);
    let decision=run.decision().unwrap();
    assert!(decision.candidates.iter().any(|candidate|candidate.action==GameAction::Move(node)),
        "supplied node {node:?} must be a currently offered map edge; screen {:?}, current {:?}, candidates {:?}",
        decision.room.as_ref().map(|room|room.kind),run.state.map.as_ref().unwrap().current,decision.candidates.iter().map(|candidate|candidate.action.clone()).collect::<Vec<_>>());
    run.step(&GameAction::Move(node)).unwrap();
}

fn finish_supplied_combat(run:&mut Run){
    for _ in 0..8{
        let Some(combat)=run.decision().unwrap().combat else{break};
        if combat.outcome==Some(Outcome::Victory){break}
        let Some(slot)=combat.enemies.iter().position(|enemy|enemy.hp>0)else{break};
        super::m9_boss::hit(run,slot,10_000);
    }
    assert_eq!(run.state.combat.as_ref().unwrap().public_full().unwrap().outcome,Some(Outcome::Victory));
    run.settle_combat().unwrap();
}

fn finish_public_masked_combat(run:&mut Run){
    for _ in 0..64{
        let decision=run.decision().unwrap();
        if decision.combat.is_none(){break}
        let action=decision.candidates.iter().find_map(|candidate|match &candidate.action{
            GameAction::Combat(CombatAction::PlayCard{target:Some(_),..})=>Some(candidate.action.clone()),
            _=>None,
        }).unwrap_or(GameAction::Combat(CombatAction::EndTurn));
        run.step(&action).unwrap();
    }
    assert_eq!(run.state.last_combat.as_ref().unwrap().outcome,Outcome::Victory);
}

fn close_supplied_boss_reward(run:&mut Run,relic:RelicId){
    choose(run,RoomChoiceKind::Proceed);
    assert_eq!(run.decision().unwrap().room.unwrap().kind,RoomViewKind::BossTreasure);
    choose(run,RoomChoiceKind::OpenChest);
    assert_eq!(run.decision().unwrap().room.unwrap().kind,RoomViewKind::BossRelicChoice);
    choose(run,RoomChoiceKind::Relic(relic));
    assert_eq!(run.decision().unwrap().room.unwrap().kind,RoomViewKind::BossRelicChild);
    choose(run,RoomChoiceKind::Proceed);
}

#[test]
fn supplied_m10_chain_executes_real_key_producers_fourth_choice_act4_sources_and_heart_proceed(){
    // Supplied native source-path diagnostic only: the three actual Act1 source
    // nodes are positioned with the existing valid-edge fixture helper, and the
    // burning elite uses legal public combat actions. This is not a normal-start
    // route/win claim; no key flag, enemy result or terminal is injected.
    let mut run=supplied_m10_chain_start();
    choose(&mut run,RoomChoiceKind::NeowLament);

    let rest=run.state.map.as_ref().unwrap().nodes.iter().find(|node|node.kind==MapNodeKind::Rest).unwrap().id;
    supplied_move_to(&mut run,rest);
    assert_eq!(run.decision().unwrap().room.unwrap().kind,RoomViewKind::Campfire);
    let ruby=choose(&mut run,RoomChoiceKind::Recall);
    assert_eq!(ruby.reward,5);assert!(run.state.keys[0]);

    let chest=run.state.map.as_ref().unwrap().nodes.iter().find(|node|node.kind==MapNodeKind::Treasure).unwrap().id;
    supplied_move_to(&mut run,chest);
    assert_eq!(run.decision().unwrap().room.unwrap().kind,RoomViewKind::Treasure);
    choose(&mut run,RoomChoiceKind::OpenChest);
    let sapphire=choose(&mut run,RoomChoiceKind::Sapphire);
    assert_eq!(sapphire.reward,5);assert!(run.state.keys[1]);
    assert!(run.decision().unwrap().room.as_ref().unwrap().choices.iter().any(|row|row.kind==RoomChoiceKind::Proceed&&row.status==RoomChoiceStatus::Offered));
    choose(&mut run,RoomChoiceKind::Proceed);

    let burning=run.state.map.as_ref().unwrap().nodes.iter().find(|node|node.kind==MapNodeKind::Elite&&node.burning).unwrap().id;
    supplied_move_to(&mut run,burning);
    assert_eq!(run.state.active_combat.as_ref().unwrap().origin,CombatOrigin::MapRoom{node:burning,kind:MapNodeKind::Elite});
    finish_public_masked_combat(&mut run);
    assert_eq!(run.state.last_combat.as_ref().unwrap().outcome,Outcome::Victory);
    assert_eq!(run.state.map.as_ref().unwrap().nodes.iter().find(|node|node.id==burning).unwrap().burning,true);
    choose(&mut run,RoomChoiceKind::CardReward);
    choose(&mut run,RoomChoiceKind::Skip);
    let emerald=choose(&mut run,RoomChoiceKind::Emerald);
    assert_eq!(emerald.reward,5);assert!(run.state.keys[2]);
    assert!(run.state.history.events.contains(&CommittedEvent::KeyAcquired(Key::Ruby)));
    assert!(run.state.history.events.contains(&CommittedEvent::KeyAcquired(Key::Sapphire)));
    assert!(run.state.history.events.contains(&CommittedEvent::KeyAcquired(Key::Emerald)));
    assert_eq!(run.state.keys,[true;3]);
    choose(&mut run,RoomChoiceKind::Proceed);

    let act1_boss=run.state.map.as_ref().unwrap().nodes.iter().find(|node|node.kind==MapNodeKind::Boss).unwrap().id;
    supplied_move_to(&mut run,act1_boss);finish_supplied_combat(&mut run);
    close_supplied_boss_reward(&mut run,RelicId::SlaversCollar);
    assert_eq!((run.state.act,run.state.floor),(2,17));

    let act2_boss=run.state.map.as_ref().unwrap().nodes.iter().find(|node|node.kind==MapNodeKind::Boss).unwrap().id;
    supplied_move_to(&mut run,act2_boss);finish_supplied_combat(&mut run);
    close_supplied_boss_reward(&mut run,RelicId::BlackBlood);
    assert_eq!((run.state.act,run.state.floor),(3,34));

    let act3_boss=run.state.map.as_ref().unwrap().nodes.iter().find(|node|node.kind==MapNodeKind::Boss).unwrap().id;
    supplied_move_to(&mut run,act3_boss);finish_supplied_combat(&mut run);
    assert_eq!(run.state.last_combat.as_ref().unwrap().encounter,EncounterId::TimeEater);
    choose(&mut run,RoomChoiceKind::Proceed);
    for stage in 1..=3{choose(&mut run,RoomChoiceKind::Act3Continue{stage});}
    assert_eq!(run.decision().unwrap().room.as_ref().unwrap().choices[0].kind,RoomChoiceKind::Act3Continue{stage:4});
    choose(&mut run,RoomChoiceKind::Act3Continue{stage:4});
    assert_eq!(run.state.act,4);
    let act4=run.public_map().unwrap();
    assert_eq!(act4.nodes.iter().map(|node|(node.floor,node.kind)).collect::<Vec<_>>(),vec![
        (51,MapNodeKind::Rest),(52,MapNodeKind::Shop),(53,MapNodeKind::Elite),(54,MapNodeKind::Boss),
    ]);

    let rest=act4.nodes[0].id;run.step(&GameAction::Move(rest)).unwrap();
    assert_eq!(run.decision().unwrap().room.unwrap().kind,RoomViewKind::Campfire);
    let campfire=run.decision().unwrap().room.unwrap();
    if let Some(row)=campfire.choices.iter().find(|row|matches!(row.kind,RoomChoiceKind::Rest{..})&&row.status==RoomChoiceStatus::Offered){
        run.step(&GameAction::Choose(row.id)).unwrap();
    }else{
        choose(&mut run,RoomChoiceKind::Smith);
        let action=run.decision().unwrap().candidates.iter().find(|candidate|candidate.object.is_some()).unwrap().action.clone();
        run.step(&action).unwrap();
    }
    let shop=run.public_map().unwrap().nodes[1].id;run.step(&GameAction::Move(shop)).unwrap();
    assert_eq!(run.decision().unwrap().room.unwrap().kind,RoomViewKind::Shop);
    choose(&mut run,RoomChoiceKind::Leave);
    let elite=run.public_map().unwrap().nodes[2].id;run.step(&GameAction::Move(elite)).unwrap();
    assert_eq!(run.state.active_combat.as_ref().unwrap().encounter,EncounterId::ShieldAndSpear);
    finish_supplied_combat(&mut run);
    assert!(!run.decision().unwrap().room.unwrap().choices.iter().any(|row|row.kind==RoomChoiceKind::Emerald));
    if run.decision().unwrap().room.as_ref().unwrap().choices.iter().any(|row|row.kind==RoomChoiceKind::CardReward&&row.status==RoomChoiceStatus::Offered){
        choose(&mut run,RoomChoiceKind::CardReward);choose(&mut run,RoomChoiceKind::Skip);
    }
    choose(&mut run,RoomChoiceKind::Proceed);
    let heart=run.public_map().unwrap().nodes[3].id;run.step(&GameAction::Move(heart)).unwrap();
    assert_eq!(run.state.active_combat.as_ref().unwrap().encounter,EncounterId::CorruptHeart);
    for _ in 0..4{
        if run.state.combat.as_ref().unwrap().public_full().unwrap().outcome==Some(Outcome::Victory){break}
        super::m9_boss::hit(&mut run,0,1_000);
        if run.state.combat.as_ref().unwrap().public_full().unwrap().outcome!=Some(Outcome::Victory){
            run.choose_combat(CombatAction::EndTurn).unwrap();
        }
    }
    assert_eq!(run.state.combat.as_ref().unwrap().public_full().unwrap().outcome,Some(Outcome::Victory));
    run.settle_combat().unwrap();
    assert_eq!(run.decision().unwrap().room.unwrap().kind,RoomViewKind::HeartVictory);
    let final_proceed=run.decision().unwrap().room.unwrap().choices[0].id;
    let final_transition=run.step(&GameAction::Choose(final_proceed)).unwrap();
    assert_eq!((final_transition.reward,final_transition.terminal),(110,Some(RunTerminal::HeartVictory)));
    assert_eq!((run.state.act,run.state.floor,run.state.room),(4,55,"TrueVictoryRoom".into()));
    assert_eq!(run.state.room_loop.as_ref().unwrap().act4.as_ref().unwrap().heart_record.as_ref().unwrap().encounter,EncounterId::CorruptHeart);
    println!("SUPPLIED native diagnostic: real Recall/Ruby, chest/Sapphire, burning-elite/Emerald, all-key fourth choice, fixed Act4 rooms, actual Shield/Spear and Heart, final Proceed→floor55/HeartVictory; not normal-start evidence.");
}

fn fresh_m10_reward_policy_run()->Run{
    let mut profile=CreationProfile::experienced();profile.neow_eligible=false;
    Run::new_room_scenario(
        RunConfig{character:crate::content::Character::Ironclad,ascension:0},profile,
        RunRng::new([0x29;32],crate::combat::rng::Partition::Test,0),RoomScenario::M10IroncladHeart,
    ).unwrap()
}

#[test]
fn m10_supplied_reward_policy_rows_sum_to_215_only_after_floor_55_and_heart_victory(){
    // This characterizes the existing typed-event reward policy only. These
    // supplied rows do not demonstrate route traversal, key-source eligibility,
    // or an actual Heart victory; those source boundaries have separate tests.
    let mut keys_only=fresh_m10_reward_policy_run();
    assert_eq!(keys_only.state.reward_ledger.total,0);
    assert_eq!(super::super::reward::commit(&mut keys_only.state,&[]).unwrap(),0);
    assert!(keys_only.state.history.events.is_empty());
    for (key,total) in [(Key::Ruby,5),(Key::Sapphire,10),(Key::Emerald,15)]{
        let event=CommittedEvent::KeyAcquired(key);
        assert_eq!(super::super::reward::commit(&mut keys_only.state,std::slice::from_ref(&event)).unwrap(),5);
        assert_eq!(keys_only.state.reward_ledger.total,total);
        assert_eq!(super::super::reward::commit(&mut keys_only.state,&[event]).unwrap(),0,"a repeated key kind is not credited twice");
        assert_eq!(keys_only.state.reward_ledger.total,total);
    }
    assert_eq!(keys_only.state.reward_ledger.total,15,"distinct key rows alone do not receive a route or ending bonus");
    assert!(!keys_only.state.history.events.iter().any(|event|matches!(event,CommittedEvent::Ended(_))));

    let mut run=fresh_m10_reward_policy_run();
    // Qualified transition annotations: Act 1 rows include its floor-17 boss
    // treasure; Act 2 starts with floor 18 and includes floor-34 treasure;
    // Act 3 runs from floor 35 through its floor-50 boss.
    let mut expected_total=0;
    for (act,first,last,stage_total) in [(1,1,17,17),(2,18,34,34),(3,35,50,50)]{
        for floor in first..=last{
            let event=CommittedEvent::FloorEntered{act,floor};
            assert_eq!(super::super::reward::commit(&mut run.state,std::slice::from_ref(&event)).unwrap(),1,"Act {act} floor {floor}");
            expected_total+=1;
            assert_eq!(run.state.reward_ledger.total,expected_total);
        }
        assert_eq!(run.state.reward_ledger.total,stage_total);
        let repeated=CommittedEvent::FloorEntered{act,floor:last};
        assert_eq!(super::super::reward::commit(&mut run.state,&[repeated]).unwrap(),0,"a repeated floor is not credited twice");
        assert_eq!(run.state.reward_ledger.total,stage_total);
    }
    assert_eq!(run.state.reward_ledger.total,50,"floors 1–50 contribute exactly 50");

    for (key,total) in [(Key::Ruby,55),(Key::Sapphire,60),(Key::Emerald,65)]{
        let event=CommittedEvent::KeyAcquired(key);
        assert_eq!(super::super::reward::commit(&mut run.state,&[event]).unwrap(),5);
        assert_eq!(run.state.reward_ledger.total,total);
    }
    assert_eq!(run.state.reward_ledger.total,65);

    // Act 4 floors 51–54 are real room-entry annotations; floor 55 is emitted
    // only by the separate TrueVictoryRoom Proceed source. No ending row is
    // supplied before that final floor event.
    for (floor,total) in [(51,75),(52,85),(53,95),(54,105)]{
        let event=CommittedEvent::FloorEntered{act:4,floor};
        assert_eq!(super::super::reward::commit(&mut run.state,&[event]).unwrap(),10);
        assert_eq!(run.state.reward_ledger.total,total);
        assert!(!run.state.history.events.iter().any(|event|matches!(event,CommittedEvent::Ended(_))));
    }
    assert_eq!(run.state.reward_ledger.total,105);
    let final_floor=CommittedEvent::FloorEntered{act:4,floor:55};
    assert_eq!(super::super::reward::commit(&mut run.state,std::slice::from_ref(&final_floor)).unwrap(),10);
    assert_eq!(run.state.reward_ledger.total,115);
    assert!(!run.state.history.events.iter().any(|event|matches!(event,CommittedEvent::Ended(_))));
    assert_eq!(super::super::reward::commit(&mut run.state,&[final_floor]).unwrap(),0,"floor 55 is also credited once");
    assert_eq!(run.state.reward_ledger.total,115);

    let heart_victory=CommittedEvent::Ended(RunTerminal::HeartVictory);
    assert_eq!(super::super::reward::commit(&mut run.state,std::slice::from_ref(&heart_victory)).unwrap(),100);
    assert_eq!(run.state.reward_ledger.total,215);

    for (terminal,expected) in [(RunTerminal::Death,-1),(RunTerminal::EmptyDeck,-1),(RunTerminal::Act3NoKeys,0)]{
        let mut control=fresh_m10_reward_policy_run();
        let event=CommittedEvent::Ended(terminal);
        assert_eq!(super::super::reward::commit(&mut control.state,std::slice::from_ref(&event)).unwrap(),expected);
        assert_eq!(control.state.reward_ledger.total,expected);
        assert_eq!(control.state.history.events,vec![event]);
    }
}
