# M11 — representative Silent A0 combat (accepted profile)

> **Accepted finite profile.** Tasks 1–8 are complete. One fresh independent GPT-6 Luna MAX review (`m11_milestone_review`; evidence: `.superpowers/sdd/2026-10-25-m11-silent-combat-mechanics/task8-independent-review.md`) returned no unresolved findings, and root accepted M11 at the scope declared here.

## Scope

M11 adds a finite representative slice of Silent A0 combat mechanics to the shared Rust engine. It does not provide a Silent Gym profile or an ordinary Neow/map/Act 1 episode. The supported content in this accepted profile is limited to the cards, powers, item behavior, and authored-combat bridge described below. Earlier M1–M10 profiles and their consumers remain in force.

The accepted Task 7 bridge starts from the public Silent A0 resources produced by Run::new: 70/70 HP, 99 gold, five StrikeGreen, five DefendGreen, Survivor, Neutralize, Ring of the Snake, three empty potion slots, and no keys. It transfers those exact unupgraded instances and resources through the existing authored-combat constructor, then enters a real owned Jaw Worm combat. Run::new exposes the Neow decision, but the authored bridge does not select Neow, traverse a map, or play Act 1. This is a canonical-resource authored combat, not a normal-start episode or Silent Gym profile.

Ring of the Snake already owns the opening draw effect. Its existing CombatStarted hook adds two cards after the five-card base draw, producing seven cards. M11 reuses and characterizes that behavior; it does not add another draw handler. Snecko Skull is exercised only as an explicitly owned item in a supplied native combat, which does not establish ordinary acquisition.

## Finite mechanics slice

The following table is the finite card/power/item slice accepted in Tasks 2–6. The Task 7 entry and demonstration evidence is separate below.

| Mechanic | Base and upgraded boundary | Status and limit |
|---|---|---|
| Deadly Poison | Cost 1; applies Poison 5 → 7. | Accepted. A separate eligible application can be blocked by Artifact. |
| Poisoned Stab | Cost 1; damage 6 → 8, then Poison 3 → 4 to the captured target. | Accepted. If the hit kills the target, the following Poison application observes the real dead-target guard. |
| Catalyst | Cost 1 and Exhaust; at its action checkpoint, add the current Poison once → twice that amount to the existing stack, for totals of 2× → 3× before other application modifiers. With no current Poison, it makes no application. | Accepted. Catalyst samples at action resolution, after preparation/payment. The Poison application path remains the owner; there is no direct stack multiplication shortcut. |
| Noxious Fumes | Cost 1 Power; applies Poison 2 → 3 to eligible live enemies at the player's post-draw start-turn checkpoint. | Accepted. This is distinct from enemy-start Poison ticks. Its played-power disposition is retained. |
| Poison tick | At the afflicted enemy's start-turn, capture the HP-loss amount from its then-current Poison; resolve that HP loss, then decrement the current Poison stack by one if it remains. | Accepted. The queued amount is not resampled at delivery; changing or removing current Poison does not erase captured damage. No general lethal-effect ordering is claimed. |
| Snecko Skull | Adds one Poison per eligible application, not per Poison point or tick. An Artifact-prevented application consumes one Artifact charge and leaves the existing Poison stack unchanged. | Accepted only as a supplied owned-item case; ordinary acquisition is outside this profile. This states the resulting application/stack state and makes no claim about whether the Skull callback runs on a prevented application. |
| Survivor / Dagger Throw | Survivor costs 1, gains Block 8 → 11, then selects one discard. Dagger Throw costs 1, deals 9 → 12, draws one, then selects one discard. | Accepted selected manual-discard routes only. |
| Reflex / Tactician | Unplayable cards. A manual discard queues Reflex draw 2 → 3 at the bottom of the action queue, or Tactician energy 1 → 2 at the top of the action queue. | Accepted. End-turn cleanup and exhaust are not manual discard. This does not claim every discard card or all/random discard ordering. |
| Cloak and Dagger / Blade Dance / Shiv | Cloak and Dagger costs 1, gains Block 6, and creates one → two ordinary Shivs. Blade Dance costs 1 and creates three → four ordinary Shivs. Shiv costs 0, deals 4 → 6 by its own upgrade, and Exhausts. Upgrading a generator does not upgrade its Shivs. | Accepted. Shivs are distinct combat-local instances, not permanent deck gains. |
| Accuracy | Cost 1; its power amount is 4 → 6 and is added to each Shiv’s own base damage, 4 → 6. Existing Shiv identities in hand, draw, discard, and exhaust are rewritten when the power is installed or stacked; later Shiv constructors use the current accumulated Accuracy amount. | Accepted. The actual CardDrawn callback rewrites only Shivs currently in hand; Shivs remaining in draw/discard/exhaust are not touched by that callback. A qualified manual discard rewrites remaining hand Shivs; end-turn cleanup does not invoke that callback. Accuracy does not add damage to every zero-cost attack. |
| Well-Laid Plans | Cost 1; retain limit 1 → 2. An optional typed end-turn selection retains selected non-Ethereal instances; Ethereal cards may be selected but still Exhaust. | Accepted. Uses the existing EndTurn parent and exact selected instances. No universal end-turn ordering, Runic Pyramid, Equilibrium, or Establishment claim is included. An enemy target is rejected before payment or mutation. |
| Nightmare | Cost 3 → 2 and Exhaust; capture an immutable stat-equivalent template at cast, reset temporary turn cost to combat cost, and preserve captured stats/upgrades and free-to-play. Each cast owns an independent installation and requests exactly three copies at the qualified next player turn-start in both versions. | Accepted in Task 6. Replaying the same physical Nightmare through supported Exhume creates a distinct installation and delivery cursor. Copy/removal actions precede ordinary draw in the source-qualified living-player start path; each copy commits to hand or overflow discard before draw selection. Accuracy's hand callback runs only on hand commit. |

