# Reachable Defect content: paired normative records

Date: 2026-10-19. Bead: `spire-gli.2.3.20`.

**Draft for independent review; specification only.** This is a standalone set of
source-backed local rules, not an assertion of complete transitive combat closure.
It supplies four basics, 18 commons, 36 uncommons, 17 rares, two fixed Status
dependencies, and four orb kinds. Section 9 identifies the exact unresolved shared
fields. Future acceptance cases below have **not** been executed as gameplay.

## 1. Scope, evidence, and record semantics

The run remains standard, unmodded, fully unlocked, Act4-enabled **Ironclad A0
Act1**, including legal deaths, skips, and deferred keys. Prismatic Shard can
expose off-color reward cards; A Note For Yourself can retrieve legally stored
cards, including other-character basics. Neither changes the actual character,
imports another character's start, enables its home events, or grants its exclusive
relic/potion pools. Source eligibility is not a uniform sampler over this table.
Stored card identity/upgrade/misc belong to the
[profile contract](2026-10-19-act1-profile-time.md), not a fabricated Defect start.

Evidence is static inspection of owned archive SHA-256
`cfad868ac8d65a88e71a0bf096fb09f78811e553effe0787c5309a655e081673`.
The upstream build/channel is not independently established. Reviewed prior paired
descriptions were used as a cross-check; retained source/bytecode was reused for
the basics, Genetic Algorithm, player/origin/copy helpers, and generation sites.
Missing nonbasic play/upgrade bodies and local orb/actions were inspected with
pinned javap and CFR 0.152. No original class, initializer, launcher, or gameplay
runtime executed. Independent wording below is factual specification, not source
or a port. Source member locators in section 10 identify the evidence boundary.

Table conventions:

- Every cost and keyword cell is **base/upgraded**; `-/-` means neither. `X/X`
  is source cost sentinel -1, not numeric zero. All 75 blue cards have one ordinary
  upgrade; an already-upgraded instance's upgrade method does nothing.
- `A/S/P` are Attack/Skill/Power. Source targets `enemy`, `all`, `self`, `none`
  retain their distinct enum meanings. `all` does not mean that the payload must
  hit every enemy: Rip and Tear and Thunder Strike use automatic random targets.
  Only `enemy` requests a player-selected enemy here. Choices of cards are separate.
- `E` = inherent Exhaust, `I` = Innate, `T` = Ethereal. No inherent Retain appears
  in these blue constructors. Powers leave cycling piles by installation, not
  ordinary exhaustion. Source-imposed exhaust/purge and Corruption are additional.
- Damage/Block are unmodified base parameters, not promised final totals. Ordinary
  card damage uses the shared per-target card calculation; Block uses ordinary card
  Block calculation before the gain action. Separate hits/gains remain separate.
  `then` describes a local request sequence, not a promise to drain every request
  after lethal damage. Later sections specify nontrivial capture times.
- Unless a row says otherwise, upgrade changes only the paired parameter shown;
  type, color (blue), rarity, target, other parameters and keyword flags are unchanged.
  Unmentioned constructor damage/Block/magic/misc fields retain shared defaults;
  they are not extra effects. The class key in backticks disambiguates identities.

## 2. Four distinct basics

These are Note-reachable identities, not aliases of Ironclad Strike/Defend.

| Kind / source class | Type; target | Cost B/U | Keywords B/U | Local payload B/U |
|---|---|---|---|---|
| Strike (Defect), `Strike_Blue` (ID `Strike_B`) | A; enemy | 1/1 | -/- | Deal 6/9. STRIKE and STARTER_STRIKE tags. |
| Defend (Defect), `Defend_Blue` (ID `Defend_B`) | S; self | 1/1 | -/- | Gain 5/8 Block. STARTER_DEFEND tag. |
| Zap, `Zap` | S; self | 1/0 | -/- | Channel one Lightning. |
| Dualcast, `Dualcast` | S; none | 1/0 | -/- | Invoke the front orb once without removal, then invoke it once with removal. Empty front/capacity gives no orb payload. |

Debug-only Strike/Defend substitutions are outside the standard run. Note alone
does not supply slots or Lightning. Shard's equip hook sets permanent slot capacity
to **one only if** the actual character is not Defect and permanent capacity was
zero; it does not channel anything. Combat setup constructs empty slots from that
capacity. Without Shard or another legal capacity effect, Ironclad capacity is zero.

## 3. Common: 18 nonbasic kinds

| Kind / source class | Type; target | Cost B/U | Keywords B/U | Local payload B/U |
|---|---|---|---|---|
| Ball Lightning, `BallLightning` | A; enemy | 1/1 | -/- | Deal 7/10; channel one Lightning. |
| Barrage, `Barrage` | A; enemy | 1/1 | -/- | One 4/6-damage hit per occupied orb when BarrageAction executes; all hits share the captured DamageInfo and selected enemy. |
| Beam Cell, `BeamCell` | A; enemy | 0/0 | -/- | Deal 3/4; request 1/2 Vulnerable on that enemy. |
| Charge Battery, `ConserveBattery` (ID `Conserve Battery`) | S; self | 1/1 | -/- | Gain 7/10 Block; install one EnergizedBlue for next energy recharge. |
| Claw, `Claw` (ID `Gash`) | A; enemy | 0/0 | -/- | Deal initial 3/5; later increase the playing card and Claws in draw/discard/hand by 2 base damage this combat (§6). |
| Cold Snap, `ColdSnap` | A; enemy | 1/1 | -/- | Deal 6/9; channel one Frost. |
| Compile Driver, `CompileDriver` | A; enemy | 1/1 | -/- | Deal 7/10; at its follow-up count distinct nonempty orb IDs and draw one per distinct ID. |
| Coolheaded, `Coolheaded` | S; self | 1/1 | -/- | Channel one Frost; request draw 1/2. |
| Go for the Eyes, `GoForTheEyes` | A; enemy | 0/0 | -/- | Deal 3/4; at follow-up request 1/2 Weak iff the selected enemy is nonnull and its intent base damage is at least zero. |
| Hologram, `Hologram` | S; self | 1/1 | E/- | Gain 3/5 Block; mandatory retrieval of one discard card if available (§7). |
| Leap, `Leap` | S; self | 1/1 | -/- | Gain 9/12 Block. |
| Rebound, `Rebound` | A; enemy | 1/1 | -/- | Deal 9/12; install one Rebound charge with a just-installed guard (§7). |
| Recursion, `Recursion` (ID `Redo`) | S; self | 1/0 | -/- | Capture the actual front orb; evoke/remove it; attempt rechannel of that same object without forcing another eviction (§5). |
| Stack, `Stack` | S; self | 1/1 | -/- | On applyPowers set base Block to discard size + 0/3; use captures the calculated Block, not a later recount. |
| Steam Barrier, `SteamBarrier` (ID `Steam`) | S; self | 0/0 | -/- | Initial base Block 6/8; gain calculated Block, then mutate same-identity combat instances' base Block by -1 (§6). |
| Streamline, `Streamline` | A; enemy | 2/2 | -/- | Deal 15/20; later reduce same-identity combat instances' combat cost by one (§6). |
| Sweeping Beam, `SweepingBeam` | A; all | 1/1 | -/- | Deal 6/9 to all; request draw one. |
| TURBO, `Turbo` (ID `Turbo`) | S; self | 0/0 | -/- | Gain 2/3 energy; create one base Void in discard. |

