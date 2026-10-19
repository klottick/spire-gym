# Act 1 execution: direct inventories and resolution boundaries

Date: 2026-10-19 · Bead: `spire-gli.2.3.3`

**Status: new static findings and bounded closure; independent review pending.**
This is a specification input, not an implemented scheduler, executed gameplay
test, whole-spec approval, or claim that every transitive content rule is closed.
It advances the [previous execution report](2026-10-18-act1-execution-closure.md)
with instruction-level direct inventories and concrete branch/commit distinctions.

## 1. Evidence and scope

The owned archive is `desktop-1.0.jar`, SHA-256
`cfad868ac8d65a88e71a0bf096fb09f78811e553effe0787c5309a655e081673`.
Upstream build/channel identity is not independently established. New analysis
indexes declarations, inheritance, actual direct invoke instructions, and field
access instructions in all **2,306 game classes**, including inner classes.
Non-game libraries and assets are not reconstructed. Targeted source reconstruction
and independent `javap` comparisons check selected dependencies. Original classes,
initializers and runtime were **not executed**.

Here, **direct complete** means the exact declared method family or direct invoke
sites in those classfiles, with owner, descriptor and instruction offset retained
privately. It does not mean reflection/native/bootstrap closure, legal reachability
of every archive class, or complete downstream behavioral proof. There are 25
`invokedynamic` sites, separately recorded rather than counted as direct calls.
Deprecated/test/later-act classes are negative context, not newly legal Act 1 content.

Evidence keys below identify methods, not unpublished document dependencies:

| Key | Static evidence boundary |
|---|---|
| E1 | All-class declaration/invoke/field index; exact unique ZIP entries and class hashes |
| E2 | `AbstractCard`: legality, copy helpers, cost methods; eight card `canUse` overrides |
| E3 | `GameActionManager`: queue selection, play history, end-turn, discard counting, filter/clear |
| E4 | `AbstractPlayer`: use, draw, refresh, damage/heal, start-turn dispatch |
| E5 | `CardGroup`: pile movement, exhaust, reset-before-moving, other-card notifications |
| E6 | Draw/discard/exhaust/retain actions; room and monster-group turn dispatch |
| E7 | Ten named X actions; Transmutation's card entry; selected cost-writing producers |
| E8 | Fairy, Tail, creature healing/max-HP, heal overrides, alternate terminal writers |
| E9 | Previously reviewed manager/room/dungeon, monster death/escape, Reaper and repeat-power bodies |
| E10 | Correction check: Setup/Forethought actions, selected-group ordering and pile insertion; Masterful Stab construction/notification/copy, player damage dispatch, True Grit exhaust and Exhume retrieval |

The private ledger retains exact source/bytecode locators, full direct inventories,
commands, failed attempts and helper checks. Public facts here are independently
worded; no original source, translated implementation, helper or fixture is published.

Checks actually run: seven private standard-library helper tests pass after
recorded failure-first runs. Negative controls cover missing/duplicate ZIP entries,
truncated/invalid class instructions, operand bytes that resemble calls, output
escape and overwrite. Eighty fresh disassembly records across 77 selected classes
match 4,819 invoke and 8,451 field instructions by offset/opcode/name/descriptor
against the index. These checks validate the static evidence machinery, not
gameplay, every caller branch, or the future cases below.

The SI-I1/SI-I2 correction additionally checks retained source anchors against
14 fresh static disassemblies; 12 corresponding outputs exactly match the
independent supporting review's records. Four failure-first document regression
checks and the seven existing helper checks pass. These are evidence/document
checks, not execution of the correction's six new gameplay obligations. Initial
evidence and old supporting drafts are preserved unchanged.

## 2. Legality: a finite direct inventory

### 2.1 All card `canUse` overrides

The card inheritance closure contains 439 classes, including the abstract base and
historical classes. Exactly nine declare `canUse`: the base and these eight
overrides. There is no additional card override hidden by an inherited method.

| Override | Additional condition / relationship to the base gate |
|---|---|
| Secret Technique | Base must pass; draw pile must contain a Skill |
| Secret Weapon | Base must pass; draw pile must contain an Attack |
| Grand Finale | Base must pass; draw pile must be empty |
| Signature Move | Base must pass; no other Attack object in hand |
| Clash | Base must pass; every card in hand must be an Attack |
| Reflex | Always rejects play |
| Tactician | Always rejects play |
| Deus Ex Machina | Always rejects play |

The base rejects negative-unplayable Statuses without Medical Kit and Curses
without Blue Candle; then checks target/combat availability and affordability gates.
The target check rejects a dying supplied enemy for enemy-targeting forms, or a
basically-dead monster group. It is **not** itself a universal non-null-target check.

