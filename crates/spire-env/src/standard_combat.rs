//! M34s-a `standard-combat-v1` catalogs (design §4.4): strict owned Standard resources for every character at A0–A20,
//! one entry per Standard setup row, optional explicit burning-elite buff. The session is the Standard arm of the shared
//! `CombatTrainingSession`; the observation is the Standard combat view.
use crate::RunSessionError as Error;
use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use spire_core::{
    combat::enemies::A0_ENCOUNTER_SETUPS,
    content::{complete, CardId, Character},
    run::{AuthoredRunStart, BurningEliteBuff, CardInstance, StandardCombatStart, STANDARD_MAX_ASCENSION},
};

pub const STANDARD_COMBAT_PROFILE_ID: &str = "standard-combat-v1";
pub const STANDARD_COMBAT_CATALOG_SCHEMA_ID: &str = "spire-gym-standard-combat-catalog-v1";
const CATALOG_DOMAIN: &[u8] = b"spire-gym/standard-combat-catalog/v1\0";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawCatalog { catalog_schema_id: String, profile_id: String, entries: Vec<Value> }
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawCard { card: String, upgrades: u32, #[serde(default)] misc: i32 }
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawEntry {
    id: String, character: String, ascension: u8, encounter: String, #[serde(default)] burning: Option<String>,
    hp: u32, max_hp: u32, gold: u32, deck: Vec<RawCard>, relics: Vec<String>,
    #[serde(default)] relic_counters: std::collections::BTreeMap<String, i32>, potions: Vec<Option<String>>,
    #[serde(default)] bottled_cards: std::collections::BTreeMap<String, Option<RawCard>>,
    #[serde(default)] position: Option<Value>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawRecipeEntry { id: String, recipe: RawRecipe }
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawRecipe { record_sha256: String, steps: usize }

/// One parsed catalog entry: an owned start (fresh or with a synthetic position) or a reachable recipe (its cached public
/// root, sampled per reset).
#[derive(Clone, Debug)]
pub(crate) enum StandardEntry { Start(StandardCombatStart), Recipe(crate::PublicRoot) }
pub const STANDARD_RECORD_MAX_BYTES: usize = 8 * 1024 * 1024;
pub const STANDARD_RECORDS_MAX_TOTAL_BYTES: usize = 64 * 1024 * 1024;

fn character(name: &str) -> Result<Character, Error> {
    [Character::Ironclad, Character::Silent, Character::Defect, Character::Watcher].into_iter()
        .find(|c| c.as_str() == name).ok_or(Error::Input("unknown Standard combat character"))
}
fn found<T: Copy>(name: &str, ids: &[T], label: impl Fn(T) -> &'static str, error: &'static str) -> Result<T, Error> {
    ids.iter().copied().find(|id| label(*id) == name).ok_or(Error::Input(error))
}

/// Parse, validate and normalise a catalog: entries keep their order; each deck is a canonical multiset (card, upgrades,
/// misc) with instances 1..n; relics keep their acquisition order. Returns the entries and the normalised descriptor.
pub(crate) fn parse(bytes: &[u8], records: &[Vec<u8>]) -> Result<(Vec<(String, StandardEntry)>, Value, [u8; 32]), Error> {
    if bytes.len() > 1_048_576 { return Err(Error::Input("combat catalog exceeds 1 MiB")) }
    // A Gym66 catalog (profile `ironclad-combat-v1/v2`, or none: Gym66 defaults to v1) is never read as a Standard one.
    if serde_json::from_slice::<Value>(bytes).ok().is_some_and(|v| v.is_object() && v.get("catalog_schema_id").is_none()
        && v.get("profile_id").is_none_or(|p| p.as_str().is_some_and(|p| p.starts_with("ironclad-combat-")))) {
        return Err(Error::UnsupportedBoundary("the Standard combat session requires a standard-combat-v1 catalog"))
    }
    let raw: RawCatalog = serde_json::from_slice(bytes).map_err(|_| Error::Input("invalid Standard combat catalog fields or types"))?;
    if raw.catalog_schema_id != STANDARD_COMBAT_CATALOG_SCHEMA_ID || raw.profile_id != STANDARD_COMBAT_PROFILE_ID {
        return Err(Error::Input("Standard combat catalog identity mismatch"))
    }
    if !(1..=256).contains(&raw.entries.len()) { return Err(Error::Input("catalog requires 1..256 entries")) }
    let records = records_by_sha(records)?;
    let mut ids = std::collections::BTreeSet::new();
    let mut entries = Vec::new();
    let mut normalized = Vec::new();
    for value in raw.entries {
        let id_ok = |id: &str, ids: &mut std::collections::BTreeSet<String>| !id.is_empty() && id.len() <= 64
            && id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-') && ids.insert(id.to_owned());
        if value.get("recipe").is_some() {
            let r: RawRecipeEntry = serde_json::from_value(value).map_err(|_| Error::Input("invalid recipe entry fields or types"))?;
            if !id_ok(&r.id, &mut ids) { return Err(Error::Input("invalid or duplicate combat entry ID")) }
            let record = records.get(r.recipe.record_sha256.as_str()).ok_or(Error::Input("recipe.record_sha256: record not supplied"))?;
            let root = recipe_root(record, r.recipe.steps)?;
            normalized.push(json!({"id": r.id, "start_kind": "reachable_recipe", "recipe": {"record_sha256": r.recipe.record_sha256, "steps": r.recipe.steps}}));
            entries.push((r.id, StandardEntry::Recipe(root)));
            continue
        }
        let r: RawEntry = serde_json::from_value(value).map_err(|_| Error::Input("invalid Standard combat catalog fields or types"))?;
        if !id_ok(&r.id, &mut ids) { return Err(Error::Input("invalid or duplicate combat entry ID")) }
        let character = character(&r.character)?;
        if r.ascension > STANDARD_MAX_ASCENSION { return Err(Error::Input("Standard combat ascension must be 0..20")) }
        let encounter = A0_ENCOUNTER_SETUPS.iter().map(|(e, _, _)| *e).find(|e| e.as_str() == r.encounter)
            .ok_or(Error::Input("encounter outside the Standard setup table"))?;
        let burning = r.burning.as_deref().map(|b| found(b, &BurningEliteBuff::ALL, BurningEliteBuff::as_str, "unknown burning-elite buff")).transpose()?;
        let mut deck = r.deck.iter().map(|c| Ok((CardId::from_name(&c.card).ok_or(Error::Input("unknown Standard card"))?, c.upgrades, c.misc)))
            .collect::<Result<Vec<_>, Error>>()?;
        deck.sort_by_key(|(id, upgrades, misc)| (id.as_str(), *upgrades, *misc));
        let relic_ids = complete::relic_union(character).into_iter().chain([spire_core::content::RelicId::RedCirclet]).collect::<Vec<_>>();
        let relics = r.relics.iter().map(|n| found(n, &relic_ids, |id| id.as_str(), "relic outside the character's Standard relics")).collect::<Result<Vec<_>, _>>()?;
        let counters = r.relic_counters.iter().map(|(n, v)| Ok((found(n, &relic_ids, |id| id.as_str(), "relic counter outside the character's Standard relics")?, *v)))
            .collect::<Result<std::collections::BTreeMap<_, _>, Error>>()?;
        let potion_ids = complete::potion_union(character);
        let potions = r.potions.iter().map(|p| p.as_deref().map(|n| found(n, &potion_ids, |id| id.as_str(), "potion outside the character's Standard potions")).transpose())
            .collect::<Result<Vec<_>, _>>()?;
        let resources = AuthoredRunStart {
            hp: r.hp, max_hp: r.max_hp, gold: r.gold,
            deck: deck.iter().enumerate().map(|(i, (id, upgrades, misc))| CardInstance { id: *id, instance: i as u64 + 1, upgrades: *upgrades, misc: *misc }).collect(),
            relics, potions, relic_counters: counters,
        };
        // PM ruling c0 (6): an owned Bottled relic names its bottled deck card (the lowest instance of that row), or `null`
        // for an owned bottle with no card (final review m1: native when no card was eligible at pickup or it was removed).
        let bottled_cards = r.bottled_cards.iter().map(|(relic, c)| {
            let relic = found(relic, &relic_ids, |id| id.as_str(), "bottled_cards: relic outside the character's Standard relics")?;
            let Some(c) = c else { return Ok((relic, None)) };
            let id = CardId::from_name(&c.card).ok_or(Error::Input("bottled_cards: unknown card"))?;
            let card = resources.deck.iter().find(|d| (d.id, d.upgrades, d.misc) == (id, c.upgrades, c.misc)).ok_or(Error::Input("bottled_cards: card is not in the deck"))?;
            Ok((relic, Some(card.instance)))
        }).collect::<Result<Vec<_>, Error>>()?;
        let position = r.position.as_ref().map(crate::standard_positions::parse_position).transpose()?;
        let start = StandardCombatStart { character, ascension: r.ascension, encounter, burning, resources, bottled_cards, position };
        start.validate_bottled_cards().map_err(Error::Input)?;
        start.validate_position().map_err(crate::standard_positions::position_error)?;
        // The core owner validates every resource rule (complete unions, upgrades, misc, slots, counters, burning).
        // A trial entry checks setup admission (`enemies::setup`) and the position's construction before any session exists.
        let mut trial = spire_core::run::Run::prepare_standard_combat(start.clone(), spire_core::run::RunRng::new([0; 32], spire_core::combat::rng::Partition::Test, 0))
            .map_err(|_| Error::Input("invalid Standard combat entry resources"))?;
        trial.enter_standard_combat_position().map_err(|e| match e {
            spire_core::run::StandardStartError::Position(p) => crate::standard_positions::position_error(p),
            spire_core::run::StandardStartError::Run(_) => Error::Input("Standard combat entry setup rejected (deck plus entry-generated cards above 999, or an inadmissible setup)"),
        })?;
        let mut entry = json!({
            "id": r.id, "character": character.as_str(), "ascension": r.ascension, "encounter": encounter.as_str(),
            "burning": burning.map(BurningEliteBuff::as_str), "hp": r.hp, "max_hp": r.max_hp, "gold": r.gold,
            "deck": deck.iter().map(|(c, u, m)| json!({"card": c.as_str(), "upgrades": u, "misc": m})).collect::<Vec<_>>(),
            "relics": start.resources.relics.iter().map(|id| id.as_str()).collect::<Vec<_>>(),
            "relic_counters": start.resources.relic_counters.iter().map(|(id, v)| (id.as_str().to_owned(), json!(v))).collect::<serde_json::Map<_, _>>(),
            "potions": start.resources.potions.iter().map(|p| p.map(|id| id.as_str())).collect::<Vec<_>>(),
            "start_kind": if start.position.is_some() { "synthetic_position" } else { "fresh_start" },
        });
        if !r.bottled_cards.is_empty() {
            entry["bottled_cards"] = start.bottled_cards.iter().map(|(relic, i)| (relic.as_str().to_owned(), i.map_or(Value::Null, |i| {
                let c = start.resources.deck.iter().find(|d| d.instance == i).unwrap();
                json!({"card": c.id.as_str(), "upgrades": c.upgrades, "misc": c.misc}) }))).collect::<serde_json::Map<_, _>>().into();
        }
        if let Some(position) = &start.position { entry["position"] = crate::standard_positions::position_json(position); }
        normalized.push(entry);
        entries.push((r.id, StandardEntry::Start(start)));
    }
    let descriptor = json!({"catalog_schema_id": STANDARD_COMBAT_CATALOG_SCHEMA_ID, "profile_id": STANDARD_COMBAT_PROFILE_ID, "entries": normalized});
    let mut h = Sha256::new();
    h.update(CATALOG_DOMAIN);
    h.update(crate::canonical_json(&descriptor).map_err(|_| Error::Input("invalid catalog"))?);
    Ok((entries, descriptor, h.finalize().into()))
}

/// The records map of a catalog (design §4.8): each ≤ 8 MiB, total ≤ 64 MiB, keyed by its lowercase hex SHA-256.
fn records_by_sha(records: &[Vec<u8>]) -> Result<std::collections::BTreeMap<String, &[u8]>, Error> {
    if records.iter().any(|r| r.len() > STANDARD_RECORD_MAX_BYTES) { return Err(Error::Input("records: a record exceeds 8 MiB")) }
    if records.iter().map(Vec::len).sum::<usize>() > STANDARD_RECORDS_MAX_TOTAL_BYTES { return Err(Error::Input("records: records exceed 64 MiB in total")) }
    Ok(records.iter().map(|r| { let d: [u8; 32] = Sha256::digest(r).into(); (crate::combat_policy::hex(&d), r.as_slice()) }).collect())
}
/// Load a recipe once (design §4.8, plan c3): the record validates its prefix (`RunSession::from_record`), which must stop at
/// a combat decision; its public root is cached. The record's own hidden state is never kept.
fn recipe_root(record: &[u8], steps: usize) -> Result<crate::PublicRoot, Error> {
    #[cfg(test)]
    RECIPE_LOADS.with(|n| n.set(n.get() + 1));
    let value: Value = serde_json::from_slice(record).map_err(|_| Error::Input("recipe: record is not JSON"))?;
    if value.get("profile_id").and_then(Value::as_str) != Some("Standard") { return Err(Error::Input("recipe: record is not a Standard Run record")) }
    let record = crate::RunEpisodeRecord::from_json(record)?;
    let session = crate::RunSession::from_record(&record, steps, crate::RunCapacities::standard_default())?;
    session.public_root().map_err(|e| match e {
        Error::UnsupportedBoundary(m) => Error::UnsupportedBoundary(m),
        _ => Error::UnsupportedBoundary("recipe: the prefix does not stop at a supported combat decision"),
    })
}
#[cfg(test)]
thread_local! { pub(crate) static RECIPE_LOADS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) }; }

