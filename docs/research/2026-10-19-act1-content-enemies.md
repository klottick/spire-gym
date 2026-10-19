# Act 1 enemy records: A0 setup, moves and conditional behavior

2026-10-19 · `spire-gli.2.3.22` · **Normative-record research candidate.**

These records supply the 25 enemy kinds missing from B07's public content
coverage. They describe standard, unmodded, fully unlocked Ironclad A0 Act 1 with
Act 4 available. They do not approve implementation or claim complete transitive
combat closure. Exact remaining obligations are listed in §10. There was **no
original-game execution**; all native acceptance cases below are future cases.

## 1. Reading a record

The owned archive is pinned by SHA-256
`cfad868ac8d65a88e71a0bf096fb09f78811e553effe0787c5309a655e081673`
(365086855 bytes). Findings are about those bytes; upstream build provenance is
not established. Retained, previously reviewed enemy source/bytecode is the main
evidence. Initial targeted reconstruction covered the common intent/preview
distinction; review correction additionally checks Strength and Block application
bounds. No source, executable port, class, helper or fixture is published with
this document.

The tables are complete local A0 records, not imports requiring unpublished
drafts. Source class and member anchors in §11 permit private evidence review;
they are not required public rule definitions.

Notation and defaults:

- `U[a,b]` is one bounded, uniform integer choice including both endpoints.
  Ordinary rolled HP initializes **current HP = max HP**. Equal-endpoint boss
  setters do not establish absence of an incidental helper call.
- `hit(n)×k` denotes **k distinct NORMAL damage actions**, each with unmodified
  base n; omitted k is 1. It is never one hit of n×k. Player is the target of
  attacks/debuffs; self is the recipient of Block/buffs unless specified.
- `+B n`, `+Str n`, `Weak n`, `Vulnerable n`, `Frail n`, and `Entangled n`
  name requested Block, Strength and power amounts. They are not promises to
  bypass Artifact, prevention, power hooks or action cancellation.
- Moves are written `source move-ID / source intent: payload`. IDs are local to
  an enemy kind, **not globally unique or proposed observation IDs**. Intent
  tokens such as `ATTACK_DEBUFF` are the source categories, not payload summaries.
  A status-producing attack can use this token without applying a debuff power.
- `none` in setup means no intrinsic starting power/Block in this kind's scoped
  ordinary route. It does not suppress battle-start relic or room effects.
- Unless overridden below, ordinary actions request their following AI choice
  through a queued RollMoveAction. Deterministic continuations do not become new
  independent categorical choices. Source rollMove can still consume an integer
  even when the selected getMove ignores it.
- Mad and Sneaky Gremlin's repeated attacks are explicit exceptions: they queue
  SetMoveAction for the next attack, not RollMoveAction or an independent next-move
  roll. Fat Gremlin's ordinary continuation still uses RollMoveAction.

### Numeric and random domains

HP, base/output damage, Block, power amounts, counters and stolen-gold ledgers
are source signed 32-bit integer fields; move IDs are signed bytes. Initial
domains and finite branch thresholds are given below. Source-defined numeric
**application bounds** are distinct from field width and resource exhaustion:

- StrengthPower's constructor, stackPower and reducePower clamp the resulting
  amount to **[-999,999]**; stacking/reduction apply their addition/subtraction
  before that clamp. The stated +Str payload remains the requested amount, not
  a pre-clipped move definition. For example, Strength999 with an accepted +3
  request remains999, not1002, through this application path.
- AbstractCreature.addBlock floors the processed gain, adds it to current Block,
  then caps currentBlock at 999 for every creature, including enemies. The nearby
  player-only achievement branch does not guard this cap. This is an upper cap
  at this helper, not a claim of a universal minimum-zero clamp or that every
  Block field write uses it; requested +B is not rewritten to the headroom.

These are not caps on all powers, HP, histories, Guardian quotas or generated
Status counts. Do not impose invented small caps on other growing counters.
The helper bounds do not prove that every intermediate field value or saturated
enemy state is naturally reachable, nor close all callback/modifier paths.
Source saturation is gameplay behavior; native numeric/allocation exhaustion
remains administrative, never defeat, skipped damage or fabricated escape. This
document does not select a new native width or import Java overflow as intended
gameplay.

For local AI, let `r` be uniform on integers 0–99; a primary interval's mass is
its integer count/100. A fair Boolean has mass 1/2. Chance-float secondary tests
use `k/2^24 < binary32(chance)`, with `k` uniform on 0…16777215 under the semantic
draw law. Define:

| Symbol | Source chance | Exact accepted grid-point count / `2^24` |
|---|---:|---:|
| `qC` | 0.5625f | 9437184 / 16777216 |
| `qT` | 0.357f | 5989466 / 16777216 |
| `qB` | 0.416f | 6979322 / 16777216 |
| `qA` | 0.4f | 6710887 / 16777216 |

These laws specify conditional semantic choices, not IID turns, original
seed/cursor parity, or independence of arbitrary fixed PRNG states. Native
`spire-lab-rng-hmac-sha256-v1` remains pinned; the seed-parity exception does not
waive these mechanically relevant conditional laws. Secondary draws occur only
in their stated branch; rejected moves are not generally rerolled/renormalized.

## 2. The 25 kind records

### 2.1 Ordinary and dependent kinds: 19

Each row includes the source identity, ordinary HP, intrinsic setup and every
scoped move payload. Shared reaction and transition definitions follow the table.

