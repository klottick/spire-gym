//! Detached public values; never inspect private random or future-AI state.
use super::*;
use super::model::State;

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
    for card in &state.piles.hand {
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

fn cards(entries: &[CardInstance]) -> Vec<CardView> {
    entries.iter().map(|c| CardView { instance_id: c.instance_id.clone(),
        card_id: c.card_id, upgraded: c.upgraded }).collect()
}

pub(super) fn public_view(state: &State) -> Result<PublicView, AccessError> {
    ready(state)?;
    let p = &state.player;
    Ok(PublicView { content_id: state.content_id.clone(), turn: state.turn,
        player: PlayerView { hp: p.hp, max_hp: p.max_hp, block: p.block, energy: p.energy,
            max_energy: p.max_energy, powers: powers(&p.powers) },
        enemies: state.enemies.iter().map(|e| EnemyView { instance_id: e.instance_id.clone(),
            enemy_id: e.enemy_id, hp: e.hp, max_hp: e.max_hp, block: e.block,
            powers: powers(&e.powers), intent_id: e.intent_id, intent_damage: e.intent_damage }).collect(),
        hand: cards(&state.piles.hand), draw_count: state.piles.draw.len(),
        discard: cards(&state.piles.discard), exhaust: cards(&state.piles.exhaust),
        revealed_order: state.public.revealed_order.clone(), outcome: state.outcome,
        hp_before_healing: state.hp_before_healing, hp_after_cleanup: state.hp_after_cleanup })
}
