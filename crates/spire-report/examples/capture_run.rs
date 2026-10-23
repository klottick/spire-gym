mod run_driver;
use serde_json::json;
use spire_report::{ReportWriter,create_trace};
fn run()->Result<(),String> {
    let mut seed=None;let mut budget=None;let mut scenario=None;let mut trace=None;
    let mut args=std::env::args().skip(1);
    while let Some(flag)=args.next() {match flag.as_str() {
        "--help"=>{println!("capture_run --seed <u64> --budget <0..4096> --scenario prefix|death --trace NEW.jsonl");return Ok(())},
        "--seed" if seed.is_none()=>seed=Some(args.next().ok_or("missing seed")?.parse::<u64>().map_err(|_|"invalid seed")?),
        "--budget" if budget.is_none()=>budget=Some(args.next().ok_or("missing budget")?.parse::<usize>().map_err(|_|"invalid budget")?),
        "--scenario" if scenario.is_none()=>scenario=Some(args.next().ok_or("missing scenario")?),
        "--trace" if trace.is_none()=>trace=Some(std::path::PathBuf::from(args.next().ok_or("missing trace")?)),
        _=>return Err(format!("unknown or duplicate flag {flag}")),
    }}
    let seed=seed.ok_or("missing --seed")?;let budget=budget.ok_or("missing --budget")?;
    if budget>4096 {return Err("budget outside 0..4096".into());}
    let death=match scenario.as_deref(){Some("prefix")=>false,Some("death")=>true,_=>return Err("--scenario must be prefix or death".into())};
    let trace=trace.ok_or("missing --trace")?;
    let file=create_trace(&trace).map_err(|e|e.to_string())?;
    let mut writer=ReportWriter::new(file,json!({"record":"header","schemaId":"spire-report-v1","schemaVersion":1,
        "source":{"kind":"run_prefix","profile":"accepted M6 Ironclad A0 captured-content prefix through floor26","producerRevision":"4874fb3+run-viewer-pr44","contentId":spire_core::combat::CONTENT_ID,"evidenceGrade":"source-grounded; not original-runtime validated"},
        "run":{"character":"ironclad","ascension":0,"seed":{"encoding":"native u64 little-endian root prefix (not StS seed code)","value":seed.to_string()},
            "caller":{"kind":"unchanged native M6 public caller; separate from-start reproduction","budget":budget,"scenario":if death {"death"}else{"prefix"},"act2BossIcon":"Champ"}}})).map_err(|e|e.to_string())?;
    let(d,_)=run_driver::capture_source(seed,budget,death,&mut writer).map_err(|e|e.to_string())?;
    if d.terminal.is_none()&&!d.room.as_ref().is_some_and(|r|r.kind==spire_core::run::RoomViewKind::Act2PrefixComplete) {
        return Err(format!("unfinished capture: budget {budget}, trace retained at {}",trace.display()));
    }
    println!("Recorded native M6 {} at floor{} to {} (not full-run completion)",if d.terminal==Some(spire_core::run::RunTerminal::Death){"Death"}else{"prefix boundary"},d.floor,trace.display());Ok(())
}
fn main()->std::process::ExitCode {match run(){Ok(())=>std::process::ExitCode::SUCCESS,Err(e)=>{eprintln!("{e}");std::process::ExitCode::FAILURE}}}
