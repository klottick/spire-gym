//! Default-off fixed diagnostic initialization and read-only checkpoints. No arbitrary state importer.
#[cfg(any(test,feature="m20-diagnostics"))]
mod m20;
#[cfg(any(test,feature="m20-diagnostics"))]
pub use m20::{M20DiagnosticLoadout,m20_supplied_act4};

#[cfg(any(test,feature="m22-diagnostics"))]
mod m22;
#[cfg(any(test,feature="m22-diagnostics"))]
pub use m22::{M22Diagnostic,M22DiagnosticBoss,M22DiagnosticBranch,m22_diagnostic,m22_supplied_boss,m22_diagnostic_checkpoint,M22DiagnosticCheckpoint,M22TransitionCheckpoint,M22CombatCheckpoint,M22DiagnosticCard,M22DiagnosticZone,M22CreationCheckpoint};

#[cfg(any(test,feature="m23-diagnostics"))]
mod m23;
#[cfg(any(test,feature="m23-diagnostics"))]
pub use m23::{M23DiagnosticBoss,M23PlayedFixture,M23Diagnostic,m23_supplied_boss,m23_played_fixture,m23_diagnostic};

#[cfg(any(test,feature="m24-diagnostics"))]
mod m24;
#[cfg(any(test,feature="m24-diagnostics"))]
pub use m24::{M24DiagnosticBoss,M24PlayedFixture,M24Diagnostic,m24_supplied_traversal,m24_supplied_boss,m24_supplied_boss_at,m24_played_fixture,m24_diagnostic,LOADOUT_V2_DECK,LOADOUT_V2_RELICS,LOADOUT_V2_POTIONS,LOADOUT_V2_HP};
