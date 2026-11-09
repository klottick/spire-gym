use crate::{combat::command::CombatAction, content::{CardId, Character, PotionId, RelicId}};
use super::{NeowOffer, RunTerminal};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)] pub struct NodeId(pub u64);
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)] pub struct ChoiceId(pub u64);
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)] pub struct SelectionId(pub u64);
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)] pub enum Key { Ruby, Sapphire, Emerald }
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GameAction { Move(NodeId), Combat(CombatAction), Choose(ChoiceId), Select(SelectionId), Confirm, Cancel,
    UsePotion { slot:u32, target:Option<crate::combat::command::TargetId> }, DiscardPotion { slot:u32 } }
#[derive(Clone, Debug, PartialEq, Eq)] pub struct PublicPlayer { /// M34a a4: bottled relic → linked master-deck card instance (the game draws a bottle on it).
    pub bottled_cards:Vec<(RelicId,u64)>, pub relic_counters:Vec<crate::items::RelicCounterView>, pub character:Character, pub hp:u32, pub max_hp:u32, pub gold:u32, pub relics:Vec<RelicId>, pub potions:Vec<Option<PotionId>>, pub keys:[bool;3] }
#[derive(Clone, PartialEq, Eq)] pub struct PublicObject { pub instance:u64, pub id:CardId, pub upgrades:u32, pub misc:i32 }
impl std::fmt::Debug for PublicObject {
    fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result {
        let mut d=f.debug_struct("PublicObject");d.field("instance",&self.instance).field("id",&self.id).field("upgrades",&self.upgrades);
        if self.misc!=0{d.field("misc",&self.misc);}d.finish()
    }
}
#[derive(Clone, Debug, PartialEq, Eq)] pub struct PublicSelection { pub required:usize, pub selected_count:usize, pub eligible:Vec<PublicObject> }
#[derive(Clone, Debug, PartialEq, Eq)] pub struct Candidate { pub id:ChoiceId, pub action:GameAction, pub label:&'static str, pub neow_offer:Option<NeowOffer>, pub object:Option<PublicObject> }
impl Candidate { #[cfg(test)] pub(crate) fn internal_test_parent(id:ChoiceId)->Self { Self{id,action:GameAction::Confirm,label:"test parent",neow_offer:None,object:None} } }
#[derive(Clone, Debug, PartialEq, Eq)] pub struct PublicDecision { pub content_profile:Option<super::RoomScenario>, pub room:Option<super::RoomView>, pub reward_total:i32, pub act:u8, pub floor:u32, pub player:PublicPlayer,
    pub combat:Option<crate::combat::CombatPublic>, pub combat_progress:super::CombatProgress, pub last_combat:Option<super::CombatRecord>,
    pub objects:Vec<PublicObject>, pub candidates:Vec<Candidate>, pub offers:Vec<NeowOffer>, pub selection:Option<PublicSelection>, pub terminal:Option<RunTerminal>,
    /// M34a a3/a4: numbers the current room displays (Standard only; `None` for captured profiles).
    pub room_params:Option<RoomParams> }

/// Numbers a Standard room displays besides its rows' kinds and prices (M34a a3/a4). Meanings are
/// fixed per (room, stage, option, index) by `rooms::standard::events::PARAM_SEMANTICS` and, for a
/// Treasure room, `stage[0]` = the unopened chest's size (1 small, 2 medium, 3 large).
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct RoomParams { pub stage:[Option<i32>;4], pub rows:Vec<(ChoiceId,[Option<i32>;3])> }
