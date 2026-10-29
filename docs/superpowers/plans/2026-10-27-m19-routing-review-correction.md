# M19 routing import technical correction

Date: 2026-10-27. **Authoritative correction for implementation; same-reviewer bounded recheck pending.** This separate addendum supersedes **only the unsafe whole-file routing-import fallback in Task3, line98 of the approved [M19 plan](2026-10-27-m19-defect-act3.md)**. Read it with that plan and the approved [design](../specs/2026-10-27-m19-defect-act3-design.md). It does not rewrite either approved document or the approved M18 inputs.

The primary user actually approved the M19 design and plan on 2026-10-27. This correction addresses the same independent reviewer's subsequently verified Important routing-import finding under the current primary-session instruction. Existing approval, finite content, caller policy, six cases, normal evidence gates and execution method remain unchanged; this is not a request for another user approval. Implementation remains held until accepted M16/M17/M18 source reconciliation and root-assigned source/release. Planning-child closure also waits for root's disposition of the bounded reviewer recheck.

## Exact preserved identities and finding

| Preserved input | SHA-256 |
| --- | --- |
| Approved M19 design | `f6c18ee70bc9b695e237ad3b6d9dae8506a7279e31124da58f1376e334e2cb02` |
| Approved M19 plan | `257e5be4351d4792a275c2d90de2dbace612ac38b2c2e88ebd3c5c76dd103f3e` |
| Original independent review | `e4a19f15a4d0a50367f8210fb8eb4cbd914f312569a1ab66cba935873bc4e53a` |
| Same-reviewer routing applicability addendum | `ece23b6a28633521c6e3369ab2dd41cb43f5069f332810b0fa835b0c97a94408` |

At accepted source `d1fa706638a31a854fdcaeed9a9a4642abf716eb`, `crates/spire-core/examples/m13_support/public_routes_v2.rs:249` starts a caller-dependent `#[cfg(test)]` module. Its lines350–353 call `super::super::Caller::new_m14`, write private Act3 progress flags, and call `super::super::action_for`. Reparenting the entire file under `m19_support` rebinds those names to M19, whose contract does not provide that M14-specific API. The two production functions' `pub(super)` visibility is insufficient to establish reuse safety in core/native test consumers. This corrects the reuse-safety claim in the retained initial source-facts record; that record and the original review remain immutable.

## Required replacement for Plan:98

Task3 consumes **an accepted shared production routing helper**, bound in Task0 to its exact accepted source path, signatures, visibility, helper SHA-256 and old/new consumer test targets. Direct whole-file import of `examples/m13_support/public_routes_v2.rs` into M19 is forbidden. If the accepted helper is absent or incompatible, the affected caller implementation blocks at Task0 and reports that exact dependency to root; the old fallback is not available.

Coordination with the M17 author establishes this **proposed future seam**, not an accepted or implemented API:

- Production-only owner: `crates/spire-core/examples/shared/public_routes.rs`, with `pub(crate) fn action(decision: &PublicDecision, map: &PublicMap, elite_reached: bool, ordinary_reached: bool) -> Result<GameAction, String>` and `pub(crate) fn action_m14` with the identical parameter and return types.
- M17's bounded extraction moves the existing production search/ranking/weights unchanged into that owner. It has no dependency on an enclosing `Caller`, `action_for`, caller-private progress fields or caller-dependent test module. M19 consumes this accepted helper, or its actual accepted equivalent, without another extraction or routing implementation.
- The existing `examples/m13_support/public_routes_v2.rs` retains both `pub(super)` signatures as forwarding adapters to `../shared/public_routes.rs`. Its entire existing caller-dependent `#[cfg(test)]` module, test ownership and assertions remain there, so its parent references continue to resolve to the actual M13/M14 caller. In particular, preserve caller-level missing-route/error assertions; do not replace them with direct-helper assertions that bypass `action_for`.
- Future M19 support includes only the shared production module, using the accepted equivalent of `../shared/public_routes.rs`. Acts1/2 retain `action`; Act3 retains `action_m14`, including its existing public-HP recovery and extra-elite ranking. This is internal example/test support, with no new public simulator API or policy change.

All names above remain provisional until Task0 records actual accepted predecessor source. Do not add a fake M14 constructor/compatibility shim, copy routing logic, suppress or weaken existing tests, or introduce a routing framework. Root carries the actual accepted helper mapping through M18's existing reconciliation gate; this addendum does not amend its approved pair.

## Existing verification lane and disposition

After implementation is separately released, the bounded extraction's accepted evidence must include the existing caller-owned tests, using `cargo test -p spire-core --test m13_silent_act2 public_routes_v2::tests --locked --offline` and the corresponding `m14_silent_act3` target. Existing assertions, route decisions and accepted traces remain preserved. M19's already-planned `m19_defect_act3` core target and `run_m19_schema`, `run_m19_session`, `run_m19_replay`, `run_m19_batch` environment targets must compile and run with the shared helper under `cfg(test)` without M14 compatibility APIs in M19. Record those accepted helper/consumer identities in the existing reconciliation and preservation receipts; no new verification framework or per-task reviewer is added.

The original six fixed triples, all three actual Act3 boss entry/matching-record gate, at least one true normal-start Act3NoKeys, genuine-loss retention and pre-outcome caller freeze remain mandatory. No runtime checks were performed for this documentation correction. Root receives the frozen addendum, source verification and exact reference to superseded Plan:98 before the **same** independent M19 reviewer rechecks only this issue. The original READY report is preserved alongside its separate applicability correction; neither the author nor this addendum declares the finding discharged before that recheck.
