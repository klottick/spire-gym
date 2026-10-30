//! Once-only default-off M23 supplied diagnostics: the conditional fixed-16 Act2 boss
//! series and the six plan Task3 played fixtures. Supplied resources, never normal-start proof.
mod m23_support;
#[cfg(feature="m23-diagnostics")]
mod run_series{
    use super::m23_support::*;
    use spire_core::run::{GameAction,PublicDecision,Run,RunError,map::PublicMap,diagnostics::{self,M23DiagnosticBoss as B,M23PlayedFixture as F}};
    pub const FIXTURE_BUDGET:usize=128;
    fn script(f:F)->fixtures::Script{match f{F::FlightThreeByrds=>fixtures::Script::Flight,F::PricesSnecko=>fixtures::Script::Prices,F::PowerScryChosenAndByrds=>fixtures::Script::PowerScry,F::WoundsSlavers|F::WoundsBookOfStabbing=>fixtures::Script::Wounds,F::StasisBronzeAutomaton=>fixtures::Script::Stasis}}
    fn fixture_policy<const I:usize>(d:&PublicDecision,_:Option<&PublicMap>,caller:&Caller)->Result<GameAction,String>{
        let f=F::ALL[I];let order=f.deck().iter().map(|(id,_)|*id).collect::<Vec<_>>();
        fixtures::script_action(script(f),&order,d,&fixture_state(caller,d))
    }
    const POLICIES:[fn(&PublicDecision,Option<&PublicMap>,&Caller)->Result<GameAction,String>;6]=[fixture_policy::<0>,fixture_policy::<1>,fixture_policy::<2>,fixture_policy::<3>,fixture_policy::<4>,fixture_policy::<5>];
    /// One real Move enters the supplied room; excluded from the policy budget and retained separately.
    fn bootstrap(run:&mut Run,initial:&mut serde_json::Value)->Result<(),String>{
        initial["diagnostic_checkpoint_before_bootstrap"]=serde_json::json!(format!("{:?}",diagnostics::m22_diagnostic_checkpoint(run).map_err(|e|format!("{e:?}"))?));
        let action=run.decision().map_err(|e|format!("{e:?}"))?.candidates.into_iter().find(|c|matches!(c.action,GameAction::Move(_))).ok_or("missing fixed supplied entry")?.action;
        let entry=run.step_observed(&action);let successful=entry.result.is_ok();let accepted=successful||matches!(&entry.result,Err(RunError::Faulted(_)));
        initial["supplied_entry"]=serde_json::json!({"scope":"supplied encounter bootstrap; excluded from policy budget/count","accepted":accepted,"successful":successful,"action":semantic_actions::action_value(&action),"post_diagnostic_checkpoint_debug":format!("{:?}",diagnostics::m22_diagnostic_checkpoint(run)),"transition_debug":format!("{:?}",entry.result),"effects_debug":format!("{:?}",entry.effects),"boundaries_debug":format!("{:?}",entry.boundaries),"post_public_debug":format!("{:?}",run.decision()),"map_debug":format!("{:?}",run.public_map())});
        entry.result.map(|_|()).map_err(|e|format!("fixed supplied entry: {e:?}"))
    }
    pub fn execute(series:&str,prepare_only:bool)->Result<(),String>{
        let directory=std::env::var_os("M23_TRACE_DIR").map(std::path::PathBuf::from).ok_or("M23_TRACE_DIR required")?;
        let freeze=std::env::var("M23_FREEZE_SHA256").map_err(|_|"M23_FREEZE_SHA256 required")?;
        if freeze.len()!=64||!freeze.bytes().all(|b|b.is_ascii_hexdigit()){return Err("invalid freeze hash".into())}
        std::fs::create_dir_all(&directory).map_err(|e|e.to_string())?;
        let mut cases:Vec<(String,Result<Run,RunError>,usize,fn(&PublicDecision,Option<&PublicMap>,&Caller)->Result<GameAction,String>,serde_json::Value)>=Vec::new();
        match series{
            "supplied-bosses"=>for b in B::ALL{cases.push((format!("seed-7-supplied-fixed16-{:?}-budget-32768.attempt.json",b),diagnostics::m23_supplied_boss(b),ACTION_LIMIT_MAX,policy,serde_json::json!({"supplied_boss":b.encounter().as_str()})));},
            "played-fixtures"=>for (i,f) in F::ALL.into_iter().enumerate(){cases.push((format!("seed-7-played-fixture-{}-budget-{FIXTURE_BUDGET}.attempt.json",f.name()),diagnostics::m23_played_fixture(f),FIXTURE_BUDGET,POLICIES[i],serde_json::json!({"fixture":f.name(),"encounter":f.encounter().as_str(),"deck":format!("{:?}",f.deck()),"relics":format!("{:?}",f.relics()),"potions":format!("{:?}",f.potions())})));},
            _=>return Err("unsupported M23 diagnostic series".into()),
        }
        if prepare_only{
            // Zero-action initialization records for the freeze; no bootstrap or policy.
            for (name,created,..) in cases{let run=created.map_err(|e|format!("{e:?}"))?;let mut v=initial_value(&run)?;v["diagnostic_checkpoint"]=serde_json::json!(format!("{:?}",diagnostics::m22_diagnostic_checkpoint(&run)));write_new(&directory.join(name.replace(".attempt.json",".initial.json")),&v)?;}
            return Ok(())
        }
        for (name,..) in &cases{if directory.join(name).exists(){return Err(format!("refusing existing attempt {name}"))}}
        let mut failures=Vec::new();
        for (name,created,budget,chooser,case) in cases{
            let path=directory.join(&name);let mut caller=Caller::new_m23(budget)?;
            let mut run=created.map_err(|e|format!("{e:?}"))?;let mut initial=initial_value(&run)?;
            let result=match bootstrap(&mut run,&mut initial){Ok(())=>shared::rollout_m23(&mut run,&mut caller,chooser),Err(e)=>Err(e)};
            persist(&path,7,case,budget,series,&freeze,&initial,&caller,Some(&run),&result)?;
            println!("{series} {name} result={result:?} policy_successful_count={} endpoint={:?} last_combat={:?}",caller.accepted(),run.decision().map(|d|(d.act,d.floor,d.terminal)),run.decision().ok().and_then(|d|d.last_combat));
            let victory=run.decision().ok().and_then(|d|d.last_combat).is_some_and(|c|c.outcome==spire_core::combat::Outcome::Victory);
            if let Err(e)=result{if !(series=="played-fixtures"&&fixtures::settled_victory(&e,victory)){failures.push(format!("{name}: {e}"))}}
        }
        if failures.is_empty(){Ok(())}else{Err(failures.join("\n"))}
    }
}
#[cfg(feature="m23-diagnostics")]
fn execute()->Result<(),String>{
    let args=std::env::args().skip(1).collect::<Vec<_>>();
    match args.as_slice(){[flag,series] if flag=="--series"=>run_series::execute(series,false),[flag,series,p] if flag=="--series"&&p=="--prepare-only"=>run_series::execute(series,true),_=>Err("usage: --series supplied-bosses|played-fixtures [--prepare-only]".into())}
}
#[cfg(not(feature="m23-diagnostics"))]
fn execute()->Result<(),String>{Err("fixed supplied diagnostics require default-off m23-diagnostics".into())}
fn main(){if let Err(e)=execute(){eprintln!("{e}");std::process::exit(1)}}
