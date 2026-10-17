# Python combat reference architecture for Phase1a

> **Approved sequencing amendment (2026-10-16):**
> [Source-first Rust development](../../scope.md#approved-source-first-sequencing-amendment-2026-10-16)
> supersedes original-capture prerequisites for Rust handoff below. Original-runtime
> comparisons remain required for later fidelity promotion, not for starting Rust.
> No historical evidence is upgraded, and Rust design/plan/start review still applies.

> **Current scope/status (2026-10-16):** This is the historical reference-architecture
> document. The subsequent [combat behavior specification](2026-10-16-python-combat-behavior-design.md)
> and plan delivered the narrow Python reference; it remains not original-runtime
> validated. Per [repository scope](../../scope.md), Python is a readable correctness
> aid for the optimized headless Rust simulator and Python learning system, not a
> competing production engine or a human-playable game. Original dated status and
> approval boundaries below describe this earlier document, not current execution
> authorization. No evidence gate is waived by this status note.

> **Approved RNG policy amendment (2026-10-16):**
> [RNG-001](../../intentional-deviations.md#rng-001--decorrelated-reproducible-project-randomness)
> intentionally replaces exact original RNG/seed/stream parity with reproducible
> project randomness decorrelated across unrelated mechanics. Preserve intended
> probabilities and gameplay dependencies. Conflicting exact-RNG requirements below
> are historical; other mechanics and original-validation gates remain unchanged.

Date: 2026-10-14

Status: Architecture draft only. This document does not authorize implementation,
game-artifact access, amendments to prior contracts, or changes to the approved
ML-first design and 16-task plan. A reviewed implementation plan and a separate
explicit start decision remain future gates.

## 1. Goal and scope

Establish a readable, independently authored Python behavioral reference for one
complete supported combat: Ironclad at Ascension 0, base and upgraded Strike,
Defend, and Bash, Cultist or Jaw Worm, and Burning Blood. It models the supported
combat from a defined combat-entry state through player decisions and automatic
resolution to a terminal result. It is not a complete run or a general Slay the
Spire engine.

Include only mechanics required to resolve that boundary: HP, block, energy,
draw and shuffle, turn reset, Strength, Ritual, Vulnerable, card play, enemy
intent/action needed by these encounters, ordered effects, victory/death, and
combat-end healing. The exact rules, ordering, arithmetic, and RNG algorithms
are evidence requirements, not presumed facts. Unsupported behavior fails
explicitly.

Exclude potions, other cards/relics/characters/encounters, acquisition, map/run
progression, other acts, full-game coverage, training/RL, throughput optimization,
and a new transport/live-adapter contract. Do not repair or extend frozen Task 2
work under this architecture.

## 2. Design principles

1. Python is the readable behavioral reference, not a performance target. Keep
   rule execution ordinary Python 3.12; do not import NumPy, tensors, or native
   code into the rules path.
2. Author rules independently from reviewed behavior. Do not mechanically
   translate Java code or include private source, bytecode, binaries, or assets.
3. Preserve the existing design's cohesive owned combat state, stable instance
   identity, explicit ordered effects, exact RNG, deliberate arithmetic, and
   separation of public information from privileged validation state.
4. Keep unresolved mechanics visible. No guessed default, no silent no-op, and
   no fabricated loss may turn missing evidence into apparent support.
5. Python self-consistency and later Python/Rust agreement are useful checks, but
   neither proves correctness against the original game.

## 3. Proposed package responsibilities

This is a suggested minimal organization, not a ratified file/API contract. Keep
it inside the existing `python/spire_lab` package; adapt names in an implementation
plan to repository conventions rather than introducing a framework.

| Module | Responsibility |
|---|---|
| `combat/model.py` | Small state/value types: combat, player, enemy, card instance, powers, ordered piles, public knowledge, pending work, and terminal outcome. |
| `combat/content.py` | Definitions for only the approved cards, encounters, and Burning Blood; identity/version of the supported content set. No executable rule bodies copied from game code. |
| `combat/rng.py` | Evidence-backed game-equivalent RNG state, bounded sampling, and shuffle operations; explicit unsupported status until verified. |
| `combat/rules.py` | Arithmetic helpers and small rule calculations, including evidenced integer width, wrapping, rounding, and floating-point behavior. |
| `combat/engine.py` | Action validation, explicit ordered scheduler, effect application, turn progression, next-decision/terminal transition, and typed failures. |
| `combat/view.py` | Player-facing view and legal-action enumeration, kept separate from full debug state. |
| `combat/snapshot.py` | In-memory clone/snapshot, restore validation, and comparable debug representation for diagnosis. |

Tests and reviewed fixtures should live in the project's normal test/fixture
locations chosen by the later plan. Do not add a generic plugin system, ECS,
custom serialization framework, or broad protocol package for this slice.

## 4. State and transition model

One combat state owns all mutable data needed to continue and explain resolution:

- Supported content identity, character/difficulty, turn/phase, and terminal state.
- Player HP/max HP, block, current/max energy, powers, turn counters, and ordered
  hand/draw/discard/exhaust piles.
- Enemy instances with stable IDs, HP/block/powers, current intent, and only the
  AI/move history needed by the supported encounter.
- Stable card-instance IDs and mutable instance attributes such as upgrade state;
  definitions remain immutable. IDs are never reused during a combat.
- Named RNG streams and exact mutable state/consumption information once evidence
  establishes which streams exist and how they are shared.
- An ordered queue of explicit effect/work values, scheduler phase, and any
  suspended continuation or pending player decision.
- Publicly revealed information separately from hidden pile order and validation
  data. A public view must not expose internal RNG, unrevealed draw order, or
  hidden AI state merely because the reference stores it.

Piles and queued work are ordered. A pile index or enemy slot is a location, not
identity. Use small dataclasses/enums or equivalent ordinary types; avoid
closures in queued work so snapshots and first-divergence reports can explain it.

Conceptual interface (names illustrative):

- `initial_combat(scenario, seed_or_rng_state) -> CombatState` constructs only a
  supported, evidence-defined entry state.
- `legal_actions(state) -> ordered actions` enumerates decisions currently
  permitted by the supported rules.
- `step(state, action) -> Transition` validates before mutation and resolves
  automatic work until the next player decision, terminal result, or typed error.
- `Transition` communicates the resulting state and whether it is a decision,
  terminal outcome, or explicit failure; it does not pretend every step is a
  player turn.
- `public_view(state)` exposes only decision-relevant player-visible facts.
- `snapshot/restore` and `compare` support internal debugging, not a wire format.

Actions are semantic (for example play this card instance at this target, or end
turn), not opaque numeric positions. Invalid actions leave the state unchanged.
Errors distinguish invalid action, unsupported mechanic, invariant violation,
and resolution guard exhaustion. Invalid actions are rejected before mutation.
Unsupported-mechanic, invariant, and guard failures retain the diagnostic partial
state and mark it faulted; it cannot be stepped or accept normal actions. Recovery
requires restoring a valid nonfaulted snapshot or constructing a new combat. The
fault record/state cannot be silently cleared or resumed, and recovery is not
automatic rollback. Faults are not terminal outcomes and never become fabricated
losses.

Apply the inherited limit of 100,000 resolved effects per external `step`. Count
each effect when the scheduler resolves it, deterministically, including effects
that later fail; do not count mere queue inspection as resolution. On reaching the
limit, stop, retain the diagnostic partial state and fault details, and fail the
experiment. Never reinterpret exhaustion as a loss. The guard and preservation
policy are established constraints, not open implementation choices.

The scheduler processes only evidenced phases needed here: card action and its
ordered effects, turn-end work, enemy actions, next-turn preparation, and terminal
cleanup. Each effect records enough source/target and calculation timing to
preserve the established behavior. Do not assume all quantities are calculated
when queued or when applied; establish timing from evidence. Victory and death
precedence, cleanup, and Burning Blood healing order must be explicit once
verified. Always report HP immediately before combat-end healing and HP after
terminal cleanup; establish the exact observation point and healing/cleanup timing
from evidence. These two values are required diagnostics, not optional fields.

## 5. Exactness: arithmetic, RNG, and timing

Python's unbounded integers and numeric defaults are not a substitute for the
game's behavior. Before declaring a mechanic supported, establish whether its
operations use fixed-width words, signed/unsigned conversion, wrapping, truncation,
rounding, or floating point. Implement narrowly scoped helpers with named intent;
do not scatter implicit coercions through card code. Tests of helpers demonstrate
the authored implementation, not the original rule absent external evidence.

Likewise, determine the relevant RNG algorithms, named-stream topology, bounded
sampling, shuffle procedure, seed/state representation, and consumption order.
Do not substitute Python's `random` or invent independent streams for convenience.
Any unspecified or unverified RNG-dependent transition is unsupported. Snapshots
must retain enough evidenced RNG state to reproduce subsequent transitions.

Order is semantic: effect insertion/processing, card movement, draw/shuffle,
enemy action, death/victory checks, and cleanup can change outcomes. Represent
order directly and test it. Do not infer a timing rule from a card description or
from agreement between two implementations.

## 6. Internal comparison and visibility boundary

The reference needs internal snapshots for cloning a state, restoring a state,
comparing two executions, and reporting the first meaningful divergence. A
comparison should identify the differing conceptual component (phase, player,
card instance/pile order, enemy, pending effect, RNG, or terminal/cleanup values)
and retain enough context to diagnose it. It need not commit to serialized field
names, ranks, byte encodings, digests, or a transport schema.

Keep three views conceptually distinct:

- Public view: facts available to the player at this decision; suitable for a
  future environment/agent boundary.
- Internal debug state: complete owned state for replay and diagnosis.
- Evidence metadata: provenance and validation notes, not game state and never
  an agent feature.

This comparison mechanism is not the frozen trace-v1 reader/writer contract, a
replacement terminal contract, or authorization to amend existing protocol
decisions. A later Rust implementation should independently choose its own
internal representation and rule code. Only agreed semantic scenario inputs,
actions, outcomes, and comparison meaning need to be comparable.

## 7. Evidence gates and validation stages

Evidence acquisition is a separate prerequisite, not work authorized by this
architecture. Existing static-inspection coverage and frozen capture findings
remain exactly as stated in the drafting brief. This design does not continue,
reset, or relabel those efforts. No private artifact or original runtime access
is authorized now. Any future source, bytecode, isolated execution, or live capture
requires its own applicable approval and identity-bound scope. Preserve the
existing requirement for original Windows capture/adapter/non-interference gates;
Linux headless execution is not promised.

Track each mechanic claim with its evidence class and identity where applicable:
reviewed original-runtime behavior is the correctness anchor; source-derived
claims, bytecode-confirmed inspection, isolated original-bytecode execution, and
live captures remain distinct classes. An isolated probe cannot establish
whole-combat or live-adapter correctness. Conflicts or missing facts block the
affected mechanic; do not promote self-authored examples or parity into evidence.

Suggested stages, each requiring its own later authorization as appropriate:

1. **Design/plan review:** confirm ownership, supported content matrix, evidence
   questions, and bounded implementation work. Resolve required unknowns before
   coding a rule as supported.
2. **Independent evidence acquisition:** under explicit scope, collect only the
   approved mechanic evidence and provenance; keep private derived material out
   of the repository. Do not infer that prior incomplete/frozen work is reopened.
3. **Provisional reference:** author rules independently. Internal tests may
   establish invariants and consistency, but status remains unvalidated where
   original evidence is absent.
4. **Original-behavior validation:** review exact expected behavior from relevant
   original evidence, then run supported whole-combat cases against it. Record
   evidence class and outcomes; unresolved disagreement blocks that mechanic or
   the claimed supported combat.
5. **Rust handoff:** only after the reference's supported behavior and comparison
   cases are reviewed, plan the independently authored Rust slice and differential
   checks against identical inputs/actions. Rust parity supplements, never
   replaces, original-game validation.

## 8. Test strategy

Use distinct test categories and label what each can establish:

- **Unit/invariant tests:** legal ranges, pile/ID uniqueness, energy and HP
  invariants, queue consistency, and snapshot/restore round trips. These test
  implementation consistency only.
- **Reachable-state properties:** generate states by supported combat setup and
  legal action sequences, then check invariants and determinism. Do not generate
  arbitrary dictionaries and call them valid game states.
- **Reviewed original-behavior fixtures:** small, explicit input/action sequences
  and expected observable transitions/outcomes, with provenance/evidence class.
  Golden fixtures contain only reviewed expected behavior, not private code or
  artifact material.
- **Vertical combats:** exercise complete supported entry-to-terminal sequences
  across the card/enemy scope, including draw/shuffle, power timing, effect order,
  death/victory, cleanup/healing, and terminal state. Expected outcomes must be
  externally grounded before they count as correctness fixtures.
- **Differential diagnostics:** later compare Python and Rust on matching semantic
  cases and produce first-divergence reports. This finds disagreements; it does
  not decide which side is correct.

Include negative checks for invalid actions leaving state unchanged, unsupported
mechanics failing explicitly, snapshot incompatibility, faulted-state rejection of
steps/actions, and guard exhaustion preserving a diagnostic faulted partial state.
Test recovery only by restoring a valid nonfaulted snapshot or initializing anew;
verify the faulted state itself cannot be cleared/resumed. Keep terminal outcomes
distinct from faults. RNG tests must cover state restoration and consumption/order
as established by evidence.

The minimum original-validation matrix inherits the approved design: complete
Cultist and Jaw Worm combat traces at no fewer than three RNG seeds each; base and
upgraded starter cards; block reset, Bash/Vulnerable, enemy buffs, shuffle,
victory/healing, death, controlled edge cases, RNG fixtures, and original-runtime
captures. Preserve the specific existing capture/adapter/non-interference gates.
This is a minimum, not a claim that these cases or evidence already exist. Provisional
self-tests, invariants, synthetic examples, and Python/Rust parity do not satisfy
original validation or permit a validated label.

Test the public-information property: states with identical public histories but
different hidden RNG state or unrevealed draw order produce identical public views
and candidate semantic descriptors/order. When an effect reveals order, retain the
revealed order in public history and test that it is represented consistently.

## 9. Bounded implementation and Rust handoff

The later plan should implement one coherent executable combat path, not separately
ratify every component or add reusable abstractions without demonstrated need.
Keep Python readable and inspectable; do not optimize it prematurely. Bound the
first supported content and test matrix to the scope in Section 1. Any expansion
requires explicit planning and evidence, not an incidental convenience change.

Rust remains the eventual optimized core in the established repository architecture.
It is to be independently authored, without Python rule calls or generated
translation. Differential testing shares only semantic case inputs/actions and
comparison meaning. A Python/Rust mismatch is a diagnostic requiring evidence-led
resolution; matching outputs alone do not validate either implementation.

## 10. Open decisions and risks

These are not silently selected by this draft:

- The precise supported combat-entry fixture and initialization ownership (including
  whether only narrowly constructed scenarios or captured entry states are used).
- The exact evidence acquisition scope, permitted methods, identities, budget, and
  approval sequence; none is authorized here.
- Which exact behavior facts are sufficiently established for each card, enemy,
  RNG operation, power, turn phase, and terminal/cleanup rule.
- Whether initial implementation needs every proposed module or should consolidate
  files; choose in the reviewed implementation plan based on clarity.
- Snapshot lifetime/format needs beyond internal clone and diagnostic comparison;
  no public persistence or cross-language wire encoding is selected.
- Diagnostic representation/details beyond the required retained faulted partial
  state, deterministic 100,000-resolved-effect per-step limit, and experiment-failure
  behavior.
- Evidence-backed ordering and arbitration when victory and player death appear
  in the same resolution window, and precise cleanup/healing timing (both pre-heal
  and post-cleanup HP remain mandatory reported values).
- Which additional whole-combat cases or capture details are needed beyond the
  inherited minimum before the label “validated” is used.

Primary risks are guessed mechanics hidden by a clean API, accidental exposure of
privileged state through the public view, Python numeric/RNG mismatch, and treating
internal consistency or Rust parity as external correctness. The plan must preserve
fail-closed support boundaries and report these risks rather than paper over them.
