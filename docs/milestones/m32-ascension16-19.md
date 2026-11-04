# M32 Ascension 16–19

Status: source candidate, not accepted; no spire-env, Gym or Python profile (exposure M34–M36).

M32 extends the default-build Standard Run boundary to A19, implements the source's A16 shop markup and the 26 monster
behavior rows whose thresholds fall at A17–A19, and removes the engine's playable encounter ceilings through combat
A20. This is a source profile and reconciliation, not original-runtime validation or a delivery record.

## Profile

### Standard boundary and shop

Standard admits A0–A19 for all four characters in a default build. A20 and above return
`Unsupported("Standard supports A0–A19")`. Direct combat setup, entry, splits, summons and event fights admit all 63
encounters at A0–A20; the Standard Run cap remains A19.

At A16+, each initially stocked Standard shop card, relic and potion row is multiplied by 1.1 using binary32 arithmetic
and libGDX rounding. The pass follows the sale halving and precedes The Courier and Membership Card discounts. It does
not affect purge or Courier restocks. Buying Membership Card in the shop halves the already marked-up stock. The
in-shop Courier reprice path is unreachable because The Courier is not stocked.

### Monster behavior rows

Each threshold applies at that level and above. The source's numeric ladders remain the M29 values.

| Source row | Threshold | Behavior at threshold |
|---|---:|---|
| Acid Slime S | A17 | Only its opening selection changes: it opens with Lick and Weak 1. Later turns keep direct Tackle/Lick alternation. |
| Acid Slime M | A17 | Selection bands become `<40 / <80 / rest`. The second band guards against two Normal Tackles with a 0.5 redirect; the last band guards against repeating Lick after one Lick, with the existing 0.4 redirect. |
| Acid Slime L | A17 | Bands become `<40 / <70 / rest`. The first band redirects after two Wound Tackles with a 0.6 Normal Tackle/Lick choice; the second guards against two Normal Tackles with a 0.6 redirect; the last guards against repeating Lick after one Lick. |
| Spike Slime M | A17 | In the high-roll branch, one Lick is enough to bar another Lick; the next move is Flame Tackle. |
| Spike Slime L | A17 | Uses the same one-Lick repeat guard as Spike Slime M; its Frail 3 payload was already present. |
| Red Louse | A17 | In the low-roll branch, one Grow bars another Grow and selects Bite. |
| Green Louse | A17 | In the low-roll branch, one Spit Web bars another Spit Web and selects Bite. |
| Gremlin Wizard | A17 | After Ultimate Blast, its next move is Blast again rather than Charge. |
| Gremlin Fat | A17 | Smash adds Frail 1 after Weak 1. |
| Slaver Red | A17 | The final fallback bars Scrape after one Scrape and selects Stab. |
| Romeo (Bandit Leader) | A17 | Cross Slash follows Cross Slash unless both of the last two moves were Cross Slash; then Agonizing Slash follows. The other cycle is unchanged. |
| Chosen | A17 | Its opening move is Hex instead of the two-hit Poke; the later cycle is unchanged. |
| Mystic (Healer) | A17 | Heal when the live non-dying, non-escaping group is missing more than 20 HP. In the attack branch, Attack is barred when it was the last move. |
| Shelled Parasite | A17 | Its opening move is always Fell; the first-move coin is removed. Later selection is unchanged. |
| Snake Plant | A17 | Spores is barred when it was the move before last as well as when it was the last move in the high-roll branch. |
| Snecko | A17 | Tail Whip applies Weak 2 after damage and before Vulnerable 2. |
| Book of Stabbing | A18 | Selecting Big Stab increments its stab counter in either roll branch, so later Multi-Stabs hit more often. This counter is payload-only in the current public view. |
| Taskmaster | A18 | Scouring Whip adds self Strength 1 after creating its Wounds. |
| Champ | A19 | Forge's cutoff becomes roll ≤30, from ≤15; its place after Anger, Execute and Taunt is unchanged. |
| Bronze Automaton | A19 | After Hyper Beam it selects Boost instead of Stunned; this early return does not advance its turn counter. |
| Darkling | A17 | Harden keeps the `darkling_harden` move name and adds Strength 2 after Block 12. The source category changes from DEFEND to DEFEND_BUFF. |
| Spire Growth | A17 | Before the ordinary roll branches, select Constrict if the player lacks Constricted and the last move was not Constrict. The existing base roll is still consumed; Constricted is 12. |
| Deca | A19 | Square of Protection keeps the `deca_square` move name and gives each living monster Block 16, then Plated Armor 3, in monster order. Its source category changes from DEFEND to DEFEND_BUFF. |
| Time Eater | A19 | Ripple adds Frail 1 after Vulnerable 1 and Weak 1; Head Slam adds two Slimed to discard after Draw Reduction; Haste gains Block 32 after its heal. |
| Spire Shield | A18 | Smash grants fixed self Block 99 instead of Block equal to its damage output. |
| Spire Spear | A18 | Burn Strike puts its two unupgraded Burns on top of the draw pile, not in discard; this adds no card-RNG draw. |

### Global fix G1: Louse opening

At every ascension and in every profile, Red and Green Louse opening selection uses the ordinary roll: ticket 0 of the
existing four-way draw selects Grow or Spit Web, and the other three tickets select Bite. The opening-history check no
longer assumes Bite. This is the one explicit A0–A15 trajectory change in M32. The shared fix follows the standing
no-freeze authority (receipt `evidence/standing-milestone-authority-20261030/primary-user-no-freeze-and-auto-build-v1.json`,
10e8d8e5) and the fix-globally rule; PM ruling `pm-ruling-frozen-trajectories-v1.md` (925f9f17) applies it without a
profile guard. The M32 profile records the changed outcomes for Standard, M5–M7, M9/M10, M13, M17–M19, combat-v1 and
Gym66. The affected first step is a generated Louse opening; runs and replays that enter a Louse combat may diverge
there. The M10 Python expected prefix was updated as a test-only change; the Python test was not run here.

