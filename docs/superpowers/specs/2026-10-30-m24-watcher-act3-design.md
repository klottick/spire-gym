# M24 finite Watcher A0 Acts 1–3 design

Date: 2026-10-30. Status: **planning proposal, version 2**. Version 2 resolves the
independent plan review v1 (NOT READY, 0 Critical / 8 Important / 7 Minor, report SHA-256
`53031efc3faa39d265560fbf4f77803208aa5727dc6525c72fc845e4925964fd`); the disposition
table is retained with the planning evidence (section 14).
Planning Bead `spire-dxo.46.1`; roadmap outcome `spire-dxo.46` ("M24 — Watcher through
Act3"). Assigned source: `main` at `38496f922d1b272f5454034cb3bb11110e6843fd` (M23
merged in PR62, M20 witness documents in PR63). The paired
[implementation plan](../plans/2026-10-30-m24-watcher-act3.md) turns this design into
tasks. This document releases no runtime work by itself.

## 1. What M24 delivers

The [roadmap](2026-10-22-narrow-gym-milestones-design.md) defines M24 as "From-start
Acts1–3 progression and actual boss interactions" for the Watcher. M24 continues the
exact M23 Run. M23 starts a Watcher A0 Run at short Neow, plays Act1 and Act2, and stops
at the healthy, zero-action `Act3Ready` screen at act 3, floor 34. M24 removes that stop.
The same Run publishes the generated Act3 map and plays Act3 through its ordinary rooms,
elites, events, shops, rest sites, treasure chest and boss. It ends at the end of Act3:

- After an actual Act3 boss Victory without all three keys, the Run ends with the
  existing `Act3NoKeys` terminal.
- After an actual Act3 boss Victory with all three keys, the Run stops at the existing
  healthy, nonterminal `Act4Ready` cutoff (act 3, floor 50, zero legal actions). Stepping
  there returns `UnsupportedBoundary("M24 stops before the Act4 door")`.
- A genuine Death or EmptyDeck ends the Run as usual at any point.

M24 is the M19 pattern (Defect Act3) applied to the Watcher. Act3 rooms, the Act3 boss
and the Act3 ending already exist as shared owners. M24 adds a profile that routes the
Watcher Run through them. It adds only the content that Act3 actually forces on a
Watcher Run, plus Watcher-specific interaction tests against the real Act3 enemies.

Act4, the Act4 door transition, Shield/Spear and the Heart are M25. The door transition
frame (the one M20 records as `ActAdvanced{from:3,to:4}` with Rest at floor 51) is not
entered by M24. The "floor 51" boundary therefore appears in M24 only as the thing it
stops in front of.

## 2. Rules that shape this design

These are binding user rules, restated so the builder does not need to look them up.

1. **The repository plays the game; it does not need to win it.** The primary scope
   clarification of 2026-10-30 says winning with this code is not in scope and a
   supplied victory is fine. So M24 has no normal-start win gate and no machinery whose
   only purpose is to chase wins. Canonical normal runs are frozen, run once, and
   recorded exactly as they happen. Supplied setups play each Act3 boss and the Act3
   elites and encounters for real, so the mechanics are exercised even when normal runs
   die early.
2. **Fix real bugs globally.** A defect found in a shared owner is fixed at the shared
   rule for every character. An accepted assertion that pinned wrong behaviour is
   amended, the change is disclosed, and the original bytes are retained. No narrow
   Watcher-only guard is allowed to hide a shared defect.
3. **One engine.** Extend the existing Run, rooms, combat, resolver, capture, encoder,
   session, replay, batch, rollout and caller owners, and the default-off diagnostics
   features. No copied loops, tables or encoders. No public supplied-run or import API.
4. **No installs.** No new dependencies, no Python rebuild, no original-game execution.

## 3. Approaches considered

| Approach | Result | Decision |
| --- | --- | --- |
| Add `RoomScenario::M24WatcherAct3` and route it through the shared full-Act3 owners, reusing the M22/M23 Watcher producers, with Parasite, Void and Red Mask added because Act3 forces them | Smallest change. Every Act3 rule already exists and has been accepted for Silent (M14) and Defect (M19). The Watcher-specific risk sits in combat interactions, which get dedicated tests. | **Selected.** |
| Also add Act3-specific Watcher card reward rows and shop rows, the way M19 added Defect Act3 triples | More Watcher cards in Act3, but no Act3 progression witness needs them. Card breadth belongs to M25, which must deliver complete ordinary Watcher A0 content. | Rejected for M24. |
| Implement the standard Watcher reward, shop and event pools now | Absorbs M25 and M26–M28, with their unresolved generator and source work. | Out of scope. |
| Skip normal runs and use supplied Act3 starts only | Loses the from-start evidence and the exact prefix check against M23. | Rejected. Supplied runs supplement normal runs; they do not replace them. |

## 4. Canonical start and the exact M23 prefix

### Start

Add `RoomScenario::M24WatcherAct3` to the existing
`Run::new_room_scenario(RunConfig, CreationProfile, RunRng, RoomScenario)` in
`crates/spire-core/src/run/rooms/mod.rs`. It admits Watcher, ascension 0, short Neow
only; a different character, ascension or full Neow returns
`RunError::Unsupported("M24 requires Watcher A0 short Neow")` before any mutation. A
profile that is not fully unlocked, or has Act 4 disabled, is rejected by the existing
`Run::new` check with its existing message, `"profile must be fully unlocked with Act 4
enabled"` (`crates/spire-core/src/run/engine.rs`), exactly as for M22 and M23; M24 adds no
second check.
Canonical resources come from the existing `Run::new`/Catalog path, exactly as in M23:
72/72 HP, 99 gold, four StrikePurple, four DefendPurple, Eruption and Vigilance as ten
distinct base permanent instances, PureWater, three empty potion slots, no keys, and
Act 4 enabled (the engine requires `act4_enabled`; see
`crates/spire-core/src/run/engine.rs`). No resource is supplied through the normal
profile.

### Prefix equality with M23

M24 must not replay, copy or regenerate anything M23 already does. Acts 1 and 2 run
through the same shared owners with the same producers, captures and RNG scopes. The
contract is the same one M23 used against M22:

- Construct M23 and M24 through the core constructor with the **same**
  `CreationProfile`, including the same Act3 `seen_bosses` entry, the same seed,
  partition and ordinal. Drive the same recorded semantic action prefix in both.
- Through Act1, the `(1,2,17)` transition, Act2, the Act2 boss reward and boss treasure,
  and the committed `(2,3,34)` transition, the two Runs agree on HP, max HP, gold,
  permanent cards (identities, order, upgrades), relics and counters, potions, keys,
  Neow state, combat and room records, captures, RNG bytes and counters, transition
  flags, the generated Act3 map and the Act3 boss order. Emitted gameplay and reward
  events and public choices match in order.
- The only allowed differences are metadata: the scenario tag, profile, content, rules,
  schema and replay identifiers, descriptor and fingerprint, profile numeric code,
  transport session identifiers and tokens, and additive vocabulary indices. Encoded
  references are compared by semantic identity.
- At the committed `(2,3,34)` transition, M23 publishes `Act3Ready` with zero actions.
  M24 publishes the same successor map as `Map` with legal Act3 move candidates and
  ordinary inventory actions, and initializes `Act3Progress` exactly once. That
  continuation and its room label are the only intended gameplay-facing difference.

**The two M23-only branches.** Exactly two core behaviour branches name M23 alone, and
both must include M24, or the M24 Act2 prefix would drift from M23:

| Branch | Today | M24 change |
| --- | --- | --- |
| `crates/spire-core/src/run/rooms/shop.rs` Watcher stock, first versus later merchant | Reads `Act2Progress.shop_visits` only `if is_m23 && destination act == 2`; otherwise reads the Act1 counter | Read the Act2 counter for M23 **or** M24 in Act2, and the Act3 counter for M24 in Act3 (see "Merchants" below) |
| `crates/spire-core/src/run/rooms/m23_content.rs` `relic_capture_scope` | `run.m23.relic.capture` only for M23 in Act2 | Same Act2 scope for M23 or M24; `run.m24.relic.capture` for M24 in Act3 (section 7) |

**How the prefix is proved.** Three checks, each at its own grade:

1. A matched-history **played** prefix in core: build M23 and M24 with the same
   constructor inputs, take each action from the M23 Run's own legal candidates by one
   fixed test rule, apply the identical action to the M24 Run, and compare the full state
   after every step until M23 reaches a terminal, `Act3Ready` or the step limit. This
   plays real Act1 and Act2 rooms; how far it gets is reported, not required.
2. A matched **owner-level** Act2 check: from equal constructed Act2 states one map edge
   before a merchant, a second merchant and an Act2 elite, really enter each room in both
   profiles. The first and later merchant rows, the elite relic capture row and the RNG
   must be equal.
3. The six canonical normal runs (section 11), compared step by step with the retained
   M23 receipts. Their Act3 history differs from native M23, but `seen_bosses[2]` is read
   only when the `(2,3,34)` transition picks the Act3 boss order
   (`crates/spire-core/src/run/rooms/transition.rs`, `successor`), so every step before
   that transition must match.

The constructed Act2 boss-source check is kept for the boss-treasure branches and the
`(2,3,34)` transition itself.

**Matched versus unmatched future history.** Native M23 keeps the default
`CreationProfile::experienced()` Act3 history, in which all three Act3 bosses are seen.
Its `(2,3,34)` transition can therefore draw boss-order shuffle tickets. M24's native
history names one unseen Act3 target, so it does not. This is the same situation M23
disclosed against M22. The exact-equality proof uses the matched core constructor;
native M23 and native M24 runs are compared only up to the `(2,3,34)` transition, and the
difference after it is disclosed, not normalized away.

## 5. Finite content: what Act3 adds and why

M24 keeps the M22/M23 producers unchanged. It adds a kind only when a captured Act3 room
or encounter produces it and refusing it would be wrong gameplay.

### Permanent cards: 29 kinds (M23's 28 plus Parasite)

The 28 M22/M23 kinds stay as they are: StrikePurple, DefendPurple, Eruption, Vigilance,
EmptyFist, MentalFortress, CarveReality, CutThroughFate, Rushdown, DeceiveReality,
ThirdEye, Foresight, Meditate, JustLucky, Nirvana, Perseverance, FlurryOfBlows, Weave,
SandsOfTime, Prostrate, Worship, WindmillStrike, Brilliance, Devotion, Establishment,
BattleHymn, LikeWater and MasterReality.

**Parasite is added** because WrithingMass is in the captured Act3 ordinary cycle and
its `writhing_parasite` move obtains a permanent Parasite through the shared permanent
gain journal (`crates/spire-core/src/combat/enemies/act3/writhing.rs`). The journal
sync at settlement (`crates/spire-core/src/run/combat.rs`, `prepare_gains`) does not
consult `mutations::supported_in`, so the Parasite does enter the Run deck.

**The real block is Watcher combat construction.** Every Watcher-content Run enters
combat with `CombatItemContext::with_m22_profile()` (`run/combat.rs`), and combat entry
then calls `cards::watcher::initialize_profile` on every deck card
(`crates/spire-core/src/combat/engine.rs`, the `m21_profile()` branch). That function
rejects any card outside `content::watcher::supports_generated`
(`crates/spire-core/src/combat/cards/watcher.rs`), so the next combat after an Implant
fails at entry. Merely adding Parasite to that set would panic, because the cost falls
back to `starting_cost(..).unwrap()` and Watcher `starting_cost` returns `None` for
Parasite.

**Decision: a global Watcher combat-construction law.** For every Watcher combat
profile (M21 authored combat, M22, M23 and M24), `initialize_profile` accepts a
permanent card whose catalog type is Curse and leaves it exactly as the shared
constructor built it: no Watcher cost, damage, block, magic number or `m21` flag. The
shared card owner already treats Parasite as unplayable and costless
(`crates/spire-core/src/combat/cards/mod.rs`). Parasite is the only Curse any captured
Watcher room produces, but the law is stated by card type, not by Parasite's ID. Watcher
cards and generated cards keep the existing initialization and the existing rejection of
anything else (Status or off-color cards in a starting deck still reject). The public
view needs no new code: `cards::watcher::public_values` returns `None` for a card without
the `m21` flag and without a Watcher starting cost, and `card_values` derives no combat
cost from the shared `base_cost`, so a Parasite is shown with no Watcher values and no
price, the same as in other characters' combats. A read-only sweep found no accepted test
that pins rejection of a Curse at Watcher combat entry (the M21 test that rejects
`Strike` and `Miracle`, `crates/spire-core/src/run/tests/m21_watcher.rs`, checks the
supplied-resource validator, which this law does not touch). If Task 0 finds one, it is
amended under rule 2 with the change disclosed.

