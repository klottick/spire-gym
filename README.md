# Spire Gym

A headless Slay the Spire 1 gym for external machine-learning consumers: a Rust
simulator, native batching, thin Python bindings and Gymnasium interfaces.
**Models, optimizers, training loops, learner checkpoints and ML experiments are
outside this repository's scope.**

Repository: [klottick/spire-gym](https://github.com/klottick/spire-gym). A bounded
native batched combat environment and single/vector Gymnasium facades are implemented
for supplied-entry Ironclad A0 encounters; this is not a full-run simulator.
Existing `spire-lab` / `spire_lab` package and crate names remain unchanged.
The [accepted M9 profile](docs/milestones/m9-act3.md) adds captured Ironclad A0
progression through Act3 and a thin single-Run `RunEnv`, after one independent review.
All six frozen normal-start seed7/11 traces are genuine losses, not demonstrated
boss50/no-key wins. Strategy development belongs to external callers.

**Current compatibility notice — 2026-10-25:** corrected shared random-spot draw
insertion changes current seeded outcomes; see the [M9 compatibility note](docs/milestones/m9-act3.md#current-rules-compatibility-2026-10-24).
Native M10 implementation and the preservation matrix are controller-verified on
the final 607-file source baseline. The fresh Python environment passed 251 tests
(235 prior plus 16 Task 28); touched-file Ruff, format, and `ty` checks passed. The
native matrix recorded 1,402 workspace passes with exactly the seven preserved
historical failures; the M10–M6 filters and 17 example/CLI invocations passed their
expected results. The [accepted M10 profile](docs/milestones/m10-heart.md) records
scoped acceptance after ONE clean independent review and verification of all 608
reviewed source files in both checkouts. Six frozen seed-7/11 normal-start traces
remain early gameplay Deaths; separate default masked-caller Deaths at 27/67 actions
are not strategy-quality or full-route evidence.
The simulator, environment interfaces, correctness tests, performance checks and
small rollout examples belong here; learner/training infrastructure does not.

The [repository scope](docs/scope.md) records the current priorities, boundaries,
delivery path and implementation status. No human-playable client, graphics,
animation, audio or interactive game UI is planned.

See [performance measurements](docs/performance.md) for opt-in release benchmarks,
versioned baseline comparisons and planned coverage for later gym milestones.

## Game-rule resources for contributors and AI agents

Start with the **[reference library index](data/README.md)**:
[cards](data/cards/README.md), [relics](data/relics/README.md),
[enemies and encounter rules](data/enemies/README.md), and
[powers / buffs / debuffs](data/powers/README.md). Read each guide's citations,
uncertainties and source grades before implementing mechanics or repeating research.
These are offline references, **not implemented-content lists or runtime proof**.
See [agent guidance](AGENTS.md) for current ownership and task constraints;
the index lists standard-library offline checks that need no pytest or installs.

## Authored combat: M1–M3

The approved [36-milestone roadmap](docs/superpowers/specs/2026-10-22-narrow-gym-milestones-design.md)
starts with **authored Ironclad A0 combat: damage, Block, Weak and Vulnerable**.
The user approved native execution of the [migration/M1 plan](docs/superpowers/plans/2026-10-22-roadmap-migration-and-m1-combat.md).
Reuse the existing simulator rather than restarting it. M1 is not a normal run;
later milestones incrementally connect fights, rooms, acts and characters.

M1 is accepted after one isolated independent review. The
[M2 implementation plan](docs/superpowers/plans/2026-10-23-m2-shared-combat-mechanics.md)
is user-approved. The [accepted M2 profile](docs/milestones/m2-combat.md) adds shared
stats/Artifact/Frail, HP loss, public potion/selection actions and real Ironclad
producers after integrated verification and one isolated review with no findings.
Seven unchanged draft failures remain; this is not a green-workspace/full-run claim.

Run the [M1 combat demonstration](docs/milestones/m1-combat.md):

```sh
cargo run -p spire-core --example m1_combat --locked --offline -- --scenario all --seed 7
cargo run -p spire-core --example m2_combat --locked --offline -- --scenario all --seed 7
```

The final target remains **all four fully unlocked characters, Ascension 0–20,
Acts 1–4 and the Heart, with all normally reachable content**; A20 is the primary
final performance workload. Beads `spire-dxo` remains the parent. Milestones
M1–36 are `.23`–`.58`. M1 `.23`/`.23.1` and M2 `.24`/`.24.1`/`.24.2` are closed;
M3 `.25`, M4 `.26`, M5 `.27`, M6 `.28` and M7 `.29` are accepted at their
declared profiles; M8 `.30`/`.30.2` are also accepted/closed at the captured
Act3 mechanics/prefix scope. M9 and M10 are accepted at their declared captured
profiles after independent review and exact-source verification. M10 is merged in
[PR46](https://github.com/klottick/spire-gym/pull/46). The user-approved M11 plan is [documented here](docs/superpowers/plans/2026-10-25-m11-silent-combat-mechanics.md). M11 `.33.2`/`.33` is accepted at its finite Silent A0 scope after one fresh independent review (`m11_milestone_review`; evidence: `.superpowers/sdd/2026-10-25-m11-silent-combat-mechanics/task8-independent-review.md`). The [accepted M11 profile](docs/milestones/m11-silent-combat.md) records representative Poison, discard, Shiv, retain, copy, and canonical-resource authored-combat behavior. The final workspace preservation matrix recorded 1,476 passed and one retained Blue Slaver and Two Fungi failure. The authored bridge bypasses Neow selection, map generation, and Act 1; it is not a Silent Gym profile or normal-start route. No Python extension refresh or Python-delivery claim was made. The user-prioritized Gym interoperability request `.66` is being implemented in parallel in an isolated worktree; M11 constrains its final integration only. M12 remains planning-only and is not the automatic next product milestone. Old unfinished components retain their scope and history as coverage umbrellas, not blockers requiring every content family before the first usable milestone.
Use fast deterministic behavior tests and focused regressions, not a custom
verification platform. Complete standard pools and the wider four-character roadmap remain outside the implemented profiles.

The [M3 plan](docs/superpowers/plans/2026-10-23-m3-persistent-combat-run.md) is
user-approved. Its [accepted persistent combat profile](docs/milestones/m3-run.md) runs
two real fights with resource/identity preservation and one-shot settlement.
One isolated milestone review found no issues; core349/7 retains exactly seven
old draft failures. This is not normal-start or full-run acceptance.

```sh
cargo run -p spire-core --example m3_run --locked --offline -- --scenario all --seed 7
```

The user-approved [M4 room-loop plan](docs/superpowers/plans/2026-10-23-m4-act1-room-loop.md)
now has an [accepted runnable profile](docs/milestones/m4-rooms.md): real normal-start
resources, Neow and room consequences on an explicitly authored nine-floor route
with captured offers—not smaller standard random pools or Act1/boss completion.
One isolated GPT-6 Luna review found no issues; seven unchanged draft failures
remain. M5 implementation is described below.

```sh
cargo run -p spire-core --example m4_rooms --locked --offline -- --scenario all --seed 7
```

The [M5 implementation plan](docs/superpowers/plans/2026-10-23-m5-ironclad-act1.md)
is user-approved. Its [accepted runnable profile](docs/milestones/m5-act1.md) completes
real public standard-resource paths through the entire generated map, all three
elite/boss alternatives, captured rewards and same-player nonterminal Act2Ready.
One isolated GPT-6 Luna milestone review found no issues. Standard pools/full Neow/Act2 gameplay
remain incomplete; seven unchanged draft failures remain, not workspace green.

```sh
cargo run -p spire-core --example m5_act1 --locked --offline -- --scenario all --seed 7
```

The [M6 plan](docs/superpowers/plans/2026-10-23-m6-act2-mechanics.md) is approved
and its [runnable profile](docs/milestones/m6-act2.md) now executes real Act2
mechanics/events and same-player public paths through floor26. One isolated Luna
review found no issues at this accepted scope. Full Act2 progression remains M7, and M5 still
stops at its unchanged no-action Act2Ready.

```sh
cargo run -p spire-core --example m6_act2 --locked --offline -- --scenario all --seed 7
```

The approved [M7 plan](docs/superpowers/plans/2026-10-23-m7-ironclad-act2.md)
has an [accepted captured Act2 profile](docs/milestones/m7-act2.md) after one
independent milestone review: actual normal-start boss/reward/chest alternatives
and same-player Act3Ready, not Act3 gameplay. Genuine Death is a valid episode
outcome, not a simulator error or claimed act completion. The preselected seed11
completes all three alternatives; seed7 also records a natural Champ loss.

```sh
cargo run -p spire-core --example m7_act2 --locked --offline -- --scenario all --seed 7
cargo run -p spire-core --example m7_act2 --locked --offline -- --scenario complete --seed 11
```

The [written M8 plan](docs/superpowers/plans/2026-10-23-m8-act3-mechanics.md)
is complete and its [accepted runnable profile](docs/milestones/m8-act3.md)
executes actual Act3 ordinary/elite/all3boss mechanics, Falling/Tomb/Red Mask and
normal-start same-player paths through actual chest43. Seed11 completes Time Eater/
Donu-Deca icon prefixes and retains a genuine Giant Head loss; all three seed7
prefixes retain natural elite losses. No seed search or forced-win tuning. ONE fresh
independent review found no findings; unchanged reviewed source and fresh checks
establish scoped M8 acceptance. Full Act3 completion stays M9, now planning only.

```sh
cargo run -p spire-core --example m8_act3 --locked --offline -- --scenario all --seed 11
cargo run -p spire-core --example m8_act3 --locked --offline -- --scenario prefix --seed 7
```

See [repository scope](docs/scope.md), [existing environment usage](docs/combat-environments.md)
and [the work-status snapshot](docs/work-status.md). Historical specs/plans retain
their evidence and approval limits; they do not override the current repo boundary.
The earlier [full-run design](docs/superpowers/specs/2026-10-22-full-run-gym-poc-design.md)
and [component plan](docs/superpowers/plans/2026-10-22-full-run-gym-poc.md) retain
their implementation history; the progressive roadmap controls current scheduling.
Portal's time condition is assumed satisfied under TIME-001; no clock or wait action.

## Status

The native-package/readiness foundation and the narrow Python combat reference
are implemented. The reference supports supplied-entry Ironclad A0 starter-card
combats against one Cultist or Jaw Worm. Its final recorded implementation run
passed 735 Python tests; it is **not original-runtime validated**.

The narrow Rust combat implementation is present, with standalone whole-combat
acceptance and a test-only Python differential harness. It is **source-grounded,
not original-runtime validated**. After the independently reviewed native provenance
correction and arbitrary-size-handle adapter fix, Task-8 verification passes:
95 differential tests, 830 full Python tests,
and, after the final minor-fix wave, 124 native tests in each workspace/release run.
Historical failed comparisons remain retained. Task reviews, whole-slice review,
final correction review and fresh controller verification are complete for that
historical core-only handoff; they do not describe verification of the later
batching/Gymnasium integration.
Full-run simulation is not implemented; bounded M3–M7 Run profiles above are native.
The learning system is external, not
an unfinished deliverable of this repository.
No learned playing-strength or optimized simulator-throughput result
is claimed. The Python reference and validation tools support those products;
they are not the final production simulator or a separate infrastructure goal.

The bounded combat/Gymnasium slice adds native scenarios, public numeric
observations, masked actions, replay and single/vector facades. A fresh offline
Rust workspace run passed 169 tests. Controller verification with a freshly built
candidate-revision extension passed all 167 environment Python tests; the full
Python suite had 995 passes and the same two baseline readiness-test failures as
main. Those CPU-only test failures are distinct from historical required-CUDA
checks that failed with CUDA allocation OOMs. No fully green suite, CUDA readiness,
or original-runtime validation is claimed for this slice.

The [ML-first design](docs/superpowers/specs/2026-10-11-spire-lab-ml-first-design.md)
and [combat behavior plan](docs/superpowers/plans/2026-10-16-python-combat-behavior.md)
record the architecture and completed reference milestone. The revised
[Linux validation-readiness specification](docs/superpowers/specs/2026-10-16-linux-validation-readiness-design.md)
is retained but capture work is deferred. The user approved
[source-first sequencing](docs/scope.md#approved-source-first-sequencing-amendment-2026-10-16):
reviewed mechanics → independent Rust combat slice → batching/bindings and
benchmarks, without waiting for original captures. Original comparisons remain a
later fidelity-promotion gate; no original-runtime-validated or transfer claim is
made. The [Rust combat specification](docs/superpowers/specs/2026-10-16-source-first-rust-combat-design.md)
and its separately approved
[implementation plan](docs/superpowers/plans/2026-10-16-source-first-rust-combat.md)
govern the current implementation, including the named native HMAC dependency.
User plan/start approval was executed and the core-only handoff is verified.
The core-only handoff preceded the separately authorized GitHub publication;
neither that publication nor this bounded environment slice promotes
original-runtime fidelity.
Historical worker/model selections are not live assignments. Beads and the
[agent guidance](AGENTS.md) govern current work; paused subagents are not resumed
automatically by an old plan or an open research issue.

## Confirmed goals

- Provide a usable, deterministic, game-correct gym for external ML consumers.
- Build and profile the game-correct simulator in Rust; learning Rust remains a
  secondary benefit. Measure representative environment throughput and resource use.
- Use Python for thin interfaces, tests and rollout examples; the readable Python
  combat reference is an offline correctness aid, not a second production engine.
- Develop a simulation tool for learning to play Slay the Spire.
- Make Heart victory the ultimate agent goal; report Act 3 completion separately.
- Plan the design and implementation before building the application.

## Historical long-term roadmap

1. Explicitly scoped Ironclad combat environments (the implemented starting slice).
2. Act 1 Ironclad runs.
3. Full-game Ironclad runs including keys, Act 4, and the Heart.
4. Full-game Ironclad across Ascension 0–20, targeting Heart victory.
5. Repeat the preceding progression for Silent.
6. Extend the same capabilities to the remaining characters, Defect and Watcher.
7. Support Daily run modifiers under their original mode rules, without
   requiring Heart access where the mode does not permit it.

This older progression is retained as history; the approved36-milestone roadmap
above governs selected delivery. Daily/custom modes are outside that selected
standard-run scope. The legacy environment remains the supplied-entry starter
slice; M1's broader authored combat profile is documented separately.

## Approved architectural direction

- Rust-native game-specific core with a batched numeric interface to Python.
- One authoritative Rust simulator for combat and future run-level transitions,
  not an approximate second simulator for between-combat decisions.
- Python provides interfaces; policy/model design and training are external. The
  [historical two-level policy direction](docs/scope.md#approved-two-level-learning-direction-2026-10-17)
  is consumer context, not in-repo ML implementation work.
- Expose player-visible information, not hidden RNG state or unseen future outcomes.
- Deliver a runnable gym vertical slice, then expand supported content.
- Use existing simulators as references, not unquestioned correctness oracles.
- Keep game fidelity, deterministic replay, and measurable training throughput
  central to the design.

Historical written designs govern their implemented slices. New narrow designs
and plans govern their own execution; scope approval and this administrative
cleanup are not blanket product-implementation authorization.

See [the design checklist](docs/design-process.md) for progress and decisions.

## License

Project code is licensed under the [MIT License](LICENSE). Wiki-derived reference
data has separate [CC BY-SA attribution and licensing](data/README.md#licensing);
the domain guides retain their specific notices. Slay the Spire is owned
by Mega Crit; this license does not grant rights to the original game or its assets.
Original game code, binaries, assets and private raw evidence are not distributed
with this project.

The opt-in [Ironclad combat training interface](docs/combat-training.md) documents
standalone full-engine entries, disabled-autoreset native batches and the common
combat-policy view for normal Runs. Existing combat-v1/M9/M10 defaults stay frozen.
