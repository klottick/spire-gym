//! Integration trace exporter, not a strategy implementation or a game resolver.
#[path="../../spire-core/examples/m9_support/mod.rs"] mod support;
use spire_core::{combat::rng::Partition,content::Character};
use spire_env::*;
use serde_json::{Value,json};
use std::collections::BTreeMap;

fn high_water(o:&RunObservation,maximum:&mut BTreeMap<String,usize>){
    for group in &o.groups{if let Some(rows)=&group.occupied{
        let count=if group.key=="potions"{group.known.iter().filter(|v|**v==1).count()}else{rows.iter().filter(|v|**v==1).count()};
        let value=maximum.entry(group.key.into()).or_default();*value=(*value).max(count);
    }}
}
fn final_columns(o:&RunObservation)->Value{
    let mut columns=serde_json::Map::new();
    for group in &o.groups{
        columns.insert(group.key.into(),json!(group.values));columns.insert(format!("{}_known",group.key),json!(group.known));
        if let Some(rows)=&group.occupied{columns.insert(format!("{}_occupied",group.key),json!(rows));}
    }
    columns.insert("action_mask".into(),json!(o.action_mask));Value::Object(columns)
}
fn execute()->Result<Value,String>{
    let contexts=[("awakened-unseen",M9BossHistory::AwakenedUnseen,support::M9BossHistory::AwakenedUnseen),("time-eater-unseen",M9BossHistory::TimeEaterUnseen,support::M9BossHistory::TimeEaterUnseen),("donu-deca-unseen",M9BossHistory::DonuDecaUnseen,support::M9BossHistory::DonuDecaUnseen)];
    let mut cases=vec![];
    for seed in [7,11]{for (name,history,example_history) in contexts{
        // Reuse the frozen public-only caller. Do not develop/tune strategy here.
        let trajectory=support::complete(seed,example_history,8192)?;let mut core=support::start(seed,example_history)?;
        let caps=RunCapacities::m9_default();let config=RunSessionConfig{character:Character::Ironclad,ascension:0,partition:Partition::Test,boss_history:history,capacities:caps.clone(),record_replay:true};
        let mut session=RunSession::new(config).map_err(|e|format!("{e:?}"))?;let mut boundary=session.reset(Some(seed)).map_err(|e|format!("{e:?}"))?;
        let mut indices=vec![];let mut maximum=BTreeMap::new();high_water(&boundary.observation,&mut maximum);
        for step in &trajectory.trace{
            let d=core.decision().map_err(|e|format!("{e:?}"))?;let index=d.candidates.iter().position(|c|c.action==step.action).ok_or("reference action not offered")?;
            let result=session.step(boundary.token,RunActionIndex(index as i64)).map_err(|e|format!("{e:?}"))?;
            if (result.reward,result.boundary.terminal)!=(step.reward,step.decision.terminal){return Err("core/session reward or outcome mismatch".into())}
            core.step(&step.action).map_err(|e|format!("{e:?}"))?;
            if core.decision().map_err(|e|format!("{e:?}"))?!=step.decision||core.public_map()!=step.map{return Err("core replay boundary mismatch".into())}
            let expected=encode_run(&step.decision,step.map.as_ref(),&caps).map_err(|e|format!("{e:?}"))?;
            if result.boundary.observation!=expected{return Err("core/session numeric mismatch".into())}
            boundary=result.boundary;high_water(&boundary.observation,&mut maximum);indices.push(index);
        }
        let record=session.record().map_err(|e|format!("{e:?}"))?.ok_or("missing native record")?.to_json().map_err(|e|format!("{e:?}"))?;
        let record:Value=serde_json::from_slice(&record).map_err(|e|e.to_string())?;
        cases.push(json!({"seed":seed,"history":name,"indices":indices,"record":record,"native_final":final_columns(&boundary.observation),"high_water":maximum}));
    }}
    Ok(json!({"cases":cases}))
}
fn main(){match execute(){Ok(value)=>println!("{value}"),Err(error)=>{eprintln!("M9 native agreement: {error}");std::process::exit(1)}}}