**Compatibility note for spire-learn:** G1 changes A0 trajectories through Louse combats in every profile. Consumers
that compare trajectories should attribute changes from the first Louse opening using `m32.first_louse_step` and the
M32 delivery evidence; profiles without a generated Louse opening have no such divergence. No spire-env or Python
source/facade behavior changed.

### Limits and outs

- A20 Standard admission and the second Act 3 boss remain M33. The M32 combat engine admits A20 encounters, but M32
  does not claim a Standard A20 Run.
- There is no M32 spire-env, Gym or Python profile. Public exposure is M34–M36. G1 test/example expectations, seeds and
  receipts may be re-pinned under the PM ruling; no spire-env or Python source/facade edit is in scope.
- M32 changes no numeric enemy ladder and does not add RNG draws to imitate original `rollMove` call counts (RNG-001).
  Original-runtime validation and strategy or win-rate targets are out of scope.
- The public enemy view has no hit-count or intent-category field. Book of Stabbing's counter and several A17–A19
  payloads therefore rely on combat tests rather than the public-intent traversal gate; the M34 notes below preserve
  this limit.

### Traversal reach and evidence index

The c2 A16–A19 default and diagnostic reach/uniform traversal samples all ended in Act 1. At each level, the default
lane ran 100 reach and 100 uniform Runs (200 traversal Runs); diagnostics ran the same 200 traversal Runs plus 100
separate supplied-resource diagnostics (300 Runs total). Every reach/uniform mode had 100 fault-free Death outcomes
in Act 1 (maximum floor 16; deaths span floors 1–16). Across all 200 default and 300 diagnostic Runs per level, faults, interim rejections and
interim-blocked Runs were zero; Act 2 boss, Act 3 boss, Act 4 and Heart counts were zero in each reach/uniform row.
The supplied-resource mode is diagnostic evidence, not traversal reach. Both c2 gates exited 0 and passed A16–A19;
the Slaver Red forbidden counter is 0 at A17–A19 in reach and uniform, and 0 in supplied diagnostics. See
`$E/m32/c2-checks/r2-traversal-summary.md` and `$E/m32/c2-checks/results-b.md`.

The supplied-resource diagnostics (not traversal reach) went further: maximum act/floor 3/42 at A16, 2/33 at A17,
3/44 at A18 and 2/33 at A19, with zero forbidden public-intent sequences at A ≥ t. They show the allowed-only A17+
sequences Gremlin Wizard Blast → Blast (8, 15 and 18 at A17, A18 and A19) and Romeo Cross Slash → Cross Slash (2 at
A17 and at A18); their `automaton_beam_stunned` count of 1 at A17–A18 is below Automaton's A19 threshold (a sanity
count). See `$E/m32/c2-checks/r2-traversal-summary.md`.

Act 2–4 behavior coverage comes from Combat-grade rows, the every-encounter sweep and constructed public Runs. Those
constructed Runs disclose their supplied HP and keys in the named tests; later-act coverage is not inferred from the
Act 1 traversal results.

The traversal records one `m32.first_louse_step` marker per Run outside the A0 control projection; the join script
`$E/m32/scripts/a0_attribution_join.py` requires each first differing step to be at or after that marker. The
traversal detector samples each monster once per `FullCombatView.turn`, keys histories by instance, starts a new opening
after entry/spawn/split, and breaks a chain on hidden, half-dead, escaped or replaced actors.

Paths beginning `$E` are under `E=/home/klottick/.local/share/spire-gym/evidence/m32-execution-20261104` in the private
evidence store.

#### A0–A15 attribution and compatibility checks

- A0 diagnostics: 300 Runs, 177 G1-attributed, 123 equal, 0 findings; join exit 0. C2 candidate digests match c1.
  Evidence: `$E/m32/c2-checks/r2-traversal-summary.md` and `$E/m32/c2-checks/results-b.md`.
- A0 default: 200 Runs, 110 G1-attributed, 90 equal, 0 findings; join exit 0. Evidence:
  `$E/m32/c1-checks/r2-traversal-summary.md`.
- A1–A15 default joins: all 200 Runs per level; every join exited 0 with 0 findings. G1-attributed / equal counts:

| A | G1 / equal |
|---:|---:|
| 1 | 109 / 91 |
| 2 | 103 / 97 |
| 3 | 109 / 91 |
| 4 | 104 / 96 |
| 5 | 104 / 96 |
| 6 | 104 / 96 |
| 7 | 103 / 97 |
| 8 | 109 / 91 |
| 9 | 104 / 96 |
| 10 | 104 / 96 |
| 11 | 103 / 97 |
| 12 | 114 / 86 |
| 13 | 103 / 97 |
| 14 | 109 / 91 |
| 15 | 106 / 94 |

  Evidence: `$E/m32/c1-checks/r2-traversal-summary.md`.
- Frozen replay: 323 inputs, 232 equal, 91 classified as G1 at/after the base first-Louse step, 0 findings and 0
  unsure. Evidence: `$E/m32/c1-checks/r2-replay-classification.md` and `$E/m32/c1-checks/results-b.md`.
- Native fuzz: candidate and base each ran 400 episodes with 0 failure groups; candidate-only and base-only groups
  were both 0. Per-seed attribution is unavailable in the aggregate `native_fuzz` summaries. Evidence:
  `$E/m32/c1-checks/r2-fuzz-compare.md` and `$E/m32/c1-checks/results-b.md`.
