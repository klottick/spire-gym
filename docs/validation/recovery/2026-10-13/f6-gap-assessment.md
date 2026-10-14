# F6 gap assessment

Recovery preparation Task 2 — READ-ONLY coverage inventory (executed vs planned).
Preparation only: no contract content is written, selected, ratified, or executed here;
Task 2 itself is not completed. No generator, probe, corpus, build, test, or game
execution happens in this step: the only byte-level operations are read-only
`sha256sum` reads over file bytes. Every status below is quoted as recorded from the
cited artifacts (historical record), not re-validated now.

Path base used below: `<BASE>` = `.superpowers/sdd/2026-10-11-spire-lab-1a/`.
Citing register: `<BASE>task-2-contract-c1-fix2-check/final-register.sha256`
(the fix2-era final register that cites the current live companion bytes).

## Context (quoted from the task-1 index and the cited records; not re-audited here)

- F6 is independent and still-open; it is not a capped repair loop; the combined final F6
  follows composition and remains future (gate 7, [recovery index](recovery-index.md)
  status block and gate summary).
- C1 = DRAFT (accepted draft only, not a freeze; ledger lines 1109–1118). The recorded
  draft acceptance is a process step and is NOT cited anywhere below as coverage proof.
- A retained file is not a retained executed run: the nine retained input artifacts
  (appendix A1–A9) are listed separately from the five retained run outputs (A10–A14).
- 938/19/35/55/35 are recorded executed diagnostic checks of the C1 slice — not corpus
  coverage, not parser implementation, not game parity. 69/123/276 are a counted schema
  inventory, a counted named-case inventory, and a planned NK set — each cited
  individually in rows R2–R4, none promoted.

## Coverage inventory

