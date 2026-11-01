use spire_core::{
    combat::rng::Partition,
    content::Character,
    run::RoomScenario,
};
use spire_env::*;

fn m25_config() -> RunSessionConfig {
    RunSessionConfig {
        character: Character::Watcher,
        ascension: 0,
        partition: Partition::Test,
        boss_history: RunBossHistory::M25WatcherHeart {
            act1: RunAct1BossTarget::Guardian,
            act2: RunAct2BossTarget::Champ,
            act3: RunAct3BossTarget::AwakenedOne,
        },
        capacities: RunCapacities::m10_default(),
        record_replay: true,
    }
}

fn m24_config() -> RunSessionConfig {
    RunSessionConfig { boss_history: RunBossHistory::M24WatcherAct3 {
        act1: RunAct1BossTarget::Guardian, act2: RunAct2BossTarget::Champ, act3: RunAct3BossTarget::AwakenedOne,
    }, ..m25_config() }
}

fn m15_config() -> RunSessionConfig {
    RunSessionConfig { character: Character::Silent, boss_history: RunBossHistory::M15SilentHeart {
        act1: RunAct1BossTarget::Guardian, act2: RunAct2BossTarget::Champ, act3: RunAct3BossTarget::AwakenedOne,
    }, ..m25_config() }
}

fn m20_config() -> RunSessionConfig {
    RunSessionConfig { character: Character::Defect, boss_history: RunBossHistory::M20DefectHeart {
        act1: RunAct1BossTarget::Guardian, act2: RunAct2BossTarget::Champ, act3: RunAct3BossTarget::AwakenedOne,
    }, ..m25_config() }
}

fn assert_m25_offer_group() {
    let descriptor = RunSession::new_m25(m25_config()).unwrap().observation_spec().unwrap().descriptor();
    assert!(descriptor["tensors"].as_array().unwrap().iter().any(|tensor| tensor["key"] == "combat_card_offers"));
}

#[test]
fn m25_batch_order_matches_singles() {
    for workers in [1, 2] {
        assert_m25_offer_group();
        let configs = vec![m25_config(), RunSessionConfig {
            boss_history: RunBossHistory::M25WatcherHeart {
                act1: RunAct1BossTarget::SlimeBoss,
                act2: RunAct2BossTarget::Collector,
                act3: RunAct3BossTarget::DonuAndDeca,
            },
            ..m25_config()
        }];
        let mut batch = RunBatch::new_m25(configs.clone(), workers).unwrap();
        let mut singles = configs.iter().cloned().map(RunSession::new_m25).collect::<Result<Vec<_>, _>>().unwrap();
        let seeds = [Some(7), Some(11)];
        let boundaries = batch.reset(&seeds).unwrap();
        for (slot, single) in singles.iter_mut().enumerate() {
            assert_eq!(boundaries[slot].observation, single.reset(seeds[slot]).unwrap().observation);
        }
        let mut tokens = boundaries.iter().map(|row| row.token).collect::<Vec<_>>();
        for _ in 0..12 {
            let indices = vec![RunActionIndex(0); 2];
            let rows = batch.step(&tokens, &indices).unwrap();
            for (slot, single) in singles.iter_mut().enumerate() {
                let one = single.step(single.decision().unwrap().token, indices[slot]).unwrap();
                assert_eq!(
                    (&rows[slot].boundary.observation, rows[slot].boundary.terminal, rows[slot].reward, rows[slot].terminated, rows[slot].truncated),
                    (&one.boundary.observation, one.boundary.terminal, one.reward, one.terminated, one.truncated),
                    "batch row order and value at slot {slot}",
                );
            }
            tokens = rows.iter().map(|row| row.boundary.token).collect();
        }
        for slot in 0..2 {
            assert_eq!(batch.record(slot).unwrap(), singles[slot].record().unwrap());
            assert_eq!(batch.reproduction(slot).unwrap(), singles[slot].reproduction().unwrap());
        }

    }
}

