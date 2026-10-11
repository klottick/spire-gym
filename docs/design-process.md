# Design process

## Brief

The user wants a new local repository under `~/repos/`, containing a Rust
Slay the Spire simulator and Python ML training. Efficient ML training is the
user's highest priority; learning Rust is a secondary benefit, not the primary
architecture optimization target. The user requested a full design and plan
before implementation.

`spire-lab` is the initial repository name, selected by the assistant and
subject to the user's preference. This is a local Git repository; no hosted
repository has been created.

## Checklist

- [x] Explore the existing C++ simulator and relevant forks.
- [x] Record confirmed intent separately from proposed assumptions.
- [x] Record the user's clarification: efficient ML training has highest priority.
- [x] Obtain approval to use the ML-first architecture review as the design basis.
- [x] Confirm the overall progression and select combat as the starting milestone.
- [x] Select Act 1-ready Ironclad combat rather than complete Ironclad combat
      through the Heart as the phase 1 scope.
- [x] Approve phase 1 supported content and correctness/learning completion gates.
- [x] Inspect server hardware and compare it with free Colab constraints.
- [x] Confirm that the user has Slay the Spire installed on Windows.
- [x] Confirm willingness to install testing mods and include the validation
      tooling in the Spire Lab repository.
- [x] Adopt the Linux server as the default development/training target and
      Windows as the game-validation target in the approved architecture.
- [x] Compare a Rust-native game-specific core, direct C++ port, and generic
      data-driven engine; user selected the Rust-native core.
- [x] Review the design in sections: state/effects, fidelity/RNG, decisions,
      observations/actions, Rust/Python boundary, training/evaluation,
      performance, repository structure, and phased scope.
- [x] Visual companion not needed: decisions were reviewed as textual contracts
      and a repository tree; no browser session was requested or opened.
- [x] Obtain conversational design approval, including Heart victory as the
      ultimate goal and Act 4 as part of full-game scope.
- [x] Write and commit the design spec under `docs/superpowers/specs/`.
- [x] Self-review the spec for scope, ambiguity, contradictions, and placeholders.
- [x] Obtain user review and approval of the written spec.
- [x] Write and self-review the 1a implementation plan.
- [ ] Obtain user review of the plan and the user's execution choice.

## Constraints and scope boundaries

- Rust for simulation; Python for training.
- Optimize for game-correct, held-out learning progress per wall-clock hour;
  simulator step throughput and Rust learning convenience are supporting goals.
- The user selected combat first, followed by Act 1 Ironclad, full-game
  Ironclad, Ascension 0–20 Ironclad, the same progression for Silent, remaining
  characters, and finally Daily run modifiers across those capabilities.
- Phase 1 targets Act 1-ready Ironclad combat for supported scenarios. Combat
  content expands with later phases rather than requiring all Ironclad cards,
  relics, potions, and encounters through the Heart before phase 2. Exact
  supported content is recorded in the approved phase 1 manifest; "Act 1-ready"
  does not claim unrestricted coverage of every combination obtainable in Act 1.
- No implementation scaffolding, dependency installation, or application code
  until the design and implementation-plan review gates are complete.
- Planning is allowed to cover the long-term roadmap, but the initial detailed
  implementation plan should target a small executable milestone.
- Repository creation and these planning notes were explicitly requested;
  they do not imply approval of the proposed architecture.

## Open decisions

- Review the 1a implementation plan; the written spec, including its concrete
  schemas, pilot defaults, and acceptance procedures, is now approved.
- Select the execution method and degree of user implementation after review of
  the written implementation plan; no implementation method is assumed yet.

## Selected architectural approach

Confirmed: build a Rust-native, game-specific simulator rather than a direct
C++ port or a generic card-game framework. Rust simulation, Python training,
and Windows game-validation tooling live in the same repository with separate
responsibilities. Existing implementations are mechanics references; observed
real-game behavior is the validation reference.

Section reviews:

- [x] Retain owned state, stable IDs, typed effects, and ordered resolution under
      the approved ML-first core direction; details receive written-spec review.
- [x] Approve supported-content fidelity/RNG targets, snapshots, and Windows
      differential validation workflow.
- [x] Approve training-facing observation/action principles and hidden-information
      separation; exact schemas still need specification.
- [x] Approve Rust/Python batching, buffer-lifetime, and reset contracts;
      concrete bindings/dependencies still need specification.
