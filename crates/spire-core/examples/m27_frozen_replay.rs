use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use spire_core::{
    combat::{Combat, CombatAction, Stance, TargetId},
    content::EncounterId,
    run::{diagnostics::*, pre_m34a_debug, ChoiceId, GameAction, NodeId, Run, SelectionId},
};
use std::{
    env,
    fs::{self, File},
    io::{BufWriter, Write},
    path::{Path, PathBuf},
    process::{Child, ChildStdin, Command, Stdio},
};

#[path = "m13_support/mod.rs"]
#[allow(dead_code)]
mod m13_support;
#[path = "m14_support/mod.rs"]
#[allow(dead_code)]
mod m14_support;
#[path = "m15_support/mod.rs"]
#[allow(dead_code)]
mod m15_support;
#[path = "m20_support/mod.rs"]
#[allow(dead_code)]
mod m20_support;
#[path = "m22_support/mod.rs"]
#[allow(dead_code)]
mod m22_support;
#[path = "m23_support/mod.rs"]
#[allow(dead_code)]
mod m23_support;
#[path = "m24_support/mod.rs"]
#[allow(dead_code)]
mod m24_support;
#[path = "m25_support/mod.rs"]
#[allow(dead_code)]
mod m25_support;

fn sha256(text: &str) -> String {
    format!("{:x}", Sha256::digest(text.as_bytes()))
}

fn get_str<'a>(value: &'a Value, key: &str) -> Result<&'a str, String> {
    value.get(key).and_then(Value::as_str).ok_or_else(|| format!("missing string {key}"))
}

fn encounter(name: &str) -> Result<EncounterId, String> {
    use EncounterId as E;
    match name {
        "Guardian" | "guardian" => Ok(E::Guardian),
        "Hexaghost" | "hexaghost" => Ok(E::Hexaghost),
        "SlimeBoss" | "slime_boss" | "slime-boss" | "slime" => Ok(E::SlimeBoss),
        "Champ" | "champ" => Ok(E::Champ),
        "BronzeAutomaton" | "bronze_automaton" | "bronze-automaton" => Ok(E::BronzeAutomaton),
        "Collector" | "collector" => Ok(E::Collector),
        "AwakenedOne" | "awakened_one" | "awakened-one" => Ok(E::AwakenedOne),
        "TimeEater" | "time_eater" | "time-eater" => Ok(E::TimeEater),
        "DonuAndDeca" | "donu_and_deca" | "donu-and-deca" => Ok(E::DonuAndDeca),
        _ => Err(format!("unsupported recorded boss {name}")),
    }
}

fn scenario_boss(scenario: &Value, key: &str) -> Result<EncounterId, String> {
    encounter(get_str(scenario, key)?)
}

fn m22_boss(value: EncounterId) -> Result<M22DiagnosticBoss, String> {
    match value {
        EncounterId::Guardian => Ok(M22DiagnosticBoss::Guardian),
        EncounterId::Hexaghost => Ok(M22DiagnosticBoss::Hexaghost),
        EncounterId::SlimeBoss => Ok(M22DiagnosticBoss::SlimeBoss),
        _ => Err(format!("unsupported M22 boss {value:?}")),
    }
}

fn m23_boss(value: EncounterId) -> Result<M23DiagnosticBoss, String> {
    match value {
        EncounterId::Champ => Ok(M23DiagnosticBoss::Champ),
        EncounterId::BronzeAutomaton => Ok(M23DiagnosticBoss::BronzeAutomaton),
        EncounterId::Collector => Ok(M23DiagnosticBoss::Collector),
        _ => Err(format!("unsupported M23 boss {value:?}")),
    }
}

fn m24_boss(value: EncounterId) -> Result<M24DiagnosticBoss, String> {
    match value {
        EncounterId::AwakenedOne => Ok(M24DiagnosticBoss::AwakenedOne),
        EncounterId::TimeEater => Ok(M24DiagnosticBoss::TimeEater),
        EncounterId::DonuAndDeca => Ok(M24DiagnosticBoss::DonuAndDeca),
        _ => Err(format!("unsupported M24 boss {value:?}")),
    }
}

fn m25_boss(value: EncounterId) -> Result<M25DiagnosticBoss, String> {
    match value {
        EncounterId::AwakenedOne => Ok(M25DiagnosticBoss::AwakenedOne),
        EncounterId::TimeEater => Ok(M25DiagnosticBoss::TimeEater),
        EncounterId::DonuAndDeca => Ok(M25DiagnosticBoss::DonuAndDeca),
        _ => Err(format!("unsupported M25 boss {value:?}")),
    }
}

fn m24_scenario(value: EncounterId) -> Result<m24_support::Scenario, String> {
    match value {
        EncounterId::AwakenedOne => Ok(m24_support::Scenario::AwakenedOne),
        EncounterId::TimeEater => Ok(m24_support::Scenario::TimeEater),
        EncounterId::DonuAndDeca => Ok(m24_support::Scenario::DonuAndDeca),
        _ => Err(format!("unsupported M24 scenario {value:?}")),
    }
}

fn m25_scenario(value: EncounterId) -> Result<m25_support::Scenario, String> {
    match value {
        EncounterId::AwakenedOne => Ok(m25_support::Scenario::AwakenedOne),
        EncounterId::TimeEater => Ok(m25_support::Scenario::TimeEater),
        EncounterId::DonuAndDeca => Ok(m25_support::Scenario::DonuAndDeca),
        _ => Err(format!("unsupported M25 scenario {value:?}")),
    }
}

fn m23_fixture(value: &str) -> Result<M23PlayedFixture, String> {
    use M23PlayedFixture as F;
    match value {
        "flight-three-byrds" => Ok(F::FlightThreeByrds),
        "prices-snecko" => Ok(F::PricesSnecko),
        "power-scry-chosen-and-byrds" => Ok(F::PowerScryChosenAndByrds),
        "wounds-slavers" => Ok(F::WoundsSlavers),
        "wounds-book-of-stabbing" => Ok(F::WoundsBookOfStabbing),
        "stasis-bronze-automaton" => Ok(F::StasisBronzeAutomaton),
        _ => Err(format!("unknown recorded M23 fixture {value}")),
    }
}

