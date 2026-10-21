use super::support::*;
use crate::*;
use crate::episode::EpisodeSequence;
use spire_core::combat::{CardKind, EnemyKind, MoveId};
use spire_core::combat::rng::Partition;

// Catches decimal-text hashing, missing length framing, truncation, and carry bugs.
#[test]
fn seed_adapter_golden_roots() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!("fixtures/seed_adapter_v1.json")).unwrap();
    for vector in fixture["vectors"].as_array().unwrap() {
        let bytes: Vec<u8> = vector["bytes"].as_array().unwrap().iter().map(|v| v.as_u64().unwrap() as u8).collect();
        let seed = IntegerSeed::from_be_bytes(&bytes).unwrap();
        assert_eq!(hex(&seed.root()), vector["root_hex"]);
        let mut padded = vec![0, 0];
        padded.extend(&bytes);
        assert_eq!(IntegerSeed::from_be_bytes(&padded).unwrap(), seed);
    }
    assert!(IntegerSeed::from_be_bytes(&[]).is_err());
    assert_eq!(IntegerSeed::from_be_bytes(&[0,0]).unwrap().as_be_bytes(), [0]);
    let s = IntegerSeed::from_be_bytes(&[255]).unwrap();
    let expanded = expand_seeds(SeedBatch::Scalar(s), 3).unwrap();
    assert_eq!(expanded.iter().map(|s| s.as_ref().unwrap().as_be_bytes()).collect::<Vec<_>>(), [vec![255], vec![1,0], vec![1,1]]);
    let wide = IntegerSeed::from_be_bytes(&[255; 33]).unwrap().add_slot(1).unwrap();
    let mut want = vec![0; 34]; want[0] = 1;
    assert_eq!(wide.as_be_bytes(), want);
    assert_eq!(IntegerSeed::from_be_bytes(&[0]).unwrap().add_slot(usize::MAX).unwrap().as_be_bytes(), usize::MAX.to_be_bytes());
    assert_eq!(expand_seeds(SeedBatch::Continue, 2).unwrap(), [None, None]);
    let explicit = vec![Some(IntegerSeed::from_be_bytes(&[1]).unwrap()), None];
    assert_eq!(expand_seeds(SeedBatch::PerSlot(explicit.clone()), 2).unwrap(), explicit);
    assert!(matches!(expand_seeds(SeedBatch::PerSlot(vec![]), 2), Err(EnvError::Input { reason: InputReason::Shape, .. })));
    assert!(matches!(expand_seeds(SeedBatch::Continue, usize::MAX), Err(EnvError::Setup { .. })));
}

// Catches accidental entropy on continuation, early commits, and ordinal reuse.
#[test]
fn sequence_stages_entropy_reseed_and_continuation() {
    let catalog = ScenarioCatalog::starter();
    let initial = EpisodeSequence::default();
    let mut calls = 0;
    let mut entropy = |bytes: &mut [u8;32]| { calls += 1; bytes.fill(0); Ok(()) };
    let first = initial.prepare_reset_with_entropy(None, None, &catalog, Partition::Train, &mut entropy).unwrap();
    assert_eq!(initial.next_ordinal(), None);
    assert_eq!(first.job.ordinal(), 0);
    assert_eq!(first.effective_seed.as_ref().unwrap().as_be_bytes(), [0]);
    let second = first.sequence.prepare_reset_with_entropy(None, None, &catalog, Partition::Train, &mut entropy).unwrap();
    assert_eq!(second.job.ordinal(), 1);
    assert_eq!(second.job.seed(), first.job.seed());
    assert_eq!(second.effective_seed, None);
    assert_eq!(calls, 1);
    let mut fail = |_: &mut [u8;32]| Err(EnvError::Setup { slot: None, message: "scripted entropy failure".into() });
    assert!(initial.prepare_reset_with_entropy(None, None, &catalog, Partition::Train, &mut fail).is_err());
    assert_eq!(initial.next_ordinal(), None);
    // Invalid scenario must fail before even consulting entropy.
    assert!(matches!(initial.prepare_reset_with_entropy(None, Some("missing"), &catalog, Partition::Train, &mut fail), Err(EnvError::Input { reason: InputReason::Scenario, .. })));
    let seed = IntegerSeed::from_be_bytes(&[1]).unwrap();
    let a = second.sequence.prepare_reset(Some(&seed), None, &catalog, Partition::Train).unwrap();
    let b = a.sequence.prepare_reset(Some(&seed), None, &catalog, Partition::Train).unwrap();
    assert_eq!(a.job, b.job);
    assert_eq!(a.job.episode_id(), "episode-0");
    assert_eq!(a.effective_seed, Some(seed));
}

