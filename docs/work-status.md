# Work orientation — 2026-10-23

Beads is authoritative; this is an administrative snapshot, not another task board.
See [scope](scope.md) for the current repo boundary and [environment usage](combat-environments.md)
for the implemented interface. No models/training/learner checkpoints belong here.

## Current workstream

2026-10-23: user approved the [M2 implementation plan](superpowers/plans/2026-10-23-m2-shared-combat-mechanics.md).
Planning `.24.1` closed; native actor claimed actual implementation `.24.2` under
M2 `.24`. Shared powers/card Block/HP-loss, public potion/selection ownership and
three Ironclad/two potion producers are integrated. See [M2 profile/commands/limits](milestones/m2-combat.md).
Public M1 5/0 + M2 12/0, shared8/0, consumers43/0 and other integrations21/0 pass.
Both examples run. Initial core candidate343/8 exposed an obsolete raw-Block-bound
assertion; corrected to approved wide-add semantics, final core344/7 has EXACT
same seven old draft failure names/messages. One isolated independent GPT-6 Luna
review found no findings; all201 reviewed public files unchanged and final public/
shared25/0+diffcheck verified. M2 `.24.2`/`.24` accepted/closed at declared scope,
not workspace-green/full-run/family/original-runtime acceptance. M3 `.25` activated
and actual `.25.1` created open/unassigned for planning only; no M3 product start.
Concurrent Luna earlier supplied bounded
read-only API notes in its OWN materialized worktree; no overlapping edits.
No installs/dependency changes/commits/cleanup/publication.

2026-10-22 migration: user approved the [36-milestone roadmap](superpowers/specs/2026-10-22-narrow-gym-milestones-design.md)
and [revised execution plan](superpowers/plans/2026-10-22-roadmap-migration-and-m1-combat.md)
(“I approve, please do this”). Milestones M1–36 are `spire-dxo.23`–`.58`, with
validated sequential delivery gates and no cycles. M1 `.23` and executable
`.23.1` were accepted/closed, followed by M2 `.24`/`.24.2` above. M3 `.25` and
planning-only `.25.1` are the active queue. Existing card author
safely released its own claim in a dedicated materialized worktree. Partial
Poison/potion/enemy drafts remain preserved and unaccepted. Old unfinished
components retain descriptions/history/internal edges under `roadmap-v2-backlog`.
Foundations `.4/.5/.6/.11` remain closed and unchanged. The
[M1 combat demonstration](milestones/m1-combat.md) and five public-API tests now
execute: real Victory/Death and bidirectional Weak/Vulnerable/Block, native Bash,
read-only full-mode menus and deterministic replay. At M1 acceptance, core335/8 matched the
actual pre-change executed baseline's eight unfinished Poison/enemy/potion draft
failures; accepted consumers43/0 and core integrations21/0 pass separately. No
workspace-green/full-run claim. One independent reviewer in its own materialized
worktree found no issues and approved scoped M1 acceptance. Controller verified
reviewed source unchanged, fresh external5/0 and clean diffcheck before normal
M1 closure/M2 planning activation. Historical396/0 is not the current baseline.

### Historical component-wave snapshot (superseded scheduling)

| Issue | Meaning |
|---|---|
| `spire-dxo` | Full-run gym PoC through Acts 1–4 and the Heart for external ML consumers |
| `spire-dxo.1` | Closed: user-approved written full-run design |
| `spire-dxo.3` | Closed: written plan approved (“lgtm”), start authorized (“You can start”) |
| `spire-dxo.5` | Closed: accepted typed registry foundation, not full playable content |
| `spire-dxo.4` | Closed: accepted native run-state foundation |
| `spire-dxo.11` | Closed: accepted native representative maps/world component |
| `spire-dxo.6` | Closed: accepted shared runtime foundation |
| `spire-dxo.7` / `.8` / `.9` | First executable handler/shared wiring joins, not accepted |
| `spire-vhh` | Known CPU-only readiness-test isolation bug, separate from gym behavior |
| `spire-12i` | Preserved-worktree/private-file disposition; still blocked, not product integration |

