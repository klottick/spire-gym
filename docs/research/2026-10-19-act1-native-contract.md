# Act 1 native contract: concrete candidate for whole-spec review

Date: 2026-10-19. Bead: `spire-gli.2.3.6`.

**CANDIDATE, not approved API, complete mechanic closure, implementation plan or
implementation authorization.** This document selects reviewable representations
instead of leaving field widths, carrier semantics and ownership implicit. It
does not select unresolved probability laws or profile/time fidelity policies.
No agent behavior, simulator, helper library or learner is implemented here.

## 1. Status, scope and baseline

Labels throughout are normative evidence/decision distinctions:

| Label | Meaning |
|---|---|
| **A** | Existing user-approved architecture; retained, not a new approval request. |
| **F** | Retained static/source fact from authored research; not a new source inspection or runtime observation. |
| **P** | Concrete proposed contract/representation in this document, awaiting whole-spec review. |
| **D** | Genuine unresolved dependency; no runnable default or implied approval. |

Unlabelled requirements in sections 3–10 are **P**, constrained by **A**. An F
claim applies only to its stated field and pinned evidence scope, never an entire
row by association. Standard fully unlocked, unmodded, Act-4-enabled Ironclad A0
Act 1 includes every legal Shard/Note route, other-color mechanics, death, skips,
deferred keys and resolved Act1Complete/Act2Ready. It does not include other
character starts, exclusive resources or their home events merely through cards.

**A:** `spire-core` owns rules, live run/combat resources, legality, automatic
resolution and RNG. `spire-env` owns episodes, public projection, candidate mapping,
batching/replay/fault coordination. `spire-python` only transports owned numeric
data and packages delivery. Policy code chooses among legal rows; it cannot
provide game callbacks, RNG, repairs or retries. One detailed combat engine serves
standalone combat and runs, with an explicit resource context.

Baseline is commit `2f1b19cdb168bef16b2112a0fb5c5e9cc47c9a98`. Actual committed
[combat model](../../crates/spire-core/src/combat/model.rs),
[engine](../../crates/spire-core/src/combat/engine.rs),
[view](../../crates/spire-core/src/combat/view.rs) and
[snapshot](../../crates/spire-core/src/combat/snapshot.rs) describe narrow supplied
combat (Strike/Defend/Bash, one supported enemy, typed handles, schema-2 snapshots).
Its scenario bounds are **not run bounds**. The committed env crate is scaffolding
and binding exposes build metadata, not the proposed packed run API. Preserve
these implemented entry points and the separately retained combat-env design pins;
do not claim the latter transport already exists on main.

**A, compatibility pins:** `spire-lab-rng-hmac-sha256-v1`, snapshot **2**,
`spire-lab-combat-reference-v2`, `spire-lab-env-seed-sha256-v1`,
`spire-lab-combat-observation-v1`, fingerprint
`8fd66d212259d2bdd27139988a772e1a6c223f8d65524f38c74174a0d0843357`,
and `hmac="=0.12.1"` remain unchanged. Neither run content nor new domains are
silently cast into old enums, widened old arrays or old snapshot payloads.

Read together with the seven merged research reports:
[native draft](2026-10-18-act1-native-contract-closure.md),
[map/RNG](2026-10-18-act1-rng-map-closure.md),
[execution](2026-10-18-act1-execution-closure.md),
[continuations](2026-10-18-act1-continuations-closure.md),
[relics](2026-10-18-act1-relics-closure.md),
[runtime boundary](2026-10-18-act1-runtime-closure.md), and
[Acts 2/3 boundary](2026-10-18-ironclad-a0-acts23-research.md).
[RNG-001](../intentional-deviations.md) remains authoritative. Preserved read-only
architecture/packed/profile/generation drafts informed this independent synthesis;
they are not required unpublished hyperlinks. Source facts refer to owned JAR
SHA256 `cfad868ac8d65a88e71a0bf096fb09f78811e553effe0787c5309a655e081673`;
upstream build/channel and original-runtime fidelity remain unestablished.

## 2. Review decisions and exact blockers

| ID | Class | Concrete disposition |
|---|---|---|
| NC01 | A | One run resource authority; same detailed combat; Rust-only gameplay. |
| NC02 | P | Ownership/continuation records in section 3; no universal transaction or FIFO scheduler. |
| NC03 | P | Finite site registry and fail-closed law records in section 4. |
| NC04 | P/D | Numeric candidate RNG namespaces in section 5; **no** cross-event independence approved by their existence. Required joint laws remain D. |
| NC05 | P | All-int64 fixed-width rows, aligned knowledge bits, packed variable lengths and typed local refs in section 6. |
| NC06 | P | Stable signed-64 candidate-index carrier, staged selections and exact preflight in section 7. No dense carrier-sized mask/head. |
| NC07 | P | Separate candidate observation/action/catalog/seed/replay/task identities, descriptor freeze rule in section 8. |
| NC08 | A/P | From-start replay, manual single reset and vector same-step replacement; concrete identities/delivery rows in sections 8–9. |
| NC09 | A/P | Invalid whole batch rejects before dispatch; runtime fault quiesces without rollback, concrete delivery/fault lifecycle in section 10. |
| NC10 | D | Profile lane supplies validated creation/output schemas, staged/committed Note semantics, discovery/default choices and time policy. This document defines attachment ownership, not their contents or defaults. |
| NC11 | D | Execution/continuation/relic integration supplies full per-site callbacks, cancellation, prices, rescue and resolved successor-ready predicates. Typed storage is not proof of those laws. |
| NC12 | D | Complete content dictionaries, independent caller/site reconciliation and per-field public-property inventory must close before descriptor/content freeze. Missing legal entries block completeness, never filter production. |

Review may accept NC02–NC09 as representations while NC04/NC10–NC12 remain open.
That is not approval to initialize a partial production catalog or start work.

## 3. State, identities and continuation ownership

### 3.1 Single mutable authority

| Record | Sole owned contents | Access/lifetime invariant |
|---|---|---|
| `RunState` | Creation snapshot; `RunResources`; map/progression; generation histories/pools; active room; combat substate; continuation arena; public knowledge; RNG authority | Owned by exactly one logical slot while installed. No resource reconciliation at fight end. |
| `RunResources` | HP/max HP, gold, permanent deck instances and modifications, relic instances/order/counters, potion slots/contents, keys, run counters | Combat/effects borrow this authority exclusively for mutation; room and combat cannot both hold independently writable versions. |
| `CombatState` | Combat instances/piles, enemies, powers/orbs/stance, turn/history, entered/current work and distinct pending containers, combat knowledge | Borrows resources through run context; owns only temporary state. Standalone uses an explicit owned resource context, not an inferred character. |
| `RoomState` | Source room identity, visible objects/offers, source-local progress and pending children | Event combat keeps its event parent; no forced ordinary reward/map routing. |
| `KnowledgeState` | Legally revealed values/positions/relationships and public remembered facts | Does not clone hidden state and mask it later in Python. |
| `Decision` | Public projection, exact core menu mapping, decision generation and private token | Immutable until accepted transition/reset; reads neither reroll nor rotate token. |
| `CreationSnapshot` | Detached validated profile, rules/content/clock IDs, root/partition and logical identity | Immutable; no caller alias, filesystem account or arbitrary supplied run checkpoint. |

Private handles are disjoint typed tuples `(run_key, kind, creation_ordinal)`;
`kind=1 permanent card, 2 combat card, 3 enemy, 4 relic, 5 offer, 6 continuation,
7 effect invocation, 8 combat, 9 room`. Ordinals are checked unsigned 64-bit,
never reused in a run. Exhaustion faults administratively. These are not public
features or RNG scope inputs by allocation accident. Public references are rebuilt
slot-local rows at each decision. Permanent-owner links on combat cards identify
the exact eligible growth owner, not whichever same-kind card is first in deck.

Copy identity is always new; `CopyRecord` carries exact source handle, source site,
capture checkpoint, source-defined captured attributes and optional permanent
growth owner. Copying a temporary card does not automatically inherit that owner.
**D:** the execution/source law decides which attributes and owner relation survive
each copy; no default stat-equivalent/empty provenance is inferred. Captured scalar
values are owned values; truly live lookups use validated same-run handles.

### 3.2 Phases and frames

Private phase codes: `1 Initializing, 2 Resolving, 3 Decision, 4 Dead,
5 Act2Ready, 6 Faulted`. `Decision` contains public context kind from section 6;
`Resolving` is not externally published as a fake empty live menu. An accepted
action advances automatic work to a genuine choice or real terminal boundary.
No action/turn/automatic-work budget is a game rule. A nonterminating legal path
cannot be converted to defeat/truncation to keep a batch moving.

`ContinuationFrame` owns: handle, parent handle or none, source `SiteKey`, source
invocation identity, frame tag, exact resume checkpoint, captured parameters,
already-committed checkpoint set, child handles, source-defined child-return rule,
selection progress, play context where applicable, and cancellation disposition.
Checkpoint IDs are authored per site; they are not wall-clock ticks. A once-only
checkpoint key is `(invocation, checkpoint, source-defined iteration ordinal)`.
There is no undo log: completed checkpoint bookkeeping prevents duplicate payment,
grant, history update or hook on resume, not rollback of real effects.

| Frame tag | Required captured data | Source-specific return target |
|---|---|---|
| 1 Neow | Chosen category/local option, drawback/reward checkpoints | Remaining blessing activation, then map when resolved |
| 2 Shop | Stock visit/version, purchased object, payment/restock checkpoint | Same merchant state, not new stock generation |
| 3 Event | Event branch/progress, prior room and source return checkpoint | That event's next state/direct-map route |
| 4 Combat | Encounter/room parent, resource context, outcome/escape classification | Source-specific reward or event return |
| 5 Reward | Ordered reward records, links, generated/claimed/removed flags | Original reward owner/remaining records |
| 6 Relic obtain | Offered/claimed relic, equip progress, nested children | Claiming reward/shop/Neow/boss parent |
| 7 Deck selection | Eligible identities, min/max, selected sequence, captured prices/amounts | Exact remove/transform/copy/upgrade site |
| 8 Combat selection | Pile eligibility, selected order, optionality and effect checkpoint | Suspended entered card/action, not next turn |
| 9 Potion use | Consumed slot, use/hooks/destroy progress, proposed outputs | Source's modal/combat/room parent |
| 10 Chest/key | Reward links, hook progress, deferred key effect | Chest/reward owner after actual key commit |
| 11 Campfire | Constructed/filtered options, appended Recall, selected child | Source room-complete boundary |
| 12 Boss handoff | Combat reward, boss chest/relic take-or-skip and nested progress | Successor setup checkpoint/Act2Ready |
| 13 Play | Physical card, origin/payment/copy/X context, target and cleanup state | Exact card-use/queue owner |
| 14 Profile transition | Owned profile-lane attachment and its commit checkpoint | Source transition, not implicit next reset |

Parent/child is an ownership graph, not a promise that every source behaves like
a LIFO UI stack. Only one frame owns a given child; source return metadata can
resume a retained ancestor or direct-map checkpoint. Frames cannot refer to retired
runs. An orphan child, cycle or double resume is a content/invariant fault.

`PlayContext` carries origin (`1 hand, 2 autoplay-existing, 3 repeat,
4 source-special`), physical/source/copy handles, target, captured energy,
captured X plus validity, free-to-play-once, ignore-total, autoplay, purge,
history eligibility and source payment/cleanup checkpoints. Boolean fields are
explicit, never inferred from origin alone. **F:** Double Tap, Burst, Amplify,
Duplication and Echo repeats copy energy-on-use while explicitly enabling both
ignore-total and autoplay. **D:** complete legality/payment/cost/callback laws,
including non-hand origins, remain source-specific; flags are not blanket bypasses.

### 3.3 Durable mutation and ending conditions