`mutations::supported_in` also admits Parasite for M24 only, which keeps `can_upgrade`,
Smith eligibility and `obtain_card` consistent for the Run deck. Parasite is never
offered as a reward or shop card, is not upgradable, and its removal applies the
existing max-HP loss once. M22 and M23 keep `supported_in` unchanged, because no M22/M23
room produces Parasite; the construction law above still applies to them.

Card rewards keep the M23 law: the **full global** `CombatRecord.visit` modulo 6 selects
the ordinary/elite row, across the act boundary, with Skip. Act3 does not reset this
cycle. The Act3 boss gives no card reward (see section 8), so the boss rows
(Brilliance, Devotion, Establishment) still come only from the Act1 and Act2 bosses.

### Relics: 19 kinds (M23's 18 plus Red Mask)

The 18 M22/M23 relics stay: PureWater, NeowsLament, Vajra, OddlySmoothStone, GoldenEye,
TeardropLocket, Lantern, Pear, Mango, ToyOrnithopter, Strawberry, Anchor, BloodVial,
HolyWater, VioletLotus, SacredBark, EmptyCage and SlaversCollar.

**Red Mask is added** because the second Act3 question room is the existing
`TombOfLordRedMask` event (`crates/spire-core/src/run/rooms/events/act3.rs`). Without a
mask, its offer trades all gold for Red Mask; with a mask, it grants 222 gold. Red Mask
applies 1 Weak to every enemy at combat start through the shared relic owner
(`crates/spire-core/src/items/relics.rs`, `battle_start_enemy_effects`, run at
`Hook::CombatStarted` in `crates/spire-core/src/combat/effects.rs`). It is not added to
any elite or chest pool.

**The real block is Watcher combat inventory validation.** At every combat entry,
`items::validate_combat_inventory` (`crates/spire-core/src/items/mod.rs`) requires every
relic of a Watcher-progression context to be in `M22_WATCHER_ACT1_RELICS`. Red Mask is
not, so the first combat after Tomb would fail at entry, and the frozen caller always
takes Tomb's offer.

**Decision.** Add `CombatItemContext::with_m24_profile()` in `items/mod.rs`. It does
everything `with_m22_profile()` does and also sets a new `watcher_act3` flag.
`run/combat.rs` selects it when `is_m24`, and `with_m22_profile()` otherwise for M22 and
M23. When the flag is set, validation checks relics against `M24_WATCHER_ACT3_RELICS`
(owned by `m24_content.rs`) instead of the M22 list; nothing else in the validation
changes. The context's `Debug` output prints `watcher_act3` only when it is true, so
M22/M23 checkpoint and fixture text is unchanged. No relic list is copied.

**Taxonomy.** The M22 relic taxonomy (`m22_content::RELIC_TAXONOMY`) stays frozen.
`m24_content.rs` owns one addition, `RELIC_TAXONOMY_ADDITIONS = [(RedMask, "special")]`
(the reference record in `data/relics/relics.json` gives Red Mask tier `SPECIAL`), and
the M24 descriptor lists the M22 rows followed by that addition.

### Potions: 7 kinds, unchanged

BlockPotion, EnergyPotion, DexterityPotion, AncientPotion, StancePotion, Ambrosia and
BottledMiracle, selected by the full global visit modulo 7, as in M23.

### Generated-only and status cards: 3 plus 5 kinds

Miracle, Smite and Safety stay generated-only. The statuses are M23's Dazed, Burn,
Slimed and Wound, plus **Void**, which Awakened One's phase-two Sludge inserts at the
**bottom** of the draw pile (`crates/spire-core/src/combat/enemies/act3/awakened.rs`,
`BottomDraw`; static findings: "inserts at BOTTOM, not random"). Act3 producers of the existing statuses are OrbWalker (Burn), Nemesis (Burn),
Repulsor (Dazed), Deca (Dazed into discard) and Reptomancer's Daggers (Wound). Upgraded
Burn remains an upgrade state of Burn, not a new kind.

The reachable physical vocabulary is therefore 29 permanent + 3 generated-only + 5
statuses = **37 card kinds**.

### Merchants

Act3 merchants use the same M23 rows. "First merchant of the act" versus "later
merchant" is decided by the Act3 shop-visit count **before** it is incremented, read from
`Act3Progress.shop_visits`, exactly as M23 reads `Act2Progress.shop_visits` for Act2.
Today the Watcher branch of `crates/spire-core/src/run/rooms/shop.rs` reads the Act2
counter only for M23 and otherwise the Act1 counter. It must read the Act2 counter for
M23 or M24 in Act2, and the Act3 counter for M24 in Act3. Purchases do not decide first
versus later.
Purge cost keeps its existing lifetime owner.

## 6. Act3 rooms

All Act3 rooms use the shared owner `crates/spire-core/src/run/rooms/act3.rs`, which
already serves M9, M10, M14, M15, M19 and M20. M24 joins `uses_full_act3`, which also
places it in `uses_captured_act3`, `uses_captured_act2`, `uses_act2_boss` and
`uses_captured_act1`. M24 does **not** join `uses_act4`. M24 joins
`uses_watcher_content`, so all M22/M23 Watcher producers stay active in Act3.

| Floor | Room | Owner and behaviour |
| --- | --- | --- |
| 35–49 | Map rooms | Generated by the actual `(2,3,34)` transition, global floors shifted by 34 |
| Monster | Ordinary capture | History length 0 → OrbWalker, 1 → ThreeShapes, then the cycle SpireGrowth, Transient, FourShapes, Maw, SphereAndTwoShapes, JawWormHorde, ThreeDarklings, WrithingMass, avoiding the previous two and the shapes pair |
| Elite | Elite capture | GiantHead, Nemesis, Reptomancer, offset by the actual boss icon (AwakenedOne 0, TimeEater 1, DonuAndDeca 2), no immediate repeat |
| Question | Events, then Monster | First question Falling, second TombOfLordRedMask, later questions are captured Monster rooms |
| 43 | Treasure | Captured chest relic (section 7) and the existing exclusive Sapphire/Proceed choices |
| 49 | Rest | Shared campfire: Rest, Smith, Recall for the Ruby key |
| 50 | Boss | The first entry of the actual Act3 boss order, which must match the map icon |

