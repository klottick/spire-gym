# Act 1 semantic execution and continuation contract

Date: 2026-10-19 · Bead: `spire-gli.2.3.42`

**Normative native semantic contract candidate, submitted for independent review; not human-approved implementation authority.** This document supplies concrete action, settlement, damage and continuation rules for standard unmodded fully-unlocked Act-4-enabled Ironclad A0 Act 1 through resolved Act2Ready. It deliberately replaces frame progression with the selected deterministic immediate-causal settlement policy. Source facts below describe bounded static inspection, not original-game execution. All future cases are **UNEXECUTED**.

The normative policy is the accepted consolidated specification at base `2e090d78297a236bfd549797e460db40d47e709f`, especially §§1.2, 4–9, 11. It controls where historical sources differ: immediate effects and mandatory causal descendants settle before victory unless an explicit gameplay interruption applies; Reaper heals on the last-enemy kill; no post-death healing/resurrection, unrelated work drain, frame simulation, extra enemy turn, rollback, or source-FIFO claim. Independent RNG, selected profile and successor policies also remain binding. This is an independently authored producer contract for the existing native semantic/site owner; it does not define wire codecs or a new RNG algorithm.

## 1. Evidence boundary and inputs

The owned source archive is SHA-256 `cfad868ac8d65a88e71a0bf096fb09f78811e553effe0787c5309a655e081673`. No original class, initializer, game runtime, or substituted original-class runtime was run for this document. Source facts are reused from accepted, independently reviewed public inputs:

