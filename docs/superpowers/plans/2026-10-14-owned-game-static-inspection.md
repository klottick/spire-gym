# Owned Game Limited Static Inspection Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development
> (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use
> checkbox (`- [ ]`) syntax for tracking.

**Goal:** Four serial, independently reviewed static-inspection tasks over the user-owned
`desktop-1.0.jar` copy producing private, hashable, evidence-class-labeled records (identity
binding, bounded class/member allowlist, RNG study, supported-combat study, combined evidence
report), never executing original game code and never replacing any original-runtime gate. The
tool use described (listing, entry-exact extraction, one-class decompilation, disassembly,
hashing) is the planned scope of a **future user-authorized** inspection, not an action taken by
this draft, which is planning-only.

**Architecture:** Spec §6 evidence track. Task 1 builds the controller-approved, entry-exact
allowlist (identity bindings, archive guard, seed lookups, entry digests; no class bodies). Each
later task extracts only its allowlisted class entries to its private raw-class root, then runs
CFR against ONE extracted class file (the original jar only as `--extraclasspath`
symbol-resolution input, never primary input) and `javap` under the shared retention policy.
Every claim carries exactly one of the four evidence classes with identity anchors (shared
`claim` fields); per-task `fingerprint` brackets gate acceptance.

**Tech Stack:** Pinned private tools by absolute path: Temurin JDK 21.0.12.1+1 (`java`, `javap`,
`jar`), CFR 0.152, host-local Python 3 **stdlib only** (`zipfile`, `hashlib`, `pathlib`) for
entry-exact read/extraction/hashing, `sha256sum`; Markdown evidence records. No installs or
downloads (python3 absent → STOP and report). Research/evidence plan, not a software
application: no tests, no corpus, no game-behavior assertions; static-inspection checks plus
fresh independent review are the checks.

**Spec:** [2026-10-13-task-2-split-track-recovery-design.md](../specs/2026-10-13-task-2-split-track-recovery-design.md)
§6 (evidence track), §7 (status vocabulary), §8 (evidence matrix); drafting authorization for
this plan: [brief.md](../../../.superpowers/sdd/2026-10-14-owned-game-static-inspection-planning/brief.md).

**Status: DRAFT FOR USER PLAN REVIEW (planning-only).** Drafting and review only are authorized
(2026-10-14). Nothing was executed and nothing is committed; the reviewed plan stays uncommitted.
Plan approval, preserved-method confirmation (Qwen worker `ollama/qwen3.8:27b`, fresh reviewer
inheriting the controller, controller adjudication — spec §7, unchanged), explicit inspection
execution consent, and per-task budget assignment are separate future user gates, none inferred
from plan approval.

## Global Constraints

**Constants (defined once here; every recipe and task inherits; no redefinition later):**
`ROOT=/home/klottick/repos/spire-game-private/`; `GAME=$ROOT/incoming/desktop-1.0.jar`;
`JDKBIN=$ROOT/tools/jdk/jdk-21.0.12.1+1/bin/`; `JAVA=$JDKBIN/java`; `JAVAP=$JDKBIN/javap`;
`JAR=$JDKBIN/jar`; `CFR=$ROOT/tools/cfr/cfr-0.152.jar`; `STAGE=$ROOT/inspection/2026-10-14-static/`.

**Tool identity (pinned from the accepted setup record, read-only grounding):**

| Tool | Path (absolute) | SHA256 (exact bytes) |
|---|---|---|
| java | `$JAVA` | `2a207f5e7d075afa01d97f8048389a64432a44c4a5af0f5e77d6e286ec5f401d` |
| javap | `$JAVAP` | `bd8fe7ac7eb91e0167766b6c37350fa96c4b601a5f086de8fa96d9f8f6a7b0dd` |
| jar | `$JAR` | `dca193d98d95aa0f14a785cf69f86187b042d10d69fa962428ea3d6427dd6649` |
| CFR (2,162,315 B) | `$CFR` | `f686e8f3ded377d7bc87d216a90e9e9512df4156e75b06c655a16648ae8765b2` |
| game copy | `$GAME` | `cfad868ac8d65a88e71a0bf096fb09f78811e553effe0787c5309a655e081673` |
| JDK archive (recorded) | `$ROOT/tools/downloads/OpenJDK21U-jdk_x64_linux_hotspot_21.0.12.1_1.tar.gz` | `ce79869e1307ed8ee1e2baa86a412b1eb5b75d10a01006d788a6f968bcfaee94` |

- Versions: Temurin `21.0.12.1` (build `21.0.12.1+1-LTS`); CFR `0.152`; inspection host
  `Linux x86_64`; game platform **user-reported** Windows — provenance fact, recorded separately
  from the Linux host, never conflated.
- CLI grounding: retained CFR help (accepted setup record) confirms `--innerclasses` (default
  true), `--clobber`, `--outputdir`, `--extraclasspath`, `--trackbytecodeloc`,
  `--methodname (string)`; official javap 21 docs (already consulted) ground this plan's
  usage — `-p`, `-s`, `-c`, `-v`, `-constants`, `-version`, `-sysinfo` (prints the class
  SHA), and the entry-selecting `jar:file:///…!/entry` source syntax. At execution, consult
  only this pinned retained help or the official public docs; no setup changes.
- Environment hygiene: every JVM invocation (including `-version` records) goes through the
  local-session-only `E` wrapper defined once in the shared policy (defined in the execution
  shell only — never a global config); all recipe paths quoted; explicit working directory
  under `$ROOT`; absolute constants only; the jar is never a launched main class.
- Windows source (user-reported verbatim from the brief):
  `C:\Program Files (x86)\Steam\steamapps\common\SlayTheSpire\desktop-1.0.jar` — recorded
  as user-reported provenance only; not a live capture; no Steam-channel/version proof is
  inferred and no game compatibility is inferred.
- Scope (user-selected, bounded): identity/hash + named class/member inventory (including one
  identified non-executing jar `META-INF`/manifest metadata read); RNG named streams,
  seed/counter init/consumption/sharing, bounded sampling, shuffle; Ironclad A0 base + upgraded
  Strike/Defend/Bash; Cultist/Jaw Worm HP/moves/AI history/intent; Burning Blood; relevant
  effect/queue/turn-cleanup/outcome timing; supporting classes only by verified reference in
  approved classes. Never: broad engine/content decompilation, whole-jar decompilation,
  game/probe execution, recompilation, installation changes, simulator/parser/training,
  frozen-contract repair, or any change to frozen contract schemas/terms. Static source/bytecode
  may inform later mechanics; never replaces the Windows live, adapter, non-interference, or
  Task 3/7 gates.
- Four evidence classes stay separate (source-derived; bytecode-confirmed; isolated
  original-bytecode execution; live original-runtime captures); this plan produces only the
  first two; `bytecode-confirmed` requires actually reviewed instructions from the retained
  bytecode artifact; conflicts fail closed as unresolved; no auto-promotion.
- Vocabulary: missing external input → BLOCKED naming the exact artifact + why (never guessed);
  embedded inspected field genuinely absent → recorded unobserved/BLOCKED (not an unhandled
  failure); inspected embedded facts recorded with provenance; identity mismatch / unsafe
  archive / unbounded scope → STOP, never labeled "compatible".
- Statuses unchanged: F1/C2/F3/F4/F5 CAP5/5 FROZEN UNACCEPTED; C1 accepted draft only; F6 open;
  contract UNRATIFIED; options UNSELECTED; Task 2 incomplete; Tasks 3–16 unstarted; never a
  sixth repair, post-cap continuation, automatic reset, or loop split; 5/5 counters untouched.
- This drafting step writes only this plan file: no staging/commit/publication/upload/push; no
  worker game/incoming/inspection access (not even stat, ZIP listing, or hashing); no SDD writes
  by workers (ledger is controller-owned); no nested agents; no download/install; setup/review
  records read-only; actual game sources unknown until authorized inspection; no game class is
  claimed observed — all source hashes remain unknown except those given here.
- Budget (PROPOSED, inert until per-task user assignment): per task, one initial pass + at most
  two correction passes; Task 4's budget covers its own composition; corrections that belong to
  Tasks 1–3 route to the owning task's remaining passes; at cap → STOP, report honestly
  (unresolved rows allowed; no fake PASS); a load-bearing identity/scope/provenance gap stops
  dependent work and hands off to the user.
- Outputs: the private inspection tree only (file map), plus (proposal only) one sanitized
  repository summary at `docs/validation/evidence/2026-10-14/static-inspection-summary.md`
  (provenance/ordinary facts/limits/hashes; never class files, source bodies, bytecode
  listings, game binaries/assets, tool distributions) — future proposal, not current write
  permission.

## Review Focus

Five checks, each pinned to the named static check of its owning task:

1. **(a) Artifact/tool mismatch** — Task 1 `T1-identity-bindings`: game-copy SHA + the three
   native executable SHAs (java/javap/jar) + the CFR jar SHA equal the pinned table;
   mismatch/absence → STOP, never "compatible"; re-pinned in each later task's `fingerprint`
   check (change → facts unaccepted).
2. **(b) Unsafe/duplicate archive entries; unsafe or escaping writes; scope or inner-class
   leakage** — Task 1 `T1-output-containment`: `guards` before any class read; `prewrite`
   task-root anchoring + private modes + exclusive create before EVERY raw/source/bytecode/
   report write; `extract-entry` exact-ZipInfo, digest-equal, contained, exclusive-create; CFR
   input exactly one extracted class file (or an approved same-name overload set scope-checked
   pre-invocation via `cfr-member`) into a FRESH per-class/per-group output leaf (never an
   existing dir); failure classification per `contain` — never on the
   study/report file's name.
3. **(c) Unknown dependency or build identity** — Task 1 `T1-identity-gaps`: inspected embedded
   facts with provenance; absence rows are CONDITIONAL on observed absence (build, channel,
   upstream source, RNG library source version) with exact input + why; no guessed values, no
   unconditional gaps; non-load-bearing absence does not prevent the narrow identity-bound
   study, but limits stay recorded and the full runtime identity gate is unmet; Windows
   platform vs Linux host distinct facts; absent seeds stay candidate-absent.
4. **(d) Conflicts across source/bytecode** — Task 4 `T4-class-separation`: exactly one
   evidence class per claim with the full anchor set; conflicts stay in an unresolved list;
   `bytecode-confirmed` cites actually reviewed instructions; no promotion, no relabel.
5. **(e) Static-only timing/conditional observations** — Task 4 `T4-static-only-boundary`: no
   runtime/parity/completion claims; ambiguous paths only as minimal probe proposals flagged
   "requires separate consent, not executed"; all Windows/Task 3/7 gates stated as not replaced.

Static checks may mechanically evaluate the actual trees (recording symlink/escape
findings before writes); greps never substitute for these checks; no helper
application, protocol corpus, or inspection software is implemented or tested.
## File map (roles)

Future writable private paths under `$STAGE`, created only by the owning task under consent;
member-bounded (area `shared`) sources/bytecode live in per-`groupkey` subdirectories:

| Task | Path (private, under `$STAGE`) | Responsibility |
|---|---|---|
| 1 | `identity/identity-record.md` | bindings (game + 3 native exe SHAs + CFR jar + JDK archive), environment, user-reported exact Windows source (provenance only), provenance vs BLOCKED/unobserved |
| 1 | `identity/class-inventory.md` | bounded inventory from entry listings + approved bounded seed lookups (names only) |
| 1 | `identity/allowlist.md` | shared allowlist (entry, FQCN, raw-class SHA, member tuples/WHOLE_CLASS basis, groupkeys, area/referrer/reason/approval); controller-serialized, append-only |
| 2 | `rng/rng-classes/` · `rng/rng-sources/` (per-class `<classkey>/` + per-group `<groupkey>/` leaves) · `rng/rng-bytecode/` (incl. `<groupkey>/` dirs) | raw class files · CFR output · retained bytecode selections |
| 2 | `rng/rng-study.md` | RNG claim records + ambiguity rows |
| 3 | `combat/combat-classes/` · `combat/combat-sources/` (per-class `<classkey>/` + per-group `<groupkey>/` leaves) · `combat/combat-bytecode/` (incl. `<groupkey>/` dirs) | same shapes, area combat |
| 3 | `combat/combat-study.md` | card/enemy/relic/order/cleanup/outcome claim records + ambiguity rows |
| 4 | `report/evidence-report.md` · `report/ambiguities.md` · `report/review-record.md` | combined report · consented probe proposals · named-check results, verdicts, per-task budget account |

Future sanitized repository summary (PROPOSAL ONLY — not current write permission):
`docs/validation/evidence/2026-10-14/static-inspection-summary.md`.

## Shared policy (defined once; tasks reference by name)

**`E` wrapper (I1) — the only fenced block in this plan; a minimal ambiguity-removing prefix,
not an inspection helper application; defined in the local execution shell only, never a
global config. All recipe paths are quoted; `-version` records also run through `E`:**
```
E() { env -u JAVA_TOOL_OPTIONS -u _JAVA_OPTIONS -u JDK_JAVA_OPTIONS "$@"; }
```
- **`prewrite`** (mandatory before EVERY raw-class/source/bytecode/report write): destination
  a component-aware descendant of the task root — the task root an approved lexical absolute
  path anchored inside `$STAGE`, which is bound to the approved `$ROOT`; the FULL ancestry
  from the filesystem root is checked component-by-component for unsafe symlinks/path
  substitution, but ownership + private-mode requirements apply ONLY to the explicitly
  controlled private containers: the existing approved `$ROOT` (recorded pre-existing and
  private, 0700; if not private → STOP and ask, never auto-alter), `$STAGE`, and the
  generated task dirs/files (new dirs created 0700, new files 0600, or stricter — enforced
  BEFORE game material is retained); trusted ancestors (`/`, `/home`, `/home/klottick`,
  `/home/klottick/repos`) are never chmod'd/modified by this plan and need no current-account
  ownership; an unsafe symlink/path substitution in ANY ancestor → STOP for user
  re-identification (never auto chmod/delete/truncate an unfamiliar or system path);
  exclusive create or verified read-only reuse (owned record + matching hash); no bare `>`
  over an existing file; CFR targets only FRESH pre-checked private directories — one per
  class/group leaf, never an existing dir; corrections write FRESH revision-leaf dirs/files
  under their own allocated task root — handoffs record the exact reviewed revision paths +
  SHA, old evidence is never overwritten, consumers read the selected reviewed version; the
  shared allowlist's authorized serialized append is the controlled exception (before/after
  hashes recorded).
