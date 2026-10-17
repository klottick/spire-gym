# Source-first Rust Combat Core Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Independently implement the supplied-entry Ironclad A0 combat slice in Rust, with exact project randomness, reviewed mechanics, owned replay and test-only Python differential checks.

**Architecture:** An owned `spire_core::combat::Combat` resolves typed actions/effects to decision, terminal or diagnostic fault checkpoints. Focused native modules separate state/content, randomness, calculations, scheduling, public views and replay. Python participates only in differential tests; original captures are a later fidelity-promotion gate.

**Tech Stack:** Existing Rust 1.99.0 toolchain/edition 2024, existing serde/serde_json/sha2; proposed `hmac = "=0.12.1"` for the unchanged HMAC-SHA256 contract. Existing CPython 3.12 and pytest 9.1.1 for test orchestration. No Python/native bindings, tensor code or game runtime in the core.

**Spec:** [Approved source-first Rust combat design](../specs/2026-10-16-source-first-rust-combat-design.md), written-spec approval received 2026-10-16.

**Current status (2026-10-16):** User plan/start approval, including the named HMAC dependency, was received and executed. Tasks 1–8 and their independent reviews, correction reviews and whole-slice review are done. Both final Minor fixes passed fresh scoped re-review. Fresh controller verification passes 124 native tests per workspace/release run and 830 full Python tests, including 95 differential tests. The core-only handoff is complete; no commit, merge, push or cleanup was performed. The normative task checkboxes below retain the approved dated plan, not retroactively invented execution history; the completed task ledger and actual command evidence are retained locally. Historical Task-7 TDD qualifications and failed comparisons remain disclosed.

## Global Constraints

- “Ironclad A0; base/upgraded Strike, Defend and Bash; Burning Blood.” One Cultist or ordinary Jaw Worm, supplied first-ready entry only.
- “source-grounded, not original-runtime validated”. Original capture prerequisites for starting Rust are superseded, not the later fidelity evidence requirements.
- `RNG_CONTRACT_ID = "spire-lab-rng-hmac-sha256-v1"`; snapshot schema **2**, content identity `spire-lab-combat-reference-v2`.
- “No initial shuffle, HP sampling or draw is performed by combat construction.” Initial player block/powers zero/empty; full-health enemy, first move selected once; input limits remain those in spec section 2.
- “Avoid whole-state copying on every action.” Snapshot and detached-view copying are explicit operations, not automatic step results.
- “attempted dispatch 100,000 is counted and faults before its payload.” Dispatch 99,999 may complete; safe-point resume preserves the same step's count.
- “faulted snapshots are diagnostic only.” Invalid actions are atomic rejections; resolution faults preserve partial state/first fault and are not losses.
- “No serialized snapshot wire protocol or direct loading of Python in-memory objects is introduced.” The interchange below is test-only semantic comparison, not snapshots or trace-v1.
- No original methods, mechanical translations, private artifacts/evidence, rendering, UI, startup, additional content, pools, bindings, learner or performance framework in product code.
- Preserve Python RNG implementation/tests/fixed fixture and all historical evidence. Do not weaken Python tests or repair historical provenance to obtain agreement.
- Use the existing isolated worktree; do not discard unrelated changes. No staging/commits/pushes, installations, permission changes, cleanup or new original access/execution without explicit scope approval.
- **Named dependency proposal:** only native `hmac = "=0.12.1"` and required transitive lock entries. Obtain explicit approval for that addition with plan/start review before fetching/resolving it; keep all existing locked package versions. No toolchain/system installation or unrelated dependency upgrade.

## Review Focus

1. Arbitrarily large/negative Python card identities must round-trip without integer narrowing or aliases such as `01`/`-0`; Tasks 2/8 exact opaque-handle and interchange tests.
2. A late invalid action, wrong target or malformed interchange value must leave the state reached by earlier valid actions unchanged; Tasks 5/8 snapshot-before-rejection tests.
3. Exhaustion during rejection, fallback or a multi-swap recycle must retain only the correct stream's latest cursor, without inventing a choice or committing partial pile movement; Tasks 1/4/6/7 tests.
4. Restoring pending turn work must retain lifecycle flags, damage references and the same dispatch budget, rather than admitting a new player choice; Tasks 2/4/7/8 safe-point tests.
5. Diagnostic helpers must not contaminate policy observations or silently omit inconvenient checkpoint fields, and a missing/stale Rust test executable must not produce a false differential pass; Tasks 5/8 noninterference and harness-failure controls.

---

## File map and implementation discipline

All new native modules are under `crates/spire-core/src/combat/`:

| File | Responsibility / owning tasks |
|---|---|
| `mod.rs` | Public re-exports, `Combat` methods; add module declarations as their files land (1–8) |
| `rng.rs` | Native project stream contract (1) |
| `content.rs` | Finite card/enemy/move/power identities and static statistics (2–3) |
| `model.rs` | Owned setup/state/work/error values and structural/runtime validation (2, extended by owners) |
| `snapshot.rs` | Detached schema-2 replay and deterministic diagnostics (2, 4) |
| `rules.rs` | Bounded arithmetic and exact history-conditioned selection (3) |
| `engine.rs` | Supplied entry, action preflight, dispatch/resolution and first fault (2, 4–5, 7) |
| `effects.rs` | Damage/block/powers, refresh, card completion, terminal work (4–5) |
| `view.rs` | Detached public values and hand-order legal actions (5) |
| `piles.rs` | Immediate discard membership, draw/recycle (5–6) |
| `turns.rs` | Current enemy execution, later selection, round/player scheduling (7) |
| `tests/mod.rs`, `tests/support.rs` | Test declarations and explicit authored setup/checkpoint helpers (1–8) |
| `tests/rng_contract.rs`, `tests/entry_replay.rs`, `tests/rules_ai.rs` | Tasks 1–3 tests |
| `tests/resolution.rs`, `tests/actions_views.rs`, `tests/piles.rs`, `tests/turns.rs` | Tasks 4–7 tests |
| `tests/whole_combat.rs`, `tests/differential.rs` | Complete-combat checks and cfg(test)-only interchange entry (8) |

Also modify `crates/spire-core/src/lib.rs` to export `combat` alongside unchanged
`trace`; modify `crates/spire-core/Cargo.toml`/`Cargo.lock` only for the named HMAC
dependency in Task 1. Task 8 creates
`tests/python/combat/rust_diff_support.py` and `tests/python/combat/test_rust_differential.py`.
Documentation status updates belong to Task 8 after verified implementation.

