//! Once-only default-off M25 supplied traversal, Act4 cases and interaction fixtures.
mod m25_support;

#[cfg(feature = "m25-diagnostics")]
fn main() {
    if let Err(error) = execute() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

#[cfg(not(feature = "m25-diagnostics"))]
fn main() {
    eprintln!("fixed supplied diagnostics require default-off m25-diagnostics");
    std::process::exit(1);
}

#[cfg(feature = "m25-diagnostics")]
fn execute() -> Result<(), String> {
    use m25_support::{fixtures, valid_sha256, FREEZE_SHA256_ENV, TRACE_DIR_ENV};
    use std::path::PathBuf;

    // Reject every malformed argument before constructing the first Run.
    let series = fixtures::parse_series(&std::env::args().skip(1).collect::<Vec<_>>())?;
    let directory = std::env::var_os(TRACE_DIR_ENV)
        .map(PathBuf::from)
        .ok_or("M25_TRACE_DIR is required for durable receipts")?;
    let freeze = std::env::var(FREEZE_SHA256_ENV)
        .map_err(|_| "M25_FREEZE_SHA256 is required")?;
    if !valid_sha256(&freeze) {
        return Err("M25_FREEZE_SHA256 must be 64 hexadecimal characters".into());
    }
    std::fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
    preflight(&directory, &series)?;

    match series {
        fixtures::Series::Full => run_series(&directory, &freeze),
        fixtures::Series::F1Corrective { v1_supplied } => run_f1_corrective(&directory, &freeze, &v1_supplied),
        fixtures::Series::G1Corrective { v1_supplied, v2_supplied } => run_g1_corrective(&directory, &freeze, &v1_supplied, &v2_supplied),
    }
}

#[cfg(feature = "m25-diagnostics")]
fn preflight(directory: &std::path::Path, series: &m25_support::fixtures::Series) -> Result<(), String> {
    use m25_support::fixtures;
    use spire_core::run::diagnostics::{M25DiagnosticBoss, M25DiagnosticLoadout, M25PlayedFixture};

    if matches!(series, fixtures::Series::G1Corrective { .. }) {
        for name in [
            format!("seed-7-supplied-act4-{}-budget-{}.attempt.json", M25DiagnosticLoadout::CoverageC.name(), fixtures::ACT4_BUDGET),
            "g1-strong-rule-informational.json".into(),
        ] {
            match std::fs::symlink_metadata(directory.join(&name)) {
                Ok(_) => return Err(format!("refusing existing M25 supplied output {name}")),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => (),
                Err(error) => return Err(format!("inspect {name}: {error}")),
            }
        }
        return Ok(());
    }
    let corrective = matches!(series, fixtures::Series::F1Corrective { .. });
    let mut names = Vec::new();
    for boss in M25DiagnosticBoss::ALL.into_iter().filter(|_| !corrective) {
        names.push(format!(
            "seed-7-supplied-traversal-{}-budget-{}.attempt.json",
            boss.name(),
            fixtures::TRAVERSAL_BUDGET
        ));
    }
    for loadout in [
        M25DiagnosticLoadout::CoverageA,
        M25DiagnosticLoadout::CoverageB,
        M25DiagnosticLoadout::CoverageC,
        M25DiagnosticLoadout::Strong,
    ]
    .into_iter()
    .filter(|loadout| {
        !corrective || matches!(loadout, M25DiagnosticLoadout::CoverageB | M25DiagnosticLoadout::Strong)
    }) {
        names.push(format!(
            "seed-7-supplied-act4-{}-budget-{}.attempt.json",
            loadout.name(),
            fixtures::ACT4_BUDGET
        ));
    }
    for fixture in M25PlayedFixture::ALL {
        names.push(format!(
            "seed-7-played-fixture-{}-budget-{}.attempt.json",
            fixture.name(),
            fixtures::FIXTURE_BUDGET
        ));
    }
    names.push("supplied-conditional-decision.json".into());
    for name in names {
        let path = directory.join(&name);
        match std::fs::symlink_metadata(&path) {
            Ok(_) => return Err(format!("refusing existing M25 supplied output {name}")),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => (),
            Err(error) => return Err(format!("inspect {}: {error}", path.display())),
        }
    }
    Ok(())
}

#[cfg(feature = "m25-diagnostics")]
fn run_series(directory: &std::path::Path, freeze: &str) -> Result<(), String> {
    use m25_support::{fixtures, *};
    use serde_json::json;
    use spire_core::run::diagnostics::{self, M25DiagnosticBoss, M25DiagnosticLoadout};

    // Design §12.4: three seed-7 Act2 boss sources, in canonical Act3 boss order.
    for boss in M25DiagnosticBoss::ALL {
        let name = format!(
            "seed-7-supplied-traversal-{}-budget-{}.attempt.json",
            boss.name(),
            fixtures::TRAVERSAL_BUDGET
        );
        let case = json!({
            "supplied_traversal": boss.encounter().as_str(),
            "act2_boss_source": boss.traversal_act2().encounter().as_str(),
            "loadout": "strong-v1",
        });
        let run = diagnostics::m25_supplied_traversal(boss)
            .map_err(|error| format!("{name} construction: {error:?}"))?;
        execute_case(
            directory,
            freeze,
            &name,
            7,
            fixtures::TRAVERSAL_BUDGET,
            "supplied-traversal",
            case,
            run,
            false,
            false,
        )?;
    }

    // Design §12.5: three coverage cases, followed by the conditional strong case.
    let mut primary = Vec::new();
    for loadout in [
        M25DiagnosticLoadout::CoverageA,
        M25DiagnosticLoadout::CoverageB,
        M25DiagnosticLoadout::CoverageC,
    ] {
        primary.push(primary_case(directory, freeze, loadout)?);
    }
    strong_and_fixtures(directory, freeze, &primary, None)
}

/// PM ruling on fault F1: re-execute only coverage B, evaluate the frozen strong-case rule over the
/// retained v1 A and C outcomes and the new B outcome, then run the eight fixtures.
#[cfg(feature = "m25-diagnostics")]
fn run_f1_corrective(directory: &std::path::Path, freeze: &str, v1_supplied: &std::path::Path) -> Result<(), String> {
    use m25_support::fixtures;
    use spire_core::run::diagnostics::M25DiagnosticLoadout;

    let mut retained = Vec::new();
    for (name, sha256) in fixtures::V1_RETAINED_PRIMARY {
        let path = v1_supplied.join(name);
        let bytes = std::fs::read(&path).map_err(|error| format!("read {}: {error}", path.display()))?;
        retained.push(fixtures::retained_outcome(&bytes, sha256).map_err(|error| format!("{name}: {error}"))?);
    }
    let b = primary_case(directory, freeze, M25DiagnosticLoadout::CoverageB)?;
    let primary = vec![retained[0].clone(), b, retained[1].clone()];
    let sources = serde_json::json!({
        "coverage-a": format!("retained v1 receipt {} ({})", fixtures::V1_RETAINED_PRIMARY[0].0, fixtures::V1_RETAINED_PRIMARY[0].1),
        "coverage-b": "executed under this freeze (F1 corrective)",
        "coverage-c": format!("retained v1 receipt {} ({})", fixtures::V1_RETAINED_PRIMARY[1].0, fixtures::V1_RETAINED_PRIMARY[1].1),
        "v1_freeze_sha256": fixtures::V1_FREEZE_SHA256,
    });
    strong_and_fixtures(directory, freeze, &primary, Some(sources))
}

/// PM ruling on G1: v1 coverage C after step 25 was produced by the G1 defect. Re-execute only
/// coverage C; evaluate the frozen strong-case rule over A (v1), B (v2) and the new C for
/// information only. The strong case and the fixtures are not run (v2 retains them as executed).
#[cfg(feature = "m25-diagnostics")]
fn run_g1_corrective(
    directory: &std::path::Path,
    freeze: &str,
    v1_supplied: &std::path::Path,
    v2_supplied: &std::path::Path,
) -> Result<(), String> {
    use m25_support::{fixtures, write_new};
    use spire_core::run::diagnostics::M25DiagnosticLoadout;

    let read = |dir: &std::path::Path, (name, sha256): (&str, &str), freeze: &str| -> Result<Result<m25_support::M25CallerOutcome, String>, String> {
        let path = dir.join(name);
        let bytes = std::fs::read(&path).map_err(|error| format!("read {}: {error}", path.display()))?;
        fixtures::retained_outcome_from(&bytes, sha256, freeze).map_err(|error| format!("{name}: {error}"))
    };
    let a = read(v1_supplied, fixtures::V1_RETAINED_PRIMARY[0], fixtures::V1_FREEZE_SHA256)?;
    let b = read(v2_supplied, fixtures::V2_RETAINED_B, fixtures::V2_FREEZE_SHA256)?;
    let c = primary_case(directory, freeze, M25DiagnosticLoadout::CoverageC)?;
    let primary = vec![a, b, c];
    let rule = fixtures::should_run_strong(&primary);
    let informational = serde_json::json!({
        "schema_id": "m25-g1-strong-rule-informational-v1",
        "primary_outcomes": primary.iter().map(|outcome| format!("{outcome:?}")).collect::<Vec<_>>(),
        "primary_sources": {
            "coverage-a": format!("retained v1 receipt {} ({})", fixtures::V1_RETAINED_PRIMARY[0].0, fixtures::V1_RETAINED_PRIMARY[0].1),
            "coverage-b": format!("retained v2 receipt {} ({})", fixtures::V2_RETAINED_B.0, fixtures::V2_RETAINED_B.1),
            "coverage-c": "executed under this freeze (G1 corrective)",
        },
        "strong_rule_result": format!("{rule:?}"),
        "strong_case_executed": false,
        "note": "information only (PM ruling): the v2 strong case stays as executed history",
    });
    write_new(&directory.join("g1-strong-rule-informational.json"), &informational)
}

#[cfg(feature = "m25-diagnostics")]
fn primary_case(
    directory: &std::path::Path,
    freeze: &str,
    loadout: spire_core::run::diagnostics::M25DiagnosticLoadout,
) -> Result<Result<m25_support::M25CallerOutcome, String>, String> {
    use m25_support::fixtures;
    use serde_json::json;
    use spire_core::run::diagnostics;

    let name = format!(
        "seed-7-supplied-act4-{}-budget-{}.attempt.json",
        loadout.name(),
        fixtures::ACT4_BUDGET
    );
    let case = json!({
        "supplied_act4_loadout": loadout.name(),
        "act3_source_boss": loadout.act3().encounter().as_str(),
        "keys": [true, true, true],
    });
    let run = diagnostics::m25_supplied_act4(loadout)
        .map_err(|error| format!("{name} construction: {error:?}"))?;
    execute_case(
        directory,
        freeze,
        &name,
        7,
        fixtures::ACT4_BUDGET,
        "supplied-act4",
        case,
        run,
        false,
        false,
    )
}

#[cfg(feature = "m25-diagnostics")]
fn strong_and_fixtures(
    directory: &std::path::Path,
    freeze: &str,
    primary: &[Result<m25_support::M25CallerOutcome, String>],
    primary_sources: Option<serde_json::Value>,
) -> Result<(), String> {
    use m25_support::{fixtures, *};
    use serde_json::json;
    use spire_core::run::diagnostics::{self, M25DiagnosticLoadout, M25PlayedFixture};

    let strong = fixtures::should_run_strong(primary)?;
    if strong {
        let loadout = M25DiagnosticLoadout::Strong;
        let name = format!(
            "seed-7-supplied-act4-{}-budget-{}.attempt.json",
            loadout.name(),
            fixtures::ACT4_BUDGET
        );
        let case = json!({
            "supplied_act4_loadout": "strong-v1",
            "act3_source_boss": loadout.act3().encounter().as_str(),
            "keys": [true, true, true],
            "conditional": "none of the three primary cases reached HeartVictory",
        });
        let run = diagnostics::m25_supplied_act4(loadout)
            .map_err(|error| format!("{name} construction: {error:?}"))?;
        execute_case(
            directory,
            freeze,
            &name,
            7,
            fixtures::ACT4_BUDGET,
            "supplied-act4-conditional-strong",
            case,
            run,
            false,
            false,
        )?;
    }
    let mut conditional = json!({
        "schema_id": "m25-supplied-conditional-decision-v1",
        "primary_outcomes": primary.iter().map(|outcome| format!("{outcome:?}")).collect::<Vec<_>>(),
        "strong_case": if strong { "executed" } else { "skipped: primary reached HeartVictory" },
        "strong_case_executed": strong,
    });
    if let Some(sources) = primary_sources {
        conditional["primary_sources"] = sources;
    }
    write_new(
        &directory.join("supplied-conditional-decision.json"),
        &conditional,
    )?;

    // Design §12.6: the eight fixed fixtures are last and retain missed checkpoints.
    for fixture in M25PlayedFixture::ALL {
        let name = format!(
            "seed-7-played-fixture-{}-budget-{}.attempt.json",
            fixture.name(),
            fixtures::FIXTURE_BUDGET
        );
        let case = json!({
            "fixture": fixture.name(),
            "encounter": fixture.encounter().as_str(),
            "deck": format!("{:?}", fixture.deck()),
            "relics": format!("{:?}", fixture.relics()),
        });
        let run = diagnostics::m25_played_fixture(fixture)
            .map_err(|error| format!("{name} construction: {error:?}"))?;
        let _outcome = execute_case(
            directory,
            freeze,
            &name,
            7,
            fixtures::FIXTURE_BUDGET,
            "played-fixture",
            case,
            run,
            true,
            true,
        )?;
    }
    Ok(())
}

#[cfg(feature = "m25-diagnostics")]
#[allow(clippy::too_many_arguments)]
fn execute_case(
    directory: &std::path::Path,
    freeze: &str,
    name: &str,
    seed: u64,
    budget: usize,
    label: &str,
    case: serde_json::Value,
    mut run: spire_core::run::Run,
    bootstrap: bool,
    fixture: bool,
) -> Result<Result<m25_support::M25CallerOutcome, String>, String> {
    use m25_support::{caller_policy, fixture_policy, initial_value, persist, shared, Caller};
    use spire_core::run::diagnostics;

    let path = directory.join(name);
    let mut initial = initial_value(&run)?;
    initial["provenance"] = serde_json::json!({ "kind": label });
    initial["diagnostic_checkpoint"] = serde_json::json!(format!(
        "{:?}",
        diagnostics::m22_diagnostic_checkpoint(&run).map_err(|error| format!("{error:?}"))?
    ));
    let entry = if bootstrap {
        m25_support::bootstrap_supplied(&mut run, &mut initial)
    } else {
        Ok(())
    };
    let mut caller = Caller::new_m25(budget)?;
    let result = match entry {
        Ok(()) if fixture => shared::rollout_m25(&mut run, &mut caller, fixture_policy),
        Ok(()) => shared::rollout_m25(&mut run, &mut caller, caller_policy),
        Err(error) => Err(error),
    };
    persist(
        &path,
        seed,
        case,
        budget,
        label,
        freeze,
        &initial,
        &caller,
        Some(&run),
        &result,
    )?;
    println!(
        "{label} {name} result={result:?} policy_successful_count={} endpoint={:?} last_combat={:?}",
        caller.accepted(),
        run.decision().map(|decision| (decision.act, decision.floor, decision.terminal)),
        run.decision().ok().and_then(|decision| decision.last_combat),
    );
    Ok(result)
}
