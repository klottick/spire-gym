//! Frozen finite M25 choices: M24 unchanged, with the shared Act3 key objective.
#[path = "../m24_support/policy.rs"]
mod m24;
#[path = "../shared/public_keys.rs"]
mod public_keys;

pub use m24::{act_progress, PublicCallerProgress};
use spire_core::combat::CombatAction;
use spire_core::run::{GameAction, PublicDecision, RoomScenario, map::PublicMap};

pub const CALLER_POLICY_REVISION: &str = "m25-public-caller-v1";

/// Choose from the current public decision only. Seed, case and hidden Run state
/// are intentionally absent from this interface.
pub fn choose(
    decision: &PublicDecision,
    map: Option<&PublicMap>,
    progress: &PublicCallerProgress,
) -> Result<GameAction, String> {
    if decision.act >= 3 {
        if let Some(action) = public_keys::key_action(
            decision,
            map,
            RoomScenario::M25WatcherHeart,
        ) {
            return Ok(action);
        }
    }

    match m24::choose(decision, map, progress) {
        Ok(action) => Ok(action),
        Err(error) => selection_fallback(decision).ok_or(error),
    }
}

/// M22 handles ordinary card selections. This covers only a public offer or
/// selection action that the frozen M24 policy reports as unrecognized.
fn selection_fallback(decision: &PublicDecision) -> Option<GameAction> {
    let combat = decision.combat.as_ref()?;
    if combat.card_offer.is_some() {
        return decision
            .candidates
            .iter()
            .find(|candidate| {
                matches!(
                    candidate.action,
                    GameAction::Combat(
                        CombatAction::ChooseCardOffer { .. } | CombatAction::SkipCardOffer
                    )
                )
            })
            .map(|candidate| candidate.action.clone());
    }
    if combat.selection.is_some() {
        return decision
            .candidates
            .iter()
            .find(|candidate| {
                matches!(
                    candidate.action,
                    GameAction::Combat(CombatAction::SelectCard { .. })
                )
            })
            .or_else(|| {
                decision.candidates.iter().find(|candidate| {
                    candidate.action == GameAction::Combat(CombatAction::ConfirmSelection)
                })
            })
            .map(|candidate| candidate.action.clone());
    }
    None
}
