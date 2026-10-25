use spire_env::*;
use spire_core::{combat::rng::Partition,content::Character,run::*};
#[path="../../spire-core/examples/m9_support/mod.rs"] mod support;
fn config(history:M9BossHistory)->RunSessionConfig{RunSessionConfig{character:Character::Ironclad,ascension:0,partition:Partition::Test,boss_history:history,capacities:RunCapacities::m9_default(),record_replay:true}}
fn prefix()->RunEpisodeRecord{
    let mut s=RunSession::new(config(M9BossHistory::AwakenedUnseen)).unwrap();let b=s.reset(Some(11)).unwrap();s.step(b.token,RunActionIndex(0)).unwrap();s.record().unwrap().unwrap()
}

#[test]
fn corrected_rules_record_keeps_v1_wire_and_observation_identity(){
    let record=prefix();
    assert_eq!(record.schema_id,"spire-gym-run-replay-v1");
    assert_eq!(record.rules_id,RUN_RULES_ID);
    assert_eq!(RUN_RULES_ID,"spire-gym-native-m9-rules-v2");
    assert_eq!(RUN_RULES_ID_V1,"spire-gym-native-m9-rules-v1");
    assert_eq!(record.observation_fingerprint.iter().map(|b|format!("{b:02x}")).collect::<String>(),
        "0893bce86e4b3abf11aa384c501503a91ae37f688ec605ab40b814ad365cb999");
    let encoded=record.to_json().unwrap();
    let decoded=RunEpisodeRecord::from_json(&encoded).unwrap();
    assert_eq!(decoded,record);
    assert_eq!(serde_json::from_slice::<serde_json::Value>(&encoded).unwrap()["schema_id"],"spire-gym-run-replay-v1");
}

#[test]
fn explicitly_labelled_legacy_v1_rules_record_fails_identity_preflight(){
    // This in-memory fixture emulates the old v1 semantic identity. It is not a
    // saved historical record and does not rewrite or bless an old artifact.
    let mut legacy_v1_rules_record_fixture=prefix();
    legacy_v1_rules_record_fixture.rules_id=RUN_RULES_ID_V1.into();
    // If replay advanced to action execution, this action would fail as unoffered.
    // Identity mismatch must win at strict preflight before Run construction/step.
    legacy_v1_rules_record_fixture.steps[0].action=GameAction::Move(NodeId(u64::MAX));
    assert!(matches!(replay_run(&legacy_v1_rules_record_fixture,RunCapacities::m9_default()),
        Err(RunSessionError::IncompatibleReplay("Run identity mismatch"))));
}

#[test]
fn run_replay_reconstructs_real_start_and_semantic_actions(){
    let contexts=[(M9BossHistory::AwakenedUnseen,support::M9BossHistory::AwakenedUnseen),(M9BossHistory::TimeEaterUnseen,support::M9BossHistory::TimeEaterUnseen),(M9BossHistory::DonuDecaUnseen,support::M9BossHistory::DonuDecaUnseen)];
    for (history,example) in contexts{
        let mut s=RunSession::new(config(history)).unwrap();let mut b=s.reset(Some(11)).unwrap();let mut core=support::start(11,example).unwrap();
        let trajectory=support::complete(11,example,8192).unwrap();
        for step in &trajectory.trace{
            let index=core.decision().unwrap().candidates.iter().position(|c|c.action==step.action).unwrap();
            let t=s.step(b.token,RunActionIndex(index as i64)).unwrap();assert_eq!((t.reward,t.boundary.terminal),(step.reward,step.decision.terminal));
            assert_eq!(t.boundary.observation,encode_run(&step.decision,step.map.as_ref(),&config(history).capacities).unwrap());b=t.boundary;core.step(&step.action).unwrap();
        }
        let record=s.record().unwrap().unwrap();assert_eq!(record.steps.iter().map(|s|s.action.clone()).collect::<Vec<_>>(),trajectory.trace.iter().map(|s|s.action.clone()).collect::<Vec<_>>());
        let bytes=record.to_json().unwrap();let decoded=RunEpisodeRecord::from_json(&bytes).unwrap();assert_eq!(decoded,record);
        let replay=replay_run(&decoded,RunCapacities::m9_default()).unwrap();assert_eq!(replay.final_decision,trajectory.final_decision);assert_eq!(replay.final_map,trajectory.final_map);assert_eq!(replay.accepted_count,trajectory.trace.len());
        assert_eq!(replay.rewards,trajectory.trace.iter().map(|s|s.reward).collect::<Vec<_>>());assert_eq!(replay.terminals,trajectory.trace.iter().map(|s|s.decision.terminal).collect::<Vec<_>>());assert_eq!(replay.reported_fault,None);
        // Compact prefixes compare each public decision/map, not just a digest.
        let mut core=support::start(11,example).unwrap();for count in 0..12{
            let mut prefix=record.clone();prefix.steps.truncate(count);prefix.terminal=None;let replay=replay_run(&prefix,RunCapacities::m9_default()).unwrap();assert_eq!(replay.final_decision,core.decision().unwrap());assert_eq!(replay.final_map,core.public_map());
            core.step(&record.steps[count].action).unwrap();
        }
    }
}

