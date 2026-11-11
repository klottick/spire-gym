# M30 Ascension 6–10

Status: merged as PR92 (`5e2a10b`); this is the finite M30 source profile, not a separate spire-env, Gym or Python profile.

This profile covers Standard Runs at A0–A10 in a default build. It records the source rules, coverage map and changes from the accepted M28 and M29 contracts. Its finite source evidence does not claim original-runtime validation.

## What M30 is

M30 raises Standard’s default-build cap to A10 for all four characters. At A6, current HP becomes libGDX round(max HP × 0.9f), using the binary32 product and applying it after the A14 max-HP loss. At A10, Ascender’s Bane is inserted at master-deck index 0 before Neow. Starter-card instance numbering stays as it was; the Bane takes the next instance number. The Bane remains a NonremovableCurse and Ethereal card through its removal, copy, obtain-hook, and combat paths.

No enemy rules change in M30. The source cross-check maps all 68 A6–A10 monster rows to M29 tests: 68 MATCH, 0 untested, 0 mismatch, 0 missing. Task 0(b) re-confirmed that the M29d candidate and main differ in enemy code only by comments in ascension.rs; combat/turns.rs is identical. The cross-check therefore stands.

The global fixes apply at every ascension:

- G1: Toke offers only unbottled purgeable cards, is Disabled when none exist, and offers Cancel after opening. If it is the only campfire option and is Disabled, the campfire completes on entry.
- G2: the shop purge row is Disabled when the deck has no unbottled purgeable card. Buying a card re-enables the same row.
- G3: Run::remove_card uses the shared removal owner after its existing guards. It refuses all three unremovable curses and performs the normal bottle, Parasite, and CardRemoved effects.
- G4: every Complete-admission Run (the Standard Run; the M26/M27 supplied diagnostics already pre-sort their decks by the same key, so their order is unchanged) shuffles the master deck in its stored order for the Watcher. The M21–M25 finite Watcher profiles keep their canonical sort.
- G5: Red Skull's Strength latch is armed only by damage, as the source's `onBloodied` is. Burning Blood's victory heal could re-arm it after the settlement reset, while the player was still at half HP or below; the stale counter then rejected the next combat's entry (`InvalidAction` on a map move). The A8/A9 supplied traversal found it (Ironclad/5, floor 30), and the base with only the cap raised faults the same way, so it predates M30.

**Compatibility.** Standard now returns Unsupported("Standard supports A0–A10") above A10. G4 changes Standard Watcher combat replays from their first combat, including at A0–A5. The Bane’s index 0 changes the A10 opening shuffle for every character. The A0–A5 Toke and shop changes are disclosed under G1 and G2.

## Limits and Outs

- A11+ rules remain for M31–M33: potion slots, upgrades, boss gold, A14 HP, A15 events, A16 shop changes, and the A20 double boss.
- Enemy behavior rows at A17+ remain for M32. M30 adds no enemy rule.
- M30 adds no separate spire-env or Python profile. Standard Run public exposure merged in M34; M34s and M35 added search and batching.
- Original-runtime validation is out of scope. Source-check §G cannot determine whether a ShowCardAndObtainEffect update reaches onObtainCard after Omamori; callback timing depends on frame delta. Test5’s normal pool registration is also not determinable statically.
- Strategy tuning and win-rate targets are out of scope.
- **Traversal reach stops at Act 2 at A6–A10.** No A6–A10 traversal Run reached an Act 3 boss, Act 4 or the Heart; the supplied mode reached 15/10/10/9/7 Act 2 bosses at A6–A10 (`$E/m30/c2-checks/r2-traversal-summary.md`). The A8 rows for Act 3 and Act 4 elites and bosses (Giant Head, Nemesis, Reptomancer, Spire Shield and Spear) and the A9 rows (Time Eater, Awakened One, Donu and Deca, the Heart) rest on the M29 combat tests (the cross-check MATCH), not on Run reachability at their thresholds. "0 faults at every level" covers the Acts the traversal reached.
- The traversal rarely reaches the G1/G2 states (Toke was offered in 3–5 Runs per level at A7–A9 and never disabled or cancelled; no empty purge). G1 and G2 rest on the Run-grade unit tests and the lifecycle matrix, not on traversal coverage.

## Evidence index

Paths beginning `$E` are under `E=<private-evidence>/m30-execution-20261103` (private evidence store, not published here).

