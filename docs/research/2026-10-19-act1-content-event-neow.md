# Act 1 normative records: Neow, question rooms and all 25 events

Date: 2026-10-19. Author task: `spire-gli.2.3.23` (B08).
**Draft for independent review; specification only, not implementation approval.**

## 1. Scope, notation and evidence limits

These are numeric and conditional records for standard, unmodded, fully unlocked,
Act-4-enabled **Ironclad, A0, Act 1**. Fully unlocked does not imply a previous
boss encounter. Other characters' cards can legally arrive through Note and
source-specific off-color mechanisms; that does not import their starts, potion
pools, exclusive relics or home events.

The authority is the owned desktop archive SHA256
`cfad868ac8d65a88e71a0bf096fb09f78811e553effe0787c5309a655e081673`.
**S** below means retained static member/caller inspection; **BC** means specifically
checked bytecode; **D** means arithmetic consequence; **U** marks an actual remaining
field. None means original gameplay execution. The public rules below are usable
without importing private tables; private locators establish provenance, not hidden
normative requirements. No reconstructed source, executable port or assets are included.

`H` is current HP, `M` maximum HP, `G` gold. A value described as captured is fixed
at event/offer construction, not silently recomputed on confirmation. `f(x)` means
IEEE binary32 rounding; decimal source constants are first rounded to binary32.
`Tq(M) = trunc(f(f(M) × f(q)))`. Java integer division truncates toward zero;
for nonnegative HP this is floor. Damage/healing/gold amounts in the records are
**requests**, not promises of their nominal net change after relics, caps, rescue,
death or deferred obtains. Section 6 states the downstream boundary.

Amounts, counters, list sizes and source upgrade counts use signed Java 32-bit
fields; floating intermediates are binary32 unless stated otherwise. Ordinary
HP percentages below are nonnegative. Shining Light uniquely uses the library's
round-to-nearest rule: `trunc(double(f(f(M) × f(.2))) + 16384.5) − 16384`.
For ordinary positive HP this is nearest integer, ties upward, **not truncation**.
Preserving that expression also states the actual source domain instead of assuming
an unlimited mathematical floor for arbitrary huge signed inputs. Exhausting native
allocation/numeric capacity is an administrative failure, not an invented game loss.

## 2. Neow: variants, ordered offers and exact payloads (S, D)

### 2.1 Which start is offered

For a standard fresh run, the character's stored spirits/boss-count value controls
the full versus short blessing: zero selects the short pair; nonzero selects the
four-option blessing. Testing override is not part of this profile. Introduction
tip state changes dialogue, not these rewards. An already-completed Neow enters
its final map dialogue without a new blessing. Daily/custom/endless branches are
not alternative standard Ironclad starts. No new native profile/time policy is
selected here.

The short pair is fixed, in order: **Neow's Lament**, or **+b maximum HP**, where
`b = T.1(M)` captured when that option is built. At M=80, b=8.

The full blessing constructs categories **0, 1, 2, 3 in that order**, drawing one
offer from each category. Categories are not merged into a uniform union. Category
2 draws its drawback first, then its reward from the resulting eligible list.
Category 3 has one candidate. Offers persist while the player chooses one; there
is no initial strategic Leave instead of choosing a blessing. Card-offer children
can subsequently be skipped; a committed removal/transform selector is not thereby
skippable. Final Neow dialogue returns to map, not an immediate discard of children.

| Category | Ordered reward candidates | Conditional offer law |
|---|---|---|
| 0 | three ordinary cards; one random rare card; remove one; upgrade one; transform one; three uncommon colorless cards | Each 1/6. |
| 1 | three potions; common relic; +b maxHP; Neow's Lament; 100 gold | Each 1/5. |
| 2 | three rare colorless; remove two unless curse drawback; rare relic; three rare cards; 250 gold unless no-gold drawback; transform two; +2b maxHP unless maxHP-loss drawback | Drawback has four equiprobable values; each eligible reward then has reciprocal list-size mass. |
| 3 | lose starter relic and obtain a boss relic | Certain category offer; boss identity uses live relic-pool law, not a fresh uniform boss catalog. |

Here each NeowReward captures its own `b` at construction. The **+2b** reward is
twice that truncated amount, not `T.2(M)`: M=79 gives +14, not +15. All M=80
starting records give +8, +16, or −8 as applicable; the percentage-current-HP
drawback at H=80 requests24, but at H=79 requests21, not23.

| Category-2 drawback | Mass | Activation request | Reward-list size; excluded reward |
|---|---:|---|---|
| maximum-HP loss | 1/4 | decrease maxHP by captured b | 6; excludes +2b |
| no gold | 1/4 | lose all live G | 6; excludes 250 gold |
| random curse | 1/4 | set a pending curse flag; later update constructs one random curse obtain effect | 6; excludes remove two |
| current-HP loss | 1/4 | HP_LOSS damage `3 × (H / 10)` using activation-time H | 7; excludes none |

The always-present category-2 rewards (rare colorless, rare relic, rare cards,
transform two) each have marginal mass **9/56**. Each conditionally excluded reward
(remove two, 250 gold, +2b) has marginal mass **5/42**. These sum to1. A particular
allowed drawback/reward pair has joint mass **1/24**, or **1/28** for current-HP
loss. Drawback and reward are therefore not independent.

### 2.2 What selecting an offer does

The drawback activation precedes reward activation. The curse drawback is deferred
to NeowReward.update, rather than being inserted before the reward's selector is
built. Its identity is drawn then from the ten-curse support in §5, not fixed at
offer construction. It may be prevented by the actual obtain path.