These are the **16 Act3 encounter setups** already named in
`enemies::A0_ENCOUNTER_SETUPS`: OrbWalker, ThreeShapes, SpireGrowth, Transient,
FourShapes, Maw, SphereAndTwoShapes, JawWormHorde, ThreeDarklings, WrithingMass,
GiantHead, Nemesis, Reptomancer, AwakenedOne, TimeEater and DonuAndDeca. JawWormHorde
keeps its existing setup-table index and RNG key. M24 adds them to M23's 24 playable
encounters for **40 playable encounters**. The Act3 actor union is 20 species
(OrbWalker, Repulsor, Exploder, Spiker, SphericGuardian, JawWorm, Darkling, SpireGrowth,
Transient, Maw, WrithingMass, GiantHead, Nemesis, Reptomancer, Dagger, Cultist,
AwakenedOne, TimeEater, Deca, Donu). SphericGuardian, JawWorm and Cultist are already in
M23, so 17 are new, for **54 cumulative enemy identities**, the same total M14 and M19
reached. Shapes, Daggers and boss partners arise only through the real setup and summon
owners.

**Falling** (`RunRngScope::Act3Falling`) captures one random card of each type (Attack,
Skill, Power) when the event is prepared and offers to remove the captured instance.
Watcher cards carry ordinary card types, so the event needs no Watcher change. A type
with no candidate is shown disabled. Removing the last card ends the Run with EmptyDeck.

**Tomb of Lord Red Mask** needs no event change. Its Red Mask is admitted to later
Watcher combats by the inventory decision in section 5.

Shared key owners stay active: Recall at a campfire gives the Ruby key, Sapphire at a
chest gives the Sapphire key, and a burning elite Victory gives the Emerald key. Each key
adds its existing +5 reward once. Keys matter only at the Act3 ending (section 8).

Fault-checkpoint labels: the room-entry and ending selectors in `act3.rs` and
`act3_end.rs` currently fall back to M8/M9 label strings for unlisted profiles (M19
included). M24 adds its own arms, `run.m24.room`, `run.m24.encounter` and
`run.m24.act3.end`, so a retained M24 fault names its real profile. Existing profiles'
labels do not change.

## 7. Relic capture in Act3

M24 reuses the single capture helper `rooms/rewards.rs::capture_relic_in_scope` with the
M22 eligibility rules: filter owned relics first, keep source order, make no RNG request
for an empty pool, and make exactly one checked `RunRng::bounded_scoped_checked` request
for any non-empty pool, including a pool of one.

For Act3 only, append `RunRngScope::M24RelicCapture` (spelling
`run.m24.relic.capture`) as the last variant in `crates/spire-core/src/run/rng.rs`. Its
key is the same as M22/M23: `[act, floor, node_id, full_visit, source_tag]`, with tag 0
for elites (using the actual `CombatRecord.visit`) and tag 1 for the ordinary chest
(using `combat_progress.entered` at the accepted OpenChest). The Act1 scope
`run.m22.relic.capture` and Act2 scope `run.m23.relic.capture` stay unchanged. The scope
choice extends the existing `relic_capture_scope` function rather than adding another.

The pools are unchanged: elites draw from the eight M22 elite relics (Vajra,
OddlySmoothStone, GoldenEye, TeardropLocket, Lantern, Pear, Mango, ToyOrnithopter) and
the chest from the three chest relics (Strawberry, Anchor, BloodVial), each filtered for
ownership. By Act3 these finite pools may be empty. An empty pool simply omits the
relic, still allows Proceed and an eligible Sapphire, and is neither a fault nor Death.
The stock is never refilled or replaced to hide exhaustion.

## 8. Act3 boss, ending and the end-of-Act3 boundary

The Act3 boss is the first entry of `act3_boss_order` that the actual `(2,3,34)`
transition created. Its entry consumes the target only on accepted `enter_owned`.

After an actual boss Victory at floor 50, the shared ending owner
(`crates/spire-core/src/run/rooms/act3_end.rs`) runs unchanged:

1. Settlement returns combat-owned inventory, checks the full card census and writes the
   matching `CombatRecord`. There is **no** card reward, gold, potion reward or third
   boss chest after the Act3 boss.
2. The screen `Act3BossComplete` offers Proceed.
3. Three `Act3Ending` dialog stages each offer `Act3Continue{stage}`.
4. After stage 3, because M24 is not in `uses_act4`:
   - with `act4_enabled` and all three keys, the Run stops at **`Act4Ready`**: act 3,
     floor 50, no terminal, zero candidates, all-zero action mask, `terminated=false`,
     `truncated=false`, no heal, no `ActAdvanced`, no Act4 map and no readiness reward;
   - otherwise, stage 4 is offered and accepting it commits **`RunTerminal::Act3NoKeys`**
     once, with terminal reward contribution 0, no floor-51 credit and no Heart bonus.

**Session behaviour at `Act4Ready`** follows the M14/M19 pattern in
`crates/spire-env/src/run_session.rs`, not M9's:

- *Publication.* The step that reaches `Act4Ready` succeeds. `check_boundary`, which runs
  when a decision is published, treats an M24 `Act4Ready` like the M14 arm: a
  nonterminal, zero-candidate decision is accepted as healthy, and anything else is an
  error (`"M24 Act4Ready must be nonterminal with no actions"`). M24 must not get an
  error-returning arm like M9's.
