# Act 1 acquisitions: source-specific continuation and exceptional-choice contracts

Date: 2026-10-19. Bead: `spire-gli.2.3.4`.

**Status: static closure of the bounded contracts below, pending independent
review; not an approved whole-Act-1 specification or implementation plan.** The
scope is standard, unmodded, fully unlocked, Act-4-enabled Ironclad A0 Act 1,
including legally acquired Shard/Note cards, skips, deaths and deferred keys.

This advances the [previous continuation findings](2026-10-18-act1-continuations-closure.md)
through actual room, screen, effect and owned-relic dispatch. It retains the
[native ownership constraints](2026-10-18-act1-native-contract-closure.md) and
[corrected relic selection distinctions](2026-10-18-act1-relics-closure.md).
Those documents remain historical evidence; this document supplies the more
specific conditional rules, not a replacement universal scheduler.

## 1. Evidence and vocabulary

The owned archive is SHA256
`cfad868ac8d65a88e71a0bf096fb09f78811e553effe0787c5309a655e081673`.
Pinned Temurin 21.0.12.1+1 and CFR 0.152 were used only as static tools. Four
archive/tool pins matched before and after inspections. Exact unique class
entries, command exits and output hashes are retained privately. No original
game class, initializer, substituted runtime or original runtime executed.

**S** means retained or fresh reconstructed source inspected through the named
members; **BC** means the particular branch/order was checked in fresh bytecode;
**D** is a direct consequence of those branches; **P** is a proposed native
contract, not an approved fidelity choice. A future acceptance case is not an
executed gameplay test. Class counts and saved disassemblies alone are not BC.

“Consume” means remove the particular offer/selection from its owning context;
it does not imply that a scheduled obtain has inserted a permanent card.
“Return” identifies a source resume point, not an assumed stack pop. Screen
completion, room `COMPLETE`, reward consumption, ownership, profile persistence
and successor readiness are different boundaries.

## 2. The dispatch and return rules actually used

| Entry / condition | Established consequence | Grade / source locator |
|---|---|---|
| Normal desktop frame reaches game update | The launcher installs the LWJGL backend; its raw and ordinary delta getters return the same field. `CardCrawlGame.render` returns for raw delta greater than `0.1f` before game update. Thus an admitted ordinary delta is at most `0.1f`; an oversized frame never reaches dungeon/effect update through this caller. | S/BC: `DesktopLauncher.main`, `LwjglApplication` constructor / `mainLoop`, `LwjglGraphics.getDeltaTime/getRawDeltaTime`, `CardCrawlGame.render`; `TimeHelper.update` does not change backend delta |
| Dungeon update reaches its effect section | Active top-level effects are updated, then tested for completion/removal; the ordinary active effect list follows the same update-before-test order. An already-done object is not filtered before its update. | BC: `AbstractDungeon.update` |
| Work is in either pending effect queue | Pending objects are transferred to their corresponding active lists after the active-list passes. Transfer does not itself update the object, or check its done flag. | BC: `AbstractDungeon.update` |
| Room contains a spawned relic | `AbstractRoom.updateObjects` updates it before checking removal from the room's relic list. This call occurs before the dungeon's selected-screen branch. | S: `AbstractRoom.updateObjects`; BC: dungeon caller |
| Relic is installed in the player's inventory | `OverlayMenu.update`, after the dungeon effect passes, updates installed relics. Its Proceed and Cancel handlers run before that owned-relic loop. Relic-specific selection consumers therefore remain live outside their original offer screen. | S: `OverlayMenu.update`, `AbstractDungeon.update` |
| Grid or card-reward screen is active | Its screen update runs; the room update is additionally called only in the Peek branch. Ordinary event/Neow updates are not a universal concurrent consumer behind every screen. | BC: `AbstractDungeon.update`; S: `EventRoom.update`, `NeowRoom.update` |
| Event/Neow room resumes with no screen up | Its event update can consume the grid result. Shop purge has consumers in both `ShopScreen.updatePurge` and `ShopRoom.updatePurge`; both clear the consumed selection. | S: named room/screen members |
| Grid closes and the combat reward list is nonempty | Grid closure replaces the remembered return screen with `COMBAT_REWARD`, even if another return screen had been remembered. | BC: `AbstractDungeon.closeCurrentScreen` |
| Card-reward screen closes normally | Its close hook runs unless screen-swap handling suppresses it. The hook itself is not acquisition and does not remove the associated card-reward row. | S/BC: `closeCurrentScreen`, `CardRewardScreen.onClose` |
| No remembered screen remains after close | The current screen becomes `NONE`; this is not an instruction to open the map. | BC: `closeCurrentScreen` |
| A remembered screen remains, without screen-swap suppression | That specific screen is reopened and the remembered field cleared; reward-time state may restore a combat-reward return marker. Reopening a card reward with a non-null reward item also establishes a combat-reward return. | S/BC: `closeCurrentScreen`, `openPreviousScreen` |

The source has a mutable remembered-screen field and special cases, not proof of
an arbitrary-depth stack. **P:** native continuations should retain semantic
parents, selected instance identities, captured parameters and the exact resume
condition. Reopening must not regenerate stock/offers, repay, or reapply hooks.
That representation must reproduce the conditional returns above rather than
promise that every child automatically reopens its apparent parent screen.

