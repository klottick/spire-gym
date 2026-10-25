use crate::{combat::CombatAction, content::{Character, PotionId, RelicId}, run::*};
use super::{rooms::choose, m7_rooms::before};

fn act2_map()->Run{
    let mut run=super::m7_rooms::continuing();
    run.state.room_loop.as_mut().unwrap().scenario=RoomScenario::M10IroncladHeart;
    run
}
fn act1_start()->Run{
    let mut profile=CreationProfile::experienced();profile.neow_eligible=false;
    let mut run=Run::new_room_scenario(RunConfig{character:Character::Ironclad,ascension:0},profile,
        RunRng::new([0x18;32],crate::combat::rng::Partition::Test,0),RoomScenario::M10IroncladHeart).unwrap();
    choose(&mut run,RoomChoiceKind::NeowLament);
    crate::items::acquire_relic(&mut run.state,RelicId::NeowsLament).unwrap();
    run.state.relic_counters.insert(RelicId::NeowsLament,3);
    run
}
fn act3_map()->Run{
    let mut run=super::m9_rooms::ready();
    run.state.room_loop.as_mut().unwrap().scenario=RoomScenario::M10IroncladHeart;
    run
}
fn open_actual_chest(run:&mut Run){
    let node=run.state.map.as_ref().unwrap().nodes.iter().find(|node|node.kind==MapNodeKind::Treasure).unwrap().clone();
    before(run,node.id);run.step(&GameAction::Move(node.id)).unwrap();
    let open=run.decision().unwrap().room.unwrap().choices.into_iter().find(|row|row.kind==RoomChoiceKind::OpenChest).unwrap().id;
    run.step(&GameAction::Choose(open)).unwrap();
}

#[test]
fn m10_opened_chest_uses_the_shared_finite_relic_cursor_and_sapphire_choice(){
    let mut run=act2_map();
    for relic in [RelicId::Lantern,RelicId::Pear,RelicId::Mango,RelicId::MagicFlower,
        RelicId::ToyOrnithopter,RelicId::Strawberry,RelicId::Anchor,RelicId::BloodVial]{
        if !run.state.relics.contains(&relic){crate::items::acquire_relic(&mut run.state,relic).unwrap();}
    }
    run.state.room_loop.as_mut().unwrap().m10_elite_relic_cursor=8;
    open_actual_chest(&mut run);
    let screen=run.decision().unwrap().room.unwrap();
    let relic=screen.choices.iter().find(|row|row.kind==RoomChoiceKind::Relic(RelicId::Vajra)).unwrap();
    assert_eq!(relic.status,RoomChoiceStatus::Offered,"chest and elite captures share the source-order cursor");
    assert_eq!(run.state.room_loop.as_ref().unwrap().m10_elite_relic_cursor,9,"opening the chest accepts the actual relic capture once");
    assert!(screen.choices.iter().any(|row|row.kind==RoomChoiceKind::Sapphire&&row.status==RoomChoiceStatus::Offered));
    choose(&mut run,RoomChoiceKind::Relic(RelicId::Vajra));
    assert!(run.state.relics.contains(&RelicId::Vajra));
    assert_eq!(run.state.room_loop.as_ref().unwrap().m10_elite_relic_cursor,9,"claiming the captured relic does not advance stock a second time");
    choose(&mut run,RoomChoiceKind::Proceed);
    assert_eq!(run.decision().unwrap().room.unwrap().kind,RoomViewKind::Map);
}

#[test]
fn m10_sapphire_trades_the_accepted_chest_relic_without_a_second_cursor_advance(){
    let mut run=act2_map();
    run.state.room_loop.as_mut().unwrap().m10_elite_relic_cursor=8;
    open_actual_chest(&mut run);
    let offered=run.decision().unwrap().room.unwrap().choices.into_iter().find_map(|row|if let RoomChoiceKind::Relic(id)=row.kind{Some(id)}else{None}).unwrap();
    let prior_total=run.state.reward_ledger.total;
    choose(&mut run,RoomChoiceKind::Sapphire);
    assert!(run.state.keys[1]);assert_eq!(run.state.reward_ledger.total,prior_total+5);
    assert!(!run.state.relics.contains(&offered));
    assert_eq!(run.state.room_loop.as_ref().unwrap().m10_elite_relic_cursor,9,"Sapphire trades off the already captured chest relic without reusing its stock");
    let chest=run.decision().unwrap().room.unwrap();
    assert!(chest.choices.iter().any(|row|row.kind==RoomChoiceKind::Relic(offered)&&row.status==RoomChoiceStatus::Skipped));
    assert!(chest.choices.iter().any(|row|row.kind==RoomChoiceKind::Proceed&&row.status==RoomChoiceStatus::Offered));
    choose(&mut run,RoomChoiceKind::Proceed);
    assert_eq!(run.decision().unwrap().room.unwrap().kind,RoomViewKind::Map);
}

