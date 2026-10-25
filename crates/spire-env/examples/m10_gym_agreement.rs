//! Public-start M10 trace exporter, not a strategy implementation or resolver.

#[path = "../../spire-core/examples/m10_support/mod.rs"]
mod support;

use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use spire_core::{
    combat::rng::Partition,
    content::Character,
    run::{PublicDecision, RunTerminal},
};
use spire_env::*;
use std::collections::BTreeMap;

const OBSERVATION_ENCODING_ID: &str = "spire-gym-run-observation-canonical-json-v1";
const SPEC_FINGERPRINT_DOMAIN: &[u8] = b"spire-gym/run-schema/v2\0";

fn hex_lower(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn sha256_hex(bytes: &[u8]) -> String {
    hex_lower(&Sha256::digest(bytes))
}

fn terminal_name(terminal: RunTerminal) -> &'static str {
    match terminal {
        RunTerminal::Death => "death",
        RunTerminal::EmptyDeck => "empty_deck",
        RunTerminal::Act3NoKeys => "act3_no_keys",
        RunTerminal::HeartVictory => "heart_victory",
    }
}

fn terminal_value(terminal: Option<RunTerminal>) -> Value {
    terminal.map_or(Value::Null, |value| json!(terminal_name(value)))
}

fn completion_name(completion: support::M10Completion) -> &'static str {
    match completion {
        support::M10Completion::Act3NoKeys => "act3_no_keys",
        support::M10Completion::Death => "death",
        support::M10Completion::EmptyDeck => "empty_deck",
        support::M10Completion::Act4Ready => "act4_ready",
        support::M10Completion::HeartVictory => "heart_victory",
        support::M10Completion::Unfinished => "unfinished",
    }
}

fn observation_payload(observation: &RunObservation) -> Value {
    let groups = observation
        .groups
        .iter()
        .map(|group| {
            json!({
                "key": group.key,
                "shape": group.shape,
                "values": group.values,
                "known": group.known,
                "occupied": group.occupied,
            })
        })
        .collect::<Vec<_>>();
    json!({ "groups": groups, "action_mask": observation.action_mask })
}

fn observation_sha256(observation: &RunObservation) -> Result<String, String> {
    let encoded = canonical_json(&observation_payload(observation))
        .map_err(|error| format!("canonical observation encoding: {error:?}"))?;
    Ok(sha256_hex(&encoded))
}

fn decision_summary(decision: &PublicDecision) -> Value {
    json!({
        "act": decision.act,
        "floor": decision.floor,
        "room_kind": decision.room.as_ref().map(|room| format!("{:?}", room.kind)),
        "candidate_count": decision.candidates.len(),
        "reward_total": decision.reward_total,
        "terminal": terminal_value(decision.terminal),
    })
}

#[derive(Default)]
struct GroupHighWater {
    capacity_key: String,
    capacity: usize,
    occupied_rows: usize,
    known_rows: usize,
    reported_rows: usize,
}

fn capacity_key(group: &str) -> Result<&'static str, String> {
    match group {
        "map_nodes" | "map_edges" | "permanent_cards" | "combat_cards" | "enemies"
        | "powers" | "relics" | "potions" | "offers" | "selection" | "orbs"
        | "actions" => Ok(match group {
            "map_nodes" => "map_nodes",
            "map_edges" => "map_edges",
            "permanent_cards" => "permanent_cards",
            "combat_cards" => "combat_cards",
            "enemies" => "enemies",
            "powers" => "powers",
            "relics" => "relics",
            "potions" => "potions",
            "offers" => "offers",
            "selection" => "selection",
            "orbs" => "orbs",
            "actions" => "actions",
            _ => unreachable!(),
        }),
        // M10 known_draw rows refer to the same combat-card table.
        "known_draw" => Ok("combat_cards"),
        _ => Err(format!("no capacity mapping for occupied group {group}")),
    }
}

