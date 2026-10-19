# Act 1 relic primary rules and shared consumers

Date: 2026-10-19. Bead: `spire-gli.2.3.5`.

**Supporting static findings, pending independent review. Not whole-Act-1 closure,
an approved native implementation contract, or runtime validation.**

## 1. Result and evidence boundary

This advances the [accepted relic inventory](2026-10-18-act1-relics-closure.md)
from identities/overrides to primary conditions, mutations, counters and actual
shared consumers. All 141 primary rows are reconciled below. In particular, the
13 classes without a primary override are **not inert**: their consumers implement
damage modifiers, potion appearance, Block/energy conservation, debuff prevention,
end-turn hand handling, X effects, information visibility, play cleanup, gold
bonuses and score. Local rules are established; their complete transitive
execution/continuation composition is not thereby established.

Authority is the owned archive SHA256
`cfad868ac8d65a88e71a0bf096fb09f78811e553effe0787c5309a655e081673`.
The JAR, Temurin 21.0.12.1+1 Java/javap and CFR 0.152 pins were checked before and
after inspection. Evidence consists of the retained 141 relic sources, fresh
targeted shared-class reconstruction/disassembly, and read-only reviewed acquisition
and execution findings. No original class, initializer, game runtime, substituted
runtime, profile, launcher or native library executed. Static tools ran with Java
option-injection environment variables removed. Source/build universality is not
asserted.

A targeted exact UTF-8 constant-pool search covered 2,306 game-class entries. It
records candidate consumers of the 141 literal IDs and, separately, the Chemical X
action-class names. Actual branches were then read. This is a bounded direct-ID
consumer inventory, **not** a dynamic call graph: generic callback dispatch,
indirect arguments and payload callees require their own evidence. Registration,
unlock, presentation, event acquisition and later-act references are distinguished
from Act 1 effect consumers. Source absence in the relic subclass proves neither
inertness nor caller closure.

Private evidence locators, not public dependencies: wave
`2026-10-19-act1-spec-closure-v1/relic-mechanics`, its claim ledger and `shared-1`
through `shared-5` manifests; retained wave-v2 `relics/classes`, its two correction
directories; reviewed followup-v1 acquisition/execution and their review records.
Only independently worded findings appear here; no source, executable translation,
helper, fixture or raw output is published.

Scope remains fully unlocked, unmodded Ironclad A0 Act 1, including legal Shard/Note
off-color content. Card color never changes the run's actual character or grants
another character's exclusive relic pool. The
[native contract draft](2026-10-18-act1-native-contract-closure.md) remains a draft.

## 2. The 13 shared-site rules

Signatures use Java source notation; these are evidence locators, not a proposed
native API. `NORMAL`, `THORNS` and `HP_LOSS` are distinct damage types.

| Relic ID / consumer | Exact local condition and effect | State, ordering and boundary |
|---|---|---|
| `Paper Frog` — `VulnerablePower.atDamageReceive(float, DamageType): float` | A nonplayer owner with Vulnerable receives NORMAL damage with factor 1.75 rather than 1.5 while the player owns the relic. Other damage types pass through. | Ownership is read during damage calculation; no relic charge/reset. The power's description also reports 75%. This does not add Vulnerable or multiply damage lacking that power. |
| `White Beast Statue` — `AbstractRoom.addPotionToRewards(): void` | Sets appearance chance to 100; **then** four or more existing room rewards set it to zero. A 0..99 potion roll is still made. | Success appends a potion reward and reduces shared potion appearance offset by 10; failure increases it by 10. Ownership does not bypass row-limit suppression, Sozu or belt capacity at claim. It changes appearance, not potion identity law. |
| `Calipers` — `GameActionManager.getNextAction(): void`, next-turn branch | Without Barricade **and** Blur, turn-boundary Block loss is 15 instead of all Block. `AbstractCreature.loseBlock(int, boolean)` floors remaining Block at zero. | Start-turn relic/card/power/orb hooks have already been called; their queued actions have not all executed. The direct Block reduction precedes the newly queued draw. Barricade/Blur suppress both ordinary loss and Calipers loss. No relic counter or spend. |
| `Ginger` — `ApplyPowerAction.update(): void` | On the first application checkpoint, player target plus power ID `Weakened` prevents application. | Target validity and source `onApplyPower` callbacks precede this gate; the gate precedes Artifact consumption and power insertion/stacking. Does not cleanse existing Weak or protect monsters. This establishes this action path, not every direct `addPower` call. |
| `Ice Cream` — `EnergyManager.recharge(): void` | Adds the recharge amount to remaining energy instead of replacing it. | Its branch precedes Conserve, so Conserve's queued reduction is not produced when Ice Cream is owned. `EnergyPanel.addEnergy(int)` has a source-backed **999 ceiling**; `prep()` copies master recharge energy and sets current energy to zero at battle preparation. No across-combat energy carry is implied. |
| `Turnip` — `ApplyPowerAction.update(): void` | Same prevention boundary as Ginger, for player-target power ID `Frail`. | Before Artifact consumption; no existing-Frail removal, monster immunity or relic counter. Source callbacks can already have run. |
| `Runic Pyramid` — `DiscardAtEndOfTurnAction.update(): void`; `RetainCardPower.atEndOfTurn(boolean): void` | Suppresses ordinary bulk end-turn discard and the Retain Cards power's selection action. Equilibrium independently suppresses both. | Explicit retain/self-retain cards are still moved out and restored by their own path. Ordinary Pyramid-held cards do **not** gain retain flags or `onRetained` calls. Remaining hand end-turn callbacks still run; Ethereal is not disabled. See below for ordering limits. |
| `Chemical X` — ten current card actions, below | Adds two to the action's effect count after its captured-energy override, before its positive-effect test and action-local upgrade increment. | Not +2 spendable energy, not a global X-cost rewrite. Payment is action-local, generally all live current energy when not free. Orb/target/action-specific gates remain; Conjure Blade's upgrade increment is passed upstream. |
| `Frozen Eye` — `DrawPileViewScreen.open(): void`; `ExhaustPileViewScreen.open(): void` | Skips the view copy's ordinary alphabetical/rarity sorting. The draw view's reversed copy exposes upcoming draw order. Exhaust view also preserves its copied sequence rather than sorting. | No draw-pile shuffle or mutation is introduced by this rule. Draw-panel/help text changes separately. The player-class literal hit is a development-loadout reference, not a second gameplay modifier. Native observations must expose legitimate order with ownership, not hidden order without it. |
| `Strange Spoon` — `UseCardAction.update(): void` | For exhaust-marked non-Power cleanup reaching this branch, one fair Boolean selects ordinary destination handling instead of actual exhaust. | `onAfterUseCard` and free/autoplay flag clearing precede it; purge and Power removal return **before** the draw. On success precedence is rebound, shuffle-back, return-to-hand, otherwise discard. It is not always discard, and does not intercept independent exhaust actions/Ethereal exhaust. Actual exhaust callbacks occur only on the exhaust path. |
| `Golden Idol` — `RewardItem.applyGoldBonus(boolean): void` | For a non-theft gold reward outside `TreasureRoom`, bonus is the source's rounded binary32 base-gold × 0.25. | Bonus is stored when constructed and recomputed on gold increment; claim passes stored base plus bonus through `gainGold`. It is **not** a universal gainGold multiplier: direct event/Fish/Coin gold and stolen-gold refunds do not acquire it by that route. Ectoplasm can still block actual gain. |
| `Odd Mushroom` — `VulnerablePower.atDamageReceive(float, DamageType): float` | Player-owned Vulnerable's NORMAL factor becomes 1.25 instead of 1.5; other damage types unchanged. | This is independent of Paper Frog's nonplayer branch, not a combined player factor. Description reflects 25%; no charge/reset or automatic removal of Vulnerable. |
| `Spirit Poop` — `GameOverScreen.checkScoreBonus(boolean): int` | Ownership sets the score presentation flag and subtracts one from this score bonus, independently of the victory Boolean. | Not HP loss, gold loss, combat debuff or an Act-1 task reward. Ownership still exists as an ordinary relic instance and can matter to generic relic-count predicates. No original full-run score is computed merely by reaching Act1Complete. |

