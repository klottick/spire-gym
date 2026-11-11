# Spire Gym

Spire Gym is a headless Slay the Spire 1 simulator for external machine-learning
and evaluation tools. One Rust engine owns game state and rules; native sessions,
observations, replay and batching are exposed through thin Python/Gymnasium
facades. Models, training loops and experiments live outside this repository.
There is no graphical client.

## Delivered scope

The Standard profile covers Ironclad, Silent, Defect and Watcher at Ascension
0–20, Acts 1–4 and the Heart, using finite Standard content. The repository also
retains legacy Run and combat training surfaces. Source grounding and reference
data are distinct: data files are cited offline records, not a runtime registry
or proof that a behavior is implemented. The simulator has not been validated
against the original game's runtime. Training and strategy evaluation belong to
consumers.

See [delivery and claim grades](docs/delivery.md) for acceptance evidence,
limits, intentional deviations, the contract freeze and consumer guidance. The
previous README body remains verbatim in the
[history archive](docs/handoffs/2026-11-11-readme-history.md).

## Quick start

With the pinned Rust toolchain and locked dependencies already available, build
the workspace and run the native example:

```sh
cargo build --workspace --locked --offline
cargo run --release --locked --offline -p spire-env --example standard_rollout -- ironclad 0 7
```

With an already provisioned Gym environment and matching imported extension,
run the Python example:

```sh
uv run --locked --no-sync python examples/standard_run.py --character ironclad --ascension 0 --seed 7
```

The setup and extension identity requirements are in
[development](docs/development.md). The examples take bounded or random legal
actions; they are demonstrations, not strategy claims.

## Examples

Run commands from the repository root. Rust examples build through Cargo;
Python examples require the provisioned Gym environment described above.

| Surface | Policy kind | Command | What it prints |
| --- | --- | --- | --- |
| Native Standard rollout | Random by default; `--policy first_legal` selects the first legal action | `cargo run --release --locked --offline -p spire-env --example standard_rollout -- --policy random` / `cargo run --release --locked --offline -p spire-env --example standard_rollout -- --policy first_legal` | Character, ascension, seed, terminal/step summary, return, and replay/fork agreement |
| Native A20 checks | Seeded uniform legal-action diagnostic | `cargo run --release --locked --offline -p spire-env --example a20_checks -- --smoke` | JSON identity, single/batch/memory phases, and capacity occupancy |
| Python Standard Run | Random legal | `uv run --locked --no-sync python examples/standard_run.py` | Rollout terminal, reward and replay/fork comparison |
| Python Run vector | Random legal by default; `--policy first_legal` available | `uv run --locked --no-sync python examples/run_vector_rollout.py --steps 8` | Per-slot episode state and throughput summary |
| Python Standard combat vector | Random legal by default; `--policy first_legal` available | `uv run --locked --no-sync python examples/standard_combat_vector.py --steps 8` | Per-slot combat terminal/reward summary |
| Python public lookahead | Sampled public belief worlds with random-legal continuation | `uv run --locked --no-sync python examples/public_lookahead.py` | Mean sampled outcomes per legal combat action; baseline only |
| Python tactical lessons | Synthetic/reachable setup; first-legal and seeded-random paired baselines | `uv run --locked --no-sync python examples/tactical_lessons.py` | Setup labels, paired outcome summaries and digest |
| Legacy `CombatVectorEnv` | Random legal, finite collection | `uv run --locked --no-sync python examples/combat_gymnasium.py` | Natural terminal summaries for completed slots; no environment time cap |
| Legacy Ironclad combat training | First legal in the demonstration | `uv run --locked --no-sync python examples/ironclad_combat_training.py` | Supplied combat, vector, and normal-Run smoke summaries |
| Legacy M9 Run | Masked random; `--nonattacking` prefers end turn | `uv run --locked --no-sync python examples/run_m9_gymnasium.py` | M9 Run episode and replay summary |
| Legacy M10 Run | Masked random; `--nonattacking` prefers end turn | `uv run --locked --no-sync python examples/run_m10_gymnasium.py` | M10 Run episode and replay summary |

The retired `m22_same_choices`, `m23_same_choices` and `m24_same_choices` Rust
examples are not available. The M21 example remains historical support.

## Contracts and limits

- [Standard Run API](docs/standard-run-api.md) defines Standard creation,
  observations, actions, replay and the M36 supplied Heart acceptance.
- [Batching](docs/batching.md) defines batch/vector behavior and public search
  worker limits.
- [Combat training](docs/combat-training.md) documents the legacy Gym66
  interfaces and their boundaries.
- [Milestone index](docs/milestones/README.md) links the finite profiles;
  [M36 final delivery](docs/milestones/m36-final-delivery.md) records the final
  profile and evidence reconciliation.

Standard source coverage does not establish normal policy reach or strategy
strength. The M36 Heart routes are supplied, not policy wins: a test-only hook
hits every enemy down to 1 HP at each combat decision, so they exercise
progression and settlement, not combat difficulty. RNG/seed-stream parity,
absolute original-game floor numbering and original-runtime fidelity are not
claimed. See [delivery limits](docs/delivery.md#known-limits-by-surface) and
[intentional deviations](docs/delivery.md#intentional-deviations).

## Licensing, attribution and trademarks

Project code is under the [MIT License](LICENSE); reference data has separate
[attribution and licensing terms](data/README.md#licensing). Slay the Spire and its original
assets remain the property of their respective owners.
