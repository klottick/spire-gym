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

/// Opt-in M15 permanent admission and full real potion-generation support.
pub const M15_SILENT_HEART_CARDS:&[crate::content::CardId]=rooms::m15_content::M15_PERMANENT_CARDS;
pub const M15_SILENT_HEART_RELICS:&[crate::content::RelicId]=&rooms::m15_content::M15_RELICS28;
pub const M15_SILENT_HEART_POTIONS:&[crate::content::PotionId]=crate::items::potion_generation::SILENT_POTIONS;
