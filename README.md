# Spire Gym

A headless, deterministic Slay the Spire (StS1) simulator built as a training and
evaluation environment for external machine-learning tools. One Rust engine owns all
game state and rules. Thin Python/Gymnasium facades expose it. There is no graphical
client, and no models or training code live here.

**Current release: [v1.0.0](https://github.com/klottick/spire-gym/releases/tag/v1.0.0).**
The roadmap is complete, and the public contracts are under a
[versioned freeze](docs/delivery.md#freeze-and-change-protocol).

## Intentions

**What this project is for**

- **Correct gameplay.** A full run should play by the original game's rules. Behaviour
  comes from static reading of the decompiled game and is checked by Rust tests.
- **One source of truth.** A single Rust engine owns everything. Python never
  reimplements game logic.
- **Reproducibility.** Seeded resets, exact replay of recorded runs, deterministic
  batches, and identity checks that tie a built extension to its source.
- **Honest information.** Observations show only what a player could know. Hidden state
  (draw order, RNG, unrevealed cards) stays private unless you call the explicitly
  privileged `exact_*` tools.
- **Throughput for learning.** Native batching with worker pools, Gymnasium vector
  environments, and public-information search for planning agents.

**What it deliberately does not do**

- **No strategy, policy or win-rate work.** Agents and training live in consumer
  repositories. A random policy usually dies in Act 1, and that is expected.
- **No original-game runtime validation.** The original game was never run, and RNG
  streams are project-owned rather than the game's own (see
  [intentional deviations](docs/delivery.md#intentional-deviations)).
- **No other game modes or interface.** Daily, custom and modded modes, the graphical
  client and career progression are out of scope.

## Coverage

| Area | Covered |
| --- | --- |
| Characters | Ironclad, Silent, Defect and Watcher, fully unlocked |
| Ascension | A0–A20, every ascension rule (including the A20 double boss) |
| Acts | Acts 1–3, then Act 4 and the Heart via the three keys |
| Cards | Every standard card in base and upgraded form (369 of 370 reference rows; Pride is Endless-only): all four character pools, plus colourless, curse and status cards and off-colour generation |
| Relics and potions | All 179 relics and 42 potions, with their full lifecycles |
| Events and rooms | Neow, all 52 events, shops, rest sites, treasure, rewards and the economy |
| Enemies | Every Act 1–4 normal, elite and boss encounter, including the Heart, with A0–A20 behaviour |
| Run API | Seeded reset, public observations and action masks, reward events, terminal reasons, records, replay to any decision, and exact checkpoints/forks (privileged) |
| Search and scenarios | Draw-pile knowledge limited to what the player can know, belief-sampled branching (`PublicSearch`), and validated tactical start positions |
| Batching and Python | Native `SessionBatch` with worker pools, partial reset, autoreset and opt-in fault isolation. Gymnasium: `RunEnv`, `RunVectorEnv`, `StandardCombatEnv`, `StandardCombatVectorEnv`, plus the older `CombatEnv`, `CombatVectorEnv`, `IroncladCombatEnv` and `IroncladCombatVectorEnv` |

**What the evidence shows.** Each claim is graded in
[delivery](docs/delivery.md#claim-grades). In summary:

- **Source-grounded rules.** Behaviour was derived from the decompiled source and is
  covered by about 5,100 Rust tests, fuzzing and full A0–A20 traversals.
- **From-start Heart acceptance.** Each character at A0 and A20 is driven from a public
  seed to Heart victory on a *supplied* route: 900 HP, all three keys, and a test-only
  hook that leaves every enemy at 1 HP. This exercises progression, routing, events,
  rewards and the endings, not combat difficulty, and it is not a policy win.
- **Not claimed:** original-runtime fidelity, original RNG and seed parity, and
  normal-play Heart reach.

## Quick start

You need the pinned Rust toolchain (see `rust-toolchain.toml`). The Python side needs
Python 3.12; setup is in [development](docs/development.md).

```sh
cargo build --workspace --locked --offline
cargo run --release --locked --offline -p spire-env --example standard_rollout -- ironclad 0 7
```

```python
from spire_lab.gymnasium import RunEnv

env = RunEnv("ironclad", 20, profile="standard")
obs, info = env.reset(seed=7)
```

A prebuilt Linux wheel for CPython 3.12 is attached to the
[v1.0.0 release](https://github.com/klottick/spire-gym/releases/tag/v1.0.0). Check `spire_lab._native.build_info()`
against the release's published `source_digest`.

## Examples

| Example | Command |
| --- | --- |
| Native Standard rollout (random or first-legal) | `cargo run --release --locked --offline -p spire-env --example standard_rollout -- --policy random` |
| A20 throughput, memory and capacity diagnostics | `cargo run --release --locked --offline -p spire-env --example a20_checks -- --smoke` |
| Python Standard run, with replay and fork check | `uv run --locked --no-sync python examples/standard_run.py` |
| Python run and combat vector environments | `uv run --locked --no-sync python examples/run_vector_rollout.py --steps 8` and `examples/standard_combat_vector.py` |
| Public-information lookahead | `uv run --locked --no-sync python examples/public_lookahead.py` |
| Tactical lessons (validated start positions) | `uv run --locked --no-sync python examples/tactical_lessons.py` |

The [examples](examples/) directory also has demos for the older Gym66, M9 and M10
surfaces. All examples use random or trivial policies; they are demonstrations, not
strategy.

## Speed

Measured on 2026-11-12 on an AMD Ryzen 9 9950X (16 cores, Linux, release build), with the
M37b and M38b speedups applied. Workload: Standard A20, all four characters, random legal
actions. The v1.0.0 figures from the same host are shown for comparison. The last column uses
the opt-in compact observation from M38e (`observation="compact"`), with the same information
in 137 KB per slot instead of 764 KB.

| Path | `record_replay` | v1.0.0 | After M38b | M38e compact |
| --- | --- | --- | --- | --- |
| Native scalar (`a20_checks`), actions/s, observation read included | off | about 11.0k | about 22k | about 23k |
| Native batch (`a20_checks`), 64 slots, 1 / 4 / 16 workers, slot steps/s | on | about 5.2k / 11.1k / 12.1k | about 13.4k / 32.6k / 41.3k | about 17.6k / 54.5k / 92.5k |
| Native batch (profiling harness, a test build about 8% slower), 64 slots, 1 / 4 / 16 workers, slot steps/s | off | about 6.3k / 14.8k / 19.6k | about 13.7k / 34.8k / 54.0k | about 17.8k / 59.9k / 138k |
| Python `RunVectorEnv` loop with a per-slot Python policy, 64 slots, 1 / 16 workers, slot steps/s | off | about 4.6k / 8.6k | about 12.7k / 38.0k | about 19k / 104k |
| Python `NativeRunBatch.step` alone (no Gymnasium facade), 64 slots, 1 / 16 workers, ms per call | off | about 13.6 / 6.6 | about 4.6 / 1.3 | about 3.3 / 0.58 |

See [performance](docs/performance.md#a20-standard-checks-after-m38e--2026-11-12) for ranges and
the short speed check used in build records.

**Where the time goes now.**
- Rules work and encoding the 764 KB dense observation take most of a step. The opt-in
  compact observation writes 137 KB; it fails closed with a typed fault if a state exceeds its
  measured capacities ([Standard Run API](docs/standard-run-api.md#compact-observation-m38e)).
- A Standard batch step encodes each observation straight into the arrays it returns to
  Python, on the batch's workers, and reuses the memory of arrays Python has released.
- Native batch callers that keep the previous call's rows until the next step hold about
  twice the observation memory; dropping them first is faster
  ([batching](docs/batching.md#slots-seeds-and-workers)).

These figures are diagnostic, not targets or guarantees. Results vary by host and run.
For the method and the remaining options, see [profiling](docs/profiling.md) and
[performance](docs/performance.md).

## Stability

v1.0.0 freezes, by version:
- the Standard family's identities;
- card metadata;
- `build_info()`;
- the batching contract and the Python API names;
- the legacy surfaces current consumers use.

Gameplay bug fixes still land. Each one bumps the affected rules id and is logged in
[CHANGES.md](CHANGES.md). Records from an older rules id fail replay with an
explicit error rather than diverging silently.

## Documentation

- [Delivery](docs/delivery.md): scope, evidence, claim grades, limits, deviations, the freeze protocol.
- [Standard Run API](docs/standard-run-api.md): observations, actions, records, replay, search and tactical starts.
- [Batching](docs/batching.md): batch and vector behaviour, faults and workers.
- [Performance](docs/performance.md): A20 diagnostic measurements.
- [Profiling](docs/profiling.md): where the time goes, and the proposed fixes.
- [Development](docs/development.md): toolchain, build, test lanes and extension identity.
- [Milestones](docs/milestones/README.md): the M1–M36 history and per-milestone evidence.
- [Release notes](docs/releases/v1.0.0.md) and the earlier [README](docs/handoffs/2026-11-11-readme-history.md).

## Licensing, attribution and trademarks

Project code is under the [MIT License](LICENSE). Reference data has separate
[attribution and licensing terms](data/README.md#licensing). Slay the Spire and its original
assets remain the property of their respective owners.
