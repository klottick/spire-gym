# Intentional deviations from the base game

These are explicit project choices, not simulator bugs or claims of exact base-game
equivalence. Preserve this distinction in designs, tests and evaluation reports.

## RNG-001 — Decorrelated, reproducible project randomness

**Decision date:** 2026-10-16

**Status:** User-approved policy; the Python randomness foundation and narrow
headless combat reference are implemented/reviewed, and the bounded Rust combat/
environment slice implements project-owned randomness with reviewed parity tests.
Original-runtime combat validation remains incomplete. See
[repository scope](scope.md) for the Rust/learning goals this policy serves.

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

The user explicitly reaffirmed conditional enemy-intention chance fidelity on
2026-10-16. Match each supported next-move law given relevant gameplay state and
selected-move history, not just overall move frequencies. Where finite sampler
semantics determine the branch mass, preserve that mass with exact rational
sampling rather than rounding the displayed decimal or choosing cleaner weights.
This does not require the original PRNG calculation or restore incidental seed/
stream coupling. The combat design records the exact supported chance tables.

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
and exhaustion tests. Its standalone milestone recorded 395 passing Python tests.
The subsequent [combat integration](superpowers/specs/2026-10-16-python-combat-behavior-design.md)
implemented typed stream ownership, public-information boundaries and schema-2
snapshots, with 735 tests passing in its final implementation verification. The
opaque schema-1 RNG slot is rejected rather than retained. These historical test
results establish authored behavior/replay, not original-runtime validation or
learned playing strength; independently authored Rust parity remains future work.

## TIME-001 — Secret Portal's elapsed-time requirement is assumed satisfied

**Decision date:** 2026-10-22. **Status:** user-approved full-run gym policy;
implementation remains subject to the full-run plan/start gate.

Treat Secret Portal's original 800-second run-playtime requirement as always
satisfied. The selected gym models ordinary play, not competitive speedrunning;
this narrow eligibility simplification is intentional, not elapsed-time parity.

Keep Act 3 and all other ordinary eligibility, event-pool/history and random
selection rules. Eligible does not mean guaranteed: do not force the event,
restore it after consumption, or expose its future selection. Accepting still
routes to the actual boss fight; declining remains available. Keys, A20's second
Act 3 boss, rewards and terminal rules are unchanged.

No run clock, frame simulation, wait action, elapsed-time feature or host-speed
dependency is required for this policy. Focused event tests must cover the assumed
time condition and unchanged non-time eligibility; existing replay/batch checks
protect independence from caller delay and worker scheduling. Retain historical
source/time findings unchanged; they describe the original rule, not this gym
choice. No product code or original-runtime-validation claim follows from recording
the decision.

**M28 implementation (2026-11-02):** the standard Run's event eligibility treats the playtime gate
as met. Secret Portal is eligible in Act 3 by the one-time rule (`event_pool::eligible`, M28c); its
body landed in M28f.

## UNLOCK-001 — Note For Yourself treats the highest unlocked Ascension as 20

**Decision date:** 2026-11-02 (PM ruling, M28c). **Status:** disclosed M28 deviation.

The original getShrine admits Note For Yourself outside Daily runs, below A15, and only when the Run
is at A0 or below the player's highest unlocked Ascension. The simulator has no unlock profile, so it
treats the highest unlocked Ascension as 20. Note For Yourself is therefore eligible at A0–A14. The
standard Run admits A0–A5 from M29d, A0–A10 from M30 and A0–A15 from M31 (A16+ lands in M32–M33), so an A15 Run never meets Note For Yourself. No other eligibility or history rule changes.

## SOFTLOCK-001 — An unguarded empty event grid completes without a selection

**Decision date:** 2026-11-02 (PM ruling, M28c review M4; extended in M28d, M28e, M28f). **Status:** disclosed
M28 deviation.

In the original, several event options open a card grid with no check that it has a card. With no
purgeable card (every card is an unremovable curse or bottled), the grid is empty and strands the
player. The simulator cannot model that soft-lock. In each case the option stays selectable, selects
nothing and the event continues, as Ancient Writing's Elegance does with an empty group in the
original:

- **Purifier** (M28c): removes nothing and the event completes.
- **Transmogrifier** (M28d; `Transmogrifier`): transforms nothing and the event completes.
- **Wing Statue** (M28d; `GoldenWing`): Pray still deals its 7 damage, then removes nothing and
  shows the result. The source opens the removal grid after the damage with no local check.
- **Beggar** (M28e; `Beggar`): Pay still takes the 75 gold, then removes nothing and the event
  completes. The source opens the removal grid after the payment with no local check.
- **The Designer** (M28f; `Designer`, PM ruling 2026-11-03): Clean Up and Full Service count every
  unbottled card, but their grids exclude Ascender's Bane, Curse of the Bell and Necronomicurse, so a
  grid can open short of its one or two cards. A grid completes only at its full count and cannot
  be cancelled, so the source strands. The gold is paid, nothing is removed or transformed, and the
  event completes; Full Service's follow-up upgrade, which runs only after the grid, does not happen.

