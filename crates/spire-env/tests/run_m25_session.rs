use serde_json::{json, Value};
use spire_core::{
    combat::rng::Partition,
    content::Character,
};
use spire_env::*;

fn config(
    act1: RunAct1BossTarget,
    act2: RunAct2BossTarget,
    act3: RunAct3BossTarget,
) -> RunSessionConfig {
    RunSessionConfig {
        character: Character::Watcher,
        ascension: 0,
        partition: Partition::Test,
        boss_history: RunBossHistory::M25WatcherHeart { act1, act2, act3 },
        capacities: RunCapacities::m10_default(),
        record_replay: true,
    }
}

fn valid_record() -> RunEpisodeRecord {
    let mut session = RunSession::new_m25(config(
        RunAct1BossTarget::Guardian,
        RunAct2BossTarget::Champ,
        RunAct3BossTarget::AwakenedOne,
    ))
    .expect("valid M25 config");
    session.reset(Some(7)).expect("M25 H1 scenario reset");
    session.record().expect("record enabled").expect("record exists").clone()
}

fn changed_record(record: &[u8], mutate: impl FnOnce(&mut Value)) -> Vec<u8> {
    let mut value: Value = serde_json::from_slice(record).expect("serialized record JSON");
    mutate(&mut value);
    serde_json::to_vec(&value).expect("mutated record JSON")
}

fn duplicate_raw_key(raw: &str, needle: &str, duplicate: &str) -> Vec<u8> {
    assert!(raw.contains(needle), "missing duplicate fixture {needle}");
    raw.replacen(needle, duplicate, 1).into_bytes()
}

#[test]
fn m25_config_history_validation_precedes_initialization() {
    let act1 = [
        RunAct1BossTarget::Guardian,
        RunAct1BossTarget::Hexaghost,
        RunAct1BossTarget::SlimeBoss,
    ];
    let act2 = [
        RunAct2BossTarget::Champ,
        RunAct2BossTarget::BronzeAutomaton,
        RunAct2BossTarget::Collector,
    ];
    let act3 = [
        RunAct3BossTarget::AwakenedOne,
        RunAct3BossTarget::TimeEater,
        RunAct3BossTarget::DonuAndDeca,
    ];
    for a1 in act1 {
        for a2 in act2 {
            for a3 in act3 {
                let mut session = RunSession::new_m25(config(a1, a2, a3))
                    .unwrap_or_else(|error| panic!("accepted triple {a1:?}/{a2:?}/{a3:?}: {error:?}"));
                session.reset(Some(7)).expect("accepted triple initializes");
                let wire = session.record().expect("record enabled").expect("record exists")
                    .to_json().expect("history codec exports");
                let record: Value = serde_json::from_slice(&wire).expect("record JSON");
                let act1_name = match a1 {
                    RunAct1BossTarget::Guardian => "guardian",
                    RunAct1BossTarget::Hexaghost => "hexaghost",
                    RunAct1BossTarget::SlimeBoss => "slime_boss",
                };
                let act2_name = match a2 {
                    RunAct2BossTarget::Champ => "champ",
                    RunAct2BossTarget::BronzeAutomaton => "bronze_automaton",
                    RunAct2BossTarget::Collector => "collector",
                };
                let act3_name = match a3 {
                    RunAct3BossTarget::AwakenedOne => "awakened-one",
                    RunAct3BossTarget::TimeEater => "time-eater",
                    RunAct3BossTarget::DonuAndDeca => "donu-and-deca",
                };
                assert_eq!(record["config"]["boss_history"]["m25-watcher-heart"], json!({
                    "act1": act1_name, "act2": act2_name, "act3": act3_name
                }), "history spelling for {a1:?}/{a2:?}/{a3:?}");
            }
        }
    }

    let base = config(act1[0], act2[0], act3[0]);
    let wrong_histories = [
        RunBossHistory::AwakenedUnseen,
        RunBossHistory::TimeEaterUnseen,
        RunBossHistory::DonuDecaUnseen,
        RunBossHistory::Act1GuardianUnseen,
        RunBossHistory::Act1HexaghostUnseen,
        RunBossHistory::Act1SlimeBossUnseen,
        RunBossHistory::M13SilentAct2 { act1: act1[0], act2: act2[0] },
        RunBossHistory::M14SilentAct3 { act1: act1[0], act2: act2[0], act3: act3[0] },
        RunBossHistory::M15SilentHeart { act1: act1[0], act2: act2[0], act3: act3[0] },
        RunBossHistory::M17DefectAct1 { act1: act1[0] },
        RunBossHistory::M18DefectAct2 { act1: act1[0], act2: act2[0] },
        RunBossHistory::M19DefectAct3 { act1: act1[0], act2: act2[0], act3: act3[0] },
        RunBossHistory::M20DefectHeart { act1: act1[0], act2: act2[0], act3: act3[0] },
        RunBossHistory::M23WatcherAct2 { act1: act1[0], act2: act2[0] },
        RunBossHistory::M24WatcherAct3 { act1: act1[0], act2: act2[0], act3: act3[0] },
    ];
    for history in wrong_histories {
        assert!(
            RunSession::new_m25(RunSessionConfig { boss_history: history, ..base.clone() }).is_err(),
            "rejected non-M25 history {history:?} before initialization"
        );
    }
    assert!(RunSession::new_m25(RunSessionConfig { character: Character::Silent, ..base.clone() }).is_err());
    assert!(RunSession::new_m25(RunSessionConfig { character: Character::Defect, ..base.clone() }).is_err());
    assert!(RunSession::new_m25(RunSessionConfig { ascension: 1, ..base.clone() }).is_err());

    let record = valid_record().to_json().expect("valid record JSON");
    for value in ["awakened_one", "time_eater", "donu_and_deca"] {
        let invalid = changed_record(&record, |v| {
            v["config"]["boss_history"]["m25-watcher-heart"]["act3"] = json!(value);
        });
        assert!(RunEpisodeRecord::from_json(&invalid).is_err(), "snake_case Act3 value {value} rejects");
    }
}

