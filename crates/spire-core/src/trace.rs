//! `trace-v1` reference trace protocol — read side.
//!
//! A trace is a UTF-8 JSON Lines document with a strict record order
//! (`header`, `initial`, `step`*, `end`), 16-digit lowercase canonical
//! hexadecimal 64-bit RNG words, a SHA-256 fingerprint chain, and
//! reference-build provenance. The schema, units, and field vocabulary are
//! documented in `docs/validation/trace-v1.md`; the writer is the Java
//! `:protocol` project under `validation/windows-mod`.
//!
//! `read_trace` is fail-closed: a malformed, truncated, oversized, or
//! schema-nonconforming trace is a `TraceError`. The parser never repairs,
//! drops, or guesses.

use std::io::Read;
use std::path::Path;

/// The only wire version accepted by this parser.
pub const TRACE_VERSION: &str = "1";

/// Default reported read limit (64 MiB); callers pass an explicit limit.
pub const DEFAULT_MAX_TRACE_BYTES: u64 = 64 * 1024 * 1024;

/// Pinned game identity (trace-local, not machine-specific).
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct GameIdentity {
    pub id: String,
    pub character: String,
    pub ascension: u32,
}

/// Where a trace came from. Never a machine path.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct Provenance {
    pub tool: String,
    pub tool_version: String,
    pub captured_at_utc: Option<String>,
}

/// Hashes of the pinned game and loader binaries the trace was captured in.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct ReferenceBuild {
    pub game_sha256: String,
    pub loader_sha256: String,
}

/// Typed header record of a `trace-v1` document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceHeader {
    pub game: GameIdentity,
    /// 64-bit RNG seed words, each already validated as 16-digit hex.
    pub seed_words: Vec<String>,
    pub provenance: Provenance,
    pub reference_build: ReferenceBuild,
}

/// Typed initial record: the canonical state before the first action.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InitialRecord {
    pub state: serde_json::Value,
}

/// Observed semantic action at a step boundary, with trace-local identities.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SemanticAction {
    PlayCard { card: String, target: String },
    EndTurn,
}

/// One indexed step: the action taken and the canonical expected state after
/// it. `expected` is kept as JSON per the approved interface and validated
/// against the `trace-v1` state field vocabulary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceStep {
    pub index: u64,
    pub action: SemanticAction,
    pub expected: serde_json::Value,
}

/// Combat outcome recorded by the typed end record.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Victory,
    Defeat,
}

/// Typed end record of a `trace-v1` document.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TraceEnd {
    pub outcome: Outcome,
}

/// A fully parsed, schema-validated `trace-v1` document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceDocument {
    pub header: TraceHeader,
    pub initial: InitialRecord,
    pub steps: Vec<TraceStep>,
    pub end: TraceEnd,
}

