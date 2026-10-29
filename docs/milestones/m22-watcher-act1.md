# M22 finite native Watcher A0 Act1 — source accepted

Status: root accepted this finite native source under the primary standing source-only workflow after ONE fresh comprehensive independent senior review of Tasks0–7 and all five focus classes returned **SOURCE ACCURACY READY,0 Critical / 0 Important / 0 Minor findings**. Publication is pending; full M22 and parent44 remain OPEN. The [public review record](../superpowers/plans/2026-10-29-m22-native-runtime-review.md) binds the reviewed source, root acceptance and actual proof grades. Normal Guardian and Hexaghost completion remain **UNPROVEN_OPEN**. The [approved design](../superpowers/specs/2026-10-28-m22-watcher-act1-design.md) and [Tasks0–7 plan](../superpowers/plans/2026-10-28-m22-watcher-act1.md) retain their exact approved bytes; current authority supersedes their historical pending-status prose. The [source/API reconciliation](m22-watcher-act1-reconciliation.md) records actual accepted owners, diagnostic grades and inherited limits.

## Finite boundary and producers

The predecessor is accepted M21 [PR60](https://github.com/klottick/spire-gym/pull/60), actual merge `f75eed197a5a51a612f2508caddb9f1477e4e7a8`. M22 extends the same Rust Run, combat, inventory, selection, settlement, mutation, RNG, encoder, session, semantic replay and ordered batch owners. Canonical Watcher A0 starts at short Neow with72/72 HP,99 gold, the ten starter instances, PureWater and three empty potion slots. The public profile admits exactly the three Act1 unseen-boss histories. Healthy Act2Ready is `(act2,floor17)`, nonterminal, with zero legal actions and no readiness bonus. Act2 boss metadata is map-only; Act2 play and a full Watcher run are outside this profile.

The single [content owner](../../crates/spire-core/src/run/rooms/m22_content.rs) supplies28 permanent kinds,18 relic kinds and seven potion kinds. Permanent admission is StrikePurple, DefendPurple, Eruption, Vigilance, EmptyFist, FlurryOfBlows, Prostrate, Worship, Brilliance, Devotion, ThirdEye, CutThroughFate, JustLucky, Weave, Foresight, Nirvana, WindmillStrike, Perseverance, SandsOfTime, Establishment, Meditate, CarveReality, DeceiveReality, MasterReality, BattleHymn, MentalFortress, Rushdown and LikeWater. Instances support base/once-upgraded state. Miracle, Smite and Safety remain generated-only; Dazed, Burn and Slimed are separate status vocabulary. No off-color/default pool or arbitrary catalog parser is added.

Card rewards use the **full** shared `CombatRecord.visit` modulo6, including visits beyond the legacy modulo3 boundary:

| Remainder | Ordered nonboss card offers |
| --- | --- |
| 0 | EmptyFist, MentalFortress, CarveReality |
| 1 | CutThroughFate, Rushdown, DeceiveReality |
| 2 | ThirdEye, Foresight, Meditate |
| 3 | JustLucky, Nirvana, Perseverance |
| 4 | FlurryOfBlows, Weave, SandsOfTime |
| 5 | Prostrate, Worship, WindmillStrike |

Boss card offers are Brilliance, Devotion and Establishment. The full visit modulo7 selects BlockPotion, EnergyPotion, DexterityPotion, AncientPotion, StancePotion, Ambrosia and BottledMiracle in that order. Shop stock uses the destination's actual pre-increment shop visit: first BattleHymn/LikeWater/MasterReality, GoldenEye and StancePotion; later MentalFortress/Meditate/Establishment, TeardropLocket and BottledMiracle. Card prices are75/75/150, relic150 and potion50. Every advertised stock member has a producer; descriptor metadata projects these same constants.

| Relic source | Ordered stock |
| --- | --- |
| Starter / short Neow | PureWater / NeowsLament |
| Elite eligible-unowned capture | Vajra, OddlySmoothStone, GoldenEye, TeardropLocket, Lantern, Pear, Mango, ToyOrnithopter |
| Ordinary chest eligible-unowned capture | Strawberry, Anchor, BloodVial |
| Captured boss menu, then Skip | HolyWater, VioletLotus, SacredBark, EmptyCage, SlaversCollar |

Elite/chest capture appends only `M22RelicCapture`, spelling `run.m22.relic.capture`, keyed by `[act,floor,node_id,full_visit,source_tag]`. Tags are0 elite/1 chest; elite visit is the actual CombatRecord visit and chest visit is `combat_progress.entered`. Ordered ownership filtering precedes sampling, using the actual detached combat inventory for elite settlement. Empty stock requests no RNG; every nonempty stock, including a singleton, uses `bounded_scoped_checked` and its qualified rejection law, including rejection of `u64::MAX`. Preparation clones RNG; existing reward/chest owners commit once. Inspection, decline, stale claims and delivery faults cannot reroll a captured source. Older scopes/counters and M20 cursor laws remain separate.

Playable encounters are the shared six ordinary captures, three elites and Guardian/Hexaghost/SlimeBoss: twelve setups and sixteen enemy species from `A0_ENCOUNTER_SETUPS`. M22 does not expose all57 standalone M21 fights as progression. Card/relic reference rarity metadata is finite taxonomy, not standard loot probability.

## Startup, ownership and native contracts

HolyWater appends one identity and replaces actual PureWater slot0 through the shared acquisition owner. It creates exactly three base Miracles through the accepted M21 pre-draw family, then opening draw. It does not append a second relic, retain PureWater's callback or add permanent children. Existing SacredBark captured potency/reinitialization, Collar battle context, EmptyCage exact selection/removal children, confirmed skip, full physical census and transition reset/heal/initialization flags remain shared. Publication does not re-equip, reheal or reset the Run again.

Native identities are profile `M22WatcherAct1`, content `spire-gym-m22-watcher-a0-act1-captured-v1`, rules `spire-gym-native-m22-rules-v1`, observation `spire-gym-m22-watcher-act1-observation-v1` and replay `spire-gym-m22-watcher-act1-replay-v1`. Default-capacity observation fingerprint is `5d1c77658141b2dfefbe95e17b85899ad6b77fb370e402bcbbe771146821e087`. Candidate order equals numeric row order. Hidden draw/RNG/AI/family/cursor state stays out of policy observations. Capacity failures name the group and retain the actual accepted prefix without truncating legal content. Tokens/indices are session-local; portable replay records semantic actions through the shared strict codec.

`RunSession::new_m22`, `RunBatch::new_m22` and `replay_run` use existing owners. Full batches preflight every slot and reserve before dispatch; a terminal/zero-action slot rejects the entire call unchanged. After dispatch, indexed first faults and peers whose delivery was lost remain retained. There is no implicit terminal-slot skipping, new scheduler, Python binding or public supplied-Run constructor. Older descriptors, parsers and M21 whole-catalog/preinitialization contracts remain separately checked.

General Run records declare seed/ordinal/config; they do not authenticate an independently original ordinal. Direct recorded ordinal1 reconstruction is tested without playing ordinal0. The external frozen-case release independently checks expected seed/Test/ordinal0 before startup. Fixed default-off diagnostic fixtures and their read-only checkpoint expose actual late Run flags/custody/faults only for constructed shared-owner proof. Their internal SAME-initializer replay is distinct from ordinary normal-start portable replay.

## Original once-only normal evidence — pre-Anchor-correction

All931 source files, three final executables, initial resources/RNG, capacities/descriptor, policy/receipt contract, six cases, controls and fixed conditional series were retained outside managed sessions before outcomes. Original freeze SHA-256 is `5960ed2a57cab102ba812823af88194b9ade8dba873c2f892462668bdf6f4cd5`; source manifest is `fe9c82aecbdd53540b5eac1a46559810a91c04eda823f8bcbbb3594d4b44c36b`. Each prescribed case ran once at budget32768, with no policy, seed or resource tuning.

| Seed / boss history | Actual endpoint | Successful / actual accepted actions | Matching boss entry and played record |
| --- | --- | --- | --- |
| 7 / Guardian | Death, act1 floor11, HP0/86 | 146 /146 | No Guardian entry; Death against GremlinNob |
| 7 / Hexaghost | Death, act1 floor16, HP0/86 | 232 /232 | Hexaghost entered, visit7 Death |
| 7 / SlimeBoss | Healthy Act2Ready, act2 floor17, HP86/86 | 273 /273 | SlimeBoss entered, visit7 Victory |
| 11 / Guardian | Death, act1 floor7, HP0/79 | 114 /114 | No Guardian entry; Death against GremlinNob |
| 11 / Hexaghost | Death, act1 floor7, HP0/79 | 133 /133 | No Hexaghost entry; Death against Lagavulin |
| 11 / SlimeBoss | Operational startup fault, act1 floor9 | 152 /153 | No SlimeBoss entry before fault |

Totals are four Deaths, one healthy cutoff and one fault:1050 successful recorded steps,1051 actual accepted Run actions. The fault occurred on `Move(NodeId(69))` after earning Anchor: strict Decision was current while Anchor's `GainBlock(10)` remained pending. The first fault, detached inventory, partial effects and source/binary identities remain immutable. Its receipt SHA-256 is `6d040a8c086749c15815d4b368a856708cf834cf080b14d57de8338e39816611`.

The original frozen native consumer matched all six original transcripts through scalar sessions, strict JSON/portable replay and full homogeneous pairs with workers1/2, including that exact fault. All six separate32-action interleaved Watcher/M15 coexistence prefixes and homogeneous prefix controls matched. M15 uses the nominated retained original32-action input; its current typed representation is a newly derived record, not a historical portable export or a new full191-action/policy run. None of these original results establishes an outcome on corrected source.

Original controls ran once with the original executable: nonattacking Guardian/seed7 died at floor1 after9 actions; budget0/1 were unfinished nonterminal prefixes with exit0; unknown scenario and budget32769 rejected as input errors with exit1 before construction. No control was rerun after correction.

## Narrow correction and separate supplied evidence

The shared startup owner appended an explicit Decision for M20 or M21 startup (including M22), while bottom item work used the existing before-Decision helper only for M20. The correction extends that existing condition to the same explicit-decision profiles. The strict guard, queue helper, top-work order and old non-opt-in path remain intact. A composed actual-Run Anchor regression failed before the production change, then passed; M21/M22/M20/legacy entry, finite admission, physical resources and strict first-fault controls pass. A preceding compiler fixture-field error is retained separately and is not behavioral RED.

Root authorized exactly one separately frozen corrective normal case. Corrective freeze is `f3cf28dd4753e2852a9976cce195fb17ebd2ecd9efed253ebc865ffc6177e55d`, source manifest `605aea5e63caa605d4983d356f49ee86922a05874a122dec78060c1b72a44341`. Only four source paths differ from the original freeze: the minimal effects condition, two regression files and the additive corrective-only consumer adapter. All nine initial resource/RNG snapshots and descriptor/capacity bytes match the original; caller, routes, resources and policy bytes are unchanged.

Corrective seed11/SlimeBoss ran once: **Death at act1 floor16,324 successful/actual accepted actions, HP0/79**, actual SlimeBoss entry and visit8 Death. Its first152 complete step rows equal the original fault prefix; its153rd semantic Move69 and predecision equal the original failed action, then succeed. The original fault is not replaced. Only this newly recorded case was consumed on matching corrected native scalar/strictJSON/portable/full homogeneous workers1/2, with agreement. The corrective adapter admits exactly this expected case/authority; it does not replay the other five originals or M15 again.

The frozen conditional rule requires a matching played boss Victory and healthy Act2Ready in the same normal case for each species. Guardian and Hexaghost still lack that normal witness, so after the bug was resolved the fixed **three** supplied diagnostics each ran for the first and only time. Inputs were seed7,72/72 HP,99 gold, PureWater/VioletLotus/Vajra/OddlySmoothStone, three empty potion slots, and four each Eruption+/Vigilance+/EmptyFist+/CarveReality+ with permanent IDs1..16. One real Move initializes each supplied encounter before the same frozen combat caller; it is excluded from the32768 policy budget and retained separately.

| Supplied boss | Actual played record / endpoint | Bootstrap / policy / total actual accepted actions |
| --- | --- | --- |
| Guardian | Visit0 Victory; healthy Act2Ready, HP72/72 | 1 /37 /38 |
| Hexaghost | Visit0 Victory; healthy Act2Ready, HP72/72 | 1 /31 /32 |
| SlimeBoss | Visit0 Victory; healthy Act2Ready, HP72/72 | 1 /25 /26 |

All three retain actual CombatEntered, CombatFinished, ActAdvanced, resource and bootstrap/RNG/effect evidence. There was no supplied fault, unfinished budget, fallback, strategy search or retry. They prove supplied boss/transition behavior, not earned normal traversal. Constructed boss-child/skip/small-deck/overflow/startup/transition-fault tests have a third, explicitly constructed grade; their fabricated source records are never played receipts. Ordinary portable normal replay is not claimed for supplied episodes.

## Validation and remaining gates

The single joined pre-Anchor-correction workspace command was `cargo test --workspace --locked --offline --no-fail-fast -j2 -- --test-threads=1`: **3071 passed, two known M18/M19 normal-gate failures, seven inherited ignores**,96 result groups and92 retained executed binaries. It is not a current corrected-source workspace result. No broad matrix was repeated for the narrow fix or documentation.

Current corrected focused checks pass268 affected M20/M21/M22 core tests with two inherited holds; seven old relic/startup,17 turn and17 resolver controls; nine actual native-owner tests;17 M21 and15 M22 native integration tests; eight M21/eight M22 caller tests and four data tests; and one corrective-release preflight test. The new two Anchor tests overlap these counts. Default-off normal, explicit-feature-on supplied and native consumer builds pass. Exact source/command/stdout/stderr/exit/toolchain/features/environment and actual tested/built binaries are retained; compiler, fixture and mistaken-oracle attempts remain separately graded.

The independent review assessed shared source compositions directly where a single combined checkpoint was absent. General delivery/capacity and shared row controls do not establish a separately executed elite-capture/capacity checkpoint, a late M22 fault for every group or a new panic/allocation iteration. Fixed diagnostic flags/invalid frames do not prove normal reachability, and direct ordinal1 proof is a zero-step record. These are explicit source/execution grades, not additional runtime claims or normal-gate waivers.

Normal Guardian and Hexaghost completion remain unproven; only original seed7 SlimeBoss supplies a normal Victory/healthy Act2Ready witness, on pre-correction source. No all-six current-source completion claim is made. The original four Deaths/fault and corrective Death remain unchanged evidence. M18/M19 normal gates/parents, M20 canonical normal Heart and M21 seed11 retain `meditate_choice`/parent43 stay open and unwaived.

Core M22 construction preserves explicitly supplied future CreationProfile histories; normal public M22 still rejects downstream variants. This does not release M23 or prove equality across different histories/RNG. M23 must later use the actual accepted M22 matched core histories, resources, identity/custody and transitions.

The candidate is native source only. There was no new original-game execution, broad archive extraction, install/dependency change, Python extension build/import/test or runtime parity proof. Inherited Scry gameplay-projection, numeric/source-authentication and original-frame/cap limitations remain as documented in the reconciliation. The independent final review is complete and root source-only acceptance is granted; publication, actual merge, verified accepted export and task/lifecycle disposition remain root-owned. M23 dependency release requires that actual M22 merge and verified export, and implementation additionally requires explicit root assignment against its already-approved plan. No M22 merged SHA or M23 implementation release is claimed.
