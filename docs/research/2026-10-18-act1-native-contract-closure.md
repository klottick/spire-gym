# Ironclad A0 Act 1: native run-contract draft

Date: 2026-10-18

**Status: supporting architectural draft; not the complete Act 1 specification, approval, or implementation authorization.**

This is a concrete ownership/interface proposal for integration into the one complete Ironclad A0 Act 1 specification. It refines, but does not replace, the user-approved architecture. Proposed type names, row layouts, source policies, and fidelity choices remain subject to whole-goal review. This is neither a product API nor evidence that a test, simulator, or original runtime was executed. Main retains integration and approval; this document creates no new delivery goal or approval gate.

## 1. Authority and evidence boundary

| Boundary | Owns | Must not become |
|---|---|---|
| `spire-core` | Typed rules/catalog; authoritative run and temporary combat state; legality, effect order, source-specific generation and RNG | Python-dependent rules, observation policy, scheduling |
| `spire-env` | Logical episodes; public projection; candidate indexing; Act 1 outcomes; native preflight, scheduling, replay, faults | Duplicate gameplay rules or alternate combat simulator |
| `spire-python` | Checked numeric transport, owned delivery, facade packaging/error conversion | Effect orchestration, gameplay legality, retries or repairs |
| Policy/future learner | Choice among published legal candidate rows | Hidden state, simulator RNG, Python gameplay callbacks |

Run and standalone combat use the same detailed combat engine with explicit resource context. A run does not maintain a separately writable combat copy of persistent HP, deck, gold, relics, potion slots, keys, or counters to reconcile later. Existing combat-v1 identities and semantics remain unchanged: `spire-lab-rng-hmac-sha256-v1`, snapshot2, `spire-lab-combat-reference-v2`, `spire-lab-combat-observation-v1`, `spire-lab-env-seed-sha256-v1`, fingerprint `8fd66d212259d2bdd27139988a772e1a6c223f8d65524f38c74174a0d0843357`. Run observations, descriptors, and replay records require separate versioned identities; do not widen or repurpose combat-v1.

Basis: the approved Act 1 architecture and preserved phase-1a drafts on run architecture, packed interface, generation context, profile, coverage, map/random, execution, acquisition, and project randomness, plus read-only inspection of the old worktree's native combat/env/binding interfaces. That worktree contains dirty and untracked work; it was context only and none was copied. Evidence is static/source-grounded, not original-runtime validation. No original game class, initializer, or runtime was executed. Owned-JAR SHA-256: `cfad868ac8d65a88e71a0bf096fb09f78811e553effe0787c5309a655e081673`; upstream build/channel and historical host-JRE applicability remain unestablished.

Evidence locators (read-only phase-1a context, not links/dependencies from the
clean main baseline): `2026-10-18-ironclad-a0-act1-architecture-design.md`,
`2026-10-18-ironclad-a0-act1-packed-interface-design.md`,
`2026-10-18-ironclad-a0-act1-generation-context-design.md`,
`2026-10-18-ironclad-a0-act1-profile-design.md`,
`2026-10-18-ironclad-a0-act1-coverage-audit.md`,
`2026-10-18-ironclad-a0-act1-map-random-followup-findings.md`,
`2026-10-18-ironclad-a0-act1-execution-followup-findings.md`,
`2026-10-18-ironclad-a0-act1-acquisition-followup-findings.md`, and
`2026-10-16-project-randomness-design.md`. The map follow-up qualifies its
empty-elite witness and shuffle counts by supplied choices/uniform64 measure;
execution follow-up distinguishes queue retention from execution and leaves
timer/callback closure open; acquisition follow-up distinguishes offer, claim,
key, and Note persistence boundaries. Source/tool identity and limits are above;
no source excerpts or private artifacts are included.

## 2. Typed state and ownership proposal

### 2.1 Persistent run and temporary combat

One `RunState` is the only authority for persistent state. Draft fields:

