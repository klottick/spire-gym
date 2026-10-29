mod m20_support;
use m20_support::*;
use spire_env::*;
#[test]
fn m20_short_diagnostic_portable_replay_uses_same_choices() {
    let cfg=default_config();let mut core=diagnostic_run();let mut s=RunSession::new_m20(cfg.clone()).unwrap();let b=s.reset(Some(7)).unwrap();
    assert_eq!(s.reproduction().unwrap().latest_decision.unwrap().unwrap(),core.decision().unwrap());
    let action=core.decision().unwrap().candidates[0].action.clone();let t=core.step(&action).unwrap();
    let result=s.step(b.token,RunActionIndex(0)).unwrap();assert_eq!(result.reward,t.reward);
    let record=s.record().unwrap().unwrap();let bytes=record.to_json().unwrap();let decoded=RunEpisodeRecord::from_json(&bytes).unwrap();
    let report=replay_run(&decoded,cfg.capacities).unwrap();assert_eq!(report.final_decision,core.decision().unwrap());assert_eq!(report.final_map,core.public_map());assert_eq!(report.accepted_count,1);
}

#[test]
fn m20_generated_offer_semantic_codec_is_additive_and_exact(){
    use spire_core::{combat::CombatAction,run::GameAction};
    let mut s=RunSession::new_m20(default_config()).unwrap();s.reset(Some(7)).unwrap();
    let base=s.record().unwrap().unwrap();
    for action in [CombatAction::ChooseCardOffer{index:2},CombatAction::SkipCardOffer]{
        let mut record=base.clone();record.steps.push(RunRecordedStep{action:GameAction::Combat(action),reward:0,terminal:None});
        // Codec-only witness. These synthetic steps are never executed/replayed.
        let wire=record.to_json().unwrap();assert_eq!(RunEpisodeRecord::from_json(&wire).unwrap(),record);
        let mut value:serde_json::Value=serde_json::from_slice(&wire).unwrap();value["steps"][0]["action"]["extra"]=serde_json::json!(0);
        assert!(RunEpisodeRecord::from_json(&serde_json::to_vec(&value).unwrap()).is_err());
        let mut old=record;old.profile_id="M19DefectAct3".into();assert!(old.to_json().is_err());
    }
    let mut value:serde_json::Value=serde_json::from_slice(&base.to_json().unwrap()).unwrap();value["supplied_start"]=serde_json::json!("Strong");
    assert!(RunEpisodeRecord::from_json(&serde_json::to_vec(&value).unwrap()).is_err());
    value.as_object_mut().unwrap().remove("supplied_start");value["config"]["supplied_start"]=serde_json::json!("Strong");
    assert!(RunEpisodeRecord::from_json(&serde_json::to_vec(&value).unwrap()).is_err());
}