- **`guards`** — before any class read: duplicate exact ZIP entry names; absolute, drive-letter,
  backslash, NUL, or parent-traversal entries; symlink or type-unsafe entries — checked via
  stdlib `ZipFile.infolist()` + `external_attr` **alongside** the `jar -tf` name listing (the
  listing gives names, not type attributes); any hit → STOP + anomaly row.
- **`extract-entry`** — entry-exact extraction, prescribed as an algorithm, not code: the
  controller-approved exact ZIP entry; exactly one `ZipInfo` (`guards` re-applied to the same
  entry set); destination built as the task's raw-class root joined with the exact package-entry
  relative class entry (independent exact file name per package entry, verified with resolved
  `Path.is_relative_to(root)` — never a substring/prefix check); every pre-existing parent
  component non-symlink and contained; entry hash equals the allowlist-bound `raw_class_sha256`
  (preserves that identity — the recomputed value matches, it does not replace, the bound
  digest); create exclusively (`open(path, "xb")` semantics — never overwrite); an existing
  destination is reusable ONLY as read-only owned record + matching hash, otherwise STOP (no
  auto-cleanup); `prewrite` applies to its destination and parents.
- **`cfr-whole`** — WHOLE_CLASS small in-scope roots only (documented reason in the allowlist);
  output dir ONE fresh, pre-checked, private leaf per class per revision (`<classkey>`) per
  `prewrite`:
  `E "$JAVA" -jar "$CFR" "<raw class file>" --extraclasspath "$GAME" --innerclasses false --clobber false --outputdir "<STAGE>/<area>/<area>-sources/<classkey>/"`