// Catches omitted bounds/ID/context validation before immutable ownership.
#[test]
fn catalog_validation_and_schedule() {
    assert!(ScenarioCatalog::new(vec![]).is_err());
    assert!(ScenarioCatalog::new(vec![template(), template()]).is_err());
    for id in ["", "-bad", "has space", "bad/", "é", &"a".repeat(65)] {
        let mut t = template(); t.id = id.into();
        assert!(ScenarioCatalog::new(vec![t]).is_err(), "{id}");
    }
    for (field, value) in [("hp",0), ("hp",1000), ("max",0), ("max",1000), ("energy",0), ("energy",1000), ("draw",0), ("draw",11), ("enemy",47), ("enemy",55)] {
        let mut t = template();
        match field { "hp" => t.player_hp=value, "max" => t.player_max_hp=value, "energy" => t.energy_per_turn=value, "draw" => t.draw_per_turn=value, _ => t.enemy_hp=value }
        assert!(ScenarioCatalog::new(vec![t]).is_err(), "{field}={value}");
    }
    let mut t = template(); t.player_max_hp=79;
    assert!(ScenarioCatalog::new(vec![t]).is_err());
    for hp in [39,45] { let mut t=template(); t.encounter=EnemyKind::JawWorm; t.enemy_hp=hp; assert!(ScenarioCatalog::new(vec![t]).is_err()); }
    let mut t=template(); t.context.act=Some(2); assert!(ScenarioCatalog::new(vec![t]).is_err());
    for count in [0,1000] { let mut t=template(); t.hand.clear(); t.draw=vec![CardSpec {kind: CardKind::Strike, upgraded:false};count]; assert!(ScenarioCatalog::new(vec![t]).is_err()); }
    let mut t=template(); t.hand=vec![t.hand[0].clone();11]; assert!(ScenarioCatalog::new(vec![t]).is_err());
    let mut t=template(); t.id="A_.-09".into(); t.player_hp=999; t.player_max_hp=999; t.energy_per_turn=999; t.draw_per_turn=10;
    t.hand=[CardKind::Strike,CardKind::Defend,CardKind::Bash].into_iter().cycle().take(10).map(|kind| CardSpec {kind, upgraded:true}).collect();
    t.draw=vec![t.hand[0].clone();989]; t.context=PublicContext {act:Some(1),next_encounter:Some(EnemyKind::JawWorm)};
    let large=ScenarioCatalog::new(vec![t]).unwrap();
    let seed=IntegerSeed::from_be_bytes(&[1]).unwrap();
    assert!(EpisodeSequence::default().prepare_reset(Some(&seed),None,&large,Partition::Test).unwrap().job.instantiate().is_ok());
    for (enemy, bounds) in [(EnemyKind::Cultist,[48,54]),(EnemyKind::JawWorm,[40,44])] {
        for hp in bounds { let mut t=template(); t.encounter=enemy; t.enemy_hp=hp; ScenarioCatalog::new(vec![t]).unwrap(); }
    }
    let catalog=ScenarioCatalog::starter();
    let mut staged=EpisodeSequence::default().prepare_reset(Some(&seed),None,&catalog,Partition::Train).unwrap();
    for n in 0..5 {
        assert_eq!(staged.job.ordinal(),n);
        assert_eq!(staged.job.template().id, if n%2==0 {"starter-cultist-a0-v1"} else {"starter-jaw-worm-a0-v1"});
        staged=staged.sequence.prepare_reset(None,None,&catalog,Partition::Train).unwrap();
    }
    let override_job=staged.sequence.prepare_reset(None,Some("starter-jaw-worm-a0-v1"),&catalog,Partition::Train).unwrap();
    assert_eq!(override_job.job.ordinal(),6);
    assert_eq!(override_job.job.template().id,"starter-jaw-worm-a0-v1");
    assert_eq!(override_job.sequence.next_ordinal(),Some(7));
    let after_override=override_job.sequence.prepare_reset(None,None,&catalog,Partition::Train).unwrap();
    assert_eq!(after_override.job.ordinal(),7);
    assert_eq!(after_override.job.template().id,"starter-jaw-worm-a0-v1");
    assert!(override_job.sequence.prepare_reset(None,Some("bad"),&catalog,Partition::Train).is_err());
    assert_eq!(override_job.sequence.next_ordinal(),Some(7));
    let last=crate::episode::exhaustion_test_sequence(&seed);
    let last_job=last.prepare_reset(None,None,&catalog,Partition::Train).unwrap();
    assert_eq!(last_job.job.ordinal(),u64::MAX);
    assert_eq!(last_job.job.episode_id(),"episode-18446744073709551615");
    assert_eq!(last_job.sequence.next_ordinal(),None);
    assert!(matches!(last_job.sequence.prepare_reset(None,None,&catalog,Partition::Train),Err(EnvError::Setup {..})));
    assert_eq!(last.next_ordinal(),Some(u64::MAX));
    assert_eq!(last_job.sequence.prepare_reset(Some(&seed),None,&catalog,Partition::Train).unwrap().job.ordinal(),0);
}

