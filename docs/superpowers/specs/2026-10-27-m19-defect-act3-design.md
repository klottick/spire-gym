# M19 finite Defect through Act3 design

Date: 2026-10-27. Status: **planning candidate; independent planning review and primary-user approval pending**. Planning Bead: `spire-dxo.41.1`. Assigned source baseline: `d1fa706638a31a854fdcaeed9a9a4642abf716eb`. Current authorization covers this design and its [implementation plan](../plans/2026-10-27-m19-defect-act3.md), not implementation, runtime episodes, builds, installs, staging, commits or publication.

## Intent, decisions and approaches

M19 delivers a finite native Defect A0 episode from canonical Neow through generated Acts 1–3 and actual Act3 boss interactions. It composes the future accepted M17/M18 prefix with the existing shared Act3 runtime and ending. The [roadmap](2026-10-22-narrow-gym-milestones-design.md) assigns keys/Heart and complete ordinary Defect A0 content to M20; the [scope](../../scope.md) remains all characters, A0–20 and normally reachable content eventually.

The user selected bounded parallel groundwork using one authoritative Rust engine. Routine choices in this document are **author proposals**, not additional user approvals. The selected approach and alternatives are:

| Approach | Consequence | Disposition |
| --- | --- | --- |
| Add `M19DefectAct3` to current Run/profile/content seams | Small additive capture; existing map/room/combat/custody/ending and public transport owners remain authoritative | Selected architecture; exact capture proposed below |
| Generalize historical profiles into a new framework first | Migrates accepted behavior and delays a concrete Defect episode | Excluded from this work |
| Supply only late-act combat fixtures or a core-only example | Useful diagnostics, but insufficient normal-start and public session/replay/batch evidence | Diagnostics supplement the selected approach |

No second engine, encounter table, permanent deck, inventory, currency journal, caller transport loop or scheduler is introduced. Combat-v1, old43/default standalone behavior, opt-in all57/v2 and accepted Run profiles keep their existing identities and behavior. Native acceptance is distinct from a built/imported Python extension, original-game execution, Java RNG/map parity, training quality and win rate.

## Inputs, source grades and start gate

The manager's 712-file accepted-baseline receipt and accepted encounter export establish the assigned starting source; this planning task does not repeat that matrix. The encounter export is retained at `/home/klottick/.local/share/spire-gym/accepted/encounters-d1fa706638a31a854fdcaeed9a9a4642abf716eb/`; its supplied manifest SHA-256 is `86740abf25d66a4fed10a22cf6c4f7b48f0c2ee13837ff65577cec4825acab3a`. The manager reports native 2043/0/0 and no Python-extension proof. This is attributed predecessor evidence, not an M19 execution result.

Exact approved documents were verified and copied into `/home/klottick/.local/share/spire-gym/evidence/m19-planning-20261027/approved-inputs/`, preserving repository-relative paths. Historic pending labels inside those documents do not override the current primary-session approval statement. Their implementation remains unaccepted.

| Approved input | SHA-256 |
| --- | --- |
| `docs/superpowers/specs/2026-10-27-m16-defect-combat-design.md` | `d43678f0803537a24ed607b770f615d8eac5c961f191a27fd1940c998c6fdd51` |
| `docs/superpowers/plans/2026-10-27-m16-defect-combat.md` | `5a61b135208822bc750b38de71331d2184e8196dfcf908ef2942da81f258a226` |
| `docs/milestones/m16-defect-combat-coverage-ledger.md` | `a6296fa220167991d273304fc416cd225d1bea539f15206adbc578ca5edad1d6` |
| `docs/superpowers/specs/2026-10-27-m18-defect-act2-design.md` | `35b44a16ce51d751720e056e3cda23acfa4c2c15d27802ffdda830a8caec6098` |
| `docs/superpowers/plans/2026-10-27-m18-defect-act2.md` | `6d977cf8936cb1de6d7b90cad9e9da4341d0387666e22c35b186afb3049848bf` |

