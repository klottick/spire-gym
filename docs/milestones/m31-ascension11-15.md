# M31 Ascension 11–15

Status: source candidate, not accepted; no spire-env, Gym or Python profile (M28 ruling Q1; exposure M34–M36).

M31 extends the Standard Run boundary and records the finite A11–A15 rules. The source basis is the static source check and engine audit listed below. This profile is a candidate record, not an acceptance or delivery record.

## What M31 is

Standard admits A0–A15 in a default build. At A16 and above it returns Unsupported("Standard supports A0–A15").

At A11, a Run starts with two potion slots; Potion Belt adds two, for four total. That shape is retained through relic equip, combat entry and settlement, slot use, generated potion fill, Entropic Brew sampling, reward/shop/event row availability, Sozu handling, and loss of Potion Belt. A potion in a retained Belt slot remains usable after the Belt is lost.

At A12, Standard card-reward automatic-upgrade chance is halved in Acts 2–4: Act 2 is 0.125 instead of 0.25, and Acts 3–4 are 0.25 instead of 0.5. Act 1 remains 0.

At A13, the boss gold base is reduced to round(binary32(base × 0.75)) before Golden Idol and other reward bonuses. This reaches the Act 1 and Act 2 boss reward screens; a Standard Act 3 boss and the Heart have no reward screen. Mind Bloom's Fight reward is 25 gold at A13+.

A14 max HP and A15 event behavior are pinned through the public Standard route. M31 changes no A14 or A15 rule code; it adds tests. No enemy behavior changes in M31.

The global fixes apply at every ascension:

- G1: Standard Run card offers opened by Orrery, Tiny House, and Dream Catcher use the ordinary reward law for that room, including its rarity threshold, live offset, choices, upgrade draw, Egg preview and card modifiers. Cauldron and Calling Bell generate and discard their hidden card row under the same law. Neow's Three Small Potions row was already implemented and remains unchanged.
- G2: after Potion Belt is lost, a potion in a retained extra slot can still be used, including a Fairy's automatic rescue. The inventory keeps its slots.
- G3: Standard shop colored-card stock and Courier restocks use the ShopRoom rare threshold 9/37, with relic rarity hooks off.

**Compatibility.** Standard now returns Unsupported("Standard supports A0–A15") for A16+. A0 Standard Runs can diverge from their first G1 site or first shop: G1 changes equip or hidden-row draws and the live rarity offset; G3 changes shop card rarity. A G2 divergence requires a lost Belt and a later potion in a retained slot. G1 is Standard-only: non-Standard Complete diagnostics retain the legacy Common/Uncommon/Uncommon equip row. The shared Courier restock law also makes Act 4 Complete diagnostics use 9/37.

## Limits and Outs