| Reward | Exact local payload and eligibility | Continuation / history distinction |
|---|---|---|
| three ordinary cards | Exactly3 offers. Each slot uncommon with mass5536482/16777216, otherwise common11240734/16777216; no rare. Uniform identity within current-character rarity, rejecting previously offered IDs while retaining that rarity. | Choose at most1 or skip. No Question Card/Crown count modifier or ordinary rarity-pity update. Three candidates are not three obtains. |
| random rare | One uniform current-character rare among16; immediately schedules Show obtain, not a three-card choice. | No ordinary pity reset; no card-choice skip. |
| three rare cards | Exactly3 distinct current-character rare IDs, each next accepted identity uniform among those not yet offered. | Choose at most1 or skip; no ordinary pity reset. |
| ordinary / rare colorless | Exactly3 distinct uncommon20 / rare15 colorless IDs respectively. Ordinary Neow rarity has no rare outcome; common is promoted to uncommon. | Choose at most1 or skip. No ordinary count/pity law; Bandage Up is eligible. |
| remove one / two | Choose exactly1 / 2 purgeable permanent instances. | Object-removal hooks on callback. No universal clamp to available count or invented empty-selector cancel. |
| upgrade one | Choose1 canUpgrade permanent instance; callback upgrades that actual object once. | Not a new card identity; no universal extra Searing Blow upgrade. |
| transform one | Choose1 purgeable; generate output first, then object-remove input, then schedule Show obtain. | Uses Neow RNG; source-specific §5 transform support. |
| transform two | Choose2 purgeables; object-remove both inputs first, then generate and schedule each output in selected order. | Outputs can repeat each other. Removal hooks can affect later generation/obtains; not two independent whole transactions. |
| three potions | Exactly3 independent flat33-ID offers, duplicates legal. | Ordinary combat-reward screen opens, then the first automatically generated card row is removed. That discarded card generation has already changed ordinary rarity history. Potion claims/skips are separate. |
| common / rare relic | One live normal pool draw at fixed common / rare tier. | Spawn-and-obtain, with actual onEquip child work; not screenless filtering. |
| boss swap | Remove the player's first starter relic **before** normal boss draw, then spawn-and-obtain. | Burning Blood is absent when Black Blood.canSpawn is tested, so Black Blood is rejected here. This is not a boss-chest Black Blood ban. |
| Lament | Fixed Neow's Lament relic, counter3; each battle-start with positive counter decrements it and sets every currently present monster's current HP to1. | Final charge changes counter to−2/used-up; maximum HP is not set to1. No random relic-tier draw or three-enemy total counter. |
| +b / +2b maxHP | Increase maximum by b / 2b, then the maxHP helper requests healing by that same amount. | Healing modifiers/cap apply; not an unconditional set-to-full. |
| 100 / 250 gold | gainGold(100 / 250) | Ectoplasm can suppress it. No independent gold draw. |

The selection update consumes its selected-card list once and clears activation.
Neow's waiting-to-save path waits for no screen, no top-level effects and completed
relic animation checks. That source save condition is not proof that every global
queue/terminal family is resolved, nor approval of a native save API.

## 3. Question rooms: numeric conditional law and history (S, BC, D)

A map `?` is resolved on entry: it can become MonsterRoom, ShopRoom, TreasureRoom
or an actual EventRoom while retaining the question-map symbol. A resulting fight
is marked combatEvent. It is **not** automatically one of the25 dialogue events.
Room-type selection precedes event identity selection. The previous/current room
used by the selector is still the room being left; its ShopRoom test matters.

For standard non-DeadlyEvents play, the elite bucket is absent (probability0).
Reset state is monster `f(.10)`, shop `f(.03)`, treasure `f(.02)`; no normalization
of these ramps into four fresh weights occurs. The elite chance field can still
increase internally but is unused for standard selection. The source resets
probabilities on dungeonTransitionSetup, not on every question-room entry.

Let m,s,t be truncations of the respective **binary32 products with100**; s is0
when leaving a ShopRoom. Define integer-cell intervals, half-open:

- A = [0, min(100,m));
- B = [min(99,m), min(100,m+s));
- C = [min(99,m+s), min(100,m+s+t)).

The final treasure cells are C, shop cells B minus C, monster cells A minus
(B union C), and event cells the complement in0..99. This explicitly preserves
the source's later-write precedence and saturation at cell99. Merely clipping
independent probabilities or assigning priority to the first bucket is not the
same law once ramps saturate.

Selection index is `trunc(f(100 × (k/16777216)))`, with k uniform0..16777215
under the primitive conditional float law. Thus exact probability is the count
of k mapped into the final cells divided by16777216, **not cell count/100 exactly**.
This is a finite mathematical law; no root-seed/cursor measure is implied.

| Initial state, no preceding shop / no overrides | Cells | Exact numerator / 16777216 |
|---|---|---:|
| Monster | 0..9 | 1677722 |
| Shop | 10..12 | 503316 |
| Treasure | 13..14 | 335545 |
| Event | 15..99 | 14260633 |

Those are nominal10/3/2/85 percent, with the actual finite masses above. Leaving a
shop suppresses its three nominal cells: monster0..9, treasure10..11, event12..99
at initial ramps. Each reroll reads the resulting state from all previous rolls.

| Override/history boundary | Exact mutation |
|---|---|
| Tiny Chest owned | Increment its live counter once per question-room roll. Reaching exactly4 resets it to0 and overrides the chosen room to treasure. The primitive draw still happened. Other count values are not a newly invented modulo rule. |
| Juzu Bracelet | A selected monster becomes event, but monster chance **resets to .10** rather than ramping. It does not suppress scripted Mushrooms/Dead Adventurer fights. |
| Monster history | On monster selection (including Juzu conversion), reset to f(.10); otherwise add f(.10), rounding binary32. Tiny-Chest-forced treasure takes the latter branch. |
| Shop history | On shop result reset to f(.03); otherwise add f(.03), binary32. A previous-shop suppressed roll still ramps when it does not select shop. |
| Treasure history | On treasure result, including Tiny Chest override, reset to f(.02); otherwise add f(.02), binary32. |
| Saved diagnostics | Previous chance values are captured before the roll; event-ID history is recorded when the selected event is constructed. These are not permission to expose hidden RNG/pool diagnostics to the agent. |

Example after an ordinary event result: live nominal chances20/6/4 with remaining
70 event cells. After a monster instead:10/6/4 with80 event cells. After a Juzu-
converted monster: the same10/6/4 state, **not20/6/4**. Exact subsequent masses use
the same finite index count, not an IID10/3/2/85 redraw. Invalid external actions
must not trigger another entry roll or counter update.

### 3.1 Selecting an actual event and consuming history

At an EventRoom, shrine branch has exact mass1/4, ordinary branch3/4 (strict .25
float threshold). An ordinary choice is uniform by bounded integer over the
**remaining eligible ordinary IDs**, then removes that ID from its ordinary list.
Shrine choice is uniform over the remaining six recurring-shrine IDs plus currently
eligible one-time IDs, then removes the selected ID from its owning list. Neither
means uniform over25 nor IID replacement. Skipping its reward does not restore an ID.

