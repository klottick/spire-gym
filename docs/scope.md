# Repository scope and current direction

Scope reaffirmed by the user on 2026-10-16, with the learning-direction amendment
approved on 2026-10-17 below. This is the current project overview;
the [approved ML-first design](superpowers/specs/2026-10-11-spire-lab-ml-first-design.md)
and subsequent approved narrow specifications supply the detailed contracts.
Historical plans/reports retain their original statuses and evidence limitations;
this overview does not retroactively accept them or authorize new execution.

## Approved source-first sequencing amendment (2026-10-16)

The user explicitly approved moving original-game validation to a later promotion
gate rather than blocking Rust implementation. This supersedes earlier requirements
that original combat captures must precede Rust development/handoff. Build from
reviewed, independently worded mechanics, checking ambiguous decompiler details
against bytecode only under applicable access approval; do not copy or mechanically
translate original methods into product code.

The source-first milestone is governed by the
[narrow Rust combat specification](superpowers/specs/2026-10-16-source-first-rust-combat-design.md)
(approved and implemented) and its separately approved
[implementation plan](superpowers/plans/2026-10-16-source-first-rust-combat.md),
not Linux capture readiness. Its core-only handoff is verified; capture-helper/
feasibility work remains deferred.
Rust development, differential testing, then separately scoped native batching,
bindings and benchmarks may precede original captures. This sequencing approval
does not approve an unwritten Rust spec/plan or authorize implementation, training,
new original access/execution, dependencies, staging, commits, pushes or cleanup.

Until reviewed original comparisons satisfy the applicable evidence gates, label
the simulator **source-grounded, not original-runtime validated**. Python/Rust
agreement, invariants and authored cases are implementation evidence, not proof of
original fidelity or demonstrated original-game transfer. Original seed/interaction/
terminal/platform requirements remain requirements for later fidelity promotion;
they are not prerequisites for beginning the source-first Rust milestone. Preserve
all historical evidence and qualifications without retroactive repair.

## Goals, in priority order

1. **An AI that plays Slay the Spire and improves.** The ultimate objective is
   Heart victory across the approved progression. Measure held-out playing strength
   gained per wall-clock hour, not only implemented content, optimizer activity or
   environment steps per second. Report Act 3 completion separately from Heart
   victory and simulator performance separately from original-game transfer.
2. **An optimized, game-correct Rust StS simulator enabling that learning.** Build a
   Rust-native, game-specific core, native batched environments and thin Python
   bindings. Profile and benchmark representative release workloads; optimize the
   actual simulation/encoding/inference/training bottlenecks rather than presumed
   ones. High throughput alone is not evidence that an agent has learned.
3. **Only the supporting tools needed for those goals.** A readable Python combat
   reference reduces mechanics risk during Rust development; original-game
   comparisons later validate fidelity. They are not competing production
   simulators, a Python optimization project or a general-purpose capture/validation
   platform.

## Headless architecture

- **Rust core:** owned combat state, content/rules, effects, randomness and replay;
  no renderer, Python interpreter, tensor-library or training dependency.
- **Rust environment layer:** scenarios, public observations, legal actions and
  native batching. Keep hot-path work native, not a Python call per effect.
- **Thin bindings and Python learning:** numeric batch interchange, models,
  rollout/training, baselines, evaluation and experiment configuration.
- **Separate offline validation:** original observations, reviewed semantic
  comparison cases and diagnostics; not a runtime dependency of training.

The product is entirely **headless and programmatic**. No graphics, animation,
audio, human-playable client or interactive game UI. Semantic actions and decision
boundaries are APIs, not human input prompts. Original-game observation tooling may
need the original game's own runtime/platform; that does not make the simulator
playable or require a running game, display or Windows connection for training.

Policies use player-visible information, never hidden RNG state, unseen draw order
or privileged evidence metadata. Preserve intended conditional probabilities,
gameplay dependencies and project replay under the approved
[RNG-001 deviation](intentional-deviations.md); original seed/PRNG parity is not the
target. Do not describe the changed joint random process as identical to the game.

## Approved two-level learning direction (2026-10-17)

The user approved **separate combat and run-level policies initially, backed by
one authoritative Rust simulator**, toward one system that can play the full game.
This records the approved architectural direction, not an implemented ML system
or a detailed training/API specification.

### Policies and model boundaries

- **Combat policy:** cards, targets, play order, potion use and ending turns;
  learn to survive while preserving HP and other resources for the run.
