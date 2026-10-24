#[path="../examples/m9_support/mod.rs"] mod support;
use support::{M9BossHistory as History,M9Completion};
use spire_core::{combat::Outcome,content::{CardId,RelicId},run::*};

fn histories()->[History;3]{[History::AwakenedUnseen,History::TimeEaterUnseen,History::DonuDecaUnseen]}
fn replay(seed:u64,history:History,r:&support::M9Report){
    let mut run=support::start(seed,history).unwrap();
    for (index,step) in r.trace.iter().enumerate(){
        assert!(run.decision().unwrap().candidates.iter().any(|c|c.action==step.action),"unoffered replay action {index}");
        let t=run.step(&step.action).unwrap();assert_eq!((t.reward,t.terminal),(step.reward,step.decision.terminal));
        assert_eq!(run.decision().unwrap(),step.decision,"replay decision {index}");assert_eq!(run.public_map(),step.map,"replay map {index}");
    }
    assert_eq!(run.decision().unwrap(),r.final_decision);assert_eq!(run.public_map(),r.final_map);
}

#[test]
fn m9_normal_start_and_all_three_boss_contexts_are_real(){
    for history in histories(){
        let d=support::start(11,history).unwrap().decision().unwrap();
        assert_eq!(d.content_profile,Some(RoomScenario::M9IroncladAct3));
        assert_eq!((d.act,d.floor,d.player.hp,d.player.max_hp,d.player.gold),(1,0,80,80,99));
        assert_eq!(d.player.relics,vec![RelicId::BurningBlood]);assert_eq!(d.player.keys,[false;3]);assert_eq!(d.player.potions,vec![None;3]);
        assert_eq!(d.objects.len(),10);assert_eq!(d.objects.iter().filter(|c|c.id==CardId::Strike&&c.upgrades==0).count(),5);assert_eq!(d.objects.iter().filter(|c|c.id==CardId::Defend&&c.upgrades==0).count(),4);assert_eq!(d.objects.iter().filter(|c|c.id==CardId::Bash&&c.upgrades==0).count(),1);
        assert_eq!((d.reward_total,d.terminal),(0,None));assert!(d.combat.is_none());
    }
}

#[test]
fn m9_from_start_gameplay_outcomes_and_public_boundaries_replay_without_win_requirement(){
    for seed in [11,7]{for history in histories(){
        let r=support::complete(seed,history,8192).unwrap();let d=&r.final_decision;
        println!("seed{seed} {history:?}: {:?}, actions{}, act{} floor{} HP{}/{} reward{} combats{:?}",r.completion,r.trace.len(),d.act,d.floor,d.player.hp,d.player.max_hp,d.reward_total,r.combats.iter().filter(|(a,_)|*a==3).collect::<Vec<_>>());
        assert!(matches!(r.completion,M9Completion::Act3NoKeys|M9Completion::Death),"not an actual gameplay outcome");
        if r.trace.iter().any(|s|s.decision.act==3&&s.decision.floor==50&&s.decision.combat.is_some()){
            assert!(r.combats.iter().any(|(a,c)|*a==3&&c.encounter==history.boss()));
        }
        if r.completion==M9Completion::Act3NoKeys{
            assert_eq!((d.act,d.floor,d.terminal),(3,50,Some(RunTerminal::Act3NoKeys)));
            assert_eq!(d.reward_total,50+5*d.player.keys.into_iter().filter(|key|*key).count() as i32);
            assert!(r.combats.iter().any(|(a,c)|*a==3&&c.encounter==history.boss()&&c.outcome==Outcome::Victory));
            assert!(r.trace.iter().any(|s|s.decision.room.as_ref().is_some_and(|r|r.kind==RoomViewKind::Act3BossComplete)));
            assert_eq!(r.trace.last().unwrap().reward,0);assert!(d.player.hp>0);
        }else{
            assert_eq!((d.player.hp,d.terminal),(0,Some(RunTerminal::Death)));assert_eq!(r.trace.last().unwrap().reward,-1);
            assert_eq!(d.reward_total,i32::try_from(d.floor).unwrap()-1+5*d.player.keys.into_iter().filter(|key|*key).count() as i32);
            assert_eq!(r.combats.last().unwrap().1.outcome,Outcome::Death);
        }
        assert!(d.candidates.is_empty());assert!(r.trace.iter().all(|s|s.decision.floor<=50&&s.decision.terminal!=Some(RunTerminal::HeartVictory)));
        replay(seed,history,&r);
    }}
    // Loss is valid gameplay. These from-start traces exercise tools/replay, not
    // an in-repo strategy's strength. Supplied m9_boss/m9_end fixtures separately
    // cover all actual boss/end paths; neither form of evidence replaces the other.
}

#[test]
fn m9_natural_death_and_budget_are_different(){
    for budget in [0,1]{let r=support::complete(11,History::TimeEaterUnseen,budget).unwrap();assert_eq!(r.completion,M9Completion::Unfinished);assert_eq!(r.trace.len(),budget);assert_eq!(r.final_decision.terminal,None);}
    assert!(support::complete(11,History::TimeEaterUnseen,8193).is_err());
    let r=support::death(11,8192).unwrap();assert_eq!((r.completion,r.final_decision.terminal,r.final_decision.player.hp),(M9Completion::Death,Some(RunTerminal::Death),0));
    assert!(r.trace.iter().filter_map(|s|if let GameAction::Combat(a)=&s.action{Some(a)}else{None}).all(|a|*a==spire_core::combat::CombatAction::EndTurn));
    replay(11,History::AwakenedUnseen,&r);
}
