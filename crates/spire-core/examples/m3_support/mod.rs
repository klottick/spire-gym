//! Authored initial resources, not a normal start or game-rule implementation.
use spire_core::{combat::rng::Partition, content::{CardId, Character, PotionId, RelicId}, run::*};

pub fn config() -> RunConfig { RunConfig { character: Character::Ironclad, ascension: 0 } }
pub fn rng(seed: u64) -> RunRng {
    let mut key = [0; 32];
    key[..8].copy_from_slice(&seed.to_le_bytes());
    RunRng::new(key, Partition::Test, 0)
}
pub fn start() -> AuthoredRunStart {
    AuthoredRunStart { hp: 40, max_hp: 80, gold: 123,
        deck: [CardId::Strike, CardId::Bash, CardId::Anger, CardId::Bloodletting, CardId::Disarm]
            .into_iter().enumerate().map(|(i,id)| CardInstance { id, instance: i as u64 + 1,
                upgrades: u32::from(i < 2) }).collect(),
        relics: vec![RelicId::BurningBlood, RelicId::BloodVial, RelicId::Lantern, RelicId::Anchor],
        potions: vec![Some(PotionId::StrengthPotion), Some(PotionId::FirePotion), Some(PotionId::BloodPotion)],
        relic_counters: Default::default() }
}
pub fn run(seed: u64) -> Run { Run::from_authored_combat(config(), start(), rng(seed)).unwrap() }

#[derive(Debug)]
pub enum DemoError { Run(RunError), BudgetExceeded {accepted:usize} }
impl From<RunError> for DemoError { fn from(error:RunError)->Self {Self::Run(error)} }
impl std::fmt::Display for DemoError {
    fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result {
        match self {
            Self::Run(error)=>write!(f,"run error: {error:?}"),
            Self::BudgetExceeded {accepted}=>write!(f,"unfinished scenario: action budget exhausted after {accepted} accepted actions"),
        }
    }
}
pub struct Caller {accepted:usize,limit:usize}
impl Caller {
    pub fn new(limit:usize)->Self {Self {accepted:0,limit:limit.min(128)}}
    pub fn accepted(&self)->usize {self.accepted}
    pub fn step(&mut self,run:&mut Run,action:spire_core::combat::CombatAction)->Result<RunTransition,DemoError> {
        if self.accepted==self.limit {return Err(DemoError::BudgetExceeded {accepted:self.accepted});}
        let action=GameAction::Combat(action);
        if !run.decision()?.candidates.iter().any(|c|c.action==action) {return Err(RunError::InvalidAction.into());}
        let result=run.step(&action);
        if result.is_ok() || matches!(&result,Err(RunError::Faulted(_))) {self.accepted+=1;}
        Ok(result?)
    }
}
