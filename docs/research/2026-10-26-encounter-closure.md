# Current encounters: bounded static findings

The initial three named class entries were inspected for this task on
2026-10-26, followed by the explicitly expanded A0 closure on 2026-10-27, using existing pinned JDK21.0.12.1+1/javap and CFR0.152 tools. The
owned archive SHA-256 is
`cfad868ac8d65a88e71a0bf096fb09f78811e553effe0787c5309a655e081673`.
Its upstream build/channel remains unverified. These are independently worded
static facts about those bytes, not original-runtime validation or PCv2.3
provenance. No game classes or initializers were executed. Extracted entries,
tool receipts and raw output remain private in independently retained evidence;
no source, bytecode or game asset is published.

Under `com.megacrit.cardcrawl`, the inspected members are:

- `monsters.exordium.SlaverBlue`: constructor, `getMove`, `takeTurn`.
- `monsters.exordium.FungiBeast`: constructor, `usePreBattleAction`, `getMove`,
  `takeTurn`.
- `powers.SporeCloudPower`: constructor and `onDeath`.

SlaverBlue entry SHA-256:
`bf3fd9bb150b729d67d8c36caa44c8789a396d3f8a9ac9c6887dc0d6d381ceab`.
FungiBeast entry SHA-256:
`01d84978223e6bbae731d464a72817c6bd4b28c3bb295b609fa0fe299dae0c5c`.
SporeCloudPower entry SHA-256:
`3dfa7432a7838263ef29edf4572417a1f5346109d857b9d4a014e12b4824f5f7`.

Blue Slaver has HP46–50 below A7 and48–52 from A7. Stab/Rake bases are12/7
below A2 and13/8 from A2. Integer proposals0–39 request Rake;40–99 request
Stab. A third consecutive Stab redirects to Rake. Below A17, a third Rake
redirects to Stab; from A17, any consecutive Rake redirects to Stab. Rake deals
its attack before requesting Weak1 below A17 or Weak2 thereafter. Each executed
turn then requests its next move. The existing native actor identifier
`slaver_blue` and intents `slaver_stab`/`slaver_rake` are preserved.

Fungi Beast has HP22–28 below A7 and24–28 thereafter, Bite base6 and prebattle
Spore Cloud2. Grow requests Strength3 below A2,4 from A2 and5 from A17.
Proposals0–59 request Bite, but after two selected Bites request Grow instead.
Proposals60–99 request Grow, but after Grow request Bite instead. These are
conditional redirects, not a renormalized pool. Each executed turn requests its
next move; actors retain separate streams and histories under project RNG-001.

Spore Cloud's death callback returns when the room is battle-ending. Otherwise
it requests player Vulnerable equal to its amount at the top of the action queue,
with a null source. The retained [Act1 death-order finding](2026-10-19-act1-content-enemies.md)
establishes that base death marks dying before the group check: a basically dead
final group suppresses Cloud; a nonfinal death can affect the next attack,
including an enemy-turn Thorns kill. Artifact uses the ordinary power-application
owner; a null source does not receive monster-source duration grace.

These facts extend the qualified [enemy reference](../../data/enemies/README.md)
for a concrete native implementation. Hash identity, static ordering, native
regressions and original-runtime validation remain separate evidence. Broader
callback combinations and standard encounter-pool completion are not established
by these three lookups. See the [current inventory](../encounter-coverage.md).

## Remaining current A0 formations and actors

The expanded task inspected named `MonsterHelper.getEncounter` factories and
Gang/Thugs/Wildlife helpers, Exordium and City encounter lists, the MonsterGroup
constructors and Sentry's constructor/prebattle/getMove. Factory array order is
stored group order. Gang removes four successive uniform member-index draws from
Mad×2, Sneaky×2, Fat×2, Shield×1, Wizard×1; sampled order is slot order. Thugs uses
weak wildlife on the left and strong humanoid on the right. Wildlife uses fair
Fungi/Jaw on the left and weak wildlife on the right. Weak wildlife is a uniform
choice among Louse, Spike(M), Acid(M), with fair Louse subtype. Strong humanoid
is a uniform choice among Cultist, Slaver, Looter, with fair Slaver subtype.
Unused candidate constructor calls do not establish original seed parity.

