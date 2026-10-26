# M12 Silent through Act1 design

Date: 2026-10-26. Status: **brainstorming/design draft for fresh planning and builder review; product implementation is held**. This is not an M12 acceptance or a claim that the partial implementation is complete.

## Intent and established decisions

The user wants the approved roadmap to continue, with Silent using Ironclad's established progression rather than copied room/act/combat code. M12 delivers a usable normal-start Silent A0 path through Act1, relevant character content, real boss alternatives, genuine loss/reward/transition behavior, and the corresponding public caller/replay. The milestone is part of the existing simulator, not a second Silent simulator or a training project.

The approved [roadmap](2026-10-22-narrow-gym-milestones-design.md) already chooses one authoritative Rust simulation, thin consumers, reuse of acts/encounters/run coordination, finite honest profiles, and content delivered alongside each playable character/act. The user reinforced “I don't want tons of copy.paste.” Those are user decisions. The finite rows, profile names and adapter signatures below are the executor's proposals within those constraints, not user-selected individual card/item choices.

Latest workflow steering requires brainstorming a plan for **each** milestone, then having it reviewed. Earlier permission to begin M12 immediately is superseded at this workflow gate. Product edits have stopped; source, evidence and build outputs are preserved. Root prioritizes accepting and merging Gym66 into main before M12. This document and its implementation plan receive a fresh independent planning/builder review under root; implementation resumes only after root releases the reviewed planning gate. One later independent milestone review remains a separate gate.

## What exists and what is missing

Accepted M11 is the immutable 622-file snapshot rooted at `ca5a984ac117d4ff6e8079b444c60fa58a3c5075594f8ee62ca9e6d31275515a`, materialized with all SHA-256 hashes and modes verified. The original committed M10 607-file proof is preserved. One fresh baseline workspace run returned exactly 1,476 passed and the sole historical Blue Slaver setup failure (core 1,217/1); no other workspace target failed.

`Run::new` and `Catalog` already own Silent's canonical 70/70 HP, 99 gold, five StrikeGreen, five DefendGreen, Survivor, Neutralize, Ring of the Snake, three empty potion slots and no keys. M11's shared card handlers own Poison/discard/Shiv/retain/Nightmare. M11's authored combat bridge bypasses Neow/map traversal and does not prove M12 progression.

The existing `RoomScenario` path owns short Neow, generated topology, map/room entry, actual encounter setup, combat custody/settlement, rewards, nested choices, shops/events/rest/treasure, boss chest and ordinary successor healing. Existing profile gates and offer rows are Ironclad-specific. M12 needs character/profile validation and finite eligible data hooks, green permanent gain/upgrade capability, and a public observation/replay identity able to represent every reachable advertised state. The existing v1/v2 public codecs intentionally reject M11 retain/Nightmare states; adding green card IDs alone is insufficient.

Gym66 is independently adding public Run lifecycle metadata, an opt-in M10 policy vocabulary and a legitimate pre-heal checkpoint fix. Its mutable tree is navigation input only. The post66 accepted immutable snapshot must be reconciled through root before M12's final integration/review; M12 does not implement a duplicate checkpoint, training session or policy transport.

## Approaches and tradeoffs

| Approach | What changes | Tradeoff and disposition |
|---|---|---|
| A. New finite Silent profile on current owners | Add `M12SilentAct1`, narrow green/item offer data and capability hooks; extend the existing session/encoder/replay/batch through profile parameters | Recommended. Small changes are traceable to the runnable milestone; preserves accepted endpoints and makes reuse explicit. Requires deliberate vocabulary/cutoff coverage. |
| B. Introduce a general public progression/profile framework now | Replace current profile helpers with a new generic hierarchy and migrate M4–M10 before adding Silent | Could consolidate future characters, but adds speculative interfaces and broad preservation risk beyond M12. Defer abstraction until concrete additional consumers require it. |
| C. Expose only a native core example until M15 | Add Silent room entry but postpone public numeric replay/session integration | Less immediate adapter work, but fails the roadmap's evolving public caller/reset/replay direction and hides encoding gaps until later. Reject for M12. |

The user already chose the shared engine architecture; no choice of a separate resolver remains open. Approach A and the specific finite profile are proposed for planning review. No substantive unanswered user preference currently changes the outcome. Root can resolve routine builder questions from the approved roadmap and supplied execution method; new product scope questions go back to the user.

