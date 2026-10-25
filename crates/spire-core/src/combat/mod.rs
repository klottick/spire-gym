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
mod exhaust;
mod powers;
mod piles;
mod turns;
mod status;
pub(crate) mod card_state;
mod characters;
pub mod cards;
mod selection;
mod hooks;
mod view;
pub(crate) mod observe;
pub use observe::{Observed, ResolvedEffect, ResolvedEffectKind, DamageCategory, PublicLife, PublicZone,
    PublicCombatSnapshot, CombatBoundary, CombatBoundaryKind};
pub mod snapshot;

pub use content::*;
pub use model::{AccessError, Action, Actor, CardHandle, CardInstance, CombatRng,
    ActorEnemyId, ActorIntentId, ActorLife, DamageEntry, DamagePayload, Effect, EffectError, EffectKind,
    EnemyHandle, EnemySetup, EnemyState, EnemyReplacementDiagnostics, SpawnRequest, EnemySpawnDiagnostics,
    CombatEntry, CombatFault, FaultKind, FaultRecord, FullCombatSetup, FullEnemySetup, FullIntentId, FullPowerEntry, FullPowerId, InputError, Outcome, Phase, Piles, PlayerState, PowerEntry,
    PowerState, PublicKnowledge, PendingSelection, RestoreError, Scenario, Source, StepError, Transition,
     TransitionStatus, PlayerFacing, EncounterSide};
pub use engine::{Combat, CombatEntryFailure};
pub use enemies::{EnemyMemory, MoveSelection};
pub use view::{CardView, PlayerView, EnemyView, PublicView, PublicProjection, CombatPublic, CombatExit, LiveCardView, StasisCardView, SelectionView, KnownDrawCardView, FullEnemyView, FullPlayerView, FullPowerView, CharacterCombatView, CombatInventoryView, EncounterFacingView, EncounterActorView};
pub use card_state::{permanent as permanent_live_card, CombatCardData, LiveCard};
pub use characters::{CharacterEffect, CharacterState, OrbKind, Stance};
pub use cards::{resolve,IroncladEffect};
pub use view::{CardCostKind,CardPublicValues};
pub use hooks::{Hook, HookEvent};
pub use selection::{EmptySelection, SelectionDisposition,SelectionOperation,SelectionParent,SelectionSourceZone};
pub use snapshot::{Divergence, Snapshot};

#[cfg(test)]
mod tests;
