# Act 1 normative economy, rewards, chests and potions

Date: 2026-10-19. Author issue: `spire-gli.2.3.24` (B08).
**Draft specification; independent review pending. No implementation approval.**

This is a standalone set of source-qualified records for standard, unmodded,
fully unlocked, Act-4-enabled **Ironclad A0 Act 1**. It supplies actual amounts,
eligibility and mutation boundaries, not just a content roster. Event-specific
payments and Neow reward quantities are outside this document. Legal off-color
cards do not change the run's character, starting resources or exclusive pools.

## 1. Authority, evidence and numeric vocabulary

The owned archive is identified by SHA256
`cfad868ac8d65a88e71a0bf096fb09f78811e553effe0787c5309a655e081673`.
Claims below are **static source/bytecode facts** for those bytes, or expressly
labelled mathematical consequences. No game class, initializer, gameplay probe,
reference simulator, profile or original runtime was executed. Independent Python
arithmetic/set checks are not game tests. Future acceptance cases remain unexecuted.

Evidence anchors are class/member locators, not required private imports:

| Key | Inspected authority |
|---|---|
| ECON-CARD | `AbstractDungeon.initializeCardPools/getRewardCards/getColorlessRewardCards/rollRarity/getCardFromPool`; `CardLibrary` rarity-only selection; `CardGroup.getRandomCard`; `AbstractRoom.getCardRarity`; Exordium chance initialization |
| ECON-SHOP | `Merchant` constructor; `ShopScreen.initCards/initRelics/initPotions/applyDiscount/setPrice/purgeCard/updatePurge/purchaseCard`; `StoreRelic.purchaseRelic`; `StorePotion.purchasePotion`; card/potion/relic `getPrice` |
| ECON-REWARD | `AbstractRoom.update/addPotionToRewards/addGoldToRewards`; `MonsterRoomElite.dropReward`; `CombatRewardScreen.setupItemReward`; `RewardItem` constructors/claim/increment; `CardRewardScreen`; `SingingBowlButton.onClick` |
| ECON-RELIC | `AbstractDungeon.initializeRelicList/returnRandomRelicKey/returnRandomRelicEnd/returnRandomScreenlessRelic/returnRandomNonCampfireRelic`; `RelicLibrary.populateRelicPool`; named relic spawn/hooks |
| ECON-CHEST | `AbstractDungeon.getRandomChest`; Small/Medium/Large/Abstract/Boss chest constructors/open; `CampfireDigEffect.update`; Matryoshka, Cursed Key and Hungry Face hooks; `ObtainKeyEffect.update` |
| POT-CATALOG | `AbstractDungeon.initializePotions` → `PotionHelper.initialize(player.chosenClass)` → `getPotions/getPotion`; all 33 eligible constructors, `initializeData`, `use`, `getPotency`; override inventory and Blood/Fruit `canUse` bytecode |
| POT-CONSUMER | `AbstractPotion.canUse/canDiscard/getPotency`; `PotionPopUp.updateInput/updateTargetMode`; `AbstractPlayer.obtainPotion/damage`; `TopPanel.destroyPotion`; `ObtainPotionAction/ObtainPotionEffect.update`; named action/power consumers in §§6–8 |

These reuse retained acquisition, generation, execution and relic inspections;
only genuinely missing potion use/potency and consumer members were newly inspected.
The private claim ledger records exact member paths, hashes and limits. The public
rules do not require those files to supply any omitted quantity.

**Numeric domains.** Source amounts, HP, prices, counts and counters are signed
32-bit Java integers; source float fields and operations are IEEE-754 **binary32**.
An integer-to-float conversion can itself round. Let `F(x)` mean binary32 rounding;
each indicated multiply, subtract and add rounds separately. `T(x)` means Java
float-to-int narrowing (toward zero for finite in-range values). Positive values
therefore truncate downward, but this is not a general integer-floor type.

`J(x)` denotes Java `Math.round(float)`, nearest integer with half ties toward
positive infinity. `G(x)` denotes the actual libGDX `MathUtils.round(float)`:
conversion of the binary64 value `x + 16384.5` to signed int, followed by signed-int
subtraction of 16384. In ordinary positive price ranges it is nearest integer with
upward half ties. Do not replace it with binary32 addition of `.5`, Python's
ties-to-even round, or assert its convenient range identity for arbitrary i32 input.

For a source float interval define the finite arithmetic description
`L(a,b,k) = F(a + F(F(k/2^24) × F(b−a)))`, with binary32 endpoints `a,b` and
integer `0 ≤ k < 2^24`. The single-draw law uses equal grid-index mass `1/2^24`,
not a continuous real interval and not a uniform integer-price range. Rounding
can reach an apparent upper endpoint. The exact amount mass is the number of
grid indices mapping to that amount divided by `2^24`. Integer random endpoints
below are **inclusive**. These descriptions state local primitive pushforwards;
they do not prove a posterior over original root seeds after arbitrary history (§9).

Native numeric/allocation exhaustion remains an administrative fault, not a gold,
inventory, retry, turn or content cap. This is a native contract constraint, not a
claim that Java overflow is mathematically unbounded or a new source policy.

## 2. Card offers and combat rewards

### 2.1 Permanent card supports

Ordinary permanent Ironclad acquisition contains **72** nonbasic IDs. It does not
apply the combat-generator HEALING exclusion. The following partition also fixes
merchant typed supports; repeated deck ownership does not deplete these lists.

| Rarity/type | Exact identities |
|---|---|
| Common Attack (14) | Anger; Body Slam; Clash; Cleave; Clothesline; Headbutt; Heavy Blade; Iron Wave; Perfected Strike; Pommel Strike; Sword Boomerang; Thunderclap; Twin Strike; Wild Strike |
| Common Skill (6) | Armaments; Flex; Havoc; Shrug It Off; True Grit; Warcry |
| Uncommon Attack (11) | Blood for Blood; Carnage; Dropkick; Hemokinesis; Pummel; Rampage; Reckless Charge; Searing Blow; Sever Soul; Uppercut; Whirlwind |
| Uncommon Skill (17) | Battle Trance; Bloodletting; Burning Pact; Disarm; Dual Wield; Entrench; Flame Barrier; Ghostly Armor; Infernal Blade; Intimidate; Power Through; Rage; Second Wind; Seeing Red; Sentinel; Shockwave; Spot Weakness |
| Uncommon Power (8) | Combust; Dark Embrace; Evolve; Feel No Pain; Fire Breathing; Inflame; Metallicize; Rupture |
| Rare Attack (5) | Bludgeon; Feed; Fiend Fire; Immolate; Reaper |
| Rare Skill (5) | Double Tap; Exhume; Impervious; Limit Break; Offering |
| Rare Power (6) | Barricade; Berserk; Brutality; Corruption; Demon Form; Juggernaut |

