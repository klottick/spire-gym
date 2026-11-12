//! M38e: the opt-in compact Standard Run observation (`spire-gym-standard-run-observation-compact-v1`). The dense
//! groups, columns, order and semantics with rows cut to the `standard-compact-v1` capacities and values in int16/int32.
use crate::*;
use crate::run_schema_standard::{compact_dtype, ValueDtype};
use serde_json::{json, Value};

fn hex(bytes: [u8; 32]) -> String { bytes.iter().map(|b| format!("{b:02x}")).collect() }
fn is_value_tensor(t: &Value) -> bool { t.get("known").is_some() }
fn number(v: &Value) -> i64 { v.as_i64().unwrap_or_else(|| if v.as_u64().is_some() { i64::MAX } else { panic!("integer bound {v}") }) }
fn clip(v: &Value, (low, high): (i64, i64)) -> i64 { number(v).clamp(low, high) }

/// Pinned at the M38e merge (frozen from then on; `m36_freeze::frozen_identities` carries the same rows).
const COMPACT_FINGERPRINT: &str = "633b9424be5070a2890b01725949447badba919c5c8c5bd798922f8c2d2a2b2d";
const DENSE_FINGERPRINT: &str = "bf8e1d111bbf2171594231b99d8d6b701244249abfbb90a11e86c4b4a405f7ae";

/// Design §6: the value dtype of every Standard group (dense is int64 everywhere).
const DTYPES: &[(&str, &str)] = &[
    ("context", "int32"), ("player", "int32"), ("map_nodes", "int32"), ("map_edges", "int16"), ("current_node", "int16"),
    ("permanent_cards", "int32"), ("combat_cards", "int32"), ("enemies", "int32"), ("powers", "int32"), ("relics", "int32"),
    ("potions", "int16"), ("offers", "int32"), ("selection", "int16"), ("selection_counts", "int16"), ("orbs", "int32"),
    ("actions", "int16"), ("combat_state", "int16"), ("known_draw", "int16"), ("stance_choices", "int16"),
    ("combat_card_offers", "int16"), ("room_params", "int32"),
];

/// E6 (review M1): at the SAME capacities, compact differs from dense only in `schema_id`, value-tensor `dtype`,
/// `low`/`high` (and `columns[i]`) clipped to the dtype, `semantics.capacity_preset` and the new `semantics.layout` /
/// `semantics.dense_schema`. Undoing exactly those differences gives the dense descriptor.
#[test]
fn compact_descriptor_differs_from_dense_only_in_documented_keys() {
    for caps in [RunCapacities::standard_compact(), RunCapacities::standard_default()] {
        let dense = standard_run_observation_spec(&caps).unwrap().descriptor();
        let compact = standard_compact_run_observation_spec(&caps).unwrap().descriptor();
        assert_eq!(compact["schema_id"], RUN_OBSERVATION_STANDARD_COMPACT_SCHEMA_ID);
        assert_eq!(compact["semantics"]["capacity_preset"], STANDARD_COMPACT_CAPACITY_PRESET_ID);
        assert_eq!(compact["semantics"]["dense_schema"], RUN_OBSERVATION_STANDARD_SCHEMA_ID);
        assert!(compact["semantics"]["layout"].as_str().is_some_and(|s| s.contains("never truncated")));
        let mut undone = compact.clone();
        undone["schema_id"] = dense["schema_id"].clone();
        undone["semantics"]["capacity_preset"] = dense["semantics"]["capacity_preset"].clone();
        let semantics = undone["semantics"].as_object_mut().unwrap();
        semantics.remove("layout");
        semantics.remove("dense_schema");
        let dense_tensors = dense["tensors"].as_array().unwrap();
        let mut clipped = 0;
        for (t, d) in undone["tensors"].as_array_mut().unwrap().iter_mut().zip(dense_tensors) {
            assert_eq!(t["key"], d["key"]);
            if !is_value_tensor(d) { continue }
            assert_eq!(d["dtype"], "int64");
            let dtype = compact_dtype(d["key"].as_str().unwrap()).expect("every Standard group has a compact dtype");
            assert_eq!(t["dtype"], dtype.name());
            for bound in ["low", "high"] {
                for (i, dv) in d[bound].as_array().unwrap().iter().enumerate() {
                    let want = clip(dv, dtype.range());
                    assert_eq!(number(&t[bound][i]), want, "{}[{i}].{bound}", d["key"]);
                    assert_eq!(number(&t["columns"][i][bound]), want, "{} columns[{i}].{bound}", d["key"]);
                    if want != number(dv) { clipped += 1 }
                    t[bound][i] = dv.clone();
                    t["columns"][i][bound] = d["columns"][i][bound].clone();
                }
            }
            t["dtype"] = json!("int64");
        }
        assert!(clipped > 0, "some dense domains exceed the compact dtypes");
        assert_eq!(undone, dense, "compact must differ from dense only in the documented keys");
    }
}

/// E6 (second half): tensor shapes and the action space follow the capacities.
#[test]
fn compact_shapes_follow_capacities() {
    let caps = RunCapacities { map_nodes: 50, map_edges: 60, permanent_cards: 30, combat_cards: 40, enemies: 6, powers: 20, relics: 25,
        potions: 5, offers: 15, selection: 35, orbs: 4, actions: 45 };
    let compact = standard_compact_run_observation_spec(&caps).unwrap().descriptor();
    let dense = standard_run_observation_spec(&caps).unwrap().descriptor();
    for (t, d) in compact["tensors"].as_array().unwrap().iter().zip(dense["tensors"].as_array().unwrap()) { assert_eq!(t["shape"], d["shape"]) }
    assert_eq!(compact["action_space"], json!({"type": "Discrete", "n": 45, "start": 0}));
    let shape = |key: &str| compact["tensors"].as_array().unwrap().iter().find(|t| t["key"] == key).unwrap()["shape"].clone();
    assert_eq!(shape("combat_cards")[0], 40);
    assert_eq!(shape("known_draw")[0], 40, "known_draw rows are the combat_cards capacity");
    assert_eq!(shape("combat_card_offers")[0], 15, "combat_card_offers rows are the offers capacity");
    assert_eq!(shape("action_mask"), json!([45]));
}

/// The frozen compact identity: the preset, the dtype table, the clipped column bounds and the fingerprint.
#[test]
fn compact_identity_pinned() {
    let caps = RunCapacities::standard_compact();
    assert_eq!(caps, RunCapacities { map_nodes: 107, map_edges: 128, permanent_cards: 256, combat_cards: 512, enemies: 12, powers: 128,
        relics: 256, potions: 16, offers: 128, selection: 512, orbs: 10, actions: 576 });
    let spec = standard_compact_run_observation_spec(&caps).unwrap();
    let d = spec.descriptor();
    let tensors = d["tensors"].as_array().unwrap();
    let dtypes = tensors.iter().filter(|t| is_value_tensor(t)).map(|t| (t["key"].as_str().unwrap().to_owned(), t["dtype"].as_str().unwrap().to_owned())).collect::<Vec<_>>();
    assert_eq!(dtypes, DTYPES.iter().map(|(k, v)| ((*k).to_owned(), (*v).to_owned())).collect::<Vec<_>>());
    let size = |t: &Value| match t["dtype"].as_str().unwrap() { "int8" => 1, "int16" => 2, "int32" => 4, other => panic!("{other}") };
    let bytes: u64 = tensors.iter().map(|t| t["shape"].as_array().unwrap().iter().map(|n| n.as_u64().unwrap()).product::<u64>() * size(t)).sum();
    assert_eq!(bytes, 136_842, "compact array bytes per slot (design §3.1)");
    let mut clipped = Vec::new();
    for t in tensors.iter().filter(|t| is_value_tensor(t)) {
        for column in t["columns"].as_array().unwrap() {
            let (low, high) = compact_dtype(t["key"].as_str().unwrap()).unwrap().range();
            if number(&column["low"]) == low || number(&column["high"]) == high {
                clipped.push(format!("{}.{}:{}..{}", t["key"].as_str().unwrap(), column["field"].as_str().unwrap(), column["low"], column["high"]));
            }
        }
    }
    assert_eq!(clipped.join(" "), CLIPPED_COLUMNS, "value columns at their dtype bounds");
    assert_eq!(d["semantics"]["capacity_preset"], STANDARD_COMPACT_CAPACITY_PRESET_ID);
    assert_eq!(hex(spec.fingerprint()), COMPACT_FINGERPRINT, "frozen compact fingerprint");
}
/// Value columns whose compact bound is the dtype bound (clipped from the dense domain, or an `i32` source at its own range).
const CLIPPED_COLUMNS: &str = concat!(
    "context.floor:0..2147483647 context.reward_total:-2147483648..2147483647 context.winged_moves:0..2147483647 ",
    "context.combats_entered:0..2147483647 context.combats_settled:0..2147483647 player.hp:0..2147483647 ",
    "player.max_hp:0..2147483647 player.gold:0..2147483647 player.block:-2147483648..2147483647 ",
    "player.energy:-2147483648..2147483647 player.max_energy:-2147483648..2147483647 player.turn:0..2147483647 ",
    "player.draw_count:0..2147483647 player.focus:-2147483648..2147483647 player.hp_before_healing:0..2147483647 ",
    "player.hp_after_cleanup:0..2147483647 player.mantra_gained:-2147483648..2147483647 ",
    "map_nodes.floor:0..2147483647 permanent_cards.upgrades:0..2147483647 ",
    "permanent_cards.misc:-2147483648..2147483647 combat_cards.zone_position:-1..2147483647 ",
    "combat_cards.upgrades:0..2147483647 combat_cards.effective_cost:-2147483648..2147483647 ",
    "combat_cards.base_damage:-2147483648..2147483647 combat_cards.base_block:-2147483648..2147483647 ",
    "combat_cards.magic_number:-2147483648..2147483647 combat_cards.damage_bonus:-2147483648..2147483647 ",
    "combat_cards.temporary_cost:-2147483648..2147483647 combat_cards.misc:-2147483648..2147483647 ",
    "combat_cards.combat_cost:-2147483648..2147483647 combat_cards.turn_cost:-2147483648..2147483647 ",
    "combat_cards.rolled_cost:-2147483648..2147483647 combat_cards.modified_combat_cost:-2147483648..2147483647 ",
    "combat_cards.hp_loss_discount:-2147483648..2147483647 enemies.hp:0..2147483647 enemies.max_hp:0..2147483647 ",
    "enemies.block:-2147483648..2147483647 enemies.intent_damage:0..2147483647 enemies.intent_hits:0..2147483647 ",
    "powers.amount:-2147483648..2147483647 relics.visible_counter:-2147483648..2147483647 ",
    "offers.price:0..2147483647 offers.amount:0..2147483647 offers.upgrades:0..2147483647 ",
    "offers.required_count:0..2147483647 offers.selected_count:0..2147483647 offers.param0:-2147483648..2147483647 ",
    "offers.param1:-2147483648..2147483647 offers.param2:-2147483648..2147483647 ",
    "selection.reveal_position:0..32767 selection.selected_ordinal:0..32767 selection_counts.required:0..32767 ",
    "selection_counts.selected_count:0..32767 selection_counts.minimum:0..32767 selection_counts.maximum:0..32767 ",
    "orbs.dark_damage:-2147483648..2147483647 actions.scalar0:0..32767 actions.scalar1:0..32767 ",
    "known_draw.offset:0..32767 room_params.stage0:-2147483648..2147483647 ",
    "room_params.stage1:-2147483648..2147483647 room_params.stage2:-2147483648..2147483647 ",
    "room_params.stage3:-2147483648..2147483647",
);