## 4. Uncommon: 36 nonbasic kinds

| Kind / source class | Type; target | Cost B/U | Keywords B/U | Local payload B/U |
|---|---|---|---|---|
| Aggregate, `Aggregate` | S; self | 1/1 | -/- | At action execution gain floor(draw-pile size / 4) or floor(size / 3) energy. |
| Auto-Shields, `AutoShields` (ID `Auto Shields`) | S; self | 1/1 | -/- | At use, only if current Block equals zero, request 11/15 calculated Block. |
| Blizzard, `Blizzard` | A; all | 1/1 | -/- | At use count successful Frost channels this combat; set base damage to count × 2/3, recalculate per-enemy damage, then request damage-all. |
| Boot Sequence, `BootSequence` | S; self | 0/0 | IE/IE | Gain 10/13 Block. |
| Bullseye, `LockOn` (ID `Lockon`) | A; enemy | 1/1 | -/- | Deal 8/11; request 2/3 Lock-On duration, not multiplier strength; local stacking/expiry are specified below. |
| Capacitor, `Capacitor` | P; self | 1/1 | -/- | Request 2/3 slot increases, each as a single-slot operation capped at ten. No occupied orbs are granted. |
| Chaos, `Chaos` | S; self | 1/1 | -/- | Sample and enqueue 1/2 random orbs at use, separately and with replacement from Dark/Frost/Lightning/Plasma, probability 1/4 each. |
| Chill, `Chill` | S; self | 0/0 | E/IE | At use count enemies for which isDeadOrEscaped is false; enqueue that many Frost channels. |
| Consume, `Consume` | S; self | 2/2 | -/- | Request +2/+3 Focus, then remove one slot. Play does not require a slot; at zero capacity the Focus request still exists. |
| Darkness, `Darkness` | S; self | 1/1 | -/- | Channel one Dark; upgrade additionally invokes the passive hooks of each Dark present when DarkImpulse executes, including the newly channeled one if still present. |
| Defragment, `Defragment` | P; self | 1/1 | -/- | Request +1/+2 Focus. |
| Doom and Gloom, `DoomAndGloom` | A; all | 2/2 | -/- | Deal 10/14 to all; channel one Dark. |
| Double Energy, `DoubleEnergy` | S; self | 1/0 | E/E | At action execution gain energy equal to then-current energy. Own ordinary cost has already been paid. Energy addition cap still applies. |
| Equilibrium, `Equilibrium` (ID `Undo`) | S; self | 2/2 | -/- | Gain 13/16 Block; install one Equilibrium intensity. Its player-end-turn hook sets retain on non-Ethereal hand cards (§7). |
| FTL, `FTL` | A; enemy | 0/0 | -/- | Deal 5/6 via follow-up; draw one iff cardsPlayedThisTurn.size - 1 < 3/4 when FTLAction executes. It inserts draw at top and damage at bottom, so this is not damage-then-draw resolution. |
| Force Field, `ForceField` | S; self | 4/4 | -/- | Gain 12/16 Block; cost evolves with notified Power plays and initializes from combat Power history (§6). |
| Fusion, `Fusion` | S; self | 2/1 | -/- | Channel one Plasma. |
| Genetic Algorithm, `GeneticAlgorithm` | S; self | 1/1 | E/E | Initial misc/base Block 1 in either state; enqueue permanent/same-identity misc growth +2/+3, then gain the already captured calculated Block (§6). |
| Glacier, `Glacier` | S; self | 2/2 | -/- | Gain 7/10 Block; channel two Frost, as separate requests. |
| Heatsinks, `Heatsinks` | P; self | 1/1 | -/- | Install Heatsink intensity 1/2; each notified Power use inserts a draw request of total intensity at top. |
| Hello World, `HelloWorld` | P; self | 1/1 | -/I | Install Hello intensity one; each start-turn hook, unless monsters basically dead, generates that many current-character common cards (§8). |
| Loop, `Loop` | P; self | 1/1 | -/- | Install Loop intensity 1/2; start turn invokes front orb's start and end passive hooks that many times, without eviction. |
| Melter, `Melter` | A; enemy | 1/1 | -/- | Remove the selected enemy's entire Block, then deal 10/14. Removal is not damage. |
| Overclock, `Overclock` (ID `Steam Power`) | S; self | 0/0 | -/- | Request draw 2/3; create one base Burn in discard. |
| Recycle, `Recycle` | S; self | 1/0 | -/- | Exhaust one mandatory selected hand card; energy is based on its selected costForTurn before exhaust (§7), not printed cost. |
| Reinforced Body, `ReinforcedBody` | S; self | X/X | -/- | Separate gains of calculated 7/9 base Block, repeated captured X plus Chemical X's two if held (§6). |
| Reprogram, `Reprogram` | S; none | 1/1 | -/- | Separate requests: -1/-2 Focus, +1/+2 Strength, +1/+2 Dexterity, in that local order. |
| Rip and Tear, `RipAndTear` | A; all | 1/1 | -/- | Two separately resolved random-enemy hits of base 7/9. Each action reselects and recalculates damage for its own target (§5). |
| Scrape, `Scrape` | A; enemy | 1/1 | -/- | Deal 7/10; request draw 4/5 with ScrapeFollowUp, which discards tracked drawn cards unless costForTurn=0 or freeToPlayOnce (§7). |
| Self Repair, `SelfRepair` | P; self | 1/1 | -/- | Install Repair intensity 7/10. At onVictory heal that intensity only if current HP>0. HEALING tag excludes it from the combat Power generator. |
| Skim, `Skim` | S; none | 1/1 | -/- | Request draw 3/4. |
| Static Discharge, `StaticDischarge` | P; self | 1/1 | -/- | Install intensity 1/2; qualifying onAttacked callback enqueues that many Lightning channels (§5). |
| Storm, `Storm` | P; self | 1/1 | -/I | Install Storm intensity one; notified Power use with positive intensity enqueues that many Lightning channels at bottom. |
| Sunder, `Sunder` | A; enemy | 3/3 | -/- | Deal 24/32 through SunderAction; after direct damage request +3 energy iff target isDying or HP<=0; then run its basically-dead pending filter. This is the source predicate, not a universal kill-credit rule. |
| Tempest, `Tempest` | S; self | X/X | E/E | Enqueue captured X + 0/1 + Chemical X's two if held, separate Lightning channels (§6). |
| White Noise, `WhiteNoise` | S; none | 1/0 | E/E | Select a fresh current-character nonhealing Power, set its turn cost to zero, request hand creation (§8). |

