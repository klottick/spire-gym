# Act 1 transitive execution: shared transition and settlement rules

Date: 2026-10-19. Bead: `spire-gli.2.3.33`.

**Status: new bounded static/bytecode closure; independent review pending.** This
is not original-game execution, a completed whole specification, a scheduler
implementation, or approval of a native timing/profile policy. It advances the
[direct execution inventory](2026-10-19-act1-execution.md),
[source-specific continuation matrix](2026-10-19-act1-continuations.md), and
[relic shared-consumer reconciliation](2026-10-19-act1-relic-mechanics.md) by
specifying the common transition points those local rules compose through. The
tables close named shared endpoints; they do not claim every possible product of
content rules has been enumerated.

This revision distinguishes marker-time and discard-action-time end-turn card
callbacks, including Entrench's source-specific Ethereal enqueue; TX18 is an
unexecuted discriminator for their insertion order.

## 1. Evidence boundary and method

The inspected owned archive is `desktop-1.0.jar`, SHA-256
`cfad868ac8d65a88e71a0bf096fb09f78811e553effe0787c5309a655e081673`. The pinned
Temurin 21.0.12.1+1, javap and CFR 0.152 identities match the authorized source
identity; prior evidence batches retain their own before/after pin records. This
lane re-read retained exact-class source and bytecode plus targeted shared-power,
player/monster damage, death/escape and Reaper members; it ran no new JVM command.
No original class, initializer, game runtime or substituted runtime was executed.
Static evidence does not identify an upstream retail build or historical host JRE.
Private member locators, commands, hashes, attempt history and the before/evidence/
after claim ledger are retained in the assigned private execution lane.

`S` means reconstructed source inspection; `BC` means the named branch or ordering
was checked in disassembly; `D` is a direct consequence. None means gameplay was
run. Proposed native interfaces are marked **P**, and remain subject to whole-spec
human approval. Public class/member names are evidence locators, not dependencies
on unpublished files.

## 2. One entered transition, several work containers

The following is the concrete `GameActionManager` selection order, not universal
FIFO and not a promise that a selected action, turn, room or effect list drains.
The manager calls at most the current action's `update` in its executing branch;
selection of later work occurs on a later manager update after the current action
is done. Enqueueing during the current call does not mean the child ran inside that
call. **S/BC:** `GameActionManager.update/getNextAction`, `addToTop/addToBottom`,
`addToTurnStart`, `clearPostCombatActions/clear`.