**A:** each qualifying effect mutates resources at resolution, including non-final
kill growth. No end-combat transaction, lethal rollback, universal queue drain or
automatic heal rescue is introduced. Current entered work, pending actions,
pre-turn work, card/monster queues and deferred effects remain separate owners.
Pending does not imply execution; clearing pending does not unwind an entered call.

**F:** reward offers can change history before claim; relic offers consume pool
entries even when skipped; Wheel gold changes at reveal; key ownership changes
at its deferred effect, not offer/Recall construction. Campfire builds/filters
initial options, appends eligible Recall, then scans the final menu. Ssserpent Head
is a Face Trader face, not The Ssssserpent's reward.

**P:** `Act2Ready` requires source-approved successor transition completed, no
unresolved strategic child or required durable handoff effect, and a valid retained
native successor-entry state. This does **not** require every unrelated visual list
to be empty. All three keys are optional; a skipped boss relic is valid. Reward is
`+1` only then, `-1` at actual death, otherwise `0`; endings terminate, never
truncate. A boss lethal hit, room COMPLETE flag or empty visible menu alone cannot
establish success. **D:** exact per-source handoff/terminal arbitration and
Reaper timer outcome are not supplied by this representation.

## 4. Finite effect-site policy registry

### 4.1 Key, record and coverage rule

Proposed `SiteKey=(namespace, owner_code, operation, variant)` uses four unsigned
32-bit integers internally; transport codes fit nonnegative int64. Namespaces:
`1 card, 2 relic, 3 potion, 4 enemy, 5 event, 6 run-system, 7 acquisition,
8 power`.
Owner code is the immutable content dictionary code (section 8); system owners
are listed below. Operation codes are explicit, not inferred from method names:

`1 use, 2 actual-draw, 3 actual-exhaust, 4 start-turn, 5 end-turn,
6 pre-draw, 7 equip, 8 reward-build, 9 claim, 10 purchase, 11 transform,
12 remove, 13 copy, 14 insert, 15 key-commit, 16 selection,
17 resume, 18 transition, 19 damage, 20 heal, 21 lifecycle,
22 pool-select, 23 initialize, 24 restock, 25 preview`.

Variant is a frozen local ordinal starting at 0, splitting distinct conditional
producer/hook sites under an owner/operation. It is **not** an invocation ordinal.
The finite candidate registry below is closed to runtime extension: neither source
strings, guessed wildcard handlers nor arbitrary owner/operation products create
entries. Nested production changes to the actual inner site. Repeated invocation
uses the same site with new semantic invocation identity, not a new policy.

Each entry has all of: `SiteKey`, request variant/parameter bounds, pool basis,
eligible-ID-set expression, exact selection kernel, duplicate/retry rule, history
reads/writes and checkpoints, randomness/cursor relation, construction/copy/cost
law, destination/capacity law, payment/commit/cancel/terminal law, continuation
tag/checkpoints, field evidence, dependencies and readiness. Basis codes:
`1 CurrentCharacter, 2 Colorless, 3 ExplicitLibrarySubset, 4 FixedIdentity,
5 ExistingInstance, 6 StoredProfile, 7 CurrentCharacterPotions,
8 SourceAcquisition`. No `Unresolved` basis is runnable.

Every law field is exactly `Fact(value, locator, grade)`, `Proposed(value,
decision_id)`, `Partial(known, exact_open_fields, locators)` or
`Open(question, dependency_id)`. Evidence grade is `StaticSource`,
`BytecodeConfirmed`, `IndependentModel`, `OriginalClass` or `OriginalRuntime`;
the last two receive no new evidence here. Readiness is `1 incomplete,
2 review-required, 3 review-closed`. No field inherits Fact from row readiness.
All registry entries in this candidate have readiness1, because at least one
required NC04/NC11/NC12 field is unresolved. This is an explicit non-runnable
candidate, not an implementation-stage readiness claim. Any required field not
given a value below is `Open(exact source-qualified field, NC11 or NC12)`;
every RNG binding is `Proposed(section5, NC04)` with its declared D obligations.
These are authoring dispositions, not runtime defaults. Readiness3 eventually
requires closed facts or explicitly approved fidelity choices for every field.

Catalog acceptance compares **independent** complete roster dispositions and
caller/override/effect-site inventory against this registry. In particular it must
detect a producer omitted from both a developer's generator flag and its registry.
Reachable missing/Partial/Open required laws prevent content initialization before
entropy, identities or episodes are allocated. A lookup missing after initialization
is an administrative content fault, never default Ironclad/colorless sampling,
unsupported-outcome reroll, skip or defeat. The candidate is intentionally not a
claim that the independent inventory or every law is already closed (NC12).

### 4.2 Direct card registry — all 39 inventoried producer kinds

Each row is a separate site; grouped cells expand to the explicitly named owners,
not a catch-all. `use/0` means operation 1, variant 0. Names are human dictionary
labels, not localized runtime string dispatch. F below is retained from the reviewed
generation findings, not new source verification. Every entry still needs NC11
for full callback/cancellation closure.

| Owners | Operation/variant | Basis and concrete policy assignment |
|---|---|---|
| Infernal Blade | use/0 | 1; uniform eligible Attack; turn discount; hand action |
| Distraction | use/0 | 1; uniform eligible Skill; turn discount; hand action |
| White Noise | use/0 | 1; uniform eligible Power; turn discount; hand action |
| Creative AI | start-turn/0 | 1; Power per intensity; normal cost, one intensity block |
| Hello World | start-turn/0 | 1; live Common pool, not HEALING-filter selector; suppress when basically dead |
| Discovery | use/0 | 1; three distinct eligible IDs, mandatory one; turn discount |
| Chrysalis | use/0 | 1; 3/5 Skill identities sampled before any child insertion; combat-base zero |
| Metamorphosis | use/0 | 1; 3/5 Attacks, same ordering distinction, separate policy |
| Foreign Influence | use/0 | 3; 55/30/15 rarity-first Attack proposals, duplicate restarts entire proposal; optional zero/one |
| Jack of All Trades | use/0 | 2; 1/2 independent-with-replacement identities; normal cost |
| Magnetism | start-turn/0 | 2; separate sample per intensity, normal cost; basically-dead guard |
| Transmutation | use/0 | 2; effective-X children before placements, source+ child upgrade then turn discount |
| Wild Strike | use/0 | 4; Wound → random draw insertion |
| Reckless Charge | use/0 | 4; Dazed → random draw insertion |
| Power Through | use/0 | 4; two Wounds → hand before source Block |
| Immolate; Overclock | use/0 | 4; each has its own Burn → discard site; source attack/draw precedes it |
| TURBO | use/0 | 4; Void → discard after energy; creation is not actual draw |
| Blade Dance | use/0 | 4; 3/4 Shivs → hand |
| Cloak and Dagger | use/0 | 4; 1/2 Shivs after Block |
| Infinite Blades | start-turn/0 | 4; Shiv, not source-upgrade-derived Shiv+ |
| Storm of Steel | use/0 | 4; source-defined discarded-set count, Shiv/Shiv+; reaction-created cards not silently included |
| Battle Hymn | start-turn/0 | 4; Smite → hand |
| Carve Reality; Deceive Reality | use/0 | 4; separate Smite/Safety hand sites after attack/Block |
| Evaluate; Pray | use/0 | 4; separate Insight → draw insertion sites |
| Study | end-turn/0 | 4; Insight → draw insertion, not start-turn hand |
| Reach Heaven | use/0 | 4; Through Violence → draw insertion |
| Collect | start-turn/0 | 4; captured delayed duration, Miracle+ each qualifying turn |
| Deus Ex Machina | actual-draw/0 | 4; 2/3 Miracles then source Exhaust; not triggered by ordinary insertion |
| Alpha; Beta | use/0 | 4; separate Beta/Omega → draw insertion sites |
| Conjure Blade | use/0 | 4; Expunger with captured X payload before placement/upgrades; zero payload still creates |
| Anger | use/0 | 5; new copy → discard after attack |
| Dual Wield | copy/0 | 5; selected Attack/Power, 1/2 hand copies |
| Endless Agony | actual-draw/0 | 5; one hand copy; copy insertion is not recursive actual draw |
| Nightmare | copy/0 | 5; capture/next-turn three-copy continuation, NC11 exact copied-state dependency |
| Alchemize | use/0 | 7; limited rarity-first potion proposal during use before later belt/Sozu check |

**F, exact standard-Ironclad support rules:** temporary colored selectors take
ordinary current-character non-basic Common/Uncommon/Rare IDs minus HEALING and
apply the requested type. Ironclad has 28 Attacks, 28 Skills, 14 Powers (70 total).
Hello World has 20 Commons. Colorless has 34 ordinary non-HEALING IDs. HEALING
exclusions are Feed/Reaper; Alchemize; Self Repair; Lesson Learned/Wish; Bandage Up
in the respective Ironclad/Silent/Defect/Watcher/colorless bases. Hand of Greed is
eligible. Basics, Status, Curse and SPECIAL never enter these selectors; Expunger
is fixed-created, not random support. Own-source generation is allowed when it
passes the actual filter, not categorically excluded. **D NC12:** the final numeric
ID-set manifest is not supplied by counts; catalog freeze must enumerate/check it.

**F, concrete kernels:** ordinary single draws are equal-ID; ordinary three-offers
are ordered samples without replacement by kind ID, with no reward-pity mutation.
Foreign Influence has Common/Uncommon/Rare support sizes 43/48/15 and initial
identity mass `55/(100*43)`, `30/(100*48)`, `15/(100*15)` respectively. Given
accepted IDs with total original proposal mass `s`, next distinct ID `x` has
`p(x)/(1-s)`. This is not fixed-rarity retry or equal-color sampling. The support
includes ordinary colorless Attacks and is not selected by printed producer color.

**F, construction distinctions:** positive turn discounts do not rewrite negative
X costs. Ordinary hand-action Master Reality has two real upgrade checkpoints;
overflow discard can retain only the earlier one. Foreign Influence has no earlier
Master Reality checkpoint: one actual hand upgrade, none on that overflow input.
Draw-pile actions here each carry one card; Searing Blow can receive two upgrades.
The animation clone is not the stored discard instance. Eviscerate receiver cost
mutation, Blood for Blood prior HP-loss count and source-specific copied costs
cannot be replaced by pristine template stats. These facts do not close all costs.

**F, insertion:** for current nonempty draw size `n`, random insertion has `n`
positions, each mass `1/n`, excluding the new-top gap; empty insertion is
deterministic. In public known order, position 0 means next-to-draw; transport
orientation is not the private container's bottom-first order. Selection and
placement use distinct registered sites/cursor relationships, not n+1 ideal gaps.

Finite child-site expansion under the same content owner completes the proposed
selection-versus-placement distinction (P keys, retained F route distinctions):

* `insert/0` for Infernal Blade, Distraction, White Noise, Creative AI, Hello
  World, Jack of All Trades, Magnetism, Transmutation: ordinary hand-action child.
* `insert/0` for Discovery, Foreign Influence: selected-offer copy/placement child,
  with their different upgrade/discount/optionality laws, not a fresh selection.
* `insert/0` for Chrysalis, Metamorphosis, Wild Strike, Reckless Charge, Evaluate,
  Pray, Study, Reach Heaven, Alpha, Beta, Conjure Blade: source-specific draw-pile
  child. Invocations/iterations distinguish multiple outputs under one site.
* `insert/0` for Power Through, Blade Dance, Cloak and Dagger, Infinite Blades,
  Storm of Steel, Battle Hymn, Carve Reality, Deceive Reality, Collect, Deus Ex
  Machina: fixed hand-placement children; counts/state remain source-bound.
* `insert/0` for Immolate, Overclock, TURBO, Anger: discard-placement children;
  Anger preserves its separate existing-instance copy law.
