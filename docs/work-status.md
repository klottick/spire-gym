# Work status

## Current snapshot — 2026-11-26, release 1.2.5

- Version 1.2.5 adds the `tiered_verbs` native playout rule, which keeps potions out of sampled
  playouts, plus gold and held potions in playout summaries (PR #132). Rules IDs remain Standard v9,
  M10 (and Gym66) v6 and M9 v7. Frozen fingerprints, replay digests, Heart pins and existing root
  digests are unchanged. See the [1.2.5 release notes](releases/v1.2.5.md) and [CHANGES](../CHANGES.md).
- The tag, the post-merge build packet and the published artifact identities follow the release merge.

## Snapshot — 2026-11-25, release 1.2.4

- Version 1.2.4 adds native batched playouts for Standard public search (PR #129) and a regression
  test for Fear No Evil's intent and hit order (PR #130). Rules IDs remain Standard v9, M10 (and
  Gym66) v6 and M9 v7. Frozen fingerprints, replay digests, Heart pins and existing root digests
  are unchanged. See the [1.2.4 release notes](releases/v1.2.4.md) and [CHANGES](../CHANGES.md).
- The tag, the post-merge build packet and the published artifact identities follow the release merge.

## Snapshot — 2026-11-25, release 1.2.3

- Version 1.2.3 packages wider Standard public-search coverage (spire-61l stages 2 and 3). PR #126
  extends exact draw knowledge and PR #127 adds roots under Runic Dome. Rules IDs remain Standard v9,
  M10 (and Gym66) v6 and M9 v7. Frozen fingerprints, replay digests and Heart pins are unchanged.
  See the [1.2.3 release notes](releases/v1.2.3.md) and [CHANGES](../CHANGES.md).
- The tag, the post-merge build packet and the published artifact identities follow the release merge.

## Snapshot — 2026-11-24, release 1.2.2

- Version 1.2.2 packages the remaining spire-sjh changes (PR #124). Reachable engine limits report
  typed bounds, FTL applies large Slow-scaled hits, and Self-Forming Clay accrues pending Block beyond
  999. Current rules IDs are Standard v9, M10 (and Gym66) v6 and M9 v7. Frozen fingerprints, replay
  digests and Heart pins are unchanged. See the [1.2.2 release notes](releases/v1.2.2.md) and
  [CHANGES](../CHANGES.md).
- The tag, the post-merge build packet and the published artifact identities follow the release merge.

## Snapshot — 2026-11-24, release 1.2.1

- Version 1.2.1 packages the first spire-sjh changes (PR #122). Giant Head's Slow stacks beyond 999,
  legal energy gains cap at 999, and selected engine faults report typed bounds. Current rules IDs are
  Standard v8, M10 (and Gym66) v5 and M9 v6. Frozen fingerprints, replay digests and Heart pins are
  unchanged. Other reachable bounds remain open. See the [1.2.1 release notes](releases/v1.2.1.md)
  and [CHANGES](../CHANGES.md).
- The tag, the post-merge build packet and the published artifact identities follow the release merge.

## Snapshot — 2026-11-22, release 1.2.0

- Version 1.2.0 packages the work merged since v1.1.1: the M39a combat-entry resource descriptor (PR #116), the
  spire-d4h card-queue order fix (PR #117), the opt-in compact standalone combat and search branch view from M39b
  (PR #119), the pytest-xdist dev dependency and `a20_checks` step-cap seeds (PR #118) and the wider `PublicRoot`
  coverage from M40a (PR #120). Current rules IDs are Standard v7, M10 (and Gym66) v4 and M9 v5; frozen fingerprints
  are unchanged. Omniscience and Storm queue-order limits remain. See the [1.2.0 release notes](releases/v1.2.0.md) and
  [CHANGES](../CHANGES.md).
- The tag, the post-merge build packet and the published artifact identities follow the release merge; use Beads and
  the primary session for live state.

## Snapshot — 2026-11-16, release 1.1.1

- Version 1.1.1 packages the spire-d8g gameplay fixes from the post-1.1.0 stress campaign (PR #114). Current rules IDs
  are Standard v6, M10 (and Gym66) v3 and M9 v4; frozen fingerprints are unchanged. In windows with no targetable enemy,
  costs are now shown and cost-reading effects act on them, as in the original. The Havoc card-order difference
  remains open (spire-d4h). See the [1.1.1 release notes](releases/v1.1.1.md) and [CHANGES](../CHANGES.md).
- The tag, the post-merge build packet and the published artifact identities follow the release merge; use Beads and
  the primary session for live state.

## Snapshot — 2026-11-16, release 1.1.0

- Version 1.1.0 packages the work merged since v1.0.0: the M37a and M38a diagnostics (PRs #106 and #108), the M37b
  and M38b internal speedups (PRs #107 and #109),
  the opt-in compact Standard observation from M38e (PR #110), and the spire-3xq combat and spire-0kr shop and reward
  fixes (PRs #111 and #112). Current rules IDs are Standard v5, M10 (and Gym66) v2 and M9 v3; frozen fingerprints are
  unchanged. See the [1.1.0 release notes](releases/v1.1.0.md) and [CHANGES](../CHANGES.md).
- The tag, the post-merge build packet and the published artifact identities follow the release merge; use Beads and
  the primary session for live state.

## Snapshot — 2026-11-11, M36 final delivery (v1.0.0)

- M36 merged on base 09b8fc81d793ab338bb8346be975ce51ba6ff161 as the commit tagged v1.0.0, the last roadmap milestone. It carries the four-character, A0–A20, Acts 1–4 and Heart target into final delivery; the accepted profiles state their individual limits.
- M36 adds the supplied from-start Heart acceptance (four characters × A0/A20), the public-search worker cap and pool lifetime, the A20 diagnostic checks, the Run and Standard combat vector examples, Python fault coverage, the versioned contract freeze (identity, replay-digest, Heart and Python-name pins) and version 1.0.0. See the [M36 profile](milestones/m36-final-delivery.md).
- Pre-merge evidence: Rust workspace 5,133 passed, 0 failed, 21 ignored; fuzz, A0–A20 traversal and frozen replay equal to base; ignored Heart sweep 84/84. A20 measurements are in [performance](performance.md#a20-standard-checks-m36). The candidate extension (lane P, `source_digest` `d4b31212…`) passed 444 Python tests with all nine fingerprints unchanged.
- The dated snapshots and workstream record moved verbatim to [the work-status history archive](handoffs/2026-11-11-work-status-history.md). This page is the current snapshot; use Beads and the primary session for live task ownership.
- The freeze takes effect at the M36 merge; the merge PR is named in [CHANGES](../CHANGES.md). The post-merge build packet then publishes the artifact identities. See the [delivery handoff](delivery.md) and [milestone index](milestones/README.md).

## Since v1.0.0 — 2026-11-12

- M38e, the opt-in compact Standard observation (`observation="compact"`), merged as PR #110. The dense observation,
  every frozen identity and replay digest are unchanged; see the [M38e profile](milestones/m38e-compact-observation.md).