## 3. Parent-to-child closure matrix

The rows describe source-local normal completion, conditional on execution
reaching the indicated consumer. They do not promise completion after a terminal
interruption. All are **S**, with section 2's dispatcher facts additionally BC.

| Parent / child site | Local commitment and result consumer | Resume / skip contract |
|---|---|---|
| Neow ordinary, rare-choice or colorless card offer | `NeowReward.activate` opens an ordinary card-reward screen with no associated reward item. Take schedules Fast obtain; Bowl uses section 5; Skip merely closes. | `NeowRoom` resumes `NeowEvent` when no screen is up; its post-blessing dialog can open the map. No invented mandatory card pick. |
| Neow upgrade / remove / transform grid | Activated `NeowReward.update` consumes selected cards once and clears them. Upgrade mutates the chosen permanent instance; removal uses object removal. Single transform generates before removal; two-card transform removes both inputs before its two generations. | The event remains the parent. Its save-wait checks are not a proof that every durable effect queue is empty. Do not make one generic remove-then-transform order. |
| Neow random card / curse drawback | Random card and later curse drawback schedule Show obtain in the top-level active list. Drawback activation precedes reward activation, but the curse drawback is constructed in the later reward update. | No extra card-choice menu. The obtain constructor/update distinction in section 4 still applies. |
| Neow random relic / boss swap | Room spawn installs a moving relic through its own update path; arrival enters `onEquip`. Boss swap removes the starter before selecting the replacement. | A nested relic choice belongs to that acquired relic, then resumes Neow. Starting-card resources do not come from an off-color card's character. |
| Neow three potions | Open combat rewards, then remove its automatically generated card-reward row. Potion claims retain capacity/Sozu distinctions. | Proceed's Neow branch marks the room complete and opens the map; leaving potions unclaimed is legal. Discarded card generation has already affected reward history. |
| Shop card purchase | Construct and enqueue Fast obtain, then spend gold and remove/restock the purchased stock under that source's rules. Constructor callbacks, stock mutation and permanent insertion are separate. | Shop remains a continuing visit; ordinary purchases do not force map exit. |
| Shop potion purchase | Sozu refusal occurs at this caller; successful belt insertion is required before spend/restock. Full belt preserves gold and stock. | A failed purchase is not an obtained potion or a consumed offer. |
| Shop paid purge | Opening the cancellable grid reserves no payment. A confirmed selected result causes a purge consumer to spend, advance price, remove the selected object, clear selection and disable this visit's service. | Cancel without a selected result neither spends nor consumes the service. Remembered Shop normally reopens, subject to section 2's nonempty-reward override. |
| Shop relic purchase | Spend and install via `instantObtain(..., true)`; `onEquip` may open a child before remaining-stock preview and restocking finish. | A child does not undo payment or justify purchasing/restocking a second time. Owned-relic updates consume its later grid result. |
| Shop Orrery / Cauldron | A labelled reward screen is opened; Orrery creates four explicit card rows plus the automatic fifth; Cauldron creates five potions then removes the automatic card row. | Labelled rewards use Cancel on reopen. Neither relic sets a universal Shop return marker: absent another marker, close reaches `NONE`; the existing merchant can reopen the same shop inventory. Do not regenerate a visit. |
| Boss-combat reward → boss treasure | `ProceedButton.goToTreasureRoom` starts a distinct room transition. Boss treasure is not a boss-combat reward-rarity context. | Remaining boss-combat rewards may be skipped. Boss lethal damage is not Act-1 task completion. |
| Boss treasure → relic take / skip | Selection marks the room's `choseRelic`; ordinary relic obtain/arrival later enters `onEquip`. Black Blood uses the replacement/instant path. Skip closes the chest/screen without acquisition. | The screen closes before ordinary arrival/onEquip children. Skipping the chest entirely is also possible through Proceed; no-pick is recorded. |
| Boss relic → grid / confirmation / rewards | Owned-relic updates, not the vanished boss offer, consume later selected results. The exact child families are below. | Their local completion/timer flags do not certify final readiness; legal child rewards remain skippable where offered. |
| Reward list → card row | `RewardItem.claimReward` opens card choice and remembers combat rewards, but returns false so the row remains pending. | Take removes this row before its Fast obtain resolves. Skip returns to the list without consuming the row, so the offer can be reopened. Whole-list Proceed is the separate decision to leave rewards. |
| Reward list → relic row | Claim uses instant obtain/onEquip and consumes the row on success; it refuses a claim while current screen is Grid. A linked sapphire row is marked ignored/done. | New relic children retain their own selection consumer and source return conditions. |
| Event → selection/result | `EventRoom.update` resumes its event only when no screen is up. A result Continue and `AbstractEvent.openMap` are distinct from completion of a child effect. | Each event below owns its result path; there is no generic automatic return-to-map after every selection. |
| Dead Adventurer / Mushrooms combat → rewards | Actual room remains EventRoom. Surviving combat constructs offers; neither `eliteTrigger` nor an elite enemy makes it MonsterRoomElite. | Proceed from their combat rewards closes and directly opens the map. Lab shares this Act-1 direct-map exception; Woman in Blue does not. Death does not grant unclaimed loot. |
| Rest → Dream Catcher | Sleep effect heals, then calls `onRest`, later generates an ordinary card offer if nonempty, and locally completes the room. Its card screen has no associated reward item. | Card/Skip/Bowl closes back to the rest context, not a fabricated combat-reward row. No card screen is opened for an empty generated offer. |

