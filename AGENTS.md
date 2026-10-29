# Agent entrypoint

This is a concise route into the repository. Follow the active user request and
primary-session instructions for authorization. Repository instruction files
apply to their scope; dated profiles, plans, status snapshots, and archives keep
their evidence but do not override current instructions.

## Start with the assigned task

1. Confirm the actual checkout with pwd, git status --short --branch,
   git branch --show-current, and git rev-parse HEAD.
2. Edit only the checkout assigned for the task. A historic absolute path, branch,
   stash, or linked worktree does not establish current ownership. Preserve
   unrelated edits and existing evidence.
3. Read the assigned Bead and the accepted profile or plan it names. Use
   work-status.md for dated orientation, then verify live ownership in Beads and
   authorization in the primary session.
4. Check repository scope and the relevant public contract before changing
   behavior. The all-character A0–20 Acts 1–4 and Heart target is delivered through
   finite profiles; each accepted profile states its own limits.

If user input is materially needed, send the concrete question, evidence, and
options to the current root conversation. Do not route user questions through an
executor, child-agent tab, Learn queue, or another secondary conversation. Use
judgment for routine implementation choices and do not add approval steps the
user did not request.

## Find the right contract and source

| Need | Start here |
| --- | --- |
| Current product boundary and delivery path | [scope](docs/scope.md) and dated [work status](docs/work-status.md) |
| Accepted behavior and evidence | [milestone profiles](docs/milestones/) and each profile's linked plan |
| Frozen combat-v1 and M9/M10 Run contracts | [environment guide](docs/combat-environments.md), [M9](docs/milestones/m9-act3.md), [M10](docs/milestones/m10-heart.md) |
| Gym66 standalone combat and common policy view | [combat-training guide](docs/combat-training.md) |
| M11 finite Silent authored-combat boundary | [M11 profile](docs/milestones/m11-silent-combat.md) |
| M12 accepted finite Silent Act1 native progression | [M12 profile](docs/milestones/m12-silent-act1.md) and [approval record](docs/superpowers/plans/2026-10-26-m12-silent-act1-approval.md) |
| M13 accepted finite Silent Acts1–2 native progression | [M13 profile](docs/milestones/m13-silent-act2.md) and [coverage amendment](docs/superpowers/plans/2026-10-26-silent-normal-run-coverage-amendment.md) |
| M14 accepted finite Silent A0 Acts 1–3 native progression | [M14 profile](docs/milestones/m14-silent-act3.md) and [approved supplied-full-boss amendment](docs/superpowers/plans/2026-10-26-m14-supplied-full-boss-approval.md); canonical normal-start completion remains unproven |
| Mechanics research and source grades | [reference index](data/README.md) and its cited domain guides |
| Build, test, packaging, and tool versions | [development](docs/development.md), [toolchain](docs/toolchain.md), Cargo.toml, pyproject.toml, and lockfiles |
| Live task state and owner | Beads context, assigned task ID, and current primary-session instructions |

The simulator has one authoritative Rust engine. crates/spire-core owns game
state and rules; crates/spire-env owns native sessions, observations, actions,
replay, and batching; crates/spire-python is the optional PyO3 transport; and
python/spire_lab/gymnasium contains thin facades. The Python combat implementation
and differential tests are reference/comparison tooling, not a second production
simulator. data/ contains cited offline records, not runtime registries or proof
that a behavior is implemented.

Extend the shared Run and combat owners. Gym66 routes validated resources through
Run::from_validated_combat_resources, Run::enter_owned, enemies::setup,
Combat::enter_with_item_context, the shared resolver, and Run::settle_combat.
enemies::A0_ENCOUNTER_SETUPS owns shared setup metadata. spire-env/src/combat_policy.rs
reuses the versioned Run encoder; spire-python/src/run_env.rs owns shared scalar,
token, numeric-array, and spec packaging helpers. Extend the existing
character/profile and content seams for new profiles; do not copy Run progression,
a second encounter table, combat rules, or Python facade loops.
The current source owners are crates/spire-core/src/run/combat.rs,
crates/spire-core/src/combat/enemies/mod.rs,
crates/spire-core/src/combat/engine.rs, crates/spire-env/src/combat_policy.rs,
and crates/spire-python/src/run_env.rs.
See the [Gym66 guide](docs/combat-training.md) and [shared-engine guidance in the
public history archive](docs/handoffs/2026-10-26-agent-guidance-history.md).
Keep combat-v1 and existing M9/M10 profiles frozen; opt-in policy-v3 is documented
separately and does not silently change the M10 Heart or default profile.

A source acceptance is not proof that the matching Python extension was built or
loaded. The default Rust workspace does not enable the Python extension feature.
The accepted M11 profile is native-only and made no Python-delivery claim.
Reference records, source-grounded behavior, built artifacts, imported modules,
and original-runtime validation are distinct evidence.

Consumer source, manifests, artifacts and required ignored evidence must outlive
managed sessions. Before archiving, preserve them in independently retained
storage, verify their hashes, and coordinate consumer release. Paseo session
archival can remove managed worktrees; a path there is not a retention guarantee.
Prefer an accepted Git commit plus a versioned read-only source export with its
manifest stored alongside it.