The full initial ordinary list contains11 IDs, with Dead Adventurer and Mushrooms
excluded until floor>6 and Cleric excluded at G<35. The recurring list contains6.
The one-time list has8 applicable Act1 IDs, of which Fountain needs isCursed and
Woman needs G≥50; Note is enabled at standard non-Daily A0 independently of victories.
Face Trader allows Exordium; the other five need no additional Act1 filter.

With all relevant lists intact, floor≤6, G≥50, no qualifying curse: ordinary
eligible count9, shrine count13. Hence each ordinary ID has mass1/12 and each
shrine/eligible-one-time ID1/52 **conditional on an actual event being selected**.
At floor>6 those ordinary masses become3/44; with a qualifying curse the shrine
count14 gives1/56. These conditional examples do not assume that the intact-list
history necessarily occurs at any particular initialized seed.

An empty eligible ordinary list delegates to shrine. A shrine-branch selection
checks raw list emptiness, not just eligible count; its combined eligible list
does not have a general proven empty fallback. If both raw lists are empty on that
branch, it tries ordinary if raw ordinary remains, else returns null. No arbitrary
retry, generated Leave event or forced combat is specified for an unclosed empty
domain. Later-act Designer/Duplicator/Knowing Skull/N'loth/Joust/Secret Portal are
not Act1 candidates, regardless of acquiring off-color cards.

## 4. All 25 event records (S unless expressly BC/D)

In these records **P** means purgeable instances minus all three bottle flags;
purgeable excludes Ascender's Bane, Curse of the Bell and Necronomicurse. **U1**
means canUpgrade instances, not distinct card IDs. Unless separately stated, the
last result/Leave dialogue calls openMap only after the local choice/child route.
That is a return destination, **not an extra initial free skip or universal drain**.
All prices are event-specific A0 prices, not merchant removal costs. Choosing an
upgrade/removal is free unless a row explicitly charges gold/HP/card/potion.

### Ordinary list: 11

| Record / actual identity | Entry and choices, exact payment/risk/reward | Selection, flags and return |
|---|---|---|
| E01 Big Fish (`Big Fish`) | No initial Leave. Banana requests captured M/3 integer heal; Donut +5 maxHP; Box fixed Regret plus one screenless random-tier relic (§5). | Box constructs curse obtain effect and spawns relic separately; prevention does not revoke the relic. Result→map. |
| E02 Dead Adventurer (`Dead Adventurer`) | floor>6. Search or Leave. On searches1/2/3, ambush probability25/50/75 percent by bounded0..99 integer. Each successful search consumes the front of the once-shuffled GOLD/NOTHING/RELIC token triple. GOLD directly requests30 gold; NOTHING none; RELIC screenless random-tier spawn-and-obtain. After3 successes, only exit remains. | Enemy fixed at construction by uniform0..2: three Sentries / Gremlin Nob / event Lagavulin. Ambush prepares25..35 inclusive fight-gold offer, monsters and eliteTrigger; next Fight adds unconsumed GOLD30 / RELIC generic-random-tier offers, then enters combat. Already consumed tokens never repeat. Victory rewards→map; EventRoom rules, not normal elite queue/drop/key. See §5 conditional token masses. |
| E03 Golden Idol (`Golden Idol`) | Take or Leave. Take first spawns Golden Idol, or Circlet if already owned; then mandatory choice: Injury; captured T.25(M) NORMAL damage; or decrease maxHP by max(1,T.08(M)). | No free post-take exit in place of a cost. Curse effect is fixed Injury, not a random curse. Relic can still have pending obtain work when the next dialogue appears. Result→map. |
| E04 Hypnotizing Colored Mushrooms (`Mushrooms`) | floor>6. No initial free Leave: Fight or Eat. Eat requests T.25(M) heal using choice-time M, then schedules fixed Parasite. Fight prepares Mushroom Lair; next Continue offers20..30 inclusive gold plus Odd Mushroom (Circlet if already owned), then enters combat. | Surviving fight uses EventRoom ordinary card/potion reward logic, not elite rewards. Reward Proceed→map directly. Eating result→map. Neither combat selection nor constructed rewards grants victory loot on death. |
| E05 Living Wall (`Living Wall`) | No initial Leave. Forget removes1 P; Change transforms1 P; Grow upgrades1 U1, with Grow disabled at entry if no upgradeable card. | All three chosen branches additionally check P nonempty before opening their grid; empty P still leads to result without a selected mutation. Change object-removes before generating output, then schedules obtain. Grow upgrades the actual instance and refreshes bottle metadata. Result→map. |
| E06 Scrap Ooze (`Scrap Ooze`) | Reach or Leave. With n prior failures: first request NORMAL damage3+n, then success count min(100,26+10n) of100 integers; success gives one screenless random-tier relic. On failure, displayed chance increases10 and next damage1. | Inclusive comparator makes nominal25 accept26/100 (BC). Conditional success numerators26,36,46,56,66,76,86,96,100; a surviving ninth Reach cannot fail. This is not a nine-action gameplay limit: death/leave can occur sooner. Leave/success→result→map. Damage precedes selection/spawn in the same local callback, not a guarantee of postlethal durable acquisition. |
| E07 Shining Light (`Shining Light`) | Enter or Leave; Enter disabled when no upgradeable card. Captured nearest-rounded binary32 .20M NORMAL damage, **before** choosing upgradeable instances. | Build U1 after damage, shuffle instances once and upgrade first min(2,size(U1)); same-kind distinct cards are eligible together. Bottle metadata refreshes. No choice grid. Result→map; exact shuffled-prefix masses remain §7. |
| E08 The Cleric (`The Cleric`) | Entry needs G≥35. Pay35 for captured T.25(M) heal; pay50 to remove1 P (gold choice disabled if G<50); or Leave. | Removal additionally checks P nonempty: only then spends50 and opens grid. Empty P goes to result without spending. Selected callback object-removes1; no merchant price escalation. Result→map. |
| E09 The Ssssserpent (`Liars Game`, class Sssserpent) | Agree or Leave. Agree first commits dialogue; next confirmation schedules fixed Doubt then requests175 gold. | Not Ssserpent Head, not a face roll, not random-curse support. Decline/result→map. Omamori can prevent Doubt without canceling the gold request. |
| E10 Wing Statue (`Golden Wing`, class GoldenWing) | Pray requests7 NORMAL damage, then a later Continue opens remove1 P; Destroy gives uniform inclusive50..80 gold if entry-time permanent deck has an Attack with baseDamage≥10; or Leave. | Requirement is one card's stored baseDamage, not combat-modified damage or total multihit damage. Purge grid has no local nonempty guard; do not invent a free replacement outcome after damage. Removal callback is later than payment. Result→map. |
| E11 World of Goop (`World of Goop`, class GoopPuddle) | Recover requests11 NORMAL damage **before**75 gold. Leave loses captured L=min(entry G, uniform inclusive20..50). No separate free Leave. | L is sampled/capped at construction, not on confirmation. At G<20, L=G certainly; at20≤G<50, each20..G−1 has mass1/31 and L=G has mass(51−G)/31; atG≥50 each20..50 has1/31. Result→map. |

