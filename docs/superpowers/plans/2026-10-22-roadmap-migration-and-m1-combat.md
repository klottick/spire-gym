# Roadmap Migration and M1 Combat Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans (recommended here) or superpowers:subagent-driven-development to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Put the approved36-milestone roadmap on the existing Beads board and deliver M1: externally usable IroncladA0 combat with damage, Block, Weak and Vulnerable on both sides.

**Architecture:** Reuse the existing `Combat::enter`/`step_full`/`public_full` resolver and typed cards/enemies. Complete the shared native preflight for the four M1 cards, including Bash, then add the legal-action adapter and a small runnable example with public-API tests. One owner delivers the entire integrated change. Every subagent, including a reviewer, uses its own worktree and build directory—not the controller's live checkout.

**Tech Stack:** Existing Rust workspace, locked/offline Cargo dependencies and shared Beads CLI. No new library, Python extension rebuild or service setup.

**Spec:** `docs/superpowers/specs/2026-10-22-narrow-gym-milestones-design.md` (user-approved2026-10-22).

## Global Constraints

- Keep one authoritative Rust simulation and thin external-consumer interfaces.
- Models/training remain outside the repo. Reuse existing accepted foundations and verified mechanics. Do not rebuild the engine or discard unfinished work.
- A milestone ends with an executable demonstration, not only metadata/tests.
- Work by shared mechanic and coherent integration, not complete card/relic/enemy families. Implement a mechanic once; player and enemy producers compose it.
- Never silently present a smaller content pool as standard full-game coverage.
- Only M1 is immediately schedulable. Later milestones are queued outcomes, not simultaneous author assignments or separate upfront implementation plans.
- Preserve existing tests, historical failures and legacy combat-v1/schema/RNG behavior. No private-source audit, new verification platform or original-game execution.
- No installs/removals, dependency/lockfile changes, commits/staging/pushes/publication, destructive cleanup, new service/configuration or broad source rollback. Ordinary Beads writes retain local Dolt history.
- The controller's existing checkout contains authoritative uncommitted/untracked simulator work. Never create a worker worktree from HEAD and assume that work is present; materialize the reviewed current public-source baseline as described below.
- Every subagent gets a dedicated worktree and build directory. No worker or reviewer runs in another agent's live source checkout. Parallel builds require source isolation as well as artifact isolation.
- New tests must be deterministic, behavior-focused, bounded and fast. They must not freeze incidental implementation details or turn elapsed-time targets into flaky correctness assertions.

## Review Focus

1. An unaffordable/wrong-target action is absent from the menu and rejected without payment, damage or RNG advancement.
2. Menu/public reads do not consume randomness or expose hidden draw order/future AI; an unsupported card must not silently disappear into an apparently supported profile.
3. Raw damage, Weak/Vulnerable rounding and Block absorption are distinguished; a test must not mistake blocked damage for missing damage.
4. Terminal victory/death has no legal actions; rejected post-terminal work does not change the outcome or become an administrative loss.
5. Same authored setup/seed and accepted actions reproduce the same public checkpoints; a demo timeout is an explicit error, not a fabricated death.

---

## Scope and existing assets

M1 is combat-only, not a full run or full content pool. Use the four unupgraded
cards `Strike`, `Defend`, `Bash`, `Clothesline` with Ironclad/A0. Existing native
Jaw Worm, Green Louse and Gremlin Nob can supply damage, player Weak and player
Vulnerable. Fixture entry is explicit; it must not be advertised as a normal
from-start episode. Do not require new Poison/other-character/ascension work.

The earlier396/0 snapshot proves specific prior behavior only. Current old-lane
work includes an active card claim and unverified potion/enemy drafts. Collect
their safe handoffs, then capture an actual pre-change build/test baseline before
M1 product changes. Do not finish every old lane as an M1 prerequisite, discard
its files, or infer a baseline from the historical count. Record exact pre-existing
failures; if the baseline cannot execute a target, do not later call its unseen
failures pre-existing by assumption. A failure affecting M1 or regressing accepted
behavior must be resolved before acceptance.

