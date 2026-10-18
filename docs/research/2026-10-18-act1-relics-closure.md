# Act 1 relic inventory and source-hook reconciliation

Date: 2026-10-18. Supporting static research for the complete Ironclad A0 Act 1 goal; **not whole-spec closure, implementation approval, or original-runtime validation**.

## Evidence boundary and result

The audit is bound to the supplied JAR SHA256 `cfad868ac8d65a88e71a0bf096fb09f78811e553effe0787c5309a655e081673`. It reconstructs each of the 141 candidate relic classes, plus `RelicLibrary`, `AbstractDungeon`, and `AbstractRelic`, with pinned CFR 0.152 and Temurin 21.0.12.1+1. Exact archive entries were unique and hashed. This is static class-source analysis; no game class was initialized or executed, and there was no launcher, real profile, asset, native, or network probe. The source provenance does not establish an upstream build/channel or compatibility beyond these bytes.

**The 141-candidate inventory and tier totals reconcile; no count correction is indicated.** Constructor IDs and tier arguments were checked against each candidate class, and the registration calls establish whether it enters the shared, Ironclad-red, or special set. The verified tier totals are 1 starter, 33 common, 30 uncommon, 28 rare, 22 boss, 17 shop, and 10 special. The detailed table below records the exact constructor ID, class, tier, acquisition/pool source and direct primary override entrypoints for every row. Hook names are inventory locators, not proof of transitive caller closure or callback execution.

Some candidate/display labels are not the internal constructor ID: Frozen Egg uses `Frozen Egg 2`; Molten Egg uses `Molten Egg 2`; Toxic Egg uses `Toxic Egg 2`; Paper Phrog uses `Paper Frog`; Neow's Lament uses `NeowsBlessing`; Wing Boots is class `WingBoots` with ID `WingedGreaves`; and The Abacus is class `Abacus` with ID `TheAbacus`. These are identity mappings, not extra relics.

## Character and route reconciliation

`RelicLibrary` registers 120 shared non-special candidates, 11 Ironclad-red candidates, and 10 shared `SPECIAL` candidates. The Ironclad-red set is Burning Blood, Red Skull, Paper Frog, Self Forming Clay, Black Blood, Champion Belt, Charon's Ashes, Magic Flower, Mark of Pain, Runic Cube, and Brimstone. The remaining 120 ordinary-tier candidates are shared. No other-character-exclusive relic appears in this 141-row Act 1 set. The initialized Ironclad pool is the shared plus red registration, subject to unlock filtering; Shard changes card reward eligibility, and Note can retrieve cards, but neither grants another character's starting/exclusive relics.

The 10 special-tier rows are source-bound, not ordinary random relic tiers: Neow's Lament comes from Neow; Golden Idol from the Golden Idol event; Odd Mushroom from Hypnotizing Colored Mushrooms; Warped Tongs from Ominous Forge; Spirit Poop from Bonfire Spirits; Face of Cleric, Cultist Headpiece, Gremlin Visage, and N'loth's Hungry Face from Face Trader; Ssserpent Head from The Ssssserpent. Their legal event/choice routes remain content obligations even if a run skips them.

Ordinary pool membership is not equivalent to eligibility at every source. The finite direct override audit found exactly 32 candidate `canSpawn` overrides (not 38; the earlier 38 constant-pool hits were discovery leads, not verified override definitions):

- Floor≤48 (or Endless): Ancient Tea Set, Ceramic Fish, Dream Catcher, Juzu Bracelet, Meal Ticket, Omamori, Potion Belt, Regal Pillow, Prayer Wheel, Darkstone Periapt, Frozen Egg 2, Meat on the Bone, Molten Egg 2, Question Card, Singing Bowl, Toxic Egg 2.
- Preserved Insect: floor≤52 (or Endless); Matryoshka: floor≤40 (or Endless); Tiny Chest: floor≤35 (or Endless); WingedGreaves: floor≤40 (or Endless).
- Shop-room exclusion plus floor≤48 (or Endless): Maw Bank, Smiling Mask, The Courier, Old Coin. These remain reachable from other legal sources.
- Deck predicates: Bottled Flame requires a non-basic Attack; Bottled Lightning a non-basic Skill; Bottled Tornado a Power.
- Girya, Peace Pipe, and Shovel require fewer than two owned among those three and floor<48 unless Endless.
- Black Blood requires Burning Blood; Ectoplasm requires Act≤1.

