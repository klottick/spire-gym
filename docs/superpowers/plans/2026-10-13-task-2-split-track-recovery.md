# Task 2 Split-Track Recovery Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development
> (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use
> checkbox (`- [ ]`) syntax for tracking.

**Goal:** Four documentation-only preparation artifacts that put the spec's portable/evidence
split in a state for separately scoped, user-selected amendment work — without writing,
selecting, ratifying, or executing any contract content, and without completing Task 2.

**Architecture:** Preparation only: a status/invariants/ownership ledger (task 1); a READ-ONLY
executed-vs-planned coverage inventory over cited artifacts with byte-level SHA-256 and no fresh
execution (task 2); a decision packet proposing one smallest-scope candidate, F5 N1 continuity
seam — proposed, never selected (task 3); the evidence-track authorization request with pending
identity fields (task 4). Gates G1–G8 carry the spec's remainder: per-unit briefs and
implementation plans, fresh reviews, composition, final F6, one explicit ratification, then a
separate parser implementation plan.

**Tech Stack:** Markdown authoring and document review; repository-local Python 3
(`hashlib.sha256`) / `sha256sum` and Git for read-only file, link, and byte checks. No installs;
no generator/test/build/probe execution. Rust/Java is outside this slice.

**Spec:** [2026-10-13-task-2-split-track-recovery-design.md](../specs/2026-10-13-task-2-split-track-recovery-design.md)
— approved by user 2026-10-13 for recovery-plan drafting and review only.

**Status: DRAFT FOR USER PLAN REVIEW.** No task executed. Plan approval alone does not
authorize task execution: execution requires the user's explicit plan approval plus explicit
confirmation of the preserved Qwen-worker / fresh-reviewer / controller method. That method is
preserved as-is, not replaced by a native run; the next user handoff confirms the method
together with the plan review, and does not need to choose a new model. Contract scope,
concrete choices, budget, and access remain separate user gates, not implied by plan approval
or by method confirmation.

## Global Constraints

- F1/C2/F3/F4/F5: CAP5/5 FROZEN UNACCEPTED (F5 carries N1–N5, all Important, all
  PartiallyAddressed — [cap ruling](../../../.superpowers/sdd/2026-10-11-spire-lab-1a/task-2-contract-f5-options-cap-ruling.md)).
  C1: accepted draft only (DRAFT, not a freeze). Full contract UNRATIFIED; options UNSELECTED.
  F6: independent, still-open coverage assessment; not a capped repair loop.
- 938/19/35/55/35 are recorded executed diagnostic checks in the C1 slice (ledger:
  `ownedmain938` / `fix2probe19` / `fix1probe35` / `C1c55` / `NEWrecompute35`, all PASS) — not
  corpus coverage, parser implementation, or game parity. A retained generator/case file is not
  a retained executed run. The C1 69-schema inventory, 123 named cases, and 276 planned NK are
  an actual inventory plus a planned corpus, each cited separately.
- Claim kind (`diagnostic-check-run` / `schema-inventory` / `named-case-inventory` /
  `planned-NK-instance`) labels a row's proof strength; it is distinct from the spec §8 evidence
  class (which stays within the spec's matrix) and from executed test / corpus coverage.
- Missing external input: BLOCKED, naming the artifact and why. Portable missing definition:
  always NeedsRevision, never relabeled BLOCKED. Unsupported mechanics: honest error, never a
  fabricated value.
- The four spec §6 evidence classes stay separate: source-derived, bytecode-confirmed
  inspection, isolated original-bytecode execution, live original-runtime captures. No
  auto-promotion across classes; conflicts fail closed as unresolved; source-derived facts
  inform, never replace, the runtime gate.
- D13 (end-record digest as a complete amendment) and the EP1 post-end-schema self-digest
  precedence, plus the raw successor-chain semantics, are binding integrity rules. The binding
  framing and raw-chain grounding are in trace-v1 (whose historical implementation baseline
  lacks these amendments). The approved D13/EP1 directions are grounded in the ledger and the
  F5 cap ruling's credits ("exact authorized immediate C1 end payload") and in the UNRATIFIED
  canonical proposal (§2.4 EP-1, line 111; §7.3, lines 811–837). The whole proposal stays
  UNRATIFIED; no current parser implements D13; none is rewritten by these artifacts. Phase 1a:
  Ironclad A0 base and upgraded Strike/Defend/Bash vs Cultist/Jaw Worm, with Burning Blood;
  not unrestricted Act 1.
- Method (spec §7): workers `ollama/qwen3.8:27b`; fresh reviewers inherit the controller
  configuration; the controller verifies findings, routes fixes to Qwen, serializes
  review/staging/commits — no broader-writing exception. Budgets are user-selected per scope:
  never a sixth-or-later repair, never post-cap continuation, never an automatic reset; no brief
  may split the same work to reset an exhausted loop; reviewing this plan is not an old repair
  round.
- Hash semantics: recorded SHA256 = SHA-256 of exact file bytes as cited by a citing document;
  fresh SHA256 (this slice) = `sha256sum` / `hashlib.sha256` over the same bytes, read-only. A
  Git object ID is never substituted for a file SHA-256; cited and fresh SHAs stay in distinct
  columns.
- SDD records are controller-owned; no task below writes into them; no existing artifact, memo,
  or review is deleted or rewritten; old wire docs and product code are read-only. The
  [16-task plan](2026-10-11-spire-lab-1a.md) is unchanged: Task 2 incomplete, Tasks 3–16 unstarted.

## Review Focus

1. **Focus (a)** — draft attempt to reset a frozen cap or broaden scope: Task 3
   `acceptance-check-focus-a` — ruling and 5/5 quoted intact; budget PROPOSED; stops on
   split/continuation.
2. **Focus (b)** — portable missing definition mislabeled runtime BLOCKED: Tasks 1 and 3
   `acceptance-check-focus-b` — F5 N1 sole-enemy identity replacement stays a portable
   NeedsRevision defect, never a missing Windows/game artifact.
3. **Focus (c)** — diagnostic runs, inventory, or planned NK mispromoted as executed corpus
   coverage: Task 2 `acceptance-check-focus-c` — kind ≠ class; 938/19/35/55/35 stays a recorded
   diagnostic run (file-vs-run retention separated); 69/123/276 individually cited, none
   promoted.
4. **Focus (d)** — unapproved jar/tool/storage path: Task 4 `acceptance-check-focus-d` — ask,
   do not act; identity fields PENDING; no scan, download, install, recorded hash, or game
   access.
5. **Focus (e)** — swapped/changed game, tool, or dependency bytes, or isolated execution
   mislabeled live: Task 4 `acceptance-check-focus-e` — no auto-promotion (in-class
   corroboration allowed); the later identity-conflict matching must correctly request the
   game artifact's byte hash, not only tool/dependency hashes; identity change stops for
   user re-identification; Task 2/3/7 gates not waived.

## File map

Four new files under `docs/validation/recovery/2026-10-13/`; the directory materializes when
task 1's file is written (the patch creates it, not the commit). Each task commits only its own
path. Frozen/unratified material is cited, never owned by these files; a selected definition is
owned by the later user-selected per-unit brief.

| Task | File (create) | Responsibility |
|---|---|---|
| 1 | `docs/validation/recovery/2026-10-13/recovery-index.md` | Ledger, invariants, ownership, gate summary, tracking |
| 2 | `docs/validation/recovery/2026-10-13/f6-gap-assessment.md` | READ-ONLY coverage inventory, kind × class |
| 3 | `docs/validation/recovery/2026-10-13/portable-scope-requests.md` | One PROPOSED candidate; gates, budget, stops |
| 4 | `docs/validation/recovery/2026-10-13/evidence-inventory.md` | Authorization request; PENDING identities |

## Execution boundaries and dependency order

- Read the spec and this plan first. This worktree exists; create or move none.
- Per task: fresh Qwen writes only that file → fresh reviewer named verdict → controller
  re-runs the named checks, adjudicates, routes fixes to Qwen → conditional commit; the
  controller serializes review/staging/commit.
- Staging: `git add` the task's own path only; verify `git diff --cached --name-only` lists
  exactly it. A parallel worker's (task 3 ∥ 4) untracked file is expected: never stage it,
  delete, or revert it. An unexpected change to a tracked file no task declared: stop and go to
  the controller; workers never revert.
- Order: 1 → 2 → 3; task 4 depends on 1 only, parallel-eligible with 3 (disjoint paths).
  Commit: `docs: add <artifact> (Task 2 recovery preparation, Task N)`.
- Global documentation-hygiene check, reused in every task: for each required heading and
  each table's exact column signature, run the fixed-string command `grep -nF -- '<exact
  heading or column signature>' <task file>` (`-F` fixed-string match; deliberately no regex
  `|` alternation patterns) and confirm it matches; `git status --porcelain` shows only the
  expected paths. The grep proves presence of the exact strings only; each task's named
  checks remain the manual semantic verification of what the content means. Whitespace checks
  are optional diagnostics; `wc -l` and file-existence are not evidence of content.
- Nothing here executes a generator, parser, corpus, probe, build, test, or the game; no test
  scaffolding; no RED/GREEN or executed-behavior claim.
- This is a spec-approved plan-drafting slice: preparation execution awaits the user's plan
  approval plus explicit confirmation of the preserved method (Qwen-worker / fresh-reviewer /
  controller; preserved, not replaced by native). It is not execution of 16-task-plan tasks
  1–4, not a sixth-or-later F1–F5 round, and no new plan loop.

---

## Task 1: Recovery index (ledger, invariants, ownership, gate summary)

**Files:** Create `docs/validation/recovery/2026-10-13/recovery-index.md`. **Depends on:** none
(first task).

**Interfaces:**
- Consumes (read-only, targeted sections — no full-history reads; the SDD files named below
  are under `.superpowers/sdd/2026-10-11-spire-lab-1a/`): the spec (full); the cap ruling
  [`task-2-contract-f5-options-cap-ruling.md`](../../../.superpowers/sdd/2026-10-11-spire-lab-1a/task-2-contract-f5-options-cap-ruling.md) (full, 90 lines — guard IDs and
  credits verbatim); the [ledger](../../../.superpowers/sdd/2026-10-11-spire-lab-1a/progress.md) — the final F5 adjudication (search
  `FINALCAP5/5`) and the final C1 acceptance at lines 1109–1118 (`ACCEPTED C1d+N1 and
  combined C1 F2/F7 DRAFT ONLY`; "this ledger is authoritative acceptance status"); the
  F4/F3/C2 final review files `task-2-contract-f4-options-fix5-review.md`,
  `task-2-contract-f3-options-fix5-review.md`, `task-2-contract-c2-options-fix5-review.md` —
  each one's actual conclusion header (read the real one; it may be titled Assessment, not a
  generic adjudication section); the C1 fix1/fix2 reports
  `task-2-contract-c1-fix1-report.md` and `task-2-contract-c1-fix2-report.md` — historical
  supporting diagnostic/draft-correction evidence only (both keep their recorded pending
  status; neither is the final acceptance basis); the unratified proposal
  `task-2-canonical-contract-proposal.md` — status header, §2.4 EP-1 (line 111), §7.3 (lines
  811–837); [trace-v1](../../validation/trace-v1.md) — binding framing and the raw
  successor-chain sections only (it has no D13/EP1 sections); slice-1b reviews plus commits
  `d213bb3`, `6b3b4ff` (I7/I8 as recorded); the approved design spec and the [16-task plan](2026-10-11-spire-lab-1a.md) —
  status and Task 2 sections only.
