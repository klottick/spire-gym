# M9 — captured Ironclad A0 through Act3 and single-Run Gym

**Accepted at the declared scope after ONE independent GPT-6 Luna milestone review.** This extends the
[accepted M8 profile](m8-act3.md), not the full standard pools, all characters/A0–20,
Act4/Heart or original-runtime fidelity. The final roadmap scope is unchanged.
[Approved plan](../superpowers/plans/2026-10-23-m9-ironclad-act3.md).

## Current rules compatibility 2026-10-24

This dated notice supplements, and does not rewrite, the historical M9 acceptance
and publication results below. Shared random-spot draw insertion was corrected from
n+1 gaps to n positions. The current native candidate uses `spire-gym-native-m9-rules-v2`; replay
wire schema remains `spire-gym-run-replay-v1`, with numeric-v1 spaces, action
vocabulary and descriptor fingerprint unchanged. The HMAC RNG algorithm and root
adaptation are unchanged.

Corrected binaries reject preserved `m9-rules-v1` records at identity preflight;
there is no promised compatibility mode. Historical replay requires the preserved
old source/binary. Current corrected-rules outcomes may differ: seed7 all three
Act3 histories now end in Death at action697/floor42/reward41, versus the historical
v1 result action778/floor47/reward46. This is not a strategy change or old-trace
compatibility claim. See [current M10 compatibility/source notes](../research/2026-10-24-m10-act4-and-ironclad.md).

This notice makes no claim of M10 milestone completion, Python extension refresh,
or original-runtime parity.

The deliverable is simulator/API tooling for externally developed strategy.
Neither a winning example policy nor learned-agent strength is an acceptance gate.

## Use it

```sh
cargo run -p spire-core --example m9_act3 --locked --offline -- --seed 11 --scenario all
cargo test -p spire-core --test m9_act3 --locked --offline
cargo test -p spire-env --locked --offline
UV_PROJECT_ENVIRONMENT=target/m9-gym-dev-venv uv run --locked --no-sync python -B examples/run_m9_gymnasium.py --character ironclad --ascension 0 --partition test --boss-history time-eater-unseen --seed 11 --budget 8192
```