### Relic-owned nested children

These are continuation facts, not the full relic-mechanics audit.

| Relic child | Selection/commit contract | Exceptional-size branch / return |
|---|---|---|
| Bottled Flame / Lightning / Tornado | Typed eligible permanent-card grid; installed relic update attaches its bottle flag, completes the room and clears selection. | Equip checks for a nonempty typed group; spawn predicate and equip predicate are separate. No arbitrary off-type fallback. |
| Dolly's Mirror | Grid over permanent deck; update makes a stat-equivalent copy, clears bottle flags on the copy and schedules Show obtain. | No local empty-deck shortcut. Selection completes the room before the copy's obtain finishes. Remembered parent is captured only if a screen was up. |
| Astrolabe | Up to three purgeable cards; per card, object removal precedes upgraded transform and queued Show obtain. | Zero: no selection/transform. One–three: automatically transform all. More: require three selected results. Clears selection and sets reward timer, not a general drain. |
| Empty Cage | Up to two purgeable cards removed through object hooks; grid branch waits for exactly two results. | Zero: sets internal selection-complete flag and returns, after setting room INCOMPLETE. One–two: automatic deletion and room COMPLETE. Do not erase this phase difference or infer deadlock from it alone: visible Proceed/event controls have separate routing. |
| Pandora's Box | Removes starter-tagged cards with an iterator, generates replacements, previews them, and opens a confirmation-only grid. | No matching starters: no confirmation grid or replacements. Confirmation schedules Fast obtains for all shown cards; it is not free re-selection. |
| Calling Bell | Confirmation-only grid schedules Fast obtain of its fixed curse. When no screen remains, installed-relic update opens rewards, clears the automatically generated rows, inserts three screenless relic offers. | Prevented curse acquisition does not condition away the subsequent relic offers. Skipping those offers remains legal. Discarded automatic card generation still changed history. |
| Tiny House | Random permanent upgrade if possible, +5 max-HP call, gold/potion rows, then labelled rewards including the ordinary automatic card offer. | No upgradeable card means only the upgrade is absent; other grants/rewards are not cancelled. Neow and boss-treasure contexts retain their own reward law. |

Locators: `NeowReward.activate/update`, `NeowEvent.update/buttonEffect`,
`ShopScreen.purchaseCard/purchasePurge/updatePurge/open`, `ShopRoom.updatePurge`,
`StoreRelic.purchaseRelic`, `StorePotion.purchasePotion`, `Merchant.update`,
`RewardItem.claimReward`, `CombatRewardScreen.open/reopen/setupItemReward`,
`BossRelicSelectScreen.update/relicObtainLogic/relicSkipLogic`,
`AbstractRelic.obtain/update/instantObtain`, and named relic `onEquip/update`.

## 4. Curse obtain and removal: close the actual entered call

### Obtain/prevention matrix

| Route and condition | Constructor / dispatch consequence | Durable obtain consequence |
|---|---|---|
| Fast obtain, curse, Omamori counter nonzero | Constructor consumes a charge and marks effect done; the dispatcher still enters update, whose own initial done guard returns. | No obtain callbacks or permanent insertion from that update. BC: `FastCardObtainEffect` plus dungeon dispatcher. |
| Show obtain, curse, Omamori counter nonzero | Constructor consumes a charge and marks done **but continues initialization and replaces duration with 0.5 (fast mode) or 2 seconds**. Dispatcher enters update before removing done effects; Show has no initial done guard and enters the card's update. | On the normal desktop caller path, the first admitted subtraction is at most `0.1f`, leaving positive duration. No obtain/deck-change hooks or permanent insertion run; the dispatcher removes the already-done effect. BC/D: constructor, update, dispatcher and upstream frame gate. |
| Either effect, no prevention | On its qualifying update, ordered relic obtain hooks precede Soul obtain; deck-change hooks follow. | `SoulGroup.obtain` reaches `Soul.obtain`, which inserts into master deck immediately; the later flying-card animation is not the insertion boundary. S. |
| Note incoming card, including a stored removable curse | Directly invokes obtain hooks, adds the card to master deck, then invokes deck-change hooks; no Omamori effect constructor is involved. | Charge is not consumed by this path. Incoming card is available before the outgoing selector is built, and can itself be selected. S: `NoteForYourself.buttonEffect`. |

The upstream desktop bound closes durable prevention for **both Fast and Show**:
there is no accumulation over later frames because the dispatcher removes the
already-done effect after its first admitted update. A rejected long frame does
not supply an oversized subtraction. Their one-time constructor bookkeeping still
runs; Show additionally marks the card seen and queues its poof effect. Show enters
card update, whereas Fast's own done guard returns before card update. Preserve
these differences without a universal done prefilter or a fabricated deck grant.

