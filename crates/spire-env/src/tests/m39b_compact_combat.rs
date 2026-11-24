use crate::*;
use crate::combat_policy::{StandardCombatCapacityPreset, standard_compact_combat_observation_spec};

#[test]
fn compact_combat_descriptor_fixture() {
    let caps = RunCapacities::standard_default();
    let bounds = standard_compact_combat_observation_spec(&caps, StandardCombatCapacityPreset::Bounds).unwrap();
    let play = standard_compact_combat_observation_spec(&caps, StandardCombatCapacityPreset::Play).unwrap();
    assert_eq!(bounds.descriptor()["schema_id"], "spire-gym-standard-combat-view-compact-v1");
    assert_ne!(bounds.fingerprint(), play.fingerprint());
    assert_eq!(bounds.descriptor()["tensors"][0]["dtype"], "int16");
    for (preset, caps, fixture, payload_bytes) in [
        (StandardCombatCapacityPreset::Bounds, RunCapacities::standard_combat_compact(),
         include_str!("fixtures/m39b-standard-combat-compact-v1.json"), 229_926),
        (StandardCombatCapacityPreset::Play, RunCapacities::standard_combat_compact_play(),
         include_str!("fixtures/m39b-standard-combat-compact-play-v1.json"), 143_313),
    ] {
        use sha2::{Digest, Sha256};
        let spec = standard_compact_combat_observation_spec(&caps, preset).unwrap();
        let descriptor = spec.descriptor();
        let expected = serde_json::from_str::<serde_json::Value>(fixture).unwrap();
        assert!(descriptor == expected["descriptor"], "descriptor for {}", preset.as_str());
        let fingerprint = crate::combat_policy::hex(&spec.fingerprint());
        assert_eq!(expected["fingerprint"], fingerprint);
        let mut digest = Sha256::new();
        digest.update(b"spire-gym/combat-view/standard/compact/v1\0");
        digest.update(canonical_json(&descriptor).unwrap());
        assert_eq!(&digest.finalize()[..], &spec.fingerprint());
        let bytes = descriptor["tensors"].as_array().unwrap().iter().map(|t| {
            let width = match t["dtype"].as_str().unwrap() { "int8" => 1, "int16" => 2, "int32" => 4, dtype => panic!("{dtype}") };
            width * t["shape"].as_array().unwrap().iter().map(|n|n.as_u64().unwrap()).product::<u64>()
        }).sum::<u64>();
        assert_eq!(bytes, payload_bytes);
    }

}

#[test]
fn compact_combat_direct_target() {
    use std::{mem::MaybeUninit, sync::atomic::{AtomicBool, Ordering}};
    use crate::run_schema_standard::{compact_dtype, ValueDtype};
    use spire_core::content::Character;
    use super::m34s_transplant::{standard_combat_session, standard_entry};

    let mut session = standard_combat_session(vec![standard_entry("test", Character::Silent, 20, "jaw_worm", None)]);
    let dense = session.reset(Some(1), None).unwrap().observation;
    let run = session.run_for_test().unwrap();
    let d = run.decision().unwrap();
    let caps = RunCapacities::standard_default();
    let mut short = dense.groups.iter().map(|g| vec![MaybeUninit::new(123_i16); g.values.len()]).collect::<Vec<_>>();
    let mut wide = dense.groups.iter().map(|g| vec![MaybeUninit::new(123_i32); g.values.len()]).collect::<Vec<_>>();
    let chunks = dense.groups.iter().zip(short.iter_mut().zip(&mut wide)).map(|(g, (s, w))| match compact_dtype(g.key).unwrap() {
        ValueDtype::I16 => ObservationChunk::I16(s),
        ValueDtype::I32 => ObservationChunk::I32(w),
    }).collect();
    let mut narrow = dense.groups.iter().flat_map(|g| std::iter::once(g.known.len()).chain(g.occupied.iter().map(Vec::len)))
        .chain([dense.action_mask.len()]).map(|n| vec![MaybeUninit::new(42_i8); n]).collect::<Vec<_>>();
    let initialized = AtomicBool::new(false);
    let mut target = ObservationTarget::typed(chunks, narrow.iter_mut().map(Vec::as_mut_slice).collect(), &initialized);
    let encoded = crate::combat_policy::encode_compact_standard_combat(&d, &caps, run.ascension(), run.room_selection_operation(), run.combat_identity(), Some(&mut target)).unwrap();
    assert!(encoded.groups.is_empty(), "fitting compact target must be used");
    assert!(initialized.load(Ordering::Acquire));
    assert_eq!(target.materialize(&dense).unwrap().unwrap(), dense);
    let mut wrong_shape = SentinelTarget::new(&dense);
    wrong_shape.short[0].pop();
    let fallback = crate::combat_policy::encode_compact_standard_combat(&d, &caps, run.ascension(), run.room_selection_operation(), run.combat_identity(), Some(&mut wrong_shape.borrow(&dense))).unwrap();
    assert!(fallback == dense);
    assert!(!wrong_shape.initialized.load(Ordering::Acquire));

    let mut wrong_type = dense.groups.iter().map(|g| vec![MaybeUninit::new(123_i64); g.values.len()]).collect::<Vec<_>>();
    let untreated = AtomicBool::new(false);
    let mut target = ObservationTarget::new(wrong_type.iter_mut().map(Vec::as_mut_slice).collect(),
        narrow.iter_mut().map(Vec::as_mut_slice).collect(), &untreated);
    let fallback = crate::combat_policy::encode_compact_standard_combat(&d, &caps, run.ascension(), run.room_selection_operation(), run.combat_identity(), Some(&mut target)).unwrap();
    assert!(fallback == dense);
    assert!(!untreated.load(Ordering::Acquire));

    let mut run_only_counter = d;
    run_only_counter.combat_progress.entered = i32::MAX as u64 + 1;
    let compact = crate::combat_policy::encode_compact_standard_combat(&run_only_counter, &caps,
        run.ascension(), run.room_selection_operation(), run.combat_identity(), None).unwrap();
    assert!(compact == dense, "a projected-out Run counter does not affect compact combat");

}

pub(super) fn standard_catalog(entries: Vec<serde_json::Value>) -> CombatTrainingCatalog {
    let value = serde_json::json!({
        "catalog_schema_id": STANDARD_COMBAT_CATALOG_SCHEMA_ID,
        "profile_id": STANDARD_COMBAT_PROFILE_ID,
        "entries": entries,
    });
    CombatTrainingCatalog::standard_from_json(&serde_json::to_vec(&value).unwrap()).unwrap()
}