- *Next step.* M24 joins the step-time cutoff predicate (`at_m14_cutoff`, set when the
  published decision is the profile's `Act4Ready`). `step_preflight` then returns
  `UnsupportedBoundary("M24 stops before the Act4 door")`, after the existing stale-token
  check and before index validation or any mutation, as for M14 and M19.

Reset, record, reproduction and close stay available. The Act4 door transition belongs
to M25.

`Act3NoKeys`, Death and EmptyDeck are distinct outcomes. A budget exhaustion is an
unfinished caller result, and a fault is an operational result; neither is Death or
completion.

## 9. Watcher-specific Act3 interactions

M21 implemented the Watcher mechanics, and M8/M9 implemented the Act3 enemies. Generic
Act3 enemy tests exist (`crates/spire-core/src/combat/tests/m8_*.rs`) and M19 composed
them with Defect (`m19_defect_interactions.rs`), but the only Watcher-versus-Act3 test
today is Meditate at Time Warp 11 (`crates/spire-core/src/combat/tests/m21_watcher_scry.rs`). The facts below come from the
source-graded [Act3 static findings](../../../data/enemies/m8-static-findings.md), the
[Watcher research](../../research/2026-10-19-act1-content-watcher.md) and the
[reference index](../../../data/README.md). An expected value that is not settled by a
named source fact must be bound to a cited source before its test is written; it must
not be guessed.

Each row becomes a combat-level composition test using real setups from
`enemies::setup` and legal actions, in a new file
`crates/spire-core/src/combat/tests/m24_watcher_act3.rs`. A test that passes on first run
is a valid regression; do not change rules to produce a failure.

| # | Interaction | What the test must show |
| --- | --- | --- |
| W1 | Time Eater Time Warp counts Watcher plays | Every played card counts, including 0-cost, retained and generated Miracle, Smite and Safety. Potions do not count. FlurryOfBlows returning from discard on a stance change is a move, not a play, and does not count. |
| W2 | Time Warp's twelfth card is a stance card | When the twelfth card is Eruption or Vigilance, its payload and stance change resolve first (including MentalFortress block and Rushdown draw requests that the source orders before after-use), then the turn ends early. No thirteenth card action is published. Time Eater gains 2 Strength, and the counter resets to 0 and persists across turns otherwise. |
| W3 | End of an early-ended turn | Retain, Establishment cost reduction, Perseverance and SandsOfTime growth, LikeWater block in Calm, and Ethereal exhaust happen exactly once, as in a normal end of turn. A Divinity entered by the twelfth card (Worship or Prostrate reaching 10 Mantra) keeps its energy for that turn only and exits at the next turn start. |
| W4 | Awakened One Curiosity | Each actual Power play in phase one (MentalFortress, Rushdown, Foresight, Nirvana, Devotion, Establishment, BattleHymn, LikeWater, MasterReality) gives the boss Strength equal to Curiosity (1). Skills, attacks, generated Smite and Miracle, and stance changes do not. |
| W5 | Awakened One rebirth with stance damage | A lethal Wrath or Divinity hit in phase one leaves the same actor half-dead (no overkill carries), with no premature Victory or settlement even if it is the only active actor. Rebirth restores 300 HP, removes Curiosity, Unawakened and debuffs, and keeps positive Strength. The final death makes surviving Cultists escape and writes one matching record. |
| W6 | Void with Watcher energy | Drawing Void loses 1 energy through the shared owner, after Calm-exit, Divinity, Violet Lotus and Teardrop energy have been applied in their real order. Scrying a Void to discard does not trigger its draw effect. |
| W7 | Donu and Deca Artifact | Red Mask's combat-start Weak consumes one of each actor's two Artifact charges and applies no Weak. AncientPotion's single Artifact charge on the player blocks only the first debuff of a multi-debuff move (Time Eater Ripple, Maw Roar); the second debuff applies, in the source order. The Red Mask case enters combat with an item context built by `with_m24_profile()` holding Red Mask, so it runs after the H1 snapshot provides that constructor; the AncientPotion case can run earlier under the existing Watcher context. |
| W8 | Deca Dazed with retain | Meditate can return a Dazed to hand and grant it Retain. The end-of-turn result follows the accepted M21 Ethereal-retain-Establishment ruling; the test cites that ruling and does not re-decide it. |
| W9 | Giant Head Slow with stances | Slow scales NORMAL damage after the Wrath or Divinity multiplier and before flooring. Slow increases only after a card's queued payload, so a multi-hit card (FlurryOfBlows, WindmillStrike) does not raise Slow between its own hits. |
| W10 | Nemesis Intangible with stances | Each positive hit is capped at 1 after the stance multiplier, so Wrath and Divinity hits deal 1 each. Zero damage stays 0. Nemesis Burns reach the discard pile; a Burn in hand at end of turn deals its damage as non-NORMAL damage, which Wrath does not double. |
| W11 | Thorns-type damage versus Wrath | Wrath doubles only incoming NORMAL damage. Spiker retaliation, Exploder's 30 explosion and SpireGrowth's Constricted end-of-turn damage use the Thorns payload and are not doubled. Spiker retaliates once per hit, so a multi-hit card can kill the player mid-card, which ends the card through shared cancellation. |
| W12 | Writhing Mass Reactive, Malleable and Implant | Each hit of a multi-hit card triggers Malleable Block growth and a Reactive re-intent. Implant records exactly one Parasite in the combat's permanent gain journal. (Settlement into the Run deck and the next combat are Run-level and are tested in plan Task 2, not here.) |
| W13 | Transient, Darklings and Reptomancer targets | Shifting's Strength loss follows actual HP loss. Fading's death is a Victory with one record. A half-dead Darkling is not a legal attack target, and Regrow and Life Link use the shared owners. Reptomancer Daggers appear only through the summon owner, are legal targets in actor order, and Dagger explosion follows its source payload. |
| W14 | Scry against Act3 statuses | Scry reveals the top cards of the draw pile, so it reveals OrbWalker's and Repulsor's shuffled Burn and Dazed when they lie within the Scry amount, and Awakened One's bottom-inserted Void only when the draw pile is no larger than the Scry amount. Deca's and Nemesis's statuses go to the discard pile and are not revealed. Discarding a status through Scry moves it without playing or drawing it, and Weave returns on Scry as usual. |

Where a row's exact member ordering (for example W2's barrier ordering against Rushdown's
queued draw) is not closed by an accepted source fact, the test pins only what the
accepted shared owner and the cited fact establish, and the open part is listed in the
reconciliation. It is not filled in by guesswork.