| Dependency | Required accepted contract before M19 implementation |
| --- | --- |
| M16 | Four-orb ownership; Focus/slots through10; all29 declared blue kinds and items; signed persistent misc and copy/cost custody; public values/Defect encoder groups. Its proposed `content/defect.rs` and `RunEncodingProfile::M16Defect` are not current source. |
| M17 | Canonical short-Neow Defect Act1; exact accepted capture/admission, real boss/reward/chest/transition, public native contracts. Planner coordination confirms M18-compatible Act1 acquisition28: EchoForm is not an Act1 offer. M17 is planning, not accepted source. |
| M18 | Approved finite Acts1–2 capture,29 blue kinds/14 relics/5 potions, actual Act2 bosses/rewards/treasure and healthy Act3Ready at `(3,34)` with initialized map/order and once-only healing. Bind actual accepted APIs and tests; approved prose alone is insufficient. |
| M15 shared work | Source is in progress and unaccepted. Bind only the eventual accepted cost/copy/selection/history/obtain seams consumed by M16–18; do not inspect a changing tree as immutable acceptance or implement a second owner. |
| Shared Act3 | Current `run/rooms/{act3,act3_end,transition,rewards,shop}.rs`, `events/act3.rs`, `run/combat.rs`, `combat/enemies/{mod,act3/*}.rs`, native Run encoder/session/replay/batch. M14's supplied-start acceptance amendment does not transfer. |

Before code, retain accepted predecessor revisions, source hashes/modes, exact symbol/type mappings, profile fingerprints, focused commands and remaining blockers in an M19 reconciliation receipt. M19 additionally needs independent planning review, actual primary-user implementation approval and assigned implementation ownership. A material capture/evidence change returns to root; routine import placement does not. Do not silently implement missing M16/M17/M18 behavior under M19.

Existing [reference records](../../../data/README.md), [Defect research](../../research/2026-10-19-act1-content-defect.md), [Act3 static findings](../../../data/enemies/m8-static-findings.md) and [ending findings](../../research/2026-10-23-m9-act3-ending.md) retain their individual grades. Printed values and dataset membership do not prove mechanics. Reuse those named facts; only a concrete missing member chain permits a bounded static lookup. Owned archive identity does not establish its build/channel or original-runtime fidelity. No broad extraction or public private-source output belongs in this plan.

## Finite content and acquisition

Canonical resources come from existing Run/Catalog: Defect75/75HP,99gold, four StrikeBlue, four DefendBlue, Zap, Dualcast, CrackedCore, three empty potion slots and no keys/map before real short Neow. M19 does not route normal progression through M16's authored constructor. Acts1–2 delegate to the actual accepted M17/M18 content owners; only the continuation after M18's Act3Ready changes. Given the same public actions, prefix resources, choices, encounters and settlement remain equivalent, apart from explicit profile identity, the requested initial Act3 history, and the continuation at `(3,34)`.

The proposed permanent universe is **30 kinds**: the approved29 blue kinds plus existing shared Parasite. Blue kinds are StrikeBlue, DefendBlue, Zap, Dualcast, BallLightning, ColdSnap, Coolheaded, Barrage, CompileDriver, Darkness, Fusion, Defragment, GeneticAlgorithm, Claw, SteamBarrier, Streamline, Storm, Recursion, BiasedCognition, Electrodynamics, Fission, Amplify, EchoForm, Capacitor, Consume, Loop, Bullseye, StaticDischarge and Heatsinks. Base/upgraded values and real mutable state are included. Blue admission intersects accepted M16 capability; Parasite uses the existing shared curse/gain owner separately and is never offered as a blue reward or shop card. It is unupgradable and its actual removal consequence stays shared.

Generated/temporary/status kinds are **Dazed, Burn, Slimed, Wound, Void**, including actual supported upgraded Burn. Awakened One's `awa_sludge` produces Void at BottomDraw, not a new Sludge kind. No Shiv, other blue, off-color or broad random generator is admitted by this profile.

The **19 relic kinds** are CrackedCore, NeowsLament, Lantern, Pear, Mango, DataDisk, ToyOrnithopter, Strawberry, Anchor, BloodVial, GoldPlatedCables, EmotionChip, EmptyCage, SlaversCollar, Vajra, OddlySmoothStone, BagOfMarbles, BronzeScales and RedMask. The **five potions** remain FirePotion, StrengthPotion, WeakPotion, FocusPotion and PotionOfCapacity (last two are proposed M16 native spellings until bound). The seven events are Neow, UpgradeShrine, Purifier, AncientWriting, Joust, Falling and TombOfLordRedMask. No complete ordinary Defect content claim follows.