For Pyramid, the end-turn action first sets aside explicitly retained cards, queues
their restoration, conditionally queues bulk discard, then visits a shuffled copy
of the remaining hand for end-turn callbacks. Restore invokes `onRetained` only
for the retain/self-retain entries and clears temporary retain. `AbstractCard`'s
Ethereal callback queues its independent exhaust. Those entered callbacks,
pending exhaust/discard/restoration actions and actual pile membership are not one
atomic event. The unseeded shuffle's observable joint order and transitive card
callbacks remain execution/random-law dependencies; Pyramid does not waive them.

Golden Idol's additional literal hits include its acquisition, event registration
and later-act ownership branches. Those do not add later-act events to Act 1.
Its reward condition is actual `TreasureRoom` inheritance, not a rule inferred
from the English word “chest”; boss-chest reward producers retain their own context.

### 2.1 Chemical X: finite current-consumer matrix

For the following actions, effect basis is captured `energyOnUse` when not -1,
otherwise live energy at action update. Add Chemical X's two at that boundary.
Except Conjure Blade, positive effect is required before payload/payment; nonfree
payment spends **live** energy, not the captured basis. Multicast first requires
an orb. Enqueue is not proof every later payload survives cancellation.

| Current card / action | Effect after relic adjustment | Local payload / distinction |
|---|---|---|
| Reinforced Body / `ReinforcedBodyAction` | adjusted count | That many Block actions, each using supplied Block amount. Upgrade changes amount, not count here. |
| Doppelganger / `DoppelgangerAction` | adjusted count +1 if upgraded | Next-turn energy and next-turn draw powers of that amount. |
| Malaise / `MalaiseAction` | adjusted count +1 if upgraded | Negative Strength then Weak applications of that amount; Artifact applies separately to those actions. |
| Multi-Cast / `MulticastAction` | adjusted count +1 if upgraded | Count−1 evocations without removal, then final evoke/removal; no-orb branch produces neither boost processing nor payment. |
| Skewer / `SkewerAction` | adjusted count | That many targeted damage actions with captured damage/type. |
| Tempest / `TempestAction` | adjusted count +1 if upgraded | That many Lightning channel actions; actual orb-slot behavior remains a payload rule. |
| Transmutation / `TransmutationAction` | adjusted count | Card use first raises energyOnUse to live energy if lower; action then makes that many colorless combat-card proposals, upgrades each if upgraded, sets turn cost zero and enqueues hand generation. |
| Whirlwind / `WhirlwindAction` | adjusted count | That many all-enemy damage actions with supplied damage array/type. |
| Collect / `CollectAction` | adjusted count +1 if upgraded | Collect power of that amount. |
| Conjure Blade / `ConjureBladeAction` | adjusted count; upgraded card passes energyOnUse+1 into action | Always constructs Expunger with that X and queues draw-pile creation, then nonfree live-energy payment; no positive-effect guard in this action. |

Two other action classes contain the ID: `BrillianceAction` has no incoming direct
action-class reference outside itself in the scanned namespace;
`DivinePunishmentAction` is referenced by `DEPRECATEDCleanseEvil`. They are not two
extra current card laws. The ten named current cards do directly construct their
listed actions. Do not make Chemical X affect ordinary current Brilliance merely
because a similarly named legacy action exists. This bounded scan does not prove
arbitrary reflection/mod reachability, which is outside scope.

## 3. Shared primary boundaries that override-only inventories miss

1. **Equipment is not one synchronous grant.** Ordinary relic obtain has a delayed
   base update path; instant obtain installs/calls equipment immediately. Bottle,
   Astrolabe, Cage, Mirror, Bell and Pandora have additional latches/updates. Room
   COMPLETE or a reward timer alone does not prove child effects committed.
2. **Resource helpers matter.** `increaseMaxHp` raises maximum and calls healing;
   its Boolean is not “do not heal.” Healing applies ordered relic modifiers then
   power modifiers and clamps to max HP. Magic Flower checks the live room's COMBAT
   phase, including qualifying endBattle healing before phase changes. Waffle's
   `false` argument still permits its +7-associated heal, followed by its full-heal
   request. A queued heal is not an unconditional resurrection.
3. **Gold callbacks are source-specific.** Ectoplasm rejects `gainGold` before gain
   callbacks. Maw Bank stops from `onSpendGold`; player `loseGold` invokes that
   family in ShopRoom, before its positive-amount test. Ordinary event loss is not
   automatically spending at a merchant. Fish/Coin/Maw/Head gold still uses gainGold.