// Catches slot-dependent derivation, wrong domain/scope/partition and implicit entry draws.
#[test]
fn logical_jobs_streams_and_partition_isolation() {
    use spire_core::combat::rng::Domain;
    let fixture:serde_json::Value=serde_json::from_str(include_str!("fixtures/seed_adapter_v1.json")).unwrap();
    let seed=IntegerSeed::from_be_bytes(&[1]).unwrap();
    let catalog=ScenarioCatalog::starter();
    let job=EpisodeSequence::default().prepare_reset(Some(&seed),None,&catalog,Partition::Train).unwrap().job;
    assert_eq!(job.catalog_fingerprint(),catalog.fingerprint());
    let scenario=job.scenario().unwrap();
    assert_eq!(scenario.rng.shuffle.id().domain(),Domain::CombatShuffle);
    assert_eq!(scenario.rng.enemy_ai.id().domain(),Domain::CombatEnemyAi);
    assert_eq!(scenario.rng.shuffle.id().scope(),["starter-cultist-a0-v1"]);
    assert_eq!(scenario.rng.enemy_ai.id().scope(),["starter-cultist-a0-v1","enemy-0"]);
    assert_eq!(scenario.rng.enemy_instance_id.as_str(),"enemy-0");
    assert_eq!(scenario.rng.shuffle.word_index(),0);
    assert_eq!(scenario.rng.enemy_ai.word_index(),0);
    assert_eq!(hex(scenario.rng.shuffle.key()),fixture["episode_0_seed_1"]["shuffle_key_hex"]);
    assert_eq!(hex(scenario.rng.enemy_ai.key()),fixture["episode_0_seed_1"]["enemy_ai_key_hex"]);
    let mut slots=[None,None,None]; slots[0]=Some(job.clone()); slots.swap(0,2);
    assert_eq!(slots[2].as_ref().unwrap().instantiate().unwrap().snapshot(),job.instantiate().unwrap().snapshot());
    let mut keys=vec![];
    for partition in [Partition::Train,Partition::Selection,Partition::Final,Partition::Test] {
        let job=EpisodeSequence::default().prepare_reset(Some(&seed),None,&catalog,partition).unwrap().job;
        assert_eq!(job.partition(),partition);
        assert_eq!(job.root(),seed.root());
        keys.push(*job.scenario().unwrap().rng.shuffle.key());
    }
    keys.sort(); keys.dedup(); assert_eq!(keys.len(),4);
}

