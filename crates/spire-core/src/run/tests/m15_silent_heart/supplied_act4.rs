//! Supplied Act4 diagnostics only; no normal-start or public reset contract.
use super::frozen_m15 as frozen;
use crate::run::*;
use crate::content::{CardId, RelicId, PotionId};

#[derive(Clone, Copy, Debug)]
enum Loadout { Silent, Searing }

const RELICS: [RelicId;9] = [RelicId::RingOfTheSnake,RelicId::Mango,RelicId::SneckoSkull,RelicId::PaperKrane,RelicId::ToughBandages,RelicId::BagOfPreparation,RelicId::Anchor,RelicId::OddlySmoothStone,RelicId::SlaversCollar];
const SILENT_DECK: [CardId;18] = [CardId::Backflip,CardId::Backflip,CardId::Backflip,CardId::Deflect,CardId::Deflect,CardId::Deflect,CardId::Footwork,CardId::Footwork,CardId::NoxiousFumes,CardId::NoxiousFumes,CardId::DeadlyPoison,CardId::DeadlyPoison,CardId::DeadlyPoison,CardId::Catalyst,CardId::Catalyst,CardId::Catalyst,CardId::Neutralize,CardId::Survivor];

// Character-neutral strong deck definition. This fixture executes Silent only;
// it adds no normal acquisition/upgrade admission or public reset API.
fn supplied_deck(loadout: Loadout) -> Result<Vec<CardInstance>,String> {
    let ids=match loadout {Loadout::Silent=>SILENT_DECK.to_vec(),Loadout::Searing=>vec![CardId::SearingBlow;10]};
    ids.into_iter().enumerate().map(|(i,id)| {
        let instance=i as u64+1;
        let mut live=crate::combat::permanent_live_card(id,crate::combat::CardInstanceId(instance),0);
        let count=match loadout {Loadout::Silent=>1,Loadout::Searing=>15};
        for _ in 0..count {crate::combat::cards::ironclad::rules::upgrade(&mut live).map_err(|e|format!("supplied actual upgrade: {e:?}"))?;}
        Ok(CardInstance{ misc:0,instance,id,upgrades:live.upgrades})
    }).collect()
}

fn prepare(loadout: Loadout) -> Result<Run, String> {
    use crate::{combat::Outcome,content::EncounterId};
    use crate::run::rooms::{act3_end::{Act3EndFrame,Act3EndPhase},view};
    let mut run=frozen::make_run(7,frozen::FIXED_CASES[0].1)?;
    // All privileged resource/provenance assignments are initialization, before
    // the supplied floor50 boundary. No state patch occurs after it.
    run.state.deck=supplied_deck(loadout)?;
    run.state.next_instance=100; // Same future allocation cursor in both fixtures.
    for id in RELICS.into_iter().skip(1) {
        crate::items::acquire_relic(&mut run.state,id).map_err(|e|format!("supplied relic {id:?}: {e:?}"))?;
    }
    if matches!(loadout,Loadout::Searing) {run.state.hp=250;run.state.max_hp=250;}
    run.state.gold=200;
    run.state.potions=vec![Some(PotionId::WeakPotion),Some(PotionId::FirePotion),Some(PotionId::StrengthPotion)];
    run.state.keys=[true;3];run.state.act4_enabled=true;
    run.state.act=3;run.state.floor=50;run.state.room="SuppliedAct3Source";
    // Synthetic prior Victory and supplied keys are declared fixture input,
    // never a played/earned prior win. Only actual Act4 fights follow this source.
    let node=NodeId(0xF150_0050);
    let source=CombatRecord{visit:0,encounter:EncounterId::AwakenedOne,outcome:Outcome::Victory};
    run.state.combat_progress=CombatProgress{entered:1,settled:1};
    run.state.last_combat=Some(source.clone());
    run.state.history=RunHistory{rooms:vec![VisitedRoom{act:3,floor:50,label:"SuppliedAct3Source"}],events:vec![CommittedEvent::CombatFinished(source.clone())]};
    run.state.reward_ledger=Default::default();
    run.state.map=Some(RunMap{act:3,boss_icon:Some(EncounterId::AwakenedOne),current:Some(node),nodes:vec![MapNode{id:node,floor:50,label:"Boss",edges:vec![],x:0,y:15,kind:MapNodeKind::Boss,hidden_kind:None,visited:true,revealed:true,encounter:Some(EncounterId::AwakenedOne),burning:false}],..RunMap::default()});
    let mut next=run.next_choice;
    let screen=view(&mut next,RoomViewKind::Act3Ending,vec![(RoomChoiceKind::Act3Continue{stage:4},None)]).map_err(|e|format!("supplied door: {e:?}"))?;
    let choice=screen.choices[0].id;
    let room=run.state.room_loop.as_mut().ok_or("supplied room owner absent")?;
    room.act1=None;room.act2=None;room.act3=None;
    room.act3_end=Some(Act3EndFrame{node,record:source,phase:Act3EndPhase::Dialog(4),accepted:None});
    room.screen=screen;room.m10_elite_relic_cursor=0;
    run.next_choice=next;run.candidates.clear();
    run.step(&GameAction::Choose(choice)).map_err(|e|format!("shared supplied Act4 transition: {e:?}"))?;
    Ok(run)
}