A direct Show update with a subtraction exceeding its initialized duration would
enter the obtain block: this is a **method-local counterfactual outside the admitted
normal desktop domain**, not a legal game choice or native fidelity obligation.
Native elapsed-time policy remains a human-unapproved profile/time proposal, but
cannot use that counterfactual to invent prevented curse acquisition. Global
admitted/current/pending-work arbitration remains execution-owned. Note's direct
obtain is still distinct. Constructor bookkeeping is not a durable deck grant.

### Removal matrix

| Site / removal path | Established order and qualification |
|---|---|
| Master-deck object removal | Removes the object, invokes that card's removal hook, then ordered relic deck-change hooks. The hook call is not guarded by the boolean result of list removal. S/BC: `CardGroup.removeCard(AbstractCard)`. Native accepted choices must refer to actual current candidates, not stale objects. |
| Parasite's removal hook | Calls max-HP decrease by 3. The helper floors maximum HP at 1 and clamps current HP down to it; it does not itself call damage, HP-loss callbacks or a death transition. A living 1-HP player is not killed by this local maximum-HP adjustment. S/BC: `Parasite.onRemoveFromMasterDeck`, `AbstractCreature.decreaseMaxHealth`. |
| String-ID removal / Pandora iterator removal | Does not perform the object-removal callback sequence. Do not give it Parasite/removal-hook behavior merely because the final deck shrinks. S/BC: string overload; S: `PandorasBox.onEquip`. |
| Shop purge / Cleric / Wing Statue | Shop pays on confirmed selection before object removal; Cleric pays before opening a nonempty mandatory grid; Wing Statue's damage occurs before its later purge selection. Independent payment/damage is not Parasite max-HP loss. S. |
| Living Wall / Transmogrifier / Astrolabe / Neow transforms | Living Wall and Transmogrifier remove the selected object before generating its replacement. Astrolabe removes each input before its upgraded generation. Neow's single/double distinction is in section 3. Generated replacement acquire hooks are a separate step. S. |
| Bonfire | Applies the rarity reward **before** object removal. For a curse this schedules Spirit Poop/Circlet, then removal invokes Parasite's hook if applicable. Do not reorder all sacrifices into remove-before-reward. S: `Bonfire.update/setReward`. |
| Note outgoing card | Object removal and its hooks precede saving the live outgoing reference. ID/upgrade persistence occurs on the next room transition, not at selection; no misc scaling is written. S. |

Parasite's local nonlethal helper does not certify every independent removal
callback, event damage, rescue or terminal path. That callback-family inventory
is the execution/relic interface, not a reason to omit already-entered site work.
Note's detached staged/committed profile outputs, death/fault/reset behavior and
elapsed-time proposal remain owned by the profile/time lane.

## 5. Singing Bowl: downstream +2 and reward-item edge conditions

`SingingBowlButton.update/onClick`, `CardRewardScreen.open/reopen`, and
`AbstractCreature.increaseMaxHp/heal(int, boolean)` establish these **S/BC** facts:

* Ordinary card offers enable Bowl when owned; draft, codex, discovery and
  choose-one contexts hide it. A tutorial can additionally suppress the buttons;
  this is not permission to offer Bowl in a discovery choice.
* On action, maximum HP increases by 2 **before** a heal request for 2. Healing
  still runs its player-relic and power modifiers, cap and dying guard. Without
  modifying hooks and while living, `h/H` becomes `(h+2)/(H+2)`, including when
  previously full. It is neither maximum-only nor a full heal. The maximum change
  is not undone because a later heal is refused.
* The button removes its exact associated reward item, then the input handler
  records Bowl, closes the screen and hides the button. It does not acquire a
  card, claim a relic/potion, or invoke sapphire-link logic.

| Associated item / remaining list | Established local behavior |
|---|---|
| Present card row among other rows | Removes only that row. Other card, gold, potion, relic and key rows survive unchanged by this removal. |
| Last card row | List becomes empty, but this callback does not run `positionRewards` or explicitly set `hasTakenAll`. Screen reopening determines controls; do not derive durable readiness from that UI flag. |
| Null item, as at Neow or Dream Catcher | Removal of null is a no-op for the normally non-null reward list; the +2/heal and source screen close still occur. No fake reward row is required. |
| Non-null item already absent | List removal failure does not suppress the HP operation. This is conditional local behavior, not evidence that stale submissions are legal native actions. |
| Unexpected item type or linked item supplied | The callback has no type check or linked-item dispatch; it only removes the supplied object. Natural card-screen callers supply card rows or null. Do not broaden policy candidates to arbitrary reward objects. |

Ordinary Skip simply closes and leaves a card row available for reopening; it
does not pay Bowl's benefit. Ordinary Take consumes that row and schedules Fast
obtain. Bowl/Take/Skip must be three distinct actions where legal, including in
multi-offer Orrery rewards. Reopening a skipped offer must not resample its cards.

## 6. Match and Keep: input, reveal, attempt and final-pair timing

**S/BC:** `GremlinMatchGame.initializeCards/placeCards/updateMatchGameLogic/update`.
At A0 the event constructs six pairs / twelve instances and starts with five
attempts. Its generators and seeded shuffle law are source-specific; Shard does
not turn this board into ordinary reward generation. The shuffle-law lane owns
its finite conditional distribution, not this continuation contract.

