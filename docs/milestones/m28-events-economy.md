# M28 events and economy rooms

Status: **candidate in progress (chunks M28a–M28d merged, M28e in review; six chunks); M28 is not accepted.** M28 is delivered in
merge chunks M28a–M28f, each frozen, independently reviewed and merged alone. This page states
what the merged chunks provide and the interim limits that remain until M28f. It is a candidate
profile, not a public profile: M28 adds no spire-env, Gym or Python profile, descriptor or
fingerprint.

## What M28 is

M28 adds the standard A0 Run, `RoomScenario::StandardA0`: any of the four characters at A0, with
full or short Neow, complete card and item admission, the generated act maps and bosses, and the
standard economy rooms and events. It reuses the shared Run, room, card (M26) and item (M27)
owners. Until the roadmap completes there is no backward-compatibility promise for earlier
profiles; each chunk still runs the frozen-replay harness and records any difference for
information.

`StandardA0` is reachable only from core crate tests and the default-off `m28-diagnostics`
feature until M28f completes every room. A build without the feature returns
`RunError::Unsupported("StandardA0 is diagnostics-only until M28f")`.

## M28a: seams, Neow and the standard shop

- **Creation and seams.** `StandardA0` creation for the four characters at A0, the seven appended
  M28 RNG scopes, the card-rarity offset `b` (start 5; omitted from text when absent), the shared
  max-HP loss owner, starter Strike/Defend tags for every character (Pandora's Box reads them),
  the shared card-rarity classifier (read-only in M28a) and the committed Neow selector
  (`RoomSelectionOperation::NeowSelect`).
- **Neow.** The short pair unchanged and the full four-category blessing: construction masses,
  the drawback exclusions, every payload, committed remove/upgrade/transform selectors, skippable
  card offers, and the boss swap with its equip continuation.
- **Shop.** One standard stock producer: five typed class cards, two colourless cards, three
  relics, three potions and one removal; the binary32 price and jitter laws, the colored sale slot,
  The Courier and Membership Card passes, the removal price precedence, purchases and Courier
  restocks.
- **Shared fixes (disclosed).** Every character's basic Strike and Defend now carry the
  STARTER_STRIKE / STARTER_DEFEND tags (only the Defect's did), and Pandora's Box reads the tags
  instead of a copied list. This also changes the opt-in card metadata descriptor: the Ironclad,
  Silent and Watcher basic Strike and Defend rows gain those tags, so the descriptor fingerprint
  moved from `3e922792…` to `cb753725…`; the Rust and Python pins were updated. A Complete shop
  purge discounted by Smiling Mask, The Courier or Membership Card is now purchasable at its
  displayed price (it was rejected before). Every shop's purge, not only the standard shop's, now
  excludes Necronomicurse, Curse of the Bell, Ascender's Bane and bottled cards, and a purge that
  cannot remove its card spends nothing (before, the M27 Act 4 shop offered them and spent gold
  first). A Complete shop's claimed removal row shows the source's next removal price, for example
  40 at base 100 with The Courier and Membership Card. The Courier price pass now always precedes Membership
  Card's, whatever the acquisition order. A standard Run's Neow selections and Neow relic equip
  choices open before the first map move. The M27 Courier restock is one parameterised owner; the
  Act 4 shop keeps its exact arguments.

## M28b: rooms, act transitions, keys and Act 4

- **World entry.** A standard map move commits traversal, floor and history through `world`, then
  runs the room producer for the resolved kind: monster, elite (burning elite buff) and boss
  combats through the shared entry gate, the standard shop, campfire and treasure. Question rooms
  use the accepted ramps plus Tiny Chest's fourth roll and Juzu Bracelet (M27's unknown-room law).
  A boss room fights the act's map boss.
- **Rewards.** One standard reward builder: gold by room kind, Golden Idol, stolen gold, the elite
  relic (and Black Star's), the unlinked Emerald row on a burning elite, M27's potion appearance,
  and card offers with the live rarity offset (rare reset, common decrement, boss rare, in-offer
  duplicate retry), M27's card shape (Question Card, Busted Crown, Prayer Wheel, N'loth's Gift),
  the act's automatic upgrade chance and egg previews. A Smoke Bomb escape keeps the room drops'
  draws and the potion roll and offset, and shows only Proceed.
- **Treasure.** Chest size and the joint tier/gold draw, binary32 gold endpoints, M27's open hooks
  (Matryoshka, Cursed Key, N'loth's Hungry Face), the base relic linked to Sapphire, the separate
  after-open pass; the boss chest offers three Boss-tier relics.
- **Campfire, transitions, keys and Act 4.** Rest, Smith, M27's relic options, then Recall when
  Act 4 is enabled and Ruby is not owned. Act 1 and 2 boss victories lead to the boss chest and the
  next act (fresh map and ramps, boss from live history, full heal). The Act 3 boss leads to the
  accepted ending, and with all three keys to Act 4, whose campfire, shop and elite rewards use the
  standard producers; the Heart route is the accepted one.
- **Shared fixes (disclosed).** Complete-admission shops (the standard shop and M27's Act 4 shop)
  preview owned eggs on their card rows, re-preview after a relic purchase and on Courier card
  restocks. A complete-admission campfire with no usable option (for example Coffee Dripper,
  Fusion Hammer and Ruby owned) now completes on entry instead of publishing no action; this
  includes M27's Act 4 campfire. Under Sozu a complete-admission shop no longer advertises its
  potion rows, whose purchase it rejects. Neow's three-potion blessing now generates and discards the
  ordinary card row, keeping its rarity-offset writes.
- **Automatic reward-card upgrades (source-checked).** The chance is 0 in Act 1, 0.25 in Act 2 and
  0.5 in Acts 3 and 4 at A0. Every Common and Uncommon reward card consumes one upgrade draw, even
  at chance 0 and before the upgradeability check; a Rare draws nothing and is never automatically
  upgraded. Eggs preview only cards the roll did not upgrade. A PM static source check of the owned
  original game confirmed these rules (retained M28 execution evidence,
  `m28b/source-check-act4-upgrade-v1.md`).

## M28c: the event core

- **Identities and pools.** Every event has an `EventId` (45 appended after the accepted seven) and
  a catalog row with its act set and pool: ordinary lists of 11, 13 and 7 events for Acts 1–3, six
  shrines in every act, and 14 one-time events. A standard Run builds its event lists at creation
  and rebuilds the ordinary and shrine lists at each act start; one-time events are never
  replenished, and a selected event leaves its list even when it is skipped.
- **Selection and eligibility.** The shrine branch is taken on a draw below 0.25. Otherwise an
  eligible ordinary event is picked uniformly, and an empty eligible list delegates to the shrine
  branch: the remaining shrines plus the eligible one-time events. Where the source defines no
  outcome, an event room with no eligible event becomes a monster room of the act (disclosed
  decision E6). Each eligibility rule is tested at its boundary: floor, gold, HP, relic count,
  act, curses, Golden Idol and Colosseum's map row.
- **Frame and executor.** A body returns a list of outcomes, and one executor applies them through
  the shared owners after checking every precondition. A rejected choice leaves the Run unchanged,
  and its option is shown disabled. Card selections (remove, upgrade, transform, duplicate)
  return to the event, and so do event combats. An event combat with rewards opens the
  EventRoom reward screen: the event's gold and relics, the ordinary card offer and the EventRoom
  potion roll, with no combat gold. The six accepted events (Upgrade Shrine, Purifier, Ancient
  Writing, Joust, Falling and Tomb of Lord Red Mask) keep their own handlers. The sample bodies
  are Big Fish, Golden Shrine, The Addict and Mind Bloom.
- **Reference record.** `data/events/` holds the 52 event records, the wiki captures and
  `build_dataset.py --check` (see [data/events](../../data/events/README.md)).
- **Shared fixes (disclosed).** Purifier's and Ancient Writing's removals offer only purgeable
  cards. Falling never takes a bottled card, and Simplicity upgrades every card tagged starter
  Strike or Defend, for every character. A fight is a boss fight when it has a BOSS-type
  monster, whatever the room, so Pantograph and Slaver's Collar work in Mind Bloom's boss fight.
  Out-of-combat damage and HP loss (events, Neow) follow one law. First Torii turns owned damage
  of 2–5 that is not HP_LOSS into 1 (Shining Light's, when max HP is 8–27; see M28d). Then
  Tungsten Rod takes 1. Below 1 HP a Fairy in a Bottle rescues first, otherwise an unused Lizard
  Tail; Mark of the Bloom blocks both. Buffer is a combat power and never applies. An event
  combat is set up with its encounter's own act. Its card reward gets every card-reward relic
  (Question Card, Busted Crown, Singing Bowl, N'loth's Gift); Prayer Wheel stays MonsterRoom-only.
  With no purgeable card, Elegance skips its grid and shows its leave screen; Purifier removes
  nothing and completes (deviation SOFTLOCK-001: the original strands the player).
- **Escaping an event fight.** A Smoke Bomb escape from an event fight with a reward screen still
  builds the EventRoom rewards and rolls the potion, then shows only Proceed, to the map. An escape
  from a fight whose rewards are disabled reopens the event at its post-fight stage. Boss-type
  fights block Smoke Bomb.

## M28d: events batch 1

- **Bodies.** The ten remaining Act 1 ordinary events (Dead Adventurer, Golden Idol, Mushrooms,
  Living Wall, Scrap Ooze, Shining Light, The Cleric, The Ssssserpent, Wing Statue, World of Goop),
  three shrines (Match and Keep, Transmogrifier, Wheel of Change) and six one-time events (Accursed
  Blacksmith, Bonfire Spirits, Face Trader, Fountain of Cleansing, Lab, Note For Yourself). Values
  come from the Act 1 event research; amounts read from max HP and the class pools are tested for
  each character.
- **Event fights.** Mushrooms fights `ThreeFungiBeasts`, three independent Fungi Beasts. Dead
  Adventurer fights three Sentries, Gremlin Nob or `LagavulinEvent`: an awake Lagavulin with no
  Block or Metallicize that opens with Siphon Soul. The fight sets the source's `eliteTrigger`, so
  elite-only combat effects apply, but the room stays an EventRoom: there is no elite relic, key or
  Prayer Wheel offer. Each fight's listed gold range is drawn when it starts and shown in one gold
  row with any fixed gold. Listed relics are ordinary reward offers.
- **Executor.** New outcomes cover a draw at choice time (Scrap Ooze, Dead Adventurer, Wheel of
  Change), per-event progress, a non-combat reward screen (Lab's potions, Wheel of Change's relic),
  Note For Yourself's direct obtain and store, and a chosen card whose effect the event decides
  (Bonfire Spirits). Wing Statue, World of Goop and Shining Light deal damage owned by the player,
  as the source passes it; Golden Idol, Scrap Ooze and Face Trader damage has no owner. Torii
  turns Shining Light's 2–5 damage (max HP 8–27) into 1. The precondition check is now exact. Earlier
  card outcomes change the deck before a later selection is checked, and an earlier selection
  takes from a later one only the cards both must share. A purchase of a relic already owned is
  rejected like an obtain.
- **Screens.** Forced one-button pages follow D1 (below). Match and Keep's rows show each revealed
  position's card as a memory aid; unmatched pairs still flip back in game state (D2). Its fixed
  starter card is Bash, Neutralize, Zap or Eruption (disclosed decision E11). Wheel of Change's
  reveal is its own step: Spin draws, commits any gold and shows the result; a forced button applies it.
- **Empty grids.** Transmogrifier and Wing Statue's Pray follow SOFTLOCK-001 when no card is
  purgeable: nothing is transformed or removed and the event continues. The original strands the
  player.
- **Forced pages (D1).** A forced one-button page may be collapsed into the preceding choice only when
  no other legal action taken on that page (out-of-combat potion use included; the Run offers it on
  event views) could lead to a different final state or RNG order. Otherwise it stays a step, whether
  or not the press changes state or draws. Collapsed: Ssssserpent's confirmation; the intro pages of
  Bonfire Spirits, Face Trader, Note For Yourself and Lab; Mushrooms' Continue and Dead Adventurer's
  ambush Fight (their gold and reward draws are keyed event draws that potions never consume, and the
  next state is combat); the Wheel's reward screen's return through the event's result page (Lab's
  goes to the map in the source); Match and Keep's Play page (the board is built with the event); and
  Wing Statue's Continue after Pray (the removal draws nothing). The first card of a Match and Keep
  attempt stays face up, as a disabled row, while the second is chosen.
- **Draw timing (D3).** Values the source draws when the event is constructed are drawn when it is
  built (World of Goop's loss, Dead Adventurer's enemy and token order); values drawn on a choice
  are drawn at the choice (Scrap Ooze's, Dead Adventurer's and Wheel of Change's results, Wing
  Statue's Destroy gold and Face Trader's traded face), and Dead Adventurer's and Mushrooms' fight
  gold when the fight starts. The order follows the per-event research. The draws are keyed `M28EventBody` draws with
  no seed parity (RNG-001); uniform shuffles stand in for the shuffle measures the research leaves open.
- **Act-dependent amounts.** Wheel of Change's gold is 100, 200 or 300 in Acts 1, 2 and 3 (its
  constructor's `setGold()` keys on the dungeon); it is captured when the event is built. No other
  M28d payout depends on the act; the other act differences are the selector's gates.
- **Ascension 15 and above.** StandardA0 runs only A0, but the bodies already carry the source's
  A15 band: Golden Shrine's Pray 50; Match and Keep's colourless pair becomes a second curse pair
  (still five attempts); Wheel of Change's loss T(.15M); Face Trader's Touch 50; Lab's two potions;
  Dead Adventurer's fight chances 35/60/85; Golden Idol's T(.35M) damage or max(1, T(.10M)) max HP;
  Scrap Ooze's first Reach 5; Shining Light's J(.30M); The Cleric's Purify 75; Ssssserpent's 150;
  World of Goop's loss 35–75. Note For Yourself is not offered at A15+. Transforms stay unupgraded
  at every ascension.

## M28e: events batch 2

- **Bodies.** Eleven Act 2 ordinary events (Beggar, Colosseum, Cursed Tome, Drug Dealer, Forgotten
  Altar, Ghosts, Masked Bandits, Nest, The Library, The Mausoleum, Vampires) and two shrines (We Meet
  Again, The Woman in Blue). Source owners are `Beggar`, `DrugDealer` and `Ghosts`; values come from
  the M28e source checks, and amounts read from max HP and the class pools are tested for each
  character.
- **Event fights.** Colosseum fights `ColosseumSlavers` (Slaver Blue, then Slaver Red; no rewards),
  then offers Escape or Fight Nobs; `ColosseumNobs` is Taskmaster, then Gremlin Nob, with a Rare and
  an Uncommon relic row, 100 gold and the elite trigger. Masked Bandits fights `MaskedBandits`:
  Pointy, Romeo and Bear in source slot order, with 25–35 gold drawn at the Fight choice and a fixed
  Red Mask row (Circlet if owned). Pay gives up all gold. The three bandits are new enemies, at A0 only.
- **Shared rules.** Event fights whose reward rows draw a relic tier now take those relics from the
  pools when the fight starts, as the source adds them before entering combat. This applies to
  every event fight, Mind Bloom and Dead Adventurer included. A rewards-disabled event fight
  (Colosseum's first) still takes the EventRoom potion roll after victory or escape. An event fight
  whose encounter has no enemy facts at the Run's ascension is rejected with a typed `Unsupported`
  before the choice is accepted. Removing or transforming a bottled card clears its bottle.
- **Executor.** New outcomes lose a held potion (We Meet Again), remove every tagged card in one step
  (Vampires' Strikes), replace a relic in its slot with no equip effect (Forgotten Altar's Golden
  Idol to Bloody Idol) and build The Library's 20-card board. The board's cards are distinct, drawn
  at the Read choice with the EventRoom rarity thresholds from the active character's pools, with or
  without Prismatic Shard (the source's plain `getCard`, not the reward builder). Cursed
  Tome's losses, and The Woman in Blue's A15 Leave, are HP loss with no owner; Nest's Stay in Line
  and Forgotten Altar's Sacrifice are unowned damage. Drug Dealer's transform grid keeps bottled
  cards; every other remove, transform or choose grid excludes them, and upgrade grids keep them.
- **Screens.** Cursed Tome's three Continue pages stay steps (a potion between the HP losses can
  change the result); its book is a skippable relic reward screen. Collapsed under D1: Colosseum's
  intro and Continue pages into one start row, and the Leave-only page after Cursed Tome's and The
  Woman in Blue's reward screens, which return to the map. Colosseum's Nobs rewards list the gold
  first; the source lists Rare, Uncommon, Gold (display order only). The Library's grid has no
  cancel; the source's cancel path is hidden. Vampires offers Give Vial only while Blood Vial is
  owned. We Meet Again's rows show what each button gives (D2): the preselected potion, the gold
  amount as the row price, and the card, whose candidate names its deck instance.
- **Potion lock.** From the start of We Meet Again until the next room is entered, no potion can be
  used or discarded, on the map after Leave included. The source tests the current room's event,
  and the room stays the event room after the map opens.
- **Empty grids.** Beggar follows SOFTLOCK-001: with no purgeable card, Pay takes the 75 gold and
  the event completes without a removal. Drug Dealer's Transform is disabled with fewer than two
  purgeable cards.
- **Draw timing (D3).** Amounts the source captures at construction are captured when the event is
  built: Cursed Tome's Take loss, Forgotten Altar's `MathUtils.round` cost (this corrects the
  truncation E15 assumed), Ghosts' and Vampires' max-HP loss, The Library's Sleep heal, and We Meet
  Again's potion, gold and card. Masked Bandits' gold, The Library's board, Mausoleum's curse roll and
  relic, and Cursed Tome's book are drawn at the choice. The Library's story text uses LibGDX's
  random and draws nothing from the simulator.
- **Ascension 15 and above.** Cursed Tome's Take 15; Forgotten Altar's cost round(.35M); Ghosts'
  three Apparitions; Nest's Smash and Grab 50; The Library's Sleep round(.20M); The Mausoleum always
  adds Writhe (the roll is still drawn); The Woman in Blue's Leave costs ceil(.05M) HP. The bandits,
  like the Colosseum formations, have A0 facts only, so both M28e fights are the typed `Unsupported`
  above A0. The Colosseum, whose only opening row is its first fight, is rejected at the Move; Masked
  Bandits still offers Pay; the bandits' A2, A7 and A17 values are recorded for M29–M33. Each bandit's opening
  `aiRng.random(99)` is not modelled (their moves are fixed). Their intents are event intents outside
  the frozen policy vocabulary.

## Interim limits (shrink with each chunk)

- The last nine event bodies (Moai Head, Mysterious Sphere, Sensory Stone, Winding Halls, Secret
  Portal, The Designer, Duplicator, Knowing Skull and N'loth) land in M28f. When one is selected,
  the move returns `RunError::Unsupported("M28 event body lands in M28f")` and leaves the Run
  unchanged, event lists included. Secret Portal's boss entry is likewise typed as landing in M28f.

- The native fuzz harness drives only the finite profiles; each chunk compares its 120 s campaign
  with its base.

## Evidence and checks

Each chunk records its spec sheet (literals copied from the cited research, every entry reviewed),
its suites (`cargo test --workspace --locked --offline` and
`cargo test -p spire-core --locked --offline --features m28-diagnostics`), its informational
frozen-replay comparison and its fuzz gap in the retained M28 execution evidence. Source research:
[Neow](../research/2026-10-19-act1-content-event-neow.md) and
[economy](../research/2026-10-19-act1-content-economy-potions.md). Original-game parity across
characters and acts beyond the research's Ironclad Act 1 scope is a design contract, not
source-verified evidence.