4. **Start preparation has explicit calls.** Slaver's Collar uses
   `beforeEnergyPrep()`, not a primary override: eliteTrigger or a BOSS enemy adds
   one master energy before preparation; its victory hook removes that increment
   if its activation pulse is set. Actual EventRoom eliteTrigger can qualify
   without becoming MonsterRoomElite for Black Star/rewards.
5. **Card callbacks are not interchangeable.** Nunchaku/Ink/Pen Nib and three-card
   counters use `onUseCard`; Pocketwatch/Choker use `onPlayCard`. Common dispatcher
   history may precede a later dead-target rejection. Cleanup's purge copies do not
   become additional cycling/exhausted cards. Complete shared callback inventories
   remain execution-lane work.
6. **Actual exhaust notifies before insertion.** `CardGroup.moveToExhaustPile`
   invokes relics, powers and the card's exhaust trigger before reset/visual effect/
   exhaust insertion and draw-or-discard refresh. Branch selects its random identity
   in the callback, then enqueues generation; its all-basically-dead guard and
   Charon's unguarded queued damage are different. Spoon success never enters this
   method merely because the card was marked exhaust.
7. **Block-loss and Block-break differ.** Calipers' `loseBlock` does not call the
   monster `brokeBlock` notifier. Damage exhausting positive monster Block, including
   exact equality, does notify Hand Drill. HP_LOSS bypasses Block decrement. The
   Block gain helper separately caps actual Block at the source-backed 999.
8. **Eligibility is not selection.** Bottles' spawn checks require nonbasic
   Attack/Skill or a Power, but equipment selection uses purgeable cards of that
   type, including eligible basics. Selected instance flags drive `initializeDeck`'s
   top-placement alongside Innate; a battle-start flash is not the draw mechanism.
   Additional opening draw is scheduled when that group exceeds master hand size.

The name `onVictory` is a callback, not an independently proven win-only guard.
The inspected `endBattle` path calls Meat on the Bone then player victory hooks;
player dispatch checks `!isDying`. Smoke/escape callers and genuine task outcomes
must be resolved by their own paths, not by adding a blanket smoke exclusion to
these relic methods. Conversely, a future queued heal does not undo established
death. Shared terminal arbitration remains X.

### 3.1 Delayed equipment: precise latches, not a generic return-to-map

| Relic | Equipment boundary | Later local commit / exact residual |
|---|---|---|
| Three bottles | If eligible typed purgeable cards exist, mark selection pending, preserve active screen, mark room INCOMPLETE, open one-card selection. | Update accepts nonempty selection, stores selected permanent instance/flag, marks COMPLETE, clears selection. Unequip clears that instance's flag. Full parent return is H below. |
| Astrolabe | Empty purgeable group returns without opening selection; size≤3 processes all immediately; otherwise requests exactly three. | Marks latch complete before per-card object removal, upgraded transform and scheduled obtain; clears selected cards and adjusts reward timer. Removal/obtain hooks and any lethal interruption are separate. |
| Empty Cage | Marks room INCOMPLETE before checking group. Empty sets its latch complete and returns; size≤2 deletes all, else requests two. | Delete removes actual objects, marks COMPLETE and clears selection. The empty branch does not itself restore COMPLETE: exceptional-state reachability/parent handling must not be invented. |
| Dolly's Mirror | Marks pending and opens master-deck one-card selection, preserving parent screen. | On exactly one selection: stat-equivalent copy, clear all three bottle flags, schedule obtain, clear selection, mark COMPLETE. Empty master-deck behavior is a continuation obligation, not a free no-op inferred here. |
| Calling Bell | Opens fixed Bell-curse confirmation; marks reward latch pending. | Once screen is down, opens rewards, clears the generated rows and supplies screenless common/uncommon/rare offers once. Intermediate discarded card generation may already mutate reward history. Curse/relic durable obtain and parent return remain separate. |
| Pandora's Box | Iterator-removes STARTER_STRIKE/STARTER_DEFEND-tagged instances; directly generates/preview-hooks one replacement per removal; opens confirmation only for positive count. | Later non-GRID update sets its latch and reward timer. Iterator removal is not object-removal callbacks; confirmation obtains are not immediate at equipment. Zero-removal branch has its own update, not a fabricated chooser. |

## 4. All-141 primary-law reconciliation

Each row uses the **constructor ID**, not necessarily its display label. The
linked accepted inventory supplies label/class/tier/source mappings and all 32
spawn overrides; it is not copied as a substitute for these laws. Default evidence
locator is the mapped relic class and the named trigger; §2/§3 name the actual
shared consumer where the subclass is insufficient. Values describe unmodded local
behavior, not a promise queued work executes after death/combat clear.

All rows have a source-established local primary rule. The last column names
remaining **composition proof obligations**, not missing/inert mechanics:

| Code | Exact remaining boundary / owner |
|---|---|
| X | Complete entered/current/pending callback/action order, cancellation, terminal and play-origin composition — execution lane. Local enqueue/mutation is established, universal scheduler closure is not. |
| H | Acquisition/removal/preview/obtain and source-parent return/commit combinations, including empty/depleted selection — continuation lane. |
| G | Full source-specific generated payload or shuffle conditional/joint law and history. Already reviewed sublaws remain retained; no uniform-permutation or independent-stream default — generation/shuffle integration. |
| P | Specific transitive power/potion/card payload semantics not fully certified by this relic audit; retain complete content requirement rather than call it closed. |
| V | Native legitimate-information projection/remembered knowledge and schema integration, not a new source-visibility uncertainty — native-contract integration. |
| R | Local direct resource/calculation rule traced through its helper; no additional relic-specific callee gap identified for that stated mutation. This does not remove X/H where also present or certify every legal combined state. |

P is intentionally not a claim of transitive closure. Its finite payload endpoints
are as follows; the applicable row names the particular power/action/card family.