| Kind / source identity | Ordinary A0 HP; setup | Move-ID / intent and base payload | AI/state rule |
|---|---|---|---|
| Cultist / `Cultist` | `U[48,54]`; none | **3 / BUFF**, Incantation: Ritual 3. **1 / ATTACK**, Dark Strike: hit(6). | First select 3; subsequently 1. Ritual's first end-of-round callback is suppressed (§4). |
| Jaw Worm / `JawWorm` | `U[40,44]`; none; ordinary, not hard mode | **1 / ATTACK**, Chomp: hit(11). **2 / DEFEND_BUFF**, Bellow: +Str3, then +B6. **3 / ATTACK_DEFEND**, Thrash: hit(7), then +B5. | First 1, then history-conditioned Jaw law (§3). No Act 3 opening Strength/Block. |
| Red Louse / `FuzzyLouseNormal` (class `LouseNormal`) | `U[10,15]`; Curl Up `U[3,7]`; Bite base `U[5,7]` | **3 / ATTACK**, Bite: hit(fixed Bite roll). **4 / BUFF**, Grow: +Str3. | Nonattack/Bite primary 25/75; third identical selection redirects to other. |
| Green Louse / `FuzzyLouseDefensive` (class `LouseDefensive`) | `U[11,17]`; Curl Up `U[3,7]`; Bite base `U[5,7]` | **3 / ATTACK**, Bite: hit(fixed Bite roll). **4 / DEBUFF**, Spit Web: Weak2. | Same 25/75 and third-repeat redirect; no Grow. |
| Acid Slime (S) / `AcidSlime_S` | `U[8,12]`; none | **1 / ATTACK**, Tackle: hit(3). **2 / DEBUFF**, Lick: Weak1. | Initial fair choice 1/2; after acting alternate 1↔2. No Split. |
| Acid Slime (M) / `AcidSlime_M` | `U[28,32]`; none | **1 / ATTACK_DEBUFF**, Corrosive Spit: hit(7), Slimed1 to discard. **2 / ATTACK**, Tackle: hit(10). **4 / DEBUFF**, Lick: Weak1. | Acid law (§3); no Split. Split-created M uses inherited HP, not this range. |
| Acid Slime (L) / `AcidSlime_L` | `U[65,69]`; Split | **1 / ATTACK_DEBUFF**, Corrosive Spit: hit(11), Slimed2 to discard. **2 / ATTACK**, Tackle: hit(16). **4 / DEBUFF**, Lick: Weak2. **3 / UNKNOWN**, Split: two Acid(M). | Acid law until living half-HP interrupt (§5). Boss-created L uses inherited HP. |
| Spike Slime (S) / `SpikeSlime_S` | `U[10,14]`; none | **1 / ATTACK**, Tackle: hit(5). | Select 1 always; no Slimed and no Split. |
| Spike Slime (M) / `SpikeSlime_M` | `U[28,32]`; none | **1 / ATTACK_DEBUFF**, Flame Tackle: hit(8), Slimed1 to discard. **4 / DEBUFF**, Lick: Frail1. | Attack 0–29 / Lick30–99; third identical selection redirects to other. No Split. |
| Spike Slime (L) / `SpikeSlime_L` | `U[64,70]`; Split | **1 / ATTACK_DEBUFF**, Flame Tackle: hit(16), Slimed2 to discard. **4 / DEBUFF**, Lick: Frail2. **3 / UNKNOWN**, Split: two Spike(M). | Same 30/70 redirect law until living half-HP interrupt. |
| Blue Slaver / `SlaverBlue` | `U[46,50]`; none | **1 / ATTACK**, Stab: hit(12). **4 / ATTACK_DEBUFF**, Rake: hit(7), then Weak1. | Rake0–39 / Stab40–99; third identical selection redirects to other, including from initial selection onward. |
| Red Slaver / `SlaverRed` | `U[46,50]`; none | **1 / ATTACK**, Stab: hit(13). **3 / ATTACK_DEBUFF**, Scrape: hit(8), then Vulnerable1. **2 / STRONG_DEBUFF**, Entangle: Entangled1. | Initial 1; exact pre/post-Entangle law and execution-set flag in §3. |
| Looter / `Looter` | `U[44,48]`; Thievery15, ledger0, attack count0 | **1 / ATTACK**, Mug: hit(10), separate theft15. **4 / ATTACK**, Lunge: hit(12), separate theft15. **2 / DEFEND**, Smoke Bomb: +B6. **3 / ESCAPE**, Escape. | Two Mugs, then one fair Smoke/Lunge choice; Lunge→Smoke→Escape. Separate accounting in §6. |
| Fungi Beast / `FungiBeast` | `U[22,28]`; Spore Cloud2 | **1 / ATTACK**, Bite: hit(6). **2 / BUFF**, Grow: +Str3. | Bite0–59, Grow60–99; third Bite→Grow, consecutive Grow→Bite. Final-death guard in §4. |
| Fat Gremlin / `GremlinFat` | `U[13,17]`; none | **2 / ATTACK_DEBUFF**, Smash: hit(4), then Weak1. | 2 repeatedly in ordinary Gang; no A17 Frail. Conditional escape99 is not an ordinary member-death reaction (§4). |
| Mad Gremlin / `GremlinWarrior` | `U[20,24]`; Angry1 | **1 / ATTACK**, Scratch: hit(4). | 1 repeatedly via queued SetMoveAction, not RollMoveAction; Angry's damage-event predicate is not an Attack-play predicate. Conditional escape99 as §4. |
| Shield Gremlin / `GremlinTsundere` | `U[12,15]`; none | **1 / DEFEND**, Protect: +B7 to resolution-selected ally, otherwise self. **2 / ATTACK**, Shield Bash: hit(6). | Initial Protect even if alone; following selection uses takeTurn's living-actor count. Once Bash, continues Bash. Resolution-time targeting in §4; conditional escape99 there. |
| Sneaky Gremlin / `GremlinThief` | `U[10,14]`; none | **1 / ATTACK**, Puncture: hit(9). | 1 repeatedly via queued SetMoveAction, not RollMoveAction, in ordinary Gang; no Thievery/gold ledger. Conditional escape99 as §4. |
| Gremlin Wizard / `GremlinWizard` | `U[21,25]`; charge counter1 | **2 / UNKNOWN**, Charging: no damage, increment charge. **1 / ATTACK**, Ultimate Blast: hit(25), reset charge0. | Initial Charging twice then Blast; thereafter three Chargings per Blast. At charge3 select Blast, not an independent random choice. Conditional escape99 as §4. |

Louse HP is rolled before constructor Bite. Curl Up is a separate HP-stream roll
in prebattle, after initial intent initialization. Bite and Curl Up do not reroll
each turn, and neither is derived from HP. Instances have separate rolls and
histories; duplicate kinds in a formation are not aliases.

### 2.2 Elite kinds: 3

