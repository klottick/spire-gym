# Windows reference capture — handoff guide (BLOCKED on user evidence)

> **Current scope context (2026-10-16):** See [repository scope](../scope.md).
> Validation is offline supporting work for the headless Rust simulator and
> measurable AI improvement, not a playable client or capture-platform product.
> This guide records the earlier Windows portable-stage handoff; its real adapter
> and platform gates remain incomplete. The separate Linux readiness specification
> is approved for planning only and does not unblock this build or authorize new
> original execution. Exact-original-RNG references in this historical handoff are
> subject to [RNG-001](../intentional-deviations.md); other evidence gates remain.

This is the **concrete Windows/mod/runtime handoff** for Task 2. The
game-independent portable stage (protocol, writer, coordinator, Rust reader,
and the fail-closed mod build) is implemented and tested on Linux. What remains
is **user-provided evidence** on a Windows machine with the pinned game. This
document is the precise list of what must be produced to unblock the rest —
it is a handoff, not a claim of completion.

> **Nothing here has been executed against the real game.** The `:mod`
> subproject build is deliberately **fail-closed**: `:mod:jar` (and
> `assemble`/`build`) cannot succeed until the items below are satisfied. This
> prevents a "successful" capture artifact from being produced without the
> actual pinned game.

## 1. Status: BLOCKED

Task 2 is **BLOCKED** on the following, which require the user's Windows
environment and the actual pinned Slay the Spire 1 build + compatible loader.
The portable, game-independent work is done (see §6 evidence); the reference
capture is pending.

> **Independent of the user evidence above**, the `:mod` build is also
> **unconditionally** blocked while a real reference-dependent capture
> integration is not implemented in this subproject (see §4). Filling a valid
> `reference-build.json` and supplying correctly-hashed jars does **not**
> produce a `:mod` artifact; neither does adding a dummy `.java` file. Only a
> future audited real adapter implementation, carrying actual reference
> evidence, removes that gate.

## 2. Prerequisites (user-provided, kept OUTSIDE this repo)

- A **pinned Slay the Spire 1** installation (Ironclad, Ascension 0).
- A **compatible pinned loader** (Mod The Spire generation).
- Two jar paths supplied as Gradle properties, both pointing **outside** the
  repository (never committed):
  - `-PgameJar=/absolute/path/to/slay-the-spire.jar` (or the platform game jar)
  - `-PmodTheSpireJar=/absolute/path/to/modthespire.jar`
- The **actual SHA-256** of each of those jars (recorded into
  `docs/validation/reference-build.json` with `status: "verified"` once
  confirmed).
- The **compatible runtime/compiler targets** for those jars (JVM/runtime
  version, and the compiler target the game runtime requires).

## 3. What the user must do on Windows

1. Build the capture mod against the pinned game:
   ```
   gradlew :mod:jar -PgameJar=<path> -PmodTheSpireJar=<path>
   ```
   This currently **fails fast (fail-closed)** listing exactly what is missing
   (see §4) until `reference-build.json` is verified and the external jars
   exist.