The newly covered fixed Act2 pairs are Looter/Mugger, Sentry/Spheric Guardian,
Cultist/Chosen and Shelled Parasite/Fungi, in that order. The mixed Sentry is
stored at index0, opens Bolt, retains its Elite actor HP38–42 and Artifact1;
room classification remains ordinary. Its first-move decision uses stored group
index parity, rather than its render coordinate. Existing three-Sentry order
continues Bolt/Beam/Bolt. Singleton and pair setup keys/admission are listed in
the [inventory](../encounter-coverage.md).

`monsters.exordium.Looter` and `monsters.city.Mugger` constructors, getMove,
takeTurn and die were checked, together with their four named ledger actions,
`actions.common.DamageAction` and `GainGoldAction` members. At A0, Looter has
HP44–48, Mug10, Lunge12 and Smoke Block6; Mugger has HP48–52, Mug10, Lunge16 and
Smoke Block11. Each has theft amount15 and its own stolen-gold ledger. The first
two moves are Mug. The second Mug's takeTurn chooses Smoke or Lunge fairly before
its queued theft work; Lunge continues to Smoke, then Escape. The native branch
latch preserves that already-owned draw across a subsequent fault or Thorns
kill, without re-rolling at the later selection checkpoint.

A ledger action records the capped amount from current currency. The following
gold-bearing DamageAction independently debits capped current currency before
its later HP hit. Block does not prevent that debit. Actual thief death earns
its ledger as a stolen-gold reward; claiming uses the ordinary gold-gain owner.
Escape marks the room mugged and leaves through the escape lifecycle, earning
no death receipt. The existing [Act1 research](2026-10-19-act1-content-enemies.md)
retains its separate all-retained-escaped ordinary-gold rule and source limits.
No native timer or original callback-interposition parity is inferred.

`monsters.city.ShelledParasite`, `SnakePlant`, `powers.PlatedArmorPower`,
`MalleablePower`, `actions.unique.VampireDamageAction.update`, and the narrowly
required AbstractMonster history and AbstractPlayer damage members were checked.
Parasite A0 HP is68–72, Double Strike6×2, Fell18 followed by Frail2, Life Suck10.
It starts with Plated Armor14 and Block14. Its first selection is a fair Double
Strike/Life Suck choice. Later tickets below20 propose Fell, but after Fell redraw
uniformly from20–99. Tickets20–59 propose Double Strike and60–99 Life Suck;
a third repeated attack redirects to the other. Removing its final armor queues
the ARMOR_BREAK transition, which publishes Stun. The Stun turn sets Fell before
the following RollMove; shared selected-move history therefore excludes another
immediate Fell proposal. This is separate from an immediate armor-decrement
mutation of intent.

Plated Armor reduces once for positive external owner-attributed NORMAL HP damage;
fully blocked, null/self source, HP_LOSS and THORNS paths are excluded. Its
pre-end-turn-card callback captures current Block amount when queued. Life Suck
uses the player's lastDamageTaken, established as the smaller of post-Block
damage and pre-hit HP. Its HealAction goes to the queue top after target damage
callbacks, ahead of previously queued retaliation. Tests distinguish this action
completion from the earlier generic damage callback.

Snake Plant A0 HP is75–79 and its attack is7×3. It starts with Malleable3.
Tickets below65 propose the triple attack, redirecting a third repeated attack
to spores; tickets65–99 propose Frail2/Weak2, redirecting consecutive spores to
attack. Malleable reacts to positive nonlethal NORMAL damage with nonnull source,
queues its current Block amount at bottom and increments synchronously; its
owner end-turn reset restores3. Writhing Mass retains its separately accepted4.
The conflicting broader A17 Snake Plant reference is not generalized here.

The same archive/tool identity and unverified upstream-build qualification apply.
Exact named-entry/member hashes and raw outputs are retained independently; no
private game source is published. These findings establish the finite source
laws used by this A0 implementation, rather than full source/runtime equality,
every possible interaction, event-context closure or other-character/A1–20 Runs.