## Test quality and runtime requirements

- Assert game behavior: legal actions, damage/Block arithmetic, power amounts at
  defined checkpoints, resource consumption and outcomes. Use public APIs for the
  integrated scenarios. Exact numeric assertions remain appropriate for exact
  game rules; replace incorrect fixtures, not those rules.
- Do not use whole-log/Debug/JSON golden files, incidental hand/menu order,
  allocation counts, function-call counts or internal RNG cursors as general
  acceptance criteria. Only test an internal/order detail when that detail is the
  actual contract under test. Snapshot equality is appropriate for proving that
  a rejected action or read did not mutate state; it is not a stored golden state.
- Use small, valid fixtures with enough HP/Block/resource headroom to distinguish
  effects. Compare raw damage, absorption and HP loss separately. Do not let a cap,
  premature kill or extra power hide a missing effect.
- Seed randomness and use short authored action sequences. Do not rely on chance
  to reach a needed AI branch; choose a deterministic valid fixture/history and
  ensure it does not reapply a debuff while its expiry is being tested. Replay
  asserts the deterministic contract, not a large frozen hidden-state transcript.
- Prefer small tables for shared rules and focused regressions for exceptions.
  No per-card copies of the same rule test, combinatorial fixture farms, sleeps,
  network access, original-game execution or expensive setup in the test hot path.
- All demonstration loops are bounded at64 accepted actions per scenario. An
  exhausted budget is an explicit error, not an outcome or an infinite retry.
- Performance targets on the development machine: the warm focused M1 test
  command should take at most5seconds, and the warm full Rust workspace test
  command at most30seconds. Record compilation and test execution separately;
  cold compilation and resource contention are reported, not mistaken for slow
  test logic. These are investigation targets, NOT wall-clock assertions inside
  tests or hardware-independent release guarantees.
- Run only the affected target while iterating. Run the broader suite once for
  the actual baseline and once for the integrated candidate, plus the targets
  affected by any later fix. Stop at a known compile blocker instead of repeating
  the same failed build. Investigate slow tests without deleting useful coverage
  or weakening semantics to meet a time target.

## Mandatory workspace isolation for subagents

The native single-owner implementer may use the controller's existing isolated
checkout. If any subagent is used—for implementation, investigation or review—it
must receive a dedicated worktree before dispatch. An assigned worktree is never
shared concurrently with another agent, even for a supposedly disjoint edit.

1. Finish the safe handoff and record the baseline in the controller workspace.
2. Create the agent's worktree with the harness-native worktree facility. The
   initial HEAD is only a starting point, not the current simulator baseline.
3. Materialize the current reviewed public-source snapshot there: tracked edits
   plus explicitly listed untracked Cargo/source/test/fixture/docs files needed
   for the task. Use ordinary copies/diffs and a small path list. Do not blindly
   copy unfamiliar/private files, original-game material, `.beads`, `.git`, build
   outputs or `.venv`, and do not create a commit just to transfer the source.
4. Confirm the listed parent/child source files match the intended baseline and
   give the worker that absolute worktree path as its working directory. Record
   the path, baseline and ownership in the ledger. Checksums are source-copy
   provenance only, not mechanics proof or a new receipt system.
5. Give it an absolute, unique `CARGO_TARGET_DIR` and a bounded build-job budget.
   Reuse installed locked/offline dependencies. Do not point it at the parent or
   another worker's artifact directory, and do not oversubscribe every CPU per
   worker. A reviewer gets its own snapshot of the complete M1 candidate.
6. The implementer owns code AND required integration in its worktree, including
   its scoped shared-engine changes. Return one complete candidate relative to
   that baseline, including new files and actual command results—not repeated
   declaration-only patches. The controller reviews/integrates it sequentially;
   never blindly merge/overwrite other uncommitted changes or claim child-only
   tests prove the integrated parent. Reviewers stay read-only in their worktrees.

No new worktrees are created during this plan-editing turn. Execution of the
reviewed plan establishes any needed agent worktrees; preserve them afterward
unless separate cleanup authority is given. M1 needs no parallel implementers.

