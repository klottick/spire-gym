use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use spire_core::{combat::{CombatAction, CombatCardRef, CombatEntry, rng::StreamState}, run::GameAction};

pub fn catalog() -> Value {
    serde_json::from_str(include_str!("../../../../performance/catalog.json")).expect("checked-in catalog")
}

/// serde_json's default Map is sorted; arrays deliberately retain their order.
pub fn canonical_hash(value: &Value) -> String {
    let bytes = serde_json::to_vec(value).expect("JSON value");
    format!("{:x}", Sha256::digest(bytes))
}

pub fn combat_action_value(action: &CombatAction) -> Value {
    match action {
        CombatAction::PlayCard { instance, target } => json!(["play", instance, target.map(|t| t.0)]),
        CombatAction::SelectCard { instance } => json!(["select", instance]),
        CombatAction::ConfirmSelection => json!(["confirm-selection"]),
        CombatAction::UsePotion { slot, target } => json!(["potion", slot, target.map(|t| t.0)]),
        CombatAction::EndTurn => json!(["end-turn"]),
    }
}

pub fn game_action_value(action: &GameAction) -> Value {
    match action {
        GameAction::Move(id) => json!(["move", id.0]),
        GameAction::Combat(a) => json!(["combat", combat_action_value(a)]),
        GameAction::Choose(id) => json!(["choose", id.0]),
        GameAction::Select(id) => json!(["select", id.0]),
        GameAction::Confirm => json!(["confirm"]),
        GameAction::Cancel => json!(["cancel"]),
        GameAction::UsePotion { slot, target } => json!(["potion", slot, target.map(|t| t.0)]),
        GameAction::DiscardPotion { slot } => json!(["discard-potion", slot]),
    }
}

fn stream_value(stream: &StreamState) -> Value {
    json!({"domain": stream.id().domain().as_str(), "scope": stream.id().scope(),
           "key": stream.key(), "word_index": stream.word_index().to_string()})
}

/// Narrow authored-entry encoder. Unsupported additions require an explicit
/// benchmark contract change rather than an unstable Debug-string fingerprint.
pub fn entry_value(entry: &CombatEntry) -> Result<Value, String> {
    use spire_core::combat::enemies::EnemyProfile;
    let s = &entry.setup;
    if !s.player.powers.is_empty() || !s.player_powers.is_empty() ||
       s.enemies.iter().any(|e| e.initial_memory.is_some() || !e.powers.is_empty() || e.profile != EnemyProfile::Standard) {
        return Err("benchmark entry encoder needs a new contract for powers/memory/profile".into());
    }
    let enemies: Vec<_> = s.enemies.iter().map(|e| json!({
        "instance":e.instance_id.as_str(), "id":e.enemy_id.as_str(), "ascension":e.ascension,
        "profile":"standard", "initial_memory":null, "powers":[],
        "hp":e.hp, "max_hp":e.max_hp, "block":e.block, "intent":e.intent.as_str(), "rng":stream_value(&e.ai_rng)
    })).collect();
    let cards: Vec<_> = entry.cards.iter().map(|c| json!({
        "instance":c.instance.0, "id":c.id.as_str(), "upgrades":c.upgrades,
        "origin":match c.origin { CombatCardRef::Permanent(id)=>json!(["permanent",id.0]), CombatCardRef::CombatLocal(id)=>json!(["local",id]) },
        "local":{"randomized_cost":c.local.randomized_cost,"temporary_cost":c.local.temporary_cost,
                 "free_play":c.local.free_play,"x_cost":c.local.x_cost,"exhaust_on_finish":c.local.exhaust_on_finish}
    })).collect();
    let bases: Vec<_> = s.enemy_damage_bases.iter().map(|((slot,intent),amount)|json!([slot,intent.as_str(),amount])).collect();
    Ok(json!({"player":{"hp":s.player.hp,"max_hp":s.player.max_hp,"block":s.player.block,
                        "energy":s.player.energy,"max_energy":s.player.max_energy,"draw_per_turn":s.player.draw_per_turn,"powers":[]},
              "player_powers":[],"character":s.character.as_str(),"card_rng":stream_value(&s.card_rng),
              "shuffle_rng":stream_value(&s.shuffle_rng),"enemies":enemies,"enemy_damage_bases":bases,
              "opening_draw":entry.opening_draw,"cards":cards}))
}