#[test]
fn run_replay_rejects_incompatible_identity_or_unoffered_action(){
    let record=prefix();
    for field in 0..8{let mut bad=record.clone();match field{0=>bad.schema_id.push('x'),1=>bad.rules_id.push('x'),2=>bad.content_id.push('x'),3=>bad.profile_id.push('x'),4=>bad.seed_adapter_id.push('x'),5=>bad.rng_id.push('x'),6=>bad.reward_id.push('x'),_=>bad.observation_fingerprint[0]^=1};assert!(matches!(replay_run(&bad,RunCapacities::m9_default()),Err(RunSessionError::IncompatibleReplay(_))));}
    let mut bad=record.clone();bad.steps[0].action=GameAction::Move(NodeId(u64::MAX));assert!(matches!(replay_run(&bad,RunCapacities::m9_default()),Err(RunSessionError::IncompatibleReplay(_))));
    let mut bad=record.clone();bad.steps[0].reward+=1;assert!(replay_run(&bad,RunCapacities::m9_default()).is_err());
    let mut wire:serde_json::Value=serde_json::from_slice(&record.to_json().unwrap()).unwrap();wire["seed"]=serde_json::json!(true);assert!(RunEpisodeRecord::from_json(&serde_json::to_vec(&wire).unwrap()).is_err());
}

#[test]
fn run_replay_keeps_fault_prefix_without_fabricated_loss_or_repeating_action(){
    let mut cfg=config(M9BossHistory::AwakenedUnseen);cfg.capacities.map_nodes=1;
    let mut s=RunSession::new(cfg).unwrap();let b=s.reset(Some(11)).unwrap();assert!(s.step(b.token,RunActionIndex(0)).is_err());
    let record=s.record().unwrap().unwrap();let record=RunEpisodeRecord::from_json(&record.to_json().unwrap()).unwrap();
    let replay=replay_run(&record,RunCapacities::m9_default()).unwrap();assert_eq!(replay.accepted_count,1);assert_eq!(replay.rewards,[0]);assert_eq!(replay.final_decision.terminal,None);assert_eq!(replay.reported_fault,record.fault);assert_eq!(replay.final_decision.room.unwrap().kind,RoomViewKind::Map);
    let mut s=RunSession::new(config(M9BossHistory::AwakenedUnseen)).unwrap();s.reset(Some(11)).unwrap();s.fail_delivery("Python reset packaging failed").unwrap();s.close().unwrap();
    let record=s.record().unwrap().unwrap();assert!(record.closed);let replay=replay_run(&RunEpisodeRecord::from_json(&record.to_json().unwrap()).unwrap(),RunCapacities::m9_default()).unwrap();assert_eq!(replay.accepted_count,0);assert!(replay.reported_fault.is_some());assert_eq!(replay.final_decision.terminal,None);
}

#[test]
fn run_replay_wire_covers_all_current_action_variants_without_discriminant_casts(){
    use spire_core::combat::{CombatAction as A,TargetId};
    let actions=vec![GameAction::Move(NodeId(4)),GameAction::Choose(ChoiceId(8)),GameAction::Select(SelectionId(12)),GameAction::Confirm,GameAction::Cancel,GameAction::UsePotion{slot:2,target:None},GameAction::DiscardPotion{slot:1},GameAction::Combat(A::PlayCard{instance:u64::MAX,target:Some(TargetId(1))}),GameAction::Combat(A::SelectCard{instance:24}),GameAction::Combat(A::UsePotion{slot:0,target:Some(TargetId(2))}),GameAction::Combat(A::EndTurn)];
    let mut record=prefix();record.steps=actions.into_iter().map(|action|RunRecordedStep{action,reward:0,terminal:None}).collect();assert_eq!(RunEpisodeRecord::from_json(&record.to_json().unwrap()).unwrap(),record);
    // Wire round-trip isn't gameplay acceptance: this deliberately unoffered script fails replay.
    assert!(replay_run(&record,RunCapacities::m9_default()).is_err());
}

#[test]
fn frozen_replay_v1_action_encoder_rejects_confirmation_and_keeps_old_roundtrip(){
    let valid=prefix();let old=valid.to_json().unwrap();
    assert_eq!(RunEpisodeRecord::from_json(&old).unwrap(),valid);

    let mut candidate=valid.clone();candidate.steps.push(RunRecordedStep{action:GameAction::Combat(spire_core::combat::CombatAction::ConfirmSelection),reward:0,terminal:None});
    assert_eq!(candidate.to_json(),Err(RunSessionError::UnsupportedBoundary("ConfirmSelection is not representable in replay v1")));
    assert_eq!(replay_run(&candidate,RunCapacities::m9_default()),Err(RunSessionError::IncompatibleReplay("ConfirmSelection is not representable in replay v1")));
}

#[test]
fn frozen_replay_v1_decoder_rejects_confirmation_input_explicitly(){
    let valid=prefix();let old=valid.to_json().unwrap();
    let mut wire:serde_json::Value=serde_json::from_slice(&old).unwrap();
    wire["steps"][0]["action"]=serde_json::json!({"verb":"combat.confirm_selection"});
    assert_eq!(RunEpisodeRecord::from_json(&serde_json::to_vec(&wire).unwrap()),
        Err(RunSessionError::IncompatibleReplay("ConfirmSelection is not representable in replay v1")));
}

#[test]
fn run_replay_reseed_and_none_continuation_use_the_recorded_episode_ordinal(){
    let mut s=RunSession::new(config(M9BossHistory::DonuDecaUnseen)).unwrap();
    for seed in [Some(7),None,Some(11),None]{
        let b=s.reset(seed).unwrap();s.step(b.token,RunActionIndex(0)).unwrap();
        let record=s.record().unwrap().unwrap();let expected=s.reproduction().unwrap();
        let decoded=RunEpisodeRecord::from_json(&record.to_json().unwrap()).unwrap();let replay=replay_run(&decoded,RunCapacities::m9_default()).unwrap();
        assert_eq!(replay.final_decision,expected.latest_decision.unwrap().unwrap());assert_eq!(replay.final_map,expected.latest_map);assert_eq!(replay.reported_fault,None);
    }
}