| Primary rows | Traced endpoint / remaining transitive obligation |
|---|---|
| Akabeko, Pen Nib, Clay | Vigor NORMAL addition/Attack removal, Pen Nib NORMAL doubling/Attack removal, and NextTurnBlock start enqueue/removal were read. Remaining damage-calculation, power-order and current/pending composition is not certified by those local power bodies. |
| Marbles, Ginger, Turnip, Belt, Mask, Frog, Mushroom, Drill | ApplyPowerAction's source callbacks/prevention/Artifact/insertion order and Vulnerable's factors are traced. Full Weak/Frail/Vulnerable stacking, duration, Artifact consumption updates and all direct-application alternatives remain shared power obligations. |
| Vajra, Smooth Stone, Skull, Kunai, Shuriken, Girya, Doll, Stone, Brimstone, Sling | Exact queued/direct Strength/Dexterity amounts and activation/counter conditions are established. `StrengthPower`/`DexterityPower`, all card damage/Block override paths and their interaction ordering are not comprehensively certified here. |
| Scales, Helix, Thread, Incense, Souvenir, Snecko, Pellets | Named Thorns/Buffer/PlatedArmor/IntangiblePlayer/Artifact/Confusion powers and `RemoveDebuffsAction` are the remaining payload families; no universal retaliation, damage-prevention, duration or cost-restoration result follows merely from their application. |
| Preparation, Puzzle, Horn, Ink, Cube, Pocketwatch, Top, Chip | DrawCardAction's actual draw/No Draw/capacity/shuffle and draw-trigger closure; Chip also manual-discard reactions. Chip's local selection retrieval/discard/count/redraw enqueue was read, not every resulting draw/discard callback. |
| Boot, Torii, Tungsten, Hourglass, Opener, Ashes, Calendar, Lament, Insect | Exact local modifier/HP-setting/enqueue facts are established. All damage-all/current-target/HP-loss/death callbacks and alternate damage producers remain transitive obligations. Insect's extra literal consumer adjusts visual scale only, not spawned HP. |
| Eggs, Candle, Kit, Pyramid, Mirror, Tongs, Choker | Per-card canUse/canUpgrade/upgrade/copy/end-turn/exhaust overrides and origin-dependent legality; no Status→Skill conversion, synthetic retain or blanket autoplay bypass. |
| Branch, Mark, Shard, Toolbox, Chemical X | Generated card/Status/choice/orb/Expunger payload, hand/full/zero-slot behavior and their actual callbacks remain required. Chemical X's count/payment is closed locally, not every produced orb/card action. |
| Bark, Sozu, Cauldron, Flower | Full reachable potion data/use/choice/copy payload and power onHeal catalog. Bark's numeric potency helper and five explicit choice/duplication consumers were read; that is not every potion law. Flower's relic-stage multiplication is closed, not every healing-power outcome. |

G likewise retains precise site dependencies: upgrade-prefix Java shuffles;
initialized ordered relic pools/rejection/depletion; Tiny Chest/Juzu room history;
ordinary/Prayer Wheel/Orrery/Tiny House/Bowl reward-helper count/history;
merchant/Courier prices and restock; Mummified Hand's eligible-instance draw;
Branch/Toolbox/Transmutation source supports; Mark's insertion; Tongs' hand shuffle;
Spoon's Boolean and its shared cursor relationship; and Confusion's draw costs.
No single uniform, replacement, rarity or independence rule fills these sites.

### 4.1 Starter and common

<!-- primary-matrix-start -->
| Constructor ID | Trigger, condition, state and primary rule | Boundary |
|---|---|---|
| `Burning Blood` | Victory: if current HP>0, heal 6 through heal modifiers/cap. | R,X |
| `Akabeko` | Battle start queues Vigor 8; NORMAL damage gains its amount until Attack onUse queues removal. Not one flat damage packet. | X,P |
| `Anchor` | Battle start queues Block 10 at bottom; room entry clears used display. | R,X |
| `Ancient Tea Set` | Rest-room entry arms counter −2; prebattle resets first-turn flag; first turn consumes arm to −1 and queues energy 2 at top. No Rest purchase required. | R,X,H |
| `Art of War` | Prebattle arms first turn/no-Attack flag; later turn starts grant energy 1 if prior interval had no Attack onUse, then rearm. First turn grants none. | R,X |
| `Bag of Marbles` | Battle start queues Vulnerable 1 on each monster. | X,P |
| `Bag of Preparation` | Battle start queues draw 2. | X,P |
| `Blood Vial` | Battle start queues heal 2 at top. | R,X |
| `Bronze Scales` | Battle start queues Thorns 3. | X,P |
| `Centennial Puzzle` | Prebattle clears once-per-combat latch; first positive combat wasHPLost queues draw 3 and sets used immediately. | X,P |
| `CeramicFish` | Actual onObtainCard calls gainGold(9); not offer generation or a global movement hook. | R,H |
| `Dream Catcher` | Campfire sleep effect, after healing/onRest hooks, opens ordinary reward-helper cards; not every heal/rest-room entry. | H,G |
| `Happy Flower` | Equip counter 0; each turn increments, 3 resets to 0 and queues energy 1; persists between fights. −1 is specially advanced to 1. | R,X |
| `Juzu Bracelet` | EventHelper converts rolled MONSTER to EVENT, still resets monster chance as that branch dictates; does not prohibit event-authored fights. | G,H |
| `Lantern` | Prebattle rearms first-turn latch; first turn queues energy 1 at top and consumes latch. | R,X |
| `MawBank` | Active onEnterRoom gains gold 12; shop onSpendGold permanently sets used counter −2. Not disabled by every nonshop loss. | R,H |
| `MealTicket` | justEnteredRoom on actual ShopRoom heals 15. | R,H |
| `Nunchaku` | Persistent Attack onUse counter: every tenth resets 0 and queues energy 1. | R,X |
| `Oddly Smooth Stone` | Battle start queues Dexterity 1. | X,P |
| `Omamori` | Starts two charges; named curse-obtain constructors consume a charge/prevent there; direct Note path differs. Counter zero marks used. | H,X |
| `Orichalcum` | End turn: Block zero or explicit trigger flag queues Block 6 at top and clears flag; turn start clears flag. Positive Block-gain callback stops pulse and floors its float argument. | R,X |
| `Pen Nib` | Persistent Attack counter; ninth queues Pen Nib power, tenth resets 0. Battle start reinstalls power at counter 9. Power doubles NORMAL damage and Attack onUse queues its removal. | X,P |
| `Potion Belt` | Equip increases capacity by 2 and appends two empty slots. | R,H |
| `PreservedInsect` | Battle start with eliteTrigger lowers each initial monster's current HP to at most truncated 75% maxHP, not maxHP itself; no spawn hook. | X,P |
| `Red Skull` | Battle-start delayed action checks bloodied; combat threshold entry/exit toggles active flag and +3/−3 Strength. Victory clears active state. | X,P |
| `Regal Pillow` | Sleep effect adds 15 to its Rest healing amount; not all healing. | R,H |
| `Smiling Mask` | Shop removal price 50 takes precedence in price paths; room callback itself only presents activation. | H |
| `Strawberry` | Equip increases maxHP 7, including helper healing. | R,H |
| `Boot` | onAttackToChangeDamage: owner-present non-THORNS/non-HP_LOSS positive amount below 5 becomes 5. Not base-damage +5. | X,P |
| `Tiny Chest` | Equip resets 0; each EventHelper room roll increments; fourth resets 0 and overrides rolled result to TREASURE before chance updates. It still makes the room roll. | G,H |
| `Toy Ornithopter` | Potion use heals 5: queued in COMBAT, immediate outside. | R,X,H |
| `Vajra` | Battle start queues Strength 1. | X,P |
| `War Paint` | Equip shuffles canUpgrade Skill instances, upgrades up to first two immediately, refreshes bottle links. Duplicate kinds are allowed. | H,G |
| `Whetstone` | Same equipment prefix rule for canUpgrade Attack instances. | H,G |

