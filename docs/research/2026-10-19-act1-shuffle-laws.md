# Act 1: compositional finite shuffle laws

Date: 2026-10-19. Task `spire-gli.2.3.2`. **Research and candidate fidelity
contract; independent review, integration and human decisions pending.** No game
execution or implementation approval is claimed.

## 1. What this adds

An exact law need not be a printed table of every permutation. Sections 2–7 give
a finite transfer-kernel specification, its termination/normalization argument,
observable projections and history conditionals. It covers arbitrary finite
Java-seeded list sizes in the source's integer-index domain, including five
faces, twelve board instances, instance-upgrade prefixes and five linked relic
pools. No independent-swap or equal-permutation assumption is used.

New consequences include:

* Under uniform64 shuffle input, five missing faces cannot have exactly equal
  probabilities. A twelve-instance board with six distinct pairs cannot have
  exactly equal probabilities across its 7,484,400 identity boards either.
* Deleting owned entries **after** the finite shuffle generally differs from
  shuffling the survivors. A concrete exact counterexample appears in §7.
* All five ordinary relic pools have at least seventeen entries before removals;
  their seeded shuffles cannot reach every labeled permutation from 48 seed bits.
* Reveals, accepted/rejected relic offers and consumed ends condition one retained
  hidden object; they do not authorize a fresh shuffle of compatible objects.
* Source input order is substantive: relic population uses hash-map encounter
  order, not sorted display lists. An ordered catalog is an explicit parameter
  until a source-host order or a proposed canonical order is selected.

The [earlier RNG/map report](2026-10-18-act1-rng-map-closure.md) established the
three-item table but left these compositions open. This document advances the
mathematics, not the unknown actual seed/history ensemble. The
[relic audit](2026-10-18-act1-relics-closure.md) supplies roster/predicate context;
the [native contract draft](2026-10-18-act1-native-contract-closure.md) remains a
proposal, not approval of a measure.

## 2. Exact finite Java-seeded kernel

This is a mathematical probability specification, not executable game logic or
an implementation prescription. Let M=2^48, a=25214903917, c=11. Internal state
space is S={0,…,M−1}. Define the permutation matrix T by T(s,t)=1 exactly when
t=(as+c) modulo M. For a post-transition state t, let u(t)=floor(t/2^17).
An input word x has initial state g(x)=(x XOR a) modulo M.

For a positive bound b≤2^31−1, partition S into accepted-output sets A(b,j),
0≤j<b, and rejected set R(b):

| Bound | Accepted states with output j | Rejected states |
|---|---|---|
| b a power of two | floor(b·u(t)/2^31)=j | None |
| Otherwise | u(t)<L(b) and u(t) modulo b=j, where L(b)=b·floor(2^31/b) | u(t)≥L(b) |

Let D(A) be the diagonal indicator matrix of set A. The exact bounded-draw
kernel, **including the final generator state**, is

Q(b,j) = Σ[r=0,…,M−1] (T D(R(b)))^r T D(A(b,j)).

This explicitly includes every possible rejected prefix, not a normalized
single-step approximation. The LCG has a full M-state cycle: c is odd and
a≡1 modulo 4. Each bound has a nonempty accepted set, so every trajectory accepts
within M advances. The sum is finite, and for each input state exactly one
(j,final state) has mass one. Consequently Σ[j]Q(b,j) is row-stochastic. It is
generally wrong to multiply independent marginal probabilities for the j's.

For an ordered list A of n **distinct instance labels**, let S(b,j) be the
permutation matrix on labeled lists that exchanges positions b−1 and j. Define
the lifted operator U(b)=Σ[j]Q(b,j)⊗S(b,j). The complete joint law of final Java
state and list, from initial-state law ν, is

λ(A,ν) = (ν⊗δ(A)) U(n) U(n−1) … U(2).

For n=0 or 1 the empty product is identity. Marginalizing its Java-state
coordinate defines Kν(A,·). Every matrix entry, index set and product is specified;
this is an exact finite construction even when its enormous matrices are never
materialized. Its proof is composition of the unique accepted transitions and
the corresponding position permutations. Induction over descending bounds gives
the inspected library shuffle law, and row normalization is preserved at each
factor. It does not require enumeration of 2^48 states or a fast counting claim.

