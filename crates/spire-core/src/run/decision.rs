use crate::{combat::command::CombatAction, content::{CardId, Character, PotionId, RelicId}};
use super::{NeowOffer, RunTerminal};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)] pub struct NodeId(pub u64);
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)] pub struct ChoiceId(pub u64);
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)] pub struct SelectionId(pub u64);
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)] pub enum Key { Ruby, Sapphire, Emerald }
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GameAction { Move(NodeId), Combat(CombatAction), Choose(ChoiceId), Select(SelectionId), Confirm, Cancel,
    UsePotion { slot:u32, target:Option<crate::combat::command::TargetId> }, DiscardPotion { slot:u32 } }
#[derive(Clone, Debug, PartialEq, Eq)] pub struct PublicPlayer { pub character:Character, pub hp:u32, pub max_hp:u32, pub gold:u32, pub relics:Vec<RelicId>, pub potions:Vec<Option<PotionId>>, pub keys:[bool;3] }
#[derive(Clone, Debug, PartialEq, Eq)] pub struct PublicObject { pub instance:u64, pub id:CardId, pub upgrades:u32 }
#[derive(Clone, Debug, PartialEq, Eq)] pub struct PublicSelection { pub required:usize, pub selected_count:usize, pub eligible:Vec<PublicObject> }
#[derive(Clone, Debug, PartialEq, Eq)] pub struct Candidate { pub id:ChoiceId, pub action:GameAction, pub label:&'static str, pub neow_offer:Option<NeowOffer>, pub object:Option<PublicObject> }
impl Candidate { #[cfg(test)] pub(crate) fn internal_test_parent(id:ChoiceId)->Self { Self{id,action:GameAction::Confirm,label:"test parent",neow_offer:None,object:None} } }
#[derive(Clone, Debug, PartialEq, Eq)] pub struct PublicDecision { pub act:u8, pub floor:u32, pub player:PublicPlayer,
    pub objects:Vec<PublicObject>, pub candidates:Vec<Candidate>, pub offers:Vec<NeowOffer>, pub selection:Option<PublicSelection>, pub terminal:Option<RunTerminal> }
