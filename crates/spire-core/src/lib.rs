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
pub mod trace;