/// Fail-closed parser errors. Every variant is an explicit rejection.
#[derive(Debug)]
pub enum TraceError {
    Io(std::io::Error),
    /// File larger than the configured read limit.
    Oversized { max_bytes: u64, size: u64 },
    /// Not valid UTF-8.
    Utf8 { bytes: usize },
    /// File is empty.
    Empty,
    /// Final line is not terminated by a newline.
    Truncated { line: usize },
    /// Blank line in the middle of the document.
    BlankLine { line: usize },
    /// Record is not the expected JSON document shape.
    Json { line: usize, message: String },
    /// Document is missing a required record.
    MissingRecord { expected: &'static str },
    /// A record appears where a different record was required.
    UnexpectedRecord { line: usize, found: String, expected: &'static str },
    /// Wire version other than `TRACE_VERSION`.
    UnknownVersion { version: String },
    /// An RNG word that is not exactly 16 lowercase hexadecimal digits.
    BadRngWord { word: String },
    /// Missing or well-formedness- failed reference-build metadata.
    MissingReferenceBuild { field: &'static str },
    /// Fingerprint does not equal the SHA-256 of the previous record line.
    BadFingerprint { line: usize },
    /// Step index does not follow the strict 0-based sequence.
    StepIndexOutOfOrder { line: usize, expected: u64, found: u64 },
    /// A step index repeated.
    DuplicateStep { index: u64 },
    /// A non-finite quantity (NaN/Infinity) in a state quantity.
    NonFiniteQuantity { line: usize },
    /// A state field outside the `trace-v1` supported vocabulary.
    UnsupportedStateField { line: usize, field: String },
    /// A player field outside the `trace-v1` supported vocabulary.
    UnsupportedPlayerField { line: usize, field: String },
    /// An action kind outside the `trace-v1` supported set.
    UnsupportedAction { line: usize, kind: String },
    /// An outcome outside the `trace-v1` supported set.
    BadOutcome { line: usize, value: String },
    /// A field that must be a JSON object is not one.
    NotAnObject { line: usize, what: &'static str },
}

impl std::fmt::Display for TraceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use TraceError as E;
        match self {
            E::Io(e) => write!(f, "trace I/O error: {e}"),
            E::Oversized { max_bytes, size } => {
                write!(f, "trace file is {size} bytes, exceeding the {max_bytes} byte read limit")
            }
            E::Utf8 { bytes } => write!(f, "trace file of {bytes} bytes is not valid UTF-8"),
            E::Empty => write!(f, "trace file is empty"),
            E::Truncated { line } => {
                write!(f, "trace file is truncated: line {line} has no terminating newline")
            }
            E::BlankLine { line } => write!(f, "blank line {line} in trace document"),
            E::Json { line, message } => write!(f, "line {line}: malformed record: {message}"),
            E::MissingRecord { expected } => write!(f, "trace document is missing its {expected} record"),
            E::UnexpectedRecord { line, found, expected } => {
                write!(f, "line {line}: found a '{found}' record where a '{expected}' record was required")
            }
            E::UnknownVersion { version } => {
                write!(f, "unsupported trace version '{version}' (this parser accepts trace-v1 only)")
            }
            E::BadRngWord { word } => {
                write!(f, "RNG word '{word}' is not exactly 16 lowercase hexadecimal digits")
            }
            E::MissingReferenceBuild { field } => {
                write!(f, "reference-build metadata field '{field}' is missing or malformed")
            }
            E::BadFingerprint { line } => {
                write!(f, "line {line}: fingerprint is not the SHA-256 of the preceding record line")
            }
            E::StepIndexOutOfOrder { line, expected, found } => {
                write!(f, "line {line}: step index {found} breaks the strict 0-based sequence (expected {expected})")
            }
            E::DuplicateStep { index } => write!(f, "step index {index} appears more than once"),
            E::NonFiniteQuantity { line } => write!(f, "line {line}: non-finite quantity (NaN/Infinity) is not allowed"),
            E::UnsupportedStateField { line, field } => {
                write!(f, "line {line}: state field '{field}' is not part of the trace-v1 supported vocabulary")
            }
            E::UnsupportedPlayerField { line, field } => {
                write!(f, "line {line}: player field '{field}' is not part of the trace-v1 supported vocabulary")
            }
            E::UnsupportedAction { line, kind } => {
                write!(f, "line {line}: action kind '{kind}' is not supported by trace-v1")
            }
            E::BadOutcome { line, value } => {
                write!(f, "line {line}: outcome '{value}' is not one of the trace-v1 supported outcomes")
            }
            E::NotAnObject { line, what } => write!(f, "line {line}: {what} must be a JSON object"),
        }
    }
}

impl std::error::Error for TraceError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            TraceError::Io(e) => Some(e),
            _ => None,
        }
    }
}

/// Top-level state fields allowed by the `trace-v1` vocabulary. These names
/// are the agreed interface with the Task 4 state model; anything else (for
/// example unsupported-mechanic fields such as `potions`) is rejected, never
/// silently dropped.
const SUPPORTED_STATE_FIELDS: &[&str] = &[
    "player", "enemies", "cards", "hand", "draw", "discard", "exhaust", "in_play", "rng", "queue",
    "pending", "phase", "turn", "outcome", "public_knowledge", "ruleset",
];

/// Player object fields allowed by the `trace-v1` vocabulary (quantities of
/// integer units, plus the trace-local relics/status lists).
const SUPPORTED_PLAYER_FIELDS: &[&str] = &[
    "hp",
    "max_hp",
    "block",
    "energy",
    "energy_per_turn",
    "base_draw",
    "relics",
    "statuses",
];

/// The wire tag of every record.
const RECORD_TAG: &str = "record";

