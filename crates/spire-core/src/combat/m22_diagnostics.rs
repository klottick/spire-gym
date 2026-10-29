//! Default-off read-only access to actual fixed-fixture combat custody.
use super::*;
use crate::run::diagnostics::{M22CombatCheckpoint,M22CreationCheckpoint,M22DiagnosticCard,M22DiagnosticZone as Z};
impl Combat{
    pub(crate) fn m22_diagnostic_checkpoint(&self)->M22CombatCheckpoint{
        let s=&self.state;let mut cards=Vec::new();
        for (zone,owners) in [
            (Z::Hand,s.piles.hand.iter().collect::<Vec<_>>()),(Z::Draw,s.piles.draw.iter().collect()),
            (Z::Discard,s.piles.discard.iter().collect()),(Z::Exhaust,s.piles.exhaust.iter().collect()),
            (Z::InPlay,s.in_play.iter().collect()),(Z::PlayFrame,s.play_frames.iter().filter_map(|f|f.parent.as_ref()).collect()),
            (Z::QueuedAutoplay,s.queued_autoplay.values().collect()),(Z::Stasis,s.stasis.values().collect()),
            (Z::PlayedPower,s.played_powers.iter().collect()),(Z::Inactive,s.inactive_cards.iter().collect()),
        ]{for owner in owners{if let Some(card)=owner.full(){cards.push(M22DiagnosticCard{zone,instance:card.instance,id:card.id,origin:card.origin.clone()});}}}
        let current_creation=s.current.as_ref().and_then(|e|if let EffectKind::Watcher(cards::watcher::WatcherEffect::Create{template,count,next})=&e.kind{Some(M22CreationCheckpoint{id:template.id,count:*count,next:*next,template_upgrades:template.upgrades})}else{None});
        let items=s.item_context.as_ref();
        M22CombatCheckpoint{cards,draw_count:s.piles.draw.len(),inventory_relics:items.map_or_else(Vec::new,|i|i.relics().to_vec()),inventory_potions:items.map_or_else(Vec::new,|i|i.potions().to_vec()),inventory_counters:items.map_or_else(Default::default,|i|i.counters().clone()),current_creation,pending_opening_hand:s.pending.iter().find_map(|e|if let EffectKind::OpeningHand{count}=e.kind{Some(count)}else{None}),fault:s.fault.clone()}
    }
}