/// Every value tensor's declared bounds fit its dtype (the encode-time check uses the dtype range itself, review M14).
#[test]
fn compact_dtypes_hold_declared_ranges() {
    for caps in [RunCapacities::standard_compact(), RunCapacities::standard_default()] {
        let d = standard_compact_run_observation_spec(&caps).unwrap().descriptor();
        for t in d["tensors"].as_array().unwrap() {
            let (low, high) = match t["dtype"].as_str().unwrap() { "int8" => (-128, 127), "int16" => (-32_768, 32_767),
                "int32" => (i64::from(i32::MIN), i64::from(i32::MAX)), other => panic!("compact dtype {other}") };
            let bounds = |v: &Value| v.as_array().cloned().unwrap_or_else(|| vec![v.clone()]);
            for (l, h) in bounds(&t["low"]).iter().zip(bounds(&t["high"])) {
                assert!(low <= number(l) && number(l) <= number(&h) && number(&h) <= high, "{} {l}..{h}", t["key"]);
            }
        }
    }
}

/// Compact capacities are 1..=`standard-default-v1` per group; the preset passes.
#[test]
fn compact_capacities_validate() {
    assert!(standard_compact_run_observation_spec(&RunCapacities::standard_compact()).is_ok());
    assert!(standard_compact_run_observation_spec(&RunCapacities::standard_default()).is_ok());
    let edits: [fn(&mut RunCapacities, usize); 12] = [|c, v| c.map_nodes = v, |c, v| c.map_edges = v, |c, v| c.permanent_cards = v,
        |c, v| c.combat_cards = v, |c, v| c.enemies = v, |c, v| c.powers = v, |c, v| c.relics = v, |c, v| c.potions = v,
        |c, v| c.offers = v, |c, v| c.selection = v, |c, v| c.orbs = v, |c, v| c.actions = v];
    let default = RunCapacities::standard_default();
    let limits = [default.map_nodes, default.map_edges, default.permanent_cards, default.combat_cards, default.enemies, default.powers,
        default.relics, default.potions, default.offers, default.selection, default.orbs, default.actions];
    for (edit, limit) in edits.iter().zip(limits) {
        for (value, ok) in [(0, false), (1, true), (limit, true), (limit + 1, false)] {
            let mut caps = RunCapacities::standard_compact();
            edit(&mut caps, value);
            let result = standard_compact_run_observation_spec(&caps);
            assert_eq!(result.is_ok(), ok, "{caps:?}");
            if !ok { assert!(matches!(result, Err(RunSessionError::Input(_))), "{caps:?}") }
        }
    }
}

/// Characterisation (review M13; passes as soon as it compiles): the dense descriptor is untouched.
#[test]
fn dense_fingerprint_unchanged() {
    assert_eq!(hex(standard_run_observation_spec(&RunCapacities::standard_default()).unwrap().fingerprint()), DENSE_FINGERPRINT);
    assert_eq!(RunObservationLayout::default(), RunObservationLayout::Dense);
    assert_eq!((RunObservationLayout::Dense.as_str(), RunObservationLayout::Compact.as_str()), ("dense", "compact"));
}

/// The value-fault group names: `"<group>.<column>"` for every Standard value column, with that group's dtype.
#[test]
fn value_column_names() {
    use crate::run_schema_standard::{value_column, value_fault_dtype};
    assert_eq!(value_column("context", 3), "context.floor");
    assert_eq!(value_column("player", 2), "player.gold");
    assert_eq!(value_column("combat_cards", 0), "combat_cards.zone");
    assert_eq!(value_fault_dtype("player.gold"), Some(ValueDtype::I32));
    assert_eq!(value_fault_dtype("selection.source_row"), Some(ValueDtype::I16));
    for bad in ["player", "player.", "player.nope", "combat_cards", "known_draw.x", ".gold", ""] { assert_eq!(value_fault_dtype(bad), None, "{bad}") }
    let d = standard_compact_run_observation_spec(&RunCapacities::standard_compact()).unwrap().descriptor();
    let mut n = 0;
    for t in d["tensors"].as_array().unwrap().iter().filter(|t| is_value_tensor(t)) {
        let key = t["key"].as_str().unwrap();
        for (i, f) in t["fields"].as_array().unwrap().iter().enumerate() {
            let name = format!("{key}.{}", f.as_str().unwrap());
            assert_eq!(value_column(key, i), name);
            assert_eq!(value_fault_dtype(&name), compact_dtype(key));
            n += 1;
        }
    }
    assert_eq!(n, 155, "lane B counted 155 Standard value columns");
}

// ---- Task 2: the encoder's dtype check and typed targets ----

use crate::run_observation::{encode_run_standard_layout_into, test_range, value_fault, ObservationChunk, StandardContext};
use crate::run_session::{create_run, RunProfile};
use spire_core::{combat::rng::Partition, content::Character, run::{map::PublicMap, PublicDecision, Run}};
use std::mem::MaybeUninit;

pub(super) const CHARACTERS: [Character; 4] = [Character::Ironclad, Character::Silent, Character::Defect, Character::Watcher];
pub(super) fn mix(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^ (x >> 31)
}
type Boundary = (PublicDecision, Option<PublicMap>, StandardContext);
/// Real Standard boundaries of a seeded uniform-random Run (the core Run, no session).
fn boundaries(character: Character, ascension: u8, seed: u64, steps: usize) -> Vec<Boundary> {
    let config = StandardRunConfig::new(character, ascension, Partition::Test).session_config();
    let mut run: Run = create_run(RunProfile::Standard, &config, seed, 0).unwrap();
    let mut out = Vec::new();
    for step in 0..steps {
        let d = run.decision().unwrap();
        out.push((d.clone(), run.public_map(), StandardContext { ascension: run.ascension(), run_selection: run.room_selection_operation() }));
        if d.terminal.is_some() || d.candidates.is_empty() { break }
        let choice = (mix(seed ^ (step as u64).wrapping_mul(0x100_0000_01B3)) % d.candidates.len() as u64) as usize;
        run.step_offered(&d.candidates[choice].action).unwrap();
    }
    out
}
fn encode(b: &Boundary, caps: &RunCapacities, layout: RunObservationLayout) -> Result<RunObservation, RunSessionError> {
    encode_run_standard_layout_into(&b.0, b.1.as_ref(), caps, b.2, layout, None)
}
/// A combat boundary with at least `cards` combat card rows.
fn combat_boundary(cards: usize) -> Boundary {
    for seed in 0..64 {
        for b in boundaries(Character::Ironclad, 20, 0x5EED + seed, 400) {
            if b.0.combat.as_ref().is_some_and(|c| c.hand.len() + c.draw_unknown.len() + c.discard.len() >= cards) { return b }
        }
    }
    panic!("no combat boundary with {cards} cards");
}
/// Capacities used only by one test (the test range override is keyed by capacities, so parallel tests never see it).
fn unique_caps(tag: usize) -> RunCapacities { RunCapacities { orbs: 10 - tag, ..RunCapacities::standard_compact() } }

/// Characterisation: a row overflow in the compact layout is the existing typed `Capacity` fault.
#[test]
fn compact_row_overflow_is_capacity_fault() {
    let b = combat_boundary(10);
    let c = b.0.combat.as_ref().unwrap();
    let rows = c.hand.len() + c.draw_unknown.len() + c.discard.len() + c.exhaust.len() + c.played_powers.len();
    let caps = RunCapacities { combat_cards: rows - 1, ..RunCapacities::standard_compact() };
    let err = encode(&b, &caps, RunObservationLayout::Compact).unwrap_err();
    assert!(matches!(err, RunSessionError::Capacity { group: "combat_cards", capacity, .. } if capacity == rows - 1), "{err:?}");
    assert_eq!(encode(&b, &caps, RunObservationLayout::Dense).unwrap_err(), err, "row overflow does not depend on the layout");
}

