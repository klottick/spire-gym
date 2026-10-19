# Act 1 semantic execution and continuation contract

Date: 2026-10-19 · Bead: `spire-gli.2.3.42`

**Normative native semantic contract candidate, submitted for independent review; not human-approved implementation authority.** This document supplies concrete action, settlement, damage and continuation rules for standard unmodded fully-unlocked Act-4-enabled Ironclad A0 Act 1 through resolved Act2Ready. It deliberately replaces frame progression with the selected deterministic immediate-causal settlement policy. Source facts below describe bounded static inspection, not original-game execution. All future cases are **UNEXECUTED**.

The normative policy is the accepted consolidated specification at base `2e090d78297a236bfd549797e460db40d47e709f`, especially §§1.2, 4–9, 11. It controls where historical sources differ: immediate effects and mandatory causal descendants settle before victory unless an explicit gameplay interruption applies; Reaper heals on the last-enemy kill; no post-death healing/resurrection, unrelated work drain, frame simulation, extra enemy turn, rollback, or source-FIFO claim. Independent RNG, selected profile and successor policies also remain binding. This is an independently authored producer contract for the existing native semantic/site owner; it does not define wire codecs or a new RNG algorithm.

## 1. Evidence boundary and inputs

The owned source archive is SHA-256 `cfad868ac8d65a88e71a0bf096fb09f78811e553effe0787c5309a655e081673`. No original class, initializer, game runtime, or substituted original-class runtime was run for this document. Source facts are reused from accepted, independently reviewed public inputs:

- [Transitive execution closure, exact reviewed revision](https://github.com/klottick/spire-gym/blob/4167152c5d0fce47586fe83f3d1cfc1f976af609/docs/research/2026-10-19-act1-transitive-execution-closure.md): common manager/container priorities, card admission checkpoints, player/monster damage split, rescue/death/escape, common durable commits, and the source-specific timer facts.
- [Direct execution families, exact reviewed revision](https://github.com/klottick/spire-gym/blob/f515954193bac8d36eb8e425051ba5940c5a4000/docs/research/2026-10-19-act1-execution.md): finite direct legality/cost/copy/callback families, X actions, corrected Setup/Forethought and Masterful Stab histories, Transmutation generated-X and Eviscerate receiver.
- [Source-specific continuations, exact reviewed revision](https://github.com/klottick/spire-gym/blob/2cbcd2d325f0e4d4f2294612cbfff131919e2288/docs/research/2026-10-19-act1-continuations.md): Neow/shop/reward/event/campfire/relic-child/Note/Match commit and return branches.
- The accepted [consolidated specification](../superpowers/specs/2026-10-19-ironclad-a0-act1-consolidated-design.md) restates these facts with selected-policy precedence and future obligations.

Grades: **F** = accepted source/static fact bounded by its listed branch; **S** = selected native policy; **P** = engineering contract proposed here for whole-spec review; **O** = exact residual. A count or local branch is not universal transitive proof. There is no claim that every constructed discriminator is an initialized legal history.

## 2. Exported semantic records

These are normative semantic data structures (not serialized layouts). Identities are private typed handles, never policy-visible IDs. State tags are exhaustive for this contract; the native owner may encode them with its own reviewed stable codes.

### 2.1 `SemanticActionState`

| State tag | Required fields | Meaning and outgoing transitions |
|---|---|---|
| `Idle` | `active_item = None` | No currently entered gameplay item. Select one admitted work item or expose a genuine decision. |
| `Preflight` | `candidate_action`, `source_site`, `target_ref?`, `captured_inputs` | Validate current phase, source legality, target, cost permission and required content bindings before semantic dispatch. Invalid external input rejects before any run/RNG/history/reward mutation. |
| `Rejected` | `reject_reason`, `source_site` | No accepted play/history/payload/payment. Clear this attempted item; unrelated pending work is not silently consumed. |
| `Admitted` | `item_id`, `origin`, `source_instance?`, `play_context`, `captured_energy`, `ignore_energy_total`, `captured_x?` | Manager accepted card after `canUse`; admitted history/callback effects commit at their source checkpoint. Late target failure does not roll these back. |
| `Entered` | `item_id`, `call_frame`, `mutation_checkpoint` | Current call is on-stack. A manager tick updates at most this item; queued work cannot recursively execute inside the entered call. |
| `EffectsOpen` | `item_id`, `immediate_effects[]`, `mandatory_children[]` | Use entry has returned; source payload and explicitly owned immediate/causal consequences are tracked separately from unrelated backlog. |
| `Suspended` | `continuation_id`, `parent_item`, `resume_checkpoint`, `captured_offer`, `selected_prefix`, `remaining_count` | A genuine policy choice is pending. Preserve offer, ordered choices, prior commits and capture; no reroll/repay on resume. |
| `Cleanup` | `item_id`, `after_use_order`, `destination` | After-use powers and ordinary cleanup run in the source-specific order. Purge/Power early returns are separate destination variants. |
| `Committed` | `item_id`, `commit_refs[]` | This item's synchronous state changes and required immediate children are complete; mark once-only checkpoints. This does not mean all queues drained. |
| `InterruptedDeath` | `death_commit`, `active_stack_ids[]` | Actual player death committed. No subsequent healing, resurrection, victory, or successful handoff. Retain already committed mutations; stop nonmandatory work according to terminal arbitration below. |
| `TerminalVictory` | `victory_source`, `remaining_causal_work[]` | Enemy-group victory candidate only. Transition to reward/room victory after mandatory causal work settles or explicit gameplay interruption; never award Act1 success here. |
| `Faulted` | `fault_stage`, `fault_site`, `committed_prefix` | Administrative post-dispatch failure. Quiesce pool; no rollback, success, death, retry, or successful output fabricated. |

`captured_inputs` is a tagged finite record: `{energy_on_use: Optional<i32>, captured_x: Optional<i32>, target: Optional<ActorRef>, origin: PlayOrigin, source_instance: Optional<InstanceRef>, payment_rule: PaymentRuleId, free_once_at_entry: bool, autoplay: bool, ignore_energy_total: bool}`. Absent means no such source value, not zero. `origin` and flags remain independent.

### 2.2 `CausalSettlement`

| Field | Domain / invariant |
|---|---|
| `settlement_id` | Private run-scoped semantic identity; not derived from worker, address, frame or completion race. |
| `active_item` | At most one currently entered `SemanticActionState` item. Retiring it is distinct from selecting its next child. |
| `causal_stack` | Ordered records `{parent_item, child_item, relation, source_site, admission_checkpoint, completion_checkpoint}` for required descendants. Each child has one cause owner and once-only completion key `(parent, checkpoint, site, source_ordinal)`. |
| `mandatory_frontier` | Ordered finite set of immediate source effects and mandatory descendants causally required to finish the current semantic item/terminal candidate. No unrelated card/relic/event queue entries enter merely because pending. |
| `backlog` | Distinct source containers: `actions`, `pre_turn_actions`, `card_queue`, `monster_snapshot_queue`, `room_objects`, `dungeon_effects_active`, `dungeon_effects_pending`, `top_level_effects_active`, `top_level_effects_pending`, `souls`, and `continuation_consumers`. Preserve each container's own owner and ordering; they are not a unified FIFO. |
| `terminal_candidate` | `None | PlayerDeath | CombatVictory | EscapeResolution | Act1Handoff | AdministrativeFault`; only one committed terminal result, with player death/fault precedence as specified below. |
| `committed_checkpoints` | Set of once-only semantic checkpoint keys; never an undo log. Mutation already committed survives later cancellation/death/fault. |
| `decision` | Optional owned continuation choice; only a genuine strategic choice suspends the run. |

Causal relations are exactly: `ImmediatePayload` (the selected card/potion/effect's direct effect); `RequiredCallback` (a callback that source dispatch synchronously invokes before returning); `RequiredChild` (a child explicitly required by that payload to reach its ordinary completion); `TerminalPayoff` (Fatal/reward/growth/relic outcome causally caused by qualifying kill); `MandatoryChoice` (a source-owned selection whose legal result is required to complete its parent); `Unrelated` (work not required by current causal chain). Only first five enter mandatory frontier; `Unrelated` remains in its source container. This classification attaches per source site, not by action class name alone.

### 2.3 Deterministic settlement transitions

| Transition | Guard | Ordered state change |
|---|---|---|
| `submit` | Live policy decision, valid candidate and current continuation token | Create `Preflight`; validate entire batch before any row dispatch. Invalid row rejects entire batch with zero mutation. |
| `admit_card` | Actual `canUse` succeeds | Apply actual source path: set manager play context and captures; run the specified ordered notification branch; record play history only on the normal admitted branch. Later target invalidation omits use/payment, but does not erase admission history. |
| `reject_card` | Earlier legality fails | Record no normal play history, payload, ordinary payment or successful-play destination. A failed autoplay cleanup branch is not a successful play. |
| `enter_effect` | Selected item is dispatchable | Mark `Entered`; execute one source boundary/callback in its declared local order. Enqueues become pending; they do not run recursively during this call. |
| `return_effect` | Entered method returns | Commit synchronous mutation; classify emitted records by relation. Add `ImmediatePayload`, `RequiredCallback`, and required descendants to this item's causal frontier in the source-sensitive order in §3. Preserve other work in its original container. |
| `suspend_choice` | Parent reaches a genuine policy decision | Store existing offer, ordered partial selection, count/eligibility and resume checkpoint. No re-generation, second payment, re-entry of completed hook, or parent duplication. |
| `resume_choice` | Same parent/source consumer and legal child result supplied | Revalidate handles against current owner state; commit only the branch's stated checkpoint; continue at saved resume point. Stale handles reject without mutation. |
| `settle_item` | Current item's direct effects complete | Resolve mandatory causal frontier, recursively, with stated source-specific tie order. When empty, run cleanup/retire item. Do not drain unrelated backlog. |
| `combat_victory_candidate` | Enemy group qualifies basically-dead | Preserve current entered stack; filter only named manager/card containers per local rule. Continue immediate/mandatory causal outcomes (including selected Reaper last-kill heal) before victory unless actual gameplay interruption. Then commit room victory/rewards through owning continuation. |
| `player_death` | Actual source damage/death branch commits death | Set terminal immediately after synchronous rescue alternatives fail. Do not dispatch pending heal or revive after this point; retain earlier commits and stop normal progression. |
| `escape` | Monster escape timer/logical escape branch resolves | Mark `escaped`, not killed/dead/Fatal. Evaluate room group terminal predicate only after all relevant actor states settle; no kill reward for escape. |
| `handoff` | Boss rewards, mandatory children and successor init complete | Commit `Act1Complete/Act2Ready`, capture task +1 once and stop before first Act2 decision. Legal skipped rewards need not be claimed. |
| `fault` | Any post-dispatch semantic/encoding/allocation/content binding fault | Preserve actual committed prefix; quiesce pool, no rollback and no ordinary terminal reward/death/reset. |

This is **semantic immediate-causal policy**, not a source-frame emulator. Where source code emits multiple causally simultaneous effects from one enumerated list, use that exact inspected iteration/insertion order. Where no source ordering distinguishes two consequences, native site contract declares stable ordered source ordinal before dispatch; do not infer wall-clock, animation or pointer order. Do not use a universal global priority between unlike containers.

## 3. Source-sensitive ordering and effect-site joins

### 3.1 Shared dispatcher checkpoints (F; selected use is P/S)

| Semantic site ID | Source/member join | Ordered behavior retained | Pending vs entered boundary |
|---|---|---|---|
| `card.manager.admission` | `GameActionManager` card selection; `AbstractCard.canUse` | CanUse before normal onPlay; set energy/ignore flags, then powers → monster powers in list order → player relics → stance → blights → hand/discard/draw cards; record history on admitted normal path | Late null/dead/escaped target failure after this checkpoint does not reverse history; omits player use/payment. |
| `card.player.use-entry` | `AbstractPlayer.useCard` | Recalculate damage; cap X only under `!ignoreEnergyTotal`; call `use`; construct `UseCardAction` and run constructor callbacks; other-card callbacks; remove from hand; ordinary debit after entry callbacks subject to flags/Corruption-Skill | Payload often remains queued; entry is not payload completion. |
| `card.after-use.cleanup` | `UseCardAction.update` | Player then monster after-use powers; purge and Power return early; ordinary route clears one-shot free flag then exhaust/rebound/shuffle-back/return/discard precedence | Cleanup is a separate pending action; no assumed universal hand destination. |
| `manager.actions` | `GameActionManager.update/getNextAction`, `addToTop/addToBottom` | Current entered item first; thereafter ordinary `actions[0]`, then `preTurnActions[0]`, then card queue, then monster turn work; top inserts at index0 and reverse insertion order | One update runs at most current action's update. No recursive global drain. |
| `manager.card-queue` | `GameActionManager.addToQueue`/`cardQueue` | Immediate insert index1 when nonempty, else index0; queue object dedupe; one null end-turn marker | Ordinary and pre-turn actions outrank queue. Producer insertions do not resolve inside current call. |
| `power.apply` | `ApplyPowerAction.update`, `AbstractCreature.addPower` | Source `onApplyPower`; target validity and named relic prevention; Artifact; stack matching IDs except Night Terror or append; only new action-path power sorts by `this.priority-other.priority`; initial/modify callbacks follow | Stable equal key only for that sort operation; direct addPower stacks/appends without sorting. No global power re-sort. |
| `player.draw.actual` | `AbstractPlayer.draw` | Card trigger while draw-top; membership change; power then relic draw notifications; no-arg wrapper refresh afterward | Draw actions have their own No Draw/hand/soul/shuffle checks; movement/generation is not draw. |
| `card.exhaust.actual` | `CardGroup.moveToExhaustPile` | Relics → powers → card callback → move/remove → refresh | Dark Embrace/Feel No Pain children queue; Dead Branch samples at relic callback. |
| `card.manual-discard` | `DiscardAction` / `GameActionManager.incrementDiscard` | Move/refresh, card manual-discard callback, then increment; Eviscerate/relic broadcast only when neither turn-ended nor end-turn discard | Trigger-created work waits in own container. |
| `turn.marker` | Manager null end-turn marker | Relic/blight; pre-card powers; orb producer; hand `triggerOnEndOfTurnForPlayingCard`; stance | Not the later end-player-turn card callback. |
| `turn.discard-action` | `DiscardAtEndOfTurnAction.update` | Retained cards to limbo and restore queued; ordinary discard actions top-inserted unless Pyramid/Equilibrium; clone of remaining hand shuffled, then `triggerOnEndOfPlayerTurn` visited | Entrench's latter callback top-inserts Ethereal cleanup above pending discard; it is not itself Ethereal. Clone does not recursively expand for new reaction cards. Shuffle has no explicit gameplay RNG argument; native deterministic semantic order replaces it without asserting source shuffle law. |
| `combat.postbattle.filter` | `GameActionManager.clearPostCombatActions` | Selective pending ordinary-action filter retains HealAction, GainBlockAction, UseCardAction and DAMAGE-typed actions; card queue cleanup separate | Does not clear pre-turn, monster snapshot, entered stack, room objects, effects/souls or all current action. |
| `combat.endbattle.clear` | `GameActionManager.clear` | Clears named action/pre-turn/card queues/current markers and named histories/turn fields | Does not undo committed state or establish universal all-container drain. |
| `player.damage` | `AbstractPlayer.damage` | Detailed matrix §6.1 | Callbacks/counters commit before rescue/death; no rollback. |
| `monster.damage` | `AbstractMonster.damage/die/update/escape` | Detailed matrix §6.2 | `isDying`, `isDead`, `isEscaping`, `escaped`, and room-over are distinct. |

“Current callback list” means the source's actual live list at that dispatcher. Do not replace it with a snapshot or a globally re-sorted callback list. Native source-local tie order retains named player/relic/power/card/monster loop ordering; cross-family work is ordered only when a traced caller establishes order.

### 3.2 Cost, flags, captures and copy provenance

| Field/operation | Contract and guard | Receiver/history rule |
|---|---|---|
| `base_cost` | Signed-i32 base/current-combat entitlement: ordinary includes zero; X sentinel −1; unplayable sentinel −2 | Distinct from mutable `costForTurn`, captured X, free flag and payment. |
| `cost_for_turn` | Separate signed-i32 state with guarded setter | `setCostForTurn` only changes an already nonnegative turn-cost instance; negative requests clamp to zero. X/unplayable negatives stay negative. |
| `free_to_play_once` | Independent per-instance boolean | Setup and Forethought set it only when mutable combat `cost > 0`, not printed/base or turn cost; no numeric-zero mutation. Existing true survives failed guard and ordinary reset; actual cleanup clears after after-use callbacks. Other explicit writer such as Confusion may clear. |
| `autoplay` / `ignore_energy_total` | Independent play flags; not general canUse bypass | Five repeat powers set both explicitly true and copy original `energyOnUse`; Omniscience queue request uses autoplay=true/ignore-total=false, later card manager derives ignore-total=true. Preserve queued request provenance separately from derived play context. |
| X capture | `energy_on_use` captured at source; sentinel −1 means use live energy under each action rule | Ordinary use only caps capture against live energy when not ignoring total; Chemical X adds +2 per X action, not pre-added once globally. Each nonfree action debit is live-energy according to its own branch. |
| `Eviscerate.makeCopy` | Fresh returned card begins cost 3 | Under map/combat guard, setter mutates **source** from source combat cost minus `totalDiscardedThisTurn`; not returned copy and not aggregate combat discard history. |
| `BloodForBlood.makeCopy` | Fresh returned object | With player, update by `−damagedThisCombat` applies to returned object; not source. |
| `SearingBlow.makeCopy` | Counter constructor stores `timesUpgraded` but starts damage 12 | Stat-equivalent path separately replays upgrades/restores selected stats. Do not assume constructor replay. |
| Stat-equivalent copy | Call source `makeCopy` first (including source side effects), then selected upgrades/stat/cost/misc/bottle/free fields from then-current source | New physical object; distinct identity. Same-instance helper shares source UUID/linkage but is not alias. Not every destination/play flag is copied. Expunger additionally restores X magic values and description list. |
| `MasterfulStab` | Fresh instance starts cost zero; qualifying positive damage-loss dispatch invokes its updater once per notified instance | Player notification visits current hand, discard, draw, not exhaust; Exhume transfers same exhausted instance with no missed-event catch-up. Not a formula from global HP loss. |
| Transmutation output | Actual-character colorless pool excluding HEALING; no self-ID exclusion; select then fresh-copy/upgrade/turn-setter/temp card | Transmutation can generate Transmutation. Nonnegative turn cost can become zero; generated X keeps negative sentinels and gets no new free flag through this path, absent unrelated writers. Producer's free state does not transfer to children. |
| Forethought+ | Ordered selected instances, any sequence including empty | Preserve exact selection and bottom insertion order; do not sort as set. Ordinary Forethought requires one when a selection is available. |

`PlayOrigin` enum: `Hand`, `TopDeckAutoplay`, `RepeatPower`, `OmniscienceOriginal`, `OmniscienceCopy`, `GeneratedOffer`, `PotionQueuedPlay`, `ReplaySource`, `OtherExplicitSource`. `PaymentRuleId` enum: `OrdinaryTurnCostAfterUse`, `OneShotFree`, `AutoplayNoOrdinaryDebit`, `XActionLiveDebit`, `CorruptionSkillExempt`, `NoUseNoDebit`, `SourceSpecific`. Each source-site row names one exact policy; `SourceSpecific` is not permission to omit the table entry at catalog freeze.

The ten X action families have exact separate gates: Whirlwind/Skewer/Body positive hit/block count; Malaise/Doppelganger/Tempest/Collect upgraded count+1; Multi-Cast requires current orb before effect **or payment**; Transmutation raises capture to at least live energy and generates at adjusted count; Conjure Blade constructs Expunger even at zero count and nonfree payment is outside the positive-count test. Retain §4.1.2's details for effect payloads; no single X rule overrides them.

### 3.3 Callback/effect-site manifest for native joins

Stable names below are semantic aliases for the existing native.34 source-site registry; native owner binds them to its assigned namespace/owner/operation/variant codes and exact catalog rows. This table is an exact required join set for this contract, not a claim all content-specific effect sites are exhaustively merged here.

| Semantic site ID | Member/caller family | Effect class / expected trace key |
|---|---|---|
| `A1.EXEC.CARD.ADMIT` | `GameActionManager` card play branch | legality→admission history→notification path; late target rejection |
| `A1.EXEC.CARD.USE` | `AbstractPlayer.useCard`, card `use` | capture/X/use-entry/callback/payment checkpoints |
| `A1.EXEC.CARD.AFTER` | `UseCardAction.update` | after-use callbacks and destination cleanup |
| `A1.EXEC.CARD.COPY` | `AbstractCard.makeCopy`, `makeStatEquivalentCopy`, same-instance copy | source and destination identity/capture/mutation record |
| `A1.EXEC.CARD.X` | ten X action update families | captured basis, modifier, payload, payment gate |
| `A1.EXEC.QUEUE.ACTION` | `GameActionManager.update/addToTop/addToBottom/addToTurnStart` | ordinary/pre-turn/current selection, ordered insertion |
| `A1.EXEC.QUEUE.CARD` | `GameActionManager.addToQueue`, `cardQueue` | card queue provenance vs derived play flags |
| `A1.EXEC.POWER.APPLY` | `ApplyPowerAction.update`, `AbstractCreature.addPower` | prevention/artifact/stack/sort/initial callbacks |
| `A1.EXEC.CARD.DRAW` | `AbstractPlayer.draw`, `DrawCardAction.update` | actual draw vs movement, callback and capacity boundary |
| `A1.EXEC.CARD.EXHAUST` | `CardGroup.moveToExhaustPile` | relic/power/card ordering and deferred reactions |
| `A1.EXEC.CARD.DISCARD` | manual/end-turn discard dispatchers | cause-specific callback and count history |
| `A1.EXEC.TURN.START` | initial-room and repeated-turn start dispatch | separate pre-draw/draw/post-draw producers |
| `A1.EXEC.TURN.END` | manager marker, room endTurn, discard action | marker versus later callback, clone and Ethereal work |
| `A1.EXEC.DAMAGE.PLAYER` | `AbstractPlayer.damage` | HP loss, counters/card notifications, rescue/death |
| `A1.EXEC.DAMAGE.MONSTER` | `AbstractMonster.damage` | target callbacks, block/HP, death/escape state |
| `A1.EXEC.DEATH.FATAL` | enemy death hooks and per-card Fatal payload sites | attributed final kill and exactly-once payout |
| `A1.EXEC.DEATH.SPLIT` | split enemy death/summon sites | parent death plus later child insertion/turn/death |
| `A1.EXEC.HEAL.REAPER` | Reaper action, heal action and postbattle filter | final-enemy native heal after immediate causal chain; no post-death heal |
| `A1.EXEC.COMBAT.VICTORY` | `MonsterGroup` room endBattle, manager filter/clear | candidate→causal settlement→reward parent |
| `A1.EXEC.COMBAT.ESCAPE` | monster escape, thief gold ledger/refund | escape distinct from death/Fatal/victory |
| `A1.EXEC.CONT.NEOW` | Neow reward/event activation/update | drawback/reward children and one-time parent resume |
| `A1.EXEC.CONT.SHOP` | shop purchase/purge/restock and relic onEquip | payment/offer commit before child, no double charge |
| `A1.EXEC.CONT.EVENT` | `EventRoom.update`, event result/Proceed | event-owned return vs direct-map exceptions |
| `A1.EXEC.CONT.REWARD` | reward row claim, screen, Bowl, skip/take | row consumption and child obtain are separate |
| `A1.EXEC.CONT.RELIC` | installed relic result-consumer update | parent may be absent; latch owns result consumer |
| `A1.EXEC.CONT.CAMPFIRE` | `RestRoom` option construction/selection | initial filter→Recall append→final scan; heal/onRest child |
| `A1.EXEC.CONT.NOTE` | `NoteForYourself.buttonEffect`, transition persistence | incoming obtain/hooks before outgoing candidate build |
| `A1.EXEC.CONT.MATCH` | `GremlinMatchGame` update/picks/pair/cleanup | five attempts, reveal and ordered semantic cleanup |
| `A1.EXEC.HANDOFF.ACT2` | boss treasure, next-dungeon and game transition | reward/children then same-player Act2 initialization, before first decision |

When native.34 provides a specific assigned ID for the same member family, this document's semantic ID is the expected trace join key; it must map one-to-one to the existing source-site row. Multiple caller variants remain separate rows when route changes payment, destination, ordering, continuation owner or generation policy. Missing/duplicate joins are validation failures, not permission to roll up under a transitive “callback covered” count.

## 4. Damage, Fatal and terminal ordering

### 4.1 Player damage (F)

| Ordinal | Source checkpoint | Commit/continuation rule |
|---:|---|---|
| 1 | Read packet/output; remember prior Block; clamp negative input; apply Intangible; handle Block except HP_LOSS path | No HP mutation yet. |
| 2 | Modifiers: source-self relic, owner powers when owner exists, target relic, target powers | Preserve order and null-owner branch. |
| 3 | Player relic `onAttack`; for nonnull owner: owner powers `onAttack`, target powers `onAttacked`, target relics `onAttacked` | Do not synthesize target-rel relic pass on null owner. |
| 4 | Target relic `onLoseHpLast`; capture `lastDamageTaken=min(modified amount,current HP)` | Later `onLoseHp` amount edits do not rewrite this capture. |
| 5 | Positive-loss path: relic `onLoseHp`, power/relic `wasHPLost`, source `onInflictDamage`; commit HP-loss/type counters, turn/combat damage counters and subtract HP | Zero/prevented amount does not take this branch. `HP_LOSS` counter differs from ordinary damage. |
| 6 | In combat, notify current hand → discard → draw instances once each (not exhaust); increment damage-event history; update HP/bloodied hooks | Notification once per instance/event, not per HP point. Earlier commits persist. |
| 7 | If HP<1 and Bloom absent: consume Fairy first, else unused Lizard Tail; synchronously call ordinary heal path and return before death assignment | If no rescue succeeds, commit actual death and clear remaining Block. No later queued heal reverses death. |

Actual player heal order is relic modifiers in relic order → power `onHeal` in power order → HP addition/cap, with dying guard. Max-HP mutations and prior counters are already committed. A min-one raw rescue amount does not imply arbitrary altered modifiers return positive HP; the inspected legal non-dying amount-preserving/boosting case does.

### 4.2 Monster damage, fatal attribution, split and escape (F/S)

| Ordinal | Source checkpoint | Commit/continuation rule |
|---:|---|---|
| 1 | Intangible/output; if dying or escaping return; clamp negative; process Block | No player rescue/card-history path. |
| 2 | If player-owned source: player relic damage modifier; owner powers; target powers; player relic attack notification; target power `wasHPLost`; owner power attack then target power attacked | Target-power `wasHPLost` loop is not inside positive-damage guard. No player-target-relic attacked branch here. |
| 3 | Capture `lastDamageTaken=min(amount,current HP)` before HP subtraction; subtract positive amount | No player `onLoseHp` sequence or player damage history. |
| 4 | HP≤0 calls `die(triggerRelics)` once: set `isDying`; target power death hooks and player relic monster-death hooks only when enabled; group basically-dead queue cleanup is later | `isDying` is not `isDead`; one death hook set only. |
| 5 | Resolve source-specific on-death/Fatal and split payloads under their own predicates, then inserted child actors | Kill reward requires actual qualifying attributed kill and exact site predicate. A split child is new later actor; parent hit/death does not pay the child’s kill reward. Suicide/cannotLose route may suppress ordinary kill hooks. |
| 6 | Death timer/logical death boundary sets `isDead`; only all-dead, not room-over, not `cannotLose` can call `endBattle` | Room terminal candidate is distinct from hit and death callback. |
| 7 | Escape transition: `isEscaping` then `escaped` at its own completion; group predicate may end battle if otherwise valid | Escape is not kill/Fatal/death/reward. Thief stolen-gold ledger and actual gold reduction/refund are separate committed effects. |

Fatal site contract is per effect: record `{site_id, source_instance, target_actor, qualifying_predicate, commit_checkpoint, reward_key}`. Predicate must distinguish (a) this effect was the attributed source of a lethal HP transition, (b) target's actual first `die` transition invoked the relevant hook, and (c) terminal room completion. A hit on a split parent, a dying target, `cannotLose` suppression, suicide, escape or a duplicate callback cannot satisfy the site unless that particular accepted primary rule says so. The once-only reward key is source invocation + qualifying target death transition; never group-aggregate simultaneous foes into duplicate payout.

The accepted primary records give representative amounts without making them one universal Fatal handler: Feed's qualifying kill increases max HP by 3/4 then requests ordinary heal of the same amount through heal modifiers/cap; Hand of Greed's qualifying kill grants 20/25 gold subject to gain prevention; Ritual Dagger gains 3/5 damage per qualifying kill. All exclude half-dead/Minion where their recorded action does; split alone is not Fatal, while a later eligible child kill may independently qualify. Reaper's heal is based on its own resolved damage/target path, not those fixed reward amounts. No task reward is granted per enemy kill.

### 4.3 Combat terminal arbitration (S/P)

1. Preserve already-entered call and synchronous source callback order. No terminal filter may unwind the stack or reverse durable growth, gold, HP, relic counters, card membership or committed keys.
2. A lethal player event reaches actual death only after synchronous rescue alternatives fail. `PlayerDeath` is terminal; no queued heal, Feed, reward, or Act2 transition after it.
3. Last-enemy damage creates `CombatVictory` candidate, not immediate room success. Finish the resolving item's immediate effects and mandatory causal descendants in the declared site order. Reaper on a last-enemy kill heals after its immediate damage/retaliation callbacks, using the selected native rule; if player death is already committed, it does not heal.
4. Apply only the local post-combat filters to their stated containers. Never equate filter or later manager clear with all-container drain. Do not invent extra enemy turn to finish unrelated queue work.
5. Commit victory rewards and room return through the owning combat/reward continuation. Fatal payouts require their exact attributed kill. Splits, escapes, stale callbacks and duplicate death notifications cannot produce another reward.
6. `Act1CompleteAct2Ready` is later still: resolve eligible rewards/mandatory choices/transition effects, initialize same-player Act2 map and required state, stop before Act2's first policy decision. `+1` exactly once there; actual death `−1`; all other intermediate/fault outcomes reward zero.

**Reaper source fact versus policy:** static source traces establish zero-duration visual work, delayed vampire damage, `lastDamageTaken` sum, optional heal scheduling, filtering and a later clear race. They do not establish a natural lost-heal or guaranteed heal timing under real frames. The selected native final-kill heal rule is normative, not a claim about the source timer outcome.

## 5. Recursive continuation grammar

A continuation is `{kind, owner, parent?, source_site, phase, resume_checkpoint, captured_offer?, selected_order[], remaining_eligible[], committed[], children[], outcome, terminal_disposition}`. `owner` is exactly one source consumer (`Run`, `Room`, `Combat`, `Event`, `Shop`, `Reward`, `Relic`, `Campfire`, `Neow`, `BossHandoff`, or `Profile`); absence of an active screen does not erase owner. `children` are an ordered list of child continuation identities, not assumed to be LIFO. Source-owned result consumers may execute from room/installed-relic updates after the original parent screen has closed.

```text
Continuation ::= Root
Root ::= Creation → Neow? → Map → Room
Room ::= CombatRoom | EventRoom | ShopRoom | CampfireRoom | ChestRoom | BossRoom
CombatRoom ::= Combat(parent=Room) → CombatResult → Reward? → owning Room resume
EventRoom ::= Event(parent=Room) → (Selection | Combat | Reward | Obtain)* → EventResultConsumer
ShopRoom ::= Shop(parent=Room) → (PurchaseCommit → ObtainChild? | PurgeSelection → PurgeCommit | RelicChild)* → ShopResume
CampfireRoom ::= Campfire(parent=Room) → OptionCommit → (GridChild | RestChild | KeyChild)? → CampfireResume
ChestRoom ::= Chest(parent=Room) → (Reward | BossRelicSelection | DeferredKeyEffect)* → Chest/Room consumer
BossRoom ::= BossCombat → BossRewards → BossTreasure → (BossRelicChild | OtherChild)* → NextDungeonCommit → Act2Init → Act2Ready
SelectionChild ::= Offered(parent, source, ordered_candidates, min, max, committed_prefix)
                  → SubmittedOrderedSelection → SourceConsumer → ParentResume
ObtainChild ::= Constructed(source, instance, capture, hooks_state, destination)
               → EnteredUpdate → inserted | prevented | completed-without-insert
                  → source consumer if one exists
Terminal ::= ActualDeath | Act2Ready | AdministrativeFault
```

Grammar rules:

- `Offered` is immutable once generated. Skip/close/reopen preserves the source row and does not reroll; accepted Take consumes at its documented source checkpoint, which may precede deferred insertion.
- `SubmittedOrderedSelection` preserves sequence when source semantics iterate sequence (Forethought+, multi-target picks, ordered candidate groups). Where selection is a subset with source-defined unordered semantics, retain the exact selected instances and source order only as required by its caller.
- Each child owns its own committed keys and source result consumer. Parent resume is allowed only after that consumer reports its branch outcome; a closed screen, queued effect, room-complete flag, or relic ownership alone is not completion.
- Before mutation, revalidate object identity, current membership, eligibility, available count, payment and parent checkpoint. A stale or cross-parent handle rejects; it does not become another legal choice or silently rebind by index.
- Cancellation is source-specific: pre-commit cancel may leave parent untouched; post-commit child cancellation retains payment/mutation and returns only by source-defined route. No universal rollback.
- Actual death interrupts active continuation chain at its source boundary; already committed mutations persist, but no uncommitted parent result is fabricated. Administrative fault quiesces the pool without death/result/retry.

### 5.1 Concrete nested-parent and selector contracts

| Parent/site | Choice cardinality and source order | Commit, child outcome, and resume |
|---|---|---|
| Neow | Full offers use source category structure (six first-category, five second-category options; eligible four drawbacks/reward rules); drawback activation precedes reward. Single transform selects one; two-card transform commits removals of both inputs before generation. | Generated automatic card/reward histories may commit before removal/skip. Resolve each child once; no second reward activation. Potion reward Proceed follows Neow map branch. |
| Shop purge | Cancellable grid; no cost at opening. Confirm one current removable candidate to enter purge consumer. | Pay/advance price/remove/disable service only after accepted result, then clear selection. Cancel without selection makes no payment/consumption. Both screen and room consumers must not double-charge. |
| Shop card/relic | Purchase current stock item. Card purchase schedules Fast obtain before spend/restock mutation; relic install/payment commits before `onEquip` child and restock completes. | Child may suspend merchant while installed-relic consumer handles it. Return to same inventory, no repeat purchase/restock. Potion full belt/Sozu shop refusal commits neither spend nor stock removal. |
| Card reward | One fixed current row; `Take`, `Skip`, `Bowl` are distinct where enabled. | Take consumes row before Fast obtain; Skip leaves row/reopen identity intact; Bowl applies maxHP +2 then normal heal +2 and removes only associated item. Child insertion is later. |
| Campfire | Construct initial options, filter each against held relic (first rejection short-circuits that option), append eligible Recall, then final usability scan. Smith/Toke candidate rules owned by option. | If none usable after final scan, room completes; never invent choice. Smith/Toke selection then cancel returns to menu; Rest heal before onRest/Dream child. Recall/Ruby commits key only in later effect. |
| Event fight | Dead Adventurer/Mushrooms/Lab remain EventRoom combat/reward; event itself owns selection/result. | Reward Proceed direct-map exception only at these source-qualified callers. Woman in Blue opens unlabelled reward and proceeds back to event/result; do not generalize Lab path. Event fight does not become MonsterRoomElite or consume normal/elite queue head. |
| Relic-owned child | Bottles typed nonempty; Mirror deck grid; Astrolabe takes up to3; Cage up to2; Pandora starter-filtered; Bell relic offers after curse update; delayed result may be consumed by installed relic while parent absent. | Commit each relic's local effects, exact empty branches in §5.2, and then use its actual result consumer. No universal relic-child screen stack or map return. |
| Note | Take/Ignore. On Take, direct incoming obtain hooks and master-deck insertion happen first; then construct outgoing removable candidates. Candidate sequence is the now-live deck order after incoming; incoming may itself be removed if removable. | Selected removal hooks precede retaining live outgoing reference; ID/upgrades persist on next room transition, not at selection. Ignore has no outgoing selection/persistence. Direct incoming bypasses Omamori obtain-effect constructors. Default stored card is Iron Wave/0 in each independent episode. |
| Match and Keep | At A0 12 instances/six pairs; five attempts; no initial strategic Leave. First pick reveals one; second reveals another. Match by ID. | Second reveal is immediate; pair resolution consumes exactly one attempt. Match removes pair and schedules Show obtain from fresh `makeCopy` of first face; miss flips down but remembered knowledge stays. Fifth resolution ends attempts, then deterministic semantic cleanup of remaining board and Complete; no sixth attempt. |
| Boss handoff | Boss reward Proceed leads boss treasure; each eligible relic take/skip and child is source-owned; chest/rewards can be legally skipped. NextDungeon may have warning/no-pick branch. | Commit next-dungeon once, then same-player successor transition/reset/heal and Act2 map/state init; expose Act2Ready before first policy decision, hide future map details per public visibility contract. |

### 5.2 Finite exceptional selector outcomes and reachability

| Selector state | Exact local contract | Reachability statement |
|---|---|---|
| Generic fixed-count grid receives fewer candidates than requested | Does not clamp count, auto-skip or invent selection. | Whether committed zero-candidate entry is reachable is not proved; preserve actual caller gate/branch. |
| Empty Cage | Zero purgeables: selection-complete latch is set but room remains earlier `INCOMPLETE`; one/two: automatic removals and `COMPLETE`; >2 requires two selected. | Source-local branch only. No legal initialized history witness/exclusion established here. |
| Purifier, Transmogrifier, Wing mandatory removal grid | Committed grid has no local empty guard; Wing damage is already before later selection. No postcommit Leave. | Reachability not proved; no fabricated fallback or claim guaranteed deadlock. |
| Note empty outgoing candidates | Incoming hooks/deck insertion precede selection; removable incoming supplies an actual candidate. If incoming is unremovable and old deck has none, local selector has no guard. | Legal initialized-history reachability unresolved; neither assert nor exclude from source branch. |
| Empty ordinary card reward/Dream | Empty card row/screen is omitted; other reward rows/Proceed survive. | Native has no empty candidate for absent row. |
| Full belt/Sozu | Full belt leaves potion reward available; Sozu reward claim can consume without insertion; Shop Sozu/full-belt refuses before spend/restock. | Exact caller-specific behavior, not universal acquire success. |
| Empty relic offer pool | Depleted common/uncommon/shop promotes with normal/front selector as specified; rare/boss Circlet fallback. Offer rejection consumes candidate under requested-tier route. | Do not turn exhaustion into generic skip or retry. |

No supplied-choice witness, forced fixture, static branch, or bounded search miss establishes legal initialized reachability/impossibility. Those remaining history proofs are not a source/runtime execution blocker for selected native semantic contract unless the whole-spec owner needs to set behavior for a newly found legal branch.

## 6. Concrete damage/continuation intersections

| Trigger | Causal descendants that must settle before terminal decision | Work that is not automatically drained |
|---|---|---|
| Player lethal hit with Fairy/Tail | synchronous Fairy-first then Tail alternate if Fairy unavailable; each ordinary heal hook/cap; then death only if still lethal | unrelated queued heal after committed death; no resurrection |
| Monster final hit with Fatal/growth | target's exact first-death callback; qualifying source Fatal payoff/growth; immediate effect children and selected Reaper heal | other pending card, monster, preturn, room/effect/soul work not causally owned |
| Split boss/slime/minion | source parent death hooks; split decision; inserted ordered child actors and their required setup before group victory predicate | parent kill payoff for child deaths; children are not synchronous actors within parent's entered damage method |
| Thief theft | stolen-gold ledger and theft action's actual gold decrement/hit in its own entered checkpoint; later refund only if source condition | no implied intervening player decision or rollback between ledger and damage where no source choice exists |
| Last enemy dies during already-entered card | finish that card's source-specified immediate use/callback tail and mandatory payoff; apply local postcombat filtering; settle Reaper heal by selected policy | do not cancel entered method, drain all action lists, run another monster turn, or infer pending card child completed |
| Escape to empty group | settle actor's own escape boundary and group terminal predicate; preserve no kill reward | escape does not trigger death/Fatal; unrelated thief/action tail not globally swept |
| Looter theft/death | Each Mug/Lunge commits ledger `min(15, gold at ledger resolution)`, then theft `DamageAction` subtracts up to 15 current gold at its first update, and applies its HP hit at a later timed completion. On Looter death, the ledger—not a recomputed decrement—feeds the `STOLEN_GOLD` reward; claiming that reward uses ordinary gainGold restrictions. | Ledger, gold decrement and hit are three commits. Escape gives no ledger refund or Fatal credit. Common-gold suppression checks all retained monsters escaped; mixed fight with a killed survivor differs. No invented player decision between ledger and theft. |
| Boss/reward choice | finish source-owned reward/child and transition mutation, then required Act2 initialization | skipped optional row/relic need not be force-claimed; no Act2 policy action |

## 7. Expected trace contract and native join obligations

Every action trace record has the exact semantic fields:

`{run_id, item_id, parent_item?, source_site_id, transition_tag, origin?, source_instance?, target_actor?, captured_energy?, ignore_energy_total, autoplay, free_once_at_admission, captured_x?, payment_rule_id, callback_ordinal?, container_id?, causality_relation?, commit_checkpoint?, continuation_id?, outcome_tag}`.

Every effect record has:

`{effect_id, producer_item, site_id, effect_kind, source_instance?, target_actor?, captured_payload, relation, queue_container, enqueue_ordinal, entered, committed, cancelled_reason?}`.

Every continuation record has:

`{continuation_id, owner_tag, parent_id?, site_id, phase_tag, resume_checkpoint, offer_identity?, eligible_instance_refs[], ordered_selected_refs[], min_count, max_count, committed_keys[], child_ids[], outcome_tag, terminal_disposition}`.

Required invariants: (1) exactly one active entered item per run; (2) effect provenance links to a registered site; (3) each mandatory causal effect reaches exactly one commit/cancel/interrupted outcome; (4) unrelated backlog never acquires a causal parent by queue proximity; (5) committed checkpoint keys are unique and monotone; (6) continuation has one owner and valid parent/resume site; (7) no transition from actual death to heal/alive/Act2Ready; (8) Reaper final-kill selected heal is emitted exactly once if player not dead and no explicit interruption; (9) no Fatal payout without exact attributed-kill predicate; (10) skip/close cannot mutate an uncommitted offer except where the source site itself commits a skip; (11) payment occurs only at named checkpoint and at most once; (12) captured fields are immutable values, source-mutated fields retain their actual receiver identity.

These fields join existing native.34 source member/site registry and later private core state; they do not specify wire encoding, descriptor hash, action vocabulary expansion, RNG derivation or a source parity trace format. The separately assigned native owner must bind the listed site aliases to accepted literal namespace/owner/operation/variant rows. A missing row or field is an open join, not a closed transitive call.

## 8. Future discriminators, all UNEXECUTED

Use the consolidated T cases as cross-document IDs. These source/model/static checks have not executed gameplay.

| Existing case | Expected settled-model result |
|---|---|
| `TX01` | ApplyPowerAction new insertion sorts by priority, stable equal-key order for that sort only; direct `addPower` stacks/appends without sorting. |
| `TX02` | Enqueued child waits until current call returns and is selected by its own list; no recursive drain. |
| `TX03` | Ordinary before pre-turn before card queue; two top insertions reverse. No universal queue order. |
| `TX04` | Early legality rejection has no normal play history; admitted then invalid target keeps admitted history but omits use/payment; suppressed-notification cleanup remains separate. |
| `TX05` | Distinct owner-sensitive damage path and capture-before-hook amount. |
| `TX06` | Later `onLoseHp` amount change does not rewrite earlier `lastDamageTaken` capture. |
| `TX07` | Once-per-instance notifications hand/discard/draw not exhaust; zero/prevented loss has no positive-loss branch. |
| `TX08` | Synchronous Fairy before Tail; forced Bloom/dying is negative control only. |
| `TX09` | Death callbacks, `isDying`, later `isDead`, split child, `cannotLose`, escape and victory remain separate; no kill/Fatal credit from escape/split alone. |
| `TX10` | Reaper last-kill heal selected after immediate damage effects unless actual death/interruption; no natural frame outcome claimed. |
| `TX11` | Amount derives from each eligible monster's captured `lastDamageTaken`, not raw packet. |
| `TX12` | Semantic Match: visible reveal order, exactly five attempts, pair resolution and cleanup; no exact-zero/frame-underflow requirement. |
| `TX13` | Offer row, Bowl and deferred obtain checkpoints occur once at named commit boundaries. |
| `TX14` | Purchase, transform, relic child and boss choice commit once; resume at actual consumer. |
| `TX15` | Cage/fixed-grid exceptional counts remain separate from reachability. |
| `TX16` | Note incoming/outgoing order and removable versus unremovable candidates; no history claim. |
| `TX17` | Preserve already committed mutations across filter/clear; no global drain or rollback. |
| `TX18` | Entrench's latter callback queues Ethereal cleanup before already pending discard, without claiming seeded source shuffle. |
| `A1C-T10` | Legality/queue/cost/copy plus immediate mandatory causal settlement, no frame scheduler. |
| `A1C-T14` | Separate damage/rescue/death/escape/split/Fatal; selected Reaper final-enemy heal after immediate work and never post-death. |
| `A1C-T18` | Five-attempt semantic Match; pure elapsed gate is satisfied without Wait/clock; no source RNG correlation. |
| `A1C-T24` | Note direct incoming-before-outgoing, hooks and legal self-selection; independent Iron Wave/0 default; no mandatory partial profile staging. |
| `A1C-T25` | Boss children/rewards and Act2 init resolve before +1, stop pre-first decision, hidden successor facts remain hidden. |

Required negative/control pairs are explicit: accepted/late-invalid vs early-rejected card; queued-vs-entered child; one vs two priority equal/insertion paths; captured vs live X; source-mutating Eviscerate vs returned-object Blood for Blood; Stab hand/discard/draw vs exhaust; zero vs positive damage; Fairy+Tail vs Bloom negative control; dying vs dead vs escaped actor; last-kill Reaper vs post-death attempt; Cage 0/1/2 and mandatory empty grid vs guarded empty selector; removable vs unremovable incoming Note; Take vs Skip vs Bowl; source child committed vs merely queued. These are acceptance specifications, not executed results.

## 9. Before/after closure disposition

| Requirement | Before this document | Concrete delta | Remaining exact limit |
|---|---|---|---|
| B04 | Direct family inventories and selected card examples; mixed action/cost/copy interactions open | Exhaustive semantic state records, transition functions, origin/payment/capture/copy/history matrix and five-repeat/Omni flag distinction; named effect joins | Not every product of content callbacks has been executed or independently source-audited. Bind all literal sites to native.34 when main routes accepted catalog. |
| B05 | Source-specific continuation rows without exact common causal/terminal interface | Recursive owner grammar, ordered commit/resume rules, selector cardinalities/branches, no global return/drain | Exceptional mandatory-empty initialized history reachability unresolved; no invented fallback. |
| B06 | Bounded player/monster branches and rescue/death facts, no combined terminal policy | Exact damage pipelines, Fatal/split/escape distinctions, once-only reward gate, death/victory/handoff arbitration | All cross-product reactions and future trajectories unexecuted. Natural Reaper timer outcomes not established (selected policy settles its result). |
| B07 | Primary tables and dispatch families separately; callbacks count did not close joins | Explicit dispatcher/member ordering and fixed semantic site alias table, actual commit/receiver/callback joins | Alias-to-native.34 literal ID equality depends on downstream main-routed accepted registry; this is not claimed resolved by counts. |
| B08 | Source local offers/returns without common exactly-once child outcome | Parent-child contract for Neow/shop/events/rewards/relics/campfire/Note/Match/boss and choice revalidation | No legal-history proof for exceptional stale/empty branches; policy doesn't add source outcomes. |
| B12 | Local source timer observations; selected policy not applied as transition model | Deterministic immediate-causal settlement explicitly replaces frames; T/TX cases aligned; Reaper heals on final kill | No source-frame parity, natural timer outcome, source RNG correlation, or original runtime. |

## 10. Evidence and limitations

The source-informed facts are bounded to the accepted inputs in §1; this author did not execute or re-run original source, classes, initializers, or gameplay. Static facts/model contracts do not establish all legal initialized histories or all cross-product effects. Future `TX01–TX18` and referenced `A1C-T*` cases remain **UNEXECUTED**. This document is the assigned single public document; detailed private source records, helper, raw logs and source bytes are not published. Whole-spec human approval, reviewed implementation plan and separate implementation-start approval remain unsatisfied.
