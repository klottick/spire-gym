# M16 finite native Defect authored combat

The manager accepted the independently reviewed finite native source for
publication preparation, based on accepted M15
`4d7e5be44fd1ce0167729ef1a4f8c9b17b9c5bdf`. The same reviewer’s correction
recheck is READY; I1/M1/M2 are closed with no additional check required. Exact
reviewed source manifest:
`ae297201e0344182d9e4a322d4ae3093dd7443fac921c23d77af486a590bcd12`.
Manager publication/main integration remains pending; no M16 merged SHA is claimed.
The [public review record](../superpowers/plans/2026-10-28-m16-native-runtime-review.md)
records acceptance and the unchanged evidence qualifications.

This profile delivers finite A0 authored combat through the shared native Rust
engine. It makes no normal Run progression, complete Defect pool, Python artifact,
original-game execution or upstream-channel fidelity claim. Normal Defect
progression remains M17 onward; the full blue pool remains M20.

The [approved design](../superpowers/specs/2026-10-27-m16-defect-combat-design.md),
[plan](../superpowers/plans/2026-10-27-m16-defect-combat.md) and
[planning coverage ledger](m16-defect-combat-coverage-ledger.md) retain their
original approval bytes. The [execution reconciliation](m16-defect-combat-reconciliation.md)
records actual accepted interfaces, factual corrections and evidence boundaries.

## Native entry and finite content

`Run::from_m16_defect_combat(M16DefectCombatStart::{Canonical, Supplied(AuthoredRunStart)}, rng)`
validates resources before transferring them to the existing Run owner.
Canonical resources are 75/75 HP, 99 gold, four StrikeBlue, four DefendBlue,
Zap, Dualcast, CrackedCore, three empty potion slots and no keys. Supplied starts
are explicitly diagnostic resources. Both enter through `Run::enter_combat`,
shared `enter_owned`, encounter setup, item context, resolver and settlement.
No supplied orb layout or state-reset API is added.

The exact card roster is:

| Family | Card kinds, base and upgraded |
| --- | --- |
| Existing blue combat | StrikeBlue, DefendBlue, Zap, Dualcast, BallLightning, ColdSnap, Coolheaded, Barrage, CompileDriver |
| Orb and Focus producers | Darkness, Fusion, Capacitor, Consume, Defragment, BiasedCognition, Loop, Electrodynamics, Bullseye, Recursion, Fission, StaticDischarge |
| Power callbacks | Storm, Heatsinks, Amplify, EchoForm |
| Mutable cards | GeneticAlgorithm, Claw, SteamBarrier, Streamline |

Explicit generated, stat-copy and same-family construction uses these same 29
kinds. This admission list creates no random generation, reward or shop pool.
The four Defect relic witnesses are CrackedCore, DataDisk, GoldPlatedCables and
EmotionChip. FocusPotion is Common; PotionOfCapacity is Uncommon. Both add two
Focus/slots respectively, or four with SacredBark. Existing shared native items
remain subject to their existing inventory validation and character exclusions.

The new constructor admits Act1 entries from the first 57 rows of the shared
`A0_ENCOUNTER_SETUPS` at the accepted baseline. The prefix is explicit in
`M16_DEFECT_COMBAT_SETUP_PREFIX_LEN` and its baseline identifier; future table
appends do not widen M16. Act and Monster/Elite/Boss kind come from that owner.
The twenty admitted setups are Cultist, JawWorm, TwoLouse, SmallSlimes,
LargeSlime, LotsOfSlimes, SlimeBoss, GremlinNob, Lagavulin, ThreeSentries,
ThreeLouse, Guardian, Hexaghost, BlueSlaver, TwoFungiBeasts, RedSlaver, Looter,
GremlinGang, ExordiumThugs and ExordiumWildlife. Unknown IDs, Act2–4 entries and
non-A0 context reject. Lagavulin uses actual native sleep, sampled HP, RNG and AI.
Existing authored/default/Gym profiles retain their admission boundaries.

## State, actions and observations

One owned slot vector represents capacity zero through ten and all four orb
kinds. A Dark object carries its own signed stored damage. Front evoke means
index zero; Barrage counts occupied slots, including sparse capacity above three.
Focus has one signed owner, clamped to [-999,999], including empty/zero-slot
states. Plasma ignores Focus; ordinary recharge follows the qualified hook phase
before queued energy and draw work resolve. Loop, Cables, Darkness+ and Chip use
their separate source-qualified invocation sites.

Amplify/Echo extend the accepted autoplay scheduler, copy identities and finish
frames. Real Power cards retire once; purge children retain ordinary use hooks.
Faults retain accepted payment, current work and owned children. Terminal cleanup
cancels future work while retaining physical owners for Run exit and settlement.

GA captures old Block before learning. Shared Run action-prefix synchronization
applies absolute misc values with contiguous ordinals, including before returning
a later fault and on Death. Copies use the existing family/provenance owner;
fresh copies cannot learn for an unrelated permanent card. Claw, SteamBarrier
and Streamline retain their combat-local values and reset on the next entry.
Steam raw Block can be negative before Dexterity and the final calculation floor.
Dark/misc/Claw/Steam mutations use signed int32 behavior without a fabricated999
field cap; Lock-On uses binary32 multiplication and saturating integer narrowing.
These numeric checkpoints do not establish universal overflow-history reachability.

