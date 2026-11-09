//! Native combat foundations, source-grounded, not original-runtime validated.
pub mod rng;
pub mod enemies;
pub mod command;
pub use command::{CardDestination, CardInstanceId, CombatAction, CombatCardRef, EnemyTarget, GeneratedCardDestination, PermanentCard, TargetId};
pub mod rules;
mod content;
mod model;
mod engine;
#[cfg(any(test,feature="m22-diagnostics"))]
mod m22_diagnostics;
mod effects;
mod card_offers;
mod generation;
mod smoke;
pub(crate) use generation::offer_request;
pub use generation::{GenerationPool,GenerationCost,GenerationWork};
mod exhaust;
mod powers;
mod piles;
mod turns;
mod status;
pub(crate) mod card_state;
mod card_faces;
pub use card_faces::{static_card_face, static_card_upgrade_limit, StaticCardFace, StaticCost, StaticTarget};
mod characters;
pub mod cards;
mod selection;
mod hooks;
mod view;
pub mod public_ids;
pub mod intents;
pub(crate) mod observe;
pub use observe::{Observed, ResolvedEffect, ResolvedEffectKind, DamageCategory, PublicLife, PublicZone,
    PublicCombatSnapshot, CombatBoundary, CombatBoundaryKind};
pub mod snapshot;

pub use content::*;
pub use model::{AccessError, Action, Actor, CardHandle, CardInstance, CombatRng,
    ActorEnemyId, ActorIntentId, ActorLife, DamageEntry, DamagePayload, Effect, EffectError, EffectKind, SilentItemEffect, PotionEffect, RelicEffect,
    EnemyHandle, EnemySetup, EnemyState, EnemyReplacementDiagnostics, SpawnRequest, EnemySpawnDiagnostics,
    CombatEntry, CombatFault, FaultKind, FaultRecord, FullCombatSetup, FullEnemySetup, FullIntentId, FullPowerEntry, FullPowerId, InputError, Outcome, Phase, Piles, PlayerState, PowerEntry,
    PowerState, PublicKnowledge, PendingSelection, SelectionResume, ResolveMarks, ManualDiscardOrigin, RestoreError, Scenario, Source, StepError, Transition,
     TransitionStatus, PlayerFacing, EncounterSide};
pub use engine::{Combat, CombatEntryFailure};
pub use enemies::{EnemyMemory, MoveSelection};
pub use enemies::act1::thieves::{ThiefEffect,ThiefMemory,ThiefGoldState,StolenGoldReceipt};
pub use view::{WatcherCardValues,StanceChoiceView,CardView, PlayerView, EnemyView, PublicView, PublicProjection, CombatPublic, CombatExit, LiveCardView, StasisCardView, SelectionView, CombatCardOfferView, KnownDrawCardView, FullEnemyView, FullPlayerView, FullPowerView, CharacterCombatView, CombatInventoryView, EncounterFacingView, EncounterActorView};
pub use card_state::{permanent_with_misc as permanent_live_card_with_misc,permanent as permanent_live_card, CombatCardData, LiveCard, PermanentCardMutation, PermanentCardUpgrade};
pub use characters::{CharacterEffect, CharacterState, OrbKind, OrbPassiveGroup, OrbState, Stance};
pub use cards::{resolve,IroncladEffect};
pub use view::{CardCostKind,CardPublicValues};
pub use hooks::{Hook, HookEvent};
pub use selection::{EmptySelection, SelectionDisposition,SelectionOperation,SelectionParent,SelectionSourceZone};
pub use snapshot::{Divergence, Snapshot};

#[cfg(test)]
pub(crate) mod tests;

/// Construct fresh finite-profile card state through the shared physical owner.
/// Generated-only kinds receive combat-local origin, never permanent admission.
pub fn m21_watcher_live_card(id:crate::content::CardId,instance:CardInstanceId,upgrades:u32)->Result<LiveCard,InputError>{
    if instance.0==0||upgrades>1||!crate::content::watcher::supports_generated(id){return Err(InputError::Unsupported("invalid finite M21 card constructor".into()))}
    let mut card=if crate::content::watcher::supports_permanent(id){card_state::permanent(id,instance,upgrades)}else{card_state::generated(id,instance,upgrades).ok_or_else(||InputError::Unsupported("M21 generated constructor missing".into()))?};
    cards::watcher::initialize_profile(&mut card)?;Ok(card)
}

/// Construct fresh complete-capability (M26) card state through the shared physical owner
/// and the colour dispatcher. Generated-only kinds receive combat-local origin.
pub fn complete_live_card(id:crate::content::CardId,instance:CardInstanceId,upgrades:u32)->Result<LiveCard,InputError>{
    if instance.0==0||!crate::content::complete::supports_generated(id){return Err(InputError::Unsupported("invalid complete M26 card constructor".into()))}
    let mut card=if crate::content::complete::supports_permanent(id){card_state::permanent(id,instance,upgrades)}else{card_state::generated(id,instance,upgrades).ok_or_else(||InputError::Unsupported("M26 generated constructor missing".into()))?};
    cards::initialize_complete(&mut card)?;Ok(card)
}
/// Construct fresh complete-Watcher (M25) card state through the shared physical owner.
/// Generated-only children receive combat-local origin, never permanent admission.
pub fn m25_watcher_live_card(id:crate::content::CardId,instance:CardInstanceId,upgrades:u32)->Result<LiveCard,InputError>{
    if instance.0==0||upgrades>1||!crate::content::watcher::supports_m25_generated(id){return Err(InputError::Unsupported("invalid complete M25 card constructor".into()))}
    let mut card=if crate::content::watcher::supports_m25_permanent(id){card_state::permanent(id,instance,upgrades)}else{card_state::generated(id,instance,upgrades).ok_or_else(||InputError::Unsupported("M25 generated constructor missing".into()))?};
    cards::watcher::initialize_m25_profile(&mut card)?;Ok(card)
}
