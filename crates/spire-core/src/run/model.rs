use std::collections::BTreeMap;
use crate::{combat::Combat, content::{CardId, Character, RelicId, PotionId}};
use super::{Candidate, ChoiceId, Key, RunRng};

#[derive(Clone, PartialEq, Eq)]
pub struct CardInstance { pub id:CardId, pub instance:u64, pub upgrades:u32, pub misc:i32 }
impl std::fmt::Debug for CardInstance {
    fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result {
        let mut d=f.debug_struct("CardInstance");d.field("id",&self.id).field("instance",&self.instance).field("upgrades",&self.upgrades);
        if self.misc!=0{d.field("misc",&self.misc);}d.finish()
    }
}
/// Explicit supplied run resources, not a normal Neow start or snapshot restore.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuthoredRunStart {
    pub hp:u32, pub max_hp:u32, pub gold:u32, pub deck:Vec<CardInstance>,
    pub relics:Vec<RelicId>, pub potions:Vec<Option<PotionId>>,
    pub relic_counters:BTreeMap<RelicId,i32>,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CombatProgress { pub entered:u64, pub settled:u64 }
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CombatRecord { pub visit:u64, pub encounter:crate::content::EncounterId, pub outcome:crate::combat::Outcome }
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct ActiveCombat { pub visit:u64, pub encounter:crate::content::EncounterId, pub origin:CombatOrigin,
    pub applied_gain_ordinal:u64,pub applied_mutation_ordinal:u64,pub acquired_instances:Vec<u64> }
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub(crate) enum CombatOrigin { Authored, Training{kind:MapNodeKind}, MapRoom{node:super::NodeId,kind:MapNodeKind} }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunTerminal { HeartVictory, Death, EmptyDeck, Act3NoKeys }
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RunFaultKind { UnsupportedBoundary(&'static str), Invariant(&'static str), Combat(Box<crate::combat::CombatFault>) }
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RunFault { pub kind:RunFaultKind, pub checkpoint:&'static str, pub committed_events:Vec<CommittedEvent> }
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RunError { InvalidAscension(u8), InvalidAction, Unsupported(&'static str), Registry(&'static str), Terminal, Faulted(RunFault) }
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CommittedEvent { FloorEntered { act:u8, floor:u32 }, KeyAcquired(Key), Ended(RunTerminal),
    ActAdvanced{from:u8,to:u8},
    CombatEntered { visit:u64, encounter:crate::content::EncounterId }, CombatFinished(CombatRecord) }
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RunTransition { pub reward:i32, pub terminal:Option<RunTerminal>, pub committed_events:Vec<CommittedEvent> }

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
pub enum ContinuationTarget { ResumeDecision(Candidate), NeowConsequence { parent:Candidate, offer:NeowOffer },
    Room {node:super::NodeId,choice:ChoiceId,operation:RoomSelectionOperation} }
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum RoomSelectionOperation {ShopPurge{price:u32},Smith,ShrineUpgrade,ShrineRemove,AncientElegance,EmptyCage}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PendingContinuation { pub parent:ContinuationTarget, pub selection:PendingSelection, pub resumed:bool }
#[derive(Clone,Debug,PartialEq,Eq)]
pub(crate) struct ActTransition {pub from:u8,pub to:u8,pub floor:u32,pub reset_applied:bool,pub heal_applied:bool,pub initialized:bool}

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
    pub(crate) boss_icon:Option<crate::content::EncounterId>,
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
    pub(crate) room_loop:Option<super::rooms::RoomLoop>,
    pub(crate) reward_ledger:super::reward::RunRewardLedger,
    pub(crate) authored_combat:bool, pub(crate) training_encounter:Option<crate::content::EncounterId>,
    pub(crate) last_combat_checkpoint:Option<super::CombatCheckpoint>, pub(crate) combat_progress:CombatProgress,
    pub(crate) active_combat:Option<ActiveCombat>, pub(crate) last_combat:Option<CombatRecord>,
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

/// Finite native authored combat, independent of normal Run progression.
#[derive(Clone,Debug,PartialEq,Eq)]
pub enum M16DefectCombatStart { Canonical, Supplied(AuthoredRunStart) }

impl std::fmt::Debug for ActiveCombat {
    fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result {
        let mut d=f.debug_struct("ActiveCombat");d.field("visit",&self.visit).field("encounter",&self.encounter).field("origin",&self.origin).field("applied_gain_ordinal",&self.applied_gain_ordinal).field("acquired_instances",&self.acquired_instances);
        if self.applied_mutation_ordinal!=0{d.field("applied_mutation_ordinal",&self.applied_mutation_ordinal);}d.finish()
    }
}

/// Accepted A0 setup metadata prefix at the M15 merge. Appends are not admission.
pub const M16_DEFECT_COMBAT_SETUP_PREFIX_LEN:usize=57;
pub const M16_DEFECT_COMBAT_SETUP_BASE:&str="4d7e5be44fd1ce0167729ef1a4f8c9b17b9c5bdf";