* `insert/0` for Dual Wield, Endless Agony, Nightmare: existing-instance hand-copy
  children; Nightmare additionally has `start-turn/0` for its delayed materialization.
* Alchemize's obtain child is acquisition namespace7/owner5 `claim/3`, with
  provenance back to card use/0; it has no card-placement child.

Similarly the explicit relic child keys are Dead Branch and Toolbox `insert/0`,
Mark of Pain `insert/0`, Dolly's Mirror `claim/0`; the four card-producing potions
each have `insert/0`. Entropic Brew's deferred obtains use acquisition7/5 claim/3
with combat/outside provenance. These are individual finite registered entries,
not a rule that any content owner may request insertion. Child records retain
exact source parameters and NC11 cancellation/callback dependencies. Hand overflow
is a branch of the same placement invocation, not an invented reroll/new producer.

### 4.3 Relic, potion and nested generation registry

| Owner/site | Operation/variant | Assigned route and required distinction |
|---|---|---|
| Dead Branch | actual-exhaust/0 | Basis1, uniform eligible untyped70; sample at relic callback before queued placement; basically-dead guard |
| Toolbox | pre-draw/0 | Basis2, three distinct34; mandatory one, no source discount; not necessarily empty opening hand |
| Dolly's Mirror | copy/0 | Basis5, permanent selected-instance copy and actual obtain hooks |
| Astrolabe | transform/0 | Basis8, selected permanent inputs then source upgrade/acquisition continuation |
| Pandora's Box | transform/0 | Basis8, actual starter Strike/Defend predicates, not names or every Basic |
| Calling Bell | equip/0; equip/1 | Separate fixed Bell curse obtain and ordered three-relic acquisition sites |
| Mark of Pain | pre-draw/0 | Basis4, two Wounds combat creation, not permanent deck changes |
| Cursed Key | lifecycle/0 | Basis8, non-boss chest random curse route, exact prevention/commit |
| Cauldron | equip/0 | Basis7, five flat-ID potion offers/reward continuation, not Alchemize law |
| Tiny House | equip/0; equip/1; equip/2 | Separate card reward, potion and instance-upgrade sites; ordering/source RNG retained |
| Orrery | equip/0 | Five real card-reward children, own reward history/skip boundaries |
| Dream Catcher | lifecycle/0 | Rest-triggered card reward child |
| Prayer Wheel; Question Card; Busted Crown; Prismatic Shard | reward-build/0 | Four distinct reward-modifier sites; not arbitrary direct generators |
| The Courier | restock/0; restock/1; restock/2 | Separate purchased card/relic/potion slot re-proposals, not reward law |
| Attack Potion; Skill Potion; Power Potion | use/0 | Three separate Basis1 typed, distinct-ID offers, optional choice, Bark copies selected identity twice |
| Colorless Potion | use/0 | Basis2 distinct34; mandatory choice in pinned bytes, unlike typed potions |
| Entropic Brew | use/0; use/1 | Combat limited versus outside unrestricted potion proposals; capacity-many attempts, not vacancy-many |
| Liquid Memories; Elixir; Gambler's Brew; Distilled Chaos | selection/0 | Four separate movement/Exhaust/discard/autoplay sites; nested producer switches to its own SiteKey |
| Five repeat powers | lifecycle/0 | One entry each for Double Tap/Burst/Amplify/Duplication/Echo; play-origin context, not independent random card creation |
| Sentries; Acid slimes; Slime Boss; Hexaghost | lifecycle/0 | Separate enemy fixed Dazed/Slimed/Burn sites; split/upgrade/count/action variants must be reconciled under NC12 |

**F:** Ironclad potion base has 33 IDs (17/9/7 by rarity). Alchemize/combat Brew
exclude Fruit Juice: 17/9/6 with per-ID mass `65/(100*17)`, `25/(100*9)`,
`10/(100*6)`. Outside Brew uses 17/9/7 and corresponding rare denominator7.
Rarity is retained across identity/restriction rejection. Brew can generate itself;
no nine other-character-exclusive potions enter via Shard/Alchemize. Combat Brew
samples capacity-many before obtain actions even under Sozu; outside Brew under
Sozu samples none. Use, relic hooks and consumed-slot destruction precede deferred
obtains; successful obtains use the live first empty slot. Bark does not double
Brew's attempts. **D:** terminal/callback integration remains NC11.

The following expanded records make field disposition explicit for three
discriminating sites (P record structure; F values retained from generation
research; remaining fields are D, not guesses):

| Field | Infernal Blade use/0 | Foreign Influence use/0 | Alchemize use/0 |
|---|---|---|---|
| Request | No caller-supplied character/pool; source play context | No caller pool; source upgrade/choice context | No caller potion pool; source play context |
| Basis/support | F CurrentCharacter, ordinary Attack minus HEALING; Ironclad28 | F registered multi-color ordinary Attacks minus HEALING; 43/48/15 | F CurrentCharacterPotions limited, Ironclad17/9/6 |
| Selection | F uniform identity | F rarity55/30/15 then equal identity in tier | F rarity65/25/10 then identity/restriction rejection retaining tier |
| Duplicates/history | F one selection; no reward-pity change | F reject accepted kind; re-propose whole rarity/identity; no reward-pity change | F no within-offer exclusion or potion-pity change; limited helper has discarded initial proposal activity |
| Draw checkpoint | F card use, before queued hand placement | F displayed offer generation; retain exact offer through selection | F card use, before queued Sozu/capacity check |
| Construction | F fresh combat card; turn discount, source+ only payment; destination-dependent Master Reality | F selected stat-copy, source+ turn discount, no action-level Master Reality, Eviscerate copy caveat | F fresh potion proposal; source+ changes payment only |
| Destination | F hand-action, full-hand discard with actual stored-instance rules | F selected hand/overflow, optional none | F later obtain fills first live empty belt slot or fails without replacement |
| Commit/blocked | F sampled even if hand full; no permanent obtain; D terminal/callback survival NC11 | F preview not obtain; D terminal/callback/cancel NC11 | F Sozu/full belt prevents obtain, not prior proposal; D terminal/callback survival NC11 |
| Continuation | P Play frame13; D exact post-child cleanup checkpoint NC11 | P CombatSelection frame8 then Play13; D exact interruption NC11 | P Play13 plus potion-obtain child; D exact interruption NC11 |
| RNG | P role8, conditional binding required NC04 | P role8; entire-offer joint kernel preserved, NC04 | P role5 or reviewed role8 binding; source boundary alone cannot settle sharing, NC04 |
| Evidence/readiness | Retained static generation selector/construction findings; readiness1 | Retained static Foreign Influence/selected-copy findings; readiness1 | Retained static potion selector/obtain findings; readiness1 |

The discarded limited-potion proposal is distinguished from a gameplay selection
law: RNG-001 does not require original incidental call-count parity, but any native
choice to omit it must preserve the accepted law and relevant future dependence.
Likewise incidental unused Discovery updates never regenerate a suspended offer.

### 4.4 Run/acquisition/site closure registry

System-owner codes below are frozen candidate IDs, separate from content owners.
Each named variant is an individual entry, not an independent RNG stream.

| Namespace/owner | Operation and variants | Contract/law status |
|---|---|---|
| 6/1 map | initialize/0 topology; initialize/1 room buckets; initialize/2 burning marker; initialize/3 buff | F ordered shared-map mechanism; D initialized zero-elite image/empty branch and exact measure |
| 6/2 encounter | pool-select/0 weak; /1 first-strong; /2 later-strong; /3 elite; /4 boss; initialize/0 composition; /1 HP; /2 opening | History/composition/setup order required; not independent encounter-name draws |
| 6/3 event | pool-select/0 ordinary-vs-shrine; /1 eligible event; /2 eligible shrine; /3 one-time list | D exact filtering/empty/history conditional law; no uniform-over25 shortcut |
| 6/4 relic pools | initialize/0 five ordered pools; pool-select/0 normal; /1 end; /2 screenless; /3 noncampfire | F skip consumption/rejection/depletion law; D full five-pool joint measure |
| 6/5 combat | initialize/0 setup; lifecycle/0 draw; /1 discard; /2 Exhaust; /3 shuffle; /4 start; /5 end; /6 victory; /7 death; /8 escape | Same engine/explicit context; D full caller overrides, entered/pending/cancel laws |
| 6/6 campfire | initialize/0 menu; selection/0 Rest; /1 Smith; /2 Dig; /3 Lift; /4 purge; /5 Recall | F initial-filter→Recall-append→scan; D each option's full continuation |
| 6/7 chest | initialize/0 ordinary; /1 boss; reward-build/0 rewards; key-commit/0 sapphire; /1 emerald; /2 ruby | F deferred key/link distinctions; D full handoff/terminal arbitration |
| 6/8 handoff | transition/0 boss rewards; /1 boss relic; /2 successor setup | +1 only after resolved Act2Ready; no Act2 gameplay implied |
| 7/1 card reward | reward-build/0 ordinary; /1 elite; /2 boss; /3 event combat; /4 Dream Catcher; /5 Orrery; /6 Tiny House; /7 Neow ordinary; /8 Neow rare; /9 Neow colorless | Separate caller records; F ordinary72/qualifying Shard320, rarity-before-identity retry; exact modifiers remain field-qualified |
| 7/2 merchant | initialize/0 colored; /1 colorless; /2 relic; /3 potion; purchase/0 card; /1 relic; /2 potion; /3 purge | F whole-proposal duplicate retry/common-Power fallthrough; D complete prices/returns |
| 7/3 transform | transform/0 Neow; /1 Living Wall; /2 Transmogrifier; /3 Astrolabe; /4 Pandora | F flat colored current-character72 or ordinary colorless35, incoming-ID exclusion; curse/removal branches distinct |
| 7/4 potion offer | reward-build/0 combat drop; /1 Lab; /2 Woman; /3 Neow; /4 Cauldron; /5 Tiny House; /6 merchant; /7 Courier | F combat/shop rarity-first, Lab/Woman/Neow/Cauldron flat; exact appearance/history and Tiny House RNG not generalized |
| 7/5 obtain/remove | claim/0 effect card; /1 direct Note; /2 relic; /3 potion; remove/0 object; /1 string; /2 iterator; preview/0 card | F Omamori effect-path exception and distinct removal hooks; not one universal obtain/remove pipeline |
| 7/6 selection | selection/0 ordered; /1 unordered; /2 bounded board; /3 fixed offer | Must bind exact parent source, count/commit/skip and revealed knowledge; no common fallback |

Finite event-owner roster: Big Fish, Dead Adventurer, Golden Idol, Hypnotizing
Colored Mushrooms, Living Wall, Scrap Ooze, Shining Light, The Cleric, The
Ssssserpent, Wing Statue, World of Goop; Golden Shrine, Match and Keep, Purifier,
Transmogrifier, Upgrade Shrine, Wheel of Change; Ominous Forge, Bonfire Spirits,
Face Trader, The Divine Fountain, Lab, A Note For Yourself, We Meet Again!, The
Woman in Blue. Each receives `selection/0` (local branch dispatch), `resume/0`
(child return) and `transition/0` (leave/terminal) registry records. Additional
explicit event records are Note `claim/0 incoming, remove/0 outgoing,
transition/1 profile`; Wheel `claim/0 reveal, claim/1 confirmation`; Match and
Keep `initialize/0 board, selection/1 first, selection/2 second, resume/1 pair,
transition/1 cleanup`; Face Trader `pool-select/0 missing faces`; Dead Adventurer
`initialize/0 category order, resume/1 combat`; Mushrooms `resume/1 combat`.