| Kind / source identity | Ordinary A0 HP; setup | Moves and exact continuation |
|---|---|---|
| Gremlin Nob / `GremlinNob` | `U[82,86]`; no initial Enrage; ordinary/event constructor enables Vulnerable | **3 / BUFF**, Bellow: Enrage2. **2 / ATTACK_DEBUFF**, Skull Bash: hit(6), then Vulnerable2. **1 / ATTACK**, Bull Rush: hit(14). First select3; subsequently Bash0–32/Rush33–99, but after two Rush selections force Bash. Consecutive Bash is legal. The constructor's non-Vulnerable variant would use ATTACK and omit Vulnerable, but is not these Act1 factory roots. |
| Lagavulin / `Lagavulin` | `U[109,111]`; ordinary asleep: queued Block8, then Metallicize8. Event awake: intrinsic Block0, no Metallicize | **5 / SLEEP**, Sleep: no attack. **4 / STUN**, Stunned: no attack. **3 / ATTACK**, Attack: hit(18). **1 / STRONG_DEBUFF**, Siphon Soul: Dexterity−1, **then** Strength−1, separately. Ordinary sleep/wake and event Siphon-first laws in §4. Source ID6 has a natural-opening no-hit branch, but the scoped ordinary natural-wake producer queues opening plus selection3, not a selected6 turn. |
| Sentry / `Sentry` | `U[38,42]` per actor; Artifact1 each | **3 / DEBUFF**, Bolt: Dazed2 to discard. **4 / ATTACK**, Beam: hit(9). In the three-actor factory, initial ordered list indices0,2 select Bolt and index1 Beam. Every actor subsequently alternates its own3↔4; deaths do not renumber survivors' phase. |

### 2.3 Boss kinds: 3

| Kind / source identity | A0 HP; setup | Moves and unmodified payload | Continuation |
|---|---|---|---|
| The Guardian / `TheGuardian` | 240; Mode Shift30, accumulated loss0, open, transition latch false | **6 / DEFEND**, Charging Up: +B9. **2 / ATTACK**, Fierce Bash: hit(32). **7 / STRONG_DEBUFF**, Vent Steam: Weak2 then Vulnerable2. **5 / ATTACK**, Whirlwind: hit(5)×4. **1 / BUFF**, Defensive Mode: Sharp Hide3. **3 / ATTACK**, Roll Attack: hit(9). **4 / ATTACK_BUFF**, Twin Slam: offensive transition, hit(8)×2, remove Sharp Hide. | Initial6→2→7→5→6 loop. Threshold transition replaces pending offensive move with1 when its queued change resolves; then1→3→4→5→6. Quota/Block20 timing is §5, not an instantaneous hit reaction. |
| Hexaghost / `Hexaghost` | 250; no intrinsic starting power; not activated; future-Sear upgrade flag false | **5 / UNKNOWN**, Activate: no hit; capture Divider base `floor(H/12)+1`. **1 / ATTACK**, Divider: captured-base hit×6. **4 / ATTACK_DEBUFF**, Sear: hit(6), Burn1 to discard (Burn+ after Inferno). **2 / ATTACK**, Tackle: hit(5)×2. **3 / DEFEND_BUFF**, Inflame: +B12 then +Str2. **6 / ATTACK_DEBUFF**, Inferno: hit(2)×6; upgrade existing draw/discard Burns, later create Burn+3. | Activate→Divider once, then Sear→Tackle→Sear→Inflame→Tackle→Sear→Inferno repeating. Per-actor orb-count state advances the cycle; visual orbs are not player Defect orbs. Capture/upgrade checkpoints in §5. |
| Slime Boss / `SlimeBoss` | 140; Split | **4 / STRONG_DEBUFF**, Goop Spray: Slimed3 to discard. **2 / UNKNOWN**, Preparing: no gameplay payload. **1 / ATTACK**, Slam: hit(35). **3 / UNKNOWN**, Split: fresh Spike(L) left, Acid(L) right. | Initial4→2→1→4 loop until living HP≤70 selects Split. Children use split-time remaining HP as both current/max, not normal large-slime rolls (§5). |

## 3. Exact history-conditioned AI

Let `last(X)` mean the last **selected** move-ID is X, and `two(X)` that the
last two selections are both X. The common setMove appends every non−1 ID,
including initial choices, direct continuations and interruptions. Selection is
not proof of execution; an interrupted earlier choice remains in history. A
queued SetMove may append later, and repeated selections of the same ID are not
silently deduplicated. Private history state is not automatically public knowledge.

### Jaw Worm

The initial selection is Chomp regardless of r. Later proposals and redirects:

| Primary interval | Without restriction | Restriction and exact replacement law |
|---|---|---|
| 0–24 | Chomp | If last(Chomp): Bellow with qC, otherwise Thrash |
| 25–54 | Thrash | If two(Thrash): Chomp with qT, otherwise Bellow |
| 55–99 | Bellow | If last(Bellow): Chomp with qB, otherwise Thrash |

Consequently after Chomp, `(Chomp,Thrash,Bellow)` masses are
`(0, 30/100 + 25/100×(1−qC), 45/100 + 25/100×qC)`.
After Bellow they are `(25/100+45/100×qB, 30/100+45/100×(1−qB), 0)`.
After one Thrash they are `(25/100,30/100,45/100)`; after two Thrashes,
`(25/100+30/100×qT,0,45/100+30/100×(1−qT))`.
For example, post-Chomp Bellow is exactly0.590625, not0.59. Rounded wiki
percentages or eligible-weight renormalization are not equivalent.

### Acid medium/large

| Primary interval | Proposal | Prohibited continuation and replacement |
|---|---|---|
| 0–29 | Spit1 | two(1): fair Tackle2 / Lick4 |
| 30–69 | Tackle2 | last(2): Spit1 with qA, otherwise Lick4 |
| 70–99 | Lick4 | two(4): Spit1 with qA, otherwise Tackle2 |

This table is a full conditional selection law: sum the primary-interval mass
times its stated conditional output mass. No other history is consulted. After
Tackle, for example, Spit has `30/100 + 40/100×qA` and Lick the complement,
not an exact46/54 law and not50/50. A large slime's split interrupt is a separate
state change, not a fourth weighted ordinary move.

### Two-way redirects and Red Slaver

| Actor | Unrestricted masses | History-conditioned result |
|---|---|---|
| Either Louse | nonattack1/4, Bite3/4 | two(nonattack)→Bite for every r; two(Bite)→nonattack for every r; otherwise unchanged |
| Blue Slaver | Rake2/5, Stab3/5 | two(Rake)→Stab; two(Stab)→Rake; otherwise unchanged |
| Spike(M/L) | Tackle3/10, Lick7/10 | two(Tackle)→Lick; two(Lick)→Tackle; otherwise unchanged |
| Fungi Beast | Bite3/5, Grow2/5 | two(Bite)→Grow; last(Grow)→Bite; otherwise unchanged |
| Nob, after initial Bellow | Bash33/100, Rush67/100 | two(Rush)→Bash for every r; no Bash-repeat prohibition |

