# Ironclad A0 Acts 2 and 3 — initial research findings

## Status and evidence boundary

This is a **supporting research draft**, not a complete specification, an
implementation plan, or permission to begin implementation. Tracking: `spire-mdc`.
Scope is standard, unmodded, fully unlocked, Act-4-enabled Ironclad A0, carrying
forward a legally reached Act 1 state. It does not require collecting keys or
reaching the Heart, and does not import another character's starting resources.

Findings are from targeted static reconstruction and bytecode inspection of the
owned archive with SHA-256
`cfad868ac8d65a88e71a0bf096fb09f78811e553effe0787c5309a655e081673`.
Inspection used pinned Temurin 21.0.12.1+1 and CFR 0.152. Original game classes,
initializers and gameplay were **not executed**. No product tests or builds ran.
Detailed source, inspection outputs and identity records remain private. The
archive's reported Steam origin does not independently establish its release channel.

The pass establishes act-local setup, nominal encounter pools and selected factory
details, the twenty ordinary event entry bodies, and the principal Act 3 route
gate. Reading those bodies does not establish transitive combat, acquisition,
screen-return, death/rescue or timing closure.

## 1. Carryover and act construction

The new-act constructor receives the existing player and remaining one-time event
list. It does not create a fresh Ironclad deck. The shared setup advances the act,
clears path and local event/shrine/encounter/boss lists, resets question-room
probability state and the shared potion-drop modifier, and requests a heal. At A0
that request is the player's maximum HP through the normal heal method—not an
unconditional assignment of current HP to maximum HP. Applicable healing rules
must still be respected.

Encounter queues and the boss list are constructed before the act-specific map.
Ordinary events, repeatable shrines, card pools and potion setup are rebuilt;
the surviving one-time event list is passed forward rather than replenished.
The setup also conditionally advances the original card RNG counter to particular
250-counter boundaries. These original cursor operations are reference facts,
not newly approved native seed/cursor-parity requirements.

| A0 setup property | Act 2 | Act 3 |
|---|---|---|
| Original fresh-map seed expression | Run seed plus act number times 100 | Run seed plus act number times 200 |
| Standard act number at construction | 2 | 3 |
| Resulting original seed offset | 200 | 600 |
| Shop/rest/event/elite room parameters | 0.05 / 0.12 / 0.22 / 0.08, binary32 | Same |
| Ordinary treasure-room parameter | Zero; this does not eliminate the fixed chest | Same |
| Small/medium/large chest weights | 50 / 33 / 17 | Same |
| Common/uncommon/rare relic weights | 50 / 33 / 17 | Same |
| Colorless rare parameter | 0.3, binary32 | Same |
| General card-upgrade chance parameter | 0.25, binary32 | 0.5, binary32 |

These are constructor parameters, **not independent per-node probabilities** or
universal rules for every card producer. Shared map constraints, assignment order,
fixed rows, burning-elite selection and exceptional cases still apply. The Act 1
zero-elite seed-image question and exact shuffle/input-measure obligations are
not answered merely by changing the map seed offset.

Evidence locators: `AbstractDungeon` fresh-act constructor and
`dungeonTransitionSetup`; `TheCity` and `TheBeyond` constructors and
`initializeLevelSpecificChances`; `CardCrawlGame.getDungeon`.

## 2. Encounter selection is history-conditioned

Both acts initially request two weak encounters, then construct a separately
filtered first strong encounter, then append twelve more strong encounters.
Thus a normally completed construction contains **fifteen regular queue entries**,
not fourteen. Ten elite entries are requested separately. These initialization
sizes are not simulator encounter limits.

Ordinary queue population rejects a candidate matching either of the last two
existing entries where those entries exist. Elite population rejects only an
immediate repeat. The special first-strong selection uses its act-specific
exclusion list instead of the ordinary last-two rule. Rejected proposals consume
draws. Do not replace these procedures with independent encounters or apply the
last-two restriction to every boundary indiscriminately.

### Act 2 pools

| Queue family | Nominal encounters and source weights |
|---|---|
| Weak | Spheric Guardian, Chosen, Shell Parasite, 3 Byrds, 2 Thieves; weight 2 each |
| Strong | Chosen and Byrds 2; Sentry and Sphere 2; Snake Plant 6; Snecko 4; Centurion and Healer 6; Cultist and Chosen 3; 3 Cultists 3; Shelled Parasite and Fungi 3 |
| Elite | Gremlin Leader, Slavers, Book of Stabbing; weight 1 each |
| Boss candidates | Automaton, Collector, Champ |

