# Repository review — 2026-10-23

## Scope and status

This records the user-requested review of **Spire Gym**, not the separate
`sts_lightspeed_all_characters` repository. The audited code revision was
[`c0746b7751773cf93cb24a780b65a7ca4a9cbe66`](https://github.com/klottick/spire-gym/commit/c0746b7751773cf93cb24a780b65a7ca4a9cbe66),
with the then-uncommitted scope, approval and status documentation also inspected.
Those documentation observations are working-tree observations, not a claim that
their exact text was committed at that revision.

**This is an audit record, not a current open-issue list or implementation plan.**
The summary PR was prepared against `ddd4df2138db8bc1860170bb079743184dbf6335`,
after later milestone, reference-library and performance work. Findings have not
all been revalidated against that newer code. In particular, the README approval
wording in finding 6 has since been superseded. Consult current milestone profiles
and task ownership before acting on any recommendation.

No fixes, policy changes, dependency changes, new verification infrastructure or
mechanics acceptance are included. Original-game execution, private-source access
and exhaustive original-game numeric parity were outside the review. Reference
datasets and their integrity checks are not implementation or runtime proof.

## Findings

P2 denotes an important correction or optimization opportunity; P3 denotes a
lower-priority or narrowly triggered issue. Numbers preserve the original report's
14 findings. Source links below are pinned to the audited revision; quoted line
numbers must not be interpreted as current-branch locations.

### Runtime correctness and evidence tools

1. **P2 — Later export failures can relabel previously delivered replay records.**
   In a two-slot manual-reset pool, completing both episodes, leaving their records
   undrained, then failing packaging for a reset of slot 1 changed slot 0's earlier
   record from `Completed` to `Faulted`. Shared queued-record updates did not honor
   the current delivery boundary. Independently reproduced through the native API.
   Restrict annotation to records owned by that delivery and cover manual-mode
   partial resets and later export faults.
   [Source: `pool.rs:483–505`](https://github.com/klottick/spire-gym/blob/c0746b7751773cf93cb24a780b65a7ca4a9cbe66/crates/spire-env/src/pool.rs#L483-L505).

2. **P2 — Java JSON parsing silently truncates decimals without exponents.**
   `intEnd` is captured before the fractional part, so checking its substring for
   `.` cannot detect a fraction. `1.75` consequently becomes the number token `1`,
   violating the parser's lossless-number contract. Preserve the complete token
   and make integer-only consumers reject fractions. Source-confirmed; Java
   execution was unavailable during the audit.
   [Source: `Json.java:401–423`](https://github.com/klottick/spire-gym/blob/c0746b7751773cf93cb24a780b65a7ca4a9cbe66/validation/windows-mod/protocol/src/main/java/spirelab/protocol/Json.java#L401-L423).

3. **P2 — Bytecode anchors can attribute another method's instructions to a selected method.**
   Member-boundary detection misses package-private declarations. A synthetic
   fixture returned `PASS` for a public `run()` using instructions belonging to a
   subsequent package-private `other()`. Reuse the existing member-boundary parser
   and add a focused regression. This does not establish that every historical
   report is wrong or authorize reopening the historical verification program.
   [Source: `evidence_report.py:199–217`](https://github.com/klottick/spire-gym/blob/c0746b7751773cf93cb24a780b65a7ca4a9cbe66/validation/evidence_report.py#L199-L217).

4. **P2 — Source anchors accept commented-out declarations.**
   Matching raw source lines allowed a commented-out `public void run()` to satisfy
   an anchor for a nonexistent real method. Independently reproduced with authored
   synthetic input. Apply existing comment/literal masking while preserving line
   numbers; no new general-purpose verifier is needed.
   [Source: `evidence_report.py:260–268`](https://github.com/klottick/spire-gym/blob/c0746b7751773cf93cb24a780b65a7ca4a9cbe66/validation/evidence_report.py#L260-L268).

### Documentation and packaging

5. **P2 — Setup/rebuild guidance omitted Gymnasium before recommending the full Python suite.**
   The audited development guide used `uv sync --locked --dev`, although Gymnasium
   was an optional extra and environment tests imported it unconditionally. The
   environment guide already showed `--extra gymnasium`. Distinguish native-only
   and Gymnasium-enabled setup, and rebuild the editable extension after sync.
   Locations observed: `docs/development.md:73,108–111`;
   [`pyproject.toml:12–13`](https://github.com/klottick/spire-gym/blob/c0746b7751773cf93cb24a780b65a7ca4a9cbe66/pyproject.toml#L12-L13).

6. **P2 — Approval wording contradicted recorded design/plan/start approval.**
   The audited working-tree README said components awaited the actual
   design/plan/start while also recording those approvals. Similar gate wording
   appeared in the deviation register. This risked unnecessary approval loops.
   **Historical observation:** the newer README has replaced that wording with
   progressive milestone status. Do not restore the old gate or treat this audit
   as a new scheduling or implementation authorization.
   Locations observed: `README.md:28–34` and
   `docs/intentional-deviations.md:87–88` in the audit's working tree.

7. **P2 — The central deviation register omitted the carried-forward empty-master-deck terminal policy.**
   The then-approved full-run design selected settled-empty-permanent-deck loss
   with a −1 terminal contribution; its predecessor explicitly called this a
   human-selected native policy, not source behavior. The central register listed
   RNG-001 and TIME-001 but not this exception. Preserve that distinction and the
   settlement checkpoint when documenting any applicable current contract. This
   was a planned-contract disclosure gap, not an implemented full-run regression;
   the later roadmap must be consulted before carrying the policy forward.
   Locations observed: full-run design line 204, predecessor consolidated design
   line 194, and `docs/intentional-deviations.md`.

8. **P2 — Gym installation required large device-readiness dependencies unused by gym stepping.**
   Torch was mandatory, while its only production Python import was the historical
   readiness module. Listed Linux x86-64/CPython 3.12 Torch and NVIDIA wheel archives
   totaled approximately **2.74 GB**, plus approximately **248 MB** of Triton.
   These are lockfile archive sizes, not measured installed size. Consider making
   readiness dependencies optional only under separately authorized packaging work;
   their deliberate retention was documented.
   [Manifest: `pyproject.toml:7–10`](https://github.com/klottick/spire-gym/blob/c0746b7751773cf93cb24a780b65a7ca4a9cbe66/pyproject.toml#L7-L10),
   [consumer: `readiness.py:20`](https://github.com/klottick/spire-gym/blob/c0746b7751773cf93cb24a780b65a7ca4a9cbe66/python/spire_lab/readiness.py#L20),
   [lockfile](https://github.com/klottick/spire-gym/blob/c0746b7751773cf93cb24a780b65a7ca4a9cbe66/uv.lock#L490-L519).

9. **P2 — The everyday “Python behavior tests” command tested readiness, not the gym.**
   The development guide directed callers only to `test_readiness.py`, which covers
   native metadata and device/optimizer smoke checks. It did not exercise the
   Python environment contract. Use environment tests for the everyday gym workflow
   and label readiness checks separately.
   Location observed: `docs/development.md:92–93`;
   [readiness tests](https://github.com/klottick/spire-gym/blob/c0746b7751773cf93cb24a780b65a7ca4a9cbe66/tests/python/test_readiness.py).

### Efficiency

10. **P2 — Shuffle sampling repeatedly hashes the same four-lane RNG block.**
    Each word recomputed HMAC, allocated its message and cloned scope-bearing
    stream state. A release microprobe compared the existing sampler with a local
    digest/cursor: nine-item shuffles took **151.8 ms versus 29.3 ms** for 100,000
    iterations; 999-item shuffles took **359.0 ms versus 72.9 ms** for 2,000.
    Fixed-stream permutations and final cursors agreed. This is roughly **5×
    sampler overhead**, not a whole-combat speedup or a validated replacement.
    Preserve rejection, exhaustion and replay contracts in any optimization, and
    reuse the repository's subsequently added performance infrastructure.
    [Source: `rng.rs:149–193`](https://github.com/klottick/spire-gym/blob/c0746b7751773cf93cb24a780b65a7ca4a9cbe66/crates/spire-core/src/combat/rng.rs#L149-L193),
    [shuffle: `rng.rs:268–278`](https://github.com/klottick/spire-gym/blob/c0746b7751773cf93cb24a780b65a7ca4a9cbe66/crates/spire-core/src/combat/rng.rs#L268-L278).

11. **P3 — Reset validation has quadratic membership work.**
    Return-all reset repeatedly searched selected indices for every slot, and the
    binding separately searched preceding indices for duplicates. A bounded debug
    invalid-reset preflight probe measured approximately **8/30/119/471/1901 ms**
    at **2k/4k/8k/16k/32k** slots. These are not release reset-throughput results;
    the O(B²) cost also follows directly from the loops. Use a selection bitmap or
    efficient sorted membership without weakening atomic preflight.
    [Native source](https://github.com/klottick/spire-gym/blob/c0746b7751773cf93cb24a780b65a7ca4a9cbe66/crates/spire-env/src/pool.rs#L189-L193),
    [binding source](https://github.com/klottick/spire-gym/blob/c0746b7751773cf93cb24a780b65a7ca4a9cbe66/crates/spire-python/src/config.rs#L123-L130).

### Offline-reference and parser edge cases

12. **P2 — The Python reference faults on accepted long integer card handles under default CPython settings.**
    Entry accepted a 5,000-digit handle, but drawing converted it with `str()` after
    moving the card, exceeding CPython 3.12's default 4,300-digit limit and leaving
    no reveal recorded. Independently reproduced. Differential tests deliberately
    use a child with `int_max_str_digits=0` for long handles; they do not establish
    default-interpreter behavior. This is a known **offline-reference** limitation,
    not a native production bug. Prefer bounded conversion and a default-settings
    regression over changing process-wide limits.
    [Reference source](https://github.com/klottick/spire-gym/blob/c0746b7751773cf93cb24a780b65a7ca4a9cbe66/python/spire_lab/combat/piles.py#L51-L54),
    [explicit test accommodation](https://github.com/klottick/spire-gym/blob/c0746b7751773cf93cb24a780b65a7ca4a9cbe66/tests/python/combat/rust_diff_support.py#L226-L243).

13. **P3 — Python public views accept busy checkpoints with stale intent damage.**
    At a legitimate Cultist safe point before queued Ritual Strength resolved,
    Python returned damage **6**, while legal-action access rejected the busy state;
    after resolution, damage was **9**. Native observations reject such checkpoints.
    Independently reproduced. Add a matching readiness guard and a focused test.
    This affects internal reference/checkpoint access, not ordinary synchronous
    native Gymnasium stepping.
    [Python view](https://github.com/klottick/spire-gym/blob/c0746b7751773cf93cb24a780b65a7ca4a9cbe66/python/spire_lab/combat/view.py#L84-L99),
    [native readiness guard](https://github.com/klottick/spire-gym/blob/c0746b7751773cf93cb24a780b65a7ca4a9cbe66/crates/spire-core/src/combat/view.rs#L55-L62).

14. **P3 — javap instruction comments can be mistaken for class wrappers.**
    An instruction comment such as `0: ldc #7 // String class Fake {` caused a
    single-class synthetic fixture to fail with “expected one javap declaring-class
    wrapper.” Recognize declaration headers rather than comment/instruction text.
    Independently reproduced without a Java runtime or original-game input.
    [Source: `java_members.py:178–181`](https://github.com/klottick/spire-gym/blob/c0746b7751773cf93cb24a780b65a7ca4a9cbe66/validation/java_members.py#L178-L181).

Additional low-priority audit observations included comments describing rust-numpy
as unused/future batching, core/env as documentation-only, and Rust parity as
future work. Preserve historical test counts, but do not confuse them with current
implementation status. These comments were not additional gameplay defects.

## Verification recorded during the audit

- Debug and release Rust workspace runs each passed **169 tests**: 112 core unit,
  14 trace integration, 38 environment and 5 Rust export-guard tests.
- An all-features workspace compile check passed, including feature-gated binding
  code. Forty-four authored whole-combat Python/native comparison cases agreed.
- Replay attribution, synthetic anchors, long-handle and busy-view cases, and the
  bounded shuffle/reset microprobes were rerun independently.
- Python pytest/Gymnasium/extension suites were not rerun: the audit checkout had
  no virtualenv, extension or installed dependencies. Java tests were not run
  because no JDK was available. Nothing was installed.

**These are historical audit results, not test results for the summary PR or for
later milestones.** There was no original-runtime validation or exhaustive numeric
parity check. Private/local probe artifacts are not published by this document.

For the documentation-only summary PR, all 14 entries and 18 pinned source links
were checked, and focused native environment/export tests passed **38 + 5** against
the newer base. These checks do not revalidate the findings or claim a green full
workspace, Python-extension runtime suite or Java suite. No product files changed.

## Follow-up boundaries

Revalidate applicability before assigning a fix. Prioritize replay attribution and
numeric/anchor correctness, then setup/status documentation. Keep reference-only
edge cases separate from production failures. Packaging and performance changes
need their own bounded scope and contract-preserving tests; use existing tooling
rather than building a new verification platform. This report does not reopen
historical research or authorize milestone implementation.
