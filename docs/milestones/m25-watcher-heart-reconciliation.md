# M25 source and API reconciliation

Status: **source candidate; once-only outcomes complete (freeze v1, the F1 corrective
freeze v2 and the G1 corrective freeze v3); independent final review pending. Not accepted.** This record accompanies the approved
[design](../superpowers/specs/2026-10-30-m25-watcher-heart-design.md) and
[plan](../superpowers/plans/2026-10-30-m25-watcher-heart.md) (version 2, design SHA-256
`04a085d1…24e0`, plan `20bf21c4…a62f`). Those documents are unchanged. This record maps their
proposed names to the actual owners, records every amendment, deviation, ruling and global
correction, and keeps the evidence grades separate.

## Source binding

Base: `main` at `7b036e8d2047f17eb765fe42b1c4bb8e164a37e9` (M24 merged in PR64). Before any
production edit, the 971 tracked files and the two approved documents were hashed into a
preimage manifest of 973 entries, retained outside the repository.

## Task M0: design §15 interface diff

Main diffed every design §15 row against the merged M24 source. Twelve rows match M24 version 2
or differ only in the way the row's "Accommodated differences" column allows:

- selector function names;
- internal schema helper names;
- extra M24-only branches whose M25 arm is the M24 arm under design §4.4.

The remaining row does not match:

- **Act3 boss-history JSON spellings.** Merged M24 uses the shared Act3 history codec. That
  codec has emitted and accepted `awakened-one`, `time-eater` and `donu-and-deca` since M14
  (`crates/spire-env/src/run_replay.rs`, `act3_history_name` and `parse_act3_history`). Design
  §11 prints the snake_case forms (`awakened_one`), copied from M24 version 2, whose own native
  test pins the hyphenated form. §15 allows no spelling difference. **Amendment A1** proposes
  that M25 use the shared codec unchanged, so the M25 example becomes
  `{"m25-watcher-heart":{"act1":"guardian","act2":"champ","act3":"awakened-one"}}` and the
  snake_case forms reject. No core change and no accepted wire value changes. Status:
  **accepted by the PM on 2026-10-31** without a separate plan re-review. The PM's basis: it
  is a mechanical wire-spelling correction that matches the design's own rule (reuse the
  shared codecs) and the merged code, and changes no behaviour, core code or accepted wire
  value. Ruling `pm-ruling-amendment-A1-v1.json`, SHA-256 `8e52331e…`; amendment SHA-256
  `52cdb768…`. Execution proceeds from Task M1.

The `uses_act4` call sites match the design §4.3 table; there is no new pre-door site.

## Bindings recorded at Task M0

| Proposed (design or plan) | Bound to |
| --- | --- |
| Supplied validation threading (plan Task M4) | Merged M24 validates supplied resources at four points: `diagnostics/m22.rs:86`, `m23.rs:48` (`supplied`), `m23.rs:103` (`boss_source`) and `m24.rs:60` (`template_at`). Main adds `boss_source_for(.., admission: WatcherAdmission)` in `m23.rs` and keeps `boss_source` as a wrapper passing `WatcherAdmission::M21`, so `m22.rs` and `m24.rs` are not edited (§15 accommodates a different helper name with the same inputs) |
| Journal variant for a permanent upgrade (plan Task C1) | A separate `combat::PermanentCardUpgrade { ordinal, instance, id }` journal in Watcher combat state, beside the accepted Parasite gain and Genetic Algorithm mutation journals, which stay unchanged. Its Debug field prints only when non-empty |
| `GenerationPool::AnyColorAttack` support (plan Task C5) | `content::watcher::foreign_influence_support()`, derived from the catalog (106 kinds, 43/48/15), shared by generation and the native descriptor |
| New power vocabulary (design §11) | `content::watcher::M25_WATCHER_POWER_IDS`, the authoritative spelling list, delivered with the identities (HC0) |
| `CombatItemContext::with_m25_profile()` (plan Task M1) | Built on `with_m22_profile()`, with private `watcher_heart`; M24's `watcher_act3` stays false in M25 combats |
| Item-context normalization in the M25-against-M24 prefix tests (plan Task M1) | Main's tests compare full Run state, and the active combat through its `Debug` text, with exactly `watcher_act3: true` and `watcher_heart: true` exchanged. No Combat-level M25 test marker is added to `combat/engine.rs` |
| Pre-door Recall row with Ruby held (design §4.3) | `m25_pre_door_act4_sites_are_inert` pins the one internal difference: M25 stores the row as Disabled, M24 as Offered. The public row, its rejection and every other state are equal |
| Act4 fault labels (plan Task M2) | `rooms::act4::{Act4FaultSite, fault_label}` covers all seven label sites, including the two that `run/combat.rs` hard-codes. M10, M15 and M20 keep `run.act4.transition` and the `run.m10.*` strings |
| Forced Act4 faults (plan Task M2) | The transition fault is forced by a constructed HP change between `prepare_transition` and `commit_transition`. The ledger cannot fail to publish `ActAdvanced` after its preflight, so the only post-accept failure reachable is the heal. Room-entry hook failure and the Heart Proceed publication cannot fail after their preflights, so `m25_act4_fault_labels_name_m25` pins only their selector. Encounter, settlement and Heart Victory faults are forced through the real owners |
| `act3_victory_source` canonical resources (plan Task M2) | `None` takes the scenario template's own `Run::new` resources (72/72 HP, 99 gold, the ten starter instances, PureWater) and passes them through the same `normalized_for(admission)` check and `Run::from_validated_combat_resources` as supplied resources. The room label `M25ConstructedAct3Source` is the same for both scenarios, so M24 and M25 sources differ only in the scenario tag |
| Act4 producers (plan Task M3) | `rooms/m25_content.rs` owns the unions (76/8/3/5/23/7), built from `content::watcher` and the M24 union, never copied. `act4_shop_rows` handles floor 52; `rewards::prepare_combat` gains an M25 Act4 card-child arm, and its Watcher elite arm uses `ACT4_ELITE_RELICS` in Act 4. `m23_content::relic_capture_scope` returns `run.m25.relic.capture` only for M25 in Act 4. Gold, the potion cycle and every key use the full global visit and the existing Watcher laws. The Act4 card-offer key uses `CombatRecord.visit` as design §6.2 says, not M10/M15/M20's Act4-local visit |
| Act4 sampling and HC0 (plan Task M3) | HC0's 47 new catalog rows are `definition_only` until their behaviour lands, so the shared sampler rejects the Purple families with `Unsupported("M25 incomplete Purple rarity family")` until HC1. `m25_act4_merchant_rows`, `m25_act4_elite_rewards_and_relic_capture` and `m25_act4_producers_exhausted_and_unaffordable` therefore first run green on H2b, as the plan's H2 gate allows |
| `WatcherAdmission::M25` (plan Task M3) | `normalized_for(M25)` checks the deck against `M25_WATCHER_HEART_CARDS` (Parasite only base) and the relics and potions through `validate_combat_inventory` under `with_m25_profile()`, the same owner the combat entry uses. `M21` and `normalized()` are byte-identical in behaviour |
| Supplied validation threading (plan Task M4) | As bound at M0: `m23::boss_source_for(.., admission)`, with `boss_source` a one-line M21 wrapper. `m22.rs` and `m24.rs` are not edited. A test shows `boss_source == boss_source_for(.., M21)` across bosses, scenarios and resources, and that `m24_supplied_traversal` equals the M21 source with its v2 loadout |
| `m25_supplied_traversal`'s Act2 boss (plan Task M4) | Design §12.4 names only the Act3 history. M25 uses M24's seed-7 canonical Act2 pairing (AwakenedOne–Champ, TimeEater–BronzeAutomaton, DonuAndDeca–Collector; M24 review N1), defined in M25's own module |
| `M25Diagnostic::HeartProceedFault` (plan Task M4) | The Heart Proceed publication cannot fail after its preflight. The constructed fault is therefore the Heart Victory Proceed *preparation* failure at settlement (exhausted choice ledger), retained once as `run.m25.heart.victory`. `DoorFault` is the constructed HP change between door preparation and commit (`run.m25.act4.transition`). `HeartVictoryPending` sets a constructed 999 max HP so the supplied lethal frontier survives the Heart's turns |
| M4 test location (plan Task M4) | The four M4 tests and the card-stage test live in the `#[cfg(test)]` module of `run/diagnostics/m25.rs`, beside the n2 tests, because they call `pub(super)` helpers (`act3_victory_source`, `m23::boss_source_for`). No public supplied-run API is added for tests |
| Native test location (plan Tasks N1–N2) | Tests that need an `m25-diagnostics` stage (`m25_lifecycle_no_cutoff_and_terminals`, `m25_delivery_fault_record`, `m25_encoding_act4_and_terminal`, `m25_numeric_privacy_and_capacity`, `m25_batch_rejects_step_with_terminal_slot`) live in `#[cfg(test)]` modules of `run_session.rs`, `run_observation.rs` and `run_batch.rs`, keeping their plan names. They use a `#[cfg(test)] pub(crate) RunSession::from_m25_diagnostic`, following the accepted M22/M24 precedent, so no public or debug-build diagnostic importer exists. The other M25 env tests are in `crates/spire-env/tests/run_m25_*.rs` |
| Legacy `profile()` in `crates/spire-env/src/run_schema.rs` | An exhaustive match, so the H1 snapshot carries one mechanical M25 rejection (M24 precedent). The Native lane owns it from H1 on |

