//! Opt-in run observation shares the ordinary run coordinator.
use super::*;
use crate::combat::{Observed,observe::Session};
impl Run {
    pub fn step_observed(&mut self,action:&GameAction)->Observed<Result<RunTransition,RunError>> {
        let session=Session::run(std::ptr::from_ref(self) as usize,std::ptr::from_ref(&self.state) as usize);
        let result=self.step(action);
        session.finish(result)
    }
}
