# Act 1 initialized RNG, map and joint-source closure

2026-10-19 · `spire-gli.2.3.32` · **Research candidate; independent review pending.**
Static evidence and exact conditional contracts only. No game class, initializer or
runtime was executed. This document advances the initialized-stream boundary; it
does not claim B02/B03 fully closed or choose a root-seed prior, host/JRE or native
RNG architecture.

## Finding and closure status

The upstream initializer is no longer an unknown black box for the audited paths.
For a fixed signed 64-bit run seed, the inspected source defines a concrete shared
root, named stream constructors, map-stream replacement, ordered five-pool seed
consumption and caller/source ownership for the linked Neow, potion and event
paths below. The corresponding initialized-state laws are **deterministic
conditional on that root, ordered catalog and declared history**. The previously
reviewed finite Java-shuffle kernel and map-topology kernel can therefore be
composed with actual source transitions instead of a generic unspecified
pushforward.

That is not an unconditional probability law. The allowed sources establish no
prior over player-entered numeric seeds, no measure over root/history pairs, and
no portable iteration order for the relic catalog's hash-backed input. Thus the
following remain explicit blockers, not independence assumptions:

* the Act1 initialized root-seed/history measure, including how an unseeded run's
  root is selected and how actions/observations condition it;
* the exact ordered relic-catalog inputs on the target game host/JRE (or a human-
  approved alternative); and
* an initialized-root witness or proof of exclusion for the zero-Elite map
  criterion. The prior 200,000-seed miss remains only a bounded search result.

This is a new source-to-stream/caller contract and evidence delta. It is **not** a
claim that those remaining input measures or the full Act1 joint law have been
evaluated. No native independent streams, sorted catalog, map retry, forced Elite
or legal cap is proposed as fact.

## 1. Evidence level and scope

Static bytecode from the owned desktop archive and the existing pinned source
reconstructions establish the claims below. The archive identity is the pinned
365,086,855-byte desktop archive with SHA-256
`cfad868ac8d65a88e71a0bf096fb09f78811e553effe0787c5309a655e081673`; installed
Temurin `21.0.12.1+1` and CFR `0.152` are inspection tools, not evidence of the
game's historical host JRE. The new seed-origin inspection selected only the
unique `SeedHelper` and `SeedPanel` archive entries; its first selection attempt
failed because the guessed `SeedPanel` entry path was absent and was retained.
The corrected entry was `com/megacrit/cardcrawl/ui/panels/SeedPanel.class`.

Earlier reviewed evidence is reused, not restated as fresh execution: the map
kernel/certificate and rank-barrier proof in [map-seed research](2026-10-19-act1-map-seed.md),
the finite Java shuffle and history-conditioning kernel in [shuffle-law research](2026-10-19-act1-shuffle-laws.md),
and event/caller facts in [Neow and event records](2026-10-19-act1-content-event-neow.md)
and [economy/potion records](2026-10-19-act1-content-economy-potions.md). Their
private artifact ledgers are not required to read or use this public contract.

The seed-origin inspection adds static bytecode for `SeedHelper` and `SeedPanel`.
The previously retained bytecode already includes `AbstractDungeon.generateSeeds`,
`AbstractDungeon.initializeRelicList`, `Exordium`, map construction/callers, RNG
classes, `RelicLibrary`, and the linked event, card and potion call sites. Exact
archive-entry identities, commands, pins and helper checks are in the private
assigned evidence lane. No source text, private path, helper or trace is published
here.

## 2. Root and source-owned streams

`AbstractDungeon.generateSeeds()` constructs twelve distinct stream objects from
the same `Settings.seed`, in this order:

| Construction order | Stream name | New-run seed input |
|---:|---|---|
| 1 | monster | root `s` |
| 2 | event | root `s` |
| 3 | merchant | root `s` |
| 4 | card | root `s` |
| 5 | treasure | root `s` |
| 6 | relic | root `s` |
| 7 | monster-HP | root `s` |
| 8 | potion | root `s` |
| 9 | AI | root `s` |
| 10 | shuffle | root `s` |
| 11 | card-randomizer | root `s` |
| 12 | miscellaneous | root `s` |