fn update_high_water(
    observation: &RunObservation,
    spec: &RunObservationSpec,
    high_water: &mut BTreeMap<String, GroupHighWater>,
) -> Result<(), String> {
    let capacities = spec.descriptor()["capacities"].clone();
    for group in &observation.groups {
        let Some(occupied) = &group.occupied else {
            continue;
        };
        let columns = *group
            .shape
            .last()
            .ok_or_else(|| format!("{} has no feature columns", group.key))?;
        if columns == 0 || group.known.len() != occupied.len() * columns {
            return Err(format!("{} known/occupied shape mismatch", group.key));
        }
        let key = capacity_key(group.key)?;
        let capacity = usize::try_from(
            capacities[key]
                .as_u64()
                .ok_or_else(|| format!("missing capacity {key} for {}", group.key))?,
        )
        .map_err(|_| format!("capacity {key} does not fit usize"))?;
        let occupied_rows = occupied.iter().filter(|value| **value == 1).count();
        let known_rows = group
            .known
            .chunks_exact(columns)
            .filter(|row| row.iter().any(|value| *value == 1))
            .count();
        // Preserve the M9 exporter convention: an empty but known potion-belt
        // slot contributes to the reported belt high-water, not only a potion.
        let reported_rows = if group.key == "potions" {
            known_rows
        } else {
            occupied_rows
        };
        if occupied_rows > capacity || reported_rows > capacity {
            return Err(format!(
                "{} high-water {reported_rows} exceeds {key} capacity {capacity}",
                group.key
            ));
        }
        let entry = high_water
            .entry(group.key.to_owned())
            .or_insert_with(|| GroupHighWater {
                capacity_key: key.to_owned(),
                capacity,
                ..GroupHighWater::default()
            });
        if entry.capacity_key != key || entry.capacity != capacity {
            return Err(format!("{} capacity mapping changed", group.key));
        }
        entry.occupied_rows = entry.occupied_rows.max(occupied_rows);
        entry.known_rows = entry.known_rows.max(known_rows);
        entry.reported_rows = entry.reported_rows.max(reported_rows);
    }
    Ok(())
}

fn high_water_value(high_water: &BTreeMap<String, GroupHighWater>) -> Value {
    let mut groups = serde_json::Map::new();
    for (key, values) in high_water {
        groups.insert(
            key.clone(),
            json!({
                "capacity_key": values.capacity_key,
                "capacity": values.capacity,
                "max_occupied_rows": values.occupied_rows,
                "max_known_rows": values.known_rows,
                "m9_compatible_high_water": values.reported_rows,
            }),
        );
    }
    Value::Object(groups)
}

fn boundary_evidence(
    index: usize,
    session_observation: &RunObservation,
    core_observation: &RunObservation,
    decision: &PublicDecision,
    spec_fingerprint: &str,
    incoming: Option<(usize, i32, Option<RunTerminal>)>,
    source_trace_match: Option<bool>,
) -> Result<Value, String> {
    if session_observation != core_observation {
        return Err(format!("numeric session/core mismatch at boundary {index}"));
    }
    // Full owned-array equality above is the correctness check. Hash the common
    // equal value once; the two named fields document that equality for consumers.
    let observation_hash = observation_sha256(session_observation)?;
    let (action_index, reward, terminal) = incoming
        .map(|(action, reward, terminal)| {
            (
                json!(action),
                json!(reward),
                terminal_value(terminal),
            )
        })
        .unwrap_or((Value::Null, Value::Null, Value::Null));
    let incoming_terminated = incoming.map(|(_, _, terminal)| terminal.is_some());
    Ok(json!({
        "index": index,
        "observation_schema_id": RUN_OBSERVATION_M10_SCHEMA_ID,
        "spec_fingerprint": spec_fingerprint,
        "observation_encoding_id": OBSERVATION_ENCODING_ID,
        "observation_sha256": observation_hash,
        "core_observation_sha256": observation_hash,
        "session_matches_core": true,
        "core_decision_matches_source_trace": source_trace_match,
        "decision": decision_summary(decision),
        "terminal": terminal_value(decision.terminal),
        "incoming_action_index": action_index,
        "incoming_reward": reward,
        "incoming_terminal": terminal,
        "terminated": incoming_terminated,
        "truncated": incoming.map(|_| false),
        "offered_action_index": Value::Null,
        "offered_action": Value::Null,
    }))
}

