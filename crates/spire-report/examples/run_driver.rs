//! Example-only use of the unchanged accepted producer's public caller.
use std::io::Write;
use spire_report::{ReportWriter,ReportError,RunRecorder};
use spire_core::run::{PublicDecision,map::PublicMap,RoomViewKind,RunTerminal};
use serde_json::json;
#[path="../../spire-core/examples/m6_support/mod.rs"]
#[allow(dead_code)] pub mod source;
pub fn capture_source<W:Write>(seed:u64,budget:usize,death:bool,writer:&mut ReportWriter<W>)->Result<(PublicDecision,Option<PublicMap>),ReportError> {
    if budget>4096 {return Err(ReportError::Invalid("budget outside 0..4096".into()));}
    let report=if death {source::death(seed,budget)}else{source::prefix(seed,vec!["Automaton","Collector"],budget)}.map_err(ReportError::Invalid)?;
    let mut run=source::start(seed,vec!["Automaton","Collector"]).map_err(ReportError::Invalid)?;
    let mut recorder=RunRecorder::new(writer);recorder.start(&run)?;
    for step in &report.trace {
        let t=recorder.step(&mut run,&step.action).map_err(|e|ReportError::Invalid(format!("native recording stopped (never retry automatically): {e:?}")))?;
        if t.reward!=step.reward||run.decision().ok().as_ref()!=Some(&step.decision)||run.public_map()!=step.map {
            return Err(ReportError::Invalid("native reproduction differs from accepted public source".into()));
        }
    }
    let decision=run.decision().map_err(|_|ReportError::Invalid("native final decision unavailable".into()))?;
    let map=run.public_map();
    if decision!=report.final_decision||map!=report.final_map {return Err(ReportError::Invalid("native final source mismatch".into()));}
    let prefix=decision.room.as_ref().is_some_and(|r|r.kind==RoomViewKind::Act2PrefixComplete);
    let status=if decision.terminal.is_some()||prefix {"complete"}else{"unfinished"};
    let outcome=decision.terminal.map(|t|match t {RunTerminal::Death=>"death",RunTerminal::HeartVictory=>"heart_victory",RunTerminal::EmptyDeck=>"empty_deck",RunTerminal::Act3NoKeys=>"act3_no_keys"});
    recorder.finish(&run,json!({"status":status,"reason":if status=="unfinished" {Some("caller action budget exhausted")}else{None}}),
        json!({"outcome":outcome,"boundary":if prefix {Some("act2_prefix_complete")}else{None}}))?;
    Ok((decision,map))
}
// Cargo discovers helpers in examples/ as binaries too.
#[allow(dead_code)] fn main() {}