| Phase / condition | Input and visible state | Commitment / next phase |
|---|---|---|
| Intro then rule explanation | No board pick yet. | The next dialog starts Play and places all cards face down. There is no source initial strategic Leave choice. |
| Play, zero wait, first face-down position chosen | Reveal that one card immediately; retain it as first choice. Clicking its now-face-up instance does not select it again. | No attempt consumed. Second choice remains genuine policy input. |
| Second face-down position chosen | Reveal it; comparison is by card ID, not object identity. | Set pair wait to 1 second for a match, 1.25 for a miss; no attempt decrement yet. |
| Nonzero pair wait, before strict underflow | Pair remains exposed; normal board selection branch is not entered. | No removal, obtain or attempt decrement merely from the second click. |
| Wait strictly crosses below zero, game not already done: match | Remove the two board objects and schedule Show obtain of a fresh `makeCopy` of the first card, not the board's stat-equivalent object. | Clear selected references and consume one attempt. Obtain resolution remains separate; eggs/curse hooks follow the actual obtain path. |
| Same boundary: miss | Turn both cards face down and clear selected references. Previously revealed knowledge need not be forgotten by the policy. | Consume one attempt, exactly as for a match. |
| That decrement reaches zero | Set game-done and a final wait of 1 second. | No sixth attempt. Next Play update enters Clean-up through the game-done branch, rather than waiting for that final timer to expire before starting cleanup. |
| Clean-up | Move remaining cards out of the board once; no board picks. | With positive remaining wait, strict underflow shows the Complete dialog. Complete's button opens the map. This is not a guaranteed obtain-effect drain. |

The five-attempt law is a game rule, not an agent/tool step cap. On the ordinary
five resolved-pair path, even five matches leave two board cards, so the empty
board controller-index branch is not justified as a normal sixth-match outcome.
Offer candidates by visible position, not hidden identity or a compacted ordering
that leaks unrevealed cards; preserve legitimately revealed memory.

**Exact timer qualification:** zero and negative are not interchangeable here.
If a pair wait lands exactly on zero, the following Play call enters the input
branch rather than the underflow-resolution branch. After the ordinary fifth
resolved pair sets wait to 1, normal next-Play cleanup entry stays positive:
that call subtracts at most `0.1f`. Cleanup's later positive wait can still land
exactly on zero, failing to enter its guarded strict-underflow completion path.
Nonpositive cleanup entry is a method-local counterfactual, not that normal
fifth-pair path. These are BC/D conditions, not observed natural-runtime
frequencies or authority to invent retries. A deterministic native logical
reveal/pair-resolution/cleanup schedule is a **P** requiring the profile/time
fidelity decision; it must not accidentally
erase these distinctions while claiming original timing parity. It must also not
expose the second hidden identity before the first-pick observation.

## 7. Finite event and empty-candidate matrix

These **S** rows cover the 25 Act-1 event sites identified by the reviewed source
inventory. “Leave” refers to a source choice, not an added escape from a committed
mandatory grid. Result Continue normally calls the event's map path, unless the
explicit reward/combat route below intervenes. Generation eligibility/history is
not redefined by this table.