Affordability first checks turn-ended, power permission, Entangled for Attacks,
relic permission, blight permission, and hand-card permission. Only its final
energy comparison accepts sufficient energy **or** free play **or** autoplay.
Normality's hand permission rejects at three recorded plays; Velvet Choker's relic
permission rejects at counter six. Neither adds an autoplay exemption. The only
power `canPlayCard` override is `NoSkillsPower`; the only card `canPlay` override
is Normality. Blight TimeMaze and relic VelvetChoker are the other non-base
`canPlay` declarations. These are different dispatch families. [E1–E4]

### 2.2 Direct caller accounting

| Family | Complete direct accounting in the indexed game classes |
|---|---|
| Card `canUse` | 13 invoke sites: manager selection; group glow; five super-calls; player targeting (2), drag (3), release (1) |
| Potion `canUse` | Separate no-argument family: six declarations, seven invokes; not card legality |
| `hasEnoughEnergy` | One declaration, five invokes: base `canUse`, energy rendering, group any-playable query, two drag paths |
| `freeToPlay` | One declaration, five invokes: manager selection, affordability, energy rendering, displayed cost, player use |
| `canPlayCard` | Two declarations including base; one direct dispatcher in affordability |
| `canPlay` | Six declarations across cards/relics/blights; three direct dispatch sites in affordability |
| `queueExtraCard` | One declaration, **zero direct invocation sites** |

The last row closes the direct-caller question, not a reflection impossibility
proof. The five repeat powers implement their own repeat producers; they must not
be modeled as calls to an assumed universally used `queueExtraCard` helper. [E1,E3,E9]

## 3. Cost state and copy state

### 3.1 Mutation interfaces, not a single cost number

There is one declaration each of `updateCost`, `setCostForTurn`,
`modifyCostForCombat`, `upgradeBaseCost`, and `resetAttributes`, with respectively
7, 23, 11, 55 and 9 direct invokes. No subclass overrides those helpers. This is
not the whole mutation inventory: direct field writes matter too. [E1,E2,E7]

| Interface / producer | Established distinction |
|---|---|
| Turn-cost setter | Acts only on an already nonnegative turn cost; clamps a supplied negative ordinary cost to zero; does not turn X/unplayable into ordinary zero |
| Combat-cost modifier | Handles positive turn cost differently from a nonpositive turn cost; the latter can preserve turn-zero while changing nonnegative combat cost |
| Cost update | Preserves the existing combat/turn-cost difference when changing combat cost, with nonnegative clamps; ordinary Curses/Statuses are excluded except Pride/Slimed |
| Base-cost upgrade | Preserves the turn/combat difference when turn cost is positive; already-zero turn cost stays zero |
| Attribute reset | Restores turn cost from combat cost and clears turn modification; does not restore printed cost or clear every play flag |
| Confusion | On an eligible nonnegative-cost draw, samples 0–3; changes both costs only when the sampled value differs from combat cost; clears one-shot free play even when the costs compare equal |
| Randomize-hand-cost | Eligible nonnegative combat costs; equal sampled combat cost leaves a different turn cost untouched; unlike Confusion, does not clear the free flag |
| Enlightenment | Reduces turn costs above one; upgraded/combat mode separately reduces combat costs above one |
| Madness | Prefers cards with positive turn cost; only absent such cards uses positive combat cost; sets both costs to zero on a qualifying instance |
| Chrysalis / Metamorphosis | Generated cards with positive combat cost receive direct combat-and-turn zeroing; X/nonpositive costs are not rewritten by this branch |
| Blood for Blood | Explicit cost-writing upgrade and history-aware fresh-copy path; not just base upgrade assignment |
| Corruption | Draw callback uses turn-cost setter; refresh and application sites also use combat-cost modification; on-use sets exhaust and ordinary payment separately exempts Skills |

The complete direct cost-field writer set outside the base is EnlightenmentAction,
MadnessAction, RandomizeHandCostAction, Chrysalis, Metamorphosis, BloodForBlood's
upgrade, and ConfusionPower. Copying and base initialization/reset are separate
base-class writers. `updateCost` callers are ForceField (two sites), MasterfulStab,
BloodForBlood (two sites), and two deprecated cards. Combat-modifier callers are
ApplyPowerAction (four pile sites), ReduceCostAction (two), BendAction (three),
EstablishmentPowerAction, and player refresh. The complete 23-site turn-setter
caller set is BetterDiscardPileToHandAction (2), ReduceCostForTurnAction,
ApplyBulletTimeAction, DiscoveryAction (2), ExhumeAction (2), TransmutationAction,
ForeignInfluenceAction, PathVictoryAction, WhiteNoise, deprecated StepAndStrike,
Distraction, Eviscerate (3), InfernalBlade, CorruptionPower, Enchiridion,
MummifiedHand, and ShowCardAndAddToHandEffect (2). Full conditional laws of all
generation/retrieval producers are **not** proved by these counts.