/// The finite input support of `standard-combat-v1`.
pub fn standard_combat_support() -> Value {
    json!({
        "characters": ["ironclad", "silent", "defect", "watcher"], "ascension": {"minimum": 0, "maximum": STANDARD_MAX_ASCENSION},
        "encounters": A0_ENCOUNTER_SETUPS.iter().map(|(e, a, k)| json!({"id": e.as_str(), "act": a, "kind": k.label()})).collect::<Vec<_>>(),
        "burning": {"buffs": BurningEliteBuff::ALL.iter().map(|b| b.as_str()).collect::<Vec<_>>(), "requires": "an elite encounter"},
        "hp": {"minimum": 1, "maximum": 999}, "deck": {"minimum": 1, "maximum": 999, "ordering": "unordered canonical multiset (card, upgrades, misc)",
            "entry_generated": "cards the entry generates (e.g. Mark of Pain's Wounds) count toward the engine's 999-card bound"},
        "cards": "the complete permanent union; upgrades as the Run's, Searing Blow up to 73 (damage within the engine's 3003 card base-damage surface); misc 0, Genetic Algorithm 1..=999, Ritual Dagger 15..=3003",
        "relics": "the character's complete relic union plus Red Circlet, in acquisition order; counters in their domains",
        "potions": "the character's complete potion union; the slot count is the ascension's base (3, or 2 from A11) plus 2 with Potion Belt",
        "gold": {"minimum": 0, "maximum": u32::MAX},
        "resource_semantics": "already owned; acquisition effects are not repeated",
        "maximum_catalog_entries": 256, "maximum_catalog_bytes": 1_048_576,
        "terminal": "the combat's own terminal: victory +1 (all-enemies-escaped victory flagged), death -1, escaped 0; no settlement, rewards or Run-level settlement effects",
        "bottled_cards": "an owned Bottled relic names its bottled deck card (a purgeable card of the bottle's type; innate as native), or null for an owned bottle with no card",
        "start_kinds": {"fresh_start": "owned resources and an encounter; no reachability claim",
            "synthetic_position": "plus a spire-gym-combat-position-v1 position at the opening decision; native-consistent synthetic state, never reachable gameplay",
            "reachable_recipe": "a Standard record prefix stopping at a combat decision (records map by SHA-256, each <= 8 MiB, total <= 64 MiB); hidden state re-sampled from its public root per reset"},
        "position": crate::standard_positions::position_support(),
        "unsupported": "hidden state, draw order, seeds and RNG in any input; turn-N, mid-turn and selection-boundary positions; enemy intents and AI memory; original-runtime parity",
    })
}