Events that check for a card first are unaffected. The Upgrade Shrine, Living Wall's Grow and
Accursed Blacksmith's Forge disable their upgrade option when no card can be upgraded. Living Wall,
The Cleric, Bonfire Spirits and Wheel of Change open their grids only when a purgeable card exists;
Drug Dealer disables its Transform with fewer than two.

## EVENT-001 — Forced one-button event pages are collapsed

**Decision date:** 2026-11-02 (PM-accepted M28 D1 checks, M28d–M28f). **Status:** disclosed M28
deviation.

The original shows some event pages (an intro, a Continue or a goodbye) whose only button has one
fixed effect. The simulator collapses such a page into the preceding choice only when no other legal
action taken on that page could lead to a different final state or RNG order. Out-of-combat potion
use counts here, because the Run offers it on event views. Otherwise the page stays a step. The
collapsed pages are listed per chunk in the [M28 profile](milestones/m28-events-economy.md) (Forced
pages, D1) and the [M28 reconciliation](milestones/m28-events-economy-reconciliation.md). Wheel of
Change's reveal and Cursed Tome's HP-loss pages stay steps.

## EVENT-005 — A shop purge with nothing purgeable is not offered

**Decision date:** 2026-11-03 (PM ruling on M30 plan review I2, under the D1/EVENT-001 rule).
**Status:** disclosed M30 deviation.

In the original, the shop's card-removal button stays clickable whatever the deck holds.
`ShopScreen.purchasePurge` (`ShopScreen.java:256,1007–1015`) opens a grid of the unbottled purgeable
cards. If every card is Ascender's Bane, Curse of the Bell, Necronomicurse or bottled, the grid is
empty and its only legal action is Cancel. Nothing is charged and no RNG is used.

The simulator marks the purge row `Disabled` while no unbottled purgeable card exists, so it is not
advertised. The status is derived each time the shop is viewed, so buying a card re-enables the same
row. Row ids and order do not change. No reachable Run state or RNG order differs from the original,
because the only reachable outcome of that step is the unchanged shop. Potion use stays available on
the shop view either way. This follows the existing price rule, where the simulator already disables
a purge row the original leaves clickable as a no-op when gold is below the price
(`ShopScreen.java:1008–1011`). Test: `m30_shop_purge_disabled_without_purgeable`, with
`m30_shop_purge_reenabled_after_buy` and `m30_shop_purge_no_fault`. Before M30, choosing the row
faulted the Run.

## EVENT-002 — Match and Keep's starter card per character (E11)

**Decision date:** 2026-11-01 (PM ruling E11). **Status:** disclosed decision, not source-settled.

Match and Keep's board holds one fixed starter card: Bash for the Ironclad, Neutralize for the
Silent, Zap for the Defect and Eruption for the Watcher. The PM accepted this recollection of each
character's event start card. The local sources settle only the Ironclad's. Test:
`m28_match_and_keep_constructs_e11_board`.

## EVENT-003 — An event room with nothing eligible becomes a monster room (E6)

**Decision date:** 2026-11-01 (M28 design §4.1 E6). **Status:** disclosed decision for a case the
source leaves open.

The event selector follows the source's branch delegation. When the branches end with no eligible
event and the source shows no result, the room becomes an ordinary monster room for the act, chosen
by the standard encounter law; no Leave-only event is invented. Tests: `m28_event_selection_law` and
`m28_event_room_monster_fallback`.

## EVENT-004 — Event formation slot order (E14)

**Decision date:** 2026-11-01 (PM ruling E14). **Status:** disclosed decision where the source does not
establish positions.

Event fights place their enemies in the encounter's listed order, and enemies act in slot order. For
Masked Bandits (Pointy, Romeo, Bear) the M28e source check confirmed that the source group keeps and
iterates this order, so only physical screen positions remain unmodelled. For the Colosseum Slavers
(Blue, then Red), the Nobs (Taskmaster, then Gremlin Nob) and Mysterious Sphere's two Orb Walkers, the
listed order is the project's slot order. Tests: `m28_formation_fights` and
`m28_event_formations_colosseum_and_bandits`.

## RNG-002 — Unseeded end-of-turn hand-card shuffle modelled with a dedicated seeded stream

**Decision date:** 2026-11-03 (PM ruling on source-check-endturn-v2, spire-oto.22). **Status:** disclosed
decision where the source is not replayable.

At the end of the player's turn the source's DiscardAtEndOfTurnAction copies the cards still in hand
and shuffles that copy with an unseeded `Collections.shuffle` (the JDK's default `Random`) before calling
each card's end-of-player-turn callback, so the order of the Ethereal exhausts is truly random. The
engine keeps that randomness but makes it reproducible: a Fisher-Yates shuffle whose draws are
HMAC-SHA256 words keyed by the combat's shuffle-stream key and labelled with the turn and position. It
reads no stream cursor, so no other stream advances. A fixed hand order is not used, because a policy
could exploit it. Tests: `fuzzfix_c_end_turn_hand_shuffle_is_keyed_by_combat_and_turn` and
`fuzzfix_c_ethereal_exhausts_follow_the_keyed_visit_order`.
