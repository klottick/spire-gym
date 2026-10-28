# M17 Defect through Act1 design

Date: 2026-10-27. Status: **proposed; independent planning review and primary-user approval pending**. Planning Bead: `spire-dxo.39.1`. Read baseline: `d1fa706638a31a854fdcaeed9a9a4642abf716eb`, branch `docs/m17-defect-act1-plan-20261027`. Current authorization is parallel groundwork and a written package, not implementation.

## Intent and approach

M17 fills the normal-start Act1 bridge between approved M16 combat and approved M18 Acts1–2 planning. A caller starts canonical Defect A0 at actual short Neow, traverses generated topology and real captured rooms/combat, resolves an actual Act1 boss and its rewards/chest, then inspects healthy nonterminal `Act2Ready`. The [roadmap](2026-10-22-narrow-gym-milestones-design.md) leaves complete Defect A0 character content and the Heart to M20. This finite capture does not establish standard random pools or all-seed completion.

The primary user selected bounded parallel groundwork on one shared Rust engine. Within that approach, this author proposes a finite `M17DefectAct1` profile, following the accepted [M12 progression shape](../../milestones/m12-silent-act1.md). A full blue-pool expansion would add unrelated M20 mechanics; a new generic profile framework would migrate accepted behavior before a concrete need; an authored-combat-only bridge would leave the normal-start milestone missing. Extend the existing owners and the small character/content seams they require. These finite choices and the evidence gate below are proposals for review and user approval, not decisions already approved for M17.

Keep `combat-v1`, default/old43 standalone combat, opt-in all57/v2, policy-v3 and accepted M9/M10/M12/M13/M14 identities and endpoints frozen. Do not widen their captured encounter pools. No second Run loop, encounter setup table, orb scheduler, master deck, inventory, currency or mutation journal, Python gameplay loop, ML work, original-game execution or publication belongs to M17.

## Inputs, source grades and execution dependencies

Root verified all 712 tracked baseline hashes against main; its receipt is retained outside Paseo at `manager-parallel-planning-20261027/accepted-baseline-hashes.json` (SHA-256 `7adfc1f5fe22ef9f8b6a64ae0bf91276d9e44bc033c08acd254712b0458aa0e0`). This planning task confirms checkout identity without repeating that verification or runtime checks.

| Input | Bound identity / qualification |
| --- | --- |
| Accepted encounters | PR53 at `d1fa706638a31a854fdcaeed9a9a4642abf716eb`; read-only accepted export `~/.local/share/spire-gym/accepted/encounters-d1fa706638a31a854fdcaeed9a9a4642abf716eb/`, manifest SHA-256 `86740abf25d66a4fed10a22cf6c4f7b48f0c2ee13837ff65577cec4825acab3a`. Root reports native 2043/0/0; no new test or Python proof here. |
| Approved M16 design / plan | Retained `2026-10-27-m16-defect-combat-design.md` SHA-256 `d43678f0803537a24ed607b770f615d8eac5c961f191a27fd1940c998c6fdd51`; `2026-10-27-m16-defect-combat.md` SHA-256 `5a61b135208822bc750b38de71331d2184e8196dfcf908ef2942da81f258a226`. **Implementation held on accepted M15; no accepted M16 source.** |
| Approved M18 design / plan | Retained `2026-10-27-m18-defect-act2-design.md` SHA-256 `35b44a16ce51d751720e056e3cda23acfa4c2c15d27802ffdda830a8caec6098`; `2026-10-27-m18-defect-act2.md` SHA-256 `6d977cf8936cb1de6d7b90cad9e9da4341d0387666e22c35b186afb3049848bf`. **Implementation held on accepted M16 and M17.** Its prefix rows are preserved here; this does not rewrite or reapprove M18. |
| M15 shared owners | User-approved implementation is in progress and unaccepted. Cost/copy/selection/history/obtain proposals are not current APIs. Reconcile immutable accepted source before using them; do not read its changing author tree as an acceptance snapshot. |

