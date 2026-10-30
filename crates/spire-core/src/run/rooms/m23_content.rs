//! Finite M23 Watcher Acts1–2: views of the single M22 producer owner plus the
//! approved Act2 capture domain. No producer rows are copied here.

use crate::content::{CardId, PotionId, RelicId};
use super::m22_content as m22;

/// Same obtainable permanent union as M22 (28 kinds).
pub const M23_WATCHER_ACT2_CARDS: &[CardId] = m22::M22_WATCHER_ACT1_CARDS;
/// Same possible relic union as M22 (18 kinds).
pub const M23_WATCHER_ACT2_RELICS: &[RelicId] = m22::M22_WATCHER_ACT1_RELICS;
/// Same potion union as M22 (7 kinds).
pub const M23_WATCHER_ACT2_POTIONS: &[PotionId] = m22::M22_WATCHER_ACT1_POTIONS;
/// Combat-generated Watcher children; never permanent acquisitions.
pub const GENERATED_ONLY: &[CardId] = crate::content::watcher::M21_WATCHER_GENERATED;
/// Reachable statuses: Act1 Dazed/Burn/Slimed plus Act2 Wound.
pub const STATUSES: &[CardId] = &[CardId::Dazed, CardId::Burn, CardId::Slimed, CardId::Wound];

/// Act2-only finite capture domain; Act1 keeps the M22 scope/key law exactly.
pub const ACT2_RELIC_CAPTURE_SCOPE: crate::run::RunRngScope = crate::run::RunRngScope::M23RelicCapture;
pub const ACT2_RELIC_CAPTURE_SCOPE_NAME: &str = "run.m23.relic.capture";

/// Act1 keeps `run.m22.relic.capture`; only M23 Act2 appends its own scope.
pub(crate) fn relic_capture_scope(state: &crate::run::RunState) -> crate::run::RunRngScope {
    if super::is_m23(state) && state.act == 2 { ACT2_RELIC_CAPTURE_SCOPE } else { m22::RELIC_CAPTURE_SCOPE }
}