## Scope and anti-goals

Success requires normal Silent creation followed by an actual short-Neow choice, generated map traversal, actual ordinary/elite/boss combat, room consequences, rewards, boss relic acquisition or explicit skip, and same-player Act2 initialization. Every Act1 boss alternative must have a reproducible from-start encounter transcript across the declared fixed runs; diagnostic boss fixtures cannot substitute for that. Genuine Death is valid and must remain distinguishable from action-budget exhaustion, operational faults and the Act2Ready profile cutoff. No policy-quality or guaranteed-win gate is added.

Normal-start evidence may not inject cards, keys, resources or later room state, change enemy values, search for easier seeds, or tune a policy to manufacture a win. Focused supplied-boundary fixtures remain useful for custody and faults and must be labeled as diagnostics. If fixed from-start evidence cannot satisfy the outcome, retain it and report the exact unresolved requirement rather than claiming acceptance from fixtures.

M12 does not implement complete standard green pools, all Neow/events, all Silent-exclusive items, Acts2–4 gameplay, higher ascension, a second training environment, a Python transport fork or an extension refresh. M15 must close complete Silent A0 coverage or expose an explicit completion blocker. Existing M1–M10 and Gym66 profiles, identities and endpoints remain intact.

## Finite content proposal and source boundaries

The product profile is named `M12SilentAct1` and explicitly described as **captured**, not a standard random Silent pool. All entries have existing actual handlers; Catalog eligibility/rarity alone is not runtime proof. Source inputs retain the grades in the [data guide](../../../data/README.md), domain guides/records, [accepted M11 profile](../../milestones/m11-silent-combat.md), and [qualified Silent report](../../research/2026-10-19-act1-content-silent.md). The ignored `content-facts.md` records the bounded reference closure; there is no new original-game execution or PRNG parity claim.

| Producer | Declared content |
|---|---|
| Starter | Catalog's four basic kinds and Ring; no duplicate starter initializer |
| Short Neow | Max HP +7 through ordinary max-HP/heal owner, or actual Neow's Lament acquisition |
| Ordinary captured card reward | Rotate `[PoisonedStab, Backflip, BladeDance]`, `[DeadlyPoison, CloakAndDagger, DaggerThrow]`, `[NoxiousFumes, Accuracy, WellLaidPlans]` by zero-based combat visit modulo 3 |
| Boss captured card reward | Nightmare plus Skip; only one currently supported green Rare, explicitly not three distinct rare offers |
| First/later shop cards | `[QuickSlash, Reflex, Catalyst]` / `[Deflect, Tactician, Nightmare]`, fixed prices `[50,75,75]` / `[50,75,150]` |
| Captured elite relic stock | Lantern, Pear, Mango, SneckoSkull, ToyOrnithopter, Strawberry, Anchor, BloodVial; skip already owned; cursor consumes offers, including declined offers; exhaustion explicit |
| Shop relic / potion | First SneckoSkull / DexterityPotion, later BagOfPreparation / FirePotion; fixed relic150/potion50, owned-relic omission and full-belt handling use shared owners |
| Ordinary chest / boss chest | Shared Strawberry/Anchor/BloodVial captures with real Sapphire exclusion; boss EmptyCage/SlaversCollar plus explicit skip/confirmation; no BlackBlood |
| Combat potion capture | `m12_content::combat_potion(visit: u64) -> PotionId` returns `[FirePotion, StrengthPotion, WeakPotion][visit % 3]`; shared `rooms/rewards.rs::prepare_combat` consumes this M12-only hook using the zero-based `CombatRecord.visit`; existing claim/consumption owners remain shared |
| Events | Existing UpgradeShrine/Purifier; no invented Silent-exclusive event |

The four distinct potion IDs are DexterityPotion, FirePotion, StrengthPotion and WeakPotion. `run/tests/m12_silent_act1.rs::m12_actual_combat_rewards_rotate_potions_and_preserve_full_belt_claim` must observe the first three actual Victory reward screens in that Fire/Strength/Weak order, claim each exact potion once, and verify disabled full-belt claim → legal discard → same captured potion claim without duplication. The full-belt branch may use a labeled supplied inventory diagnostic; it is not normal-start proof.

