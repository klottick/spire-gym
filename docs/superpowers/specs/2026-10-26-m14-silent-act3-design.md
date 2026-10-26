# M14 Silent through Act3 design

Date: 2026-10-26. Status: **proposed design for combined design/plan review and primary-session approval**. This packet authorizes no product work. Planning proceeds alongside M13; implementation requires the approved reviewed M14 plan and accepted M13 source.

## Intent, authority and integration assumptions

The [roadmap](2026-10-22-narrow-gym-milestones-design.md) requires normal-start Silent progression through Acts1–3 and real boss interactions through the public caller. The user has selected the authoritative Rust Run/combat owners and native execution with Luna MAX assistants and one final milestone reviewer. Those architecture/method decisions are established; the finite choices below are proposals, not previously approved requirements.

Planning source is accepted M12 commit `afacd0934c55ceac159e684cfcb3f2565e7cb6b6`, including the existing M8/M9/M10 shared Act3 implementation. M12's accepted matrix was 1,529 passes and one historical Blue Slaver fixture failure; it is dated preservation evidence, not an M14 test result. The approved M13 contracts are inputs, not accepted runtime evidence:

| Contract | SHA-256 |
|---|---|
| `docs/superpowers/specs/2026-10-26-m13-silent-act2-design.md` | `d611d277995b7b7123adc5ac08a97b9b6fb778c9ebf352e3d69ae32729872b99` |
| `docs/superpowers/plans/2026-10-26-m13-silent-act2.md` | `e105b6376f1368d835429d4a87bd0266c24129b4f7e42dac5e658feb85e143a8` |

The M13 pair was read from its independent planning checkout. At implementation start, root reconciles this packet against the accepted M13 revision: shared Silent profile/content gates, Act2-local counters, Wound vocabulary, typed Act1+Act2 history, replay codec and public caller. File-local adaptations with unchanged contracts are routine; a material scope/behavior difference goes to root. This reconciliation is a dependency check, not a duplicate planning/approval gate. Do not touch the active M13 implementation checkout.

Root's planning-time update reports that M13's six frozen normal routes currently all end in genuine Death, only seed7 Guardian→Champ enters its Act2 boss, and none reaches Act3Ready. M13's coverage/completion gates are therefore **unmet**, and root has not accepted M13. This is reported dependency evidence, not an M14 run or permission to tune the caller. Root also reports an in-progress shared Ancient Writing correction for Silent basic-card upgrades; reuse its accepted source/regression rather than adding another event upgrader. Continue planning now; do not execute M14 until the predecessor is accepted and its actual evidence reconciled.

The approved M13 phrase “21 Act2 enemy identities” means21 additional to M12 in its actual proposed table; the reachable Act2 union also includes reused Cultist, for22 Act2 species and37 cumulative through Act2. Preserve the approved document bytes and spell this census correction out in M14's metadata/evidence; no encounter or ID is added by correcting the count.

## Approaches and selected direction

| Approach | Tradeoff and proposed decision |
|---|---|
| A. Opt-in `M14SilentAct3` on shared M13 and M9 owners | Recommended and consistent with the selected architecture. Compose the accepted M13 normal start with actual M9 Act3 rooms/ending, adding finite Silent content dispatch and a distinct native identity. |
| B. Widen M13 or reuse the M9 profile identity | Rejected: changes an accepted predecessor's cutoff, character contract, vocabulary or replay identity. |
| C. Author an Act3 start or a separate Silent progression/encoder | Rejected: fails normal-start evidence and duplicates established owners. |

M14 supplies canonical Silent A0 resources through `Run::new_room_scenario`, real short Neow, generated maps, actual prior boss rewards/children, once-only transitions and shared combat entry/settlement. It continues at the actual Act2→3 initialization at floor34, exposes all captured Act3 edges through rest49 and the selected boss50, then uses the shared Act3 ending. No separate game loop or combat rules are introduced.

## Finite content and eligibility proposal

