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
