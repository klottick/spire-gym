# M25 finite Watcher A0 Acts 1–4 and Heart design

Date: 2026-10-30. Status: **planning proposal, version 2**. Version 2 resolves the
independent plan review v1 (NOT READY, 0 Critical / 4 Important / 7 Minor, report SHA-256
`55e6bc616312c01644d3a51d2afe410fc26f5e5cff12ba61755a17db98e11230`) and applies the PM's
rulings (section 16); the disposition table is retained with the planning evidence (section
17). Planning Bead `spire-dxo.47.1`; roadmap outcome `spire-dxo.47` ("M25 — Watcher through
Act4/Heart"). Planning source: `main` at `38496f922d1b272f5454034cb3bb11110e6843fd` (M23
merged in PR62, M20 witness documents in PR63).

**Predecessor contract.** M25 is reviewed and approved against the M24 planning pair version
2 as a contract: design SHA-256
`ca0474de2a0bba7a27dc831e9533bc9822f508093e53cccc000420c62228c516`, plan SHA-256
`c98de65eb786024f0d950dc7ae59cc88f367d0aecd7e2f7c3d6da39ab279a545`, interface list SHA-256
`c72a8f29470cb08e22968398e64fe9fe9a8494261d91b6aa384a4bcc72f6a2ef`, all at
`/home/klottick/.local/share/spire-gym/evidence/m24-execution-20261030/planning-v2/`. That
pair passed its own independent review and is being implemented. M25 execution cannot start
until M24 is merged. Section 15 lists every M24 interface M25 depends on and the exact check
the plan's Task M0 makes against the merged source, including when a short amendment must go
back to the same plan reviewer. The paired [implementation
plan](../plans/2026-10-30-m25-watcher-heart.md) turns this design into tasks. This document
releases no runtime work by itself.

## 1. What M25 delivers

The [roadmap](2026-10-22-narrow-gym-milestones-design.md) defines M25 as "Real keys and
complete Heart path; ordinary character card/exclusive-item/event coverage at A0 is
complete, not only one successful stance deck." M25 therefore has two halves.

**The Run half.** M25 continues the exact M24 Run. M24 starts a Watcher A0 Run at short
Neow, plays Acts 1–3, and after an actual Act3 boss Victory with all three keys stops at the
healthy, zero-action `Act4Ready` screen (act 3, floor 50). M25 removes that stop. The same
Run offers the fourth Act3 ending dialog, goes through the shared Act4 door, and plays the
fixed Act4 route: Rest (floor 51), Shop (52), the Spire Shield and Spire Spear elite (53)
and the Corrupt Heart (54). After an actual Heart Victory the player accepts the offered
Proceed, which enters floor 55 (`TrueVictoryRoom`) and commits the `HeartVictory` terminal.
A Run ends in exactly one of these ways:

- `HeartVictory` after the final Proceed;
- `Act3NoKeys` after an Act3 boss Victory without all three keys (unchanged from M24);
- a genuine Death or EmptyDeck at any point.

M25 has no cutoff screen. Keys are earned only through the shared key owners: Recall at a
campfire (Ruby), the Sapphire choice at a non-boss chest, and a burning-elite Victory
(Emerald).

**The content half.** M25 completes the Watcher character at A0:

- all 75 ordinary Watcher cards in base and once-upgraded form (28 already accepted in
  M21–M24, 47 new);
- all Watcher generated children (Miracle, Smite, Safety already accepted; Insight, Through
  Violence, Beta, Omega and Expunger new) and Wish's three choice-only options;
- the complete set of cards Foreign Influence can create: every ordinary, non-healing Attack
  of any colour, 106 kinds, 82 of them from other characters or colourless;
- all nine Watcher-exclusive relics (four new) and all three Watcher-exclusive potions
  (already accepted);
- every event the M25 profile can reach at A0, exercised with Watcher semantics.

Every item is implemented in the shared engine to its source law and exercised by
meaningful tests. Supplied and constructed setups are valid ways to exercise it. A policy
win with these cards is not required.

## 2. Rules that shape this design

These are binding user rules, restated so the builder does not need to look them up.

1. **The repository plays the game; it does not need to win it.** The primary scope
   clarification of 2026-10-30 says winning with this code is not in scope and a supplied
   victory is fine. M25 has no normal-start win gate. "Complete coverage" means every
   ordinary Watcher card (base and upgraded), every Watcher-exclusive relic and potion, and
   every event the profile can reach at A0 is implemented to source law and exercised by
   meaningful tests. Supplied fixtures are acceptable. A policy winning with them is not
   the criterion.
2. **Fix real bugs globally.** A defect found in a shared owner is fixed at the shared rule
   for every character. An accepted assertion that pinned wrong behaviour is amended, the
   change is disclosed, and the original bytes are retained. No narrow Watcher-only guard
   may hide a shared defect. This applies to all three lanes.
3. **One engine.** Extend the existing Run, rooms, combat, resolver, generation, offers,
   capture, encoder, session, replay, batch, rollout and caller owners and the default-off
   diagnostics features. No copied loops, tables, encoders or validators. No public
   supplied-run or import API.
4. **No installs.** No new dependencies, no Python rebuild, no original-game execution.

## 3. Approaches considered

| Approach | Result | Decision |
| --- | --- | --- |
| Continue the exact M24 Run (same Acts 1–3 producers), join the shared Act4 owners, implement the complete Watcher roster and its dependencies in the shared engine, and let the new Act4 producers (Shield/Spear card reward, Act4 merchant, Act4 elite relic) draw from the complete roster | Keeps the exact M24 prefix the PM asked for. Every ordinary Watcher card and every exclusive relic gets a real normal acquisition path in the profile. Act4 needs new producer laws in any case, because the Watcher merchant has no Act4 arm today (`crates/spire-core/src/run/rooms/shop.rs` reads Act1 progress, which the door clears). | **Selected.** |
| Replace the finite Acts 1–3 offers with complete-roster captures from the first floor, as M15 and M20 did | Every card is obtainable early, but the M24 prefix is lost after the first differing offer, which contradicts the predecessor contract. | Rejected for M25. Recorded as open question 1 for the PM. |
| Continue M24 exactly and keep the M22 rows in Act4 too, making the new content reachable only through supplied setups | Smallest Run change, but 47 ordinary cards and 4 exclusive relics would have no acquisition path in the profile that claims complete coverage, and the normal encoder would need a separate diagnostic vocabulary for them. | Rejected. |
| Restrict Foreign Influence to Watcher Attacks, or leave it out | Breaks its source law or leaves an ordinary Watcher card uncovered. | Rejected. Its full output closure is in scope (section 5.3). |
| Implement the full standard reward, shop, event, Neow and potion-factory pools now | Absorbs M26–M28. | Out of scope. |

## 4. Start, the exact M24 prefix and matched history

### 4.1 Start

Add `RoomScenario::M25WatcherHeart` to the existing
`Run::new_room_scenario(RunConfig, CreationProfile, RunRng, RoomScenario)` in
`crates/spire-core/src/run/rooms/mod.rs`. It admits Watcher, ascension 0, short Neow only;
anything else returns `RunError::Unsupported("M25 requires Watcher A0 short Neow")` before
any mutation. A profile that is not fully unlocked, or has Act 4 disabled, is rejected by
the existing `Run::new` check with its existing message, `"profile must be fully unlocked
with Act 4 enabled"`, exactly as for M22–M24. Canonical resources come from the existing
`Run::new`/Catalog path: 72/72 HP, 99 gold, four StrikePurple, four DefendPurple, Eruption
and Vigilance as ten distinct base permanent instances, PureWater, three empty potion
slots, no keys, Act 4 enabled. No resource is supplied through the normal profile.

### 4.2 Prefix equality with M24

M25 must not replay, copy or regenerate anything M24 does. Acts 1–3 run through the same
shared owners with the same producers, captures and RNG scopes as M24.

- Construct M24 and M25 through the core constructor with the **same** `CreationProfile`,
  seed, partition and ordinal, and drive the same recorded semantic actions in both.
- Through Act1, `(1,2,17)`, Act2, the Act2 boss reward and treasure, `(2,3,34)`, Act3, the
  Act3 boss Victory, `Act3BossComplete` and the first three `Act3Ending` dialog stages, the
  two Runs agree on HP, max HP, gold, permanent cards (identities, order, upgrades, misc),
  relics and counters, potions, keys, Neow state, combat and room records, captures, RNG
  bytes and counters, transition flags, maps and boss orders. Emitted gameplay and reward
  events and public choices match in order.
- The only allowed differences are metadata: scenario tag, profile, content, rules,
  schema and replay identifiers, descriptor and fingerprint, profile numeric code,
  transport session identifiers and tokens, additive vocabulary indices and fault
  checkpoint labels. Encoded references are compared by semantic identity.
- **The first gameplay-facing difference.** After the accepted `Act3Continue{stage:3}`
  with Act 4 enabled and all three keys, M24 (which by its contract does not join
  `uses_act4`; M24 design version 2 §8) publishes `Act4Ready` with zero actions,
  while M25 publishes the fourth `Act3Ending` stage with its single `Act3Continue{stage:4}`
  choice, whose acceptance is the Act4 door. Without all three keys both profiles offer
  stage 4 and commit `Act3NoKeys` identically
  (`crates/spire-core/src/run/rooms/act3_end.rs`, the `Dialog(3)` arm).

**Matched future history.** Act 4 has no boss choice: the fixed Act4 map's boss icon is
always the Corrupt Heart (`crates/spire-core/src/run/rooms/act4.rs`, `fixed_map`). The
native M25 history therefore has exactly the same three typed targets as M24. Unlike M23
against M22 or M24 against M23, native M24 and native M25 Runs built from the same triple
are expected to agree exactly up to the first gameplay-facing difference above. There is no
unmatched-history caveat to disclose.

**How the prefix is proved.** Four checks, each at its own grade:

1. A matched-history **played** prefix in core: build M24 and M25 with the same inputs,
   take each action from the M24 Run's own legal candidates by one fixed test rule, apply
   the same action to M25, and compare the state listed above after every step until M24
   reaches a terminal, `Act4Ready` or the step limit. How far it gets is reported, not
   required.
2. A matched **owner-level** Act3 check: from equal constructed Act3 states one map edge
   before a first and a later Act3 merchant, an Act3 elite, the floor-43 chest, Falling and
   Tomb, really enter each room in both profiles. Rows, captures and RNG must be equal.
3. A **constructed ending** check: for each boss and each of the eight key masks, build an
   M24 source and an M25 source with M25's own Act3 boss Victory source constructor (section
   12.7), which takes the scenario as an input, assert the two sources are equal apart from
   metadata, and run the real ending in both profiles. They agree through stage 3; with all
   keys M24 shows `Act4Ready` and M25 shows stage 4; every other mask commits `Act3NoKeys`
   identically.
4. The six canonical normal runs (section 12.3), compared step by step with the six M24
   canonical cases run once as a labelled comparison control on the same frozen tree, and
   then with M24's retained receipts. Against the control, any difference before Act 3 is a
   defect. Against the retained receipts, every difference must trace to a disclosed global
   fix, named with its first differing step; an unexplained difference is a defect. The
   control never replaces M24's retained evidence.

### 4.3 Joining the shared Act4 predicate