Every task follows write tests → observed RED → minimal implementation → GREEN →
exact source/test handoff and fresh review. A missing API compile error is valid
RED; a dependency/network failure, zero selected tests or unrelated baseline
failure is not. No empty/stub success, ignored failing test or fabricated command
history. Stage-dependent work kinds explicitly return unsupported until their
owner lands; this is not a completed engine until all eight tasks pass review.

Run baseline/RED/GREEN/verification Cargo commands with child-only
`CARGO_NET_OFFLINE=true` (or append `--offline`); the sole exception is Task 1's
explicitly approved dependency-resolution command. A missing cache/toolchain is
an execution prerequisite to surface, not permission for automatic installations.

Before Task 1, record actual baselines with `cargo test --workspace --locked` and
`.venv/bin/python -B -m pytest -q`, plus HEAD/index/status and the fixed fixture
hash. The last recorded Python count was 735, not a substitute for this fresh run.
Use fresh exclusively created local execution/report directories under
`.superpowers/sdd/2026-10-16-source-first-rust-plans/`; preserve failures and exact
per-task source snapshots since no new commits are authorized. Do not import raw
private originals for review or silently broaden the current worktree's scope.

## Shared native types and boundaries

Declare public setup/value types in `model.rs`, finite content enums in
`content.rs`; re-export them from `combat`. Internal `State` and its fields are
visible only within `combat`. All owned values support `Clone`, `Debug`, `PartialEq`
and `Eq` where meaningful; observation/transition enums may also be `Copy`.
Finite content, Domain, Partition, phase/status/outcome/fault enums are `Copy`.

| Value | Exact fields / variants |
|---|---|
| `CardHandle(String)` | Private canonical signed-decimal text, parsed by `CardHandle::parse(&str) -> Result<Self, InputError>`; `as_str(&self) -> &str`. Grammar `0` or `-?[1-9][0-9]*`, no magnitude limit/arithmetic. This preserves Python's arbitrary integer identities without a big-integer dependency. |
| `EnemyHandle(String)` | Private nonempty text other than reserved `player`; `parse`/`as_str` as above. Public owner need not equal a private RNG scope suffix. |
| `CardKind`, `EnemyKind`, `MoveId`, `PowerId` | `Strike/Defend/Bash`; `Cultist/JawWorm`; `Incantation/Attack/Chomp/Thrash/Bellow`; `Strength/Ritual/Vulnerable`. Reject incompatible enum combinations in validation. |
| `CardInstance` | `instance_id: CardHandle, card_id: CardKind, upgraded: bool` |
| `PowerState`, `PowerEntry` | `amount: i32, skip_first: bool, monster_source_grace: bool`; entry `id: PowerId, state: PowerState` |
| `PlayerState` | `hp,max_hp,block,energy,max_energy,draw_per_turn: i32`; `powers: Vec<PowerEntry>` |
| `EnemySetup` | `instance_id: EnemyHandle, enemy_id: EnemyKind`; `hp,max_hp,block: i32`; `powers: Vec<PowerEntry>, intent_id: MoveId, move_history: Vec<MoveId>` |
| `Piles` | `hand,draw,discard,exhaust: Vec<CardInstance>` |
| `CombatRng` | `contract_id: String, shuffle: StreamState, enemy_ai: StreamState, enemy_instance_id: EnemyHandle` |
| `Scenario` | `encounter_id: EnemyKind, player: PlayerState, enemies: Vec<EnemySetup>, piles: Piles, rng: CombatRng` |
| Runtime `EnemyState` | Setup fields plus `selected_move: MoveId, damage_entries: BTreeMap<MoveId, DamageEntry>, intent_damage: Option<i32>`; `DamageEntry { base: i32, output: i32 }` |
| `Phase`, `Outcome`, `TransitionStatus` | `Decision/PlayerAction/EnemyTurn/RoundEnd/PlayerStart/Terminal`; `Victory/Death`; `Decision/Terminal/Fault` |
| `FaultKind`, `FaultRecord` | `Unsupported/Invariant/EffectGuard/RngExhausted`; record `kind: FaultKind, message: String, resolved_effects: u32` |
| `InputError` | `Invariant(String), Unsupported(String)` |
| `StepError`, `AccessError`, `RestoreError` | `InvalidAction/NotReady/Terminal/Faulted`; `NotReady/Faulted`; `Compatibility/Faulted/Invalid(InputError)` respectively |
| `Action` | `PlayCard { card_instance_id: CardHandle, target_id: Option<EnemyHandle> }, EndTurn` |
| `Transition` | `status: TransitionStatus` only; state stays owned by `Combat` |
| `Actor`, `Source`, `DamagePayload` | `Player/Enemy(EnemyHandle)`; `None/Card(CardHandle)/Actor(Actor)`; `Fixed(i32)/EnemyMove(MoveId)` |
| `Effect` | `source: Source, target: Option<Actor>, kind: EffectKind` |
| `EffectKind` | `Damage(DamagePayload), GainBlock(i32), ApplyPower { id: PowerId, amount: i32 }, ReducePower { id: PowerId, amount: i32 }, RemovePower(PowerId), FinishCard, Decision, Terminal(Outcome), Draw(i32), Recycle, DiscardHand, EndPlayerTurn, MonsterStart, EnemyMove, RollMove, EndRound` |

`State` fields: `content_id: String, phase: Phase, turn: u32, player: PlayerState,
enemies: Vec<EnemyState>, piles: Piles, card_ids_issued: BTreeSet<CardHandle>,
rng: CombatRng, pending: VecDeque<Effect>, current: Option<Effect>,
in_play: Option<CardInstance>, resolved_effects: u32, public: PublicKnowledge,
outcome: Option<Outcome>, fault: Option<FaultRecord>, hp_before_healing: Option<i32>,
hp_after_cleanup: Option<i32>`. `PublicKnowledge { revealed_order: Vec<CardHandle> }`.
`Combat { state: State }` has no exposed mutable-state accessor.

Content enums expose `as_str(self) -> &'static str` using the existing lower-snake
names. Card/enemy handles implement lexical `Ord`/`PartialOrd`; MoveId implements
`Ord` for owned damage maps, whose test projection sorts by semantic string name.

Preserve power insertion order for hook scheduling with `Vec<PowerEntry>`; reject
duplicate IDs. Public power projections sort by semantic ID name. Issued-card sets
sort lexically by canonical handle text for diagnostics only, never gameplay order.
Numbers use checked operations; valid turn is `1..=i32::MAX`, power amounts at most
`i32::MAX`, dispatch count at most 100,000. Maximum NORMAL damage is 1515
(`(11+999)*3/2`); block sum at most 1007 before cap, Strength sum 1002, HP healing
1005 before cap. Use wider checked intermediates for power stacking/turn advance.