Ordinary colorless acquisition contains **35** IDs:

| Rarity | Exact identities |
|---|---|
| Uncommon (20) | Bandage Up; Blind; Dark Shackles; Deep Breath; Discovery; Dramatic Entrance; Enlightenment; Finesse; Flash of Steel; Forethought; Good Instincts; Impatience; Jack of All Trades; Madness; Mind Blast; Panacea; Panic Button; Purity; Swift Strike; Trip |
| Rare (15) | Apotheosis; Chrysalis; Hand of Greed; Magnetism; Master of Strategy; Mayhem; Metamorphosis; Panache; Sadistic Nature; Secret Technique; Secret Weapon; The Bomb; Thinking Ahead; Transmutation; Violence |

Shard changes the ordinary **reward** identity selector to the union of unlocked
ordinary common/uncommon/rare Ironclad, Silent, Defect, Watcher and colorless cards:
**76 common / 160 uncommon / 84 rare = 320 IDs**. Selection is equal over eligible
IDs **within the chosen rarity**, not equal per color. Basics, Status, Curse and
SPECIAL rarities cannot match; HEALING-tagged ordinary cards do match. Shard does
not widen merchant stock, transforms or the typed potion/combat generation below.

### 2.2 Rarity, offer history and acquisition

| Field | Normative source rule |
|---|---|
| Base offer count | 3, passed through owned relic count hooks in inventory order; Question Card adds 1, Busted Crown subtracts 2. These change offer count, not copies of the selected card. |
| Base thresholds | Ordinary/Event/Shop/Rest context: rare `R=3`, uncommon width `U=37`; elite: `R=10,U=40`; MonsterRoomBoss forces rare. Apply actual relic rarity hooks before classification. Other-character home relics/events are not granted by Note. |
| Live rarity offset | Starts `b=5`. Classify integer `n∈[0,99]` after adding `b`: rare below `R`, uncommon below `R+U`, common otherwise. Probability is the corresponding count of integers /100, clipped by that finite support. |
| Offset mutation | In ordinary `getRewardCards`, generated rare resets `b=5`; generated common changes it to `max(−40,b−1)`; uncommon leaves it. Once per generated slot **before** identity retries; not on taking the card. Boss generated slots also reset it. |
| Initial masses | Ordinary: rare/uncommon/common `0/35/65%`; elite `5/40/55%`. Ordinary at `b=−40`: `43/37/20%`. Successive slots use the changed offset, not one cached initial vector. |
| Duplicate handling | Reject an already offered `cardID`, resampling identity at the **same fixed rarity**. No extra rarity roll or offset mutation for that rejection. Distinctness is within one offer, not across separate offers or deck ownership. |
| Upgrade/preview | Act 1 automatic upgrade chance is 0. Actual relic preview hooks still run. Eggs upgrade their matching type only when upgradeable and not already upgraded; previews are not obtains. |
| Player decision | Take one card or skip; taking removes the reward row and schedules a FastCard obtain effect. Permanent insertion and obtain/master-deck hooks occur later, not at preview. Singing Bowl supplies an alternative +2 maxHP request, then removes its linked reward row and closes through its dedicated callback. |
| Prayer Wheel | A separate second ordinary card offer in ordinary nonelite/nonboss MonsterRoom only. It observes the first offer's history mutations; no duplicate exclusion spans both. Event combat is not an ordinary MonsterRoom. |

The separate dungeon **colorless reward** helper chooses rare on a strict
binary32 `.3` Boolean, otherwise uncommon, uses count hooks and fixed-rarity
duplicate rejection. Its rare slots reset shared `b`; uncommon slots do not
decrement it. The strict `.3` mass is `5033165/16777216`, not exactly 30%.
Neow-specific card helpers are not this helper or the combat-reward helper.

### 2.3 Reward amounts and generation boundary

These amounts are offered after the relevant surviving-combat completion path;
they are not an automatic grant on entering/fighting/dying in the room.

| Reward source | Actual A0 quantity / condition |
|---|---|
| Ordinary MonsterRoom gold | Integer 10–20, each local primitive integer mass 1/11; suppressed when monsters escaped. |
| MonsterRoomElite gold | Integer 25–35, each 1/11, on its actual elite-room path. |
| MonsterRoomBoss gold | `100 + integer(−5,5)`, hence 95–105, each 1/11; no A13 reduction at A0. Boss-room type, not a boss-like encounter name, owns this branch. |
| Ordinary gold reward aggregation | Added gold merges into the first GOLD row and recomputes its bonus from the aggregate. Golden Idol adds `G(F(base×F(.25)))` unless the actual room is TreasureRoom. This is not rounding each constituent separately. |
| Recovered stolen gold | Separate STOLEN_GOLD aggregation, using the theft flag; no Golden Idol bonus. The thief's actual remaining stolen amount supplies the quantity; an escaped thief is not an unconditional full refund. |
| Gold claim | Calls `gainGold(base+bonus)` once. Ectoplasm rejects the gain; otherwise positive gain mutates gold and invokes ordered `onGainGold`. Spending invokes `onSpendGold` in ShopRoom, then positive subtraction/clamp-to-zero and `onLoseGold`; offer construction invokes neither. |
| Elite relic | One relic, tier common/uncommon/rare 50/33/17 from integer 0–99. Black Star adds another noncampfire relic with a fresh tier roll. Emerald offer follows on a marked node when unowned. It is **not linked** to the relic. |
| Card reward | Standard reward-screen setup copies room rewards, then auto-adds an ordinary offer unless actual room is TreasureRoom, RestRoom, or has an event with `noCardsInRewards=true`. Boss is rare; event combat is ordinary EventRoom rarity. TreasureRoomBoss is **not** a TreasureRoom subclass and does not inherit that exclusion. No ordinary elite relic/key or Prayer Wheel is implied by event `eliteTrigger`. |
| Potion appearance | §5.2, after room drop rewards and before reward-screen auto card setup. Appearance, identity, claim and use are distinct operations. |
| Smoke/escape | The smoked reward opening skips setupItemReward and shows Proceed; the mugged opening does run setup. The room's rewardAllowed predicate still owns whether to open rewards. Do not award the ordinary victory bundle on an unresolved escape/death or infer it from `COMPLETE`; §9 retains terminal arbitration limits. |