| Checkpoint when selecting next work | Exact precedence / mutation | Boundary that remains distinct |
|---|---|---|
| Manager ordinary work | Remove index 0 of `actions`; otherwise index 0 of `preTurnActions`; otherwise inspect `cardQueue`; otherwise enqueue/dispatch monster turns; after that, turn-ended start-of-turn branch | An `actions` child can preempt queued pre-turn/card/monster work; it does not preempt an already entered `currentAction`. |
| Ordinary-action insertion | `addToTop` inserts at index 0; `addToBottom` appends. Repeated top insertions reverse the producer's insertion order. Both reject outside room `COMBAT`. | These are list-local rules, not one global ordering across card queue, monsters, effects, souls or room objects. |
| Pre-turn insertion | `addToTurnStart` inserts at index 0 of its own list, only in `COMBAT`; all ordinary `actions` still have selection priority over it. | “Turn start” does not mean immediate execution or priority over a current action. |
| Card queue immediate insertion | If already nonempty, insert at index 1; if empty, append at index 0. Ordinary actions and then pre-turn actions are selected before card queue entries. A single null end-turn marker is managed separately. | Multiple producers can place a new card behind the current queue head; repeated insertions can reverse among the newly inserted entries. This is not FIFO across work types. |
| Card selection/dispatch | Set autoplay from queue item, optionally choose random target, evaluate actual `canUse` (or explicit `dontTriggerOnUseCard` alternative), then set captured energy/ignore flags; on normal-notification branch invoke callbacks and commit history; only afterward reject a later null/dead/escaped enemy target or enter player use | A rejected legality gate does not make normal play history. Accepted history is not rolled back by later target rejection, and a late target rejection omits player use/payment. |
| No-play autoplay cleanup | A failed autoplay card sets the suppress-notification flag and queues `UseCardAction` for cleanup; it is not a normal successful payload/history branch. | The producer's destination flags and `UseCardAction` cleanup still need their own branch. |
| Monster turn queue | Once card queue is empty, enqueue monster snapshots if not already queued; process one head monster, enter `takeTurn` and its turn-power producer calls, then remove the snapshot | The next monster or room end-of-round work is not a child executed within `takeTurn`. |
| Null card-queue end-turn marker | Calls room relic/blight end-turn hooks, player pre-card powers, queues orb handling, calls current-hand `triggerOnEndOfTurnForPlayingCard`, then stance end-turn | This marker-time card callback is distinct from the later discard action's `triggerOnEndOfPlayerTurn` dispatch below. |
| Room end-turn scheduling | Separately invokes player end-turn work, queues clear/discard actions and enemy-turn wrapper, and resets the named pile/card attributes | Scheduling this work does not execute the discard action or its later per-card callbacks. |
| `DiscardAtEndOfTurnAction.update` | On its initial update, moves retain/self-retain objects to limbo and queues their restore action at top; unless Pyramid/Equilibrium suppress bulk discard, repeatedly top-inserts discard actions; then clones the remaining hand, calls `Collections.shuffle` on that clone, and visits it via `triggerOnEndOfPlayerTurn` | A visited Entrench overrides this latter callback and top-inserts `ExhaustAllEtherealAction`; Entrench is not itself Ethereal. That queued cleanup sits ahead of already queued discard work; repeated top insertions reverse producer order, including among Entrench callbacks. The shuffle call has no explicit gameplay RNG argument, so no seeded or uniform law is asserted. |
| Victory filtering | On basically-dead monster group, `cleanCardQueue` removes card-queue items still in hand and marks limbo cards fading; `clearPostCombatActions` removes pending ordinary actions except `HealAction`, `GainBlockAction`, `UseCardAction`, and `DAMAGE`-typed actions | Does not clear `preTurnActions`, monster snapshots, entered stacks, room objects, dungeon effect/soul lists or every current action. It is a selective filter, not cancellation rollback. |
| Later manager clear | `clear` empties ordinary/pre-turn/card queues, nulls current/previous/start actions, clears named play/orb/stance histories and resets turn fields | It does not undo durable state already changed; it does not itself clear the monster queue or prove all room/effect/soul work is gone. |

The two end-turn card callbacks above are not aliases: `triggerOnEndOfTurnForPlayingCard`
is marker-time work; `triggerOnEndOfPlayerTurn` is called later by
`DiscardAtEndOfTurnAction` on a shuffled clone of the remaining hand. Entrench's
override implements only the latter and queues Ethereal cleanup at top after the
bulk discard actions were inserted. That action is pending work, not synchronous
exhaustion during the callback. The source's `Collections.shuffle` invocation has
no explicit gameplay RNG parameter; its native order/measure is unresolved here.

The game-level dungeon update has additional independently ordered containers:
room-object updates, active top-level and ordinary effects (update before done
test/removal), then transfer from pending effect queues without updating the newly
transferred effects in that pass. Installed relic updates occur in the overlay
update after those effect passes, while screen and room consumers are conditional
on the selected screen (room update additionally occurs in Peek). A transfer or
screen reopen is not a universal callback/drain. These shared facts are retained
and bytecode-checked in the continuation evidence; local source-specific return
conditions are tabulated there rather than replaced with a generic continuation
stack.

## 3. Power/relic callback order, priority ties and list mutation

There is no single global callback order. This finite precedence table states the
shared dispatchers and their actual receiver order. Iteration is over the current
collections; callbacks may mutate state or queue work. Do not silently snapshot
all callback lists or re-sort after every callback. **S/BC:** `ApplyPowerAction`,
`AbstractCreature.addPower`, `AbstractPower.compareTo`, `AbstractPlayer.damage`,
`AbstractMonster.damage`, `AbstractPlayer.heal`, `AbstractMonster.heal`.