- **`cfr-member`** — shared support members only, per same-name overload GROUP (one approved
  call per group; group output dir keyed by that `groupkey`):
  PRE-INVOCATION (never post-only): from the approved Task 1 headers, obtain the COMPLETE
  same-name overload set for the selected name; the entire name-selected set must lie inside
  approved class/member/topic scope, otherwise STOP before decompilation (request bounded
  resolution); multiple approved same-name overloads may share that one call with the declared
  member set and the unique group output key (no overwrites); never assume one selected
  descriptor. Command: the `cfr-whole` command with `--outputdir` replaced by
  `--outputdir "<STAGE>/<area>/<area>-sources/<groupkey>/"` and `--methodname "<name>"` added —
  the ordinary method **name** verbatim (`<init>`/`<clinit>` per convention), never a full
  descriptor string; output dir fresh per `prewrite`. POST-OUTPUT (additional check): any
  emitted overload or member body outside the declared set → output violates scope: STOP, do
  not assume isolation; never a whole-class fallback.
- **`javap-root`** — WHOLE_CLASS roots only, direct retention allowed; input is the SAME
  hash-bound extracted class file (no game-jar classpath FQCN, no multi-release ambiguity);
  redirection only to a fresh pre-checked private file per `prewrite`:
  `E "$JAVAP" -p -c -s -v -constants "<raw class file>" > "<STAGE>/<area>/<area>-bytecode/<FQCN>.txt"`
