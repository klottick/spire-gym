//! M34s-c `spire-gym-combat-position-v1` catalog field (design §4.8; c0 `task0-v1.md` §2): strict JSON → the core
//! [`Position`], whose owner (`combat::position`) holds every rule. Hidden-state fields are rejected by name at any level.
use crate::RunSessionError as Error;
use serde::Deserialize;
use serde_json::{json, Value};
use spire_core::combat::position::{power_id, EnemyFields, Position, PositionCard, PositionError, PositionPiles, PLAYER_POWERS, POSITION_SCHEMA_ID};
use spire_core::combat::{OrbKind, Stance};
use spire_core::content::CardId;
use std::collections::BTreeMap;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawPosition { #[serde(default)] enemies: Vec<RawEnemy>, #[serde(default)] player: Option<RawPlayer>, #[serde(default)] piles: Option<RawPiles> }
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawEnemy { slot: u8, #[serde(default)] hp: Option<i32>, #[serde(default)] block: Option<i32>, #[serde(default)] powers: BTreeMap<String, i32> }
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawPlayer {
    #[serde(default)] powers: BTreeMap<String, i32>, #[serde(default)] block: Option<i32>, #[serde(default)] energy: Option<i32>,
    #[serde(default)] stance: Option<String>, #[serde(default)] mantra: Option<i32>, #[serde(default)] orb_slots: Option<u8>, #[serde(default)] orbs: Option<Vec<String>>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawPiles { #[serde(default)] hand: Vec<RawCard>, #[serde(default)] draw_top: Vec<RawCard>, #[serde(default)] draw: Vec<RawCard>, #[serde(default)] discard: Vec<RawCard>, #[serde(default)] exhaust: Vec<RawCard> }
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawCard { card: String, #[serde(default)] upgrades: u32, #[serde(default)] misc: i32, #[serde(default)] generated: bool }

/// Hidden state a position never carries (plan c1): named in the rejection.
fn hidden_field(name: &str) -> Option<&'static str> {
    Some(match name {
        "order" => "position: hidden-state field `order` is not admitted (the gym owns draw order)",
        "seed" => "position: hidden-state field `seed` is not admitted",
        "rng" => "position: hidden-state field `rng` is not admitted",
        "intent" => "position: hidden-state field `intent` is not admitted (intents are rolled natively)",
        "memory" => "position: hidden-state field `memory` is not admitted (AI memory is native)",
        "instance" => "position: hidden-state field `instance` is not admitted",
        "handle" => "position: hidden-state field `handle` is not admitted",
        "turn" => "position: hidden-state field `turn` is not admitted (positions are at the opening decision)",
        _ => return None,
    })
}
fn scan_hidden(v: &Value) -> Result<(), Error> {
    match v {
        Value::Object(map) => map.iter().try_for_each(|(k, v)| hidden_field(k).map_or(Ok(()), |m| Err(Error::Input(m))).and_then(|()| scan_hidden(v))),
        Value::Array(items) => items.iter().try_for_each(scan_hidden),
        _ => Ok(()),
    }
}
/// Admitted keys per level (the serde structs above): an unknown key is `Input` naming its level and the admitted keys.
fn check_keys(v: &Value, admitted: &[&str], msg: &'static str) -> Result<(), Error> {
    match v.as_object() { Some(m) if !m.keys().all(|k| admitted.contains(&k.as_str())) => Err(Error::Input(msg)), _ => Ok(()) }
}
fn scan_keys(v: &Value) -> Result<(), Error> {
    const PILES: [&str; 5] = ["hand", "draw_top", "draw", "discard", "exhaust"];
    check_keys(v, &["enemies", "player", "piles"], "position: unknown field (admitted: enemies, player, piles)")?;
    for e in v.get("enemies").and_then(Value::as_array).into_iter().flatten() {
        check_keys(e, &["slot", "hp", "block", "powers"], "position.enemies[]: unknown field (admitted: slot, hp, block, powers)")?;
    }
    if let Some(p) = v.get("player") {
        check_keys(p, &["powers", "block", "energy", "stance", "mantra", "orb_slots", "orbs"],
            "position.player: unknown field (admitted: powers, block, energy, stance, mantra, orb_slots, orbs)")?;
    }
    if let Some(p) = v.get("piles") {
        check_keys(p, &PILES, "position.piles: unknown field (admitted: hand, draw_top, draw, discard, exhaust)")?;
        for row in PILES.iter().filter_map(|k| p.get(*k)).filter_map(Value::as_array).flatten() {
            check_keys(row, &["card", "upgrades", "misc", "generated"], "position.piles card row: unknown field (admitted: card, upgrades, misc, generated)")?;
        }
    }
    Ok(())
}
/// A core position error as a session error (`Input` names the field; a boundary is `UnsupportedBoundary`).
pub(crate) fn position_error(e: PositionError) -> Error {
    match e { PositionError::Input(m) => Error::Input(m), PositionError::Unsupported(m) => Error::UnsupportedBoundary(m) }
}
fn powers(map: &BTreeMap<String, i32>, unknown: &'static str) -> Result<Vec<(spire_core::combat::FullPowerId, i32)>, Error> {
    map.iter().map(|(k, v)| Ok((power_id(k).ok_or(Error::Input(unknown))?, *v))).collect()
}
fn cards(rows: &[RawCard]) -> Result<Vec<PositionCard>, Error> {
    rows.iter().map(|r| Ok(PositionCard { id: CardId::from_name(&r.card).ok_or(Error::Input("piles: unknown card id"))?, upgrades: r.upgrades, misc: r.misc, generated: r.generated })).collect()
}
const STANCES: [(&str, Stance); 4] = [("neutral", Stance::Neutral), ("wrath", Stance::Wrath), ("calm", Stance::Calm), ("divinity", Stance::Divinity)];
const ORBS: [(&str, OrbKind); 4] = [("lightning", OrbKind::Lightning), ("frost", OrbKind::Frost), ("dark", OrbKind::Dark), ("plasma", OrbKind::Plasma)];

/// Parse a catalog entry's `position` (strict; the core validates the rules).
pub(crate) fn parse_position(v: &Value) -> Result<Position, Error> {
    scan_hidden(v)?;
    scan_keys(v)?;
    let raw: RawPosition = serde_json::from_value(v.clone()).map_err(|_| Error::Input("position: invalid type or missing field"))?;
    let enemies = raw.enemies.iter().map(|e| Ok(EnemyFields { slot: e.slot, hp: e.hp, block: e.block, powers: powers(&e.powers, "enemies.powers: unknown power id")? }))
        .collect::<Result<Vec<_>, Error>>()?;
    let mut p = Position { enemies, ..Position::default() };
    if let Some(pl) = raw.player {
        p.powers = powers(&pl.powers, "player.powers: unknown power id")?;
        (p.block, p.energy, p.mantra, p.orb_slots) = (pl.block, pl.energy, pl.mantra, pl.orb_slots);
        p.stance = pl.stance.map(|s| STANCES.iter().find(|(n, _)| *n == s).map(|(_, x)| *x).ok_or(Error::Input("player.stance: unknown stance"))).transpose()?;
        p.orbs = pl.orbs.map(|o| o.iter().map(|s| ORBS.iter().find(|(n, _)| n == s).map(|(_, x)| *x).ok_or(Error::Input("player.orbs: unknown orb"))).collect::<Result<Vec<_>, _>>()).transpose()?;
    }
    if let Some(pi) = raw.piles {
        p.piles = Some(PositionPiles { hand: cards(&pi.hand)?, draw_top: cards(&pi.draw_top)?, draw: cards(&pi.draw)?, discard: cards(&pi.discard)?, exhaust: cards(&pi.exhaust)? });
    }
    Ok(p)
}

/// The normalised position in the catalog descriptor (canonical: sorted power maps, explicit defaults).
pub(crate) fn position_json(p: &Position) -> Value {
    let pw = |v: &[(spire_core::combat::FullPowerId, i32)]| v.iter().map(|(id, a)| (id.0.to_owned(), json!(a))).collect::<serde_json::Map<_, _>>();
    let row = |c: &PositionCard| json!({"card": c.id.as_str(), "upgrades": c.upgrades, "misc": c.misc, "generated": c.generated});
    let rows = |v: &[PositionCard]| v.iter().map(row).collect::<Vec<_>>();
    let mut enemies = p.enemies.iter().map(|e| json!({"slot": e.slot, "hp": e.hp, "block": e.block, "powers": pw(&e.powers)})).collect::<Vec<_>>();
    enemies.sort_by_key(|e| e["slot"].as_u64());
    let mut draw = p.piles.as_ref().map(|x| rows(&x.draw)).unwrap_or_default();
    draw.sort_by_key(|r| r.to_string());
    json!({"schema_id": POSITION_SCHEMA_ID, "enemies": enemies,
        "player": {"powers": pw(&p.powers), "block": p.block, "energy": p.energy,
            "stance": p.stance.map(|s| STANCES.iter().find(|(_, x)| *x == s).unwrap().0), "mantra": p.mantra, "orb_slots": p.orb_slots,
            "orbs": p.orbs.as_ref().map(|o| o.iter().map(|k| ORBS.iter().find(|(_, x)| x == k).unwrap().0).collect::<Vec<_>>())},
        "piles": p.piles.as_ref().map(|x| json!({"hand": rows(&x.hand), "draw_top": rows(&x.draw_top), "draw": draw, "discard": rows(&x.discard), "exhaust": rows(&x.exhaust)}))})
}

/// The position part of the environment support (design §4.8).
pub(crate) fn position_support() -> Value {
    json!({"schema_id": POSITION_SCHEMA_ID, "boundary": "the opening (first main) decision of the native start; an opening selection or offer is UnsupportedBoundary",
        "enemies": {"requires": "a fixed formation with seed-independent species (not the shapes encounters)", "slot": "formation index (setup order), not the view's logical slot", "hp": "setup HP before the opening roll, 1..=the species' minimum max HP at the ascension",
            "block": "0..=999 at the opening decision", "powers": "setup values re-asserted at the opening decision (0 removes); every species: strength, vulnerable, weak, poison; the entry's declared burning buff (metallicize or regenerate) on every member with one amount; species-native artifact, thorns, strength_up, spore_cloud, plated_armor, metallicize"},
        "player": {"powers": PLAYER_POWERS, "block": "0..=999", "energy": "0..=999", "stance": "neutral|wrath|calm|divinity (Watcher card laws)", "mantra": "0..=9",
            "orb_slots": "1..=10 with orbs (combats with orb slots); channel totals = listed orbs of each kind", "unlisted": "native opening values (relic-start powers kept)"},
        "piles": {"all_or_nothing": true, "hand": "<= 10, left to right; innate/bottled deck cards required; not empty while Unceasing Top would draw; costs rolled as if drawn (Confused/Snecko on the session card stream)",
            "draw_top": "known top cards, top first", "draw": "unordered multiset shuffled by the gym", "discard": "oldest first", "exhaust": "oldest first",
            "deck_rows": "every entry deck card exactly once", "generated": "statuses (wound, dazed, burn, slimed, void) and the character's generated cards; opening-generated cards are discarded and de-issued"},
        "never": "hidden state is never an input (draw order, seeds and random streams, intents, AI memory, card instances or handles, the turn number) and neither are relic counters, HP, max HP or items",
        "label": "synthetic_position: native-consistent synthetic state, never reachable gameplay"})
}
