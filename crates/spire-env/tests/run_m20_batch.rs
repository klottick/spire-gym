mod m20_support;
use m20_support::*;
use spire_env::*;
#[test]
fn m20_batch_shared_scheduler_short_diagnostic() {
    for workers in [1,2] {
        let cfg=default_config();let mut batch=RunBatch::new_m20(vec![cfg.clone();2],workers).unwrap();let b=batch.reset(&[Some(7);2]).unwrap();
        let mut single=RunSession::new_m20(cfg).unwrap();let sb=single.reset(Some(7)).unwrap();let expected=single.step(sb.token,RunActionIndex(0)).unwrap();
        let result=batch.step(&b.iter().map(|b|b.token).collect::<Vec<_>>(),&[RunActionIndex(0);2]).unwrap();
        for slot in 0..2 {assert_eq!(result[slot].reward,expected.reward);assert_eq!(batch.record(slot).unwrap(),single.record().unwrap());assert_eq!(batch.reproduction(slot).unwrap(),single.reproduction().unwrap());}
    }
}

#[test]
fn m20_batch_preflight_rejects_every_bad_slot_before_peers_advance() {
    for workers in [1,2] {
        let cfg=default_config();let mut bad=cfg.clone();bad.boss_history=RunBossHistory::Act1GuardianUnseen;
        assert!(RunBatch::new_m20(vec![cfg.clone(),bad],workers).is_err());assert!(RunBatch::new_m20(vec![],workers).is_err());assert!(RunBatch::new_m20(vec![cfg.clone()],0).is_err());
        let mut batch=RunBatch::new_m20(vec![cfg;2],workers).unwrap();let initial=batch.reset(&[Some(7),Some(11)]).unwrap();let tokens=initial.iter().map(|b|b.token).collect::<Vec<_>>();
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
fn m20_batch_postaccepted_faults_retain_every_slot_prefix_without_retry() {
    for workers in [1,2] {
        let cfg=default_config();let mut small=cfg.clone();small.capacities.map_nodes=1;
        let mut batch=RunBatch::new_m20(vec![cfg,small],workers).unwrap();let b=batch.reset(&[Some(7);2]).unwrap();let tokens=b.iter().map(|b|b.token).collect::<Vec<_>>();
        let error=batch.step(&tokens,&[RunActionIndex(0);2]).unwrap_err();let RunBatchError::Fault{slots}=error else {panic!("expected retained postaccepted fault")};
        assert_eq!(slots.len(),2);assert_eq!((slots[0].slot,slots[0].committed,slots[0].delivery_lost),(0,true,true));assert_eq!((slots[1].slot,slots[1].committed,slots[1].delivery_lost),(1,true,false));
        let before=(0..2).map(|i|batch.reproduction(i).unwrap()).collect::<Vec<_>>();for i in 0..2 {assert_eq!(batch.record(i).unwrap().unwrap().steps.len(),1);}
        assert!(batch.step(&tokens,&[RunActionIndex(0);2]).is_err());assert!(batch.reset(&[None;2]).is_err());for i in 0..2 {assert_eq!(batch.reproduction(i).unwrap(),before[i]);}
        for i in 0..2 {let r=batch.record(i).unwrap().unwrap();assert_eq!(replay_run(&RunEpisodeRecord::from_json(&r.to_json().unwrap()).unwrap(),RunCapacities::m10_default()).unwrap().accepted_count,1);}
        batch.close().unwrap();
    }
}

#[test]
fn m20_batch_none_and_implicit_ordinals_match_scalar_full_resets() {
    for workers in [1,2] {
        let cfg=default_config();let mut b=RunBatch::new_m20(vec![cfg.clone();2],workers).unwrap();
        let mut singles=[RunSession::new_m20(cfg.clone()).unwrap(),RunSession::new_m20(cfg).unwrap()];
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