fn execute() -> Result<Value, String> {
    let contexts = [
        (
            "awakened-unseen",
            M9BossHistory::AwakenedUnseen,
            support::M9BossHistory::AwakenedUnseen,
        ),
        (
            "time-eater-unseen",
            M9BossHistory::TimeEaterUnseen,
            support::M9BossHistory::TimeEaterUnseen,
        ),
        (
            "donu-deca-unseen",
            M9BossHistory::DonuDecaUnseen,
            support::M9BossHistory::DonuDecaUnseen,
        ),
    ];
    let mut cases = Vec::new();

    for seed in [7_u64, 11] {
        for (history_name, history, support_history) in contexts {
            // Reuse the frozen public-only caller. Do not develop/tune strategy here.
            let trajectory = support::rollout(seed, support_history, 8192)?;
            if trajectory.completion != support::M10Completion::Death {
                return Err(format!(
                    "preserved {seed}/{history_name} normal-start outcome changed: {}",
                    completion_name(trajectory.completion)
                ));
            }

            let capacities = RunCapacities::m10_default();
            let config = RunSessionConfig {
                character: Character::Ironclad,
                ascension: 0,
                partition: Partition::Test,
                boss_history: history,
                capacities: capacities.clone(),
                record_replay: true,
            };
            let mut core = support::start(seed, support_history)?;
            let mut session = RunSession::new_m10(config)
                .map_err(|error| format!("create M10 session: {error:?}"))?;
            let spec = session
                .observation_spec()
                .map_err(|error| format!("M10 observation spec: {error:?}"))?;
            let spec_descriptor = spec.descriptor();
            let spec_bytes = canonical_json(&spec_descriptor)
                .map_err(|error| format!("canonical M10 descriptor: {error:?}"))?;
            let spec_descriptor_sha256 = sha256_hex(&spec_bytes);
            let spec_fingerprint_bytes = spec.fingerprint();
            let mut fingerprint = Sha256::new();
            fingerprint.update(SPEC_FINGERPRINT_DOMAIN);
            fingerprint.update(&spec_bytes);
            let descriptor_fingerprint: [u8; 32] = fingerprint.finalize().into();
            if descriptor_fingerprint != spec_fingerprint_bytes {
                return Err(format!("M10 descriptor/fingerprint contract mismatch at {seed}/{history_name}"));
            }
            let spec_fingerprint = hex_lower(&spec_fingerprint_bytes);

            let mut boundary = session
                .reset(Some(seed))
                .map_err(|error| format!("M10 reset {seed}/{history_name}: {error:?}"))?;
            let initial_decision = core
                .decision()
                .map_err(|error| format!("initial core decision: {error:?}"))?;
            let initial_map = core.public_map();
            let mut core_observation = encode_run_m10(
                &initial_decision,
                initial_map.as_ref(),
                &capacities,
            )
            .map_err(|error| format!("initial M10 core encoding: {error:?}"))?;
            if boundary.terminal != initial_decision.terminal {
                return Err("initial session/core terminal mismatch".into());
            }
            let mut boundary_rows = vec![boundary_evidence(
                0,
                &boundary.observation,
                &core_observation,
                &initial_decision,
                &spec_fingerprint,
                None,
                None,
            )?];
            let mut high_water = BTreeMap::new();
            update_high_water(&boundary.observation, &spec, &mut high_water)?;
            let mut indices = Vec::with_capacity(trajectory.trace.len());

            for (step_index, expected) in trajectory.trace.iter().enumerate() {
                let before = core
                    .decision()
                    .map_err(|error| format!("core decision before step: {error:?}"))?;
                let action_index = before
                    .candidates
                    .iter()
                    .position(|candidate| candidate.action == expected.action)
                    .ok_or_else(|| {
                        format!(
                            "source action not offered at {seed}/{history_name}/{step_index}"
                        )
                    })?;
                if boundary.observation.action_mask.get(action_index) != Some(&1) {
                    return Err(format!(
                        "source action mask missing at {seed}/{history_name}/{step_index}"
                    ));
                }
                boundary_rows
                    .last_mut()
                    .ok_or("missing current boundary")?["offered_action_index"] =
                    json!(action_index);

                let result = session
                    .step(boundary.token, RunActionIndex(action_index as i64))
                    .map_err(|error| {
                        format!("M10 step {seed}/{history_name}/{step_index}: {error:?}")
                    })?;
                let transition = core
                    .step(&expected.action)
                    .map_err(|error| format!("core step {seed}/{history_name}/{step_index}: {error:?}"))?;
                let after = core
                    .decision()
                    .map_err(|error| format!("core decision after step: {error:?}"))?;
                let actual_map = core.public_map();
                if result.reward != transition.reward
                    || result.boundary.terminal != after.terminal
                    || result.terminated != after.terminal.is_some()
                    || result.truncated
                {
                    return Err(format!(
                        "session/core outcome mismatch at {seed}/{history_name}/{step_index}"
                    ));
                }
                if after != expected.decision || actual_map != expected.map {
                    return Err(format!(
                        "core/source trace boundary mismatch at {seed}/{history_name}/{step_index}"
                    ));
                }
                core_observation = encode_run_m10(
                    &after,
                    actual_map.as_ref(),
                    &capacities,
                )
                .map_err(|error| {
                    format!("M10 core encoding {seed}/{history_name}/{step_index}: {error:?}")
                })?;
                if result.boundary.observation != core_observation {
                    return Err(format!(
                        "session/core numeric mismatch at {seed}/{history_name}/{step_index}"
                    ));
                }
                let next_boundary = boundary_evidence(
                    step_index + 1,
                    &result.boundary.observation,
                    &core_observation,
                    &after,
                    &spec_fingerprint,
                    Some((action_index, result.reward, result.boundary.terminal)),
                    Some(true),
                )?;
                boundary_rows.push(next_boundary);
                update_high_water(&result.boundary.observation, &spec, &mut high_water)?;
                indices.push(action_index);
                boundary = result.boundary;
            }

            let final_decision = core
                .decision()
                .map_err(|error| format!("final core decision: {error:?}"))?;
            let final_map = core.public_map();
            if final_decision != trajectory.final_decision || final_map != trajectory.final_map {
                return Err(format!("final core/source mismatch at {seed}/{history_name}"));
            }
            if boundary.terminal != Some(RunTerminal::Death) {
                return Err(format!("actual Death missing at {seed}/{history_name}"));
            }

            let record = session
                .record()
                .map_err(|error| format!("export M10 record: {error:?}"))?
                .ok_or("M10 recording unexpectedly disabled")?;
            if record.steps.len() != trajectory.trace.len()
                || record.terminal != trajectory.final_decision.terminal
            {
                return Err(format!("M10 semantic record mismatch at {seed}/{history_name}"));
            }
            for (recorded, expected) in record.steps.iter().zip(&trajectory.trace) {
                if recorded.action != expected.action
                    || recorded.reward != expected.reward
                    || recorded.terminal != expected.decision.terminal
                {
                    return Err(format!("M10 semantic step mismatch at {seed}/{history_name}"));
                }
            }
            let record_bytes = record
                .to_json()
                .map_err(|error| format!("serialize M10 semantic record: {error:?}"))?;
            let record_value: Value = serde_json::from_slice(&record_bytes)
                .map_err(|error| format!("parse canonical M10 record: {error}"))?;
            let record_steps = record_value["steps"]
                .as_array()
                .ok_or("semantic record lacks steps")?;
            for (row, recorded_step) in boundary_rows
                .iter_mut()
                .take(record_steps.len())
                .zip(record_steps)
            {
                row["offered_action"] = recorded_step["action"].clone();
            }
            let record_sha256 = sha256_hex(&record_bytes);

            let decoded = RunEpisodeRecord::from_json(&record_bytes)
                .map_err(|error| format!("decode M10 semantic record: {error:?}"))?;
            if decoded != record {
                return Err(format!("M10 semantic round-trip mismatch at {seed}/{history_name}"));
            }
            let replay = replay_run(&decoded, capacities.clone())
                .map_err(|error| format!("replay M10 record: {error:?}"))?;
            let expected_rewards = trajectory
                .trace
                .iter()
                .map(|step| step.reward)
                .collect::<Vec<_>>();
            let expected_terminals = trajectory
                .trace
                .iter()
                .map(|step| step.decision.terminal)
                .collect::<Vec<_>>();
            if replay.accepted_count != trajectory.trace.len()
                || replay.final_decision != trajectory.final_decision
                || replay.final_map != trajectory.final_map
                || replay.rewards != expected_rewards
                || replay.terminals != expected_terminals
                || replay.reported_fault.is_some()
            {
                return Err(format!("M10 replay mismatch at {seed}/{history_name}"));
            }
            let replay_observation = encode_run_m10(
                &replay.final_decision,
                replay.final_map.as_ref(),
                &capacities,
            )
            .map_err(|error| format!("encode replay final boundary: {error:?}"))?;
            if replay_observation != boundary.observation {
                return Err(format!("M10 replay final observation mismatch at {seed}/{history_name}"));
            }
            let replay_observation_sha256 = observation_sha256(&replay_observation)?;

            cases.push(json!({
                "seed": seed,
                "history": history_name,
                "profile_id": "M10IroncladHeart",
                "completion": completion_name(trajectory.completion),
                "observation_schema_id": RUN_OBSERVATION_M10_SCHEMA_ID,
                "record_schema_id": RUN_RECORD_SCHEMA_ID_M10,
                "spec_fingerprint": spec_fingerprint,
                "spec_descriptor_sha256": spec_descriptor_sha256,
                "spec_fingerprint_domain": "spire-gym/run-schema/v2\\0",
                "capacities": spec_descriptor["capacities"],
                "indices": indices,
                "boundaries": boundary_rows,
                "high_water": high_water_value(&high_water),
                "record_sha256": record_sha256,
                "record": record_value,
                "replay": {
                    "accepted_count": replay.accepted_count,
                    "rewards": replay.rewards,
                    "terminals": replay.terminals.iter().map(|terminal| terminal_value(*terminal)).collect::<Vec<_>>(),
                    "final_terminal": terminal_value(replay.final_decision.terminal),
                    "final_observation_sha256": replay_observation_sha256,
                    "final_decision_matches_core": replay.final_decision == final_decision,
                    "final_map_matches_core": replay.final_map == final_map,
                    "reported_fault": replay.reported_fault.is_some(),
                },
                "session_run_observed_through": "death",
                "unobserved_sections": ["session-run Act 3 boss", "session-run Act 4 traversal", "session-run Heart combat"],
                "supplied_task23_chain_included": false,
            }));
        }
    }

    Ok(json!({
        "scope": "six public normal-start M10 episodes; supplied Task23 key/Act4 diagnostics are separate",
        "observation_encoding_id": OBSERVATION_ENCODING_ID,
        "cases": cases,
    }))
}

