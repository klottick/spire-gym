#[path="common/native.rs"] mod native;
use spire_report::*;
use spire_core::{combat::*,run::*};
use serde_json::json;
use std::io::{self,Write};

#[test]
fn run_chosen_offer() {
    let mut run=native::m4::run(7);
    for (action,_,_) in native::route() {
        let before=run.decision().unwrap();
        let price=if let GameAction::Choose(id)=action {before.room.as_ref().filter(|r|r.kind==RoomViewKind::Shop).and_then(|r|r.choices.iter().find(|c|c.id==id)).and_then(|c|c.price)}else{None};
        if let Some(price)=price {
            let mut writer=ReportWriter::new(Vec::new(),native::header()).unwrap();
            let mut recorder=RunRecorder::new(&mut writer);recorder.start(&run).unwrap();recorder.step(&mut run,&action).unwrap();
            let after=run.decision().unwrap();assert_eq!(before.player.gold-after.player.gold,price);
            let report=read_report(&writer.into_inner()[..],64*1024*1024).unwrap();let a=&report["frames"][1]["action"];
            assert_eq!(a["preActionOffer"]["price"],price);assert!(a["preActionOffer"]["cardId"].is_string());
            assert_eq!(a["optionKey"],report["frames"][1]["beforeScreen"]["options"].as_array().unwrap().iter().find(|o|o["data"]["price"]==price).unwrap()["key"]);
            return;
        }run.step(&action).unwrap();
    }panic!("actual offered shop purchase required")
}
#[test]
fn run_invariance_and_selection_parent() {
    let mut plain=native::m4::run(7);let mut run=plain.clone();
    let mut writer=ReportWriter::new(Vec::new(),native::header()).unwrap();let mut recorder=RunRecorder::new(&mut writer);recorder.start(&run).unwrap();
    let mut selected=false;
    for (action,expected,_) in native::route() {
        let t=recorder.step(&mut run,&action).unwrap();assert_eq!(t,plain.step(&action).unwrap());assert_eq!(t,expected);assert_eq!(run,plain);
        selected|=matches!(action,GameAction::Select(_));
    }
    recorder.finish(&run,json!({"status":"complete","reason":"M4 test prefix"}),json!({"outcome":null,"boundary":"m4_prefix"})).unwrap();
    let report=read_report(&writer.into_inner()[..],64*1024*1024).unwrap();
    assert!(selected);
    for f in report["frames"].as_array().unwrap().iter().filter(|f|f["action"]["type"]=="select") {assert!(f["action"]["parentActionId"].is_string());}
}
#[test]
fn run_lethal_boundary() {
    let mut run=native::lethal();let mut writer=ReportWriter::new(Vec::new(),native::header()).unwrap();
    let mut recorder=RunRecorder::new(&mut writer);recorder.start(&run).unwrap();
    for slot in 0..3 {recorder.step(&mut run,&GameAction::Combat(CombatAction::UsePotion{slot,target:Some(TargetId(1))})).unwrap();if run.decision().unwrap().combat.is_none(){break;}}
    let report=read_report(&writer.into_inner()[..],64*1024*1024).unwrap();let frame=report["frames"].as_array().unwrap().last().unwrap();
    assert!(frame["state"]["combat"].is_null());assert_eq!(frame["combatFinal"]["enemies"][0]["hp"],0);
    assert_eq!(frame["combatFinal"]["player"]["block"],0);
    assert_eq!(frame["combatFinal"]["player"]["energy"],3);
    assert_eq!(frame["state"]["player"]["hp"],46);
    assert_eq!(frame["effects"].as_array().unwrap().iter().filter(|e|e["kind"]=="heal"&&e["amount"]==6).count(),1);
}
#[test]
fn run_recorder_write_failure_after_step() {
    struct Sink {bytes:Vec<u8>,fail:std::rc::Rc<std::cell::Cell<bool>>}
    impl Write for Sink {fn write(&mut self,b:&[u8])->io::Result<usize>{if self.fail.get(){Err(io::Error::other("sink full"))}else{self.bytes.extend(b);Ok(b.len())}}fn flush(&mut self)->io::Result<()>{Ok(())}}
    let fail=std::rc::Rc::new(std::cell::Cell::new(false));
    let mut writer=ReportWriter::new(Sink{bytes:vec![],fail:fail.clone()},native::header()).unwrap();
    let mut run=native::m4::run(7);let mut control=run.clone();let action=run.decision().unwrap().candidates[0].action.clone();
    let mut recorder=RunRecorder::new(&mut writer);recorder.start(&run).unwrap();fail.set(true);
    assert!(matches!(recorder.step(&mut run,&action),Err(CaptureError::WriteAfterCall(_))));
    control.step(&action).unwrap();assert_eq!(run,control);
    let recovery=recorder.take_recovery().expect("accepted native action evidence survives delivery failure");
    assert!(recovery.transition.is_some());assert_eq!(recovery.after,Some(run.decision().unwrap()));
    assert_eq!(recovery.frame.as_ref().unwrap()["id"],"1");
    assert!(recorder.finish(&run,json!({"status":"complete","reason":null}),json!({"outcome":null,"boundary":null})).is_err());
}

