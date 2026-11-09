# M33 Ascension 20

Status: source candidate, not accepted. A default build exposes Standard A0–A20 for all four characters; A21 is rejected. No spire-env or Python change or delivery claim is included. This profile records engine source and native tests, not original-runtime validation.

## Profile

### Standard boundary and A20 Act 3 bosses

Standard admits ascensions 0 through 20. The error for A21 is `Unsupported("Standard supports A0–A20")`. Combat-level ascension support through A20 predates M33; M33 exposes A20 through the Standard Run boundary.

At Act 3 start, Standard builds and retains the boss order once. Source `TheBeyond.initializeBoss` checks unseen bosses in priority order and duplicates the only unseen boss in its list. At A20, Proceed starts a second, distinct Act 3 boss only when the list remaining after the first boss has two entries, which happens only when all three Act 3 bosses were seen at Act start. If any boss is unseen, the Run fights one Act 3 boss. The second boss is drawn natively with key `[3, Boss, 0xB055, 1]`; it is a new scoped key and makes no source-RNG-parity claim (RNG-001). The selected order is not re-derived after the first boss is marked seen. The source also constructs a hidden boss-gold row between the fights, but no source-stream parity is claimed for that draw.

The first boss settlement gives no reward screen, claimed gold, card or potion reward, chest, key, or act-transition heal. Existing keys remain unchanged. Victory hooks still run and may heal according to the player's relics, powers and HP. Room-level potion use and discard remain available on the `Act3BossComplete` screen. Proceed enters the second boss as an ordinary room: floor +1, `FloorEntered` then normal room entry, room/floor item events and hooks, and battle-start relic effects run again. The synthetic map node copies the first boss node's coordinates, has no edges, is entered with the no-Winged-Boots-charge path, and does not change the map boss icon. The second victory then follows the ordinary Act 3 ending and key/door checks. Secret Portal's Act 3 boss receives the same second boss when its retained order has two entries. A Mind Bloom boss fight is an EventRoom fight; it neither consumes this order nor starts a second boss.

The non-Standard A20 defensive route reports `Unsupported("A20 Act 3 double boss is Standard-only")`. Current constructors cannot create a non-Standard A20 Run; this is typed defense, not a supported profile.

### Global fix G1: Act 3 unseen priority

At every ascension, the unseen priority is Awakened One, Donu and Deca, then Time Eater. This is the source order. Acts 1–2 remain in their existing source-matching domain order. Among Act 3 seen sets, only `{Awakened One}` changes its selected unseen boss (to Donu and Deca); captured profiles already use their source order. This is a global selector correction, not an A20-only guard, under the approved plan.

### Act 3 and Act 4 floors

The second boss is Run floor 51. Following a double boss, the native Act 4 map is Rest 52, Shop 53, Shield and Spear 54, Heart 55, and TrueVictory 56. These are the accepted M10 collapsed Act 3-ending mapping shifted by one room; they are not asserted as absolute source-game floor numbers. The reward ledger gains one floor credit for Act 3 `FloorEntered{act: 3, floor: 51}`; Act 4 continues to credit five rooms × ten. The floor-keyed native RNG inputs in Act 4 therefore move at A20 after a double boss. No source RNG parity or unchanged-draw-count claim is made. A19 retains floors 51–54 and TrueVictory 55. Captured M10/M15/M20/M25 Act 4 paths keep their existing floor base of 50.

The plan follows the PM floor ruling that an ordinary second room entry increments the engine floor by one. It does not adopt the addendum's literal reading of the `INTENT_TIP`-guarded `floorNum` increment as the contract: that reading conflicts with the accepted engine room-floor mapping and PM ruling m28f Q2. Source absolute floor numbering remains outside this profile's claim.

### Other A20 rules and census bound

No other A20 gameplay rule was found. A static CFR decompile of the whole game JAR's gameplay, UI and monster packages (1,637 classes, review M7; the JAR was never run) was searched for every `ascensionLevel` comparison against 19, 20 or 21 or a variable: `ProceedButton` is the sole A20 gameplay gate. A20 comparator sites in `AbstractMonster`, DeathScreen, VictoryScreen, CharacterOption, CustomModeScreen, and TopPanel concern final-boss presentation/achievements, unlocks, selector bounds, or display and are out of scope. The absence claim is bounded to that static comparison search. Existing A19+ monster thresholds remain active at A20; the census found no additional A20 monster threshold.

A diagnostic Act 3 start stores an order only when it contains two entries. At A0–A19, including the A0 diagnostic event start, M33 therefore does not add an order field to state. The only A0–A19 behavior delta in scope is G1 for the Act 3 `{Awakened One}`-seen profile; Acts 1–2 priority is unchanged.

## Traversal reach and exposure

No A20 traversal Run reached an Act 3 boss. In the runner B check (25 seeds × four characters per mode), the 800 default Runs (uniform and reach) all ended in Act 1; of the 1,200 diagnostics Runs, the supplied mode reached Act 3 (floor 40 at most) but no Act 3 boss. The earlier item-9 smoke (48 default, 16 supplied Runs) did not reach Act 3. The double-boss and Heart path claims therefore rest on constructed public-route tests: the Heart tests start from Neow with supplied 400/400 HP, three keys, and lethal hits as needed. The supplied resources and constructed route are explicit test conditions, not traversal reach or a realistic A20 win-rate claim.