| Act3 producer | Exact proposed capture |
| --- | --- |
| Ordinary/elite card reward | Five rotating triples: `[BallLightning,ColdSnap,Coolheaded]`; `[Barrage,CompileDriver,Darkness]`; `[Fusion,Defragment,GeneticAlgorithm]`; `[Claw,SteamBarrier,Streamline]`; `[Storm,Recursion,BiasedCognition]`; plus Skip. Use zero-based act-local settled-combat visit modulo5. |
| Potion reward | Fire/Strength/Weak by the full local visit modulo3. Pass the full visit to Defect selectors: current `rewards.rs` computes `ordinal=local%3`, which must remain only for old profiles. |
| First/later local shop | `[Capacitor,Consume,Loop]` / `[Bullseye,StaticDischarge,Heatsinks]`, each75; unowned GoldPlatedCables/EmotionChip150; FocusPotion/PotionOfCapacity50; shared purge/Leave. Count Act3 shop visits independently of prior acts. |
| Elite relic sequence | `[Lantern,Pear,Mango,DataDisk,ToyOrnithopter,Strawberry,Anchor,BloodVial,Vajra,OddlySmoothStone,BagOfMarbles,BronzeScales]`; skip owned kinds; published offer commits its next cursor even if declined. Exhaustion returns `RunError::Unsupported("captured Defect Act3 elite relic sequence exhausted")`, with no further cursor advance, replacement or refill. Accepted combat/fault custody remains retained. |
| Chest43 | Existing first-unowned Strawberry/Anchor/BloodVial, exclusive Sapphire and Proceed. If all owned, show the three disabled relic rows; Sapphire/Proceed remain as currently entitled. Never make loot disappear to mask exhaustion. |
| Questions | Existing Falling, then TombOfLordRedMask, then actual Monster capture. Falling removes the captured permanent instance/type; Tomb gains222gold when mask-owned or trades all captured gold for one RedMask. Missing types, zero gold, repeated/stale choices and accepted-child faults keep existing rules. |
| Act1/2 boss rewards | Accepted M17 `[Electrodynamics,Fission,Amplify]` / M18 `[Electrodynamics,Amplify,EchoForm]`; actual shared boss treasure EmptyCage/SlaversCollar and mandatory children or confirmed skip; no BlackBlood. |
| Act3 boss50 Victory | Existing settlement, Proceed and ending dialogs. No gold/card/potion reward, third boss chest or hidden loot RNG advance. |

Shared acquisition, belt/disabled claims, affordability, upgrade/removal, counters and single-commit fault behavior remain authoritative. The extended stock is finite: prior inventories can exhaust it. A publication fault is not Death or a successful episode; do not prune routes or refill stock after seeing outcomes.

## Actual Act3 route and setup identity

Continue the actual successor map/order created by Act2's transition; do not reconstruct a fresh Act3 player, mutate a generated boss icon, heal again or emit a second ActAdvanced. Shared Act3 supports floors35–50, chest43, rest49 and boss50. Capture selection and history restrictions stay unchanged.

| Captured class | Actual `EncounterId` and preserved RNG scope key |
| --- | --- |
| Ordinary | JawWormHorde9, OrbWalker27, ThreeShapes28, FourShapes29, SphereAndTwoShapes30, ThreeDarklings31, SpireGrowth32, Transient33, Maw34, WrithingMass35 |
| Elite | GiantHead36, Nemesis37, Reptomancer38 |
| Boss | AwakenedOne39, TimeEater40, DonuAndDeca41 |

These are **16 named setup IDs**, not a contiguous registry slice. `enemies::A0_ENCOUNTER_SETUPS` and `enemies::setup` stay the single setup owner, including shape alternatives, actual actor HP/powers and summons. Task68's57 standalone setups do not widen these pools.