### 4.2 Uncommon

| Constructor ID | Trigger, condition, state and primary rule | Boundary |
|---|---|---|
| `Blue Candle` | Shared base legality permits otherwise-unplayable curses; curse onUse queues HP_LOSS 1 and marks both card and cleanup exhaust. All other legality/HP-loss modifiers remain. | X,P |
| `Bottled Flame` | Typed Attack selection/flag at delayed equipment update; initializeDeck top-placement is shared, not its battle-start flash. | H,X,G |
| `Bottled Lightning` | Same for Skill. Spawn's nonbasic check is not the equipment list's filter. | H,X,G |
| `Bottled Tornado` | Same for Power. Unequip clears matching permanent instance flag. | H,X,G |
| `Darkstone Periapt` | Actual obtain with card color CURSE increases maxHP 6, with helper healing; not prevention/offer alone. | R,H |
| `Eternal Feather` | onEnterRoom RestRoom heals 3×integer(deck size/5), even if Rest itself is disabled/not chosen. | R,H |
| `Frozen Egg 2` | Matching Power preview/obtain upgrades only canUpgrade and not already upgraded; equip revisits existing reward cards. | H,P |
| `Gremlin Horn` | onMonsterDeath requires currentHealth==0 and group not basically dead; queues energy 1 then draw 1 at bottom. Not final-group payoff. | R,X,P |
| `HornCleat` | Battle start counter 0; second turn queues Block 14 once, marks spent; victory clears spent display and sets −1. | R,X |
| `InkBottle` | Persistent onUse count; tenth resets 0 and queues draw 1; battle start restores armed display at 9. | X,P |
| `Kunai` | Turn counter 0; each third Attack onUse resets and queues Dexterity 1; victory counter −1. | X,P |
| `Letter Opener` | Turn counter 0; each third Skill onUse queues all-enemy THORNS 5 and resets; victory −1. | X,P |
| `Matryoshka` | Two charges; nonboss chest before base relic decrements charge, adds a common/uncommon relic reward, then marks used if exhausted. Source uses strict .75 Boolean. | H,G |
| `Meat on the Bone` | endBattle explicitly triggers before ordinary player victory hooks; living HP≤maxHP/2 heals 12 using live state. | R,X |
| `Mercury Hourglass` | Every turn start queues all-enemy THORNS 3. | X,P |
| `Molten Egg 2` | Frozen Egg rule for Attack; already-upgraded Searing Blow does not receive another acquisition upgrade. | H,P |
| `Mummified Hand` | Power onUse chooses a hand instance with base cost>0, turn cost>0, not free-once, excluding queued cards; sets selected turn cost 0 immediately; empty set does nothing. | X,G,P |
| `Ornamental Fan` | Turn counter 0; every third Attack onUse queues Block 4 and resets; victory −1. | R,X |
| `Pantograph` | Battle start with any BOSS enemy queues one heal 25, then returns. Not one heal per boss actor. | R,X |
| `Paper Frog` | Shared Vulnerable factor 1.75 for nonplayer owner/NORMAL only; §2. | X,P |
| `Pear` | Equip increases maxHP 10 with helper healing. | R,H |
| `Question Card` | Reward-count callback adds 1 only at callers using it, not every card generator. | H,G |
| `Self Forming Clay` | Each positive combat wasHPLost queues Next Turn Block 3; that power queues its accumulated Block/removal at next start. | R,X,P |
| `Shuriken` | Turn counter 0; every third Attack onUse queues Strength 1 and resets; victory −1. | X,P |
| `Singing Bowl` | Button onClick increases maxHP 2 (helper heals), then removes its captured reward row; update then closes via Bowl/card-screen paths. Relic update itself is only click presentation. Full parent return remains H. | R,H |
| `StrikeDummy` | atDamageModify adds 3 to cards with STRIKE tag, not all Attack cards or all names containing “Strike.” | X,P |
| `Sundial` | Equip 0; every third onShuffle resets 0 and queues energy 2; persists between combats. Actual shuffle notification boundary matters. | R,X,G |
| `The Courier` | Shop price multiplier .8 and restocks bought stock via source-specific paths; room callback only presents activation. | H,G |
| `Toxic Egg 2` | Frozen Egg rule for Skill; preview and actual obtain remain separate calls with upgraded guard. | H,P |
| `White Beast Statue` | Potion chance override precedes four-row suppression and appearance-history mutation; §2. | H,G |

### 4.3 Rare