fn m24_fixture(value: &str) -> Result<M24PlayedFixture, String> {
    use M24PlayedFixture as F;
    match value {
        "slow-giant-head" => Ok(F::SlowGiantHead),
        "intangible-nemesis" => Ok(F::IntangibleNemesis),
        "daggers-reptomancer" => Ok(F::DaggersReptomancer),
        "implant-writhing-mass" => Ok(F::ImplantWrithingMass),
        "half-dead-darklings" => Ok(F::HalfDeadDarklings),
        "burn-orb-walker" => Ok(F::BurnOrbWalker),
        "constrict-spire-growth" => Ok(F::ConstrictSpireGrowth),
        "thorns-four-shapes" => Ok(F::ThornsFourShapes),
        "fading-transient" => Ok(F::FadingTransient),
        _ => Err(format!("unknown recorded M24 fixture {value}")),
    }
}

fn m25_fixture(value: &str) -> Result<M25PlayedFixture, String> {
    use M25PlayedFixture as F;
    match value {
        "x1-facing-shield-spear" => Ok(F::FacingShieldSpear),
        "x2-burns-shield-spear" => Ok(F::BurnsShieldSpear),
        "x3-generation-shield-spear" => Ok(F::GenerationShieldSpear),
        "x4-powers-shield-spear" => Ok(F::PowersShieldSpear),
        "x5-beat-heart" => Ok(F::BeatHeart),
        "x6-invincible-heart" => Ok(F::InvincibleHeart),
        "x7-statuses-heart" => Ok(F::StatusesHeart),
        "x8-turn-control-heart" => Ok(F::TurnControlHeart),
        _ => Err(format!("unknown recorded M25 fixture {value}")),
    }
}

fn m25_loadout(value: &str) -> Result<M25DiagnosticLoadout, String> {
    match value {
        "coverage-a" => Ok(M25DiagnosticLoadout::CoverageA),
        "coverage-b" => Ok(M25DiagnosticLoadout::CoverageB),
        "coverage-c" => Ok(M25DiagnosticLoadout::CoverageC),
        "strong-v1" => Ok(M25DiagnosticLoadout::Strong),
        _ => Err(format!("unknown recorded M25 loadout {value}")),
    }
}

fn constructed_run(input: &Value, scenario: &Value) -> Result<Run, String> {
    let milestone = get_str(input, "milestone")?;
    let kind = get_str(input, "kind")?;
    let seed = input.get("seed").and_then(Value::as_u64).ok_or("missing seed")?;
    let scenario_text = scenario;
    match milestone {
        "M13" => {
            let s = m13_support::Scenario {
                act1_boss: scenario_boss(scenario_text, "act1_boss")?,
                act2_boss: scenario_boss(scenario_text, "act2_boss")?,
            };
            m13_support::make_run(seed, s)
        }
        "M14" => {
            let s = m14_support::Scenario {
                act1_boss: scenario_boss(scenario_text, "act1_boss")?,
                act2_boss: scenario_boss(scenario_text, "act2_boss")?,
                act3_boss: scenario_boss(scenario_text, "act3_boss")?,
            };
            if kind == "supplied" {
                let run = m14_support::make_run(seed, s)?;
                m27_m14_supplied_run(run, s.act3_boss)
            } else {
                m14_support::make_run(seed, s)
            }
        }
        "M15" => {
            let s = m15_support::Scenario {
                act1_boss: scenario_boss(scenario_text, "act1_boss")?,
                act2_boss: scenario_boss(scenario_text, "act2_boss")?,
                act3_boss: scenario_boss(scenario_text, "act3_boss")?,
            };
            if kind == "supplied" {
                let run = m15_support::make_run(seed, s)?;
                m27_m15_supplied_run(run)
            } else {
                m15_support::make_run(seed, s)
            }
        }
        "M20" => {
            if scenario_text.is_object()
                && input.pointer("/provenance/kind").and_then(Value::as_str) == Some("supplied")
            {
                return m20_supplied_act4(M20DiagnosticLoadout::Defect)
                    .map_err(|error| format!("{error:?}"));
            }
            let s = m20_support::Scenario {
                act1_boss: scenario_boss(scenario_text, "act1_boss")?,
                act2_boss: scenario_boss(scenario_text, "act2_boss")?,
                act3_boss: scenario_boss(scenario_text, "act3_boss")?,
            };
            m20_support::make_run(seed, s)
        }
        "M22" => {
            let act1 = m22_boss(scenario_boss(scenario_text, "act1_boss")?)?;
            if scenario_text.get("supplied_boss").is_some() || kind == "supplied" {
                m22_supplied_boss(act1).map_err(|error| format!("{error:?}"))
            } else {
                let s = match act1 {
                    M22DiagnosticBoss::Guardian => m22_support::Scenario::Guardian,
                    M22DiagnosticBoss::Hexaghost => m22_support::Scenario::Hexaghost,
                    M22DiagnosticBoss::SlimeBoss => m22_support::Scenario::SlimeBoss,
                };
                m22_support::make_run(seed, s)
            }
        }
        "M23" => {
            if let Some(fixture) = scenario_text.get("fixture").and_then(Value::as_str) {
                return m23_played_fixture(m23_fixture(fixture)?).map_err(|error| format!("{error:?}"));
            }
            if let Some(boss) = scenario_text.get("supplied_boss").and_then(Value::as_str) {
                return m23_supplied_boss(m23_boss(encounter(boss)?)?).map_err(|error| format!("{error:?}"));
            }
            let target = m23_boss(scenario_boss(scenario_text, "act2_boss")?)?;
            let s = match target {
                M23DiagnosticBoss::Champ => m23_support::Scenario::Champ,
                M23DiagnosticBoss::BronzeAutomaton => m23_support::Scenario::BronzeAutomaton,
                M23DiagnosticBoss::Collector => m23_support::Scenario::Collector,
            };
            m23_support::make_run(seed, s, kind == "control")
        }
        "M24" => {
            if let Some(fixture) = scenario_text.get("fixture").and_then(Value::as_str) {
                return m24_played_fixture(m24_fixture(fixture)?).map_err(|error| format!("{error:?}"));
            }
            if let Some(boss) = scenario_text.get("supplied_traversal").and_then(Value::as_str) {
                return m24_supplied_traversal(m24_boss(encounter(boss)?)?).map_err(|error| format!("{error:?}"));
            }
            if let Some(boss) = scenario_text.get("supplied_boss").and_then(Value::as_str) {
                let keys = scenario_text.get("keys").and_then(Value::as_array).ok_or("M24 supplied boss keys missing")?;
                if keys.len() != 3 { return Err("M24 supplied boss must record three keys".into()); }
                let keys = [keys[0].as_bool().unwrap_or(false), keys[1].as_bool().unwrap_or(false), keys[2].as_bool().unwrap_or(false)];
                return m24_supplied_boss_at(m24_boss(encounter(boss)?)?, keys, seed).map_err(|error| format!("{error:?}"));
            }
            let s = m24_scenario(scenario_boss(scenario_text, "act3_boss")?)?;
            m24_support::make_run(seed, s, kind == "control")
        }
        "M25" => {
            if let Some(fixture) = scenario_text.get("fixture").and_then(Value::as_str) {
                return m25_played_fixture(m25_fixture(fixture)?).map_err(|error| format!("{error:?}"));
            }
            if let Some(boss) = scenario_text.get("supplied_traversal").and_then(Value::as_str) {
                return m25_supplied_traversal(m25_boss(encounter(boss)?)?).map_err(|error| format!("{error:?}"));
            }
            if let Some(loadout) = scenario_text.get("supplied_act4_loadout").and_then(Value::as_str) {
                return m25_supplied_act4(m25_loadout(loadout)?).map_err(|error| format!("{error:?}"));
            }
            let s = m25_scenario(scenario_boss(scenario_text, "act3_boss")?)?;
            m25_support::make_run(seed, s, kind == "control")
        }
        _ => Err(format!("no existing Run constructor route for {milestone}")),
    }
}