## Integrations

- **HC0** (Content C1), integrated 2026-10-31 after PM verification: `c1.patch`
  (SHA-256 `26b32594…`, 15 Content-owned files) plus the PM-approved
  `selection-omniscience-arms.diff` (`c29ce2be…`). The diff adds four mechanical
  exhaustive-match arms for `SelectionOperation::Omniscience`, outside the Content lane's
  paths, in `crates/spire-env/src/run_schema_{m10,m12,m15}.rs` and
  `crates/spire-report/src/actions.rs`. This follows the M20/M23 precedent; no accepted
  value changes. Pre- and post-integration manifests are retained outside the repository.
  After integration: core lib 2116 passed, 0 failed, 3 ignored; spire-env lib 90/0/5;
  spire-report 55/0.
- **HC1** (Content C2–C6), integrated 2026-10-31. Pre/post manifests `aa57022d…`/`5ad1e58d…`.
  The first verification run found that the package omitted `run/diagnostics/m25.rs` changes
  (E0603); after the packaging fix the core lib was 2147/0/3.
- **HC2** (Content C7 compositions and the Burn-order fix R4), integrated 2026-10-31, five paths
  (pre/post `b107dd58…`/`1d332659…`). The R2 relic admission was already in Main's Task M3.
- **RQ-OMNI** (Content, several same-source autoplay continuations), integrated 2026-10-31 as
  full bytes over identical bases: `autoplay.rs`, `combat/tests/mod.rs` and the new
  `m25_watcher_nested_autoplay.rs` (pre/post `040c9d1d…`/`773a2e1e…`).
- **Native H3 and H3b-final**, integrated 2026-10-31: 14 Native-owned `crates/spire-env` files,
  13 of them changed (pre/post `51809b48…`/`6af6e3ed…`). The native env suite was 565/0/5 on
  its delivery.
- **Snapshots published to the lanes**: H1 (`ada7be9b…`), H2 (`36ac0f36…`), H2b (`0271b3b4…`),
  H2c (`fefca5e8…`) and H2d (the RQ-OMNI delta over H2c). The native lane built H3b-final on
  H2d. All manifests and patches are retained outside the repository.
- **Post-outcome N3 consumer** (test-only, PM-approved): see the N3 section.

## Plan review version 2 Minors

- **n1.** `BandageUp` has no native identity. The single card-tag owner marks HEALING on
  exactly six native kinds: Feed, Reaper, Alchemize, SelfRepair (its Defect metadata),
  LessonLearned and Wish. No `BandageUp` identity is added. The colourless 34 is the native
  catalog, which has no Bandage Up row; there is no exclusion list. `card_tags` defines only
  `Healing` and `Strike` for non-Defect kinds; `StarterStrike` and `StarterDefend` stay
  Defect-only. STRIKE is thirteen kinds, without Thunderclap.
- **n2.** Tests that call `act3_victory_source` live in a `#[cfg(test)]` module of
  `crates/spire-core/src/run/diagnostics/m25.rs`, following the M23 diagnostics tests.

## User ruling: Vault against Time Warp

The primary user ruled that Vault played as Time Eater's twelfth card negates Time Warp's
forced end of turn. Vault's own effect governs:

- the turn ends once;
- the enemy turn and the end of round are skipped;
- the player starts an extra turn, with player end-of-turn and start-of-turn effects running once each.

The retained static finding settles the open detail (`data/enemies/m8-static-findings.md`,
Time Warp). At exactly twelve, Time Warp resets its counter to 0 and queues Strength 2 for
every encounter actor before its early-end request, and the card's payload precedes that
after-use callback. Time Eater therefore still gains 2 Strength, and the counter reads 0.

This clarifies plan Review Focus 5. The enemy-turn skip is tested explicitly. The ruling applies
to the shared Time Warp and turn owners for every character. The user's authority covers it, so
no plan re-review is needed.

## Inherited global correction

M24-D1, the Time Warp barrier draining queued ordinary work before the forced end, is inherited
unchanged ([M24 reconciliation](m24-watcher-act3-reconciliation.md)).

## Global correction: Perfected Strike and the card-tag owner

Perfected Strike counted the hard-coded `has_strike_tag` list
(`crates/spire-core/src/combat/cards/ironclad.rs`). That list included Thunderclap and omitted
Wild Strike, Swift Strike, Sneaky Strike, Windmill Strike, Thunder Strike and Meteor Strike.
The accepted test `perfected_strike_counts_distinct_strike_tags_in_hand_draw_and_discard_only`
(`crates/spire-core/src/combat/tests/full_cards.rs`) pinned the wrong set: 14 base and 18
upgraded, where the source set gives 12 and 15. The fix (plan Task C5) makes `has_strike_tag`
read the single card-tag owner (`content::is_strike`). The assertion is amended from `(0, 14), (1, 18)`
to `(0, 12), (1, 15)`, and the original bytes remain at base `7b036e8d`. The sweep of
retained outcomes (H2c consolidated run) found six retained fixtures that contain Perfected
Strike or STRIKE-named cards. All six pass, and no retained outcome changed.