| Constructor ID | Trigger, condition, state and primary rule | Boundary |
|---|---|---|
| `Bird Faced Urn` | Power onUse queues heal 2 at top. | R,X |
| `Calipers` | Shared next-turn loss 15 absent Barricade/Blur; no relic counter; §2. | R,X |
| `CaptainsWheel` | Battle counter 0; third turn queues Block 18 once, marks spent; victory clears spent state/counter. | R,X |
| `Champion Belt` | ApplyPowerAction, player source/nonself target/Vulnerable/no target Artifact, invokes onTrigger to queue Weak 1. Source notification is before target insertion. | X,P |
| `Charon's Ashes` | Every actual exhaust callback queues all-enemy THORNS 3 at top; no Branch-style basically-dead guard in relic. | X,P |
| `Dead Branch` | Actual exhaust, if group not basically dead: selects combat-random card now, queues hand creation at bottom. Selection and insertion are different boundaries. | X,G,P |
| `Du-Vu Doll` | Equip/master-deck-change recount CURSE-type instances; battle start queues that much Strength if positive. Cached count depends on actual notification path. | H,X,P |
| `FossilizedHelix` | Battle start queues Buffer 1; room entry resets display. Positive-damage prevention belongs Buffer, not a postdeath heal. | X,P |
| `Gambling Chip` | Pre-draw resets activated; first postdraw marks activated and queues any-number discard/redraw action. Selected cards use manual-discard triggers, not end-turn discard. | X,P |
| `Ginger` | Shared player Weak application prevention before Artifact; §2. | X,P |
| `Girya` | Campfire Lift usable while persistent counter<3; battle start queues Strength equal to nonzero counter. Lift effect, not adding option, increases counter. | H,X,P |
| `Ice Cream` | Recharge adds retained energy with source addEnergy cap 999; battle prep clears current energy; §2. | R,X |
| `Incense Burner` | Equip 0; every sixth turn resets 0 and queues Intangible 1; persists across fights; −1 specially advances to 1. | X,P |
| `Lizard Tail` | Player damage rescue checks unused counter −1 after Fairy; trigger synchronously heals max(1,maxHP/2), marks −2. Heal modifiers remain. | R,X |
| `Magic Flower` | onPlayerHeal in a live COMBAT room rounds requested amount×1.5; outside unchanged. Helper then applies powers/cap. | R,X,P |
| `Mango` | Equip maxHP +14 with helper healing. | R,H |
| `Old Coin` | Equip gainGold(300), subject to actual gainGold guards. | R,H |
| `Peace Pipe` | Adds Toke option enabled only for nonbottled purgeable cards; selected removal retains actual source callbacks. | H |
| `Pocketwatch` | Battle resets count/first turn; onPlay counts; later postdraw count≤3 queues draw 3, then clears count. No first-turn bonus. Victory resets −1. | X,P |
| `Prayer Wheel` | CombatRewardScreen adds a second card offer only in ordinary MonsterRoom, excluding elite/boss and EventRoom fights; later offer sees prior history changes. | H,G |
| `Shovel` | Adds Dig campfire option; Dig's source generates the relic offer/obtain, not the option constructor. | H,G |
| `StoneCalendar` | Battle counter 0; turn starts increment; end of exactly turn 7 queues all-enemy THORNS 52; victory resets −1/pulse. | X,P |
| `Thread and Needle` | Battle start queues Plated Armor 4. | X,P |
| `Torii` | onAttacked reduces owner-present non-THORNS/non-HP_LOSS amount 2..5 to 1, before Tungsten's final-loss reduction. | X,P |
| `TungstenRod` | onLoseHpLast subtracts 1 from positive loss, including applicable HP_LOSS paths; not one reduction per turn. | X,P |
| `Turnip` | Shared player Frail application prevention before Artifact; §2. | X,P |
| `Unceasing Top` | Refresh-hand queues draw 1 only with empty pending action list/hand, active turn/combat, no screen/No Draw/temporary disable, and nonempty draw or discard. Prebattle disables, turn start enables/resets disable. | X,P |
| `WingedGreaves` | Starts three charges; permits otherwise-unconnected same-next-row destinations as checked by map edges; selecting a nonnormal winged connection consumes one, last marks −2. Normal edges cost none. | H,V |

### 4.4 Boss

| Constructor ID | Trigger, condition, state and primary rule | Boundary |
|---|---|---|
| `Astrolabe` | Up to three purgeable removals/upgraded transforms via immediate-small-group or delayed-three branch; §3.1. | H,G |
| `Black Blood` | Boss-relic selection uses instant slot-0 replacement rather than ordinary append; living victory heal 12. Own canSpawn needs Burning Blood; starter-removed Neow swap fails it. | R,H,X |
| `Black Star` | Actual MonsterRoomElite adds extra noncampfire relic reward; onVictory/room hooks alone only manage display. EventRoom eliteTrigger is insufficient. | H,G |
| `Busted Crown` | Equip/unequip master energy ±1; reward-count callback subtracts 2 where invoked. | R,H,G |
| `Calling Bell` | Fixed curse confirmation, then once-only three screenless relic offers after screen-down; §3.1. | H,G,X |
| `Coffee Dripper` | Master energy ±1 on equip/unequip; disables exact ordinary RestOption, not RestRoom entry effects or all campfire options. | R,H |
| `Cursed Key` | Master energy ±1; nonboss onChestOpen schedules random curse obtain; bossChest true produces none. Chest skip produces no open hook. | R,H,G |
| `Ectoplasm` | Master energy ±1; shared gainGold early return prevents actual gain and gain callbacks. Offers/refunds are still source-defined. | R,H |
| `Empty Cage` | Up to two object removals with distinct empty/automatic/grid paths; §3.1. | H,X |
| `Fusion Hammer` | Master energy ±1; disables exact ordinary SmithOption, not every upgrade producer. | R,H |
| `Mark of Pain` | Master energy ±1; battle start queues two Wounds in draw with source shuffle/insertion semantics, not hand or permanent deck. | R,X,G,P |
| `Pandora's Box` | Iterator-removes starter-tagged Strikes/Defends, generates/preview-hooks one current-character replacement each, then confirmation obtains; §3.1. | H,G,X |
| `Philosopher's Stone` | Master energy ±1; battle-start monsters and each spawn notification immediately add Strength 1 and modify-power notification. Split children qualify; no Insect spawn effect inferred. | R,X,P |
| `Runic Cube` | Every positive combat wasHPLost queues draw 1 at top. | X,P |
| `Runic Dome` | Master energy ±1; shared monster presentation suppresses intent display/tooltips, not intent selection/actions. | R,V |
| `Runic Pyramid` | Suppresses bulk discard/Retain Cards selection, not Ethereal or all end-turn effects; §2. | X,P |
| `SacredBark` | Equip reinitializes held potion data; AbstractPotion potency doubles through its Bark branch, while typed choice/duplication potions retain distinct explicit handling. Not every potion payload is “double all fields.” | H,P |
| `SlaversCollar` | Explicit beforeEnergyPrep tests eliteTrigger/BOSS enemy, temporarily master energy +1; activated victory removes it. No ordinary onEquip energy grant. | R,X |
| `Snecko Eye` | Equip/unequip master hand size ±2; prebattle queues Confusion. Randomization is the power's draw behavior, not a universal initial card-cost rewrite. | X,G,P |
| `Sozu` | Master energy ±1; potion refusal is caller-specific: reward claim can consume without obtain, store refuses before payment, low-level obtainPotion has no universal Sozu gate. | R,H,P |
| `Tiny House` | Equip random upgrade prefix of one, maxHP +5, gold/potion rewards and ordinary-context card reward screen. Immediate upgrade/maxHP precede claims; not boss-combat forced rare. | R,H,G |
| `Velvet Choker` | Master energy ±1; battle/turn resets 0; onPlay increments up to 6; canPlay rejects at ≥6; victory −1. Applies through actual legality dispatch, not hand plays alone. | R,X,P |

