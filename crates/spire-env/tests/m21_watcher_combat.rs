use spire_env::*;
use spire_core::{combat::{rng::Partition, CombatAction},content::*,run::*};
fn supplied(cards:&[CardId])->M21WatcherCombatStart{M21WatcherCombatStart::Supplied(AuthoredRunStart{hp:60,max_hp:72,gold:7,deck:cards.iter().enumerate().map(|(i,id)|CardInstance{instance:31+i as u64,id:*id,upgrades:0,misc:0}).collect(),relics:vec![],potions:vec![None;3],relic_counters:Default::default()})}
fn catalog()->CombatTrainingCatalog{CombatTrainingCatalog::m21(vec![M21WatcherCombatEntry{id:"A".into(),encounter:EncounterId::JawWorm,start:M21WatcherCombatStart::Canonical},M21WatcherCombatEntry{id:"B".into(),encounter:EncounterId::Cultist,start:supplied(&[CardId::DefendPurple])}]).unwrap()}
#[test]
fn m21_watcher_numeric_view_exposes_only_public_order(){
 let mut run=Run::prepare_m21_watcher_combat(M21WatcherCombatStart::Canonical,EncounterId::JawWorm,RunRng::new([7;32],Partition::Test,0)).unwrap();run.enter_training_combat(EncounterId::JawWorm).unwrap();
 let d=run.decision().unwrap();let caps=combat_policy_capacities();let spec=watcher_combat_observation_spec(&caps).unwrap();let obs=encode_watcher_combat(&d,&caps).unwrap();
 assert_eq!(spec.descriptor()["vocabularies"]["stance"],serde_json::json!(["neutral","wrath","calm","divinity"]));
 assert_eq!(obs.group("player").unwrap().values[8],0);assert!(obs.group("stance_choices").is_some());
 assert_eq!(obs.action_mask.iter().filter(|x|**x==1).count(),d.candidates.len());
}
#[test]
fn m21_watcher_entry_transfers_canonical_or_diagnostic_resources(){
 let c=catalog();assert_eq!(c.descriptor()["entries"].as_array().unwrap().len(),2);
 let mut s=CombatTrainingSession::new(c,Partition::Test).unwrap();assert!(s.record().unwrap().is_none());
 let a=s.reset(Some(7),Some("A")).unwrap();assert_eq!(a.observation.group("player").unwrap().values[0..3],[72,72,99]);assert_eq!(a.observation.group("permanent_cards").unwrap().occupied.as_ref().unwrap().iter().sum::<i8>(),10);
 let b=s.reset(None,Some("B")).unwrap();assert_eq!(b.info["episode_ordinal"],1);assert_eq!(b.observation.group("player").unwrap().values[0..3],[60,72,7]);
}
#[test]
fn m21_watcher_replay_validates_full_catalog_before_nonfirst_entry(){
 let c=catalog();let descriptor=c.descriptor();let fingerprint=c.fingerprint();let mut s=CombatTrainingSession::new(c,Partition::Test).unwrap();s.reset(Some(7),Some("A")).unwrap();let b=s.reset(None,Some("B")).unwrap();
 // B's one card is Defend, followed by EndTurn; use a semantic row, never a stored numeric identity.
 let end=b.observation.group("actions").unwrap();let index=(0..end.shape[0]).find(|i|end.known[i*end.shape[1]]==1&&end.values[i*end.shape[1]]==10).unwrap();s.step(b.token,RunActionIndex(index as i64)).unwrap();
 let r=s.record().unwrap().unwrap();assert_eq!(r.ordinal,1);assert_eq!(r.entry_id,"B");assert_eq!(r.catalog_descriptor,descriptor);assert_eq!(r.catalog_fingerprint,fingerprint);assert_eq!(r.selected_entry,r.catalog_descriptor["entries"][1]);
 let imported=CombatEpisodeRecord::from_json(&r.to_json().unwrap()).unwrap();assert_eq!(r,imported);let report=replay_m21_combat(&r,combat_policy_capacities()).unwrap();assert_eq!(report.accepted_count,1);assert_eq!(report.rewards,vec![0]);assert_eq!(report.final_boundary.unwrap().observation,s.decision().unwrap().observation);
 let mut bad=r.clone();bad.catalog_descriptor["entries"][0]["encounter"]=serde_json::json!("cultist");assert!(matches!(replay_m21_combat(&bad,combat_policy_capacities()),Err(RunSessionError::IncompatibleReplay(_))));assert!(bad.to_json().is_err());
 for id in ["A","absent"]{let mut bad=r.clone();bad.entry_id=id.into();assert!(replay_m21_combat(&bad,combat_policy_capacities()).is_err());}
}
fn one(start:M21WatcherCombatStart)->CombatTrainingCatalog{CombatTrainingCatalog::m21(vec![M21WatcherCombatEntry{id:"case".into(),encounter:EncounterId::Cultist,start}]).unwrap()}
fn core(start:M21WatcherCombatStart)->Run{let mut root=[0;32];root[..8].copy_from_slice(&7u64.to_le_bytes());let mut r=Run::prepare_m21_watcher_combat(start,EncounterId::Cultist,RunRng::new(root,Partition::Test,0)).unwrap();r.enter_training_combat(EncounterId::Cultist).unwrap();r}
fn step_pair(r:&mut Run,s:&mut CombatTrainingSession,a:CombatAction)->CombatTrainingStep{let d=r.decision().unwrap();let i=d.candidates.iter().position(|c|c.action==GameAction::Combat(a.clone())).unwrap();let b=s.decision().unwrap();r.step(&GameAction::Combat(a)).unwrap();s.step(b.token,RunActionIndex(i as i64)).unwrap()}
#[test]
fn m21_watcher_numeric_selection_orders_costs_flags_and_stance_choices(){
 use spire_core::combat::*;
 // A public projection fixture starts from a real decision; only public fields
 // are varied. Hidden piles/RNG never enter the numeric encoder.
 let r=core(supplied(&[CardId::WindmillStrike,CardId::Perseverance,CardId::SandsOfTime]));let mut d=r.decision().unwrap();
 let c=d.combat.as_mut().unwrap();c.character.stance=Stance::Divinity;c.mantra_gained=Some(23);
 c.hand[0].values.as_mut().unwrap().base_damage=Some(18);c.hand[0].values.as_mut().unwrap().base_block=Some(11);let w=c.hand[0].watcher.as_mut().unwrap();w.combat_cost=Some(1);w.turn_cost=Some(0);w.self_retain=true;w.retain=true;
 let a=c.hand[0].clone();let b=c.hand[1].clone();
 c.selection=Some(SelectionView{parent:SelectionParent::Power{turn:1,id:FullPowerId("foresight")},source_zone:SelectionSourceZone::Draw,disposition:SelectionDisposition::Discard,operation:SelectionOperation::Scry,required:0,minimum:0,maximum:2,candidates:vec![a.clone(),b.clone()],selected:vec![b.instance,a.instance]});
 d.candidates.clear();
 let caps=combat_policy_capacities();let o=encode_watcher_combat(&d,&caps).unwrap();assert_eq!(o.group("player").unwrap().values[8],3);assert_eq!(o.group("player").unwrap().values[12],23);
 let rows=o.group("selection").unwrap();assert_eq!(&rows.values[..6],&[4,0,1,1,0,2]);assert_eq!(&rows.values[6..12],&[4,1,1,1,1,1]);assert_eq!(&rows.known[..12],&[1;12]);
 let c=o.group("combat_cards").unwrap();assert_eq!([c.values[8],c.values[9],c.values[19],c.values[20],c.values[21],c.values[22]],[18,11,1,0,1,1]);
 let c=d.combat.as_mut().unwrap();c.selection=None;c.stance_choice={let mut start=supplied(&[CardId::DefendPurple]);if let M21WatcherCombatStart::Supplied(r)=&mut start{r.potions[0]=Some(PotionId::StancePotion);}let mut r=core(start);r.step(&GameAction::Combat(CombatAction::UsePotion{slot:0,target:None})).unwrap();r.decision().unwrap().combat.unwrap().stance_choice};
 d.candidates=vec![Candidate{id:ChoiceId(0),action:GameAction::Combat(CombatAction::ChooseStance{stance:Stance::Wrath}),label:"stance",neow_offer:None,object:None},Candidate{id:ChoiceId(1),action:GameAction::Combat(CombatAction::ChooseStance{stance:Stance::Calm}),label:"stance",neow_offer:None,object:None}];
 let o=encode_watcher_combat(&d,&caps).unwrap();let g=o.group("stance_choices").unwrap();assert_eq!(g.values,vec![1,2]);assert_eq!(g.occupied,Some(vec![1,1]));let g=o.group("actions").unwrap();assert_eq!(g.values[8],0);assert_eq!(g.values[17],1);assert_eq!(g.known[8],1);assert_eq!(g.known[17],1);assert_eq!(&o.action_mask[..3],&[1,1,0]);
}
#[test]
fn m21_watcher_semantic_replay_preserves_scry_and_deselection(){
 let start=supplied(&[CardId::ThirdEye;8]);let mut r=core(start.clone());let mut s=CombatTrainingSession::new(one(start),Partition::Test).unwrap();s.reset(Some(7),None).unwrap();
 let id=r.decision().unwrap().combat.unwrap().hand[0].instance.0;step_pair(&mut r,&mut s,CombatAction::PlayCard{instance:id,target:None});
 let sel=r.decision().unwrap().combat.unwrap().selection.unwrap();let a=sel.candidates[0].instance.0;let b=sel.candidates[1].instance.0;
 for action in [CombatAction::SelectCard{instance:a},CombatAction::SelectCard{instance:b},CombatAction::DeselectCard{instance:a},CombatAction::SelectCard{instance:a}]{step_pair(&mut r,&mut s,action);}
 let rec=s.record().unwrap().unwrap();let rr=replay_m21_combat(&CombatEpisodeRecord::from_json(&rec.to_json().unwrap()).unwrap(),combat_policy_capacities()).unwrap();assert_eq!(rr.final_decision,Some(r.decision().unwrap()));assert_eq!(rr.accepted_count,5);
 assert_eq!(r.decision().unwrap().combat.unwrap().selection.unwrap().selected.iter().map(|id|id.0).collect::<Vec<_>>(),vec![b,a]);
 step_pair(&mut r,&mut s,CombatAction::ConfirmSelection);let rec=s.record().unwrap().unwrap();assert_eq!(replay_m21_combat(&rec,combat_policy_capacities()).unwrap().final_decision,Some(r.decision().unwrap()));
 // Fresh zero-confirm consumes no selection and still resumes once.
 let mut r=core(supplied(&[CardId::ThirdEye;8]));let mut s=CombatTrainingSession::new(one(supplied(&[CardId::ThirdEye;8])),Partition::Test).unwrap();s.reset(Some(7),None).unwrap();let id=r.decision().unwrap().combat.unwrap().hand[0].instance.0;step_pair(&mut r,&mut s,CombatAction::PlayCard{instance:id,target:None});step_pair(&mut r,&mut s,CombatAction::ConfirmSelection);assert_eq!(replay_m21_combat(&s.record().unwrap().unwrap(),combat_policy_capacities()).unwrap().final_decision,Some(r.decision().unwrap()));
}
#[test]
fn m21_watcher_semantic_replay_preserves_meditate_and_potion_parent(){
 let start=supplied(&[CardId::DefendPurple,CardId::DefendPurple,CardId::Meditate]);let mut r=core(start.clone());let mut s=CombatTrainingSession::new(one(start),Partition::Test).unwrap();s.reset(Some(7),None).unwrap();
 let d=r.decision().unwrap();let hand=&d.combat.as_ref().unwrap().hand;let defenders=hand.iter().filter(|c|c.id==CardId::DefendPurple).map(|c|c.instance.0).collect::<Vec<_>>();let meditate=hand.iter().find(|c|c.id==CardId::Meditate).unwrap().instance.0;
 for id in defenders{step_pair(&mut r,&mut s,CombatAction::PlayCard{instance:id,target:None});}step_pair(&mut r,&mut s,CombatAction::PlayCard{instance:meditate,target:None});let sel=r.decision().unwrap().combat.unwrap().selection.unwrap();let id=sel.candidates[1].instance.0;step_pair(&mut r,&mut s,CombatAction::SelectCard{instance:id});
 let rec=s.record().unwrap().unwrap();assert_eq!(replay_m21_combat(&rec,combat_policy_capacities()).unwrap().final_decision,Some(r.decision().unwrap()));step_pair(&mut r,&mut s,CombatAction::ConfirmSelection);
 let mut start=supplied(&[CardId::DefendPurple]);if let M21WatcherCombatStart::Supplied(x)=&mut start{x.potions[0]=Some(PotionId::StancePotion);}
 let mut r=core(start.clone());let mut s=CombatTrainingSession::new(one(start),Partition::Test).unwrap();s.reset(Some(7),None).unwrap();step_pair(&mut r,&mut s,CombatAction::UsePotion{slot:0,target:None});let rec=s.record().unwrap().unwrap();assert_eq!(replay_m21_combat(&rec,combat_policy_capacities()).unwrap().final_decision,Some(r.decision().unwrap()));
 step_pair(&mut r,&mut s,CombatAction::ChooseStance{stance:spire_core::combat::Stance::Calm});let rec=s.record().unwrap().unwrap();assert_eq!(replay_m21_combat(&rec,combat_policy_capacities()).unwrap().final_decision,Some(r.decision().unwrap()));let v:serde_json::Value=serde_json::from_slice(&rec.to_json().unwrap()).unwrap();assert_eq!(v["steps"][1]["action"],serde_json::json!({"verb":"combat.choose_stance","stance":"calm"}));
}
#[test]
fn m21_watcher_projection_rejects_ambiguous_modal_owners(){
 let r=core(supplied(&[CardId::ThirdEye;8]));let mut d=r.decision().unwrap();let c=d.combat.as_mut().unwrap();c.stance_choice={let mut start=supplied(&[CardId::DefendPurple]);if let M21WatcherCombatStart::Supplied(r)=&mut start{r.potions[0]=Some(PotionId::StancePotion);}let mut r=core(start);r.step(&GameAction::Combat(CombatAction::UsePotion{slot:0,target:None})).unwrap();r.decision().unwrap().combat.unwrap().stance_choice};c.selection={let mut r=core(supplied(&[CardId::ThirdEye;8]));let instance=r.decision().unwrap().combat.unwrap().hand[0].instance.0;r.step(&GameAction::Combat(CombatAction::PlayCard{instance,target:None})).unwrap();r.decision().unwrap().combat.unwrap().selection};assert!(encode_watcher_combat(&d,&combat_policy_capacities()).is_err());
}
#[test]
fn m21_watcher_session_retains_delivery_fault_and_tokens(){
 let mut s=CombatTrainingSession::new(catalog(),Partition::Test).unwrap();let mut other=CombatTrainingSession::new(catalog(),Partition::Test).unwrap();let a=s.reset(Some(7),Some("B")).unwrap();let wrong=other.reset(Some(7),Some("B")).unwrap();let before=s.record().unwrap();assert!(matches!(s.step(wrong.token,RunActionIndex(0)),Err(RunSessionError::StaleToken)));assert_eq!(s.record().unwrap(),before);
 let b=s.step(a.token,RunActionIndex(0)).unwrap();assert!(!b.truncated);assert!(matches!(s.step(a.token,RunActionIndex(0)),Err(RunSessionError::StaleToken)));s.fail_delivery("external packaging");s.fail_delivery("later");let rec=s.record().unwrap().unwrap();assert_eq!(rec.fault.as_ref().unwrap().error.message,"external packaging");assert_eq!(rec.steps.len(),1);assert!(rec.outcome.is_none());assert!(s.reset(Some(8),None).is_err());s.close().unwrap();let r=s.record().unwrap().unwrap();assert!(r.closed);let report=replay_m21_combat(&CombatEpisodeRecord::from_json(&r.to_json().unwrap()).unwrap(),combat_policy_capacities()).unwrap();assert_eq!(report.accepted_count,1);assert_eq!(report.reported_fault,r.fault);assert!(report.final_decision_error.is_none(),"external delivery error must not become a core error");
 let legacy=CombatTrainingCatalog::from_json(br#"{"entries":[{"id":"old","encounter":"cultist","hp":50,"max_hp":80,"deck":[{"card":"defend","upgrades":0}],"relics":[],"potions":[null,null,null]}]}"#).unwrap();let mut legacy=CombatTrainingSession::new(legacy,Partition::Test).unwrap();assert!(legacy.record().unwrap().is_none());legacy.reset(Some(7),None).unwrap();assert!(legacy.record().unwrap().is_none());
}
#[test]
fn m21_watcher_strict_record_json_rejects_hidden_duplicate_and_wrong_identities(){
 let mut s=CombatTrainingSession::new(catalog(),Partition::Test).unwrap();s.reset(Some(7),Some("B")).unwrap();let record=s.record().unwrap().unwrap();let original:serde_json::Value=serde_json::from_slice(&record.to_json().unwrap()).unwrap();
 for key in ["schema_id","profile_id","rules_id","content_id","observation_schema_id","rng_id","seed_adapter_id","reward_id"]{let mut v=original.clone();v[key]=serde_json::json!("wrong");assert!(CombatEpisodeRecord::from_json(&serde_json::to_vec(&v).unwrap()).is_err(),"{key}");}
 for key in ["entry_id","selected_entry","catalog_descriptor","catalog_fingerprint","observation_fingerprint"]{let mut v=original.clone();v.as_object_mut().unwrap().remove(key);assert!(CombatEpisodeRecord::from_json(&serde_json::to_vec(&v).unwrap()).is_err(),"{key}");}
 let mut v=original.clone();v["hidden_draw"]=serde_json::json!([]);assert!(CombatEpisodeRecord::from_json(&serde_json::to_vec(&v).unwrap()).is_err());
 let bytes=String::from_utf8(record.to_json().unwrap()).unwrap();let duplicate=bytes.replacen("\"seed\":7","\"seed\":7,\"seed\":8",1);assert_ne!(bytes,duplicate);assert!(CombatEpisodeRecord::from_json(duplicate.as_bytes()).is_err());
 let duplicate=bytes.replacen("\"instance\":31","\"instance\":31,\"instance\":31",1);assert_ne!(bytes,duplicate);assert!(CombatEpisodeRecord::from_json(duplicate.as_bytes()).is_err());
 let mut bad=record.clone();bad.steps.push(RunRecordedStep{action:GameAction::Combat(CombatAction::EndTurn),reward:1,terminal:None});assert!(bad.to_json().is_err());let mut bad=record.clone();bad.outcome=Some(spire_core::combat::Outcome::Victory);assert!(bad.to_json().is_err());
}
#[test]
fn m21_watcher_capacity_fault_after_scry_keeps_accepted_prefix(){
 let start=supplied(&[CardId::ThirdEye;8]);let r=core(start.clone());let d=r.decision().unwrap();let instance=d.combat.unwrap().hand[0].instance.0;
 let mut caps=combat_policy_capacities();caps.selection=1;let mut s=CombatTrainingSession::new_m21(one(start),Partition::Test,caps.clone()).unwrap();let b=s.reset(Some(7),None).unwrap();let play=r.decision().unwrap().candidates.iter().position(|c|c.action==GameAction::Combat(CombatAction::PlayCard{instance,target:None})).unwrap();assert!(matches!(s.step(b.token,RunActionIndex(play as i64)),Err(RunSessionError::Capacity{group:"selection",..})));
 let rec=s.record().unwrap().unwrap();assert_eq!(rec.steps.len(),1);assert_eq!(rec.steps[0].reward,0);assert!(rec.fault.as_ref().unwrap().committed);assert!(rec.fault.as_ref().unwrap().gameplay_recorded);assert!(rec.outcome.is_none());let rr=replay_m21_combat(&CombatEpisodeRecord::from_json(&rec.to_json().unwrap()).unwrap(),caps).unwrap();assert_eq!(rr.accepted_count,1);let c=rr.final_decision.unwrap().combat.unwrap();assert_eq!(c.player.block,7);assert_eq!(c.player.energy,2);assert_eq!(c.selection.unwrap().candidates.len(),3);assert_eq!(rr.reported_fault,rec.fault);
}
#[test]
fn m21_watcher_replay_settles_real_victory_and_death(){
 // Fixed finite lifecycle fixtures, not the prescribed caller/scenarios.
 let mut start=supplied(&[CardId::Eruption;5]);if let M21WatcherCombatStart::Supplied(r)=&mut start{for c in &mut r.deck{c.upgrades=1;}}
 let mut r=core(start.clone());let mut s=CombatTrainingSession::new(one(start),Partition::Test).unwrap();s.reset(Some(7),None).unwrap();
 let ids=r.decision().unwrap().combat.unwrap().hand.iter().map(|c|c.instance.0).collect::<Vec<_>>();let target=r.decision().unwrap().combat.unwrap().targets[0].0;
 for instance in ids.into_iter().take(3){let step=step_pair(&mut r,&mut s,CombatAction::PlayCard{instance,target:Some(target)});assert_eq!(step.reward,0);assert!(!step.terminated);}
 step_pair(&mut r,&mut s,CombatAction::EndTurn);let instance=r.decision().unwrap().combat.unwrap().hand[0].instance.0;let t=step_pair(&mut r,&mut s,CombatAction::PlayCard{instance,target:Some(target)});assert_eq!(t.reward,1);assert!(t.terminated);assert!(!t.truncated);assert!(t.boundary.observation.action_mask.iter().all(|n|*n==0));let rec=s.record().unwrap().unwrap();assert_eq!(rec.outcome,Some(spire_core::combat::Outcome::Victory));let report=replay_m21_combat(&rec,combat_policy_capacities()).unwrap();assert_eq!(report.outcome,rec.outcome);assert_eq!(report.final_decision,Some(r.decision().unwrap()));
 let mut start=supplied(&[CardId::DefendPurple]);if let M21WatcherCombatStart::Supplied(r)=&mut start{r.hp=1;}
 let mut r=core(start.clone());let mut s=CombatTrainingSession::new(one(start),Partition::Test).unwrap();s.reset(Some(7),None).unwrap();let first=step_pair(&mut r,&mut s,CombatAction::EndTurn);assert_eq!(first.reward,0);let death=step_pair(&mut r,&mut s,CombatAction::EndTurn);assert_eq!(death.reward,-1);assert!(death.terminated);assert!(!death.truncated);let rec=s.record().unwrap().unwrap();assert_eq!(rec.outcome,Some(spire_core::combat::Outcome::Death));let rec=CombatEpisodeRecord::from_json(&rec.to_json().unwrap()).unwrap();let report=replay_m21_combat(&rec,combat_policy_capacities()).unwrap();assert_eq!(report.outcome,rec.outcome);assert_eq!(report.final_decision,Some(r.decision().unwrap()));
}
#[test]
fn m21_watcher_old_schema_guards_and_frozen_descriptors(){
 use spire_core::combat::Stance;
 let config=RunSessionConfig{character:Character::Ironclad,ascension:0,partition:Partition::Test,boss_history:M9BossHistory::AwakenedUnseen,capacities:RunCapacities::m9_default(),record_replay:true};let mut old=RunSession::new(config).unwrap();old.reset(Some(7)).unwrap();let record=old.record().unwrap().unwrap();
 for a in [CombatAction::ChooseStance{stance:Stance::Calm},CombatAction::DeselectCard{instance:1}]{let mut bad=record.clone();bad.steps.push(RunRecordedStep{action:GameAction::Combat(a),reward:0,terminal:None});assert!(bad.to_json().is_err());assert!(replay_run(&bad,RunCapacities::m9_default()).is_err());}
 let mut start=supplied(&[CardId::DefendPurple]);if let M21WatcherCombatStart::Supplied(r)=&mut start{r.potions[0]=Some(PotionId::StancePotion);}let mut r=core(start);r.step(&GameAction::Combat(CombatAction::UsePotion{slot:0,target:None})).unwrap();assert!(encode_run(&r.decision().unwrap(),None,&combat_policy_capacities()).is_err());
 let shared=watcher_combat_observation_spec(&combat_policy_capacities()).unwrap();let desc=shared.descriptor();let amount=desc["tensors"].as_array().unwrap().iter().find(|t|t["key"]=="powers").unwrap();assert_eq!(amount["low"][3],i32::MIN);assert_eq!(amount["high"][3],i32::MAX);
}
#[test]
fn m21_watcher_fault_json_requires_complete_error_receipt(){
 let mut s=CombatTrainingSession::new(catalog(),Partition::Test).unwrap();s.reset(Some(7),Some("B")).unwrap();s.fail_delivery("external packaging");let r=s.record().unwrap().unwrap();let mut v:serde_json::Value=serde_json::from_slice(&r.to_json().unwrap()).unwrap();v["fault"]["error"].as_object_mut().unwrap().remove("capacity");assert!(CombatEpisodeRecord::from_json(&serde_json::to_vec(&v).unwrap()).is_err());
}
#[test]
fn m21_watcher_catalog_normalizes_resources_and_keeps_shared_57_setup_inventory(){
 let entries=spire_core::combat::enemies::A0_ENCOUNTER_SETUPS.iter().map(|(id,_,_)|M21WatcherCombatEntry{id:id.as_str().into(),encounter:*id,start:M21WatcherCombatStart::Canonical}).collect::<Vec<_>>();assert_eq!(entries.len(),57);let all=CombatTrainingCatalog::m21(entries).unwrap();assert_eq!(all.descriptor()["entries"].as_array().unwrap().len(),57);
 let make=|cards:Vec<CardInstance>|one(M21WatcherCombatStart::Supplied(AuthoredRunStart{hp:60,max_hp:72,gold:7,deck:cards,relics:vec![RelicId::Vajra,RelicId::GoldenEye],potions:vec![None;3],relic_counters:Default::default()}));
 let cards=vec![CardInstance{id:CardId::StrikePurple,instance:8,upgrades:0,misc:0},CardInstance{id:CardId::DefendPurple,instance:31,upgrades:1,misc:0}];let a=make(cards.clone());let mut reversed=cards;reversed.reverse();let b=make(reversed);assert_eq!(a.descriptor(),b.descriptor());assert_eq!(a.fingerprint(),b.fingerprint());assert_eq!(a.descriptor()["entries"][0]["start"]["resources"]["deck"][0]["instance"],31);
 for card in [CardId::Miracle,CardId::Smite,CardId::Safety,CardId::Strike]{assert!(CombatTrainingCatalog::m21(vec![M21WatcherCombatEntry{id:"bad".into(),encounter:EncounterId::Cultist,start:supplied(&[card])}]).is_err());}
 let mut invalid=supplied(&[CardId::DefendPurple,CardId::DefendPurple]);if let M21WatcherCombatStart::Supplied(r)=&mut invalid{r.deck[1].instance=r.deck[0].instance;}assert!(CombatTrainingCatalog::m21(vec![M21WatcherCombatEntry{id:"bad".into(),encounter:EncounterId::Cultist,start:invalid}]).is_err());
}
#[test]
fn m21_watcher_replay_actual_core_fault_retains_one_issued_child_and_consumed_potion(){
 // Supplied identity-capacity diagnostic, not canonical acquisition reachability:
 // the first of two source-created children uses the final u64 identity, and
 // the shared second-child issuer faults. No fault hook or alternate engine.
 let start=M21WatcherCombatStart::Supplied(AuthoredRunStart{hp:72,max_hp:72,gold:99,deck:vec![CardInstance{instance:u64::MAX-1,id:CardId::DefendPurple,upgrades:0,misc:0}],relics:vec![],potions:vec![Some(PotionId::BottledMiracle),None,None],relic_counters:Default::default()});
 let mut core=core(start.clone());let initial=core.decision().unwrap();let index=initial.candidates.iter().position(|c|c.action==GameAction::Combat(CombatAction::UsePotion{slot:0,target:None})).unwrap();
 let mut session=CombatTrainingSession::new(one(start),Partition::Test).unwrap();let boundary=session.reset(Some(7),Some("case")).unwrap();
 let core_error=core.step(&GameAction::Combat(CombatAction::UsePotion{slot:0,target:None})).unwrap_err();assert!(matches!(core_error,spire_core::run::RunError::Faulted(_)));
 assert!(matches!(session.step(boundary.token,RunActionIndex(index as i64)),Err(RunSessionError::Core(spire_core::run::RunError::Faulted(_)))));
 let record=session.record().unwrap().unwrap();let fault=record.fault.as_ref().unwrap();assert!(fault.committed);assert!(!fault.gameplay_recorded);assert_eq!(record.outcome,None);assert!(record.steps.is_empty());assert_eq!(session.diagnostic()["accepted_steps"],1);
 let raw=record.to_json().unwrap();let restored=CombatEpisodeRecord::from_json(&raw).unwrap();let report=replay_m21_combat(&restored,combat_policy_capacities()).unwrap();assert_eq!(report.reported_fault,record.fault);assert_eq!(report.outcome,None);assert_eq!(report.accepted_count,1);
 let source=format!("{core_error:?}");assert!(source.contains("generated card identity exhausted"));assert!(source.contains("next: 1"));
 let frozen=session.diagnostic();assert!(session.reset(Some(11),Some("case")).is_err());assert_eq!(session.diagnostic(),frozen);
}
#[test]
fn m21_watcher_session_all57_initial_boundaries_use_same_encoder(){
 let entries=spire_core::combat::enemies::A0_ENCOUNTER_SETUPS.iter().map(|(id,_,_)|M21WatcherCombatEntry{id:id.as_str().into(),encounter:*id,start:M21WatcherCombatStart::Canonical}).collect();let catalog=CombatTrainingCatalog::m21(entries).unwrap();let mut session=CombatTrainingSession::new(catalog,Partition::Test).unwrap();
 for (encounter,_,_) in spire_core::combat::enemies::A0_ENCOUNTER_SETUPS{session.reset(Some(7),Some(encounter.as_str())).unwrap_or_else(|e|panic!("{}: {e:?}",encounter.as_str()));}
}
#[test]
fn m21_watcher_source_absent_magic_has_unknown_mask_without_hiding_real_cost_or_retain(){
 let mut session=CombatTrainingSession::new(catalog(),Partition::Test).unwrap();let b=session.reset(Some(7),Some("A")).unwrap();let descriptor=session.observation_spec().unwrap().descriptor();let miracle=descriptor["vocabularies"]["card"].as_array().unwrap().iter().position(|v|v=="miracle").unwrap() as i64;
 let cards=b.observation.group("combat_cards").unwrap();let width=cards.shape[1];let row=cards.values.chunks(width).position(|v|v[2]==miracle).unwrap();let offset=row*width;
 assert_eq!(cards.known[offset+10],0);assert_eq!(cards.values[offset+10],0);assert_eq!((cards.known[offset+19],cards.known[offset+20],cards.known[offset+21]),(1,1,1));assert_eq!((cards.values[offset+19],cards.values[offset+20],cards.values[offset+21]),(0,0,1));
}
