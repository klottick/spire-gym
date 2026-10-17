# Linux capture readiness for the headless combat reference

> **Deferred after approved sequencing change (2026-10-16):** The user approved
> [source-first Rust development](../../scope.md#approved-source-first-sequencing-amendment-2026-10-16).
> This approved specification is retained, but its helper/feasibility work is no
> longer the next milestone or a prerequisite for Rust. Original comparisons are
> deferred to later fidelity promotion. Requirements below govern capture work
> only if resumed under separate plan/access approvals; no capture work has begun.

Date: 2026-10-16

**Status: written specification approved by the user on 2026-10-16, with the
repository-wide scope reflected in the main documentation.** The user approved Linux
capture readiness first, then approved the separation between capture requests,
execution, result records and combat coverage tracking. This document develops
that agreed design; it does not authorize implementation or original execution.
Following the user's scope review, this specification explicitly subordinates validation
tooling to the optimized Rust simulator and measurable AI improvement. It is not
an implementation plan or a separate capture-platform project.
Preserve the selected Sol implementer/fresh-independent-review method for the
later implementation plan. No staging, commits, installations or cleanup follow
from writing or approving this specification.

## 1. Intent and success criteria

The [current repository scope](../../scope.md) and README describe the same goal
hierarchy, non-goals and present implementation status. Approval permits the next
planning step, subject to the concrete-consumer/feasibility guard in section 2;
it does not approve a nonexistent implementation plan or original execution.

The goal hierarchy remains the approved
[ML-first repository design](2026-10-11-spire-lab-ml-first-design.md):

1. **End goal: an AI that plays Slay the Spire and improves**, ultimately toward
   the approved full-run/Heart progression. Measure held-out playing strength
   gained per wall-clock hour, not merely implemented content or simulation speed.
2. **Enabling product: an optimized, game-correct Rust StS simulator**, with native
   batched environments and thin Python bindings for Python learning/evaluation.
   Profile and benchmark real workloads; Rust throughput supports learning rather
   than replacing its success criteria.
3. **Supporting work only: the readable Python reference and original-game
   validation.** These reduce the risk of training against wrong mechanics; they
   are not a second production simulator, a Python performance project or the end
   product. This Linux increment is only a prerequisite for that validation.

Spire Lab is a **headless simulator for automated experiments and ML training**,
not a human-playable game. Do not build rendering, animation, audio, an interactive
game UI or a playable client. Combat actions are programmatic inputs; decision
boundaries are state transitions, not prompts to a human. Simulation must not
require the original game, a display server or capture tooling to run.

The Python supplied-entry combat slice is implemented and independently reviewed,
but is not original-runtime validated. This milestone prepares trustworthy Linux
capture infrastructure and states exactly which original evidence remains missing.
It must not substitute more self-consistency tests for that missing evidence.

The delivery path stays bounded: validate the narrow supported combat reference,
independently implement the corresponding Rust slice, add the native batching/
Python training boundary, measure it in release builds, and run the initial
learner against random/heuristic baselines on held-out scenarios. Expand combat
content and later run capabilities through the existing phased roadmap. Do not
require a full-game simulator before reaching the first narrow learning loop.
Existing evidence/approval gates remain intact; this statement grants no Rust
execution or validated-learning label prematurely.

Success means:

1. A small authored capture helper passes real, synthetic-only Linux process tests
   and targeted injected-failure controls, including independent cleanup attempts.
2. Requests, executable/input identities, output retention and failures have
   explicit, inspectable semantics. Failure evidence is retained, not overwritten.
3. A prospective matrix covers the required original Cultist/Jaw Worm observations
   without claiming they have been collected or that Windows gates are satisfied.
4. Existing combat/RNG tests remain green and standalone RNG bytes stay unchanged.

Passing synthetic helper tests is **not** progress evidence for agent strength,
Rust performance or original-game correctness. Report it only as a supporting
readiness result, with the actual remaining path to original combat evidence.

Actual Linux execution of the original game is a later scoped decision. Headless
simulation does **not** establish that the original game's runtime can execute
without graphics, assets, a loader or other initialization. Do not assume it can.

## 2. Scope and alternatives

Selected approach: author a small repository-owned process-capture helper, test it
with authored children, and document prospective combat coverage. This replaces no
historical artifact and performs no original capture during this increment.

Alternatives considered:

- Patch the old private one-off runner: less immediate code, but couples future
  tests to private scripts and risks confusing prospective fixes with old evidence.
- Build a full original-runtime combat adapter now: closer to the ultimate evidence
  goal, but depends on unestablished runtime, setup and observation assumptions.

Keep responsibilities separate:

| Component | Role | Proposed location |
|---|---|---|
| Capture helper | One checked child invocation and its owned lifecycle/result | `validation/capture_runner.py` |
| Authored controls | Real synthetic children plus narrow failure injection | `tests/python/test_capture_runner.py` |
| Readiness/evidence guide | Current readiness, control coverage, prospective original matrix | `docs/validation/linux-readiness.md` |

Use Python 3.12's standard library and existing pytest; no new dependencies, JVM
installation, Steam installation or external service. These are validation tools,
not code imported by `python/spire_lab/combat/` or the training hot path. No generic
job scheduler, shell-command CLI, trace parser, fixture loader or plugin system.

Compose the applicable checked-file operations from `validation/evidence_report.py`
where suitable. Do not expand `StaticAudit`'s static-only command grammar to execute
the game; do not retrofit this helper into historical studies. Existing helpers
may be read as authored code, not treated as proof of the new runner's behavior.

### Scope traceability and stopping rules

| Proposed work | Necessary contribution to the repository goal |
|---|---|
| One-invocation capture/cleanup helper | Prevent failed or contaminated captures from being accepted as mechanics evidence for Rust |
| Authored exceptional-path controls | Exercise the specific runner failure modes already found, without risking original/private evidence |
| Small combat coverage matrix | Identify the exact missing comparisons needed for the supported reference/Rust handoff |

Before helper implementation is planned, identify its **next concrete StS
validation consumer**: the proposed supported case, intended invocation boundary,
required observations, missing runtime/adapter facts and approvals. A hypothetical
ability to run arbitrary commands is not enough justification. This is a planning
description, not permission to inspect or execute new original inputs. If a usable
capture path cannot yet be described, surface the feasibility blocker and propose
a bounded investigation instead of building an unused generic runner.

Retain the required privacy, process cleanup and truthful failure safeguards, but:

- Implement only the one-invocation path and controls needed for that consumer.
  No general capture framework, scheduler, storage service, generic schema tooling,
  additional platform support or extensibility work for hypothetical consumers.
- Stop readiness development when its explicit controls pass and the remaining
  original-validation handoff/blockers are documented. The next proposed work must
  advance actual supported-combat evidence, not another layer of helper polishing.
- Do not add new correctness or promotion gates beyond the approved requirements
  without a concrete defect or a user-reviewed scope change. Conversely, do not
  waive existing original evidence/Windows gates to accelerate a status label.
- Keep capture, hashing and reporting outside simulator execution and training.
  This milestone changes neither the Rust architecture nor its performance budget.

The implementation plan must show the consumer and these stopping rules before
breaking helper work into tasks. Completion is not conditioned on making this
tool generally useful beyond the immediate StS validation need.

## 3. Request and trust boundary

A request is a detached, immutable value describing exactly one invocation:

- A descriptive case ID, exact nonempty argv tuple, explicit absolute executable,
  explicit approved working/output root and a fresh invocation-directory name.
- Expected SHA-256 identities for the executable and every explicitly supplied
  authored script, local helper and input needed by that command.
- Mandatory positive finite execution timeout, cleanup grace and observed output
  size limit. Proposed project bounds: execution at most 300 seconds, cleanup grace
  at most 5 seconds, and observed limit at most 16 MiB **per stream**. No defaults
  may silently replace missing limits. Time values are seconds; the byte limit is
  an integer. Bool-as-number and nonfinite values fail.
- The exact approved command tuple must be supplied separately as an allowlist
  entry; a differing argument, executable or order is rejected before launch.

The later implementation plan fixes Python types/signatures and concrete synthetic
argv values. The allowlist is a caller-supplied operational constraint, not evidence
of user authorization and not a way to recognize arbitrary commands as safe.
This milestone constructs only authored synthetic requests; no game commands,
private game inputs, desktop actions or runtime-adapter requests are implemented.

No shell, command-string interpolation, PATH lookup, implicit working-directory
selection, automatic dependency discovery, executable downloading or retries.
Paths must be lexical absolute paths without `..`; reject symlink components.
The invocation-directory name is one nonempty path component, not an absolute path,
`.` or `..`; reject separators and NULs. Stream/record names are helper-owned,
distinct reserved names, never caller-selected paths that could alias inputs.
The caller must explicitly supply the real executable path if its usual launcher
is a symlink. Tool files are regular pinned inputs, not files whose permissions
the helper may repair. Private output containers must be owned by the current
user with mode 0700; output leaves use 0600 and reject unexpected hard links.

Use an explicit minimal child environment, not ambient environment inheritance:
fixed C locale and a fresh invocation-local temporary directory; no inherited
Python/JVM injection, dynamic-loader injection, display/desktop variables, secrets
or user startup configuration. Synthetic interpreter commands use isolated mode.
Close child stdin, avoid inherited unrelated descriptors and create a new session
so the invocation owns a process group. Child-created files inherit umask 0077.

This is **not a sandbox**. Only trusted, reviewed children are in scope. There is
no claim of hostile-code containment, complete dependency attestation, prevention
of network access, hard memory/disk quotas or protection against concurrent hostile
filesystem mutation. Pinned declared files do not enumerate a whole interpreter,
standard library, native-loader or OS dependency graph. Do not overstate them.

## 4. Capture lifecycle and output ownership

The lifecycle is single-use and has explicit phases:

1. Validate the request/allowlist and opening identities before launching anything
   or creating the invocation directory. Input/identity failure cannot run a child.
2. Exclusively create the fresh private invocation directory and fixed stdout,
   stderr and temporary locations. Never reopen an existing attempt for writing.
3. Recheck declared inputs immediately before launch. Launch the exact executable
   with stdout/stderr directed to already opened private files, not tool/model
   stdout, memory buffers or a terminal. Capture bytes without decoding them.
4. Observe child progress against a monotonic execution deadline and both file-size
   thresholds. Check at intervals no longer than 50 ms during normal observation.
   Timeout or threshold crossing is an operational failure and initiates cleanup.
5. On every post-launch exit path, perform the independent cleanup sequence in
   section 5. An exited direct child does not imply its descendants have exited.
6. Where safe, close/verify/hash retained streams, check closing declared identities
   and retain a finite result record. No raw output is returned or printed.

The size threshold is an **observed stop threshold**, not a hard byte quota:
direct-file output can overshoot between observations and during termination.
Preserve the observed bytes and failure; do not truncate evidence to manufacture
compliance. If writers or file identity cannot be accounted for, mark the artifact
unsettled/unverified rather than claiming a final digest. A later original-runtime
scope must assess expected output and these resource limits before approval.

Record the exact request, expected/actual identities, phase events and process
observations. Bind the runner source and explicitly used local authored helpers
as well as executable/input files before its control campaign and before any
later original use. Recheck those identities at closure. Source-on-disk checks are
provenance observations, not proof of in-memory loaded code or OS attestation.

All failures preserve their fresh directory and any partial files. No unlink,
recursive deletion, chmod repair, overwrite, automatic retry, historical evidence
discovery or search for the previously unavailable shell log. Creating a new
owned directory/file with its required mode is not permission repair.

## 5. Failure and cleanup semantics

Keep the earliest causal failure separate from the complete ordered error list.
Events distinguish preflight, output open, spawn, observation, child nonzero exit,
timeout, output threshold, signal, wait/reap, descriptor close, identity closure
and result retention. Never convert these into simulated game losses.

After a child was successfully created, cleanup must independently attempt:

1. Termination of the invocation-owned process group, including when the leader
   exited but an ordinary group descendant could still be writing.
2. A bounded wait for the direct child, with escalation to group kill and an
   independent direct-child kill fallback when termination cannot be confirmed.
3. A final bounded direct-child wait/reap attempt even if an earlier signal/wait
   failed. No unbounded `wait` is permitted; each wait uses the remaining budget
   for that cleanup phase. Allow at most two cleanup-grace phases after execution.
4. Closure of each acquired descriptor independently, including stdout if stderr
   creation fails before launch. One close failure must not suppress another.
5. Closing identity checks and result retention independently of earlier cleanup
   failures, when the owned directory/files permit those operations safely.

A not-yet-created child has no valid group ID: never signal a guessed ID or the
parent's group. Distinguish already-gone from permission/other signal failures.
Do not scan/kill unrelated system processes. Only the direct child can be reaped
by this helper; ordinary descendants in its group can be signaled but are not
claimed as reaped. Descendants escaping the session/group, PID/group reuse,
uninterruptible OS calls, parent SIGKILL and machine failure are outside its
containment guarantees. Unconfirmed cleanup is an explicit failed result.

Record successful signal delivery or an already-absent group as those observations,
not as proof that every descendant was reaped or the group was exhaustively
inspected. Required cleanup success means the direct child was reaped and the
prescribed group/fallback operations completed without unresolved operational
errors. It does not upgrade the containment guarantees above.

The runner exposes its last/final result even when a later operation fails.
Ordinary operational failures return a failed result; they never return success
merely because a child exited zero. For cancellation by `KeyboardInterrupt` or
`SystemExit`, attempt the same cleanup and finalization, then re-raise the original
exception object with the result accessible on the runner. Do not let a secondary
cleanup/retention error replace the cancellation. This is tested control flow,
not a claim that every asynchronous interruption is recoverable.

## 6. Result record and privacy

The result separates observations from acceptance:

- Case/attempt identity, exact command/environment description and declared file
  bindings; opening, invocation and closing check results.
- Whether spawn succeeded; owned PID/group identifiers; observed return code or
  signal; whether the direct child was reaped; timeout/output-limit flags.
- Ordered operations and all failures, including first cause and unsuccessful
  cleanup/close attempts. Unknown/not-attempted observations stay explicit.
- Per-stream path, observed size and, only when verified stable, byte count/SHA-256.
- Overall operational success and a separate finite metadata-retention outcome.

Success requires exit zero, no limit violation, successful required checks and
cleanup, verified outputs and successful metadata retention. It establishes only
that an authored invocation completed under these checks, not combat correctness.

Write a private `capture.json` exclusively after output finalization. Its contents
cover events before its own retention; it cannot claim its own hash or future
verification. The runner's outer final result separately records that file's
actual retention/readback identity or error. Do not recursively journal journals
or overwrite `capture.json` to add its own hash. A partial write stays partial and
failed; a caller may retain the outer result only at a distinct fresh location.
The inner record explicitly leaves its own retention pending; only the outer
result can assert overall success after retention/readback. Do not serialize an
inner success claim that presumes those later operations succeeded.

No game code, bytecode, binaries, assets, private raw output, private command paths
or full exception text enters public reports or committed fixtures. The returned
full result is private when used on private inputs. Public administrative reports
contain bounded case/status/count summaries only, never the raw result object.
Purely authored fixtures may be inspected in tests; do not make private files a
test dependency. No stdout/stderr payload belongs in the result's representation.

## 7. Prospective original-combat matrix

The readiness guide describes required coverage; it is not a new trace/wire schema,
an importer, a completed test corpus or a repair of frozen Task 2 contracts.
Each row identifies a requirement, proposed case labels, required observation,
matching method, evidence class/status and later review gate. Initially every
original-runtime row is **awaiting original capture**; inherited static facts and
735 authored Python tests are recorded separately, never as runtime satisfaction.

Minimum matrix:

| Requirement | Required later original evidence |
|---|---|
| Complete Cultist combats | At least three distinct original seeds; supplied first-ready decision through terminal cleanup |
| Complete Jaw Worm combats | At least three distinct original seeds; same boundary |
| Card scope | Base/upgraded Strike, Defend and Bash across the reviewed corpus |
| Block and damage | Block gain/consumption/caps, player/enemy block-reset boundaries and supported arithmetic examples |
| Vulnerable | Bash's damage-before-application, stacking/expiry and relevant source-side grace |
| Enemy lifecycle | Cultist Ritual first-skip/Strength/preview progression; Jaw Worm move/history restrictions, Thrash damage-before-block and Bellow buff/block |
| Piles | Actual draw order, reverse ordinary hand discard and at least one discard recycle/shuffle |
| Victory | Burning Blood capped healing; player HP immediately before victory healing and after completed cleanup |
| Death | No victory healing; both HP observations; no ordinary work after terminal completion |
| Controls | Legal/unsupported boundary and selected controlled edge cases, with individually justified observation points |
| Reference/platform integrity | Identity-bound adapter/export/load/visibility and enabled-vs-disabled non-interference checks; Windows gates remain separate |

Three seeds per enemy is a minimum original corpus requirement, not a statistical
proof of enemy chances. Cases may satisfy multiple rows only when their actual
observations support them; six complete captures alone do not satisfy every row.

For each later case, require the original build identity, private original seed
binding, actual first-ready state, semantic action identities, reviewed random
outcomes needed for alignment, decision/terminal observations, raw evidence
identities and independent review. Capture setup manipulations must be disclosed;
do not infer natural original startup from a supplied state. Unsupported original
content or an unobservable required checkpoint fails the affected case's gate.

Under [RNG-001](../../intentional-deviations.md#rng-001--decorrelated-reproducible-project-randomness),
shared original/project seeds are not matching instructions. A future comparison
adapter must explicitly define how equivalent semantic states/actions and random
outcomes are supplied or aligned. This milestone does not add RNG overrides or
search project seeds until outputs happen to agree. Do not compare divergent
random trajectories as if every difference were a mechanics defect, or use RNG-001
to excuse a real mechanics mismatch. Preserve exact conditional chance laws and
their existing rational tests; runtime frequencies do not replace that proof.

## 8. Synthetic controls and acceptance

Write focused failing tests before implementing the helper. Use the real Linux
child lifecycle for ordinary success/nonzero/timeout/group behavior; monkeypatch
only specific exceptional operations whose failure needs deliberate injection.
No mocked success-path subprocess is sufficient evidence of process cleanup.

Required control families:

1. Success and nonzero exit, distinct stdout/stderr bytes/hashes, exact argv/cwd/
   environment, empty stdin and no raw output leakage to parent tool streams.
2. Timeout, bounded output-threshold crossing and a leader that exits while an
   ordinary same-group child remains; retained evidence and no surviving runnable
   owned child in controlled cases. Test zombie/reap status honestly.
3. Rejected command/input identity, unsafe paths/modes, existing output destination
   and changed closing identity; assert whether any child launched via markers.
4. Partial descriptor acquisition and spawn failure; all acquired descriptors
   receive their independent cleanup attempts and existing evidence is untouched.
5. Injected observation/wait/signal/close failures, including multiple failures
   together; assert subsequent attempts actually occur and original-cause
   precedence plus every distinct error is retained.
6. Cancellation, exhausted cleanup budgets, closing-check failure, result-write/
   readback failure and attempted reuse; no success or automatic retry.
7. Mutation-sensitive checks: suppressing later cleanup after the first error or
   replacing the first failure must break targeted assertions, not just counts.

Synthetic tests may create their own bad-mode/link/mismatch fixtures to test
rejection; they must not repair or reuse actual private evidence. Real-process
controls have an independent test watchdog and finally-based test-owned cleanup
so a faulty helper cannot hang the suite or leave live authored children. Record
any watchdog intervention as failure; never silently turn it into a passing run.
Use explicit fixtures/markers rather than precise sleep-duration performance
assertions. Keep stdout/stderr workloads finite in all controls.

Retain actual commands/results and exact final source identities for an independent
review. Run focused controls, the full configured Python suite and whitespace/
scope checks. Do not claim a final test count before execution. Verify the existing
RNG code/tests and fixed fixture remain unchanged; fixture SHA-256 stays
`369d91e2ce3be38c191ef5ceab6fe6e1bf5c73f21504e1748074e9c35d962bad`.

Before any future original input, exercise the **exact** runner and relevant local
dependencies on authored controls first. Substantive source changes invalidate
that prospective control binding. Later successful controls do not retroactively
cure historical missing preflight, finalizer/outer-launch attribution, F30
quarantine or cleanup-robustness limitations.

## 9. Explicit exclusions and next gates

Do not change combat rules/snapshots/RNG, the original C++ repository, training,
Rust simulation, trace-v1 or its frozen/unratified recovery work, Windows mod
readiness predicates, `reference-build.json` verification or historical evidence.
No Steam/server installation, game/loader download, desktop access, original JAR
read/execution, new original method study or private corpus audit is authorized.

Next gates, in order:

1. User reviews and approves this written specification — received on 2026-10-16,
   with the requested repository-wide scope documentation aligned.
2. Identify the concrete next StS validation consumer and any feasibility blocker
   under section 2; then write a bounded implementation plan with exact interfaces,
   tests, lifecycle sequencing and stopping rules. Retain the selected
   Sol/fresh-reviewer method. Do not plan unused generic infrastructure.
3. User reviews and approves the plan/start before helper/test implementation.
4. Implement synthetic readiness, obtain fresh review and verify the final source.
5. Separately design/approve original Linux feasibility, access, execution, adapter
   observation/matching and private evidence retention; collect nothing by default.
6. Obtain/review original combat evidence and outstanding platform gates before a
   validated-reference label or Rust handoff. Linux readiness alone closes none
   of those gates.

The implementation may claim **synthetic Linux capture readiness** only within
its measured controls and documented limitations. It may not claim sandboxing,
universal exception safety, original headless-runtime support or original combat
validation. Keep all evidence and worktree history intact; nothing is pushed,
merged, staged, committed or deleted automatically.