- M9 and M10 examples: all six cases per profile ended in Death on both trees. M9 candidate/base action counts were
  699/697 for each seed-7 history; for seed 11 they were 708/711 (Awakened One), 811/814 (Time Eater), and 828/831
  (Donu/Deca). M10 counts were 175/177 for each seed-7 history and 108/110 for each seed-11 history. Evidence:
  `$E/m32/c1-checks/r2-examples-compare.md` and `$E/m32/c1-checks/results-b.md`.
- Final c3 workspace (after the final-review I1 Acid tests): 4,770 passed, 0 failed, 11 ignored; the candidate name
  comparison recorded 47 `m32_` test additions plus six expected replacement names. Evidence:
  `$E/m32/c3-checks/results-a.md` (c2: 4,766 / 43 in `$E/m32/c2-checks/results-a.md`).

#### A16–A19 final traversal gate

For each A16–A19 level, the c2 summary reports 100 reach and 100 uniform Runs in default mode, and the same 100 + 100
traversal Runs in diagnostics mode. Each reach/uniform row has 100/100 fault-free Death outcomes in Act 1 (maximum floor 16);
faults, interim rejections and interim-blocked Runs are all zero, as are Act 2 boss, Act 3 boss, Act 4 and Heart counts.
Diagnostics adds 100 supplied-resource Runs per level; those are a separate mode and are not counted as traversal reach.
Both c2 `m32_gate.py` invocations exited 0 with `M32 traversal gate: PASS for A16, A17, A18, A19`. The forbidden
`slaver_red_scrape_scrape` count is zero at A17–A19 in reach and uniform, and zero in supplied diagnostics. Evidence:
`$E/m32/c2-checks/r2-traversal-summary.md` and `$E/m32/c2-checks/results-b.md`.

- Source plan and design: `$E/planning-v2/2026-11-04-m32-ascension16-19-plan.md` and `...-design.md`.
- Static source lanes and engine audit: `$E/planning-v2/research/m32-source-{nonmonster,exordium,city,beyond-ending}-v1.md` and `m32-engine-audit-v1.md`.
- Independent plan review and PM rulings: `$E/plan-review-v1/review-v2.md`, `$E/pm-ruling-intent-names-v1.md`, and
  `$E/pm-ruling-frozen-trajectories-v1.md`.
- Execution binding, ownership and G1 records: `$E/m32/task0e-checkpoint-v1.md`, `$E/m32/task0-asst/`, and
  `$E/m32/g1-repins-v1.md`; the dispatcher ruling is `$E/m32/briefs/rows-ruling-v1.md`.
- Source rows and runtime evidence: `$E/m32/lane-act1/report.md`, `lane-act2/report.md`, `lane-act34/report.md`,
  `item9-sweep-route-v1.md` and `item10-traversal-v1.md`.
- C1 runner records: `$E/m32/c1-checks/results-a.md`, `$E/m32/c1-checks/results-b.md`,
  `$E/m32/c1-checks/r2-traversal-summary.md`, `$E/m32/c1-checks/r2-replay-classification.md`,
  `$E/m32/c1-checks/r2-fuzz-compare.md` and `$E/m32/c1-checks/r2-examples-compare.md`.
- Final c2 runner records: `$E/m32/c2-checks/results-a.md`, `$E/m32/c2-checks/results-b.md` and
  `$E/m32/c2-checks/r2-traversal-summary.md`.
  C2 supersedes C1 for the final workspace count, the A0 diagnostics recheck and A16–A19 traversal/gate results;
  the c1 summary retains the A0 default and A1–A15 joins, and c1 supplies the replay, fuzz and example evidence.

## Coverage ledger

The lane locators below name the row sections in the static source records. Grades are Run, Combat or Table. Numeric
rows are unchanged from M29; the listed M29 tests remain their native pins. This ledger maps the candidate source rows
to the checked-in native tests and named traversal/evidence gates; it is not original-runtime validation.

### Lane A — non-monster rules, shop and census

| Source row | Locator | Grade | Test(s) | Note |
|---|---|---|---|---|
| Census: 83 comparison sites at A16–A19; 82 monster sites and one non-monster ShopScreen site. Counts are 1/47/16/19 at A16/A17/A18/A19. | `m32-source-nonmonster-v1.md` §1; monster site census §§4 in lanes B–D | Table | UNMAPPED for the static site count; related engine checks: `m32_every_setup_admitted_a0_to_a20`, `m29_all_species_landed` | Static source inventory, not executable behavior. The admission test covers 63 encounter setups but does not assert the source-site census. No other non-monster threshold appears in the band. |
| A16 Standard shop stock gets ×1.1 after sale rounding and before Courier/Membership. | `m32-source-nonmonster-v1.md` §2; design §§2.2, 6.6 | Run | `m32_a16_shop_prices_vs_a15`; `m32_a16_shop_discount_literals`; `m32_a16_courier_restock_no_markup`; `m32_a16_membership_bought_in_shop`; `m32_a15_shop_unchanged` | Tests all stock kinds, discounts, sale, unchanged purge, restock exclusion and A15 control. The in-shop Courier reprice path is unreachable; oracle (d) covers Membership. |
| Standard cap changes to A19; A20 is rejected. | Engine audit §1; design §§3 E1, 6.1 | Run | `m32_standard_creation_a0_to_a19_all_characters`; `m32_standard_rejects_a20_and_above` | Public Standard constructor, four characters and exact unsupported boundary. |
| Playable encounter ceilings are removed; setup and event admission continue through combat A20. | Engine audit §2; design §§3 E2, 6.2 | Combat / Run | `m32_every_setup_admitted_a0_to_a20`; `m29_combat_admission_accepts_through_a20`; `m29_every_encounter_every_ascension`; `m29_d_formations_admit_through_a20`; `m29_event_fight_admits_a17`; `m28_event_enemies_admit_a17_combat`; `m28_colosseum_and_masked_bandits_fights_enter_at_a17`; `m28_review_event_interim_leaves_the_whole_run`; `m28_event_combat_unsupported_ascension_is_rejected_up_front` | Includes 63 setups × A0–A20, split/spawn/event paths and the retained A21 unsupported witness. |
| Every admitted encounter survives a randomized combat sweep. | Design §7; `$E/m32/item9-sweep-route-v1.md` | Combat | `m32_every_encounter_a0_to_a20_sweep` | 63 setups × A0–A20 × three seeds, up to 120 legal actions, runtime validation and public view each step. |
| HP- or state-gated spawn, split, revive and rebirth paths work at A17–A20. | Design §7; `$E/m32/item9-sweep-route-v1.md` | Combat | `m32_threshold_paths_a17_to_a20` | Drives Slime Boss/large-slime splits, Gremlin Leader Rally, Torch Heads, Automaton orbs, daggers, Time Eater Haste, Champ Anger, Darkling half-dead/revive and Awakened One rebirth. |
| Standard Run reaches an A17 behavior, event fights, bosses and Act 4 through public Run ownership. | Design §7; `$E/m32/item9-sweep-route-v1.md` | Run | `m32_a17_first_act1_combat_public_behaviour`; `m32_event_fights_a17_to_a19`; `m32_bosses_and_act4_a17_to_a19` | Deterministic constructed Runs cover event fights, Act 2/3 bosses, Shield and Spear, and Heart; supplied HP/keys are disclosed in the tests. These, the Combat-grade rows and every-encounter sweep support later-act behavior; traversal itself ended in Act 1. |
| Traversal detector and forbidden-intent gate. | Design §6.9; `$E/m32/item10-traversal-v1.md`; `$E/m32/c2-checks/r2-traversal-summary.md` | Run | `m32_detector_synthetic_literals_cover_every_forbidden_rule`; script m32_gate.py | Six synthetic detector checks passed in the item-10 record. Both c2 gates pass A16–A19; Slaver Red's forbidden count is zero at A17–A19. Book of Stabbing and payload-only rows remain excluded from the public-intent gate. |