Thus there is no context-free total ordering of “Confusion versus upgrade versus
Corruption.” Resolve actual calls in their current callback order. In particular,
draw's own-card callback precedes power callbacks, and refresh follows them. Power
collections are sorted on new application; Confusion specifies priority zero.
The proof obligation is the concrete ordered collection and its mutations, not a
new fixed global priority table. [E2,E4,E7]

### 3.2 Complete direct copy distinctions

All 439 card classes declare `makeCopy` (including the abstract declaration).
Of the 438 concrete method bodies, 435 directly delegate to a kind constructor;
the three additional direct state-reading bodies are Eviscerate, Blood for Blood,
and Searing Blow. Eviscerate consults combat discard history; Blood for Blood
consults combat damage-event history; Searing Blow passes its upgrade count to
construction. Constructor behavior is a further dependency, not proved absent
because a copy method is short. [E1,E2,E7]

| Copy route | State contract supported by the helper bodies |
|---|---|
| Fresh-kind `makeCopy` | Kind-specific construction; not an all-fields snapshot |
| Stat-equivalent | Starts with fresh-kind construction and upgrade replay, then explicitly restores selected source statistics, costs, upgrade count, misc, bottle markers and one-shot free flag; does not copy every mutable play/destination flag |
| Same-instance helper | Stat-equivalent object plus source UUID; distinct object with shared source linkage, not an alias to the source object |
| Expunger override | The sole stat-equivalent override; additionally restores its X-related magic values and clones the description list |

There are exactly two stat-equivalent declarations and 78 direct invokes, and
one same-instance declaration and ten invokes. The latter are manager
`queueExtraCard`, two temporary hand/discard constructors' copy helpers, deck-group
construction, the five repeat powers, and Necronomicon (later-act negative context).
Do not erase permanent-instance linkage or equate shared UUID with physical object
identity. Temporary-copy provenance and permanent growth targets remain separate.

### 3.3 Per-instance cost/state submatrix

These source/bytecode-checked rules supersede the older Silent supporting draft's
unconditional Setup-zero and Masterful-Stab-global-history formulations (review
findings **SI-I1** and **SI-I2**). The older draft is unchanged. Here `cost` means
the instance's **mutable combat cost**, not an immutable printed/base cost;
`costForTurn` and `freeToPlayOnce` are separate state. [E2,E4,E5,E10]

| Instance state / operation | Established cost/state consequence | Identity / boundary |
|---|---|---|
| Positive combat cost, changed turn cost | Setup and Forethought grant `freeToPlayOnce` when `cost > 0`, even if turn cost is already zero or differs otherwise; not numeric zeroing of either cost field | Both singleton and selection-return branches act on the selected object |
| Zero combat cost, positive turn cost | Neither action grants a new free flag: the guard reads combat cost, not turn cost | In particular, a base-zero card with only a turn-cost increase is not made free by this rule |
| X or negative-unplayable cost | Neither action grants a new free flag for the negative sentinel | Movement is not permission to play; X capture/payment remains source-specific |
| Existing free flag / ordinary reset / use cleanup | A failed positive-cost guard does not clear a pre-existing free flag; `resetAttributes` restores turn cost from combat cost without clearing the flag; UseCardAction clears it after after-use callbacks, before destination handling | Other explicit writers, including Confusion, can clear it; this is not an unconditional until-play guarantee through every interaction |
| Masterful Stab construction / notification | Fresh construction starts at cost zero; each `tookDamage` notification calls the cost updater with +1 on that instance, preserving the updater's combat/turn-cost distinction and clamps | This is per-instance notification state, not a formula evaluated from a global combat HP-loss counter |
| Qualifying player-loss dispatch | Positive damage amount in combat dispatches to hand, discard, then draw; **not exhaust**. Zero/prevented loss supplies no notification through this branch | One notification per visited instance per qualifying event, not per HP point; not restricted to the `HP_LOSS` damage type |
| Exhaust → loss → Exhume | The exhausted Stab misses those notifications; Exhume returns the existing eligible object with no missed-loss catch-up | Both singleton and selected retrieval paths preserve that object; retrieval is not fresh construction or ordinary draw |
| Stab fresh-kind versus stat-equivalent copy | `makeCopy` constructs a fresh zero-cost Stab without reading combat-loss history; the stat-equivalent helper instead restores selected source costs/state after construction | Neither route is a universal global-history catch-up; other mutations still compose in actual order |

Setup sends the selected card to draw-top; Forethought sends it to draw-bottom.
Forethought's ordinary form requires one card when selection is available;
Forethought+ allows any number, including an empty choice. Its selected sequence
is ordered and iterated for bottom insertion: keep the source sequence and its
insertion effects, not a sorted set of identities. Different selected orders of
the same subset can change future draws. Neither movement is an actual draw or a
generic zero-cost rewrite. [E5,E10]

