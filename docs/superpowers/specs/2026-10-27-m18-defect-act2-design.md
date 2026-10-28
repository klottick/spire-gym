# M18 Defect through Act2 design

Date: 2026-10-27. Status: **proposed design and plan; independent planning review and actual user approval pending**. The current session authorizes planning in parallel with M16, not implementation. Baseline: `e5b07da5609a3a2654b3bced3f8388ea96324a8f`; planning Bead `spire-dxo.40.1`. No upstream or prior-milestone approval transfers to M18.

## Intent and established constraints

M18 extends the same canonical Defect A0 Run through generated Acts 1–2, actual ordinary/elite/boss combat and real rewards, ending at a healthy nonterminal `Act3Ready`. The [roadmap](2026-10-22-narrow-gym-milestones-design.md) assigns complete ordinary Defect A0 character content and Heart progression to M20. M18 is a declared finite capture, not a full-pool or successful-build completeness claim.

The user selected one authoritative Rust engine. Extend the shared Run, character mechanics, encounter setup, resolver, room/reward/shop/rest/transition, observation/session/replay/batch owners. Do not create a Defect Run loop, resolver, enemy table, master deck, currency journal or Python facade. Keep combat-v1, policy-v3 and existing M9/M10/M12/M13/M14 identities and boundaries frozen. Native source acceptance makes no Python-extension, original-runtime, Java RNG/map parity, training or win-rate claim.

## What is present, and what is proposed

At the stated baseline, `run/engine.rs` and `content/cards.rs` already own Defect's 75/75 HP, 99 gold, four StrikeBlue, four DefendBlue, Zap, Dualcast, CrackedCore, three empty potion slots and no keys. `combat/characters.rs` has Lightning/Frost and three initial slots; it does not yet implement M16's complete four-orb contract. `cards::public_values` currently delegates only to Ironclad/Silent value owners, so complete blue visible numeric values are another explicit M16 dependency. Thirteen non-definition-only blue kinds in the catalog are not a Defect progression profile.

The accepted [M12 design](2026-10-26-m12-silent-act1-design.md), [M13 design](2026-10-26-m13-silent-act2-design.md), and [M13 profile](../../milestones/m13-silent-act2.md) establish the reusable finite progression shape. `run/rooms/{act1,act2,rewards,shop,boss,transition}.rs` own the lifecycle. `spire-env/src/{run_session,run_observation,run_schema_m13,run_replay,run_batch}.rs` own its public form. M13's Silent tables/encoder gates cannot be used as Defect content by relabelling them.

| Dependency | Required acceptance before M18 implementation | Present planning status / reconciliation |
| --- | --- | --- |
| M16 | Four owned orb kinds; Focus/slots through cap 10; exact declared card/item mechanics; persistent card mutation custody; public values and additive Defect encoding | Concurrent proposal, not accepted source. Planner coordination agrees on the 29 kinds below and proposed `content/defect.rs` admission seam. Do not route normal M18 through M16's proposed authored-combat constructor. |
| M17 | Canonical Defect short-Neow normal start; captured Act1 card/item gains and upgrades; all three initial-history boss alternatives; actual boss reward/chest children; once-only heal and same-player Act2Ready; native reset/replay/batch and finite vocabulary | **Unassigned, unplanned and unaccepted.** These are minimum contract assumptions, not existing APIs. The capture below is a concrete proposed M17-compatible prefix. Compare against the actual accepted M17 catalog, actions, records, cutoff, descriptors and source hashes before implementation; any mismatch is a named blocker until reconciled. M18 must not silently implement M17. |
| M15 shared work | Only shared generation/cost/history APIs actually needed by an accepted dependency, with exact accepted source identity | The M15 v3 design and plan are primary-user-approved for implementation; M15 source is not implemented or accepted. Do not name its planned functions as existing or implement another history owner. Use only shared APIs actually needed by M18 after their accepted source identity is bound; basic M18 need not wait for unrelated Silent completeness. |
| Task68 encounters | Accepted common `A0_ENCOUNTER_SETUPS`, shared startup/settlement and any additive public checkpoint fields | Task68 is accepted and PR53 is merged at `d1fa706638a31a854fdcaeed9a9a4642abf716eb`; its 57-encounter/v2 source preserves existing captured Act2 selection, old setup keys and default profiles. Current M18 route has no theft actors; no second theft journal or v2 sidecar is required. Reconcile this accepted source identity into the M18 implementation baseline; the planning checkout remains at the stated e5b07da baseline. |