#[test]
fn m25_raw_json_strict() {
    let record = valid_record();
    let raw = record.to_json().expect("record JSON");
    let value: Value = serde_json::from_slice(&raw).expect("record parses as JSON");
    assert_eq!(
        value["config"]["boss_history"],
        json!({"m25-watcher-heart":{"act1":"guardian","act2":"champ","act3":"awakened-one"}})
    );
    assert_eq!(RunEpisodeRecord::from_json(&raw), Ok(record));

    let raw_text = std::str::from_utf8(&raw).expect("canonical JSON is UTF-8");
    for (needle, duplicate) in [
        ("\"profile_id\":\"M25WatcherHeart\"", "\"profile_id\":\"M25WatcherHeart\",\"profile_id\":\"M25WatcherHeart\""),
        ("\"character\":\"watcher\"", "\"character\":\"watcher\",\"character\":\"watcher\""),
        ("\"m25-watcher-heart\":{", "\"m25-watcher-heart\":{},\"m25-watcher-heart\":{"),
        ("\"act3\":\"awakened-one\"", "\"act3\":\"awakened-one\",\"act3\":\"awakened-one\""),
    ] {
        assert!(RunEpisodeRecord::from_json(&duplicate_raw_key(raw_text, needle, duplicate)).is_err());
    }
    let actions = format!("\"actions\":{}", value["config"]["capacities"]["actions"]);
    let duplicate_actions = format!("{actions},{actions}");
    assert!(RunEpisodeRecord::from_json(&duplicate_raw_key(raw_text, &actions, &duplicate_actions)).is_err());

    for invalid in [
        changed_record(&raw, |v| v["unexpected"] = json!(1)),
        changed_record(&raw, |v| v["config"]["unexpected"] = json!(1)),
        changed_record(&raw, |v| v["config"]["capacities"]["unexpected"] = json!(1)),
        changed_record(&raw, |v| v["config"]["boss_history"]["unexpected"] = json!("value")),
        changed_record(&raw, |v| v["config"]["boss_history"]["m25-watcher-heart"]["unexpected"] = json!("value")),
        changed_record(&raw, |v| v["config"]["boss_history"]["m25-watcher-heart"]["act3"] = json!(7)),
    ] {
        assert!(RunEpisodeRecord::from_json(&invalid).is_err(), "unknown or wrong-type raw field accepted");
    }
}