A source-static legal Ironclad cross-source discriminator is: obtain Masterful
Stab through a legal off-color route, select it for True Grit+ exhaustion, lose
positive HP while it remains exhausted, then return it with Exhume. With unrelated
cost mutations excluded, the missed event does not increase that object's cost.
The global-history formula would incorrectly add an increment. This is a static
counterexample, **not executed gameplay or an initialized-root-seed witness**;
prevention, intervening modifiers and selection/resume behavior remain separate
composition obligations. [E2,E4,E5,E10]

## 4. Origin, payment, captured X, history and destination

### 4.1 Common checkpoints

These checkpoints describe entered calls, not a promise to drain all their effects:

1. Manager installs autoplay status, optionally chooses a random target, and calls
   actual `canUse`. Its explicit `dontTriggerOnUseCard` alternative is a different
   branch, not an origin-wide legality waiver.
2. On acceptance it installs captured energy/ignore flags, invokes on-play
   notifications, then records normal play history unless notifications are suppressed.
3. A later enemy-target null/dead-or-escaped branch can omit player use **after**
   those history effects. A rejection at the earlier gate does not record them.
4. Player use recalculates card damage, adjusts applicable X capture, invokes
   `card.use`, constructs UseCardAction (whose on-use callbacks run immediately),
   issues hand other-card callbacks, removes the card from hand, then charges
   ordinary energy. Invoking `card.use` does **not** mean all damage/actions it
   enqueues have already resolved.
5. UseCardAction's first update calls player then monster after-use powers before
   destination handling. Purge and Power return early. Only the ordinary remainder
   reaches Spoon/exhaust/rebound/shuffle-back/return/discard handling. [E2–E5,E9]

| Origin / branch | Payment and captured value | History / cleanup consequence |
|---|---|---|
| Ordinary accepted hand use | Positive ordinary turn cost is charged after use-entry callbacks unless free/autoplay/Corruption-Skill exempt; X payment belongs to its action | Normal on-play history before use; normal destination rules |
| One-shot free hand use | Ordinary payment waived; not an exemption from the earlier permission gates | Normal history and callbacks; free flag later cleared by cleanup |
| Havoc top-card autoplay | Actual pile object moved through limbo; NewQueue captures then-current energy; autoplay avoids ordinary payment and makes X one-shot free | Still gated; exhaust-once requested, subject to separate Power/purge/Spoon branches |
| Mayhem / Distilled Chaos top-card autoplay | Same top-card/queue machinery with source-specific timing and exhaust argument | No general “draw” callback merely for top-card movement; different outer producers remain distinct |
| Double Tap / Burst / Amplify / Duplication / Echo repeat | Copy original `energyOnUse`; **explicitly set both `ignoreEnergyTotal=true` and `autoplayCard=true`** | Same-instance-copy linkage, purge cleanup; normal eligible repeated-play history, not a second cycling card |
| Omniscience | Removes the selected draw-pile object, marks it exhaust, queues that original and stat-equivalent copies through NewQueue autoplay | Original uses its exhaust route; additional copies are purge-on-use with distinct copy UUIDs, not the five powers' same-UUID route |
| Failed autoplay gate | No payload/payment or normal history; manager suppresses on-use notifications for cleanup | Cleanup can still process the producer's destination flags; not equivalent to a played zero-effect card |
| Accepted but later invalid enemy target | On-play/history already committed; player use and ordinary payment omitted | Limbo/fading handling is not normal UseCardAction destination processing |

NewQueue rejects duplicate **object** entries and queues a null end-turn marker
only if none exists. “Immediate” insertion is index one when a card is already
queued, otherwise append; repeated insertions at index one reverse their producer
order. Ordinary pending actions take priority over pre-turn actions, then card
queue work. Multiple outer top-card actions can therefore capture queue entries
before the inner card uses run. These are specific selection/insertion facts,
not a universal FIFO. [E3,E6,E9]

### 4.2 X is not “energy spent”

The ten inspected X actions distinguish captured energy, resolution-time energy,
Chemical X, upgraded amount and a captured free flag. Most use non-sentinel
capture in preference to current energy and add Chemical X's two. The ordinary
player-use path caps a requested X to current energy only when ignore-energy is
false. Autoplay makes the X card one-shot free before entering its payload.

| X producer | Additional action-level condition / distinction |
|---|---|
| Whirlwind / Skewer / Reinforced Body | Positive effective count queues individual damage/block operations; non-free payment consumes the current total when this action resolves |
| Malaise / Doppelganger / Tempest / Collect | Upgraded form adds one to effective count; positive-count payload and non-free payment branch |
| Multi-Cast | Entire effect/payment branch additionally requires a current orb; no-orb is not an automatic all-energy debit |
| Transmutation | Card use first raises capture to at least current energy; action adds Chemical X, creates ordinary turn-zero results, and pays current total only for positive count when non-free |
| Conjure Blade | Constructs Expunger even at zero effective count; non-free payment is not inside a positive-count gate |