These dispatch records require per-branch effect/checkpoint inventories as field
data; their existence does not certify every branch. Missing enemy action variants,
event conditional sites, primary relic consumers and transitive callbacks are
**D NC11/NC12**, not intentionally omitted legal content. The 141 relic IDs and
32 direct canSpawn overrides are inventory facts, not full primary-mechanic proofs.
All 13 global/site-only relics retain required consumer records: Paper Frog, White
Beast Statue, Calipers, Ginger, Ice Cream, Turnip, Runic Pyramid, Chemical X,
Frozen Eye, Strange Spoon, Golden Idol, Odd Mushroom, Spirit Poop. No absence of a
subclass override licenses an inert implementation.

P consumer SiteKeys for those 13 (all namespace2, actual relic owner; no F claim
of complete primary rule): Paper Frog `damage/0`; White Beast Statue
`reward-build/0`; Calipers `lifecycle/0` (Block carry); Ginger `lifecycle/0`
(Weak application); Ice Cream `lifecycle/0` (energy carry); Turnip
`lifecycle/0` (Frail application); Runic Pyramid `lifecycle/0` (turn-end hand);
Chemical X `use/0` (source-specific X calculation); Frozen Eye `lifecycle/0`
(knowledge projection); Strange Spoon `lifecycle/0` (played-card cleanup);
Golden Idol `reward-build/0`; Odd Mushroom `damage/0`; Spirit Poop
`lifecycle/0` (exact consumer/relevance still Open NC11). Shared consumer invocation
binds that exact registry owner; these slots cannot prove a hook runs or invent
a new source-specific effect where none applies before Act2Ready.

**F, relic selector discriminator:** normal rejection consumes its candidate and
retries end at the **requested tier**; end rejection retries itself. Nonempty boss
uses front in both helpers, other nonempty tiers use front(normal)/back(end).
Depleted Common/Uncommon/Shop promotion calls normal/front for
Uncommon/Rare/Uncommon, including from end. Empty Rare/Boss has its actual
Circlet/Red Circlet branch, not truncation or arbitrary replacement.

## 5. RNG scope proposal without invented stochastic closure

Existing HMAC-v1 derivation/word exhaustion stays unchanged; new registry version
must assign new domains rather than mutate old `combat.shuffle`,
`combat.enemy.hp`, `combat.enemy.ai` arities/meaning. Root, partition and logical
episode identity are already derivation inputs, not duplicated as scope strings.
Candidate namespace prefix is `run.a1.candidate1.`:

| Domain suffix / numeric role | Candidate scope tuple | Proposed draw boundary and retained obligation |
|---|---|---|
| `map` / 1 | `(act)` | Topology then room bucket then marker/buff in ordered generation; D shared-cursor law/empty branch |
| `encounter` / 2 | `(act)` | Queue construction/selection history, not fresh scope per queue entry; D boss/composition relations |
| `relic-pools` / 3 | `(run-pools)` | Five ordered shuffles and subsequent removals/depletion; **one joint-law obligation**, not five approved independent streams |
| `card-acquisition` / 4 | `(act)` | Ordered offer creation across rewards; pity lives in state; D cross-source/cross-event cursor/kernel law |
| `potion-acquisition` / 5 | `(act)` | Appearance and identity at respective source checkpoints; D whether/how these share a cursor |
| `merchant` / 6 | `(act, visit)` | Ordered proposals/retries/restocks; D cross-visit relation and price draws |
| `event` / 7 | `(act)` | Ordered selection and local outcomes; D selection/outcome/board/face/upgrade relationships |
| `combat-generation` / 8 | `(combat)` | Actual inner producer at source boundary, retained offers; D sharing among producer/target/cost roles |
| `combat-placement` / 9 | `(combat)` | Actual insertion/shuffle/exhaust checkpoints; D cross-role relations, no existing combat stream hijack |
| `combat-ai` / 10 | `(combat, enemy-semantic-position)` | Selected-move/history law; D complete setup/summon semantic IDs and new-domain approval |
| `combat-hp` / 11 | `(combat, enemy-semantic-position)` | Actual construction checkpoint, not reorder-all-HP-before-composition |

Scope components are printable ASCII semantic labels with canonical decimal
ordinals, satisfying HMAC-v1's 1–255-byte field rule. `act="1"`,
`run-pools="initial"`; visit/combat ordinals follow source transition order,
not worker allocation. Enemy semantic position includes parent summon/split
invocation and child position where needed; arbitrary handle renaming changes
neither stream assignment nor results. Proposed position string is
`entry:<formation-position>` or `spawn:<semantic-spawn-ordinal>:<child-position>`,
with checked unsigned64 decimal components; no recursively concatenated ancestry
can exceed the field-width rule. A semantic spawn ordinal advances only at actual
source spawn/split checkpoints, never object allocation. **D NC11:** exact checkpoint
order and constructor interleaving still require source closure.

**P:** the table chooses a *candidate naming/granularity proposal*, not independent
invocations. Each site additionally requires `KernelBinding(kernel_id,
conditioning_state, history_transition, correlated_sites, input_measure,
cursor_relationship)`. The binding may prescribe a shared cursor, upfront joint
latent variable or conditional transition kernel across rows/domains; it may also
require rejecting this proposed split. Missing binding is a pre-initialization
catalog error, not IID sampling. A distinct domain/key in a schema is not evidence
two events are semantically unrelated. No per-slot/per-card/per-turn independent
substream is implied. Exact finite mapper biases and relevant dependencies survive.

**D, exact unresolved conditional obligations:** initialized/history-conditioned map
input measure and zero-elite seed image; shuffle pushforwards for missing faces,
duplicate-instance Match-and-Keep boards, selected-instance upgrade prefixes,
and ordered five-relic-pool joint law with owned removals/rejections/depletion;
boss-selection conditioning on discovery/encounter history; cross-event sharing
where relevant to observable futures. A uniform64 input through the exact Java
mapper is an explicit *possible fidelity decision*, not the initialized root-seed
law, not uniform permutations and not automatically approved independent events.
The recorded unequal three-item mapper counts do not solve these larger laws.
No frequencies, supplied legal choices or missing seed witness prove closure.

Fixed identity/copy/context lookup uses no selection word; legitimate placement,
hook or conditional effect randomness still uses its own registered binding.
Out-of-input errors consume none; post-dispatch exhaustion is administrative and
retains cursor provenance, not retry/loss/rollback. Policy sampling is outside the
simulator RNG and never becomes a required agent behavior in this contract.

## 6. Concrete packed observation schema

### 6.1 Scalars, masks, offsets and references

All feature values below are native-endian **signed int64 (64-bit, 8-byte)**;
all `known` bits are int8 (0/1). All arrays are C-contiguous exports owned
independently of live state. `U=2^63-1`; row counts/offsets/indices are `0..U`,
with legal indices strictly less than their segment length. Scalar game integers
use `[-2^63, U]` unless a narrower semantic domain below applies; this transport
range is not a gameplay cap or permission to change source integer/binary32 laws.
Overflow in arithmetic/encoding/allocation faults, never clamps a legal value.
Where source arithmetic has a fixed-width rounding/overflow rule, preserve it
explicitly before transport; no generic int64 arithmetic substitution is implied.

Every table `t` has `values[M_t,W_t]`, `known[M_t,W_t]`, `offsets[B+1]`.
Known bits cover **every column**, eliminating a second partial-column mapping.
Offsets are int64, start0, end`M_t`, nondecreasing, count rows not bytes. Unknown
value is exactly0 with known0; known zero has known1. Inapplicable optional value
is known1 sentinel `-1` where specified. Row existence distinguishes absent
objects from a present but unknown face; no fake padding rows. Empty table keeps
shape `[0,W_t]` and all-zero offsets. All 18 tables always exist.

Exact numeric observation storage is `252*B + 144*(B+1) +
9*sum_t(M_t*W_t)` bytes: 28 fixed cells at9 bytes including masks, 18 offset
arrays, and9 bytes per table cell including masks. Independently owned finals,
metadata/profile attachments, facade containers and necessary input/output copies
add to peak memory; this is accounting, not a throughput or allocation guarantee.

`Ref=(table_id,local_row)` targets the **same slot and same decision export**;
absent is `(0,-1)`, player is `(0,0)` only where ActorRef is allowed. Table0 is
otherwise not addressable. Validate both tag and local bound, not global-buffer
bound. Only the transport accessor computes `offsets[slot]+local_row`. Spans
`(start,count)` address that slot's declared target table with checked addition;
empty is `(0,0)`. Unknown reference has both cells masked0/zero, never a hidden
handle. Private instance IDs, root/seed/cursors/tokens/episode/worker IDs are not
features. Knowledge is applied **before** lengths/order/references are chosen.

Fixed `context[B,12]`, `player[B,16]` have equally shaped int8 known arrays.
Columns below are numbered from zero in the stated order; width is exact.

| Fixed row | Ordered columns |
|---|---|
| context W12 | phase, decision_kind, act, global_floor, current_node_ref, turn, terminal_outcome, parent_context_kind, selected_count, minimum_select, maximum_select, ordered_selection |
| player W16 | character, ascension, hp, max_hp, gold, block, energy, max_energy, draw_per_turn, potion_capacity, ruby, sapphire, emerald, stance, orb_capacity, mantra |

Public phase `1 Neow, 2 Map, 3 Room, 4 Combat, 5 Selection, 6 Dead,
7 Act2Ready`; terminal_outcome `0 none, 1 Death, 2 Act1CompleteAct2Ready`.
Decision/context kinds `0 none, 1 Neow, 2 Map, 3 Event, 4 Shop, 5 Campfire,
6 Chest, 7 Reward, 8 BossRelic, 9 Combat, 10 DeckSelection,
11 CombatSelection, 12 MatchBoard, 13 PotionSelection, 14 Handoff`.
Character `1 Ironclad, 2 Silent, 3 Defect, 4 Watcher` (creation admits only1),
Ascension exactly0; stance `0 none, 1 Neutral, 2 Wrath, 3 Calm, 4 Divinity`.
Bits are0/1; `act/floor/turn` nonnegative with turn0 when no combat; current node
is table1 local row or-1. Inapplicable combat fields are masked0, not invented0.
HP/max HP and persistent resources remain visible at terminal. Selection fields
are masked outside selection; max-select is a source limit, not table capacity.

### 6.2 Table IDs, widths and columns

This layout is a complete **candidate structural schema**. Content-vocabulary
closure is the explicit NC12 dependency, not an unregistered catch-all meaning.
All `*_code`/`*_kind` values reference frozen descriptor dictionaries; no Rust
enum casting. Unspecified boolean fields use0/1. Counts/positions/spans use0..U.

| ID / table | W | Ordered columns (zero-based) |
|---|---:|---|
| 1 nodes | 7 | x, y, room_kind, visited, reachable, burning_marker, boss_code |
| 2 edges | 3 | from_node_ref, to_node_ref, currently_selectable |
| 3 permanent_cards | 9 | card_code, upgrades, misc, bottle_kind, removable, upgradeable, base_cost_kind, base_cost, public_position |
| 4 combat_cards | 12 | card_code, upgrades, misc, zone, position, cost_kind, base_cost, turn_cost, public_owner_ref, retained, exhaust_on_use, ethereal |
| 5 relics | 4 | relic_code, public_position, counter, used |
| 6 potions | 4 | belt_slot, occupied, potion_code, potency |
| 7 actors | 8 | enemy_code, formation_position, hp, max_hp, block, intent_code, intent_damage, intent_hits |
| 8 powers | 5 | owner_table, owner_ref, power_code, amount, public_position |
| 9 orbs | 5 | position, orb_code, passive_amount, evoke_amount, counter |
| 10 offers | 9 | offer_kind, content_code, target_table, target_ref, amount, price_kind, price_amount, status, public_position |
| 11 room_objects | 7 | object_kind, public_position, face_state, content_kind, content_code, object_state, remaining_attempts |
| 12 selection | 5 | source_table, source_ref, selected_ordinal, selected, eligible |
| 13 knowledge | 6 | fact_kind, subject_table, subject_ref, position, value_kind, value |
| 14 properties | 8 | owner_table, owner_ref, property_code, ordinal, value_kind, value, ref_table, ref_row |
| 15 history | 7 | fact_kind, ordinal, subject_kind, subject_code, value_kind, value, visible_floor |
| 16 candidates | 8 | action_kind, source_table, source_ref, target_table, target_ref, option_code, parameter_start, parameter_count |
| 17 candidate_parameters | 6 | parameter_kind, ordinal, value_kind, value, ref_table, ref_row |
| 18 relations | 5 | relation_kind, from_table, from_ref, to_table, to_ref |

