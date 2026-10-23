use spire_report::{OutputReservation,ReportError,export_html};
use std::{collections::BTreeMap,fs::File,io::{BufReader,Write},path::PathBuf};

const USAGE:&str="spire-report --trace INPUT.jsonl --output REPORT.html [--assets MANIFEST.json]\nPackages display records only. Does not simulate, restore, or complete a game.";

fn run()->Result<(),ReportError>{
    let args:Vec<_>=std::env::args_os().skip(1).collect();
    if args.len()==1 && args[0]=="--help" {println!("{USAGE}");return Ok(());}
    let mut options=BTreeMap::new();
    let mut args=args.into_iter();
    while let Some(flag)=args.next(){
        let flag=flag.to_str().ok_or_else(||ReportError::Invalid("flag must be UTF-8".into()))?.to_owned();
        if !matches!(flag.as_str(),"--trace"|"--output"|"--assets") || options.contains_key(&flag){
            return Err(ReportError::Invalid(format!("unknown or duplicate option: {flag}")));
        }
        let value=args.next().ok_or_else(||ReportError::Invalid(format!("missing value for {flag}")))?;
        if value.to_str().is_some_and(|text|text.starts_with("--")){return Err(ReportError::Invalid(format!("missing value for {flag}")));}
        options.insert(flag,PathBuf::from(value));
    }
    let trace=options.get("--trace").ok_or_else(||ReportError::Invalid("missing --trace".into()))?;
    let output=options.get("--output").ok_or_else(||ReportError::Invalid("missing --output".into()))?;
    let mut html=Vec::new();
    // Validate input, artwork and embedded resources before reserving any output.
    export_html(BufReader::new(File::open(trace)?),options.get("--assets").map(PathBuf::as_path),&mut html)?;
    let mut reservation=OutputReservation::new(output)?;
    reservation.writer().write_all(&html)?;
    reservation.publish()?;
    println!("Wrote offline report: {}",output.display());
    Ok(())
}

fn main(){
    if let Err(error)=run(){eprintln!("{error}\n{USAGE}");std::process::exit(1);}
}
