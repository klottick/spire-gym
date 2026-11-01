# M25 finite native Watcher A0 Acts 1–4 and Heart — candidate

Status: **source candidate; frozen once-only sequences complete (freeze v1, the F1
corrective freeze v2 and the G1 corrective freeze v3); independent final review pending. Not accepted.**
This profile implements the approved [design](../superpowers/specs/2026-10-30-m25-watcher-heart-design.md)
and [plan](../superpowers/plans/2026-10-30-m25-watcher-heart.md). Owners, deviations, global
corrections and lane provenance are in the [reconciliation](m25-watcher-heart-reconciliation.md);
per-item test keys are in the [coverage ledger](m25-watcher-heart-coverage-ledger.md). The
predecessor is M24 ([PR64](https://github.com/klottick/spire-gym/pull/64)).

## Finite boundary

Canonical Watcher A0 starts at short Neow with 72/72 HP, 99 gold, the ten starter instances,
PureWater and three empty potion slots. The profile replays the exact M24 Acts 1–3 run
(`RoomScenario::M25WatcherHeart`). M24's `Act4Ready` cutoff is removed. After an Act3 boss
Victory with all three keys, the fourth ending dialog goes through the shared Act4 door, and
the fixed route follows: Rest 51, Shop 52, Shield and Spear 53, Corrupt Heart 54. After a Heart
Victory the player accepts the offered Proceed, which enters floor 55 (`TrueVictoryRoom`). A
Run ends in exactly one of these ways:

- `HeartVictory` after the final Proceed (floor 55);
- `Act3NoKeys` after an Act3 boss Victory without all three keys (unchanged from M24);
- a genuine Death or EmptyDeck at any point.

Keys are earned only through the shared owners: Recall at a campfire (Ruby), the Sapphire
choice at a non-boss chest, and a burning-elite Victory (Emerald).

| Union | Members |
| --- | --- |
| Permanent admission (76) | All 75 ordinary Watcher cards (28 accepted in M21–M24, 47 new) plus Parasite |
| Card vocabulary (171) | 76 permanent + 8 Watcher generated children (Miracle, Smite, Safety, Insight, ThroughViolence, Beta, Omega, Expunger) + 82 non-Watcher Foreign Influence outputs + 5 statuses |
| Choice-only rows (3) | Wish: BecomeAlmighty, FameAndFortune, LiveForever |
| Foreign Influence support (106) | Every ordinary, non-healing Attack of any colour |
| Relics (23) / potions (7) | M24's 19 + Damaru, Duality, CloakClasp, Melange / M22's 7 (unchanged) |
| Encounters / enemy identities | 42 setups (M24's 40 + ShieldAndSpear, CorruptHeart) / 57 |
| Native vocabulary | 171 cards / 3 choice-only rows / 23 relics / 7 potions; M24 descriptors and fingerprints unchanged |

A1–20, full Neow, the standard reward, shop, event and potion pools, Watcher typed generation
pools, transforms, off-colour permanent acquisition, Python delivery and original-runtime or
Java parity are outside this profile (design §13). Finite relic stock can be exhausted, and
that is reported.

## Shared-owner changes in this candidate

Each change is a global correction at source law for all characters, disclosed in the
[reconciliation](m25-watcher-heart-reconciliation.md):

- **Perfected Strike and the card-tag owner.** STRIKE and HEALING come from one card-tag owner.
  This amends a wrong accepted M10 assertion; retained outcomes were swept and none changed.
- **Autoplay census and repeat ownership.** Omniscience continuations are counted in the
  autoplay census, and a queued repeat is owned while its source is in play or suspended.
- **Several same-source autoplay continuations (Content RQ-OMNI).** This also fixed Burst + Havoc,
  Duplication + Havoc and Double Tap + Duplication.
- **Watcher recharge and Energized.**
- **Public previous card type.** It is filled only in M25 Watcher combat, and older profiles'
  views are unchanged.
- **F1: a power-owned selection during a card play.** Found by the once-only coverage-B case:
  Melange's shuffle Scry during InnerPeace.
- **Native: M25 strict replay of card-offer actions** (`run_replay.rs`). Found by N3; M25 only.
- **G1: generated Watcher cards keep their M25 setup** (`card_state::generated`). Found by M26; for example, Stasis returning
  Talk to the Hand kept Exhaust only after the fix. Post-freeze; covered by the no-drift replay.
- **G2: Deus Ex Machina in hand without a draw is unplayable** (`cards/mod.rs`), not `Unsupported`. Found by M26.
- **I-1: Master Reality creation law** (`piles::insert_generated`). Every non-Curse, non-Status card created during combat is
  upgraded in every destination: hand, hand overflow, draw pile and discard. Watcher fixed children keep their action-construction
  and hand checkpoints, and Foreign Influence its hand checkpoint, as the source does. Found by final review v1.
- **R4: Burn discards itself at end of turn even when retained** (`piles.rs`, `DiscardHand`). This applies to every character,
  so a Silent's Well-Laid Plans can no longer keep a Burn. Only the retention is withdrawn; the hand still discards in its
  ordinary order.
- **The pending card queue is cancelled on EndPlayerTurn** (`turns.rs`) for every character, as the source's
  `ClearCardQueueAction` does. In practice only Vault reaches it.
- **Collect counts Chemical X** (+2, like every other X-cost card). This cannot be reached in M25, because Chemical X is outside
  the relic union.

## Evidence (grades kept separate)

The three freeze records, the once-only invocation logs, the no-drift replay and the
comparisons are described in the [reconciliation](m25-watcher-heart-reconciliation.md).
Normal-start results are policy outcomes, not gates (2026-10-30 scope clarification).

| Grade | Result |
| --- | --- |
| Normal played | Six canonical cases: all Death in Acts 1–2 (Act1 floors 7–16, one at Act2 floor 23), identical step for step to the M24 comparison control on the same tree and to M24's retained receipts, with the disclosed `previous_card_type` mask. No normal Act3 entry. Controls (nonattacking death, budgets 0 and 1, four input errors) behaved as designed. |
| Supplied played | Traversal from a constructed Act2 boss source with strong loadout v1. TimeEater and DonuAndDeca: every key earned in play, then the door, Act4, Shield and Spear, the Heart and `HeartVictory` (362 and 380 actions). AwakenedOne: Death at the Act3 boss. Act4 from a constructed Act3 Victory source with supplied keys: coverage A, B and C Death at Shield and Spear. B is the F1 corrective (freeze v2). C is the G1 corrective (freeze v3); v1 C after step 25 is pre-correction history. The strong v1 case, run under the frozen rule, reached `HeartVictory` (55 actions). Eight interaction fixtures: all Death; their checkpoints are in the coverage ledger. |
| Constructed | Every card, child, Wish option, Foreign Influence output, relic, potion and event through the shared owners (ledger keys G, D, S, P, F, C, I, A, R). Door and Heart across all eight key masks, faults, room stages and the new selections. |
| Native | 171/3/23/7 vocabulary, state rows, privacy, capacity, strict history, replay, batch and coexistence (`run_m25_*`). Recorded-action agreement (N3, post-outcome, test-only): all 24 retained transcripts (canonical, controls, traversal, Act4, strong, fixtures) agree in single sessions, strict JSON or same-initializer replay, and batches with workers 1 and 2. N3 found and fixed one native defect: M25 strict replay of card-offer actions. |

Not established: Python delivery, original-runtime or Java parity, a normal-start Act3 entry,
key set, Act4 entry or Heart Victory, and any strategy quality.
