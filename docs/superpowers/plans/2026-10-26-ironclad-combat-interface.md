# Ironclad Combat Interface Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans task-by-task; senior owns integration and useful isolated Luna/MAX assistance. Steps use checkbox syntax.

**Goal:** Deliver authoritative standalone Ironclad A0 combats and the same public combat view during normal Runs.

**Architecture:** Separate standalone session uses existing Run entry/Combat owners and settlement. A common numeric encoder selects M10-semantic combat groups; additive Run sidecars preserve old tensors and replay.

**Tech Stack:** Existing Rust, PyO3, NumPy, Gymnasium; locked offline dependencies.

**Spec:** docs/superpowers/specs/2026-10-26-ironclad-combat-interface-design.md

## Global Constraints

- Isolated accepted M10 source; no frozen root/main/Learn edits, installs, commits, publication or cleanup.
- Explicit Ironclad A0; finite lists and strict public configuration; no private state imports.
- Preserve old combat-v1 and M9/M10 schema/default/fingerprint/record contracts.
- Native owner/fault retention and owned arrays; batch disabled autoreset/manual final ownership.
- Unique absolute Cargo target; jobs2, RUST_TEST_THREADS=1, --locked --offline; scratch on /home.
- One fresh Luna/MAX final review and parent integration agreement before final acceptance.

## Review Focus

1. Invalid/hidden entry keys and numeric booleans reject before abandoning current state.
2. HP/item/deck acquisition vs already-owned resource semantics and elite/boss hooks are correct.
3. Combat/card/potion selectors route to policy; noncombat selectors do not; dispatch index is unchanged.
4. Accepted effect followed by capacity/export fault cannot be retried or reset into a fabricated loss.
5. Batch terminal rows/masked reset/preflight and array ownership survive partial failures and close.

### Task 1: Core full entry and durable cleanup checkpoint

**Files:** new run/training.rs and tests/combat_training.rs; scoped run/{mod,model,engine,combat}.rs and items/mod.rs.
**Interfaces:** `CombatTrainingEntry` (public typed resources without handles), `Run::prepare_combat_training(entry, rng) -> Result<Run, RunError>`, `Run::enter_training_combat(encounter)` and `Run::combat_checkpoint()`.
- [ ] Write/run failing real-entry tests for non-M3 cards, multiple acts, upgrades, items, deterministic multiset entry, validation, old M3 rejection and cleanup checkpoints.
- [ ] Implement finite entry validation/canonicalization, existing Run entry with typed training battle/room origin; capture public terminal HP checkpoints during settlement without changing CombatRecord.
- [ ] Run core new tests plus relevant M10/M3 preservation; expected new tests all pass, old seven remain historical.

### Task 2: Common view, strict catalog and native session

**Files:** new env/combat_training.rs, env/combat_policy.rs, tests/combat_training.rs; scoped env/{lib,run_observation,run_session}.rs.
**Interfaces:** `CombatTrainingCatalog::from_json`, `CombatTrainingSession::{new,reset,preflight_step,step,decision,close,fail_delivery,environment_spec}`; common descriptor/encoder and public Run combat sidecar.
- [ ] Write/run failing tests for catalog hidden keys/bools, all advertised native setups/cards, common shape/mask/reference mappings, outcomes/resources/Run lifecycle, tokens/capacity fault/no retry.
- [ ] Implement strict catalog/fingerprints, fixed common capacities, session token/seed/ordinal/fault ownership, public Run identity and common view methods.
- [ ] Run new env tests plus existing env preservation; expected no new failure.

### Task 3: Owned Python single facade and additive Run methods

**Files:** new python binding combat_training.rs and gymnasium/ironclad.py; scoped binding lib/run_env, stubs, exports, Python tests.
**Interfaces:** `NativeIroncladCombatSession`, `IroncladCombatEnv`, RunEnv.environment_spec/combat_observation_spec/combat_view.
- [ ] Write/run failing actual extension tests for strict inputs, numeric shapes/masks/ownership, selection and potion actions, public identities/Run routing and packaging-fault retention.
- [ ] Implement thin GIL-released native wrapper and token-guarded Python facade; no simulator rules in Python.
- [ ] Build isolated source-matched extension using existing interpreter/dependencies, run focused Python tests; expected all pass and exact loaded path recorded.

### Task 4: Efficient native manual-reset batch

**Files:** native combat_training batch and gymnasium/ironclad_vector.py plus batch tests.
**Interfaces:** native batch reset/step, stacked owned numeric outputs and explicit per-slot diagnostics; IroncladCombatVectorEnv with disabled autoreset.
- [ ] Write/run failing tests for batch/single equality, terminal old rows, masked reset, atomic invalid preflight, partial runtime fault and export ownership.
- [ ] Implement one GIL-released serial native batch, no Python per-slot native calls or automatic replacement; freeze batch after uncertain partial failure.
- [ ] Run binding and facade tests; expected all pass.

### Task 5: Delivery, preservation and independent review

**Files:** docs/combat-training.md, public examples, README/API guidance and exact ignored integration packet.
- [ ] Exercise real public examples in both modes; run workspace, focused preservation, Python suite and changed-file checks.
- [ ] Publish exact signatures/IDs/content lists/limits/reset/lifecycle/error/final/batch semantics and build provenance.
- [ ] Materialize/hashverify candidate and preimages; dispatch ONE fresh independent Luna/MAX final feature reviewer; fix substantive findings with observed RED→GREEN.
- [ ] Report review-ready packet to parent; do not close .66 or send final READY until integrated acceptance agreement.

### Additional user review focus: shared ownership, no copied progression

- [ ] Verify one shared character-aware validated resource builder; Ironclad-only
  public validation stays fixed, future profiles do not require copying startup.
- [ ] Verify standalone consumes authoritative enemy setup metadata and stable
  RNG keys; common view calls shared Run encoding; Python uses shared scalar,
  token, numeric and descriptor packaging. No copied map/room/reward/resolver.
- [ ] Confirm `AGENTS.md` and public guide name the actual extension seams and
  preserve frozen profiles; no speculative framework or Silent public API.

### Task2/3 observed source ruling: Flex and frozen v2

- [ ] Extend ONLY the new common view and opt-in normal-Run policy-v3 descriptor
  with `lose_strength`; reuse one power-vocabulary seam in the shared encoder.
- [ ] Test all75 numeric plays, Flex application/expiry/seed-action replay,
  policy-Run replay roundtrip, old/new identity rejection and frozen v2 hash.
- [ ] Export `m10-ironclad-policy` in Rust/Python profile maps, stubs and public
  examples; old defaults/profile/record identities remain unchanged.

### Final review scoped correction pass

- [x] Observed actual public RED for cross-mode token collision; use one shared
  checked Run owner allocator and test foreign-token atomicity/reset/batch cases.
- [x] Observed actual public RED for unbounded Core Debug diagnostic using real
  Feed/max-HP runtime fault; reuse bounded Run fault projection. Test accepted
  batch prefix, consumed failing attempt, first-fault retention and no retry.
- [ ] Same original reviewer checks only affected source/regressions and the
  full updated public census; no second independent feature reviewer.