All other candidate classes inherit base `canSpawn`; that is not evidence that every acquisition helper uses every tier. Source families differ: normal/elite/chest relic rewards, boss relic rewards, Neow boss swap, merchant stock, event relics and screenless selectors have separate pool and continuation rules. In particular, the screenless selector rejects Bottled Flame/Lightning/Tornado/Whetstone; the non-campfire selector rejects Peace Pipe/Shovel/Girya. Offers consume pool entries even if skipped.

Pool selection has distinct rejection and depletion branches; do not summarize them as one retry direction. The normal/front helper consumes the front for nonempty common, uncommon, rare, shop, and boss tiers. It consumes a rejected candidate before retrying through the end helper. The end helper consumes the back for nonempty common, uncommon, rare, and shop tiers, but consumes the front for boss; it also consumes a rejected candidate before retrying through the end helper. The end helper's depleted common, uncommon, and shop branches promote by calling the normal/front helper for uncommon, rare, and uncommon respectively. The normal helper's depleted common, uncommon, and shop branches recurse through the normal/front helper for uncommon, rare, and uncommon respectively. Empty rare falls back to `Circlet`; empty boss falls back to `Red Circlet` in either helper. These are depletion branches, not a claim that ordinary relic replacement is inexhaustible. Circlet and Red Circlet are not part of the 141 authored candidate roster or ordinary pool.

Neow's boss swap uses the boss tier and source predicates after the active starter has been removed. This makes Black Blood (requires Burning Blood) ineligible for the swap, without making it categorically ineligible at a later boss reward when Burning Blood is still owned. Ectoplasm's Act≤1 check permits this Act. Pool membership, a relic's own predicate, and a particular source's selector are separate facts.

## Per-relic primary-entrypoint matrix

The methods listed are direct subclass overrides other than presentation/copy plumbing. The 13 rows with no such override are explicitly marked; their effect may reside in shared engine checks or an event branch. Neither a missing override nor a `canSpawn` pass is a completed mechanic.