- Produces (exact column signatures; document tables, not APIs):
  - Cluster ledger: `Unit | Cluster | Status | Frozen grounding (path) | Residual references (IDs) | Provisional deps (drafting names) | Ownership | Notes`
  - Accepted invariants: `Accepted invariant | Authoritative basis | Status | Preservation requirement`
  - Acceptance tracking: `Cluster | Last completed process step | Basis (artifact + recorded SHA256) | Contract status` — the process-step column never means contract acceptance or ratification.

- [ ] **Step 1: Targeted grounding pass.** Read only the named sections. Check: an unsaved list
  of the cluster statuses exactly as read — F1/C2/F3/F4/F5 CAP5/5 FROZEN UNACCEPTED; F5 N1–N5
  Important/PartiallyAddressed; C1 DRAFT; full UNRATIFIED, options UNSELECTED; F6
  independent/still open; Task 2 incomplete; Tasks 3–16 unstarted. Any unreadable artifact:
  record its exact path, stop for the controller; no inference.
- [ ] **Step 2: Cluster ledger.** One row per unit C1, C2, F1, F3, F4, F5, F6 (F2 in `Notes` as
  an accepted constraint, not a repair cluster); statuses verbatim. `Residual references
  (IDs)`: F5 = N1–N5 per the cap ruling; C2/F3/F4 = the finding IDs their fix5 reviews name;
  where a unit has no IDs, say exactly that — never invent. `Provisional deps`: the spec's
  examples, prefixed "provisional, to be resolved in briefs; not a selected architecture".
  `Ownership`: frozen material is citation, not owner; a selected definition belongs to the
  later user-selected brief.
