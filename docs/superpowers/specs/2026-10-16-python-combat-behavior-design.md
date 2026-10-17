# Narrow Python combat behavior under RNG-001

> **Approved sequencing amendment (2026-10-16):** The user approved
> [source-first Rust development](../../scope.md#approved-source-first-sequencing-amendment-2026-10-16).
> The earlier original-evidence-before-Rust restriction in section 10 is superseded.
> Original comparisons remain required for later fidelity promotion; this reference
> is still not original-runtime validated. Rust design/plan/start review and all
> access/privacy controls remain in force. Historical evidence is not promoted.

Date: 2026-10-16

**Status: written specification approved by the user on 2026-10-16.** The user
explicitly required original enemy intention chances despite different RNG
calculations. Subsequent plan/start approval authorized the Python implementation.
The supplied-entry slice is implemented as a **static-evidence-grounded Python
reference, not original-runtime validated**. Final controller verification of the
corrected source passed 735 tests, including 52 authored whole-combat acceptance
cases; whitespace checks passed and the fixed project RNG fixture is unchanged.
Independent task and whole-engine reviews completed. The whole-engine review's
single Minor checkpoint-immutability test/report overclaim was corrected using a
detached capture-time baseline, demonstrated by four alias-corruption detections,
and closed by scoped re-review with no outstanding findings.
The original-validation and external gates in section 10 remain unmet; this status
does not authorize original execution, Rust work, Git commit/push or cleanup.

## 1. Outcome and boundaries

Deliver one readable, independently authored Python combat reference: ordinary
Ironclad A0, one Cultist or ordinary Jaw Worm, base/upgraded Strike, Defend and Bash,
and Burning Blood. Support complete combat **from a supplied first ready player
decision through subsequent choices and terminal cleanup**.

The purpose remains a correct reference for later independent Rust work and
eventual policy training, not Python throughput optimization. Existing structural
state/snapshot/fault work and the standalone project RNG are foundations to
integrate, not tasks to repeat.

Exclude original dungeon/encounter startup, initial deck generation/shuffle/draw,
initial HP sampling, original input/button/animation loops, potions, additional
content, negative Strength, modifiers/retention exceptions, other Ascensions,
run progression, Rust, training and transport/adapter changes. No original-game
code, mechanical translation, binaries, assets or private evidence enters product
code or repository fixtures.

This supersedes the unimplemented behavioral details of the historical Python
combat plan, not completed structural work or accepted evidence. The older
[reference architecture](2026-10-14-python-combat-reference-design.md) remains
applicable except where this explicit boundary and
[RNG-001](../../intentional-deviations.md#rng-001--decorrelated-reproducible-project-randomness)
amend it. Inherited original-validation gates are unchanged.

## 2. Supplied entry state, not original startup

`initial_combat(scenario) -> CombatState` copies and validates an explicit,
content-constrained first-decision scenario. It does not run the original game's
precombat setup or secretly choose omitted values.

Scenario supplies:

- Player HP/max HP, current energy, ordinary per-turn energy reset amount and
  per-turn draw count. No implicit Ironclad HP, energy or hand-size defaults.
- Stable card instances with supported IDs and upgrade flags, ordered hand/draw
  piles and their declared card ownership. Initial discard/exhaust piles are empty.
  The supplied deck composition is experimental input, not a claim that a run
  generator produced it; this milestone does not validate acquisition history.
- One ordinary supported enemy with full initial HP, zero block, no powers and its
  published first intent: Cultist Incantation or Jaw Worm Chomp. Selected-move
  history contains that first selection exactly once. Enemy HP/max HP must agree
  and be in the reviewed A0 range: Cultist 48–54; ordinary Jaw Worm 40–44.
- A typed project RNG bundle with the existing contract identity and complete
  replay values. No root seed is required once streams have been derived.

Entry has player block zero, no player powers, no suspended work, turn 1 and a live
decision phase. It cannot smuggle in arbitrary enemy move histories, mid-turn
power flags, missing cards or an already terminal/faulted combat.

For a deliberately bounded numeric surface, supplied player max HP and ordinary
energy reset amount are positive integers at most 999; HP is positive and at most
max HP, and current energy equals the reset amount. Per-turn draw count is 1–10;
initial hand size is at most 10. These are **project input limits**, not asserted
original character defaults. The supplied deck has 1–999 cards, all owned exactly
once across initial hand/draw. Card and enemy identities are stable, unique and
independent of hidden seed values; bool is not accepted as an integer.

Validation checks all structural types, issued-card ownership, initial conditions,
content, ranges, stream identities and phase consistency before creating mutable
owned combat state. Failure leaves caller-owned scenario objects and streams
unchanged. The engine never guesses omitted state or treats an arbitrary dictionary
as reachable original gameplay.

## 3. Conditional chance fidelity is mandatory

RNG-001 changes the random generator and incidental cross-mechanic coupling,
**not the supported mechanics' probability laws**. Preserve the probability of
each next enemy selection conditional on the relevant gameplay state and selected
move history. Forced moves, repeat restrictions and fallback probabilities are
part of that law; matching long-run move frequencies is insufficient.

This claim is about the usual uniform-random-word probability model. It is not
equality conditioned on an original hidden seed, proof of PRNG independence or
global equality of all trajectory correlations. The intentional removal of original
cross-domain seed/stream coupling remains disclosed.

### Cultist

The initial selection is Incantation; subsequent selections are attacks with
probability one. Do not introduce random alternatives or consume AI words merely
to imitate original call counts. Supplied entry already includes the first intent.

### Ordinary Jaw Worm

The initial selection is Chomp with probability one. For later selections, sample
an integer uniformly in `[0, 100)` from this enemy's AI stream:

| Integer bucket | Ordinary selection | Restriction and fallback |
|---|---|---|
| 0–24 | Chomp | If last selection was Chomp, choose Bellow with probability `9/16`, otherwise Thrash. |
| 25–54 | Thrash | If the last two selections were Thrash, choose Chomp with probability `2994733/8388608`, otherwise Bellow. |
| 55–99 | Bellow | If last selection was Bellow, choose Chomp with probability `3489661/8388608`, otherwise Thrash. |

Only draw a fallback Bernoulli when that bucket's restriction applies. Use the
approved unbiased integer/rational sampling API, not Python random or float
comparisons. Selection appends history once; later intent publication does not
append it again. A newly selected move is for a later enemy turn, not an additional
move in the turn that scheduled the roll.

The fractions above preserve the finite chance law of the reviewed original
strict-less float sampler. In particular, nominal `0.357` and `0.416` must **not**
be substituted as `357/1000`, `416/1000`, convenient ratios or renormalized weights.
The existing project Bernoulli supports the exact denominators without implementing
the original RNG or float overloads.

The composed next-selection law, in Chomp/Thrash/Bellow order, is:

| Relevant history at selection time | Chomp | Thrash | Bellow |
|---|---:|---:|---:|
| First selection | 1 | 0 | 0 |
| Last Chomp | 0 | `131/320` | `189/320` |
| Last Bellow | `73349989/167772160` | `94422171/167772160` | 0 |
| Exactly one trailing Thrash | `1/4` | `3/10` | `9/20` |
| Two trailing Thrashes | `29955719/83886080` | 0 | `53930361/83886080` |

These are exact-rational consequences of accepted enemy and sampler facts, not
new original runtime observations. Implement the two-stage rule above; this table
is an independent acceptance oracle, not a license to change project consumption
by replacing the rule with a one-draw aggregate choice.

Other supported random mechanics retain their laws too: deck draws are without
replacement, and recycle uses the approved uniform project permutation. Supplying
enemy HP at entry does not claim to implement an HP sampler. Future random content
needs its own evidenced law and explicit domain assignment.

## 4. Owned state and responsibilities

Keep ordinary Python dataclasses/enums and small modules in `python/spire_lab/combat`:

| Module | Responsibility |
|---|---|
| `model.py` | Scenario/state/action/effect values, typed stream ownership, phase/power/AI flags, structural and semantic invariants, first-fault state. |
| `content.py` | Only supported content identities and independently worded static data. |
| `rng.py` | Existing approved standalone RNG; no algorithm/API rewrite. |
| `rules.py` | Small arithmetic and history-conditioned selection calculations. |
| `engine.py` | Input validation, ordered work, effect application, phase transitions and terminal handling. |
| `view.py` | Detached public projection and deterministic legal-action enumeration. |
| `snapshot.py` | Deep internal snapshots/restoration and detached divergence reports. |

No plugin architecture, alternate RNG backend, ECS, generic event bus or new wire
format. A plan may split an oversized implementation file by responsibility without
turning this slice into independent subsystem milestones.

State owns card identities, ordered piles and in-play card ownership until use
movement completes, player/enemy values, effective enemy
damage and published intent, selected history, typed powers with necessary lifecycle
flags, pending work/current continuation, RNG, public knowledge, terminal HP values
and first fault. A plain power amount is insufficient for Ritual's skip-first and
Vulnerable's lifecycle flag; these must be explicit snapshotted state.

Use the draw-list end as top. Recycling transfers shuffled discard storage order
into the draw-list end; later draws consume that sequence in reverse storage order.
Hand and discard order remain explicit, not sets. Card source membership changes
before discard/recycle animation completion in the reviewed original; the reference
must not defer gameplay membership to a render timer. Preserve reviewed ordinary
discard traversal and draw split ordering, not a new convenient sort.

## 5. RNG integration and replay ownership

Replace the provisional `model.RngState.slots` with a frozen combat bundle holding
`RNG_CONTRACT_ID`, one `combat.shuffle` stream and one `combat.enemy.ai` stream for
the sole supported enemy. Each is the existing immutable `RandomStreamState`:
stream identity, key and next-word index. Validate domains, scope arities, shared
combat scope and unambiguous enemy ownership. Do not alias cursors or derive one
stream by advancing another.

The `combat.enemy.hp` registry domain remains available in the standalone module,
but this supplied-entry engine neither samples HP nor needs an unused HP cursor.
Caller-side episode/partition/private scope allocation still follows the approved
[randomness design](2026-10-16-project-randomness-design.md). Public card/target
handles must not expose private derivation IDs, roots, keys or cursors.

An effect atomically installs each returned stream value as sampling proceeds.
Rejections advance only the owning stream. On exhaustion, preserve the exception's
latest cursor in the owned partial state and record a distinct RNG-exhaustion fault;
never wrap, reseed, resample from another stream or convert exhaustion into death.

Empty/singleton project shuffles and deterministic probability endpoints retain
the standalone contract's zero-word consumption. Original wrapper call counts are
not a requirement. The chance law, gameplay timing of shuffle/selection and exact
**project** replay are requirements.

## 6. Semantic API and resolution boundary

- `initial_combat(scenario)` returns a copied, validated ready state.
- `legal_actions(state)` returns an ordered sequence of `PlayCard` and `EndTurn`.
- `step(state, action)` mutates owned state and returns a `Transition` at the next
  live player decision, terminal cleanup or diagnostic fault.
- `public_view(state)` returns a detached public value, not internal state aliases.
- Snapshot/restore/compare remain internal debugging APIs, not policy features.

Attacks target the sole live enemy; Defend has no target. Check phase, exact action
type, card membership, supported identity, target and sufficient energy before any
mutation or sampling. Invalid actions raise the existing invalid-action error and
leave all state, queues, RNG and observations unchanged. Terminal/faulted/busy state
does not admit ordinary actions.

Terminal legal-action enumeration returns an empty sequence. Faulted-state
enumeration raises the existing faulted-state error; busy state cannot masquerade
as a ready decision. No normal action is a continuation/resume command.

Legal-action order follows public hand order, with EndTurn last; it is independent
of hidden draw order, RNG, evidence metadata and private identifiers. Do not execute
the engine speculatively to enumerate actions. At a live decision, all gameplay work
and continuations for the preceding action/turn are resolved and derived combat
values are current. An original enabled button is not evidence of quiescence.

## 7. Ordered behavior, not a frame simulator

Queued work is data-only, with explicit source/target, calculation timing and
payload ownership. Top insertion prepends; bottom insertion appends. Already
selected current work is not replaced by a top insertion. Reject unknown kinds
explicitly. Do not hide rule execution in callbacks or closures.

Presentation time can be omitted only while preserving its gameplay mutations,
hook order and decision/terminal observations. Construction/enqueue side effects
are distinct from action-resolution effects; logical quiescence is a project API
boundary, not claimed parity with every original intermediate animation frame.

### Player card use

| Card | Cost | Base payload | Upgraded payload |
|---|---:|---|---|
| Strike | 1 | 6 damage | 9 damage |
| Defend | 1 | 5 block | 8 block |
| Bash | 2 | 8 damage, then 2 Vulnerable | 10 damage, then 3 Vulnerable |

Calculate target-specific card damage before invoking its payload scheduling.
Bash's new Vulnerable is applied after its already calculated attack; existing
Vulnerable can affect that attack. The ordinary played card leaves hand, spends
energy and completes its reviewed use/discard sequence; it cannot be played twice
while waiting to reach discard. Preserve construction-time use hooks separately
from later movement. No free/autoplay/exhaust/retain variants are added.

### EndTurn and following decision

Resolve both evidenced internal player-end and enemy/round/next-player phases:
ordinary end-turn flags/hooks and discard; monster-start block reset/hooks;
execution of the previously published enemy move and its ordered effects; its
roll for a later move; monster end-turn and player/monster end-of-round hooks;
next-player setup, block reset, energy recharge, intent publication and draw.
The implementation must preserve exact accepted queue placement within those
phases, not treat this descriptive list as an interchangeable set of operations.

Jaw Worm Chomp carries 11 damage; Thrash carries 7 damage then 5 block; Bellow
applies 3 Strength before 6 block. Cultist Incantation applies Ritual 3; its attacks
carry base damage 6. Roll-move work follows the move payload.

Power notifications synchronously refresh persistent enemy damage and separately
refresh intent damage. An enemy damage action retains its damage entry; do not
assume every queued enemy attack is an immutable calculated snapshot.

In particular, constructing the next-turn draw immediately recharges energy,
runs recharge hooks and publishes intents. Pending end-round Ritual Strength can
resolve afterward, before draw, and refresh damage/intent again. Calls named
post-draw made after enqueueing are not evidence that draw already completed.
Expose only the fully resolved next decision, not that intermediate preview.

Preserve the hand cap of 10 and update-time draw/discard sizes. When a request
exceeds the existing draw pile, process existing-pile draw, recycle shuffle and
residual draw in that order. Do not shuffle remaining draw cards into discard or
draw with replacement. Per-card membership/hook order and ordinary end-turn
discard order follow the accepted deck/lifecycle facts.

### Powers and arithmetic

Use only needed NORMAL damage/block rules: Strength modifies NORMAL attack damage;
Vulnerable multiplies qualifying received damage by 1.5; clamp nonnegative and floor
at the evidenced pipeline stage. Block consumes ordinary damage before downstream
HP effects; accumulation is capped at 999. Do not apply integer truncation early.

Strength stacks and caps at 999 in the supported nonnegative slice. Ritual starts
with its skip-first flag, then adds Strength through its evidenced monster
end-of-round path, not every generically named end-turn hook. Positive Vulnerable
stacks on the existing object and expires by its reviewed end-round reduction;
reapplication must not recreate/reset its lifecycle flag.

With initial enemy Strength zero, no initial Ritual and no other modifiers,
Cultist's first attack after Incantation is 6; later ordinary growth is 3 per
growing round up to Strength 999, not unbounded damage. This is static composition,
not an original runtime capture.

The supplied numeric bounds and supported content keep reachable arithmetic in
safe integer/binary32 ranges. The plan must demonstrate that bound for every
implemented calculation and pin floor/cap tests. No generic Java-overflow/float
emulator is needed here; equally, an unbounded Python integer or default float is
not permission to silently support states outside the reviewed numeric surface.

## 8. Terminal and diagnostic contract

Stop admitting player choices when the player dies or the sole enemy is lethally
defeated. Distinguish local lethal damage, enemy death completion, victory callbacks
and final cleanup. Do not return a normal decision while terminal work remains.

Record `hp_before_healing` immediately before victory healing; Burning Blood heals
synchronously by 6, capped at max HP, for a living non-dying player in this slice.
It is not a queued heal that disappears when the action queue is cleared. On death,
do not invoke victory healing; both contracted HP observations are zero. Record
`hp_after_cleanup` at the completed terminal checkpoint.

Last-enemy lethality can prune queued work before battle-end clearing. Retained
action classes are not a promise that every such action executes. In this one-enemy,
starter-card slice there is no supported simultaneous player/enemy lethal source;
do not invent a general simultaneous-lethal precedence rule for excluded content.

Terminal cleanup clears gameplay pending/current work and continuations according
to the bounded contract. Do not invent a global terminal hand/pile/block reset.
Retain mechanically resolved diagnostic state; do not claim that its representation
or every presentation-sensitive terminal pile location matches original internals.
The original-validation terminal comparison is outcome and the two HP checkpoints,
with later adapter/non-interference gates still required.

Keep existing pre-mutation invalid-action rejection and first-fault preservation.
Unsupported mechanics, invariant failure, RNG exhaustion and guard exhaustion
retain partial state and block further normal actions. Faults are not losses and
do not imply automatic rollback. Recovery requires valid nonfaulted restoration
or new initialization; callers cannot clear a fault and continue.

Use one deterministic counter per external step for resolved/attempted effect
dispatch, including dispatches that fail, not queue inspection. At count 100,000,
stop with an effect-guard fault before applying that dispatch's payload; preserve
the counted failing dispatch and already applied work for diagnosis. Do not grant
extra work because it appears likely to terminate. Tests must pin 99,999 versus
100,000 and preserve a prior first fault.

## 9. Snapshot compatibility and visibility

Use snapshot schema **2** and supported content identity
`spire-lab-combat-reference-v2`. The opaque schema-1 RNG has no lossless migration:
reject old snapshots explicitly, rather than inventing keys/cursors. The standalone
`RNG_CONTRACT_ID` does not change.

Snapshots deep-copy complete internal state, including typed streams, power flags,
history, pending/current work, continuations, public knowledge and terminal/fault
diagnostics. Faulted snapshots remain useful for diagnosis but cannot be restored
to resume. Valid nonfaulted restoration validates compatibility and invariants and
returns detached ownership; it never silently discards pending work. Mid-resolution
continuations, where tested internally, resume through the resolver rather than
admitting a new player action.

Preserve detached, first-component divergence reporting and stable identity.
Explicitly migrate structural tests affected by the intentional schema/type change;
do not delete their ownership, fault or comparison assertions to obtain green tests.

Public view includes player HP/block/energy/powers, hand identities/upgrades,
observable discard/exhaust cards and pile counts, enemy HP/block/public powers and
published intent, turn, public reveals and contracted terminal values. It excludes
RNG, private scope/episode IDs, unrevealed draw order, future moves/history used
internally for AI, pending private work, fault internals and evidence metadata.
Previously public move information need not be erased from public history, but
internal AI fields are not new policy features. Public values and legal actions
must agree for identical public histories despite different hidden RNG/draw order.

## 10. Evidence status and acceptance

The private combat study established 46 bounded static facts, then a 20-record
lifecycle follow-up independently reviewed with three qualifications. Two local
initialization/guard links were subsequently bytecode-confirmed and independently
accepted; the progression wording was qualified using existing facts. The isolated
F30 observation remains quarantined, not retroactively repaired. Fresh accepted
notification/refresh findings are separate evidence.

Exact probability tables additionally derive from accepted enemy and sampler
facts. Their arithmetic was independently recomputed with exact fractions.
Source-derived rules, selected-bytecode confirmation, authored mathematical
consequences and original-runtime observations remain distinct. **No new original
combat runtime cases were captured in this study.**

Full original text, detailed ledgers and raw evidence remain private outside Git.
Local administrative reports bind exact claim/artifact/member/range identities;
this design contains only bounded independently worded behavior and requirements.
Historical source-attribution, capture-preflight and cleanup-robustness limits stay
disclosed. Any future capture runner needs the applicable prospective hardening/
authored controls; no successful static review retroactively fixes those issues.

The implementation plan must cover:

1. Exact AI conditional laws: enumerate all 100 buckets and every relevant history
   class with rational fallback weights; verify the composed table, forced moves,
   impossible repetitions, selection-history timing and project consumption.
   Frequency smoke tests cannot replace this check.
2. Reviewed card/power/block/damage/order examples, upgrade cases, Ritual first-skip,
   Vulnerable stacking/expiry, caps and damage-refresh versus intent timing.
3. Complete supplied-entry combats against each enemy, including multiple turns,
   draw/recycle, insufficient deck/hand cap, victory/Burning Blood and death.
4. Invalid-action atomicity, public-information noninterference, scenario/input
   ownership, snapshot-v2 compatibility/replay, domain isolation, exhaustion,
   first-fault recovery policy and exact resolution-guard boundary.
5. Regression of existing structural and standalone RNG acceptance, preserving the
   fixed public/project RNG fixture. No private evidence becomes a required fixture.

These tests establish authored implementation behavior. They do not grant a
validated label. Original validation still requires reviewed complete Cultist and
Jaw Worm captures at no fewer than three original seeds per enemy, required card/
power/shuffle/terminal interactions, both HP checkpoints and the existing Windows,
adapter/export/load/visibility/non-interference gates. Under RNG-001, use matched
semantic states and supplied random outcomes where feasible; shared seeds are not
an outcome oracle. Any new adapter work requires its own concrete plan.

Do not start Rust until the applicable reference behavior/evidence handoff is
reviewed. Python/Rust agreement and self-authored fixtures are not substitutes for
original-game validation.

## 11. Review and execution gates

The conversational design and this written specification are approved. Prepare a
concrete TDD implementation plan with exact APIs/tests, evidence bindings,
compatibility migrations and red/green verification commands. Execution requires
the applicable plan/start approval and preserves the authorized Sol implementation/
independent-review method. Written-spec approval does not approve a plan that has
not yet been reviewed.

No product edits, new dependencies, original runtime access, staging/commit/push,
cleanup or new subsystem are authorized by this document. Keep the checkpoint
commit and all preceding accepted/failed evidence intact.