Under **uniform64 input**, g has exactly 2^16 preimages per state. Thus ν(s)=1/M.
For any observable F of the final labeled list,

Pr(F=y) = Σ[p:F(p)=y] Kν(A,p).

Every mass is an integer multiple of 1/M. Equal weights of labeled permutations,
card kinds or surviving entries do not follow. For a fixed input word the law
is a point mass; for any other seed law μ replace ν by g's pushforward of μ.
These are different inputs to the same exact kernel.

Two immediate proofs are useful. For n≥3, n! has an odd divisor, so 1/n! cannot
be an integer multiple of 1/M: the labeled law is not ideal uniform. For n≥17,
n!>M, so some labeled orders have zero mass. The latter is a seed-image theorem,
not a bounded-search miss and not a statement that a particular order is absent.

## 3. Conditioning and composition contract

Carry the joint latent law, not just a head/prefix marginal. Let z contain the
ordered inputs, linked source RNG states, shuffled instance lists and any
mechanic-relevant state. Let w(z) be its mass. For a legal action transcript h,
C(h) is the set of latent states consistent with **all** observations and
resolved effects in that transcript. For a subsequent observable Y,

Pr(Y=y | h) = Σ[z∈C(h),Y(z)=y] w(z) / Σ[z∈C(h)] w(z).

The denominator must be positive; a zero-mass history has no conditional law
under that measure, not an invented retry/outcome. Composition with another
stochastic producer uses its joint conditional kernel before this projection.
If a policy chooses actions using public history alone, its action likelihood
is constant across compatible hidden states and cancels. A hidden-informed or
externally conditioned policy requires its actual likelihood; actions are not
automatically harmless evidence in an arbitrary ensemble.

This covers first/second reveals, mismatches, matched removals, upgrade results,
offer skips, eligibility failures and front/back depletion. Remembering a reveal
is not exposing unobserved positions. Sampling a compatible continuation from
this posterior is distributionally legitimate; sampling it **uniformly** without
its retained weights is not.

## 4. Face Trader: every missing-set size

At trade resolution, the source constructs the missing subsequence of this
ordered ID list: `CultistMask`, `FaceOfCleric`, `GremlinMask`, `NlothsMask`,
`SsserpentHead`. If that subsequence is empty it substitutes the singleton
`Circlet`. It seeds a Java shuffle with one `miscRng.randomLong()` and obtains a
copy of the first entry. Ssserpent Head belongs here, not The Ssssserpent reward.

Let A(O) be that ordered missing subsequence for owned set O, or `(Circlet)`.
For each face f the exact conditional mass is Kν(A(O),{p:p[0]=f}), using the
applicable ν conditional on the source history—not a global five-face weight.

| Missing count | Exact uniform64 consequence | Source consumption |
|---|---|---|
| 0 | Circlet with probability one | One source long; no Java bounded draws |
| 1 | The sole missing face with probability one | One source long; no Java bounded draws |
| 2 | Each missing face exactly 1/2; T is bijective and its high bit is balanced | One source long; one Java bounded draw |
| 3 | Head projection of U(3)U(2), with the retained unequal finite weights | One source long; Java rejection paths retained |
| 4 | Head projection of U(4)U(3)U(2); no equality assumption | Same boundary |
| 5 | Head projection of U(5)…U(2); not all five masses equal since 5 does not divide M | Same boundary |

The probability-one cases are not zero-consumption source calls. Removing an
owned face happens **before** this shuffle, unlike initialized relic-pool owned
removals. Repeated trades, if considered in a legal context, reconstruct the
current missing list; they are not successive heads of one original face order.

## 5. Match and Keep: labels, duplicate IDs and reveal memory

At A0 the six initial cards are drawn/constructed in this order: Rare, Uncommon,
Common, Uncommon colorless, random curse, current character's event starter
(Bash for Ironclad, not a random Strike/Defend).
Preview-obtain relic hooks apply to each original before its stat-equivalent copy
is appended to the six-copy suffix. The resulting twelve objects are shuffled
once using a `miscRng` long. The source compares matches by **card ID**, not by
which original/copy pair an object came from. The construction must therefore
not invent six exclusive pair identities. The formula below also handles repeated
IDs; it does not assert every multiplicity pattern is reachable in standard A0.