fn use_potion(observation: &RunObservation) -> RunActionIndex {
    let descriptor = standard_combat_observation_spec(&RunCapacities::standard_default()).unwrap().descriptor();
    let verb = descriptor["vocabularies"]["verb"].as_array().unwrap().iter().position(|v| v == "use_potion").unwrap();
    let actions = observation.group("actions").unwrap();
    let index = actions.values.chunks_exact(actions.shape[1]).enumerate()
        .position(|(i, row)| observation.action_mask[i] == 1 && row[0] == verb as i64).unwrap();
    RunActionIndex(index as i64)
}

#[test]
fn compact_combat_step_capacity_fault() {
    use serde_json::json;
    use spire_core::{combat::rng::Partition, content::Character};
    use super::m34s_transplant::standard_entry;

    let mut entry = standard_entry("test", Character::Silent, 20, "jaw_worm", None);
    entry["potions"] = json!(["attack_potion", null]);
    for preset in [StandardCombatCapacityPreset::Bounds, StandardCombatCapacityPreset::Play] {
        let mut config = StandardCombatSessionConfig::compact(preset);
        config.capacities.offers = 1;
        let mut session = CombatTrainingSession::new_standard_with_config(standard_catalog(vec![entry.clone()]), Partition::Test, config).unwrap();
        let initial = session.reset(Some(7), None).unwrap();
        assert!(matches!(session.step(initial.token, use_potion(&initial.observation)), Err(RunSessionError::DeliveryFault("post-step combat encoding failed"))));
        assert_eq!(session.fault_cause(), Some(RunSessionError::Capacity { group: "combat_card_offers", required: 3, capacity: 1 }));
        assert_eq!(session.diagnostic()["accepted_steps"], 1);
        assert!(session.record().unwrap().is_none());
    }
}

pub(super) fn assert_widens(compact: &RunObservation, dense: &RunObservation) {
    fn prefix<T: Copy + Default + PartialEq>(compact: &[T], dense: &[T]) -> bool {
        dense.split_at_checked(compact.len()).is_some_and(|(a, b)| a == compact && b.iter().all(|v| *v == T::default()))
    }
    assert_eq!(compact.groups.len(), dense.groups.len());
    for (c, d) in compact.groups.iter().zip(&dense.groups) {
        assert_eq!(c.key, d.key);
        assert_eq!(c.shape.len(), d.shape.len(), "rank of {}", c.key);
        assert_eq!(c.shape.last(), d.shape.last(), "width of {}", c.key);
        assert!(prefix(&c.values, &d.values), "values of {}", c.key);
        assert!(prefix(&c.known, &d.known), "known flags of {}", c.key);
        assert!(match (&c.occupied, &d.occupied) {
            (None, None) => true,
            (Some(c), Some(d)) => prefix(c, d),
            _ => false,
        }, "occupancy of {}", c.key);
    }
    assert!(prefix(&compact.action_mask, &dense.action_mask), "action mask");
}

#[test]
fn compact_combat_value_fault_and_trailing_potion_slots() {
    use serde_json::json;
    use spire_core::{combat::rng::Partition, content::Character};
    use super::m34s_transplant::standard_entry;
    let mut gold = standard_entry("gold", Character::Ironclad, 20, "jaw_worm", None);
    gold["gold"] = json!(2147483648_u32);
    let mut belt = standard_entry("belt", Character::Ironclad, 20, "jaw_worm", None);
    belt["relics"] = json!(["potion_belt"]);
    belt["potions"] = json!(["block_potion", null, null, null]);
    let catalog = standard_catalog(vec![gold, belt]);
    for caps in [RunCapacities::standard_combat_compact_play(), RunCapacities::standard_default()] {
        let config = StandardCombatSessionConfig { layout: RunObservationLayout::Compact, capacities: caps, capacity_preset: Some(StandardCombatCapacityPreset::Play) };
        let mut s = CombatTrainingSession::new_standard_with_config(catalog.clone(), Partition::Test, config).unwrap();
        let value = RunSessionError::Capacity { group: "player.gold", required: 2147483648, capacity: 2147483647 };
        assert_eq!(s.reset(Some(13), Some("gold")).err(), Some(value.clone()));
        assert_eq!(s.fault_cause(), Some(value));
    }
    let mut config = StandardCombatSessionConfig::compact(StandardCombatCapacityPreset::Play);
    config.capacities.potions = 3;
    let mut s = CombatTrainingSession::new_standard_with_config(catalog, Partition::Test, config).unwrap();
    assert_eq!(s.reset(Some(13), Some("belt")).err(), Some(RunSessionError::Capacity { group: "potions", required: 4, capacity: 3 }));
}

