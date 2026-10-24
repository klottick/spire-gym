# Spire Gym offline run viewer

Status: **written specification approved by the user on 2026-10-23**. The user approved adapting the
previously selected viewer to Spire Gym, connecting existing combat replay first
and defining the full-run recording interface without implementing missing game
mechanics. This file does not authorize implementation, installation, or publication.

## 1. Intent and retained requirements

Make a recorded simulation understandable without reconstructing it from logs.
Combat is the main readability problem, but the viewer must also represent the
map, events, shops, rewards, rest sites, selections, inventory and run outcomes.

Retain the approved tactical-overview direction:

- One slide per actual chosen action, with automatic consequences grouped with it.
- Static, utilitarian presentation: no animation, autoplay, gameplay controls,
  alternative-action exploration or invented decision rationale.
- Original StS1 artwork when explicitly supplied; names and numbers when missing.
  No replacement illustrations, committed proprietary images or runtime downloads.
- Relics, counters and all known potion slots across the top; player on the left,
  played card/action centrally, enemy roster to the right, hand and pile inspection
  below, ordered resolution information in a separate inspector.
- Keep the played card visible after it leaves the hand, including subsequent
  selection and End Turn frames. Distinguish resolved enemy moves from next intent.
- Show actual post-resolution HP, a static hatched net-loss segment, separate
  Block, and readable before/after values. List gross damage and healing separately;
  neither absorbed Block nor overkill is HP lost.
- Previous/Next, scrubber, arrow keys, and jumps by act, floor, combat and turn.
  Preserve keyboard focus and stack panels on narrow screens.
- Export one self-contained HTML document that opens with `file://` offline.

This is optional inspection tooling for the headless gym, not a playable client,
training system, simulator renderer dependency or replacement for public APIs.

## 2. Repository and actual integration baseline

The correct repository is `klottick/spire-gym`, checked out as `spire-lab`.
The isolated viewer worktree is
`/home/klottick/.local/share/opencode/worktree/503d4a/run-viewer`.
The other repository's merged C++ recorder is **not** this implementation and
will not be transplanted, reverted or treated as a Spire Gym feature.

There are two distinct source baselines:

1. The viewer worktree was created from committed main revision `c0746b7`.
   `cargo test --workspace --locked --offline` passed **169 tests** there.
   That revision has supplied-entry combat and `spire_env::replay`, but no run
   engine. CPython 3.12 exists; pytest/NumPy were not available in the inspected
   interpreter, and no Python dependencies were installed or Python tests run.
2. The shared tracker identifies the current run-delivery workspace at
   `/home/klottick/.local/share/opencode/worktree/503d4a/full-run-gym-poc-20261022`.
   Its inspected committed tip was `5b1cd19` on `feat/run-act2-prefix`.
   Public `Run::decision`, `Run::step`, `Run::public_map`, `CombatPublic` and the
   documented M1–M6 producers already exist there. M6 is accepted only at its
   declared Ironclad/A0 captured-content prefix through floor 26, not complete
   Act 2, full standard content or Heart runs. Its documented baseline has seven
   older failing core tests; the legacy 169-test result is not evidence for it.

**Integration-base decision for implementation:** use the existing accepted native
run producer line, pinned and coordinated with its owner before shared-file edits,
rather than rebuilding it on obsolete main. Preparing the viewer branch from an
agreed committed base must not merge, copy or discard another workspace's
uncommitted files. The current viewer HEAD has not been changed for this decision.
If the accepted producer tip advances, identify its exact public API and baseline
test results before implementing against it.

The first real end-to-end adapter is the existing supplied-entry combat replay.
The same report contract can then consume the existing accepted run-prefix producer.
The viewer does not implement later acts, broaden supported content, change gym
milestones, claim Heart completion or restrict the gym's eventual four-character,
Ascension 0–20 scope. A captured prefix must say it is a prefix.

## 3. Existing interfaces to reuse

### Combat episodes

`spire_env::EpisodeRecord` retains an immutable `EpisodeJob`, accepted action
indices, status, schema/content identifiers and optional fault attempt.
`spire_env::replay` validates compatibility and re-executes from the initial job
through the real native menu/transition path. It is not snapshot import.

Reuse that validation and replay path. Do not implement a second validator or
simulator, deserialize private job internals from arbitrary JSON, or turn numeric
policy observations into guesses about card identities and effects. A minimal
opt-in replay visitor can expose semantic actions, detached public values and
resolved effect observations while retaining existing `replay` behavior.

Python's existing reproduction dictionaries are privileged evidence, not a new
restore protocol. Supporting arbitrary saved reproduction-dictionary import is
not required for the first viewer delivery.

### Run progression

The accepted producer exposes `PublicDecision` with current resources, permanent
cards, candidates, selections, optional `CombatPublic`, actual combat progress,
room information and terminal state. `RunTransition` supplies committed events
and reward; `Run::public_map` omits hidden outcomes and scheduling history.