M25 joins `uses_full_act3` (which also places it in `uses_captured_act3`,
`uses_captured_act2`, `uses_act2_boss` and `uses_captured_act1`), `uses_watcher_content`
and `uses_act4` in `crates/spire-core/src/run/rooms/mod.rs`. `uses_act4` has call sites that
run before the door. Each was read at the planning source; for Watcher content each is
either guarded by `act == 4` or has no effect. Task M0 repeats this check on the merged M24
source and fails the plan if anything changed.

| Pre-door `uses_act4` site | Effect for Watcher content |
| --- | --- |
| `act1.rs`, `act2.rs`, `act3.rs`: commit the shop's cloned RNG after preparing a merchant | No effect. The Watcher merchant rows are fixed and draw no RNG (`shop.rs`, Watcher branch), so the committed clone equals the original. |
| `shop.rs`: use the current node instead of node 0 in the shop key | No effect. The Watcher branch ignores the key. |
| `rewards.rs`: the `uses_act4` chest branch | Never reached. The Watcher chest branch is checked first and returns. |
| `rest.rs`: store Recall as Disabled when the Ruby key is held, and reject choosing it | Public behaviour identical. The shared public projection already disables Recall whenever Ruby is held, for every profile (`rooms/mod.rs`, `public`), and choosing it fails in both profiles because `mutations::acquire_key` rejects a held key. The only difference is the stored, unpublished status of that row. It is not in the compared state of section 4.2, which compares public choices; the prefix test compares the room screen through `public()`. |
| `act3_end.rs`: stage 3 leads to stage 4, and stage 4 with all keys opens the door through `act4::prepare_transition` and `act4::commit_transition`, which run while the Run is still in act 3 | This is the intended difference. |
| `rooms/mod.rs` move and finish routing, `run/combat.rs` Act4 visit counter and Act4 admission, and every other `act4.rs` function (room entry, combat source, Heart Victory and Proceed, room finish) | Guarded by `act == 4`. |

### 4.4 M24-only branches M25 must join

The M24 plan adds branches that name M24 alone. M25 must take the same branch wherever it
affects Acts 1–3, or the prefix would drift. At the planning source these are the M24 plan's
own additions (interface list version 2):

| M24 branch | M25 arm |
| --- | --- |
| `shop.rs` Watcher merchant: Act2 counter for M23 or M24 in Act2; Act3 counter for M24 in Act3 | Same counters for M25 in Act2 and Act3. Act4 gets its own law (section 6.2). |
| `m23_content::relic_capture_scope`: `run.m23.relic.capture` in Act2, `run.m24.relic.capture` in Act3 | Same scopes for M25 in Acts 1–3. Only Act4 uses the new `run.m25.relic.capture`. |
| `mutations::supported_in` admits Parasite for M24 | M25 admits its complete permanent union, which includes Parasite (section 5.6). |
| `run/combat.rs` selects `CombatItemContext::with_m24_profile()` for M24 | M25 selects its own `with_m25_profile()` (section 5.7). |
| `act3.rs` and `act3_end.rs` fault labels `run.m24.room`, `run.m24.encounter`, `run.m24.act3.end` | M25 labels `run.m25.room`, `run.m25.encounter`, `run.m25.act3.end` (metadata). |
| `create_run` complement arms for Acts 1, 2 and 3 | M25 joins all three, the same way. |
| `check_boundary` M24 `Act4Ready` arm and `at_m14_cutoff` | M25 joins neither; it has no cutoff. |

Task M0 lists every `is_m24` branch in the merged source and records the M25 decision for
each. A branch not in this table that affects Acts 1–3 gets the M24 behaviour for M25.

## 5. Complete Watcher A0 content

Values below come from the source-graded
[Watcher research](../../research/2026-10-19-act1-content-watcher.md) §§2–7, the
[card reference](../../../data/cards/README.md) and its
`data/cards/implementation-notes.json`, read together with the native owners. The planning
research reports that mapped them to the native source are retained with the planning
evidence (section 17). They are implementation inputs, not original-runtime validation.

### 5.1 Ordinary cards

The Watcher has 75 ordinary cards: 4 Basic, 19 Common, 35 Uncommon and 17 Rare, by type 27
Attacks, 36 Skills and 12 Powers. Neither Discipline nor Unraveling is a card in the
reference roster. Basics are not reward cards.

**Accepted in M21–M24 (28).** StrikePurple, DefendPurple, Eruption and Vigilance (Basic);
EmptyFist, FlurryOfBlows, Prostrate, ThirdEye, CutThroughFate and JustLucky (Common);
Worship, Weave, Foresight, Nirvana, WindmillStrike, Perseverance, SandsOfTime, Meditate,
CarveReality, DeceiveReality, BattleHymn, MentalFortress, Rushdown and LikeWater
(Uncommon); Brilliance, Devotion, Establishment and MasterReality (Rare). Their behaviour
is accepted (`crates/spire-core/src/content/watcher.rs`, `combat/cards/watcher.rs`). M25
keeps it and adds composition tests where new content touches it.

**New in M25 (47).** Three of them, Consecrate, EmptyBody and EmptyMind, already have a
native ID and a generic handler with tests (`combat/cards/mod.rs`,
`combat/tests/full_cards.rs`) but are outside every Watcher profile. M25 admits them and
verifies the existing payload against the values below; a mismatch is a shared defect fixed
under rule 2. The other 44 have no native ID yet.

Costs and values are base → upgraded. R = Retain, E = Exhaust, I = Innate, Eth = Ethereal,
X = X-cost.

| Common (13) | Type | Cost | Effect, base → upgraded | Keywords |
| --- | --- | --- | --- | --- |
| BowlingBash | Attack | 1 | 7 → 10 damage to the target, once for each enemy that is not dead or escaped when counted (separate hits) | — |
| Consecrate | Attack | 0 | 5 → 8 damage to all enemies | — |
| Crescendo | Skill | 1 → 0 | Enter Wrath | R, E |
| CrushJoints | Attack | 1 | 8 → 10 damage; if the previous card played was a Skill, 1 → 2 Vulnerable | — |
| EmptyBody | Skill | 1 | 7 → 10 Block, then exit the stance | — |
| Evaluate | Skill | 1 | 6 → 10 Block; shuffle an Insight into the draw pile | — |
| FlyingSleeves | Attack | 1 | 4 → 6 damage twice | R |
| FollowUp | Attack | 1 | 7 → 11 damage; if the previous card played was an Attack, gain 1 energy | — |
| Halt | Skill | 0 | 3 → 4 Block; in Wrath an extra 9 → 14, modified separately, in one gain | — |
| PressurePoints | Skill | 1 | Apply 8 → 11 Mark to the target, then every enemy loses HP equal to its Mark | — |
| Protect | Skill | 2 | 12 → 16 Block | R |
| SashWhip | Attack | 1 | 8 → 10 damage; if the previous card played was an Attack, 1 → 2 Weak | — |
| Tranquility | Skill | 1 → 0 | Enter Calm | R, E |

| Uncommon (21) | Type | Cost | Effect, base → upgraded | Keywords |
| --- | --- | --- | --- | --- |
| Collect | Skill | X | At the start of each of the next X → X+1 turns, add an upgraded Miracle to hand | X, E |
| Conclude | Attack | 1 | 12 → 16 damage to all enemies, then end the turn | — |
| EmptyMind | Skill | 1 | Draw 2 → 3, then exit the stance | — |
| Fasting | Power | 2 | Gain 3 → 4 Strength and Dexterity; gain 1 less energy each turn (Energy Down 1, a debuff that player Artifact can block) | — |
| FearNoEvil | Attack | 1 | 8 → 11 damage; if the target intends to attack, enter Calm | — |
| ForeignInfluence | Skill | 0 | Choose 1 of 3 Attacks of any colour to add to hand; upgraded, it costs 0 this turn | E |
| Indignation | Skill | 1 | In Wrath, 3 → 5 Vulnerable to all enemies; otherwise enter Wrath | — |
| InnerPeace | Skill | 1 | In Calm, draw 3 → 4; otherwise enter Calm | — |
| Pray | Skill | 1 | Gain 3 → 4 Mantra; shuffle an Insight into the draw pile | — |
| ReachHeaven | Attack | 2 | 10 → 15 damage; shuffle a Through Violence into the draw pile | — |
| Sanctity | Skill | 1 | 6 → 9 Block; if the previous card played was a Skill, draw 2 | — |
| SignatureMove | Attack | 2 | 30 → 40 damage; playable only if it is the only Attack in hand | — |
| SimmeringFury | Skill | 1 | At the start of the next turn, enter Wrath and draw 2 → 3 | — |
| Study | Power | 2 → 1 | At the end of each turn, shuffle an Insight into the draw pile | — |
| Swivel | Skill | 2 | 8 → 11 Block; the next Attack played costs 0 | — |
| TalkToTheHand | Attack | 1 | 5 → 7 damage; then installs Block Return 2 → 3 on the enemy (a debuff Artifact can block): whenever the player attacks it later, gain that much Block. The card's own hit does not return Block | E |
| Tantrum | Attack | 1 | 3 damage 3 → 4 times; enter Wrath; shuffle this card into the draw pile | — |
| Wallop | Attack | 2 | 9 → 12 damage; gain Block equal to the unblocked damage dealt | — |
| WaveOfTheHand | Skill | 1 | This turn, whenever the player gains a positive amount of Block, apply 1 → 2 Weak to all enemies; the power is removed at the end of the round | — |
| WheelKick | Attack | 2 | 15 → 20 damage; draw 2 | — |
| WreathOfFlame | Skill | 1 | Gain 5 → 8 Vigor: added to NORMAL damage through the damage-give hook and removed by the next Attack's use callback | — |

| Rare (13) | Type | Cost | Effect, base → upgraded | Keywords |
| --- | --- | --- | --- | --- |
| Alpha | Skill | 1 | Shuffle a Beta into the draw pile | E, upgraded I |
| Blasphemy | Skill | 1 | Enter Divinity; at the start of the next turn, lose 99,999 HP | E, upgraded R |
| ConjureBlade | Skill | X | Shuffle an Expunger with X → X+1 hits into the draw pile | X, E |
| DeusExMachina | Skill | Unplayable | When drawn, add 2 → 3 Miracles to hand, then exhaust this card | E |
| DevaForm | Power | 3 | At the start of each turn gain energy, starting at 1 and growing by 1 each turn | base Eth |
| Judgment | Skill | 1 | If the target monster has 30 → 40 or less HP, request an Instant Kill of it | — |
| LessonLearned | Attack | 2 | 10 → 13 damage; if fatal, upgrade a random eligible card in the permanent deck | E |
| Omniscience | Skill | 4 → 3 | Choose a card in the draw pile; play it twice, then exhaust it | E |
| Ragnarok | Attack | 3 | 5 → 6 damage to a random enemy, 5 → 6 times | — |
| Scrawl | Skill | 1 → 0 | Draw until the hand has 10 cards | E |
| SpiritShield | Skill | 2 | Gain 3 → 4 Block for each other card in hand | — |
| Vault | Skill | 3 → 2 | Take an extra turn after this one: end the turn and skip the enemy turn | E |
| Wish | Skill | 3 | Choose one: 3 → 4 Strength, 25 → 30 Gold, or 6 → 8 Plated Armor | E |

### 5.2 Watcher generated children and Wish options

