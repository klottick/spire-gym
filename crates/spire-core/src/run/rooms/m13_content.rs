//! Finite Silent Act2 rows composed from the accepted M12 catalog.
use super::*;

pub(in crate::run) fn reward_cards(boss: bool, local_visit: usize) -> Vec<CardId> {
    if boss { vec![CardId::Nightmare] } else { super::m12_content::reward_cards(false, local_visit) }
}

pub(in crate::run) fn combat_potion(local_visit: u64) -> PotionId {
    [PotionId::FirePotion, PotionId::StrengthPotion, PotionId::WeakPotion][(local_visit % 3) as usize]
}

pub(in crate::run) fn elite_relic(start: usize, owned: &[RelicId]) -> Result<(RelicId, usize), RunError> {
    super::m12_content::elite_relic_with_exhaustion(start, owned, "captured Silent Act2 elite relic sequence exhausted")
}