Public methods, added by their owning tasks:

```rust
Combat::new(scenario: &Scenario) -> Result<Combat, InputError>
Combat::step(&mut self, action: &Action) -> Result<Transition, StepError>
Combat::legal_actions(&self) -> Result<Vec<Action>, AccessError>
Combat::public_view(&self) -> Result<PublicView, AccessError>
Combat::snapshot(&self) -> Snapshot
Combat::restore(snapshot: &Snapshot) -> Result<Combat, RestoreError>
snapshot::compare(left: &Snapshot, right: &Snapshot) -> Option<Divergence>
```

`Snapshot` owns combat-private (`pub(super)` within its module)
`schema_version: u32, content_id: String,
rng_contract_id: String, state: State`; no serde or mutable public fields.
`Divergence { component: &'static str, left: String, right: String }` holds detached
debug renderings of the first unequal component, not a policy observation or wire
encoding. Compare metadata in content/schema/RNG order, then every State field in
the order listed above; label embedded content `state.content_id` so even malformed
diagnostic snapshots cannot conceal a metadata/state mismatch. Tests may inspect/forge private
state through `combat::tests`, not a public effect-stepping/debug mutation API.

## Task 1: Native project RNG and immutable sampling

**Files:** Create `combat/mod.rs`, `rng.rs`, `tests/mod.rs`, `tests/rng_contract.rs`;
modify `crates/spire-core/src/lib.rs`, `crates/spire-core/Cargo.toml`, `Cargo.lock`.

**Interfaces:** `rng::RNG_CONTRACT_ID` exact existing identity. `Domain` is
`CombatShuffle/CombatEnemyHp/CombatEnemyAi`; `Partition` is
`Train/Selection/Final/Test`. `StreamId::new(domain: Domain, scope: Vec<String>)
-> Result<StreamId,RngError>` validates arity/ASCII. `StreamState::new(id: StreamId,
key: [u8;32], word_index: u128) -> Result<StreamState,RngError>` validates the
inclusive `2**64` sentinel. Private fields have read-only `id()`, `key()`,
`word_index()` accessors, returning `&StreamId`, `&[u8;32]`, `u128` respectively.
`StreamId::domain(&self) -> Domain`, `scope(&self) -> &[String]`, and
`Domain::as_str(self)`/`Partition::as_str(self) -> &'static str` expose validated
identity values. `RngError { Invalid(String), Exhausted { state: StreamState } }`.

- `make_stream(root: &[u8;32], partition: Partition, episode: &str, id: &StreamId) -> Result<StreamState,RngError>`.
- `draw_u64(state: &StreamState) -> Result<(u64,StreamState),RngError>`.
- `uniform_below(state: &StreamState, bound: u128) -> Result<(u64,StreamState),RngError>`; bound `1..=2**64`.
- `bernoulli(state: &StreamState, numerator: u128, denominator: u128) -> Result<(bool,StreamState),RngError>`.
- `weighted_index(state: &StreamState, weights: &[u128]) -> Result<(usize,StreamState),RngError>`; checked positive sum at most `2**64`.
- `shuffle<T: Clone>(state: &StreamState, items: &[T]) -> Result<(Vec<T>,StreamState),RngError>`.

Combat-private (`pub(super)`) calculation seams used by production and scripted tests:
`hmac_sha256(key:&[u8], message:&[u8]) -> [u8;32]`,
`uniform_from_words(bound:u128, next:&mut impl FnMut() -> Result<u64,RngError>)
-> Result<u64,RngError>`,
`shuffle_from_words<T:Clone>(items:&[T], next:&mut impl FnMut() -> Result<u64,RngError>)
-> Result<Vec<T>,RngError>`, and
`weighted_from_ticket(weights:&[u128], ticket:u64) -> Result<usize,RngError>`.
These are internal finite-algorithm seams, not runtime-configurable RNG backends.

- [ ] **Step 1: Write failing native RNG tests.** `fixture_known_answers` loads the
  unchanged JSON with `include_str!` using a path rooted at `CARGO_MANIFEST_DIR`
  (`../../tests/python/fixtures/combat/project_rng_v1.json`); checks both RFC cases,
  exact framed bytes/keys/blocks/words and every sampling/shuffle fixture row.
  `all_lanes_boundary_and_final_cursor` checks lane 0–3, index 3→4, final drawable
  index `2**64-1`→sentinel, exhaustion with no wrap and original input unchanged.
  `scripted_rejection_consumption` supplies `[u64::MAX, 7]` for bound 10: reject
  first, return 7 after two words; test exhaustion after that rejection retains
  the latest cursor. Use private shared sampling helpers for scripted word input,
  never a public backend or global mutable test hook.
  `invalid_and_zero_word_cases` rejects empty weights/overflow sum/bad scope/ASCII/
  bounds before sampling; bound one, Bernoulli endpoints and zero/one shuffle
  consume zero. `domain_isolation_and_replay`, `weighted_small_cases` and
  `three_item_all_shuffle_paths` check identities, immutable copies, zero weights,
  six permutations, duplicates and replay. Assert values, not frequencies.
- [ ] **Step 2: Run** `cargo test -p spire-core --lib --locked combat::tests::rng_contract::`.
  Expect RED from tests referencing the missing native RNG, not zero selected tests.
- [ ] **Step 3: Implement** the interfaces using standard
  `hmac::{Hmac, Mac}` with `sha2::Sha256`, `new_from_slice`, `update`,
  `finalize().into_bytes()`. Only after named dependency approval add the exact
  HMAC manifest entry; run `cargo check -p spire-core` once to resolve that addition.
  Inspect lock diff: no existing version upgrades. Stop on unexpected dependency
  changes rather than broadening scope. Preserve big-endian framing/lane order,
  `u128` rejection arithmetic and all unchanged contract semantics; no custom hash
  primitive, cache or generator substitution.
- [ ] **Step 4: Run** the Step 2 command and `cargo test -p spire-core --locked`.
  Expect GREEN, with named RNG tests actually selected and existing trace tests passing.
- [ ] **Step 5: Retain exact source/tests and obtain fresh independent review.**
  Verify RFC/fixture independence, checked upper endpoints, no input mutation and
  exact dependency delta. No commit. Correct/review findings before Task 2.

## Task 2: Typed supplied entry and detached schema-2 replay

