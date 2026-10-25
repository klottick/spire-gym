//! Character-local stance and orb mechanics used by the shared combat resolver.
use super::{Actor, DamagePayload, Effect, EffectKind, Source};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stance { Neutral, Wrath, Calm }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OrbKind { Lightning, Frost }

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CharacterState {
    pub stance: Stance,
    pub orb_slots: Vec<Option<OrbKind>>,
    pub focus: i32,
}

impl CharacterState {
    pub fn for_character(character: crate::content::Character) -> Self {
        let orb_slots = if character == crate::content::Character::Defect { vec![None; 3] } else { Vec::new() };
        Self { stance: Stance::Neutral, orb_slots, focus: 0 }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CharacterEffect { Stance(Stance), Focus(i32), Channel(OrbKind), EvokeFront { remove: bool }, EvokeAt { index: usize }, EndTurn }

pub(super) fn enqueue(state: &mut super::model::State, source: Source, action: CharacterEffect) {
    state.pending.push_front(Effect { source, target: Some(Actor::Player),
        kind: EffectKind::Character(action) });
}

pub(super) fn apply(state: &mut super::model::State, source: &Source,
    action: CharacterEffect) -> Result<(), super::EffectError>
{
    match action {
        CharacterEffect::Focus(delta) => {
            if !state.character.orb_slots.iter().any(Option::is_some){return Err(super::InputError::Invariant("Focus adjustment without an active Orb".into()).into())}
            if delta<0 {
                state.pending.push_front(Effect{source:source.clone(),target:Some(Actor::Player),
                    kind:EffectKind::ApplyFullPower{id:super::FullPowerId("focus"),amount:delta}});
                return Ok(())
            }
            apply_focus(state,delta)?;
        }
        CharacterEffect::Stance(next) => {
            let previous = state.character.stance;
            if previous == next { return Ok(()); }
            if previous == Stance::Calm {
                state.player.energy = state.player.energy.checked_add(2)
                    .filter(|energy| *energy <= 999)
                    .ok_or_else(|| super::InputError::Unsupported("stance energy outside supported bounds".into()))?;
            }
            state.character.stance = next;
            super::effects::refresh_enemy_values(state)?;
        }
        CharacterEffect::Channel(orb) => {
            if let Some(slot) = state.character.orb_slots.iter_mut().find(|slot| slot.is_none()) {
                *slot = Some(orb);
            } else if let Some(index) = state.character.orb_slots.iter().position(Option::is_some) {
                state.pending.push_front(Effect { source: source.clone(), target: Some(Actor::Player),
                    kind: EffectKind::Character(CharacterEffect::Channel(orb)) });
                state.pending.push_front(Effect { source: source.clone(), target: Some(Actor::Player),
                    kind: EffectKind::Character(CharacterEffect::EvokeAt { index }) });
            }
        }
        CharacterEffect::EvokeFront { remove } => {
            if let Some(index) = state.character.orb_slots.iter().position(Option::is_some) {
                let orb = state.character.orb_slots[index].unwrap();
                invoke(state, source, orb, true)?;
                if remove {
                    state.character.orb_slots.remove(index);
                    state.character.orb_slots.push(None);
                }
            }
        }
        CharacterEffect::EvokeAt { index } => {
            if index < state.character.orb_slots.len() {
                if let Some(orb) = state.character.orb_slots[index] {
                    invoke(state, source, orb, true)?;
                    state.character.orb_slots.remove(index);
                    state.character.orb_slots.push(None);
                }
            }
        }
        CharacterEffect::EndTurn => {
            let orbs = state.character.orb_slots.iter().flatten().copied().collect::<Vec<_>>();
            for orb in orbs.into_iter().rev() {
                if matches!(orb, OrbKind::Lightning | OrbKind::Frost) { invoke(state, source, orb, false)?; }
            }
        }
    }
    Ok(())
}

pub(super) fn apply_focus(state:&mut super::model::State,delta:i32)->Result<(),super::InputError>{
    if !state.character.orb_slots.iter().any(Option::is_some){return Err(super::InputError::Invariant("Focus adjustment without an active Orb".into()))}
    state.character.focus=state.character.focus.checked_add(delta)
        .filter(|focus|(-999..=999).contains(focus))
        .ok_or_else(||super::InputError::Unsupported("Focus outside supported bounds".into()))?;
    Ok(())
}

fn invoke(state: &mut super::model::State, source: &Source, orb: OrbKind, evoke: bool) -> Result<(), super::EffectError> {
    match orb {
        OrbKind::Frost => {
            let base: i32 = if evoke { 5 } else { 2 };
            let amount = base.checked_add(state.character.focus)
                .ok_or_else(|| super::InputError::Unsupported("Frost Block outside supported bounds".into()))?.max(0);
            state.pending.push_front(Effect { source: source.clone(), target: Some(Actor::Player),
                kind: EffectKind::GainBlock(amount) });
        }
        OrbKind::Lightning => {
            let count = state.enemies.iter().filter(|enemy| enemy.hp > 0).count();
            if count == 0 { return Ok(()); }
            let stream = state.card_rng.as_ref().ok_or_else(|| super::InputError::Invariant("card random stream missing".into()))?;
            let (index, next) = if count == 1 {
                let (_, next) = super::rng::draw_u64(stream)?;
                (0, next)
            } else {
                super::rng::uniform_below(stream, count as u128)?
            };
            state.card_rng = Some(next);
            let target = state.enemies.iter().filter(|enemy| enemy.hp > 0).nth(index as usize)
                .map(|enemy| enemy.instance_id.clone());
            if let Some(target) = target {
                let base: i32 = if evoke { 8 } else { 3 };
                let amount = base.checked_add(state.character.focus)
                    .ok_or_else(|| super::InputError::Unsupported("Lightning evoke outside supported bounds".into()))?.max(0);
                state.pending.push_front(Effect { source: source.clone(), target: Some(Actor::Enemy(target)),
                    kind: EffectKind::Damage(DamagePayload::Orb(amount)) });
            }
        }
    }
    Ok(())
}