These are not independent random variables. At construction, each has the same
expanded XS state and zero wrapper counter, but is a separate object; subsequent
consumption on one object does not advance another. The static seed expansion
maps a 64-bit seed to two XS words using the audited hash expansion. Seed `0` and
`2^63` alias the same expanded pair; the image has `2^64 - 1` distinct pairs, not
all nonzero 128-bit states. The conditional mapping is exact; a prior over the
64-bit input is not supplied by the constructor.

`SeedPanel.confirm` trims its displayed string and delegates nonempty input to
`SeedHelper.setSeed`; that helper converts the accepted alphabet as a base-35
integer into the signed-64 seed value, with `O` normalized to `0`. Empty input
clears the explicit-seed flag and seed. The inspected seed helper also contains a
separate timestamp-seeded random-daily path. These paths do **not** establish a
probability law for standard new-run user choices or timestamp values. The panel catches `NumberFormatException` and assigns `Long.MAX_VALUE`; this
branch is not a random draw and is not a seed prior. Save loading instead restores
the saved root and per-stream counters; it is not the fresh-new-run initializer
modeled below.

### Source-domain dependency matrix

| Observable/caller | Source and initialization/cursor contract | Link that must be retained |
|---|---|---|
| Act1 map topology | `Exordium` replaces `mapRng` with a fresh wrapper seeded from `s + actNum`; for Act1, `actNum = 1`. This happens after the superclass initialization path. | The topology, conditional rejection steps and any repair draws all use that same replacement stream. Earlier relic initialization does not advance it. |
| Room-type bucket | Same `mapRng` XS object after topology. The descending bucket shuffle calls the underlying XS bounded primitive directly, bypassing the wrapper counter. | Bucket order is joint with the topology state/cursor, not a separately seeded permutation. |
| Burning-Elite marker | Same map stream after topology and bucket shuffle; source gates on Act4 availability and emerald-key ownership. | Enabled singleton selection still consumes a bounded draw. Enabled empty selection increments the wrapper counter then fails before raw advancement; it is not an observed reachable root. |
| Five initialized relic pools | The `relic` object is seeded from `s`. `initializeRelicList` populates C/U/R/Shop/Boss, then consumes one `relicRng.randomLong()` for each pool in that order, passing each word to a fresh Java `Random` shuffle. | The five words are consecutive outputs of one evolving source. Each Java shuffle's internal rejection/consumption is local to its newly seeded Java generator and does not advance `relicRng`. Owned relic deletion occurs after the five shuffles. |
| Neow categories and ordinary/forced-rare colored offers | The Neow event RNG is initialized from the run seed for its category construction and activation choices. Colored identity and rarity calls share that event-owned stream; duplicate retries keep the selected rarity and consume another identity draw only. | Drawback/category, offers and later actions are conditioned by the common root and the observed/action history. A fixed-rarity output does not erase the consumed rarity draw. |
| Neow colorless offers and deferred curse | Neow rarity is drawn on Neow's event RNG; colorless card identity uses `cardRng`. A pending curse flag is set at activation; its identity is later drawn from `cardRng`. | The two card-RNG uses share the advancing `cardRng` cursor, and histories can condition it jointly with Neow's event-RNG observations. Rejection/prevention does not restore a consumed draw. |
| Neow/Lab/Woman potion offers | Successive flat-ID selections use the shared `potionRng`; duplicates are allowed. Neow's ordinary combat reward also generates then removes its automatic card row. | Potion calls share a cursor; the discarded card offer changes card-RNG and ordinary rarity history and is not rolled back. |
| Question-room float and event branch/ID | The room transition creates an event stream copy from run seed plus the current `eventRng` counter, consumes the question-room float before Tiny Chest/Juzu handling, and installs the advanced copy on the ordinary path. `EventRoom` then creates the branch/ID copy from the resulting counter; branch precedes eligible-ID draw. | Room result, event branch, eligible event identity, list filtering and later history are one counter-linked joint, not two independent event samples. |

The table describes audited source ownership and initialization boundaries, not a
claim that every caller's whole history was statically exhausted or that the
target game's host/JRE/catalog state is known. Event/caller local laws and
conditional finite kernels remain in their linked documents.

## 3. Exact conditional construction (and its limits)

Let `s` be a fixed root seed; let `H` be a specified legal action/history
transcript; and let `O` be one complete ordered-catalog input under a specified
host/order rule. The audited source then defines, without an independence choice,
the following deterministic sequence:

1. Expand `s` into the common initial XS pair and instantiate the twelve named
   stream objects in §2.
