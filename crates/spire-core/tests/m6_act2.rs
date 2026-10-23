#[path="../examples/m6_support/mod.rs"]mod support;
use spire_core::{combat::Outcome,content::{EncounterId,EventId},run::{GameAction,RoomScenario,RoomViewKind,RunTerminal}};

#[test]
fn three_normal_start_seed7_profiles_reach_real_act2_prefix(){
    for (seen,elite) in [(vec!["Automaton","Collector"],EncounterId::GremlinLeader),(vec!["Champ","Collector"],EncounterId::Slavers),(vec!["Champ","Automaton"],EncounterId::BookOfStabbing)]{
        let r=support::prefix(7,seen,4096).unwrap();let d=&r.final_decision;assert_eq!((d.act,d.floor,d.player.hp>0,d.terminal,d.reward_total),(2,26,true,None,26));assert_eq!(d.content_profile,Some(RoomScenario::M6Act2Prefix));assert_eq!(d.room.as_ref().unwrap().kind,RoomViewKind::Act2PrefixComplete);assert!(d.candidates.is_empty());
        let act2=r.combats.iter().filter(|(act,_)|*act==2).map(|(_,r)|r).collect::<Vec<_>>();assert!(act2.iter().any(|r|r.encounter==elite&&r.outcome==Outcome::Victory));assert!(act2.iter().filter(|r|matches!(r.encounter,EncounterId::SphericGuardian|EncounterId::ThreeByrds|EncounterId::CenturionAndHealer|EncounterId::Snecko|EncounterId::ChosenAndByrds|EncounterId::ThreeCultists)&&r.outcome==Outcome::Victory).count()>=2);assert!(r.events.contains(&EventId::AncientWriting));assert!(r.trace.iter().any(|s|matches!(s.action,GameAction::Choose(_))&&s.decision.room.as_ref().is_some_and(|v|v.kind==RoomViewKind::Treasure)&&s.decision.floor==26));assert!(r.trace.iter().all(|s|s.decision.act<=2&&s.decision.floor<=26));
    }
}

#[test]
fn replay_public_steps_rewards_stale_and_second_seed(){
    for seed in [7,11]{let seen=vec!["Automaton","Collector"];let r=support::prefix(seed,seen.clone(),4096).unwrap();let mut replay=support::start(seed,seen).unwrap();for step in &r.trace{
        let t=replay.step(&step.action).unwrap();assert_eq!(t.reward,step.reward);assert_eq!(replay.decision().unwrap(),step.decision);assert_eq!(replay.public_map(),step.map);
    }assert_eq!(replay.decision().unwrap(),r.final_decision);assert_eq!(r.final_decision.room.as_ref().unwrap().kind,RoomViewKind::Act2PrefixComplete);let snapshot=replay.decision().unwrap();if let Some(step)=r.trace.last(){assert!(replay.step(&step.action).is_err());assert_eq!(replay.decision().unwrap(),snapshot);}}
}

#[test]
fn real_death_and_budget_are_not_prefix_completion(){
    let r=support::death(7,4096).unwrap();assert_eq!(r.final_decision.terminal,Some(RunTerminal::Death));assert_eq!(r.final_decision.player.hp,0);assert!(r.combats.iter().any(|(_,r)|r.outcome==Outcome::Death));assert!(r.final_decision.candidates.is_empty());
    for budget in [0,1]{let r=support::prefix(7,vec!["Automaton","Collector"],budget).unwrap();assert_eq!(r.trace.len(),budget);assert_ne!(r.final_decision.room.as_ref().unwrap().kind,RoomViewKind::Act2PrefixComplete);assert_eq!(r.final_decision.terminal,None);}
}

#[test]
fn supplied_resources_diagnostics_use_actual_six_encounters_and_exit(){
    let reports=support::encounters(7,4096).unwrap();assert_eq!(reports.len(),6);for encounter in [EncounterId::GremlinLeader,EncounterId::Slavers,EncounterId::BookOfStabbing,EncounterId::Champ,EncounterId::Collector,EncounterId::BronzeAutomaton]{
        let r=reports.iter().find(|r|r.encounter==encounter).unwrap();assert_eq!(r.exit.outcome,Outcome::Victory);assert_eq!(r.view.outcome,Some(Outcome::Victory));assert!(!r.actions.is_empty());assert!(r.actions.iter().any(|a|matches!(a,spire_core::combat::CombatAction::EndTurn)));assert!(r.view.stasis.is_empty());assert!(r.exit.stasis.is_empty());
    }
}

#[test]
fn caller_reserves_defense_before_nonlethal_full_turn_bludgeon(){
    use spire_core::{combat::{Combat,CombatAction,CardInstanceId,PlayerState,FullPowerId,FullPowerEntry,PowerState,permanent_live_card,enemies::{self,SetupContext},rng::Partition},content::{CardId,Character},run::RunRng};
    let mut e=enemies::setup(EncounterId::GremlinLeader,2,0,&mut RunRng::new([41;32],Partition::Test,0),SetupContext{player:PlayerState{hp:17,max_hp:80,block:0,energy:3,max_energy:3,draw_per_turn:3,powers:vec![]},player_powers:vec![],character:Character::Ironclad,cards:[CardId::Defend,CardId::Defend,CardId::Bludgeon].into_iter().enumerate().map(|(i,id)|permanent_live_card(id,CardInstanceId(i as u64+1),0)).collect(),opening_draw:3,encounter_visit:0}).unwrap();
    e.setup.enemies[0].powers.push(FullPowerEntry{id:FullPowerId("strength"),state:PowerState{amount:8,skip_first:false,monster_source_grace:false}});
    let mut c=Combat::enter(e).unwrap();let view=c.public_full().unwrap();assert!(view.enemies.iter().any(|e|e.intent_damage.is_some_and(|n|n>=10)));let chosen=support::combat_action(&view,&c.legal_actions_full().unwrap());assert!(matches!(chosen,CombatAction::PlayCard{instance:1|2,target:None}),"visible severe incoming must reserve defense, not nonlethal all-energy boss damage");c.step_full(&chosen).unwrap();assert_eq!(c.public_full().unwrap().player.block,5);
}

#[test]
fn current_act2_map_menu_does_not_publish_future_room_offers(){
    let r=support::prefix(7,vec!["Automaton","Collector"],4096).unwrap();let map_steps=r.trace.iter().filter(|s|s.decision.act==2&&s.decision.room.as_ref().is_some_and(|r|r.kind==RoomViewKind::Map)).collect::<Vec<_>>();assert!(!map_steps.is_empty());
    for s in map_steps{assert!(s.decision.room.as_ref().unwrap().choices.is_empty());assert!(s.decision.offers.is_empty());assert!(s.map.as_ref().unwrap().nodes.iter().filter(|n|!n.visited).all(|n|n.resolved_room.is_none()));}
}
