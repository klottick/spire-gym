mod m19_support;
use m19_support::*;
use spire_env::*;
#[test]
fn m19_batch_shared_scheduler_short_diagnostic() {
    for workers in [1,2] {
        let cfg=default_config();let mut batch=RunBatch::new_m19(vec![cfg.clone();2],workers).unwrap();let b=batch.reset(&[Some(7);2]).unwrap();
        let mut single=RunSession::new_m19(cfg).unwrap();let sb=single.reset(Some(7)).unwrap();let expected=single.step(sb.token,RunActionIndex(0)).unwrap();
        let result=batch.step(&b.iter().map(|b|b.token).collect::<Vec<_>>(),&[RunActionIndex(0);2]).unwrap();
        for slot in 0..2 {assert_eq!(result[slot].reward,expected.reward);assert_eq!(batch.record(slot).unwrap(),single.record().unwrap());assert_eq!(batch.reproduction(slot).unwrap(),single.reproduction().unwrap());}
    }
}

#[test]
fn m19_batch_preflight_rejects_every_bad_slot_before_peers_advance() {
    for workers in [1,2] {
        let cfg=default_config();let mut bad=cfg.clone();bad.boss_history=RunBossHistory::Act1GuardianUnseen;
        assert!(RunBatch::new_m19(vec![cfg.clone(),bad],workers).is_err());assert!(RunBatch::new_m19(vec![],workers).is_err());assert!(RunBatch::new_m19(vec![cfg.clone()],0).is_err());
        let mut batch=RunBatch::new_m19(vec![cfg;2],workers).unwrap();let initial=batch.reset(&[Some(7),Some(11)]).unwrap();let tokens=initial.iter().map(|b|b.token).collect::<Vec<_>>();
        let before=(0..2).map(|i|batch.reproduction(i).unwrap()).collect::<Vec<_>>();
        for indices in [[RunActionIndex(0),RunActionIndex(-1)],[RunActionIndex(i64::MAX),RunActionIndex(0)]] {
            assert!(matches!(batch.step(&tokens,&indices),Err(RunBatchError::Input{..})));for i in 0..2 {assert_eq!(batch.reproduction(i).unwrap(),before[i]);}
        }
        for (t,a) in [(&tokens[..1],&[RunActionIndex(0);2][..]),(&tokens[..],&[RunActionIndex(0)][..])] {
            assert!(matches!(batch.step(t,a),Err(RunBatchError::Input{..})));for i in 0..2 {assert_eq!(batch.reproduction(i).unwrap(),before[i]);}
        }
        assert!(batch.reset(&[None]).is_err());for i in 0..2 {assert_eq!(batch.reproduction(i).unwrap(),before[i]);}
        let next=batch.step(&tokens,&[RunActionIndex(0);2]).unwrap();let next_tokens=next.iter().map(|s|s.boundary.token).collect::<Vec<_>>();
        let before=(0..2).map(|i|batch.reproduction(i).unwrap()).collect::<Vec<_>>();
        for stale in [[initial[0].token,next_tokens[1]],[next_tokens[0],initial[1].token]] {
            assert!(matches!(batch.step(&stale,&[RunActionIndex(0);2]),Err(RunBatchError::Input{..})));for i in 0..2 {assert_eq!(batch.reproduction(i).unwrap(),before[i]);}
        }
        let reset=batch.reset(&[Some(7),None]).unwrap();
        for i in 0..2 {let r=batch.reproduction(i).unwrap();let d=r.latest_decision.unwrap().unwrap();assert_eq!((d.player.hp,d.player.gold,d.objects.len()),(75,99,10));assert!(r.latest_map.is_none());assert_ne!(reset[i].token,next_tokens[i]);assert!(batch.record(i).unwrap().unwrap().steps.is_empty());assert_eq!(r.ordinal,if i==0{0}else{1});}
        assert_eq!(batch.reproduction(1).unwrap().seed,Some(11));
        batch.close().unwrap();assert!(batch.reproduction(0).unwrap().closed);
    }
}

#[test]
fn m19_batch_postaccepted_faults_retain_every_slot_prefix_without_retry() {
    for workers in [1,2] {
        let cfg=default_config();let mut small=cfg.clone();small.capacities.map_nodes=1;
        let mut batch=RunBatch::new_m19(vec![cfg,small],workers).unwrap();let b=batch.reset(&[Some(7);2]).unwrap();let tokens=b.iter().map(|b|b.token).collect::<Vec<_>>();
        let error=batch.step(&tokens,&[RunActionIndex(0);2]).unwrap_err();let RunBatchError::Fault{slots}=error else {panic!("expected retained postaccepted fault")};
        assert_eq!(slots.len(),2);assert_eq!((slots[0].slot,slots[0].committed,slots[0].delivery_lost),(0,true,true));assert_eq!((slots[1].slot,slots[1].committed,slots[1].delivery_lost),(1,true,false));
        let before=(0..2).map(|i|batch.reproduction(i).unwrap()).collect::<Vec<_>>();for i in 0..2 {assert_eq!(batch.record(i).unwrap().unwrap().steps.len(),1);}
        assert!(batch.step(&tokens,&[RunActionIndex(0);2]).is_err());assert!(batch.reset(&[None;2]).is_err());for i in 0..2 {assert_eq!(batch.reproduction(i).unwrap(),before[i]);}
        for i in 0..2 {let r=batch.record(i).unwrap().unwrap();assert_eq!(replay_run(&RunEpisodeRecord::from_json(&r.to_json().unwrap()).unwrap(),RunCapacities::m10_default()).unwrap().accepted_count,1);}
        batch.close().unwrap();
    }
}

