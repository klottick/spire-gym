//! M28 traversal (design §14, plan Task X1, PM ruling Q3): for each character, `--seeds` seeded
//! `StandardA0` Runs (full Neow on odd seeds, short on even) driven by a seeded uniform random
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
//! Usage: cargo run --release -p spire-core --example m28_traversal [--features m28-diagnostics] -- --out DIR
//!        [--seeds 25] [--cap 20000] [--policy uniform|reach|supplied]
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use spire_core::{
    combat::rng::Partition,
    content::Character,
    run::{CommittedEvent, CreationProfile, GameAction, PublicDecision, RoomScenario, RoomViewKind, Run, RunConfig, RunRng},
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

fn new_run(character: Character, seed: u64, supplied: bool) -> Result<Run, String> {
    if supplied {
        #[cfg(feature = "m28-diagnostics")]
        return spire_core::run::diagnostics::standard_supplied_run(character, seed % 2 == 1, SUPPLIED_HP, seed % 2 == 0, RunRng::new(root(character, seed), Partition::Test, 0))
            .map_err(|error| format!("supplied creation: {error:?}"));
        #[cfg(not(feature = "m28-diagnostics"))]
        return Err("the supplied mode needs --features m28-diagnostics".into());
    }
    let profile = CreationProfile { neow_eligible: seed % 2 == 1, ..CreationProfile::experienced() };
    Run::new_room_scenario(RunConfig { character, ascension: 0 }, profile, RunRng::new(root(character, seed), Partition::Test, 0), RoomScenario::StandardA0)
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

struct Outcome { steps: usize, terminal: Option<String>, fault: Option<String>, act: u8, floor: u32, actions: Vec<GameAction>, last: Option<PublicDecision> }

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

fn traverse(character: Character, seed: u64, cap: usize, reach: bool, supplied: bool, census: &mut Census) -> Outcome {
    let mut out = Outcome { steps: 0, terminal: None, fault: None, act: 0, floor: 0, actions: Vec::new(), last: None };
    let mut run = match new_run(character, seed, supplied) { Ok(run) => run, Err(error) => { out.fault = Some(error); return out } };
    let mut policy = u64::from_le_bytes(root(character, seed)[..8].try_into().unwrap()) ^ if reach { 0x5245_4143_48 } else { 0x4D32_3846 };
    let mut room_kind = String::new();
    loop {
        let decision = match run.decision() { Ok(decision) => decision, Err(error) => { out.fault = Some(format!("decision at step {}: {error:?}", out.steps)); break } };
        census.observe(&decision);
        (out.act, out.floor) = (decision.act, decision.floor);
        if decision.content_profile != Some(RoomScenario::StandardA0) {
            out.fault = Some(format!("step {}: content profile {:?} is not StandardA0", out.steps, decision.content_profile));
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
        // Every advertised candidate is stepped on a clone: any rejection is a fault (all-legal check).
        let mut scores = Vec::with_capacity(decision.candidates.len());
        for candidate in &decision.candidates {
            let mut clone = run.clone();
            match catch_unwind(AssertUnwindSafe(|| clone.step(&candidate.action))) {
                Ok(Ok(_)) => scores.push(if !reach { 0 } else if decision.combat.is_some() { rollout(clone) } else { score(&clone) }),
                Ok(Err(error)) => { out.fault = Some(format!("step {}: advertised {:?} rejected on clone: {error:?} (room {:?})", out.steps, candidate.action, decision.room.as_ref().map(|room| &room.kind))); break }
                Err(_) => { out.fault = Some(format!("step {}: panic on clone for {:?}", out.steps, candidate.action)); break }
            }
        }
        if out.fault.is_some() { break }
        let draw = splitmix(&mut policy);
        let greedy = if decision.combat.is_some() { draw % 10 != 0 } else { draw % 2 == 0 };
        let index = if reach && greedy {
            let best = *scores.iter().max().unwrap();
            let ties: Vec<usize> = (0..scores.len()).filter(|i| scores[*i] == best).collect();
            ties[(draw >> 8) as usize % ties.len()]
        } else { ((draw >> 32) % decision.candidates.len() as u64) as usize };
        let candidate = &decision.candidates[index];
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
fn replay(character: Character, seed: u64, supplied: bool, outcome: &Outcome) -> Result<(), String> {
    let mut run = new_run(character, seed, supplied)?;
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
    let jobs: Vec<(&str, Character, u64)> = policies.iter().flat_map(|p| CHARACTERS.into_iter().flat_map(move |c| (1..=seeds).map(move |seed| (*p, c, seed))))
        .filter(|(_, c, seed)| only.as_ref().is_none_or(|only| *only == format!("{c:?}/{seed}"))).collect();
    // Runs are independent and deterministic; each is its own thread, results kept in job order.
    let results: Vec<(Value, Census)> = std::thread::scope(|scope| {
        let handles: Vec<_> = jobs.iter().map(|&(policy, character, seed)| scope.spawn(move || {
            let mut census = Census::default();
            let supplied = policy == "supplied";
            let mut outcome = traverse(character, seed, cap, policy != "uniform", supplied, &mut census);
            if outcome.fault.is_none() {
                if let Err(error) = replay(character, seed, supplied, &outcome) { outcome.fault = Some(error) }
            }
            let record = json!({"policy":policy,"character":format!("{character:?}"),"seed":seed,"neow":if seed % 2 == 1 {"full"} else {"short"},
                "steps":outcome.steps,"terminal":outcome.terminal,"act":outcome.act,"floor":outcome.floor,"fault":outcome.fault,
                "replay":if outcome.fault.is_none() {"equal"} else {"not run"}});
            eprintln!("{record}");
            (record, census)
        })).collect();
        handles.into_iter().map(|handle| handle.join().expect("traversal thread")).collect()
    });
    let (mut runs, mut faults, mut total, mut per_character) = (Vec::new(), Vec::new(), Census::default(), BTreeMap::<String, Census>::new());
    for ((policy, character, _), (record, census)) in jobs.iter().zip(results) {
        if !record["fault"].is_null() { faults.push(record.clone()) }
        total.merge(&census);
        per_character.entry(format!("{policy}/{character:?}")).or_default().merge(&census);
        runs.push(record);
    }
    let per_character: BTreeMap<String, Value> = per_character.iter().map(|(key, census)| (key.clone(), census.to_json())).collect();
    let report = json!({"schema":"spire-gym.m28-traversal.v1","seeds_per_character":seeds,"cap":cap,"policies":{"uniform":"seeded uniform random over advertised candidates","supplied":"reach policy on a StandardA0 Run with supplied max HP 400 and, on even seeds, all three keys (m28-diagnostics; grade Supplied, not traversal)","reach":"combat: each candidate valued by a greedy play-out of the turn (survive, then kill); out of combat: one-step current-HP value; greedy on 90% of combat and 50% of other decisions, else uniform"},
        "runs":runs.len(),"fault_free":runs.len() - faults.len(),"faults":faults,"census":total.to_json(),"census_by_character":per_character,"results":runs});
    fs::write(out.join("traversal.json"), serde_json::to_vec_pretty(&report).unwrap()).expect("write traversal.json");
    println!("{} of {} runs fault-free", runs.len() - faults.len(), runs.len());
    if faults.is_empty() { ExitCode::SUCCESS } else { ExitCode::FAILURE }
}
