//! M28 traversal (design §14, plan Task X1, PM ruling Q3): for each character, `--seeds` seeded
//! `Standard` Runs (full Neow on odd seeds, short on even) driven by a seeded uniform random
//! legal policy to a terminal. A fault is reaching the action cap, a rejected advertised action
//! (any `Err`, including `Unsupported`), a non-terminal decision with no candidates, a room outside
//! the standard scenario, a replay mismatch or a panic. Deaths are policy outcomes and never gate.
//! Writes `traversal.json` (per-run results, census, faults) to `--out`; exits 1 on any fault.
//!
//! Two policies (both legal-only, neither a strategy claim): `uniform`, the design's random legal
//! policy, and `reach`, which values combat candidates by a greedy turn play-out so Runs reach later acts
//! for the census. Every advertised candidate is also stepped on a clone each decision.
//!
//! With `--features m28-diagnostics` a third mode, `supplied`, runs the reach policy on a Run with
//! supplied max HP and keys so it plays through every act (design §14 grade Supplied).
//!
//! M29 (plan M29a1 item 9, erratum v3 n4): `--ascension N` or `--ascensions A-B` runs each level
//! (A1–A5 are public since the M29 close, A6–A10 since M30 and A11–A15 since M31; the supplied mode above A0 needs `--features m29-diagnostics`). An advertised candidate rejected with a
//! typed interim (an encounter whose enemy facts have not landed at this ascension) is counted and
//! left out of the policy's choice set; a decision whose every candidate is an interim rejection
//! ends the Run as `interim_blocked`, which is not a fault. Each Run also records interim counts by
//! public room context, its per-act elite target/placement and its HP around each act transition. One ascension writes
//! `DIR/traversal.json`; a range writes `DIR/a<N>/traversal.json` per level.
//!
//! M30 (plan item 10): each Run also records its start HP and max HP, the Ascender's Bane count and
//! first index at creation and at the end, and a census of Toke rows (offered, disabled, cancelled),
//! affordable shop purge rows disabled for want of a purgeable card (G2) and Bane copies, under the separate `m30` key (outside the A0 control
//! projection, as the `m29` key is).
//!
//! M31 (plan item 8, review n1): each Run also records, under the separate `m31` key, its A0-control
//! attribution markers (the first step of a G1 equip/hidden card row, of a G3 shop, of a Potion Belt
//! loss and of a potion held in a kept Belt slot), its potion slots at creation and at the end, its
//! Belt gains and losses, its offered card rows by act (combat or equip, with and without upgrades),
//! its boss gold rows, its event option hits and Note For Yourself sightings. Per-step decision
//! digests (sha256 of step, public decision and chosen action, 48 bits) go to `digests.jsonl` beside
//! `traversal.json`, using only the public API, so the same example built on a base tree gives the
//! first differing step of each Run.
//!
//! Usage: cargo run --release -p spire-core --example m28_traversal [--features m28-diagnostics|m29-diagnostics] -- --out DIR
//!        [--seeds 25] [--cap 20000] [--policy uniform|reach|supplied] [--ascension N | --ascensions A-B]
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use spire_core::{
    combat::rng::Partition,
    content::{CardId, Character, EventId, RelicId},
    run::{map::room_quotas, CommittedEvent, CreationProfile, GameAction, MapNodeKind, PublicDecision, RoomChoiceKind, RoomChoiceStatus, RoomScenario, RoomViewKind, Run, RunConfig, RunError, RunRng},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    env, fs,
    panic::{catch_unwind, AssertUnwindSafe},
    path::PathBuf,
    process::ExitCode,
};

const CHARACTERS: [Character; 4] = [Character::Ironclad, Character::Silent, Character::Defect, Character::Watcher];

