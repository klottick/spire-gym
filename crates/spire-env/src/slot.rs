//! Exclusive native episode ownership and cached public decision boundaries.
use spire_core::combat::{Combat, Outcome, TransitionStatus,Transition,StepError,Observed};
use crate::{DecisionToken, EnvError, EpisodeInfo, EpisodeJob, FaultStage, ObservationRow, SlotFault};
use crate::actions::{CandidateMenu, MenuAction};
use crate::episode::{EpisodeSequence, PreparedEpisode};
use crate::knowledge::PublicKnowledge;
use crate::observation::encode;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SlotStatus { Uninitialized, Ready, Terminal }

#[derive(Default)]
pub(crate) struct Slot {
    pub(crate) sequence: EpisodeSequence,
    pub(crate) live: Option<LiveEpisode>,
    pub(crate) retired:Option<std::sync::Arc<crate::replay::Trace>>,
    #[cfg(test)]
    pub(crate) injection:Option<crate::tests::workers_faults::Injection>,
}

pub(crate) struct LiveEpisode {
    pub(crate) combat: Combat,
    pub(crate) job: EpisodeJob,
    pub(crate) knowledge: PublicKnowledge,
    pub(crate) row: ObservationRow,
    pub(crate) menu: CandidateMenu,
    pub(crate) token: DecisionToken,
    pub(crate) steps: u64,
    pub(crate) info: EpisodeInfo,
    pub(crate) trace:Option<std::sync::Arc<crate::replay::Trace>>,
    pub(crate) trace_queued:bool,
}

impl Slot {
    pub(crate) fn status(&self) -> SlotStatus {
        match &self.live {
            None => SlotStatus::Uninitialized,
            Some(l) if l.info.outcome != 0 => SlotStatus::Terminal,
            Some(_) => SlotStatus::Ready,
        }
    }
    pub(crate) fn build(prepared: PreparedEpisode) -> Result<Self, EnvError> {
        let combat = prepared.job.instantiate()?;
        let knowledge = PublicKnowledge::from_template(prepared.job.template());
        let menu = CandidateMenu::from_combat(&combat)?;
        let row = encode(&combat,&knowledge,&menu,&prepared.job.template().context)?;
        let info = EpisodeInfo { episode_id:prepared.job.episode_id(),
            scenario_id:prepared.job.template().id.clone(), episode_steps:0, outcome:0,
            hp_before_healing:None,hp_after_cleanup:None };
        Ok(Self { sequence:prepared.sequence,retired:None,
            #[cfg(test)]
            injection:None,
            live:Some(LiveEpisode { combat,
            job:prepared.job,knowledge,row,menu,token:DecisionToken::default(),steps:0,info,trace:None,trace_queued:false }) })
    }
}

impl LiveEpisode {
    /// The coordinator has validated the complete batch and checked steps + 1.
    pub(crate) fn advance(&mut self, accepted: &MenuAction) -> Result<(), EnvError> {
        let result=self.combat.step(&accepted.core);
        self.complete_advance(accepted,result)
    }
    pub(crate) fn advance_observed(&mut self, accepted:&MenuAction)->Observed<Result<(),EnvError>> {
        let observed=self.combat.step_observed(&accepted.core);
        Observed {result:self.complete_advance(accepted,observed.result),effects:observed.effects,boundaries:observed.boundaries}
    }
    fn complete_advance(&mut self,accepted:&MenuAction,result:Result<Transition,StepError>)->Result<(),EnvError> {
        let transition = result
            .map_err(|e| self.failure(FaultStage::Resolution,format!("cached action rejected: {e:?}")))?;
        if transition.status == TransitionStatus::Fault {
            return Err(self.failure(FaultStage::Resolution,"core transition fault".into()));
        }
        self.steps += 1;
        let p = self.combat.public_projection()
            .map_err(|e| self.failure(FaultStage::Encoding,format!("completed projection: {e:?}")))?;
        self.knowledge.observe_completed(&self.row,accepted,&p)?;
        let outcome = match p.outcome { None => 0,Some(Outcome::Victory) => 1,Some(Outcome::Death) => 2 };
        self.info.episode_steps = self.steps;
        self.info.outcome = outcome;
        self.info.hp_before_healing = p.hp_before_healing;
        self.info.hp_after_cleanup = p.hp_after_cleanup;
        self.menu = CandidateMenu::from_combat(&self.combat)?;
        self.row = encode(&self.combat,&self.knowledge,&self.menu,&self.job.template().context)?;
        // Until coordination publishes a fresh decision, no old token is valid.
        self.token = DecisionToken::default();
        Ok(())
    }
    fn failure(&self, stage: FaultStage, message: String) -> EnvError {
        EnvError::Fault { records:vec![SlotFault { slot:None,stage,message,core_fault:self.combat.fault_record() }] }
    }
}