/// A value outside the group's dtype range is `Capacity { group: "<group>.<column>", required: |v|, capacity: max }`;
/// the dense encode of the same boundary passes no range.
#[test]
fn compact_value_fault_names_the_column() {
    let b = combat_boundary(10);
    let caps = unique_caps(1);
    let compact = encode(&b, &caps, RunObservationLayout::Compact).unwrap();
    let floor = i64::from(b.0.floor);
    let context = compact.groups.iter().find(|g| g.key == "context").unwrap();
    assert_eq!(context.values[3], floor);
    {
        let _narrow = test_range::narrow(caps.clone(), "context", (0, 0));
        let err = encode(&b, &caps, RunObservationLayout::Compact).unwrap_err();
        assert_eq!(err, RunSessionError::Capacity { group: "context.character", required: context.values[0] as usize, capacity: 0 });
        assert!(encode(&b, &caps, RunObservationLayout::Dense).is_ok(), "dense passes no range");
    }
    {
        // A row group: the first combat card value above 2 (`card_kind`) faults.
        let _narrow = test_range::narrow(caps.clone(), "combat_cards", (-1, 2));
        let err = encode(&b, &caps, RunObservationLayout::Compact).unwrap_err();
        assert!(matches!(err, RunSessionError::Capacity { group: "combat_cards.card_kind", capacity: 2, required } if required > 2), "{err:?}");
    }
    assert_eq!(encode(&b, &caps, RunObservationLayout::Compact).unwrap(), compact, "the override is gone");
    // Out-of-range magnitudes: always required > capacity.
    assert_eq!(value_fault("player", 2, 40_000, (-32_768, 32_767)), RunSessionError::Capacity { group: "player.gold", required: 40_000, capacity: 32_767 });
    assert_eq!(value_fault("player", 3, -32_769, (-32_768, 32_767)), RunSessionError::Capacity { group: "player.block", required: 32_769, capacity: 32_767 });
    let RunSessionError::Capacity { required, capacity, .. } = value_fault("player", 3, i64::MIN, (i64::from(i32::MIN), i64::from(i32::MAX))) else { panic!() };
    assert_eq!((required, capacity), (1usize << 63, i32::MAX as usize));
}

/// The writes outside `row()` (the Standard ascension write and the reference/draw-row `-1` writes) are checked too.
#[test]
fn compact_direct_writes_are_checked() {
    let mut b = combat_boundary(10);
    b.1 = None; // no map: the context row holds only small values (character, act, floor, room, counts)
    let caps = unique_caps(2);
    assert!(b.0.floor < 19 && b.2.ascension == 20);
    {
        let _narrow = test_range::narrow(caps.clone(), "context", (0, 19));
        let err = encode(&b, &caps, RunObservationLayout::Compact).unwrap_err();
        assert_eq!(err, RunSessionError::Capacity { group: "context.ascension", required: 20, capacity: 19 }, "the Standard ascension write");
        let mut a0 = b.clone();
        a0.2.ascension = 0;
        assert!(encode(&a0, &caps, RunObservationLayout::Compact).is_ok(), "only the ascension write is out of range");
    }
    {
        // References of occupied rows start at -1 (`permanent_row`, before any row is written).
        let _narrow = test_range::narrow(caps.clone(), "combat_cards", (0, i64::from(i32::MAX)));
        let err = encode(&b, &caps, RunObservationLayout::Compact).unwrap_err();
        assert_eq!(err, RunSessionError::Capacity { group: "combat_cards.permanent_row", required: 1, capacity: i32::MAX as usize });
    }
}

/// Typed stacked chunks for `spec` (compact: the group dtypes; dense: i64).
struct Chunks { wide: Vec<Wide>, narrow: Vec<Vec<MaybeUninit<i8>>> }
enum Wide { I64(Vec<MaybeUninit<i64>>), I32(Vec<MaybeUninit<i32>>), I16(Vec<MaybeUninit<i16>>) }
impl Chunks {
    fn new(spec: &RunObservationSpec, force_i64: bool) -> Self {
        let d = spec.descriptor();
        let (mut wide, mut narrow) = (Vec::new(), Vec::new());
        for t in d["tensors"].as_array().unwrap() {
            let n = t["shape"].as_array().unwrap().iter().map(|n| n.as_u64().unwrap() as usize).product::<usize>();
            match (t["dtype"].as_str().unwrap(), force_i64) {
                ("int8", _) => narrow.push(vec![MaybeUninit::new(7); n]),
                ("int64", _) | (_, true) => wide.push(Wide::I64(vec![MaybeUninit::new(7); n])),
                ("int32", _) => wide.push(Wide::I32(vec![MaybeUninit::new(7); n])),
                ("int16", _) => wide.push(Wide::I16(vec![MaybeUninit::new(7); n])),
                (other, _) => panic!("{other}"),
            }
        }
        Self { wide, narrow }
    }
    fn target<'a>(&'a mut self, initialized: &'a std::sync::atomic::AtomicBool) -> ObservationTarget<'a> {
        let wide = self.wide.iter_mut().map(|w| match w { Wide::I64(v) => ObservationChunk::I64(v), Wide::I32(v) => ObservationChunk::I32(v),
            Wide::I16(v) => ObservationChunk::I16(v) }).collect();
        ObservationTarget::typed(wide, self.narrow.iter_mut().map(|v| v.as_mut_slice()).collect(), initialized)
    }
    /// The chunks widened to i64 in group order (values, known, occupied), then the mask.
    fn widened(&self) -> (Vec<Vec<i64>>, Vec<Vec<i8>>) {
        // SAFETY (test): every element was initialised to 7 or written by the encoder.
        let wide = self.wide.iter().map(|w| match w {
            Wide::I64(v) => v.iter().map(|x| unsafe { x.assume_init() }).collect(),
            Wide::I32(v) => v.iter().map(|x| i64::from(unsafe { x.assume_init() })).collect(),
            Wide::I16(v) => v.iter().map(|x| i64::from(unsafe { x.assume_init() })).collect() }).collect();
        (wide, self.narrow.iter().map(|v| v.iter().map(|x| unsafe { x.assume_init() }).collect()).collect())
    }
}
fn flatten(o: &RunObservation) -> (Vec<Vec<i64>>, Vec<Vec<i8>>) {
    let wide = o.groups.iter().map(|g| g.values.clone()).collect();
    let mut narrow = Vec::new();
    for g in &o.groups { narrow.push(g.known.clone()); if let Some(occ) = &g.occupied { narrow.push(occ.clone()) } }
    narrow.push(o.action_mask.clone());
    (wide, narrow)
}

/// Encoding into typed int16/int32 chunks equals the owned compact observation; `materialize` widens them back.
#[test]
fn typed_target_equals_owned_compact() {
    let caps = RunCapacities::standard_compact();
    let spec = standard_compact_run_observation_spec(&caps).unwrap();
    let mut seen_combat = false;
    for (i, character) in CHARACTERS.into_iter().enumerate() {
        for b in boundaries(character, 20, 0xC0 + i as u64, 120) {
            seen_combat |= b.0.combat.is_some();
            let owned = encode(&b, &caps, RunObservationLayout::Compact).unwrap();
            let mut chunks = Chunks::new(&spec, false);
            let initialized = std::sync::atomic::AtomicBool::new(false);
            let mut target = chunks.target(&initialized);
            let slab = encode_run_standard_layout_into(&b.0, b.1.as_ref(), &caps, b.2, RunObservationLayout::Compact, Some(&mut target)).unwrap();
            assert!(slab.groups.is_empty(), "written into the target");
            assert_eq!(target.materialize(&owned).unwrap().unwrap(), owned, "materialize widens the typed chunks");
            drop(target);
            assert_eq!(chunks.widened(), flatten(&owned));
        }
    }
    assert!(seen_combat);
    // Chunks of another dtype (i64 for a compact layout) do not fit: the target is untouched and the result is owned.
    let b = combat_boundary(5);
    let mut chunks = Chunks::new(&spec, true);
    let initialized = std::sync::atomic::AtomicBool::new(false);
    let mut target = chunks.target(&initialized);
    let owned = encode_run_standard_layout_into(&b.0, b.1.as_ref(), &caps, b.2, RunObservationLayout::Compact, Some(&mut target)).unwrap();
    assert!(!owned.groups.is_empty() && !initialized.load(std::sync::atomic::Ordering::Acquire));
    // And a dense layout never writes into typed (narrow) chunks.
    let mut chunks = Chunks::new(&spec, false);
    let initialized = std::sync::atomic::AtomicBool::new(false);
    let mut target = chunks.target(&initialized);
    let owned = encode_run_standard_layout_into(&b.0, b.1.as_ref(), &caps, b.2, RunObservationLayout::Dense, Some(&mut target)).unwrap();
    assert!(!owned.groups.is_empty() && !initialized.load(std::sync::atomic::Ordering::Acquire));
}

/// The owned compact observation has the compact shapes; widened and zero-padded it is the dense observation (E1 core).
pub(super) fn widen(compact: &RunObservation, dense_caps: &RunCapacities) -> RunObservation {
    let dense = standard_run_observation_spec(dense_caps).unwrap().descriptor();
    let mut out = RunObservation { groups: Vec::new(), action_mask: vec![0; dense_caps.actions] };
    out.action_mask[..compact.action_mask.len()].copy_from_slice(&compact.action_mask);
    for (g, t) in compact.groups.iter().zip(dense["tensors"].as_array().unwrap().iter().filter(|t| is_value_tensor(t))) {
        assert_eq!(t["key"], g.key);
        let shape = t["shape"].as_array().unwrap().iter().map(|n| n.as_u64().unwrap() as usize).collect::<Vec<_>>();
        let n = shape.iter().product::<usize>();
        let mut values = vec![0; n];
        values[..g.values.len()].copy_from_slice(&g.values);
        let mut known = vec![0; n];
        known[..g.known.len()].copy_from_slice(&g.known);
        let occupied = g.occupied.as_ref().map(|o| { let mut v = vec![0; shape[0]]; v[..o.len()].copy_from_slice(o); v });
        out.groups.push(RunFeatureGroup { key: g.key, shape, values, known, occupied });
    }
    out
}