- **`javap-member`** — shared members only, per group: capture the whole-class output
  transiently WITHOUT persisting it: `E "$JAVAP" -p -c -s -v -constants "<raw class file>"`
  (same hash-bound class file); retain ONLY the approved group's member blocks (directory key =
  that `groupkey`) plus the necessary declaring-class header/fields/constants under
  `"<STAGE>/<area>/<area>-bytecode/<groupkey>/"` (per `prewrite`); no unrelated method bodies
  retained; if an approved member block cannot be isolated unambiguously → STOP and request
  scope — never save the full class as fallback.
- **`fingerprint`** — `sha256sum "$GAME" "$JAVA" "$JAVAP" "$JAR" "$CFR"` (all values equal the
  identity record); the WORKER brackets its before/after actual reads; the REVIEWER brackets
  its own read-only verification reads; changed value → STOP and that actor's facts/findings
  unaccepted; the two brackets are separate actors, never conflated.
- **`contain`** — expected files = raw-class files, CFR sources (per-class `<classkey>/` leaves + per-group `<groupkey>/` dirs),
  retained bytecode selections, and the task's study/report Markdown under its `$STAGE` paths;
  FAIL = a CFR-emitted class outside the allowlist, or a symlink/path escape — never the
  study/report file's name.
- **`allowlist`** entry fields: `zip_entry` (exact); `fqcn`; `raw_class_sha256` (entry bytes;
  bound in Task 1; re-verified by `extract-entry`); `members` (approved canonical member
  tuples, grouped by name) or `WHOLE_CLASS` (documented reason; small in-scope root only);
  `area` (`rng`|`combat`|`shared`); `referrer` (seed or verified reference in an approved
  class); `reason`; `approved_by` (controller, before any tool body access). Member/group
  keys (chosen ONE byte encoding; no Unicode normalization): **`memberkey`** (per approved
  member) = SHA256 of the compact UTF-8 JSON array
  `["<FQCN>","<member name>","<JVM descriptor>"]` — the CANONICAL TUPLE (ordinary member name
  verbatim; constructor `<init>`; class initializer `<clinit>`); **`groupkey`** (per same-name
  overload group = one CFR call) = SHA256 of
  `["<FQCN>","<name>",["<desc1>","<desc2>",…]]` — ordered approved descriptor list, same
  encoding; **`classkey`** (per WHOLE_CLASS root class, per revision) = SHA256 of the exact
  FQCN (UTF-8) — one fresh output leaf per class per revision; the worker keeps the approved
  class → `classkey` → path mapping, and a distinct approved class mapping to an existing
  `classkey`/output path → STOP (no "collision-free" claim). The full selector names + exact
  descriptors remain in the allowlist as the
  human-readable authority, taken ONLY from approved headers — never guessed; if a distinct
  tuple/group ever maps to an already-existing key/output path → STOP (no "collision-free"
  claim).