## File and interface ownership

| Path | Responsibility |
|---|---|
| Existing shared tracker |36 milestone outcomes and one executable M1 issue; preserve old issues/history. |
| `.superpowers/sdd/2026-10-22-roadmap-migration-and-m1-combat/progress.md` | New plan ledger and actual new milestone/issue ID mapping; no second tracker. |
| `crates/spire-core/src/combat/engine.rs` | New full-mode legal-action adapter; any demonstrated M1 gap integrated by the same owner. |
| `crates/spire-core/src/combat/cards/ironclad.rs` | Native Bash payload through the same preparation/damage/power path as the other M1 cards; legacy combat-v1 handling remains unchanged. |
| `crates/spire-core/src/combat/{effects,hooks}.rs` | Scoped shared Weak/Vulnerable timing corrections if required by the explicit lifetime tests below. |
| `crates/spire-core/examples/m1_support/mod.rs` | Small authored fixture/runner helpers shared only by the M1 example and integration tests; no game rules or simulator. |
| `crates/spire-core/examples/m1_combat.rs` | Finite scripted public caller; prints public observations, menus, actions and actual outcomes. |
| `crates/spire-core/tests/m1_combat.rs` | External-consumer tests, importing the small example support module with a relative `#[path]`; no private combat-state access. |
| `docs/milestones/m1-combat.md` | Exact command, declared roster, results and limitations. |
| `AGENTS.md`, `docs/scope.md`, `docs/work-status.md`, `README.md` | Concise new workstream guidance and links; preserve qualified historical results. |

Do not pre-author a generic scenario framework. If a concrete M1 regression needs
a fix in `effects.rs`, `hooks.rs`, `turns.rs` or an owned card/enemy handler, the
same M1 owner makes that scoped fix with its regression and includes it in the
single candidate. No repeated controller/worker declaration-only patch exchange.

### Task 1: Preserve handoffs and migrate the active workstream

Execution status: complete; actual M1–36 IDs are `spire-dxo.23`–`.58`, executable
M1 child `.23.1`. Safe handoff/guarded migration and graph checks are in the ledger.

**Interfaces:** existing board `/home/klottick/repos/spire-lab`, database`spire`, project`a61e792f-9730-47c0-b262-7dac036d6bf5`; actor`opencode-session-recovery-20261021-v1`. Produces actual milestone IDs and the M1 implementation issue, recorded in the new ledger.