## 5. Rare: 17 nonbasic kinds; orb and damage rules

| Kind / source class | Type; target | Cost B/U | Keywords B/U | Local payload B/U |
|---|---|---|---|---|
| All for One, `AllForOne` (ID `All For One`) | A; enemy | 2/2 | -/- | Deal 10/14; scan discard and request retrieval of each card whose combat cost=0 **or** freeToPlayOnce is true (§7). |
| Amplify, `Amplify` | S; self | 1/1 | -/- | Install 1/2 charges: next qualifying nonpurge Power uses each queue one repeat; charge expires at player turn end (§6). |
| Biased Cognition, `BiasedCognition` | P; self | 1/1 | -/- | Request +4/+5 Focus, then one Bias debuff intensity. Each start-turn Bias hook requests negative Focus equal to total Bias intensity. |
| Buffer, `Buffer` | P; self | 2/2 | -/- | Install 1/2 Buffer charges. Its inspected onAttackedToChangeDamage hook returns zero and queues charge reduction only for damage>0; full HP-loss/prevention arbitration remains §9. |
| Core Surge, `CoreSurge` | A; enemy | 1/1 | E/E | Deal 11/15; request one Artifact. |
| Creative AI, `CreativeAI` | P; self | 3/2 | -/- | Install intensity one; start turn generates one fresh nonhealing current-character Power per total intensity, without a zero-cost discount (§8). |
| Echo Form, `EchoForm` | P; self | 3/3 | T/- | Install Echo intensity one. Source turn-position/copy guard, not an unconditional doubling of every play (§6). |
| Electrodynamics, `Electrodynamics` | P; self | 2/2 | -/- | Request Electro (Lightning hits all) then channel 2/3 Lightning. Repeated applications do not multiply Lightning damage. |
| Fission, `Fission` | S; none | 0/0 | E/E | Capture occupied count n at FissionAction execution; base removes orbs without evoke, upgrade requests evoke-all; then gain n energy and draw n (§5.2). |
| Hyperbeam, `Hyperbeam` | A; all | 2/2 | -/- | Deal 26/34 to all; request -3 Focus. |
| Machine Learning, `MachineLearning` | P; self | 1/1 | -/I | Apply Draw intensity one: increases gameHandSize by one, rather than drawing immediately or raising the ten-card hand capacity. |
| Meteor Strike, `MeteorStrike` | A; enemy | 5/5 | -/- | Deal 24/30; channel three Plasma. STRIKE tag. |
| Multi-Cast, `MultiCast` (ID `Multi-Cast`) | S; none | X/X | -/- | If a front orb exists, invoke it captured X + 0/1 + Chemical X's two times; only final invocation removes it, only when total>0 (§6). |
| Rainbow, `Rainbow` | S; self | 2/2 | E/- | Channel one Lightning, one Frost, one Dark, in that request order. |
| Reboot, `Reboot` | S; self | 0/0 | E/E | Request ShuffleAll, draw-pile shuffle, then draw 4/6. Participating hand/discard, random moves and asynchronous transfer limits are §7. |
| Seek, `Seek` | S; none | 0/0 | E/E | Mandatory selection/movement of 1/2 draw-pile cards if available, not actual draw (§7). |
| Thunder Strike, `ThunderStrike` | A; all | 3/3 | -/- | At use count successful Lightning channels this combat; enqueue that many random-enemy hits of base 7/9, each reselecting and recalculating. STRIKE tag. |

### 5.1 Ordered slots, channel, passive, evoke

The front is **logical index zero**, not a tooltip's ambiguous screen direction.
Empty capacity, occupied sequence, Focus, per-orb Dark payload, and successful
channel history are distinct state. No Defect-only starting Lightning is implicit.