- **Allowlist ownership** — the controller serializes and approves each addition
  (area/referrer/reason) first; the executing Qwen task then appends the approved entry to
  `identity/allowlist.md` — an explicit additional write permission limited to that file's
  appended entries, append-only, no overwrite of existing rows without controller instruction;
  the shared file's version/hashes per task are reported in handoffs. This is not automatic new
  topics and defines no contract ownership.
- **`claim`** record fields: `id`; `area`; `finding` (plain language; verbatim source bodies
  never enter the summary); `evidence_class` = exactly one of `source-derived` |
  `bytecode-confirmed`; `zip_entry`; `member` (canonical tuple FQCN + name + JVM descriptor, or
  the approved group set); `classkey` (WHOLE_CLASS) / `groupkey` (shared) for path naming; anchors (bytecode offset(s) and/or CFR
  source file + line, in
  the retained artifacts); retained artifacts with SHAs: `raw_class_file`(+SHA),
  `source_file`(+SHA) when source-derived, `bytecode_file`(+SHA) when bytecode-confirmed (with
  the actually reviewed instructions noted); `tool` (CFR jar SHA or JDK tool SHA, as used);
  `game_sha256`; `confidence`; `ambiguity` (minimal probe note → Task 4). No claim may cite
  unapproved bodies, members, overloads, or classes.
## Execution order, gates, method (shared)

- Serial tasks 1 → 2 → 3 → 4; each starts after its controller-approved gate.
- Per task: WORKER `fingerprint` (before) → worker writes only its own paths (per `prewrite`)
  → WORKER `fingerprint` (after) → fresh reviewer (inherits the controller config) VERIFIES
  READ-ONLY against the named checks, bracketing its own reads (no new outputs, no rewrites) →
  verdict → controller adjudicates, routes verified corrections to the Qwen worker within the
  owning task's remaining passes, serializes scope changes → handoff (status, paths + byte
  counts, exact reviewed revision paths/SHAs, shared allowlist before/after hashes, check
  outputs, gaps) persisted by the controller.
- Worker and reviewer fingerprint brackets are separate actors — never conflated. Workers never
  write SDD records, never stage/commit, never continue after cap, never auto-clean or modify
  files they do not own. Budget inert until the user assigns it per task; no old loop
  continuation; 5/5 counters untouched.

### Task 1 — Identity, archive guard, inventory, bounded allowlist

**Files (create, later under consent):** `identity/identity-record.md`,
`identity/class-inventory.md`, `identity/allowlist.md`.
**Consumes:** user-supplied game path + user-reported Windows hash/platform (facts, not live
captures); pinned tool paths/SHAs (setup record, read-only).
**Produces:** the three `identity/` files — bindings, provenance vs gaps, allowlist entries —
consumed exactly as approved by Tasks 2–4; BLOCKED/unobserved rows.

- [ ] **Step 1: Preconditions and identity bindings — `T1-identity-bindings` (focus a).**
  Verify this repo's tracked baseline clean; only owned expected untracked paths (this draft;
  controller administrative workspace) allowed; no auto-cleaning other work. `fingerprint`
  must equal the tool-identity table; record `E "$JAVA" -version`, `E "$JAVAP" -version`,
  `E "$JAR" --version`, host `Linux x86_64`, the user-reported Windows platform, and the exact
  user-reported Windows source path (Global Constraints) — provenance only. Record `$ROOT`
  pre-existing and private (0700); if not private → STOP and ask (never auto-alter).
  Mismatch/absence →
  STOP row; never "compatible".
- [ ] **Step 2: Archive guard — `T1-output-containment` (focus b).** Name listing
  `E "$JAR" -tf "$GAME"` (listing only; no extraction; no game class loading) plus `guards` via
  stdlib `ZipFile.infolist()`/`external_attr` (type/symlink, exact duplicates); record entry
  count + anomaly rows (expected: none); any hit → STOP before any selection or extraction.