| Site | Actual order and mutation rule | Consequence / limit |
|---|---|---|
| New power via `ApplyPowerAction` | Source powers receive `onApplyPower`; Champion Belt's local callback may enqueue its consequence; target validity and Ginger/Turnip prevention precede Artifact; Artifact may consume its trigger and return; otherwise matching target powers stack (all matching IDs encountered, except `Night Terror`) or new power is appended, the target list is sorted, and initial-application/modify-power callbacks follow | Prevention may occur after source callbacks. Artifact consumption follows named relic prevention. Stacking and insertion are separate branches; no apply means no new power's initial callback. |
| Power comparator | `compareTo` is `this.priority - other.priority`. `ApplyPowerAction` sorts only after adding a new power. Java's stable list sort preserves the input order for equal comparator results at that sort; equal priority therefore is not an ID/name ordering. | Re-sorting after a new insertion is observable. Do not claim arbitrary cross-JRE byte-for-byte behavior from this classfile, and do not treat the mutable list as permanently sorted if other code changes priority or uses another add path. |
| Direct `AbstractCreature.addPower` | Iterate existing list and stack every matching ID; append if none; no sorting in this helper | Not equivalent to the `ApplyPowerAction` new-power path. Direct callers are a distinct source-site obligation. |
| Shared damage modifiers | For player target, source-self relics, source powers (if owner exists), target relics, target powers modify amount; later attack notifications and target `onAttacked` callbacks follow source-specific owner branches. Monster target uses a different order and does not invoke player damage's relic target callbacks. | No universal “all powers then all relics” damage modifier order. See §5. |
| Power lifecycle passes | Start, end, draw and turn passes iterate the target's existing power list in list order. Addition/reordering is performed at the explicit application sites above, not by a universal callback dispatcher. | An action queued by a callback runs only if/when its owning manager reaches it. |
| Player healing | Living check; player relic heal modifiers in relic-list order; then power `onHeal` in power-list order; then HP addition/cap | Maximum-HP mutation can precede this and survive a refused heal. This is not monster healing order. |
| Monster healing | Dying check; power `onHeal` in current order; then add/cap; no player relic modifier pass | It is not player healing with an empty relic list. |

No defensively copied callback snapshot is established by these loops. Where a
callback directly changes a collection during iteration, the source-specific
collection mutation and its actual caller must be inspected; queued `ApplyPower`
work normally creates a later checkpoint, but that is not permission to rewrite a
direct `addPower` call as queued work. No universal priority table is asserted for
Relics, cards, stances, blights, powers, room hooks and actions together.

## 4. Cost, free-play, capture, history and cleanup as ordered checkpoints

| Stage | Established rule | Not interchangeable with |
|---|---|---|
| Legality | Base card gate runs status/curse permissions, target/group checks and ordered turn/power/Entangled/relic/blight/hand permissions before the final energy test; only that final energy comparison accepts enough energy, free play or autoplay | An autoplay label is not a universal early legality bypass. Normality/Choker/NoSkills and card-specific `canUse` gates remain active. |
| Accepted manager entry | Install autoplay/target; call `canUse`; set captured energy and ignore-total; for ordinary notification branch visit player powers, monster powers in monster order, player relics, stance, blights, then hand/discard/draw cards; commit play counters/history | `dontTriggerOnUseCard` is a separate branch; later invalid-target handling cannot retroactively undo this history. |
| Player use entry | Recalculate damage; cap eligible X against live energy only when ignore-total is false; call card `use`; construct `UseCardAction` and run its constructor-time on-use callbacks; notify hand's other-card path; remove from hand; debit ordinary energy unless free/autoplay/Corruption-Skill exception | `card.use` often queues payload. It is not proof of synchronous resolution, nor a payment before the use-entry callbacks. |
| X action resolution | Each X action reads captured energy unless sentinel `-1`, otherwise live energy, then applies its own Chemical X/upgrade/positive-count and special gates. Nonfree debit generally uses live energy at that action | Do not fold Chemical X into captured X once at card selection or charge the same captured payment at both repeat and action. Multi-Cast's no-orb and Conjure Blade's zero-effect creation remain distinct. |
| Use cleanup | First update invokes player then monster after-use powers; purge and Power return early; ordinary cleanup clears free-once before selecting Spoon/exhaust/rebound/shuffle-back/return/discard precedence | A Power cleanup return is not the ordinary destination matrix; marked exhaust is not itself movement through the actual exhaust callbacks. |
| Copy/recursion | Five repeat powers carry original energy and explicitly set autoplay and ignore-total true; same-instance route retains shared UUID but allocates a distinct object. Omniscience queues original/exhaust and stat-equivalent purge copies with different identity/flags | No call to the unused direct `queueExtraCard` helper is inferred; no single copy means all mutable flags copied. |
| Per-instance exceptions | Setup/Forethought grant free-once only when mutable combat `cost > 0`; no numeric zeroing. Masterful Stab increments its own cost on qualifying notifications, not by global loss total. Transmutation's turn-zero setter leaves negative X sentinels alone; its self-generated X result receives no new free flag through the inspected fresh/upgrade/copy/hand path absent unrelated modifiers. | These corrected source guards are not replaced by broad “generated cards cost zero” or “damage history catch-up” rules. |