Realistic A20 profiles are all-seen because seen-boss flags are global preferences and reaching A20 entails prior Act 3 victories. The one-boss A20 branch is a synthetic or custom profile case. M34's environment workload should default to an all-seen, `experienced()`-like profile.

M34 consumers should account for `Act3BossComplete + Proceed` also preceding the second boss; the synthetic second-boss node appearing in the map view (M9 map capacity 106); Act 4 floors 52–55 after a double boss; and the requirement that all three Act 3 bosses be seen for the double boss. The engine profile does not deliver those changes through spire-env or Python.

## Coverage ledger

Lane A is `m33-source-double-boss-v1.md` and its addendum. Lane B is `m33-source-a20-census-v1.md`. “Presentation, out” and “no A20 rule” identify census rows with no new M33 gameplay assertion.

| Source lane row(s) | Grade | Native test / disposition |
|---|---|---|
| A §1: all-seen list has two distinct bosses; any unseen list yields one; order built once; native second key | Run | `m33_a20_all_seen_order_two_distinct`, `m33_a20_any_unseen_order_single`, `m33_a19_order_single_unchanged`; `m33_a20_second_boss_entry`; `m33_a20_run_rules_equal_a19` |
| A §2: Proceed gate, ordinary second-room entry and event-origin exclusion | Run | `m33_a20_second_boss_entry`, `m33_a20_mind_bloom_war_keeps_order`; `m33_a19_order_single_unchanged` |
| A §3: first settlement, no rewards/chest/key/heal except victory hooks; inter-boss potions | Run | `m33_a20_first_boss_settlement_no_rewards`, `m33_a20_potions_between_bosses` |
| A §4: second boss setup, hooks and second settlement to ending | Run | `m33_a20_second_room_hooks`, `m33_a20_second_boss_settlement_reaches_ending`, `m33_a20_death_at_second_boss` |
| A §5: Secret Portal shares the double-boss route | Run | `m33_a20_secret_portal_double_boss` (to `Act3NoKeys`), `m33_a20_heart_path_after_portal` (through the Heart) |
| A §6: death, ending and tamper/invariant handling | Run | `m33_a20_death_at_second_boss`, `m33_a20_double_boss_tamper_faults`, `m33_a20_second_victory_requires_the_first` |
| A §7: no source shuffle parity; second-boss key; hidden boss-gold draw; floor-dependent RNG disclosure | Run / policy | `m33_a20_all_seen_order_two_distinct`, `m33_a20_run_rules_equal_a19`; native selection follows RNG-001. Exact original RNG parity, including the hidden boss-gold row draw, is out of scope. |
| B §1: direct A20 comparators | Run / out | Proceed gate is covered by the A rows above. AbstractMonster final-boss block, DeathScreen, VictoryScreen, CharacterOption, CustomModeScreen and TopPanel: presentation, out. |
| B §1: indirect progression, selectors, score, saves and metrics | Census | Presentation, out; no A20 gameplay rule. |
| B §2: Ruby, Emerald and Sapphire keys; Act 3 door; fixed Act 4 route; Act 4 thresholds; Heart terminal | Run | `m33_a20_second_boss_settlement_reaches_ending` (no keys: `Act3NoKeys`), `m33_a20_act4_route_floors`, `m33_a20_heart_path_all_characters`, `m33_a20_run_rules_equal_a19` |
| B §3: Neow, start state, map, regular rooms, rewards, shop, events, rest, potions | Run | No A20 rule; `m33_a20_run_rules_equal_a19` checks A20 against A19 start and first-room facts. |
| B §4: monster thresholds and Heart thresholds | Combat / Run | No A20 rule beyond already-active A19 rows; `m33_a20_run_rules_equal_a19` checks Shield and Spear and Heart setup against A19. |
| B §5: boss-count effects outside ProceedButton | Census | Per-boss hooks still run at both victories; `m33_a20_first_boss_settlement_no_rewards`, `m33_a20_second_room_hooks`; score/stat displays: presentation, out. |
| B §6: Act 1–3 unseen priority and all-seen native shuffle | Run | `m33_g1_act3_unseen_priority`, `m33_boss_unseen_priority_matrix`, `m33_a20_all_seen_order_two_distinct`, `m33_a20_any_unseen_order_single`; all-seen native RNG trace parity is out under RNG-001. |

## Reconciliation