fn splitmix(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

fn root(character: Character, seed: u64) -> [u8; 32] {
    Sha256::digest(format!("m28-traversal/{character:?}/{seed}").as_bytes()).into()
}

/// The supplied mode (`m28-diagnostics`): max HP 400 (below the 999 cap) and, on even seeds, all three keys.
#[cfg(feature = "m28-diagnostics")]
const SUPPLIED_HP: u32 = 400;

/// The typed interim strings (erratum v3 n4): map-room combat entry and the event executor.
const INTERIM: [&str; 3] = ["enemy setup facts unavailable for this act/ascension",
    "event combat enemy setup unavailable at this ascension", "event boss enemy setup unavailable at this ascension"];

fn new_run(character: Character, seed: u64, supplied: bool, ascension: u8) -> Result<Run, String> {
    if supplied {
        #[cfg(feature = "m29-diagnostics")]
        if ascension != 0 {
            return spire_core::run::diagnostics::standard_supplied_run_at(character, ascension, seed % 2 == 1, SUPPLIED_HP, seed % 2 == 0, RunRng::new(root(character, seed), Partition::Test, 0))
                .map_err(|error| format!("supplied creation: {error:?}"));
        }
        #[cfg(feature = "m28-diagnostics")]
        return if ascension != 0 { Err("the supplied mode above A0 needs --features m29-diagnostics".into()) } else {
            spire_core::run::diagnostics::standard_supplied_run(character, seed % 2 == 1, SUPPLIED_HP, seed % 2 == 0, RunRng::new(root(character, seed), Partition::Test, 0))
                .map_err(|error| format!("supplied creation: {error:?}")) };
        #[cfg(not(feature = "m28-diagnostics"))]
        return Err("the supplied mode needs --features m28-diagnostics".into());
    }
    let profile = CreationProfile { neow_eligible: seed % 2 == 1, ..CreationProfile::experienced() };
    Run::new_room_scenario(RunConfig { character, ascension }, profile, RunRng::new(root(character, seed), Partition::Test, 0), RoomScenario::Standard)
        .map_err(|error| format!("creation: {error:?}"))
}

/// The variant name of a `Debug` rendering (`Event(Neow)` → `Event`, `Gold(25)` → `Gold`).
fn variant(debug: String) -> String {
    debug.split(|c: char| c == '(' || c == '{' || c == ' ').next().unwrap_or_default().to_string()
}

#[derive(Default)]
struct Census {
    rooms: BTreeMap<String, u64>,
    events: BTreeMap<String, u64>,
    encounters: BTreeMap<String, u64>,
    relics: BTreeSet<String>,
    potions: BTreeSet<String>,
    cards: BTreeSet<String>,
    choices: BTreeMap<String, BTreeMap<String, u64>>,
    terminals: BTreeMap<String, u64>,
    acts: BTreeMap<String, u64>,
}

impl Census {
    fn observe(&mut self, decision: &PublicDecision) {
        decision.player.relics.iter().for_each(|relic| { self.relics.insert(format!("{relic:?}")); });
        decision.player.potions.iter().flatten().for_each(|potion| { self.potions.insert(format!("{potion:?}")); });
        decision.objects.iter().for_each(|object| { self.cards.insert(format!("{:?}", object.id)); });
    }
    fn merge(&mut self, other: &Census) {
        let add = |into: &mut BTreeMap<String, u64>, from: &BTreeMap<String, u64>| from.iter().for_each(|(k, v)| *into.entry(k.clone()).or_default() += v);
        add(&mut self.rooms, &other.rooms);
        add(&mut self.events, &other.events);
        add(&mut self.encounters, &other.encounters);
        add(&mut self.terminals, &other.terminals);
        add(&mut self.acts, &other.acts);
        self.relics.extend(other.relics.iter().cloned());
        self.potions.extend(other.potions.iter().cloned());
        self.cards.extend(other.cards.iter().cloned());
        for (room, choices) in &other.choices { add(self.choices.entry(room.clone()).or_default(), choices); }
    }
    fn to_json(&self) -> Value {
        json!({"rooms":self.rooms,"events":self.events,"encounters":self.encounters,"relics":self.relics,"potions":self.potions,
            "cards":self.cards,"choices_by_room":self.choices,"terminals":self.terminals,"max_act":self.acts})
    }
}

struct Outcome { steps: usize, terminal: Option<String>, fault: Option<String>, act: u8, floor: u32, actions: Vec<GameAction>, last: Option<PublicDecision>,
    interim_rejections: u64, interim_by_context: BTreeMap<String, u64>, interim_blocked: Option<String>, elites: Vec<Value>, transitions: Vec<Value>,
    m30: M30Facts, m31: M31Facts, digests: String }

/// M31 (plan item 8) per-Run facts. Marker steps index the decision at which the class starts:
/// the step whose action gains a G1 relic or rests with Dream Catcher, the first Shop decision (G3),
/// the step whose action loses Potion Belt, and the first decision after that loss holding a potion
/// in a slot at or above the base.
#[derive(Default)]
struct M31Facts { first_g1_step: Option<usize>, first_shop_step: Option<usize>, belt_lost_step: Option<usize>,
    kept_slot_potion_step: Option<usize>, slots_creation: Option<usize>, slots_end: usize, belt_gained: u64, belt_lost: u64,
    card_rows: BTreeMap<String, [u64; 3]>, boss_gold: Vec<Value>, event_options: BTreeMap<String, u64>, mind_bloom_fights: u64,
    note_for_yourself_seen: u64 }

impl M31Facts {
    fn to_json(&self) -> Value {
        let rows: BTreeMap<&String, Value> = self.card_rows.iter().map(|(k, [rows, cards, upgraded])| (k, json!({"rows":rows,"cards":cards,"upgraded":upgraded}))).collect();
        json!({"first_g1_step":self.first_g1_step,"first_shop_step":self.first_shop_step,"belt_lost_step":self.belt_lost_step,
            "kept_slot_potion_step":self.kept_slot_potion_step,"potion_slots_creation":self.slots_creation,"potion_slots_end":self.slots_end,
            "belt_gained":self.belt_gained,"belt_lost":self.belt_lost,"card_rows":rows,"boss_gold":self.boss_gold,
            "event_options":self.event_options,"mind_bloom_fights":self.mind_bloom_fights,"note_for_yourself_seen":self.note_for_yourself_seen})
    }
}

/// G1 sites (M31 design §6.4): the equip card offers and the hidden-row relics.
const G1_RELICS: [RelicId; 4] = [RelicId::Orrery, RelicId::TinyHouse, RelicId::Cauldron, RelicId::CallingBell];

/// M30 (plan item 10) per-Run facts.
#[derive(Default)]
struct M30Facts { start: Option<(u32, u32)>, bane_creation: Option<(usize, Option<usize>)>, bane_end: (usize, Option<usize>), bane_max: usize,
    toke_offered: u64, toke_disabled: u64, toke_cancelled: u64, purge_disabled: u64 }

/// The Bane count and first master-deck index of a decision.
fn bane(decision: &PublicDecision) -> (usize, Option<usize>) {
    (decision.objects.iter().filter(|o| o.id == CardId::AscendersBane).count(), decision.objects.iter().position(|o| o.id == CardId::AscendersBane))
}

impl M30Facts {
    fn to_json(&self) -> Value {
        let bane = |(count, index): (usize, Option<usize>)| json!({"count":count,"index":index});
        json!({"start_hp":self.start.map(|s| s.0),"start_max_hp":self.start.map(|s| s.1),"bane_creation":self.bane_creation.map(bane),
            "bane_end":bane(self.bane_end),"bane_max":self.bane_max,"bane_copies":self.bane_max.saturating_sub(self.bane_creation.map_or(0, |b| b.0)),
            "toke_offered":self.toke_offered,"toke_disabled":self.toke_disabled,"toke_cancelled":self.toke_cancelled,"shop_purge_disabled":self.purge_disabled})
    }
}

/// Attribute a typed interim rejection to public map facts or the named event screen.
fn interim_context(run: &Run, decision: &PublicDecision, action: &GameAction) -> String {
    if let Some(RoomViewKind::Event(event)) = decision.room.as_ref().map(|room| room.kind) {
        return format!("event:{event:?}");
    }
    if let GameAction::Move(id) = action {
        if let Some(map) = run.public_map() {
            if let Some(node) = map.nodes.iter().find(|node| node.id == *id) {
                if node.kind == MapNodeKind::Boss {
                    if let Some(encounter) = map.boss { return format!("act{}/{encounter:?}", map.act); }
                }
                return format!("act{}/{:?}", map.act, node.kind);
            }
        }
    }
    let room = decision.room.as_ref().map(|room| variant(format!("{:?}", room.kind))).unwrap_or_else(|| "unknown".into());
    format!("act{}/{}", decision.act, room)
}

/// One-step lookahead value for the reach policy (combat only): survive, then kill.
fn score(run: &Run) -> i64 {
    let Ok(decision) = run.decision() else { return i64::MIN };
    if decision.terminal.is_some() { return if decision.player.hp == 0 { i64::MIN / 2 } else { 1 << 40 } }
    let Some(combat) = &decision.combat else { return (1 << 30) + 4 * i64::from(decision.player.hp) };
    let enemies: i64 = combat.enemies.iter().filter(|enemy| !enemy.escaped).map(|enemy| i64::from(enemy.hp.max(0) + enemy.block)).sum();
    let incoming: i64 = combat.enemies.iter().filter_map(|enemy| enemy.intent_damage).map(i64::from).sum();
    4 * i64::from(combat.player.hp) + i64::from(combat.player.block).min(incoming) - enemies
}

/// Reach-policy combat value: after the candidate, play the turn out greedily by `score` (at most
/// 24 actions) and value the state when the turn changes or the combat ends.
fn rollout(mut run: Run) -> i64 {
    let turn = |run: &Run| run.decision().ok().and_then(|d| d.combat.map(|c| c.turn));
    let start = turn(&run);
    for _ in 0..24 {
        if start.is_none() || turn(&run) != start { break }
        let Ok(decision) = run.decision() else { break };
        if decision.terminal.is_some() { break }
        let best = decision.candidates.iter().filter_map(|candidate| {
            let mut clone = run.clone();
            clone.step(&candidate.action).ok().map(|_| (score(&clone), clone))
        }).max_by_key(|(value, _)| *value);
        match best { Some((_, next)) => run = next, None => break }
    }
    score(&run)
}

fn traverse(character: Character, seed: u64, cap: usize, reach: bool, supplied: bool, ascension: u8, census: &mut Census) -> Outcome {
    let mut out = Outcome { steps: 0, terminal: None, fault: None, act: 0, floor: 0, actions: Vec::new(), last: None,
        interim_rejections: 0, interim_by_context: BTreeMap::new(), interim_blocked: None, elites: Vec::new(), transitions: Vec::new(), m30: M30Facts::default(),
        m31: M31Facts::default(), digests: String::new() };
    let mut run = match new_run(character, seed, supplied, ascension) { Ok(run) => run, Err(error) => { out.fault = Some(error); return out } };
    let mut map_act = 0u8;
    let mut policy = u64::from_le_bytes(root(character, seed)[..8].try_into().unwrap()) ^ if reach { 0x5245_4143_48 } else { 0x4D32_3846 };
    let mut room_kind = String::new();
    let mut last_choice: Option<RoomChoiceKind> = None;
    let base_slots = if ascension >= 11 { 2 } else { 3 };
    let mut equip_open = false;
    loop {
        let decision = match run.decision() { Ok(decision) => decision, Err(error) => { out.fault = Some(format!("decision at step {}: {error:?}", out.steps)); break } };
        census.observe(&decision);
        // M31 facts read at every decision.
        out.m31.slots_creation.get_or_insert(decision.player.potions.len());
        out.m31.slots_end = decision.player.potions.len();
        if out.m31.belt_lost_step.is_some() && out.m31.kept_slot_potion_step.is_none()
            && decision.player.potions.iter().skip(base_slots).any(Option::is_some) { out.m31.kept_slot_potion_step = Some(out.steps); }
        if out.m31.first_shop_step.is_none() && decision.room.as_ref().is_some_and(|room| room.kind == RoomViewKind::Shop) { out.m31.first_shop_step = Some(out.steps); }
        // M30 facts: creation values from the first decision; the Bane at every decision (the last one is the end).
        if out.m30.start.is_none() { out.m30.start = Some((decision.player.hp, decision.player.max_hp)); out.m30.bane_creation = Some(bane(&decision)); }
        out.m30.bane_end = bane(&decision);
        out.m30.bane_max = out.m30.bane_max.max(out.m30.bane_end.0);
        (out.act, out.floor) = (decision.act, decision.floor);
        // Each fresh ordinary map's elite target (connected rows 0–14 except row 13) and placement.
        if let Some(map) = run.public_map().filter(|map| map.act != map_act) {
            map_act = map.act;
            if (1..=3).contains(&map.act) {
                let n = map.nodes.iter().filter(|node| node.y <= 14 && node.y != 13).count();
                let requested = room_quotas(n, ascension)[2];
                let placed = map.nodes.iter().filter(|node| node.kind == MapNodeKind::Elite).count();
                out.elites.push(json!({"act":map.act,"n":n,"requested":requested,"placed":placed,"unplaced":requested.saturating_sub(placed)}));
            }
        }
        if decision.content_profile != Some(RoomScenario::Standard) {
            out.fault = Some(format!("step {}: content profile {:?} is not Standard", out.steps, decision.content_profile));
            break;
        }
        if let Some(room) = &decision.room {
            let kind = variant(format!("{:?}", room.kind));
            if kind != room_kind {
                *census.rooms.entry(kind.clone()).or_default() += 1;
                // M31: offered card rows by act and source; boss gold rows; Note For Yourself.
                if !matches!(room.kind, RoomViewKind::Rewards | RoomViewKind::CardReward) { equip_open = false; }
                if room.kind == RoomViewKind::CardReward {
                    let cards: Vec<u32> = room.choices.iter().filter_map(|row| match row.kind { RoomChoiceKind::Card { upgrades, .. } => Some(upgrades), _ => None }).collect();
                    let entry = out.m31.card_rows.entry(format!("act{}/{}", decision.act, if equip_open { "equip" } else { "combat_or_event" })).or_default();
                    entry[0] += 1; entry[1] += cards.len() as u64; entry[2] += cards.iter().filter(|u| **u > 0).count() as u64;
                }
                if room.kind == RoomViewKind::Rewards {
                    let boss = run.public_map().is_some_and(|map| map.current.and_then(|c| map.nodes.iter().find(|n| n.id == c)).is_some_and(|n| n.kind == MapNodeKind::Boss));
                    let gold: Vec<u32> = room.choices.iter().filter_map(|row| match row.kind { RoomChoiceKind::Gold(g) => Some(g), _ => None }).collect();
                    if boss && !gold.is_empty() && !equip_open {
                        out.m31.boss_gold.push(json!({"act":decision.act,"gold":gold,"golden_idol":decision.player.relics.contains(&RelicId::GoldenIdol)}));
                    }
                }
                if room.kind == RoomViewKind::Event(EventId::NoteForYourself) { out.m31.note_for_yourself_seen += 1; }
                // M30: Toke and purge row status at room entry (G1, G2).
                for row in &room.choices {
                    match (&row.kind, row.status) {
                        (RoomChoiceKind::Toke, RoomChoiceStatus::Offered) => out.m30.toke_offered += 1,
                        (RoomChoiceKind::Toke, RoomChoiceStatus::Disabled) => out.m30.toke_disabled += 1,
                        // G2 only: a purge row Disabled although its price is affordable (not the price rule).
                        (RoomChoiceKind::Purge, RoomChoiceStatus::Disabled) if row.price.is_some_and(|p| p <= decision.player.gold) => out.m30.purge_disabled += 1,
                        _ => (),
                    }
                }
                if let RoomViewKind::Event(event) = room.kind { *census.events.entry(format!("{event:?}")).or_default() += 1; }
                room_kind = kind;
            }
        }
        if let Some(terminal) = decision.terminal {
            out.digests.push_str(&digest(out.steps, &decision, None));
            out.terminal = Some(format!("{terminal:?}")); out.last = Some(decision); break
        }
        if decision.candidates.is_empty() && env::var("M28_TRACE").is_ok() { eprintln!("DUMP {decision:#?}"); }
        if decision.candidates.is_empty() { out.fault = Some(format!("step {}: non-terminal decision with no candidates: {:?}", out.steps, decision.room)); break }
        if out.steps >= cap { out.fault = Some(format!("action cap {cap} reached at act {} floor {}", decision.act, decision.floor)); break }
        // Every advertised candidate is stepped on a clone: any rejection is a fault (all-legal check),
        // except a typed interim rejection above A0, which leaves the policy's choice set.
        let mut scores = Vec::with_capacity(decision.candidates.len());
        let mut open = Vec::with_capacity(decision.candidates.len());
        for (position, candidate) in decision.candidates.iter().enumerate() {
            let mut clone = run.clone();
            match catch_unwind(AssertUnwindSafe(|| clone.step(&candidate.action))) {
                Ok(Ok(_)) => { open.push(position); scores.push(if !reach { 0 } else if decision.combat.is_some() { rollout(clone) } else { score(&clone) }) }
                Ok(Err(RunError::Unsupported(message))) if ascension != 0 && INTERIM.contains(&message) => {
                    out.interim_rejections += 1;
                    *out.interim_by_context.entry(interim_context(&run, &decision, &candidate.action)).or_default() += 1;
                }
                Ok(Err(error)) => { out.fault = Some(format!("step {}: advertised {:?} rejected on clone: {error:?} (room {:?})", out.steps, candidate.action, decision.room.as_ref().map(|room| &room.kind))); break }
                Err(_) => { out.fault = Some(format!("step {}: panic on clone for {:?}", out.steps, candidate.action)); break }
            }
        }
        if out.fault.is_some() { break }
        if open.is_empty() {
            out.interim_blocked = Some(format!("step {} act {} floor {} room {:?}", out.steps, decision.act, decision.floor, decision.room.as_ref().map(|room| &room.kind)));
            out.last = Some(decision);
            break;
        }
        let draw = splitmix(&mut policy);
        let greedy = if decision.combat.is_some() { draw % 10 != 0 } else { draw % 2 == 0 };
        let index = if reach && greedy {
            let best = *scores.iter().max().unwrap();
            let ties: Vec<usize> = (0..scores.len()).filter(|i| scores[*i] == best).collect();
            ties[(draw >> 8) as usize % ties.len()]
        } else { ((draw >> 32) % open.len() as u64) as usize };
        let candidate = &decision.candidates[open[index]];
        if env::var("M28_TRACE").is_ok() { eprintln!("TRACE f{} hp{} room {:?} combat_turn {:?} pick {:?} scores {:?} actions {:?} selection {:?} hand {:?}", decision.floor, decision.player.hp, decision.room.as_ref().map(|r| &r.kind), decision.combat.as_ref().map(|c| (c.turn, c.player.hp, c.player.energy, c.enemies.iter().map(|e| e.hp).collect::<Vec<_>>())), candidate.action, scores, decision.candidates.iter().map(|c| &c.action).collect::<Vec<_>>(), decision.combat.as_ref().map(|c| &c.selection), decision.combat.as_ref().map(|c| c.hand.iter().map(|h| format!("{:?}", h.id)).collect::<Vec<_>>())); }
        if let (None, GameAction::Choose(id)) = (&decision.combat, &candidate.action) {
            if let Some(row) = decision.room.as_ref().and_then(|room| room.choices.iter().find(|row| row.id == *id)) {
                *census.choices.entry(room_kind.clone()).or_default().entry(variant(format!("{:?}", row.kind))).or_default() += 1;
            }
        }
        let action = candidate.action.clone();
        if action == GameAction::Cancel && last_choice == Some(RoomChoiceKind::Toke) { out.m30.toke_cancelled += 1; }
        if let GameAction::Choose(id) = &action {
            last_choice = decision.room.as_ref().and_then(|room| room.choices.iter().find(|row| row.id == *id)).map(|row| row.kind.clone());
            if let (Some(RoomViewKind::Event(event)), Some(RoomChoiceKind::EventOption { stage, option })) = (decision.room.as_ref().map(|r| r.kind), &last_choice) {
                *out.m31.event_options.entry(format!("{event:?}/{stage}/{option}")).or_default() += 1;
                if event == EventId::MindBloom && *stage == 0 && *option == 0 { out.m31.mind_bloom_fights += 1; }
            }
            if matches!(last_choice, Some(RoomChoiceKind::Rest { .. })) && decision.player.relics.contains(&RelicId::DreamCatcher) {
                out.m31.first_g1_step.get_or_insert(out.steps);
                equip_open = true;
            }
        }
        out.digests.push_str(&digest(out.steps, &decision, Some(&action)));
        let relics_before = decision.player.relics.clone();
        match catch_unwind(AssertUnwindSafe(|| run.step(&action))) {
            Ok(Ok(transition)) => {
                if let Ok(after) = run.decision() {
                    let gained: Vec<RelicId> = after.player.relics.iter().filter(|r| !relics_before.contains(r)).copied().collect();
                    if gained.iter().any(|r| G1_RELICS.contains(r)) { out.m31.first_g1_step.get_or_insert(out.steps); equip_open = true; }
                    if gained.contains(&RelicId::PotionBelt) { out.m31.belt_gained += 1; }
                    if relics_before.contains(&RelicId::PotionBelt) && !after.player.relics.contains(&RelicId::PotionBelt) {
                        out.m31.belt_lost += 1;
                        out.m31.belt_lost_step.get_or_insert(out.steps);
                    }
                }
                for event in &transition.committed_events {
                    if let CommittedEvent::CombatEntered { encounter, .. } = event { *census.encounters.entry(format!("{encounter:?}")).or_default() += 1; }
                    if let CommittedEvent::ActAdvanced { from, to } = event {
                        let after = run.decision().ok().map(|d| (d.player.hp, d.player.max_hp));
                        out.transitions.push(json!({"from":from,"to":to,"hp_before":decision.player.hp,"max_hp_before":decision.player.max_hp,
                            "hp_after":after.map(|a| a.0),"max_hp_after":after.map(|a| a.1)}));
                    }
                }
            }
            Ok(Err(error)) => { out.fault = Some(format!("step {}: advertised {action:?} rejected: {error:?} (room {:?})", out.steps, decision.room.map(|room| room.kind))); break }
            Err(_) => { out.fault = Some(format!("step {}: panic on {action:?}", out.steps)); break }
        }
        out.actions.push(action);
        out.steps += 1;
    }
    if let Some(terminal) = &out.terminal { *census.terminals.entry(terminal.clone()).or_default() += 1; }
    *census.acts.entry(out.act.to_string()).or_default() += 1;
    out
}

/// One step's decision digest (review n1): the first 48 bits of sha256 over the step, the public
/// decision and the chosen action (none at a terminal), as 12 hex digits.
fn digest(step: usize, decision: &PublicDecision, action: Option<&GameAction>) -> String {
    let hash = Sha256::digest(format!("{step}|{decision:?}|{action:?}").as_bytes());
    hash[..6].iter().map(|b| format!("{b:02x}")).collect()
}

/// Replays the recorded actions on a fresh Run; the final decision must equal the live one.
fn replay(character: Character, seed: u64, supplied: bool, ascension: u8, outcome: &Outcome) -> Result<(), String> {
    let mut run = new_run(character, seed, supplied, ascension)?;
    for (index, action) in outcome.actions.iter().enumerate() {
        run.step(action).map_err(|error| format!("replay step {index}: {error:?}"))?;
    }
    let decision = run.decision().map_err(|error| format!("replay decision: {error:?}"))?;
    if Some(&decision) != outcome.last.as_ref() { return Err("replay final decision differs".into()) }
    Ok(())
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    let arg = |name: &str| args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).cloned();
    let seeds: u64 = arg("--seeds").map(|v| v.parse().expect("--seeds")).unwrap_or(25);
    let cap: usize = arg("--cap").map(|v| v.parse().expect("--cap")).unwrap_or(20_000);
    let out = PathBuf::from(arg("--out").expect("--out DIR"));
    fs::create_dir_all(&out).expect("create --out");
    let policies: Vec<&str> = match arg("--policy").as_deref() {
        Some("uniform") => vec!["uniform"], Some("reach") => vec!["reach"], Some("supplied") => vec!["supplied"],
        Some(other) => panic!("unknown --policy {other}"),
        None if cfg!(feature = "m28-diagnostics") => vec!["uniform", "reach", "supplied"],
        None => vec!["uniform", "reach"],
    };
    let only = arg("--only"); // `Character/seed`, for reproducing one Run
    let ascensions: Vec<u8> = match (arg("--ascension"), arg("--ascensions")) {
        (Some(level), None) => vec![level.parse().expect("--ascension")],
        (None, Some(range)) => {
            let (low, high) = range.split_once('-').expect("--ascensions A-B");
            (low.parse().expect("--ascensions")..=high.parse().expect("--ascensions")).collect()
        }
        (None, None) => vec![0],
        _ => panic!("use --ascension or --ascensions, not both"),
    };
    let mut any_fault = false;
    for &ascension in &ascensions {
        let dir = if ascensions.len() == 1 { out.clone() } else { out.join(format!("a{ascension}")) };
        fs::create_dir_all(&dir).expect("create ascension dir");
        any_fault |= !run_level(&dir, ascension, seeds, cap, &policies, only.as_deref());
    }
    if any_fault { ExitCode::FAILURE } else { ExitCode::SUCCESS }
}