#[test]
#[ignore = "requires sealed M39b fixture streams staged via SPIRE_M39B_WIDE_FIXTURES"]
fn m39b_compact_combat_wide_lockstep() {
    use serde_json::Value;
    use spire_core::combat::rng::Partition;
    use std::{collections::BTreeMap, path::PathBuf};

    fn maxima(observation: &RunObservation, seen: &mut BTreeMap<&'static str, usize>) {
        let last = |v: &[i8]| v.iter().rposition(|x| *x != 0).map_or(0, |i| i + 1);
        for g in &observation.groups {
            let count = g.occupied.as_deref().map_or(1, last);
            let value = seen.entry(g.key).or_insert(0);
            *value = (*value).max(count);
        }
        let mask_width = seen.entry("action_mask").or_insert(0);
        *mask_width = (*mask_width).max(last(&observation.action_mask));
    }

    fn assert_boundary(dense: &CombatPolicyBoundary, compact: &CombatPolicyBoundary) {
        assert_widens(&compact.observation, &dense.observation);
        let mut expected_info = dense.info.clone();
        expected_info["combat_view_id"] = STANDARD_COMBAT_COMPACT_VIEW_ID.into();
        assert_eq!(compact.info, expected_info);
    }

    let directory = PathBuf::from(std::env::var_os("SPIRE_M39B_WIDE_FIXTURES").expect("sealed fixture directory"));
    let mut checked = [0, 0];
    for (filename, n, accepted) in [("bounds.jsonl", 173, 14069), ("play.jsonl", 108, 3603)] {
        let content = std::fs::read_to_string(directory.join(filename)).unwrap();
        let mut rows = content.lines().map(|line| serde_json::from_str::<Value>(line).unwrap()).collect::<Vec<_>>();
        assert_eq!(rows.pop().unwrap()["descriptor"], standard_combat_observation_spec(&RunCapacities::standard_default()).unwrap().descriptor());
        assert_eq!(rows.len(), n);
        assert_eq!(rows.iter().map(|r| r["accepted_indices"].as_array().unwrap().len()).sum::<usize>(), accepted);
        for row in rows {
            let catalog = CombatTrainingCatalog::standard_from_json(&serde_json::to_vec(&row["catalog"]).unwrap()).unwrap();
            let seed = row["seed"].as_u64();
            let id = row["id"].as_str();
            let mut dense = CombatTrainingSession::new_standard(catalog.clone(), Partition::Test).unwrap();
            let mut bd = dense.reset(seed, id).unwrap();
            assert_eq!(bd.info["start_kind"], row["reset_kind"]);
            let mut twins = Vec::new();
            for preset in [StandardCombatCapacityPreset::Bounds, StandardCombatCapacityPreset::Play] {
                let mut s = CombatTrainingSession::new_standard_with_config(catalog.clone(), Partition::Test, StandardCombatSessionConfig::compact(preset)).unwrap();
                match (preset, row["lane"].as_str().unwrap()) {
                    (StandardCombatCapacityPreset::Play, "entry_bound" | "relic_union") => {
                        let (group, required, capacity) = if row["lane"] == "entry_bound" {
                            ("permanent_cards", row["deck"].as_u64().unwrap() as usize, 96)
                        } else {
                            ("relics", row["catalog"]["entries"][0]["relics"].as_array().unwrap().len(), 32)
                        };
                        assert_eq!(s.reset(seed, id).err(), Some(RunSessionError::Capacity { group, required, capacity }), "{filename} {id:?}");
                        checked[1] += 1;
                    }
                    _ => {
                        let bc = s.reset(seed, id).unwrap();
                        assert_boundary(&bd, &bc);
                        twins.push((s, bc));
                    }
                }
            }
            let mut seen = BTreeMap::new();
            maxima(&bd.observation, &mut seen);
            for idx in row["accepted_indices"].as_array().unwrap() {
                let action = RunActionIndex(idx.as_i64().unwrap());
                let dense_step = dense.step(bd.token, action).unwrap();
                for (s, bc) in &mut twins {
                    assert_eq!(bc.observation.action_mask[action.0 as usize], 1);
                    let step = s.step(bc.token, action).unwrap();
                    assert_eq!((step.reward, step.terminated, step.truncated), (dense_step.reward, dense_step.terminated, dense_step.truncated));
                    assert_boundary(&dense_step.boundary, &step.boundary);
                    *bc = step.boundary;
                }
                bd = dense_step.boundary;
                maxima(&bd.observation, &mut seen);
            }
            assert_eq!(serde_json::to_value(&seen).unwrap(), row["max"], "{filename} {id:?}");
            if let Some(index) = row["failed_index"].as_i64() {
                let idx = RunActionIndex(index);
                let error = dense.step(bd.token, idx).unwrap_err();
                assert!(matches!(error, RunSessionError::Core(_)));
                assert_eq!(format!("step: {error:?}"), row["status"].as_str().unwrap());
                for (s, bc) in &mut twins {
                    assert_eq!(s.step(bc.token, idx).unwrap_err(), error);
                }
            } else {
                match row["status"].as_str().unwrap() {
                    "cap" => assert!(row["turn"] == 100 || row["step_count"] == 1200),
                    "selection_probe" => assert!(seen["selection"] > 0),
                    terminal => assert_eq!(bd.info["terminal_reason"], terminal),
                }
            }
            checked[0] += 1;
        }
    }
    assert_eq!(checked, [281, 20]);
}

#[test]
fn compact_combat_batch_first_reset_policy() {
    use serde_json::json;
    use spire_core::{combat::rng::Partition, content::Character};
    use std::sync::Arc;
    use super::m34s_transplant::standard_entry;
    let good = standard_entry("good", Character::Ironclad, 20, "jaw_worm", None);
    let mut expensive = good.clone();
    expensive["id"] = json!("gold");
    expensive["gold"] = json!(2147483648_u32);
    let mut large = good.clone();
    large["id"] = json!("large");
    large["deck"] = json!(vec![json!({"card":"strike","upgrades":0}); 97]);
    let catalog = Arc::new(standard_catalog(vec![good, expensive, large]));
    let gold = RunSessionError::Capacity { group: "player.gold", required: 2147483648, capacity: 2147483647 };
    let deck = RunSessionError::Capacity { group: "permanent_cards", required: 97, capacity: 96 };
    let widths = StandardCombatSessionConfig::compact(StandardCombatCapacityPreset::Play);
    for policy in [FaultPolicy::Isolate, FaultPolicy::Freeze] {
        for bad in [false, true] {
            let mut batch = CombatTrainingBatch::new_standard_with_config(catalog.clone(), Partition::Test, 2, None,
                BatchConfig { workers: 2, autoreset: AutoresetMode::Disabled, fault_policy: policy }, widths.clone()).unwrap();
            let second = if bad { "large" } else { "good" };
            let result = batch.reset(&[Some(4), Some(5)], &[Some("gold".into()), Some(second.into())], None);
            assert_eq!(batch.session(0).unwrap().fault_cause(), Some(gold.clone()));
            assert_eq!(batch.session(1).unwrap().fault_cause(), bad.then(|| deck.clone()));
            if policy == FaultPolicy::Freeze {
                assert!(matches!(result, Err(SessionBatchError::Fault(_))), "reset must freeze");
                if !bad { assert!(batch.committed_result(1).is_ok()); }
                continue
            }
            let rows = result.unwrap_or_else(|err| panic!("compact isolate reset should publish rows: {err:?}"));
            assert_eq!(rows.len(), 2);
            for row in rows.iter().take(if bad { 2 } else { 1 }) {
                assert!(row.boundary.observation.action_mask.iter().all(|v| *v == 0));
                assert_eq!(row.boundary.token, [0_u64, 0].into());
                assert!(row.boundary.observation.groups.iter().all(|g| g.values.iter().all(|v| *v == 0)
                    && g.known.iter().all(|v| *v == 0) && g.occupied.iter().flatten().all(|v| *v == 0)));
                let report = row.fault.as_ref().unwrap();
                assert_eq!(report.stage, BatchFaultStage::Reset);
                assert!(!report.committed);
            }
            assert!(batch.committed_result(0).is_err());
            let rows = batch.reset(&[Some(6), None], &[Some("good".into()), None], Some(&[true, false])).unwrap();
            assert!(rows[0].fault.is_none());
            assert_eq!(batch.session(0).unwrap().fault_cause(), None);
            let retired = batch.take_fault_records();
            assert_eq!(retired.len(), 1);
            assert_eq!(retired[0].error, Some(gold.clone()));
            assert!(retired[0].record.is_none());
            assert!(batch.take_completed_records().is_empty());
        }
    }
}

