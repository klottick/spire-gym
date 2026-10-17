# Spire Gym

A headless Slay the Spire 1 simulation and machine-learning project. The end goal
is an AI that plays StS and improves; the enabling product is an optimized,
game-correct Rust simulator with native batching and Python learning code.

Repository: [klottick/spire-gym](https://github.com/klottick/spire-gym). Gymnasium
compatibility is the approved next interface direction, not yet implemented.
Existing `spire-lab` / `spire_lab` package and crate names remain unchanged.
Moving ML training and experiments into a separate repository is deferred; the
simulator, environment interfaces, correctness tests and examples belong here.

The [repository scope](docs/scope.md) records the current priorities, boundaries,
delivery path and implementation status. No human-playable client, graphics,
animation, audio or interactive game UI is planned.

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
final correction review and fresh controller verification are complete.
Native batching and the learning system are not implemented.
No learned playing-strength or optimized simulator-throughput result
is claimed. The Python reference and validation tools support those products;
they are not the final production simulator or a separate infrastructure goal.

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
publication does not promote original-runtime fidelity or implement Gymnasium support.
The current selected workflow is
Sol implementation with fresh independent reviews, not the historical worker
selection in the [original 1a plan](docs/superpowers/plans/2026-10-11-spire-lab-1a.md).

## Confirmed goals

- Prioritize efficient ML training: learning progress on held-out scenarios per
  unit of wall-clock time, backed by game-correct simulation.
- Build and profile the game-correct simulator in Rust; learning Rust remains a
  secondary benefit. Native batching and thin bindings serve the training path.
- Use Python for model training, baselines, evaluation and experiments; the readable
  Python combat reference is an intermediate correctness aid, not a performance target.
- Develop a simulation tool for learning to play Slay the Spire.
- Make Heart victory the ultimate agent goal; report Act 3 completion separately.
- Plan the design and implementation before building the application.

## Confirmed roadmap

1. Act 1-ready Ironclad combat simulation and training (the starting milestone).
2. Act 1 Ironclad runs.
3. Full-game Ironclad runs including keys, Act 4, and the Heart.
4. Full-game Ironclad across Ascension 0–20, targeting Heart victory.
5. Repeat the preceding progression for Silent.
6. Extend the same capabilities to the remaining characters, Defect and Watcher.
7. Support Daily run modifiers under their original mode rules, without
   requiring Heart access where the mode does not permit it.

Detailed content coverage and acceptance criteria are scoped per phase.
The initial slice remains 1a (starter cards versus Cultist/Jaw Worm), not the
entire roadmap: reviewed mechanics → independent Rust slice → batching/bindings and release
benchmarks → a measured learner-versus-baseline comparison → expanded content.
Phase 1 will support explicitly scoped Act 1 combat scenarios; complete
Ironclad combat through the Heart is not a prerequisite for phase 2.

## Approved architectural direction

- Rust-native game-specific core with a batched numeric interface to Python.
- [Separate combat and run-level policies initially](docs/scope.md#approved-two-level-learning-direction-2026-10-17),
  backed by one authoritative Rust simulator. Run-level training delegates fights
  to the same detailed combat engine, not an approximate second simulator.
- Python handles ML training/evaluation; its combat reference is only a correctness
  aid. Shared model representations or a unified policy remain later experiments.
- Train and evaluate agents using player-visible information, not hidden RNG
  state or unseen future outcomes.
- Start with a small, validated end-to-end training slice, then expand content.
- Use existing simulators as references, not unquestioned correctness oracles.
- Keep game fidelity, deterministic replay, and measurable training throughput
  central to the design.

The written design, including concrete contracts, pilot defaults, and acceptance
procedures, is approved. Later narrow specifications and reviewed plans govern
their own execution; scope approval is not blanket implementation authorization.

See [the design checklist](docs/design-process.md) for progress and decisions.

## License

Project code is licensed under the [MIT License](LICENSE). Slay the Spire is owned
by Mega Crit; this license does not grant rights to the original game or its assets.
Original game code, binaries, assets and private raw evidence are not distributed
with this project.