| Condition / operation | Exact local rule |
|---|---|
| Channel with capacity zero | Return without installation, onChannel notification, or channel-history append. Chaos has already sampled its orb(s) at use, so a failed channel does not erase those selection requests. |
| Channel with an empty slot | Fill the first empty index; notify player powers' onChannel; append the actual orb object to combat and turn channel history; apply Focus. |
| Channel at full positive capacity | Insert front eviction and retry of the same channel request at top (animation, evoke, channel in effective local order). Successful installation/history occurs on retry, not twice. Global interruptions remain §9. |
| Increase capacity through Capacitor | Execute one-slot increases; each returns unchanged at capacity=10. Repeated Capacitor cannot exceed ten on this route. Empty slots are appended. |
| Consume at positive capacity | Remove the last slot/object, whether empty or occupied, without invoking its evoke payload; decrease capacity by one. At zero, slot removal is a no-op. |
| Evoke/remove front | Invoke front's onEvoke, shift remaining slots toward zero, append an empty slot. Empty front or empty slot vector is a no-op. |
| Evoke without removal | Invoke the currently front object's onEvoke, leave it in place. No passive growth is implied. |
| Recursion | RedoAction captures front object at execution. If occupied, queues evoke/remove followed by ChannelAction(object,false). This retries only if an empty slot exists; it does not manufacture capacity or evict a second orb. **Dark accumulated damage survives**, because this is the same object, not makeCopy. |
| Loop / Darkness+ | Loop invokes both start/end hooks on front for each intensity; Darkness+ does likewise once on each present Dark. These are passive invocations, not evoke, channel, or history increments. Empty orb hooks do nothing. Defect-exclusive Cables is not imported into Ironclad eligibility. |

Let F be current Focus (zero if absent), and D a particular Dark's stored evoke
amount. The following amounts are captured into their payload requests at each
hook invocation. Focus changes recalculate Lightning/Frost amounts and Dark's
**future growth**, not Dark's already accumulated D.

| Orb | Initial state; natural passive | Evoke |
|---|---|---|
| Lightning | Passive max(0,3+F) damage at end turn. | max(0,8+F) damage. Each request selects a random eligible enemy; Electro changes both passive and evoke to all-enemy orb damage. |
| Frost | Passive max(0,2+F) Block at end turn. | max(0,5+F) Block. No Dexterity/Frail card calculation: the orb supplies the amount directly to GainBlockAction; gain/reaction hooks still apply. |
| Dark | D initially 6; passive growth max(0,6+F) at end turn, added to D immediately by that hook. | Damage D; target is the first monster in source list order with minimum current HP among those not isDeadOrEscaped, chosen when DarkOrbEvokeAction is constructed. Ties are not random; its local scan does not exclude halfDead separately. |
| Plasma | Passive one energy at **start** turn. Focus does not change it. | Two energy; unaffected by Focus. No Plasma passive at natural end turn. |

Lightning and Dark create THORNS-type damage, not Attack card plays. Strength,
Weak, Wrath, Vulnerable, attack-play counters and attack-only retaliation must not
be indiscriminately applied. Lock-On modifies the orb amount by binary32 ×1.5,
then Java float-to-int conversion (truncate toward zero for ordinary nonnegative
values). Lightning's single-target action reads Lock-On after target selection;
Electro uses the orb-damage matrix's per-enemy Lock-On path. Dark reads Lock-On at
its action's initial update, after target capture. Downstream defenses/death still
use damage processing; an orb amount is not guaranteed HP lost.

Lock-On is a turn-based debuff: its amount is duration, not multiplier strength.
Successful ordinary reapplications add durations; the inherited branch for an
existing amount=-1 does not stack. At end of round, amount zero requests removal;
otherwise request reduction by one. When that reduction resolves against current amount one, it
requests removal instead of leaving zero for another round. A reduction smaller
than the current amount subtracts it; one at least as large requests removal.
Its multiplier remains ×1.5 regardless of duration. No local duration cap is
evidenced; source signed-int overflow and native fidelity remain §6.3/§9.
These local hooks do not close global dispatch, ordering or terminal interruption.

Lightning passive/evoke and each Rip and Tear/Thunder Strike action use
cardRandomRng and the same alive-only no-exception selector: return null if the
room is basically dead; otherwise list monsters excluding halfDead, isDying,
isEscaping, and select one index uniformly. Empty candidates return null without
a bounded-index call; singleton candidates still call bounded random(0,0).
Selections are made separately at action execution, not preselected together at
card use. Random-attack actions then recalculate that card's damage for the chosen
enemy. Chaos instead samples its 1/2 orb identities at use using cardRandomRng,
including Plasma; each sample is a fresh four-way index, with replacement.
These are conditional local laws, not root-seed membership or RNG-cursor parity.

Static Discharge's exact local predicate is damageAmount>0, nonnull owner other
than the power owner, and damage type neither THORNS nor HP_LOSS. It runs in
onAttacked, after the player's earlier damage-change callbacks but before final
HP-loss callbacks and subtraction. Therefore “actual positive HP lost” is **not**
its predicate: later Tungsten Rod/other loss changes may still prevent the loss.
It inserts separate channel requests at top; death/rescue and terminal cancellation
of those requests are not certified by this predicate.

### 5.2 Fission and repeated evokes

Fission captures **occupied** count n, not slot count, once. Effective local
top-insertion order is remove-all/evoke-all, gain n energy, draw n. Base removal
repeatedly removes front without onEvoke. Upgrade's evoke-all action schedules one
front evoke per slot present at its own execution; empty-slot invokes do nothing.
The energy/draw count remains the earlier n even if later reactions change orbs.
Plasma evoke energy is additional to Fission+'s n energy. There is no single
aggregate damage, Block or energy event standing for all these effects.

Dualcast's nonremoving invocation precedes its final removing invocation.
Multi-Cast's positive count schedules count-1 nonremoving invokes followed by one
removing invoke. The same front is normally repeated, but the requests look up
the then-current front when executed; they do not carry an immutable captured orb
pointer. In-flight reactions/terminal clears must therefore remain separate from
the local repetition law. Multi-Cast with no front orb, or total<=0, does **not**
remove an orb and does not perform its action-owned energy payment.

## 6. Instance state, costs, payment, copies, and growth