- **Run policy:** card rewards and skips, map/path choices, shops, events,
  resting/upgrading and other supported between-combat decisions; learn to build
  a deck and resource position that can win the run.
- Start with separately trainable models/checkpoints so one policy can be frozen
  while evaluating or training the other. Keep interfaces modular and replaceable.
  Shared card/deck/relic representations with separate policy/value heads are a
  later measured option, not an initial requirement. A single unified network is
  also an experiment, not the required architecture.
- Both policies use player-visible information only. Combat decisions may need
  public run context for resource conservation; neither policy may consume hidden
  RNG state, unseen draw order or privileged simulator diagnostics.

### One simulation, two training interfaces

All production gameplay rules, state transitions, randomness and native batching
remain in Rust. Python owns ML training/evaluation and thin numeric interchange;
the Python combat reference remains an offline correctness aid, not a production
simulation layer. The Rust simulator can run without Python.

Combat training can start at a supplied combat state and step individual combat
decisions. Run training can step strategic decisions and delegate each fight to
the combat policy until combat completes, returning the actual resulting run state
including consumed resources and terminal cleanup. The run learner may treat a
fight as one larger transition, but the same detailed Rust combat engine still
executes it; there is no second approximate simulator or initial fixed-HP-loss
shortcut. A learned combat-outcome predictor may be investigated later as a
planning accelerator, with its errors measured against detailed simulation.

### Training progression and common objective

1. Establish combat competence and evaluate it on the supported content.
2. Once run simulation/content is supported, train strategic choices against a
   frozen combat checkpoint to make their consequences more stable.
3. Alternate policy improvements, retaining older checkpoints for robustness and
   evaluating the combined system rather than assuming independent gains compose.
4. Evaluate held-out full runs as support expands, ultimately targeting Heart
   success and playing-strength gains per wall-clock hour; report Act 3 separately.

Combat victory alone is a bootstrap objective, not the eventual full-game target.
HP, potions and other remaining resources affect future success. Card/draft value
also depends on the combat policy's ability to use the resulting deck; account for
this coupling instead of optimizing two incompatible local objectives. Exact
rewards, value targets, model sizes, update schedules and interfaces require later
specification and evaluation; this amendment does not silently set them.

This approval does not expand the current starter-card combat content, require a
full-game engine before the first combat learner, or authorize implementation,
training, dependencies, original access, staging, commits, pushes or cleanup.
Keep the existing progression and obtain separately reviewed specifications/plans
and start approval for later implementation milestones. Original-runtime fidelity
remains unvalidated until its own evidence gates pass.

## Repository boundary and Gymnasium direction (2026-10-17)

The user approved tested Gymnasium compatibility as part of the next batched
combat environment/bindings milestone: thin single-environment and native-vector
interfaces over the same Rust engine and public observation encoder, not a Python
simulation or a Python loop over environments. Concrete interface/reset/action
contracts were specified, reviewed and separately authorized for implementation.
The supplied-entry combat slice now has native batching and tested single/vector
Gymnasium facades with masked actions. This is a bounded interface implementation,
not full-run support or a promise of compatibility with arbitrary learners, mask
conventions or vector adapters; framework-specific adapters are added only when a
selected learner needs them. Fresh exact-target extension and full-suite controller
verification remain pending.

The repository is now named **spire-gym** on GitHub under **klottick**. The user
separately authorized public MIT-licensed repository creation and publication of
the relevant authored files. Local directories, existing package/crate names and
historical documents need not be renamed. A later repository split may move ML
training, experiments and checkpoints out while retaining the Rust simulator,
environment/bindings, correctness tests and examples here; no migration is started.
This administrative publication does not authorize new simulation implementation,
training, dependencies, original access, evidence promotion or cleanup.

## Delivery path

1. Design and plan the source-first narrow Rust combat slice from the reviewed
   mechanics and completed Python reference, then obtain implementation approval.
2. Independently author that Rust combat slice and differential
   checks; shared semantic cases are not mechanical translation of rule code.
3. Add native batching/public encodings and thin Python bindings, verifying
   ownership, replay, masking, reset/terminal semantics and hidden-information
   boundaries. This milestone is implemented for the bounded supplied-entry
   combat slice; it does not provide run-level environments or new gameplay content.
4. Benchmark representative release builds and compare a first learner against
   random and fixed public-information heuristic baselines on held-out scenarios.
   Report learning curves against decisions and wall-clock time.
5. Expand through the approved roadmap using correctness and learning results.