The Act3 actor union has20 species: OrbWalker, Repulsor, Exploder, Spiker, SphericGuardian, JawWorm, Darkling, SpireGrowth, Transient, Maw, WrithingMass, GiantHead, Nemesis, Reptomancer, Dagger, Cultist, AwakenedOne, TimeEater, Deca, Donu. SphericGuardian/JawWorm/Cultist overlap the prefix;17 are additions to M18's cumulative37 species, yielding54 cumulative species. Vocabulary includes actual shapes/Daggers/boss partners and every reachable intent/power, not just the first actors observed in six runs.

## Defect/Act3 combinations and source obligations

All cases below are labelled supplied diagnostics unless reached naturally in the fixed normal runs. Use actual setup IDs and accepted M16 card/item rules; minimal diagnostic hands, HP frontiers and source-qualified intent checkpoints do not substitute for normal-start evidence. Add no new public supplied-start API. Existing behavior may pass new combination tests immediately; do not manufacture a failing test by changing rules.

1. **GiantHead/Nemesis/shapes:** Lightning/Dark are THORNS-category orb damage; Slow scales NORMAL attacks, not orb damage. Slow increments after actual notified card use. Pin a base Barrage hit4 at Slow5 to6, while zero-Focus Lightning passive stays3. Actual Nemesis Intangible caps each positive orb/attack hit to1, zero remains0, respecting Block. Spiker retaliation does not trigger on orb THORNS; ordinary Barrage hits use shared retaliation and can kill the player mid-card. Fission/slot changes never synthesize a card-play counter per orb.
2. **TimeEater interruption:** use actual boss with11 existing TimeWarp uses, then an accepted blue repeated card/Power. The twelfth notified use requests the shared barrier once; accepted current-card/parent work unwinds, remaining queued repeats follow accepted source cancellation, real end-turn Frost/Dark hooks and expiry happen once, and no thirteenth ordinary action is published first. Verify counter persistence between turns, Strength+2 and player Death priority. Consume accepted M15/M16 repeat semantics; do not add a blue-only end-turn scheduler. Exact repeat/barrier member ordering must be source-bound before implementation, not guessed from the old Ironclad-only cancellation match.
3. **AwakenedOne Power and revival:** Amplify/Echo Power repeats notify Curiosity1 for each actual unsuppressed Power use in phase1; orb passive/evoke is not a Power play. Use a lethal real orb hit at the phase1 frontier: same actor enters HalfDead/Rebirth without premature Victory or Run settlement, including when it is the last active actor. After Rebirth, no Curiosity reappears; final phase2 death and Cultist escape produce one matching record. Void draw/energy loss and full census remain real.
4. **Donu/Deca and Dark target lifetime:** preserve actual actor order Deca then Donu, Artifact2 each, independent turns and Dazed. Orb damage does not consume Artifact; Bullseye's LockOn debuff does, and accepted LockOn arithmetic applies only when present. Test Dark's source-list tie rule, target capture time, dead-target continuation and surviving partner; no target-by-species shortcut. For ThreeDarklings, preserve the distinction between HalfDead and dead/escaped: accepted Defect research says Dark's local scan does not separately exclude HalfDead. Do not replace that scan with `hp>0`/Active filtering. Existing revival/collapse/Victory ownership governs the result; Electrodynamics may actually down all three.
5. **WrithingMass, persistent misc and events:** actual Implant gains exactly one Parasite through the existing gain journal, including an accepted-prefix fault or Death after the gain. GeneticAlgorithm mutation uses the accepted M16 identity/journal alongside it; no gain ordinal or settlement double-applies either. Later combat retains learned misc; Falling removes precisely its captured instance without mutating a same-kind twin. Removing actual Parasite applies existing max-HP loss once. Tomb/captured relics and room-acquired Cables/Chip/DataDisk continue through shared owners. Include sparse10-slot layout and >3 occupied Barrage with these actual encounters.

M16 already owns signed Dark/misc values, slot cap10, Focus, Plasma, passive group order/Cables, Chip versus Discharge triggers, Claw/Steam/Streamline reset and blue repeats. M19 tests their Act3 composition, not a second implementation. The accepted [static findings](../../../data/enemies/m8-static-findings.md) ground Slow/Intangible/TimeWarp/Curiosity; [Defect findings](../../research/2026-10-19-act1-content-defect.md) ground orb damage, Dark targeting and copy rules. A specific unresolved repeat/barrier or HalfDead damage chain blocks that test/producer until a bounded named-source resolution; it does not authorize a generic audit or guessed expected result.