Table-specific meanings/bounds:

* Nodes sort `(y,x)` and edges `(from,to)`; coordinates are nonnegative source map
  positions, not a guessed dense map size. `room_kind=0 unknown, 1 monster,
  2 elite, 3 boss, 4 event, 5 shop, 6 rest, 7 treasure, 8 boss treasure`.
  Unknown room/boss identity has known0; visible question mark stays event/question
  category, never pre-reveals its future resolved room. All currently public nodes
  and topology remain present. Burning marker follows legitimate revelation.
* Permanent rows retain visible deck order/multiplicity, all upgrades/scaling and
  bottle kind (`0 none,1 Attack,2 Skill,3 Power`). Cost kind
  `1 ordinary,2 X,3 unplayable`; numeric cost cells are masked for nonordinary.
  Counts/flags and future modifier properties are not compressed by card name.
* Combat zone `1 hand,2 draw,3 discard,4 exhaust,5 in-play,6 other-public`.
  Hand order is visible; known draw position0 is next draw. Unknown-order draw
  inventory uses canonical public semantic multiset order with position masked,
  **not** private pile order. Include only card identities/counts actually known;
  unknown draw facts use knowledge aggregate counts, never fake known identities.
  Public-owner-ref targets table3 only when that relation is legitimately known;
  otherwise mask it. No private generated/copy ordinal leaks.
* Relic list order is public retained order, including rule-significant counters;
  hidden counters are masked (and cannot leak through properties). Potions contain
  one row per real belt slot, including known empty occupied0, potion_code-1 and
  masked potency. Slot capacity is resource state, not a padded machine maximum.
* Actors preserve visible formation order; dead/escaped visible actors persist only
  for their legitimate visibility lifetime. Owner refs for powers are ActorRefs:
  `(0,0)` player or `(7,r)` enemy. Runic Dome masks intent **and reconstructive
  intent damage/hits/properties**; no next-selected-move or private AI history leaks.
  Powers preserve visible list order, since tied callback priority can matter.
  Orbs are player slots, source-visible order; no off-color card grants free slots.
* Offer kind `1 card,2 relic,3 potion,4 gold,5 key,6 HP,7 maxHP,8 removal,
  9 upgrade,10 transform,11 source-option`; target is an optional public instance.
  Price kind `0 none,1 gold,2 HP-loss,3 damage,4 maxHP,5 source-composite`;
  exact multi-resource prices attach properties, not one invented scalar.
  Status `1 offered,2 selected,3 claimed,4 skipped,5 removed,6 disabled` describes
  visible menu records, not ownership. Disabled offers can be visible but have no
  candidate. Reward links use table18, never implicit adjacency.
* Room object kind `1 board-position,2 event-option,3 campfire-option,
  4 chest,5 reward-container,6 source-public-state`; face state
  `0 not-applicable,1 face-down,2 exposed,3 remembered-hidden,4 removed`.
  A board row exists for each visible position even when content cells are masked.
  Remembered identity is exposed only if legitimate knowledge retains it.
* Selection rows refer to public eligible/selected objects; selected order is
  retained, unselected ordinal-1. Order affects Forethought+ and cannot be sorted
  away. Interface staging preserves source commit/RNG boundaries; no fabricated
  Finish/Cancel action where source lacks one.
* Knowledge fact kind `1 draw-count,2 discard-count,3 exhaust-count,
  4 known-next-card,5 known-bottom-card,6 scry-window,7 remembered-board,
  8 announced-boss,9 source-revealed`; only present public facts are listed.
  Position0 means next in the relevant known sequence. Subject may be absent when
  the fact is an aggregate; known source contents are not hypothetical predictions.
* History contains game-visible remembered facts, not an unbounded diagnostic log.
  Fact kinds `1 visited-room,2 revealed-event,3 chosen-option,4 revealed-board,
  5 publicly-known-play,6 publicly-known-discard,7 source-public-counter`.
  Required history cannot be dropped for a guessed retention length; sufficient
  source-approved public summaries may replace repetitions only with equivalence
  proof. Hidden rarity/drop/encounter counters never enter by naming them history.
* Relations kind `1 reward-exclusive,2 reward-parent,3 public-copy-source,
  4 public-permanent-owner,5 public-selection-source,6 public-known-order`.
  Relations express legitimate gameplay identity, not arbitrary handle equality.
  If hidden identity affects behavior but cannot be exposed, NC11/NC12 must provide
  the lawful public representation; no hidden-ID tiebreaker is permitted.

### 6.3 Typed property vocabulary, ordering and privacy completeness

Properties represent additional rule-significant public values in fixed-width
rows, not opaque JSON. Value kind `1 signed-integer,2 boolean,3 enum,
4 binary32-bits,5 row-reference`; binary32 bits are an unsigned32 bit pattern
stored exactly in nonnegative int64, never a float cast. For kind5, value=0 and
`(ref_table,ref_row)` is the value; otherwise ref is absent `(0,-1)`.
Property owners additionally admit `(0,0)` player and `(0,1)` context. Context
sentinel `(0,1)` is legal **only for table14**, never an ActorRef/candidate target;
player retains the ActorRef meaning. Other property owner refs target an actual
row of tables1–13, never properties/candidates
recursively. Subject/content-kind codes share the content namespaces in section8;
their code is typed by the kind, not an ambiguous cross-catalog integer.

Reference typing is explicit: table2 endpoints→1; table4 public_owner_ref→3;
table8 owner→player or7; table10 target→3/4/5/6/7/11 or absent; table12
source→3/4/6/10/11; table13 subject→1/3/4/7/10/11 or absent; table18
endpoints→1–13 with relation-specific kinds frozen by descriptor. Candidate
source→3/4/6/10/11/12 or absent, target→player/1/2/3/4/6/7/10/11 or absent,
with each action-kind's exact pair validated against its core menu. Candidate
target admits ActorRef player `(0,0)`; it never admits context `(0,1)`.
Tables14/17 kind5 values may reference1–13, never themselves or candidates;
tables13/15 scalar value_kind admits only1–4 (their subject identifies the object).
These finite allowed sets do not themselves establish legal action membership.

`object_state` enum is `0 neutral,1 available,2 disabled,3 selected,
4 resolving-public-stage,5 completed,6 removed`; source-specific stage detail
uses property36 with a frozen owner-qualified enum. `price_amount`, intent
damage/hits and other optional scalars are masked when not applicable. For cost
kind X/unplayable the numeric base/turn cost is masked; the X payload uses an
explicit property where public. Relic counters allow signed sentinels only when
the owner-qualified catalog declares them; no counter-1→0 normalization.
`potency`, damage/Block, ordinary costs, amounts/prices, durations and upgrades
are nonnegative where their source law so specifies; signed source values (e.g.
Strength/Focus, misc or sentinel counters) retain signed64. General transport
range never widens a source's legal value domain or bypasses source validation.

Frozen candidate property codes:
`1 base-damage,2 displayed-damage,3 base-block,4 displayed-block,5 magic-number,
6 captured-X,7 cost-until-played,8 cost-for-combat,9 cost-free-once,
10 autoplay-flag,11 ignore-total-flag,12 purge-flag,13 source-site-public,
14 copy-origin-public,15 retained-duration,16 delayed-duration,
17 uses-remaining,18 gold-price-component,19 HP-price-component,
20 maxHP-price-component,21 selected-order,22 can-skip,23 can-confirm,
24 scry-count,25 public-discard-count,26 public-HP-loss-count,
27 public-play-count,28 public-Power-play-count,29 Mantra,
30 orb-slot-count,31 Focus,32 kill-growth,33 potion-capacity,
34 key-kind,35 burning-buff-public,36 source-option-stage,
37 draw-order-known,38 outgoing-card-public,39 upgrade-count,
40 source-visible-counter,41 nominal-price-binary32,42 intent-visible,
43 payment-status,44 transform-input-kind,45 public-turn-cost-modifier`.
All are **P representations**, not permission to expose their private counterparts.
Property40 requires a frozen content-qualified subkey in ordinal; it cannot be
used as an undocumented escape hatch. No numeric time property is assigned here:
NC10 must supply policy-approved public clock fields and their identity.

At catalog freeze every public state-bearing content field must map to a fixed
column, a typed property/relationship, or an explicitly evidenced nonpublic or
irrelevant disposition. Every property has owner kinds, multiplicity, value domain,
visibility and semantic subkey list. A newly discovered public field requiring a
new code changes the descriptor/content identity rather than being silently dropped
or put in code40. This inventory is **D NC12**, not a claim these 45 codes prove
all content coverage. Binary32 transport is available where an actual public value
needs it, not a demand to expose hidden probability thresholds.

Canonical table order preserves visible source order when significant. Otherwise
sort complete public tuples lexicographically (including known bits and public
relations); do not use private instance/creation IDs as tie-breakers. Equivalent
duplicate rows retain multiplicity; their private menu mappings must be proven
observationally interchangeable if no public distinction orders them. Parameters
sort by owning candidate then source-defined ordinal; properties by public owner,
code, ordinal; relations by public tuple; history by legitimate public sequence.
If equal-looking choices have different public futures, the descriptor is incomplete,
not permission to merge or privately order them. Costs/upgrades/scaling/bottles and
provenance relevant to legality cannot disappear under equal display names.

## 7. Candidate carrier, parameters and whole-input validation

Action-kind codes: `1 play-card,2 end-turn,3 use-potion,4 discard-potion,
5 traverse-map,6 choose-event-option,7 buy,8 purge,9 choose-campfire,
10 claim-reward,11 skip-reward,12 take-boss-relic,13 skip-boss-relic,
14 select-instance,15 deselect-instance,16 confirm-selection,
17 cancel-selection,18 reveal-board,19 Singing-Bowl,20 proceed,
21 choose-source-option`. Code21 needs a frozen source option_code and exact
core mapping; not arbitrary callback invocation. NC10 may add an approved wait/time
action under a new descriptor; none is silently assigned here.

Parameter kind `1 count,2 chosen-position,3 ordered-selection-member,
4 X-public-value,5 option-argument,6 public-price-component,7 source-public-flag`.
Rows use section6 value kinds; all spans are local to table17. Ordinary candidates
have `(parameter_start,parameter_count)=(0,0)`. A candidate is already a complete
legal core choice, not a partially validated request assembled from caller edits.
Public candidate rows are features; caller submits only local index plus token.
Do not accept edited candidate/parameter arrays as gameplay input.

Native `step_batch` input is exact unique slot indices int64[B], candidate
indices int64[B], and private per-slot decision tokens. For full-pool stepping B
equals pool size; explicit subset stepping is a separate operation with ascending
indices and the same preflight. Candidate carrier is `0 <= a < U`, stable across
decisions. Actual live legal set is exactly `0 <= a < N_slot`; a carrier-valid
index need not be legal. N can reach the representable count U, with last index
U-1. No array, mask, logits or one-hot vector is allocated with length U.

Candidate order: action-kind, rule-visible source order, target order, option-code,
ordered parameter tuple. Preserve source option order where strategic display order
is rule-significant by using its public_position as source order. Separate target
rows mean separate candidates. Map candidates reference exact edges/nodes; no
unpublished legal edge is selected by coordinate fallback. All native tokens are
opaque provenance, not numeric learned features or RNG identity inputs.