// Breaks caught: omitted shared Act4 initialization, upgrade clamp/fresh-kind
// damage reset, physical-ID collision, changed fallback RNG, or widened admission.
#[test]
fn m15_supplied_act4_fixture_preserves_upgrade_and_profile_boundaries() {
    let first=prepare(Loadout::Silent).unwrap();
    let strong=prepare(Loadout::Searing).unwrap();
    for run in [&first,&strong] {
        let d=run.decision().unwrap();
        assert_eq!((d.act,d.floor),(4,50));
        assert_eq!(d.content_profile,Some(RoomScenario::M15SilentHeart));
        assert!(d.combat.is_none() && d.terminal.is_none());
        assert_eq!(d.player.gold,200);assert_eq!(d.player.keys,[true;3]);
        assert_eq!(d.player.relics,vec![RelicId::RingOfTheSnake,RelicId::Mango,RelicId::SneckoSkull,RelicId::PaperKrane,RelicId::ToughBandages,RelicId::BagOfPreparation,RelicId::Anchor,RelicId::OddlySmoothStone,RelicId::SlaversCollar]);
        assert_eq!(d.player.potions,vec![Some(PotionId::WeakPotion),Some(PotionId::FirePotion),Some(PotionId::StrengthPotion)]);
        assert_eq!(run.public_map().unwrap().nodes.iter().map(|n|n.floor).collect::<Vec<_>>(),vec![51,52,53,54]);
        assert_eq!(run.state.room_loop.as_ref().unwrap().m10_elite_relic_cursor,0);
        assert_eq!(run.state.next_instance,100);
        assert!(run.state.pending.is_none() && run.state.selected_neow.is_none());
        assert_eq!(run.state.reward_ledger.total,0,"supplied prior history earns no earlier rewards");
        assert_eq!(run.state.deck.iter().map(|c|c.instance).collect::<std::collections::BTreeSet<_>>().len(),run.state.deck.len());
    }
    assert_eq!((first.state.hp,first.state.max_hp),(84,84));
    assert_eq!((strong.state.hp,strong.state.max_hp),(250,250));
    assert_eq!(first.state.rng,strong.state.rng);
    assert_eq!(first.state.relic_counters,strong.state.relic_counters);
    assert_eq!(first.state.deck.iter().map(|c|c.id).collect::<Vec<_>>(),vec![CardId::Backflip,CardId::Backflip,CardId::Backflip,CardId::Deflect,CardId::Deflect,CardId::Deflect,CardId::Footwork,CardId::Footwork,CardId::NoxiousFumes,CardId::NoxiousFumes,CardId::DeadlyPoison,CardId::DeadlyPoison,CardId::DeadlyPoison,CardId::Catalyst,CardId::Catalyst,CardId::Catalyst,CardId::Neutralize,CardId::Survivor]);
    assert!(first.state.deck.iter().all(|c|c.upgrades==1));
    assert_eq!(strong.state.deck.len(),10);
    for card in &strong.state.deck {
        assert_eq!((card.id,card.upgrades),(CardId::SearingBlow,15));
        let live=crate::combat::permanent_live_card(card.id,crate::combat::CardInstanceId(card.instance),card.upgrades);
        assert_eq!(crate::combat::cards::ironclad::rules::public_values(&live).unwrap().base_damage,Some(177));
    }
    let mut normal=frozen::make_run(7,frozen::FIXED_CASES[0].1).unwrap();let before=normal.clone();
    assert_eq!(crate::run::mutations::obtain_card(&mut normal.state,CardId::SearingBlow,0),Err(RunError::InvalidAction));
    assert_eq!(normal,before);
    if let Some(directory)=std::env::var_os("SPIRE_M15_SUPPLIED_INPUT_EXPORT") {
        let directory=std::path::PathBuf::from(directory);std::fs::create_dir_all(&directory).unwrap();
        write_json_new(&directory.join("silent.initial.json"),&initial_snapshot(&first,Loadout::Silent)).unwrap();
        write_json_new(&directory.join("searing.initial.json"),&initial_snapshot(&strong,Loadout::Searing)).unwrap();
    }
}