| Event / source class | Nested or exceptional-choice law |
|---|---|
| Big Fish / `BigFish` | Heal, max-HP or curse+relic branch; no initial Leave. Curse and spawned relic are different work. Result Continue is not automatic acquisition completion. |
| Dead Adventurer / `DeadAdventurer` | Search/Leave; successful tokens removed from later fight rewards. Exhausted searches reach the result path, not an extra free search. Combat/reward/map route is section 3. |
| Golden Idol / `GoldenIdolEvent` | Take/Leave; Take schedules relic before the mandatory injury/damage/max-HP choice. That second choice is not a new free Leave. |
| Mushrooms / `Mushrooms` | Fight/eat, no initial Leave. Eat heals before scheduling Parasite. Fight uses event combat and skippable rewards. |
| Living Wall / `LivingWall` | Upgrade initially disabled if no upgradeable card. Each selected route separately checks the nonbottled purgeable count before opening its grid; zero bypasses grid to result, without selection mutation. This extra gate also applies to Grow. No initial Leave is fabricated. |
| Scrap Ooze / `ScrapOoze` | Reach/Leave on repeated failures; damage precedes success test. Successful relic branch reaches result; no forced retry. |
| Shining Light / `ShiningLight` | Enter disabled when no upgradeables; Leave remains. Enter damages before selecting up to two upgradeable instances. One candidate upgrades once; no fabricated second card. |
| Cleric / `Cleric` | Affordability controls heal/remove choices; Leave remains. Remove with zero nonbottled purgeables opens no grid and spends no removal gold, then reaches Proceed. |
| Ssssserpent / `Sssserpent` | Agree confirmation precedes gold+Doubt scheduling; Leave exists. This is not Face Trader's Ssserpent Head reward. |
| Wing Statue / `GoldenWing` | Damage branch leads through a later mandatory purge grid, without a local empty-candidate guard. Gold branch has its own permanent base-damage eligibility; Leave exists initially. Do not turn an empty committed grid into a free post-damage Leave. |
| World of Goop / `GoopPuddle` | Recover invokes damage before gold; alternate Leave loses the precomputed capped gold amount, including zero if no gold. No card candidate is required. |
| Golden Shrine / `GoldShrine` | Ordinary gold, gold+fixed Regret, or Leave; curse uses its effect path, not a direct Note obtain. |
| Match and Keep / `GremlinMatchGame` | Section 6; five attempts, no initial strategic Leave or forced extra attempt. |
| Purifier / `PurificationShrine` | Initial remove/Leave; choosing remove opens a noncancellable grid without an empty guard. No generic automatic skip is evidenced after committing it. |
| Transmogrifier / `Transmogrifier` | Initial transform/Leave; committed noncancellable grid likewise lacks an empty guard. Callback removes the object before generation and queued Show obtain. |
| Upgrade Shrine / `UpgradeShrine` | Upgrade disabled when no upgradeables; Leave remains. Callback upgrades the permanent selected instance and refreshes its bottle link. |
| Wheel / `GremlinWheelGame` | Gold applies at reveal, not confirmation again. Other results have their own later callback. Purge result checks for nonempty nonbottled purgeables; empty opens no grid and grants no replacement. Relic result opens skippable rewards; no automatic relic grant. |
| Ominous Forge / `AccursedBlacksmith` | Upgrade disabled when no upgradeables; rummage and Leave remain. Rummage's Pain effect and Warped Tongs obtain are distinct. |
| Bonfire / `Bonfire` | With no nonbottled purgeables, the Choose phase moves to Complete without a sacrifice/reward. Otherwise reward precedes object removal. There is no general arbitrary sacrifice or initial Leave. |
| Face Trader / `FaceTrader` | Touch/trade/Leave; missing-face selection falls back to Circlet if no face remains. Source shuffle measure is a separate dependency, not uniform categories by tooltip. |
| Divine Fountain / `FountainOfCurseRemoval` | Cleanse/Leave; cleanse walks actual eligible curse objects, retaining unremovable/bottled exclusions. An empty removable subset does not create a fallback card or curse. |
| Lab / `Lab` | Offers potions with auto card suppressed. Full belt does not suppress the offers; claims can fail. Proceed uses the direct-map exception. |
| Note / `NoteForYourself` | Take or Ignore after initial reveal. Incoming direct obtain precedes building outgoing nonbottled purgeables. A valid removable incoming card supplies a candidate even if the prior removable subset was empty. The selector has no local empty guard if the incoming card is itself unremovable and no other candidate exists. Ignore has no outgoing selection/write. |
| We Meet Again / `WeMeetAgain` | Missing potion, insufficient gold or no eligible nonbasic/noncurse card disables the corresponding sacrifice; Leave remains. One actual sacrifice precedes one spawned relic, not a three-relic menu. |
| Woman in Blue / `WomanInBlue` | Buy potion quantity or A0 Leave. Each paid branch calls ordinary unlabelled `CombatRewardScreen.open()`; Proceed closes it back to event/result, not Lab's direct-map exception. This is not labelled `open(String)`/Cancel handling; claims retain Sozu/capacity behavior. |

### Non-event empty/depleted choices

| Condition | Actual law; no universal empty-list fallback |
|---|---|
| Ordinary reward helper produces zero cards | `CombatRewardScreen.setupItemReward` does not append that empty card row. Existing other rewards and Proceed remain. Dream Catcher separately guards its card-screen opening. |
| Card row skipped / list has no claimable potion capacity | Skip/Proceed remains distinct from successful claims. Full belt leaves a potion reward available; Sozu reward claim consumes it without insertion, unlike shop refusal. |
| Shop stock exhausted or unaffordable | No fabricated purchase; shop close/room Proceed remains. A cancellable empty paid-purge grid can be exited without payment. |
| Grid requests a fixed count larger than candidate count | Generic grid open does not clamp the requested count or auto-complete empty selection. Source-specific automatic smaller-count branches, such as Cage/Astrolabe, must not be inferred for every caller. |
| Common/uncommon/shop relic pool depleted | Promotion preserves the corrected helper direction: depleted common→uncommon, uncommon→rare, shop→uncommon through normal/front. Normal rejection retries through end at requested tier; end retries through itself. |
| Rare/boss relic pool depleted | Circlet / Red Circlet respectively, not an absent reward or silent reroll. Nonempty boss selection uses front even in the end helper. Offers consumed from pools remain consumed when skipped. |
| Event-selection ordinary list empty | Ordinary helper delegates to shrine selection. The shrine helper has no established empty-set fallback. This is not permission to invent regeneration, forced event, skip or defeat; actual initialized reachability remains a separate generation proof. |

Mandatory empty-grid paths and unusual count/phase combinations above are
**closed local source conditions**, not proof that each is naturally reachable
in initialized Act 1. The smallest remaining obligation is a legal history witness
or exclusion proof for each problematic entry state, followed by an explicit
native decision if necessary. Do not silently disable a source action, invent a
post-commit Leave, or call a tool failure a death.

## 8. Campfire, keys and the resolved handoff

Retain this exact construction order: **initial Rest/Smith/relic-added options →
held-relic usability filtering, short-circuiting per rejected option → eligible
Recall append → final usable-option scan**. Coffee Dripper rejects RestOption;
Fusion Hammer rejects SmithOption. Their combination does not disable appended
Recall. Smith requires upgradeables; Peace Pipe supplies Toke usability from its
own eligible count; exhausted Girya is not a forced Lift. If the final scan finds
none usable, the room completes rather than fabricating a choice. **S**, with
the two named boss-relic predicates retained from reviewed BC.

