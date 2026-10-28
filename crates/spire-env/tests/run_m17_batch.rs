mod m17_support;
use m17_support::*;
use spire_env::*;
#[test]
fn m17_batch_matches_single_replay_and_preserves_peer_preflight(){
    let cases=retained();
    for (seed,scenario,run,caller) in &cases{for workers in [1,2]{
        let cfg=config(*scenario);let mut batch=RunBatch::new_m17(vec![cfg.clone();2],workers).unwrap();let mut bounds=batch.reset(&[Some(*seed);2]).unwrap();
        for (i,step) in caller.trace.iter().enumerate(){
            let index=step.decision.candidates.iter().position(|c|c.action==step.action).unwrap() as i64;
            for slot in 0..2{let r=batch.reproduction(slot).unwrap();assert_eq!(r.latest_decision.unwrap().unwrap(),step.decision);assert_eq!(r.latest_map,step.map);}
            let result=batch.step(&bounds.iter().map(|b|b.token).collect::<Vec<_>>(),&[RunActionIndex(index);2]).unwrap();
            let next=caller.trace.get(i+1).map(|s|&s.decision).unwrap_or(caller.final_decision.as_ref().unwrap());let map=caller.trace.get(i+1).map(|s|s.map.as_ref()).unwrap_or(caller.final_map.as_ref());
            for slot in 0..2{assert_eq!(result[slot].reward,step.transition.reward);assert_eq!(result[slot].boundary.terminal,step.transition.terminal);assert!(!result[slot].truncated);assert_eq!(result[slot].boundary.observation,encode_run_m17(next,map,&cfg.capacities).unwrap());bounds[slot]=result[slot].boundary.clone();}
        }
        for slot in 0..2{let record=batch.record(slot).unwrap().unwrap();let report=replay_run(&RunEpisodeRecord::from_json(&record.to_json().unwrap()).unwrap(),cfg.capacities.clone()).unwrap();assert_eq!(report.final_decision,run.decision().unwrap());assert_eq!(report.final_map,run.public_map());assert_eq!(report.accepted_count,caller.accepted());}
        let before=(0..2).map(|i|batch.reproduction(i).unwrap()).collect::<Vec<_>>();let tokens=bounds.iter().map(|b|b.token).collect::<Vec<_>>();assert!(matches!(batch.step(&tokens,&[RunActionIndex(0);2]),Err(RunBatchError::Input{..})));for slot in 0..2{assert_eq!(batch.reproduction(slot).unwrap(),before[slot]);}
        assert!(batch.reset(&[Some(*seed)]).is_err());for slot in 0..2{assert_eq!(batch.reproduction(slot).unwrap(),before[slot]);}
        let reset=batch.reset(&[Some(*seed),None]).unwrap();for slot in 0..2{assert_ne!(reset[slot].token,bounds[slot].token);let d=batch.reproduction(slot).unwrap().latest_decision.unwrap().unwrap();assert_eq!((d.player.hp,d.player.gold,d.objects.len()),(75,99,10));assert!(batch.reproduction(slot).unwrap().latest_map.is_none());assert!(batch.record(slot).unwrap().unwrap().steps.is_empty());}
    }}
    // Heterogeneous recorded histories stop at the first ending. Every proposed
    // batch operation still has one action for every slot; no no-op lanes.
    for workers in [1,2]{
        let left=&cases[0];let right=&cases[5];let mut batch=RunBatch::new_m17(vec![config(left.1),config(right.1)],workers).unwrap();let mut bounds=batch.reset(&[Some(left.0),Some(right.0)]).unwrap();
        let n=left.3.trace.len().min(right.3.trace.len());
        for i in 0..n{let rows=[&left.3.trace[i],&right.3.trace[i]];let actions=rows.map(|s|RunActionIndex(s.decision.candidates.iter().position(|c|c.action==s.action).unwrap() as i64));let result=batch.step(&bounds.iter().map(|b|b.token).collect::<Vec<_>>(),&actions).unwrap();bounds=result.into_iter().map(|r|r.boundary).collect();}
        let before=(0..2).map(|i|batch.reproduction(i).unwrap()).collect::<Vec<_>>();assert!(batch.step(&bounds.iter().map(|b|b.token).collect::<Vec<_>>(),&[RunActionIndex(0);2]).is_err());for slot in 0..2{assert_eq!(batch.reproduction(slot).unwrap(),before[slot]);}
    }
}
#[test]
fn m17_batch_stale_invalid_and_constructor_preflight(){
    let cfg=config(caller::Scenario::Guardian);let mut bad=cfg.clone();bad.boss_history=RunBossHistory::Act1GuardianUnseen;assert!(RunBatch::new_m17(vec![cfg.clone(),bad],2).is_err());assert!(RunBatch::new_m17(vec![cfg.clone()],0).is_err());
    let mut batch=RunBatch::new_m17(vec![cfg;2],2).unwrap();let initial=batch.reset(&[Some(7),Some(11)]).unwrap();let before=(0..2).map(|i|batch.reproduction(i).unwrap()).collect::<Vec<_>>();
    for indices in [[RunActionIndex(0),RunActionIndex(-1)],[RunActionIndex(i64::MAX),RunActionIndex(0)]]{assert!(batch.step(&initial.iter().map(|b|b.token).collect::<Vec<_>>(),&indices).is_err());for slot in 0..2{assert_eq!(batch.reproduction(slot).unwrap(),before[slot]);}}
    let new=batch.reset(&[Some(7),None]).unwrap();assert!(batch.step(&[initial[0].token,new[1].token],&[RunActionIndex(0);2]).is_err());batch.step(&new.iter().map(|b|b.token).collect::<Vec<_>>(),&[RunActionIndex(0);2]).unwrap();
}
