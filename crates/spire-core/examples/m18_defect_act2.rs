mod m18_support;
fn main(){
    let options=match m18_support::parse_args(std::env::args().skip(1)){Ok(o)=>o,Err(e)=>{eprintln!("{e}");std::process::exit(1)}};
    let mut failed=false;let mut unfinished=false;
    for scenario in options.scenarios{
        let mut run=m18_support::make_run(options.seed,scenario).expect("validated fixed config");let mut caller=if options.nonattacking{m18_support::Caller::nonattacking(options.limit)}else{m18_support::Caller::new(options.limit)};
        let result=m18_support::rollout(&mut run,&mut caller,scenario);
        // Always print partial evidence, including on an error, before aggregation.
        println!("seed={} scenario={} result={result:?} accepted={} final={:?} map={:?}",options.seed,scenario.label(),caller.accepted(),run.decision(),run.public_map());
        for (i,step) in caller.trace.iter().enumerate(){println!("step={i} {step:?}");}
        if let Some(dir)=std::env::var_os("M18_EVIDENCE_DIR"){
            let path=std::path::PathBuf::from(dir);std::fs::create_dir_all(&path).expect("retain evidence directory");
            let receipt=m18_support::action_receipt(options.seed,scenario,&caller,&result);
            std::fs::write(path.join(format!("{}-{}.json",options.seed,scenario.label())),serde_json::to_vec_pretty(&receipt).unwrap()).expect("retain full public receipt");
        }
        failed|=result.is_err();unfinished|=matches!(result,Ok(m18_support::CallerOutcome::BudgetExhausted));
    }
    if failed{std::process::exit(1)}if unfinished{std::process::exit(2)}
}