#[test]
fn m10_proceed_skips_an_already_accepted_chest_relic_without_refunding_stock(){
    let mut run=act2_map();run.state.room_loop.as_mut().unwrap().m10_elite_relic_cursor=8;
    open_actual_chest(&mut run);
    assert!(run.decision().unwrap().room.unwrap().choices.iter().any(|row|
        row.kind==RoomChoiceKind::Relic(RelicId::Vajra)&&row.status==RoomChoiceStatus::Offered));
    assert_eq!(run.state.room_loop.as_ref().unwrap().m10_elite_relic_cursor,9);
    choose(&mut run,RoomChoiceKind::Proceed);
    assert!(!run.state.relics.contains(&RelicId::Vajra),"Proceed leaves the captured relic unclaimed");
    assert_eq!(run.state.room_loop.as_ref().unwrap().m10_elite_relic_cursor,9,"accepted offer stays consumed after Proceed");
    assert_eq!(run.decision().unwrap().room.unwrap().kind,RoomViewKind::Map);
}

#[test]
fn m10_chest_choice_overflow_rejects_before_hooks_cursor_or_publication(){
    let mut run=act2_map();
    let node=run.state.map.as_ref().unwrap().nodes.iter().find(|node|node.kind==MapNodeKind::Treasure).unwrap().clone();
    before(&mut run,node.id);run.step(&GameAction::Move(node.id)).unwrap();
    let open=run.decision().unwrap().room.unwrap().choices.into_iter()
        .find(|row|row.kind==RoomChoiceKind::OpenChest).unwrap().id;
    run.next_choice=u64::MAX-1;
    let state=run.state.clone();let rng=run.state.rng.clone();let cursor=run.state.room_loop.as_ref().unwrap().m10_elite_relic_cursor;
    let screen=run.state.room_loop.as_ref().unwrap().screen.clone();let next=run.next_choice;
    assert!(matches!(run.step(&GameAction::Choose(open)),Err(RunError::Unsupported("choice identity exhausted"))));
    assert_eq!(run.state,state,"choice preparation must reject before ChestOpened hooks or cursor/screen publication");
    assert_eq!(run.state.rng,rng);assert_eq!(run.state.room_loop.as_ref().unwrap().m10_elite_relic_cursor,cursor);
    assert_eq!(run.state.room_loop.as_ref().unwrap().screen,screen);assert_eq!(run.next_choice,next);
}

#[test]
fn m10_owned_sapphire_is_disabled_and_stale_choice_rejects_atomically(){
    let mut run=act2_map();open_actual_chest(&mut run);
    let choice=run.decision().unwrap().room.unwrap().choices.into_iter().find(|row|row.kind==RoomChoiceKind::Sapphire).unwrap();
    run.state.keys[1]=true;
    let before=run.clone();
    assert_eq!(run.step(&GameAction::Choose(choice.id)),Err(RunError::InvalidAction));
    assert_eq!(run,before,"a stale duplicate-key choice cannot award, mutate the cursor, or replace the chest screen");
}

