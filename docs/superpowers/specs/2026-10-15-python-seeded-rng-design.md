# Python seeded RNG reference — Task3 design amendment

> **Confirmed intentional deviation (2026-10-16):**
> [RNG-001](../../intentional-deviations.md#rng-001--decorrelated-reproducible-project-randomness)
> is now user-approved policy. This exact-original-RNG draft remains superseded;
> its vector-parity and NaN-replication gates are not replacement-RNG requirements.

> **Direction superseded before approval:** the user subsequently stated that exact
> original RNG matching is not required and that original correlated randomness
> need not carry forward into training. Retain this draft as history; do not execute
> its exact-bit core/full acceptance proposal. See the latest steering entry in
> `.superpowers/sdd/2026-10-15-python-behavior-readiness/progress.md`. A replacement
> randomness design is being clarified; no simulator implementation is authorized.

Date: 2026-10-15

Status: **Draft for review, not implementation authority.** The user selected this
narrower milestone and requested design/plan preparation. Written-design review,
the concrete implementation plan and implementation-start approval remain distinct.
No Git staging or commits are authorized.

## 1. Purpose and amendment boundary

Independently author the seeded RNG portion of the readable Python reference,
using the accepted original engine/wrapper observations as private correctness
fixtures. Success means exact selected results, engine states, wrapper counters,
failure mutations and replay—not a working or validated combat simulator.

Proposed staging after the evidence-to-test mapping: **core first**, with all ten
engine operations, nine wrapper operations and four constructor signatures. Defer
the two wrapper floating scaling/range operations in their entirety until their
exceptional raw-bit policy is established. This is a proposed scope refinement for
review, not a silently accepted exclusion of fourteen inconvenient outputs.

This amends only the RNG portion of Task3 in
`docs/superpowers/plans/2026-10-14-python-combat-reference.md`. It retains the
architecture in `2026-10-14-python-combat-reference-design.md` and the accepted
structural Tasks1–2. The original plan is retained as history; a new plan will
identify the precise superseded gate and executable subset.

Do not integrate these RNG values into `CombatState` yet. In particular,
`model.RngState(slots=...)`, `Scenario.rng`, `CombatState.rng`, validation,
snapshot schema/content identity and first-fault behavior remain unchanged.
No stream names, stream topology, combat seed mapping or scheduling are inferred.

Out of scope: shuffle/draw, combat arithmetic and mechanics, public views,
Tasks4–5, Rust, training, performance, noarg/copy/global RNG helpers, inherited
APIs/Gaussian caches, original Windows/library parity and live adapters/captures.

## 2. Evidence boundary

The accepted reference set consists of:

| Selection | Private relative path | Bytes | SHA256 |
|---|---|---:|---|
| Engine | `stage04-runtime/vectors.json` | 948749 | `ed6de38521c7e2d28a577f689bc0a069cf7f80a6925f3dacdeed159f1373bd71` |
| Wrapper | `stage07-wrapper/vectors.json` | 682676 | `95a4afb4c91cc4653a43298f3a2f4b2f9adc7739006de845efeb551fff90c804` |

Paths are relative to the approved private RNG runtime-probe tree. Exact identity,
lineage, logger conditions, selected plans, controls and receipts are retained in
the existing dispositions and review reports:

- `.superpowers/sdd/2026-10-15-python-behavior-readiness/engine-vector-disposition.md`
- `.superpowers/sdd/2026-10-15-python-behavior-readiness/wrapper-vector-disposition.md`
- `probe-review.md`, `probe-review-completion.md` and `wrapper-probe-review.md`
  in the same administrative directory.

The selected engine coverage is 413 completed cases and 707 calls including 42
exceptions. The selected wrapper coverage is 283 invoked cases: 281 completed
sequences and two constructor failures, with 445 calls including 50 exceptions.
That is 696 selected cases, 694 completed sequences, 1152 sequence calls and 92
call exceptions across the two selections. Setup, getter, restoration, draw and
constructor controls and repeated JVMs are separate—not extra selected calls.

Retain all selected cases; do not cherry-pick passing ones or turn failures into
skips. The old stage04 wrapper initialization failure/skips are historical records,
not an additional wrapper oracle alongside the successful stage07 selection.

The runtime evidence is isolated original-bytecode execution on pinned Linux
Java21, with an explicitly controlled original SimpleLogger path for the wrapper.
It does not establish exhaustive inputs, original Windows behavior or combat
correctness. Behavior facts from the separately accepted static study retain their
static evidence class; neither evidence class becomes a whole-combat capture.

The private mapping handoff is
`.superpowers/sdd/2026-10-15-python-behavior-readiness/rng-contract-planning-report.md`.
It binds the case/call/control selections and a sixteen-entry behavior ledger in
`planning-rng-contract-20261015-b/` under the private probe tree. Selected inventory:
`selected-planning-inventory.json`, 3183 bytes, SHA256
`c704a35626385ce2a9d25a417f9329f3a79aab4c5529cb48087e5fb6674b2245`.
The ledger is `normative-behavior-ledger.json`, 138932 bytes, SHA256
`46602c548607825fee007043afd3d57315bcaf63e894d423c4a41f1695b33d50`.
This newly authored planning mapping needs its own review; acceptance of its input
vectors is not automatic acceptance of every derived ledger statement.

The implementation plan must bind exact case IDs, descriptors and assertion paths
from that mapping. Separate supported behavior facts, observed samples and unresolved
paths. Runtime sampling alone is not an algorithm specification. Any unresolved
operation must be explicitly excluded in the reviewed plan or resolved before
implementation; never use a sample lookup table as the RNG implementation.

### Exceptional-floating gap and two distinct gates

G01 concerns general NaN payload/sign propagation in wrapper floating arithmetic.
Fourteen selected `random(FF)F` result records have exact known NaN encodings, but
the accepted facts do not establish a general independent policy for all NaN/
invalid-arithmetic inputs or target runtimes. The unobserved exceptional paths of
`random(F)F` also have no general raw-bit guarantee. Known sample encodings must
remain unchanged; matching them by lookup or canonicalizing them is not a repair.

The proposed **core gate** excludes both `random(F)F` and `random(FF)F` completely,
not just their NaN-producing calls. Its case selection must be fixed before product
implementation by descriptor membership: retain each entire case only if all its
sequence operations are in the core surface. Do not delete an operation from a
mixed sequence, inject recorded intermediate states to bypass it, or select cases
by whether an implementation passes. The plan must give exact included/excluded
IDs and counts, with retained control membership and zero skips within the core.

The **full seeded-RNG gate** remains all 696 cases/1152 calls and their selected
controls, including every raw NaN encoding, with zero skips. It stays blocked by
G01. Passing the smaller core gate is not passing the full gate or all of Task3.
All excluded records remain in the immutable full inventory and gap report; no
xfail/skip or altered expected value may make the full gate green.

Independent mapping/boundary review: `rng-design-mapping-review.md` in the same
administrative directory. It found no API/type/privacy boundary defect and checked
the following exact proposed whole-case selection, without approving implementation:

| Coverage | Proposed core | Excluded intact |
|---|---:|---:|
| Cases | 640 | 56 |
| Engine / wrapper cases | 413 / 227 | 0 / 56 |
| Completed sequences / constructor failures | 638 / 2 | 56 / 0 |
| Calls | 1040 | 112 |
| Call exceptions / failure continuations | 92 / 15 | 0 / 0 |
| Replay checks | 638 | 56 |
| Wrapper replay restores | 225 | 56 |

All nineteen core signatures and all four constructors remain represented. Core
retains all 182 draw controls/296 draws, eight initial-state harness controls and
seven original constructor/counter controls. Exact IDs, descriptors and control
references are bound by the private review's
`mapping-review-20261015T224055272288Z/proposed-core-selection.json`, 355571 bytes,
SHA256 `fd6e4536eced079609025732d448d58e2731d0d89e91c33d4c22c47dbffca679`.
These counts describe a proposed scope, not executed Python tests.

## 3. State and value ownership

All runtime types below are project-authored frozen value dataclasses in
`python/spire_lab/combat/rng.py`. They contain only ordinary Python values and
have no original classloader, Java runtime, native or tensor dependency.

| Type | Fields / purpose |
|---|---|
| `EngineRngState` | `word0: int`, `word1: int`; both represented as signed64 values |
| `SeededRngState` | `engine: EngineRngState`, `counter: int`; signed32 counter |
| `Float32` | `bits: int`; unsigned 32-bit encoding, not a Python-float surrogate |
| `Float64` | `bits: int`; unsigned 64-bit encoding |
| `RngFailure` | `original_class: str`, `original_cause_class: str \| None`; source error categories only |
| `RngStep[S, T]` | `state: S`, `value: T \| None`, `error: RngFailure \| None` |
| `RngConstruction[S]` | `state: S \| None`, `error: RngFailure \| None` |

These are the proposed project API names. The private planning appendix uses
`EngineState`, `WrapperState`, `F32Bits`, `F64Bits`, `CallResult` and
`ConstructorResult` as proposal labels for the corresponding concepts; it does not
override this public ownership/name decision or require additional alias classes.

`EngineRngState` does not claim to encode inherited platform state. A selected
call's replay state is the pair plus wrapper counter where applicable. Do not add
Gaussian-cache or other inherited behavior under this representation.

Dataclass construction and API boundaries require exact scalar types: bool is not
accepted as int, and machine-width integer inputs must be in their declared signed
domain. Encoding fields require unsigned-width integers. Malformed project inputs
raise authored `TypeError`/`ValueError` before calculation; they are not fabricated
original-game failures. Do not reject an in-domain state pair merely because it
looks unusual, or impose conventional restrictions on bounds/counters before
performing the evidenced operation.

Functions are pure: they return the next immutable state, leaving the supplied
state untouched. No mutable generator, global seed, Python `random`, hidden cache,
ambient randomness or implicit draw occurs. Reusing an input state is deliberate
branching/replay, not shared-generator aliasing.

## 4. Named operation surface

Use explicit named functions rather than a production string/descriptor dispatcher.
The Java signatures below identify evidence selections, not copied implementation.
All operation functions take the state first and return `RngStep`.

### Engine

| Project function | Selected original signature | Value type |
|---|---|---|
| `engine_next_long(state)` | `nextLong()J` | `int` |
| `engine_next_int(state)` | `nextInt()I` | `int` |
| `engine_next_float(state)` | `nextFloat()F` | `Float32` |
| `engine_next_double(state)` | `nextDouble()D` | `Float64` |
| `engine_next_boolean(state)` | `nextBoolean()Z` | `bool` |
| `engine_next_int_bounded(state, bound)` | `nextInt(I)I` | `int` |
| `engine_next_long_bounded(state, bound)` | `nextLong(J)J` | `int` |
| `engine_next_bits(state, bits)` | `next(I)I` | `int` |
| `engine_set_seed(state, seed)` | `setSeed(J)V` | `None` |
| `engine_set_state(state, word0, word1)` | `setState(JJ)V` | `None` |

Factories: `engine_from_seed(seed: int) -> EngineRngState` and
`engine_from_words(word0: int, word1: int) -> EngineRngState` cover the selected
`(J)V` and `(JJ)V` constructors. Seed/words are signed64; bound/bits parameters use
their listed signature widths. Protected `next(I)I` is an explicitly selected
low-level operation, not evidence for arbitrary inherited APIs.

### Seeded wrapper

| Project function | Selected original signature | Value type |
|---|---|---|
| `seeded_next_long(state)` | `randomLong()J` | `int` |
| `seeded_next_boolean(state)` | `randomBoolean()Z` | `bool` |
| `seeded_next_float(state)` | `random()F` | `Float32` |
| `seeded_int_max(state, maximum)` | `random(I)I` | `int` |
| `seeded_int_range(state, minimum, maximum)` | `random(II)I` | `int` |
| `seeded_long_max(state, maximum)` | `random(J)J` | `int` |
| `seeded_long_range(state, minimum, maximum)` | `random(JJ)J` | `int` |
| `seeded_float_max(state, maximum)` — deferred | `random(F)F` | `Float32` |
| `seeded_float_range(state, minimum, maximum)` — deferred | `random(FF)F` | `Float32` |
| `seeded_probability(state, probability)` | `randomBoolean(F)Z` | `bool` |
| `seeded_set_counter(state, counter)` | `setCounter(I)V` | `None` |

Integer parameters use signature widths; floating inputs are `Float32` values,
not implicitly coerced Python floats. Factories
`seeded_from_seed(seed: int | None)` and
`seeded_from_seed_counter(seed: int | None, counter: int)` return
`RngConstruction[SeededRngState]` for the selected boxed-Long constructors.
Null seed is an observed original constructor path, not a host type error.
The two deferred function names document the full surface only; do not create
stubs, export them or implement a finite-only overload under those names in the
core milestone. Unit floating draws and probability comparisons remain in the core;
they do not claim a policy for exceptional floating arithmetic.

### Outcomes, errors and replay

A normal call returns `error=None`, the evidenced next state and its value; a
successful void setter has `value=None`. An observed call failure returns
`value=None`, a source-category `RngFailure` and the actual partial next state.
It must not replace that state with the input or assume one draw per API call.

This low-level API does not latch failures. The original selected sequences
continue after captured call exceptions, so replay tests must do the same using
the returned state. This does not alter the separate combat first-fault policy.
Malformed host inputs remain separate from observed original-call failures.

A successful constructor has a state and no error; a failed constructor has
`state=None` and an error, with no fabricated partially constructed generator.
These alternatives are exclusive: reject a construction with both state and error
or neither. A call step always has a state; a failing step cannot also have a
non-null value, and a successful non-void operation must have its declared value.
Original exception messages, causes' messages and Java float text remain private
diagnostics; authored error categories must match observed class/cause categories,
but the Python API does not promise copied messages or Java exception objects.
The planning map enumerates 92 call exceptions as
`java.lang.IllegalArgumentException` with no cause, and the two constructor
failures as `java.lang.NullPointerException` with no cause. Retain per-operation
membership and exact failure-state handoffs rather than assuming all invalid
inputs share a pre-draw validation path. Fifteen observed continuations follow a
failed call; eligible core cases must preserve them, not terminate early.

Snapshot/replay needs no new persistence format: retain the frozen state value and
apply the same operations again. Do not call `seeded_set_counter` as a restoration
shortcut. The original harness's reflective counter restoration remains disclosed
as an evidence control, not a game API or an asserted inverse.

## 5. Numeric exactness and independent authorship

`combat/_rng_numeric.py` owns narrowly scoped fixed-width and floating encoding/
arithmetic helpers required by the selected RNG paths. It is not `rules.py` and
does not establish combat damage, rounding or power mechanics.

The operation ledger governs where signed32/signed64 wrapping, logical/arithmetic
shifts, rejection loops, endpoint calculations, counter changes and binary32/
binary64 rounding occur. Do not apply a single end-of-expression cast or Python's
unbounded arithmetic where intermediate-width behavior is required.

Float results are compared by exact encoding. The full gate includes signed zero,
infinities and observed NaN sign/payload bits without normalization; its G01 block
is explicit. `Float32`/`Float64` are the public low-level values so Python's NaN
equality, float formatting or implicit float64 arithmetic cannot silently define
the contract. Core helpers cover only its selected paths, not deferred floating
scaling/range arithmetic; do not build a general numeric library or quietly assert
a hardware-independent NaN arithmetic rule.

Author behavior independently from reviewed facts and mathematical obligations.
Do not copy/transliterate Java bodies, ship original classes, invoke Java at test
or runtime, or generate product code/expected values from the probe implementation.
No lookup against private fixture inputs is allowed in product code.

## 6. Tests, fixture privacy and acceptance

Planned files, subject to the concrete implementation plan:

- `python/spire_lab/combat/rng.py`
- `python/spire_lab/combat/_rng_numeric.py`
- `tests/python/combat/test_rng_numeric.py`
- `tests/python/combat/test_rng_engine.py`
- `tests/python/combat/test_rng_wrapper.py`
- `tests/python/combat/test_rng_private_vectors.py`
- `tests/python/fixtures/combat/rng_cases.py` — test-only fixture adapter.

No existing model/snapshot/content exports or dependency/configuration changes are
needed. Test-only operation dispatch is finite and explicit; do not introduce an
importable production reflection layer, plugin system or generic evidence SDK.

Portable tests cover authored type/domain/error/immutability contracts, finite-word
and bit-encoding properties, deterministic branching and state-result structure.
They are self-authored tests, not evidence of original equivalence.

Private-oracle tests read exact identity-bound original fixture selections through
the explicit `SPIRE_LAB_RNG_ORACLE_ROOT` environment variable. With it absent,
portable runs explicitly skip the private-oracle suite. If it is present, missing
files, wrong hashes, malformed schema, unknown operation/case, incomplete coverage
or missing expected observations are hard failures, never skips or fixture refresh.
This is the chosen project variable; the appendix's proposed
`SPIRE_LAB_PRIVATE_RNG_ORACLE_ROOT` spelling is not a second alias or silent fallback.
The mandatory core acceptance run supplies the root and requires zero oracle skips
and exact preselected core coverage counts. A portable-only green suite cannot pass
that gate. Reports must separately state the full gate remains blocked by G01,
rather than displaying its omitted cases as a full-oracle pass.

The test adapter checks file sizes/hashes and schema before dispatch, never extracts
classes or runs Java, and fails closed on unsupported manifest forms. The plan must
pin every additional case-plan/control file actually needed by the adapter; vector
hashes alone must not be misrepresented as binding a different unselected manifest.
Descriptor/argument conversion is explicit and lossless, especially for float bits.
Read only explicitly selected regular files using checked descriptors, without
following symlinks, and verify expected size before loading a payload. The adapter
must not scan the private tree, inspect tool logs or load arbitrary referenced paths.
Fixture comparisons are authored-code tests against accepted data, not a new game
evidence acquisition; they do not require Java or the original archive at test time.

For each case selected for the applicable gate compare initialization, operation identity/order, result
type/value or float bits, both state words, counter where applicable, source error
categories and post-failure state; then repeat from the saved initial value.
Include two constructor failures without replay and the separately identified
original draw/constructor controls. Do not count controls as selected sequence calls
or a repeat as new coverage. Full control inventory: 182 draw controls/296 primitive
draws, 694 replay checks, 281 wrapper replay restores, eight initial-state harness
controls and seven original constructor/counter controls. The plan must name exact
core control membership rather than assume all controls survive case selection.

Raw vectors, original text, private ledgers and raw-derived diagnostics stay outside
Git. Repository fixtures contain authored loading/comparison code and permitted
path/hash/case/operation metadata only, not copied numeric vector payloads.
Test output must identify case ID, operation and differing field path without
printing private inputs, expected/actual payloads, original messages or traceback
locals. Synthetic mismatch tests must verify this output boundary before real
fixtures are exercised, including pytest assertion rendering and failure reports.
Parameterize tests by metadata-only case IDs, never raw case/state objects. Keep
loading and comparisons inside the private adapter; it returns only sanitized
status/field-path diagnostics. Catch unexpected comparison/parse failures before
their sensitive frames reach pytest, without forwarding exception text or chained
tracebacks. Do not use introspected equality assertions over private payloads.
The synthetic leak checks must include unexpected errors, captured stdout/stderr
and pytest local-variable display, not just the intended mismatch message.

Acceptance also requires independent code review, the complete applicable portable
suite, a zero-skip core private oracle run and no changes to the accepted structural
combat behavior. Future evidence harnesses must keep immutable stage identity
separate from per-process labels, per WR-ADMIN-1; do not rewrite old receipts.

## 7. Readiness and next handoff

The next implementation plan must supply a reviewed operation-to-evidence contract,
exact fixture selections and assertions, helper ownership and red/green steps.
Public report metadata cannot substitute for a missing behavioral specification.
If required facts remain unresolved, report the exact blocked paths before asking
to start implementation. Do not silently mark all of Task3 ready from these counts.

**D-PREREQ-1:** the new ledger's sixteen behavior clauses N01–N16 have independently
checked references, not semantic approval. The reviewer read the mapper's bounded
source but did not read those behavior-prose bodies under the harness-source-only
transport exception. Necessary clause/accepted-fact semantic reading through normal
tools/model context needs its own narrow permission; it must not be implied by
matching hashes, pointer resolution or the boundary verdict. No original game code,
raw numeric vectors or renewed original execution is needed for that review.
Complete it before declaring an algorithmic plan execution-ready. This is a review
readiness prerequisite, not a defect in the accepted observations or an artificial
retry limit. Written user design/plan approval remains separate.

On core completion, the permitted status is an independently authored seeded RNG
core matching its exact declared subset of accepted observations. The full gate
remains blocked by G01 until a reviewed exceptional arithmetic policy and all full
fixtures pass. Shuffle, combat
integration, Windows validation and the complete Python reference remain gated.
Implementation execution and any Git write require their applicable approvals.