Claim | Claim kind | Recorded value | Evidence class (spec §8) | Artifact path | Cited SHA256 (from the citing document) | Freshly computed SHA256 (this step, bytes only) | Retained? | Proof note
---|---|---|---|---|---|---|---|---
R1 record executed diagnostic checks: ownedmain938 / fix2probe19 / fix1probe35 / C1c55 / NEWrecompute35, all PASS exit 0 (C1 slice) | diagnostic-check-run | 938 / 19 / 35 / 55 / 35 PASS, recorded (ledger lines 1061–1062, 1081–1082; fix2-report covering-run block); not re-run here | Executed protocol/corpus test (recorded run; diagnostic-only proof limit — not full corpus, not parser, not game) | `<BASE>task-2-contract-c1-fix2-check/`: run-fix2-main.log, run-fix2-green.log, run-fix1-probe-fix2.log, run-c1c-probe-fix2.log, fix2-recompute.log; citing report `<BASE>task-2-contract-c1-fix2-report.md` | A10–A14 (values cited in the fix2 final register); A15 (report) | A10–A14 fresh = cited (MATCH); A15 fresh = cited (MATCH) | yes — run outputs themselves are retained, plus the citing report; inputs (A1–A9) listed separately | Recorded executed diagnostic checks on the portable baseline. NOT corpus coverage, NOT parser implementation, NOT game parity. No fresh execution in this step. The cited SHAs were verified to hash these exact retained artifacts, not other payloads/blobs. A fresh hash alone does not certify a run; the run rests on these retained outputs plus the cited PASS entries
R2 C1 69-schema-object inventory (36 wire + 33 native) | schema-inventory | 69 (36wire + 33native), recorded (ledger lines 858–859, 868–872: "69schemas / 276NKplannedinstances, not276executedtests") | Contract review (static artifact inventory; counted inventory, not an executed test) | Citing: ledger lines 858–859, 868–872; fix2-report line 143 ("NK_objects 69"). Underlying live bytes: A6–A9 | none cited for the count (ledger entries cite counts only — none invented); underlying manifests: A6–A9 | A6–A9 fresh = cited (MATCH, bytes only); the count itself is not a file byte | yes (inventory recorded in the cited documents; underlying manifest bytes retained) | Static schema-object inventory. Not an executed test. Not per-case human ratification of all 69 objects. Not corpus coverage
R3 C1 123 named-case inventory (C 3, P 5, N 93, LC 11, NV 11) | named-case-inventory | named 123; "countedinventory NOT276executedcases/productcoverage" (ledger lines 868–872); fix2-report line 143 (re-parsed live) | Contract review (counted named cases; not executed) | Citing: ledger lines 868–872; fix2-report line 143 | none cited for the count (citing entries cite counts only — none invented) | n/a (a count, not a file byte — no fresh hash claimed for the count) | yes (named-case list recorded in the cited documents) | Recorded count of named cases. Not executed cases. Not per-case human ratification. Never classed as run or coverage evidence
R4 276 planned NK instances (69 NK objects × 4; NK_matches_manifests True) | planned-NK-instance | 276 PLANNED; "276NKplannedinstances, not276executedtests" (ledger line 859); "PLANNED, not executed tests (F6 OPEN)" (fix2-report lines 143–144) | Planned synthetic case (counterexamples with expected bytes; not yet executed) | Citing: ledger lines 858–859; fix2-report lines 143–144, 292–293. Manifest basis bytes: A6–A9 | none cited for the count (count-only citations — none invented); manifest basis: A6–A9 | n/a (a count); A6–A9 fresh = cited (MATCH) as basis bytes | yes (plan recorded; manifests retained) | Planned synthetic corpus material. Zero executions. Never classed as an executed test. A corpus plan is a future user-selected gate, not an established basis
R5 retained input artifact: canonical generator/check script — canonical byte-identical counterpart of the owned execution carrier for the recorded run | n/a — step-2 retention record (input artifact, not a coverage claim) | bytes retained; cited SHA = fresh SHA | no separate class claim (input to the recorded run R1; an input file is not run evidence) | `<BASE>task-2-contract-examples/generate_and_check.py` | A1 (fix2 final register; ledger line 1065 "G a11444cb…") | A1 fresh = cited (MATCH) | yes | Its existence proves retention of these canonical-generator bytes only (a byte-identical counterpart of the owned execution carrier), not that any run occurred; the owned execution carrier recorded for the run is `<BASE>task-2-contract-c1-fix2-check/generate_and_check.py`, named separately here — it is not invoked or executed in this step (no new execution); run evidence = R1 (A10–A14)
R6 retained input artifact: per-run diagnostic profile (input bytes) | n/a — step-2 retention record (input artifact, not a coverage claim) | bytes retained; cited SHA = fresh SHA | no separate class claim (input to R1; not run evidence) | `<BASE>task-2-contract-examples/diagnostic-profile.json` | A2 (fix2 final register) | A2 fresh = cited (MATCH) | yes | Retained case/input file, not a retained executed run
R7 retained input artifact: shared diagnostic profile (input bytes) | n/a — step-2 retention record (input artifact, not a coverage claim) | bytes retained; cited SHA = fresh SHA | no separate class claim (input to R1; not run evidence) | `<BASE>task-2-contract-examples/diagnostic-profile-shared.json` | A3 (fix2 final register) | A3 fresh = cited (MATCH) | yes | Retained case/input file, not a retained executed run
R8 retained input artifact: victory trace companion (case file) | n/a — step-2 retention record (input artifact, not a coverage claim) | bytes retained; cited SHA = fresh SHA | no separate class claim (case input for R1; not run evidence) | `<BASE>task-2-contract-examples/victory.jsonl` | A4 (fix2 final register) | A4 fresh = cited (MATCH) | yes | Retained case file; no execution claim attached
R9 retained input artifact: defeat trace companion (case file) | n/a — step-2 retention record (input artifact, not a coverage claim) | bytes retained; cited SHA = fresh SHA | no separate class claim (case input for R1; not run evidence) | `<BASE>task-2-contract-examples/defeat.jsonl` | A5 (fix2 final register) | A5 fresh = cited (MATCH) | yes | Retained case file; no execution claim attached
R10 retained input artifact: content manifest (inventory basis bytes) | n/a — step-2 retention record (input artifact, not a coverage claim) | bytes retained; cited SHA = fresh SHA | no separate class claim (basis bytes for R2/R4; not run evidence) | `<BASE>task-2-contract-examples/manifests/content-manifest.json` | A6 (fix2 final register) | A6 fresh = cited (MATCH) | yes | Manifest bytes retained; counts in R2/R4 cite them as basis only
R11 retained input artifact: native-snapshot manifest (inventory basis bytes) | n/a — step-2 retention record (input artifact, not a coverage claim) | bytes retained; cited SHA = fresh SHA | no separate class claim (basis bytes for R2/R4; not run evidence) | `<BASE>task-2-contract-examples/manifests/native-snapshot-manifest.json` | A7 (fix2 final register) | A7 fresh = cited (MATCH) | yes | Manifest bytes retained; counts in R2/R4 cite them as basis only
R12 retained input artifact: rules manifest (inventory basis bytes) | n/a — step-2 retention record (input artifact, not a coverage claim) | bytes retained; cited SHA = fresh SHA | no separate class claim (basis bytes for R2/R4; not run evidence) | `<BASE>task-2-contract-examples/manifests/rules-manifest.json` | A8 (fix2 final register) | A8 fresh = cited (MATCH) | yes | Manifest bytes retained; counts in R2/R4 cite them as basis only
R13 retained input artifact: trace-schema manifest (inventory basis bytes) | n/a — step-2 retention record (input artifact, not a coverage claim) | bytes retained; cited SHA = fresh SHA | no separate class claim (basis bytes for R2/R4; not run evidence) | `<BASE>task-2-contract-examples/manifests/trace-schema-manifest.json` | A9 (fix2 final register) | A9 fresh = cited (MATCH) | yes | Manifest bytes retained; counts in R2/R4 cite them as basis only