| Candidate label | Constructor ID | Class | Tier | Pool / source eligibility | Direct primary override entrypoints |
|---|---|---|---|---|---|
| Burning Blood | `Burning Blood` | `BurningBlood` | `STARTER` | Ironclad-red; Ironclad starting relic; removed when an active starter is not kept | `onVictory` |
| Akabeko | `Akabeko` | `Akabeko` | `COMMON` | shared; Shared or Ironclad-red common relic pool | `atBattleStart` |
| Anchor | `Anchor` | `Anchor` | `COMMON` | shared; Shared or Ironclad-red common relic pool | `atBattleStart`, `justEnteredRoom` |
| Ancient Tea Set | `Ancient Tea Set` | `AncientTeaSet` | `COMMON` | shared; Shared or Ironclad-red common relic pool | `atPreBattle`, `atTurnStart`, `canSpawn`, `onEnterRestRoom`, `setCounter` |
| Art of War | `Art of War` | `ArtOfWar` | `COMMON` | shared; Shared or Ironclad-red common relic pool | `atPreBattle`, `atTurnStart`, `onUseCard`, `onVictory` |
| Bag of Marbles | `Bag of Marbles` | `BagOfMarbles` | `COMMON` | shared; Shared or Ironclad-red common relic pool | `atBattleStart` |
| Bag of Preparation | `Bag of Preparation` | `BagOfPreparation` | `COMMON` | shared; Shared or Ironclad-red common relic pool | `atBattleStart` |
| Blood Vial | `Blood Vial` | `BloodVial` | `COMMON` | shared; Shared or Ironclad-red common relic pool | `atBattleStart` |
| Bronze Scales | `Bronze Scales` | `BronzeScales` | `COMMON` | shared; Shared or Ironclad-red common relic pool | `atBattleStart` |
| Centennial Puzzle | `Centennial Puzzle` | `CentennialPuzzle` | `COMMON` | shared; Shared or Ironclad-red common relic pool | `atPreBattle`, `justEnteredRoom`, `onVictory`, `wasHPLost` |
| Ceramic Fish | `CeramicFish` | `CeramicFish` | `COMMON` | shared; Shared or Ironclad-red common relic pool | `canSpawn`, `onObtainCard` |
| Dream Catcher | `Dream Catcher` | `DreamCatcher` | `COMMON` | shared; Shared or Ironclad-red common relic pool | `canSpawn` |
| Happy Flower | `Happy Flower` | `HappyFlower` | `COMMON` | shared; Shared or Ironclad-red common relic pool | `atTurnStart`, `onEquip` |
| Juzu Bracelet | `Juzu Bracelet` | `JuzuBracelet` | `COMMON` | shared; Shared or Ironclad-red common relic pool | `canSpawn` |
| Lantern | `Lantern` | `Lantern` | `COMMON` | shared; Shared or Ironclad-red common relic pool | `atPreBattle`, `atTurnStart` |
| Maw Bank | `MawBank` | `MawBank` | `COMMON` | shared; Shared or Ironclad-red common relic pool | `canSpawn`, `onEnterRoom`, `onSpendGold`, `setCounter` |
| Meal Ticket | `MealTicket` | `MealTicket` | `COMMON` | shared; Shared or Ironclad-red common relic pool | `canSpawn`, `justEnteredRoom` |
| Nunchaku | `Nunchaku` | `Nunchaku` | `COMMON` | shared; Shared or Ironclad-red common relic pool | `onUseCard` |
| Oddly Smooth Stone | `Oddly Smooth Stone` | `OddlySmoothStone` | `COMMON` | shared; Shared or Ironclad-red common relic pool | `atBattleStart` |
| Omamori | `Omamori` | `Omamori` | `COMMON` | shared; Shared or Ironclad-red common relic pool | `canSpawn`, `setCounter` |
| Orichalcum | `Orichalcum` | `Orichalcum` | `COMMON` | shared; Shared or Ironclad-red common relic pool | `atTurnStart`, `onPlayerEndTurn`, `onPlayerGainedBlock`, `onVictory` |
| Pen Nib | `Pen Nib` | `PenNib` | `COMMON` | shared; Shared or Ironclad-red common relic pool | `atBattleStart`, `onUseCard` |
| Potion Belt | `Potion Belt` | `PotionBelt` | `COMMON` | shared; Shared or Ironclad-red common relic pool | `canSpawn`, `onEquip` |
| Preserved Insect | `PreservedInsect` | `PreservedInsect` | `COMMON` | shared; Shared or Ironclad-red common relic pool | `atBattleStart`, `canSpawn` |
| Red Skull | `Red Skull` | `RedSkull` | `COMMON` | Ironclad-red; Shared or Ironclad-red common relic pool | `atBattleStart`, `onBloodied`, `onNotBloodied`, `onVictory` |
| Regal Pillow | `Regal Pillow` | `RegalPillow` | `COMMON` | shared; Shared or Ironclad-red common relic pool | `canSpawn` |
| Smiling Mask | `Smiling Mask` | `SmilingMask` | `COMMON` | shared; Shared or Ironclad-red common relic pool | `canSpawn`, `onEnterRoom` |
| Strawberry | `Strawberry` | `Strawberry` | `COMMON` | shared; Shared or Ironclad-red common relic pool | `onEquip` |
| The Boot | `Boot` | `Boot` | `COMMON` | shared; Shared or Ironclad-red common relic pool | `onAttackToChangeDamage` |
| Tiny Chest | `Tiny Chest` | `TinyChest` | `COMMON` | shared; Shared or Ironclad-red common relic pool | `canSpawn`, `onEquip` |
| Toy Ornithopter | `Toy Ornithopter` | `ToyOrnithopter` | `COMMON` | shared; Shared or Ironclad-red common relic pool | `onUsePotion` |
| Vajra | `Vajra` | `Vajra` | `COMMON` | shared; Shared or Ironclad-red common relic pool | `atBattleStart` |
| War Paint | `War Paint` | `WarPaint` | `COMMON` | shared; Shared or Ironclad-red common relic pool | `onEquip` |
| Whetstone | `Whetstone` | `Whetstone` | `COMMON` | shared; Shared or Ironclad-red common relic pool | `onEquip` |
| Blue Candle | `Blue Candle` | `BlueCandle` | `UNCOMMON` | shared; Shared or Ironclad-red uncommon relic pool | `onUseCard` |
| Bottled Flame | `Bottled Flame` | `BottledFlame` | `UNCOMMON` | shared; Shared or Ironclad-red uncommon relic pool | `atBattleStart`, `canSpawn`, `onEquip`, `onUnequip`, `update` |
| Bottled Lightning | `Bottled Lightning` | `BottledLightning` | `UNCOMMON` | shared; Shared or Ironclad-red uncommon relic pool | `atBattleStart`, `canSpawn`, `onEquip`, `onUnequip`, `update` |
| Bottled Tornado | `Bottled Tornado` | `BottledTornado` | `UNCOMMON` | shared; Shared or Ironclad-red uncommon relic pool | `atBattleStart`, `canSpawn`, `onEquip`, `onUnequip`, `update` |
| Darkstone Periapt | `Darkstone Periapt` | `DarkstonePeriapt` | `UNCOMMON` | shared; Shared or Ironclad-red uncommon relic pool | `canSpawn`, `onObtainCard` |
| Eternal Feather | `Eternal Feather` | `EternalFeather` | `UNCOMMON` | shared; Shared or Ironclad-red uncommon relic pool | `onEnterRoom` |
| Frozen Egg | `Frozen Egg 2` | `FrozenEgg2` | `UNCOMMON` | shared; Shared or Ironclad-red uncommon relic pool | `canSpawn`, `onEquip`, `onObtainCard`, `onPreviewObtainCard` |
| Gremlin Horn | `Gremlin Horn` | `GremlinHorn` | `UNCOMMON` | shared; Shared or Ironclad-red uncommon relic pool | `onMonsterDeath` |
| Horn Cleat | `HornCleat` | `HornCleat` | `UNCOMMON` | shared; Shared or Ironclad-red uncommon relic pool | `atBattleStart`, `atTurnStart`, `onVictory` |
| Ink Bottle | `InkBottle` | `InkBottle` | `UNCOMMON` | shared; Shared or Ironclad-red uncommon relic pool | `atBattleStart`, `onUseCard` |
| Kunai | `Kunai` | `Kunai` | `UNCOMMON` | shared; Shared or Ironclad-red uncommon relic pool | `atTurnStart`, `onUseCard`, `onVictory` |
| Letter Opener | `Letter Opener` | `LetterOpener` | `UNCOMMON` | shared; Shared or Ironclad-red uncommon relic pool | `atTurnStart`, `onUseCard`, `onVictory` |
| Matryoshka | `Matryoshka` | `Matryoshka` | `UNCOMMON` | shared; Shared or Ironclad-red uncommon relic pool | `canSpawn`, `onChestOpen`, `setCounter` |
| Meat on the Bone | `Meat on the Bone` | `MeatOnTheBone` | `UNCOMMON` | shared; Shared or Ironclad-red uncommon relic pool | `canSpawn`, `onBloodied`, `onNotBloodied`, `onTrigger` |
| Mercury Hourglass | `Mercury Hourglass` | `MercuryHourglass` | `UNCOMMON` | shared; Shared or Ironclad-red uncommon relic pool | `atTurnStart` |
| Molten Egg | `Molten Egg 2` | `MoltenEgg2` | `UNCOMMON` | shared; Shared or Ironclad-red uncommon relic pool | `canSpawn`, `onEquip`, `onObtainCard`, `onPreviewObtainCard` |
| Mummified Hand | `Mummified Hand` | `MummifiedHand` | `UNCOMMON` | shared; Shared or Ironclad-red uncommon relic pool | `onUseCard` |
| Ornamental Fan | `Ornamental Fan` | `OrnamentalFan` | `UNCOMMON` | shared; Shared or Ironclad-red uncommon relic pool | `atTurnStart`, `onUseCard`, `onVictory` |
| Pantograph | `Pantograph` | `Pantograph` | `UNCOMMON` | shared; Shared or Ironclad-red uncommon relic pool | `atBattleStart` |
| Paper Phrog | `Paper Frog` | `PaperFrog` | `UNCOMMON` | Ironclad-red; Shared or Ironclad-red uncommon relic pool | No relic-class primary override; global/site residual |
| Pear | `Pear` | `Pear` | `UNCOMMON` | shared; Shared or Ironclad-red uncommon relic pool | `onEquip` |
| Question Card | `Question Card` | `QuestionCard` | `UNCOMMON` | shared; Shared or Ironclad-red uncommon relic pool | `canSpawn`, `changeNumberOfCardsInReward` |
| Self-Forming Clay | `Self Forming Clay` | `SelfFormingClay` | `UNCOMMON` | Ironclad-red; Shared or Ironclad-red uncommon relic pool | `wasHPLost` |
| Shuriken | `Shuriken` | `Shuriken` | `UNCOMMON` | shared; Shared or Ironclad-red uncommon relic pool | `atTurnStart`, `onUseCard`, `onVictory` |
| Singing Bowl | `Singing Bowl` | `SingingBowl` | `UNCOMMON` | shared; Shared or Ironclad-red uncommon relic pool | `canSpawn`, `update` |
| Strike Dummy | `StrikeDummy` | `StrikeDummy` | `UNCOMMON` | shared; Shared or Ironclad-red uncommon relic pool | `atDamageModify` |
| Sundial | `Sundial` | `Sundial` | `UNCOMMON` | shared; Shared or Ironclad-red uncommon relic pool | `onEquip`, `onShuffle` |
| The Courier | `The Courier` | `Courier` | `UNCOMMON` | shared; Shared or Ironclad-red uncommon relic pool | `canSpawn`, `onEnterRoom` |
| Toxic Egg | `Toxic Egg 2` | `ToxicEgg2` | `UNCOMMON` | shared; Shared or Ironclad-red uncommon relic pool | `canSpawn`, `onEquip`, `onObtainCard`, `onPreviewObtainCard` |
| White Beast Statue | `White Beast Statue` | `WhiteBeast` | `UNCOMMON` | shared; Shared or Ironclad-red uncommon relic pool | No relic-class primary override; global/site residual |
| Bird-Faced Urn | `Bird Faced Urn` | `BirdFacedUrn` | `RARE` | shared; Shared or Ironclad-red rare relic pool | `onUseCard` |
| Calipers | `Calipers` | `Calipers` | `RARE` | shared; Shared or Ironclad-red rare relic pool | No relic-class primary override; global/site residual |
| Captain's Wheel | `CaptainsWheel` | `CaptainsWheel` | `RARE` | shared; Shared or Ironclad-red rare relic pool | `atBattleStart`, `atTurnStart`, `onVictory` |
| Champion Belt | `Champion Belt` | `ChampionsBelt` | `RARE` | Ironclad-red; Shared or Ironclad-red rare relic pool | `onTrigger` |
| Charon's Ashes | `Charon's Ashes` | `CharonsAshes` | `RARE` | Ironclad-red; Shared or Ironclad-red rare relic pool | `onExhaust` |
| Dead Branch | `Dead Branch` | `DeadBranch` | `RARE` | shared; Shared or Ironclad-red rare relic pool | `onExhaust` |
| Du-Vu Doll | `Du-Vu Doll` | `DuVuDoll` | `RARE` | shared; Shared or Ironclad-red rare relic pool | `atBattleStart`, `onEquip`, `onMasterDeckChange`, `setCounter` |
| Fossilized Helix | `FossilizedHelix` | `FossilizedHelix` | `RARE` | shared; Shared or Ironclad-red rare relic pool | `atBattleStart`, `justEnteredRoom` |
| Gambling Chip | `Gambling Chip` | `GamblingChip` | `RARE` | shared; Shared or Ironclad-red rare relic pool | `atBattleStartPreDraw`, `atTurnStartPostDraw` |
| Ginger | `Ginger` | `Ginger` | `RARE` | shared; Shared or Ironclad-red rare relic pool | No relic-class primary override; global/site residual |
| Girya | `Girya` | `Girya` | `RARE` | shared; Shared or Ironclad-red rare relic pool | `addCampfireOption`, `atBattleStart`, `canSpawn` |
| Ice Cream | `Ice Cream` | `IceCream` | `RARE` | shared; Shared or Ironclad-red rare relic pool | No relic-class primary override; global/site residual |
| Incense Burner | `Incense Burner` | `IncenseBurner` | `RARE` | shared; Shared or Ironclad-red rare relic pool | `atTurnStart`, `onEquip` |
| Lizard Tail | `Lizard Tail` | `LizardTail` | `RARE` | shared; Shared or Ironclad-red rare relic pool | `onTrigger`, `setCounter` |
| Magic Flower | `Magic Flower` | `MagicFlower` | `RARE` | Ironclad-red; Shared or Ironclad-red rare relic pool | `onPlayerHeal` |
| Mango | `Mango` | `Mango` | `RARE` | shared; Shared or Ironclad-red rare relic pool | `onEquip` |
| Old Coin | `Old Coin` | `OldCoin` | `RARE` | shared; Shared or Ironclad-red rare relic pool | `canSpawn`, `onEquip` |
| Peace Pipe | `Peace Pipe` | `PeacePipe` | `RARE` | shared; Shared or Ironclad-red rare relic pool | `addCampfireOption`, `canSpawn` |
| Pocketwatch | `Pocketwatch` | `Pocketwatch` | `RARE` | shared; Shared or Ironclad-red rare relic pool | `atBattleStart`, `atTurnStartPostDraw`, `onPlayCard`, `onVictory` |
| Prayer Wheel | `Prayer Wheel` | `PrayerWheel` | `RARE` | shared; Shared or Ironclad-red rare relic pool | `canSpawn` |
| Shovel | `Shovel` | `Shovel` | `RARE` | shared; Shared or Ironclad-red rare relic pool | `addCampfireOption`, `canSpawn` |
| Stone Calendar | `StoneCalendar` | `StoneCalendar` | `RARE` | shared; Shared or Ironclad-red rare relic pool | `atBattleStart`, `atTurnStart`, `justEnteredRoom`, `onPlayerEndTurn`, `onVictory` |
| Thread and Needle | `Thread and Needle` | `ThreadAndNeedle` | `RARE` | shared; Shared or Ironclad-red rare relic pool | `atBattleStart` |
| Torii | `Torii` | `Torii` | `RARE` | shared; Shared or Ironclad-red rare relic pool | `onAttacked` |
| Tungsten Rod | `TungstenRod` | `TungstenRod` | `RARE` | shared; Shared or Ironclad-red rare relic pool | `onLoseHpLast` |
| Turnip | `Turnip` | `Turnip` | `RARE` | shared; Shared or Ironclad-red rare relic pool | No relic-class primary override; global/site residual |
| Unceasing Top | `Unceasing Top` | `UnceasingTop` | `RARE` | shared; Shared or Ironclad-red rare relic pool | `atPreBattle`, `atTurnStart`, `onRefreshHand` |
| Wing Boots | `WingedGreaves` | `WingBoots` | `RARE` | shared; Shared or Ironclad-red rare relic pool | `canSpawn`, `setCounter` |
| Astrolabe | `Astrolabe` | `Astrolabe` | `BOSS` | shared; Shared or Ironclad-red boss relic pool; eligible source predicates still apply | `onEquip`, `update` |
| Black Blood | `Black Blood` | `BlackBlood` | `BOSS` | Ironclad-red; Shared or Ironclad-red boss relic pool; eligible source predicates still apply | `canSpawn`, `onVictory` |
| Black Star | `Black Star` | `BlackStar` | `BOSS` | shared; Shared or Ironclad-red boss relic pool; eligible source predicates still apply | `onEnterRoom`, `onVictory` |
| Busted Crown | `Busted Crown` | `BustedCrown` | `BOSS` | shared; Shared or Ironclad-red boss relic pool; eligible source predicates still apply | `changeNumberOfCardsInReward`, `onEquip`, `onUnequip` |
| Calling Bell | `Calling Bell` | `CallingBell` | `BOSS` | shared; Shared or Ironclad-red boss relic pool; eligible source predicates still apply | `onEquip`, `update` |
| Coffee Dripper | `Coffee Dripper` | `CoffeeDripper` | `BOSS` | shared; Shared or Ironclad-red boss relic pool; eligible source predicates still apply | `canUseCampfireOption`, `onEquip`, `onUnequip` |
| Cursed Key | `Cursed Key` | `CursedKey` | `BOSS` | shared; Shared or Ironclad-red boss relic pool; eligible source predicates still apply | `justEnteredRoom`, `onChestOpen`, `onEquip`, `onUnequip` |
| Ectoplasm | `Ectoplasm` | `Ectoplasm` | `BOSS` | shared; Shared or Ironclad-red boss relic pool; eligible source predicates still apply | `canSpawn`, `onEquip`, `onUnequip` |
| Empty Cage | `Empty Cage` | `EmptyCage` | `BOSS` | shared; Shared or Ironclad-red boss relic pool; eligible source predicates still apply | `onEquip`, `update` |
| Fusion Hammer | `Fusion Hammer` | `FusionHammer` | `BOSS` | shared; Shared or Ironclad-red boss relic pool; eligible source predicates still apply | `canUseCampfireOption`, `onEquip`, `onUnequip` |
| Mark of Pain | `Mark of Pain` | `MarkOfPain` | `BOSS` | Ironclad-red; Shared or Ironclad-red boss relic pool; eligible source predicates still apply | `atBattleStart`, `onEquip`, `onUnequip` |
| Pandora's Box | `Pandora's Box` | `PandorasBox` | `BOSS` | shared; Shared or Ironclad-red boss relic pool; eligible source predicates still apply | `onEquip`, `update` |
| Philosopher's Stone | `Philosopher's Stone` | `PhilosopherStone` | `BOSS` | shared; Shared or Ironclad-red boss relic pool; eligible source predicates still apply | `atBattleStart`, `onEquip`, `onSpawnMonster`, `onUnequip` |
| Runic Cube | `Runic Cube` | `RunicCube` | `BOSS` | Ironclad-red; Shared or Ironclad-red boss relic pool; eligible source predicates still apply | `wasHPLost` |
| Runic Dome | `Runic Dome` | `RunicDome` | `BOSS` | shared; Shared or Ironclad-red boss relic pool; eligible source predicates still apply | `onEquip`, `onUnequip` |
| Runic Pyramid | `Runic Pyramid` | `RunicPyramid` | `BOSS` | shared; Shared or Ironclad-red boss relic pool; eligible source predicates still apply | No relic-class primary override; global/site residual |
| Sacred Bark | `SacredBark` | `SacredBark` | `BOSS` | shared; Shared or Ironclad-red boss relic pool; eligible source predicates still apply | `onEquip` |
| Slaver's Collar | `SlaversCollar` | `SlaversCollar` | `BOSS` | shared; Shared or Ironclad-red boss relic pool; eligible source predicates still apply | `onVictory` |
| Snecko Eye | `Snecko Eye` | `SneckoEye` | `BOSS` | shared; Shared or Ironclad-red boss relic pool; eligible source predicates still apply | `atPreBattle`, `onEquip`, `onUnequip` |
| Sozu | `Sozu` | `Sozu` | `BOSS` | shared; Shared or Ironclad-red boss relic pool; eligible source predicates still apply | `onEquip`, `onUnequip` |
| Tiny House | `Tiny House` | `TinyHouse` | `BOSS` | shared; Shared or Ironclad-red boss relic pool; eligible source predicates still apply | `onEquip` |
| Velvet Choker | `Velvet Choker` | `VelvetChoker` | `BOSS` | shared; Shared or Ironclad-red boss relic pool; eligible source predicates still apply | `atBattleStart`, `atTurnStart`, `canPlay`, `onEquip`, `onPlayCard`, `onUnequip`, `onVictory` |
| Brimstone | `Brimstone` | `Brimstone` | `SHOP` | Ironclad-red; Shared or Ironclad-red shop relic pool; shop-specific exclusions still apply | `atTurnStart` |
| Cauldron | `Cauldron` | `Cauldron` | `SHOP` | shared; Shared or Ironclad-red shop relic pool; shop-specific exclusions still apply | `onEquip` |
| Chemical X | `Chemical X` | `ChemicalX` | `SHOP` | shared; Shared or Ironclad-red shop relic pool; shop-specific exclusions still apply | No relic-class primary override; global/site residual |
| Clockwork Souvenir | `ClockworkSouvenir` | `ClockworkSouvenir` | `SHOP` | shared; Shared or Ironclad-red shop relic pool; shop-specific exclusions still apply | `atBattleStart` |
| Dolly's Mirror | `DollysMirror` | `DollysMirror` | `SHOP` | shared; Shared or Ironclad-red shop relic pool; shop-specific exclusions still apply | `onEquip`, `update` |
| Frozen Eye | `Frozen Eye` | `FrozenEye` | `SHOP` | shared; Shared or Ironclad-red shop relic pool; shop-specific exclusions still apply | No relic-class primary override; global/site residual |
| Hand Drill | `HandDrill` | `HandDrill` | `SHOP` | shared; Shared or Ironclad-red shop relic pool; shop-specific exclusions still apply | `onBlockBroken` |
| Lee's Waffle | `Lee's Waffle` | `Waffle` | `SHOP` | shared; Shared or Ironclad-red shop relic pool; shop-specific exclusions still apply | `onEquip` |
| Medical Kit | `Medical Kit` | `MedicalKit` | `SHOP` | shared; Shared or Ironclad-red shop relic pool; shop-specific exclusions still apply | `onUseCard` |
| Membership Card | `Membership Card` | `MembershipCard` | `SHOP` | shared; Shared or Ironclad-red shop relic pool; shop-specific exclusions still apply | `onEnterRoom` |
| Orange Pellets | `OrangePellets` | `OrangePellets` | `SHOP` | shared; Shared or Ironclad-red shop relic pool; shop-specific exclusions still apply | `atTurnStart`, `onUseCard` |
| Orrery | `Orrery` | `Orrery` | `SHOP` | shared; Shared or Ironclad-red shop relic pool; shop-specific exclusions still apply | `onEquip` |
| Prismatic Shard | `PrismaticShard` | `PrismaticShard` | `SHOP` | shared; Shared or Ironclad-red shop relic pool; shop-specific exclusions still apply | `onEquip` |
| Sling of Courage | `Sling` | `Sling` | `SHOP` | shared; Shared or Ironclad-red shop relic pool; shop-specific exclusions still apply | `atBattleStart` |
| Strange Spoon | `Strange Spoon` | `StrangeSpoon` | `SHOP` | shared; Shared or Ironclad-red shop relic pool; shop-specific exclusions still apply | No relic-class primary override; global/site residual |
| The Abacus | `TheAbacus` | `Abacus` | `SHOP` | shared; Shared or Ironclad-red shop relic pool; shop-specific exclusions still apply | `onShuffle` |
| Toolbox | `Toolbox` | `Toolbox` | `SHOP` | shared; Shared or Ironclad-red shop relic pool; shop-specific exclusions still apply | `atBattleStartPreDraw` |
| Neow's Lament | `NeowsBlessing` | `NeowsLament` | `SPECIAL` | shared-special; Neow opening blessing offer | `atBattleStart`, `setCounter` |
| Golden Idol | `Golden Idol` | `GoldenIdol` | `SPECIAL` | shared-special; Golden Idol Act 1 event | No relic-class primary override; global/site residual |
| Odd Mushroom | `Odd Mushroom` | `OddMushroom` | `SPECIAL` | shared-special; Hypnotizing Colored Mushrooms event | No relic-class primary override; global/site residual |
| Warped Tongs | `WarpedTongs` | `WarpedTongs` | `SPECIAL` | shared-special; Ominous Forge event | `atTurnStartPostDraw` |
| Spirit Poop | `Spirit Poop` | `SpiritPoop` | `SPECIAL` | shared-special; Bonfire Spirits event | No relic-class primary override; global/site residual |
| Face of Cleric | `FaceOfCleric` | `FaceOfCleric` | `SPECIAL` | shared-special; Face Trader event | `onVictory` |
| Cultist Headpiece | `CultistMask` | `CultistMask` | `SPECIAL` | shared-special; Face Trader event | `atBattleStart` |
| Gremlin Visage | `GremlinMask` | `GremlinMask` | `SPECIAL` | shared-special; Face Trader event | `atBattleStart` |
| N'loth's Hungry Face | `NlothsMask` | `NlothsMask` | `SPECIAL` | shared-special; Face Trader event | `onChestOpenAfter`, `setCounter` |
| Ssserpent Head | `SsserpentHead` | `SsserpentHead` | `SPECIAL` | shared-special; The Ssssserpent event | `onEnterRoom` |