- [ ] **Step 3: Bounded seed lookups (listing names only; no `javap`, no decompilation).**
  Candidates are name-level lookups from general context — presence UNOBSERVED, never claimed.
  Deterministic bounded set: Cards — Ironclad red folder only (no other characters): `Strike`,
  `Defend`, `Bash` base + upgraded variant names under the Ironclad character folder. RNG
  (stream owner + helper): EXACTLY two tentative entries —
  `com/megacrit/cardcrawl/random/Random.class` and
  `com/megacrit/cardcrawl/dungeons/AbstractDungeon.class`; no name filter, no fallback scan
  of any kind — either absent → candidate-absent row with the exact missing input recorded,
  and no alternative lookup until the user approves new scope. Enemies: `Cultist`, `JawWorm`
  (monster folder). Relic: `BurningBlood`. Present → candidate row (presence NOT claimed
  observed); absent → candidate-absent row with the exact missing input. Supporting
  dependencies: only as entries of the selected jar via verified
  references; an absent necessary dependency → BLOCKED row naming exact class/member + required
  external artifact + why (no outside filesystem scan, fetch, inference, or topic expansion).
- [ ] **Step 4: Controller approval gate.** Controller approves each exact class (area/
  referrer/reason + `approved_by`) before any header read or tool body access (allowlist
  ownership rule).
- [ ] **Step 5: Header reads and raw-class SHA binding.** Per approved candidate, define the
  EXACT-entry URI once via Python stdlib: `ENTRY_URI = "jar:" + Path(GAME).as_uri() + "!/"
  + urllib.parse.quote(zip_entry, safe="/")` (the class entry is already guarded and
  controller-approved; the URI example is the generic Oracle format
  `jar:file:///path/to.jar!/pkg/Class.class` — NOT an asserted game entry). Then the FULL
  header (private members + exact JVM descriptors) on that EXACT entry:
  `E "$JAVAP" -p -s -sysinfo "$ENTRY_URI"` (`-constants` optional if needed; `-p -s` headers
  only, no `-c` body code; NOT a classpath FQCN — classpath selection is not
  entry/multi-release disambiguated). Require exit 0; PARSE the class SHA printed by
  `-sysinfo` and compare it with the SHA256 computed by stdlib over the exact selected ZipInfo
  entry bytes (in memory, no write) — both must match, and the matched pair is bound as
  `raw_class_sha256` with BOTH observations recorded (identity metadata pre-approval; hash
  values are not game facts). Mismatch or unsupported output → STOP — no classpath fallback,
  no extra identity raw-class paths. Record the class descriptor + the COMPLETE member/
  overload sets as found — this ties the complete overloads to the approved entry; the
  approved headers feed the `cfr-member` pre-invocation set check.
- [ ] **Step 6: Embedded metadata and identity gaps — `T1-identity-gaps` (focus c).** Stdlib
  exact-entry read of `META-INF/MANIFEST.MF` (no extraction) and only further metadata entries
  the allowlist names. Facts OBSERVED → recorded with provenance (field, value, entry, game
  SHA); OBSERVED ABSENCE (MANIFEST/field absent; build, channel, upstream source, RNG library
  source version unavailable) → conditional BLOCKED/unobserved rows with exact input + why — no
  unconditional gap claims, never guessed, no Steam-channel/build proof or game-compatibility
  inference. Non-load-bearing absence does NOT prevent the narrow identity-bound study (limits
  recorded; full runtime identity gate unmet; no "compatible" label).
- [ ] **Step 7: Write the three `identity/` files** (all writes under `prewrite`) —
  bindings/platform/provenance-vs-gap records; count/anomaly/candidate rows; full `allowlist`
  entries (member tuples + groupkeys + classkeys bound) with controller approval per row.
- [ ] **Step 8: Fresh-reviewer VERIFICATION (read-only) + handoff.** The reviewer performs no
  writes and produces no tool outputs; it brackets its own artifact reads with `fingerprint`
  (before/after) and verifies the EXISTING three `identity/` files against
  `T1-identity-bindings`, `T1-output-containment` (guard + `prewrite`/`contain` state,
  symlink/escape findings recorded), and `T1-identity-gaps` — no re-run of producing steps; any
  replay on concrete risk is memory-capture only, no rewrite; named verdict returned.
  Controller adjudicates; verified corrections route to the Qwen worker within Task 1's
  remaining budget (fresh revision-leaf per `prewrite`); then handoff: status, paths + byte
  counts, exact reviewed revision paths/SHAs, named-check outputs, gaps.
### Task 2 — Static RNG study

**Files (create, later under consent):** `rng/rng-classes/`, `rng/rng-sources/`,
`rng/rng-bytecode/`, `rng/rng-study.md`.
**Consumes:** allowlist entries `area = rng` (+ approved `shared`) with bound
`raw_class_sha256`, member tuples + groupkeys + classkeys; `identity-record.md` bindings.
**Produces:** `rng-study.md` claim records (shared `claim` fields, one class each, anchored) +
ambiguity rows feeding Task 4.

- [ ] **Step 1: Fingerprint (before).** `fingerprint` values equal the identity record; change →
  STOP, no facts accepted (focus a).