The last weak Spheric Guardian excludes Sentry and Sphere from the first strong
draw; 3 Byrds excludes Chosen and Byrds; Chosen excludes both Chosen and Byrds and
Cultist and Chosen. Other last-weak identities add no first-strong exclusions.

### Act 3 pools

| Queue family | Nominal encounters and source weights |
|---|---|
| Weak | 3 Darklings, Orb Walker, 3 Shapes; weight 2 each |
| Strong | Spire Growth, Transient, 4 Shapes, Maw, Sphere and 2 Shapes, Jaw Worm Horde, 3 Darklings, Writhing Mass; weight 1 each |
| Elite | Giant Head, Nemesis, Reptomancer; weight 2 each |
| Boss candidates | Awakened One, Time Eater, Donu and Deca |

The last weak 3 Darklings excludes 3 Darklings; 3 Shapes excludes 4 Shapes. The
Orb Walker case lists Orb Walker as an exclusion, but Orb Walker is absent from
this strong pool. Do not invent an additional exclusion to make that case effective.

Weights are normalized after sorting by weight, using binary32 arithmetic;
selection uses a strict comparison against accumulated normalized weights.
The tables give source weights, not proof of exact rational outcome probabilities
under an initialized or history-conditioned RNG measure.

In the fully boss-seen profile branch, each boss candidate list is shuffled using
a Java-seeded shuffle after encounter queue construction consumes the same original
monster RNG. Three candidates do not establish a uniform first-boss law. Boss room
entry removes the first boss-list entry. The A20 second-boss branch exists but is
outside this A0 scope; it must not create an A0 second boss.

Fully unlocked is not synonymous with every boss having been seen. For alternative
discovery histories, Act 2 prioritizes the first unseen boss in Champ, Automaton,
Collector order; Act 3 uses Awakened One, Donu and Deca, Time Eater order. That
branch creates a single forced identity and duplicates it in the internal list;
it does not grant two A0 boss fights. Profile defaults and discovery-update
boundaries still require explicit reconciliation rather than inferring them from
unlock flags.

Evidence locators: act classes' `generateMonsters`, weak/strong/elite generation,
`generateExclusions`, `initializeBoss`; `AbstractDungeon.populateFirstStrongEnemy`
and `populateMonsterList`; `MonsterInfo.normalizeWeights` and `roll`;
`MonsterRoomBoss.onPlayerEntry`.

## 3. Encounter names are not sufficient specifications

Factory inspection establishes several concrete distinctions:

- Act 2's 2 Thieves is Looter plus Mugger; Slavers is Blue Slaver, Taskmaster,
  Red Slaver in that order. Gremlin Leader starts with two generated gremlins and
  the leader. Summoning, replacement, escape and death rules remain separate work.
- The second Colosseum encounter called Colosseum Nobs is **Taskmaster plus one
  Gremlin Nob**, not two Nobs.
- Act 3's 3 Shapes and 4 Shapes select ordered slots without replacement from two
  Repulsors, two Exploders and two Spikers. Enemy constructors run between slot
  selections; this is not permission to reorder all composition draws before all
  constructor draws. The mixed sphere encounter uses a different helper for its
  two shape slots, not the same shrinking six-item pool.
- The live Mysterious Sphere event constructs **2 Orb Walkers**. A different
  encounter factory branch named Mysterious Sphere constructs two shapes and an
  Orb Walker; name similarity does not make that the event's fight.
- Awakened One's group is two Cultists followed by Awakened One. Donu and Deca's
  factory group order is Deca then Donu. Group order and instance identity can
  matter for targeting and action order; printed encounter names cannot replace them.

These facts do not close any enemy's HP, initial intent, AI, power, damage,
revival, summons, escape, victory detection or reward law. In particular Darkling
revival, Awakened One phases, Transient survival, Writhing Mass intent changes,
Time Eater interruption, minions and retained cards/resources need explicit audits.

Evidence locators: `MonsterHelper.getEncounter`, `spawnShapes`, `getAncientShape`;
`Colosseum.buttonEffect`; `MysteriousSphere` constructor.

## 4. Event pools and local branch inventory

Act 2 initializes thirteen ordinary event IDs and Act 3 initializes seven. Both
rebuild six ordinary shrines: Match and Keep, Wheel of Change, Golden Shrine,
Transmogrifier, Purifier and Upgrade Shrine. Remaining run-wide one-time events
are an additional, eligibility-filtered source—not part of those 13/7 counts.
Selected IDs are removed from their applicable pools; a new act does not recreate
already consumed one-time events.

