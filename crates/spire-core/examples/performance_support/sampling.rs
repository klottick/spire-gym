use super::*;
use serde_json::{json, Value};
use std::{collections::BTreeSet, hint::black_box, time::Instant};

#[derive(Clone, Copy, Debug)]
pub struct Sampling { pub warmup: usize, pub samples: usize, pub iterations: usize }
impl Default for Sampling { fn default()->Self { Self {warmup:1,samples:7,iterations:10} } }
impl Sampling {
    pub fn validate(self)->Result<(),String> {
        if self.warmup>100 || !(1..=100).contains(&self.samples) || !(1..=1000).contains(&self.iterations) {
            return Err("limits: warmup 0..100, samples 1..100, iterations 1..1000".into());
        } Ok(())
    }
}

pub struct Options { pub sampling:Sampling, pub filter:Option<String>, pub list:bool }
pub fn parse_args(args:&[String])->Result<Options,String> {
    let mut options=Options {sampling:Sampling::default(),filter:None,list:false};
    let mut seen=BTreeSet::new();let mut iter=args.iter();
    while let Some(flag)=iter.next() {
        if !seen.insert(flag) {return Err(format!("duplicate option {flag}"));}
        if flag=="--list" {options.list=true;continue;}
        if !["--samples","--iterations","--warmup","--case"].contains(&flag.as_str()) {return Err(format!("unknown option {flag}"));}
        let value=iter.next().ok_or_else(||format!("missing value for {flag}"))?;
        if flag=="--case" {options.filter=Some(value.clone());continue;}
        let n=value.parse::<usize>().map_err(|_|format!("invalid integer for {flag}"))?;
        match flag.as_str() {"--samples"=>options.sampling.samples=n,"--iterations"=>options.sampling.iterations=n,"--warmup"=>options.sampling.warmup=n,_=>unreachable!()}
    }
    options.sampling.validate()?;
    Ok(options)
}

pub fn measure(case:&PreparedCase,boundary:Boundary,options:Sampling)->Result<Value,String> {
    options.validate()?;
    let preparation=Instant::now();
    let endpoint=preflight(case)?;
    let mut preparation_ns=preparation.elapsed().as_nanos();
    let mut samples=Vec::with_capacity(options.samples);
    for sample_index in 0..options.warmup+options.samples {
        let setup=Instant::now();
        let mut states=(0..options.iterations).map(|_|fresh(case)).collect::<Result<Vec<_>,_>>()?;
        // Allocate before the timer and keep whole states alive until after it.
        let mut totals=Vec::with_capacity(states.len());
        preparation_ns+=setup.elapsed().as_nanos();
        let started=Instant::now();
        for state in &mut states {totals.push(black_box(state.replay(black_box(case),boundary)?));}
        let elapsed_ns=started.elapsed().as_nanos();
        for (state,total) in states.iter().zip(&totals) {
            if state.endpoint(*total)?!=endpoint {return Err("measured episode endpoint/reward changed".into());}
        }
        if sample_index>=options.warmup {
            samples.push(json!({"elapsed_ns":u64::try_from(elapsed_ns).map_err(|_|"sample timer overflow")?,
                "episodes":options.iterations,"accepted_actions":totals.iter().map(|t|t.accepted).sum::<usize>()}));
        }
    }
    let c=&case.contract;
    Ok(json!({"id":case.id.as_str(),"version":c["version"],"contract_sha256":canonical_hash(c),
        "boundary":boundary.as_str(),"profile":c["profile"],"character":c["character"],"ascension":c["ascension"],"seed":c["seed"],
        "batch_size":1,"workers":1,"input_sha256":canonical_hash(&case.input),"actions_sha256":case.actions_sha256(),
        "accepted_per_episode":case.actions.len(),"expected_contract_version":c["expected_contract_version"],
        "endpoint":endpoint,"preflight_passed":true,"samples":samples,
        "preparation_ns":u64::try_from(preparation_ns).map_err(|_|"preparation timer overflow")?}))
}

pub fn run_suite(options:Sampling,case_filter:Option<&str>)->Result<Value,String> {
    options.validate()?;
    let ids:Vec<_>=CaseId::ALL.into_iter().filter(|id|case_filter.is_none_or(|f|id.as_str()==f)).collect();
    if ids.is_empty() {return Err("unknown or planned case: no executable workload".into());}
    let mut cases=vec![];
    for id in ids { let case=prepare(id)?; for boundary in [Boundary::Native,Boundary::Public] {cases.push(measure(&case,boundary,options)?);} }
    let catalog=catalog();
    Ok(json!({"schema":"spire-gym-native-samples-v1","catalog_sha256":canonical_hash(&catalog),"catalog":catalog,
        "sampling":{"warmup":options.warmup,"samples":options.samples,"iterations":options.iterations},"cases":cases}))
}