// Convert the recorded public verb schema to the same GameAction variants.
fn target(value: &Value) -> Result<Option<TargetId>, String> {
    match value {
        Value::Null => Ok(None),
        Value::Number(number) => number.as_u64().map(|id| Some(TargetId(id))).ok_or("invalid target id".into()),
        _ => Err("invalid target value".into()),
    }
}

fn number(value: &Value, key: &str) -> Result<u64, String> {
    value.get(key).and_then(Value::as_u64).ok_or_else(|| format!("missing action number {key}"))
}

fn structured_action(value: &Value) -> Result<GameAction, String> {
    let verb = get_str(value, "verb")?;
    match verb {
        "choose" => Ok(GameAction::Choose(ChoiceId(number(value, "choice")?))),
        "move" => Ok(GameAction::Move(NodeId(number(value, "node")?))),
        "select" => Ok(GameAction::Select(SelectionId(number(value, "selection")?))),
        "confirm" => Ok(GameAction::Confirm),
        "cancel" => Ok(GameAction::Cancel),
        "use_potion" => Ok(GameAction::UsePotion { slot: number(value, "slot")? as u32, target: target(value.get("target").unwrap_or(&Value::Null))? }),
        "discard_potion" => Ok(GameAction::DiscardPotion { slot: number(value, "slot")? as u32 }),
        "combat.end_turn" => Ok(GameAction::Combat(CombatAction::EndTurn)),
        "combat.confirm_selection" => Ok(GameAction::Combat(CombatAction::ConfirmSelection)),
        "combat.play_card" => Ok(GameAction::Combat(CombatAction::PlayCard { instance: number(value, "instance")?, target: target(value.get("target").unwrap_or(&Value::Null))? })),
        "combat.select_card" => Ok(GameAction::Combat(CombatAction::SelectCard { instance: number(value, "instance")? })),
        "combat.deselect_card" => Ok(GameAction::Combat(CombatAction::DeselectCard { instance: number(value, "instance")? })),
        "combat.use_potion" => Ok(GameAction::Combat(CombatAction::UsePotion { slot: number(value, "slot")? as usize, target: target(value.get("target").unwrap_or(&Value::Null))? })),
        "combat.choose_card_offer" => Ok(GameAction::Combat(CombatAction::ChooseCardOffer { index: number(value, "index")? as u32 })),
        "combat.skip_card_offer" => Ok(GameAction::Combat(CombatAction::SkipCardOffer)),
        "combat.choose_stance" => {
            let stance = match get_str(value, "stance")? {
                "neutral" => Stance::Neutral,
                "wrath" => Stance::Wrath,
                "calm" => Stance::Calm,
                "divinity" => Stance::Divinity,
                other => return Err(format!("unknown stance {other}")),
            };
            Ok(GameAction::Combat(CombatAction::ChooseStance { stance }))
        }
        _ => Err(format!("unsupported recorded action verb {verb}")),
    }
}

fn numeric_after(text: &str, marker: &str) -> Result<u64, String> {
    let start = text.find(marker).ok_or_else(|| format!("missing {marker} in action"))? + marker.len();
    let digits = text[start..].chars().take_while(|char| char.is_ascii_digit()).collect::<String>();
    digits.parse().map_err(|_| format!("invalid integer after {marker}"))
}