- [ ] **Step 3: Accepted invariants table** (four columns above): binding framing + raw
  successor-chain (trace-v1, which has no D13/EP1 sections); D13 and EP1 (ledger + cap
  ruling credits "exact authorized immediate C1 end payload"; the UNRATIFIED proposal's
  §2.4 EP-1 (line 111) and §7.3 (lines 811–837) — approved direction, unimplemented, and no
  current parser implements D13; the cap ruling declined reopening D13); C1 (final
  acceptance: ledger lines 1109–1118 — `ACCEPTED C1d+N1 and combined C1 F2/F7 DRAFT ONLY`,
  ledger authoritative; `task-2-contract-c1-fix1-report.md` / `-fix2-report.md` are
  historical supporting evidence only, each keeping its recorded pending status — accepted
  draft only; the acceptance certifies neither process/collateral safety nor full
  ratification); F2/F7 (covered by the same ledger lines 1109–1118 combined draft
  acceptance; accepted draft, not promoted); I1/I6 (cap ruling
  credits — preserved exactly as recorded; their bodies are not restated here); I7/I8
  (slice-1b reviews + commits `d213bb3`, `6b3b4ff` — implemented and verified in a recorded
  slice, a status distinct from accepted-draft; not promoted); R5-core, R7-cost, finite
  Q/I/U/A bounds (cap ruling). `Status` copied verbatim from the basis file;
  `Preservation requirement` says only "stays as-is, not promoted, not rewritten by this
  plan's artifacts". No new normative rule is invented — in particular, do not invent the
  bodies of I1/I6, D13, EP1, or C1.