- [ ] Obtain a safe concrete handoff from the active original card actor. It releases its OWN claim normally; no force/reclaim/impersonation. Preserve source/tests/patches and record unverified status. Do not dispatch another broad family batch.
- [ ] Read `context --json`, `ping --json`, current `list --label poc-gym --status open,in_progress,blocked --json` and `ready --label poc-gym --json`; assert the database/project above before writes. Use only `/home/klottick/.local/bin/bd -C /home/klottick/repos/spire-lab ... --json`.
- [ ] Preview the migration as a plain Markdown mapping in the new ledger. Keep `spire-dxo` as the existing long-term parent and all accepted foundation issues closed. Keep unfinished old component issues and their internal dependency edges as traceable coverage umbrellas, not false completions.
- [ ] Create/reuse36 milestone children of `spire-dxo`, with the exact M1–M36 outcomes from the approved spec, label`roadmap-v2`, and metadata `roadmap_version=2026-10-22-v2`, `milestone_number=1..36`. New records use `--type milestone --status blocked --no-inherit-labels`, remain unassigned and do not carry `poc-gym` while staged. Discover existing records with `list --all --limit 0 --label roadmap-v2 --json`; verify their numbers, titles, scope and metadata before reusing them. Record every actual returned ID immediately. An interrupted run must reconcile existing records rather than blindly create duplicates or assume the preceding command did not commit.
- [ ] Add sequential delivery-gate dependencies M2→M1 through M36→M35 (each later milestone depends on the previous). These represent the selected delivery order, not a claim that every later character technically depends on the previous one.
- [ ] Move unfinished old components `.7/.8/.9/.10/.12`–`.22` out of the active `poc-gym` label only after their claims are released. Use `update --if-assignee '' --if-status <observed-status>` guards; if either changed, stop and reconcile rather than force/retry the stale write. Add `roadmap-v2-backlog`, retain all other labels/descriptions/history, and append the roadmap pointer once. No deletion, reparenting or blind dependency removal.
- [ ] Create/reuse one staged M1 implementation child, initially blocked/unassigned with `--no-inherit-labels`. Its own description MUST contain the demonstrated behavior, exact IroncladA0 four-card/enemy fixture scope, `enter`/`step_full`/`public_full`/new-menu interfaces, one integration owner, actual `.5/.6` prerequisites, excluded work, explicit acceptance tests/lifetimes, example command and baseline/verification requirements from Task2. Link this plan and the approved roadmap. A title/link alone is insufficient. Record the selected execution owner; that actual actor claims it normally before implementation.
- [ ] The M1 child depends only on accepted `.5/.6`, not entire card/item/enemy families or closing its own M1 parent. Link old components as provenance, not active blockers. Reconcile and verify all35 milestone sequencing edges and the M1 child edges, then run `bd dep cycles --json`. Until this validation succeeds, publish no new active `poc-gym` record. Never force through an unexpected edge/cycle or overwrite changed metadata.
- [ ] Publish the validated graph using guarded updates: change staged milestones to `open`/unassigned so their real prerequisite edges—not a permanently stored `blocked` status—govern readiness. Add `poc-gym` only to M1 and its implementation child and open the child. M2–36 stay outside the active filter and have their prior milestone as an active blocker; no future milestone is claimable early. Re-read `ready --label roadmap-v2 --json`, active-list state and actual dependency records to check this, plus36 unique numbers, one executable M1 child, unchanged accepted foundations and no overlapping old claim. Update guidance/docs to this workstream.

If migration stops before publication, staged records remain explicitly blocked,
not accidentally ready. If it stops during publication, the already validated
dependency graph still gates future work; resume from actual stored state and the
ID mapping. Do not treat a failed CLI response as proof that no write happened.

### Task 2: Deliver one coherent public M1 combat capability

Execution status: implemented and focused verification passed. See the ledger
for observed RED→GREEN evidence, actual baseline failures and scoped fixes.

**Consumes:** `Combat::enter(CombatEntry) -> Result<Combat, RunError>`, `Combat::step_full(&CombatAction) -> Result<(), RunError>`, `Combat::public_full() -> Result<CombatPublic, RunError>`, `Combat::snapshot() -> Snapshot` for detached equality checks, exported native setup/card/enemy/RNG types. Existing `Combat::restore` accepts legacyv2 only; M1 does not extend restoration. Reconstruct a fixture and replay its accepted action prefix when an independent equivalent combat is needed.

**Produces:** `Combat::legal_actions_full(&self) -> Result<Vec<CombatAction>, crate::run::RunError>` and the runnable example below. Existing legacy `legal_actions()` is unchanged.

