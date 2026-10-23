# Gym delivery roadmap: narrow start through complete standard scope

Date: 2026-10-22
Status: user-approved roadmap on2026-10-22 ("I approve the roadmap"). The revised
execution/Beads migration plan and native single-owner method are also approved
("I approve, please do this"). Migration and scoped M1 are now accepted; M2
planning is ready. Execution/evidence/limits are recorded in that plan's ledger.

## Intent and scope amendment

The user requested a narrower, mechanics-first delivery path after slow progress
under the full-run-first breakdown. They approved an initial Ironclad A0 combat
milestone with damage, Block, Vulnerable and Weak through player and enemy actions,
then requested approximately ten milestones. They corrected the first draft:
Act1 belongs aroundM5, Act2 atM7, Act3 atM9 and Act4 atM10, followed by additional
milestones for the other characters. This revision follows that progression.
The user then requested the complete project's milestone sequence, including
everything afterM10. This version proposes36 outcome milestones in total.

The immediate objective is a usable Ironclad A0 gym, developed through all four
acts incrementally rather than postponing run integration untilM9. The
long-term objective remains the four standard characters, all normally reachable
content, Acts1–4/Heart and A0–20. That broader objective is no longer proposed as
an acceptance gate for every early deliverable. Completing these ten milestones
means the first character's declared A0 path reaches the Heart, not that all
characters/content/ascensions are complete. Milestones11–36 explicitly cover the
other characters, complete reachable content, A0–20 and final gym hardening.

Keep one authoritative Rust simulation and thin external-consumer interfaces.
Models/training remain outside the repo. Reuse existing accepted foundations and
verified mechanics. Do not rebuild the engine or discard unfinished work.

## Delivery principles

- A milestone ends with an executable demonstration, not only metadata/tests.
- Public caller interfaces, observations/actions, reset and replay evolve with
  the playable path; they are not all postponed until after the four acts.
- Work by shared mechanic and coherent integration, not complete card/relic/enemy
  families. Implement a mechanic once; player and enemy producers compose it.
- Ordinary differences such as amount, target, source, ordering and trigger time
  are explicit inputs/composition. Real exceptions remain named exceptions, not
  duplicated Weak/Vulnerable rules or a generic rules-language framework.
- Only M1 is immediately schedulable. Later milestones are queued outcomes, not
  simultaneous author assignments or separate upfront implementation plans.
- Milestones are outcome gates, not equal-duration tasks. Decompose the next
  milestone into coherent executable issues when it becomes actionable.
- Content is implemented alongside the character/act that needs it. Later
  coverage milestones close remaining cases and cross-system interactions; they
  are not permission to defer all card/item/event implementation until the end.
- Some milestones may already be largely implemented. Verify and reuse that work;
  create issues only for actual gaps, integration and demonstration.
- A narrow profile must identify its supported cards, enemies and room paths.
  Never silently present a smaller content pool as standard full-game coverage.
  Deterministic fixtures prove their stated path, not all normal runs.

## Proposed milestones

### M1 — Usable basic combat

Ironclad A0 combat can initialize, expose legal player actions, resolve damage and
Block, end turns and reach victory or defeat. Vulnerable and Weak operate through
both player and enemy producers, with correct damage rounding and turn timing.
Reuse the existing starter/selected-card and enemy implementations. A minimal
scripted caller and documented command exercise real fights; no fixed-HP-loss
substitute. This is a usable combat checkpoint, not another promise of a later API.

### M2 — Shared card and power mechanics needed for Act1

Extend the same combat path with the shared mechanics needed by the initial
Ironclad/Act1 roster: Strength, Dexterity, Frail, Artifact, damage types, draw,
discard/reshuffle, exhaust and explicit card selection. Establish consistent
application, stacking, prevention and timing using representative producers.
Existing implementations count; do not repeat the whole engine or port complete
character families before the next runnable milestone. Later exceptional mechanics
are added with the act/content that needs them.

### M3 — Persistent run: consecutive real fights

Connect run-owned state to actual combat entry and settlement. Two consecutive
fights preserve HP, permanent card identity/upgrades, gold, inventory and counters;
combat-local powers/cards clear correctly. Entry and settlement effects execute
once. Include actual relic/potion consumers and a loss path. This establishes the
run/combat bridge early, without depending on every game card or item.

### M4 — Playable Act1 room loop

Connect real Neow consequences, map choices, combat rewards, rest, shops,
treasure and event decisions for the declared initial roster. Offers, acquisition
and nested selections retain their actual consequences and ownership. The public
caller can progress across rooms without resource injection, fake menu effects,
automatic choices or fixed-outcome combats. Broaden mechanics as these real paths
require them, not through an upfront every-mechanic gate.

### M5 — Ironclad through Act1