**Files:** Create `content.rs`, `model.rs`, `snapshot.rs`, `engine.rs`,
`tests/support.rs`, `tests/entry_replay.rs`; extend `mod.rs`/`tests/mod.rs`.

**Interfaces:** Produces the shared types above, `Combat::new`, snapshot/restore/
compare, and internal `validate_scenario(&Scenario) -> Result<(),InputError>`,
`validate_runtime(&State) -> Result<(),InputError>`. `State` continues to be private.
`tests::support::base_scenario() -> Scenario` uses player 40/50 HP, energy/reset 3,
draw 5, Cultist 48, hand Strike#1/Defend#2/Bash#3, empty other piles, first intent/
history. Derive RNG with zero root, partition Test, episode `combat-behavior-plan`,
shuffle scope `[combat-plan-1]`, AI scope `[combat-plan-1,enemy-scope-1]`, owner
`enemy-1`. Initial damage entries: Cultist Attack 6, Jaw Worm Chomp 11/Thrash 7.

- [ ] **Step 1: Write failing entry/replay tests**, including:
  ```rust
  let scenario = base_scenario();
  let combat = Combat::new(&scenario).unwrap();
  assert_eq!(combat.state.phase, Phase::Decision);
  assert_eq!(combat.state.turn, 1);
  assert_eq!(combat.state.resolved_effects, 0);
  assert_eq!(combat.snapshot().schema_version, 2);
  assert_eq!(combat.state.enemies[0].move_history, vec![MoveId::Incantation]);
  assert!(combat.state.pending.is_empty() && combat.state.current.is_none());
  ```
  `entry_bounds_and_stream_owner` covers 0/1000 max HP/reset, current HP bounds,
  draw 0/11, deck 0/1000, hand 11, duplicate cards, initial powers/discard,
  wrong encounter/move/history/HP, enemy count/owner and scope/domain mismatch.
  `opaque_handles_are_lossless` round-trips `-1`, `0` and
  `1267650600228229401496703205376` (2**100); rejects `01`, `-0`, `+1`, empty and
  nondecimal input. Enemy handles reject empty/reserved `player`.
  `entry_and_restore_are_detached` mutates separate scenario/live/restored owners
  against independent baselines. `snapshot_compatibility_precedes_fault` rejects
  schema 1, wrong content/contract or mismatched embedded identity before fault
  rejection; fault snapshots cannot restore. `busy_snapshot_preserves_all_fields`
  constructs a valid between-dispatch queue and checks equality of all flags,
  streams, pending work and count after restore; resumption comes in Task 4.
  `first_component_diagnostics_are_detached` mutates later states after comparison
  and checks the captured first-component values unchanged.
- [ ] **Step 2: Run** `cargo test -p spire-core --lib --locked combat::tests::entry_replay::`.
  Expect RED for the missing types/constructor/replay behavior.
- [ ] **Step 3: Implement** types, entry derivation and complete structural/runtime
  checks. Construction takes a borrowed Scenario and clones only after validation.
  Initial streams/cursors are copied, never sampled. Validation distinguishes
  ready, busy, terminal and diagnostic fault states; allow zero HP with pending
  terminal work and selected/published move differences during a busy phase.
  Restoring an interrupted nonfaulted current payload is invalid. Snapshot
  creation itself does not repair/validate diagnostic state.
- [ ] **Step 4: Run** the Step 2 command and `cargo test -p spire-core --locked`.
  Expect GREEN, preserving Task 1 and trace regressions.
- [ ] **Step 5: Retain exact handoff and obtain fresh review.** Review every owned
  field and numeric/handle conversion; do not serialize Snapshot or expose mutable
  runtime fields merely for the future differential harness. No commit.

## Task 3: Bounded arithmetic and exact conditional enemy laws

**Files:** Create `rules.rs`, `tests/rules_ai.rs`; extend `content.rs`/module declarations.

**Interfaces:** `CardStats { cost,damage,block,vulnerable: i32 }` and
`card_stats(kind: CardKind, upgraded: bool) -> CardStats` in `content.rs`.
In `rules.rs`, all arithmetic returns `Result<...,InputError>`:
`normal_damage(base:i32,strength:i32,vulnerable:bool) -> Result<i32,InputError>`,
`block_after_add(current:i32,amount:i32) -> Result<i32,InputError>`,
`hp_and_block_after_damage(hp:i32,block:i32,damage:i32) -> Result<(i32,i32),InputError>`,
`clamped_strength(current:i32,delta:i32) -> Result<i32,InputError>`.
`Ratio { numerator:u128, denominator:u128 }`;
`jaw_worm_fallback(history:&[MoveId],roll:u8) -> Result<Option<Ratio>,InputError>`;
`jaw_worm_move(history:&[MoveId],roll:u8,fallback:Option<bool>) -> Result<MoveId,InputError>`;
`choose_enemy_move(kind:EnemyKind,history:&[MoveId],stream:&StreamState)
-> Result<(MoveId,StreamState),SelectionError>`, with
`SelectionError { Invalid(InputError), Random(RngError) }`.

For consumption/error tests, combat-private (`pub(super)`) trait and helper:
`SelectionDraws` has
`bucket(&mut self) -> Result<u8,RngError>` and
`coin(&mut self, probability:Ratio) -> Result<bool,RngError>`;
`select<D:SelectionDraws>(kind:EnemyKind, history:&[MoveId], draws:&mut D)
-> Result<MoveId,SelectionError>` shares the finite selection control flow.
The only production adapter owns a local native stream; scripted implementations
exist only in tests. This is not a public alternate generator/backend API.

- [ ] **Step 1: Write failing arithmetic/selection tests.** Pin all six card rows;
  damage `(9,0,true)=13`, `(6,999,false)=1005`, `(11,999,true)=1515`;
  block `(998,8)=999`; HP/block `(40,5,11)=(34,0)`, `(5,0,11)=(0,0)`;
  Strength `(998,3)=999`. Reject negative/out-of-surface arguments.
  `exact_history_laws` enumerates 0–99 and both fallback results for histories
  empty, Chomp, Chomp/Bellow, Chomp/Thrash, Chomp/Thrash/Thrash; asserts exact
  Chomp/Thrash/Bellow rows:
  `(1,0,0)`, `(0,131/320,189/320)`,
  `(73349989/167772160,94422171/167772160,0)`, `(1/4,3/10,9/20)`,
  `(29955719/83886080,0,53930361/83886080)`.
  Accumulate rational weights with common denominator `100*8388608` in `u128`
  and check by cross multiplication, not floating-point tolerances.
  `only_restricted_buckets_request_fallback` pins all three exact Ratio arguments,
  boundaries 24/25/54/55/99, at most one bucket plus one conditional coin, and
  no draw for forced first moves/Cultist attacks. For counters/errors use a
  private selector-only draw seam with scripted bucket/coin results, not public
  RNG backends or shared mutable global hooks. `fixed_project_selection` uses
  base_scenario's AI stream: initial roll 29, choose Thrash after Chomp, cursor 1.
  `invalid_selection_does_not_sample` checks wrong enemy/history/domain and roll
  100; `fallback_exhaustion_is_not_selection` preserves the latest exhausted state.
