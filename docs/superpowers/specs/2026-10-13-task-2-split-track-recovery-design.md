# Task 2 split-track recovery design (portable closure vs original-runtime evidence)

Date: 2026-10-13
Status: DRAFT FOR USER REVIEW

## 1. Scope, status, and authorization

The user approved the recovery direction and the act of writing this architectural
design, but has NOT yet approved this written spec. Approval of this document
permits writing a NEW Task 2 recovery implementation plan only. It does not rewrite
or re-authorize the existing approved design spec or 16-task plan, both of which
stay unchanged. An approved plan plus an explicit execution choice remain separate
later gates.

This is a recovery workflow design, not a replacement contract, not an amendment
approval, and not correctness evidence. The only authorization in force now is to
draft, review, and commit this one document.

Separately scoped amendment selections and ratification, and source/runtime access
authorization, all remain explicit decisions. None of them is inferred from this
spec. The F1/C2/F3/F4/F5 CAP5/5 FROZEN UNACCEPTED clusters, C1 as an accepted draft
only, F6 as an independent open coverage assessment, and Tasks 3-16 status are
unchanged.

## 2. Binding context (referenced, not re-derived)

- Drafting-baseline commit `6b3b4ff` (the HEAD at drafting time; it will advance
  after this doc commit). No AGENTS.md was found in the current tree or ancestors.
- `docs/validation/trace-v1.md` provides the binding framing and raw predecessor-chain
  semantics. Unimplemented accepted-draft extensions keep a separate status.
- F1/C2/F3/F4/F5 remain CAP5/5 FROZEN UNACCEPTED (F5 carries N1-N5 Important residuals).
  The F5 cap ruling supersedes the worker's COMPLETE/READY claims and preserves the
  frozen memo bytes. C1 is an accepted draft only, not a freeze. The full contract is
  UNRATIFIED and options are UNSELECTED. F6 is an independent, still-open coverage
  assessment. Accepted C1/F2/F7/I7/I8 constraints stay as they are and are not promoted.
- D13 and the immediate post-end-schema EP1 self-digest precedence, plus the raw
  successor-chain semantics, are binding integrity rules this design must not touch.
- The existing 938/19/35/55/35 diagnostics are recorded executed recomputed checks,
  not parser implementation, not corpus coverage, and not game parity.
- The C1 69-schema-object inventory, 123 named cases, and 276 planned NK instances are
  an actual schema inventory plus a planned corpus; they do not prove executed corpus
  coverage.

Authoritative citations (read-only grounding, not owners of any new contract):
- Ledger: [progress.md](../../../.superpowers/sdd/2026-10-11-spire-lab-1a/progress.md)
- [F5 cap ruling](../../../.superpowers/sdd/2026-10-11-spire-lab-1a/task-2-contract-f5-options-cap-ruling.md)
- [F4 fix5 review](../../../.superpowers/sdd/2026-10-11-spire-lab-1a/task-2-contract-f4-options-fix5-review.md)
- [F3 fix5 review](../../../.superpowers/sdd/2026-10-11-spire-lab-1a/task-2-contract-f3-options-fix5-review.md)
- [C2 fix5 review](../../../.superpowers/sdd/2026-10-11-spire-lab-1a/task-2-contract-c2-options-fix5-review.md)
- [Unratified proposal](../../../.superpowers/sdd/2026-10-11-spire-lab-1a/task-2-canonical-contract-proposal.md)
- [Approved design spec](2026-10-11-spire-lab-ml-first-design.md) and [approved 1a plan](../plans/2026-10-11-spire-lab-1a.md)
- [trace-v1 framing](../../validation/trace-v1.md)

## 3. Intent

Unblock phase-1a Rust simulator and Python ML work without erasing any cap ruling
and without weakening the original-runtime gates. Phase 1a is Ironclad A0 base and
upgraded Strike, Defend, and Bash against Cultist and Jaw Worm, with Burning Blood.
It is not unrestricted Act 1. This is done by separating portable closure from
original-runtime evidence, instead of continuing broad memo repair loops.

## 4. Alternatives considered

A — Continue the broad memo (rejected). More rounds repeat the exhausted loop, stack
an unreviewed layer on the same base, and cannot reset a capped session. Rejected.

B — Wait entirely for the original-runtime capture (rejected). It stalls all portable
work on access, decompilation, and evidence that may be days away. Rejected as sole.