| Field | Contents and constraint |
|---|---|
| `creation` | Owned validated immutable `RunCreationProfile`; ruleset/character/Ascension, prior-run facts required by rules, seed/partition and version IDs. Not an account or save object. |
| `player` | Current/max HP, gold, permanent card instances, relic instances/counters, potion belt/slots, keys and other persistent resources. |
| `map` | Topology/room assignments, burning-elite marker, current node/floor, visited and legitimately revealed facts, required history. Private generation state stays private. |
| `phase` | Tagged phase for Neow, map, room, combat, genuine decision, suspension, death, or resolved Act-1 completion. |
| `continuations` | Typed parent contexts and captured parameters/choice progress/resume point; owned values or stable IDs, never dangling pointers or duplicate mutable resources. |
| `rng` | Authoritative project root/partition and mechanic stream states; never policy features. |
| `knowledge` | Rule-legitimate revealed facts and reveal boundaries, not a copy of hidden state. |

A draft `RunPhase` vocabulary is `Neow`, `MapChoice`, `Room(RoomId)`, `Combat(CombatId)`, `Decision(DecisionKind)`, `Suspended`, `Dead`, `Act1Complete`. Do not merge distinct source-specific screens into one generic menu. Run owns phase during combat; combat is temporary substate, not a second episode.

`CombatState` owns enemies, piles, turn-local effects, action queues and combat knowledge/RNG. Persistent effects access a borrowed `RunCombatContext`. Standalone combat gets an explicit validated player/resource context; character is never inferred from deck or card color.

Distinct typed identities are required for permanent deck instances, temporary combat instances, generated instances, effect sites, choices and private decision tokens. A card retains rule-relevant kind, upgrade, cost/stat overrides, scaling, and provenance. A copy has a new instance identity and explicit `CopyOrigin` (source, producing site, capture boundary); it does not alias or mutate its source unless the rule says so. Feed/Genetic Algorithm/Lesson Learned-like permanent changes target the authoritative permanent instance/resource at their qualifying resolution point, including non-final kills. Temporary upgrades/generated cards do not become permanent merely by surviving in a combat pile.

`PlayOrigin` is explicit (at least hand, queued/autoplay, repeat, and source-defined origins), carrying captured X/energy and source-defined flags through continuation state. Static research notes repeated-play sites that copy X while explicitly setting ignore-total and autoplay flags; this is not a universal legality bypass. Full origin/cost/callback closure is unresolved, so no single global payment order is inferred here.

### 2.2 Durable effects, interruption and terminal outcomes

There is no global end-of-room/combat commit transaction. Each durable mutation occurs when its qualifying effect resolves, in source-defined order. A later interruption/fault does not roll back committed effects; a queued effect is not proof it executes. No universal queue drain, lethal rollback, automatic rescue, or “all queues empty” completion predicate is introduced.

Death is terminal when the supported rule establishes it. A future queued heal does not undo established death; rescue and later damage are separate events. Static timer/headless and broader callback/cancellation questions remain unresolved.

Act-1 task reward is `+1` only at resolved `Act1Complete` after boss rewards, boss-relic take/skip, and every resulting nested choice/effect; `-1` on death; `0` otherwise. Both endings are `terminated=true`, `truncated=false`. No turn/step/time cap, administrative defeat, or truncation of legal candidates. Never end at boss lethal damage or before nested handoff work. `Act1Complete` retains native Act-2-entry state without claiming Act 2 gameplay or Heart victory.

## 3. Effect-site/source policy and randomness

### 3.1 Source contract

Every content operation names a typed `EffectSiteId` and receives authoritative `RunContext`, validated source parameters, and the source-defined continuation. Current character comes from the run at the effect boundary—not printed card color, copy origin, outer effect, or global mutable state.

Each site independently declares eligible support, rarity/category/weights, exclusions, duplicate/history behavior, copy/upgrade/cost treatment, RNG domain/scope and draw time, mutation/commit point, destination/continuation, blocked/empty behavior, and evidence status. Distinguish random offer, fixed named creation, copy, transform, potion offer, permanent acquisition, and movement/selection. Nested producers switch to the actual inner site. No fallback to Ironclad, colorless, “any legal card,” reroll, dropped effect, or defeat unless that exact source rule is evidenced.

