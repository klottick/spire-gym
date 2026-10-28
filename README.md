# Spire Gym

Spire Gym is a headless Slay the Spire 1 simulator for external machine-learning
and evaluation tools. One Rust engine owns the game rules, with native sessions,
replay and batching, plus thin Python/Gymnasium interfaces. Training loops,
models and experiments live outside this repository. There is no graphical client.

## What works today

Current playable profiles include **Ironclad and Silent at Ascension 0**, plus
accepted finite native **Defect A0 Act1** progression.
They use generated maps with finite captured content: supported cards and
encounters have real mechanics, but reward, shop and event pools are bounded.

| Surface | Implemented coverage | Current limit |
| --- | --- | --- |
| Native Ironclad Runs | Captured Acts 1–3; opt-in Act 4, keys, Shield/Spear and Heart mechanics; all 75 ordinary red card kinds | Normal-start traversal to the Heart and HeartVictory remain unobserved. [Ironclad profile](docs/milestones/m10-heart.md) |
| Native Silent Runs | Canonical starts, captured Acts 1–4/Heart, all 75 green card kinds and exclusive items; Poison, discard, Shiv, retain, copy and generation mechanics | Finite captured reward/shop/event content; native-only delivery. Canonical normal-start Heart completion remains unproven. [Silent profile](docs/milestones/m15-silent-heart.md) |
| Native Defect Act1 Runs | Canonical short-Neow starts, shared generated route and 12 captured encounters; 28 obtainable card kinds, 14 relics and five potions; healthy nonterminal Act2Ready | Source accepted after independent final READY; M17 publication/main integration pending. Six prescribed normal runs complete, without an all-seed or policy-strength claim; native-only. [Act1 profile](docs/milestones/m17-defect-act1.md) |
| Native Defect authored combat | Finite A0: 29 blue card kinds, four orbs, Focus, slots 0–10, four relics, two potions and persistent card mutations; shared Run and versioned native observation | Authored entry only; merged through [PR55](https://github.com/klottick/spire-gym/pull/55) at `02ea211`. Its fixtures do not establish normal progression or Python delivery. [Defect profile](docs/milestones/m16-defect-combat.md) |
| Python/Gymnasium Ironclad combat | Full-engine standalone supplied-resource fights, native batches and a common combat view inside Runs | Finite A0 inputs and encounters. [Combat API](docs/combat-training.md) |
| Python/Gymnasium Ironclad Runs | Captured single-Run environment, legal-action masks, numeric observations and replay; explicit M10 opt-in | Silent progression is native-only; native acceptance does not establish delivery by an installed Python extension. [Run API](docs/milestones/m10-heart.md#opt-in-usage) |

Silent Act 3 coverage includes actual entry into all three bosses across 42
fixed normal-start cases; every case ends in Death. Six separate supplied-loadout
cases win full-HP bosses and reach the no-key ending. These are distinct from
normal-start completion. M15 separately records six normal-run Deaths and a
supplied Act 4 start reaching HeartVictory in 85 legal actions before the final
review corrections. The corrected implementation passed independent source
review and focused native integration checks; those checks do not replay that win.

M16 Defect entry uses canonical starting resources or explicitly supplied
resources through the shared native Run. M16 merged through PR55 at
`02ea21175aab4962671d4610f8b72d468a2fdad5`. Its authored fixtures bypass normal
progression and remain separate from M17 normal-start evidence. The full blue
pool remains M20. The
[review record](docs/superpowers/plans/2026-10-28-m16-native-runtime-review.md)
separates corrected focused checks from pre-correction workspace/example evidence.
This adds no Defect Gym/Python delivery or original-runtime validation.

M17's independently reviewed native source is accepted for publication
preparation; main integration remains pending. All six prescribed canonical
seed7/11 boss cases enter and defeat the actual requested boss, resolve rewards
and treasure, and reach Act2Ready at 82/82 HP. The same recorded actions agree
through native scalar sessions, portable replay and workers1/2 batches, including
full reset and heterogeneous preflight. The current source-bound workspace result
is 2,448 passed, zero failed and two existing held M15 tests. The
[M17 review record](docs/superpowers/plans/2026-10-28-m17-native-runtime-review.md)
binds these finite outcomes and source relationships. This establishes no
all-seed completion, strong policy, full blue pool, Act2 play, Python delivery or
original-runtime fidelity.

The smaller [combat-v1 API](docs/combat-environments.md) remains available for
supplied first-ready Ironclad states against Cultist or Jaw Worm.

These profiles do not establish complete standard content pools or original-game
runtime/RNG parity. Later Defect acts and full blue pools, Watcher,
Ascension 1–20 and complete normally reachable Heart runs remain part of the
[product target](docs/scope.md). Detailed supported
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
For current Silent progression, see the [native Silent example](crates/spire-core/examples/m15_silent_heart.rs).

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
