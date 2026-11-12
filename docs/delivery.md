# Delivery and compatibility

This page is the public handoff for the M36 final-delivery profile. It describes
source integration, acceptance grades, limitations and consumer steps. A
verified source build and an imported Python extension are separate evidence.

## 1. Delivered scope and surfaces

The Standard profile covers Ironclad, Silent, Defect and Watcher at A0–A20,
Acts 1–4 and the Heart, using finite Standard content. Rust owns game state and
rules. `spire-env` provides native Run sessions, observations, actions, replay,
batching, public search and `standard-combat-v1`. Python bindings transport those
owners; Gymnasium facades remain thin. Machine-learning models and training
loops live outside the repository. Since M38e, Standard Run sessions and batches
can opt into a compact observation layout (`observation="compact"`) with the same
public information in fewer bytes; the dense observation stays the default
([Standard Run API](standard-run-api.md#compact-observation-m38e)).

Legacy consumer surfaces remain supported under the contract freeze: M9 Run-v1,
M10 Heart Run-v2, M10 policy Run-v3, Gym66 `ironclad-combat-v1`/`-v2`, their
policy views, and legacy `CombatEnv`/`CombatVectorEnv`. The removed native name
`NativeIroncladCombatBatch` has no alias; use
`NativeCombatTrainingBatch(..., kind="ironclad")`. The Rust examples
`m22_same_choices.rs`, `m23_same_choices.rs` and `m24_same_choices.rs` were
retired because their private-receipt comparison is stale after disclosed global
fixes. Standard records and replay are the current consumer path.

## 2. Build and verify

The default native workspace build and test command is:

```sh
cargo build --workspace --locked --offline
cargo test --workspace --locked --offline
```

The default Cargo workspace does not enable the Python extension feature. For a
Python-delivery claim, use the isolated extension workflow in
[combat training](combat-training.md#shared-implementation-and-extension-build)
and the provisioned environment; do not infer extension delivery from a Rust
build or source tree. The routine Python lane is documented in
[development](development.md). Inspect `_native.__file__` and
`_native.build_info()` in the same environment as the test run. Establish a
source match from Rust, binding and relevant Python input hashes, build features
and profile, `dirty`, `source_digest`, `file_digest`, native module or wheel
SHA-256, descriptor fingerprints and exact test results. Matching a Git revision
or descriptor fingerprint alone is insufficient.

The verified post-merge M35 packet at PR102 (`09b8fc81`) reports
`dirty=false`, source digest
`bb1888874447b0fd40090323b10311be45349f568d84f35cccb7813a05deb43a` and 427
passing Python tests. That is M35 evidence, not M36 delivery. The M36 artifact
identity comes from the post-merge build of the commit tagged v1.0.0: it must
report `dirty=false` and the pre-merge lane-P `source_digest`, and its wheel and
`.so` SHA-256 values are published with the v1.0.0 Release, not in this page.
Between the merge and that build, Python delivery of M36 rests only on the
pre-merge lane-P evidence in section 3.

## 3. Acceptance evidence and grades

| Evidence | Result | Grade and limit |
| --- | --- | --- |
| From-start Heart acceptance | 4 characters × {A0, A20}; 8 supplied routes (HP/max HP 900, three keys, and a test-only hook that hits every enemy down to 1 HP at each combat decision); replay, deterministic construction, semantic pins and B=8 twin | Native, source-grounded, supplied; validates progression and settlement only on supplied routes. See [Heart contract](standard-run-api.md#from-start-heart-acceptance-m36). |
| Ignored Heart sweep | 84 supplied routes, four characters × A0–A20; final sweep 84/84 | Native diagnostic; not a normal-policy reach gate |
| Examples | Rust and Python Standard, vector, combat and legacy demonstrations; subprocess example smoke | Example execution; sample action rules are not strategy evidence |
| A20 checks | S scalar (1,024 A20 episodes, about 10,300 actions/s), B batch scaling (64 slots, 1/4/16 workers), M memory (about 1.1 GB at 256 slots) and C capacity headroom; eight supplied route measurements | Diagnostic only, no thresholds; maxima are lower bounds. See [performance](performance.md#a20-standard-checks-m36) |
| Native freeze | `frozen_identities`, `frozen_replay_digests`, Heart pins | Native contract assertions; old records fail closed on identity changes |
| Native suites | Rust workspace 5,133 passed, 0 failed, 21 ignored (base 5,124/0/19 plus the 11 M36 tests); core diagnostics 4,286 passed; fuzz traces, A0–A20 traversal and the 323-input frozen replay equal to base | Native; establishes that M36 changes no gameplay outside the disclosed test-only hook |
| Python API freeze | `test_frozen_api.py` checks Gymnasium exports, `_native` public callables and `build_info()` keys | Python API-name check; imported-artifact source match must be shown separately |
| Python lane P (candidate extension) | Built from the read-only candidate source; imported from that copy with `build_info()` `features=["spire-python/extension"]`, `profile=release`, `source_digest` `d4b31212…`; all nine fingerprints recorded by the M35 build packet equal. Python suite 444 passed (incl. build identity, frozen API, examples smoke of nine scripts, vector faults, public-search cap); readiness 8/8 with the GPU hidden and visible; three retained consumer replays without `RunFaultError` | Python-delivered at the candidate source identity; the post-merge packet must report `dirty=false` and the same `source_digest` |

The Heart command is `cargo test --locked --offline -p spire-env m36_heart`.
The matrix, supplied values and full evidence boundary are recorded in the
[Standard Run API](standard-run-api.md#from-start-heart-acceptance-m36).

<a id="claim-grades"></a>
## 4. Claim grades

- **Source-grounded** means the behavior was derived from static reading of
  the decompiled original game source and is checked by tests of this Rust
  implementation. The original game was never run.
- **Reference-informed** means cited community or other reference data informed
  an implementation. It is not proof that an engine path implements the record.
- **Original-runtime** would require evidence from the original game process.
  None is claimed here.
- **Constructed** describes a test-created state; **supplied** describes a Run
  given resources or scripted outcomes; **traversal** describes actions through
  generated Run progression; **normal** describes the ordinary configured
  start without injected resources. These are setup/evidence labels, not grades
  of fidelity. Supplied success does not establish normal reach.
- **Native-only** means evidence covers Rust source/tests, without a verified
  imported extension claim. **Python-delivered** requires an imported,
  source-matched extension plus its artifact identity and exact test results.
- **Source accepted/READY**, **Run**, **Combat**, **Census** and **diagnostic**
  describe review or assertion ownership; they do not imply original-runtime
  fidelity or Python delivery.

## 5. Claims not made

This delivery makes no original-runtime fidelity, source RNG/seed-stream
parity, strategy strength, win rate, or normal-start Heart reach claim. It does
not claim absolute original-game floor numbering, unrestricted/randomized full
content pools, universal Gymnasium wrapper compatibility, or throughput as an
agent episode rate. A20 supplied routes are not ordinary-policy traversal.

<a id="intentional-deviations"></a>
## 6. Intentional deviations

The current IDs and descriptions are maintained in
[intentional deviations](intentional-deviations.md):

| ID | Disclosed behavior |
| --- | --- |
| RNG-001 | Uses project-scoped reproducible RNG rather than original PRNG and seed-stream coupling; intended conditional probabilities are preserved, not source RNG traces. |
| TIME-001 | Treats Secret Portal's 800-second eligibility gate as satisfied while retaining ordinary eligibility and event choice. |
| UNLOCK-001 | Assumes Ascension 20 is unlocked; Note For Yourself may appear at A0–A14 and never A15+. |
| SOFTLOCK-001 | Empty unguarded event grids complete without a pick rather than stranding the Run. |
| EVENT-001 | Collapses forced one-button pages only when no legal action changes final state or RNG order. |
| EVENT-002 | Uses per-character Match and Keep starter cards; source settlement is only established for Ironclad. |
| EVENT-003 | Empty event selection becomes an ordinary monster room where source leaves its result open. |
| EVENT-004 | Event fights use listed enemy order where source does not establish positions. |
| EVENT-005 | Hides an empty shop purge as Disabled; it re-enables when a purgeable card exists. |
| RNG-002 | Models the source's unseeded end-turn hand shuffle with a dedicated keyed project stream. |
| M33-A20-001 | Uses a native keyed A20 second-boss draw and engine-floor +1 ruling; no source RNG or absolute source-floor parity. |

EVENT-002/003/004 remain source-open or PM-accepted rulings as described in the
linked deviation records; the table does not promote them to source-settled
facts.

<a id="known-limits-by-surface"></a>
## 7. Known limits by surface

- **Standard Run:** no strategy or win-rate claim; finite content and deviations
  remain. Random policies commonly end in Act 1. Heart acceptance is supplied, not a
  policy win: enemies are hit down to 1 HP at each combat decision, so it
  exercises progression, routing, rewards, events and Neow, not combat
  difficulty.
- **Tactical starts and public search:** bounded to documented combat horizons;
  no chance-node enumeration, full Run futures, hidden-intent support, or native
  wall-clock deadline. Checkpoints are privileged exact-world, in-memory tools;
  they are not portable or public-information planning inputs.
- **`standard-combat-v1`:** ends at the engine's combat terminal; Run settlement
  effects do not apply there.
- **Card identities per combat:** a combat issues at most 999 card identities,
  deck included; identities are never reused. A combat that would create more
  (a long or generation-heavy fight) stops with the typed `Unsupported` fault
  "generated card count/runtime boundary". The original game has no such cap.
- **Courier shop rows:** the Standard `offers` group grows by one row per Courier
  purchase in a shop visit (the original restocks in place). The dense preset's
  512 rows hold at least 497 purchases in one visit; beyond that the observation
  is a typed `Capacity` fault.
- **Batching:** public search is capped at 256 workers and keeps a live search's
  first parallel pool until it drops or its worker count changes. Search preflight errors do not identify an
  entry index. A scheduler/bookkeeping panic freezes a batch; a resumed isolate
  may be reported unaffected. CombatPool's committed-undelivered state is
  informational because it has no committed-result recovery API. Tested
  Gymnasium wrappers are a subset, not universal compatibility or unmodified
  mask-blind `check_env` acceptance. See [batching](batching.md).
- **Compact observation (M38e):** its capacities come from measured maxima, not
  engine bounds; a larger state (for example a long generation stall past about
  40 turns, a deck above 256 cards, or repeated Courier restocks) is a typed
  `Capacity` fault that freezes the session or slot, never a truncation. Larger
  capacities cure a row overflow but not a value fault. There is no compact
  Standard combat view, `standard-combat-v1` layout or public-search layout.
- **Python delivery:** only a source-matched imported extension establishes
  Python delivery. The M35 post-merge receipt does not verify the M36 artifact.
- **Performance:** single-host diagnostic measurements; capacity maxima are lower
  bounds, not guarantees, and no thresholds are imposed. Random A20 play ends in
  Act 1, so late-game capacity use comes from the supplied routes.
- **Original game:** no game execution, original RNG roster/algorithm match, or
  Windows non-interference result is claimed.
- **Earlier normal outcomes:** preserve each profile's actual outcome and
  acceptance limits. M15's six ordinary starts all died before Act 3; M18/M19 normal
  outcome gates were closed as strategy-only, with original outcomes intact;
  M20 canonical normal Heart and M22 normal boss victory remain unproven.

<a id="freeze-and-change-protocol"></a>
## 8. Freeze and change protocol

The versioned contract freeze takes effect at the M36 merge. Frozen identities
include the Standard family (observation and combat-view schemas/fingerprints,
rules/content/replay IDs, reward, seed and RNG IDs, capacity preset and
vocabulary), card metadata, `build_info()` fields, batching contract, Python
facade names and signatures, and legacy M9/M10, Gym66 and combat-v1 surfaces used by
consumers. Existing pinned fingerprints include Standard Run
`bf8e1d11…`, Standard combat view `5d8147e1…`, card metadata `effbb810…`, M9
`270163f7…`, M10 Heart `dccc7624…`, M10 policy `28d92d89…`, Gym66 views
`4c73f7de…` and `5ba729fd…`, plus combat-v1 `8fd66d21…`. From the M38e merge
the opt-in compact Standard observation is frozen too: schema
`spire-gym-standard-run-observation-compact-v1`, preset `standard-compact-v1`
and fingerprint `633b9424…`. The complete source
table and replay corpus are asserted by the freeze tests. Batching has no
standalone version ID. **Gym66 shares the M10 rules ID; combat-v1 has no rules
ID.**

1. Fix real gameplay bugs globally. **Affected** means every frozen profile
   whose gameplay routes through the changed rules code. Since the engine is
   shared, treat all frozen profiles as affected by default. A review may exempt
   a profile only with a stated reason that the changed path is unreachable in
   its content; record that reason in `CHANGES.md`.
2. Bump every affected rules ID (for example, Standard rules v4 and M10 rules
   v2; Gym66 shares the M10 bump), re-pin frozen replay digests and Heart pins,
   and add a `CHANGES.md` entry. Combat-v1 has no rules ID: record the change
   and re-pin its digest. Update pre-existing descriptor, metadata and
   vocabulary pins alongside the freeze identity table.
3. A schema-layout change to observations, actions or descriptors bumps the
   schema version and fingerprint and receives a `CHANGES.md` entry and
   DELIVERY note.
4. Additive API names or optional arguments with unchanged defaults need a
   `CHANGES.md` entry but no version bump. Removing or renaming a frozen name
   requires user approval.
5. Keep `frozen_identities`, `frozen_replay_digests`, Heart pins and
   `test_frozen_api.py` green for other changes. **Old records fail closed** on
   an identity bump. There is no multi-version support; the old per-merge freeze
   ceremony does not return.
6. Every merge still receives the auto-build packet.

**What the pins catch.** The freeze tests are a partial backstop, not the
definition of "affected". `frozen_identities` catches changed IDs and
fingerprints. `frozen_replay_digests` replays a fixed corpus whose Standard
episodes are random A0/A10/A20 runs that all die in Act 1 within 95 steps, plus
short legacy episodes; the Heart pins run with every enemy hit down to 1 HP.
So a change to Act 2–4 or high-ascension monster behaviour, to late-game cards,
relics or events, or to anything else outside that corpus can leave every pin
green. A gameplay change still needs the rules-ID bump of rule 2 even when the
pins stay green. `test_frozen_api.py` checks Python names only: the exported
Gymnasium names, the `_native` public callables (stub and runtime) and the
`build_info()` keys. Signatures (parameters and defaults) are frozen by policy
under rule 4 and checked in review, not by a test.

Reproduce the routine native Heart acceptance with:

```sh
cargo test --locked --offline -p spire-env m36_heart
```

Version `1.0.0` is the frozen source release. After merge and verification of
the post-merge build packet, the PM creates annotated tag `v1.0.0` on the merge
commit and a GitHub Release; there is no PyPI release. The Release body and
attached `DELIVERY.md` publish the built wheel and `.so` SHA-256 values and the
final `source_digest`. These artifact values are not asserted in this handoff.

## 9. Open work and Beads dispositions

The post-merge build and the Bead closures are recorded with the v1.0.0 Release
and in Beads. The accepted dispositions for `spire-dxo.58`, `spire-dmm`,
`spire-0t4`, `spire-vhh`, `spire-oto`, `spire-831`, `spire-2o7`,
`spire-14g`/`spire-bhl`/`spire-tsw`, `spire-90f`, `spire-dxo.7–.22` and the
`spire-dxo` epic are summarized in the
[M36 profile](milestones/m36-final-delivery.md#beads-disposition). The PM records
their execution. `spire-12i`, `spire-erz` and the deferred Beads are outside this work and
remain untouched.

## 10. Consumer guidance

Consumers should re-pin from the verified post-merge build packet, including
the source digest, file digest, built artifact hashes, features/profile,
descriptor fingerprints and exact test results. spire-learn's runtime profile
still pins Gym `afb51f5f` and card metadata `b60b66…`; it should re-pin the
metadata fingerprint to `effbb810…`. The consumer does not yet use the Standard
Run API. Removed names include `NativeIroncladCombatBatch` (no alias) and Rust examples
`m22_same_choices.rs`, `m23_same_choices.rs`, `m24_same_choices.rs`. Legacy
M9/M10, Gym66 and combat surfaces remain available under the protocol. See
[release notes](releases/v1.0.0.md), [CHANGES](../CHANGES.md), and the
[milestone index](milestones/README.md).