- G1 applies to the Standard Run only; non-Standard Complete diagnostics keep the legacy C/U/U equip row (review I1). The M27 equip fixtures stay on that route.
- The source's ShopRoom.rewards re-lists, and may re-claim, earlier rows (statically inferred); not emulated, out of scope (review n2).
- G2 narrows to the rejected potion-use cases: a retained slot was blocked only when the effect carried Source::Potion{slot} (including Energy Potion under Complete items and Fairy rescue). Other potion uses, such as Block Potion for Ironclad, Silent and Defect, already worked. Finite authored profiles retain their fixed three-slot contract.
- A13 boss gold is covered only at the Act 1 and Act 2 boss reward screens. Daily-mode gold follows its separate flat-100 rule and is outside this Standard profile.
- A0 attribution granularity (final review m3). The G3 marker records only the first shop, so a G3 attribution means "diverged at or after the Run's first shop", not "diverged at a shop". In 10 of 41 differing A0 default Runs, 27 of 84 A0 diagnostics Runs and 2–5 per A1–A10 level, the first difference falls well after the first shop (for example A7 `reach/Ironclad/4`: shop at step 80, first difference at 191); these are probably later shops or Courier restocks, since a first shop's coloured stock is unchanged whenever all five draws avoid n+b ∈ [3,9)∪[40,46). The traversal output has no per-step room kinds, so this was not localized further. The final review's static check found no engine change that can reach A0–A10 outside G1–G3: every engine hunk is ascension-gated (A11+, A12+ or A13+), the cap, the item-context field read only through the potion base, or a G1, G2 or G3 site.
- Traversal reach. The c1 traversals (25 seeds per character; uniform and reach in a default build, plus the diagnostics supplied mode) had 0 faults, 0 interim rejections and 0 blocked Runs at every level A0–A15, created every A11–A15 Run with two potion slots, matched the A14–A15 start HP literals, and met no Note For Yourself at A15. Default-build A11–A15 Runs reached at most Act 2 floor 25; the diagnostics modes reached Act 3 (floor 50 at A15), with an Act 2 boss cleared in 1–3 Runs at A13–A15 and no Act 3 boss, Act 4 or Heart. Boss gold rows without Golden Idol read 95–105 at A11–A12 and 71–79 at A13–A15, including Act 2 bosses. No traversal Run lost a Potion Belt or fought Mind Bloom (2–3 Runs per diagnostics level gained a Belt and ended with four slots). So G2, Mind Bloom, the Act 3–4 A12 rows, the Act 2–3 A15 events and the Act 2 A13 boss rest on the Run-grade unit tests above, not on traversal coverage.
- A16+ rules remain outside M31: the A16 shop-price change, A17–A19 enemy behavior, and the A20 double boss. No enemy work of any kind is in M31.
- spire-env, Gym, and Python exposure remains M34–M36. The Run observation already carries the live potion vector; M31 does not change the encoder or schema.
- Original-runtime validation is out of scope. The source check does not establish runtime RNG parity; the engine continues to use its keyed RNG scopes. Note For Yourself's saved-unlock rule remains UNLOCK-001.
- Strategy and win-rate targets are out of scope.

## Evidence index

Paths beginning $E are under E=/home/klottick/.local/share/spire-gym/evidence/m31-execution-20261104 (private evidence store, not published here).

- Static source record: $E/planning-v2/research/m31-source-check-v1.md
- Engine source audit and equip-offer comparison: $E/planning-v2/research/m31-engine-audit-v1.md and $E/planning-v2/research/m31-equip-offers-v1.md
- Design, implementation plan and independent plan review: $E/planning-v2/2026-11-04-m31-ascension11-15-design.md, $E/planning-v2/2026-11-04-m31-ascension11-15-plan.md, and $E/plan-review-v1/review-v2.md (READY; static plan review).
- Main-tree binding and G3 ruling: $E/m31/task0-bind.md and $E/m31/pm-ruling-g3-v1.md
- Potion and reward matrix reports: $E/m31/asst-potions/report.md and $E/m31/asst-rewards/report.md. The integrated workspace and diagnostics receipts are $E/m31/main-workspace-run2.log and $E/m31/main-diag-run1.log.
- Original bytes for amended accepted assertions: $E/m31/amended-original/SHA256SUMS.base-5e2a10b4 and the files beside it.
- Candidate Runner A, Runner B, traversal, replay and attribution receipts: $E/m31/c1-checks/ (results-a.md, results-b.md, r2-traversal-summary.md, r2-join-*.json, r2-fuzz-compare.md, r2-replay-classification-main-v1.md). Mutation pass: $E/m31/mutations-v1/summary.md and addendum-main-v1.md (27/28 caught, M9 equivalent).

## Coverage ledger

The static source record is the check of desktop-1.0.jar with SHA-256 cfad868ac8d65a88e71a0bf096fb09f78811e553effe0787c5309a655e081673. Grades are Run, Combat or Table. Test names below map source rows to the native tests in the candidate evidence; this is not original-runtime validation.

### §A: threshold sweep

