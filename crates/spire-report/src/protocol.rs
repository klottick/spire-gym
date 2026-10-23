use crate::ReportError;
use serde_json::{Map, Value, json};
use std::io::{BufRead, Read, Write};

/// Streaming display writer. Retains catalog definitions and ordering metadata, not frames.
/// Any write/flush failure poisons the writer: recovering the sink never repairs history.
pub struct ReportWriter<W: Write> {
    out: W,
    history: History,
    poisoned: bool,
}

impl<W: Write> ReportWriter<W> {
    pub fn new(out: W, header: Value) -> Result<Self, ReportError> {
        validate_header(&header)?;
        let mut writer = Self { out, history: History::default(), poisoned: false };
        writer.write(&header)?;
        Ok(writer)
    }

    pub fn catalog(&mut self, entries: Value) -> Result<(), ReportError> {
        self.append(json!({"record":"catalog","entries":entries}))
    }

    pub fn frame(&mut self, frame: Value) -> Result<(), ReportError> {
        self.append(json!({"record":"frame","frame":frame}))
    }

    /// Flush before attempting native work; this cannot guarantee future disk capacity.
    pub fn checkpoint(&mut self) -> Result<(), ReportError> {
        self.ready()?;
        self.flush()
    }

    pub fn finish(&mut self, capture: Value, game: Value) -> Result<(), ReportError> {
        self.append(json!({"record":"footer","capture":capture,"game":game}))
    }

    /// Recover the output, including complete records after a failure. Does not finish it.
    pub fn into_inner(self) -> W { self.out }

    fn ready(&self) -> Result<(), ReportError> {
        require(!self.poisoned, "writer is poisoned by an earlier I/O failure")?;
        require(!self.history.finished, "writer already finished")
    }

    fn append(&mut self, record: Value) -> Result<(), ReportError> {
        self.ready()?;
        self.history.accept(&record)?;
        self.write(&record)
    }

    fn write(&mut self, record: &Value) -> Result<(), ReportError> {
        // Serialize before touching the sink; a partial record is never marked complete.
        let mut bytes = serde_json::to_vec(record)?;
        bytes.push(b'\n');
        if let Err(error) = self.out.write_all(&bytes) {
            self.poisoned = true;
            return Err(error.into());
        }
        self.flush()
    }

    fn flush(&mut self) -> Result<(), ReportError> {
        self.out.flush().map_err(|error| {
            self.poisoned = true;
            error.into()
        })
    }
}

#[derive(Default)]
struct History {
    next: u64,
    catalog: Map<String, Value>,
    finished: bool,
}

impl History {
    fn accept(&mut self, record: &Value) -> Result<(), ReportError> {
        require(!self.finished, "record after footer")?;
        match record["record"].as_str() {
            Some("catalog") => {
                let entries = record["entries"].as_object().ok_or_else(|| invalid("catalog entries must be an object"))?;
                for (key, definition) in entries {
                    require(!key.is_empty() && definition.is_object(), "invalid catalog definition")?;
                    require(self.catalog.get(key).is_none_or(|old| old == definition), "conflicting immutable catalog definition")?;
                }
                self.catalog.extend(entries.clone());
            }
            Some("frame") => {
                let frame = &record["frame"];
                require(frame.is_object(), "frame must be an object")?;
                require(frame["id"].as_str() == Some(&self.next.to_string()), "frame IDs must be consecutive decimal strings starting at zero")?;
                if self.next == 0 {
                    require(frame.get("previousFrameId").is_some_and(Value::is_null), "initial frame must have null previousFrameId")?;
                    require(frame["kind"] == "initial", "first frame must be initial")?;
                } else {
                    require(frame["previousFrameId"].as_str() == Some(&(self.next - 1).to_string()), "incorrect previousFrameId")?;
                    require(frame["kind"] != "initial", "duplicate initial frame")?;
                }
                require(matches!(frame["kind"].as_str(),Some("initial"|"action"|"boundary"|"fault"|"end")), "invalid frame kind")?;
                require(matches!(frame["stateValidity"].as_str(),Some("settled"|"last_known")), "invalid stateValidity")?;
                for field in ["act", "floor", "turn"] {
                    nullable(frame, field, |value| value.as_u64().is_some())?;
                }
                nullable(frame, "combatId", Value::is_string)?;
                nullable(frame, "action", Value::is_object)?;
                if frame["kind"] == "action" { require(frame["action"].is_object(), "accepted action needs an action")?; }
                nullable(frame, "beforeScreen", Value::is_object)?;
                nullable(frame, "combatFinal", Value::is_object)?;
                validate_state(&frame["state"])?;
                let effects = frame["effects"].as_array().ok_or_else(|| invalid("frame effects must be an array"))?;
                let mut last_sequence = None;
                for effect in effects {
                    let sequence = effect["sequence"].as_u64().filter(|n| *n <= u32::MAX.into())
                        .ok_or_else(|| invalid("effect sequence must be a bounded integer"))?;
                    require(last_sequence.is_none_or(|last| sequence > last), "effects must have increasing sequence")?;
                    require(effect["kind"].is_string(), "effect kind must be text")?;
                    last_sequence = Some(sequence);
                }
                if let Some(parent) = frame["action"]["parentActionId"].as_str() {
                    let id = parent.parse::<u64>().map_err(|_| invalid("invalid parentActionId"))?;
                    require(id < self.next && id.to_string() == parent, "parentActionId must reference an earlier frame")?;
                }
                validate_refs(frame)?;
                self.next = self.next.checked_add(1).ok_or_else(|| invalid("frame ID limit exceeded"))?;
            }
            Some("footer") => {
                validate_footer(record)?;
                self.finished = true;
            }
            _ => return Err(invalid("unknown or misplaced record type")),
        }
        Ok(())
    }
}

