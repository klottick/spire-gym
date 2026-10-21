//! Native combat foundations, source-grounded, not original-runtime validated.
pub mod rng;
pub mod rules;
mod content;
mod model;
mod engine;
mod effects;
mod piles;
mod turns;
mod view;
pub mod snapshot;

pub use content::*;
pub use model::{AccessError, Action, Actor, CardHandle, CardInstance, CombatRng,
    DamageEntry, DamagePayload, Effect, EffectError, EffectKind, EnemyHandle, EnemySetup, EnemyState,
    FaultKind, FaultRecord, InputError, Outcome, Phase, Piles, PlayerState, PowerEntry,
    PowerState, PublicKnowledge, RestoreError, Scenario, Source, StepError, Transition,
    TransitionStatus};
pub use engine::Combat;
pub use view::{CardView, PlayerView, EnemyView, PublicView, PublicProjection};
pub use snapshot::{Divergence, Snapshot};

#[cfg(test)]
mod tests;
