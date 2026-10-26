# Spire Gym

Spire Gym is a headless Slay the Spire 1 simulator for external machine-learning
consumers. It uses one Rust simulation engine, native environment APIs, and thin
Python/Gymnasium interfaces. Models, optimizers, training loops, learner
checkpoints, and experiments are outside this repository.

## Start here

| If you are… | Read… |
| --- | --- |
| An agent receiving a task | [Agent entrypoint](AGENTS.md), then the assigned Bead and current profile or plan |
| Checking supported behavior | [Accepted milestone profiles](docs/milestones/) and [environment contracts](docs/combat-environments.md) |
| Choosing the current project scope | [Scope](docs/scope.md) and the dated [work-status snapshot](docs/work-status.md) |
| Building or selecting a test lane | [Development workflow](docs/development.md) and [toolchain](docs/toolchain.md) |
| Researching a game mechanic | [Reference library](data/README.md) and its cited domain guides |

## Implemented surface

The repository contains a bounded supplied-entry Ironclad A0 combat Gym, captured
Ironclad A0 progression/Run profiles, and finite native Silent A0 progression
through Acts1–2 to Act3Ready, including the smaller M11 authored-combat slice.
M10 adds captured Act 4/Heart mechanics and tools, but normal-start traversal
to the Heart and HeartVictory remains unobserved. These are separate versioned
profiles with explicit limits; they do not claim complete standard content, a full
all-character run simulator, original-runtime parity, or a fresh Python artifact
for every accepted native change. Read the relevant profile before relying on a
behavior or result. The M11 authored bridge uses canonical Silent starting
resources but bypasses Neow selection, map generation, and Act 1.

Gym66 adds an opt-in Ironclad A0 combat-training interface over the same Rust
engine: standalone full-engine entries, disabled-autoreset native batches, and a
common public combat view inside normal Runs. The [combat-training guide](docs/combat-training.md)
describes its API, finite inputs, and limits. It uses the opt-in m10-ironclad-policy Run-v3 profile;
the existing M10 Heart profile and default M9 profile retain their frozen
contracts. The guide does not claim complete standard pools, original-runtime
parity, learner training, or a strategy win.

The [final product target](docs/scope.md) remains all four standard characters,
Ascension 0–20, Acts 1–4, and the Heart, with normally reachable content. Current
task ownership and milestone state change over time; consult [Beads and the
current status snapshot](docs/work-status.md), then verify live ownership in the
tracker. Check Beads and the current primary session for the active task and
assigned checkout. This dated snapshot is not an assignment or worktree instruction.

M12 is accepted as a finite native Silent A0 Act1 profile on the same Run,
with normal-start fixed seed 7/11 boss alternatives, rewards, Act2Ready and native
session/replay/batch agreement. The [M12 profile](docs/milestones/m12-silent-act1.md)
records exact scope, outcomes and limits. Independent milestone review and root
verification are complete; no new Python-delivery claim is made.

M13 is accepted as a finite native Silent A0 Acts1–2 profile on the same Run.
It retains 19 permanent green card kinds, 13 relic kinds and four potion kinds,
with generated Wound support. The six fixed runs provide actual entry into all
three Act2 boss species, three Act3Ready completions and three genuine Deaths.
The [M13 profile](docs/milestones/m13-silent-act2.md) records the exact outcomes,
accepted coverage amendment and native session/replay/batch evidence.

M14's [design](docs/superpowers/specs/2026-10-26-m14-silent-act3-design.md) and
[plan](docs/superpowers/plans/2026-10-26-m14-silent-act3.md) are approved. Its
implementation uses the accepted M13 source through the primary session's
assigned checkout and live Beads ownership.

## Architecture and references

crates/spire-core owns game state and rules. crates/spire-env owns native
environment sessions, observations, actions, replay, and batching. Python bindings
and Gymnasium wrappers stay thin and reuse that engine. The Python combat code is
reference/comparison tooling, not a second production simulator. The
[reference library](data/README.md) contains cited offline records; it is not an
implemented-content registry or runtime evidence.

See [performance measurements](docs/performance.md) for opt-in benchmark guidance.
The project code is under the [MIT License](LICENSE); reference data has separate
[attribution and licensing terms](data/README.md#licensing). Slay the Spire and
its original assets remain the property of their respective owners.