Let D=(d0,…,d5,d0′,…,d5′) be the complete pre-shuffle instance records, with
their generated/previewed properties. Labels are distinct even if IDs coincide.
The source places list index i at column i modulo 4 and row i modulo 3; this is
**not row-major placement**. It is a bijection onto twelve cells. Define B(D,p)
by this placement and by projecting each label to its complete revealed card
record. Then the board law is the weighted pushforward of Kν(D,p) by B.

If only IDs are observed and identity multiplicities are m1,…,mk, an identity
board's fiber has ∏[j]mj! labeled permutations. Its probability is the **sum of
their actual weights**, not that factorial times one arbitrarily chosen weight.
With six distinct IDs appearing twice there are 12!/(2!)^6=7,484,400 identity
boards. This number has odd factors, so they cannot all have equal mass under
uniform64 seed input. Uniform labeled permutations would induce uniform identity
boards for fixed multiplicities, but that is a different initial measure.

For a reveal transcript h, retain boards matching every previously revealed
record at its original cell, plus matching/removal outcomes. For a chosen
unremoved cell q, the next revealed record's mass is §3 with Y=B[q]. After a
first reveal at q, the chance that a second cell r matches is the same ratio
with predicate ID(B[r])=ID(B[q]). Do not replace it with “one partner among the
remaining cells”: an ID may have more than two instances, and even a two-copy
board is not assumed exchangeable under its seeded measure.

There are five completed attempts, each consuming two legal flips. A resolved
match removes those two objects and queues **one** acquisition made from the
chosen card's ordinary copy; a mismatch turns both down without deleting memory
of what was seen. Both outcomes decrement attempts, including the fifth match.
Remaining cards retain their placed cells: list removal does not re-place them.
The law retains complete records until projection: ID equality alone is not
sufficient to forget distinct reveal properties or the chosen-card acquisition
route. Cleanup timing and acquisition callbacks are separate continuation facts,
not newly executed gameplay tests here.

The conditional shuffle law is now specified for each D. Its unconditional law
is Σ[D]Pr(D|h)Kν(·|D,h), not necessarily Pr(D|h) times an independent uniform64
kernel. Card generation and this seed draw use different source stream objects
but may share a root and conditioned history. No complete Pr(D|h) is evaluated
here. §8 makes declaring such independence a decision, not a source fact.

## 6. Upgrade-instance prefixes

Each audited caller filters **master-deck encounter order**, retaining individual
instances, then unconditionally consumes one `miscRng` long to seed its shuffle.
The empty/singleton cases still consume that source long.

| Caller | Eligible ordered instances A | Upgraded prefix length |
|---|---|---|
| Shining Light | Currently `canUpgrade` | min(2,n) |
| Whetstone | Currently upgradeable Attacks | min(2,n) |
| War Paint | Currently upgradeable Skills | min(2,n) |
| Tiny House | Currently `canUpgrade` | min(1,n) |

Here n is the eligible-instance count. For length k, the exact **ordered** selected-instance law is
Pr((i1,…,ik))=Σ[p:p[:k]=(i1,…,ik)]Kν(A,p). An unordered selected-set law sums
over its distinct ordered prefixes. Card-kind projections happen afterwards;
two same-kind instances are not collapsed before filtering or selection. For
k=|A| the selected set is certain, but the processing order need not be.

The full permutation kernel also retains caller-specific extra references.
Notably, Tiny House with at least two candidates upgrades only the first, yet
performs bottled-card checks for both first and second. Thus its complete caller
projection may need a two-instance prefix even though its upgrade projection
needs only one. This is not a claim it upgrades two cards. Subsequent potion
generation explicitly receives `miscRng`, so its source cursor follows the seed
draw even if no card can upgrade. This document does not establish that potion's
complete acquisition law.

Other paths using `CardGroup.shuffle()` seed from `shuffleRng`; its explicit-RNG
overload uses the passed stream. They can use §2's mapper, but not an inferred
misc-stream measure or the four callers' eligibility rule.

## 7. Five relic pools, owned removal and consumption

Let A=(A_C,A_U,A_R,A_S,A_B) be the five **ordered pre-shuffle** pool lists.
Initialization draws five successive longs from `relicRng`, in Common,
Uncommon, Rare, Shop, Boss order; each long seeds its own Java shuffle. Every
long is consumed even for a hypothetical empty/singleton list. Java rejection
advances only that invocation's Java state, not `relicRng`. The source wrapper's
call counter and the underlying generator cursor are not interchangeable when
earlier bounded source draws have rejected.

