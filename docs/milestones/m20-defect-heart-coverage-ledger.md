# M20 Defect A0 coverage ledger

2026-10-28 planning candidate; no implementation, tests or episodes executed.
Read the [design](../superpowers/specs/2026-10-28-m20-defect-heart-design.md) and
[plan](../superpowers/plans/2026-10-28-m20-defect-heart.md). Checkboxes remain
open until exact accepted implementation and evidence are attached. A constructor,
a catalog entry and a favorable build are different from an executable rule.

## Evidence and interpretation

Read baseline is accepted M15 merge `4d7e5be44fd1ce0167729ef1a4f8c9b17b9c5bdf`,
765 files, retained manifest SHA-256
`0b7475fd1ffdec7c5919f532961d8ec37995603e1e1cad2a37024afcb268f6c0`.
All ten primary-approved M16–M19 inputs, including the authoritative M19 routing
correction, are hash-verified in the external planning packet. Their historic
pending labels do not override current approval; their runtime APIs remain
unaccepted. Task0 must bind the actual accepted implementations before M20 starts.

Roster evidence: [paired blue records](../research/2026-10-19-act1-content-defect.md)
§§2–8 and [card reference](../../data/cards/README.md). The named static
`CardLibrary.addBlueCards()` registration contains 75 distinct constructors;
Defect's `getCardPool` calls the nonbasic blue admission method. Reference type
and rarity partitions give 4 basics +18 common +36 uncommon +17 rare =75.
There are 24 Attacks, 37 Skills and 14 Powers including basics; ordinary
nonbasic counts are 23/34/14. Self Repair is the sole blue HEALING exclusion in
the combat generator, yielding 23/34/13 =70. The common source has 18 kinds.
The Ironclad-context generator examples in the old research are not Defect pools.

[Relic references](../../data/relics/README.md) give nine Defect-exclusive kinds.
Named local checks of the five additions and `PotionHelper.getPotions(DEFECT,false)`
qualify their callbacks and all three exclusive potion identities. All static
facts use owned archive `cfad868ac8d65a88e71a0bf096fb09f78811e553effe0787c5309a655e081673`
and provisioned javap21.0.12.1; upstream build/channel remains unverified. Raw
outputs stay private. These facts and existing research are source-grounded
inputs, not original-runtime validation or transitive native proof.

Every blue row below has the same completion obligation: base/+ definition,
starting price versus current legality, target/flags, payload, visible values,
upgrade and copy reconstruction, correct finish/cancellation, and permanent
obtain/upgrade/remove/next-entry behavior. **Publication owner P** means Task8:
`spire-env/src/run_schema_m20.rs` vocabulary plus the shared `run_observation.rs`
encoder, tested by `tests/run_m20_schema.rs`. **Acquisition owner A** means Task5:
`run/rooms/m20_content.rs` on shared rewards/shop/mutations. No per-card encoder,
factory, loop or journal is introduced.

Baseline labels: **B** = one of the 13 actual native M15 blue payload kinds
(public values and full profile admission still need checking); **D** = missing
from those 13 at the read baseline. **M16** = required approved 29-kind predecessor
capability, not a claim of current acceptance. **T1/T2/T3** identify the concrete
M20 implementation/test owner in the plan. Every row is publication P/acquisition A.
Tasks6/8/9 additionally test composition, transport and retained evidence.

## Every blue kind