- [ ] **Step 2: Run** `cargo test -p spire-core --lib --locked combat::tests::rules_ai::`.
  Expect RED for absent calculations/selection, not a changed RNG fixture.
- [ ] **Step 3: Implement** bounded calculations and the two-stage selection law.
  Exact `floor(1.5*n)` is `(3*n)/2` on this nonnegative bounded surface; prove the
  shared numeric bounds above. Selection neither appends history nor publishes
  intent. Private draw seams may share selection/sampling control flow with tests,
  but expose no configurable generator or mechanically translated rule body.
- [ ] **Step 4: Run** the Step 2 command and `cargo test -p spire-core --locked`.
  Expect GREEN including exact probabilities, consumption and rejection coverage.
- [ ] **Step 5: Retain handoff and obtain fresh review**, independently recomputing
  the rational rows and checking integer floors against the bounded mechanics.
  No original-RNG counter/NaN study, raw vectors or commit.

## Task 4: Ordered resolver, power lifecycle and diagnostic faults

**Files:** Create `effects.rs`, `tests/resolution.rs`; extend `engine.rs`,
`model.rs`, `snapshot.rs` and module declarations.

**Interfaces:** Internal `engine::dispatch_one(state:&mut State) -> ()` retains
faults rather than unwinding; `engine::resolve(state:&mut State) -> Transition`
continues without resetting the dispatch count. `effects::apply_effect(state:&mut
State,effect:&Effect) -> Result<(),EffectError>`, with
`model::EffectError { Invalid(InputError), Random(RngError) }`; route future owners through
the same signature. `effects::refresh_enemy_values(&mut State) -> Result<(),InputError>`.
`model::mark_fault(&mut State,kind:FaultKind,message:String) -> ()` preserves the
first record and uses the existing counted dispatch. Current work owns the selected
effect until successful completion. Exhaustion errors validate/install only matching
owned stream identity/key and monotonically advanced cursor before faulting.
Map InputError::Unsupported to Unsupported fault, InputError::Invariant or
RngError::Invalid to Invariant, exhausted RNG to RngExhausted, and the dispatch
limit to EffectGuard. A mismatched exhaustion owner is Invariant, not a replacement
of some unrelated stream.

- [ ] **Step 1: Write failing resolver/power tests.** `prepend_keeps_current_owner`
  proves front insertion cannot replace current work. `damage_reference_refresh`
  queues enemy Attack damage, changes Strength through an earlier effect, and
  observes refreshed output; fixed card damage remains its captured amount.
  `power_flags_stack_without_reset` tests Ritual skip-first initialization,
  existing Vulnerable flag preservation, capped Strength and insertion order.
  `full_reduction_queues_removal_first` checks zero-amount removal and immediate
  derived-value refresh. `dispatch_99999_finishes_100000_faults` uses zero-block
  work followed by Decision, with 99,999 versus 100,000 total dispatches; assert
  failing marker retained, no payload/terminal/loss, exact count and partial state.
  `resume_does_not_reset_budget` restores between completed dispatches at count
  99,999 with pending work and faults before the next payload. `first_fault_wins`
  and `exhaustion_installs_only_owned_progress` check repeated fault attempts,
  wrong-stream errors, and latest AI/shuffle cursor retention without cross-talk.
  `unsupported_and_invariant_faults_preserve_work` uses incompatible move/actor
  references, not a permissive unknown-effect no-op. `all_safe_point_fields_replay`
  restores a successful power/damage sequence after each completed dispatch.
- [ ] **Step 2: Run** `cargo test -p spire-core --lib --locked combat::tests::resolution::`.
  Expect RED on missing dispatch/payload/fault behavior.
- [ ] **Step 3: Implement** finite dispatch and damage/block/power/Decision owners.
  Increment count before applying the selected payload; count 100,000 faults first.
  Do not retry an interrupted current effect. Queue lethal Terminal work at front;
  terminal implementation lands in Task 5. Stage-dependent kinds return explicit
  Unsupported until their task lands, not silently succeed. Convert checked
  in-resolution failures to the proper first fault, not panics or gameplay losses.
- [ ] **Step 4: Run** the Step 2 command and `cargo test -p spire-core --locked`.
  Expect GREEN including safe-point replay and Task 1–3 regressions.
- [ ] **Step 5: Retain handoff and obtain fresh review.** Review all first-fault,
  current/pending ownership and guard paths, including diagnostic restoration
  rejection. No clear-fault API or commit.

## Task 5: Card actions, detached public views and terminal handling

**Files:** Create `view.rs`, `piles.rs` (discard membership only),
`tests/actions_views.rs`; extend `engine.rs`, `effects.rs` and module declarations.

**Interfaces:** Produces `Combat::step`, `legal_actions`, `public_view` above.
`view::legal_actions(&State) -> Result<Vec<Action>,AccessError>` and
`view::public_view(&State) -> Result<PublicView,AccessError>` implement them.
`piles::discard_card(state:&mut State,card:&CardHandle,end_turn:bool)
-> Result<(),InputError>` handles immediate in-play/hand→discard movement.
`PublicView` fields mirror the existing Python public view: `content_id: String,
turn:u32, player:PlayerView, enemies:Vec<EnemyView>, hand:Vec<CardView>,
draw_count:usize, discard:Vec<CardView>, exhaust:Vec<CardView>,
revealed_order:Vec<CardHandle>, outcome:Option<Outcome>,
hp_before_healing:Option<i32>, hp_after_cleanup:Option<i32>`.
`CardView` copies card fields; `PlayerView` has HP/max/block/energy/max_energy and
sorted `powers: Vec<(PowerId,i32)>`; `EnemyView` has handle/kind/HP/max/block,
sorted powers, intent ID/damage. No internal flags or hidden data in these types.