The ordered source guards that survive integration are: Eviscerate's fresh-copy
setter targets the **source** under its map/combat condition and uses per-turn
discard history; Blood for Blood's combat-damage reduction targets the **returned
fresh card**; Forethought+ preserves the selected sequence; Mayhem's play-card
producer occurs before the draw it queues; and “no child draw enqueued” is not the
same state transition as constructing a zero-amount draw action with a history
clearing update. These source-specific distinctions are restated because they
alter composition, not as a second roster of primary content.

## 5. Damage, heal, Fatal, split, escape and cancellation boundaries

Player and monster `damage` are separate state machines. The order below is the
source order for the named base paths; content overrides, DamageInfo construction,
source ownership, power order and action placement still select the branch.

### 5.1 Player damage path

| Boundary | Player-target behavior (S/BC) |
|---|---|
| Input / Block | Read `DamageInfo.output`, remember prior Block, clamp negative amount to zero, apply player Intangible ceiling, then decrement Block (HP_LOSS is its own no-Block path). |
| Modifiers | If owner is player, player relic `onAttackToChangeDamage`; if owner exists, owner powers `onAttackToChangeDamage`; target relic then target power `onAttackedToChangeDamage`. Player-owned relic `onAttack` follows; with nonnull owner, owner powers `onAttack`, target powers `onAttacked`, then target relics `onAttacked`. Null-owner damage skips that latter source/target callback block. |
| Last-loss and loss hooks | Target relic `onLoseHpLast` runs before `lastDamageTaken` is assigned. `lastDamageTaken = min(modified amount, current HP)` precedes the positive-loss guard and the subsequent `onLoseHp` power amount transforms. Positive loss then calls relic `onLoseHp`, power then relic `wasHPLost`, and source powers' `onInflictDamage`. These are notification amounts/checkpoints, not final HP. |
| Commit | For positive post-hook amount, update HP-loss counter only for HP_LOSS, turn/combat damage counters, subtract HP, and in combat notify the then-current hand, discard and draw card objects once each and increment `damagedThisCombat`. The card notification is not once per lost HP point and excludes exhaust. |
| Rescue / terminal | Update clamped HP and bloodied callbacks, then if HP<1 and Bloom absent consume Fairy first; otherwise unused Tail; both synchronously call heal and return before `isDead` assignment. Otherwise assign death/screen and clear remaining Block. Earlier counters, notifications and HP changes are not rolled back by rescue. |

`onLoseHp` can change the later amount while `lastDamageTaken` was already
captured. Reaper's healing amount consequently uses the monster-side per-target
`lastDamageTaken` values, not a generic “attack damage” total. A prevented/zero
loss does not enter the positive player notification/HP-commit branch. A queued
heal is not a resurrection after `isDead` has committed.

### 5.2 Monster damage, death and escape

