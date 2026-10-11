//! `trace-v1` protocol parser tests against the manually authored
//! `protocol-test` samples in `validation/fixtures/protocol/`.
//!
//! These samples are wire-format parser tests (explicitly labeled
//! `protocol-test` in `validation/fixtures/manifest.json`). They are not
//! reference-runtime oracles and contain only well-formed synthetic
//! metadata.

use spire_core::trace::{read_trace, Outcome, SemanticAction, TraceError};
use std::path::{Path, PathBuf};

fn protocol_case(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../validation/fixtures/protocol")
        .join(format!("{name}.jsonl"))
}

#[test]
fn brief_acceptance_assertions() -> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(read_trace(&protocol_case("valid"), 64 * 1024 * 1024)?.steps.len(), 2);
    assert!(read_trace(&protocol_case("rng_numeric_instead_of_hex"), 64 * 1024 * 1024).is_err());
    assert!(read_trace(&protocol_case("unknown_version"), 64 * 1024 * 1024).is_err());
    Ok(())
}

#[test]
fn valid_trace_content_is_typed() {
    let doc = read_trace(&protocol_case("valid"), 64 * 1024 * 1024)
        .expect("the accepted protocol-test sample must parse");
    assert_eq!(doc.header.game.id, "slay-the-spire");
    assert_eq!(doc.header.game.character, "ironclad");
    assert_eq!(doc.header.seed_words, vec!["0000000000000001".to_string()]);
    assert_eq!(
        doc.initial.state["player"]["hp"], 70,
        "initial canonical state must be preserved without numeric loss"
    );
    assert_eq!(
        doc.steps[0].action,
        SemanticAction::PlayCard {
            card: "strike-1".to_string(),
            target: "cultist-1".to_string()
        }
    );
    assert_eq!(doc.steps[0].expected["player"]["energy"], 2);
    assert_eq!(doc.steps[0].expected["enemies"][0]["hp"], 48);
    assert_eq!(doc.steps[1].index, 1);
    assert_eq!(doc.steps[1].action, SemanticAction::EndTurn);
    assert_eq!(doc.steps[1].expected["turn"], 2);
    assert_eq!(doc.end.outcome, Outcome::Victory);
}

#[test]
fn rng_numeric_instead_of_hex_is_rejected() {
    assert!(read_trace(&protocol_case("rng_numeric_instead_of_hex"), 64 * 1024 * 1024).is_err());
}

#[test]
fn unknown_version_is_rejected() {
    let err = read_trace(&protocol_case("unknown_version"), 64 * 1024 * 1024)
        .expect_err("version 99 must be rejected");
    let shown = format!("{err}");
    assert!(
        matches!(err, TraceError::UnknownVersion { ref version } if version == "99"),
        "got: {shown}"
    );
}

#[test]
fn missing_header_is_rejected() {
    let err = read_trace(&protocol_case("missing_header"), 64 * 1024 * 1024)
        .expect_err("a document starting at the initial record must be rejected");
    let shown = format!("{err}");
    assert!(
        matches!(err, TraceError::UnexpectedRecord { expected: "header", .. }),
        "got: {shown}"
    );
}

#[test]
fn missing_end_is_rejected() {
    let err = read_trace(&protocol_case("missing_end"), 64 * 1024 * 1024)
        .expect_err("a document without an end record must be rejected");
    let shown = format!("{err}");
    assert!(
        matches!(
            err,
            TraceError::MissingRecord { expected } | TraceError::UnexpectedRecord { expected, .. }
            if expected == "end"
        ),
        "got: {shown}"
    );
}

#[test]
fn duplicate_step_index_is_rejected() {
    let err = read_trace(&protocol_case("duplicate_step"), 64 * 1024 * 1024)
        .expect_err("a repeated step index must be rejected");
    let shown = format!("{err}");
    assert!(
        matches!(err, TraceError::DuplicateStep { index: 0 }
            | TraceError::StepIndexOutOfOrder { expected: 1, found: 0, .. }),
        "got: {shown}"
    );
}

#[test]
fn nonfinite_quantity_is_rejected() {
    let err = read_trace(&protocol_case("nonfinite_quantity"), 64 * 1024 * 1024)
        .expect_err("a NaN quantity must be rejected");
    let shown = format!("{err}");
    assert!(
        matches!(err, TraceError::NonFiniteQuantity { .. }),
        "got: {shown}"
    );
}

#[test]
fn truncated_document_is_rejected() {
    let err = read_trace(&protocol_case("truncated"), 64 * 1024 * 1024)
        .expect_err("a document whose final line is cut off must be rejected");
    let shown = format!("{err}");
    assert!(
        matches!(err, TraceError::Truncated { .. }),
        "got: {shown}"
    );
}

#[test]
fn oversized_file_is_rejected() {
    let dir = Path::new("/tmp/opencode");
    std::fs::create_dir_all(dir).expect("temp dir");
    let path = dir.join(format!("spire_trace_oversized_{}.jsonl", std::process::id()));
    std::fs::write(&path, b"xx").expect("write oversized sample");
    let result = read_trace(&path, 1);
    let _ = std::fs::remove_file(&path);
    let shown = format!("{result:?}");
    assert!(matches!(result, Err(TraceError::Oversized { max_bytes: 1, size: 2 })), "got: {shown}");
}

#[test]
fn missing_reference_build_metadata_is_rejected() {
    let err = read_trace(&protocol_case("missing_reference_build"), 64 * 1024 * 1024)
        .expect_err("missing reference-build metadata must be rejected");
    let shown = format!("{err}");
    assert!(
        matches!(err, TraceError::MissingReferenceBuild { .. }),
        "got: {shown}"
    );
}

#[test]
fn unsupported_state_field_is_rejected() {
    let err = read_trace(&protocol_case("unsupported_state_field"), 64 * 1024 * 1024)
        .expect_err("potions are unsupported mechanics and must be rejected");
    let shown = format!("{err}");
    assert!(
        matches!(err, TraceError::UnsupportedStateField { field, .. } if field == "potions"),
        "got: {shown}"
    );
}

#[test]
fn tampered_fingerprint_is_rejected() {
    let err = read_trace(&protocol_case("tampered_fingerprint"), 64 * 1024 * 1024)
        .expect_err("a broken fingerprint chain must be rejected");
    let shown = format!("{err}");
    assert!(
        matches!(err, TraceError::BadFingerprint { line: 3 }),
        "got: {shown}"
    );
}

/// Cross-language interop (evidence run only): reads a trace produced by the
/// Java `:protocol` writer. Skipped unless `SPIRE_JAVA_TRACE` points at the
/// Java-written file, so the default suite stays hermetic.
#[test]
fn interop_java_written_trace_reads() {
    let path = match std::env::var("SPIRE_JAVA_TRACE") {
        Ok(p) => p,
        Err(_) => {
            eprintln!("SKIP: SPIRE_JAVA_TRACE not set (no Java-written sample in this run)");
            return;
        }
    };
    let doc = read_trace(Path::new(&path), 64 * 1024 * 1024)
        .unwrap_or_else(|e| panic!("the Java :protocol writer produced a trace the Rust reader must accept: {e}"));
    assert_eq!(doc.steps.len(), 2);
    assert_eq!(
        doc.steps[0].action,
        SemanticAction::PlayCard {
            card: "strike-1".to_string(),
            target: "cultist-1".to_string()
        }
    );
    assert_eq!(doc.end.outcome, Outcome::Victory);
}