### Lane B — Act 1

| Source row | Locator | Grade | Test(s) | Note |
|---|---|---|---|---|
| Global Louse opening roll and non-Bite opening history are accepted at every A. | Design §6.8; `m32-source-exordium-v1.md` L89–115 | Combat | `m32_g1_louse_opening_roll`; `m32_g1_louse_nonattack_opening_history_accepted` | Red and Green species, A0 and A17; the test seed sweep observes both opening outcomes. |
| Acid Slime S opening-only Lick; later turns alternate directly. | `m32-source-exordium-v1.md` L22–34 | Combat | `m32_acid_slime_s_a17_opening_lick` | A16 coin opening and A17 forced opening; no re-roll on later turns. |
| Acid Slime M A17 bands and repeat guards. | `m32-source-exordium-v1.md` L35–47 | Combat | `m32_acid_slime_m_a17_bands_and_guards`; `m32_acid_slime_a17_band2_two_tackle_guard`; `m32_acid_slime_a17_band2_redirect_probabilities`; `m32_acid_slime_a17_band3_last_lick_guard`; `m32_acid_slime_l_a17_band1_redirect` | Bands (A16 middle vs A17 first band) and the setup/live routes; each A17 sub-rule separately: band-2 two-Tackle guard, band-2 coin redirect (0.5, against 0.4 and 0.6), band-3 last-Lick guard, band-1 coin kept. Each sub-rule's removal turns its test red (final review I1; `$E/m32/review-fix-v1/`). |
| Acid Slime L A17 bands and repeat guards. | `m32-source-exordium-v1.md` L48–60 | Combat | `m32_acid_slime_l_a17_bands_and_guards`; `m32_acid_slime_a17_band2_two_tackle_guard`; `m32_acid_slime_a17_band2_redirect_probabilities`; `m32_acid_slime_a17_band3_last_lick_guard`; `m32_acid_slime_l_a17_band1_redirect` | Bands and the setup/live routes; each A17 sub-rule separately: band-1 0.6 redirect, band-2 two-Tackle guard with 0.6 redirect, band-3 last-Lick guard. Each sub-rule's removal turns its test red (final review I1; `$E/m32/review-fix-v1/`). |
| Spike Slime M/L bar a repeated Lick after one Lick. | `m32-source-exordium-v1.md` L61–88 | Combat | `m32_spike_slime_m_a17_single_lick_guard`; `m32_spike_slime_l_a17_single_lick_guard` | Spike Slime S has no A16–A19 selection delta. |
| Red and Green Louse bar repeating Grow/Spit Web after one non-attack move. | `m32-source-exordium-v1.md` L89–116 | Combat | `m32_louse_red_a17_single_grow_guard`; `m32_louse_green_a17_single_spit_web_guard` | A17 guards are separate from G1's global ordinary opening roll. |
| Gremlin Wizard repeats Blast after Ultimate Blast. | `m32-source-exordium-v1.md` L117–129 | Combat | `m32_gremlin_wizard_a17_blast_repeats` | Covers Gremlin Gang at A17 and Gremlin Leader minion at A18. |
| Gremlin Fat Smash adds Frail 1 after Weak 1. | `m32-source-exordium-v1.md` L130–142 | Combat | `m32_gremlin_fat_a17_smash_frail` | Live payload order. |
| Slaver Red bars a second Scrape in the used- and unused-Entangle fallback branches. | `m32-source-exordium-v1.md` L143–155; `$E/m32/c2-checks/results-b.md` | Combat | `m32_slaver_red_a17_single_scrape_guard`; `m32_slaver_red_a17_scrape_guard_before_entangle` | Threshold rule is A17. The added regression test covers the unused-Entangle fallback; c2 gate counts are zero at A17–A19. |
| Split children select A17 slime rules. | `m32-source-exordium-v1.md` §§5–6; design §7 | Combat | `m32_split_children_a17_ai` | Slime Boss → Large → Medium child selection is checked. |
| Numeric values for Act 1 normals, slimes, Louses and Gremlins remain at M29 values. | `m32-source-exordium-v1.md` §Numeric cross-check | Combat | `m29_acid_slime_s_ascension_values`; `m29_acid_slime_m_ascension_values`; `m29_acid_slime_l_ascension_values`; `m29_spike_slime_s_ascension_values`; `m29_spike_slime_m_ascension_values`; `m29_spike_slime_l_ascension_values`; `m29_louse_red_ascension_values`; `m29_louse_green_ascension_values`; `m29_cultist_ascension_values`; `m29_jaw_worm_ascension_values`; `m29_fungi_beast_ascension_values`; `m29_looter_ascension_values`; `m29_gremlin_mad_ascension_values`; `m29_gremlin_sneaky_ascension_values`; `m29_gremlin_fat_ascension_values`; `m29_gremlin_shield_ascension_values`; `m29_gremlin_wizard_ascension_values` | Numeric cross-check at A16/A17/A18/A19 matched source; M32 changes no numeric ladder. |
| Numeric values for Act 1 elites, boss and Slavers remain at M29 values. | `m32-source-exordium-v1.md` §Numeric cross-check | Combat | `m29_gremlin_nob_ascension_values`; `m29_lagavulin_ascension_values`; `m29_sentry_ascension_values`; `m29_hexaghost_ascension_values`; `m29_slime_boss_ascension_values`; `m29_guardian_ascension_values`; `m29_slaver_blue_ascension_values`; `m29_slaver_red_ascension_values` | Includes the already-landed Gremlin Nob A18 cycle and Slaver Blue A17 Rake guard source recheck. |