### 6.1 Distinct cost and payment fields

Do not combine printed initial cost, mutable combat `cost`, `costForTurn`,
`freeToPlayOnce`, autoplay, `energyOnUse`, `ignoreEnergyOnUse`, queue
ignore-total, Corruption and FreeAttack into one discounted-cost scalar.

On the inspected player-use path, card damage is calculated, X capture may be
clamped to available energy unless ignoreEnergyOnUse, autoplay X sets
freeToPlayOnce, use issues its payload, cleanup is queued, other-card notifications
occur, the card leaves hand, and ordinary positive-cost payment follows. Ordinary
payment is skipped for freeToPlay(), autoplay, or Corruption Skills. freeToPlay()
is one-play-free or combat FreeAttack on an Attack; it is not necessarily a zero
costForTurn mutation. X actions below own a distinct later payment.

For Tempest/Reinforced Body/Multi-Cast, the action captures the card's
energyOnUse and freeToPlayOnce. At action execution its base count is captured X
if not -1, otherwise then-current energy; Chemical X adds two. Tempest+/Multi-Cast+
add one; Reinforced Body+ changes Block 7→9 instead. Only positive total produces
iterations and, if not freeToPlayOnce, consumes **all then-current energy**, not
the count or necessarily the earlier captured X. Multi-Cast additionally requires
a front orb before any count/payment work. Therefore zero-X upgrade and Chemical X
cases, free repeats, and no-orb Multi-Cast cannot share a generic pay-X routine.

| Mutable kind / field | Exact local state law |
|---|---|
| Force Field | Constructor, when dungeon/map context exists, applies updateCost(-1) for each historical combat Power play. Its triggerOnCardPlayed does the same per notified Power. Ordinary UseCardAction construction notifies hand/discard/draw, not exhaust/limbo, unless dontTriggerOnUseCard suppresses it. updateCost floors combat cost at zero; preserves the old combat-vs-turn offset where a real combat-cost change occurs, then floors turn cost at zero. This is not active-Power counting; the complete mixed-origin callback inventory remains §9. |
| Streamline | ReduceCostAction targets shared identity across cardInUse, hand, draw, discard, exhaust, limbo, with object deduplication. modifyCostForCombat(-1): if turn cost>0, reduce/floor it and assign combat cost to it; otherwise, for nonnegative combat cost reduce/floor combat cost and leave turn cost zero. Thus temporary zero plus combat discount is not simply printed 2 minus play count. |
| Steam Barrier | ModifyBlockAction uses the same shared-identity combat set and adds -1 to baseBlock. **There is no baseBlock floor in this source action**; its apparent negative check clamps baseDamage, not baseBlock. Ordinary calculated Block has its own nonnegative floor. Upgrade adds two to the current baseBlock; it does not reset accumulated wear. |
| Claw | GashAction first adds two to the playing object, then Claw instances in discard, draw, hand. It does not scan exhaust/limbo/master deck and does not install a global future-Claw bonus. Fresh generated Claw starts 3/5; a stat-equivalent copy captures an existing baseDamage instead. Each qualifying repeated use can issue another growth action. |
| Genetic Algorithm | misc starts one, independently of upgrade; applyPowers sets baseBlock=misc. IncreaseMiscAction adds 2/3 to master-deck cards matching shared identity and to the deduplicated combat set above, then recalculates. Its unused constructor miscValue is not a cap/snapshot restoration. The subsequent GainBlockAction already captured the old calculated Block when use ran. Growth is not victory-only and is not applied to every same-name card. A fresh-identity temporary copy cannot update an unrelated master card. |

White Noise's setCostForTurn(0) changes only nonnegative turn costs, floors a
negative requested amount at zero, and marks turn-cost modification when different
from combat cost. It is not freeToPlayOnce and not a combat discount. Cost-changing
upgrades preserve the old turn-minus-combat offset when the old turn cost is
positive (the offset itself may be negative), then floor the result at zero; a zero
turn cost stays zero. They mark upgradedCost rather than granting one-play-free.
resetAttributes restores turn cost from combat cost and clears
the turn-cost flag; this does not itself certify every lifecycle reset caller.

### 6.2 Copy capture and repeats

Every blue makeCopy constructs fresh constructor state, including Force Field's
history-based cost initialization when its context guard holds. It does not copy
the source instance's learned misc, combat wear or upgrade by itself. Stat-equivalent copy
then replays upgrade count and captures name/target/upgrade state, base
damage/Block/magic, combat and turn cost, both cost-modified flags, bottle flags,
seen/locked, misc and **freeToPlayOnce**. It has fresh shared identity. It does not
automatically copy energyOnUse, ignoreEnergyOnUse, autoplay, purgeOnUse,
dontTriggerOnUseCard, retain or arbitrary transient fields. A same-instance copy
adds preservation of shared identity to this capture; it is still another object.
These are internal identity relationships, not a requirement to expose UUIDs.

Amplify requires a nonpurge Power and positive charge; it captures a same-instance
copy, puts it in limbo, marks purgeOnUse and queues it with the original
energyOnUse, autoplay=true and ignore-total=true. It decrements a charge and removes
the power when spent; player turn end removes remaining charges. Echo resets its
cardsDoubledThisTurn at start turn; for a nonpurge play with positive intensity it
uses the precise predicate cardsPlayedThisTurn.size - cardsDoubledThisTurn <=
intensity, then increments that counter and queues the same kind of copy. Purge
guard prevents these repeat copies recursively generating themselves. Echo and
Amplify can each queue a repeat; they are not multiplicative recursion.

Heatsinks/Storm are Power-use notifications, not passive checks of installed Power
count. Their stacking adds intensity. A repeated Power can install another effect
and notify already installed effects; multiplying one effect scalar cannot replace
that sequence. Complete mixed-origin legality/destination/cancellation is §9, not
inferred from the repeat flags.

### 6.3 Source numeric domains and native boundary

