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
    pub applied_gain_ordinal:u64,pub applied_mutation_ordinal:u64,pub acquired_instances:Vec<u64>,
    /// M25 Lesson Learned: the last permanent-upgrade journal ordinal applied to the deck.
    pub applied_upgrade_ordinal:u64 }
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub(crate) enum CombatOrigin { Authored, Training{kind:MapNodeKind}, MapRoom{node:super::NodeId,kind:MapNodeKind},
    /// M28 standard event combat (design §5.7): admitted only from the live event frame at `node`.
    Event{node:super::NodeId,event:crate::content::EventId,stage:u8} }
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
pub enum RoomSelectionOperation {ShopPurge{price:u32},Smith,ShrineUpgrade,ShrineRemove,AncientElegance,EmptyCage,RelicEquip{relic:RelicId},CampfireToke,
    /// M28 committed Neow selector (design §5.6, §6.5): `count` cards, no Cancel.
    NeowSelect{op:NeowSelectOp,count:u8},
    /// M28 event card selection (design §5.6): `count` cards, returning to the event frame.
    EventSelect{event:crate::content::EventId,stage:u8,op:EventSelectOp,count:u8}}
/// An event selection's operation; each has one candidate filter (design §5.6).
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum EventSelectOp {Remove,Upgrade,Transform,Duplicate,
    /// Choose a purgeable card for the event body to use (Bonfire Spirits, Note For Yourself).
    Choose}
/// The Neow selector's operation (Neow research :109-112).
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum NeowSelectOp {Remove,Upgrade,Transform}
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

#[derive(Clone, PartialEq, Eq)]
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
    /// M26 card admission selector; `Profile` keeps every scenario's finite law.
    pub(crate) card_admission:CardAdmission,
    /// M27 item admission selector; `Profile` keeps every scenario's finite item law.
    pub(crate) item_admission:ItemAdmission,
    /// M27 complete-admission relic offer history; absent under every frozen profile.
    pub(crate) relic_pools:Option<crate::items::relic_pools::RelicPools>,
    /// Bottled relic card links (design §7.5), moved into combat with the inventory.
    pub(crate) relic_cards:BTreeMap<RelicId,u64>,
    /// Complete-item potion appearance history; zero under every frozen profile.
    pub(crate) potion_offset:i32,
    /// M28 standard card-rarity offset `b` (design §6.1); `None` outside `RoomScenario::StandardA0`.
    pub(crate) card_rarity_offset:Option<i32>,
    /// M28 standard event lists and history (design §5.2); `None` outside `RoomScenario::StandardA0`.
    pub(crate) event_pools:Option<super::rooms::standard::event_pool::EventPools>,
    /// Complete-item per-slot stored potency, paired with its potion identity.
    pub(crate) potion_potencies:Vec<Option<(PotionId,i32)>>,
    /// M27 Run-potion restriction supplied by We Meet Again (false in frozen profiles).
    pub(crate) we_meet_again:bool,
}

