# Windows reference capture — handoff guide (BLOCKED on user evidence)

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

## 4. Fail-closed evidence (already demonstrated on Linux)

`gradlew :mod:jar` (no user jars, `reference-build.json` status `pending`):

```
> Task :mod:referenceReadiness FAILED

FAILURE: Build failed with an exception.

* What went wrong:
Execution failed for task ':mod:referenceReadiness'.
> capture mod readiness FAILED (fail-closed):
    - user-provided -PgameJar is not set
    - user-provided -PmodTheSpireJar is not set
    - reference-build.json status is "pending" (need "verified")
    - reference-build.json game_sha256 is missing or not 64 lowercase hex
    - reference-build.json loader_sha256 is missing or not 64 lowercase hex
  Provide the pinned game/loader jars and a verified reference-build.json. See docs/validation/capture-guide.md.

BUILD FAILED in 2s
```

(Exit code 1; no artifact — `mod/build/libs` does not exist afterwards.)

No `:mod` artifact is produced. This is the required behavior: a successful
`:mod:jar` is impossible without the pinned reference data and the user's
external dependencies.

## 5. What stays OUT of the repo

- Game binaries, assets, and decompiled game source — **never committed**.
- External game/loader jars — referenced by path, never stored here.
- Unreviewed raw capture outputs — kept outside and only committed as reviewed
  fixtures after verification.

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