### Recurring shrine list: 6

| Record / actual identity | Choices and numeric outcome | Selection, flags and return |
|---|---|---|
| E12 Golden Shrine (`Golden Shrine`, class GoldShrine) | Pray100 gold; Desecrate275 gold then schedule fixed Regret; or Leave. | No random curse, no payment deducted; gainGold guards still apply. Result→map. |
| E13 Match and Keep (`Match and Keep!`, class GremlinMatchGame) | No initial Leave: introductions then12 hidden positions,6 pairs, **5 pair attempts**. Generate one current rare16, uncommon36, common20, one uncommon colorless20, one random curse10, and fixed Ironclad event starter **Bash**. | Preview hooks run on each original, then a stat-equivalent partner is made; twelve objects shuffled once. Select two separate face-down positions; face-up chosen object cannot be selected again. IDs determine match. A match removes both and schedules a fresh makeCopy obtain (not the preview's saved upgraded state); mismatch turns both down. Decrement one attempt only after pair resolution, including fifth-match reward, then cleanup→result→map. No sixth pair attempt, no hidden-ID action ordering. Timer exact-zero completion remains §7. |
| E14 Purifier (`Purifier`, class PurificationShrine) | Remove1 P or Leave; no fee. | Opens committed grid without local nonempty guard; callback object-removes selected card. Leave/result→map; not a general empty-grid escape rule. |
| E15 Transmogrifier (`Transmorgrifier`, source spelling) | Transform1 P or Leave; no fee. | No local nonempty guard. Callback object-removes input, generates via misc RNG and §5 support, schedules Show obtain on effectsQueue. Result→map. |
| E16 Upgrade Shrine (`Upgrade Shrine`) | Upgrade1 U1 or Leave; upgrade disabled if U1 empty. | Sets room COMPLETE when choosing either branch; chosen-card callback still upgrades/refreshed bottle metadata afterward. COMPLETE alone is not child completion. Result→map. |
| E17 Wheel of Change (`Wheel of Change`, class GremlinWheelGame) | Mandatory spin; integer0..5 uniform, each1/6:100 gold / screenless random-tier relic offer / heal(M) / fixed Decay / remove1 P / HP_LOSS T.1(M). | Gold is committed at **result reveal**, not confirmation (BC-retained correction); confirmation does not award it twice. Others apply on result confirmation. Purge opens only when P nonempty; otherwise no removal. Relic is offered on ordinary open() reward screen and can be skipped; noCardsInRewards=true. Continue/children→Leave→map. Result6 is not an outcome; outcome index5 is the damage case. |

### Act1-applicable one-time list: 8

| Record / actual identity | Choices and numeric outcome | Selection, flags and return |
|---|---|---|
| E18 Ominous Forge (`Accursed Blacksmith`) | Forge upgrades1 U1 (disabled if empty); Rummage fixed Pain and Warped Tongs; or Leave. | No rummage failure lottery. Curse effect then relic spawn are separate; no existing-Tongs→Circlet substitution at this site. Upgrade callback refreshes bottle metadata. Result→map. |
| E19 Bonfire Spirits (`Bonfire Elementals`, class Bonfire) | Intro→sacrifice1 P; no initial strategic Leave. BASIC: nothing; COMMON or SPECIAL: heal5; UNCOMMON: heal(M); RARE: +10 maxHP then heal(new M); CURSE: Spirit Poop, or Circlet if Spirit Poop owned. | If P empty, CHOOSE proceeds to no-offer result. Reward calls occur **before object removal** and its hooks. Rare maxHP helper also requests heal10 before the explicit full-heal request. Parasite sacrifice loses3 maxHP afterward; do not reverse the order. Result→map. |
| E20 Face Trader (`FaceTrader`) | Intro→Touch, Trade or Leave. Touch first requests75 gold, then captured max(1,entry M/10) NORMAL damage. Trade selects one missing face; no fee/card sacrifice. | Face candidates, in pre-shuffle order: CultistMask, FaceOfCleric, GremlinMask, NlothsMask, SsserpentHead, omitting owned IDs; if none, Circlet. Shuffle remaining list once, take first. Not guaranteed40/40/20 categories after ownership changes; exact mass field is §7. Result→map. |
| E21 Divine Fountain (`Fountain of Cleansing`, class FountainOfCurseRemoval) | Entry needs isCursed: at least one CURSE other than Ascender's Bane, Curse of the Bell, Necronomicurse. Cleanse or Leave, no fee. | Cleanse traverses master deck backwards, object-removing CURSEs except those three and cards flagged inBottleFlame or inBottleLightning. This local predicate does **not** test inBottleTornado; ordinary P tests all3. Each removal runs its hooks; not one batched no-hook clear. Result→map. |
| E22 Lab (`Lab`) | Continue presents exactly3 flat33-ID potion offers at A0. No initial strategic Leave; individual potions/whole rewards can be skipped. | Clears prior room rewards, noCardsInRewards=true, room COMPLETE, calls ordinary combatRewardScreen.open(). Proceed has an explicit Lab exception returning directly to map, not to event dialogue. Full belt/Sozu affect claim, not these generated identities. |
| E23 Note For Yourself (`NoteForYourself`) | Standard non-Daily A0 enabled. Intro→take stored card or Ignore. Default stored identity Iron Wave, upgrade count0; loaded identity is freshly copied and upgraded stored-count times; misc/permanent scaling is not restored. | Take calls actual relic onObtainCard hooks, inserts the retrieved instance, calls onMasterDeckChange, **then** opens outgoing choose1 P. Thus the retrieved card may itself be stored. Selected outgoing instance is object-removed and retained by event reference; actual NOTE_CARD/NOTE_UPGRADE persistence is on nextRoomTransition leaving the event, not retrieval/selection. Ignore has no outgoing write. Direct obtain bypasses Omamori's obtain-effect constructor, not Fish/Darkstone/eggs. Result→map; native profile-output/death rules remain unapproved. |
| E24 We Meet Again (`WeMeetAgain`) | Preselect an owned potion; gold amount uniform inclusive50..min(150,G) if G≥50; one nonbasic, noncurse permanent instance. Each missing kind disables its corresponding sacrifice. Give the shown potion, shown gold, shown card, or Leave. | Potion and card choices are fixed by separate seeded shuffles at construction, not freely selected later. Card filter has no bottle exclusion and permits eligible SPECIALs; no home-event import. Sacrifice occurs before one screenless random-tier relic spawn-and-obtain. Does not offer three relics. Result→map. |
| E25 The Woman in Blue (`The Woman in Blue`) | Entry needs G≥50 although actual prices are20/30/40 for1/2/3 flat33-ID potions; or Leave for no A0 damage/payment. | Spend immediately, clear room rewards, build selected number, room COMPLETE, noCardsInRewards=true. All paid branches call **open()**, not labelled open(String). Reward Proceed resumes the event/result route, unlike Lab's direct-map exception; final dialogue→map. Potion claims may still fail for full capacity or be suppressed by Sozu. |

## 5. Actual support and source-specific random laws

### 5.1 Permanent cards, transforms, curses and potions

Neow, Match card slots and colored transforms use the **actual current character's**
initialized ordinary pools. Ironclad's supports are given below. No HEALING-tag
exclusion applies to these permanent acquisition sources: Feed, Reaper and Bandage
Up are not removed by borrowing a combat temporary-generator rule. Shard does not
widen these Neow/Match/transform selectors. Event-combat ordinary reward offers
have their own Shard policy; this does not turn fixed Parasite/Doubt or Note into
random multi-color requests.

| Support | Complete named identities |
|---|---|
| Ironclad common20 | Anger; Armaments; Body Slam; Clash; Cleave; Clothesline; Flex; Havoc; Headbutt; Heavy Blade; Iron Wave; Perfected Strike; Pommel Strike; Shrug It Off; Sword Boomerang; Thunderclap; True Grit; Twin Strike; Warcry; Wild Strike |
| Ironclad uncommon36 | Battle Trance; Blood for Blood; Bloodletting; Burning Pact; Carnage; Combust; Dark Embrace; Disarm; Dropkick; Dual Wield; Entrench; Evolve; Feel No Pain; Fire Breathing; Flame Barrier; Ghostly Armor; Hemokinesis; Infernal Blade; Inflame; Intimidate; Metallicize; Power Through; Pummel; Rage; Rampage; Reckless Charge; Rupture; Searing Blow; Second Wind; Seeing Red; Sentinel; Sever Soul; Shockwave; Spot Weakness; Uppercut; Whirlwind |
| Ironclad rare16 | Barricade; Berserk; Bludgeon; Brutality; Corruption; Demon Form; Double Tap; Exhume; Feed; Fiend Fire; Immolate; Impervious; Juggernaut; Limit Break; Offering; Reaper |
| Colorless uncommon20 | Bandage Up; Blind; Dark Shackles; Deep Breath; Discovery; Dramatic Entrance; Enlightenment; Finesse; Flash of Steel; Forethought; Good Instincts; Impatience; Jack of All Trades; Madness; Mind Blast; Panacea; Panic Button; Purity; Swift Strike; Trip |
| Colorless rare15 | Apotheosis; Chrysalis; Hand of Greed; Magnetism; Master of Strategy; Mayhem; Metamorphosis; Panache; Sadistic Nature; Secret Technique; Secret Weapon; The Bomb; Thinking Ahead; Transmutation; Violence |
| Random curse10 | Clumsy; Decay; Doubt; Injury; Normality; Pain; Parasite; Regret; Shame; Writhe |

For a transform: a CURSE input draws uniformly from random curse10 minus its own
ID if present; COLORLESS input from ordinary colorless35 minus its own ID if
present; all other colors from actual-character72 minus its own ID if present.
Thus an off-color Note card absent from red support has72 possible outputs, not
its printed character's pool. A colorless SPECIAL transforms into ordinary35,
not into its later-act event support. Normal outputs are fresh base copies;
these event/Neow calls do not request autoUpgrade. Actual obtain hooks may upgrade
them later. Multiple transforms do not reject repeated output IDs across inputs.

The Neow curse drawback and Match curse use flat10, each1/10; they exclude
Ascender's Bane, Curse of the Bell, Necronomicurse and Pride. Fixed event curses
in §4 are fixed identities, not ten-way draws. Ordinary colored or colorless
transforms never gain arbitrary Status/BASIC/SPECIAL support.

Neow/Lab/Woman use flat independent potion IDs, **each1/33 per offer**, duplicates
allowed. The full identity support (not a second potion-effect catalog) is:
Attack Potion, Blessing of the Forge, Block Potion, Blood Potion, Colorless Potion,
Dexterity Potion, Energy Potion, Explosive Potion, Fear Potion, Fire Potion,
Power Potion, Skill Potion, Speed Potion, Flex Potion, Strength Potion, Swift
Potion, Weak Potion; Ancient Potion, Distilled Chaos, Duplication Potion, Elixir,
Essence of Steel, Gambler's Brew, Liquid Bronze, Liquid Memories, Regen Potion;
Cultist Potion, Entropic Brew, Fairy in a Bottle, Fruit Juice, Heart of Iron,
Smoke Bomb, Snecko Oil. Rarity totals17/9/7 do not make these65/25/10 category
draws. Ironclad's Elixir/Blood/Heart are eligible; Silent/Defect/Watcher exclusive
potions remain excluded even with their printed cards in the deck. Potion use,
Bark payloads and merchant/combat reward quantities are separate economy records.

### 5.2 Relic selection is live history, not a catalog lottery

For every unspecified event random tier here, integer0..99 gives common50,
uncommon33, rare17 percent. Neow common/rare/boss use their fixed tier instead.
The initialized shared+Ironclad tier lists are shuffled once and depleted by
offers and rejected candidates, including unclaimed offers. Current deck, floor,
owned relics and source affect canSpawn; conditional on fully specified live
ordered lists and predicates, identity is deterministic, not equiprobable across
remaining IDs.

The already reviewed [relic identity reconciliation](2026-10-18-act1-relics-closure.md)
establishes the tier membership; that roster is **not still an unknown**. For these
event/Neow sources, the complete initial ordinary tier supports before spawn and
screenless filtering are:

| Tier | Complete candidate labels (internal IDs mapped by the linked reconciliation) |
|---|---|
| Common33 | Akabeko; Anchor; Ancient Tea Set; Art of War; Bag of Marbles; Bag of Preparation; Blood Vial; Bronze Scales; Centennial Puzzle; Ceramic Fish; Dream Catcher; Happy Flower; Juzu Bracelet; Lantern; Maw Bank; Meal Ticket; Nunchaku; Oddly Smooth Stone; Omamori; Orichalcum; Pen Nib; Potion Belt; Preserved Insect; Red Skull; Regal Pillow; Smiling Mask; Strawberry; The Boot; Tiny Chest; Toy Ornithopter; Vajra; War Paint; Whetstone |
| Uncommon30 | Blue Candle; Bottled Flame; Bottled Lightning; Bottled Tornado; Darkstone Periapt; Eternal Feather; Frozen Egg; Gremlin Horn; Horn Cleat; Ink Bottle; Kunai; Letter Opener; Matryoshka; Meat on the Bone; Mercury Hourglass; Molten Egg; Mummified Hand; Ornamental Fan; Pantograph; Paper Phrog; Pear; Question Card; Self-Forming Clay; Shuriken; Singing Bowl; Strike Dummy; Sundial; The Courier; Toxic Egg; White Beast Statue |
| Rare28 | Bird-Faced Urn; Calipers; Captain's Wheel; Champion Belt; Charon's Ashes; Dead Branch; Du-Vu Doll; Fossilized Helix; Gambling Chip; Ginger; Girya; Ice Cream; Incense Burner; Lizard Tail; Magic Flower; Mango; Old Coin; Peace Pipe; Pocketwatch; Prayer Wheel; Shovel; Stone Calendar; Thread and Needle; Torii; Tungsten Rod; Turnip; Unceasing Top; Wing Boots |
| Boss22 | Astrolabe; Black Blood; Black Star; Busted Crown; Calling Bell; Coffee Dripper; Cursed Key; Ectoplasm; Empty Cage; Fusion Hammer; Mark of Pain; Pandora's Box; Philosopher's Stone; Runic Cube; Runic Dome; Runic Pyramid; Sacred Bark; Slaver's Collar; Snecko Eye; Sozu; Tiny House; Velvet Choker |

- Normal draws consume the **front**. End draws consume the back of nonempty
  common/uncommon/rare/shop, but **front of boss**.
- A failed canSpawn consumes the rejected entry; both initial normal and end
  requests retry through the **end helper at the originally requested tier**.
- Empty common promotes to uncommon; empty uncommon to rare; empty shop to
  uncommon. These promotions call **normal/front**, including from end helper.
  Empty rare returns Circlet; empty boss returns Red Circlet.
- Screenless callers consume normal candidates but reject Bottled Flame,
  Bottled Lightning, Bottled Tornado and Whetstone, continuing at their requested
  tier. **War Paint is not screenless-excluded.** Screenless is not the separate
  noncampfire exclusion of Peace Pipe/Shovel/Girya.
- Bottles need an eligible nonbasic Attack/Skill or Power; Black Blood requires
  Burning Blood; Ectoplasm is allowed in Act1. Two owned among Girya/Peace Pipe/
  Shovel prevent the third, and these three require floor<48. Floor≤48 applies
  to Ancient Tea Set, Ceramic Fish, Dream Catcher, Juzu Bracelet, Meal Ticket,
  Omamori, Potion Belt, Regal Pillow, Prayer Wheel, Darkstone Periapt, all three
  eggs, Meat on the Bone, Question Card and Singing Bowl. Preserved Insect allows
  floor≤52; Matryoshka/Wing Boots≤40; Tiny Chest≤35. Maw Bank/Smiling Mask/Courier/
  Old Coin require non-ShopRoom and floor≤48. Ordinary Act1 floors satisfy these
  floor limits. All other listed candidates inherit base canSpawn=true. These
  shop-room exclusions are not generic event exclusions.

E01/E02 successful search/E06/E17/E24 use screenless; E02's **combat** relic offer
uses generic normal draw; Neow fixed-tier draws are generic. Fixed Idol/Mushroom/
Tongs/Poop/faces do not pass through a random common/uncommon/rare tier picker.
Full per-ID initial pool/order distribution and relevant shuffle correlations
are not replaced by50/33/17 tier rates; §7 records that exact remaining field.

### 5.3 Dead Adventurer conditional category record

The pre-shuffle token order is GOLD, NOTHING, RELIC (indices0,1,2). For the
**explicitly hypothetical uniform64 Java-seed input measure**, previously reviewed
exact mapper counts over281474976710656 internal seeds are:

| Token index order | Numerator |
|---|---:|
| 0,1,2 | 46912496118442 |
| 0,2,1 | 46912496115666 |
| 1,0,2 | 46912496118441 |
| 1,2,0 | 46912496118369 |
| 2,0,1 | 46912496121215 |
| 2,1,0 | 46912496118523 |

These are not exact1/6. A successful observed prefix conditions the remaining
orders by their compatible numerators: next-token probability is the sum for
that next token divided by the sum for the observed prefix. For example, after
GOLD first, NOTHING next has46912496118442/93824992234108 and RELIC next has
46912496115666/93824992234108. Search risk remains25/50/75 percent conditional
on surviving successes and choosing to continue. This table neither selects the
native shuffle input measure nor proves independence from enemy/risk streams.
The actual initialized/history-conditioned joint measure remains §7, rather than
silently treating these numbers as unconditional original gameplay frequencies.

## 6. Payment, hooks, prevention, legal death and durable commitment

These boundaries are part of every numeric row, not permission to erase local
order or run one universal transaction/return algorithm:

| Operation | Actual local/downstream distinction |
|---|---|
| gainGold | Ectoplasm returns before adding gold/hooks. Otherwise positive amount is added then relic onGainGold callbacks run. A nominal175/275/100/75 is not always that net gold. Costs use loseGold, not negative gainGold. |
| heal / maxHP gain | Healing returns if already dying; actual relic onPlayerHeal and power onHeal hooks apply, then current HP caps at maxHP. increaseMaxHp first adds its amount and then requests healing by that amount. Bonfire's later heal(maxHP) is an additional request. |
| maxHP loss / Parasite | decreaseMaxHealth subtracts, floors maxHP at1, clamps current HP down to maxHP. It is **not damage or automatic death**. Parasite object-removal calls this with3. That local hook does not prove all arbitrary removals nonlethal. |
| NORMAL damage / HP_LOSS | Ordinary event DamageInfo calls in §4 are NORMAL; Neow current-HP drawback and Wheel damage explicitly HP_LOSS. Golden Idol/Ooze/Face/Wheel/Neow pass null damage owner; Wing/Goop/Shining pass the player. Do not normalize those owner fields: actual damage/rescue/death hooks own net HP and terminal arbitration. Lethal Wing/Ooze/Goop/Face choices remain legal; no preemptive native ban is justified by a later reward/removal. |
| object removal | Remove chosen instance, run card onRemoveFromMasterDeck, then ordered relic onMasterDeckChange. String/iterator removals elsewhere are different paths. Fountain processes cards backwards; Bonfire rewards before this removal. No all-effects rollback on later failure/death. |
| Show / Fast obtain and Omamori | Omamori prevents eligible curse obtains and spends a charge at construction. Normal desktop raw-delta>0.1 frames return before game update; an admitted first Show update cannot exhaust its ≥.5-second initial duration before the already-done object is removed. Thus normal prevented **Show and Fast both suppress permanent insertion and obtain/deck-change hooks**. Their one-time constructor bookkeeping and entered-update behavior still differ. Oversized-delta isolated-method counterfactual is not a legal phantom-curse route. |
| actual card obtain | If not prevented, obtain hooks, permanent insertion and deck-change notifications occur at the route's actual later effect/soul boundary; constructing a reward or parent COMPLETE is insufficient. Ceramic Fish requests9 gold/card, Darkstone requests+6 maxHP/acquired curse, eggs upgrade matching upgradeable not-already-upgraded cards when their hooks run. Mere preview/rejection is not durable acquisition: explicit preview hooks (including Match's egg previews) still run, without inventing Fish/Darkstone obtain hooks for offers. |
| Note direct obtain | Calls actual hooks/insertion directly; it bypasses Omamori effect prevention, not Darkstone/Fish/eggs. Retrieved card can be the outgoing choice. Later room-transition persistence remains separate from native profile-output policy. |
| potion reward claim | Sozu's reward-claim branch removes the row without obtaining. Without Sozu, a full belt rejects claim and leaves the row available; generation did not require capacity. Paid Woman rewards need not refund event payment if skipped/unobtainable. |
| event combat | Dead Adventurer eliteTrigger does not make the room MonsterRoomElite. Both event combats have ordinary EventRoom rarity/history, no elite-room relic drop/emerald or elite-queue pop, no Prayer Wheel extra ordinary-MonsterRoom offer. Existing token/fixed-event relic offers stay distinct from auto rewards. |
| map / terminal | Lab, Mushrooms and Dead Adventurer have explicit reward-Proceed direct-map exceptions. Woman and Wheel use their event/result return. Local callback continuation after a damage call is not proof a later queued child survives every terminal interruption. +1 is only fully resolved Act1Complete/Act2Ready; actual death −1; unresolved children cannot be dropped to manufacture either. |