All input pairs and the M16 coverage ledger are copied with verified hashes under `~/.local/share/spire-gym/evidence/m17-planning-20261027/approved-inputs/`. Root's current approval report governs their approval status; stale pending/unassigned language inside retained documents is historical. Preserve the bytes and record reconciliation rather than editing the approved pairs.

At this baseline, `run/engine.rs`, `run/profile.rs` and `content/cards.rs` supply canonical Defect resources and experienced creation history. `run/rooms/act1.rs` already owns the shared captured route; `rooms/{neow,rewards,shop,rest,events,selection,boss,transition}.rs` own real consequences. `run/mutations.rs` presently admits Ironclad/Silent profiles, not this Defect progression. `run/combat.rs` owns inventory transfer, combat records, gains and settlement. `spire-env` already supplies one encoder, session, replay parser and batch scheduler. Catalog presence or `definition_only` metadata does not establish active effect support.

M16 must deliver and have accepted source for its four-orb state, Focus/slot cap10, blue public values, exact29 capability, item hooks, repeats/cost integration and GA per-action absolute-misc mutation synchronization. M17 consumes these contracts; it does not implement missing M16 mechanics. Bind the actually accepted M15 cost/history dependencies through M16 rather than adding another history layer. A named unresolved prerequisite blocks only its affected integration, with the exact missing rule/API reported to root. Implementation requires the reviewed M17 pair, actual primary-user approval, an assigned implementation Bead and accepted predecessor reconciliation.

Use the [reference index](../../../data/README.md), [card guide](../../../data/cards/README.md), [relic guide](../../../data/relics/README.md) and [qualified Defect report](../../research/2026-10-19-act1-content-defect.md). Printed fields are not evaluated rules; historical static findings retain their review/composition limits. The owned archive SHA-256 is `cfad868ac8d65a88e71a0bf096fb09f78811e553effe0787c5309a655e081673`, with upstream channel/build unverified. No new private lookup is needed to draft this bridge. If execution exposes an ambiguous named rule, reuse retained qualifications or inspect only the authorized named class/member, never run the original game or publish private bytes. Keep RNG-001/TIME-001 and the distinction between source-grounded and original-runtime validated.

## Exact finite acquisition and admission

The following captures deliberately match the approved M18 Act1 prefix. They are deterministic supported placements, not ordinary rarity, tier, drop-chance, pity or complete shop samplers. Base and once-upgraded forms use accepted M16/shared implementations.

| Producer | Exact proposed stock / rule |
| --- | --- |
| Canonical starter | `Run::new`/Catalog: 75/75 HP, 99 gold, four StrikeBlue, four DefendBlue, Zap, Dualcast, CrackedCore, three empty potion slots, no keys. No separate initializer or supplied-start API. |
| Short Neow | Actual Max HP +7 (82/82 afterward) or NeowsLament, selected before generated-map creation. Full Neow is outside this profile. |
| Ordinary/elite card rewards | Five triples in order: `[BallLightning, ColdSnap, Coolheaded]`; `[Barrage, CompileDriver, Darkness]`; `[Fusion, Defragment, GeneticAlgorithm]`; `[Claw, SteamBarrier, Streamline]`; `[Storm, Recursion, BiasedCognition]`. Index by zero-based Act1 settled combat visit modulo5; actual boss uses its separate row. Include Skip. |
| Boss card reward | `[Electrodynamics, Fission, Amplify]`, plus Skip. No EchoForm offer here. |
| First/later merchant cards | `[Capacitor, Consume, Loop]` / `[Bullseye, StaticDischarge, Heatsinks]`, prices `[75,75,75]` for both. First means Act1 `shop_visits==0` before entry commits; every later visit uses the latter row. |
| Elite relic offers | `[Lantern, Pear, Mango, DataDisk, ToyOrnithopter, Strawberry, Anchor, BloodVial]`. Skip owned kinds; successful publication consumes the selected cursor even if declined. Exhaustion is `RunError::Unsupported("captured Defect Act1 elite relic sequence exhausted")`, with no refill or failing cursor advance. |
| Merchant relic/potion | First GoldPlatedCables / FocusPotion; later EmotionChip / PotionOfCapacity, fixed 150/50 gold. Omit owned relics; retain shared affordability, full-belt, purge and Leave semantics. These are captured placements, not original shop-tier claims. |
| Combat potion rewards | `[FirePotion, StrengthPotion, WeakPotion]` by zero-based settled visit modulo3. A disabled full-belt offer survives discard; one accepted claim grants one potion. |
| Ordinary treasure | Existing first unowned `[Strawberry, Anchor, BloodVial]` and real Sapphire-versus-relic exclusion/Proceed. Preserve existing exhaustion behavior; do not refill it or silently reuse a relic. |
| Boss treasure | EmptyCage / SlaversCollar, owned-item disabling and explicit skip/cancel/confirmation. EmptyCage's real mandatory removal children must complete before proceeding. BlackBlood and other Ironclad exclusives are not offered. |
| Events/rest | Neow, then shared UpgradeShrine and Purifier on the first/second question visits; later questions resolve as Monster. Shared rest/Smith/Recall, permanent upgrade/removal, shop purge and real key custody remain available. No character-exclusive event is invented. |

