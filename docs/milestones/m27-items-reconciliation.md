# M27 items reconciliation

Status: **source frozen at freeze v5 (2026-11-02), candidate for independent final review.** This
record accompanies the M27 [profile](m27-items.md) and approved
[design](../superpowers/specs/2026-10-31-m27-items-design.md). M27 is not a public profile or a
Python-delivery claim.

## Source binding

The source base is commit db42bf4d7bd7bd0f3e5a8a844fb3e44724d65d2c (PR #68). Freeze v5 is retained in
main-v1/freeze-v5/ with its 1,090-entry tree manifest
ec633c788b49b937029efa27f05713ea153493f3fbe2f964d4b956e3114014c1 and its independent verification in
main-v1/freeze-v5/verification-v1/. Freeze v5 is freeze v4 plus the final-review fixes D5–D16 below: it differs
from freeze v4 in 18 source and test paths and 4 documentation paths (main-v1/freeze-v5/delta-vs-freeze-v4.txt).
Freeze v4 (main-v1/freeze-v4/; tree manifest
f069e1442236cdd965a1c2d40b23898a8d08e79de67e23abb494b5d3eed136aa) is kept as history. Freeze v4 is freeze v3 rebased onto PR #68,
as the PM ordered before the candidate. Freeze v3 was the corrective freeze the PM ordered after
candidate v1 (main-v1/HANDOFF-H7.md, PM RULING): plan Tasks X1 and X2 were executed, and the defects
they found were fixed at the shared owners. The freeze-v4 independent verification is
main-v1/freeze-v4/verification-v1/. Freeze v3 (tree 3eaca55d…, built on a86df922;
main-v1/freeze-v3b-void-prerebase/ and freeze-v3-void/) and freeze v2 (main-v1/freeze-v2/; PM proceed
receipt root-freeze-v2-proceed-v1.json, cba3a952…) are kept as history. Freeze v4 differs from freeze
v3 in 34 paths: the 32 PR #68 paths and the two D4 item files (main-v1/rebase-db42/). Freeze v3
differed from freeze v2 in the 13 paths of main-v1/integration-9/changed.txt. Documentation edits do
not change the frozen source; any other source change invalidates freeze v5. References below to
base a86df922 address pre-M27 original bytes. PR #68 changed none of the six amended accepted tests.

The freeze-v5 evidence reports, on a fresh copy and target, 3,986 passed / 0 failed / 11 ignored in
108 workspace test binaries, and 3,380 / 0 / 5 in 37 binaries with m27-diagnostics enabled. The
m27_ library filter reports 494 / 0 / 0, and 521 / 0 / 0 with m27-diagnostics. The growth from
freeze v4 (3,976, 3,367, 484 and 508) is the 13 final-review tests, three of them m27-diagnostics only. The growth from
freeze v3 (3,958 and 3,354) is PR #68's own tests. The growth from freeze v2 (3,949, 3,341 and 475)
was the nine X1 tests and the four feature-gated X2 tests. The X1
tests are Constructed; the X2 tests are Supplied played (design §15.1). The evidence makes no Normal
played or Native-agreement claim. Build and receipt details are in main-v1/freeze-v5/INVOCATIONS.md.

## Global corrections (G1–G10)

The regression tests below are Constructed evidence (design §15.1). Original source bytes for
unamended corrections remain addressable at pre-M27 base a86df922. Where an accepted assertion
changed, its separate retained copy is named below and again in the amended-assertion table.

**G1 — Sozu master energy.** Change: count Sozu in the shared +1 master-energy law, with the
other eligible energy relics. Shared owners:
[items/laws.rs](../../crates/spire-core/src/items/laws.rs) and
[combat/engine.rs](../../crates/spire-core/src/combat/engine.rs), with the per-relic rule in
[accepted_law.rs](../../crates/spire-core/src/items/relic_effects/accepted_law.rs). Reach: accepted
M15 and M20 combat-item contexts that admit Sozu; no finite Run union owns Sozu. The old law fails
[m27_g1_sozu_energy_was_missing](../../crates/spire-core/src/items/tests/m27_energy_law.rs#L56);
the amended accepted M15 Alchemize assertion is at
[m15_generation.rs:251](../../crates/spire-core/src/combat/tests/m15_generation.rs#L251).
Its original bytes are main-v1/a22-energy/original/crates/spire-core/src/combat/tests/m15_generation.rs.

**G2 — Sozu blocks potion claims.** Change: reward and shop owners must route potion claims through
the Sozu obtain rule. Shared owners:
[rewards.rs](../../crates/spire-core/src/run/rooms/rewards.rs),
[shop.rs](../../crates/spire-core/src/run/rooms/shop.rs), and
[items/mod.rs](../../crates/spire-core/src/items/mod.rs). Reach: M27 Complete-item Run reward and
shop producers; existing finite Run profiles do not own Sozu. The old direct-slot writes fail the
Constructed [shop](../../crates/spire-core/src/run/tests/m27_potion_shop.rs#L42) and
[reward](../../crates/spire-core/src/run/tests/m27_potion_rewards.rs#L80) owner tests. No accepted
assertion was amended; original bytes are at base a86df922.

**G3 — Potion Belt equip.** Change: equipping Potion Belt appends two empty slots once, rather
than relying on tests or callers to construct a five-slot belt. Shared owner:
[items/lifecycle.rs](../../crates/spire-core/src/items/lifecycle.rs). Reach: complete-item Run
acquisition/equip; no frozen profile Run union owns Potion Belt. The old equip law fails
[m27_obtain_equip_resource_laws](../../crates/spire-core/src/run/tests/m27_relic_lifecycle.rs#L161).
No accepted assertion was amended; original bytes are at base a86df922.

**G4 — relic visit order.** Change: one acquisition-order dispatcher visits relics in every
context, preserving explicit top-queued work ordering. Shared owner:
[items/dispatch.rs](../../crates/spire-core/src/items/dispatch.rs). Reach: all profiles pass
through the shared dispatcher; M15 Silent has the only observed frozen combat-view changes, with
Twisted Funnel before Bag of Marbles or Red Mask. The old module-order loops fail
[m27_dispatch_cross_path_order_follows_acquisition](../../crates/spire-core/src/items/tests/m27_dispatch.rs#L16)
and the pair matrix
[m27_cofiring_pairs_follow_acquisition_order](../../crates/spire-core/src/items/tests/m27_cofiring.rs#L301).
Original source bytes are at base a86df922; pre-M27 transcript bytes are in main-v1/m2-replay/.

**G5 — active items marked definition-only.** Change: clear the erroneous flag on 14 relic and
eight potion rows that already have accepted effects. Shared owner:
[content/items.rs](../../crates/spire-core/src/content/items.rs). Reach: catalog metadata for every
character and the complete item unions; no frozen gameplay law changes. The old rows fail
[m27_identity_catalog_rows](../../crates/spire-core/tests/m27_catalog.rs#L48). No accepted
assertion was amended; original bytes are at base a86df922.

**G6 — complete tier and rarity metadata.** Change: provide relic tiers and potion rarities for
all standard identities instead of only Silent and Defect tables. Shared owners:
[content/items.rs](../../crates/spire-core/src/content/items.rs) and
[content/mod.rs](../../crates/spire-core/src/content/mod.rs). Reach: the shared Catalog and M27
complete pool/tier consumers; frozen producer lists remain fixed. The old missing metadata fails
[m27_identity_catalog_rows](../../crates/spire-core/tests/m27_catalog.rs#L48). No accepted
assertion was amended; original bytes are at base a86df922.

**G7 — Watcher potion eligibility.** Change: add Bottled Miracle, Stance Potion and Ambrosia to
the shared class-potion table. Shared owner:
[content/items.rs](../../crates/spire-core/src/content/items.rs). Reach: complete Watcher potion
pool and catalog eligibility; the pre-M27 production tree had no caller of this eligibility query.
The old shared eligibility fails
[m27_watcher_class_potions_are_class_restricted](../../crates/spire-core/tests/m27_catalog.rs#L88).
No accepted assertion was amended; original bytes are at base a86df922.

**G8 — class relic table completeness.** Change: extend the class-relic table from 14 entries to
all 40 class relics, as accepted by the PM. Shared owner:
[content/items.rs](../../crates/spire-core/src/content/items.rs), EXCLUSIVE_RELIC_CHARACTERS.
Reach: catalog eligibility and M27 complete item validation; the pre-M27 tree had no production
caller of this eligibility query. The old 14-row table fails
[m27_catalog_matches_reference](../../crates/spire-core/tests/m27_catalog.rs#L146). No accepted
assertion was amended; original bytes are at base a86df922. The source finding and ruling are in
main-v1/task0/seam-map.md and pm-rulings-H2-v1.json.

**G9 — Victory relics on escape.** Change in the candidate: Burning Blood, Black Blood, Meat on
the Bone and Face of Cleric do not receive a Victory settlement on Escaped. Shared owners:
[run/combat.rs](../../crates/spire-core/src/run/combat.rs) and
[items/relics.rs](../../crates/spire-core/src/items/relics.rs). Reach: the complete-item
settlement owner; M15 Silent and M20 Defect Smoke Bomb contexts cannot own the Ironclad Victory
relics, so this is frozen-unreachable. The old unconditional settlement fails
[m27_g9_escape_skips_victory_relics](../../crates/spire-core/src/combat/tests/m27_rescue_escape.rs#L114).
No accepted assertion was amended; original bytes are at base a86df922. The PM accepted G9 as a
global correction on condition that Main cite the source fact (pm-rulings-H2-v1.json; its sentence
"Burning Blood and Black Blood heal after a Smoke Bomb escape" names the defect being corrected).
The cited facts are the hook column of the retained relic research, which gives Burning Blood
`onVictory` ([act1-relics-closure.md:40](../research/2026-10-18-act1-relics-closure.md#L40)),
the catalog's "Victory:" primary rules, and Main-3's recorded source reading that `endBattle`
skips `onVictory` in a smoked room (main-v1/assistant-contracts/a26-combat-potions/CONTRACT.md).
Grade: static source reading through retained research; no original-runtime check.

**G10 — legacy Pure Water and Holy Water timing.** Change: both starter effects use the context's
single pre-draw visit in every context, including Legacy, rather than firing at the later
CombatStarted hook. Shared owners:
[items/mod.rs](../../crates/spire-core/src/items/mod.rs) and
[items/relics.rs](../../crates/spire-core/src/items/relics.rs). Reach: accepted Legacy combat
fixtures; M21 Watcher startup already uses its pre-draw hook, and no M9/M10 public observation
changes. The old Legacy timing fails
[m27_g10_legacy_pure_water_pre_draw](../../crates/spire-core/src/items/tests/m27_existing_relics.rs#L263);
the five amended accepted assertions and original bytes are listed below. Ruling: G10 in
spec-rulings-main2-v1.md supersedes R-PW2's earlier Legacy-unchanged clause. The five archived
originals are under a12-m3/original/.

## Corrections from user and PM rulings

The user corrected the proposed curse-claim path: shop card rows offer coloured and colourless
cards only, card rewards never offer curses, and the shop's only curse interaction is removal. HANDOFF-H4 records
the correction, the reverted shop-purchase path and dropped reward-claim change. The Constructed
[shop invariant](../../crates/spire-core/src/run/tests/m27_act4_producers.rs#L192) checks frozen,
complete Act 4 and Courier replacement rows. Relic-authored curse obtains remain Omamori-aware.

PM ruling F7 supersedes the design's empty-Boss-pool invariant error: when the Boss tier is
exhausted, return the source Red Circlet fallback (`returnRandomRelicKey`; retained research
[act1-continuations.md:268](../research/2026-10-19-act1-continuations.md#L268)); keep Red
Circlet outside ordinary unions. It refines planning ruling 3 (Red Circlet excluded from pools).
Constructed checks are
[m27_relic_pool_law_empty_boss_returns_red_circlet](../../crates/spire-core/src/run/tests/m27_relic_pools.rs#L86)
and [m27_red_circlet_catalog_row_not_in_union](../../crates/spire-core/src/items/tests/m27_capability.rs#L95).
The ruling is recorded in main-v1/HANDOFF-H3.md, main-v1/assistant-contracts/a19-pools/ADDENDUM-1.md
and main-v1/integration-5/NOTES.md; planning ruling 3 is in pm-open-question-rulings-v1.json.

The Smoke Bomb / Toy Ornithopter source check is retained as
source-check-smoke-bomb-ornithopter-v1.md (SHA-256 cedae5dc…). Its grade is static analysis of
the privately owned original game; no game code was executed. Smoke Bomb marks escape and does not
clear queued work; Ornithopter queues Heal(5) after potion-use callbacks. The resulting shared
queue fix preserves the manual-use heal. In frozen M15/M20 contexts Ornithopter work is
Complete-only and pending work is empty at manual use, so the change is reachable code without a
frozen behaviour difference. Integration-7 and the final replay evidence support this boundary.

Pen Nib's accepted public counter remains visible_counters. Its new internal counter owner and
complete_counter_view do not change that accepted view; the Constructed
[m27_complete_counter_view](../../crates/spire-core/src/run/tests/m27_relic_lifecycle.rs#L278)
test pins the separate complete view. There is no M27 public profile.

The PM accepted all seven planning recommendations in pm-open-question-rulings-v1.json and design
§18: no M27 public profile; M27 owns item pools, tier rolls, potion appearance and shared producer
laws while M28 owns room contents, unknown-room resolution and map path rules; Red Circlet remains
outside ordinary pools; relics and potions remain class-restricted while their effect laws work for
any eligible character; one dispatcher serves every profile and differences are disclosed; combat
DiscardPotion is complete-only; and the Opus helper senior owns Lane J. Review v1 I5 also rules
that complete-item Act 4 shop and elite rows use M27's general pools and tier rolls while frozen
profiles keep their existing lists. Main-2's source-sheet rulings, including the F13 and F14 pool
details below, are recorded in spec-rulings-main2-v1.md. The M27 evidence makes no original-runtime
or Java RNG parity claim.

The documentation-only Gym66 extension command in
[combat-training.md:253](../combat-training.md#L253) gained maturin's --release flag (Main-4,
HANDOFF-H4). This records the build command for an existing extension workflow; it is not evidence
that M27's Python extension was built or delivered.

## Lane-P potion fixes

Integration-7 found three shared-owner defects. On the red-rebased tree with all fixes reverted,
the lane recorded 122 passed / 26 failed / 0 ignored. Each individual fix was attributed by
re-running the red lane: Energy Potion alone 124 / 24 / 0 (two failures removed); potency alone
142 / 6 / 0 (20 removed); Smoke Bomb queue handling alone 126 / 22 / 0 (four removed). All three
together passed 148 / 0 / 0. Records: main-v1/integration-7/NOTES.md, fixes.patch and
red-rebased-v1/.

1. The Energy Potion capped-energy source in
   [combat/cards/defect.rs](../../crates/spire-core/src/combat/cards/defect.rs) now admits
   complete-items contexts for all four characters; previously the Ironclad and Silent source
   paths rejected it. Constructed coverage is in
   [combat/tests/m27_potions.rs](../../crates/spire-core/src/combat/tests/m27_potions.rs).
2. Potion callers now pass the base amount once to
   [items/potions.rs](../../crates/spire-core/src/items/potions.rs) potency; they no longer
   multiply the base by potency(1) and then apply potency again. This fixes Flex, Speed,
   Regeneration, Liquid Bronze and Essence of Steel across four characters (multiplicities
   5/5/5/3/4; Cultist Potion remains 1). Constructed coverage is in the same potion test file.
3. [combat/effects.rs](../../crates/spire-core/src/combat/effects.rs) queues CombatSettled and
   Terminal(Escaped) after work already queued by Smoke Bomb, so Toy Ornithopter's Heal(5) is
   not dropped. The source basis is the static check above; the potion owner tests and
   [m27_smoke_bomb_rules_and_settlement](../../crates/spire-core/src/combat/tests/m27_rescue_escape.rs#L62)
   cover the constructed boundary. None of the three fixes changes frozen-profile behaviour.

## Shared-owner fixes from X2 (D1–D3)

The supplied Act 4 Runs of X2 found three shared-owner defects. Each is fixed at the shared owner for
every character and reached only under complete item admission. The X2 lane recorded the REDs on the
unfixed tree (main-v1/x2-supplied/run1/), and the fixed tree passed them (main-v1/x2-supplied/). Records:
main-v1/integration-9/ and main-v1/HANDOFF-H8.md.

1. **D1 — elite potion appearance row count.** The four-row check precedes the auto-added card
   reward (economy research [§5.2](../research/2026-10-19-act1-content-economy-potions.md#L332)).
   The complete Act 4 elite owner in
   [rewards.rs](../../crates/spire-core/src/run/rooms/rewards.rs#L426) counted the CardReward row, so
   Gold plus two Relic rows (Black Star) suppressed White Beast Statue's row. It now counts the Gold
   and Relic rows only.
2. **D2 — Golden Idol.** The +25% gold bonus applies to every combat gold row except a
   TreasureRoom's (economy research
   [§2.3](../research/2026-10-19-act1-content-economy-potions.md#L128)). The
   [relic body](../../crates/spire-core/src/items/relic_effects/run.rs#L194) matched ordinary
   combats only; it now matches elite, boss and event combats too.
3. **D3 — Maw Bank floor entry.** Maw Bank's `onEnterRoom` gains 12 gold on every room entry
   (design Appendix A, MawBank). `RunItemEvent::FloorEntered` was dispatched only by the generic
   `world::enter` constructor, which always faults, so no real room owner reached it. Each room
   owner (act1, act2, act3, act4, profile and boss treasure) now dispatches it immediately before
   `RoomEntered`, inside the complete-admission block.

These M27-lane tests were corrected with the fixes (original bytes: main-v1/candidate-v1/tree/):
[m27_act4_potion_producers_under_complete_elite](../../crates/spire-core/src/run/tests/m27_potion_rewards.rs#L112)
asserted that Gold, two Relics and the card row suppress the row. It now asserts that three counted
rows show the row and four suppress it.
[m27_golden_idol_stored_gold_bonus](../../crates/spire-core/src/run/tests/m27_relics_run.rs#L659)
adds Elite, Boss and Event rooms, and
[m27_run_seams_reached](../../crates/spire-core/src/run/tests/m27_run_seams.rs#L114) counts five
FloorEntered events, one for each real room entry plus the generic constructor. None of these is a
pre-M27 accepted assertion. The frozen replay and gate-audit addendum show no frozen change.

## Rebase onto PR #68 (D4)

PR #68 (db42bf4d, spire-14g) merged into main while M27 was frozen. The M27 tree was rebased file by
file with a86df922 as the base: 21 PR #68 paths that M27 does not touch were taken as they are, and
the 11 shared paths were three-way merged. Two of the merges conflicted and were resolved so that
both rule sets hold. In
[apply_replacement](../../crates/spire-core/src/combat/enemies/mod.rs#L363), hits queued at the
replaced Slime are cancelled first, then each child's M27 EnemySpawned relic phase runs. The
[Exhaust relic phase](../../crates/spire-core/src/combat/exhaust.rs#L16) keeps M27's dispatch and
uses PR #68's targetable() enemy list. M27's own line changes are identical against either base.
PR #68's rules are unchanged. These are the No Draw no-op (M27's power-applied phase sees no change,
so no relic callback runs), cancel_queued_hits_at, targetable() and the Large Slime split order.
Records: main-v1/rebase-db42/REPORT.md.

**D4 — all-enemy relic producers use targetable().** PR #68 put every all-enemy producer on one
[targetable()](../../crates/spire-core/src/combat/model.rs#L211) predicate. It skips dying,
half-dead and escaped enemies. The M27 producers now use it too:
[CapEnemyHp](../../crates/spire-core/src/combat/effects.rs#L502) and the
[relic-phase live-enemy list](../../crates/spire-core/src/combat/effects.rs#L956). That list feeds
Mercury Hourglass, Stone Calendar, Letter Opener and Brimstone's enemy Strength. The battle-start
producers also read it now instead of every actor:
[Philosopher's Stone](../../crates/spire-core/src/items/relics.rs#L62),
[Red Mask and Bag of Marbles](../../crates/spire-core/src/items/relics.rs#L189) and
[Twisted Funnel](../../crates/spire-core/src/items/silent.rs#L27). The predicate is the one these
producers already open-coded (active with HP). Every actor is constructed active at combat start.
The result is therefore unchanged: the replay copy matched the pre-rebase copy 323/323. Potions
already used PR #68's targetable() targets.

## Final-review fixes (D5–D16)

The independent final review of candidate v2 (evidence/m27-independent-senior-review-20261102/review-v1.md)
returned NOT READY with four Important (I1–I4) and seven Minor (m1–m7) findings. None was reachable by a
frozen profile. Each is fixed at its shared owner for every character and is reached only under complete
item admission. Each fix has a test that fails on candidate v2 and passes on freeze v5 (main-v1/integration-10/
and main-v1/assistant-contracts/m10-tests/out/). Two PM static source checks of the original settle m4, m6
and m7 (m27-execution-20261031/source-check-potion-drop-eggs-v1.md and source-check-nloth-hungry-face-v1.md).

1. **D5 — Maw Bank and the shop card removal (I1).** The shop purge spends gold in the ShopRoom, so it now
   dispatches the same shop-spend event as a product purchase
   ([selection.rs](../../crates/spire-core/src/run/rooms/selection.rs)). Every ShopRoom gold spend
   dispatches it; Maw Bank is spent by any of them (MAW_BANK effect). Test:
   m27_review_i1_maw_bank_shop_purge_spends_gold_and_disables (end-to-end on the complete Act 4 route).
2. **D6 — Mutagenic Strength applies Strength Down (I2).** Battle start applies 3 Strength, then the
   shared `lose_strength` debuff 3, as Flex does
   ([turns.rs](../../crates/spire-core/src/items/relic_effects/turns.rs)). The real end of turn removes
   the Strength. Artifact blocks the debuff and Orange Pellets removes it, so the Strength is then
   kept. The former relic end-turn reduction is gone. The M27 Mutagenic tests now check the debuff and use
   a real end of turn.
3. **D7 — the one heal law (I3).** Lee's Waffle raises maximum HP by 7, then heals to the maximum
   through the heal law, so Mark of the Bloom keeps the gain and prevents the heal. Meat on the Bone and
   Face of Cleric heal on victory while the room is still COMBAT: the new
   `victory_heal_amount` ([items/mod.rs](../../crates/spire-core/src/items/mod.rs)) applies Magic Flower
   (12 → 18; 1 → 2). The supplied Watcher set owns Mark of the Bloom; its Waffle assertion now expects
   no heal. Every other heal already uses the law: outside combat `mutations::heal` and
   `increase_max_hp` (rest, Act 4, Neow, act transition, rewards, Meal Ticket, Eternal Feather, Bloody
   Idol, Darkstone Periapt, Tiny House, relic acquisition, outside-combat potions), inside combat
   `heal_player` with the live heal room. The non-complete Burning Blood settlement bridge is a frozen
   path and is unchanged.
4. **D8 — Tiny Chest counts rolls (I4).** `run_laws::unknown_room` takes the Run state and must be
   called once per qualifying Unknown-room roll. Tiny Chest increments its counter on each call; the
   fourth call resets it to 0 and forces Treasure. The test drives the law, not a hand-set counter.
5. **D9 — Matryoshka's extra relic (I4).** `run_laws::chest_contents` now returns a Result and is
   atomic. While a charge remains, a non-Boss chest rolls Common 75% / Uncommon 25% on the relic tier
   stream, takes that tier's front relic from the pools and spends the charge. `ChestRules.extra_relic`
   carries it. The charge is no longer spent by the chest-opened Run event.
6. **D10 — N'loth's Hungry Face (m7).** `ChestRules.empty` is replaced by `remove_first_relic`. The
   next non-Boss chest loses its first relic row and the Sapphire Key linked to it; gold and other rows
   stay, and Boss chests are unaffected. `run_laws::apply_chest_rules` applies the rules to the chest
   rows in source order: Matryoshka's relic first (before-open callback), then Hungry Face (after-open
   callback). With both, the chest keeps its own relic and key.
7. **D11 — Exhaust order (m1).** Relic bottom work is queued before power and card exhaust work
   ([exhaust.rs](../../crates/spire-core/src/combat/exhaust.rs)). Dead Branch's card reaches the hand
   before Dark Embrace draws. Dead Branch is the only relic with Exhaust bottom work.
8. **D12 — a lost Potion Belt keeps its slots (m2).** Under complete admission five slots stay valid
   without the belt (F6), so combat entry and potion obtains still succeed.
9. **D13 — Red Circlet can be obtained (m3).** `obtain_relic` admits the F7 Boss fallback.
10. **D14 — Eggs upgrade cards already on the reward screen (m4).** Frozen, Molten and Toxic Egg
    upgrade every matching unupgraded card in the generated card rewards when obtained.
11. **D15 — Entropic Brew outside combat draws one proposal per slot (m5).** Complete admission now
    draws `potions.len()` proposals, as the accepted path does. The obtained potions are unchanged;
    the RunState RNG differs for a later draw on the same scope. The M27 Brew test now expects three
    proposals.
12. **D16 — potion offset after an escape (m6).** When every monster escaped, only the chance is 0. The
    roll still happens and the offset keeps its history (+10 on a miss). A player Smoke Bomb escape
    rolls as a won fight, updates the offset and discards any rolled potion with the hidden rewards;
    `combat_appearance` documents this for its M28 caller.

The four test files that ended with an extra blank line (the candidate-v2 publication erratum) are fixed.

## Amended accepted assertions

The checkout source scan found five G10 (M27) comments on accepted tests, plus the separate
new-regression-test comment; it found no G1 (M27) comment. G1's accepted M15 assertion carries a
G1 explanation in its assertion message, not a separate comment.
These are all six accepted assertions amended in M27; each source line and original byte copy is
listed. G10's ruling is the single-visit Legacy correction described above.

| Accepted test source line | Disclosure comment or assertion | Ruling | Original bytes |
| --- | --- | --- | --- |
| [combat/tests/m15_generation.rs:251](../../crates/spire-core/src/combat/tests/m15_generation.rs#L251) | No standalone disclosure comment; assertion message says Sozu supplies corrected G1 master energy. | G1: add Sozu's +1 master energy. | main-v1/a22-energy/original/crates/spire-core/src/combat/tests/m15_generation.rs |
| [combat/tests/m5_elites.rs:103](../../crates/spire-core/src/combat/tests/m5_elites.rs#L103) | Pure Water now faults at hook-free pre-draw before Neow's Lament. | G10: pre-draw in Legacy too. | a12-m3/original/m5_elites.rs |
| [combat/tests/full_runtime.rs:643](../../crates/spire-core/src/combat/tests/full_runtime.rs#L643) | Legacy Pure Water faults before CombatStarted can draw for Ring of the Snake. | G10: pre-draw in Legacy too. | a12-m3/original/crates/spire-core/src/combat/tests/full_runtime.rs |
| [items/tests.rs:111](../../crates/spire-core/src/items/tests.rs#L111) | Legacy Pure Water creates Miracle before the opening draw; a full hand leaves a starter card in draw. | G10: pre-draw in Legacy too. | a12-m3/original/crates/spire-core/src/items/tests.rs |
| [run/tests/m21_watcher.rs:30](../../crates/spire-core/src/run/tests/m21_watcher.rs#L30) | The second fixture is Legacy, not M21; its Miracle is in hand before the opening draw. | G10: pre-draw in Legacy too. | a12-m3/original/crates/spire-core/src/run/tests/m21_watcher.rs |
| [run/tests/rooms.rs:67](../../crates/spire-core/src/run/tests/rooms.rs#L67) | Pure Water faults during hook-free startup before CombatStarted decrements Neow's Lament. | G10: pre-draw in Legacy too. | a12-m3/original/crates/spire-core/src/run/tests/rooms.rs |

## Frozen transcript replay

Design §6.2 compares each retained M13–M25 transcript step-by-step on the merged pre-M27 source
and the candidate, using full combat State Debug, public_full (which preserves enemy-power row
order), and RunState Debug for Run transcripts. Legacy public_view is not sufficient because its
power sorting would conceal reorder differences. The replay includes the retained canonical,
supplied and control attempts plus each of the 78 actual co-firing relic pairs in both acquisition
orders.

main-v1/m2-replay/report.md first classified the only order differences as G4. The final
m27_frozen_replay run over pair-scripts-v2/inputs-v2.json had 323 inputs. Against BASE-A86 the
only differences were the disclosed reorder-only pair-059 and pair-060 at battle start:
Twisted Funnel before Bag of Marbles changes Vulnerable-then-Poison to Poison-then-Vulnerable;
Twisted Funnel before Red Mask changes Weak-then-Poison to Poison-then-Weak. These are visible
power-order changes in the constructed M15 Silent combat view. No retained external M13–M25 Run
transcript reaches either pair. Integration-8's copy comparison against pair-scripts-v3/copy-run1
was 323/323 identical. Replay artifacts are in main-v1/integration-7/NOTES.md and
main-v1/integration-8/. After the D1–D3 fixes the replay was rerun on the freeze-v3 copy, and again
it was 323/323 identical to pair-scripts-v3/copy-run1 (main-v1/freeze-v3-void/logs/compare-copy5-vs-v3-copy.log).
After the rebase onto PR #68 a fresh baseline was built from db42bf4d and run twice; the two runs were
323/323 identical. It is also 323/323 identical to BASE-A86, so PR #68 changes no frozen input.
Against it the freeze-v4 copy differs only in the same reorder-only pair-059 and pair-060. The
freeze-v4 copy is 323/323 identical to pair-scripts-v3/copy-run1 (main-v1/freeze-v4/INVOCATIONS.md).
The freeze-v5 copy (replay-copy-run8) is 323/323 identical to the freeze-v4 copy and differs from the
unchanged BASE-DB42 baseline only in the same reorder-only pair-059 and pair-060 (main-v1/freeze-v5/REPORT.md).

## Gate audit (design §6.3)

The final site sweep in main-v1/gate-audit-v1/site-triage.md classifies 639 H0 rows as 514
MATCHED, 104 MATCHED-CHANGED and 21 REMOVED; all 21 removed sites were moved, with none dropped.
It also records 167 NEW sites: A 87, I 51, P 17, C 4, S 4, O 3 and Debug 1. Among 51 frozen-reachable
sites, 38 are unchanged or Complete-gated, 11 are reachable-identical and two are disclosed
corrections G4 and G10. There are no other frozen behaviour changes and no public combat-v1,
M9 or M10 observation changes. G4's two pair scripts are frozen M15 constructed observations,
not one of those public-profile schemas. The gate tests
[m27_no_inert_items](../../crates/spire-core/src/items/tests/m27_gate_audit.rs#L835) and
[m27_single_item_owners](../../crates/spire-core/src/items/tests/m27_gate_audit.rs#L1017) passed;
integration-7 records 475 / 0 / 0 m27_ library tests at that gate. Freeze v3 adds eight
complete-gated production sites for D1–D3 (main-v1/freeze-v4/gate-audit-addendum.md): one in
rewards.rs, one Golden Idol relic body and six FloorEntered dispatches. None is frozen-reachable.
Freeze v4 adds the two conflict resolutions and the D4 sites, all reachable with identical results.
Freeze v5 adds the D5–D16 sites, all complete-gated and none frozen-reachable
(main-v1/freeze-v5/gate-audit-addendum.md).
Both gate tests pass on freeze v4.

## Cross-character item matrix (X1, design §15.3)

[m27_cross_character.rs](../../crates/spire-core/src/items/tests/m27_cross_character.rs) runs M26's
stripped context: Jaw Worm at setup HP, 80/80, energy 10, five Defends, opening draw 1, and only the
item under test. For each of the literal 139 shared relics and 30 shared potions, it measures each
character's public-state delta against the same script without the item and requires it to equal
Ironclad's. Run-owner relics also compare their Run event deltas. Literal rows cover the
character-sensitive cases: the nine +1 energy relics, Slaver's Collar and Ice Cream (including its
999 cap); Prismatic Shard's orb slots (1/1/3/1); the card generators Dead Branch, Enchiridion,
Nilry's Codex, Toolbox, Orrery and Tiny House and the Attack, Skill and Power Potions (class colour
only); and the potion generators Cauldron, White Beast Statue's offer and Entropic Brew (the
character's 33 potions only). Grade: Constructed, 9 tests. X1 found no defect; its three
intermediate REDs were fixture errors (main-v1/x1-cross-character/reds.md). Inertness is the
existing [m27_capability_overlay_inert](../../crates/spire-core/src/items/tests/m27_inert.rs#L55).

## Supplied Act 4 Runs (X2, design §15.4)

[m27_supplied.rs](../../crates/spire-core/src/run/tests/m27_supplied.rs) (feature
m27-diagnostics) plays one supplied Run per character from `complete_act4_run`, using only legal
`Run::step` actions. The rooms are floor 51 campfire, floor 52 shop (one relic and one potion row),
floor 53 elite with every reward row claimed, and floor 54 Heart with Fire Potion on turn 1. In combat
the policy plays the first legal card, otherwise ends the turn, and never discards a potion. Each set
holds the same 20 combat relics, Neow's Lament at counter 1 (so the elite's enemies start at 1 HP) and
three potions. On top of that: Ironclad has the rest relics with Black Blood and the Golden and Bloody
Idols; Silent has the shop discounts, Maw Bank, Meal Ticket, Black Star and Coffee Dripper; Defect
has Fusion Hammer, Singing Bowl, Prismatic Shard, Sacred Bark, Lizard Tail and a Fairy potion;
Watcher has Busted Crown, Ectoplasm, Runic Dome, Snecko Eye and Mark of the Bloom. The sets and
their rulings are main-v1/x2-supplied/sets.md and assistant-contracts/x2-supplied/PHASE-B-RULING*.md.
Each room's expectations are literal section 11.4 rules: the shop relic's Shop-tier membership,
its 150 price before discounts and its removal from the pool; each potion row's class membership
and rarity price; the elite relic's ownership and consumption; Black Star's non-campfire second
relic; and White Beast Statue's row with a −10 offset. Also asserted: a full belt keeping the
offer, the heal laws from a supplied 30 HP, the gold laws, the counters across both combats, the
inventory returned once after each settlement, and the Defect rescue order (Fairy, then Lizard
Tail). Grade: Supplied played, 4 tests. All four Runs end in a Heart Death that settles with no fault;
this is a policy outcome, not a strategy claim. The Run 1 REDs led to D1–D3 above.

## Frontier bindings (design §13)

- **F1 — Disclosed decision, bound.** One acquisition-order visit per phase; bottom work stays in visit
  order, while explicit top groups reverse among themselves. All 78 co-firing pairs are exercised in
  both orders. Tests: m27_dispatch_top_work_order and m27_cofiring_pairs_follow_acquisition_order.
  G4 is the only frozen-visible consequence.
- **F2 — Native mapping bound.** onPlayCard maps to CardPlayed before payload; onUseCard maps to
  CardAfterUse after payload and before FinishCard; AfterUse is a later distinct phase, while
  CardAfterPlayed remains the Silent power hook. The binding is from the engine seam and Main's F2
  record; it is not a Native agreement claim.
- **F3 — Bound by G9.** Burning Blood, Black Blood, Meat on the Bone and Face of Cleric are
  Victory relics and do not run when the combat ends Escaped. The Constructed G9 test
  m27_g9_escape_skips_victory_relics pins this; the source facts and the PM acceptance are under
  G9 above. Frozen-unreachable: M15 and M20 Smoke Bomb contexts cannot own these relics.
- **F4 — Disclosed decision, bound.** Held potion potency stays at the value fixed when created or
  obtained after Sacred Bark is lost. Test: m27_sacred_bark_lost_potency_reaches_combat_use. The
  Sacred Bark record has no unequip rule.
- **F5 — Bound.** Removing a linked bottled card clears the link; a transformed replacement is a
  new, unbottled instance. Tests: m27_bottled_flame_boundary, m27_bottled_lightning_boundary and
  m27_bottled_tornado_boundary.
- **F6 — Disclosed decision, bound (PM-accepted; supersedes the H0 proposal to remove two slots).**
  Losing Potion Belt leaves its extra two slots and their potions in order; there is no source
  unequip rule and the catalog law permanently adds two slots. Test: m27_replacement_and_loss and its
  F6 assertion.
- **F7 — PM source ruling, bound.** This supersedes design §7.4's invariant-error proposal: an
  exhausted Boss pool returns fallback Red Circlet (source `returnRandomRelicKey`), which stays
  outside ordinary unions. Tests:
  m27_relic_pool_law_empty_boss_returns_red_circlet and m27_red_circlet_catalog_row_not_in_union.
- **F8 — Bound.** Every potion effect carries Source::Potion{slot,id} through the shared effect
  owner. The all-character potion tests and potion-owner inventory are in m27_potion_owner.rs,
  m27_potions.rs and m27_potion_lifecycle.rs.
- **F9 — Disclosed decision, bound.** Necronomicon repeats the first Attack each turn with paid
  cost at least 2 (X uses paid energy), free; power duplicators queue before its repeat. Tests:
  m27_necronomicon_with_duplicators,
  m27_engine_3_necronomicon_counts_x_and_queues_after_power_duplicates and
  m27_necronomicon_pickup_adds_necronomicurse.
- **F10 — Bound.** Ice Cream adds unspent energy to master energy in each character's recharge
  owner before energized, energized_blue and energy_down; cap 999. Evidence: m27_energy_law_all_characters
  and relic research §2.
- **F11 — Disclosed decision, bound.** Snecko Eye randomizes cost on draw; Mummified Hand,
  Enchiridion and Warped Tongs apply their own cost changes afterwards. Tests:
  m27_snecko_eye_confused_on_draw, m27_mummified_hand_power_cost_zero,
  m27_enchiridion_class_power_zero and m27_warped_tongs_postdraw_upgrade.
- **F12 — Disclosed decision, bound under Complete.** Runic Dome hides enemy intent and intent
  damage; Frozen Eye reveals the actual current draw order and no extra private state. Tests:
  m27_runic_dome_hide_enemy_intents and m27_frozen_eye_draw_order_visible.
- **F13 — PM source rulings, bound.** Girya, Peace Pipe and Shovel spawn below floor 48 while
  fewer than two of the trio are owned; Mercury Hourglass has no cutoff; Hungry Face comes from
  Face Trader and, on the next non-boss chest after base contents, removes only the first relic
  row and linked item (implemented by apply_chest_rules, D10); Bloody Idol's source is Forgotten Altar. Source rulings are in
  spec-rulings-main2-v1.md. Constructed tests:
  m27_relic_pool_law_spawn_restrictions_use_floors_acts_deck_and_source,
  m27_nloths_hungry_face_next_chest_relic_removal and m27_bloody_idol_gold_gain_heal_five.
- **F14 — PM-bound pools, bound.** Dead Branch uses the player's Combat Any pool with healing
  excluded; Enchiridion uses the player's Power pool with healing excluded and makes the choice
  free this turn; Toolbox offers three distinct ordinary colourless ids before opening draw;
  Nilry's Codex offers three distinct player-class Any cards, can be skipped, and shuffles the
  choice into draw. Tests: m27_dead_branch_actual_exhaust_card,
  m27_enchiridion_class_power_zero, m27_toolbox_three_colorless_choose and
  m27_nilrys_codex_choose_one_of_three.
- **F15 — Bound.** Catalog effects and spec-sheet v2 bind seven relics: Bloody Idol heals 5 on
  gold gain; Circlet has no separate effect and is the Rare-tier fallback; Enchiridion offers a Power
  before opening draw; Mutagenic Strength grants 3 Strength and a matching Strength Down debuff
  that removes it at the end of turn 1 (Artifact blocks the debuff and Orange Pellets removes it; D6);
  Necronomicon's repeat is bound under F9; Nilry's Codex presents its skippable end-turn choice;
  N'loth's Gift triples base Rare chance while the pity offset still applies unchanged. Tests:
  m27_bloody_idol_gold_gain_heal_five, m27_circlet_rare_pool_fallback,
  m27_enchiridion_class_power_zero, m27_mutagenic_strength_first_turn_strength,
  m27_necronomicon_first_attack_repeat, m27_nilrys_codex_choose_one_of_three and
  m27_nloths_gift_rare_chance_tripled.

## M28 call list (design §11.3)

M28 must connect the constructed Run laws to the rooms and run surfaces it owns:

- Unknown rooms: Juzu Bracelet's monster-to-event conversion, Tiny Chest's treasure-room
  trigger and Ssserpent Head's gold.
- Map path selection: Wing Boots' off-path moves.
- Card reward shape: N'loth's Gift's ×3 Rare chance and Prayer Wheel's second ordinary-combat
  card reward.
- Act 1–3 chest contents: Matryoshka, Cursed Key and N'loth's Hungry Face.
- Events and Neow: the swap and trades listed in design §7.3.
- Score: Spirit Poop's score adjustment.

M27 exposes these named entry points in
[items/run_laws.rs](../../crates/spire-core/src/items/run_laws.rs): unknown_room, map_path,
card_reward_shape, chest_contents and score_adjustment. Design §11.3 requires M28 Task 0 to
check that each room owner calls the right law. Call unknown_room exactly once per qualifying
Unknown-room roll (Tiny Chest counts there; D8). Call chest_contents once per opened chest and apply
its rules to the chest rows with apply_chest_rules (D9, D10). Call potion_offers::combat_appearance for
a Smoke Bomb escape as a won ordinary fight and discard the potion (D16). Until then this evidence is
Constructed only.

## Not established

Design §16 remains the boundary. M27 establishes no public profile, descriptor or Python surface
for complete items; no full ownership of the rooms that offer items beyond shared owners; no
M28 unknown-room, map-path or card-reward schedule; no A1–A20 content including the A11 potion
slot; no Red Circlet membership in ordinary unions (only the F7 exhaustion fallback); no M34
score screen; and no original-runtime, Java RNG, strategy or win claim. The §11.3 room laws remain
Constructed until M28 connects them.

X2 has bounded coverage. Burning Blood is not in a supplied set: it is mutually exclusive with Black
Blood in the single Ironclad Run, and Constructed lifecycle tests cover it. Sozu, Circlet, Omamori and
Darkstone Periapt (no curse obtain on the route) and the §11.3 room laws are outside the supplied
route. Equip effects are asserted only for the shop and elite relics actually obtained. The rescue
heal is observed as post-step HP within the 60% and 50% bounds, not as a separate heal event. The
shop and elite selector ends (back and front) are Constructed pool-law tests plus a reading of the
call sites, not an X2 assertion.

No Normal played or Native agreement result exists because M27 has no profile.
