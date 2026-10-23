# Offline recorded-run viewer

The optional `spire-report` crate packages **display JSONL**, not game snapshots,
into a self-contained read-only HTML slideshow. It never executes gameplay while
packaging or viewing. Existing gym interfaces and the `spire version` command are
unchanged.

Native capture uses the pinned accepted `5b1cd19` producer: supplied-entry Ironclad
A0 combat replay, then the accepted **M6 captured-content run prefix through floor
26**. It does not implement M7, full Act 2, standard content pools, later acts or
Heart runs. The viewer adds observation, not gameplay. All work is currently
uncommitted; integration into newer producer revisions is separate.

## Capture a native recording

Create an artifact directory first. Every trace and HTML destination must be new:

```sh
mkdir -p files/run-viewer/my-capture
cargo run -p spire-report --example capture_combat --locked --offline -- \
  --seed 7 --trace files/run-viewer/my-capture/combat.jsonl
cargo run -p spire-report --example capture_run --locked --offline -- \
  --seed 7 --budget 4096 --scenario prefix \
  --trace files/run-viewer/my-capture/run-prefix.jsonl
cargo run -p spire-report --locked --offline -- \
  --trace files/run-viewer/my-capture/run-prefix.jsonl \
  --output files/run-viewer/my-capture/run-prefix.html
```

The combat example creates an ordinary pool episode, then replays its accepted
actions through the shared native replay validator/transition. The run example
imports the unchanged M6 example caller (not a production policy), takes its actual
from-start action sequence, and records a separate identical native run while
checking every public decision, map and reward against that source. The selected
Act 2 icon is Champ; no seed or resource search is performed.

`capture_run --scenario death` records genuine gameplay Death. Budgets `0`/`1`
produce retained **unfinished** traces and a nonzero process exit; valid Death
exits successfully but is never labelled completion of the game. The finite budget
range is `0..4096`. Examples do not import arbitrary EpisodeJob JSON, snapshots or
Python reproduction dictionaries. They need neither a Python extension nor art.

The env seed is an unsigned integer represented losslessly in text. The M6 caller
places the requested u64's little-endian bytes in its native root prefix; this is
caller metadata, not an StS game-code conversion or Java RNG parity claim. Reports
do not expose roots, keys, cursors or unseen draw order.

## Package an existing display trace

```sh
cargo run -p spire-report --locked --offline -- --help
cargo run -p spire-report --locked --offline -- \
  --trace path/to/display.jsonl --output path/to/new-report.html
```

Open the HTML directly using `file://`; no web server, original game, Python
extension, network, browser storage or simulator endpoint is required. Previous/
Next, arrow keys, scrubber and context jumps navigate recorded frames only.
Recorded options are information, not gameplay controls. Arrow keys in editable
controls retain their native behavior. The jump menu changes context once.

Both capture status and gameplay outcome are displayed. A completed captured M6
prefix is not victory, full Act 2 or a finished run. A budget stop, interruption,
abandonment or operational fault is not inferred to be gameplay death. The accepted
native producer's actual limits are in [M6](milestones/m6-act2.md).

The display protocol is `spire-report-v1`, not the original-reference `trace-v1`
format, an EpisodeJob restore format or a Python reproduction-dictionary importer.
Seed text follows the declared caller/native encoding; it is not an original-game
seed-code conversion or Java RNG parity claim. Public adapters must exclude hidden
RNG state, unseen draw order, future actions and unoffered loot. Unknown values are
not zero, an empty inventory or invented intent hit counts.

## Optional original artwork

No artwork is shipped. Missing art remains a name/number label. Images and caches
must remain outside source control; Mega Crit Games owns original game artwork.
The small source catalog is incomplete and does not establish native content or
artwork coverage.

Explicit local preparation, using the already installed project interpreter:

```sh
/home/klottick/.local/share/uv/python/cpython-3.12.15-linux-x86_64-gnu/bin/python3.12 \
  tools/prepare_report_assets.py --local-dir /path/to/supplied/originals \
  --output-dir files/run-viewer/art
cargo run -p spire-report --locked --offline -- \
  --trace path/to/display.jsonl --output path/to/new-report.html \
  --assets files/run-viewer/art/manifest.json
```

Local imports use the original filename from each source URL. Missing originals
produce warnings. Acquisition from HTTPS sources is a separate, explicitly invoked
operation: use `--download --key card/bash` instead of `--local-dir`; downloading
requires named keys. No actual artwork download has been performed by this work.
There are no build-time, export-time or browser-time downloads. HTTPS redirects
cannot downgrade to HTTP. Cache preparation never overwrites different files or
follows destination symlinks; identical cached bytes may be reused.

