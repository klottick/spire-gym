# M28 events and economy rooms

Status: **candidate in progress (chunks M28a and M28b merged, M28c in review; six chunks); M28 is not accepted.** M28 is delivered in
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
  of 2–5 that is not HP_LOSS into 1; event damage has no owner, so Torii never applies to it. Then
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

## Interim limits (shrink with each chunk)

- The other event bodies land in M28d–M28f. When the selected event has no body yet, the move
  returns `RunError::Unsupported("M28 event body lands in M28d")` (or `M28e`/`M28f`) and leaves
  the Run unchanged, event lists included. Secret Portal's boss entry is likewise typed as
  landing in M28f.

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