| Source row | Locator | Grade | Test(s) | Note |
|---|---|---|---|---|
| A11 player construction: base slots fall from 3 to 2 | verification/2026-11-03-m29-asc-a-v1/decompiled/com/megacrit/cardcrawl/characters/AbstractPlayer.java:235,292–307 | Run | m31_a11_two_slots_before_neow | Checks A10/A11 for all four characters, short and full Neow, before Neow changes the inventory. |
| A12 TheCity: upgrade chance .25 to .125 | verification/2026-11-03-m29-asc-a-v1/decompiled/com/megacrit/cardcrawl/dungeons/TheCity.java:94 | Table | m31_a12_upgrade_chance_table; m31_a12_reward_upgrade_boundary | Act 2 boundary also checks an ordinary Standard reward at A11/A12. |
| A12 TheBeyond: upgrade chance .5 to .25 | verification/2026-11-03-m29-asc-a-v1/decompiled/com/megacrit/cardcrawl/dungeons/TheBeyond.java:89 | Table | m31_a12_upgrade_chance_table; m31_a12_reward_upgrade_boundary; m31_a12_equip_offer_upgrade | Act 3 ordinary and equip offers use the reduced chance. |
| A12 TheEnding: upgrade chance .5 to .25; Exordium remains 0 | verification/2026-11-03-m29-asc-a-v1/decompiled/com/megacrit/cardcrawl/dungeons/TheEnding.java:173; context-decompiled/com/megacrit/cardcrawl/dungeons/Exordium.java:123 | Table | m31_a12_upgrade_chance_table | The table covers all four acts and A11/A12 values. |
| A13 boss-clear gold base is multiplied by .75 and rounded; normal/elite gold is unchanged | verification/2026-11-03-m29-asc-a-v1/decompiled/com/megacrit/cardcrawl/rooms/AbstractRoom.java:339–350 | Combat | m31_a13_boss_gold_table; m31_a13_boss_gold_act2; m31_a13_boss_gold_golden_idol; m31_a13_elite_and_normal_gold_unchanged | Boss reward screens exist for Acts 1 and 2; Idol is applied after the reduced base. |
| A14 initial Exordium max-HP loss precedes A6 start-HP assignment | verification/2026-11-03-m29-asc-a-v1/decompiled/com/megacrit/cardcrawl/dungeons/AbstractDungeon.java:2674–2687 | Run | m31_a14_start_hp_standard; m31_a14_neow_max_hp_rewards | Public Standard start HP/max HP and full-Neow max-HP effects are checked for all four characters. |
| A13 Mind Bloom Fight reward: 50 to 25 gold | verification/2026-11-03-m29-asc-a-v1/decompiled/com/megacrit/cardcrawl/events/beyond/MindBloom.java:85–94 | Run | m31_a13_mind_bloom_gold | Checks the fixed award at A0/A12 and A13/A15. |
| A15 event changes and Note For Yourself gate | Source-check §A; event-specific source locators and tests are itemized under §F below. | Run | §F rows below; m31_a15_note_for_yourself_pool_gate | The 22 event IDs with A15 effects are detailed under §F. |
| Monster rules: none first change in A11–A15 | Source-check §A threshold sweep; encounter ceilings in crates/spire-core/src/combat/enemies/ascension.rs:37–48 | Combat | m29_every_encounter_every_ascension; m29_all_species_landed; m29_d_formations_ceilings | No monster threshold lies in 11–15. Encounter ceilings are 16/17/18/20; source behavior thresholds next occur at A17–A20. |

The §A progression, stats, achievement and UI-only sweep found no A11–A15-only branch; those categories are outside the Run behavior profile.

### §B: A11 potion slots