- [ ] **Step 4: Acceptance tracking + gate summary.** F5: "cap 5/5 final adjudication — a
  process step, explicitly not contract acceptance"; basis = cap ruling + Step 5's IDs; status
  FROZEN UNACCEPTED. C1: process step "combined C1/F2/F7 draft acceptance (C1d+N1)";
  basis = ledger lines 1109–1118 (authoritative acceptance status; the fix1/fix2 reports
  remain historical supporting evidence only); status = DRAFT.
  Gates in order: user
  scope/choices/budget → per-unit brief + its own implementation plan (each selected) → fresh
  review → controller verification → per-candidate acceptance → full composition → combined
  final F6 → one explicit ratification → separate parser implementation plan. Check:
  "acceptance" appears only in the C1-draft context and named gates.
- [ ] **Step 5: Record frozen identities verbatim** from the cap ruling, marked "recorded by
  the cap ruling and cited here — not freshly computed in this slice" (memo bytes unchanged):
  final memo blob `479a0ff74b2d1f333feb27e6c7a81db1933d7ce6`, SHA256
  `425d8c8cbd83f15c76f4ae5c93e524d30b22259e1345b0ae2b48fe61696f96d6`, 121 lines; final review
  blob `58c699ef38f0d6f6144799f26869341a782cef40`, SHA256
  `9ab0d320a52299b1b3d3bfca54475b87d4561d9c56b0de4f9bb2ab75166c1586`, 300 lines;
  reviewed-before-fix4 memo blob `6efc0fb0b8be32e822c009d742edbbd9e1878c23`, SHA256
  `fb265f15b1a92f09a9902ada36f8bac3f8797050beca004c91a711a8f92c0bb9`, 107 lines.
- [ ] **Step 6: Verification, review, adjudication.** `acceptance-check-focus-b`: the N1
  sole-enemy identity row is a portable NeedsRevision defect; no portable-defect row labeled
  BLOCKED/missing artifact. `acceptance-check-status-purity`: CAP5/5 FROZEN UNACCEPTED, C1
  DRAFT, UNRATIFIED, UNSELECTED verbatim; no completeness/ratification/coverage claim. Global
  hygiene: fixed-string `grep -nF --` confirms heading "Recovery index" and all three exact
  column signatures; semantics per the two named checks above. Fresh
  reviewer's named verdict; controller re-runs these checks, adjudicates (record states verdict
  and resolution).
- [ ] **Step 7: Conditional commit.** If accepted: `git add` only this file; verify `git diff
  --cached --name-only`; commit. As single writer: `git status --porcelain` empty afterward.

## Task 2: F6 coverage gap assessment (executed vs planned, READ-ONLY)

