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
//! (A1–A5 are public since the M29 close, A6–A10 since M30, A11–A15 since M31, A16–A19 since M32 and A20 since M33; the supplied mode above A0 needs `--features m29-diagnostics`). An advertised candidate rejected with a
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
//! M34a: `M28_DIGEST_PROJECTION=pre-m34a` digests the decision text without the M34a public-view
//! fields, so the A0 control compares an M34a tree with a pre-M34a tree on the same projection.
//!
//! M33: the separate `m33` key records the public Act 3 boss sequence and Act 4/Heart reach; it
//! does not alter the A0 control projection, existing `results` rows, or `digests.jsonl`.
//!
//! Usage: cargo run --release -p spire-core --example m28_traversal [--features m28-diagnostics|m29-diagnostics] -- --out DIR
//!        [--seeds 25] [--cap 20000] [--policy uniform|reach|supplied] [--ascension N | --ascensions A-B]
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use spire_core::{
    combat::rng::Partition,
    content::{CardId, Character, EncounterId, EventId, RelicId},
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
    m30: M30Facts, m31: M31Facts, m32: M32Facts, m33: M33Facts, digests: String }

#[derive(Default)]
struct M33BossFight { floor: u32, encounter: Option<String>, won: Option<bool> }

#[derive(Default)]
struct M33Facts { act3_boss_fights: Vec<M33BossFight>, second_act3_boss_entered: bool,
    act4_rest_floor: Option<u32>, heart_entered: bool, heart_won: bool }

impl M33Facts {
    fn to_json(&self) -> Value {
        json!({"act3_boss_fights":self.act3_boss_fights.iter().map(|fight| json!({
            "floor":fight.floor,"encounter":fight.encounter,"won":fight.won
        })).collect::<Vec<_>>(),"second_act3_boss_entered":self.second_act3_boss_entered,
        "act4_rest_floor":self.act4_rest_floor,"heart_entered":self.heart_entered,"heart_won":self.heart_won})
    }
}

#[derive(Default)]
struct M33Totals { runs_with_0_act3_boss_fights:u64, runs_with_1_act3_boss_fight:u64,
    runs_with_2_act3_boss_fights:u64, runs_with_other_act3_boss_fight_count:u64,
    distinct_pair_violations:u64, runs_reaching_act4:u64, runs_reaching_heart:u64,
    runs_entering_second_act3_boss:u64 }

impl M33Totals {
    fn add_run(&mut self, facts:&M33Facts) {
        match facts.act3_boss_fights.len() {
            0 => self.runs_with_0_act3_boss_fights += 1,
            1 => self.runs_with_1_act3_boss_fight += 1,
            2 => self.runs_with_2_act3_boss_fights += 1,
            _ => self.runs_with_other_act3_boss_fight_count += 1,
        }
        let mut seen = BTreeSet::new();
        if facts.act3_boss_fights.iter().filter_map(|fight| fight.encounter.as_ref()).any(|id| !seen.insert(id)) {
            self.distinct_pair_violations += 1;
        }
        self.runs_reaching_act4 += u64::from(facts.act4_rest_floor.is_some());
        self.runs_reaching_heart += u64::from(facts.heart_entered);
        self.runs_entering_second_act3_boss += u64::from(facts.second_act3_boss_entered);
    }

    fn to_json(&self) -> Value {
        json!({"runs_with_0_act3_boss_fights":self.runs_with_0_act3_boss_fights,
            "runs_with_1_act3_boss_fight":self.runs_with_1_act3_boss_fight,
            "runs_with_2_act3_boss_fights":self.runs_with_2_act3_boss_fights,
            "runs_with_other_act3_boss_fight_count":self.runs_with_other_act3_boss_fight_count,
            "distinct_pair_violations":self.distinct_pair_violations,
            "runs_reaching_act4":self.runs_reaching_act4,"runs_reaching_heart":self.runs_reaching_heart,
            "runs_entering_second_act3_boss":self.runs_entering_second_act3_boss})
    }
}

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