## Global correction: Omniscience continuations in the autoplay census

`validate_autoplay_runtime` (`crates/spire-core/src/combat/engine.rs`) built its queued-autoplay
reference census without the Watcher `OmnisciencePlay` and `OmniscienceRepeat` continuations.
When Omniscience chose another Omniscience, the nested draw-pile choice paused with a queued
child that had no counted continuation. The step itself succeeded, but post-step validation
failed, so the Run rejected an offered public `SelectCard` with `InvalidAction`. In StS,
Omniscience can choose another Omniscience and the chain continues (offline wiki snapshot,
revision 66667; community reference). The fix adds both variants to the census and to the
faulted-deferred-child match. It is a shared validator correction, not an Omniscience guard.
The `OmniscienceRepeat` validator in `crates/spire-core/src/combat/cards/watcher.rs` also
required the repeat to own the top play frame. A repeat still queued behind its source while a
nested choice is open therefore failed validation. It is now also owned while its source card is
in play or suspended as the parent of a play frame. This follows the shared precedent of Burst's
repeat (`combat/cards/silent/state.rs`). Both changes are needed.
`engine.rs` changed from the HC1 bytes `dc064868…` to `efbb246a…`. The Main regression test is
`m25_omniscience_choosing_omniscience_chains_through_public_actions`, and the M6 caller test
`m25_caller_new_selections_close_and_return_to_play` found the defect.

## Global correction: several same-source autoplay continuations (Content RQ-OMNI)

A card with more than one queued autoplay continuation from the same source faulted. For
example, an Omniscience chosen by another Omniscience owns its own choice's play and also its
repeat. The shared `defer_parent_finish` (`crates/spire-core/src/combat/cards/ironclad/autoplay.rs`)
suspended the parent for only the first continuation, and `resume_parent` then finished the
parent. The Content lane's `next_parent_continuation` re-suspends the same parent for each
further continuation. The defect was shared: Burst + Havoc, Duplication Potion + Havoc and
Double Tap + Duplication also faulted before the fix. Content added four RED→GREEN regressions
in `combat/tests/m25_watcher_nested_autoplay.rs`. Main integrated the full bytes after
confirming identical bases (`autoplay.rs` `5c0dd62c…` → `633c86e1…`) and kept its own
`engine.rs` and `watcher.rs` changes.

## Global correction: Watcher recharge and Energized

The Watcher's deferred turn-start recharge (`crates/spire-core/src/combat/cards/watcher.rs`)
set energy to `max_energy − Energy Down` and ignored Energized. The shared recharge in
`combat/turns.rs` skips Energized for deferred characters. A Watcher's Energized, for example
from a Flying Knee obtained through Foreign Influence, therefore granted nothing and was never
removed. The recharge now follows the StS order: recharge, then Energized (and Energized blue)
added and removed, then Energy Down, floored at 0. An M26 plan reviewer found it by reading the
code, and the PM forwarded it.

## Public previous card type

The native observation must encode the type of the previous card played this combat (design
§11). Core kept it private. `CombatPublic.previous_card_type` now publishes it, filled only
for a Watcher combat under the M25 item profile. Debug prints it only when it is set, so
older profiles' public views and Debug output are unchanged. Accepted test literals that build
`CombatPublic` gained `previous_card_type:None` mechanically, with no assertion change:
`examples/m15_support/caller_tests.rs`, `tests/m20_caller_policy.rs`,
`tests/m21_watcher_combat.rs`, `tests/m22_watcher_act1.rs`, `tests/m23_watcher_act2.rs`,
`tests/m24_watcher_act3.rs` and `crates/spire-env/tests/run_m15_schema.rs`.

## Task M5 and M6 notes

- `ActiveCombat.applied_upgrade_ordinal` is new, and Debug prints it only when it is non-zero.
  The `ActiveCombat` literals in the accepted tests `run/tests/{combat,m10_gains,m14_silent_act3,
  m8_acquisition,m8_power_census}.rs` gained `applied_upgrade_ordinal:0`. No assertion changed.
- Lesson Learned's in-combat copy keeps its upgrades (Content C3 decision). The settlement
  exit check therefore subtracts the applied journal upgrades per permanent instance.
- Of the `m25_content` and `m25_events` tests, only the Lesson Learned persist-once test was
  RED when upgrade settlement was stubbed. The other five pass on existing owners, so they are
  recorded as characterization tests.
- The constructed `OmniscienceChoice` stage uses 8 upgraded Omniscience (5 in hand, 3 in draw),
  not 10. With 10, a chained choice needs five caller steps, and the plan bounds the
  selection-closing caller test at four.
- The shared caller (`examples/m13_support/mod.rs`) gains `CallerProfile::M25`,
  `M25CallerOutcome`, `Caller::new_m25` and `rollout_m25`. The M25 branch reuses the shared
  Heart ending check `heart_stop_outcome` for `M25WatcherHeart` and the M22-style observed step.
  The M20–M24 caller targets are unchanged and pass.

## Global correction F1: a power-owned selection during a card play

Found by the once-only supplied Act4 coverage-B case under freeze v1. At floor 53 (Shield and
Spear) the Calm Watcher, with a full hand and an empty draw pile, played the offered
InnerPeace. Its Draw 3 recycled the discard pile, drew one card into the last free hand slot,
and Melange opened its Scry on the shuffle. The step succeeded inside the combat, but the
post-step validator rejected the state, so `Run::step` returned `InvalidAction` for a public
candidate. The validator for `Hook::CardAfterUse` and `Hook::CardAfterPlayed`
(`crates/spire-core/src/combat/engine.rs`) required any pending selection to have the playing
card as its `SelectionParent::Card`. Melange's Scry has `SelectionParent::Power` and may
legitimately interrupt the play (StS: the shuffle's relic action resolves before the draw
continues). The general pending-selection validator already pins every card-parented
selection to the in-play card, so the extra clause added nothing for card parents and wrongly
rejected power parents. The clause is removed for all characters.
`engine.rs` changed from `e58c3c88…` to `690ed2db…`. The regression tests are in
`combat/tests/m25_watcher_selection_during_play.rs`: the Melange + InnerPeace case failed with
the invariant before the fix and passes after it, and two controls (no Melange; an Ironclad
full-hand draw after a recycle) pass both before and after.

## Freeze, once-only outcomes and the F1 corrective freeze

- **Freeze gate and freeze v1.** On the exact frozen copy, all 21 gate commands exited 0
  (workspace 3387/0/10; feature-on core 2762/0/5). The freeze v1 record (`b0e4c5ff…`) holds the
  1005-file source manifest, the three sealed executables, the caller files, every case,
  loadout and fixture, the M24 comparison control and 22 zero-action initial snapshots. The
  M25 examples have no prepare-only mode, so an out-of-tree helper that compiles the frozen
  example support files produced the initials. It calls the constructors and `initial_value`
  only.
