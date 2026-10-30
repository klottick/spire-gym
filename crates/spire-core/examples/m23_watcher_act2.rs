//! Fixed M23 canonical cases and controls over the shared rollout; one receipt per attempt, never overwritten.
mod m23_support;
use m23_support::*;
fn main(){if let Err(e)=execute(){eprintln!("{e}");std::process::exit(1)}}
fn execute()->Result<(),String>{
    // Input errors are rejected here, before any Run construction or receipt.
    let o=parse_args(std::env::args().skip(1))?;
    let directory=std::env::var_os("M23_TRACE_DIR").map(std::path::PathBuf::from).ok_or("M23_TRACE_DIR is required for durable receipts")?;
    let freeze=std::env::var("M23_FREEZE_SHA256").map_err(|_|"M23_FREEZE_SHA256 is required")?;
    if freeze.len()!=64||!freeze.bytes().all(|b|b.is_ascii_hexdigit()){return Err("invalid M23 freeze hash".into())}
    std::fs::create_dir_all(&directory).map_err(|e|e.to_string())?;
    let label=if o.nonattacking{"nonattacking-control"}else if o.budget<ACTION_LIMIT_MAX{"budget-control"}else{"normal"};
    for scenario in &o.scenarios{let path=attempt_path(&directory,o.seed,*scenario,label,o.budget);if path.exists(){return Err(format!("refusing existing M23 attempt {}",path.display()))}}
    let mut failures=Vec::new();
    for scenario in o.scenarios{
        let boss1=act1(o.seed,scenario,o.nonattacking)?;
        let path=attempt_path(&directory,o.seed,scenario,label,o.budget);let mut caller=Caller::new_m23(o.budget)?;caller.nonattacking=o.nonattacking;
        let mut created=make_run(o.seed,scenario,o.nonattacking);
        if let Err(error)=&created{let result=Err(error.clone());persist(&path,o.seed,case(boss1,scenario),o.budget,label,&freeze,&serde_json::Value::Null,&caller,None,&result)?;failures.push(error.clone());continue}
        let run=created.as_mut().unwrap();let initial=match initial_value(run){Ok(v)=>v,Err(error)=>{let result=Err(error.clone());persist(&path,o.seed,case(boss1,scenario),o.budget,label,&freeze,&serde_json::Value::Null,&caller,Some(run),&result)?;failures.push(error);continue}};
        if o.prepare_only{write_new(&directory.join(format!("seed-{}-{}-{}-initial.json",o.seed,boss1.as_str(),scenario.name())),&initial)?;continue}
        let result=shared::rollout_m23(run,&mut caller,policy);
        persist(&path,o.seed,case(boss1,scenario),o.budget,label,&freeze,&initial,&caller,Some(run),&result)?;
        println!("seed={} act1={} act2={} result={result:?} accepted_actions={} endpoint={:?}",o.seed,boss1.as_str(),scenario.name(),caller.accepted(),run.decision().map(|d|(d.act,d.floor,d.terminal)));
        if let Err(e)=result{failures.push(e)}
    }
    if failures.is_empty(){Ok(())}else{Err(failures.join("\n"))}
}