| Kind (native spelling proposed where absent) | Rarity/type | Read baseline / dependency | Implementation and discriminating test obligation | Done/evidence |
|---|---|---|---|---|
| DefendBlue | Basic/S | B / M16 | Task0 bind M16; Task6 preserve: 5/8 card Block; distinct blue basic; P/A | [ ] |
| Dualcast | Basic/S | B / M16 | Task0 bind M16; Task6 preserve: front twice; remove only on final evoke; P/A | [ ] |
| StrikeBlue | Basic/A | B / M16 | Task0 bind M16; Task6 preserve: 6/9 normal damage; distinct blue basic; P/A | [ ] |
| Zap | Basic/S | B / M16 | Task0 bind M16; Task6 preserve: 1/0 cost; one successful Lightning channel; P/A | [ ] |
| BallLightning | Common/A | B / M16 | Task0 bind M16; Task6 preserve: 7/10 then Lightning; terminal pruning; P/A | [ ] |
| Barrage | Common/A | B / M16 | Task0 bind M16; Task6 preserve: occupied count at continuation; sparse cap10 and >3 hits; P/A | [ ] |
| BeamCell | Common/A | B / M20 | T1: 3/4 damage then Vulnerable1/2; retain actual handler; P/A | [ ] |
| ChargeBattery | Common/S | D / M20 | T1: Block7/10 then next-recharge energy1; stacked constructor/application; P/A | [ ] |
| Claw | Common/A | D / M16 | Task0 bind M16; Task6 preserve: 3/5 then local growth2; fresh generated copy resets; P/A | [ ] |
| ColdSnap | Common/A | B / M16 | Task0 bind M16; Task6 preserve: 6/9 then Frost; P/A | [ ] |
| CompileDriver | Common/A | B / M16 | Task0 bind M16; Task6 preserve: distinct four kinds; two Dark payloads count once; P/A | [ ] |
| Coolheaded | Common/S | B / M16 | Task0 bind M16; Task6 preserve: Frost then actual draw1/2; P/A | [ ] |
| GoForTheEyes | Common/A | B / M20 | T1: 3/4 then Weak1/2 iff source intent base>=0; hidden intent stays private; P/A | [ ] |
| Hologram | Common/S | D / M20 | T2: Block3/5; discard selection; full hand leaves source; base Exhaust only; P/A | [ ] |
| Leap | Common/S | B / M20 | T1: card Block9/12; retain actual handler; P/A | [ ] |
| Rebound | Common/A | D / M20 | T2: damage9/12; just-installed after-use guard; exhaust/purge/Power precedence; P/A | [ ] |
| Recursion | Common/S | D / M16 | Task0 bind M16; Task6 preserve: cost1/0; same Dark object survives; P/A | [ ] |
| Stack | Common/S | D / M20 | T1: capture discard size+0/3 at calculation, not delayed recount; P/A | [ ] |
| SteamBarrier | Common/S | D / M16 | Task0 bind M16; Task6 preserve: stored6/8 wears below0; upgrade adds2 to worn state; P/A | [ ] |
| Streamline | Common/A | D / M16 | Task0 bind M16; Task6 preserve: 15/20; family combat-cost reduction with turn-zero branch; P/A | [ ] |
| SweepingBeam | Common/A | B / M20 | T1: AoE6/9 then draw1; retain actual handler; P/A | [ ] |
| Turbo | Common/S | D / M20 | T1: energy2/3 then one base Void in discard; insertion is not draw; P/A | [ ] |
| Aggregate | Uncommon/S | D / M20 | T1: execution-time floor(draw size/4 or /3); P/A | [ ] |
| AutoShields | Uncommon/S | D / M20 | T1: use-time Block==0 gates calculated11/15; P/A | [ ] |
| Blizzard | Uncommon/A | D / M20 | T1: successful Frost channel history x2/3; no passive/failed channel count; P/A | [ ] |
| BootSequence | Uncommon/S | D / M20 | T1: Innate/Exhaust Block10/13; P/A | [ ] |
| Bullseye | Uncommon/A | D / M16 | Task0 bind M16; Task6 preserve: 8/11 then Lock-On duration2/3; Artifact and binary32; P/A | [ ] |
| Capacitor | Uncommon/P | D / M16 | Task0 bind M16; Task6 preserve: 2/3 empty slots, cap10; P/A | [ ] |
| Chaos | Uncommon/S | D / M20 | T1: 1/2 independent four-way samples at use, including at zero slots; P/A | [ ] |
| Chill | Uncommon/S | D / M20 | T1: count not-dead-or-escaped at use; one Frost each; Exhaust, +Innate; P/A | [ ] |
| Consume | Uncommon/S | D / M16 | Task0 bind M16; Task6 preserve: Focus2/3 then remove last slot without evoke, even at zero; P/A | [ ] |
| Darkness | Uncommon/S | D / M16 | Task0 bind M16; Task6 preserve: Dark6; + invokes existing Dark passives; P/A | [ ] |
| Defragment | Uncommon/P | D / M16 | Task0 bind M16; Task6 preserve: Focus1/2; empty-slot/Artifact composition; P/A | [ ] |
| DoomAndGloom | Uncommon/A | D / M20 | T1: AoE10/14 then Dark; P/A | [ ] |
| DoubleEnergy | Uncommon/S | D / M20 | T1: cost1/0, Exhaust; double then-current postpayment energy, add cap999; P/A | [ ] |
| Equilibrium | Uncommon/S | D / M20 | T2: Block13/16, temporary retain on non-Ethereal hand; proper end-round expiry; P/A | [ ] |
| ForceField | Uncommon/S | D / M20 | T2: cost4 initialized from combat Power history and notified live plays; Block12/16; P/A | [ ] |
| FTL | Uncommon/A | D / M20 | T2: 5/6; history-minus-current <3/4 at follow-up, top draw before bottom damage; P/A | [ ] |
| Fusion | Uncommon/S | D / M16 | Task0 bind M16; Task6 preserve: Plasma; cost2/1; correct start energy; P/A | [ ] |
| GeneticAlgorithm | Uncommon/S | D / M16 | Task0 bind M16; Task6 preserve: misc1; captured old Block then persistent growth2/3 on accepted actions; P/A | [ ] |
| Glacier | Uncommon/S | D / M20 | T1: card Block7/10 then two separate Frost channels; P/A | [ ] |
| Heatsinks | Uncommon/P | D / M16 | Task0 bind M16; Task6 preserve: notified Power draw1/2; nested draw history; P/A | [ ] |
| HelloWorld | Uncommon/P | D / M20 | T3: start common18; actual Defect; guard basically-dead; +Innate; P/A | [ ] |
| Loop | Uncommon/P | D / M16 | Task0 bind M16; Task6 preserve: front start+end passives1/2; no global Cables doubling; P/A | [ ] |
| Melter | Uncommon/A | D / M20 | T1: remove entire target Block then normal10/14; removal is not damage; P/A | [ ] |
| Overclock | Uncommon/S | D / M20 | T1: actual draw2/3 then base Burn in discard even when upgraded; P/A | [ ] |
| Recycle | Uncommon/S | D / M20 | T2: mandatory hand exhaust; selected turn cost/X energy captured before exhaust hooks; P/A | [ ] |
| ReinforcedBody | Uncommon/S | D / M20 | T2: captured X + ChemicalX2 separate Block7/9 gains; late payment; P/A | [ ] |
| Reprogram | Uncommon/S | D / M20 | T1: Focus-1/-2, Strength1/2, Dexterity1/2 in order; Artifact branches; P/A | [ ] |
| RipAndTear | Uncommon/A | D / M20 | T1: two separately targeted/recalculated random hits7/9; P/A | [ ] |
| Scrape | Uncommon/A | D / M20 | T2: 7/10 then draw4/5; shared drawn-history filter turn-zero OR free-once; P/A | [ ] |
| SelfRepair | Uncommon/P | D / M20 | T3: Repair7/10; positive-HP battle-end healing; excluded from combat generators; P/A | [ ] |
| Skim | Uncommon/S | D / M20 | T1: actual draw3/4; no false generation callback; P/A | [ ] |
| StaticDischarge | Uncommon/P | D / M16 | Task0 bind M16; Task6 preserve: earlier qualifying onAttacked, not final HP loss; P/A | [ ] |
| Storm | Uncommon/P | D / M16 | Task0 bind M16; Task6 preserve: actual Power notifications; upgraded Innate; P/A | [ ] |
| Sunder | Uncommon/A | D / M20 | T1: 24/32 then energy3 by source dying/HP predicate; HalfDead/lifecycle; P/A | [ ] |
| Tempest | Uncommon/S | D / M20 | T2: X+0/1(+ChemicalX2) separate Lightning; Exhaust and action-owned payment; P/A | [ ] |
| WhiteNoise | Uncommon/S | D / M20 | T3: cost1/0 Exhaust; use-time nonhealing Power13; turn-zero before creation upgrade; P/A | [ ] |
| AllForOne | Rare/A | D / M20 | T2: 10/14 then execution-time discard cost==0 OR free-once retrieval; recheck source; P/A | [ ] |
| Amplify | Rare/S | D / M16 | Task0 bind M16; Task6 preserve: nonpurge Power repeats1/2; expiry and finish; P/A | [ ] |
| BiasedCognition | Rare/P | D / M16 | Task0 bind M16; Task6 preserve: Focus4/5 then Bias1; constructor/application distinct; P/A | [ ] |
| Buffer | Rare/P | D / M20 | T3: constructor/application1/2; positive damage-change prevention and reduction custody; P/A | [ ] |
| CoreSurge | Rare/A | D / M20 | T1: 11/15 then Artifact1, Exhaust; P/A | [ ] |
| CreativeAI | Rare/P | D / M20 | T3: cost3/2; start Power13 including itself; no Hello guard or free discount; P/A | [ ] |
| EchoForm | Rare/P | D / M16 | Task0 bind M16; Task6 preserve: turn-position guard; base Ethereal; purge repeat ownership; P/A | [ ] |
| Electrodynamics | Rare/P | D / M16 | Task0 bind M16; Task6 preserve: Electro then Lightning2/3; no multiplied damage; P/A | [ ] |
| Fission | Rare/S | D / M16 | Task0 bind M16; Task6 preserve: occupied n; remove vs evoke; captured energy/draw and lethal interruption; P/A | [ ] |
| Hyperbeam | Rare/A | D / M20 | T1: AoE26/34 then Focus-3; queued order; P/A | [ ] |
| MachineLearning | Rare/P | D / M20 | T3: master draw+1, not immediate draw or hand-cap increase; +Innate; P/A | [ ] |
| MeteorStrike | Rare/A | D / M20 | T1: cost5, damage24/30 then three Plasma; STRIKE tag; P/A | [ ] |
| MultiCast | Rare/S | D / M20 | T2: captured X+0/1(+ChemicalX2); current front per invoke, no orb/no count means no payment; P/A | [ ] |
| Rainbow | Rare/S | D / M20 | T1: ordered Lightning/Frost/Dark; base Exhaust, +cycles; P/A | [ ] |
| Reboot | Rare/S | D / M20 | T2: hand/discard transfer+shuffle then draw4/6; no exhaust/Power resurrection; P/A | [ ] |
| Seek | Rare/S | D / M20 | T2: draw selection1/2; sorted knowledge, selected order, full hand to discard; no draw hook; P/A | [ ] |
| ThunderStrike | Rare/A | D / M20 | T1: successful Lightning history, separate random7/9 hits; STRIKE tag; P/A | [ ] |