The new slice does not claim the complete ordinary Silent card pool, all Silent-exclusive relics or potions, full card/item offer or acquisition pools, all shared event branches, or milestone acceptance for other characters. This is an M11 acceptance boundary; it does not imply that shared mechanics are character-gated. Backflip, Deflect, Quick Slash, and existing starter behavior remain part of the previously supported shared engine; their presence does not expand this M11 roster.


### Source-corrected historical regression details

The original test names were preserved, while two source-inaccurate details were corrected. In `catalyst_adds_twice_the_current_poison_and_exhausts_but_absence_is_noop`, base Catalyst adds the current Poison amount once (total 2×); upgraded Catalyst adds it twice (total 3×). The base oracle is therefore Poison 4 → 8 and 6,000 → 12,000; the upgraded rows retain 4 → 12 and add 6,000 → 18,000. This follows the [Silent mechanics report](../research/2026-10-19-act1-content-silent.md), §§3–4 and 9: the 9,999 constructor cap applies to a new Poison object, while stacking onto an existing Poison uses the raw action amount without that constructor cap. The Catalyst test fixture now constructs a real permanent Catalyst with its actual Exhaust default instead of changing another card's identity/flags. The independent `catalyst_permanent_card_state_carries_its_exhaust_flag` regression still checks the factory default. These are source-grounded corrections, not removed or weakened regressions.


### Nightmare turn-start and Accuracy hand boundary

The bounded local static lookup supports this order for the normal living-player turn-start path: Nightmare's copy/removal actions enter the queue before ordinary draw, and the creation action commits each copy to hand or overflow discard before draw selection. Task 6's accepted native tests exercise the scoped delivery owner and its per-installation cursor. The original-source grade remains static/decompiler and bytecode evidence only; there is no live-game execution, frame-timing, or generalized ordering claim for other start-turn powers.

