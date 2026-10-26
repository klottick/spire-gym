use spire_env::*;
use spire_core::{combat::rng::Partition,content::Character};
fn catalog()->CombatTrainingCatalog{CombatTrainingCatalog::from_json(br#"{"entries":[{"id":"wide","encounter":"three_sentries","hp":50,"max_hp":80,"deck":[{"card":"immolate","upgrades":1},{"card":"warcry","upgrades":1},{"card":"armaments","upgrades":1},{"card":"whirlwind","upgrades":0},{"card":"defend","upgrades":0}],"relics":["burning_blood","anchor"],"potions":["elixir",null,null]}]}"#).unwrap()}
#[test]
fn training_catalog_validates_hidden_inputs_and_numeric_types(){
    let c=catalog();assert_eq!(c.descriptor()["entries"][0]["id"],"wide");
    for bytes in [br#"{"entries":[],"draw":[]}"#.as_slice(),br#"{"entries":[{"id":"a","encounter":"cultist","hp":true,"max_hp":80,"deck":[],"relics":[],"potions":[null,null,null]}]}"#]{assert!(CombatTrainingCatalog::from_json(bytes).is_err());}
}
#[test]
fn training_view_seed_masks_tokens_and_public_metadata(){
    let mut s=CombatTrainingSession::new(catalog(),Partition::Test).unwrap();let b=s.reset(Some(7),None).unwrap();
    assert_eq!(b.info["routing"],"combat");assert!(b.observation.action_mask.iter().any(|x|*x==1));
    assert!(b.observation.group("map_nodes").is_none());assert_eq!(b.observation.group("enemies").unwrap().occupied.as_ref().unwrap().iter().sum::<i8>(),3);
    let spec=s.observation_spec().unwrap();assert_eq!(spec.descriptor()["schema_id"],COMBAT_POLICY_VIEW_ID);
    let identity=s.environment_spec().unwrap();assert_eq!(identity["profile_id"],"ironclad-combat-v1");assert!(identity["rules_id"].is_string());
    assert!(s.step(b.token,RunActionIndex(-1)).is_err());assert_eq!(s.decision().unwrap(),b);
    let next=s.step(b.token,RunActionIndex(0)).unwrap();assert!(s.step(b.token,RunActionIndex(0)).is_err());
    assert_ne!(next.boundary.token,b.token);assert_eq!(b.observation.group("player").unwrap().values[0],50);
}
#[test]
fn run_common_view_does_not_change_old_contract_or_routing(){
    let mut s=RunSession::new_m10(RunSessionConfig{character:Character::Ironclad,ascension:0,partition:Partition::Test,boss_history:M9BossHistory::AwakenedUnseen,capacities:RunCapacities::m10_default(),record_replay:true}).unwrap();
    let spec=s.observation_spec().unwrap();let b=s.reset(Some(11)).unwrap();let r=s.record().unwrap();let v=s.combat_view().unwrap();
    assert_eq!(v.info["routing"],"noncombat");assert!(v.observation.action_mask.iter().all(|x|*x==0));assert_eq!(v.token,b.token);
    assert_eq!(s.observation_spec().unwrap(),spec);assert_eq!(s.record().unwrap(),r);
    let ids=s.environment_spec().unwrap();assert_eq!(ids["rules_id"],RUN_RULES_ID_M10);
}
#[test]
fn training_delivery_fault_is_not_death_and_cannot_reset_or_retry(){
    let mut s=CombatTrainingSession::new(catalog(),Partition::Test).unwrap();let b=s.reset(Some(1),None).unwrap();
    s.step(b.token,RunActionIndex(0)).unwrap();s.fail_delivery("test export fault");
    assert!(s.reset(Some(1),None).is_err());assert!(s.decision().is_err());assert_eq!(s.diagnostic()["accepted_steps"],1);assert_eq!(s.diagnostic()["fault"],"DeliveryFault(\"test export fault\")");
    s.close().unwrap();assert_eq!(s.diagnostic()["accepted_steps"],1);
}
#[test]
fn training_all_advertised_encounters_publish_numeric_views_through_real_settlement(){
    for row in combat_training_support()["encounters"].as_array().unwrap(){
        let raw=serde_json::json!({"entries":[{"id":"case","encounter":row["id"],"hp":1,"max_hp":80,"deck":[{"card":"strike","upgrades":0}],"relics":[],"potions":[null,null,null]}]});
        let c=CombatTrainingCatalog::from_json(&serde_json::to_vec(&raw).unwrap()).unwrap();let mut s=CombatTrainingSession::new(c,Partition::Test).unwrap();let mut b=s.reset(Some(11),None).unwrap();
        for _ in 0..100{if b.info["last_exit"].is_object(){break}let g=b.observation.group("actions").unwrap();let idx=g.values.chunks(8).position(|r|r[0]==10).unwrap();let step=s.step(b.token,RunActionIndex(idx as i64)).unwrap_or_else(|e|panic!("{}: {e:?}",row["id"]));b=step.boundary;}
        assert!(b.info["last_exit"].is_object(),"{}",row["id"]);assert!(b.observation.action_mask.iter().all(|x|*x==0));
    }
}
#[path="../../spire-core/examples/m10_support/mod.rs"]mod normal_support;
#[test]
fn normal_run_uses_same_dispatch_and_real_potion_selector_then_continues_after_victory(){
    use spire_core::{run::GameAction,combat::CombatAction,content::PotionId};
    let trace=normal_support::rollout(11,normal_support::M9BossHistory::AwakenedUnseen,8192).unwrap();
    let mut core=normal_support::start(11,normal_support::M9BossHistory::AwakenedUnseen).unwrap();
    let mut s=RunSession::new_m10(RunSessionConfig{character:Character::Ironclad,ascension:0,partition:Partition::Test,boss_history:M9BossHistory::AwakenedUnseen,capacities:RunCapacities::m10_default(),record_replay:false}).unwrap();s.reset(Some(11)).unwrap();
    let mut saw_selector=false;let mut saw_noncombat_selector=false;let mut saw_victory_after_selector=false;let mut settled=0;
    for expected in trace.trace {
        let mut d=core.decision().unwrap();let mut view=s.combat_view().unwrap();
        assert_eq!(view.info["active"],d.combat.is_some());
        if let Some(c)=&d.combat {
            let full=s.decision().unwrap();let full_actions=full.observation.group("actions").unwrap();let common=view.observation.group("actions").unwrap();
            assert_eq!(&full_actions.values[..common.values.len()],common.values.as_slice());
            if !saw_selector {if let Some(slot)=c.inventory.as_ref().unwrap().potions.iter().position(|p|*p==Some(PotionId::Elixir)) {
                let action=GameAction::Combat(CombatAction::UsePotion{slot,target:None});let idx=d.candidates.iter().position(|v|v.action==action).unwrap();
                s.step(view.token,RunActionIndex(idx as i64)).unwrap();core.step(&action).unwrap();view=s.combat_view().unwrap();
                assert_eq!(view.info["routing"],"combat_selection");assert_eq!(view.info["selection_owner"],"potion");
                let d2=core.decision().unwrap();let action=GameAction::Combat(CombatAction::ConfirmSelection);let idx=d2.candidates.iter().position(|v|v.action==action).unwrap();
                s.step(view.token,RunActionIndex(idx as i64)).unwrap();core.step(&action).unwrap();
                view=s.combat_view().unwrap();assert_eq!(view.observation.group("potions").unwrap().values[slot],0);
                saw_selector=true;d=core.decision().unwrap();
            }}
        }else{assert!(view.observation.action_mask.iter().all(|v|*v==0));if d.selection.is_some(){saw_noncombat_selector=true;assert_eq!(view.info["routing"],"noncombat");}}
        let idx=d.candidates.iter().position(|v|v.action==expected.action).unwrap();let t=s.step(view.token,RunActionIndex(idx as i64)).unwrap();core.step(&expected.action).unwrap();
        let after=s.combat_view().unwrap();let after_d=core.decision().unwrap();
        if after_d.combat_progress.settled>settled {
            settled=after_d.combat_progress.settled;
            assert_eq!(after.info["last_exit"]["hp_after_cleanup"],after_d.player.hp);
            assert_eq!(after.observation.group("player").unwrap().values[0],i64::from(after_d.player.hp));
            assert_eq!(after.observation.group("permanent_cards").unwrap().occupied.as_ref().unwrap().iter().filter(|x|**x==1).count(),after_d.objects.len());
            if saw_selector&&after.info["last_exit"]["outcome"]=="victory"{assert!(!t.terminated);assert!(!t.truncated);assert!(t.boundary.terminal.is_none());saw_victory_after_selector=true;}
        }
        if saw_victory_after_selector&&saw_noncombat_selector{break}
    }
    assert!(saw_selector);assert!(saw_noncombat_selector);assert!(saw_victory_after_selector);
}
#[test]
fn training_all_red_variants_publish_and_card_selectors_keep_row_references(){
    let support=combat_training_support();
    for row in support["cards"].as_array().unwrap(){for upgraded in [0,1]{
        let raw=serde_json::json!({"entries":[{"id":"card","encounter":"cultist","hp":80,"max_hp":80,"deck":vec![serde_json::json!({"card":row["id"],"upgrades":upgraded});5],"relics":["mark_of_pain"],"potions":[null,null,null]}]});
        let c=CombatTrainingCatalog::from_json(&serde_json::to_vec(&raw).unwrap()).unwrap();let mut s=CombatTrainingSession::new(c,Partition::Test).unwrap();let b=s.reset(Some(11),None).unwrap();
        let a=b.observation.group("actions").unwrap();let index=a.values.chunks(8).position(|r|r[0]==8).unwrap();let next=s.step(b.token,RunActionIndex(index as i64)).unwrap_or_else(|e|panic!("{}+{upgraded}: {e:?}",row["id"])).boundary;
        assert!(next.observation.group("player").is_some());
        if row["id"]=="warcry" {
            assert_eq!(next.info["routing"],"combat_selection");assert_eq!(next.info["selection_owner"],"card");
            let g=next.observation.group("actions").unwrap();let index=g.values.chunks(8).position(|r|r[0]==9).unwrap();
            let card_row=g.values[index*8+2] as usize;assert_eq!(next.observation.group("combat_cards").unwrap().occupied.as_ref().unwrap()[card_row],1);
            s.step(next.token,RunActionIndex(index as i64)).unwrap();
        }
    }}
}
#[test]
fn training_catalog_is_unordered_reset_rejections_are_atomic_and_seeds_are_owned(){
    let c=catalog();let mut raw=c.descriptor();raw.as_object_mut().unwrap().remove("catalog_schema_id");raw["entries"][0]["deck"].as_array_mut().unwrap().reverse();raw["entries"][0]["relics"].as_array_mut().unwrap().reverse();
    let permuted=CombatTrainingCatalog::from_json(&serde_json::to_vec(&raw).unwrap()).unwrap();assert_eq!(c.fingerprint(),permuted.fingerprint());
    let mut a=CombatTrainingSession::new(c,Partition::Test).unwrap();let mut b=CombatTrainingSession::new(permuted,Partition::Test).unwrap();let first=a.reset(Some(7),None).unwrap();let other=b.reset(Some(7),None).unwrap();assert_eq!(first.observation,other.observation);
    assert!(a.reset(Some(99),Some("absent")).is_err());assert_eq!(a.decision().unwrap(),first);
    let second=a.reset(None,None).unwrap();assert_eq!(second.info["episode_ordinal"],1);assert_eq!(second.info["effective_seed"],7);
    let replay=a.reset(Some(7),None).unwrap();assert_eq!(first.observation,replay.observation);assert_ne!(first.token,replay.token);
}
#[test]
fn policy_run_profile_is_opt_in_versioned_and_replays_without_changing_m10(){
    let config=RunSessionConfig{character:Character::Ironclad,ascension:0,partition:Partition::Test,boss_history:M9BossHistory::AwakenedUnseen,capacities:RunCapacities::m10_default(),record_replay:true};
    let mut policy=RunSession::new_m10_policy(config.clone()).unwrap();let mut old=RunSession::new_m10(config).unwrap();
    let b=policy.reset(Some(11)).unwrap();let o=old.reset(Some(11)).unwrap();assert_eq!(b.observation,o.observation);
    assert_eq!(policy.observation_spec().unwrap().descriptor()["schema_id"],"spire-gym-run-observation-v3");
    assert_eq!(old.observation_spec().unwrap().descriptor()["schema_id"],"spire-gym-run-observation-v2");
    assert_ne!(policy.observation_spec().unwrap().fingerprint(),old.observation_spec().unwrap().fingerprint());
    policy.step(b.token,RunActionIndex(0)).unwrap();let record=policy.record().unwrap().unwrap();assert_eq!(record.schema_id,"spire-gym-run-replay-v3");
    let parsed=RunEpisodeRecord::from_json(&record.to_json().unwrap()).unwrap();let replay=replay_run(&parsed,RunCapacities::m10_default()).unwrap();assert_eq!(replay.accepted_count,1);
}
#[test]
fn training_flex_numeric_power_is_public_and_expires_at_actual_end_turn(){
    let raw=serde_json::json!({"entries":[{"id":"flex","encounter":"cultist","hp":80,"max_hp":80,"deck":vec![serde_json::json!({"card":"flex","upgrades":1});5],"relics":[],"potions":[null,null,null]}]});
    let mut s=CombatTrainingSession::new(CombatTrainingCatalog::from_json(&serde_json::to_vec(&raw).unwrap()).unwrap(),Partition::Test).unwrap();let b=s.reset(Some(11),None).unwrap();
    let next=s.step(b.token,RunActionIndex(0)).unwrap().boundary;let vocabulary=s.observation_spec().unwrap().descriptor()["vocabularies"]["power"].as_array().unwrap().clone();let loss=vocabulary.iter().position(|v|v=="lose_strength").unwrap() as i64;
    let rows=next.observation.group("powers").unwrap();assert!(rows.values.chunks(4).any(|r|r[2]==loss&&r[3]==4));
    let actions=next.observation.group("actions").unwrap();let end=actions.values.chunks(8).position(|r|r[0]==10).unwrap();let after=s.step(next.token,RunActionIndex(end as i64)).unwrap().boundary;
    let rows=after.observation.group("powers").unwrap();assert!(!rows.values.chunks(4).any(|r|r[2]==loss));assert!(!rows.values.chunks(4).any(|r|r[0]==1&&r[2]==1&&r[3]!=0));
    let replay=s.reset(Some(11),None).unwrap();assert_eq!(replay.observation,b.observation);
    let replay=s.step(replay.token,RunActionIndex(0)).unwrap().boundary;assert_eq!(replay.observation,next.observation);
    let replay=s.step(replay.token,RunActionIndex(end as i64)).unwrap().boundary;assert_eq!(replay.observation,after.observation);
}
#[test]
fn policy_run_identity_rejects_mixed_old_records(){
    let mut s=RunSession::new_m10_policy(RunSessionConfig{character:Character::Ironclad,ascension:0,partition:Partition::Test,boss_history:M9BossHistory::AwakenedUnseen,capacities:RunCapacities::m10_default(),record_replay:true}).unwrap();s.reset(Some(11)).unwrap();
    let r=s.record().unwrap().unwrap();let mut mixed=r.clone();mixed.schema_id=RUN_RECORD_SCHEMA_ID_M10.into();assert!(mixed.to_json().is_err());
    let mut mixed=r;mixed.profile_id="M10IroncladHeart".into();assert!(mixed.to_json().is_err());
    assert_eq!(s.environment_spec().unwrap()["profile_id"],"M10IroncladPolicy");
}