- **Once-only v1 (12 invocations).** The six canonical cases all ended in Death in Acts 1–2
  (146, 232, 425, 133, 324 and 114 accepted actions). The death control ended in Death after 9
  actions. Budgets 0 and 1 stopped unfinished. The four input errors were rejected before any
  Run was built. In the supplied traversal, TimeEater and DonuAndDeca reached `HeartVictory`
  with every key earned in play; AwakenedOne died at the Act3 boss. Act4 coverage A and C ended
  in Death at Shield and Spear. Coverage B hit fault F1, and the frozen strong-case rule stopped
  the series, so the strong case and the eight fixtures did not run.
- **PM ruling and freeze v2.** The fix above, then a no-drift replay. All 21 completed v1
  transcripts were replayed by recorded candidate index on the fixed source and matched step
  for step (decision, map, action, effects, boundaries, transition and endpoint). Coverage B's
  31-step prefix also matched, and its failed action is now accepted. Freeze v2
  (`879fbb1b…`) changes exactly six paths from v1: the fix, its test file and module line, and
  the disclosed caller option. That option, `--series f1-corrective --v1-supplied DIR`, reruns
  only coverage B and then applies the frozen strong-case rule to A (v1), B (v2) and C (v1)
  from receipts whose SHA-256 is pinned in source, followed by the fixtures. With no
  arguments the example runs the v1 series. Gate v2 passed: workspace 3392/0/10; feature-on
  core 2767/0/5. The v2 initials are byte-identical to v1.
- **Once-only v2 (one invocation).** Coverage B ended in Death at Shield and Spear (66
  actions). No primary case reached `HeartVictory`, so the strong v1 case ran and reached
  `HeartVictory` (55 actions). All eight fixtures ended in Death. The v1 outcomes, including
  the faulted coverage-B receipt, are kept as pre-correction history.
- **M24 comparison (design §4.2 check 4).** Each canonical M25 case equals the M24 comparison
  control run on the same frozen tree, and also M24's retained receipts, step for step
  through the endpoint. One normalization is applied: the content-profile name, and the
  M25-only `previous_card_type` field is removed. That field is checked against its owner by
  `assert_prefix` in `tests/m25_watcher_heart.rs`. No retained-receipt difference needs
  attribution.

## Global correction G1: generated Watcher cards built without their M25 initializer

The M26 lane found this while building on frozen M25, and the PM ruled that it be fixed
before merge. `card_state::generated` (`crates/spire-core/src/combat/card_state.rs`) built an
M25 Watcher kind from the bare permanent descriptor, without the M25 initializer. Every M25
constructor applies that initializer by hand afterwards, so only shared owners that call
`generated` directly saw the bare card. One example is Stasis's returned card
(`combat/enemies/act2/stasis.rs`, which reads `exhaust_on_finish` from `generated`): a Talk to
the Hand stolen by a Bronze Orb came back without Exhaust. `generated` now applies
`initialize_m25_base_profile` to every `supports_m25_generated` kind. The assignments are
idempotent, so the M25 callers are unchanged. The fix has the same shape as the M26 lane's
fix, so M26's rebase merges cleanly. The regression tests are in
`combat/tests/m25_watcher_generated_construction.rs`: every Watcher kind, base and upgraded,
must build to its M25 constructor's combat-local fields, and a real Bronze Automaton Stasis
steal-and-return must keep Talk to the Hand's Exhaust. This is a post-freeze correction. No
outcome is rerun. Every v1 and v2 transcript (31 receipts) is replayed with the same recorded
actions on the fixed source as a no-drift check. The PM retains the evidence (`fault-G1/a25`), and
the final stable run follows it. Any divergence would go back to the PM before the candidate.
The replay found one: v1 Act4 coverage C from step 26 on. At step 25, Foreign Influence+ chose Talk to
the Hand, and v1 built it without Exhaust (the G1 defect). The PM ruled that v1 coverage C after step
25 is kept unchanged as pre-correction history. A narrow freeze v3 reruns only coverage C, once,
through the disclosed caller option `--series g1-corrective --v1-supplied DIR --v2-supplied DIR`. That
option reads the retained A (v1) and B (v2) receipts, whose SHA-256 is pinned in source. It evaluates
the frozen strong-case rule over A, B and the new C for information only; the v2 strong case stays as
executed history. N3 uses the v3 coverage-C receipt.

- **Freeze v3** (`5577735e…`) changes 16 paths from v2. They are the G1 and G2 fixes and their tests, the g1-corrective
  option and its test, the N3 consumer and the native codec fix, and documents. A first gate run (v3) is void: its fault-reproduction
  copy and its gate copy shared one Cargo target directory, so cargo's dependency info pointed at the other tree and the gate
  ran a stale pre-G2 binary. The gate was rerun on fresh, separate targets (v3b) and passed all 23 commands: workspace
  3398/0/11; feature-on core 2772/0/5; G1 and G2 tests 2/0 each. The no-drift replay on the v3 source showed only the two
  known deltas (v1 coverage C from step 26; the v1 coverage-B fault endpoint). The v3 initials are byte-identical to v1.
- **Once-only v3 (one invocation).** Coverage C ended in Death at Shield and Spear (36 actions). The informational
  strong-case rule over A (v1 Death), B (v2 Death) and C (v3 Death) gives "run the strong case". That agrees with the v2
  execution; the strong case was not rerun.

## Global correction G2: Deus Ex Machina in hand without a draw

The M26 lane also found this. Deus Ex Machina is Unplayable, and only its on-draw trigger acts.
The shared unplayable list in `crates/spire-core/src/combat/cards/mod.rs` omitted it. A Deus Ex Machina
that reached the hand without being drawn therefore made legal-action enumeration return
`Unsupported` for every character, the Watcher included. In M25 this happens when a Bronze Orb's
Stasis captures it from the draw pile and returns it to hand. It is now in the unplayable list, the
same fix as M26's. The regression tests are in `combat/tests/m25_watcher_unplayable_in_hand.rs`:
Deus Ex Machina placed in hand (base and upgraded), and a real Stasis steal-and-return. It is
covered by the same no-drift replay; no retained transcript reached that state.

## Recorded native agreement (Task N3, post-outcome, test-only)

The PM approved N3 as a disclosed addition made after the outcomes. It consumes the retained
receipts and cannot change a frozen outcome. The consumer `m25_recorded_native_agreement` is
ignored by default. It lives in the `#[cfg(test)] mod m25_tests` of
`crates/spire-env/src/run_session.rs`, not in `tests/run_m25_batch.rs` as the plan named. The
supplied starts need crate-internal constructors, so it follows the M24 supplied-agreement
precedent and the M20 `#[cfg(test)]` batch constructor. The test-only helpers are
`RunSession::from_m25_supplied` and `RunBatch::from_m25_supplied`/`m25_test_boundaries`; no
public or debug-build importer exists. Run it with
`SPIRE_M25_RECORDED_RELEASE=<release> SPIRE_M25_RECORDED_RELEASE_SHA256=<sha> cargo test -p spire-env --lib m25_recorded_native_agreement -- --ignored --nocapture`.
The release lists 24 transcripts with their SHA-256 and freeze:

- freeze v1: 6 canonical, 3 controls, 3 traversal, Act4 A and C;
- freeze v2: Act4 B, the strong case and 8 fixtures.
- release v2 (after the G1 ruling): Act4 C is taken from freeze v3 instead of v1. The consumer accepts an optional third
  freeze; this is a test-only change.
- The final stable sequence after freeze v3 had three runs. Run-v3 ran the full matrix, but N3 was held because the consumer
  was still pinned to release v1. Run-v3b came after the test-only pin update to release v2 and gave N3 23/24, because the
  consumer's preserved-freeze list lacked v3. Run-v3c came after adding v3 to that list and gave N3 24/24. Each delta changed
  only `crates/spire-env/src/run_session.rs`, inside `#[cfg(test)] mod m25_tests`.

Two receipts are excluded. The faulted v1 coverage-B receipt is pre-correction history, and the
no-drift replay covers its prefix. The M24 control is M24 evidence.

For every case the consumer decodes the recorded semantic actions with the strict profile
codec, applying a fixture's bootstrap Move first. It drives a native single session from the
matching initializer and compares every boundary, reward, combat record and terminal. Normal
starts are also round-tripped through strict JSON and replayed; supplied starts are replayed
from the same initializer. Homogeneous batches with workers 1 and 2 are run for each series.
**Result: 24/24 agree, with no differing step.**

**Native defect found by N3 and fixed: M25 strict replay of card-offer actions.** The strict
replay codec (`crates/spire-env/src/run_replay.rs`, `parse_action`) admitted
`combat.choose_card_offer` and `combat.skip_card_offer` only for M15 and M20. M25 offers them
(Foreign Influence), so an M25 record with a Foreign Influence choice could not be decoded. The
first failure was Act4 coverage C, step 25. Both arms now admit `M25WatcherHeart`; no other
profile changes. The regression test is `m25_strict_replay_decodes_card_offer_actions` in
`crates/spire-env/tests/run_m25_replay.rs`, which fails before the fix and passes after it.
After the fix: spire-env lib 98 passed, 6 ignored; full env tests 566 passed, 6 ignored, 0
failed. No once-only outcome depends on this codec, because the examples use the core Run directly.

## Final review v1 corrections

The fresh independent final review v1 returned NOT READY: 0 Critical, 3 Important and 5 Minor findings. The PM ruled on each.
The fixes below are global corrections at source law for every character. The amended accepted assertions keep their
original bytes in the M25 evidence (`fault-I1/original-bytes`).

**I-1: Master Reality creation law.** Master Reality reads "Whenever a card is created during combat, Upgrade it". The
candidate upgraded only Watcher fixed children and Foreign Influence outputs that entered the hand. Cards shuffled into the
draw pile were never upgraded: Alpha → Beta → Omega, Conjure Blade's Expunger, Reach Heaven's Through Violence, and the Insight
from Evaluate, Pray and Study. The wiki snapshot (`data/cards/sources/wiki-pages.json`, Evaluate, Pray, Study, Reach Heaven,
Conjure Blade and Alpha) says each of these is upgraded.

The shared creation owner `piles::insert_generated` now applies the upgrade for every character. It upgrades a base card once
and never a Curse or a Status, in every destination, a hand-overflow copy included. Two hand routes have their own source
checkpoints and keep them, through `MasterRealityRoute::RouteChecked`:

- **Watcher fixed children** (Carve Reality, Deceive Reality, Battle Hymn, Pure Water, Bottled Miracle and Deus Ex
  Machina). `MakeTempCardInHandAction`'s constructor upgrades the template when Master Reality is already
  installed, and the hand helper checks again when the card enters the hand. The overflow helper upgrades only a display
  copy (M21 source rulings, accepted in M21). M25 had built a base template and skipped the constructor check. That
  difference is removed, so M25 now follows M21. One consequence: Deus Ex Machina drawn into a full hand, with Master
  Reality already installed, now sends upgraded Miracles to discard.
- **Foreign Influence** has no action-level check (`docs/research/2026-10-19-act1-complete-play-origins.md`, Foreign
  Influence row). Its hand helper upgrades the copy that enters the hand, and its overflow helper upgrades only a display
  copy. A Foreign Influence output that overflows to discard therefore stays at its base upgrade.

The accepted assertions changed are T12 (`m25_t12_generation_and_master_reality_at_act4`: the draw-pile children are now
upgraded) and `m25_generation_full_hand_overflow` (the Deus Ex Machina overflow above). Each draw-pile generator's route is
pinned in T12, and every destination by `m25_master_reality_upgrades_creations_in_every_destination`. Both fail on the
candidate-v1 source. The M21 Master Reality tests are unchanged.

**I-3: R4, Burn retention.** Burn "discards itself even when otherwise retained or kept by Runic Pyramid"
(`data/cards/implementation-notes.json`, BURN). Before M25, a Burn chosen by a retain effect stayed in hand. For every
character, `EffectKind::DiscardHand` (`piles.rs`) now removes each Burn from `temporary_retain` and clears its local retain
before both the generic and the Watcher discard paths. Only the retention is withdrawn: the Burn leaves in the ordinary
hand-discard order, so the discard order of a hand without retention is unchanged. Runic Pyramid is not implemented in any
accepted profile. Tests:

- `m25_t08_spear_burns_retain_and_wrath` covers the Watcher path.
- `m25_silent_well_laid_plans_cannot_retain_a_burn` (`combat/tests/m25_burn_retain_all_characters.rs`) covers the generic
  path. It fails when the `temporary_retain` withdrawal is removed, as a Watcher-only fix would do.

**Card queue on EndPlayerTurn.** For every character, `EffectKind::EndPlayerTurn` (`turns.rs`) cancels the pending card
queue (`cancel_pending_card_queue`), as the source's `ClearCardQueueAction` does. In practice only Vault reaches it with a
non-empty queue.

**m-5: Collect and Chemical X.** Collect is an X-cost card and now adds Chemical X's +2 to its turns, in both the payload and
the validator, as Conjure Blade already did. Test: `m25_collect_counts_chemical_x`. M25 cannot reach this, because Chemical X
is outside its relic union.

**Retained-outcome sweep.** After these fixes, every retained one-time transcript was replayed with its recorded actions on
the fixed source (32 receipts: the 6 canonical cases, 3 controls, 6 M24 controls, 3 traversal cases, Act4 A, B (v1 and v2) and
C (v1 and v3), the v2 strong case and the 8 fixtures). 30 are byte-identical. The other two are the known pre-correction
deltas: v1 coverage C from step 26 (G1) and the v1 coverage-B fault endpoint (F1). The v3 coverage C, which plays Master
Reality+, is identical, so no corrective freeze v4 was needed and every recorded outcome stands. R4 and the card-queue
cancel were already in every M25 freeze. For earlier profiles, R4 was checked when it was made (content lane HC2): the
accepted M13 caller transcripts (`m13_silent_act2::m13_caller_transcripts_unchanged_by_m14_extension`) pass with it, and the workspace suite in the final stable run
includes every accepted profile's transcript and caller tests.

## Frontiers and disclosed project decisions (design §5.9)

This table separates source-settled facts from choices made for open parts. “Project decision” means the choice is not established by source law; cited tests pin that finite choice. Every named test exists in the tree and runs in the final stable run.

