# M27 relic and potion lifecycles design

Date: 2026-10-31. Status: **planning proposal, version 2**. Version 2 resolves the independent
plan review v1 (NOT READY, 0 Critical / 5 Important / 9 Minor, report SHA-256
`c770ea9b1145511e5fde80b23fb5a2dedbe854655c09ac8a9c257bef904753f6`) and applies the PM's rulings on
the planning questions (section 18) and the PM's guidance on the review; the disposition table is
retained with the planning evidence (section 19).
Planning Bead `spire-dxo.49.1` (actor `codex-m27-planning-senior-20261031-v1`); roadmap
outcome `spire-dxo.49` ("M27 — Complete relic and potion lifecycles"). Planning source:
`main` at `7b036e8d2047f17eb765fe42b1c4bb8e164a37e9` (M24 merged in PR64). Planning authority:
the user's plan-ahead exception
(`/home/klottick/.local/share/spire-gym/evidence/standing-milestone-authority-20261030/primary-user-self-proceed-and-plan-ahead-v1.json`,
SHA-256 `783622a6bcf45e3d77de962bc0109fb1e0984b1b2173323e146f43e6b87654d9`).

**Predecessor contracts.** Neither predecessor is merged. M27 is planned against:

| Contract | Files (SHA-256) | State on 2026-10-31 |
| --- | --- | --- |
| M25 (Watcher A0 Acts 1–4 and Heart) | design `04a085d101d6de9198616fabed17b45fc3c91206336670201b79ee6f985e24e0`, plan `20bf21c46525520e18232b7880fd12859b61451d38252f27e8dc4e0d0650a62f` in `/home/klottick/.local/share/spire-gym/evidence/m25-execution-20261030/planning-v2/`, plus every PM-approved amendment up to the merge (A1, `pm-ruling-amendment-A1-v1.json`, `8e52331e…`, a wire-spelling correction M27 does not touch, and any later one) | Approved; final correction and candidate |
| M26 (shared and off-colour cards) | design `16db3830438e2e27df07c1141a9abe10a855b91dd52fece81b144bd7a0fc322d`, plan `ed02ae017121ff414e67f3f59369158ed0f53b44df3214858bc420cb90175081` in `/home/klottick/.local/share/spire-gym/evidence/m26-execution-20261031/planning-v3/`; Task M0 seam map `main-v1/task0/seam-map.md` (`61337c7b…`) | Approved; implemented on frozen M25, waiting for the M25 merge |

M27 execution cannot start until M25 and M26 are both merged. Section 17 lists every M25 and
M26 interface M27 depends on and the check Task M0 makes against the merged source, including
when an amendment must go back to the plan reviewer. The paired
[implementation plan](../plans/2026-10-31-m27-items.md) turns this design into tasks. This
document releases no runtime work by itself.

## 1. What M27 delivers

The [roadmap](2026-10-22-narrow-gym-milestones-design.md) defines M27 as: "Close remaining
standard relic/potion effects, counters, acquisition/replacement/removal, source-specific use,
healing/prevention/rescue, escape and interactions across fights and room boundaries. No inert
registered items or duplicated inventories."

M27 is the item layer that M28 (events, Neow, shops, rest, treasure and the standard pools)
calls. After M27:

1. **Every standard relic exists and works.** The complete relic union is the 178 normally
   obtainable rows of `data/relics/relics.json` plus the Circlet fallback: **179 relics**. Red
   Circlet is excluded (catalog `availability: unobtainable`, "no normal vanilla source"), as
   M26 excluded Pride. M27 adds the **117** identities that M25 does not add; every one of them
   is a shared relic. The 62 relics that exist after M25 keep their accepted behaviour and lose
   their profile gates under the complete capability. Every trigger of every relic runs at
   the shared owner of that trigger, for every character that can hold it.
2. **Every standard potion works for every character that can hold it.** All 42 potions
   (30 shared, 3 per class) already have identities; M27 makes each usable by every character
   whose potion pool contains it, in combat and, where the source allows, outside combat, with
   source-specific use (manual or automatic, targeted or not, phase and room limits), Sacred
   Bark potency and the potion relics (Sozu, Potion Belt, Toy Ornithopter, White Beast Statue,
   Cauldron).
3. **Complete item lifecycles in the Run.** Relic obtain with its equip effects (including the
   card selections of the bottles, Astrolabe, Empty Cage, Dolly's Mirror, Orrery and the rest),
   the four boss starter replacements, relic loss with its unequip effects, counters and
   charges that persist across combats and rooms, potion obtain, discard, use and capacity, the
   random-potion factory for all four characters, and complete relic pools with history, spawn
   conditions, depletion promotion and the Circlet fallback.
4. **Healing, prevention, rescue and escape.** One healing law with its relic modifiers;
   damage, HP-loss and debuff prevention relics; Fairy in a Bottle and Lizard Tail rescue with
   Mark of the Bloom; Smoke Bomb escape and what an escaped combat gives.
5. **Interactions across fights and rooms.** Item laws run at every shared Run owner that
   produces their trigger (combat entry and settlement, rest, shop, elite rewards, chests, room
   entry, card obtain and removal, gold, healing and maximum HP). Where the room that triggers a
   law is one that M28 completes (unknown-room resolution, map path choice, card-reward rarity,
   events, Neow), M27 delivers the law as a named item function with constructed tests and M28
   calls it (section 1.1).
6. **One inventory and one dispatcher.** Relics, potion slots, potencies and counters keep one
   owner each (the existing move-only custody between `RunState` and `CombatItemContext`). One
   relic dispatcher runs relic callbacks in acquisition order. One catalog-derived item
   capability replaces the per-profile allow-lists under the complete overlay; the frozen
   profile lists stay as they are for their profiles.

Every item is implemented to source law and exercised by meaningful tests. Constructed and
supplied setups are valid. No outcome run, normal-start route or policy win is required
(section 2, rule 1).

### 1.1 What M27 owns and what it leaves

| Area | M27 owns | Left to |
| --- | --- | --- |
| Relic identities and effects | All 179 relics (section 4), every trigger, all four characters | — |
| Red Circlet | — (excluded, unobtainable; boss-pool exhaustion is unreachable, section 7.4) | — |
| Relic acquisition | The single obtain owner and every equip effect, including selections; boss starter replacement; the Neow boss swap's "lose the starter relic" mutation; relic loss with unequip effects; complete relic pools, the ordinary (50/33/17) and merchant (48/34/18) tier rolls, front/back selection, spawn conditions, depletion promotion and Circlet | Which room offers what (elite, chest, shop, boss, event and Neow rows), chest-size tier tables, the complete shop stock and prices: M28 — except the existing Act 4 shop and elite rewards under complete admission (next row) |
| Act 4 shop and elite rewards under `ItemAdmission::Complete` | Every relic and potion identity they offer comes from M27's pools, tier rolls and potion producers (section 11.4); the accepted row structure stays; frozen profiles keep their finite lists unchanged | Every other room, complete shop generation and price laws: M28 |
| Event-only (Special) relics | Their effects (section 4) | Their acquisition through events and Neow: M28 |
| Shop relics' effects (Membership Card, The Courier, Smiling Mask, Maw Bank, Meal Ticket) | The price, restock, removal-cost, gold and heal laws, wired into the shared shop owner under complete admission | Complete shop generation, rows and base prices: M28 |
| Rest-site relics (Regal Pillow, Dream Catcher, Coffee Dripper, Fusion Hammer, Girya, Peace Pipe, Shovel, Eternal Feather, Ancient Tea Set) | Their options, heal amounts and outcomes, wired into the shared campfire owner under complete admission | Key interactions at the campfire (Recall): already shared; any further rest-room flow: M28 |
| Chest, unknown-room and map relics (Matryoshka, Cursed Key, N'loth's Hungry Face, Tiny Chest, Juzu Bracelet, Ssserpent Head, Wing Boots) | Their laws as named item functions; wired where a shared owner exists today (chest open hooks, room entry) | Unknown-room resolution, treasure flow and map path rules: M28 |
| Card-reward relics (Question Card, Busted Crown, Prayer Wheel, Singing Bowl, N'loth's Gift, Prismatic Shard, Dream Catcher, Orrery, Tiny House) | The reward-shape laws (choice count, extra rewards, skip option, rare factor, any-colour pool switch), wired into the shared elite reward owner under complete admission | The card-reward rarity schedule and every other reward screen: M28 |
| Relic-driven card changes (eggs, War Paint, Whetstone, Astrolabe, Pandora's Box, Empty Cage, Dolly's Mirror, bottles, Necronomicon, Calling Bell, Omamori, Darkstone Periapt, Ceramic Fish, Du-Vu Doll) | The relic hooks, flags and choices, calling M26's card owners | Card pools, obtain, upgrade, remove, transform and duplicate owners: M26 (done) |
| Potions | All 42 for every character whose pool holds them; slots, obtain, discard, use, potency, rescue, escape; the rarity-first, limited and flat factory helpers for all four characters; the combat potion appearance counter and White Beast Statue; the Act 4 shop and elite potion identities under complete admission (section 11.4) | Which other rooms add potion rewards and merchant potion rows: M28 |
| Ascension | — | A11 potion slot reduction (M31) and every other ascension change (M29–M33) |
| Public observation of the complete item set | — (section 14) | The first profile that publishes it (M28 standard profile or M34) |
| Neow | Neow's Lament (accepted) and the swap mutation above | The Neow flow and its other blessings: M28 |
| Score (Spirit Poop) | The ownership predicate and score adjustment as an item law | Any score screen: M34 |

## 2. Rules that shape this design

Binding user rules, restated so the builder does not need to look them up.

1. **The repository plays the game; it does not need to win it**
   (`/home/klottick/.local/share/spire-gym/evidence/primary-scope-clarification-20261030/gameplay-not-strategy-v1.txt`,
   SHA-256 `6a471ebf…`). "Complete" means implemented to source law and exercised by
   meaningful tests. Constructed and supplied fixtures are acceptable. There is no
   normal-start win, route or outcome gate.
2. **Fix real bugs globally.** A defect in a shared owner is fixed at the shared rule for every
   character and profile. An accepted assertion that pinned wrong behaviour is amended, the
   change is disclosed, the original bytes are retained, and a regression test must fail under
   any narrower fix. Section 12 lists the corrections planning already found.
3. **One engine.** Extend the existing catalog, item context, relic and potion owners, combat
   hooks, resolver, Run mutations, rooms and diagnostics. No copied relic tables, potion
   tables, dispatch loops, samplers, validators or counters. No public supplied-run or import
   API. Diagnostics stay default-off.
4. **No installs.** No new dependencies, no Python rebuild, no original-game execution.
5. **Frozen profiles stay frozen.** Combat-v1, the default and standalone (Gym66) profiles,
   policy-v3 and every accepted M9–M26 profile, finite list, descriptor, fingerprint, wire
   value and RNG key stay unchanged, except mechanical new exhaustive-match arms and the
   disclosed global corrections of section 12.
6. **No inert registered items and no duplicated inventories** (the roadmap outcome). Every
   relic and potion identity the engine registers has its complete effect; every relic list,
   potion slot list, counter and potency is owned once.

## 3. Approaches considered

| Approach | Result | Decision |
| --- | --- | --- |
| One complete item capability derived from the catalog, one relic dispatcher in acquisition order, complete Run item owners (obtain, equip, replace, lose, pools, potions), a combat-context overlay and a Run admission selector beside M26's, evidence by constructed and supplied tests; public exposure left to the first profile that publishes it | Closes every relic and potion once for all four characters. Frozen profiles keep their finite admission. M28 calls finished item owners when it builds the standard rooms | **Selected** |
| Widen the M10, M15, M20 and M25 Heart profiles to admit every relic and potion | Breaks their frozen descriptors, finite unions and prefixes | Rejected |
| Add a new public Run profile for M27 | Needs complete shop, chest, event and Neow rooms that M28 owns; would be a second, partial standard profile | Rejected (as M26 PM ruling 1) |
| Implement only the missing relics, per character module, as M15/M20/M25 did | Leaves every accepted relic and potion behind a profile gate, keeps one dispatcher per character and keeps the duplicated admission lists | Rejected |
| Keep the existing per-character relic modules as they are and add a second dispatcher for new relics | Two dispatch loops; cross-module relic order would follow module order, not acquisition order | Rejected (rule 3) |

## 4. The complete item set

Values come from `data/relics/relics.json` (and its `local_static` rows), the
[relic mechanics research](../../research/2026-10-19-act1-relic-mechanics.md) (§2–§4, the
141 local primary rules), the [economy and potion research](../../research/2026-10-19-act1-content-economy-potions.md)
(§4–§8), the [relic reference](../../../data/relics/README.md), and the planning research
retained at `/home/klottick/.local/share/spire-gym/evidence/m27-execution-20261031/planning-v1/`
(`research-A/` relic inventory, `research-B/` potion inventory and lifecycle, `research-C/`
plumbing, trigger taxonomy, deferred sites and precedents). They are implementation inputs,
not original-runtime validation.

### 4.1 Relic inventory after M25

Main at `7b036e8d` has 58 concrete `RelicId` variants (plus the generic `Unknown(&str)`
constructor, which is not an identity) and 58 catalog definitions; none is registered but inert.
Four identities are aliases whose catalog keys differ from their names (Ring of the Snake
`snake_ring`, Mark of the Bloom, Paper Phrog `paper_frog`, Paper Krane `paper_crane`). M25 adds
Damaru, Duality, Cloak Clasp and Melange.

| Tier | After M25 | New in M27 | Complete union |
| --- | ---: | ---: | ---: |
| Starter | 4 | 0 | 4 |
| Common | 16 | 20 | 36 |
| Uncommon | 9 | 27 | 36 |
| Rare | 10 | 24 | 34 |
| Boss | 15 | 15 | 30 |
| Shop | 5 | 15 | 20 |
| Special (event, Neow, fallback) | 3 | 16 | 19 (Red Circlet excluded) |
| **Total** | **62** | **117** | **179** |

The 40 class relics (Ironclad 11, Silent 11, Defect 9, Watcher 9) all exist after M25; every new
relic is shared. Per character the complete union is Ironclad 150, Silent 150, Defect 148,
Watcher 148 (139 shared plus the class's own). The research's status of the 62 existing relics:
33 are profile- or character-gated and 29 are partial (effect present, but full pools, room use
or some catalog trigger missing); none is complete for all characters (`research-A/relic-summary.md`).

### 4.2 New relic identities

Appended to `RelicId` after M25's variants, without renumbering, each with a catalog row that is
not `definition_only`. Variant names follow the existing style (`TheSpecimen`); `as_str` is the
snake_case of the variant (Task M0 confirms no collision). Per-relic rules are in Appendix A.

| Tier | New identities |
| --- | --- |
| Common (20) | Akabeko, AncientTeaSet, ArtOfWar, TheBoot, CentennialPuzzle, CeramicFish, DreamCatcher, HappyFlower, JuzuBracelet, MawBank, MealTicket, Nunchaku, Omamori, PenNib, PreservedInsect, RegalPillow, SmilingMask, TinyChest, WarPaint, Whetstone |
| Uncommon (27) | BlueCandle, BottledFlame, BottledLightning, BottledTornado, TheCourier, DarkstonePeriapt, EternalFeather, FrozenEgg, GremlinHorn, HornCleat, InkBottle, Kunai, LetterOpener, Matryoshka, MeatOnTheBone, MercuryHourglass, MoltenEgg, MummifiedHand, OrnamentalFan, Pantograph, QuestionCard, Shuriken, SingingBowl, StrikeDummy, Sundial, ToxicEgg, WhiteBeastStatue |
| Rare (24) | BirdFacedUrn, Calipers, CaptainsWheel, DeadBranch, DuVuDoll, FossilizedHelix, GamblingChip, Ginger, Girya, IceCream, IncenseBurner, LizardTail, OldCoin, PeacePipe, Pocketwatch, PrayerWheel, Shovel, StoneCalendar, ThreadAndNeedle, Torii, TungstenRod, Turnip, UnceasingTop, WingBoots |
| Boss (15) | Astrolabe, BlackStar, BustedCrown, CallingBell, CoffeeDripper, CursedKey, Ectoplasm, FusionHammer, PandorasBox, PhilosophersStone, RunicDome, RunicPyramid, SneckoEye, TinyHouse, VelvetChoker |
| Shop (15) | TheAbacus, Cauldron, ClockworkSouvenir, DollysMirror, FrozenEye, HandDrill, MedicalKit, MembershipCard, OrangePellets, Orrery, PrismaticShard, SlingOfCourage, StrangeSpoon, Toolbox, LeesWaffle |
| Special (16) | BloodyIdol, Circlet, CultistHeadpiece, Enchiridion, FaceOfCleric, GoldenIdol, GremlinVisage, MutagenicStrength, Necronomicon, NilrysCodex, NlothsGift, NlothsHungryFace, OddMushroom, SpiritPoop, SsserpentHead, WarpedTongs |

`RelicTier` gains `Special` (appended). Cultist Headpiece and Circlet have no gameplay effect
beyond being owned relics (catalog `effect`; Circlet stacks, section 7.4); Spirit Poop's only
effect is the score law (section 1.1). Ownership of all three still counts for every
relic-count predicate.

### 4.3 Potions

Main has all 42 standard potion identities (43 `PotionId` variants only because of the
`Unknown(&str)` sentinel). None is registered but inert, but only Block, Energy, Dexterity and
Ancient Potions work in all four characters' contexts. Thirty-seven are gated by profile or
character (the generated-card, stat and utility potions behind `shared_generated_profile()`,
which only M15 and M20 set; the M21 Watcher seven-potion use list; the class potions behind
their profile), and Fairy in a Bottle is partial (no Lizard Tail fallback, section 9.3).
Details per potion: `research-B/potion-inventory.tsv`.

| Rarity | Shared | Ironclad | Silent | Defect | Watcher | Total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Common | 16 | Blood Potion | Poison Potion | Focus Potion | Bottled Miracle | 20 |
| Uncommon | 8 | Elixir | Cunning Potion | Potion of Capacity | Stance Potion | 12 |
| Rare | 6 | Heart of Iron | Ghost in a Jar | Essence of Darkness | Ambrosia | 10 |
| **Total** | **30** | 3 | 3 | 3 | 3 | **42** |

A character's potion union is the 30 shared potions plus its own three (33), the same lists M26
built for the factory (`items/potion_generation.rs`, M26 seam map F16). No normal source gives
one character another class's potion: every random producer uses the player's class
(`initializePotions` passes `player.chosenClass`, economy research §5.1). Admission therefore
keeps class potions to their class, and the use law of every potion is character-independent.

### 4.4 Union totals

| Union | Members | Count |
| --- | --- | --- |
| Complete relics | Every `normal` catalog row plus Circlet | 179 |
| Relics per character | 139 shared + own class | Ironclad 150, Silent 150, Defect 148, Watcher 148 |
| Complete potions | Every standard potion | 42 |
| Potions per character | 30 shared + own class | 33 each |
| Persistent relic counters | Section 7.5 | 18 relics; plus the bottle card links (3) |

## 5. One complete item capability

### 5.1 Catalog and unions

- The 117 identities of section 4.2 are appended with catalog rows. One relic metadata table
  gives every relic's tier (`Catalog::relic_tier` today reads only the Silent and Defect tier
  tables plus a Holy Water special case, so it returns `None` for most relics); the Silent and
  Defect tier tables are folded into it rather than kept beside it. `EXCLUSIVE_RELIC_CHARACTERS`
  stays the one class table (Task M0 checks it lists exactly the 40 class relics).
- One potion metadata table gives every potion's rarity (today only Silent and Defect rarities
  are recorded). `EXCLUSIVE_POTION_CHARACTERS` has 9 entries on main and in the M26 tree: the
  Watcher's Bottled Miracle, Stance Potion and Ambrosia are missing, so `potion_eligible_for`
  treats them as shared; M27 adds them (correction G7), giving exactly the 12 class potions.
  Ironclad and Watcher rarities are the ones M26 bound (seam map F16: Blood Potion, Elixir, Heart
  of Iron and Bottled Miracle, Stance Potion, Ambrosia as Common, Uncommon, Rare).
