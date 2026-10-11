# `trace-v1` — reference trace wire protocol

Version: **1** (the only version the reference reader/writer accept).
Scope: **game-independent wire protocol** for reference Slay the Spire combat
traces. This document defines the schema, the strict record ordering, and the
validation rules that the Rust reader (`spire_core::trace`) and the Java
writer (`spirelab.protocol`) must both satisfy.

> **Status.** This is the **portable protocol stage** of Task 2. The wire
> format is implemented and round-trip-tested cross-language (Java writer →
> Rust reader). A **valid** trace can be produced and consumed by the portable
> writer/reader today. A **reference** trace — one whose `reference_build`
> hashes, RNG seed words, and state values come from the actually pinned game
> at a verified capture time — is still **PENDING** user evidence (see
> `reference-build.json` and `capture-guide.md`). The protocol-test fixtures
> under `validation/fixtures/protocol/` are **not** reference oracles.

## 1. File framing

- Encoding: **UTF-8**.
- Shape: **JSON Lines** — exactly one JSON object per line, lines terminated by
  `\n`, the final line included. No blank lines, no trailing data after the
  last newline.
- Size: the reference reader enforces a bounded read (default **64 MiB**);
  callers pass an explicit limit. Oversized input is a hard error.
- Integrity: records are chained with a **SHA-256** fingerprint of the
  complete preceding line (see §6). This is the integrity mechanism; there is
  no separate per-line digest header.

## 2. Record set and strict ordering

A trace is the ordered sequence, with exactly these records in exactly this
order (any other order is a hard error):

1. **one** `header` (line 1, carries no fingerprint)
2. **one** `initial` (the starting canonical state)
3. **zero or more** `step` records, with their `index` strictly increasing
   `0, 1, 2, …` with no gaps and no repeats
4. **one** `end`

Each non-header record carries the SHA-256 fingerprint of the full preceding
line. The reader verifies the chain end-to-end; the writer enforces ordering
and refuses to append out of sequence.

Every record has a `"record"` tag (`"header"` / `"initial"` / `"step"` /
`"end"`). The reader and writer both use **deny-unknown-fields**: any field
outside the per-record schema (or any nested object outside its schema) is a
hard error — the protocol never silently drops a mismatching field.

## 3. Record schemas

### 3.1 header (line 1)

```
{
  "record": "header",
  "version": "1",
  "game": { "id": string, "character": string, "ascension": uint },
  "rng":   { "seed_words": string[] },
  "provenance": {
    "tool": string,
    "tool_version": string,
    "captured_at_utc": string | null
  },
  "reference_build": {
    "game_sha256": string,    // 64 lowercase hex
    "loader_sha256": string   // 64 lowercase hex
  }
}
```

- `version` must be exactly `"1"`.
- `game.id` / `game.character` non-blank; `game.ascension` a non-negative int.
- `rng.seed_words`: one or more words, **each exactly 16 lowercase hex
  digits** (a lossless 64-bit word). A numeric value here is a hard error.
- `reference_build.*` must each be exactly 64 lowercase hex. In a real
  reference capture these are the SHA-256 of the pinned game jar and the
  pinned loader. In a well-formed **protocol-test** fixture they may be
  synthetic (e.g. `0…0`, `f…f`) but must still be well-formed.
- `provenance.captured_at_utc` may be `null` (synthetic/placeholder fixtures).

### 3.2 initial

```
{
  "record": "initial",
  "fingerprint": string,       // sha256(header line)
  "state": <CanonicalState>    // see §4
}
```

### 3.3 step

```
{
  "record": "step",
  "fingerprint": string,       // sha256(preceding line)
  "index": uint,               // 0-based, strictly increasing, no gaps
  "action": <SemanticAction>,  // see §5
  "expected": <CanonicalState> // canonical state AFTER the action
}
```

### 3.4 end

```
{
  "record": "end",
  "fingerprint": string,       // sha256(preceding line)
  "outcome": "victory" | "defeat"
}
```

## 4. Canonical state

`state` (initial) and `expected` (step) are a **canonical state** object. The
top-level keys are an agreed vocabulary shared with the Rust state model; an
unknown key is a hard error (never silently dropped).

**Top-level state keys** (order-insensitive; all optional per record, but a
given well-formed reference should carry the full supported set):

| key | type | meaning / units |
| --- | --- | --- |
| `player` | object | the acting player (Ironclad in 1a) |
| `enemies` | array(object) | live enemies present |
| `cards` | array(object) | card instances in play (supported: Strike/Defend/Bash, base/upgrade) |
| `hand` | array(string) | card ids in hand |
| `draw` | array(string) | draw pile, top-first |
| `discard` | array(string) | discard pile |
| `exhaust` | array(string) | exhausted pile |
| `in_play` | array(string) | cards currently in play (e.g. played auras) |
| `rng` | object | RNG stream position(s); see §4.1 |
| `queue` | array | pending action/card queue (empty at a valid capture entry) |
| `pending` | array | pending effects / resolutions |
| `phase` | string | current macro phase (`"player_turn"` / `"enemy_turn"` / …) |
| `turn` | uint | turn counter |
| `outcome` | string \| null | combat outcome once resolved, else null |
| `public_knowledge` | object | the **publicly observable** subset (what a policy may rely on) |
| `ruleset` | object | character / ascension / mode descriptor |