fn initial_snapshot(run:&Run,loadout:Loadout)->serde_json::Value {
    serde_json::json!({
        "schema":"m15-supplied-act4-initial-v1","lane":"supplied-only; not canonical normal progression",
        "loadout":format!("{loadout:?}"),"seed":7,"caller_revision":frozen::CALLER_POLICY_REVISION,"budget":8192,
        "synthetic_prior_source":"AwakenedOne Victory visit0; all keys supplied, never played/earned; shared stage4 door transition performed during initialization",
        "decision_debug":format!("{:#?}",run.decision()),"map_debug":format!("{:#?}",run.public_map()),
        "rng_debug":format!("{:#?}",run.state.rng),"run_debug":format!("{run:#?}"),
        "deck":run.state.deck.iter().map(|c|serde_json::json!({"instance":c.instance,"id":format!("{:?}",c.id),"upgrades":c.upgrades})).collect::<Vec<_>>(),
        "hp":run.state.hp,"max_hp":run.state.max_hp,"gold":run.state.gold,
        "relics":run.state.relics.iter().map(|r|format!("{r:?}")).collect::<Vec<_>>(),
        "potions":run.state.potions.iter().map(|p|p.map(|p|format!("{p:?}"))).collect::<Vec<_>>(),
        "relic_counters_debug":format!("{:?}",run.state.relic_counters),"next_instance":run.state.next_instance,"next_choice":run.next_choice,
        "normal_completion":"UNPROVEN; this supplied input replaces no original normal evidence"
    })
}

fn write_json_new(path:&std::path::Path,value:&serde_json::Value)->Result<(),String> {
    use std::io::Write;
    let mut file=std::fs::OpenOptions::new().write(true).create_new(true).open(path).map_err(|e|format!("create {}: {e}",path.display()))?;
    let bytes=serde_json::to_vec_pretty(value).map_err(|e|e.to_string())?;
    file.write_all(&bytes).and_then(|_|file.write_all(b"\n")).and_then(|_|file.sync_all()).map_err(|e|format!("write {}: {e}",path.display()))
}

fn run_one(mut run:Run,directory:&std::path::Path)->Result<(Result<frozen::M15CallerOutcome,String>,serde_json::Value),String> {
    use crate::content::EnemyId;
    let mut caller=frozen::Caller::new_m15(8192)?;
    let result=frozen::rollout_m15(&mut run,&mut caller,frozen::key_action);
    frozen::persist_attempt(directory,7,frozen::FIXED_CASES[0].1,8192,&result,&caller,Some(&run))?;
    let initial_combat=|floor|caller.trace.iter().find_map(|step|if step.decision.floor==floor{step.decision.combat.as_ref()}else{None});
    let elite=initial_combat(53);let heart=initial_combat(54);
    let elite_full=elite.map(|view|view.enemies.len()==2 && view.enemies.iter().any(|e|e.enemy_id==EnemyId::SpireShield && e.hp==110 && e.max_hp==110) && view.enemies.iter().any(|e|e.enemy_id==EnemyId::SpireSpear && e.hp==160 && e.max_hp==160));
    let heart_full=heart.map(|view|view.enemies.len()==1 && view.enemies[0].enemy_id==EnemyId::CorruptHeart && view.enemies[0].hp==750 && view.enemies[0].max_hp==750);
    let records=caller.trace.iter().flat_map(|step|&step.transition.committed_events).filter_map(|e|if let CommittedEvent::CombatFinished(record)=e{Some(format!("{record:?}"))}else{None}).collect::<Vec<_>>();
    let decision=run.decision().map_err(|e|format!("endpoint decision: {e:?}"));
    let genuine_death=matches!(&result,Ok(frozen::M15CallerOutcome::Death)) && decision.as_ref().is_ok_and(|d|d.terminal==Some(RunTerminal::Death)) && caller.failed_action_for_diagnostics().is_none();
    let source_error=if elite_full==Some(false)||heart_full==Some(false){Some("reached encounter did not begin with its declared full-HP native setup".to_owned())}else{None};
    let summary=serde_json::json!({"result":result.as_ref().map(|v|format!("{v:?}")).map_err(|e|e.clone()),"accepted":caller.accepted(),"genuine_death":genuine_death,"source_error":source_error,"shield_spear_initial_full_hp":elite_full,"heart_initial_full_hp":heart_full,"actual_act4_combat_records":records,"endpoint_debug":format!("{decision:#?}"),"last_transition_debug":caller.trace.last().map(|s|format!("{:?}",s.transition)),"failed_action_debug":format!("{:?}",caller.failed_action_for_diagnostics()),"normal_completion":"UNPROVEN"});
    write_json_new(&directory.join("supplied-result.json"),&summary)?;
    eprintln!("M15 SUPPLIED result={result:?} accepted={} genuine_death={genuine_death} elite_full={elite_full:?} heart_full={heart_full:?}",caller.accepted());
    Ok((result,summary))
}