struct SentinelTarget {
    short: Vec<Vec<std::mem::MaybeUninit<i16>>>,
    wide: Vec<Vec<std::mem::MaybeUninit<i32>>>,
    flags: Vec<Vec<std::mem::MaybeUninit<i8>>>,
    initialized: std::sync::atomic::AtomicBool,
}

impl SentinelTarget {
    fn new(template: &RunObservation) -> Self {
        use std::mem::MaybeUninit;
        Self {
            short: template.groups.iter().map(|g| vec![MaybeUninit::new(123); g.values.len()]).collect(),
            wide: template.groups.iter().map(|g| vec![MaybeUninit::new(123); g.values.len()]).collect(),
            flags: template.groups.iter().flat_map(|g| std::iter::once(g.known.len()).chain(g.occupied.iter().map(Vec::len)))
                .chain([template.action_mask.len()]).map(|n| vec![MaybeUninit::new(42); n]).collect(),
            initialized: std::sync::atomic::AtomicBool::new(false),
        }
    }

    fn borrow<'a>(&'a mut self, template: &RunObservation) -> ObservationTarget<'a> {
        use crate::run_schema_standard::{compact_dtype, ValueDtype};
        let chunks = template.groups.iter().zip(self.short.iter_mut().zip(&mut self.wide))
            .map(|(g, (s, w))| match compact_dtype(g.key).unwrap() {
                ValueDtype::I16 => ObservationChunk::I16(s),
                ValueDtype::I32 => ObservationChunk::I32(w),
            }).collect();
        ObservationTarget::typed(chunks, self.flags.iter_mut().map(Vec::as_mut_slice).collect(), &self.initialized)
    }
}

#[test]
fn compact_combat_batch_target_fault_retirement_and_recovery() {
    use serde_json::json;
    use spire_core::{combat::rng::Partition, content::Character};
    use std::sync::Arc;
    use super::m34s_transplant::standard_entry;
    let bad = standard_entry("bad", Character::Ironclad, 20, "jaw_worm", None);
    let mut good = bad.clone();
    good["id"] = json!("good");
    good["gold"] = json!(98);
    let mut expensive = good.clone();
    expensive["id"] = json!("expensive");
    expensive["gold"] = json!(2147483648_u32);
    let catalog = Arc::new(standard_catalog(vec![bad, good, expensive]));
    let mut widths = StandardCombatSessionConfig::compact(StandardCombatCapacityPreset::Play);
    widths.capacities.powers = 63;
    let mut batch = CombatTrainingBatch::new_standard_with_config(catalog.clone(), Partition::Test, 2, None,
        BatchConfig { workers: 2, autoreset: AutoresetMode::Disabled, fault_policy: FaultPolicy::Isolate }, widths.clone()).unwrap();
    let start = batch.reset(&[Some(4), Some(5)], &[Some("bad".into()), Some("good".into())], None).unwrap();
    let template = batch.zero_boundary().unwrap().observation.clone();
    let mut buffers = [SentinelTarget::new(&template), SentinelTarget::new(&template)];
    let targets = buffers.iter_mut().map(|b| b.borrow(&template)).collect::<Vec<_>>();
    let tokens = [start[0].boundary.token, start[1].boundary.token];
    let mut peer = CombatTrainingSession::new_standard_with_config((*catalog).clone(), Partition::Test, widths.clone()).unwrap();
    let start_peer = peer.reset(Some(5), Some("good")).unwrap();
    let expected = peer.step(start_peer.token, RunActionIndex(0)).unwrap();
    let hook = crate::run_observation::test_range::inflate(widths.capacities, "player", 2, i64::from(i32::MAX), (99, 99));
    let result = batch.step_into(&tokens, &[RunActionIndex(0); 2], targets).unwrap();
    drop(hook);
    let committed_fault = RunSessionError::Capacity { group: "player.gold", required: 2147483746, capacity: 2147483647 };
    assert_eq!(batch.session(0).unwrap().fault_cause(), Some(committed_fault.clone()));
    assert_eq!(result[0].fault.as_ref().unwrap().stage, BatchFaultStage::Encode);
    assert!(result[0].fault.as_ref().unwrap().committed);
    assert!(result[0].boundary.observation == template);
    assert!(result[1].boundary.observation.groups.is_empty());
    assert!(buffers[1].borrow(&template).materialize(&template).unwrap().unwrap() == expected.boundary.observation);
    assert!(batch.committed_result(1).unwrap().boundary.observation == expected.boundary.observation);
    assert!(batch.committed_result(0).is_err());
    let replacement = batch.reset(&[Some(7), None], &[Some("expensive".into()), None], Some(&[true, false])).unwrap();
    assert!(replacement[0].fault.is_some());
    let retired = batch.take_fault_records();
    assert_eq!(retired.len(), 1);
    assert_eq!(retired[0].error, Some(committed_fault));
    let new_fault = RunSessionError::Capacity { group: "player.gold", required: 2147483648, capacity: 2147483647 };
    assert_eq!(batch.session(0).unwrap().fault_cause(), Some(new_fault.clone()));
    let resumed = batch.reset(&[Some(8), None], &[Some("good".into()), None], Some(&[true, false])).unwrap();
    assert!(resumed[0].fault.is_none());
    assert_eq!(batch.session(0).unwrap().fault_cause(), None);
    assert_eq!(batch.take_fault_records()[0].error, Some(new_fault));
    assert!(batch.take_fault_records().is_empty());
}

pub(super) fn generated_position(character: &str, card: &str, copies: usize, generated: &str, count: usize, pile: &str) -> serde_json::Value {
    use serde_json::json;
    let deck = vec![json!({"card":card,"upgrades":1}); copies];
    let mut piles = json!({"hand":deck, "draw":[], "draw_top":[], "discard":[], "exhaust":[]});
    piles[pile] = json!(vec![json!({"card":generated,"upgrades":0,"misc":0,"generated":true});count]);
    json!({"id":"probe","character":character,"ascension":20,"encounter":"three_darklings",
        "hp":999,"max_hp":999,"gold":0,"deck":deck,"relics":[],"potions":[null,null],
        "position":{"player":{"energy":9},"piles":piles}})
}