Red Slaver first selects Stab and clears its first-selection flag. Afterwards:

| Entangle actually used? | Other state | Next selection law |
|---|---|---|
| No | not two(Scrape) | r75–99 Entangle (1/4); r0–74 Scrape (3/4) |
| No | two(Scrape) | r75–99 Entangle (1/4); r0–74 Stab (3/4) |
| Yes | neither two(Stab) nor two(Scrape) | r55–99 Stab (45/100); r0–54 Scrape (55/100) |
| Yes | two(Stab) | Scrape for every r |
| Yes | two(Scrape) | Stab for every r |

`usedEntangle` becomes true in Entangle **takeTurn**, not at selection or when
Artifact allows/prevents the power. Entangle remains in selection history.
There is no fresh 25% Entangle after the execution-set flag, and no unrestricted
55%-Stab law. The ordinary pre-Entangle non-Entangle path is history-driven,
not a fresh Stab/Scrape coin on each turn.

## 4. Local reaction and counter contracts

| Site | Exact local predicate, effect and checkpoint |
|---|---|
| Curl Up | Once only. Positive NORMAL damage with nonnull owner and damage strictly less than pre-subtraction current HP qualifies. Fully blocked/zero, other damage types and lethal hits do not. Latch immediately; closing/Block/removal go to the action-list bottom. It is not synchronous Block against all already-queued later hits. |
| Mad's Angry1 | Positive owner-attributed damage excluding HP_LOSS and THORNS queues +Str1 at top. The power itself has no surviving-hit check; later ApplyPower dead-target handling can reject it. Not triggered merely by playing an Attack that deals no qualifying damage. |
| Nob's Enrage2 | A qualifying Skill onUseCard callback queues +Str2 at top, after Enrage exists. No trigger from Attack/Power type, energy payment, or Slimed's Status type. Common UseCardAction's dontTriggerOnUseCard gate matters; repeat/autoplay does not imply callback suppression. Complete mixed-origin arbitration remains EN-U1. |
| Guardian Sharp Hide3 | A qualifying Attack onUseCard callback appends one player THORNS hit3 at bottom. One Attack's multihit payload is not multiple callbacks. Repeat plays can have separate callbacks; exact mixed-origin/cancellation composition remains EN-U1/U2. |
| Fungi Spore Cloud2 | onDeath first tests battle-ending status. Base die marks dying first. A final basically-dead group suppresses the cloud; with another living actor, queue player Vulnerable2 at top with null source, including enemy-turn Thorns kills. Not a reward-owned debuff after every fight. |
| Cultist Ritual3 | Its monster end-of-round hook skips the first occurrence; each later occurrence queues +Str3. Incantation does not make the first Dark Strike base9 automatically. |
| Shield Protect | Choose uniformly among eligible **actor instances at action resolution**, excluding self, dying actors and intent=ESCAPE. If empty, use self. The predicate does not independently exclude halfDead/isEscaping. Following intent is chosen separately during takeTurn by count of not-dying/not-escaping actors: count>1 keeps Protect, otherwise Bash. Thus pending Protect can self-Block after its last ally dies; it does not become a retroactive Bash. |
| Wizard charge | Start1. Charging increments; reaching3 selects Blast. Blast resets0 before selecting Charging. Interrupted/cancelled queued work and selected history are distinct from this execution counter. |
| Gremlin escape99 | All five kinds retain local escape/deathReact behavior with **99 / ESCAPE**, queuing EscapeAction. The reviewed direct caller scan found the broadcast outside the Act1 ordinary roots, and no direct escapeNext call. Gang setup has no leader/Minion, and killing an ordinary member does not broadcast flight. This bounded direct-unmodded caller fact is not reflection/mod/native closure and does not add the caller's later-act encounter. |

### Lagavulin ordinary and event timelines

- Ordinary setup queues Block8 then Metallicize8. Start Sleep; its first two
  sleeping actions do not attack. The **third sleeping action also deals no hit**:
  it queues opening and an Attack selection for the following action. No extra
  stunned action is added for natural wake.
- HP change through its damage override before wake is triggered selects STUN4
  and queues opening. A fully blocked/zero-HP-loss hit does not wake it. Opening
  is guarded against a dying actor; a selected stun on a lethal path is not
  survival/revival. After a surviving damage wake: Stunned→Attack→Attack→Siphon.
- Awake event construction sets its out/trigger flags; prebattle selects Siphon
  and installs neither ordinary Block8 nor Metallicize8. Thus intrinsic event
  opening Block is **0**, not8. Event cycle: Siphon→Attack→Attack→Siphon.
- Awake selection uses the executed attack counter (reset0 by Siphon, increment
  per Attack) and the two-Attack selection-history check. Do not substitute a
  global turn modulo if interrupted selections differ.
- Opening queues **reduce Metallicize by8**, not remove all Metallicize, and
  does not synchronously clear remaining Block. Siphon queues Dexterity−1 before
  Strength−1 as two ApplyPower actions: Artifact1 blocks the first, leaving the
  second eligible. Source power notifications occur before Artifact handling;
  successful application/stacking invokes onModifyPower.

## 5. Replacement and boss checkpoints

### Split, spawn and actual death

For Slime Boss and both large slimes, the local damage override runs after base
damage. A non-dying actor at `current HP ≤ max HP / 2` selects Split immediately
and queues a further SetMove; large slimes also latch the trigger. For positive
integer HP this includes exactly `HP≤floor(maxHP/2)`. Lethal damage has already
marked dying and does not trigger a live split. There is no local NORMAL-only
restriction, but each external HP-loss producer must actually enter this override.

Intent interruption is **not** immediate replacement. Children are constructed
in Split takeTurn using the parent's **then-current HP** as both child current
and max HP. Later damage before that turn can therefore lower both. Boss makes
Spike(L) left then Acid(L) right; each large makes two same-type M, left then
right. No M→S split exists. Fresh children receive zero inherited Poison and do
not copy parent powers/debuffs; their own Split/ordinary setup still applies.
Child first moves use that kind's normal initial AI, not the parent's pending
move/history. They are non-Minions with fresh identities.

Cannot-lose protection brackets the suicide/spawn sequence. Suicide calls
die(false), suppressing the base power-onDeath/relic-onMonsterDeath callbacks;
replacement is not an attributed kill/Fatal payoff. Spawn-relic callbacks precede
child init, power application and insertion by horizontal position. The reviewed
concrete relic override is Philosopher's Stone (+Str1 to a spawned monster), not
an automatic transfer of every parent modifier. Ordinary battle-start callbacks
are not universally rerun for each child.