The relevant source fields/counts are signed Java **32-bit integers**. Constructors
in these records supply the finite values above; legal histories, not arbitrary
external integer injection, generate subsequent states. Arithmetic is source int
addition/multiplication/division (including wrap at int overflow), except explicit
binary32 damage/Block modifiers. Nonnegative pile-size division in Aggregate
discards the remainder. Dark D, Claw damage, Genetic misc and Steam baseBlock have
no explicit gameplay cap in the inspected mutation methods. Do not invent 999
caps for those fields or clamp Steam's stored baseBlock to zero.

Concrete scoped caps/floors: Capacitor's one-slot route stops at ten; Consume
stops at zero. Focus stacking/reduction clamps to [-999,999] and removes zero;
its constructor itself merely accepts the supplied amount. Strength/Dexterity
stack/reduction have the same signed clamps. EnergizedBlue constructor/stack cap
at 999. EnergyPanel.addEnergy caps the resulting total at 999; EnergyPanel.useEnergy
floors consumption at zero, while setEnergy assigns directly without that add cap.
Thus doubling ordinary energy 600 produces 999, not 1200. Card-calculated
damage/Block have nonnegative calculation floors; source raw state remains separate.

No general cap is evidenced for all other power intensities: Bias/Heatsink/Hello
override stacking with integer addition; other inherited stack behavior must not
be extrapolated from Focus. Draw intensity is not a hand-capacity increase.
The hand admission capacity here is ten. A larger legal deck, history, orb payload,
or request sequence is not permission to truncate content or invent a defeat.

**Native contract, not an approved overflow-fidelity choice:** represent all
rule-relevant state losslessly within supported allocation/numeric resources;
exhaustion is administrative failure, not gameplay clamping, death, retry or a
missing card. Exact handling of legally reachable source-overflow edge histories
must remain an explicit fidelity decision if native arithmetic differs. This
document does not claim to prove a finite bound or reachability of every int value.

## 7. Movement, selection, retention, and dependent Statuses

| Site | Exact local behavior, not a universal movement law |
|---|---|
| Hologram | Empty discard gives no selection; with <=one card it automatically tries retrieval. Otherwise require one grid choice. Hand<10 moves the same card to hand; at full hand it remains in discard. No draw callback merely from this movement. |
| Seek | Empty draw pile gives no selection. If size<=requested count, process all in source group order; otherwise show a separately sorted candidate group, not hidden draw order, and process selected order. Full-hand selected/automatic cards move to discard; other selected cards move to hand. No draw callback. |
| All for One | Scan discard at follow-up execution in source group order. Predicate is **cost==0 or freeToPlayOnce**, not costForTurn==0/freeToPlay()/X. Enqueue one DiscardToHandAction per match. Each later action rechecks that the same card remains in discard and hand<10; otherwise leaves it there. |
| Recycle | Empty hand no-op; singleton automatically chosen; otherwise mandatory one-card hand selection. Selected costForTurn=-1 gives current energy at selection/retrieval boundary; >0 gives that value; zero/-2 give none. Enqueue gain at top **before** moving to exhaust, so exhaust-hook insertions can interpose before the gain resolves. It does not use freeToPlayOnce as a zero-cost predicate and does not inherently exhaust itself. |
| Scrape | Draw request carries a ScrapeFollowUp callback. The callback iterates DrawCardAction.drawnCards, preserving cards with costForTurn=0 or freeToPlayOnce; for others it requests hand→discard, invokes triggerOnManualDiscard and increments the non-end-turn discard counter. This is not a filter over the whole hand. Exact shared drawnCards ownership under nested draw callbacks remains §9. |
| Reboot | ShuffleAll invokes onShuffle relic hooks at construction, shuffles discard, inserts a random hand-to-deck request for up to 99 cards (actual hand<=10), and transfers discard through souls. Hand moves sample without replacement via successive random-card removals using cardRandomRng; discard/draw shuffles use shuffleRng. No exhausted card, installed Power, master-deck card, or arbitrary limbo object is revived. Final transfer/shuffle/draw ordering through souls remains §9. |
| Rebound | First after-use notification following installation only clears justEvoked. Each later after-use consumes a charge; non-Power sets that UseCardAction's reboundCard flag, Power still consumes without topdecking. End turn removes remaining Rebound. Cleanup gives actual exhaust precedence over rebound; a Strange Spoon prevention takes the nonexhaust branch where rebound can apply. Purge/Power paths have separate returns. It is not unconditional resurrection from exhaust or installation. |
| Equilibrium | At player end turn set retain=true only on non-Ethereal current hand cards. At end round remove if intensity zero, otherwise request reduce by one. This temporary flag differs from selfRetain and does not stop all turn-end card effects. Exact cross-source cleanup ordering remains §9. |
| Self Repair | Local onVictory reads positive current HP and heals total Repair intensity through normal healing hooks. It cannot locally resurrect a player with HP<=0. Invocation on escape versus other battle endings, modifier order and terminal clear remain shared ending fields (§9). |

There is no named Defect special-card output beyond these fixed Statuses; orbs are
not card kinds. Burn and Void remain COLORLESS **Status**, not Skills. Neither
passes ordinary canUpgrade selection; Burn has an explicit upgrade method for
source-supplied Burn+ while Void's upgrade method is empty.

| Dependent kind | Source target/cost; base/upgraded payload | Creation and conditional use |
|---|---|---|
| Burn (`Burn`) | none; -2/-2 (unplayable); end-turn 2/4 THORNS damage to player | Overclock and Overclock+ create **base** Burn, one in discard, fresh identity. End-turn playing-card hook marks dontTriggerOnUseCard and enqueues its special end-turn play; use issues self-damage only with that flag. Ordinary permitted Status play (e.g. Medical Kit) does not itself issue that damage. Shared prevention/cleanup applies. |
| Void (`VoidCard`, ID `Void`) | none; -2; Ethereal; no upgraded variant | TURBO and TURBO+ create one fresh base Void in discard. Actual draw queues loss of one energy; mere Hologram/Seek/All for One movement does not. use has no payload. End-turn Ethereal exhaustion is separate from the draw loss. |