fn debug_target(text: &str) -> Result<Option<TargetId>, String> {
    if text.contains("target:None") { return Ok(None); }
    if text.contains("target:Some(TargetId(") {
        return Ok(Some(TargetId(numeric_after(text, "target:Some(TargetId(")?)));
    }
    Err(format!("unrecognized target in action {text}"))
}

fn parse_combat_debug(text: &str) -> Result<GameAction, String> {
    if text == "EndTurn" { return Ok(GameAction::Combat(CombatAction::EndTurn)); }
    if text == "ConfirmSelection" { return Ok(GameAction::Combat(CombatAction::ConfirmSelection)); }
    if text == "SkipCardOffer" { return Ok(GameAction::Combat(CombatAction::SkipCardOffer)); }
    if text.starts_with("PlayCard{") {
        return Ok(GameAction::Combat(CombatAction::PlayCard { instance: numeric_after(text, "instance:")?, target: debug_target(text)? }));
    }
    if text.starts_with("UsePotion{") {
        return Ok(GameAction::Combat(CombatAction::UsePotion { slot: numeric_after(text, "slot:")? as usize, target: debug_target(text)? }));
    }
    if text.starts_with("SelectCard{") {
        return Ok(GameAction::Combat(CombatAction::SelectCard { instance: numeric_after(text, "instance:")? }));
    }
    if text.starts_with("DeselectCard{") {
        return Ok(GameAction::Combat(CombatAction::DeselectCard { instance: numeric_after(text, "instance:")? }));
    }
    if text.starts_with("ChooseCardOffer{") {
        return Ok(GameAction::Combat(CombatAction::ChooseCardOffer { index: numeric_after(text, "index:")? as u32 }));
    }
    if text.starts_with("ChooseStance{") {
        let stance = if text.contains("stance:Neutral") { Stance::Neutral }
            else if text.contains("stance:Wrath") { Stance::Wrath }
            else if text.contains("stance:Calm") { Stance::Calm }
            else if text.contains("stance:Divinity") { Stance::Divinity }
            else { return Err(format!("unknown stance in {text}")); };
        return Ok(GameAction::Combat(CombatAction::ChooseStance { stance }));
    }
    Err(format!("unsupported combat action debug {text}"))
}

fn debug_action(value: &str) -> Result<GameAction, String> {
    let compact = value.chars().filter(|char| !char.is_whitespace()).collect::<String>();
    if compact.starts_with("Choose(ChoiceId(") { return Ok(GameAction::Choose(ChoiceId(numeric_after(&compact, "Choose(ChoiceId(")?))); }
    if compact.starts_with("Move(NodeId(") { return Ok(GameAction::Move(NodeId(numeric_after(&compact, "Move(NodeId(")?))); }
    if compact.starts_with("Select(SelectionId(") { return Ok(GameAction::Select(SelectionId(numeric_after(&compact, "Select(SelectionId(")?))); }
    if compact == "Confirm" { return Ok(GameAction::Confirm); }
    if compact == "Cancel" { return Ok(GameAction::Cancel); }
    if compact.starts_with("Combat(") && compact.ends_with(')') {
        return parse_combat_debug(&compact[7..compact.len() - 1]);
    }
    Err(format!("unsupported action debug {value}"))
}

fn action_value(step: &Value) -> Result<(GameAction, String), String> {
    if let Some(value) = step.get("action") {
        if value.is_object() {
            return Ok((structured_action(value)?, format!("{value}")));
        }
        if let Some(value) = value.as_str() { return Ok((debug_action(value)?, value.to_owned())); }
    }
    let value = get_str(step, "action_debug")?;
    Ok((debug_action(value)?, value.to_owned()))
}

fn expected_attempt_post(step: &Value) -> Option<Result<&str, &str>> {
    let post = step.get("post")?;
    if let Some(value) = post.pointer("/decision/debug").and_then(Value::as_str) {
        return Some(Ok(value));
    }
    if let Some(value) = post.get("decision_debug").and_then(Value::as_str) {
        return Some(Ok(value));
    }
    if let Some(value) = post.get("decision_error").and_then(Value::as_str) {
        return Some(Err(value));
    }
    None
}

fn expected_attempt_pre(step: &Value) -> Option<&str> {
    step.pointer("/pre/decision/debug").and_then(Value::as_str)
        .or_else(|| step.pointer("/pre/decision_debug").and_then(Value::as_str))
}

fn actual_decision(run: &Run, pretty: bool) -> Result<String, String> {
    let decision = run.decision().map_err(|error| format!("{error:?}"))?;
    Ok(pre_m34a_debug(&if pretty { format!("{decision:#?}") } else { format!("{decision:?}") }))
}

fn actual_decision_result(run: &Run, pretty: bool) -> Result<String, String> {
    match run.decision() {
        Ok(decision) => Ok(pre_m34a_debug(&if pretty { format!("{decision:#?}") } else { format!("{decision:?}") })),
        Err(error) => Err(format!("{error:?}")),
    }
}

fn current_endpoint(run: &Run) -> String {
    match run.decision() {
        Ok(decision) => format!("act={} floor={} room={:?} terminal={:?}", decision.act, decision.floor, decision.room.as_ref().map(|room| room.kind), decision.terminal),
        Err(error) => format!("decision-error:{error:?}"),
    }
}

struct GzipSink {
    child: Child,
    stdin: BufWriter<ChildStdin>,
}

impl GzipSink {
    fn create(path: &Path) -> Result<Self, String> {
        let file = File::create(path).map_err(|error| error.to_string())?;
        let mut child = Command::new("gzip").args(["-n", "-c"]).stdin(Stdio::piped()).stdout(Stdio::from(file)).spawn().map_err(|error| format!("gzip spawn failed: {error}"))?;
        let stdin = child.stdin.take().ok_or("gzip stdin unavailable")?;
        Ok(Self { child, stdin: BufWriter::new(stdin) })
    }
    fn write(&mut self, value: &Value) -> Result<(), String> {
        serde_json::to_writer(&mut self.stdin, value).map_err(|error| error.to_string())?;
        self.stdin.write_all(b"\n").map_err(|error| error.to_string())
    }
    fn finish(mut self) -> Result<(), String> {
        self.stdin.flush().map_err(|error| error.to_string())?;
        drop(self.stdin);
        let output = self.child.wait().map_err(|error| error.to_string())?;
        if !output.success() { return Err(format!("gzip exited with {output}")); }
        Ok(())
    }
}

