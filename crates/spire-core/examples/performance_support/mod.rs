//! Opt-in benchmark support, deliberately outside the production engine.
#![allow(dead_code)]
#[path = "../m1_support/mod.rs"]
pub mod m1;
#[path = "../m5_support/mod.rs"]
pub mod m5;
#[path = "../m6_support/mod.rs"]
pub mod m6;
pub mod contract;
pub mod workloads;
pub mod sampling;
pub use contract::*;
pub use workloads::*;
pub use sampling::*;
