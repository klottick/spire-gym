# Spire Gym

Spire Gym is a headless Slay the Spire 1 simulator for external machine-learning
and evaluation tools. One Rust engine owns the game rules, with native sessions,
replay and batching, plus thin Python/Gymnasium interfaces. Training loops,
models and experiments live outside this repository. There is no graphical client.

## What works today

Current playable profiles focus on **Ironclad and Silent at Ascension 0**.
They use generated maps with finite captured content: supported cards and
encounters have real mechanics, but reward, shop and event pools are bounded.

| Surface | Implemented coverage | Current limit |
| --- | --- | --- |
| Native Ironclad Runs | Captured Acts 1–3; opt-in Act 4, keys, Shield/Spear and Heart mechanics; all 75 ordinary red card kinds | Normal-start traversal to the Heart and HeartVictory remain unobserved. [Ironclad profile](docs/milestones/m10-heart.md) |
| Native Silent Runs | Canonical starts and finite Acts 1–3 progression; Poison, discard, Shiv, retain and copy mechanics | Finite green pools; native-only delivery; stops before Act 4. Canonical normal completion is unproven. [Silent profile](docs/milestones/m14-silent-act3.md) |
| Python/Gymnasium Ironclad combat | Full-engine standalone supplied-resource fights, native batches and a common combat view inside Runs | Finite A0 inputs and encounters. [Combat API](docs/combat-training.md) |
| Python/Gymnasium Ironclad Runs | Captured single-Run environment, legal-action masks, numeric observations and replay; explicit M10 opt-in | Silent progression is native-only; native acceptance does not establish delivery by an installed Python extension. [Run API](docs/milestones/m10-heart.md#opt-in-usage) |

Silent Act 3 coverage includes actual entry into all three bosses across 42
fixed normal-start cases; every case ends in Death. Six separate supplied-loadout
cases win full-HP bosses and reach the no-key ending. These are distinct from
normal-start completion.

The smaller [combat-v1 API](docs/combat-environments.md) remains available for
supplied first-ready Ironclad states against Cultist or Jaw Worm.

These profiles do not establish complete standard content pools or original-game
runtime/RNG parity. Defect, Watcher, Ascension 1–20 and complete normally reachable
Heart runs remain part of the [product target](docs/scope.md). Detailed supported
content and boundaries are recorded in the [profile contracts](docs/milestones/).

## Try a native rollout

With the [pinned Rust toolchain](rust-toolchain.toml) and cached Cargo dependencies,
run a deterministic captured Ironclad Act 1 example from the repository root:

```sh
cargo run --release --locked --offline -p spire-core --example m5_act1 -- \
  --seed 7 --scenario guardian --budget 4096
```

It prints decisions, resources and the nonterminal `Act2Ready` endpoint. This is
a fixed caller over a finite profile; it does not train a policy.
For current Silent progression, see the [native Silent example](crates/spire-core/examples/m14_silent_act3.rs).

For Python, follow [development setup](docs/development.md#setup-in-a-fresh-checkout)
to build the optional extension, then use the
[combat example](docs/combat-training.md) or [Run example](docs/milestones/m10-heart.md#opt-in-usage).
Python/Gymnasium execution uses the compiled module in that environment; build
matching source before relying on newer native behavior.

## Measured speed

Measured on 2026-10-26 at commit `a8261aa`, release Rust 1.99.0, AMD Ryzen 9
9950X. Medians from seven samples of 100 episodes, after one discarded warmup:

| Workload | Native actions/s | Public-loop actions/s |
| --- | ---: | ---: |
| Authored Jaw Worm combat | 388k | 257k |
| Captured Act 1 → Act2Ready | 173k | 130k |
| Captured Act 2 prefix → floor 26 | 147k | 110k |

These are accepted actions per second on fixed Ironclad A0 scripts, using one
worker and batch size 1. The public loop includes observation/menu work before
each action. Compilation, initialization, policy selection and teardown are
outside runtime timing. See [measurements and reproduction](docs/performance.md)
for hardware, source identity, sample ranges and exact workloads.

## Code and references

[`spire-core`](crates/spire-core) owns game state and rules;
[`spire-env`](crates/spire-env) adds sessions, observations, replay and batching;
[`spire-python`](crates/spire-python) and the
[Gymnasium facades](python/spire_lab/gymnasium) expose that engine to Python.
The separate Python combat implementation is reference/comparison tooling.
The [reference library](data/README.md) contains cited offline records, rather
than a registry of implemented behavior.

Project code is under the [MIT License](LICENSE); reference data has separate
[attribution and licensing terms](data/README.md#licensing). Slay the Spire and
its original assets remain the property of their respective owners.