/// Read at most `limit + 1` bytes from `reader` into `buf` (appending).
/// Returns the number of bytes actually delivered by the source.
///
/// `Read::take` caps the number of **bytes delivered to `buf`** at
/// `limit + 1`, regardless of the source's actual size.  This bounds the
/// delivered byte count, not Vec capacity or all process allocations.
/// `limit.saturating_add(1)` prevents u64 overflow; when `limit == u64::MAX`
/// the cap saturates at `u64::MAX` bytes (no representable sentinel beyond).
fn read_at_most<R: Read>(
    reader: R,
    limit: u64,
    buf: &mut Vec<u8>,
) -> std::io::Result<u64> {
    let mut bounded = reader.take(limit.saturating_add(1));
    let count = std::io::copy(&mut bounded, buf)?;
    Ok(count)
}

/// Bounded read + authoritative error mapping used by `read_trace`.
///
/// Reads at most `max_bytes + 1` bytes from `source` into a fresh buffer.
/// Returns the bytes on success; returns `TraceError::Oversized` with the
/// actual delivered count when the source delivered more than `max_bytes`.
///
/// This is the private production seam: it is called by `read_trace` on a
/// `File` handle and is also called directly by the test suite on finite
/// counting sources to verify the `Oversized` mapping and consumption bound
/// without a real file.
fn bounded_read_check<R: Read>(
    source: R,
    max_bytes: u64,
) -> Result<Vec<u8>, TraceError> {
    let mut buf = Vec::new();
    let count = read_at_most(source, max_bytes, &mut buf).map_err(TraceError::Io)?;
    if count > max_bytes {
        return Err(TraceError::Oversized { max_bytes, size: count });
    }
    Ok(buf)
}

/// Read and validate a `trace-v1` trace file of at most `max_bytes` bytes.
///
/// The check is fail-closed and order-sensitive: size limit, UTF-8 framing,
/// record order (`header`, `initial`, `step`*, `end`), strict 0-based step
/// indexing, RNG word and reference-build well-formedness, state field
/// vocabulary, and the SHA-256 fingerprint chain.
pub fn read_trace(path: &Path, max_bytes: u64) -> Result<TraceDocument, TraceError> {
    // Early size check via metadata (optimization; NOT the authoritative check).
    let meta = std::fs::metadata(path).map_err(TraceError::Io)?;
    if meta.len() > max_bytes {
        return Err(TraceError::Oversized { max_bytes, size: meta.len() });
    }
    // Open once, bounded read + authoritative Oversized check via the same
    // production seam used by tests.
    let file = std::fs::File::open(path).map_err(TraceError::Io)?;
    let bytes = bounded_read_check(file, max_bytes)?;
    let text = std::str::from_utf8(&bytes)
        .map_err(|_| TraceError::Utf8 { bytes: bytes.len() as usize })?;
    if text.is_empty() {
        return Err(TraceError::Empty);
    }
    let mut lines: Vec<&str> = text.split('\n').collect();
    if !text.ends_with('\n') {
        return Err(TraceError::Truncated {
            line: text.lines().count() + 1,
        });
    }
    lines.pop(); // the empty piece after the final newline
    if lines.iter().any(|l| l.is_empty()) {
        let line = lines.iter().position(|l| l.is_empty()).unwrap() + 1;
        return Err(TraceError::BlankLine { line });
    }
    if lines.len() < 3 {
        return Err(TraceError::MissingRecord { expected: "end" });
    }

    // Strict record order: header, initial, step*, end — parsed in sequence.
    let end_line = lines.len();
    let header = parse_header(1, lines[0])?;
    let initial = parse_initial(2, lines[1], lines[0])?;

    let mut steps = Vec::with_capacity(end_line - 3);
    let mut next_index: u64 = 0;
    for at in 2..end_line - 1 {
        let line = at + 1; // 1-based
        let step = parse_step(line, lines[at], lines[at - 1])?;
        if step.index != next_index {
            if step.index < next_index {
                return Err(TraceError::DuplicateStep { index: step.index });
            }
            return Err(TraceError::StepIndexOutOfOrder {
                line,
                expected: next_index,
                found: step.index,
            });
        }
        next_index += 1;
        steps.push(step);
    }

    let end = parse_end(end_line, lines[end_line - 1], lines[end_line - 2])?;

    Ok(TraceDocument {
        header,
        initial,
        steps,
        end,
    })
}