For Note, an unrecognized source-library stored ID falls back to Iron Wave before
fresh copying. This describes the source's fallback, not approval to accept malformed
native profile descriptors. The legal descriptor/output contract remains separately
owned; neither fabricated prior misc nor another character's home event follows.

Singing Bowl remains a source-specific card-reward alternative when applicable,
not another randomly generated event card: its request is +2 maxHP, whose helper
requests heal2; there is **no second explicit heal2 call** in the button. Taking/skipping/
Bowl callbacks must retain their actual associated reward/parent semantics;
Neow's null parent item is not license to invent a new linked reward row.

## 7. Actual unclosed fields, not unspecified event amounts

All25 local branches and Neow amounts above are stated, not deferred to a private
import or generic “unknown rules.” The following **specific composed fields** are
still blockers to a complete whole-Act1 claim:

| Field / affected sites | Established information | Remaining obligation |
|---|---|---|
| seeded-shuffle input/joint measure | Dead Adventurer six numeric mapper counts above; exact current supports/order and no-replacement consumption for all sites | Initialized/history-conditioned seed measure and relevant joint dependence; uniform64 is not silently approved. |
| larger shuffle outcome law | Face missing subset≤5; Match12 labeled positions then identity/reveal history; Shining first min(2,n) U1; WeMeet potion/card separate captured lists; live depleted relic pools | Exact finite observable pushforwards/posteriors for these varying lists and jointly relevant history. Do not assume uniform faces, unordered pair upgrades, or uniformly permuted boards. Conditional on supplied complete ordered state/output shuffle, local mutation is specified. |
| transitive relic callback effects | Reviewed tier roster, all32 candidate canSpawn overrides, normal/end/retry/fallback/screenless laws and actual fixed rewards | Whole callback/death/nested-screen composition for each actually obtained relic remains separate relic-mechanics/execution work; a reconciled roster is not that proof. |
| exceptional selectors | Local empty guards identified per site; Neow exact counts and unguarded Purifier/Transmogrifier/Wing/Note distinguished | Which exceptional empty/short states have initialized legal-history witnesses, and their complete screen/terminal behavior. No generic count clamp, cancel, reoffer or failure-as-defeat. |
| Match timers | Five resolved attempts, fifth match included, entered-Play cleanup starts with positive wait on normal admitted desktop frames | Strict-negative versus exact-zero cleanup crossing and a separately reviewed native elapsed-time abstraction. Native timer choice is not selected here. |
| downstream death/parent families | Actual local source order, guarded healing, curse prevention, explicit direct-map exceptions | Full callback/terminal arbitration for every interruption/rescue/child family; event-specific escape/Smoke Bomb resolution cannot be inferred from eliteTrigger. |
| Note profile | ID/upgrades reconstruction, no stored misc, direct take before outgoing selection, next-room source persistence | Native descriptor admissibility and output on legal death/fault/reset/replay are profile-owner/human decisions. No death-equals-administrative-fault or success-only publication assumption. |