### Lane C — Act 2

| Source row | Locator | Grade | Test(s) | Note |
|---|---|---|---|---|
| Romeo's A17 Cross Slash repeat and two-Cross return. | `m32-source-city-v1.md` L22–35 | Combat | `m32_romeo_a17_double_cross_slash` | Tests the corrected Cross/Cross parity path and stream state. |
| Chosen opens with Hex at A17. | `m32-source-city-v1.md` L36–49 | Combat | `m32_chosen_a17_opening_hex` | Poke remains at A16. |
| Mystic heal cutoff and repeated-Attack guard change at A17. | `m32-source-city-v1.md` L50–63 | Combat | `m32_mystic_a17_heal_cutoff_and_attack_guard` | Missing HP cutoff is >20; the live formation includes only eligible actors. |
| Shelled Parasite opens with Fell and removes its first-move coin. | `m32-source-city-v1.md` L64–77 | Combat | `m32_shelled_parasite_a17_opening_fell` | Fixed selection adds no engine draw. |
| Snake Plant bars Spores when it was the move before last in the high-roll branch. | `m32-source-city-v1.md` L78–91 | Combat | `m32_snake_plant_a17_spores_guard` | History-sensitive public move sequence. |
| Snecko Tail Whip applies Weak 2 between damage and Vulnerable 2. | `m32-source-city-v1.md` L92–105 | Combat | `m32_snecko_a17_tail_whip_weak` | Payload order and Weak amount. |
| Book of Stabbing Big Stab selection increments the A18 counter. | `m32-source-city-v1.md` L106–119 | Combat | `m32_book_of_stabbing_a18_big_stab_counts` | Payload-only: the test checks memory and resolved damage-effect count, not a public hit count (review I3). |
| Taskmaster Whip adds Strength 1 after Wounds at A18. | `m32-source-city-v1.md` L120–133 | Combat | `m32_taskmaster_a18_whip_strength` | Test is physically in crates/spire-core/src/combat/tests/m32_act1.rs; the source row is City/Act 2. |
| Champ Forge cutoff moves to roll ≤30 at A19. | `m32-source-city-v1.md` L134–147 | Combat | `m32_champ_a19_forge_cutoff` | Position in the selector tree is retained. |
| Bronze Automaton selects Boost rather than Stunned after Hyper Beam. | `m32-source-city-v1.md` L148–161 | Combat | `m32_automaton_a19_boost_after_beam` | Early return does not advance the turn count. |
| Numeric values for City normals, elites, bosses and summons remain at M29 values. | `m32-source-city-v1.md` §Numeric cross-check | Combat | `m29_bandit_pointy_ascension_values`; `m29_bandit_leader_ascension_values`; `m29_bandit_bear_ascension_values`; `m29_byrd_ascension_values`; `m29_centurion_ascension_values`; `m29_mystic_ascension_values`; `m29_chosen_ascension_values`; `m29_mugger_ascension_values`; `m29_shelled_parasite_ascension_values`; `m29_snake_plant_ascension_values`; `m29_snecko_ascension_values`; `m29_spheric_guardian_ascension_values`; `m29_bronze_automaton_ascension_values`; `m29_bronze_orb_ascension_values`; `m29_champ_ascension_values`; `m29_collector_ascension_values`; `m29_torch_head_ascension_values`; `m29_slaver_blue_ascension_values`; `m29_slaver_red_ascension_values`; `m29_book_of_stabbing_ascension_values`; `m29_gremlin_leader_ascension_values`; `m29_taskmaster_ascension_values` | All listed A16–A19 numeric source/engine values matched; M32 changes no numeric ladder. The M29 Spheric Guardian label issue is recorded below. |

### Lane D — Acts 3–4