Use the existing configured Gym-only environment or the separately authorized
[development setup](../development.md#gym-only-development-and-on-demand-quality-checks).
Commands here are not authority to install dependencies or rebuild an extension.

```python
from spire_lab.gymnasium import RunEnv

game = RunEnv("ironclad", 0, partition="test", record_replay=True)
try:
    obs, info = game.reset(seed=11)
    action = game.action_space.sample(mask=obs["action_mask"])
    obs, reward, terminated, truncated, info = game.step(action)
    record = game.record()  # privileged bytes; never an agent feature
finally:
    game.close()
```

Constructor requires character/ascension; only `ironclad`, `0` are supported.
Options: `partition=train|selection|final|test`, independent prior-boss history
`awakened-unseen|time-eater-unseen|donu-deca-unseen`, fixed `capacities` overrides,
and opt-in `record_replay`. No deck/HP/gold/keys/scenario injection through reset.
These histories select the Act3 icon, not future combat outcomes.

## Actual normal-start evidence — 2026-10-24

Every episode starts with80/80HP,99gold, five Strikes/four Defends/Bash, Burning
Blood, three empty potion slots and no keys. Same Run/player through all acts.
Short Neow and valid independent discovery histories match the earlier callers.

| Seed / Act3 icon | Actions | Actual final encounter/floor | HP/max | Reward |
|---|---:|---|---:|---:|
|7 / Awakened One|778|Sphere and Two Shapes Death47|0/105|46|
|7 / Time Eater|778|Sphere and Two Shapes Death47|0/105|46|
|7 / Donu/Deca|778|Sphere and Two Shapes Death47|0/105|46|
|11 / Awakened One|711|Giant Head Death41|0/105|40|
|11 / Time Eater|813|Maw Death47|0/119|46|
|11 / Donu/Deca|830|Maw Death47|0/119|46|

All six original losses remain unchanged, with every public decision/map/reward
replayed. Core→native session numeric equality is checked at every boundary;
fresh binding→Gym arrays/rewards/endings and the complete semantic records agree.
**No normal-start boss50 entry or no-key victory was observed.** Boss/end coverage
below uses separately labelled supplied native diagnostics, not winning runs.
No seed search, resource injection or policy tuning was used to manufacture wins.

The Python demo samples current masked actions (not the frozen native policy).
Observed random seed11: Death67 actions/floor4/reward3; seed7: Death27/floor1/
reward0. Its nonattacking diagnostic dies in9 actions/floor1/reward0. Death exits0;
budgets0/1 are unfinished errors,8193 is invalid, not termination/truncation.

## Native progression and finite content

`RoomScenario::M9IroncladAct3` continues the M8 captures through every generated
late-Act3 edge, floors35–49, real rest49 and map-selected boss50. It does not change
M5's Act2Ready17, M6's chest26, M7's Act3Ready34 or M8's chest43 cutoffs.

- Ordinary/elite encounters, Falling/Tomb then Monster questions, card/potion rows,
  prices and purge costs reuse the declared M8 captures. No standard pool claim.
  Counters/cursors/history continue through43; later shops disable owned unique
  stock. No route pruning, reroll or inventory refill to conceal exhaustion.
- The existing eight elite relics are followed, in M9 Act3 only, by **Vajra**
  (Strength1), **Oddly Smooth Stone** (Dexterity1), **Bag of Preparation** (two actual
  extra opening draws), **Bag of Marbles** (initial enemies Vulnerable1, subject to
  Artifact), **Bronze Scales** (player Thorns3 per incoming NORMAL hit, even if fully
  blocked; not HP_LOSS/THORNS/self/no-attacker damage). Real acquisition and native
  startup/retaliation execute, not inert catalog rows or permanent stat injection.
- At most five elites fit an Act3 path: two in rows5–7, three in9–13, chest row8
  and no adjacent same-kind parents. Five extra unowned relics cover that bound;
  stocks skip owned items and advance once, never fabricate duplicates/Circlets.
- Chest43 retains optional Proceed. Exhausted/owned relics are Disabled; unowned
  Sapphire remains an exclusive alternative. Exhaustion alone does not block exit.
- Actual Awakened revival/Cultist escape, Time Warp payload/disposition/forced turn,
  and Donu/Deca partner work must finish before the one combat settlement owner.
  Supplied native boss-entry/lifecycle/gain-census/fault diagnostics cover all three.

## Qualified final source and outcomes

[Task-local static findings](../research/2026-10-23-m9-act3-ending.md) close the
actual reward/Proceed/Heart sequence. Owned archive upstream build/channel remains
unverified; no original game execution or Java PRNG/cursor parity is claimed.

Final A0 Act3 boss Victory suppresses combat-reward-screen publication: **no
claimable gold/card/potion rewards or third boss chest**. Actual boss Proceed then
three Heart dialog choices follow. With any missing key, a fourth actual choice
commits `Act3NoKeys` once, not combat Death. With all three actually acquired keys
and Act4 enabled, the third choice stops pre-door at `Act4Ready`, act3/floor50:
alive, nonterminal/no actions, no heal/floor51/new-act or readiness bonus. The Gym
adapter retains that accepted state and raises `RunUnsupportedBoundaryError`;
it never returns a successful nonterminal empty mask or fake truncation.

Supplied native tests exercise all eight key combinations, all three actual boss
sources, full belt, disabled Act4, stale choices and preparation/publication faults.
They are not normal-start key or completion demonstrations.

Rewards: each highest newly entered Acts1–3 floor+1, each first distinct key+5,
Death/settled EmptyDeck−1, Act3NoKeys contribution0, real HeartVictory+100 only.
Operational failures and caller budgets add no penalty or completion reward.
Manual reset only; base `truncated=False`. Terminal reasons are `death`,
`empty_deck`, `act3_no_keys` (future `heart_victory` is not delivered by M9).

## Numeric transport, reset and replay

`observation_spec()` is Rust-owned `spire-gym-run-observation-v1`, separate from
unchanged combat-v1. Standard fixed Dict arrays: int64 features, int8 per-field
known bits, row occupancy and action mask; Discrete candidate indices. Unknowns
are not known zero. Public rows/relative references replace internal handles;
no draw-pile rows/order, hidden lengths, seed/cursors, private histories or queues.
Ordinary info contains only `schema_id` and public `terminal_reason`.

| Row group | Default capacity | Observed six-trace high-water |
|---|---:|---:|
|map_nodes / map_edges|106 /742|69 /83|
|permanent_cards / combat_cards|256 /1024|24 /34|
|enemies / powers|32 /2048|9 /16|
|relics / potion slots|256 /16|9 /3|
|offers / selection|512 /256|5 /16|
|orbs / actions|10 /2048|0 /28|

These observations are not proven maxima for every strategy or unbounded combat.
Capacities are immutable transport settings, not game limits. Every row axis has
explicit overflow coverage; overflow never silently drops rows or legal actions.

Seed adapter `spire-gym-run-u64-le-v1`: exact built-in unsigned64-bit integer,
first8 bytes little-endian in zero root32, native Partition and checked ordinal.
Reject bool/negative/overflow/NumPy seeds. Explicit seed resets ordinal0;
`reset(None)` continues ordinal+1, first unseeded reset is seed0/ordinal0.
Gym's compatibility RNG and caller sampling do not drive native gameplay.

`NativeRunSession.step(index, token)` rejects stale/cross-session/illegal candidates
without mutation. The facade holds its current token; a bare Gym integer cannot
prove observation freshness if it still names a legal current candidate.
Outputs are owned/C-contiguous/native-endian and do not alias native or earlier
outputs. Native operation ownership spans final Python tuple packaging; reentry
and concurrent operations reject Busy, without rerunning accepted work.

An accepted core/encoding/packaging fault retains the actual Run, source, gains,
RNG, action and first fault. It freezes subsequent reset/step; inspect privileged
`reproduction()`/`record()` even after idempotent close. No rollback/auto-reset,
retry or fabricated loss. A healthy reset intentionally abandons the old episode;
retrieve its optional record first. Opt-in recording grows with accepted actions.

Semantic bytes use `spire-gym-run-replay-v1`, exact content/rules/reward/schema/
seed-adapter identities, profile/history/seed/ordinal and typed actions, not
transient candidate indices. `_native.NativeRunSession.replay(record)` reexecutes
the same Run and reports original portable fault metadata separately. Full native
fault custody remains in the live Rust reproduction owner; external packaging/
allocator faults are not promised to recur. No universal private snapshot codec.

## Verification boundaries

Fresh reviewer found no Critical/Important/Minor findings and assessed all five
focus classes as satisfied. All490 materialized public source files and the
extension hash were unchanged in both controller/reviewer checkouts before final
controller verification: core675/7 with exact historical assertion bodies,
publicM1–M9 59/0, spire-env56/0, native binding5/0, Python environment195/0 and all
three changed-file prek checks. Only administrative acceptance text changed after
that review; no commit, push, merge, cleanup or M10 implementation is implied.

Tasks9/10 use the freshly rebuilt local extension, Python3.12.15, NumPy2.5.4,
Gymnasium1.4.0, pytest9.1.1 and maturin1.15.0. Environment tests195/0, including167
unchanged combat tests,22 Run boundary/ownership tests and six from-start agreement
cases. Changed Python/stub paths pass pinned prek/Ruff/ty on demand. No shared
Git hooks, broad reformat or training dependencies are required.

The Gym-only environment intentionally has no Torch: an attempted full Python
collection reports `tests/python/test_readiness.py` missing Torch, not a gym defect.
The seven historical Rust Poison/Catalyst/Artifact/BlueSlaver–Fungi assertions
remain explicit; no whole-workspace-green claim. Reference datasets unchanged.
Broader non-readiness Python testing reports1019 passes/68 failures:60 unchanged
legacy differential snapshot-field mismatches and eight metadata fixture failures
from running temporary crates under this Cargo workspace. Initial `/tmp` quota
errors and the local-temp rerun are retained, not hidden by a green-suite label.
The original environment affected by maturin's automatic Torch/CUDA install is
preserved; actual verification uses a separate Gym-only environment and no-deps
editable metadata install. See development instructions to avoid repeating it.

No RunPool/vectorization, Act4 gameplay, complete random pools, other characters/
Ascensions, learning system, universal wrapper compatibility or runtime fidelity
promotion is included. M10 is planning-only after M9 acceptance, not automatic
implementation or publication authority.