Keep separate laws for temporary generators, rewards, transforms, shops, copies, fixed specials, and potions. Existing research specifically distinguishes Shard-expanded reward support from ordinary generator support; flat transform support from rarity-first rewards; merchant whole-proposal retry from fixed-rarity identity retry; Colorless Potion choice from skippable typed potion; and Foreign Influence rarity retry from retained-tier potion duplicate rejection. These examples do not close the complete source-law table.

### 3.2 RNG vocabulary proposal (not approved identifiers)

Preserve the pinned HMAC-SHA256 v1 contract: semantic identity, project domain/scope, checked exhaustion, no worker/address/token derivation, and no original seed/stream parity. Preserve mechanic-relevant conditional laws and intentional dependencies. Existing combat domains do not become run domains.

Candidate *families* for a future registry:

| Candidate family | Semantic scope proposal | Unresolved boundary |
|---|---|---|
| `run.map.generation` | Run identity, map-generation identity | Topology/room-stage cursor sharing, shuffle, fallback, zero-elite handling and input measure are open. |
| `run.encounter.select` | Run identity, room/family, selection ordinal | Composition/history laws and stream split unresolved. |
| `run.encounter.hp` | Run identity, room, enemy semantic ordinal | Exact call sites/domain split unresolved. |
| `run.reward.card-offer` | Run identity, reward-event identity | Source policy, history, retries unresolved per site. |
| `run.reward.relic-offer` | Run identity, reward-event identity | Tier/list depletion and retries unresolved. |
| `run.reward.potion-offer` | Run identity, reward-event identity | Flat versus rarity-first source distinctions required. |
| `run.shop.stock` | Run identity, shop visit identity | Proposal rejection/history laws required. |
| `run.event.outcome` | Run identity, event visit, effect site, attempt ordinal | Do not merge with event selection absent evidence. |
| `run.combat.<mechanic>` | Run and combat semantic identities plus mechanic scope | Only after reviewed assignment; do not silently widen current combat streams. |

Names/scopes are vocabulary only, not assigned IDs or claims every family is an
independent stream. Final registry must document stable semantic scope, call timing,
failure behavior, and intentional correlations. Never derive from slot, worker,
completion order, pointer, generated address or policy token. Stream isolation must
not erase within-offer, cross-room, history or without-replacement dependencies.
RNG exhaustion is an administrative fault with cursor provenance, never loss,
retry, or rollback.

### 3.3 Proposed effect-site policy record (draft, not approved)

Every producer inventory entry should use one record per **effect site**, not one
row per card/relic/potion name. This documentation schema is deliberately explicit
about evidence state: an open fact cannot be filled by a runtime default. Its type
and field names are proposals for the whole-goal review, not Rust declarations.