The fixed discard producers use stat-equivalent copies of a new Status template;
Master Reality does not upgrade Status/Curse. Do not turn Overclock+ into Burn+
or invent Void+. Retain, Medical Kit, autoplay/end-turn flags, draw/exhaust effects
and terminal cancellation need their actual source paths, not “unplayable, erase”.

## 8. Generation: actual character and exact source policies

Player-owned Ironclad card-pool initialization is authoritative. Shard reward
expansion does **not** replace ordinary combat-generator pools with all colors.
Note origin and the color of the generating card do not select a different player.
Each selection below is a new independent bounded-index call conditional on the
then-applicable ordered candidate list, not rarity-first sampling or deduplication.

| Producer | Selection and capture | Creation/cost |
|---|---|---|
| Hello World | At start-turn hook, if room not basically dead, one selection per Hello intensity from current commonCardPool using cardRandomRng. No healing-tag filter and no rarity roll. | makeCopy of selected template; MakeTempCardInHandAction construction applies Master Reality to eligible non-Status/non-Curse, then stat-equivalent fresh-identity output. No cost discount. |
| White Noise | At use, concatenate source common/uncommon/rare pools, filter type=Power and not HEALING; uniform index using cardRandomRng. | makeCopy, then set turn cost zero **before** creation-action Master Reality upgrade; fresh identity; ordinary hand-overflow creation path. |
| Creative AI | At start-turn hook, same nonhealing Power selection policy once per Creative AI intensity. No Hello-style basically-dead guard inside this hook. | makeCopy; creation-action Master Reality upgrade; fresh identity; no zero-cost assignment. |
| TURBO / Overclock | Fixed Void / base Burn, respectively. No card-selection RNG. | One Status to discard; no Master Reality upgrade. |
| Copies and retrieval | Copy the specified kind/state, or move existing instance. No random-pool fallback. | §6.2 and §7; copying a Defect card does not recolor or reroll it. |

In the fully unlocked unmodded Ironclad pool, Hello World's common candidates are:
Anger, Armaments, Body Slam, Clash, Cleave, Clothesline, Flex, Havoc, Headbutt,
Heavy Blade, Iron Wave, Perfected Strike, Pommel Strike, Shrug It Off, Sword
Boomerang, Thunderclap, True Grit, Twin Strike, Warcry, Wild Strike. It samples
the **current** common pool, not a permanent all-blue common list or a 75-card pool.
The source getCard call itself does not remove its selection.

White Noise/Creative AI's Ironclad Power candidates are: Barricade, Berserk,
Brutality, Combust, Corruption, Dark Embrace, Demon Form, Evolve, Feel No Pain,
Fire Breathing, Inflame, Juggernaut, Metallicize, Rupture. Each has probability
1/14 under this initialized source pool. There is no Defect Self Repair or Creative
AI self-generation in that Ironclad set. Across another actual character the
nonhealing filter still matters; it is not replaced by a producer-color heuristic.

Generation hand admission uses capacity ten; overflow is requested to discard,
not silently erased. Show/add effects and souls complete later than the creation
request; exact durable admission/callback/terminal-clear boundaries remain §9.
Card-generation laws do not decide potion eligibility or transform/shop/reward
policies, and those sources must not inherit this table merely because they can
eventually produce the same kind.

## 9. Exact shared residual fields: not false transitive closure

The paired records and local action predicates above are evidence-backed facts.
They are not substitutes for these still-required composition fields. This list
narrows the interface rather than hiding every missing fact behind `CARD-X`.

| Field still requiring integrated proof/decision | Affected records / established boundary |
|---|---|
| scheduler.startTurn.order; energyRecharge versus orb/start-power hooks versus opening draw | Plasma, Loop, Hello, Creative AI, Bias, Charge Battery, Machine Learning. Local hook phases and payloads are known; total cross-container order is not supplied here. |
| scheduler.endTurn.order; retain/Ethereal/end-turn-play cleanup precedence | Lightning/Frost/Dark, Echo, Equilibrium, Burn/Void, Rebound. Do not infer immunity or one generic end queue. |
| scheduler.terminal.pending/current/cardQueue/preTurn/effects/souls disposition and death/rescue arbitration | All multi-payload cards, Fission+, Static Discharge, Sunder, Self Repair, orb hits. Local filters/HP predicates do not prove future dispatch or whether a queued resource/heal commits. No invented always-drain rule. |
| damage.prevention.fullOrder and heal.endingEligibility/modifiers | Buffer, Static Discharge, Burn, Sunder, Self Repair. The local callback predicates and positive-HP Repair guard above are fixed; complete legal prevention/rescue/escape/victory caller closure is not. |
| callbacks.playHistory.notificationInventory and cost.layerPrecedence under all origins | Force Field, FTL, Claw, Streamline; Confused/Corruption/FreeAttack/generated copies/repeats. Exact local cost mutators are known; exhaustive caller coverage and mixed-origin legality remain open. |
| copies.sameIdentity.dynamicCurrentCard versus repeated/temporary/permanent source ownership | Genetic Algorithm and UUID-targeted mutations. Static matching sets/copy capture are fixed; all nested repeat/exhaust/death interleavings and Note serialization round trips are not proven by this table. |
| draw.drawnCards.nestedOwnership and onDraw/discard callback continuation | Scrape, Overclock, all draw payloads. Shared mutable drawnCards tracking cannot be replaced with a guessed private callback-local list without closure. |
| shuffle.soulsCompletion/order and generated.handAdmission.commit | Reboot; all generated outputs/overflow. Inspected requests, piles, source RNGs and copy flags do not establish all effect/soul update and terminal-interruption checkpoints. |
| numeric.sourceOverflow.reachableHistories and native fidelity mapping | Unbounded-in-method Dark/misc/Claw/Steam/history/intensity growth. Source widths/caps above are known; no proof of all legal overflow histories or new gameplay cap. |
| observation.hiddenIntent/selectionKnowledge and suspended-choice resumption | Go for the Eyes, Seek, Hologram, Recycle. Do not expose hidden Runic Dome intent or draw order; exact detached native choice/continuation mapping requires integrated API review. |

