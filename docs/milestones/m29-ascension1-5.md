# M29 Ascension 1–5

Status: M29a1, M29a2, M29b, M29c and M29d merged as PR84, 86, 88, 89 and 91 (`417ad9c`). This page states the finite M29 evidence and limits; it is not a separate Gym/Python profile. Standard Run exposure followed in M34. From M29d a default build admits A0–A5 standard Runs as a
native Rust constructor; before M29d, A1–A5 was diagnostics-only (`m29-diagnostics`). The
[coverage ledger](m29-ascension1-5-coverage-ledger.md) maps every inventory row to its tests and
the [reconciliation](m29-ascension1-5-reconciliation.md) lists every disclosed change.

## What M29 is

M29 extends the standard Run (`RoomScenario::Standard`) from A0 to A0–A5 for all four characters:
the two run-level rules the source has at A1–A5, and every numeric enemy ascension value at every
threshold (HP, damage, power amounts, block, card and hit counts). Enemy behaviour changes (AI and
new effects) stay in M32; each species has a playable ceiling below its first pending behaviour
row. Values come from a static source check (the M29 ascension inventory); expected test values
are copied from reviewed spec sheets.

M29 is delivered in merge chunks, merged in the fixed order M29a1 → a2 → b → c → d. The planned
chunks d, e1, e2 and f merge together as M29d (PM ruling: one merge per milestone chunk from now on).

**Compatibility note.** M29a1 renames `RoomScenario::StandardA0` to `RoomScenario::Standard`.
`StandardA0` in the accepted M28 records (the M28 profile, coverage ledger and reconciliation, and
the `m28_*` test names they cite) means `Standard` at A0. The two M28 error strings were renamed
too: M28's `Unsupported("StandardA0 is A0 only")` is now `Unsupported("Standard is A0 only until
M29f")` in a default build (and `Unsupported("Standard supports A0–A5")` above A5), and
`"StandardA0 stored note …"` is now `"Standard stored note …"`. M29d removes the default-build
string `Unsupported("Standard is A0 only until M29f")`; A6+ still returns
`Unsupported("Standard supports A0–A5")`.

## M29a1: run rules

- **Exposure.** `Run::new_room_scenario(RunConfig { character, ascension }, profile, rng,
  RoomScenario::Standard)` admits A0 in a default build and A0–A5 with `m29-diagnostics`. A default
  build rejects A1–A5 with `Unsupported("Standard is A0 only until M29f")`; every build rejects A6+
  with `Unsupported("Standard supports A0–A5")` (`STANDARD_MAX_ASCENSION = 5`). Frozen finite
  profiles (M4–M25) stay A0-only. (M29d removes the default-build A1–A5 rejection.)
- **A1 elite quota.** The ordinary map's room-type targets use the source's binary32 law,
  `Math.round((float)n * chance)` (JDK exact rounding), for every room kind; from A1 the elite target
  is `Math.round((float)n * 0.08f * 1.6f)`, left to right. For A0 the binary32 law equals the former
  integer law for every reachable `n`, so A0 maps are unchanged. Placement rules are unchanged:
  requested elites that cannot be placed become Monster rooms. Act 4's fixed map has no quota.
- **A5 act-transition heal.** Entering a fresh act (1→2, 2→3, and 3→4 on the key route) heals
  `MathUtils.round((float)(max − cur) * 0.75f)` from A5, `max` below A5, through the ordinary heal
  owner (Mark of the Bloom blocks it). There is no heal at the Act 3 boss's Proceed or after the
  Heart.
- **Gate audit.** The Act 3 ending admits every ascension below 20 (the A20 two-boss route is a typed
  `Unsupported` naming M33); the Act 3 Falling and Tomb of Lord Red Mask handlers read the standard
  Run's ascension (frozen profiles stay A0). Event fights and Secret Portal keep reading the combat
  setup predicate.
- **Shared numeric owner.** `run/numeric.rs` holds JDK `Math.round(float)` and libGDX
  `MathUtils.round(float)`; the shop and events reuse the latter unchanged.
- **Traversal.** `examples/m28_traversal.rs` gains `--ascension N` and `--ascensions A-B`; each Run
  records its ascension, typed interim rejections, `interim_blocked` (every advertised candidate is
  a typed interim rejection: the Run stops, not a fault), per-act elite target and placement, and HP
  around each act transition.

