//! Spire Lab simulation core.
//!
//! This crate hosts the deterministic Slay the Spire combat simulation and
//! its batched numeric interface. Per the plan, the core keeps no Python,
//! tensor-library, renderer, or training dependency, and game logic never
//! uses Rust's default RNG.
//!
//! `spire_core::trace` implements the game-independent `trace-v1` reference
//! trace protocol (read side). The matching writer lives in the Java
//! `:protocol` project under `validation/windows-mod`, which must produce
//! files this module accepts.
#[cfg(test)]
extern crate self as spire_core;

pub mod combat;
pub mod content;
pub mod items;
pub mod trace;
pub mod run;

/// Cargo features enabled when this crate was compiled, excluding `default`.
pub const ENABLED_FEATURES: &[&str] = &[
    #[cfg(feature = "fuzz-diagnostics")] "spire-core/fuzz-diagnostics",
    #[cfg(feature = "m20-diagnostics")] "spire-core/m20-diagnostics",
    #[cfg(feature = "m22-diagnostics")] "spire-core/m22-diagnostics",
    #[cfg(feature = "m23-diagnostics")] "spire-core/m23-diagnostics",
    #[cfg(feature = "m24-diagnostics")] "spire-core/m24-diagnostics",
    #[cfg(feature = "m25-diagnostics")] "spire-core/m25-diagnostics",
    #[cfg(feature = "m26-diagnostics")] "spire-core/m26-diagnostics",
    #[cfg(feature = "m27-diagnostics")] "spire-core/m27-diagnostics",
    #[cfg(feature = "m28-diagnostics")] "spire-core/m28-diagnostics",
    #[cfg(feature = "m29-diagnostics")] "spire-core/m29-diagnostics",
    #[cfg(feature = "m34-diagnostics")] "spire-core/m34-diagnostics",
];
