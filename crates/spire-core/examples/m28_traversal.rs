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
//! (A1–A5 need `--features m29-diagnostics` until M29f). An advertised candidate rejected with a
//! typed interim (an encounter whose enemy facts have not landed at this ascension) is counted and
//! left out of the policy's choice set; a decision whose every candidate is an interim rejection
//! ends the Run as `interim_blocked`, which is not a fault. Each Run also records interim counts by
//! public room context, its per-act elite target/placement and its HP around each act transition. One ascension writes
//! `DIR/traversal.json`; a range writes `DIR/a<N>/traversal.json` per level.
//!
//! Usage: cargo run --release -p spire-core --example m28_traversal [--features m28-diagnostics|m29-diagnostics] -- --out DIR
//!        [--seeds 25] [--cap 20000] [--policy uniform|reach|supplied] [--ascension N | --ascensions A-B]
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use spire_core::{
    combat::rng::Partition,
    content::Character,
    run::{map::room_quotas, CommittedEvent, CreationProfile, GameAction, MapNodeKind, PublicDecision, RoomScenario, RoomViewKind, Run, RunConfig, RunError, RunRng},
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
    interim_rejections: u64, interim_by_context: BTreeMap<String, u64>, interim_blocked: Option<String>, elites: Vec<Value>, transitions: Vec<Value> }

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
        interim_rejections: 0, interim_by_context: BTreeMap::new(), interim_blocked: None, elites: Vec::new(), transitions: Vec::new() };
    let mut run = match new_run(character, seed, supplied, ascension) { Ok(run) => run, Err(error) => { out.fault = Some(error); return out } };
    let mut map_act = 0u8;
    let mut policy = u64::from_le_bytes(root(character, seed)[..8].try_into().unwrap()) ^ if reach { 0x5245_4143_48 } else { 0x4D32_3846 };
    let mut room_kind = String::new();
    loop {
        let decision = match run.decision() { Ok(decision) => decision, Err(error) => { out.fault = Some(format!("decision at step {}: {error:?}", out.steps)); break } };
        census.observe(&decision);
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
                if let RoomViewKind::Event(event) = room.kind { *census.events.entry(format!("{event:?}")).or_default() += 1; }
                room_kind = kind;
            }
        }
        if let Some(terminal) = decision.terminal { out.terminal = Some(format!("{terminal:?}")); out.last = Some(decision); break }
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
        match catch_unwind(AssertUnwindSafe(|| run.step(&action))) {
            Ok(Ok(transition)) => {
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
    let results: Vec<(Value, Value, Census)> = std::thread::scope(|scope| {
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
            eprintln!("{record} {m29}");
            (record, m29, census)
        })).collect();
        handles.into_iter().map(|handle| handle.join().expect("traversal thread")).collect()
    });
    let (mut runs, mut faults, mut total, mut per_character) = (Vec::new(), Vec::new(), Census::default(), BTreeMap::<String, Census>::new());
    let (mut m29_runs, mut blocked, mut rejections, mut interim_by_context) = (Vec::new(), 0u64, 0u64, BTreeMap::<String, u64>::new());
    let mut elite_census = BTreeMap::<String, [u64; 3]>::new();
    for ((policy, character, _), (record, m29, census)) in jobs.iter().zip(results) {
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
            "elites_by_act":elite_census,"runs":m29_runs}});
    fs::write(dir.join("traversal.json"), serde_json::to_vec_pretty(&report).unwrap()).expect("write traversal.json");
    println!("A{ascension}: {} of {} runs fault-free ({blocked} interim-blocked)", runs.len() - faults.len(), runs.len());
    faults.is_empty()
}
