//! Public values; never inspect private random or future-AI state.
use super::*;
use super::model::{OwnedCard, State};
use crate::content::EnemyId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CardView { pub instance_id: CardHandle, pub card_id: CardKind, pub upgraded: bool }
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlayerView {
    pub hp: i32, pub max_hp: i32, pub block: i32, pub energy: i32, pub max_energy: i32,
    pub powers: Vec<(PowerId, i32)>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EnemyView {
    pub instance_id: EnemyHandle, pub enemy_id: EnemyKind, pub hp: i32, pub max_hp: i32,
    pub block: i32, pub powers: Vec<(PowerId, i32)>, pub intent_id: MoveId, pub intent_damage: Option<i32>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublicView {
    pub content_id: String, pub turn: u32, pub player: PlayerView, pub enemies: Vec<EnemyView>,
    pub hand: Vec<CardView>, pub draw_count: usize, pub discard: Vec<CardView>, pub exhaust: Vec<CardView>,
    pub revealed_order: Vec<CardHandle>, pub outcome: Option<Outcome>,
    pub hp_before_healing: Option<i32>, pub hp_after_cleanup: Option<i32>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LiveCardView { pub instance: CardInstanceId, pub id: crate::content::CardId,
    pub origin: CombatCardRef, pub upgrades: u32 }
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectionView {
    pub candidates: Vec<LiveCardView>, pub required: usize,
    pub selected: Vec<CardInstanceId>, pub disposition: super::SelectionDisposition,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CombatInventoryView {
    pub relic_counters:Vec<crate::items::RelicCounterView>,
    pub relics: Vec<crate::content::RelicId>, pub potions: Vec<Option<crate::content::PotionId>>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CombatPublic {
    pub content_id: String, pub turn: u32, pub player: FullPlayerView, pub enemies: Vec<FullEnemyView>,
    pub character: CharacterCombatView,
    pub hand: Vec<LiveCardView>, pub draw_count: usize, pub discard: Vec<LiveCardView>,
    pub exhaust: Vec<LiveCardView>, pub outcome: Option<Outcome>,
    pub hp_before_healing: Option<i32>, pub hp_after_cleanup: Option<i32>,
    pub targets: Vec<(TargetId, bool)>,
    pub selection: Option<SelectionView>,
    pub inventory: Option<CombatInventoryView>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FullPowerView { pub id: super::FullPowerId, pub amount: i32 }
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FullPlayerView {
    pub hp: i32, pub max_hp: i32, pub block: i32, pub energy: i32, pub max_energy: i32,
    pub powers: Vec<FullPowerView>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FullEnemyView {
    pub instance_id: EnemyHandle, pub enemy_id: EnemyId, pub hp: i32, pub max_hp: i32,
    pub block: i32, pub powers: Vec<FullPowerView>, pub intent: super::FullIntentId,
    pub intent_damage: Option<i32>, pub replaced: bool,
}

/// Native combat-to-run handoff. It contains only actual full-run cards and
/// persistent combat result/resource values; run settlement remains a later owner.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CombatExit {
    pub hp: i32, pub max_hp: i32, pub block: i32, pub energy: i32,
    pub outcome: Outcome, pub hand: Vec<LiveCard>, pub draw: Vec<LiveCard>,
    pub discard: Vec<LiveCard>, pub exhaust: Vec<LiveCard>, pub in_play: Option<LiveCard>,
    pub(crate) item_context: Option<crate::items::CombatItemContext>,
}

/// Bounded public actor values and borrowed visible piles at a ready boundary.
/// No draw order, reveal history, private lifecycle flags or future selections.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublicProjection<'a> {
    pub turn: u32, pub player: PlayerView, pub enemies: Vec<EnemyView>,
    pub draw_per_turn: i32, pub draw_count: usize,
    pub hand: &'a [CardInstance], pub discard: &'a [CardInstance], pub exhaust: &'a [CardInstance],
    pub outcome: Option<Outcome>,
    pub hp_before_healing: Option<i32>, pub hp_after_cleanup: Option<i32>,
}

impl Combat {
    /// Read public values without copying visible piles or growing reveal history.
    /// Decision and terminal boundaries use the same access checks as `public_view`.
    pub fn public_projection(&self) -> Result<PublicProjection<'_>, AccessError> {
        if self.state.full_runtime { return Err(AccessError::NotReady); }
        let state = &self.state;
        ready(state)?;
        Ok(PublicProjection { turn: state.turn, player: player(state), enemies: enemies(state),
            draw_per_turn: state.player.draw_per_turn, draw_count: state.piles.draw.len(),
            hand: &self.legacy_cache.hand, discard: &self.legacy_cache.discard, exhaust: &self.legacy_cache.exhaust,
            outcome: state.outcome, hp_before_healing: state.hp_before_healing,
            hp_after_cleanup: state.hp_after_cleanup })
    }

    /// Explicit diagnostic access to a detached first-fault summary, including while faulted.
    pub fn fault_record(&self) -> Option<FaultRecord> {
        self.state.fault.clone()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CharacterCombatView {
    pub stance: super::characters::Stance,
    pub orbs: Vec<Option<super::characters::OrbKind>>,
    pub focus: i32,
}

pub(super) fn full_player(state: &State) -> FullPlayerView {
    let p = &state.player;
    let mut powers = p.powers.iter().map(|p| FullPowerView {
        id: super::FullPowerId(p.id.as_str()), amount: p.state.amount,
    }).chain(state.player_full_powers.iter().map(|p| FullPowerView { id: p.id, amount: p.state.amount })).collect::<Vec<_>>();
    powers.sort_by_key(|p| p.id.0);
    FullPlayerView { hp: p.hp, max_hp: p.max_hp, block: p.block, energy: p.energy,
        max_energy: p.max_energy, powers }
}

pub(super) fn full_enemies(state: &State) -> Vec<FullEnemyView> {
    state.enemies.iter().map(|e| FullEnemyView {
        instance_id: e.instance_id.clone(),
        enemy_id: match e.enemy_id { super::ActorEnemyId::Full(id) => id,
            super::ActorEnemyId::Legacy(id) => match id { EnemyKind::Cultist => EnemyId::Cultist, EnemyKind::JawWorm => EnemyId::JawWorm } },
        hp: e.hp, max_hp: e.max_hp, block: e.block,
        powers: e.powers.iter().map(|p| FullPowerView { id: super::FullPowerId(p.id.as_str()), amount: p.state.amount })
            .chain(e.full_powers.iter().map(|p| FullPowerView { id: p.id, amount: p.state.amount }))
            .collect(),
        intent: match e.intent_id { super::ActorIntentId::Full(id) => id,
            super::ActorIntentId::Legacy(id) => super::FullIntentId::from_name(id.as_str()) },
        intent_damage: e.intent_damage, replaced: e.life == super::ActorLife::Replaced,
    }).collect()
}

pub(super) fn ready(state: &State) -> Result<(), AccessError> {
    if state.fault.is_some() { return Err(AccessError::Faulted); }
    if !matches!(state.phase, Phase::Decision | Phase::Terminal)
        || state.current.is_some() || !state.pending.is_empty()
        || (state.phase == Phase::Decision && state.in_play.is_some()) {
        return Err(AccessError::NotReady);
    }
    Ok(())
}

pub(super) fn legal_actions(state: &State) -> Result<Vec<Action>, AccessError> {
    ready(state)?;
    if state.phase == Phase::Terminal { return Ok(vec![]); }
    let mut actions = vec![];
    for owned in &state.piles.hand {
        let Some(card) = owned.legacy() else { continue; };
        if card_stats(card.card_id, card.upgraded).cost > state.player.energy { continue; }
        let target_id = match card.card_id {
            CardKind::Defend => None,
            CardKind::Strike | CardKind::Bash => {
                let Some(enemy) = state.enemies.iter().find(|e| e.hp > 0) else { continue; };
                Some(enemy.instance_id.clone())
            }
        };
        actions.push(Action::PlayCard { card_instance_id: card.instance_id.clone(), target_id });
    }
    actions.push(Action::EndTurn);
    Ok(actions)
}

fn powers(entries: &[PowerEntry]) -> Vec<(PowerId, i32)> {
    let mut values: Vec<_> = entries.iter().map(|p| (p.id, p.state.amount)).collect();
    values.sort_by_key(|(id, _)| id.as_str());
    values
}

fn cards(entries: &[OwnedCard]) -> Result<Vec<CardView>, AccessError> {
    entries.iter().map(|owned| owned.legacy().map(|c| CardView { instance_id: c.instance_id.clone(),
        card_id: c.card_id, upgraded: c.upgraded }).ok_or(AccessError::NotReady)).collect()
}

pub(super) fn player(state: &State) -> PlayerView {
    let p = &state.player;
    PlayerView { hp: p.hp, max_hp: p.max_hp, block: p.block, energy: p.energy,
        max_energy: p.max_energy, powers: powers(&p.powers) }
}

pub(super) fn enemies(state: &State) -> Vec<EnemyView> {
    state.enemies.iter().map(|e| EnemyView { instance_id: e.instance_id.clone(),
        enemy_id: e.enemy_id.legacy().expect("legacy public view only receives legacy actors"), hp: e.hp, max_hp: e.max_hp, block: e.block,
        powers: powers(&e.powers), intent_id: e.intent_id.legacy().expect("legacy public view only receives legacy intents"),
        intent_damage: e.intent_damage }).collect()
}

pub(super) fn public_view(state: &State) -> Result<PublicView, AccessError> {
    ready(state)?;
    Ok(PublicView { content_id: state.content_id.clone(), turn: state.turn,
        player: player(state), enemies: enemies(state),
         hand: cards(&state.piles.hand)?, draw_count: state.piles.draw.len(),
         discard: cards(&state.piles.discard)?, exhaust: cards(&state.piles.exhaust)?,
        revealed_order: state.public.revealed_order.clone(), outcome: state.outcome,
        hp_before_healing: state.hp_before_healing, hp_after_cleanup: state.hp_after_cleanup })
}