The PoC now has **20 component Beads, `spire-dxo.3`–`.22`**, not just a scope
placeholder. They cover plan/start, run state, content and mechanics, encounters,
run/combat settlement, world/economy/events/rest, keys/Heart, observations/actions,
rewards, replay, bindings, batching, end-to-end demonstration and final review.
Each records its deliverable, exclusions, acceptance checks and real prerequisites.
Registry `.5`, run foundation `.4`, native maps/world `.11` and shared runtime `.6`
are accepted; only cards `.7`, items `.8` and enemies `.9` are allocated. Other
implementation/acceptance components remain blocked/unassigned. The user selected all four standard fully
unlocked characters and all A0–20, A20-primary; no reduced profile or hidden
reachable-content exclusions.

The user approved the written design on 2026-10-22: “the spec looks good.” The
[written full-run design](superpowers/specs/2026-10-22-full-run-gym-poc-design.md)
is recorded under closed `.1`. The [approved implementation plan](superpowers/plans/2026-10-22-full-run-gym-poc.md)
under `.3` maps the existing components to concrete files, contracts, dependency
waves and conventional checks. The user corrected the unsupported inference that
Reaper could lose healing at combat end. The global frame/clock proposal is
withdrawn, not a combat implementation requirement. User-approved TIME-001 assumes
Portal's 800-second condition is satisfied, retaining other eligibility and random
selection; no clock or wait action. No timing question remains open. The written
plan is user-approved and start is authorized; 20 planned dependency additions are
applied with no cycles. The user consented to a fresh worktree, now created at
`/home/klottick/.local/share/opencode/worktree/503d4a/full-run-gym-poc-20261022`.
Ten current authored docs were carried over; main and historical worktrees remain
untouched. Its fresh offline Rust workspace baseline passed 169 tests, 0 failed,
before product edits. The first registry author was
`ses_e9c8254a8ffeBCqzV713lohbNr` (`openai/gpt-6-luna`, actor
`luna-poc-content-20261022-v1`); its component is now accepted as described below.
The registry foundation is now accepted/closed after independent review, one
bounded Note-domain/missing-starter fix and scoped independent re-review. Fresh
controller offline workspace verification passed 174 tests, 0 failed, on the same
reviewed bytes. Exact starts, producer/storage distinctions and explicit incomplete
pool errors are covered; full family metadata/mechanics are still required later.
The original NEEDS_CONTEXT report, failed tests and scope-boundary correction are
retained in the task ledger, not rewritten as success. No source audit/verifier or
private-source access was started. This is not full playable catalog/run acceptance.

The next run-state foundation `.4` was implemented by
`ses_e9c638df3ffeSjIT5xf2PJeHjJ` (`openai/gpt-6-luna`, actor
`luna-poc-run-20261022-v1`) and its own claim released pending independent review.
The reported A6 rounding concern was corrected with literal-oracle red/green;
author reports179/0 workspace tests before review. Independent review required
scoped RNG and live coordinator/continuation fixes; the same author returned the
fix, accepted after independent scoped re-review resolved all material gaps.
Fresh controller workspace tests passed189/0 on the unchanged reviewed candidate;
one Minor warning remains for shared hooks awaiting later gameplay owners/final
review. This accepts only the foundation, not playable full runs or real Neow effects.