## Core and native contract proposals

All new symbols here are **proposals**. M16–18's proposed names must be replaced by their actual accepted mapping at the start gate; constructor conversion or a second state owner is forbidden.

- `RoomScenario::M19DefectAct3`, using existing `Run::new_room_scenario(RunConfig, CreationProfile, RunRng, RoomScenario) -> Result<Run,RunError>`; Defect A0/short Neow only. Extend current shared character/content and full-Act3 gates. Proposed `run/rooms/m19_content.rs` owns only Act3 capture data/helpers and exported `M19_DEFECT_ACT3_CARDS`, `M19_DEFECT_ACT3_RELICS`, `M19_DEFECT_ACT3_POTIONS`; prefix data delegates to accepted owners.
- Internal helpers: `reward_cards(local_visit: u64) -> Vec<CardId>`, `combat_potion(local_visit: u64) -> PotionId`, `shop_cards(later: bool) -> [CardId;3]`, `elite_relic(start: usize, owned: &[RelicId]) -> Result<(RelicId,usize),RunError>`. The card helper delegates to the accepted five-row M18 selector after safe modulo5. No duplicate effect or room body.
- Add `RunProfile::M19DefectAct3`; `RunSession::new_m19(config: RunSessionConfig) -> Result<RunSession,RunSessionError>` and `RunBatch::new_m19(configs: Vec<RunSessionConfig>, workers: usize) -> Result<RunBatch,RunBatchError>` through current `with_profile`/`new_with`.
- Extend the **one** `RunSessionConfig.boss_history` with `RunBossHistory::M19DefectAct3 { act1: RunAct1BossTarget, act2: RunAct2BossTarget, act3: RunAct3BossTarget }`. Require all three existing typed target enums. JSON example: `"boss_history":{"m19-defect-act3":{"act1":"guardian","act2":"champ","act3":"awakened-one"}}`. Reject missing/duplicate/unknown fields, unknown targets, partial objects and cross-profile histories before construction/reset. Reuse accepted strict parsing if present; a `serde_json::Value` alone cannot detect duplicate keys. Preserve old wire spellings.
- Apply all three targets to `CreationProfile.seen_bosses[0..3]` **before** construction, using existing unseen-boss complements (`Automaton` is the history string for BronzeAutomaton). Map generation/transition selects actual icons/orders; do not edit them later. All27 typed triples are valid configurations, but only the fixed six below are prescribed normal evidence.
- `m19_run_observation_spec(caps: &RunCapacities) -> Result<RunObservationSpec,RunSessionError>` and `encode_run_m19(decision: &PublicDecision, map: Option<&PublicMap>, caps: &RunCapacities) -> Result<RunObservation,RunSessionError>`. Add `run_schema_m19.rs` and a proposed `RunEncodingProfile::M19Defect` dispatch through `encode_run_with_schema`. Compose accepted M18 vocabulary/Defect value groups with named Act3 additions and common descriptor packagers; do not copy an encoder/schema/session loop.
- Identities: `spire-gym-m19-defect-act3-observation-v1`, `spire-gym-m19-defect-act3-replay-v1`, `spire-gym-native-m19-rules-v1`, `spire-gym-m19-defect-a0-acts1-3-captured-v1`; profile `M19DefectAct3`. Existing reward/seed/RNG identity owners remain shared. No ambiguous ordinal generic schema version.

Public projection includes every30/19/5 admitted row, five generated statuses, actual enemies/encounters/events/intents/powers/choices, four orbs with Dark payload, all known empty positions/occupancy through10, Focus, real visible misc/cost/Block, persistent card rows, selected-card operations and source-qualified active/half-dead/summoned row references. Action row i remains candidate i, mask1 exactly for legal published rows, unused capacity0; no paging or filtering. Use `RunCapacities::m10_default()` (orbs10); too-small capacity faults explicitly without truncation. Signed int32 fields use native int64 transport as accepted by M16; unknown differs from known0, empty slot from padding. No hidden draw order, RNG/AI state, future offers, raw family handles or mutation ordinals enter observations. Reproduction remains privileged. No Defect Ironclad-policy-sidecar or Python delivery claim.

## Endings and boundary custody