- Static source record: $E/planning-v2/research/m30-source-check-v1.md
- Accepted design and implementation plan: $E/planning-v2/2026-11-03-m30-ascension6-10-design.md and 2026-11-03-m30-ascension6-10-plan.md
- Engine audit and enemy cross-check: $E/planning-v2/research/m30-engine-audit-v1.md and m30-enemy-crosscheck-v1.md
- Main-tree binding and enemy re-confirmation: $E/m30/task0-bind.md
- Plan review: $E/plan-review-v1/review-v2.md
- Runner receipts on the read-only candidate snapshot c2 (manifest `$E/m30/candidate-c2-snapshot.sha256`): `$E/m30/c2-checks/results-a.md` and `results-b.md`, with the c1 base outputs in `$E/m30/c1-checks/`. The c1 run and its fixes are in `$E/m30/c1-disposition-v1.md`.
  - Workspace: 4681 passed, 0 failed, 11 ignored. Test names vs base: the M30 `m30_` additions and the two documented renames only.
  - Diagnostics lanes: 4054/0/5 (`m28-diagnostics,m29-diagnostics`), spire-fuzz 17/0/0, `m28_ m29_ m30_` 567/0/0, shapes 1/0/0. Python D1 checks pass.
  - Traversal, 25 seeds per character: A0 (diagnostics and default) and A1–A5 (default) are fault-free. Every Run that differs from base 417ad9ce is a Watcher Run (G4, traced to the first combat's opening hand); Ironclad, Silent and Defect records and census equal the base. A6–A10, default (uniform, reach) and diagnostics (plus supplied), have 0 faults, 0 interim rejections and 0 blocked Runs at every level. Start HP equals §B, and every A10 Run has the Bane at index 0 at creation and at least one at its end.
  - Fuzz gate (seeds 1–100, four drivers): 0 failures on both sides; no candidate-only failure group.
  - Frozen replay (323 inputs): 323 equal, 0 different on the raw compare.

## Coverage ledger

The source record is the static check of desktop-1.0.jar, SHA-256 cfad868ac8d65a88e71a0bf096fb09f78811e553effe0787c5309a655e081673. Grades are Run, Combat, or Table. The §A monster test names are those in m30-enemy-crosscheck-v1.md; file prefixes are relative to crates/spire-core/src/combat/tests/.

### §A: run and progression rows

| Source row | Locator | Grade | Test | Note |
|---|---|---|---|---|
| A6 AbstractDungeon: start HP becomes round(max HP × 0.9f) | verification/2026-11-03-m29-asc-a-v1/decompiled/com/megacrit/cardcrawl/dungeons/AbstractDungeon.java:2682 | Run | m30_a6_start_hp_all_characters; all_84_starts | Eight literals are listed under §B. |
| A10 AbstractDungeon: insert Ascender’s Bane and mark its ID seen | verification/2026-11-03-m29-asc-a-v1/decompiled/com/megacrit/cardcrawl/dungeons/AbstractDungeon.java:2685 | Run | m30_bane_first_before_neow; m30_bane_insertion_dispatches_no_item_event | Insertion and absence of obtain events are checked under §D; the seen marker is source-derived. |
| A10 DeathScreen achievement | verification/2026-11-03-m29-asc-a-v1/decompiled/com/megacrit/cardcrawl/screens/DeathScreen.java:188 | Table | None | Progression-only; outside Run rules. |
| A10 VictoryScreen achievement | verification/2026-11-03-m29-asc-a-v1/decompiled/com/megacrit/cardcrawl/screens/VictoryScreen.java:168 | Table | None | Progression-only; outside Run rules. |

### §A: monster rows

| Source row | Locator | Grade | M29 test from cross-check | Note |
|---|---|---|---|---|
| A9 AwakenedOne HP 300→320; rebirth max HP 300→320 | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/monsters/beyond/AwakenedOne.java:158,259 | Combat | m29_d_values.rs::m29_awakened_one_ascension_values | 68/68 MATCH. |
| A7 Darkling HP 48–56→50–59 | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/monsters/beyond/Darkling.java:101 | Combat | m29_d_values.rs::m29_darkling_ascension_values | 68/68 MATCH. |
| A9 Deca HP 250→265 | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/monsters/beyond/Deca.java:90 | Combat | m29_d_values.rs::m29_deca_ascension_values | 68/68 MATCH. |
| A9 Donu HP 250→265 | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/monsters/beyond/Donu.java:83 | Combat | m29_d_values.rs::m29_donu_ascension_values | 68/68 MATCH. |
| A7 Exploder HP 30→30–35 | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/monsters/beyond/Exploder.java:70 | Combat | m29_d_values.rs::m29_exploder_ascension_values | 68/68 MATCH. |
| A8 Giant Head HP 500→520 | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/monsters/beyond/GiantHead.java:82 | Combat | m29_d_values.rs::m29_giant_head_ascension_values | 68/68 MATCH. |
| A8 Nemesis HP 185→200 | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/monsters/beyond/Nemesis.java:106 | Combat | m29_d_values.rs::m29_nemesis_ascension_values | 68/68 MATCH. |
| A7 Orb Walker HP 90–96→92–102 | verification/2026-11-03-m28f-v1/decompiled/OrbWalker/com/megacrit/cardcrawl/monsters/beyond/OrbWalker.java:73 | Combat | m29_d_values.rs::m29_orb_walker_ascension_values | 68/68 MATCH. |
| A8 Reptomancer HP 180–190→190–200 | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/monsters/beyond/Reptomancer.java:96 | Combat | m29_d_values.rs::m29_reptomancer_ascension_values | 68/68 MATCH. |
| A7 Repulsor HP 29–35→31–38 | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/monsters/beyond/Repulsor.java:65 | Combat | m29_d_values.rs::m29_repulsor_ascension_values | 68/68 MATCH. |
| A7 Spiker HP 42–56→44–60 | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/monsters/beyond/Spiker.java:73 | Combat | m29_d_values.rs::m29_spiker_ascension_values | 68/68 MATCH. |
| A7 Spire Growth HP 170→190 | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/monsters/beyond/SpireGrowth.java:71 | Combat | m29_d_values.rs::m29_spire_growth_ascension_values | 68/68 MATCH. |
| A9 Time Eater HP 456→480 | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/monsters/beyond/TimeEater.java:112 | Combat | m29_d_values.rs::m29_time_eater_ascension_values | 68/68 MATCH. |
| A7 Writhing Mass HP 160→175 | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/monsters/beyond/WrithingMass.java:85 | Combat | m29_d_values.rs::m29_writhing_mass_ascension_values | 68/68 MATCH. |
| A7 Bandit Bear HP 38–42→40–44 | verification/2026-11-02-m28e-b-v1/decompiled/com/megacrit/cardcrawl/monsters/city/BanditBear.java:75 | Combat | m29_d_values.rs::m29_bandit_bear_ascension_values | 68/68 MATCH. |
| A7 Bandit Leader HP 35–39→37–41 | verification/2026-11-02-m28e-b-v1/decompiled/com/megacrit/cardcrawl/monsters/city/BanditLeader.java:78 | Combat | m29_d_values.rs::m29_bandit_leader_ascension_values | 68/68 MATCH. |
| A7 Bandit Pointy HP 30→34 | verification/2026-11-02-m28e-b-v1/decompiled/com/megacrit/cardcrawl/monsters/city/BanditPointy.java:60 | Combat | m29_d_values.rs::m29_bandit_pointy_ascension_values | 68/68 MATCH. |
| A8 Book of Stabbing HP 160–164→168–172 | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/monsters/city/BookOfStabbing.java:80 | Combat | m29_c_elites_bosses.rs::m29_book_of_stabbing_ascension_values | 68/68 MATCH. |
| A9 Bronze Automaton block 9→12 | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/monsters/city/BronzeAutomaton.java:108 | Combat | m29_d_values.rs::m29_bronze_automaton_ascension_values | 68/68 MATCH. |
| A9 Bronze Orb HP 52–58→54–60 | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/monsters/city/BronzeOrb.java:69 | Combat | m29_d_values.rs::m29_bronze_orb_ascension_values | 68/68 MATCH. |
| A7 Byrd HP 25–31→26–33 | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/monsters/city/Byrd.java:100 | Combat | m29_d_values.rs::m29_byrd_ascension_values | 68/68 MATCH. |
| A7 Centurion HP 76–80→78–83 | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/monsters/city/Centurion.java:71 | Combat | m29_d_values.rs::m29_centurion_ascension_values | 68/68 MATCH. |
| A9 Champ HP 420→440; Forge 5→6; block 15→18 | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/monsters/city/Champ.java:138,150 | Combat | m29_d_values.rs::m29_champ_ascension_values | 68/68 MATCH. |
| A7 Chosen HP 95–99→98–103 | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/monsters/city/Chosen.java:103 | Combat | m29_d_values.rs::m29_chosen_ascension_values | 68/68 MATCH. |
| A8 Gremlin Leader HP 140–148→145–155 | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/monsters/city/GremlinLeader.java:95 | Combat | m29_c_elites_bosses.rs::m29_gremlin_leader_ascension_values | 68/68 MATCH. |
| A7 Healer HP 48–56→50–58 | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/monsters/city/Healer.java:79 | Combat | m29_d_values.rs::m29_mystic_ascension_values | 68/68 MATCH. |
| A7 Mugger HP 48–52→50–54 | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/monsters/city/Mugger.java:86 | Combat | m29_d_values.rs::m29_mugger_ascension_values | 68/68 MATCH. |
| A7 Shelled Parasite HP 68–72→70–75 | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/monsters/city/ShelledParasite.java:109 | Combat | m29_d_values.rs::m29_shelled_parasite_ascension_values | 68/68 MATCH. |
| A7 Snake Plant HP 75–79→78–82 | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/monsters/city/SnakePlant.java:85 | Combat | m29_d_values.rs::m29_snake_plant_ascension_values | 68/68 MATCH. |
| A7 Snecko HP 114–120→120–125 | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/monsters/city/Snecko.java:100 | Combat | m29_d_values.rs::m29_snecko_ascension_values | 68/68 MATCH. |
| A8 Taskmaster HP 54–60→57–64 | verification/2026-11-02-m28e-c-v1/decompiled/com/megacrit/cardcrawl/monsters/city/Taskmaster.java:46 | Combat | m29_c_elites_bosses.rs::m29_taskmaster_ascension_values | 68/68 MATCH. |
| A9 The Collector HP 282→300; block 15→18 | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/monsters/city/TheCollector.java:123 | Combat | m29_d_values.rs::m29_collector_ascension_values | 68/68 MATCH. |
| A9 Torch Head HP 38–40→40–45 | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/monsters/city/TorchHead.java:64 | Combat | m29_d_values.rs::m29_torch_head_ascension_values | 68/68 MATCH. |
| A9 Corrupt Heart HP 750→800 | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/monsters/ending/CorruptHeart.java:112 | Combat | m29_d_values.rs::m29_corrupt_heart_ascension_values | 68/68 MATCH. |
| A8 Spire Shield HP 110→125 | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/monsters/ending/SpireShield.java:76 | Combat | m29_d_values.rs::m29_spire_shield_ascension_values | 68/68 MATCH. |
| A8 Spire Spear HP 160→180 | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/monsters/ending/SpireSpear.java:79 | Combat | m29_d_values.rs::m29_spire_spear_ascension_values | 68/68 MATCH. |
| A7 Acid Slime L HP 65–69→68–72 | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/monsters/exordium/AcidSlime_L.java:110 | Combat | m29_b_act1_normals.rs::m29_acid_slime_l_ascension_values | 68/68 MATCH. |
| A7 Acid Slime M HP 28–32→29–34 | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/monsters/exordium/AcidSlime_M.java:81 | Combat | m29_b_act1_normals.rs::m29_acid_slime_m_ascension_values | 68/68 MATCH. |
| A7 Acid Slime S HP 8–12→9–13 | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/monsters/exordium/AcidSlime_S.java:63 | Combat | m29_b_act1_normals.rs::m29_acid_slime_s_ascension_values | 68/68 MATCH. |
| A7 Cultist HP 48–54→50–56 | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/monsters/exordium/Cultist.java:78 | Combat | m29_b_act1_normals.rs::m29_cultist_ascension_values | 68/68 MATCH. |
| A7 Fungi Beast HP 22–28→24–28 | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/monsters/exordium/FungiBeast.java:73 | Combat | m29_b_act1_normals.rs::m29_fungi_beast_ascension_values | 68/68 MATCH. |
| A7 Fat Gremlin HP 13–17→14–18 | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/monsters/exordium/GremlinFat.java:72 | Combat | m29_b_act1_normals.rs::m29_gremlin_fat_ascension_values | 68/68 MATCH. |
| A8 Gremlin Nob HP 82–86→85–90 | verification/2026-11-02-m28e-c-v1/decompiled/com/megacrit/cardcrawl/monsters/exordium/GremlinNob.java:61 | Combat | m29_c_elites_bosses.rs::m29_gremlin_nob_ascension_values | 68/68 MATCH. |
| A7 Sneaky Gremlin HP 10–14→11–15 | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/monsters/exordium/GremlinThief.java:62 | Combat | m29_b_act1_normals.rs::m29_gremlin_sneaky_ascension_values | 68/68 MATCH. |
| A7 Shield Gremlin HP 12–15→13–17; Protect block 7→8 | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/monsters/exordium/GremlinTsundere.java:70 | Combat | m29_b_act1_normals.rs::m29_gremlin_shield_ascension_values | 68/68 MATCH. |
| A7 Mad Gremlin HP 20–24→21–25 | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/monsters/exordium/GremlinWarrior.java:67 | Combat | m29_b_act1_normals.rs::m29_gremlin_mad_ascension_values | 68/68 MATCH. |
| A7 Gremlin Wizard HP 21–25→22–26 | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/monsters/exordium/GremlinWizard.java:67 | Combat | m29_b_act1_normals.rs::m29_gremlin_wizard_ascension_values | 68/68 MATCH. |
| A9 Hexaghost HP 250→264 | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/monsters/exordium/Hexaghost.java:139 | Combat | m29_c_elites_bosses.rs::m29_hexaghost_ascension_values | 68/68 MATCH. |
| A7 Jaw Worm HP 40–44→42–46 | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/monsters/exordium/JawWorm.java:105 | Combat | m29_b_act1_normals.rs::m29_jaw_worm_ascension_values | 68/68 MATCH. |
| A8 Lagavulin HP 109–111→112–115 | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/monsters/exordium/Lagavulin.java:101 | Combat | m29_c_elites_bosses.rs::m29_lagavulin_ascension_values | 68/68 MATCH. |
| A7 Looter HP 44–48→46–50 | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/monsters/exordium/Looter.java:89 | Combat | m29_b_act1_normals.rs::m29_looter_ascension_values | 68/68 MATCH. |
| A7 Green Louse HP 11–17→12–18; Curl Up 3–7→4–8 | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/monsters/exordium/LouseDefensive.java:82,94 | Combat | m29_b_act1_normals.rs::m29_louse_green_ascension_values; m29_louse_bite_and_curl_up_ranges | 68/68 MATCH. |
| A7 Red Louse HP 10–15→11–16; Curl Up 3–7→4–8 | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/monsters/exordium/LouseNormal.java:73,85 | Combat | m29_b_act1_normals.rs::m29_louse_red_ascension_values; m29_louse_bite_and_curl_up_ranges | 68/68 MATCH. |
| A8 Sentry HP 38–42→39–45 | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/monsters/exordium/Sentry.java:91 | Combat | m29_c_elites_bosses.rs::m29_sentry_ascension_values | 68/68 MATCH. |
| A7 Slaver Blue HP 46–50→48–52 | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/monsters/exordium/SlaverBlue.java:66 | Combat | m29_c_elites_bosses.rs::m29_slaver_blue_ascension_values | 68/68 MATCH. |
| A7 Slaver Red HP 46–50→48–52 | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/monsters/exordium/SlaverRed.java:83 | Combat | m29_c_elites_bosses.rs::m29_slaver_red_ascension_values | 68/68 MATCH. |
| A9 Slime Boss HP 140→150 | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/monsters/exordium/SlimeBoss.java:125 | Combat | m29_c_elites_bosses.rs::m29_slime_boss_ascension_values | 68/68 MATCH. |
| A7 Spike Slime L HP 64–70→67–73 | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/monsters/exordium/SpikeSlime_L.java:103 | Combat | m29_b_act1_normals.rs::m29_spike_slime_l_ascension_values | 68/68 MATCH. |
| A7 Spike Slime M HP 28–32→29–34 | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/monsters/exordium/SpikeSlime_M.java:74 | Combat | m29_b_act1_normals.rs::m29_spike_slime_m_ascension_values | 68/68 MATCH. |
| A7 Spike Slime S HP 10–14→11–15 | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/monsters/exordium/SpikeSlime_S.java:57 | Combat | m29_b_act1_normals.rs::m29_spike_slime_s_ascension_values | 68/68 MATCH. |
| A9 Guardian HP 240→250; Mode Shift 30→35 | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/monsters/exordium/TheGuardian.java:132 | Combat | m29_c_elites_bosses.rs::m29_guardian_ascension_values | 68/68 MATCH. |

The A7 Shield Gremlin Protect value is the corrected name for the source-check’s GremlinTsundere row. The cross-check maps it to the M29 HP and Block assertions. No A6–A10 enemy row lacks a M29 assertion.

### §B: A6 start HP

The source law is MathUtils.round((float)maxHP * 0.9f), with a binary32 product. It runs after the A14 max-HP loss. The eight post-round HP literals are:

| Character | A0 max HP | A6–A13 HP | A14+ max HP | A14+ HP | Locator | Grade | Test | Note |
|---|---:|---:|---:|---:|---|---|---|---|
| Ironclad | 80 | 72 | 75 | 68 | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/characters/Ironclad.java:137,182 | Run | m30_a6_start_hp_all_characters; all_84_starts | A13/A14 use Run::new, as plan review v2 n2 specifies. |
| Silent | 70 | 63 | 66 | 59 | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/characters/TheSilent.java:141,186 | Run | m30_a6_start_hp_all_characters; all_84_starts | A13/A14 use Run::new, as plan review v2 n2 specifies. |
| Defect | 75 | 68 | 71 | 64 | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/characters/Defect.java:138,183 | Run | m30_a6_start_hp_all_characters; all_84_starts | A13/A14 use Run::new, as plan review v2 n2 specifies. |
| Watcher | 72 | 65 | 68 | 61 | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/characters/Watcher.java:177,229 | Run | m30_a6_start_hp_all_characters; all_84_starts | A13/A14 use Run::new, as plan review v2 n2 specifies. |
| Rounding helper | Binary32 product rounded by libGDX’s offset-and-cast rule | — | — | — | verification/2026-11-02-m28e-a-v1/decompiled/com/badlogic/gdx/math/MathUtils.java:234–235 | Run | m30_a6_start_hp_all_characters | The literals are copied from §B, not computed in the test. |

Neow Max HP, Max HP loss, and Current HP loss read the A6 start HP. The hand-worked literals are pinned by m30_neow_hp_rewards_and_drawbacks_at_a6. The source ordering and A14 input are in §B of the source record.

### §C: Ascender’s Bane combat behavior

| Source row | Locator | Grade | Test | Note |
|---|---|---|---|---|
| Curse, SPECIAL rarity, cost −2, Ethereal; no use, upgrade, or retain override | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/cards/curses/AscendersBane.java:25–40; verification/2026-11-02-m28e-c-v1/decompiled/com/megacrit/cardcrawl/cards/AbstractCard.java:657–667, 2155–2164 | Combat | m30_bane_combat_ethereal_exhaust_fires_exhaust_hooks | Ethereal exhausts at turn end. Upgrade eligibility is mapped under §E. |
| Blue Candle plays a Curse, costs 1 HP, then exhausts it | verification/2026-11-02-m28e-c-v1/decompiled/com/megacrit/cardcrawl/cards/AbstractCard.java:839–875, 895–909; verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/relics/BlueCandle.java:40–49 | Combat | m30_bane_combat_blue_candle_plays_it | Bane itself has no use effect. |
| Medical Kit applies to Status, not Curse | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/relics/MedicalKit.java:30–37; verification/2026-11-02-m28e-c-v1/decompiled/com/megacrit/cardcrawl/cards/AbstractCard.java:895–909 | Combat | m30_bane_combat_medical_kit_does_not_enable | Bane stays unplayable. |
| Runic Pyramid does not stop Ethereal exhaust | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/relics/RunicPyramid.java:15–35; verification/2026-11-03-endturn-v1/src/com/megacrit/cardcrawl/actions/common/DiscardAtEndOfTurnAction.java:26–48 | Combat | m30_bane_combat_runic_pyramid_still_exhausts | Tests the end-turn path. |
| Exhume returns Bane; it exhausts again | m30-source-check-v1.md §C; crates/spire-core/src/combat/tests/m10_selection.rs:430–435 | Combat | m30_bane_combat_exhume_returns_and_exhausts_again | Two exhaust notifications are asserted. |
| Well-Laid Plans and Equilibrium do not save Ethereal Bane | RetainCardsAction.java:27,34; EquilibriumPower.java:41; crates/spire-core/src/combat/effects.rs:584–592 | Combat | m30_bane_combat_retain_effects_do_not_save_bane | The source offers Bane in the retain picker; it does not retain it. |
| Du-Vu Doll counts Bane as a Curse | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/relics/DuVuDoll.java:53–71 | Combat | m30_bane_combat_du_vu_doll_counts_it | Bane is counted by card type. |

### §D: Starting deck insertion

| Source row | Locator | Grade | Test | Note |
|---|---|---|---|---|
| A10 direct insertion before starter deck and Neow; no obtain effect | verification/2026-11-02-m28e-c-v1/decompiled/com/megacrit/cardcrawl/dungeons/AbstractDungeon.java:2574–2579; verification/2026-11-03-m29-asc-a-v1/decompiled/com/megacrit/cardcrawl/dungeons/AbstractDungeon.java:2678–2687; verification/2026-11-02-m28e-c-v1/decompiled/com/megacrit/cardcrawl/cards/CardGroup.java:438–440; verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/neow/NeowRoom.java:19–22 | Run | m30_bane_first_before_neow; m30_bane_insertion_dispatches_no_item_event | Bane is deck index 0. The starters follow. |
| Master deck order feeds the seeded opening shuffle | m30-source-check-v1.md §D; m30-engine-audit-v1.md §4 | Combat | m30_bane_position_feeds_opening_shuffle | At A10, index 0 changes the opening shuffle. |
| A10 combat settlement keeps the master-deck Bane | m30-engine-audit-v1.md §4; crates/spire-core/src/run/combat.rs:58–70,278–299 | Run | m30_a10_combat_settlement_keeps_bane | Combat exhaust does not remove the master-deck card. |

### §E: removal, selection, transform, copy, store, and upgrade sites

The matrix tests are in `crates/spire-core/src/run/tests/m30_bane_lifecycle.rs`; each runs a Standard A10 Run through the public route, with Necronomicurse, Curse of the Bell and a bottled card where the predicate applies.

| Source row | Locator | Grade | Test | Note |
|---|---|---|---|---|
| CardGroup.getPurgeableCards | verification/2026-11-02-m28e-c-v1/decompiled/com/megacrit/cardcrawl/cards/CardGroup.java:961–967 | Table | `m30_bane_matrix_get_purgeable` | Excludes Bane, Necronomicurse, and Curse of the Bell; bottles are a caller filter. |
| Neow: remove one | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/neow/NeowReward.java:270 | Run | `m30_bane_matrix_neow_remove_one` | Bane is not offered. The engine uses `purgeable` where the source uses the unfiltered getPurgeableCards; they are equivalent because no bottle can exist before the first Neow choice (design §6.4), and the test fixture has no bottle. |
| Neow: remove two | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/neow/NeowReward.java:274 | Run | `m30_bane_matrix_neow_remove_two`, `m30_bane_matrix_neow_short_remove_two` | Fixed two-card count. |
| Neow: transform one | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/neow/NeowReward.java:303 | Run | `m30_bane_matrix_neow_transform_one` | Bane is not offered. The engine uses `purgeable` where the source uses the unfiltered getPurgeableCards; they are equivalent because no bottle can exist before the first Neow choice (design §6.4), and the test fixture has no bottle. |
| Neow: transform two | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/neow/NeowReward.java:307 | Run | `m30_bane_matrix_neow_transform_two` | Fixed two-card count. |
| Shop purge | verification/2026-11-03-m29-asc-a-v1/decompiled/com/megacrit/cardcrawl/shop/ShopScreen.java:256, 328–337, 1007 | Run | `m30_bane_matrix_shop_purge`, `m30_bane_matrix_shop_purge_empty` | M30 availability is pinned by `m30_shop_purge_*` below G2. |
| Peace Pipe option | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/relics/PeacePipe.java:53 | Run | `m30_bane_matrix_peace_pipe_candidates`, `m30_bane_matrix_peace_pipe_empty_status` | G1 also has direct tests. |
| Empty Cage | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/relics/EmptyCage.java:49–60 | Run | `m30_bane_matrix_empty_cage` | Bane is excluded; the caller does not filter bottled cards. |
| Astrolabe | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/relics/Astrolabe.java:46–62 | Run | `m30_bane_matrix_astrolabe` | Bottled candidates remain eligible here. |
| Bottled Flame | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/relics/BottledFlame.java:52,60 | Run | `m30_bane_matrix_bottled_flame` | Curse is outside the Attack set. |
| Bottled Lightning | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/relics/BottledLightning.java:52,60 | Run | `m30_bane_matrix_bottled_lightning` | Curse is outside the Skill set. |
| Bottled Tornado | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/relics/BottledTornado.java:53,61 | Run | `m30_bane_matrix_bottled_tornado` | Curse is outside the Power set. |
| Back to Basics (purge) | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/events/city/BackToBasics.java:78–80 | Run | `m30_bane_matrix_ancient_writing_elegance` | Empty purge group skips the branch. |
| Old Beggar | verification/2026-11-02-m28e-a-v1/decompiled/com/megacrit/cardcrawl/events/city/Beggar.java:88 | Run | `m30_bane_matrix_beggar_softlock` | SOFTLOCK-001: pays 75 gold and completes with no card selection. |
| Drug Dealer / Augmenter | verification/2026-11-02-m28e-a-v1/decompiled/com/megacrit/cardcrawl/events/city/DrugDealer.java:54, 139–144 | Run | `m30_bane_matrix_drug_dealer` | Two-card request disables below two purgeable cards. |
| Cleric | verification/2026-11-03-m29-asc-a-v1/decompiled/com/megacrit/cardcrawl/events/exordium/Cleric.java:85–88 | Run | `m30_bane_matrix_cleric` | No charge with an empty candidate group. |
| Golden Wing | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/events/exordium/GoldenWing.java:124 | Run | `m30_bane_matrix_wing_statue_softlock` | SOFTLOCK-001: Pray still deals damage, then the empty grid completes. |
| Living Wall (Forget/Change) | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/events/exordium/LivingWall.java:106–119 | Run | `m30_bane_matrix_living_wall_forget`, `m30_bane_matrix_living_wall_change` | Empty eligible group skips the removal or transform. |
| Living Wall (Grow) | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/events/exordium/LivingWall.java:54–57, 117–119 | Run | `m30_bane_matrix_living_wall_grow` | Upgrade disabled when no card can upgrade. |
| Bonfire / Bonfire Spirits | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/events/shrines/Bonfire.java:111–114 | Run | `m30_bane_matrix_bonfire_spirits` | Removal branch requires a candidate. |
| Designer | verification/2026-11-03-m29-asc-a-v1/decompiled/com/megacrit/cardcrawl/events/shrines/Designer.java:176–180, 203–217 | Run | `m30_bane_matrix_designer_clean_remove_softlock`, `m30_bane_matrix_designer_clean_transform_softlock`, `m30_bane_matrix_designer_full_service_softlock`, `m30_bane_matrix_designer_punch`, `m30_bane_matrix_designer_a10_short_clean_transform` | SOFTLOCK-001: raw menu count can exceed the filtered selection. |
| Designer at A10 with one purgeable card | verification/2026-11-03-m29-asc-a-v1/decompiled/com/megacrit/cardcrawl/events/shrines/Designer.java:176–180, 203–217 | Run | `m30_bane_matrix_designer_a10_short_clean_transform` | SOFTLOCK-001: Bane counts in the menu; the two-card grid has only one card. |
| Gremlin Wheel / Wheel of Change | verification/2026-11-03-m29-asc-a-v1/decompiled/com/megacrit/cardcrawl/events/shrines/GremlinWheelGame.java:309–312 | Run | `m30_bane_matrix_wheel_of_change` | Removal outcome checks for a candidate. |
| Note for Yourself | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/events/shrines/NoteForYourself.java:71–73 | Run | `m30_bane_matrix_note_for_yourself` | Bane is not stored. |
| Purification Shrine | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/events/shrines/PurificationShrine.java:68 | Run | `m30_bane_matrix_purifier_softlock` | SOFTLOCK-001: empty picker completes without removing a card. |
| Campfire TokeEffect | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/vfx/campfire/CampfireTokeEffect.java:61–71 | Run | `m30_bane_matrix_peace_pipe_candidates`, `m30_bane_matrix_peace_pipe_cancel` | Picker uses unbottled purgeables; Cancel is covered separately. |
| Transmogrifier | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/events/shrines/Transmogrifier.java:72 | Run | `m30_bane_matrix_transmogrifier_softlock` | SOFTLOCK-001: empty transform picker completes. |
| Dolly’s Mirror | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/relics/DollysMirror.java:44–56 | Run | `m30_bane_matrix_dollys_mirror`; m30_bane_copy_dolly_appends_unremovable_ethereal | Raw master-deck selector can copy Bane. |
| Duplicator | verification/2026-11-02-m28f-v1/decompiled/Duplicator/com/megacrit/cardcrawl/events/shrines/Duplicator.java:90–100 | Run | `m30_bane_matrix_duplicator`; m30_bane_copy_duplicator_appends | Raw master-deck selector can copy Bane. |
| We Meet Again | verification/2026-11-02-m28e-c-v1/decompiled/com/megacrit/cardcrawl/events/shrines/WeMeetAgain.java:59–69, 102–107 | Run | `m30_bane_matrix_we_meet_again` | Curse type excludes Bane. |
| Falling | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/events/beyond/Falling.java:62–73, 117–132; verification/2026-11-03-m28f-v1/decompiled/CardHelper/com/megacrit/cardcrawl/helpers/CardHelper.java:92–107 | Run | `m30_bane_matrix_falling` | Attack/Skill/Power selector excludes Curse. |
| Vampires | verification/2026-11-02-m28e-c-v1/decompiled/com/megacrit/cardcrawl/events/city/Vampires.java:98–110 | Run | `m30_bane_matrix_vampires` | StarterStrike tags exclude Bane. |
| Pandora’s Box | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/relics/PandorasBox.java:62–80 | Run | `m30_bane_matrix_pandoras_box` | Bane has no starter tag. |
| Fountain of Curse Removal / Cleansing | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/events/shrines/FountainOfCurseRemoval.java:57–65 | Run | `m30_bane_matrix_fountain_of_cleansing` | Source checks Flame and Lightning bottles only; keep that literal predicate. |
| CardGroup.getUpgradableCards | verification/2026-11-02-m28e-c-v1/decompiled/com/megacrit/cardcrawl/cards/CardGroup.java:944–955 | Table | `m30_bane_matrix_get_upgradable` | Curse cards cannot upgrade. |
| Neow upgrade | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/neow/NeowReward.java:320 | Run | `m30_bane_matrix_neow_upgrade` | Bane is not offered. |
| Designer adjustment | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/events/shrines/Designer.java:174–176, 195, 240–253 | Run | `m30_bane_matrix_designer_adjust_chosen`, `m30_bane_matrix_designer_adjust_random` | Upgrade pool excludes Bane. |
| Campfire Smith | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/rooms/CampfireUI.java:115–117; CampfireSmithEffect.java:77 | Run | `m30_bane_matrix_smith` | Smith is Disabled when no card can upgrade. |
| Accursed Blacksmith / Forge | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/events/shrines/AccursedBlacksmith.java:51–56, 86 | Run | `m30_bane_matrix_accursed_blacksmith` | Forge is Disabled when no card can upgrade. |
| Upgrade Shrine | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/events/shrines/UpgradeShrine.java:42–49, 77 | Run | `m30_bane_matrix_upgrade_shrine` | Upgrade option is Disabled when no card can upgrade. |
| Tiny House | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/relics/TinyHouse.java:40–62 | Run | `m30_bane_matrix_tiny_house` | Bane stays unupgraded. |
| War Paint | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/relics/WarPaint.java:38–60 | Run | `m30_bane_matrix_war_paint` | Bane stays unupgraded. |
| Test 5 relic implementation | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/relics/Test5.java:38–64 | Table | none (no engine site; Out) | Out: normal pool registration is not established. |
| Whetstone | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/relics/Whetstone.java:38–60 | Run | `m30_bane_matrix_whetstone` | Bane stays unupgraded. |
| Back to Basics (upgrade) | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/events/city/BackToBasics.java:98–105 | Run | `m30_bane_matrix_ancient_writing_simplicity` | Starter tags exclude Bane. |
| Mind Bloom | verification/2026-11-02-m28e-c-v1/decompiled/com/megacrit/cardcrawl/events/beyond/MindBloom.java:80–86 | Run | `m30_bane_matrix_mind_bloom` | Bane stays unupgraded. |
| Shining Light | verification/2026-11-03-m29-asc-a-v1/decompiled/com/megacrit/cardcrawl/events/exordium/ShiningLight.java:61–66, 104–110 | Run | `m30_bane_matrix_shining_light` | Bane stays unupgraded; no-upgrade choice is Disabled. |
| Lesson Learned (combat) | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/actions/watcher/LessonLearnedAction.java:52–59 | Table | `m30_bane_matrix_lesson_learned` | Upgradeable-card predicate excludes Bane. |
| Increase Misc (combat) | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/actions/defect/IncreaseMiscAction.java:29–37 | Table | none (no engine site matches this row; skipped by the matrix) | Exact UUID scan; Bane is not randomly selected. |
| Ritual Dagger (combat) | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/actions/unique/RitualDaggerAction.java:46–57 | Table (static) | `m30_bane_matrix_ritual_dagger_static` | Exact UUID scan; Bane is not a normal target. Static only: the master-deck misc update is driven by the M26 supplied play (`m26_supplied.rs`, Fatal +3). |
| Necronomicon removal | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/relics/Necronomicon.java:61–68 | Run | `m30_bane_matrix_necronomicon_unequip` | Only Necronomicurse is removed. |
| Gremlin Match Game | verification/2026-11-03-m28f-v1/decompiled/com/megacrit/cardcrawl/events/shrines/GremlinMatchGame.java:309–310; verification/2026-11-02-m28e-c-v1/decompiled/com/megacrit/cardcrawl/dungeons/AbstractDungeon.java:1105–1108 | Run | `m30_bane_matrix_match_and_keep` | Generated-card pool; no Bane from the curse pool. |

### §F: protected curse IDs and random curse pools

| Source row | Locator | Grade | Test | Note |
|---|---|---|---|---|
| Special curse IDs are excluded from random curse pools | verification/2026-11-02-m28e-c-v1/decompiled/com/megacrit/cardcrawl/cards/CardGroup.java:961–967; verification/2026-11-02-m28e-c-v1/decompiled/com/megacrit/cardcrawl/characters/AbstractPlayer.java:721–731; verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/dungeons/AbstractDungeon.java:1192–1198; verification/2026-11-02-m28e-a-v1/decompiled/com/megacrit/cardcrawl/helpers/CardLibrary.java:1398–1415 | Table | `m30_bane_matrix_random_curse_pool` | The source also filters Pride; the matrix pool assertion pins Bane, Necronomicurse, and Curse of the Bell. |
| Raw removal refuses the three unremovable curses | m30-engine-audit-v1.md §3; crates/spire-core/src/run/engine.rs:172–176 | Run | m30_raw_remove_refuses_unremovable_curses; `m30_bane_matrix_raw_remove` | This is the G3 API boundary, not a random-pool assertion. |

### §G: curse counts and obtain hooks

| Source row | Locator | Grade | Test | Note |
|---|---|---|---|---|
| Du-Vu Doll counts CURSE cards in combat | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/relics/DuVuDoll.java:53–71 | Combat | m30_bane_combat_du_vu_doll_counts_it | The test checks Bane’s combat count; direct A10 insertion does not call the master-deck change hook. |
| Omamori with ShowCardAndObtainEffect | verification/2026-11-03-m28f-v1/decompiled/ShowCardAndObtainEffect/com/megacrit/cardcrawl/vfx/cardManip/ShowCardAndObtainEffect.java:41–54, 104–116 | Run | m30_bane_copy_omamori_absorbs | Callback timing after a negated obtain depends on frame delta; not determinable statically. |
| Omamori with FastCardObtainEffect | verification/2026-11-03-m28f-v1/decompiled/FastCardObtainEffect/com/megacrit/cardcrawl/vfx/FastCardObtainEffect.java:26–30, 40–48 | Run | m30_bane_copy_omamori_absorbs | The test pins Omamori absorption through the Dolly copy path; this FastCard callback path is source-derived. |
| Darkstone Periapt | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/relics/DarkstonePeriapt.java:32–40 | Run | m30_bane_copy_darkstone_and_fish_react | +6 max HP for an obtained Bane copy. |
| Ceramic Fish | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/relics/CeramicFish.java:28–39 | Run | m30_bane_copy_darkstone_and_fish_react | +9 gold for an obtained Bane copy. |
| Blue Candle | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/relics/BlueCandle.java:40–49 | Combat | m30_bane_combat_blue_candle_plays_it | −1 HP and exhaust. |
| Medical Kit | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/relics/MedicalKit.java:30–37; verification/2026-11-02-m28e-c-v1/decompiled/com/megacrit/cardcrawl/cards/AbstractCard.java:895–909 | Combat | m30_bane_combat_medical_kit_does_not_enable | Status-only play exception. |
| Necronomicon | verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/relics/Necronomicon.java:49–68; verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/cards/curses/Necronomicurse.java:39–55 | Run | `m30_bane_matrix_necronomicon_unequip` | Its special removal behavior targets Necronomicurse, not Bane. |

### §H: other A6–A10 gates

None found beyond the A6 current-HP assignment, A10 Bane insertion, the §A monster rows, and the A10 achievement unlocks.

## Reconciliation

This section records disclosed changes and amendments against accepted M28/M29 assertions. It does not report runner outcomes. Every amended accepted-test original is retained in $E/m30/amended-original/.

| ID | Change | Source or ruling | Pinning tests | Amended accepted assertions | Status |
|---|---|---|---|---|---|
| E1 | Standard now admits A0–A10 in a default build; the first rejected level is A11; error text is Unsupported("Standard supports A0–A10"). | M30 plan §2 item 1; plan-review v2 n1. | m30_standard_creation_a0_to_a10_all_characters; m30_standard_rejects_a11_and_above; renamed M29 gate tests. | m29_standard_default_build_admits_a1_to_a5_rejects_a6 → m29_standard_default_build_admits_a1_to_a5 (formerly); m29_standard_rejects_a6_and_above → m29_standard_rejects_above_cap (formerly). Retarget m28_standard_creation_all_characters and m28_standard_a0_is_public_in_a_default_build from A6 to A11; update the error string. | Candidate; originals kept in $E/m30/amended-original/. |
| E2 | A6 start HP uses libGDX rounding of a binary32 90% product after the A14 max-HP loss; literals are unchanged. | Source-check §B; verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/dungeons/AbstractDungeon.java:2682; MathUtils.round at verification/2026-11-02-m28e-a-v1/decompiled/com/badlogic/gdx/math/MathUtils.java:234–235. | m30_a6_start_hp_all_characters; all_84_starts. | No HP value changed. | Candidate. |
| E3 | A10+ Bane moves to deck index 0 before starter cards; instance numbering stays fixed. This changes A10 opening shuffles. | Source-check §D; verification/2026-11-03-m29-asc-a-v1/decompiled/com/megacrit/cardcrawl/dungeons/AbstractDungeon.java:2678–2687; design §6.3. | m30_bane_first_before_neow; m30_bane_insertion_dispatches_no_item_event; m30_bane_position_feeds_opening_shuffle; m30_a10_combat_settlement_keeps_bane. | all_84_starts is amended to assert Bane at index 0 and to check starters after it; original bytes retained. | Candidate. |
| E7 | Consolidate removal rules in mutations::removable and rooms::selection::purgeable. Replace the structural relic and Empty Cage local predicates, and route Fountain pool eligibility through the shared owner. | Design §6.4; source-check §E Fountain of Curse Removal: verification/2026-11-03-m30-asc-v1/decompiled/com/megacrit/cardcrawl/events/shrines/FountainOfCurseRemoval.java:57–65. | M30 lifecycle matrix; m28_fountain_of_cleansing_removes_qualifying_curses_backwards. | None. The source tests Flame and Lightning bottle flags only (FountainOfCurseRemoval.java:61); retain that body check. The accepted Fountain test also pins Tornado. | Candidate; behavior-preserving consolidation. |
| G1 | Toke filters its picker, becomes Disabled when no unbottled purgeable card exists, and can be cancelled back to the campfire. A campfire with no Offered option completes at entry. A0 deltas: (a) Necronomicurse, Curse of the Bell and bottled cards leave the Toke candidates; (b) fewer selection ids shift later choice ids; (c) the Toke selection gains Cancel; (d) Toke is Disabled with no unbottled purgeable card. | Source-check §E: PeacePipe.java:53; CampfireTokeEffect.java:61–71; CancelButton.java:98–108. Plan review v1 m3 and m5; PM ruling on review v1 I1. | m30_toke_excludes_unpurgeable_and_bottled; m30_toke_disabled_without_purgeable; m30_toke_cancel_returns_to_campfire; m30_campfire_only_disabled_toke_completes_at_entry. | No accepted assertion amended. | Candidate. |
| G2 | Shop purge is Disabled when no unbottled purgeable card exists; buying a card enables the same row. | D1/EVENT-001; source-check §E ShopScreen.java:256, 328–337, 1007; EVENT-005. | m30_shop_purge_disabled_without_purgeable; m30_shop_purge_reenabled_after_buy; m30_shop_purge_no_fault. | No accepted assertion amended. | Candidate; EVENT-005 is next to EVENT-001 in docs/intentional-deviations.md. |
| G3 | Run::remove_card keeps its guards and delegates to mutations::remove_card, including bottle cleanup, Parasite max-HP loss, and CardRemoved dispatch. | Design §6.7; m30-engine-audit-v1 §3; source-check §F special curse exclusions. | m30_raw_remove_refuses_unremovable_curses; m30_raw_remove_uses_owner_side_effects. | No caller assertion changed. | Candidate. |
| G4 | Combat entry of every Complete-admission Run (the Standard Run; the M26/M27 supplied diagnostics pre-sort their decks by the same key, so they are unchanged) preserves master-deck order before the seeded shuffle; the sort is kept for the finite M21–M25 Watcher profiles. | PM ruling W1; source-check §D; task0-bind §(c). | m30_g4_standard_combat_shuffles_master_order_at_a0; m30_bane_position_feeds_opening_shuffle. | A0 and A1–A5 Standard Watcher replay/opening-hand results change from the first combat. Consumers replaying Standard Watcher Runs must account for the new order. | Candidate; consumer disclosure required. |
| G5 | Red Skull activates only on an HP loss; a heal that leaves the player bloodied no longer re-arms the latch after the settlement reset. | RedSkull.java (onBloodied needs the COMBAT phase; onVictory clears isActive); AbstractCreature.java:441 (heal calls only onNotBloodied). Found by runner B c1, A8/A9 supplied Ironclad/5; reproduced on base 417ad9ce with only the cap raised. | m30_red_skull_latch_cleared_after_settlement_heal | No accepted assertion amended. | Candidate; global fix. |
| D-EV | `data/events/events.json` is regenerated (`python3 -B data/events/build_dataset.py`): only the recorded SHA-256 of `docs/intentional-deviations.md` changes, because M30 adds EVENT-005 and updates the Note For Yourself range. | data/events/README.md checks. | `tests/python/test_events_reference.py`; `build_dataset.py --check` | None. | Candidate. |
| T1 | Add the traversal report key m30, separate from the existing A0 projection. It records start HP/max HP by character; Bane count and index at creation and terminal; Toke Offered, Disabled, and cancelled counts; the count of affordable shop-purge rows Disabled for want of a purgeable card (G2; c1 counted price-disabled rows too, and was fixed in c2); and Bane-copy count. | M30 plan §2 item 10; M30 progress-v1. | Runner B traversal receipts: $E/m30/c2-checks/r2-traversal-summary.md. | No accepted assertion amended. | Candidate. |
| N1 | Correct the retain-test brief: Well-Laid Plans offers Bane, as the source offers every hand card; only non-Ethereal choices are marked retained. | Source-check §C; RetainCardsAction.java:27,34; EquilibriumPower.java:41; plan-review v2 m8. | m30_bane_combat_retain_effects_do_not_save_bane. | No accepted assertion amended; the test now picks Bane or Defend and verifies Bane exhausts. | Correction to the test brief, not a behavior change. |

### Source and evidence limitations

M30 merged as PR92 (`5e2a10b`). The runner receipts above are on snapshot c2. M30 does not independently claim original-runtime validation.