T1/T2 use `combat/cards/defect.rs` (or its accepted focused submodules),
`card_state.rs`, existing selections/piles/history/turns; tests are
`combat/tests/m20_defect_direct.rs` and `m20_defect_state.rs`.
T3 uses those owners plus `combat/generation.rs`, powers/hooks and tests
`combat/tests/m20_defect_generation.rs` / `m20_defect_powers.rs`.
M16 mechanics stay in their actual accepted owner. Task6 tests them in new
compositions; missing M16 delivery is a prerequisite, not work to clone here.

## Exclusive items: all nine relics and three potions

All rows have Task4 implementation in shared `items/{mod,relics,potions}.rs`
and accepted M16 item module, Task5 acquisition, Task8 publication P, and tests
`items/tests/m20_defect.rs` / `run/tests/m20_defect_content.rs`.

| Kind / native identity proposal | Predecessor versus addition | Real rule and acquisition/custody test | Done/evidence |
|---|---|---|---|
| CrackedCore | M16, baseline native | Canonical starter; one Lightning at accepted opening phase; FrozenCore replacement removes it once | [ ] |
| DataDisk | M16 | Common; battle-start Focus1, including no occupied slots; normal finite stock | [ ] |
| GoldPlatedCables | M16 | Uncommon; extra front index0 passive only at qualified groups; normal stock | [ ] |
| EmotionChip | M16 | Rare; positive HP-loss arm/start consumption/reset, distinct from Discharge; normal stock | [ ] |
| SymbioticVirus | M20 | Uncommon; pre-battle Dark6 channel, not a free evoke; normal stock | [ ] |
| FrozenCore | M20 | Boss; requires/replaces CrackedCore; player-end hook channels one Frost iff any empty slot, not every empty slot; no Neow swap | [ ] |
| Inserter | M20 | Boss; equip counter0; each turn increments (legacy -1→1); at2 reset0 and queue +1 slot. Persistent across fights, cap10 does not suppress the counter reset | [ ] |
| NuclearBattery | M20 | Boss; pre-battle channel one Plasma, does not replace CrackedCore or grant energy directly | [ ] |
| RunicCapacitor | M20 | Shop; pre-battle resets first-turn flag, first atTurnStart queues +3 empty slots then clears flag. Re-entry repeats once, later turns do not | [ ] |
| FocusPotion | M16 | Common potency2 Focus; Bark4; live use and inventory consumption once | [ ] |
| PotionOfCapacity | M16 | Uncommon potency2 slot increases; Bark4, cap10; inventory consumption once | [ ] |
| EssenceOfDarkness | M20 | Rare potency1; action traverses current slot capacity and channels potency Dark per slot; Bark2. Zero slots none; occupied/full/sparse slots cause real evictions, not direct vector replacement | [ ] |