fn write_pair_step(
    combat: &Combat,
    index: usize,
    action: &str,
    jsonl: &mut BufWriter<File>,
    texts: &mut GzipSink,
    g4: &mut Vec<String>,
    input: &Value,
) -> Result<(), String> {
    let state = combat.m27_state_debug();
    let public = combat.public_full().map_err(|error| format!("public_full failed at {index}: {error:?}"))?;
    let public_text = pre_m34a_debug(&format!("{public:?}"));
    let state_hash = sha256(&state);
    let public_hash = sha256(&public_text);
    let line = json!({"step":index,"action":action,"state_sha":state_hash,"public_full_sha":public_hash,"run_state_sha":Value::Null});
    serde_json::to_writer(&mut *jsonl, &line).map_err(|error| error.to_string())?;
    jsonl.write_all(b"\n").map_err(|error| error.to_string())?;
    texts.write(&json!({"step":index,"action":action,"state_debug":state,"public_full_debug":public_text,"run_state_debug":Value::Null}))?;
    if index == 0 {
        let owned = input.get("relics").and_then(Value::as_array).ok_or("missing pair relics")?;
        let has_twisted = owned.iter().any(|id| id.as_str() == Some("TwistedFunnel"));
        let has_marbles = owned.iter().any(|id| id.as_str() == Some("BagOfMarbles"));
        let has_mask = owned.iter().any(|id| id.as_str() == Some("RedMask"));
        if has_twisted && (has_marbles || has_mask) {
            let powers = public.enemies.first().ok_or("G4 pair has no enemy-1")?.powers.iter().map(|power| power.id.0).collect::<Vec<_>>();
            g4.push(format!("{}\t0\t{}\t{}", input["input_id"].as_str().unwrap_or("?"), owned.iter().filter_map(Value::as_str).collect::<Vec<_>>().join(","), powers.join(">")));
        }
    }
    Ok(())
}

fn replay_pair(input: &Value, out: &Path, g4: &mut Vec<String>) -> Result<(usize, String), String> {
    let input_id = get_str(input, "input_id")?;
    let character = match get_str(input, "character")? {
        "Ironclad" => spire_core::content::Character::Ironclad,
        "Silent" => spire_core::content::Character::Silent,
        "Defect" => spire_core::content::Character::Defect,
        "Watcher" => spire_core::content::Character::Watcher,
        other => return Err(format!("unknown pair character {other}")),
    };
    let profile = get_str(input.get("context").ok_or("missing pair context")?, "diagnostic_profile")?;
    let relics = input.get("relics").and_then(Value::as_array).ok_or("missing pair relics")?.iter().map(|value| match value.as_str() {
        Some("Anchor") => Ok(spire_core::content::RelicId::Anchor),
        Some("BagOfMarbles") => Ok(spire_core::content::RelicId::BagOfMarbles),
        Some("BagOfPreparation") => Ok(spire_core::content::RelicId::BagOfPreparation),
        Some("BloodVial") => Ok(spire_core::content::RelicId::BloodVial),
        Some("BronzeScales") => Ok(spire_core::content::RelicId::BronzeScales),
        Some("CrackedCore") => Ok(spire_core::content::RelicId::CrackedCore),
        Some("Damaru") => Ok(spire_core::content::RelicId::Damaru),
        Some("DataDisk") => Ok(spire_core::content::RelicId::DataDisk),
        Some("EmotionChip") => Ok(spire_core::content::RelicId::EmotionChip),
        Some("GoldPlatedCables") => Ok(spire_core::content::RelicId::GoldPlatedCables),
        Some("GoldenEye") => Ok(spire_core::content::RelicId::GoldenEye),
        Some("Inserter") => Ok(spire_core::content::RelicId::Inserter),
        Some("MarkOfPain") => Ok(spire_core::content::RelicId::MarkOfPain),
        Some("NeowsLament") => Ok(spire_core::content::RelicId::NeowsLament),
        Some("OddlySmoothStone") => Ok(spire_core::content::RelicId::OddlySmoothStone),
        Some("PureWater") => Ok(spire_core::content::RelicId::PureWater),
        Some("RedMask") => Ok(spire_core::content::RelicId::RedMask),
        Some("RedSkull") => Ok(spire_core::content::RelicId::RedSkull),
        Some("RingOfTheSnake") => Ok(spire_core::content::RelicId::RingOfTheSnake),
        Some("RunicCapacitor") => Ok(spire_core::content::RelicId::RunicCapacitor),
        Some("RunicCube") => Ok(spire_core::content::RelicId::RunicCube),
        Some("SelfFormingClay") => Ok(spire_core::content::RelicId::SelfFormingClay),
        Some("TwistedFunnel") => Ok(spire_core::content::RelicId::TwistedFunnel),
        Some("Vajra") => Ok(spire_core::content::RelicId::Vajra),
        Some(other) => Err(format!("unknown pair relic {other}")),
        None => Err("non-string pair relic".into()),
    }).collect::<Result<Vec<_>, _>>()?;
    let setup = input.get("setup").ok_or("missing pair setup")?;
    let player = setup.get("player").ok_or("missing pair player")?;
    let hp = player["hp"].as_i64().ok_or("missing pair HP")? as i32;
    let max_hp = player["max_hp"].as_i64().ok_or("missing pair max HP")? as i32;
    let seed_byte = setup["seed_byte"].as_u64().ok_or("missing pair seed byte")? as u8;
    let copies = setup["deck_copies"].as_u64().ok_or("missing pair deck count")? as usize;
    let opening_draw = setup["opening_draw"].as_u64().ok_or("missing pair opening draw")? as usize;
    let pair_enemy = match setup.get("enemy").and_then(Value::as_str).unwrap_or("Cultist") {
        "Cultist" => None,
        "JawWorm" => Some(spire_core::content::EncounterId::JawWorm),
        other => return Err(format!("{input_id}: unsupported pair enemy {other}")),
    };
    let pair_deck_card = match setup.get("deck_card").and_then(Value::as_str).unwrap_or("Strike") {
        "Strike" | "StrikeGreen" | "StrikeBlue" | "StrikePurple" => None,
        "Zap" => Some(spire_core::content::CardId::Zap),
        "Bludgeon" => Some(spire_core::content::CardId::Bludgeon),
        "Foresight" => Some(spire_core::content::CardId::Foresight),
        other => return Err(format!("{input_id}: unsupported pair deck card {other}")),
    };
    let pair_enemy_hp = setup.get("enemy_hp").map(|value| value.as_i64()
        .map(|hp| hp as i32).ok_or_else(|| format!("{input_id}: invalid pair enemy HP"))).transpose()?;
    let mut combat = m27_pair_combat(character, relics, profile, hp, max_hp, seed_byte, copies, opening_draw, pair_enemy, pair_deck_card, pair_enemy_hp).map_err(|error| format!("{input_id}: {error:?}"))?;
    let script = input.get("script").and_then(Value::as_array).ok_or("missing pair script")?;
    let mut jsonl = BufWriter::new(File::create(out.join(format!("{input_id}.jsonl"))).map_err(|error| error.to_string())?);
    let mut texts = GzipSink::create(&out.join(format!("{input_id}.texts.jsonl.gz")))?;
    for (index, step) in script.iter().enumerate() {
        let action = get_str(step, "action")?;
        if index == 0 {
            if action != "enter" { return Err(format!("{input_id}: script must start with entry")); }
        } else {
            match action {
                "EndTurn" => combat.step_full(&CombatAction::EndTurn)
                    .map_err(|error| format!("{input_id}: {action} at {index}: {error:?}"))?,
                "PlayCard" => {
                    let hand_index = usize::try_from(number(step, "hand_index")?)
                        .map_err(|_| format!("{input_id}: hand index is out of range"))?;
                    let instance = combat.public_full().map_err(|error| format!("{input_id}: {error:?}"))?
                        .hand.get(hand_index).map(|card| card.instance.0)
                        .ok_or_else(|| format!("{input_id}: hand index {hand_index} is absent at step {index}"))?;
                    let card_target = target(step.get("target").unwrap_or(&Value::Null))?;
                    combat.step_full(&CombatAction::PlayCard { instance, target: card_target })
                        .map_err(|error| format!("{input_id}: {action} at {index}: {error:?}"))?;
                }
                other => return Err(format!("{input_id}: unsupported pair action {other}")),
            }
        }
        write_pair_step(&combat, index, action, &mut jsonl, &mut texts, g4, input)?;
    }
    let endpoint = format!("{:?}", combat.public_full().map_err(|error| format!("{error:?}"))?.outcome);
    jsonl.flush().map_err(|error| error.to_string())?;
    texts.finish()?;
    Ok((script.len(), endpoint))
}