#[test]
fn compact_boundaries_widen_to_dense() {
    let compact_caps = RunCapacities::standard_compact();
    let dense_caps = RunCapacities::standard_default();
    for (i, character) in CHARACTERS.into_iter().enumerate() {
        for b in boundaries(character, if i % 2 == 0 { 0 } else { 20 }, 0xE1 + i as u64, 300) {
            let compact = encode(&b, &compact_caps, RunObservationLayout::Compact).unwrap();
            assert_eq!(widen(&compact, &dense_caps), encode(&b, &dense_caps, RunObservationLayout::Dense).unwrap());
        }
    }
}

// ---- Task 3: sessions, records, replay, checkpoints and the fault cause ----

use super::m36_heart::{cell_seed, supplied_start};
use super::standard_traversal::{context, needs_supplied_wound, simple_action};
use spire_core::run::diagnostics::supplied_wound_enemies;

pub(super) fn standard_config(character: Character, ascension: u8, caps: RunCapacities, record: bool) -> StandardRunConfig {
    let mut config = StandardRunConfig::new(character, ascension, Partition::Test);
    config.capacities = caps;
    config.record_replay = record;
    config
}
pub(super) fn session(character: Character, ascension: u8, caps: RunCapacities, layout: RunObservationLayout, record: bool) -> RunSession {
    RunSession::new_standard_with_layout(standard_config(character, ascension, caps, record), layout).unwrap()
}
/// A seeded uniform choice among the published legal actions.
pub(super) fn uniform(obs: &RunObservation, seed: u64, step: usize) -> i64 {
    let legal = obs.action_mask.iter().enumerate().filter(|(_, m)| **m == 1).map(|(i, _)| i).collect::<Vec<_>>();
    legal[(mix(seed ^ (step as u64).wrapping_mul(0x100_0000_01B3)) % legal.len() as u64) as usize] as i64
}
fn dense_shapes() -> &'static Vec<Vec<usize>> {
    static S: std::sync::OnceLock<Vec<Vec<usize>>> = std::sync::OnceLock::new();
    S.get_or_init(|| standard_run_observation_spec(&RunCapacities::standard_default()).unwrap().descriptor()["tensors"].as_array().unwrap()
        .iter().filter(|t| is_value_tensor(t)).map(|t| t["shape"].as_array().unwrap().iter().map(|n| n.as_u64().unwrap() as usize).collect()).collect())
}
/// E1: compact widened and zero-padded to the dense capacities equals the dense observation.
pub(super) fn assert_widens(compact: &RunObservation, dense: &RunObservation, at: &str) {
    assert_eq!(compact.groups.len(), dense.groups.len(), "{at}");
    assert_eq!(&dense.action_mask[..compact.action_mask.len()], &compact.action_mask[..], "{at}: action mask");
    assert!(dense.action_mask[compact.action_mask.len()..].iter().all(|m| *m == 0), "{at}: mask padding");
    for ((c, d), shape) in compact.groups.iter().zip(&dense.groups).zip(dense_shapes()) {
        assert_eq!((c.key, &d.shape), (d.key, shape), "{at}");
        let n = c.values.len();
        assert_eq!(&d.values[..n], &c.values[..], "{at}: {} values", c.key);
        assert_eq!(&d.known[..n], &c.known[..], "{at}: {} known", c.key);
        assert!(d.values[n..].iter().all(|v| *v == 0) && d.known[n..].iter().all(|k| *k == 0), "{at}: {} padding", c.key);
        match (&c.occupied, &d.occupied) {
            (Some(co), Some(dd)) => { assert_eq!(&dd[..co.len()], &co[..], "{at}: {} occupied", c.key); assert!(dd[co.len()..].iter().all(|o| *o == 0)) }
            (None, None) => {}
            _ => panic!("{at}: {} occupancy kind", c.key),
        }
    }
}
/// Dense and compact sessions in lockstep; the policy reads the dense boundary. Returns the room kinds seen.
fn lockstep(dense: &mut RunSession, compact: &mut RunSession, cap: usize, supplied: bool, mut policy: impl FnMut(&RunObservation, usize) -> i64, name: &str)
    -> std::collections::BTreeSet<String> {
    let (mut d, mut c) = (dense.decision().unwrap(), compact.decision().unwrap());
    let mut rooms = std::collections::BTreeSet::new();
    for step in 0..cap {
        assert_widens(&c.observation, &d.observation, &format!("{name} step {step}"));
        rooms.insert(context(&d.observation).2);
        if d.terminal.is_some() { break }
        if supplied && needs_supplied_wound(&d.observation) {
            d = dense.apply_supplied(supplied_wound_enemies).unwrap();
            c = compact.apply_supplied(supplied_wound_enemies).unwrap();
            assert_widens(&c.observation, &d.observation, &format!("{name} wound {step}"));
        }
        let index = RunActionIndex(policy(&d.observation, step));
        let (rd, rc) = (dense.step(d.token, index).unwrap(), compact.step(c.token, index).unwrap());
        assert_eq!((rd.reward, rd.terminated, rd.truncated, &rd.reward_events), (rc.reward, rc.terminated, rc.truncated, &rc.reward_events), "{name} {step}");
        (d, c) = (rd.boundary, rc.boundary);
    }
    assert_eq!(d.terminal, c.terminal);
    rooms
}

/// E1 on the 8 M36 supplied Heart routes (every room kind through the Heart and TrueVictory).
#[test]
fn compact_widens_to_dense_on_supplied_routes() {
    let mut rooms = std::collections::BTreeSet::new();
    for (ci, character) in CHARACTERS.into_iter().enumerate() {
        for ascension in [0u8, 20] {
            let seed = cell_seed(ci, ascension);
            let start = supplied_start(character, ascension, seed);
            let mut dense = RunSession::from_standard_supplied(standard_config(character, ascension, RunCapacities::standard_default(), false), start.clone(), seed).unwrap();
            let mut compact = RunSession::from_standard_supplied_layout(standard_config(character, ascension, RunCapacities::standard_compact(), false), start, seed, RunObservationLayout::Compact).unwrap();
            assert_eq!(compact.observation_spec().unwrap(), standard_compact_run_observation_spec(&RunCapacities::standard_compact()).unwrap());
            rooms.extend(lockstep(&mut dense, &mut compact, 12_000, true, |o, _| simple_action(o) as i64, &format!("{character:?} A{ascension}")));
            assert_eq!(dense.decision().unwrap().terminal, Some(spire_core::run::RunTerminal::HeartVictory));
        }
    }
    for room in ["neow", "map", "combat", "rewards", "card_reward", "shop", "campfire", "event", "treasure", "boss_relic_choice", "act3_ending", "heart_victory"] {
        assert!(rooms.contains(room), "room {room} not reached: {rooms:?}");
    }
}

/// E1 on random play (4 characters × A0/A10/A20 × 8 seeds) and the supplied full-length variants (no wound until a
/// fight passes 15 turns). Run by runner 2: `cargo test --release ... m38e_ -- --include-ignored`.
#[test]
#[ignore]
fn compact_widens_to_dense_full_length_and_random() {
    for (ci, character) in CHARACTERS.into_iter().enumerate() {
        for ascension in [0u8, 10, 20] {
            for k in 0..8u64 {
                let seed = 0xE1E1 + 64 * ci as u64 + 8 * u64::from(ascension) + k;
                let mut dense = session(character, ascension, RunCapacities::standard_default(), RunObservationLayout::Dense, false);
                let mut compact = session(character, ascension, RunCapacities::standard_compact(), RunObservationLayout::Compact, false);
                dense.reset(Some(seed)).unwrap();
                compact.reset(Some(seed)).unwrap();
                lockstep(&mut dense, &mut compact, 3_000, false, |o, step| uniform(o, seed, step), &format!("{character:?} A{ascension} s={seed}"));
            }
        }
        let seed = cell_seed(ci, 20);
        let start = supplied_start(character, 20, seed);
        let mut dense = RunSession::from_standard_supplied(standard_config(character, 20, RunCapacities::standard_default(), false), start.clone(), seed).unwrap();
        let mut compact = RunSession::from_standard_supplied_layout(standard_config(character, 20, RunCapacities::standard_compact(), false), start, seed, RunObservationLayout::Compact).unwrap();
        let (mut d, mut c) = (dense.decision().unwrap(), compact.decision().unwrap());
        for step in 0..20_000 {
            assert_widens(&c.observation, &d.observation, &format!("{character:?} full-length {step}"));
            if d.terminal.is_some() { break }
            if needs_supplied_wound(&d.observation) && super::standard_traversal::field(&d.observation, "player", 0, 6) > 15 {
                d = dense.apply_supplied(supplied_wound_enemies).unwrap();
                c = compact.apply_supplied(supplied_wound_enemies).unwrap();
            }
            let index = RunActionIndex(simple_action(&d.observation) as i64);
            d = dense.step(d.token, index).unwrap().boundary;
            c = compact.step(c.token, index).unwrap().boundary;
        }
    }
}

