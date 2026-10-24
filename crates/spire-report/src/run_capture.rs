use crate::{ReportWriter,ReportError};
use spire_core::run::*;
use serde_json::{Value,json};
use std::io::Write;
#[derive(Debug)]
pub enum CaptureError { Producer(RunError), WriteBeforeCall(ReportError), WriteAfterCall(ReportError), AdapterAfterCall(ReportError) }
/// Detached public evidence for a native call whose report could not be delivered.
/// Never a snapshot/restore protocol or an instruction to execute the action again.
#[derive(Clone,Debug)]
pub struct CaptureRecovery {
    pub action:Value, pub before:PublicDecision, pub after:Option<PublicDecision>,
    pub map:Option<spire_core::run::map::PublicMap>, pub transition:Option<RunTransition>,
    pub effects:Vec<spire_core::combat::ResolvedEffect>, pub boundaries:Vec<spire_core::combat::CombatBoundary>,
    pub frame:Option<Value>,
}
pub struct RunRecorder<'a,W:Write> {
    writer:&'a mut ReportWriter<W>,next:u64,started:bool,failed:bool,owner:Option<usize>,
    parent:Option<(String,Option<Value>)>,
    recovery:Option<CaptureRecovery>,
}
impl<'a,W:Write> RunRecorder<'a,W> {
    /// Take the undelivered call's public evidence. Does not unpoison the writer,
    /// reset the recorder or permit replaying the engine action.
    pub fn take_recovery(&mut self)->Option<CaptureRecovery> {self.recovery.take()}
    pub fn new(writer:&'a mut ReportWriter<W>)->Self {Self {writer,next:0,started:false,failed:false,owner:None,parent:None,recovery:None}}
    pub fn start(&mut self,run:&Run)->Result<(),ReportError> {
        if self.started||self.failed {return Err(invalid("recorder already started or failed"));}
        let d=run.decision().map_err(|_|invalid("initial public decision unavailable"))?;
        let state=crate::run_snapshot(&d,run.public_map().as_ref())?;
        self.writer.frame(self.frame(&d,"initial",Value::Null,Value::Null,state,vec![],Value::Null,"settled"))?;
        self.owner=Some(std::ptr::from_ref(run) as usize);self.started=true;self.next=1;Ok(())
    }
    pub fn step(&mut self,run:&mut Run,action:&GameAction)->Result<RunTransition,CaptureError> {
        if !self.started||self.failed||self.owner!=Some(std::ptr::from_ref(run) as usize) {return Err(CaptureError::WriteBeforeCall(invalid("recorder inactive or run moved/replaced; do not retry after post-call failure")));}
        let before=run.decision().map_err(CaptureError::Producer)?;
        if !before.candidates.iter().any(|c|&c.action==action) {return Err(CaptureError::Producer(RunError::InvalidAction));}
        let id=crate::snapshot::combat_id(&before);
        let mut described=crate::describe_run_action(&before,action,&id).map_err(CaptureError::WriteBeforeCall)?;
        let prior=crate::run_snapshot(&before,run.public_map().as_ref()).map_err(CaptureError::WriteBeforeCall)?;
        let before_screen=prior["screen"].clone();
        if matches!(action,GameAction::Select(_)|GameAction::Combat(spire_core::combat::CombatAction::SelectCard{..})|GameAction::Cancel) {
            if let Some((parent,card))=&self.parent {
                described["parentActionId"]=json!(parent);
                if let Some(card)=card {described["card"]=card.clone();}
            }
        }
        self.writer.checkpoint().map_err(CaptureError::WriteBeforeCall)?;
        // Exactly one native action. No fallback, retry, settlement or policy here.
        let observed=run.step_observed(action);
        let result=observed.result;
        // Retain typed public evidence BEFORE any fallible post-call adapter or
        // write. No private RunState/CombatFault/queue/snapshot is stored here.
        self.recovery=Some(CaptureRecovery {action:described.clone(),before,after:run.decision().ok(),map:run.public_map(),
            transition:result.as_ref().ok().cloned(),effects:observed.effects,boundaries:observed.boundaries,frame:None});
        let recovery=self.recovery.as_ref().unwrap();
        let before=&recovery.before;
        let after=&recovery.after;
        let (kind,validity,state)=match &result {
            Ok(_)=> {
                let d=match after {Some(d)=>d,None=>{self.failed=true;return Err(CaptureError::AdapterAfterCall(invalid("accepted action public decision unavailable")));}};
                let state=match crate::run_snapshot(d,recovery.map.as_ref()) {Ok(s)=>s,Err(e)=>{self.failed=true;return Err(CaptureError::AdapterAfterCall(e));}};
                ("action","settled",state)
            },
            Err(RunError::InvalidAction) if recovery.effects.is_empty()&&recovery.boundaries.is_empty()=>{
                self.recovery=None;return Err(CaptureError::Producer(RunError::InvalidAction));
            },
            Err(_)=>{self.failed=true;("fault","last_known",prior.clone())},
        };
        let final_combat=recovery.boundaries.iter().rev().find_map(|b| {
            if b.kind!=spire_core::combat::CombatBoundaryKind::Terminal{return None;}
            match &b.public {spire_core::combat::PublicCombatSnapshot::Native(p)=>Some(crate::snapshot::native_combat(p,&id)),_=>None}
        }).transpose().map_err(|e|{self.failed=true;CaptureError::AdapterAfterCall(e)})?.unwrap_or(Value::Null);
        let d=after.as_ref().unwrap_or(before);
        let mut frame=self.frame(d,kind,described.clone(),before_screen,state,crate::actions::effects(&recovery.effects,&id),final_combat,validity);
        if before.combat.is_some() {frame["combatId"]=json!(id);frame["turn"]=json!(before.combat.as_ref().unwrap().turn);}
        if let Ok(t)=&result {frame["reward"]=json!(t.reward);frame["committedEvents"]=json!(t.committed_events.iter().map(committed_event).collect::<Vec<_>>());}
        let selection=d.selection.is_some()||d.combat.as_ref().is_some_and(|c|c.selection.is_some());
        self.recovery.as_mut().unwrap().frame=Some(frame.clone());
        if let Err(e)=self.writer.frame(frame) {self.failed=true;return Err(CaptureError::WriteAfterCall(e));}
        if selection&&self.parent.is_none() {self.parent=Some((self.next.to_string(),described.get("card").filter(|v|v.is_object()).cloned()));}
        if !selection {self.parent=None;}
        self.next+=1;
        if result.is_ok(){self.recovery=None;}
        result.map_err(CaptureError::Producer)
    }
    pub fn finish(&mut self,run:&Run,capture:Value,game:Value)->Result<(),ReportError> {
        if !self.started||self.failed||self.owner!=Some(std::ptr::from_ref(run) as usize) {return Err(invalid("recorder inactive, failed or run moved/replaced; success footer forbidden"));}
        self.writer.finish(capture,game)?;self.started=false;Ok(())
    }
    #[allow(clippy::too_many_arguments)]
    fn frame(&self,d:&PublicDecision,kind:&str,action:Value,before:Value,state:Value,effects:Vec<Value>,final_combat:Value,validity:&str)->Value {
        json!({"id":self.next.to_string(),"previousFrameId":self.next.checked_sub(1).map(|i|i.to_string()),"kind":kind,"act":d.act,"floor":d.floor,
            "combatId":d.combat.as_ref().map(|_|crate::snapshot::combat_id(d)),"turn":d.combat.as_ref().map(|c|c.turn),"action":action,"beforeScreen":before,"state":state,
            "effects":effects,"combatFinal":final_combat,"stateValidity":validity})
    }
}
fn invalid(text:&str)->ReportError {ReportError::Invalid(text.into())}
fn committed_event(e:&CommittedEvent)->Value {match e {
    CommittedEvent::FloorEntered{act,floor}=>json!({"kind":"floor_entered","act":act,"floor":floor}),
    CommittedEvent::ActAdvanced{from,to}=>json!({"kind":"act_advanced","from":from,"to":to}),
    CommittedEvent::CombatEntered{visit,encounter}=>json!({"kind":"combat_entered","visit":visit.to_string(),"encounter":encounter.as_str()}),
    CommittedEvent::CombatFinished(r)=>json!({"kind":"combat_finished","visit":r.visit.to_string(),"encounter":r.encounter.as_str(),"outcome":match r.outcome {spire_core::combat::Outcome::Victory=>"victory",spire_core::combat::Outcome::Death=>"death"}}),
    CommittedEvent::KeyAcquired(k)=>json!({"kind":"key_acquired","keyName":match k {Key::Ruby=>"ruby",Key::Sapphire=>"sapphire",Key::Emerald=>"emerald"}}),
    CommittedEvent::Ended(t)=>json!({"kind":"ended","terminal":crate::snapshot::terminal(*t)}),
}}