- [ ] Before M1 source changes, run `cargo check -p spire-core --lib --locked --offline` and one `cargo test --workspace --locked --offline`. Record source/workspace identity, exact failed targets/test names or compiler errors, and elapsed build/test time. If an unrelated failure stops the workspace command before accepted consumer tests run, run `cargo test -p spire-env -p spire-python --locked --offline` directly once and record it separately. Fix only concrete M1 integration blockers; preserve unrelated work. A compile-only run is not an executed test baseline. Use this actual baseline for candidate comparison, never396/0 by recollection.
- [ ] Add public-API tests in `crates/spire-core/tests/m1_combat.rs` and minimal fixture support. Run `cargo test -p spire-core --test m1_combat --locked --offline`. Initially expect the missing public full-mode menu adapter or a concrete behavior failure, not an invented missing mechanic. Existing passing effects require no rewrite.
- [ ] First make the full-mode preparation path complete for the M1 roster. Currently `cards::prepare` reports Bash unsupported and `step_full` falls back to legacy-compatible execution; directly propagating that error would break the new menu. Add native Bash handling to the existing `ironclad::payload` function: cost2, base/upgraded damage8/10 then Vulnerable2/3, using the existing shared modified-damage helper and ordinary full-power application. Do not change the legacy combat-v1 path or add a second Bash calculation in the menu. Pin Bash's presence in the menu and base Bash under player Weak dealing6 rather than8.
- [ ] Implement `legal_actions_full` in `engine.rs` using that complete, pure `cards::prepare` path for all four M1 cards, enumerating visible hand instances and `None`/active stable targets in deterministic order. Skip `InvalidAction` candidates; propagate unsupported/fault conditions instead of silently shrinking support. Append `EndTurn` at a ready decision, return an empty menu at terminal, and reject non-full/busy states. M1 has no selection cards; selection-screen generalization is outside this task. Reads must not mutate state/RNG.
- [ ] Pin the following small public-API test set; fixtures use real native actions, not direct damage/HP assignments after entry:

| Test | Required assertions |
|---|---|
| `m1_public_menu_is_legal_and_read_only` | Four-card fixture, energy3: Bash2/Clothesline2/Strike1/Defend1 have correct targets. After Bash, unaffordable2-cost cards are absent; invalid/foreign targets and replay of a spent instance do not mutate a detached snapshot. Repeated menu/public reads leave snapshots equal; every returned candidate succeeds from an independently reconstructed fixture with the same accepted action prefix. Unsupported card setup yields an explicit menu error, not a silent smaller valid menu. |
| `m1_damage_block_and_debuffs_work_on_both_sides` | Small table: Bash8 then Strike6 on Vulnerable gives9; Jaw Chomp11 against Defend5 costs6HP; Clothesline12 appliesWeak so Chomp11 becomes8; Green Louse's actual SpitWeb makes Strike6 become4 and Bash8 become6; Nob's actual SkullBash appliesVulnerable so BullRush14 becomes21 before Block. Assert raw damage separately from absorption and no extra payment. Use valid histories and public-menu actions, not an impossible Nob history, an unaccounted Enrage Strength bonus or a compile error counted as gameplay RED. |
| `m1_debuff_lifetimes_follow_application_source` | Use the literal checkpoint table below for Weak and Vulnerable from both sides. Fixtures must not reapply the tested debuff during the expiry sequence; source/turn ordering is the contract, not incidental helper calls or unrelated AI draws. |
| `m1_victory_death_and_replay_are_real` | Finite scripted play reaches Victory in an actual fight; an EndTurn-only loss fixture reaches Death. Terminal menus are empty, post-terminal steps leave outcome/snapshot unchanged, and repeating the same seed/setup/actions matches every public checkpoint. Budget exhaustion returns an explicit demo error, never Death. |

Debuff lifetime checkpoints (base amounts; no intervening reapplication):

| Producer and recipient | After application / first affected decision | Subsequent full-round boundaries |
|---|---|---|
| Player Clothesline → enemy Weak2 | Amount2 immediately after the card; the upcoming enemy attack is weakened. | Amount1 after the first enemy phase/round end; removed after the second. |
| Player Bash → enemy Vulnerable2 | Amount2 immediately after Bash's damage; a following Strike is9. | Amount1 after the first enemy phase/round end; removed after the second. |
| Enemy Green Louse SpitWeb → player Weak2 | Amount2 at the first player decision after the enemy application, not prematurely1. Player Strike is4 and Bash is6. | Amount1 at the following player decision after another full round; removed after the next full round. |
| Enemy Nob SkullBash → player Vulnerable2 | Amount2 at the first player decision after the enemy application. A later base14 BullRush is21 before Block. | Amount1 after the following full round; removed after the next. |

Fix any demonstrated lifetime gap in the shared power application/turn-hook path,
not in card-specific expiry logic. Preserve legacy Vulnerable behavior and the
rest of the existing accepted combat contract.