- [x] Approve initial model/PPO baseline, win objective, curriculum, and
      evaluation principles; defaults and numerical acceptance gates remain.
- [x] Phase 1 content and acceptance gates.
- [x] Approve repository layout, integration choices, and 1a -> 1b -> 1c order.

### Retained core model under the approved architectural direction

These principles were retained in the approved ML-first direction. Concrete
state fields and scheduler behavior will receive written-spec review.

- Immutable rules/content definitions are separate from mutable combat state.
- Each environment owns its player, cards and piles, enemies, powers, relic
  counters, potions, RNG streams, pending effects, and pending decision.
- Runtime card/enemy identities are stable IDs, not references or pile indices.
- Player actions and automatic effects are explicit enums and data, not queued
  closures or a per-card inheritance hierarchy.
- An ordered effect scheduler preserves game-specific trigger ordering and can
  pause/resume for intermediate player choices with explicit continuation data.
- One external step validates the action before mutation, then resolves
  automatic work until the next decision, terminal outcome, or explicit error.
- Unsupported effects and internal execution limits are errors, not victories,
  losses, or silently ignored actions.
- A snapshot includes all owned mutable execution state, RNG streams, and any
  suspended continuation. Start with normal cloning and optimize from profiling.
- Later run state composes with combat state; do not implement map/shop/event
  systems or speculative framework hooks in the first combat slice.

## Approved ML-first architectural direction

The following section reviews preserve the decisions as they were made.
Their then-open numerical/interface details are now resolved in the approved
written spec; plan review and execution choice remain separate gates.

The user approved the review as the basis for detailed design. Individual
interface contracts, content gates, rewards, and validation details still need
section review; this is not approval of a complete spec or implementation plan.

The previously proposed owned-state/effect model remains a reasonable core,
but is insufficient by itself to ensure efficient learning. The training data
path, representations, and evaluation must guide the implementation.

- Native Rust environment pool, with one batched Python boundary rather than
  Python calls/processes per environment or effect. Begin with synchronous
  batches; add asynchronous scheduling only if measured stragglers justify it.
- Contiguous reusable numeric observation/action/reward/status buffers. Keep
  serialized traces, debug strings, and per-step state cloning off the training
  path. Array ownership/lifetime contracts must prevent rollout data corruption.
- Compact shared rules, owned mutable state, typed effects and correct trigger
  order. Preallocate common buffers with explicit growth/overflow behavior;
  performance limits must never silently discard effects or game content.
- Structured observations for cards, enemies, powers, relic counters, and
  public draw-pile composition without unseen order. Share encoders where
  appropriate; benchmark simple models before increasing model complexity.
- Masked action candidates with card/target/choice features. Sequential choices
  for multi-card selection avoid enumerating all subsets. Internal entity IDs
  are not policy features, and legal-action generation must not leak hidden state.
- A small policy/value model trained with PPO is an initial measurable baseline,
  not a claim of algorithmic optimality. Compare feed-forward/public-history
  features with recurrence as needed. Search/imitation are later controlled
  experiments, not dependencies of the first training loop.
- Realistic scenario distributions with varied decks, HP, relics, potions, and
  opponents. Mix easier and harder cases; retain earlier cases to detect
  forgetting. Hold out complete scenario configurations and seeds, not just
  repeatedly reshuffle a single starting deck.
- Report learning curves against both training decisions and wall-clock time,
  across multiple training seeds. Compare random and heuristic baselines; report
  win rate plus HP/resource outcomes. Evaluate canonical victory separately
  from any shaped training reward and phase-specific curriculum success.
- Correct termination/truncation and recurrent-state reset handling. Auto-reset
  must preserve the final observation for value bootstrapping. Internal errors
  fail the experiment rather than being silently reset or counted as game losses.
- The actor and any search teacher use player-visible information. Debug traces
  and hidden RNG state remain confined to simulation and validation tooling.
- CPU worker counts, environment counts, inference device, batch sizes, and
  learning budget are tunable. Benchmark CPU versus GPU inference for small
  models; do not optimize for GPU utilization or raw simulator throughput alone.

Design references inspected: EnvPool's native batched/thread-pool execution
and PufferLib's reusable-buffer/batched-inference design. Neither is selected
as a dependency. Their published throughput figures are not estimates for this
game or this implementation.

