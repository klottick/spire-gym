mod m22_support;
use m22_support::*;
fn main(){if let Err(e)=execute(){eprintln!("{e}");std::process::exit(1)}}
fn execute()->Result<(),String>{
    let o=parse_args(std::env::args().skip(1))?;
    let directory=std::env::var_os("M22_TRACE_DIR").map(std::path::PathBuf::from).ok_or("M22_TRACE_DIR is required for durable receipts")?;
    let freeze=std::env::var("M22_FREEZE_SHA256").map_err(|_|"M22_FREEZE_SHA256 is required")?;
    if freeze.len()!=64||!freeze.bytes().all(|b|b.is_ascii_hexdigit()){return Err("invalid M22 freeze hash".into())}
    std::fs::create_dir_all(&directory).map_err(|e|e.to_string())?;
    let label=if o.nonattacking{"nonattacking-control"}else if o.budget<32768{"budget-control"}else{"normal"};
    for scenario in &o.scenarios{let path=attempt_path(&directory,o.seed,*scenario,label,o.budget);if path.exists(){return Err(format!("refusing existing M22 attempt {}",path.display()))}}
    let mut failures=Vec::new();
    for scenario in o.scenarios{
        let path=attempt_path(&directory,o.seed,scenario,label,o.budget);let mut caller=Caller::new_m22(o.budget)?;caller.nonattacking=o.nonattacking;
        let mut created=make_run(o.seed,scenario);
        if let Err(error)=&created{let result=Err(error.clone());persist(&path,o.seed,scenario,o.budget,label,&freeze,&serde_json::Value::Null,&caller,None,&result)?;failures.push(error.clone());continue}
        let mut run=created.as_mut().unwrap();let initial=match initial_value(&run){Ok(v)=>v,Err(error)=>{let result=Err(error.clone());persist(&path,o.seed,scenario,o.budget,label,&freeze,&serde_json::Value::Null,&caller,Some(&run),&result)?;failures.push(error);continue}};
        if o.prepare_only{write_new(&directory.join(format!("seed-{}-{}-initial.json",o.seed,scenario.name())),&initial)?;continue}
        let result=shared::rollout_m22(&mut run,&mut caller,policy);
        persist(&path,o.seed,scenario,o.budget,label,&freeze,&initial,&caller,Some(&run),&result)?;
        println!("seed={} scenario={} result={result:?} accepted_actions={} endpoint={:?}",o.seed,scenario.name(),caller.accepted(),run.decision().map(|d|(d.act,d.floor,d.terminal)));
        if let Err(e)=result{failures.push(e)}
    }
    if failures.is_empty(){Ok(())}else{Err(failures.join("\n"))}
}