| Boundary | Monster-target behavior (S/BC) |
|---|---|
| Input / terminal guard | Monster Intangible may rewrite `DamageInfo.output`; then `isDying` or `isEscaping` returns before target damage resolution. Negative amount clamps to zero; Block is processed. |
| Modifiers and notifications | Player relic `onAttackToChangeDamage` only when owner is player; source powers when owner exists; target powers `onAttackedToChangeDamage`; player relic `onAttack` when player-owned; target powers `wasHPLost` then owner powers `onAttack`, then target powers `onAttacked`. The target-power `wasHPLost` loop is not inside a positive-damage guard. This differs from player damage and has no corresponding target relic `onAttacked` pass here. |
| Health boundary | Set `lastDamageTaken = min(amount, currentHP)` before subtracting positive damage. The base monster method has no player card-damage-history pass and no player `onLoseHp`/rescue sequence. If HP is <=0, enter `die`; if the group is basically dead, clean the card queue, fade/clear limbo and hide combat panels. |
| `die(triggerRelics)` | First transition only (`!isDying`): set `isDying`; if HP<=0 and trigger flag true, visit target powers' `onDeath`; if trigger flag true visit player relic `onMonsterDeath`; if group basically dead, disable end-turn and exhaust/clear limbo; clamp negative HP and start death timer. This is not immediate `isDead`. |
| Death timer settlement | Monster update decrements timer; below zero sets `isDead`, and only if all monsters are dead and room is neither already over nor `cannotLose` calls `endBattle`; then disposes and clears powers. `endBattle` sets its own room flag/timer and performs its own callback/manager-clear boundary. |
| Escape settlement | `escape` sets `isEscaping` and a separate 3-second timer. When it crosses below zero, set `escaped`; the same group-dead / room-not-over / not-cannot-lose condition may call `endBattle`. Escape is not death, Fatal, kill credit or player victory by implication. |
| Split / summon | A split child is a new monster insertion and its own later update/turn/death path. Parent death callbacks, child presence, group-basic-dead tests and action cancellation are distinct checkpoints; a split trigger is not itself a final kill reward. |

These transitions explain why last-enemy filtering and later `endBattle`/manager
clear cannot be represented by rolling back an entered damage call. Durable card
growth, gold, HP, relic counters, power hooks or already-created pile membership
commit where their source effects actually run. Conversely, queueing a Fatal
payoff or successor child does not prove it ran before cancellation. Source-specific
Fatal actions must bind the intended target and their own lethal/kill condition;
there is no single group-level “Fatal succeeded” transition inferred from a death
animation, split, suicide, escape or room flag.

### 5.3 Reaper/Match local timer law; unresolved settlement

The ordinary desktop caller rejects a frame whose raw delta exceeds `0.1` seconds
before game update. Thus a source update admitted through that caller receives no
more than that frame delta; this is a **source caller bound**, not a native agent
clock proposal or a measured runtime frequency. The profile/time document's
proposed 60-Hz/exact-elapsed/Wait contract remains unapproved.

For Reaper, the inspected all-enemy action waits through its local duration, then
visits current non-dying/non-escaping monsters in list order, invokes their damage
paths, sums each target's captured `lastDamageTaken`, and queues optional visual
work plus a HealAction when the sum is positive. In non-fast mode it inserts a
wait before that heal. The action also inserts its own top-priority wait after this
branch; if the group is basically dead it selectively filters pending ordinary
work, retaining HealAction and damage-typed work at that checkpoint. Those facts
prove a conditional queueing path and a clear/filter race surface, not a natural
lost heal: later room endBattle/manager clear may intervene, while the heal may
also be selected and apply synchronously before a later clear. No exact elapsed
schedule, natural timer-loss witness, or universal Looter interposition law is
established. Looter escape timing, if relevant, has its own actor/timer and is not
interchangeable with monster death or an action queue wait.