The reviewed [shuffle-measure boundary](2026-10-18-act1-rng-map-closure.md)
supplies context, not an implicit choice of measure. The
[profile/time draft](2026-10-19-act1-profile-time.md) remains a draft; this document
does not adopt its unapproved options. Seed/cursor nonparity under RNG-001 does not
waive the listed conditional/joint mechanic laws.

## 8. Discriminating future cases — NOT EXECUTED GAMEPLAY

1. Neow M=H=79: b=7, +2b=14, current-HP drawback21. Curse excludes remove-two;
   no-gold excludes250; maxHP loss excludes+2b; damage leaves7 candidates.
2. Compare the three Neow ordinary slots with a normal reward: no rare/pity/count
   modifiers for Neow; rare direct/triple draws do not reset ordinary pity.
   Neow potions' discarded auto card offer **does** alter ordinary history.
3. Question reset→event versus reset→Juzu-monster: next nominal states20/6/4 and
   10/6/4; Tiny Chest fourth roll still draws, forces treasure, resets its counter,
   ramps monster/shop and resets treasure. Test binary32 ramps and saturated cell99.
4. At floor6/7 and G34/35/49/50 vary event eligibility without restoring consumed
   IDs. A lone Bell curse does not qualify Fountain; ordinary Parasite does.
