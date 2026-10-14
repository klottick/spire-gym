# Recovery index

Preparation index for the Task 2 split-track recovery: cluster ledger, accepted
invariants, acceptance tracking, and frozen identities. It is a documentation
index only. It performs no contract repair, no contract selection, and no
ratification; it invents no new rules and does not restate or re-derive the
bodies of I1/I6, D13, EP1, or C1.

All statuses below are copied from the cited recorded artifacts. Paths are
relative to this file and resolve within the repository.

## Status as read (grounding checklist)

- F1 / C2 / F3 / F4 / F5: CAP5/5 FROZEN UNACCEPTED
  ([cap ruling](../../../../.superpowers/sdd/2026-10-11-spire-lab-1a/task-2-contract-f5-options-cap-ruling.md),
  final line: "F1/C2/F3/F4 likewise stay CAP5/5 FROZEN UNACCEPTED").
- F5 residuals N1–N5: all Important, all PartiallyAddressed (cap ruling).
- C1: DRAFT — accepted draft only, not a freeze ([ledger](../../../../.superpowers/sdd/2026-10-11-spire-lab-1a/progress.md)
  lines 1109–1118: "ACCEPTED C1d+N1 and combined C1 F2/F7 DRAFT ONLY" …
  "this ledger is authoritative acceptance status").
- Full contract: UNRATIFIED; options: UNSELECTED (cap ruling, final line).
- F6: independent, still-open coverage assessment; not a capped repair loop
  (binding context; cap ruling).
- Task 2 incomplete; Tasks 3–16 unstarted (cap ruling, final line).
- All named artifacts were readable; no artifact was unreadable, so no
  citation-missing stop applies in this slice.

### Targeted sources read in this slice (no full-history audit)

- Cap ruling: full (90 lines) — guard IDs, credits, frozen-evidence table.
- Ledger `progress.md`: the `FINALCAP5/5` adjudication block (line 2516
  onward) and the C1 acceptance lines 1109–1118 only.
