//! Finite M24 Watcher Acts1–3: the M22/M23 producer owner plus only what captured
//! Act3 forces on a Watcher Run (Writhing Mass Parasite, Tomb Red Mask, Awakened
//! One Void). No producer rows are copied; the unions extend the M22 slices.

use crate::content::{CardId, PotionId, RelicId};
use super::m22_content as m22;

const fn extend<T: Copy, const N: usize>(base: &[T], extra: T) -> [T; N] {
    let mut out = [extra; N];
    let mut i = 0;
    while i < base.len() { out[i] = base[i]; i += 1; }
    out
}
const CARDS: [CardId; m22::M22_WATCHER_ACT1_CARDS.len() + 1] = extend(m22::M22_WATCHER_ACT1_CARDS, CardId::Parasite);
const RELICS: [RelicId; m22::M22_WATCHER_ACT1_RELICS.len() + 1] = extend(m22::M22_WATCHER_ACT1_RELICS, RelicId::RedMask);

/// M22's 28 permanent kinds plus Parasite (Writhing Mass Implant only; never offered).
pub const M24_WATCHER_ACT3_CARDS: &[CardId] = &CARDS;
/// M22's 18 relic kinds plus Red Mask (Tomb of Lord Red Mask only).
pub const M24_WATCHER_ACT3_RELICS: &[RelicId] = &RELICS;
/// Same seven potion kinds as M22/M23.
pub const M24_WATCHER_ACT3_POTIONS: &[PotionId] = m22::M22_WATCHER_ACT1_POTIONS;
/// Combat-generated Watcher children; never permanent acquisitions.
pub const GENERATED_ONLY: &[CardId] = crate::content::watcher::M21_WATCHER_GENERATED;
/// M23's statuses plus Awakened One's bottom-inserted Void.
pub const STATUSES: &[CardId] = &[CardId::Dazed, CardId::Burn, CardId::Slimed, CardId::Wound, CardId::Void];
/// Descriptor taxonomy rows added after the frozen M22 taxonomy.
pub const RELIC_TAXONOMY_ADDITIONS: &[(RelicId, &str)] = &[(RelicId::RedMask, "special")];

/// Act3-only finite capture domain; Act1/Act2 keep the M22/M23 scope and key law.
pub const ACT3_RELIC_CAPTURE_SCOPE: crate::run::RunRngScope = crate::run::RunRngScope::M24RelicCapture;
pub const ACT3_RELIC_CAPTURE_SCOPE_NAME: &str = "run.m24.relic.capture";