fn validate_header(header: &Value) -> Result<(), ReportError> {
    require(header["record"] == "header", "first record must be header")?;
    require(header["schemaId"] == "spire-report-v1" && header["schemaVersion"].as_u64() == Some(1), "unsupported report schema")?;
    let source = &header["source"];
    require(matches!(source["kind"].as_str(),Some("combat_episode"|"run_prefix"|"run"|"authored_fixture")), "invalid source kind")?;
    for field in ["profile", "producerRevision", "contentId"] { nullable(source, field, Value::is_string)?; }
    require(source["evidenceGrade"].is_string(), "missing evidence grade")?;
    let run = &header["run"];
    nullable(run, "character", Value::is_string)?;
    nullable(run, "ascension", |value| value.as_u64().is_some())?;
    nullable(run, "caller", Value::is_object)?;
    nullable(&run["seed"], "encoding", Value::is_string)?;
    nullable(&run["seed"], "value", Value::is_string)
}

fn validate_footer(record: &Value) -> Result<(), ReportError> {
    let capture = &record["capture"];
    require(matches!(capture["status"].as_str(),Some("complete"|"unfinished"|"abandoned"|"faulted"|"interrupted")), "invalid capture status")?;
    nullable(capture, "reason", Value::is_string)?;
    let game = &record["game"];
    require(game.get("outcome").is_some(), "missing game outcome")?;
    require(game["outcome"].is_null() || matches!(game["outcome"].as_str(),Some("combat_victory"|"death"|"heart_victory"|"empty_deck"|"act3_no_keys")), "invalid game outcome")?;
    nullable(game, "boundary", Value::is_string)
}

fn validate_state(state: &Value) -> Result<(), ReportError> {
    require(state.is_object(), "frame state must be an object")?;
    require(state["screen"].is_object(), "state screen must be an object")?;
    require(state["screen"]["kind"].is_string() && state["screen"]["name"].is_string(), "screen needs a kind and name")?;
    require(state["screen"]["options"].is_array(), "screen options must be an array")?;
    nullable(&state["screen"], "selection", Value::is_object)?;
    require(state["player"].is_object(), "state player must be an object")?;
    for field in ["inventory", "map", "combat"] { nullable(state, field, Value::is_object)?; }
    nullable(state, "terminal", Value::is_string)
}

fn nullable(object: &Value, field: &str, check: impl FnOnce(&Value) -> bool) -> Result<(), ReportError> {
    let value = object.get(field).ok_or_else(|| invalid(&format!("missing {field}")))?;
    require(value.is_null() || check(value), &format!("invalid {field}"))
}

fn validate_refs(value: &Value) -> Result<(), ReportError> {
    match value {
        Value::Array(values) => { for value in values { validate_refs(value)?; } }
        Value::Object(fields) => {
            for (key,value) in fields {
                if matches!(key.as_str(),"id"|"ref"|"previousFrameId"|"combatId"|"parentActionId"|"optionKey") {
                    require(value.is_null() || value.is_string(), "identifiers and references must be strings")?;
                }
                validate_refs(value)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn invalid(message: &str) -> ReportError { ReportError::Invalid(message.into()) }
fn require(ok: bool, message: &str) -> Result<(), ReportError> {
    if ok { Ok(()) } else { Err(invalid(message)) }
}

/// Load display JSONL, keeping only newline-terminated records. A torn/missing footer
/// means interrupted capture, never an inferred terminal gameplay result.
/// This is deliberately separate from the original-reference trace-v1 parser.
pub fn read_report<R: BufRead>(input: R, max_bytes: usize) -> Result<Value, ReportError> {
    let limit = u64::try_from(max_bytes).ok().and_then(|limit|limit.checked_add(1))
        .ok_or_else(|| invalid("invalid report size limit"))?;
    let mut bytes = Vec::new();
    input.take(limit).read_to_end(&mut bytes)?;
    require(bytes.len() <= max_bytes, "report size limit exceeded")?;
    let mut lines = bytes.split_inclusive(|byte| *byte == b'\n');
    let first = lines.next().ok_or_else(|| invalid("missing report header"))?;
    require(first.ends_with(b"\n"), "incomplete report header")?;
    let header: Value = serde_json::from_slice(first)?;
    validate_header(&header)?;
    let mut history = History::default();
    let mut frames = Vec::new();
    let mut footer = None;
    let mut warnings = Vec::new();
    for line in lines {
        require(!history.finished, "data after footer")?;
        if !line.ends_with(b"\n") {
            warnings.push("Incomplete final record ignored".to_owned());
            break;
        }
        let record: Value = serde_json::from_slice(line)?;
        history.accept(&record)?;
        match record["record"].as_str() {
            Some("frame") => frames.push(record["frame"].clone()),
            Some("footer") => footer = Some(record),
            _ => {}
        }
    }
    let footer = footer.unwrap_or_else(|| {
        warnings.push("Missing footer: capture interrupted; gameplay outcome unknown".into());
        json!({"capture":{"status":"interrupted","reason":"missing complete footer"},"game":{"outcome":null,"boundary":null}})
    });
    Ok(json!({"schemaId":header["schemaId"],"schemaVersion":header["schemaVersion"],
        "source":header["source"],"run":header["run"],"catalog":history.catalog,
        "frames":frames,"capture":footer["capture"],"game":footer["game"],"warnings":warnings,
        "assets":{"byKey":{},"blobs":{},"sources":{},"attribution":[]}}))
}