- https://envpool.readthedocs.io/en/latest/
- https://puffer.ai/docs.html

No benchmark or training experiment has been run; the best model, algorithm,
and parallelism settings remain empirical decisions.

### Approved section: training-facing contract

The user approved this section. Exact field lists, capacities, and binding
signatures will be made explicit in the written spec.

- Keep a single-environment correctness API and a native batched pool that uses
  the same rules engine. Python training communicates with the pool through
  batch reset/step operations, not calls to individual game objects.
- Emit versioned, contiguous numeric buffers for global/player features, hand
  cards, enemies, public pile composition, powers/relics/potions, pending-choice
  context, candidate-action features, and padding masks.
- Return legal candidate actions per decision; an action index identifies a
  candidate in that environment's current decision, not a persistent global
  action label. Never silently clip candidate lists or observations to fit a
  buffer; capacities and overflow behavior are part of the contract.
- Multi-card choices are staged selections with explicit confirmation and no
  game effects until the original choice is committed. Selection bookkeeping
  must not introduce extra time discounting or reveal hidden information.
- Return per-environment rewards, terminated/truncated flags, final-observation
  data, and episode/reset metadata. Preserve the terminal decision's transition
  even when resetting that slot for further rollout collection.
- Buffers can be reused only under a documented lifetime contract: rollout
  storage retains copies before a subsequent call overwrites data, and GPU
  transfers finish before source memory is reused. Offer an owned-copy path for
  debugging and tests.
- Keep environment IDs, scenario seeds, privileged snapshots, and validation
  metadata separate from model inputs. Use the same observation encoder for
  the single-environment reference and the batch path.
- Validate single-versus-batch equality, masking, reset/final-observation
  semantics, buffer lifetimes, and invariance to hidden state before training.

### Approved section: initial learning setup

The user approved this section. Hyperparameters, content manifests, evaluation
corpora, and numerical acceptance gates will be specified separately.

- Initial learner: synchronous masked PPO with a shared compact encoder, a
  candidate-action scoring head, and a scalar value head. Both actor and critic
  initially use public information only.
- Share small encoders across cards and across enemies; pool entity features
  into state context and retain selected card/target features for candidate
  scores. Begin with a feed-forward model and explicit public-history features.
  Recurrent memory or attention are controlled variants, not initial requirements.
- Baseline combat objective: terminal reward +1 for victory, -1 for death,
  zero intermediate reward, and undiscounted finite-combat returns. Track HP
  and potion outcomes separately. Resource-aware rewards and potential-based
  shaping are versioned experiments, not silent changes to the win objective.
- No per-card damage rewards or arbitrary click penalties. Bookkeeping choices
  must not change the objective by adding time discounting. Administrative
  time limits remain truncations, not defeats; internal errors fail the run.
- Curriculum: basic supported fights/decks, then additional supported mechanics
  and opponents, then a mixed Act 1 distribution including elites/bosses and
  resource variation. Preserve earlier cases in later sampling mixtures.
- Keep separate training, model-selection, and final held-out scenario sets.
  Hold out both random seeds and realistic deck/encounter/resource combinations
  within implemented content; do not claim unsupported-card generalization.
- Compare random and fixed heuristic baselines on identical evaluation scenarios.
  Evaluate at least three independent training seeds for promoted configurations.
  Report win rate with uncertainty, HP/potion outcomes, timeout/error counts,
  decisions processed, and wall-clock learning curves.
- Select worker counts, environment counts, rollout lengths, model width, and
  inference device using bounded pilot experiments. No hardware utilization or
  throughput target replaces the held-out playing-strength objective.
- Search, imitation, recurrence, and reward variants are added one at a time
  against the baseline. Learned actor/encoder weights may initialize phase 2,
  but combat-only value targets/rewards must be reconsidered for full-run play.
- Quantitative phase 1 acceptance thresholds and default hyperparameters still
  require specification; no training performance has been measured or promised.

### Approved section: fidelity, RNG, and game validation

The user approved this section. Reference-build identifiers, fixture manifests,
and trace schemas will be specified in the written spec and capture workflow.

- Use a pinned Slay the Spire 1 game build and explicitly recorded test-mod
  versions as the reference. Compare mechanics-relevant canonical state and
  decisions; exclude rendering, animation, UUID implementation details, and
  unrelated internal data.
