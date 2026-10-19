# Act 1 initialized maps and empty burning-Elite selection

2026-10-19 · `spire-gli.2.3.1` · **Research candidate; independent review pending.**
Scope: ordinary unmodded Ironclad, Ascension 0, Act 1, Act 4 available.
This is neither original-game execution nor implementation approval.

## Result

The initialized-stream model produces a concrete **three requested, two assigned
Elites** counterexample. Thus even initialized sampling must not equate the quota
with the assigned count. **Zero assigned Elites remains unproved for initialized
root seeds.** The bounded search is not an impossibility result.

New closure consists of the initialized underfill certificate, a necessary-and-
sufficient zero-Elite criterion, and the exact empty-marker failure/cursor
contract. The earlier [supplied-choice witness](2026-10-18-act1-rng-map-closure.md)
remains valid at its stated kernel scope; it is not promoted to a root-seed proof.

## Initialized underfill certificate

Coordinates below are zero-based `(column, row)`. The private, independently
authored model starts with **map seed 5989**, corresponding to numeric run seed
**5988** when Act 1 initializes the map stream with `run seed + 1`.

| Quantity | Deterministic model result |
|---|---|
| Quota base / ordinary unassigned capacity | `N = 41`, `U = 37` |
| Initial padded bucket counts | Shop 2, Rest 5, Elite 3, Event 9, Monster 18 |
| Assigned Elite coordinates | `(3,5)`, `(6,12)` |
| Ordinary assignment failures | `(5,13)`, `(6,13)` |
| Unconsumed bucket | one Rest, one Elite |
| Final repair | both null nodes become Monsters, not Elites |
| Burning-Elite marker selected | `(6,12)` |

At both failed nodes, Rest is floor-ineligible and Elite matches a parent. Failed
assignment consumes neither token. The final fill happens only after the ordinary
scan, explaining why a requested Elite remains unused.

The certificate retains the full seeded RNG and graph/assignment transcript
privately. Its evidence grade is **static source tracing plus an independently
reconstructed model**, not a successfully played original run. Source/model
correspondence requires independent review. It establishes neither zero-Elite
realizability nor a universal two-Elite minimum.

## Two different RNG cursors

New-run `Exordium` creates a fresh map RNG immediately before map generation.
Earlier relic/scene work therefore does not advance this replacement stream.
Initialization expands its 64-bit seed into the two XS state words; seed zero
aliases the minimum signed 64-bit value. This is not an arbitrary 128-bit state.

| Stage | Game wrapper counter | Underlying XS advancement |
|---|---|---|
| Initialization | zero | initialized state, no final-stream output consumed |
| Six walks | 90 ordinary calls, plus second-start retries and ancestor-repair calls | at least one raw step per call; bounded rejection can add steps |
| Room-bucket shuffle | **no increment** | descending bounded swaps use the underlying XS object directly |
| Ordinary assignment and final null fill | no increment | no map-stream draw |
| Enabled marker selection, nonempty candidates | one increment | bounded selection consumes raw output, including when only one candidate exists |
| Enabled marker selection, empty candidates | **one increment** | **zero raw steps; exception before sampling** |

Node visual jitter/animation uses separate `MathUtils.random` calls, not the passed
map stream. The room-bucket shuffle is not one of the Java-48-bit-seeded shuffles:
it receives XS itself. A wrapper counter alone consequently cannot reconstruct
the map RNG state.

For map seed 5989, there are six start calls, 84 successor calls, eight repair
calls and no second-start retry: wrapper/raw cursors are both 98 after topology.
The 36 bucket swaps advance raw cursor to 134 while wrapper counter stays 98.
Marker selection advances them to 135 and 99 respectively. No bounded rejection
occurs in this retained trajectory. These are model-checked counts, not runtime
measurements.

## A sharper zero-Elite proof obligation

Fix a completed topology and its actual shuffled, individually labeled room
bucket. Give tokens their positions in that shuffled bucket, and let `e` be the
position of its first Elite token. Consider the same ordinary assignment with
Elite tokens suppressed, preserving all other rules and delayed null filling.
For each ordinary Elite-eligible node `v`, let `r(v)` be the position of the token
chosen in that suppressed assignment, or infinity if none is chosen.

**For a positive Elite quota, the source kernel assigns zero Elites exactly when
every `r(v) < e`.** Eligible ordinary rows are 5–13 except fixed Treasure row 8;
fixed rows 0 and 14 are not ordinary assignments.

Proof: before the first Elite assignment, the two scans have identical non-Elite
tokens, earlier rooms and nulls. Early rows reject Elites by floor. At every later
eligible node an Elite is legal until the first one has been assigned: there is
no earlier Elite parent or sibling to exclude it. The source therefore chooses
the suppressed scan's token precisely when it precedes the first Elite. A first
violation assigns an Elite; absent any violation the scans agree to completion.
Final null filling creates only Monsters.

A useful necessary consequence follows when the padded bucket has exactly `U`
tokens and Elite quota `q > 0`: **zero Elites requires at least `q` ordinary holes
in rows 1–4, and no ordinary holes later.** Every unused Elite contributes to the
remaining bucket, whose size equals the number of failed ordinary assignments.
An eligible hole cannot occur while an unassigned Elite remains legal. Enough
early holes alone is not sufficient; token ordering still matters.