The first-turn timing of RunicCapacitor and pre-battle channels must be joined
against the accepted opening/turn scheduler, not flattened into starter capacity6.
FrozenCore's source-local empty-slot rule does not establish its ordering against
Equilibrium, Burn, passive Frost and Cables; named caller closure F4 below owns it.

## Generated and shared closure required before enabling M20

| Exact kind set / producer | Implementation and publication/test owner | Source/fault boundary | Done/evidence |
|---|---|---|---|
| Hello common18 = every Common row above | Task3 `generation::pool` adds actual-character Common policy; P; `m20_defect_generators_complete_support_and_capture` | No healing/type filter or rarity roll; source-current common pool; no removal; Hello guard distinct from CreativeAI | [ ] |
| Power13 = Capacitor, Defragment, Heatsinks, HelloWorld, Loop, StaticDischarge, Storm, BiasedCognition, Buffer, CreativeAI, EchoForm, Electrodynamics, MachineLearning | Task3 shared pool/capture/delivery; P; same support test | WhiteNoise use capture/turn-zero vs CreativeAI start capture/ordinary cost; self-recursion admitted, SelfRepair excluded | [ ] |
| Attack23 = nonbasic Attack rows above | Task3 shared typed offers/Metamorphosis; P; `m20_defect_generated_templates_and_cost_layers` | Full domain; no basics, type/color substitution or rarity lottery | [ ] |
| Skill34 = nonbasic Skill rows above | Task3 shared typed offers/Chrysalis; P; same test | Includes Seek, Reboot, WhiteNoise, GA and all three X kinds; no convenient subset | [ ] |
| Any70 = Attack23 +Skill34 +Power13 | Task3 Discovery actual-character selector; P; `m20_defect_nested_offers_keep_parent_and_delivery_cursor` | Three distinct IDs, mandatory untyped offer; typed potion offers allow Skip; Bark copies independently issued; X retains sentinel | [ ] |
| L34 = Apotheosis, Blind, Chrysalis, DarkShackles, DeepBreath, Discovery, DramaticEntrance, Enlightenment, Finesse, FlashOfSteel, Forethought, GoodInstincts, HandOfGreed, Impatience, JackOfAllTrades, Madness, Magnetism, MasterOfStrategy, Mayhem, Metamorphosis, MindBlast, Panacea, Panache, PanicButton, Purity, SadisticNature, SecretTechnique, SecretWeapon, SwiftStrike, TheBomb, ThinkingAhead, Transmutation, Trip, Violence | Existing accepted M15 colorless/`generation.rs` owners extended for actual Defect, Task3; P; generation/power/selection tests | Every identity, including recursive Discovery/Jack/Chrysalis/Metamorphosis/Transmutation/Magnetism and Mayhem autoplay, admitted. BandageUp HEALING and specials excluded by source policy, not missing handlers | [ ] |
| Burn and Void | Shared Status constructors/draw/end-turn hooks, Task1/2; P | Overclock always base Burn; Turbo base Void; discard insertion is not draw. Actual draw loses energy, Ethereal/retention/end-turn play remain separate | [ ] |
| Dazed, Slimed, Wound plus Burn/Void (five encounter kinds total) | Shared encounter/card owners, Task6; P | Burn+ is one upgraded kind; no fabricated Void+ or new Sludge; real custody/census under Seek/Reboot/Equilibrium | [ ] |
| Parasite | Existing gain/remove journal, Task5/6; P | Actual Implant gain, removal maxHP loss, same-instance Falling, accepted-prefix fault/Death; not a blue reward | [ ] |
| Shared30 potions = BlockPotion, DexterityPotion, EnergyPotion, ExplosivePotion, FirePotion, StrengthPotion, SwiftPotion, WeakPotion, FearPotion, AttackPotion, SkillPotion, PowerPotion, ColorlessPotion, FlexPotion, SpeedPotion, BlessingOfTheForge, RegenPotion, AncientPotion, LiquidBronze, GamblersBrew, EssenceOfSteel, DuplicationPotion, DistilledChaos, LiquidMemories, CultistPotion, FruitJuice, SneckoOil, FairyPotion, SmokeBomb, EntropicBrew | Task4 extends existing `items/potion_generation.rs` and use/obtain gates; P; `m20_defect_potion_factory_and_live_obtain` | Reuse all30 accepted handlers with actual Defect, no Silent exclusives. Generated full33 includes three blue exclusives; 17/9/7 rarity counts. Combat limited rejects FruitJuice→32 at17/9/6; outside unrestricted remains33 | [ ] |
| Apotheosis/Blessing upgrade every blue kind; generated ForceField, GA, Claw, Steam and Streamline | Task2/3 shared upgrade/construction/copy owners; P | Fresh constructor state vs stat capture vs same-family repeat; temporary upgrade preserves permanent misc, no master upgrade. Creation-time upgrade/cost hook ordering remains explicit even though foreign MasterReality acquisition is outside M20 | [ ] |
| Panache/TheBomb timers, SadisticNature, Mayhem, Forethought/Setup-like free flag and selected-card continuations | Accepted M15 colorless/delayed/public owners, Task3/6/8 | Public current timers/amounts, ordinary/purge Power retirement, source parents, TimeWarp/Beat/terminal pruning, no second delayed counter store | [ ] |