**Files:** Create `docs/validation/recovery/2026-10-13/f6-gap-assessment.md`. **Depends on:**
task 1 (committed index).

**Interfaces:**
- Consumes (read-only): task 1's ledger + Accepted invariants (C1 draft status); spec §2/§8;
  and evidence inputs under `.superpowers/sdd/2026-10-11-spire-lab-1a/task-2-contract-examples/`,
  bytes inspectable and hashable but never executed: `generate_and_check.py`,
  `diagnostic-profile.json`, `diagnostic-profile-shared.json`, `victory.jsonl`, `defeat.jsonl`,
  and `manifests/` with its four exact files `content-manifest.json`,
  `native-snapshot-manifest.json`, `rules-manifest.json`, `trace-schema-manifest.json`; plus
  the ledger count lines 858–859 (`36wire+33native=69schemas / 276NKplannedinstances, not
  executed tests`) and 868–872 (`69/276plannedNK, named 123; counted inventory NOT 276
  executed cases/product coverage`) — the 69/123/276 basis, read-only; and the
  938/19/35/55/35 claimed diagnostic-run PASS entries (lines 1061–1081; search
  `ownedmain938` / `NEWrecompute35`; no re-run), kept distinct from the counts. These SDD
  artifacts, not the `docs/validation/` file map, ground the rows; the fix2-report's pending
  status does not propagate into any "final" claim here.
- Produces (exact signatures):
  - Coverage inventory: `Claim | Claim kind | Recorded value | Evidence class (spec §8) | Artifact path | Cited SHA256 (from the citing document) | Freshly computed SHA256 (this step, bytes only) | Retained? | Proof note`
  - Gap summary: `Evidence class (spec §8, all eight rows) | Status (established / partial / not established) | Basis (cited artifacts) | Owner (later gate)`

- [ ] **Step 1: Read task 1 outputs.** Quote as context (not re-audit): F6 independent and
  still-open, combined final still required after composition; C1 DRAFT (accepted draft only);
  a retained file is not a retained executed run.
- [ ] **Step 2: Verify evidence inputs (read-only).** `ls` the six paths — the five top-level
  files plus the `manifests/` directory; `sha256sum` (or `hashlib.sha256`) the five top-level
  files and the four exact manifest paths (`content-manifest.json`,
  `native-snapshot-manifest.json`, `rules-manifest.json`, `trace-schema-manifest.json`),
  recorded "freshly computed in this step, bytes only, no execution". Any absent path:
  `Retained? = no`, proof note "unretained historical evidence; cannot be certified"; where
  the ledger-referenced run output (the PASS logs/reports) is not itself retained, record
  that the report is not retained executed-run evidence even if the generator/case file
  exists. Check: every enumerated path exists with a fresh SHA or is marked unretained; no
  retention from existence alone; no Git object ID in a SHA-256 column.
- [ ] **Step 3: Inventory — four individually cited rows.** 938/19/35/55/35 (kind
  `diagnostic-check-run`; recorded executed diagnostic checks, all PASS; basis: ledger PASS
  entries at lines 1061–1081 via `ownedmain938` / `NEWrecompute35`; class "executed
  protocol/corpus test" held as a recorded run; note "not corpus coverage; no fresh execution
  here"); 69 schema objects (`schema-inventory`; "contract review" as a static artifact;
  basis: ledger lines 858–859 and 868–872); 123 named cases (`named-case-inventory`;
  "contract review"; basis: ledger lines 868–872, `named 123`); 276 planned NK
  (`planned-NK-instance`; "planned synthetic case"; basis: ledger lines 858–859 — planned
  instances, not executed). Check: kind and class never substitute for each other; no "all
  coverage is complete" claim; the C1 draft acceptance (ledger 1109–1118) is not cited as
  coverage proof.
- [ ] **Step 4: Gap summary — one row per all eight spec §8 classes.** "Executed
  protocol/corpus test" = partial: the implemented portable baseline has recorded executed
  diagnostic runs (basis: cited ledger entries + example files); full Task 2 corpus coverage is
  not established and needs a separately approved corpus plan. "Parity" = not established:
  gated on original-runtime evidence, tasks 2/3/7 unchanged. Source-derived study,
  bytecode-confirmed inspection, isolated execution, live capture = not established (no
  retained basis). Check: "not established" never denies the portable baseline's recorded
  executions; no promotion.