- [ ] **Step 2: Extraction.** `extract-entry` per allowlisted entry into `rng/rng-classes/`
  (fresh private dir per `prewrite`); no other entries read or written; duplicate/absent entry
  or non-reusable existing destination → STOP per the shared algorithm (focus b).
- [ ] **Step 3: Decompilation, one invocation per selection/group.** WHOLE_CLASS roots:
  `cfr-whole` into its FRESH `rng/rng-sources/<classkey>/` leaf (one per class per revision;
  distinct-class mapping → STOP); shared: `cfr-member` per approved same-name overload
  group (pre-invocation set scope-check from the approved headers) into its `<groupkey>/`
  directory; post-output guard per the shared recipe (focus b); record exact command, exit
  code, emitted files per invocation.
- [ ] **Step 4: Bytecode confirmation.** `javap-root` (WHOLE_CLASS) or `javap-member` (shared,
  per group) on the SAME hash-bound extracted class files into `rng/rng-bytecode/` under the
  shared retention and STOP rules (focus b/d); `bytecode-confirmed` only where instructions
  were actually reviewed.
- [ ] **Step 5: Write `rng-study.md`.** Named streams, seed/counter init, consumption, sharing,
  bounded sampling, shuffle — each row a full `claim` record (descriptors, anchors, retained
  artifacts + SHAs, tool + game SHAs); source/bytecode conflict → unresolved row, no promotion;
  static-unresolvable paths → ambiguity rows, minimal probe proposals flagged "requires
  separate consent, not executed" (focus e); honest gaps, never guesses.
