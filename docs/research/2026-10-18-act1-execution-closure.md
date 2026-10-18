# Act 1 execution: bounded closure and remaining obligations

Date: 2026-10-18. Beads task: `spire-gli.2.2.2`.

**Status: bounded static research, not complete execution closure.** This report
narrows callback and scheduler claims, identifies an ordinary-action boundary in
the Looter theft path, and presents Reaper timing options for main review. It does
not approve a timer policy, complete the whole Act 1 specification, or establish
runtime behavior.

## Evidence boundary

Findings are pinned to the owned `desktop-1.0.jar` bytes, SHA-256
`cfad868ac8d65a88e71a0bf096fb09f78811e553effe0787c5309a655e081673`; upstream
game build/channel identity is not established. This report reuses prior reviewed
static work: 2,306 game-class scans, 52 fresh disassemblies and 119 source
inspections. These counts describe prior evidence, not a new exhaustive scan by
this task. No original game class, initializer, substituted runtime, or game
launcher ran. Static mechanics and prior file-guard tests are not gameplay tests.

## Bounded findings

### Legality, payment, copying, and callbacks

Play origin, legality, payment, captured X, callbacks, and final destination must
remain distinct. A free/autoplay queue does not establish a universal legality
bypass: earlier turn, target, card, power, relic, blight, and source-specific
restrictions remain relevant. On the inspected common successful player path,
history/callback timing precedes later dead/null-target handling; payload resolves
before cleanup/on-use callbacks, hand removal and ordinary payment. After-use
callbacks precede destination cleanup, while purge and power paths have separate
early returns.

The five repeat powers—Double Tap, Burst, Amplify, Duplication, and Echo—carry the
original `energyOnUse` value but explicitly set `ignoreEnergyTotal` and
`autoplayCard` true on the queued repeat. They do not merely preserve the original
play's flags. These queue flags do not certify a universal legality bypass.

The prior all-class scan bounded the direct `onDrawOrDiscard` override family;
that result does not close downstream `applyPowers`/glow work or other draw,
exhaust, discard, shuffle, start-turn, and end-turn callbacks. Complete
`canUse`/free/cost overrides, cost precedence, all copy-state overrides,
`queueExtraCard` callers, and transitive callback closure remain open. No
universal FIFO ordering or rollback behavior follows from the inspected paths.

### Work queues, lethal paths, and durable effects

Pending actions are not the whole execution state. The detached current action,
pending and pre-turn actions, card and monster queues, room objects, dungeon
effects/souls, and animation-driven death/escape completion are distinct. The
last-enemy pending filter does not unwind a running method or prove that those
other containers are drained. Victory hooks precede a fuller clear; that clear
does not undo committed mutations or imply that every other list is cleared.
Ordinary combat updates advance monsters before the manager, and room/effect
dispatch occurs at separate points. A death-screen transition blocks subsequent
normal dispatch but does not unwind an entered call stack or promise indefinite
future updates.

On the inspected damage-lethal path, held Fairy is considered before unused
Lizard Tail; the rescue heal runs synchronously through heal hooks before the
branch returns without assigning death. Raw minimum-one healing does not
guarantee positive HP after modifiers. This is not a complete legal prevention
or healing inventory, and rescue is not rollback of distinct queued work. Other
lethal origins, full damage-all/death/relic/power ordering, and legal modal
interruption at terminal boundaries remain unclosed.

### Looter theft interposition

The ordinary Looter attack producer queues an animation, a separate ledger
increment action, and a `DamageAction` carrying the gold amount in one
monster-turn sequence. The ledger increment and actual gold decrement are
separate checkpoints, followed by the HP hit. In that inspected ordinary path,
there is no player decision between the ledger and queued theft action; no
natural player-chosen interposition is demonstrated. A fixture that directly
mutates the ledger or gold is not a reachability witness.

This is a bounded static conclusion about ordinary player actions in that path,
not proof against asynchronous engine timing, external effects, or uninspected
mod/native hooks. No interposition was executed or observed. Looter death refunds
use its ledger amount rather than recomputing the player's actual gold decrement.

### Reaper timing and headless fidelity choices

The inspected multi-enemy Reaper damage action can trigger retaliation (including
Guardian Sharp Hide) before its later queued heal. An early pending-action filter
may retain that heal while removing other pending work; later end-battle clearing
can still remove pending/current work. This establishes a static ordering
mechanism—not a lost-heal outcome, elapsed-time threshold, or exact frame law.

Main has three explicit choices to evaluate; none is selected here:

1. Use deterministic source-order headless completion checkpoints, documenting
   that timer parity with the original is not established.
2. Model animation/death timers and relevant `FAST_MODE`/update-delta behavior as
   explicit state, after reviewing source closure and a justified mapping to
   headless steps. This research does not provide that mapping.
3. Declare a deterministic headless fidelity boundary for this case, retain the
   ordering hazard and discriminating acceptance cases, and record the deviation
   for whole-spec review.

Do not silently select a historical order, invent a timer constant, or make heal
success depend on a fabricated threshold. No gameplay cap, retry, defeat, or
universal drain rule is authorized by this evidence.

## Closure and residual matrix

| Family | Bounded result | Still open |
|---|---|---|
| Repeat-play queue flags | Five named powers explicitly enable both queue flags while copying energy-on-use. | Mixed repeat sources, legality, copy provenance, dead targets, cancellation. |
| Direct draw/discard callback | Named direct override set was scanned and inspected previously. | Transitive callbacks and other lifecycle callback families. |
| Damage/rescue | Fairy-before-Tail and one synchronous rescue branch are bounded. | Other lethal origins, full prevention/heal modifiers, all attack/damage-all/death/relic/power combinations. |
| Scheduler/terminal | Named containers, filter, and later clear are distinct. | Complete caller closure, alternate endings, legal suspended-choice interruption and durable-commit matrix. |
| Looter | Ordinary queued path has no demonstrated player-choice interposition between ledger and theft. | Timer/external-hook behavior and runtime reachability. |
| Reaper | Sharp Hide-before-heal exposure is a static mechanism. | Whether heal is lost in original runtime; headless policy and timer mapping. |

Acquisition/UI-return and Note/profile transition details, and the full 141-relic
roster, belong to the assigned continuation and relic research lanes. This report
does not duplicate or claim their closure; only shared scheduler-interface
constraints above are relevant here. Original-runtime behavior remains unknown
pending independently reviewed confinement and explicit main handoff.

## Review handoff

Private source locators and artifact identities are retained in the assigned
verification lane, not published here. Fresh independent review and main
integration remain pending. The report does not close the parent task, whole
written-spec review, implementation plan, or human approval gates.