An opt-in recording wrapper captures the public decision and chosen typed action
before calling the existing `Run::step` **once**, then captures its actual result.
Existing callers still choose actions. The recorder is not a policy and does not
auto-select a reward, target, path or card. Shared engine hooks remain narrowly
scoped observation additions, coordinated with the run owner.

Use `CombatPublic` while combat owns resources. Do not substitute the persistent
checkpoint's stale HP, potion inventory or relic counters. Preserve terminal
combat evidence before automatic settlement removes the live combat, as well as
the actual settled run state; do not manufacture another agent action for cleanup.

## 4. Architecture and ownership

1. **Native observation seam:** opt-in resolved-effect observations in the existing
   Rust combat resolver. No recorder or image ownership inside cloned gameplay
   state, no changes to RNG, rule calculations, action validity or queue ordering.
2. **Report adapter/recorder:** a focused optional `spire-report` crate converts
   native public values, accepted actions and effects into display-only records.
   It supports combat replay and the run producer contract through separate small
   adapters, sharing the display schema rather than gameplay logic.
3. **Offline exporter and artwork manifest:** package display JSONL, maintained
   HTML/CSS/JavaScript sources and referenced local image bytes. Keep viewer
   resources outside the core simulator. Prefer existing dependencies and standard
   library facilities; any new dependency or installation needs explicit authority.
4. **Browser viewer:** plain static scripts render recorded values only. Reuse
   appropriate original viewer layout/rendering code after checking its wire
   assumptions; do not import its C++ IDs, RNG, seed conversion or engine bindings.

A native reporting example demonstrates an actual existing combat episode and,
once integrated against the accepted producer, an actual captured run prefix.
A packaging command takes display JSONL and writes HTML; it does not execute a
game from a purported saved state. Existing examples, CLI version behavior,
Gymnasium return values and Python numeric observations remain unchanged.

The initial workflow must be runnable through native Rust and standalone HTML
without making Python-extension rebuilding a prerequisite. A later thin Python
convenience entry point may call the same native exporter; it is not a second
recording implementation and is not required for initial acceptance.

The viewer owns its reporting crate, static resources, asset manifest/preparation
helper, tests and documentation. It does not own the relic/card datasets, content
mechanics, milestone plans, training code or other workers' source files.
In particular, `docs/research/relics/` in `spire-gym` is reserved for the separate
relic-reference task, not a viewer input or integration deliverable. Its earlier
reservation in the other repository is withdrawn; no viewer work changes it.

## 5. Display contract: `spire-report-v1`

Use a separately named/versioned JSON Lines report protocol. Do not repurpose or
relax the strict original-reference `trace-v1` parser, fingerprint chain, RNG-word
rules or provenance requirements.

Records are `header`, optional immutable `catalog` additions, ordered `frame`
records, and a `footer`. Flush complete records incrementally. Packaging can load
one report in memory; recording does not retain a full simulation history itself.

### Header and outcome separation

- Schema ID/version; producer revision/content identifier and declared profile.
- Source kind: `combat_episode`, `run_prefix`, `run` or `authored_fixture`.
- Character, ascension and known caller configuration; absent fields are explicit.
- User-provided gym seed as lossless text with its declared encoding/contract,
  not an StS game-code conversion or claim of original Java RNG parity.
- Evidence grade: source-grounded, not original-runtime validated.

Capture completion is separate from gameplay outcome. A complete recording of an
M6 prefix is **not** victory, death, Act 2 completion or a finished run. Record
native terminal reasons separately, preserve the actual prefix boundary, and
label user budget stops as unfinished. An empty action menu alone is not proof
of a terminal outcome. Faulted/abandoned/active episodes retain those distinctions.

### Frames and historical context

Each frame has a consecutive ID, previous-frame reference, kind, known act/floor/
combat/turn context, chosen action, pre-action offers, settled display state and
ordered effects. Initialization and automatic boundaries have explicit non-action
labels. Selection continuation references the initiating card/offer.

Store large IDs as strings. Use native card instance/origin IDs and combat-scoped
enemy lifetime IDs; a replacement or summon cannot inherit a dead actor's history.
Keep logical position separately from identity. Retain escaped/replaced/defeated
actors as historical targets without presenting their old intent as upcoming.

State includes public player resources, keys, deck, relics/counters, potion slots,
room/screen options and known public map information. Combat additionally includes
actor identities, powers, hand/piles, current costs/upgrades, pending selection,
known character state and public Stasis holdings where present in the producer.
Do not invent unavailable values or treat `null` as zero/empty.

Capture exact candidate identities and their structured pre-action values,
including prices and selection eligibility. Do not identify the chosen option by
name matching, absent-ID equality, current array position or later reconstructed
stock. Preserve offers after purchases, reward removal and screen transitions.

### Ordered effects

Observe resolved work, not queue scheduling alone. Preserve known source/target,
move/card identity, before/after values, incoming damage, absorbed Block, actual
HP lost, known overkill, healing, status changes and important inventory changes.
End Turn retains the enemy move group until its queued consequences resolve.
Unattributed work stays explicitly unattributed, not guessed from a net delta.

