use std::collections::{BTreeMap, BTreeSet, VecDeque};
use super::content::*;
use super::rng::{RngError, StreamState};
use crate::content::EnemyId;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CardHandle(String);

impl CardHandle {
    pub fn parse(text: &str) -> Result<Self, InputError> {
        let digits = text.strip_prefix('-').unwrap_or(text).as_bytes();
        if text != "0" && (digits.is_empty() || !(b'1'..=b'9').contains(&digits[0])
            || !digits.iter().all(u8::is_ascii_digit)) {
            return Err(InputError::Invariant("card handle must be canonical signed decimal".into()));
        }
        Ok(Self(text.into()))
    }

    pub fn as_str(&self) -> &str { &self.0 }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct EnemyHandle(String);

impl EnemyHandle {
    pub fn parse(text: &str) -> Result<Self, InputError> {
        if text.is_empty() || text == "player" {
            return Err(InputError::Invariant("empty or reserved enemy handle".into()));
        }
        Ok(Self(text.into()))
    }

    pub fn as_str(&self) -> &str { &self.0 }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CardInstance { pub instance_id: CardHandle, pub card_id: CardKind, pub upgraded: bool }
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PowerState { pub amount: i32, pub skip_first: bool, pub monster_source_grace: bool }
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub(super) struct CombustState { pub copies:u16 }
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PowerEntry { pub id: PowerId, pub state: PowerState }
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FullIntentId { Incantation, Attack, Chomp, Thrash, Bellow, Bite, Grow, SpitWeb,
    GremlinNobSkullBash, GremlinNobBullRush,
    AcidTackle, AcidSpit, AcidLick, AcidSplit,
    SpikeTackle, SpikeFlameTackle, SpikeLick, SpikeSplit,
    SlimeBossSticky, SlimeBossPrepSlam, SlimeBossSlam, SlimeBossSplit,
    HeartDebilitate, HeartBloodShots, HeartEcho, HeartBuff,
    Unknown(&'static str) }
impl FullIntentId {
    pub fn from_name(id: &'static str) -> Self { match id {
        "incantation" => Self::Incantation, "attack" => Self::Attack, "chomp" => Self::Chomp,
        "thrash" => Self::Thrash, "bellow" => Self::Bellow, "bite" => Self::Bite,
        "grow" => Self::Grow, "spit_web" => Self::SpitWeb,
        "skull_bash" => Self::GremlinNobSkullBash, "bull_rush" => Self::GremlinNobBullRush,
        "acid_tackle" => Self::AcidTackle, "acid_spit" => Self::AcidSpit,
        "acid_lick" => Self::AcidLick, "acid_split" => Self::AcidSplit,
        "spike_tackle" => Self::SpikeTackle, "spike_flame_tackle" => Self::SpikeFlameTackle,
        "spike_lick" => Self::SpikeLick, "spike_split" => Self::SpikeSplit,
        "sticky" => Self::SlimeBossSticky, "prep_slam" => Self::SlimeBossPrepSlam,
        "slam" => Self::SlimeBossSlam, "split" => Self::SlimeBossSplit,
        "heart_debilitate"=>Self::HeartDebilitate,"heart_blood_shots"=>Self::HeartBloodShots,"heart_echo"=>Self::HeartEcho,"heart_buff"=>Self::HeartBuff,
        _ => Self::Unknown(id),
    } }
    pub const fn as_str(self) -> &'static str { match self {
    Self::Incantation => "incantation", Self::Attack => "attack", Self::Chomp => "chomp",
    Self::Thrash => "thrash", Self::Bellow => "bellow", Self::Bite => "bite",
        Self::Grow => "grow", Self::SpitWeb => "spit_web",
        Self::GremlinNobSkullBash => "skull_bash", Self::GremlinNobBullRush => "bull_rush",
        Self::AcidTackle => "acid_tackle", Self::AcidSpit => "acid_spit",
        Self::AcidLick => "acid_lick", Self::AcidSplit => "acid_split",
        Self::SpikeTackle => "spike_tackle", Self::SpikeFlameTackle => "spike_flame_tackle",
        Self::SpikeLick => "spike_lick", Self::SpikeSplit => "spike_split",
        Self::SlimeBossSticky => "sticky", Self::SlimeBossPrepSlam => "prep_slam",
        Self::SlimeBossSlam => "slam", Self::SlimeBossSplit => "split",
        Self::HeartDebilitate=>"heart_debilitate",Self::HeartBloodShots=>"heart_blood_shots",Self::HeartEcho=>"heart_echo",Self::HeartBuff=>"heart_buff",
        Self::Unknown(id) => id,
    } }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FullPowerId(pub &'static str);
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FullPowerEntry { pub id: FullPowerId, pub state: PowerState }
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ActorEnemyId { Legacy(EnemyKind), Full(EnemyId) }
impl ActorEnemyId {
    pub fn legacy(&self) -> Option<EnemyKind> { if let Self::Legacy(id) = self { Some(*id) } else { None } }
    pub fn as_str(&self) -> &'static str { match self { Self::Legacy(id) => id.as_str(), Self::Full(id) => id.as_str() } }
}
impl PartialEq<EnemyKind> for ActorEnemyId { fn eq(&self, other: &EnemyKind) -> bool { self.legacy() == Some(*other) } }
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ActorIntentId { Legacy(MoveId), Full(FullIntentId) }
impl ActorIntentId {
    pub fn legacy(&self) -> Option<MoveId> { if let Self::Legacy(id) = self { Some(*id) } else { None } }
    pub fn as_str(&self) -> &'static str { match self { Self::Legacy(id) => id.as_str(), Self::Full(id) => id.as_str() } }
}
impl PartialEq<MoveId> for ActorIntentId { fn eq(&self, other: &MoveId) -> bool { self.legacy() == Some(*other) } }
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlayerState {
    pub hp: i32, pub max_hp: i32, pub block: i32, pub energy: i32,
    pub max_energy: i32, pub draw_per_turn: i32, pub powers: Vec<PowerEntry>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EnemySetup {
    pub instance_id: EnemyHandle, pub enemy_id: EnemyKind, pub hp: i32, pub max_hp: i32,
    pub block: i32, pub powers: Vec<PowerEntry>, pub intent_id: MoveId, pub move_history: Vec<MoveId>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Piles {
    pub hand: Vec<CardInstance>, pub draw: Vec<CardInstance>,
    pub discard: Vec<CardInstance>, pub exhaust: Vec<CardInstance>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum OwnedCard { Legacy(CardInstance), Full(super::card_state::LiveCard) }

impl OwnedCard {
    pub(super) fn handle(&self) -> CardHandle {
        match self {
            Self::Legacy(card) => card.instance_id.clone(),
            Self::Full(card) => super::card_state::handle(card.instance),
        }
    }
    pub(super) fn legacy(&self) -> Option<&CardInstance> {
        if let Self::Legacy(card) = self { Some(card) } else { None }
    }
    pub(super) fn full(&self) -> Option<&super::card_state::LiveCard> {
        if let Self::Full(card) = self { Some(card) } else { None }
    }
    pub(super) fn full_mut(&mut self) -> Option<&mut super::card_state::LiveCard> {
        if let Self::Full(card) = self { Some(card) } else { None }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct OwnedPiles {
    pub hand: Vec<OwnedCard>, pub draw: Vec<OwnedCard>,
    pub discard: Vec<OwnedCard>, pub exhaust: Vec<OwnedCard>,
}
impl OwnedPiles {
    pub(super) fn from_legacy(piles: &Piles) -> Self {
        let wrap = |pile: &[CardInstance]| pile.iter().cloned().map(OwnedCard::Legacy).collect();
        Self { hand: wrap(&piles.hand), draw: wrap(&piles.draw),
            discard: wrap(&piles.discard), exhaust: wrap(&piles.exhaust) }
    }
    pub(super) fn legacy_projection(&self) -> Option<Piles> {
        let project = |pile: &[OwnedCard]| pile.iter().map(|card| card.legacy().cloned()).collect::<Option<Vec<_>>>();
        Some(Piles { hand: project(&self.hand)?, draw: project(&self.draw)?,
            discard: project(&self.discard)?, exhaust: project(&self.exhaust)? })
    }
    pub(super) fn all<'a>(&'a self, in_play: Option<&'a OwnedCard>) -> impl Iterator<Item=&'a OwnedCard> {
        self.hand.iter().chain(&self.draw).chain(&self.discard).chain(&self.exhaust).chain(in_play)
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CombatRng {
    pub contract_id: String, pub shuffle: StreamState, pub enemy_ai: StreamState,
    pub enemy_instance_id: EnemyHandle,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Scenario {
    pub encounter_id: EnemyKind, pub player: PlayerState, pub enemies: Vec<EnemySetup>,
    pub piles: Piles, pub rng: CombatRng,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FullEnemySetup {
    pub initial_memory:Option<super::enemies::EnemyMemory>,
    pub instance_id: EnemyHandle, pub enemy_id: EnemyId, pub ascension: u8,
    pub profile: super::enemies::EnemyProfile, pub hp: i32, pub max_hp: i32,
    pub block: i32, pub intent: FullIntentId, pub powers: Vec<FullPowerEntry>, pub ai_rng: StreamState,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FullCombatSetup {
    pub player: PlayerState, pub player_powers: Vec<FullPowerEntry>,
    pub character: crate::content::Character, pub card_rng: StreamState,
    pub shuffle_rng: StreamState, pub enemies: Vec<FullEnemySetup>,
    /// Per-actor selected bases produced by encounter setup, keyed by ordered formation slot and intent.
    pub enemy_damage_bases: BTreeMap<(u32, FullIntentId), i32>,
}
/// Full entry has typed actors/streams and never needs placeholder legacy rows.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CombatEntry { pub setup: FullCombatSetup, pub opening_draw: usize, pub cards: Vec<super::card_state::LiveCard> }
#[derive(Clone,Debug,PartialEq,Eq)]
pub(crate) struct PermanentCardGain{pub ordinal:u64,pub id:crate::content::CardId,pub upgrades:u32}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DamageEntry { pub base: i32, pub output: i32 }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActorLife { Active, Replaced, Escaped, HalfDead }
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum PlayerFacing { Left, Right }
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum EncounterSide { Left, Right }

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EnemyState {
    pub instance_id: EnemyHandle, pub enemy_id: ActorEnemyId, pub hp: i32, pub max_hp: i32,
    pub block: i32, pub powers: Vec<PowerEntry>, pub full_powers: Vec<FullPowerEntry>,
    pub intent_id: ActorIntentId, pub move_history: Vec<ActorIntentId>,
    pub selected_move: ActorIntentId, pub damage_entries: BTreeMap<MoveId, DamageEntry>,
    pub full_damage_entries: BTreeMap<FullIntentId, DamageEntry>, pub full_ai_rng: Option<StreamState>,
    pub full_memory: Option<super::enemies::EnemyMemory>,
    pub intent_damage: Option<i32>,
    pub life: ActorLife,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EnemyReplacementDiagnostics {
    pub parent: EnemyHandle,
    pub next_child: usize,
    pub prepared_children: Vec<EnemyHandle>,
    pub latest_child_cursor: Option<u128>,
}
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct SpawnRequest {pub summoner:EnemyHandle,pub slot:u8,pub species:EnemyId}
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct EnemySpawnDiagnostics {pub request:SpawnRequest,pub ordinal:u64,pub prepared_child:Option<EnemyHandle>,pub constructor_cursor:u128,pub latest_child_cursor:u128}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase { Decision, Selection, PlayerAction, EnemyTurn, RoundEnd, PlayerStart, Terminal }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome { Victory, Death }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransitionStatus { Decision, Selection, Terminal, Fault }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FaultKind { Unsupported, Invariant, EffectGuard, RngExhausted }
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FaultRecord { pub kind: FaultKind, pub message: String, pub resolved_effects: u32 }
/// Lossless native diagnostics attached to the shared run fault boundary.
/// Contains only attempted/queued work, never private RNG stream state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CombatFault {
    pub record: FaultRecord,
    pub current: Option<Effect>,
    pub pending: VecDeque<Effect>,
    pub attempted_action: Option<super::CombatAction>,
    pub selection: Option<PendingSelection>,
    pub replacement_progress: Option<EnemyReplacementDiagnostics>,
    pub spawn_progress:Option<EnemySpawnDiagnostics>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InputError { Invariant(String), Unsupported(String) }
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EffectError { Invalid(InputError), Random(RngError) }

impl From<InputError> for EffectError {
    fn from(error: InputError) -> Self { Self::Invalid(error) }
}

impl From<RngError> for EffectError {
    fn from(error: RngError) -> Self { Self::Random(error) }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StepError { InvalidAction, NotReady, Terminal, Faulted }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AccessError { NotReady, Faulted }
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RestoreError { Compatibility, Faulted, Invalid(InputError) }
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action { PlayCard { card_instance_id: CardHandle, target_id: Option<EnemyHandle> }, EndTurn }
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Transition { pub status: TransitionStatus }
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Actor { Player, Enemy(EnemyHandle) }
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Source { None, Card(CardHandle), Potion{slot:usize,id:crate::content::PotionId}, Actor(Actor) }
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DamagePayload {
    Fixed(i32), Orb(i32),
    /// Player-owned THORNS damage; distinct from NORMAL attack damage and HP_LOSS.
    Thorns(i32),
    /// HP_LOSS bypasses Block and NORMAL-only modifiers/triggers.
    HpLoss(i32),
    EnemyMove(MoveId), FullEnemyMove(FullIntentId),
}
/// Detached result from the single shared damage commit, not a second calculator.
#[derive(Clone,Debug,PartialEq,Eq)]
pub(super) struct DamageResult {pub source:Source,pub target:Actor,pub normal:bool,pub hp_loss:bool,pub thorns:bool,pub attempted:i32,pub blocked:i32,pub hp_lost:i32,pub hp_after:i32}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Effect { pub source: Source, pub target: Option<Actor>, pub kind: EffectKind }
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EffectKind {
    Damage(DamagePayload), GainBlock(i32), Heal(i32), IncreaseMaxHpThenHeal(i32), NeowsLament, ApplyPower { id: PowerId, amount: i32 },
    ReducePower { id: PowerId, amount: i32 }, RemovePower(PowerId),
    ApplyFullPower { id: FullPowerId, amount: i32 }, ReduceFullPower { id: FullPowerId, amount: i32 }, RemoveFullPower(FullPowerId),
    FinishCard, Decision, JuggernautRandomDamage(i32),
    Terminal(Outcome), Draw(i32), Recycle, DiscardHand,
    DrawOne, AfterCardDrawn { card:CardHandle },
    GainEnergy(i32),LoseEnergy(i32),
    CreateCard { id: crate::content::CardId, destination: super::command::CardDestination, upgrades: u32 },
    CopyCard { source: CardHandle, destination: super::command::CardDestination },
    ExhaustHandCard { instance: super::CardInstanceId },
    ExhaustBatchStart { filter: ExhaustBatchFilter, random: bool, finish: ExhaustBatchFinish },
    ExhaustBatchContinue(ExhaustBatch),
    SelectCard { candidates: Vec<CardHandle>, empty: super::selection::EmptySelection },
    SelectCards { candidates: Vec<CardHandle>, required: usize,
        disposition: super::selection::SelectionDisposition, empty: super::selection::EmptySelection },
    /// Capture the current hand only when this queued effect executes (after preceding draws).
    SelectHandCardsAfterDraw { required: usize,
        disposition: super::selection::SelectionDisposition, empty: super::selection::EmptySelection },
    Hook(super::hooks::Hook), EndPlayerTurn, MonsterStart,
    EnemyMove, RollMove, EndRound, StartNextPlayer,
    Character(super::characters::CharacterEffect),
    ShieldBashFocus,
    CardContinuation(super::cards::DefectContinuation),
    Ironclad(super::cards::IroncladEffect),
    ReplaceEnemy { parent: EnemyHandle, children: Vec<EnemyId> },
    GuardianChange(super::enemies::GuardianEffect),
    HexaghostBurn(super::enemies::HexaghostEffect),
    GroundByrd,
    SpawnEnemy(SpawnRequest), CompleteSpawn,
    RallyGremlins { remaining:u8 }, ProtectGremlin, SetEnemyMove { intent:FullIntentId }, EscapeEnemy,
    BookStab { remaining:u32 },
    ChampCleanse,
    CollectorSuicide,
    AutomatonSuicide,
    ShapeSuicide,ShapeDamageComplete,
    RevivalHeal(i32),DarklingCollapse,DarklingRevive,DarklingsDefeat,
    TransientFade,TransientRecover,MawNom{remaining:u32},
    ObtainParasite,ResetMalleable,
    ReptomancerSuicide,
    AwakenedCollapse,AwakenedRebirth,AwakenedCanLose,
    TimeWarpEndTurn,TimeEaterCleanse,
    CaptureStasis { orb:EnemyHandle }, ReleaseStasis { orb:EnemyHandle }, CorruptHeartBuff,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublicKnowledge { pub revealed_order: Vec<CardHandle>,pub known_draw:Vec<CardHandle>,pub known_draw_bottom:Vec<CardHandle> }

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PendingSelection { pub parent: super::selection::SelectionParent, pub source_zone:super::selection::SelectionSourceZone, pub candidates: Vec<CardHandle>,
    pub operation: super::selection::SelectionOperation,
    pub selected: Vec<CardHandle>, pub minimum:usize,pub maximum:usize,
    pub disposition: super::selection::SelectionDisposition, pub empty: super::selection::EmptySelection }

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub(super) enum FinishDisposition { Natural, Exhaust, Purge }

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum ExhaustBatchFilter { NonAttack, AllHand }
#[derive(Clone,Debug,PartialEq,Eq)]
pub enum ExhaustBatchFinish {
    BlockPerExhaust(i32),
    DamagePerExhaust{target:Actor,damage:i32},
    DamageAfterBatch{target:Actor,damage:i32},
}
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct ExhaustBatch {
    pub owner:CardHandle,pub filter:ExhaustBatchFilter,pub candidates:Vec<CardHandle>,pub random:bool,pub remaining:u8,
    pub committed:u32,pub finish:ExhaustBatchFinish,
}

#[derive(Clone,Debug,PartialEq,Eq)]
pub(super) struct PlayContext {pub card:CardHandle,pub target:Option<Actor>,pub energy_on_use:i32,pub paid:i32,pub disposition:FinishDisposition}
#[derive(Clone,Debug,PartialEq,Eq)]
pub(super) struct PlayFrame { pub parent:Option<OwnedCard>,pub parent_handle:CardHandle,pub parent_context:Option<PlayContext>,pub child:CardHandle,pub target:Option<Actor>,pub captured_energy:i32,pub paid:i32,pub disposition:FinishDisposition,pub parent_finish_checkpoint_done:bool }

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct State {
    pub content_id: String, pub phase: Phase, pub turn: u32, pub player: PlayerState,
    pub enemies: Vec<EnemyState>, pub piles: OwnedPiles, pub card_ids_issued: BTreeSet<CardHandle>,
    pub rng: CombatRng, pub pending: VecDeque<Effect>, pub current: Option<Effect>,
    pub in_play: Option<OwnedCard>, pub resolved_effects: u32, pub public: PublicKnowledge,
    pub outcome: Option<Outcome>, pub fault: Option<FaultRecord>,
    pub hp_before_healing: Option<i32>, pub hp_after_cleanup: Option<i32>,
    pub full_runtime: bool,
    pub pending_selection: Option<PendingSelection>,
    pub hook_events: Vec<super::hooks::HookEvent>,
    pub attempted_action: Option<super::CombatAction>,
    pub fault_selection: Option<PendingSelection>,
    pub player_full_powers: Vec<FullPowerEntry>,
    pub character: super::characters::CharacterState,
    pub card_rng: Option<StreamState>,
    pub item_context: Option<crate::items::CombatItemContext>,
    pub replacement_progress: Option<super::enemies::ReplacementProgress>,
    pub actor_slots:BTreeMap<EnemyHandle,u8>,
    pub facing:Option<PlayerFacing>,
    pub spawn_ordinals:BTreeMap<EnemyHandle,u64>,
    pub spawn_progress:Option<super::enemies::act2::spawn::SpawnProgress>,
    pub stasis:BTreeMap<EnemyHandle,OwnedCard>,
    pub permanent_gains:Vec<PermanentCardGain>,pub next_permanent_gain:u64,
    pub played_powers:Vec<OwnedCard>,
    pub inactive_cards:Vec<OwnedCard>,
    pub play_frames:Vec<PlayFrame>,
    pub play_context:Option<PlayContext>,
    pub queued_autoplay:BTreeMap<CardHandle,OwnedCard>,
    pub pending_time_warp:Option<EnemyHandle>,
    pub damage_events_this_combat:u32,
    /// Combust stacks are counted independently of the summed damage amount.
    pub combust:Option<CombustState>,
}

pub(super) fn mark_fault(state: &mut State, kind: FaultKind, message: String) {
    if state.fault.is_none() {
        state.fault = Some(FaultRecord { kind, message, resolved_effects: state.resolved_effects });
    }
}