#[test]
fn compact_play_generated_selection_capacity() {
    use spire_core::combat::rng::Partition;
    for count in [192, 193] {
        let catalog = standard_catalog(vec![generated_position("defect","seek",1,"dazed",count,"draw")]);
        let mut dense = CombatTrainingSession::new_standard(catalog.clone(), Partition::Test).unwrap();
        let initial = dense.reset(Some(13), None).unwrap();
        let action = RunActionIndex(initial.observation.action_mask.iter().position(|m| *m == 1).unwrap() as i64);
        let wanted = dense.step(initial.token, action).unwrap().boundary;
        assert_eq!(wanted.observation.action_mask.iter().filter(|m| **m == 1).count(), count);
        for (action_width, selection_width) in [(192, 192), (193, 192), (193, 193)] {
            let mut config = StandardCombatSessionConfig::compact(StandardCombatCapacityPreset::Play);
            config.capacities.actions = action_width;
            config.capacities.selection = selection_width;
            let mut compact = CombatTrainingSession::new_standard_with_config(catalog.clone(), Partition::Test, config).unwrap();
            let initial = compact.reset(Some(13), None).unwrap();
            let result = compact.step(initial.token, action);
            match (count > action_width, count > selection_width) {
                (false, false) => assert_widens(&result.unwrap().boundary.observation, &wanted.observation),
                (too_many_actions, _) => {
                    assert!(matches!(result, Err(RunSessionError::DeliveryFault(_))));
                    let (group, capacity) = if too_many_actions { ("actions", action_width) } else { ("selection", selection_width) };
                    assert_eq!(compact.fault_cause(), Some(RunSessionError::Capacity { group, capacity, required: count }));
                }
            }
        }
    }
}

#[test]
fn compact_play_power_writer_capacity() {
    use spire_core::{combat::{FullPowerId, FullPowerView}, content::Character};
    use super::m34s_transplant::{standard_entry, standard_combat_session};
    let mut session = standard_combat_session(vec![standard_entry("test", Character::Ironclad, 20, "three_darklings", None)]);
    session.reset(Some(13), None).unwrap();
    let run = session.run_for_test().unwrap();
    let mut decision = run.decision().unwrap();
    let edit = |decision: &mut spire_core::run::PublicDecision, rows| {
        let combat = decision.combat.as_mut().unwrap();
        let ids = spire_core::combat::public_ids::PUBLIC_POWERS;
        let powers = |n| ids.iter().take(n).map(|id| FullPowerView { id: FullPowerId(id), amount: 1 }).collect();
        combat.player.powers = powers(32);
        for enemy in &mut combat.enemies { enemy.powers.clear(); }
        combat.enemies[0].powers = powers(rows - 32);
    };
    for rows in [64, 65] {
        edit(&mut decision, rows);
        let mut caps = RunCapacities::standard_combat_compact_play();
        let encode = |caps: &RunCapacities| crate::combat_policy::encode_compact_standard_combat(
            &decision,caps,run.ascension(),run.room_selection_operation(),run.combat_identity(),None);
        if rows == 65 {
            assert_eq!(encode(&caps).err(), Some(RunSessionError::Capacity { group:"powers",required:65,capacity:64 }));
        }
        caps.powers = rows;
        let expected = crate::combat_policy::encode_standard_combat(&decision,&RunCapacities::standard_default(),run.ascension(),run.room_selection_operation(),run.combat_identity()).unwrap();
        assert_widens(&encode(&caps).unwrap(), &expected);
    }
}

#[test]
fn compact_play_late_inventory_capacity() {
    use serde_json::json;
    use spire_core::{combat::rng::Partition, content::{Character, RelicId, complete}};
    use super::m34s_transplant::standard_entry;
    for count in [32, 33] {
        let mut entry = standard_entry("test", Character::Ironclad, 20, "jaw_worm", None);
        let relics = complete::relic_union(Character::Ironclad).into_iter().filter(|id| !matches!(id,
            RelicId::NeowsLament | RelicId::BlackBlood | RelicId::BottledFlame | RelicId::BottledLightning
            | RelicId::BottledTornado | RelicId::PotionBelt | RelicId::MarkOfPain | RelicId::GamblingChip))
            .take(count).map(RelicId::as_str).collect::<Vec<_>>();
        entry["relics"] = json!(relics);
        let catalog = standard_catalog(vec![entry]);
        let mut config = StandardCombatSessionConfig::compact(StandardCombatCapacityPreset::Play);
        if count == 33 {
            let mut narrow = CombatTrainingSession::new_standard_with_config(catalog.clone(), Partition::Test, config.clone()).unwrap();
            assert_eq!(narrow.reset(Some(13), None).err(), Some(RunSessionError::Capacity { group:"relics",required:33,capacity:32 }));
        }
        config.capacities.relics = count;
        let mut compact = CombatTrainingSession::new_standard_with_config(catalog.clone(), Partition::Test, config).unwrap();
        let mut dense = CombatTrainingSession::new_standard(catalog, Partition::Test).unwrap();
        assert_widens(&compact.reset(Some(13), None).unwrap().observation, &dense.reset(Some(13), None).unwrap().observation);
    }
}

