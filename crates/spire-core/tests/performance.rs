//! Benchmark behavior tests: no wall-clock pass/fail limits.
#[path = "../examples/performance_support/mod.rs"]
mod support;
use serde_json::json;
use support::*;

#[test]
fn three_real_workloads_have_expected_endpoints() {
    for id in CaseId::ALL {
        let case = prepare(id).unwrap();
        let endpoint = preflight(&case).unwrap();
        match id {
            CaseId::Combat => {
                assert_eq!(endpoint["outcome"], "victory");
                assert_eq!(endpoint["enemy_hp"], 0);
                assert_eq!(endpoint["candidates"], 0);
            }
            CaseId::Act1 => {
                assert_eq!(endpoint["accepted_actions"], 212);
                assert_eq!(endpoint["outcome"], "Act2Ready");
                assert_eq!(endpoint["reward"], 17);
                assert_eq!(endpoint["hp"], 95);
            }
            CaseId::Act2Prefix => {
                assert_eq!(endpoint["accepted_actions"], 397);
                assert_eq!(endpoint["outcome"], "Act2PrefixComplete");
                assert_eq!(endpoint["hp"], 91);
                assert_eq!(endpoint["floor"], 26);
                assert_eq!(endpoint["reward"], 26);
                assert_eq!(endpoint["candidates"], 0);
                assert!(endpoint["terminal"].is_null());
            }
        }
    }
}

#[test]
fn fresh_replays_match_prepared_public_boundaries_and_rewards() {
    let case = prepare(CaseId::Act2Prefix).unwrap();
    let mut a = fresh(&case).unwrap();
    let mut b = fresh(&case).unwrap();
    let x = a.replay(&case, Boundary::Native).unwrap();
    let y = b.replay(&case, Boundary::Public).unwrap();
    assert_eq!(x, y);
    assert_eq!(a.endpoint(x).unwrap(), b.endpoint(y).unwrap());
    assert_eq!(preflight(&case).unwrap()["reward"], 26);
}

#[test]
fn altered_or_truncated_script_is_not_completion() {
    let mut case = prepare(CaseId::Act1).unwrap();
    case.actions.pop();
    assert!(preflight(&case).is_err());
    let mut case = prepare(CaseId::Combat).unwrap();
    case.actions.swap(0, 1);
    assert!(preflight(&case).is_err());
}

#[test]
fn creation_and_ordered_action_changes_change_digest() {
    let case = prepare(CaseId::Combat).unwrap();
    let mut creation = case.input.clone();
    creation["entry"]["player"]["hp"] = json!(79);
    assert_ne!(canonical_hash(&creation), canonical_hash(&case.input));
    let mut actions = case.action_values();
    actions.reverse();
    assert_ne!(canonical_hash(&json!(actions)), case.actions_sha256());
}

#[test]
fn canonical_objects_ignore_key_insertion_order() {
    assert_eq!(canonical_hash(&json!({"a":1,"b":2})),
               canonical_hash(&serde_json::from_str::<serde_json::Value>("{\"b\":2,\"a\":1}").unwrap()));
}

#[test]
fn sampling_defaults_and_boundaries() {
    let default = Sampling::default();
    assert_eq!((default.warmup, default.samples, default.iterations), (1,7,10));
    for settings in [Sampling {warmup:0,samples:1,iterations:1}, Sampling {warmup:100,samples:100,iterations:1000}] {
        assert!(settings.validate().is_ok());
    }
    for settings in [Sampling {warmup:101,..default}, Sampling {samples:0,..default},
                     Sampling {samples:101,..default}, Sampling {iterations:0,..default},
                     Sampling {iterations:1001,..default}] {
        assert!(settings.validate().is_err());
    }
    for args in [vec!["--samples","-1"],vec!["--samples","999999999999999999999999"],
                 vec!["--samples"],vec!["--samples","1","--samples","2"],vec!["--seed","7"]] {
        assert!(parse_args(&args.into_iter().map(str::to_owned).collect::<Vec<_>>()).is_err());
    }
}

#[test]
fn planned_or_unknown_filter_rejects() {
    let options = Sampling {warmup:0,samples:1,iterations:1};
    assert!(run_suite(options,Some("python")).is_err());
    assert!(run_suite(options,Some("unknown")).is_err());
}

#[test]
fn native_and_public_modes_have_same_work_and_outcome() {
    let case=prepare(CaseId::Act1).unwrap();
    let options=Sampling {warmup:0,samples:1,iterations:1};
    let a=measure(&case,Boundary::Native,options).unwrap();
    let b=measure(&case,Boundary::Public,options).unwrap();
    for key in ["input_sha256","actions_sha256","accepted_per_episode","endpoint"] {
        assert_eq!(a[key],b[key]);
    }
    assert_eq!(a["samples"][0]["accepted_actions"],212);
}

#[test]
fn warmups_are_not_samples() {
    let case=prepare(CaseId::Combat).unwrap();
    for warmup in [0,1] {
        let row=measure(&case,Boundary::Native,Sampling {warmup,samples:1,iterations:1}).unwrap();
        assert_eq!(row["samples"].as_array().unwrap().len(),1);
        assert_eq!(row["samples"][0]["episodes"],1);
        assert_eq!(row["samples"][0]["accepted_actions"],case.actions.len());
    }
}