Smith/Toke grid cancellation returns to the campfire menu through Cancel's
RestRoom branch. Their effect consumers apply selected mutations only when a
result exists. Rest's heal/`onRest` and Dream Catcher remain ordered as section 3.
Recall schedules its own effect; that schedules Ruby and locally completes the
room before the Ruby flag changes. Emerald and Sapphire also commit in later
`ObtainKeyEffect.update`, not on offer/claim construction. Sapphire competes only
with its linked base relic; Emerald is not relic-linked. Legal skips and deferred
keys are not failures to repair.

The source Act-1 successor path is more concrete than “boss defeated → done”:

| Boundary | Source-established state / remaining obligation |
|---|---|
| Boss rewards Proceed | Moves to `TreasureRoomBoss`, whose Exordium successor name is TheCity. |
| Boss treasure take/skip and children | Source-specific local completion as section 3; Proceed can leave unclaimed offers. No generic list-empty requirement. |
| Next-dungeon Proceed | Records no-pick if applicable; a three-relic warning can delay this handler once; then starts fade and marks dungeon beaten. It does not itself construct Act 2. |
| Main game transition | `CardCrawlGame.update` waits for the beaten/faded condition, creates the transition screen, then constructs the named successor with the same player when that screen completes. |
| Successor construction | Base dungeon transition setup advances act, clears/reinitializes per-act lists and room probabilities, resets the room potion modifier and performs the A0 max-HP-sized heal request. This uses actual healing hooks, not unconditional HP assignment. The source also advances the original card-RNG counter within its guarded 250/500/750 bands; that bookkeeping is not a new RNG001 cursor-parity promise. TheCity then constructs its map and empty starting node. |

Locators: `ProceedButton.goToTreasureRoom/goToNextDungeon`,
`TreasureRoomBoss.getNextDungeonName`, `CardCrawlGame.update/getDungeon`,
`AbstractDungeon` new-act constructor / `dungeonTransitionSetup`, `TheCity`
constructor. These are **S**, not execution of Act 2 or closure of its map laws.

**Required native boundary:** +1 only after the complete Act-1 handoff's required
nested work and handoff mutations resolve; -1 only on actual death; otherwise 0.
Do not award +1 at boss lethal, the boss selection's close, a room-complete flag,
or the first fade flag. **P:** expose an explicit owned successor-ready result
distinct from an intermediate completion marker. Whether that result contains a
generated Act-2 map or only the precisely defined entry state is a native-schema/
whole-spec decision, not permission to omit heal/reset/child obligations. Global
terminal arbitration stays execution-owned. A queued effect is not guaranteed to
run; a committed mutation is not rolled back on fault. No universal return,
forced drain, retry, rollback or administrative defeat is introduced.

## 9. New discriminating acceptance obligations

These are future native cases, **not tests run here**. Each must assert exact
resources, consumed/remaining candidates, public reveal state, continuation owner,
effect status and relevant RNG/history change, not just a terminal label.