If any test exposes a defect in a shared owner, the fix follows rule 2: correct the
shared law for all characters, amend any accepted assertion that pinned the wrong
behaviour with a disclosed diff, retain the original bytes, and add a regression that
fails under a narrower fix. The impact on previously accepted profiles and retained
outcomes is recorded the way M23 recorded the Confused cost correction.

## 10. Native profile, schema and history

All names are proposals until Task 0 of the plan maps them to the accepted M23 owners.

| Item | Proposed value |
| --- | --- |
| Scenario | `RoomScenario::M24WatcherAct3`, predicate `is_m24` |
| Profile | `RunProfile::M24WatcherAct3` |
| Constructors | `RunSession::new_m24(config: RunSessionConfig) -> Result<Self, RunSessionError>`; `RunBatch::new_m24(configs: Vec<RunSessionConfig>, workers: usize) -> Result<Self, RunBatchError>` |
| History | `RunBossHistory::M24WatcherAct3 { act1: RunAct1BossTarget, act2: RunAct2BossTarget, act3: RunAct3BossTarget }` |
| History JSON | `{"m24-watcher-act3":{"act1":"guardian","act2":"champ","act3":"awakened_one"}}` |
| Content ID | `spire-gym-m24-watcher-a0-acts1-3-captured-v1` |
| Rules ID | `spire-gym-native-m24-rules-v1` |
| Observation ID | `spire-gym-m24-watcher-act3-observation-v1` |
| Replay ID | `spire-gym-m24-watcher-act3-replay-v1` |
| Fingerprint domain | `spire-gym/run-schema/m24-watcher-act3/v1\0` (one terminating NUL byte) |
| Cutoff error | `UnsupportedBoundary("M24 stops before the Act4 door")` |

Target spellings reuse the existing shared codecs: Act1 `guardian`, `hexaghost`,
`slime_boss`; Act2 `champ`, `bronze_automaton`, `collector`; Act3 `awakened_one`,
`time_eater`, `donu_and_deca`. Task 0 confirms these against the accepted strict parser
before use. All three targets are installed in `CreationProfile.seen_bosses[0..3]` before
construction, using the existing unseen-boss complements (`Automaton` is the history
string for BronzeAutomaton). In `create_run` (`crates/spire-env/src/run_session.rs`), M24
joins all three complement matches: the Act1 match next to M23, the Act2 match next to
M23 (and the profile list guarding it), and the Act3 match next to M14, M15, M19 and M20.
Joining the Act3 match also keeps M24 out of the legacy `else` branch, which would
otherwise overwrite `seen_bosses[1]`. Icons and boss orders are never edited after
creation. All 27 typed triples are valid configurations.

`new_m24` admits only Watcher A0 with a complete M24 triple. It rejects the M22 and M23
histories, other profiles' histories, partial, duplicate-key, extra-key, wrong-type and
unknown values before Run initialization. Old wire values, M9 aliases and all existing
profiles are unchanged. New enum variants may require new arms in existing exhaustive
matches, including the two frozen M10 test matches that M20 and M23 already extended;
those are mechanical additions with no changed assertion.

The descriptor `run_schema_m24.rs` composes the accepted M23 Watcher schema with the
accepted Act3 vocabulary already used by `run_schema_m19.rs`: the 16 Act3 setups, 17 new
actors, their intents and powers (including slow, intangible, time warp, curiosity,
regenerate, unawakened, artifact, malleable, reactive, shifting, fading, explosive,
thorns-type retaliation, constricted, regrow and life link as actually named in the
accepted owners), Void, Parasite, Red Mask, the Falling and Tomb events and their choice
kinds, and the `Act3BossComplete`, `Act3Ending` and `Act4Ready` views with
`Act3Continue` choices. It does not copy a row encoder or a vocabulary table: producer
and vocabulary metadata project from the single content and setup owners. The catalog
distinguishes obtainable 29 cards, 19 relics and 7 potions, generated-only 3, statuses
5, 40 playable encounters and 54 enemy identities.

`m24_run_observation_spec(&RunCapacities)` and
`encode_run_m24(&PublicDecision, Option<&PublicMap>, &RunCapacities)` return the
existing result types through `encode_run_with_schema`. The public projection keeps every
M23 rule: current stance (Neutral 0, Wrath 1, Calm 2, Divinity 3), current Mantra versus
gained history, installed current powers, current combat and turn prices, Retain state,
played-power and held-card custody, entitled Scry reveal order and selection, and
nonphysical stance choices. Act3 adds actual half-dead and summoned actor rows, enemy
Artifact and Intangible amounts, Slow and Time Warp counters as installed powers, and
the Parasite curse row. Candidate *i* stays row *i*; capacity is never met by filtering,
truncation or paging; a too-small capacity is an explicit structured error. No hidden
draw order, RNG, AI state, family handle or resolver cursor enters the observation.

Replay, record, session lifecycle and batch keep their existing owners and M23 rules:
strict raw-JSON parsing that rejects duplicate and unknown keys at every new nesting
level; typed and raw validation sharing one path; implicit ordinal initialized directly;
semantic actions matched against current candidates; core faults reproduced once at their
checkpoint; whole-array batch preflight before dispatch; indexed `delivery_lost` for
successful peers after a post-dispatch fault; workers 1 and 2 matching single sessions in
input-slot order; full reset only. Coexistence is tested with separate homogeneous
batches and interleaved single sessions; the scheduler is not copied.

## 11. Evidence

Every result is reported at one of these grades, and the grades are never merged:

| Grade | Meaning |
| --- | --- |
| **Normal played** | Canonical start, frozen public caller, real progression. A policy result, never a gate. |
| **Supplied played** | Unearned resources installed by a default-off diagnostics constructor, then only legal actions through the real owners. Valid gameplay validation; never normal traversal. |
| **Constructed** | An owner checkpoint built directly (for example a constructed boss Victory record) to exercise a specific owner. Proves that owner's law, not that play reached it. |
| **Native** | Rust session, replay and batch agreement on recorded actions. Not Python delivery. |

### Canonical normal runs (normal played)

Six cases, each run exactly once with budget 32768 accepted actions and the frozen caller:

| Seed | Act1 | Act2 | Act3 |
| --- | --- | --- | --- |
| 7 | Guardian | Champ | AwakenedOne |
| 7 | Hexaghost | BronzeAutomaton | TimeEater |
| 7 | SlimeBoss | Collector | DonuAndDeca |
| 11 | Guardian | BronzeAutomaton | DonuAndDeca |
| 11 | Hexaghost | Collector | AwakenedOne |
| 11 | SlimeBoss | Champ | TimeEater |

These extend M23's six pairs with M19's Act3 targets. The caller is the M23 caller
(`examples/m23_support/policy.rs`, which delegates to the frozen M22 policy) extended
so the shared rollout recognizes the M24 endpoints, plus one legality rule. The M22
policy's existing default room rule already makes a deterministic choice on Falling,
Tomb, `Act3BossComplete` and `Act3Ending`, and the shared route helper already routes
Act3 maps. No Act3-specific strategy is added, and keys are whatever that policy earns.

**Campfire legality rule.** The M22 policy rests only below half HP and otherwise
chooses Smith, then Proceed. A campfire offers only Rest, Smith and Recall
(`crates/spire-core/src/run/rooms/rest.rs`), and Smith is disabled when no card can be
upgraded (`run/rooms/mod.rs`). At half HP or more with nothing to upgrade, the M22 policy
therefore has no prescribed action and returns an error, even though Rest (and Recall,
without the Ruby key) are legal. The M24 policy wrapper
(`examples/m24_support/policy.rs`) adds exactly one rule before delegating: at a
Campfire where Smith is not offered and HP is at least half, choose Rest, which is always
offered. This only replaces a caller error with a legal action; every decision where the
M23 policy already acts is unchanged. The frozen M22 and M23 policy files are not edited.

Because the Act1–Act2 prefix is identical to M23 under the same actions, the expected
outcome is that each case reproduces its M23 endpoint: Deaths in Act1 for five cases and
in Act2 at floor 23 for seed 7 SlimeBoss→Collector. Native M23 and native M24 differ
only in the Act3 history input, which first matters at the `(2,3,34)` transition that no
M23 case reached. If that holds, it is an exact native-level prefix witness. If any case
differs before floor 34, that is a defect to investigate, not a policy result. Whatever
happens is recorded; reaching Act3, an Act3 boss or an ending is not required.

Controls: the nonattacking seed 7 Guardian/Champ/AwakenedOne control; budgets 0 and 1
(unfinished, nonterminal); and input errors for an unknown scenario, budget 32769, a
malformed budget and unsupported seed 13, each rejected before a Run is built.

### Supplied Act3 traversal (supplied played)

To exercise Act3 progression in play even though normal runs die earlier, three supplied
cases (one per Act3 boss, seed 7, budget 32768) start at a **constructed Act2 boss
source**, as M23's `m23-diagnostics` stages do: a completed `(1,2,17)` predecessor frame,
an Act2 boss Victory record, and the Act2 boss card reward. M24 reuses M23's constructor
rather than copying it: the private helpers in `crates/spire-core/src/run/diagnostics/m23.rs`
(`canonical`, `creation` and `boss_source`) take the scenario and the resources as
parameters. M23's own call sites pass `RoomScenario::M23WatcherAct2` and canonical
resources, so M23's stages and their existing test are unchanged byte for byte. From there only the frozen
caller and legal actions advance the Run: card reward, boss treasure, boss relic menu,
the actual `(2,3,34)` transition, the generated Act3 map, every Act3 room the route
visits, and the Act3 boss and ending. The Act2 boss record is constructed provenance; the
rest is played. The loadout is the Act3 loadout below, installed before the first
published decision.

### Supplied Act3 bosses (supplied played)

Six cases (seeds 7 and 11, each Act3 boss once) start one map edge before the actual
floor-50 boss, after a constructed completed `(2,3,34)` predecessor frame and Act3
progress, at the floor-49 rest site, as M14's approved supplied-boss amendment did. A
real Move enters the full-HP boss; only the frozen caller and legal actions follow.
Seed 7 cases hold no keys, so a Victory exercises the played path to `Act3NoKeys`. Seed
11 cases hold all three keys (supplied, unearned), so a Victory exercises the played path
to `Act4Ready`. Budget 32768 accepted actions.

**Act3 loadout v2** (used by traversal and boss cases), frozen before any execution:
Watcher A0, 100/100 HP, 99 gold, all cards upgraded once, permanent IDs 1..18 in this
order: MentalFortress×2, LikeWater, BattleHymn, Devotion, Eruption×3, EmptyFist×3,
CarveReality, WindmillStrike, Vigilance×3, Prostrate, ThirdEye. Relics in this order:
PureWater, VioletLotus, TeardropLocket, GoldenEye, Vajra, OddlySmoothStone. Potion
slots: BlockPotion, DexterityPotion, AncientPotion. Supplied resources are validated by
the existing `M21WatcherCombatStart::Supplied(..).normalized()`, which admits only the M21
Watcher relics (`crates/spire-core/src/run/training.rs`, `M21_WATCHER_RELICS`). Version 1
held Anchor, Lantern and BloodVial and would have failed that check; version 2 uses only
M21-admitted relics and potions, and every item is also in the M24 finite union. The
validation owner is not changed. Because the deck starts fully upgraded, Smith is
disabled at campfires until a reward adds a base card; the campfire legality rule above
makes that a Rest rather than a caller error. The five Powers deliberately exercise Curiosity; ThirdEye exercises
Scry against Act3 statuses. The loadout is chosen for mechanic coverage and survivability;
it is not tuned after any outcome. The high HP is an unearned supplied resource and is
labelled as such.

### Supplied Act3 interaction fixtures (supplied played)

Nine fixtures, each seed 7, 72/72 HP, 99 gold, three empty potion slots, no keys, budget
128 accepted combat actions, starting one edge before the real captured room (the capture history and boss icon are
set so `act3::capture_room` selects the named encounter):

