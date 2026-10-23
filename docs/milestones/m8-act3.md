# M8 — Act3 mechanics and captured prefix through chest43

**Accepted at the declared M8 scope after ONE independent milestone review.**
Ironclad/A0, source-grounded native mechanics and explicitly captured room content.
Not full Act3 completion, standard random pools, other characters/Ascensions,
Act4/Heart or original-runtime validation. The final four-character/A0–20/Acts1–4/
Heart target is unchanged. [Approved plan](../superpowers/plans/2026-10-23-m8-act3-mechanics.md).

## Run it

```sh
cargo run -p spire-core --example m8_act3 --locked --offline -- --seed 11 --scenario all
cargo run -p spire-core --example m8_act3 --locked --offline -- --seed 7 --scenario prefix
cargo test -p spire-core --test m8_act3 --locked --offline
cargo test -p spire-core --lib m8_ --locked --offline
```

Scenarios: `prefix`, `diagnostics`, `death`, `all`; default11/all/budget8192.
Budgets0/1, invalid arguments and native faults exit nonzero, never fabricated
completion/truncation. A real Death is valid gameplay and exits zero, labelled
**not prefix completion**. The nonattacking normal-start Death and supplied-resource
combat diagnostics are separately labelled.

## Observed normal-start outcomes — 2026-10-23

All start from80/80HP,99gold, five Strikes/four Defends/Bash, Burning Blood,
three empty potion slots, no keys. Short-Neow eligibility and valid prior boss
histories select Guardian in Act1, Automaton in Act2, and the named Act3 icon;
no resources or fight results are injected. The same Run/player continues.

| Seed / Act3 icon | Actions | Final HP | Reward | Actual outcome |
|---|---:|---:|---:|---|
|11 Awakened One|711|0|40|real Death, Act3 floor41/Giant Head|
|11 Time Eater|721|91/119|43|alive, nonterminal/no-action prefix43|
|11 Donu/Deca|738|85/119|43|same scoped boundary|
|7 Awakened One|694|0|39|real Death, Act3 floor40/Giant Head|
|7 Time Eater|685|0|39|real Death, Act3 floor40/Nemesis|
|7 Donu/Deca|689|0|39|real Death, Act3 floor40/Reptomancer|

These are the preselected seeds, not a search for easier outcomes. Natural losses
are tested/replayed and retained without tuning them away. Completed seed11 paths
include actual Orb Walker, Shapes and Growth fights, the icon-selected elite,
Falling's permanent instance removal, all-gold Tomb acquisition of Red Mask,
actual offered/played Inflame, loot and chest43 source resolution. The Act3 icon
does **not** mean its boss was fought on this prefix. Full normal-start boss50/
reward/Act3 completion remains M9. Nonattacking Death takes10 actions at floor1,
reward0 (floor credit1 minus Death1).

## Native mechanics

- Orb Walker/Burn/growth; independently sampled Shapes with Repulsor, Exploder
  fixed30/self-suicide ordering and Spiker per-hit Thorns; mixed Sphere reuses its
  existing actual handler. Hard Jaw Worm Horde remains implemented.
- Darklings retain HalfDead/Regrowing/Reincarnate ownership; only actual all-down
  defeat settles. Growth's Constricted, Transient's damage-based temporary Strength
  loss/restoration and fifth-attack-before-expiry, and Maw's selected-turn Nom.
- Writhing Mass uses exact conditional rerolls, Reactive/Malleable and permanent
  Parasite obtain. Its combat journal synchronizes into the Run once before any
  next decision or later fault; source/origin census excludes only those gains.
  A later paid purge really removes Parasite and reduces maxHP3; combat exhaust
  is not permanent removal. These are focused native ownership proofs, not a claim
  that the displayed normal-start paths acquired Parasite.
- Giant Head countdown/Slow after-use and one-floor damage arithmetic; Nemesis
  history/cooldown redirects, absent-only Intangible and per-positive-hit cap.
- Reptomancer/Daggers use actual initial/action order, independent tracked slots,
  prepared spawn custody, attack-before-self-loss and callback-delivering parent
  cleanup. A source-qualified already-prepared child can survive parent cleanup
  and keep combat live; Minion is not a blanket victory/deletion rule.
- Awakened One has real First/Rebirth/Second, typed debuff/explicit-ID removal,
  positive Strength retention, staged full heal, forced Echo and final living-Cultist
  escape. A lone zero-HP boss awaiting Rebirth remains a live decision even after
  both Cultists died. Played Powers move once into an inactive owned zone included
  in public history/exit/Run census. Inflame grants2/3 Strength; Void's actual draw
  loses1 energy/clamps0, is Ethereal, and Sludge inserts it at draw-pile bottom.
- Time Eater's exact integer/history/finite0.66f branches, strict one-time Haste
  selection below halfHP, typed cleanse and heal-to-half. Time Warp counts after
  current-card payload/selection, before disposition; counter persists and resets
  at12. An owned barrier preserves legitimate card/discard/exhaust work, then uses
  one ordinary EndPlayerTurn without a thirteenth-play opportunity. Before/after-
  barrier faults retain ownership; existing sequential terminal priority remains.
  Draw Reduction reduces size1 with application grace then duration decay.
- Deca then Donu are separate bosses with independent execution-advanced cycles,
  Artifact2 each,10×2 attacks/Dazed2, Block16 and Strength3. Buff publication snapshots
  every current group member; action-specific later target guards skip ineligible
  targets. Already-published work for a survivor remains earned after source death.
  Killing one alone does not win or reset the other.