## Gap summary

Evidence class (spec §8, all eight rows) | Status (established / partial / not established) | Basis (cited artifacts) | Owner (later gate)
---|---|---|---
Executed protocol/corpus test | partial | The implemented portable baseline has recorded executed diagnostic runs (ledger lines 1061–1062, 1081–1082; fix2-report covering-run block; retained run outputs A10–A14 MATCH; input bytes A1–A9 MATCH). "partial" records that baseline, and does not deny it | The partial basis is recorded only; full Task 2 corpus coverage is NOT established and needs a separately approved corpus plan (user gate 1 scope → gate 2 per-unit brief → gates 6–7 composition then combined final F6)
Contract review | partial | Recorded review rounds exist for specific units (C1 draft acceptance, ledger lines 1109–1118, quoted as recorded — certifies neither process compliance nor collateral safety; F1/C2/F3/F4/F5 frozen with Important residuals per the cap ruling). The 69/123/276 figures are counted inventories (R2–R4), not per-case human ratification | Gates 5 and 8 (per-candidate acceptance; one explicit ratification). Full contract UNRATIFIED; options UNSELECTED
Planned synthetic case | not established | 276 planned NK instances are recorded plan material (ledger lines 858–859; fix2-report lines 143–144: "PLANNED, not executed tests (F6 OPEN)"), with zero executions and no approved corpus plan | Gate 2 (a user-selected per-unit brief that approves a corpus plan), then gates 6–7
Source-derived study | not established | No source-derived basis is retained in the cited artifacts; no game source exists in this slice (spec §6 gate; evidence-track authorization is a later user gate) | Evidence track (preparation task 4 request; identity fields PENDING); user-selected
Bytecode-confirmed inspection | not established | No bytecode-confirmed basis is retained in the cited artifacts; no game bytecode in this slice | Evidence track (preparation task 4 request; identity fields PENDING); user-selected
Isolated original-bytecode execution | not established | No retained isolated original-bytecode probe. The 938/19/35/55/35 probes (R1) are portable diagnostic runs on the baseline, NOT isolated original-bytecode probes — no promotion across classes | Evidence track (preparation task 4 request; identity fields PENDING); user-selected
Live original-runtime capture | not established | No live original-runtime capture is retained among the cited artifacts | Evidence track (preparation task 4 request; identity fields PENDING); user-selected
Parity | not established | Gated on original-runtime evidence; the cited records state explicitly "no Rust/Javaparserparity/originalruntime/validatedlearning claim" (ledger lines 1083–1084) | Task 2 tasks 2/3/7 unchanged; after composition (gates 6–9)

## Provenance and limitations

- Fresh SHA-256 ≠ proof of a run: the fresh hashes in the appendix certify byte identity
  between the current files and the cited values; they never certify that a run happened.
  Run evidence rests on the retained run outputs (A10–A14, containing executed results
  such as "PASS checks: 938") plus the cited PASS entries (ledger 1061–1062, 1081–1082).
- Cited SHAs genuinely hash the cited artifacts: every cited value used here (A1–A16)
  was re-computed read-only this step and MATCHES. Era note: the fix1-era register
  (`<BASE>task-2-contract-c1-fix1-check/final-register.sha256`) cites
  `task-2-contract-examples/generate_and_check.py` = `8be67e4912cf788c2c2142a241ef21fc8bf43787d01533e8329fdb1f19d35f44` —
  that is the fix1-era bytes, verified here against the retained era copy
  `<BASE>task-2-contract-c1-fix1-check/generate_and_check.py` (fresh `sha256sum` MATCH).
  It is cited nowhere as the hash of the current live file (which hashes to A1). Cited
  and fresh values stay in distinct columns throughout; no Git object ID appears in any
  SHA-256 column.
