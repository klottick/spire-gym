mod m17_support;
fn main(){
    let options=match m17_support::parse_args(std::env::args().skip(1)){Ok(o)=>o,Err(e)=>{eprintln!("{e}");std::process::exit(1)}};
    let mut failed=false;
    for scenario in options.scenarios{
        let mut run=m17_support::make_run(options.seed,scenario).expect("validated fixed config");let mut caller=m17_support::Caller::new(options.limit);
        let result=m17_support::rollout(&mut run,&mut caller,scenario);
        // Always print partial evidence, including on an error, before aggregation.
        println!("seed={} scenario={} result={result:?} accepted={} final={:?} map={:?}",options.seed,scenario.label(),caller.accepted(),run.decision(),run.public_map());
        for (i,step) in caller.trace.iter().enumerate(){println!("step={i} {step:?}");}
        if let Some(dir)=std::env::var_os("M17_EVIDENCE_DIR"){
            let path=std::path::PathBuf::from(dir);std::fs::create_dir_all(&path).expect("retain evidence directory");
            let receipt=m17_support::action_receipt(options.seed,scenario,&caller,&result);
            std::fs::write(path.join(format!("{}-{}.json",options.seed,scenario.label())),serde_json::to_vec_pretty(&receipt).unwrap()).expect("retain full public receipt");
        }
        failed|=result.is_err();
    }
    if failed{std::process::exit(1)}
}