Runtime `.6` remains partial. A bounded separate Sol recipe resolved its internal
card/pile identity migration: old public types, wide handles and unchanged native
consumers compile again over one owned runtime. Fresh controller workspace tests
passed223/0 before the next resume. The author then reported226/0 with multi-pick/
CombatExit added; the reported final blocker is lossless full fault/RunError
integration. Controller applied the join and the author corrected the observed
preflight/budget failures; fresh controller workspace229/0 passed. The first
independent component review found an Important gap: hooks record events without
dispatching behavior. A bounded separate Sol ruling also confirmed full enemy/
intent/power representation and per-actor RNG gaps. The original author is now
returned one fix wave on the same resolver, preserving legacy APIs; actual
family mechanics remain later owners. Fresh controller workspace236/0 and
diffcheck passed for fix1; scoped review addressed its originals but found a new
full-power refresh/validator mismatch. The author returned a narrow fix2 with actual
entry/ready-state regressions; fresh controller239/0 and diffcheck pass on the
candidate. Scoped re-review2 resolved the remaining finding without new breakage;
`.6` is accepted/closed as the shared runtime foundation. Historical missing test-first reds
remain explicit, not retroactively claimed. Actual family mechanics, run producers,
settlement and complete playable runs are not accepted by that foundation.

Next approved wave is three Luna authors in disjoint card/character, item and
all-Act encounter modules and their registry families. Initial returns were a
copy helper, metadata test and no-code context blocker—not family implementations.
Same authors resumed with smaller concrete native handler/fixture/wiring steps to
resolve the circular registration wait; full component scope remains required.
Shared root/state/effect/dispatch/export/test wiring remains controller-integrated
exact applicable patches, not prose-only requests. Card module/tests are now
registered and their prepare/commit integrated:6card/140combat tests passed, with
an earlier247workspace snapshot. Card author is now building real character
starter mechanics. Enemy native dispatch/setup joins and borrow correction now
pass4focused tests; combat144/2red includes active Neutralize/Survivor additions.
Enemy author continues real Louse formation/AI/hit-trigger work, not a completed
all-Act component. Revised item root wiring is applied, but its tests have not
run: active character/Louse additions currently fail compilation pending shared
type joins. Louse root joins are now applied; remaining compile errors are pending
character module/effect/state. The character patch is being corrected for snapshot
completeness, true player-end orb timing, Weak ownership and final rounding; enemy
owner has returned its selected-character/card-RNG adapter without passing tests.
The joined starter snapshot now passes a fresh locked offline workspace run:
276passed/0failed and diffcheck clean; actual cards14/enemies9/items10/bounds2
cases execute. This is not family acceptance. Original authors continue the next
common-card, starter-relic entry and Slime mechanics batches in disjoint files;
276/0 predates their next edits, not current concurrent-source verification.
Those next batches have now returned unexecuted, compile-blocked drafts. Card
owner is converting a prose-only request into applicable native insertion wiring;
The separate Sol ruling is delivered: stable parent tombstones plus appended
children, preserved actor-RNG/queued work, and lossless typed failed-entry combat
ownership. Item/enemy owners now implement those exact shared requests/tests;
The card author's applicable-patch attempt then failed integrity/context checks;
the bounded arbiter ruling now supplies exact overflow/fixture and unified-diff
repair instructions. Card owner is preparing that patch. Final lossless item
entry wiring is integrated but untested, its owner idle. Enemy owner is producing
a smaller typed intent/damage join before the still-required real split stage;
pending split must fault explicitly, never become valid no-op content. Those
card and typed Slime requests are now integrated. Corrected fixtures and native
late-startup-fault retention verify in a fresh controller workspace317/0 with
diffcheck clean (actual cards36/enemies19/items18 cases). Successful Split still
remains unsupported and required Stage B, not proven by that count. Original
The health/startup hook and Split data-only connector are now joined, allowing
tests to execute. Controller filters found cards37/2, enemies19/2 and items23/0.
After those behavior reds, the card owner corrected Quick Slash, Cloak's target
and full damage delegation; its focused suite reports39/0. The workspace attempt
stopped at core261/2 on the two Split tests. The real replacement body and shared
wiring have now been integrated: fresh controller enemy22/0 and workspace328/0,
with clean diffcheck, exercise actual Split and retained staged faults. Original
owners continue Ironclad attacks, potion fidelity and SlimeBoss mechanics;328/0
predates those next edits, not future concurrent verification. The next potion
test now executes and exposes Fire Potion's wrong CurlUp trigger (5Block vs0).
Boss shared wiring is now integrated but compile-blocked by pending THORNS.
Corrected numeric/THORNS patches are now integrated. Fresh focused tests execute:
cards44/1, enemies25/1, items24/1. Test-only card/item corrections now returned
with author focused45/0 and25/0; the boss correction then returned. Fresh controller
workspace verification now passes343/0 with clean diffcheck, including enemy26
cases. Original owners continue actual Defect/Frost, required Bark/health potions,
and Cultist/Act1Jaw ascension thresholds. First item return contained only an API/
registration request: just the IDs/registry are now joined. Its two new tests
now execute after the Frost diagnostic join. Fresh controller card tests pass54/0;
items25/2 initially failed on missing Bark/health behavior, and the ascension test
first needed a corrected override-map expectation. Both implementations and shared
joins are now verified in fresh controller workspace367/0 with clean diffcheck
(current cards54/enemies30/items34). This includes real Cultist/Jaw A0–20/live A20
and Bark/Blood/Fruit behavior, not all enemy ascensions or full healing modifiers.
Original owners continue Watcher commons, Bloom healing suppression and Acts2/3
formations. A redundant encounter-design approval stop was resolved under the
existing approved scope. Bloom's ID-only join now lets tests run: Watcher0/5 and
items34/3 exposed missing handlers/healing suppression. All three authors have now
returned. Fresh controller workspace verification passes381/0 with clean diffcheck,
including Watcher stance-exit/AoE cards, owned Bloom suppression on current heal
paths, and real Acts2/3 Cultist/Jaw formations across A0–20. The enemy author applied
its four-file shared request contrary to the controller-only boundary; its incorrect
controller attribution is corrected in the report, and controller inspection and
verification assess the current bytes without reverting or reapplying them. All
three original claims were released. User then explicitly requested resumption;
the controller reopened/allocated the three tasks and dispatched their original
sessions for Defect common attack follow-ups, Magic Flower/Toy Ornithopter and
Gremlin Nob A0–20. Mechanics families remain incomplete/unaccepted and downstream
components stay blocked.381/0 predates this resumed work.
Item Flower/Toy bodies have returned and the exact shared Heal delegation is now
joined. Static inspection found wrong test arithmetic and insufficient HP headroom;
the original item owner returned test-only corrections and released its claim
while card/enemy work continues. The Defect four-file shared request is now joined:
fresh live focused cards67/0 and items42/0 execute. Inspection found an untested
potential one-orb Barrage extra hit; the original card owner is adding runtime
regressions and a bounded correction while the enemy owner continues. No new
whole-workspace result was claimed at that point. The owner reproduced and fixed
the real one-orb Barrage extra hit; the controller joined its strict continuation
validation and the Nob hook/setup patch. A Nob test-only correction accounts for
Block10 without changing raw damage14. After all returns, fresh workspace396/0
and clean diffcheck include cards68/enemies37/items42. The original three sessions
are already resumed on Silent Poison/Artifact, six finite combat potions and Blue
Slaver/Two Fungi Beasts A0–20;396/0 predates their new edits. No mechanics family
acceptance or downstream release. The enemy owner has since returned tests-only,
compile-blocked by the item owner's active potion match migration; it is idle
until the concrete item handoff enables an actual enemy behavior RED. Card/item
authors remained active at that point. The item session recovered from a transport
error and has now returned the six potion bodies and tests; consumer corrections
are unverified. Its latest compile blocker is the card owner's pending ApplyPoison
join, not the old potion match error. Item/enemy owners are idle pending that
concrete shared return; only the card owner is active.396/0 remains the last
verified snapshot. Targeted public
numeric references for those two enemies
are reference-informed, not original-source/runtime proof. No wider review/audit
or family acceptance is implied. No
reduced profiles/pools or fake gameplay; bridge `.10` stays blocked on accepted
completion of all three.239/0 predates this wave, not current in-progress results.