The enemy-action queue is a group snapshot: spawned children are not appended
to the already-built queue for that round. Parent replacement is not room victory
while offspring remain. Actual child kills can qualify for their own hooks;
complete Fatal/Poison/multitarget/rescue arbitration is still EN-U1/U2.

### Guardian: queued transition, not immediate twenty Block

Only surviving HP decrease while open and not transition-latched contributes to
the initial30 quota. Blocked damage contributes0. The triggering hit resets the
internal loss accumulator, latches the transition and appends visual/change-state
work at bottom. **It does not itself grant Block or publish Defensive Mode.**
When the defensive change runs, it appends Mode Shift removal and Block20,
increases the next quota by10 and selects/publishes Defensive Mode1, closing the
actor. Earlier queued damage can precede the new Block. Excess threshold loss is
not credited to the following quota; latched/closed damage does not accumulate it.
Successive quotas are30,40,50,…, not a cap at50.

Defensive Mode's turn appends Sharp Hide3 and selects Roll Attack. Roll selects
Twin Slam. Twin Slam appends offensive change, two distinct hit8 actions, then
Sharp Hide removal, selecting Whirlwind next. Offensive change itself appends
the new Mode Shift/reset and any residual-Block loss at bottom: that insertion
does not move them ahead of the already queued two hits. Shared timers, Reaper
heal survival, rescue and card-origin laws are not settled by this sequence.

### Hexaghost: captured base versus mutable combat effects

Activate takeTurn captures player **current** HP H and writes Divider base
`floor(H/12)+1`, before selecting Divider. H=11/12/23/24 gives1/2/2/3 per hit.
Do not recapture on Divider resolution or after each hit. Strength and other
combat modifications remain distinct from this captured base.

Inferno queues six hit2 damage actions before BurnIncreaseAction, and sets the
actor's future-Sear-upgraded flag while producing that turn. BurnIncrease first
upgrades existing Burns in **draw and discard only**, not hand/exhaust/limbo;
at a later action gate it constructs/inserts three fresh Burn+. This does not
upgrade arbitrary later Immolate/Overclock Burns. Existing travelling cards
already in draw/discard are members; merely pending future producers are not.
Interruptions before upgrade, between upgrade and new insertion, or after insertion
are distinct durable states. This is not an observed timer/outcome trace.

### No enemy resurrection route added

None of these 25 local kind records has a revive/rebirth state machine. Fresh
split children are not revived parents. Half-dead and rescue checks in shared
actions do not add Darkling/Awakened One or later-act enemies. Player Fairy/Lizard
Tail rescue and legal player-card death effects remain shared execution content,
not a new enemy move. Enemy death, split-suicide, monster escape, player Smoke
Bomb and administrative fault are separate terminal causes.

## 6. Looter: theft, escape and reward ownership

Mug1 is selected initially. First Mug selects another Mug. During the **second
Mug's takeTurn**, choose Smoke2 or Lunge4 with a fair chance-float0.5 test; no
later repeat choice is made. Lunge selects Smoke; Smoke queues Block6 then
Escape3. Escape sets the room mugged flag and queues EscapeAction, not die.

Every Mug/Lunge queues a separate ledger action adding `min(15, player gold at
ledger resolution)`, followed by a gold-carrying DamageAction. The DamageAction's
initial update subtracts up to15 current gold directly; its later timed completion
applies the HP hit. Theft is not contingent on penetrating Block/positive HP
damage and does not use loseGold's callback route. Ledger increment, actual gold
decrement and HP damage are three checkpoints, not one atomic transaction.

The inspected ordinary queue offers no player decision between ledger and theft;
invented fixture mutation is not a legal interposition witness. Exact asynchronous
or other unclosed interposition/timer behavior remains EN-U3. Death uses the
ledger, not a hindsight recomputation of decrements, to add a **STOLEN_GOLD reward**.
That is not instant restitution. Golden Idol does not multiply this reward;
claiming uses gainGold and its restrictions. An Ectoplasm conditional in that
helper does not legalize Ectoplasm acquisition before Act1 ends.

An escaped Looter gives no ledger refund or Fatal credit. Common ordinary-gold
suppression tests **all retained monsters escaped**; a mixed Thugs fight with a
killed survivor is different. Mugged/card-reward handling is separately owned by
the reward path, not inferred from whether one actor disappeared. Neither enemy
escape nor player Smoke Bomb is automatically player victory, death or a promise
to drain every pending effect before screen/room transition.

## 7. Encounter factories, queues and event distinctions

Enemy kind ID, encounter key, room class and event owner are different fields.
The following ordered factory descriptions define the scoped 20 ordinary families
(4 weak +10 strong +3 elite +3 boss). Duplicate actors keep individual setup,
phase/history and reward/death identities.

| Pool / factory key | Relative weight | Ordered construction |
|---|---:|---|
| Weak `Cultist` | 2 | one Cultist |
| Weak `Jaw Worm` | 2 | one non-hard Jaw Worm |
| Weak `2 Louse` | 2 | two slots, independent fair red/green choices |
| Weak `Small Slimes` | 2 | fair choice: Spike(S) left + Acid(M) right, or Acid(S) left + Spike(M) right |
| Strong `Blue Slaver` | 2 | one Blue Slaver |
| Strong `Red Slaver` | 1 | one Red Slaver |
| Strong `Looter` | 2 | one Looter |
| Strong `Large Slime` | 2 | fair Acid(L)/Spike(L) choice |
| Strong `3 Louse` | 2 | three ordered independent fair-color slots |
| Strong `2 Fungi Beasts` | 2 | two Fungi |
| Strong `Gremlin Gang` | 1 | four successive uniform remaining-member index selections from Mad×2, Sneaky×2, Fat×2, Shield×1, Wizard×1; sample order is slot order; no final helper shuffle |
| Strong `Lots of Slimes` | 1 | exhaust the five-member pool Spike(S)×3/Acid(S)×2 by successive uniform remaining-member indices; order is sampled |
| Strong `Exordium Thugs` | 1.5 | left: red Louse1/6, green Louse1/6, Spike(M)1/3, Acid(M)1/3; right: Cultist1/3, Looter1/3, Blue Slaver1/6, Red Slaver1/6, from separate category/subtype choices |
| Strong `Exordium Wildlife` | 1.5 | left fair Fungi/Jaw; right uses Thugs' left weak-wildlife law |
| Elite `Gremlin Nob` | 1 | one ordinary Vulnerable-enabled Nob |
| Elite `Lagavulin` | 1 | one asleep Lagavulin |
| Elite `3 Sentries` | 1 | ordered three Sentries, Bolt/Beam/Bolt initial phase |
| Boss `The Guardian` | — | one Guardian |
| Boss `Hexaghost` | — | one Hexaghost |
| Boss `Slime Boss` | — | one Slime Boss; offspring not extra pool entries |