| Record field | Proposed type / requiredness | Semantics |
|---|---|---|
| `site_id` | `EffectSiteId`, required | Stable authored identifier, e.g. `reward.ordinary_card_slot`; not an object address, slot, or dynamic counter. |
| `producer` | `ProducerRef`, required | Tagged content/effect owner and local effect-site label; nested producers identify the inner producer. |
| `route` | `ProductionRoute`, required | One of `TemporaryGeneration`, `PermanentReward`, `ShopProposal`, `Transform`, `FixedCreation`, `Copy`, `PotionOffer`, `Acquisition`, `MovementOrSelection`, or a separately reviewed route. |
| `result_kind` | `ResultKind`, required | Card/potion/relic offer, fixed instance, copy, transformed instance, resource mutation, or selection/movement. Prevents offers and acquisitions from being conflated. |
| `context_basis` | `ContextBasis`, required | Tagged `CurrentCharacter`, `Colorless`, `FixedContent`, `ExistingInstance`, `RouteSpecific`, or `Unresolved`; states how run character/current instance participates. `Unresolved` is documentation-only and not runnable. |
| `eligibility` | `EvidenceField<EligibilityLaw>`, required | Exact support, filters/exclusions, source character/route modifiers; a count alone is not an eligible-ID set. |
| `selection_law` | `EvidenceField<SelectionLaw>`, required | Tagged flat support, rarity/category then identity, weighted proposal, fixed result, source-specific law, or open. Carries weights/order and conditional dependencies when established. |
| `duplicate_policy` | `EvidenceField<DuplicatePolicy>`, required | Distinguishes no duplicates, retry identity retaining category/rarity, retry complete proposal, allow duplicate, source-defined alternative, and open. “No duplicate rule” is not represented by a missing field. |
| `history_reads` | `EvidenceField<List<HistoryRead>>`, required | Named authoritative counters/pools read, scope, and selection-time snapshot. An evidenced empty list differs from unknown history dependencies. |
| `history_writes` | `EvidenceField<List<HistoryWrite>>`, required | Counter/pool mutation and exact semantic boundary (proposal, offer, claim, effect, or later transition), ordering, and once-only key. |
| `rng_use` | `EvidenceField<RngUse>`, required | `NoRandomness`, or candidate domain/scope, draw boundary, cursor relationship, rejection/consumption behavior, and intentional dependency references. It is not inferred from the route name. |
| `construction` | `EvidenceField<ConstructionLaw>`, required | Output identity, upgrades/cost/scaling, copy provenance, instance creation and any hook ordering. Explicit `NotApplicable` must be evidenced. |
| `destination` | `EvidenceField<DestinationLaw>`, required | Offer, temporary pile, permanent deck, belt/slot, resource, or source-defined destination; includes full/blocked handling. |
| `commit_boundary` | `EvidenceField<CommitBoundary>`, required | When durable mutation becomes authoritative; distinguish generation, selection, confirmation, acquisition, and later transition. |
| `continuation` | `EvidenceField<ContinuationContract>`, required | Typed resume target/captured parameters/choice progress, or evidenced no-continuation. |
| `blocked_behavior` | `EvidenceField<BlockedBehavior>`, required | Exact source-defined skip/refusal/fallback if established; otherwise explicitly open. No invented retry or dropped effect. |
| `evidence` | Nonempty `List<EvidenceLocator>`, required | Independently worded locator, evidence grade, claim coverage, and limitations; never a raw source excerpt. |
| `readiness` | `PolicyReadiness`, required | `DraftIncomplete`, `ReviewRequired`, or `ReviewClosed`; only the designated whole-spec review can close a record for implementation. |
| `notes` | `List<String>`, optional | Rationale that does not replace any required field or fact status. |

Proposed fact wrapper: `EvidenceField<T>` is exactly one of `Established(value,
grade, locators)`, `Partial(known_values, open_subfields, locators)`, or
`Open(question, locators)`. `Established` may also state `NotApplicable` only with
supporting evidence. `Partial` cannot be consumed as if its open fields had defaults.
Evidence grade is one of `StaticSource`, `BytecodeConfirmed`, `ModelChecked`,
`OriginalClassExecuted`, or `OriginalRuntime`; this task supplies no new
`OriginalClassExecuted`/`OriginalRuntime` evidence. A candidate RNG assignment is a
design proposal and must be labeled separately from a source-evidenced RNG fact.

**Fail-closed authoring/consumption rule:** the inventory may contain
`DraftIncomplete` rows, but the finalized content catalog cannot silently consume
them as a policy. Whole-spec catalog preflight rejects any missing required site,
unresolved required law, or site with readiness other than `ReviewClosed` as a
content/specification error before run initialization. If a supposedly closed
policy lookup is absent or malformed after initialization, raise an administrative
content fault with the site and provenance; never fall back at runtime to another
character, pool, stream, fixed card, skip, reroll, or defeat. This is a proposed
fail-closed contract, not a product implementation.

### 3.4 Illustrative site records and domain relationships

The following rows demonstrate how to fill the proposed record without pretending
all source laws are known. Proposed domain/scope values are **candidate design
assignments only**. A shared candidate cursor is not a claim about original RNG
correlation; domain assignments require review against RNG-001 and the final source
law table.