### 4.5 Shop and special

| Constructor ID | Trigger, condition, state and primary rule | Boundary |
|---|---|---|
| `Brimstone` | Every turn queues player Strength 2 and each monster Strength 1 at top; top insertion reverses encounter order. | X,P |
| `Cauldron` | Equip creates five flat-player-pool potion offers, opens rewards, removes automatically generated card row; discarded generation can still change pity. | H,G,P |
| `Chemical X` | Action-local effect count +2; ten current consumers and explicit gates/payment in §2.1. | X,P,G |
| `ClockworkSouvenir` | Battle start queues Artifact 1. | X,P |
| `DollysMirror` | Selected permanent-card stat-equivalent copy, bottle flags cleared, scheduled obtain; §3.1. | H,X,P |
| `Frozen Eye` | Shared draw/exhaust view sorting bypass and legitimate draw-order visibility; §2. | V |
| `HandDrill` | Actual monster damage Block-break notification queues Vulnerable 2, including exact depletion; not generic loseBlock. | X,P |
| `Lee's Waffle` | Equip maxHP +7 (helper also heals), then heal request equal to new maxHP. | R,H |
| `Medical Kit` | Base legality allows otherwise-unplayable Status; Status onUse marks card/cleanup exhaust, without converting Status type to Skill. | X,P |
| `Membership Card` | Shop .5 discount via shop/store paths; purchase immediately affects remaining stock. Room hook itself only presents activation. | H |
| `OrangePellets` | Turn resets Attack/Skill/Power flags; onUse records type; all three queues RemoveDebuffs and immediately resets flags, allowing another triple that turn. | X,P |
| `Orrery` | Four explicit card reward offers plus the screen's ordinary auto-added offer in shop; five offers share changing reward history, not one five-card choice. | H,G |
| `PrismaticShard` | Equip grants one master orb slot only if non-Defect and current master slots zero; shared reward helper widens eligible reward colors, not merchant/generator/transform character authority. | H,G,P |
| `Sling` | Battle start with eliteTrigger queues Strength 2; actual elite room type is not required. | X,P |
| `Strange Spoon` | Conditional single Boolean at exhaust-marked cleanup; purge/Power excluded and actual destination precedence retained; §2. | X,G |
| `TheAbacus` | Every onShuffle queues Block 6 at bottom; source shuffle callbacks determine trigger, not any arbitrary pile movement. | R,X,G |
| `Toolbox` | Pre-draw battle hook queues ChooseOneColorless action; choice/generation/cancellation remain real work before ready state. | X,G,P |
| `NeowsBlessing` | Starts three; each qualifying battle-start callback spends one and sets all initial monsters' current HP to 1, without changing maxHP. Last marks used −2. | X,P |
| `Golden Idol` | Stored non-theft/non-TreasureRoom gold reward bonus, not universal gainGold modifier; §2. | R,H |
| `Odd Mushroom` | Shared player Vulnerable NORMAL factor 1.25; §2. | X,P |
| `WarpedTongs` | Each postdraw queues random hand upgrade: only canUpgrade non-Status candidates, shuffled then first upgraded; empty does nothing. Temporary combat instance, not permanent upgrade. | X,G,P |
| `Spirit Poop` | Full-game score bonus −1 and flag; not Act-1 reward/debuff; §2. | R,V |
| `FaceOfCleric` | Victory increases maxHP 1 with helper healing. | R,X |
| `CultistMask` | Battle-start speech/sound actions only in inspected primary payload; no numeric combat buff. Generic relic ownership still exists. | X |
| `GremlinMask` | Battle start queues player Weak 1 through ApplyPowerAction; Ginger can prevent before Artifact. | X,P |
| `NlothsMask` | One charge; nonboss onChestOpenAfter removes first relic reward and its linked item where applicable, then marks −2. Matryoshka/base order matters. | H |
| `SsserpentHead` | Face Trader face; onEnterRoom actual EventRoom gains gold 50 via gainGold. Not The Ssssserpent's reward. | R,H |
<!-- primary-matrix-end -->

## 5. Retained acquisition laws and lane interfaces

Do not regress the reviewed selector corrections while integrating these mechanics:

* Normal nonempty tier selection takes front. A rejected `canSpawn` candidate is
  consumed, then dispatches to the **end helper at the originally requested tier**,
  including rejection after a promotion. End rejection retries through itself.
* End selection uses back for nonempty common/uncommon/rare/shop, **front for boss**.
  Empty common/uncommon/shop promotion calls **normal/front** uncommon/rare/uncommon,
  respectively; it is not a permanent switch to back. Empty rare/boss fallback is
  Circlet/Red Circlet, outside the 141 candidate roster.
* Face Trader supplies Ssserpent Head among five missing-face candidates; The
  Ssssserpent offers Doubt and gold. No event-source alias may merge them.
* Campfire order remains initial relic usability filtering, Recall append, then
  final usability scan. Dripper/Hammer do not remove Recall, Feather or Tea Set.

The execution interface must retain the card's origin, captured energy, live
energy, free/autoplay/ignore-total/purge/exhaust/destination flags, actual callback
family and phase, mutable action arguments, and distinct entered/current/pending
work. Counters consume **their callback**, not an invented “successful payload”
event. Five repeat powers explicitly retain their true autoplay/ignore-total flags;
this report neither weakens them nor declares universal legality bypass. Reaper's
timer-dependent heal/clear outcome remains unproved.

The continuation interface must retain source/parent identity, saved screen and
resume point, selected permanent instance identity, pending-equipment latch,
already-committed resource mutations and outstanding obtain effects separately.
Preview, prevention, actual obtain, object/iterator removal, deck-change and later
profile write are not a universal commit callback. Accepted Note retrieval-before-
outgoing-selection and direct-hook distinctions stand. This lane supplies local
equipment boundaries, not a new generic FIFO/drain/return-to-map rule.