Gang sampling is over remaining **members**, not five equiprobable kinds or
sampling with replacement. After both Fat members are drawn, Fat has zero next
mass; Shield/Wizard cannot repeat. Lots is not five IID type rolls. Thugs is not
flat four-way per slot. Some factory helpers construct candidates before category
selection; discarded construction's incidental RNG consumption is not required
seed parity, but the observable category/setup law remains required.

### Queue law and boss selection

The initial ordinary list contains three weak entries, one first-strong, then
twelve further strong entries (16 total); elite list has ten entries. Ordinary
append retries the **original** weighted proposal until it differs from the last
two list entries. Elite append rejects only the preceding elite. First-strong
has a separate exclusion retry: after final weak `2 Louse`, reject `3 Louse`;
after `Small Slimes`, reject `Large Slime` and `Lots of Slimes`. It does not
independently apply the ordinary last-two filter. Leftover Looter/Blue-Slaver
exclusion branches cannot be reached from this weak roster.

Weights sort ascending, retaining insertion order on ties, normalize using
binary32 total and accumulate strict upper float cutoffs against the 24-bit draw
grid. Weak total8 and strong total16 give exact quarter/weight fractions here.
Unrejected elite proposals in Nob/Lagavulin/Sentries order have respectively
5592406/5592405/5592405 grid points out of16777216, not exact thirds. At an
eligible set E, repeat rejection gives conditional mass `p(i)/Σ[j∈E]p(j)` for
eligible i, retaining the original proposal law. This formula does not declare
independent initialized-stream histories or invent an empty-set fallback.

Room entry reads the head; **departure** from ordinary MonsterRoom removes the
ordinary head, departure from MonsterRoomElite removes the elite head. A question
room instantiated as MonsterRoom uses the ordinary route. Event-owned fights below
do not consume these queues merely because eliteTrigger is true.

Standard boss discovery chooses the first unseen in Guardian→Hexaghost→Slime
order. If all seen, source calls a three-item Java shuffle seeded from monsterRng;
no uniform-unseen rule applies. A singleton is duplicated in its list. Announcement
reads the head; boss entry obtains/removes it. Seen is recorded at the boss's
prebattle, not announcement or victory. Exact initialized/history-conditioned
shuffle measure and profile authority remain separate obligations, not evaluated
uniform permutation masses in this document; see the
[profile contract](2026-10-19-act1-profile-time.md).

### Four event-combat variants and noncombat event damage

| Owner / encounter | Exact relevant distinction |
|---|---|
| Dead Adventurer / `3 Sentries` | Three normal A0 Sentry instances and their ordered phases, constructed by the event, not ordinary elite-list consumption |
| Dead Adventurer / `Gremlin Nob` | Normal Vulnerable-enabled A0 Nob; no invented non-Vulnerable event variant |
| Dead Adventurer / `Lagavulin Event` | Awake constructor, no intrinsic opening Block/Metallicize; Siphon first, not three fast-forwarded sleeping turns |
| Mushrooms / `The Mushroom Lair` | Three ordinary Fungi, not the normal two-Fungi family; each retains Spore Cloud and its own AI |

Dead Adventurer constructor chooses event variant by `U[0,2]` in
Sentries/Nob/awake-Lagavulin order, and shuffles the one-each GOLD/NOTHING/RELIC
categories once using a miscRng-seeded Java Random. The latter call's existence
does **not** prove six equiprobable permutations or its initialized joint law.
Search integer risk is25,50,75 out of100 as successive successes remove the next
category. Leave is offered between searches; three successes end searching.
Ambush chooses the already-fixed event fight; the presentation Fight control is
not a new strategic escape. Opening description identifies the variant.

The event records ordinary elite gold `U[25,35]`. Before combat it adds remaining
unsearched gold30/relic categories as rewards; searched categories cannot duplicate.
It sets eliteTrigger, so Preserved Insect, Sling of Courage and Slaver's Collar
can apply. It does **not** use MonsterRoomElite.dropReward, Black Star's extra
relic or burning-room setup merely because the actors are elite kinds.

Mushrooms fight records gold `U[20,30]`, plus Odd Mushroom (Circlet if already
owned); Odd Mushroom is victory loot, not in-fight protection from Spore Cloud.
Eat is noncombat: heal `trunc(0.25×maxHP)` then request Parasite acquisition.
Common event combat initializes the group and runs prebattle preparation; its
reward screen is event-aware. Prayer Wheel's extra ordinary-MonsterRoom card set
does not follow just from event combat. Ordinary eligible gold's Golden Idol
bonus is a reward hook, not an intrinsic Mushrooms×1.25 enemy modifier.

Scrap Ooze's repeated HP-payment/relic-risk interaction is **not an enemy or
encounter factory**, nor damage from a monster Attack. Likewise noncombat event
damage, Golden Idol traps and Neow HP costs do not create an extra enemy kind or
consume combat queues by analogy. Their actual payment/death/rescue and parent
event continuations require their source-owned event/shared laws. Off-color
cards/Note do not import later-act enemy encounters or other-character starts.

## 8. Burning elite application and Emerald Key are different operations

For an actual marked elite room with Final Act available, choose `U[0,3]` once:

| Branch | Applied to **every** actor, including all three Sentries |
|---:|---|
| 0 | Strength+2 |
| 1 | max-HP bonus `round(binary32(0.25×old maxHP))`; add bonus to max HP and heal that amount |
| 2 | Metallicize4 |
| 3 | monster Regeneration3 |

For these positive small HP values, the HP bonus is nearest integer with halves
up. It is a rounded **bonus**, not simply truncate(1.25×HP). Prebattle queues
ordinary monster setup before burning application. Combat-wait work applies these
before battle-start relic callbacks. Preserved Insect then limits **current HP**
to `trunc(binary32(0.75×new maxHP))`; it does not lower max HP and its local path
has no minimum-one clamp. For a rolled Sentry38, HP bonus10 gives max48; Insect
then current36. For rolled39, bonus10 gives max49/current36. Ordinary Lagavulin
Metallicize8 stacks with burning4; opening reduces8 and leaves4.

