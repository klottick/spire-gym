# Reference-audit findings versus the Act 1/2 implementation

Checked against committed baseline `61c1d042b454a82addafa8ad08b46a939d5704bc`.
This is a focused maintenance follow-up to the reference-library audit, not an
expansion of the [captured Act 2 profile](../milestones/m7-act2.md).

## Applicability and disposition

| Audit finding | Implemented behavior at this baseline | Disposition |
|---|---|---|
| Mystic healing uses the living group's combined missing HP | `combat/enemies/act2/partners.rs` incorrectly required one actor to be missing at least 16 HP | **Fixed**: sum missing HP across living actors in the live context; select Heal at total 16 or more for the existing A0 profile |
| Mystic's higher threshold starts at A17; no third consecutive selected Heal | Mystic supports A0 only; the selected-history repeat limiter already excludes a third Heal | Keep the existing limiter; test it with distributed injuries. Do not add an A17 profile |
| Taskmaster's Wound increase starts at A3, not A2 | The implemented A0 Scouring Whip creates one Wound in discard; non-A0 profiles are unsupported | Correct at its supported scope; no payload or Ascension expansion |
| Snake Plant's A17 history cycle does not force the opener | Encounter ID/world-pool metadata exists, but no Snake Plant enemy handler | No existing implementation to correct; metadata is not mechanics |
| Spire Growth's Constricted increase and eligible forced selection start at A17 | Act 3 encounter metadata only; no enemy handler | Outside this Act 1/2 fix |
| Writhing Mass opens with Block Attack rather than Big Hit in the inspected source | Act 3 encounter metadata only; no enemy handler | Outside this Act 1/2 fix; later Reactive probabilities remain qualified |
| Deus Ex Machina's reference note overstates Miracle/self-exhaust order | No `CardId` or executable handler | No existing implementation to correct; do not add a card or infer scheduler behavior |
| FTL captures its threshold but reads the card count later | No `CardId` or executable handler | No existing implementation to correct |
| Wave of the Hand requests removal at end of round | No executable card/power handler | No existing implementation to correct |

The absence checks covered native registries, handlers and captured run paths,
not just display names. No corresponding Act 1 handler defect was found among
these audit subjects. Reference JSON and annotations are intentionally unchanged
in this maintenance patch; their separate source qualifications still apply.

## Mystic correction and regressions

The live selection context already excludes inactive/escaped actors. The selector
now sums only positive-HP actors' deficits, using a wider accumulator rather than
overflowing a creature-sized integer. Dead actors do not contribute. Heal remains
a priority selection with no ordinary Attack/Buff RNG draw; it does not bypass
the existing last-two-selected-Heals restriction.

Two injuries of 8 HP now trigger Heal even though neither actor individually
reaches 16. A combined deficit of 15 does not. Native regression tests exercise
both actor orderings, resulting group healing, dead-partner exclusion, unchanged
forced-selection RNG, and repeated selections without executing the heals.
These tests use the actual combat entry and queued effects, not a second AI model.

The source-qualified rule comes from the audit of `Healer` constructor,
`getMove` and `takeTurn`: selected class SHA-256
`04d11eb43426f1ebc9ae0c25d3e10f4f5faa814894f368749dfd8323b6581fbc`,
owned archive SHA-256
`cfad868ac8d65a88e71a0bf096fb09f78811e553effe0787c5309a655e081673`.
The archive's upstream PC v2.3 provenance remains unverified; this is static
source evidence, not original-runtime validation. No game files or reconstructed
source are published. The [enemy reference](../../data/enemies/README.md) retains
the community/source disagreements at their original grades.

## Intentional benchmark-contract change

The corrected priority changes the existing seed-7 public Act 2 prefix, without
changing its seed, resources or policy: the Champ-icon route now reaches the same
nonterminal floor-26/reward-26 boundary in **397 actions with 91 HP**. The old
workload expected 399 actions/49 HP. Its strict endpoint checks exposed this
change, so both workload and expected-contract versions advance to 2; the checks
are not removed or loosened. Old version-1 timing samples remain historical and
must not be compared as equivalent work. See [performance guidance](../performance.md).

This does not tune gameplay to force a win, change the reward policy, add content,
or turn valid Death into a simulator error. Existing milestone status documents
retain their dated historical outcomes.

## Verification boundaries

Both new regressions failed on the original per-actor predicate before the native
fix. The pre-change full offline workspace run recorded seven existing core-draft
failures: six Poison/Catalyst/Artifact cases in `combat::tests::full_cards`, and
`blue_slaver_and_two_fungi_encounters_construct_real_actors_across_ascensions`.
They are unrelated unfinished handlers/profiles and are not removed, ignored or
weakened here. Verification compares the candidate to that exact failure baseline;
this patch does not claim a fully green workspace or broader content acceptance.