Native representative maps/world `.11` is accepted/closed after independent review
and3scoped fix/reviews. Final privacy, eligibility/no-draw, external generator and
sanitized formatting cases pass; fresh controller external2/run30/workspace223 on
unchanged reviewed files. Minor native-consumer warnings remain for final review.
This accepts the native map/world component, NOT actual room effects, encounter
constructors/AI, Neow, keys/Act4/full-run gameplay or original-runtime fidelity.
Those remain named downstream obligations; no other component is newly released.
223/0 is pre-resume evidence, not a result for future in-progress edits.

Python/device tests and extension installation have not run in this worktree.
Main/historical worktrees remain untouched; no commits/pushes.

Verification belongs to those delivery tasks: existing tests, small deterministic
fixtures, replay and focused regressions. No custom parser/verifier/semantic-law
language, receipt pipeline or mutation farm is in scope. One concise design and
one reviewed implementation plan govern the work; component/final reviews do not
mean a new approval cycle for every card or assertion. The selected reward policy
in closed `spire-8jm.9.1` is reused, not reopened as an unresolved choice.

Use the `poc-gym` label for the primary queue. Broader unfinished specification
work has `backlog-full-scope`; paused verifier work additionally has
`deferred-verification`. Only actual ongoing work belongs in progress. Deferred
items retain owners, returned review outcomes, exact evidence paths and residuals.
Deferral does not close requirements, accept candidates or remove dependency edges.

