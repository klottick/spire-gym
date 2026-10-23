# M2: shared combat mechanics

M2 extends [M1](m1-combat.md)'s **authored Ironclad A0 combat**, not normal
runs, complete pools or a full-game gym. One Rust state/queue owns all gameplay;
the example supplies initial fixtures and chooses public legal actions.

## Run it

```sh
cargo run -p spire-core --example m2_combat --locked --offline -- --scenario all --seed 7
cargo test -p spire-core --test m1_combat --test m2_combat --locked --offline
cargo test -p spire-core --lib combat::tests::shared_mechanics --locked --offline
```

Options: `--scenario all|powers|selection|damage`, `--seed <u64>` (default7),
`--help`. Each scenario allows at most64 accepted actions; exhausted budget is a
nonzero error, not Death. Only the lethal HP-loss checkpoint claims a terminal
outcome; other checkpoints do not imply a completed fight/run. M1 remains runnable.

Literal summary output from `all --seed 7`:

```text
scenario=powers checkpoints complete; accepted_actions=19
scenario=selection checkpoints complete; accepted_actions=5
scenario=damage checkpoints complete; accepted_actions=11
```

Intervening output shows public HP/Block/energy/powers, intents, hand identities,
draw **count**, discard/exhaust counts, inventory, selection, menus/actions and
outcomes—not hidden streams or draw order. Logs/menu order are not test goldens.

## Declared profile and exact checkpoints

Supplied base cards: M1 Strike/Defend/Bash/Clothesline, reusable Pommel Strike/
Shrug It Off/Slimed and new **Bloodletting/Burning Pact/Disarm**. Tests include
the three new cards' upgrades. Native enemies: M1 Jaw Worm, Green Louse, Gremlin
Nob and existing A0 medium Spike Slime. HP normally80; injured20/3HP fixtures
test self-loss. No enemy rules live in the caller. This is not a complete pool.

Demonstration inventories have at most3 slots. Existing finite Strength/Block/
Fire/Swift/Weak/Fear/Energy handlers are reusable; new **Dexterity Potion** and
**Ancient Potion** provide stat/Artifact producers. Sacred Bark cases live in
focused tests, not oversized example inventories.

| Behavior | Exact representative result |
|---|---|
| Strength Potion then Strike | Strength2; Strike6→8 |
| Disarm / upgraded | Cost1, enemy Strength−2/−3, Chomp11→9/8, parent exhausts |
| Dexterity Potion / Sacred Bark | Dexterity2/4; Defend5→7/9 |
| Frail / Frail with Dexterity2 | Defend5→3 / `(5+2)×0.75` floors to5 |
| Direct Block12 with Dexterity/Frail | Remains12; potion/orb/enemy gains do not use card scaling |
| Ancient Potion / Sacred Bark | Artifact1/2; one charge prevents one whole supported ordinary debuff application |
| Prevented stack | Weak2 stays2 when another Weak3 is blocked; Artifact is not cleansing |
| Energy Potion / Sacred Bark | Energy3→5/7; max-energy stays3; invalid overflow does not spend slot |
| Bloodletting / upgraded | Cost0; HP_LOSS3 first, then energy2/3; HP20/Block5→HP17/Block5 |
| Lethal self-loss | HP3/Block5→Death; Block retained and later energy does not execute |
| THORNS against Block | Raw20/Block7→13HP loss, no Strength/Weak/Vulnerable scaling |
| Selected exhaustion/recycle | Six-card sequence ends hand4/exhaust1/discard1, energy0; only selected Defend identity exhausts |

Burning Pact costs1, exhausts one remaining hand instance, draws2/3 and then
discards the parent once. Empty hand still draws; singleton is forced; multiple
choices suspend for a real selection. Existing cap10, insufficient supply and
ordinary shuffle/recycle apply. Selected, forced and finish exhaustion each
notify `CardExhausted` once for the actual owned identity.

Full-mode Strength clamps at[−999,999]; zero is removed. Dexterity has an explicit
supported native signed surface[−999,999], not an asserted game-wide clamp.
Ordinary positive Weak/Vulnerable/Frail and negative Strength/Dexterity applications
are supported debuffs. Buffs/reductions/removals/HP loss do not spend Artifact.
Future immunities, direct-application exceptions and temporary-stat powers are
not implemented by this checkpoint.