Single facade accepts a built-in integer (not bool) or exact signed64 scalar;
reject float/coercion, other integer scalar widths and scalar arrays. Vector facade
accepts exact native-endian int64[B], copied before native dispatch, including
strided/read-only arrays; no lists/coercion. A candidate-aware consumer can choose
from the actual segment. No policy sampler or dense-head learner is required or
implemented. Standard-space presentation is a separately tested adapter of these
rows, not a promise of generic fixed-head/flattening/wrapper compatibility.

Common preflight, for **every addressed row** before any transition: exact
operation, dtype, shape, contiguous-copy success, index range, unique ascending
slots, descriptor/config compatibility, operation lease availability and the
operation-specific lifecycle predicate below. Check row/reference/span/byte
arithmetic and complete all pre-dispatch reservations for identity/token counters,
coordination/replay bookkeeping and minimal failure reporting.
Reservations check capacity without consuming semantic ordinals or installing IDs.

* **Step-only preflight:** pool Ready, every addressed slot Live with a current
  episode/decision token, nonterminal actionable phase, candidate bound, exact
  private menu membership and core validation of that mapped choice. Uninitialized
  or Terminal addressed slots reject, including one such row in full-pool step.
* **Reset-only preflight:** pool Uninitialized or Ready, every addressed slot
  Uninitialized, Live or Terminal. Reset requires no current decision token,
  candidate or actionable phase. Validate **all** detached creation profiles,
  profile/time policies, seeds (tag, width and range), clock/config inputs, stream
  identities, next episode ordinals and duplicate-RunKey constraints, and complete
  **all** required reservations before any entropy draw, identity installation,
  ordinal consumption, abandonment or run mutation. The same rule covers first
  initialization, explicit terminal reset and administrative live-run abandonment.

Operating, Faulted and Closed pools reject step/reset; Operating cannot be entered
reentrantly. Addressed-slot readiness, not initialization of every pool slot, is
required. Partial initialization is permitted: unaddressed Uninitialized slots stay
uninitialized and do not block valid subset operations. Empty index batches reject.
`read_batch` requires Ready and only Live/Terminal addressed slots, not a token or
candidate; it rejects an addressed Uninitialized slot rather than fabricating a row.

| Operation | Allowed quiescent pool states | Allowed addressed slot states | Action token/menu required |
|---|---|---|---|
| step_batch | Ready | Live | yes |
| reset_batch | Uninitialized, Ready | Uninitialized, Live, Terminal | no |
| read_batch | Ready | Live, Terminal | no |

Any invalid row rejects all rows with no state/RNG/history/reward/token/profile/
episode advancement. No bool coercion, modulo, clamp, row drop or mapped no-op.
Unavoidable future encoding/allocation failure after dispatch is **not** promised
atomic preflight; it follows section10. Native stale tokens reject. An integer-only
facade can protect its cached menu token, not infer stale caller *intent* if an old
index happens to be legal in the current menu; disclose that limitation.

## 8. Identities, descriptor, creation and from-start replay

Proposed distinct candidate identity strings:

| Meaning | Candidate identity |
|---|---|
| Observation layout | `spire-lab-act1-observation-candidate1` |
| Action/menu mapping | `spire-lab-act1-candidate-action-candidate1` |
| Content/site manifest | `spire-lab-act1-content-sites-candidate1` |
| RNG binding registry | `spire-lab-act1-rng-bindings-candidate1` |
| Root adapter | `spire-lab-act1-seed-sha256-candidate1` |
| Replay | `spire-lab-act1-replay-from-start-candidate1` |
| Task | `spire-lab-ironclad-a0-act1-sparse-candidate1` |
| Native delivery/fault | `spire-lab-act1-delivery-candidate1` |

These are **P**, not released v1 aliases. Descriptor contains exact ordered fixed
fields and 18 table layouts, dtypes/bit widths/bounds/mask rules, all dictionaries,
references/ordering, task/action/final semantics and profile/time attachment IDs.
Proposed fingerprint is SHA256 of UTF-8 canonical JSON: keys ASCII-sorted, no
whitespace, integer literals only, arrays retain declared order, no floats or
duplicate keys. Hash the descriptor excluding its own fingerprint. Hash full
content/site/kernel manifests separately and include those hashes. **D:** no final
descriptor hash is asserted while NC10–NC12/required bindings remain open. Document
hash is not a substitute. Any meaning/layout/code/order change requires a new
reviewed identity/hash; unknown versions reject, never best-effort reinterpret.

Concrete dictionary rule: content-kind namespaces are `1 card,2 relic,3 potion,
4 enemy,5 encounter,6 event,7 power,8 intent,9 orb,10 source-option,
11 room-object,12 public-fact,13 effect-site`. Within each namespace use authored
ASCII semantic keys, sorted bytewise, and assign codes1 through the finite entry
count; 0 is reserved unknown, -1 absent where allowed. Character-specific basics
have distinct keys (e.g. `card.ironclad.strike`, `card.silent.strike`), never display
name aliases. Every entry maps to one exact source identity and its reachability
disposition; aliases do not allocate a second kind. Codes must fit unsigned32 for
SiteKey owners. The sorted full mapping and its hash are part of the descriptor;
adding/reordering keys changes the candidate identity, not a silent recode. Site
namespace5 event owners use content-kind6; site namespace8 power uses content-kind7.
Effect-site dictionary13 is the sorted numeric SiteKey list; a public site
property uses that code, not one of its private invocation handles.

Option and intent enum codes are owner-qualified (`owner-key/local-option-key`),
not an unchecked shared numeric literal. This exact finite coding rule fixes how
numeric IDs are assigned; **D NC12** still requires the complete authored identity
and field manifest. It cannot be reconstructed from a count, dirty implementation
enum or the rows already marked as producers. The structural schema remains
standalone even while that factual catalog dependency prevents a frozen fingerprint.

Logical stream identity is a caller-supplied 128-bit value; episode ordinal is
unsigned64, checked. `RunKey=(stream_identity, episode_ordinal)` is detached
metadata, not slot ID. Semantically identical keys/creation inputs/accepted choices
must agree across pool packing and worker counts. Auto-reset increments that
stream's ordinal; invalid preflight does not reserve/consume it. Explicit reset
also gets a new ordinal, never restores an old run. Duplicate live RunKeys in a
pool reject; independent replay may recreate a key in a separate replay owner.

Seed input is one of `Root32` (exact32 bytes), `Seed64` (unsigned64), or
`Entropy32` (request to capture exactly32 system-entropy bytes after input preflight).
For Seed64, proposed root is SHA256 of bytes
`spire-lab/act1/seed/candidate1` followed by a NUL byte and seed encoded unsigned
big-endian64. Entropy failure is administrative initialization failure, never a
replacement seed. Root32 is used unchanged. Existing env-seed-v1 derivation is
untouched. HMAC-v1 episode field is ASCII
`a1:<stream identity as 32 lowercase hex>:<ordinal as canonical decimal>`;
partition remains train/selection/final/test. Auto-reset reuses configured root,
partition and detached profile with the next semantic episode identity; it does
not use new wall-clock entropy or inherit Note output. Clock attachment is NC10.

Replay record is privileged, owned, opt-in and from-start only. Header contains
all identities/hashes above, source/rules revision, root/seed adapter/partition,
RunKey, exact creation profile snapshot and profile/time policy attachment.
Decision records contain accepted ordinal, public-decision digest, local candidate
index, semantic action tuple (typed private source provenance plus selected
semantic identity/order), and post-boundary public/core comparison digests.
Live tokens/worker/slot addresses are not replay identities. An accepted action
is recorded only after it passes preflight; post-dispatch faults retain accepted
prefix and last known checkpoint, never fabricate completed delivery.

### 8.1 Replay outer byte grammar

This **P** grammar fixes framing independently of the unresolved attachment/core
contents. `||` is byte concatenation; `[n]` means exactly n repetitions, not a
length field. `u8`, `u16`, `u64` are unsigned integers of 1, 2, 8 bytes respectively,
all multibyte integers big-endian. `i64` is eight-byte big-endian two's complement.
`HASH` is exactly32 raw bytes, never hex text. `BYTES(x) = u64(len(x)) || x`;
length is **bytes**, excluding the prefix. `STR(s) = BYTES(UTF8(s))`, with strict
Unicode scalar UTF-8 (no BOM, malformed/overlong sequences or lone surrogates).
No normalization, NUL terminator, padding, alignment or implicit host-endian field.

`SCHEMA = STR(schema_id) || HASH(schema_hash)`; nonempty IDs identify an exact
version. `ATTACHMENT = SCHEMA || BYTES(payload)`. For any stated type T,
`OPT(T) = u8(0)` for absent or `u8(1) || T` for present. No other presence tag is
valid. Absent and present-empty payloads differ; no bytes follow an absent tag.
Payload bytes are opaque to this outer grammar, owned and validated by their
schema contract, not reserialized as text. NC10 supplies profile/time contents and
allowed presence; this framing does not select defaults or approve their schemas.

The complete ordered file grammar is:

* `FILE = 41315250 || u16(1) || BYTES(HEADER) || u64(record_count)
  || RECORD[record_count] || u8(end_status) || OPT(ATTACHMENT)`.
  The first four bytes are literal hex (ASCII A1RP), not a string field. The final
  attachment is interrupted-record diagnostics only. Status `0 clean-prefix` or
  `1 terminal` requires it absent; `2 interrupted` requires it present. Unknown
  status/tag/version rejects. End-of-file must follow immediately.
* `HEADER = SCHEMA[8] || HASH(descriptor_hash) || STR(source_revision)
  || STR(rules_revision) || u8(seed_kind) || seed_argument || root32
  || u8(partition) || stream16 || u64(episode_ordinal)
  || OPT(ATTACHMENT) || OPT(ATTACHMENT) || OPT(ATTACHMENT) || OPT(SCHEMA)`.
  The eight SCHEMA pairs are exactly the section8 identity-table order:
  observation, action, content/site, RNG registry, root adapter, replay, task,
  delivery/fault. Each hash pins that component's canonical descriptor/manifest,
  not the hash of its name; the separate descriptor hash pins their full composition.
  Source/rules revisions are nonempty exact revision strings. `seed_kind` is
  `1 Root32,2 Seed64,3 Entropy32`; seed_argument is u64(seed) **only for2**, empty
  for1/3. root32 is always exactly32 bytes (including captured entropy for3).
  Partition codes `1 train,2 selection,3 final,4 test` preserve existing meanings.
  stream16 is the 128-bit stream identity big-endian, high byte first. The three
  attachment slots are, in order, creation-profile snapshot, profile policy, time
  policy. The last optional SCHEMA identifies the private-core digest encoding;
  absent explicitly disclaims full-core comparison, never means public-only core.
* `RECORD = u8(1) || BYTES(DECISION)`; tag1 is the only record kind in this version.
  `DECISION = u64(accepted_ordinal) || HASH(pre_public) || u64(candidate_index)
  || BYTES(semantic_action_json) || u8(completion) || post_fields`.
  Ordinals start0 and are consecutive. Candidate_index must additionally fit the
  section7 carrier/legal menu. Completion `0` has empty post_fields; completion `1`
  has `HASH(post_public) || OPT(HASH(post_core))`. Other completion tags reject.
  A completed record's post_core presence must equal header core-schema presence.
  Only the last record may be incomplete, and only with end_status2. Interrupted
  status may also follow a completed prefix (e.g. later export failure); it does
  not certify successful delivery. Zero records are allowed, including an
  initialization fault after a complete header exists. If entropy/header creation
  itself fails, retain diagnostics outside FILE, not a fabricated root/header.
  Diagnostics retain last known site/invocation/checkpoint and external cause
  where available under their schema, not fake post digests.
  Terminal status requires the regenerated last boundary (or initialization for
  zero records) to be terminal. Prefix status promises only the recorded prefix.