Metallicize's pre-end-turn-card hook queues Block equal to its amount. Monster
Regeneration's end-turn hook queues Heal3 if not half-dead/dying/dead; unlike
player Regeneration, this local power does not decrement its amount each turn.
Actual heal/power/death composition still follows the shared callers, not an
extra action before every enemy hit.

The room's reward routine adds elite relic reward(s), then offers an Emerald Key
when Final Act is available, the key is absent, the room is marked, and rewards
are nonempty. **Offering, choosing and durable possession are not one operation**:
the flag is set through an obtain-key effect, not by buff application or enemy
death alone. Skipping/deferred key acquisition must remain representable. Event
elite identities do not supply this marked-room reward path.

Map marker selection is a different RNG/site. The current reviewed map work
establishes the empty-candidate failure branch but **does not prove root-seed
reachability or impossibility of zero assigned elites**. No retry, regeneration,
forced placement, markerless success or defeat policy follows; preserve the
[exact map obligation](2026-10-19-act1-map-seed.md). This enemy record does not
turn a search miss into a gameplay minimum.

## 9. Published intent, damage-time state and fixed Status production

A selected move contains kind-local ID, intent category, base damage and optional
multihit count. createIntent publishes from that selected record and computes its
damage preview; it is not a new move roll or damage resolution. applyPowers
updates the actor's DamageInfo entries separately and recalculates the selected
move's preview. For the scoped ordinary preview, attacker damage-give powers,
target damage-receive powers, target stance, final-give and final-receive hooks
are evaluated in source list order using binary32 intermediates, then floor and
nonnegative clamp. This is a preview path, not proof of all target-damage hooks.

Enemy takeTurn producers queue DamageActions using DamageInfo objects, **not a
copy of displayed intentDmg**. Their actions later call target.damage, with
then-applicable Block, damage/reaction/death checks. A queued action retains its
DamageInfo reference; do not assert either immutable preview-as-hit or universal
recalculation of every modifier before every hit. Captured Divider base, changes
to Weak/Strength/Vulnerable/stance, per-hit Block and rescue/cancellation are
different checkpoints. Runic Dome hides actual intent/derived previews, not
already-observed moves; hidden state must not leak through legality or menus.

All nine named fixed Status/mutation sites are explicit here:

| Site | Requested output and destination |
|---|---|
| Sentry Bolt | Dazed2 → discard |
| Acid(M) Spit | Slimed1 → discard, after queued hit |
| Acid(L) Spit | Slimed2 → discard, after queued hit |
| Spike(M) Flame Tackle | Slimed1 → discard, after queued hit |
| Spike(L) Flame Tackle | Slimed2 → discard, after queued hit |
| Slime Boss Goop | Slimed3 → discard; no hit |
| Hexaghost Sear | Burn1 → discard, upgraded iff its own Inferno flag; after queued hit |
| Hexaghost Inferno creation | Burn+3 → discard at later BurnIncrease gate, after six queued hits |
| Hexaghost Inferno mutation | existing draw/discard Burns upgraded; no obtain/draw event |

These are fixed Status producers, not random character/Shard pools or permanent
deck acquisition. Dazed is Ethereal, Slimed is a cost1 Status, Burn/Burn+ have
turn-end2/4 damage; creating them does not itself count as drawing them or playing
a Skill. Named small-count discard paths commit pile membership when the effect
is constructed; subsequent soul travel is visual, not a second insertion. This
refines the older “wait for soul travel” uncertainty without asserting all
callbacks/effects universally finish on combat ending.

## 10. Exact unsupported fields and future acceptance discriminators

Local records above do not need an unpublished roster/AI import. The following
are required **remaining composition fields**, not alternative approximate values
for any row. Preserve them as blockers where a complete execution result depends
on them. The existing [execution report](2026-10-18-act1-execution-closure.md)
explains why a pending-action list alone cannot certify quiescence.

| ID | Exact remaining obligation | Future discriminator — **not executed gameplay** |
|---|---|---|
| EN-U1 callback/origin composition | Complete legal card-origin/cost/copy and transitive power/relic callback ordering across Enrage, Sharp Hide, Curl Up/Angry, Poison and Fatal; direct-family scans do not certify every descendant callback. | Repeated/autoplay Attack versus multihit Attack; lethal/surviving/blocked damage; split versus Feed/Hand of Greed kill. |
| EN-U2 terminal arbitration | Full prevention/heal modifiers, alternate lethal origins, damage-all/per-hit death ordering, pending/current/pre-turn/card/monster/effect work and legal suspended-choice endings; retained work is not guaranteed execution. | Final Fungi versus nonfinal Thorns kill; six-hit rescue; split in flight; player death while later heal/Status work exists. |
| EN-U3 timing and theft | Original timer/FAST_MODE/update-delta outcome, including Reaper heal cancellation and legal asynchronous interposition between Looter ledger/theft; ordinary queue has no demonstrated intervening player decision. | Ledger-only checkpoint versus actual theft; Guardian retaliation ahead of queued Reaper heal with later endBattle clear. Do not assert heal loss/success from static ordering alone. |
| EN-U4 reward/continuation durability | All event/combat reward producer modifiers, nested relic/potion/card obtain suspension, death/escape return ownership and key-effect completion; a reward entry/local COMPLETE is not drained durable acquisition. | Searched relic then ambush; all-escaped versus mixed Thugs; offered/unclaimed/obtained Emerald Key; Mushroom reward interrupted by nested obtain. |
| EN-U5 initialized random joints | Root/history-conditioned seeded Java shuffle measures for boss/all-seen and Dead Adventurer categories, and their coupling to earlier draws/profile. A specified call/seed source is not a certified uniform permutation. | Same visible family with different prior history; preserve category depletion and boss discovery rather than resampling independently. |
| EN-U6 empty marker | Root-seed witness or exclusion proof for zero assigned elites, plus whole-spec fidelity treatment if that branch is encountered. | Supplied-choice empty fixture is administrative map failure, not executed root reachability or defeat/retry. |

