# Source-first narrow Rust combat core

Date: 2026-10-16

**Status: specification and subsequent implementation plan/start approved.** The
narrow native core is implemented in the current worktree. Following the separately
reviewed native Draw/RollMove producer provenance correction, Task-8 verification
passes: 5 native whole-combat tests, 95 differential tests, 830 full Python tests,
and, after the final minor-fix wave, 124 native tests in each workspace/release run. Both formerly failing fault
cases now agree on complete checkpoints without normalizer/Python rule changes;
the original failed exchanges and blocked report remain historical evidence.
The test-only adapter now preserves arbitrary-size integer handles beyond CPython's
default decimal limit, including reveal execution in a disposable comparison child;
parent settings and gameplay code remain unchanged. The unchanged default Python
product engine still has its enormous-handle reveal conversion limit.
User plan/start approval was executed. All task reviews (including the adapter fix)
and whole-slice review are done. Both whole-slice Minor findings are corrected and
approved by fresh scoped re-review. Fresh controller verification passed: 124 native
tests in debug/release and 830 Python tests. The core-only handoff is complete.
The normative dated body below retains its original authorization sequence.
This is **source-grounded, not original-runtime validated**, not a batching,
benchmark, learning or original-capture milestone. The
[implementation plan](../plans/2026-10-16-source-first-rust-combat.md) governs that
separate handoff. No staging or commit is authorized.

## 1. Purpose and governing contracts

Build the first independently authored Rust combat slice toward an optimized,
game-correct headless StS simulator and an AI that plays StS and improves.
Held-out playing strength gained per wall-clock hour remains the end metric;
this milestone establishes a native correctness baseline, not a trained agent or
a measured throughput improvement.