/// The derived field text, with `card_admission` printed only when `Complete`
/// (M25 prefix tests compare RunState Debug text).
impl std::fmt::Debug for RunState {
    fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result{
        let mut value=f.debug_struct("RunState");
        value.field("room_loop",&self.room_loop);
        value.field("reward_ledger",&self.reward_ledger);
        value.field("authored_combat",&self.authored_combat);
        value.field("training_encounter",&self.training_encounter);
        value.field("last_combat_checkpoint",&self.last_combat_checkpoint);
        value.field("combat_progress",&self.combat_progress);
        value.field("active_combat",&self.active_combat);
        value.field("last_combat",&self.last_combat);
        value.field("character",&self.character);
        value.field("ascension",&self.ascension);
        value.field("hp",&self.hp);
        value.field("max_hp",&self.max_hp);
        value.field("gold",&self.gold);
        value.field("deck",&self.deck);
        value.field("relics",&self.relics);
        value.field("potions",&self.potions);
        value.field("keys",&self.keys);
        value.field("act",&self.act);
        value.field("floor",&self.floor);
        value.field("room",&self.room);
        value.field("act4_enabled",&self.act4_enabled);
        value.field("neow_eligible",&self.neow_eligible);
        value.field("seen_bosses",&self.seen_bosses);
        value.field("stored_note",&self.stored_note);
        value.field("terminal",&self.terminal);
        value.field("map",&self.map);
        value.field("history",&self.history);
        value.field("relic_counters",&self.relic_counters);
        value.field("combat",&self.combat);
        value.field("selected_neow",&self.selected_neow);
        value.field("fault",&self.fault);
        value.field("last_selection",&self.last_selection);
        value.field("last_resumed_parent",&self.last_resumed_parent);
        value.field("rng",&self.rng);
        value.field("pending",&self.pending);
        value.field("next_instance",&self.next_instance);
        value.field("resume_count",&self.resume_count);
        if let Some(pools)=&self.relic_pools{value.field("relic_pools",pools);}
        if self.card_admission!=CardAdmission::Profile{value.field("card_admission",&self.card_admission);}
        if self.item_admission!=ItemAdmission::Profile{value.field("item_admission",&self.item_admission);}
        if !self.relic_cards.is_empty(){value.field("relic_cards",&self.relic_cards);}
        if self.potion_offset!=0{value.field("potion_offset",&self.potion_offset);}
        if let Some(offset)=self.card_rarity_offset{value.field("card_rarity_offset",&offset);}
        if let Some(pools)=&self.event_pools{value.field("event_pools",pools);}
        if !self.potion_potencies.is_empty(){value.field("potion_potencies",&self.potion_potencies);}
        if self.we_meet_again{value.field("we_meet_again",&self.we_meet_again);}
        value.finish()
    }
}

/// Which card set the Run's mutation owners and combat entry admit (design §5.3).
#[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]
pub enum CardAdmission{
    /// Each scenario's finite profile law (`run::mutations::supported_in`).
    #[default] Profile,
    /// The complete permanent union (`content::complete::supports_permanent`).
    Complete,
}

/// Which relic and potion sets the Run's item owners and combat entry admit (M27 design
/// §5.3). `Complete` requires `CardAdmission::Complete` (`RunAdmission::new`).
#[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]
pub enum ItemAdmission{
    /// Each scenario's finite item law.
    #[default] Profile,
    /// The character's complete relic and potion unions (`content::complete`).
    Complete,
}

/// A valid pair of admissions, checked before any Run state is built.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub(crate) struct RunAdmission{pub(crate) cards:CardAdmission,pub(crate) items:ItemAdmission}
impl RunAdmission{
    pub(crate) fn new(cards:CardAdmission,items:ItemAdmission)->Result<Self,RunError>{
        if items==ItemAdmission::Complete&&cards!=CardAdmission::Complete{return Err(RunError::Unsupported("complete items require complete card admission"))}
        Ok(Self{cards,items})
    }
}

/// Finite native authored combat, independent of normal Run progression.
#[derive(Clone,Debug,PartialEq,Eq)]
pub enum M16DefectCombatStart { Canonical, Supplied(AuthoredRunStart) }

impl std::fmt::Debug for ActiveCombat {
    fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result {
        let mut d=f.debug_struct("ActiveCombat");d.field("visit",&self.visit).field("encounter",&self.encounter).field("origin",&self.origin).field("applied_gain_ordinal",&self.applied_gain_ordinal).field("acquired_instances",&self.acquired_instances);
        if self.applied_mutation_ordinal!=0{d.field("applied_mutation_ordinal",&self.applied_mutation_ordinal);}if self.applied_upgrade_ordinal!=0{d.field("applied_upgrade_ordinal",&self.applied_upgrade_ordinal);}d.finish()
    }
}

/// Accepted A0 setup metadata prefix at the M15 merge. Appends are not admission.
pub const M16_DEFECT_COMBAT_SETUP_PREFIX_LEN:usize=57;
pub const M16_DEFECT_COMBAT_SETUP_BASE:&str="4d7e5be44fd1ce0167729ef1a4f8c9b17b9c5bdf";