A repeat can retain an earlier capture without paying it again, but Transmutation
can subsequently raise that value. Chemical X must not be pre-added to the stored
capture and then added again by each repeat. The table closes these action-level
branches, **not** every downstream orb/channel/generated-card interaction. [E7]

## 5. Lifecycle: complete direct families, bounded transitive closure

Counts below include base declarations, archive-only content and superclass calls.
They are method-family counts, not numbers of legal triggers or implemented rules.

| Named family | Declarations / direct invokes | Actual dispatcher boundary |
|---|---:|---|
| `triggerWhenDrawn` | 7 / 3 | Player draw plus two superclass calls |
| `onCardDraw` | 6 / 2 | Player draw: powers then relics |
| `onDrawOrDiscard` | 4 / 2 | Player refresh: powers then relics |
| `onExhaust` | 6 / 2 | Group exhaust: relics then powers |
| `triggerOnExhaust` | 3 / 1 | Group exhaust: card after those notifications |
| `triggerOnManualDiscard` | 3 / 6 | DiscardAction (3), DiscardSpecific, Scrape follow-up, Gambling Chip |
| `didDiscard` | 2 / 3 | Player updates hand, discard, draw cards |
| `onManualDiscard` | 4 / 1 | Manager discard increment, conditionally |
| `onShuffle` | 4 / 3 | EmptyDeckShuffle constructor; ShuffleAction update; ShuffleAll constructor |
| Card/relic/blight `atTurnStart` | 32 / 5 | Player relic/blight dispatch and draw/hand/discard card passes |
| Card `atTurnStartPreDraw` | 5 / 1 | Player hand pre-draw pass |
| Relic `atTurnStartPostDraw` | 4 / 1 | Player post-draw relic pass |
| Power/stance `atStartOfTurn` | 30 / 2 | Creature power pass; player stance call |
| Power `atStartOfTurnPostDraw` | 8 / 1 | Creature post-draw power pass |
| `onPlayerEndTurn` | 9 / 2 | Room relic then blight passes |
| `atEndOfTurnPreEndTurnCards` | 4 / 2 | Room player pre-card powers; creature nonplayer branch |
| `triggerOnEndOfTurnForPlayingCard` | 7 / 1 | Manager hand pass at null end-turn marker |
| `atEndOfTurn` | 29 / 1 | Creature power pass with player/nonplayer argument |
| `triggerOnEndOfPlayerTurn` | 4 / 1 | End-turn discard action's shuffled hand clone |
| `onRetained` | 4 / 1 | Restore-retained action, before reinsertion |
| `atEndOfRound` | 22 / 2 | Monster-group player then surviving-monster passes |
| `triggerWhenCopied` | 1 / 2 | Hand-add effect constructors; base body is empty |
| `triggerOnCardPlayed` | 2 / 3 | UseCardAction constructor's hand/discard/draw passes |
| `triggerOnOtherCardPlayed` | 3 / 2 | Player use to group, then group to cards |
| `onPlayCard` | 8 / 8 | Manager powers/relics/stance/blights and hand/discard/draw cards |
| `onUseCard` | 41 / 3 | UseCardAction constructor's player powers, relics, monster powers |
| `onAfterCardPlayed` | 3 / 1 | Group other-card pass; not deferred until queued damage finishes |
| `onAfterUseCard` | 6 / 2 | UseCardAction update: player then monster powers |
| `onMoveToDiscard` | 7 / 1 | Card teleport helper, not every generic discard movement |
| `onScry` / `triggerOnScry` | 2 / 1 each | ScryAction's power notification and card follow-up |
| `onDeath` | 4 / 3 | Base monster die and two later-act damage overrides |

For clarity, the four actual `onDrawOrDiscard` declarations are the two bases,
AccuracyPower and StrikeUpPower. Earlier five-class name-prefilter counts also
included the player caller; five candidate classes is not five overrides.
Other similarly named but empty/uninvoked hooks must not be substituted:
`triggerAtStartOfTurn` has one declaration and zero invokes; `onRetain` and
`onDiscard` have neither declarations nor direct invokes. [E1]

### 5.1 Draw, exhaust and discard resolution

