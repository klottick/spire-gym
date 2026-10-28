//! Detached in-memory checkpoints, not a serialized wire protocol.
use super::{CONTENT_ID, RestoreError};
use super::engine::{validate_runtime, Combat};
use super::model::State;
use super::rng::RNG_CONTRACT_ID;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Snapshot {
    pub(super) schema_version: u32,
    pub(super) content_id: String,
    pub(super) rng_contract_id: String,
    pub(super) state: State,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Divergence { pub component: &'static str, pub left: String, pub right: String }

impl Combat {
    pub fn snapshot(&self) -> Snapshot {
        let schema_version = if is_legacy_v2_state(&self.state) { 2 } else { 3 };
        Snapshot { schema_version, content_id: CONTENT_ID.into(),
            rng_contract_id: RNG_CONTRACT_ID.into(), state: self.state.clone() }
    }

    pub fn try_snapshot_v2(&self) -> Result<Snapshot, RestoreError> {
        if !is_legacy_v2_state(&self.state) { return Err(RestoreError::Compatibility); }
        Ok(Snapshot { schema_version: 2, content_id: CONTENT_ID.into(),
            rng_contract_id: RNG_CONTRACT_ID.into(), state: self.state.clone() })
    }

    pub fn restore(snapshot: &Snapshot) -> Result<Self, RestoreError> {
        if snapshot.schema_version != 2 || snapshot.content_id != CONTENT_ID
            || snapshot.rng_contract_id != RNG_CONTRACT_ID
            || snapshot.state.content_id != snapshot.content_id
            || snapshot.state.rng.contract_id != snapshot.rng_contract_id {
            return Err(RestoreError::Compatibility);
        }
        if !is_legacy_v2_state(&snapshot.state) { return Err(RestoreError::Compatibility); }
        if snapshot.state.fault.is_some() { return Err(RestoreError::Faulted); }
        validate_runtime(&snapshot.state).map_err(RestoreError::Invalid)?;
        let legacy_cache = snapshot.state.piles.legacy_projection().ok_or(RestoreError::Compatibility)?;
        Ok(Self { state: snapshot.state.clone(), legacy_cache })
    }
}

pub fn compare(left: &Snapshot, right: &Snapshot) -> Option<Divergence> {
    macro_rules! component {
        ($label:expr, $a:expr, $b:expr) => {
            if $a != $b {
                return Some(Divergence { component: $label, left: format!("{:?}", $a), right: format!("{:?}", $b) });
            }
        };
    }
    component!("content_id", left.content_id, right.content_id);
    component!("schema_version", left.schema_version, right.schema_version);
    component!("rng_contract_id", left.rng_contract_id, right.rng_contract_id);
    macro_rules! state_fields {
        ($($field:ident),+ $(,)?) => { $(component!(stringify!($field), left.state.$field, right.state.$field);)+ };
    }
    component!("state.content_id", left.state.content_id, right.state.content_id);
    state_fields!(phase, turn, player, enemies, piles, card_ids_issued, rng, pending, current,
        in_play, resolved_effects, public, outcome, fault, attempted_action, fault_selection, player_full_powers,
        temporary_retain, nightmare_templates, next_nightmare_installation_id,
        actor_slots,facing,spawn_ordinals,spawn_progress,
        stasis,inactive_cards,permanent_gains,next_permanent_gain,played_powers,play_frames,play_context,queued_autoplay,pending_time_warp,
        hp_before_healing, hp_after_cleanup, full_runtime, pending_selection, hook_events, item_context,combust,
        character, card_rng, replacement_progress, damage_events_this_combat, silent,colorless);
    None
}

fn is_legacy_v2_state(state: &State) -> bool {
    !state.full_runtime && state.enemies.len() == 1
        && state.silent==super::cards::SilentState::default()
        && state.enemies.iter().all(|enemy| enemy.life == super::ActorLife::Active)
        && state.replacement_progress.is_none() && state.pending_selection.is_none()
        && state.hook_events.is_empty() && state.attempted_action.is_none()
        && state.fault_selection.is_none() && state.player_full_powers.is_empty()
        && state.play_frames.is_empty()
        && !state.pending.iter().chain(state.current.iter()).any(|effect|matches!(effect.kind,
            super::model::EffectKind::Alchemize{..}|super::model::EffectKind::SmokeEscape|super::model::EffectKind::Generate(_)|super::model::EffectKind::JuggernautRandomDamage(_)|super::model::EffectKind::SilentItem(_)|super::model::EffectKind::Potion(_)|super::model::EffectKind::CardOffer{..}|super::model::EffectKind::DeliverCardOffer{..}|super::model::EffectKind::ObtainPotion{..}))
        && state.damage_events_this_combat==0
        && state.colorless==super::model::ColorlessState::default()
        && state.play_context.is_none()
        && state.queued_autoplay.is_empty()
        && state.item_context.is_none()
        && state.character == super::characters::CharacterState::for_character(crate::content::Character::Ironclad)
        && state.card_rng.is_none()
        && state.piles.all(state.in_play.as_ref()).all(|card| card.legacy().is_some())
}
