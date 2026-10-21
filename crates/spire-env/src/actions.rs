//! Cached owned core actions in public row order, never handle-text order.
use spire_core::combat::{Action, Combat};
use crate::{CardSpec, EnvError, InputReason};
use crate::error::input;
use crate::observation::{capacity, encoding_error};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct MenuAction {
    pub(crate) core: Action,
    pub(crate) source: Option<CardSpec>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Candidate {
    action: MenuAction,
    source_row: i32,
    target_row: i32,
}

// Task 4 supplies the production slot consumer of these internal interfaces.
#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CandidateMenu { candidates: Vec<Candidate> }

#[allow(dead_code)]
impl CandidateMenu {
    pub(crate) fn from_combat(combat: &Combat) -> Result<Self, EnvError> {
        let p = combat.public_projection().map_err(|e| encoding_error(format!("projection: {e:?}")))?;
        capacity(p.hand.len(), p.enemies.len(), 0)?;
        let legal = combat.legal_actions().map_err(|e| encoding_error(format!("legal menu: {e:?}")))?;
        capacity(p.hand.len(), p.enemies.len(), legal.len())?;
        let mut candidates = Vec::with_capacity(legal.len());
        for core in legal {
            let (source, source_row, target_row) = match &core {
                Action::EndTurn => (None, -1, -1),
                Action::PlayCard { card_instance_id, target_id } => {
                    let row = p.hand.iter().position(|c| &c.instance_id == card_instance_id)
                        .ok_or_else(|| encoding_error("legal source absent from public hand"))?;
                    let card = &p.hand[row];
                    let target = match target_id {
                        None => -1,
                        Some(id) => p.enemies.iter().position(|e| &e.instance_id == id)
                            .ok_or_else(|| encoding_error("legal target absent from public enemies"))? as i32,
                    };
                    (Some(CardSpec { kind: card.card_id, upgraded: card.upgraded }), row as i32, target)
                }
            };
            candidates.push(Candidate { action: MenuAction { core, source }, source_row, target_row });
        }
        candidates.sort_by_key(|c| (c.source_row, c.target_row));
        Ok(Self { candidates })
    }

    pub(crate) fn resolve(&self, index: i64) -> Result<&MenuAction, EnvError> {
        usize::try_from(index).ok().and_then(|i| self.candidates.get(i)).map(|c| &c.action)
            .ok_or_else(|| input(InputReason::IllegalAction, "candidate is padding or out of range"))
    }

    pub(crate) fn rows(&self) -> impl Iterator<Item = (&MenuAction, i32, i32)> {
        self.candidates.iter().map(|c| (&c.action, c.source_row, c.target_row))
    }
}