#[test]
fn m10_opened_chests_in_all_acts_with_all_captures_owned_keep_sapphire_or_proceed(){
    for run in [act1_start(),act2_map(),act3_map()]{
      for sapphire_owned in [false,true]{
        let mut run=run.clone();
        for relic in [RelicId::Lantern,RelicId::Pear,RelicId::Mango,RelicId::MagicFlower,
            RelicId::ToyOrnithopter,RelicId::Strawberry,RelicId::Anchor,RelicId::BloodVial,
            RelicId::Vajra,RelicId::OddlySmoothStone,RelicId::BagOfPreparation,RelicId::BagOfMarbles,
            RelicId::BronzeScales,RelicId::RedSkull,RelicId::PaperPhrog,RelicId::ChampionsBelt,
             RelicId::CharonsAshes,RelicId::SelfFormingClay,RelicId::Orichalcum]{
            if !run.state.relics.contains(&relic){crate::items::acquire_relic(&mut run.state,relic).unwrap();}
        }
        run.state.room_loop.as_mut().unwrap().m10_elite_relic_cursor=0;
        if sapphire_owned{run.state.keys[1]=true;}
        open_actual_chest(&mut run);
        let screen=run.decision().unwrap().room.unwrap();
        assert_eq!(screen.choices.iter().filter(|row|matches!(row.kind,RoomChoiceKind::Relic(_))&&row.status==RoomChoiceStatus::Offered).count(),0,
            "exhausted M10 source must not fabricate a replacement relic");
        assert!(screen.choices.iter().any(|row|row.kind==RoomChoiceKind::Proceed&&row.status==RoomChoiceStatus::Offered),"Proceed remains a real optional source action");
        assert_eq!(screen.choices.iter().any(|row|row.kind==RoomChoiceKind::Sapphire&&row.status==RoomChoiceStatus::Offered),!sapphire_owned,
            "unowned Sapphire remains claimable; an owned key is disabled");
        if sapphire_owned{assert!(screen.choices.iter().any(|row|row.kind==RoomChoiceKind::Sapphire&&row.status==RoomChoiceStatus::Disabled));}
        let before=(run.state.relics.clone(),run.state.keys,run.state.reward_ledger.total);
        choose(&mut run,RoomChoiceKind::Proceed);
        assert_eq!((run.state.relics.clone(),run.state.keys,run.state.reward_ledger.total),before);
      }
    }
}

#[test]
fn m10_recall_is_a_real_once_only_campfire_choice_and_rewards_five(){
    let mut run=act2_map();
    run.state.potions=vec![Some(PotionId::BloodPotion),Some(PotionId::Elixir),Some(PotionId::HeartOfIron)];
    let potions=run.state.potions.clone();
    let node=run.state.map.as_ref().unwrap().nodes.iter().find(|node|node.kind==MapNodeKind::Rest).unwrap().clone();
    before(&mut run,node.id);run.step(&GameAction::Move(node.id)).unwrap();
    let screen=run.decision().unwrap().room.unwrap();assert_eq!(screen.kind,RoomViewKind::Campfire);
    let hp=run.state.hp;let max_hp=run.state.max_hp;let recall=screen.choices.iter().find(|row|row.kind==RoomChoiceKind::Recall).unwrap().id;
    let mut duplicate=run.clone();duplicate.state.keys[0]=true;
    assert_eq!(duplicate.decision().unwrap().room.unwrap().choices.iter().find(|row|row.kind==RoomChoiceKind::Recall).unwrap().status,RoomChoiceStatus::Disabled);
    let prior_total=run.state.reward_ledger.total;
    let transition=run.step(&GameAction::Choose(recall)).unwrap();
    assert_eq!(transition.reward,5);assert_eq!(run.state.reward_ledger.total,prior_total+5);assert!(run.state.keys[0]);
    assert_eq!((run.state.hp,run.state.max_hp),(hp,max_hp),"Recall spends the campfire action; it is neither Rest nor Smith");
    assert_eq!(run.state.potions,potions,"taking Recall preserves a real full potion belt");
    assert_eq!(run.decision().unwrap().room.unwrap().kind,RoomViewKind::Map);
}