Acts1–2 retain the approved M13 capture and all predecessor cutoffs. M14 keeps the 19 M12 green card kinds and four potions unchanged. The permanent deck can additionally acquire **Parasite** from actual Writhing Mass work; it is not a reward/shop green card. The complete additional/generated vocabulary is `Shiv`, `Dazed`, `Burn`, `Slimed`, `Wound`, `Void`, `Parasite`: six temporary card kinds plus the permanent-acquisition curse. Shiv is an Attack, not a Status. Awakened One's **Sludge is the `awa_sludge` move creating Void at BottomDraw**, not a separate Sludge CardId.

| Act3 producer | Exact proposed capture |
|---|---|
| Ordinary card rewards | M12's three triples by Act3-local settled combat ordinal modulo3: Poisoned Stab/Backflip/Blade Dance; Deadly Poison/Cloak and Dagger/Dagger Throw; Noxious Fumes/Accuracy/Well-Laid Plans. |
| Potion rewards | Fire/Strength/Weak by the same Act3-local ordinal modulo3. |
| First/later Act3 shop | Act3-local shop visit0: Quick Slash/Reflex/Catalyst at50/75/75, unowned Snecko Skull at150, Dexterity Potion at50. Later: Deflect/Tactician/Nightmare at50/75/150, unowned Bag of Preparation at150, Fire Potion at50. Shared purge price, affordability, ownership and belt rules apply. |
| Elite relics | M12 eight in order (Lantern, Pear, Mango, Snecko Skull, Toy Ornithopter, Strawberry, Anchor, Blood Vial), then Vajra, Oddly Smooth Stone, Bag of Preparation, Bag of Marbles, Bronze Scales. Act3-local cursor skips owned rows. Successful publication commits the next cursor even if the relic is skipped; a failed lookup advances nothing new. |
| Chest43 | M9's first unowned Strawberry/Anchor/Blood Vial, exclusive Sapphire alternative, and Proceed. When all three relics are owned, they are disabled; Proceed remains legal. No fabricated duplicate, refill or new stock. |
| Questions/events | First Falling, second Tomb of Lord Red Mask, then the actual Monster capture. Add these to M13's Neow/Upgrade Shrine/Purifier/Ancient Writing/Joust vocabulary. |
| Final boss50 | Actual settlement and Proceed/dialog only: **no claimable gold/card/potion rewards and no third boss chest**. |

The resulting relic universe is 18 kinds: the 13 M12 kinds plus Vajra, Oddly Smooth Stone, Bag of Marbles, Bronze Scales and Red Mask. Bag of Preparation was already among the 13. All added relic mechanics already have shared native owners; no inert relic rows or new mechanics solely to enlarge this table.

The finite elite list **does not guarantee five unowned Act3 offers for every possible earlier inventory**: Bag of Preparation may already be owned, and earlier acts can consume the base stock. Exhaustion returns `RunError::Unsupported("captured Silent Act3 elite relic sequence exhausted")` through retained accepted-source fault custody, with no duplicate grant or new cursor advance. That route is an operational content boundary, not Death or completion. Keep late edges available; do not prune routes to hide it. M15 must resolve or explicitly block full-pool completion on this limitation.

The shared Act3 ordinary sequence is OrbWalker, ThreeShapes, then the history-filtered cycle SpireGrowth, Transient, FourShapes, Maw, SphereAndTwoShapes, JawWormHorde, ThreeDarklings, WrithingMass. Elites are GiantHead, Nemesis, Reptomancer; bosses are AwakenedOne, TimeEater, DonuAndDeca. These are **16 encounter IDs** (10 ordinary/3 elite/3 boss); JawWormHorde is an earlier entry in `A0_ENCOUNTER_SETUPS`, not in its contiguous Act3 tail. Reuse the shared sampler/setup metadata without another table of gameplay rules.

The actual Act3 setup union is **20 enemy species**: OrbWalker, Repulsor, Exploder, Spiker, SphericGuardian, JawWorm, Darkling, SpireGrowth, Transient, Maw, WrithingMass, GiantHead, Nemesis, Reptomancer, Dagger, Cultist, AwakenedOne, TimeEater, Deca, Donu. Seventeen are new to M13; SphericGuardian/JawWorm/Cultist are reused. Include shape alternatives, all actual Dagger summons and boss partners, not just initially visible actors. Promote M13's three map-only Act3 boss encounter IDs to playable for M14 alone.