- Historical status scoping: all PASS, acceptance, and freeze statements are quoted as
  recorded, not re-validated now; nothing was executed in this step (no import, no
  execution of the generator or fixtures; byte reads only).
- Retention, not guesswork: all nine input files, the five run outputs, the fix2 report,
  and the C1c probe carrier exist and were hash-verified, so the "unretained" branch was
  not triggered. Had a run record been missing, it would be marked "unretained historical
  evidence; cannot be certified" rather than guessed or executed now.
- Scope limits: no game source, no bytecode, no isolated original-bytecode probe, and no
  live original-runtime capture evidence appears in this slice, and none is inferred.
  F6 final after composition is still future (gate 7); C1 remains DRAFT; the full
  contract remains UNRATIFIED and options UNSELECTED. Nothing here selects, ratifies, or
  completes Task 2.
- All old-plan artifacts and SDD records are read-only; the recovery index is not
  modified. No staging or commit happens in this step; the controller persists the final
  handoff as the report and the uncommitted diff for the fresh reviewer.

## Fresh SHA-256 appendix (this step; bytes only, no execution)

Commands (read-only): `ls` of `<BASE>task-2-contract-examples/` and `manifests/`;
`sha256sum` over the nine input paths (cwd `<BASE>task-2-contract-examples/`) and over
the seven supplemental paths (cwd `<BASE>/`). Full 64-hex values, no truncation.
"Cited by" names the citing document for that exact path.

ID | Artifact path | Cited by | Cited SHA256 | Fresh SHA256 (this step) | Match
---|---|---|---|---|---
A1 | `<BASE>task-2-contract-examples/generate_and_check.py` | fix2 final register; ledger line 1065 (G value) | a11444cbac58f2ff40496614dddbd7d6846579b504563c07d3709d75010ca3ba | a11444cbac58f2ff40496614dddbd7d6846579b504563c07d3709d75010ca3ba | MATCH
A2 | `<BASE>task-2-contract-examples/diagnostic-profile.json` | fix2 final register | dc5464db0d6b599ac13764dc3cd9a3b859336804771a16895f3568f3c621f809 | dc5464db0d6b599ac13764dc3cd9a3b859336804771a16895f3568f3c621f809 | MATCH
A3 | `<BASE>task-2-contract-examples/diagnostic-profile-shared.json` | fix2 final register | d15be69d2ba02b4172ff834c790845cc83b7f09a5c40e783a84a58acd0089a28 | d15be69d2ba02b4172ff834c790845cc83b7f09a5c40e783a84a58acd0089a28 | MATCH
A4 | `<BASE>task-2-contract-examples/victory.jsonl` | fix2 final register | a7f670019d0be2a89528b63548ffb5fb02c2fcffc740245d4f3730c44e7e0792 | a7f670019d0be2a89528b63548ffb5fb02c2fcffc740245d4f3730c44e7e0792 | MATCH
A5 | `<BASE>task-2-contract-examples/defeat.jsonl` | fix2 final register | d69d3a5b3cd1e4b94f7dbc3c52ae1f5fade2155cfe9e00bd7809cb0488f06df7 | d69d3a5b3cd1e4b94f7dbc3c52ae1f5fade2155cfe9e00bd7809cb0488f06df7 | MATCH
A6 | `<BASE>task-2-contract-examples/manifests/content-manifest.json` | fix2 final register | a3bcd7f6c37f6912d7687768f599c0c4bc649cf1ef3f0a9f0f705ebb66c891ac | a3bcd7f6c37f6912d7687768f599c0c4bc649cf1ef3f0a9f0f705ebb66c891ac | MATCH
A7 | `<BASE>task-2-contract-examples/manifests/native-snapshot-manifest.json` | fix2 final register | f23927cdc4a6a5bbed935195fac1bb09aff6f54d8b1465bd5463c00f85ec2401 | f23927cdc4a6a5bbed935195fac1bb09aff6f54d8b1465bd5463c00f85ec2401 | MATCH
A8 | `<BASE>task-2-contract-examples/manifests/rules-manifest.json` | fix2 final register | 9f40bc5c9cc4cd0002de9fc3891478b3ee9dd3c7683c1e76efea51521fbfac2d | 9f40bc5c9cc4cd0002de9fc3891478b3ee9dd3c7683c1e76efea51521fbfac2d | MATCH
A9 | `<BASE>task-2-contract-examples/manifests/trace-schema-manifest.json` | fix2 final register | 7b6b8cf486a4aead3c919e4b90e35734e92e6131d86154b7cb89ee8735b3d64f | 7b6b8cf486a4aead3c919e4b90e35734e92e6131d86154b7cb89ee8735b3d64f | MATCH
A10 | `<BASE>task-2-contract-c1-fix2-check/run-fix2-main.log` (ownedmain938 output) | fix2 final register | 438c615b385c8c8d49ef30f551005eaa38a4e76722f83234d9087e36de22cc00 | 438c615b385c8c8d49ef30f551005eaa38a4e76722f83234d9087e36de22cc00 | MATCH
A11 | `<BASE>task-2-contract-c1-fix2-check/run-fix2-green.log` (fix2probe19 output) | fix2 final register | 3ebf2f57f8130e796ea2d0072665e2504eb14818a93ab641d9ce41f1989086eb | 3ebf2f57f8130e796ea2d0072665e2504eb14818a93ab641d9ce41f1989086eb | MATCH
A12 | `<BASE>task-2-contract-c1-fix2-check/run-fix1-probe-fix2.log` (fix1probe35 output) | fix2 final register | 2fc94ea33b0e953c73e2283e476cfb29dec48878cd727f59d69498dc0cafc805 | 2fc94ea33b0e953c73e2283e476cfb29dec48878cd727f59d69498dc0cafc805 | MATCH
A13 | `<BASE>task-2-contract-c1-fix2-check/run-c1c-probe-fix2.log` (C1c55 output) | fix2 final register | 6f9e92908d850302236d11fdd5cb8c57c9117b757eb839c72cc8a884e368075d | 6f9e92908d850302236d11fdd5cb8c57c9117b757eb839c72cc8a884e368075d | MATCH
A14 | `<BASE>task-2-contract-c1-fix2-check/fix2-recompute.log` (NEWrecompute35 output) | fix2 final register | 9cc690febbefc499167b61e6f67546f9af87924b86cc0925a0b4584a1703dbfa | 9cc690febbefc499167b61e6f67546f9af87924b86cc0925a0b4584a1703dbfa | MATCH
A15 | `<BASE>task-2-contract-c1-fix2-report.md` (citing report) | fix2 final register (self-row) | 9e5919a2c7bf258a66b4b8afafc70dffc2fc5f80da5d84f77ef60b2fd31fd467 | 9e5919a2c7bf258a66b4b8afafc70dffc2fc5f80da5d84f77ef60b2fd31fd467 | MATCH
A16 | `<BASE>task-2-contract-c1c-check/c1c_ast_probe.py` (C1c probe carrier, unchanged) | fix2 final register | 1fabf69ed62aa8075ab9fc7422c07b88edae6e24d64901d237e87a1dd5f867c5 | 1fabf69ed62aa8075ab9fc7422c07b88edae6e24d64901d237e87a1dd5f867c5 | MATCH