Execution additionally requires actual root-session approval of the reviewed M18 design/plan and an assigned implementation Bead. The reconciliation receipt records accepted revisions, input hashes, exact API mapping, frozen old fingerprints, and blockers; an accepted predecessor alone is insufficient. A material change to this finite capture or evidence gate returns to the root/user for an amendment.

## Approaches

| Approach within the shared engine | Tradeoff | Proposal |
| --- | --- | --- |
| A. Distinct finite `M18DefectAct2` profile using character-owned admission and current progression owners | Bounded content/public changes; preserves M17's Act2Ready contract; requires explicit schema and dependency reconciliation | Recommended. It directly delivers the runnable milestone without speculative infrastructure. |
| B. Generalize all historical profiles into a new framework first | Could consolidate future characters, but migrates accepted behavior and adds design risk before a second Defect progression consumer exists | Defer. Extract only a small shared hook proven necessary by M16/M17/M18. |
| C. Publish only a core example and delay session/replay/batch | Less immediate transport work, but does not deliver the roadmap's evolving native public interface or expose its encoding gaps | Reject for M18. |

The architecture is a user decision; A and the exact rows below are author proposals under the authorized combined planning/review process. No human selection or document approval is claimed.

## Exact finite content and producers

Every identifier in this section is a **proposed M18 capture**, including kinds which depend on future M16. Read the [reference index](../../../data/README.md) and domain guides before records: card printed values are not evaluated effects; relic static findings retain their stated pending-review grade; dataset availability is not native support. No private raw source is published. The 29 permanent kinds are exactly the union of the starter, five ordinary triples, two shop triples and both boss triples below; base/upgraded and real mutable values are included. The future M17 prefix must be reconciled, never retroactively inferred accepted from this table.

| Producer | Exact proposed row / rule |
| --- | --- |
| Canonical start | Existing Catalog/Run starter above, no duplicate initializer; real short Neow offers max HP +7 or NeowsLament |
| Ordinary card reward | Five rotating triples: `[BallLightning, ColdSnap, Coolheaded]`; `[Barrage, CompileDriver, Darkness]`; `[Fusion, Defragment, GeneticAlgorithm]`; `[Claw, SteamBarrier, Streamline]`; `[Storm, Recursion, BiasedCognition]`. Zero-based act-local settled combat visit modulo 5; Skip remains legal. |
| Act1 / Act2 boss card reward | Proposed prefix `[Electrodynamics, Fission, Amplify]`; M18 Act2 `[Electrodynamics, Amplify, EchoForm]`, plus Skip. These are declared captures, not standard RNG/rarity generation. |
| First / later shop cards | `[Capacitor, Consume, Loop]` / `[Bullseye, StaticDischarge, Heatsinks]`, prices `[75,75,75]` for both. Act2 uses its own `shop_visits`, independent of Act1. Shared affordability, purge, upgrade and Leave rules apply. |
| Elite relic sequence per act | `[Lantern, Pear, Mango, DataDisk, ToyOrnithopter, Strawberry, Anchor, BloodVial]`; skip owned kinds. Successful publication commits the next cursor even if declined; exhausted lookup returns `RunError::Unsupported("captured Defect Act2 elite relic sequence exhausted")` for Act2 without a further cursor advance, refill or replacement. |
| Shop relic / potion | First `GoldPlatedCables` / `FocusPotion`; later `EmotionChip` / `PotionOfCapacity`, fixed relic150/potion50. Owned relics are omitted; full-belt handling stays shared. These shop witnesses are captured placements, not assertions of original acquisition tiers. |
| Combat potions | `[FirePotion, StrengthPotion, WeakPotion]` by act-local settled visit modulo 3; offered/claimed/disabled/discarded custody is shared. |
| Ordinary / boss treasure | Existing Strawberry/Anchor/BloodVial captures and Sapphire exclusion; boss EmptyCage/SlaversCollar, owned-item disabling, mandatory removal children or explicit skip/cancel/confirmation. Never offer BlackBlood to Defect. |
| Events | `Neow`, `UpgradeShrine`, `Purifier`, `AncientWriting`, `Joust`. Act2 questions use AncientWriting first, then Joust when shared 50-gold eligibility permits or Purifier otherwise; subsequent questions use the shared Monster path. No other events or curses are advertised. |