/// The compact session reports the compact spec; `environment_spec` carries the compact ids and preset; dense is unchanged.
#[test]
fn compact_session_spec_and_environment_spec() {
    let compact = session(Character::Silent, 3, RunCapacities::standard_compact(), RunObservationLayout::Compact, false);
    assert_eq!(compact.observation_layout(), RunObservationLayout::Compact);
    let spec = compact.observation_spec().unwrap();
    assert_eq!(hex(spec.fingerprint()), COMPACT_FINGERPRINT);
    let env = compact.environment_spec().unwrap();
    assert_eq!(env["schema_id"], RUN_OBSERVATION_STANDARD_COMPACT_SCHEMA_ID);
    assert_eq!(env["observation_fingerprint"], COMPACT_FINGERPRINT);
    assert_eq!(env["capacity_preset"], STANDARD_COMPACT_CAPACITY_PRESET_ID);
    let custom = session(Character::Silent, 3, RunCapacities { powers: 64, ..RunCapacities::standard_compact() }, RunObservationLayout::Compact, false);
    assert_eq!(custom.environment_spec().unwrap()["capacity_preset"], "custom");
    let dense_at_default = session(Character::Silent, 3, RunCapacities::standard_default(), RunObservationLayout::Compact, false);
    assert_eq!(dense_at_default.environment_spec().unwrap()["capacity_preset"], "custom", "compact at the dense capacities is custom");
    let dense = RunSession::new_standard(standard_config(Character::Silent, 3, RunCapacities::standard_default(), false)).unwrap();
    assert_eq!(dense.observation_layout(), RunObservationLayout::Dense);
    let env = dense.environment_spec().unwrap();
    assert_eq!((env["schema_id"].as_str(), env["observation_fingerprint"].as_str(), env["capacity_preset"].as_str()),
        (Some(RUN_OBSERVATION_STANDARD_SCHEMA_ID), Some(DENSE_FINGERPRINT), Some(crate::STANDARD_CAPACITY_PRESET_ID)));
    let mut keys = env.as_object().unwrap().keys().cloned().collect::<Vec<_>>();
    keys.sort();
    let mut compact_keys = compact.environment_spec().unwrap().as_object().unwrap().keys().cloned().collect::<Vec<_>>();
    compact_keys.sort();
    assert_eq!(keys, compact_keys, "no new environment_spec key");
    // Invalid compact capacities and compact on a captured profile are rejected before anything is built.
    let over = standard_config(Character::Silent, 3, RunCapacities { actions: 4096, ..RunCapacities::standard_compact() }, false);
    assert!(matches!(RunSession::new_standard_with_layout(over, RunObservationLayout::Compact), Err(RunSessionError::Input(_))));
}

/// Plays `steps` uniform steps (or until terminal) and returns the session.
fn played(mut s: RunSession, seed: u64, steps: usize) -> RunSession {
    let mut b = s.reset(Some(seed)).unwrap();
    for step in 0..steps {
        if b.terminal.is_some() { break }
        b = s.step(b.token, RunActionIndex(uniform(&b.observation, seed, step))).unwrap().boundary;
    }
    s
}

#[test]
fn compact_record_replays_and_restores() {
    let caps = RunCapacities::standard_compact();
    let s = played(session(Character::Defect, 7, caps.clone(), RunObservationLayout::Compact, true), 0x7E5, 150);
    let record = s.record().unwrap().unwrap();
    assert_eq!(hex(record.observation_fingerprint), COMPACT_FINGERPRINT, "the record header is the compact fingerprint");
    assert_eq!(record_observation_layout(&record), Ok(RunObservationLayout::Compact));
    let bytes = record.to_json().unwrap();
    let back = RunEpisodeRecord::from_json(&bytes).unwrap();
    assert_eq!(back, record);
    let report = replay_run(&back, caps.clone()).unwrap();
    assert_eq!(report.accepted_count, record.steps.len());
    let replayed = crate::encode_run_standard_decision_with_layout(&report.final_decision, report.final_map.as_ref(), &caps, 7,
        report.final_room_selection, RunObservationLayout::Compact).unwrap();
    assert_eq!(replayed, s.decision().unwrap().observation, "the replay report re-encodes in the record's layout");
    let restored = RunSession::from_record(&back, record.steps.len(), caps.clone()).unwrap();
    assert_eq!(restored.observation_layout(), RunObservationLayout::Compact);
    assert_eq!(hex(restored.observation_spec().unwrap().fingerprint()), COMPACT_FINGERPRINT);
    assert_eq!(hex(restored.record().unwrap().unwrap().observation_fingerprint), COMPACT_FINGERPRINT);
    assert_eq!(restored.decision().unwrap().observation, s.decision().unwrap().observation);
    // A dense record still replays as dense.
    let dense = played(session(Character::Defect, 7, RunCapacities::standard_default(), RunObservationLayout::Dense, true), 0x7E5, 150);
    let dense_record = dense.record().unwrap().unwrap();
    assert_eq!(record_observation_layout(&dense_record), Ok(RunObservationLayout::Dense));
    let dense_restored = RunSession::from_record(&dense_record, dense_record.steps.len(), RunCapacities::standard_default()).unwrap();
    assert_eq!(dense_restored.observation_layout(), RunObservationLayout::Dense);
    // A flipped fingerprint byte, and capacities the compact spec cannot be built at, are IncompatibleReplay.
    let mut flipped = record.clone();
    flipped.observation_fingerprint[0] ^= 1;
    assert!(matches!(replay_run(&flipped, caps.clone()), Err(RunSessionError::IncompatibleReplay(_))));
    assert!(matches!(record_observation_layout(&flipped), Err(RunSessionError::IncompatibleReplay(_))));
    let mut forged = record.clone();
    forged.config.capacities.actions = 4096;
    assert!(matches!(replay_run(&forged, caps.clone()), Err(RunSessionError::IncompatibleReplay(_))), "{:?}", replay_run(&forged, caps.clone()));
    // Caller capacities follow the record layout's rule.
    assert!(matches!(replay_run(&record, RunCapacities { actions: 4096, ..caps.clone() }), Err(RunSessionError::Input(_))));
    assert!(replay_run(&record, RunCapacities::standard_default()).is_ok(), "compact at the dense capacities is valid");
}

/// E7: a compact and a dense record of the same seed and actions differ only in `observation_fingerprint` and capacities.
#[test]
fn compact_and_dense_records_differ_only_in_identity() {
    let c = played(session(Character::Watcher, 12, RunCapacities::standard_compact(), RunObservationLayout::Compact, true), 0xE7, 200);
    let d = played(session(Character::Watcher, 12, RunCapacities::standard_default(), RunObservationLayout::Dense, true), 0xE7, 200);
    let (mut c, d) = (c.record().unwrap().unwrap(), d.record().unwrap().unwrap());
    assert_ne!(c.observation_fingerprint, d.observation_fingerprint);
    c.observation_fingerprint = d.observation_fingerprint;
    c.config.capacities = d.config.capacities.clone();
    assert_eq!(c, d);
}

/// The first step of `seed` that needs more than `cap` rows in `group` (uniform play), with the boundary before it.
fn overflow_session(character: Character, ascension: u8, caps: RunCapacities, record: bool, seed: u64) -> (RunSession, RunBoundary, RunActionIndex, usize) {
    let mut s = session(character, ascension, caps, RunObservationLayout::Compact, record);
    let mut b = s.reset(Some(seed)).unwrap();
    for step in 0..5_000 {
        assert!(b.terminal.is_none(), "no overflow before terminal");
        let index = RunActionIndex(uniform(&b.observation, seed, step));
        let mut probe = RunSession::exact_fork(&s.exact_checkpoint().unwrap()).unwrap();
        let token = probe.decision().unwrap().token;
        if probe.step(token, index).is_err() { return (s, b, index, step) }
        b = s.step(b.token, index).unwrap().boundary;
    }
    panic!("no overflow");
}

/// E4 step overflow: the step commits, then the post-step encode overflows (`permanent_cards = 11`: Ironclad A10 starts
/// with 11 cards, so the first card added overflows).
#[test]
fn compact_step_overflow_surface() {
    let caps = RunCapacities { permanent_cards: 11, ..RunCapacities::standard_compact() };
    for record in [true, false] {
        let (mut s, b, index, _) = overflow_session(Character::Ironclad, 10, caps.clone(), record, 0x0F);
        let err = s.step(b.token, index).unwrap_err();
        assert_eq!(err, RunSessionError::DeliveryFault("post-step Run encoding failed"));
        assert_eq!(s.decision().unwrap_err(), err, "retained");
        assert_eq!(s.fault_cause(), Some(RunSessionError::Capacity { group: "permanent_cards", required: 12, capacity: 11 }), "record_replay={record}");
        if !record { continue }
        let rec = s.record().unwrap().unwrap();
        let fault = rec.fault.as_ref().unwrap();
        assert!(fault.committed && fault.gameplay_recorded);
        assert_eq!(fault.error.kind, crate::RunRecordErrorKind::Capacity);
        assert_eq!(fault.error.capacity.as_ref().map(|c| (c.group.as_str(), c.required, c.capacity)), Some(("permanent_cards", 12, 11)));
        let rec = RunEpisodeRecord::from_json(&rec.to_json().unwrap()).unwrap();
        assert!(replay_run(&rec, caps.clone()).is_ok(), "replay reproduces the recorded capacity fault");
        let err = RunSession::from_record(&rec, rec.steps.len(), caps.clone()).err();
        assert_eq!(err, Some(RunSessionError::DeliveryFault("post-step Run encoding failed")), "from_record reproduces it");
        // Under larger capacities the prefix continues.
        let larger = RunSession::from_record(&rec, rec.steps.len(), RunCapacities::standard_compact()).unwrap();
        assert!(larger.decision().is_ok() && larger.fault_cause().is_none());
    }
}