## Primary mechanics and interaction obligations

The per-ID matrix is a finite inventory of direct hook entrypoints. Its distinct source families include pickup/equipment mutations; battle and turn starts; end-turn counters; card play/draw/exhaust; qualifying victory; gold spending/earning; card acquisition preview versus obtain; damage, Block, healing and HP-loss modifiers; chest-open hooks; potion use; campfire availability; reward-count changes; and delayed/relic-selection updates. Each direct override must be represented at its true trigger, not normalized to one generic pickup callback. Primary resource families include HP/max-HP, gold, combat resources, deck/card state, potion capacity/obtain, relic counters, chest curses/relics/keys, and map/knowledge-facing state.

Thirteen candidate classes define no non-display/non-copy primary override: Paper Frog, White Beast Statue, Calipers, Ginger, Ice Cream, Turnip, Runic Pyramid, Chemical X, Frozen Eye, Strange Spoon, Golden Idol, Odd Mushroom, and Spirit Poop. This is not an inertness classification. Their behavior can be implemented at shared action/resource logic, public-information projection, or a source-specific event. Fresh per-relic call-site closure for those global/site effects is still required. A design must not classify the mechanic as complete merely because its concrete class has an ID/tier or passes `canSpawn`.

Cross-relic integration still required includes actual current-character initialized pools; eligible-ID and depletion laws; no universal pool inference from Shard; acquisition preview/commit separation; Omamori and curse prevention; on-obtain ordering; eggs, permanent-copy and removal hooks; repeated counters and reset boundaries; death/cancellation and already-done effects; delayed chest/key/relic outcomes; and all parent-screen continuations. The nested UI return contract and universal shared callback closure are outside this audit. It does not turn local source facts into whole-Act-1 readiness.

## Validation and remaining limits

A private, failure-first Python-standard-library validator checks the 141 class/input records, unique IDs, tier totals, source-hash correspondence, and constructor IDs/tier arguments against reconstructed class source. Its 6 tests passed, including negative controls for duplicate IDs, candidate/manifest mismatch, missing metadata, and constructor-ID/tier mismatches. No gameplay, model, product, build, or original-runtime test was run. Static reconstruction establishes source structure for the pinned bytes, not actual runtime behavior or distributional parity.

The fresh independent review should verify the candidate-to-constructor mapping and sampled/all relevant per-class callbacks, challenge global-site gaps and source predicates, and confirm that the public claims remain bounded. Main retains Act 1 integration, acceptance, and all whole-spec/plan/start gates; this supporting report closes none of them by itself.