2. Consume the five relic-source longs in C/U/R/Shop/Boss order. For each pool,
   apply the finite Java-seeded shuffle law to that pool's **ordered** input `Oᵢ`
   using its corresponding captured word. Apply source-ordered owned-removal and
   any later eligibility/depletion/promotion operation only at its actual call
   site; do not commute those operations through the shuffle.
3. For Act1 map creation, form the fresh `s+1` seed with signed-64 wrap semantics;
   expand that seed independently, run the six-path topology algorithm including
   every rejection/repair draw, then the direct-XS bucket swaps, row-major room
   assignment and gated marker draw. Preserve both wrapper counter and raw XS
   state. The accepted underfill and marker cursor details are in the map-seed
   report.
4. For each Neow, card, curse, potion and event caller, apply the source-owner
   sequence and state transition in the matrix above to the currently retained
   stream states. Apply observation/action conditioning to the **joint** latent
   transcript before projecting the requested observable. A skipped, rejected,
   depleted or hidden result remains part of the transcript when the source
   already consumed its draw.

For fixed `(s,H,O)` these operations yield a point mass on their deterministic
outputs. If a separately declared root/history/order prior is `π(s,H,O)`, an
observable event `Y=y` has mass equal to the sum of `π` over exactly those inputs
whose complete source transition yields `y`; for a history-conditioned query the
same sum is restricted to histories consistent with all observed outcomes and
normalized by their total mass. This is now a fully named conditional
construction: the initializers, source objects, counters/raw transitions,
ordered relic seed calls, exact map reset, shuffle operators and relevant
caller/depletion checkpoints are specified above and in the cited kernel
documents. It is **not** an evaluated unconditional distribution because the
source does not select `π` or `O` for a universal player-seed/host ensemble.

Useful consequences that do not need a prior:

* Given the same numeric root and Act1 context, map topology/bucket/marker are
  reproducible under the same source/JRE semantics. They are not independent of
  each other, nor are they the continuation of the pre-map relic stream.
* Given the same root and a fixed ordered catalog, all five relic pools are
  jointly determined from five linked source words and their five finite Java
  shuffles. Replacing them with five independent ideal permutations changes the
  conditional law.
* Given a fixed complete observed/action history, posterior weights are inherited
  from the joint root and consumed cursor path. Marginal uniformity of a primitive
  or a fresh event seed does not follow from this source wiring.

## 4. Facts, recommendations and fidelity decisions

### Static facts

* The source initializes the listed domains from one root and later uses distinct
  mutable stream objects; the exact new-run map replacement and five relic-seed
  call order are described above.
* The map topology and room assignment source kernel, its request/assignment
  underfill witness, and its exact conditional zero-Elite rank-barrier criterion
  are in the reviewed map report. No root-seed witness/exclusion is supplied.
* `RelicLibrary` population uses hash-backed iteration before the per-pool seeded
  shuffle. There is no evidence here that the actual input is sorted, nor a
  cross-host stable list. The exact iteration order is therefore a host/JRE
  parameter unless a target host is pinned.
* Neow/colorless/curse/potion/event ownership and cursor-order facts are summarized
  in §2 and sourced in the linked public normative records. Those records
  explicitly retain initialized/history-conditioned joints as open.

### Candidate project measurements (not game facts)

* A **conditional deterministic-seed** target can take the 64-bit numeric root,
  ordered catalog and legal history as explicit inputs and preserve the source
  transitions without pretending the player chose seeds uniformly.
* If a probability objective is required, a **declared uniform-64 numeric-root
  prior** is one possible project measure, not a source fact. Its induced law
  would use the same aliasing source map and common-root stream relation above;
  it must not be replaced by uniform 128-bit XS states.
* Pinning a particular target JRE/catalog enumeration is another possible
  fidelity contract. A canonical ID ordering could instead be proposed for
  review, but would not be described as the historical game order.

### Human decisions still required

Select whether the public contract is conditional on explicit seeds or needs an
unconditional prior; if the latter, define the root-generation/user-selection and
history-policy measure. Select a target JRE/catalog-order provenance or approve
an alternative ordered-catalog contract. Decide separately whether a map
construction exception on an enabled empty marker is an administrative fault;
do not make it a death/reward, retry, forced-placement, rollback or successful
markerless map by implication. These choices are not approved by this research
assignment.

## 5. Obligation ledger and remaining proof