fn write_run_snapshot(
    run: &Run,
    step_index: i64,
    action: &str,
    kind: &str,
    matched: Option<bool>,
    error: Option<&str>,
    jsonl: &mut BufWriter<File>,
    texts: &mut GzipSink,
    g4: &mut Vec<String>,
    input_id: &str,
    prior_combat: bool,
) -> Result<(), String> {
    let capture = m27_capture_run(run).map_err(|error| format!("run capture failed: {error:?}"))?;
    let state_sha = capture.combat_state_debug.as_deref().map(sha256);
    // M34a: the frozen inputs predate the M34a view fields; record and hash the pre-M34a projection.
    let public_full_debug = capture.public_full_debug.as_deref().map(pre_m34a_debug);
    let public_full_sha = public_full_debug.as_deref().map(sha256);
    let run_state_sha = sha256(&capture.run_state_debug);
    let line = json!({
        "step":step_index,"kind":kind,"action":action,
        "state_sha":state_sha,"public_full_sha":public_full_sha,"run_state_sha":run_state_sha,
        "post_decision_match":matched,"step_error":error,
    });
    serde_json::to_writer(&mut *jsonl, &line).map_err(|error| error.to_string())?;
    jsonl.write_all(b"\n").map_err(|error| error.to_string())?;
    texts.write(&json!({
        "step":step_index,"kind":kind,"action":action,
        "state_debug":capture.combat_state_debug,"public_full_debug":public_full_debug,
        "run_state_debug":capture.run_state_debug,
    }))?;
    let has_funnel = capture.relics.contains(&spire_core::content::RelicId::TwistedFunnel);
    let has_marbles = capture.relics.contains(&spire_core::content::RelicId::BagOfMarbles);
    let has_mask = capture.relics.contains(&spire_core::content::RelicId::RedMask);
    if has_funnel && (has_marbles || has_mask) && capture.combat_state_debug.is_some() && !prior_combat {
        let relics = capture.relics.iter().map(|relic| format!("{relic:?}")).collect::<Vec<_>>().join(",");
        let powers = if capture.enemy1_power_order.is_empty() { "-".to_owned() } else { capture.enemy1_power_order.join(">") };
        g4.push(format!("{input_id}\t{step_index}\t{relics}\t{powers}"));
    }
    Ok(())
}

fn attempt_decision_match(run: &Run, expected: Option<Result<&str, &str>>, pretty: bool) -> Result<bool, String> {
    match expected {
        Some(Ok(expected)) => Ok(actual_decision(run, pretty)? == expected),
        Some(Err(expected)) => Ok(actual_decision_result(run, pretty).err().as_deref() == Some(expected)),
        None => Err("recorded post decision_debug is missing".into()),
    }
}