| Source row | Locator | Grade | Test(s) | Note |
|---|---|---|---|---|
| New Run inventory: 3 slots below A11, 2 at A11+ | AbstractPlayer.java:235,292–307 | Run | m31_a11_two_slots_before_neow | Public Standard creation; A10 is the t−1 control. |
| Potion Belt appends two slots to the live base | PotionBelt.java:24–29; AbstractRelic.java:477 | Run / Combat | m31_a11_potion_belt_four_slots; m31_a0_potion_belt_unchanged | A11 changes 2→4; A0 remains 3→5. |
| Complete capacity validation, combat entry and settlement preserve the Run vector | crates/spire-core/src/items/mod.rs:326–335; crates/spire-core/src/run/combat.rs:382–404 | Combat | m31_a11_first_combat_enters; m31_a11_potion_belt_four_slots; m31_a11_belt_generated_potions_fill_four | Checks two-slot and four-slot entry, use and return through settlement. |
| Generated potion fill uses free live slots; a full inventory does not replace an occupied potion | AbstractPlayer.java:2135–2155; EntropicBrew.java:48–75; Alchemize.java:23–28 | Combat | m31_a11_belt_generated_potions_fill_four; m31_a11_entropic_brew_samples_two; m31_a11_alchemize_full_loses_potion | Belted A11 fills slots 2–3 and rejects a fifth; regular A11 Brew samples two; full Alchemize loses its generated potion. |
| Potion Belt loss keeps its extra vector entries and existing contents | AbstractPlayer.java:2074–2105; PotionBelt.java:24–29 | Run / Combat | m31_a11_belt_lost_keeps_four; m31_lost_belt_kept_slot_usable; m31_lost_belt_fairy_rescue_kept_slot | A11 keeps four slots; retained A0 slot 4 and A11 slot 3 remain usable. |
| A retained extra-slot potion is not gated by Belt ownership (G2) | TopPanel.java:470–501,559–560; PotionPopUp.java:202–243,261–272; AbstractPotion.java:427–434 | Combat | m31_lost_belt_kept_slot_usable; m31_lost_belt_fairy_rescue_kept_slot | Exercises Source::Potion slot effects and Fairy auto-rescue after loss. |
| Full reward, shop and event potion rows become Disabled; Sozu discards/rejects according to the owner | RewardItem.java:299–316; StorePotion.java:74–99; Lab.java:50–64 | Run | m31_a11_belt_rows_disabled_only_when_four_full; m31_a11_shop_potion_disabled_when_full; m31_a11_sozu_potion_rows | With Belt, rows are Offered at three of four occupied and Disabled at four; Sozu reward/shop behavior is pinned. |
| Neow Three Small Potions fills the two-slot A11 inventory and disables the remaining row | NeowReward.java:210–244,288–300 | Run | m31_a11_neow_potion_bundle_two_slots | Three rows are generated; two fill the inventory and the third is Disabled. |
| Lab potion rows change from three at A14 to two at A15; A15 has two base slots | Lab.java:50–64; AbstractPlayer.java:235,292–307 | Run | m31_a15_lab_two_rows_two_slots | A14/A15 row-count boundary and claimable slots are checked. |

### §C: A12 card-upgrade chance

| Source row | Locator | Grade | Test(s) | Note |
|---|---|---|---|---|
| Exordium Act 1 remains 0.0 at A12 | context-decompiled/com/megacrit/cardcrawl/dungeons/Exordium.java:123 | Table | m31_a12_upgrade_chance_table | No A12 transition in Act 1. |
| TheCity Act 2 changes .25 to .125 | TheCity.java:94 | Table / Combat | m31_a12_upgrade_chance_table; m31_a12_reward_upgrade_boundary | Ordinary non-Rare upgrade draw is tested on either side of the A12 boundary. |
| TheBeyond Act 3 changes .5 to .25 | TheBeyond.java:89–91 | Table / Combat | m31_a12_upgrade_chance_table; m31_a12_reward_upgrade_boundary; m31_a12_equip_offer_upgrade | Combat and G1 equip offer use the same Act 3 chance. |
| TheEnding Act 4 changes .5 to .25 | TheEnding.java:172–174 | Table | m31_a12_upgrade_chance_table | Chance table is checked; A12+ branch is shared with Act 3. |
| Each non-Rare receives the upgrade draw after card identities; Rare cards skip the draw | AbstractDungeon.java:1511–1559 | Combat | m31_a12_reward_upgrade_boundary; m31_a12_equip_offer_upgrade | Same-key boundary draws separate A11 and A12; Rare cards remain unupgraded. |