#[test]
fn m19_batch_matches_single_replay_for_every_fixed_case() {
    for (seed,scenario,core,trace) in retained() {
        let (single,record)=consume_single(seed,scenario,&core,&trace);let cfg=case_config(scenario);
        for workers in [1,2] {
            let mut batch=RunBatch::new_m19(vec![cfg.clone();2],workers).unwrap();let mut bounds=batch.reset(&[Some(seed);2]).unwrap();
            for step in &trace.trace {
                let index=step.decision.candidates.iter().position(|c|c.action==step.action).unwrap();
                for slot in 0..2 {let r=batch.reproduction(slot).unwrap();assert_eq!(r.latest_decision.unwrap().unwrap(),step.decision);assert_eq!(r.latest_map,step.map);assert_eq!(bounds[slot].observation,encode_run_m19(&step.decision,step.map.as_ref(),&cfg.capacities).unwrap());}
                let result=batch.step(&bounds.iter().map(|b|b.token).collect::<Vec<_>>(),&[RunActionIndex(index as i64);2]).unwrap();
                for result in &result {assert_eq!((result.reward,result.boundary.terminal),(step.transition.reward,step.transition.terminal));assert!(!result.truncated);assert_eq!(result.terminated,step.transition.terminal.is_some());}
                bounds=result.into_iter().map(|r|r.boundary).collect();
            }
            if let Some((d,_,action,error))=&trace.fault_attempt {
                let index=d.candidates.iter().position(|c|c.action==*action).unwrap();let actual=batch.step(&bounds.iter().map(|b|b.token).collect::<Vec<_>>(),&[RunActionIndex(index as i64);2]).unwrap_err();
                let RunBatchError::Fault{slots}=actual else {panic!("expected retained failed core attempts")};assert_eq!(slots.len(),2);
                for slot in slots {let RunSessionError::Core(actual)=slot.error else{panic!("core error lost")};assert_eq!(format!("{actual:?}"),*error);assert!(!slot.delivery_lost);}
            }
            for slot in 0..2 {assert_eq!(batch.record(slot).unwrap().unwrap(),record);assert_eq!(batch.reproduction(slot).unwrap(),single.reproduction().unwrap());}
            let before=(0..2).map(|i|batch.reproduction(i).unwrap()).collect::<Vec<_>>();
            if trace.fault_attempt.is_some()||core.decision().unwrap().candidates.is_empty() {
                assert!(batch.step(&bounds.iter().map(|b|b.token).collect::<Vec<_>>(),&[RunActionIndex(0);2]).is_err());for i in 0..2 {assert_eq!(batch.reproduction(i).unwrap(),before[i]);}
            }
        }
    }
}

#[test]
fn m19_heterogeneous_prefix_stops_at_earliest_death_or_cutoff() {
    let cases=retained();let left=&cases[0];let right=&cases[5];
    for workers in [1,2] {
        let mut batch=RunBatch::new_m19(vec![case_config(left.1),case_config(right.1)],workers).unwrap();let mut bounds=batch.reset(&[Some(left.0),Some(right.0)]).unwrap();
        let length=left.3.trace.len().min(right.3.trace.len());
        for i in 0..length {
            let rows=[&left.3.trace[i],&right.3.trace[i]];let actions=rows.map(|s|RunActionIndex(s.decision.candidates.iter().position(|c|c.action==s.action).unwrap() as i64));
            let next=batch.step(&bounds.iter().map(|b|b.token).collect::<Vec<_>>(),&actions).unwrap();
            for slot in 0..2 {assert_eq!(next[slot].reward,rows[slot].transition.reward);assert_eq!(next[slot].boundary.terminal,rows[slot].transition.terminal);}
            bounds=next.into_iter().map(|r|r.boundary).collect();
        }
        let before=(0..2).map(|i|batch.reproduction(i).unwrap()).collect::<Vec<_>>();
        let earliest=if left.3.trace.len()<=right.3.trace.len(){0}else{1};let d=before[earliest].latest_decision.as_ref().unwrap().as_ref().unwrap();
        assert!(d.terminal.is_some()||d.room.as_ref().is_some_and(|r|r.kind==spire_core::run::RoomViewKind::Act4Ready),"UNMET: heterogeneous case stops at budget/fault rather than true endpoint");
        let mut actions=[RunActionIndex(0);2];actions[earliest]=RunActionIndex(i64::MAX);
        assert!(matches!(batch.step(&bounds.iter().map(|b|b.token).collect::<Vec<_>>(),&actions),Err(RunBatchError::Input{..})));
        for slot in 0..2 {assert_eq!(batch.reproduction(slot).unwrap(),before[slot]);}
        let reset=batch.reset(&[Some(left.0),None]).unwrap();for slot in 0..2 {assert_ne!(reset[slot].token,bounds[slot].token);assert!(batch.record(slot).unwrap().unwrap().steps.is_empty());assert!(batch.reproduction(slot).unwrap().latest_map.is_none());}
    }
}

#[test]
fn m19_batch_none_and_implicit_ordinals_match_scalar_full_resets() {
    for workers in [1,2] {
        let cfg=default_config();let mut b=RunBatch::new_m19(vec![cfg.clone();2],workers).unwrap();
        let mut singles=[RunSession::new_m19(cfg.clone()).unwrap(),RunSession::new_m19(cfg).unwrap()];
        for seeds in [[None,None],[Some(7),None],[None,None],[Some(11),Some(7)]] {
            let bounds=b.reset(&seeds).unwrap();
            for slot in 0..2 {
                let expected=singles[slot].reset(seeds[slot]).unwrap();
                assert_eq!(bounds[slot].observation,expected.observation);
                assert_eq!(b.reproduction(slot).unwrap(),singles[slot].reproduction().unwrap());
            }
        }
    }
}
