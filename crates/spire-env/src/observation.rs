//! Fixed numeric storage and public-only encoding. No snapshot or draw-order access.
use spire_core::combat::{CardInstance, Combat, EnemyKind, Outcome, PowerId, PublicProjection, card_stats};
use crate::{CardSpec, EnvError, FaultStage, InputReason, PublicContext, SlotFault};
use crate::actions::CandidateMenu;
use crate::error::input;
use crate::knowledge::{PublicKnowledge, card_index, intent_id};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ObservationRow {
    pub player: [i32;7],
    pub player_powers: [i32;3],
    pub hand_kind: [i32;10],
    pub hand_upgraded: [i8;10],
    pub hand_features: [[i32;4];10],
    pub hand_mask: [i8;10],
    pub hand_playable: [i8;10],
    pub enemy_kind: [i32;1],
    pub enemy: [[i32;4];1],
    pub enemy_powers: [[i32;3];1],
    pub enemy_intent: [i32;1],
    pub enemy_damage_mask: [i8;1],
    pub enemy_history: [[i32;2];1],
    pub enemy_history_mask: [[i8;2];1],
    pub deck_counts: [[i32;2];3],
    pub pile_counts: [[[i32;2];3];3],
    pub in_play_counts: [[i32;2];3],
    pub relic_presence: [i8;1],
    pub context: [i32;4],
    pub context_mask: [i8;4],
    pub outcome: [i32;1],
    pub action_kind: [i32;32],
    pub action_source: [i32;32],
    pub action_target: [i32;32],
    pub action_features: [[i32;4];32],
    pub action_mask: [i8;32],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TensorValues { I32(Vec<i32>), I8(Vec<i8>) }

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NamedTensor {
    pub key: &'static str,
    pub shape: Vec<usize>,
    pub values: TensorValues,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObservationBatch {
    pub rows: usize,
    pub tensors: Vec<NamedTensor>,
}

// Recursive fixed arrays flatten directly into their final owned typed buffer.
trait Flatten<T> { fn append(&self, values: &mut Vec<T>); }
impl Flatten<i32> for i32 { fn append(&self, values: &mut Vec<i32>) { values.push(*self); } }
impl Flatten<i8> for i8 { fn append(&self, values: &mut Vec<i8>) { values.push(*self); } }
impl<T, A: Flatten<T>, const N: usize> Flatten<T> for [A;N] {
    fn append(&self, values: &mut Vec<T>) { for a in self { a.append(values); } }
}

impl ObservationBatch {
    pub fn from_rows(rows: &[ObservationRow]) -> Result<Self, EnvError> {
        let mut tensors = crate::error::reserved(26)?;
        macro_rules! export {
            ($key:ident, $variant:ident, $($dim:expr),+) => {{
                let single: usize = 1 $( * $dim )+;
                let len = rows.len().checked_mul(single)
                    .ok_or_else(|| crate::error::setup("observation batch size overflow"))?;
                let mut values = crate::error::reserved(len)?;
                for row in rows { row.$key.append(&mut values); }
                tensors.push(NamedTensor { key: stringify!($key), shape: vec![rows.len(), $($dim),+],
                    values: TensorValues::$variant(values) });
            }};
        }
        export!(player,I32,7);
        export!(player_powers,I32,3);
        export!(hand_kind,I32,10);
        export!(hand_upgraded,I8,10);
        export!(hand_features,I32,10,4);
        export!(hand_mask,I8,10);
        export!(hand_playable,I8,10);
        export!(enemy_kind,I32,1);
        export!(enemy,I32,1,4);
        export!(enemy_powers,I32,1,3);
        export!(enemy_intent,I32,1);
        export!(enemy_damage_mask,I8,1);
        export!(enemy_history,I32,1,2);
        export!(enemy_history_mask,I8,1,2);
        export!(deck_counts,I32,3,2);
        export!(pile_counts,I32,3,3,2);
        export!(in_play_counts,I32,3,2);
        export!(relic_presence,I8,1);
        export!(context,I32,4);
        export!(context_mask,I8,4);
        export!(outcome,I32,1);
        export!(action_kind,I32,32);
        export!(action_source,I32,32);
        export!(action_target,I32,32);
        export!(action_features,I32,32,4);
        export!(action_mask,I8,32);
        Ok(Self { rows: rows.len(), tensors })
    }
}

pub(crate) fn encoding_error(message: impl Into<String>) -> EnvError {
    EnvError::Fault { records: vec![SlotFault { slot: None, stage: FaultStage::Encoding,
        message: message.into(), core_fault: None }] }
}

pub(crate) fn capacity(hand: usize, enemies: usize, actions: usize) -> Result<(), EnvError> {
    if hand > 10 || enemies > 1 || actions > 32 {
        return Err(input(InputReason::Capacity, "observation v1 capacity exceeded"));
    }
    Ok(())
}

pub(crate) fn features(card: &CardSpec) -> [i32;4] {
    let s = card_stats(card.kind, card.upgraded);
    [s.cost, s.damage, s.block, s.vulnerable]
}
fn counts(cards: &[CardInstance]) -> [[i32;2];3] {
    let mut result = [[0;2];3];
    for c in cards { result[card_index(c.card_id)][usize::from(c.upgraded)] += 1; }
    result
}
pub(crate) fn enemy_kind(kind: EnemyKind) -> i32 {
    match kind { EnemyKind::Cultist => 1, EnemyKind::JawWorm => 2 }
}
fn powers(entries: &[(PowerId, i32)]) -> [i32;3] {
    let mut result = [0;3];
    for &(id, value) in entries {
        result[match id { PowerId::Strength => 0, PowerId::Ritual => 1, PowerId::Vulnerable => 2 }] = value;
    }
    result
}

#[allow(dead_code)]
pub(crate) fn encode(combat: &Combat, knowledge: &PublicKnowledge, menu: &CandidateMenu,
    context: &PublicContext) -> Result<ObservationRow, EnvError> {
    let p = combat.public_projection().map_err(|e| encoding_error(format!("projection: {e:?}")))?;
    encode_projected(&p, knowledge, menu, context)
}

pub(crate) fn encode_projected(p: &PublicProjection<'_>, k: &PublicKnowledge,
    menu: &CandidateMenu, context: &PublicContext) -> Result<ObservationRow, EnvError> {
    capacity(p.hand.len(), p.enemies.len(), menu.rows().count())?;
    let e = p.enemies.first().ok_or_else(|| encoding_error("missing public enemy"))?;
    let mut row = ObservationRow { action_source: [-1;32], action_target: [-1;32], ..Default::default() };
    let turn = i32::try_from(p.turn).map_err(|_| encoding_error("turn exceeds int32"))?;
    row.player = [p.player.hp,p.player.max_hp,p.player.block,p.player.energy,p.player.max_energy,p.draw_per_turn,turn];
    row.player_powers = powers(&p.player.powers);
    for (i, c) in p.hand.iter().enumerate() {
        row.hand_kind[i] = card_index(c.card_id) as i32 + 1;
        row.hand_upgraded[i] = i8::from(c.upgraded);
        row.hand_mask[i] = 1;
        row.hand_features[i] = features(&CardSpec { kind: c.card_id, upgraded: c.upgraded });
    }
    row.enemy_kind = [enemy_kind(e.enemy_id)];
    row.enemy = [[e.hp,e.max_hp,e.block,e.intent_damage.unwrap_or(0)]];
    row.enemy_powers = [powers(&e.powers)];
    row.enemy_intent = [intent_id(e.intent_id)];
    row.enemy_damage_mask = [i8::from(e.intent_damage.is_some())];
    row.enemy_history = k.history;
    row.enemy_history_mask = k.history_mask;
    row.deck_counts = k.deck;
    row.in_play_counts = k.in_play;
    let hand = counts(p.hand);
    let discard = counts(p.discard);
    let exhaust = counts(p.exhaust);
    let mut draw = [[0;2];3];
    for kind in 0..3 { for upgrade in 0..2 {
        let n = i64::from(k.deck[kind][upgrade]) - i64::from(hand[kind][upgrade])
            - i64::from(discard[kind][upgrade]) - i64::from(exhaust[kind][upgrade])
            - i64::from(k.in_play[kind][upgrade]);
        draw[kind][upgrade] = i32::try_from(n).ok().filter(|&n| n >= 0)
            .ok_or_else(|| encoding_error("negative or overflowing known draw count"))?;
    } }
    if draw.iter().flatten().map(|&n| n as usize).sum::<usize>() != p.draw_count {
        return Err(encoding_error("known draw counts disagree with published draw length"));
    }
    row.pile_counts = [draw,discard,exhaust];
    row.relic_presence = [1];
    row.context = [1,0,i32::from(context.act.unwrap_or(0)),context.next_encounter.map(enemy_kind).unwrap_or(0)];
    row.context_mask = [1,1,i8::from(context.act.is_some()),i8::from(context.next_encounter.is_some())];
    row.outcome = [match p.outcome { None => 0, Some(Outcome::Victory) => 1, Some(Outcome::Death) => 2,Some(Outcome::Escaped)=>return Err(encoding_error("M15 escape requires M15 native profile")) }];
    if p.outcome.is_none() {
        for (i, (action, source, target)) in menu.rows().enumerate() {
            row.action_kind[i] = if action.source.is_some() { 2 } else { 1 };
            row.action_source[i] = source;
            row.action_target[i] = target;
            row.action_mask[i] = 1;
            if let Some(card) = &action.source {
                row.hand_playable[source as usize] = 1;
                row.action_features[i] = features(card);
            }
        }
    }
    Ok(row)
}