This is **29 permanent card kinds, 14 relic kinds, five potion kinds**. Relics are CrackedCore, NeowsLament, Lantern, Pear, Mango, DataDisk, ToyOrnithopter, Strawberry, Anchor, BloodVial, GoldPlatedCables, EmotionChip, EmptyCage, SlaversCollar. Actual generated combat kinds are Dazed, Burn (including its upgrade), Slimed and Wound; none is a permanent reward. Leap, BeamCell, GoForTheEyes and SweepingBeam can be implemented in older combat content yet remain outside this M18 capture. No broad random card generator or off-color acquisition is added.

`content/defect.rs` is M16's **proposed**, not existing, shared owner for `M16_DEFECT_CARDS`, `supports_permanent(CardId) -> bool`, and `supports_generated(CardId) -> bool`. M18's declared allowlist intersects that accepted capability rather than adding a fallback to all blue catalog entries. Preserve M17's accepted prefix admission. M18 room capture helpers select data only; card laws, acquisition, upgrades, persistent misc, generated/copy family identity and mechanics remain in accepted shared owners.

## Shared route and encounter boundary

Reuse the actual shared Act1 capture (six ordinary IDs Cultist, JawWorm, TwoLouse, LargeSlime, ThreeLouse, LotsOfSlimes; elites GremlinNob, Lagavulin, ThreeSentries; bosses Guardian, Hexaghost, SlimeBoss), subject to M17 reconciliation. Act2 uses the unchanged M6/M7/M13 capture:

- Ordinary: SphericGuardian, ThreeByrds, CenturionAndHealer, Snecko, ChosenAndByrds, ThreeCultists.
- Elite: GremlinLeader, Slavers, BookOfStabbing.
- Boss: Champ, BronzeAutomaton, Collector.

Use shared `enemies::A0_ENCOUNTER_SETUPS` and `enemies::setup`, never a copied HP/actor table. Act2 emits 22 species including the already-Act1 Cultist: SphericGuardian, Byrd, Centurion, Mystic, Snecko, Chosen, Cultist, GremlinLeader, GremlinMad, GremlinSneaky, GremlinFat, GremlinShield, GremlinWizard, SlaverBlue, Taskmaster, SlaverRed, BookOfStabbing, Champ, BronzeAutomaton, BronzeOrb, Collector, TorchHead. This is 21 additions to the stated Act1 enemy vocabulary, not 21 total Act2 species. Act3's AwakenedOne, TimeEater and DonuAndDeca are map icons only.

M18 continues through real Act2 boss33, rewards and boss treasure34, mandatory children/confirmed skip, then the existing transition's once-only healing, ActAdvanced event and actual Act3 map/boss-order initialization. At `(act,floor)=(3,34)`, `Act3Ready` has terminal None, no legal actions, no readiness reward, `terminated=false`, `truncated=false`. Existing boss-treasure floor credit remains real. Reject a further action with `UnsupportedBoundary("M18 stops at Act3Ready")` before mutation; reset/close remain healthy. Caller budget is a separate caller result. Do not expose an Act3 combat path.

## Relevant actual Act2 interactions

These are labelled supplied combat/room diagnostics, independent of normal-start outcome evidence. Use actual encounter setup and accepted M16 rules; a fixture may choose a small hand/seed or valid inventory for a named interaction, never stand in for boss traversal. Fix only a reproduced source-grounded defect in the existing owner.