- [Transitive execution closure, exact reviewed revision](https://github.com/klottick/spire-gym/blob/4167152c5d0fce47586fe83f3d1cfc1f976af609/docs/research/2026-10-19-act1-transitive-execution-closure.md): common manager/container priorities, card admission checkpoints, player/monster damage split, rescue/death/escape, common durable commits, and the source-specific timer facts.
- [Direct execution families, exact reviewed revision](https://github.com/klottick/spire-gym/blob/f515954193bac8d36eb8e425051ba5940c5a4000/docs/research/2026-10-19-act1-execution.md): finite direct legality/cost/copy/callback families, X actions, corrected Setup/Forethought and Masterful Stab histories, Transmutation generated-X and Eviscerate receiver.
- [Source-specific continuations, exact reviewed revision](https://github.com/klottick/spire-gym/blob/2cbcd2d325f0e4d4f2294612cbfff131919e2288/docs/research/2026-10-19-act1-continuations.md): Neow/shop/reward/event/campfire/relic-child/Note/Match commit and return branches.
- Accepted [member mechanics](2026-10-19-act1-content-silent.md) ([E21 reviewed head](https://github.com/klottick/spire-gym/blob/60f97e2bbc8b4c2b35622f039bd972685fd73142/docs/research/2026-10-19-act1-content-silent.md)), [Defect](https://github.com/klottick/spire-gym/blob/40c1a2f93fb65bfa0b19d1ca1618edbdb3ccd476/docs/research/2026-10-19-act1-content-defect.md) (E22), [Watcher](https://github.com/klottick/spire-gym/blob/1f3abc31306410a0a4ea48ab73bea1351a40fd21/docs/research/2026-10-19-act1-content-watcher.md) (E23), [Red/colorless](https://github.com/klottick/spire-gym/blob/6ec9659534b19ff57cd9699bfa9fb09793e977d0/docs/research/2026-10-19-act1-content-red-colorless.md) (E24), and [enemy records](https://github.com/klottick/spire-gym/blob/e56d4ace59219d07d01dd69f117c9534f9fbe941/docs/research/2026-10-19-act1-content-enemies.md) (E20): selected exact member callbacks/payloads and branch limits used in §3.3A.2; these do not establish all transitive compositions.
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
| `TerminalLoss` | `terminal_commit`, `source_root_id`, `committed_deck_count = 0` | Settled `EmptyMasterDeckLoss`; one −1 capture, terminated true/truncated false, no HP/death mutation. No later root is admitted. |
| `TerminalVictory` | `victory_source`, `remaining_causal_work[]` | Enemy-group victory candidate only. Transition to reward/room victory after mandatory causal work settles or explicit gameplay interruption; never award Act1 success here. |
| `Faulted` | `fault_stage`, `fault_site`, `committed_prefix` | Administrative post-dispatch failure. Quiesce pool; no rollback, success, death, retry, or successful output fabricated. |

`captured_inputs` is a tagged finite record: `{energy_on_use: Optional<i32>, captured_x: Optional<i32>, target: Optional<ActorRef>, origin: PlayOrigin, source_instance: Optional<InstanceRef>, payment_rule: PaymentRuleId, free_once_at_entry: bool, autoplay: bool, ignore_energy_total: bool}`. Absent means no such source value, not zero. `origin` and flags remain independent.

### 2.2 `CausalSettlement`

| Field | Domain / invariant |
|---|---|
| `settlement_id` | Private run-scoped semantic identity; not derived from worker, address, frame or completion race. |
| `active_item` | At most one currently entered `SemanticActionState` item. Retiring it is distinct from selecting its next child. |
| `causal_stack` | Ordered records `{parent_item, child_item, edge_key, source_site, admission_checkpoint, completion_checkpoint}` for required descendants. Each child has one cause owner and once-only completion key `(parent_invocation, checkpoint, edge_key)`; ordering comes from the concrete producer row, never an unspecified source ordinal. |
| `mandatory_frontier` | Ordered finite set of immediate source effects and mandatory descendants causally required to finish the current semantic item/terminal candidate. No unrelated card/relic/event queue entries enter merely because pending. |
| `backlog` | Distinct source containers: `actions`, `pre_turn_actions`, `card_queue`, `monster_snapshot_queue`, `room_objects`, `dungeon_effects_active`, `dungeon_effects_pending`, `top_level_effects_active`, `top_level_effects_pending`, `souls`, and `continuation_consumers`. Preserve each container's own owner and ordering; they are not a unified FIFO. |
| `run_terminal` | `None | ActualPlayerDeath | EmptyMasterDeckLoss | Act2Ready`; only these are gameplay episode terminals and task reward boundaries. |
| `room_outcome` | `None | CombatVictoryCandidate | CombatWon | CombatEscaped | CombatInterrupted`; a room result is not a run terminal. An escaped-only room is not kill victory or victory loot. |
| `operational_stop` | `None | AdministrativeFault`; quiesces the pool without fabricating a game terminal, reward, defeat, rollback or retry. |
| `committed_checkpoints` | Set of once-only semantic checkpoint keys; never an undo log. Mutation already committed survives later cancellation/death/fault. |
| `decision` | Optional owned continuation choice; only a genuine strategic choice suspends the run. |

Causal relations are exactly: `ImmediatePayload` (direct state mutation in the entered producer); `RequiredCallback` (synchronous callback actually invoked before producer returns); `RequiredChild` (separately pending action explicitly emitted by the producer/callback and required by its payload); `TerminalPayoff` (once-only qualifying kill payoff); `MandatoryChoice` (source-owned selection required to complete its parent); `Unrelated` (work not required by current causal chain). `ImmediatePayload` and `RequiredCallback` commit synchronously inside the entered call and are not queued frontier entries. Only emitted `RequiredChild`, `TerminalPayoff`, and `MandatoryChoice` records can extend the frontier; `Unrelated` remains in its original container. The producer tables in §3.3A assign these relations at emission; unknown producer/relation is a content-binding fault, never an inferred class-name rule.

### 2.3 Deterministic settlement transitions

| Transition | Guard | Ordered state change |
|---|---|---|
| `submit` | Live policy decision, valid candidate and current continuation token | Create `Preflight`; validate entire batch before any row dispatch. Invalid row rejects entire batch with zero mutation. |
| `admit_card` | Actual `canUse` succeeds | Apply actual source path: set manager play context and captures; run the specified ordered notification branch; record play history only on the normal admitted branch. Later target invalidation omits use/payment, but does not erase admission history. |
| `reject_card` | Earlier legality fails | Record no normal play history, payload, ordinary payment or successful-play destination. A failed autoplay cleanup branch is not a successful play. |
| `enter_effect` | Selected item is dispatchable | Mark `Entered`; execute one source boundary/callback in its declared local order. Enqueues become pending; they do not run recursively during this call. |
| `return_effect` | Entered method returns | Commit its synchronous writes; materialize the exact source-site `CausalEffect` emission records from §3.3A; add only typed `RequiredChild` edges to this item's frontier in their mapped order. `Unrelated` records remain in their original containers. |
| `frontier_decision_suspend` | Next ordered frontier record is `MandatoryChoice` with a nonempty legal set | Do not dispatch it as automatic work. Save exact frontier head/tail, `run_terminal` and `room_outcome`, owner, ordered eligible handles, cardinality, committed prefix and resume checkpoint; set `Suspended`; publish only legal choices. No sibling frontier item, terminal commit, cleanup, payment, reroll or unrelated work runs while suspended. |
| `publish_legal_choice` | Suspended choice has an owner and current candidate set | Expose precisely the listed candidates and min/max. Mandatory choices have no Skip/Cancel unless their row below says so. Empty sets use §5.2, never an invented control. |
| `commit_choice_result` | Submitted ordered handles satisfy the row and remain eligible | Commit the source consumer's named mutation once; create exact result children at stated checkpoints; record outcome/child IDs. Stale, repeated, cross-owner or out-of-range results reject without mutation and leave suspension unchanged. |
| `resume_frontier` | Source consumer and required result children report completion or specified terminal outcome | Restore saved frontier and saved run/room outcome; mark choice checkpoint once; continue at saved head/tail. Do not re-enter producer, repay, rebuild offer or lose earlier commit. Nested child choice returns to this saved frontier. |
| `settle_item` | Current item's direct effects complete | Process the finite ordered frontier using §3.3A. Automatic effects dispatch; a decision invokes `frontier_decision_suspend`; an interruption edge stops as specified. When empty, run cleanup/retire item. Never drain unrelated backlog. |
| `combat_victory_candidate` | All live group actors satisfy the actual death/room predicate and `cannotLose` does not block it | Record room candidate, not run terminal. Finish the active producer's direct work and typed mandatory descendants, including selected final-kill Reaper heal unless actual death interrupts. Then commit CombatWon/reward through its owning room continuation; do not drain unrelated containers. |
| `player_death` | Actual player damage/death path reaches death after synchronous Fairy-then-Tail rescue attempts fail | Set `run_terminal=ActualPlayerDeath` exactly once. Preserve committed prefix; interrupt all not-yet-entered gameplay causal descendants, especially heals/revivals; no room victory can supersede it. |
| `settled_empty_master_deck` | Root satisfies §2.4 settlement predicate, permanent master-deck count is zero, and no terminal/fault was committed earlier | Set `run_terminal=EmptyMasterDeckLoss`; commit one −1 terminal capture with `terminated=true,truncated=false` and no HP/death-hook mutation. This check precedes any next root/decision/event/Act2Ready transition. |
| `escape` | An actor reaches its own `escaped` checkpoint | Set that actor's `escaped` flag; never synthesize `isDead`, Fatal, kill credit, task death, or CombatWon. Continue other live actors. If caller's actual all-escaped/no-live-actor predicate closes the encounter, record `room_outcome=CombatEscaped`; owning room handles the no-kill/no-victory-loot branch and run continues. |
| `handoff` | Boss rewards, mandatory children and successor init complete | Commit `Act1Complete/Act2Ready` as the sole success terminal, capture task +1 once, stop before first Act2 decision. Legal skipped rewards need not be claimed. |
| `fault` | Any post-dispatch semantic/encoding/allocation/content binding fault | Set `operational_stop=AdministrativeFault`; preserve actual committed prefix and quiesce pool. Do not create a gameplay terminal/reward/death/reset/rollback; if a terminal was already committed, preserve its outcome and capture facts unchanged. |

This is **semantic immediate-causal policy**, not a source-frame emulator. The producer rows in §3.3A specify typed child edges and their mapped tie keys for the bounded branches. There is no free `source ordinal unspecified` fallback: an effect absent from the table or a producer needing an unlisted tie is a binding fault pending a concrete row. Do not infer wall-clock, animation or pointer order. Do not use a universal global priority between unlike containers.

### 2.4 Settled causal roots and terminal arbitration (S/P)

#### Root and child identity

`CausalRoot` is the finite semantic invocation created by one accepted external decision, one accepted room/turn producer, or one explicit continuation result. Its identity is `(run_instance, root_sequence, root_kind, source_member, source_branch, owner_ref, admission_checkpoint)`. `root_sequence` is allocated only after whole-batch preflight succeeds; rejected input does not consume it. Root identity is not a queue ID, actor address, or completion-time value. A nested choice suspends and resumes the same root. An independently accepted decision/event is a new root and cannot be made a child merely because it was already queued.

Each root carries `phase ∈ {Preflight, Active, Suspended, Settling, Settled, Interrupted, Faulted}`, `entered_item?`, `mandatory_frontier[]`, `children[]`, `commit_ledger[]`, `settlement_result?`, and `root_token`. Each child record is `{child_id, parent_child_id?, producer_invocation_id, root_id, edge_key, producer_member, producer_branch, captured_inputs, relation, source_container, enqueue_operation, emitted_index, entered_guard, commit_checkpoint, tie_key, state}`. `child_id` is unique inside the run; `(producer_invocation_id, parent_child_id, edge_key)` is unique within its declared producer scope. The same local edge key may be reused under distinct valid parents/invocations. The owning parent and root must both be live and match when a child is dispatched, consumed, or retired. A child is `Pending → Entered → Committed → Retired`, or `Pending → Cancelled` only by an explicit interruption rule. `Committed` and `Cancelled` records retire once; repeated dispatch, completion, consume, or retirement rejects without changing state. A detached result retains its original `root_id`, owner, commit ledger and token until the designated consumer consumes it once; detachment changes storage, not causal ownership.

The root's mandatory frontier contains only effects/children explicitly emitted by the current producer rows or an accepted concrete source-member row. It excludes unrelated backlog, optional future offers and sibling roots. Every root-owned emitted child is tracked to a terminal disposition even when its `mandatory` flag is false; nonmandatory does not make an emitted child disappear from the root graph. Work owned by an unrelated producer is recorded as external work outside this root graph at emission, and remains pending without blocking or being drained by this root. `settle_root` is enabled iff the entered item has returned, every synchronous callback/write is recorded, the root-owned mandatory frontier is empty, every emitted root child is committed or explicitly cancelled under its interruption rule, and every required result consumer has consumed or retired its result. Pending replacement/obtain/creation effects block settlement whenever their declared commit can change permanent deck membership. An unresolved producer binding is a fault, not evidence that its frontier is empty.

#### Ordered root lifecycle

| Phase | Required action / guard | Result |
|---|---|---|
| `Preflight` | Validate phase/token, source binding, full batch, target and cost; validate all child/root references before dispatch. | Invalid input is neutral: no root sequence, state, history, RNG, reward or token consumption. |
| `Active` | Enter exactly one accepted root item; record source-member/branch and captured inputs before effects. Complete its synchronous body and callbacks in the declared local order. | Materialize each declared emission with its parent/root/edge identity; never infer children from container adjacency. |
| `Settling` | Process only this root's ordered mandatory frontier. Choices suspend the root; on return restore the same frontier cursor and consume the detached result once. | Ordered producer effects and descendants complete before parent retirement. Unrelated work remains in its original container and is not drained. |
| `Settled` | Require the settlement predicate above, then arbitrate any terminal candidate against committed terminal/fault axes. | Root token is consumed once; no child may be added to this root afterward. |
| `Interrupted` | Actual HP death commits, or a concrete gameplay interruption rule cancels not-yet-entered children. | Retain committed prefix; no post-death heal or revival; root outcome records interruption, not successful completion. |
| `Faulted` | Post-dispatch semantic, encoding, allocation, ownership or content-binding fault. | Preserve committed facts; quiesce pool. Never convert fault into death/loss or overwrite an already committed terminal outcome. |

#### Permanent-deck loss and terminal axes

Terminal state is an immutable tagged outcome, not a mutable priority flag: `run_terminal ∈ {None, ActualPlayerDeath, EmptyMasterDeckLoss, Act2Ready}`; `terminal_commit` records `{kind, source_root_id, commit_checkpoint, reward_delta, terminated, truncated, token_consumed}` exactly once. `room_outcome` and `operational_stop` remain orthogonal. Once `terminal_commit` exists, later faults may set `operational_stop=AdministrativeFault` and stop delivery, but may not replace, erase, or recalculate its kind/reward/terminated/truncated facts. Pool delivery/reset state is a separate record and cannot masquerade as a terminal outcome.

At `Settled`, if and only if the accepted root's completed mutation leaves permanent `master_deck.count == 0`, commit `EmptyMasterDeckLoss` before admitting any next root, ordinary decision, room/event, or `Act2Ready` transition. Delivery is one reward `−1`, `terminated=true`, `truncated=false`; it does not set HP to zero, execute death/rescue hooks, award death credit, or fabricate a death event. The predicate is evaluated against the committed permanent master-deck collection, not draw/discard/exhaust piles. A temporary zero during a transform, remove-before-obtain, multi-transform, replacement, or other mandatory child is not eligible while the same root has a pending mutation that can restore deck membership. Invalid/preflight-rejected roots and roots whose settlement faults before a valid completed predicate do not cause this loss. A last-card Toke/removal that settles with no replacement child commits the loss before another event/Act2Ready decision can enter.

Transmogrifier's retained no-op is exactly: positive permanent master-deck count and zero purgeable, non-bottled eligible card instances at its selection checkpoint. It is a nonempty-deck no-op; it settles normally and does not produce `EmptyMasterDeckLoss`. With one or more eligible instances it must select exactly one under the accepted source/policy row; no event, option, or other fallback may substitute. A future optional reward cannot rescue an already settled zero-deck root.

Terminal arbitration is chronological and typed, not an enum-priority list. (1) A previously committed terminal outcome is immutable. (2) A fault before any terminal commit leaves terminal unset and records only operational fault. (3) A fault after terminal commit preserves that terminal and its one-time reward facts, while recording a separate delivery fault; it cannot trigger a second capture, reset, or vector result. (4) During a live root, a committed actual HP death interrupts pending gameplay descendants and is the terminal outcome; a merely pending victory/loss candidate cannot outrank it. (5) After a root fully settles without HP death/fault, evaluate permanent-deck-zero loss; only if false may a combat/continuation success proceed. (6) `Act2Ready` is committed only after boss reward/mandatory choices and successor initialization settle, and is impossible after any earlier terminal commit. Escape remains a room outcome, not episode termination. Any received state claiming two incompatible terminal commits, post-terminal root admission, post-terminal token reuse, death plus successful handoff, or fault-as-loss is rejected as an impossible state without overwriting the recorded outcome.

#### Concrete trace records and typed validation

For every root and its terminal check, emit an ordered semantic trace record with fields `{run_instance, root_id, root_kind, source_member, source_branch, owner_ref, root_token, phase_before, phase_after, child_id?, parent_child_id?, edge_key?, producer_invocation_id?, relation?, captured_inputs, commit_checkpoint, permanent_master_deck_before, permanent_master_deck_after, mandatory_pending_before, mandatory_pending_after, root_children_nonterminal_before, root_children_nonterminal_after, run_terminal_before, run_terminal_after, terminal_commit?, operational_stop_before, operational_stop_after, detached_result_ref?, result_consume_count, reward_delta, terminated, truncated}`. `source_member` and `source_branch` refer to the concrete accepted producer row/member, not a guessed wire ID. Each emitted root child gets an `Emit` record even when nonmandatory; its own entry/commit/retirement each gets a separate record keyed by child ID, producer invocation, parent, and local edge key. Settlement evidence is a final `RootSettled` record whose computed predicate includes zero nonterminal root children and exactly-once result consumption; external work remains separate and pending.

Typed overwrite validation is part of the state transition: parse `terminal_overwrite_effect` as the closed enum `{None, Allow}` from the authoritative typed field. `None` rejects any terminal mutation after a terminal commit; `Allow` does not permit the runtime to overwrite terminal outcome and is accepted only for a specifically authorized nonterminal projection field. The validator must compare the parsed enum and target field: it must reject both an unknown enum value and an `Allow` value paired with `run_terminal`/`terminal_commit`. Hardcoding “overwrite forbidden” while allowing a typed `Allow` mutation to pass is invalid. No terminal record can be overwritten by either variant.

#### Required fail-closed semantic cases

The finite model/checker for this contract must reject: child rooted under another invocation; duplicate emit/dispatch/consume/retire; a settled root with pending mandatory replacement/obtain; loss tested during a transient transform-empty interval; permanent zero evaluated before an emitted replacement commits; draining unrelated backlog; offering optional rescue after settled loss; retained Transmogrifier no-op with eligible cards or an empty-deck no-op; invalid/preflight input that consumes a token; a fault reclassified as loss; HP death reclassified as loss; a post-death heal; an already committed terminal erased by later fault; an `Allow` overwrite of terminal outcome; duplicate terminal reward capture; detached result consumed twice; or new root/token admission after terminal commit. These are semantic-policy controls, not claims of runtime/gameplay coverage. EX-I1 concrete per-member callback graph, EX-I3 source selector branches, EX-I4 exceptional initialized reachability, and native wire/codebook binding remain separate producer obligations until their accepted interfaces are joined.

Authoritative machine-readable semantic schema and bounded transition vocabulary. These semantic tags are not native numeric codes or serialized bytes. Native owns codebook/version/encoding; this record supplies the typed meaning, allowed transition operands and equality relations:

```json
{
  "contract": "semantic-causal-root-terminal-v2",
  "root_phases": ["Preflight", "Active", "Suspended", "Settling", "Settled", "Interrupted", "Faulted"],
  "child_states": ["Pending", "Entered", "Committed", "Cancelled", "Retired"],
  "terminal_kinds": ["ActualPlayerDeath", "EmptyMasterDeckLoss", "Act2Ready"],
  "terminal_fields": ["source_root_id", "commit_checkpoint", "reward_delta", "terminated", "truncated", "token_consumed"],
  "operational_stop": ["None", "AdministrativeFault"],
  "room_outcomes": ["None", "CombatVictoryCandidate", "CombatWon", "CombatEscaped", "CombatInterrupted"],
  "loss": {"predicate": "SettledRoot && PermanentMasterDeckCount == 0 && PriorTerminal == None && OperationalFault == None", "reward_delta": -1, "terminated": true, "truncated": false, "hp_mutation": false, "death_hooks": false},
  "transmogrifier_noop": {"permanent_master_deck_count": ">0", "eligible_purgeable_non_bottled_count": 0, "empty_deck_fallback": false},
  "overwrite": {"field": "terminal_overwrite_effect", "enum": ["None", "Allow"], "terminal_commit_allow": false},
  "root_settlement_requires": ["entered_item_returned", "synchronous_writes_recorded", "mandatory_frontier_empty", "all_children_terminal", "result_consumed_once", "not_faulted", "not_hp_dead"],
  "token_rule": "one admission after preflight; consumed once at settlement; no admission after terminal",
  "trace_fields": ["sequence", "run_instance", "root_id", "root_kind", "source_member", "source_branch", "owner_ref", "root_token", "phase_before", "phase_after", "child_id", "parent_child_id", "edge_key", "producer_invocation_id", "relation", "captured_inputs", "commit_checkpoint", "permanent_master_deck_before", "permanent_master_deck_after", "mandatory_pending_before", "mandatory_pending_after", "root_children_nonterminal_before", "root_children_nonterminal_after", "run_terminal_before", "run_terminal_after", "terminal_commit", "operational_stop_before", "operational_stop_after", "detached_result_ref", "result_consume_count", "reward_delta", "terminated", "truncated"],
  "record_schemas": {
    "Root": {"required": ["run_instance", "root_id", "root_kind", "source_member", "source_branch", "owner_ref", "root_token", "phase", "entered_item_returned", "synchronous_writes_recorded", "mandatory_frontier", "children", "commit_ledger", "permanent_master_deck_ids", "permanent_master_deck_instances", "permanent_master_deck_revision", "event_phase"], "types": {"run_instance": "nonempty-string", "root_id": "nonempty-string", "root_kind": "nonempty-string", "source_member": "nonempty-string", "source_branch": "nonempty-string", "owner_ref": "nonempty-string", "root_token": "nonempty-string", "phase": "root_phases", "entered_item_returned": "boolean", "synchronous_writes_recorded": "boolean", "mandatory_frontier": "array-child-id", "children": "array-child-record", "commit_ledger": "array-commit-record", "permanent_master_deck_ids": "array-card-instance-id", "permanent_master_deck_instances": "array-master-deck-instance", "permanent_master_deck_revision": "nonnegative-integer", "event_phase": "EventPhase"}},
    "Child": {"required": ["run_instance", "root_id", "parent_child_id", "child_id", "edge_key", "producer_invocation_id", "producer_member", "producer_branch", "owner_ref", "relation", "state", "mandatory", "commit_checkpoint"], "types": {"run_instance": "nonempty-string", "root_id": "nonempty-string", "parent_child_id": "nullable-string", "child_id": "nonempty-string", "edge_key": "nonempty-string", "producer_invocation_id": "nonempty-string", "producer_member": "nonempty-string", "producer_branch": "nonempty-string", "owner_ref": "nonempty-string", "relation": "CausalRelation", "state": "child_states", "mandatory": "boolean", "commit_checkpoint": "nonempty-string"}},
    "Trace": {"required": ["sequence", "run_instance", "root_id", "root_kind", "source_member", "source_branch", "owner_ref", "root_token", "phase_before", "phase_after", "child_id", "parent_child_id", "edge_key", "producer_invocation_id", "relation", "captured_inputs", "commit_checkpoint", "permanent_master_deck_before", "permanent_master_deck_after", "mandatory_pending_before", "mandatory_pending_after", "root_children_nonterminal_before", "root_children_nonterminal_after", "run_terminal_before", "run_terminal_after", "terminal_commit", "operational_stop_before", "operational_stop_after", "detached_result_ref", "result_consume_count", "reward_delta", "terminated", "truncated"], "types": {"sequence": "nonnegative-integer", "run_instance": "nonempty-string", "root_id": "nonempty-string", "root_kind": "nonempty-string", "source_member": "nonempty-string", "source_branch": "nonempty-string", "owner_ref": "nonempty-string", "root_token": "nonempty-string", "phase_before": "root_phases", "phase_after": "root_phases", "child_id": "nullable-string", "parent_child_id": "nullable-string", "edge_key": "nullable-string", "producer_invocation_id": "nullable-string", "relation": "nullable-CausalRelation", "captured_inputs": "object", "commit_checkpoint": "nonempty-string", "permanent_master_deck_before": "nonnegative-integer", "permanent_master_deck_after": "nonnegative-integer", "mandatory_pending_before": "nonnegative-integer", "mandatory_pending_after": "nonnegative-integer", "root_children_nonterminal_before": "nonnegative-integer", "root_children_nonterminal_after": "nonnegative-integer", "run_terminal_before": "nullable-terminal_kinds", "run_terminal_after": "nullable-terminal_kinds", "terminal_commit": "nullable-TerminalCommit", "operational_stop_before": "operational_stop", "operational_stop_after": "operational_stop", "detached_result_ref": "nullable-string", "result_consume_count": "nonnegative-integer", "reward_delta": "integer", "terminated": "boolean", "truncated": "boolean"}},
    "TerminalCommit": {"required": ["cause", "run_instance", "generation", "source_root_id", "commit_checkpoint", "reward_delta", "terminated", "truncated", "token_consumed", "capture_id"], "types": {"cause": "terminal_kinds", "run_instance": "nonempty-string", "generation": "nonnegative-integer", "source_root_id": "nonempty-string", "commit_checkpoint": "nonempty-string", "reward_delta": "integer", "terminated": "boolean", "truncated": "boolean", "token_consumed": "boolean", "capture_id": "nonempty-string"}},
    "TerminalOverwriteOperand": {"required": ["owner_ref", "root_id", "checkpoint", "target_field", "attempted_value", "terminal_overwrite_effect"], "types": {"owner_ref": "nonempty-string", "root_id": "nonempty-string", "checkpoint": "nonempty-string", "target_field": "OverwriteTarget", "attempted_value": "typed-projection-value", "terminal_overwrite_effect": "OverwriteEffect"}},
    "DeckMutation": {"required": ["root_id", "source_member", "commit_checkpoint", "removed_instance_ids", "added_instance_ids", "added_instance_records", "permanent_master_deck_before", "permanent_master_deck_after"], "types": {"root_id": "nonempty-string", "source_member": "nonempty-string", "commit_checkpoint": "nonempty-string", "removed_instance_ids": "array-card-instance-id", "added_instance_ids": "array-card-instance-id", "added_instance_records": "array-master-deck-instance", "permanent_master_deck_before": "nonnegative-integer", "permanent_master_deck_after": "nonnegative-integer"}},
    "MasterDeckInstance": {"required": ["instance_id", "order", "purgeable", "bottled"], "types": {"instance_id": "nonempty-string", "order": "nonnegative-integer", "purgeable": "boolean", "bottled": "boolean"}},
    "MasterDeckSnapshot": {"required": ["run_instance", "root_id", "owner_ref", "checkpoint", "revision", "instance_records"], "types": {"run_instance": "nonempty-string", "root_id": "nonempty-string", "owner_ref": "nonempty-string", "checkpoint": "nonempty-string", "revision": "nonnegative-integer", "instance_records": "array-master-deck-instance"}},
    "TransmogrifierCandidateResult": {"required": ["run_instance", "root_id", "owner_ref", "checkpoint", "revision", "candidate_instance_ids"], "types": {"run_instance": "nonempty-string", "root_id": "nonempty-string", "owner_ref": "nonempty-string", "checkpoint": "nonempty-string", "revision": "nonnegative-integer", "candidate_instance_ids": "array-card-instance-id"}},
    "ExternalWork": {"required": ["work_id", "producer_invocation_id", "owner_ref", "state", "relation"], "types": {"work_id": "nonempty-string", "producer_invocation_id": "nonempty-string", "owner_ref": "nonempty-string", "state": "ExternalWorkState", "relation": "CausalRelation"}},
    "TerminalOutcomeProjection": {"required": ["cause", "run_instance", "generation", "source_root_id", "commit_checkpoint", "reward_delta", "terminated", "truncated", "capture_id"], "types": {"cause": "terminal_kinds", "run_instance": "nonempty-string", "generation": "nonnegative-integer", "source_root_id": "nonempty-string", "commit_checkpoint": "nonempty-string", "reward_delta": "integer", "terminated": "boolean", "truncated": "boolean", "capture_id": "nonempty-string"}}
  },
  "event_phases": ["InitialChoice", "TransformChild", "CompleteLeave", "Map"],
  "causal_relations": ["ImmediatePayload", "RequiredCallback", "RequiredChild", "TerminalPayoff", "MandatoryChoice", "Unrelated"],
  "external_work_states": ["Pending", "Committed", "Cancelled", "Retired"],
  "overwrite_effects": ["None", "Allow"],
  "projection_fields": ["RunState.terminal", "RunState.run_terminal", "RunState.terminal_commit", "StepInfo.terminal", "FinalDelivery.outcome", "FinalDelivery.reward", "ResetCapture.terminal", "ReplayProjection.end_status", "ReplayProjection.record_terminal"],
  "overwrite_targets": ["RunState.terminal", "RunState.run_terminal", "RunState.terminal_commit", "StepInfo.terminal", "FinalDelivery.outcome", "FinalDelivery.reward", "ResetCapture.terminal", "ReplayProjection.end_status", "ReplayProjection.record_terminal"],
  "projection_value_types": {"RunState.terminal": "TerminalCommit", "RunState.run_terminal": "terminal_kinds", "RunState.terminal_commit": "TerminalCommit", "StepInfo.terminal": "TerminalCommit", "FinalDelivery.outcome": "TerminalOutcomeProjection", "FinalDelivery.reward": "integer", "ResetCapture.terminal": "TerminalCommit", "ReplayProjection.end_status": "terminal_kinds", "ReplayProjection.record_terminal": "TerminalCommit"},
  "projection_owner_by_target": {"RunState.terminal": "RunState", "RunState.run_terminal": "RunState", "RunState.terminal_commit": "RunState", "StepInfo.terminal": "StepInfo", "FinalDelivery.outcome": "FinalDelivery", "FinalDelivery.reward": "FinalDelivery", "ResetCapture.terminal": "ResetCapture", "ReplayProjection.end_status": "ReplayProjection", "ReplayProjection.record_terminal": "ReplayProjection"},
  "projection_equalities": [
    {"source": "TerminalCommit", "target": "RunState.terminal", "fields": ["cause", "run_instance", "generation", "source_root_id", "commit_checkpoint", "reward_delta", "terminated", "truncated", "token_consumed", "capture_id"]},
    {"source": "TerminalCommit.cause", "target": "RunState.run_terminal", "fields": ["cause"]},
    {"source": "TerminalCommit", "target": "RunState.terminal_commit", "fields": ["cause", "run_instance", "generation", "source_root_id", "commit_checkpoint", "reward_delta", "terminated", "truncated", "token_consumed", "capture_id"]},
    {"source": "TerminalCommit", "target": "StepInfo.terminal", "fields": ["cause", "run_instance", "generation", "source_root_id", "commit_checkpoint", "reward_delta", "terminated", "truncated", "token_consumed", "capture_id"]},
    {"source": "TerminalCommit", "target": "FinalDelivery.outcome", "fields": ["cause", "run_instance", "generation", "source_root_id", "commit_checkpoint", "reward_delta", "terminated", "truncated", "capture_id"]},
    {"source": "TerminalCommit.reward_delta", "target": "FinalDelivery.reward", "fields": ["reward_delta"]},
    {"source": "TerminalCommit", "target": "ResetCapture.terminal", "fields": ["cause", "run_instance", "generation", "source_root_id", "commit_checkpoint", "reward_delta", "terminated", "truncated", "token_consumed", "capture_id"]},
    {"source": "TerminalCommit.cause", "target": "ReplayProjection.end_status", "fields": ["cause"]},
    {"source": "TerminalCommit", "target": "ReplayProjection.record_terminal", "fields": ["cause", "run_instance", "generation", "source_root_id", "commit_checkpoint", "reward_delta", "terminated", "truncated", "token_consumed", "capture_id"]}
  ],
  "transitions": {
    "preflight_reject": {"from": "Preflight", "to": "Rejected", "preconditions": ["input_invalid_or_batch_invalid"], "effects": ["no_root_sequence", "no_state_rng_history_reward_or_token_mutation"]},
    "admit_root": {"from": "Preflight", "to": "Active", "preconditions": ["input_valid", "root_id_fresh_in_run_generation", "token_unused_in_generation", "terminal_absent", "owner_current"], "effects": ["create_root", "consume_token_once_on_settlement_not_admission"]},
    "emit_child": {"from": "Active|Settling", "to": "same", "preconditions": ["producer_row_declares_edge", "root_and_parent_match", "parent_invocation_valid", "edge_unique_within_parent_invocation"], "effects": ["append_child_to_same_root_frontier", "record_child_even_when_mandatory_false"]},
    "emit_external_work": {"from": "external producer", "to": "ExternalWork.Pending", "preconditions": ["work is not emitted by this root producer invocation", "external owner explicit"], "effects": ["record external backlog outside root child graph", "root settlement does not drain external work"]},
    "dispatch_child": {"from": "Pending", "to": "Entered", "preconditions": ["root_matches", "parent_matches", "child_pending", "terminal_absent"], "effects": ["dispatch_once"]},
    "commit_child": {"from": "Entered", "to": "Committed", "preconditions": ["root_matches", "commit_checkpoint_matches"], "effects": ["commit_once", "remove_from_mandatory_frontier"]},
    "suspend_result": {"from": "Active|Settling", "to": "Suspended", "preconditions": ["owned_mandatory_choice"], "effects": ["detach_result_once", "preserve_root_cursor"]},
    "consume_result": {"from": "detached", "to": "consumed", "preconditions": ["root_matches", "owner_matches", "consume_count==0"], "effects": ["consume_once", "resume_same_root"]},
    "settle_root": {"from": "Settling", "to": "Settled", "preconditions": ["entered_item_returned", "synchronous_writes_recorded", "mandatory_frontier_empty", "all_children_terminal", "result_consumed_once", "not_faulted", "not_hp_dead"], "effects": ["evaluate_permanent_master_deck_once", "consume_root_token_once"], "writes": {"root.phase": "Settled", "token.consume_count": 1, "terminal_check_field": "permanent_master_deck_count", "child_terminal_states": ["Committed", "Cancelled", "Retired"], "external_work_policy": "leave_pending"}},
    "commit_empty_deck_loss": {"from": "Settled", "to": "terminal", "preconditions": ["prior_terminal_absent", "operational_stop==None", "permanent_master_deck_count==0"], "effects": ["cause=EmptyMasterDeckLoss", "reward_delta=-1", "terminated=true", "truncated=false", "hp_mutation=false", "death_hooks=false", "capture_once", "project_equal_fields"], "writes": {"cause": "EmptyMasterDeckLoss", "reward_delta": -1, "terminated": true, "truncated": false, "hp_mutation": false, "death_hooks": false, "capture_count": 1}},
    "transmog_child_entry": {"from": "InitialChoice", "to": "TransformChild", "preconditions": ["accepted Transform option", "same event root live", "complete master snapshot matches current source owner/root/revision/order/metadata", "candidate result references that exact snapshot"], "effects": ["derive eligible IDs from complete ordered master snapshot by purgeable && !bottled", "require source candidate result exact equality", "owner=TransmogrifierTransformChild"]},
    "transmog_noop": {"from": "TransformChild", "to": "CompleteLeave", "preconditions": ["root_live", "permanent_master_deck_count>0", "eligible_candidate_ids=[]"], "effects": ["result=TransmogrifierEmptyTransformNoOp", "close child only", "detach result to EventRoom consumer", "consume result once", "resume same event root", "reward_delta=0", "no deck/RNG/HP/gold mutation"], "writes": {"result_kind": "TransmogrifierEmptyTransformNoOp", "result_owner": "TransmogrifierTransformChild", "result_consumer": "EventRoom", "child_state": "Retired", "event_phase": "CompleteLeave", "reward_delta": 0, "deck_mutation": false, "rng_advance": false, "hp_gold_mutation": false}},
    "transmog_select": {"from": "TransformChild", "to": "TransformChild", "preconditions": ["root_live", "permanent_master_deck_count>0", "eligible_candidate_ids_nonempty", "selected_count==1", "selected_id_in_snapshot"], "effects": ["remove_selected_then_generate", "emit_required_obtain_child", "settle_same_root"]},
    "pool_fault": {"from": "any", "to": "operational_stop=AdministrativeFault", "preconditions": [], "effects": ["preserve_terminal_commit", "do_not_make_loss"]},
    "overwrite": {"from": "terminal committed", "to": "reject", "preconditions": ["typed owner/root/checkpoint/target/attempted-value/effect operand validates", "target_field in overwrite_targets", "effect in overwrite_effects"], "effects": ["reject terminal targets regardless effect", "allow only named nonterminal projection target", "preserve projection equality"], "writes": {"operand_type": "TerminalOverwriteOperand", "target_enum": "overwrite_targets", "effect_enum": "overwrite_effects", "terminal_targets": ["RunState.terminal", "RunState.run_terminal", "RunState.terminal_commit", "StepInfo.terminal", "FinalDelivery.outcome", "FinalDelivery.reward", "ResetCapture.terminal", "ReplayProjection.end_status", "ReplayProjection.record_terminal"], "allowed_nonterminal_targets": []}},
    "enter_item": {"from": "Active", "to": "entered", "preconditions": ["root token live", "root has no entered item"], "effects": ["mark entered item once"]},
    "return_item": {"from": "entered", "to": "Settling", "preconditions": ["entered body returned"], "effects": ["record synchronous writes and callbacks"]},
    "update_permanent_deck": {"from": "Active|Settling", "to": "same", "preconditions": ["typed DeckMutation owner/root/checkpoint", "removed IDs are current permanent-deck members", "added IDs are fresh instances", "count follows membership equation"], "effects": ["apply committed remove/add instance IDs and metadata once", "increment permanent deck revision", "count=before-|removed|+|added|"], "writes": {"record_type": "DeckMutation", "membership_is_authoritative": true}},
    "hp_death": {"from": "live root", "to": "Interrupted", "preconditions": ["typed actual_hp_after integer <= 0 from committed damage checkpoint", "prior terminal absent"], "effects": ["terminal cause=ActualPlayerDeath", "cancel not-entered mandatory children", "preserve committed prefix"]},
    "explicit_leave": {"from": "CompleteLeave", "to": "Map", "preconditions": ["later explicit event decision", "Transmogrifier no-op child already consumed"], "effects": ["close event once"]},
    "reset_episode": {"from": "terminal delivered", "to": "new run generation", "preconditions": ["explicit reset request", "terminal capture already detached"], "effects": ["retain immutable ResetCapture and replay record", "retire old roots children and tokens", "new run instance required", "clear current episode terminal only after capture"]}
  },
  "transition_guard_vocabulary": ["Transmogrifier no-op child already consumed", "accepted Transform option", "added IDs are fresh instances", "all_children_terminal", "candidate result references that exact snapshot", "child_pending", "commit_checkpoint_matches", "complete master snapshot matches current source owner/root/revision/order/metadata", "consume_count==0", "count follows membership equation", "edge_unique_within_parent_invocation", "effect in overwrite_effects", "eligible_candidate_ids=[]", "eligible_candidate_ids_nonempty", "entered body returned", "entered_item_returned", "explicit reset request", "external owner explicit", "input_invalid_or_batch_invalid", "input_valid", "later explicit event decision", "mandatory_frontier_empty", "not_faulted", "not_hp_dead", "operational_stop==None", "owned_mandatory_choice", "owner_current", "owner_matches", "parent_invocation_valid", "parent_matches", "permanent_master_deck_count==0", "permanent_master_deck_count>0", "prior terminal absent", "prior_terminal_absent", "producer_row_declares_edge", "removed IDs are current permanent-deck members", "result_consumed_once", "root has no entered item", "root token live", "root_and_parent_match", "root_id_fresh_in_run_generation", "root_live", "root_matches", "same event root live", "selected_count==1", "selected_id_in_snapshot", "synchronous_writes_recorded", "target_field in overwrite_targets", "terminal capture already detached", "terminal_absent", "token_unused_in_generation", "typed DeckMutation owner/root/checkpoint", "typed actual_hp_after integer <= 0 from committed damage checkpoint", "typed owner/root/checkpoint/target/attempted-value/effect operand validates", "work is not emitted by this root producer invocation", "candidate snapshot is complete and source-bound"],
  "transition_effect_vocabulary": ["allow only named nonterminal projection target", "append_child_to_same_root_frontier", "apply committed remove/add instance IDs and metadata once", "cancel not-entered mandatory children", "capture_once", "cause=EmptyMasterDeckLoss", "clear current episode terminal only after capture", "close child only", "close event once", "commit_once", "consume result once", "consume_once", "consume_root_token_once", "consume_token_once_on_settlement_not_admission", "count=before-|removed|+|added|", "create_root", "death_hooks=false", "derive eligible IDs from complete ordered master snapshot by purgeable && !bottled", "detach result to EventRoom consumer", "detach_result_once", "dispatch_once", "do_not_make_loss", "emit_required_obtain_child", "evaluate_permanent_master_deck_once", "hp_mutation=false", "increment permanent deck revision", "mark entered item once", "new run instance required", "no deck/RNG/HP/gold mutation", "no_root_sequence", "no_state_rng_history_reward_or_token_mutation", "owner=TransmogrifierTransformChild", "preserve committed prefix", "preserve projection equality", "preserve_root_cursor", "preserve_terminal_commit", "project_equal_fields", "record external backlog outside root child graph", "record synchronous writes and callbacks", "record_child_even_when_mandatory_false", "reject terminal targets regardless effect", "remove_from_mandatory_frontier", "remove_selected_then_generate", "require source candidate result exact equality", "result=TransmogrifierEmptyTransformNoOp", "resume same event root", "resume_same_root", "retain immutable ResetCapture and replay record", "retire old roots children and tokens", "reward_delta=-1", "reward_delta=0", "root settlement does not drain external work", "settle_same_root", "terminal cause=ActualPlayerDeath", "terminated=true", "truncated=false", "record external work outside same-root child graph"],
  "transmogrifier": {"source_option": "Transform", "source_member": "event.transmogrifier", "source_checkpoint": "TransformChildEntry", "parent_owner": "EventRoom", "child_owner": "TransmogrifierTransformChild", "result_owner": "TransmogrifierTransformChild", "master_snapshot_owner": "RunState.master_deck", "candidate_result_owner": "TransmogrifierTransformChild", "candidate_source": "complete live permanent master deck instance records at child entry", "candidate_predicate": "purgeable && !bottled", "candidate_order": "visible deck order", "ordinary_selection_count": 1, "source_grid_flags": {"count": 1, "forUpgrade": false, "forTransform": true, "canCancel": false, "forPurge": false}, "empty_nonempty_result": "TransmogrifierEmptyTransformNoOp", "no_op_parent_checkpoint": "CompleteLeave", "leave_dispatch": "separate explicit later decision", "no_empty_deck_fallback": true},
  "trace_order": "strictly increasing sequence per run; root/child ownership fields must match transition state"
}
```

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

`PlayOrigin` enum: `Hand`, `HavocTop`, `MayhemTop`, `DistilledChaosTop`, `RepeatPower`, `OmniscienceOriginal`, `OmniscienceCopy`, `GeneratedOffer`, `PotionQueuedTop`, `ReplaySource`, `OtherExplicitSource`. `PaymentRuleId` enum: `OrdinaryTurnCostAfterUse`, `OneShotFree`, `AutoplayNoOrdinaryDebit`, `XActionLiveDebit`, `CorruptionSkillExempt`, `NoUseNoDebit`. The complete bounded dispatch matrix is §3.3B; `OtherExplicitSource` is admitted only by an additional explicit row with all fields below, otherwise it faults before dispatch.

The ten X action families have exact separate gates: Whirlwind/Skewer/Body positive hit/block count; Malaise/Doppelganger/Tempest/Collect upgraded count+1; Multi-Cast requires current orb before effect **or payment**; Transmutation raises capture to at least live energy and generates at adjusted count; Conjure Blade constructs Expunger even at zero count and nonfree payment is outside the positive-count test. Retain §4.1.2's details for effect payloads; no single X rule overrides them.

### 3.3 Callback/effect-site manifest for native joins

Stable names below are semantic aliases for the existing native.34 source-site registry; native owner binds them to its assigned namespace/owner/operation/variant codes and exact catalog rows. This table is an exact required join set. Its member-family aliases do not infer effect edges or payment; the finite producer rows below do that work.

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

#### 3.3A Finite producer → effect → commit edges (S unless marked F)

`Enqueue` creates pending work; `enter` is that work's own selected state transition; `commit` is the named mutation boundary. A child enters the causal frontier only where this table says so. Effects numbered within a row have that stable semantic tie order; different rows/containers have no invented global priority. `INTERRUPT` means committed player death or administrative fault, not a merely pending terminal candidate. `CHOICE` means suspend under §2.3.

Each emitted child is represented as `{edge_key, producer_member, producer_branch, parent_invocation, captured_inputs, effect_kind, relation, source_container, enqueue_operation, emitted_index, entered_guard, commit_checkpoint, interrupt_rule, tie_key, terminal_scope}`. `captured_inputs` contains only values actually captured at that source call (including target/owner/type/count where applicable); `commit_checkpoint` names the actual state write or list movement. Synchronous writes/callbacks are recorded in the parent trace but do not masquerade as queued children. `edge_key` is unique within its producer invocation and is not a source ordinal, wire ID, or callback-count placeholder.

| Producer / source variant | Ordered emitted effects and cause ownership | Enqueue → enter → commit | Terminal edge / tie rule |
|---|---|---|---|
| Admitted card use, source-identified card | 1 direct `ImmediatePayload` operations in the card's listed use order; 2 synchronous `RequiredCallback`s in the caller's recorded order; 3 only actions required to finish that payload become `RequiredChild`; 4 qualifying Fatal/growth is `TerminalPayoff`. Other emissions are `Unrelated`, not causal by queue proximity. | `use` enqueues; each action enters separately; state commits in that action/callback. After-use/destination is a separate `UseCardAction` producer below. | Finish entered use and synchronous tail; then listed descendants; `INTERRUPT` stops. Preserve source list order, otherwise no cross-container comparison. |
| `DamageAction` / player-owned hit | 1 target damage pipeline §4; 2 target's first qualifying death callback; 3 exact attributed Fatal effect; 4 split-child insertion/setup only if this death site emits it; 5 selected Reaper heal only if its final-kill predicate is true. | Damage request enqueues; its action enters; HP/death commits there; Fatal/growth commits in its qualifying callback/action; child insertion and setup are separate; Reaper uses ordinary heal at selected final-kill checkpoint. | `INTERRUPT` stops pending descendants. Otherwise settle qualifying payoff/children before victory. No aggregate duplicate reward. |
| `ApplyPowerAction`, new-power path | 1 source `onApplyPower`; 2 validity/prevention/Artifact; 3 stack-or-append; 4 priority sort only on new action-path append; 5 initial/modify callbacks in source order. Callback effects become children only when separately named in this table. | Enqueue action; enter callback/guard at update; stack/append commits before sort; callbacks emit their own pending effects. | `INTERRUPT`; no terminal payoff merely from power application. Live callback-list order per §3.1. |
| Actual draw | 1 call moved card's `triggerWhenDrawn` while it remains draw-top; 2 commit draw-top→hand movement; 3 power notifications; 4 relic notifications. Only explicit callback outputs are children; movement/generation alone is not a draw. | Enter draw method/action; membership commits at movement; notifications run synchronously in listed order; their requests enqueue. | `INTERRUPT` after entered synchronous call returns; no callback inferred from transfer into draw. |
| Actual exhaust | 1 relic callbacks; 2 power callbacks; 3 card callback; 4 pile move. | Enter exhaust movement; synchronous callbacks in order; actual pile membership commits after callbacks. Each emitted request remains pending until selected. | `INTERRUPT` after call return; Dead Branch samples at its relic callback, not later. |
| Manual discard | 1 move/refresh; 2 card manual-discard callback; 3 discard counter/history notification. | Enter discard action; membership commits at move; callback requests enqueue; counter commits at final listed checkpoint. | `INTERRUPT` after synchronous call; end-turn discard is a different producer and does not inherit manual-discard edges. |
| Start-turn Magnetism | For each intensity in source callback order: generate one L34 card, place via its named creation route. | Pre-normal-draw hook enqueues each generation; generation/placement commits at its own action. | Explicit terminal interruption; intensity order retained. It does not imply all generated cards enter hand before normal draw. |
| Start-turn Mayhem | Pre-normal-draw callback enqueues one deferred wrapper/intensity; each wrapper's later entry enqueues one draw-top autoplay; each card then follows its own origin row. | Hook → wrapper enqueue; wrapper enters separately; top-card action enqueues; card admission/use/cleanup enter separately. | Explicit terminal interruption. No assertion that all nested plays precede normal draw; preserve wrapper producer order. |
| Havoc top-card | 1 move current draw-top instance through limbo; 2 enqueue Havoc-origin play; 3 admitted card payload; 4 requested exhaust-once destination. | Capture energy/target at the top-card source checkpoint; card later enters common manager gate; exhaust commits only on actual exhaust movement. | Failed gate has no successful play payload/history/payment; cleanup still obeys producer flags. `INTERRUPT` stops pending child. |
| Mayhem top-card | 1 move current draw-top instance through limbo; 2 enqueue Mayhem-origin play; 3 admitted card payload; no Havoc-imposed exhaust. | Deferred wrapper and top-card action enter separately; target/capture at top-card queue construction; card later enters manager gate. | Failed gate is not successful play; cleanup uses actual card flags. No draw callback from movement. |
| Distilled Chaos, each 3–6 action instances | 1 capture each target proposal during potion use, before any child resolves; 2 enqueue each separate top-card action; 3 each child uses its own admission/payload/destination path. | Target proposals/actions are created during potion use; action records enter separately; card manager validates at its own entry. | Preserve producer insertion order when selected from that container; earlier resolution does not redraw later proposal. `INTERRUPT` stops not-entered children. |
| Double Tap / Burst / Amplify / Duplication / Echo repeat | One child per actual source request; each keeps its captured energy and same-instance linkage; each child's play/payload/cleanup is separate. | Repeat requests enqueue from producer; manager admission and use are separate; purge-on-use commits after callbacks. | Preserve insertion order; actual turn-ending/death branch interrupts not-entered repeats. No energy debit a second time. |
| Omniscience | 1 selected original `CHOICE`; 2 mark original exhaust/remove; 3 enqueue original; 4 construct stat-equivalent purge copy; 5 enqueue copy; 6 original play; 7 copy play. Requests 3–5 exist before either child resolves. | Selection/result commits; each NewQueue captures energy at that action's execution; both queue requests are autoplay=true/ignore-total=false; manager derives card ignore-on-use only after successful canUse. | Original request before copy; death stops not-entered child. Failure of original does not undo/remove copy or alter its flags. |
| Potion-queued play other than Distilled Chaos | Specific potion row owns selected/moved card; its admitted child emits that card's direct payload and cleanup. No potion-wide canUse bypass. | Potion consume/selection commits at parent checkpoint; child queue enters later; captures/target are only those stated in that potion row. | Parent is not re-entered on child resume; death retains prior potion commit and interrupts pending children. |
| Generated offer / created card | Candidate construction is not play and emits no play-history/callback edge. A selected candidate emits only its named obtain/placement child; a later admitted play is a new producer. | Generation commits when sampled; choice suspends; obtain/placement commits in its own effect; play history only at manager admission. | Skip semantics belong to offer row. No inferred card cost payment at generation/selection. |
| Source-owned selector/result consumer | `CHOICE` then only the row's stated result mutations/children; queued effect is not result-consumer completion. | Commit result at exact row checkpoint; each child enters separately; parent resumes only when consumer reports completion/outcome. | `INTERRUPT` preserves committed prefix but fabricates no uncommitted parent result; row order is tie order. |
| Combat-victory candidate / handoff | Candidate → local named post-combat filter → current-item required descendants → combat result/reward parent → eligible reward/mandatory children → successor initialization. | Candidate is not room success; reward take/skip commits at its row; Act2Ready only after successor init commits. | PlayerDeath/fault outrank victory; no unrelated queue drain or extra enemy turn; +1 only at Act2Ready. |

#### 3.3A.1 Selected deterministic settlement and tie keys (S)

These rules define the native semantic linearization; they do not claim incidental source-frame timing. They apply only to effects with a concrete producer row above or below.

1. Complete the currently entered method, including its synchronous writes, callback calls, and source-local loop, before entering any action it emitted. A synchronous callback's direct writes are part of that call; a separately queued action from it is a child record.
2. For each emitted action, retain its concrete container and insertion operation from the producer. `addToTop` uses index-zero stack semantics (later insertion precedes earlier); `addToBottom` appends; card queue uses its established index-one/empty insertion rule. These rules order siblings only within the producer-owned causal set; they do not compare unrelated containers.
3. When the accepted row gives multiple mandatory children, settle the listed source emission order after the current call returns; finish each child and its own required descendants before the next sibling. Source-local multi-hit/target loops complete within the entered method as described by their exact member row, before any emitted child enters.
4. Callback tie order is the actual named receiver-family/list iteration order in §3.1 and the member row. Actor-list/selected-list order is used only where the accepted source row states it. No sort, snapshot, source ordinal, pointer order, or global FIFO is invented.
5. If two required effects have no ordering in their concrete row or the source-sensitive ordering tables, the producer binding is incomplete and fails closed; do not choose a plausible order. Unrelated backlog never enters the frontier by adjacency or priority. Actual player death after synchronous rescue interrupts all not-yet-entered gameplay children. Room victory waits for active immediate work and its required frontier; `Act2Ready` waits for the boss handoff. Escape remains room-scoped.

The trace's deterministic `tie_key` is `(parent_invocation, producer_member, producer_branch, callback_receiver_order?, callback_list_index?, loop_or_target_index?, emitted_index, enqueue_operation, insertion_index?)`. Optional components are absent only when the concrete producer row establishes they are inapplicable; a required but unavailable component is an unresolved join, not a default value.

#### 3.3A.2 Concrete card/effect producer expansions (F member mechanics; S settlement)

These rows bind selected ordinary producers to their accepted public mechanical definitions. “Causal child” below means an explicitly constructed pending action; synchronous mutations/callbacks complete within the entered producer and are not separately scheduled. Any listed child is still subject to its own entry guard. This is not permission to expand an unlisted card through a generic `use` handler.

| Exact producer member/branch | Captured state and emitted records in source order | Commit/interrupt and terminal classification | Accepted mechanical definition |
|---|---|---|---|
| `Bane.use` → first `DamageAction`, then `BaneAction` | Capture selected target and two separate player-owned `DamageInfo` payloads at card use. Emit first ordinary hit, then dedicated Bane follow-up with its own captured target/payload. | First hit and its synchronous recipient callbacks commit before second action can enter. Follow-up exits on null target; requires current Poison; initial-duration branch additionally checks positive target HP and, for non-THORNS payload, source not dying; at completion deliver only to positive-HP target. Actual player death interrupts not-entered child. A second hit is not a second generic `DamageAction`. | E21 Silent §7, Bane; consolidated §7. |
| `PoisonPower.atStartOfTurn` → `PoisonLoseHpAction` | Capture Poison amount into HP_LOSS request; later decrement the currently present Poison only at action completion. | In Combat and not basically-dead, loss action enters; target-positive-HP gate, recipient HP_LOSS pipeline, then current-Poison decrement/removal and local cleanup. Player death is run terminal and prevents subsequent causal heal; monster death/split remains a distinct monster transition. | E21 Silent §7, Poison; consolidated §7. |
| `CatalystAction` | At action entry sample current Poison `P`; emit one application carrying raw stack amount `P` and a constructed Poison amount `P` (the normal accepted path yields `2P→3P` only if same power survives and application is accepted). No Poison means no child. | Sampling commits no Poison write; the application enters separately and commits through ApplyPowerAction guards. Artifact, reactions, replacement, or absent source power prevent treating `3P` as unconditional assignment. | E21 Silent §7, Catalyst. |
| `BouncingFlask.use` / bounce action | Initial target uses `cardRandomRng` and null-exception/alive-only candidates excluding halfDead/isDying/isEscaping; repeats are allowed. Each non-final bounce captures its next target before the current Poison application resolves. | Empty candidates yield null; singleton still requests bounded index. Each captured target must remain positive HP when application enters; null/stale target and basically-dead branches stop, never resample. Keep current application/wait and next-bounce insertion in the source's exact top order. | E21 Silent §7, Bouncing Flask. |
| `PiercingWail.use` | Snapshot monster-list membership and Artifact-at-use restoration eligibility. First pass emits one −M Strength application per monster; second emits +M Shackled restoration only for use-time non-Artifact members. Both lists are constructed before first application resolves. | Every application separately enters target/Artifact/stack guards. Consuming Artifact in first pass cannot alter the second-pass snapshot. All-Artifact and empty-list branches emit zero restoration (and zero per-monster applications for the empty list). | E21 Silent §3 and §7, PiercingWail. |
| `CorpseExplosion.use` / `CorpseExplosionPower.onDeath` | Use requests Poison, then a separate Corpse Explosion installation. On death, only owner HP≤0 and not-basically-dead emits area THORNS damage, null owner, payload `owner.maxHealth-at-callback × current explosion amount`. | Guard-false branch emits no damage child. Payload is not sampled at card use and does not use remaining HP. Each target is a separate recipient damage entry; no Attack/Fatal attribution is inferred. | E21 Silent §5 and §7, Corpse Explosion. |
| `FiendFireAction.update` | Capture `n=remaining hand size` at this action's update, not card use. Emit `n` individual random one-card exhaust requests from the shrinking original hand and `n` distinct `D(7/10)` hit edges. Do not collapse the hits into one `n×D` packet. | Exhaust reactions do not recursively add cards to the captured exhaust set. Each actual exhaust runs relic→power→card hooks before pile move. Each hit uses the common targeted-damage template and its own entered/commit/interruption outcome; hit and exhaust edges remain separate siblings in the concrete action insertion order. | E24 R70 and §5, multi-exhaust. |
| `SecondWind.use` | Snapshot non-Attacks at use; emit front-inserted single-card exhaust actions followed by distinct Block events according to this source sequence. | Exhaust callback commits synchronously at each movement; Block events enter only after preceding top-inserted exhaust work under the source action-container semantics. Do not fold into one Block or infer every non-Attack is Skill. | E24 red/colorless §5, multi-exhaust. |
| `SeverSoul.use` | Select eligible non-Attack hand cards and front-insert exhaustion requests ahead of its hit request. | Each actual exhaust invokes hooks and moves the card; hit is a distinct later action. Source selection order is the accepted member's order, not universal hand order. Death interrupts pending hit. | E24 red/colorless §5, multi-exhaust. |
| `CalculatedGamble.use` | Capture `n` at its action entry. For `n>0`, emit random discard(n) then draw(n); for `n=0`, emit neither child (not `DrawCardAction(0)`). | Discard reactions complete before the draw child enters. Draw history is reset only by an actually entered draw action; the no-child zero branch does not clear it. Reaction-drawn cards do not enlarge captured `n`. | E21 Silent §5, source-specific discard/draw transactions. |
| `StormOfSteel.use` / `BladeFuryAction` | Capture `n`; prepare one template before discard execution. Base uses ordinary Shiv. Upgraded branch fresh-constructs Shiv, copies it, and upgrades the copy. Construct `MakeTempCardInHandAction(template,n)` at this checkpoint; its constructor applies Master Reality to eligible template before `DiscardAction(n)` is constructed/prepended. | Discard executes first, but the prepared template and constructor-time Master Reality checkpoint precede it. At creation-action entry make `n` stat-equivalent copies; admit available hand capacity, send overflow to discard (not deletion or manual-discard callback). `n=0` has no card-placement child, while the prepared-template/constructor checkpoint still occurs. | E21 Silent §8, Storm of Steel and `MakeTempCardInHandAction`. |
| `Unload.use` specific discard | Capture target card; at action entry require it still belongs to hand; move it, update discard count/notifications, then invoke that card's manual-discard hook. | This callback order differs from selected/random DiscardAction. Each emitted callback child belongs to this exact specific-discard event; no mutation if target no longer in hand. | E21 Silent §5, source-specific discard and draw transactions. |
| `DrawCardAction.update` + `drawnCards` follow-up | Initial update clears shared draw history even on designated zero-count/No Draw/empty/full-hand exits. A real draw calls the card's draw hook while draw-top, commits its movement, records it before power/relic draw callbacks; follow-up scans only that actual shared history. | Draw membership and callback commit in actual source order; a follow-up is a child only when its producer explicitly supplied one. Escape Plan's Skill test sees shared history; no synthetic zero-count draw may be introduced because it would clear history. | E21 Silent §5; E19 §5. |
| `EndTurnDeathPower` / end-turn card callbacks | Keep marker dispatch separate from later `DiscardAtEndOfTurnAction`: the latter operates on its shuffled remaining-hand clone and visits `triggerOnEndOfPlayerTurn`; Entrench's callback top-inserts `ExhaustAllEtherealAction`. | Entrench's inserted action is a causal child of that callback, not evidence that Entrench is Ethereal. Top insertion order is index-zero stack behavior; callbacks are not recursively run inside the current callback. No claim all children resolve before the next source hook. | E24 red/colorless §5; E19 §5; TX18 remains unexecuted. |
| `VampireDamageAllEnemiesAction` (Reaper) | Retain its member-specific target set and per-target damage; aggregate actual committed `lastDamageTaken`, not requested packet. Heal request is based on that sum. | Area hit loops over source target order; selected policy schedules one ordinary heal after active hit/callback/mandatory causal chain if a qualifying final-enemy kill occurred and player is not actually dead. Actual death interrupts it; this selected ordering does not assert source-timer/frame outcome. | E28 §§5–6; E24 red/colorless §5, Fatal/heal; consolidated §1.2. |

The table intentionally names only branches for which the accepted mechanical record gives the captures, emitted effects, and local guards above. It does **not** close every admitted card/relic/power/enemy callback producer. The unresolved member set is a real EX-I1 residual, not covered by a family alias or callback count; see §9 and the continuation-v2 report.

#### 3.3A.3 E21 Silent finite primary-member crosswalk (F member facts; S causal binding)

This unit covers the 76 accepted E21 primary rows only: four basics, nineteen commons, thirty-three uncommons, nineteen rares, and dependent Shiv. The stable member identity is the accepted E21 source ID in the first cell, not a generated canonical key. Each locator points to the complete paired source row; payload text is copied whole, without semicolon splitting. Use the admitted-card-use template in §3.3A.2 only for ordinary card admission; the payload column identifies the exact effect-template sequence/deltas, and the branch column points to distinct accepted callback, constructor, delayed, discard, choice, or special hit behavior. This section does not close global callback/death/history/RNG rules left open by the source record.

| E21 source ID / display | Accepted primary row | Exact member payload / edge-template delta | Additional distinct branch and accepted fact anchor |
|---|---:|---|---|
| `Strike_G` (Strike) | E21 Silent:63 |One damage hit. **Templates:** `HIT`.| Ordinary admitted use emits the exact member payload shown; child/action callbacks enter through their own common source template and commit/guard rows in §§3.1–3.3A.1 and E21 §§6–8. No extra member-specific branch is asserted beyond the accepted row. |
| `Defend_G` (Defend) | E21 Silent:64 |One player Block gain. **Templates:** `BLOCK`.| Ordinary admitted use emits the exact member payload shown; child/action callbacks enter through their own common source template and commit/guard rows in §§3.1–3.3A.1 and E21 §§6–8. No extra member-specific branch is asserted beyond the accepted row. |
| `Neutralize` | E21 Silent:65 |Damage, then M Weak. **Templates:** `HIT + APPLY_POWER`.| Ordinary admitted use emits the exact member payload shown; child/action callbacks enter through their own common source template and commit/guard rows in §§3.1–3.3A.1 and E21 §§6–8. No extra member-specific branch is asserted beyond the accepted row. |
| `Survivor` | E21 Silent:66 |Block, then selected discard of 1 remaining hand card. **Templates:** `BLOCK + DISCARD_CHOICE`.| Ordinary admitted use emits the exact member payload shown; child/action callbacks enter through their own common source template and commit/guard rows in §§3.1–3.3A.1 and E21 §§6–8. No extra member-specific branch is asserted beyond the accepted row. |
| `Acrobatics` | E21 Silent:75 |Request M draws, then selected discard 1. **Templates:** `DRAW + DISCARD_CHOICE`.| Ordinary admitted use emits the exact member payload shown; child/action callbacks enter through their own common source template and commit/guard rows in §§3.1–3.3A.1 and E21 §§6–8. No extra member-specific branch is asserted beyond the accepted row. |
| `Backflip` | E21 Silent:76 |Block, then draw 2. **Templates:** `BLOCK + DRAW`.| Ordinary admitted use emits the exact member payload shown; child/action callbacks enter through their own common source template and commit/guard rows in §§3.1–3.3A.1 and E21 §§6–8. No extra member-specific branch is asserted beyond the accepted row. |
| `Bane` | E21 Silent:77 |At card use capture the target and both DamageInfo payloads; enqueue the first hit then a dedicated BaneAction. Its Poison, target-HP and initial-duration source-dying gates are specified in §7, not a generic second DamageAction. **Templates:** `CAPTURED_HIT + BANE_FOLLOWUP`.| Captured target/two DamageInfo payloads; first ordinary hit then dedicated BaneAction with null/Poison/initial-duration/source-dying/completion-target gates. E21 §7, lines 278–299. |
| `Blade Dance` | E21 Silent:78 |Create M ordinary Shivs, not Shiv+ by the card upgrade. **Templates:** `TEMP_CREATE`.| Ordinary admitted use emits the exact member payload shown; child/action callbacks enter through their own common source template and commit/guard rows in §§3.1–3.3A.1 and E21 §§6–8. No extra member-specific branch is asserted beyond the accepted row. |
| `Cloak And Dagger` (Cloak and Dagger) | E21 Silent:79 |Block, then create M ordinary Shivs. **Templates:** `BLOCK + TEMP_CREATE`.| Ordinary admitted use emits the exact member payload shown; child/action callbacks enter through their own common source template and commit/guard rows in §§3.1–3.3A.1 and E21 §§6–8. No extra member-specific branch is asserted beyond the accepted row. |
| `Dagger Spray` | E21 Silent:80 |Two distinct all-enemy damage requests using the card's calculated matrix. **Templates:** `AREA_HIT ×2`.| Ordinary admitted use emits the exact member payload shown; child/action callbacks enter through their own common source template and commit/guard rows in §§3.1–3.3A.1 and E21 §§6–8. No extra member-specific branch is asserted beyond the accepted row. |
| `Dagger Throw` | E21 Silent:81 |Damage, draw 1, selected discard 1. **Templates:** `HIT + DRAW + DISCARD_CHOICE`.| Ordinary admitted use emits the exact member payload shown; child/action callbacks enter through their own common source template and commit/guard rows in §§3.1–3.3A.1 and E21 §§6–8. No extra member-specific branch is asserted beyond the accepted row. |
| `Deadly Poison` | E21 Silent:82 |Apply M Poison. **Templates:** `APPLY_POWER`.| Ordinary admitted use emits the exact member payload shown; child/action callbacks enter through their own common source template and commit/guard rows in §§3.1–3.3A.1 and E21 §§6–8. No extra member-specific branch is asserted beyond the accepted row. |
| `Deflect` | E21 Silent:83 |Block. **Templates:** `BLOCK`.| Ordinary admitted use emits the exact member payload shown; child/action callbacks enter through their own common source template and commit/guard rows in §§3.1–3.3A.1 and E21 §§6–8. No extra member-specific branch is asserted beyond the accepted row. |
| `Dodge and Roll` | E21 Silent:84 |Block now; install Next Turn Block with the same calculated Block captured at use. **Templates:** `BLOCK + DELAYED_BLOCK`.| Capture calculated card Block at use and install Next Turn Block; delayed delivery uses stored amount. E21 §7 delayed-state table. |
| `Flying Knee` | E21 Silent:85 |Damage; install Energized 1. **Templates:** `HIT + APPLY_POWER`.| Ordinary admitted use emits the exact member payload shown; child/action callbacks enter through their own common source template and commit/guard rows in §§3.1–3.3A.1 and E21 §§6–8. No extra member-specific branch is asserted beyond the accepted row. |
| `Outmaneuver` | E21 Silent:86 |Install Energized 2→3; these amounts are literals, not M. **Templates:** `APPLY_POWER`.| Ordinary admitted use emits the exact member payload shown; child/action callbacks enter through their own common source template and commit/guard rows in §§3.1–3.3A.1 and E21 §§6–8. No extra member-specific branch is asserted beyond the accepted row. |
| `PiercingWail` (Piercing Wail) | E21 Silent:87 |Request −M Strength for each monster; separately request Shackled restoration M only for monsters without Artifact at use-time. See §7. **Templates:** `TWO_PASS_APPLY_POWER`.| Two monster-list passes; restoration eligibility snapshots Artifact at card use; each application has independent resolution-time gates. E21 §7, lines 351–359. |
| `Poisoned Stab` | E21 Silent:88 |Damage, then M Poison. **Templates:** `HIT + APPLY_POWER`.| Ordinary admitted use emits the exact member payload shown; child/action callbacks enter through their own common source template and commit/guard rows in §§3.1–3.3A.1 and E21 §§6–8. No extra member-specific branch is asserted beyond the accepted row. |
| `Prepared` | E21 Silent:89 |Draw M, then selected discard M. **Templates:** `DRAW + DISCARD_CHOICE`.| Ordinary admitted use emits the exact member payload shown; child/action callbacks enter through their own common source template and commit/guard rows in §§3.1–3.3A.1 and E21 §§6–8. No extra member-specific branch is asserted beyond the accepted row. |
| `Quick Slash` | E21 Silent:90 |Damage, then draw 1. **Templates:** `HIT + DRAW`.| Ordinary admitted use emits the exact member payload shown; child/action callbacks enter through their own common source template and commit/guard rows in §§3.1–3.3A.1 and E21 §§6–8. No extra member-specific branch is asserted beyond the accepted row. |
| `Slice` | E21 Silent:91 |One damage hit. **Templates:** `HIT`.| Ordinary admitted use emits the exact member payload shown; child/action callbacks enter through their own common source template and commit/guard rows in §§3.1–3.3A.1 and E21 §§6–8. No extra member-specific branch is asserted beyond the accepted row. |
| `Underhanded Strike` (Sneaky Strike) | E21 Silent:92 |Damage; at the later conditional-energy action, gain 2 if totalDiscardedThisTurn>0. This is not cost cancellation/refund. **Templates:** `HIT + CONDITIONAL_ENERGY`.| Ordinary admitted use emits the exact member payload shown; child/action callbacks enter through their own common source template and commit/guard rows in §§3.1–3.3A.1 and E21 §§6–8. No extra member-specific branch is asserted beyond the accepted row. |
| `Sucker Punch` | E21 Silent:93 |Damage, then M Weak. **Templates:** `HIT + APPLY_POWER`.| Ordinary admitted use emits the exact member payload shown; child/action callbacks enter through their own common source template and commit/guard rows in §§3.1–3.3A.1 and E21 §§6–8. No extra member-specific branch is asserted beyond the accepted row. |
| `Accuracy` | E21 Silent:101 |Install Accuracy M; Shiv identity-specific rewriting in §8. **Templates:** `INSTALL_POWER + IDENTITY_CALLBACK`.| Identity-specific Shiv constructor, stack, and pile-rewrite callbacks; not a generic Attack modifier. E21 §8, lines 442–447. |
| `All Out Attack` (All-Out Attack) | E21 Silent:102 |All-enemy damage, then random discard 1. **Templates:** `AREA_HIT + RANDOM_DISCARD`.| Ordinary admitted use emits the exact member payload shown; child/action callbacks enter through their own common source template and commit/guard rows in §§3.1–3.3A.1 and E21 §§6–8. No extra member-specific branch is asserted beyond the accepted row. |
| `Backstab` | E21 Silent:103 |One damage hit. **Templates:** `HIT`.| Ordinary admitted use emits the exact member payload shown; child/action callbacks enter through their own common source template and commit/guard rows in §§3.1–3.3A.1 and E21 §§6–8. No extra member-specific branch is asserted beyond the accepted row. |
| `Blur` | E21 Silent:104 |Block; install Blur 1, preserving Block at the relevant next start-turn removal checkpoint. **Templates:** `BLOCK + RETAIN_POWER`.| Retains Block at the relevant next start-turn removal checkpoint; delayed power is separate from immediate Block. E21 §7 delayed-state table. |
| `Bouncing Flask` | E21 Silent:105 |M separate Poison-3 applications to source-selected random enemies, with repeats allowed; §7 gives sampling checkpoints. **Templates:** `CAPTURED_RANDOM_TARGET + REPEATED_APPLY_POWER`.| Initial target and next-target sampling checkpoints, allowed repeats, null/stale/positive-HP/empty/singleton branches. E21 §7, lines 331–342. |
| `Calculated Gamble` | E21 Silent:106 |Snapshot remaining hand size n at its action; for n>0, random-discard n then request n draws; n=0 enqueues neither discard nor draw. Both versions call the same non-extra-draw action branch. **Templates:** `CAPTURE_COUNT + RANDOM_DISCARD + CONDITIONAL_DRAW`.| Capture n; n>0 random discard then draw; n=0 has no child and does not trigger DrawCardAction history reset. E21 §6, lines 233–239, 247–265. |
| `Caltrops` | E21 Silent:107 |Install Thorns M; retaliate only on qualified incoming damage callbacks (§7), including fully blocked qualifying attacks. **Templates:** `INSTALL_POWER + DAMAGE_CALLBACK`.| Install Thorns M; callback requires incoming type not THORNS/HP_LOSS, nonnull attacker distinct from owner; fully blocked qualifying attack still retaliates; prepend THORNS damage. E21 §7, lines 369–376. |
| `Catalyst` | E21 Silent:108 |If Poison exists at action resolution, request an additional current amount→twice current amount as Poison. No Poison means no application; not direct power multiplication. **Templates:** `CONDITIONAL_CAPTURED_APPLY_POWER`.| Ordinary admitted use emits the exact member payload shown; child/action callbacks enter through their own common source template and commit/guard rows in §§3.1–3.3A.1 and E21 §§6–8. No extra member-specific branch is asserted beyond the accepted row. |
| `Choke` | E21 Silent:109 |Damage; separately install Choked M. Its on-use hook requests M HP loss; it removes at afflicted start-turn. **Templates:** `HIT + INSTALL_POWER + HP_LOSS_CALLBACK`.| Install Choked M separately; onUseCard requests HP loss; afflicted start-turn removes it. E21 §5 row; §7, lines 361–367. |
| `Concentrate` | E21 Silent:110 |Selected discard M (all available when undersized), then gain 2 energy even if fewer were available. No extra minimum-hand legality override. **Templates:** `DISCARD_CHOICE + ENERGY`.| Ordinary admitted use emits the exact member payload shown; child/action callbacks enter through their own common source template and commit/guard rows in §§3.1–3.3A.1 and E21 §§6–8. No extra member-specific branch is asserted beyond the accepted row. |
| `Crippling Poison` (Crippling Cloud) | E21 Silent:111 |Unless monsters are basically dead, visit non-dead/non-dying monsters and enqueue M Poison then 2 Weak per monster. Separate applications. **Templates:** `GUARDED_MULTI_TARGET_APPLY_POWER`.| Ordinary admitted use emits the exact member payload shown; child/action callbacks enter through their own common source template and commit/guard rows in §§3.1–3.3A.1 and E21 §§6–8. No extra member-specific branch is asserted beyond the accepted row. |
| `Dash` | E21 Silent:112 |Block first, then damage. It remains an Attack for type hooks. **Templates:** `BLOCK + HIT`.| Ordinary admitted use emits the exact member payload shown; child/action callbacks enter through their own common source template and commit/guard rows in §§3.1–3.3A.1 and E21 §§6–8. No extra member-specific branch is asserted beyond the accepted row. |
| `Distraction` | E21 Silent:113 |Fresh generated Skill from actual-character source pools; set its eligible turn cost to 0, then temporary hand creation. §8 defines filtering, not a green pool. **Templates:** `GENERATE_CHOICE + TEMP_CREATE`.| Generated actual-character Skill selector, eligible turn-cost setter, then temporary creation; Master Reality is sampled at creation-action construction. E21 §8, lines 402–412 and 435–440. |
| `Endless Agony` | E21 Silent:114 |Damage on use. Actual draw separately creates one stat-equivalent temporary copy at queue top; creation is not a draw. **Templates:** `HIT + DRAW_CALLBACK_TEMP_CREATE`.| Damage on use plus separate draw-triggered stat-equivalent temporary-copy creation; creation is not a draw. E21 §5 row and §8, lines 458–460. |
| `Escape Plan` | E21 Silent:115 |Draw 1 with a follow-up that scans DrawCardAction.drawnCards for any Skill; if found, enqueue one captured-B Block gain, then stop scanning. **Templates:** `DRAW + HISTORY_FOLLOWUP`.| Draw follow-up scans actual shared drawnCards history; zero/no-draw branch cannot synthesize a draw or Block. Nested history is explicitly unresolved in source record. E21 §6, lines 247–257. |
| `Eviscerate` | E21 Silent:116 |Three distinct hits. Each received discard notification lowers eligible turn cost by 1; draw resets it from combat cost minus turn discard counter. Exact hooks in §6. **Templates:** `HIT ×3 + COST_CALLBACK`.| Three distinct hits; per-instance discard notification lowers eligible turn cost; on-draw setter uses combat cost minus discard history; copy-side source mutation is specified. E21 §6, lines 240–246. |
| `Expertise` | E21 Silent:117 |At action resolution snapshot hand size h; enqueue one draw of M−h only when h<M; otherwise enqueue none, not a zero-count draw or a loop maintaining M after reactions. **Templates:** `CAPTURED_DEFICIT_DRAW`.| Action captures h; one draw child only if h<M; otherwise no child, not zero-count history-clearing draw. E21 §6, lines 258–266. |
| `Finisher` | E21 Silent:118 |At follow-up resolution count ATTACK entries in cardsPlayedThisTurn, subtract 1, and enqueue that many hits if target exists with positive HP. Ordinary current Finisher is excluded by the subtraction. **Templates:** `CAPTURED_HISTORY_MULTI_HIT`.| Hit count captured at follow-up from cardsPlayedThisTurn ATTACK entries minus current Finisher; target must be present and positive HP. E21 §6, lines 258–266. |
| `Flechettes` | E21 Silent:119 |At its action, enqueue one hit per current hand card whose type is SKILL, including unplayable Skills; not per playable card. **Templates:** `CAPTURED_HAND_MULTI_HIT`.| Action-time hand scan; one hit per SKILL including unplayable Skills, not per playable card. E21 §6, lines 258–266. |
| `Footwork` | E21 Silent:120 |Apply M Dexterity. **Templates:** `APPLY_POWER`.| Ordinary admitted use emits the exact member payload shown; child/action callbacks enter through their own common source template and commit/guard rows in §§3.1–3.3A.1 and E21 §§6–8. No extra member-specific branch is asserted beyond the accepted row. |
| `Heel Hook` | E21 Silent:121 |Always prepend damage; if target is non-null and already has Weak at helper entry, also enqueue gain 1 energy then draw 1 behind that damage. Weak only gates the energy/draw; no positive-HP predicate or post-hit Weak retest is added. **Templates:** `HIT + CONDITIONAL_ENERGY_DRAW`.| Ordinary admitted use emits the exact member payload shown; child/action callbacks enter through their own common source template and commit/guard rows in §§3.1–3.3A.1 and E21 §§6–8. No extra member-specific branch is asserted beyond the accepted row. |
| `Infinite Blades` | E21 Silent:122 |Install Infinite Blades 1; at start-turn create one ordinary Shiv per power amount unless monsters are basically dead. Upgrade does not increase amount. **Templates:** `INSTALL_POWER + START_TURN_CREATE`.| Start-turn creation uses one ordinary Shiv template per power amount unless monsters basically dead. E21 §7 delayed-state table. |
| `Leg Sweep` | E21 Silent:123 |Apply M Weak, then player Block. **Templates:** `APPLY_POWER + BLOCK`.| Ordinary admitted use emits the exact member payload shown; child/action callbacks enter through their own common source template and commit/guard rows in §§3.1–3.3A.1 and E21 §§6–8. No extra member-specific branch is asserted beyond the accepted row. |
| `Masterful Stab` | E21 Silent:124 |Damage. Each qualifying per-instance tookDamage notification calls combat-cost update +1, not a global combat-loss-count formula (§6). **Templates:** `HIT + DAMAGE_CALLBACK_COST`.| Per-instance tookDamage notification updates combat cost; callback location and no catch-up for exhausted instances are specified. E21 §6, lines 198–213. |
| `Noxious Fumes` | E21 Silent:125 |Install M; at player post-draw turn-start request amount Poison separately for each non-dead/non-dying monster, unless monsters are basically dead. **Templates:** `INSTALL_POWER + POST_DRAW_APPLY_POWER`.| Player post-draw start-turn producer; separate Poison application per eligible monster, with basically-dead and per-target guards. E21 §7, lines 319–320 and delayed-state table. |
| `Predator` | E21 Silent:126 |Damage; install Draw Card Next Turn 2. **Templates:** `HIT + DELAYED_DRAW`.| Ordinary admitted use emits the exact member payload shown; child/action callbacks enter through their own common source template and commit/guard rows in §§3.1–3.3A.1 and E21 §§6–8. No extra member-specific branch is asserted beyond the accepted row. |
| `Reflex` | E21 Silent:127 |Ordinary canUse always false. Manual-discard callback appends draw M; direct use body also requests draw M if another origin reaches it, not an ordinary legality grant. **Templates:** `MANUAL_DISCARD_DRAW_CALLBACK`.| Ordinary canUse false; manual-discard callback appends draw M; direct use body is distinct and not an ordinary legality grant. E21 §6, lines 228–232; E21 row. |
| `Riddle With Holes` (Riddle with Holes) | E21 Silent:128 |Five distinct damage hits. **Templates:** `HIT ×5`.| Ordinary admitted use emits the exact member payload shown; child/action callbacks enter through their own common source template and commit/guard rows in §§3.1–3.3A.1 and E21 §§6–8. No extra member-specific branch is asserted beyond the accepted row. |
| `Setup` | E21 Silent:129 |Choose one remaining hand instance, move toward draw-top; set one-play-free only when its mutable combat cost>0. Never a universal numeric-zero rewrite (§6). **Templates:** `CARD_CHOICE_MOVE + CONDITIONAL_FREE_FLAG`.| Choice/movement branch: empty no-op, singleton automatic, otherwise select one; free flag only when mutable combat cost>0; sentinel cases explicit. E21 §6, lines 165–188. |
| `Skewer` | E21 Silent:130 |One hit per positive effective X; upgrade changes per-hit damage, not hit count (§8). **Templates:** `CAPTURED_X_HIT_LOOP`.| X wrapper has member-specific positive-effect gate, captured/resolved energy, Chemical X/upgraded delta and payment rule; distinct payload is in exact member row and E21 §8, lines 483–502. |
| `Tactician` | E21 Silent:131 |Ordinary canUse always false; manual-discard callback prepends gain M energy. Its use body is empty. **Templates:** `MANUAL_DISCARD_ENERGY_CALLBACK`.| Ordinary canUse false; manual-discard callback prepends gain M energy; use body empty. E21 §6, lines 228–232; E21 row. |
| `Terror` | E21 Silent:132 |Apply 99 Vulnerable duration, not an infinite-duration administrative sentinel. **Templates:** `APPLY_POWER`.| Apply 99 Vulnerable duration; duration is not infinite sentinel. Application still uses the independent ApplyPower guards in E21 §7, lines 301–311. |
| `Well Laid Plans` (Well-Laid Plans) | E21 Silent:133 |Install Retain Cards M; optional end-turn selection up to amount, with Ethereal/Pyramid/Equilibrium distinctions (§6). **Templates:** `INSTALL_POWER + END_TURN_CHOICE`.| Optional end-turn selector only with nonempty hand and without Pyramid/Equilibrium; Ethereal selected cards do not receive temporary retain. E21 §6, lines 268–274. |
| `A Thousand Cuts` | E21 Silent:142 |Install Thousand Cuts M; after-card-played callback requests amount all-enemy THORNS damage, not another Attack play. **Templates:** `INSTALL_POWER + AFTER_PLAY_AREA_THORNS`.| onAfterCardPlayed callback requests all-enemy THORNS damage; distinct from Attack play. E21 §5 row. |
| `Adrenaline` | E21 Silent:143 |Gain 1→2 energy, then draw 2. **Templates:** `ENERGY + DRAW`.| Ordinary admitted use emits the exact member payload shown; child/action callbacks enter through their own common source template and commit/guard rows in §§3.1–3.3A.1 and E21 §§6–8. No extra member-specific branch is asserted beyond the accepted row. |
| `After Image` | E21 Silent:144 |Install After Image 1; on-use callback requests Block equal to power amount. Upgrade changes Innate, not Block. **Templates:** `INSTALL_POWER + ON_USE_BLOCK_CALLBACK`.| onUseCard callback requests Block equal to installed power amount; newly installed power does not receive earlier parent callbacks. E21 §5 row; §7, lines 361–367. |
| `Venomology` (Alchemize) | E21 Silent:145 |Sample a limited-route potion at use, enqueue obtain; actual Ironclad potion eligibility, Sozu and belt resolution are distinct (§8). **Templates:** `CAPTURE_POTION + OBTAIN`.| Potion sampled at use before obtain; actual-character limited pool, Sozu at obtain execution, belt mutation at resolution. E21 §8, lines 414–422. |
| `Bullet Time` | E21 Silent:146 |Enqueue No Draw then eligible current-hand turn-cost zeroing. Does not set Setup's free flag or rewrite X/unplayable sentinels. **Templates:** `NO_DRAW + TURN_COST_MUTATION`.| Ordinary admitted use emits the exact member payload shown; child/action callbacks enter through their own common source template and commit/guard rows in §§3.1–3.3A.1 and E21 §§6–8. No extra member-specific branch is asserted beyond the accepted row. |
| `Burst` | E21 Silent:147 |Install Burst M; next M eligible non-purge Skill uses each enqueue one additional play, not M extra plays of a single Skill. **Templates:** `INSTALL_POWER + REPEAT_PLAY_CALLBACK`.| Next M eligible non-purge Skill plays each request one same-instance repeat; explicit autoplay and ignoreEnergyTotal flags, purge prevents recursive charge. E21 §8, lines 475–481. |
| `Corpse Explosion` | E21 Silent:148 |Apply M Poison, then separately CorpseExplosionPower 1. On qualifying death request owner's then-max-HP × explosion stacks as all-enemy THORNS (§7). **Templates:** `APPLY_POWER + DEATH_CALLBACK_AREA_THORNS`.| Poison application and separate power installation; onDeath guard and maxHP-at-callback × stacks all-enemy THORNS payload with null owner. E21 §7, lines 344–349. |
| `Die Die Die` | E21 Silent:149 |One all-enemy damage request. **Templates:** `AREA_HIT`.| Ordinary admitted use emits the exact member payload shown; child/action callbacks enter through their own common source template and commit/guard rows in §§3.1–3.3A.1 and E21 §§6–8. No extra member-specific branch is asserted beyond the accepted row. |
| `Doppelganger` | E21 Silent:150 |Effective X gains a +1 upgraded bonus; if positive, install that much Energized then Draw Card Next Turn (§8). **Templates:** `CAPTURED_X + DELAYED_POWER`.| X wrapper has member-specific positive-effect gate, captured/resolved energy, Chemical X/upgraded delta and payment rule; distinct payload is in exact member row and E21 §8, lines 483–502. |
| `Envenom` | E21 Silent:151 |Install Envenom 1; onAttack with positive damageAmount, NORMAL damage and target other than owner prepends Poison equal to power amount. **Templates:** `INSTALL_POWER + ATTACK_CALLBACK_APPLY_POWER`.| onAttack callback requires positive damageAmount, NORMAL type, and target != owner; prepends Poison. E21 §7, lines 374–376. |
| `Glass Knife` | E21 Silent:152 |Two hits, then UUID-scoped baseDamage−2 floored at zero across matching battle instances (§8); not permanent master-deck loss. **Templates:** `PREBUILT_HIT ×2 + UUID_MUTATION`.| Two prebuilt DamageInfo hits, followed by UUID-scoped baseDamage−2 (zero floor) across matching battle instances; mutation is not a queued ordinary hit. E21 §8, lines 467–473. |
| `Grand Finale` | E21 Silent:153 |Ordinary canUse also requires draw pile size=0; payload is one all-enemy hit. Discard pile need not be empty. Other origins' checks remain distinct. **Templates:** `GUARDED_AREA_HIT`.| Ordinary canUse includes draw-pile-empty predicate; discard-pile state is not part of that guard; other origins remain distinct. E21 §5 row. |
| `Malaise` | E21 Silent:154 |For positive effective X (+1 upgraded), request −X Strength then X Weak; no restoration power (§8). **Templates:** `CAPTURED_X + TWO_APPLY_POWER`.| X wrapper has member-specific positive-effect gate, captured/resolved energy, Chemical X/upgraded delta and payment rule; distinct payload is in exact member row and E21 §8, lines 483–502. |
| `Night Terror` (Nightmare) | E21 Silent:155 |Choose one remaining hand card; capture/reset a stat-equivalent template when constructing its independent delayed power; next start-turn request M temporary copies. **Templates:** `CARD_CHOICE + CAPTURED_TEMPLATE + DELAYED_TEMP_CREATE`.| Selected card captured/reset in power constructor; next start-turn creates M temporary copies from independent template. E21 §8, lines 458–465. |
| `Phantasmal Killer` | E21 Silent:156 |Install Phantasmal 1; next start-turn install Double Damage with initial duration 1 and reduce Phantasmal by 1. NORMAL damage ×2, not ×2 per stack. **Templates:** `INSTALL_POWER + START_TURN_POWER_CALLBACK`.| Next start-turn applies Double Damage with initial duration 1 then reduces Phantasmal; multiplier is NORMAL damage ×2 once. E21 §7 delayed-state table. |
| `Storm of Steel` | E21 Silent:157 |Snapshot remaining hand size n; prepare one Shiv→Shiv+ template and construct its temporary-creation action before prepending discard n. Discard executes first; insertion later makes n stat-equivalent copies from that prepared template (§8). Upgrade changes template state, not n. **Templates:** `CAPTURE_COUNT + CONSTRUCT_TEMPLATE + TEMP_CREATE + DISCARD`.| Template constructor and Master Reality checkpoint occur before discard execution; base/upgraded template construction differ; later copies use prepared template and overflow goes to discard. E21 §8, lines 424–440. |
| `Tools of the Trade` | E21 Silent:158 |Install Tools 1; post-draw start-turn request amount draws, then selected discard amount. Stacks increase both, not the number of independent selections. **Templates:** `INSTALL_POWER + POST_DRAW_DRAW_DISCARD`.| Post-draw start-turn emits amount draws then selected discard; stacks change amounts, not selection count. E21 §7 delayed-state table. |
| `Unload` | E21 Silent:159 |Damage; snapshot all current non-Attack hand identities and prepend specific discards in iteration order (therefore reverse pending order). Recheck membership at each discard. **Templates:** `HIT + SPECIFIC_DISCARD_BATCH`.| Ordinary admitted use emits the exact member payload shown; child/action callbacks enter through their own common source template and commit/guard rows in §§3.1–3.3A.1 and E21 §§6–8. No extra member-specific branch is asserted beyond the accepted row. |
| `Wraith Form v2` (Wraith Form) | E21 Silent:160 |Apply IntangiblePlayer M, then independently Wraith Form −1; downside loses 1 Dexterity per power amount magnitude at end-turn (§7). **Templates:** `APPLY_POWER ×2 + END_TURN_CALLBACK`.| Install IntangiblePlayer M then independent Wraith Form −1; end-turn Dexterity-loss callback is a distinct debit/power branch. E21 §7 delayed-state table. |
| `Shiv` | E21 Silent:161 |One damage hit; constructor includes current Accuracy amount in baseDamage. See §8 for existing-instance rewrites and generated upgrades. **Templates:** `CONSTRUCTOR_MUTATED_HIT`.| Ordinary admitted use emits the exact member payload shown; child/action callbacks enter through their own common source template and commit/guard rows in §§3.1–3.3A.1 and E21 §§6–8. No extra member-specific branch is asserted beyond the accepted row. |

Per-row trace binding uses the exact member alias and source-row locator above. For each listed child preserve its payload/count/target capture checkpoint, source insertion operation, child-entry guard, state commit, interruption prefix and terminal scope from the cited source rule; direct synchronous power/card callbacks are not converted into queued children. Common primitives may be reused only where the complete payload is that primitive: one calculated targeted hit, one all-enemy matrix request, Block gain, draw request, discard transaction, power application, creation/choice, or a named callback. Multi-hit, X, constructor/copy/mutation, delayed/power callback, and conditional branches retain their explicit deltas above and are not aliases for one primitive. The accepted E21 material does not define every cross-system recipient continuation: CARD-DEATH, CARD-HISTORY, CARD-ORDER, CARD-RNG and CARD-COPY residuals in E21 §10 remain open; this crosswalk does not invent their outcomes.

#### 3.3A.4 Six E21 Silent card request graphs (F source edges; S immediate-causal settlement)

This bounded graph covers Dash, Die Die Die, Leg Sweep, Poisoned Stab, Quick Slash and Slice, base and upgraded. The structured JSON below is the **sole normative six-member semantic contract**: it records source payload/captures, typed ordered edges, guards, commits, interruption/terminal disposition and expected causal cases. The private expected record is separately source/policy anchored and compared field-for-field to the delivered JSON. The following table is generated deterministically from that contract and is informative only; it is not a second manually authored semantic authority. Its JSON-valued semantic cells are parsed and checked against the deterministic projection, so whitespace/reflow is irrelevant and changed capture/order/guard/commit/case data is rejected without an English-phrase parser.

`F` identifies accepted E21 source edges and shared primitive facts; `S` identifies deterministic immediate-causal settlement. Each card payload edge remains a separate requested child and does not imply a second play dispatch. PLAY cleanup/exhaust is owned only by the separate §3.3B lifecycle.

**Poisoned Stab source/native ordering and shared life-state binding.** E21 Silent:88 emits `DamageAction(target,D)` followed unconditionally by `ApplyPowerAction(target, Poison, M)`; no nonlethal/positive-HP predicate is part of that source payload. In retained source output, `ApplyPowerAction.update()` first exits for null target or `target.isDeadOrEscaped()`, then runs source `onApplyPower` hooks and its Champion Belt branch, rechecks the monster with that same helper, and only then proceeds to target-side application/prevention/Artifact/stack-or-append behavior. The helper's actual body is `isDying || halfDead || (!isPlayer && AbstractMonster.isEscaping)`: it does not read HP, `isDead`, or the monster `escaped` flag. Thus HP≤0 alone is not this guard; `isDying`, `halfDead`, and monster `isEscaping` are.

**Source order:** the hit enters `AbstractMonster.damage`; HP is committed, and HP≤0 synchronously calls `die()`. `die()` sets `isDying` before its synchronous power death hooks and enabled player-relic monster-death hooks return. `isDead` is a later `updateDeathAnimation`/death-timer checkpoint. **Selected native causal order:** after the hit's synchronous `damage`/`die` hook prefix returns, enter the already-emitted Poison child before a victory candidate, without advancing the death timer. Therefore a lethal hit reaches the child with `isDying=true`; the real shared helper returns true and `ApplyPowerAction` produces no Poison mutation. This is helper-derived source behavior, not a new HP-positive policy. For a nonlethal target not otherwise rejected by the helper, the child enters the ordinary shared power pipeline; callbacks/prevention/Artifact still decide whether Poison commits. Do not retarget. Actual `PlayerDeath` before child entry interrupts it. Unrelated queue work and independently queued callback work remain under the shared scheduler and are not drained or claimed by this local producer contract.

**Normative structured contract:**

<!-- six-member-semantic-contracts-v7 -->
```json
{
  "schema": "six-member-semantic-contracts-v3",
  "policy": "JSON is the sole normative six-member semantic contract; the table is a deterministic informative projection; local causal policy only, not source-frame parity",
  "cards": [
    {
      "id": "Dash",
      "source": "2026-10-19-act1-content-silent.md:112",
      "branches": [
        {
          "version": "base",
          "captured": {
            "target": "selected monster at admitted use",
            "damage": {
              "type": "NORMAL",
              "amount": 10,
              "owner": "player"
            },
            "block": {
              "type": "Block",
              "amount": 10,
              "owner": "player"
            }
          },
          "edges": [
            {
              "key": "dash.block",
              "kind": "BlockGain",
              "receiver": "player",
              "value_type": "Block",
              "value": 10,
              "order": 1,
              "guard": "shared Block primitive entry guard",
              "commit": "player Block mutation in Block primitive"
            },
            {
              "key": "dash.hit",
              "kind": "DamageAction",
              "receiver": "captured target",
              "value_type": "DamageInfo NORMAL",
              "value": 10,
              "order": 2,
              "guard": "shared monster-damage entry guard; never retarget",
              "commit": "target HP/death in monster damage primitive"
            }
          ],
          "interrupt": "player death during an entered edge stops every later edge; block remains committed if hit callbacks cause death",
          "terminal": "settle hit death callbacks before victory candidate"
        },
        {
          "version": "upgraded",
          "captured": {
            "target": "selected monster at admitted use",
            "damage": {
              "type": "NORMAL",
              "amount": 13,
              "owner": "player"
            },
            "block": {
              "type": "Block",
              "amount": 13,
              "owner": "player"
            }
          },
          "edges": [
            {
              "key": "dash.block",
              "kind": "BlockGain",
              "receiver": "player",
              "value_type": "Block",
              "value": 13,
              "order": 1,
              "guard": "shared Block primitive entry guard",
              "commit": "player Block mutation in Block primitive"
            },
            {
              "key": "dash.hit",
              "kind": "DamageAction",
              "receiver": "captured target",
              "value_type": "DamageInfo NORMAL",
              "value": 13,
              "order": 2,
              "guard": "shared monster-damage entry guard; never retarget",
              "commit": "target HP/death in monster damage primitive"
            }
          ],
          "interrupt": "player death during an entered edge stops every later edge; block remains committed if hit callbacks cause death",
          "terminal": "settle hit death callbacks before victory candidate"
        }
      ],
      "cases": {
        "normal": {
          "precondition": "selected target survives and no callback kills player",
          "ordered_results": [
            {
              "edge": "dash.block",
              "state": "entered",
              "result": "committed Block"
            },
            {
              "edge": "dash.hit",
              "state": "entered",
              "result": "committed target damage"
            }
          ],
          "resume": "parent continues after both edges",
          "terminal": "no victory unless hit kills final target"
        },
        "final_enemy": {
          "precondition": "dash.hit lethally damages the final target",
          "ordered_results": [
            {
              "edge": "dash.block",
              "state": "entered",
              "result": "committed Block"
            },
            {
              "edge": "dash.hit",
              "state": "entered",
              "result": "committed target damage and death callbacks"
            }
          ],
          "resume": "causal chain settles before victory",
          "terminal": "victory candidate may commit only after hit callbacks"
        },
        "player_death": {
          "precondition": "hit callback commits actual player death",
          "ordered_results": [
            {
              "edge": "dash.block",
              "state": "entered",
              "result": "committed Block"
            },
            {
              "edge": "dash.hit",
              "state": "entered",
              "result": "committed hit prefix; death interrupts remaining descendants"
            }
          ],
          "resume": "no gameplay resume after PlayerDeath",
          "terminal": "PlayerDeath"
        }
      },
      "evidence_kind": "accepted E21 source member plus shared primitive contract; selected settlement policy separately labeled"
    },
    {
      "id": "Die Die Die",
      "source": "2026-10-19-act1-content-silent.md:149",
      "branches": [
        {
          "version": "base",
          "captured": {
            "target_set": "source all-enemy traversal at use",
            "damage_matrix": {
              "type": "NORMAL",
              "amount": 13,
              "owner": "player",
              "per_target": true
            }
          },
          "edges": [
            {
              "key": "ddd.area",
              "kind": "DamageAllEnemiesAction",
              "receiver": "captured target set",
              "value_type": "per-target DamageInfo NORMAL matrix",
              "value": 13,
              "order": 1,
              "guard": "shared area/monster-damage primitive guards per recipient",
              "commit": "independent recipient HP/death commits in source traversal order"
            }
          ],
          "interrupt": "player death in an entered recipient chain stops not-entered later producer children; no second card payload edge",
          "terminal": "finish the entered area loop and its required death callbacks before victory"
        },
        {
          "version": "upgraded",
          "captured": {
            "target_set": "source all-enemy traversal at use",
            "damage_matrix": {
              "type": "NORMAL",
              "amount": 17,
              "owner": "player",
              "per_target": true
            }
          },
          "edges": [
            {
              "key": "ddd.area",
              "kind": "DamageAllEnemiesAction",
              "receiver": "captured target set",
              "value_type": "per-target DamageInfo NORMAL matrix",
              "value": 17,
              "order": 1,
              "guard": "shared area/monster-damage primitive guards per recipient",
              "commit": "independent recipient HP/death commits in source traversal order"
            }
          ],
          "interrupt": "player death in an entered recipient chain stops not-entered later producer children; no second card payload edge",
          "terminal": "finish the entered area loop and its required death callbacks before victory"
        }
      ],
      "cases": {
        "normal": {
          "precondition": "player survives all recipient callbacks",
          "ordered_results": [
            {
              "edge": "ddd.area",
              "state": "entered",
              "result": "each captured recipient receives its committed hit"
            }
          ],
          "resume": "parent continues after area loop",
          "terminal": "none unless captured recipients die"
        },
        "final_enemy": {
          "precondition": "one captured recipient is the final lethal target",
          "ordered_results": [
            {
              "edge": "ddd.area",
              "state": "entered",
              "result": "source-order recipient hit/death prefix completes"
            }
          ],
          "resume": "area loop and mandatory death callbacks settle before victory",
          "terminal": "victory candidate after entered area causal work"
        },
        "player_death": {
          "precondition": "recipient callback commits actual player death during area loop",
          "ordered_results": [
            {
              "edge": "ddd.area",
              "state": "entered",
              "result": "committed recipient prefix; PlayerDeath interrupts later gameplay descendants"
            }
          ],
          "resume": "no gameplay resume after PlayerDeath",
          "terminal": "PlayerDeath; no Exhaust payload edge"
        }
      },
      "play_cleanup": "E=true; separate admitted PLAY UseCardAction exhaust/destination lifecycle, not a second payload edge",
      "evidence_kind": "accepted E21 source member plus shared primitive contract; selected settlement policy separately labeled"
    },
    {
      "id": "Leg Sweep",
      "source": "2026-10-19-act1-content-silent.md:123",
      "branches": [
        {
          "version": "base",
          "captured": {
            "target": "selected monster at admitted use",
            "weak": {
              "type": "Weak",
              "amount": 2
            },
            "block": {
              "type": "Block",
              "amount": 11,
              "owner": "player"
            }
          },
          "edges": [
            {
              "key": "leg.weak",
              "kind": "ApplyPowerAction",
              "receiver": "captured target",
              "value_type": "Weak debuff",
              "value": 2,
              "order": 1,
              "guard": "non-null/live target and independent debuff/Artifact/prevention guard",
              "commit": "accepted Weak stack/append in power primitive"
            },
            {
              "key": "leg.block",
              "kind": "BlockGain",
              "receiver": "player",
              "value_type": "Block",
              "value": 11,
              "order": 2,
              "guard": "shared Block primitive entry guard",
              "commit": "player Block mutation"
            }
          ],
          "interrupt": "player death during Weak application/callback prevents later Block; committed Weak remains",
          "terminal": "settle Weak application/callback then Block before victory candidate"
        },
        {
          "version": "upgraded",
          "captured": {
            "target": "selected monster at admitted use",
            "weak": {
              "type": "Weak",
              "amount": 3
            },
            "block": {
              "type": "Block",
              "amount": 14,
              "owner": "player"
            }
          },
          "edges": [
            {
              "key": "leg.weak",
              "kind": "ApplyPowerAction",
              "receiver": "captured target",
              "value_type": "Weak debuff",
              "value": 3,
              "order": 1,
              "guard": "non-null/live target and independent debuff/Artifact/prevention guard",
              "commit": "accepted Weak stack/append in power primitive"
            },
            {
              "key": "leg.block",
              "kind": "BlockGain",
              "receiver": "player",
              "value_type": "Block",
              "value": 14,
              "order": 2,
              "guard": "shared Block primitive entry guard",
              "commit": "player Block mutation"
            }
          ],
          "interrupt": "player death during Weak application/callback prevents later Block; committed Weak remains",
          "terminal": "settle Weak application/callback then Block before victory candidate"
        }
      ],
      "cases": {
        "normal": {
          "precondition": "target accepts Weak; player survives callbacks",
          "ordered_results": [
            {
              "edge": "leg.weak",
              "state": "entered",
              "result": "Weak application commits"
            },
            {
              "edge": "leg.block",
              "state": "entered",
              "result": "player Block commits"
            }
          ],
          "resume": "parent continues after both edges",
          "terminal": "none unless callback creates victory"
        },
        "final_enemy": {
          "precondition": "Weak callbacks or prior state make this the final enemy resolution",
          "ordered_results": [
            {
              "edge": "leg.weak",
              "state": "entered",
              "result": "accepted Weak commit or typed power no-effect"
            },
            {
              "edge": "leg.block",
              "state": "entered",
              "result": "Block commits"
            }
          ],
          "resume": "both causal edges settle before victory",
          "terminal": "victory candidate after Block"
        },
        "player_death": {
          "precondition": "Weak application callback commits actual player death",
          "ordered_results": [
            {
              "edge": "leg.weak",
              "state": "entered",
              "result": "committed power prefix; callback death"
            },
            {
              "edge": "leg.block",
              "state": "not_entered",
              "result": "interrupted; no Block mutation"
            }
          ],
          "resume": "no gameplay resume after PlayerDeath",
          "terminal": "PlayerDeath"
        }
      },
      "evidence_kind": "accepted E21 source member plus shared primitive contract; selected settlement policy separately labeled"
    },
    {
      "id": "Poisoned Stab",
      "source": "2026-10-19-act1-content-silent.md:88; §7 ApplyPowerAction rules",
      "branches": [
        {
          "version": "base",
          "captured": {
            "target": "selected monster at admitted use",
            "damage": {
              "type": "NORMAL",
              "amount": 6,
              "owner": "player"
            },
            "poison": {
              "type": "Poison",
              "amount": 3
            }
          },
          "edges": [
            {
              "key": "ps.hit",
              "kind": "DamageAction",
              "receiver": "captured target",
              "value_type": "DamageInfo NORMAL",
              "value": 6,
              "order": 1,
              "guard": "shared monster-damage entry guard",
              "commit": "target HP/death in damage primitive"
            },
            {
              "key": "ps.poison",
              "kind": "ApplyPowerAction",
              "receiver": "captured target",
              "value_type": "Poison debuff",
              "value": 3,
              "order": 2,
              "guard": "ApplyPowerAction.update target null/helper guard; after source hooks, recheck monster helper before application; no direct HP guard",
              "commit": "accepted Poison stack/append only; no mutation on no-effect or prevention"
            }
          ],
          "interrupt": "actual player death before Poison child entry interrupts it; no post-death Poison",
          "terminal": "settle Poison child using actual shared helper after synchronous hit/death hooks and before victory candidate; no HP-only policy"
        },
        {
          "version": "upgraded",
          "captured": {
            "target": "selected monster at admitted use",
            "damage": {
              "type": "NORMAL",
              "amount": 8,
              "owner": "player"
            },
            "poison": {
              "type": "Poison",
              "amount": 4
            }
          },
          "edges": [
            {
              "key": "ps.hit",
              "kind": "DamageAction",
              "receiver": "captured target",
              "value_type": "DamageInfo NORMAL",
              "value": 8,
              "order": 1,
              "guard": "shared monster-damage entry guard",
              "commit": "target HP/death in damage primitive"
            },
            {
              "key": "ps.poison",
              "kind": "ApplyPowerAction",
              "receiver": "captured target",
              "value_type": "Poison debuff",
              "value": 4,
              "order": 2,
              "guard": "ApplyPowerAction.update target null/helper guard; after source hooks, recheck monster helper before application; no direct HP guard",
              "commit": "accepted Poison stack/append only; no mutation on no-effect or prevention"
            }
          ],
          "interrupt": "actual player death before Poison child entry interrupts it; no post-death Poison",
          "terminal": "settle Poison child using actual shared helper after synchronous hit/death hooks and before victory candidate; no HP-only policy"
        }
      ],
      "cases": {
        "normal": {
          "precondition": "captured target is not dying, half-dead, or escaping at child entry; player survives",
          "ordered_results": [
            {
              "edge": "ps.hit",
              "state": "entered",
              "result": "damage commits; nonlethal target remains outside shared dead-or-escaped helper"
            },
            {
              "edge": "ps.poison",
              "state": "entered",
              "result": "ApplyPower source hooks/target recheck/prevention/Artifact run; Poison commits only if accepted"
            }
          ],
          "resume": "parent continues after entered shared power result",
          "terminal": "none unless hit killed target"
        },
        "final_enemy": {
          "precondition": "lethal hit synchronously sets isDying and completes death hooks; player survives",
          "ordered_results": [
            {
              "edge": "ps.hit",
              "state": "entered",
              "result": "HP/death prefix commits; die sets isDying before synchronous death hooks return"
            },
            {
              "edge": "ps.poison",
              "state": "entered",
              "result": "shared isDeadOrEscaped helper is true from isDying; ApplyPower exits with no Poison mutation"
            }
          ],
          "resume": "no retarget; required child no-effect settles before victory",
          "terminal": "victory candidate only after Poison child helper-guard result"
        },
        "player_death": {
          "precondition": "hit callback commits actual player death before Poison child entry",
          "ordered_results": [
            {
              "edge": "ps.hit",
              "state": "entered",
              "result": "committed hit/death prefix"
            },
            {
              "edge": "ps.poison",
              "state": "not_entered",
              "result": "interrupted by PlayerDeath; no mutation"
            }
          ],
          "resume": "no gameplay resume after PlayerDeath",
          "terminal": "PlayerDeath"
        }
      },
      "evidence_kind": "accepted E21 source member plus shared primitive contract; selected settlement policy separately labeled",
      "source_policy_note": "Unconditional E21 DamageAction then ApplyPowerAction is source payload. Poison entry uses the actual shared ApplyPower helper and per-application pipeline; no direct positive-HP guard."
    },
    {
      "id": "Quick Slash",
      "source": "2026-10-19-act1-content-silent.md:90; shared draw transaction §3.3A.1",
      "branches": [
        {
          "version": "base",
          "captured": {
            "target": "selected monster at admitted use",
            "damage": {
              "type": "NORMAL",
              "amount": 8,
              "owner": "player"
            },
            "draw_count": 1
          },
          "edges": [
            {
              "key": "qs.hit",
              "kind": "DamageAction",
              "receiver": "captured target",
              "value_type": "DamageInfo NORMAL",
              "value": 8,
              "order": 1,
              "guard": "shared monster-damage entry guard",
              "commit": "target HP/death in damage primitive"
            },
            {
              "key": "qs.draw",
              "kind": "DrawCardAction",
              "receiver": "player draw/hand",
              "value_type": "draw count",
              "value": 1,
              "order": 2,
              "guard": "on entry apply No Draw, hand-capacity, and pile-availability draw guards",
              "commit": "card becomes hand member at actual draw movement; no-draw exit commits no card movement"
            }
          ],
          "interrupt": "actual player death before draw entry interrupts draw; no card after death",
          "terminal": "draw child or its explicit no-effect result settles after lethal hit and before victory candidate"
        },
        {
          "version": "upgraded",
          "captured": {
            "target": "selected monster at admitted use",
            "damage": {
              "type": "NORMAL",
              "amount": 12,
              "owner": "player"
            },
            "draw_count": 1
          },
          "edges": [
            {
              "key": "qs.hit",
              "kind": "DamageAction",
              "receiver": "captured target",
              "value_type": "DamageInfo NORMAL",
              "value": 12,
              "order": 1,
              "guard": "shared monster-damage entry guard",
              "commit": "target HP/death in damage primitive"
            },
            {
              "key": "qs.draw",
              "kind": "DrawCardAction",
              "receiver": "player draw/hand",
              "value_type": "draw count",
              "value": 1,
              "order": 2,
              "guard": "on entry apply No Draw, hand-capacity, and pile-availability draw guards",
              "commit": "card becomes hand member at actual draw movement; no-draw exit commits no card movement"
            }
          ],
          "interrupt": "actual player death before draw entry interrupts draw; no card after death",
          "terminal": "draw child or its explicit no-effect result settles after lethal hit and before victory candidate"
        }
      ],
      "cases": {
        "normal": {
          "precondition": "player survives; draw primitive permits one card",
          "ordered_results": [
            {
              "edge": "qs.hit",
              "state": "entered",
              "result": "damage commits"
            },
            {
              "edge": "qs.draw",
              "state": "entered",
              "result": "one card movement commits"
            }
          ],
          "resume": "parent resumes after draw completion",
          "terminal": "none unless hit killed target"
        },
        "final_enemy": {
          "precondition": "hit lethally damages final enemy; player survives",
          "ordered_results": [
            {
              "edge": "qs.hit",
              "state": "entered",
              "result": "lethal hit/death callbacks commit"
            },
            {
              "edge": "qs.draw",
              "state": "entered",
              "result": "one draw commits or explicit shared draw-guard NoEffect"
            }
          ],
          "resume": "draw result settles before victory; no unrelated queue drain",
          "terminal": "victory candidate after required draw result"
        },
        "player_death": {
          "precondition": "hit callback commits actual player death before draw entry",
          "ordered_results": [
            {
              "edge": "qs.hit",
              "state": "entered",
              "result": "committed hit/death prefix"
            },
            {
              "edge": "qs.draw",
              "state": "not_entered",
              "result": "interrupted by PlayerDeath; no draw"
            }
          ],
          "resume": "no gameplay resume after PlayerDeath",
          "terminal": "PlayerDeath"
        }
      },
      "evidence_kind": "accepted E21 source member plus shared primitive contract; selected settlement policy separately labeled"
    },
    {
      "id": "Slice",
      "source": "2026-10-19-act1-content-silent.md:91",
      "branches": [
        {
          "version": "base",
          "captured": {
            "target": "selected monster at admitted use",
            "damage": {
              "type": "NORMAL",
              "amount": 6,
              "owner": "player"
            }
          },
          "edges": [
            {
              "key": "slice.hit",
              "kind": "DamageAction",
              "receiver": "captured target",
              "value_type": "DamageInfo NORMAL",
              "value": 6,
              "order": 1,
              "guard": "shared monster-damage entry guard",
              "commit": "target HP/death in damage primitive"
            }
          ],
          "interrupt": "actual player death terminates entered callback chain and no later work",
          "terminal": "settle hit/death callbacks before victory candidate"
        },
        {
          "version": "upgraded",
          "captured": {
            "target": "selected monster at admitted use",
            "damage": {
              "type": "NORMAL",
              "amount": 9,
              "owner": "player"
            }
          },
          "edges": [
            {
              "key": "slice.hit",
              "kind": "DamageAction",
              "receiver": "captured target",
              "value_type": "DamageInfo NORMAL",
              "value": 9,
              "order": 1,
              "guard": "shared monster-damage entry guard",
              "commit": "target HP/death in damage primitive"
            }
          ],
          "interrupt": "actual player death terminates entered callback chain and no later work",
          "terminal": "settle hit/death callbacks before victory candidate"
        }
      ],
      "cases": {
        "normal": {
          "precondition": "player survives hit callbacks",
          "ordered_results": [
            {
              "edge": "slice.hit",
              "state": "entered",
              "result": "one damage hit commits"
            }
          ],
          "resume": "parent continues after hit chain",
          "terminal": "none unless hit killed target"
        },
        "final_enemy": {
          "precondition": "Slice hit lethally damages the final target",
          "ordered_results": [
            {
              "edge": "slice.hit",
              "state": "entered",
              "result": "lethal hit and mandatory death callbacks settle"
            }
          ],
          "resume": "causal hit chain settles before victory",
          "terminal": "victory candidate after hit chain"
        },
        "player_death": {
          "precondition": "hit callback commits actual player death",
          "ordered_results": [
            {
              "edge": "slice.hit",
              "state": "entered",
              "result": "committed hit prefix; PlayerDeath interrupts later work"
            }
          ],
          "resume": "no gameplay resume after PlayerDeath",
          "terminal": "PlayerDeath"
        }
      },
      "evidence_kind": "accepted E21 source member plus shared primitive contract; selected settlement policy separately labeled"
    }
  ],
  "provenance": {
    "expected_contract_owner": "six-normative-v7",
    "public_representation": "sole normative structured JSON; visible table is deterministic informative projection whose semantic JSON cells are parsed and compared to the source/policy oracle",
    "source_policy_boundary": "Poisoned Stab uses actual ApplyPowerAction isDeadOrEscaped helper; no positive-HP-only source or native guard"
  },
  "source_policy_provenance": {
    "e21": "accepted 2026-10-19-act1-content-silent.md:88; Poisoned Stab emits Damage then M Poison without nonlethal condition",
    "apply_power_rule": "E21 §7: ApplyPowerAction uses its shared target guard, per-application hooks/prevention/Artifact and stack/append behavior",
    "retained_bytecode": "combat-laws-v5/evidence/{05-AbstractCreature,06-AbstractMonster,11-ApplyPowerAction}.stdout.txt; commands/pins in combat-laws-v5/evidence/javap-commands.json",
    "guard_authority": "actual transitive AbstractCreature.isDeadOrEscaped helper, not helper name inference and not direct HP positivity",
    "bytecode_output_sha256": {
      "AbstractCreature.isDeadOrEscaped (05-AbstractCreature.stdout.txt)": "71a6a2922a4dcdc28ead5ee96d160725aa2e354264f54de40319a8a02b7fd468",
      "AbstractMonster (06-AbstractMonster.stdout.txt)": "723e753d43429b4bc110a560a44a00f58c43049581f01b7719741754b7e643ba",
      "ApplyPowerAction.update (11-ApplyPowerAction.stdout.txt)": "a8601758e79bb07dbb80c895f917154dbb178c251e8e437ea2e49f1033f573ef"
    },
    "bytecode_method_anchors": {
      "AbstractCreature.isDeadOrEscaped": "05 output lines 2792-2815",
      "AbstractMonster.damage": "06 output lines 2002-2024, 2299-2304",
      "AbstractMonster.die(boolean)": "06 output lines 3530-3558; isDying set before hooks",
      "AbstractMonster.updateDeathAnimation": "06 output lines 3339-3390; isDead later",
      "ApplyPowerAction.update": "11 output lines 237-250 and 272-343"
    }
  },
  "shared_life_state_binding": {
    "source_guard": "AbstractCreature.isDeadOrEscaped",
    "helper_truth_table": {
      "hp_nonpositive_only": false,
      "is_dying": true,
      "half_dead": true,
      "monster_is_escaping": true,
      "monster_escaped_flag_only": false,
      "is_dead_flag_only": false
    },
    "helper_definition": "isDying OR halfDead OR (not isPlayer AND AbstractMonster.isEscaping); does not read currentHealth, isDead, or AbstractMonster.escaped",
    "apply_power_checkpoints": [
      "ApplyPowerAction.update: null target OR helper true => finish without entering application",
      "after source onApplyPower hooks and Champion Belt branch: monster helper is rechecked; helper true => finish before target-side application/Artifact/stack append"
    ],
    "lethal_hit_checkpoint": "synchronous target.damage commits HP, invokes die(), sets isDying, and completes synchronous death hooks before DamageAction returns",
    "death_hook_order": "AbstractMonster.damage after HP<=0 invokes die(); die sets isDying first, then synchronous power onDeath hooks and relic onMonsterDeath hooks when enabled; returns with deathTimer active and isDead not yet set",
    "logical_death_checkpoint": "later AbstractMonster.updateDeathAnimation after deathTimer expires sets isDead; do not advance this frame/timer checkpoint ahead of the required immediate child",
    "child_entry_checkpoint": "after synchronous DamageAction target.damage/die hooks return; before victory candidate",
    "native_life_state_at_lethal_child": "isDying=true; isDead transition remains later and is not advanced before this child",
    "positive_hp_policy": "none; HP alone is not the shared helper predicate",
    "player_death": "actual PlayerDeath before child entry interrupts not-entered Poison child; no post-death effect",
    "remaining_global_dependency": "unrelated queue contents and independently queued work from callbacks remain governed by the shared scheduler and are not drained or claimed by this local producer contract",
    "poison_hp_nonpositive_control": {
      "precondition": "HP<=0 while isDying=false, halfDead=false, and monster isEscaping=false",
      "helper_result": false,
      "meaning": "ApplyPower helper itself would not reject this state; no direct HP test exists. This state is not the ordinary post-lethal DamageAction checkpoint because die() is synchronous."
    }
  }
}
```
<!-- /six-member-semantic-contracts-v7 -->

**Generated informative projection (all semantic cells are JSON data):**

<!-- six-member-generated-view-v7 -->
| Member | Branch | E21 source | Captures (JSON) | Ordered edges (JSON) | Interruption, terminal and cases (JSON) |
|---|---|---|---|---|---|
| Dash | base | 2026-10-19-act1-content-silent.md:112 | {"target":"selected monster at admitted use","damage":{"type":"NORMAL","amount":10,"owner":"player"},"block":{"type":"Block","amount":10,"owner":"player"}} | [{"key":"dash.block","kind":"BlockGain","receiver":"player","value_type":"Block","value":10,"order":1,"guard":"shared Block primitive entry guard","commit":"player Block mutation in Block primitive"},{"key":"dash.hit","kind":"DamageAction","receiver":"captured target","value_type":"DamageInfo NORMAL","value":10,"order":2,"guard":"shared monster-damage entry guard; never retarget","commit":"target HP/death in monster damage primitive"}] | {"interrupt":"player death during an entered edge stops every later edge; block remains committed if hit callbacks cause death","terminal":"settle hit death callbacks before victory candidate","cases":{"normal":{"precondition":"selected target survives and no callback kills player","ordered_results":[{"edge":"dash.block","state":"entered","result":"committed Block"},{"edge":"dash.hit","state":"entered","result":"committed target damage"}],"resume":"parent continues after both edges","terminal":"no victory unless hit kills final target"},"final_enemy":{"precondition":"dash.hit lethally damages the final target","ordered_results":[{"edge":"dash.block","state":"entered","result":"committed Block"},{"edge":"dash.hit","state":"entered","result":"committed target damage and death callbacks"}],"resume":"causal chain settles before victory","terminal":"victory candidate may commit only after hit callbacks"},"player_death":{"precondition":"hit callback commits actual player death","ordered_results":[{"edge":"dash.block","state":"entered","result":"committed Block"},{"edge":"dash.hit","state":"entered","result":"committed hit prefix; death interrupts remaining descendants"}],"resume":"no gameplay resume after PlayerDeath","terminal":"PlayerDeath"}}} |
| Dash | upgraded | 2026-10-19-act1-content-silent.md:112 | {"target":"selected monster at admitted use","damage":{"type":"NORMAL","amount":13,"owner":"player"},"block":{"type":"Block","amount":13,"owner":"player"}} | [{"key":"dash.block","kind":"BlockGain","receiver":"player","value_type":"Block","value":13,"order":1,"guard":"shared Block primitive entry guard","commit":"player Block mutation in Block primitive"},{"key":"dash.hit","kind":"DamageAction","receiver":"captured target","value_type":"DamageInfo NORMAL","value":13,"order":2,"guard":"shared monster-damage entry guard; never retarget","commit":"target HP/death in monster damage primitive"}] | {"interrupt":"player death during an entered edge stops every later edge; block remains committed if hit callbacks cause death","terminal":"settle hit death callbacks before victory candidate","cases":{"normal":{"precondition":"selected target survives and no callback kills player","ordered_results":[{"edge":"dash.block","state":"entered","result":"committed Block"},{"edge":"dash.hit","state":"entered","result":"committed target damage"}],"resume":"parent continues after both edges","terminal":"no victory unless hit kills final target"},"final_enemy":{"precondition":"dash.hit lethally damages the final target","ordered_results":[{"edge":"dash.block","state":"entered","result":"committed Block"},{"edge":"dash.hit","state":"entered","result":"committed target damage and death callbacks"}],"resume":"causal chain settles before victory","terminal":"victory candidate may commit only after hit callbacks"},"player_death":{"precondition":"hit callback commits actual player death","ordered_results":[{"edge":"dash.block","state":"entered","result":"committed Block"},{"edge":"dash.hit","state":"entered","result":"committed hit prefix; death interrupts remaining descendants"}],"resume":"no gameplay resume after PlayerDeath","terminal":"PlayerDeath"}}} |
| Die Die Die | base | 2026-10-19-act1-content-silent.md:149 | {"target_set":"source all-enemy traversal at use","damage_matrix":{"type":"NORMAL","amount":13,"owner":"player","per_target":true}} | [{"key":"ddd.area","kind":"DamageAllEnemiesAction","receiver":"captured target set","value_type":"per-target DamageInfo NORMAL matrix","value":13,"order":1,"guard":"shared area/monster-damage primitive guards per recipient","commit":"independent recipient HP/death commits in source traversal order"}] | {"interrupt":"player death in an entered recipient chain stops not-entered later producer children; no second card payload edge","terminal":"finish the entered area loop and its required death callbacks before victory","cases":{"normal":{"precondition":"player survives all recipient callbacks","ordered_results":[{"edge":"ddd.area","state":"entered","result":"each captured recipient receives its committed hit"}],"resume":"parent continues after area loop","terminal":"none unless captured recipients die"},"final_enemy":{"precondition":"one captured recipient is the final lethal target","ordered_results":[{"edge":"ddd.area","state":"entered","result":"source-order recipient hit/death prefix completes"}],"resume":"area loop and mandatory death callbacks settle before victory","terminal":"victory candidate after entered area causal work"},"player_death":{"precondition":"recipient callback commits actual player death during area loop","ordered_results":[{"edge":"ddd.area","state":"entered","result":"committed recipient prefix; PlayerDeath interrupts later gameplay descendants"}],"resume":"no gameplay resume after PlayerDeath","terminal":"PlayerDeath; no Exhaust payload edge"}}} |
| Die Die Die | upgraded | 2026-10-19-act1-content-silent.md:149 | {"target_set":"source all-enemy traversal at use","damage_matrix":{"type":"NORMAL","amount":17,"owner":"player","per_target":true}} | [{"key":"ddd.area","kind":"DamageAllEnemiesAction","receiver":"captured target set","value_type":"per-target DamageInfo NORMAL matrix","value":17,"order":1,"guard":"shared area/monster-damage primitive guards per recipient","commit":"independent recipient HP/death commits in source traversal order"}] | {"interrupt":"player death in an entered recipient chain stops not-entered later producer children; no second card payload edge","terminal":"finish the entered area loop and its required death callbacks before victory","cases":{"normal":{"precondition":"player survives all recipient callbacks","ordered_results":[{"edge":"ddd.area","state":"entered","result":"each captured recipient receives its committed hit"}],"resume":"parent continues after area loop","terminal":"none unless captured recipients die"},"final_enemy":{"precondition":"one captured recipient is the final lethal target","ordered_results":[{"edge":"ddd.area","state":"entered","result":"source-order recipient hit/death prefix completes"}],"resume":"area loop and mandatory death callbacks settle before victory","terminal":"victory candidate after entered area causal work"},"player_death":{"precondition":"recipient callback commits actual player death during area loop","ordered_results":[{"edge":"ddd.area","state":"entered","result":"committed recipient prefix; PlayerDeath interrupts later gameplay descendants"}],"resume":"no gameplay resume after PlayerDeath","terminal":"PlayerDeath; no Exhaust payload edge"}}} |
| Leg Sweep | base | 2026-10-19-act1-content-silent.md:123 | {"target":"selected monster at admitted use","weak":{"type":"Weak","amount":2},"block":{"type":"Block","amount":11,"owner":"player"}} | [{"key":"leg.weak","kind":"ApplyPowerAction","receiver":"captured target","value_type":"Weak debuff","value":2,"order":1,"guard":"non-null/live target and independent debuff/Artifact/prevention guard","commit":"accepted Weak stack/append in power primitive"},{"key":"leg.block","kind":"BlockGain","receiver":"player","value_type":"Block","value":11,"order":2,"guard":"shared Block primitive entry guard","commit":"player Block mutation"}] | {"interrupt":"player death during Weak application/callback prevents later Block; committed Weak remains","terminal":"settle Weak application/callback then Block before victory candidate","cases":{"normal":{"precondition":"target accepts Weak; player survives callbacks","ordered_results":[{"edge":"leg.weak","state":"entered","result":"Weak application commits"},{"edge":"leg.block","state":"entered","result":"player Block commits"}],"resume":"parent continues after both edges","terminal":"none unless callback creates victory"},"final_enemy":{"precondition":"Weak callbacks or prior state make this the final enemy resolution","ordered_results":[{"edge":"leg.weak","state":"entered","result":"accepted Weak commit or typed power no-effect"},{"edge":"leg.block","state":"entered","result":"Block commits"}],"resume":"both causal edges settle before victory","terminal":"victory candidate after Block"},"player_death":{"precondition":"Weak application callback commits actual player death","ordered_results":[{"edge":"leg.weak","state":"entered","result":"committed power prefix; callback death"},{"edge":"leg.block","state":"not_entered","result":"interrupted; no Block mutation"}],"resume":"no gameplay resume after PlayerDeath","terminal":"PlayerDeath"}}} |
| Leg Sweep | upgraded | 2026-10-19-act1-content-silent.md:123 | {"target":"selected monster at admitted use","weak":{"type":"Weak","amount":3},"block":{"type":"Block","amount":14,"owner":"player"}} | [{"key":"leg.weak","kind":"ApplyPowerAction","receiver":"captured target","value_type":"Weak debuff","value":3,"order":1,"guard":"non-null/live target and independent debuff/Artifact/prevention guard","commit":"accepted Weak stack/append in power primitive"},{"key":"leg.block","kind":"BlockGain","receiver":"player","value_type":"Block","value":14,"order":2,"guard":"shared Block primitive entry guard","commit":"player Block mutation"}] | {"interrupt":"player death during Weak application/callback prevents later Block; committed Weak remains","terminal":"settle Weak application/callback then Block before victory candidate","cases":{"normal":{"precondition":"target accepts Weak; player survives callbacks","ordered_results":[{"edge":"leg.weak","state":"entered","result":"Weak application commits"},{"edge":"leg.block","state":"entered","result":"player Block commits"}],"resume":"parent continues after both edges","terminal":"none unless callback creates victory"},"final_enemy":{"precondition":"Weak callbacks or prior state make this the final enemy resolution","ordered_results":[{"edge":"leg.weak","state":"entered","result":"accepted Weak commit or typed power no-effect"},{"edge":"leg.block","state":"entered","result":"Block commits"}],"resume":"both causal edges settle before victory","terminal":"victory candidate after Block"},"player_death":{"precondition":"Weak application callback commits actual player death","ordered_results":[{"edge":"leg.weak","state":"entered","result":"committed power prefix; callback death"},{"edge":"leg.block","state":"not_entered","result":"interrupted; no Block mutation"}],"resume":"no gameplay resume after PlayerDeath","terminal":"PlayerDeath"}}} |
| Poisoned Stab | base | 2026-10-19-act1-content-silent.md:88; §7 ApplyPowerAction rules | {"target":"selected monster at admitted use","damage":{"type":"NORMAL","amount":6,"owner":"player"},"poison":{"type":"Poison","amount":3}} | [{"key":"ps.hit","kind":"DamageAction","receiver":"captured target","value_type":"DamageInfo NORMAL","value":6,"order":1,"guard":"shared monster-damage entry guard","commit":"target HP/death in damage primitive"},{"key":"ps.poison","kind":"ApplyPowerAction","receiver":"captured target","value_type":"Poison debuff","value":3,"order":2,"guard":"ApplyPowerAction.update target null/helper guard; after source hooks, recheck monster helper before application; no direct HP guard","commit":"accepted Poison stack/append only; no mutation on no-effect or prevention"}] | {"interrupt":"actual player death before Poison child entry interrupts it; no post-death Poison","terminal":"settle Poison child using actual shared helper after synchronous hit/death hooks and before victory candidate; no HP-only policy","cases":{"normal":{"precondition":"captured target is not dying, half-dead, or escaping at child entry; player survives","ordered_results":[{"edge":"ps.hit","state":"entered","result":"damage commits; nonlethal target remains outside shared dead-or-escaped helper"},{"edge":"ps.poison","state":"entered","result":"ApplyPower source hooks/target recheck/prevention/Artifact run; Poison commits only if accepted"}],"resume":"parent continues after entered shared power result","terminal":"none unless hit killed target"},"final_enemy":{"precondition":"lethal hit synchronously sets isDying and completes death hooks; player survives","ordered_results":[{"edge":"ps.hit","state":"entered","result":"HP/death prefix commits; die sets isDying before synchronous death hooks return"},{"edge":"ps.poison","state":"entered","result":"shared isDeadOrEscaped helper is true from isDying; ApplyPower exits with no Poison mutation"}],"resume":"no retarget; required child no-effect settles before victory","terminal":"victory candidate only after Poison child helper-guard result"},"player_death":{"precondition":"hit callback commits actual player death before Poison child entry","ordered_results":[{"edge":"ps.hit","state":"entered","result":"committed hit/death prefix"},{"edge":"ps.poison","state":"not_entered","result":"interrupted by PlayerDeath; no mutation"}],"resume":"no gameplay resume after PlayerDeath","terminal":"PlayerDeath"}}} |
| Poisoned Stab | upgraded | 2026-10-19-act1-content-silent.md:88; §7 ApplyPowerAction rules | {"target":"selected monster at admitted use","damage":{"type":"NORMAL","amount":8,"owner":"player"},"poison":{"type":"Poison","amount":4}} | [{"key":"ps.hit","kind":"DamageAction","receiver":"captured target","value_type":"DamageInfo NORMAL","value":8,"order":1,"guard":"shared monster-damage entry guard","commit":"target HP/death in damage primitive"},{"key":"ps.poison","kind":"ApplyPowerAction","receiver":"captured target","value_type":"Poison debuff","value":4,"order":2,"guard":"ApplyPowerAction.update target null/helper guard; after source hooks, recheck monster helper before application; no direct HP guard","commit":"accepted Poison stack/append only; no mutation on no-effect or prevention"}] | {"interrupt":"actual player death before Poison child entry interrupts it; no post-death Poison","terminal":"settle Poison child using actual shared helper after synchronous hit/death hooks and before victory candidate; no HP-only policy","cases":{"normal":{"precondition":"captured target is not dying, half-dead, or escaping at child entry; player survives","ordered_results":[{"edge":"ps.hit","state":"entered","result":"damage commits; nonlethal target remains outside shared dead-or-escaped helper"},{"edge":"ps.poison","state":"entered","result":"ApplyPower source hooks/target recheck/prevention/Artifact run; Poison commits only if accepted"}],"resume":"parent continues after entered shared power result","terminal":"none unless hit killed target"},"final_enemy":{"precondition":"lethal hit synchronously sets isDying and completes death hooks; player survives","ordered_results":[{"edge":"ps.hit","state":"entered","result":"HP/death prefix commits; die sets isDying before synchronous death hooks return"},{"edge":"ps.poison","state":"entered","result":"shared isDeadOrEscaped helper is true from isDying; ApplyPower exits with no Poison mutation"}],"resume":"no retarget; required child no-effect settles before victory","terminal":"victory candidate only after Poison child helper-guard result"},"player_death":{"precondition":"hit callback commits actual player death before Poison child entry","ordered_results":[{"edge":"ps.hit","state":"entered","result":"committed hit/death prefix"},{"edge":"ps.poison","state":"not_entered","result":"interrupted by PlayerDeath; no mutation"}],"resume":"no gameplay resume after PlayerDeath","terminal":"PlayerDeath"}}} |
| Quick Slash | base | 2026-10-19-act1-content-silent.md:90; shared draw transaction §3.3A.1 | {"target":"selected monster at admitted use","damage":{"type":"NORMAL","amount":8,"owner":"player"},"draw_count":1} | [{"key":"qs.hit","kind":"DamageAction","receiver":"captured target","value_type":"DamageInfo NORMAL","value":8,"order":1,"guard":"shared monster-damage entry guard","commit":"target HP/death in damage primitive"},{"key":"qs.draw","kind":"DrawCardAction","receiver":"player draw/hand","value_type":"draw count","value":1,"order":2,"guard":"on entry apply No Draw, hand-capacity, and pile-availability draw guards","commit":"card becomes hand member at actual draw movement; no-draw exit commits no card movement"}] | {"interrupt":"actual player death before draw entry interrupts draw; no card after death","terminal":"draw child or its explicit no-effect result settles after lethal hit and before victory candidate","cases":{"normal":{"precondition":"player survives; draw primitive permits one card","ordered_results":[{"edge":"qs.hit","state":"entered","result":"damage commits"},{"edge":"qs.draw","state":"entered","result":"one card movement commits"}],"resume":"parent resumes after draw completion","terminal":"none unless hit killed target"},"final_enemy":{"precondition":"hit lethally damages final enemy; player survives","ordered_results":[{"edge":"qs.hit","state":"entered","result":"lethal hit/death callbacks commit"},{"edge":"qs.draw","state":"entered","result":"one draw commits or explicit shared draw-guard NoEffect"}],"resume":"draw result settles before victory; no unrelated queue drain","terminal":"victory candidate after required draw result"},"player_death":{"precondition":"hit callback commits actual player death before draw entry","ordered_results":[{"edge":"qs.hit","state":"entered","result":"committed hit/death prefix"},{"edge":"qs.draw","state":"not_entered","result":"interrupted by PlayerDeath; no draw"}],"resume":"no gameplay resume after PlayerDeath","terminal":"PlayerDeath"}}} |
| Quick Slash | upgraded | 2026-10-19-act1-content-silent.md:90; shared draw transaction §3.3A.1 | {"target":"selected monster at admitted use","damage":{"type":"NORMAL","amount":12,"owner":"player"},"draw_count":1} | [{"key":"qs.hit","kind":"DamageAction","receiver":"captured target","value_type":"DamageInfo NORMAL","value":12,"order":1,"guard":"shared monster-damage entry guard","commit":"target HP/death in damage primitive"},{"key":"qs.draw","kind":"DrawCardAction","receiver":"player draw/hand","value_type":"draw count","value":1,"order":2,"guard":"on entry apply No Draw, hand-capacity, and pile-availability draw guards","commit":"card becomes hand member at actual draw movement; no-draw exit commits no card movement"}] | {"interrupt":"actual player death before draw entry interrupts draw; no card after death","terminal":"draw child or its explicit no-effect result settles after lethal hit and before victory candidate","cases":{"normal":{"precondition":"player survives; draw primitive permits one card","ordered_results":[{"edge":"qs.hit","state":"entered","result":"damage commits"},{"edge":"qs.draw","state":"entered","result":"one card movement commits"}],"resume":"parent resumes after draw completion","terminal":"none unless hit killed target"},"final_enemy":{"precondition":"hit lethally damages final enemy; player survives","ordered_results":[{"edge":"qs.hit","state":"entered","result":"lethal hit/death callbacks commit"},{"edge":"qs.draw","state":"entered","result":"one draw commits or explicit shared draw-guard NoEffect"}],"resume":"draw result settles before victory; no unrelated queue drain","terminal":"victory candidate after required draw result"},"player_death":{"precondition":"hit callback commits actual player death before draw entry","ordered_results":[{"edge":"qs.hit","state":"entered","result":"committed hit/death prefix"},{"edge":"qs.draw","state":"not_entered","result":"interrupted by PlayerDeath; no draw"}],"resume":"no gameplay resume after PlayerDeath","terminal":"PlayerDeath"}}} |
| Slice | base | 2026-10-19-act1-content-silent.md:91 | {"target":"selected monster at admitted use","damage":{"type":"NORMAL","amount":6,"owner":"player"}} | [{"key":"slice.hit","kind":"DamageAction","receiver":"captured target","value_type":"DamageInfo NORMAL","value":6,"order":1,"guard":"shared monster-damage entry guard","commit":"target HP/death in damage primitive"}] | {"interrupt":"actual player death terminates entered callback chain and no later work","terminal":"settle hit/death callbacks before victory candidate","cases":{"normal":{"precondition":"player survives hit callbacks","ordered_results":[{"edge":"slice.hit","state":"entered","result":"one damage hit commits"}],"resume":"parent continues after hit chain","terminal":"none unless hit killed target"},"final_enemy":{"precondition":"Slice hit lethally damages the final target","ordered_results":[{"edge":"slice.hit","state":"entered","result":"lethal hit and mandatory death callbacks settle"}],"resume":"causal hit chain settles before victory","terminal":"victory candidate after hit chain"},"player_death":{"precondition":"hit callback commits actual player death","ordered_results":[{"edge":"slice.hit","state":"entered","result":"committed hit prefix; PlayerDeath interrupts later work"}],"resume":"no gameplay resume after PlayerDeath","terminal":"PlayerDeath"}}} |
| Slice | upgraded | 2026-10-19-act1-content-silent.md:91 | {"target":"selected monster at admitted use","damage":{"type":"NORMAL","amount":9,"owner":"player"}} | [{"key":"slice.hit","kind":"DamageAction","receiver":"captured target","value_type":"DamageInfo NORMAL","value":9,"order":1,"guard":"shared monster-damage entry guard","commit":"target HP/death in damage primitive"}] | {"interrupt":"actual player death terminates entered callback chain and no later work","terminal":"settle hit/death callbacks before victory candidate","cases":{"normal":{"precondition":"player survives hit callbacks","ordered_results":[{"edge":"slice.hit","state":"entered","result":"one damage hit commits"}],"resume":"parent continues after hit chain","terminal":"none unless hit killed target"},"final_enemy":{"precondition":"Slice hit lethally damages the final target","ordered_results":[{"edge":"slice.hit","state":"entered","result":"lethal hit and mandatory death callbacks settle"}],"resume":"causal hit chain settles before victory","terminal":"victory candidate after hit chain"},"player_death":{"precondition":"hit callback commits actual player death","ordered_results":[{"edge":"slice.hit","state":"entered","result":"committed hit prefix; PlayerDeath interrupts later work"}],"resume":"no gameplay resume after PlayerDeath","terminal":"PlayerDeath"}}} |
<!-- /six-member-generated-view-v7 -->

#### 3.3B Exact bounded play-origin admission/payment/history/cleanup matrix (F facts; S settlement)

“Common gate” means actual `canUse` and target validation; autoplay is not a legality bypass. Captured energy, queue `ignoreEnergyTotal`, derived card `ignoreEnergyOnUse`, autoplay, and one-shot-free are independent values. Late invalid target retains already committed admission history but omits player use/payment; early gate rejection has no normal play history. “Ordinary history” means the accepted manager's on-play notification/history branch.

| Play origin / route | Admission and target rule | Captures / flags | Exact payment checkpoint | History / callbacks | Cleanup / failed branch |
|---|---|---|---|---|---|
| Hand, ordinary | Common gate; normal chosen target | Current manager energy capture; autoplay=false; ignore-total=false; preserve current per-instance free flag | After use-entry callbacks, debit ordinary turn cost unless one-shot free or Corruption-Skill exemption; X actions retain their own per-action guards | On successful admission, normal history/notifications then use callbacks and payload | UseCardAction callbacks then free reset then actual exhaust/rebound/shuffle/return/discard precedence. Early reject: no history/use/payment. Late invalid: keep admitted history, omit use/payment, use limbo/fade branch. |
| Hand, one-shot free | Same common gate as ordinary hand | Same captures; free flag=true; it does not set autoplay or ignore-total | No ordinary debit; X action's own free/live-debit guard remains applicable | Same admitted normal history/callbacks | Same cleanup; clear free only at its actual after-use cleanup checkpoint. |
| Havoc top card | Move actual draw-top object through limbo; child uses common gate | NewQueue's then-current energy; autoplay=true, queue ignore-total=false; manager derives card ignore-on-use=true only after canUse; autoplay route sets X free state | No ordinary selected-card debit | Admitted child uses manager's ordinary accepted history/callback path; rejected gate has no normal history/on-use branch | Havoc requests exhaust once; actual Power/purge/Spoon/ordinary destination precedence controls. Failed child is not successful play. |
| Mayhem top card | Deferred wrapper then top-card action; child common gate | Captured at queue construction; autoplay=true; preserve queue/derived flag distinction | No ordinary debit | Admitted normal history/callback path; failure has no normal history | No Havoc-imposed exhaust; card/source flags determine actual destination. |
| Distilled Chaos top card | Potion builds 3–6 actions and captures each target proposal before resolution; each child then enters common gate | Per-action target/energy retained; autoplay route, not repeat-power flags | No ordinary selected-card debit | Each admitted child receives normal manager history/callback path at its actual entry | Per-action destination; no blanket exhaust. Earlier child does not regenerate later captured target. |
| Repeat power: Double Tap, Burst, Amplify, Duplication, Echo | Each queued repeat independently enters common gate | Original energyOnUse; explicitly autoplay=true **and ignoreEnergyTotal=true**; same-instance copy/linkage | No second energy debit for repeated child; retain one-shot/free reset separately | Each accepted repeat takes normal eligible play-history/callback path; callback frequency follows actual instance notification | Purge-on-use after callbacks; repeats retain producer order. Rejected repeat is not normal play. |
| Omniscience original | Selected draw instance is marked exhaust/removed before queue; later common gate | NewQueue request randomTarget=true, immediateCard=false, autoplay=true, ignoreEnergyTotal=false; energy captured when NewQueue action executes; card ignore-on-use derived=true only after canUse | No ordinary debit via autoplay; never rewrite queue flag to derived card flag | Admitted manager history/callback path; failure has no normal history and does not undo selection/removal | Original exhaust route with ordinary after-use precedence. |
| Omniscience copy | Fresh stat-equivalent purge-on-use copy constructed before either child resolves; independently common gate | Fresh identity; same queue flags/capture as original (autoplay=true, ignore-total=false); derived ignore-on-use=true after gate | No ordinary debit | Its own admitted manager history/callback path; construction itself is not play history | Purge-on-use after callbacks; original result does not alter copy fields. |
| Other potion-queued play | Only an explicitly named potion producer may dispatch; child still uses common gate | Preserve that producer's captured target/energy/flags; no generic potion defaults | No ordinary hand-card debit unless its concrete source row explicitly says otherwise; potion consume is separate parent commit | Admitted child manager history; early reject none; resumption never repeats potion use | Producer-specific destination only. Missing source variant row faults before mutation rather than selecting a fallback. |
| Generated/created offer (not yet played) | Not a play origin; parent choice and obtain rules apply | Creation captures no energy/X/play flags | No card payment at creation or selection; purchase/payment only at owning parent checkpoint | No play history/onPlay callbacks until a later admitted play | Creation/obtain destination per parent; no UseCardAction cleanup merely from construction. |
| `ReplaySource` / `OtherExplicitSource` | No wildcard admission rule: a new source must be named with its concrete member/variant before dispatch; then that row states common or distinct gate | No inferred copy identity, target, capture, autoplay or ignore flags | No inferred payment waiver | No inferred notification/history suppression | No inferred destination. Fail closed until a complete row is published. |

The matrix closes the bounded routes named here; source facts remain F and deterministic settlement remains S. A future source variant must supply both rows in §3.3A and all six columns in this matrix. No generic enum member substitutes for the rule.

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

Shared ApplyPower life-state binding for the six-card Poison child: `AbstractCreature.isDeadOrEscaped()` is exactly `isDying || halfDead || (!isPlayer && AbstractMonster.isEscaping)` in the retained source output; it does not test HP, `isDead`, or `AbstractMonster.escaped`. `AbstractMonster.damage()` synchronously invokes `die()` after HP≤0; `die()` sets `isDying` before synchronous death hooks, while the later death-timer update sets `isDead`. The §3.3A.4 native causal checkpoint runs the required child after that synchronous prefix and before the later timer checkpoint, so the child observes the actual helper guard without a synthetic HP-positive test. This is a local producer binding, not a claim that all callback queues drain.

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
| Purifier, Transmogrifier, Wing mandatory removal grid | **Source fact:** these committed source paths have no local empty guard; Wing damage is already before later selection, and the Transmogrifier caller includes its source-local Complete/Leave behavior while the grid is open. **Selected Transmogrifier exception:** on live nonempty permanent deck with zero eligible candidates, native bypasses grid into `TransmogrifierEmptyTransformNoOp`, closes only that child and resumes the existing Complete/Leave decision. | The selected Transmogrifier result bridge is a native policy, not a source empty guard or source no-op fact. Purifier/Wing do not inherit it. Exceptional source reachability remains unproved; no generic fallback or guaranteed-deadlock claim. |
| Note empty outgoing candidates | Incoming hooks/deck insertion precede selection; removable incoming supplies an actual candidate. If incoming is unremovable and old deck has none, local selector has no guard. | Legal initialized-history reachability unresolved; neither assert nor exclude from source branch. |
| Empty ordinary card reward/Dream | Empty card row/screen is omitted; other reward rows/Proceed survive. | Native has no empty candidate for absent row. |
| Full belt/Sozu | Full belt leaves potion reward available; Sozu reward claim can consume without insertion; Shop Sozu/full-belt refuses before spend/restock. | Exact caller-specific behavior, not universal acquire success. |
| Empty relic offer pool | Depleted common/uncommon/shop promotes with normal/front selector as specified; rare/boss Circlet fallback. Offer rejection consumes candidate under requested-tier route. | Do not turn exhaustion into generic skip or retry. |

No supplied-choice witness, forced fixture, static branch, or bounded search miss establishes legal initialized reachability/impossibility. Those remaining history proofs are not a source/runtime execution blocker for selected native semantic contract unless the whole-spec owner needs to set behavior for a newly found legal branch.

### 5.3 Decision suspension and exact bounded selector contract

The following rows replace a generic “selection handler” transition. The selected sequence is the submitted sequence when the source consumer iterates a list; otherwise the row says source-order subset. Candidate order is the visible/source-built order, never hidden pile order unless that order is already visible. Each row's result is committed once before the listed child; child completion is what resumes the saved parent.

| Owner / selector | Legal candidates and cardinality | Result commit, descendants, and resume |
|---|---|---|
| Neow reward offer | Choose exactly one of the current ordered eligible reward candidates; category 0 has 6, category 1 has 5, category 2 has the exact conditional eligible list after drawback; category 3 is the certain boss-swap offer. No initial Leave. | Commit chosen offer once; activate drawback before reward; generated offers/history stay committed even if later skipped where row allows. Complete each selection/obtain child before resuming Neow; final dialogue returns to map. |
| Neow ordinary card offers | Exactly 1 of 3 distinct current offers; each displayed offer identity is fixed for that continuation. | Take commits row before Fast obtain; Skip does not consume/re-roll row. Return only through Neow's result consumer. |
| Neow one/two-card removal/transform | Remove one: exactly 1 eligible purgeable; remove two/transform two: exactly 2 eligible purgeables; preserve selected order. Transform one: exactly 1; transform inputs commit removal before generation. | Validate all handles/count before first commit; remove through object hooks in selected order; generate after required removals; child obtains complete before parent resumes. No clamp or empty fallback. |
| Neow upgrade | Exactly 1 currently upgradeable permanent instance. | Upgrade that instance at accepted result; no second prompt; resume Neow. |
| Relic Bottle | Exactly 1 currently eligible typed permanent card; typed predicate must match relic. | Installed relic result consumer applies bottle flag once and completes room; no off-type fallback. |
| Dolly's Mirror | Exactly 1 current permanent-deck instance. | Commit selected source then create one stat-equivalent copy, clear bottle flags on copy, schedule Show obtain; room completion and obtain completion are separate; resume installed-relic consumer. |
| Astrolabe | 0 candidates: no selector; 1–3: automatic all, no decision; >3: choose exactly 3 from purgeable set in selected order. | For each input remove through object hook before upgraded transform; enqueue each resulting Show obtain in selected order; result consumer resumes after its own timer/children, not global drain. |
| Empty Cage | 0: source-local latch/phase branch in §5.2, no fabricated choice; 1–2: automatic remove all; >2: choose exactly 2 current purgeable objects. | For 1–2 each object removal commits and room completes; >2 commits selected removals then room completion. Zero preserves its distinct incomplete phase pending reachability disposition below. |
| Pandora's Box | Select no cards: if starter set empty, no confirmation selector; otherwise confirmation-only screen contains exactly all generated replacements, with no re-selection. | Remove starter-tagged cards by its iterator semantics; generate replacements; confirmation commits all queued Fast obtains. No generic skip/partial take. |
| Note Take | One initial Take/Ignore. On Take, incoming obtain and hooks commit before outgoing candidates are built. Then choose exactly 1 from current nonbottled purgeables; incoming is included iff it meets that live predicate. | Candidate list is built after incoming callbacks and deck insertion. Selected outgoing object-removal hooks commit before retaining its live ID/upgrade descriptor; persistence occurs at next room transition. Ignore has no outgoing selector or write. Incoming obtain does not consume Omamori. |
| Shop purge | Cancellable selector; exactly 1 current purgeable object to confirm, or cancel before payment. | Confirm payment/service-use commit precedes object removal; then resume merchant. Cancel commits nothing. Screen/room consumers cannot charge twice. |
| Ordinary card reward | Exactly 1 offer identity on Take, or distinct Skip; Bowl is a separate action when enabled, not a card selection. | Take removes row before Fast obtain; Skip retains same row for reopen; Bowl commits +2 maxHP then ordinary heal and removes associated row. Result consumer resumes reward list. |
| Dream Catcher/rest offer | Exactly 1 of fixed generated card offers or Skip/Bowl if that context exposes them; empty offer suppresses selector. | Take/Skip/Bowl closes to rest parent; no fabricated combat reward item. Heal/onRest commit precedes the generated offer. |
| Event single choice | Exact displayed source options, each committed result listed at the event row in §5.1; no generic Leave. | Commit selected branch once and enqueue only that branch's effects; source result/Proceed consumer resumes event or map according to its named route. |
| Event mandatory grid | Purifier/Transmogrifier/Wing committed source grid requests exactly 1; candidates are current purgeable instances in displayed order; no source Skip/Cancel after commit. Other event grids use their caller's exact count, not this row. For Transmogrifier only, selected native policy bypasses opening the source grid for nonempty deck/zero eligible instances and returns the typed no-op child result. | Preserve pre-grid payment/damage commits. Ordinary eligible Transmogrifier choice removes selected instance then generates; the mandatory obtain child remains in the same root. The zero-eligible native no-op closes that child then resumes Complete/Leave; it does not call Leave automatically or fabricate a source callback. Purifier/Wing retain their source empty-case dispositions. |
| Multi-target card choice | Exactly the source card's requested count/range and live candidate predicate (e.g. Forethought+ 0..hand size in selected order; Purity 0..3/5 other hand; fixed one-card retrieval is forced at one candidate and modal above one). | Captured producer data remains unchanged; commit selected ordered instances at that action's own movement/effect checkpoint, then resume parent card payload without replaying hit/payment. |
| Match and Keep | First pick exactly 1 face-down position; second pick exactly 1 other face-down position. Five attempts; there is no strategic Skip/Leave. Positions are public, unrevealed identities hidden. | Second reveal is immediate; pair resolution commits one attempt and match/miss outcome; match schedules fresh-copy obtain. Resume game only after pair resolution; fifth attempt transitions to cleanup, then Complete. |
| Wish / other option callback | Wish exposes exactly 1 of 3 ordered options (Strength, gold, Plated Armor); option callback is not a card play. | Commit exactly selected option effect once and resume card payload; no energy/card cleanup for option object. Other option menus require their own named finite source row. |
| Boss relic / reward | Boss relic offers: Take one offered relic or Skip/Proceed as enabled; each relic child is a separate continuation. Reward rows independently Take/Skip/Proceed; no forced claim. | Take/skip consumes its own row once; relic child result consumer must complete before chest resumes; then next-dungeon transition and Act2 init run before Act2Ready. |

Every `CHOICE` encountered at the mandatory frontier enters `Suspended` before any later frontier item or victory commit. A valid answer commits its exact listed result, waits for its children/result consumer, then resumes the exact saved frontier and terminal candidate. A no-choice automatic branch never suspends. Empty mandatory cases do not get an implicit legal answer; their scope disposition is separate and explicit in §5.4.

### 5.4 Exceptional-state scope disposition (O; no reachability claim)

| State | Source fact | Ordinary-history evidence and current disposition |
|---|---|---|
| Zero-candidate committed mandatory grid (Purifier, Transmogrifier, Wing Statue) | Source callers have no local empty guard; post-commit cancellation/fallback is not source-established. Wing damage precedes opening the later grid. | Selected native Transmogrifier no-op resolves only its narrow nonempty-deck/zero-eligible case. Purifier and Wing retain their unresolved source-empty disposition; no ordinary-history witness or exclusion is asserted. |
| Empty Cage at boss relic child | Zero purgeables sets selection-complete latch but leaves earlier room phase INCOMPLETE; 1–2 auto-remove/complete; >2 requests two. | This is a distinct phase branch; no initialized-history witness/exclusion is in accepted records. Keep zero-phase distinct; do not coerce to the 1–2 path. The exact visible parent/room action after zero is not closed by local branch evidence. |
| Note incoming unremovable and no other outgoing candidate | Incoming is directly obtained with hooks/deck insertion before selector construction; if removable, incoming itself is a candidate. Selector has no empty guard. | Selected default incoming Iron Wave/0 is removable, so that selected default excludes this empty branch. Supported profile overrides/legal stored-off-color values are not proven to exclude an unremovable incoming card; nor is a legal initialized witness established. Thus default-only exclusion is not universal full-scope proof. No fabricated outgoing choice or fallback is specified. |

**Main-directed disposition required:** the accepted static record set does not provide the ordinary-history witnesses/exclusions needed to close those exceptional states. This correction does not claim they are unreachable. If main supplies a proven legal history reaching any state, the remaining product-policy choice is exact: (a) preserve the source's committed no-choice/incomplete continuation (possible nonprogress), (b) add a selected native legal recovery/terminal rule, or (c) narrow legal profile/content scope. Option (b) changes player-visible behavior and cannot be chosen as a source fact. Until reachability is established or the policy choice is approved, this contract is deliberately **NOT CLOSED for exceptional empty-state full scope**. Product-policy escalation is conditional on such a proven reachable history; no original-runtime or frame-timer proof is requested.

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

`{edge_key, producer_member, producer_branch, parent_invocation, source_actor?, target_actor?, captured_inputs, effect_kind, relation, source_container, enqueue_operation, emitted_index, entered_guard, commit_checkpoint, interrupt_rule, tie_key, terminal_scope, outcome}`.

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
| B04 | Direct family inventories and selected card examples; mixed action/cost/copy interactions open | Explicit bounded origin/payment/history matrix for Hand, free, Havoc, Mayhem, Distilled Chaos, repeat powers, Omniscience original/copy and non-play offers; five-repeat/Omni flag distinction | `ReplaySource`/`OtherExplicitSource` fail closed until separately named; bounded rows do not prove every source-site variant is exhausted. No `SourceSpecific` payment fallback. |
| B05 | Source-specific continuation rows without exact common causal/terminal interface | Recursive owner grammar, explicit frontier suspension/result-commit/resume transition and named selector cardinalities | Exceptional empty-selector/Cage/Note ordinary-history reachability and outcome remain OPEN; §5.4 records missing witness/exclusion and policy options without inventing a source fallback. |
| B06 | Bounded player/monster branches and rescue/death facts, no combined terminal policy | Damage pipelines, Fatal/split/escape distinctions, once-only reward gate, death/victory/handoff arbitration | Full cross-product reactions and future trajectories unexecuted. Natural Reaper timer outcomes not established (selected policy settles its result). |
| B07 | Primary tables and dispatch families separately; callbacks count did not close joins | Typed causal-edge schema, selected native settlement/tie rules, named prior producer expansions, and the finite 76-member E21 Silent primary-card locator/payload crosswalk in §3.3A.3 | **EX-I1 remains open:** §3.3A.3 closes E21 primary-row membership and carries accepted local payload/branch references, but does not close cross-system recipient/death/history/RNG/copy/ordering residuals in E21 §10. E22–E24 card families, non-card producer families, exact accepted .52/.53 joins, and expected-trace verification remain pending; native site equality is separately pending. |
| B08 | Source-local offers/returns without common exactly-once child outcome | Parent-child contract for named Neow/shop/reward/relic/event/campfire/Note/Match/boss selectors and revalidation | Event-choice rows beyond the named selector contracts and exceptional empty-state histories remain open; no source outcome is fabricated. |
| B12 | Local source timer observations; selected policy not applied as transition model | Deterministic immediate-causal settlement explicitly replaces frames; T/TX cases aligned; Reaper heals on final kill | No source-frame parity, natural timer outcome, source RNG correlation, or original runtime. |

## 10. Evidence and limitations

The source-informed facts are bounded to the accepted inputs in §1; this author did not execute or re-run original source, classes, initializers, or gameplay. Static facts/model contracts do not establish all legal initialized histories or all cross-product effects. Future `TX01–TX18` and referenced `A1C-T*` cases remain **UNEXECUTED**. This document is the assigned single public document; detailed private source records, helper, raw logs and source bytes are not published. It is a corrected bounded draft but **not complete for all producer variants or exceptional empty-state full scope**; the precise remaining evidence and conditional policy escalation are recorded in §5.4 and §9. Whole-spec human approval, reviewed implementation plan and separate implementation-start approval remain unsatisfied.