M20's card vocabulary is **115 kinds**:75 blue +Parasite +five Status +L34.
No Shiv is produced by an actual-Defect generator here. A generated foreign
character card or missing constructor is an explicit unsupported boundary,
not grounds to substitute a Silent card or trim a source pool.

## Exact named source frontiers and enablement blockers

These are bounded checkpoints with existing facts to reuse first. They are not
a universal source audit. Task0 maps each to accepted predecessor evidence or
Task1–6 resolves only its named chain before enabling its affected producer.

| Frontier | Already established / exact missing composition | Owning task |
|---|---|---|
| F1 actual Defect pool | `Defect.getCardPool`→`CardLibrary.addBlueCards`→`AbstractDungeon.initializeCardPools`, named blue constructors and HEALING SelfRepair; all75/rarity/type sets | Task0/3 verifies full native constructors, typed domain and permanent-vs-generated admission; absence blocks complete family |
| F2 start order | M16 has explicit normal recharge/start-orb residual; Hello/CreativeAI/Draw/EnergizedBlue, relics and opening callbacks join `GameActionManager.getNextAction`, `EnergyManager.recharge`, `AbstractPlayer.applyStartOfTurn*`, `AbstractCreature.applyStartOfTurnPowers` | Task0 binds M16 resolution; Task1/3/4 extend only named new hooks; no guessed Plasma total |
| F3 cost/history/selection | Existing blue study fixes local ForceField/FTL/X/Seek/Hologram/Recycle/Scrape/Reboot rules; M15 actual draw history/cost/copy/finish owners are accepted | Task2 binds `UseCardAction`, `FTLAction`, `ForceField.configureCostsOnNewCard`, `ScrapeFollowUpAction`, `DrawCardAction`, `ShuffleAllAction`, player/CardGroup update and souls transfer only as needed for exact nested-history/transfer completion |
| F4 end/prevention/heal | Local FrozenCore/Buffer/SelfRepair facts known, total callback order not inferred | Task3/4/6 bind `AbstractPlayer.damage/applyEndOfTurnTriggers`, `GameActionManager` early-end path, `BufferPower.onAttackedToChangeDamage`, `RepairPower.onVictory`, shared battle-end/Smoke caller; constructor vs application and queued reduction/heal cancellation explicit |
| F5 repeated plays/life | M19 mandatory TimeWarp/Echo/Amplify/HalfDead source reconciliation remains required | Task0 binds accepted result; Task6 extends actual Heart Beat/Invincible/Shield facing. Named chain: TimeWarpPower, EchoPower, AmplifyPower, UseCardAction, NewQueueCardAction, early-end manager, DarkOrbEvokeAction and DamageAction |
| F6 item custody | Five new relic/three potion local methods checked; `EssenceOfDarknessAction.update` reads live capacity and invokes ordinary channel repeatedly | Task4 joins FrozenCore obtain replacement and opening/end callers to accepted inventory/slot owner; no direct vector swap or blanket relic cleanup |
| F7 event scope | Named act event/shrine lists, shared special list and EventHelper dispatch show no Defect-only event family; this does not validate all branches of every shared event | Task5 tests the seven captured events' actual blue basic/type/misc/curse semantics. Full standard shared lists/Neow/off-color Note remain M26–28 |

