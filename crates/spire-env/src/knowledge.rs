//! Bounded knowledge from declared deck counts and completed public actions only.
use spire_core::combat::{Action, CardKind, MoveId, PublicProjection};
use crate::{EnvError, ScenarioTemplate};
use crate::actions::MenuAction;
use crate::observation::{ObservationRow, encoding_error};

#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PublicKnowledge {
    pub(crate) deck: [[i32;2];3],
    pub(crate) history: [[i32;2];1],
    pub(crate) history_mask: [[i8;2];1],
    pub(crate) in_play: [[i32;2];3],
}

pub(crate) fn card_index(kind: CardKind) -> usize {
    match kind { CardKind::Strike => 0, CardKind::Defend => 1, CardKind::Bash => 2 }
}
pub(crate) fn intent_id(intent: MoveId) -> i32 {
    match intent { MoveId::Incantation => 1, MoveId::Attack => 2,
        MoveId::Chomp => 3, MoveId::Thrash => 4, MoveId::Bellow => 5 }
}

#[allow(dead_code)]
impl PublicKnowledge {
    pub(crate) fn from_template(template: &ScenarioTemplate) -> Self {
        let mut deck = [[0;2];3];
        for card in template.hand.iter().chain(&template.draw) {
            deck[card_index(card.kind)][usize::from(card.upgraded)] += 1;
        }
        Self { deck, history: [[0;2]], history_mask: [[0;2]], in_play: [[0;2];3] }
    }

    /// Call once after a successful completed core step, never on rejected/faulted steps.
    pub(crate) fn observe_completed(&mut self, previous: &ObservationRow,
        accepted: &MenuAction, projected: &PublicProjection<'_>) -> Result<(), EnvError> {
        if projected.enemies.len() != 1 { return Err(encoding_error("expected one public enemy")); }
        if matches!(accepted.core, Action::EndTurn) && projected.outcome.is_none() {
            self.history[0] = [previous.enemy_intent[0], self.history[0][0]];
            self.history_mask[0] = [1, self.history_mask[0][0]];
        }
        self.in_play = [[0;2];3];
        if projected.outcome.is_some() {
            if let Some(card) = &accepted.source {
                self.in_play[card_index(card.kind)][usize::from(card.upgraded)] = 1;
            }
        }
        Ok(())
    }
}