| Fixture / encounter | Deck (base unless +) and relics | Checkpoints observed if reached |
| --- | --- | --- |
| Slow / GiantHead | FlurryOfBlows, Eruption, WindmillStrike, Vigilance, EmptyFist; PureWater | W9 |
| Intangible / Nemesis | FlurryOfBlows, Eruption+, CarveReality, ThirdEye, Vigilance; PureWater, VioletLotus | W10, W14 |
| Daggers / Reptomancer | EmptyFist, StrikePurple, StrikePurple, Eruption, Vigilance; PureWater | W13 |
| Implant / WrithingMass | FlurryOfBlows, WindmillStrike, Meditate, Vigilance, Eruption; PureWater | W12 |
| HalfDead / ThreeDarklings | Eruption+, EmptyFist, StrikePurple, StrikePurple, Vigilance; PureWater | W13 |
| Burn / OrbWalker | ThirdEye+, Foresight, Eruption, Vigilance, Weave; PureWater | W14 |
| Constrict / SpireGrowth | Eruption, Vigilance, Prostrate, LikeWater, StrikePurple; PureWater | W11 |
| Thorns / FourShapes | FlurryOfBlows, WindmillStrike, Eruption, Vigilance, EmptyFist; PureWater | W11 |
| Fading / Transient | Eruption+, WindmillStrike, Vigilance, Perseverance, Prostrate; PureWater | W13 |

Each uses the frozen M23 caller's combat rules unchanged: use a potion in ascending slot
order when offered, choose Calm at a stance choice, confirm Scry and Meditate when
confirm is offered (otherwise take the first legal selection), and otherwise play the
first legal card, then EndTurn. Elites are selected through the boss icon's elite offset
(AwakenedOne → GiantHead, TimeEater → Nemesis, DonuAndDeca → Reptomancer at elite history
length 0). A fixture may die or miss a checkpoint; an unobserved
checkpoint is reported as missing played proof and remains pinned by the combat
composition tests. No draws, AI tickets or loadouts are searched.

### Constructed diagnostics

A default-off `m24-diagnostics` Cargo feature in `crates/spire-core` (depending on
`m23-diagnostics`, as that one depends on `m22-diagnostics`) provides typed constructed
stages for core and native tests. It contains no public supplied-run API:

- an Act3 boss source with a constructed Victory record for each boss and each of the
  eight key masks, which runs the real ending to `Act3NoKeys` or `Act4Ready`;
- ending faults: a source that changes after an accepted dialog choice, and a failed
  `Act3NoKeys` terminal commit, each retaining the first fault under `run.m24.act3.end`
  and committing nothing twice;
- the M24 `(2,3,34)` continuation from a constructed Act2 boss source to the Act3 `Map`;
- Falling, Tomb (with and without a mask) and the floor-43 chest, for capture and fault
  tests;
- Act3 Scry, Meditate and stance choices for semantic replay tests.

### Freeze gate

Nothing is frozen until the source is final. The freeze requires all of the following on
the exact tree being frozen: every RED reported by the Combat lane is resolved, with its
global fix and disclosure written; the Native lane's delivery is integrated; and the full
`m24_` core and environment suites plus the M21, M22 and M23 regression suites pass. A
fix that is needed after the freeze requires a new, separately recorded freeze; the
earlier outcomes are kept as history and are not rerun to replace them.

### Native agreement and preservation

The six canonical transcripts, the supplied traversal and boss transcripts and the
interaction-fixture transcripts are fed to native single sessions, strict JSON replay
and homogeneous batches with workers 1 and 2, using the recorded semantic actions.
Supplied and constructed starts reach native lanes only through the default-off
diagnostics constructors, never through a public API. Coexistence interleaves M24, M23
and one accepted Silent and one accepted Defect profile for 32 accepted actions each,
crossing at least one combat boundary.

One stable preservation run closes the milestone:
`cargo test --workspace --locked --offline --no-fail-fast`, plus the focused M24 core,
environment and integration targets and the changed example invocations. The two
scope-ignored strategy gates (M18, M19) and the inherited holds stay as they are.
Python delivery, original-runtime parity and Java RNG/map parity are not claimed.

## 12. Limits

M24 does not claim: Act4, the Act4 door, Shield/Spear, the Heart; A1–A20; full Neow;
Watcher cards, relics, potions or events outside the finite union; standard reward,
shop or event pools; a normal-start Act3 entry, boss Victory or ending; Python extension
delivery; original-runtime or Java parity; any strategy or training quality. Supplied
and constructed results are never presented as normal traversal. The finite stock can be
exhausted; that is reported, not hidden.

## 13. Final review focus

The one fresh independent final milestone review should check:

1. **Prefix and continuation.** M23/M24 equality under matched history through
   `(2,3,34)`; the only gameplay-facing difference is `Map` versus `Act3Ready`;
   `Act3Progress` initialized once; unmatched-history divergence disclosed.
2. **Finite content and combat admission.** Exactly 29/19/7 plus 3 and 5; Parasite
   gained only through the gain journal and admitted to every Watcher combat by the
   type-based construction law; Red Mask only through Tomb and admitted at combat entry
   only under `with_m24_profile()`; M22/M23 `Debug` text unchanged; Act2 and Act3
   merchant first/later from the right act counter; full global visit preserved; no new
   producer rows.
3. **Act3 capture.** `run.m24.relic.capture` used only in Act3; empty, singleton and
   saturated pools; no reroll on inspection, decline, stale input, replay or fault.
4. **Watcher × Act3 combat.** W1–W14 against real setups and cited source facts; any
   shared-owner fix applied globally with disclosed amendments.
5. **Ending.** Boss Victory → Proceed → dialogs → `Act3NoKeys` or `Act4Ready` across all
   eight key masks; no boss loot; Death, EmptyDeck, budget and fault kept distinct; the
   `UnsupportedBoundary` cutoff mutates nothing.
6. **Native contract.** Strict history and identity validation before construction;
   replay, batch preflight, faults, `delivery_lost`, workers 1/2 agreement, coexistence;
   old descriptors and fingerprints unchanged.
7. **Evidence honesty.** Normal, supplied, constructed and native grades reported
   separately; once-only execution with a freeze before outcomes; every loss retained.

## 14. Planning record

Version 1, its research reports, decisions log and hashes are retained outside the
managed session at
`/home/klottick/.local/share/spire-gym/evidence/m24-execution-20261030/planning-v1/`.
Version 2, the review disposition table and the list of interfaces changed since
version 1 are at
`/home/klottick/.local/share/spire-gym/evidence/m24-execution-20261030/planning-v2/`.