fn input_steps(record: &Value, payload: &Value) -> Result<Vec<Value>, String> {
    if get_str(record, "file_kind")? == "trace-text" {
        Ok(payload.get("steps").and_then(Value::as_array).ok_or("trace tape steps missing")?.clone())
    } else {
        Ok(payload.get("steps").and_then(Value::as_array).ok_or("attempt steps missing")?.clone())
    }
}

fn replay_external(record: &Value, out: &Path, g4: &mut Vec<String>) -> Result<RunSummary, String> {
    let input_id = get_str(record, "input_id")?;
    let source_path = get_str(record, "path")?;
    let milestone = get_str(record, "milestone")?;
    let is_trace = get_str(record, "file_kind")? == "trace-text";
    let payload: Value = if is_trace {
        let tape_path = get_str(record, "trace_tape_path")?;
        serde_json::from_slice(&fs::read(tape_path).map_err(|error| format!("read tape: {error}"))?).map_err(|error| format!("invalid trace tape: {error}"))?
    } else {
        serde_json::from_slice(&fs::read(source_path).map_err(|error| format!("read attempt: {error}"))?).map_err(|error| format!("invalid attempt JSON: {error}"))?
    };
    let scenario = if is_trace { payload.get("scenario").ok_or("trace scenario missing")? } else { payload.get("scenario").ok_or("attempt scenario missing")? };
    let seed = if is_trace { payload.get("seed").and_then(Value::as_u64).ok_or("trace seed missing")? } else { payload.get("seed").and_then(Value::as_u64).ok_or("attempt seed missing")? };
    let mut metadata = record.clone();
    metadata["seed"] = json!(seed);
    if let Some(provenance) = payload.get("provenance") { metadata["provenance"] = provenance.clone(); }
    let initial = constructed_run(&metadata, scenario);
    let jsonl_path = out.join(format!("{input_id}.jsonl"));
    let text_path = out.join(format!("{input_id}.texts.jsonl.gz"));
    let mut jsonl = BufWriter::new(File::create(jsonl_path).map_err(|error| error.to_string())?);
    let mut texts = GzipSink::create(&text_path)?;
    let steps = input_steps(record, &payload)?;
    let pretty = is_trace || get_str(record, "milestone")? == "M15";
    let mut prefix = 0usize;
    let mut matched = true;
    let mut divergence_step = None;
    let mut divergence = None;
    let mut endpoint = "constructor failed".to_owned();
    let mut run_opt = None;

    match initial {
        Ok(mut run) => {
            endpoint = current_endpoint(&run);
            let mut stop = false;
            // A supplied-entry Move is an explicit recorded action outside the caller policy tape.
            if !stop && !is_trace {
                let bootstrap_count = payload.get("bootstrap_accepted_count").and_then(Value::as_u64).unwrap_or(0);
                let bootstrap = payload.pointer("/provenance/initial/supplied_entry");
                if bootstrap_count > 0 {
                    if let Some(entry) = bootstrap {
                        if let Some(action_record) = entry.get("action") {
                            let action = structured_action(action_record)?;
                            let action_debug = format!("{action:?}");
                            let prior = run.decision().map(|decision| decision.combat.is_some()).unwrap_or(false);
                            let step_result = if matches!(milestone, "M22" | "M23" | "M24" | "M25") {
                                run.step_observed(&action).result
                            } else {
                                run.step(&action)
                            };
                            let error = step_result.as_ref().err().map(|error| format!("{error:?}"));
                            let success_matches = step_result.is_ok() == entry.get("accepted").and_then(Value::as_bool).unwrap_or(true);
                            write_run_snapshot(&run, -1, &action_debug, "bootstrap", Some(success_matches), error.as_deref(), &mut jsonl, &mut texts, g4, input_id, prior)?;
                            if !success_matches || error.is_some() {
                                divergence_step = Some(0);
                                divergence = Some(format!("bootstrap action failed or acceptance differs: {}", error.unwrap_or_else(|| "recorded accepted flag differs".into())));
                                matched = false;
                                stop = true;
                            }
                            endpoint = current_endpoint(&run);
                        }
                    }
                }
            }

            if !stop {
                if is_trace {
                    if let Some(first) = steps.first() {
                        let expected = first.get("pre_decision_sha256").and_then(Value::as_str);
                        let actual = actual_decision(&run, true)?;
                        if expected != Some(sha256(&actual).as_str()) {
                            divergence_step = Some(0);
                            divergence = Some("initial pre decision differs from recorded trace".to_owned());
                            matched = false;
                            stop = true;
                        }
                    }
                } else if let Some(first) = steps.first() {
                    if let Some(expected) = expected_attempt_pre(first) {
                        let actual = actual_decision(&run, pretty)?;
                        if actual != expected {
                            divergence_step = Some(0);
                            divergence = Some("initial pre decision differs from recorded attempt".to_owned());
                            matched = false;
                            stop = true;
                        }
                    }
                }
            }

            for (index, step) in steps.iter().enumerate() {
                if stop { break; }
                let (action, action_text) = match action_value(step) {
                    Ok(action) => action,
                    Err(error) => {
                        divergence_step = Some(index);
                        divergence = Some(format!("recorded action could not be parsed: {error}"));
                        matched = false;
                        break;
                    }
                };
                let prior = run.decision().map(|decision| decision.combat.is_some()).unwrap_or(false);
                let result = if matches!(milestone, "M22" | "M23" | "M24" | "M25") {
                    run.step_observed(&action).result
                } else {
                    run.step(&action)
                };
                if let Err(error) = &result {
                    let error = format!("{error:?}");
                    write_run_snapshot(&run, index as i64, &action_text, "action", Some(false), Some(&error), &mut jsonl, &mut texts, g4, input_id, prior)?;
                    divergence_step = Some(index);
                    divergence = Some(format!("Run::step rejected recorded action: {error}"));
                    matched = false;
                    endpoint = current_endpoint(&run);
                    break;
                }
                let post_match = if is_trace {
                    let expected = steps.get(index + 1).and_then(|next| next.get("pre_decision_sha256")).and_then(Value::as_str)
                        .or_else(|| payload.get("final_decision_sha256").and_then(Value::as_str));
                    let actual = actual_decision(&run, true)?;
                    expected.map(|sha| sha == sha256(&actual)).ok_or_else(|| "trace has no post decision for this final step".to_owned())
                } else {
                    attempt_decision_match(&run, expected_attempt_post(step), pretty)
                };
                let post_match = match post_match {
                    Ok(value) => value,
                    Err(reason) => {
                        write_run_snapshot(&run, index as i64, &action_text, "action", Some(false), Some(&reason), &mut jsonl, &mut texts, g4, input_id, prior)?;
                        divergence_step = Some(index);
                        divergence = Some(reason);
                        matched = false;
                        endpoint = current_endpoint(&run);
                        break;
                    }
                };
                write_run_snapshot(&run, index as i64, &action_text, "action", Some(post_match), None, &mut jsonl, &mut texts, g4, input_id, prior)?;
                if !post_match {
                    divergence_step = Some(index);
                    divergence = Some("replayed post decision differs from recorded decision".into());
                    matched = false;
                    endpoint = current_endpoint(&run);
                    break;
                }
                prefix += 1;
                endpoint = current_endpoint(&run);
            }

            // Empty tapes have no post-step checkpoint; compare their retained endpoint.
            if !stop && divergence.is_none() && !is_trace && steps.is_empty() {
                if let Some(expected) = payload.pointer("/endpoint/decision_debug").and_then(Value::as_str) {
                    if actual_decision(&run, false)? != expected {
                        matched = false;
                        divergence = Some("constructor endpoint differs from recorded decision".into());
                        divergence_step = Some(0);
                    }
                }
            }
            run_opt = Some(run);
        }
        Err(error) => {
            matched = false;
            divergence_step = Some(0);
            divergence = Some(format!("existing constructor route failed: {error}"));
        }
    }

    if let Some(run) = run_opt.as_ref() { endpoint = current_endpoint(run); }
    jsonl.flush().map_err(|error| error.to_string())?;
    texts.finish()?;
    let status = if divergence.is_some() { "diverged" } else { "replayed" };
    let divergence_step_text = divergence_step.map(|step| step.to_string()).unwrap_or_default();
    let divergence_text = divergence.as_deref().unwrap_or("").replace(['\t', '\n', '\r'], " ");
    Ok(RunSummary {
        input: input_id.to_owned(),
        steps: steps.len(),
        endpoint,
        replay_status: status.to_owned(),
        replayed_prefix_steps: prefix,
        matched_record: if matched { "Y" } else { "N" }.to_owned(),
        divergence_step: divergence_step_text,
        divergence: divergence_text,
    })
}

