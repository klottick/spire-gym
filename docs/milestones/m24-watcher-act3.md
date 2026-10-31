# M24 finite native Watcher A0 Acts 1–3 — candidate

Status: **source candidate; frozen once-only sequence complete; independent final review pending. Not accepted.**
This profile implements the approved [design](../superpowers/specs/2026-10-30-m24-watcher-act3-design.md)
and [plan](../superpowers/plans/2026-10-30-m24-watcher-act3.md). Owners, deviations and
corrections are in the [reconciliation](m24-watcher-act3-reconciliation.md). The predecessor
is M23 ([PR62](https://github.com/klottick/spire-gym/pull/62)).

## Finite boundary

Canonical Watcher A0 starts at short Neow with 72/72 HP, 99 gold, the ten starter instances,
PureWater and three empty potion slots. The profile replays the exact M23 Acts 1–2 prefix and
continues from the `(2,3,34)` transition through the shared captured Act3 (floors 35–49,
chest 43, rest 49, boss 50). Possible endpoints:

- After an Act3 boss Victory without all three keys: `RunTerminal::Act3NoKeys`.
- After an Act3 boss Victory with all three keys: the healthy nonterminal `Act4Ready` cutoff
  (act 3, floor 50, zero legal actions). A further step returns
  `UnsupportedBoundary("M24 stops before the Act4 door")`.
- A genuine Death or EmptyDeck at any point.

Act4, the Act4 door, the Heart, A1–20, full Neow, other Watcher content, standard pools,
Python delivery and original-runtime validation are outside this profile.

| Union | Members |
| --- | --- |
| Permanent cards (29) | The M22/M23 28 plus Parasite (Writhing Mass Implant only; never offered) |
| Generated-only (3) / statuses (5) | Miracle, Smite, Safety / Dazed, Burn, Slimed, Wound, Void |
| Relics (19) / potions (7) | The M22/M23 18 plus Red Mask (Tomb only) / the M22/M23 7 |
| Playable encounters / actors | 40 setups (24 M23 + 16 Act3) / 54 enemy identities |

Card rewards keep the full global visit (mod 6 rows, mod 7 potions). Act3 merchants choose
first or later stock from the Act3 visit counter. Act3 elites and chests capture relics under
`run.m24.relic.capture` with key `[act, floor, node_id, full_visit, source_tag]`; an empty
pool offers nothing and makes no request. Act3 questions are Falling, then Tomb of Lord Red
Mask, then captured Monster rooms.

## Shared-owner changes in this candidate

- **Watcher Curse construction law (M21–M24).** A Curse permanent such as Parasite enters
  Watcher combat with its shared unplayable, costless fields.
- **M24 Watcher inventory context.** `with_m24_profile()` admits Red Mask; the M22/M23
  contexts are unchanged.
- **Global Time Warp barrier correction (M24-D1, all characters).** Ordinary work queued
  after the twelfth card resolves before Time Eater's early end.

## Evidence (grades kept separate)

The freeze (`e072c4e8…`) and the once-only invocation log are described in the
[reconciliation](m24-watcher-act3-reconciliation.md). Normal-start results are policy
outcomes, not gates (2026-10-30 scope clarification).

| Grade | Result |
| --- | --- |
| Normal played | Six canonical cases: all Death (Act1 floors 7–16, one at Act2 floor 23), identical step for step to the M23 receipts. No normal Act3 entry. Controls behaved as designed. |
| Supplied played | Time Eater (seed 7, no keys): Victory, then the real Act3 ending to `Act3NoKeys`. The other five bosses: Death. Traversal: Act2 boss treasure, `(2,3,34)`, Act3 rooms, then Death at floor 39 (×3). Nine interaction fixtures: all Death. |
| Constructed | `Act4Ready` across key masks, ending faults and rejections, room and choice stages. `Act4Ready` has no played witness. |
| Native | Canonical receipts matched in scalar, strict JSON, portable replay and batches with workers 1 and 2, plus 4-lane coexistence (32 actions each, with combat). The 18 supplied and fixture receipts matched in a test built from the frozen source. |
| Combat compositions | W1–W14, W2b and W7b on real setups. The W3 RED led to the global Time Warp fix. |

Not established: Python delivery, original-runtime or Java parity, normal Act3 completion and a
played `Act4Ready`.
