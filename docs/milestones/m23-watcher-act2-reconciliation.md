# M23 source and API reconciliation

This record accompanies the immutable [approved design](../superpowers/specs/2026-10-28-m23-watcher-act2-design.md) and [plan](../superpowers/plans/2026-10-28-m23-watcher-act2.md). Their historical "pending/unimplemented M21/M22" prose is preserved unchanged. It is superseded by accepted M21 ([PR60](https://github.com/klottick/spire-gym/pull/60)) and M22 ([PR61](https://github.com/klottick/spire-gym/pull/61), merge `02839be64c2fb94195aa2045aee8062656199894`, 933-file manifest `880c538faab667e9bf10816beddc327d48ffa4e44c248132f73c03245f82049c`) and by root's M23 execution release. This record states the actual accepted owners that the proposed names map to. It is not an acceptance record.

## Accepted source binding

Before production edits, all 933 tracked files matched the accepted M22 manifest by SHA-256 and size, with only the two approved M23 documents untracked. The four inherited M21 source gates are consumed at their accepted grades through named regression owners: Scry collapse/caller (`combat/tests/m21_watcher_scry.rs`, `m21_watcher_review.rs`), later recharge (`m21_watcher_creation.rs`), Ethereal-retain-Establishment (`m21_watcher_retain.rs`) and deferred MasterReality (`m21_watcher_creation.rs`, `m21_watcher_review.rs`, `m21_watcher_stances.rs`). Scry remains a finite gameplay projection, not raw queue/callback/frame/cap equality. No archive extraction or game execution was added; the bounded AncientWriting facts retained by M23 planning qualify the purple Simplicity extension.

## Proposed names and actual owners

| Proposed interface | Actual accepted owner |
| --- | --- |
| `RoomScenario::M23WatcherAct2` on `Run::new_room_scenario` | `run/rooms/mod.rs`; Watcher A0 short Neow only, otherwise `Unsupported("M23 requires Watcher A0 short Neow")` before mutation |
| Captured Act1/Act2/full-Act2 predicates | `uses_captured_act2` (and therefore `uses_captured_act1`) plus one `uses_act2_boss` predicate replacing five identical boss-continuation expressions in `act2.rs`, `boss.rs` and `transition.rs`; M23 is not in playable Act3/Act4 or Silent/Defect predicates |
| Shared Watcher producer dispatch | `uses_watcher_content` (M22 or M23) at the existing M22 producer sites: `mutations.rs`, `rooms/rewards.rs`, `rooms/shop.rs`, `rooms/boss.rs`, `run/combat.rs` (the `watcher_progression` item owner) and `rooms/events.rs` |
| `M23_WATCHER_ACT2_{CARDS,RELICS,POTIONS}` | `run/rooms/m23_content.rs` value views of the `m22_content` slices; no producer row is copied |
| `capture_relic_in_scope` / `capture_relic` | `rooms/rewards.rs::capture_relic_in_scope` is the single implementation; `capture_m22_relic` remains the exact M22-scope adapter |
| `RunRngScope::M23RelicCapture` | `run/rng.rs`, appended last, spelling `run.m23.relic.capture`; Act1 keeps `run.m22.relic.capture` |
| First/later merchant per act | `rooms/shop.rs` reads the destination act's pre-increment `Act2Progress.shop_visits` for M23 Act2 |
| Purple AncientSimplicity | `rooms/events.rs` adds StrikePurple/DefendPurple under Watcher content through the shared upgrade owner |
| Act3Ready | Existing `transition::begin/drive` (2,3,34) frame and `RoomViewKind::Act3Ready` zero-candidate publication |
| Caller/rollout | `examples/m13_support/mod.rs` gains `CallerProfile::M23`, `M23CallerOutcome` and `rollout_m23` in the same `rollout_shared` loop; `examples/m23_support/policy.rs` delegates to the frozen M22 policy with per-act route witnesses and AncientSimplicity |
| Supplied diagnostics | Default-off `m23-diagnostics` feature (`run/diagnostics/m23.rs`) for the conditional fixed-16 Act2 bosses and the six plan Task3 fixtures; the read-only Watcher checkpoint now admits M22 or M23 |

## Matched and unmatched future history

The accepted core constructor accepts an explicitly supplied `CreationProfile` future history for both M22 and M23. Under the same matched history, seed partition and ordinal, the full Run (including RNG, captures, records, successor map and transition flags) is equal through Act1 and the `(1,2,17)` transition for all nine boss pairs. The only differences are the scenario tag, M22's zero-action `Act2Ready` screen versus M23's `Map`, and M23's once-initialized `Act2Progress`. Native M22 keeps the all-seen Act2 history and draws Act2 boss-order shuffle tickets. M23's targeted unseen boss does not draw them, so Act2 RNG and maps then differ. That difference is disclosed, not normalized.

## Evidence grades

Owner-level tests use supplied map positions, visit counters and Lament HP to exercise shared owners. They are not canonical traversal. Per the primary scope clarification (2026-10-30; ability to play, not strategy), supplied victories are valid gameplay validation when labelled as supplied, and canonical-policy results are reported as policy outcomes, not gaps. The supplied Act2 starts construct a completed `(1,2,17)` predecessor frame so the actual `(2,3,34)` owner can run. That frame is constructed provenance, not an earned Act1.

## Confused draw combat price (user ruling M23-D1 v2)

An integrated Task3 composition test showed a real shared-owner gap. A card drawn under Confused and then priced again without a fresh draw reverted to its constructor price. Source `ConfusionPower.onCardDraw` sets the drawn card's combat cost (`card.cost`) as well as its turn price. The engine's draw writer, however, updated `mutable_combat_cost` only for ForceField, and the combat price is `mutable_combat_cost.or(randomized_cost)`. On the user's decision, the fix is global (`combat/piles.rs`): whenever the drawn instance already has a combat-cost owner, the writer stores the Confused price there too, matching the existing Snecko Oil writer. BloodForBlood and ForceField keep their specific branches. This changes behavior for every character whose cards carry a preset `mutable_combat_cost`: Defect (M16–M20), Watcher (M21–M23), and any card whose combat cost was previously modified.

Two accepted M18 assertions in `combat/tests/m18_defect_interactions.rs::play_confused` pinned the old behavior: before and after play they required `mutable_combat_cost == base`. Both now assert the source law: the instance's combat cost equals its Confused price. A new check also confirms that the card's printed base cost is unchanged. Nothing else in M18 changed. The original file bytes, the exact diff, the failing pre-amendment run, and a rejected narrower Watcher-only variant are retained with the correction evidence. A new Defect regression (`combat/tests/m23_confused_cost.rs`) fails under the narrower variant and passes under the adopted fix.

Retained accepted outcomes from before this correction are history and are not rerun.

Impact sweep, read-only. A Confused draw of a card with a preset combat cost requires both a Snecko encounter and such a card. That combination is reachable in M15, M18, M19, M20 and M21, and in M23. It is not reachable in M9–M14, M16, M17 or M22. Snecko Oil is admitted only in M15 and M20. All retained M15–M22 outcome receipts were searched. Only the three M18 original normal cases (seed 7 Guardian→Champ, seed 11 Hexaghost→Collector, seed 11 SlimeBoss→Champ) contain Confused, together with the repository M18/M19 fixtures that share their prefixes. In each of them no Confused-drawn instance was priced again in hand without a fresh draw; the only visible effect is the discard and played-power prices described below. Retained M20, M21 and M22 outcome receipts contain no Confused or Snecko step.

### Regenerated M18/M19 expected transcripts

Under the corrected law, the frozen M18/M19 expected transcripts showed different public prices for Confused-drawn cards that had left the hand (discard pile or played powers) after turn expiry. Six repository fixtures were regenerated: `crates/spire-core/tests/fixtures/m18/{7-guardian-champ,11-hexaghost-collector,11-slime_boss-champ}.json` and `fixtures/m19/{7-guardian-champ-awakened_one,11-hexaghost-collector-awakened_one,11-slime_boss-champ-time_eater}.json`.

Regeneration replayed the SAME recorded candidate indices on the corrected source, with no chooser call. The regeneration tool and its report are retained. It hard-stops on any illegal recorded action and on any difference in map, transition, reward, fault or terminal. Decisions must be identical after masking only `effective_cost`, and only while the player is Confused. The six unaffected fixtures re-serialized byte-identically. Each affected fixture differs at one or three steps of a single Snecko combat, only in those public prices, and the combat ends one to three steps later. The original bytes and hashes are retained with the correction evidence, as are the original M18 outcome receipts. Those receipts remain pre-correction history and are untouched. All twelve fixtures replay byte-identically against the final cost law below.

## Confused and Snecko Oil cost-law checklist (PM addendum to M23-D1)

The user supplied these rules, checked against the retained offline wiki captures (`data/cards/sources/wiki-pages.json`, `data/powers/sources/wiki-pages.json`). They are pinned by `combat/tests/m23_cost_law.rs` on the shared owners:

| Rule (capture title, revision) | Admitted where reachable | Result |
| --- | --- | --- |
| Corruption keeps Skills at 0 under Confused (Corruption, 56614) | M10 (Snecko Oil is not co-admitted with Corruption anywhere) | already correct; regression |
| Force Field roll replaces cost, power discount continues; Snecko Oil too | M20 | correct after the global fix; regression |
| Blood for Blood roll replaces cost, HP-loss reduction continues, upgrade lowers a positive cost by 1 (Blood for Blood, 56417) | M10 | already correct; regression |
| Masterful Stab roll overrides, increases continue (Masterful Stab, 57491) | M15 | correct after the global fix; regression |
| Enlightenment+/Madness/generated (Chrysalis, Metamorphosis) costs overridden by Confused and Snecko Oil (66701, 66711, 66754, 66758) | M15, M20 | correct after the global fix; regression (prior states are their owners' exact writes) |
| Eviscerate: roll is the combat cost; an equal roll keeps the turn discount (Eviscerate, 57257; source onCardDraw) | M15 | **RED, fixed**: the Confused writer now changes cost and turn price only when the roll differs from the current cost |
| Bullet Time zeroes randomized hand for the turn only (Bullet Time, 56919) | M15 | already correct; regression |
| Upgrade of a randomized card takes the upgraded cost unless randomized to 0 (Armaments, 56323) | M10 (Armaments), M15/M20 (Apotheosis, Blessing of the Forge) | **RED, fixed**: payload-priced (Ironclad) cards lacked the source `upgradeBaseCost` rebase; the in-combat upgrade sites now share `card_state::upgrade_live_card` |
| Streamline reduction continues from the roll | M18–M20 | correct after the global fix; regression |
| X-cost / unplayable cards are not randomized | all | already guarded (accepted M10 test) |
| Confused draw and Snecko Oil clear `freeToPlayOnce` (Forethought, 66704 update history: "Fixed Confusion card not affecting cards reduced by Setup or Forethought"; source `ConfusionPower.onCardDraw`, `RandomizeHandCostAction`) | M15 (Setup, Forethought) and M20 (generated Forethought), with Snecko and Snecko Oil in both. No retained outcome step or repository fixture contains such a draw or Oil use | **RED, fixed** (final review I-1). The engine carries source `freeToPlayOnce` as `free_play` and as Setup/Forethought's `free_to_play_once`. The Confused writer cleared only the first and Snecko Oil cleared neither; both writers now clear both. Autoplay children are played from the draw pile through limbo and never pass the draw callback |

Not reachable in any accepted profile (no speculative code): Snecko Oil with Corruption; Lesson Learned; Snecko Eye as a relic.

## Native integration notes

The native M23 profile (`run_schema_m23.rs` and the M23 arms in `lib.rs`, `run_session.rs`, `run_replay.rs`, `run_batch.rs` and `run_observation.rs`) was delivered by a separate helper lane against a published core snapshot and integrated unchanged. Adding `RunBossHistory::M23WatcherAct2` makes two frozen exhaustive `match` statements non-exhaustive. Following the M20 precedent, `crates/spire-env/tests/run_m10_session.rs` and `run_m10_batch.rs` add the variant to their existing unreachable arms. This is a mechanical change; no assertion changed. The env test dependency enables the default-off `m23-diagnostics` feature next to the existing M20/M22 diagnostic features.

## Helper-lane provenance

Two isolated helper lanes worked on M23 under the PM, each against an immutable published snapshot of this checkout. The senior integrated their deliveries sequentially with recorded pre/post hashes:

- Task3 combat compositions (`crates/spire-core/src/combat/tests/m23_watcher_act2.rs` and one `mod` line) were applied unchanged. Five tests were green regressions. The sixth was a retained RED that exposed the Confused draw-price defect above; it was reproduced on the integrated tree before the shared-owner fix.
- The native schema/session/replay/batch profile (Tasks5–7) was integrated unchanged, plus the two approved exhaustive-arm edits. A follow-up lane adds core-fault, delivery-fault, worker-panic, allocation and real-Act3Ready native coverage on the default-off `m23-diagnostics` stages (`run/diagnostics/m23.rs`). Those stages are constructed owner checkpoints with a constructed `(1,2,17)` predecessor and boss record, never played receipts.

The senior kept every core Run, combat, RNG, content, caller, diagnostic and freeze seam.

## Caller and fixed diagnostics

The canonical caller extends the shared rollout owner with `CallerProfile::M23`, `M23CallerOutcome` and `rollout_m23`. It stops at healthy Act3Ready (3/34, zero actions) or a terminal. The pure policy delegates to the frozen M22 policy with per-act route witnesses and AncientSimplicity. Inputs pin the six seed/boss pairs, the seed-7 Guardian→Champ nonattacking control, budgets 0..=32768 and rejection before Run construction. The default-off series runner (`examples/m23_supplied_diagnostics.rs`) uses the same rollout for the six plan Task3 played fixtures (budget 128, fixed public scripts in `examples/m23_support/fixtures.rs`) and the conditional fixed-16 supplied Act2 bosses. Each starts one map edge before a real captured Act2 room, and its single bootstrap Move is excluded from the budget.

## Native follow-up deliveries

Two further native deliveries were integrated unchanged, each with recorded pre/post hashes matching the delivered hashes. The first adds tests on the `m23-diagnostics` stages: actual Act3Ready and Act2 boss choices for every branch, actual Scry/Meditate/Stance replay, actual core-fault and delivery-fault records, batch post-dispatch core faults with indexed `delivery_lost`, and a worker panic that keeps ownership. The second adds test-only allocation/reservation-failure and reset-path panic hooks (`#[cfg(test)]` in `error.rs` and `run_session.rs`) and two M23 batch tests. Neither delivery changes production code.

## Scope reconciliation (2026-10-30)

Under the primary scope clarification (receipt `6a471ebf82e034b43dddc3352bb0d9d2839a391d4ca5aa9a1c265952e42e0498`) and the PM disposition (`35d2fdb7acc3393ed3183af3dd58086928d7e02b28141ccd622850ab558d908c`), two strategy-only outcome gates now carry `#[ignore = "strategy outcome gate …; run with --ignored"]`. They are `m18_from_start_fixed_pairs_require_actual_boss_entry_and_completion` (`crates/spire-core/tests/m18_defect_act2.rs`) and `m19_six_fixed_triples_require_actual_boss_records_and_normal_no_keys` (`crates/spire-core/tests/m19_defect_act3.rs`). Their assertion bytes are unchanged. By default both are reported ignored. Under `--ignored` both still fail with their original messages: "M18 normal gate unmet: entered/matching Act2 boss records [Collector, Champ]" and "M19 normal gate unmet: actual Act3 entries/matching records []". The original files, the additions-only diff and both runs are retained with the scope evidence. The M18–M22 milestone profiles and the work-status page each gained a short scope-reconciliation section. Their historical result text is unchanged.

## Post-freeze reporting-only correction

After the once-only sequence, the supplied-diagnostics runner (`crates/spire-core/examples/m23_supplied_diagnostics.rs`) was corrected not to treat a fixture's designed "fixture combat settled" signal after a played Victory as an error exit. The mapping is `examples/m23_support/fixtures.rs::settled_victory`, pinned by `m23_fixture_settled_victory_is_success_not_error` (RED then GREEN, with both binaries retained). No gameplay, core or native code changed. The frozen executables, receipts and outcomes stay as recorded, so the two Victory receipts carry the historical exit 1. The M22 and M20 supplied runners do not use such a signal.