`RelicLibrary.populateRelicPool` appends eligible shared-map entries and then
the actual character's exclusive-map entries. These are hash-map iteration
orders, not the separately sorted display lists. For fully unlocked Ironclad
the pre-removal counts are (33,30,28,17,22). Exact ID order remains an input to
this law; historical host-JRE order is not established merely by these counts.
Shard or a Note card does not change the character-exclusive relic basis.

Let η(s1,…,s5) be the **joint** law of the five folded Java seeds. Given a seed,
let J(A_t,s_t) denote the deterministic list output specified by §2. Let R_O
delete the source-specified starting/owned removal IDs after shuffling,
preserving survivor order. O denotes that removal list, not an inferred filter:
initialization appends current owned relic IDs to it when floor number≥1.
The exact retained-pool joint law is

W(P) = Σ[s1,…,s5] η(s1,…,s5) · 1{P=R_O(J(A_C,s1),…,J(A_B,s5))}.

There is no factorization unless η and any jointly random A,O justify it.
Owned IDs may themselves carry information about earlier offers; conditioning
on them is not the same as declaring a fixed exogenous deletion set.

**Deletion does not commute with this mapper.** For an initial triple (0,1,2)
under uniform64, delete label 1 after shuffling. Using the reviewed exact
three-item weights, survivor order (0,2) has numerator 140737488352549 over M;
(2,0) has numerator 140737488358107. Shuffling (0,2) directly instead gives
M/2=140737488355328 per order. The discrepancy is 2779/M, not numerical noise.
This is a small exact counterexample to the general optimization, not a claim
that a particular Act 1 pool has three entries.

### Consumption as a deterministic pushforward, not fresh randomness

For each retained pool state P, source context τ, requested tier t and helper
mode N(normal) or E(end), define the selection transcript by these static rules:

| Situation | Removed/returned candidate or next helper |
|---|---|
| N, nonempty C/U/R/S/B | Remove front |
| E, nonempty C/U/R/S | Remove back |
| E, nonempty B | Remove front |
| Either, empty C | N at U |
| Either, empty U | N at R |
| Either, empty S | N at U |
| Either, empty R / B | Circlet / Red Circlet |
| Candidate fails `canSpawn` | Candidate stays consumed; E at the **requested tier of that call** |

A promoted call returns through its caller's own eligibility check; its rejected
result, if any, therefore follows that caller's requested-tier rule. Do not
flatten the stack into a permanently promoted tier. Circlet and Red Circlet
inherit the base accepting predicate in the inspected classes; they terminate
ordinary depleted-tier selection. Other source-specific exclusions (screenless,
non-campfire, merchant context) are outer selectors, not pool-wide removals.
They must compose their actual consumed/retried requests with this table.

Call the resulting deterministic map F_a(P,τ)=(offer,new P,new τ,transcript),
where a supplies the source request and actual predicate context. Each rejected
pool candidate decreases the finite total pool size; promotions are acyclic and
fallback accepts. This establishes finite resolution of these inner helpers,
not termination of every outer selector under every invented predicate.
Push W jointly through F_a, retain its state, then condition on the actually
observed offer/effects by §3. For successive requests use the composed F's.
Skipped offers remain consumed. Rejected candidates can remain private; the
posterior sums over compatible hidden rejection traces rather than exposing
them. No reroll of the catalog or resampling a uniformly eligible survivor is
equivalent in general.

For example, N on `[bad,middle,tail]` with only `bad` ineligible consumes `bad`
then `tail`, returning `tail` and retaining `[middle]`. E on empty Common with
Uncommon `[u0,u1]` returns `u0`, not `u1`. Boss E always consumes its front.
These are deterministic discriminator fixtures, not initialized seed witnesses.

## 8. Explicit input-measure choices and recommendation

The following source-state mathematics makes a correlated alternative precise
without publishing an executable RNG port. On 64-bit vectors, + between vector
maps means XOR, L_k/R_k are zero-filling left/right bit-shift matrices, and
integer products/sums below are modulo 2^64. Define Z=I+R_33 and multiplication
maps M_d(x)=d·x. Let

H = Z ∘ M_0xc4ceb9fe1a85ec53 ∘ Z ∘ M_0xff51afd7ed558ccd ∘ Z.