A successful generated-card hand commit invokes the Accuracy-only hand refresh. It creates no ordinary CardDrawn, CardDiscarded, or reveal event. Overflow to discard does not invoke the hand callback. The captured template is an independent stat-equivalent snapshot; each cast has its own installation even when the same physical Nightmare is replayed through supported Exhume. Faults preserve committed children and the per-installation cursor, and generated children do not become permanent gains. State comparison includes temporary-retain membership and the Nightmare installation counter.

The public `nightmare_capture` report purpose represents the real capture choice. Frozen M10 v2 rejects the Nightmare card with `Input("unsupported CardId")`; a separate codec-only fixture verifies fail-closed rejection of the unsupported capture-selection operation before encoding; M11 adds no M10 observation or replay schema. These are consumer-boundary checks, not a new M11 ABI.

## Canonical-resource entry and Run ownership

Task 7 extends the existing authored-combat boundary for Silent A0 by validating every supplied initial resource against Run::new and the shared Catalog: HP/max HP, gold, ordered deck identities and upgrades, relics, empty potion slots, and relic counters. Keys remain false; no keys override or duplicate starter initializer was added. Existing Ironclad M3 authored cases and foreign-character rejection remain supported. The starter restriction applies at construction only, so evolved resources remain valid at later combat entry. Focused checks confirm the public Neow choice remains unselected, the opening hand contains seven cards from the five-card base draw plus Ring's existing draw-two hook, public legal combat actions are available, and the relic/potions have one combat inventory owner. Settlement returns the same permanent card identities and the Ring exactly once.

The fixed-seed starter example reads the public Neow decision, transfers those exact canonical resources into the authored bridge, enters a real Jaw Worm encounter, and follows the first currently legal action. It reaches Victory at seed 7 after 37 accepted actions and seed 11 after 21. The ordinary Run settlement path records the real combat result; a second combat after Victory confirms entry does not restore starter HP. This does not exercise ordinary Neow selection, map generation, Act 1 progression, or acquired-card progression.


## Evidence and source limits

The final Task 8 preservation matrix ran on the 622-file integrated candidate. The workspace had 1,476 passed and one failed; the only failure was the retained baseline test combat::tests::full_enemies::blue_slaver_and_two_fungi_encounters_construct_real_actors_across_ascensions. The core library target had 1,217 passed and one failed. These are the same Blue Slaver/Two Fungi encounter-setup failure; every other workspace target passed.

Across the 29 matrix invocations, seven were native checks: the workspace plus M11 Silent 62/0, M10 453/0, M9 32/0, M8 115/0, M7 16/0, and M6 95/0. The other 22 were example/CLI invocations, all matching expected results, including two all-scenario M11 seed invocations and three expected-error controls. The M9/M10 fixed-seed outcomes and settlement prefixes matched accepted M10 logs exactly. Source stayed stable during the matrix and diff check passed. No Python extension refresh was performed; M11 makes no fresh Python-delivery claim. The single independent review completed as `m11_milestone_review` (`.superpowers/sdd/2026-10-25-m11-silent-combat-mechanics/task8-independent-review.md`) with no unresolved findings.
The six historical Poison/Card test names remain and now pass. This does not claim that every original test body is byte-for-byte unchanged: the Catalyst numeric oracle and permanent-card fixture received documented source corrections above. No test was deleted, skipped, or weakened.

Fault and invalid-action coverage retains the real owning continuation in the named cases: captured Poison tick damage and current-stack decrement; dead-target guards and ordinary self-HP-loss tick provenance; committed manual-discard reactions; generated Shiv insertion; typed retain/end-turn parent and selected instances; independent Nightmare installation cursors and already-created children. Preflight-invalid target/selection requests remain atomic. These cases do not establish universal callback, terminal, or lethal-effect ordering. The retained Blue Slaver/Two Fungi setup failure is not addressed by this slice. A preexisting victory-heal HP checkpoint reporting defect is separately tracked in `.66`; M11 does not change that path or claim general pre-healing checkpoint correctness.