| Operation | Established ordering / branch | Downstream boundary |
|---|---|---|
| Actual draw | Card draw-trigger while still draw-top; hand insertion and draw removal; power then relic draw notifications; no-argument draw then refreshes | Direct `draw(n)` does not itself perform the wrapper refresh or DrawCardAction's No Draw/shuffle checks |
| DrawCardAction | No Draw/nonpositive/empty/full-hand exits; waits for active souls; splits shortage into draw-existing, shuffle, draw-remainder | Follow-up pushed at completion, including blocked exits; remaining draw is not guaranteed after lethal filtering |
| Refresh | Power/relic refresh notifications, Corruption hand mutation, hand power recalculation, glow | `applyPowers`/glow dispatch are additional transitive dependencies, not empty UI by assumption |
| Exhaust movement | Relic then power then card exhaust notifications, before source removal; effect creation and exhaust insertion; refresh | Dark Embrace queues draw at bottom only while monsters are not basically dead; Feel No Pain queues block at bottom; neither runs synchronously inside notification |
| Manual discard | Move first (including refresh), card manual-discard notification, then increment | Increment always changes discard count; Eviscerate updates and relic manual-discard callbacks require not-turn-ended and not-end-turn |
| End-turn discard | Retained/self-retained cards removed to limbo; restore queued; discard queued unless Pyramid/Equilibrium; shuffled remaining-hand clone receives end-player-turn hooks | Ethereal hooks push targeted exhaustion above pending discard; exhaust-trigger draw is queued, not an immediate expansion of that clone |
| Restore retained | Calls retained hook before returning each qualifying object to hand; clears temporary retain, not self-retain | Pyramid's ordinary non-discard is not automatically an `onRetained` event |

Direct concrete draw-power overrides are Confusion, Corruption, Evolve and Fire
Breathing. Evolve queues extra draws for Statuses only and checks No Draw when
notified; Fire Breathing queues damage for Statuses **or** Curses. Draw-trigger
card overrides are Doubt, Endless Agony, Eviscerate, Deus Ex Machina, Void, and a
deprecated card. Generation/retrieval/pile movement is not universally actual draw.

Fiend Fire captures the **number** of hand cards, queues that many hits, then
pushes that many random single-card exhaust actions ahead of them. It is not the
same producer as Second Wind: Second Wind captures non-Attack **objects**, pushes
per-card block operations, then targeted exhaustion. Exhaust reactions may append
work, but these producers do not recursively increase their captured counts.
Random exhaust selection occurs when each exhaust action resolves. [E4–E7]

### 5.2 Turn dispatch is not one synchronous drain

The null end-turn marker calls relic/blight end-turn notifications, player
pre-card powers, queues orb handling, calls hand end-turn-for-playing hooks, and
calls the stance end-turn hook. Separately, room endTurn calls player end-turn
powers, queues clear-card and discard actions, resets draw/discard/hand attributes,
and queues its enemy-turn wrapper. Top-inserted Ethereal work can run before the
already-pending discard; bottom-inserted exhaust draw is not the same priority.

After monster work, the manager's turn-ended branch calls monster-group end
logic unless skipping monsters. That dispatch is surviving-monster end-turn,
player end-of-round, then surviving-monster end-of-round. It then calls player
stance/relic/blight start, pre-draw hand cards, draw/hand/discard start cards,
start powers and orbs, resets turn/history/block state, queues normal draw, and
invokes post-draw relic/power producers if battle is not over. Their **producer
calls** occur while draw remains queued; their inserted work and direct mutations
must be distinguished. Initial-room start dispatch differs and omits that same
pre-draw-hand call; it cannot be replaced with the repeated-turn sequence. [E3–E6,E9]

The end-turn hand clone uses `Collections.shuffle` without an explicit gameplay
RNG argument. Its ordering law is an additional source-policy dependency; this
document neither substitutes a uniform permutation nor approves a native stream.

## 6. Rescue, alternate lethal paths and durable mutations

The direct healing inventory has four `onPlayerHeal` declarations: base, Magic
Flower, Mark of the Bloom, deprecated Dodecahedron. Magic Flower rounds a 1.5
multiplier only in combat; Bloom returns zero; Dodecahedron delegates the amount
to base after pulse work. There are **no concrete power `onHeal` overrides** in
this archive. Creature healing rejects `isDying`, applies player relic amount
modifiers then power modifiers, changes HP and caps at max HP. It is not a new
death-state reset. Bloom's home event/later-act relic is not a legal fresh Act 1
route; it is a negative control, not a legal zero-heal witness. [E1,E8]

