# Evidence inventory request

Authorization request for the spec §6 evidence track only. Every identity,
tool, path, and scope mentioned here is a request to the user; this file
grants nothing. No game search, access, hash record, scan, download,
install, tool selection, or execution occurs here, and none is authorized
here. This is a preparation request only: it does not claim task 2 of the
16-task plan is complete, it does not waive any original-runtime gate, and
it does not select or install any tool.

## Status as read (grounding checklist)

- The user's current approval covers the four preparation documents under
  the approved method only — drafting, review, commit — not access, tool
  selection, budget, or any recovery task execution.
- The spec's own approval line authorizes recovery-plan drafting and review
  only; the current context approval is for this preparation document and
  the three sibling preparation documents — it grants no access.
- Spec §6: before any access, a user-provided and user-authorized path and
  an explicit scope are required; there is no game search and no access by
  inference. Reconstructed source and bytecode stay outside this repo, in a
  private sibling directory under the user's home repos tree, only if later
  approved; nothing is installed or downloaded now, and none by default.
- Spec §6: the identity record, when later taken under authorization,
  carries the game build/channel/platform, the artifact SHA256, the
  referenced class and member identities, the tool artifact SHAs with
  versions, and the RNG dependency/library source artifacts with versions.
  Recording those later does not authorize selecting, installing, accessing,
  or hashing any tool now.
- Spec §7 (status vocabulary): a missing genuine external input is BLOCKED,
  naming the artifact and the why. A portable missing definition is never
  relabeled BLOCKED; none appears in this file. Unsupported mechanics would
  be an honest error, never a fabricated value.
- Consumed from the [recovery index](recovery-index.md) (task 1 ledger,
  read-only): C1 DRAFT (accepted draft only); F1/C2/F3/F4/F5 CAP5/5 FROZEN
  UNACCEPTED; full contract UNRATIFIED; options UNSELECTED; ownership:
  frozen material is citation, not owner. Task 2 incomplete; tasks 3–16
  unstarted.
- Grounding: the [approved design spec](../../../superpowers/specs/2026-10-13-task-2-split-track-recovery-design.md),
  §6 (evidence track) and §7 (status vocabulary), read-only.

## Authorization request

The rows below are the four BLOCKED missing inputs in the spec §7
vocabulary: each names the required artifact and the why, and each
"Required from user" cell states only what the user supplies. No concrete
path, recorded hash, tool identifier, or storage location is asserted
anywhere in this file — only requests.

| Requested item | Current status | Required from user | Why (spec §6) |
|---|---|---|---|
| Exact path to the user's own `desktop-1.0.jar` game artifact | BLOCKED — no path provided; no search was performed | the user-owned exact path to that artifact | a user-provided, user-authorized path is required before any access; no game search and no access by inference |
| Inspection scope over that artifact | BLOCKED — no scope selected | an explicit scope selection; the six scoped-question areas below are the bounded set, nothing broader | bounded initial inventory and questions; no full recompilation and no cloning of the game engine |
| Tool names + versions (decompiler; inspection; execution), with the SHA256 of the exact tool-artifact bytes recorded later, under separate authorization | BLOCKED — no tool selected; nothing installed or downloaded | only the user's selection of each tool name + version, limited to that selection (no artifact, hash, or path supplied now) | the identity record includes the SHA256 of the actual tool artifacts used, with versions; this is not authorization to select, install, or hash any tool now |
| Exact private output path for reconstructed source/bytecode | BLOCKED — a suggestion only: sibling directory outside this Git tree under the user's home repos tree | the exact path (user-provided) of the private out-directory, if later approved | reconstructed source and bytecode stay outside this repo, only if later approved; no game binary, asset, or decompiled source is redistributed or git-added |

## Identity fields (awaiting the user's decision)

All rows below await the user's decision; nothing is selected, installed,
accessed, or hashed in this step.