5. Scrap first roll73 fails,74 succeeds; ninth surviving reach has100/100 success.
   Do not interpret a lethal damage call as “no random proposal was locally reached.”
6. WeMeet charged50..150 is construction-time; its card may be bottled. Fountain
   excludes Flame/Lightning flags locally, while P excludes Tornado too.
7. Bonfire rare: +10 maxHP helper/heal then full-heal request, then removal. Parasite
   sacrifice: fixed reward path before maxHP−3; floor1/clamp is not HP_LOSS damage.
8. Wheel gold is already committed at reveal; repeated confirmation must not add100
   again. Relic case offers, does not force obtain. Woman open() returns to event;
   Lab open() has explicit direct-map Proceed.
9. Normal prevented Show/Fast curse never inserts or fires Fish/Darkstone/deck-change;
   Note direct curse can do so. Do not use an excluded >.1 desktop frame to create
   a phantom Omamori curse. Preserve one-time source-specific bookkeeping.
10. Off-color Note input transforms to red72 when not colorless/curse, with no
    same-ID exclusion if absent; SPECIAL colorless transforms to ordinary35.
    Face Trader can give SsserpentHead; Ssssserpent gives Doubt+175, never that face.
11. Final Match match on attempt5 still obtains one fresh copy before cleanup;
    no upgraded-preview alias or hidden-ID candidate ordering. Repeated card kinds
    in Shining Light are distinct selectable upgrade instances.