Important eligibility distinctions include Beggar's 75-gold gate, Colosseum being
strictly beyond half the map's row count, and Moai Head requiring Golden Idol or
current/max HP at most one half. One-time additions include Designer and Duplicator
in Acts 2/3, Knowing Skull and N'loth in Act 2, and Secret Portal in Act 3.
Their additional gold/HP/relic/time requirements still apply. Face Trader is not
an Act 3 candidate. The full filtered inventory, empty-pool behavior and history
remain source-specific obligations.

The following records **local entry-body facts**, not complete event implementations.
Damage/heal amounts are requests through the stated game pathways; relics,
prevention, rescue, death and pending obtain effects can change actual results.
Percentage calculations use the original binary32 operations and rounding sites,
not a generic exact-rational percentage helper.

### Act 2 ordinary events

| Event ID | Local A0 branch facts / required distinction |
|---|---|
| Addict | Pay 85 gold for a random screenless relic, steal one with Shame, or leave. Relic selection precedes payment in the paid branch. |
| Back to Basics | Remove one eligible non-bottled purgeable card, or upgrade upgradeable cards tagged starter Strike/Defend. This is not every basic card, nor every card containing “Strike.” The remove branch checks for a nonempty group before opening selection. |
| Beggar | Pay 75 gold before a later purge-selection stage, or leave. Selection completion removes the chosen instance. Its empty-group behavior cannot be borrowed from Back to Basics. |
| Colosseum | First Slaver fight disables rewards; on return, leave or take the Taskmaster/Nob fight with explicit rare relic, uncommon relic and 100-gold rewards and elite-trigger flag. These are two combats under one event parent. |
| Cursed Tome | Continued reading requests successive HP losses of 1, 2 and 3; the last choice requests 10 more to obtain a book, or 3 more to stop. Missing Necronomicon/Enchiridion/Nilry's Codex candidates are selected, with Circlet if none remain. Damage precedes the obtain-reward setup. |
| Drug Dealer | Obtain J.A.X., select two purgeable cards to transform, or obtain Mutagenic Strength, substituting Circlet if already owned. Transform removal, generation and queued obtain are distinct boundaries. |
| Forgotten Altar | Golden Idol exchange; or increase max HP by 5 before damage based on the earlier captured 25%-of-max-HP amount; or obtain Decay. Idol slot replacement is not a generic remove-and-append operation. |
| Ghosts | Reduce max HP by a captured ceiling of half maximum, capped below the whole maximum, then queue five Apparitions; or decline. |
| Masked Bandits | Pay all current gold, including the zero-gold case, or fight the already constructed bandit group. The fight explicitly adds 25–35 gold and Red Mask, or Circlet if Red Mask is owned. |
| Nest | Gain 99 gold, or request 6 damage before queuing Ritual Dagger. |
| The Library | Choose from twenty distinct-ID candidates with rarity re-rolled on duplicate proposals and preview-obtain hooks; or request healing based on rounded binary32 0.33 times maximum HP. The selected offer is copied before its obtain effect. |
| The Mausoleum | Opening draws a boolean for Writhe and then selects a random screenless relic; or decline. A curse offer is not proof that a curse reaches the deck. |
| Vampires | Pay captured ceiling of 30% maximum HP, capped below the whole maximum, or give Blood Vial when present; remove all starter-Strike-tagged cards and queue five Bites. This is not replacement of all attacks or exactly five removed Strikes. Decline remains legal. |

### Act 3 ordinary events

| Event ID | Local A0 branch facts / required distinction |
|---|---|
| Falling | Samples one non-bottled instance for each present Attack/Skill/Power type during construction; later offers those fixed instances in Skill/Power/Attack order. Missing-type options are disabled; no eligible types produces a leave outcome. This is not an arbitrary purge choice or a resample at confirmation. |
| MindBloom | Fight a shuffled Act 1 boss for explicit 50-gold and rare-relic rewards; upgrade every upgradeable deck card and obtain Mark of the Bloom; or choose a floor-dependent gold/curse versus heal/curse branch. The visual twenty-card limit does **not** limit actual upgrades. |
| The Moai Head | Directly reduce maximum HP by the captured rounded 12.5% amount, clamp in its local order, then request healing; exchange Golden Idol for 333 gold; or leave. The first branch does not call the generic max-HP-decrease method. |
| Mysterious Sphere | Fight the constructor-created two Orb Walkers for explicit 45–55 gold and a screenless rare relic, or leave. Enemy construction occurs before the fight choice. |
| SensoryStone | Request one, two or three colorless card rewards; the two/three branches construct/open rewards before requesting 5/10 HP loss. This does not mean collection occurs before damage. Memory-text selection consumes an original misc-RNG seeded shuffle before reward construction. |
| Tomb of Lord Red Mask | Owning Red Mask enables 222 gold; without it, paying all current gold obtains Red Mask; leaving remains available. Do not impose an invented positive-gold minimum. |
| Winding Halls | Request rounded 12.5%-of-max damage then queue two Madness; request rounded 25%-of-max healing then queue Writhe; or decrease max HP by rounded 5%. Captured amounts and per-branch operation order differ. |