Orrery in a shop creates four explicit card rewards plus the screen's auto-added
ordinary offer: five separately generated offers with shared changing offset.
Tiny House uses its actual Neow/boss-chest room context, **not** MonsterRoomBoss's
rare override: first one random permanent upgrade when available, then +5 maxHP
request, 50 gold and one flat-ID potion, then opening the ordinary card-reward screen.
Cauldron supplies five flat-ID potion reward rows.
Cauldron's discarded auto card offer, and Calling Bell's cleared auto rewards,
can already have mutated rarity history. Deleting an offer does not rewind it.
Dream Catcher instead invokes the ordinary card helper directly after the Rest
heal/onRest hooks, using actual RestRoom rarity/count/history rules, then returns
through its campfire continuation. It is not the screen setup's auto-card branch
or a special fixed-rare reward; Shard can affect its ordinary reward selector.

## 3. Merchant records

### 3.1 Initial stock and conditional proposals

| Stock | Exact production rule |
|---|---|
| Colored cards | Two Attacks, two Skills, one Power from current-character typed permanent pools (§2.1), not Shard and not the combat HEALING filter. Each proposal uses the actual ShopRoom rarity rule with live `b`, but does **not** run reward-helper offset mutation. |
| Second Attack / Skill | Reject the first same-type ID, then reroll **both rarity and identity**. If an eligible identity has original proposal mass `p(i)`, its accepted mass is `p(i)/(1−p(first))`; rarity marginals therefore change. Do not retain the rejected proposal's rarity as in rewards. |
| Power type miss | Common has no eligible Power: common proposals fall through to the uncommon typed pool. Thus uncommon total mass includes common+uncommon rarity mass; rare uses six eligible rare Powers. This is not a fresh Power-specific rarity lottery. |
| Colorless | One uncommon from 20, one rare from 15, equal identity mass within that rarity. Bandage Up is eligible here. No colored-card ownership exclusion. |
| Relics | Two slots with tier 48% common /34% uncommon /18% rare; third initially SHOP. End-selector pool consumption (§4), not fresh uniform-ID draws. |
| Potions | Three unrestricted rarity-first proposals (§5.1), repeats allowed; neither full belt nor Sozu removes their stock offers. |
| Removal | One purchase per shop, independent of stock/Courier. Uses actual nonbottled purgeables: excludes Necronomicurse, Curse of the Bell and Ascender's Bane, plus all bottle flags. |

### 3.2 Initial prices, sale and discounts

| Price stage | Exact arithmetic / order |
|---|---|
| Colored card base | Common 50 / uncommon 75 / rare 150. Price `T(F(base × L(F(.9),F(1.1),k)))`. |
| Colorless base | Same rarity base, then another binary32 multiplication by `F(1.2)`, **before** truncating to int. |
| Sale | Uniform one of the five colored slots, then integer price division by 2. No colorless sale. This happens before discounts. |
| Relic base | Common 150 / uncommon 250 / rare 300 / SHOP 150. Round `G(F(base × L(F(.95),F(1.05),k)))`. No eligible relic overrides the base price member. |
| Potion base | Common 50 / uncommon 75 / rare 100, same `.95–1.05` finite jitter and `G` rounding as relics. |
| A0 modifiers | No A16 markup. Existing Courier applies `.8`, then Membership Card `.5`; each step independently applies `G(F(current_integer_price × factor))` to every current item. Never combine these into one final rounding. |
| Removal base | Run-wide `purgeCost=75` initially. Every purchased removal increases that base by 25, even with Mask. Entering shop initially sets actual price to base, then runs the discount rules below. |

**Removal's nonidealized precedence is required.** `applyDiscount` recomputes
actual removal price from the **base** for its current multiplier, rather than
discounting the previous actual price. With both Courier and Membership on initial
shop entry, Membership therefore leaves `G(F(base×.5))`, not combined `.4`.
At base 75 this is **38**, not 30. Smiling Mask overrides to **50**, regardless
of either discount. After a paid purge, the explicit next-price branch instead
uses `G(F(F(new_base×.8)×.5))` when both are owned; one relic uses its one factor.
The incremented base 100 then yields 40 with both. This computed next price does
not grant another removal in the same shop. Buying Membership invokes its actual
discount pass immediately, including that recomputation for removal.

### 3.3 Purchase and Courier replacements

| Operation | Actual order / replacement |
|---|---|
| Card purchase | Requires sufficient gold; schedules FastCard obtain, spends the displayed price, then replaces/removes stock. Durable card hooks are not preview hooks or already committed by scheduling. |
| Colored Courier replacement | Same purchased type, ordinary rarity proposal with no offset update; reject colorless, but **no** rejection of purchased ID or other remaining same-type stock. Preview hooks run. |
| Colorless replacement | Rare on strict binary32 `.3`, otherwise uncommon. Unlike initial stock, the former rare/uncommon slot is not fixed to its old rarity. |
| Replacement card price | Binary32 base×`.9–1.1` jitter, colorless×1.2 if applicable, Courier×.8, Membership×.5, then **one final truncation**. No new half-price sale. Example raw integer base-price input 151 gives 61 with sequential initial discount rounding, versus 60 with replacement-style multiply/truncate. |
| Relic purchase | Spends gold, installs via instantObtain and runs `onEquip`, re-previews shop cards, then Courier replacement. Nested relic reward/selection must retain parent shop ownership; purchase is not a universal return-to-map. |
| Relic replacement | New ordinary merchant tier 48/34/18 even in the former SHOP-only slot. Repeatedly excludes Old Coin, Smiling Mask, Maw Bank and Courier; these also have ShopRoom spawn exclusions. They are not globally illegal Act 1 relics. |
| Potion purchase | Sozu rejects first. Otherwise sufficient gold and **successful** first-empty-slot obtain are required before spending/restocking. Full belt leaves gold and stock untouched. |
| Relic/potion replacement price | Fresh base, `.95–1.05` jitter with `G`, then separate Courier and Membership `G` passes. Not the card replacement's final-truncation expression. |
| Paid removal | Opening selection does not spend. Selected callback spends actual price, increments base, removes the selected object with card-removal/master-deck callbacks, and disables removal. Cancellation is not a paid purge. Parasite removal's HP consequences are not erased by labelling this a shop service. |