Complete normal-start-to-Act1-boss progression for the declared IroncladA0 scope,
including real ordinary/elite fights, room consequences, boss reward and transition
to Act2. Exercise each Act1 boss across reproducible runs rather than supporting
only one fixed winning path. Public observations/actions and a runnable caller
cover this episode; loss remains a valid gameplay outcome. State content coverage
and remaining unsupported branches explicitly—act traversal alone is not proof
that every eligible card/event/item is complete.

### M6 — Act2 mechanics and content integration

Add the enemies, elite/boss mechanics, events and player/item interactions required
by Act2. Reuse the established application/trigger/card/run systems. Implement
new mechanics only where concrete Act2 content needs them, including multi-actor
interactions and persistent room consequences. Demonstrate real Act2 encounters
and room transitions before claiming the complete act path.

### M7 — Ironclad through Act2

Run from the normal start through Acts1–2, resolve actual Act2 bosses/rewards and
transition to Act3 with correct permanent resources. Exercise the Act2 boss
alternatives across reproducible runs, including non-winning outcomes. No fresh
Act2 fixture substitutes for the from-start path; standalone fixtures remain useful
for diagnosis. State actual content coverage and remaining gaps.

### M8 — Act3 mechanics and content integration

Add the Act3 encounters, bosses, events and required player/item interactions,
including distinctive counters, phases and interruption behavior. Reuse the same
resolver and run coordinator. Preserve prior acts while demonstrating actual Act3
fight/room behavior. There is no new character-wide implementation prerequisite.

### M9 — Ironclad through Act3

Run from the normal start through all three ordinary acts and their real bosses,
room consequences and rewards for IroncladA0. Exercise the Act3 boss alternatives
across reproducible runs. Expose correct Act3 completion/non-goal and death
outcomes without pretending Act3 completion is Heart victory. Seeded replay and
the thin gym interface must follow this extended path, not just the old combats.

### M10 — Ironclad through Act4 and the Heart

Complete real Ruby/Emerald/Sapphire acquisition in earlier acts, key-gated
progression, the Act4 room sequence, Shield/Spear and Heart. Demonstrate a genuine
normal-start IroncladA0 Heart path and distinct no-key/death outcomes. Finish the
corresponding public observations/actions, rewards/termination, reset/replay and
small native-batch agreement. No injected keys, near-Heart fixture or scripted
combat result substitutes for the complete episode. Document the exact supported
content and fidelity limits; do not claim all-character/A0–20 acceptance.

## Silent: milestones11–15, A0

Reuse the same acts, encounters, run coordinator, shared mechanics and interfaces.
Implement the character's ordinary card pool, exclusive relics/potions/events and
their interactions across this phase rather than proving only one winning deck.
The established engine is not reimplemented as a Silent simulator.

| Milestone | Outcome and executable evidence |
|---|---|
| M11 — Silent combat mechanics | Actual Poison, discard, Shiv, retain and relevant power/trigger/copy behavior through representative real cards/items. Normal Silent start and character-owned state work with the existing engine. |
| M12 — Silent through Act1 | From-start Act1 progression, relevant character content and real boss alternatives; valid loss/reward/transition behavior. |
| M13 — Silent through Act2 | From-start Acts1–2 progression and character/Act2 interactions; no late-act state injection. |
| M14 — Silent through Act3 | From-start Acts1–3 progression and boss interactions through the public caller. |
| M15 — Silent through Act4/Heart | Real key collection, Shield/Spear and Heart from normal start; character card/exclusive-item/event coverage atA0 is complete and listed limitations are resolved or explicitly prevent completion. |

## Defect: milestones16–20, A0

Reuse the shared act/run systems and extend only the genuine orb/card/item
mechanics. Preserve already implemented Lightning/Frost/Focus work.

| Milestone | Outcome and executable evidence |
|---|---|
| M16 — Defect combat mechanics | All four orb kinds, channel/passive/evoke/Focus, slot changes up to the real cap, power interactions and persistent card mutations. Barrage includes occupied counts above3 and sparse high-capacity layouts. |
| M17 — Defect through Act1 | Normal-start Act1 progression using actual character content and boss alternatives. |
| M18 — Defect through Act2 | From-start Acts1–2 progression and relevant encounter/card/item interactions. |
| M19 — Defect through Act3 | From-start Acts1–3 progression and actual boss interactions. |
| M20 — Defect through Act4/Heart | Real keys and complete Heart path; ordinary character card/exclusive-item/event coverage atA0 is complete, not only one successful orb build. |

## Watcher: milestones21–25, A0

Reuse the shared act/run systems and already implemented Calm/Wrath/Miracle work.
Do not treat the Watcher's individual card producers as separate stance engines.