| `site_id` | Evidence-established fields | Candidate domain / scope and draw boundary | History/correlation; explicit open fields |
|---|---|---|---|
| `reward.ordinary_card_slot` | Ordinary permanent-card support is 72 Ironclad identities without the combat-generator HEALING exclusion. A qualifying Shard reward uses rarity-only support 320 including ordinary colorless identities; reward history changes once per generated slot before fixed-rarity identity retry. These are source-grounded, bounded acquisition findings, not every reward helper's law. | Candidate `run.reward.card-offer`, scope `(run_identity, reward_event_identity)`; one cursor is proposed across slots generated by that reward event, with slot order as draw history rather than a scope key. Identity rejection occurs inside the already-selected rarity. Whether rarity and identity share this cursor is **unresolved**; different reward events with different scope identities have distinct derived cursors under this proposal. | Reward-slot history and intentional sequential offer dependencies are not replaced by independent per-slot sampling. Exact complete eligibility IDs/conditional masses for each distinct reward source, all modifiers, exact stream/cursor relation and nonordinary helper routing remain **open**; do not generalize this row to Neow, Dream Catcher, Orrery, or Tiny House. |
| `shop.card_stock_proposal` | Static acquisition findings distinguish merchant rejection of a second Attack/Skill as whole-proposal retry (category/rarity and identity are proposed again), and common Power proposal fall-through to uncommon. This is not reward fixed-rarity identity retry. | Candidate `run.shop.stock`, scope `(run_identity, shop_visit_identity, stock_generation_phase)`; one ordered cursor is proposed within a phase across proposals/retries, not per slot. Whether initial stock and restock use the same phase/cursor is **unresolved**. | Ordered stock proposals and retry consumption are candidate intentional within-visit correlation. Whether restocks, relic modifications, and other merchant random choices share the cursor/domain is **unresolved**. Exact prices, every slot/rejection law and complete source callbacks remain outside this example. |
| `combat.hello_world.common_offer` | Reviewed generation findings identify the current character's Common pool as the basis for Hello World; on Ironclad it is not Shard's expanded reward pool and is not inferred from a card's printed color. This establishes pool basis only, not the complete selection law. | Candidate `run.combat.card-generation`, scope `(run_identity, combat_identity)`; draw boundary is when the turn-start offer is generated, before it is inserted into the hand. Per-turn/per-site substreams are **not** silently introduced. | Candidate shared cursor with other random combat card-generation sites in that combat is an explicit proposal to preserve ordered consumption, **not established source correlation**. Exact eligible IDs/masses, repeat/copy and turn history, exclusions, cursor sharing with Distraction/Infernal Blade/other producers, and all output/commit hooks remain **open**. Each site retains its own policy even if a future review selects a shared cursor. |
| `map.room_bucket_assignment` | Reviewed map findings identify a retained shuffled room bucket and row-major assignment/fallback path; a separate reviewed supplied-choice witness reaches an empty elite set. This does not prove initialized-root-seed realizability or establish an input measure. | Candidate `run.map.generation`, scope `(run_identity, map_generation_identity)`; candidate draw boundary is bucket shuffle before deterministic assignment/fallback, not a fresh draw per cell. A shared map-generation cursor is proposed but not source-established. | Bucket order and assignment/fallback are history-dependent; do not replace with independent per-row draws or assert a uniform permutation. Whether topology and room assignment share this cursor, semantic input measure, zero-elite behavior, root-seed realizability, and general/larger shuffle laws remain **open**. No repair/retry/fallback is selected here. |

Evidence locators for the established portions of these examples:

* `2026-10-18-ironclad-a0-act1-acquisition-followup-findings.md` — ordinary and
  Shard rewards; merchant retry.
* `2026-10-18-ironclad-a0-act1-generation-context-design.md` — Hello World
  current-character basis.
* `2026-10-18-ironclad-a0-act1-map-java-findings.md` — map bucket assignment.
* `2026-10-18-ironclad-a0-act1-map-random-followup-findings.md` — supplied-choice
  zero-elite witness and measure boundary.

These locators support only the stated fields, not the proposed domain assignments
or unresolved claims.

The candidate scope intentionally uses one semantic visit/phase identity for ordered
site activity where shared consumption is being considered; it does **not** create
one stream per card/source/slot by default. Conversely, similarly named routes are
not thereby assigned one universal law: the source-policy record remains distinct
per effect site. Until main reviews domain granularity and evidence coverage, the
candidate assignments above are documentation examples only and cannot be used as
runtime defaults. In particular, no uniform shuffle measure, universal stream
independence, or exact source callback closure is asserted.

