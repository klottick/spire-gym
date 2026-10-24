use crate::{ReportWriter,ReportError};
use std::io::Write;
use spire_env::{EpisodeRecord,ReplayResult};
use serde_json::{json,Value};

pub fn capture_combat<W:Write>(record:&EpisodeRecord,writer:&mut ReportWriter<W>)->Result<ReplayResult,ReportError> {
    let combat=format!("{}/combat-0",record.job.episode_id());
    let mut previous=None;
    let mut adapter_error=None;
    let result=spire_env::replay_observed(record,|visit| {
        let result=(|| {
            writer.checkpoint()?;
            let state=match &visit.public {Some(public)=>crate::snapshot::legacy_snapshot_for(public,&combat)?,
                None=>previous.as_ref().map(|p|crate::snapshot::legacy_snapshot_for(p,&combat)).transpose()?.ok_or_else(||ReportError::Invalid("no public replay checkpoint".into()))?};
            let action=visit.action.as_ref().zip(previous.as_ref()).map(|(a,p)|crate::actions::legacy_action(a,p,&combat));
            let kind=match visit.kind {spire_env::ReplayVisitKind::Initial=>"initial",spire_env::ReplayVisitKind::AcceptedAction=>"action",spire_env::ReplayVisitKind::FaultAttempt=>"fault"};
            writer.frame(json!({"id":visit.index.to_string(),"previousFrameId":visit.index.checked_sub(1).map(|i|i.to_string()),"kind":kind,
                "act":record.job.template().context.act,"floor":null,"combatId":combat,"turn":visit.public.as_ref().map(|p|p.turn),
                "action":action,"beforeScreen":null,"state":state,"effects":crate::actions::effects(&visit.effects,&combat),
                "combatFinal":null,"stateValidity":if visit.public.is_some(){"settled"}else{"last_known"}}))?;
            if let Some(public)=visit.public {previous=Some(public);}
            Ok::<_,ReportError>(())
        })();
        result.map_err(|error| {let text=error.to_string();adapter_error=Some(error);text})
    });
    if let Some(error)=adapter_error {return Err(error);}
    let result=result.map_err(|e|ReportError::Invalid(format!("native replay: {e:?}")))?;
    let status=match record.status {spire_env::RecordStatus::Active=>"unfinished",spire_env::RecordStatus::Completed=>"complete",spire_env::RecordStatus::Abandoned=>"abandoned",spire_env::RecordStatus::Faulted=>"faulted"};
    let outcome=previous.as_ref().and_then(|p|p.outcome).map(|o|match o {spire_core::combat::Outcome::Victory=>"combat_victory",spire_core::combat::Outcome::Death=>"death"});
    writer.finish(json!({"status":status,"reason":match record.status {spire_env::RecordStatus::Active=>Some("active native prefix"),spire_env::RecordStatus::Abandoned=>Some("native episode abandoned"),spire_env::RecordStatus::Faulted=>Some("native record faulted; external faults are not fabricated"),_=>None}}),
        json!({"outcome":outcome,"boundary":Value::Null}))?;
    Ok(result)
}
