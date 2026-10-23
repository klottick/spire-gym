# M3: persistent consecutive combat

M3 connects real combat to one persistent Rust run owner. It is an **authored
Ironclad A0 checkpoint**, not normal Neow/map/reward progression or a full run.
It reuses [M1](m1-combat.md)/[M2](m2-combat.md), not another simulator.

## Run it

```sh
cargo run -p spire-core --example m3_run --locked --offline -- --scenario all --seed 7
cargo test -p spire-core --test m1_combat --test m2_combat --test m3_run --locked --offline
```

Options: `--scenario all|consecutive|death`, `--seed <u64>` (default7), `--help`.
Each scenario allows≤128 accepted combat actions; the consecutive script makes
exactly2 entries, the death script1. Exhausting a caller budget is an explicit
nonzero unfinished error, never a fabricated Death or Victory.

Literal observed summary output for `all --seed 7`:

```text
scenario=consecutive fights=2 hp=66 max_hp=80 gold=123 terminal=None accepted_actions=17
scenario=death actual_terminal=Some(Death) hp=0 accepted_actions=1
```

Output between these lines contains public resources/inventory/deck, intents,
hand identities/origins, draw count, captured selections, menus/actions and
committed events. No hidden streams/draw order or raw Run Debug is printed.
Output order/Debug wording are not stored test goldens.

## Explicit profile and resource checks

`Run::from_authored_combat` accepts Ironclad/A0 supplied resources; standard
`Run::new` and its84 start profiles remain unchanged. Supported authored encounter
requests are **Jaw Worm/Cultist**, built by native enemy setup with real sampled
HP (Jaw40–44, Cultist48–54), AI and visit-separated streams. This is not a reduced
standard encounter pool. Other bridge requests explicitly error.

Permanent input cards: Strike, Defend, Bash, Clothesline, Iron Wave, Pommel Strike,
Shrug It Off, Anger, Bloodletting, Burning Pact, Disarm, upgrades0/1. Existing
finite item handlers are reused; unknown/unimplemented inventory rejects. Initial
HP must be1..max≤999, deck1..999 unique nonzero permanent IDs with available next
identity, and exactly3 A0 potion slots. No normal-start reachability is asserted
for arbitrary authored inventories. No map node or FloorEntered event is forged;
map-dependent item qualifiers retain existing native context rules.

Main supplied fixture: HP40/max80, gold123, five permanent cards `[Strike+, Bash+,
Anger, Bloodletting, Disarm]`, Burning Blood/Blood Vial/Lantern/Anchor, and slots
`[StrengthPotion, FirePotion, BloodPotion]`. Standard energy3/draw5; resources are
supplied only at construction, never injected between fights.

| Checkpoint | Actual semantic expectation |
|---|---|
| First entry | Vial40→42HP, Lantern4energy/max3, Anchor10Block; entered1/settled0 |
| First turn | Disarm exhausts its permanent card; Bloodletting loses3HP through Block; Strength Potion gives2; Anger/Bash+/Strike+ deal8+12+16 before Fire20 kills |
| First exit | Burning Blood39→45 once, not51; slots `[None,None,BloodPotion]`; permanent five-card deck unchanged |
| Second entry | Vial45→47; fresh Lantern4/Anchor10; no old Strength, Anger copy or exhausted-card exclusion; same permanent IDs/upgrades |
| Second turn1 | Blood Potion16→63, Bloodletting3→60; attacks29 leave Cultist alive; real Incantation executes |
| Second turn2 | Energy3 (no repeat Lantern); original/local Anger9+9, then permanent Bash+15 kills while Bash remains in_play at terminal export |
| Second exit | Burning Blood60→66 once; max80/gold123 unchanged, slots all empty, entered2/settled2, visits0/1; no run terminal invented |
| Loss | Supplied11HP with Burning Blood only; actual Jaw Chomp11→Death0, no resurrection/heal/reward |

Additional focused cases cover real Fruit Juice max80→85 persisting across
settlement/next entry, Pact selection through the run action menu, same-encounter
stream separation and opaque relic-counter storage. The six-card Pact fixture
uses a real Swift Potion to draw its sixth card; it does not override standard
draw5. Relic-counter preservation does not claim new counter-relic mechanics.