C — Split the tracks (recommended). Close what can be closed portably now, and gather
original-runtime evidence under an explicit, identity-bound gate. Each track keeps a
small, self-contained, fail-closed footprint. Recommended.

## 5. Portable track — small standalone amendment candidates

This is not the actual replacement predicates, schemas, ranks, or repair content. It
is a design for how later briefs will be cut. Concrete contract choices are
deliberately not prescribed here.

Each candidate is a small, self-contained amendment candidate with:

- Grounding and ownership: which frozen artifact it cites as grounding, which cluster
  residual it closes, and that a new candidate artifact owns any new definition
  (the frozen material is citation, not owner).
- Dependencies: the other candidates and accepted invariants it presupposes.
- Input guards: the field paths, numeric class (Q/I/U/A), and null-versus-absent
  meaning it uses, all chosen to fail closed.
- Diagnostic ordering: a single failure-emission order owned by exactly one candidate.
- Unique failure owner: every emitted failure has exactly one owner; no two candidates
  claim the same byte or owner.
- Counterexample and expected output: a known input and the exact bytes/digest when
  those are defined. A portable missing definition is always NeedsRevision (never
  relabeled BLOCKED). A genuine missing external input names the precise required
  artifact and why, without a fabricated value.

Mapping of later candidates to the frozen clusters (drafting names; the authoritative
mapping stays in the cluster rulings):

- F1/C2: continuation guards, stream and SR read scope, and effect/target ownership.
- F3: component and provenance manifests, self-descriptor, schema identity, and range.
- F4: inventory and history, canonical target/grammar, manifest entries, fingerprints.
- F5: guarded reads, diagnostics, exact bytes, terminal/cleanup ownership (N1-N5).
- F6: independently scoped coverage-gap assessment, kept separate and not a repair loop.

Do not claim all of these run independently. Cross-unit dependencies are provisional
examples to resolve in briefs, not a selected architecture (e.g., F3 before F4, or
read guards before byte emission). Candidates are ordered along resolved edges, then
a full-composition review runs over the combined result. F6 may run earlier as an
independently scoped coverage-gap assessment under existing authorization; the
combined final coverage assessment is required after composition.

The controller maintains the accepted-invariants and cross-unit ownership index and
adjudicates. It does not author the replacement implementation. Every future brief
needs user-approved scope, selection, and budget, and none may reset or extend an
exhausted loop by splitting the same work.

## 6. Evidence track — owned-game source/bytecode and original-runtime evidence

This track uses the user's exact Slay the Spire 1 desktop-1.0.jar. Before any access,
a user-provided and user-authorized path and explicit scope are required. There is no
game search and no access by inference.

Reconstructed source and bytecode stay outside this repo, in a private sibling
directory under the user's home repos tree, only if later approved. No game binary,
asset, or decompiled source is redistributed or git-added. Nothing is installed or
downloaded now, and none by default.

When identity records are taken later under authorization, record the game
build/channel/platform and its artifact SHA256, the referenced class and member
identities, the SHA256 of the actual decompiler and inspection/execution tool
artifacts used, and the relevant RNG dependency/library source artifacts, with
versions. This does not authorize selecting, installing, accessing, or hashing any
tool now.

Four evidence classes stay separate and are never conflated: source-derived claims,
bytecode-confirmed inspection, isolated original-bytecode execution, and live
original-runtime captures. Isolated execution cannot be elevated to full-combat or
live-adapter evidence. Any conflict among them fails closed as unresolved.
Source-derived facts may inform candidates but never replace the runtime correctness
gate.

Bounded initial inventory and questions:

- RNG: named streams, seed/counter words, bounded sampling, shuffle.
- A narrow card set (base/upgraded Strike, Defend, Bash) with exact values.
- The two enemies (Cultist, Jaw Worm) HP/moves/AI state needed for closure.
- Burning Blood effect, turn, cleanup, and terminal mechanics.

For each ambiguous behavior, propose the minimal probe that disambiguates it. There is
no full recompilation and no cloning of the game engine. On Linux, original-bytecode
probes may be possible, but only under explicit later scope and a matching artifact
identity. They do not replace the existing Windows live, adapter, and non-interference
gates without a separately approved change.

## 7. Error handling and status vocabulary

- Missing external input: BLOCKED, naming the specific required artifact and why.
- Portable missing definition: always NeedsRevision, never relabeled as BLOCKED.
- Unsupported mechanics: an honest error, never a fabricated value.