- Target exact action-by-action behavior and game RNG parity for supported
  combat content. Validate RNG primitives, shuffles, selection procedures, and
  stream consumption as well as damage/block rounding and trigger ordering.
  Do not postpone all training until unimplemented full-game RNG behavior matches.
- Own named game RNG streams per environment. Keep scenario sampling and policy
  randomness separate. Batch scheduling, logging, and worker count must not
  change an environment's transitions for the same initial state/actions.
- Provide ordinary full Rust snapshots for debugging/replay with all mechanics
  state, pending effects/choices, and RNG streams; cloning is not a per-step
  training requirement. Version snapshots and reject incompatible rules/schema.
- Include Windows game-side mod source and capture tools in this repository.
  Passive capture must not consume RNG or alter mechanics. Controlled scenario
  setup is a separate test mode and recorded as injected rather than natural.
- Capture a complete supported initial combat state at a clean decision boundary,
  including privileged RNG/pile information needed for replay, followed by
  semantic actions and canonical states at subsequent decision boundaries.
  Do not assume arbitrary mid-animation game snapshot import is supported.
- Store portable versioned traces, initially readable JSON-lines with optional
  compression. Replay them on Linux and report the first divergent action/field.
  Expand diagnostic effect-level logging only when needed to localize a mismatch.
- Layer tests: unit/interaction tests, invariants and randomized supported
  scenarios, snapshot/batch determinism, and differential game traces. All
  required supported-corpus comparisons must pass before promoting a simulator
  build for substantive training; absence of mismatches is not proof of full parity.
- Experimental/unsupported content is not sampled as validated training content.
  Training checkpoints and experiment records include rules/content/schema
  versions, scenario split identity, code revision, and seed configuration.
- No original game binaries/assets/decompiled source are checked into the repo.
  Privileged trace fields are excluded from actor/critic inputs and any teacher.

### Approved section: phase 1 content and completion gates

The user approved this section. Exact encounters, generated-effect closure, test
matrices, scenario splits, and statistical procedures will be expanded in the
written spec without broadening the approved content manifest.

- Phase 1 ruleset: ordinary Ironclad combat at Ascension 0, without Daily
  modifiers. Explicitly reject unsupported character/Ascension/modifier content.
- Deliver 1a first: starter Strike/Defend/Bash, Cultist and Jaw Worm, seeded
  combat/replay, Windows capture, batch stepping, and the first Python learning
  experiment. Burning Blood's combat-end behavior is included.
- Expand 1b/1c before phase 1 exit: all Act 1 normal encounter templates, the
  three elite encounters, and Slime Boss/Guardian/Hexaghost. Include status cards
  and power interactions generated by those enemies, including upgraded Burn.
- Player-card manifest: starter cards plus all 20 Ironclad common cards and 10
  selected uncommon cards, in base/upgraded forms. Common cards: Anger,
  Armaments, Body Slam, Clash, Cleave, Clothesline, Flex, Havoc, Headbutt,
  Heavy Blade, Iron Wave, Perfected Strike, Pommel Strike, Shrug It Off,
  Sword Boomerang, Thunderclap, True Grit, Twin Strike, Warcry, Wild Strike.
  Selected uncommon cards: Battle Trance, Bloodletting, Burning Pact, Disarm,
  Entrench, Feel No Pain, Flame Barrier, Inflame, Metallicize, Seeing Red.
- Relic manifest: Burning Blood, Vajra, Oddly Smooth Stone, Anchor,
  Bag of Preparation, Lantern. Potion manifest: Fire, Block, Strength,
  Dexterity, Energy, Weak, Explosive. No random-card-generating potions yet.
- Content manifests explicitly define generated-card/effect closure; scenario
  validation rejects combinations outside that closure before rollout collection.
  No silently replaced cards, filtered rewards presented as full-game parity,
  or no-op unsupported effects.
- Out of scope: map/events/shops/rewards, later-act enemies, remaining cards and
  relics/potions, other characters, higher Ascensions, and Daily modifiers.
- Phase 2 can begin from this supported subset but must expand coverage before
  claiming unrestricted Act 1 runs; Act 1 can expose rare cards and other content
  excluded from phase 1's manifest.