## Choose the smallest meaningful check

| Change | Existing lane | What it establishes |
| --- | --- | --- |
| Rust core, environment, or CLI | cargo test --workspace --locked --offline, or the accepted profile's focused command | Native Rust behavior; not Python-extension delivery |
| Python/Gym facade | In an already provisioned Gym environment: `uv run --locked --no-sync python -B -m pytest tests/python/env tests/python/test_ironclad_training.py tests/python/test_combat_policy_view.py -q` | Facade and Gym66 session/policy-view behavior against the imported extension; establish source match first |
| Python combat reference / legacy comparator | `uv run --locked --no-sync python -B -m pytest tests/python/combat -q` | Separate compatibility lane; known legacy mismatches are baseline-reported, not Python-package delivery proof |
| Reference datasets/builders | Commands in data/README.md and its cited domain guide | Data identity, attribution, or deterministic generation; not mechanics acceptance |

Before relying on Python results, inspect `spire_lab._native.__file__` and
`spire_lab._native.build_info()` in the same environment. Record the imported path
and build context (Python environment, build command, Rust toolchain, Cargo
features, and revision/dirty state). `build_info()` reports build revision and
Rust version, not source hashes. Establish source match from the context and
receipt: hashes for Rust, binding, and Python inputs that affect the extension,
the native module or wheel SHA-256, relevant descriptor fingerprint(s), and exact
test results. A Git revision or descriptor fingerprint alone does not establish
source identity. A different Git revision does not invalidate a verified receipt
when the intervening change is documentation-only or the build used verified
dirty source bytes; matching HEAD alone does not prove a stale artifact current.
If source match cannot be established, report that limit and do not count the
run as Python proof. For an authorized binding task, use the isolated artifact
workflow in [combat training](docs/combat-training.md#shared-implementation-and-extension-build)
and preserve unrelated environments. Do not sync or rebuild solely because HEAD
advanced for documentation. Historical failure counts belong to their
revision-qualified status/profile evidence; do not force known losses green or
claim the full suite is green from a scoped run.

For documentation-only work, check changed links, paths, and commands. The
unstaged git diff --check omits untracked files. Before committing, stage only
the explicit task-owned paths, then run git diff --cached --check to include new
files. Never use blanket git add. Runtime suites do not validate prose.

## Milestone workflow and Beads

For each milestone, follow the current user-required sequence: brainstorm its
concrete design, write the plan, and obtain the required independent plan review.
Then honor explicit plan/start approval already given; do not ask again after a
passed gate unless the user changes scope or current instructions require it.
Use meaningful TDD and source-checkable facts for new behavior, then one fresh
independent final milestone review. Do not create per-task review swarms unless
the user asks. An accepted predecessor or open downstream Bead alone does not
authorize a milestone; check the current session and [work status](docs/work-status.md).

The primary user’s standing authorization permits source merge and dependency
release for already-approved finite profiles after one fresh independent senior
source-accuracy READY, resolved material technical findings, named source checks
and applicable meaningful native/preservation checks at explicit grades, with no
unexpected unresolved runtime failure. Preserve unmatched normal gates, failed
assertions, original losses/faults/missing endpoints and open milestone parents;
source acceptance is not full milestone completion. Do not add strategy tuning,
extra outcome runs or scope expansion. Material scope or new source limitations
outside existing authority return to the primary session; do not re-ask approval
for work already covered by these conditions.

After code is merged and required source/evidence retention, hash verification
and consumer release are complete, normally close our own completed tasks and
automatically archive agents/worktrees we spawned. Do not require another user
cleanup request. Preserve active/future work, independently retained evidence and
unrelated work; verify ownership and completion before lifecycle cleanup.

Use the existing repository Beads context and a distinct actor for a normal claim:

    bd context --json
    bd ready --label poc-gym --json
    bd show TASK_ID --json
    bd --actor UNIQUE_TASK_ACTOR update TASK_ID --claim --json
    bd --actor UNIQUE_TASK_ACTOR heartbeat TASK_ID
    bd --actor UNIQUE_TASK_ACTOR note TASK_ID "Progress and next step"

Use your own actor for progress notes and heartbeats. Do not initialize another
tracker, overwrite another live claim, force or reclaim an issue, or start an
overlapping controller. If Beads is unavailable, report the concrete blocker to
the primary session rather than creating a replacement tracker. Beads answers
task state and ownership; the user session answers authorization. If they
conflict, send the discrepancy to the primary session before the affected action.

## Historical records

The dated [work-status page](docs/work-status.md) is orientation, not a live
assignment. The former long startup guide is retained in the
[agent-guidance archive](docs/handoffs/2026-10-26-agent-guidance-history.md), and
the earlier README in the [README archive](docs/handoffs/2026-10-26-readme-history.md).
Those archives preserve public history, not current instructions. Private handoffs,
ignored evidence, and raw source material are not published here.