/// E4 reset overflow: `permanent_cards = 11`: Silent's 12-card start overflows at reset (the error is `Capacity` itself);
/// Ironclad A0 (10 cards) and A10 (11 with Ascender's Bane) start.
#[test]
fn compact_reset_overflow_surface() {
    let caps = RunCapacities { permanent_cards: 11, ..RunCapacities::standard_compact() };
    let mut silent = session(Character::Silent, 0, caps.clone(), RunObservationLayout::Compact, true);
    let err = silent.reset(Some(1)).unwrap_err();
    assert_eq!(err, RunSessionError::Capacity { group: "permanent_cards", required: 12, capacity: 11 });
    assert_eq!(silent.fault_cause(), Some(err.clone()));
    let fault = silent.record().unwrap().unwrap().fault.unwrap();
    assert!(!fault.committed && fault.error.kind == crate::RunRecordErrorKind::Capacity);
    for ascension in [0, 10] { assert!(session(Character::Ironclad, ascension, caps.clone(), RunObservationLayout::Compact, false).reset(Some(1)).is_ok()) }
    // Dense at the same capacities faults identically (row overflows do not depend on the layout).
    let mut dense = session(Character::Silent, 0, caps, RunObservationLayout::Dense, false);
    assert_eq!(dense.reset(Some(1)).unwrap_err(), err);
    assert_eq!(dense.fault_cause(), Some(err), "fault_cause is kept for every layout");
}

/// Review I1: value faults ("<group>.<column>") export, import, replay and restore for compact records; they recur at
/// any capacities; a dense record with such a group is still rejected.
#[test]
fn compact_value_fault_record_round_trip() {
    // The hook adds 2^31 to one column (values from `from` on): a real out-of-int32 value, as an unclamped counter would be.
    for (tag, group, column, from, name) in [(3usize, "context", 3usize, 1i64, "context.floor"), (4, "combat_cards", 2, 0, "combat_cards.card_kind")] {
        let caps = unique_caps(tag);
        let _inflate = test_range::inflate(caps.clone(), group, column, 1 << 31, (from, i64::MAX));
        let mut s = session(Character::Ironclad, 0, caps.clone(), RunObservationLayout::Compact, true);
        let mut b = s.reset(Some(0xF00D)).unwrap();
        let err = {
            let mut step = 0;
            loop {
                match s.step(b.token, RunActionIndex(uniform(&b.observation, 0xF00D, step))) {
                    Ok(r) => b = r.boundary,
                    Err(e) => break e,
                }
                step += 1;
                assert!(step < 2_000 && b.terminal.is_none(), "{group}: no value fault");
            }
        };
        assert_eq!(err, RunSessionError::DeliveryFault("post-step Run encoding failed"));
        let Some(RunSessionError::Capacity { group: got, required, capacity }) = s.fault_cause() else { panic!("{:?}", s.fault_cause()) };
        assert_eq!((got, capacity), (name, i32::MAX as usize));
        assert!(required > capacity);
        let rec = s.record().unwrap().unwrap();
        let bytes = rec.to_json().unwrap();
        let back = RunEpisodeRecord::from_json(&bytes).unwrap();
        assert_eq!(back, rec);
        assert!(replay_run(&back, caps.clone()).is_ok(), "{group}: replay reproduces the value fault");
        assert_eq!(RunSession::from_record(&back, back.steps.len(), caps.clone()).err(), Some(err.clone()));
        assert_eq!(RunSession::from_record(&back, back.steps.len() - 1, caps.clone()).unwrap().fault_cause(), None, "the prefix before it is healthy");
        // A dense record carrying a value-fault group is rejected by the strict codec.
        let dense = played(session(Character::Ironclad, 0, RunCapacities::standard_default(), RunObservationLayout::Dense, true), 0xF00D, 3);
        let mut forged: Value = serde_json::from_slice(&dense.record().unwrap().unwrap().to_json().unwrap()).unwrap();
        let mut fault: Value = serde_json::from_slice(&bytes).unwrap();
        forged["steps"] = fault["steps"].take();
        forged["terminal"] = fault["terminal"].take();
        forged["fault"] = fault["fault"].take();
        assert!(matches!(RunEpisodeRecord::from_json(&serde_json::to_vec(&forged).unwrap()), Err(RunSessionError::IncompatibleReplay(_))));
        // A compact record whose value-fault capacity is not the dtype maximum, or whose required value fits, is rejected.
        let mut wrong: Value = serde_json::from_slice(&bytes).unwrap();
        wrong["fault"]["error"]["capacity"]["capacity"] = json!(12345);
        assert!(matches!(RunEpisodeRecord::from_json(&serde_json::to_vec(&wrong).unwrap()), Err(RunSessionError::IncompatibleReplay(_))));
        let mut fits: Value = serde_json::from_slice(&bytes).unwrap();
        fits["fault"]["error"]["capacity"]["required"] = json!(i32::MAX);
        assert!(matches!(RunEpisodeRecord::from_json(&serde_json::to_vec(&fits).unwrap()), Err(RunSessionError::IncompatibleReplay(_))));
    }
}

/// The value fault recurs under the larger dense-preset capacities (the escape hatch does not apply to value faults).
#[test]
fn compact_value_fault_recurs_at_any_capacities() {
    let caps = unique_caps(5);
    let mut s = session(Character::Ironclad, 20, caps.clone(), RunObservationLayout::Compact, true);
    let _inflate = test_range::inflate(caps.clone(), "context", 1, 1 << 31, (0, i64::MAX));
    let err = s.reset(Some(5)).unwrap_err();
    assert_eq!(err, RunSessionError::Capacity { group: "context.ascension", required: (1 << 31) + 20, capacity: i32::MAX as usize });
    let rec = RunEpisodeRecord::from_json(&s.record().unwrap().unwrap().to_json().unwrap()).unwrap();
    assert!(!rec.fault.as_ref().unwrap().committed);
    assert!(replay_run(&rec, caps.clone()).is_ok());
    // Larger capacities (the dense preset's rows; a unique key so no parallel test sees the hook).
    let larger = RunCapacities { orbs: 4, ..RunCapacities::standard_default() };
    let _wide = test_range::inflate(larger.clone(), "context", 1, 1 << 31, (0, i64::MAX));
    assert!(replay_run(&rec, larger.clone()).is_ok(), "the value fault recurs at larger capacities");
    assert_eq!(RunSession::from_record(&rec, 0, larger).err(), Some(err));
}

/// E8: for each costly group at a real boundary with n occupied rows: that group at n encodes (and widens to dense); at
/// n - 1 it faults with {group, required: n, capacity: n - 1}. known_draw and combat_card_offers are sized by
/// combat_cards and offers (N4), so their own capacities are checked through the coupled group.
#[test]
fn compact_capacity_edges() {
    let set: [(&str, fn(&mut RunCapacities, usize)); 6] = [("permanent_cards", |c, v| c.permanent_cards = v), ("combat_cards", |c, v| c.combat_cards = v),
        ("powers", |c, v| c.powers = v), ("offers", |c, v| c.offers = v), ("selection", |c, v| c.selection = v), ("actions", |c, v| c.actions = v)];
    let dense_caps = RunCapacities::standard_default();
    let mut found = std::collections::BTreeMap::new();
    'search: for (i, character) in CHARACTERS.into_iter().enumerate() {
        for b in boundaries(character, 20, 0xE8 + i as u64, 600) {
            let dense = encode(&b, &dense_caps, RunObservationLayout::Dense).unwrap();
            for (group, _) in set {
                let n = dense.groups.iter().find(|g| g.key == group).unwrap().occupied.as_ref().unwrap().iter().filter(|o| **o == 1).count();
                if n >= 2 && !found.contains_key(group) { found.insert(group, b.clone()); }
            }
            if found.len() == set.len() { break 'search }
        }
    }
    for (group, edit) in set {
        let b = found.get(group).unwrap_or_else(|| panic!("no boundary with {group} rows"));
        let dense = encode(b, &dense_caps, RunObservationLayout::Dense).unwrap();
        let n = dense.groups.iter().find(|g| g.key == group).unwrap().occupied.as_ref().unwrap().iter().filter(|o| **o == 1).count();
        let mut caps = RunCapacities::standard_compact();
        edit(&mut caps, n);
        let compact = encode(b, &caps, RunObservationLayout::Compact).unwrap_or_else(|e| panic!("{group} at {n}: {e:?}"));
        assert_widens(&compact, &dense, group);
        edit(&mut caps, n - 1);
        assert_eq!(encode(b, &caps, RunObservationLayout::Compact).unwrap_err(), RunSessionError::Capacity { group, required: n, capacity: n - 1 }, "{group}");
    }
    // N4: known_draw rows are bounded by combat_cards, and combat_cards is checked first.
    let b = combat_boundary(10);
    let rows = encode(&b, &dense_caps, RunObservationLayout::Dense).unwrap().groups.iter().find(|g| g.key == "combat_cards").unwrap()
        .occupied.as_ref().unwrap().iter().filter(|o| **o == 1).count();
    let caps = RunCapacities { combat_cards: rows - 1, ..RunCapacities::standard_compact() };
    assert!(matches!(encode(&b, &caps, RunObservationLayout::Compact), Err(RunSessionError::Capacity { group: "combat_cards", .. })));
}

