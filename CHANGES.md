# Changelog

Changes from the versioned contract freeze onward. The earlier `## CHANGES`
entries are retained in [observation vocabulary history](docs/observation-vocabulary.md).

## Unreleased

- Add M37a profiling diagnostics: the ignored `m37_phase_timing` test in
  `spire-env`, `examples/run_vector_phase_timing.py`, [profiling](docs/profiling.md) and
  a README Speed section. These are diagnostics and documentation only. No gameplay,
  identity, fingerprint, observation or API change.
- M37b internal performance work, with no contract change. Gameplay, RNG use, IDs, rules
  IDs, fingerprints, observations, masks, records and replay bytes are unchanged; the
  freeze tests, frozen replay, traversal and fuzz hashes match v1.0.0. See
  [performance](docs/performance.md) and [profiling](docs/profiling.md) for the numbers.
  - Card preparation clones the combat state once per legal-action enumeration, not once
    per (card, target) payload.
  - `RunSession` returns each observation by move and keeps no copy. `decision()`
    re-encodes it from the unchanged Run, so the bytes are identical.
  - The session step no longer re-enumerates legal actions to check an action it has
    just published.
  - Record headers reuse a per-session observation-spec fingerprint.
  - The Python batch binding stacks observations per slot on the batch's own workers,
    with the GIL released. Each call still returns new arrays.
  - Two internal, `#[doc(hidden)]` additive Rust methods support this: `Run::step_offered`
    and `SessionBatch::map_jobs`. They are not part of the documented API and add no
    Python names.
- Add M38a scaling diagnostics: the ignored `m38_component_scaling` test, `M37_CAPACITIES`,
  `M37_STACK=parallel|parallel-reuse` and `M37_EARLY_DROP` in `m37_phase_timing`, the
  `--loops` and `--drop-before-step` options and a release timer in
  `examples/run_vector_phase_timing.py`, and a
  [parallel scaling](docs/profiling.md#m38a-parallel-scaling) report. These are test and
  example diagnostics only. No engine, binding, gameplay, identity, fingerprint, observation
  or API change.

## 1.0.0 — effective at the merge of PR #103

See the [1.0.0 release notes](docs/releases/v1.0.0.md).

- Freeze the documented Standard, card metadata, `build_info()`, batching,
  Python facade and selected legacy consumer contracts. Add native identity and
  semantic replay pins, Heart pins and Python public-name checks.
- Bound `PublicSearch` worker counts to `1..=256`; reject invalid values with
  `search workers must be 1..=256`. Use weak per-count pool handles and a single
  strong hot pool; a live search holds its selected pool until it drops or `with_workers`
  changes its count, reversing
  M35 P13's per-count residency.
- Fix the M34b diagnostics-only (`m34-diagnostics`) supplied-wound hook so its
  scripted hit cannot kill a target. This is test-only and does not change gameplay or a rules ID.
- Add the supplied from-start Heart acceptance for all four characters at A0
  and A20, plus the ignored A0–A20 supplied sweep.
- Add `a20_checks`, `run_vector_rollout.py`, `standard_combat_vector.py`, and a
  `standard_rollout --policy random|first_legal` option; add example and vector
  fault coverage. Isolate the readiness test's mocked CPU probe so it passes
  with the GPU hidden or visible. Repair the `test_build_metadata.py` probe
  crate, broken since M35 made `build.rs` digest the workspace through
  `src/identity.rs` (test-only).
- Retire Rust examples `m22_same_choices.rs`, `m23_same_choices.rs`, and
  `m24_same_choices.rs`; they depended on stale private receipts. The M21
  example remains historical support.
- Set the project and Python package version to `1.0.0`.
- Publish M36 contract, delivery, performance, profile and release documentation.