struct RunSummary {
    input: String,
    steps: usize,
    endpoint: String,
    replay_status: String,
    replayed_prefix_steps: usize,
    matched_record: String,
    divergence_step: String,
    divergence: String,
}

impl RunSummary {
    fn tsv(&self) -> String {
        format!("{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}", self.input, self.steps, self.endpoint.replace(['\t','\n','\r'], " "), self.replay_status, self.replayed_prefix_steps, self.matched_record, self.divergence_step, self.divergence)
    }
}

fn clean(text: &str) -> String { text.replace(['\t', '\n', '\r'], " ") }

fn main() -> Result<(), String> {
    let args = env::args_os().skip(1).collect::<Vec<_>>();
    if args.len() != 2 { return Err("usage: m27_frozen_replay INPUTS.json OUTPUT_DIR".into()); }
    let input_path = PathBuf::from(&args[0]);
    let out = PathBuf::from(&args[1]);
    fs::create_dir_all(&out).map_err(|error| error.to_string())?;
    let inventory: Value = serde_json::from_slice(&fs::read(input_path).map_err(|error| error.to_string())?).map_err(|error| format!("invalid inputs: {error}"))?;
    let inputs = inventory.get("inputs").and_then(Value::as_array).ok_or("inputs array missing")?;
    let mut summary = String::from("input\tsteps\tendpoint\treplay_status\treplayed_prefix_steps\tmatched_record\tdivergence_step\tdivergence\n");
    let mut g4 = vec![String::from("input\tstep\trelics\tenemy1_public_full_power_order")];
    for input in inputs {
        let input_id = get_str(input, "input_id")?;
        match get_str(input, "type")? {
            "scripted_pair_combat" => match replay_pair(input, &out, &mut g4) {
                Ok((steps, endpoint)) => summary.push_str(&format!("{}\t{}\t{}\treplayed\t{}\tN/A\t\t\n", clean(input_id), steps, clean(&endpoint), steps)),
                Err(error) => summary.push_str(&format!("{}\t0\t-\tdiverged\t0\tN/A\t0\t{}\n", clean(input_id), clean(&error))),
            },
            "external_log" => {
                let result = replay_external(input, &out, &mut g4);
                match result {
                    Ok(result) => summary.push_str(&format!("{}\n", result.tsv())),
                    Err(error) => summary.push_str(&format!("{}\t0\t-\tdiverged\t0\tN\t0\t{}\n", clean(input_id), clean(&error))),
                }
            }
            _ => summary.push_str(&format!("{}\t0\t-\tdiverged\t0\tN\t0\tunknown input type\n", clean(input_id))),
        }
    }
    fs::write(out.join("summary.tsv"), summary).map_err(|error| error.to_string())?;
    fs::write(out.join("g4.tsv"), format!("{}\n", g4.join("\n"))).map_err(|error| error.to_string())?;
    Ok(())
}
