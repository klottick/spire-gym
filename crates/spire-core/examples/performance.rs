//! Opt-in performance measurements, not an ordinary correctness-test timer.
mod performance_support;
use performance_support::*;

fn run()->Result<(),String> {
    let options=parse_args(&std::env::args().skip(1).collect::<Vec<_>>())?;
    let output=if options.list {catalog()} else {
        if cfg!(debug_assertions) {return Err("timing requires a release build; --list is available in debug".into());}
        run_suite(options.sampling,options.filter.as_deref())?
    };
    println!("{}",serde_json::to_string(&output).map_err(|e|e.to_string())?);
    Ok(())
}

fn main() {
    if let Err(error)=run() { eprintln!("performance: {error}");std::process::exit(2); }
}
