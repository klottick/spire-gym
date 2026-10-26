//! Owned run lifecycle. Intermediate room mechanics fail explicitly until their owners land.
mod model;
mod profile;
mod decision;
mod engine;
mod combat;
mod training;
pub use training::*;
mod observe;
mod reward;
mod mutations;
pub(crate) mod rooms;
pub use rooms::{RoomScenario,RoomView,RoomViewKind,RoomChoiceKind,RoomChoiceView,RoomChoiceStatus};
mod rng;
pub mod map;
pub mod world;
pub mod public;
pub use model::*;
pub use profile::*;
pub use decision::*;
pub use engine::*;
pub use rng::*;
pub use map::ActMap;
#[cfg(test)] mod tests;

pub use rooms::m14_content::{M14_SILENT_ACT3_CARDS,M14_SILENT_ACT3_RELICS,M14_SILENT_ACT3_POTIONS};
pub use rooms::m12_content::{M12_SILENT_ACT1_CARDS,M12_SILENT_ACT1_RELICS,M12_SILENT_ACT1_POTIONS};