- G1 is the source-priority correction approved for every ascension. It changes only an Act 3 profile with exactly Awakened One seen and Donu and Deca and Time Eater unseen; Act 1–2 stay unchanged. The all-seen selector retains the same candidate domain and distribution but uses native scoped RNG under RNG-001.
- The A20 Act 3 order is captured at act start and never rebuilt after a boss is marked seen. This resolves review M1: the amended `m29_act3_boss_ending_standard_a1` starts its Run at A0, so its order remains one boss even after the test sets ascension to 20. It now witnesses the ordinary single-boss ending and `Act3NoKeys`; the constructed A20 tests prove the double route.
- The second synthetic node copies the first boss node's `x` and `y`, and uses the portal-style commit path. Winged Boots charges do not change (review M2). Potion use and discard between bosses remain valid and persist (M3). The Mind Bloom event boss neither consumes the order nor starts `SecondBoss` (M4). Diagnostic starts store only two-entry orders, avoiding an A0–A19 diagnostic state delta (M5).
- Floors 51–56 are relative to the accepted M10 collapsed map, shifted by one ordinary room entry; they are not absolute source floors. This records PM ruling m28f Q2 over the addendum's literal `INTENT_TIP` reading. The reward ledger's extra Act 3 floor credit, unchanged five Act 4 credits, and A20-only movement of floor-keyed Act 4 native RNG are disclosed (M6).
- The census claim is bounded to a static search of the full-JAR CFR output (cards, relics, powers, potions, events, rewards, rooms, map, Neow, shop, helpers, dungeons, UI, screens, actions, characters and monsters) for `ascensionLevel` comparisons; it found `ProceedButton` as the sole A20 gameplay gate, with the sites listed above classified as presentation/out (M7). The one variable comparison with gameplay effect, Note For Yourself below the unlocked maximum, is already closed from A15 by the A15 rule and UNLOCK-001.
- Seen flags are global. Realistic A20 profiles are all-seen; an unseen A20 profile is synthetic/custom. M34 should default to an `experienced()`-like profile (M8). The G1 reach audit included `choose_encounter`'s Boss fallback; no additional caller without a boss icon was found (M9).
- The non-Standard A20 guard is typed `Unsupported("A20 Act 3 double boss is Standard-only")`; constructors make it unreachable in current profiles.
- Original amended bytes are retained under `m33/amended-original/` in the execution evidence. Amended tests: `m28_standard_creation_all_characters`, `m28_standard_a0_is_public_in_a_default_build`, `m29_standard_rejects_above_cap`, `m30_standard_rejects_above_cap`, `m31_standard_rejects_above_cap`, `m29_act3_boss_ending_standard_a1`, and renamed `m32_standard_rejects_above_cap` (formerly `m32_standard_rejects_a20_and_above`). The originals are evidence files, not repository paths.
- Traversal reach is limited as described above. Both boss and Heart route assertions rely on constructed/supplied tests from Neow, including 400/400 HP and three keys; lethal hits are supplied to finish the fights. These tests establish sequence and terminal behavior, not normal-start reach or strategy strength.

## Evidence

Source and test evidence: M33 execution `progress.md`, `task0-bind.md`, `item9-traversal-v1.md`, `task0-asst/`, `red1-initial.log`, `core1-after-impl.log`, and `core2-after-amend.log`. `core2-after-amend.log` records `test result: ok. 3529 passed; 0 failed; 3 ignored` for its stated spire-core lane. Item 9's short smoke reports 48/48 default A20 Runs fault-free and 16/16 supplied Runs fault-free, but neither reaches Act 3. These are source/candidate records, not the final runner gates.

- Runner A (read-only snapshot c1): `cargo test --workspace --locked --offline --no-fail-fast` 4793 passed, 0 failed, 11 ignored. The
  test-name delta against main `0ba3038a` is exactly 23 `m33_` additions and the one rename (candidate v1; the final-review fixes add `m33_a20_second_victory_requires_the_first`, so 24).
- Runner B (c1): spire-core with `m28-diagnostics,m29-diagnostics` 4166/0/5; spire-fuzz diagnostics 17/0/0; the `m28_`–`m33_` filters
  679/0/0. Traversal: A0 (diagnostics and default) and A1–A19 (default), 25 seeds × four characters per mode, are identical to base in every
  report field outside the `m29`–`m33` keys and in every per-step digest. A20: 800 default and 1,200 diagnostics Runs, 0 faults, 0 interim;
  the base tree rejects A20 at creation. No A20 traversal Run reached an Act 3 boss. Native fuzz (seeds 1–100, four drivers): 400 episodes,
  0 failures, same summary hash as base. Frozen replay: 323 of 323 inputs equal. Examples (M9/M10 agreement, M21–M24 modes, spire-env
  and spire-core example tests): equal to base.
- Reference data: `data/events/events.json` pins the sha256 of `docs/intentional-deviations.md` and cites it by line, so the new M33 entry
  is appended at the end of that file and `events.json` is regenerated (source hash only). Event and enemy reference checks pass on the
  final tree.
- Independent final milestone review: READY (0 Critical, 0 Important, 6 Minor). The minor fixes are tests and docs only: the Secret Portal tests are split (ending without keys; Heart path with keys), `m33_a20_run_rules_equal_a19` builds each Run from its seed, two stale cap-test doc comments are amended, these docs state reach exactly, and `m33_a20_second_victory_requires_the_first` covers the history and first-node conditions of the second boss's source (both conjunct deletions fail it).