| ID | Topic | Settled by a cited source fact or open part | Binding / decision as implemented | Reasoning (why this choice) | Pinning test (file::fn) |
|---|---|---|---|---|---|
| F1 | Foreign Influence support and proposal law | Source: any-colour proposals, rarity bands, exclusions, and whole-proposal duplicate retry (docs/research/2026-10-19-act1-content-watcher.md:380-391; data/cards/sources/wiki-pages.json:963). | Source-settled: rolls 0–99 map Common 0–54, Uncommon 55–84, Rare 85–99; exact support is 106 ordinary Attack kinds (43/48/15), no colour predicate, excludes HEALING/Curse/Status/Basic/Special; duplicates retry the whole proposal. | Follows source rarity/type/tag facts and the complete native catalog; eligibility is not inferred from colour or card text. | crates/spire-core/src/combat/tests/m25_watcher_generation.rs::m25_identity_catalog_rows_and_capability; delivery: crates/spire-core/src/combat/tests/m25_watcher_interactions.rs::m25_t12_generation_and_master_reality_at_act4 |
| F2 | Vault, Time Warp, and skipped round | Source: player end/start effects run while enemy and end-of-round work are skipped; Blasphemer still fires (data/cards/implementation-notes.json:103-105). Time Warp is separately fixed by the primary-user ruling (docs/milestones/m25-watcher-heart-reconciliation.md:106-122), not by the Vault source note. Invincible reset is open in design §5.9 F2. | Project decision under the user ruling: Time Warp’s twelfth-card reset and queued Strength resolve before Vault takes control; Vault then performs one end, skips enemy/end-round callbacks, and starts one extra player turn. Invincible is covered by F2-a. | Preserves the shared Time Warp callback while honoring the explicit ruling that Vault owns the single turn ending. | crates/spire-core/src/combat/tests/m25_watcher_powers.rs::m25_powers_stacked_turn_endings; crates/spire-core/src/combat/tests/m25_watcher_interactions.rs::m25_t09_vault_against_act4 |
| F3 | Blasphemy start-turn ordering | Source: Blasphemer is a positive buff and requests 99,999 HP loss at next player-turn start (data/cards/implementation-notes.json:23-25). Ordering against other start-turn work is open in design §5.9 F3. | See F3-a: project decision is to resolve Blasphemer HP_LOSS before later Watcher start-turn Collect creation; lethal outcome suppresses that callback. | Follows the HP_LOSS and terminal boundary rather than draining later callbacks after death. | crates/spire-core/src/combat/tests/m25_watcher_powers.rs::m25_powers_stacked_turn_endings |
| F4 | Omniscience and Time Warp | Source: selected card is played twice without normal energy; X uses remaining energy; a turn-ending child may interrupt (data/cards/implementation-notes.json:76-78). Special Time Warp ordering and unplayable selections remain open in design §5.9 F4. | Project decision: selection and each child play resolve before Omniscience’s after-use counter; children count separately. A child that ends the turn cancels the queued repeat. Unplayable child behavior is F4-b. | Reuses shared card-use and turn-end continuations without inventing a general queue-drain rule. | crates/spire-core/src/combat/tests/m25_watcher_interactions_w.rs::m25_w15_timewarp_turn_control; crates/spire-core/src/combat/tests/m25_watcher_state.rs::m25_state_judgment_and_omniscience; crates/spire-core/src/combat/tests/m25_watcher_powers.rs::m25_powers_stacked_turn_endings |
| F5 | Melange shuffle-triggered Scry | Source: Scry 3 follows a reshuffle after the causing draw finishes; insertion is not a reshuffle; Golden Eye raises it to 5 (data/relics/relics.json:5175-5182). Design §5.9 F5 leaves composition with Weave/Nirvana and an in-progress card play to bind. | Project decision: dispatch the power-parented Scry after the draw, using shared Scry for Weave and Nirvana. During a card play it may suspend the play; the pending card continuation remains valid and resumes after selection (F5-a; the F1 correction). | Preserves the after-draw checkpoint and the pending card-use continuation. | crates/spire-core/src/items/tests/m25_watcher.rs::m25_items_new_relics_through_real_combat; crates/spire-core/src/combat/tests/m25_watcher_selection_during_play.rs::m25_melange_shuffle_scry_during_inner_peace_keeps_the_play_valid |
| F6 | Wave of the Hand and Duality | Source: Wave reacts to positive Block and expires at round end (data/cards/sources/wiki-pages.json:2085; research :127). Duality follows Attack resolution; its own Block precedes Dexterity (data/relics/relics.json:2755-2762). Exact shared checkpoint was open in design §5.9 F6. | Project decision: grant Duality at shared after-use, after the complete Attack payload; one actual Attack grants one temporary Dexterity (F6-a). | The after-use owner observes the completed payload, so Dexterity cannot change the Attack’s own Block. | crates/spire-core/src/items/tests/m25_watcher.rs::m25_items_new_relics_through_real_combat |
| F7 | Cloak Clasp end-turn Block | Source: one hand-count Block event after Orichalcum and before Ethereal/discard (data/relics/relics.json:1971-1978). Order against Like Water, Retain, Burn, Omega, and Study is open in design §5.9 F7. | Project decision: emit one hand-count event at CloakClaspBeforeCleanup; the finite test pins Orichalcum and Ethereal/discard boundaries, not every listed pair. See F7-a and report. | One request matches the record and applies shared Block modifiers once. | crates/spire-core/src/items/tests/m25_watcher.rs::m25_items_new_relics_through_real_combat; related Vault ordering: crates/spire-core/src/combat/tests/m25_watcher_interactions.rs::m25_t09_vault_against_act4 |
| F8 | Lesson Learned target and RNG | Source: qualifying fatal upgrades an eligible permanent-deck instance and excludes specified non-final/minion cases (data/cards/implementation-notes.json:67-68; research :516-523). Native RNG stream is open in design §5.9 F8. | Project decision (F8-a): use RunCombatCardRandom, journal the permanent-deck upgrade for settlement, and leave the in-combat copy unchanged. | The source names misc RNG but no distinct native stream; the shared card-random owner already handles native random card choices. | crates/spire-core/src/combat/tests/m25_watcher_state.rs::m25_state_lesson_learned_requests_permanent_upgrade; crates/spire-core/src/combat/tests/m25_watcher_interactions.rs::m25_t11_lesson_learned_and_wish_settlement |
| F9 | Judgment target death boundaries | Source: at action-time HP ≤30/40 against a monster, request Instant Kill, not damage (data/cards/sources/wiki-pages.json:1179; research :146,560-566). Invincible, half-dead, Awakened One, and minion interactions are open in design §5.9 F9. | Project decision: pass the threshold result into the shared InstantKill/death lifecycle. Judgment does not spend Invincible; Darklings and Awakened One retain half-death/rebirth; minions use their normal death owner. | Reuses downstream death, revival, and encounter rules rather than adding a card-specific defeat path. | crates/spire-core/src/combat/tests/m25_watcher_state.rs::m25_state_judgment_and_omniscience; crates/spire-core/src/combat/tests/m25_watcher_interactions_w.rs::m25_w17_mark_intangible_and_half_dead; crates/spire-core/src/combat/tests/m25_watcher_interactions.rs::m25_t02_invincible_allowance |
| F10 | Deus Ex Machina draw trigger | Source: actual draw creates Miracles then exhausts the card; insertion/movement is not an on-play trigger (data/cards/implementation-notes.json:47-48; snapshot data/cards/sources/wiki-pages.json:633). | Source-settled: only an actual draw triggers; insertion, Scry, and Meditate do not. | Uses the named draw callback rather than treating zone entry as a draw. | crates/spire-core/src/combat/tests/m25_watcher_powers.rs::m25_powers_constructor_stack_and_callbacks; crates/spire-core/src/combat/tests/m25_watcher_interactions.rs::m25_t01_beat_of_death_counts_watcher_plays |
| F11 | Talk to the Hand qualifying later hits | Source: own hit precedes installation; qualifying later callbacks include zero/fully blocked hits but exclude THORNS/HP_LOSS (data/cards/sources/wiki-pages.json:1947; research :494-500). Invincible-capped hits are open in design §5.9 F11. | Project decision (F11-a): each normal Attack damage callback qualifies even when Invincible caps HP loss; THORNS and HP_LOSS remain excluded. | Follows attack-hit callbacks rather than every form of HP loss; the finite Heart case pins the Invincible edge. | crates/spire-core/src/combat/tests/m25_watcher_powers.rs::m25_powers_constructor_stack_and_callbacks; crates/spire-core/src/combat/tests/m25_watcher_interactions.rs::m25_t04_painful_stabs_and_block_return |
| F12 | Wallop and damage accounting | Source: after target damage, positive lastDamageTaken requests Block (data/cards/sources/wiki-pages.json:2073; research :509-514). Block, Invincible, overkill, and terminal continuation were open in design §5.9 F12. | Project decision for pinned cases: read shared post-damage lastDamageTaken; Block/no HP loss gives no Block, Invincible caps it to HP lost, overkill caps it to remaining HP. No broader post-terminal guarantee is asserted. | Uses the post-damage field rather than nominal damage that did not remove HP. | crates/spire-core/src/combat/tests/m25_watcher_state.rs::m25_state_signature_wallop_tantrum |
| F13 | Deva Form, Fasting, and Simmering Fury start turn | Source: Deva grows recharge; Fasting Energy Down is separate (data/cards/implementation-notes.json:44-45; research :121,143,485-492). Order is open in design §5.9 F13. | See F13-a: project decision sets recharge synchronously to max energy minus Energy Down; Deva gain resolves before Simmering Fury stance/draw. Calm exit, Violet Lotus, Rushdown, and Mental Fortress continue through shared stance hooks. | Keeps Energy Down separate from Deva growth and uses shared recharge/stance owners. | crates/spire-core/src/combat/tests/m25_watcher_powers.rs::m25_powers_constructor_stack_and_callbacks |
| F14 | Swivel entitlement and Wreath Vigor | Source: Swivel has no turn expiry and qualifying non-purge Attack use decrements it; Wreath adds Vigor through normal damage and removes it on Attack use (research :123,132; snapshots data/cards/sources/wiki-pages.json:1923,2157). Cost/autoplay/created-attack and multi-hit/repeat details were open in design §5.9 F14. | Project decision: every non-purge Attack use consumes Swivel at CardPlayed, including zero/fixed-cost, autoplayed originals, and created Attacks; purge repeats do not. Wreath adds Vigor to each normal multi-hit then removes it after that Attack, so an Omniscience repeat gets none. | One shared use hook covers Attack routes; damage-give and after-use hooks pin the finite Vigor sequence. | crates/spire-core/src/combat/tests/m25_watcher_powers.rs::m25_powers_constructor_stack_and_callbacks |
| F15 | Off-colour Attacks and character context | Source: Foreign Influence is an explicit any-colour exception; nested context-sensitive generation keeps the actual Run character (research :373-378). Searing Blow has an MR generation quirk (data/cards/implementation-notes.json:99-101; snapshot data/cards/sources/wiki-pages.json:1683). The exact Searing Blow result remains open in design §5.9 F15. | Source-settled: selecting an off-colour Attack does not change Run character context. Project decision: preserve that context and use the shared Master Reality creation owner; add no Searing Blow-specific override. | The any-colour exception belongs to Foreign Influence; it does not broaden unrelated context-bound generators. | crates/spire-core/src/combat/tests/m25_watcher_generation.rs::m25_identity_construction_admission; crates/spire-core/src/combat/tests/m25_watcher_interactions.rs::m25_t01_beat_of_death_counts_watcher_plays |
| F16 | Fear No Evil, Signature Move, Spirit Shield, and Scrawl | Source card snapshots cover these cards (data/cards/sources/wiki-pages.json:855,1677,1767,1815); Spirit Shield count/modifier rule is also data/cards/research-annotations.json:85-88. Scrawl stop conditions are open in design §5.9 F16. | Fear No Evil mapping is the project decision in F16-a/F16-b. Signature Move counts another Attack even if unplayable; Spirit Shield counts other hand instances once and applies modifiers once. Project decision (F16-c): Scrawl stops at the shared draw owner’s available-card boundary if both piles are empty; a drawn Deus Ex Machina resolves during the loop. | Shared owners keep legality, Block, and draw behavior consistent; only intent mapping and the empty-pile boundary need a project binding. | crates/spire-core/src/combat/tests/m25_watcher_direct.rs::m25_direct_base_plus_values_and_flags; crates/spire-core/src/combat/tests/m25_watcher_state.rs::m25_state_signature_wallop_tantrum, ::m25_state_zero_x_and_empty_pile_edges; crates/spire-core/src/combat/tests/m25_watcher_interactions.rs::m25_t01_beat_of_death_counts_watcher_plays |
| F2-a | Vault: Invincible reset | Open part: design §5.9 F2 says Invincible resets at a monster-turn checkpoint, which Vault skips. | Project decision: reset only at MonsterStart; Vault leaves the current allowance unchanged. | Ties reset to the shared monster-start owner Vault skips. | crates/spire-core/src/combat/tests/m25_watcher_powers.rs::m25_powers_stacked_turn_endings; crates/spire-core/src/combat/tests/m25_watcher_interactions.rs::m25_t09_vault_against_act4 |
| F3-a | Blasphemy before Collect | Open part: design §5.9 F3 leaves order against other start-turn work unsettled. | Project decision: Blasphemer HP_LOSS resolves before Watcher Collect creation; lethal outcome suppresses that later callback. | Uses the terminal boundary rather than forcing queued work to drain after death. | crates/spire-core/src/combat/tests/m25_watcher_powers.rs::m25_powers_stacked_turn_endings |
| F4-a | Omniscience nested choice order | Open part: nested Omniscience order is unresolved (docs/research/2026-10-19-act1-content-watcher.md:578; comment crates/spire-core/src/combat/tests/m25_watcher_state.rs:143-146). | Project decision: reuse Seek’s (kind, instance) order, not draw order; selection does not publish draw order. | Reuses deterministic selection presentation without claiming it is source ordering. | crates/spire-core/src/combat/tests/m25_watcher_state.rs::m25_state_judgment_and_omniscience |
| F4-b | Omniscience with an unplayable child | Source Clash note: an unplayable indirect child is exhausted without being played (data/cards/implementation-notes.json:31-33); exact Omniscience composition is open in design §5.9 F4. | Project decision: shared autoplay suppresses the child, then disposes the selected original under Omniscience’s Exhaust disposition; no child CardPlayed or purge copy. | Preserves the imposed Exhaust without fabricating a play or second object. | crates/spire-core/src/combat/tests/m25_watcher_state.rs::m25_state_judgment_and_omniscience |
| F5-a | Melange selection during a card play (once-only F1 correction) | Source: power-parented Scry follows the draw/reshuffle boundary (data/relics/relics.json:5175-5182). Validator correction and evidence are recorded at docs/milestones/m25-watcher-heart-reconciliation.md:213-230. | Project decision: Melange Scry may suspend Inner Peace’s draw while retaining pending CardAfterUse/FinishCard; confirming Scry resumes and finishes the card play. | Removes the invalid parent restriction while preserving both the relic selection and card continuation. | crates/spire-core/src/combat/tests/m25_watcher_selection_during_play.rs::m25_melange_shuffle_scry_during_inner_peace_keeps_the_play_valid |
| F6-a | Duality timing against the Attack payload | Source says the Attack’s Block resolves before Dexterity (data/relics/relics.json:2755-2762); exact hook placement was open in design §5.9 F6. | Project decision: grant Dexterity at shared after-use, after the complete Attack payload; Skills do not trigger it. | Honors the own-Block order and uses one after-use callback per Attack. | crates/spire-core/src/items/tests/m25_watcher.rs::m25_items_new_relics_through_real_combat |
| F7-a | Cloak Clasp hand-count event | Source: one event after Orichalcum and before Ethereal/normal discard (data/relics/relics.json:1971-1978). | Project decision: take one hand-size snapshot at CloakClaspBeforeCleanup and request one Block gain; the test does not settle every F7 interaction. | One request applies Block modifiers once at the source-named checkpoint. | crates/spire-core/src/items/tests/m25_watcher.rs::m25_items_new_relics_through_real_combat |
| F8-a | Lesson Learned RNG and combat copy | Source: misc RNG selects an eligible permanent-deck instance; that permanent deck is the target (research :516-523; data/cards/implementation-notes.json:67-68). Stream mapping is open in design §5.9 F8. | Project decision: use RunCombatCardRandom; journal the permanent-instance upgrade for settlement and leave the combat copy unchanged. | The shared native card-random stream owns random card choices; the journal updates the source-defined permanent target once. | crates/spire-core/src/combat/tests/m25_watcher_state.rs::m25_state_lesson_learned_requests_permanent_upgrade |
| F11-a | Talk to the Hand on Invincible-capped hits | Open part: design §5.9 F11 asks which later hits return Block, including Invincible-capped hits. | Project decision: normal Attack damage callbacks qualify per hit even if Invincible caps HP loss; THORNS/HP_LOSS do not. | Follows Attack-hit callbacks and excludes unrelated HP loss. | crates/spire-core/src/combat/tests/m25_watcher_powers.rs::m25_powers_constructor_stack_and_callbacks; crates/spire-core/src/combat/tests/m25_watcher_interactions.rs::m25_t04_painful_stabs_and_block_return |
| F13-a | Start-turn recharge and delayed powers | Open part: design §5.9 F13 leaves recharge, stance exit, delayed Wrath/draw, and stance-triggered hooks unsettled. | Project decision: set recharge to max energy minus Energy Down; resolve Deva before Simmering Fury stance/draw; Calm/Violet Lotus/Rushdown/Mental Fortress use shared owners. | Keeps Energy Down distinct from Deva growth and preserves queued stance reactions. | crates/spire-core/src/combat/tests/m25_watcher_powers.rs::m25_powers_constructor_stack_and_callbacks |
| F16-a | Fear No Evil intent mapping in tests | Source: four attack-intent categories (research :105; snapshot data/cards/sources/wiki-pages.json:855). Native FullIntentId has no category (crates/spire-core/src/combat/model.rs:45-51). | Project decision: classify the current intent as one of the four exactly when its native intent has a damage entry with base ≥0; queue the hit before Calm. | Reuses the intent-damage table read by Spot Weakness/Go for the Eyes rather than inferring from display names. | crates/spire-core/src/combat/tests/m25_watcher_direct.rs::m25_direct_base_plus_values_and_flags |
| F16-b | Fear No Evil intent mapping in the owner | Open part: FullIntentId exposes no category; display tokens are not the game enum. | Project decision: implement the same damage-entry predicate in m25_direct::intends_attack, aligned with F16-a’s test binding. | One predicate aligns card behavior with native intent data. | crates/spire-core/src/combat/cards/watcher/m25_direct.rs::intends_attack; pin: crates/spire-core/src/combat/tests/m25_watcher_direct.rs::m25_direct_base_plus_values_and_flags |
| W-RNG | Tantrum returned draw-pile insertion | Open part: source research leaves insertion endpoint/joint RNG history open (research :581). | Project decision: return the same Tantrum instance through shared ShuffledDraw insertion using card-random RNG. | Reuses shared insertion rather than inventing a card-specific stream or shuffle. | crates/spire-core/src/combat/tests/m25_watcher_state.rs::m25_state_signature_wallop_tantrum |
| F16-c | Scrawl at empty piles | Open part: design §5.9 F16 leaves empty-pile and mid-loop generated-card stop conditions open. | Project decision: with both piles empty, stop at the shared draw owner’s available-card boundary; an actually drawn Deus Ex Machina resolves its callback during Scrawl. | The draw owner determines whether another card exists and processes draw-triggered callbacks. | crates/spire-core/src/combat/tests/m25_watcher_state.rs::m25_state_zero_x_and_empty_pile_edges; crates/spire-core/src/combat/tests/m25_watcher_interactions.rs::m25_t01_beat_of_death_counts_watcher_plays |
| MR-I1 | Master Reality on created cards | Source: "Whenever a card is created during combat, Upgrade it" (data/cards/sources/wiki-pages.json:1257; generator examples at :57, :447, :813, :1467, :1551, :1887). The hand routes' checkpoints come from the M21 source rulings (constructor check, then hand-helper check; the overflow helper upgrades only a display copy) and the play-origins research (Foreign Influence has no action-level check). | Source-settled (final review I-1): the shared creation owner upgrades every non-Curse, non-Status created card once, for every character and destination, hand overflow included. Watcher fixed children keep the constructor and hand checkpoints, and Foreign Influence keeps only its hand checkpoint, so its overflow copy stays base. | One shared owner carries the source-wide rule; the two route-checked hand routes follow their own cited source checkpoints. | crates/spire-core/src/combat/tests/m25_watcher_interactions.rs::m25_t12_generation_and_master_reality_at_act4; crates/spire-core/src/combat/tests/m25_watcher_interactions.rs::m25_master_reality_upgrades_creations_in_every_destination; crates/spire-core/src/combat/tests/m25_watcher_generation.rs::m25_generation_full_hand_overflow |

## Not established

Native Rust results only. No Python delivery, original-runtime parity, normal-start Act 4
entry or Heart Victory is claimed.
