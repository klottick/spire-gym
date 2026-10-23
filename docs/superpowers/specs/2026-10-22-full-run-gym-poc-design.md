# Full-run Spire Gym PoC

**Status:** written design approved by the user on 2026-10-22 ("the spec looks
good"). Beads: `spire-dxo` / `spire-dxo.1`. The user also approved the detailed
plan (“lgtm”) and authorized start (“You can start”); `.3` is closed. Execution
workspace setup precedes product edits; no product code is started yet.
This supersedes Act 1-only endpoints and in-repo ML requirements, not accepted
mechanics, evidence qualifications or existing public combat contracts.

**Planning clarification:** the [implementation-plan draft](../plans/2026-10-22-full-run-gym-poc.md#timing-correction-and-portal-handling)
withdraws an unsupported Reaper/frame-simulation rationale after user correction.
The user selected [TIME-001](../../intentional-deviations.md#time-001--secret-portals-elapsed-time-requirement-is-assumed-satisfied):
assume Portal's 800-second condition is always satisfied, preserving other
eligibility and normal random selection. No clock, frame or wait subsystem is
required. Plan/start approval is recorded; workspace setup remains.

## 1. Outcome and scope

Deliver a headless gym for **Ironclad, Silent, Defect and Watcher, fully unlocked,
Ascension 0–20, Acts 1–4 through the Heart**. A20 is the primary performance
target. Include normal reachable content: native/off-color cards, shared cards,
statuses/curses, character mechanics, enemies/bosses, relics, potions, events,
shops, rewards and rest actions. A run starts normally; it is not a combat fixture
or a fixed winning itinerary. An external caller makes every player decision.

Daily/custom/modded modes, career unlock progression, graphics and actual ML code
are excluded. No hidden pool cuts, inert content, approximate fights or automatic
choice policy are acceptable shortcuts. A known missing mechanic is an unfinished
product task, not permission to declare ordinary supported runs complete.

Use the already selected project RNG/maps, independent-profile and reward policies.
There is no original seed/map equivalence target or original-runtime fidelity claim.
Treat the product as **source-grounded, not original-runtime validated**.

## 2. Reuse versus actual gaps

| Existing code | Reuse / necessary extension |
|---|---|
| `spire-core::combat` owned state, typed effects, RNG and snapshots | Extend one engine with required cards, powers, characters, enemies, selection continuations and real combat entry/settlement |
| Current scenario/engine validation | Keep the supplied-ready combat contract; add run entry rather than pretending it already performs initial draws/hooks |
| Current boolean card upgrade state and small player/power model | Add upgrade counts/card-specific persistent state and required Silent, Defect and Watcher mechanics; do not force them into starter-card fields |
| `spire-env` worker pool, tokens, reset/delivery/error behavior | Reuse those utilities/contracts for native run slots; no Python simulation loop |
| Public encoding, native numeric exports, Gymnasium facades | Preserve combat-v1; add a separate full-run schema and facades |
| Authored research and accepted partial mechanics | Reuse at their actual grade; investigate only a concrete missing implementation rule |

The current core validates exactly one enemy and implements starter-card actions;
the existing observation schema also has one enemy and a three-card vocabulary.
Full-game support is real engine/content work, not a metadata relabel. The narrow
combat interfaces remain regression anchors, not the new full-game limits.
Character coverage includes Ironclad Strength/exhaust interactions, Silent discard/
poison, Defect orb/channel/evoke/Focus, and Watcher stances/Mantra, alongside their
other reachable card/item rules. These use the same effect/decision lifecycle.

## 3. Ownership and interfaces

**`spire-core` owns gameplay.** Add a `run` module containing creation/configuration,
persistent state and progression. Its coordinator invokes the existing, extended
combat engine. Content lives in ordinary typed Rust modules/data, not a DSL or
second interpreter. A permanent card's identity/state and its temporary combat
instances are related explicitly; copies do not alias mutable state accidentally.
Author implementations independently from qualified facts; do not copy or
mechanically translate original game source into the product.

**`spire-env` owns environment sessions.** Each slot owns a run, public-knowledge
state, reward bookkeeping, decision token and optional replay recorder. It encodes
public decisions and batches independent slots with existing native workers.
Existing `CombatPool` behavior stays supported; share useful worker/export helpers,
not a speculative universal pool/plugin abstraction.

**`spire-python` and Python own adaptation only.** Add `NativeRunPool`, `RunEnv`
and `RunVectorEnv` alongside the existing combat APIs. Python builds spaces,
validates/packs arguments and packages owned results; it never resolves game rules.
No separate Python full-run reference engine is required.

Conceptual native boundary; exact Rust names are fixed once in `.3`:

```text
Run::new(RunConfig, resolved_creation_profile, run_rng) -> Result<Run, RunError>
Run::decision() -> Result<PublicDecision | RunTerminal, RunError>
Run::step(GameAction) -> Result<{ next_boundary, committed_events }, RunError>
NativeRunPool.reset(seed_inputs, addressed_rows) -> observations + infos + tokens
NativeRunPool.step(candidate_indices, tokens) -> observations + rewards + statuses
```

`RunConfig` fixes character, ascension and standard fully-unlocked/Act4-enabled
rules. The content/ruleset and profile identities are recorded for replay. The
run owns HP, gold, permanent deck, relics/potions/counters, keys, act/floor/map,
pool/pity/history state and suspended room/combat choices. There are no real saves,
global mutable account preferences or hidden Python gameplay callbacks.
`committed_events` is a small typed result for actual floor/key/terminal changes
needed by reward bookkeeping, not an audit log or general event-sourcing system.

## 4. One decision lifecycle

A step validates a current legal action before mutation, commits it, then advances
automatic effects to the next player decision, genuine run terminal or explicit
fault. Core actions are typed; a published menu maps each candidate to an action.
Tokens bind native submissions to the slot and current decision. Candidate indices
are not permanent action/card identities and raw Gym integers cannot prove which
observation the caller used; preserve that existing documented limitation.

Decisions cover Neow, map movement, combat, event/shop/chest/reward/rest choices,
potions and card/target/ordered-selection continuations. Select cards incrementally
when needed; do not enumerate every subset. Confirm/cancel exists only when legal.
The pending choice owns its captured objects and resumes its actual parent once.
A nonterminal published boundary has at least one legal action. A zero-eligible
selection follows its actual automatic/cancel rule; it must not strand the caller
with an empty mask or invent a choice.

Combat victory settles the room and returns to the run; it does **not** terminate
the episode. Ordinary room/combat escape is not an early run terminal. Boss rewards
and required child choices settle before Act changes. Implement A20's second Act3
boss and correct key/Act4/Heart transitions, not direct Heart teleportation.

Invalid caller inputs reject without gameplay/RNG/reward advancement. After a
runtime failure, keep honest fault/reproduction information; do not manufacture a
loss, silently reset, roll back committed effects or permit accidental action retry.
Allocation/encoding/execution limits are operational faults, not game rules.

## 5. Public data and Gym interface

The new descriptor is `spire-gym-run-observation-v1`; do not widen combat-v1 in
place. One native descriptor owns column meanings, vocabularies, dtypes, shapes,
capacities and its fingerprint; Python must not maintain a second feature schema.
New full-run feature values use signed int64 and known/row/action masks use int8;
semantic fixed-width/binary32 rules are applied in gameplay before encoding, not
replaced by generic int64 arithmetic. Batch action indices are signed int64.

Public groups cover context/player, known map nodes/edges, permanent/combat cards,
actors/intents, powers, relics, potions, orbs/stances, visible offers/choices,
selection state and action-candidate features. Use numeric values, per-field known
bits and occupied-row masks. Hide unknown values **and** hidden lengths/order/
references; no unseen draw order, future event/offer, RNG/seed/cursor or internal
handle becomes a policy feature. Privileged reproduction is a separate explicit API.

Native live collections grow as owned Rust collections. For the initial Gym ABI,
use padded numeric `Dict` observations and `Discrete(A)` candidate actions, with
`action_mask[A]` and candidate features. A pool's immutable capacity configuration
fixes shapes for all its slots/characters; spaces do not change during an episode.
The native descriptor supplies the capacities and valid numeric domains. Their
literal preset values/column lists belong to the single encoder owner in `.16`,
before binding work uses them—not a separate schema compiler or source census.

No truncation, silent candidate removal or virtual paging actions. Insufficient
export capacity is an explicit resource/configuration error, never a fake loss or
claim that the game's collection has that limit. Larger capacities are selectable
when constructing a pool. The default preset must cover the declared acceptance
workloads; capacity/overflow and ownership tests are required. This is a disclosed
finite-memory transport constraint, not permission to omit reachable content.
The default is a versioned native preset; overrides validate before initialization
and remain fixed for that pool. No automatic shape change occurs on a later step.

```text
RunEnv(character=..., ascension=..., capacities=..., record_replay=False)
  reset(seed=None, options=None) -> (observation, info)
  step(candidate_index) -> (observation, reward, terminated, truncated, info)
RunVectorEnv(configs=[...], workers=..., capacities=..., record_replay=False)
  reset(seed=..., options={reset_mask: ...}) -> (batched_observation, infos)
  step(int64[B] candidate_indices) -> batched five-tuple
```

Character/ascension are explicit, not a hidden Ironclad/A0 default. Mixed profiles
share the full-run schema. Single reset is manual; vector uses explicitly declared
same-step autoreset, preserving the terminal observation/reward/info separately
from the replacement observation. Validate the complete addressed batch before
dispatch; partial reset leaves unselected runs unchanged. New facade metadata and
space/return types must pass tests against the pinned Gymnasium version.
Exported arrays must not mutate after return or alias later-step live storage.

## 6. Profiles, RNG and replay

Use an immutable resolved creation profile per episode, not inherited career state.
Retain the experienced independent-profile defaults: Neow eligible, ordinary boss
history populated, stored Note card Iron Wave/0, fully unlocked and Act4 enabled.
Character-specific normal starting decks/resources/relics and ascension effects
come from the selected ruleset. Do not let Gym reset inject a custom winning deck,
gold, keys, future map or arbitrary combat state. Internal test/snapshot APIs are
separate privileged facilities, not the normal full-run demonstration.

Existing accepted RNG-001 policy remains: project-owned reproducible streams,
intended conditional probabilities, without-replacement/history/pity behavior and
representative valid maps. Extend run-specific stream identities without changing
deployed combat-v1 streams. Reads, encoding and worker scheduling consume no
gameplay randomness. Equal per-run profile/seed/action sequences agree across
serial/batched execution; scalar vector seed expansion follows the existing
documented slot-index convention. Explicit reseeding restarts that slot's sequence;
`None` continues it. Replacement episodes use the configured creation profile,
never the retired run's Note/history/resources.

Replay records capture versioned content/rules/profile/RNG/reward identities,
effective seed and accepted semantic actions. From-start replay reconstructs the
run normally. Snapshots own core state plus environment knowledge/reward/continuation
state needed to resume. Reject incompatible records; no universal cross-version
codec, external save compatibility or generic event-sourcing framework is required.

## 7. Outcomes and already selected rewards

| Committed event / outcome | Reward / meaning |
|---|---|
| New highest floor in Acts1–3 | +1 |
| New highest floor in Act4 | +10 instead of +1 |
| First actual acquisition of each distinct key in this run | +5 |
| Heart victory | +100 once; successful terminal |
| Genuine death or selected settled-empty-permanent-deck loss | −1 terminal contribution |
| Legal Act3 completion without all keys | Non-goal terminal; no win bonus/death penalty |
| Other actions, reads, reset/abandonment, invalid input or operational faults | No extra reward; faults are not gameplay outcomes |

This reuses closed decision `spire-8jm.9.1`, including no duplicate/revisit/key
farming. A step may commit several reward events; sum each exactly once. Report
Heart success separately from accumulated return. Natural run endings set
`terminated`; the base gym imposes no artificial gameplay time-limit truncation.
Operational faults raise errors instead of returning a successful step. Callers
may stop collection, but that does not win/lose/truncate the unfinished run.

## 8. Acceptance and verification budget

Completion requires all scoped product components, not just the following tests.
These checks are practical acceptance evidence, not proof of exhaustive fidelity:

| Check | Required evidence |
|---|---|
| Content/mechanics | Every reachable definition wired to real behavior; focused expected-value/order/lifecycle tests for changed mechanics, not row counts or hashes |
| Profile/ascension | Creation/start rules for all4 × 21 configurations; targeted threshold cases, all four character mechanics, A20 two-boss path and public-visibility boundaries |
| Complete runs | At least one reproducible legal from-start Heart-path case per character; actual choices/fights/key collection, plus separate A20 transition checks |
| Terminals/errors | Genuine losses, legal no-key completion, unsupported/internal faults and exactly-once reward/settlement are distinct |
| Native/Python/batch | Same results for equal per-run inputs; masks/tokens, reset/autoreset/finals, ownership and snapshot/replay tests |
| Delivery | Documented non-learning rollout command, existing Rust/Python suites, explicit limitations and one integrated review |

Target invocation delivered by `.21` (the new script does not exist yet):

```sh
.venv/bin/python -B examples/run_gymnasium.py --character ironclad --ascension 20 --seed 7 --num-envs 1 --policy random
```

That is a smoke rollout, not a promised winning seed. Publish the actual legal
seed/action traces for the four Heart-path acceptance cases once they execute;
do not invent outcomes or build a learner/search platform to certify the example.

Use existing test runners, small deterministic fixtures and seeded replay.
Late-Act fixtures supplement, not replace, from-start runs. Fix actual product
bugs with regressions. Preserve the known readiness issue `spire-vhh` if still
present; do not call a qualified suite fully green or add a GPU gate to gym work.
One small declared release workload may measure A20 runtime/memory sanity; no
invented speed target, optimization campaign or learned-strength result.

**No verification rabbit hole:** every check names a concrete product behavior or
defect. No Java parser, symbolic executor, law DSL, general verifier, receipt
pipeline, source-wide/foreign-body census or broad-copy/mutation farm. Reuse facts
at their actual grade; inspect only a load-bearing unknown. Normal TDD iterations
are expected. Repeated review-level failures of a verification approach require a
bounded root decision, not another checker rewrite or review swarm per assertion.

## 9. Handoff to the execution plan

Beads `.4`–`.15` own gameplay/content; `.16`–`.20` own encoding/reward/replay and
interfaces; `.21` integrates all implementation components; `.22` reviews the
whole result. `.3` will assign concrete files/contracts and Luna work waves.
Pin shared Rust state/content/action interfaces first; only then parallelize
disjoint implementations. Do not dispatch all20 tasks together or have multiple
workers rewrite shared enums/modules. Keep one concise design and plan, not one
approval cycle per card/field. No old verification worker is resumed automatically.

Written-design approval permits the `.3` implementation planning pass; reviewed-
plan and explicit start decisions still precede product implementation. No
commits, pushes, installs, original runtime or private source publication is
authorized by this document.

### Reference boundaries

- [Existing combat environment contract](../../combat-environments.md) and current
  public Rust core/environment/binding source establish what is already implemented.
- [RNG-001](../../intentional-deviations.md) retains the selected randomness policy.
  Historical Act1 RNG/map/profile/native research supplies qualified mechanics and
  engineering context, not an Act1 cutoff, blanket acceptance or another audit gate.
- [Repository scope](../../scope.md) and Beads are current. This design does not
  promote static research into original-runtime validation.
- Official Gymnasium [spaces](https://gymnasium.farama.org/api/spaces/composite/)
  and [vector API](https://gymnasium.farama.org/api/vector/) informed the thin API;
  implemented behavior must be tested against the repository's pinned dependency.
