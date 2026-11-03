# M29 Ascension 1–5

Status: **candidate, in progress: chunk M29a1 (run rules) under review; M29a2–M29f to follow.** This
page states what M29 provides so far and its limits. It is a candidate profile, not a public
profile: M29 adds no spire-env, Gym or Python profile (PM ruling Q1 of M28 holds; native exposure is
M34–M36). Until M29f, A1–A5 standard Runs are **diagnostics-only** (`m29-diagnostics`).

## What M29 is

M29 extends the standard Run (`RoomScenario::Standard`) from A0 to A0–A5 for all four characters:
the two run-level rules the source has at A1–A5, and every numeric enemy ascension value at every
threshold (HP, damage, power amounts, block, card and hit counts). Enemy behaviour changes (AI and
new effects) stay in M32; each species has a playable ceiling below its first pending behaviour
row. Values come from a static source check (the M29 ascension inventory); expected test values
are copied from reviewed spec sheets.

M29 is delivered in merge chunks, merged in the fixed order M29a1 → a2 → b → c → d → e1 → e2 → f.

**Compatibility note.** M29a1 renames `RoomScenario::StandardA0` to `RoomScenario::Standard`.
`StandardA0` in the accepted M28 records (the M28 profile, coverage ledger and reconciliation, and
the `m28_*` test names they cite) means `Standard` at A0.

## M29a1: run rules

- **Exposure.** `Run::new_room_scenario(RunConfig { character, ascension }, profile, rng,
  RoomScenario::Standard)` admits A0 in a default build and A0–A5 with `m29-diagnostics`. A default
  build rejects A1–A5 with `Unsupported("Standard is A0 only until M29f")`; every build rejects A6+
  with `Unsupported("Standard supports A0–A5")` (`STANDARD_MAX_ASCENSION = 5`). Frozen finite
  profiles (M4–M25) stay A0-only.
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

## Limits (M29a1)

- **Interim.** Combat-level ascension lands in M29a2–M29e2. Until then, under `m29-diagnostics`, every
  A1–A5 combat except the seven encounters `setup_supports_ascension` already admits (Cultist, Jaw
  Worm, Three Cultists, Jaw Worm Horde, Gremlin Nob, Blue Slaver, Two Fungi Beasts) is rejected at
  entry with the typed `Unsupported("enemy setup facts unavailable for this act/ascension")`. Event
  fights above A0 stay disabled choices.
- A6+ run-level rules are dormant (`Run::new` codes A6/A10/A11/A14; creation caps at A5). Hand-forward
  for M30: `run/engine.rs:17` computes A6 start HP as the integer `(max * 9 + 5) / 10`; the source is
  `MathUtils.round(max * 0.9f)`.
- The native fuzz harness has no standard-Run driver and is A0/finite-profile evidence only.
