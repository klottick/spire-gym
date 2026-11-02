//! Run entry points for the rooms M28 completes (M27 design §11.3). Each visits owned relics
//! in acquisition order through lane R3's handlers; M2's bodies change nothing.
#![allow(dead_code)]
use crate::content::RelicId;
use crate::run::{RoomChoiceKind, RunError, RunState};

/// The reward room kind (design §8.2 appearance law, §11.3 card-reward shape).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RewardRoom { Ordinary { enemy_escaped: bool }, Elite, Event, Boss }

/// Unknown-room resolution rules (Juzu Bracelet, Tiny Chest, Ssserpent Head).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct UnknownRoomRules { pub no_monsters: bool, pub force_treasure: bool }
/// Map path rules (Wing Boots).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct MapPathRules { pub off_path_moves: u32 }
/// Card reward shape (Question Card, Busted Crown, Prayer Wheel, N'loth's Gift).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct CardRewardShape { pub choices: i32, pub rewards: u32, pub rare_factor: u32 }
impl Default for CardRewardShape { fn default() -> Self { Self { choices: 3, rewards: 1, rare_factor: 1 } } }
/// Chest contents rules (Cursed Key, Matryoshka, N'loth's Hungry Face). Apply them to the built
/// chest rows with [`apply_chest_rules`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct ChestRules {
    pub curse: bool,
    /// Matryoshka's extra relic, already taken from the relic pools. Source onChestOpen adds it
    /// before the chest's own gold, relic and Sapphire Key rows.
    pub extra_relic: Option<RelicId>,
    /// N'loth's Hungry Face (source onChestOpenAfter): remove the first relic row and the Sapphire
    /// Key linked to it (the row right after it); gold and every other row stay.
    pub remove_first_relic: bool,
}

/// Apply chest rules to the chest's own rows (gold, relic, Sapphire Key linked to that relic, in
/// source order): Matryoshka's relic goes first, then Hungry Face removes the first relic row.
pub(crate) fn apply_chest_rules(rules: &ChestRules, rows: &mut Vec<RoomChoiceKind>) {
    if let Some(relic) = rules.extra_relic { rows.insert(0, RoomChoiceKind::Relic(relic)); }
    if rules.remove_first_relic {
        if let Some(index) = rows.iter().position(|row| matches!(row, RoomChoiceKind::Relic(_))) {
            rows.remove(index);
            if rows.get(index) == Some(&RoomChoiceKind::Sapphire) { rows.remove(index); }
        }
    }
}

/// Call exactly once per qualifying Unknown-room roll: Tiny Chest counts the roll here.
pub(crate) fn unknown_room(state: &mut RunState) -> UnknownRoomRules {
    let mut rules = UnknownRoomRules::default();
    for id in state.relics.clone() { super::relic_effects::run::unknown_room(id, state, &mut rules); }
    rules
}
pub(crate) fn map_path(state: &RunState) -> MapPathRules {
    let mut rules = MapPathRules::default();
    for id in state.relics.clone() { super::relic_effects::run::map_path(id, state, &mut rules); }
    rules
}
pub(crate) fn card_reward_shape(state: &RunState, room: RewardRoom) -> CardRewardShape {
    let mut shape = CardRewardShape::default();
    for id in state.relics.clone() { super::relic_effects::run::card_reward_shape(id, state, room, &mut shape); }
    shape
}
/// Call exactly once per opened chest. Atomic: on an error the state is unchanged.
pub(crate) fn chest_contents(state: &mut RunState, boss: bool) -> Result<ChestRules, RunError> {
    let mut rules = ChestRules::default();
    let mut staged = state.clone();
    for id in staged.relics.clone() { super::relic_effects::run::chest_contents(id, &mut staged, boss, &mut rules)?; }
    *state = staged;
    Ok(rules)
}
pub(crate) fn score_adjustment(state: &RunState) -> i32 {
    let mut score = 0;
    for id in state.relics.clone() { super::relic_effects::run::score_adjustment(id, state, &mut score); }
    score
}