## 4. Choice/payment/history and suspended continuations

An accepted action resolves automatic effects until the next genuine player choice,
death, or complete Act-1 handoff; no strategic option is auto-picked. UI-only clicks
may be collapsed only if legal choices, commitment, ordering and RNG timing are
preserved. For each choice, validate current token and exact menu membership, then
follow that source's payment/commit order. There is no universal pay-before-effects
or pay-after-effects rule. Energy, gold, HP, keys, selection consumption, counters,
offer generation and history updates each follow their own site contract.

A typed continuation records source, parent resume point, selected instance IDs in
order where order matters, remaining count/eligibility, captured X/cost, and
commit/cancel semantics. Forethought+ is an ordered sequence, not an unordered
subset. Nested combat/events/rewards/relic selections resume the correct parent;
never reroll or pay twice on resume, reveal early, discard parent state, or apply an
effect twice. Death precedence is source-specific. A generated offer is not an
acquisition; counters/history may update at offer generation. Key flags can commit
when the effect resolves, not when offered. Note incoming obtain/hooks, outgoing
removal, and later transition persistence are separate boundaries.

History is authoritative game state, not reconstructed from logs or accepted
rewards. Apply each change once at its source-defined boundary. Static findings
identify selection-time encounter history, reward rarity mutation before identity
retry, relic consumption even when skipped, and separate Note persistence timing;
these are examples, not a complete source table.

### Detached profile/reset

`RunCreationProfile` is an owned value, fully validated before entropy draw, episode
allocation, or installation. It holds only supported rule-relevant prior-run facts;
it is not an arbitrary starting deck/resource override or imported save. The run
captures a detached copy. Do not choose disputed defaults for previous boss reach,
boss-seen history, or stored Note card in this component draft; support all variants
legal under final rules and reject invalid values without fallback/RNG consumption.

Vector reset consumes only explicit request/configuration and its declared seed /
partition sequence. A prior slot's death, Note event, or profile effect cannot mutate
another reset's input. `ProfileEffects` is a possible detached caller result, but
whether it is exposed at event resolution or only at later persistence transition is
unresolved. Note evidence distinguishes direct incoming acquisition/hooks,
outgoing removal, and NOTE_CARD/NOTE_UPGRADE persistence on next-room transition;
this draft does not collapse those boundaries or pick death/fault output semantics.

## 5. Proposed native API/results

| Operation | Input | Result/contract |
|---|---|---|
| `RunPool::new(config, descriptor)` | Batch size, worker count, partition, reset mode, opt-in recording | Checked construction or setup/input error. |
| `reset_batch(indices, requests)` | Unique slots and owned profile/seed request per row | Stage/validate requests and reservations before installing runs; return initial decisions, private tokens, detached info. |
| `read_batch(indices)` | Slot indices | Owned decisions; no RNG/state/token advancement. |
| `step_batch(indices, tokens)` | One local candidate index/token per configured slot | Current rows, rewards/end flags, compact final rows and metadata. Preflight every row before any transition. |
| `replay_from_start(record)` | Versioned creation inputs, seed/partition, accepted candidate choices | Recreate through normal initialization and replay legal choices; no arbitrary live-state import. |
| `close()` | None | Quiesce/release pool state; already delivered arrays remain valid. |

A decision is public observation + canonically ordered legal candidate table + opaque
`DecisionToken` outside learned features. Each candidate privately maps to one exact
core choice. Submission is a slot-local row index; reject cross-slot/global packed
offsets, stale token, wrong phase, absent menu member or out-of-range index. Integer
facades cannot detect stale caller intent if the old index is legal in a new menu;
do not promise universal stale-intent detection.

For same-step autoreset, capture owned final observation/resources/outcome/identity
before installing replacement. Current rows describe current/replacement episodes;
reward/end flags and final data describe the stepped episodes. Never apply submitted
action to replacement. Manual terminal slots remain terminal until explicit reset.

## 6. Packed numeric tables

The approved direction is fixed-width numeric rows in variable-length packed tables.
Draft descriptor per table: stable table ID, width `W`, `int64` values, column/code
meanings, schema version, optional aligned knowledge-mask width `K`.