#[test]
fn compact_combat_same_step_final_survives_target_reuse() {
    use serde_json::json;
    use spire_core::{combat::rng::Partition, content::Character};
    use std::sync::Arc;
    use super::m34s_transplant::standard_entry;
    let mut entry = standard_entry("exit", Character::Ironclad, 20, "jaw_worm", None);
    entry["potions"] = json!(["smoke_bomb", null]);
    let catalog = Arc::new(standard_catalog(vec![entry]));
    let widths = StandardCombatSessionConfig::compact(StandardCombatCapacityPreset::Bounds);
    for (policy, workers) in [(FaultPolicy::Isolate, 1), (FaultPolicy::Freeze, 16)] {
        for reset_fault in [false, true] {
            let mut batch = CombatTrainingBatch::new_standard_with_config(catalog.clone(), Partition::Test, 16, None,
                BatchConfig { workers, autoreset: AutoresetMode::SameStep, fault_policy: policy }, widths.clone()).unwrap();
            let rows = batch.reset(&[Some(27); 16], &[const { None }; 16], None).unwrap();
            let template = batch.zero_boundary().unwrap().observation.clone();
            let mut targets = (0..16).map(|_| SentinelTarget::new(&template)).collect::<Vec<_>>();
            let mut scalar = CombatTrainingSession::new_standard_with_config((*catalog).clone(), Partition::Test,widths.clone()).unwrap();
            let prior = scalar.reset(Some(27), None).unwrap();
            let action = use_potion(&prior.observation);
            let expected_final = scalar.step(prior.token, action).unwrap().boundary.observation;
            let expected_reset = scalar.reset(None, None).unwrap().observation;
            let tokens = rows.iter().map(|r| r.boundary.token).collect::<Vec<_>>();
            if reset_fault { batch.inject(0, crate::session_batch::TestInject::ResetError); }
            let result = batch.step_into(&tokens, &[action; 16], targets.iter_mut().map(|x| x.borrow(&template)).collect());
            let freeze = reset_fault && policy == FaultPolicy::Freeze;
            if freeze {
                assert!(matches!(result, Err(SessionBatchError::Fault(_))));
            } else {
                let rows = result.unwrap();
                for row in &rows {
                    assert!(row.terminated);
                    assert!(row.final_boundary.as_ref().unwrap().observation == expected_final);
                }
                if reset_fault {
                    assert_eq!(rows[0].fault.as_ref().unwrap().stage, BatchFaultStage::Autoreset);
                    assert!(rows[0].fault.as_ref().unwrap().committed);
                } else {
                    assert!(rows[0].boundary.observation.groups.is_empty());
                    assert!(targets[0].borrow(&template).materialize(&template).unwrap().unwrap() == expected_reset);
                }
                batch.fail_export();
            }
            let recovered = batch.recover_committed().unwrap();
            assert!(recovered[0].as_ref().unwrap().final_boundary.as_ref().unwrap().observation == expected_final);
            for row in recovered.iter().skip(1) {
                let row = row.as_ref().unwrap();
                assert!(row.boundary.observation == expected_reset);
                assert!(row.final_boundary.as_ref().unwrap().observation == expected_final);
            }
            assert!(batch.take_completed_records().is_empty());
        }
    }
}

pub(super) fn compact_search_limits() -> SearchLimits {
    SearchLimits { max_live_branches: 8, max_branches: 16, max_steps_per_branch: 64, max_total_steps: 256 }
}

#[test]
fn compact_search_first_view_rejected_atomically() {
    use serde_json::json;
    use spire_core::{combat::rng::Partition, content::Character};
    use super::m34s_transplant::standard_entry;
    let search_config = SearchConfig { research_seed: 17, limits: compact_search_limits() };
    let entry = standard_entry("early", Character::Ironclad, 20, "jaw_worm", None);
    for (group, mut entry) in [("permanent_cards", entry.clone()), ("player.gold", entry)] {
        let mut layout = StandardCombatSessionConfig::compact(StandardCombatCapacityPreset::Play);
        let (required, capacity) = if group == "player.gold" {
            entry["gold"] = json!(2147483648_u32);
            (2147483648, 2147483647)
        } else {
            layout.capacities.permanent_cards = 1;
            (entry["deck"].as_array().unwrap().len(), 1)
        };
        let mut dense = CombatTrainingSession::new_standard(standard_catalog(vec![entry]), Partition::Test).unwrap();
        dense.reset(Some(13), None).unwrap();
        let mut search = PublicSearch::new_with_combat_layout(dense.public_root().unwrap(), search_config, layout).unwrap();
        let before = search.counters();
        assert_eq!(search.branch(None), Err(RunSessionError::Capacity { group, required, capacity }));
        assert_eq!(search.counters(), before);
        assert_eq!(search.used_samples_for_test(), 0);
    }

    let mut dense = CombatTrainingSession::new_standard(standard_catalog(vec![generated_position("defect","seek",1,"dazed",989,"draw")]), Partition::Test).unwrap();
    let initial = dense.reset(Some(13), None).unwrap();
    dense.step(initial.token, RunActionIndex(0)).unwrap();
    let root = dense.public_root().unwrap();
    let mut config = StandardCombatSessionConfig::compact(StandardCombatCapacityPreset::Play);
    config.capacities.combat_cards = 1024;
    config.capacities.selection = 989;
    let mut search = PublicSearch::new_with_combat_layout(root.clone(), search_config, config.clone()).unwrap();
    assert_eq!(search.branch(None), Err(RunSessionError::Capacity { group: "actions", required: 989, capacity: 192 }));
    assert_eq!(search.used_samples_for_test(), 0);
    let mut wide = config;
    wide.capacities.actions = 989;
    let mut search = PublicSearch::new_with_combat_layout(root, search_config, wide).unwrap();
    let b = search.branch(None).unwrap();
    assert_eq!(search.decision(b).unwrap().observation.action_mask, vec![1; 989]);
}

#[test]
fn compact_search_current_menu_width_preflight_and_successor_overflow() {
    use spire_core::combat::rng::Partition;
    let mut dense = CombatTrainingSession::new_standard(standard_catalog(vec![generated_position("defect","seek",1,"dazed",193,"draw")]), Partition::Test).unwrap();
    dense.reset(Some(13), None).unwrap();
    let root = dense.public_root().unwrap();
    let cfg = SearchConfig { research_seed: 17, limits: compact_search_limits() };
    for view in [BranchView::Mask, BranchView::Full] {
        let mut s = PublicSearch::new_with_combat_layout(root.clone(), cfg, StandardCombatSessionConfig::compact(StandardCombatCapacityPreset::Play)).unwrap().with_workers(16).unwrap();
        let a = s.branch(Some(0)).unwrap();
        let b = s.branch(Some(1)).unwrap();
        let x = s.decision(a).unwrap();
        let y = s.decision(b).unwrap();
        let over = RunSessionError::Capacity { group: "actions", required: 193, capacity: 192 };
        assert!(matches!(s.step(a, x.token, RunActionIndex(192), view), Err(RunSessionError::Input(_))));
        s.widen_legal_table_for_test(a, 193);
        let before = s.counters();
        for i in [0, 192] {
            assert_eq!(s.step(a, x.token, RunActionIndex(i), view).err(), Some(over.clone()));
            let req = [BranchStepRequest { branch:b, token:y.token, index:RunActionIndex(1) },
                       BranchStepRequest { branch:a, token:x.token, index:RunActionIndex(i) }];
            assert_eq!(s.step_many(&req, view).err(), Some(over.clone()));
        }
        assert_eq!(before, s.counters());
        assert!(s.pool_for_test().is_none());
        assert!(s.actions(a).unwrap().is_empty());
        assert!(s.actions(b).unwrap().is_empty());
        assert_eq!(s.decision(b).unwrap(), y);
        let c = s.branch(Some(2)).unwrap();
        let z = s.decision(c).unwrap();
        let req = [BranchStepRequest { branch:b, token:y.token, index:RunActionIndex(1) },
                   BranchStepRequest { branch:c, token:z.token, index:RunActionIndex(0) }];
        let outcomes = s.step_many(&req, view).unwrap();
        assert!(matches!(outcomes[0], BranchStepOutcome::Stepped(_)));
        assert_eq!(outcomes[1], BranchStepOutcome::Fault { error:over.clone(), stage:"encode", committed:true });
        assert_eq!(s.actions(b).unwrap(), vec![1]);
        assert_eq!(s.actions(c).unwrap(), vec![0]);
        assert_eq!(s.counters()["total_steps"], 2);
        assert_eq!(s.step(c, z.token, RunActionIndex(0), view).err(), Some(RunSessionError::BranchFaulted));
        let before = s.counters();
        assert_eq!(s.decision(c).err(), Some(over));
        assert_eq!(s.counters(), before);
    }
}