fn main() {
    match execute() {
        Ok(value) => println!("{value}"),
        Err(error) => {
            eprintln!("M10 native agreement: {error}");
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::execute;

    #[test]
    fn exports_six_m10_cases_with_evidence_at_every_real_boundary() {
        let evidence = execute().expect("six public-start M10 traces should export");
        assert_eq!(
            evidence["observation_encoding_id"],
            "spire-gym-run-observation-canonical-json-v1"
        );
        let cases = evidence["cases"].as_array().expect("case array");
        assert_eq!(cases.len(), 6, "seeds 7/11 × three frozen histories");

        for case in cases {
            let steps = case["record"]["steps"].as_array().expect("semantic steps");
            let boundaries = case["boundaries"].as_array().expect("boundary evidence");
            assert_eq!(boundaries.len(), steps.len() + 1, "reset plus every accepted step");
            assert_eq!(case["record"]["schema_id"], "spire-gym-run-replay-v2");
            assert_eq!(case["observation_schema_id"], "spire-gym-run-observation-v2");
            assert_eq!(case["completion"], "death", "retain the actual early ending");
            let recorded_fingerprint = case["record"]["observation_fingerprint"]
                .as_array()
                .expect("semantic record fingerprint")
                .iter()
                .map(|byte| u8::try_from(byte.as_u64().expect("fingerprint byte")).unwrap())
                .collect::<Vec<_>>();
            let recorded_fingerprint_hex = recorded_fingerprint
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>();
            assert_eq!(
                case["spec_fingerprint"], recorded_fingerprint_hex,
                "export the raw observation fingerprint, not a hash of its bytes"
            );
            assert_eq!(case["session_run_observed_through"], "death");
            assert_eq!(case["supplied_task23_chain_included"], false);
            assert_eq!(case["replay"]["reported_fault"], false);
            assert_eq!(case["replay"]["accepted_count"], steps.len());
            assert_eq!(case["replay"]["final_terminal"], "death");
            assert_eq!(
                case["replay"]["final_observation_sha256"],
                boundaries.last().expect("terminal boundary")["observation_sha256"]
            );
            assert!(case["spec_descriptor_sha256"].as_str().is_some());
            assert_eq!(
                case["high_water"]["known_draw"]["capacity_key"],
                "combat_cards"
            );
            assert_eq!(
                case["high_water"]["known_draw"]["capacity"],
                case["capacities"]["combat_cards"]
            );

            for (index, boundary) in boundaries.iter().enumerate() {
                assert_eq!(boundary["index"], index);
                assert_eq!(boundary["observation_schema_id"], "spire-gym-run-observation-v2");
                assert_eq!(boundary["spec_fingerprint"], case["spec_fingerprint"]);
                assert_eq!(boundary["session_matches_core"], true);
                assert_eq!(
                    boundary["observation_sha256"],
                    boundary["core_observation_sha256"]
                );
                assert_eq!(boundary["observation_sha256"].as_str().map(str::len), Some(64));
                if index == 0 {
                    assert_eq!(boundary["incoming_reward"], serde_json::Value::Null);
                    assert_eq!(boundary["incoming_terminal"], serde_json::Value::Null);
                    assert_eq!(boundary["terminated"], serde_json::Value::Null);
                    assert_eq!(boundary["truncated"], serde_json::Value::Null);
                }
                if index < steps.len() {
                    assert!(boundary["offered_action_index"].is_u64());
                    assert_eq!(boundary["offered_action_index"], case["indices"][index]);
                    assert_eq!(boundary["offered_action"], steps[index]["action"]);
                    assert_eq!(
                        boundaries[index + 1]["incoming_action_index"],
                        boundary["offered_action_index"]
                    );
                    assert_eq!(
                        boundaries[index + 1]["incoming_reward"],
                        steps[index]["reward"]
                    );
                    assert_eq!(
                        boundaries[index + 1]["incoming_terminal"],
                        steps[index]["terminal"]
                    );
                } else {
                    assert!(boundary["offered_action_index"].is_null());
                    assert_eq!(boundary["terminal"], "death");
                }
            }

            for high_water in case["high_water"].as_object().expect("high-water map").values() {
                let used = high_water["m9_compatible_high_water"].as_u64().unwrap();
                let capacity = high_water["capacity"].as_u64().unwrap();
                assert!(used <= capacity, "measured group high-water exceeds capacity");
            }
        }
    }
}
