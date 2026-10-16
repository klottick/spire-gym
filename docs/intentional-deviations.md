# Intentional deviations from the base game

These are explicit project choices, not simulator bugs or claims of exact base-game
equivalence. Preserve this distinction in designs, tests and evaluation reports.

## RNG-001 — Decorrelated, reproducible project randomness

**Decision date:** 2026-10-16

**Status:** User-approved policy; standalone Python randomness foundation
implemented and verified on 2026-10-16. Combat integration remains pending.

### Decision and rationale

Exact replication of the base game's PRNG, seed-to-outcome mapping, stream coupling
and RNG-call consumption is not required. Use project-owned, reproducible randomness
with independently derived streams for unrelated mechanics, rather than carrying
forward incidental seed/stream correlations. The purpose is to train policies on
gameplay rather than incidental properties of the original RNG implementation.

Preserve the supported game's intended probabilities and conditional rules,
including intentional dependencies such as drawing without replacement and enemy
move-history restrictions. This is not permission to make every outcome IID,
change encounter rules or replace shuffling with independent replacement draws.

Keep seeds and internal RNG state out of policy observations. Retain deterministic
replay and snapshot restoration under the project's own randomness contract.
Independent streams here are a design objective, not a claim that deterministic
PRNG streams are mathematically independent or impossible to infer.

### Correctness and reporting consequences

- Seed-for-seed or bit-for-bit agreement with the original RNG is **not** a project
  acceptance criterion. The superseded original-RNG core/full vector gates and
  exceptional-NaN replication work do not block the replacement RNG design.
- Probability laws and intentional gameplay dependencies still require evidence
  and tests. Altering incidental coupling changes the overall joint process;
  do not describe this simulator as stochastically identical to the base game.
- Validate mechanics against original evidence, using matched supplied random
  outcomes where feasible rather than assuming a shared seed produces the same
  outcomes. This does not waive existing original-game validation requirements.
- Training/evaluation reports must disclose RNG-001 and distinguish simulator
  results from original-game transfer results. Do not label an expected RNG-induced
  trajectory difference as a mechanics defect or use this exception to excuse one.
- Later Python/Rust parity should target the approved project randomness contract,
  not original PRNG parity; agreement still does not establish game correctness.

### Scope and retained history

This decision supersedes conflicting exact-original-RNG requirements only. It does
not waive non-RNG arithmetic, card/enemy rules, effect ordering, visibility, fault
semantics or other correctness/privacy constraints.

Retain prior accepted RNG evidence and failed histories unchanged as reference
material. The 2026-10-15 exact seeded-RNG design remains a superseded draft, not
approved implementation. The replacement design/plan separately specifies the
algorithm, stream boundaries, seed derivation and state representation; this
policy alone does not authorize simulator edits, new evidence access or Git writes.

Approved replacement design:
[`2026-10-16-project-randomness-design.md`](superpowers/specs/2026-10-16-project-randomness-design.md).
The user separately approved its design and
[implementation plan](superpowers/plans/2026-10-16-project-randomness.md) on
2026-10-16. The standalone module `python/spire_lab/combat/rng.py` is implemented
with fixed project/public vectors and reviewed reproducibility, sampling, shuffle
and exhaustion tests. Final verification: 395 Python tests passed. Combat stream
ownership, policy observation boundaries, snapshot integration and Rust parity
remain future work; the existing opaque combat RNG slot is unchanged.