## Named checks (for the fresh reviewer / controller)

- `acceptance-check-focus-c`: the four claim kinds — `diagnostic-check-run` /
  `schema-inventory` / `named-case-inventory` / `planned-NK-instance` — remain four
  distinct rows (R1–R4); R1 and its basis state "recorded executed diagnostic checks;
  not corpus coverage; no fresh execution here"; the planned-NK row (R4) is never classed
  as an executed test; inventory rows (R2–R4) are never classed as coverage; 938/19/35/55/35
  is stated as NOT parser/full-corpus/game; 69/123/276 are individually cited; no
  "all coverage is complete" claim appears; the C1 draft acceptance (ledger 1109–1118)
  is not cited as coverage proof.
- Global hygiene (fixed-string, no regex):
  `grep -nF -- 'F6 gap assessment' <this file>`;
  `grep -nF -- 'Claim | Claim kind | Recorded value | Evidence class (spec §8) | Artifact path | Cited SHA256 (from the citing document) | Freshly computed SHA256 (this step, bytes only) | Retained? | Proof note' <this file>`;
  `grep -nF -- 'Evidence class (spec §8, all eight rows) | Status (established / partial / not established) | Basis (cited artifacts) | Owner (later gate)' <this file>`;
  `git status --porcelain` shows only the expected paths.
- Hash verification record: the two `sha256sum` commands listed under "Fresh SHA-256
  appendix"; all sixteen listed values (nine required input files + seven supplemental)
  MATCH their cited values; the fix1-era copy verification
  (`<BASE>task-2-contract-c1-fix1-check/generate_and_check.py` = `8be67e49…`, era
  register's cited value for that era) is recorded in the provenance section.
- Retention semantics: rows R5–R13 retain input artifacts only; the five run outputs
  (A10–A14) are the retained actual run outputs; a missing run record would be marked
  unretained, never guessed or executed.