#[test]
fn run_lethal_write_failure_retains_terminal_evidence() {
    struct Sink {bytes:Vec<u8>,fail:std::rc::Rc<std::cell::Cell<bool>>}
    impl Write for Sink {fn write(&mut self,b:&[u8])->io::Result<usize>{if self.fail.get(){Err(io::Error::other("sink full"))}else{self.bytes.extend(b);Ok(b.len())}}fn flush(&mut self)->io::Result<()>{Ok(())}}
    let fail=std::rc::Rc::new(std::cell::Cell::new(false));
    let mut writer=ReportWriter::new(Sink{bytes:vec![],fail:fail.clone()},native::header()).unwrap();
    let mut run=native::lethal();let mut recorder=RunRecorder::new(&mut writer);recorder.start(&run).unwrap();
    for slot in 0..2 {recorder.step(&mut run,&GameAction::Combat(CombatAction::UsePotion{slot,target:Some(TargetId(1))})).unwrap();}
    assert!(recorder.take_recovery().is_none(),"successful writes retain no full history");
    let mut control=run.clone();let action=GameAction::Combat(CombatAction::UsePotion{slot:2,target:Some(TargetId(1))});
    fail.set(true);assert!(matches!(recorder.step(&mut run,&action),Err(CaptureError::WriteAfterCall(_))));control.step(&action).unwrap();assert_eq!(run,control);
    let evidence=recorder.take_recovery().expect("retain lethal action after sink failure");
    assert_eq!(evidence.action["type"],"combat_potion");assert!(evidence.after.as_ref().unwrap().combat.is_none());
    assert!(evidence.transition.is_some());
    let terminal=evidence.boundaries.iter().find(|b|b.kind==CombatBoundaryKind::Terminal).unwrap();
    let PublicCombatSnapshot::Native(public)=&terminal.public else {panic!("native terminal public snapshot")};
    assert_eq!((public.enemies[0].hp,public.player.hp),(0,46));
    assert!(evidence.effects.iter().any(|e|matches!(e.kind,ResolvedEffectKind::Damage{incoming:20,..})));
    assert!(evidence.effects.iter().any(|e|matches!(e.kind,ResolvedEffectKind::Heal{amount:6,..})));
    assert_eq!(evidence.frame.as_ref().unwrap()["combatFinal"]["enemies"][0]["hp"],0);
    assert!(recorder.take_recovery().is_none());
    assert!(matches!(recorder.step(&mut run,&action),Err(CaptureError::WriteBeforeCall(_))));assert_eq!(run,control);
    assert!(recorder.finish(&run,json!({"status":"complete","reason":null}),json!({"outcome":null,"boundary":null})).is_err());
    let bytes=writer.into_inner().bytes;let report=read_report(&bytes[..],64*1024*1024).unwrap();
    assert_eq!(report["capture"]["status"],"interrupted");assert_eq!(report["frames"].as_array().unwrap().len(),3,"prior complete records remain recoverable");
}

#[test]
fn run_recorder_rejects_foreign_owner() {
    let run=native::m4::run(7);let mut foreign=run.clone();let before=foreign.clone();
    let mut writer=ReportWriter::new(Vec::new(),native::header()).unwrap();let mut recorder=RunRecorder::new(&mut writer);recorder.start(&run).unwrap();
    let action=foreign.decision().unwrap().candidates[0].action.clone();
    assert!(matches!(recorder.step(&mut foreign,&action),Err(CaptureError::WriteBeforeCall(_))));assert_eq!(foreign,before);
}

