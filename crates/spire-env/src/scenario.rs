//! Ordered, validated supplied setups. No random generation or entry-hook execution.
use std::{collections::BTreeSet, sync::Arc};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use spire_core::combat::{CardKind, EnemyKind};
use crate::{EnvError, InputReason, canonical_json};
use crate::error::{input, reserved, setup};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CardSpec { pub kind: CardKind, pub upgraded: bool }

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublicContext { pub act: Option<u8>, pub next_encounter: Option<EnemyKind> }

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScenarioTemplate {
    pub id: String,
    pub encounter: EnemyKind,
    pub player_hp: i32,
    pub player_max_hp: i32,
    pub energy_per_turn: i32,
    pub draw_per_turn: i32,
    pub enemy_hp: i32,
    pub hand: Vec<CardSpec>,
    /// Bottom to top: the last entry is drawn first.
    pub draw: Vec<CardSpec>,
    pub context: PublicContext,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScenarioCatalog {
    templates: Arc<[Arc<ScenarioTemplate>]>,
    fingerprint: [u8;32],
}

fn validate(t: &ScenarioTemplate) -> Result<(), EnvError> {
    let id = t.id.as_bytes();
    if !(1..=64).contains(&id.len()) || !id[0].is_ascii_alphanumeric()
        || !id.iter().all(|b| b.is_ascii_alphanumeric() || b"_.-".contains(b)) {
        return Err(input(InputReason::Scenario, "scenario ID must match [A-Za-z0-9][A-Za-z0-9_.-]{0,63}"));
    }
    if !(1..=999).contains(&t.player_max_hp) || !(1..=t.player_max_hp).contains(&t.player_hp)
        || !(1..=999).contains(&t.energy_per_turn) || !(1..=10).contains(&t.draw_per_turn) {
        return Err(input(InputReason::Range, "invalid supplied player HP, energy or draw count"));
    }
    let bounds = match t.encounter { EnemyKind::Cultist => 48..=54, EnemyKind::JawWorm => 40..=44 };
    if !bounds.contains(&t.enemy_hp) {
        return Err(input(InputReason::Range, "enemy HP outside supported encounter bounds"));
    }
    let total = t.hand.len().checked_add(t.draw.len()).ok_or_else(|| setup("card count overflow"))?;
    if t.hand.len() > 10 || !(1..=999).contains(&total) {
        return Err(input(InputReason::Capacity, "supplied setup requires 1–999 cards and at most 10 in hand"));
    }
    if t.context.act.is_some_and(|act| act != 1) {
        return Err(input(InputReason::Scenario, "only unknown or Act 1 context is supported"));
    }
    Ok(())
}

impl ScenarioCatalog {
    pub fn new(templates: Vec<ScenarioTemplate>) -> Result<Self, EnvError> {
        if templates.is_empty() { return Err(input(InputReason::Scenario, "catalog must not be empty")); }
        u64::try_from(templates.len()).map_err(|_| setup("catalog length exceeds ordinal range"))?;
        let mut ids = BTreeSet::new();
        // Validate every template before installing shared immutable owners.
        for t in &templates {
            validate(t)?;
            if !ids.insert(&t.id) { return Err(input(InputReason::Scenario, "duplicate scenario ID")); }
        }
        let mut owned = reserved(templates.len())?;
        owned.extend(templates.into_iter().map(Arc::new));
        let mut catalog = Self { templates: owned.into(), fingerprint: [0;32] };
        let mut hash = Sha256::new();
        hash.update(b"spire-lab/env-catalog/v1\0");
        hash.update(canonical_json(&catalog.descriptor())?);
        catalog.fingerprint = hash.finalize().into();
        Ok(catalog)
    }

    pub fn starter() -> Self {
        let cards = |kinds: &[CardKind]| kinds.iter().map(|kind| CardSpec { kind: *kind, upgraded: false }).collect();
        use CardKind::{Strike, Defend, Bash};
        let templates = [("starter-cultist-a0-v1", EnemyKind::Cultist, 52),
            ("starter-jaw-worm-a0-v1", EnemyKind::JawWorm, 42)].into_iter()
            .map(|(id, encounter, enemy_hp)| ScenarioTemplate {
                id: id.into(), encounter, player_hp: 80, player_max_hp: 80,
                energy_per_turn: 3, draw_per_turn: 5, enemy_hp,
                hand: cards(&[Strike, Defend, Strike, Defend, Bash]),
                draw: cards(&[Strike, Strike, Strike, Defend, Defend]),
                context: PublicContext { act: None, next_encounter: None },
            }).collect();
        Self::new(templates).expect("authored starter templates are valid")
    }

    pub fn descriptor(&self) -> Value {
        let cards = |pile: &[CardSpec]| pile.iter().map(|c| json!({"kind":c.kind.as_str(), "upgraded":c.upgraded})).collect::<Vec<_>>();
        json!({"schema_id":"spire-lab-scenario-catalog-v1", "templates":self.templates.iter().map(|t| json!({
            "id":t.id, "encounter":t.encounter.as_str(), "player_hp":t.player_hp,
            "player_max_hp":t.player_max_hp, "energy_per_turn":t.energy_per_turn,
            "draw_per_turn":t.draw_per_turn, "enemy_hp":t.enemy_hp,
            "hand":cards(&t.hand), "draw":cards(&t.draw),
            "context":{"act":t.context.act, "next_encounter":t.context.next_encounter.map(EnemyKind::as_str)}
        })).collect::<Vec<_>>()})
    }

    pub fn fingerprint(&self) -> [u8;32] { self.fingerprint }

    pub(crate) fn select(&self, ordinal: u64, id: Option<&str>) -> Result<Arc<ScenarioTemplate>, EnvError> {
        let selected = match id {
            Some(id) => self.templates.iter().find(|t| t.id == id)
                .ok_or_else(|| input(InputReason::Scenario, "unknown scenario ID"))?,
            None => &self.templates[(ordinal % self.templates.len() as u64) as usize],
        };
        Ok(Arc::clone(selected))
    }
}