| Card | Cost | Effect, base → upgraded | Produced by | Status |
| --- | --- | --- | --- | --- |
| Miracle | 0 | Gain 1 → 2 energy; R, E | PureWater, HolyWater, BottledMiracle, DeusExMachina, Collect (upgraded) | Accepted |
| Smite | 1 | 12 → 16 damage; R, E | CarveReality, BattleHymn | Accepted |
| Safety | 1 | 12 → 16 Block; R, E | DeceiveReality | Accepted |
| Insight | 0 | Draw 2 → 3; R, E | Evaluate, Pray, Study | New |
| ThroughViolence | 0 | 20 → 30 damage; R, E | ReachHeaven | New |
| Beta | 2 → 1 | Shuffle a base Omega into the draw pile; E | Alpha (a base Beta) | New |
| Omega | 3 | Power: at the end of each turn, 50 → 60 non-attack damage to all enemies, not modified by Strength, Vulnerable or stance | Beta | New |
| Expunger | 1 | 9 → 15 damage, repeated for its stored hit count (zero is possible) | ConjureBlade | New |

Wish's options, BecomeAlmighty (3 → 4 Strength), FameAndFortune (25 → 30 gold) and
LiveForever (6 → 8 Plated Armor), are choice-only. They are never physical cards, never
enter a pile and are never permanent. Their upgrade state comes from Wish, and Master
Reality does not upgrade them (`data/cards/implementation-notes.json`, WISH and the three
option entries). Choosing one applies its effect through the shared owners: Strength and
Plated Armor as powers, gold as a permanent Run gain (section 5.8).

### 5.3 The Foreign Influence closure