## API and ownership

See [`m3_support`](../../crates/spire-core/examples/m3_support/mod.rs) for the
typed initial resources and [`m3_run`](../../crates/spire-core/examples/m3_run.rs)
for a caller. The essential native API is:

```rust,ignore
let mut run = Run::from_authored_combat(config, resources, rng)?;
run.enter_combat(EncounterId::JawWorm)?; // explicit authored entry, not a map choice
let observation = run.decision()?;
// Choose one observation.candidates[i].action: GameAction::Combat(...).
run.step(&action)?;
// Actual terminal combat automatically settles before returning.
```

`choose_combat` is a compatibility adapter to the same path. Combat-only potion
and selection commands stay inside `GameAction::Combat`; no second action engine.
Active observations project current combat-owned HP/maxHP/items, not the stale
persistent checkpoint or temporarily empty run inventory. The permanent deck
stays canonical in RunState. Public values are detached; reads/invalid requests
do not consume a potion, RNG, visit or gameplay action. Outside actions and deck
removal are rejected while combat owns resources.

Entry rejection leaves the run unchanged. An accepted startup fault retains its
actual mutated Combat, transferred inventory, visit and first fault. An accepted
potion fault retains consumed resources/current work/action provenance. Neither
fault becomes Death or retries startup/use. Settlement validates all permanent
origins across every pile **and fatal in_play**, excluding generated CombatLocal
cards; exhausted originals are not deleted. Unsupported permanent transformations
fail explicitly. Next entry reconstructs fresh local state with native inherent
card flags, rather than copying the previous combat's temporary state.

Combat already dispatched its real settlement hook/Burning Blood. The bridge
returns those post-hook HP/maxHP and actual inventory once; it does not call the
run-level heal hook again. A validation/conflicting-owner settlement fault retains
the terminal combat, not a retryable accepted kill. Gold/permanent deck/opaque
relic counters are not overwritten. `CombatProgress` counts entered and settled
outcomes (including death), not wins; `last_combat` reports actual outcome/visit.

Entry returns one CombatEntered event, live/selection actions return none,
victory returns CombatFinished, and death returns CombatFinished plus Ended(Death).
Run history records each once. Between authored fights there is no fake reward,
continue or map menu. Gameplay death remains readable and terminal; later actions
cannot advance it. Two victories are not HeartVictory or a finished run.

## Verification and limitations (2026-10-23)

**Accepted at declared M3 scope** after one isolated independent GPT-6 Luna
milestone review with no findings or declined judgments. Controller verified
all208 reviewed public files unchanged and freshly ran public28/0, internal3/0,
shared10/0 and clean diffcheck before normal closure of `.25.2`/`.25`.
M4 planning-only `.26.1` is open/unassigned, not product start.

Checks: public M1 5/0, M2 12/0, M3 11/0; internal bridge3/0 and shared10/0.
Integrated core **349passed/7failed** versus executed pre-M3 baseline344/7: all
seven old failure names/messages match exactly (six unfinished Poison/Catalyst/
Artifact card/tick drafts and Blue Slaver/Two Fungi setup). No tests skipped or
draft failures accepted. Cargo stops before consumers on those failures, so
accepted consumers43/0 and all core integrations49/0 were run directly. M1/M2/M3
examples run. **No green-workspace/full-run claim.**

Workspace wall2.64s (build2.48s/core tests0.14s), consumers wall0.26s
(build0.01s/tests0.20s); M3 example wall0.145s/build0.13s. Times are diagnostics,
not correctness gates. No installs/new dependencies/commits/cleanup or Python
extension/Gymnasium rebuild. Checks are source-grounded/reference-informed,
**not original-runtime validated**.
Final unchanged-reviewed public/internal/shared checks wall0.034/0.021/0.013s;
build0.02/0.00/0.00s, tests0.00/0.01/0.00s. No diagnostic threshold exceeded.

Normal Neow consequences, map/room choices, rewards/acquisition/economy remain
M4/later; no general checkpoint restoration, complete card/item/encounter pool,
other-character bridge or ascension bridge acceptance. Final scope stays all four
characters/A0–20/Acts1–4/Heart. Existing standalone starts/combat functionality
is preserved, not reclassified as implemented normal-run progression.