1. **Lightning, Artifact and Flight:** against SphericGuardian and BronzeAutomaton, orb damage leaves Artifact 3 unchanged and still respects real Block (SphericGuardian starts Block 40). Against actual Byrds, zero-Focus Lightning passive3/evoke8 bypasses Flight mitigation and does not consume Flight; a normal Barrage hit follows Flight's normal-damage rule. DataDisk's actual one Focus increases Lightning to4/9 and Frost to3/6; do not replace damage with debuff application.
2. **Frost and real multiattack:** channel Frost using ColdSnap/Coolheaded against BookOfStabbing or Slavers; assert passive Block is granted at the real end-turn checkpoint and absorbs consecutive actual hits once through the common damage owner. Distinguish blocked hits from HP loss for StaticDischarge/EmotionChip; use M16's accepted ordering, no pre-aggregated HP shortcut.
3. **Dark and summons:** against Collector/TorchHead or GremlinLeader's actual summons, Dark retains its accumulated per-orb payload across Focus changes, chooses a live lowest-current-HP target using the accepted tie rule at evoke time, and removes/compacts once. Retained target handles must not transfer damage to a dead/replaced summon. Recursion preserves the accepted Dark payload; full slots and empty/sparse cap10 layouts use the same M16 slot owner.
4. **Plasma, Confused and slot changes:** actual Snecko applies Confused; use legal Fusion/Fission/Consume/Capacitor and Focus/Capacity potion actions. Plasma energy is unaffected by Focus; draw cost randomization uses the existing shared cost/RNG owner and does not alter persistent base costs. Assert real action energy and slot cap10, not a selected favorable draw cost. Invalid/full-capacity actions follow accepted M16 semantics without consuming potion twice.
5. **Persistent identity through Stasis and act transition:** actually play GeneticAlgorithm through Run, in a later actual Automaton fight let BronzeOrb's real Stasis capture that already-mutated instance before it is played again, resolve owner death/return or combat end, and then verify onward combat entry. Preserve the permanent instance/family's exact misc once; temporary/fresh copies obey accepted M16 identity rules. Test accepted-prefix fault and genuine Death as well as Victory: persistence is not victory-only and no second master deck is introduced. Shared source-owner fault custody, not a rollback of accepted actions, governs retry.
6. **Item and power timing:** acquire the captured DataDisk, GoldPlatedCables, EmotionChip and declared potions through actual room owners, then use accepted M16 hooks in an actual Act2 setup. Pair Cables' extra front-slot (index0) passive in eligible natural scheduler groups and Chip's prior-HP-loss trigger with Frost/Dark/Plasma and the actual damage sequence; blocked damage alone must not arm HP-loss triggers. Do not reimplement the scheduler, counters or persistent mutation journal.

Reference support is source-qualified: existing runtime damage/Artifact/Flight paths substantiate row1; M16's accepted source must substantiate new Dark/Plasma/slot/item/misc rows. Needed unresolved named source facts block only the affected interaction until resolved with the already authorized bounded archive/tools. Owned archive identity does not establish upstream build/channel or original-runtime fidelity.

## Proposed native interfaces and schema

All names in this section are additive **proposals**. Preserve the existing constructor signatures and transport loops.

- Core `RoomScenario::M18DefectAct2` and the current `Run::new_room_scenario(RunConfig, CreationProfile, RunRng, RoomScenario) -> Result<Run, RunError>`; Defect A0/short Neow only. Extend shared act capability gates using accepted M17 content hooks. Proposed `run/rooms/m18_content.rs` owns only the Act2 rows; re-export `M18_DEFECT_ACT2_CARDS`, `M18_DEFECT_ACT2_RELICS`, `M18_DEFECT_ACT2_POTIONS` from `run/mod.rs`. Actual M17 prefix hooks remain its owner.
- `RunProfile::M18DefectAct2`; `RunSession::new_m18(config: RunSessionConfig) -> Result<RunSession, RunSessionError>`; `RunBatch::new_m18(configs: Vec<RunSessionConfig>, workers: usize) -> Result<RunBatch, RunBatchError>`, using shared `with_profile` / `new_with`.
- Keep one `RunSessionConfig.boss_history`. Add `RunBossHistory::M18DefectAct2 { act1: RunAct1BossTarget, act2: RunAct2BossTarget }` using the existing typed target enums. Exact JSON: `"boss_history":{"m18-defect-act2":{"act1":"guardian","act2":"champ"}}`. Require both fields, reject duplicate/unknown fields, unknown targets, partial objects and cross-profile histories before construction/reset. Map targets into `CreationProfile.seen_bosses[0]` and `[1]` before Run construction, leaving `[2]` at experienced-profile defaults. Never edit generated maps/history later.
- `m18_run_observation_spec(caps: &RunCapacities) -> Result<RunObservationSpec, RunSessionError>` and `encode_run_m18(decision: &PublicDecision, map: Option<&PublicMap>, caps: &RunCapacities) -> Result<RunObservation, RunSessionError>`. Compose accepted M16 Defect character/card value groups and shared Run room/map groups using the one row encoder and descriptor packagers, with a proposed `RunEncodingProfile::M18Defect` branch if that accepted seam exists. Do not copy M13's schema body or claim M16's proposed symbols exist today.
- Identities: `spire-gym-m18-defect-act2-observation-v1`, `spire-gym-m18-defect-act2-replay-v1`, `spire-gym-native-m18-rules-v1`, `spire-gym-m18-defect-a0-acts1-2-captured-v1`, profile `M18DefectAct2`. Do not reuse an ordinal generic v4/v5 identity that could collide with M15/M17.
- Publish every reachable card/value, four orb kinds with retained Dark payload, slot count/occupancy/empty slots through10, Focus, Plasma energy, visible persistent misc, power/counter, generated status, Stasis/summon identity by row reference, intent, finite event, selection and room choice. Reuse `RunCapacities::m10_default()` (already orbs10); custom capacities overflow explicitly without truncation. Known empty slots differ from N/A/padding. Foreign IDs and operations fail closed per profile.
- Public observations do not expose hidden draw order, RNG state, enemy AI memory, future offers, raw instance/family handles or private mutation journals. Real visible misc and Dark amounts are values, not internal IDs. Privileged reproduction stays privileged. Existing Ironclad combat-policy sidecars remain frozen; M18 does not claim a compatible Defect sidecar or Python transport.