// Catches omitted/default fields, altered catalog order, float acceptance and noncanonical keys.
#[test]
fn authored_descriptor_and_canonical_fingerprint() {
    use sha2::{Digest,Sha256};
    assert_eq!(hex(&Sha256::digest(include_bytes!("fixtures/scenario_catalog_v1.json"))),"806d2ba6c53aef4d817b65e12981ae27ea9ea999be597e875f47d55bf70e7008");
    assert_eq!(hex(&Sha256::digest(include_bytes!("fixtures/seed_adapter_v1.json"))),"31b02e0e56aac9348a7715b189a1d35c071ac85e1a54312da1e148b8d4497be1");
    let fixture:serde_json::Value=serde_json::from_str(include_str!("fixtures/scenario_catalog_v1.json")).unwrap();
    let catalog=ScenarioCatalog::starter();
    assert_eq!(catalog.descriptor(),fixture["descriptor"]);
    assert_eq!(hex(&catalog.fingerprint()),fixture["fingerprint_hex"]);
    assert_eq!(canonical_json(&serde_json::json!({"z":[null,true,2],"a":{"β":"é","a":-1}})).unwrap(), "{\"a\":{\"a\":-1,\"β\":\"é\"},\"z\":[null,true,2]}".as_bytes());
    assert!(canonical_json(&serde_json::json!({"nested":[1.0]})).is_err());
    assert!(canonical_json(&serde_json::json!(1.5)).is_err());
    let a=template(); let mut b=template(); b.id="second".into();
    assert_ne!(ScenarioCatalog::new(vec![a.clone(),b.clone()]).unwrap().fingerprint(),ScenarioCatalog::new(vec![b,a]).unwrap().fingerprint());
}

// Catches incorrect authored ready states, pile reordering, and mutable catalog aliases.
#[test]
fn starter_catalog_is_exact_and_owned() {
    let catalog = ScenarioCatalog::starter();
    let descriptor = catalog.descriptor();
    assert_eq!(descriptor["templates"][0]["enemy_hp"], 52);
    assert_eq!(descriptor["templates"][1]["enemy_hp"], 42);
    assert_eq!(descriptor["templates"][0]["context"]["act"], serde_json::Value::Null);
    assert_eq!(descriptor["templates"][0]["context"]["next_encounter"], serde_json::Value::Null);
    let sequence = EpisodeSequence::default();
    for (n, id, enemy, hp, intent) in [
        (0, "starter-cultist-a0-v1", EnemyKind::Cultist, 52, MoveId::Incantation),
        (1, "starter-jaw-worm-a0-v1", EnemyKind::JawWorm, 42, MoveId::Chomp),
    ] {
        let staged = sequence.prepare_reset(Some(&IntegerSeed::from_be_bytes(&[0]).unwrap()), Some(id), &catalog, Partition::Train).unwrap();
        let scenario = staged.job.scenario().unwrap();
        assert_eq!(descriptor["templates"][n]["id"], id);
        assert_eq!(scenario.encounter_id, enemy);
        assert_eq!((scenario.player.hp, scenario.player.max_hp, scenario.player.energy, scenario.player.max_energy, scenario.player.draw_per_turn), (80,80,3,3,5));
        assert_eq!(scenario.player.block, 0);
        assert!(scenario.player.powers.is_empty());
        assert_eq!(scenario.enemies[0].hp, hp);
        assert_eq!(scenario.enemies[0].max_hp, hp);
        assert_eq!(scenario.enemies[0].block, 0);
        assert!(scenario.enemies[0].powers.is_empty());
        assert_eq!(scenario.enemies[0].intent_id, intent);
        assert_eq!(scenario.enemies[0].move_history, [intent]);
        assert_eq!(scenario.enemies[0].instance_id.as_str(), "enemy-0");
        assert_eq!(scenario.piles.hand.iter().map(|c| c.card_id).collect::<Vec<_>>(), [CardKind::Strike, CardKind::Defend, CardKind::Strike, CardKind::Defend, CardKind::Bash]);
        assert_eq!(scenario.piles.draw.iter().map(|c| c.card_id).collect::<Vec<_>>(), [CardKind::Strike, CardKind::Strike, CardKind::Strike, CardKind::Defend, CardKind::Defend]);
        for (i, card) in scenario.piles.hand.iter().chain(&scenario.piles.draw).enumerate() {
            assert_eq!(card.instance_id.as_str(), i.to_string());
            assert!(!card.upgraded);
        }
        assert!(scenario.piles.discard.is_empty() && scenario.piles.exhaust.is_empty());
        assert!(staged.job.instantiate().unwrap().public_view().unwrap().revealed_order.is_empty());
    }
    let mut changed = descriptor.clone();
    changed["templates"][0]["enemy_hp"] = 999.into();
    assert_eq!(catalog.descriptor(), descriptor);
    let mut input = template();
    let owned = ScenarioCatalog::new(vec![input.clone()]).unwrap();
    input.hand.clear();
    input.id.clear();
    assert_eq!(owned.descriptor()["templates"][0]["id"], "authored");
    assert_eq!(owned.descriptor()["templates"][0]["hand"].as_array().unwrap().len(), 1);
}