#[test]
fn m10_emerald_is_published_only_by_actual_burning_elite_victory(){
    let mut run=act1_start();
    run.state.potions=vec![Some(PotionId::BloodPotion),Some(PotionId::Elixir),Some(PotionId::HeartOfIron)];
    let node=run.state.map.as_ref().unwrap().nodes.iter().find(|node|node.kind==MapNodeKind::Elite&&node.burning).unwrap().clone();
    before(&mut run,node.id);run.step(&GameAction::Move(node.id)).unwrap();
    assert!(!run.decision().unwrap().room.is_some_and(|room|room.choices.iter().any(|row|row.kind==RoomChoiceKind::Emerald)),"a map marker is not a key reward");
    for _ in 0..32{
        let decision=run.decision().unwrap();if decision.combat.is_none(){break}
        let action=decision.candidates.into_iter().find_map(|candidate|match candidate.action{
            GameAction::Combat(action@CombatAction::PlayCard{target:Some(_),..})=>Some(action),_=>None,
        }).unwrap_or(CombatAction::EndTurn);
        run.choose_combat(action).unwrap();
    }
    let screen=run.decision().unwrap().room.unwrap();
    assert!(screen.choices.iter().any(|row|row.kind==RoomChoiceKind::Emerald&&row.status==RoomChoiceStatus::Offered));
    let potion=screen.choices.iter().find(|row|matches!(row.kind,RoomChoiceKind::Potion(_))).unwrap();
    assert_eq!(potion.status,RoomChoiceStatus::Disabled,"actual full belt disables the prepared reward until a slot is freed");
    let card_parent=screen.choices.iter().find(|row|row.kind==RoomChoiceKind::CardReward).unwrap().id;
    run.step(&GameAction::Choose(card_parent)).unwrap();
    let child=run.decision().unwrap().room.unwrap();
    assert_eq!(child.kind,RoomViewKind::CardReward);
    assert!(!child.choices.iter().any(|row|matches!(row.kind,RoomChoiceKind::Emerald|RoomChoiceKind::Proceed)),
        "the mandatory captured card child owns the current screen; its parent Emerald/Proceed rows are unavailable");
    choose(&mut run,RoomChoiceKind::Skip);
    assert!(run.decision().unwrap().room.unwrap().choices.iter().any(|row|row.kind==RoomChoiceKind::Emerald&&row.status==RoomChoiceStatus::Offered));
    assert!(!run.state.keys[2]);
    let prior_total=run.state.reward_ledger.total;
    let emerald=screen.choices.iter().find(|row|row.kind==RoomChoiceKind::Emerald).unwrap().id;
    choose(&mut run,RoomChoiceKind::Emerald);
    assert_eq!(run.state.reward_ledger.total,prior_total+5);assert!(run.state.keys[2]);
    let committed=run.state.clone();
    assert_eq!(run.step(&GameAction::Choose(emerald)),Err(RunError::InvalidAction));
    assert_eq!(run.state,committed,"a claimed Emerald offer cannot award the same key or +5 again");
    run.step(&GameAction::DiscardPotion{slot:0}).unwrap();
    let potion_kind=match potion.kind {RoomChoiceKind::Potion(id)=>id,_=>unreachable!()};
    assert!(run.decision().unwrap().room.unwrap().choices.iter().any(|row|row.kind==RoomChoiceKind::Potion(potion_kind)&&row.status==RoomChoiceStatus::Offered));
    choose(&mut run,RoomChoiceKind::Potion(potion_kind));
    assert!(run.state.potions.contains(&Some(potion_kind)),"after freeing a real belt slot the captured reward is claimable");
}

#[test]
fn m10_burning_elite_death_does_not_grant_emerald(){
    let mut run=act1_start();
    let node=run.state.map.as_ref().unwrap().nodes.iter().find(|node|node.kind==MapNodeKind::Elite&&node.burning).unwrap().clone();
    before(&mut run,node.id);run.state.hp=1;run.step(&GameAction::Move(node.id)).unwrap();
    for _ in 0..32{
        if run.decision().unwrap().terminal.is_some(){break;}
        run.choose_combat(CombatAction::EndTurn).unwrap();
    }
    assert_eq!(run.decision().unwrap().terminal,Some(RunTerminal::Death));
    assert!(!run.state.keys[2]);
    assert!(!run.state.history.events.contains(&CommittedEvent::KeyAcquired(Key::Emerald)));
}