/// Decode one record line of the expected kind after a generic JSON check.
fn parse_record<T: serde::de::DeserializeOwned>(
    line: usize,
    raw: &str,
    expected_kind: &'static str,
) -> Result<T, TraceError> {
    let kind = record_kind(line, raw)?;
    if kind != expected_kind {
        return Err(TraceError::UnexpectedRecord {
            line,
            found: kind,
            expected: expected_kind,
        });
    }
    let value: Result<T, _> = serde_json::from_str(raw);
    value.map_err(|e| json_error(line, raw, e))
}

fn record_kind(line: usize, raw: &str) -> Result<String, TraceError> {
    let value: serde_json::Value = record_value(line, raw, "record tag")?;
    value
        .get(RECORD_TAG)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| TraceError::Json {
            line,
            message: "missing or non-string 'record' tag".to_string(),
        })
}

fn record_value(line: usize, raw: &str, what: &str) -> Result<serde_json::Value, TraceError> {
    serde_json::from_str::<serde_json::Value>(raw).map_err(|e| {
        if raw.contains("NaN") || raw.contains("Infinity") {
            TraceError::NonFiniteQuantity { line }
        } else {
            TraceError::Json {
                line,
                message: format!("{what}: {e}"),
            }
        }
    })
}

fn json_error(line: usize, raw: &str, e: serde_json::Error) -> TraceError {
    if raw.contains("NaN") || raw.contains("Infinity") {
        TraceError::NonFiniteQuantity { line }
    } else {
        TraceError::Json {
            line,
            message: e.to_string(),
        }
    }
}