fn outcome_problem(result:&Result<frozen::M15CallerOutcome,String>,summary:&serde_json::Value)->Option<String> {
    if !summary["source_error"].is_null(){return Some(summary["source_error"].to_string())}
    match result {
        Ok(frozen::M15CallerOutcome::HeartVictory|frozen::M15CallerOutcome::Death)=>None,
        _=>Some(format!("supplied diagnostic stopped with non-gameplay-terminal/error result {result:?}; no fallback or retry authorized")),
    }
}

#[test]
#[ignore = "Requires separately frozen supplied inputs and explicit manager release; never normal evidence"]
fn m15_supplied_act4_runtime_first_conditional_sequence()->Result<(),String> {
    let directory=std::path::PathBuf::from(std::env::var_os("SPIRE_M15_SUPPLIED_OUTPUT_DIR").ok_or("missing supplied output dir")?);
    let inputs=std::path::PathBuf::from(std::env::var_os("SPIRE_M15_SUPPLIED_INPUT_DIR").ok_or("missing frozen supplied input dir")?);
    if directory.exists(){return Err("supplied output directory already exists; refusing repeat".into())}
    // BOTH definitions are prepared and checked against pre-outcome frozen
    // snapshots before the first caller action. The second remains untouched.
    let first=prepare(Loadout::Silent)?;let fallback=prepare(Loadout::Searing)?;
    for (name,run,kind) in [("silent",&first,Loadout::Silent),("searing",&fallback,Loadout::Searing)] {
        let expected:serde_json::Value=serde_json::from_slice(&std::fs::read(inputs.join(format!("{name}.initial.json"))).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
        if initial_snapshot(run,kind)!=expected{return Err(format!("{name} supplied initial context differs from pre-outcome freeze"))}
    }
    if first.state.rng!=fallback.state.rng{return Err("fallback initial RNG differs".into())}
    std::fs::create_dir_all(&directory).map_err(|e|e.to_string())?;
    let first_dir=directory.join("first-silent");std::fs::create_dir(&first_dir).map_err(|e|e.to_string())?;
    let (first_result,first_summary)=run_one(first,&first_dir)?;
    let trigger=first_summary["genuine_death"]==true && outcome_problem(&first_result,&first_summary).is_none();
    write_json_new(&directory.join("conditional-decision.json"),&serde_json::json!({"fallback_triggered":trigger,"condition":"Only genuine first Death, no operational/source error or budget cutoff","first_result":first_summary}))?;
    let mut problem=outcome_problem(&first_result,&first_summary);
    let fallback_summary=if trigger {
        let fallback_dir=directory.join("fallback-searing");std::fs::create_dir(&fallback_dir).map_err(|e|e.to_string())?;
        let (result,summary)=run_one(fallback,&fallback_dir)?;problem=outcome_problem(&result,&summary);Some(summary)
    }else{None};
    write_json_new(&directory.join("sequence-result.json"),&serde_json::json!({"schema":"m15-supplied-act4-runtime-first-sequence-v1","first":first_summary,"fallback_triggered":trigger,"fallback":fallback_summary,"problem":problem,"normal_completion":"UNPROVEN; all original normal losses retained","qualification":"Supplied gameplay diagnostics; genuine Death is valid, no win requirement or policy tuning"}))?;
    if let Some(error)=problem{return Err(error)}
    Ok(())
}
