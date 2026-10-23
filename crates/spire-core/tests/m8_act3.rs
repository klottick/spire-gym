#[path="../examples/m8_support/mod.rs"] mod support;
use spire_core::{combat::Outcome,content::{EncounterId,EventId,RelicId},run::*};

fn histories()->Vec<Vec<&'static str>>{vec![vec![],vec!["AwakenedOne","DonuAndDeca"],vec!["AwakenedOne"]]}
fn complete(d:&PublicDecision)->bool{d.terminal.is_none()&&d.room.as_ref().is_some_and(|r|r.kind==RoomViewKind::Act3PrefixComplete)}

#[test]
fn m8_normal_start_prefix_or_genuine_death(){
    let mut completions=0;for seen in histories(){let initial=support::start(11,seen.clone()).unwrap().decision().unwrap();assert_eq!((initial.player.hp,initial.player.max_hp,initial.player.gold,initial.objects.len()),(80,80,99,10));assert_eq!(initial.player.relics,vec![RelicId::BurningBlood]);assert_eq!(initial.player.keys,[false;3]);assert!(initial.player.potions.iter().all(Option::is_none));let r=support::prefix(11,seen,8192).unwrap();let d=&r.final_decision;assert!(complete(d)||d.terminal==Some(RunTerminal::Death));if complete(d){completions+=1;assert_eq!((d.act,d.floor,d.player.hp>0,d.reward_total),(3,43,true,43));assert!(d.candidates.is_empty());assert!(r.trace.iter().all(|s|s.decision.floor<=43));assert_eq!(r.trace.last().unwrap().reward,0);}else{assert_eq!(d.player.hp,0);assert_eq!(r.trace.last().unwrap().reward,-1);}}
    assert!(completions>0,"preselected seed11 must exercise at least one actual prefix, not only diagnostic fixtures");
}

#[test]
fn m8_from_start_persistent_act3_event_and_elite(){
    let mut persistent=false;let mut elite=false;let mut inflame_played=false;
    for seen in histories(){let r=support::prefix(11,seen,8192).unwrap();elite|=r.combats.iter().any(|(act,c)|*act==3&&matches!(c.encounter,EncounterId::GiantHead|EncounterId::Nemesis|EncounterId::Reptomancer)&&c.outcome==Outcome::Victory);inflame_played|=r.trace.iter().filter(|s|s.decision.act==3).any(|s|s.decision.combat.as_ref().is_some_and(|c|c.played_powers.iter().any(|card|card.id==spire_core::content::CardId::Inflame)));
        for pair in r.trace.windows(2){let before=&pair[0].decision;let after=&pair[1].decision;if before.act!=3{continue}if before.room.as_ref().is_some_and(|r|r.kind==RoomViewKind::Event(EventId::Falling))&&after.objects.len()+1==before.objects.len(){persistent=true;}if !before.player.relics.contains(&RelicId::RedMask)&&after.player.relics.contains(&RelicId::RedMask){assert_eq!(after.player.gold,0);persistent=true;}}
    }assert!(persistent,"normal-start persistent event consequence required");assert!(elite,"normal-start real Act3 elite required");assert!(inflame_played,"the actually offered Power must execute on a public normal-start path");
}

#[test]
fn m8_preselected_natural_losses_remain_death_not_completion(){
    for (seed,seen,floor,reward) in [(11,vec![],41,40),(7,vec![],40,39),(7,vec!["AwakenedOne","DonuAndDeca"],40,39),(7,vec!["AwakenedOne"],40,39)]{let r=support::prefix(seed,seen,8192).unwrap();let d=r.final_decision;assert_eq!((d.act,d.floor,d.player.hp,d.terminal,d.reward_total),(3,floor,0,Some(RunTerminal::Death),reward));assert!(!complete(&d));assert!(d.candidates.is_empty());assert_eq!(r.trace.last().unwrap().reward,-1);}
}

#[test]
fn m8_public_replay_privacy_and_stale_cutoff(){
    for seed in [7,11]{for seen in histories(){let r=support::prefix(seed,seen.clone(),8192).unwrap();let mut replay=support::start(seed,seen).unwrap();for s in &r.trace{let t=replay.step(&s.action).unwrap();assert_eq!(t.reward,s.reward);assert_eq!(replay.decision().unwrap(),s.decision);assert_eq!(replay.public_map(),s.map);if s.decision.room.as_ref().is_some_and(|r|matches!(r.kind,RoomViewKind::Map|RoomViewKind::Act3PrefixComplete)){assert!(s.decision.offers.is_empty());assert!(s.decision.room.as_ref().unwrap().choices.is_empty());assert!(s.map.as_ref().unwrap().nodes.iter().filter(|n|!n.visited).all(|n|n.resolved_room.is_none()));}}
        assert_eq!(replay.decision().unwrap(),r.final_decision);assert!(complete(&r.final_decision)||r.final_decision.terminal==Some(RunTerminal::Death));let old=replay.decision().unwrap();let map=replay.public_map();assert!(replay.step(&r.trace.last().unwrap().action).is_err());assert_eq!(replay.decision().unwrap(),old);assert_eq!(replay.public_map(),map);
    }}
}

#[test]
fn m8_budget_is_not_a_game_outcome(){
    for budget in [0,1]{let r=support::prefix(11,vec![],budget).unwrap();assert_eq!(r.trace.len(),budget);assert_eq!(r.final_decision.terminal,None);assert!(!complete(&r.final_decision));}
    let r=support::death(11,8192).unwrap();assert_eq!(r.final_decision.terminal,Some(RunTerminal::Death));assert_eq!(r.final_decision.player.hp,0);assert!(r.final_decision.candidates.is_empty());assert!(!complete(&r.final_decision));assert_eq!(r.trace.last().unwrap().reward,-1);
}

#[test]
fn m8_supplied_diagnostics_execute_lifecycles_and_all_bosses(){
    let reports=support::diagnostics(11,8192).unwrap();for id in [EncounterId::GiantHead,EncounterId::Nemesis,EncounterId::Reptomancer,EncounterId::AwakenedOne,EncounterId::TimeEater,EncounterId::DonuAndDeca]{assert!(reports.iter().any(|r|r.encounter==id&&r.view.outcome.is_some()));}
    assert!(reports.iter().any(|r|r.encounter==EncounterId::ThreeDarklings&&r.saw_half_dead&&r.saw_darkling_return));assert!(reports.iter().any(|r|r.encounter==EncounterId::AwakenedOne&&r.saw_half_dead));assert!(support::diagnostics(11,0).is_err());
}
