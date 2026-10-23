//! Actual supplied-entry native episode; public numeric caller, then shared replay.
use spire_env::*;
use spire_report::{ReportWriter,capture_combat,create_trace};
use serde_json::json;
use std::{sync::Arc,path::PathBuf};

fn run()->Result<(),String> {
    let mut seed=None;let mut trace=None;let mut args=std::env::args().skip(1);
    while let Some(arg)=args.next() {match arg.as_str() {
        "--seed" if seed.is_none()=>seed=Some(args.next().ok_or("missing seed")?.parse::<u64>().map_err(|_|"invalid u64 seed")?),
        "--trace" if trace.is_none()=>trace=Some(PathBuf::from(args.next().ok_or("missing trace")?)),
        "--help"=>{println!("capture_combat --seed <u64> --trace NEW.jsonl");return Ok(())},
        _=>return Err(format!("unknown or duplicate flag {arg}")),
    }}
    let seed=seed.ok_or("missing --seed")?;let trace=trace.ok_or("missing --trace")?;
    let out=create_trace(&trace).map_err(|e|e.to_string())?;
    let mut writer=ReportWriter::new(out,json!({"record":"header","schemaId":"spire-report-v1","schemaVersion":1,
        "source":{"kind":"combat_episode","profile":"starter-cultist-a0-v1 supplied entry","producerRevision":"5b1cd19+viewer-uncommitted","contentId":spire_core::combat::CONTENT_ID,"evidenceGrade":"source-grounded; not original-runtime validated"},
        "run":{"character":"ironclad","ascension":0,"seed":{"encoding":"native env unsigned integer decimal","value":seed.to_string()},"caller":{"kind":"finite public-mask caller; isolated from-start replay"}}})).map_err(|e|e.to_string())?;
    let pool=CombatPool::new(Arc::new(ScenarioCatalog::starter()),PoolConfig {record_replay:true,..PoolConfig::default()}).map_err(|e|format!("{e:?}"))?;
    let mut tokens=pool.reset_batch(&[0],&[ResetRequest {seed:Some(IntegerSeed::from_be_bytes(&seed.to_be_bytes()).map_err(|e|format!("{e:?}"))?),scenario_id:None}]).map_err(|e|format!("{e:?}"))?.decisions.tokens;
    for step in 0..128 {
        let rows=pool.read_rows(&[0]).map_err(|e|format!("{e:?}"))?;
        let TensorValues::I8(mask)=&rows.observations.tensors.iter().find(|t|t.key=="action_mask").ok_or("missing public mask")?.values else {return Err("invalid public mask".into())};
        // Include a real End Turn, then spend the first actually offered action.
        let index=if step==0 {0}else{mask[1..32].iter().position(|&n|n==1).map_or(0,|i|i as i64+1)};
        let result=pool.step_batch(&[index],&tokens).map_err(|e|format!("{e:?}"))?;
        tokens=result.current.tokens;
        if result.terminated[0] {
            let record=pool.reproduction(0).map_err(|e|format!("{e:?}"))?.ok_or("missing native record")?;
            capture_combat(&record,&mut writer).map_err(|e|e.to_string())?;
            println!("Recorded {} native actions to {} (combat only)",record.actions.len(),trace.display());return Ok(())
        }
    }
    let record=pool.reproduction(0).map_err(|e|format!("{e:?}"))?.ok_or("missing native record")?;
    capture_combat(&record,&mut writer).map_err(|e|e.to_string())?;
    Err("unfinished combat: 128-action budget exhausted".into())
}
fn main()->std::process::ExitCode {match run(){Ok(())=>std::process::ExitCode::SUCCESS,Err(e)=>{eprintln!("{e}");std::process::ExitCode::FAILURE}}}