## M29a2: combat ascension owner

- **One owner.** The combat `State` carries one `ascension`, set at entry from the setup; entry
  rejects a formation whose members disagree (`Unsupported("combat enemies disagree on
  ascension")`). Move selection, move payloads, pre-battle work, spawned and split children, the
  Awakened One's rebirth (its new maximum HP, the revival heal and both rebirth checks of the
  runtime validator) and the runtime validator's damage bounds read it. The per-actor copies in the Cultist, Jaw
  Worm, Gremlin Nob and Fungi Beast memories are folded into it and `SlaverBlueAscension` is merged
  into `SlaverBlue`; their values at every threshold are unchanged. The combat state text (and its
  hash) now prints `ascension` and the folded memories, for every combat including A0.
- **Per-species owner.** `combat/enemies/ascension.rs` holds the A0 values that used to live outside
  the species code: the Louse Bite and Curl Up and Darkling Nip roll ranges (the draw bound follows
  the range size), the Giant Head and Transient bases, the Mad Gremlin's Angry (at setup and when
  summoned), the Awakened One's rebirth HP (max HP, revival heal and validator) and the pre-battle
  power table. A0 values and RNG draws
  are unchanged; chunks b–e2 fill the ladders.
- **Ceilings and spawn closure.** Each species has a playable ceiling (16, 17, 18 or 20: one below its
  first behaviour change still pending in M32). An encounter's ceiling is the minimum over every
  species any draw of it can contain and everything those species spawn or split into (Slime Boss
  and Gremlin Leader 16, Shield and Spear 17, Donu and Deca 18). Setup and the event executor
  reject an ascension above the encounter ceiling with `Unsupported("enemy behaviour above this
  ascension lands in M32")`, setup before any RNG draw. Combat entry applies the same rule to the
  formation it is given: it rejects any member above the lowest ceiling of that member and
  everything it spawns or splits into (a hand-built solo Slime Boss or Gremlin Leader is rejected
  at A17).
- **Typed failures.** Genuine setup precondition failures (empty deck, opening draw above 10, player
  HP out of range) now return `Unsupported("combat setup preconditions failed")`, so the typed
  interim string means only "species values not landed yet".

## M29b: Act 1 normals

- **Values.** The 17 Act 1 normal species (Acid Slime S/M/L, Spike Slime S/M/L, both Louses, Cultist,
  Jaw Worm, Fungi Beast, Looter and the five small Gremlins) carry their full numeric ladder, A0–A20:
  HP from A7, damage bases from A2 (Louse Bite roll 6–8), Louse Curl Up 4–8 from A7 and 9–12 from
  A17, Red Louse Grow 4, Spike Slime L Frail 3, Mad Gremlin Angry 2 and Looter gold 20 from A17,
  Shield Gremlin Protect Block 8 from A7 and 11 from A17. Cultist, Jaw Worm and Fungi Beast were
  already implemented and are re-verified. Values come from the reviewed spec sheet (static source
  check, cross-checked with `data/enemies/enemies.json`: no disagreement).
- **Owner sites.** The Looter's per-hit gold cap (theft, its validator and the Thievery amount) and
  the Shield Gremlin's Protect Block now read the per-species owner at the combat ascension.
- **Ceilings.** Values above a species' playable ceiling (16 for the slimes except Spike Slime S, the
  Louses, Fat Gremlin and Gremlin Wizard) are checked at table level only; Gremlin Gang's ceiling is
  16. The behaviour rows behind those ceilings stay in M32: the slime, Louse and Gremlin Wizard A17
  move selection, and the Fat Gremlin's Smash adding Frail 1 from A17.

## M29c: Act 1 elites and bosses, Slavers, Act 2 elites

