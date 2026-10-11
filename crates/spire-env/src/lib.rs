//! Spire Lab environment abstractions.
//!
//! Phase 1a scaffolding: documentation-only placeholder for the learning
//! environment built on `spire-core` (device, batch, and readiness support).

/// Re-export the core this environment is built on.
pub use spire_core::VERSION as CORE_VERSION;

/// The version of the `spire-env` crate.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