2. Install the built mod into the game and launch with the pinned loader.
3. At the pinned game's update boundary, passively **observe** (no action
   issued, no RNG consumed — passive capture must not replace mechanics or
   change effect order):
   - export **isolated reference RNG vectors** into `reference_rng/` (never the
     live run's streams);
   - capture a **short supported combat trace** (Ironclad A0; Strike/Defend/Bash
     base+upgraded; Cultist or Jaw Worm; Burning Blood; **no** potions, maps,
     events, or other content) into `captures/`;
   - record the **actual game/loader hashes** and the compatible
     runtime/compiler targets.
4. Verify **non-interference**: run the same controlled setup with capture
   **disabled** and **enabled**; the canonical states and the live RNG
   counters must be **identical**. Capture must not consume RNG or alter effect
   order.
5. Populate `docs/validation/reference-build.json`:
   - `status: "verified"`,
   - real `game_sha256` / `loader_sha256` (from the actual pinned jars),
   - real `runtime_target` / `compiler_target`.

## 4. Fail-closed readiness (the `:mod` build gate)

`capture mod readiness` is a **honest fail-closed predicate** that reports every
unmet requirement independently and **cannot be bypassed**. It does not rely on
a file-name, filename-count, source-presence, or boolean trick. In particular,
adding a dummy/empty/unrelated `.java` file does **not** unblock it (that was an
explicit defect, S3). Instead the gate is **unconditionally** blocked while
reference-dependent capture integration is pending.

Each unmet requirement is reported as its own `[MOD-READINESS]` diagnostic:

- the pinned game / loader jars must be present
  (-PgameJar / -PmodTheSpireJar), and the **actual digest of each supplied file
  is compared to the claimed SHA-256** (a hash-shaped string that does not match
  the real file is a failure);
- the supported `schema` must be present (e.g. `spire-lab/reference-build/1`);
- `status` must be `"verified"` **and** `verified` must be boolean `true`;
- `game` and `loader` objects must be present with nonblank version IDs and
  valid lower-case 64-hex SHA-256s; the flat `game_sha256` / `loader_sha256`
  must also be valid and **agree with the nested hashes** (a contradiction is
  an independent failure, not a silently-skipped agreement test);
- optional flat version aliases are checked by **key presence**, not by
  value-non-null: an absent `game_version`/`loader_version` is allowed, but a
  supplied value (an explicit `null`, a non-string, a blank string, or a value
  that disagrees with the nested `game.version` / `loader.version`) is rejected
  with its own specific alias diagnostic;
- `runtime_target` and `compiler_target` must both be a **canonical** Java
  feature version — a strict positive decimal integer **string of ASCII digits**
  (no letters, no fraction, no sign, no leading zero, no Unicode digits, no
  overflow) — missing, null, or wrong-typed targets are their own blockers;
- `compiler_target` must be inside the pinned local build compiler's actually
  supported `--release` set — `tools/jdk17`'s `javac` supports releases
  **7–17**; e.g. `compiler_target: "4"` or `"19"` is a named unsupported-release
  diagnostic, not a silently-accepted target;
- `runtime_target` must be at least the current `:protocol` bytecode release
  (**release 17**): the protocol is compiled to release 17, so a lower runtime
  is a named protocol-compatibility blocker requiring a later, evidence-backed
  protocol retarget — not a claimed compatibility;
- `compiler_target` must be ≤ `runtime_target` (bytecode cannot target a newer
  JVM than the game runs). A valid synthetic target does **not** establish game
  compatibility; it only establishes a *formally valid* reference target;
- the build honors the `--release` / build-JDK distinction: `JavaCompile`'s
  toolchain is pinned to the **build** JDK 17, while a structurally valid
  declared `compiler_target` binds `options.release` (the bytecode/API level)
  before any compile is permitted; invalid or missing metadata never silently
  defaults to ambient bytecode — the readiness gate fails first. The resulting
  binding can be inspected with `:mod:inspectCompileTarget` (reports
  `options.release` and the pinned toolchain) without weakening any gate;
- the **unconditional** gate: `reference-dependent capture integration is
  pending` — this subproject has no real, reviewed, reference-dependent capture
  implementation, so no `:mod` jar/assemble/build is produced. Removing this
  gate is part of a future audited real-adapter implementation with actual
  reference evidence, not a filename match, a boolean, or a source-count trick.

Because that last gate is unconditional today, `:mod:jar` / `:mod:assemble` /
`:mod:build` (and the compile step that would feed them) all fail; a source-less
or source-`Dummy` `:mod` build cannot emit an empty artifact. `compileJava` is
gated on readiness so no ambient-default compile happens while the gate is
open.

On the current pending reference (no user jars, `status:"pending"`,
`verified:false`, no capture integration), `gradle :mod:referenceReadiness`
fails with the specific diagnostics (exit code 1, no `mod/build/libs`):

```
> Task :mod:referenceReadiness FAILED

* What went wrong:
Execution failed for task ':mod:referenceReadiness'.
> capture mod readiness FAILED (fail-closed):
    [MOD-READINESS] game jar is not provided (-PgameJar not set)
    [MOD-READINESS] loader jar is not provided (-PmodTheSpireJar not set)
    [MOD-READINESS] status is "pending" (need "verified")
    [MOD-READINESS] verified must be boolean true (got false)
    [MOD-READINESS] game.version is required (a nonblank version ID)
    [MOD-READINESS] loader.version is required (a nonblank version ID)
    [MOD-READINESS] game.sha256 is not a 64-char lowercase hex SHA-256
    [MOD-READINESS] loader.sha256 is not a 64-char lowercase hex SHA-256
    [MOD-READINESS] game_sha256 is not a 64-char lowercase hex SHA-256
    [MOD-READINESS] loader_sha256 is not a 64-char lowercase hex SHA-256
    [MOD-READINESS] runtime_target must be a positive decimal integer string (got null)
    [MOD-READINESS] compiler_target must be a positive decimal integer string (got null)
    [MOD-READINESS] reference-dependent capture integration is pending — …
  Provide the pinned game/loader jars, a verified and internally consistent
  reference-build.json with real digests and real runtime/compiler targets, AND
  a real reference-dependent capture implementation in this subproject before
  any :mod artifact may be produced. See docs/validation/capture-guide.md.

BUILD FAILED
```

The focused suite under
`validation/windows-mod/tests/test_mod_readiness.py` drives the real Gradle
predicate in disposable project copies and asserts the *specific* diagnostic
for each requirement above, independently. It also:

- drives the actual `:mod:compileJava` / `:mod:jar` / `:mod:assemble` /
  `:mod:build` artifact paths with a **valid source-bearing** scaffold
  (`DummyCaptureScaffold.java`, filename matching its public class) and asserts
  the named integration blocker, **no compile execution / class output, and no
  jar** — compile gating is certified with a real source file, never with a
  NO-SOURCE fixture;
- proves compile-gate **regression sensitivity** in a disposable *mutant* copy
  where only `compileJava`'s `dependsOn(referenceReadiness)` edge is removed:
  the same harmless scaffold then compiles and produces its class, demonstrating
  that the gate — and nothing else — is what blocks the compile. The mutant
  lives only in the throwaway copy; the production blocker is never removed.

## 5. What stays OUT of the repo

- Game binaries, assets, and decompiled game source — **never committed**.
- External game/loader jars — referenced by path, never stored here.
- Unreviewed raw capture outputs — kept outside and only committed as reviewed
  fixtures after verification.

## 5a. Reproducible Java builds: strict dependency locking + SHA-256 verification

`:protocol` (and, transitively, the subprojects) build with **build-enforced**
dependency integrity, not a "call a BOM a lock":

- **Resolved-graph lock**: every configuration is locked and enforced in
  `LockMode.STRICT` (see `validation/windows-mod/build.gradle.kts`). The
  committed `validation/windows-mod/protocol/gradle.lockfile` pins the exact
  resolved coordinates (JUnit 5.13.4 BOM tree, opentest4j, apiguardian,
  junit-platform). A build whose graph does not match the committed lock
  **fails** — it does not silently re-resolve or upgrade.
- **Artifact SHA-256 verification**: the committed
  `validation/windows-mod/gradle/verification-metadata.xml` pins the SHA-256 of
  every resolved artifact. A build whose downloaded artifact does not match the
  recorded digest **fails** (SHA-256, not SHA-1/MD5).

**Normal locked use** is just running the build normally; the lock and the
verification metadata are always enforced — there is no lenient/off mode and no
silent rewrite. For example, with a toolchain JDK and an isolated user home:

```
gradle :protocol:test                       # enforces lock + SHA-256 verification
```

**Intentional lock/checksum update** is a deliberate, reviewed change — never a
default. To refresh the lock after a *reviewed* dependency change (e.g. a JUnit
upgrade approved separately), run:

```
gradle --write-locks --write-verification-metadata sha256 :protocol:test
```

…and then **diff** `protocol/gradle.lockfile` and
`gradle/verification-metadata.xml` against the committed files; only the reviewed
coordinate/checksums change. Do not run `--write-locks` casually — it will
re-record whatever the current repositories resolve and silently drift the
reviewed graph. The focused
`validation/windows-mod/tests/test_dependency_locking.py` demonstrates that a
deliberate graph change and a deliberate checksum change each **fail** the build.

## 6. Portable evidence already produced (Task 2 portable stage)

Implemented and tested (Linux; **not** a substitute for Windows/mod/runtime
evidence):

- Rust reader: `crates/spire-core` `spire_core::trace::read_trace` — 14
  protocol tests (schema, ordering, lossless hex RNG words, fingerprint chain,
  non-finite rejection, oversized limit, unknown-version rejection,
  missing-reference-build rejection, unsupported-state-field rejection) — pass.
- Java writer/coordinator: `validation/windows-mod/protocol` (`TraceWriter`,
  `CaptureCoordinator`, `BoundaryEvent`, `GameView`) — round-trip, numeric-loss,
  dedup-at-unchanged-boundary, passive (zero live-RNG calls) tests — pass.
- **Cross-language** round trip: a trace written by the Java `TraceWriter` is
  parsed by the Rust reader (`interop_java_written_trace_reads`, driven via
  `SPIRE_JAVA_TRACE`).
- Fail-closed `:mod:jar` (see §4).
- Toolchain (test-only, worktree-local, checksum-verified): JDK
  `17.0.20.1` (Temurin) and Gradle `8.14.6`.

## 7. Blocked items (must NOT be guessed or fabricated)

- The pinned game's actual RNG stream roster/algorithm — **pending** real
  capture; do not invent independent streams where the game shares one.
- A verified `reference-build.json` (real hashes + real runtime/compiler
  targets) — **pending**.
- `ReferenceRngExporter` real export (isolated reference RNG vectors) — **pending**;
  the coordinator intentionally never samples the live RNG.
- A real `CaptureMod` / `UpdatePatch` / `LiveGameView` / `ControlledSetup`
  implementation against the pinned game — **pending** the pinned game/loader.
- Enabled-vs-disabled non-interference verification — **pending** Windows run.

> `ControlledSetup` remains **disabled** unless explicitly selected, and any
> injected case must be labeled as injected.