| Source row | Locator | Grade | Test(s) | Note |
|---|---|---|---|---|
| Darkling Harden retains its move identity and adds Strength 2 after Block 12. | `m32-source-beyond-ending-v1.md` L21–33 | Combat | `m32_darkling_a17_harden_strength` | Payload/category change; public name remains `darkling_harden`. |
| Spire Growth forces Constrict when the player lacks Constricted and the last move was not Constrict. | `m32-source-beyond-ending-v1.md` L34–46 | Combat | `m32_spire_growth_a17_forced_constrict` | Setup and live selector routes are tested. The context-free general dispatcher remains Unsupported at A16/A17 under the rows ruling. |
| Deca Square gives each living actor Block 16 then Plated Armor 3. | `m32-source-beyond-ending-v1.md` L47–59 | Combat | `m32_deca_a19_square_plated_armor` | Payload/category change; ordered live group and dead-partner case. |
| Time Eater A19 Ripple, Head Slam and Haste payload additions. | `m32-source-beyond-ending-v1.md` L60–72 | Combat | `m32_time_eater_a19_ripple_slam_haste` | Includes final forced-Haste no-draw assertion after the attempted baseline draw was reverted. |
| Spire Shield Smash grants fixed Block 99 at A18. | `m32-source-beyond-ending-v1.md` L73–85 | Combat | `m32_spire_shield_a18_smash_block_99` | Move identity and resolved damage remain unchanged. |
| Spire Spear Burn Strike puts two Burns on top of draw at A18. | `m32-source-beyond-ending-v1.md` L86–98 | Combat | `m32_spire_spear_a18_burns_top_of_draw` | No card RNG; tests that both Burns are drawn next turn. |
| Numeric values for Beyond, Ending and Heart remain at M29 values. | `m32-source-beyond-ending-v1.md` §Numeric cross-check | Combat | `m29_darkling_ascension_values`; `m29_exploder_ascension_values`; `m29_maw_ascension_values`; `m29_orb_walker_ascension_values`; `m29_repulsor_ascension_values`; `m29_spiker_ascension_values`; `m29_spire_growth_ascension_values`; `m29_transient_ascension_values`; `m29_writhing_mass_ascension_values`; `m29_giant_head_ascension_values`; `m29_nemesis_ascension_values`; `m29_reptomancer_ascension_values`; `m29_dagger_ascension_values`; `m29_awakened_one_ascension_values`; `m29_time_eater_ascension_values`; `m29_deca_ascension_values`; `m29_donu_ascension_values`; `m29_spire_shield_ascension_values`; `m29_spire_spear_ascension_values`; `m29_corrupt_heart_ascension_values`; `m29_all_species_landed` | A16/A17/A18/A19 numeric cross-check matched source; M32 changes no numeric ladder. |

## Reconciliation

### E1 cap and E2 ceiling changes

The Standard cap is 19 and the rejection message names A0–A19. The two M28 Standard creation/public-gate tests now
reject A20. The M29 and M30 cap tests retain A20/A21 rejection and no longer treat A16 as unsupported. The renamed
M31 test is `m31_standard_rejects_a16_and_above` → `m31_standard_rejects_above_cap` (with a `Formerly` comment).

| Prior test | Current test / retarget |
|---|---|
| `m28_standard_creation_all_characters` | Same name; all characters now create through A19. |
| `m28_standard_a0_is_public_in_a_default_build` | Same name; A20 is now the rejected public boundary. |
| `m29_standard_rejects_above_cap` | Same name; A16 is accepted, A20/A21 remain rejected. |
| `m30_standard_rejects_above_cap` | Same name; A16 is accepted, A20/A21 remain rejected. |
| `m31_standard_rejects_a16_and_above` | `m31_standard_rejects_above_cap`; A20/A21 remain rejected. |

The playable-ceiling machinery and `ABOVE_CEILING` branch were removed from setup, combat entry, split, spawn and event
execution. `OUT_OF_RANGE`, `INTERIM`, species support, spawn closure and encounter metadata remain. Ceiling-pinning
tests were changed as follows; the positive A17+ admission checks replace assertions against ceilings:

| Former test | Current test / disposition |
|---|---|
| `m29_playable_ceiling_rejects_above` | `m29_combat_admission_accepts_through_a20` |
| `m29_d_formations_ceilings` | `m29_d_formations_admit_through_a20` |
| `m29_event_fight_above_ceiling_names_m32` | `m29_event_fight_admits_a17` |
| `m28_event_enemies_a0_only` | `m28_event_enemies_admit_a17_combat` |
| `m28_colosseum_above_a0_is_rejected_at_the_move` | `m28_colosseum_and_masked_bandits_fights_enter_at_a17` |
| `m29_spawn_closure_gates_encounter` | Name retained; now checks supported spawn admission. |
| `m29_every_encounter_every_ascension` | Name retained; obsolete ceiling rejection arm removed. |
| `m29_act1_normal_formations_leave_interim` | Name retained; A17+ formations are admitted. |
| `m29_c_formations_leave_interim` | Name retained; A17+ formations are admitted. |
| `m28_review_event_interim_leaves_the_whole_run` | Name retained; A21 rejection and A17 continuation are covered. |
| `m28_event_combat_unsupported_ascension_is_rejected_up_front` | Name retained; A21 remains unsupported and the A17 Masked Bandits path enters. |

The M29 `m29_all_species_landed` function name differs from a planning citation; that live test is used as named. E4
ascension threading carries selector context and turn state to the behavior owners and is behavior-neutral outside the
new threshold branches.

### E6 A16 shop order

One Standard shop owner applies the 1.1 pass after the sale, to each initially stocked card/relic/potion, before
`ShopPrice` dispatch. Courier and Membership discounts then run in source order. Purge remains excluded; all three
Courier restocks are unchanged. `m32_a16_membership_bought_in_shop` proves the in-shop Membership reprice. The
Courier reprice branch after buying a relic is unreachable because The Courier cannot be stocked; review m5 therefore
limits oracle (d) to Membership.

### C1-to-C2 traversal gate reconciliation