- [ ] Implement `m1_combat` with standard-library argument parsing only. Interface: `--scenario all|victory|death|modifiers`, `--seed <u64>` (default7). Support helpers construct the explicit IroncladA0 roster, execute only current public-menu actions and report observations/actions/outcomes. A script may choose actions; the simulator must not acquire an automatic choice policy. Bound the script at64 accepted actions per scenario and return nonzero if unfinished.
- [ ] Run `cargo test -p spire-core --test m1_combat --locked --offline` and `cargo run -p spire-core --example m1_combat --locked --offline -- --scenario all --seed 7`. Expected: all M1 assertions pass, real Victory/Death and bidirectional modifier evidence print, no fault/timeout, no hidden RNG/draw-order output. If a mechanic fails, add/keep the focused RED and make the smallest authoritative shared-rule fix; do not special-case the example's card names in the engine.

### Task 3: Verify, document and accept M1—not the whole simulator

Execution status: complete. M1 `.23`/`.23.1` accepted/closed after one isolated
review with no findings and unchanged-source verification. M2 `.24`/planning-only
`.24.1` are ready, not permission to start unplanned M2 product edits. The eight
pre-existing draft failures remain explicit; no green full-workspace claim.

**Produces:** qualified milestone demonstration and actual task/milestone disposition, not a full-run release.

- [ ] Run `cargo test --workspace --locked --offline` once on the unchanged integrated M1 candidate, plus `git diff --check`, and compare with Task2's recorded actual pre-change baseline. If an unrelated failing target prevents accepted consumers from running, run their targets directly as at baseline. Report which targets actually executed; unobserved tests are not passes or proven pre-existing failures. M1/accepted-behavior regressions must be resolved before acceptance. Record warm-command timing against the diagnostic targets, without timing assertions inside tests. No historical396/0 promotion, stale extension check or silent test deletion.
- [ ] Write `docs/milestones/m1-combat.md` with the exact command/roster, representative literal outputs, public API usage and limitation: authored IroncladA0 combat scenarios, not a normal run or complete content pool. Link it from README/status docs; no new broad test harness.
- [ ] Perform one independent milestone-level review in the reviewer's OWN dedicated candidate worktree/build directory, against M1, the five Review Focus points and the test-quality/runtime requirements. Give it the complete candidate and existing command evidence; do not rerun the entire suite merely because the reviewer is a different agent. It may reproduce a concrete suspected defect. Fix substantive findings under the same implementation owner; no per-card review swarm or whole-project re-audit.
- [ ] Re-run the covering commands if reviewed/integrated bytes change. Then close the M1 implementation issue/M1 outcome only with actual evidence and normal claim ownership; leave old full-scope umbrellas incomplete. Activate M2's `poc-gym` label and create its bounded just-in-time planning work, rather than leaving the next outcome indefinitely hidden/blocked. Do not restart broad old content waves or start later product work beyond its reviewed plan.

## Execution and self-review

**Recommended:** native single-owner execution for this tightly coupled migration/API/example change, then one independent M1 review in its own worktree. If a subagent implementer is chosen instead, it also receives its own worktree. No parallel implementers are needed forM1. Later independent workers each receive separate coherent source/build workspaces; no worker starts from bare HEAD when the required simulator exists only in current uncommitted files.

Self-review: Task1 covers existing-board migration/preservation and truthful states;
Task2 covers M1 public usability and all five review risks; Task3 covers the
demonstration, regressions and qualified acceptance. M2–36 are intentionally only
outcomes from the approved roadmap. No old family-completion dependency is
introduced, no whole content pool is claimed, no source/receipt verifier is added,
and no commit/install/destructive step is part of this plan. Review corrections
are explicit: Bash's missing native preflight, literal debuff lifetimes, actual
pre-change regression evidence, a self-contained executable Bead, staged/idempotent
migration with stale-claim guards, fast non-brittle tests and per-subagent worktrees.

**Execution approval:** on2026-10-22 the user approved the revised plan and native
single-owner execution: “I approve, please do this”. Execute the tasks above;
retain one independent milestone review in its own materialized worktree.
