use spire_core::{combat::rng::Partition, content::Character};
use spire_env::{
    RunAct1BossTarget, RunAct2BossTarget, RunAct3BossTarget, RunBatch, RunBatchError,
    RunBossHistory, RunCapacities, RunRecordErrorKind, RunSession, RunSessionConfig,
    RunSessionError,
};

fn history(
    act1: RunAct1BossTarget,
    act2: RunAct2BossTarget,
    act3: RunAct3BossTarget,
) -> RunBossHistory {
    RunBossHistory::M15SilentHeart { act1, act2, act3 }
}

fn config(
    history: RunBossHistory,
    record_replay: bool,
) -> RunSessionConfig {
    RunSessionConfig {
        character: Character::Silent,
        ascension: 0,
        partition: Partition::Test,
        boss_history: history,
        capacities: RunCapacities::m10_default(),
        record_replay,
    }
}

fn default_config() -> RunSessionConfig {
    config(
        history(
            RunAct1BossTarget::Guardian,
            RunAct2BossTarget::Champ,
            RunAct3BossTarget::AwakenedOne,
        ),
        true,
    )
}

#[test]
fn m15_batch_workers_match_short_single_session_prefixes() {
    let configs = vec![
        default_config(),
        config(
            history(
                RunAct1BossTarget::Hexaghost,
                RunAct2BossTarget::BronzeAutomaton,
                RunAct3BossTarget::TimeEater,
            ),
            true,
        ),
    ];
    let mut results_by_worker = Vec::new();
    for workers in [1, 2] {
        let mut batch = RunBatch::new_m15(configs.clone(), workers).unwrap();
        let mut singles = configs
            .iter()
            .cloned()
            .map(|config| RunSession::new_m15(config).unwrap())
            .collect::<Vec<_>>();
        let mut boundaries = batch.reset(&[Some(7), Some(11)]).unwrap();
        let mut single_boundaries = singles
            .iter_mut()
            .zip([7, 11])
            .map(|(session, seed)| session.reset(Some(seed)).unwrap())
            .collect::<Vec<_>>();
        for slot in 0..2 {
            assert_eq!(boundaries[slot].observation, single_boundaries[slot].observation);
            assert_ne!(boundaries[slot].token, single_boundaries[slot].token);
        }

        let tokens = boundaries.iter().map(|boundary| boundary.token).collect::<Vec<_>>();
        let steps = batch
            .step(&tokens, &[spire_env::RunActionIndex(0); 2])
            .unwrap();
        for slot in 0..2 {
            let single = singles[slot]
                .step(single_boundaries[slot].token, spire_env::RunActionIndex(0))
                .unwrap();
            assert_eq!(steps[slot].boundary.observation, single.boundary.observation);
            assert_eq!(steps[slot].boundary.terminal, single.boundary.terminal);
            assert_eq!(steps[slot].reward, single.reward);
            boundaries[slot] = steps[slot].boundary.clone();
            single_boundaries[slot] = single.boundary;
        }
        let records = (0..2)
            .map(|slot| {
                let batch_record = batch.record(slot).unwrap().unwrap();
                assert_eq!(Some(batch_record.clone()), singles[slot].record().unwrap());
                assert_eq!(batch.reproduction(slot).unwrap(), singles[slot].reproduction().unwrap());
                batch_record
            })
            .collect::<Vec<_>>();
        results_by_worker.push(records);
    }
    assert_eq!(results_by_worker[0], results_by_worker[1]);
}

#[test]
fn m15_batch_preflights_the_whole_step_before_mutating_any_slot() {
    let mut batch = RunBatch::new_m15(vec![default_config(), default_config()], 2).unwrap();
    let boundaries = batch.reset(&[Some(7), Some(11)]).unwrap();
    let before = (0..2)
        .map(|slot| batch.reproduction(slot).unwrap())
        .collect::<Vec<_>>();
    let tokens = boundaries.iter().map(|boundary| boundary.token).collect::<Vec<_>>();

    let mut stale = tokens.clone();
    stale[1] = stale[0];
    assert!(matches!(
        batch.step(&stale, &[spire_env::RunActionIndex(0); 2]),
        Err(RunBatchError::Input {
            slot: Some(1),
            error: RunSessionError::StaleToken,
        })
    ));
    assert!(matches!(
        batch.step(&tokens, &[spire_env::RunActionIndex(0), spire_env::RunActionIndex(-1)]),
        Err(RunBatchError::Input { slot: Some(1), error: RunSessionError::Input(_) })
    ));
    for slot in 0..2 {
        assert_eq!(batch.reproduction(slot).unwrap(), before[slot]);
        assert_eq!(batch.record(slot).unwrap().unwrap().steps.len(), 0);
    }
}

#[test]
fn m15_batch_rejects_a_cross_profile_config_before_creating_sessions() {
    let mut foreign = default_config();
    foreign.boss_history = RunBossHistory::M14SilentAct3 {
        act1: RunAct1BossTarget::Guardian,
        act2: RunAct2BossTarget::Champ,
        act3: RunAct3BossTarget::AwakenedOne,
    };
    assert!(matches!(
        RunBatch::new_m15(vec![default_config(), foreign], 2),
        Err(RunBatchError::Input {
            slot: Some(1),
            error: RunSessionError::Input("boss history does not match Run profile"),
        })
    ));
}