Reuse the source-qualified `act3_end.rs` owner: actual boss50 Victory/census/item return, `Act3BossComplete`, Proceed, then accepted dialog stages. No-key/final-act-disabled path takes four Continue choices and commits `RunTerminal::Act3NoKeys` once with terminal reward contribution0. It is a non-goal terminal, distinct from real combat `Death` and supported `EmptyDeck`. No floor51 credit, bonus boss loot or invented HeartVictory.

The all-key/final-act-enabled **supplied-only** diagnostic follows the existing non-M10 branch after dialog3 to healthy `Act4Ready`: terminalNone, zero actions, reward0, `terminated=false`, `truncated=false`, still act3/floor50, no heal/ActAdvanced/Act4 map. M19 must not become `is_m10` to enter the door. `RunSession::step` at this cutoff returns `UnsupportedBoundary("M19 stops before the Act4 door")` before action validation/mutation, preserving the healthy published boundary and reset/close; stale-token validation keeps the established precedence. There is no public injection option or normal all-key completion requirement here. Ordinary actual key choices already provided by the shared finite rooms remain meaningful, without a Heart claim.

Accepted ending/selection/potion/mutation work retains its source and first fault on delivery failure. Reject preflight-invalid actions without mutation; never roll back an accepted child, duplicate resource return, or translate an operational fault into Death/truncation. Session tokens, reset, semantic replay and batch behavior remain in their current owners.

## Frozen normal evidence and acceptance

Before prescribed outcome execution, preserve/hash exact public-only caller source, manifest, priorities, route helper identity and **10,000 accepted-action budget per case**. The caller reads only `PublicDecision`, current `PublicMap`, legal candidates and its own prior public trace. Reuse the accepted M18 caller and the existing public topology helper; M19 may add only its named continuation/ending decisions and predeclared Act3 route call. No hidden inspection, resource/HP injection, fabricated outcomes, seed search or post-outcome policy tuning.

| Seed | Act1 target | Act2 target | Act3 target |
| --- | --- | --- | --- |
| 7 | Guardian | Champ | AwakenedOne |
| 7 | Hexaghost | BronzeAutomaton | TimeEater |
| 7 | SlimeBoss | Collector | DonuAndDeca |
| 11 | Guardian | BronzeAutomaton | DonuAndDeca |
| 11 | Hexaghost | Collector | AwakenedOne |
| 11 | SlimeBoss | Champ | TimeEater |

Task3 in the plan freezes the full caller decision policy before these runs; any upstream policy mapping differences are reconciled then, before outcomes. Do not import M14's additional36 cases or supplied-start relaxation. Retain all six results and complete accepted-action transcripts before aggregate gate assertions, including first fault/capacity/unsupported details.

Every case begins at canonical Neow/no map with initial history only. On actual arrival, verify each map's selected icon, real ordinary/elite entry and matching settlement records, real boss entry and same-visit record; do not require fictional later states after Death. **Global gate:** all three current-act boss species have actual Act3 entry and matching records across the six, and at least one true normal-start case wins boss50, proceeds through the actual no-key ending and reaches Act3NoKeys. Preserve every genuine loss. Map icons, supplied victories or adapter agreement cannot satisfy this gate. A missing global requirement remains unmet; no hypothetical-loss amendment or strategy work is authorized.

Single sessions, portable semantic replay and batches with workers1/2 must agree with core for every actual decision/map/action/reward/terminal/record and final reproduction. Include separate normal Death and budgets0/1, accepted-prefix/capacity faults, stale/invalid/foreign config, reset and a supplied healthy Act4Ready cutoff. Budget exhaustion is an unfinished caller result; fault/unsupported is an operational result. Neither is Death, completion or automatic truncation.

Use focused TDD for the changed owners/combinations, then one final stable preservation matrix against the actually accepted predecessor source, followed by **one fresh independent final milestone reviewer** coordinated by root. Preserve exact source/docs/modes/receipts outside Paseo before review or session release. The eventual milestone profile must state all outcomes, missing gates, finite stock/content limits and native-only grades. No per-task review swarm, new verification framework or test-count target. Planning itself checks source references, links/paths/commands and new-file whitespace only; it claims no M19 runtime result or acceptance.