For a word r, e(r)=r except e(0)=2^63. Initialized source state is
σ(r)=(H(e(r)),H(H(e(r)))). Define the binary linear state map

X(u,v) = (v, (I+R_17)(I+L_23)u + (I+R_26)v).

The emitted word at updated state (v,z) is v+z modulo 2^64. This specifies
finite state/word relations; it does not mandate these states or source words
as a native representation. H is bijective (odd multipliers and invertible
XOR shifts), but e aliases 0 and 2^63. Thus the initialized state ensemble has
2^64−1 distinct states, one with twice the weight under uniform64 seed prior;
it is not uniform over the 2^128−1 nonzero source states.

For any source entry-state law ρ, define ηρ as the pushforward of ρ through
five successive X transitions, emitted-word relation and g. This **fully
specifies** the five-input law in §7. Retaining X^5's state also specifies the
joint law with later `relicRng` activity. For later positive bounded source
draws, use the first-accepted transition relation with u=floor(word/2),
L=b·floor(2^63/b), u<L and result u modulo b. Include all rejected source
transitions before the next seed-producing long. Do not use the Java Q for
these source draws: its state space and bit extraction differ. This relation
can be defined over its finite state graph: search through at most 2^128−1
distinct nonzero states; a wholly rejecting cycle has a separate no-return
marker. That marker completes the mathematical relation, not an asserted
reachable source failure or a native defeat/retry policy. No source-cycle
termination theorem for every possible later bound is claimed here; the five
unbounded initialization-long transitions require no such theorem.

| Candidate target measure | Exact meaning | Decision/limitation |
|---|---|---|
| **U64 per declared independent shuffle** | ν is uniform on S; for five independent pool seeds η=ν⊗5 | Simple, preserves each invocation's exact mapper bias. **Explicitly discards** source five-input correlation; RNG-001 does not already approve that. Nor does it prove independence from board/card generation. |
| **Initialized semantic block** | One uniform64 block word r; ρ=σ's pushforward; linked calls retain the source-state joint relation above | Preserves initialized within-block seed-image/cursor relationships, without tying the block to unrelated root mechanics. Marginals are not assumed U64. Boundary, ordered inputs and linked callers require approval. |
| **Full nonzero source-state block** | ρ uniform on all nonzero 128-bit states; use ηρ | Preserves sequential source-state correlation but changes initialized support/weights. Not the initialized game law. |
| **Actual initialized/history-conditioned source ensemble** | Declare root/profile prior and policy/history likelihood; push to entry state, ordered inputs and context, then apply §§2–7 jointly | Source determinism alone does not choose that prior. All earlier draws/rejections and history selection matter. No evaluated full Act 1 ensemble is supplied here. |
| **Ideal uniform permutations** | Replace K by equal labeled-permutation weights | Removes finite mapper bias and, for large pools, changes seed-image support. Requires an additional fidelity deviation; not implied by seed nonparity. |

For the full-nonzero-state option, X is invertible. At any one word position,
each emitted word has 2^64 state preimages, except zero loses the excluded zero
state. Consequently if an observable has uniform64 Java-seed count C over M,
its one-invocation mass under this option is
(C·2^80−1{the word-zero outcome has that observable})/(2^128−1).
This derives a larger-law marginal transformation, **not** a five-pool product
formula. Five emitted words together have at most 2^128−1 possible tuples;
initialized blocks have at most 2^64−1, versus 2^320 independent uniform word
tuples. Difference of latent measures is not by itself proof that every
observable distinguishes them; observable equality would need its own proof.

**Recommendation for human consideration, not a selected policy:** keep the
exact mapper and retained-history kernels. Prefer an **initialized semantic
block for relic-pool initialization and its subsequent linked relic draws**, so
cross-pool/cursor information is not discarded without proof. For genuinely
isolated event/upgrade shuffles, U64 is a simpler defensible target, but approve
each independence boundary explicitly—especially board generation versus board
shuffle and Tiny House upgrade versus potion generation. If boundary closure is
not established, retain a joint conditional input, not a default independent
stream. An alternative whole-spec decision may prefer a broader idealized law;
that must be stated as such rather than disguised as source parity.