/// Review I2: the layout survives `fresh_like`, `exact_fork` and `exact_restore`; restoring a dense checkpoint into a
/// compact session with equal capacities is `Input` and changes nothing.
#[test]
fn compact_layout_carried() {
    let caps = RunCapacities::standard_compact();
    let compact = played(session(Character::Silent, 5, caps.clone(), RunObservationLayout::Compact, true), 0x12, 40);
    let fresh = compact.fresh_like().unwrap();
    assert_eq!((fresh.observation_layout(), hex(fresh.observation_spec().unwrap().fingerprint())), (RunObservationLayout::Compact, COMPACT_FINGERPRINT.to_owned()));
    let checkpoint = compact.exact_checkpoint().unwrap();
    let fork = RunSession::exact_fork(&checkpoint).unwrap();
    assert_eq!(fork.observation_layout(), RunObservationLayout::Compact);
    assert_eq!(hex(fork.record().unwrap().unwrap().observation_fingerprint), COMPACT_FINGERPRINT);
    assert_eq!(fork.decision().unwrap().observation, compact.decision().unwrap().observation);
    let mut fork = fork;
    let mut fresh_record = played(session(Character::Silent, 5, caps.clone(), RunObservationLayout::Compact, true), 0x99, 3);
    fresh_record.exact_restore(&checkpoint).unwrap();
    assert_eq!(fresh_record.decision().unwrap().observation, compact.decision().unwrap().observation);
    // A dense checkpoint at equal capacities into a compact session: Input, nothing changed.
    let dense = played(session(Character::Silent, 5, caps.clone(), RunObservationLayout::Dense, true), 0x12, 40);
    let before = fork.decision().unwrap();
    let before_record = fork.record().unwrap();
    assert!(matches!(fork.exact_restore(&dense.exact_checkpoint().unwrap()), Err(RunSessionError::Input(_))));
    let after = fork.decision().unwrap();
    assert_eq!((after.token, &after.observation), (before.token, &before.observation));
    assert_eq!(fork.record().unwrap(), before_record);
    assert!(fork.fault_cause().is_none() && fork.step(after.token, RunActionIndex(uniform(&after.observation, 1, 0))).is_ok());
}

/// E9 (review M4): a compact session's combat view equals a dense session's at the same capacities.
#[test]
fn compact_combat_view_equals_dense_at_same_caps() {
    let caps = RunCapacities::standard_compact();
    let mut c = session(Character::Defect, 15, caps.clone(), RunObservationLayout::Compact, false);
    let mut d = session(Character::Defect, 15, caps.clone(), RunObservationLayout::Dense, false);
    let (mut bc, mut bd) = (c.reset(Some(0xE9)).unwrap(), d.reset(Some(0xE9)).unwrap());
    let mut views = 0;
    for step in 0..400 {
        if bc.terminal.is_some() { break }
        if context(&bd.observation).2 == "combat" {
            let (vc, vd) = (c.combat_view().unwrap(), d.combat_view().unwrap());
            assert_eq!((vc.observation, vc.info), (vd.observation, vd.info));
            views += 1;
        }
        let index = RunActionIndex(uniform(&bd.observation, 0xE9, step));
        bc = c.step(bc.token, index).unwrap().boundary;
        bd = d.step(bd.token, index).unwrap().boundary;
    }
    assert!(views > 10);
    assert_eq!(c.combat_observation_spec().unwrap(), d.combat_observation_spec().unwrap());
}

/// `observation="compact"` exists only for the Standard profile.
#[test]
fn layout_rejected_on_captured_profiles() {
    let config = RunSessionConfig { character: Character::Ironclad, ascension: 0, partition: Partition::Test, boss_history: crate::M9BossHistory::AwakenedUnseen,
        capacities: RunCapacities::m9_default(), record_replay: false };
    assert!(matches!(RunSession::with_profile_layout(config, RunProfile::M9IroncladAct3, RunObservationLayout::Compact), Err(RunSessionError::Input(_))));
}

// ---- Task 4: native batches ----

use crate::{AutoresetMode, BatchConfig, BatchFaultStage, FaultPolicy, RunSessionBatch, SessionBatchError, SlotDelivery, SlotRow};

fn batch(configs: Vec<StandardRunConfig>, layout: RunObservationLayout, workers: usize, autoreset: AutoresetMode, fault_policy: FaultPolicy) -> RunSessionBatch {
    RunSessionBatch::new_run(configs, |c| RunSession::new_standard_with_layout(c, layout), BatchConfig { workers, autoreset, fault_policy }).unwrap()
}
fn eight(caps: &RunCapacities, record: bool) -> Vec<StandardRunConfig> {
    (0..8).map(|slot| standard_config(CHARACTERS[slot % 4], if slot < 4 { 0 } else { 20 }, caps.clone(), record)).collect()
}
fn picks(rows: &[SlotRow<RunBoundary>], step: usize) -> Vec<RunActionIndex> {
    rows.iter().enumerate().map(|(slot, r)| {
        let legal = r.boundary.observation.action_mask.iter().enumerate().filter(|(_, m)| **m == 1).map(|(i, _)| i).collect::<Vec<_>>();
        RunActionIndex(if legal.is_empty() { 0 } else { legal[(mix((slot as u64) << 32 ^ step as u64) % legal.len() as u64) as usize] as i64 })
    }).collect()
}
/// Rows with the per-batch session id (token word 0) cleared; serials and everything else compare.
fn untokened(rows: &[SlotRow<RunBoundary>]) -> Vec<SlotRow<RunBoundary>> {
    let clear = |b: &RunBoundary| RunBoundary { token: RunDecisionToken([0, <[u64; 2]>::from(b.token)[1]]), ..b.clone() };
    rows.iter().map(|r| SlotRow { boundary: clear(&r.boundary), final_boundary: r.final_boundary.as_ref().map(clear), ..r.clone() }).collect()
}
/// A compact row widened equals the dense row (boundary, final boundary, rewards, flags and fault rows).
fn assert_row_widens(c: &SlotRow<RunBoundary>, d: &SlotRow<RunBoundary>, at: &str) {
    assert_widens(&c.boundary.observation, &d.boundary.observation, at);
    assert_eq!((c.boundary.terminal, c.reward, &c.reward_events, c.terminated, c.truncated, c.autoreset, &c.fault),
        (d.boundary.terminal, d.reward, &d.reward_events, d.terminated, d.truncated, d.autoreset, &d.fault), "{at}");
    match (&c.final_boundary, &d.final_boundary) {
        (Some(cf), Some(df)) => { assert_widens(&cf.observation, &df.observation, at); assert_eq!(cf.terminal, df.terminal) }
        (None, None) => {}
        _ => panic!("{at}: final boundary presence"),
    }
}

/// Compact batches (workers 1 and 4, NEXT_STEP and SAME_STEP, 200 steps) widen to the dense batch row for row. The dense
/// batch equals scalar sessions (M35) and compact scalar widens to dense scalar (E1); `widen` is injective, so each
/// compact row equals its scalar compact session.
#[test]
fn compact_batch_equals_scalar() {
    for autoreset in [AutoresetMode::NextStep, AutoresetMode::SameStep] {
        let mut dense = batch(eight(&RunCapacities::standard_default(), false), RunObservationLayout::Dense, 1, autoreset, FaultPolicy::Isolate);
        let mut compacts = [1, 4].map(|w| batch(eight(&RunCapacities::standard_compact(), false), RunObservationLayout::Compact, w, autoreset, FaultPolicy::Isolate));
        let seeds = (0..8u64).map(|s| Some(0xBA7C + s)).collect::<Vec<_>>();
        let mut d = dense.reset(&seeds, &vec![None; 8], None).unwrap();
        let mut cs = compacts.each_mut().map(|b| b.reset(&seeds, &vec![None; 8], None).unwrap());
        let (mut terminals, mut resets) = (0, 0);
        for step in 0..200 {
            for (w, c) in cs.iter().enumerate() { for (slot, (cr, dr)) in c.iter().zip(&d).enumerate() { assert_row_widens(cr, dr, &format!("{autoreset:?} w{w} step {step} slot {slot}")) } }
            let indices = picks(&d, step);
            let tokens = |rows: &[SlotRow<RunBoundary>]| rows.iter().map(|r| r.boundary.token).collect::<Vec<_>>();
            d = dense.step(&tokens(&d), &indices).unwrap();
            for (b, c) in compacts.iter_mut().zip(cs.iter_mut()) { *c = b.step(&tokens(c), &indices).unwrap() }
            terminals += d.iter().filter(|r| r.terminated).count();
            resets += d.iter().filter(|r| r.autoreset || r.final_boundary.is_some()).count();
        }
        assert!(terminals > 0 && resets > 0, "{autoreset:?}: autoresets exercised ({terminals}, {resets})");
        // Masked reset rows too.
        let mask = (0..8).map(|s| s % 3 == 0).collect::<Vec<_>>();
        let seeds = (0..8u64).map(|s| Some(0x3A5C + s)).collect::<Vec<_>>();
        let d = dense.reset(&seeds, &vec![None; 8], Some(&mask)).unwrap();
        for b in compacts.iter_mut() {
            for (slot, (cr, dr)) in b.reset(&seeds, &vec![None; 8], Some(&mask)).unwrap().iter().zip(&d).enumerate() { assert_row_widens(cr, dr, &format!("masked reset slot {slot}")) }
        }
    }
}

/// Typed (int16/int32) slab targets through `step_into` equal the owned rows of an identical batch, including SAME_STEP
/// `final_boundary` materialisation and autoreset rows written into the targets.
#[test]
fn compact_batch_step_into_typed() {
    let caps = RunCapacities::standard_compact();
    let spec = standard_compact_run_observation_spec(&caps).unwrap();
    for (workers, autoreset) in [(1, AutoresetMode::SameStep), (4, AutoresetMode::SameStep), (4, AutoresetMode::NextStep)] {
        let mut rows_batch = batch(eight(&caps, false), RunObservationLayout::Compact, workers, autoreset, FaultPolicy::Isolate);
        let mut slab_batch = batch(eight(&caps, false), RunObservationLayout::Compact, workers, autoreset, FaultPolicy::Isolate);
        let seeds = (0..8u64).map(|s| Some(0x51AB + s)).collect::<Vec<_>>();
        let mut r = rows_batch.reset(&seeds, &vec![None; 8], None).unwrap();
        let mut sb = slab_batch.reset(&seeds, &vec![None; 8], None).unwrap();
        assert_eq!(untokened(&r), untokened(&sb));
        let (mut written, mut finals) = (0, 0);
        for step in 0..200 {
            let indices = picks(&r, step);
            r = rows_batch.step(&r.iter().map(|x| x.boundary.token).collect::<Vec<_>>(), &indices).unwrap();
            let mut chunks = (0..8).map(|_| Chunks::new(&spec, false)).collect::<Vec<_>>();
            let flags = (0..8).map(|_| std::sync::atomic::AtomicBool::new(false)).collect::<Vec<_>>();
            let targets = chunks.iter_mut().zip(&flags).map(|(c, f)| c.target(f)).collect::<Vec<_>>();
            let mut next = slab_batch.step_into(&sb.iter().map(|x| x.boundary.token).collect::<Vec<_>>(), &indices, targets).unwrap();
            for (slot, row) in next.iter_mut().enumerate() {
                if row.boundary.observation.groups.is_empty() {
                    written += 1;
                    let (wide, narrow) = chunks[slot].widened();
                    assert_eq!((wide, narrow), flatten(&r[slot].boundary.observation), "w{workers} {autoreset:?} step {step} slot {slot}");
                    row.boundary.observation = r[slot].boundary.observation.clone();
                }
                finals += usize::from(row.final_boundary.is_some());
            }
            assert_eq!(untokened(&next), untokened(&r), "w{workers} {autoreset:?} step {step}");
            sb = next;
        }
        assert!(written > 1_000 && (autoreset == AutoresetMode::NextStep || finals > 0), "{written} {finals}");
    }
}