Frail follows M1 Weak's source-sensitive grace: monster application survives its
application round and is present at the first affected player decision, then
expires after the next round without reapplication. Fixed seed7 Spike Lick→Flame
Tackle and Green Louse SpitWeb histories supply real producers; tests assert the
non-reapplication prerequisite. Other seeds may change AI/checkpoint duration;
unfinished budgets/errors are reported, not replaced with scripted results.

## Public API and ownership

See [`m2_support`](../../crates/spire-core/examples/m2_support/mod.rs) and
[`m2_combat`](../../crates/spire-core/examples/m2_combat.rs) for the typed caller.

```rust,ignore
let mut items = Some(CombatItemContext::new(relics, potion_slots));
let mut combat = Combat::enter_with_item_context(entry, &mut items)?;
let view = combat.public_full()?; // detached optional inventory and selection
let actions = combat.legal_actions_full()?;
combat.step_full(&CombatAction::UsePotion { slot: 0, target: None })?;
```

`CombatItemContext` is public under `spire_core::items`; mutable fields remain
private. `CombatPublic::inventory` is a detached projection, not a second mutable
inventory. Targets are stable1-based enemy slots; potion slots are zero-based.
Reads and rejected actions do not change resources or RNG.

`CombatEntryFailure::Rejected` retains the caller's `Some(items)`; `Faulted {
combat, fault }` retains the actual post-transfer mutated combat, without
refunding/replaying startup. Debug does not format that combat or hidden streams.
Accepted potion faults retain the consumed slot, attempted action and work;
retry returns the same fault. Unknown/unimplemented inventory IDs reject before
transfer. Normal room acquisition and run-inventory persistence remain later work.

At Selection, menus contain **only captured eligible unselected `SelectCard`**
identities. Public selection includes candidates, required count, selected IDs
and disposition. Potion/card/end-turn actions are rejected there. Parent cost/
work/finish occur once; stale/repeated picks cannot resume it again. Ready menus
and execution share pure preparation. Terminal menus are empty; faults remain
errors, not losses. Legacy `Action`, `legal_actions()` and combat-v1 remain
unchanged. Full-mode snapshot restoration is still absent.

## Verification and limits (2026-10-23)

**Accepted at declared M2 scope** after one isolated independent GPT-6 Luna
review with no findings. Controller verified all201 reviewed public files
unchanged and freshly reran M1/M2/shared checks plus clean diffcheck before normal
closure of `.24.2`/`.24`. M3 `.25.1` is open for planning only.

Checks: M1 public5/0, M2 public12/0, shared8/0, accepted consumers43/0,
other core integrations21/0. Both examples run. The old six-potion consumer now
passes after removal of its invalid Chomp override from the Louse-only sampled
Bite map; canonical Chomp11 and all resource/damage assertions are unchanged.

Fresh pre-M2 baseline: core335/8. Six unfinished Poison/Catalyst/Artifact drafts
and Blue Slaver/Two Fungi setup remain outside M2, preserved/not skipped. An
integrated run also exposed an old test rejecting wide Block gains; its contract
was updated to approved wide-add/cap semantics while retaining negative/current/
legacy bounds. Final corrected core run: **344passed/7failed**; all seven failure
names and messages match the executed pre-M2 baseline. **No green-workspace or
full-run claim.**

Candidate workspace wall1.92s (build1.77s, core tests0.14s); consumers wall0.28s
(build0.01s, tests0.22s). Timing is diagnostic only: no sleeps/network or
machine-dependent correctness assertions. No installs/dependency changes/commits
or publication; Python extension/Gymnasium binaries were not rebuilt.
Corrected core wall1.00s (build0.84s, tests0.15s); final reviewed public/shared
commands wall0.019/0.014s, build/test reporting0.00s.

This is **source-grounded/reference-informed, not original-runtime validated**.
Normal encounter sampling, room progression, rewards and consecutive-fight
settlement remain M3/later work. Poison ticks/cards and complete content are not
silently included. Final scope remains four characters/A0–20/Acts1–4/Heart.
