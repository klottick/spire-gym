use std::collections::BTreeMap;
use crate::{combat::Combat, content::{CardId, Character, RelicId, PotionId}};
use super::{Candidate, ChoiceId, Key, RunRng};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CardInstance { pub id:CardId, pub instance:u64, pub upgrades:u32 }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunTerminal { HeartVictory, Death, EmptyDeck, Act3NoKeys }
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RunFaultKind { UnsupportedBoundary(&'static str), Invariant(&'static str), Combat(Box<crate::combat::CombatFault>) }
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RunFault { pub kind:RunFaultKind, pub checkpoint:&'static str, pub committed_events:Vec<CommittedEvent> }
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RunError { InvalidAscension(u8), InvalidAction, Unsupported(&'static str), Registry(&'static str), Terminal, Faulted(RunFault) }
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CommittedEvent { FloorEntered { act:u8, floor:u32 }, KeyAcquired(Key), Ended(RunTerminal) }
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RunTransition { pub terminal:Option<RunTerminal>, pub committed_events:Vec<CommittedEvent> }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NeowReward { ThreeCards, RandomRare, RemoveOne, UpgradeOne, TransformOne, ThreeUncommonColorless,
    ThreePotions, CommonRelic, MaxHp, Lament, Gold, RareColorless, RareRelic, ThreeRareCards,
    TransformTwo, RemoveTwo, Gold250, MaxHpTwice, BossSwap }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NeowDrawback { MaxHpLoss, NoGold, RandomCurse, CurrentHpLoss }
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NeowOffer { pub category:u8, pub reward:NeowReward, pub drawback:Option<NeowDrawback> }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EmptySelectionRule { ResumeParent, Fault }
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectionCandidate { pub id:super::SelectionId, pub card:CardInstance }
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PendingSelection { pub candidates:Vec<SelectionCandidate>, pub required:usize,
    pub selected:Vec<CardInstance>, pub empty_rule:EmptySelectionRule }
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ContinuationTarget { ResumeDecision(Candidate), NeowConsequence { parent:Candidate, offer:NeowOffer } }
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PendingContinuation { pub parent:ContinuationTarget, pub selection:PendingSelection, pub resumed:bool }

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum MapNodeKind { Monster, Elite, Shop, Rest, Question, Treasure, Boss, Event }
impl MapNodeKind {
    pub const fn label(self)->&'static str { match self {
        Self::Monster=>"Monster", Self::Elite=>"Elite", Self::Shop=>"Shop", Self::Rest=>"Rest",
        Self::Question=>"?", Self::Treasure=>"Treasure", Self::Boss=>"Boss", Self::Event=>"Event",
    }}
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MapNode {
    pub(crate) id:super::NodeId, pub(crate) floor:u32, pub(crate) label:&'static str, pub(crate) edges:Vec<super::NodeId>,
    pub(crate) x:u8, pub(crate) y:u8, pub(crate) kind:MapNodeKind, pub(crate) hidden_kind:Option<MapNodeKind>,
    pub(crate) visited:bool, pub(crate) revealed:bool, pub(crate) encounter:Option<crate::content::EncounterId>, pub(crate) burning:bool,
}
#[derive(Clone, Default, PartialEq, Eq)]
pub struct RunMap {
    pub(crate) act:u8, pub(crate) draft_ordinal:u64, pub(crate) nodes:Vec<MapNode>,
    pub(crate) current:Option<super::NodeId>, pub(crate) winged_moves:u8,
    pub(crate) encounter_history:Vec<crate::content::EncounterId>,
    pub(crate) regular_encounter_history:Vec<crate::content::EncounterId>,
    pub(crate) elite_encounter_history:Vec<crate::content::EncounterId>,
    // Binary32 probability values stored as bits so RunMap retains Eq/replay semantics.
    pub(crate) question_ramps:[u32;3], pub(crate) question_roll_ordinal:u64,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VisitedRoom { pub act:u8, pub floor:u32, pub label:&'static str }
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RunHistory { pub rooms:Vec<VisitedRoom>, pub events:Vec<CommittedEvent> }

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RunState {
    pub character:Character, pub ascension:u8, pub hp:u32, pub max_hp:u32, pub gold:u32,
    pub deck:Vec<CardInstance>, pub relics:Vec<RelicId>, pub potions:Vec<Option<PotionId>>,
    pub keys:[bool;3], pub act:u8, pub floor:u32, pub room:&'static str,
    pub act4_enabled:bool, pub neow_eligible:bool, pub seen_bosses:[Vec<&'static str>;3], pub stored_note:(CardId,u32),
    pub terminal:Option<RunTerminal>, pub(crate) map:Option<RunMap>, pub history:RunHistory,
    pub relic_counters:BTreeMap<RelicId,i32>, pub combat:Option<Combat>,
    pub selected_neow:Option<NeowOffer>, pub fault:Option<RunFault>,
    pub last_selection:Option<Vec<CardInstance>>, pub last_resumed_parent:Option<ChoiceId>,
    pub(crate) rng:RunRng, pub(crate) pending:Option<PendingContinuation>,
    pub(crate) next_instance:u64, pub(crate) resume_count:u8,
}
