//! Shared power and intent vocabulary owner for every observation encoder.
//! Both tables derive from the engine registry in `spire_core::combat::public_ids`,
//! so no encoder can lack a power or intent the public combat view can publish.
use crate::RunSessionError;
use serde_json::{json,Value};
use spire_core::combat::public_ids::{PUBLIC_INTENTS,PUBLIC_POWERS};

fn padded(names:&[&str])->Value{json!(std::iter::once("padding").chain(names.iter().copied()).collect::<Vec<_>>())}
fn index(table:&[&str],name:&str)->Option<i64>{table.iter().position(|value|*value==name).map(|index|index as i64+1)}
/// Padded power vocabulary: row 0 is padding, row i+1 is `PUBLIC_POWERS[i]`.
pub(crate) fn powers()->Value{padded(PUBLIC_POWERS)}
/// Padded intent vocabulary: row 0 is padding, row i+1 is `PUBLIC_INTENTS[i]`.
pub(crate) fn intents()->Value{padded(PUBLIC_INTENTS)}
/// Replace a descriptor's power and intent vocabularies with the shared tables.
pub(crate) fn install(vocab:&mut Value){vocab["power"]=powers();vocab["intent"]=intents();}
pub(crate) fn power(name:&str)->Result<i64,RunSessionError>{index(PUBLIC_POWERS,name).ok_or(RunSessionError::Input("unregistered public power"))}
pub(crate) fn intent(name:&str)->Result<i64,RunSessionError>{index(PUBLIC_INTENTS,name).ok_or(RunSessionError::Input("unregistered public intent"))}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{RunAct1BossTarget as A1,RunAct2BossTarget as A2,RunAct3BossTarget as A3,RunActionIndex,RunBossHistory as H,RunCapacities,RunObservation,RunSession,RunSessionConfig};
    use spire_core::{combat::{rng::Partition,FullIntentId,FullPowerId,FullPowerView},content::Character,run::PublicDecision};
    type Encoder=fn(&PublicDecision,&RunCapacities)->Result<RunObservation,RunSessionError>;

    fn config(character:Character,boss_history:H)->RunSessionConfig{RunSessionConfig{character,ascension:0,partition:Partition::Test,boss_history,capacities:RunCapacities::m10_default(),record_replay:false}}
    /// First public combat decision reached by always taking the first legal action.
    fn first_combat(mut session:RunSession)->PublicDecision{
        let mut token=session.reset(Some(7)).unwrap().token;
        for _ in 0..64{
            let decision=session.reproduction().unwrap().latest_decision.unwrap().unwrap();
            if decision.combat.as_ref().is_some_and(|c|!c.enemies.is_empty()){return decision}
            token=session.step(token,RunActionIndex(0)).unwrap().boundary.token;
        }
        panic!("no combat decision within 64 first-legal steps")
    }
    /// Strip noncombat routing so an authored-combat encoder accepts a Run combat payload.
    fn combat_only(mut d:PublicDecision)->PublicDecision{d.room=None;d.content_profile=None;d.offers.clear();d.last_combat=None;d.player.relics.clear();d.player.relic_counters.clear();d.player.potions.iter_mut().for_each(|p|*p=None);
        if let Some(i)=d.combat.as_mut().and_then(|c|c.inventory.as_mut()){i.relics.clear();i.relic_counters.clear();i.potions.iter_mut().for_each(|p|*p=None);}d.candidates.retain(|c|matches!(c.action,spire_core::run::GameAction::Combat(_)));d}
    fn encoders()->Vec<(&'static str,PublicDecision,Encoder)>{
        let ironclad=|h|config(Character::Ironclad,h);let silent=|h|config(Character::Silent,h);let defect=|h|config(Character::Defect,h);let watcher=|h|config(Character::Watcher,h);
        let (g,c,a)=(A1::Guardian,A2::Champ,A3::AwakenedOne);
        let m9=first_combat(RunSession::new(ironclad(H::AwakenedUnseen)).unwrap());
        let m10=first_combat(RunSession::new_m10(ironclad(H::AwakenedUnseen)).unwrap());
        let m17=first_combat(RunSession::new_m17(defect(H::M17DefectAct1{act1:g})).unwrap());
        let m22=first_combat(RunSession::new_m22(watcher(H::Act1GuardianUnseen)).unwrap());
        vec![
            ("m9",m9,|d,c|crate::encode_run(d,None,c)),
            ("m10",m10.clone(),|d,c|crate::encode_run_m10(d,None,c)),
            ("m10_policy",m10.clone(),|d,c|crate::encode_run_policy(d,None,c)),
            ("combat_policy_v1",m10.clone(),|d,_|crate::combat_policy::encode_combat_policy(d,None)),
            ("combat_policy_v2",m10,|d,_|crate::combat_policy::encode_combat_policy_v2(d,None)),
            ("m12",first_combat(RunSession::new_m12(silent(H::Act1GuardianUnseen)).unwrap()),|d,c|crate::encode_run_m12(d,None,c)),
            ("m13",first_combat(RunSession::new_m13(silent(H::M13SilentAct2{act1:g,act2:c})).unwrap()),|d,c|crate::encode_run_m13(d,None,c)),
            ("m14",first_combat(RunSession::new_m14(silent(H::M14SilentAct3{act1:g,act2:c,act3:a})).unwrap()),|d,c|crate::encode_run_m14(d,None,c)),
            ("m15",first_combat(RunSession::new_m15(silent(H::M15SilentHeart{act1:g,act2:c,act3:a})).unwrap()),|d,c|crate::encode_run_m15(d,None,c)),
            ("m16_defect_combat",combat_only(m17.clone()),|d,c|crate::encode_defect_combat(d,c)),
            ("m17",m17,|d,c|crate::encode_run_m17(d,None,c)),
            ("m18",first_combat(RunSession::new_m18(defect(H::M18DefectAct2{act1:g,act2:c})).unwrap()),|d,c|crate::encode_run_m18(d,None,c)),
            ("m19",first_combat(RunSession::new_m19(defect(H::M19DefectAct3{act1:g,act2:c,act3:a})).unwrap()),|d,c|crate::encode_run_m19(d,None,c)),
            ("m20",first_combat(RunSession::new_m20(defect(H::M20DefectHeart{act1:g,act2:c,act3:a})).unwrap()),|d,c|crate::encode_run_m20(d,None,c)),
            ("m21_watcher_combat",combat_only(m22.clone()),|d,c|crate::encode_watcher_combat(d,c)),
            ("m22",m22,|d,c|crate::encode_run_m22(d,None,c)),
            ("m23",first_combat(RunSession::new_m23(watcher(H::M23WatcherAct2{act1:g,act2:c})).unwrap()),|d,c|crate::encode_run_m23(d,None,c)),
            ("m24",first_combat(RunSession::new_m24(watcher(H::M24WatcherAct3{act1:g,act2:c,act3:a})).unwrap()),|d,c|crate::encode_run_m24(d,None,c)),
            ("m25",first_combat(RunSession::new_m25(watcher(H::M25WatcherHeart{act1:g,act2:c,act3:a})).unwrap()),|d,c|crate::encode_run_m25(d,None,c)),
        ]
    }
    fn power_rows(o:&RunObservation)->Vec<i64>{let g=o.group("powers").unwrap();let occupied=g.occupied.as_ref().unwrap();
        g.values.chunks_exact(4).zip(occupied).filter(|(_,o)|**o==1).map(|(row,_)|row[2]).collect()}

    #[test]
    fn unregistered_names_are_rejected_not_aliased(){
        assert_eq!(power("not_a_registered_power"),Err(RunSessionError::Input("unregistered public power")));
        assert_eq!(intent("not_a_registered_intent"),Err(RunSessionError::Input("unregistered public intent")));
        assert_eq!(power("padding"),Err(RunSessionError::Input("unregistered public power")));
        assert_eq!((power(PUBLIC_POWERS[0]),intent(PUBLIC_INTENTS[0])),(Ok(1),Ok(1)));
    }

    #[test]
    fn registry_tables_are_unique_and_unpadded(){
        for table in [PUBLIC_POWERS,PUBLIC_INTENTS]{
            let mut seen=std::collections::BTreeSet::new();
            for name in table{assert!(seen.insert(*name),"duplicate registry name {name}");assert_ne!(*name,"padding");}
        }
    }

    /// Walk the engine's full public power and intent registry through every encoder.
    #[test]
    fn every_encoder_encodes_every_registered_power_and_intent(){
        let caps=RunCapacities::m10_default();
        for (name,decision,encode) in encoders(){
            encode(&decision,&caps).unwrap_or_else(|e|panic!("{name} rejects its own unmodified combat decision: {e:?}"));
            let rounds=PUBLIC_POWERS.len().max(PUBLIC_INTENTS.len());
            for round in 0..rounds{
                let power=PUBLIC_POWERS[round%PUBLIC_POWERS.len()];let intent=PUBLIC_INTENTS[round%PUBLIC_INTENTS.len()];
                let mut d=decision.clone();let combat=d.combat.as_mut().unwrap();
                combat.player.powers=vec![FullPowerView{id:FullPowerId(power),amount:1}];
                combat.enemies[0].powers=vec![FullPowerView{id:FullPowerId(power),amount:2}];
                combat.enemies[0].intent=FullIntentId::from_name(intent);
                let observation=encode(&d,&caps).unwrap_or_else(|e|panic!("{name} cannot encode power {power} / intent {intent}: {e:?}"));
                let id=self::power(power).unwrap();
                assert_eq!(power_rows(&observation),vec![id,id],"{name} power {power}");
                let enemies=observation.group("enemies").unwrap();let cols=*enemies.shape.last().unwrap();
                assert_eq!(enemies.values[4],self::intent(intent).unwrap(),"{name} intent {intent} (row width {cols})");
            }
        }
    }

    /// Every descriptor publishes exactly the shared tables, so indices agree with the encoder.
    #[test]
    fn every_descriptor_uses_the_shared_power_and_intent_vocabulary(){
        let c=RunCapacities::m10_default();
        let specs=[crate::run_observation_spec(&c),crate::m10_run_observation_spec(&c),crate::policy_run_observation_spec(&c),
            crate::combat_policy_observation_spec(),crate::combat_policy_observation_spec_v2(),crate::m12_run_observation_spec(&c),
            crate::m13_run_observation_spec(&c),crate::m14_run_observation_spec(&c),crate::m15_run_observation_spec(&c),
            crate::defect_combat_observation_spec(&c),crate::m17_run_observation_spec(&c),crate::m18_run_observation_spec(&c),
            crate::m19_run_observation_spec(&c),crate::m20_run_observation_spec(&c),crate::watcher_combat_observation_spec(&c),
            crate::m22_run_observation_spec(&c),crate::m23_run_observation_spec(&c),crate::m24_run_observation_spec(&c),crate::m25_run_observation_spec(&c)];
        for spec in specs{
            let d=spec.unwrap().descriptor();let id=d["schema_id"].clone();
            assert_eq!(d["vocabularies"]["power"],powers(),"{id} power vocabulary");
            assert_eq!(d["vocabularies"]["intent"],intents(),"{id} intent vocabulary");
            for tensor in d["tensors"].as_array().unwrap(){
                for column in tensor["columns"].as_array().into_iter().flatten(){
                    let high=column["high"].as_u64();
                    match column["field"].as_str(){
                        Some("power_kind")=>assert_eq!(high,Some(PUBLIC_POWERS.len() as u64),"{id} power_kind high"),
                        Some("intent_kind")=>assert_eq!(high,Some(PUBLIC_INTENTS.len() as u64),"{id} intent_kind high"),
                        _=>{}
                    }
                }
            }
        }
    }
}