#[test]
fn m25_batch_preflight_full_reset() {
    for workers in [1, 2] {
        assert_m25_offer_group();
        let configs = vec![m25_config(), m25_config()];
        let mut batch = RunBatch::new_m25(configs.clone(), workers).unwrap();
        let seeds = [Some(7), Some(11)];
        let mut singles = configs.iter().cloned().map(RunSession::new_m25).collect::<Result<Vec<_>, _>>().unwrap();
        let first = batch.reset(&seeds).unwrap();
        for (slot, single) in singles.iter_mut().enumerate() {
            assert_eq!(first[slot].observation, single.reset(seeds[slot]).unwrap().observation);
        }
        let before = [batch.reproduction(0).unwrap(), batch.reproduction(1).unwrap()];
        let tokens = [first[0].token, first[1].token];
        assert!(matches!(batch.step(&tokens[..1], &[RunActionIndex(0)]), Err(RunBatchError::Input { slot: None, .. })));
        assert!(matches!(batch.reset(&[Some(7)]), Err(RunBatchError::Input { slot: None, .. })));
        assert!(matches!(batch.step(&tokens, &[RunActionIndex(0), RunActionIndex(-1)]), Err(RunBatchError::Input { slot: Some(1), .. })));
        assert!(matches!(batch.step(&[tokens[0], RunDecisionToken::from([0, 0])], &[RunActionIndex(0); 2]), Err(RunBatchError::Input { slot: Some(1), error: RunSessionError::StaleToken })));
        assert_eq!([batch.reproduction(0).unwrap(), batch.reproduction(1).unwrap()], before, "invalid whole-batch calls do not mutate peers");

        let rows = batch.step(&tokens, &[RunActionIndex(0); 2]).unwrap();
        assert_eq!(rows.len(), 2);
        for single in &mut singles {
            single.step(single.decision().unwrap().token, RunActionIndex(0)).unwrap();
        }
        let reset = batch.reset(&[None, Some(13)]).unwrap();
        let reset_seeds = [None, Some(13)];
        for (slot, single) in singles.iter_mut().enumerate() {
            assert_eq!(reset[slot].observation, single.reset(reset_seeds[slot]).unwrap().observation);
            assert_eq!(batch.reproduction(slot).unwrap(), single.reproduction().unwrap(), "full reset scalar state at slot {slot}");
        }
        assert_ne!(reset[0].token, rows[0].boundary.token);
        assert_eq!((batch.record(0).unwrap().unwrap().seed, batch.record(0).unwrap().unwrap().ordinal), (7, 1));
        assert_eq!((batch.record(1).unwrap().unwrap().seed, batch.record(1).unwrap().unwrap().ordinal), (13, 0));
        assert!(reset.iter().all(|row| row.observation.action_mask.iter().any(|value| *value == 1)));
        batch.close().unwrap();
    }
}