### §D: A13 gold

| Source row | Locator | Grade | Test(s) | Note |
|---|---|---|---|---|
| Non-Daily boss gold draw 95–105 becomes round(binary32(base × .75)) at A13+ | AbstractRoom.java:339–350 | Combat | m31_a13_boss_gold_table; m31_a13_boss_gold_act2 | Act 1 and Act 2 Standard boss rows are compared with the hand-worked 11-value source table. |
| Golden Idol bonus is applied after the reduced boss base | AbstractRoom.java:663–670; RewardItem.java:138–155,192–199 | Combat | m31_a13_boss_gold_golden_idol | The test checks the 25% bonus against the reduced amount. |
| Normal and elite gold has no A13 reduction | AbstractRoom.java:339–350 | Combat | m31_a13_elite_and_normal_gold_unchanged | Same-seed A12/A13 Act 2 normal and elite rows match. |
| Mind Bloom Fight gives 50 gold below A13 and 25 at A13+ | MindBloom.java:85–100 | Run | m31_a13_mind_bloom_gold | Fixed reward; the event's boss-list shuffle is separate. |

### §E: A14 max HP

| Source row | Locator | Grade | Test(s) | Note |
|---|---|---|---|---|
| A14 character max-HP loss, then A6's 90% current-HP assignment | AbstractDungeon.java:2674–2687; character max-HP loss getters in source-check §E | Run | m31_a14_start_hp_standard | Public A13/A14 Standard runs check all four characters: A14 is 68/75, 59/66, 64/71 and 61/68 current/max HP. |
| Full Neow MaxHp and MaxHpLoss use b=floor(max HP × .1) at A14 | NeowReward.java:288–300; source-check §E | Run | m31_a14_neow_max_hp_rewards | All four characters; b is 7/6/7/6 and values are hand-worked. |

### §F: A15 event branches and pools