- Correctness gate: every supported mechanic has focused tests; required
  mechanics/encounter differential fixtures pass; snapshot/batch replay,
  masking, reset/buffer-lifetime, and hidden-information tests pass. Capture
  missing game fixtures as explicit blockers, not silently waived checks.
- Throughput gate: report repeatable simulation-only and end-to-end benchmarks
  across worker/batch configurations and select a measured useful batch setup.
  Do not invent a step-rate guarantee before implementation.
- Learning gate: three independent training runs, a frozen 1,000-scenario
  evaluation corpus spanning supported encounter tiers/decks/resources, and
  identical-scenario random/heuristic baselines. Require a statistically credible
  improvement over random; matching or beating the fixed heuristic is a
  performance target, tracked separately rather than presumed guaranteed.
  Report per-tier results, uncertainty, HP/potion outcomes, timeouts/errors,
  and complete wall-clock budgets. Do not describe pipeline-only completion as
  demonstrated agent strength.

### Approved section: repository layout and build stages

The user approved the repository structure and staged build order. No crates,
bindings, training code, or mod implementation have been scaffolded yet.

- Cargo workspace with four narrowly scoped crates: `spire-core` (owned combat
  state, rules/effect scheduler, RNG, snapshots), `spire-env` (scenario validation,
  observation/action encoding, native environment pool), `spire-python` (thin
  Python extension bindings), `spire-cli` (replay, comparisons, benchmarks).
  `spire-env` depends on core; bindings/CLI depend on env/core as needed. Core
  has no Python/ML runtime dependency.
- Python package under `python/spire_lab/` with environment adapters, model,
  rollout/PPO learner, evaluation, and experiment entry points. Keep the PPO
  loop small and auditable, based on vetted algorithm mechanics rather than
  designing a new RL framework; preserve attribution for adapted code.
- Native extension through PyO3 and mixed-package builds through maturin.
  Native pool work detaches from Python; Rust workers never access Python objects.
  Numeric buffers are exposed through a documented safe ownership interface.
- PyTorch for models/training and NumPy-facing numeric interchange. Select and
  lock compatible package versions during toolchain setup, with an actual
  RTX 5090 CUDA computation/gradient smoke test rather than just GPU enumeration.
- `validation/windows-mod/` contains project-owned game-side mod source and
  reproducible build/install instructions; `validation/fixtures/` contains
  reviewed redistributable traces. Local game dependencies remain external.
- `configs/` contains versioned experiment defaults/splits. `docs/` contains
  specs, manifests, validation provenance, benchmark reports, and roadmap.
  Large runs/checkpoints go in ignored `artifacts/`, with small summary reports
  retained separately. Git lockfiles/config hashes identify reproducible runs.
- Build order: 1a validated starter-combat-to-training slice, 1b normal encounters
  and expanded mechanics/content, 1c elites/bosses and phase 1 completion corpus.
  Each stage includes corresponding Windows fixtures and learning measurements,
  not a months-long content expansion before the first ML experiment.
- Written design covers shared architecture, phase 1 boundaries, and the
  long-term roadmap. The first detailed implementation plan targets 1a only;
  later plans use actual correctness/performance/learning findings from 1a.
- Layout approval permits design documentation, not scaffolding or dependency
  installation; the written-spec and implementation-plan gates still apply.

Current documentation consulted for these proposed integration choices:

- https://github.com/PyO3/pyo3/tree/main/guide/src
- https://github.com/PyO3/maturin/blob/main/guide/src/project_layout.md
- https://github.com/pytorch/pytorch/blob/main/docs/source/notes/cuda.md

## Confirmed roadmap

1. Act 1-ready Ironclad combat for explicitly supported scenarios.
2. Act 1 Ironclad.
3. Full-game Ironclad including keys, Act 4, and Heart victory.
4. Ascension 0–20 full-game Ironclad targeting Heart victory.
5. Repeat the preceding progression for Silent.
6. Remaining characters (Defect and Watcher); their order is not yet selected.
7. Daily run modifiers handled across the preceding capabilities under their
   original mode rules, without inventing Heart access.

Confirmed: Heart victory is the ultimate agent goal. Act 3 completion and Heart
victory are reported separately; combat-only success is an intermediate metric.

These are scope goals, not claims of implemented or validated coverage.
Architecture should accommodate the roadmap without implementing later phases
prematurely. Each phase will need explicit content and fidelity acceptance
criteria; trained-agent capability will be measured separately from simulator
content coverage.

