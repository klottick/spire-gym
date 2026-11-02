# M28 events and economy rooms

Status: **candidate in progress (chunk M28a of six); M28 is not accepted.** M28 is delivered in
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

## Interim limits (shrink with each chunk)

- Standard map rooms land in M28b: a `Move` on a standard Run returns
  `RunError::Unsupported("M28 standard map rooms land in M28b")`.
- The standard shop is reachable only through the `m28-diagnostics` shop builder until M28b.
- Neow's three-potion reward does not yet generate and discard the ordinary card row; its
  card-rarity offset writes land with M28b's reward-card helper.
- Events land in M28c–M28f.
- Shop card rows always show `upgrades: 0`. Molten, Toxic and Frozen Egg upgrade the card when
  it is obtained, so the deck is right, but the shop row does not preview the upgrade. The M27
  Act 4 shop behaves the same way. The preview lands with M28b's shared reward and shop card rows.
- The fuzz `FullRun` lane is not merged; its gap is recorded per chunk until it merges.

## Evidence and checks

Each chunk records its spec sheet (literals copied from the cited research, every entry reviewed),
its suites (`cargo test --workspace --locked --offline` and
`cargo test -p spire-core --locked --offline --features m28-diagnostics`), its informational
frozen-replay comparison and its fuzz gap in the retained M28 execution evidence. Source research:
[Neow](../research/2026-10-19-act1-content-event-neow.md) and
[economy](../research/2026-10-19-act1-content-economy-potions.md). Original-game parity across
characters and acts beyond the research's Ironclad Act 1 scope is a design contract, not
source-verified evidence.