## Admission, later work and final accounting

Task5's exact captured producer/stock definitions are in the design. Its26
normal relic kinds include every nine exclusive; its eleven direct potion
placements plus unrestricted Brew make the full33 factory relevant. Task8
publishes all26/33/115 and every reachable actor/intent/power/choice, even if no
fixed run encounters them. Source family preflight happens before enabling
capture/sampling; a single missing eligible member blocks that family and must
be corrected, never hidden by filtering.

M26 owns complete standard shared/off-color card/transform/Note/Shard generation;
M27 owns remaining standard shared relic/potion acquisition and lifecycles;
M28 owns full standard event/Neow/economy pools. M20 includes their dependencies
needed by its admitted producers now: L34, actual Defect typed generators,
all33 actual-character potion outputs, shared selections/copies/upgrades,
Parasite and the captured seven event branches. It cannot defer a recursive
output or a reachable callback merely because another milestone has a broad
coverage title. A newly discovered required dependency is recorded here and
assigned to its actual shared owner before enablement; material scope changes
go to root for disposition. Finite standard-pool exclusions remain explicit.

Before acceptance attach exact accepted source hashes and named semantic test
results to each open row/group. Do not mark rows complete from this planning
package, a native build, one winning deck, adapter agreement or an unverified
Python module. Task9 preserves all original outcomes and one independent final
milestone review; root owns actual acceptance and release.