## Written-spec review

Spec: [ML-first design](superpowers/specs/2026-10-11-spire-lab-ml-first-design.md).

The user approved the written spec on 2026-10-11, including exact observation
layouts, owned-copy binding outputs, pilot budgets, scenario splits, and learning
gates. Only the 1a slice receives the first detailed implementation plan.
No execution method has been selected; no application code or product
dependencies have been added.

Self-review checks:

- [x] No placeholders or incomplete sections in the written spec.
- [x] Architecture, content scope, and training/validation contracts agree.
- [x] First implementation scope is explicitly 1a, not the full roadmap.
- [x] Clarified original RNG stream sharing, non-hand choice observations,
      capacity behavior, budget stopping rule, and evaluation-corpus retirement.

## Implementation-plan review

Plan: [Spire Lab 1a](superpowers/plans/2026-10-11-spire-lab-1a.md).

- [x] Write 16 test-driven tasks scoped to the approved first slice.
- [x] Self-review spec coverage, task steps, interface/type consistency, five
      review-focus failure classes, and document proportion.
- [x] Clarify prerequisite type ownership, game-independent mod protocol tests,
      optional native scenario sampling, non-mutating resume observations,
      restore-token expiry, and semantic-versus-capacity schema fingerprints.
- [ ] Obtain user plan review and selection of subagent-driven or native execution.

No implementation tasks have been executed, dependencies installed, or worktree
created. Windows fixture handoffs and per-seed training budgets are explicit
parts of the plan, not assumed available results.

## Real-game validation setup

Confirmed: the user has Slay the Spire installed on Windows and is willing to
install testing mods. The user requires the project's validation tooling to
live in the Spire Lab repository alongside Rust simulation and Python training.

Planned responsibilities, with implementation details still subject to design:

- Game-side test mod source and build/install instructions for Windows.
- Trace capture utilities and a versioned action/state interchange format.
- Rust replay/comparison tooling that reports the first relevant divergence.
- Reviewed regression fixtures with game/mod version and capture provenance.
- Explicit separation of privileged validation state from policy observations;
  debug traces must not accidentally become agent inputs.

The repository should contain the project's own tooling and redistributable
fixtures, not the original game binaries, assets, or decompiled game source.

Proposed setup: use Windows as the real-game reference and the Linux server
for Rust simulator development and Python training. Capture versioned combat
action/state traces on Windows and transfer them to the server as regression
fixtures. A persistent live connection between machines is not required for
the initial validation workflow. Exact capture tools and fixture contents
remain to be designed; do not redistribute game binaries or assets.

## Hardware observations and recommendation

Inspected on 2026-10-09 (an availability snapshot, not a reserved allocation):

- Linux x86_64; AMD Ryzen 9 9950X, 16 physical cores / 32 logical CPUs.
- Process CPU affinity includes all 32 logical CPUs.
- Approximately 59.5 GiB system RAM reported, with approximately 51 GiB
  available at inspection time.
- NVIDIA GeForce RTX 5090; 32,607 MiB total GPU memory reported. GPU
  utilization was 0% and memory use was 524 MiB at inspection time.
- No simulator or training throughput benchmark has been performed, and the
  Python training framework/CUDA compatibility has not been tested.

The user may use this server and asked whether free Colab would be superior.
Default in the approved architecture: use the server as the primary target,
with CPU simulation workers and batched GPU learning/inference. Worker counts,
environment batches, and memory budgets should be configurable and selected by
measurement, leaving resources for other server workloads. Do not add a
distributed training architecture solely because cloud compute is available.

Google's current Colab FAQ states that free GPU access is restricted, GPU types
and limits vary, and free notebook runtimes last at most 12 hours depending on
availability and usage. It is an optional portable execution environment, not
the recommended primary platform for this hardware.

Source: https://research.google.com/colaboratory/faq.html

## Reference projects

- Existing local C++ reference: `../sts_lightspeed_all_characters/`.
- Original: https://github.com/gamerpuppy/sts_lightspeed
- Broad all-character work: https://github.com/SnyderConsulting/sts_lightspeed
- Another all-character fork: https://github.com/ben-w-smith/sts_lightspeed

Reference code and documentation were inspected, but fork test suites were not
run. All-character coverage claims do not establish complete game parity. Any
reused code must retain its applicable license and attribution.