The manifest contains `schemaVersion:1`, `assets:{key:{path,source}}` and an
`attribution` array. Paths are relative to the manifest directory. Traversal,
absolute paths, escaping symlinks, nonregular files, active SVG and invalid passive
image headers are rejected. PNG/JPEG/WebP inputs are limited to 16 MiB per image;
header screening is not full image decoding or artwork-authenticity validation.
Only referenced available images are embedded, deduplicated by exact bytes. Browser
decode failures fall back to labels.

## Safe output and interrupted traces

- Existing report/trace destinations, including dangling symlinks, are never
  overwritten. HTML is written to an exclusively created same-directory temporary
  file, flushed/synced and published with a non-clobber link. Concurrent destination
  creation makes publication fail rather than replacing that file.
- A failed HTML export leaves its original trace untouched. Temporary HTML cleanup
  removes only the owned temporary file. Failed traces are retained, not removed.
- Native recording distinguishes errors **before** an engine call from write or
  adapter errors **after** it. A post-call error must never be retried automatically:
  the native action may already have advanced. The recorder stops after such a
  failure and forbids a success footer. Isolated replay/export errors never mark a
  healthy live pool as gameplay-faulted.
- Before dropping a failed `RunRecorder`, call `take_recovery()` to retrieve its
  one undelivered action: detached before/after public decisions, map, committed
  transition (when accepted), resolved effects, combat boundaries and the complete
  display frame when adaptation reached that point. Adapter failure still retains
  the typed public evidence even if no frame could be built. No private snapshot,
  RNG or work queue is included. Taking evidence never re-enables stepping or a
  success footer; it is for explicit inspection/persistence, not action retry.
  Successful delivery clears this evidence rather than retaining a full history.
  Disk failure or a process crash cannot promise durable storage of unwritten data.
- Import defaults to a 64 MiB display-trace limit. Only newline-terminated complete
  records are retained. A torn final record or missing complete footer means
  interrupted capture; malformed interior records, inconsistent references and
  unsupported schemas are rejected. This does not relax `trace-v1`.
- JSON is inert and escaped for HTML. Rendering uses text APIs and passive embedded
  image URLs. The page's content security policy blocks external resources and
  connections. Untrusted data is never used as a template replacement source.

## Verification

```sh
cargo build -p spire-report --all-targets --locked --offline
cargo test -p spire-report --locked --offline
/home/klottick/.local/share/uv/python/cpython-3.12.15-linux-x86_64-gnu/bin/python3.12 \
  -m unittest discover -s tools/tests -p test_prepare_report_assets.py
cargo build -p spire-report --locked --offline
SPIRE_REPORT_BIN="$PWD/target/debug/spire-report" NODE_PATH="$SPIRE_PLAYWRIGHT_NODE_MODULES" \
  SPIRE_REPORT_GENERATED_DIR="$PWD/files/run-viewer/my-capture" \
  node "$SPIRE_PLAYWRIGHT_CLI" test -c crates/spire-report/tests/browser/playwright.config.cjs
```

Browser checks require an existing Playwright/Chromium installation, not a runtime
dependency. They open actual Rust-exported HTML through `file://` with network
disabled. `SPIRE_REPORT_GENERATED_DIR` must contain `combat.html`, `run-prefix.html`,
`death.html` and `unfinished.html`; generate/package the latter two using the death
scenario and budget1. Without that variable, generated-report browser checks are
explicitly skipped; fixture checks alone are not native-run evidence.

Installed Playwright1.64.1 was located at
`/home/klottick/repos/present-wishlist/frontend/node_modules/` for this session,
used read-only with the existing Chromium cache. No installation is implied.

The native recorder is bound to the Run's stable address between start/finish:
do not move or replace the Run mid-capture. Invalid actions add no frame; faults
retain actual effects with a labelled attempted action and last-known state.
Native current costs are recorded; unavailable base costs, free flags and intent
hit counts remain Unknown rather than being calculated in JavaScript. Only public
selection candidates and publicly exposed Stasis cards are included.

Manual disabled-overhead workload: `cargo bench -p spire-report --bench disabled
--locked --offline -- 1000 7`. The same std-only benchmark source is compiled against
base and candidate in separate builds; raw results are retained in the viewer
ledger directory. Its allocator instrumentation exists only in that benchmark
binary, never in simulator code. Measurements describe that supplied-combat
workload, not all gym throughput. Original-art verification remains unavailable
until original images are explicitly supplied/prepared; tiny generated test images
are not original-art evidence.

The pinned native base has seven inherited core failures involving unfinished
Poison/Catalyst/Artifact and Blue Slaver/Fungi drafts. The workspace is not green;
these are not repaired or hidden by the report tests. Focused passing report or
browser tests do not establish complete simulator support.
