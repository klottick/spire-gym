# Work status

## Current snapshot — 2026-11-11, M36 final delivery (v1.0.0)

- M36 merged on base 09b8fc81d793ab338bb8346be975ce51ba6ff161 as the commit tagged v1.0.0, the last roadmap milestone. It carries the four-character, A0–A20, Acts 1–4 and Heart target into final delivery; the accepted profiles state their individual limits.
- M36 adds the supplied from-start Heart acceptance (four characters × A0/A20), the public-search worker cap and pool lifetime, the A20 diagnostic checks, the Run and Standard combat vector examples, Python fault coverage, the versioned contract freeze (identity, replay-digest, Heart and Python-name pins) and version 1.0.0. See the [M36 profile](milestones/m36-final-delivery.md).
- Pre-merge evidence: Rust workspace 5,133 passed, 0 failed, 21 ignored; fuzz, A0–A20 traversal and frozen replay equal to base; ignored Heart sweep 84/84. A20 measurements are in [performance](performance.md#a20-standard-checks-m36). The candidate extension (lane P, `source_digest` `d4b31212…`) passed 444 Python tests with all nine fingerprints unchanged.
- The dated snapshots and workstream record moved verbatim to [the work-status history archive](handoffs/2026-11-11-work-status-history.md). This page is the current snapshot; use Beads and the primary session for live task ownership.
- The freeze takes effect at the M36 merge; the merge PR is named in [CHANGES](../CHANGES.md). The post-merge build packet then publishes the artifact identities. See the [delivery handoff](delivery.md) and [milestone index](milestones/README.md).

## Since v1.0.0 — 2026-11-12

- M38e, the opt-in compact Standard observation (`observation="compact"`), is a source candidate on branch
  `feat/m38e-compact-observation-20261112`. The dense observation, every frozen identity and replay digest are unchanged;
  see the [M38e profile](milestones/m38e-compact-observation.md).
