# M1: usable basic combat

M1 exposes **authored Ironclad A0 combat**, not a normal-start run, unrestricted
card pool or full-game gym. It reuses the authoritative Rust combat engine.
The scripted caller is outside the simulator; no ML code or automatic gameplay
policy is embedded in the engine.

## Run it

```sh
cargo run -p spire-core --example m1_combat --locked --offline -- --scenario all --seed 7
cargo test -p spire-core --test m1_combat --locked --offline
```

Options: `--scenario all|victory|death|modifiers`, `--seed <u64>` (default7),
`--help`. Each scenario allows at most64 accepted actions. Exhausting a budget
returns an error/nonzero exit, never a fabricated Death. `modifiers` finishes
after its named checkpoints, not after claiming those fights or a run are won.

The roster is unupgraded **Strike, Defend, Bash, Clothesline**, energy3, drawing
all four supplied cards each turn. Enemy fixtures use native A0 **Jaw Worm**
(42HP/Chomp opener), **Green Louse** (15HP, Bite6, Curl Up5, actual SpitWeb AI)
and **Gremlin Nob** (85HP/Bellow opener, actual SkullBash/BullRush AI and Enrage).
These are explicitly supplied setups, not encounter-pool sampling or a normal
starting deck. Player HP starts at80, except the injured11HP loss fixture.
There is no relic/inventory/run integration in this example.

Representative literal summary lines from `all --seed 7`:

```text
scenario=victory actual_outcome=Victory
scenario=death actual_outcome=Death
modifier checkpoints complete; accepted_actions=16 (not a completed run)
```

The intervening output contains only public HP/Block/energy/powers, current
intents and raw incoming damage, visible hand identities, draw **count**, menus,
chosen actions and actual outcomes—not hidden RNG or draw order. Menu/hand order
is not a stored test golden.

## Demonstrated rules

| Checkpoint | Exact behavior |
|---|---|
| Bash then Strike | Bash deals8 and applies Vulnerable2; Strike then deals9, total17HP loss against unblocked Jaw Worm |
| Defend against Chomp | Raw11, Block5, HP loss6 (80→74) |
| Clothesline, Defend, Chomp | Clothesline deals12/applies Weak2; Chomp raw8, Block5, HP loss3 (80→77) |
| Actual SpitWeb then player attacks | Weak makes Strike6→4 and Bash8→6; Louse Curl Up adds5 Block after Strike, so Bash absorbs5 and loses only1 additionalHP |
| Actual SkullBash then BullRush | Player Vulnerable makes base14→21 before Block; no Skills are played into Enrage in this fixture |
| Player-applied Weak2/Vulnerable2 | Amount2 after the card,1 after the first enemy phase, removed after the second |
| Enemy-applied Weak2/Vulnerable2 | Amount2 at the first player decision after application,1 at the next, then removed without intervening reapplication |

Lifetime tests use valid seeded AI histories: Louse seed7 and Nob seed8, whose
Bellow→SkullBash→two BullRushes avoids reapplication during the expiry check.
The example accepts other seeds; changed AI/outcomes are real, and a fault or
unfinished budget is reported rather than replaced with a scripted result.

## Public API

See [`m1_support`](../../crates/spire-core/examples/m1_support/mod.rs) for a complete
typed `CombatEntry` fixture and [`m1_combat`](../../crates/spire-core/examples/m1_combat.rs)
for the caller. The essential API is:

```rust,ignore
let mut combat = Combat::enter(entry)?;
let observation = combat.public_full()?;
let actions = combat.legal_actions_full()?;
// The external caller selects a member of actions.
combat.step_full(&action)?;
```

`legal_actions_full` is read-only, with stable1-based enemy-slot targets. It uses
the same card preflight as execution, excludes unaffordable/wrong/dead targets,
and returns an empty menu at Victory/Death. Unknown/unimplemented handlers return
an explicit error rather than silently shrinking the profile. Faults are retained
errors, not gameplay losses. M1 originally rejected selection screens; the
[M2 extension](m2-combat.md) now exposes selection-only legal menus and public
potion actions. Legacy `legal_actions()` remains unchanged.

Rejected actions and repeated public/menu reads leave detached snapshots equal.
Full-mode snapshot restoration is not added: replay reconstructs the authored
entry with the same seed and accepted actions, comparing public checkpoints.

## Verification and limits (2026-10-22)

**Accepted at the declared M1 scope** after one independent review in its own
materialized source worktree, with no findings. Controller confirmed reviewed
source unchanged and fresh focused tests/diffcheck before closing M1. This does
not accept the known draft failures or claim a green full workspace.

- Five focused public-API tests pass; the command completes in under1second on
  the development machine, with test execution reported as0.00s. Tests use exact
  rule values, small fixtures, bounded histories and no sleeps/network/log goldens.
- Integrated core run: **335 passed,8 failed**, exactly the same eight unfinished
  draft tests observed before M1 product changes: six Poison/Catalyst/Artifact
  cases, Blue Slaver/Two Fungi setup, and the six-potion consumer fixture. None
  was deleted, skipped or falsely called a pass. The initial baseline also had a
  test-only borrowed-target compile error, repaired by cloning the same handle.
- Because Cargo stops on those core failures, accepted consumers were run
  directly: **38 environment +5 binding tests pass**. Existing content/item/map/
  trace integration targets also pass (**21 tests**). Python extension/Gymnasium
  binaries were not rebuilt or revalidated in this milestone.
- Workspace build/test command wall time is about1.2seconds (build1.05s, core
  test execution0.14s). Diagnostic warm targets are5s focused/30s workspace,
  not timing assertions or hardware-independent correctness guarantees.

These checks demonstrate the declared combat path, not normal runs, complete
character/card/item/encounter coverage, original seed parity or original-runtime
fidelity. The implementation remains **source-grounded/reference-informed, not
original-runtime validated**. Full scope remains the approved four-character
A0–20 Acts1–4/Heart [roadmap](../superpowers/specs/2026-10-22-narrow-gym-milestones-design.md).