#[test]
fn m25_coexistence() {
    let mut m25 = RunBatch::new_m25(vec![m25_config()], 1).unwrap();
    let mut m24 = RunBatch::new_m24(vec![m24_config()], 1).unwrap();
    let mut m15 = RunBatch::new_m15(vec![m15_config()], 1).unwrap();
    let mut m20 = RunBatch::new_m20(vec![m20_config()], 1).unwrap();
    let m25_descriptor = RunSession::new_m25(m25_config()).unwrap().observation_spec().unwrap().descriptor();
    assert_eq!(m25_descriptor["semantics"]["finite_content"]["permanent_cards"].as_array().unwrap().len(), 76);
    let rows = [
        ("M25WatcherHeart", RoomScenario::M25WatcherHeart, m25.reset(&[Some(7)]).unwrap()),
        ("M24WatcherAct3", RoomScenario::M24WatcherAct3, m24.reset(&[Some(7)]).unwrap()),
        ("M15SilentHeart", RoomScenario::M15SilentHeart, m15.reset(&[Some(7)]).unwrap()),
        ("M20DefectHeart", RoomScenario::M20DefectHeart, m20.reset(&[Some(7)]).unwrap()),
    ];
    for (profile, scenario, boundaries) in rows {
        assert_eq!(boundaries.len(), 1);
        let (record, reproduction) = match profile {
            "M25WatcherHeart" => (m25.record(0).unwrap().unwrap(), m25.reproduction(0).unwrap()),
            "M24WatcherAct3" => (m24.record(0).unwrap().unwrap(), m24.reproduction(0).unwrap()),
            "M15SilentHeart" => (m15.record(0).unwrap().unwrap(), m15.reproduction(0).unwrap()),
            _ => (m20.record(0).unwrap().unwrap(), m20.reproduction(0).unwrap()),
        };
        assert_eq!(record.profile_id, profile);
        assert_eq!(reproduction.latest_decision.unwrap().unwrap().content_profile, Some(scenario));
    }
    for batch in [&mut m25, &mut m24, &mut m15, &mut m20] { batch.close().unwrap(); }
}

#[test]
fn m25_batch_postdispatch_faults() {
    for workers in [1, 2] {
        let cfg = m25_config();
        let mut small = cfg.clone();
        small.capacities.map_nodes = 1;
        let mut batch = RunBatch::new_m25(vec![cfg, small], workers).unwrap();
        let initial = batch.reset(&[Some(7), Some(7)]).unwrap();
        let tokens = initial.iter().map(|row| row.token).collect::<Vec<_>>();
        let RunBatchError::Fault { slots } = batch
            .step(&tokens, &[RunActionIndex(0); 2])
            .unwrap_err()
        else {
            panic!("expected indexed post-dispatch capacity fault")
        };
        assert_eq!(slots.len(), 2);
        assert_eq!(
            (slots[0].slot, slots[0].committed, slots[0].delivery_lost),
            (0, true, true),
        );
        assert_eq!(
            (slots[1].slot, slots[1].committed, slots[1].delivery_lost),
            (1, true, false),
        );
        assert_eq!(
            slots[1].error,
            RunSessionError::DeliveryFault("post-step Run encoding failed"),
        );
        let first = batch.record(0).unwrap().unwrap();
        let second = batch.record(1).unwrap().unwrap();
        assert_eq!((first.steps.len(), second.steps.len()), (1, 1));
        let first_fault = first.fault.as_ref().unwrap();
        assert!(first_fault.committed && first_fault.gameplay_recorded);
        assert_eq!(first_fault.error.kind, RunRecordErrorKind::DeliveryFault);
        let second_fault = second.fault.as_ref().unwrap();
        assert!(second_fault.committed && second_fault.gameplay_recorded);
        assert_eq!(second_fault.error.kind, RunRecordErrorKind::Capacity);
        let capacity = second_fault.error.capacity.as_ref().unwrap();
        assert_eq!(capacity.group, "map_nodes");
        assert!(capacity.required > capacity.capacity);
        assert_eq!(capacity.capacity, 1);
        for record in [&first, &second] {
            let decoded = RunEpisodeRecord::from_json(&record.to_json().unwrap()).unwrap();
            assert_eq!(&decoded, record);
            assert_eq!(replay_run(&decoded, RunCapacities::m10_default()).unwrap().accepted_count, 1);
        }
        let before = [batch.reproduction(0).unwrap(), batch.reproduction(1).unwrap()];
        assert!(batch.step(&tokens, &[RunActionIndex(0); 2]).is_err());
        assert!(batch.reset(&[None, None]).is_err());
        assert_eq!([batch.reproduction(0).unwrap(), batch.reproduction(1).unwrap()], before);
        batch.close().unwrap();
        assert!(batch.record(0).unwrap().unwrap().closed);
        assert!(batch.record(1).unwrap().unwrap().closed);
    }
}