## 4. Relic history and chests

### 4.1 Relic selection is history-bearing

Initialization creates shared+current-Ironclad unlocked tier lists, shuffles each,
and removes designated starting/owned entries. Offered entries are consumed even
when skipped. Subsequent selection is not catalog sampling with replacement.

| Requested selector/state | Actual consumption / continuation |
|---|---|
| Normal, nonempty C/U/R/SHOP | Front of the requested tier. |
| End, nonempty C/U/R/SHOP | Back of the requested tier. |
| Either, nonempty BOSS | **Front**, not end/back for the end helper. |
| Rejected `canSpawn` candidate | Candidate remains consumed. Normal rejection dispatches to **end at the originally requested tier**; end rejection recurses through end at that requested tier. |
| Empty common/uncommon/SHOP | Promote respectively to uncommon/rare/uncommon through **normal/front** helper, including when reached from end. Do not retain back direction across depletion promotion. |
| Empty rare / boss | Circlet / Red Circlet fallback respectively. Circlet stacking has its actual instant-obtain counter path, not an arbitrary native maximum. |
| Screenless source | Consumes/rejects Bottled Flame, Bottled Lightning, Bottled Tornado and Whetstone; **War Paint is not in this exclusion**. |
| Noncampfire source | Consumes/rejects Peace Pipe, Shovel and Girya. |

Spawn eligibility still reads live source conditions: Black Blood needs Burning
Blood; Neow swap removes the starter before drawing. Ectoplasm is legal in Act 1.
Bottles require eligible actual permanent cards (nonbasic Attack/Skill, or Power).
For this profile the relevant additional spawn conditions are finite:

| Relic(s) | Source spawn condition in the standard non-Endless run |
|---|---|
| Courier, Maw Bank, Old Coin, Smiling Mask | Not ShopRoom, and floor≤48. |
| Girya, Peace Pipe, Shovel | Floor<48 and fewer than two owned among those three. |
| Black Blood / Ectoplasm | Own Burning Blood / act≤1, respectively. |
| Bottled Flame / Lightning / Tornado | A nonbasic permanent Attack / nonbasic permanent Skill / permanent Power exists, respectively. |
| Ancient Tea Set, Ceramic Fish, Darkstone Periapt, Dream Catcher, Frozen Egg, Molten Egg, Toxic Egg, Juzu Bracelet, Meal Ticket, Meat on the Bone, Omamori, Potion Belt, Prayer Wheel, Question Card, Regal Pillow, Singing Bowl | Floor≤48. |
| Matryoshka, Wing Boots / Tiny Chest / Preserved Insect | Floor≤40 / floor≤35 / floor≤52, respectively. |

Other-character starter replacements still cannot enter Ironclad's registered
pools merely because their own spawn predicates could pass. This is not a claim
that every relic constructor/spawn predicate alone proves its full primary effect.
The exact measure on initial seeded permutations and their history-conditioned
remaining orders remains separate (§9); “shuffled” does not prove uniformity.

### 4.2 Shovel Dig: separate relic-reward source

At a RestRoom campfire, choosing Dig schedules `CampfireDigEffect`. When its dig
branch runs, it clears that room's reward list and appends **one relic offer**.
It draws a tier using the ordinary dungeon relic-tier helper (Act 1: common /
uncommon / rare 50/33/17 from an inclusive integer 0–99), then uses the ordinary
**normal/front** relic selector. The same consumed-candidate, rejection and
depletion-promotion rules in §4.1 apply; this is not a fresh catalog draw. The
effect marks the local room phase COMPLETE and opens the reward screen. A relic
row is offered, not installed: claim, skip and durable continuation are separate
transitions, with full terminal arbitration still open in §9.

Dig is not chest opening: it has no chest-size/shared gold-tier roll, gold, key,
chest-opening hooks (`onChestOpen`/`onChestOpenAfter`), or chest sapphire linkage.
It does not use the merchant end selector or Black Star's noncampfire selector.
The reward screen's ordinary card setup excludes RestRoom, so this route adds no
automatic card reward. Do not infer that the local COMPLETE phase settles reward
claiming or terminal composition.

### 4.3 Ordinary chest joint outcomes

Chest size uses an integer 0–99: small 50%, medium 33%, large 17%. A second integer
0–99 is **shared** between that chest's relic tier and gold-presence test:

| Size / nominal gold | Shared-roll intervals and joint outcome |
|---|---|
| Small / 25 | 0–49: common+gold (50%); 50–74: common/no gold (25%); 75–99: uncommon/no gold (25%). |
| Medium / 50 | 0–34: common+gold (35%); 35–84: uncommon/no gold (50%); 85–99: rare/no gold (15%). |
| Large / 75 | 0–49: uncommon+gold (50%); 50–74: uncommon/no gold (25%); 75–99: rare/no gold (25%). |

For nominal gold `N`, compute **separate endpoints** `a=F(N×F(.9))` and
`b=F(N×F(1.1))`, then amount `J(L(a,b,k))`. This is **not**
`J(F(N×L(F(.9),F(1.1),k)))`. At `N=25,k=3355440`, the actual amount is 24;
the factored expression gives 23. First indices reaching 24 differ by two grid
points, a probability/CDF difference `2/2^24`, not merely seed alignment.

Opening invokes ordered `onChestOpen(false)` hooks first, then chest gold/curse
and base relic, attaches sapphire to that **base** relic row when unowned, then
ordered `onChestOpenAfter(false)`, then opens rewards. Matryoshka adds a separate
common/uncommon relic on strict `.75` (exact 3/4), consuming one of its two charges.
Cursed Key schedules a random curse on ordinary opening; it is not a paid obtain
or a forced sapphire. Hungry Face removes the first relic and its adjacent linked
row, consuming its counter; extra relic and base-relic linkage must remain distinct.
The random-curse helper's support is Clumsy, Decay, Doubt, Injury, Normality, Pain,
Parasite, Regret, Shame and Writhe, equal local ID mass 1/10. Ascender's Bane,
Necronomicurse, Curse of the Bell and Pride are excluded. Scheduling that obtain
still preserves actual curse-prevention/obtain callbacks; it is not automatic deck
insertion at chest preview.