| Field | Value |
|---|---|
| game build | PENDING USER AUTHORIZATION |
| game channel | PENDING USER AUTHORIZATION |
| game platform | PENDING USER AUTHORIZATION |
| game artifact (the user's exact `desktop-1.0.jar`) + its SHA256 | PENDING USER AUTHORIZATION |
| referenced class identities | PENDING USER AUTHORIZATION |
| referenced member identities | PENDING USER AUTHORIZATION |
| decompiler tool name + version + SHA256 | PENDING USER AUTHORIZATION |
| inspection tool name + version + SHA256 | PENDING USER AUTHORIZATION |
| execution tool name + version + SHA256 | PENDING USER AUTHORIZATION |
| RNG dependency/library source artifact name + version + SHA256 | PENDING USER AUTHORIZATION |
| private storage location (proposed: sibling directory outside this Git tree under the user's home repos tree; exact path user-provided) | PENDING USER AUTHORIZATION |

## Scoped questions (spec §6 bounded initial inventory)

Each probe is a minimal disambiguating proposal, shaped to its question
and keyed to later user-authorized inspection or observation; none was
executed in this step; none selects a tool, class identity, path, or
constant. If a later proposal hits an unsupported or unavailable
original-runtime input, it must name the exact input gap and request
separate scope — no fabricated scenario. Source inspection does not
establish runtime evidence.

| Area | Question | Minimal later-probe (proposal only) |
|---|---|---|
| RNG | Which named streams feed the randomness phase 1a consumes; which seed/counter words, bounded-sampling behavior, and shuffle behavior must be pinned by identity? | Later, under separate authorization: inspection of seed initialization and stream sharing, counter updates, the actual bounded-sampling and shuffle implementations, and their call sites; if ambiguity persists, a separately authorized isolated original-runtime primitive or shuffle observation recording actual before/after state, counters, and outputs, with no guessed expected words — proposal only, not executed in this step |
| Cards | What are the exact base and upgraded values for Strike, Defend, and Bash that closure requires? | Later, under separate authorization: inspection of the base and upgraded definitions and the effect creation/execution timing; if needed, a later supported play capture — proposal only, not executed in this step |
| Enemies | Which HP, moves, and AI state of Cultist and Jaw Worm does A0 closure need? | Later, under separate authorization: inspection of the HP, AI history, random-draw call sites, and intent publication; if needed, a later minimal supported turn sequence — proposal only, not executed in this step |
| Burning Blood | What are the effect's turn, cleanup, and terminal semantics? | Later, under separate authorization: inspection of the combat-end hook, heals, and outcome branches; if needed, a later before/after cleanup HP capture under separate consent — proposal only, not executed in this step |
| COMBAT turn cleanup | What is the ownership and timing of turn cleanup? | Later, under separate authorization: inspection of queue/hook ordering and the actual boundary scheduling; if needed, a minimal before/after end-turn capture — proposal only, not executed in this step |
| COMBAT terminal | What are the terminal mechanics and its owner? | Later, under separate authorization: inspection of the outcome/cleanup branches; if needed, a minimal supported victory/death capture — proposal only, not executed in this step |

## Evidence classes (spec §6 — four separate classes)

| Class | Definition (spec §6) | Promotion rule | Prohibition |
|---|---|---|---|
| Source-derived claims (spec §8: source-derived study) | what decompilation suggests, with identity | informs only; never replaces a runtime gate | never promoted to another class; never used as parity or readiness basis |
| Bytecode-confirmed inspection | bytecode-confirmed facts, with identity | own-class facts only | never promoted to execution or live evidence |
| Isolated original-bytecode execution | an isolated probe against original bytecode, with identity | no auto-promotion to full-combat or live-adapter evidence; corroboration remains allowed only as the other class's own evidence with its own identity | never treated as a live capture or full-game parity; never waives the Windows gates |
| Live original-runtime capture | the original game behaving as claimed, under its original runtime, with identity | basis for full-game parity/readiness promotion under the approved gates | never conflated with lower classes; only under the original-runtime gates |

## Gates and identity-conflict rule

- The approved task 2/3/7 Windows live, adapter, and non-interference gates
  are never waived by this file; they remain the prerequisite for
  full-game parity and readiness promotion. An authorized unit-level
  isolated RNG parity or probe does not itself require a full Windows
  handshake, and it does not satisfy those gates. An authorized isolated
  probe may test an isolated unit only; it may not test the live combat
  adapter or full readiness.
- Identity conflict (fail closed): the later identity-conflict matching
  covers the game artifact's byte hash, not only tool and dependency
  hashes. A user-supplied identity that changes or conflicts with a pending
  field or a known artifact stops the record for user re-identification; it
  is never marked compatible, and no pending-row check here or elsewhere
  simulates that behavior. No simulated proof is asserted now.
- Content-level conflict (fail closed; separate from the identity
  re-identification rule above): conflicting evidence content across the
  four classes — even when the record identities agree — stays
  unresolved, and no version or input is waived; no class promotion
  occurs until separately authorized corroboration resolves the conflict.
  Stated as documentation only; no tested behavior is claimed.

## CLI deferral

Tool CLI syntax is deferred until the actually approved tool version is
confirmed and its current documentation fetched. No CLI option, jar
invocation, or command line appears in this file.

## Named checks for the fresh reviewer and controller

- `acceptance-check-focus-d`: no scan, hash record, download, install, or
  game-access artifact appears in this file; all eleven identity rows carry
  the identical user-pending phrase (a count of exactly eleven is expected;
  the phrase is not re-spelled in this note); the four missing-input rows
  name the artifact and the why.
- `acceptance-check-focus-e`: the four class rows stay separate; the
  isolated row forbids auto-promotion while allowing same-class
  corroboration as that class's own evidence; the Windows gate paragraph,
  the identity re-identification rule, and the content-level conflict rule
  (unresolved, no version/input waived, no promotion even with matching
  identities) are documented as rules, not as claims of tested behavior;
  no row claims isolated output is live, and no row waives task 2/3/7.
- Global hygiene: fixed-string `grep -nF --` must find the heading
  "Evidence inventory request" and all four exact column signatures above;
  the eleven-row phrase count and the prohibited-token greps must pass,
  re-run by the controller; the prohibited tokens are not re-spelled in
  this file. The greps prove presence and absence of exact strings only;
  the two named checks are the semantic verification of what the content
  means.