| Obligation | Before this delta | New evidence / result | After |
|---|---|---|---|
| B02: map root and initialization | Fresh map seed/kernel and underfill certificate were audited; upstream initial root/history prior was not mapped. | Static `generateSeeds`/`Exordium` relation pins all twelve same-root new-run domains and the fresh `s+1` Act1 map stream; map topology then bucket call sequence is explicit. | **Partial.** Fixed-root map transition is specified. Root/history prior and initialized zero-Elite witness/exclusion remain open. |
| B03: five live relic pools | Conditional shuffle, post-shuffle ownership and local rejection/depletion laws existed; upstream five-word joint source relation and actual ordered catalog provenance remained missing. | `initializeRelicList` populates C/U/R/Shop/Boss, takes five consecutive relic-source longs in that order, and shuffles each ordered pool with a separate Java generator; no source RNG independence is asserted. | **Partial.** Conditional joint is specified for a fixed root and ordered inputs. Exact host ordering and unconditional root/history law remain open. |
| B03: Neow/card/curse/potion/event joints | Local primitive kernels and caller identities existed, but initialized joint boundary was explicitly unresolved. | Source-owned event/card/potion domains, deferred curse timing, duplicate retries, discarded Neow ordinary card row, and room-float→counter-copy→event-branch/ID sequence are now tied to the common root and concrete cursors. | **Partial.** Source dependency/cursor graph is concrete; actual root/history/action measure and evaluated posterior/joint outputs remain unclosed. |
| Zero Elite in initialized image | Conditional finite kernel criterion and supplied-choice fixture existed; a bounded search miss did not prove root reachability or exclusion. | The root question is reduced to the criterion evaluated on `G(s),B(s)` produced by the *same* `s+1` map stream. No unsupported independent topology/bucket substitution is admitted. | **Open.** No initialized witness or domain-wide exclusion proof. Search miss is not proof. |

The smallest honest blocker is not a larger blind search or another roster: first
declare the source/prior and target catalog ordering; then either produce one
complete initialized-root certificate satisfying the criterion, or prove the
criterion impossible over that declared root domain. The original runtime remains
unexecuted and is not an authorized route to fill the gap.

## 6. Checks and non-checks

Private standard-library source-wiring checks were authored failure-first. The
first red run failed because the helper did not yet exist; a follow-up test-edit
attempt exposed test-key mistakes and was retained; corrected tests then passed
4/4. They check: twelve distinct same-seed initial stream states/cursors; the five
successive relic seed words for root 5988; independent fresh Act1 map reseeding to
5989 without changing the relic cursor; 64-bit wrap/seed-alias behavior; and
negative controls for independent-per-pool reseeding and non-wrapping addition.
The `5988` output is an arithmetic/model check, not execution of source classes.
Prior map/shuffle failure-first, property and negative-control tests remain in the
prior private reports; they were not relabeled as new runtime evidence.

No original game class, initializer, constructor or runtime ran. No project
product/build tests, dependency installs, downloads, network probes, GPU probes,
launcher/save/profile access, sandbox survey, benchmark or training was performed.
All four pinned source/tool files were checked before/after the static work. The
tests do not establish a probability prior, host-specific catalog order, map
zero-Elite root witness, full-history posterior, or original-runtime agreement.

### Future discriminators (not executed gameplay tests)

1. For one declared root and pinned target JRE/order, independently reconstruct
   all twelve initial streams, the five captured relic words, all five input and
   output pools, and their depletion behavior; reject a five-independent-shuffle
   substitution.
2. For the same root, independently reconstruct Act1 map seed `s+1`, every
   topology bounded call/rejection/repair, direct-XS bucket swaps, room assignment
   and marker cursor; reject continuing the relic stream or counting bucket swaps
   as wrapper calls.
3. Observe Neow ordinary/forced-rare colored and colorless paths, then deferred
   curse and potion claims/skips; distinguish consumed override/retry draws,
   shared card/potion cursor, and the discarded automatic card row.
4. Observe a question-room event path with Tiny Chest/Juzu state, then event
   branch, eligible ID and later event state; distinguish the pre-override room
   float and linked counter copies from independent fresh domains.
5. Prove or refute the rank-barrier condition for at least one actual initialized
   root on the same map seed. A supplied legal choice, model-only map and search
   miss are not substitutes.

The integrated Act1 draft remains blocked; this document does not close whole-spec
approval, implementation planning or implementation-start gates.