* `values_t: int64[M_t, W_t]`, C-contiguous/native endian.
* `offsets_t: int64[B+1]`, start 0, end `M_t`, nondecreasing.
* Optional `known_t: int8[M_t, K_t]`, row-aligned, only 0/1.

Slot `i` owns `[offsets[i], offsets[i+1])`; row length is the offset difference,
not a second mutable length field. Empty segments have equal offsets. Cross-table
references are typed slot-local row numbers; absent is `-1`. Convert to global
buffer index only at transport boundary and validate within the slot's segment.
Offsets, tokens, handles and worker order are not features.

Candidate table-family proposal: permanent cards, combat cards, relics, potions,
map nodes, public history, room/selection choices, enemies, enemy powers, candidates,
and candidate parameters.
Bounded player/map/context fields may be fixed arrays with explicit occupancy and
knowledge masks. Family names do not authorize exposing hidden objects. Variable
piles, histories, offers, selections, enemies and candidates are not padded to
guessed limits.

Draft minimum candidate row: `(candidate_kind, source_table_kind, source_local_row,
target_local_row, parameter_start, parameter_count)`; absent references `-1`.
`candidate_parameters` is a second packed table with its own `[B+1]` offsets and
descriptor-declared fixed-width numeric rows; `parameter_start/count` are local to
the slot's parameter segment. This lets ordered selections be represented without
enumerating every subset. Final column meanings/vocabularies remain whole-spec work.
No legal choices may be merged because their names match.

All row counts, widths, bytes, offsets, conversions, references and allocations use
checked arithmetic. No legal-state clamp, truncate, filter, policy page, or gameplay
cap. Numeric/allocation/resource exhaustion is administrative: preserve already
committed effects, quiesce and fault the pool, no fake loss/truncation, retry,
rollback, or partial success. `int64` is a draft transport choice, not proof every
source numeric law fits; finalize exact field bounds/representations before approval.

## 7. Observation, legality and privacy

A separately versioned run descriptor fingerprints ordered fields, vocabularies,
dtypes, widths/bounds, table/offset rules, masks, ordering, action carrier, and
terminal/final contract. Schema meaning/layout changes require a new reviewed
identity. The final schema must be self-describing rather than depend on private
Rust enum casts.

Rows may contain only presently public resources/inventory, visible/revealed map
and visit facts, current room/offers/choices, current combat knowledge, and
legitimately retained revealed facts. Candidate rows contain exactly legal core
choices. Ordering depends only on public state and rule-significant visible order.
Different upgrades, costs, scaling, provenance, bottle state or choice meaning do
not become equivalent because card names match.

Never expose seed/root/key/cursor, hidden future encounters, unrevealed question
mark/event identities, unoffered pools, future shuffle order, hidden Match-and-Keep
faces, hidden Runic Dome intent/damage, unobserved counters, instance IDs, worker or
slot allocation, or diagnostic/replay internals. Table length/order are public too.
Masks distinguish absent, present-unknown and known-zero. Public-equivalence
review compares fixed fields, table lengths/order, masks, references and candidates,
not just a flattened vector.

Legality requires exact dtype/shape/range, readiness/phase, token and menu
membership for all rows before dispatch. No coercion of bool/float, clamp, modulo,
invalid-row dropping or no-op mapping. Reads do not advance provenance. Accepted
transitions rebuild affected decisions and issue new tokens. Action carrier and
Gymnasium presentation remain open whole-spec decisions; never allocate a dense
mask/head proportional to a machine-range carrier.

## 8. Replay, batch atomicity and administrative faults

Opt-in run replay is from-start, not snapshot restoration. Record detached creation
profile, seed/root adapter and partition, semantic run identity, rules/content/
schema/task/RNG identities, and accepted semantic candidate sequence. Tokens are
not replay identities. Validate record and recreate through ordinary construction,
then apply the accepted prefix through the ordinary core menu. No arbitrary state
import, later-mutated caller profile, hidden process history, or fabricated
external allocation/packaging fault. Mismatch is a provenance-bearing error.

Before dispatch, validate every action/token/slot/phase/readiness and reserve
metadata/resources whose failure must be atomic. Any malformed row rejects the
whole batch without state, RNG, token, episode identity/ordinal, profile or history
advancement. This is not rollback after gameplay dispatch.