Claiming the linked base relic suppresses sapphire and vice versa. Claiming sapphire
only schedules the key effect; its durable flag changes in `ObtainKeyEffect.update`,
not at offer creation. Emerald has no such relic exclusion. Skipping either key or
the whole chest is legal. No automatic successful Act 1 handoff follows from offers.

**Boss chest:** constructs three sequential boss-tier offers, consuming pool entries;
choose one or skip. It calls opening hooks with `true`, explicitly omitting
Matryoshka, and opens the boss-relic screen. There is no ordinary chest gold/size
roll, no sapphire attachment, and no ordinary after-open pass. Cursed Key's boss
guard creates no curse here. Relic installation, nested children, skipped rewards
and resolved Act2Ready are separate from constructing these three offers.

## 5. Potion factory, acquisition and appearance

### 5.1 Actual catalog and proposal laws

The current Ironclad factory list is **33**, partitioned **17 common /9 uncommon /
7 rare**, all recorded in §6. It is **not 53**. The all-character factory list is
42 = 30 shared + 12 character-specific. Ironclad adds Blood Potion, Elixir and
Heart of Iron to the 30 shared IDs. The following nine are absent from its actual
list, not just omitted from an informal roster:

| Exclusive character | Excluded factory IDs |
|---|---|
| Silent | `Poison Potion`; `CunningPotion`; `GhostInAJar` |
| Defect | `FocusPotion`; `PotionOfCapacity`; `EssenceOfDarkness` |
| Watcher | `BottledMiracle`; `StancePotion`; `Ambrosia` |

`initializePotions` passes **player.chosenClass**; the source card's printed color,
Shard, Note storage and deck composition never substitute for that authority.
Ironclad playing legally obtained Silent Alchemize (`Venomology`) therefore
generates from the **Ironclad** potion pool, not Silent exclusives.