This criterion was checked against all 120 category-block orders on the accepted
supplied-choice topology and against 128 initialized model maps. The mathematical
argument, not those finite checks, supplies the equivalence. It does not make
topology and bucket independent.

### What still has to be proved

Let `G(s)` and `B(s)` denote the topology and shuffled bucket obtained from the
**same initialized map seed `s`**, including all conditional bounds, repairs,
rejections and cursor advancement. The remaining question is whether an
admissible run seed produces an `s` for which the criterion holds for
`G(s), B(s)`. Either of the following closes it:

1. One admissible root seed, a complete deterministic certificate and independently
   checked source correspondence establishing the criterion; or
2. A proof excluding that criterion over the declared initialized-seed domain.

The completed deterministic search examined map seeds 1 through 200,000 and found
no zero-Elite case; its minimum was two. This is only a bounded search miss, not
either proof above, a frequency-derived law, or a gameplay limit. Unsafe game
execution and new dependencies were not used to extend it.

For an explicitly chosen uniform ensemble of all 64-bit map-seed inputs, the
zero-Elite probability is the number satisfying this predicate divided by
`2^64`. Under a different admissible seed/history measure, it is that measure's
mass on satisfying seeds. The source fixes the conditional mapping, not a unique
prior over user inputs. Neither uniform full-128-bit states nor independently
legal supplied choices are substitutes for this initialized ensemble. No native
sampling measure is selected here.

## Empty burning-Elite selection: source branch matrix

The marker operation scans grid rooms for Elite instances in row/column order.
It does not separately require outgoing edges. In the scoped ordinary map,
nonactive nodes do not acquire Elite rooms.

| Act 4 available? | Emerald already owned? | Candidate count | Source-local result |
|---|---|---|---|
| No | either | any | return without marker RNG call |
| Yes | Yes | any | return without marker RNG call |
| Yes | No | positive | bounded index selection; chosen node's marker flag becomes true |
| Yes | No | zero | wrapper counter increments; nonpositive bound throws `IllegalArgumentException` before raw XS advancement, list indexing or marker mutation |

The empty branch is **map-creation failure, not legal defeat**. Map construction
and room assignment have already mutated the map; `firstRoomChosen` is set false
and fade-in is requested before marker selection. There is no local rollback,
regeneration, forced Elite or death conversion in the inspected marker,
generation, Exordium-construction or immediate dungeon-factory path. This is not
a claim about the outer launcher's complete exception presentation.

The conditional failure is established independently of whether an initialized
seed reaches it. It must not be advertised as an observed original crash, a
playable zero-Elite map, or a negative-reward game outcome.

## Recommended specification treatment — not an approved new rule

Retain an explicit administrative map-construction fault if enabled marker
selection encounters no candidates. Do not silently reroll, force an Elite,
continue with a markerless map, or report death/reward −1. This follows the
existing administrative-failure boundary without asserting that the branch is
reachable under initialized roots.

Exact initialized support still needs the certificate/exclusion proof above and
an explicit native map-law decision. A guaranteed-success sampler, rejection
conditioning, forced placement or altered marker policy changes the mapping/law;
each would require an explicit fidelity decision, not an appeal to RNG-001's
seed-parity waiver. The whole specification must retain this precise unresolved
obligation rather than declare map nonemptiness proved.

### Future native acceptance cases, not executed product tests

| Case | Required discriminator |
|---|---|
| Seeded underfill certificate | preserve two assigned Elites rather than enforcing the quota of three |
| Supplied-choice zero-Elite fixture | preserve delayed Monster filling; do not claim root-seed realizability from it |
| Empty candidates with either marker gate disabled | no marker RNG call or fault |
| Empty candidates with marker enabled | administrative fault, no synthetic defeat, reroll or successful map |
| Singleton enabled marker and forced bounded rejection | distinguish wrapper calls from raw steps; do not skip a singleton draw |

## Evidence and actual checks

Pinned owned archive SHA256:
`cfad868ac8d65a88e71a0bf096fb09f78811e553effe0787c5309a655e081673`.
All four authorized archive/tool pins were checked before and after inspection.
Twenty-two claim-relevant unique ZIP entries and installed-library shuffle
bytecode were inspected with the pinned `javap`; no game class or initializer ran.
Historical host-JRE provenance remains unestablished.

Source anchors are `Exordium`'s new-run constructor and level chances;
`AbstractDungeon.generateMap`, `setEmeraldElite`, `fadeIn` and transition setup;
`MapGenerator`'s path/ancestor/crossing/trim operations; `RoomTypeAssigner`'s
distribution and eligibility methods; game `Random` and `RandomXS128`; map-node,
edge and room construction; and `CardCrawlGame.getDungeon`.

Private evidence locator: `verification/2026-10-19-act1-spec-closure-v1/map-seed/`
contains the claim ledger, identity records, seeded certificate and checks. This
is an evidence locator, not a public hyperlink or required document dependency.

Seven stdlib-only model tests passed, including failure-first development,
forced bounded rejection, invalid-bound counter/state behavior, singleton marker
consumption, repair-required tape rejection, the prior supplied-choice witness,
initialized underfill, shuffle-counter bypass and the rank-barrier comparisons.
The separate 200,000-seed bounded search completed. Model agreement is not
independent original-game validation. Source, helpers, fixtures and detailed
traces remain private. Product builds/tests were not run for this document.