The [card](../../data/cards/README.md), [power](../../data/powers/README.md), and [relic](../../data/relics/README.md) datasets and the [Silent mechanics report](../research/2026-10-19-act1-content-silent.md) are implementation references at their documented grades. The report is an author draft with explicit composition limits; community reference snapshots are not runtime registries or original-game validation. The bounded Nightmare source lookup supports only the named living-player turn-start and hand/discard commit chains at static/decompiler and bytecode grade. The owned source build/channel is unverified. No original-game execution, Java parity, or exhaustive transitive source clearance is claimed. The reference guides retain their own source attribution and licensing details.

## Entry, outcomes, and exclusions

The runnable native example separates the canonical-resource starter combat from five supplied-card diagnostic combats. The supplied rows do not represent ordinary card acquisition. Both fixed seeds use the same requested scenarios; no seed search or policy tuning was performed. The caller's starter action rule is deliberately simple and is not a strategy-quality result.

| Scenario | Seed 7 | Seed 11 | Witness |
|---|---:|---:|---|
| Canonical Silent starter / authored Jaw Worm | Victory, 37 actions | Victory, 21 actions | Exact Run::new Silent A0 resources enter owned combat; this bypasses Neow selection, map generation, and Act 1. |
| Supplied Poison | Victory, 10 actions | Victory, 10 actions | Noxious Fumes and Poisoned Stab/Catalyst produce public Poison 18. |
| Supplied discard | Victory, 22 actions | Victory, 23 actions | Dagger Throw selects the actual Reflex: public hand 5→6 after one card is discarded and two drawn. Survivor selects the actual Tactician; Block is 8 and energy changes 1→2 after Survivor's payment. |
| Supplied Shiv | Victory, 13 actions | Victory, 13 actions | Accuracy modifies a real Blade Dance Shiv to 8 damage; the same instance deals 8 and Exhausts. |
| Supplied retain | Victory, 17 actions | Victory, 28 actions | Well-Laid Plans retains the same Strike instance in hand on turn 2. |
| Supplied Nightmare copy | Victory, 14 actions | Victory, 14 actions | Nightmare exhausts with zero immediate copies; turn 2 shows three distinct CombatLocal StrikeGreen copies, and one is played for 6 combined HP/Block damage. |

A genuine native Victory or Death is a valid combat outcome. These outcomes do not establish learned strength or full-run success. Budget exhaustion, invalid arguments, and operational faults remain distinguishable from Death. The authored bridge records a real result through existing settlement and preserves evolved state at second-combat entry; it is not Neow choice, map traversal, Act 1 progression, RunSession/replay, or a Gym profile.

M11 adds no Python facade, native numeric profile, extension refresh, or M11 replay schema. It preserves the existing M9/M10 public contracts and existing source IDs. A default-feature workspace build is not Python-delivery proof.
## Later milestones

M12 owns Silent from-start Act 1 progression, Neow consequences, route/reward/transition behavior, and real boss alternatives. M13 extends from-start coverage through Act 2; M14 through Act 3 and its boss interactions; M15 covers Act 4/Heart from normal start and requires complete A0 Silent card/exclusive-item/event coverage or an explicit blocker. None of that progression is included in the accepted M11 scope. The user-prioritized Gym interoperability implementation `.66` proceeds in parallel in an isolated worktree; M11 constrains final integration only. M12 remains planning-only and is not the automatic next product step; no M12 product work is authorized.

## Completion status

Tasks 1–8 and the final preservation matrix are complete. M11 is accepted at the finite representative Silent A0 scope after review `m11_milestone_review` (`.superpowers/sdd/2026-10-25-m11-silent-combat-mechanics/task8-independent-review.md`). This profile does not claim Neow/map/Act 1 progression, a Silent Gym profile, full standard pools, Python delivery, or original-game parity.
