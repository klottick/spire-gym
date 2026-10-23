#[path="../examples/m7_support/mod.rs"] mod support;
use spire_core::{combat::Outcome,content::{EncounterId,EventId,RelicId},run::*};

#[test]
fn three_normal_start_seed11_paths_resolve_act2_boss_rewards_into_act3() {
    for (seen,boss) in [(vec!["Automaton","Collector"],EncounterId::Champ),(vec!["Champ","Collector"],EncounterId::BronzeAutomaton),(vec!["Champ","Automaton"],EncounterId::Collector)] {
        let initial=support::start(11,seen.clone()).unwrap().decision().unwrap();
        assert_eq!((initial.player.hp,initial.player.max_hp,initial.player.gold,initial.objects.len()),(80,80,99,10));
        assert_eq!(initial.player.relics,vec![RelicId::BurningBlood]);assert!(initial.player.potions.iter().all(Option::is_none));assert_eq!(initial.player.keys,[false;3]);
        let r=support::complete(11,seen,4096).unwrap();let d=&r.final_decision;
        assert_eq!((d.act,d.floor,d.player.hp>0,d.terminal,d.reward_total),(3,34,true,None,34));
        assert_eq!(d.content_profile,Some(RoomScenario::M7Act2Captured));assert_eq!(d.room.as_ref().unwrap().kind,RoomViewKind::Act3Ready);assert!(d.candidates.is_empty());
        let act2=r.combats.iter().filter(|(act,_)|*act==2).map(|(_,r)|r).collect::<Vec<_>>();
        assert!(act2.iter().any(|r|r.encounter==boss&&r.outcome==Outcome::Victory));
        assert!(act2.iter().filter(|r|matches!(r.encounter,EncounterId::SphericGuardian|EncounterId::ThreeByrds|EncounterId::CenturionAndHealer|EncounterId::Snecko|EncounterId::ChosenAndByrds|EncounterId::ThreeCultists)&&r.outcome==Outcome::Victory).count()>=2);
        assert!(act2.iter().any(|r|matches!(r.encounter,EncounterId::GremlinLeader|EncounterId::Slavers|EncounterId::BookOfStabbing)&&r.outcome==Outcome::Victory));
        assert!(r.events.contains(&EventId::AncientWriting));
        assert!(r.trace.iter().any(|s|s.decision.act==2&&s.decision.floor==34&&s.decision.room.as_ref().unwrap().kind==RoomViewKind::BossTreasure&&s.reward==1));
        assert!(r.trace.iter().all(|s|s.decision.floor<=34));assert_eq!(r.trace.last().unwrap().reward,0);
        let parent=&r.trace[r.trace.len()-2].decision;
        assert_eq!((parent.act,parent.floor),(2,34));assert_eq!(parent.objects,d.objects);
        assert_eq!((&parent.player.relics,&parent.player.potions,&parent.player.relic_counters,parent.player.gold,parent.player.keys),(&d.player.relics,&d.player.potions,&d.player.relic_counters,d.player.gold,d.player.keys));
    }
}

#[test]
fn m7_public_replay_second_seed_and_stale_ready_actions() {
    for seed in [7,11] {
        let seen=vec!["Automaton","Collector"];let r=support::complete(seed,seen.clone(),4096).unwrap();let mut replay=support::start(seed,seen).unwrap();
        for s in &r.trace {let t=replay.step(&s.action).unwrap();assert_eq!(t.reward,s.reward);assert_eq!(replay.decision().unwrap(),s.decision);assert_eq!(replay.public_map(),s.map);}
        let d=replay.decision().unwrap();
        let expected=if seed==7{(2,33,Some(RunTerminal::Death),32)}else{(3,34,None,34)};
        assert_eq!((d.act,d.floor,d.terminal,d.reward_total),expected);assert_eq!(d,r.final_decision);
        let old=replay.decision().unwrap();let old_map=replay.public_map();assert!(replay.step(&r.trace.last().unwrap().action).is_err());assert_eq!(replay.decision().unwrap(),old);assert_eq!(replay.public_map(),old_map);
    }
}

#[test]
fn m7_seed7_natural_boss_loss_is_valid_not_a_failed_simulation() {
    for (seen,boss,terminal) in [(vec!["Automaton","Collector"],EncounterId::Champ,Some(RunTerminal::Death)),(vec!["Champ","Collector"],EncounterId::BronzeAutomaton,None),(vec!["Champ","Automaton"],EncounterId::Collector,None)] {
        let r=support::complete(7,seen,4096).unwrap();let d=&r.final_decision;
        assert_eq!(d.terminal,terminal);assert!(d.candidates.is_empty());
        if terminal==Some(RunTerminal::Death) {
            assert_eq!((d.act,d.floor,d.player.hp,d.reward_total),(2,33,0,32));
            assert!(r.combats.iter().any(|(act,c)|*act==2&&c.encounter==boss&&c.outcome==Outcome::Death));
            assert_eq!(r.trace.last().unwrap().reward,-1);
            assert!(!r.trace.iter().any(|s|s.decision.act==3||s.decision.floor==34));
            assert_ne!(d.room.as_ref().unwrap().kind,RoomViewKind::Act3Ready);
        } else {
            assert_eq!((d.act,d.floor,d.reward_total),(3,34,34));assert!(d.player.hp>0);
            assert_eq!(d.room.as_ref().unwrap().kind,RoomViewKind::Act3Ready);
            assert!(r.combats.iter().any(|(act,c)|*act==2&&c.encounter==boss&&c.outcome==Outcome::Victory));
        }
    }
}