#[test]
fn m15_capacity_fault_retains_first_prefix_and_quiesces_delivered_peers() {
    for workers in [1, 2] {
        let mut constrained = default_config();
        constrained.capacities.map_nodes = 1;
        let mut no_record = default_config();
        no_record.record_replay = false;
        let configs = vec![constrained.clone(), default_config(), no_record];
        let mut batch = RunBatch::new_m15(configs.clone(), workers).unwrap();
        let boundaries = batch.reset(&[Some(7), Some(7), Some(7)]).unwrap();
        let tokens = boundaries.iter().map(|boundary| boundary.token).collect::<Vec<_>>();
        let indices = [spire_env::RunActionIndex(0); 3];
        let faults = match batch.step(&tokens, &indices).unwrap_err() {
            RunBatchError::Fault { slots } => slots,
            error => panic!("expected retained M15 capacity and delivery faults, got {error:?}"),
        };
        assert_eq!(faults.len(), 3);
        assert_eq!((faults[0].slot, faults[0].committed, faults[0].delivery_lost), (0, true, false));
        assert_eq!((faults[1].slot, faults[1].committed, faults[1].delivery_lost), (1, true, true));
        assert_eq!((faults[2].slot, faults[2].committed, faults[2].delivery_lost), (2, true, true));

        let first_record = batch.record(0).unwrap().unwrap();
        assert_eq!(first_record.steps.len(), 1);
        let retained = first_record.fault.as_ref().unwrap();
        assert!(retained.committed && retained.gameplay_recorded);
        assert_eq!(retained.error.kind, RunRecordErrorKind::Capacity);
        assert_eq!(retained.error.capacity.as_ref().unwrap().group, "map_nodes");

        let peer_record = batch.record(1).unwrap().unwrap();
        assert_eq!(peer_record.steps.len(), 1);
        let peer_fault = peer_record.fault.as_ref().unwrap();
        assert!(peer_fault.committed && peer_fault.gameplay_recorded);
        assert_eq!(peer_fault.error.kind, RunRecordErrorKind::DeliveryFault);

        assert_eq!(batch.record(2).unwrap(), None);
        let unrecorded_peer = batch.reproduction(2).unwrap();
        assert!(unrecorded_peer.last_accepted_step.is_some());
        assert!(matches!(unrecorded_peer.fault, Some(RunSessionError::DeliveryFault(_))));

        let mut single = RunSession::new_m15(constrained).unwrap();
        let initial = single.reset(Some(7)).unwrap();
        assert_eq!(
            single.step(initial.token, spire_env::RunActionIndex(0)).unwrap_err(),
            RunSessionError::DeliveryFault("post-step Run encoding failed")
        );
        assert_eq!(batch.record(0).unwrap(), single.record().unwrap());
        assert_eq!(batch.reproduction(0).unwrap(), single.reproduction().unwrap());

        let frozen = (0..3)
            .map(|slot| batch.reproduction(slot).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(batch.step(&tokens, &indices), Err(RunBatchError::Fault { slots: faults.clone() }));
        assert_eq!(batch.reset(&[Some(11); 3]), Err(RunBatchError::Fault { slots: faults }));
        for slot in 0..3 {
            assert_eq!(batch.reproduction(slot).unwrap(), frozen[slot]);
        }
    }
}

#[test]
fn m15_batch_reset_is_full_batch_and_none_advances_its_episode() {
    for workers in [1,2] {
        let mut batch=RunBatch::new_m15(vec![default_config();2],workers).unwrap();
        let before=batch.reset(&[Some(7),Some(11)]).unwrap();
        let tokens=before.iter().map(|row|row.token).collect::<Vec<_>>();
        batch.step(&tokens,&[spire_env::RunActionIndex(0);2]).unwrap();
        let records=(0..2).map(|slot|batch.record(slot).unwrap()).collect::<Vec<_>>();
        assert!(matches!(batch.reset(&[None]),Err(RunBatchError::Input{slot:None,..})));
        for slot in 0..2 {assert_eq!(batch.record(slot).unwrap(),records[slot]);}
        let reset=batch.reset(&[None,Some(11)]).unwrap();
        for slot in 0..2 {
            let record=batch.record(slot).unwrap().unwrap();
            assert_eq!(record.steps.len(),0);
            assert!(record.terminal.is_none() && record.fault.is_none());
            assert_ne!(reset[slot].token,tokens[slot]);
            assert_eq!(record.seed,if slot==0{7}else{11});
            assert_eq!(record.ordinal,if slot==0{1}else{0});
        }
        assert!(matches!(batch.step(&tokens,&[spire_env::RunActionIndex(0);2]),Err(RunBatchError::Input{slot:Some(0),error:RunSessionError::StaleToken})));
    }
}