This is **28 obtainable permanent card kinds, 14 relic kinds and five potion kinds**. `M17_DEFECT_ACT1_CARDS` is the union of the starter and card rows above. Relics are CrackedCore, NeowsLament, Lantern, Pear, Mango, DataDisk, ToyOrnithopter, Strawberry, Anchor, BloodVial, GoldPlatedCables, EmotionChip, EmptyCage, SlaversCollar. Potions are FirePotion, StrengthPotion, WeakPotion, FocusPotion, PotionOfCapacity. Generated encounter statuses are Dazed, Burn (including upgraded Burn), Slimed; they are never permanent rewards. Wound first belongs to M18 encounters, not this captured Act1 route.

M16's proposed `content::defect::{M16_DEFECT_CARDS,supports_permanent,supports_generated}` owns **29 mechanics-capable blue kinds**, with the same kinds admitted for permanent and explicit-copy construction. M17 acquisition/admission intersects that capability with its28-kind profile; it does not change the29-kind capability or pretend to implement the remaining ordinary blue pool. The full alignment is:

| M16 capability group | M17 producer alignment |
| --- | --- |
| StrikeBlue, DefendBlue, Zap, Dualcast | Canonical starter |
| BallLightning, ColdSnap, Coolheaded, Barrage, CompileDriver, Darkness, Fusion, Defragment, GeneticAlgorithm, Claw, SteamBarrier, Streamline, Storm, Recursion, BiasedCognition | Five ordinary reward triples |
| Capacitor, Consume, Loop, Bullseye, StaticDischarge, Heatsinks | First/later shops |
| Electrodynamics, Fission, Amplify | Act1 boss reward |
| EchoForm | M16 mechanic/copy capability only; no normal M17 acquisition. Approved M18 adds its Act2 boss producer, making the Acts1–2 acquisition union29. |

Explicit same-family/stat/fresh copies of actually admitted cards use M16's shared rules; no random generator or foreign-card fallback is added. Leap, BeamCell, GoForTheEyes and SweepingBeam can retain existing combat behavior without entering this profile. Profile constants describe acquisition, not a second rule registry. M17's schema need only admit its28 kinds plus the three reachable statuses, and should reject EchoForm in a forged M17 decision. M18 composes a larger vocabulary through its own opt-in schema.

The existing reward owner currently calculates `local % 3` before calling captured card selectors. Add the Defect branch using the **unreduced** visit for modulo5; do not globally change legacy selectors or their rows. Act1 starts at visit0, advances on actual settlement, and does not advance for rejection, re-observation or delivery retry. Do not add another visit journal. Preserve shared once-only currency, item counters and obtain hooks through the accepted action/child commit boundary.

## Shared route, bosses and persistence

Reuse `rooms/act1.rs::start/capture_room/enter`, generated map eligibility and `enemies::A0_ENCOUNTER_SETUPS`/`setup`. The exact12 playable captures remain:

- Ordinary: Cultist, JawWorm, TwoLouse, LargeSlime, ThreeLouse, LotsOfSlimes.
- Elite: GremlinNob, Lagavulin, ThreeSentries.
- Boss: Guardian, Hexaghost, SlimeBoss.

There are16 reachable actor kinds including split offspring: Cultist, JawWorm, LouseRed, LouseGreen, AcidSlimeS/M/L, SpikeSlimeS/M/L, GremlinNob, Lagavulin, Sentry, Guardian, Hexaghost, SlimeBoss. Champ, BronzeAutomaton and Collector appear only as successor-map icons. Accepted standalone all57 support is not permission to add more captured Run encounters or advertise Act2 combat.

Initial creation history selects actual boss alternatives before Neow/map generation. Map/reward/public code cannot replace a generated boss. Use the existing Act1 unseen mapping: Guardian -> `["Hexaghost","SlimeBoss"]`; Hexaghost -> `["Guardian","SlimeBoss"]`; SlimeBoss -> `["Guardian","Hexaghost"]`. Keep experienced Act2/3 history defaults. Actual entry updates the common history/visit owner. Count coverage from `CombatEntered { visit, encounter }` paired with `CombatFinished(CombatRecord { visit, encounter, outcome })` on the same visit, never from a boss icon.

All combat uses shared Run entry/settlement and M16 mechanics, including CrackedCore's one starting Lightning, DataDisk's once-per-entry Focus, Cables/Chip's accepted phases, potion custody and generated statuses. One authoritative inventory/counter owner moves into combat and returns on actual settlement. GA learned misc synchronizes after each attempted action per accepted M16, including an accepted prefix that later faults or ends in Death; settlement is not another growth. Campfire/upgrade/removal and later entry preserve the surviving permanent instance's learned misc. Combat-local Claw/SteamBarrier/Streamline mutations and powers/orbs reset through accepted shared owners, not an M17 cleanup pass.

Actual boss16 Victory leads to card reward and boss treasure17, real removal children or confirmed skip, then the existing transition: reset transient act state once, ordinary heal once, initialize actual Act2 map/boss order and commit ActAdvanced once. At `(act,floor)=(2,17)`, `Act2Ready` has terminal None, no legal actions, no success/readiness reward, `terminated=false`, `truncated=false`. Preserve real floor17 credit; do not erase it when asserting zero readiness bonus. No fabricated Win/HeartVictory/Act3NoKeys terminal. Core rejects further room actions as InvalidAction before mutation; native step reports `UnsupportedBoundary("M17 stops at Act2Ready")` while decision/reset/close remain healthy.

## Proposed interfaces and the M18 handoff

All additions below are **proposed**, to be bound to actual accepted predecessor symbols before implementation:

| Owner | Additive contract |
| --- | --- |
| Core Run | `RoomScenario::M17DefectAct1`; unchanged `Run::new_room_scenario(config: RunConfig, profile: CreationProfile, rng: RunRng, scenario: RoomScenario) -> Result<Run, RunError>`, Defect/A0/short-Neow validation. |
| Finite data | New `run/rooms/m17_content.rs` with `M17_DEFECT_ACT1_{CARDS,RELICS,POTIONS}` exported through `run/mod.rs`; `reward_cards(boss: bool, local_visit: u64) -> Vec<CardId>`, `combat_potion(local_visit: u64) -> PotionId`, `shop_cards(later: bool) -> [CardId;3]`, `elite_relic(start: usize, owned: &[RelicId]) -> Result<(RelicId,usize),RunError>`. Data selection only, no mechanics/loop copy. |
| Native profile | `RunProfile::M17DefectAct1`; `RunSession::new_m17(config: RunSessionConfig) -> Result<RunSession,RunSessionError>` and `RunBatch::new_m17(configs: Vec<RunSessionConfig>, workers: usize) -> Result<RunBatch,RunBatchError>` through existing `with_profile`/`new_with`. |
| Boss configuration | One existing `RunSessionConfig.boss_history` field, with `RunBossHistory::M17DefectAct1 { act1: RunAct1BossTarget }`. Exact JSON example: `"boss_history":{"m17-defect-act1":{"act1":"guardian"}}`; targets `guardian`, `hexaghost`, `slime_boss`. Reject missing/duplicate/unknown fields, unknown targets and cross-profile history before construction/reset. Preserve old strings and the `M9BossHistory` alias. |
| Observation | `m17_run_observation_spec(caps: &RunCapacities) -> Result<RunObservationSpec,RunSessionError>`; `encode_run_m17(decision: &PublicDecision, map: Option<&PublicMap>, caps: &RunCapacities) -> Result<RunObservation,RunSessionError>`, using `RunEncodingProfile::M17Defect` and one `encode_run_with_schema`. Compose accepted M16 Defect state/value groups with shared Run room/map groups; do not copy a schema body. |