Perform original-game comparisons as a later fidelity-promotion milestone under
separately approved capture scope. They remain necessary for validated-fidelity and
original-game-transfer claims, not for starting Rust. Each later implementation or
training milestone still requires its own reviewed plan and start approval.

The first learning loop is deliberately narrow: Ironclad A0 starter cards against
Cultist/Jaw Worm. Do not wait for a full-game engine to start the first supported
training slice, or label that slice unrestricted StS mastery. Existing design,
plan/start and evidence approvals still apply to each actual implementation step.

Content progression remains combat → Act 1 Ironclad → full-game/Heart Ironclad →
A0–A20 → the same progression for Silent → Defect/Watcher → Daily modifiers under
their original mode rules. Content support is not evidence of agent competence.

## Current implementation status

This table describes the implemented source tree and its recorded verification,
not a claim that the planned interfaces or full-game content already exist.

| Area | Status |
|---|---|
| Rust/Python packaging and device readiness | Implemented smoke/readiness tooling; not a trained agent or simulator-performance result |
| Narrow Python combat reference | Implemented and independently reviewed: supplied ready state, base/upgraded Strike/Defend/Bash, Burning Blood, one Cultist/Jaw Worm, necessary powers/piles/turns, owned project RNG and schema-2 replay |
| Python verification | Reviewed reference baseline: 735 tests, including 52 whole-combat acceptance cases. Final minor-fix verification: 830 full-suite tests and all 95 differential tests pass, including above-CPython-threshold handles and isolated reveal execution. Native provenance correction and adapter fix are independently reviewed; historical failed exchanges remain retained. The unchanged default Python product engine still has its enormous-handle reveal conversion limit. Not original-runtime correctness evidence |
| Original combat validation | Incomplete; the reference remains static-evidence-grounded, **not original-runtime validated** |
| Narrow Rust combat core | User plan/start approval executed; supplied-entry combat, project RNG, public actions/views, snapshots, native acceptance and test-only differential harness implemented. Historical core-handoff controller workspace/release runs each passed 124 native tests; complete differential checkpoints agreed, including both formerly failing fault cases. Task reviews, whole-slice review, final correction review and controller verification were complete for that core-only handoff, not the later batching/Gymnasium integration. **Source-grounded, not original-runtime validated**; no optimization/throughput claim |
| Native environment pool/batched interface | Implemented for bounded supplied-entry Ironclad A0 combat with public observations, replay, native workers and single/vector Gymnasium facades. This integration candidate freshly passed 169 offline Rust workspace tests; the retained 167 environment Python tests used a previously built extension. Exact-target extension/full-suite controller verification pending. Not full-run support; required-CUDA readiness has two retained OOM failures |
| Learning/training/evaluation system | Not implemented; no learned playing-strength or release simulator-throughput result claimed |
| Linux capture readiness | Approved specification retained; helper/feasibility work deferred in favor of source-first Rust. A concrete capture consumer remains required if resumed |
| Windows real capture adapter/platform gates | Incomplete/fail-closed; Linux readiness does not satisfy them |

## Validation scope and stopping rules

The [Linux readiness specification](superpowers/specs/2026-10-16-linux-validation-readiness-design.md)
is supporting work for the next actual StS comparison, not a standalone product.
Before planning helper implementation, identify the concrete next validation case,
intended invocation boundary, observations, unknown runtime/adapter facts and
required approvals. If there is no usable capture path yet, propose a bounded
feasibility investigation rather than building unused generic infrastructure.

Keep necessary identity/privacy checks, timeouts, independent child/descriptor
cleanup and truthful failure retention. Stop readiness work once its required
controls and original-validation handoff are complete, if resumed; do not add
speculative helper layers or block the now-approved source-first Rust path on them.
Do not create new promotion gates without evidence or user-reviewed scope changes,
and do not waive existing gates to claim completion sooner.

Original validation still needs complete supported combats at no fewer than three
original seeds per enemy, the required interactions and both terminal HP
observations, plus applicable adapter/export/load/visibility/non-interference and
Windows gates. Exact conditional AI laws remain required despite RNG-001. Python
self-tests and later Python/Rust agreement cannot replace original observations.

No playable client, generic card-game engine, generic capture framework, unrelated
infrastructure, private game redistribution or installation is implied by scope
approval. Preserve private raw evidence outside Git and historical failures as
recorded. Approval of a written spec permits planning, not execution of a plan
that has not yet been reviewed; no automatic staging, commit, push or cleanup.
