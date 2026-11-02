//! Run-owned equip effects for Sacred Bark and Prismatic Shard (M27 Task M5).
use crate::{content::{PotionId, RelicId}, run::{ItemAdmission, RunError, RunState}};

use super::lifecycle::EquipWork;

pub(crate) fn base_potion_potency(id: PotionId) -> i32 {
    match id {
        PotionId::FirePotion => 20,
        PotionId::BloodPotion => 20,
        PotionId::FruitJuice => 5,
        PotionId::BlockPotion => 12,
        PotionId::EnergyPotion | PotionId::StrengthPotion | PotionId::DexterityPotion
        | PotionId::FocusPotion | PotionId::PotionOfCapacity => 2,
        PotionId::SwiftPotion | PotionId::CunningPotion | PotionId::DistilledChaos => 3,
        PotionId::WeakPotion | PotionId::FearPotion => 3,
        PotionId::PoisonPotion => 6,
        PotionId::AncientPotion => 1,
        PotionId::HeartOfIron => 6,
        PotionId::GhostInAJar => 1,
        PotionId::AttackPotion | PotionId::SkillPotion | PotionId::PowerPotion
        | PotionId::ColorlessPotion | PotionId::LiquidMemories | PotionId::DuplicationPotion
        | PotionId::EssenceOfDarkness => 1,
        PotionId::FlexPotion | PotionId::SpeedPotion | PotionId::RegenPotion => 5,
        PotionId::LiquidBronze => 3,
        PotionId::EssenceOfSteel => 4,
        PotionId::CultistPotion => 1,
        PotionId::SneckoOil => 5,
        PotionId::FairyPotion => 30,
        PotionId::ExplosivePotion => 10,
        PotionId::Ambrosia | PotionId::BottledMiracle => 2,
        PotionId::Elixir | PotionId::SmokeBomb | PotionId::BlessingOfTheForge
        | PotionId::GamblersBrew | PotionId::StancePotion | PotionId::EntropicBrew | PotionId::Unknown(_) => 0,
    }
}

pub(crate) fn initial_potion_potency(id: PotionId, sacred_bark: bool) -> i32 {
    base_potion_potency(id) * if sacred_bark { 2 } else { 1 }
}

/// Reconcile direct Run producers by identity while keeping the stored potency of
/// unchanged potion instances. Empty belts stay unmaterialized for compact Debug.
pub(crate) fn reconcile_potion_potencies(
    relics: &[RelicId],
    potions: &[Option<PotionId>],
    previous: &[Option<(PotionId, i32)>],
) -> Vec<Option<(PotionId, i32)>> {
    if !potions.iter().any(Option::is_some) { return Vec::new(); }
    let sacred_bark = relics.contains(&RelicId::SacredBark);
    potions.iter().enumerate().map(|(slot, potion)| {
        potion.map(|id| {
            previous.get(slot).copied().flatten()
                .filter(|(previous_id, _)| *previous_id == id)
                .unwrap_or((id, initial_potion_potency(id, sacred_bark)))
        })
    }).collect()
}

pub(crate) fn equip(id: RelicId, state: &mut RunState, _work: &mut EquipWork) -> Result<bool, RunError> {
    if state.item_admission != ItemAdmission::Complete { return Ok(false); }
    match id {
        RelicId::SacredBark => {
            state.potion_potencies = state.potions.iter().map(|potion| {
                potion.map(|id| (id, initial_potion_potency(id, true)))
            }).collect();
            if !state.potions.iter().any(Option::is_some) { state.potion_potencies.clear(); }
            Ok(true)
        }
        // Shard's permanent effect is read once at each shared combat entry.
        RelicId::PrismaticShard => Ok(true),
        _ => Ok(false),
    }
}