## Fixed from-start evidence and outcome gate

Before any outcome runs, retain and hash one deterministic public-only caller with its action budget (10,000 accepted actions per case), route tie-breaks, card/selection/target/potion/room priorities and six-case manifest. It reads only `PublicDecision`, current `PublicMap`, legal candidates and its own prior public trace. Reuse the shared public topology routing helper and accepted M17 caller where possible; no hidden state or script that injects room/combat outcomes.

| Seed | Act1 target | Act2 target |
| --- | --- | --- |
| 7 | Guardian | Champ |
| 7 | Hexaghost | BronzeAutomaton |
| 7 | SlimeBoss | Collector |
| 11 | Guardian | BronzeAutomaton |
| 11 | Hexaghost | Collector |
| 11 | SlimeBoss | Champ |

Keep every case and action transcript. Each starts at canonical Defect Neow with no map; actual Neow creates the selected Act1 icon; the same Run traverses actual Act1 boss entry/reward/chest into the selected Act2 map. Count boss coverage only on actual entered combat plus a matching `CombatRecord`, never an icon. The fixed matrix must enter all three Act2 boss species, with at least one ordinary and elite Act2 entry, and at least one canonical normal-start Act2 boss Victory through actual rewards/chest to Act3Ready. Report each case's Act1/Act2 progress and uncovered requirements; do not promise six wins or all three boss victories.

Genuine Death remains valid gameplay and is replayed as Death. Early Death leaves unvisited boss coverage unmet; all fixed deaths with no Act3Ready leave completion unmet. Budget exhaustion, finite-stock error and delivery fault are separate. Do not remove cases, search seeds, tune the frozen policy after outcomes, alter card/enemy/resources or inject later state to pass. A correctness fix needs a source-grounded regression and full six-case rerun with old evidence retained. Any different coverage or supplied-loadout alternative requires an explicit M18 amendment in the primary session; M13/M14 amendments do not transfer automatically.

Normal evidence also includes nonattacking natural Death and budgets0/1; labelled diagnostics cover invalid/stale actions, wrong profile/history, foreign content, full belt, counter exhaustion, accepted-child/delivery fault and cutoff reset. Compare core, native single, semantic replay and ordered batch workers1/2 for all six cases, every public boundary/reward/terminal and map. Actual supplied interaction wins cannot satisfy the from-start gate.

## Implementation and evidence handoff

The [implementation plan](../plans/2026-10-27-m18-defect-act2.md) assigns cohesive TDD work and five review-focus classes. No builds, product tests, installs, commits, PRs or cleanup are authorized during planning. Check changed documentation links/paths/commands. Preserve exact candidate documents, baseline public source and relevant qualified input hashes in independently retained read-only storage outside managed sessions; root dispatches one independent planning reviewer only after receiving those identities.

After actual execution approval and upstream reconciliation, use focused failing tests for new behavior, one stable final integrated native preservation matrix, and one fresh independent final milestone review. Record the accepted-source baseline's real failure inventory rather than copying historical counts or forcing known losses green. Native results do not refresh the Python extension; any future Python claim requires a separate authorized source/artifact/import receipt. Root owns outcome acceptance and publication administration. This planning task does not change the parent outcome or M17.