#[test]
fn m7_real_death_and_budget_are_not_completion() {
    let r=support::death(7,4096).unwrap();let d=r.final_decision;
    assert_eq!(d.content_profile,Some(RoomScenario::M7Act2Captured));assert_eq!((d.terminal,d.player.hp),(Some(RunTerminal::Death),0));assert!(d.candidates.is_empty());
    assert!(r.combats.iter().any(|(_,c)|c.outcome==Outcome::Death));assert_eq!(r.trace.last().unwrap().reward,-1);
    assert!(r.trace.iter().all(|s|s.decision.act!=3));
    for budget in [0,1] {let r=support::complete(7,vec!["Automaton","Collector"],budget).unwrap();assert_eq!(r.trace.len(),budget);assert_eq!(r.final_decision.terminal,None);assert_ne!(r.final_decision.room.as_ref().unwrap().kind,RoomViewKind::Act3Ready);}
}

#[test]
fn m7_public_maps_do_not_reveal_future_offers_or_actor_rng() {
    let r=support::complete(7,vec!["Champ","Automaton"],4096).unwrap();
    for s in r.trace.iter().filter(|s|s.decision.room.as_ref().is_some_and(|r|matches!(r.kind,RoomViewKind::Map|RoomViewKind::Act3Ready))) {
        assert!(s.decision.offers.is_empty());assert!(s.decision.room.as_ref().unwrap().choices.is_empty());
        assert!(s.map.as_ref().unwrap().nodes.iter().filter(|n|!n.visited).all(|n|n.resolved_room.is_none()));
    }
    let map=r.final_map.unwrap();assert_eq!(map.act,3);assert!(map.current.is_none());assert!(map.nodes.iter().all(|n|!n.visited));
}

#[test]
fn m7_campfire_prioritizes_earned_bludgeon_upgrade() {
    let r=support::complete(7,vec!["Champ","Automaton"],4096).unwrap();
    let (index,step)=r.trace.iter().enumerate().find(|(_,s)|s.decision.act==2&&s.decision.selection.is_some()&&s.decision.room.as_ref().unwrap().kind==RoomViewKind::Campfire&&s.decision.candidates.iter().any(|c|c.object.as_ref().is_some_and(|o|o.id==spire_core::content::CardId::Bludgeon&&o.upgrades==0))).expect("actual earned Bludgeon is offered to Smith on this public route");
    let wanted=&step.decision.candidates.iter().find(|c|c.object.as_ref().is_some_and(|o|o.id==spire_core::content::CardId::Bludgeon)).unwrap().action;
    assert_eq!(&r.trace[index+1].action,wanted,"spend the real upgrade on the earned high-impact attack before low-impact damage");
}

#[test]
fn m7_public_policy_reserves_one_defense_then_affordable_bludgeon() {
    use spire_core::{combat::{Combat,CombatAction,CardInstanceId,PlayerState,permanent_live_card,rng::Partition,enemies::{self,SetupContext}},content::{CardId,Character}};
    for (energy,hp) in [(4,30),(3,8)] {
        let entry=enemies::setup(EncounterId::JawWorm,1,0,&mut RunRng::new([41;32],Partition::Test,0),SetupContext{player:PlayerState{hp,max_hp:80,block:0,energy,max_energy:energy,draw_per_turn:3,powers:vec![]},player_powers:vec![],character:Character::Ironclad,cards:[CardId::Bludgeon,CardId::Defend,CardId::Clothesline].into_iter().enumerate().map(|(i,id)|permanent_live_card(id,CardInstanceId(i as u64+1),0)).collect(),opening_draw:3,encounter_visit:0}).unwrap();
        let mut c=Combat::enter(entry).unwrap();
        if energy==4 {
            let a=support::combat_action(&c.public_full().unwrap(),&c.legal_actions_full().unwrap());
            assert_eq!(a,CombatAction::PlayCard{instance:2,target:None},"reserve the spare energy for defense, not the two-energy competing attack");
            c.step_full(&a).unwrap();let a=support::combat_action(&c.public_full().unwrap(),&c.legal_actions_full().unwrap());
            assert!(matches!(a,CombatAction::PlayCard{instance:1,..}),"do not spend the reserved three energy on another smaller play");
        } else {
            let a=support::combat_action(&c.public_full().unwrap(),&c.legal_actions_full().unwrap());
            assert!(!matches!(a,CombatAction::PlayCard{instance:1,..}),"a nonlethal all-energy attack cannot replace affordable survival work");
        }
    }
}

#[test]
fn m7_public_policy_plays_free_anger_before_paid_draw() {
    use spire_core::{combat::{Combat,CombatAction,CardInstanceId,PlayerState,permanent_live_card,rng::Partition,enemies::{self,SetupContext}},content::{CardId,Character}};
    let entry=enemies::setup(EncounterId::JawWorm,1,0,&mut RunRng::new([41;32],Partition::Test,0),SetupContext{player:PlayerState{hp:30,max_hp:80,block:0,energy:3,max_energy:3,draw_per_turn:3,powers:vec![]},player_powers:vec![],character:Character::Ironclad,cards:[CardId::Anger,CardId::ShrugItOff,CardId::Strike].into_iter().enumerate().map(|(i,id)|permanent_live_card(id,CardInstanceId(i as u64+1),0)).collect(),opening_draw:3,encounter_visit:0}).unwrap();
    let c=Combat::enter(entry).unwrap();let view=c.public_full().unwrap();
    let a=support::combat_action(&view,&c.legal_actions_full().unwrap());
    assert!(matches!(a,CombatAction::PlayCard{instance:1,..}),"free damage/copy should precede a paid draw that can reshuffle the discard");
}
