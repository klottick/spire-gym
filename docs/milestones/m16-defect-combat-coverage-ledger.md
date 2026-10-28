# M16 proposed Defect combat coverage ledger

Planning only, 2026-10-27. Baseline `e5b07da5609a3a2654b3bced3f8388ea96324a8f`.
Read with the [design](../superpowers/specs/2026-10-27-m16-defect-combat-design.md)
and [plan](../superpowers/plans/2026-10-27-m16-defect-combat.md).
No future test below has run. Native implementation facts, local static rules,
community references and original-runtime validation remain distinct evidence.

## Finite roster and closure

| Exact M16 kinds | Baseline | Proposed proof / owner |
|---|---|---|
| StrikeBlue, DefendBlue, Zap, Dualcast, BallLightning, ColdSnap, Coolheaded, Barrage, CompileDriver | Actual native payloads; existing Barrage sparse/>3 tests | Preserve base/upgraded rules; four-kind CompileDriver; front/Dualcast; Tasks1/5 |
| Darkness, Fusion, Capacitor, Consume, Defragment, Recursion, Fission | Missing native identities/producers | Dark/Plasma, capacity0..10, Focus without orb, stored-D transfer, remove versus evoke; Task1 |
| BiasedCognition, Loop, Electrodynamics, Bullseye, StaticDischarge | Missing native producers/callbacks | Artifact, phase order, target mode/duration and onAttacked stage; Task2 |
| Storm, Heatsinks, Amplify, EchoForm | Missing native blue producers | Real notified plays, same-family purge repeats and expiry; Task3 |
| GeneticAlgorithm, Claw, SteamBarrier, Streamline | Missing native producers/state | Permanent misc versus combat wear/cost/identity; Task4 |
| CrackedCore | Real combat-start channel | Canonical entry exactly once; Tasks2/5 |
| DataDisk, GoldPlatedCables, EmotionChip | Reference rows, no accepted native mechanics | Empty-slot Focus, precise passive groups, HP-loss flag/reset; Task2 |
| FocusPotion, PotionOfCapacity (proposed native names) | No accepted native producers | Real use, Bark potency, inventory/fault custody; Task2 |

The card rows total29. Explicit fresh/generated, stat-equivalent and same-family
construction are admitted for these same kinds; generated identity does not mean
random generation. Every declared kind must have cost, targeting, upgrade,
visible values, legal play, actual effects, disposition and copy state. No-op or
metadata-only rows prevent M16 acceptance. Shared existing items/cards used as
interaction fixtures are not newly claimed complete pools. Off-roster existing
blue behavior is preserved. M17–20 acquisition/captured offers are separate gates.

## Qualified source checkpoints

The [paired Defect research](../research/2026-10-19-act1-content-defect.md)
sections5–7 supplies orb values, local target/copy/mutation rules, integer domains
and explicit composition residuals. Additional named static inspection uses owned
archive SHA-256 `cfad868ac8d65a88e71a0bf096fb09f78811e553effe0787c5309a655e081673`
and pinned javap21.0.12.1. Archive identity does not establish build/channel or
original-runtime parity. Raw source/bytecode and exact command/output hashes stay
in retained private evidence; this table is independently worded.

| Named source members | Established local checkpoint |
|---|---|
| `GameActionManager.getNextAction`; `AbstractPlayer.applyStartOfTurnRelics/PreDrawCards/Cards/Orbs`; `AbstractCreature.applyStartOfTurnPowers` | At normal turn transition, invoke relics, pre-draw cards, cards, powers, then natural start-orb hooks; ordinary draw is queued afterward. Invocation order and later queued work are distinct. |
| `AbstractPlayer.applyStartOfTurnOrbs`; `TriggerEndOfTurnOrbsAction.update` | Iterate slot order, then Cables adds front index0's same-phase hook if nonempty. It is not a scan for the last/first occupied slot. |
| `DarkImpulseAction.update` | Darkness+ invokes start+end only on Dark slots, then once more on front index0 only if that front is Dark and Cables is owned. |
| `LoopPower.atStartOfTurn` | Invoke front start+end hooks for each intensity; no local Cables check and no removal. |
| `EmotionChip.atTurnStart/wasHPLost/onVictory`; `ImpulseAction.update` | Positive combat HP-loss notification arms Chip; start consumes flag and queues Impulse at bottom. Impulse invokes each orb start+end, then nonempty front again with Cables. Victory clears flag. |
| `BiasPower.atStartOfTurn`; `Plasma.onStartOfTurn/onEvoke` | Bias queues negative Focus at bottom; Plasma queues passive energy at bottom, evoke energy at top. Do not replace all phases with immediate resource mutation. |
| `StaticDischargePower.onAttacked`; `AbstractPlayer.damage` | Positive nonself-owner non-THORNS/non-HP_LOSS onAttacked input queues individual Lightning at top. Player onAttacked powers precede later loss-reducer callbacks and positive wasHPLost notifications; Chip and Discharge cannot share an HP-lost predicate. |
| `DataDisk.atBattleStart`; `FocusPotion/PotionOfCapacity.getPotency/use`; `AbstractPotion.getPotency` | DataDisk requests Focus1; both potions have base potency2, Sacred Bark doubles potency; capacity potion routes through repeated one-slot increases. |
| `GeneticAlgorithm.use`; `IncreaseMiscAction.update` | Captured old Block and learned misc are separate; identity-matched master and deduplicated combat objects grow, not all same-name cards and not only winners. |

The remaining exact ordering frontier is the ordinary subsequent-turn caller of
`EnergyManager.recharge`. Known start-hook order does not establish recharge
relative to Plasma, powers and draw. Initial combat has a separate queued-energy
path and cannot substitute for that proof. Task1 must resolve this named caller
before freezing the normal-turn Plasma ordering test. Later defensive conversion
of wrapped signed Dark payloads and reachability of numeric extremes also retain
the paired report’s stated limits; no complete overflow-history claim is made.

Freeze the exact static source-fact report alongside the plan. These narrow
caller/conversion boundaries are explicit execution checkpoints, not
permission to guess ordering, scan the whole archive or claim transitive source
closure. The new native tests prove the implemented composition; source agreement
alone is not an executed gameplay result.

## Integration ownership

Current context after the independent planning review: M15 v3 design/plan is
user-approved and execution has started, while its shared generator/cost/history/
repeat source/API remains unaccepted. M16 reuses actual accepted owners before
adding blue callbacks; no random generator is required by this finite roster.
Task68 PR53 is merged to main and accepted at
`d1fa706638a31a854fdcaeed9a9a4642abf716eb`; retained read-only source export:
`/home/klottick/.local/share/spire-gym/accepted/encounters-d1fa706638a31a854fdcaeed9a9a4642abf716eb`.
Root reports native2043/0 for that source; this task ran no runtime check.
Reconcile its exact accepted source before integration. The planning read baseline
remains e5b07da, and the original frozen evidence is unchanged. M17 is unassigned
and supplies no accepted API. M18 design/plan is now user-approved with the same29
card admission set and `M18DefectAct2` captured producers; its execution still
requires accepted M16/M17 and root release. Those offers do not enlarge M16,
establish M17 acceptance or imply M16 user approval.