All lengths/counts must be in `0..U` (section6), despite their unsigned64 encoding.
This bound does not narrow Seed64 or the unsigned64 episode/accepted ordinals.
Before allocation or advancing a reader, check additions, count*element-width,
remaining enclosing bytes and host-size conversions without wrapping. A BYTES
child must consume exactly its declared length; a HEADER/DECISION must have no
unconsumed suffix. Reject truncated, oversized, unknown-tag, extra-byte or
noncanonical input; never ignore an extension or retry a different framing.
Encoding failure is administrative, not legal-content truncation. A capture may
retain an in-memory accepted prefix on export failure without promising that a
partially written FILE is readable; record_count describes a finalized owned file.

Canonical JSON for descriptor hashes and semantic actions is precisely: objects
with unique ASCII keys sorted by unescaped key bytes, ordered arrays, Unicode
scalar strings and base10 integer literals in `[-2^63,2^64-1]` only. No booleans,
null, floats, insignificant whitespace, duplicate keys, leading zeros, plus signs
or negative zero; whitespace inside strings is preserved. Escape quote and
backslash as `\"` and `\\`; encode U+0000..U+001F as lowercase `\u00xx`, all
other scalars directly as UTF-8 (including slash, U+007F and non-ASCII); no other
escapes or normalization. An optional semantic tuple member uses `[]` or `[value]`,
not null. The semantic-action root is an ordered array, framed by BYTES, with
typed private source references using semantic run-local provenance, not addresses.
The action component schema pins its exact tuple fields/dictionaries; their full
content closure remains NC11/NC12. Empty `[]` below is a framing fixture, not a
legal action. Opaque attachments need not be JSON.

### 8.2 Public-decision digest and encoding vectors

Public-decision SHA256 input has **no outer BYTES prefix**: raw descriptor HASH,
then `context.values[0..12)`, `context.known[0..12)`,
`player.values[0..16)`, `player.known[0..16)` for the one addressed slot, then
tables1 through18. Each table contributes `u64(segment_row_count)`, **all** its
values in row-major/declared-column order as i64, then **all** its known bytes in
the same order. Fixed values also use i64; known bytes are raw0/1 (not ASCII or
bit-packed). Do not interleave each cell with its known bit. Widths/order come
from section6; no table-ID tags or fixed-array lengths are added. Empty segments
contribute eight zero bytes and no cells/masks. Validate unknown-value zero rules
before hashing. Exclude batch dimensions, offsets, tokens, slot/worker metadata,
RunKeys, attachments and private/core state. For vector same-step replacement the
post_public digest describes the stepped run's detached terminal boundary, not the
replacement; replacement creation has its own from-start record. Pre_public
describes that run's regenerated actionable boundary.

The following **independently calculated author representation vectors** use hex
notation `00*n` = n zero bytes and `+` = byte concat in the table (equivalent to
`||` above), with no printed separators in the actual bytes. They are structural
fixtures, not source/runtime evidence or valid initialized game states. SHA256
covers the exact expanded bytes in column2.

| Vector | Exact bytes/preimage in hex notation | SHA256 |
|---|---|---|
| G01 empty bytes | `0000000000000000` (BYTES of empty payload) | `af5570f5a1810b7af78caf4bc70a660f0df51e42baf91d4de5b2328de0e83dfc` |
| G02 UTF-8 | `0000000000000002c3a9` (STR of U+00E9, length2 bytes not1 scalar) | `1d9b2f68e31ef2b6730c67d7729ca6a523e7f19d8987ca7c72eb93cb3bb9d979` |
| G03 incomplete record | `01 + 000000000000003b + 00*48 + 0000000000000002 + 5b5d + 00` (ordinal/index0, zero pre digest, JSON `[]`, completion0) | `cbba1c720f971d132277b6a1547aa68dee3929bd387ab1b9bca27be9ee76611d` |
| G04 empty tables | `00*428` (zero descriptor/fixed values/known, all18 tables empty) | `fced892c73c66d68fdbb01bb68464549f19996231e05ceddce888520d3b70227` |
| G05 signed/mask/table | `00*32 + ffffffffffffffff + 00*88 + 01 + 00*155 + 0000000000000001 + 00*199` (context cell0=-1/known1; table1 has one all-unknown width7 row; other tables empty) | `9970458824150d3f8e19e7bcb2d608b9405f2cc31f6fedfd0136e5464b3eea46` |
| G06 zero-record file | `41315250000100000000000001b8 + H + 00*10`, where H is defined below | `01ed74ddb35dcc8f98e2542a0b85707c5cbbfc3506f1149f720e0950a5979a19` |

G05 is491 bytes: descriptor32 + context108 + player144 + table1(8+56+7) +
other17 lengths136. Its deliberately unknown fixed cells do not certify legal
phase/content. Optional-attachment framing examples: absent=`00`;
present with schema ID `x`, zero schema hash and empty payload=
`01 || 0000000000000001 || 78 || 00*32 || 0000000000000000`.
These distinguish empty payload, absent attachment, byte count, sign encoding,
mask order, table order and record/action framing without private fixtures.
For G06, H is exactly `(000000000000000178 || 00*32)[8] || 00*32
|| (000000000000000178)[2] || 01 || 00*32 || 01 || 00*28`, where parenthesized
`[n]` repeats the entire group. H is440 bytes and FILE464 bytes: eight schema
pairs with fixture ID `x`/zero hash, zero descriptor hash, source/rules `x`, Root32
zero root, train, zero stream/ordinal, four absent header options; count0, clean
prefix and absent diagnostics. Fixture IDs/hashes are not recognized schemas, so
this is a framing vector, **not** a semantically acceptable runnable replay file.

Full private-core digest encoding is **D NC11/NC12** until the authoritative state
inventory closes; it must include hidden/RNG state and cannot be substituted with
a public digest. Replay records without an approved core encoding cannot claim
full core equivalence merely because observations agree.

Replay verifies header/inputs, performs normal initialization, regenerates the
menu at each boundary, compares digests and the index-to-semantic mapping, then
applies the ordinary core choice. Mismatch returns earliest decision/component
with provenance, not a repaired path. Recording off does not change RNG/state.
No arbitrary snapshot import, caller-mutated profile reread or hidden process
history. External allocation/export failure cannot be recreated as gameplay;
report it as external provenance rather than clean replay equivalence. Snapshot2
remains the old combat-only contract; no run checkpoint restore API is promised.

## 9. Operations, reset, detached finals and delivery metadata

| Operation | Exact ownership/outcome contract |
|---|---|
| `new(config)` | Validate B>0, workers>0/configured effective limit, identities, reset/recording mode, complete catalog/bindings/profile/time compatibility; no run installed until explicit reset. |
| `reset_batch(indices, requests)` | Section7 reset-only preflight accepts addressed Uninitialized/Live/Terminal slots; initialize/install each requested new run and initial genuine decision. Nonrequested slots untouched, including uninitialized ones; runtime failures follow section10. |
| `read_batch(indices)` | Return detached cached public decisions and non-policy tokens/metadata; no state/RNG/token/ordinal advancement. |
| `step_batch(indices, actions, tokens)` | One accepted local choice per addressed live slot; automatic resolution and optional same-step replacement; output in requested logical slot order. |
| `replay_from_start(record)` | Separate owned replay instance/result, not live pool installation or recovery of a faulted pool. |
| `close()` | Quiesce, release live state; idempotent; old arrays/final/profile outputs remain valid. |

Mode `1 SingleManual`: exactly one slot; terminal observation stays readable with
zero candidates and preserved final resources until explicit reset. Terminal step
is invalid. Mode `2 VectorSameStep`: every ended addressed episode is captured
before replacement; replacement initial observation appears in the same result.
No mode silently resets a faulted pool; create a new pool after fault/close.

`read_batch` and successful `reset_batch` return the addressed packed observations,
current identities/tokens and delivery metadata, with no task reward or fabricated
terminal transition. Reset does not report the replaced run as a successful final.
An explicit reset of a live run is an administrative abandonment, not death, +1,
truncation or a recorded accepted gameplay action. Its opt-in lifecycle record
contains old/new RunKeys and the reset request, separately from replay actions.
Subset operations use B=number addressed, while `slot_indices[B]` maps those rows
to pool slots; compact final_slots below index this **returned** order. Metadata
also retains pool slot identity outside policy features to avoid ambiguity.

Reset-specific future acceptance cases (not executed native tests):

| Case | Setup/discriminator | Required assertion |
|---|---|---|
| R01 first reset | New VectorSameStep pool with multiple Uninitialized slots; reset a proper subset with valid requests | No token/menu required; only addressed slots receive their first RunKeys/decisions; remaining slots stay Uninitialized; valid initialized subset read/step works, full-pool read/step rejects until all addressed slots qualify |
| R02 terminal reset | SingleManual terminal with zero candidates, retained final resources | Explicit valid reset accepted without terminal action/token; installs new ordinal/initial decision, no fabricated reward/final transition; old detached terminal values unchanged |
| R03 abandonment reset | Live decision and cached old token | Valid reset without action/token abandons old run, installs new ordinal/profile; not defeat/truncation/gameplay action; old token cannot step replacement and old outputs stay detached |
| R04 invalid final reset | Partially initialized multi-slot pool with Uninitialized/Live rows; last request has bad profile/seed/config/ordinal or reservation failure; repeat invalid reset on SingleManual terminal | Reject entire batch before entropy/identity consumption or abandonment; all slot lifecycles, installed runs, profiles, tokens, RNG/history/reward/ordinals and nonrequested slots unchanged; no partial install |

Successful step returns current packed batch, rewards float32[B] exactly -1/0/+1,
terminated bool[B], truncated bool[B] all false, non-policy metadata and compact
finals. Rewards/end flags identify **stepped** episodes; current observations and
tokens identify current/replacement episodes. Native metadata has:

* `current_identity[B]`, `stepped_identity[B]`: RunKey (128-bit stream plus64-bit
  ordinal, exact bytes/integers, not float conversion); `decision_generation[B]`
  unsigned64 and opaque current `token[B]` outside learned features.
* `delivery_id`: pool-private checked unsigned64 operation ordinal, plus a unique
  pool-instance128 identity. It never enters RNG/gameplay. Read delivery IDs may
  advance delivery bookkeeping only; they do not rotate decisions.
* `final_present` bool[B], `final_slots` int64[K] ascending, exactly true slots,
  independently owned final packed batch with K segments. Final segment j maps to
  logical returned slot `final_slots[j]`, carries retired identity/outcome/reward
  and zero candidates. K=0 and K=B have exact empty/full shapes, not fake rows.
* `profile_attachment`: profile-lane-versioned owned result per current/final
  identity, with explicit presence/status supplied by NC10. No automatic mutation
  of reset requests, no completed profile result invented after a fault.

Concrete metadata transport uses `run_identity uint64[B,3]` (stream high64,
stream low64, ordinal), `decision_generation uint64[B]`, `token uint8[B,32]`,
`delivery_identity uint64[3]` (pool high64, pool low64, operation ordinal), and
`slot_indices int64[B]`; stepped/final identity arrays use the same three-column
format. Tokens are exact opaque32-byte values tied to pool/RunKey/decision; reject
foreign or old values, never derive gameplay streams from them. Counter overflow
is administrative. These buffers are non-policy metadata, not additional tables.
Profile attachment has `schema_id`, `schema_hash[32]`, `presence bool[B]` and owned
per-row length-delimited bytes; NC10 supplies validated payload/status semantics.

Single facade returns one observation with native offsets removed; vector facade
may present each table as per-slot dictionaries of numeric slices retaining their
detached owner. No Python card objects or rules are reconstructed. Final facade
delivery must retain exact retired identities/presence; any standard environment
info convention is a separately version-tested adapter, not reason to change the
native contract. The operation lease remains held through all facade packaging.