#[derive(Clone, Copy)]
enum PatternToken { Any, Intent(&'static str) }
#[derive(Clone, Copy)]
enum IntentPattern {
    OpeningIs(&'static str),
    OpeningNot(&'static str),
    Sequence(&'static [PatternToken]),
}
#[derive(Clone, Copy)]
struct IntentRule {
    id: &'static str,
    species: &'static str,
    threshold: u8,
    pattern: IntentPattern,
    lane_locator: &'static str,
    source_rule: &'static str,
}

const FORBIDDEN_INTENT_RULES: &[IntentRule] = &[
    IntentRule { id:"acid_slime_s_opening_tackle", species:"AcidSlimeS", threshold:17,
        pattern:IntentPattern::OpeningIs("acid_tackle"),
        lane_locator:"m32-source-exordium-v1.md:24,27; S/exordium/AcidSlime_S.java:57-58,82-111",
        source_rule:"At A17+, the first move is Lick; the A0 opening may be Tackle." },
    IntentRule { id:"acid_slime_m_lick_lick", species:"AcidSlimeM", threshold:17,
        pattern:IntentPattern::Sequence(&[PatternToken::Intent("acid_lick"),PatternToken::Intent("acid_lick")]),
        lane_locator:"m32-source-exordium-v1.md:37,40; S/exordium/AcidSlime_M.java:76,87-103",
        source_rule:"At A17+, the high-band guard bars Lick after one Lick; A0 checks two." },
    IntentRule { id:"acid_slime_l_lick_lick", species:"AcidSlimeL", threshold:17,
        pattern:IntentPattern::Sequence(&[PatternToken::Intent("acid_lick"),PatternToken::Intent("acid_lick")]),
        lane_locator:"m32-source-exordium-v1.md:50,53; S/exordium/AcidSlime_L.java:115-135,137-185",
        source_rule:"At A17+, the high-band guard bars Lick after one Lick; A0 checks two." },
    IntentRule { id:"spike_slime_m_lick_lick", species:"SpikeSlimeM", threshold:17,
        pattern:IntentPattern::Sequence(&[PatternToken::Intent("spike_lick"),PatternToken::Intent("spike_lick")]),
        lane_locator:"m32-source-exordium-v1.md:63,66; S/exordium/SpikeSlime_M.java:68-69,80-93",
        source_rule:"At A17+, the high-band guard bars Lick after one Lick; A0 checks two." },
    IntentRule { id:"spike_slime_l_lick_lick", species:"SpikeSlimeL", threshold:17,
        pattern:IntentPattern::Sequence(&[PatternToken::Intent("spike_lick"),PatternToken::Intent("spike_lick")]),
        lane_locator:"m32-source-exordium-v1.md:76,79; S/exordium/SpikeSlime_L.java:96-125,127-171",
        source_rule:"At A17+, the high-band guard bars Lick after one Lick; A0 checks two." },
    IntentRule { id:"red_louse_grow_grow", species:"LouseRed", threshold:17,
        pattern:IntentPattern::Sequence(&[PatternToken::Intent("grow"),PatternToken::Intent("grow")]),
        lane_locator:"m32-source-exordium-v1.md:91,94; S/exordium/LouseNormal.java:67-90,92-121",
        source_rule:"At A17+, the low-band guard bars Grow after one Grow; A0 checks two." },
    IntentRule { id:"green_louse_spit_web_spit_web", species:"LouseGreen", threshold:17,
        pattern:IntentPattern::Sequence(&[PatternToken::Intent("spit_web"),PatternToken::Intent("spit_web")]),
        lane_locator:"m32-source-exordium-v1.md:104,107; S/exordium/LouseDefensive.java:76-99,101-130",
        source_rule:"At A17+, the low-band guard bars Spit Web after one Spit Web; A0 checks two." },
    IntentRule { id:"gremlin_wizard_blast_charge", species:"GremlinWizard", threshold:17,
        pattern:IntentPattern::Sequence(&[PatternToken::Intent("gremlin_blast"),PatternToken::Intent("gremlin_charge")]),
        lane_locator:"m32-source-exordium-v1.md:119,122; S/exordium/GremlinWizard.java:55-79,81-130",
        source_rule:"At A17+, Blast directly sets the next intent to Blast instead of Charge." },
    IntentRule { id:"slaver_red_scrape_scrape", species:"SlaverRed", threshold:17,
        pattern:IntentPattern::Sequence(&[PatternToken::Intent("slaver_scrape"),PatternToken::Intent("slaver_scrape")]),
        lane_locator:"m32-source-exordium-v1.md:145,148; S/exordium/SlaverRed.java:102-158,161-188",
        source_rule:"At A17+, the final guard bars Scrape after one Scrape; A0 checks two." },
    IntentRule { id:"romeo_agonizing_cross_agonizing", species:"BanditLeader", threshold:17,
        pattern:IntentPattern::Sequence(&[PatternToken::Intent("romeo_agonizing_slash"),PatternToken::Intent("romeo_cross_slash"),PatternToken::Intent("romeo_agonizing_slash")]),
        lane_locator:"m32-source-city-v1.md:22,28; BanditLeader.java:59-89,106-140,167-169",
        source_rule:"At A17+, Cross repeats unless the prior two moves are both Cross; A0 returns to Agonizing." },
    IntentRule { id:"chosen_opening_poke", species:"Chosen", threshold:17,
        pattern:IntentPattern::OpeningIs("chosen_poke"),
        lane_locator:"m32-source-city-v1.md:36,42; Chosen.java:73-92,98-118",
        source_rule:"At A17+, Chosen opens with Hex instead of Poke." },
    IntentRule { id:"mystic_attack_attack", species:"Mystic", threshold:17,
        pattern:IntentPattern::Sequence(&[PatternToken::Intent("mystic_attack"),PatternToken::Intent("mystic_attack")]),
        lane_locator:"m32-source-city-v1.md:50,56; Healer.java:65-96,105-144",
        source_rule:"At A17+, Attack requires the immediately prior move not to be Attack; A0 checks two." },
    IntentRule { id:"shelled_parasite_opening_not_fell", species:"ShelledParasite", threshold:17,
        pattern:IntentPattern::OpeningNot("shelled_parasite_fell"),
        lane_locator:"m32-source-city-v1.md:64,70; ShelledParasite.java:81-124,132-168",
        source_rule:"At A17+, the first move is always Fell; A0 chooses Double Strike or Life Suck." },
    IntentRule { id:"snake_plant_spores_x_spores", species:"SnakePlant", threshold:17,
        pattern:IntentPattern::Sequence(&[PatternToken::Intent("snake_plant_enfeebling_spores"),PatternToken::Any,PatternToken::Intent("snake_plant_enfeebling_spores")]),
        lane_locator:"m32-source-city-v1.md:78,84; SnakePlant.java:70-95,118-165",
        source_rule:"At A17+, Spores is barred when it was either of the previous two moves." },
    IntentRule { id:"automaton_beam_stunned", species:"BronzeAutomaton", threshold:19,
        pattern:IntentPattern::Sequence(&[PatternToken::Intent("automaton_beam"),PatternToken::Intent("automaton_stun")]),
        lane_locator:"m32-source-city-v1.md:148,154; BronzeAutomaton.java:75-96,98-124",
        source_rule:"At A19+, Hyper Beam is followed by Boost instead of Stunned." },
];

const ALLOWED_ONLY_INTENT_RULES: &[IntentRule] = &[
    IntentRule { id:"gremlin_wizard_blast_blast", species:"GremlinWizard", threshold:17,
        pattern:IntentPattern::Sequence(&[PatternToken::Intent("gremlin_blast"),PatternToken::Intent("gremlin_blast")]),
        lane_locator:"m32-source-exordium-v1.md:119,122; S/exordium/GremlinWizard.java:55-79,81-130",
        source_rule:"At A17+, Blast directly sets another Blast; A0 sets Charge." },
    IntentRule { id:"romeo_cross_cross", species:"BanditLeader", threshold:17,
        pattern:IntentPattern::Sequence(&[PatternToken::Intent("romeo_cross_slash"),PatternToken::Intent("romeo_cross_slash")]),
        lane_locator:"m32-source-city-v1.md:22,28; BanditLeader.java:59-89,106-140,167-169",
        source_rule:"At A17+, a Cross Slash can be followed by another Cross Slash." },
    IntentRule { id:"automaton_beam_boost", species:"BronzeAutomaton", threshold:19,
        pattern:IntentPattern::Sequence(&[PatternToken::Intent("automaton_beam"),PatternToken::Intent("automaton_boost")]),
        lane_locator:"m32-source-city-v1.md:148,154; BronzeAutomaton.java:75-96,98-124",
        source_rule:"At A19+, Hyper Beam is followed by Boost instead of Stunned." },
];

#[derive(Clone, Debug, PartialEq, Eq)]
struct TranscriptEnemy {
    instance_id: String,
    species: String,
    intent: String,
    hp: i32,
    half_dead: bool,
    escaped: bool,
    replaced: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct TranscriptTurn { combat_id: u32, turn: u32, enemies: Vec<TranscriptEnemy> }
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct SyntheticTranscript { turns: Vec<TranscriptTurn> }

#[derive(Default)]
struct M32IntentFacts {
    enemy_intents: BTreeMap<String, BTreeMap<String, u64>>,
    opening_intents: BTreeMap<String, BTreeMap<String, u64>>,
    intent_transitions: BTreeMap<String, BTreeMap<String, u64>>,
    forbidden_sequences: BTreeMap<String, u64>,
    allowed_only_sequences: BTreeMap<String, u64>,
}

fn detect_m32_intents(transcript: &SyntheticTranscript) -> M32IntentFacts {
    let mut facts = M32IntentFacts::default();
    for rule in FORBIDDEN_INTENT_RULES { facts.forbidden_sequences.insert(rule.id.into(), 0); }
    for rule in ALLOWED_ONLY_INTENT_RULES { facts.allowed_only_sequences.insert(rule.id.into(), 0); }
    let (mut combat_id, mut seen, mut sampled_turn, mut chains) =
        (None, BTreeSet::<String>::new(), BTreeMap::<String, u32>::new(), BTreeMap::<String, Vec<String>>::new());
    for sample in &transcript.turns {
        if combat_id != Some(sample.combat_id) {
            combat_id = Some(sample.combat_id);
            seen.clear();
            sampled_turn.clear();
            chains.clear();
        }
        let present: BTreeSet<&str> = sample.enemies.iter().map(|enemy| enemy.instance_id.as_str()).collect();
        let missing: Vec<String> = chains.keys().filter(|id| !present.contains(id.as_str())).cloned().collect();
        for id in missing { chains.remove(&id); }
        for enemy in &sample.enemies {
            let id = enemy.instance_id.clone();
            if enemy.replaced || enemy.escaped || enemy.half_dead || enemy.hp <= 0 || enemy.intent == "hidden" {
                chains.remove(&id);
                seen.insert(id.clone());
                sampled_turn.insert(id, sample.turn);
                continue;
            }
            if sampled_turn.get(&id) == Some(&sample.turn) { continue; }
            sampled_turn.insert(id.clone(), sample.turn);
            *facts.enemy_intents.entry(enemy.species.clone()).or_default().entry(enemy.intent.clone()).or_default() += 1;
            let opening = seen.insert(id.clone());
            if opening {
                *facts.opening_intents.entry(enemy.species.clone()).or_default().entry(enemy.intent.clone()).or_default() += 1;
            }
            let prior = chains.get(&id).and_then(|moves| moves.last()).cloned();
            if let Some(previous) = prior {
                *facts.intent_transitions.entry(enemy.species.clone()).or_default()
                    .entry(format!("{previous} -> {}", enemy.intent)).or_default() += 1;
            }
            let history = chains.entry(id).or_default();
            history.push(enemy.intent.clone());
            if history.len() > 3 { history.remove(0); }
            for rule in FORBIDDEN_INTENT_RULES.iter().chain(ALLOWED_ONLY_INTENT_RULES) {
                if rule.species != enemy.species { continue; }
                let matched = match rule.pattern {
                    IntentPattern::OpeningIs(want) => opening && enemy.intent == want,
                    IntentPattern::OpeningNot(want) => opening && enemy.intent != want,
                    IntentPattern::Sequence(tokens) => {
                        history.len() >= tokens.len() &&
                            history[history.len() - tokens.len()..].iter().zip(tokens).all(|(actual, token)| match token {
                                PatternToken::Any => true,
                                PatternToken::Intent(want) => actual == want,
                            })
                    }
                };
                if matched {
                    if FORBIDDEN_INTENT_RULES.iter().any(|candidate| candidate.id == rule.id) {
                        *facts.forbidden_sequences.entry(rule.id.into()).or_default() += 1;
                    } else {
                        *facts.allowed_only_sequences.entry(rule.id.into()).or_default() += 1;
                    }
                }
            }
        }
    }
    facts
}

fn pattern_json(pattern: IntentPattern) -> Value {
    match pattern {
        IntentPattern::OpeningIs(intent) => json!({"opening_is":intent}),
        IntentPattern::OpeningNot(intent) => json!({"opening_is_not":intent}),
        IntentPattern::Sequence(tokens) => json!(tokens.iter().map(|token| match token {
            PatternToken::Any => "*".to_string(),
            PatternToken::Intent(intent) => (*intent).to_string(),
        }).collect::<Vec<_>>()),
    }
}

fn intent_rules_json(rules: &[IntentRule]) -> Vec<Value> {
    rules.iter().map(|rule| json!({"id":rule.id,"species":rule.species,"threshold":rule.threshold,
        "pattern":pattern_json(rule.pattern),"lane_locator":rule.lane_locator,"source_rule":rule.source_rule})).collect()
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ShopRowFact { step: usize, kind: String, price: Option<u32>, status: String,
    courier_held: bool, membership_held: bool, smiling_mask_held: bool }
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct ShopRowKey { kind: String, price: Option<u32>, status: String,
    courier_held: bool, membership_held: bool, smiling_mask_held: bool }
impl From<&ShopRowFact> for ShopRowKey {
    fn from(row: &ShopRowFact) -> Self { Self { kind:row.kind.clone(), price:row.price, status:row.status.clone(),
        courier_held:row.courier_held, membership_held:row.membership_held, smiling_mask_held:row.smiling_mask_held } }
}

#[derive(Default)]
struct M32Facts {
    first_louse_step: Option<usize>,
    intents: M32IntentFacts,
    shop_rows: Vec<ShopRowFact>,
}
impl M32Facts {
    fn to_json(&self) -> Value {
        let shop_rows: Vec<Value> = self.shop_rows.iter().map(|row| json!({"step":row.step,"kind":row.kind,"price":row.price,
            "status":row.status,"courier_held":row.courier_held,"membership_held":row.membership_held,
            "smiling_mask_held":row.smiling_mask_held})).collect();
        json!({"first_louse_step":self.first_louse_step,"enemy_intents":self.intents.enemy_intents,
            "opening_intents":self.intents.opening_intents,"intent_transitions":self.intents.intent_transitions,
            "forbidden_sequences":self.intents.forbidden_sequences,"allowed_only_sequences":self.intents.allowed_only_sequences,
            "shop_rows":shop_rows})
    }
}

#[derive(Default)]
struct M32Totals {
    runs_with_first_louse: u64,
    first_louse_steps: BTreeMap<String, u64>,
    enemy_intents: BTreeMap<String, BTreeMap<String, u64>>,
    opening_intents: BTreeMap<String, BTreeMap<String, u64>>,
    intent_transitions: BTreeMap<String, BTreeMap<String, u64>>,
    forbidden_sequences: BTreeMap<String, u64>,
    allowed_only_sequences: BTreeMap<String, u64>,
    shop_rows: BTreeMap<ShopRowKey, u64>,
}
impl M32Totals {
    fn add_run(&mut self, facts: &M32Facts) {
        if let Some(step) = facts.first_louse_step {
            self.runs_with_first_louse += 1;
            *self.first_louse_steps.entry(step.to_string()).or_default() += 1;
        }
        let add_nested = |into: &mut BTreeMap<String, BTreeMap<String, u64>>, from: &BTreeMap<String, BTreeMap<String, u64>>| {
            for (species, counts) in from { for (name, count) in counts {
                *into.entry(species.clone()).or_default().entry(name.clone()).or_default() += count;
            }}
        };
        add_nested(&mut self.enemy_intents, &facts.intents.enemy_intents);
        add_nested(&mut self.opening_intents, &facts.intents.opening_intents);
        add_nested(&mut self.intent_transitions, &facts.intents.intent_transitions);
        for (rule, count) in &facts.intents.forbidden_sequences { *self.forbidden_sequences.entry(rule.clone()).or_default() += count; }
        for (rule, count) in &facts.intents.allowed_only_sequences { *self.allowed_only_sequences.entry(rule.clone()).or_default() += count; }
        for row in &facts.shop_rows { *self.shop_rows.entry(ShopRowKey::from(row)).or_default() += 1; }
    }
    fn to_json(&self) -> Value {
        let shop_rows: Vec<Value> = self.shop_rows.iter().map(|(row, count)| json!({"kind":row.kind,"price":row.price,
            "status":row.status,"courier_held":row.courier_held,"membership_held":row.membership_held,
            "smiling_mask_held":row.smiling_mask_held,"count":count})).collect();
        json!({"runs_with_first_louse":self.runs_with_first_louse,"first_louse_steps":self.first_louse_steps,
            "enemy_intents":self.enemy_intents,"opening_intents":self.opening_intents,"intent_transitions":self.intent_transitions,
            "forbidden_sequences":self.forbidden_sequences,"allowed_only_sequences":self.allowed_only_sequences,"shop_rows":shop_rows})
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
        m31: M31Facts::default(), m32: M32Facts::default(), m33: M33Facts::default(), digests: String::new() };
    let mut run = match new_run(character, seed, supplied, ascension) { Ok(run) => run, Err(error) => { out.fault = Some(error); return out } };
    let mut map_act = 0u8;
    let mut policy = u64::from_le_bytes(root(character, seed)[..8].try_into().unwrap()) ^ if reach { 0x5245_4143_48 } else { 0x4D32_3846 };
    let mut room_kind = String::new();
    let mut last_choice: Option<RoomChoiceKind> = None;
    let base_slots = if ascension >= 11 { 2 } else { 3 };
    let mut equip_open = false;
    let mut transcript = SyntheticTranscript::default();
    let (mut active_combat, mut combat_id) = (false, 0u32);
    let mut m33_act3_visit: Option<u64> = None;
    let mut m33_heart_visit: Option<u64> = None;
    let mut m33_act3_ending_seen = false;
    loop {
        let decision = match run.decision() { Ok(decision) => decision, Err(error) => { out.fault = Some(format!("decision at step {}: {error:?}", out.steps)); break } };
        if decision.room.as_ref().is_some_and(|room| room.kind == RoomViewKind::Act3Ending) { m33_act3_ending_seen = true; }
        if let Some(map) = run.public_map() {
            if map.act == 4 {
                out.m33.act4_rest_floor = map.nodes.iter().find(|node| node.kind == MapNodeKind::Rest).map(|node| node.floor);
            }
        }
        if decision.act == 4 && decision.room.as_ref().is_some_and(|room| room.kind == RoomViewKind::Combat)
            && run.public_map().is_some_and(|map| map.act == 4 && map.current.and_then(|id| map.nodes.iter().find(|node| node.id == id)).is_some_and(|node| node.kind == MapNodeKind::Boss)) {
            out.m33.heart_entered = true;
        }
        if decision.combat.is_some() {
            let visit = decision.combat_progress.entered.saturating_sub(1);
            let current_is_boss = run.public_map().is_some_and(|map| map.act == decision.act
                && map.current.and_then(|id| map.nodes.iter().find(|node| node.id == id)).is_some_and(|node| node.kind == MapNodeKind::Boss));
            if decision.act == 3 && current_is_boss && m33_act3_visit != Some(visit) {
                if !m33_act3_ending_seen && out.m33.act3_boss_fights.iter().any(|fight| fight.won == Some(true)) {
                    out.m33.second_act3_boss_entered = true;
                }
                out.m33.act3_boss_fights.push(M33BossFight { floor:decision.floor, encounter:None, won:None });
                m33_act3_visit = Some(visit);
            }
            if decision.act == 4 && current_is_boss && m33_heart_visit != Some(visit) { m33_heart_visit = Some(visit); }
        } else {
            if let Some(visit) = m33_act3_visit.take() {
                if let Some(record) = decision.last_combat.as_ref().filter(|record| record.visit == visit) {
                    if let Some(fight) = out.m33.act3_boss_fights.last_mut() {
                        fight.encounter = Some(format!("{:?}", record.encounter));
                        fight.won = Some(record.outcome == spire_core::combat::Outcome::Victory);
                    }
                }
            }
            if let Some(visit) = m33_heart_visit.take() {
                if decision.last_combat.as_ref().is_some_and(|record| record.visit == visit
                    && record.encounter == EncounterId::CorruptHeart && record.outcome == spire_core::combat::Outcome::Victory) {
                    out.m33.heart_won = true;
                }
            }
        }
        if decision.terminal == Some(spire_core::run::RunTerminal::HeartVictory) { out.m33.heart_won = true; }
        census.observe(&decision);
        if let Some(combat) = &decision.combat {
            if !active_combat { combat_id += 1; active_combat = true; }
            if out.m32.first_louse_step.is_none() && combat.enemies.iter().any(|enemy|
                matches!(enemy.enemy_id, spire_core::content::EnemyId::LouseRed | spire_core::content::EnemyId::LouseGreen)) {
                out.m32.first_louse_step = Some(out.steps);
            }
            transcript.turns.push(TranscriptTurn { combat_id, turn:combat.turn, enemies:combat.enemies.iter().map(|enemy| TranscriptEnemy {
                instance_id:enemy.instance_id.as_str().to_owned(), species:format!("{:?}", enemy.enemy_id),
                intent:enemy.intent.as_str().to_owned(), hp:enemy.hp, half_dead:enemy.half_dead, escaped:enemy.escaped,
                replaced:enemy.replaced,
            }).collect() });
        } else { active_combat = false; }
        if decision.room.as_ref().is_some_and(|room| room.kind == RoomViewKind::Shop) {
            let (courier_held, membership_held, smiling_mask_held) = (
                decision.player.relics.contains(&RelicId::TheCourier),
                decision.player.relics.contains(&RelicId::MembershipCard),
                decision.player.relics.contains(&RelicId::SmilingMask),
            );
            if let Some(room) = &decision.room {
                for row in &room.choices { out.m32.shop_rows.push(ShopRowFact {
                    step:out.steps, kind:format!("{:?}", row.kind), price:row.price, status:format!("{:?}", row.status),
                    courier_held, membership_held, smiling_mask_held,
                }); }
            }
        }
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
    out.m32.intents = detect_m32_intents(&transcript);
    if let Some(terminal) = &out.terminal { *census.terminals.entry(terminal.clone()).or_default() += 1; }
    *census.acts.entry(out.act.to_string()).or_default() += 1;
    out
}

/// One step's decision digest (review n1): the first 48 bits of sha256 over the step, the public
/// decision and the chosen action (none at a terminal), as 12 hex digits.
fn digest(step: usize, decision: &PublicDecision, action: Option<&GameAction>) -> String {
    let mut text = format!("{decision:?}");
    if env::var("M28_DIGEST_PROJECTION").as_deref() == Ok("pre-m34a") { text = pre_m34a(&text); }
    let hash = Sha256::digest(format!("{step}|{text}|{action:?}").as_bytes());
    hash[..6].iter().map(|b| format!("{b:02x}")).collect()
}

/// M34a: the decision text without the M34a public-view fields (enemy intent category, planned hits
/// and Invincible cap; bottled-card links; room params), so a base tree and an M34a tree running this
/// same example digest the same projection (`M28_DIGEST_PROJECTION=pre-m34a`). Text-only, so it
/// also builds on a pre-M34a tree, where it changes nothing.
fn pre_m34a(text: &str) -> String {
    let mut out = text.to_owned();
    for (key, trailing) in [(", intent_category: ", false), (", intent_hits: ", false), (", invincible_cap: ", false),
        ("bottled_cards: ", true), (", room_params: ", false)] {
        while let Some(start) = out.find(key) {
            let bytes = out.as_bytes();
            let (mut end, mut depth) = (start + key.len(), 0i32);
            while end < bytes.len() {
                match bytes[end] {
                    b'(' | b'[' | b'{' => depth += 1,
                    b')' | b']' | b'}' if depth == 0 => break,
                    b')' | b']' | b'}' => depth -= 1,
                    b',' if depth == 0 => break,
                    _ => {}
                }
                end += 1;
            }
            // A last field's value ends before the closing ` }`; keep that space.
            while end > start + key.len() && bytes[end - 1] == b' ' { end -= 1; }
            if trailing && out[end..].starts_with(", ") { end += 2; }
            out.replace_range(start..end, "");
        }
    }
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
    let results: Vec<(Value, Value, Value, Value, String, Census, M32Facts, M33Facts, Value)> = std::thread::scope(|scope| {
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
            let m33 = json!({"policy":policy,"character":format!("{character:?}"),"seed":seed,"ascension":ascension,"facts":outcome.m33.to_json()});
            eprintln!("{record} {m29} {m30} {m31} {m33}");
            (record, m29, m30, m31, digests, census, outcome.m32, outcome.m33, m33)
        })).collect();
        handles.into_iter().map(|handle| handle.join().expect("traversal thread")).collect()
    });
    let (mut runs, mut faults, mut total, mut per_character) = (Vec::new(), Vec::new(), Census::default(), BTreeMap::<String, Census>::new());
    let (mut m29_runs, mut blocked, mut rejections, mut interim_by_context) = (Vec::new(), 0u64, 0u64, BTreeMap::<String, u64>::new());
    let mut elite_census = BTreeMap::<String, [u64; 3]>::new();
    let (mut m30_runs, mut m30_totals) = (Vec::new(), BTreeMap::<String, u64>::new());
    let (mut m31_runs, mut m31_totals, mut digest_lines) = (Vec::new(), BTreeMap::<String, u64>::new(), Vec::new());
    let (mut m32_runs, mut m32_totals_by_mode) = (Vec::new(), BTreeMap::<String, M32Totals>::new());
    let (mut m33_runs, mut m33_totals_by_mode) = (Vec::new(), BTreeMap::<String, M33Totals>::new());
    for ((policy, character, seed), (record, m29, m30, m31, digests, census, m32, m33, m33_run)) in jobs.iter().zip(results) {
        m32_totals_by_mode.entry((*policy).to_string()).or_default().add_run(&m32);
        m32_runs.push(json!({"policy":policy,"character":format!("{character:?}"),"seed":seed,"ascension":ascension,"facts":m32.to_json()}));
        m33_totals_by_mode.entry((*policy).to_string()).or_default().add_run(&m33);
        m33_runs.push(m33_run);
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
    let m32_totals_by_mode: BTreeMap<String, Value> = m32_totals_by_mode.iter().map(|(mode, totals)| (mode.clone(), totals.to_json())).collect();
    let m33_totals_by_mode: BTreeMap<String, Value> = m33_totals_by_mode.iter().map(|(mode, totals)| (mode.clone(), totals.to_json())).collect();
    let report = json!({"schema":"spire-gym.m28-traversal.v1","seeds_per_character":seeds,"cap":cap,"policies":{"uniform":"seeded uniform random over advertised candidates","supplied":"reach policy on a Standard Run with supplied max HP 400 and, on even seeds, all three keys (m28-diagnostics; grade Supplied, not traversal)","reach":"combat: each candidate valued by a greedy play-out of the turn (survive, then kill); out of combat: one-step current-HP value; greedy on 90% of combat and 50% of other decisions, else uniform"},
        "runs":runs.len(),"fault_free":runs.len() - faults.len(),"faults":faults,"census":total.to_json(),"census_by_character":per_character,"results":runs,
        "m29":{"ascension":ascension,"interim_blocked_runs":blocked,"interim_rejections":rejections,"interim_by_context":interim_by_context,
            "elites_by_act":elite_census,"runs":m29_runs},
        "m30":{"ascension":ascension,"totals":m30_totals,"runs":m30_runs},
        "m31":{"ascension":ascension,"totals":m31_totals,"runs":m31_runs},
        "m32":{"schema":"spire-gym.m32-traversal.v1","ascension":ascension,"forbidden_rules":intent_rules_json(FORBIDDEN_INTENT_RULES),
            "allowed_only_at_a_ge_t":intent_rules_json(ALLOWED_ONLY_INTENT_RULES),"totals_by_mode":m32_totals_by_mode,"runs":m32_runs},
        "m33":{"schema":"spire-gym.m33-traversal.v1","ascension":ascension,"totals_by_mode":m33_totals_by_mode,"runs":m33_runs}});
    fs::write(dir.join("digests.jsonl"), digest_lines.join("\n") + "\n").expect("write digests.jsonl");
    fs::write(dir.join("traversal.json"), serde_json::to_vec_pretty(&report).unwrap()).expect("write traversal.json");
    println!("A{ascension}: {} of {} runs fault-free ({blocked} interim-blocked)", runs.len() - faults.len(), runs.len());
    faults.is_empty()
}

#[cfg(test)]
mod m32_detector_tests {
    use super::*;

    fn enemy(instance_id: &str, species: &str, intent: &str) -> TranscriptEnemy {
        TranscriptEnemy { instance_id:instance_id.into(), species:species.into(), intent:intent.into(),
            hp:20, half_dead:false, escaped:false, replaced:false }
    }
    fn turn(combat_id: u32, turn: u32, enemies: Vec<TranscriptEnemy>) -> TranscriptTurn {
        TranscriptTurn { combat_id, turn, enemies }
    }
    fn count(map: &BTreeMap<String, BTreeMap<String, u64>>, species: &str, key: &str) -> u64 {
        map.get(species).and_then(|counts| counts.get(key)).copied().unwrap_or(0)
    }

    #[test]
    fn m32_detector_counts_equal_samples_on_consecutive_turns() {
        let lick = enemy("slime-m", "AcidSlimeM", "acid_lick");
        let transcript = SyntheticTranscript { turns:vec![
            turn(1, 1, vec![lick.clone()]),
            turn(1, 1, vec![lick.clone()]),
            turn(1, 2, vec![lick]),
        ]};
        let facts = detect_m32_intents(&transcript);
        assert_eq!(count(&facts.enemy_intents, "AcidSlimeM", "acid_lick"), 2);
        assert_eq!(count(&facts.intent_transitions, "AcidSlimeM", "acid_lick -> acid_lick"), 1);
        assert_eq!(facts.forbidden_sequences["acid_slime_m_lick_lick"], 1);
    }

    #[test]
    fn m32_detector_does_not_cross_link_two_instances_of_one_species() {
        let transcript = SyntheticTranscript { turns:vec![
            turn(1, 1, vec![enemy("left", "AcidSlimeM", "acid_lick"), enemy("right", "AcidSlimeM", "acid_spit")]),
            turn(1, 2, vec![enemy("left", "AcidSlimeM", "acid_spit"), enemy("right", "AcidSlimeM", "acid_lick")]),
        ]};
        let facts = detect_m32_intents(&transcript);
        assert_eq!(facts.forbidden_sequences["acid_slime_m_lick_lick"], 0);
        assert_eq!(count(&facts.intent_transitions, "AcidSlimeM", "acid_lick -> acid_lick"), 0);
    }

    #[test]
    fn m32_detector_hidden_intent_breaks_the_chain() {
        let transcript = SyntheticTranscript { turns:vec![
            turn(1, 1, vec![enemy("red", "LouseRed", "grow")]),
            turn(1, 2, vec![enemy("red", "LouseRed", "hidden")]),
            turn(1, 3, vec![enemy("red", "LouseRed", "grow")]),
        ]};
        let facts = detect_m32_intents(&transcript);
        assert_eq!(facts.forbidden_sequences["red_louse_grow_grow"], 0);
        assert_eq!(count(&facts.intent_transitions, "LouseRed", "grow -> grow"), 0);
        assert_eq!(count(&facts.enemy_intents, "LouseRed", "grow"), 2);
        assert_eq!(count(&facts.opening_intents, "LouseRed", "grow"), 1);
    }

    #[test]
    fn m32_detector_spawned_split_child_gets_its_own_opening() {
        let transcript = SyntheticTranscript { turns:vec![
            turn(1, 1, vec![enemy("large", "AcidSlimeL", "acid_tackle")]),
            turn(1, 2, vec![enemy("large", "AcidSlimeL", "acid_lick"), enemy("split-child", "AcidSlimeM", "acid_lick")]),
        ]};
        let facts = detect_m32_intents(&transcript);
        assert_eq!(count(&facts.opening_intents, "AcidSlimeM", "acid_lick"), 1);
        assert_eq!(count(&facts.intent_transitions, "AcidSlimeM", "acid_lick -> acid_lick"), 0);
    }

    #[test]
    fn m32_detector_matches_snake_plant_and_romeo_three_step_patterns() {
        let transcript = SyntheticTranscript { turns:vec![
            turn(1, 1, vec![enemy("plant", "SnakePlant", "snake_plant_enfeebling_spores"),
                enemy("romeo", "BanditLeader", "romeo_agonizing_slash")]),
            turn(1, 2, vec![enemy("plant", "SnakePlant", "snake_plant_chomp_chomp"),
                enemy("romeo", "BanditLeader", "romeo_cross_slash")]),
            turn(1, 3, vec![enemy("plant", "SnakePlant", "snake_plant_enfeebling_spores"),
                enemy("romeo", "BanditLeader", "romeo_agonizing_slash")]),
        ]};
        let facts = detect_m32_intents(&transcript);
        assert_eq!(facts.forbidden_sequences["snake_plant_spores_x_spores"], 1);
        assert_eq!(facts.forbidden_sequences["romeo_agonizing_cross_agonizing"], 1);
    }

    #[test]
    fn m32_detector_synthetic_literals_cover_every_forbidden_rule() {
        for rule in FORBIDDEN_INTENT_RULES {
            let intents: Vec<String> = match rule.pattern {
                IntentPattern::OpeningIs(intent) => vec![intent.into()],
                IntentPattern::OpeningNot(intent) => vec![format!("not-{intent}")],
                IntentPattern::Sequence(tokens) => tokens.iter().map(|token| match token {
                    PatternToken::Any => "synthetic_x".into(),
                    PatternToken::Intent(intent) => (*intent).into(),
                }).collect(),
            };
            let transcript = SyntheticTranscript { turns:intents.iter().enumerate().map(|(i, intent)|
                turn(1, i as u32 + 1, vec![enemy("one", rule.species, intent)])).collect() };
            let facts = detect_m32_intents(&transcript);
            assert_eq!(facts.forbidden_sequences[rule.id], 1, "{}", rule.id);
        }
    }
}

#[cfg(test)]
mod m34a_projection_tests {
    use super::pre_m34a;

    /// The traversal's local projection must equal pre-M34a Debug text byte for byte, including the
    /// ` }` after a removed last field (literals only, so the example still builds on a base tree).
    #[test]
    fn pre_m34a_removes_first_middle_and_last_fields_exactly() {
        for (m34a, base) in [
            ("D { a: 1, room_params: None }", "D { a: 1 }"),
            ("D { a: 1, room_params: Some(RoomParams { stage: [Some(2), None, None, None], rows: [(ChoiceId(1), [Some(-3), None, None])] }) }",
             "D { a: 1 }"),
            ("P { bottled_cards: [(BottledFlame, 10)], relic_counters: [] }", "P { relic_counters: [] }"),
            ("[E { logical_slot: 0, intent_category: Some(Attack), intent_hits: Some(2), invincible_cap: Some(300) }, \
              E { logical_slot: 1, intent_category: None, intent_hits: None, invincible_cap: None }]",
             "[E { logical_slot: 0 }, E { logical_slot: 1 }]"),
            ("X { player: P { bottled_cards: [], hp: 80 }, combat: None, room_params: None }",
             "X { player: P { hp: 80 }, combat: None }"),
        ] {
            assert_eq!(pre_m34a(m34a), base, "{m34a}");
            assert_eq!(pre_m34a(base), base, "a pre-M34a text is unchanged");
        }
    }
}
