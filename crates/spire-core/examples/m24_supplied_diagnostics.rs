//! Once-only default-off M24 supplied diagnostics (design §11): the supplied Act3 traversal
//! (3), the supplied Act3 bosses (6) and the played interaction fixtures (9). Supplied
//! resources and constructed predecessors, never normal-start proof.
mod m24_support;
#[cfg(feature="m24-diagnostics")]
mod run_series{
    use super::m24_support::*;
    use spire_core::run::{GameAction,PublicDecision,Run,RunError,map::PublicMap,diagnostics::{self,M24DiagnosticBoss as B,M24PlayedFixture as F}};
    pub const FIXTURE_BUDGET:usize=128;
    type Chooser=fn(&PublicDecision,Option<&PublicMap>,&Caller)->Result<GameAction,String>;
    /// One real Move enters the supplied room; excluded from the policy budget and retained separately.
    fn bootstrap(run:&mut Run,initial:&mut serde_json::Value)->Result<(),String>{
        initial["diagnostic_checkpoint_before_bootstrap"]=serde_json::json!(format!("{:?}",diagnostics::m22_diagnostic_checkpoint(run).map_err(|e|format!("{e:?}"))?));
        let action=run.decision().map_err(|e|format!("{e:?}"))?.candidates.into_iter().find(|c|matches!(c.action,GameAction::Move(_))).ok_or("missing fixed supplied entry")?.action;
        let entry=run.step_observed(&action);let successful=entry.result.is_ok();let accepted=successful||matches!(&entry.result,Err(RunError::Faulted(_)));
        initial["supplied_entry"]=serde_json::json!({"scope":"supplied encounter bootstrap; excluded from policy budget/count","accepted":accepted,"successful":successful,"action":semantic_actions::action_value(&action),"post_diagnostic_checkpoint_debug":format!("{:?}",diagnostics::m22_diagnostic_checkpoint(run)),"transition_debug":format!("{:?}",entry.result),"effects_debug":format!("{:?}",entry.effects),"boundaries_debug":format!("{:?}",entry.boundaries),"post_public_debug":format!("{:?}",run.decision()),"map_debug":format!("{:?}",run.public_map())});
        entry.result.map(|_|()).map_err(|e|format!("fixed supplied entry: {e:?}"))
    }
    pub fn execute(series:&str,prepare_only:bool)->Result<(),String>{
        let directory=std::env::var_os("M24_TRACE_DIR").map(std::path::PathBuf::from).ok_or("M24_TRACE_DIR required")?;
        let freeze=std::env::var("M24_FREEZE_SHA256").map_err(|_|"M24_FREEZE_SHA256 required")?;
        if freeze.len()!=64||!freeze.bytes().all(|b|b.is_ascii_hexdigit()){return Err("invalid freeze hash".into())}
        std::fs::create_dir_all(&directory).map_err(|e|e.to_string())?;
        // (name, start, seed, budget, chooser, needs bootstrap Move, case)
        let mut cases:Vec<(String,Result<Run,RunError>,u64,usize,Chooser,bool,serde_json::Value)>=Vec::new();
        match series{
            "supplied-traversal"=>for b in B::ALL{cases.push((format!("seed-7-supplied-traversal-{}-budget-32768.attempt.json",b.name()),diagnostics::m24_supplied_traversal(b),7,ACTION_LIMIT_MAX,policy,false,serde_json::json!({"supplied_traversal":b.encounter().as_str(),"act2_boss_source":b.traversal_act2().encounter().as_str(),"loadout":"v2"})));},
            "supplied-bosses"=>for (seed,keys) in [(7u64,[false;3]),(11,[true;3])]{for b in B::ALL{cases.push((format!("seed-{seed}-supplied-boss-{}-keys-{}-budget-32768.attempt.json",b.name(),if keys==[true;3]{"all"}else{"none"}),diagnostics::m24_supplied_boss_at(b,keys,seed),seed,ACTION_LIMIT_MAX,policy,true,serde_json::json!({"supplied_boss":b.encounter().as_str(),"keys":keys,"loadout":"v2"})));}},
            "played-fixtures"=>for f in F::ALL{cases.push((format!("seed-7-played-fixture-{}-budget-{FIXTURE_BUDGET}.attempt.json",f.name()),diagnostics::m24_played_fixture(f),7,FIXTURE_BUDGET,fixture_policy,true,serde_json::json!({"fixture":f.name(),"encounter":f.encounter().as_str(),"deck":format!("{:?}",f.deck()),"relics":format!("{:?}",f.relics())})));},
            _=>return Err("unsupported M24 diagnostic series".into()),
        }
        if prepare_only{
            // Zero-action initialization records for the freeze; no bootstrap or policy.
            for (name,created,..) in cases{let run=created.map_err(|e|format!("{e:?}"))?;let mut v=initial_value(&run)?;v["diagnostic_checkpoint"]=serde_json::json!(format!("{:?}",diagnostics::m22_diagnostic_checkpoint(&run)));write_new(&directory.join(name.replace(".attempt.json",".initial.json")),&v)?;}
            return Ok(())
        }
        for (name,..) in &cases{if directory.join(name).exists(){return Err(format!("refusing existing attempt {name}"))}}
        let mut failures=Vec::new();
        for (name,created,seed,budget,chooser,needs_bootstrap,case) in cases{
            let path=directory.join(&name);let mut caller=Caller::new_m24(budget)?;
            let mut run=created.map_err(|e|format!("{e:?}"))?;let mut initial=initial_value(&run)?;
            let entered=if needs_bootstrap{bootstrap(&mut run,&mut initial)}else{Ok(())};
            let result=match entered{Ok(())=>shared::rollout_m24(&mut run,&mut caller,chooser),Err(e)=>Err(e)};
            persist(&path,seed,case,budget,series,&freeze,&initial,&caller,Some(&run),&result)?;
            println!("{series} {name} result={result:?} policy_successful_count={} endpoint={:?} last_combat={:?}",caller.accepted(),run.decision().map(|d|(d.act,d.floor,d.terminal)),run.decision().ok().and_then(|d|d.last_combat));
            let victory=run.decision().ok().and_then(|d|d.last_combat).is_some_and(|c|c.outcome==spire_core::combat::Outcome::Victory);
            if let Err(e)=result{if !(series=="played-fixtures"&&settled_victory(&e,victory)){failures.push(format!("{name}: {e}"))}}
        }
        if failures.is_empty(){Ok(())}else{Err(failures.join("\n"))}
    }
}
#[cfg(feature="m24-diagnostics")]
fn execute()->Result<(),String>{
    let args=std::env::args().skip(1).collect::<Vec<_>>();
    match args.as_slice(){[flag,series] if flag=="--series"=>run_series::execute(series,false),[flag,series,p] if flag=="--series"&&p=="--prepare-only"=>run_series::execute(series,true),_=>Err("usage: --series supplied-traversal|supplied-bosses|played-fixtures [--prepare-only]".into())}
}
#[cfg(not(feature="m24-diagnostics"))]
fn execute()->Result<(),String>{Err("fixed supplied diagnostics require default-off m24-diagnostics".into())}
fn main(){if let Err(e)=execute(){eprintln!("{e}");std::process::exit(1)}}