Also recommend ascending UTF-8-byte constructor-ID order across each native relic
input list **only as a separate proposed deviation** from unspecified host
hash-map order. An exact source-host order instead requires its own established
ordered catalog. These are not interchangeable under a biased mapper. The
conditional law is complete for either declared ordered input; the declaration
itself remains unapproved.

Any eventual target sampling remains under `spire-lab-rng-hmac-sha256-v1` and
reviewed semantic scopes; the mathematical latent-state description does not
replace that contract with a source RNG implementation or promise original
seed/cursor parity. “Uniform64” describes a normative input measure, not a
theorem that deterministic HMAC outputs over an unspecified seed population are
independent uniform random variables. No new domain IDs are approved here.

## 9. Evidence, checks and exact remaining obligations

**Static evidence:** owned JAR SHA256
`cfad868ac8d65a88e71a0bf096fb09f78811e553effe0787c5309a655e081673`.
Four archive/tool pins were verified before/after. Sixteen exact unique class
entries were freshly inspected with pinned Temurin 21.0.12.1+1 `javap`, plus
installed `java.util.Random` and `java.util.Collections`. CFR 0.152 was pinned,
not newly invoked. Tool JVM option-injection variables were removed. Original
game classes, constructors, initializers and runtime were **never executed**.
Historical host JRE and upstream build/channel remain unestablished.

Source anchors are `FaceTrader.getRandomFace`; `GremlinMatchGame` construction,
`initializeCards`, `placeCards`, `updateMatchGameLogic`; the four upgrade callers;
`CardGroup.shuffle` overloads; `RelicLibrary.populateRelicPool`;
`AbstractDungeon.initializeRelicList`, both relic-key helpers and RNG setup;
the source Random wrapper/XS state relations; Ironclad's event starter and
CardLibrary's curse source; base/Circlet/RedCirclet predicates;
installed Random seed/bounded methods and Collections shuffle. Prior map/random,
generation/acquisition and relic evidence was read-only context. Private evidence
locator: wave `2026-10-19-act1-spec-closure-v1`, lane `shuffle-laws`, claim ledger.
No private source, helper, fixture, raw log or attachment is published.

**Mathematics/model grade:** nine private Python-standard-library tests passed
after nine expected missing-helper failures. Reduced-domain exhaustive checks
cover first-return rejection/cursor and descending composition; full-width narrow
controls cover a power-of-two draw and constructed rejection-tail input. Weighted
duplicate-board posteriors, instance prefixes/deletion, non-product joints,
relic rejection direction/promotion/fallback and initialized-state alias/cursor
controls passed. These checks exercise independently authored helpers; they are
not independent original-game validation or an evaluated full-size distribution.
Four deliberate model mutations were caught after an end-mode test-coverage gap
was strengthened; that initial failed verification attempt is retained privately.
The deletion counterexample uses the previously reviewed exact triple counts;
this lane did not recompute their full-size floor sums.

Future acceptance cases, **not executed gameplay tests**, must discriminate:

| Case | Required distinction |
|---|---|
| Face missing sizes 0–5 | Correct ordered support; seed consumption even for deterministic result; five-way equality not assumed |
| Board repeated ID and mismatch/revisit | Weighted label-to-record pushforward, ID match equivalence, original cells, retained reveal memory, one obtain and fifth-attempt resolution |
| Upgrade empty/one/many with duplicate kinds | Current instance filtering/order, exact prefix, one source seed, Tiny House's second bottled check without a second upgrade |
| Relic post-shuffle removal | Counterexample above must not be replaced by survivor-only shuffle |
| Relic N rejection / E depletion / boss E | Front→back on rejection, promoted-tier front, boss front; consumed entries remain absent on skip |
| Linked pool evidence | Posterior across all five pools and retained source state; no product of marginal tables unless selected/proved |
| Input order change | Source-host order versus proposed canonical order is versioned/declared, never silently sorted |

**Remaining decisions/proofs:** choose the semantic measure and independence
boundaries; declare relic input order; supply producer laws/joint contexts when
unconditional boards or complete source-specific offers are wanted; and evaluate
any particular actual root/profile/history ensemble if that is the fidelity target.
The finite conditional kernels above do not depend on those evaluations to be
well-defined, but they do not choose their inputs. Efficient future exact sampling,
full effect/callback closure, independent review, whole-spec approval, plan review
and implementation-start authorization remain separate. No product builds,
dependencies, benchmarks, training or original-runtime probes were performed.