| Source row | Locator | Grade | Test(s) | Note |
|---|---|---|---|---|
| Mind Bloom — Fight, 50→25 gold at A13 | MindBloom.java:85–100 | Run | m31_a13_mind_bloom_gold | The event is an EventRoom; no boss-clear award applies. |
| Moai Head — Jump Inside, round(.125M)→round(.18M) max-HP loss | MoaiHead.java:42–70 | Run | m28_a15_m28f_moai_head_jump; m31_a14_arms_of_a15_only_tests | M28 pins A15 for all characters; M31 adds the A14 arm. |
| Winding Halls — Embrace, Focus and Retrace | WindingHalls.java:46–68,77–123 | Run | m28_a15_m28f_winding_halls_madness; m28_a15_m28f_winding_halls_rest; m28_a15_m28f_winding_halls_maze; m31_a14_arms_of_a15_only_tests | All three A15 branches; M31 adds the A14 arm for each. |
| Cursed Tome — final reading HP loss 10→15 | CursedTome.java:59–65,68–134 | Run | m28_cursed_tome_take_a15_plus | Earlier page losses remain 1/2/3. |
| Forgotten Altar — Sacrifice, round(.25M)→round(.35M) | ForgottenAltar.java:54–70,78–94 | Run | m28_forgotten_altar_sacrifice_a15_plus | Captured from pre-gain max HP; +5 max HP precedes applying it. |
| Ghosts — Apparitions 5→3; HP-loss formula unchanged | Ghosts.java:38–53,63–106 | Run | m28_ghosts_accept_a15_plus | Pins count and unchanged HP cost. |
| Nest — Rob, 99→50 gold | Nest.java:46–50 | Run | m28_the_nest_smash_and_grab_a15_plus; m31_a15_event_from_public_route | A15 value is also selected from a public Standard A15 route. |
| The Library — Sleep heal round(.33M)→round(.20M) | TheLibrary.java:48–53,77–90 | Run | m28_the_library_sleep_a15_plus | A14/A15 values use binary32 and libGDX rounding. |
| The Mausoleum — Writhe chance 50%→100%, retaining the boolean draw | TheMausoleum.java:44–46,64–88 | Run | m28_the_mausoleum_open_a15_plus; m31_a14_arms_of_a15_only_tests | M28 pins A15; M31 samples both A14 arms. |
| Cleric — Purge 50→75 gold | Cleric.java:34–59 | Run | m28_a15_cleric_purify_price | Heal option remains 35 gold. |
| Dead Adventurer — Search encounter chance 25%→35% | DeadAdventurer.java:74–80,111–125 | Run | m28_a15_dead_adventurer_thresholds | M28 test pins thresholds and source order. |
| Golden Idol Event — Touch/Take fractions .25/.08→.35/.10 | GoldenIdolEvent.java:54–75,121–140 | Run | m28_a15_golden_idol_costs; m31_a15_event_from_public_route | A15 values are checked through a public Standard A15 event route. |
| Goop Puddle — gold-loss draw 20–50→35–75 | GoopPuddle.java:44–55 | Run | m28_a15_world_of_goop_leave_loss | Loss is capped at held gold. |
| Scrap Ooze — first reach damage 3→5 | ScrapOoze.java:44–50,59–83 | Run | m28_a15_scrap_ooze_first_damage | Each failed reach adds one damage. |
| Shining Light — entry damage round(.20M)→round(.30M) | ShiningLight.java:58–66,75–120 | Run | m28_a15_shining_light_damage | Upgrade shuffle remains a separate path. |
| Sssserpent — gold 175→150 | Sssserpent.java:46–59,63–88 | Run | m28_a15_ssssserpent_gold | Fixed gold amount. |
| Designer — Adjust/Clean Up/Full Service prices 40/60/90→50/75/110; Punch HP loss 3→5 | Designer.java:60–76,168–226,240–267 | Run | m28_a15_m28f_designer_costs_and_rows; m31_a14_arms_of_a15_only_tests; m31_a15_designer_punch | M31 adds the A14 arm for the existing A15-only price assertion and applies Punch at A14 (3) and A15 (5). |
| Face Trader — Touch, gold 75→50 | FaceTrader.java:46–54,70–90,125 | Run | m28_a15_face_trader_touch | Touch damage is unchanged. |
| Gold Shrine — Pray 100→50 gold | GoldShrine.java:51–56,70–100 | Run | m28_a15_golden_shrine_pray | Desecrate remains 275 gold. |
| Gremlin Match Game — colorless uncommon replaced by a second random curse | GremlinMatchGame.java:73–109 | Run | m28_a15_match_and_keep_second_curse; m28_a15_match_and_keep_still_five_attempts | Six-card board count and five attempts remain. |
| Gremlin Wheel Game — losing damage trunc(.10M)→trunc(.15M) | GremlinWheelGame.java:105–118,247–255,278–322 | Run | m28_a15_wheel_of_change_hp_loss | The independent animation RNG draw follows the result. |
| Lab — potion reward rows 3→2 | Lab.java:50–64 | Run | m28_a15_lab_potions; m31_a15_lab_two_rows_two_slots | A14/A15 Lab row count and A15 two-slot inventory are pinned. |
| Woman in Blue — fourth option changes from Leave to Punch for ceil(.05M) HP loss | WomanInBlue.java:42–59,61–110 | Run | m28_woman_in_blue_leave_a15_plus | Three potion purchase options remain unchanged. |
| Note For Yourself — excluded from the one-time pool at A15 | AbstractDungeon.java:1439–1466; event_pool.rs:35–59 | Run | m28_a15_and_act_gates_of_the_one_time_rows; m31_a15_note_for_yourself_pool_gate | M31 checks A14/A15 eligibility on the public Standard route. |

### §G: other A11–A15 sites