Damage followed by healing and terminal healing remain separate observations;
automatic settlement cannot erase a lethal hit or apply Burning Blood twice.
Input-selection boundaries close the current capture safely and resume under the
subsequent real selection without duplicating effects. Faults may leave partial
work: preserve observed effects and the last known state with its validity label,
not a fabricated settled snapshot.

## 6. Privacy and simulator invariance

Default report contents follow public display projections, not `Debug` dumps,
private snapshots or reproduction-record internals. Exclude RNG roots/cursors,
unseen draw order, future AI selections, hidden room contents and future loot.
Unknown draw order stays unknown; visible pile inspection uses only public data.
Caller metadata is kept separate from policy observations and labelled as such.

Record only committed actions of the selected episode/run. Do not record search
rollouts, vector replacement episodes as if they were terminal resources, or a
second execution of a chosen action. Fault metadata must not silently become
success. Export failures must not mark a healthy live gym pool as gameplay-faulted.

Ordinary stepping with recording off must serialize no state, allocate no trace
data or change existing output. Compare observed/unobserved action histories,
public boundary values, native state/RNG equality and terminal results. Measure
disabled overhead on an unchanged fixed workload rather than invent a timing gate.

## 7. Export, artwork and failure handling

- The final HTML embeds inert JSON, CSS, scripts and referenced PNG/JPEG/WebP data
  URLs. No external scripts, fonts, network calls or browser simulator endpoint.
- Escape script-closing sequences, `<`, `>`, `&` and Unicode line separators.
  Render untrusted names/descriptions with DOM text APIs, not executable markup.
- Reject unsafe manifests, absolute/traversing paths, symlink escapes and active
  SVG. Deduplicate exact image bytes and embed only referenced available art.
- Acquire/import optional original images only through an explicit preparation
  operation. Keep images/private game files outside source control and preserve
  Mega Crit ownership and source attribution. No build-time download requirement.
- Missing art is a label, not a broken image, replacement illustration or failed
  simulation. The old 23-image example catalog can seed acquisition metadata but
  does not establish complete artwork coverage for Spire Gym.
- Validate output destinations before capture/export. Never overwrite existing
  reports or traces; publish through an exclusive same-directory temporary file.
- Missing footer/incomplete final record means interrupted capture. Retain complete
  records; reject malformed interior records, inconsistent references and unsupported
  versions rather than silently repairing history. This behavior belongs only to
  the display protocol, not the strict reference trace reader.
- A handled export failure returns failure and keeps recoverable display records.
  A fatal process crash may leave only the trace; finished HTML is not promised.

## 8. Acceptance and review

1. Capture a real native combat replay with card actions and End Turn, opening and
   terminal states, stable card IDs, actual HP/Block/healing and no hidden state.
2. On the agreed run-producer base, capture an existing normal-start run-prefix
   example through actual combat and non-combat decisions, including multi-enemy
   frames where that profile produces them. Preserve its real cutoff/coverage.
   Authored fixtures supplement but do not replace this evidence.
3. Verify duplicate cards, selections, summons/replacements, multi-hit/multi-target
   damage, fully/partially blocked hits, overkill and damage followed by healing
   using supported native cases. Future-only mechanics get clearly labelled schema
   fixtures, not a claim of implemented game behavior.
4. Check all screen renderers, exact historical selections/prices, lifecycle labels,
   prefix/unfinished/fault/death outcomes, unknown values and missing artwork.
5. Open actual exported HTML through `file://` with network disabled: navigation,
   pile inspection, current card metadata, original-image decoding, safe text,
   desktop/mobile layout, zero external requests and no page errors.
6. Run existing relevant native suites and invariance checks. Establish the agreed
   source base's actual failures by name/message; do not conceal inherited failures
   behind a new green focused suite or call the whole workspace green.
7. Use focused ordinary tests and one integrated independent review, not a new
   verification platform, content audit or review swarm. Keep original-runtime
   validation, later gym milestones and dataset integration outside this task.

### Review gate

The user reviews this written specification before implementation planning.
The implementation plan must pin the agreed native base, concrete shared-hook
ownership, CLI/API interfaces, test commands and existing baseline failures.
The user then reviews that plan and selects execution before product code starts.
No commits, pushes, installs or changes to another workspace are implied by this
specification. The existing approved visual direction does not need to be chosen
again.

### Preparation checklist

- [x] Confirm task: run viewer only, correct repository, relic dataset untouched.
- [x] Create isolated native-managed worktree with user consent.
- [x] Run legacy committed-main Rust baseline; disclose Python environment limit.
- [x] Inspect existing combat replay and current accepted public run interfaces.
- [x] Write specification and self-review scope, provenance, failures and interfaces.
- [x] User reviews this written specification.
- [x] Write and review the concrete implementation plan (approved 2026-10-23).