Mind Bloom's third option uses global floor number modulo 50: at most 40 offers
999 gold plus two Normalities; later floors offer a maximum-HP heal request plus
Doubt. Its boss shuffle uses the original misc RNG, not the act boss-list selection
site. The precise initialized/history-conditioned shuffle laws remain unproved.

Evidence locators: act event/shrine initializers; `AbstractDungeon.generateEvent`,
`getEvent`, `getShrine`; the named event constructors, `buttonEffect`, `update` and
local selection helpers; `CardHelper.hasCardWithType` and `returnCardOfType`.

## 5. A newly explicit headless clock obligation

Secret Portal's selection eligibility requires Act 3 and original run playtime
**at least 800 seconds**. Shared dungeon updates add frame delta to that playtime
unless the clock is stopped. This is not a floor count or a combat-turn threshold.
Accepting the portal later schedules a boss-room transition; it does not instantly
kill the boss, grant rewards or complete the act. Declining remains possible.

A fast headless environment cannot silently substitute simulator CPU time, freeze
time at zero, or always enable the portal. A written fidelity decision must explain
the authoritative time input, replay, permissible waiting/decision timing and public
observation. Possibilities to evaluate include explicit recorded logical elapsed
time or a fully specified canonical timing model; neither is approved here.
Animation collapse alone does not establish equivalence when time gates content.

Evidence locators: `AbstractDungeon.update` and `getShrine`;
`SecretPortal.buttonEffect`. No timed original-runtime probe ran.

## 6. Boss rewards, Act 3 clear and the Act 4 gate

Act 2 boss combat is followed by the boss treasure/relic stage. Its next dungeon
is Act 3. Take/skip and any nested relic/card choices remain real work before a
resolved handoff; killing the boss is not that handoff.

At A0, proceeding from the Act 3 boss routes into the Spire Heart victory event,
not another ordinary boss chest. That event checks final-act availability and all
three **resolved** key flags. With all keys it can enter the door transition,
whose event-mode exit names TheEnding as the next dungeon. Without the keys the
event follows its non-Heart ending, eventually setting player dying/dead flags
and creating a death screen. Those flags do not, by themselves, specify a native
combat-defeat classification or the future full-run learning reward.

Accordingly, the eventual result vocabulary must distinguish ordinary death,
Act 3 clear without entering Act 4, Act-4-ready handoff and actual Heart victory.
Those are semantic distinctions to design, not approved new API enum names or a
change to the already scoped Act 1 reward contract. Skipping keys must not be
silently forbidden or converted to an administrative error. Three keys do not
by themselves establish Heart victory.

Evidence locators: `ProceedButton.update` and transition methods;
`TreasureRoomBoss.getNextDungeonName`; `VictoryRoom.onPlayerEntry`;
`SpireHeart.buttonEffect` and `goToFinalAct`; `DoorUnlockScreen.exit`;
`CardCrawlGame.getDungeon` and gameplay transition handling.

## 7. What this does not close

- Enemy-by-enemy A0 constructor, AI, intent, summon/revival/escape and action laws,
  and their complete reachable card/power/relic interactions.
- Full act-local and carried-special event eligibility/branch closure, exceptional
  inventories, death/rescue/prevention, nested screen returns and obtain dispatch.
- Expanded relic availability, act-gated spawning, shop/reward generation and
  persistent pool depletion across both acts. Act 1's 141-candidate audit is not
  automatically a later-act roster.
- Exact map seed-image/empty-set policy, binary32 threshold pushforwards and all
  initialized/history-conditioned shuffle measures and correlations.
- Headless run-time semantics, terminal/profile boundaries and complete native
  API/observation/replay/batch design. Hidden future queues, unrevealed offers and
  future boss-list order must not leak into observations.
- Safe original-game execution, independent runtime comparisons, a complete Acts
  2/3 specification, the whole Act 1 specification, or implementation authorization.

No application dependencies, simulator behavior, combat-v1 protocol pins or
implementation gates change in this research document. PR publication is not
integration or acceptance; fresh independent review remains required.
