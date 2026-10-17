use std::collections::{BTreeMap, BTreeSet, VecDeque};
use super::content::*;
use super::rng::{RngError, StreamState};

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
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PowerEntry { pub id: PowerId, pub state: PowerState }
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
pub struct DamageEntry { pub base: i32, pub output: i32 }
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EnemyState {
    pub instance_id: EnemyHandle, pub enemy_id: EnemyKind, pub hp: i32, pub max_hp: i32,
    pub block: i32, pub powers: Vec<PowerEntry>, pub intent_id: MoveId, pub move_history: Vec<MoveId>,
    pub selected_move: MoveId, pub damage_entries: BTreeMap<MoveId, DamageEntry>, pub intent_damage: Option<i32>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase { Decision, PlayerAction, EnemyTurn, RoundEnd, PlayerStart, Terminal }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome { Victory, Death }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransitionStatus { Decision, Terminal, Fault }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FaultKind { Unsupported, Invariant, EffectGuard, RngExhausted }
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FaultRecord { pub kind: FaultKind, pub message: String, pub resolved_effects: u32 }
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
pub enum Source { None, Card(CardHandle), Actor(Actor) }
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DamagePayload { Fixed(i32), EnemyMove(MoveId) }
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Effect { pub source: Source, pub target: Option<Actor>, pub kind: EffectKind }
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EffectKind {
    Damage(DamagePayload), GainBlock(i32), ApplyPower { id: PowerId, amount: i32 },
    ReducePower { id: PowerId, amount: i32 }, RemovePower(PowerId), FinishCard, Decision,
    Terminal(Outcome), Draw(i32), Recycle, DiscardHand, EndPlayerTurn, MonsterStart,
    EnemyMove, RollMove, EndRound,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublicKnowledge { pub revealed_order: Vec<CardHandle> }

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct State {
    pub content_id: String, pub phase: Phase, pub turn: u32, pub player: PlayerState,
    pub enemies: Vec<EnemyState>, pub piles: Piles, pub card_ids_issued: BTreeSet<CardHandle>,
    pub rng: CombatRng, pub pending: VecDeque<Effect>, pub current: Option<Effect>,
    pub in_play: Option<CardInstance>, pub resolved_effects: u32, pub public: PublicKnowledge,
    pub outcome: Option<Outcome>, pub fault: Option<FaultRecord>,
    pub hp_before_healing: Option<i32>, pub hp_after_cleanup: Option<i32>,
}

pub(super) fn mark_fault(state: &mut State, kind: FaultKind, message: String) {
    if state.fault.is_none() {
        state.fault = Some(FaultRecord { kind, message, resolved_effects: state.resolved_effects });
    }
}