Each slot exclusively owns mutable state. Assign semantic identities before
parallel work; use persistent pool-private workers or direct native iteration for
one worker. Publish in logical slot order, never completion order. Worker count and
neighboring slots cannot affect a run's choices, observation or RNG progression.

On post-dispatch core/encoding/allocation/binding/export fault: quiesce all workers;
retain committed mutations and first-fault slot/stage/cause/RNG provenance; emit no
fabricated successful batch; fault the pool. No retry, rollback, silent reset,
defeat conversion, or repeated action after export failure. Keep operation lease
through final/table packing and facade delivery to attribute failure to this
delivery. Detached arrays/finals survive later step/reset/close. Reset can stage all
requests before installation; true runtime-construction failure must preserve the
same no-rollback/provenance contract.

## 9. Fidelity blockers/options (not silently selected)

1. **Zero elite/root seed:** a legal supplied-choice map witness reaches an empty
   elite set. Root-seed realizability and native empty-set behavior remain open. Do
   not invent elite insertion, retry/regeneration, node deletion, or loss.
2. **Input measure/larger shuffles:** three-item seeded-shuffle counts are conditional
   on uniform64 input. Initialized/history-conditioned measure and larger relic,
   face and board shuffle laws are not settled. Do not assume uniform permutations.
3. **Reaper/headless timing:** static order suggests possible retaliation/heal
   interruption, but no frame/timer outcome was observed. No universal drain or
   lost-heal result is chosen.
4. **Callback/cost closure:** global origins, nested costs, callback closure, alternate
   terminal paths and transitive reactions remain partial. Typed requirements do not
   certify all laws.
5. **Note profile boundary:** incoming hooks, outgoing removal, saved live reference,
   and later transition persistence differ. Exposure boundary and death/fault result
   are open; reset never implicitly shares the result.
6. **Source policy inventory:** current-player authority/no fallback are constraints;
   exact eligible IDs, conditional masses, duplicate/history, construction, commit,
   and terminal behavior are not closed for every site.
7. **Profile/numeric/facade choices:** disputed defaults, exact descriptor fingerprint,
   all enum/bounds, candidate carrier, and Gymnasium presentation await whole-spec
   review.

## 10. Future acceptance obligations (not executed)

The integrated plan should specify table-driven exact expected state, RNG domain/
cursor, public rows, candidates, and outcomes for at least:

| Area | Required discriminator |
|---|---|
| Authority/copy | Permanent mutation updates one run authority; temporary copy/upgrade does not mutate its permanent source; captured X/origin survives resume. |
| Effect sites | Same off-color producer under explicit characters follows its policy; colorless/fixed/copy/transform/reward/potion laws remain distinct; absent policy faults, never defaults. |
| Payment/history | Offer vs claim, skipped relic consumption, rarity-before-identity retry, key commit, Wheel reveal/confirmation, Note transition persistence each occur once at their evidenced boundary. |
| Continuations | Ordered Forethought; nested combat/event/relic return; death interruption; boss reward/relic handoff; +1 only when all work resolves. |
| Atomicity/legality | Invalid last row, stale/cross-slot token/index, negative/one-past-end, bad dtype/shape, terminal action and absent candidate leave all rows unchanged. |
| Tables/visibility | Empty/mixed segments, offsets/local refs, growing legal rows, overflow fault; hidden-state changes preserve public projection, legitimate reveal changes it. |
| Replay/reset | Detached profile; from-start prefix reproducibility; no cross-slot profile inheritance; identity mismatch rejects rather than substitutes. |
| Fault/delivery | Partial durable effects persist on fault, pool quiesces and emits no success; export fault attributes current delivery; old/final arrays stay detached. |
| Compatibility | Existing combat-v1 APIs/identities unchanged; run observation/action/replay identities independent. |

These are future obligations, not tests run. No implementation, build, benchmark,
original-game execution, or runtime probe was performed for this report. Whole Act 1
written-spec approval, coordinated plan, separate start approval, fresh independent
review, and the unresolved fidelity items above remain in force.