| Row | C1 finding | Cause | C2 correction and evidence |
|---|---|---|---|
| Slaver Red A17 guard in the unused-Entangle fallback | The c1 traversal gate found `slaver_red_scrape_scrape` at reach 3 / uniform 1 on each A17–A19 level; both gate invocations exited 1. | The A17 guard was applied only in the used-Entangle branch, while the source's final fallback applies whether Entangle is used or not. | C2 applies the guard red-first in the final fallback. `m32_slaver_red_a17_scrape_guard_before_entangle` passes; the c2 gate count is 0 at A17–A19 and both gate invocations exit 0. Evidence: `$E/m32/c1-checks/results-b.md`, `$E/m32/c2-checks/results-b.md` and `$E/m32/c2-checks/r2-traversal-summary.md`. |

### G1 authority, compatibility and first divergences

G1 is authorized by the standing no-freeze receipt and fix-globally rule, recorded in PM ruling 925f9f17. It applies
to Standard, M5–M7, M9/M10, M13, M17–M19, combat-v1 and Gym66 with no profile guard. The supplied combat-v1 fixtures
are Cultist/Jaw Worm and cannot contain a Louse; Gym66's advertised catalog includes TwoLouse and ThreeLouse. Any
changed trajectory starts at the first generated Louse opening. These are policy outcomes, not acceptance or win-rate
gates.

| Pin/profile | Change or disposition | Traced first divergence |
|---|---|---|
| `two_louse_setup_builds_independent_red_and_green_actors_and_publishes_real_moves` | Replaced forced Bite expectation with the roll derived from the same enemy-AI stream. | Initial TwoLouse selection, visit 8. Exact slot roll is UNSURE; the test computes it from the stream. |
| `fuzzfix_c2_gremlin_horn_draw_resolves_before_the_next_monster_moves`; `fuzzfix_d_power_callback_of_the_last_move_precedes_the_monster_pass`; `fuzzfix_d_monster_pass_and_round_end_follow_slot_order` | AI seed `[0x42; 32]` → `[0x02; 32]`; original effect/order assertions stay. | Turn 1 TwoLouse selection: prior stream gave slot 0 Bite/slot 1 Grow; retargeted stream gives Bite/Bite. |
| M9 agreement, seeds 7 and 11 | Rust outcome pins remain; all six cases end in Death on both trees. Candidate/base action counts: seed 7, 699/697 for all three histories; seed 11, 708/711 (Awakened One), 811/814 (Time Eater), and 828/831 (Donu/Deca). Python `test_run_from_start.py` has no step-count literal, but its per-step parity is affected; Python was not run. Evidence: `$E/m32/c1-checks/r2-examples-compare.md`. | First differing action ordinal 137 at seed 7 and 61 at seed 11, at the first Louse combat. |
| M10 agreement and Python `test_run_m10_from_start.py:294` | All six cases end in Death on both trees; candidate/base action counts are 175/177 for each seed-7 history and 108/110 for each seed-11 history. Python expected prefixes 177→175 (seed 7) and 110→108 (seed 11); Python test not run. Evidence: `$E/m32/c1-checks/r2-examples-compare.md`. | First differing observation boundary 81 at seed 7 and 66 at seed 11. |
| M13 `m13_caller_transcripts_unchanged_by_m14_extension` | Three seed-7 digests re-pinned; seed-11 digests unchanged. | Seed 7: first Louse in A1/F7, visit 2; exact action index UNSURE. Seed 11: first Louse A1/F5, visit 2; no digest changed. |
| Four performance pins and catalog | Act 1 accepted actions 212→214; Act 2 prefix 397→399. Expected HP/rewards remain unchanged. | First Louse combat on each captured route; exact action ordinal UNSURE. |
| Gym66 and M5–M7 | Louse-containing TwoLouse/ThreeLouse routes are subject to G1; fixed supplied Bite examples remain authored inputs. | First generated Louse opening. No profile-specific guard. |
| combat-v1 supplied fixtures | No fixture change: supplied single Cultist/Jaw Worm entries contain no Louse. | No Louse opening in the supplied fixture; checked as unchanged. |

M13 digest values (seed 7, in scenario order) changed as follows; the three seed-11 values did not change:

| Old digest | New digest |
|---|---|
| `ad0829f7683e82a6129d07e3a62e309e10be09845fb20504d7b22e32da63959e` | `9fcd115a6a61758017d4d245acc14b80eab0feb6a87579eef98829d82e549a72` |
| `625924fb8d554c1463be6783384e514d66a156dc0a820a62616afff3812cfaa4` | `3dfd6025e93fe9b0347ad8ee30a72a430cc9c6dc1dbfce0b6ea9555e1f374dcf` |
| `89e6dc50b0a1ded4a3657ff167cd044ddc9c742ea3c622b743f31a09cce8eda3` | `c0cf546673b57289c2502aa5ef2ea0fde774b8cb132378814a6311c51a510e37` |

The M10 exact-prefix test is a test-only expected-value edit. Its extension/Gym boundary parity was not checked because no
source-matched Python extension was built. The Rust M9/M10 routes and Gym66 Rust catalog tests passed in the Task 0
G1 evidence; the four named performance pins and the M13 debug digests were re-pinned. No Python runtime result is
claimed.

#### Regenerated M17–M19 receipts: new policy outcomes

All 18 receipts were regenerated by their original examples with the same seed, scenario, caller revision and action
budget. They are **new policy outcomes, not re-pins**. The endpoint class/act/floor is unchanged for every case; M17
accepted actions differ for six rows, while all M18/M19 action counts are unchanged. `decisions[]` identifies the
first Louse decision; `actions[]` and transition ordinals are the retained trace indices.