- **Values.** Eleven species carry their full numeric ladder, A0–A20 (elites change HP at A8 and
  damage at A3; bosses HP at A9 and damage at A4):
  - Lagavulin: Attack 20; Siphon Soul −2 Dexterity, then −2 Strength, from A18.
  - Sentry: Beam 10; Dazed 3 per Bolt from A18.
  - Hexaghost: Fire Tackle 6 and Inferno 3; Inflame Strength 3 and two Burns per Sear from A19.
  - Slime Boss: Slam 38; Goop Spray 5 Slimed from A19.
  - The Guardian: Fierce Bash 36 and Roll 10; Mode Shift 35 from A9 and 40 from A19 (at setup, then
    +10 per cycle as before); Sharp Hide 4 from A19.
  - Slaver Red: HP from A7, Stab 14 and Scrape 9 from A2; Scrape Vulnerable 2 from A17.
  - Book of Stabbing: Multi-Stab 7 and Big Stab 24.
  - Gremlin Leader: Encourage Strength 4 from A3 and 5 from A18; Encourage Block 10 from A18.
  - Taskmaster: 2 Wounds per Whip from A3 and 3 from A18.
  - Gremlin Nob and Slaver Blue were already implemented and are re-verified.
- **Source.** Values come from the reviewed spec sheet (static source check, cross-checked with
  `data/enemies/enemies.json`). The source also gives Slime Boss a Tackle damage (9, A4 10) and
  Taskmaster a second damage (4) that no move uses; the engine has no entry for either.
- **Admission.** The Act 1 elite and boss formations, Red Slaver, Exordium Thugs, the Act 2 Gremlin
  Leader, Slavers and Book of Stabbing elites, and the event fights of these species (Dead
  Adventurer, the Colosseum's two fights, Mind Bloom's Act 1 boss) leave the interim list.
- **Ceilings.** Encounter ceilings are 16 for Slime Boss (its slime children), Gremlin Leader (Fat
  Gremlin, Gremlin Wizard), Red Slaver, Exordium Thugs, Slavers and the Colosseum's first fight, and
  17 for Book of Stabbing and the Colosseum's second fight (Taskmaster). Values above a ceiling are
  checked at table level only. The behaviour rows behind those ceilings stay in M32: Slaver Red A17
  move selection, Book of Stabbing's A18 stab counter and Taskmaster's A18 Strength.

## M29d: Act 2 normals and bosses, Act 3, Act 4 and the close

