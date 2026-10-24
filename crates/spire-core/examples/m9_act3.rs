mod m9_support;
use m9_support::{M9BossHistory as History,M9Completion};

fn execute()->Result<(),String>{
    let mut seed=11_u64;let mut budget=8192_usize;let mut scenario="all".to_owned();
    let mut args=std::env::args().skip(1);
    while let Some(arg)=args.next(){
        let value=args.next().ok_or_else(||format!("missing value for {arg}"))?;
        match arg.as_str(){
            "--seed"=>seed=value.parse().map_err(|_|"seed must be u64")?,
            "--budget"=>budget=value.parse().map_err(|_|"budget must be in 0..8192")?,
            "--scenario"=>scenario=value,_=>return Err(format!("unknown argument {arg}")),
        }
    }
    if budget>8192||!matches!(scenario.as_str(),"complete"|"death"|"all"){return Err("scenario complete|death|all; budget 0..8192".into())}
    let mut reports=vec![];
    if scenario!="death"{for history in [History::AwakenedUnseen,History::TimeEaterUnseen,History::DonuDecaUnseen]{reports.push((format!("{history:?}"),m9_support::complete(seed,history,budget)?));}}
    if scenario!="complete"{reports.push(("nonattacking".into(),m9_support::death(seed,budget)?));}
    let mut unfinished=false;
    for (label,r) in reports{
        let d=&r.final_decision;
        let outcome=match r.completion{M9Completion::Act3NoKeys=>"Act3 no-key non-goal completion (NOT Heart victory)",M9Completion::Death=>"valid gameplay Death (NOT Act3 completion)",M9Completion::EmptyDeck=>"EmptyDeck loss",M9Completion::Act4Ready=>"unsupported pre-Act4 boundary (NOT completion)",M9Completion::Unfinished=>"unfinished caller budget (NOT gameplay outcome)"};
        println!("seed{seed} {label}: {outcome}; actions{} act{} floor{} HP{}/{} reward{} terminal{:?}",r.trace.len(),d.act,d.floor,d.player.hp,d.player.max_hp,d.reward_total,d.terminal);
        println!("  actual Act3 combat settlements: {:?}",r.combats.iter().filter(|(act,_)|*act==3).collect::<Vec<_>>());
        unfinished|=matches!(r.completion,M9Completion::Unfinished|M9Completion::Act4Ready);
    }
    if unfinished{Err("unfinished or unsupported boundary; not a loss/truncation".into())}else{Ok(())}
}
fn main(){if let Err(e)=execute(){eprintln!("M9: {e}");std::process::exit(1)}}