| Path | Exact bounded consequence |
|---|---|
| Player damage becomes lethal | If no Bloom, held Fairy is checked before unused Tail. Fairy/Tail heal synchronously from zero through real heal hooks and return before this branch assigns death |
| Fairy | Minimum-one raw heal based on potion potency/max HP; potion consumption is immediate, not a pending HealAction |
| Tail | Minimum-one raw half-max heal; counter becomes used; its queued visual is not the heal itself |
| Positive-heal condition | On a non-dying player with legal amount-preserving/boosting modifiers, that minimum-one is positive after hooks. A forced Bloom/dying fixture does not disprove this conditional or establish an Act 1 route |
| HP-loss / Blasphemy | LoseHPAction still invokes damage with HP_LOSS. EndTurnDeathPower queues LoseHPAction; it is not an independent hard assignment of player death |
| Max-HP reduction | Clamps max HP to at least one and caps current HP; does not itself assign death or invoke Fairy/Tail |
| InstantKillAction | Directly zeros target HP before zero-amount HP-loss damage. Its only external direct constructor caller is JudgementAction; do not generalize this enemy path to player damage/rescue |
| SuicideAction | Monster-specific die route; do not infer ordinary Fatal qualification from split/suicide |
| Explicit terminal writers | Outside initialization, direct `isDead` writes / DeathScreen construction reduce to player damage, later-act SpireHeart, abandon confirmation, and monster/revival paths. The latter are not additional legal player-combat rescue paths |

This narrows terminal **assignment** closure; it does not audit all 88 direct
`damage` invokes, every damage override or all reaction payloads. Player damage
commits HP/counters and damage-card updates before rescue/death. Rescue does not
reverse those writes or cancel unrelated already-entered code. A queued heal
does not establish rescue after terminal commitment. [E1,E4,E8,E9]

### Durable-commit and cancellation matrix

| Mutation / work | Commit boundary | What later interruption does **not** establish |
|---|---|---|
| Accepted play history | Manager on-play branch before later target handling | Not undone because payload is omitted or battle ends |
| Ordinary energy debit | After player use-entry/callback construction; X debit at its own action | Not a debit at card selection; no refund transaction inferred |
| HP/max-HP/permanent growth | Actual qualifying effect or synchronous method, e.g. Fatal payoff at its hit | Combat victory is not the only commit point; later queue loss is not rollback |
| Exhaust membership | After exhaust notifications and source removal | Queued draw/block/create responses need not already have executed |
| Temporary insertion | Some effect constructors commit pile membership before visual completion | Effect deletion does not imply membership undo or a second insertion |
| Theft ledger / gold / hit | Separate anonymous ledger action, theft-action first update, later damage | No demonstrated ordinary player choice between ledger and theft; no inferred rollback |
| Last-enemy filter | Only manager pending actions; retains heal/block/use and DAMAGE-typed entries | Does not clear pre-turn/card/monster/effect/soul state or unwind entered calls |
| Later endBattle / manager clear | Victory notifications before manager clear | Clear resets named manager work/history but does not reverse durable changes; monster queue, room objects and dungeon effect/soul containers are not proven universally cleared |
| Terminal screen routing | Changes later normal dispatch | Does not unwind an entered stack or guarantee indefinite processing of effect tails |

**Reaper remains unproved as a lost-heal/timer outcome.** Retaliation can precede
its later heal; early filtering can retain that heal while later endBattle clearing
can remove pending/current work. These facts neither establish that the heal is
lost in a natural original run nor select a frame/delta threshold. The
[runtime report](2026-10-18-act1-runtime-closure.md) does not supply an approved
general game runner. A deterministic headless policy versus explicit timers is
still a human-unapproved fidelity decision, coordinated with profile/time research.

## 7. Discriminating future acceptance obligations

**All cases below are unimplemented and unexecuted gameplay obligations.** Private
helper tests validate indexing/guards, not these mechanics. Pairwise constructed
states are not claims that all combinations have natural Act 1 reachability.

