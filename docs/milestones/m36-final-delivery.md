# M36 — Final delivery and versioned contract freeze

**Profile status.** M36 merged on the M35 merge base `09b8fc81` as the commit
tagged v1.0.0; the pre-merge evidence below was taken on the read-only
candidate source. The approved scope is the four-character Standard target at A0–A20,
Acts 1–4 and the Heart, with finite Standard content. The M36 merge activates the
freeze. Source integration, profile acceptance, Python artifact delivery and
original-runtime validation remain distinct claims; see the
[delivery handoff](../delivery.md).

## Scope and decisions

M36 consolidates delivery evidence and freezes selected versioned interfaces.
It does not add strategy tuning or a normal-win gate. The approved decisions
are: **U1** versioned contract freeze F-a; **U2** qualified Beads closure and
disposition; **U3** version `1.0.0`. The design rulings **D1** retain the
documented existing fault-smoke packets as fault-smoke inputs and close the
identified upstream-fixed issues at merge with consumer re-pin notes.

## Deliverables and evidence ledger

| Task | Delivered scope | Evidence grade and boundary |
| --- | --- | --- |
| 0 — baseline / binding | Frozen M36 task contract, source identity and acceptance lanes | Source identity and retained evidence; no gameplay conclusion by itself |
| 1 — public search worker bound | Worker count accepts `1..=256`; weak registry plus one hot pool; a search holds its pool until it drops or `with_workers` changes its count | Native source/tests; Python input-error path covered separately |
| 2 — from-start Heart | Four characters × {A0, A20}, 8 supplied routes; replay, deterministic construction, semantic pins and B=8 twin; ignored 84-route A0–A20 sweep | Native, source-grounded, supplied. No normal reach, strategy, Python Heart route or original-runtime claim |
| 3 — A20 checks | `a20_checks` reports scalar S, batch B, memory M and capacity C diagnostics | Native diagnostic harness; two release runs and the route measure are in [performance](../performance.md#a20-standard-checks-m36) |
| 4 — examples and retirement | RunVectorEnv and StandardCombatVectorEnv examples; policy option for `standard_rollout`; examples smoke test; retire m22/m23/m24_same_choices | Example smoke evidence; sample policies are demonstrations, not strategy evidence |
| 5 — Python fault coverage | SAME_STEP statistics/fault composition and isolated Run/vector-combat fault/recovery checks; readiness-test GPU isolation fix; `test_build_metadata.py` probe repaired (broken since M35's build-script identity digest; test-only) | Developed against the base extension; passed on the lane-P candidate extension (444 Python tests) |
| 6 — freeze and version | Frozen identity and replay assertions, Python public-name checks, version 1.0.0 | Native and Python-source tests; imported extension proof depends on build receipt |
| 7 — docs and delivery | Contracts, limits, README, changelog, release notes, profile and handoff | Documentation of evidence and freeze; not runtime validation |

### Heart acceptance details

Command: `cargo test --locked --offline -p spire-env m36_heart`. Each cell uses
the public seed derived from `0xB0+i` at A0 and `0xB8+i` at A20, with
`i=0..3` in Ironclad, Silent, Defect, Watcher order. Creation is the default
experienced profile with full Neow. Test-only supply sets HP and max HP to 900
and adds three keys. At every combat decision where an enemy is above 1 HP, the
test-only M34b wound hook hits each living enemy down to 1 HP through the shared
resolver; M36 corrected it so the hit is halved rather than ever killing. The
policy's next attack then finishes the enemy, so fights are not really fought.
The route validates creation, Neow, room and map progression, events, shops,
rewards, keys, the Act 3 ending, reaching the A20 second boss, Act 4,
HeartVictory settlement, replay and terminal views. It does not exercise real
combat difficulty; each route is supplied, not a policy win. Its accepted step counts
are 326/345 (Ironclad), 331/333 (Silent), 309/329 (Defect) and 351/419
(Watcher), A0/A20 respectively.

The ignored A0–A20 sweep completed 84/84 after a test-only supply-hook defect
was fixed. Two runs reached the Heart at lower floors after Secret Portal
skipped to the boss; two policy outcomes died and were reseeded with disclosure.
These are supplied diagnostic outcomes, not normal traversal. The M36 scope
uses the later 2026-10-30 rule that supplied victory is valid when labeled as
such; it supersedes spire-dxo.21's earlier ACC wording against injected-resource
full-run success.

## Coverage and retained boundaries

| Area | Integrated interface / result | Remaining limit |
| --- | --- | --- |
| Standard Run | Four characters, A0–A20, Acts 1–4/Heart; creation, observations, replay, checkpoints and public actions | No strategy or normal Heart reach claim; bounded/captured content and intentional deviations remain |
| Standard combat | `standard-combat-v1`, native sessions and Python facade | Separate combat terminal semantics; does not include Run settlement |
| Public search | Belief, knowledge, bounded branching, tactical starts and `step_many` | Bounded combat horizon; no chance-node enumeration or full Run futures; no native wall-clock deadline |
| Batching | Shared Run/combat batch owners and vector facades | See [batching limits](../batching.md); fault and wrapper caveats remain |
| Legacy consumers | M9, M10, Gym66 v1/v2 and legacy combat names remain frozen | Compatibility changes follow the protocol; not all legacy surfaces are the preferred new consumer path |
| Python delivery | Lane P built the 1.0.0 extension from the read-only candidate source: `source_digest` `d4b31212…`, features `["spire-python/extension"]`, profile `release`; nine fingerprints equal to the M35 packet; 444 Python tests passed; readiness 8/8 with the GPU hidden and visible; n14g/bhl/tsw fault smoke without `RunFaultError` | Candidate evidence. The post-merge packet must report `dirty=false` and the same `source_digest`; its wheel and module hashes are published with the Release |
| Performance | A20 diagnostic harness, two release measurements, the eight-route measure and rerun commands | Single-host diagnostics with no thresholds; capacity maxima are lower bounds. See [performance](../performance.md#a20-standard-checks-m36) |
| Original runtime | No original game was run for this milestone | No original-runtime fidelity, RNG/seed parity or transfer claim |

### Global supply-hook fix

The first ignored sweep found that a test-only wound hit could become lethal
through The Boot, Wrath or Vulnerable, causing a combat decision outside a
public step. M36 changed the diagnostic hook to trial the hit on a clone and
halve lethal damage until all targets survive. The fix is under the default-off
diagnostics feature, is test-only and does not alter gameplay or a rules ID.
Red/green logs are retained in the private M36 evidence packet. The original
M34b/M35 routes and eight Heart pins did not move.

## Freeze inventory

The M36 merge freezes the Standard family (rules/content/replay IDs, observation
and combat-view descriptors, standard combat catalog schema, reward/seed/RNG
IDs, vocabulary and capacity preset), card metadata v1, `build_info()` fields,
batching contract, Python facade names and signatures (names tested; signatures
frozen by policy and checked in review), and the legacy M9, M10 Heart,
M10 policy Run, Gym66 v1/v2 and legacy combat-v1 surfaces used by consumers.
Batching has no separate version ID; its identity is the member profiles and
the published batching contract. Exact checkpoints remain opaque and in-memory.
Native-only M12–M25 profiles, authored M16/M21, diagnostics, examples, test
helpers and checkpoint internals are outside this freeze's frozen-name set.

Frozen fingerprints are listed in the
[change protocol](../delivery.md#freeze-and-change-protocol). The freeze tests
are `frozen_identities`, `frozen_replay_digests`, the Heart pins and
`tests/python/env/test_frozen_api.py`. Gym66 shares the M10 rules ID. Combat-v1
has no rules ID and is protected by identity and replay digest checks.

## Beads disposition

These are the approved merge-time dispositions; execution/closure is recorded by
the PM in Beads.

| Bead(s) | Disposition |
| --- | --- |
| `spire-dxo.58` M36 | Close after merge and the post-merge build packet |
| `spire-dmm` | M5 worker cap and M6 coverage delivered; close at merge |
| `spire-0t4` | Resolved by retiring the stale same-choices examples; close at merge |
| `spire-vhh` | Reproduced with GPU hidden; test isolation fix passed with GPU hidden and visible; close as fixed |
| `spire-oto` | All 28 children closed; close as no remaining work |
| `spire-831`, `spire-2o7` | Delivered through M34s (PR99–101) and M35 `step_many`; close at merge |
| `spire-14g`, `spire-bhl`, `spire-tsw` | Fixed in merged PRs 67–68; close with upstream-fixed note and preserve permanent fault-smoke inputs |
| `spire-90f` | Metadata v1 delivered in PR70 and evolved to `effbb810…`; close with re-pin note |
| `spire-dxo.7–.22` | Close with qualification/mapping note; `.22` only after `spire-vhh` closes |
| `spire-dxo` epic | Close after M36 merge, build and freeze |
| `spire-12i`, `spire-erz` and the deferred Beads | Untouched; out of scope |

The merged profile history and individual finite limits remain in the
[milestone index](README.md) and linked profiles. M36 evidence does not
promote a native-only profile into Python delivery or rewrite prior
losses, faults, or missing endpoints.