| Receipt | Old endpoint → new endpoint | Actions old → new | First G1 divergence / first JSON difference |
|---|---|---:|---|
| `m17/11-guardian.json` | Act2Ready A2/F17 → Act2Ready A2/F17 | 202 → 201 | LouseRed decision 27 / `actions[28]` / transition 29 |
| `m17/11-hexaghost.json` | Act2Ready A2/F17 → Act2Ready A2/F17 | 206 → 205 | LouseRed decision 27 / `actions[28]` / transition 29 |
| `m17/11-slime-boss.json` | Act2Ready A2/F17 → Act2Ready A2/F17 | 192 → 191 | LouseRed decision 27 / `actions[28]` / transition 29 |
| `m17/7-guardian.json` | Act2Ready A2/F17 → Act2Ready A2/F17 | 199 → 195 | LouseRed+Green decision 105 / `actions[114]` / transition 116 |
| `m17/7-hexaghost.json` | Act2Ready A2/F17 → Act2Ready A2/F17 | 198 → 194 | LouseRed+Green decision 102 / `actions[111]` / transition 113 |
| `m17/7-slime-boss.json` | Act2Ready A2/F17 → Act2Ready A2/F17 | 209 → 205 | LouseRed+Green decision 113 / `actions[122]` / transition 124 |
| `m18/11-guardian-bronze_automaton.json` | Death A2/F23 → Death A2/F23 | 287 → 287 | LouseRed decision 37 / receipt JSON unchanged |
| `m18/11-hexaghost-collector.json` | Act3Ready A3/F34 → Act3Ready A3/F34 | 429 → 429 | LouseRed decision 37 / receipt JSON unchanged |
| `m18/11-slime_boss-champ.json` | Death A2/F33 → Death A2/F33 | 417 → 417 | LouseRed decision 37 / receipt JSON unchanged |
| `m18/7-guardian-champ.json` | Death A2/F24 → Death A2/F24 | 316 → 316 | LouseRed decision 49 / receipt JSON unchanged |
| `m18/7-hexaghost-bronze_automaton.json` | Death A2/F23 → Death A2/F23 | 288 → 288 | LouseRed decision 49 / receipt JSON unchanged |
| `m18/7-slime_boss-collector.json` | Death A1/F16 → Death A1/F16 | 196 → 196 | LouseRed decision 49 / receipt JSON unchanged |
| `m19/11-guardian-bronze_automaton-donu_and_deca.json` | Death A2/F23 → Death A2/F23 | 287 → 287 | LouseRed decision 37 / receipt JSON unchanged |
| `m19/11-hexaghost-collector-awakened_one.json` | Death A3/F40 → Death A3/F40 | 511 → 511 | LouseRed decision 37 / receipt JSON unchanged |
| `m19/11-slime_boss-champ-time_eater.json` | Death A2/F33 → Death A2/F33 | 417 → 417 | LouseRed decision 37 / receipt JSON unchanged |
| `m19/7-guardian-champ-awakened_one.json` | Death A2/F24 → Death A2/F24 | 316 → 316 | LouseRed decision 49 / receipt JSON unchanged |
| `m19/7-hexaghost-bronze_automaton-time_eater.json` | Death A2/F23 → Death A2/F23 | 288 → 288 | LouseRed decision 49 / receipt JSON unchanged |
| `m19/7-slime_boss-collector-donu_and_deca.json` | Death A1/F16 → Death A1/F16 | 196 → 196 | LouseRed decision 49 / receipt JSON unchanged |

The M17 Act2Ready, M18 Act3Ready and receipt coverage claims remain at their recorded grades. M18/M19 normal-start
completion gates that were already unmet remain unproven; no new route coverage is claimed.

### Public intent naming and M34 exposure

The accepted PM ruling `pm-ruling-intent-names-v1.md` keeps `darkling_harden` and `deca_square` as the same move
identities while their source categories change to DEFEND_BUFF and their payloads change. A second name would split
history identity and change descriptor fingerprints. A per-monster intent category is deferred to M34 (Bead
spire-dxo.56). The public view exposes intent name and resolved intent damage, but no category or hit-count field.

M34 exposure notes: Book of Stabbing's Big Stab counter is payload-only because no public hit count is present.
spire-env's current `intent_hits("spear_skewer") = 3` while source Spire Spear has four hits from A3; M32 does not
change the facade. These limits are not inferred from the current public intent names.

### RNG-001, withdrawn findings and source-label corrections

Under RNG-001, M32 does not add draws to match `rollMove` call counts. The source consumes an unconditional `random(99)`
on fixed or forced paths where the engine may not: Chosen, Mystic's forced Heal, Snecko Glare, Taskmaster, Champ forced
branches, Bronze Automaton, Romeo, Deca, Spire Shield/Spear, half-dead Darkling and Time Eater Haste. The source also
has `takeTurn` AI draws not consumed by the engine for Mugger SFX, Gremlin Leader's quote and Centurion's protect
target. These are recorded trajectory differences under RNG-001, not behavior-row defects. Time Eater Haste consumes
zero engine AI draws: the lane's attempted `uniform_below(100)` before the forced-Haste return was reverted, and the
final `m32_time_eater_a19_ripple_slam_haste` assertion checks that the stream is unchanged.

E12: Acid Slime S changes only the empty-history opening at A17; later turns directly alternate Tackle/Lick, as at A0.
E13: `escapeNext()` has no external caller in the scanned source/JAR constant pools; the Gremlin per-monster escape
latch remains outside the live route. Gremlin Leader death queues the already modeled EscapeAction.

The M29 Spheric Guardian numeric note labeled the A17 Block 25→35 change as Harden. The source and engine assign
25→35 to the opening Activate; Harden remains Block 15. The engine numeric values are correct; the M29 label was not.

The Growth context-free dispatcher arm stays `Unsupported` at every ascension because it lacks player Constricted
context and is not a live selection route. The setup-opening and live-roll routes carry that context and are tested;
the context-free Unsupported result is retained as a regression assertion under the rows ruling.