Brief illustration: the F5 N1 sole-enemy identity replacement is a portable
NeedsRevision defect even when Windows is unavailable; an unavailable authorized game
artifact is BLOCKED naming the exact required input. This is classification only, not
a new predicate, rank, schema, diagnostic byte, or frozen repair.

Worker and reviewer configuration:

- Qwen drafting workers run as `ollama/qwen3.8:27b`.
- Fresh reviewers inherit the controller configuration.
- The controller verifies valid findings and routes fixes back to a Qwen worker.
  There is no controller broader-writing exception.

Budgets and stop rules are explicitly selected by the user for each future scope:

- Bounded, assigned budgets with a stated stop rule.
- Never a sixth repair, never same-session post-cap continuation, never an
  automatic reset. A new spec review is not an old contract repair round.

## 8. Verification strategy and evidence matrix

Every claim is labeled by the evidence class it rests on:

- Contract review: a human reads the candidate against the frozen residuals.
- Planned synthetic case: a counterexample with expected bytes, not yet executed.
- Executed protocol/corpus test: a run with a recorded result (portable behavior).
- Source-derived study: what decompilation suggests, with identity.
- Bytecode-confirmed inspection: bytecode-confirmed facts, with identity.
- Isolated original-bytecode execution: an isolated probe, with identity; not
  full-combat or live-adapter.
- Live original-runtime capture: the original game behaving as claimed, with identity.
- Parity: the simulator matching the original on the same input.

No generator, parser re-emission, or corpus execution happens in this design step.
The 938/19/35/55/35 diagnostics are recorded executed recompute; the 69-schema,
123-case, and 276-NK set is an actual schema inventory plus a planned corpus, neither
of which proves executed corpus coverage. Later, reviewed exact bytes/digests and
test expectations establish portable behavior only. D13, EP1 self-digest precedence,
and the raw successor-chain semantics are preserved. Detailed current rules stay at
their authoritative docs/rulings, not rewritten here.

## 9. Handoffs, small deliverables, and sequencing

Small deliverables (intentionally few, no universal schema, no plugin system):

- This design document, the only output of the current step.
- One brief per portable candidate (scope/selection/budget/expected bytes), and one
  evidence-inventory file (questions/identities/minimal probes).
- The controller's accepted-invariants and cross-unit ownership index (adjudication).

Clear acceptance gates:

- A candidate is accepted only after fresh independent review, controller
  verification, and explicit user selection/ratification.
- Expected bytes and unique failure owners are design requirements the brief must
  define, not a requirement to implement parser emission before the contract is
  ratified. A portable missing definition is always NeedsRevision.
- Evidence records are accepted only within their own evidence class; mixed or
  elevated classes fail closed.

Sequencing (parallel tracks are optional once each has its own explicit
authorization; command/source/runtime work does not wait for full portable closure):

1. Now: this design document only.
2. After written-spec approval: a NEW Task 2 recovery implementation plan (not a
   rewrite of the existing, unchanged approved design spec and 16-task plan).
3. After recovery-plan approval and execution-method selection, in separately
   scoped briefs: portable candidates in resolved dependency order,
   then full composition and the combined F6 assessment. Parser implementation is
   gated on the ratified portable contract, full composition, and the relevant F6
   requirements.
4. Evidence track and Tasks 3/7 original-runtime gates, each under separately
   selected scope. Task 3/7 require original-runtime evidence, not inferred facts.

No portable or full Task 2 completion is asserted; each track reports its own gate.

## 10. Not authorized (explicit)

- No implementation, decompilation, or contract repair right now.
- No commit except the new design document, and only after review.
- No installs, test/build/probe execution, or game search/access.
- No nested agents, no generator execution, no model changes.
- No continuation of the F1-F5 exhausted rounds or a sixth repair. F6 is an
  independent coverage assessment, not a capped repair loop.

## 11. Deferred decisions and the next gate

User spec approval is the next gate; after it, a NEW Task 2 recovery plan may be
written. The following are deferred, to be selected only when their track proceeds,
and are not blockers to this spec's approval:

- Evidence-track sibling storage location and scope, when/if authorized.
- The F1-versus-C2 intra-cluster boundary, before candidate briefs are cut.
- Initial portable-candidate order and budget, when proceeding to briefs.
- The decompiler tool and version, for any later source/bytecode record.