#[test]
fn compact_search_mask_continues_from_published_legal_menu() {
    use serde_json::json;
    use spire_core::{combat::rng::Partition, content::Character};
    use super::m34s_transplant::standard_entry;
    let mut entry = standard_entry("offers",Character::Ironclad,20,"jaw_worm",None);
    entry["potions"] = json!(["attack_potion",null]);
    let mut dense = CombatTrainingSession::new_standard(standard_catalog(vec![entry]),Partition::Test).unwrap();
    dense.reset(Some(13),None).unwrap();
    let root = dense.public_root().unwrap();
    let cfg = SearchConfig { research_seed: 17, limits: compact_search_limits() };
    let mut wide = StandardCombatSessionConfig::compact(StandardCombatCapacityPreset::Play);
    wide.capacities.offers = 1;
    let mut s = PublicSearch::new_with_combat_layout(root,cfg,wide).unwrap();
    let a = s.branch(Some(0)).unwrap();
    let b = s.branch(Some(0)).unwrap();
    let x = s.decision(a).unwrap();
    let y = s.decision(b).unwrap();
    let index = use_potion(&x.observation);
    let over = RunSessionError::Capacity { group:"combat_card_offers",required:3,capacity:1 };
    assert_eq!(s.step(a,x.token,index,BranchView::Full).err(),Some(over.clone()));
    assert_eq!(s.actions(a).unwrap(), vec![index.0]);
    assert!(s.fork(a).is_err());
    let mask = s.step(b,y.token,index,BranchView::Mask).unwrap();
    let before = s.counters();
    assert_eq!(s.decision(a).err(),Some(over.clone()));
    assert_eq!(s.decision(b).err(),Some(over));
    assert_eq!(s.counters(),before);
    let fork=s.fork(b).unwrap();
    let choose = RunActionIndex(mask.action_mask.iter().position(|x|*x==1).unwrap() as i64);
    let continued = s.step(b,mask.token,choose,BranchView::Full).unwrap();
    let twin = s.step(fork,[fork.0,0].into(),choose,BranchView::Full).unwrap();
    assert!(continued.observation == twin.observation);
    assert_eq!(s.actions(b).unwrap(),s.actions(fork).unwrap());
}

#[test]
fn compact_search_full_widens() {
    use spire_core::{combat::rng::Partition, content::Character};
    use super::m34s_transplant::standard_entry;
    let catalog = standard_catalog(vec![standard_entry("twin", Character::Silent, 20, "three_darklings", None)]);
    let layout = StandardCombatSessionConfig::compact(StandardCombatCapacityPreset::Play);
    let mut dense = CombatTrainingSession::new_standard(catalog.clone(), Partition::Test).unwrap();
    let mut compact = CombatTrainingSession::new_standard_with_config(catalog, Partition::Test, layout.clone()).unwrap();
    dense.reset(Some(13), None).unwrap();
    compact.reset(Some(13), None).unwrap();
    let cfg = SearchConfig { research_seed: 17, limits: compact_search_limits() };
    let mut ordinary = PublicSearch::new(dense.public_root().unwrap(), cfg).unwrap();
    let mut small = PublicSearch::new_with_combat_layout(compact.public_root().unwrap(), cfg, layout).unwrap().with_workers(16).unwrap();
    assert_eq!(ordinary.spec()["root_digest"], small.spec()["root_digest"]);
    let a = ordinary.branch(Some(0)).unwrap();
    let b = small.branch(Some(0)).unwrap();
    let x = ordinary.decision(a).unwrap();
    let y = small.decision(b).unwrap();
    assert_widens(&y.observation, &x.observation);
    let index = RunActionIndex(x.observation.action_mask.iter().position(|m| *m == 1).unwrap() as i64);
    let u = ordinary.step(a, x.token, index, BranchView::Full).unwrap();
    let v = small.step(b, y.token, index, BranchView::Full).unwrap();
    assert_widens(&v.observation.unwrap(), &u.observation.unwrap());
    assert_eq!((v.reward, v.terminated, v.truncated), (u.reward, u.terminated, u.truncated));
    let child_root = small.public_root(b).unwrap();
    let mut child_compact = PublicSearch::new_with_combat_layout(child_root.clone(), cfg,
        StandardCombatSessionConfig::compact(StandardCombatCapacityPreset::Bounds)).unwrap();
    let mut child_dense = PublicSearch::new(child_root, cfg).unwrap();
    assert_eq!(child_compact.spec()["root_digest"], child_dense.spec()["root_digest"]);
    let left = child_compact.branch(None).unwrap();
    let right = child_dense.branch(None).unwrap();
    assert_widens(&child_compact.decision(left).unwrap().observation, &child_dense.decision(right).unwrap().observation);
}

fn sjh_catalog() -> CombatTrainingCatalog {
    use serde_json::json;
    let entry = json!({
        "id": "head-recycle", "character": "ironclad", "ascension": 0,
        "encounter": "giant_head", "hp": 80, "max_hp": 80, "gold": 0,
        "deck": (["finesse", "finesse", "strike", "transmutation"].map(|card| json!({"card": card, "upgrades": 0}))),
        "relics": ["sundial", "chemical_x"], "potions": [null, null, null]
    });
    let bytes = serde_json::to_vec(&json!({
        "catalog_schema_id": STANDARD_COMBAT_CATALOG_SCHEMA_ID,
        "profile_id": STANDARD_COMBAT_PROFILE_ID, "entries": [entry]
    })).unwrap();
    CombatTrainingCatalog::standard_from_json(&bytes).unwrap()
}