/// Wire shapes (private). `deny_unknown_fields` keeps the parser from
/// silently deleting mismatching fields: any field outside `trace-v1` is a
/// hard error.
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct WireHeader {
    #[allow(dead_code)]
    record: String,
    version: String,
    game: GameIdentity,
    rng: RngHeader,
    provenance: Provenance,
    reference_build: Option<ReferenceBuild>,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct RngHeader {
    seed_words: Vec<String>,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct WireInitial {
    #[allow(dead_code)]
    record: String,
    fingerprint: String,
    state: serde_json::Value,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct WireStep {
    #[allow(dead_code)]
    record: String,
    fingerprint: String,
    index: u64,
    action: serde_json::Value,
    expected: serde_json::Value,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct WireEnd {
    #[allow(dead_code)]
    record: String,
    fingerprint: String,
    outcome: String,
}

fn parse_header(line: usize, raw: &str) -> Result<TraceHeader, TraceError> {
    let wire: WireHeader = parse_record(line, raw, "header")?;
    if wire.version != TRACE_VERSION {
        return Err(TraceError::UnknownVersion {
            version: wire.version,
        });
    }
    if wire.game.id.is_empty() || wire.game.character.is_empty() {
        return Err(TraceError::Json {
            line,
            message: "game identity (id and character) must be non-empty".to_string(),
        });
    }
    if wire.provenance.tool.is_empty() || wire.provenance.tool_version.is_empty() {
        return Err(TraceError::Json {
            line,
            message: "provenance must name the capture tool and its version".to_string(),
        });
    }
    if wire.rng.seed_words.is_empty() {
        return Err(TraceError::BadRngWord {
            word: "(missing rng.seed_words)".to_string(),
        });
    }
    for word in &wire.rng.seed_words {
        if !is_hex16(word) {
            return Err(TraceError::BadRngWord { word: word.clone() });
        }
    }
    let reference_build = match wire.reference_build {
        Some(rb) => rb,
        None => {
            return Err(TraceError::MissingReferenceBuild {
                field: "reference_build",
            })
        }
    };
    if !is_hex64(&reference_build.game_sha256) {
        return Err(TraceError::MissingReferenceBuild {
            field: "reference_build.game_sha256",
        });
    }
    if !is_hex64(&reference_build.loader_sha256) {
        return Err(TraceError::MissingReferenceBuild {
            field: "reference_build.loader_sha256",
        });
    }
    Ok(TraceHeader {
        game: wire.game,
        seed_words: wire.rng.seed_words,
        provenance: wire.provenance,
        reference_build,
    })
}

fn parse_initial(line: usize, raw: &str, prev: &str) -> Result<InitialRecord, TraceError> {
    let wire: WireInitial = parse_record(line, raw, "initial")?;
    validate_state(line, &wire.state)?;
    verify_fingerprint(line, &wire.fingerprint, prev)?;
    Ok(InitialRecord {
        state: wire.state,
    })
}

fn parse_step(line: usize, raw: &str, prev: &str) -> Result<TraceStep, TraceError> {
    let wire: WireStep = parse_record(line, raw, "step")?;
    let action = semantic_action(line, &wire.action)?;
    validate_state(line, &wire.expected)?;
    verify_fingerprint(line, &wire.fingerprint, prev)?;
    Ok(TraceStep {
        index: wire.index,
        action,
        expected: wire.expected,
    })
}

fn parse_end(line: usize, raw: &str, prev: &str) -> Result<TraceEnd, TraceError> {
    let wire: WireEnd = parse_record(line, raw, "end")?;
    let outcome = match wire.outcome.as_str() {
        "victory" => Outcome::Victory,
        "defeat" => Outcome::Defeat,
        other => {
            return Err(TraceError::BadOutcome {
                line,
                value: other.to_string(),
            })
        }
    };
    verify_fingerprint(line, &wire.fingerprint, prev)?;
    Ok(TraceEnd { outcome })
}

fn semantic_action(line: usize, value: &serde_json::Value) -> Result<SemanticAction, TraceError> {
    let obj = value
        .as_object()
        .ok_or(TraceError::NotAnObject { line, what: "action" })?;
    let kind = obj.get(RECORD_ACTION_KIND).and_then(|v| v.as_str()).unwrap_or("");
    match kind {
        "play_card" => {
            if obj.len() != 3 {
                return Err(TraceError::UnsupportedAction {
                    line,
                    kind: "play_card".to_string(),
                });
            }
            let card = non_empty(obj, "card", line)?;
            let target = non_empty(obj, "target", line)?;
            Ok(SemanticAction::PlayCard { card, target })
        }
        "end_turn" => {
            if obj.len() != 1 {
                return Err(TraceError::UnsupportedAction {
                    line,
                    kind: "end_turn".to_string(),
                });
            }
            Ok(SemanticAction::EndTurn)
        }
        _ => Err(TraceError::UnsupportedAction {
            line,
            kind: kind.to_string(),
        }),
    }
}

const RECORD_ACTION_KIND: &str = "type";

fn non_empty(
    obj: &serde_json::Map<String, serde_json::Value>,
    key: &str,
    line: usize,
) -> Result<String, TraceError> {
    obj.get(key)
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .ok_or(TraceError::UnsupportedAction {
            line,
            kind: key.to_string(),
        })
}

fn validate_state(line: usize, state: &serde_json::Value) -> Result<(), TraceError> {
    let obj = state
        .as_object()
        .ok_or(TraceError::NotAnObject { line, what: "state" })?;
    for key in obj.keys() {
        if !SUPPORTED_STATE_FIELDS.contains(&key.as_str()) {
            return Err(TraceError::UnsupportedStateField {
                line,
                field: key.clone(),
            });
        }
    }
    if let Some(player) = obj.get("player") {
        let player = match player.as_object() {
            Some(p) => p,
            None => {
                return Err(TraceError::NotAnObject {
                    line,
                    what: "state.player",
                })
            }
        };
        for key in player.keys() {
            if !SUPPORTED_PLAYER_FIELDS.contains(&key.as_str()) {
                return Err(TraceError::UnsupportedPlayerField {
                    line,
                    field: key.clone(),
                });
            }
        }
    }
    Ok(())
}

fn verify_fingerprint(line: usize, claimed: &str, prev_line: &str) -> Result<(), TraceError> {
    if is_hex64(claimed) && claimed == sha256_hex(prev_line.as_bytes()) {
        Ok(())
    } else {
        Err(TraceError::BadFingerprint { line })
    }
}

fn sha256_hex(data: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let digest = Sha256::digest(data);
    let mut out = String::with_capacity(64);
    for byte in digest {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

fn is_hex16(word: &str) -> bool {
    word.len() == 16 && word.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

fn is_hex64(word: &str) -> bool {
    word.len() == 64 && word.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

// ---------------------------------------------------------------------------
// bounded read helper (private — not part of public API)
// ---------------------------------------------------------------------------

// Tests for `read_at_most` — the bounded-read primitive used by `read_trace`.
// These unit tests prove that a source delivering more bytes than the
// configured limit is bounded to `limit + 1` bytes (one sentinel), and that
// common limit edges (zero, tiny, u64::MAX) do not overflow or hang.
#[cfg(test)]
mod bounded_read_tests {
    use super::*;
    use std::io::Cursor;

    /// A 20-byte source with limit=5 reads exactly 6 bytes (limit+1 sentinel).
    #[test]
    fn oversized_source_is_capped_at_limit_plus_one() {
        let data = vec![0u8; 20];
        let cursor = Cursor::new(data.as_slice());
        let mut buf = Vec::new();
        let count = read_at_most(cursor, 5, &mut buf).unwrap();
        assert_eq!(count, 6, "bounded to limit+1 (sentinel), got {count}");
        assert_eq!(buf.len(), 6);
        assert!(count > 5, "count > limit signals oversized");
    }

    /// A 10-byte source with limit=0 reads exactly 1 sentinel byte.
    #[test]
    fn zero_limit_reads_one_sentinel() {
        let data = vec![1u8; 10];
        let cursor = Cursor::new(data.as_slice());
        let mut buf = Vec::new();
        let count = read_at_most(cursor, 0, &mut buf).unwrap();
        assert_eq!(count, 1, "zero limit reads 1 sentinel byte, got {count}");
        assert_eq!(buf.len(), 1);
    }

    /// Exactly `limit` bytes available → reads all, count = limit (not oversized).
    #[test]
    fn exact_limit_passes_through() {
        let data = vec![7u8; 5];
        let cursor = Cursor::new(data.as_slice());
        let mut buf = Vec::new();
        let count = read_at_most(cursor, 5, &mut buf).unwrap();
        assert_eq!(count, 5, "exact limit passes through, got {count}");
        assert_eq!(buf.len(), 5);
    }

    /// Fewer bytes available than limit → reads all, count < limit.
    #[test]
    fn undersized_source_reads_all_available() {
        let data = vec![0u8; 3];
        let cursor = Cursor::new(data.as_slice());
        let mut buf = Vec::new();
        let count = read_at_most(cursor, 10, &mut buf).unwrap();
        assert_eq!(count, 3, "short source reads all 3 bytes, got {count}");
        assert_eq!(buf.len(), 3);
    }

    /// u64::MAX limit must not cause a u64 overflow panic.
    #[test]
    fn u64_max_limit_no_overflow() {
        let data = vec![0u8; 10];
        let cursor = Cursor::new(data.as_slice());
        let mut buf = Vec::new();
        let count = read_at_most(cursor, u64::MAX, &mut buf).unwrap();
        assert_eq!(count, 10, "u64::MAX limit reads all 10 bytes without panic, got {count}");
    }

    /// Limit=2 with 7-byte source: reads at most 3 (limit+1), not 7.
    #[test]
    fn tiny_limit_caps_at_plus_one() {
        let data = vec![0u8; 7];
        let cursor = Cursor::new(data.as_slice());
        let mut buf = Vec::new();
        let count = read_at_most(cursor, 2, &mut buf).unwrap();
        assert_eq!(count, 3, "tiny limit=2 caps at 3 (limit+1), got {count}");
        assert!(count > 2, "count > limit signals oversized");
    }

    /// Empty source with any limit → 0 bytes read.
    #[test]
    fn empty_source_reads_zero() {
        let data: Vec<u8> = Vec::new();
        let cursor = Cursor::new(data.as_slice());
        let mut buf = Vec::new();
        let count = read_at_most(cursor, 100, &mut buf).unwrap();
        assert_eq!(count, 0, "empty source reads 0 bytes");
        assert!(buf.is_empty());
    }

    // -----------------------------------------------------------------------
    // S2: post-metadata authoritative bounded-read / Oversized regression
    // -----------------------------------------------------------------------

    use std::cell::RefCell;
    use std::rc::Rc;

    /// Counting `Read` wrapper: tracks the exact number of bytes delivered
    /// to the parser by the source.  Used to prove that a finite oversized
    /// source is consumed at most `cap + 1` bytes, not the full source size.
    struct CountingRead<R: Read> {
        source: R,
        counter: Rc<RefCell<usize>>,
    }

    impl<R: Read> CountingRead<R> {
        /// Create a new `CountingRead` and a cloned counter handle.
        fn new(source: R) -> (Self, Rc<RefCell<usize>>) {
            let counter = Rc::new(RefCell::new(0usize));
            (Self { source, counter: Rc::clone(&counter) }, counter)
        }
    }

    impl<R: Read> Read for CountingRead<R> {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            let n = self.source.read(buf)?;
            *self.counter.borrow_mut() += n;
            Ok(n)
        }
    }

    /// A 20-byte finite source with `max_bytes=5`:
    ///  - `bounded_read_check` returns `TraceError::Oversized { max_bytes: 5, size: 6 }`
    ///  - The `CountingRead` counter confirms at most 6 bytes were consumed
    ///    (the source actually has 20 bytes available).
    #[test]
    fn bounded_read_check_oversized_asserts_error_and_consumes_at_most_cap_plus_one() {
        let data = vec![0u8; 20];
        let (counting, counter) = CountingRead::new(Cursor::new(data.as_slice()));

        let result = bounded_read_check(counting, 5);
        let consumed = *counter.borrow();

        match result {
            Err(TraceError::Oversized { max_bytes, size }) => {
                assert_eq!(max_bytes, 5, "Oversized.max_bytes must be the configured cap");
                assert_eq!(size, 6, "Oversized.size = cap+1 sentinel: got {size}");
                assert!(size > 5, "size > max_bytes confirms rejection path");
            }
            other => panic!("expected TraceError::Oversized, got: {other:?}"),
        }

        assert!(consumed <= 6,
            "CountingRead consumed {consumed} bytes from a 20-byte source; must be <= cap+1=6");
        assert_eq!(consumed, 6,
            "exactly cap+1 bytes delivered by source: got {consumed}");
    }

    /// `max_bytes=0` with a 10-byte source: reads 1 sentinel byte, not 10.
    #[test]
    fn bounded_read_check_zero_limit_returns_oversized_at_one_sentinel() {
        let data = vec![0u8; 10];
        let (counting, counter) = CountingRead::new(Cursor::new(data.as_slice()));

        match bounded_read_check(counting, 0) {
            Err(TraceError::Oversized { max_bytes, size }) => {
                assert_eq!(max_bytes, 0);
                assert_eq!(size, 1, "zero-limit: exactly 1 sentinel byte read, got {size}");
            }
            other => panic!("expected Oversized, got: {other:?}"),
        }
        assert_eq!(*counter.borrow(), 1, "consumed exactly 1 byte from a 10-byte source");
    }

    /// /dev/zero (Linux): metadata reports size 0, actual stream is infinite.
    /// The bounded-read path must reject with Oversized and consume only cap+1.
    /// This is the production "misleading initial size" scenario: the early
    /// metadata check (0 < cap) passes, then bounded read detects overflow.
    #[cfg(target_os = "linux")]
    #[test]
    fn dev_zero_bounded_read_returns_oversized_with_sentinel() {
        let cap: u64 = 4;
        let result = bounded_read_check(std::fs::File::open("/dev/zero").unwrap(), cap);
        match result {
            Err(TraceError::Oversized { max_bytes, size }) => {
                assert_eq!(max_bytes, 4, "Oversized.max_bytes = 4 (configured cap)");
                assert_eq!(size, 5, "Oversized.size = cap+1 = 5, got {size}");
            }
            other => panic!("expected Oversized from /dev/zero with cap=4, got: {other:?}"),
        }
    }

    /// /dev/null: always delivers 0 bytes → `Vec` is empty → `read_trace`
    /// maps to `Empty`.  This is a control test, not a consumption test.
    #[cfg(target_os = "linux")]
    #[test]
    fn dev_null_bounded_read_returns_empty_ok() {
        let result = bounded_read_check(std::fs::File::open("/dev/null").unwrap(), 100);
        match result {
            Ok(buf) => assert!(buf.is_empty(), "/dev/null delivers 0 bytes; buf must be empty"),
            Err(e) => panic!("expected Ok(empty buf) for /dev/null, got: {e:?}"),
        }
    }
}
