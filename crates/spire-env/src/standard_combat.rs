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
struct RawCatalog { catalog_schema_id: String, profile_id: String, entries: Vec<RawEntry> }
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawCard { card: String, upgrades: u32, #[serde(default)] misc: i32 }
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawEntry {
    id: String, character: String, ascension: u8, encounter: String, #[serde(default)] burning: Option<String>,
    hp: u32, max_hp: u32, gold: u32, deck: Vec<RawCard>, relics: Vec<String>,
    #[serde(default)] relic_counters: std::collections::BTreeMap<String, i32>, potions: Vec<Option<String>>,
}

fn character(name: &str) -> Result<Character, Error> {
    [Character::Ironclad, Character::Silent, Character::Defect, Character::Watcher].into_iter()
        .find(|c| c.as_str() == name).ok_or(Error::Input("unknown Standard combat character"))
}
fn found<T: Copy>(name: &str, ids: &[T], label: impl Fn(T) -> &'static str, error: &'static str) -> Result<T, Error> {
    ids.iter().copied().find(|id| label(*id) == name).ok_or(Error::Input(error))
}

/// Parse, validate and normalise a catalog: entries keep their order; each deck is a canonical multiset (card, upgrades,
/// misc) with instances 1..n; relics keep their acquisition order. Returns the entries and the normalised descriptor.
pub(crate) fn parse(bytes: &[u8]) -> Result<(Vec<(String, StandardCombatStart)>, Value, [u8; 32]), Error> {
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
    let mut ids = std::collections::BTreeSet::new();
    let mut entries = Vec::new();
    let mut normalized = Vec::new();
    for r in raw.entries {
        if r.id.is_empty() || r.id.len() > 64 || !r.id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-') || !ids.insert(r.id.clone()) {
            return Err(Error::Input("invalid or duplicate combat entry ID"))
        }
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
        let start = StandardCombatStart { character, ascension: r.ascension, encounter, burning, resources };
        // The core owner validates every resource rule (complete unions, upgrades, misc, slots, counters, burning).
        // A trial entry checks setup admission (`enemies::setup`) before any session exists.
        let mut trial = spire_core::run::Run::prepare_standard_combat(start.clone(), spire_core::run::RunRng::new([0; 32], spire_core::combat::rng::Partition::Test, 0))
            .map_err(|_| Error::Input("invalid Standard combat entry resources"))?;
        trial.enter_standard_combat().map_err(|_| Error::Input("Standard combat entry setup rejected (deck plus entry-generated cards above 999, or an inadmissible setup)"))?;
        normalized.push(json!({
            "id": r.id, "character": character.as_str(), "ascension": r.ascension, "encounter": encounter.as_str(),
            "burning": burning.map(BurningEliteBuff::as_str), "hp": r.hp, "max_hp": r.max_hp, "gold": r.gold,
            "deck": deck.iter().map(|(c, u, m)| json!({"card": c.as_str(), "upgrades": u, "misc": m})).collect::<Vec<_>>(),
            "relics": start.resources.relics.iter().map(|id| id.as_str()).collect::<Vec<_>>(),
            "relic_counters": start.resources.relic_counters.iter().map(|(id, v)| (id.as_str().to_owned(), json!(v))).collect::<serde_json::Map<_, _>>(),
            "potions": start.resources.potions.iter().map(|p| p.map(|id| id.as_str())).collect::<Vec<_>>(),
        }));
        entries.push((r.id, start));
    }
    let descriptor = json!({"catalog_schema_id": STANDARD_COMBAT_CATALOG_SCHEMA_ID, "profile_id": STANDARD_COMBAT_PROFILE_ID, "entries": normalized});
    let mut h = Sha256::new();
    h.update(CATALOG_DOMAIN);
    h.update(crate::canonical_json(&descriptor).map_err(|_| Error::Input("invalid catalog"))?);
    Ok((entries, descriptor, h.finalize().into()))
}

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
        "unsupported": "hidden state, draw order, seeds and RNG; positions and recipes (M34s-c); original-runtime parity",
    })
}