fn sjh_card_index(run: &spire_core::run::Run, id: spire_core::content::CardId) -> RunActionIndex {
    use spire_core::{combat::CombatAction, run::GameAction};
    let decision = run.decision().unwrap();
    let combat = decision.combat.unwrap();
    RunActionIndex(decision.candidates.iter().position(|candidate| {
        let GameAction::Combat(CombatAction::PlayCard { instance, .. }) = candidate.action else { return false };
        combat.hand.iter().any(|card| card.instance.0 == instance && card.id == id)
    }).unwrap() as i64)
}

#[test]
fn sjh_giant_head_long_turn() {
    use spire_core::{combat::rng::Partition, content::{CardId, Character}};
    let catalog = sjh_catalog();
    let mut dense = CombatTrainingSession::new_standard(catalog.clone(), Partition::Test).unwrap();
    let mut compact = CombatTrainingSession::new_standard_with_config(
        catalog, Partition::Test, StandardCombatSessionConfig::compact(StandardCombatCapacityPreset::Play),
    ).unwrap();
    dense.reset(Some(7), None).unwrap();
    compact.reset(Some(7), None).unwrap();
    let mut run = RunSession::from_standard_supplied(
        StandardRunConfig::new(Character::Ironclad, 0, Partition::Test),
        dense.run_for_test().unwrap().clone(), 7,
    ).unwrap();

    for step in 0..1_606 {
        let index = sjh_card_index(dense.run_for_test().unwrap(), CardId::Finesse);
        let dense_token = dense.decision().unwrap().token;
        let compact_token = compact.decision().unwrap().token;
        let run_token = run.decision().unwrap().token;
        dense.step(dense_token, index).unwrap_or_else(|error| panic!("play {step}: {error:?}"));
        compact.step(compact_token, index).unwrap();
        run.step(run_token, index).unwrap();
    }
    let actual = dense.run_for_test().unwrap().decision().unwrap().combat.unwrap();
    assert_eq!(actual, compact.run_for_test().unwrap().decision().unwrap().combat.unwrap());
    assert_eq!(actual, run.run_for_test().unwrap().decision().unwrap().combat.unwrap());
    assert_eq!(actual.turn, 1);
    assert_eq!(actual.enemies[0].hp, 500);
    assert_eq!(actual.enemies[0].powers.iter().find(|power| power.id.0 == "slow").unwrap().amount, 1_606);
    assert_eq!(actual.player.energy, 999);
    assert_eq!(actual.player.block, 999);
}

#[test]
fn sjh_bound_search_redaction_and_prefix() {
    use spire_core::{combat::{EngineBound, rng::Partition}, content::{CardId, Character}};
    let mut session = CombatTrainingSession::new_standard(sjh_catalog(), Partition::Test).unwrap();
    let mut boundary = session.reset(Some(7), None).unwrap();
    for _ in 0..1_510 {
        let index = sjh_card_index(session.run_for_test().unwrap(), CardId::Finesse);
        boundary = session.step(boundary.token, index).unwrap().boundary;
    }
    let limits = SearchLimits { max_live_branches: 3, max_branches: 3, max_steps_per_branch: 4, max_total_steps: 12 };
    let mut search = PublicSearch::new(session.public_root().unwrap(), SearchConfig { research_seed: 17, limits }).unwrap();
    let expected = EngineBound::new("generated_card_request", 999, 1_001);
    let first = search.branch(Some(0)).unwrap();
    let bad_index = sjh_card_index(search.branch_run_for_test(first), CardId::Transmutation);
    let token = search.decision(first).unwrap().token;
    assert_eq!(search.step(first, token, bad_index, BranchView::Full), Err(RunSessionError::EngineBound(expected)));
    assert!(search.actions(first).unwrap().is_empty());
    assert_eq!(search.decision(first).unwrap().info["end_reason"], "fault");

    let bad = search.branch(Some(1)).unwrap();
    let good = search.branch(Some(2)).unwrap();
    let bad_index = sjh_card_index(search.branch_run_for_test(bad), CardId::Transmutation);
    let good_index = sjh_card_index(search.branch_run_for_test(good), CardId::Finesse);
    let requests = [
        BranchStepRequest { branch: bad, token: search.decision(bad).unwrap().token, index: bad_index },
        BranchStepRequest { branch: good, token: search.decision(good).unwrap().token, index: good_index },
    ];
    let outcomes = search.step_many(&requests, BranchView::Mask).unwrap();
    assert!(matches!(&outcomes[0], BranchStepOutcome::Fault {
        error: RunSessionError::EngineBound(bound), stage: "step", committed: true,
    } if *bound == expected));
    assert!(matches!(&outcomes[1], BranchStepOutcome::Stepped(_)));
    assert!(search.actions(bad).unwrap().is_empty());
    assert_eq!(search.actions(good).unwrap(), [good_index.0]);

    let mut config = StandardRunConfig::new(Character::Ironclad, 0, Partition::Test);
    config.record_replay = true;
    let mut run = RunSession::from_standard_supplied(
        config, session.run_for_test().unwrap().clone(), 7,
    ).unwrap();
    let good_index = sjh_card_index(run.run_for_test().unwrap(), CardId::Finesse);
    run.step(run.decision().unwrap().token, good_index).unwrap();
    let bad_index = sjh_card_index(run.run_for_test().unwrap(), CardId::Transmutation);
    let error = run.step(run.decision().unwrap().token, bad_index).unwrap_err();
    assert_eq!(error.engine_bound(), Some(expected));
    assert_eq!(run.fault_cause(), Some(error));
    let record = run.record().unwrap().unwrap();
    assert_eq!(record.steps.len(), 1);
    let fault = record.fault.unwrap();
    assert!(fault.committed);
    assert!(!fault.gameplay_recorded);

    let bad_index = sjh_card_index(session.run_for_test().unwrap(), CardId::Transmutation);
    let error = session.step(boundary.token, bad_index).unwrap_err();
    assert_eq!(error.engine_bound(), Some(expected));
    assert_eq!(session.fault_cause(), Some(error));
}