> `public_knowledge` is carried **separately** so that policy-visible history
> is never inferred from privileged fields. A fixture that requires an
> unsupported mechanic (potions, maps/events, other heroes, unsupported relics)
> must be **rejected**, not coerced.

### 4.1 `player` object

| key | type | units |
| --- | --- | --- |
| `hp` / `max_hp` | int | hit points |
| `block` | int | block |
| `energy` / `energy_per_turn` | int | energy |
| `base_draw` | int | cards drawn per turn |
| `relics` | array(string) | owned relics (supported: e.g. Champion) |
| `statuses` | array(string) | active player statuses (e.g. `weak:2`, `vulnerable:3`) |

### 4.2 `rng` object

64-bit RNG words/seeds are encoded **losslessly as 16-digit lowercase hex
strings** — never as JSON numbers (which would be a type error on read and
could lose precision).

| key | type | meaning |
| --- | --- | --- |
| `stream_id` | string | identifier of the RNG stream this state observes |
| `position` | uint | monotonically increasing operation counter for that stream |

> **Do not invent independent streams where the game shares one.** The actual
> stream roster/algorithm of the pinned game is pending user capture evidence.
> `stream_id`/`position` are the wire slot for it; a real reference capture
> must populate them from the isolated reference RNG export, not from a
> hand-guessed model.

### 4.3 `enemies` entries

Each enemy object carries `id` (trace-local, stable within the trace),
`hp`/`max_hp`/`block`, its `intent` (kind, strength, applied `statuses`),
`powers` (e.g. Rally with `stacks`/`max_stacks`), and `phase`.

## 5. `SemanticAction`

Wire-encoded as an object with a `"type"` discriminator:

| type | fields | example |
| --- | --- | --- |
| `"play_card"` | `card` (trace-local card id), `target` (trace-local target id) | `{"type":"play_card","card":"strike-1","target":"cultist-1"}` |
| `"end_turn"` | none | `{"type":"end_turn"}` |

- Identities are **trace-local** — they are stable string ids minted within the
  trace, not heap/UUID addresses and never privileged RNG state.
- `play_card` must carry non-blank `card` and `target`; `end_turn` carries only
  its `type`. Any other `type` is rejected.
- The coordinator matches these against the **observed** play/end-turn events at
  the pinned game's update boundary; it does **not** issue actions.

## 6. Fingerprint chain (integrity)

For each record after the header:

```
fingerprint = sha256_hex( utf8_bytes( full_preceding_line_without_trailing_newline ) )
```

`sha256_hex` is 64 lowercase hex chars. The chain starts at the header, so the
`initial` line's fingerprint binds the header, and every step binds the record
before it. Tampering with any single line breaks every link after it. The
reader reports `BadFingerprint` on the first broken link; the writer computes
the link from the last line it wrote, so out-of-order or re-ordered appends
cannot fabricate a valid chain.

## 7. Reader validation catalog (Rust `spire_core::trace`)

The read side is **fail-closed**. Represented error classes:

| condition | error |
| --- | --- |
| file larger than the given read limit | `Oversized` |
| not UTF-8 / empty / no terminating newline / blank line | `Utf8` / `Empty` / `Truncated` / `BlankLine` |
| missing end / record order violated | `MissingRecord` / `UnexpectedRecord` |
| bad JSON or unknown field | `Json` |
| `version` != `"1"` | `UnknownVersion` |
| seed word not 16-hex, or a number | `BadRngWord` / `Json` |
| reference-build hash missing/malformed | `MissingReferenceBuild` |
| broken fingerprint link | `BadFingerprint` |
| step index gap/repeat/out-of-order | `StepIndexOutOfOrder` / `DuplicateStep` |
| NaN/Infinity anywhere a number is expected | `NonFiniteQuantity` |
| state/player field outside the vocabulary | `UnsupportedStateField` / `UnsupportedPlayerField` |
| action kind unknown / missing identity | `UnsupportedAction` |
| outcome not victory/defeat | `BadOutcome` |

## 8. Writer/coordinator contract (Java `spirelab.protocol`)

- `TraceWriter` appends records in the strict order, computes and enforces the
  fingerprint chain, and refuses out-of-sequence use.
- `CaptureCoordinator.afterGameUpdate(GameView)` is **observe-only**:
  - first clean boundary (no action) → writes `header` + one `initial`;
  - each changed boundary with an observed action → writes exactly one `step`;
  - **repeated observations at the same unchanged boundary are deduplicated —
    nothing is written and the live RNG is never sampled**;
  - `endCombat(Outcome)` writes the single `end`.
- `BoundaryEvent.rngOperationCount()` exists **only** so a fake view can count
  accidental live-RNG sampling; passive capture must never call it.

## 9. Portability and cross-language agreement

- The Rust reader and the Java writer implement the **same** wire format. A
  sample written by the Java `TraceWriter` must parse with the Rust
  `spire_core::trace::read_trace` (verified by the `interop_*` round trip and
  the `SPIRE_JAVA_TRACE` interop test).
- Integer bounds: the Java side rejects integers above the unsigned 64-bit
  range to match the Rust reader's integer domain.
- Neither side depends on Python, tensor libraries, renderers, or training
  tooling.

## 10. What this file does NOT claim

- It does not document the pinned game's real RNG stream/algorithm (pending).
- It does not contain a verified reference capture or its hashes (pending).
- It does not authorize treating any `protocol-test` fixture as a game-correctness oracle.