- C2/F3/F4 fix5 review files: verdict headers, finding dispositions, and
  preserved-credits blocks (each file's real conclusion header read as found).
- C1 fix1/fix2 reports: status headers and their recorded pending blocks;
  neither report was treated as final acceptance basis.
- Unratified proposal: status header, §2.4 EP-1 (line 111), §7.3 (lines 811–837).
- trace-v1: status block, §1–§2 framing, §6 fingerprint chain, §10
  (it has no D13/EP1 sections — confirmed from its section list).
- Slice-1b: `task-2-review-slice-1b.md` (I7/I8 verdicts), the follow-up-2 and
  follow-up-3 re-review verdict blocks, and commits `d213bb3` / `6b3b4ff`
  (message + file stat only, via read-only Git inspection).
- Approved design spec and 16-task plan: status lines and the Task 2 section
  of the plan only; phase-1a scenario boundaries from the spec.
- This plan's workspace: the task-1 brief, `binding-context.md`, and the
  approved recovery design spec (full, both short) — the only files of this
  slice consulted beyond the cited old-plan scratch.

## Cluster ledger

Unit | Cluster | Status | Frozen grounding (path) | Residual references (IDs) | Provisional deps (drafting names) | Ownership | Notes
---|---|---|---|---|---|---|---
C1 | C1 draft cluster | DRAFT (accepted draft only; not a freeze) | [ledger](../../../../.superpowers/sdd/2026-10-11-spire-lab-1a/progress.md) lines 1109–1118; [fix1 report](../../../../.superpowers/sdd/2026-10-11-spire-lab-1a/task-2-contract-c1-fix1-report.md) and [fix2 report](../../../../.superpowers/sdd/2026-10-11-spire-lab-1a/task-2-contract-c1-fix2-report.md) historical supporting evidence only | None recorded as frozen residuals; the fix1 (R1–R5) and fix2 (S1–S4) rounds keep their recorded pending statuses | None specified — provisional, to be resolved in briefs; not a selected architecture | Frozen material is citation, not owner; a selected definition belongs to the later user-selected brief | F2 is an accepted constraint covered by the combined C1/F2/F7 draft acceptance (ledger 1109–1118), not a repair cluster; the acceptance certifies neither process compliance nor collateral safety
C2 | C2/F1 options cluster | CAP5/5 FROZEN UNACCEPTED | [C2/F1 fix5 review](../../../../.superpowers/sdd/2026-10-11-spire-lab-1a/task-2-contract-c2-options-fix5-review.md) (Verdict: NeedsRevision; 1 Important, 0 Minor, 0 Critical) | F4-1–F4-4, F5-1 (F4-1/F4-2/F4-4 Addressed; F4-3 NotAddressed; F5-1 the remaining Important) | None specified beyond the shared C2/F1 review — provisional, to be resolved in briefs; not a selected architecture | Frozen material is citation, not owner; a selected definition belongs to the later user-selected brief | F5-1 is a portable guard defect, never relabeled BLOCKED (portable missing definition stays NeedsRevision)
F1 | C2/F1 options cluster | CAP5/5 FROZEN UNACCEPTED | cap ruling (final line); the C2/F1 fix5 review names C2 findings and records "F1 options remain unselected/open" | None — the cited review assigns no finding IDs to F1 (said exactly; not invented) | None specified — provisional, to be resolved in briefs; not a selected architecture | Frozen material is citation, not owner; a selected definition belongs to the later user-selected brief | F1-versus-C2 intra-cluster boundary is a deferred decision (recovery design spec §11)
F3 | F3 options cluster | CAP5/5 FROZEN UNACCEPTED | [F3 fix5 review](../../../../.superpowers/sdd/2026-10-11-spire-lab-1a/task-2-contract-f3-options-fix5-review.md) (Verdict: NeedsRevision; 2 Addressed, 3 NotAddressed; 3 Important consolidated groups) | S3 and S3 binding and S4 (NotAddressed, Important); N1 and D1 (Addressed) | Spec example "F3 before F4" — provisional, to be resolved in briefs; not a selected architecture | Frozen material is citation, not owner; a selected definition belongs to the later user-selected brief | None
F4 | F4 options cluster | CAP5/5 FROZEN UNACCEPTED | [F4 fix5 review](../../../../.superpowers/sdd/2026-10-11-spire-lab-1a/task-2-contract-f4-options-fix5-review.md) (Verdict: NeedsRevision; A Addressed; B/C/D/E NotAddressed; R2 preserved) | A (Addressed), B/C/D/E (NotAddressed — the four Important roots), R2 (Addressed/preserved) | Spec example "F3 before F4" and "read guards before byte emission" — provisional, to be resolved in briefs; not a selected architecture | Frozen material is citation, not owner; a selected definition belongs to the later user-selected brief | None
F5 | F5 options cluster | CAP5/5 FROZEN UNACCEPTED | cap ruling; frozen evidence in the Frozen identities below | N1–N5 (each VALID / Important / PartiallyAddressed) | Spec example "read guards before byte emission" — provisional, to be resolved in briefs; not a selected architecture | Frozen material is citation, not owner; a selected definition belongs to the later user-selected brief | N1 sole-enemy identity is a portable NeedsRevision defect (recovery design spec §7); genuine external inputs (B7/B10/B18, RNG, public observation) remain BLOCKED per the cap ruling and do not excuse portable omissions
F6 | Independent coverage assessment | Independent, still-open; not a capped repair loop | cap ruling ("F6 coverage assessment … still outstanding"); binding context | None assigned (open assessment; no finding IDs named — said exactly) | May run earlier as an independently scoped assessment; combined final F6 follows composition — provisional, to be resolved in briefs; not a selected architecture | Frozen material is citation, not owner; a selected definition belongs to the later user-selected brief | No coverage is claimed achieved; F6 reports its own gate

## Accepted invariants

Accepted invariant | Authoritative basis | Status | Preservation requirement
---|---|---|---
Binding framing + raw successor-chain semantics (fingerprint = SHA-256 of the full preceding line; strict record order) | [trace-v1](../../trace-v1.md) — status block and §6 (the document has no D13/EP1 sections) | Wire format implemented and round-trip-tested cross-language; reference trace still PENDING user evidence (status quoted from the file) | Stays as-is, not promoted, not rewritten by this plan's artifacts
D13 — end-record digest as a complete amendment | [ledger](../../../../.superpowers/sdd/2026-10-11-spire-lab-1a/progress.md) and cap ruling credit "exact authorized immediate C1 end payload"; [unratified proposal](../../../../.superpowers/sdd/2026-10-11-spire-lab-1a/task-2-canonical-contract-proposal.md) §7.3 (lines 811–837) | Approved binding direction; unimplemented; full proposal UNRATIFIED; no current parser implements D13; the cap ruling declined reopening D13 | Stays as-is, not promoted, not rewritten by this plan's artifacts
EP1 — end-digest precedence | unratified proposal §2.4 EP-1 (line 111: "end digest first") and §7.3 | Approved binding direction; unimplemented; full proposal UNRATIFIED | Stays as-is, not promoted, not rewritten by this plan's artifacts
C1 — combined C1/F2/F7 draft | ledger lines 1109–1118 ("ACCEPTED C1d+N1 and combined C1 F2/F7 DRAFT ONLY"; ledger authoritative); fix1/fix2 reports are historical supporting evidence only, each keeping its recorded pending status | DRAFT (accepted draft only); the acceptance certifies neither process compliance nor collateral safety nor full-contract ratification | Stays as-is, not promoted, not rewritten by this plan's artifacts
F2 — stable-profile vs per-run state constraint | same ledger lines 1109–1118 combined draft basis; proposal revision-C1 F2 note (status quoted there as part of the C1 draft) | DRAFT — accepted draft, not promoted | Stays as-is, not promoted, not rewritten by this plan's artifacts
F7 — exact Unicode-scalar string rule | same ledger lines 1109–1118 combined draft basis; fix1 report records R2/F7 as a separately accepted scoped draft (historical) | DRAFT — accepted draft, not promoted | Stays as-is, not promoted, not rewritten by this plan's artifacts
I1/I6 | cap ruling credit list ("Preserve I1/I6 …") | Preserved exactly as recorded; their bodies are not restated here | Stays as-is, not promoted, not rewritten by this plan's artifacts
I7 — honest fail-closed mod readiness | [slice-1b review](../../../../.superpowers/sdd/2026-10-11-spire-lab-1a/task-2-review-slice-1b.md) (I7 NOT ADDRESSED at head 51ee70d) and [follow-up 2](../../../../.superpowers/sdd/2026-10-11-spire-lab-1a/task-2-rereview-slice-1b-followup-2.md) / [follow-up 3](../../../../.superpowers/sdd/2026-10-11-spire-lab-1a/task-2-rereview-slice-1b-followup-3.md) reviews with commits `d213bb3` and `6b3b4ff` | Implemented and verified in a recorded slice; a status distinct from accepted-draft; not promoted | Stays as-is, not promoted, not rewritten by this plan's artifacts
I8 — durable strict graph locks / checksum verification | slice-1b review (I8 ADDRESSED at head 51ee70d); commits `d213bb3`, `6b3b4ff` record the later follow-ups | Implemented and verified in a recorded slice; a status distinct from accepted-draft; not promoted | Stays as-is, not promoted, not rewritten by this plan's artifacts
R5-core, R7-cost, finite Q/I/U/A bounds | cap ruling credit list ("Preserve … original R5-core/R7-cost and finite Q/I/U/A bounds") | Preserved exactly as credited; bodies not restated here | Stays as-is, not promoted, not rewritten by this plan's artifacts

## Acceptance tracking

Cluster | Last completed process step | Basis (artifact + recorded SHA256) | Contract status
---|---|---|---
C1 | Combined C1/F2/F7 draft acceptance (C1d+N1) — a process step recorded in the ledger | [ledger](../../../../.superpowers/sdd/2026-10-11-spire-lab-1a/progress.md) lines 1109–1118, the authoritative acceptance status; no file-SHA256 is cited for that entry, and none is computed or invented here; the fix1/fix2 reports remain historical supporting evidence only, each with its recorded pending status | DRAFT
F5 | Cap 5/5 final adjudication — a process step, explicitly not contract acceptance | [cap ruling](../../../../.superpowers/sdd/2026-10-11-spire-lab-1a/task-2-contract-f5-options-cap-ruling.md), recorded SHA256 there: final memo `425d8c8cbd83f15c76f4ae5c93e524d30b22259e1345b0ae2b48fe61696f96d6` (121 lines), final independent review `9ab0d320a52299b1b3d3bfca54475b87d4561d9c56b0de4f9bb2ab75166c1586` (300 lines), reviewed-before-fix4 memo `fb265f15b1a92f09a9902ada36f8bac3f8797050beca004c91a711a8f92c0bb9` (107 lines); identical values in the frozen-identities table below; not recomputed here | FROZEN UNACCEPTED

The process-step column never means contract acceptance or ratification.

### Gate summary (in order)

1. User scope / choices / budget.
2. Per-unit brief plus its own implementation plan (each user-selected).
3. Fresh review (fresh reader, inherited configuration).
4. Controller verification.
5. Per-candidate acceptance (explicit per candidate).
6. Full composition over the combined result.
7. Combined final F6 coverage assessment.
8. One explicit ratification of the composed portable contract.
9. A separate parser implementation plan, gated on the ratified contract.

Review of this plan is not an old repair round; no budget or selection is
implied by this index.

## Frozen identities (recorded, not recomputed)

Recorded by the cap ruling and cited here — not freshly computed in this
slice; the frozen memo bytes are unchanged.

| Frozen artifact (per cap ruling) | Local Git blob | Recorded SHA256 | Recorded lines |
|---|---|---|---|
| Final memo | `479a0ff74b2d1f333feb27e6c7a81db1933d7ce6` | `425d8c8cbd83f15c76f4ae5c93e524d30b22259e1345b0ae2b48fe61696f96d6` | 121 |
| Final independent review | `58c699ef38f0d6f6144799f26869341a782cef40` | `9ab0d320a52299b1b3d3bfca54475b87d4561d9c56b0de4f9bb2ab75166c1586` | 300 |
| Reviewed-before-fix4 memo | `6efc0fb0b8be32e822c009d742edbbd9e1878c23` | `fb265f15b1a92f09a9902ada36f8bac3f8797050beca004c91a711a8f92c0bb9` | 107 |

A Git object ID is never substituted for a file SHA-256; these recorded values
are cited verbatim from the cap ruling (also cited in the F5 acceptance-tracking
basis) and are not re-derived or recomputed in this slice.

## Named checks for the fresh reviewer / controller

- `acceptance-check-focus-b`: in this index the N1 sole-enemy identity entry is
  a portable NeedsRevision defect (F5 row, cluster ledger; recovery design
  spec §7), and no portable-defect row is labeled BLOCKED or missing artifact.
- `acceptance-check-status-purity`: CAP5/5 FROZEN UNACCEPTED, C1 DRAFT,
  UNRATIFIED, and UNSELECTED appear verbatim; there is no completeness
  claim, ratification claim, or coverage-achieved claim in this index.
- Global hygiene: fixed-string `grep -nF --` must find the heading
  `Recovery index` and all three exact column signatures above.
- This index cites no guessed IDs and adds no normative rule; the fresh
  reviewer's named verdict and the controller's re-run and adjudication
  are recorded by the controller, not here.