| Milestone | Outcome and executable evidence |
|---|---|
| M21 — Watcher combat mechanics | Neutral/Calm/Wrath/Divinity, Mantra, Scry, retain, generated cards and relevant powers; correct stance-exit/resource/damage timing. |
| M22 — Watcher through Act1 | Normal-start Act1 progression using actual character content and boss alternatives. |
| M23 — Watcher through Act2 | From-start Acts1–2 progression and relevant encounter/card/item interactions. |
| M24 — Watcher through Act3 | From-start Acts1–3 progression and actual boss interactions. |
| M25 — Watcher through Act4/Heart | Real keys and complete Heart path; ordinary character card/exclusive-item/event coverage atA0 is complete, not only one successful stance deck. |

Ironclad card/exclusive-item/event coverage grows duringM1–10 and receives the
same A0 character-coverage requirement atM10. This does not require all off-color
cards before the first Ironclad milestone. Early declared profiles remain explicit;
full standard-profile completeness is established byM26–28, not inferred from a
single trajectory. None of these gates permit deleting unsupported definitions,
silently shrinking a standard pool or replacing real fights with scripts.

## Complete shared reachable content: milestones26–28

These close specific remaining cases across the implemented characters/acts.
Content needed for an earlier playable profile must land earlier; this phase
must not become a late dumping ground for ordinary gameplay systems.

| Milestone | Outcome and executable evidence |
|---|---|
| M26 — Shared/off-color card coverage | Complete ordinary colorless, curse, status, generated, transformed and copied-card paths; legal off-color/Note/Shard acquisition uses the same implementations. Complete eligible card producer pools without hidden cuts, including exceptional upgrades/misc/identity behavior. |
| M27 — Complete relic and potion lifecycles | Close remaining standard relic/potion effects, counters, acquisition/replacement/removal, source-specific use, healing/prevention/rescue, escape and interactions across fights and room boundaries. No inert registered items or duplicated inventories. |
| M28 — Complete events and economy | Close remaining reachable Neow/events/shrines/one-time branches, shops, rest/treasure/key interactions, generation/eligibility/history and nested selections across all characters/acts atA0. All standard A0 content pools are complete; no route is omitted because acceptance traces did not visit it. |

## Ascension progression: milestones29–33, all four characters

Each milestone implements the changed rules in its range, not a second simulator
or a blanket A0 fallback. Reuse already implemented starting-state/enemy profiles.
Tests cover each distinct below/at-threshold rule and integrated runs for all four
characters. Representative traces establish playable integration, while focused
cases establish the other changed branches; traces alone are not coverage proof.

| Milestone | Outcome and executable evidence |
|---|---|
| M29 — Ascension1–5 | Actual early ascension changes to encounters, damage and healing, with correct character/act interactions. All four characters can use the supported run API through these levels. |
| M30 — Ascension6–10 | Starting-HP, boss/other stat and Ascender's Bane behavior at the actual thresholds; full lifecycle of the curse, not just starting-deck membership. |
| M31 — Ascension11–15 | Potion capacity, reward/economy, max-HP and event changes at their actual thresholds, preserving all earlier progression. |
| M32 — Ascension16–19 | Higher-tier economy and normal/elite/boss AI/effect changes, with per-actor configuration, actual public intents and live payloads rather than HP-only changes. |
| M33 — Ascension20 | Correct two-distinct-Act3-boss sequence, settlements/rewards/key progression and from-start Heart paths for all four characters. All A0–20 branches remain supported; A20 is the primary final workload. |

## Complete gym contracts and delivery: milestones34–36

These harden and complete the interfaces used throughout the preceding
milestones; they are not the first time an external caller can use the gym.

| Milestone | Outcome and executable evidence |
|---|---|
| M34 — Full-run API/reset/replay contract | Complete public observations/action masks/capacities, selected reward ledger, exact termination distinctions, seeded reset, independent creation profiles/Note, checkpoints and semantic replay across all content/characters/ascensions. Hidden state stays private, retained outputs are owned, faults do not become gameplay losses or retry accepted effects. |
| M35 — Native batching and Python/Gymnasium hardening | Single/serial/worker/vector agreement for mixed characters/A0/A20, partial reset and terminal/final observations, delivery/export failure ownership and compatible thin spaces/bindings. Validate the actual rebuilt extension under separate authorized local-refresh permission, not a stale binary. No Python gameplay engine. |
| M36 — Verified final delivery | Reproducible from-start Heart acceptance, documented random/scripted rollout examples, bounded representative A20 throughput/memory/capacity checks, one integrated review and a qualified handoff. Resolve actual release blockers without a new verification platform or trained-agent benchmark. Keep source-grounded/reference-informed/original-runtime claims distinct. |

## Project completion boundary