- [ ] **Step 1: Write failing action/view/terminal tests.** `bash_then_strike_order`
  starts Cultist 48 with base Bash#1/Strike#2 and energy 3: after Bash HP40,
  Vulnerable2, energy1; after Strike HP31, energy0. Test upgrades separately.
  `late_invalid_action_is_atomic` plays a valid card, snapshots, then submits
  insufficient-energy/missing-card/wrong-target/Defend-with-target actions;
  assert exact before/after snapshot/RNG equality. `terminal_and_busy_reject_steps`
  and `faulted_views_are_errors` distinguish all API errors without mutation.
  `public_hidden_state_noninterference` changes unrevealed draw order, valid keys/
  cursors/private scope and internal AI details while holding the public history
  fixed; public values/action order remain identical. `views_and_actions_are_owned`
  mutates returned containers and advances the engine against an earlier baseline.
  `burning_blood_terminal_checkpoints` sets enemy HP low through internal setup,
  tests a lethal Strike with living player HP40/max50→40/46 and 49/50→49/50; no
  decision before cleanup, terminal actions empty, no blanket pile reset, lethal
  card still accounted for. `death_never_heals_or_rolls` queues lethal enemy damage
  before exhausted AI selection: death0/0, no later draw/selection/fault.
- [ ] **Step 2: Run** `cargo test -p spire-core --lib --locked combat::tests::actions_views::`.
  Expect RED on missing step/view/terminal behavior.
- [ ] **Step 3: Implement** complete action preflight, card payload scheduling,
  FinishCard/discard membership, public projections and synchronous terminal heal/
  cleanup. Step resets the count only after a valid external action is accepted;
  return `Transition { status }`, not a snapshot/view/state clone. EndTurn preflights
  and schedules EndPlayerTurn; its owner is explicitly unsupported until Task 7.
  Enumerate only public hand-order legal choices with EndTurn last, no speculative
  engine calls/RNG use. In terminal cleanup clear gameplay work, not every diagnostic
  pile/block/card value; preserve the supported in-play owner on lethal card use.
- [ ] **Step 4: Run** the Step 2 command and `cargo test -p spire-core --locked`.
  Expect GREEN for cards/views/terminal paths; do not claim complete EndTurn yet.
- [ ] **Step 5: Retain handoff and obtain fresh review.** Inspect observation type
  fields and all pre-mutation rejection paths, including post-valid-action errors.
  No Python binding, speculative action encoder or commit.

## Task 6: Draw, discard traversal and transactional recycle

**Files:** Extend `piles.rs`, `engine.rs`; create `tests/piles.rs` and its declaration.

**Interfaces:** Add `piles::apply_effect(&mut State,&Effect) -> Result<(),EffectError>`
for Draw, Recycle, DiscardHand; use Task 5 `discard_card` and Task 1 `shuffle`.
Route those three variants in the resolver, preserving the existing work catalog.

- [ ] **Step 1: Write failing pile tests.** `draw_uses_end_and_preserves_owners`
  draws storage `[1,2,3]` in order `[3,2,1]`; verify public reveal order/issued set.
  `discard_hand_visits_reverse_hand_order` moves hand `[1,2,3]` to discard `[3,2,1]`.
  `split_draw_existing_recycle_residual` inspects the queue and final hand: existing
  draw executes before recycle, remaining draw after; never combine remaining
  draw with discard. `draw_rechecks_sizes_and_hand_cap` covers 9→10 hand, empty
  piles, insufficient total cards and sizes altered by earlier pending work.
  `fixed_recycle_draw_order` shuffles discard `[1,2,3]` with base_scenario stream:
  storage `[2,3,1]`, then draw order `[1,3,2]`, cursor2; singleton/empty recycle
  consume no words. `recycle_exhaustion_preserves_piles` starts three discard
  cards one word before RNG exhaustion; asserts latest exhausted shuffle cursor,
  original discard/draw intact, AI unchanged, fault not death. Check exact unique
  card ownership before/after every successful dispatch, including in play.
- [ ] **Step 2: Run** `cargo test -p spire-core --lib --locked combat::tests::piles::`.
  Expect RED on explicitly unsupported pile operations or missing ordering behavior.
- [ ] **Step 3: Implement** pile owners and queue routing. Recompute draw limits
  when each Draw resolves. Shuffle a detached discard permutation; commit pile
  transfer only on successful complete sampling. Retain exhausted sampling cursor
  through the resolver's fault policy even when pile transfer does not commit.
- [ ] **Step 4: Run** the Step 2 command and `cargo test -p spire-core --locked`.
  Expect GREEN including card-use ownership/terminal regressions.
- [ ] **Step 5: Retain handoff and obtain fresh review**, focusing on traversal,
  split draw, empty/singleton consumption and exhaustion transaction boundaries.
  No custom deck generator or commit.

## Task 7: Enemy execution, round hooks and next-player decision

**Files:** Create `turns.rs`, `tests/turns.rs`; extend `engine.rs` and declarations.

**Interfaces:** `turns::apply_effect(&mut State,&Effect) -> Result<(),EffectError>`
owns EndPlayerTurn, MonsterStart, EnemyMove, RollMove, EndRound.
`turns::schedule_next_player_turn(&mut State) -> Result<(),InputError>` performs
construction-time next-player setup and appends Draw/Decision. Route all remaining
work variants; no stage-dependent unsupported owner may remain at completion.

- [ ] **Step 1: Write failing turn tests.** `cultist_ritual_first_skip_and_growth`
  checks Incantation then attack6, then attack9 with Strength3; inspect skip-first
  flips and queued Strength versus next-player setup timing. `published_move_runs`
  ensures current Chomp damages11 before later Thrash selection and never executes
  newly selected Thrash in that turn. `jaw_move_payload_order` tests Thrash damage7
  then block5, Bellow Strength3 then block6, and enemy block reset at monster start.
  `next_player_construction_before_ritual_payload` snapshots after EndRound has
  recharged energy/reset block/published intent but before pending Strength/draw;
  resolve afterward and verify final refreshed attack/intent and draw order.
  `vulnerable_expiry_preserves_grace` tests reapplication/stacking without resetting
  flags and round-side reduction/removal. `roll_installs_then_appends_once` checks
  history, cursor, selected move and later publication separately.
  `fallback_exhaustion_preserves_prior_turn_damage` retains partial HP/work and
  exhausted AI cursor, with no fabricated history/publication. For forced fallback
  error injection, keep the seam internal to tests and call the same selector/
  error-installation path; do not expose runtime generator substitution.
  `turn_surface_limit_is_explicit` injects turn `i32::MAX`: next-player advance
  faults Unsupported without wrap or loss. `resume_each_turn_safe_point` restores
  between every completed dispatch across Incantation and a growing round and
  compares complete continuation/RNG/flags/count plus final public values/actions.