- `definition_only` means "registered for metadata only, with no effect". Main marks 22 item rows
  that way although every one is accepted and active (14 relics: Burning Blood, Ring of the Snake,
  Cracked Core, Pure Water, Blood Vial, Anchor, Lantern, Strawberry, Pear, Mango, Sacred Bark, Mark
  of the Bloom, Magic Flower, Toy Ornithopter; 8 potions: Fire, Fruit Juice, Block, Energy,
  Strength, Swift, Weak, Fear). M27 clears all 22 (correction G5), so after M27 no relic or potion
  row is `definition_only`, and the unions below never need to interpret the flag.
- M26's `content::complete` gains `supports_relic(RelicId) -> bool`,
  `relic_union(Character) -> Vec<RelicId>`, `supports_potion(PotionId) -> bool` and
  `potion_union(Character) -> Vec<PotionId>`, computed from the catalog rows (catalog order),
  never written as literal lists. Tests assert the literal sizes 179 and 150/150/148/148 for
  relics and 42 and 33 per character for potions. `Unknown(..)` is never supported and Red
  Circlet has no identity.
- A data test in the style of `crates/spire-core/tests/m22_watcher_content_data.rs` reads
  `data/relics/relics.json` and checks, for every row, the identity mapping (including the four
  aliases of section 4.1 and Champion Belt, whose catalog id `CHAMPION_BELT` and variant
  `ChampionsBelt` differ), tier and characters; a literal table in the test checks the 42 potion rarities and
  classes against economy research §6.

### 5.2 Combat admission

M27 follows the M20 and M26 rule: **add a distinct complete capability; do not widen the finite
predicates.**

- `CombatItemContext::with_complete_items()` sets a private `complete_items` flag, read by
  `complete_items()`. It is valid only on a context that already has M26's `complete_cards()`:
  relic effects create and change cards from the complete pools, so complete items without
  complete cards is rejected by validation as an invariant error. `Debug` prints the flag only
  when true.
- Under the flag, `validate_combat_inventory` routes first to one new validator,
  `validate_complete_inventory(context, character)`: every relic is in
  `relic_union(character)` and appears once (a second Circlet is Circlet's counter, section 7.4);
  no starter together with its boss replacement; every potion is in `potion_union(character)`; the slot count is 3, or 5 with
  Potion Belt (A0); every counter key is an owned counter-bearing relic with a value in its
  domain (section 7.5). The frozen branches (Watcher M21/M22/M24/M25, M20, M15, base allow-lists)
  are untouched and are not reached under the flag.
- Under the flag, every item site is re-gated by the rule of section 6.3, and the potion potency
  owner is on for every potion (section 8.4).
- Without the flag, every existing admission, rejection and `Debug` text is byte-for-byte
  unchanged.

### 5.3 Run admission

- `RunState` gains `item_admission: ItemAdmission` with `ItemAdmission::{Profile, Complete}`,
  defaulting to `Profile`, beside M26's `card_admission`. `Complete` requires
  `CardAdmission::Complete`; a constructor that would combine complete items with profile cards
  fails before any Run state is built.
- Under `Profile` every Run item owner behaves exactly as today. Under `Complete` the relic and
  potion owners of sections 7–11 admit the complete unions, and `run/combat.rs` adds
  `with_complete_items()` when it builds the combat context.
- Supplied validation reuses M26's `CompleteCombatStart::normalized(character)`, which already
  builds the character's complete Heart item context and calls the shared item validator: under
  complete items it validates the supplied relics, potions and counters through
  `validate_complete_inventory`, the same validator, not a copy (Task M0 binds whether the
  selector is a parameter or a sibling variant).
- `item_admission` is omitted from `RunState` text when it is `Profile`, as M26 did for
  `card_admission`. Task M0 re-checks that neither `RunState` nor `CombatItemContext` text
  reaches any checkpoint, fingerprint, descriptor or replay JSON (M26 H0 §5 found they do not).
- Nothing public selects `Complete` in M27. Only core tests and the default-off diagnostics of
  section 14 build such Runs.

## 6. One relic dispatcher and the gate rule

### 6.1 Order law

The relic reference states: "Relic acquisition order can affect callback and action-queue order.
Do not replace documented hook phases with one global alphabetical relic order"
(`data/relics/relics.json` `global_rules`, source `wiki:GAMBLING_CHIP`). The law M27 implements:

1. Each documented phase is its own dispatch: combat start before the opening draw, combat
   start, after the opening draw, turn start before draw, turn start, after draw, card drawn, card
   use (`onUseCard`), card play (`onPlayCard`), after use, attack resolved, manual discard, exhaust,
   draw pile shuffled, Block broken, HP lost, power applied, enemy died, enemy spawned, hand emptied,
   potion used, end of turn before hand cards, end of turn before hand cleanup, end of turn, combat
   end, and each Run event of section 11. Single-relic modifiers (damage given and received, HP-loss
   amount, Block decay, recharge, play legality, repeat play, would-exhaust, power attempt, would-die,
   heal, public view, Scry, stance exit, orb passives) are laws (6.2), not phases. The relic research §3.5 is explicit
   that `onUseCard` (Nunchaku, Ink Bottle, Pen Nib and the three-card counters) and
   `onPlayCard` (Pocketwatch, Velvet Choker) are different callbacks.
2. Within a phase, relics are visited in the player's relic order, which is acquisition order.
   A boss starter replacement takes the starter's position, as the existing `replace_*` owners
   do.
3. Each relic's work goes on the side its source rule names. Bottom work runs after everything
   already queued, in visit order. Top work runs before everything already queued; because each
   `addToTop` goes in front of the previous one, among relics of one phase the later-visited
   relic's top work runs first. Task M0 checks the resolver's existing `top` insertion
   (`combat/effects.rs`, "for effect in work.top.into_iter().rev()") against this rule and
   records, for every pair of co-firing top-queued relics in a frozen profile, whether the order
   is observable. An observable difference is a global correction (section 12), never a
   profile guard.

### 6.2 One dispatcher

- A new Main-owned module `crates/spire-core/src/items/dispatch.rs` owns the only relic visit
  loops: one for combat phases and one for Run events. Main has many more relic visit and bridge
  paths than `relic_hook_effects` (which runs generic relics, then Silent, then Ironclad, then
  Ironclad turn relics) and `relics::on_run_hook`: the enemy start-of-combat effects (generic then
  Silent, pushed to the front of the queue), Brimstone's enemy work (spliced into the top work),
  the Silent pre-draw and death work, the Defect pre-battle, turn-start and end-of-turn work, the
  exhaust, damage, Runic Cube and HP-state bridges, the discard-relic gate, the shuffle notifier,
  the Watcher and Defect card-owner relic checks, and more. **Appendix C names every one** (from
  `planning-v2/research-A2/relic-visit-paths.tsv`) with its disposition: **D**, replaced by the
  dispatcher (its per-relic work becomes a handler); **L**, a single-relic modifier that becomes one
  `laws.rs` call at its owner site; **O**, owner policy that reads ownership but is not a relic visit
  (obtain, replacement, admission, validators, room producers), which stays with its owner. Task M0
  re-verifies the list on merged M25 and M26, and `m27_single_relic_visit_loop` fails if any relic
  visit remains outside `dispatch.rs`, `laws.rs`, `run_laws.rs` and the O rows.
- The dispatcher calls one handler per relic. The existing relic code in `items/relics.rs`,
  `items/silent.rs`, `items/ironclad.rs`, `items/ironclad_turns.rs` and the Defect callbacks in
  `items/mod.rs` is split into per-relic handlers with unchanged bodies (the M26 split rule:
  splitting must not reorder anything for the profile that owns it). New relics' handlers live
  in new files under `crates/spire-core/src/items/relic_effects/`, one per implementation lane
  (`turns.rs`, `play.rs`, `run.rs`, `structural.rs`, plan "Lanes"), so that no two agents edit one
  file; `dispatch.rs`, `laws.rs` and `run_laws.rs` (Main) hold the loops and call the lane files.
  Main creates every stub in every lane file, every M27 test file and every shared `mod` line before
  the lanes start (plan Task M2); shared module files have one owner, Main.
- The dispatcher is used by every context, frozen and complete: there is one loop, not a frozen
  loop and a complete loop. Frozen behaviour may change only through a named, disclosed global
  correction (rule 2). The evidence is a **deterministic replay of every retained frozen-profile
  transcript** on the new source, compared step by step with the same replay on the pre-M27 merged
  source on **full state**: the combat `State` `Debug` text (which includes both RNG streams, the
  pending and current work and the item context) and `public_full` (which keeps enemy power vector
  order), and the `RunState` `Debug` text (which includes the Run RNG) for Run transcripts. The
  legacy `public_view` is never the comparison surface, because it sorts powers by id and would hide
  exactly the reorders at stake. The retained transcripts are the final canonical, supplied and
  control attempt logs of M13–M25 retained in the evidence store
  (`planning-v2/research-A2/transcripts.md`; Task M0 publishes the exact list and replays each through
  its milestone's own replay route), and, for each of the 79 cross-path co-firing relic pairs in
  accepted finite unions (`research-A2/cofiring-pairs.tsv`), a scripted combat in both acquisition
  orders. The accepted M10–M26 suites, whose inline deterministic action procedures and M17–M19 JSON
  fixtures are transcripts too, must stay green on their own assertions, amended only by disclosed
  corrections. Every difference must trace to a named,
  disclosed global correction (G4 or a row Task M0 adds); an unexplained difference stops the work,
  and Main reports it to the PM. Every new `RunState` and `CombatItemContext` field is omitted from
  `Debug` at its default (as M26 did for `card_admission`), so the full-state text changes only
  through a disclosed correction.
- Modifier laws that are not queued work (damage given and received, HP loss, Block decay,
  energy recharge, card legality and cost, power-application prevention, healing, rescue,
  observation) are one function each in a new Main-owned `crates/spire-core/src/items/laws.rs`,
  called from exactly one owner site and visiting owned relics in order where more than one
  relic applies; for each relic the loop calls that relic's law stub in its lane file, whose body
  the lane fills.

### 6.3 Gate rule

Every relic, potion or profile comparison in `combat/**`, `items/**` and `run/**` that can
change item behaviour falls into exactly one class:

- **Class I (item law): capability gate.** Reached only when a specific relic is owned or a
  specific potion is used. The new gate is `old condition || complete_items()`, keeping the
  ownership or use test (for example Paper Krane:
  `(items.m15_profile() || items.complete_items()) && items.owns_relic(PaperKrane)`).
- **Class A (admission).** Profile allow-lists and validators. Under the flag the complete
  validator and unions apply (section 5.2); the frozen branches are unchanged.
- **Class P (potion use).** Character or profile gates on a potion's use (`items/potions.rs`,
  `combat/engine.rs` Entropic Brew and Smoke Bomb, `combat/cards/watcher.rs` potion sources).
  Under the flag each potion's own use law applies for any character whose union holds it
  (admission keeps class potions to their class); phase, target and capacity checks stay.
- **Class S (state).** On every path, with a result that depends on item state existing (the
  potency vector, a potion source frame). The new gate is `old condition || <state exists>`.
- **Class C (kept).** Profile startup structure and frozen surfaces: the M21 draw-pile sort
  before the opening shuffle, the M21 pre-draw and opening-hand scheduling, the compact Ironclad
  snapshot form, and each character's recharge owner (it gains the energy-relic and Ice Cream
  laws by calling `items::laws`, not by moving).
- **Class O (overlay).** Trackers switched on by `with_complete_items()`.
- **Debug rows.** Unchanged for frozen contexts; new fields printed only when set.