For Match and Keep, the continuation source trace proves strict `< 0` timer gates,
not `<= 0`: after the second selection it reveals both cards immediately and sets
pair wait to the match/miss duration; only a later strict underflow resolves that
pair and consumes an attempt. The fifth resolved pair sets game-done and a 1-second
cleanup wait; next Play update enters cleanup through game-done, while cleanup's
own strict-underflow gate may fail on an exact-zero landing. Under admitted frame
deltas no single subtraction can skip from the ordinary 1-second branch across
zero; exact-zero can occur and is observably different from negative. The normal
fifth-pair path's next Play entry remains positive after a maximum `0.1` subtraction.
These are source-condition deductions, not natural frame-frequency measurements.
Native timing must explicitly choose the clock, preserve the distinction if it
claims parity, and must not use an arbitrary large delta to manufacture an outcome.

## 6. Continuation commit/return arbitration

The prior continuation matrix remains the source-specific authority for which
parent consumes a child result. This table closes the common checkpoints that
prevent “screen closed means completed” and “queued means durable” substitutions.

| Source family | Before | Actual commit / consumer | After, cancel or terminal branch |
|---|---|---|---|
| Card reward row | Claim opens the card choice and leaves its row pending | Take removes the exact row before deferred Fast obtain; Skip closes and leaves the offer available | Reopen reuses the offer; neither Take nor screen close proves later obtain insertion. |
| Shop card purchase | Purchase handler identifies current stock | Construct/enqueue Fast obtain, then spend and mutate/remove/restock stock under that handler | Parent shop can resume; child obtain does not authorize another payment/purchase. |
| Shop relic purchase | Current stock/purchase accepted | Spend and install via instant path; onEquip may open child before restock finishes | Child selection is owned by relic result consumer; payment/first installation are not undone on return. |
| Neow transform | Selected object(s) are current candidates | Single transform generates before removal; two-card transform removes both before generation | The two-source order is not collapsed; event parent resumes through its own no-screen path. |
| Boss relic take/skip | Boss screen choice is current | Screen closes; ordinary take schedules/spawns relic, later arrival reaches onEquip. Black Blood replacement is a distinct instant path. | Child onEquip may suspend parent; skipped legal treasure/reward stays skipped, not auto-claimed. |
| Delayed relic child | Relic is already owned and its update observes its pending latch/result | Its source-specific callback mutates flags/deck/removal and clears selection/sets its own room or reward marker | Local completion is not successor readiness, full action drain, or automatic map return. Empty Cage's zero-input branch can leave a different room phase than its latch. |
| Note direct obtain/removal | Incoming card is stored source payload | Direct incoming hooks and master-deck insertion precede outgoing candidate construction; selected outgoing object removal precedes saved-reference write at next-room transition | This does not call Omamori's obtain-effect constructors. The incoming card can supply a removable candidate only when its own ID is removable. |
| Event result / Proceed | Source-specific result choice is entered | Event's own Continue/openMap, event-combat rewards or labeled/unlabeled reward handlers perform distinct effects | EventRoom resumes only at its no-screen consumer. Lab/Dead Adventurer/Mushrooms direct-map path is not Woman in Blue's reward return. |
| Boss → successor | Boss reward Proceed enters treasure; treasure take/skip and remaining child screens remain source-owned | Next-dungeon handler (after its one-time warning if needed) begins fade and marks dungeon beaten; later game transition builds successor with the same player and performs new-act reset/heal setup | +1 is not boss lethal, room COMPLETE, screen close, or first fade marker. Whether native successor payload includes the generated next map is still a human schema decision. |

One source-consumer fact is useful for the whole nested-choice class: installed
relic result consumers execute in the overlay relic-update pass, not only in the
offer screen that acquired them. Accordingly, the parent may be absent while a
child selection is pending. Source latches and actual result ownership, not an
assumed arbitrary-depth screen stack, determine resume. The previously published
continuation table supplies the per-event, shop, Neow, campfire, relic and reward
rows; this document adds their shared commit checkpoints rather than duplicating
their 25-event inventory.

## 7. Obligation ledger: before / new evidence / after