/// One ascension's traversal; writes `traversal.json` to `dir` and returns whether it was fault-free.
fn run_level(dir: &std::path::Path, ascension: u8, seeds: u64, cap: usize, policies: &[&str], only: Option<&str>) -> bool {
    let jobs: Vec<(&str, Character, u64)> = policies.iter().flat_map(|p| CHARACTERS.into_iter().flat_map(move |c| (1..=seeds).map(move |seed| (*p, c, seed))))
        .filter(|(_, c, seed)| only.is_none_or(|only| only == format!("{c:?}/{seed}"))).collect();
    // Runs are independent and deterministic; each is its own thread, results kept in job order.
    let results: Vec<(Value, Value, Value, Value, String, Census)> = std::thread::scope(|scope| {
        let handles: Vec<_> = jobs.iter().map(|&(policy, character, seed)| scope.spawn(move || {
            let mut census = Census::default();
            let supplied = policy == "supplied";
            let mut outcome = traverse(character, seed, cap, policy != "uniform", supplied, ascension, &mut census);
            if outcome.fault.is_none() {
                if let Err(error) = replay(character, seed, supplied, ascension, &outcome) { outcome.fault = Some(error) }
            }
            let record = json!({"policy":policy,"character":format!("{character:?}"),"seed":seed,"neow":if seed % 2 == 1 {"full"} else {"short"},
                "steps":outcome.steps,"terminal":outcome.terminal,"act":outcome.act,"floor":outcome.floor,"fault":outcome.fault,
                "replay":if outcome.fault.is_none() {"equal"} else {"not run"}});
            // M29 fields, kept beside the M28 record so the A0 control projection is unchanged (erratum v3 n3).
            let m29 = json!({"policy":policy,"character":format!("{character:?}"),"seed":seed,"ascension":ascension,
                "interim_rejections":outcome.interim_rejections,"interim_by_context":outcome.interim_by_context,
                "interim_blocked":outcome.interim_blocked,"elites":outcome.elites,"transitions":outcome.transitions});
            let m30 = json!({"policy":policy,"character":format!("{character:?}"),"seed":seed,"ascension":ascension,"facts":outcome.m30.to_json()});
            let m31 = json!({"policy":policy,"character":format!("{character:?}"),"seed":seed,"ascension":ascension,"facts":outcome.m31.to_json()});
            let digests = json!({"policy":policy,"character":format!("{character:?}"),"seed":seed,"ascension":ascension,"digests":outcome.digests}).to_string();
            eprintln!("{record} {m29} {m30} {m31}");
            (record, m29, m30, m31, digests, census)
        })).collect();
        handles.into_iter().map(|handle| handle.join().expect("traversal thread")).collect()
    });
    let (mut runs, mut faults, mut total, mut per_character) = (Vec::new(), Vec::new(), Census::default(), BTreeMap::<String, Census>::new());
    let (mut m29_runs, mut blocked, mut rejections, mut interim_by_context) = (Vec::new(), 0u64, 0u64, BTreeMap::<String, u64>::new());
    let mut elite_census = BTreeMap::<String, [u64; 3]>::new();
    let (mut m30_runs, mut m30_totals) = (Vec::new(), BTreeMap::<String, u64>::new());
    let (mut m31_runs, mut m31_totals, mut digest_lines) = (Vec::new(), BTreeMap::<String, u64>::new(), Vec::new());
    for ((policy, character, _), (record, m29, m30, m31, digests, census)) in jobs.iter().zip(results) {
        let facts = &m31["facts"];
        for key in ["belt_gained", "belt_lost", "mind_bloom_fights", "note_for_yourself_seen"] {
            *m31_totals.entry(key.to_string()).or_default() += facts[key].as_u64().unwrap_or(0);
        }
        for key in ["first_g1_step", "first_shop_step", "belt_lost_step", "kept_slot_potion_step"] {
            *m31_totals.entry(format!("runs_with_{key}")).or_default() += u64::from(!facts[key].is_null());
        }
        if let Some(slots) = facts["potion_slots_creation"].as_u64() { *m31_totals.entry(format!("runs_created_with_{slots}_slots")).or_default() += 1; }
        *m31_totals.entry(format!("runs_ended_with_{}_slots", facts["potion_slots_end"])).or_default() += 1;
        *m31_totals.entry("boss_gold_rows".to_string()).or_default() += facts["boss_gold"].as_array().map_or(0, |rows| rows.len() as u64);
        m31_runs.push(m31);
        digest_lines.push(digests);
        for key in ["toke_offered", "toke_disabled", "toke_cancelled", "shop_purge_disabled", "bane_copies"] {
            *m30_totals.entry(key.to_string()).or_default() += m30["facts"][key].as_u64().unwrap_or(0);
        }
        m30_runs.push(m30);
        if !record["fault"].is_null() { faults.push(record.clone()) }
        total.merge(&census);
        per_character.entry(format!("{policy}/{character:?}")).or_default().merge(&census);
        runs.push(record);
        blocked += u64::from(!m29["interim_blocked"].is_null());
        rejections += m29["interim_rejections"].as_u64().unwrap_or(0);
        if let Some(contexts) = m29["interim_by_context"].as_object() {
            for (context, count) in contexts { *interim_by_context.entry(context.clone()).or_default() += count.as_u64().unwrap_or(0); }
        }
        for elite in m29["elites"].as_array().into_iter().flatten() {
            let entry = elite_census.entry(format!("act{}", elite["act"])).or_default();
            for (slot, key) in ["requested", "placed", "unplaced"].into_iter().enumerate() { entry[slot] += elite[key].as_u64().unwrap_or(0) }
        }
        m29_runs.push(m29);
    }
    let per_character: BTreeMap<String, Value> = per_character.iter().map(|(key, census)| (key.clone(), census.to_json())).collect();
    let elite_census: BTreeMap<String, Value> = elite_census.into_iter().map(|(act, [requested, placed, unplaced])| (act, json!({"requested":requested,"placed":placed,"unplaced":unplaced}))).collect();
    let report = json!({"schema":"spire-gym.m28-traversal.v1","seeds_per_character":seeds,"cap":cap,"policies":{"uniform":"seeded uniform random over advertised candidates","supplied":"reach policy on a Standard Run with supplied max HP 400 and, on even seeds, all three keys (m28-diagnostics; grade Supplied, not traversal)","reach":"combat: each candidate valued by a greedy play-out of the turn (survive, then kill); out of combat: one-step current-HP value; greedy on 90% of combat and 50% of other decisions, else uniform"},
        "runs":runs.len(),"fault_free":runs.len() - faults.len(),"faults":faults,"census":total.to_json(),"census_by_character":per_character,"results":runs,
        "m29":{"ascension":ascension,"interim_blocked_runs":blocked,"interim_rejections":rejections,"interim_by_context":interim_by_context,
            "elites_by_act":elite_census,"runs":m29_runs},
        "m30":{"ascension":ascension,"totals":m30_totals,"runs":m30_runs},
        "m31":{"ascension":ascension,"totals":m31_totals,"runs":m31_runs}});
    fs::write(dir.join("digests.jsonl"), digest_lines.join("\n") + "\n").expect("write digests.jsonl");
    fs::write(dir.join("traversal.json"), serde_json::to_vec_pretty(&report).unwrap()).expect("write traversal.json");
    println!("A{ascension}: {} of {} runs fault-free ({blocked} interim-blocked)", runs.len() - faults.len(), runs.len());
    faults.is_empty()
}