Foreign Influence is an explicit any-colour exception
([Watcher research](../../research/2026-10-19-act1-content-watcher.md) §7.1;
[complete RNG maps](../../research/2026-10-19-act1-complete-rng-maps.md), Foreign
Influence row; [native contract](../../research/2026-10-19-act1-native-contract.md) "F,
concrete kernels"):

- Each of the three proposals first rolls 0–99 with the combat card-random stream: 0–54
  Common, 55–84 Uncommon, 85–99 Rare. It then picks a kind uniformly in that tier. A
  duplicate kind retries the **whole** proposal, including the rarity roll. The support
  excludes Basic and Special rarity, Curses, Statuses and HEALING-tagged cards, and has no
  colour predicate, so colourless Attacks are included.
- The player selects zero or one of the three offered kinds. The chosen kind becomes a
  fresh stat-equivalent copy in hand, or in the discard pile when the hand holds 10 cards.
  Upgraded Foreign Influence sets that copy's turn cost to 0. Master Reality upgrades the
  copy once when it enters the hand, and not when it overflows to discard.
- RNG-001 applies (`docs/intentional-deviations.md`): the conditional probabilities and the
  whole-proposal retry must hold; Java call-for-call parity is not required.

The support has 43 Common, 48 Uncommon and 15 Rare kinds, 106 in total. The HEALING-tagged
Attacks Feed, Reaper and LessonLearned are excluded; Hand of Greed is not tagged, so it is
included ([red and colourless research](../../research/2026-10-19-act1-content-red-colorless.md)
§6; Watcher research rows for Lesson Learned and Wish).

**One card-tag owner.** The native catalog already has `CardTag::{Strike, StarterStrike,
StarterDefend, Healing}` (`content/cards.rs`), but only inside `BlueCardMetadata`, so tags
exist for Defect cards only; the colourless generator excludes Bandage Up by an explicit list
and Perfected Strike uses a separate hard-coded list (below). Task C1 adds one general owner,
proposed `content::cards::card_tags(CardId) -> &'static [CardTag]`, that returns the existing
`BlueCardMetadata` tags unchanged for Defect cards and the cited tags for every other kind:
`Healing` for exactly Feed, Reaper, Alchemize, SelfRepair, LessonLearned, Wish and BandageUp
([native contract research](../../research/2026-10-19-act1-native-contract.md), HEALING
exclusions list), and `Strike` for exactly the source STRIKE set in the next paragraph. The
Foreign Influence filter and Perfected Strike both read this owner, and a test checks that it
agrees with every `BlueCardMetadata` row. No existing generator or pool changes. Derived from
the cited reference partitions:

| Colour | Common | Uncommon | Rare | Kinds |
| --- | --- | --- | --- | --- |
| Watcher | BowlingBash, Consecrate, CrushJoints, CutThroughFate, EmptyFist, FlurryOfBlows, FlyingSleeves, FollowUp, JustLucky, SashWhip | CarveReality, Conclude, FearNoEvil, ReachHeaven, SandsOfTime, SignatureMove, TalkToTheHand, Tantrum, Wallop, Weave, WheelKick, WindmillStrike | Brilliance, Ragnarok | 24 |
| Ironclad | Anger, BodySlam, Clash, Cleave, Clothesline, Headbutt, HeavyBlade, IronWave, PerfectedStrike, PommelStrike, SwordBoomerang, Thunderclap, TwinStrike, WildStrike | BloodForBlood, Carnage, Dropkick, Hemokinesis, Pummel, Rampage, RecklessCharge, SearingBlow, SeverSoul, Uppercut, Whirlwind | Bludgeon, FiendFire, Immolate | 28 |
| Silent | Bane, DaggerSpray, DaggerThrow, FlyingKnee, PoisonedStab, QuickSlash, Slice, SneakyStrike, SuckerPunch | AllOutAttack, Backstab, Choke, Dash, EndlessAgony, Eviscerate, Finisher, Flechettes, HeelHook, MasterfulStab, Predator, RiddleWithHoles, Skewer | DieDieDie, GlassKnife, GrandFinale, Unload | 26 |
| Defect | BallLightning, Barrage, BeamCell, Claw, ColdSnap, CompileDriver, GoForTheEyes, Rebound, Streamline, SweepingBeam | Blizzard, Bullseye, DoomAndGloom, FTL, Melter, RipAndTear, Scrape, Sunder | AllForOne, CoreSurge, Hyperbeam, MeteorStrike, ThunderStrike | 23 |
| Colourless | — | DramaticEntrance, FlashOfSteel, MindBlast, SwiftStrike | HandOfGreed | 5 |
| **Total** | **43** | **48** | **15** | **106** |

The 82 non-Watcher kinds are already implemented natively for their own characters (M10
Ironclad, M15 Silent, M20 Defect, and the accepted 34-kind colourless set). M25 makes each
of them constructible and playable by an actual Watcher, in base and upgraded form, through
its existing owner. A character-specific guard that blocks lawful Watcher use is a shared
defect fixed under rule 2, not a reason to drop the kind. Examples the tests must cover:
Defect Attacks with zero orb slots (a channel fails, Barrage and Thunder Strike hit zero
times, Hyperbeam lowers the Watcher's Focus), Silent Attacks that read discard or
skill-in-hand history, Searing Blow receiving exactly one Master Reality upgrade on the
hand route (native contract research, "F, construction distinctions"; F15), and Perfected
Strike's STRIKE count. The outputs of these cards stay inside the closure: Anger and Endless
Agony copy themselves, and Wild Strike, Reckless Charge and Immolate create Wound, Dazed and
Burn, which are already in the status vocabulary.

**Audit depth.** Task C5 checks each of the 82 against its research row, with literal
primary values in base and upgraded form, in a Watcher play. M25 fixes every shared defect
that check exposes, for all characters, and does not otherwise re-audit behaviour that only
another character can reach. Planning already found one: `has_strike_tag`
(`crates/spire-core/src/combat/cards/ironclad.rs`) counts Thunderclap and omits WildStrike,
SwiftStrike, SneakyStrike, ThunderStrike, MeteorStrike and WindmillStrike, while the source
STRIKE set is the four basic Strikes plus Perfected, Pommel, Twin, Wild, Swift, Sneaky,
Thunder, Meteor and Windmill Strike
([red and colourless research](../../research/2026-10-19-act1-content-red-colorless.md),
Perfected Strike paragraph). M25 corrects it globally by making `has_strike_tag` read the
single card-tag owner above, so only one STRIKE list remains. Two accepted profiles can reach
the changed behaviour, because Wild Strike, Thunderclap and Perfected Strike are in both
pools: M10, and the training (Gym66 standalone combat) profile whose `TRAINING_CARDS`
(`crates/spire-core/src/run/training.rs`) is exposed by `crates/spire-env/src/combat_training.rs`
and pinned by `crates/spire-core/tests/combat_training.rs`. The correction is disclosed for
both, with their affected tests amended and a retained-outcome impact sweep for each, exactly
as M23 handled the Confused cost law. No descriptor, fingerprint or wire value changes.

Task M0 verifies the 43/48/15 membership against the native catalog. A mismatch is
reported and resolved at its owner; the family is never trimmed to what happens to work,
and an incomplete family rejects Foreign Influence before any RNG or payment, as M20 does
for its generators.

### 5.4 Exclusive relics and potions

The relic reference lists nine Watcher-exclusive relics (`data/relics/README.md`):

| Relic | Tier | Rule | Status |
| --- | --- | --- | --- |
| PureWater | Starter | Combat start: add 1 Miracle | Accepted |
| HolyWater | Boss | Replaces PureWater; combat start: add 3 Miracles | Accepted |
| VioletLotus | Boss | Leaving Calm gives 1 more energy | Accepted |
| TeardropLocket | Uncommon | Start each combat in Calm | Accepted |
| GoldenEye | Rare | Each Scry looks at 2 more cards | Accepted |
| Damaru | Common | At the start of each turn, gain 1 Mantra | New |
| Duality | Uncommon | Whenever the player plays an Attack, gain 1 temporary Dexterity | New |
| CloakClasp | Rare | At the end of the turn, gain 1 Block per card in hand | New |
| Melange | Shop | Whenever the draw pile is shuffled, Scry 3 | New |

The three Watcher-exclusive potions, BottledMiracle, StancePotion and Ambrosia, are
accepted and already in the M22 potion cycle. No M25 producer samples the random potion
factory (the Watcher potions are placed by fixed cycles), so the Watcher potion factory
(the 30 shared plus 3 Watcher potions) is not reachable in M25 and stays in M27.

### 5.5 Curses and statuses

Parasite (from Writhing Mass, admitted by M24) is the only curse. The statuses are Dazed,
Burn, Slimed, Wound and Void, all already in the M24 vocabulary. Act 4 adds no new kind: the
Heart's Debilitate shuffles one of each of the five statuses into the draw pile, Spire Spear
adds Burns to the discard pile, and the off-colour Attacks above create only Wound, Dazed and
Burn.

### 5.6 Union totals

| Union | Members | Count |
| --- | --- | --- |
| Permanent admission | 75 ordinary Watcher cards + Parasite | 76 |
| Card vocabulary | 76 permanent + 8 Watcher generated children + 82 non-Watcher Foreign Influence outputs + 5 statuses | 171 |
| Choice-only rows | BecomeAlmighty, FameAndFortune, LiveForever | 3 |
| Relics | M24's 19 + Damaru, Duality, CloakClasp, Melange | 23 |
| Potions | M22's 7 (unchanged) | 7 |
| Playable encounters / enemy identities | M24's 40 + ShieldAndSpear and CorruptHeart / M24's 54 + SpireShield, SpireSpear, CorruptHeart | 42 / 57 |

The permanent union is owned once, in `run/rooms/m25_content.rs`, built from
`content::watcher`'s complete roster plus Parasite, never from a copied literal list. The
relic union is the M24 union followed by the four new relics, built the same way.

### 5.7 Combat admission

Every Watcher combat today initializes each card through
`cards::watcher::initialize_profile`, which admits only the finite M21 set, the three
accepted children and (from M24) any Curse. Relic validation at combat entry uses a
profile-specific list in `items::validate_combat_inventory`. M25 follows the M20 rule:
**add a distinct complete-Watcher capability; do not widen the finite M21–M24 predicates.**

- `content::watcher` gains the complete lists `M25_WATCHER_CARDS` (75),
  `M25_WATCHER_GENERATED` (8) and `M25_WATCHER_CHOICES` (3), with
  `supports_m25_permanent(CardId)` and `supports_m25_generated(CardId)`. The M21 lists and
  predicates are unchanged.
- `CombatItemContext::with_m25_profile()` (in `items/mod.rs`, following M24's
  `with_m24_profile()`) sets a `watcher_heart` flag. Relic validation then checks the 23-relic
  M25 union. `Debug` prints the flag only when it is true, so older checkpoint and fixture
  text is unchanged. `run/combat.rs` selects it when `is_m25`.
- Under that flag, Watcher card construction admits the complete roster and the eight
  children with their Watcher initialization, Curses as M24 decided, and Foreign Influence
  outputs with **their own colour's existing initialization**. Off-colour cards in a
  permanent deck and Statuses in a starting deck still reject. Without the flag, behaviour
  is byte-for-byte unchanged.
- Supplied resources for M25 diagnostics are validated by the existing normalization owner
  (`run/training.rs`, `M21WatcherCombatStart`) extended with an explicit admission profile,
  proposed `WatcherAdmission::{M21, M25}`, that reads the M25 unions for `M25`. The M21
  admission sets and their tests are unchanged, and M24 version 2 had to restrict its
  loadout only because the M21 validator was the only one. The supplied-start helpers call
  that validator internally today (`run/diagnostics/m22.rs` and `m23.rs` call
  `M21WatcherCombatStart::Supplied(..).normalized()` before
  `Run::from_validated_combat_resources`), so the admission profile is threaded through them
  as one more parameter: Main threads it through `run/diagnostics/m23.rs` (parameterization
  only; M23 and M24 call sites pass `WatcherAdmission::M21` and their diagnostics stay
  byte-identical), and M25's own constructors (section 12.7) pass `WatcherAdmission::M25`.
  M25 does not copy the validator.

### 5.8 New shared mechanics

Each mechanic is implemented once, at the named owner, and serves every character. Owners
are from the planning research; Task M0 binds the exact names.

| Mechanic | Cards | Existing owner to extend |
| --- | --- | --- |
| Type of the previous card played this combat | FollowUp, CrushJoints, SashWhip, Sanctity | Play history (`combat/hooks.rs`, `combat/turns.rs`) |
| Separate repeated hits with a count fixed at use | BowlingBash, FlyingSleeves, Tantrum, Expunger | Damage action owner (`combat/model.rs`) |
| Random-target repeated hits | Ragnarok | The owner already used by Sword Boomerang, Rip and Tear and Thunder Strike |
| End the turn now | Conclude | `EndPlayerTurn` scheduling (`combat/turns.rs`), as Meditate and Time Warp use |
| Extra player turn that skips the enemy turn and end-of-round work | Vault | Turn owner (`combat/turns.rs`, `combat/engine.rs`) |
| HP loss at the start of the next turn (Blasphemer) | Blasphemy | Powers and turn-start hook |
| Mark and its trigger | PressurePoints | Enemy power storage; the HP-loss owner Poison uses |
| Block Return on an enemy | TalkToTheHand | Damage hooks |
| Vigor: added to NORMAL damage through the damage-give hook, removed by the next Attack's use callback | WreathOfFlame | Normal damage path |
| Block-gained callback | WaveOfTheHand | Block gain owner; new hook |
| Next Attack costs 0 | Swivel | Free-play and payment flags (`combat/card_state.rs`) |
| Delayed next-turn Wrath and draw | SimmeringFury | Turn-start hook |
| End-of-turn creation into the draw pile | Study | End-of-turn hook and the creation owner |
| Energy Down | Fasting | Turn-start energy loss |
| Growing turn-start energy | DevaForm | Watcher recharge (`combat/cards/watcher.rs`) |
| Delayed upgraded Miracle each turn | Collect | Turn-start creation |
| End-of-turn non-attack damage to all enemies | Omega | End-of-turn hook |
| Trigger on actual draw | DeusExMachina | The draw callback Void already uses |
| Choose a draw-pile card, play it twice, exhaust it | Omniscience | Selection and autoplay owners (`combat/cards/ironclad/autoplay.rs`) |
| Instant kill below a threshold | Judgment | Enemy lifecycle owner |
| Last unblocked damage on the target | Wallop | `DamageResult` |
| Hand-content legality | SignatureMove | Play legality (`combat/cards/mod.rs`) |
| Block per other card in hand; draw until hand is 10 | SpiritShield; Scrawl | Hand and draw owners |
| Same instance shuffled back into the draw pile | Tantrum | Pile owner (`combat/piles.rs`) |
| X capture with a stored hit count | ConjureBlade/Expunger, Collect | X capture owner (`combat/cards/mod.rs`) |
| Enemy intent category | FearNoEvil | Intent data (`combat/model.rs`) |
| Choice-only offer | Wish | Offer and selection owner (`combat/card_offers.rs`) |
| Any-colour Attack proposal | ForeignInfluence | `combat/generation.rs` (new pool next to the existing ones) and `card_offers.rs` |
| Permanent card upgrade from combat | LessonLearned | The fatal callback Feed and Hand of Greed use (`combat/cards/ironclad/fatal.rs`) and the combat-to-Run mutation journal (`run/combat.rs`), which today carries only Genetic Algorithm |
| Permanent gold from combat | Wish (FameAndFortune) | The Hand of Greed gold path and its Run sync |
| Plated Armor for the Watcher | Wish (LiveForever) | The Plated Armor end-of-turn hook, whose profile gate must admit Watcher |
| Relic hooks | Damaru (turn start), Duality (Attack use), CloakClasp (end of turn), Melange (draw-pile shuffle) | `items/relics.rs`; Melange also needs a post-shuffle hook in `combat/piles.rs` |

### 5.9 Named source frontiers

These orderings are not settled by a single accepted owner. Each must be bound to a cited
source fact (the research documents above, `data/cards/implementation-notes.json`, accepted
tests or retained static findings) before its test is frozen.

The engine still has to do something for every legal play, so a part that stays unsettled
after Task M0 is handled this way (planner's decision): the Content lane implements the
behaviour the cited sources most directly support, records it in the reconciliation as a
disclosed project decision with its reasoning and the facts it rests on, and pins it in a
test labelled as that decision; the PM is told before the freeze. An unsettled frontier
never becomes an `Unsupported` rejection of a legal play and is never presented as a
source-settled fact.

| # | Frontier | Settled by | Still to bind |
| --- | --- | --- | --- |
| F1 | Foreign Influence support and proposal law | Section 5.3 sources | Exact 106-kind membership against the native catalog |
| F2 | Vault | VAULT note: player end-of-turn and start-of-turn effects run; the enemy turn and end-of-round work are skipped; Blasphemer still fires | Whether Invincible's allowance resets (its reset is a monster-turn checkpoint) and how Time Warp's counter behaves across the skipped round |
| F3 | Blasphemy | BLASPHEMY note: 99,999 HP loss at the next turn start; a positive buff for Artifact; stance changes keep it | Order against other turn-start effects |
| F4 | Omniscience | OMNISCIENCE note: two plays without their energy cost; X uses the remaining energy without spending it; a turn-ending chosen card can interrupt the repeat | Time Warp ordering, and an unplayable chosen card (CLASH note) |
| F5 | Melange | Relic record | Timing of the Scry against the draw that caused the shuffle; GoldenEye, Weave and Nirvana in that Scry |
| F6 | Wave of the Hand and Duality | Card and relic records: Wave of the Hand fires only on a positive gained amount and is removed at the end of the round | Duality's Dexterity timing against the Attack's own damage and any Block it grants |
| F7 | Cloak Clasp | Relic record | Order against Like Water, Retain and Ethereal, Burn, Omega and Study at end of turn |
| F8 | Lesson Learned | LESSON_LEARNED note: the permanent deck is upgraded; fatal excludes minions, Awakened One phase 1 and non-final Darkling kills | RNG stream for the card choice; whether the in-combat copy is also upgraded. The upgrade travels as a combat journal entry applied to the Run deck once at settlement, as Genetic Algorithm's growth is today (`run/combat.rs`, whose mutation journal currently accepts only Genetic Algorithm). If the in-combat copy is upgraded too, the exit census check that compares each permanent card's upgrades with the Run deck (`run/combat.rs`) must account for that journal entry; that is a disclosed extension of the census, not a relaxation |
| F9 | Judgment | Card record | Interaction with Invincible, half-dead actors and minions |
| F10 | Deus Ex Machina | DEUS_EX_MACHINA note: exhausts itself when drawn, after creating Miracles | That creation, Scry or Meditate moves are not draws |
| F11 | Talk to the Hand | Card record: the card's own hit returns no Block; Artifact can block the installation | Which later hits return Block (per hit, multi-hit, and hits capped by Invincible) |
| F12 | Wallop | Watcher research row | `lastDamageTaken` with Block, Invincible and overkill |
| F13 | Deva Form, Fasting and Simmering Fury | DEVA_FORM note; Fasting record: player Artifact can block Energy Down | Turn-start order against Calm exit energy, Violet Lotus, Divinity exit, Rushdown and Mental Fortress |
| F14 | Swivel and Wreath of Flame | Card records: Vigor is added through the NORMAL damage-give hook and removed by an Attack's use callback | Which Attack consumes Swivel's entitlement (zero-cost, X-cost, autoplayed, created); Vigor on multi-hit and repeated plays |
| F15 | Off-colour Attacks played by a Watcher | Their accepted owners | Character-specific guards (orb slots, Silent history, STRIKE tags); Searing Blow with Master Reality (SEARING_BLOW note) |
| F16 | Fear No Evil, Signature Move, Spirit Shield, Scrawl | Watcher research rows and §5: Fear No Evil enters Calm for the ATTACK, ATTACK_BUFF, ATTACK_DEBUFF and ATTACK_DEFEND intents, with its hit queued first; Signature Move rejects another Attack in hand even if that one is unplayable; Spirit Shield counts the other cards in hand once and applies Block modifiers once (`data/cards/research-annotations.json`, SPIRIT_SHIELD) | Scrawl's stop conditions with empty piles and with a card drawn mid-loop that creates cards (Deus Ex Machina) |

### 5.10 Not included

Watcher typed generation pools and the Watcher potion factory (no M25 producer reaches
them; M26 and M27); Chemical X, Ectoplasm, Runic Pyramid and other relics outside the union;
the standard reward, shop, event and Neow pools; transforms and off-colour permanent
acquisition (M26).

## 6. Acquisition

### 6.1 Acts 1–3

Unchanged from M24 (section 4.2): the M22 card rows chosen by the full global
`CombatRecord.visit` modulo 6 with Skip, the boss rows, the first and later merchant rows
chosen by the destination act's pre-increment shop counter, the M22 elite and chest relic
pools under the per-act capture scopes, the boss relic menu, the potion cycle modulo 7,
Parasite only from Writhing Mass and Red Mask only from Tomb.

### 6.2 Act4 producers

Act4 is new in M25, and the Watcher merchant has no Act4 arm today. The Act4 producers
draw from the complete roster. Card families are the nonbasic Watcher cards of each rarity
in the standard catalog: Common 19, Uncommon 35, Rare 17. Permanent rewards include
LessonLearned and Wish (HEALING tags affect combat generators, not rewards).

| Act4 producer | Law |
| --- | --- |
| Floor 51 campfire | Shared `rest::screen_for_run`: Rest, Smith, Recall (disabled, because Ruby is held) |
| Floor 52 merchant cards | One Common, one Uncommon and one Rare, distinct, at 50 / 75 / 150 gold, sampled once through the shared staged sampler `m15_content::sample_capture_row_for` with a new family function `m25_content::purple_candidates` and scope `RunRngScope::M25CardOffer` (spelling `run.m25.card.offer`), semantic key `[4, 52, node]` plus the slot |
| Floor 52 merchant relic | Melange at 150 gold if not owned (shop-tier exclusive at 150, as M15 and M20 priced theirs) |
| Floor 52 merchant potion | StancePotion at 50 gold (the M22 first-merchant potion; Act4 has one merchant) |
| Floor 52 purge and leave | Existing lifetime purge cost and Leave |
| Floor 53 elite card child | One Common and two Uncommons, distinct, with Skip, same sampler and scope, semantic key `[4, 53, node, CombatRecord.visit, node kind]` plus the slot |
| Floor 53 elite relic | The single capture helper `rewards::capture_relic_in_scope` with scope `RunRngScope::M25RelicCapture` (`run.m25.relic.capture`), key `[4, 53, node, CombatRecord.visit, 0]`, pool Damaru, Duality, CloakClasp in that order, filtered for ownership; an empty pool makes no request and omits the row |
| Floor 53 gold and potion | Existing Watcher laws: elite gold 25 plus a bounded roll under `CombatGoldReward`; potion by the full global visit modulo 7 |
| Floor 54 Heart | No reward screen; the `HeartVictory` Proceed only |

Both new scopes are appended last in `crates/spire-core/src/run/rng.rs`. Before any RNG
request, the M25 wrapper checks that each family has exactly its expected size (19/35/17),
as M20's wrapper does for its own families, and the shared sampler then rejects an empty
family or any definition-only kind. Either failure is an explicit `RunError::Unsupported`,
never a smaller sample. The merchant RNG is committed once by the shared Act4 room owner.

### 6.3 Normal reachability

With these laws every ordinary Watcher card has a normal acquisition path: 24 nonbasic
cards and the 4 basics in Acts 1–3, and all 71 nonbasic cards by rarity in Act 4. All nine
exclusive relics are reachable (PureWater at the start, HolyWater and VioletLotus from the
boss menu, GoldenEye and TeardropLocket from elites and merchants, Damaru, Duality and
CloakClasp from the Act4 elite, Melange from the Act4 merchant), and all three exclusive
potions through the cycles. A canonical normal run is not expected to reach Act 4; this is
a property of the profile, not a claim about any outcome.

## 7. Act4 rooms, the Heart and the end of the Run

M25 uses the shared owners in `crates/spire-core/src/run/rooms/act3_end.rs` and `act4.rs`,
which already serve M10, M15 and M20.

1. **Door.** At stage 4 with Act 4 enabled, all three keys and a current Act3 boss Victory
   source, accepting `Act3Continue{stage:4}` prepares every fallible input first, then
   commits once: act 4, the Act1–3 progress cleared, an ordinary heal of max HP through the
   healing owner, the fixed map published, and `ActAdvanced{from:3,to:4}` with a
   transition reward of 0.
2. **Route.** Rest 51 → Shop 52 → Elite 53 (ShieldAndSpear) → Boss 54 (CorruptHeart), one
   edge each, all nodes non-burning. The Heart cannot be entered until the Shield and Spear
   Victory is recorded and its reward screen is closed.
3. **Shield and Spear (A0).** Shield 110 HP and Spear 160 HP, each with Artifact 1 and Back
   Attack 1; the player has Surrounded. Targeting one turns the player towards it; killing
   one faces the survivor and removes Surrounded and Back Attack. Shield: Bash 12 and
   Strength −1 for a player without orb slots, Fortify 30 Block to each enemy, Smash 34
   and Block equal to its damage. Spear: Burn Strike 5 × 2 and two Burns into the discard
   pile, Skewer 10 × 3, Strength +2 to all enemies
   (`combat/enemies/act4/shield_spear.rs`).
4. **Corrupt Heart (A0).** 750 HP, Invincible 300 per turn, Beat of Death 1. Its first move
   is Debilitate (Vulnerable 2, Weak 2, Frail 2, and one Dazed, Slimed, Wound, Burn and Void
   shuffled into the draw pile). Then Blood Shots (2 × 12) and Echo (40) in a random order,
   then a Buff (+2 Strength after clearing negative Strength, plus, by buff count, Artifact
   2, Beat of Death +1, Painful Stabs, +10 Strength, +50 Strength), repeating
   (`combat/enemies/act4/heart.rs`).
5. **Heart Victory.** Settlement writes the record and shows `HeartVictory` with one
   Proceed and no reward screen. Accepting it commits `FloorEntered{act:4,floor:55}` and
   `Ended(HeartVictory)`: floor 55, room `TrueVictoryRoom`, terminal `HeartVictory`, view
   `TrueVictory`, no further actions.

Reward ledger (`crates/spire-core/src/run/reward.rs`): +10 for each new highest Act4
floor, +5 once per key, +100 for `HeartVictory`, 0 for `Act3NoKeys`, −1 for Death or
EmptyDeck. The final Proceed therefore contributes +110 once.

`HeartVictory`, `Act3NoKeys`, Death and EmptyDeck are distinct terminals. A pending
`HeartVictory` screen is nonterminal. Budget exhaustion is an unfinished caller result and a
fault is an operational result; neither is Death or completion. The Act4 owner's fault
labels are shared M10 strings today (`run.act4.transition`, `run.m10.act4.room`,
`run.m10.act4.encounter`, `run.m10.act4.settlement`, `run.m10.heart.victory`,
`run.m10.heart.proceed`). M25 adds a selector so an M25 fault is retained as
`run.m25.act4.transition`, `run.m25.act4.room`, `run.m25.act4.encounter`,
`run.m25.act4.settlement`, `run.m25.heart.victory` or `run.m25.heart.proceed`; M10, M15 and
M20 labels do not change.

## 8. Keys

The key owners are shared and unchanged: Recall at a campfire acquires Ruby, the Sapphire
choice at a non-boss chest acquires Sapphire instead of the relic, and the Emerald row after
a burning-elite Victory acquires Emerald. Each key adds +5 once. The burning elite is marked
on each generated Act1–3 map while Emerald is missing (`crates/spire-core/src/run/map.rs`);
the fixed Act4 map has none. Keys matter only at the Act3 ending.

- **Normal play.** The frozen M25 caller (section 12.3) is the M24 caller unchanged in Acts 1
  and 2. From Act 3 on it first applies the shared public key objective
  `examples/shared/public_keys.rs::key_action` (Recall while Ruby is missing, Sapphire and
  opening chests while Sapphire is missing, the Emerald row and the shortest public route to
  a burning elite while Emerald is missing, stage 4 and the Heart Proceed), which M15 and
  M20 already use. Restricting it to Act 3 keeps every canonical case expected to reproduce
  its M24 endpoint, since none is expected to reach Act 3, while a Run that does reach Act 3
  pursues all three keys. Keys are whatever that policy earns.
- **Supplied play.** The supplied traversals (section 12.4) start before Act 3 without keys,
  so any key they hold at the Act3 ending was earned in play through the real owners. The
  supplied Act4 cases (section 12.5) start with three supplied keys; their keys are
  labelled unearned.
- **Constructed.** Core tests drive each key owner with real actions on constructed states
  and run the real ending for every boss and every key mask.

## 9. Events

The native event registry has exactly seven events: Neow (the short start), UpgradeShrine,
Purifier, AncientWriting, Joust, Falling and TombOfLordRedMask
(`crates/spire-core/src/content/events.rs`). The M25 profile reaches all of them at A0 and no
others: Act1 question rooms are UpgradeShrine then Purifier, Act2 AncientWriting then Joust
(gold at least 50) or Purifier, Act3 Falling then Tomb, and later questions are captured
Monster rooms. The event reference lists no Watcher-exclusive event. The other StS1 A0
events are not native in any profile and belong to M28, as M15 and M20 also decided.

"Event coverage relevant to the Watcher" therefore means: every one of the seven reachable
events is exercised with the complete Watcher roster and its physical-instance, type,
starter-tag, upgrade and removal semantics.

| Event | Watcher semantics to test |
| --- | --- |
| Neow (short) | NeowsLament and +7 max HP for the canonical Watcher start |
| UpgradeShrine | Upgrading new cards with cost-changing upgrades (Crescendo, Tranquility, Scrawl, Study, Vault, Omniscience, Beta-producing Alpha gaining Innate); Parasite and already-upgraded cards are not eligible |
| Purifier | Removing a new card by exact instance; removing Parasite applies its max-HP loss once; removing the last card ends the Run with EmptyDeck |
| AncientWriting | Simplicity upgrades only the purple STARTER Strikes and Defends, never other Strike-named or Defend-like cards; Elegance removes one instance |
| Joust | Payment and reveal unchanged with a complete-roster deck |
| Falling | Captures one card of each type from a complete-roster deck (Powers now include Fasting, Study and DevaForm) by exact instance; a missing type is disabled; Parasite is never captured; `[Parasite, X]` removes X and does not end the Run; `[X]` ends with EmptyDeck |
| TombOfLordRedMask | Gold 0 and 99 without a mask, 222 with one; Red Mask's Weak meets enemy Artifact |

## 10. Watcher × Act4 and new-card × Act3 compositions

Each row becomes a combat-level composition test with real setups from `enemies::setup`
and legal actions, in `crates/spire-core/src/combat/tests/m25_watcher_interactions.rs`. An
expected value must come from a cited source fact (section 5.9 and the
[Act3 static findings](../../../data/enemies/m8-static-findings.md)); a test that passes on
first run is a valid regression.

| # | Interaction | What the test must show |
| --- | --- | --- |
| T1 | Beat of Death counts Watcher plays | Each actual card play deals Beat of Death once, including zero-cost Miracle and Insight, Smite, Safety, ThroughViolence, Expunger, Foreign Influence outputs, each of Omniscience's two plays, and Tantrum. Scry, stance changes, potions, FlurryOfBlows returning and Deus Ex Machina's draw trigger do not. Beat damage is THORNS-type and is not doubled by Wrath. A lethal Beat ends the card through shared cancellation with Death. |
| T2 | Invincible allowance | Wrath and Divinity hits, Ragnarok's random hits, Tantrum and FlyingSleeves hits, Omega's end-of-turn damage and PressurePoints' Mark HP loss draw from one remaining allowance (300) that resets at the shared checkpoint. Judgment follows F9. |
| T3 | Debilitate statuses with Watcher draw and Scry | The five statuses are shuffled into the draw pile; Scry reveals one only inside the Scry window; Void's energy loss on draw follows Calm exit, Divinity and Violet Lotus energy in their real order; Scrawl draws statuses; Meditate can retain one. |
| T4 | Painful Stabs and Block Return | With Painful Stabs, each unblocked normal Heart hit adds one Wound to the discard pile. Talk to the Hand's Block Return gives Block per player attack hit on the Heart, following F11. |
| T5 | Heart Artifact and Watcher debuffs | After its first Buff the Heart has Artifact 2. SashWhip Weak, CrushJoints and Indignation Vulnerable, WaveOfTheHand Weak and PressurePoints Mark consume charges in play order. Blasphemer on the player is a positive buff. |
| T6 | Shield and Spear facing | A single-target Watcher Attack on Shield or Spear turns the player; all-enemy and random-target damage (Consecrate, Conclude, Ragnarok, Omega, the Mark trigger) follows the bound facing law; a partner's death removes Surrounded and Back Attack. |
| T7 | Shield Bash and Watcher Strength | Bash gives a Watcher Strength −1; Fasting and BecomeAlmighty Strength compose with it; the Heart's negative-Strength clearing applies only to the Heart. |
| T8 | Spear Burns with Retain and Wrath | Burn Strike puts two Burns in the discard pile; a Burn in hand at end of turn deals THORNS-type damage that Wrath does not double; a Burn discards itself even when retained (BURN note). |
| T9 | Vault against Act4 enemies | Vault ends the turn, skips the enemy turn and end-of-round work, and starts another player turn: no enemy move, intents unchanged, player end-of-turn effects (Burn, LikeWater, Omega, Study, CloakClasp) still run. Invincible's reset follows F2. |
| T10 | Blasphemy at Act4 | Blasphemer's 99,999 HP loss happens at the start of the next player turn unless combat has ended; after Vault it happens at the extra turn's start; a Heart Victory in the same turn ends combat with no death. |
| T11 | Lesson Learned and Wish at Act4 settlement | A fatal Lesson Learned on Shield or Spear (a partner death) or on the Heart upgrades one eligible permanent card once; FameAndFortune gold and that upgrade survive Victory and Death settlement exactly once; a non-fatal hit upgrades nothing. |
| T12 | Generated cards and Master Reality at Act4 | Alpha → Beta → Omega, ConjureBlade → Expunger, ReachHeaven → ThroughViolence, Evaluate/Pray/Study → Insight and Foreign Influence outputs reach their real destinations; Master Reality upgrades only at the route-specific checkpoints; a full hand sends hand creations to discard. |
| W15 | Time Warp with turn-control cards | Vault, Conclude and Meditate as the twelfth card; Omniscience's two plays count twice; Foreign Influence outputs count; the counter resets per source. |
| W16 | Curiosity with new Powers | Fasting, Study, DevaForm and Omega each give Curiosity Strength once per play; Skills such as Wish, SimmeringFury and WreathOfFlame do not. |
| W17 | Mark against Intangible and half-dead actors | Mark HP loss on an Intangible Nemesis and on Darklings follows the bound law; Judgment against a half-dead Darkling follows F9. |
| W18 | Writhing Mass with new multi-hits | Tantrum, FlyingSleeves, Ragnarok, Expunger and BowlingBash trigger Malleable and Reactive per hit; Mark HP loss is not an attack and follows the bound law. |
| W19 | Giant Head Slow with new multi-hits | Slow grows after each card, not per hit. |
| W20 | Awakened One rebirth with Mark and Omega | Rebirth removes Mark with the other debuffs; Omega can down phase one without a premature Victory. |

If a test exposes a shared-owner defect, rule 2 applies: fix the shared law for all
characters, amend and disclose any accepted assertion that pinned the wrong behaviour,
retain the original bytes, add a regression that fails under a narrower fix, and record the
impact on older profiles and retained outcomes the way M23 recorded the Confused cost
correction.

## 11. Native profile, schema and history

All names are proposals until Task M0 maps them to the merged M24 owners.

| Item | Proposed value |
| --- | --- |
| Scenario | `RoomScenario::M25WatcherHeart`, predicate `is_m25` |
| Profile | `RunProfile::M25WatcherHeart` |
| Constructors | `RunSession::new_m25(config: RunSessionConfig) -> Result<Self, RunSessionError>`; `RunBatch::new_m25(configs: Vec<RunSessionConfig>, workers: usize) -> Result<Self, RunBatchError>` |
| History | `RunBossHistory::M25WatcherHeart { act1: RunAct1BossTarget, act2: RunAct2BossTarget, act3: RunAct3BossTarget }` |
| History JSON | `{"m25-watcher-heart":{"act1":"guardian","act2":"champ","act3":"awakened_one"}}` |
| Content ID | `spire-gym-m25-watcher-a0-heart-captured-v1` |
| Rules ID | `spire-gym-native-m25-rules-v1` |
| Observation ID | `spire-gym-m25-watcher-heart-observation-v1` |
| Replay ID | `spire-gym-m25-watcher-heart-replay-v1` |
| Fingerprint domain | `spire-gym/run-schema/m25-watcher-heart/v1\0` (one terminating NUL byte) |
| Cutoff | None |

Target spellings reuse the shared codecs exactly as M24 binds them (Act3 `awakened_one`,
`time_eater`, `donu_and_deca`). M25 joins all three complement matches of `create_run`
next to M24, which keeps it out of the legacy `else` branch. All 27 typed triples are
valid. `new_m25` admits only Watcher A0 with a complete M25 triple and rejects the M22–M24
histories, other profiles' histories, and partial, duplicate-key, extra-key, wrong-type and
unknown values before Run initialization. M25 joins neither the `Act4Ready` publication arm
nor the step-time cutoff; like M20, it publishes Act4 decisions, `HeartVictory` and the
terminal `TrueVictory` state. New enum variants may need mechanical arms in existing
exhaustive matches, including the frozen M10 test matches; no assertion changes.

`run_schema_m25.rs` composes, through existing builders and without copying any table: the
M24 Watcher schema; the shared Act4 vocabulary that M20 inherits through the M15 and M10
schemas (the ShieldAndSpear and CorruptHeart setups, their three actors, ten intents and four
Act4 powers, facing and Surrounded, Invincible remaining and cap, the `HeartVictory` and
`TrueVictory` views and the Act4 map); the complete Watcher card, child, choice and relic
vocabulary; and the 82 non-Watcher Foreign Influence outputs with the card-value
projections their own schemas already use (for example Searing Blow's upgrade count,
Rampage growth, Silent and Defect history-dependent values) and every power, intent and
selection operation those outputs can create when a Watcher plays them (for example Poison
and Choke on enemies, Lock-On, Focus, next-turn energy and draw, and the Headbutt and Dagger
Throw selections). As in `run_schema_m20.rs`, class-only rows that no M25 card can create
stay out of the vocabulary. Producer and taxonomy metadata
project from the single content owners; Melange, Damaru, Duality and CloakClasp get
taxonomy rows in an M25 addition list, as M24 added Red Mask, without changing M22 or M24
metadata.

`m25_run_observation_spec(&RunCapacities)` and `encode_run_m25(&PublicDecision,
Option<&PublicMap>, &RunCapacities)` return the existing result types through
`encode_run_with_schema`. The projection keeps every M24 rule and adds the public state the
new content creates: Mark and Block Return per enemy; Vigor, Wave of the Hand, the next-free
Attack entitlement, Simmering Fury, Study, Energy Down, Deva Form's current gain and growth,
Collect's remaining turns, Omega, Blasphemer, Plated Armor and temporary Dexterity on the
player; the type of the previous card played this combat (it decides FollowUp, CrushJoints,
SashWhip and Sanctity); each card's current values (Expunger's stored hits, Halt's Wrath
total, Spirit Shield's Block, SignatureMove's legality through the action mask); the Wish
choice rows and the Foreign Influence offer rows through the shared offer codec. Omniscience's
selection shows the draw pile's contents without revealing its order, using the same rule as
Seek. Candidate *i* is row *i*; capacity is never met by filtering, truncation or paging; a
too-small capacity is an explicit structured error. Default capacities reuse
`m10_default()` as M20 does. No hidden draw order, future offer, RNG, AI state or resolver
cursor enters the observation.

Replay, record, lifecycle and batch keep their existing owners and the M24 rules: strict
raw-JSON parsing that rejects duplicate and unknown keys at every nesting level, typed and
raw validation on one path, the implicit ordinal initialized directly, semantic actions
matched against current candidates, core faults reproduced once at their checkpoint,
whole-array batch preflight, indexed `delivery_lost` after a post-dispatch fault, workers 1
and 2 matching single sessions in input order, and full reset only.

## 12. Evidence

The paired plan splits the work into three lanes: Main (Run seams, `items/mod.rs` as its
single owner, diagnostics including the `m23.rs` admission-profile parameter, integration,
caller, evidence and documents), Content (the combat and content owners and the item files
other than `items/mod.rs`, including every new card, power, child, relic and shared combat
fix) and Native (`crates/spire-env`). It fixes their file ownership and the handoffs between
them, including a second Native snapshot that contains the Content delivery. "Helper
deliveries" below means the Content and Native lanes' deliveries to Main.

### 12.1 Grades

Every result is reported at exactly one grade, and grades are never merged.

| Grade | Meaning |
| --- | --- |
| **Normal played** | Canonical start, frozen public caller, real progression. A policy result, never a gate. |
| **Supplied played** | Unearned resources installed by a default-off diagnostics constructor, then only legal actions through the real owners. Valid gameplay validation; never normal traversal. |
| **Constructed** | An owner checkpoint built directly (a combat entered with chosen cards, a constructed boss Victory record, a constructed Act4 progress) to exercise a specific owner. Proves that owner's law, not that play reached it. |
| **Native** | Rust session, replay and batch agreement on recorded actions. Not Python delivery. |

### 12.2 Per-card, relic and potion coverage tests (constructed)

M20's coverage keys are the model (`docs/milestones/m20-defect-heart-execution-ledger.md`).
Each new card, child, option, Foreign Influence output and relic gets at least one test that
plays or triggers it through the shared owners in base and upgraded form, with literal
expected values from section 5 or a cited fact, never computed by the helper under test.

| Key | File | Covers |
| --- | --- | --- |
| G | `combat/tests/m25_watcher_generation.rs` | All 75 Watcher cards and 8 children base and upgraded: construction, metadata, upgrade and copy; Insight, ThroughViolence, Beta, Omega and Expunger creation; Wish's three options |
| D | `combat/tests/m25_watcher_direct.rs` | Table of literal base/upgraded costs, targets, flags, damage, Block and magic values for the direct new cards |
| S | `combat/tests/m25_watcher_state.rs` | Previous-card-type conditions, SignatureMove legality, Wallop, Halt, SpiritShield, Scrawl, Tantrum's return, X capture for Collect and ConjureBlade, Omniscience, Judgment, FearNoEvil, Ragnarok targets |
| P | `combat/tests/m25_watcher_powers.rs` | Mark, Block Return, Vigor, Wave of the Hand, next-free Attack, Simmering Fury, Study, Energy Down, Deva Form, Collect, Omega, Blasphemer, Vault, Conclude, Deus Ex Machina, Plated Armor for the Watcher |
| F | `combat/tests/m25_watcher_foreign_influence.rs` | The 106-kind support by tier; proposal law with whole-proposal retry; zero-or-one selection; upgraded turn-cost 0; Master Reality once in hand and not on overflow; all 82 non-Watcher outputs played by an actual Watcher, base and upgraded |
| C | `combat/tests/m25_watcher_interactions.rs` | Rows T1–T12 and W15–W20 |
| I | `items/tests/m25_watcher.rs` | Damaru, Duality, CloakClasp and Melange through real combat entry, turns and exit; the five accepted relics and three potions preserved under the M25 context |
| A | `run/tests/m25_watcher_content.rs` | All 75 cards obtained, upgraded at Smith and UpgradeShrine, removed and entered next combat under M25; the unions closed; Lesson Learned, FameAndFortune and a Foreign Influence Hand of Greed kill each persisting into the Run exactly once after Victory and after Death; relic counters across combats; the seven events (section 9) |
| R | `run/tests/m25_watcher_heart.rs` | Start, prefix and the inert pre-door sites; Act4 producers; keys and all eight masks; door, route, Heart Proceed and terminals; fault labels |
| N | `crates/spire-env/tests/run_m25_{schema,session,replay,batch}.rs` | The 171/3/23/7 vocabulary, every base/upgraded value and new state row, privacy, capacity and the native contract |

### 12.3 Canonical normal runs (normal played)

Six cases, each run exactly once with budget 32768 accepted actions and the frozen caller,
the same triples as M24:

| Seed | Act1 | Act2 | Act3 |
| --- | --- | --- | --- |
| 7 | Guardian | Champ | AwakenedOne |
| 7 | Hexaghost | BronzeAutomaton | TimeEater |
| 7 | SlimeBoss | Collector | DonuAndDeca |
| 11 | Guardian | BronzeAutomaton | DonuAndDeca |
| 11 | Hexaghost | Collector | AwakenedOne |
| 11 | SlimeBoss | Champ | TimeEater |

The caller is the M24 caller (`examples/m24_support/policy.rs`, which adds the campfire Rest
rule and delegates to the frozen M23 and M22 policies) with one addition in a new
`examples/m25_support/policy.rs`: in act 3 or later the shared key objective is tried first
(section 8). The M22 policy already makes a deterministic legal choice at every Act4 room:
the first offered card at a card reward, the first affordable card and then Leave at a
merchant, Rest or Smith at the campfire, and the first non-skip row and then Proceed
elsewhere, including stage 4 and the `HeartVictory` Proceed. Its combat rules (potions first,
Calm at a stance choice, confirm Scry and Meditate, otherwise the first legal selection, then
the first legal card, then EndTurn) apply to the new selections (Foreign Influence, Wish,
Omniscience). If Task M0 finds that a new selection uses an action the M22 rule does not
recognise, the M25 wrapper takes the first offered candidate of that selection; that rule is
written and tested before the freeze. No strategy is added.

Expected outcome: Acts 1–2 are identical to M24 under the same actions and the same caller,
so each case reproduces the M24 comparison control run on the same frozen tree (section 4.2,
check 4). A difference from the control before Act 3 is a defect to investigate, not a
policy result. A difference from M24's retained receipts must trace to a disclosed global
fix. If a case reaches Act 3, the key objective may make it diverge from M24 there; that is
the documented difference. Whatever happens is recorded.

Controls: the nonattacking seed 7 Guardian/Champ/AwakenedOne control; budgets 0 and 1
(unfinished, nonterminal); and input errors for an unknown scenario, budget 32769, a
malformed budget and unsupported seed 13, each rejected before a Run is built.

### 12.4 Supplied traversal with earned keys (supplied played)

Three cases, seed 7, one per Act3 boss, budget 32768, start at a **constructed Act2 boss
source** without keys, built by M23's parameterized `boss_source` helper (an M24 version 2
interface, section 15) with the M25 scenario, the boss's Act3 history, the strong loadout
below and the `WatcherAdmission::M25` profile that Main threads through it (section 5.7).
From there only
the frozen caller and legal actions advance the Run: the Act2 boss card reward and treasure,
`(2,3,34)`, the Act3 map with the key objective, every Act3 room the route visits, the Act3
boss and ending, and, if all three keys were earned, the door, the Act4 rooms, Shield and
Spear, the Heart and the final Proceed. The Act2 boss record is constructed provenance; every
key is earned in play. A case that reaches the final Proceed is the end-to-end played
witness for real keys and the complete Heart path; a case that ends earlier is recorded where
it ended, with every key it earned.

**Strong loadout v1** (frozen before any execution): Watcher A0, 200/200 HP (unearned), 99
gold, permanent IDs 1–10 all PressurePoints upgraded once, relic PureWater, three empty
potion slots. Ten copies of one card make the simple caller's card order irrelevant, and
growing Mark is HP loss that Block does not stop. It is the Watcher counterpart of M20's ten
upgraded Searing Blows: chosen for survivability, not tuned after any outcome.

### 12.5 Supplied Act4 and Heart (supplied played)

Three primary cases, seed 7, budget 8192, start at a **constructed Act3 boss Victory
source** at `Act3BossComplete` with all three keys (supplied, unearned), built by M25's own
constructor `act3_victory_source` (section 12.7). M25 does not rely on any M24 diagnostics
helper for this start. The caller then
plays Proceed, the three dialog stages, stage 4 and the door, Rest 51, Shop 52, Shield and
Spear, the reward screen, the Heart and the final Proceed. Each primary case uses one
coverage loadout, so that together they bring 46 of the 47 new cards and all four new relics
into real Act4 play (Blasphemy is exercised by fixture X8 instead, because the simple caller
would play it and die the next turn). Each case reports which cards were actually played; a
card never drawn or played before the case ends is reported as unobserved and stays pinned by
its constructed tests.

All coverage loadouts: Watcher A0, 100/100 HP (unearned), 99 gold, permanent IDs in the
listed order, `+` = upgraded once; the remaining cards are base, so Smith stays legal.

| Loadout (Act3 source) | Deck | Relics | Potions |
| --- | --- | --- | --- |
| A, stance and attack (AwakenedOne) | Eruption, Vigilance, MentalFortress+, Rushdown, BowlingBash, Consecrate+, Crescendo, CrushJoints+, FlyingSleeves, FollowUp+, SashWhip, Tantrum+, Indignation, FearNoEvil+, SignatureMove, WheelKick+, Wallop, Ragnarok+, Judgment, Conclude+ | PureWater, VioletLotus, Duality, Damaru | StancePotion, BlockPotion, AncientPotion |
| B, retain, scry and block (TimeEater) | Vigilance, ThirdEye, LikeWater+, EmptyBody, Halt+, Protect, Tranquility+, Evaluate, Pray+, Sanctity, Swivel+, InnerPeace, EmptyMind+, WaveOfTheHand, SpiritShield+, Scrawl, PressurePoints+, TalkToTheHand, Study+ | PureWater, TeardropLocket, GoldenEye, CloakClasp, Melange | BlockPotion, DexterityPotion, BottledMiracle |
| C, powers and generation (DonuAndDeca) | Eruption, Vigilance, MasterReality+, BattleHymn, Collect, Fasting+, SimmeringFury, WreathOfFlame+, ReachHeaven, ForeignInfluence+, Alpha, ConjureBlade+, DeusExMachina, DevaForm+, LessonLearned, Omniscience+, Vault, Wish+ | PureWater, Vajra, OddlySmoothStone | EnergyPotion, Ambrosia, StancePotion |

**Conditional strong case.** If none of the three primary cases reaches `HeartVictory`, run
one more case once from the same kind of source with the strong loadout v1. A primary case
that ends in an operational fault blocks this step until the fault is diagnosed as a defect
under the freeze rules; Death, EmptyDeck and budget exhaustion do not block it. If any
primary case reaches `HeartVictory`, the strong case is recorded as skipped and unexecuted.

### 12.6 Supplied interaction fixtures (supplied played)

Eight fixtures, seed 7, 72/72 HP, 99 gold, three empty potion slots, all keys, budget 128
accepted combat actions. Each starts one edge before the real Act4 room. M25's fixture
constructor (section 12.7) builds it by running the real door from `act3_victory_source`
and then setting only the Act4 position and, for the Heart, a constructed recorded Shield and
Spear Victory with its rewards closed.

| Fixture / encounter | Deck (base unless +) and relics | Checkpoints observed if reached |
| --- | --- | --- |
| X1 Facing / ShieldAndSpear | BowlingBash, Consecrate+, Ragnarok, TalkToTheHand, Wallop; PureWater | T4, T6 |
| X2 Burns and Wrath / ShieldAndSpear | Tantrum, Indignation, Crescendo, FlyingSleeves, Halt+; PureWater | T7, T8 |
| X3 Generation / ShieldAndSpear | Alpha+, ConjureBlade, ReachHeaven, ForeignInfluence+, Study+; PureWater | T12 |
| X4 Powers and relics / ShieldAndSpear | DevaForm+, Fasting, SimmeringFury, WaveOfTheHand, Collect; PureWater, Damaru, Duality, CloakClasp | F6, F7, F13 |
| X5 Beat of Death / CorruptHeart | FollowUp, SashWhip, CrushJoints, Sanctity, Evaluate; PureWater | T1 |
| X6 Invincible and Mark / CorruptHeart | PressurePoints+, Judgment, SignatureMove, WheelKick, Eruption+; PureWater | T2, T5 |
| X7 Statuses, Scry and Melange / CorruptHeart | InnerPeace, EmptyMind, Pray, Scrawl, ThirdEye+; PureWater, Melange, GoldenEye | T3, F5 |
| X8 Turn control / CorruptHeart | Vault, Conclude, Blasphemy, Swivel, WreathOfFlame; PureWater | T9, T10 |

They use the frozen caller's combat rules unchanged. A fixture may die or miss a
checkpoint; an unobserved checkpoint is reported as missing played proof and remains pinned
by the composition tests. No draws, AI tickets or loadouts are searched.

### 12.7 Constructed diagnostics

A default-off `m25-diagnostics` Cargo feature in `crates/spire-core` (depending on
`m24-diagnostics`) provides the fixed constructors above and typed constructed stages for
core and native tests: the door source for every boss and key mask, door and Heart Proceed
faults, each Act4 room, a pending `HeartVictory`, and the new selections (Foreign Influence,
Wish, Omniscience) for semantic replay. It contains no public supplied-run API and no
arbitrary state importer. `spire-env` enables it only as a dev-dependency feature. Main owns
it in `crates/spire-core/src/run/diagnostics/m25.rs`, following the existing diagnostics
pattern (`m20.rs`, `m23.rs` and M24's planned `m24.rs`): a template from
`Run::new_room_scenario`, resources checked by the normalization owner, then
`Run::from_validated_combat_resources`, then only the frames the target owner requires.

**The Act3 boss Victory source** is an M25 addition, not an M24 interface:

```rust
pub(super) fn act3_victory_source(scenario: RoomScenario, boss: M25DiagnosticBoss,
    keys: [bool; 3], resources: Option<AuthoredRunStart>, admission: WatcherAdmission)
    -> Result<Run, RunError>;
```

It builds a Watcher A0 Run of the given scenario (M25, or M24 for the prefix check of section
4.2) with the boss's Act3 history in `CreationProfile.seen_bosses`, checks the resources
(canonical when `None`) under `admission`, and sets exactly: act 3 and floor 50; an Act3 map
whose current, visited floor-50 Boss node holds the boss and whose icon is the boss;
`combat_progress` entered and settled at 1; `last_combat` and a `CombatFinished` history
event for one constructed Victory record of that boss; the given keys; the
`Act3BossComplete` screen with its single Proceed; and the shared `Act3EndFrame` in its
`AwaitingSource` phase. From there only the real ending owner runs. M20's
`m20_supplied_act4` builds the same kind of synthetic source at stage 4; M25 starts one step
earlier so the real ending stages are played. Its test,
`m25_act3_victory_source_runs_the_real_ending`, checks the exact state with zero accepted
actions for every boss and key mask, that the real ending reaches stage 4, and that resources
outside the admission profile are rejected before any Run state is returned;
`m25_ending_matches_m24_until_stage_four` checks that the M24 and M25 sources differ only in
metadata and that stage 4 then leads to the door or `Act3NoKeys`.

### 12.8 Freeze gate

Nothing is frozen until the source is final. On the exact tree being frozen: every RED from
the content lane is resolved with its global fix and disclosure written; any native-lane
shared-owner report is resolved; both helper deliveries are integrated; the full `m25_` core
and environment suites plus the M21–M24 and M20 regression suites pass; and the full
workspace suite (`cargo test --workspace --locked --offline --no-fail-fast`) and the
feature-on core suite (`cargo test -p spire-core --features m25-diagnostics --locked
--offline`) pass with no failure other than the inherited, documented ones. The global fixes
M25 makes (the Perfected Strike correction and any guard fix in the Ironclad, Silent and
Defect owners, the card-tag owner, the post-shuffle and block-gained hooks) reach M5–M20 and
the training profile, so their suites must be green before anything is frozen. A fix needed
after the freeze requires a new, separately recorded freeze; earlier outcomes are kept as
history and are not rerun to replace them.

### 12.9 Native agreement and preservation

The six canonical transcripts, the traversal, Act4 and fixture transcripts are fed to
native single sessions, strict JSON replay (normal starts) or internal same-initializer
replay (supplied starts), and homogeneous batches with workers 1 and 2, using the recorded
semantic actions. Supplied starts reach native lanes only through the default-off
diagnostics constructors. Coexistence interleaves M25, M24, one accepted Silent and one
accepted Defect profile for 32 accepted actions each, crossing at least one combat boundary.

One stable preservation run closes the milestone:
`cargo test --workspace --locked --offline --no-fail-fast`, plus the focused M25 targets,
the feature-on and feature-off core checks and the changed example invocations. The two
scope-ignored strategy gates (M18, M19) and the inherited holds stay as they are. Python
delivery, original-runtime parity and Java RNG or map parity are not claimed.

## 13. Limits

M25 does not claim: A1–A20; full Neow; the standard reward, shop, event, potion-factory or
Neow pools; Watcher typed generation pools; transforms or off-colour permanent acquisition;
relics, potions or events outside the finite unions; a normal-start Act 3 entry, key set,
Act 4 entry or Heart Victory; Python extension delivery; original-runtime or Java parity; any
strategy or training quality. Supplied and constructed results are never presented as normal
traversal. Finite relic stock can be exhausted; that is reported, not hidden.

## 14. Final review focus

The one fresh independent final milestone review should check:

1. **Prefix and continuation.** M24/M25 equality under matched history through stage 3; the
   only gameplay-facing difference is stage 4 versus `Act4Ready`; canonical cases equal the
   M24 comparison control before Act 3 and every retained-receipt difference traces to a
   disclosed fix; every pre-door `uses_act4` site inert; every `is_m24` branch has the
   recorded M25 arm; the Task M0 interface check and any amendment are on record.
2. **Complete roster.** All 75 cards and 8 children base and upgraded, the three Wish options,
   and the 106-kind Foreign Influence support, each to its cited values; literal expected
   values; no definition-only or trimmed family; old finite predicates unchanged; one
   card-tag owner for HEALING and STRIKE that agrees with the Defect metadata.
3. **New mechanics and frontiers.** Each section 5.8 mechanic at one shared owner; each F1–F16
   bound to a cited fact or recorded as a disclosed project decision; any shared fix applied
   globally and disclosed, including the Perfected Strike correction for M10 and the training
   profile.
4. **Act4 and the Heart.** Door, route, Shield and Spear, Heart, `HeartVictory` Proceed
   across all key masks; Act4 producers and scopes; terminals, budget and faults distinct;
   M25 fault labels.
5. **Run-level content.** Admission unions closed; Lesson Learned, FameAndFortune and a
   Foreign Influence Hand of Greed kill persist once; relic hooks across combats; the seven
   events with Watcher semantics; the supplied starts built by M25's own constructors under
   the M25 admission profile.
6. **Native contract.** 171/3/23/7 vocabulary, new state rows, privacy (Omniscience's
   selection hides order), capacity errors, strict history, replay, batch, coexistence, old
   descriptors and fingerprints unchanged.
7. **Evidence honesty.** Grades separate; freeze before outcomes; once-only execution;
   every loss, fault and missing checkpoint retained; supplied keys labelled; canonical normal
   completion reported as unproven unless observed; the freeze gate ran the full workspace
   suite on the frozen tree.

## 15. M24 interfaces this design depends on

M25 is reviewed and approved against the M24 planning version 2 contract, which passed its
own independent review. M25 execution cannot start until M24 is merged.

**Re-review trigger (PM ruling).** At Task M0, before any code, Main diffs each interface in
the table below against the merged M24 source, using the check in the last column, and
records the result in the seam map. If every interface matches M24 version 2, or differs only
in a way these documents already accommodate (named in the "Accommodated differences"
column), execution proceeds. Otherwise Main writes a short M25 amendment (the interface, what
changed, the affected sections and tasks, and the proposed change) and sends it to the same
plan reviewer; execution does not continue until that reviewer returns READY on the
amendment.

| Interface (M24 v2 proposal) | Used by M25 for | Task M0 check against merged M24 | Accommodated differences |
| --- | --- | --- | --- |
| `RoomScenario::M24WatcherAct3`, `is_m24`; M24 in `uses_full_act3` and `uses_watcher_content`, not in `uses_act4` | Prefix comparison; the M24-only branch list (section 4.4) | Read the predicate bodies; M24 is in exactly those two predicates and not in `uses_act4` | Spelling of the variant or predicate |
| Every behaviour branch that names M24 alone (M24 v2 lists `shop.rs` Act2 and Act3 counters and `m23_content::relic_capture_scope`; Task M0 lists all) | M25 takes the same arm in Acts 1–3 | List every `is_m24` branch in core and env; for each, record the M25 arm under section 4.4 | Extra branches whose M25 arm is "same as M24" under the section 4.4 rule |
| `M24_WATCHER_ACT3_{CARDS,RELICS,POTIONS}`, `GENERATED_ONLY`, `STATUSES`; `RELIC_TAXONOMY_ADDITIONS` (Red Mask) | Building the M25 unions and taxonomy by extension | Compare members and order with M24 v2 §5 (29 cards, 19 relics, 7 potions, 3 generated, 5 statuses, Red Mask "special") | Different constant names or module, same members |
| Watcher Curse construction law in `cards::watcher::initialize_profile` | M25 extends the same owner | A Curse permanent constructs for every Watcher profile with no Watcher fields; Status and off-colour permanents still reject | The law expressed by a different predicate with the same results |
| `CombatItemContext::with_m24_profile()`, its `watcher_act3` flag and the M24 relic validation | Pattern for `with_m25_profile()` | Validation checks the M24 relic union under the flag; `Debug` prints the flag only when true | Different flag name |
| `RunRngScope::M24RelicCapture` (`run.m24.relic.capture`), appended last | M25 uses the same Act3 scope and appends its own two scopes after it | Variant position and spelling | None (the spelling is part of RNG keys) |
| Fault labels `run.m24.room`, `run.m24.encounter`, `run.m24.act3.end` and their selectors | Pattern for M25's Acts 1–3 labels | Selector function shape in `act3.rs` and `act3_end.rs` | Different selector function names |
| `RunProfile::M24WatcherAct3`, `RunBossHistory::M24WatcherAct3 {act1, act2, act3}`, history JSON and Act3 spellings, M24 arms in the three `create_run` complement matches | M25's history, codec and complements | Round-trip the M24 v2 JSON example; confirm the Act3 spellings and all three complement arms | None for spellings (they are wire values); different internal function names |
| `check_boundary` M24 `Act4Ready` arm and M24 in `at_m14_cutoff` with the message "M24 stops before the Act4 door" | M25 must join neither | A healthy zero-action `Act4Ready` publishes for M24, and the next step returns that error | A renamed cutoff predicate with the same behaviour |
| `run_schema_m24.rs`, `m24_run_observation_spec`, `encode_run_m24` | Base of `run_schema_m25.rs` | The M24 descriptor is built from builders M25 can compose without copying a table | Different builder names |
| M23 diagnostics helpers parameterized by M24: `creation(act2, act3)`, `canonical(boss, scenario, act3)`, `boss_source(boss, scenario, act3, resources)`, `pub(super)` in `run/diagnostics/m23.rs` | M25's Act2 boss source for the supplied traversal; Main adds the `WatcherAdmission` parameter (section 5.7) | Signatures exist with those inputs; record exactly where supplied resources are validated in merged M24 | A different parameter order or helper name with the same inputs; validation in a different helper of the same file |
| `CallerProfile::M24`, `M24CallerOutcome`, `rollout_m24`, `examples/m24_support/policy.rs::choose` with the campfire Rest rule | The M25 caller delegates to it | Signature of `choose` and the campfire rule's behaviour on M24 v2's own test decisions | None for the rule's behaviour |
| M24's six canonical receipts and its Act3 combat tests `m24_watcher_act3.rs` (W1–W14) | Expected canonical endpoints; W15–W20 extend them | Receipts retained with hashes; the W1–W14 tests present and green | Different test function names |

M25 does not depend on any M24 diagnostics constructor for its Act4 starts or fixtures; it
builds them itself (section 12.7).

## 16. PM rulings on the planning questions

Version 1 asked three questions. The PM ruled on all of them
(`/home/klottick/.local/share/spire-gym/evidence/m25-execution-20261030/pm-open-question-rulings-v1.json`,
SHA-256 `7ffe3905c054ad3713e5e0a56197d5dd3f44dc8b5e77d221ea0de3284402d86b`), and these
documents apply the rulings:

1. **Acquisition breadth:** keep the exact M24 prefix and widen only the Act4 producers
   (sections 4 and 6).
2. **Event scope:** the seven events the profile reaches, exercised with Watcher semantics,
   now; the other A0 events in M28 (section 9).
3. **Foreign Influence:** the full 106-kind closure stays in M25, because complete Watcher card
   coverage is M25's outcome (section 5.3).
4. **Perfected Strike:** fix the shared defect globally under the user rule and disclose its
   impact (section 5.3, which also names the training profile).

For the M24 dependency the PM also ruled that M25 is approved against the M24 version 2
contract, that execution waits for the M24 merge, that Task M0 applies the re-review trigger
of section 15, and that M25 owns its Act4 supplied-start constructor (section 12.7). No
question is open.

## 17. Planning record

Version 1, its research reports (Watcher roster and items; Act4, keys, producers, events and
native seams), the verification sweep, the builder check, the decisions log and its hashes
are retained outside the managed session at
`/home/klottick/.local/share/spire-gym/evidence/m25-execution-20261030/planning-v1/`.
Version 2, the review v1 disposition table and `SHA256SUMS` are at
`/home/klottick/.local/share/spire-gym/evidence/m25-execution-20261030/planning-v2/`.