Caller mutation of any exported array cannot alter native cached decisions or
other exported batches. Retaining an individual slice retains its owner. Subsequent
step/reset/close cannot mutate old current/final/profile values. Immutable shared
storage is allowed only where caller mutation cannot alter authority; a nominal
read-only flag by itself does not establish detachment.

## 10. Invalid input versus runtime administrative fault

Pool lifecycle `1 Uninitialized,2 Ready,3 Operating,4 Faulted,5 Closed`.
Slot lifecycle is `Uninitialized`, `Live` or `Terminal` at quiescent usable
boundaries; Live means an installed genuine nonterminal Decision, Terminal an
installed Dead/Act2Ready boundary. These slot states are distinct from private
resolving phases and whole-pool Operating/Faulted/Closed. A successfully created
pool is Uninitialized when no run is installed; after any successful reset it is
Ready, including partial initialization. Pool Ready does not imply every slot is
initialized. Section7 checks only addressed slots; unrequested slots are untouched.
On accepted dispatch Uninitialized/Ready becomes Operating and returns to the
appropriate quiescent state on success. Preflight rejection preserves the prior
pool/slot states. Post-dispatch failure faults the whole pool, including its
uninitialized slots; it is not partial-init success or rollback. Faulted/Closed
never regain readiness through reset. No public observation exists for an
Uninitialized slot, and read never initializes one implicitly.
One operation lease prevents reentrant/concurrent read/step/reset/delivery on the
same pool. Slots have exclusive mutable owners; persistent native pool-private
workers dispatch only after all-row preflight, effective-one iterates directly.
Semantic identities are assigned before dispatch; result order is logical order,
never completion order. No per-effect Python callbacks or new thread per step.

| Failure boundary | Result | Preserved/forbidden |
|---|---|---|
| Invalid caller/config/profile/token/candidate before dispatch | Typed input error; return to prior readiness | Every slot unchanged, zero RNG/history/ordinal/reward/token changes; no rollback needed |
| Reservation/setup allocation before dispatch | Administrative setup error, no installation | No partial reset/identity allocation; no entropy drawn before prior validation |
| Core, initialization, RNG, invariant, encoding/allocation after dispatch | Quiesce all workers, retain mutations, fault entire pool | No successful subset/batch, no retry/rollback/reset/defeat/truncation |
| Binding export or facade packaging before lease release | Same runtime fault attached to current delivery and its current/newly retired records | Do not annotate previously delivered episodes or repeat submitted actions |
| Caller error after successful delivery (e.g. learner failure) | Outside completed native delivery | Cannot retroactively fault an old delivery or claim replayed gameplay failure |

Fault record is detached privileged diagnostics, not a policy observation. Required
fields: delivery identity, operation (`new/reset/read/step/replay/close`), stage
`1 initialize,2 core,3 project,4 pack,5 export,6 facade,7 bookkeeping`, fault kind
`1 content,2 invariant,3 RNG-exhaustion,4 numeric,5 allocation,6 worker,
7 encoding,8 binding,9 facade`, affected RunKeys/current-retired association,
logical slot, site/invocation/checkpoint when available, accepted-action ordinal,
last committed checkpoint, original cause and optional RNG domain/scope/cursor.
First observed fault cause is retained; concurrent additional causes append with
their own attribution. No scheduling-dependent “first fault” is claimed replay-
deterministic. Workers quiesce before the error escapes; no queued operation starts.

Diagnostic numeric envelope is one signed64 row of width12 per cause, columns
`operation, stage, kind, pool_slot, association, accepted_ordinal_ref,
site_namespace, site_owner, site_operation, site_variant, checkpoint, cursor_state`,
with matching int8 known bits. Operation codes are1–6 in the order listed above;
association `0 none,1 current,2 newly-retired,3 current-and-newly-retired`.
`accepted_ordinal_ref` indexes the same cause's aligned uint64 ordinal attachment,
not a narrowing cast of that ordinal; absent is known1 sentinel-1. Identity
buffers retain unsigned64/128 widths from section9. A cursor exceeding
signed64 is not cast into this row: cursor_state `0 absent,1 available` points
to an exact unsigned128 `(high64,low64)` diagnostic pair, including exhausted
HMAC-v1 sentinel2^64; unknown ordinal/checkpoint fields mask0. Semantic invocation
and domain/scope/cause strings are owned length-delimited diagnostic attachments,
never policy features. If ordinary allocation failure prevents detailed formatting,
the pre-reserved minimal error retains delivery/operation/stage/kind/association
and original cause category; do not promise allocating arbitrary text under OOM.

There is no fabricated successful final/profile payload after failed delivery.
Already applied durable changes remain in retained fault provenance; the API does
not promise rollback or a usable postfault gameplay observation. Administrative
faults earn no task reward. Faulted pools allow only diagnostics/close, not step,
read-as-success, reset or retry; previously detached successful arrays remain valid.

## 11. Acceptance cases and exact integration gaps

All cases below are **future acceptance specifications, NOT executed gameplay
tests**. The author validates this document's structure/links/layout arithmetic,
not any implemented simulator. D-qualified cases cannot assert invented outcomes.

| Case | Setup/discriminator | Required assertion or exact prerequisite |
|---|---|---|
| C01 authority | Non-final Feed/growth, temporary upgraded copy and permanent source | One live resource/growth owner changes at resolution; no final-pile reconciliation or accidental permanent copy growth; NC11 owner law required |
| C02 current player | Ironclad off-color White Noise/Alchemize via Note, autoplay and repeat | Actual Ironclad pool, not printed color/outer policy; no exclusive Silent potions or free orb resources; explicit four-character resolver fixtures without other-character run starts |
| C03 exact offers | Three-card ordinary versus Foreign Influence | Distinct-ID ordered support; conditional p(x)/(1-s) for Foreign Influence; retry rarity only where source says; no reward-pity mutation by temporary selector |
| C04 upgrade/overflow | Master Reality, Attack Potion at hand9 selecting Searing Blow | One selected identity; first copy can be +2 hand and second +1 discard; Foreign Influence contrast; stored input versus animation clone preserved |
| C05 placement | Empty/one/n-card draw pile, Chrysalis/Metamorphosis | Empty deterministic; one-card insertion below old top; n positions not n+1; all identities before ordered insertion children |
| C06 potion phases | Brew under Sozu inside/outside combat, capacity vs vacancies, initially full belt | Combat still proposes capacity-many, outside Sozu none; deferred fill includes consumed slot, leftmost live vacancy; Bark no attempt doubling |
| C07 nested origin | Distilled Chaos→Distraction, Burst→Alchemize, Elixir→Branch | Switch to inner SiteKey; captured X/flags/payment survive pause; no second offer generation or payment on resume |
| C08 histories | Skipped relic, reward duplicates, merchant rejection | Pool entry consumed on offer; reward rarity/history once per slot before fixed-rarity retry; merchant whole proposal retry; front/back/depletion discriminator preserved |
| C09 continuations | Neow/shop/boss relic/event children and ordered Forethought+ | Return to exact parent/checkpoint; A then B differs from B then A where source says; no universal map/FIFO/drain/transaction shortcut; NC11 full branch laws |
| C10 durable keys/ending | Recall with other campfire options filtered; chest linked sapphire; boss relic skip/child | Filter→append→scan; deferred flags not early; legal key deferral/skip; no +1 until fully resolved Act2Ready |
| C11 death/timing | Retaliation/rescue/heal, Reaper pending/current work | Preserve entered/pending distinctions; no universal rescue/drain/lost-heal assertion; NC10/NC11 timer/fidelity decision prerequisite |
| C12 offsets | Table lengths[2,0,3] | Offsets[0,2,2,5]; slot2 local1 maps global3, stays local1 in features; neighbor repack/worker count changes no slot content/menu |
| C13 empty/large | Empty table/full terminal, legal growing piles/deck/candidates | Exact[0,W]/[0,W] masks; every legal row retained at any representable length; no dense cap, page, merge-by-name or truncation; real allocation exhaustion administrative |
| C14 references | Row within global buffer but outside submitting slot; malformed spans/tags | Reject transport/content invariant, never resolve neighbor; checked additions/multiplications/conversions, no wrapping |
| C15 knowledge | Rename handles/swap unknown order or hidden board/AI while public state equal | Fixed fields/table lengths/order/masks/refs/candidates identical; Frozen Eye/scry/real reveal change only legitimate knowledge; Dome reconstructive damage/hits hidden |
| C16 input atomicity | Invalid final row, stale/cross-slot token, -1/N/U index, bool/float/wrong-width/wrong-shape | Entire batch unchanged including RNG, tokens, histories, IDs/profile and rewards; no action-on-terminal; carrier membership alone insufficient |
| C17 staged choices | Ordered/unordered selections with min/max and optionality | No combinatorial subset enumeration needed; selected sequence retained; no premature commit/reveal/RNG or invented Cancel/Confirm |
| C18 reset/finals | Mixed endings K0/KB/consecutive same-step reset, retained old slices | End flags/reward/retired final identity align; replacement action never executed; single terminal awaits explicit reset; caller mutation and later close cannot corrupt old/native values |
| C19 profile | Note incoming/outgoing and leaving/death/fault/reset | Independent reset profiles never inherit result; NC10 exact staged/committed attachment and reveal boundary required; do not label selection as transition persistence |
| C20 replay | Same creation/choices across packing/workers/recording off-on; section8 vectors and malformed lengths/tags/JSON/order | Same semantic boundaries/digests; captured profile, exact index-to-choice mapping; byte/digest vectors match, malformed/overflow/extra-byte wire rejects; mismatch earliest provenance; no snapshot import or external-fault equivalence claim |
| C21 runtime fault | Mutation then core/packing/export/facade failure and concurrent worker | Committed progress retained, all workers quiesced, whole pool faulted; no success/reward/fake loss/rollback/retry; current delivery/current-retired attribution, not prior episode |
| C22 registry omissions | Add independently identified producer/consumer not preflagged | Catalog coverage rejects missing site/law/public field; complete labels cannot conceal Partial/Open; no unknown-field fallback at runtime |
| C23 stochastic dependency | Five relic shuffles, missing faces, duplicate board, upgrade prefix, map-empty branch | Exact conditional/joint input measure/binding must be supplied/approved; namespace separation or mapper existence cannot approve IID; search miss is not impossibility |
| C24 compatibility | Existing supplied combat scenarios/RNG/snapshot/public view and future run API | Existing entry semantics and all six pins unchanged; no run rows/domain casts into combat-v1; old scenario limits do not cap run states |

### Human choices and main-owned integration

The remaining **human representation choices** are NC02/NC03/NC05–NC09: accept or
revise these exact records, numeric layouts/carrier, separate identities, seed
adapter, replay/delivery semantics within retained architecture. No need to
reapprove unchanged A constraints. **Fidelity choices** require the exact
conditional/joint measure decision in NC04 (including whether proposed domain
splits are lawful) and the profile/time lane's eventual defaults/clock/Note
output choices in NC10; no always-on/zero/CPU-time Secret Portal rule is selected.

Genuine **research/integration dependencies**, not questions resolved by a human
merely blessing a schema: zero-elite initialized image/empty branch, unresolved
source conditional kernels, complete callback/cost/terminal/primary-relic and
parent-return inventories, numeric content/property dictionary equality, exact
Act2Ready transition and profile/time attachment. Main integrates independently
reviewed sibling results later; this author has not polled them or incorporated
unreviewed output. If a fidelity deviation is proposed instead of a missing proof,
state the changed observable law explicitly for human decision, never relabel it F.

No original game/class/initializer/runtime was executed, no product/dependency
files changed, and no builds/benchmarks/training ran. Documentation checks cannot
close whole-spec approval, implementation-plan review or separate start approval.