This yields 19 permanent green kinds, 13 possible relic kinds and four potion kinds. SneckoSkull replaces the Ironclad-only MagicFlower in a declared elite capture; this changes rarity tier and must not be presented as a tier-preserving random drop. BlackBlood, Brimstone, MarkOfPain, RunicCube, BloodPotion, Elixir and HeartOfIron are outside these Silent offers. No unsupported cards are silently filtered from a claimed full pool.

Generated Shiv, copied accepted cards and actual Act1 Dazed/Burn/Slimed states must encode. Base/upgraded cards and their runtime values must encode. Powers include existing Act1 enemy/shared/item powers plus actual `poison`, `noxious_fumes`, `accuracy`, `retain_cards`, `nightmare` IDs. Manual-discard, optional retain/Confirm0, Nightmare capture/copy/hand-overflow and played-power/public-playing ownership are reachable and cannot be omitted from the descriptor.

## Shared owners and exact interface proposal

- Core: add `RoomScenario::M12SilentAct1`; `Run::new_room_scenario(config: RunConfig, profile: CreationProfile, rng: RunRng, scenario: RoomScenario) -> Result<Run, RunError>` retains its signature. Only M12 permits Silent A0; all existing scenarios retain Ironclad A0/short-Neow validation.
- `run/rooms/m12_content.rs` provides finite rows and public `M12_SILENT_ACT1_CARDS: &[CardId]`, `M12_SILENT_ACT1_RELICS: &[RelicId]`, `M12_SILENT_ACT1_POTIONS: &[PotionId]`, re-exported from `run/mod.rs`. No new room loop or encounter simulator.
- `mutations::supported_in(&RunState, CardId) -> bool` returns M12's permanent allowlist for M12; use existing `obtain_card`, `upgrade_card`, card laws and selections. It must not admit Ironclad cards through a legacy fallback or offer Shiv as a permanent reward.
- Existing `rooms/rewards.rs`, `shop.rs`, `boss.rs` consume the M12 row hooks. Existing `act1.rs`, topology, enemy setup, combat startup/settlement and `transition.rs` remain authoritative.
- Public: extend the same `RunSession` and `RunBatch` with `new_m12(config: RunSessionConfig) -> Result<RunSession, RunSessionError>` and `new_m12(configs: Vec<RunSessionConfig>, workers: usize) -> Result<RunBatch, RunBatchError>`. Generalize the boss-history enum to `RunBossHistory` with the old three variants plus `Act1GuardianUnseen`, `Act1HexaghostUnseen`, `Act1SlimeBossUnseen`; preserve `M9BossHistory` as a public compatibility alias and old serialized spellings. Validate profile/history pairs before reset or mutation.
- Descriptor/replay: `m12_run_observation_spec(&RunCapacities) -> Result<RunObservationSpec, RunSessionError>` and `encode_run_m12(&PublicDecision, Option<&PublicMap>, &RunCapacities) -> Result<RunObservation, RunSessionError>`. Observation ID `spire-gym-m12-silent-act1-observation-v1`; replay ID `spire-gym-m12-silent-act1-replay-v1`; rules ID `spire-gym-native-m12-rules-v1`; content ID `spire-gym-m12-silent-a0-act1-captured-v1`; profile ID `M12SilentAct1`. Keep Gym66's v3 policy identity distinct.
- Extend Gym66's accepted `encode_run_with_power` seam into a small private finite vocabulary parameter where M12 also needs card/item/profile/selection differences. Keep its power-only wrapper and v1/v2 wrappers stable. Use one row encoder, existing descriptor domain/group helpers and canonical JSON/fingerprint owner, rather than copying encoder/schema-construction bodies. Unknown IDs/operations fail closed for each profile.
- `RunEpisodeRecord`, `replay_run` and the existing record/session/batch ownership remain authoritative. Add M12 provenance and history parsing without creating a parallel replay engine. No new Python classes or long transport copies.

## Cutoff, errors and privacy

Act2Ready occurs after actual Act1 boss rewards, source-owned boss-chest children and ordinary act healing at act2/floor17. It is nonterminal, has no legal actions and adds no readiness bonus. Publish a valid zero action mask; do not convert it into a delivery fault, Death or victory. A new action at that boundary returns `UnsupportedBoundary("M12 stops at Act2Ready")` before mutation and without freezing the session. Reset and close remain available. The bounded caller explicitly returns a cutoff outcome; replay and batch expose the same final decision/mask. Native `RunStepResult.terminated` remains false and `truncated` false at this simulation boundary; caller budget exhaustion is an explicit separate caller result, not a synthetic Run terminal.