- [ ] **Step 5: Verification, review, adjudication.** `acceptance-check-focus-c`: four kinds
  remain four distinct rows; the diagnostic row notes "executed diagnostic checks; not corpus
  coverage; no fresh execution"; planned-NK never classed as an executed test; inventory rows
  never classed as coverage. Global hygiene: fixed-string `grep -nF --` confirms heading
  "F6 gap assessment" and both exact signatures; semantics per `acceptance-check-focus-c`.
  Fresh reviewer's verdict (kinds/classes not conflated, no promotion, C1 draft
  preserved, retention grounded); controller re-reproduces Step 2's hashes read-only and
  adjudicates.
- [ ] **Step 6: Conditional commit.** Stage only this file; verify `git diff --cached
  --name-only` (a parallel worker's untracked file stays untouched); commit.

## Task 3: Portable scope request (user decision packet)

**Files:** Create `docs/validation/recovery/2026-10-13/portable-scope-requests.md`.
**Depends on:** tasks 1 and 2 (consumed read-only).

**Interfaces:**
- Consumes: task 1's ledger + Accepted invariants; task 2's inventory + gap summary; spec
  §4–§5; the cap ruling (no-continuation line, N1 adjudication).
- Produces (exact signature): decision packet —
  `Proposal | Frozen-residual grounding (IDs) | PROPOSED smallest scope | Material difference from the broad memo | Input owners (Task 1 Ownership column) | Output owner (later user-selected brief) | Outstanding concrete choices (questions) | Deliverable allowlist | Exclusions | Gates | PROPOSED budget | Stop rules`

- [ ] **Step 1: PROPOSE one unambiguous smallest-scope candidate.** Proposal: "F5 N1 — the
  length-1 sole-enemy identity continuity defect plus its associated safe-read obligations, with
  a single failure owner and a single diagnostic-emission order". Exclusions: the other N1 seams
  (roster rename, unique SR resolver dispatch, rank seams), all of N2–N5, every C1/C2/F1/F3/F4
  item. Where N1's safe read depends on a definition currently in N2 scope: an exact
  NeedsRevision selection question ("which unit owns it") — never new content. Check: exactly
  one proposed target; all seams enumerated; no predicate, rank, schema, byte, or ownership
  drafted.
- [ ] **Step 2: Material difference from the broad memo.** Not a restatement of Alt-A, but
  structural: (i) one unit, one failure owner, one emission order — the reviewer verifies a unit
  boundary instead of re-reviewing the whole memo; (ii) input ownership from task 1, output
  ownership the later user-selected brief; (iii) scope small enough that a failed review
  isolates one candidate. Check: the cell cites task 1's Ownership column as input and names
  the later brief as output owner.
- [ ] **Step 3: Outstanding choices as questions** (each "unselected; the later brief must
  choose", never a replacement predicate): which field path would own the continuity check;
  whether any numeric class (Q/I/U/A) is involved and where; blank-vs-absent for the identity
  field; the failure-emission order and unique owner; which known input and expected-byte
  counterexample are defined and required; Step 1's dependency-ownership question(s).
- [ ] **Step 4: Gates, PROPOSED budget, stop rules.** Gates: fresh review → controller
  verification → user separately approves scope + choices + budget → then (G2) the per-unit
  brief, its implementation plan, and execution authorization — each its own user-selected act.
  Budget, verbatim: "PROPOSED — effective only after the user separately authorizes this scope,
  its choices, and a budget: one initial draft plus at most TWO corrections, as a new amendment
  budget under a separately scoped, explicitly authorized amendment. F1–F5 round counters
  (including F5 5/5) are unchanged; this is not a sixth or seventh repair, not a continuation of
  an exhausted round, and not a reset; preparing documents under review budgets never opens an
  endless new plan loop." Stop rules, each to the user gate: budget exhausted without a
  user-selected follow-on; missing external input (BLOCKED, artifact named); ambiguous selected
  schema or boundary (NeedsRevision); dependency conflict; the work turns out to be a split of
  the same exhausted-loop defect or an old-repair continuation (unauthorized — stop the
  proposal). Check: the budget cell begins exactly as written; it appears nowhere else as
  committed.
- [ ] **Step 5: F1-versus-C2 boundary decision record.** If the scope later grouped F1/C2
  content, list the boundary in Outstanding choices as a required user decision, both
  candidate boundaries stated; no step resolves it; a decision record, not a TODO.
- [ ] **Step 6: Verification, review, adjudication.** `acceptance-check-focus-a`: cap ruling
  quoted, 5/5 unchanged, budget PROPOSED-only, split/continuation stop present.
  `acceptance-check-focus-b`: N1 a portable NeedsRevision defect, never BLOCKED/missing
  artifact; no cell drafts a repair, names a future brief's filename, or presents an unselected
  field/rank/owner as approved. Links actually used (`recovery-index.md`,
  `f6-gap-assessment.md`, the cap ruling, the spec) resolve. Global hygiene: fixed-string
  `grep -nF --` confirms heading "Portable scope requests" and the packet signature;
  semantics per its two named checks. Fresh reviewer's verdict; controller re-verifies,
  adjudicates.
- [ ] **Step 7: Conditional commit.** Stage only this file; verify `git diff --cached
  --name-only`; commit.

## Task 4: Evidence inventory request (user authorization packet)

**Files:** Create `docs/validation/recovery/2026-10-13/evidence-inventory.md`. **Depends on:**
task 1; parallel-eligible with task 3 (disjoint paths); controller serializes commits.

**Interfaces:**
- Consumes: task 1's ledger (Status, Ownership); spec §6/§7.
- Produces (exact signatures):
  - Authorization request: `Requested item | Current status | Required from user | Why (spec §6)`
  - Identity fields — exactly eleven rows, `Field | Value`, all values the literal
    `PENDING USER AUTHORIZATION`: game build; game channel; game platform; game artifact (the
    user's exact `desktop-1.0.jar`) + its SHA256; referenced class identities; referenced member
    identities; decompiler tool name + version + SHA256; inspection tool name + version +
    SHA256; execution tool name + version + SHA256; RNG dependency/library source artifact name
    + version + SHA256; private storage location (proposed: sibling directory outside this Git
    tree under the user's home repos tree; exact path user-provided).
  - Scoped questions — exactly six rows, `Area | Question | Minimal later-probe (proposal only)`:
    RNG (named streams, seed/counter words, bounded sampling, shuffle); cards (base/upgraded
    Strike, Defend, Bash — exact values); enemies (Cultist, Jaw Worm — HP/moves/AI state needed
    for closure); Burning Blood (effect semantics); COMBAT turn cleanup (ownership, timing);
    COMBAT terminal (mechanics, owner).
  - Evidence classes: `Class | Definition (spec §6) | Promotion rule | Prohibition`

- [ ] **Step 1: Authorization request.** Cells for the exact jar path, inspection scope,
  approved tool names + versions, exact private storage path; each `Required from user` states
  only what the user supplies. Check: no concrete jar path, recorded SHA, tool identifier, or
  private path in the file — only requests.
- [ ] **Step 2: Eleven identity rows**, all `PENDING USER AUTHORIZATION`; the RNG row carries
  the SHA256 inside the combined field (spec §6: identity "with versions" plus SHA). Check:
  `grep -c` of the literal returns 11; no 40/64-char hex string anywhere.
- [ ] **Step 3: Six question rows.** Each probe cell: the minimal disambiguating probe +
  "proposal only — not executed in this step"; no expected RNG values, no card/HP numbers, no
  fabricated constants. Check: `grep -niE "expected.?value|seed = |hp = "` empty.
- [ ] **Step 4: Class table + gate paragraph.** Promotion rules: source-derived — informs only,
  never replaces a runtime gate; inspection — its own class's facts; isolated execution — "no
  auto-promotion to full-combat or live-adapter evidence; corroboration remains allowed only
  as the other class's own evidence with its own identity"; live capture — basis for full-game
  parity/readiness promotion under the approved gates. Gate paragraph: the approved task 2/3/7
  Windows live/adapter/non-interference gates are not waived and remain the prerequisite for
  full-game parity/readiness promotion; an authorized unit-level isolated RNG or probe parity
  does not itself require a full Windows handshake, and does not satisfy those gates. Identity
  conflict: a user-provided identity — the later identity-conflict matching covers the game
  artifact's byte hash, not only tool/dependency hashes — that changes or conflicts with a
  pending field or known artifact stops the record for user re-identification; never marked
  compatible, and no PENDING-row check simulates that behavior. Check: all three named statements present.
- [ ] **Step 5: Missing inputs + CLI deferral.** One BLOCKED row per missing input (jar path;
  tool + version; private path; scope), each naming the artifact and why. One line: tool CLI
  syntax is deferred until the actually approved tool version is confirmed and its current
  documentation fetched; no CLI option, jar invocation, or command line appears in this file.
  Check: `grep -niE "java -jar|javap|unzip|sudo"` empty.
- [ ] **Step 6: Verification, review, adjudication.** `acceptance-check-focus-d`: no
  scan/hash-record/download/install/game-access artifact; all identity fields PENDING.
  `acceptance-check-focus-e`: four class rows separate; isolated row forbids auto-promotion
  while allowing in-class corroboration; Windows gate + identity-conflict rule present; no row
  claims isolated output is live or waives task 2/3/7. Global hygiene: fixed-string
  `grep -nF --` confirms heading "Evidence inventory request" and all four exact signatures;
  semantics per its two named checks. Fresh reviewer's verdict; controller re-verifies the Step 2/5 greps,
  adjudicates. (Approving this plan needs no user tool selection; that stays a later gate.)
- [ ] **Step 7: Conditional commit.** Stage only this file; verify `git diff --cached
  --name-only`; a parallel task-3 file may exist untracked — never stage, delete, or revert it;
  commit.

## Conditional Follow-On Gates (the spec's remainder, not a ready-software plan)

Each gate needs its own explicit user acceptance; nothing auto-inherits; unselected schemas,
ranks, fields, boundaries, or budgets are never derived, invented, or defaulted at any gate.

- **G1 — User scope/selection/budget before any amendment drafting.** Each artifact is handed
  to the user as soon as its own review and commit complete — not after all four; the task-4
  packet may be handed on its own review, and its track proceeds independently.
- **G2 — Per-unit amendment-task authorization.** After a selection, the user further authorizes
  the actual candidate-amendment task: the per-unit brief (filename allocated by that
  selection) plus its own implementation plan plus execution authorization. This plan stops
  before all three; no automatic code or brief. Per-candidate acceptance and the G4
  ratification are two distinct user acts, not two automatic ratifications.
- **G3 — Composition and final F6.** Authorized units in resolved dependency order →
  full-composition review → combined final F6 assessment, a new deliverable (task 2's inventory
  is input context only).
- **G4 — Single ratification.** One explicit act over the combined portable contract; it does
  not follow automatically from G2 or G3.
- **G5 — Parser implementation plan.** Only after G4, as a separate document with its own
  execution-method selection; this plan carries none beyond the four documentation tasks.
- **G6 — Original-runtime gates unchanged.** Task 2/3 per the 16-task plan: identity and
  reference evidence required, not inferred; Windows live/adapter/non-interference captures
  precede full-game parity and readiness promotion.
- **G7 — Evidence-track gates.** User-selected path, tool, private out-directory, scope before
  any inspection; each isolated probe needs its own execution consent; source-derived facts
  inform only; conflicts fail closed; identity change → user re-identification.
- **G8 — No derivation.** No gate selects, ratifies, budgets, or derives what that gate's user
  acceptance did not explicitly select.

**Plan exit success:** the four artifacts drafted, fresh-reviewed, controller-verified, committed
(one file per commit), and handed per G1. That is not Task 2 completion: Task 2 incomplete,
Tasks 3–16 unstarted, the 16-task plan unchanged, F1–F5 frozen, C1 a draft, the contract
unratified, options unselected, every original-runtime gate intact.

## Self-review

| Spec § | Covered by |
|---|---|
| §1 authorization | Goal; Status; G1 |
| §2 binding context | T1 steps 1, 3, 5 |
| §3 intent | T1 ledger; G8 |
| §4 alternatives | T3 step 2 |
| §5 portable track | T1 ledger; T3; G1–G5 |
| §6 evidence track | T4; G6, G7 |
| §7 vocabulary/budgets | Global Constraints; T1/T3 checks; G2 |
| §8 matrix | T2 kind × class; G3, G5, G6 |
| §9 sequencing | T1 step 4; G1–G8 |
| §10 prohibitions | Boundaries; all steps non-executing |
| §11 deferred | T3 step 5; T4 step 1; G1 |

Focus checks: (a) T3 · (b) T1 + T3 · (c) T2 · (d) T4 · (e) T4 — named
`acceptance-check-focus-*` per Review Focus.

Proportion (bytes, not line count): approved spec ≈15.3 KB; this plan ≈35.3 KB (~2.3× the
spec's bytes), all normal-length lines. No implementation bodies or test scaffolding;
concrete documentation tasks and deferred gates only.