- **Values.** The remaining 37 species carry their full numeric ladder, A0–A20 (normals change HP
  at A7 and damage at A2; elites HP at A8 and damage at A3; bosses HP at A9 and damage at A4):
  - Act 2: the three Bandits (Romeo's Agonizing Slash Weak 3 and Bear Hug Dexterity −4 from A17),
    Byrd (Peck 6 hits from A2, Flight 4 from A17), Centurion (Protect Block 20 from A17), Mystic
    (Strength 3 from A2 and 4 from A17; heal 20 from A17), Chosen, Mugger (Thievery 20 and Smoke
    Bomb Block 17 from A17), Shelled Parasite, Snake Plant, Snecko, Spheric Guardian (Activate Block
    35 from A17), Bronze Automaton (Boost Strength 4 from A4, Block 12 from A9) and Bronze Orb, the
    Champ (Gloat, Anger, Defensive Stance Block and Metallicize at A4/A9/A19) and the Collector
    (Buff Strength and Block at A4/A9/A19, Mega Debuff 5 from A19) with its Torch Heads.
  - Act 3: Darkling (Nip roll 9–13 from A2), Exploder, the Maw (Drool Strength 5 and Roar 5 from
    A17), Orb Walker (Strength Up 5 from A17), Repulsor, Spiker (Thorns 4 from A2 and 7 from A17),
    Spire Growth (Constrict 12 from A17), Transient (attack sequence from 40 at A2, Fading 6 from
    A17), Writhing Mass, Giant Head (It Is Time sequence from 40 at A3; first countdown 4 from A18),
    Nemesis (5 Burns from A18), Reptomancer (two Daggers per Summon from A18, at most four) and the
    Dagger.
  - Bosses and Act 4: the Awakened One (Strength 2 from A4; rebirth HP 320 from A9; Regenerate 15 and
    Curiosity 2 from A19), Time Eater, Deca and Donu (Artifact 3 from A19), Spire Shield and Spire
    Spear (Artifact 2 from A18; Skewer 4 hits from A3) and the Corrupt Heart (Blood Shots 15 hits
    from A4; Invincible 200 and Beat of Death 2 from A19).
- **Owner sites.** The literals that lived outside the species code now read the per-species owner at
  the combat ascension: the Heart's Invincible refill and Byrd's Flight refill (turn code), the
  Transient's Fading validator, the Darkling Nip and Transient base checks of the runtime validator,
  and the Giant Head's first countdown. The stored-memory check (`EnemyMemory::valid_for`) accepts
  the union over ascensions for the Darkling and the Transient; the validator, which knows the
  combat ascension, checks the exact value.
- **Global fixes.** Writhing Mass's Malleable starts and resets at 3, not 4, at every ascension
  (spire-oto.24; the one-argument `MalleablePower` constructor). The Awakened One's pre-battle
  powers are in the source's queue order: Regenerate, Curiosity, Unawakened, then Strength from A4
  (all have the same priority, so the stable sort keeps queue order). Both change A0 state; the
  reconciliation lists the amended accepted assertions.
- **Source.** Values come from three reviewed spec sheets (static source check, cross-checked with
  `data/enemies/enemies.json`). The Writhing Mass record is corrected (Malleable 3); the other
  disagreements are resolved by the source and listed in the reconciliation.
- **Ceilings.** Species ceilings: 16 for Romeo, Chosen, Mystic, Shelled Parasite, Snake Plant,
  Snecko, Darkling and Spire Growth; 17 for Spire Shield and Spire Spear; 18 for the Champ, Bronze
  Automaton, Deca and Time Eater; 20 for the rest. Encounter ceilings follow (Masked Bandits,
  Centurion and Mystic, Cultist and Chosen, the Shelled Parasite fights, Snake Plant, Snecko, Three
  Darklings and Spire Growth 16; Shield and Spear 17; Bronze Automaton, Champ, Time Eater, Donu and
  Deca 18). The behaviour rows behind them stay in M32 (the coverage ledger lists each one).
- **Close.** Every species supports A0–A20 numerically, so no encounter returns the typed interim
  rejection at or below its ceiling. A default build admits A0–A5 standard Runs for all four
  characters; the retargeted M28 and M29a1 gate tests now check the A6 rejection.
- **Traversal.** 25 seeds × 4 characters at each of A1–A5: a default build (uniform and reach) and
  `m29-diagnostics` (uniform, reach and supplied) are fault-free with no typed interim rejection.
  Supplied Runs reach the Act 2 bosses at every level and the Act 3 bosses at A1–A3. No traversal
  Run reaches Act 4 at any level, the A0 base included (deaths are policy outcomes, not a gate), so
  a constructed public Run (supplied HP and keys) covers the Act 3 boss and all of Act 4, Shield and
  Spear then the Heart, at each of A1–A5 and checks every advertised Act 4 action
  (`m29_act3_boss_and_act4_a1_to_a5`; PM acceptance at review m1: no Act 4 rule changes at A1–A5).
  Reaching Act 4 with keys the policy earned itself stays outside this evidence. The A0 default traversal equals the base; under diagnostics two supplied
  Runs differ, both after a Writhing Mass fight (the Malleable fix).

## Limits

- **Interim closed.** No standard-Run combat or event fight at A0–A5 returns the typed interim
  `Unsupported("enemy setup facts unavailable for this act/ascension")`; the code path remains only
  for an encounter with no species set and for ascensions above 20. Above an encounter's ceiling (A17+ for the lowest ceilings), setup still returns
  `Unsupported("enemy behaviour above this ascension lands in M32")`; the Run never reaches it
  (creation caps at A5).
- **Behaviour rows (M32).** Enemy AI and new effects at A17+ (for example the Mystic's heal cutoff,
  the Champ's Forge cutoff, the Snecko's Weak, Deca's Plated Armor, the Time Eater's A19 effects,
  the Shield's 99 Block, the Spear's draw-pile Burns, the Darkling's Strength and Spire Growth's
  forced Constrict) were outside M29; M32 later implemented these A17–A19 behaviors. Their amounts were checked at table level in M29 where numeric.
- A6+ run-level rules are dormant (`Run::new` codes A6/A10/A11/A14; creation caps at A5). Hand-forward
  for M30: `run/engine.rs:17` computes A6 start HP as the integer `(max * 9 + 5) / 10`; the source is
  `MathUtils.round(max * 0.9f)`.
- The M29-era native fuzz harness had no Standard Run driver and its evidence was A0/finite-profile scoped; M34 later added Standard Run fuzz coverage.