| Source | Exact local identity law |
|---|---|
| Combat drop, merchant, Courier potion replacement | Roll rarity once: integers 0–64 common, 65–89 uncommon, 90–99 rare. Uniform proposals from the 33-ID list are rejected until chosen rarity matches. Thus per-ID masses are `65/(100×17)`, `25/(100×9)`, `10/(100×7)`. Repeats allowed; no depletion/potion-appearance offset mutation here. |
| Flat factory helper | Equal 1/33 per ID, ignoring rarity; used by Lab, Woman in Blue, Neow potion rewards and Cauldron. Tiny House uses this identity law with its supplied RNG. These are not the rarity-first helper. Event quantities are specified by their own records. |
| Alchemize / combat Entropic Brew | Limited rarity-first helper. Rarity remains fixed. First flat proposal is unconditionally discarded; later proposals reject Fruit Juice and wrong rarity. Fruit Juice is the only additional ID exclusion on this path. Effective support 17/9/**6**; rare per-ID mass `10/(100×6)`, common/uncommon unchanged. Brew and all eligible healing potions other than Fruit Juice remain possible. |
| Outside-combat Entropic Brew | Unrestricted rarity-first helper, **not flat33**. Fruit Juice is eligible; same 17/9/7 masses as unrestricted above. Phase is resolved by Brew's actual use branch. |

The discarded first limited proposal is a source fact, not authorization to add
a gameplay retry cap or to adopt original incidental cursor parity. Conditional
laws across externally observed histories still require §9's declared input measure.

### 5.2 Combat potion appearance counter

The appearance offset starts **0**. Actual elite/EventRoom combat uses chance
`40+offset`; ordinary MonsterRoom (including boss inheritance) uses it only when
no monster escaped, otherwise starts at 0. White Beast Statue then overrides to
100. **Four or more existing room reward rows then override to 0**, even with
White Beast. This check precedes auto-added card rewards, not after an assumed
final bundle size. Roll integer 0–99 and succeed strictly below chance. Success
adds one unrestricted rarity-first potion offer and subtracts 10 from offset;
failure adds 10. Out-of-range chance is interpreted by the finite integer test,
not clamped state or a resetting pity cap. Offers can be generated with a full belt.
Claiming, skipping, using and selecting potion identity do not change this offset.

## 6. All 33 eligible potion records

Amounts are **local requests**, not guaranteed net damage, healing or power gains.
`base → Bark` gives the actual payload with Sacred Bark. Unless overridden in §7,
manual use requires live combat, monsters not basically dead, turn not ended,
and no We Meet Again event. “Self” means no monster selection; only Fire/Fear/Weak
require one monster. Combat powers do not become persistent run stats.

| Display name / exact factory ID | Tier | Base → Sacred Bark payload; target and lifetime |
|---|---|---|
| Block Potion / `Block Potion` | C | Self Block request **12→24**, ordinary Block lifetime; GainBlockAction, not a card with Dexterity-scaled printed block. |
| Dexterity Potion / `Dexterity Potion` | C | Self **2→4 Dexterity**, combat power. |
| Energy Potion / `Energy Potion` | C | Self **2→4 energy** via GainEnergyAction; not a permanent energy-master increase. |
| Explosive Potion / `Explosive Potion` | C | **10→20** all-enemy damage matrix with null action source, NORMAL type and the matrix's enemy-only flag; no selected monster. Do not relabel as HP_LOSS or Fire Potion's THORNS route. |
| Fire Potion / `Fire Potion` | C | Selected enemy **20→40**, player-owned THORNS DamageInfo with enemy-powers-only adjustment, then DamageAction. Not a normal Attack card/Strength calculation. |
| Strength Potion / `Strength Potion` | C | Self **2→4 Strength**, combat power. |
| Swift Potion / `Swift Potion` | C | Draw **3→6** cards, ordinary draw hooks/hand capacity apply; no cost randomization. |
| Weak Potion / `Weak Potion` | C | Selected enemy **3→6 Weak**, `isSourceMonster=false`; debuff duration follows ordinary owner turn hooks, not “three player turns” by assumption. |
| Fear Potion / `FearPotion` | C | Selected enemy **3→6 Vulnerable**, `isSourceMonster=false`; ordinary debuff application/duration. |
| Attack Potion / `AttackPotion` | C | Three distinct Attack offers; choose one identity, **1→2 stat-equivalent copies** with a guarded zero-cost request. See §7.1 for exact support, optionality and overflow. |
| Skill Potion / `SkillPotion` | C | Three distinct Skill offers; choose one identity, **1→2 copies**, same guarded zero-cost request and local creation rules. |
| Power Potion / `PowerPotion` | C | Three distinct Power offers; choose one identity, **1→2 copies**, same guarded zero-cost request and local creation rules. |
| Colorless Potion / `ColorlessPotion` | C | Three distinct ordinary nonhealing colorless offers; **1→2 copies** of chosen identity, same guarded zero-cost request and overflow route; colorless is an explicit source policy. |
| Flex Potion / `SteroidPotion` | C | Self **5→10 Strength**, then separate **5→10 Lose Strength** power request. End-of-turn loss applies negative Strength and schedules removal of the loss power. Artifact/prevention can distinguish the two applications. |
| Speed Potion / `SpeedPotion` | C | Self **5→10 Dexterity**, then separate **5→10 Lose Dexterity** request; analogous end-of-turn negative Dexterity/removal, not a timeless stat. |
| Blessing of the Forge / `BlessingOfTheForge` | C | Upgrade every currently upgradeable hand instance once on action resolution, then apply powers. **Unchanged by Bark** (potency 0); not permanent deck upgrades or two upgrades. |
| Blood Potion / `BloodPotion` | C | Self heal `T(F(maxHP×F(p/100)))`, **p=20→40**; queued HealAction in combat, direct player.heal outside. No minimum-one clamp in this potion. Special use gate §7. |
| Regen Potion / `Regen Potion` | U | Self **5→10 Regeneration**. End-turn RegenAction requests current amount healing, then decrements player Regeneration by 1/removes at zero. Uninterrupted isolated totals **15→55**, not 15→30. Action requires COMBAT; heal requires currentHP>0. |
| Ancient Potion / `Ancient Potion` | U | Self **1→2 Artifact** charges, consumed by ordinary debuff prevention rather than timed expiration. |
| Liquid Bronze / `LiquidBronze` | U | Self **3→6 Thorns**, combat power; on qualifying normal damage owned by another attacker, retaliation uses THORNS damage. Not all HP loss or self-damage. |
| Gambler's Brew / `GamblersBrew` | U | Choose any number (including zero) of hand cards to discard, then draw that count. **Unchanged by Bark** (potency 0). Manual-discard notifications occur per selected instance; empty hand schedules no action. |
| Essence of Steel / `EssenceOfSteel` | U | Self **4→8 Plated Armor**. Pre-end-turn-card hook requests that much Block. Qualifying positive HP damage with other nonnull owner and neither THORNS nor HP_LOSS queues a reduction by 1. |
| Duplication Potion / `DuplicationPotion` | U | Self **1→2 Duplication** charges. A non-purgeOnUse card with positive charges creates one same-instance repeat and consumes one charge. End-of-round removes if already zero, otherwise reduces by 1; do not replace this with unconditional end-turn removal. Repeat copies captured energy and sets autoplay/ignore-total true. |
| Distilled Chaos / `DistilledChaos` | U | Queue **3→6** PlayTopCardAction instances. Each target proposal is captured during potion use, while constructing the queued action list and **before any queued card resolves**; these are not fresh independent live-target draws after prior cards resolve. Retain each action's downstream queue/hand/death behavior. No payment of an ordinary selected card cost. This is not a card-offer lottery. |
| Liquid Memories / `LiquidMemories` | U | Retrieve **1→2** selected discard instances to hand, requesting turn cost 0 through the guarded setter. Mandatory count where available; auto-all when discard count≤requested. Each move requires hand size<10; otherwise that instance remains discarded. Not “overflow to discard” construction. |
| Elixir / `ElixirPotion` | U | Choose any number including zero of hand instances to Exhaust; **unchanged by Bark** (potency 0). Uses actual Exhaust movement/hooks, not manual discard or deletion. |
| Cultist Potion / `CultistPotion` | R | Self **1→2 Ritual**. Each player end-turn hook requests that much Strength; persists as a combat power, not an immediate-only Strength grant. |
| Fruit Juice / `Fruit Juice` | R | Immediate **+5→+10 maxHP** request, then ordinary heal request of the added amount through increaseMaxHp. Special use gate §7; not eligible in limited generation. |
| Snecko Oil / `SneckoOil` | R | Draw **5→10**, then randomize costs of the **whole resulting hand**. Each nonnegative base-cost instance proposes integer 0–3; if proposed value differs, update base and turn cost and mark modified. Negative/X/unplayable base costs are skipped; Bark does not change 0–3. |
| Fairy in a Bottle / `FairyPotion` | R | Never manually usable. Damage-path rescue at HP<1 requests heal `max(1,T(F(maxHP×F(p/100))))`, **p=30→60**, then destroys its slot. Exact rescue ordering and no-use-hook distinction §8. |
| Smoke Bomb / `SmokeBomb` | R | Escape request, **unchanged by Bark** (potency 0). Requires ordinary use gate and no BOSS-type monster or BackAttack power anywhere in room list. Sets smoked/isEscaping and disables end-turn; not automatic victory, death or reward claim. |
| Entropic Brew / `EntropicBrew` | R | One proposal per **total live potionSlots**, combat limited versus outside unrestricted (§5). Bark doubles stored potency but **use loops over live slot count, not potency**: no doubled proposal count. Its vacated own slot can be filled. Special gate and Sozu/order §7.2. |
| Heart of Iron / `HeartOfIron` | R | Self **6→12 Metallicize**, requesting that much Block at pre-end-turn-card hook each turn, combat duration. |

Sacred Bark doubles the integer returned by the ordinary potency accessor; it
does not automatically duplicate use callbacks or every effect. Its onEquip
re-runs initializeData on potions already in the belt, so stored payloads refresh.
The zero-potency effects and Brew's slot-count loop explain their unchanged effects.

## 7. Potion choices, phase and belt semantics

### 7.1 Card-producing potions

Attack support is the 30 Attacks in §2.1 **minus Feed and Reaper = 28**.
Skill support is all 28 Skills in §2.1. Power support is all 14 Powers there.
Colorless support is the 35 listed colorless IDs **minus Bandage Up = 34**.
These are equal-ID combat selectors after filtering rarity/type/HEALING; not
rarity-first, not Shard, not another character because the potion/card is off-color.
Basics, curses, Status and all SPECIAL IDs including Expunger are absent by the
pool chain. Hand of Greed is eligible; durable resource output is not a universal ban.

The displayed offer is ordered uniform sampling without replacement over its
support under the local uniform proposal law; each next eligible ID has mass
`1/(N−accepted_count)`. Repeated IDs retry identity, not a rarity lottery. No card
reward offset changes. Typed Attack/Skill/Power custom offers pass their source's
optional-choice flag; Colorless passes the other flag. The inspected custom combat
screen exposes skip for the typed path, not a universal potion/Discovery skip rule.
Selecting does not reroll the displayed candidates. Bark copies one selected
identity twice; it does not generate six offers or two independent selections.

Construction makes stat-equivalent copies, explicitly upgrades each for Master
Reality when present, then calls `setCostForTurn(0)`. That setter changes an
**existing nonnegative** turn cost to 0; an existing negative turn cost (including
X-cost −1) is left unchanged. The call does not set a generic free-play flag.
Thus the local request is not an unconditional zero-cost result. One child goes
to hand if size<10,
otherwise discard; two at size≤8 both to hand, at size9 one hand/one discard,
at size10 both discard. These destinations are chosen before deferred add effects;
route-specific downstream creation upgrades/Searing Blow and interruption remain §9.
The action computes candidate lists on update calls beyond its initial presentation;
do not infer original cursor parity or certify all scheduler-conditioned random
consumption from the three visible choices alone.

### 7.2 Use, obtain, discard and Sozu

| Boundary | Exact source behavior |
|---|---|
| Starting belt | A0 has 3 slots. Potion Belt adds 2 live slots on equip, yielding 5 with it. Slot occupancy is run-owned; no potion list is depleted by obtaining one. |
| Ordinary obtain | Scans in belt order to the first PotionSlot, replaces it, sets slot/obtained state and flashes. Full belt returns false. This helper has **no Sozu test** and no generic onObtainPotion relic callback. The indexed overload directly replaces its supplied slot and is not the scan helper. |
| Reward claim | With Sozu, flashes and returns success, so the reward is removed without obtain. Without Sozu, full belt returns false and keeps the offer; success obtains. This differs from shop rejection. |
| Combat ObtainPotionAction | Its first update checks Sozu; otherwise attempts ordinary obtain. An already generated proposal can fail due to current capacity; it is not resampled until it fits. |
| Outside ObtainPotionEffect | On update, checks Sozu, optionally attempts ordinary obtain, then marks done. No full-belt choice or inferred replacement prompt. |
| Manual nontargeted use | Valid use calls potion.use first, then each ordered relic.onUsePotion, then destroys the consumed slot. Pending actions can resolve after that slot is vacated. |
| Manual targeted use | Selecting a valid non-dying hovered monster calls use(target), queues HandCheckAction in combat, calls ordered onUsePotion, destroys slot. Canceling targeting does not consume it or invoke use hooks. |
| Discard | Replaces slot with PotionSlot, without use/payload/onUsePotion. canDiscard rejects We Meet Again. Sozu does not forbid discarding or using already held potions. |
| Phase exceptions | Blood Potion, Fruit Juice and Brew reject when turnHasEnded **and** room phase COMBAT; otherwise reject We Meet Again but do not require the ordinary live-monster/combat predicate. Hence they can be usable outside combat and are not restricted by the base potion gate. Fairy is never manually usable; Smoke adds its own restrictions. |
| Brew in combat | During use, generates exactly live slot-count limited proposals and queues that many obtains, even with Sozu or initially full belt. The normal manual-use path then destroys Brew's slot before queued obtains. Sozu/capacity are checked later per obtain. |
| Brew outside combat | Sozu branches before generating proposals. Otherwise generates live slot-count unrestricted proposals into obtain effects, then normal use hooks/slot destruction occur. Full/unused proposals are not evidence of fewer source draws. |

No callback in this table approves a universal FIFO/drain/return-to-map policy.
Alchemize's ordinary action creates its potion proposal before later obtain checks;
repeats resolve as separate actions with live belt/Sozu state, not a cached Silent
pool or a victory reward. Merely acquiring a potion does not invoke onUsePotion.

## 8. Healing, maxHP, death and consumer distinctions

| Consumer | Precise local distinction |
|---|---|
| Ordinary healing | AbstractCreature.heal returns if isDying; ordered player relic onPlayerHeal callbacks precede target power onHeal callbacks, then HP addition capped at maxHP. Bloodied-to-not-bloodied callbacks follow. A listed heal is not unconditional net HP. |
| Magic Flower | In actual COMBAT with a current map node, transforms heal amount to `G(F(amount×F(1.5)))`; otherwise unchanged. Blood's own percentage rounding happens **before** this callback. This applies to actual heals, not multiplying maxHP gain itself. |
| Mark of the Bloom | Its heal callback returns 0; the player damage rescue branch also explicitly skips Fairy/Lizard Tail when owned. Do not model the rescue exclusion solely as a rounded zero heal. |
| Fruit Juice | increaseMaxHp first adds 5/10 maxHP, then heals by that requested amount through normal callbacks. A blocked heal does not undo the maxHP increase. CurrentHP can gain a different amount from maxHP because of healing hooks/cap. |
| Toy Ornithopter | Manual potion-use hook requests heal 5: queued in COMBAT, direct outside. Bark does **not** double this relic's 5. Discard/failed obtain/offer generation do not invoke it. Its queued heal is not proof of survival after a lethal payload. |
| Fairy rescue | In AbstractPlayer.damage after actual damage/HP-loss mutation and card notifications, HP<1 checks Mark of Bloom, then first belt Fairy, then unused Lizard Tail only when no Fairy. Fairy uses its heal/clamp and destroys slot; caller also destroys that slot and returns from damage. This path does **not** run PotionPopUp's onUsePotion callbacks, so no automatic Ornithopter heal. It is not a universal resurrection hook for every possible maxHP/removal/death path. |
| Plated Armor | A queued one-point reduction requires positive qualifying HP damage, other nonnull owner, and neither HP_LOSS nor THORNS. Blocked hits/self-owned HP loss do not satisfy that local hook. This is separate from amount of the potion application. |
| Thorns | Requires nonnull other attacker and neither THORNS nor HP_LOSS type. **No positive-damage test occurs in this hook**, so it must not be replaced by Plated Armor's positive-HP-loss predicate. Retaliation is queued at top, not an immediate simultaneous “net damage” adjustment. |
| Regeneration | End-turn power queues its RegenAction at top; action exits outside COMBAT, heals only a positive-HP target, then decrements/removes the player power. The isolated 15/55 totals assume enough qualifying turns, missing HP and no modifiers/interruption. |
| Temporary stat loss | Flex/Speed first apply the positive stat, then a separate debuff power. Its end-turn hook applies the negative stat and schedules loss-power removal. Prevention/Artifact and intervening stat changes can distinguish this from restoring a saved pre-potion stat. |
| Distilled Chaos / Duplication | Inner card effects retain their actual current-character source policies and captured energy/history/targets. Distilled Chaos captures target proposals during potion.use before queued cards resolve; downstream retargeting, death and cancellation remain action/scheduler-specific. A potion does not erase autoplay versus normal play, origin, payment or nested continuation identity. Multi-card death/cancellation is not settled merely by counting queued actions. |

Mark of the Bloom is stated as an explicit source rescue/heal guard, **not** a
newly eligible Act 1 acquisition; its later-act home event is not imported here.

## 9. Exact remaining proof obligations; no silent approximation

This document supplies missing **public local numeric records**, not every transitive
composition, the whole Act 1 specification, or any human implementation gate.
Facts above are not a proposal to change the source rules. Native state/continuation
representation must still be reviewed under the existing
[execution boundary](2026-10-18-act1-execution-closure.md) and
[RNG/map boundary](2026-10-18-act1-rng-map-closure.md).

| Residual | Exact unsupported field / smallest justified position |
|---|---|
| ECON-JOINT | Full observable joint/conditional masses after arbitrary history require the declared root-seed/input measure and exact seeded-shuffle pushforward/posterior for relic pools (and any mechanically relevant shared random dependencies). The finite float/integer laws above do not certify uniform remaining permutations or independently reset streams. Preserve conditional source transformations; do not substitute a universal random pool. |
| ECON-TERMINAL | Whether each pending reward, maxHP/heal, repeat, obtain/key effect or nested shop child survives/cancels when death, escape, battle end and parent-screen transitions compete. Local call order is evidenced; complete transitive scheduling and durable Act2Ready are not inferred. No rollback/fake defeat/retry on administrative failure. |
| POT-COPY | Complete route composition of Discovery stat-equivalent copies, Master Reality, Searing Blow repeat upgrades, downstream add-effect upgrades, later X/turn-cost precedence, prior per-instance history and full-hand deferred placement under other simultaneous effects. The local guarded `setCostForTurn(0)` result and use-time Distilled Chaos target proposals are specified; neither is a universal final-upgrade/cost or downstream target/scheduler theorem. |
| ECON-EXCEPTIONAL | Reachability and exact handling of exhausted/empty exceptional candidate lists, zero accepted proposal mass, unusual purge selection/death and numeric exhaustion beyond the supported local records. Do not invent a rarity retry, fallback relic/card, dropped child or gameplay cap from a helper/tool limit. |
| POT-RESCUE | Universal death prevention across non-damage maxHP/removal paths and lethal multi-effect interactions remains unproved here. The precise Fairy damage callback is not a blanket rescue contract. |

Future discriminating acceptance cases (**not executed gameplay**):

1. Initial versus second merchant Attack with one high-mass rejected identity;
   contrast whole-proposal retry with fixed-rarity reward retry and offset mutation.
2. Both discount relics at purge base75: initial38, after paid increment base100
   computed40; Mask50 overrides both. Contrast initial-item rounding and restock
   truncation, and verify a purchase does not grant another purge.
3. Chest `N=25,k=3355440`: actual24 versus factored23; preserve the size-specific
   shared gold/tier integer and base-relic sapphire linkage with Matryoshka/Hungry Face.
4. End selector depleted common plus two different uncommon entries: promotion takes
   normal/front. Rejected normal rare goes end at requested rare; boss stays front.
5. Ironclad off-color Alchemize versus Brew inside/outside combat: limited rare6
   excludes Fruit Juice; outside rare7 includes it; no Silent exclusives. Brew with
   five live slots and four occupied generates five proposals, not one or ten.
6. Four existing reward rows plus White Beast: potion appearance fails and offset
   increases10; full belt alone still permits offers. Sozu reward claim removes its
   row, shop purchase keeps stock/gold, combat Brew still generates proposals.
7. Blood versus Fairy minimum healing at small maxHP, Magic Flower combat/noncombat,
   Fruit maxHP-before-heal, Mark of Bloom rescue exclusion, and Fairy without manual
   use/Ornithopter callback. Do not assert unproved post-lethal continuation results.
8. Bark typed-potion same chosen identity twice at hand sizes8/9/10, versus Memories
   retaining an unmoved discard instance at capacity. Discovery copies are not new offers.
9. Bark Regen isolated55 rather than30; Duplication charge consumption and its actual
   end-round decrement; Snecko whole-hand nonnegative base-cost0–3 versus negative/X.
10. Attack Potion chooses Whirlwind and creates one/two copies with Bark: the
    negative X turn cost remains negative; Liquid Memories retrieves an X-cost
    discard instance with and without Bark and leaves that cost negative; include
    an ordinary positive-cost retrieved/copied card control. These are unexecuted.
11. Shovel Dig: one ordinary tier proposal and normal-selector relic offer, including
    consumed-candidate/depletion behavior; contrast no chest hooks/gold/keys and no
    automatic RestRoom card reward. Claim/skip/terminal composition remains separate.

## 10. Verification performed and handoff

Private standard-library tests were written failure-first: three extraction/environment
tests, then six arithmetic/catalog tests. The combined **9 tests pass**, including
duplicate/missing/nonclass/traversal ZIP negative controls, JVM option stripping,
the chest endpoint discriminator, discount/purge distinctions, finite rarity counts,
whole-proposal rejection, actual factory33/tier counts and strict float-grid mass.
These validate helpers and stated arithmetic consequences, not original gameplay.

All newly invoked Java tools were pinned installed javap/CFR only, with the three
JVM option environment variables removed. Exact unique class ZIP entries, class
hashes, commands/statuses, exclusive output directories and four input/tool pins
before/after were retained. One wrong namespace lookup failed closed before a JVM
invocation; its script/log/output directory remain preserved, followed by a distinct
corrected inspection. No archive-wide decompile, assets, source/helper publication,
dependency installation, gameplay tests, GPU work or product changes occurred.

Only this public document is authored in the assigned public worktree. Claim ledger,
source anchors, test logs, failed attempt and manifests stay private. The issue remains
**in progress for independent exact-head review and main readback**; no parent/gate
closure, main mutation, PR readiness, merge or cleanup is performed by the author.