## Historical lanes, not current assignments

All 126 unfinished `spire-8jm` full-spec/verifier issues are explicitly deferred,
not left open/blocked as if they were active work. The 15 legacy reference/upstream
tasks and old runtime-isolation investigation are also deferred. These 142 records
carry `historical-backlog`; none is a pending dispatch or mandatory PoC gate.
Original blocker relationships and evidence remain intact for later scoped reuse.

`spire-8jm` retains the broad requirements; `spire-8jm.6.7.1` retains the unfinished
custom verifier. Old Act 1-only proposals, external correction/PR tasks, worker
selections and source-proof queues in those records are historical, not current
Heart-gym instructions. No historical record was deleted or falsely completed.
The only non-PoC current follow-ups are `spire-vhh` (`maintenance-gym`) and
`spire-12i` (`preservation-gate`). Beads history is preserved separately from the
actionable queue; closed records are not new work artifacts.

The selected target is full runs through the Heart for all four fully unlocked
characters at A0–20, with no separate Act 1 release gate. Character/ascension
breadth is no longer an open decision. Historical slices remain usable at their
existing grade; planning now identifies missing full-run behavior and defines
shared interfaces/acceptance, not another scope reduction or research campaign.

## Quick-win integration audit

Fresh read-only checks before this documentation edit established:

- All **41 local branch tips** are ancestors of `main` at `c0746b7`.
- GitHub reports **no open PRs** in `klottick/spire-gym`.
- All **126 public product/manifest files** compared in the preserved `phase-1a`
  working checkout are byte-identical to `main`, including its untracked copies
  of already integrated environment files. No additional product paths were found.
- The native combat/batching/Gymnasium integration is already landed. Do not
  re-merge the stale checkout, replay its old plans or duplicate its implementation.

These are integration/identity findings, not a new test run, fidelity proof or
permission to remove branches/worktrees. Private/historical notes and uncommitted
material remain preserved. The six linked checkouts and existing untracked files
were not deleted; one unexpected historical checkout was deliberately not opened.

## Close-to-ready versus accepted

Private source tranches are not product merge candidates. The retained bounded
Watcher review reported YES, but fresh controller acceptance is outstanding and
the broader Watcher-card task remains incomplete. Ironclad/Defect helper and
selector-validator review gaps remain unresolved. Accepted partial source facts
can be reused at their exact grade when needed; none of these broad tasks was
marked complete to make the board look clean.

There is no outstanding local branch or open PR to merge as a shortcut. The useful
next work is execution-workspace setup and the first ready registry/foundation
component, not another historical integration or general
verifier cycle. Current planning changes documentation/tracker state only:
no product or dependency changes, new runtime tests, commits, pushes or merges.