Native projection must carry public relic ownership/charge states, relevant local
counters, pending choices and legitimate visibility without leaking hidden future
order. Frozen Eye and Runic Dome are source visibility rules; choosing packed row
offsets, knowledge retention and versioning belongs to the native contract. Original
full-run score and the Act-1 task's +1/−1/0 reward remain distinct quantities.

## 6. Discriminating future acceptance cases

These are **specified cases, not executed gameplay tests**. Fixtures must establish
legal origins or explicitly label injected-state conditional checks.

| Case | Distinction required |
|---|---|
| A01 — player/enemy Vulnerable with both relics | NORMAL factors are player 1.25/enemy 1.75; THORNS/HP_LOSS do not gain either multiplier. No Vulnerable means neither factor. |
| A02 — Statue with 3 versus 4 existing reward rows | Three permits forced appearance/offset−10; four suppresses/offset+10 while still consuming appearance roll. Full belt changes claim, not generation. |
| A03 — Calipers at Block 8/20, with neither/Barricade/Blur | Neither yields 0/5; either power keeps 8/20. Queued next-turn Block is not deducted before it resolves. |
| A04 — Ginger/Turnip plus Artifact | Correct player debuff is prevented without spending Artifact; source callback still occurs. Other debuffs and monster targets retain their actual gates. |
| A05 — Ice Cream at 2 plus recharge 3, with Conserve | Current becomes 5 and Conserve is not reduced through the skipped branch. 998+3 clamps 999. Next combat prep resets to zero before recharge. |
| A06 — Pyramid ordinary, explicitly retained and Ethereal cards | Ordinary held card gets no synthetic onRetained; explicit retain follows restore path; Ethereal still queues exhaust. Well-Laid Plans selection is suppressed. |
| A07 — Chemical X captured zero, live three, free/nonfree | Positive +2 effect comes from relic, not spendable energy; nonfree action spends live three while free leaves it. Include no-orb Multicast and zero-count Conjure Blade exceptions. |
| A08 — Frozen Eye removed/acquired around a view | Same pile membership; sorted hidden-order view without ownership versus upcoming order with it. Exhaust-view ordering also follows its actual branch. No hidden order in candidate IDs. |
| A09 — Spoon success with rebound/return/exhaust, purge copy and Ethereal | One draw only in qualifying cleanup; success follows ordinary destination priority. Purge/Power/Ethereal independent path does not consume this Spoon draw or trigger faux exhaust callbacks. |
| A10 — Idol gold reward 6, direct gold 6, stolen reward 6, chest reward 6 | Non-TreasureRoom non-theft reward stores bonus 2; other named routes do not. Increment recomputes from total; Ectoplasm blocks actual claim gain, not offer text. |
| A11 — Spirit Poop versus task reward | Score bonus differs by one, not terminal HP or Act1Complete reward. Generic relic-count threshold can still see ownership. |
| A12 — Collar in normal/event-elite/boss fight | No equip-wide +1; correct pre-prep temporary increment/removal. Event eliteTrigger can activate Collar/Sling/Insect but does not grant Black Star's elite-room reward. |
| A13 — Bottle eligibility versus equipment selection | Nonbasic requirement for offer eligibility; basic typed card can be selected once eligible. Flagged instance participates in opening top-placement; selected-copy bottle flags clear for Mirror. |
| A14 — Tiny Chest fourth roll with Juzu | Roll still consumed, Tiny Chest forces treasure before chance-history processing; Juzu is not a ban on Mushrooms/Dead Adventurer combat. |
| A15 — per-combat/per-turn/persistent counters | Flower/Incense/Nunchaku/Ink/Nib/Sundial persist; Kunai/Shuriken/Fan/Opener reset each turn; wheel/cleat/calendar battle counters restart. No global reset-all. |
| A16 — positive HP loss versus prevented zero | Puzzle once, Clay/Cube per positive combat notification; Torii/Tungsten/Buffer prevention can remove qualifying positive loss. Distinct later hits remain distinct. |
| A17 — Mirror/Cage/Astrolabe/Bell/Pandora interruption | Assert each local mutation once and real remaining child/parent; no duplicate grant on reopen, rollback on death, or room-COMPLETE shortcut. Explicitly exercise empty conditional branches without inventing reachability. |
| A18 — last enemy exhaust/death | Branch guard versus Ashes enqueue, Horn's currentHP/group gate, and later manager cancellation are distinguishable. Selecting a Branch card is not inserting it. |
| A19 — exact Block depletion and ordinary loss | Hand Drill on positive monster Block exhausted by damage, even equality; not Calipers/ordinary loseBlock or HP_LOSS bypass. |
| A20 — selection history/rejection/depletion | Normal rejection retries end at requested tier; end boss remains front; depleted C/U/shop promotion returns normal/front. Skipped offers remain consumed. |

## 7. Validation and precise closure limits

Private failure-first standard-library checks validate exact constant-pool matches,
malformed constant-pool/header refusal, unique class-entry selection, unsafe/missing/duplicate
entry rejection, environment stripping without input mutation, all-141 public-row
coverage, duplicate/missing/unknown-boundary controls and local-link validity.
Fresh manifests retain selected entry hashes, static commands and exits. A failed
wrong-package Vigor selection is preserved; corrected selection used
`powers.watcher.VigorPower`. No failed attempt was recast as gameplay behavior.

The new local closure is §2's thirteen concrete consumer rules; the explicit
Collar/preparation, energy cap, bottle, exhaust, Block-break, helper-heal and delayed
equipment distinctions; and the per-ID primary facts rather than merely override
names. Source and selected bytecode support these bounded facts. Helper tests
validate evidence/coverage tooling, not original mechanics. No model agreement is
offered as independent game validation.

Still required: the X/H/G/P/V obligations named per row; complete reachable
power/card/potion payload and shared callback composition; precise observable
shuffle laws; acquisition parent returns/exceptional inventory reachability;
original timer-dependent cancellation outcomes; and integrated native information
and terminal contracts. In particular, assigning a primary fact to every row is
**not** closing all 141 transitive mechanics. No gameplay tests ran, and none of
the future cases is marked passed. Main owns integration/acceptance; whole-spec
human approval, plan review and implementation-start authorization remain separate.