#[test]
fn run_rest_records_primitive_heal() {
    let mut run=native::m4::run(7);
    for (action,_,_) in native::route() {
        let d=run.decision().unwrap();
        let rest=match &action {GameAction::Choose(id)=>d.room.as_ref().is_some_and(|r|r.choices.iter().any(|c|c.id==*id&&matches!(c.kind,RoomChoiceKind::Rest{..}))),_=>false};
        if rest {
            let mut writer=ReportWriter::new(Vec::new(),native::header()).unwrap();let mut recorder=RunRecorder::new(&mut writer);recorder.start(&run).unwrap();recorder.step(&mut run,&action).unwrap();
            let report=read_report(&writer.into_inner()[..],64*1024*1024).unwrap();
            let heal=report["frames"][1]["effects"].as_array().unwrap().iter().find(|e|e["kind"]=="heal").expect("actual noncombat healing commit");
            assert_eq!(heal["before"],d.player.hp);assert_eq!(heal["after"],run.decision().unwrap().player.hp);return;
        }run.step(&action).unwrap();
    }panic!("real rest action required")
}

#[test]
fn run_combat_selection_parent() {
    use spire_core::content::{CardId,Character,EncounterId};
    let mut run=Run::from_authored_combat(RunConfig{character:Character::Ironclad,ascension:0},AuthoredRunStart {
        hp:40,max_hp:80,gold:99,deck:[CardId::BurningPact,CardId::Strike,CardId::Defend].into_iter().enumerate().map(|(i,id)|spire_core::run::CardInstance{instance:i as u64+1,id,upgrades:0}).collect(),
        relics:vec![],potions:vec![None;3],relic_counters:Default::default()},RunRng::new([7;32],rng::Partition::Test,0)).unwrap();
    run.enter_combat(EncounterId::Cultist).unwrap();
    let mut writer=ReportWriter::new(Vec::new(),native::header()).unwrap();let mut recorder=RunRecorder::new(&mut writer);recorder.start(&run).unwrap();
    recorder.step(&mut run,&GameAction::Combat(CombatAction::PlayCard{instance:1,target:None})).unwrap();
    let pick=run.decision().unwrap().candidates[0].action.clone();recorder.step(&mut run,&pick).unwrap();
    let report=read_report(&writer.into_inner()[..],64*1024*1024).unwrap();let frames=report["frames"].as_array().unwrap();
    assert_eq!(frames[2]["action"]["parentActionId"],"1");assert_eq!(frames[2]["action"]["card"],frames[1]["action"]["card"]);
    assert_eq!(frames[2]["state"]["player"]["energy"],2);
    assert_eq!(frames[1]["action"]["card"]["cardId"],"burning_pact");
}

#[test]
fn report_formats_combat_confirmation_as_distinct_action(){
    use spire_core::content::{CardId,Character,EncounterId};
    let mut run=Run::from_authored_combat(RunConfig{character:Character::Ironclad,ascension:0},AuthoredRunStart {
        hp:40,max_hp:80,gold:99,deck:[CardId::BurningPact,CardId::Strike,CardId::Defend].into_iter().enumerate().map(|(i,id)|spire_core::run::CardInstance{instance:i as u64+1,id,upgrades:0}).collect(),
        relics:vec![],potions:vec![None;3],relic_counters:Default::default()},RunRng::new([7;32],rng::Partition::Test,0)).unwrap();
    run.enter_combat(EncounterId::Cultist).unwrap();let mut d=run.decision().unwrap();
    let action=GameAction::Combat(CombatAction::ConfirmSelection);
    d.candidates.push(Candidate{id:ChoiceId(999),action:action.clone(),label:"Confirm selection",object:None,neow_offer:None});
    let event=describe_run_action(&d,&action,"run-1").unwrap();
    assert_eq!(event["type"],"confirm_selection");
    assert_eq!(event["optionKey"],"combat/confirm_selection");
    assert_eq!(event["description"],"Confirm selection");
}

#[test]
fn run_relic_acquisition_keeps_actual_healing() {
    let mut run=native::m4::run(7);
    for (action,_,_) in native::route() {
        let d=run.decision().unwrap();let strawberry=match &action {GameAction::Choose(id)=>d.room.as_ref().is_some_and(|r|r.choices.iter().any(|c|c.id==*id&&c.kind==RoomChoiceKind::Relic(spire_core::content::RelicId::Strawberry))),_=>false};
        if strawberry {
            let mut writer=ReportWriter::new(Vec::new(),native::header()).unwrap();let mut recorder=RunRecorder::new(&mut writer);recorder.start(&run).unwrap();recorder.step(&mut run,&action).unwrap();
            let report=read_report(&writer.into_inner()[..],64*1024*1024).unwrap();
            assert!(report["frames"][1]["effects"].as_array().unwrap().iter().any(|e|e["kind"]=="heal"&&e["amount"]==7));return;
        }run.step(&action).unwrap();
    }panic!("actual Strawberry acquisition required")
}
