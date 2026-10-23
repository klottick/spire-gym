//! Native combat foundations, source-grounded, not original-runtime validated.
pub mod rng;
pub mod enemies;
pub mod command;
pub use command::{CardDestination, CardInstanceId, CombatAction, CombatCardRef, EnemyTarget, GeneratedCardDestination, PermanentCard, TargetId};
pub mod rules;
mod content;
mod model;
mod engine;
mod effects;
mod powers;
mod piles;
mod turns;
mod status;
mod card_state;
mod characters;
pub mod cards;
mod selection;
mod hooks;
mod view;
pub mod snapshot;

pub use content::*;
pub use model::{AccessError, Action, Actor, CardHandle, CardInstance, CombatRng,
    ActorEnemyId, ActorIntentId, ActorLife, DamageEntry, DamagePayload, Effect, EffectError, EffectKind,
    EnemyHandle, EnemySetup, EnemyState, EnemyReplacementDiagnostics, SpawnRequest, EnemySpawnDiagnostics,
    CombatEntry, CombatFault, FaultKind, FaultRecord, FullCombatSetup, FullEnemySetup, FullIntentId, FullPowerEntry, FullPowerId, InputError, Outcome, Phase, Piles, PlayerState, PowerEntry,
    PowerState, PublicKnowledge, RestoreError, Scenario, Source, StepError, Transition,
    TransitionStatus};
pub use engine::{Combat, CombatEntryFailure};
pub use enemies::{EnemyMemory, MoveSelection};
pub use view::{CardView, PlayerView, EnemyView, PublicView, PublicProjection, CombatPublic, CombatExit, LiveCardView, StasisCardView, SelectionView, FullEnemyView, FullPlayerView, FullPowerView, CharacterCombatView, CombatInventoryView};
pub use card_state::{permanent as permanent_live_card, CombatCardData, LiveCard};
pub use characters::{CharacterEffect, CharacterState, OrbKind, Stance};
pub use cards::resolve;
pub use hooks::{Hook, HookEvent};
pub use selection::{EmptySelection, SelectionDisposition};
pub use snapshot::{Divergence, Snapshot};

#[cfg(test)]
mod tests;