## 9. Source-claim ledger and actual verification

Private retained evidence root **A** is
`verification/2026-10-18-act1-followup-v1/acquisition/` in the private verification
workspace; class/member locators below use its `classes/<class>/source/` layout.
Source and detailed command output remain private. Reviewed corrections, rather
than the frozen initial report's obsolete summaries, control relic depletion and
Wheel gold. The newer continuation review's desktop-frame and Woman-overload
corrections are incorporated here without treating its unreviewed initial prose
as a source fact.

| Claim group | Actual members inspected / private evidence |
|---|---|
| Neow variants, ordered categories, payloads and flags | A neow/NeowEvent constructor, buttonEffect, miniBlessing, blessing, update; neow/NeowReward constructors, getRewardOptions, getRewardDrawbackOptions, activate, update, getRewardCards/getColorlessRewardCards/rollRarity |
| Question law | Fresh own exact-unique helpers/EventHelper class: roll/resetProbabilities BC offsets0–741 and0–23; retained source plus fresh reconstruction. AbstractDungeon.nextRoomTransition and dungeonTransitionSetup callers. |
| Exact rounding / direct payload types | Fresh own exact-unique libGDX MathUtils.round BC and reconstruction; DamageInfo constructors establish default NORMAL; NeowsLament constructor/atBattleStart/setCounter BC closes counter3→−2 and current-HP-only effect. |
| Event lists and entry guards | A dungeons/Exordium.initializeEventList/initializeShrineList; AbstractDungeon.initializeSpecialOneTimeEventList/isNoteForYourselfAvailable/generateEvent/getEvent/getShrine; AbstractPlayer.isCursed |
| E01–E11 | A events/exordium/BigFish, DeadAdventurer, GoldenIdolEvent, Mushrooms, LivingWall, ScrapOoze, ShiningLight, Cleric, Sssserpent, GoldenWing, GoopPuddle: actual constructor/button/update/reward/selector bodies; CardHelper.hasCardWithXDamage |
| E12–E17 | A events/shrines/GoldShrine, GremlinMatchGame, PurificationShrine, Transmogrifier, UpgradeShrine, GremlinWheelGame: actual constructor/button/update/match/cleanup/preApplyResult/applyResult bodies |
| E18–E25 | A events/shrines/AccursedBlacksmith, Bonfire, FaceTrader, FountainOfCurseRemoval, Lab, NoteForYourself, WeMeetAgain, WomanInBlue: actual constructor/button/update/selector bodies |
| Pools and source-specific support | A policy-supports.json plus AbstractDungeon card/transform/relic helpers, CardLibrary.getCurse, PotionHelper.getRandomPotion; retained reviewed generation constructor metadata, not combat-pool analogy |
| Downstream hooks/returns | A CardGroup removal/purge/bottle helpers, Parasite.onRemoveFromMasterDeck, Darkstone/Fish/egg hooks, Fast/Show/Soul, RewardItem.claimReward, CombatRewardScreen/CardRewardScreen, ProceedButton; retained AbstractCreature maxHP/heal and SingingBowlButton.onClick bodies; continuation independent review's fresh desktop/render/backend chain |

Own private lane `verification/2026-10-19-act1-spec-closure-v1/content-event-neow/`
retains `claim-ledger.json`, input hashes, `static-commands.json`, four-pin before/
after checks, failure-first and negative-control helper tests, `arithmetic.json`,
and document/path/link/privacy/whitespace checks. Static scope was four fresh
classes/eight JVM tool commands, not a blind archive reconstruction. Java injection
variables were stripped; no game class/initializer/runtime was executed. Arithmetic
checks independently validate documentary finite counts, not original gameplay.

The current-wave documentation baseline exception covers only the already recorded
two CUDA-environment failures, conditional on no new failures and independent exact-
revision review. It is not a green-suite claim or permission for product/test fixes.
This author did not rerun product builds, install dependencies, use GPU, benchmark,
train, mutate main, merge, clean up branches or spawn nested agents. Remaining
mechanic fields are exactly §7, not waived by publication of these numeric records.