Sixteen supplied native diagnostics run actual ordinary/lifecycle/all3elite/all3boss
bodies on each planned seed. They explicitly supply999HP,4energy,Strength20 and
an upgraded finite deck; Time Warp11 and Writhing's initial Parasite are named
supplied states. Darkling down/revival and Awakened half-death are observed. They
are **not** normal-start evidence or game-value changes to enemy bodies.

## Captured progression, events and resources

`RoomScenario::M8Act3Prefix` retains the accepted M5/M6/M7 Acts1–2 captures, actual
second boss reward/chest/removal children and once-only2→3 heal/map initialization.
M5 still stops at Act2Ready17; M6 at actual chest26 Proceed; M7 at Act3Ready34.
Only M8 publishes third-act Map at34 and admits generated edges35–43.

- First ordinary captures are Orb Walker then Three Shapes. Strong rotation is
  Growth/Transient/FourShapes/Maw/SphereAndTwoShapes/HardJaw/ThreeDarklings/Writhing,
  scanning to preserve first-strong Shapes and recent-family exclusions. Elites
  rotate Head/Nemesis/Reptomancer, offset by the public initialized icon, without
  immediate repeats. Actual HP/composition/AI/combat effects are native, not wins
  scripted by the capture. No edge is removed to hide an operational limit.
- Questions capture Falling, Tomb, then ordinary Monster, not standard event
  probabilities. Falling samples one canonical permanent instance per present
  Attack/Skill/Power type once at entry, displays Skill/Power/Attack, disables absent
  types and offers Leave if none. Duplicate kinds retain separate identities;
  only the fixed chosen instance is removed through ordinary callbacks. Curse-only
  Parasite is not removable here. EmptyDeck−1 is the declared gym settlement, not
  an asserted explicit Falling damage action. Bottled/unrepresented flags are unsupported.
- Tomb offers222gold with Red Mask; otherwise pays all actual gold, including0,
  before one real Red Mask acquisition. Preview is not obtain; result Leave
  completes the source once. Red Mask queues initial-group player-source Weak1
  at battle start, subject to ordinary per-target Artifact, not each turn or on
  later summons. Native presentation omits Falling's resource-neutral intro Continue;
  cached sampling/choice effects are unchanged by that presentation choice.
- Ordinary/elite gold remains10–20/25–35, Fire/Strength/Weak potion cycle and current
  three captured card rows; only the first Act3 ordinary row becomes
  `[Inflame,ShrugItOff,Anger]`. Actual Act3-local settlement/capture counters are
  separate from global history and earlier acts, including real Death settlement.
- Elite relics scan the existing finite eight-item sequence, skipping owned items;
  capture cursor advances on offering. Ordinary chest captures are first-unowned
  Strawberry/Anchor/BloodVial versus Sapphire; later shop is Clothesline50,
  StrengthPotion50, Anchor150 and inherited purge cost. Owned unique M8 shop stock
  is Disabled. Exhaustion is an explicit limit, never fabricated replenishment or
  victory/loss. Optional chest Proceed remains available without taking loot.
- Completion occurs only after actual visited Treasure43 source Proceed with no
  pending child: `Act3PrefixComplete`, alive/nonterminal, no actions including
  potions. Merely floor43 is insufficient; no entered44/50 or new act/clear bonus.
  No-key reward43 is real floor credit only; actual distinct keys add5, Death−1.

## Evidence and limits

Public callers use current observations, legal menus and visible map topology;
all accepted steps replay decisions/maps/reward deltas, including natural losses.
Future resolved rooms/offers, draw order, queues and actor RNG remain private.
Preflight failures are atomic; accepted faults preserve original work/source/
inventory/latest cursors and freeze, never reroll or masquerade as a game outcome.

Task-local owned-archive static findings close named mechanics, not original-game
execution or PCv2.3 build provenance. The owned archive's upstream build/channel
remains **unverified**. Original Java PRNG/seed/cursor/tie parity is not promised
(RNG-001); universal simultaneous-lethal arbitration, arbitrary autoplay and foreign
flag/callback compositions remain qualified. TIME-001 remains unchanged; no clock.
Reference datasets retain27 provisional enemy labels and12 unresolved power IDs;
their integrity checks are not product or broader mechanic proofs.

No complete standard pools, broader cards/relics/events, other-character/A0–20
acceptance, full Act3/Act4/Heart, Python run extension/batching migration or original-
runtime validation is claimed. Historical Poison/Catalyst/Artifact and Slaver/Fungi
draft failures remain explicit; the workspace is not green. Review/acceptance status
and the final fresh bounded verification matrix are recorded in the execution ledger.

The fresh independent review found no Critical/Important/Minor findings and assessed
all five required focus areas. The controller verified all462 reviewed public files
unchanged in both worktrees before fresh checks: nativeM8 111/0, publicM1–M8 56/0,
nativeM6 92/0/M7 16/0, consumers43/0, other30/0, reporting1/0, all eight examples,
both planned seeds, invalid arguments and budget0/1 behavior. Core647/7 retains
exactly the seven historical underlying failures. Reference70/0, Pythonperf22/0,
both builders' regeneration checks and diffcheck pass as their separate lanes.
No source fixes followed review; only administrative acceptance records changed.
M8 implementation/outcome are closed; M9 planning is open, not authorized product
implementation. All M8 work remains uncommitted and worktrees/evidence preserved.