Deduplicating those explicit sets gives54 cumulative enemy species (M12's16,21 additional Act2,17 additional Act3). Metadata distinguishes per-act membership from cumulative vocabulary rather than adding overlapping per-act totals.

## Source-grounded interactions and diagnostics

The implementation adds combined Silent/Act3 regressions on actual `enemies::setup` entries. Existing mechanic tests are oracles; their presence does not prove the combinations. Supplied room/combat diagnostics are labelled separately from normal-start evidence.

- **Donu/Deca and Poison:** each actual A0 actor starts with Artifact2. Without Red Mask, two separate Poison applications consume two charges without adding Poison; the first unblocked base Deadly Poison with Snecko Skull applies6. With an actually acquired Red Mask, opening Weak consumes one Artifact on each initial actor, leaving1 and no Weak; one further Poison application consumes the remaining charge. Partner work and final settlement remain shared.
- **Awakened One and Silent Powers:** Noxious Fumes, Accuracy and Well-Laid Plans each trigger A0 Curiosity Strength+1 after their payload. Poison is cleared at first-phase collapse; positive Strength survives, Curiosity is removed, and the same boss revives at300HP. Half-death is not victory; final second-phase death settles once and runs the shared surviving-Cultist escape. Sludge creates Void at the bottom of draw; drawing it loses one available energy through its existing draw hook, not through discard.
- **Time Eater and owned card work:** play eleven ordinary cards, then test a Shiv as card12 and, separately, Nightmare or a manual-discard card with selection as card12. Complete the selected card's payload/disposition exactly once, reset Time Warp to0, add Strength2 and force one ordinary end turn only after owned selections complete. The forced turn honors Well-Laid Plans retention and delivers three distinct temporary Nightmare copies on the next player turn. Haste below half HP cleanses Poison once; no pre-existing Poison is assumed to survive that cleanse.
- **All three elites:** actual Nemesis Intangible caps each positive Shiv hit and Poison tick at1 while active; Poison still decrements through the shared tick. Giant Head Slow counts played cards, including generated Shivs, and modifies subsequent NORMAL damage rather than Poison HP loss. Reptomancer's real Daggers create Wound in discard; select Wound with Survivor/Dagger Throw to prove it moves without Reflex/Tactician listeners, and in separate selections discard Reflex itself for draw2 or base/upgraded Tactician itself for energy1/2. Retain/copy ownership must not convert any generated card into a permanent gain.
- **Writhing Mass and Parasite:** its actual curse move appends one permanent-gain journal entry; the same Run assigns a permanent identity once, including on an accepted later fault and Victory/Death settlement. Acquisition has no max-HP penalty. Permanent purge removes that exact instance and lowers maximum HP by3, floored at1, clamping current HP only when needed; Smith excludes it. A temporary Nightmare copy/exhaustion is not permanent removal.
- **Falling/Tomb and Red Mask:** Falling snapshots one permanent instance per Attack/Skill/Power, presents Skill/Power/Attack, disables absent types, excludes Parasite and removes only the captured duplicate instance. A last-card removal commits EmptyDeck once; a curse-only deck offers Leave. Tomb exchanges all current gold (including0) for unowned Red Mask, or grants222 gold when already owned; no RNG/repeat mutation. Red Mask applies opening Weak only to initial enemies, subject to Artifact, not later summons.

Use the existing [reference index](../../../data/README.md), [M8 static findings](../../../data/enemies/m8-static-findings.md), native `combat/tests/m8_*`/`m11_silent_*`, `run/tests/m8_acquisition.rs`, `m8_events.rs`, and `m9_end.rs` as bounded evidence. The [Act3 ending finding](../../research/2026-10-23-m9-act3-ending.md) is task-local static evidence from an archive with unverified upstream build/channel; native source and tests are separate evidence. No fresh private-source audit or original-game execution is required or claimed.

## Shared interfaces and compatibility

Extend `RoomScenario` and the internal `RunProfile` with `M14SilentAct3`, using the existing character/profile gates in `run/rooms/mod.rs`, shared `act3.rs`, `transition.rs`, `rewards.rs`, `shop.rs` and `act3_end.rs`. Place only finite row selection in `run/rooms/m14_content.rs`; reuse M12/M13 content functions. Do not make M14 satisfy `is_m9` or `is_m13` by aliasing identities. Extend behavior-specific profile gates narrowly, keeping predecessor values/output unchanged.

Retain the one `RunSessionConfig.boss_history` field. Reuse M13's `RunAct1BossTarget`/`RunAct2BossTarget`; add `RunAct3BossTarget { AwakenedOne, TimeEater, DonuAndDeca }` and `RunBossHistory::M14SilentAct3 { act1, act2, act3 }`. M14 accepts only its full triple and installs all three `CreationProfile.seen_bosses` sets **before** Run construction. Accept all27 typed combinations; the evidence matrix is a declared subset. Replay JSON is exactly `"boss_history":{"m14-silent-act3":{"act1":"guardian","act2":"champ","act3":"awakened-one"}}`, with Act3 strings `awakened-one`, `time-eater`, `donu-and-deca`. Reject missing/unknown/extra fields, wrong profiles and old scalar histories under M14 before construction.

All six old M9/M10/M12 history variants, `M9BossHistory` alias, their string spellings, and M13's pair object remain unchanged. Adding Rust enum variants requires downstream exhaustive matches to add arms; this is an explicit additive Rust source-compatibility tradeoff, not wire compatibility loss. Existing combat-v1, M9/M10/M12/M13 schemas, IDs, fingerprints, cutoffs and recorded behavior remain frozen; no Python facade or extension changes are proposed.

Add `RunSession::new_m14(config: RunSessionConfig) -> Result<RunSession, RunSessionError>`, `RunBatch::new_m14(configs: Vec<RunSessionConfig>, workers: usize) -> Result<RunBatch, RunBatchError>`, `m14_run_observation_spec(caps: &RunCapacities) -> Result<RunObservationSpec, RunSessionError>` and `encode_run_m14(decision: &PublicDecision, map: Option<&PublicMap>, caps: &RunCapacities) -> Result<RunObservation, RunSessionError>`. Use the existing session/row encoder/descriptor helpers and `RunBatch::new_with`; add no copied scheduler, encoder or Python loop. The finite descriptor covers every declared card/item/event/enemy/encounter, actual powers/intents/selections and ending choices, rejecting foreign IDs. Only entitled known-draw and selected/copy/retain rows are public; no hidden draw order, queues, RNG or histories. Capacity overflow remains explicit, with no truncated actions.

Like the finite M12/M13 delivery, M14 does not advertise the Ironclad combat-policy sidecar: `combat_view_id` is null, and sidecar spec/view calls return an unsupported-boundary error without freezing healthy state. Full-pool/Gym66 sidecar expansion is outside this profile.

Proposed identities: observation `spire-gym-m14-silent-act3-observation-v1`; replay `spire-gym-m14-silent-act3-replay-v1`; rules `spire-gym-native-m14-rules-v1`; content `spire-gym-m14-silent-a0-acts1-3-captured-v1`; profile `M14SilentAct3`. Reuse the accepted reward, RNG and seed-adapter IDs. Default capacities remain `RunCapacities::m10_default()`; capacity settings are transport limits, not gameplay limits.

## Actual ending and fault contract

On real boss50 Victory, settlement returns inventory and validates census before exposing `Act3BossComplete` with Proceed. Then the actual three dialog choices occur. Missing any key (or disabled Act4) exposes the fourth dialog choice, which commits **`RunTerminal::Act3NoKeys` once**, with terminal contribution0, no HP loss, floor51 credit or HeartVictory bonus. Genuine combat Death and EmptyDeck retain their ordinary −1 terminal contribution. Floor and first-key rewards retain existing semantics.

With all keys and Act4 enabled, follow the M9 core branch: stop after dialog3 at **Act4Ready, act3/floor50**, alive, nonterminal, zero legal actions, with no heal, new act, fourth door choice or readiness bonus. M14's native session publishes this as a healthy inspectable/resettable cutoff; attempted step returns `RunSessionError::UnsupportedBoundary("M14 stops before the Act4 door")` without mutation. This differs only in M14 session handling; do not change M9's accepted unsupported-publication behavior or M10's fourth-choice Act4 transition. All-key coverage here is a **supplied diagnostic**, not normal-start key/Heart proof or M15 closure.

Invalid/stale/cross-session actions reject without mutation. A failure after accepted gameplay retains the actual Run, accepted action, gains/cursors/RNG and first fault; subsequent step/reset remain frozen. Do not roll back, repeat a claim, auto-reset, convert an operational fault into Death, or award completion on budget exhaustion. Replay preserves portable fault metadata and accepted actions; single and batch sessions must agree.

## Fixed normal-start evidence and freeze policy

Use one public-data caller, reusing the accepted M13 policy/loop through a bounded shared helper rather than copying it. Extend only the missing Act3 event/ending/cutoff handling and act-local route tracking. Freeze source hash, public tie-breaks and action budget **before running the following outcomes**; preserve the M13 wrapper's old behavior with transcript equality. Caller decisions may inspect current public map topology, legal choices, cards and intents, never hidden state or future offers. No later-act injection, map/history mutation after construction, enemy/resource modification, seed search, matrix substitution or outcome-driven policy tuning.

| Seed | Act1 target | Act2 target | Act3 target |
|---|---|---|---|
|7|Guardian|Champ|AwakenedOne|
|7|Hexaghost|BronzeAutomaton|TimeEater|
|7|SlimeBoss|Collector|DonuAndDeca|
|11|Guardian|BronzeAutomaton|DonuAndDeca|
|11|Hexaghost|Collector|AwakenedOne|
|11|SlimeBoss|Champ|TimeEater|

Each episode starts70/70HP,99gold, five StrikeGreen/five DefendGreen/Survivor/Neutralize, Ring of the Snake, three empty potion slots and no keys, then chooses real short Neow. Assert actual map icon and real entered/settled boss at each reached act, Act3 ordinary and elite entry, and real selected boss50 entry/record. **All six routes must enter their declared Act3 boss for the proposed matrix gate; at least one must win, resolve the actual Proceed/dialog sequence and reach Act3NoKeys.** These are implementation acceptance gates, not promised outcomes. A natural Death after boss entry is valid loss evidence; Death before entry leaves coverage unmet. If the frozen matrix fails either gate, report M14 incomplete to root without replacing outcomes or using supplied fixtures as normal-start proof.

Record all six outcomes, action counts, entered encounters, rewards, source hashes and caller hash, including genuine losses and finite faults. Separate diagnostics cover all three real boss lifecycles, eight key combinations, nonattacking Death, EmptyDeck, budgets0/1 and invalid/stale actions. Use maximum/default caller budget8192, rejecting8193, without fake termination. Replay compares every public decision/map/action/reward and ending; batch compares single-session results across worker counts1/2 and at least one accepted-fault lane.

## Limits and M15 obligation

This is native Rust Silent A0 finite progression, not the full ordinary reward/shop/event sampler, all Silent cards/exclusive items, Act4/Heart, A1–20, Java RNG/map parity, Python-extension delivery, original-runtime validation or policy-strength evidence. Beat of Death/Invincible belong to Heart, not an invented Act3 requirement. M15 must enumerate every still-missing Silent A0 card, exclusive relic/potion, event and interaction against the cited catalogs and source owners, then implement it or name the exact source/runtime blocker that prevents completion. Finite stock and omitted pools cannot silently count as complete coverage.

## Builder decisions for review

The finite18-relic/19-green-plus-Parasite universe, Act3-local offer order, explicit exhaustion, six triples, ≥1 normal Act3NoKeys gate, all-key supplied-only boundary, versioned IDs and triple JSON are planner proposals requiring primary-session approval. Helper placement and test fixture construction may be settled locally within those contracts. Root dispatches one fresh independent planning reviewer of this exact design/plan pair; implementation follows only after the reviewed plan is approved and M13 source accepted.