IDs: profile `M17DefectAct1`; observation `spire-gym-m17-defect-act1-observation-v1`; replay `spire-gym-m17-defect-act1-replay-v1`; rules `spire-gym-native-m17-rules-v1`; content `spire-gym-m17-defect-a0-act1-captured-v1`. Reward/seed/RNG IDs remain shared. No generic ordinal-v4 collision or new dispatch/session token format.

Adding Rust enum variants requires downstream exhaustive matches to add arms, as in accepted M12; this is not a frozen Rust ABI or unchanged exhaustive-match source compatibility. Existing variants, spellings and behavior remain intact. The new M17 history is profile-bound; it does not reinterpret M12's legacy Act1 strings.

The M17 encoder exposes every admitted card's real base/upgraded/mutable visible values, learned signed misc, four orb kinds, exact occupied/empty positions through10, Dark stored amount, signed Focus, energy, visible counters/powers, statuses, intents, selections and room choices. Reuse M16's empty0/lightning1/frost2/dark3/plasma4 vocabulary and signed-int32-in-int64 value domain. Known empty slots differ from unused padding; non-Dark damage is unknown, not a fake Dark0. Default `RunCapacities::m10_default()` already provides orbs10. An undersized group returns a capacity fault, never truncation, clamping or paging. Candidate row i remains `PublicDecision.candidates[i]`; mask length/action space are `caps.actions`, entries1 for candidates and0 for padding. Reuse shared row-reference and descriptor packagers.

Public views omit hidden draw order, future offers/AI/RNG, raw resolver/family identities and private journal ordinals. Core semantic actions/reproduction may retain the existing identity handles; the numeric policy observation uses rows. Foreign content/profile/terminal combinations reject explicitly. M17 uses the full Run combat view; existing Ironclad sidecars remain unchanged and `combat_view_id` is null. Native-only delivery adds no Python facade/extension claim.

M18 will start its own canonical Run with initial composite history `{act1,act2}` and reuse these exact Act1 data helpers plus shared owners, then allow the existing transition to continue into Act2. It does not resume/convert a stopped M17 session or invoke M16's authored constructor. For equal seed/Act1 history and caller actions, prefix resources/choices/records/misc/counters must agree through boss chest and heal. Explicit profile/schema identity, the initial Act2 history override, the resulting successor boss/map, and Map-versus-Act2Ready continuation are intentional differences. M17 does not consume M18's proposed API or authorize it. M19 planning confirmed it will consume accepted M17/M18 prefix owners without duplicating them; M21's authored Watcher profile has no overlapping Run progression owner.

## Proposed evidence and honest outcomes

Before prescribed outcomes, retain and SHA-256 hash the deterministic **public-only** caller, all support inputs (including reused routing), action budget and six-case manifest. The caller reads only current `PublicDecision`, `PublicMap`, legal candidates and its own prior public trace. Fix the policy in the plan before execution; no hidden inspection, seed search, automatic combat result or injected later resources. Budget: 10,000 accepted actions per case.