Additional local future cases: all25 kind identities and move payloads; HP/setup
endpoints and separate Louse rolls; every primary interval endpoint/redirect in
§3; selection interrupted before execution; initial Shield alone; Wizard's first
two versus later three charges; asleep/event Lagavulin and Artifact1 Siphon;
Guardian threshold-crossing queued multihit; boss70→further-damaged split with
fresh child max/current and no same-round child action; Divider capture across
player HP changes; Inferno pile/checkpoint distinctions; all three elites × four
burning branches and Insect rounding. These are specification obligations, not
tests asserted to have passed.

**UNEXECUTED numeric/application discriminators:** Strength999 with an accepted +3
request remains999; Strength−999 with an accepted −1 request remains−999;
construction outside either Strength endpoint clamps at that endpoint. Block995
with processed gain7 ends at999, and Block999 with a positive processed gain
remains999, for an enemy as well as the player. Fractional processed gain is
floored before addition and capping. Preserve the original requested +Str/+B
amounts separately from applied state; test modifier/prevention paths separately.
These supplied-state future cases do not certify natural reachability or execute
gameplay. They do not cap HP, other powers, histories or Guardian quotas.

## 11. Source-claim ledger and actual verification

The principal retained evidence collection is
`inspection/2026-10-18-act1-java-blockers-59a62fb2/enemies/` in the private
verification repository. Later reviewed caller/effect facts are in
`verification/2026-10-18-act1-followup-v1/execution/`. This lane's private ledger
and new intent inspection are under
`verification/2026-10-19-act1-spec-closure-v1/content-enemies/`.
These are evidence locators, **not broken public hyperlinks or required imports**.

| Public claims | Exact source member family / evidence grade |
|---|---|
| All25 HP/setup/payload/move-ID/intent records | Named `monsters.exordium` classes' constructors, prebattle, getMove and takeTurn; retained static reconstructions, with earlier specifically reviewed bytecode for material comparisons/order |
| Integer supports, Louse setup, finite float masses | `AbstractMonster.setHp`, game `Random.random/randomBoolean`, `RandomXS128.nextInt/nextLong/nextFloat/nextBoolean`; retained source plus reviewed bounded-rejection/grid bytecode |
| Strength/Block application bounds and Mad/Sneaky continuation exceptions | `StrengthPower` constructor offsets27–58, stackPower68–99, reducePower45–76; `AbstractCreature.addBlock` offsets187–235; retained `GremlinWarrior.takeTurn` and `GremlinThief.takeTurn` SetMoveAction producers, with `GremlinFat.takeTurn` RollMoveAction contrast; personally read source/bytecode, not runtime |
| Selection history and conditional AI | `AbstractMonster.setMove/lastMove/lastTwoMoves/rollMove`; JawWorm/SlaverRed getMove and named slime/Louse/Slaver/Fungi/Nob getMove; reviewed source and named Jaw/Red/history offsets |
| Reactions, wake, split and Guardian | Named enemy damage/changeState/takeTurn; CurlUp/Angry/Anger/SharpHide/SporeCloud/Ritual powers; ApplyPowerAction, Artifact, SpawnMonsterAction, SuicideAction, MonsterGroup.queueMonsters; retained source and selected bytecode, not runtime |
| Looter accounting and terminal limits | `Looter.takeTurn/die`, its two anonymous ledger actions, `DamageAction.update/stealGold`, room stolen-reward/all-escaped handling; reviewed follow-up caller and manager/animation paths |
| Factories and encounter queues | `MonsterHelper.getEncounter` Act1 branches and named small-slime/gremlin/wildlife/humanoid helpers; `Exordium.generate*`, `AbstractDungeon.populate*`, `MonsterInfo.normalizeWeights/roll`, room departure; retained source |
| Burning and key offer | `AbstractPlayer.preBattlePrep`, room update/applyEmeraldEliteBuff, `MonsterRoomElite.dropReward/addEmeraldKey`, IncreaseMaxHpAction, AbstractCreature.increaseMaxHp, MathUtils.round, PreservedInsect; retained source and reviewed rounding casts |
| Event combats, seen and fixed Status sites | DeadAdventurer/Mushrooms constructors/buttonEffect, AbstractEvent.enterCombat, boss prebattle and UnlockTracker; Hexaghost/BurnIncrease/discard effects; reviewed source/caller follow-ups |
| Preview versus payload | Fresh `AbstractMonster.createIntent()V` offsets0–96; `applyPowers()V` offsets42–118; `calculateDamage(I)V` offsets44–264, plus retained DamageAction producer/reference usage; targeted CFR and personally read javap, no original class execution |

Actual checks in this lane: all four archive/tool identity pins checked before
and after static inspection; **84** exact unique historical ZIP entries matched
their recorded sizes/hashes; **27** retained enemy source files (25 classes plus
two Looter anonymous actions) were inventoried. Initial inspection extracted one
unique AbstractMonster class and ran one javap plus three targeted CFR selections,
all exit0 with Java option injection variables removed. This is evidence reuse,
not a new all-class or all-transitive scan.

Six private stdlib tests passed, covering pin mismatch, missing/duplicate ZIP
entries, output escape/overwrite/symlink rejection, Java environment stripping,
finite float-grid counts and record-identity rejection. Failure-first output is
retained (initially three assertion failures and one not-implemented error; two
later assertion failures for the added checks). These are helper/arithmetic
tests, not game behavior validation. All25 single-record omission mutations,
exact rational examples, local links, whitespace, privacy and one-path publication
checks are recorded privately. An initial inventory-check attempt included the
separate event table and was rejected; the corrected check scopes the25 kind
records explicitly. No product build/test, game probe, benchmark or training ran
here.

**Review correction EN-R1/EN-R2:** Strength's resulting-amount clamp and the
all-creature addBlock cap are now explicit without changing requested payloads;
Mad/Sneaky repeats explicitly override the RollMove default. The full independent
report and actual retained source/bytecode were read before adopting either fix.
Correction inspection checked all four pins before/after, extracted two exact
unique classes, and ran two javap calls plus one targeted addBlock CFR selection,
all exit0 with Java injection variables stripped. Three document regression checks
failed before correction and passed afterwards; the original six helper tests
also passed. These are document/helper checks only. Initial artifacts and failure
logs remain preserved; EN-U1–EN-U6 and independent exact-head recheck remain open.

The documented current-wave baseline exception concerns only the two already
reproduced CUDA-environment test failures in documentation-only integration;
it is not a green suite, new-failure waiver or permission for product changes.
Independent review and controller/main readback remain required at the exact PR
head; the Bead remains in progress. Whole-spec approval, implementation-plan
review and implementation-start authorization remain separate gates.