Invalid/stale tokens, wrong profile/history, foreign card/character, unaffordable or full-belt choices and exhausted capacities reject before mutation where existing contracts require. Accepted children retain the real owner/current frame if delivery fails; no transaction rewind or blanket cleanup is added. Public observations expose only entitled state, never raw instance IDs, future draw order, hidden AI/RNG, unoffered cards or queue internals. Privileged reproduction remains privileged.

## Execution evidence and review gates

Bind each native scenario before constructing the Run: `Guardian` (and the separate `Death` control) sets initial `CreationProfile.seen_bosses[0] = ["Hexaghost", "SlimeBoss"]`; `Hexaghost` sets `["Guardian", "SlimeBoss"]`; `SlimeBoss` sets `["Guardian", "Hexaghost"]`. `examples/m12_support/mod.rs::act1_seen_bosses(scenario: Scenario) -> Vec<&'static str>` supplies that initial profile to `make_run` before `Run::new_room_scenario`. For the same scenarios, initial RunSession config selects `RunBossHistory::{Act1GuardianUnseen, Act1HexaghostUnseen, Act1SlimeBossUnseen}` respectively; `run_session.rs::create_run` maps that typed input to the same initial creation history before Run construction. Existing `act1::start` chooses the unseen boss and generates the map when the real Neow choice resolves. Never mutate the boss order/icon, generated map, run history or encounter after construction to select a scenario.

Use fixed seed7 and seed11, each with all three actual Act1 boss-history alternatives, plus a nonattacking Death control and bounded invalid/zero/small-budget controls. `crates/spire-core/tests/m12_silent_act1.rs::m12_initial_boss_history_drives_generated_map_and_actual_encounter` must assert for each of the six normal scenario/seed pairs: map absent at initial Neow; after its actual selection, generated public map boss equals the requested Guardian/Hexaghost/SlimeBoss; the normal offered route reaches the corresponding real `CombatRecord.encounter` (and observed boss combat) without state/map overrides. A genuine early Death remains recorded as a valid gameplay outcome but leaves that scenario's boss-entry requirement unmet; it must not be counted as boss coverage or repaired through seed search/injection/tuning. Choose and record the public caller's deterministic rules before those runs, using observed legal actions, deck, current hand, public map and intents only. Do not change those rules to force favorable results. Record encountered enemies/bosses, room/custody/reward transitions, accepted actions, HP, final reason and replay agreement. Real losses need no repair merely because the caller loses.

Five review input classes map to concrete tests in the plan: canonical start/inventory/Neow; finite green/item acquisition and stock; full reachable public selection/power vocabulary with fail-closed old profiles; source-owned boss chest/ordinary-heal/cutoff/fault custody; normal-start boss alternatives/natural loss/budget/replay/batch and preservation.

After the reviewed-plan gate releases: repair and verify preserved partial work incrementally; integrate isolated authors sequentially with explicit preimages; reconcile root-provided accepted post66 snapshot; run one appropriate integrated workspace/preservation/examples matrix. Baseline's sole Blue Slaver failure stays qualified unless the accepted post66 baseline legitimately changes it. Freeze exact full source hashes/modes and materialize a separate snapshot/unique target for ONE fresh Luna MAX milestone reviewer, not an author or the planning reviewer. Root rules declined findings, performs final integration and accepts/closes M12. No commit/push/install/cleanup authority is inferred.

## Preserved partial state

Product hold snapshot: `.superpowers/sdd/2026-10-26-m12-silent-act1/product-hold-manifest.json`, 625 public paths rooted at `e2596873dc275267964d799b56e3c5ad5528b10004b966082a81ca35ba74f79f` (nine core deltas plus the earlier plan relative to accepted M11). Three focused tests passed before more room tests were added. The latest room-test compile failed on helper visibility and a moved test value; it has not been repaired under the hold. Isolated adapter and caller partials are uncompiled/unintegrated and preserved. No normal-start Act1 completion/boss result exists yet. Planning review must evaluate this design independently of partial code; preserved code is not a reason to rubber-stamp a choice.