/// `permanent_cards = 11` (deterministic): Silent overflows at reset, Ironclad A10 on its first added card.
fn eleven() -> RunCapacities { RunCapacities { permanent_cards: 11, ..RunCapacities::standard_compact() } }

#[test]
fn compact_batch_overflow_policies() {
    // Freeze, reset: stage `reset` for the Silent slot; the native report carries the Capacity error.
    let configs = vec![standard_config(Character::Ironclad, 10, eleven(), true), standard_config(Character::Silent, 0, eleven(), true)];
    let mut frozen = batch(configs.clone(), RunObservationLayout::Compact, 2, AutoresetMode::NextStep, FaultPolicy::Freeze);
    let Err(SessionBatchError::Fault(fault)) = frozen.reset(&[Some(1), Some(1)], &[None, None], None) else { panic!("reset must fault") };
    assert!(matches!(fault.slots[1].delivery, SlotDelivery::Faulted { stage: BatchFaultStage::Reset, committed: false, .. }), "{:?}", fault.slots[1]);
    assert_eq!(fault.slots[1].error, Some(RunSessionError::Capacity { group: "permanent_cards", required: 12, capacity: 11 }));
    assert_eq!(frozen.session(1).unwrap().fault_cause(), Some(RunSessionError::Capacity { group: "permanent_cards", required: 12, capacity: 11 }));
    // Isolate: the Silent slot is a zero row with its fault; the Ironclad A10 slot later faults at `encode`.
    let mut iso = batch(configs, RunObservationLayout::Compact, 2, AutoresetMode::NextStep, FaultPolicy::Isolate);
    let mut rows = iso.reset(&[Some(1), Some(1)], &[None, None], None).unwrap();
    let zero = iso.zero_boundary().unwrap().observation.clone();
    assert_eq!(rows[1].fault.as_ref().map(|f| (f.stage, f.message.as_str(), f.committed)), Some((BatchFaultStage::Reset, "Run capacity exceeded", false)));
    assert_eq!(rows[1].boundary.observation, zero);
    assert!(zero.groups.iter().all(|g| g.values.iter().all(|v| *v == 0)) && zero.action_mask.iter().all(|m| *m == 0));
    let mut encode_fault = None;
    for step in 0..400 {
        let tokens = rows.iter().map(|r| r.boundary.token).collect::<Vec<_>>();
        rows = iso.step(&tokens, &picks(&rows, step)).unwrap();
        if let Some(f) = &rows[0].fault { encode_fault = Some(f.clone()); break }
    }
    let f = encode_fault.expect("Ironclad A10 adds a card");
    assert_eq!((f.stage, f.message.as_str(), f.committed), (BatchFaultStage::Encode, "post-step Run encoding failed", true));
    assert_eq!(iso.session(0).unwrap().fault_cause(), Some(RunSessionError::Capacity { group: "permanent_cards", required: 12, capacity: 11 }));
    // The explicit seeded masked reset resumes slot 0 with a fresh session of the same (compact) layout.
    let resumed = iso.reset(&[Some(9), None], &[None, None], Some(&[true, false])).unwrap();
    assert!(resumed[0].fault.is_none());
    let session = iso.session(0).unwrap();
    assert_eq!(session.observation_layout(), RunObservationLayout::Compact);
    let compact_fp = standard_compact_run_observation_spec(&eleven()).unwrap().fingerprint();
    assert_eq!(session.observation_spec().unwrap().fingerprint(), compact_fp);
    assert_eq!(iso.record(0).unwrap().unwrap().observation_fingerprint, compact_fp);
    // Freeze, step: stage `encode` with the native "post-step Run encoding failed" report.
    let mut frozen = batch(vec![standard_config(Character::Ironclad, 10, eleven(), false); 2], RunObservationLayout::Compact, 2, AutoresetMode::NextStep, FaultPolicy::Freeze);
    let mut rows = frozen.reset(&[Some(1), Some(2)], &[None, None], None).unwrap();
    let fault = (0..400).find_map(|step| {
        let tokens = rows.iter().map(|r| r.boundary.token).collect::<Vec<_>>();
        match frozen.step(&tokens, &picks(&rows, step)) { Ok(next) => { rows = next; None } Err(SessionBatchError::Fault(f)) => Some(f), Err(e) => panic!("{e:?}") }
    }).expect("an Ironclad A10 slot adds a card");
    let faulted = fault.slots.iter().find(|s| matches!(s.delivery, SlotDelivery::Faulted { .. })).unwrap();
    assert!(matches!(faulted.delivery, SlotDelivery::Faulted { stage: BatchFaultStage::Encode, committed: true, .. }), "{faulted:?}");
    assert_eq!(faulted.error, Some(RunSessionError::DeliveryFault("post-step Run encoding failed")));
}

/// Review I5a / N5: the reset half of an autoreset overflows (a value fault on the new episode's floor-0 context, armed
/// process-wide after the first reset so the worker thread sees it): stage `autoreset`; SAME_STEP keeps the committed
/// terminal (`recover_committed`) and materialises it from typed targets.
#[test]
fn compact_batch_autoreset_overflow() {
    for (tag, autoreset) in [(6usize, AutoresetMode::NextStep), (7, AutoresetMode::SameStep)] {
        let caps = unique_caps(tag);
        let spec = standard_compact_run_observation_spec(&caps).unwrap();
        // Find the terminal step of slot 0 with the hook off.
        let run = |arm_at: Option<usize>| {
            let mut b = batch(vec![standard_config(Character::Ironclad, 20, caps.clone(), false)], RunObservationLayout::Compact, 4, autoreset, FaultPolicy::Freeze);
            let mut rows = b.reset(&[Some(0xA7)], &[None], None).unwrap();
            let mut guard = None;
            for step in 0..5_000 {
                if Some(step) == arm_at { guard = Some(test_range::inflate(caps.clone(), "context", 3, 1 << 31, (0, 0))) }
                let mut chunks = Chunks::new(&spec, false);
                let flag = std::sync::atomic::AtomicBool::new(false);
                let result = b.step_into(&[rows[0].boundary.token], &picks(&rows, step), vec![chunks.target(&flag)]);
                match result {
                    Ok(next) => { if next[0].terminated && arm_at.is_none() { return (step, None, b) } rows = next }
                    Err(SessionBatchError::Fault(f)) => { drop(guard); return (step, Some(f), b) }
                    Err(e) => panic!("{e:?}"),
                }
            }
            panic!("no terminal");
        };
        let (terminal_step, none, _) = run(None);
        assert!(none.is_none());
        // NEXT_STEP: the reset runs on the call after the terminal step; SAME_STEP: in the terminal step itself.
        let arm = if autoreset == AutoresetMode::NextStep { terminal_step + 1 } else { terminal_step };
        let (step, fault, b) = run(Some(arm));
        assert_eq!(step, arm, "{autoreset:?}");
        let fault = fault.unwrap();
        let committed = autoreset == AutoresetMode::SameStep;
        assert!(matches!(fault.slots[0].delivery, SlotDelivery::Faulted { stage: BatchFaultStage::Autoreset, .. }), "{:?}", fault.slots[0]);
        assert_eq!(fault.slots[0].error, Some(RunSessionError::Capacity { group: "context.floor", required: 1 << 31, capacity: i32::MAX as usize }));
        if committed {
            assert!(matches!(fault.slots[0].delivery, SlotDelivery::Faulted { committed: true, .. }));
            let recovered = b.recover_committed().unwrap()[0].clone().expect("the committed terminal is recoverable");
            assert!(recovered.terminated);
            let terminal = recovered.final_boundary.as_ref().unwrap_or(&recovered.boundary);
            assert!(terminal.terminal.is_some() && !terminal.observation.groups.is_empty(), "materialised from the typed target");
        }
    }
}

/// Dense and compact sessions never share a batch (the existing one-descriptor rule).
#[test]
fn dense_and_compact_slots_do_not_mix() {
    let configs = [RunObservationLayout::Dense, RunObservationLayout::Compact].map(|l| (standard_config(Character::Ironclad, 0, RunCapacities::standard_compact(), false), l)).to_vec();
    let err = RunSessionBatch::new_run(configs, |(c, l)| RunSession::new_standard_with_layout(c, l),
        BatchConfig { workers: 1, autoreset: AutoresetMode::Disabled, fault_policy: FaultPolicy::Freeze }).err();
    assert_eq!(err, Some(SessionBatchError::Input { slot: Some(1), error: RunSessionError::Input("batch slots must share one observation descriptor") }));
}