| ID | Required discriminator |
|---|---|
| X01 | Autoplay Reflex/Tactician/Deus remains rejected; autoplay Clash/Signature Move still sees their actual hand predicates |
| X02 | Normality at three and Choker at six block a normal-notification autoplay/repeat before history; contrast explicit suppressed-notification cleanup |
| X03 | Earlier target-gate rejection versus accepted-then-later-invalid-target: distinguish history, payload, payment and cleanup |
| X04 | For each of the five repeat powers, ordinary-origin false flags become true/true; energy is copied, UUID linkage shared, object identity distinct |
| X05 | Mix repeat producers: verify actual power iteration, index-one reversal, pending-action priority and no fabricated `queueExtraCard` call |
| X06 | Cost sample equals combat cost while turn cost differs: preserve that difference; Confusion still clears free flag, hand-randomization does not |
| X07 | Upgrade positive versus zero turn cost; reset versus combat mutation; Corruption's draw/setter, refresh/modifier, exhaust and payment branches kept separate |
| X08 | Stat copy versus fresh copy of Eviscerate/Blood for Blood/Searing Blow; Expunger X survives; destination flags not automatically copied |
| X09 | Repeated X with capture below newly available energy: Transmutation raises it; ordinary X action does not universally do so; Chemical X applied once per action |
| X10 | Multi-Cast with no orb; Conjure Blade with X zero; Collect+/Malaise+ at zero captured energy; distinguish payload/payment branches |
| X11 | Real draw versus retrieval/generation: own-card trigger, membership, powers/relics, refresh and Evolve/Fire Breathing eligibility |
| X12 | Draw shortage with active souls, No Draw, full hand, and follow-up: verify source-specific exits/order without a universal draw-drain assumption |
| X13 | Manual discard versus end-turn discard: movement/refresh before card hook, increment always, conditional Eviscerate/relic broadcast |
| X14 | Ethereal plus Dark Embrace, Pyramid/Equilibrium and retained cards: targeted exhaust priority, queued draw, clone membership, retained-hook distinction |
| X15 | Fiend Fire captured count/random selections versus Second Wind object snapshot; reactions do not enlarge captured work; track RNG selection timing |
| X16 | Fairy and Tail both available: Fairy consumed, Tail preserved; legal heal boost; separately label forced Bloom/dying negative controls |
| X17 | HP_LOSS/Blasphemy versus max-HP reduction and enemy InstantKill/Suicide; do not share one unconditional death/rescue handler |
| X18 | Effect already entered versus detached current versus each pending container: preserve earlier HP/gold/deck writes across filter and clear |
| X19 | Reaper retaliation/filter/endBattle schedules: record static alternatives; do not assign a natural timing outcome without its missing proof/policy |
| X20 | Setup and Forethought with positive combat cost but turn cost zero versus another changed turn cost: grant the one-play-free flag without numeric zeroing; contrast ordinary reset preserving that flag and UseCardAction clearing it |
| X21 | Setup/Forethought with zero combat cost and positive turn cost, including base-zero/turn-only increase: no new free flag; contrast initially false versus pre-existing true flag and keep later modifiers separate |
| X22 | Setup/Forethought on X/unplayable cards: movement still occurs but supplies no new free flag or legality bypass; keep X captured value/payment and unplayable overrides distinct, including a pre-existing-flag control |
| X23 | Masterful Stab in hand/discard/draw versus exhaust during qualifying positive combat loss; contrast zero/prevented loss and ordinary damage versus HP_LOSS; increment once per notification, not per HP point or global event count |
| X24 | Legally acquired Stab → True Grit+ exhaust → positive loss → Exhume: return the same object with no catch-up; test singleton and selected retrieval, then a subsequent qualifying loss in hand; compare fresh-kind/stat-equivalent copies without importing global history |
| X25 | Forethought+ same subset in different selected order, plus empty choice and ordinary mandatory-single contrast: preserve selected sequence, exact bottom-insertion effects, per-card positive-cost free guard and once-only parent effects on resume |

## 8. Closure status and exact shared interfaces

| Obligation | Current status / remaining proof |
|---|---|
| Direct legality/free/cost/copy/extra-queue families | Finite complete classfile inventories, with actual instruction sites; eight legality overrides and direct helper distinctions inspected |
| Cost precedence / copy construction | Important mutations and ten X action branches advanced; Setup/Forethought conditional flags and Masterful Stab's per-instance notification/copy/retrieval distinction checked; remaining helper callers, constructor history dependencies and legal mixed mutations still require per-site closure |
| Direct lifecycle families | Enumerated exactly, including empty/unused names; draw/exhaust/discard/retain and shared turn dispatch traced |
| Full transitive lifecycle | Not closed: every reachable override's payload, nested choice, power sorting/mutation, applyPowers/glow and mixed relic reactions require composition against the shared dispatcher |
| Lethal commit boundaries | Direct death/heal writer families narrowed; all attack/damage-all/Poison/Fatal/split/escape/reaction combinations not thereby closed |
| Reaper | Natural lost-heal outcome and original timer law absent; headless policy unapproved |

Continuation owners must consume **entered callback versus queued work**, source
object identity, commit checkpoint, return owner and terminal disposition; this
lane does not duplicate acquisition UI returns. Relic owners supply each of the
141 legal relics' trigger predicates/counters and any direct shared-field consumer;
this lane supplies dispatcher order, not a second primary relic audit. Generation
owners supply source/character/pool/cost/insertion law for each generated result;
copy/autoplay does not change the run-owned character. Profile/time owners supply
explicit elapsed-time and persistence decisions, not a silent Reaper default.

The [native contract](2026-10-18-act1-native-contract-closure.md) must preserve
origin, object/source/permanent identities, captured values, mutable flags,
individual callback checkpoints and distinct work containers. Invalid inputs do
not advance state/RNG/reward; runtime exhaustion/failure is an administrative fault,
not rollback, fake defeat, retry or a dropped batch row. Existing combat-v1/RNG
identities remain unchanged. This research does not approve their replacement,
implementation planning or product implementation. The Bead stays in progress
pending independent review and main readback.