| ID | Fixture / discriminator |
|---|---|
| C01 | Same card offer: Skip then reopen preserves row and identities; Take consumes once before deferred insertion; Bowl consumes with no card insertion. |
| C02 | Neow/Dream Catcher null-item Bowl versus present last card row versus card row among mixed rewards; +2/heal in all legal cases, only associated row removed. |
| C03 | Full HP and damaged HP Bowl: distinguish +2/+2 from maximum-only/full-heal; dying/refused-heal preserves already-applied maximum change. |
| C04 | Fast prevented curse: one charge, entered update, no obtain/deck-change hooks or insertion; no repeat charge on list transfer. |
| C05 | Show prevented curse through the normal desktop caller, fast/ordinary duration and admitted delta at 0 and `0.1f`: one charge, positive remainder, entered card update, no obtain/deck-change hooks or permanent insertion, then removal without accumulation. Preserve one-time seen/poof/bookkeeping; oversized rejected frames do not update it. |
| C06 | Note curse with Omamori/Fish/Darkstone: no Omamori charge; incoming hooks/insertion precede outgoing candidates; selecting retrieved card is legal. |
| C07 | Parasite object removal at maxHP 1, 3 and 4 while living: floor/clamp without fabricated damage/death; string/iterator removals do not inherit object hooks. |
| C08 | Independent removal effects: Wing damage before grid, Shop payment before removal, Bonfire reward before removal, Note removal before saved reference. |
| C09 | Neow single versus double transform, Astrolabe and Transmogrifier: distinguish generation/removal ordering and callback effects at each site. |
| C10 | Shop purge cancelled at browsing/confirmation versus accepted: no payment/service use until selected result; both consumers cannot charge twice. |
| C11 | Mirror/bottle selection in Shop with and without Peek and with nonempty reward-list override: test actual result-consumer order, not presumed universal parent stack. |
| C12 | Paid shop relic opens a grid/reward screen before restock completes: retain payment, stock and exactly-once onEquip; close/reenter does not regenerate shop. |
| C13 | Cage 0/1/2/3 purgeables; Astrolabe 0/1/3/4; Pandora zero/nonzero starters: verify distinct automatic/grid/phase branches without invented fixed-count clamp. |
| C14 | Calling Bell prevented curse followed by all/partial/no relic claims; preserve intermediate discarded card-generation history. |
| C15 | Neow boss swap with nested selection versus boss-treasure acquisition of same relic: different starter eligibility and correct parent. |
| C16 | Match first pick: reveal one, retain five attempts; reject picking that face-up object as a second position. |
| C17 | Match/miss second pick: reveal second before delayed resolution; both consume one attempt only at resolution, fresh-copy obtain only for match. |
| C18 | Fifth resolved pair match/miss: no sixth attempt; game-done→cleanup→Complete→map, without declaring queued obtain already inserted. |
| C19 | Pair-wait exact zero and cleanup's later exact-zero landing: record source conditions and selected native timing policy, no accidental false parity. Normal fifth-pair next-Play cleanup entry remains positive under the desktop bound; nonpositive entry is not a required normal-path outcome. |
| C20 | Boards differing only in unrevealed identities/order have identical public rows/candidates until legitimate reveal; known faces remain remembered after miss. |
| C21 | Empty Living Wall, Bonfire and Cleric: their evidenced result/Continue paths differ from committed empty Purifier/Transmogrifier/Wing grids and Note with an unremovable incoming card. Supply reachability witness/exclusion, not a fallback. |
| C22 | No upgradeables at Smith, Upgrade Shrine, Forge and Shining Light; exhausted Girya; Dripper+Hammer with/without Ruby: retain initial-filter/Recall-append/final-scan order. |
| C23 | Ordinary zero-card offer and empty Dream Catcher offer: no empty card row/screen; other rewards and legal leaving survive. |
| C24 | Full-belt potion reward, Sozu reward and Sozu shop purchase: respectively retained offer, consumed-without-obtain, refused-without-spend. |
| C25 | Depleted common/shop with two distinct promoted candidates, normal rejection and depleted boss: preserve draw side, consumed history and exact fallback. |
| C26 | Event-combat rewards and Lab Proceed versus Woman in Blue's ordinary unlabelled `open()`/Proceed path: direct-map exception versus event return; no illicit elite key/Prayer Wheel reward or substituted labelled Cancel contract. |
| C27 | Chest linked Sapphire versus extra relic and independent Emerald; claim/Recall before key effect versus after: only resolved flag changes. |
| C28 | Boss rewards/chest/child take or skip before successor transition: +1 only after required effects and heal/reset boundary; unclaimed legal rewards need not be forced. |
| C29 | Legal death: actual -1, no fabricated grants or +1; administrative fault: not a game defeat or +1. In each multi-stage route retain committed mutations without rollback. Note persistence/detached output assertions depend on the separately reviewed, human-unapproved profile contract; this row does not impose success-only output or suppress a death-committed Note result. |

## 10. Verification performed and exact residual interfaces

This lane performed 34 successful static tool commands over 21 unique class
entries (21 disassemblies, 13 focused reconstructions), inspected the relevant
retained source members, and matched all 914 artifacts in the frozen acquisition
manifest. Review correction added 9 successful static commands over 5 exact class
entries (5 disassemblies, 4 focused reconstructions), closing the desktop caller's
delta bound; one entry overlaps the initial set. The private standard-library
evidence helper had seven expected unimplemented red failures, then seven passing
tests, including duplicate/missing/
unsafe ZIP entries, contaminated JVM environment, reused/escaping output and
wrong hash/length negative controls. An incorrect NeowRoom package lookup failed
closed and was preserved before corrected exact-entry inspection. Five author
prose-regression checks failed on the prior text then passed on the corrections;
these test wording, not source/runtime behavior. No gameplay,
product build, original-runtime test, benchmark or distribution experiment ran.

The remaining interfaces are bounded, not a claim that all earlier unknowns
remain untraced:

1. **Execution:** complete callback/terminal arbitration across other families,
   including Peek-dependent result consumers and effects admitted before a
   transition. Section 2 supplies actual local dispatch order, not a global proof.
2. **Profile/time:** an explicit, human-unapproved native elapsed-time abstraction
   constrained by the closed normal desktop prevention path, and treatment of
   Match's strict timer boundaries; Note staged/committed detached output and
   death/fault/reset/replay policy. Review of a profile proposal is not human
   approval. No original timing frequencies were measured; oversized-delta Show
   acquisition is not an unresolved normal-desktop outcome.
3. **Exceptional reachability:** legal initialized-history witness or exclusion
   for committed empty mandatory grids, zero-Cage phase and stale/mistyped reward
   records. Local behavior is specified; natural reachability is not invented.
4. **Shuffle/generation:** board/face/relic-pool joint laws and initialized
   event-list exhaustion proof. No uniform-permutation or zero-probability claim
   follows from this continuation trace.
5. **Native handoff schema:** precise successor-ready payload and generation
   boundary, incorporating the evidenced transition heal/reset and all resolved
   children. Whole-spec human approval remains required.

Source, reconstructed code, helpers, binaries, raw logs and detailed manifests
remain private. Only independently authored factual/design summaries are here.
Independent exact-head recheck and main readback are pending; the Bead remains in progress.