- [ ] **Step 2: Run** `cargo test -p spire-core --lib --locked combat::tests::turns::`.
  Expect RED on the explicit not-yet-supported turn owners.
- [ ] **Step 3: Implement** turn owners from the reviewed semantic ordering, not
  translation of Python/original bodies. EndPlayerTurn prepends DiscardHand then
  MonsterStart; MonsterStart resets enemy block and schedules the published move;
  its payloads precede RollMove then EndRound. EndRound visits player then enemy
  powers in insertion order, appends round payloads, then construction-time next-
  player reset/recharge/publication appends Draw/Decision behind those payloads.
  Publish no intermediate decision. Preserve counted failures and check turn bounds.
- [ ] **Step 4: Run** the Step 2 command and `cargo test -p spire-core --locked`.
  Expect GREEN; verify every supported EffectKind now has a real owner.
- [ ] **Step 5: Retain handoff and obtain fresh review**, checking exact queue
  placement, reference-based enemy damage and all replay checkpoints. No frame
  simulator, original execution or commit.

## Task 8: Whole-combat acceptance and test-only Python comparison

**Files:** Create native `tests/whole_combat.rs`, `tests/differential.rs`, Python
`rust_diff_support.py`, `test_rust_differential.py` at mapped paths; extend test
declarations only. Update README/scope/spec implementation status after verification.

**Interfaces:** Native `combat::tests::differential::protocol` is an ordinary unit
test, entirely `cfg(test)`; `encode_checkpoint(&Combat) -> serde_json::Value` and
request decoding live only in that test module. Python support defines
`run_python_case(case: dict) -> dict`, `normalize_python_checkpoint(state) -> dict`,
`parse_test_executable(stdout: str, manifest: Path) -> Path`, and
`run_rust_case(executable: Path, case: dict, directory: Path) -> dict`.
`assert_case_equal(left: dict, right: dict) -> None` compares complete responses
and reports the first case/index/semantic component divergence; corruption controls
must exercise this same helper, not a disconnected equality assertion.
Pytest session fixture `rust_combat_executable` performs one locked offline build.
No Cargo target/feature/product CLI or public state-mutation API is added.

### Exact authored test interchange

Request schema `spire-lab-combat-diff-v1` has exactly `schema`, `case_id`, `scenario`,
`actions`; case_id is a nonempty string. Scenario uses the shared Scenario field names. Cards are
`{instance_id: decimal_string, card_id: lower_snake_case, upgraded: bool}`;
player/enemy fields follow the shared type table, with `enemies` an array (the
engine enforces exactly one) and powers an insertion-ordered array of
`{id, amount, skip_first, monster_source_grace}`. Piles have all four named arrays.
RNG has `contract_id`, `enemy_instance_id`, and `shuffle`/`enemy_ai`, each
`{domain, scope: [strings], key_hex: 64_lowercase_hex_chars, word_index: decimal_string}`.
All supplied cursor values must be in `0..=18446744073709551616`.

Actions are exactly `{kind:"play_card",card_instance_id:decimal_string,target_id:
string_or_null}` or `{kind:"end_turn"}`. Other gameplay numbers are JSON integers
in the native i32 range; reject booleans/floats as integers, missing/extra fields,
malformed hex and noncanonical decimal strings. Interchange text must be Unicode
scalar text; reject lone surrogates in Python rather than replacing or accepting
text the Rust decoder cannot represent. Native enums map
to the existing Python lower-snake names (`jaw_worm`, `incantation`, etc.). A
well-formed excluded card/enemy/move/power ID or action kind is Unsupported;
wrong schema, unknown RNG domain or malformed encoding is
Interchange. Validly encoded out-of-gameplay-range values go to engine validation
as Invariant. No partition is carried in already derived request streams.
Decode actions individually immediately before submission, so late malformed
action rejection can be checked against the already reached checkpoint. A target
that is a string but cannot name an enemy (empty or reserved `player`) is a
semantic InvalidAction rejection, not numeric coercion or an actor-ID alias.

Response has exactly `schema`, `case_id`, `entry_error`, `checkpoints`. Entry error
is null or `interchange`/`unsupported`/`invariant`; no checkpoints on entry failure.
Otherwise checkpoints include index0 for entry and one per requested action.
Each record has `index`, `status` (`decision`/`terminal`/`fault`/`rejected`),
`rejection` (null or `interchange`/`unsupported`/`invalid_action`/`not_ready`/
`terminal`/`faulted`), `state`, `public`, `legal`. Rejections preserve the previous
full state; fault state has `public:null,legal:null`. Terminal legal list is empty.
Continue recording rejected post-terminal/fault actions without clearing state.

Canonical `state` uses every shared State field in the shared field order. Include
`resolved_effects`, normalized `pending`/`current`, complete power flags and
damage-entry values, not just HP/outcome. State RNG uses the request stream form;
handles/reveals/issued IDs remain strings, power order is retained, issued IDs and
damage-entry maps sort by canonical string name. Effects normalize to
`{kind,source,target,payload}` with kind/field names from the reviewed Python work
catalog; amounts versus move references remain distinct. Source is null, or
`{actor: string}`, or `{card: decimal_string}` so numeric-looking enemy handles
cannot alias cards. In-play card is an ordinary card object or null; fault is
null or `{kind, resolved_effects}` (omit human message text only). Public uses
the Python PublicView field names, card handles as strings, sorted `[power,amount]`
pairs; legal is an ordered array in the Action request form. Normalize only at
entry/decision/terminal/fault boundaries; cross-engine internal dispatch-count
differences are investigated, never discarded to force equality.

The Python fixture builds with
`cargo test -p spire-core --lib --no-run --locked --offline --message-format=json`
(180s timeout). Parse compiler-artifact records, requiring exact resolved core
manifest path, `target.name == "spire_core"`, target kind `lib`, `profile.test ==
true` and a nonnull executable; require exactly one matching existing executable.
Do not guess hashed paths or skip on build/cargo failure. Invoke that executable
with `--exact combat::tests::differential::protocol --nocapture` (30s timeout),
passing child-only `SPIRE_COMBAT_DIFF_REQUEST`/`SPIRE_COMBAT_DIFF_RESPONSE` absolute
file paths. Request/response files are fresh per case; response uses create-new,
not overwrite. No global environment mutation, shell interpolation or game data.
Retain a real failed comparison's request/responses/logs in its fresh local
execution stage; pytest scratch files are not the sole history. Later success
must not overwrite the earlier disagreement or failure control.
Assert exit0, exactly one passing/zero ignored tests, matching schema/case_id and
all requested checkpoint indices before comparing values. Missing output, zero
selected tests or stale identity is a harness failure, not an agreement result.
Use only metadata from this successful build invocation. Legitimately cached
Cargo artifacts are allowed; no wall-clock mtime or speculative rebuild gate.