Nothing else changes in 11–15. The source sweep found no A11–A15 change to shop prices, rest healing, relic effects, potion potency, card-reward row count, map generation, Neow's normal reward path, keys or monster setup. Potion Belt's +2 is an item effect at every ascension; Lab's A15 potion-row reduction is covered above.

## Reconciliation

This section records candidate changes and amendments against the accepted M28–M30 tests. It does not claim final milestone acceptance. Original bytes for amended accepted assertions are retained in $E/m31/amended-original/.

| ID | Change | Source or ruling | Pinning tests | Amended accepted assertions | Status |
|---|---|---|---|---|---|
| E1 | Standard admits A0–A15 in the default build; A16 is the first rejected level; error text is Unsupported("Standard supports A0–A15"). | Source-check §A; plan item 1; review-v2. | m31_standard_creation_a0_to_a15_all_characters; m31_standard_rejects_a16_and_above. | Retarget m28_standard_creation_all_characters and m28_standard_a0_is_public_in_a_default_build from rejecting A11 to rejecting A16 and update the error. Retarget m29_standard_rejects_above_cap to admit A11/A15 while retaining 16/20/21 rejection. Rename m30_standard_rejects_a11_and_above to m30_standard_rejects_above_cap, with a Formerly doc comment; it rejects 16/20/21. The A0–A10 creation test remains unchanged. | Source candidate; originals retained. |
| E2 | A11 Run/Complete contexts use a two-slot base; authored and finite A0 contexts retain three. | Source-check §B; task0-bind §(b). | m31_a11_two_slots_before_neow; m31_a11_first_combat_enters. | No accepted test assertion changes the finite-profile contract. | Candidate. |
| E3 | Potion Belt adds two to the live base: four slots at A11, five below A11; entry, use and settlement keep the vector. | Source-check §B; design §6.2–§6.3. | m31_a11_potion_belt_four_slots; m31_a0_potion_belt_unchanged; m31_a11_belt_generated_potions_fill_four. | Existing finite three-to-five tests remain unchanged. | Candidate. |
| E4 | G2 permits potion use from an extra slot retained after Belt loss, at every ascension; the slot's potion may also trigger Fairy rescue. | Source-check §B; design §6.3; progress-v1 narrowing. | m31_lost_belt_kept_slot_usable; m31_a11_belt_lost_keeps_four; m31_lost_belt_fairy_rescue_kept_slot. | No accepted rule assertion changed. | Candidate. |
| E5 | Complete potion capacity is based on the ascension base and live vector length for generation, Entropic Brew, Disabled rows and Sozu behavior. | Source-check §B; Task 0(b) census. | m31_a11_belt_generated_potions_fill_four; m31_a11_entropic_brew_samples_two; m31_a11_alchemize_full_loses_potion; m31_a11_belt_rows_disabled_only_when_four_full; m31_a11_shop_potion_disabled_when_full; m31_a11_sozu_potion_rows; m31_a11_neow_potion_bundle_two_slots; m31_a15_lab_two_rows_two_slots. | Three A0 fixture edits also size potion slots from their edited ascension before combat: m28_a15_m28f_mysterious_sphere_unsupported_and_leave, m28_a15_m28f_secret_portal_enter_unsupported_and_leave, and m28_event_combat_unsupported_ascension_is_rejected_up_front. Their original bytes are retained. | Candidate. |
| E6 | A12 halves Standard automatic-upgrade chance in Acts 2–4; A1 stays 0. | Source-check §C; plan item 4. | m31_a12_upgrade_chance_table; m31_a12_reward_upgrade_boundary; m31_a12_equip_offer_upgrade. | m28_card_reward_upgrade_chance_by_act keeps its name and A0 values; its call sites now pass ascension 0, with original bytes retained. | Candidate. |
| E7 | G1 visible Standard equip rows use the source room threshold and ordinary reward law; shared offset writes and card modifiers follow the ordinary path. | Equip-offers research §§1–4; task0-bind §(c); review I1/I2. | m31_equip_orrery_shop_law; m31_equip_tiny_house_law; m31_equip_dream_catcher_law; m31_equip_egg_preview; m31_equip_keys_disjoint; m31_a12_equip_offer_upgrade. | G1 is Standard-only where a live card rarity offset exists. Non-Standard Complete diagnostics retain C/U/U and are pinned by m31_legacy_equip_row_on_diagnostics. M27 equip assertions remain unchanged. | Candidate. |
| E7b | G1 hidden rows at Cauldron and Calling Bell are generated and discarded, so their rarity rolls update the offset. | Equip-offers research §2; task0-bind §(c); PM ruling G1(b). | m31_hidden_row_moves_offset; m31_hidden_row_calling_bell. | Neow Three Small Potions already generates and discards its card row; it remains byte-for-byte unchanged apart from the stale Neow comment. m28_neow_three_potions_discards_card_row stays the pin. | Candidate. |
| E8 | A13 scales the Act 1/2 boss base before Golden Idol and other reward bonuses. | Source-check §D; plan item 5; review m4. | m31_a13_boss_gold_table; m31_a13_boss_gold_act2; m31_a13_boss_gold_golden_idol; m31_a13_elite_and_normal_gold_unchanged. | No M5 captured-reward assertion changes; that path remains A0. Act 3 boss and Heart have no reward screen. | Candidate. |
| E9 | Mind Bloom's Fight reward is 25 gold from A13. | Source-check §§A, D and F. | m31_a13_mind_bloom_gold. | No accepted assertion changed. | Candidate. |
| E10 | A14 max-HP and Neow combinations are pinned through public Standard creation. | Source-check §E; plan item 6. | m31_a14_start_hp_standard; m31_a14_neow_max_hp_rewards. | No A14 rule code changes; tests only. | Candidate. |
| E11 | A15 event rules and the Note For Yourself pool gate are pinned at their boundary and through public Standard creation. | Source-check §F; Task 0(d) reach record; plan item 7. | The §F table maps all 22 A15-effect events to their M28 test; m31_a14_arms_of_a15_only_tests adds the A14 arm for Moai, Winding Halls, Mausoleum and Designer; m31_a15_designer_punch asserts its HP loss; m31_a15_note_for_yourself_pool_gate and m31_a15_event_from_public_route use public Standard runs. | No A15 event behavior code changes; tests only. | Candidate. |
| G3 | Standard shop colored stock and Courier restock use ShopRoom's 9/37 threshold at every ascension. | PM ruling G3; task0-bind §(c). | m31_shop_stock_shop_thresholds; m31_shop_courier_restock_shop_thresholds. | The Courier restock law is shared with the non-Standard Act 4 Complete shop, which therefore also restocks at 9/37 (diagnostics only). | Candidate. |
| A0 attribution | Per-Run markers and public per-step decision/action digests classify A0–A10 differences; the join compares candidate/base digests and accepts a delta only at or after its relevant marker. | Review-v2 n1; PM ruling G3. | Candidate traversal plus $E/m31/scripts/a0_attribution_join.py. | Markers: first_g1_step, belt_lost_step, kept_slot_potion_step and first_shop_step. Neow Three Small Potions is not a G1 marker. Each step emits the first 48 bits of SHA-256 over the step, public decision and chosen action. | c1: every A0 (diagnostics 300 Runs, default 200) and A1–A10 (default, 200 per level) Run whose digests differ from base 5e2a10b4 carries a G3 or G1 marker at or before its first differing step; 0 findings in all 12 joins (A0 diagnostics 84 differing Runs, 81 G3 / 3 G1; A0 default 41; A1–A10 374). No G2 Run occurred. Limit (final review m3): a G3 attribution only means the first difference is at or after the first shop; 10 of 41 (A0 default), 27 of 84 (A0 diagnostics) and 2–5 per A1–A10 level first differ well after it, and the review found no other engine change reachable at A0–A10. Frozen replay: 323/323 inputs equal. Fuzz gate: 0 failure groups on either side (summary hashes differ, as expected from G3). |