Completion means the standard fully unlocked four-character A0–20 headless gym
through the Heart with all normally reachable content and the delivery contracts
above. Intermediate playable profiles may be narrower only when explicitly named;
they do not replace this final scope. Daily/custom/modded modes, graphics, career
progression and in-repo ML training are outside this selected project scope.
Original-game execution/fidelity promotion, commits, merges and publication still
require their separate authority; finishingM36 does not grant those permissions.

## Testing proportional to mechanics

Retain existing tests and their failure history. Do not delete regressions to
reduce a count. Change how new work is packaged and verified:

1. Small table-driven tests establish the shared rule and meaningful boundaries.
2. One or a few deterministic integrated scenarios demonstrate the milestone.
3. Ordinary cards/enemies receive cheap data/wiring cases; distinctive ordering,
   triggers or exceptions receive focused regressions.

The user explicitly requires tests that are neither brittle nor slow. Assert
semantic behavior and exact rule values, not incidental debug/log/JSON output,
internal call/allocation counts or unrelated hand/RNG details. Snapshot equality
is useful for nonmutation, not as a large stored golden. Use small valid fixtures
with headroom, seeded bounded scenarios and focused tables; no sleeps, network
dependencies or duplicated per-card rule tests. Do not weaken useful assertions
or discard regressions merely to reduce a count or runtime.

For M1, investigate a warm focused test command above5seconds or a warm Rust
workspace command above30seconds on the development machine. Record build time
separately from test execution and account for cold compilation/contention.
These are diagnostic performance targets, not flaky elapsed-time assertions or
hardware-independent correctness gates. Iterate on the affected target; capture
one real pre-change baseline and one integrated-candidate run, then recheck only
what later fixes affect. Preserve existing failures honestly.

No required test-count target, per-card review gate, private-source audit, parser,
symbolic executor, generic verifier or mutation farm. Tests should answer a product
question. Avoid repeated compile attempts against a known incomplete interface.
The last396/0 snapshot is historical evidence, not proof of current uncommitted
work or completion of any new milestone.

## Work ownership and parallel builds

Parallel builds are supported; a shared mutable source checkout was the problem.
Every subagent MUST have its own dedicated worktree and unique build directory,
including read-only reviewers and investigative agents. No subagent works in the
controller's or another worker's live checkout. Base each on a known coherent
source snapshot; separate artifact directories alone do not isolate changing
source. Reuse installed locked dependencies; no installs or service changes are
authorized by this design. Bound per-worker build jobs to avoid oversubscription.

Use one owner for each shared-engine/interface change. That owner delivers its
code, necessary integration and tests as one coherent candidate, rather than
repeated declaration-only requests. Truly independent work may run in parallel
against the stable interface; integrate sequentially and verify the combined
candidate. Source isolation does not eliminate semantic dependency or merge risk.

Exact workspace/source handoff must preserve existing uncommitted/untracked work.
During execution, materialize the reviewed public tracked edits and explicitly
listed untracked task files into each native-created worktree; HEAD alone does
not contain the current simulator. Do not copy private/unfamiliar material or
invent commits to transfer it. Each implementer delivers one coherent candidate
including its required integration, and the controller integrates sequentially.
No old active ownership is silently overlapped; collect safe handoffs first.
No worktrees are created in this plan-editing turn. Needed worker isolation is
part of the reviewed execution plan, not authority for commits, blind merges,
resets or worktree deletion; preserve worktrees until cleanup is separately allowed.

## Proposed Beads migration

Use the existing shared tracker only. After approval and the revised execution
plan, create/link the36 milestone outcomes in this progressive roadmap.
For M1, add only a small number of concrete issues for actual gaps and the runnable
demonstration. Decompose later milestones just in time; do not manufacture another
large speculative task forest.

Preserve accepted foundation issues and their acceptance qualifications. Existing
all-cards/items/enemies/full-run components remain historical work records and
long-term coverage umbrellas, not falsely completed issues. Move them out of the
immediate scheduling gate without deleting their scope, partial implementation,
claims or dependencies blindly. Replace blocking relationships only after mapping
the selected narrow capabilities to their real consumers.

Every executable issue must state: demonstrated behavior, explicit content scope,
required shared interfaces, a single integration owner, real prerequisites and a
small verification command/scenario. Status must distinguish active work, an
actual external/dependency blocker, queued work and a finished handoff. A worker's
normal release is not an indefinite scheduling decision.

## Approval boundary

The user approved this complete36-milestone roadmap on2026-10-22, then approved
the revised execution/Beads migration plan and native execution. Migration and
scoped M1 acceptance are complete; M2 planning is next, not automatic later code
authority. Roadmap approval is not permission for installs, commits,
publication, destructive cleanup or silently overlapping an active worker.