| Obligation | Before this document | New evidence and exact delta | After; residual not claimed closed |
|---|---|---|---|
| B04 dispatch/cost/copy | Direct family inventory and corrected per-card examples; mixed callbacks/queue endpoints remained open | §§2–4 tie manager selection, callbacks, history, cost/payment and cleanup to explicit ordered checkpoints, including priority arbitration and no-child-vs-zero-child distinction | Shared checkpoint classes are specified. Every content override and every mixed reachable mutation combination are not exhaustively composed. |
| B05 work completion/continuations | Source-specific parent-child rows; no universal scheduler | §§2 and 6 distinguish current, pending, pre-turn, card, monster, room/effect work and source commit/consumer/return endpoints | No global FIFO, unconditional drain, screen-stack law or universal return-to-map. Exact legal empty-selector history remains open. |
| B06 damage/heal/death/Fatal | Player rescue and selected terminal writers bounded; all damage and reaction chains unresolved | §5 states exact distinct player/monster hook order, positive-loss commit, card-history and rescue checkpoint, delayed death/escape, split and cancellation boundaries | Does not enumerate all damage producers/overrides/powers, all Fatal action predicates, all simultaneous/reaction combinations or natural escape interposition. |
| B07 card/power/relic interaction | Complete local card/relic/enemy primary records; transitive consumer composition remained a whole-class residual | §§3–5 supply common power insertion/tie/list rule and damage/heal/turn checkpoint matrix to compose against those primaries | Does not claim exhaustive product of every legal content interaction; source-site records remain necessary for local payloads. |
| B08 nested offers/economy | Local paired offer and continuation records, not common durable commit closure | §6 maps acceptance → durable resource/offer mutation → owned result consumer → conditional return across major nested-source classes | Every source's generation measure and exceptional-state initialized reachability remain the respective RNG/continuation obligations. |
| B12 time/settlement | Local timer observations; native clock unapproved; Reaper lost heal not proved | §5.3 derives only caller-admitted delta≤0.1 for desktop and local strict comparison/queue alternatives for Reaper and Match | No native 60-Hz/exact-elapsed policy, no Reaper natural loss/success outcome, no universal Looter interposition. |
| Exceptional empty/Cage/stale selector | Local source branches known; history witness/exclusion missing | Current source traces show exactly which branches need an empty candidate and distinguish empty incoming Note from removable incoming Note; generation/map law integration still required | No legal initialized-history witness or impossibility proof is claimed. Do not disable/auto-skip committed selector based on a model miss. |

### Exact exceptional-choice proof status

The retained continuation facts do establish source-local conditional behavior:
Empty Cage with zero purgeables sets its selection latch but can leave the room
`INCOMPLETE`; Purifier, Transmogrifier and Wing Statue enter fixed-count grids
without a local empty fallback; Note adds the incoming card before computing its
outgoing candidates. A legal removable incoming Note card is a constructive
**local** nonempty candidate, but no initialized-profile/history/map witness is
claimed here. If the incoming stored card is itself unremovable and the prior deck
has no removable cards, Note has no local guard; the source branch alone neither
proves that state naturally reachable nor excludes it. Stale/mistyped reward rows
are conditional source behavior, not a licensed native action. Root-conditioned
event and history laws are owned by the RNG/map proof lane; no supplied choice,
model selection or bounded-search miss is elevated to a witness or exclusion.

## 8. Discriminating future acceptance cases

**Unimplemented and unexecuted native obligations.** Static checks above are not
these gameplay tests. Fixtures below must be labeled as source-conditional when
their initialized Act 1 reachability is unresolved.