- [ ] **Step 6: Containment.** `contain` the actual `rng/` tree (symlink/escape findings
  recorded; failure per classification — never the study file's name).
- [ ] **Step 7: Fingerprint (after) + fresh-reviewer VERIFICATION (read-only).** Reviewer
  verifies the existing `rng/` outputs (labels/anchors, member retention, containment) with its
  own before/after read bracket; no new outputs; verdict; controller adjudicates.
- [ ] **Step 8: Handoff.** Status, paths + byte counts, exact reviewed revision paths/SHAs,
  named-check outputs, gaps.

### Task 3 — Static supported-combat study

**Files (create, later under consent):** `combat/combat-classes/`, `combat/combat-sources/`,
`combat/combat-bytecode/`, `combat/combat-study.md`.
**Consumes:** allowlist entries `area = combat` (+ approved `shared`) as in Task 2.
**Produces:** `combat-study.md` claim records (same `claim` fields) + ambiguity rows.
Steps mirror Task 2 with `combat/` paths and the same named checks/STOP rules:

- [ ] **Step 1: Fingerprint (before)** — same rule; change → STOP (focus a).
- [ ] **Step 2: Extraction.** `extract-entry` per allowlisted entry into
  `combat/combat-classes/` (fresh private dir per `prewrite`; focus b).
- [ ] **Step 3: Decompilation.** `cfr-whole` (roots) into FRESH
  `combat/combat-sources/<classkey>/` leaves / `cfr-member` per approved same-name overload
  group (pre-invocation set check) into `combat/combat-sources/<groupkey>/`, under the shared
  STOP rules; per-invocation record (focus b).
- [ ] **Step 4: Bytecode confirmation.** `javap-root`/`javap-member` (per group) on the SAME
  hash-bound extracted class files into `combat/combat-bytecode/`; member-bounded retention as
  in Task 2 Step 4 (focus b/d).
- [ ] **Step 5: Write `combat-study.md`.** As far as allowlisted code shows: base + upgraded
  Strike/Defend/Bash values and effect timing; Cultist/Jaw Worm HP, moves, AI history, intent
  publication; Burning Blood effect/turn/cleanup/terminal mechanics; relevant effect queue, turn
  cleanup, outcome timing. Each row a full `claim` record; a value not present in allowlisted
  code → honest gap row, never guessed (focus c semantics); unresolvable paths → probe-proposal
  rows flagged "requires separate consent, not executed" (focus e).
- [ ] **Step 6: Containment.** `contain` the actual `combat/` tree (as in Task 2 Step 6).
- [ ] **Step 7: Fingerprint (after) + fresh-reviewer VERIFICATION (read-only)** (as in
  Task 2 Step 7).
- [ ] **Step 8: Handoff** (as in Task 2 Step 8).
### Task 4 — Combined reviewed evidence report and proposal handoff

**Files (create, later under consent):** `report/evidence-report.md`,
`report/ambiguities.md`, `report/review-record.md`.
**Consumes:** Tasks 1–3 outputs in full (allowlist + versions/hashes, both studies, identity
record, all retained artifact paths + SHAs).
**Produces:** the combined report; the ambiguity/probe list; the sanitized summary PROPOSAL
(content only — no repository write); the handoff report the controller persists.
**Budget:** Task 4's own initial + ≤2 corrections cover its composition; corrections belonging
to Tasks 1–3 consume those owning tasks' remaining passes.

- [ ] **Step 1: Fingerprint (before) + composition checks.** Every Task 2/3 claim cites
  allowlist-approved `zip_entry`/`member` (canonical tuple), bound `raw_class_sha256`, retained
  artifact paths + SHAs, tool SHA, game SHA; stale/missing citations → defect rows routed for
  correction within budget.
- [ ] **Step 2: `T4-class-separation` (focus d).** Exactly one evidence class per claim; classes
  never conflated; `bytecode-confirmed` each cite actually reviewed instructions from the
  retained bytecode artifact; every conflict sits in an unresolved list; no promotion, no
  relabel.
- [ ] **Step 3: `T4-static-only-boundary` (focus e).** No runtime behavior, parity, or
  Task-2/3-completion claims; `ambiguities.md` lists every ambiguous path with a minimal probe
  proposal flagged "requires separate consent, not executed"; Windows live/adapter/
  non-interference and Task 3/7 gates stated as not replaced.
- [ ] **Step 4: Sanitized summary proposal.** Inside `evidence-report.md`, draft the exact
  intended content of `docs/validation/evidence/2026-10-14/static-inspection-summary.md`:
  provenance, ordinary facts, limits, hashes only; confirm no class files, source bodies,
  bytecode listings, game binaries/assets, tool distributions; no repository write — any later
  one needs separate user consent + controller staging/commit.
- [ ] **Step 5: Fingerprint (after) + composition VERIFICATION (read-only).** Fresh reviewer
  verifies Steps 2–3 over the EXISTING combined report and all private outputs (re-verifies
  `contain` across all four task trees), bracketing its own reads; no rewrites; verdict;
  controller records verdict, per-task budget account, unresolved gaps in `review-record.md`
  (written by the worker's correction path where needed, per `prewrite`).
- [ ] **Step 6: Handoff.** Per-task status; paths + byte counts; exact reviewed revision
  paths/SHAs; named-check outputs; unresolved gaps; per-task budget account; explicit: nothing
  committed, no original game code executed, no probe ran, no installation occurred;
  load-bearing gap → stop dependent work, hand off to the user.

## Self-review

- **Spec + review coverage (I/M mapped):** §6 identities → Task 1 (game + three native exe SHAs
  + CFR jar + JDK archive [M1]; user-reported exact Windows source path (provenance only) [I7];
  distinct platform records; manifest provenance vs conditional BLOCKED/unobserved rows [I7];
  bound entry digests); bounded seed set — cards Ironclad red folder only, no other
  characters; RNG = EXACTLY two tentative entries, absent → candidate-absent, no fallback
  scan / alternative lookup [I6] → Task 1 Step 3; exact-entry FULL headers via `ENTRY_URI` +
  `-sysinfo` class-SHA tie to exact entry bytes [I5] → Task 1 Step 5 and the `cfr-member`
  pre-invocation overload-set check [I3]; canonical tuples / groupkeys, per-class `classkey`
  output leaf (distinct-class mapping → STOP, no collision-free claim) [I4]; RNG → Task 2; cards/enemies/relic/cleanup/terminal → Task 3; four classes,
  fail-closed conflicts, consented probe proposals → Tasks 2–4; §7 vocabulary → Task 1 Step 6 +
  per-task gap rows; whole-jar excluded by `extract-entry` + single-file CFR input + member
  isolation; safe writes via `prewrite` (all writes, private modes, exclusive create, revision
  leaves) [I2]; production vs read-only review separation with separate worker/reviewer
  fingerprint brackets [I8]; `E` wrapper quoted paths incl. versions [I1]. No requirement
  without a task; no runtime-class claims.
- **Step scan:** one action + one checkable outcome per step (pinned values, shared-policy
  names + STOP conditions, expected states); absent seeds/values → candidate-absent or gap
  rows, never game facts; unsupported selectors / non-isolable members → STOP for scope, never
  widening fallbacks or overwrites; no bare `>` over existing files; corrections = fresh
  revision-leaf; reviewers never write or produce outputs.
- **Interface consistency:** shared names (`E`, `prewrite`, `guards`, `extract-entry`,
  `cfr-whole`, `cfr-member`, `javap-root`, `javap-member`, `fingerprint`, `contain`, `allowlist`,
  `claim`, `memberkey`/`groupkey`/`classkey`) and the constants are defined once and referenced identically
  in every task; the five named checks are spelled identically in Review Focus and owning steps.
- **Proportion [M2]:** spec 267 lines / 15,297 bytes; this draft's final measured line/byte
  counts and ratio are reported in the drafting handoff. No giant table rows; no helper
  application bodies, no new schemas — the only fenced block is the minimal `E()` wrapper
  (justified to remove ambiguity, not a helper app); no copied schemas, no corpus, no audits.
- **Authorization:** drafting only writes this plan file; per-task budgets stay PROPOSED/inert;
  user plan review, preserved-method confirmation, per-task budget assignment, and execution
  consent all remain distinct future user gates; 5/5 caps, C1 draft, F6 open unchanged; no old
  loop reset; the repository summary stays a private proposal only; no game class claimed
  observed.