In `spire-env`, use `defect_combat_observation_spec(&caps)` and
`encode_defect_combat(&decision, &caps)` for the opt-in
`spire-gym-m16-defect-combat-observation-v1` / `M16DefectCombat` schema.
The shared Run encoder adds permanent/combat misc and orb kind/Dark payload.
Signed values travel as native int64 with signed-int32 descriptor bounds.
Ten logical orb rows are the default; capacity nine on a real ten-slot state
returns an explicit required10/capacity9 error. Empty slots, padding and unknown
Dark fields differ. Draw order, RNG keys, future enemy choices, family IDs and
mutation ordinals remain private. Candidate order and existing action row
references are preserved exactly; action capacity and mask length equal
`caps.actions`, with one mask entry per original candidate.

This adds no session token, dispatch format, batch API, copied loop or Python
facade. The existing default descriptors and policy sidecar remain frozen.

## Executable and test evidence

Run with the provisioned locked/offline native toolchain:

```sh
CARGO_BUILD_JOBS=2 cargo run -p spire-core --locked --offline --example m16_defect_combat -- --seed 7 --scenario all --budget 8192
CARGO_BUILD_JOBS=2 cargo test -p spire-core --lib --locked --offline m16_ -- --test-threads=1
CARGO_BUILD_JOBS=2 cargo test -p spire-env --locked --offline --test m16_defect_observation -- --test-threads=1
CARGO_BUILD_JOBS=2 cargo test --workspace --locked --offline -- --test-threads=1
```

Execution used a dedicated retained target/tmp, jobs2 and test-threads1. Raw logs,
commands, preimages, source/mode manifests and the exact candidate are retained
outside managed sessions in the execution review packet. Rust/cargo1.99.0 was
already provisioned; no installation or Python-extension feature/build/import
was performed.

**Pre-correction evidence:** the following fixed results belong to initial
source manifest `f2fcc5dd0b06b8c8a1bbd08914805733300eeb4cb2691ef4c93e7c7f2fbe5e2e`.
The initial workspace result was 2,421 passed, zero failed and two existing held
tests ignored. That matrix and the full example series were not rerun during
the bounded correction pass. They establish no corrected-source whole-matrix
or example-series result.

The fixed seed7 scenario table was frozen before its first execution. Every
semantic action replayed through an identical cloned shared Run, comparing the
transition, complete Run and public decision. All cases completed on that first
execution:

| Label | Actions | Actual result | Witness |
| --- | ---: | --- | --- |
| starter-canonical-Cultist | 12 | Victory, HP74 | Canonical resources and real combat |
| supplied-orbs-Lagavulin | 20 | Victory, HP45 | Card-produced capacity10; all four kinds; occupied-four Barrage |
| supplied-callbacks-Lagavulin | 16 | Victory, HP39 | Storm, Heatsinks, Echo, Amplify; Focus4 and Lightning |
| supplied-genetic-two-fights-Cultist, fight1 | 8 | Victory, HP75 | Permanent misc1→4 |
| same supplied Run, fight2 | 8 | Victory, HP75 | Permanent misc4→7; local Claw reset |
| supplied-death-control-Cultist | 2 | Death, HP0 | Actual Run Death terminal |

These supplied resources were not earned through progression. This one canonical
victory is a fixture result, not a policy-strength or general win guarantee.
No randomized policy trials, seed/resource search or enemy/outcome substitution
were used. Interior sparse slots are separate real shared-resolver unit evidence:
`defect_barrage_repeats_one_captured_hit_per_occupied_orb` retains the five-of-ten
and ten-occupied cases and per-hit damage/hooks; M16 front-evoke and Fission tests
exercise sparse/terminal continuations. The constructor starts empty and the
supported card producer paths preserve density, so the public example does not
claim to produce an interior hole.

Focused coverage includes all29 base/upgraded uses; orb timing/numeric/target
boundaries; actual repeat, Stasis, fault and Death custody; forged journal and
terminal ownership rejection; two-fight local reset; and canonical/supplied
encoder agreement. Existing autoplay32, Shield/Spear26, full-card74,
M15 cost/copy22, potion16, Run settlement13 and upgrade3 checks passed on this
initial pre-correction candidate. Six existing env schema/policy targets passed65
checks, including
predecessor descriptor/fingerprint sentinels. These scoped results are distinct
from the bounded correction verification recorded with the new candidate receipt.

The correction routes Heatsinks’ earned Draw to the front of the shared queue;
Storm retains bottom insertion. Actual callback draws precede the original Power
payload and both Echo/Amplify purge children. Listener capture, payment, Power
retirement and purge ownership remain shared. The runtime validator now accepts
positive signed-int32 Heatsinks/Bias intensities consistently with their retained
stack rules; Focus retains its separate [-999,999] clamp. Constructed998→1000
Heatsinks and999→1000 Bias fixtures exercise real upgraded cards, without claiming
a public history reaching those prestates. The diagnostic snapshot comparator
now includes Defect counters/journal after all existing comparison labels.

Corrected-source focused verification passed189 test instances with zero failures:
M16 core49, shared autoplay32, Silent draw/history8, entry/replay/snapshot21,
full-runtime validator27, potion actions16, and native M16/M10/M15 observation
checks4/21/11. These bounded affected checks support the same-reviewer READY
recheck and manager source acceptance. They are not a new workspace matrix,
example series or public high-stack reachability proof.

Source qualifications remain static named-member inspection of the authorized
archive and accepted native source. They are not original-runtime validation,
an authenticated distribution claim or a complete callback/animation total order.