The current `examples/m13_support/public_routes_v2.rs` cannot be imported wholesale into M17: its `cfg(test)` block calls the enclosing M13/M14 caller. Propose the narrow production-only owner `crates/spire-core/examples/shared/public_routes.rs`, with `pub(crate) fn action(decision: &PublicDecision, map: &PublicMap, elite_reached: bool, ordinary_reached: bool) -> Result<GameAction,String>` and the same signature for `action_m14`. Relocate the existing production search, ranking and weights without changing behavior; keep the existing M13/M14 caller-dependent tests and assertions under `m13_support/public_routes_v2.rs`, with its current `pub(super)` entry points forwarding to the shared functions. M17 imports only the production owner. No routing copy, fake M14 caller shim, suppressed tests or new framework. If accepted upstream source already extracted this boundary, bind its exact path/API instead. The implementation plan includes later focused old/new consumer checks; no source extraction or compilation occurs during this documentation correction. Root records the eventual shared-helper mapping for approved M18 downstream reconciliation without editing that approved pair.

| Seed | Initial Act1 boss targets |
| --- | --- |
| 7 | Guardian, Hexaghost, SlimeBoss |
| 11 | Guardian, Hexaghost, SlimeBoss |

Proposed acceptance gate: globally across these six retained canonical runs, actual entry plus same-visit combat record for **all three Act1 bosses**, at least one actual ordinary and elite entry, and **at least one** actual boss Victory through rewards/chest/children/heal to `Act2Ready`. Do not require six wins or all three victories. Unvisited bosses remain unmet coverage; six genuine deaths with no completion leave completion unmet. This gate is a proposal, not an accepted amendment or promised outcome. M14's supplied-start amendment does not transfer.

Retain every genuine Death and its transcript. Caller budget exhaustion, stock exhaustion, capacity/delivery/core faults, EmptyDeck and healthy cutoff are distinct outcomes. Separate nonattacking controls at both seeds establish Death; budgets0/1 establish nonterminal caller stops. Supplied room/combat diagnostics establish mechanics and custody only, never normal-start traversal. A correctness fix needs a named source-grounded regression, old evidence retention and rerun of the fixed six cases. Policy/content/seed changes after outcomes require actual primary-user authorization; a loss is not evidence of a simulator bug.

Compare all six traces stepwise across core, native single, portable semantic replay and ordered batches with workers1/2, including every reward/terminal/map and visible orb/misc value. For full traces of differing lengths, run each fixed case in two identically configured/seeded lanes through its own ending; a separate heterogeneous-prefix check stops when an earlier member reaches Death/cutoff and verifies rejected further step preflight leaves every peer unchanged. Use existing full-batch `reset(&[Option<u64>])`: one entry per slot, every slot resets, and `None` selects a seed rather than an untouched slot. Check full reset restores each slot independently and rejected reset preflight preserves all slots; add no selective-reset API. Diagnostics cover all12 setup kinds, all28/14/5 stock wiring, mutation persistence, full belt, stock/counter exhaustion, nested boss children, accepted-prefix delivery fault, invalid/stale actions, malformed history, capacity errors, cutoff/reset and old-profile preservation. Focused TDD and one final stable native matrix are sufficient; no new verification framework or per-task review swarm.

## Review and preservation

The [implementation plan](../plans/2026-10-27-m17-defect-act1.md) defines cohesive work and five Review Focus classes. Planning checks links, paths, proposed commands and whitespace including untracked files; runtime suites do not validate prose. No builds, episodes, installs, staging, commits, push, PR or cleanup are authorized now.

Preserve continuous decisions, approved inputs, named source facts and immutable candidate copies with SHA/mode manifests at `~/.local/share/spire-gym/evidence/m17-planning-20261027/`, independently of Paseo. Notify root with exact frozen identities **before** root dispatches one independent planning reviewer. This author does not review its own package or dispatch a reviewer. Later authorized implementation needs one fresh independent final milestone reviewer, distinct from author/facts/planning reviewer, on frozen source, plus root verification/actual acceptance. Native source acceptance does not establish a matching Python build/import, original-runtime fidelity, M18 completion or M20 coverage.