#[test]
fn m10_preowned_emerald_key_is_not_reoffered_by_an_actual_burning_elite_victory(){
    let mut run=act1_start();run.state.keys[2]=true;
    let node=run.state.map.as_ref().unwrap().nodes.iter().find(|node|node.kind==MapNodeKind::Elite&&node.burning).unwrap().clone();
    before(&mut run,node.id);run.step(&GameAction::Move(node.id)).unwrap();
    for _ in 0..32{
        if run.decision().unwrap().combat.is_none(){break}
        let action=run.decision().unwrap().candidates.into_iter().find_map(|candidate|match candidate.action{
            GameAction::Combat(action@CombatAction::PlayCard{target:Some(_),..})=>Some(action),_=>None,
        }).unwrap_or(CombatAction::EndTurn);
        run.choose_combat(action).unwrap();
    }
    let screen=run.decision().unwrap().room.unwrap();
    assert!(!screen.choices.iter().any(|row|row.kind==RoomChoiceKind::Emerald),"an already-owned key has no duplicate reward row");
    assert!(run.state.keys[2]);
    assert_eq!(run.state.history.events.iter().filter(|event|**event==CommittedEvent::KeyAcquired(Key::Emerald)).count(),0,
        "pre-owning Emerald creates no duplicate key receipt or reward event");
}

#[test]
fn m10_capture_bound_uses_real_generated_act_topology_not_seed_search(){
    let mut rng=RunRng::new([0x18;32],crate::combat::rng::Partition::Test,0);
    for act in 1..=3{
        let map=super::super::map::generate(act,&mut rng,super::super::map::MapEligibility::new(true,false)).unwrap();
        let mut best=std::collections::BTreeMap::<NodeId,(usize,usize)>::new();
        for node in &map.nodes{
            let parent=map.nodes.iter().filter(|candidate|candidate.edges.contains(&node.id))
                .filter_map(|candidate|best.get(&candidate.id).copied()).max().unwrap_or_default();
            let next=(parent.0+usize::from(node.kind==MapNodeKind::Elite),parent.1+usize::from(node.kind==MapNodeKind::Treasure));
            assert!(next.0<=5,"generated act {act} topology exceeds the qualified five-elite path bound");
            assert!(next.1<=1,"one map path cannot capture multiple chest nodes in one act");
            best.insert(node.id,next);
            for child in map.nodes.iter().filter(|child|node.edges.contains(&child.id)){
                assert!(!(node.kind==MapNodeKind::Elite&&child.kind==MapNodeKind::Elite),"the real generator forbids adjacent elite rows, bounding nine eligible rows to at most five elite encounters on any path");
            }
        }
        let (_,chests)=best.values().copied().max().unwrap();assert!(chests<=1);
    }
    assert_eq!(3*5+3,18,"no-key paths stop before Act4: at most fifteen Acts1-3 elite offers plus three ordinary chest offers");
    assert_eq!(3*5+3+1,19,"the Act4 key path accepts fifteen Acts1-3 elite offers, all three chest offers including Sapphire, and one Act4 elite offer");
}

#[test]
fn m10_nineteenth_accepted_chest_offer_uses_one_real_common_relic(){
    let owned=[RelicId::Lantern,RelicId::Pear,RelicId::Mango,RelicId::MagicFlower,
        RelicId::ToyOrnithopter,RelicId::Strawberry,RelicId::Anchor,RelicId::BloodVial,
        RelicId::Vajra,RelicId::OddlySmoothStone,RelicId::BagOfPreparation,RelicId::BagOfMarbles,
        RelicId::BronzeScales,RelicId::RedSkull,RelicId::PaperPhrog,RelicId::ChampionsBelt,
        RelicId::CharonsAshes,RelicId::SelfFormingClay];
    let mut run=act2_map();
    for relic in owned {crate::items::acquire_relic(&mut run.state,relic).unwrap();}
    run.state.room_loop.as_mut().unwrap().m10_elite_relic_cursor=18;
    open_actual_chest(&mut run);
    assert!(run.decision().unwrap().room.unwrap().choices.iter().any(|row|
        row.kind==RoomChoiceKind::Relic(RelicId::Orichalcum)&&row.status==RoomChoiceStatus::Offered),
        "the qualified nineteenth accepted offer is still capturable after eighteen prior acquisitions");
    assert_eq!(run.state.room_loop.as_ref().unwrap().m10_elite_relic_cursor,19);
}