| ID | Discriminator |
|---|---|
| TX01 | Add a power with lower/equal/higher priority through ApplyPowerAction; check comparator order, stable equal-priority relative order, sort only on new insertion, and direct `addPower` versus action path. |
| TX02 | Source callback enqueues a child while current action remains entered; prove child runs only after return/current completion and according to its own container's priority. |
| TX03 | Top-insert A then B while ordinary queue is nonempty; establish B-before-A and action-before-preTurn-before-card precedence without extending the claim across other containers. |
| TX04 | Accepted card, later dead target, earlier legality rejection, and explicit suppressed-notification alternative: compare play history, onPlay counters, use, debit and cleanup separately. |
| TX05 | Same player-damage packet with null owner, player owner and monster owner; distinguish owner callback branches, lastDamageTaken timing, HP_LOSS counters, card notifications and actual HP commit. |
| TX06 | A power changes amount in onLoseHp after lastDamageTaken capture; verify Reaper's captured monster-side value versus final player HP and subsequent heal amount. |
| TX07 | Positive player loss, zero/prevented loss and HP_LOSS: check order and once-per-card-object notification across hand/discard/draw, no exhaust callback, then rescue after mutation. |
| TX08 | Lethal player damage with Fairy and Tail: Fairy first, Tail preserved; then Bloom/dying injected negative control, explicitly not a natural Act 1 witness. |
| TX09 | Monster lethal hit with surviving ally, last monster, `cannotLose`, split and escaping actor: keep `isDying`/`isDead`/`escaped`, death callbacks, group test and room settlement distinct. |
| TX10 | Reaper's pending heal retained by last-enemy filter, then selected before versus removed by later manager clear; no claimed natural outcome unless exact approved timer policy and source schedule are supplied. |
| TX11 | Reaper damage updates `lastDamageTaken` per eligible nonescaping monster; vary Block/prevention/target death and ensure no heal is inferred from raw packet amount. |
| TX12 | Match pair timer landing exactly zero versus below zero, fifth attempt then cleanup, with admitted frame max `0.1`; preserve strict underflow and revealed-information boundary. |
| TX13 | Skip and reopen card reward, take and defer obtain, Bowl null item and last row; commit one source mutation and distinguish later child insertion. |
| TX14 | Neow one/two-card transform, shop purchase, relic onEquip child and boss take/skip; assert source-local mutation happens once and child resumes only at its actual consumer. |
| TX15 | Empty Cage 0/1/2 and committed fixed grid with zero candidates: test conditional local branch separately from an initialized legal witness/exclusion; never treat forced fixture as proof. |
| TX16 | Note incoming removable versus unremovable card with empty prior candidates; direct obtain and candidate construction precede outgoing selection, but only a proven natural profile/history fixture can settle reachability. |
| TX17 | Last-enemy filter followed by endBattle/manager clear: preserve earlier HP/gold/deck/relic writes, distinguish admitted current stack, each pending list and effects/souls, and avoid fake rollback/defeat. |
| TX18 | End turn with Entrench in the remaining-hand clone, Ethereal cards and other queued discards: distinguish marker-time `triggerOnEndOfTurnForPlayingCard` from later shuffled-clone `triggerOnEndOfPlayerTurn`; check Entrench's override inserts top-priority Ethereal cleanup ahead of already queued discard work, without treating Entrench as an Ethereal card or assuming a seeded/uniform shuffle. |

## 9. Remaining exact proof obligations and approvals

1. Independently review this exact document head; no such review has occurred.
2. Compose remaining individual primary effect-site branches and legal combinations
   against these shared dispatchers. This document does not provide a giant total
   ordering that removes source-specific guards.
3. Establish initialized legal histories or source-image exclusions for mandatory
   empty selectors, Empty Cage's zero-input phase and stale/mistyped records. No
   model choice or search failure is evidence of impossibility.
4. Determine the natural Reaper/lost-heal and Looter interposition result only with
   an approved source-time contract and proof; no native timing choice or universal
   interposition rule is adopted here.
5. Preserve all human gates: whole-spec approval, coordinated implementation-plan
   review, and separate implementation-start approval remain open. No product code,
   dependencies, original-game execution, build, benchmark or training is authorized.

The native contract must preserve source origin, object/source/permanent identity,
captured values, callback checkpoints, separate work containers, committed state,
source-owned continuation and administrative fault without rollback. Invalid input
still advances no state/RNG/reward; content remains untruncated. The Act-1 task
reward is +1 only at resolved successor-ready handoff, −1 only on actual death, and
0 otherwise. All future cases in §8 are **UNEXECUTED**. This bounded author issue
remains in progress pending independent review and main readback.