With neither environment variable set, the ordinary native protocol test runs a
fixed ready-checkpoint encoding assertion; with exactly one variable it fails.
No ignored adapter test or Python invocation in native cargo tests. Full Python
tests may build/run the native test binary, not a training/runtime dependency.

- [ ] **Step 1: Write failing whole-combat and harness tests.** Native
  `two_turn_victories` uses player40/50, energy/reset3, draw3, upgraded Bash#1/
  Strike#2/Defend#3 in hand, no draw, base streams, Cultist48 or JawWorm40; play
  #1,#2,EndTurn,#1,#2. Pin intermediate enemy HP25/17 then10/2; hand after turn
  `[1,3,2]`, shuffle cursor2, AI cursor0/1; terminal HP40/46 and29/35 respectively.
  `cultist_death` uses player14/20, draw1, hand base Defend#1 and three EndTurns:
  player14→8→0, AI cursor0. `jaw_death_stops_rng` uses player10 and exhausted
  AI at entry; EndTurn dies with unchanged AI, no later selection or healing.
  `whole_combat_safe_points_and_independent_snapshot_baselines` replays from every
  completed dispatch and decision; verify saved capture-time state, not a restored
  value compared to itself. `forty_project_legal_walks` uses 20 episode identities
  per enemy: episode `combat-vertical-{episode}` for episode 0–19, base scopes,
  nine cards from `[Bash,Strike,Defend]` repeated three times with handles1–9,
  upgrade flag `(zero_based_card_index + episode) % 2 != 0`, hand first3/draw rest.
  Pick public legal action `(episode + zero_based_step) % legal_count`, at most
  200 steps, complete card ownership and direct/restored equality each step.
  These are authored acceptance tests, not original seeds or transfer evidence.
  Python tests `fixed_checkpoints_match`, `legal_walk_checkpoints_match`,
  `late_rejection_preserves_checkpoint`, `exhaustion_and_post_fault_match`,
  `large_negative_handles_round_trip`,
  `malformed_interchange_never_coerces_values`, `normalizer_has_every_component`
  cover both enemies, fixed victories/deaths, replay, public values/actions, all
  canonical fields and bool/float/large-ID cases. Exhaustion cases include AI
  sentinel at entry followed by nonlethal EndTurn, and multi-card recycle with
  only one drawable shuffle word; assert partial state/current/pending/count and
  a subsequent rejected faulted action match without attempting fault restoration.
  Add controls
  `missing_ambiguous_or_stale_artifact_fails`, `zero_tests_or_missing_response_fails`,
  `one_field_corruption_is_detected`, checking native private flags/RNG/pile/order
  and public/action projection mismatches. Use narrow authored subprocess doubles
  for harness failures alongside real successful native executions; never hide a
  failing real comparison behind a mock.
- [ ] **Step 2: Run** `cargo test -p spire-core --lib --locked combat::tests::whole_combat::`
  and `.venv/bin/python -B -m pytest -q tests/python/combat/test_rust_differential.py`.
  Expect RED from missing tests/normalizers/protocol, not absent-cargo skips or
  treating zero executed Rust tests as success.
- [ ] **Step 3: Implement** test-only request/response mapping and orchestration.
  Reuse public constructor/step/view/snapshot APIs; use private native state only
  in cfg(test) encoding. Python constructs existing dataclasses explicitly and
  normalizes their values, with no product rule changes. For each legal fixed/walk
  case compare independent direct and restored runs in each engine as well as
  cross-engine checkpoints. Restore only nonfaulted safe checkpoints; terminal
  restoration remains terminal and faulted snapshots stay diagnostic-only.
  Investigate any gameplay discrepancy against reviewed
  facts/independent examples; put fixes in the owning task with regression/review.
- [ ] **Step 4: Run full verification**: `cargo test --workspace --locked`,
  `cargo test -p spire-core --release --locked`,
  `.venv/bin/python -B -m pytest -q`, `git diff --check`, and
  `sha256sum tests/python/fixtures/combat/project_rng_v1.json`; require SHA256
  `369d91e2ce3be38c191ef5ceab6fe6e1bf5c73f21504e1748074e9c35d962bad`.
  Require zero failures and real differential execution; a
  release test run checks semantics, not throughput. No toolchain/component
  installs, generic capture tests or original execution. Record actual counts,
  command outputs and limitations; update README/scope/spec only afterward.
- [ ] **Step 5: Obtain fresh whole-slice review** of exact source snapshots and
  results after task review, independently checking chance laws, internal replay,
  public information, fault boundaries and no hidden Python/game runtime dependency.
  Resolve findings, obtain scoped fresh re-review, rerun verification and retain
  the final handoff. Do not stage/commit, start batching or claim original fidelity.

## Self-review and execution handoff

This plan covers one testable native slice, not full-game infrastructure. Spec
sections 1–2 and 10 govern all tasks; components/types/API in 3–4 map to Tasks 1–5;
mechanics in 5 map to Tasks 3–7; randomness in 6 maps to Task 1 and integration;
terminal/faults in 7 map to Tasks 4–7; snapshots/comparison in 8 map to Tasks 2/4/8;
all verification items in 9 have named tests above. Each Review Focus item maps
to explicit owning tests. The plan's examples prescribe assertions/values, not
translated implementation bodies.

Official API references checked during planning:
[HMAC 0.12.1](https://docs.rs/hmac/0.12.1/hmac/),
[its digest/sha2 compatibility](https://docs.rs/crate/hmac/0.12.1/source/Cargo.toml),
[Cargo JSON artifact messages](https://doc.rust-lang.org/cargo/reference/external-tools.html#artifact-messages).
Context7 lookup hit its monthly quota; version-specific official documentation
was used instead. No dependency resolution or product commands ran during planning.

Next: user reviews this plan and explicitly approves start and the named native
HMAC dependency. Preserve the selected Sol/fresh-reviewer method. This plan itself
authorizes no installation, Git write, original access or execution. After a
verified core handoff, separately scope native batching/bindings, release benchmarks
and the first learner-versus-baseline comparison; capture tooling remains deferred.