The [approved source-first amendment](../../scope.md#approved-source-first-sequencing-amendment-2026-10-16)
moves original-game comparisons to a later fidelity-promotion gate. Original
captures and Linux capture readiness no longer block Rust development. Label
this milestone **source-grounded, not original-runtime validated**. Python/Rust
agreement cannot establish original-game fidelity or demonstrated transfer.

The following remain normative behavioral inputs:

- [Narrow combat contract](2026-10-16-python-combat-behavior-design.md), especially
  supplied entry, exact conditional chances, ordering, powers, terminal handling,
  faults, replay and visibility. Its Python implementation choices are not Rust
  layout requirements, and its original-captures-before-Rust restriction is
  superseded by the approved sequencing amendment.
- [Project randomness contract](2026-10-16-project-randomness-design.md), sections
  3–6, and [RNG-001](../../intentional-deviations.md): unchanged project words,
  sampling, consumption and stream isolation, not original seed/PRNG parity.
- [ML-first architecture](2026-10-11-spire-lab-ml-first-design.md) and
  [current scope](../../scope.md): native simulation, later native batching/thin
  Python learning boundary, no playable client and no runtime game dependency.

Independently author Rust behavior from reviewed mechanics and requirements.
Neither original methods nor the Python rule implementation is a mechanical
translation template. Existing Python tests supply authored examples and a second
implementation to compare, not an unquestionable oracle.

## 2. Scope and exclusions

Support complete combat from an explicit first ready player decision through
subsequent card choices, enemy turns and terminal cleanup:

- Ironclad A0; base/upgraded Strike, Defend and Bash; Burning Blood.
- One Cultist or ordinary Jaw Worm, and only the powers/arithmetic/pile mechanics
  needed for this slice: nonnegative Strength, Ritual, Vulnerable, NORMAL damage,
  block, ordinary card use/discard, draw/recycle and turn sequencing.
- Project RNG ownership, exact conditional enemy selection, legal actions, public
  observations, snapshots/replay and administrative faults.

Use the same supplied-entry conditions and numeric gameplay bounds as the narrow
combat contract: no implicit HP/energy/draw defaults, initial player block/powers
zero/empty, energy equal to the supplied reset amount, hand at most 10, total deck
1–999, initial discard/exhaust empty, one full-health enemy with no block/powers,
and the published first move selected exactly once: Cultist Incantation or Jaw
Worm Chomp. Cultist HP is 48–54; ordinary Jaw Worm HP is 40–44. Player max HP and
reset energy are 1–999, with positive current HP at most max HP; per-turn draw is
1–10. Initial card ownership is unique and complete. No initial shuffle, HP
sampling or draw is performed by combat construction.

Exclude encounter/dungeon startup, run/deck generation, acquisition history,
additional cards/relics/potions/enemies, other Ascensions/characters/modifiers,
negative Strength and unsupported retention/autoplay rules. Also exclude native
pools, bindings, numeric feature encoders, training, performance tooling, capture
helpers/adapters and new original-game access/execution from this milestone.

No renderer, animation/frame loop, audio, interactive UI, Python interpreter,
tensor library, game JAR or capture connection is required to run Rust combat.
Do not build a plugin framework, ECS, generic card-game engine or alternate RNG
backend. Keep the existing Rust trace protocol unchanged.

## 3. Components and owned representation

Add `spire_core::combat` alongside the existing `trace` module. Keep internal
modules focused by responsibility:

| Responsibility | Contents |
|---|---|
| Model/content | Validated Scenario, stable typed identities, supported content, combat state, phases, powers, actions, effects and errors |
| RNG | Independent native implementation of the existing project contract and owned combat stream bundle |
| Rules | Damage/block calculations, exact rational AI decisions and supported static payloads |
| Resolver | Ordered effect dispatch, card/pile/turn operations, terminal processing and first-fault preservation |
| View | Detached public observation and deterministic legal-action enumeration |
| Snapshot | Detached complete state, compatibility checks, restoration and semantic divergence diagnostics |

The implementation plan may split pile, power and turn owners into smaller files
without adding separate subsystems. No module executes Python or loads private
game evidence. Name and justify any dependency changes in the implementation plan
before execution approval; this specification installs or changes none.

`Combat` privately owns mutable gameplay state. Use typed card/content/move/power
identities and effect variants rather than string-dispatched callbacks or generic
payload maps. Stable public card/enemy handles remain separate from private RNG
scope IDs; test interchange must preserve identity exactly, without truncation,
float coercion or dependence on native addresses. Handles are not arithmetic values.

Own ordered hand/draw/discard/exhaust collections and the in-play card, issued-card
membership, player/enemy values, damage entries, published intent and selected move
history, lifecycle flags, pending/current work and continuations, public knowledge,
RNG streams, terminal HP observations and first fault. A card has exactly one
gameplay owner, including while in play. Preserve explicit order; draw-pile end is
top. Public powers expose amounts, not private lifecycle flags.

Use ordinary growable collections and exclusive mutable access to one combat.
Do not impose speculative fixed capacities or share mutable global state. Existing
gameplay/input limits remain; growable storage does not expand supported content.
Avoid whole-state copying on every action. Explicit snapshot/observation operations
may copy their requested data; do not automatically create both after every step.

## 4. Semantic API and transition boundary

The public shape is a validated constructor plus explicit operations on owned
combat; exact Rust field/type spelling belongs in the implementation plan:

| Operation | Contract |
|---|---|
| Construct from Scenario | Validate supplied first-ready input and produce independently owned state, without sampling or mutating caller-retained input |
| Enumerate legal actions | Ordered `PlayCard` choices in public hand order, then `EndTurn`; no speculative execution or RNG use |
| Step with an action | Mutate in place through an exclusive reference and resolve to decision, terminal or fault; return a small transition summary, not a combat clone |
| Read public observation | Detached policy-visible values only, independent of future mutations |
| Snapshot | Explicit detached complete internal checkpoint, including diagnostic fault state |
| Restore | Validate compatibility/invariants and construct an independent nonfaulted combat; failure does not replace an existing combat |
| Compare diagnostics | Deterministically report the first differing semantic component with detached diagnostic values |

Actions are typed `PlayCard(card_handle, target)` and `EndTurn`. Strike/Bash target
the sole live enemy; Defend is untargeted. Check phase, ownership, content, target
and energy before mutation or sampling. Invalid actions return a typed rejection
and preserve gameplay state, pending work, RNG and observations exactly.

At a ready decision, derived values are current and gameplay work/continuations
are exhausted. Terminal legal actions are empty. Ordinary stepping rejects
terminal, busy or faulted state; faulted legal-action/observation access returns a
typed faulted-state error, not a plausible policy observation. Busy state is not
a normal public checkpoint. Diagnostic snapshots are separate from observations.

Public observations include the contracted player/enemy values and public powers,
published intent, hand identities/upgrades, visible discard/exhaust, pile counts,
turn/public reveals and terminal outcome/HP observations. Exclude RNG keys/cursors,
private derivation IDs, unseen draw order, selected future moves/private history,
work queues, fault details and evidence metadata. Identical public histories with
different hidden state must yield identical observations and ordered legal actions.

## 5. Resolution and mechanics fidelity

Implement data-only typed work with explicit ownership and timing. Prepend/append
semantics must preserve accepted top/bottom insertion, and prepending cannot
replace already selected current work. Distinguish construction/enqueue mutations
from later payload execution. Preserve card/pile/turn hook order without simulating
presentation time. Unsupported effects are rejected or faulted explicitly, never
silently ignored.

Required card payloads are Strike 6/9 damage for 1 energy, Defend 5/8 block for 1,
and Bash 8/10 damage followed by 2/3 Vulnerable for 2. Calculate Bash's attack before
its new Vulnerable applies. Preserve removal from hand, energy spending, in-play
ownership and reviewed use/discard ordering.

Carry forward the reviewed end-player, monster-start, current enemy move, later
move selection, end-round and next-player setup/draw ordering. Chomp is 11 damage;
Thrash is 7 damage then 5 block; Bellow is 3 Strength before 6 block. Incantation
applies Ritual 3; Cultist attack base damage is 6. Refresh persistent enemy damage
and published intent at the evidenced notification points. A queued enemy attack
may refer to its refreshed damage entry, not an immutable early damage snapshot.

Preserve Ritual's skip-first lifecycle and Vulnerable's stacking/expiry flags.
With the supported initial state, Cultist's first attack after Incantation is 6;
later growth is 3 per growing round until Strength caps at 999. Damage/block
arithmetic preserves evidenced nonnegative clamp, multiplication, floor and cap
stages, including the block cap of 999. Demonstrate safe intermediate numeric
bounds in the plan; no unchecked overflow, early integer truncation, float chance
approximation or generic Java numeric emulator.

Preserve ordinary end-turn discard order, hand cap 10, draw-pile end orientation
and update-time draw sizes. Split insufficient-pile draws into existing draw,
discard recycle/shuffle and remaining draw in the accepted order. Do not combine
draw/discard before shuffling or draw with replacement.

Enemy selection must implement the exact two-stage law and history timing in
[combat section 3](2026-10-16-python-combat-behavior-design.md#3-conditional-chance-fidelity-is-mandatory).
Cultist choices are forced and use no AI words. Jaw Worm first selection is Chomp;
later integer buckets 0–24/25–54/55–99 select Chomp/Thrash/Bellow, with restricted
fallback Bernoulli probabilities respectively `9/16`, `2994733/8388608` and
`3489661/8388608`. Sample fallback only when required, append history once on
selection and execute the already published move before rolling a later move.
Prove agreement with the exact composed table; matching aggregate frequencies is
insufficient. Do not substitute nominal decimal thresholds or a one-draw aggregate
sampler that changes project consumption.

## 6. Randomness and cross-language replay

Keep `RNG_CONTRACT_ID = "spire-lab-rng-hmac-sha256-v1"` unchanged. Implement the full
existing derivation/framing, HMAC-SHA256 block/lane order, rejection sampling,
rational Bernoulli, weighted sampling and descending Fisher–Yates semantics.
Validate parameters before sampling. Preserve zero-word endpoint/bound-one and
empty/singleton behavior. Use the unchanged public/project fixture at
`tests/python/fixtures/combat/project_rng_v1.json`; it is not original-game evidence.

Represent the next-word cursor exactly over `0..=2**64`, including the exhausted
sentinel. Use a checked wider cursor or equivalent explicit exhaustion state, never
wrapping `u64`. Sampling failures retain the latest consumed cursor. Combat owns
one shuffle and one enemy-AI stream with validated shared combat scope and enemy
ownership. Derive streams independently; no HP stream is sampled at supplied entry.

Only semantic stream identity, key and next-word index determine replay. Do not
add a shared cache, alternate generator or incidental consumption changes. Any
local derived cache must be reconstructible and irrelevant to snapshot equality.
Future parallel scheduling must not affect an episode's draws; this milestone
introduces no parallel pool or episode allocator.

## 7. Terminal and fault handling

Return terminal only after the supported terminal work completes. Record
`hp_before_healing` immediately before victory healing; Burning Blood heals a
living player synchronously by 6 capped at max HP. Record `hp_after_cleanup` after
cleanup. Death does not heal and records zero for both values. Do not run later
enemy selection after damage kills the player or invent simultaneous-lethal rules
for excluded content. Do not reset every pile/block value merely because combat
ended; preserve contracted ownership and diagnostic state.

Distinguish invalid caller input/action from in-resolution unsupported mechanics,
invariant failure, RNG exhaustion and effect-guard faults. Resolution faults retain
partial state, the first fault and current/pending diagnostic work. They are not
losses, rollbacks or retriable player decisions. Recovery requires validated
restoration of a nonfaulted snapshot or reinitialization; no clear-fault-and-resume.

Preserve the per-external-step dispatch budget: dispatch 99,999 may finish;
attempted dispatch 100,000 is counted and faults before its payload. Safe-point
resumption retains that same step's count. Queue inspection does not spend budget.
Checked numeric failures become explicit faults rather than wrapped gameplay.
Process aborts/allocation failure are not claimed recoverable simulation outcomes.

## 8. Snapshots, safe points and semantic comparison

Use the existing semantic snapshot schema **2**, content identity
`spire-lab-combat-reference-v2`, and unchanged RNG contract. These identify the
supported semantics, not a shared Python/Rust memory layout. No serialized snapshot
wire protocol or direct loading of Python in-memory objects is introduced. Reject
schema 1, wrong content/contract and inconsistent snapshot state explicitly.

Snapshots own complete state, including lifecycle flags, histories, ordered piles,
in-play ownership, damage entries, public knowledge, RNG and work/continuations.
They remain unchanged when the live or restored combat advances. Restoration
validates before constructing a usable combat; faulted snapshots are diagnostic
only. Nonfaulted terminal snapshots are restorable as terminal, not live decisions.

Test internal replay between completed effect dispatches, with no interrupted
current payload. Preserve pending work, continuation and the current step's budget;
resume through an internal resolver entry, not normal action submission. Arbitrary
instruction-level interruption or replay of a partially applied effect is excluded.
Production callers need not receive an effect-step control API merely to test this.

For Python/Rust comparison, normalize decision/terminal checkpoints into explicitly
named semantic components: phase/turn, player/enemy values and power flags, damage
entries/intent/history, ordered piles/in-play/issued identities, RNG identity/keys/
cursors, public knowledge, outcome/HP values and fault kind/count where applicable.
Compare public observations and legal actions separately. Sort only unordered maps
and sets; never reorder piles or actions. Compare values exactly, without floating
tolerance or raw-object/serialized-byte equality. Human diagnostic message wording
need not match.

Internal effect queues may have different Rust representations. Check each
engine's complete safe-point replay independently; compare cross-language internal
continuations only at explicitly matched semantic boundaries. Do not mask a real
gameplay difference by dropping relevant state from the checkpoint comparison.

## 9. Verification and evidence boundaries

The implementation plan must define concrete tests and red/green commands for:

1. Constructor bounds, content/phase/stream ownership, retained-input independence,
   invalid-action atomicity and explicit unsupported-state handling.
2. Public HMAC known answers and the unchanged fixed RNG fixture, all four lanes,
   block boundaries, final word/sentinel, forced rejection/exhaustion, exact
   consumption, identity framing/domain isolation and all small shuffle paths.
3. All 100 Jaw Worm buckets for every relevant history class, exact restricted
   fallback weights and composed probabilities, forced moves, repeat restrictions,
   selection/history timing and AI consumption. Frequency smoke is supplemental.
4. Base/upgraded cards, Bash/Vulnerable ordering, damage/block floors/caps, Ritual
   and Vulnerable lifecycle, persistent damage/intent refresh and turn boundaries.
5. Ownership/order through play/discard, hand cap, empty/insufficient draws and
   recycle, including singleton/empty shuffle consumption.
6. Complete victories/deaths against both enemies; independently pinned authored
   cases include Cultist terminal HP `40/46` and Jaw Worm `29/35` before healing/
   after cleanup, using the fixed scenarios/actions in
   `tests/python/combat/test_vertical.py`. These are not original captures.
7. Independent capture-time snapshot baselines, detached restores/divergence values,
   compatibility rejection, supported internal safe-point resume and state/queue/RNG
   preservation on faults, including exact 99,999/100,000 dispatch boundaries.
8. Hidden-state noninterference, deterministic public action order, terminal/fault
   API behavior, replayed legal walks and test-only cross-language checkpoints.
9. Existing Rust trace and Python regression suites, preserving all prior RNG code,
   tests and fixed fixture. Do not weaken Python tests to accommodate Rust output.

Use a narrow test-only interchange for authored scenarios, action sequences and
normalized checkpoints, not a product capture/transport framework. The plan must
pin its field mapping, exact integer/handle encoding, Rust test executable entry
and Python orchestration before implementation. Rust-only correctness tests must
run without Python; differential tests may invoke both implementations but Python
is never a runtime dependency of `spire-core`.

Expected values and probability laws must be fixed independently of the Rust code
under test. Cross-language disagreement is a finding to investigate against the
reviewed mechanics and independently derived examples, not permission to patch
either engine until they agree. Any needed original access requires separate
approval. Do not conceal disagreements with skipped tests or regenerate expected
fixtures from failing product output.

Source-derived facts, selected-bytecode confirmation, authored derivations,
self-tests, differential agreement and original-runtime evidence remain separate.
Private originals/raw evidence stay outside Git; existing quarantines, failed
attempts and provenance qualifications remain intact. No original capture is
performed or claimed by this milestone. Later fidelity promotion still requires
the existing original seed/interaction/terminal/platform evidence; this design
neither waives it nor restores it as a prerequisite for starting Rust.

## 10. Completion and next handoff

After implementation approval, completion requires the scoped native behavior,
reviewed tests/differential results, replay/visibility/fault guarantees and fresh
verification evidence. Preserve the selected Sol implementer/fresh independent
task review and whole-slice review workflow in the implementation plan. Disclose
remaining evidence limits and any corrections to Python as separately reviewed
changes with regression tests, not silent oracle repair.

Hand off the single-combat API and deterministic scenario/action/checkpoint corpus
to later native batching/bindings and representative release benchmarking. Do not
claim optimized throughput, learning gains or game transfer from this slice's unit
tests. Do not add speculative performance gates, full-game content or capture
infrastructure before that handoff. Later design/plan approvals govern those
milestones; the goal remains the first narrow, measured learning loop, then the
approved phased expansion.

Written-spec approval permits writing-plans, not code. The next authorization is
review/start approval of the implementation plan. No dependencies, original
access/execution, staging, commits, pushes, cleanup or implementation are authorized
by writing it.