Existing [execution research](2026-10-18-act1-execution-closure.md) and
[continuation research](2026-10-18-act1-continuations-closure.md) constrain these
fields, but neither a roster count nor this local static work asserts their full
closure. No proposed native ordering/overflow choice is silently approved here.

## 10. Evidence locators, checks, and future acceptance

The private claim ledger retains exact paths, hashes, commands and method ranges.
Non-sensitive source locators for every public record:

- All 75 rows: `cards.blue.<class key>` constructor, use, upgrade, makeCopy; custom
  applyPowers/calculateCardDamage/triggerOnCardPlayed/configureCostsOnNewCard where
  named above. Basics/Genetic use retained full bodies; remaining 70 use fresh
  missing-body inspection, cross-checked against retained constructor identities.
- Orbs: `orbs.AbstractOrb`, `Dark`, `Frost`, `Lightning`, `Plasma`; player channel,
  evoke/remove, slot increase/decrease, filledOrbCount and start-turn-orb methods.
  ChannelAction, RedoAction, DarkImpulseAction, FissionAction, EvokeAllOrbsAction,
  RemoveAllOrbsAction, MulticastAction and TempestAction supply the local conditions.
- Mutable state/copies: AbstractCard copy/cost/reset methods; GetAllInBattleInstances;
  GashAction, ReduceCostAction, ModifyBlockAction, IncreaseMiscAction; Amplify/Echo
  onUseCard. Actual callback/ending helpers are retained rather than newly rebuilt.
- Generation: AbstractDungeon.initializeCardPools/getCard/returnTrulyRandomCardInCombat,
  player pool selection, HelloPower/CreativeAIPower, WhiteNoise.use, temporary-card
  creation and Status methods. Random damage: MonsterGroup.getRandomMonster,
  AttackDamageRandomEnemyAction, LightningOrbPassive/EvokeAction, DarkOrbEvokeAction,
  DamageInfo's orb-damage matrix. LockOnPower's end-round hook, inherited
  AbstractPower.stackPower/reducePower and ReducePowerAction.update establish the
  local duration lifecycle, not global hook dispatch.

Actual checks are private static/document checks: failure-first helper import
test, duplicate/missing ZIP entry and output-containment negative controls,
environment-stripping test, four input/tool pins before and after inspection,
75-kind/rarity/target/cost/keyword coverage, method/identity checks, local links,
whitespace, privacy and one-public-path allowlist. Failed inspection attempts are
retained, not relabeled gameplay outcomes. None executes these future cases:

| Future case | Discriminating expected local result / remaining observation |
|---|---|
| DF-N01 zero capacity | Note Zap at zero slots installs/history-appends nothing; Chaos still makes its identity samples. Capacitor enables later channeling, without creating starting Lightning. |
| DF-N02 queue and Consume | With three occupied slots, Consume removes last without evoke and requests Focus; at zero still requests Focus. Repeated Capacitor stops at ten. |
| DF-N03 Dark identity | Start D=6,F=0; passive makes D=12. Change F to -10: future growth zero, D stays12. Recursion preserves12; fresh Dark starts6. Lowest-HP tie selects first source-list enemy, not random. |
| DF-N04 Plasma and Fission | Focus±9 never changes Plasma 1/2. Base Fission of Frost+Plasma grants n=2 energy/draw without orb payload; upgrade additionally requests Frost evoke and Plasma's two energy. Assert final timing only after §9 closure. |
| DF-N05 X/no-orb | Captured X=0: base Multi-Cast leaves orb untouched/no action payment; upgraded invokes once when orb exists. No-orb upgraded Multi-Cast still no payment; Tempest+ requests one Lightning. Chemical X adds two, not extra paid energy. |
| DF-N06 wear/growth | Steam starts6 then stores5, eventually negative baseBlock while calculated gain floors; upgrade adds2 to worn state. Fresh Claw does not inherit prior Claw growth; stat-copy does. Genetic gains old captured Block while matching misc grows2/3 even before victory. |
| DF-N07 cost predicates | A combat-cost1/turn-cost0 card without freeToPlayOnce is not All for One eligible; combat-cost1 with freeToPlayOnce is. Scrape uses turn-zero OR one-play-free. Recycle turn-cost X uses current energy; unplayable gives none. |
| DF-N08 generator authority | In Ironclad context White Noise/Creative AI select only the 14 listed Powers, Hello only current red commons. Copies/fixed Burn/Void keep their identities. Master Reality does not upgrade either Status. |
| DF-N09 callbacks/ending | Buffer prevents positive incoming amount at its change hook; Static Discharge must use its callback-time positive amount, not final HP lost. Lethal/retaliation/escape cases remain blocked on precise §9 fields, not guessed totals. |
| DF-N10 choices and transport | Seek full-hand outputs go to discard; Hologram full-hand retrieval remains there; neither triggers Void draw loss. Resume choices once, without duplicate payment/output or hidden-pile-order leakage; API mapping remains integrated work. |
| DF-N11 FTL/random | Draw condition uses history size minus one at follow-up and top-inserted draw precedes appended damage; random hits reselect/recalculate each time. Test singleton RNG consumption, target death, and terminal interruption separately. |
| DF-N12 Lock-On lifetime | UNEXECUTED: with no other mutation/interruption and each requested local action resolved, duration 2→1→removed over two end-round hooks, not 2→1→0 for another round. Successful base-plus-upgraded applications give stacked 2+3=5 duration, still ×1.5 orb damage rather than a stronger multiplier. Global dispatch/terminal ordering remains §9. |

This document does not approve the whole specification, an implementation plan,
implementation start, or main merge. Independent exact-head review and controller
readback are required. The current documentation-only baseline exception covers
only the two already-known CUDA-environment failures with no new failures; this
author did not rerun product suites or change product/tests/dependencies.