The planning sweep (`research-C/deferred-sites.tsv`) re-located the 76 relic and potion sites
that M26's Task M0 deferred to M27 (`m26 main-v1/task0/sweep-sites/sites.tsv`, `OutOfScope`
rows) and found 7 more, 83 in all (the counts below are the plan reviewer's recount of that file);
Task M0 completes the table on merged M25 and M26 and assigns each row its class by the rule above.

| Kind (sweep) | Rows | Class | Examples |
| --- | ---: | --- | --- |
| Relic effect gate | 16 | I | Holy Water and Teardrop Locket (`m22`/`m21` gates), Twisted Funnel, Ninja Scroll, The Specimen (`m15`), Paper Krane, Wrist Blade, the Tough Bandages/Tingsha/Hovering Kite discard gate, the M20 Defect pre-battle, turn-start and Frozen Core callbacks, Ring of the Serpent's extra draw, Emotion Chip |
| Potion use gate | 29 | P (source routing rows: S) | The `potions.rs` profile and character gates, Smoke Bomb and the generated-card and stat potions behind `shared_generated_profile()`, Entropic Brew in and out of combat, the Watcher potion sources and stance-potion choice, Fairy rescue, the Run obtain path with Sozu |
| Inventory and replacement admission | 21 | A | The `validate_combat_inventory` branches and allow-lists, the class and duplicate checks in `relics::acquire`, the three starter-slot replacement checks, the capacity invariant |
| Pool gate | 3 | A | Class potion pools (M26 completes them), the reward, boss-menu and shop stock tables; under complete admission the Act 4 shop and elite reward identities come from the general owners (section 11.4), and the finite tables stay unchanged for their profiles |
| Potency state | 2 | S | The M21 potency and last-use provenance owner, on for every complete context (section 8.4) |
| Startup structure and snapshot | 5 | C, except Ring of the Serpent (I) and Pure Water's phase (section 7.6) | The M21 draw-pile sort, pre-draw and opening-hand scheduling, the compact Ironclad snapshot |
| Debug formatting | 6 | Debug rows | The conditional `CombatItemContext` and Silent-state fields |
| Other | 1 | Identity helper | `watcher::active`; callers carry their own class |

### 6.4 Inertness

Class I sites are reached only under the flag. Class S sites run their old branch while the state
is absent. Class C sites are unchanged. The single dispatcher is the one structural change for
frozen contexts, and the replay evidence of section 6.2 shows that every frozen difference it causes
is a disclosed correction. Separately, on the new source, for each Heart context with its own
accepted relics and potions, a scripted combat has equal full state (`State` `Debug` and
`public_full`) at every step with and without `with_complete_items()` (`m27_capability_overlay_inert`,
section 15.3): the flag alone changes nothing for a profile's own items.

### 6.5 Hook points

The planning taxonomy (`research-C/trigger-taxonomy.tsv`) maps the catalog's 119 trigger labels
to engine owners. Most map to existing `Hook` variants (`CombatStartedPreDraw`, `CombatStarted`,
`TurnStartedPreDraw`, `TurnStarted`, `CardPlayed`, `CardAfterUse`, `CardDrawn`,
`CardManuallyDiscarded`, `CardExhausted`, `Damage`, `Death`, `PlayerPreEndTurnCards`,
`PlayerEndTurn`, `TurnEnded`, `CombatSettled`, and the Run-side `RoomEntered`, `Rested`,
`ChestOpened{boss}`, `ChestOpenedAfter{boss}`, `CardAcquired`, `CardRemoved`). M25 and M26 add three
hooks M27 reuses rather than duplicates: `Hook::DrawPileShuffled` (M25 plan C6, dispatched from the
shuffle in `combat/piles.rs`; Melange), `Hook::AttackResolved` (Duality) and
`Hook::CloakClaspBeforeCleanup{hand_size}` (Cloak Clasp's end-of-turn point before hand cleanup).
The other seams below do not exist on main; Main adds each one, once, at the named owner, with a stub
body that changes nothing until a relic is owned. Task M0 binds the exact lines on merged M25 and M26.

| Seam | Owner today | Relics |
| --- | --- | --- |
| Draw pile shuffled (after the shuffle that a draw caused) | M25's `Hook::DrawPileShuffled` (reused) | The Abacus, Sundial (Melange from M25) |
| Attack resolved; end of turn before hand cleanup | M26's `Hook::AttackResolved` and `Hook::CloakClaspBeforeCleanup{hand_size}` (reused) | Duality and Cloak Clasp (accepted); any new relic at those points uses them |
| Card use vs card play distinction (`onUseCard` vs `onPlayCard`, relic research §3.5) | `combat/engine.rs` card use and `CardAfterUse` | Use: Nunchaku, Pen Nib, Ink Bottle, Kunai, Shuriken, Ornamental Fan, Letter Opener, Bird-Faced Urn, Mummified Hand, Orange Pellets, Necronomicon (Duality from M25); play: Pocketwatch, Velvet Choker |
| Card play legality | `engine.rs` `validate_card_use` and the action mask | Velvet Choker (6 per turn), Blue Candle (curses playable), Medical Kit (statuses playable) |
| Repeat play | The autoplay owner (`combat/cards/ironclad/autoplay.rs`) | Necronomicon |
| Played card would exhaust | `combat/piles.rs` / `combat/exhaust.rs`, before the exhaust callbacks | Strange Spoon |
| Outgoing attack damage modifiers | The shared damage owner (`combat/effects.rs`, `combat/cards/mod.rs`) | Pen Nib (double), Strike Dummy (+3 on STRIKE-tagged cards), The Boot (unblocked 1–4 becomes 5); Paper Phrog and Wrist Blade re-gated |
| Incoming damage and HP-loss modifiers | `combat/effects.rs` | Torii (unblocked attack damage 2–5 becomes 1), Tungsten Rod (HP loss −1), Odd Mushroom (player Vulnerable 25%); Paper Krane re-gated |
| Enemy Block broken | `combat/effects.rs` damage owner (exact equality counts, relic research §3.7) | Hand Drill |
| Power application attempt | The power owner (`combat/effects.rs` / `combat/powers.rs`), after source callbacks and before Artifact (relic research §2) | Ginger (Weak), Turnip (Frail) |
| Would die (player) | `combat/effects.rs` damage owner, the Fairy rescue site | Lizard Tail (after Fairy; section 9.3) |
| Enemy spawned | `combat/enemies/mod.rs` | Philosopher's Stone (Strength to every enemy, including later spawns) |
| Hand empty after the queue drains | `combat/turns.rs` | Unceasing Top |
| Block decay at turn start | `combat/turns.rs` (before the newly queued draw, relic research §2) | Calipers |
| Energy recharge | Each character's recharge owner (non-deferred `turns.rs`, Defect, Watcher; class C) | Ice Cream (carry unspent energy; precedes Conserve); boss +1 energy relics enter master energy at combat preparation |
| Opening deck order | Combat entry draw-pile construction (`engine.rs`) | Bottled Flame/Lightning/Tornado (bottled card placed like Innate, relic research §3.8) |
| Enemy intent and draw-pile view | `combat/view.rs` public view | Runic Dome (intents hidden), Frozen Eye (draw order visible) |
| Potion used | The single potion-use owner (section 8.3) | Toy Ornithopter (accepted; moved behind the seam) |
| Run: floor and room entry | `run/world.rs` floor entry, `Hook::RoomEntered` | Maw Bank, Meal Ticket (shop), Eternal Feather and Ancient Tea Set (rest), Ssserpent Head (unknown room) |
| Run: campfire options, heal and outcome | `run/rooms/rest.rs` | Regal Pillow, Dream Catcher, Coffee Dripper, Fusion Hammer, Girya, Peace Pipe, Shovel |
| Run: shop prices, purchase and removal | `run/rooms/shop.rs` | Membership Card, The Courier, Smiling Mask, Maw Bank |
| Run: reward building | `run/rooms/rewards.rs` | Question Card, Busted Crown, Prayer Wheel, Singing Bowl, N'loth's Gift, Prismatic Shard, Golden Idol, White Beast Statue, Black Star, Sozu |
| Run: before and after card obtain | M26's `run::mutations::obtain_card` (every obtain path) | Frozen/Molten/Toxic Egg (upgrade before adding), Omamori (negate a curse), Ceramic Fish, Darkstone Periapt, Du-Vu Doll |
| Run: gold gain and spend | `run::mutations::gain_gold`, the shop's spend | Ectoplasm (blocks gain), Bloody Idol (heal 5 on gain), Maw Bank (stops on spend) |
| Run: healing | `items::ordinary_heal_amount_in_room` (the one heal law, section 9.1) | Mark of the Bloom, Magic Flower (accepted) |
| Laws for rooms M28 completes | none today (unknown-room resolution, map path choice, card-reward rarity roll, score) | Juzu Bracelet, Tiny Chest, Wing Boots, N'loth's Gift rarity factor, Spirit Poop |

## 7. Relic lifecycle

### 7.1 Obtain and equip

- One obtain owner, `items::obtain_relic(state, id, source)`, generalizes today's
  `relics::acquire`. Under `Profile` it is `acquire` exactly. Under `Complete` it checks, in
  order: no active combat; `relic_union(character)`; not already owned (except Circlet, which
  stacks, section 7.4); the boss replacement rule (section 7.2); then appends the relic and runs
  its equip effect once.
- Equip effects (catalog `pickup` and `pickup_*` triggers): maximum HP (Strawberry 7, Pear 10,
  Mango 14, Lee's Waffle 7 then a full heal, through the existing `increaseMaxHp` law: raise the
  maximum, then heal by the same amount through the heal law, research §3.2); gold (Old Coin
  300); potion slots (Potion Belt appends two empty slots once; today no equip path adds them);
  card changes through M26's owners (War Paint and Whetstone upgrade two random eligible Skills
  and Attacks, Astrolabe transforms three chosen cards with upgrade, Pandora's Box transforms
  every Strike and Defend, Empty Cage removes two chosen cards, Dolly's Mirror duplicates one
  chosen card, Calling Bell adds Curse of the Bell, Necronomicon adds Necronomicurse); offers
  (Calling Bell's three relics, Cauldron's five flat-pool potions, Orrery's five card rewards,
  Tiny House's potion, card reward, 50 gold, 5 maximum HP and random upgrade); the bottles'
  one-card selection; counters seeded (section 7.5).
- Every selection is a Run continuation through the existing Run selection owner
  (`run/rooms/selection.rs`, `PendingContinuation`), with the exact latches of relic research
  §3.1 (bottles, Astrolabe, Empty Cage, Dolly's Mirror, Calling Bell, Pandora's Box). An empty
  eligible group follows that table (for example Astrolabe with no purgeable card opens nothing;
  three or fewer are all transformed). A rejected selection leaves the Run unchanged.
- An equip effect that offers rewards (Calling Bell, Cauldron, Orrery, Tiny House) produces its
  offers through the reward owner; claiming them uses the ordinary obtain owners (Sozu applies to
  potion offers, section 8.2).

### 7.2 Boss starter replacement

The four replacements (Black Blood for Burning Blood, Ring of the Serpent for Ring of the Snake,
Frozen Core for Cracked Core, Holy Water for Pure Water) keep the existing owners
(`relics::replace_*`): the boss relic takes the starter's position and the starter's counter is
removed. Under `Complete` they are reached through `obtain_relic`. A replacement relic is
offered only when its starter is owned (catalog `acquisition.replaces`; economy research §4.1
"Black Blood needs Burning Blood"), which the pool owner enforces (section 7.4).

### 7.3 Relic loss

- One loss owner, `items::lose_relic(state, id)`, removes the relic outside combat and runs its
  unequip rule: the bottles clear their bottled card link (relic research §3.1), Potion Belt
  removes its two slots (planning expectation: the last two slots, only if they are empty; Task M0
  binds the source rule as frontier F6),
  Girya's future Strength ends (catalog note), boss +1 energy ends with the relic (energy is
  read from owned relics at combat preparation, section 6.5), and every counter of the relic is
  removed. A relic that is not owned is an error with the Run unchanged.
- Callers: the Neow boss swap ("lose your starter relic, obtain a random boss relic") is an M27
  mutation, `items::neow_boss_swap(state, key)`, which removes the starter before drawing (economy
  research §4.1); the Neow screen that offers it is M28. Event trades (N'loth's two offered
  relics, Forgotten Altar's Golden Idol for Bloody Idol) call `lose_relic` and `obtain_relic`
  from their M28 owners.

### 7.4 Relic pools

- One pool owner, `items::relic_pools`, with its state on `RunState` (`relic_pools`), built when a
  Run enters `ItemAdmission::Complete`: for each tier (common, uncommon, rare, shop, boss) the
  character's eligible relics (139 shared plus the class's, by tier) are shuffled with the
  Run RNG under `RunRngScope::M27RelicPools` (with `M27RelicTier` and `M27PotionOffer` for the draws
  of section 11.4, all appended after M26's last scope), and every owned relic is removed (economy
  research §4.1).
- Selection law (economy research §4.1): normal selection takes the front of the requested tier,
  the end selector takes the back (boss always front); a taken candidate is consumed even if
  skipped; a candidate whose spawn condition fails stays consumed, and the selection retries
  from the end of the originally requested tier; an empty common, uncommon or shop tier promotes
  to uncommon, rare or uncommon through the normal selector; an empty rare tier gives Circlet.
  The boss tier cannot empty in a standard run: about a dozen boss candidates are consumed at
  most (three per Act 1–3 boss chest, the Neow swap, and spawn-condition rejections such as Ectoplasm
  after Act 1) against 21 boss relics for the Watcher and 22 for every other character, so its
  catalog fallback (Red Circlet, excluded) is unreachable; an empty
  boss tier is an invariant error, never a silent substitute (frontier F7).
- Spawn conditions are the economy research §4.1 table (floor limits for Courier, Maw Bank, Old
  Coin, Smiling Mask, Girya, Peace Pipe, Shovel, Matryoshka, Wing Boots, Tiny Chest, Preserved
  Insect and the sixteen floor ≤ 48 relics; Black Blood needs Burning Blood (and likewise each
  boss replacement needs its starter); Ectoplasm needs act ≤ 1; the bottles need an eligible
  permanent card; screenless sources exclude the bottles and Whetstone; non-campfire sources
  exclude Girya, Peace Pipe and Shovel) plus each catalog row's `acquisition.restrictions`.
- Two tier rolls are pool-owner functions, each one 0–99 integer: the **ordinary** roll (Common 50,
  Uncommon 33, Rare 17; relic reference `global_rules`), used by elite rewards, Black Star and
  Shovel's Dig; and the **merchant** roll (Common 48, Uncommon 34, Rare 18; economy research §3.1),
  used by merchant relic slots and The Courier's relic replacement. Chest-size tier tables, the
  complete shop's slots and which rooms offer which tier are M28's.
- Circlet: offered when a pool is exhausted as above; owning a second Circlet increments its
  counter instead of adding a relic (economy research §4.1 "Circlet stacking has its actual
  instant-obtain counter path").

### 7.5 Counters and per-relic state

All relic state has one owner. Persistent counters live in the existing `relic_counters` map,
which moves into combat with the inventory and back at settlement; combat-only state lives in
one private combat-local map on `CombatItemContext` that is dropped at settlement and omitted
from `Debug` when empty. The accepted named latch fields (Orichalcum, Lantern, Hovering Kite,
Wrist Blade, Emotion Chip, Runic Capacitor, Slaver's Collar) stay as they are.

| Relic | Persistent state (domain) | Source |
| --- | --- | --- |
| Pen Nib, Nunchaku, Ink Bottle | Attack or card count toward 10 (0–9) | catalog `state_and_reset` |
| Incense Burner | Turn count toward 6 (0–5) | catalog |
| Happy Flower | Turn count toward 3 (0–2) | catalog |
| Sundial | Shuffle count toward 3 (0–2) | catalog |
| Inserter, Neow's Lament | Accepted (−1–1; 1–3 or −2) | accepted |
| Omamori | Charges (0–2) | catalog |
| Wing Boots | Charges (0–3) | catalog |
| Matryoshka | Charges (0–2) | catalog |
| Girya | Lifts (0–3) | catalog |
| Tiny Chest | Unknown-room count toward 4 (0–3) | catalog |
| Circlet | Number owned (1–n) | economy research §4.1 |
| Lizard Tail, Maw Bank, N'loth's Hungry Face | Used / disabled flag | catalog |
| Ancient Tea Set | Armed after resting until the next combat's first turn | catalog |
| Bottled Flame, Lightning, Tornado | Link to one permanent card instance (a relic-card link map moved with the inventory, not a counter, not a card flag) | relic research §3.1 |

Combat-local state (reset each combat): the per-turn attack and skill counts (Kunai, Shuriken,
Ornamental Fan, Letter Opener), Velvet Choker's per-turn play count, Pocketwatch's per-turn play
count, Orange Pellets' card types this turn, Art of War's "Attack played this turn", the turn
number for Horn Cleat, Captain's Wheel and Stone Calendar, Centennial Puzzle and Gambling Chip
"used", Necronomicon "used this turn", Mutagenic Strength's first-turn loss. Task M0 checks each
against its catalog row and research rule and records it in the H0 counter table.

### 7.6 Phase of existing startup relics

Pure Water's Miracle is created before the opening draw under M21 (`atBattleStartPreDraw`) and at
combat start otherwise (`items/relics.rs`). The source phase is before the opening draw; under
complete items Pure Water and Holy Water use it. The frozen branch is kept for non-M21 contexts
only if Task M0 shows no accepted non-M21 context can own Pure Water; otherwise the difference is
a global correction (section 12).

## 8. Potion lifecycle

Source law is economy research §5–§8; engine facts are `research-B/potion-lifecycle.md`.

### 8.1 Slots and capacity

- Potion slots keep their one owner: `RunState.potions`, moved into `CombatItemContext` for a
  combat and back once at settlement. Previews and public views stay projections.
- Capacity law: 3 slots at A0, plus 2 with Potion Belt. Potion Belt's equip appends two empty
  slots once (today no equip path does, correction G3). The validator's fixed 3-or-5 shape stays
  for frozen contexts; the complete validator uses the law. The A11 base of 2 is M31's (the
  current A11 constructor and the 3-or-5 validator disagree; recorded as a limit, section 16).

### 8.2 Obtain

One obtain law for every producer: the first empty slot in belt order; a full belt returns
"not obtained". Sozu is checked by the consumer, as the source does (economy research §7.2):

| Consumer | With Sozu | Full belt |
| --- | --- | --- |
| Combat `ObtainPotion` (Alchemize, combat Brew, card effects) | Refused at dispatch (accepted) | Not obtained; the proposal is not resampled |
| Reward claim | The reward is removed without obtaining (today the direct claim ignores Sozu: correction G2) | The offer stays |
| Shop purchase | Refused before payment (today ignored: G2) | Refused |
| Outside-combat obtain (Brew outside, Cauldron, Tiny House, M28 events) | Checked before generating proposals (Brew) or before obtaining | Not obtained |

Obtaining never calls the potion-use callbacks.

### 8.3 Use and discard

- One combat use owner (`prepare_combat_potion` and `use_combat_potion`) and one Run use owner.
  Manual use: the potion's effect, then the ordered relic potion-use callbacks (Toy Ornithopter,
  5 HP, not doubled by Sacred Bark), then the slot is emptied. A targeted potion needs a valid,
  non-dying enemy; cancelling targeting changes nothing.
- Phase law: a combat potion needs live combat, an unfinished turn and a living enemy. Blood
  Potion, Fruit Juice and Entropic Brew are also usable outside combat and are rejected in combat
  only after the turn has ended. Every potion is rejected during We Meet Again (the event, M28,
  supplies that fact). Fairy in a Bottle is never manually usable; Smoke Bomb adds its own rule
  (section 10).
- Under complete items, every potion in the character's union uses this law for any character:
  the 30 potion-use gate rows (`shared_generated_profile()`, the M21 seven-potion list, the
  Silent and Defect `m15`/`m20` profile tests, Entropic Brew's Silent-or-M20-Defect test, the
  Watcher potion-source routing) are re-gated as Class P. Outside combat, Entropic Brew uses the
  unrestricted rarity-first helper (Fruit Juice possible) and checks Sozu first; in combat it
  uses the limited helper and queues one obtain per live slot.
- Effect provenance: potion effects that need their slot (potency, consumption after
  resolution) carry the potion source (`Source::Potion{slot, id}`), as M20's Energy Potion and
  M21's Watcher potions do; the game-level owner of anything they apply stays the player. Task M0
  lists every consumer of `Source::Potion` and binds the rule (frontier F8).
- Discard: outside combat the accepted Run action stays and gains the We Meet Again fact. In
  combat, the source allows discarding a potion (no effect, no use callbacks); under complete
  items the combat action set gains `DiscardPotion { slot }`, reachable only under the flag, so no
  frozen mask or encoder changes (`crates/spire-env` gets mechanical arms only).

### 8.4 Potency and Sacred Bark

- One potency owner: the accepted M21 per-slot potency and last-use provenance on
  `CombatItemContext`, switched on for every complete context. Potency is fixed when the potion
  is created or obtained, doubled if Sacred Bark is owned then, and recomputed for every held
  potion when Sacred Bark is obtained (its equip re-initializes held potions). Losing Sacred
  Bark does not reduce held potions (frontier F4).
- The doubled values and exceptions per potion are in Appendix B (from `research-B` and the
  Sacred Bark record: Smoke Bomb, Snecko Oil's cost range, Toy Ornithopter's 5 HP, Blessing of
  the Forge, Gambler's Brew, Elixir and Entropic Brew's count are not doubled; the card potions
  double the copies of the chosen card, not the three offers).

### 8.5 Factory and producers

- The three helpers of economy research §5.1 serve all four characters: **rarity-first**
  (one 0–99 roll: 0–64 Common, 65–89 Uncommon, 90–99 Rare, then uniform proposals from the
  33-potion class pool until the rarity matches) for combat drops, merchants and Courier
  replacements; **limited** (Alchemize and combat Entropic Brew: rarity fixed, first flat
  proposal discarded, Fruit Juice excluded) — M26 made Alchemize all-character; M27 makes combat
  Brew all-character; and **flat** (1/33 per potion) for Cauldron, Tiny House and M28's events and
  Neow.
- Combat potion appearance (economy research §5.2): one offset on `RunState` (starting at 0);
  elite and event-room combats use chance 40 + offset; an ordinary combat, including a boss combat
  (the boss room inherits the ordinary monster room, economy research §5.2), uses it only if no
  enemy escaped (otherwise it starts from 0); White Beast Statue sets 100; four or more existing reward
  rows set 0; roll 0–99, success below the chance adds one rarity-first potion offer and lowers
  the offset by 10, failure raises it by 10. Claiming, skipping or using the offer does not change
  the offset. Under complete admission the shared elite reward owner calls this law; M28's
  reward builder calls it for every other combat room.
- Merchant potion identities and The Courier's replacement potion are rarity-first producer
  functions; their rows and prices are M28's.

### 8.6 Potion relics

Sozu (+1 energy at combat preparation — missing today, correction G1 — and the obtain rule of
8.2), Potion Belt (8.1), Toy Ornithopter (8.3), Sacred Bark (8.4), White Beast Statue (8.5),
Cauldron and Tiny House (equip offers, 7.1), The Courier (8.5 and section 11).

## 9. Healing, prevention and rescue

### 9.1 One healing law

Every heal goes through one law, the accepted `items::ordinary_heal_amount_in_room` extended
(economy research §8): no heal while dying; the player's relic heal callbacks in relic order
(Mark of the Bloom makes it 0; Magic Flower multiplies by 1.5, rounded as the accepted code does,
only in live combat with a current map node), then power callbacks; the result is capped at
maximum HP. Rest healing, potions, relic heals (Burning Blood, Black Blood, Blood Vial, Meat on
the Bone, Bird-Faced Urn, Pantograph, Meal Ticket, Eternal Feather, Bloody Idol, Lee's Waffle,
Lizard Tail, Fairy, Toy Ornithopter) and card heals all use it. Maximum-HP gains use the
accepted `increaseMaxHp` law (raise the maximum, then heal by the same amount through this law).

### 9.2 Prevention

Each is one law at one owner (section 6.5): Ginger and Turnip block Weak and Frail at the
power-application gate (after source callbacks, before Artifact); Torii, Tungsten Rod, Odd Mushroom
and Paper Krane modify incoming damage and HP loss; Fossilized Helix gives Buffer 1 at combat
start; Calipers limits Block decay to 15 (Barricade and Blur still suppress all decay); Omamori
negates curse obtains; Ectoplasm blocks gold gain; Sozu blocks potion obtains; Mark of the Bloom
blocks healing and rescue; Runic Pyramid suppresses the end-of-turn hand discard (Retain and
Ethereal keep their own paths); Coffee Dripper and Fusion Hammer remove the Rest and Smith
options; Busted Crown removes two card choices.

### 9.3 Rescue

On the damage path, after the HP mutation and card notifications, if the player's HP is below 1
(economy research §8): if Mark of the Bloom is owned there is no rescue; otherwise the first Fairy
in belt order heals 30% of maximum HP (60% with Sacred Bark), at least 1, and its slot is emptied
without the potion-use callbacks (no Toy Ornithopter); otherwise an unused Lizard Tail heals 50% of
maximum HP and is marked used. The rescue is the damage path only, not a general resurrection, and
a later queued heal never undoes an established death. The accepted Fairy branch is re-gated as
Class P; Lizard Tail is new.

## 10. Escape

- Smoke Bomb is usable in combat except when any enemy is a boss or has Back Attack (economy
  research; `combat/smoke.rs`). Task M0 checks that the accepted boss list equals the boss
  enemies of every A0 Act 1–4 encounter (`EncounterId` boss setups) and replaces the list by
  that encounter fact if it does not.
- An escaped combat ends as `Escaped`: no combat rewards, the room completes, the inventory is
  returned once (the Smoke Bomb slot empty), persistent counters persist and combat-local state is
  dropped.
- Today the escape queues `CombatSettled` (`combat/effects.rs` `SmokeEscape`), and Burning Blood and
  Black Blood heal on that hook with no outcome check (`items/relics.rs` `on_run_hook` and the
  `CombatSettled` arms). Frontier F3 starts from that fact: if the sources show that combat-end relics
  (Burning Blood, Black Blood, Meat on the Bone, Face of Cleric) do not run after an escape, the change
  is a global correction that reaches M15 and is disclosed; otherwise the accepted behaviour stays and
  the new combat-end relics follow it.
- The source rule is enemy-based: Smoke Bomb cannot be used while any monster in the room is of
  boss type or has Back Attack (economy research §6, Smoke Bomb record; research-B §5). The engine
  uses a fixed list of boss enemy ids instead of the boss type. Task M0 checks that the list equals
  every boss-type enemy of A0 Acts 1–4; under complete items the rule reads the enemy's boss type
  from the shared enemy data, so a boss fought inside an event (M28, for example Mind Bloom's) is
  covered by the same fact. (The plan review's Minor 8 called the source rule room-based; the cited
  source states the monster condition, and the disposition records this.)
- Enemy escapes (Looter, Mugger, the thieves) are accepted enemy behaviour; M27 adds only their
  effect on potion appearance (8.5).

## 11. Interactions across fights and rooms

### 11.1 Across fights

Persistent counters, bottle links, potion slots and potencies cross combat boundaries through the
single custody move. After Victory, Death and Escape the inventory returns exactly once; combat-
local state never reaches the Run; a counter that reaches its trigger in one combat triggers in
the next (Pen Nib's tenth Attack, Nunchaku, Ink Bottle, Incense Burner, Happy Flower, Sundial,
Inserter). Neow's Lament's three combats are counted the same way (accepted).

### 11.2 Shared Run owners under complete admission

Under `ItemAdmission::Complete` the existing shared owners call the item laws:

| Owner | Relics and potions |
| --- | --- |
| Combat entry (`Run::enter_owned`, combat preparation) | Boss energy relics, Slaver's Collar (accepted), Preserved Insect and Sling of Courage (elite), Pantograph (boss), Neow's Lament (accepted), Du-Vu Doll, Girya, Ancient Tea Set |
| Combat settlement (`Run::settle_combat`) | Burning Blood, Black Blood (accepted), Meat on the Bone, Face of Cleric; counters returned |
| Campfire (`run/rooms/rest.rs`) | Regal Pillow, Dream Catcher, Coffee Dripper, Fusion Hammer, Girya, Peace Pipe, Shovel, Eternal Feather and Ancient Tea Set on entry |
| Shop (`run/rooms/shop.rs`) | Meal Ticket on entry, Membership Card and The Courier prices, Smiling Mask removal price, Maw Bank on spend, The Courier's replacement rows, Sozu on potion purchase; relic and potion identities as section 11.4 |
| Combat rewards (`run/rooms/rewards.rs`, the elite and Act 4 paths) | Golden Idol, White Beast Statue and the appearance law, Black Star, Question Card, Busted Crown, Singing Bowl, Prismatic Shard, N'loth's Gift's factor where the rarity roll exists, Sozu on claim; relic and potion identities as section 11.4 |
| Chest open hooks (`Hook::ChestOpened{boss}`) | Matryoshka, Cursed Key, N'loth's Hungry Face |
| Floor and room entry | Maw Bank, Ssserpent Head (unknown room), Tiny Chest's count |
| Card obtain and removal (M26 mutation owners) | Eggs, Omamori, Ceramic Fish, Darkstone Periapt |
| Gold, healing and maximum HP | Ectoplasm, Bloody Idol, Mark of the Bloom, Magic Flower |

In M27's complete Runs these owners are reached by the Act 4 sequence of the four complete Heart
scenarios (floor 51 campfire, 52 shop, 53 elite and its rewards, 54 Heart), which is the supplied
played path of section 15.4.

### 11.3 Laws for rooms M28 completes

Some triggers come from rooms that exist today only in finite profile form or not at all: unknown
rooms (Juzu Bracelet, Tiny Chest's treasure room, Ssserpent Head's gold), map path choice (Wing
Boots), the card-reward rarity roll (N'loth's Gift's tripled Rare chance, Prayer Wheel's second
reward in ordinary combats), treasure chests in Acts 1–3 (Matryoshka, Cursed Key, N'loth's Hungry
Face), events and Neow (the swap and trades of 7.3) and the score (Spirit Poop). M27 implements each
as a named entry point in `items/run_laws.rs` (Main) with its relic bodies in
`items/relic_effects/run.rs`, and constructed tests that call it with the exact
inputs the room will supply; M28 calls these functions when it builds the rooms, and M28's Task 0
checks the list. None of these relics is inert: each law is implemented and tested, and every
owner that exists today calls it under complete admission.

### 11.4 Act 4 producers under complete admission

PM ruling (review v1, I5): under `ItemAdmission::Complete` the existing Act 4 shop and elite rewards
draw their relic and potion identities from M27's general owners; the frozen per-profile lists stay
unchanged for frozen profiles. The accepted row structure stays (which rows exist and the card rows
are as M26 left them: three colour cards at 50/75/150 in the shop, Common/Uncommon/Uncommon plus Skip
after the elite; `research-B2/facts-v2.md` §1); complete shop generation, the full merchant stock and
price laws remain M28's.

| Producer (complete admission) | Accepted today (frozen profiles keep it) | Under complete admission |
| --- | --- | --- |
| Floor 52 shop relic row (one row, price 150) | M10 Brimstone, M15 Twisted Funnel, M20 Runic Capacitor, M25 Melange, each if not owned | `take(tier Shop, Back, source Shop)` (the source's Shop-tier slot uses the end selector, economy research §3.1); price the Shop-tier base 150 |
| Floor 52 shop potion rows (M10 three rows, M15/M20/M25 one row) | Fixed potions at fixed prices | Each row's identity from `rarity_first(character)`; price the source base for its rarity (Common 50, Uncommon 75, Rare 100, economy research §3.2) without the merchant jitter, which is M28's |
| Shop discounts and removal | Accepted purge cost | The Courier ×0.8 then Membership Card ×0.5, each step rounded separately (economy research §3.2); Smiling Mask sets the removal price to 50; the removal precedence rule of §3.2 |
| The Courier's replacement after a purchase | — | Relic: `take(tier roll_tier(Merchant), Back, source Shop)` excluding Old Coin, Smiling Mask, Maw Bank and The Courier, priced at its tier base (150/250/300); potion: `rarity_first`; card: the row's accepted colour sampler (economy research §3.3) |
| Floor 53 elite relic | M10 and M15/M20 finite cursors; M25 uniform among Damaru, Duality, Cloak Clasp | `take(tier roll_tier(Ordinary), Front, source Elite)`; with Black Star a second `take(tier roll_tier(Ordinary), Front, source Elite)` that excludes the campfire relics Girya, Peace Pipe and Shovel (catalog Black Star note) |
| Floor 53 elite potion | Fixed per-profile cycles, no roll | `combat_appearance(state, RewardRoom::Elite, existing_rows, key)` then `rarity_first`; White Beast Statue and Sozu as section 8 |
| Floor 53 elite gold | 25 plus a 0–10 roll | Unchanged, with Golden Idol's bonus (section 11.2) |
| Floor 53 card reward | Colour sampler, Common/Uncommon/Uncommon plus Skip | As M26 left it, with the card-reward shape laws (Question Card one more card from the same sampler, Busted Crown two fewer, Singing Bowl's option, Prismatic Shard switching the sampler to M26's any-colour pool) |

The draws use new RNG scopes appended after M26's last scope (`M27RelicPools`, `M27RelicTier`,
`M27PotionOffer`), keyed by act, floor, node, visit and a source tag, so frozen profiles draw nothing
new.

## 12. Global corrections planning already found

Each is fixed at the shared owner for every character and profile and disclosed in the M27
reconciliation with the original assertion bytes retained. Task M0 confirms each on merged M25
and M26, records which accepted contexts reach it, and lists the accepted assertions to amend
(the "Amended" column is planning's expectation).

| # | Defect | Fix | Amended |
| --- | --- | --- | --- |
| G1 | Sozu's +1 energy is missing: combat preparation adds master energy only for Slaver's Collar and Mark of Pain (`combat/engine.rs` combat entry), while M15 and M20 contexts admit Sozu | One master-energy law at combat preparation adds 1 per owned energy relic (Sozu, Mark of Pain, the new boss energy relics; Slaver's Collar only in elite and boss combats) | Any M15/M20 test pinning energy with Sozu owned |
| G2 | Reward claim and shop purchase write a potion straight into the first empty slot (`run/rooms/rewards.rs`, `run/rooms/shop.rs`) and ignore Sozu | Section 8.2 consumer rules in the shared reward and shop owners | None expected (no accepted test claims a potion with Sozu) |
| G3 | Potion Belt's equip adds no slots; only the validator's 3-or-5 shape knows the relic | Equip appends two empty slots once (8.1) | Any test that builds a five-slot belt by hand still passes; none expected |
| G4 | Relic visits follow module and bridge order (generic, then Silent, then Ironclad, then Ironclad turns; front splices) instead of acquisition order. Known frozen-observable instance: M15 Silent with Twisted Funnel and Bag of Marbles or Red Mask (all three in `M15_RELICS28`): today the enemy always gains Vulnerable or Weak before Poison; under the order law the acquisition order decides. The enemy power order is visible in `public_full` and in the Run observation's ordered power rows (`crates/spire-env/src/run_observation.rs`, descriptor `run_schema.rs`), so the M15 observation bytes of those Runs change: a disclosed frozen-observation change. The other 77 cross-path pairs differ in internal work order (`State` `Debug`) | The single dispatcher (6.2), for every context | Every accepted test that pins the old order and every replayed transcript step that differs, each listed by Task M0 and Main with its pair, step and source rule; observation changes listed separately |
| G5 | 22 item rows are `definition_only: true` although each effect is accepted and active (`content/items.rs` 14 relic rows and 8 potion rows, section 5.1) | Every item row is not definition-only; `m27_identity_catalog_rows` asserts all 221 rows | Any catalog test pinning a flag (Task M0 lists them) |
| G6 | Relic tier and potion rarity metadata exist only for Silent and Defect items (`Catalog::relic_tier`, `potion_rarity`) | One complete metadata table each (5.1) | Any test pinning `None` for another character's item |
| G7 | `EXCLUSIVE_POTION_CHARACTERS` lacks the Watcher's three class potions, so `potion_eligible_for(BottledMiracle, Ironclad)` is `Some(true)` | Add the three rows; every eligibility caller sees 12 class potions | Task M0 records every caller of `potion_eligible_for` and whether an accepted context reaches the wrong answer; any test pinning it |

Task M0 and the gate audit may add rows; each follows the same rule.

## 13. Named source frontiers

Each frontier is bound to a cited fact at Task M0 (research documents, `data/`, accepted tests or
retained static findings). A frontier still unsettled is handled as in M25 and M26: the lane
implements the behaviour the cited sources most directly support, records it as a disclosed
project decision, pins it in a test labelled with the frontier number, and the PM is told before
the freeze. A frontier never becomes an `Unsupported` rejection of a legal action.

| # | Frontier | Settled by | Still to bind |
| --- | --- | --- | --- |
| F1 | Queue side of each relic's work and the order of co-firing top-queued relics | Catalog `local_static.primary_rule` ("queues … at bottom"); relic research §4; 6.1 rule 3 | Every pair co-firing in one phase, with its observable order |
| F2 | Which engine hook is `onUseCard` and which is `onPlayCard` | Relic research §3.5 | The mapping onto `CardPlayed`, `CardAfterUse` and `CardAfterPlayed` |
| F3 | Combat-end relics after a Smoke Bomb escape | Today `SmokeEscape` queues `CombatSettled` and Burning/Black Blood heal on it with no outcome check; relic research §3 (no blanket exclusion either way) | Whether the source runs Burning Blood, Black Blood, Meat on the Bone and Face of Cleric on an escape; a difference is an M15-reaching global correction |
| F4 | Held potions after Sacred Bark is lost | Sacred Bark record (equip re-initializes; no unequip rule recorded) | Confirm potencies stay doubled |
| F5 | Bottled card removed or transformed later | Relic research §3.1 (unequip clears the flag) | A stale link has no effect; a transformed card is a new instance and is not bottled |
| F6 | Potion Belt lost with potions in its slots | Potion Belt record | Which slots are removed and what happens to their potions |
| F7 | Boss-tier exhaustion | Economy research §4.1 (Red Circlet fallback); catalog (Red Circlet unobtainable) | Confirm unreachable; invariant error |
| F8 | Potion effect provenance under complete items | Accepted M20 Energy Potion and M21 Watcher potion sources | Every consumer of `Source::Potion` |
| F9 | Necronomicon with Double Tap, Burst, Echo Form and Necronomicurse | Necronomicon record (first Attack costing 2 or more each turn is played twice) | Order of the repeat against the other duplicators; X-cost and cost-changed cards |
| F10 | Ice Cream with the deferred Defect and Watcher recharge and M26's next-turn energy law | Relic research §2 (Ice Cream precedes Conserve; 999 cap); M26 seam map §2 law | Where unspent energy enters the law (`energized`, `energized_blue`, `energy_down`) |
| F11 | Snecko Eye's Confused cost against cost-changing effects | Snecko Eye record; accepted Confused law (M23 global fix) | Order against Mummified Hand, Enchiridion's free Power and Warped Tongs |
| F12 | Runic Dome and Frozen Eye in the core public view | Relic research §2 (Frozen Eye exposes the actual order only with ownership) | The public-view fields hidden or revealed, and that private state never leaks |
| F13 | Catalog conflicts research-A flags | `research-A/catalog-lifecycle.tsv` | Campfire relic floor limits (Girya, Peace Pipe, Shovel), Mercury Hourglass cutoff, N'loth's Hungry Face relic versus gold, Bloody Idol's source label |
| F14 | Card pools of relic generators | M26 `content::pools` | Dead Branch, Enchiridion, Nilry's Codex and Toolbox pools for each character (player's class pool, colourless) |
| F15 | The seven new relics without a local primary rule (Bloody Idol, Circlet, Enchiridion, Mutagenic Strength, Necronomicon, Nilry's Codex, N'loth's Gift) | Catalog `effect`, `notes`, `state_and_reset` | Timing and queue side of each |

## 14. Native exposure and diagnostics

- **No new public profile or descriptor in M27** (recommended in section 18, as M26 PM ruling 1).
  Every existing descriptor, fingerprint and vocabulary is unchanged; appended `RelicId` variants
  and the complete-only `DiscardPotion` combat action do not enter any existing explicit
  vocabulary. Mechanical new arms in exhaustive matches are allowed in `crates/spire-env`,
  `crates/spire-report` (`actions.rs` `action_type`, new report spelling `discard_potion`) and the
  spire-core examples that match the combat action enum (Task M0 lists them).
- Runic Dome and Frozen Eye change what the core public combat view shows, only under the flag
  (F12). The first profile that publishes complete items (M28 or M34) maps them into its
  descriptor.
- The accepted `visible_counters` view (Neow's Lament and Inserter) is unchanged; the complete
  counter view (`items::complete_counter_view`) is provided for the first publishing profile.
- A default-off `m27-diagnostics` feature in `crates/spire-core` (depending on `m26-diagnostics`)
  provides the constructors core tests use: a Run of each character's complete Heart scenario
  with `CardAdmission::Complete`, `ItemAdmission::Complete` and supplied relics, potions and
  counters validated by the complete validator, built as M26's diagnostics are (a template from
  `Run::new_room_scenario`, the normalization owner, then `Run::from_validated_combat_resources`),
  and the frozen transcript replay example of section 6.2 (`examples/m27_frozen_replay.rs`). No
  public supplied-run API and no arbitrary state importer.

## 15. Evidence

### 15.1 Grades

The M25 grades apply. M27 uses **Constructed** (an owner checkpoint built directly) and
**Supplied played** (a supplied start, then only legal actions through the real owners). It has
no **Normal played** cases and no **Native** agreement runs, because it publishes no profile.

### 15.2 Coverage tests

Every expected value is a literal from section 4, Appendix A or B, or a cited fact, never computed
by the helper under test. Paths start at `crates/spire-core/`.

| Key | File | Covers |
| --- | --- | --- |
| K | `tests/m27_catalog.rs` and `src/items/tests/m27_capability.rs` | Identities, rows, tiers, classes, unions 179/150/150/148/148 and 42/33, the data check against `data/relics/relics.json`, G5–G6 |
| D | `src/items/tests/m27_dispatch.rs` | Order law (6.1): the first test fails on main (M15 Twisted Funnel with Bag of Marbles); the 79 co-firing pairs in both acquisition orders; top and bottom sides; `m27_single_relic_visit_loop` against Appendix C; G4. The frozen transcript replay of 6.2 is evidence run by `examples/m27_frozen_replay.rs` |
| C1 | `src/items/tests/m27_relics_turns.rs` | Lane R1 relics (Appendix A) through real combat entry, turns and exit |
| C2 | `src/items/tests/m27_relics_play.rs` | Lane R2 relics: card use and play, damage, power and prevention seams |
| C3 | `src/run/tests/m27_relics_run.rs` | Lane R3 relics at their Run owners and named laws (11.2–11.3) |
| J | `src/items/tests/m27_relics_structural.rs` and `src/run/tests/m27_relic_selections.rs` | Lane J relics: selections, repeat play, exhaust redirect, legality, observation |
| E | `src/items/tests/m27_existing_relics.rs` | Each of the 62 accepted relics under complete items for its class (and every shared one for all four characters), equal to its accepted behaviour |
| L | `src/run/tests/m27_relic_lifecycle.rs` | Obtain and equip, selections, replacement, loss, Neow swap, pools and their law, spawn conditions, promotion, Circlet, counters across combats; atomic rejections |
| P | `src/combat/tests/m27_potions.rs`, `src/run/tests/m27_potion_lifecycle.rs` and Main's `src/items/tests/m27_potion_owner.rs` | Every potion for every character whose union holds it, base and with Sacred Bark; phase, target and We Meet Again; discard in and out of combat; obtain consumers with Sozu and full belts; factory helpers; appearance law; G1–G3 |
| H | `src/combat/tests/m27_rescue_escape.rs` and `src/items/tests/m27_energy_law.rs` | Healing law order, prevention relics, Fairy and Lizard Tail with and without Mark of the Bloom, Smoke Bomb rules and the escaped combat's settlement |
| X | `src/items/tests/m27_cross_character.rs` | The cross-character item matrix (15.3) |
| I | `src/items/tests/m27_inert.rs` | `m27_capability_overlay_inert` for each Heart context |
| S | `src/run/tests/m27_supplied.rs` | Supplied played Act 4 Runs (15.4) |

### 15.3 The cross-character item matrix

**Basis.** M26's stripped matrix context (M26 seam map §6: Jaw Worm at setup HP, player HP 80/80,
energy and maximum energy 10, five red Defends, `opening_draw` 1) plus
`with_complete_cards().with_complete_items()`, no relic except the one under test, three empty
potion slots except the potion under test.

**Comparison.** For each shared relic and potion and each character, the test records the delta of
the public state over a fixed script (combat start, one turn, the trigger, end of turn, combat end
where relevant) and compares it with the Ironclad delta. Character-sensitive rows have literal
per-character expectations instead: the recharge laws (Ice Cream, the boss energy relics, Sozu),
Prismatic Shard's orb slot for a character without one, the card-generating relics' pools (Dead
Branch, Enchiridion, Nilry's Codex, Toolbox, Orrery, Tiny House) and the potion-generating relics'
class pools (Cauldron, White Beast Statue's offer, Entropic Brew). A difference elsewhere is a
defect fixed under rule 2.

**Inertness.** On the new source, for each Heart context with its own accepted relics and potions,
one scripted combat has equal full state (`State` `Debug` and `public_full`) at every step with and
without `with_complete_items()`.

### 15.4 Supplied played Runs

For each character, a supplied complete Run starts at the Act 4 map of its complete Heart scenario
with a supplied relic and potion set and plays legal actions through the real owners (in combat
the first legal card play, else End Turn; potions only where the script names them; `DiscardPotion`
never unless scripted): the campfire
(rest relics), the shop (shop relics, a potion purchase), the elite and its rewards (combat relics,
reward relics, the appearance law), and the Heart (combat relics, rescue). Counter-bearing relics
are checked across the two combats, the inventory once after each settlement, and a Death in the
Heart combat settles without a fault. The room expectations follow section 11.4: the shop's relic row
is an unowned Shop-tier relic of the character's union taken from the back of the Shop-tier pool (the pool
shrinks by one and the relic is never offered again in the Run) at 150 before discounts; each potion
row is one of the character's 33 potions priced by its rarity; the elite relic comes from the front of
the rolled tier and is consumed; Black Star adds a second, non-campfire relic; the elite potion row is
present exactly when the appearance roll succeeds, and the offset moves by −10 or +10 accordingly. The four supplied sets together hold every relic whose Run
trigger exists in Act 4 and at least twenty combat relics each.

### 15.5 Freeze and preservation

Nothing is frozen until the source is final: every lane RED is resolved with its fix and disclosure,
the gate audit is complete, and the full `m27_` suites, every accepted M10–M26 suite, the training
profile suite, the full workspace suite (`cargo test --workspace --locked --offline --no-fail-fast`)
and the feature-on core suite (`cargo test -p spire-core --features m27-diagnostics --locked
--offline`) pass on the exact tree with no failure other than the inherited, documented ones, and
the frozen transcript replay of section 6.2 on that tree shows only disclosed differences. The
tree's SHA-256 manifest and the suite logs are retained outside the session. There are no once-only
outcome runs.

## 16. Limits

M27 does not claim: any public profile, descriptor or Python surface for the complete item set;
the rooms that offer items (elite, chest, shop and boss rows, events, Neow) beyond the shared owners
of section 11.2 (M28); unknown-room resolution, map path rules and the card-reward rarity schedule
(M28); A1–A20, including the A11 potion slot (M31); Red Circlet; the score screen (M34); original-
runtime or Java RNG parity; any strategy or win. The Run laws of section 11.3 are Constructed until
M28's rooms call them. Constructed and supplied results are never presented as normal traversal.

## 17. M25 and M26 interfaces this design depends on

**Re-review trigger.** At Task M0, before any code, Main diffs each interface below against the
merged M25 and M26 source using the check in the last column and records the result in the seam map.
If every row matches the M25 version 2 contract (with every PM-approved amendment up to the merge)
and the M26 version 3 contract (likewise), or differs
only as the "Accommodated differences" column allows, execution proceeds. Otherwise Main writes a
short M27 amendment (the interface, what changed, the affected sections and tasks, the proposed
change) and sends it through the PM to the same plan reviewer; execution does not continue until
that reviewer returns READY on it.

| Interface | Used by M27 for | Task M0 check against merged source | Accommodated differences |
| --- | --- | --- | --- |
| M25: Damaru, Duality, Cloak Clasp, Melange identities and handlers; the 23-relic M25 union | Section 4 counts; their handlers move behind the dispatcher unchanged | Four variants and handlers present; M25 item tests (`items/tests/m25_watcher.rs`) green | Handler location or name |
| M25: `Hook::DrawPileShuffled` dispatched from the draw-caused shuffle in `combat/piles.rs` (M25 plan C6) | The shuffle seam (6.5) for The Abacus and Sundial | The variant exists and is dispatched after the shuffle a draw caused | A different variant name with the same dispatch point |
| M26: `Hook::AttackResolved` and `Hook::CloakClaspBeforeCleanup{hand_size}` | Reused seams (6.5); no duplicate hook | Variants exist with these dispatch points | Different names |
| M26 seam map §6 fixtures: the stripped matrix context (Jaw Worm, HP 80/80, energy 10, five red Defends) and the unstripped own-colour scripts per character | Bases of tests D, I and X | Present as M26's tests use them | Different helper names, same setup |
| M25: `CombatItemContext::with_m25_profile()` and the M25 Watcher Heart context; `RoomScenario::M25WatcherHeart` | Base of the Watcher complete context and the Act 4 supplied Run | Constructor and scenario exist; Act 4 floors 51–54 as rest, shop, elite, Heart | Different names |
| M25: the M21 potion potency and last-use provenance owner, unchanged | The single potency owner (8.4) | Fields and accessors as on main | None |
| M26: `content::complete` and `content::pools` (`CardPool`, `cards(pool)`) | Unions 5.1 and every card-generating relic | Module paths and signatures as `research-C/m26-interfaces.md` | Function names, same members |
| M26: `CombatItemContext::{with_complete_cards, complete_cards}`; `complete_live_card` | Precondition of `with_complete_items()`; relic-created cards | Present; Debug prints only when true | Different internal names |
| M26: `CardAdmission::{Profile, Complete}` on `RunState`; the variant `CompleteCombatStart::Supplied` and its method `normalized(character)`; `complete_heart_context`; `Run::prepare_complete_combat` | `ItemAdmission` beside it; supplied validation (5.3); the H2 test constructor and diagnostics (14) | Signatures as `research-C/m26-interfaces.md` | A different internal validator split with the same results |
| M26: Run mutation owners `obtain_card`, `remove_card`, `upgrade_card`, `can_upgrade`, `transform_card(.., auto_upgrade, key)`, `duplicate_card`, `draw_card`, `draw_distinct` (`pub(super)` in `run/mutations.rs`) | Every relic card change (7.1) and the card-obtain seam | Signatures and that every obtain path goes through `obtain_card` | Names; extra parameters with defaults; widening their visibility to `pub(crate)` so `items/` can call them (no behaviour change) |
| M26: potion class pools for all four characters and the limited lists; Alchemize for every character | The factory helpers (8.5) | `candidates(character, limited)` for all four; sizes 33 and 32 | Location |
| M26: `RunRngScope` appended scopes | M27 appends `M27RelicPools`, `M27RelicTier` and `M27PotionOffer` after them | Position and spelling of the last scope | None (spellings are RNG keys) |
| M26: `m26-diagnostics` feature and its complete Heart constructors | Base of `m27-diagnostics` | Feature exists | Different constructor names |
| M26: the next-turn energy law (seam map §2) and G7 | Ice Cream and the energy relics (F10) | Law present in all three recharge owners | Different helper names |
| M25 and M26 suites green on the merged tree | Preservation baseline | All `m25_` and `m26_` suites green | Different test names |

## 18. PM rulings on the planning questions

The PM accepted all seven recommendations below
(`/home/klottick/.local/share/spire-gym/evidence/m27-execution-20261031/pm-open-question-rulings-v1.json`,
SHA-256 `29a041e3dae2f11a8422bb852ae96cc3e10c7f7ed6b5fb28e8beb15c94b01574`), and ruled on review v1's
I5: under complete item admission the existing Act 4 shop and elite rewards draw from M27's general
pools and tier rolls, and the frozen per-profile lists stay for frozen profiles (section 11.4). No
question is open.

1. **Public profile.** Recommendation: none in M27, as M26 (core constructed and supplied tests;
   the M28 standard profile or M34 publishes complete items).
2. **Room ownership split.** Recommendation: M27 owns relic and potion pools, the ordinary tier
   roll, the appearance law and the potion and relic producer functions; M28 owns which room offers
   what, chest-size tables, shop slots and prices, unknown-room resolution and map path rules
   (section 1.1).
3. **Red Circlet.** Recommendation: excluded (unobtainable), as Pride was in M26.
4. **Class restriction.** Recommendation: no off-class relics or potions (no normal source gives
   them); every use and effect law is still character-independent.
5. **Single dispatcher for frozen profiles.** Recommendation: yes; one acquisition-order dispatcher
   for every context; frozen evidence is the full-state replay of every retained transcript
   (section 6.2), and every difference is a disclosed global correction (G4), rather than a second
   loop for complete items. (The PM's ruling read "byte-equal replay or disclosed global fix"; review
   v1 I2 and the PM's guidance replaced the byte-equal claim with the full-state replay.)
6. **Combat potion discard.** Recommendation: add it as a complete-only combat action (8.3); frozen
   masks and encoders unchanged.
7. **Opus helper senior.** Recommendation: use one for Lane J (selection and structural relics) in
   sub-phase 2, alongside Main; this keeps Main on shared seams (plan, Lanes).

## 19. Planning record

Version 1, the research reports (`research-A` relic inventory, `research-B` potion inventory and
lifecycle, `research-C` plumbing, trigger taxonomy, deferred sites, precedents and M26 interfaces),
the research briefs, the builder check and decisions log, and `SHA256SUMS` are retained at
`/home/klottick/.local/share/spire-gym/evidence/m27-execution-20261031/planning-v1/`.
Version 2, the review v1 disposition table, the v2 research (`research-A2` relic visit paths,
co-firing pairs, retained transcripts and the full-state surface; `research-B2` Act 4 producers,
action-enum matches, Smoke Bomb, M26 hooks, mutation visibility, item metadata and finite unions)
and `SHA256SUMS` are at
`/home/klottick/.local/share/spire-gym/evidence/m27-execution-20261031/planning-v2/`.

## Appendix A. Per-relic specification

One row per relic of the complete union (179). **Status:** New (M27 adds the identity), M25 (added by M25), Lift (accepted; M27 lifts its gates and completes its lifecycle). **Phases** are the catalog `triggers` mapped to section 6.5's seams. **Lane** owns the handler body and its test (plan, Lanes); Main owns every seam and every Lift and M25 row. **Key** is the coverage test file (15.2). **Rule** is the catalog `local_static.primary_rule` where one exists, otherwise the catalog `effect`; the full rows, notes, restrictions and uncertainties are in `data/relics/relics.json` and `research-A/catalog-lifecycle.tsv`, and the binding detail is the cited research row. Values in this table are planning inputs; Task M0 confirms each against its source.

| Relic | Tier | Class | Status | Phases | Lane | Key | Rule (source summary) |
| --- | --- | --- | --- | --- | --- | --- | --- |
| BurningBlood | Starter | I | Lift | combat end | Main | E | Victory: if current HP>0, heal 6 through heal modifiers/cap. |
| CrackedCore | Starter | D | Lift | combat start | Main | E | At the start of each combat, Channel 1 Lightning. [accepted milestone source] |
| PureWater | Starter | W | Lift | before opening draw | Main | E | At the start of each combat, add 1 Miracle into your hand. [accepted milestone source] |
| RingOfTheSnake | Starter | S | Lift | draw | Main | E | At the start of each combat, draw 2 additional cards. [accepted milestone source] |
| Akabeko | Common | all | New | combat start, card use/play (F2) | R1 | C1 | Battle start queues Vigor 8; NORMAL damage gains its amount until Attack onUse queues removal. Not one flat damage packet. |
| Anchor | Common | all | Lift | combat start | Main | E | Battle start queues Block 10 at bottom; room entry clears used display. |
| AncientTeaSet | Common | all | New | campfire, turn start | R3 | C3 | Rest-room entry arms counter −2; prebattle resets first-turn flag; first turn consumes arm to −1 and queues energy 2 at top. No Rest purchase requi... |
| ArtOfWar | Common | all | New | end of turn, turn start | R1 | C1 | Prebattle arms first turn/no-Attack flag; later turn starts grant energy 1 if prior interval had no Attack onUse, then rearm. First turn grants none. |
| BagOfMarbles | Common | all | Lift | combat start | Main | E | Battle start queues Vulnerable 1 on each monster. |
| BagOfPreparation | Common | all | Lift | draw | Main | E | Battle start queues draw 2. |
| BloodVial | Common | all | Lift | combat start | Main | E | Battle start queues heal 2 at top. |
| TheBoot | Common | all | New | damage given | R2 | C2 | onAttackToChangeDamage: owner-present non-THORNS/non-HP_LOSS positive amount below 5 becomes 5. Not base-damage +5. |
| BronzeScales | Common | all | Lift | combat start, damage received | Main | E | Battle start queues Thorns 3. |
| CentennialPuzzle | Common | all | New | HP loss | R2 | C2 | Prebattle clears once-per-combat latch; first positive combat wasHPLost queues draw 3 and sets used immediately. |
| CeramicFish | Common | all | New | card obtain | R3 | C3 | Actual onObtainCard calls gainGold(9); not offer generation or a global movement hook. |
| Damaru | Common | W | M25 | turn start | Main | E | At the start of your turn, gain 1 Mantra. [accepted milestone source] |
| DataDisk | Common | D | Lift | combat start | Main | E | Start each combat with 1 Focus. [accepted milestone source] |
| DreamCatcher | Common | all | New | campfire | R3 | C3 | Campfire sleep effect, after healing/onRest hooks, opens ordinary reward-helper cards; not every heal/rest-room entry. |
| HappyFlower | Common | all | New | turn start | R1 | C1 | Equip counter 0; each turn increments, 3 resets to 0 and queues energy 1; persists between fights. −1 is specially advanced to 1. |
| JuzuBracelet | Common | all | New | unknown room (11.3) | R3 | C3 | EventHelper converts rolled MONSTER to EVENT, still resets monster chance as that branch dictates; does not prohibit event-authored fights. |
| Lantern | Common | all | Lift | turn start | Main | E | Prebattle rearms first-turn latch; first turn queues energy 1 at top and consumes latch. |
| MawBank | Common | all | New | floor entry, shop | R3 | C3 | Active onEnterRoom gains gold 12; shop onSpendGold permanently sets used counter −2. Not disabled by every nonshop loss. |
| MealTicket | Common | all | New | shop | R3 | C3 | justEnteredRoom on actual ShopRoom heals 15. |
| Nunchaku | Common | all | New | card use/play (F2) | R2 | C2 | Persistent Attack onUse counter: every tenth resets 0 and queues energy 1. |
| OddlySmoothStone | Common | all | Lift | combat start | Main | E | Battle start queues Dexterity 1. |
| Omamori | Common | all | New | card obtain | R3 | C3 | Starts two charges; named curse-obtain constructors consume a charge/prevent there; direct Note path differs. Counter zero marks used. |
| Orichalcum | Common | all | Lift | end of turn | Main | E | End turn: Block zero or explicit trigger flag queues Block 6 at top and clears flag; turn start clears flag. Positive Block-gain callback stops pul... |
| PenNib | Common | all | New | card use/play (F2), damage given | R2 | C2 | Persistent Attack counter; ninth queues Pen Nib power, tenth resets 0. Battle start reinstalls power at counter 9. Power doubles NORMAL damage and ... |
| PotionBelt | Common | all | Lift | equip | Main | E | Equip increases capacity by 2 and appends two empty slots. |
| PreservedInsect | Common | all | New | combat start | R1 | C1 | Battle start with eliteTrigger lowers each initial monster's current HP to at most truncated 75% maxHP, not maxHP itself; no spawn hook. |
| RedSkull | Common | I | Lift | combat start, HP loss | Main | E | Bloodied means integer HP≤half maxHP; above half exits. Battle start clears active; delayed action directly installs Strength 3 and activates if in... |
| RegalPillow | Common | all | New | heal | R3 | C3 | Sleep effect adds 15 to its Rest healing amount; not all healing. |
| SmilingMask | Common | all | New | shop | R3 | C3 | Shop removal price 50 takes precedence in price paths; room callback itself only presents activation. |
| SneckoSkull | Common | S | Lift | power applied | Main | E | Whenever you apply Poison, apply an additional 1 Poison. [accepted milestone source] |
| Strawberry | Common | all | Lift | equip | Main | E | Equip increases maxHP 7, including helper healing. |
| TinyChest | Common | all | New | unknown room (11.3) | R3 | C3 | Equip resets 0; each EventHelper room roll increments; fourth resets 0 and overrides rolled result to TREASURE before chance updates. It still make... |
| ToyOrnithopter | Common | all | Lift | potion | Main | E | Potion use heals 5: queued in COMBAT, immediate outside. |
| Vajra | Common | all | Lift | combat start | Main | E | Battle start queues Strength 1. |
| WarPaint | Common | all | New | equip | R3 | C3 | Equip shuffles canUpgrade Skill instances, upgrades up to first two immediately, refreshes bottle links. Duplicate kinds are allowed. |
| Whetstone | Common | all | New | equip | R3 | C3 | Same equipment prefix rule for canUpgrade Attack instances. |
| BlueCandle | Uncommon | all | New | card use/play (F2) | R2 | C2 | Shared base legality permits otherwise-unplayable curses; curse onUse queues HP_LOSS 1 and marks both card and cleanup exhaust. All other legality/... |
| BottledFlame | Uncommon | all | New | equip, before opening draw | J | J | Typed Attack selection/flag at delayed equipment update; initializeDeck top-placement is shared, not its battle-start flash. |
| BottledLightning | Uncommon | all | New | equip, before opening draw | J | J | Same for Skill. Spawn's nonbasic check is not the equipment list's filter. |
| BottledTornado | Uncommon | all | New | equip, before opening draw | J | J | Same for Power. Unequip clears matching permanent instance flag. |
| TheCourier | Uncommon | all | New | shop | R3 | C3 | Shop price multiplier .8 and restocks bought stock via source-specific paths; room callback only presents activation. |
| DarkstonePeriapt | Uncommon | all | New | card obtain | R3 | C3 | Actual obtain with card color CURSE increases maxHP 6, with helper healing; not prevention/offer alone. |
| Duality | Uncommon | W | M25 | after use | Main | E | Whenever you play an Attack, gain 1 temporary Dexterity. [accepted milestone source] |
| EternalFeather | Uncommon | all | New | campfire | R3 | C3 | onEnterRoom RestRoom heals 3×integer(deck size/5), even if Rest itself is disabled/not chosen. |
| FrozenEgg | Uncommon | all | New | card obtain | R3 | C3 | Matching Power preview/obtain upgrades only canUpgrade and not already upgraded; equip revisits existing reward cards. |
| GoldPlatedCables | Uncommon | D | Lift | orb passives | Main | E | Your rightmost Orb triggers its passive an additional time. [accepted milestone source] |
| GremlinHorn | Uncommon | all | New | death | R2 | C2 | onMonsterDeath requires currentHealth==0 and group not basically dead; queues energy 1 then draw 1 at bottom. Not final-group payoff. |
| HornCleat | Uncommon | all | New | turn start | R1 | C1 | Battle start counter 0; second turn queues Block 14 once, marks spent; victory clears spent display and sets −1. |
| InkBottle | Uncommon | all | New | after use | R2 | C2 | Persistent onUse count; tenth resets 0 and queues draw 1; battle start restores armed display at 9. |
| Kunai | Uncommon | all | New | card use/play (F2) | R2 | C2 | Turn counter 0; each third Attack onUse resets and queues Dexterity 1; victory counter −1. |
| LetterOpener | Uncommon | all | New | card use/play (F2) | R2 | C2 | Turn counter 0; each third Skill onUse queues all-enemy THORNS 5 and resets; victory −1. |
| Matryoshka | Uncommon | all | New | chest | R3 | C3 | Two charges; nonboss chest before base relic decrements charge, adds a common/uncommon relic reward, then marks used if exhausted. Source uses stri... |
| MeatOnTheBone | Uncommon | all | New | combat end | R1 | C1 | endBattle explicitly triggers before ordinary player victory hooks; living HP≤maxHP/2 heals 12 using live state. |
| MercuryHourglass | Uncommon | all | New | turn start | R1 | C1 | Every turn start queues all-enemy THORNS 3. |
| MoltenEgg | Uncommon | all | New | card obtain | R3 | C3 | Frozen Egg rule for Attack; already-upgraded Searing Blow does not receive another acquisition upgrade. |
| MummifiedHand | Uncommon | all | New | card use/play (F2) | R2 | C2 | Power onUse chooses a hand instance with base cost>0, turn cost>0, not free-once, excluding queued cards; sets selected turn cost 0 immediately; em... |
| NinjaScroll | Uncommon | S | Lift | before opening draw | Main | E | At the start each combat, add 3 Shivs into your hand. [accepted milestone source] |
| OrnamentalFan | Uncommon | all | New | card use/play (F2) | R2 | C2 | Turn counter 0; every third Attack onUse queues Block 4 and resets; victory −1. |
| Pantograph | Uncommon | all | New | combat start | R1 | C1 | Battle start with any BOSS enemy queues one heal 25, then returns. Not one heal per boss actor. |
| PaperKrane | Uncommon | S | Lift | damage received | Main | E | Enemies with Weak deal 40% less damage rather than 25%. [accepted milestone source] |
| PaperPhrog | Uncommon | I | Lift | damage given | Main | E | Shared Vulnerable factor 1.75 for nonplayer owner/NORMAL only; §2. |
| Pear | Uncommon | all | Lift | equip | Main | E | Equip increases maxHP 10 with helper healing. |
| QuestionCard | Uncommon | all | New | rewards | R3 | C3 | Reward-count callback adds 1 only at callers using it, not every card generator. |
| SelfFormingClay | Uncommon | I | Lift | HP loss, turn start | Main | E | Each positive combat wasHPLost queues Next Turn Block 3; that power queues its accumulated Block/removal at next start. |
| Shuriken | Uncommon | all | New | card use/play (F2) | R2 | C2 | Turn counter 0; every third Attack onUse queues Strength 1 and resets; victory −1. |
| SingingBowl | Uncommon | all | New | rewards | R3 | C3 | Button onClick increases maxHP 2 (helper heals), then removes its captured reward row; update then closes via Bowl/card-screen paths. Relic update ... |
| StrikeDummy | Uncommon | all | New | damage given | R2 | C2 | atDamageModify adds 3 to cards with STRIKE tag, not all Attack cards or all names containing “Strike.” |
| Sundial | Uncommon | all | New | shuffle | R1 | C1 | Equip 0; every third onShuffle resets 0 and queues energy 2; persists between combats. Actual shuffle notification boundary matters. |
| SymbioticVirus | Uncommon | D | Lift | combat start | Main | E | At the start of each combat, Channel 1 Dark. [accepted milestone source] |
| TeardropLocket | Uncommon | W | Lift | combat start | Main | E | Start each combat in Calm. [accepted milestone source] |
| ToxicEgg | Uncommon | all | New | card obtain | R3 | C3 | Frozen Egg rule for Skill; preview and actual obtain remain separate calls with upgraded guard. |
| WhiteBeastStatue | Uncommon | all | New | rewards | R3 | C3 | Potion chance override precedes four-row suppression and appearance-history mutation; §2. |
| BirdFacedUrn | Rare | all | New | card use/play (F2) | R2 | C2 | Power onUse queues heal 2 at top. |
| Calipers | Rare | all | New | Block decay | Main | H/L | Shared next-turn loss 15 absent Barricade/Blur; no relic counter; §2. |
| CaptainsWheel | Rare | all | New | turn start | R1 | C1 | Battle counter 0; third turn queues Block 18 once, marks spent; victory clears spent state/counter. |
| ChampionsBelt | Rare | I | Lift | power applied | Main | E | ApplyPowerAction, player source/nonself target/Vulnerable/no target Artifact, invokes onTrigger to queue Weak 1. Source notification is before targ... |
| CharonsAshes | Rare | I | Lift | exhaust | Main | E | Every actual exhaust callback queues all-enemy THORNS 3 at top; no Branch-style basically-dead guard in relic. |
| CloakClasp | Rare | W | M25 | end of turn (hand) | Main | E | At the end of your turn, gain 1 Block for each card in your hand. [accepted milestone source] |
| DeadBranch | Rare | all | New | exhaust | R2 | C2 | Actual exhaust, if group not basically dead: selects combat-random card now, queues hand creation at bottom. Selection and insertion are different ... |
| DuVuDoll | Rare | all | New | combat start (curse count of the deck at battle start; plan R1) | R1 | C1 | Equip/master-deck-change recount CURSE-type instances; battle start queues that much Strength if positive. Cached count depends on actual notificat... |
| EmotionChip | Rare | D | Lift | HP loss, turn start | Main | E | If you lost HP during the previous turn, trigger the passive ability of all Orbs at the start of your turn. [accepted milestone source] |
| FossilizedHelix | Rare | all | New | combat start, HP loss | R1 | C1 | Battle start queues Buffer 1; room entry resets display. Positive-damage prevention belongs Buffer, not a postdeath heal. |
| GamblingChip | Rare | all | New | after opening draw | J | J | Pre-draw resets activated; first postdraw marks activated and queues any-number discard/redraw action. Selected cards use manual-discard triggers, ... |
| Ginger | Rare | all | New | power attempt | R2 | C2 | Shared player Weak application prevention before Artifact; §2. |
| Girya | Rare | all | New | campfire, combat start | R3 | C3 | Campfire Lift usable while persistent counter<3; battle start queues Strength equal to nonzero counter. Lift effect, not adding option, increases c... |
| GoldenEye | Rare | W | Lift | scry | Main | E | Whenever you Scry, Scry 2 additional cards. [accepted milestone source] |
| IceCream | Rare | all | New | recharge | Main | H/L | Recharge adds retained energy with source addEnergy cap 999; battle prep clears current energy; §2. |
| IncenseBurner | Rare | all | New | turn start | R1 | C1 | Equip 0; every sixth turn resets 0 and queues Intangible 1; persists across fights; −1 specially advances to 1. |
| LizardTail | Rare | all | New | would die | Main | H/L | Player damage rescue checks unused counter −1 after Fairy; trigger synchronously heals max(1,maxHP/2), marks −2. Heal modifiers remain. |
| MagicFlower | Rare | I | Lift | heal | Main | E | onPlayerHeal in a live COMBAT room rounds requested amount×1.5; outside unchanged. Helper then applies powers/cap. |
| Mango | Rare | all | Lift | equip | Main | E | Equip maxHP +14 with helper healing. |
| OldCoin | Rare | all | New | equip | R3 | C3 | Equip gainGold(300), subject to actual gainGold guards. |
| PeacePipe | Rare | all | New | campfire | R3 | C3 | Adds Toke option enabled only for nonbottled purgeable cards; selected removal retains actual source callbacks. |
| Pocketwatch | Rare | all | New | end of turn, draw | R1 | C1 | Battle resets count/first turn; onPlay counts; later postdraw count≤3 queues draw 3, then clears count. No first-turn bonus. Victory resets −1. |
| PrayerWheel | Rare | all | New | rewards | R3 | C3 | CombatRewardScreen adds a second card offer only in ordinary MonsterRoom, excluding elite/boss and EventRoom fights; later offer sees prior history... |
| Shovel | Rare | all | New | campfire | R3 | C3 | Adds Dig campfire option; Dig's source generates the relic offer/obtain, not the option constructor. |
| StoneCalendar | Rare | all | New | end of turn | R1 | C1 | Battle counter 0; turn starts increment; end of exactly turn 7 queues all-enemy THORNS 52; victory resets −1/pulse. |
| TheSpecimen | Rare | S | Lift | death | Main | E | Whenever an enemy dies, transfer any Poison it has to a random enemy. [accepted milestone source] |
| ThreadAndNeedle | Rare | all | New | combat start | R1 | C1 | Battle start queues Plated Armor 4. |
| Tingsha | Rare | S | Lift | manual discard | Main | E | Whenever you discard a card during your turn, deal 3 damage to a random enemy. [accepted milestone source] |
| Torii | Rare | all | New | damage received | R2 | C2 | onAttacked reduces owner-present non-THORNS/non-HP_LOSS amount 2..5 to 1, before Tungsten's final-loss reduction. |
| ToughBandages | Rare | S | Lift | manual discard | Main | E | Whenever you discard a card during your turn, gain 3 Block. [accepted milestone source] |
| TungstenRod | Rare | all | New | HP loss | R2 | C2 | onLoseHpLast subtracts 1 from positive loss, including applicable HP_LOSS paths; not one reduction per turn. |
| Turnip | Rare | all | New | power attempt | R2 | C2 | Shared player Frail application prevention before Artifact; §2. |
| UnceasingTop | Rare | all | New | hand empty | J | J | Refresh-hand queues draw 1 only with empty pending action list/hand, active turn/combat, no screen/No Draw/temporary disable, and nonempty draw or ... |
| WingBoots | Rare | all | New | map (11.3) | R3 | C3 | Starts three charges; permits otherwise-unconnected same-next-row destinations as checked by map edges; selecting a nonnormal winged connection con... |
| Astrolabe | Boss | all | New | equip | J | J | Up to three purgeable removals/upgraded transforms via immediate-small-group or delayed-three branch; §3.1. |
| BlackBlood | Boss | I | Lift | combat end | Main | E | Boss-relic selection uses instant slot-0 replacement rather than ordinary append; living victory heal 12. Own canSpawn needs Burning Blood; starter... |
| BlackStar | Boss | all | New | rewards | R3 | C3 | Actual MonsterRoomElite adds extra noncampfire relic reward; onVictory/room hooks alone only manage display. EventRoom eliteTrigger is insufficient. |
| BustedCrown | Boss | all | New | turn start, rewards | R3 | C3 | Equip/unequip master energy ±1; reward-count callback subtracts 2 where invoked. [+1 energy (Main, G1 law)] |
| CallingBell | Boss | all | New | equip | J | J | Fixed curse confirmation, then once-only three screenless relic offers after screen-down; §3.1. |
| CoffeeDripper | Boss | all | New | turn start, campfire | R3 | C3 | Master energy ±1 on equip/unequip; disables exact ordinary RestOption, not RestRoom entry effects or all campfire options. [+1 energy (Main, G1 law)] |
| CursedKey | Boss | all | New | turn start, chest | R3 | C3 | Master energy ±1; nonboss onChestOpen schedules random curse obtain; bossChest true produces none. Chest skip produces no open hook. [+1 energy (Main, G1 law)] |
| Ectoplasm | Boss | all | New | turn start, gold | R3 | C3 | Master energy ±1; shared gainGold early return prevents actual gain and gain callbacks. Offers/refunds are still source-defined. [+1 energy (Main, G1 law)] |
| EmptyCage | Boss | all | Lift | equip | Main | E | Up to two object removals with distinct empty/automatic/grid paths; §3.1. |
| FrozenCore | Boss | D | Lift | end of turn | Main | E | Replaces Cracked Core. If you end your turn with any empty Orb slots, Channel 1 Frost. [accepted milestone source] |
| FusionHammer | Boss | all | New | turn start, campfire | R3 | C3 | Master energy ±1; disables exact ordinary SmithOption, not every upgrade producer. [+1 energy (Main, G1 law)] |
| HolyWater | Boss | W | Lift | before opening draw | Main | E | Replaces Pure Water. At the start of each combat, add 3 Miracles into your hand. [accepted milestone source] |
| HoveringKite | Boss | S | Lift | manual discard | Main | E | The first time you discard a card each turn, gain 1 Energy. [accepted milestone source] |
| Inserter | Boss | D | Lift | turn start | Main | E | Every 2 turns, gain 1 Orb slot. [accepted milestone source] |
| MarkOfPain | Boss | I | Lift | after opening draw, turn start | Main | E | Master energy ±1; battle start queues two Wounds in draw with source shuffle/insertion semantics, not hand or permanent deck. |
| NuclearBattery | Boss | D | Lift | combat start | Main | E | At the start of each combat, Channel 1 Plasma. [accepted milestone source] |
| PandorasBox | Boss | all | New | equip | J | J | Iterator-removes starter-tagged Strikes/Defends, generates/preview-hooks one current-character replacement each, then confirmation obtains; §3.1. |
| PhilosophersStone | Boss | all | New | turn start, enemy spawn | Main | H/L | Master energy ±1; battle-start monsters and each spawn notification immediately add Strength 1 and modify-power notification. Split children qualif... [+1 energy (Main, G1 law)] |
| RingOfTheSerpent | Boss | S | Lift | draw | Main | E | Replaces Ring of the Snake. At the start of your turn, draw 1 additional card. [accepted milestone source] |
| RunicCube | Boss | I | Lift | HP loss | Main | E | Every positive combat wasHPLost queues draw 1 at top. |
| RunicDome | Boss | all | New | turn start, public view | J | J | Master energy ±1; shared monster presentation suppresses intent display/tooltips, not intent selection/actions. [+1 energy (Main, G1 law)] |
| RunicPyramid | Boss | all | New | end of turn (hand) | J | J | Suppresses bulk discard/Retain Cards selection, not Ethereal or all end-turn effects; §2. |
| SacredBark | Boss | all | Lift | potion | Main | E | Equip reinitializes held potion data; AbstractPotion potency doubles through its Bark branch, while typed choice/duplication potions retain distinc... |
| SlaversCollar | Boss | all | Lift | turn start, combat end | Main | E | Explicit beforeEnergyPrep tests eliteTrigger/BOSS enemy, temporarily master energy +1; activated victory removes it. No ordinary onEquip energy grant. |
| SneckoEye | Boss | all | New | combat start, draw, card drawn | J | J | Equip/unequip master hand size ±2; prebattle queues Confusion. Randomization is the power's draw behavior, not a universal initial card-cost rewrite. |
| Sozu | Boss | all | Lift | turn start, potion | Main | E | Master energy ±1; potion refusal is caller-specific: reward claim can consume without obtain, store refuses before payment, low-level obtainPotion ... |
| TinyHouse | Boss | all | New | equip | J | J | Equip random upgrade prefix of one, maxHP +5, 50 base-gold reward and one random-identity potion offer, then ordinary-context card reward screen. U... |
| VelvetChoker | Boss | all | New | turn start, card use/play (F2), play legality | J | J | Master energy ±1; battle/turn resets 0; onPlay increments up to 6; canPlay rejects at ≥6; victory −1. Applies through actual legality dispatch, not... [+1 energy (Main, G1 law)] |
| VioletLotus | Boss | W | Lift | stance | Main | E | Whenever you exit Calm, gain an additional Energy. [accepted milestone source] |
| WristBlade | Boss | S | Lift | damage given | Main | E | Attacks that cost 0 deal 4 additional damage. [accepted milestone source] |
| TheAbacus | Shop | all | New | shuffle | R1 | C1 | Every onShuffle queues Block 6 at bottom; source shuffle callbacks determine trigger, not any arbitrary pile movement. |
| Brimstone | Shop | I | Lift | turn start | Main | E | Every turn queues player Strength 2 and each monster Strength 1 at top; top insertion reverses encounter order. |
| Cauldron | Shop | all | New | equip | J | J | Equip creates five flat-player-pool potion offers, opens rewards, removes automatically generated card row; discarded generation can still change p... |
| ChemicalX | Shop | all | Lift | X cost | Main | E | Action-local effect count +2; ten current consumers and explicit gates/payment in §2.1. |
| ClockworkSouvenir | Shop | all | New | combat start | R1 | C1 | Battle start queues Artifact 1. |
| DollysMirror | Shop | all | New | equip | J | J | Selected permanent-card stat-equivalent copy, bottle flags cleared, scheduled obtain; §3.1. |
| FrozenEye | Shop | all | New | public view | J | J | Shared draw/exhaust view sorting bypass and legitimate draw-order visibility; §2. |
| HandDrill | Shop | all | New | Block broken | R2 | C2 | Actual monster damage Block-break notification queues Vulnerable 2, including exact depletion; not generic loseBlock. |
| MedicalKit | Shop | all | New | card use/play (F2) | R2 | C2 | Base legality allows otherwise-unplayable Status; Status onUse marks card/cleanup exhaust, without converting Status type to Skill. |
| Melange | Shop | W | M25 | shuffle | Main | E | Whenever you shuffle your draw pile, Scry 3. [accepted milestone source] |
| MembershipCard | Shop | all | New | shop | R3 | C3 | Shop .5 discount via shop/store paths; purchase immediately affects remaining stock. Room hook itself only presents activation. |
| OrangePellets | Shop | all | New | after use | R2 | C2 | Turn resets Attack/Skill/Power flags; onUse records type; all three queues RemoveDebuffs and immediately resets flags, allowing another triple that... |
| Orrery | Shop | all | New | equip | J | J | Four explicit card reward offers plus the screen's ordinary auto-added offer in shop; five offers share changing reward history, not one five-card ... |
| PrismaticShard | Shop | all | New | rewards, combat start | Main | H/L | Equip grants one master orb slot only if non-Defect and current master slots zero; shared reward helper widens eligible reward colors, not merchant... |
| RunicCapacitor | Shop | D | Lift | combat start | Main | E | Start each combat with 3 additional Orb Slots. [accepted milestone source] |
| SlingOfCourage | Shop | all | New | combat start | R1 | C1 | Battle start with eliteTrigger queues Strength 2; actual elite room type is not required. |
| StrangeSpoon | Shop | all | New | would exhaust | J | J | Conditional single Boolean at exhaust-marked cleanup; purge/Power excluded and actual destination precedence retained; §2. |
| Toolbox | Shop | all | New | before opening draw | J | J | Pre-draw battle hook queues ChooseOneColorless action; choice/generation/cancellation remain real work before ready state. |
| TwistedFunnel | Shop | S | Lift | combat start | Main | E | At the start of each combat, apply 4 Poison to ALL enemies. [accepted milestone source] |
| LeesWaffle | Shop | all | New | equip | R3 | C3 | Equip maxHP +7 (helper also heals), then heal request equal to new maxHP. |
| BloodyIdol | Special | all | New | gold | R3 | C3 | Whenever you gain Gold, heal 5 HP. [no local primary rule (F15)] |
| Circlet | Special | all | New | pool, equip | R3 | C3 | Collect as many as you can. [no local primary rule (F15)] |
| CultistHeadpiece | Special | all | New | combat start | R1 | C1 | Battle-start speech/sound actions only in inspected primary payload; no numeric combat buff. Generic relic ownership still exists. |
| Enchiridion | Special | all | New | before opening draw | R1 | C1 | At the start of each combat, add a random Power card into your hand. It costs 0 for that turn. [no local primary rule (F15)] |
| FaceOfCleric | Special | all | New | combat end | R1 | C1 | Victory increases maxHP 1 with helper healing. |
| GoldenIdol | Special | all | New | rewards | R3 | C3 | Stored non-theft/non-TreasureRoom gold reward bonus, not universal gainGold modifier; §2. |
| GremlinVisage | Special | all | New | combat start | R1 | C1 | Battle start queues player Weak 1 through ApplyPowerAction; Ginger can prevent before Artifact. |
| MarkOfBloom | Special | all | Lift | heal | Main | E | You can no longer heal. [accepted milestone source] |
| MutagenicStrength | Special | all | New | combat start, end of turn | R1 | C1 | Start each combat with 3 Strength. At the end of your first turn, lose 3 Strength. [no local primary rule (F15)] |
| Necronomicon | Special | all | New | equip, card use/play (F2) | J | J | The first Attack played each turn that costs 2 or more is played twice. Upon pickup, obtain a special Curse. [no local primary rule (F15)] |
| NeowsLament | Special | all | Lift | combat start | Main | E | Starts three; each qualifying battle-start callback spends one and sets all initial monsters' current HP to 1, without changing maxHP. Last marks u... |
| NilrysCodex | Special | all | New | end of turn (hand) | J | J | At the end of your turn, you may choose 1 of 3 random cards to insert into your draw pile. [no local primary rule (F15)] |
| NlothsGift | Special | all | New | rarity roll (11.3) | R3 | C3 | Triples the chance of finding Rare cards in eligible combat card rewards. [no local primary rule (F15)] |
| NlothsHungryFace | Special | all | New | chest | R3 | C3 | One charge; nonboss onChestOpenAfter removes first relic reward and its linked item where applicable, then marks −2. Matryoshka/base order matters. |
| OddMushroom | Special | all | New | damage received | R2 | C2 | Shared player Vulnerable NORMAL factor 1.25; §2. |
| RedMask | Special | all | Lift | combat start | Main | E | At the start of each combat, apply 1 Weak to ALL enemies. [accepted milestone source] |
| SpiritPoop | Special | all | New | score (11.3) | R3 | C3 | Full-game score bonus −1 and flag; not Act-1 reward/debuff; §2. |
| SsserpentHead | Special | all | New | unknown room (11.3) | R3 | C3 | Face Trader face; onEnterRoom actual EventRoom gains gold 50 via gainGold. Not The Ssssserpent's reward. |
| WarpedTongs | Special | all | New | after draw | R1 | C1 | Each postdraw queues random hand upgrade: only canUpgrade non-Status candidates, shuffled then first upgraded; empty does nothing. Temporary combat... |

## Appendix B. Per-potion specification

One row per standard potion (42). **Effect** is base → with Sacred Bark, from economy research §6 and the records cited in `research-B/potion-inventory.tsv` (which also gives every use site, gate and pool membership by path and line). **Status** is the planning status on main. Every row is tested by key P for every character whose union holds the potion (section 15.2); Lane P owns the tests and Main owns the use owner.

| Potion | Rarity | Class | Effect (base → Sacred Bark) | Target | Use contexts | Status | M27 change |
| --- | --- | --- | --- | --- | --- | --- | --- |
| FirePotion | Common | Any | 20→40 damage to chosen foe | enemy | combat only | gated | Gates lifted for its union (Class P) |
| BloodPotion | Common | Ironclad | Heal floor(maxHP×20%)→floor(maxHP×40%); no minimum-one clamp | none | combat and outside combat | gated | Gates lifted for its union (Class P) |
| FruitJuice | Rare | Any | Max HP +5→+10, then ordinary heal by that amount | none | combat and outside combat | gated | Gates lifted for its union (Class P) |
| BlockPotion | Common | Any | 12→24 Block | none | combat only | complete-all | Kept; complete lifecycle (8.1–8.5) |
| EnergyPotion | Common | Any | 2→4 Energy | none | combat only | complete-all | Kept; complete lifecycle (8.1–8.5) |
| StrengthPotion | Common | Any | 2→4 Strength | none | combat only | gated | Gates lifted for its union (Class P) |
| SwiftPotion | Common | Any | Draw 3→6 cards | none | combat only | gated | Gates lifted for its union (Class P) |
| WeakPotion | Common | Any | 3→6 Weak | enemy | combat only | gated | Gates lifted for its union (Class P) |
| FearPotion | Common | Any | 3→6 Vulnerable | enemy | combat only | gated | Gates lifted for its union (Class P) |
| DexterityPotion | Common | Any | 2→4 Dexterity | none | combat only | complete-all | Kept; complete lifecycle (8.1–8.5) |
| AncientPotion | Uncommon | Any | 1→2 Artifact charges | none | combat only | complete-all | Kept; complete lifecycle (8.1–8.5) |
| Elixir | Uncommon | Ironclad | Choose any hand cards, including zero, to Exhaust; Bark unchanged | none | combat only | gated | Gates lifted for its union (Class P) |
| HeartOfIron | Rare | Ironclad | 6→12 Metallicize | none | combat only | gated | Gates lifted for its union (Class P) |
| PoisonPotion | Common | Silent | 6→12 Poison | enemy | combat only | gated | Gates lifted for its union (Class P) |
| CunningPotion | Uncommon | Silent | Create 3→6 upgraded Shivs | none | combat only | gated | Gates lifted for its union (Class P) |
| GhostInAJar | Rare | Silent | 1→2 Intangible turns | none | combat only | gated | Gates lifted for its union (Class P) |
| SmokeBomb | Rare | Any | Escape; Sacred Bark does not amplify it | none | combat only; adds escape restrictions | gated | Gates lifted for its union (Class P) |
| AttackPotion | Common | Any | Offer 3 distinct Attacks; chosen identity 1→2 copies | none | combat only | gated | Gates lifted for its union (Class P) |
| SkillPotion | Common | Any | Offer 3 distinct Skills; chosen identity 1→2 copies | none | combat only | gated | Gates lifted for its union (Class P) |
| PowerPotion | Common | Any | Offer 3 distinct Powers; chosen identity 1→2 copies | none | combat only | gated | Gates lifted for its union (Class P) |
| ColorlessPotion | Common | Any | Offer 3 distinct ordinary non-healing Colorless cards (economy research §6); chosen identity 1→2 copies | none | combat only | gated | Gates lifted for its union (Class P) |
| FlexPotion | Common | Any | 5→10 Strength now, then 5→10 Lose Strength | none | combat only | gated | Gates lifted for its union (Class P) |
| SpeedPotion | Common | Any | 5→10 Dexterity now, then 5→10 Lose Dexterity | none | combat only | gated | Gates lifted for its union (Class P) |
| RegenPotion | Uncommon | Any | 5→10 Regeneration | none | combat only | gated | Gates lifted for its union (Class P) |
| LiquidBronze | Uncommon | Any | 3→6 Thorns | none | combat only | gated | Gates lifted for its union (Class P) |
| EssenceOfSteel | Uncommon | Any | 4→8 Plated Armor | none | combat only | gated | Gates lifted for its union (Class P) |
| CultistPotion | Rare | Any | 1→2 Ritual | none | combat only | gated | Gates lifted for its union (Class P) |
| BlessingOfTheForge | Common | Any | Upgrade current upgradeable hand cards once; Bark unchanged | none | combat only | gated | Gates lifted for its union (Class P) |
| SneckoOil | Rare | Any | Draw 5→10, then randomize resulting hand costs 0–3; range unchanged | none | combat only | gated | Gates lifted for its union (Class P) |
| FairyPotion | Rare | Any | Automatic rescue heal max(1,floor(maxHP×30%→60%)); never manual | none (automatic self rescue) | automatic only | partial | Lizard Tail order and gates (9.3) |
| ExplosivePotion | Common | Any | 10→20 all-enemy damage | all | combat only | gated | Gates lifted for its union (Class P) |
| GamblersBrew | Uncommon | Any | Choose any number of hand cards including zero to discard, then draw same count; Bark unchanged | none | combat only | gated | Gates lifted for its union (Class P) |
| LiquidMemories | Uncommon | Any | Retrieve 1→2 discard cards, requested cost 0 | none | combat only | gated | Gates lifted for its union (Class P) |
| DuplicationPotion | Uncommon | Any | 1→2 Duplication charges | none | combat only | gated | Gates lifted for its union (Class P) |
| DistilledChaos | Uncommon | Any | Play top 3→6 cards | none | combat only | gated | Gates lifted for its union (Class P) |
| EntropicBrew | Rare | Any | One limited potion proposal per live belt slot in combat; outside unrestricted; Bark does not multiply count | none | combat + outside combat | gated | Gates lifted for its union (Class P) |
| FocusPotion | Common | Defect | 2→4 Focus | none | combat only | gated | Gates lifted for its union (Class P) |
| PotionOfCapacity | Uncommon | Defect | 2→4 orb slots, maximum 10 | none | combat only | gated | Gates lifted for its union (Class P) |
| EssenceOfDarkness | Rare | Defect | Channel 1→2 Dark per current slot | none | combat only | gated | Gates lifted for its union (Class P) |
| StancePotion | Uncommon | Watcher | Choose Wrath or Calm, Wrath offered first; zero potency | none | combat only; Watcher choice | gated | Gates lifted for its union (Class P) |
| Ambrosia | Rare | Watcher | Request one Divinity transition; metadata potency 2→4 is not transition count | none | combat only; Watcher effect | gated | Gates lifted for its union (Class P) |
| BottledMiracle | Common | Watcher | Create 2→4 Miracles | none | combat only; Watcher effect | gated | Gates lifted for its union (Class P) |

## Appendix C. Existing relic visit and bridge paths

Every function on main (and the M26 tree, prefixed `m26tree:`) that reads owned relics to produce or modify gameplay, from `planning-v2/research-A2/relic-visit-paths.tsv` (paths relative to `crates/spire-core/src/`; that file has callers, queue placement and co-firing notes). **D** = replaced by the single visit loop in `items/dispatch.rs` (its per-relic work becomes a handler); **L** = a single-relic modifier that becomes one `items/laws.rs` call at its owner site; **O** = owner policy that reads ownership but is not a relic visit (obtain, replacement, admission, validators, room producers), which stays with its owner under `items/lifecycle.rs`, `items/relic_pools.rs` or the room. Counts: D 49, L 26, O 46 (121 paths). Task M0 re-verifies the list on merged M25 and M26 and publishes it in H0; Task M2's `m27_single_relic_visit_loop` fails if any relic visit remains outside `dispatch.rs`, `laws.rs`, `run_laws.rs` and the O rows.

| Path | Function | Phase or hook | Relics | Disposition |
| --- | --- | --- | --- | --- |
| `items/relics.rs:9` | `on_run_hook` | Hook::CombatSettled | BurningBlood, BlackBlood, plus ordinary heal modifiers | D |
| `items/relics.rs:18` | `on_combat_hook` | CombatStarted, CombatStartedPreDraw, TurnStarted, PlayerEndT | Orichalcum, Lantern, NeowsLament, Anchor, HolyWater, TeardropLocket, DataDisk, Vajra, Oddl | D |
| `items/relics.rs:133` | `battle_start_enemy_effects` | CombatStarted enemy setup | RedMask, BagOfMarbles | D |
| `items/relics.rs:85` | `acquire` | Run relic obtain / replacement | all acquired ids; duplicate/character checks; BlackBlood, RingOfTheSerpent, FrozenCore, Ho | D |
| `items/silent.rs:9` | `on_combat_hook` | Hook::TurnStarted; Hook::CardManuallyDiscarded | ToughBandages, Tingsha, HoveringKite | D |
| `items/silent.rs:40` | `battle_start_enemy_effects` | CombatStarted enemy setup | TwistedFunnel | D |
| `items/silent.rs:51` | `pre_draw_work` | Combat pre-draw/opening hand | NinjaScroll | D |
| `items/silent.rs:61` | `death_work` | Hook::EnemyDied / poison death | TheSpecimen | D |
| `items/ironclad.rs:8` | `on_hook` | CombatStarted, TurnStartedPreDraw, CombatSettled | RedSkull, SelfFormingClay | D |
| `items/ironclad.rs:25` | `on_damage` | positive player HP loss | SelfFormingClay; delegates RedSkull state threshold | D |
| `items/ironclad.rs:47` | `on_exhaust` | card exhaust | CharonsAshes | D |
| `items/ironclad_turns.rs:9` | `relic_hook` | CombatStarted, TurnStartedPreDraw | MarkOfPain, Brimstone | D |
| `items/ironclad_turns.rs:27` | `brimstone_enemy_work` | Hook::TurnStartedPreDraw enemy pass | Brimstone | D |
| `items/ironclad_turns.rs:38` | `runic_cube_draw` | positive HP-loss event | RunicCube | D |
| `items/mod.rs:107` | `take_from_run` | entering combat | entire RunState relic inventory | D |
| `items/mod.rs:114` | `return_to_run` | settlement ownership return | entire combat relic inventory | D |
| `items/mod.rs:281` | `relic_hook_effects` | all shared combat Hook callbacks | delegates relics, Silent, Ironclad and Ironclad-turn listeners | D |
| `items/mod.rs:291` | `relic_damage_effects` | positive/negative damage callback | delegates SelfFormingClay and RedSkull HP-state callback | D |
| `items/mod.rs:293` | `relic_exhaust_effects` | card exhaust | CharonsAshes | D |
| `items/mod.rs:295` | `brimstone_enemy_work` | player turn-start enemy pass | Brimstone | D |
| `items/mod.rs:296` | `runic_cube_draw` | positive HP-loss event | RunicCube | D |
| `items/mod.rs:298` | `battle_start_enemy_effects` | CombatStarted enemy setup | RedMask, BagOfMarbles, TwistedFunnel | D |
| `items/mod.rs:306` | `silent_death_work` | poisoned enemy death | TheSpecimen | D |
| `items/mod.rs:313` | `silent_pre_draw_work` | Combat pre-draw | NinjaScroll | D |
| `items/mod.rs:424` | `prepare_combat_potion_inner` | combat potion use | ToyOrnithopter; heal modifiers indirectly | D |
| `items/mod.rs:499` | `apply_accepted_run_potion_obtain` | generated Run potion obtain | Sozu | D |
| `items/mod.rs:522` | `use_m15_entropic_brew_outside` | outside-combat Entropic Brew | Sozu | D |
| `items/mod.rs:620` | `run_potion_use_heal` | outside-combat potion-use hook | ToyOrnithopter plus ordinary heal modifiers | D |
| `items/mod.rs:626` | `apply_run_potion_use_hook` | outside-combat potion consumed | ToyOrnithopter | D |
| `items/mod.rs:640` | `defect_pre_battle_effects` | Defect pre-battle relic pass | SymbioticVirus, NuclearBattery, RunicCapacitor | D |
| `items/mod.rs:653` | `defect_turn_start_effects` | Hook::TurnStartedPreDraw | RunicCapacitor, Inserter | D |
| `items/mod.rs:674` | `defect_end_turn_work` | Hook::PlayerPreEndTurnCards | FrozenCore | D |
| `combat/effects.rs:113` | `apply_effect_inner` | Apply power; potion obtain | EmotionChip, Sozu | D |
| `combat/exhaust.rs:17` | `exhaust dispatch` | card exhaust | CharonsAshes via items::relic_exhaust_effects | D |
| `combat/piles.rs:426` | `discard_selected_card_ordered` | Hook::CardManuallyDiscarded | ToughBandages, Tingsha, HoveringKite | D |
| `combat/characters.rs:162` | `passive_group` | natural/start/end/all orb passive phases | GoldPlatedCables via cables | D |
| `combat/cards/defect.rs:322` | `turn_started` | Hook::TurnStarted / Defect passives | EmotionChip, GoldPlatedCables | D |
| `combat/cards/watcher.rs:37` | `stance_transition` | Calm exit | VioletLotus | D |
| `combat/engine.rs:421` | `Combat::enter_with_item_context` | Combat startup and owned-inventory admission | RingOfTheSerpent, SlaversCollar, MarkOfPain; all inventory passed to context | D |
| `run/rooms/rewards.rs:100` | `choose` | relic reward/choice commit | state.relics for duplicate/repeat validation and accepted capture | D |
| `run/rooms/shop.rs:72` | `choose` | shop purchase | state.relics indirectly through acquire_relic duplicate/replacement checks | D |
| `run/rooms/boss.rs:37` | `choose` | boss relic choice | state.relics | D |
| `run/rooms/events/act3.rs:36` | `choose` | Tomb of Lord Red Mask reward | RedMask | D |
| `run/rooms/events/act3.rs:62` | `drive` | Tomb of Lord Red Mask continuation | RedMask | D |
| `m26tree:items/relics.rs:18` | `on_combat_hook` | PlayerEndTurn, TurnStartedPreDraw, AttackResolved, DrawPileS | CloakClasp, Damaru, Duality, Melange, GoldenEye, Orichalcum plus existing hook relics | D |
| `m26tree:combat/hooks.rs:21` | `dispatch` | Hook::TurnStartedPreDraw, PlayerPreEndTurnCards, Death, Hook | Defect RunicCapacitor/Inserter/FrozenCore and Silent TheSpecimen callbacks | D |
| `m26tree:combat/turns.rs:92` | `apply_effect` | EffectKind::EndPlayerTurn before hand cleanup | CloakClasp; Orichalcum interaction | D |
| `m26tree:combat/piles.rs:35` | `notify_draw_pile_shuffled` | Hook::DrawPileShuffled | Melange plus GoldenEye amount modifier | D |
| `combat/hooks.rs:20` | `dispatch` | TurnStartedPreDraw, PlayerPreEndTurnCards, Death | RunicCapacitor, Inserter, FrozenCore, TheSpecimen; M15 profile gate | D |
| `items/ironclad.rs:33` | `on_hp_state` | player HP state change | RedSkull | L |
| `items/mod.rs:90` | `consume_fairy_for_rescue` | lethal-damage rescue | MarkOfBloom, SacredBark | L |
| `items/mod.rs:124` | `reinitialize_potion_potencies` | Potion use setup | SacredBark indirectly via initial_potency | L |
| `items/mod.rs:128` | `initial_potency` | M21 potion creation/use | SacredBark | L |
| `items/mod.rs:270` | `ordinary_heal_amount_in_room` | heal/repair amount computation | MarkOfBloom, MagicFlower | L |
| `items/mod.rs:292` | `relic_hp_state_effects` | HP state change / healing update | RedSkull | L |
| `items/mod.rs:578` | `prepare_run_potion_inner` | outside-combat potion use | ToyOrnithopter, MarkOfBloom, MagicFlower | L |
| `items/potions.rs:19` | `use_potion_inner` | potion use | SacredBark | L |
| `combat/generation.rs:227` | `potion_copies` | generated potion copy count | SacredBark | L |
| `combat/powers.rs:117` | `apply_effect` | Apply poison / Vulnerable effect | SneckoSkull, ChampionsBelt | L |
| `combat/effects.rs:48` | `refresh_enemy_values` | enemy power value refresh | PaperKrane | L |
| `combat/effects.rs:746` | `heal_player` | combat healing | MarkOfBloom, MagicFlower | L |
| `combat/characters.rs:158` | `cables` | Defect orb passive | GoldPlatedCables | L |
| `combat/cards/ironclad.rs:93` | `payload (Whirlwind branch)` | X-cost attack payload construction | ChemicalX | L |
| `combat/cards/defect/state.rs:11` | `payload` | X-cost Defect card payload | ChemicalX | L |
| `combat/cards/mod.rs:441` | `payload_with_wrist_blade` | Attack payload construction | WristBlade | L |
| `combat/cards/mod.rs:498` | `normal_damage_pipeline` | normal card damage | PaperPhrog; WristBlade bonus via context field | L |
| `combat/cards/mod.rs:526` | `normal_potion_damage` | potion damage | PaperPhrog | L |
| `combat/cards/watcher.rs:152` | `payload` | scry/foresight generation | GoldenEye | L |
| `combat/cards/watcher.rs:351` | `hook` | Hook::TurnStartedPreDraw | GoldenEye | L |
| `run/mutations.rs:36` | `heal` | Rest-site heal | MarkOfBloom, MagicFlower | L |
| `m26tree:combat/cards/watcher.rs:500` | `scry` | Scry from card/relic | GoldenEye, Melange | L |
| `combat/effects.rs:768` | `ordinary_heal_now` | combat heal/repair | MarkOfBloom, MagicFlower via ordinary_heal_amount_in_room | L |
| `m26tree:combat/cards/watcher.rs:357` | `payload` | ThirdEye/CutThroughFate/JustLucky scry payload | GoldenEye | L |
| `m26tree:combat/cards/watcher.rs:632` | `hook` | Hook::TurnStartedPreDraw | GoldenEye | L |
| `combat/cards/silent/x_repeat.rs:11` | `chemical_x_unavailable` | Silent X-cost repeat payload | ChemicalX | L |
| `items/relics.rs:142` | `replace_burning_blood` | Boss relic replacement | BurningBlood, BlackBlood | O |
| `items/relics.rs:147` | `replace_ring_of_the_snake` | Boss relic replacement | RingOfTheSnake, RingOfTheSerpent | O |
| `items/relics.rs:156` | `replace_cracked_core` | Boss relic replacement | CrackedCore, FrozenCore | O |
| `items/relics.rs:164` | `replace_pure_water` | Boss relic replacement | PureWater, HolyWater | O |
| `items/mod.rs:98` | `CombatItemContext::new` | inventory context construction | NeowsLament, Inserter | O |
| `items/mod.rs:144` | `owns_relic` | shared ownership predicate | any RelicId | O |
| `items/mod.rs:201` | `valid_potion_capacity` | inventory validation | PotionBelt | O |
| `items/mod.rs:205` | `validate_potion_obtain_input` | potion obtain validation | PotionBelt via valid_potion_capacity | O |
| `items/mod.rs:327` | `validate_combat_inventory` | combat entry admission | finite profile relic unions, duplicates, replacement pairs, character/profile gates | O |
| `items/mod.rs:558` | `validate_m15_entropic_brew_outside` | outside-combat potion validation | PotionBelt | O |
| `items/mod.rs:682` | `validate_defect_relic_counters` | M20 inventory validation | Inserter | O |
| `combat/cards/defect.rs:167` | `validate_work` | EssenceOfDarkness potency validation | SacredBark | O |
| `combat/cards/defect.rs:400` | `validate_character` | M20 Slots, passive group and potion capacity effects | RunicCapacitor, Inserter, EmotionChip, SacredBark | O |
| `combat/cards/defect.rs:445` | `capped_energy_source` | M20 variable Energy / EnergyPotion | SacredBark | O |
| `combat/cards/watcher.rs:107` | `validate` | Violet Lotus energy payload | VioletLotus | O |
| `combat/engine.rs:1382` | `valid_potion_slot` | potion slot admission | PotionBelt | O |
| `combat/engine.rs:1411` | `validate_effect` | exhaust/death/power payload validation | CharonsAshes, TheSpecimen | O |
| `combat/engine.rs:2073` | `validate_runtime` | decision and terminal-state validation | PaperKrane, BurningBlood, BlackBlood, MarkOfBloom, MagicFlower | O |
| `run/rooms/rewards.rs:9` | `capture_with_extra` | relic reward selection | owned relic slice against finite pool | O |
| `run/rooms/rewards.rs:13` | `capture_elite_relic` | elite relic capture | owned relic slice | O |
| `run/rooms/rewards.rs:21` | `capture_m10_chest_relic` | chest relic capture | owned relic slice | O |
| `run/rooms/rewards.rs:24` | `prepare_combat` | combat reward screen preparation | active CombatItemContext inventory or RunState relics; M23/M22/M20/M15 content pools | O |
| `run/rooms/rewards.rs:203` | `capture_m22_relic` | M22 relic selection | owned relic slice | O |
| `run/rooms/rewards.rs:207` | `capture_relic_in_scope` | Run-scoped relic selection | owned relic slice plus RNG scope | O |
| `run/rooms/shop.rs:7` | `screen_with_rng_at` | shop screen generation | state.relics across finite captured and M25 shop rows | O |
| `run/rooms/boss.rs:80` | `select_cage` | EmptyCage card removal | EmptyCage plus current deck; relic single check | O |
| `run/rooms/mod.rs:45` | `public` | duplicate relic choice disabled state | state.relics | O |
| `run/rooms/events/act3.rs:10` | `prepare` | Tomb of Lord Red Mask setup | RedMask | O |
| `run/rooms/events/act3.rs:33` | `check_source` | Tomb of Lord Red Mask | RedMask | O |
| `run/training.rs:25` | `CombatTrainingEntry::validate_encounters` | training loadout validation | BurningBlood, BlackBlood, NeowsLament and finite training relic union | O |
| `run/training.rs:64` | `prepare_validated_training` | counter initialization | NeowsLament via training relic list | O |
| `run/rooms/m12_content.rs:39` | `elite_relic` | Silent elite reward pool | owned relic slice | O |
| `run/rooms/m14_content.rs:32` | `elite_relic` | Silent Act3 reward pool | owned relic slice | O |
| `run/rooms/m15_content.rs:235` | `capture_relic` | Silent Heart profile reward pool | owned relic slice | O |
| `run/rooms/m17_content.rs:39` | `elite_relic` | Defect elite reward pool | owned relic slice | O |
| `run/rooms/m19_content.rs:31` | `elite_relic` | Defect Act3 reward pool | owned relic slice | O |
| `run/rooms/m20_content.rs:45` | `normal_relic` | Defect Heart reward pool | owned relic slice | O |
| `m26tree:run/rooms/m25_content.rs:72` | `act4_shop_rows` | Act4 shop relic rows | owned relic inventory | O |
| `m26tree:combat/cards/watcher.rs:225` | `validate` | M26 relic scry payload validation | Melange, VioletLotus | O |
| `m26tree:combat/cards/watcher.rs:532` | `validate_selection` | Melange scry choice validation | Melange | O |
| `combat/cards/defect.rs:265` | `valid_focus_application` | FocusPotion / DataDisk Focus application | SacredBark, DataDisk | O |
| `run/rooms/rewards.rs:14` | `capture_m9_elite_relic` | M9 elite relic capture | owned relic slice | O |
| `run/rooms/rewards.rs:17` | `capture_m10_elite_relic` | M10 elite relic capture | owned relic slice | O |
| `run/rooms/m12_content.rs:38` | `elite_relic_with_exhaustion` | M12/M13 Silent elite relic capture | owned relic slice | O |
| `run/rooms/m13_content.rs:12` | `elite_relic` | M13 Silent elite relic capture | owned relic slice | O |
| `run/rooms/m18_content.rs:9` | `elite_relic` | M18 Defect elite relic capture | owned relic slice | O |
