# Ironclad A0 Act 1 — native consumer schema and runtime contract

**Status: phase-one consumer contract for independent review. Producer joins named in §12 are intentionally unresolved until main routes the exact independently reviewed support revisions. This document is not a claim of complete source-member equality, a runtime codec, whole-spec approval, or implementation authorization.**

Normative model: [the accepted consolidated specification at main merge `2e090d78297a236bfd549797e460db40d47e709f`](../superpowers/specs/2026-10-19-ironclad-a0-act1-consolidated-design.md), especially §1.2, §3–4, §9–13 and §17. Its policy choices control over any older candidate herein cited: independent reproducible native RNG systems and mechanic-conditioned intended weights; source-informed representative valid maps with at least one Elite and deterministic reject/regenerate; deterministic semantic settlement with immediate mandatory causal work and final-kill Reaper healing; no elapsed-only wait; experienced independent profiles with Note Iron Wave/0 default and no automatic inheritance or required staged export; initialize Act 2 and stop before its first player decision. Source correlation/root-history ensembles, source map/zero-Elite parity proofs, frame-clock timing, and mandatory staged Note export are not requirements.

This specification defines the consumer schema and adapter/core boundaries needed to bind the accepted model. A producer import is complete only when its main-routed reviewed document closes the exact keyed relation stated in §12 and the private equality checks pass after integration. Symbolic imports below are integration interfaces, not opaque runtime fields and not evidence of completion.

## 1. Authority, versions and invariants

`spire-core` is the sole authority for rules, run state, action legality, effect ordering, source-informed content semantics and native RNG. `spire-env` owns episodes, slots, public projection, candidate indexing, reward, batching, task/fault provenance and replay orchestration. `spire-python` owns checked numeric transport, detached facade values and error conversion. The policy chooses only an index in the current published legal candidate segment; it cannot submit callbacks, edit parameters, inspect hidden state, or repair a rejected action.

One mutable `RunState` owns persistent resources. Combat is a temporary overlay using the same combat engine as standalone combat and borrows persistent resources through a revision-checked context; there is no second writable HP/deck/gold/relic/potion/key authority. Applied durable effects remain applied through later death or administrative fault; queued effects are not committed effects. Private identity, ordering, cursor and diagnostic data never enter learned features. Invalid preflight changes no state, RNG, reward, identity ordinal or token. Post-dispatch failure is administrative, preserves committed effects, quiesces the pool and emits no fabricated successful result.

All integer arithmetic in the carrier, spans, lengths, allocations and IDs is checked before the associated mutation. Carrier exhaustion is administrative, never a legal gameplay cap, truncation, reroll, defeat or rollback. A source-specific wrap, cap, sentinel or rounding rule applies only at its named owner and operation. No frame clock, elapsed-time state, wait opcode, source-seed parity or source-correlation field is introduced solely to retain a historical proposal.

New run identities are separately versioned from unchanged standalone identities: `spire-lab-rng-hmac-sha256-v1`; `snapshot2`; `spire-lab-combat-reference-v2`; `spire-lab-env-seed-sha256-v1`; `spire-lab-combat-observation-v1`; combat fingerprint `8fd66d212259d2bdd27139988a772e1a6c223f8d65524f38c74174a0d0843357`; and `hmac="=0.12.1"`. Run schema, action, catalog/site registry, RNG bindings, task, delivery and replay each have their own versioned identity. No new final fingerprint is claimed until all frozen descriptor and producer inputs exist.

## 2. Type, numeric and collection grammar

The following names denote exact transport-neutral types; widths and bounds are normative unless marked **producer join open**.

| Type | Definition |
|---|---|
| `U8/U16/U32/U64` | Unsigned fixed-width integer; big-endian in canonical bytes. |
| `I8/I16/I32/I64` | Two's-complement signed fixed-width integer; big-endian in canonical bytes. A Java `int` maps by exact signed-i32 value and sign-extension into i64 carrier; no narrowing. |
| `U128` | Exactly 16 unsigned bytes, high-order byte first; used only for caller stream identity. |
| `Hash32` | Exactly 32 raw SHA-256 bytes. |
| `Token32` | Exactly 32 opaque provenance bytes; never an action meaning or learned feature. |
| `Bool` | Exactly false/true, encoded 0/1; all other numeric values reject. |
| `Bytes` | `U64(byte_length) || exact bytes`; length and enclosing remaining bytes checked before allocation. |
| `Text` | `Bytes(strict Unicode-scalar UTF-8)`; no NUL terminator, normalization, surrogate, BOM or alternate spelling. Authored identifiers are nonempty ASCII with namespace-qualified owner. |
| `Vec<T>` | `U64(count)` then exactly `count` canonical `T` values; no guessed content cap; checked representational/allocation ceiling only. |
| `Option<T>` | tag `0` absent; tag `1` followed by exactly one `T`; no other tag. Absent differs from present-empty. |
| `Enum<K>` | Closed, explicitly versioned codebook `K`; codes are not compiler or source enum ordinals. Unknown codes reject. |
| `Handle<K>` | `(run_identity:RunIdentity, kind:Enum<K>, ordinal:U64)`; ordinal is checked monotonic and never reused within a run, including after removal. Exhaustion faults administratively; no truncation or alias. Handles are private and never serialized to observations. |
| `RowRef<T>` | Internal and canonical form is `(table_id:I64, slot_local_row:I64)`. The one absent value is `(0,-1)`; present refs use table IDs `1..18` and local row `0..I64_MAX-1`. For typed single-cell observation fields the table ID is descriptor-fixed and only `slot_local_row` is carried (`-1` means absent); generic reference columns carry the explicit `(table_id,local_row)` pair. Validate slot-local row before any global offset conversion. |
| `F32Bits` | Exact raw IEEE-754 binary32 payload. Each owning source field must specify permitted finite/special values and the operation boundary at which binary32 rounds. No decimal or binary64 round-trip. No public float field is enabled without a closed owner/domain/visibility row. |
| `KnowledgeCell<T>` | `(known:Bool, value:T)`; unknown requires canonical zero/empty payload. Known zero differs from unknown. Absent is represented only by the field's declared `Option`, never inferred from mask zero. |
| `OrderedSet<T>` | `Vec<T>` with unique members, preserving rule-significant source order; duplicate member is invalid. A field explicitly declared multiset preserves multiplicity. |
| `Span` | `(start:I64,count:I64)` within the same addressed slot's variable table; both nonnegative and checked addition, no cross-slot address. Since offsets are signed i64, a slot table segment has at most `I64_MAX` rows and its final valid row is `I64_MAX-1`. |
| `SignedSourceInt` | Exact signed-i32 member value where its producer proves Java `int`; otherwise a field-qualified exact wider type. Java arithmetic wraps only where that actual operation wraps. |
| `Count/Ordinal` | U64 carrier value bounded by the actual finite collection/menu at that boundary; no fixed content limit. Source list-size/index has source-i32 semantics only when the owning producer establishes it. |
| `SettledPrice` | Tagged `Gold(I32)`, `HpLoss(I32)`, `Damage(I32)`, `MaxHp(I32)`, or owner-registered `Composite(ordered Vec<PriceComponent>)`. Composite is unemittable until every component owner/member/domain/visibility join is closed. Source float intermediates are not settled prices. |

Closed API enums: `Partition={Train,Selection,Final,Test}` with wire codes `{1,2,3,4}`; `ResetMode={SingleManual,VectorSameStep}`; `RecordingMode={Off,FromStart}`. Unknown codes reject. `RunIdentity=(stream:U128,episode:U64)`. `RunPhase={Creation,Neow,MapChoice,Room(RoomHandle),Combat(CombatHandle),Decision(DecisionKind),Suspended(ContinuationHandle),Dead(DeathRecord),Act2Ready(TransitionCheckpoint)}`. `TerminalOutcome={Death,Act1CompleteAct2Ready}`. These are environment-owned lifecycle values, not casts of source enums.

Errors distinguish malformed input, stale/foreign provenance, source-rule refusal (only where the current menu exposes a legal refusal choice), and administrative failure. A source-law unknown is a spec/catalog preflight error, not a default branch. An invalid value never reaches source rule execution or consumes entropy.

## 3. Versioned public API and facade

These are normative logical operations; language bindings may use different names but not different semantics.

| Operation | Input and preflight | Result / effects |
|---|---|---|
| `create_pool(config, descriptor)` | `PoolConfig(batch_size:U64>0,worker_count:U32>0,partition:Partition,reset_mode:ResetMode,recording:RecordingMode)` plus exact descriptor/catalog/site identities; validates all configuration and frozen contracts. | `ReadyPool` with all slots `Uninitialized`; no run, seed consumption or episode ordinal allocation. |
| `reset_batch(indices, requests)` | Nonempty ascending unique slot indices; one owned `ResetRequest(profile,seed_request)` per row. Validate all rows, profile/member domains, requested seed variant, next episode ordinals/run keys and resource reservations before entropy, ID allocation, abandonment or installation. No step token/menu/action predicate applies to reset. | Detached initial observations/info only for addressed rows. Invalid row rejects the whole batch with no mutation. Live reset means explicit administrative abandonment, not death/truncation/reward. Unaddressed slots unchanged. |
| `read_batch(indices)` | Nonempty ascending unique valid initialized slot indices; Ready pool; no action/token needed. Faulted/Closed/Operating pools reject. | Detached current decision and metadata; does not allocate/renew decision token, advance semantic RNG, run logic or allocate episode identity. Delivery bookkeeping may advance only its private operation ordinal. |
| `step_batch(indices, actions)` | Exactly the configured live slots for the step call, one `StepAction(candidate_index:U64,token:Token32)` per addressed row. Validate exact dtype/shape, readiness, live phase, current decision generation/token, candidate bounds and private core-choice membership for every row before any dispatch. | One ordered result per logical slot. Resolve automatic rules to next genuine decision, actual death or completed Act1 handoff; never auto-select strategy. No turn/step/time cap, frame/wait action or policy callback. |
| `replay_from_start(record)` | Complete versioned record validated in full before construction. | Recreates normally from creation input and deterministic native RNG contract, checks each public/private binding and applies only accepted semantic candidates through regenerated current menus. Never restores arbitrary live state. First mismatch is a provenance-bearing replay error. |
| `close()` | Pool not concurrently operating; idempotence/error code is versioned. | Quiesces/releases pool-owned state and workers. Previously returned arrays and final snapshots remain owned and valid. |

### 3.1 Reset, seed and identity records

`RunIdentity=(caller_stream:U128,episode_ordinal:U64)`. It is independent of slot and worker. `SeedRequest` is exactly one of `Root32(bytes[32])`, `Seed64(U64)`, or `Entropy32` (captured 32 system-random bytes once, only after all-row preflight). Seed64 derivation bytes are `UTF8("spire-lab/act1/seed/candidate1") || 00 || U64_BE(seed)`, SHA-256 output is the root. Root32 is used byte-for-byte. Entropy failure is an administrative reset fault; no replacement seed is silently generated. Episode key is HMAC-v1 over the canonical UTF-8 string `a1:<32-lowercase-hex-stream>:<canonical-decimal-ordinal>`. Partitions are frozen codes `1 train, 2 selection, 3 final, 4 test`. Run seed domains and site/site-state bindings are the native RNG producer import in §12; the above envelope does not assert any source seed mapping or completed producer implementation.

Reset modes are `SingleManual` (exactly one slot; a terminal state remains readable with zero candidates until explicit reset) and `VectorSameStep` (step captures retired final state before installing a replacement in the same addressed slot). A vector step's returned live observation describes the replacement; reward/terminated/truncated/final describe the retired episode. The submitted action is never applied to the replacement. Reset never imports a previous slot's Note output or profile mutation. Every episode gets its own resolved creation profile.

### 3.2 Python numeric facade

The binding accepts built-in `int` or NumPy `int64` scalar for a scalar candidate index; `bool`, float, other integer dtype, list and scalar arrays reject. Vector input is a native-endian contiguous NumPy `int64[B]` exactly matching addressed rows; no implicit conversion, clamp, modulo, list coercion or global packed offset. Native transport is fixed-width: `int64[M_t,W_t]` values, same-shaped `int8[M_t,W_t]` known mask containing only 0/1, and `int64[B+1]` offsets. For each table offsets start at zero, are nondecreasing, end at M, count rows, and each slot owns `[offset[i],offset[i+1])`. Every table exists; empty has shape `[0,W]`. Cross-table refs and candidate parameter spans are slot-local. All length/product/byte/conversion/allocation arithmetic checked. Legal table growth has no guessed bound, padding, paging, truncation or forced resampling.

Single observations are detached `Dict(fixed,tables)`; each table contains row-major values and aligned known mask. Vector outputs contain one detached segment per slot, not padded object-per-row arrays. Any Gymnasium space/wrapper projection is a separately versioned facade profile; it cannot alter native candidate meaning. No dense mask proportional to the maximum carrier is permitted. Exact third-party wrapper acceptance remains a future test, not a product result claimed here. Reference row values use signed i64 throughout: absent `-1`, present `0..I64_MAX-1`, with target table fixed by descriptor for typed cells or carried in the adjacent table-id cell for generic pairs. Before packing, reject any segment count or cumulative offset above `I64_MAX` as an administrative representational overflow; never cap game content, wrap, truncate, or resample. `count=I64_MAX` is representable; `count=I64_MAX+1` is not because the exclusive end offset cannot fit `int64`. Thus the maximum valid local row is `I64_MAX-1`.

## 4. Public observation descriptor

The corrected schema identity is `spire-lab-act1-observation-candidate5` (candidates 2–4 are superseded by Appendix A). Descriptor bytes use the exact canonical grammar in Appendix A.4, excluding the descriptor's own digest. It freezes table IDs/column order, wire type, codebooks, absent/unknown semantics, typed power-owner facade conversion, reference allowlists, ownership/visibility predicate IDs, canonical ordering, task/final semantics, facade profile and versioned content/site registry digests. A semantic or layout change increments the identity; no final fingerprint is emitted in this phase while producer catalogs/visibility rows are unresolved.

Fixed `context` row has exactly 12 `i64` values plus 12 `i8` known cells, ordered: `phase, decision_kind, act, global_floor, current_node_ref, turn, terminal_outcome, parent_context_kind, selected_count, min_select, max_select, ordered`.

Fixed `player` row has exactly 16 values plus 16 known cells, ordered: `character, ascension, hp, max_hp, gold, block, energy, max_energy, draw_per_turn, potion_capacity, ruby, sapphire, emerald, stance, orb_capacity, mantra`.

All 18 variable tables always exist. Every column is one signed i64 carrier cell with aligned known bit unless its descriptor version explicitly changes that column. The table ID and column order below are normative:

| ID | Table | Width | Ordered columns |
|---:|---|---:|---|
| 1 | `nodes` | 7 | `x,y,room_kind,visited,reachable,burning_marker,boss_code` |
| 2 | `edges` | 3 | `from_node_ref,to_node_ref,currently_selectable` |
| 3 | `permanent_cards` | 9 | `card_code,upgrades,misc,bottle_kind,removable,upgradeable,base_cost_kind,base_cost,public_position` |
| 4 | `combat_cards` | 12 | `card_code,upgrades,misc,zone,position,cost_kind,base_cost,turn_cost,public_owner_ref,retained,exhaust_on_use,ethereal` |
| 5 | `relics` | 4 | `relic_code,public_position,counter,used` |
| 6 | `potions` | 4 | `belt_slot,occupied,potion_code,potency` |
| 7 | `actors` | 8 | `enemy_code,formation_position,hp,max_hp,block,intent_code,intent_damage,intent_hits` |
| 8 | `powers` | 5 | `owner_table,owner_ref,power_code,amount,public_position`; owner facade admits only Player `(0,0)` or live actor `(7,row)`; facade conversion is not RowRef conversion |
| 9 | `orbs` | 5 | `position,orb_code,passive_amount,evoke_amount,counter` |
| 10 | `offers` | 9 | `offer_kind,content_code,target_table,target_ref,amount,price_kind,price_amount,status,public_position` |
| 11 | `room_objects` | 7 | `object_kind,public_position,face_state,content_kind,content_code,object_state,remaining_attempts` |
| 12 | `selection` | 5 | `source_table,source_ref,selected_ordinal,selected,eligible` |
| 13 | `knowledge` | 6 | `fact_kind,subject_table,subject_ref,position,value_kind,value` |
| 14 | `properties` | 8 | `owner_table,owner_ref,property_code,ordinal,value_kind,value,ref_table,ref_row` |
| 15 | `history` | 7 | `fact_kind,ordinal,subject_kind,subject_code,value_kind,value,visible_floor` |
| 16 | `candidates` | 8 | `action_kind,source_table,source_ref,target_table,target_ref,option_code,parameter_start,parameter_count` |
| 17 | `candidate_parameters` | 6 | `parameter_kind,ordinal,value_kind,value,ref_table,ref_row` |
| 18 | `relations` | 5 | `relation_kind,from_table,from_ref,to_table,to_ref` |

Widths sum to 118 table columns; with context and player there are exactly 146 carrier cells. This arithmetic checks shape only, not source-member equality.

### 4.1 Field-domain, owner and visibility contract

Field semantics are column-qualified, never inferred from a row-wide integer type. Public descriptor validation binds each column to one `FieldContract`:

`FieldContract=(field_key,wire_kind,legal_domain,unit,sentinel_set,source_owner,read_members,write_members,mutation_boundary,rounding_rule,overflow_rule,visibility_predicate,owner_lifetime,carrier_mapping,producer_join_id)`.

For `carrier_mapping`, exactly one is required: fixed cell; named table column; `properties(owner_table,owner_ref,property_code,ordinal)` with a frozen property code and exact owner-qualified domain; or explicit `NotPublic(reason,proof_locator)` / `NotRelevant(reason,proof_locator)`. One source member cannot silently map to multiple carriers; each public carrier has exactly one semantic owner. A source class/member set and carrier set must compare equal after explicit exclusions and property expansion; neither a row count nor an enum signature is sufficient.

| Field family | Consumer domain and units | Ownership/lifetime and source binding status | Visibility rule |
|---|---|---|---|
| `context.*` | Explicit codebook or source signed-i32 act/floor/turn; counts are actual active menu list lengths; `ordered` is Bool. `current_node_ref` is a typed node row or absent. | Adapter owns phase, decision kind, terminal, parent kind and selection descriptor; source owns act/floor/turn. Exact dispatcher/current selector producer import still open. | Only current phase/menu and public context; no continuation ID/token. Mask non-applicable combat/selection values rather than fabricate zero. |
| `player.character,ascension` | Character code Ironclad=1 for run; A0=0. | Chosen run profile and authoritative player state. | Public. |
| `player.hp,max_hp,gold,block,energy,max_energy,draw_per_turn,potion_capacity,orb_capacity` | Exact owner-qualified source values; no general clamp/cap. Each direct Java int is signed-i32. 999 applies only to the evidenced addBlock/addEnergy helper call, not all values. | Run player is sole persistent owner; combat borrows resource context. Setter/transition and actual getter/member joins remain producer-qualified where not already accepted. | Current public resources; omit/mask values that are not valid in current phase. |
| `player.ruby,sapphire,emerald` | Bool 0/1, acquired key flags. | Run settings/key state; emerald key is not node's burning designation. | Public after actual acquisition commit. |
| `player.stance,mantra` | Closed stance code; Mantra is owner-qualified signed-i32 amount, absent when owner power absent. | Player stance and exact player-owned Mantra power. Exact power identity binding is catalog import. | Public if current rules reveal; no hidden identity-bearing pointer. |
| `nodes.*` | Coordinates signed source i32; room kind enum; flags Bool; boss code owner-qualified key/absent. | Map state; node burning marker and acquired Emerald key are different fields. Node ordering `(y,x)`; edges sort `(from,to)`. Source equality and exact boss preview identity remain open. | Reachable/visited and burning marker emitted only at specific public checkpoint. Hidden future room identity/boss and unrevealed map content masked. Exact predicate imports from catalog/handoff. |
| `edges.*` | Local node references and Bool current legal traversal. | Map topology, no object pointers. Exact menu/current traversal owner/callback import remains open. | Only edges currently legally visible; `currently_selectable` exactly equals current menu predicate, not a guess. |
| `permanent_cards.*` | Card code; exact signed-i32 upgrades/misc/base cost; base cost kind ordinary `>=0`, X=`-1`, unplayable=`-2`; independent Bool flags and public ordinal. | Persistent card instance identity distinct from kind. Per-card upgrade/misc/cost domains and every writer/copy/capture owner require source-member joins. | Card identity and individual values only under exact entitled owner predicate; hidden configured Note card not exposed before legal reveal. Misc/bottle/cost may be masked independently. |
| `combat_cards.*` | Same exact kind/member domains; `zone` enum; position public ordinal or unknown; base and turn cost independently signed-i32; flags Bool. `exhaust_on_use` means exact consumer stage, not OR of unrelated source fields. | Temporary combat instance; `public_owner_ref` absent unless public owner proof exists. `retained` derives the declared live retain predicate; no identity aliases. | Draw order is hidden except legitimate Frozen Eye/known top-bottom facts; Runic Dome hides intent/damage; hidden flags/misc/cost masked. Per-cell predicate and checkpoint remain producer joins. |
| `relics.*` | Relic key, list ordinal, exact relic-qualified signed-i32 counter, Bool used. No universal counter clamp. | Relic instance and owner-specific constructor/writers/lifetime. Full literal ID/counter rows are catalog-numeric import. | Public relic identity/order; counter only when its exact owner predicate permits. |
| `potions.*` | Real slot index, Bool occupancy, potion key or absent for actual empty slot, potion-qualified signed-i32 potency. Empty actual slots are rows; no padding. | Run-owned actual belt slot/list; potion identity and potency accessor/call chain from catalog-numeric. | Held potion/slot public; potency only when owner says public/applicable. Never expose unheld future potion. |
| `actors.*` | Enemy key; formation ordinal; HP/maxHP/Block signed source domains; owner-qualified intent key and exact signed payload/count. | Combat owns actor; all identities/move payloads bind to finite source keys and exact writer/rounding members. | Intent, damage and hit count masked under Runic Dome or other suppressing rule; actor identity/HP only at current legitimate reveal. |
| `powers.*` | Power key and owner-qualified signed-i32 amount; `PowerOwner` is `Player` singleton or live `Actor(Enemy Handle)`; public facade pair `(0,0)` maps only to Player and `(7,actor_slot_local_row)` maps through the current actor table. Neither facade pair is a general RowRef. | PlayerState is the sole player-power authority; each EnemyState owns its actor powers; CombatBridgeState carries no duplicate. Owner collection and live handle must agree; ordinal is unique within owner lifetime. | Amount/counter only under per-power visibility; no hidden future power. |
| `orbs.*` | Orb key/slot ordinal and separately typed passive/evoke/counter signed domains. No invented common orb counter. | Combat owner, orb identity and owner-specific amount/rounding. | Current owned orb state only; private random future draw/cursor excluded. |
| `offers.*` | Closed offer/content tags; typed optional target; offer-specific amount; settled `SettledPrice`; lifecycle status; public ordinal. | Offer instance distinct from claim/acquisition. Each exact producer constructor→offer→claim/payment callback is catalog/execution import; price owner must be explicitly registered. | Only currently offered/revealed menu rows. Do not disclose unoffered pool or hidden future contents. Unregistered composite price is unemittable, never generic property 41. |
| `room_objects.*` | Closed object/face/state codebooks, typed contents, public ordinal and exact source attempt count if present. | Room/event/board object lifetime; no mutation by projection. Site and attempt-counter join pending. | Face-down Match value absent/unknown; remembered removed faces persist only as learned facts. Event/question identities remain hidden until reveal. |
| `selection.*` | Typed source ref, exact selected sequence ordinal or absent, Bool selected/eligible. | Current selector owner; orderedness/min/max/cancel/resume are per-owner continuation fields. Exact selector callback set is execution producer import. | Only current legal selector and revealed entries. No hidden candidate eligibility/selection leakage. |
| `knowledge.*` | Closed fact/value kinds and fact-qualified payload; references local or absent. | Rule-legitimate durable/recently learned facts; reveal checkpoint owns lifetime. | Knowledge is the explicit reveal boundary. Hidden fields do not become known merely because a row exists. |
| `properties.*` | Closed codebook and exact owner-qualified scalar/ref domain. Key `(owner,property,ordinal)` unique. Ref uses explicit typed table/row pair. | Frozen finite owner/content/property table; no generic extension bag. Codes 1–45 are candidate identities only until exact owner/domain equality closes. | Property is emitted only if its registered owner predicate is true at this checkpoint. Reserved property 41 stays unemittable pending owner proof. |
| `history.*` | Closed fact and subject vocabularies, exact fact-qualified payload and public floor ordinal. | Authoritative mechanics history, mutated only at exact source boundary; never rebuilt from logs/offers/reward totals. | Public facts only after their reveal floor/checkpoint. Private RNG/mechanical history excluded. |
| `candidates.*` | Closed action tag, typed optional source/target, owner-qualified option key and span into same-slot parameters. Candidate index is local `[0,N)`. | Each row maps privately and bijectively to exactly one current core choice. Full live menu/core set equality comes from execution producer. | Exactly legal current candidates in deterministic public-semantic order; no hidden fields, handles, or policy token in features. |
| `candidate_parameters.*` | Closed param/value-kind tags; ordered ordinal; typed scalar/ref. | Action- and option-qualified parameter grammar, no caller-writable candidate payload. | Only parameters of published legal candidates; never include hidden target outcomes. |
| `relations.*` | Closed relation kind and typed endpoint pair with owner-specific multiplicity. | Public relation only; endpoint pairs and cardinality from the catalog/action/effect owner. | Every relation must already be public; no private source/copy identity. |

The public crosswalk deliberately binds source-member names and source domains only when supported by a routed exact producer. Until then, a row is `Open(producer_join_id)` and cannot be advertised as an established source fact or used as an implementation default. Exact field keys are the fully qualified `context.<column>`, `player.<column>`, and `<table>.<column>` names in the fixed order above. Producer reconciliation must produce one value for every key and explicitly classify all source members as mapped, property-mapped or excluded.

### 4.2 Codebooks, references, canonical ordering

Enum codes are explicit per descriptor revision, start at 1 except the stated zero/absent values. Codebook families: `Phase(Neow,Map,Room,Combat,Selection,Dead,Act2Ready)`; `Outcome(None,Death,Act1CompleteAct2Ready)`; `Decision(None,Neow,Map,Event,Shop,Campfire,Chest,Reward,BossRelic,Combat,DeckSelection,CombatSelection,MatchBoard,PotionSelection,Handoff)`; `Character(Ironclad,Silent,Defect,Watcher)` (creation admits Ironclad only); `Stance(None,Neutral,Wrath,Calm,Divinity)`; `RoomKind(Unknown,Monster,Elite,Boss,Event,Shop,Rest,Treasure,BossTreasure)`; `Zone(Hand,Draw,Discard,Exhaust,InPlay,OtherPublic)`; `Bottle(None,Attack,Skill,Power)`; `CostKind(Ordinary,X,Unplayable)`; `OfferKind(Card,Relic,Potion,Gold,Key,HP,MaxHP,Removal,Upgrade,Transform,SourceOption)`; `PriceKind(None,Gold,HpLoss,Damage,MaxHP,RegisteredComposite)`; `OfferStatus(Offered,Selected,Claimed,Skipped,Removed,Disabled)`; `ObjectKind(BoardPosition,EventOption,CampfireOption,Chest,RewardContainer,SourcePublicState)`; `FaceState(NotApplicable,FaceDown,Exposed,RememberedHidden,Removed)`; `ObjectState(Neutral,Available,Disabled,Selected,ResolvingPublicStage,Completed,Removed)`; `KnowledgeFact(DrawCount,DiscardCount,ExhaustCount,KnownNext,KnownBottom,ScryWindow,RememberedBoard,AnnouncedBoss,SourceRevealed)`; `HistoryFact(VisitedRoom,RevealedEvent,ChosenOption,RevealedBoard,PublicPlay,PublicDiscard,PublicCounter)`; `RelationKind(RewardExclusive,RewardParent,PublicCopySource,PublicPermanentOwner,PublicSelectionSource,PublicKnownOrder)`; `ValueKind(Signed,Boolean,Enum,Binary32Bits,RowRef)`.

Table references are slot-local. Allowed edges endpoints → nodes; combat card public owner → permanent card or absent; power owner is the separate typed facade conversion defined in Appendix A.1 (Player `(0,0)` or live actor `(7,row)`, never a RowRef); offer target → permanent/combat card, relic, potion, actor or room object; selection source → permanent/combat card, potion, offer or room object; knowledge subject → node, permanent/combat card, actor, offer or room object; candidate source → permanent/combat card, potion, offer, room object or selector; candidate target → player, node, edge, permanent/combat card, potion, actor, offer or room object; property/parameter refs → public tables 1–13; relation endpoints → 1–13 with per-relation endpoint rules. Invalid table-kind pair rejects even if the numeric row exists. Absent pair is exactly `(0,-1)` at the facade; no half-absent pair.

Unknown cells carry canonical zero and known=0. Known numeric zero carries known=1. A permitted absent value is separately tagged and known, not inferred from mask. Table order is `nodes (y,x)`, `edges (from,to)`, otherwise rule-significant visible source order or bytewise order of the complete public semantic tuple; hidden IDs, addresses, worker/slot order and private RNG order cannot break ties. If two rows differ in a public legal future, they must not be collapsed. Forethought+ and all ordered selectors preserve submitted/selected sequence. Duplicate selector member is invalid unless its owner explicitly declares a multiset.

## 5. Action grammar and legality

The action carrier is `(candidate_index:U64,decision_token:Bytes[32])`; the token is provenance only, excluded from policy observation, replay identity and canonical semantic choice. Candidate row `i` denotes one exact core `CoreChoice`. The closed action kinds and consumer meaning are:

| Code | Kind | Exact source/target and parameters |
|---:|---|---|
| 1 | `PlayCard` | Source combat-card row; target/targets and X/ordered extra params exactly from its current source-qualified core choice. |
| 2 | `EndTurn` | No source/target/parameters. |
| 3 | `UsePotion` | Source potion slot; target/choice arguments from current legal potion menu. |
| 4 | `DiscardPotion` | Source actual potion slot; no target or hidden payload. |
| 5 | `TraverseMap` | Source current node and destination node/edge. |
| 6 | `ChooseEventOption` | Current event handle internally; option key from current visible menu. |
| 7 | `Buy` | Exact current shop offer and payment source/option fields. |
| 8 | `Purge` | Exact current purge selector/payment choice. |
| 9 | `ChooseCampfire` | Exact currently usable source-owned campfire option. |
| 10 | `ClaimReward` | Exact currently offered reward row. |
| 11 | `SkipReward` | Exact source-owned skip candidate only where the live menu offers it. |
| 12 | `TakeBossRelic` | Exact offered boss-relic row. |
| 13 | `SkipBossRelic` | Exact source-owned skip candidate only where offered. |
| 14 | `SelectInstance` | One eligible current selector member, if owner allows incremental selection. |
| 15 | `DeselectInstance` | One currently selected member, if owner allows deselection. |
| 16 | `ConfirmSelection` | Owner-qualified selection state meeting live min/max and commit preconditions. |
| 17 | `CancelSelection` | Only if source owner exposes cancel in this phase; cancellation disposition is exact owner law. |
| 18 | `RevealBoard` | One currently face-down legal Match position; returns only legally revealed result. |
| 19 | `SingingBowl` | Exact current reward row/skip choice and its source-specific maxHP/heal behavior. |
| 20 | `Proceed` | Exact current continuation/menu proceed member; no generic screen-close action. |
| 21 | `SourceOption` | Frozen owner-qualified source option key plus exact typed arguments; never arbitrary callback dispatch. There is no clock/Wait option. |

Parameter kinds are `Count(U64)`, `ChosenPosition(U64)`, `OrderedSelectionMember(RowRef)`, `PublicX(I32)`, `OptionArgument(owner-qualified value)`, `PublicPriceComponent(registered typed component)`, and `SourcePublicFlag(Bool)`. Each action+option pair has a closed ordered parameter schema. No extra, missing, reordered, duplicate, foreign, hidden or out-of-domain parameter is accepted.

Selectors are incremental `SelectionState=(owner:SelectorKey,eligible:OrderedSet<SelectionMember>,selected:Vec<SelectionMember>,min:U64,max:U64,ordered:Bool,commit:CommitDisposition,cancel:CancelDisposition,parent:ContinuationId,resume:CheckpointKey)`. `0≤min≤max≤eligible.len`; selected members are unique and members of the exact current eligible set; if `ordered=false`, canonical order is owner-defined and if true insertion order is preserved. There is no enumerated-subset explosion or universal skip. Empty selection is legal only when that source owner explicitly permits `min=0` and its commit callback accepts it.

Preflight binds the token to `(pool_identity,run_identity,decision_generation,phase,menu_digest)` and confirms the candidate's slot-local index and exact core-choice membership. Token mismatch, stale generation, wrong slot, wrong phase, malformed parameter, duplicate/foreign selection member, absent menu row, or out-of-range index rejects before any transition. Reusing an old integer index when the exact same index happens to remain legal after a new decision cannot reveal stale caller intent; no stronger claim is made. Reads do not mint tokens; each accepted transition rebuilds decision and advances generation exactly once. Terminal observations have zero candidates.

## 6. Native hidden-state grammar

The private core uses a finite recursive tagged union, not source object dumps, opaque required payloads, unbounded byte blobs or the historical eight-tag layout. The canonical top-level is `CoreStateV5=(version=5,run:RunState,private_catalog_revision:Hash32)`; `PlayerState.powers` and `EnemyState.powers` are the distinct authoritative owner collections, and the combat bridge does not duplicate them. Recursion is only through finite vectors and typed handles into records present in the same complete state. Every handle reference resolves exactly once; dangling, duplicate identity, wrong owner, cross-run reference and unowned mutable resource are invalid. Vectors are length-delimited and may grow to the representational limit; no guessed game cap.

### 6.1 Required top-level variants

| Variant | Required fields and ownership |
|---|---|
| `RunState` | `identity,profile,rules_version,content_revision,phase,act,floor,player,map,economy,room?,continuations,history,knowledge,rng,transition?,terminal?,task_reward,decision_generation,next_handle_ordinal,next_site_invocation,commit_ledger`. This is the only persistent mutable authority. |
| `PlayerState` | `character,ascension,hp,max_hp,gold,block,energy,max_energy,draw_per_turn,potion_capacity,orb_capacity,stance,inventory,deck,growth,history`. `inventory` and `deck` are the single owners of their respective instances. Resource field transitions are operation-specific, not saturated by this schema. |
| `InventoryState` | Ordered relic instances with owner-qualified counters/used flags; exactly `potion_capacity` real slots represented as `Vec<Option<PotionInstance>>`; acquired key flags; no padding beyond actual capacity. This is the sole owner of relic, potion and key instances. |
| `DeckState` | Permanent card instances; temporary/generated combat instances; one ordered membership relation for piles; bottle links; copy provenance; instance-generation ordinal. One instance belongs to only the pile/membership states legal at that checkpoint. |
| `MapState` | Generation identity; finite node and directed-edge records; room assignments; current node/floor; visited/revealed state; at-least-one-elite validity bit derivable and verified against map; burning marker reference or absent; boss identity only in private state until public reveal; topology/history. Invalid map drafts are rejected and regenerated before exposure; no patch insertion, magic retry bound or defeat. |
| `RoomState` | Room handle/kind/phase; exact room objects, offers and selectors; optional typed `EventState` and `CombatBridgeState`; parent/child continuations and once-only commit keys. No source room object ID/blob. |
| `EventState` | Event semantic key and instance handle; phase; ordered offered option keys and selection; eligibility/reveal state; attempt count; typed payment and committed effects; event history; exact resume parent. |
| `EconomyState` | Non-owning gold reference to `PlayerState.gold`; current shop visit/stock/prices; typed card/relic/potion/reward pools and consumed exclusions; pity counters; purchased service keys and history. |
| `RewardState` | Stable offer identities and ordered rows; generated-vs-claimed/skipped distinction; source/pool key; rarity/history mutation boundary; selected row; pending child; commit checkpoint. A generated offer may mutate history before acquisition. |
| `ContinuationState` | Exact finite record in §6.2; an acyclic ownership graph with explicit parent edges, not assumed LIFO. Captured values are typed snapshots; live reads are explicit typed references. |
| `CombatBridgeState` | Combat identity and borrowed run revision; enemies/moves; combat instances/piles; turn/energy/block/power/orb/stance state; histories; current entered effect and each separate ordinary/pre-turn/card/monster/effect/soul container; combat-owned native RNG site references; interruption and combat-terminal checkpoints. Does not duplicate writable persistent resources. |
| `HistoryState` (run history) | Typed authoritative counters, ordered encounter/reward/event/relic/card/potion histories, per-instance notification history, Note exchange-used flag and any mechanic-required history. Public projection is a separate filtered view. No log reconstruction. Exact child families are defined in Appendix A. |
| `NativeRngState` | Accepted producer contract concretely bound: `spire-lab-act1-rng-hmac-sha256-v1`; Seed64/Root32/Entropy32 adapters `spire-lab-act1-seed-sha256-candidate1` / `spire-lab-act1-root32-candidate1` / `spire-lab-act1-entropy32-candidate1`; partitions `train,selection,final,test`; run key `a1:<32-lowercase-hex-stream-identity>:<canonical-u64-decimal-episode>`; frame `(partition,run_id,role_id,site_id,scope...)`, BE32 field count plus BE32 byte lengths and exact printable non-space ASCII fields; HMAC-SHA256 stream/block domains `spire-lab/act1/rng/v1/stream` and `/block`; u128 word cursor `0..=2^64`, four big-endian u64 lanes per block, exact rejection `below` law. The accepted public RNG/maps contract defines the exact 13 semantic roles and 86 literal base site IDs/scopes, replay retention, and map binding; this consumer schema binds those values. `campfire.option.*`, `boss.reward.*`, and `boss.relic.*` are family markers, not allowed site IDs, and their concrete callback/site expansions remain explicitly pending EXEC/CAT. The 20 temporary generation families and run-combat callback mapping remain accepted mechanics/role inputs but their exact owner-qualified callback/site-ID joins remain pending EXEC/CAT. Role/site/key/counter/seed/rejection/history/draft state is private. No source-root/history ensemble or source correlation fields. |
| `TransitionState` | Boss reward/relic resolution; every mandatory child and commit key; transition hook progress; successor map and required Act2 start state; exact `Act2Ready` checkpoint and public projection; explicit invariant that first Act2 policy decision has not occurred. No later-act policy state. |
| `KnowledgeState` | Current map/room reveals, announced boss, legitimate draw/scry/bottom facts, remembered Match positions/faces, public history, reveal checkpoints and owners. Hidden underlying values remain in their authoritative state and are not copied into public knowledge. |

All tags are mandatory only where their selected model state exists; the tag names themselves do not force obsolete features. In particular no frame-clock `TimeState` or staged pending/committed `ProfileStage` exists. No original-runtime frame restoration exists.

### 6.2 Concrete recursive child records

`ContinuationState=(id:Handle<Continuation>,parent:Option<Handle<Continuation>>,source:SiteKey,phase:ContinuationPhase,resume:CheckpointKey,captured:Vec<CapturedValue>,eligible:Vec<SelectionMember>,selected:Vec<SelectionMember>,remaining:Vec<SelectionMember>,payment:PaymentState,children:Vec<Handle<Continuation>>,commit_keys:Vec<CommitKey>,cancel:CancelDisposition,return_phase:RunPhase)`.

`CapturedValue` is a tagged union of `I32`, `U32`, `Bool`, `EnumValue(owner,code)`, `F32Bits`, `ContentKey(namespace,key)`, `CardSnapshot`, `ActorSnapshot`, `ResourceSnapshot`, `OrderedHandles(kind,Vec<Handle<kind>>)`, `TypedRowRefs(table,Vec<RowRef<table>>)`, and `Price(RegisteredPrice)`. No arbitrary map/string/byte payload. Every variant has a fixed field grammar and source capture checkpoint; dynamic members do not borrow mutable source objects.

`CommitKey=(run_identity,invocation:U64,checkpoint:CheckpointKey,source_ordinal:U64)` is unique within the run and written exactly once. `PaymentState` tags `NotDue`, `Reserved(amount,resource,source_checkpoint)`, `Committed(amount,resource,commit_key)`, `Refused(reason_key)`, `Cancelled(disposition)`; a resumed child cannot repay. `CancelDisposition` is `NotCancellable`, `ReturnToParent`, `CommitPartial(ordered Vec<CommittedEffect>)`, or `SourceTerminal(outcome)` only where owner law selects it. `CheckpointKey` is a frozen owner-qualified semantic key, never a frame/timer count.

`RoomObject` is one of `BoardPosition(position,face,Option<revealed ContentKey>,attempt_state)`, `EventOption(option_key,eligibility,payment_key,commit_key?)`, `CampfireOption(option_key,usability,commit_key?)`, `Chest(chest_kind,reward_state,key_state)`, `RewardContainer(offer_ids,claim_state)`, `ShopService(service_key,offer_id?,enabled,price_key)`, or `SourcePublicState(owner_key,closed typed field vector)`. `Offer` is `(id,kind,content_key,source_site,source_instance?,target?,amount?,settled_price?,status,public_order,claim_commit?)`. Both have exact owner-qualified dictionaries; no general dynamic object bag.

`CardSnapshot` contains `(card_key,upgrade_count,misc,base_cost,turn_cost,flags,magic_values,damage_values,block_values,copy_origin,capture_checkpoint)` with each optional member present only if its field contract/entitlement says it exists. `PermanentCard` adds owner instance handle, permanent growth state, bottle link and legal-removal/upgrade predicates. `CombatCard` has a distinct combat-instance handle, source/permanent provenance, zone, exact current cost layers, one-shot flags, copied fields and temporary duration. Copies allocate new nonreused handles and explicitly name source, site, capture boundary and fields copied; no aliasing. Permanent growth always targets its actual permanent owner at the qualifying boundary.

`EnemyState=(handle,enemy_key,hp,max_hp,block,formation_ordinal,move_history,selected_move,move_payload,powers,death_state,escape_state,split_state,terminal_credit)`; `death_state` distinguishes alive/dying/dead, `escape_state` is separate, split children receive new handles and do not inherit parent kill credit. `MovePayload` is owner-qualified typed intent with exact fields; hidden/suppressed payload remains in core but not projection.

`CombatWork` is a tagged union `EnteredAction`, `OrdinaryAction`, `PreTurnAction`, `CardQueueEntry`, `MonsterSnapshot`, `RoomEffect`, `DungeonEffect`, `Soul`, each carrying stable private handle, owner-qualified action key, typed captured parameters, target handles, source/continuation identity, and commit/cancel checkpoint. Queues remain separate and preserve their own insertion/selection algorithms; there is no universal FIFO or drain. Execution producer imports close exact action tag payloads and causal settlement. Until then this declared union is a consumer requirement, not encoded producer equality.

`HistoryState` is not a generic string-counter map. It is the product of finite records `EncounterHistory(queue-family,ordered last-family keys,selection counters)`, `RewardHistory(site key,generated slot count,pity state,consumed identity set)`, `RelicHistory(pool key,ordered remaining keys,consumed rejected identities,owned exclusion set)`, `EventHistory(pool key,visit count,eligibility facts,removed identities,attempt state)`, `CardHistory(per-instance play/use/loss/discard/upgrade records)`, `PotionHistory(site-specific offers/consumption/phase state)`, `NoteHistory(exchange-used,stored-card identity,upgrade count,persistence checkpoint)`, and additional named mechanic records only where a selected mechanic consumes them. All fields have owner/site keys; history changes at the exact producer-defined boundary, never from logs.

## 7. Visibility and equality

Core equality compares the complete canonical `CoreStateV5` encoding including hidden, causal and all independently advancing RNG streams, their exact tuple keys/cursors, invocation/conditioning/rejection state, retained root metadata and map-draft state. It does not compare source objects, pointer values, rendering, frame time or diagnostic logs. A hidden field can differ while public projection remains byte-identical; if it influences a future transition it must exist in core state and replay equality. Every field has a public visibility rule `Visibility=(owner_key,field_key,predicate_key,checkpoint_key,mask_behavior,reveal_lifetime,evidence_state)`. `mask_behavior` is either `NotEmitted`, or present row with `known=0` and canonical zero; the choice is frozen per field. A masked identifier cannot leak through row existence, order, count, references or candidate set.

Public state excludes seed/key/cursor, stream/site state, hidden future encounter/event/pool/board/card order, unrevealed question marks/options, private identity handles, memory allocation, worker/slot IDs, pending callback payloads, hidden counters, replay traces and diagnostic internals. Frozen Eye reveals current draw order only, not a future reshuffle; legal top/bottom and scry facts have explicit lifetimes. Runic Dome removes both intent and damage-derived fields. Match face-down positions can be public while values remain unknown; remembered face is a legitimate learned fact after the reveal. A configured Note identity stays hidden until its legal reveal.

Public-equivalence tests compare fixed cells, every table's length/order/mask/ref, candidate rows and parameters. Renaming private handles while consistently rewriting references must preserve public bytes and canonical core semantics except the handle-independent equality normalization. Mutating a hidden causal payload preserves public bytes until reveal and changes core bytes/replay digest. A legal reveal changes only the entitled fields/facts and the public digest.

## 8. Detached profile, reset and terminal outputs

`CreationProfileV2=(character:Ironclad,ascension:0,unlocks:FullyUnlocked,neow_eligibility:Full,ordinary_bosses_seen:All,creation_history:CreationHistory,stored_note_card:Option<StoredCard>,stored_note_upgrades:U32,note_exchange_available:Bool,declared_overrides:Vec<ProfileOverride>)`. Default is all unlocked/full Neow/all ordinary bosses seen, and unupgraded Iron Wave as the independent Note card. Overrides must be explicit, legal, owned values and validated completely before seed/identity advancement. Profile has no arbitrary save/account import or arbitrary starting deck/resources. Each episode captures a deep detached profile. Within-run Note exchange/off-color stored cards remain legal; no automatic cross-episode inheritance. Minimal optional output/reuse may be specified only as an explicitly requested detached result; staged pending/committed export is not required.

Standard start values are 80/80 HP, 99 gold, Burning Blood, five Ironclad Strikes, four Ironclad Defends, Bash, three empty potion slots and no keys, subject to exact profile/start source-field joins. Death returns actual death, reward `-1`, terminated true, truncated false; no late heal revives it. Resolved `Act2Ready` after rewards, mandatory choices, transition effects and Act2 initialization returns `+1`, terminated true, truncated false, with no Act2 choice. Nonterminal returns reward zero and both flags false. Fault never becomes death/success/profile output and never triggers reset; committed in-run mutations remain internal to faulted pool.

One final-delivery record is `(present,retired_run_identity,retired_decision_generation,retired_observation,retired_resources,retired_context,retired_outcome,retired_reward,retired_info)`, all detached and self-owned. Vector `final_present:Bool[B]`, ascending `final_slots:I64[K]`, and an independent packed K-segment terminal observation batch are returned; K=0 and K=B retain exact empty/full shapes. Final rows have zero candidates. In same-step autoreset, top-level observation/current identity/info describe the replacement, while reward/end flags/final describe the retired run. Manual terminal returns the terminal observation until explicit reset. Previously returned observations, profiles, infos, arrays and finals survive later step/reset/close and caller mutation cannot affect core state.

## 9. Fault, batch and worker semantics

`PoolState` is `Uninitialized`, `Ready`, `Operating`, `Faulted`, `Closed`; slot state is `Uninitialized`, `Live`, `Terminal`. A Ready pool may contain uninitialized slots after a proper-subset reset. `step_batch` addresses the exact configured live slots and validates all before dispatch; `read/reset` address nonempty ascending unique subsets. No uninitialized slot manufactures an observation. Faulted pool permits diagnostic read only if it cannot be represented as successful gameplay, plus close; no reset, replay, retry or resume.

Preflight includes dtype/shape/range, slot/lifecycle/mode, reset profile and seed variants, uniqueness/order, token/decision generation/menu membership for step, selector/candidate legality, content/schema IDs, checked references/spans/lengths/bytes, duplicate run key, next ordinal and pre-reservable bookkeeping. If any row fails, reject entire batch before RNG, token, profile, run identity, episode ordinal, reward, state or live-abandonment mutation. Reservations do not consume semantic identity.

After dispatch, core, binding, encoding, numeric, worker, allocation or facade delivery fault: stop new work; quiesce all workers; preserve all applied durable mutations and the first observed failure with slot/stage/site/checkpoint/cursor provenance; emit no partial successful batch, fake final, fake loss, rollback or duplicate action. Fault the pool. No promise that OOM can allocate diagnostic text. The operation lease spans dispatch through projection, packing and facade ownership transfer so a delivery failure is attributed to the current step/reset/replay. Already delivered detached values are not retroactively faulted. Runtime construction failure after valid reset preflight follows the same no-rollback rule.

Slots exclusively own mutable state. Semantic run/site identities assigned before parallel work. Worker count and adjacent slots cannot influence results; publish by logical slot order. Persistent pool-private workers or direct native iteration at effective one are implementation choices; no per-step thread creation, global gameplay lock or Python per-effect callbacks. No product implementation or performance claim is made here.

## 10. Canonical encoding, fingerprints and replay

All fixed-width integers are big-endian. `BYTES(x)=U64_BE(length)||x`; `STR(s)=BYTES(strict UTF8(s))`; `HASH` is exactly 32 raw bytes; schema reference is `STR(identity)||HASH(component_descriptor)`. `Option<T>` uses one-byte tag 0 or tag 1 then T. Vectors carry U64 element count. Enclosing lengths and all products/sums validate before allocation/read advance. Parsers reject unknown tags/versions, truncation, trailing bytes, duplicate/noncanonical keys, invalid UTF-8, alternate numeric spellings and malformed nested records.

The private canonical state stream is `A1CS || U16_BE(CoreStateV5.version=5) || SCHEMA(core_identity,core_descriptor_hash) || BYTES(encode(RunState)) || Hash32(private_catalog_revision)`. `RunState` fields appear exactly in the corrected Appendix A registry order; nested records have no redundant per-record version prefix and their exact field grammars are versioned by the root core-schema identity. No host map iteration: maps encode as sorted unique keys using their field's bytewise canonical key ordering; ordered game/source sequences retain their exact order. Floating values encode as raw binary32 bits. Handles encode typed run identity/kind/ordinal; no pointer/address. Equality SHA-256 covers all these bytes and the selected rules/content/site/RNG descriptor hashes. A digest is not a substitute for the bytes/schema and no fixed final digest is invented.

Public digest remains the E17 framing rule over descriptor hash, fixed context/player values then known masks, and tables 1–18 each row count then row-major values then known bytes; no offsets, batch dimensions, token, run key or private state. Unknown payloads are canonical zero. Empty table contributes its eight-byte zero row count. Six established framing known-answer vectors G01–G06 in the accepted spec remain unchanged and are structural vectors, not game states.

`ReplayFileV3` is a complete from-start record: `magic="A1RP";version=3;Header;U64(record_count);Vec<DecisionRecord>;end_status;Option<ReplayDiagnostic>`. Header contains the eight versioned schema refs in order: observation, action, content/site, native RNG, seed adapter, replay, task, delivery; composed descriptor hash; exact source/rules revision strings; seed-kind/argument/root bytes; partition; caller stream and episode ordinal; detached `CreationProfileV2`; mandatory `CoreStateV5` schema ref; and explicit `seed_adapter_id`, `rng_contract_id`, `run_id`, native-RNG component schema identity, and aggregate RNG-pool schema identity. No arbitrary snapshot restore or absent-core-schema success mode. From-start regeneration rebuilds and compares the entire `RunRngState`; the aggregate is never installed through a public restore API. Root32/Entropy32 omit Seed64 argument bytes while retaining exact root bytes. Each decision record is `accepted_ordinal:U64,pre_public_hash:Hash32,pre_core_hash:Hash32,candidate_index:U64,semantic_action:CanonicalActionBytes,completion_tag`; complete tag carries post-public and post-core hashes. Ordinals start at 0 and are consecutive. Only last record can be incomplete and then end status is interrupted with diagnostic. Terminal status requires regenerated actual terminal boundary. Empty record list is allowed only with a complete valid header (e.g. construction failure after header); entropy/header failure cannot fabricate a root/header.

Canonical action JSON is a separate grammar from descriptor JSON: it uses unique ASCII keys sorted by bytes, ordered arrays, strict Unicode scalar strings, base-10 integers only, no booleans/null/floats/insignificant whitespace/duplicate keys/leading plus/leading zeros/negative zero. Optional action fields are omitted when absent, not null; a boolean semantic argument is represented as integer `0` or `1`, not a JSON boolean. Escape quote/backslash; controls U+0000..001F use lowercase `\u00xx`; other Unicode is raw UTF-8 with no normalization. Action semantic fields and codebooks are exact current candidate row translation; candidate index/token are separately recorded and tokens are not replay identity. Replay recreates from profile/seed/partition and normal initialization, regenerates current menu and digests, verifies candidate index maps to the recorded semantic action, then applies ordinary core choice. First mismatch reports component/run/ordinal/site provenance; never repairs/substitutes. Recording off cannot change gameplay. External allocation/export faults are not fabricated as deterministic replay events.

Fingerprint input is canonical descriptor bytes plus separate hashes of complete ordered namespace catalog, property/field manifest, visibility manifest, action/menu manifest, site registry, RNG bindings, private-core grammar, task and delivery contracts. Each component has identity/version and no self-hash recursion. Any field meaning, alias set, table/column order, codebook, visibility predicate, canonicalization, action/site binding, core tag/field or replay law change creates a new version/digest. The precise final registry bytes and digest are intentionally deferred until producer rows are accepted; this document emits no invented hash.

## 11. Exact unresolved producer joins (phase one)

Every join is keyed and has two-way equality criteria. Main must route the actual exact-head independently reviewed revision for each producer; until then the joined fields stay visibly open and this document is not full B09/B10 completion.

| Import key | Producer export and consumer relation | Acceptance test after routing |
|---|---|---|
| `EXEC.SemanticActionState` | Execution producer's finite action/site/selector/callback identity table joins `ActionKind`, `SourceOption`, action parameter grammar, core-choice private mapping and candidate menu rows; `CausalSettlement`, `ContinuationState`, hook matrix join work tags, source checkpoints, commit/cancel and interruption/damage/terminal state. | Exact source producer tuple set equals the candidate action/site/selector set in both directions; every tuple has its typed payload and resume/commit law; no unmatched owner/callback, no hidden-choice projection and no omitted causal child. |
| `RNG.NativeRngState` | Accepted exact PR33 output is bound into `NativeRngState`, replay header/state, 13 semantic roles/86 exact literal base site IDs; `campfire.option.*`, `boss.reward.*`, and `boss.relic.*` are withheld as unresolved families rather than wildcard IDs. Binding includes selected Seed64/Root32/Entropy32 adapters, exact HMAC frame/key/block/word laws, and Act1/Act2 map input/topology/room/marker/validation/reject-regenerate signature. The accepted run-seed KAT is Root32 `000102…1f`, fields `("test","a1:000…000:0","run.map.draft","map.topology","act1","0")`, key `5c8754b4ae444a0fd9a24cf7645cd68896b7eb7e01cc0ec69759ef936d2e371d`, first word `7d85bafa62cbb99d`. | This closes only the routed RNG/maps producer contract. Literal owner-qualified generation callback sites and source-member/catalog/domain/visibility joins still owned by EXEC/CAT, including the three wildcard family expansions, 20 temporary generation-family site-ID mappings, and run-combat callback mapping, remain pending. No source RNG port/parity, source map/zero-Elite proof, or retry limit is claimed. |
| `CAT.SourceIdentityCatalog` | Catalog/numeric producer's literal namespace identities, aliases/collisions and `NumericDomainCatalog` member facts join every field contract, codebook, content/intent/event/option/property/action/site key and exact source-member read/write/domain. | Two-way exact catalog and source-member/property/exclusion equality; every carrier has one owner/domain/unit/sentinel/rounding/overflow/visibility predicate; duplicate alias or unmatched source member fails. |
| `HAND.CreationProfile` | Handoff/profile producer's exact profile/default/override/card validation, start values, `SuccessorState`, `TransitionCheckpoint`, and visibility records join reset input, ProfileV2, Player/Deck/Map/TransitionState, Act2Ready observation, final reward and replay creation attachment. | Exact profile field equality and validation; exact Act2 transition order/payload and required-state projection; first decision absent; hidden successor data cannot alter public output until reveal; no staged profile requirement. |

These imports are not names standing in for arbitrary bytes. Each is a public protocol key with finite child record requirements. The consumer schema above already defines ownership and placement; imported facts fill exact member/domain/site/value/predicate cells. After main routes inputs, this author will create one exact row per import field and each of the 146 fixed/table carrier fields; resolve all aliases, sites, numeric and visibility joins, replace `Open` with `Established` or explicitly excluded/nonpublic records and rerun two-way set-equality and failure-first checks. Any genuine missing factual law will identify exact source member/method and attempted static evidence. Routine field widths/order/site contract decisions are engineering, not a user questionnaire. Any new game-policy choice goes to main before writing a default.

## 12. Delivery status and evidence boundary

**Phase-one result:** the consumer-side versioned API, fixed/variable observation layout, typed reference/action/selector grammar, selected-model state ownership and recursive field grammar, visibility model, reset/step/final/fault/detachment behavior, canonical public/private replay encoding and explicit producer join keys are specified here. The contract distinguishes exact encoding proposals from accepted source facts and selected policy.

**Still open, not implied closed:** source-member/domain and property/exclusion equality across all 146 fields; complete literal namespaces/aliases and all 131-card-subset extension; action/selector/site/callback equality; exact numeric transitions/rounding/overflow and per-field visibility; EXEC causal work/continuation payload and callback joins; HAND profile/successor data; and final descriptor/catalog/core fingerprints. RNG/maps producer binding is accepted and concretely incorporated, but that does not close its remaining cross-owner catalog/action/visibility joins. The source producer PRs are not imported by working-tree observation. The old eight candidate tag names are not mandatory feature names; no frame-time state or staged profile output is required. State schema signature and structural helper checks do not constitute runtime codec equality or complete source proof.

This document does not report gameplay acceptance, product build/tests, original game/class/initializer execution, external reference runs, benchmark, implementation or implementation plan. All A1C-T/TX acceptance cases remain **UNEXECUTED**. Whole-spec human approval, reviewed coordinated implementation plan and separate implementation-start authorization remain unsatisfied.

<!-- BEGIN GENERATED NATIVE GRAMMAR APPENDIX -->

## Appendix A — corrected complete consumer wire grammar (candidate 5)

**Normative correction precedence.** This appendix supersedes conflicting earlier candidate-2 statements in §§2, 4, 6.2 and 10 for RowRef, child types, enum codes, and descriptor bytes. It is a consumer-owned finite representation grammar, not source-value proof. Exact producer-supplied semantic key sets, legal member domains, and site values remain open at the four join keys in §11. A source value can populate this grammar only after that join; an open key set is not an unbounded payload type.

### A.1 Row-reference codec and bounds

`RowRef<T>` core value is `(table_id:I64, local_row:I64)`. The sole absent value is `(0,-1)`. It is invalid to encode `(0,anything_else)`, `(nonzero,-1)`, or an unknown table ID. Present table IDs are exactly 1–18; a typed `RowRef<T>` additionally requires the ID assigned to `T`. The observation wire uses a single signed-i64 local-row cell where the descriptor fixes T, or the adjacent signed-i64 `(table_id,local_row)` columns where a field is generic. Both forms decode through this one core pair conversion; re-encoding a typed ref restores its fixed table ID. Row 0 is the first row. Slot-local row count and every exclusive packed offset are signed-i64 nonnegative values no greater than `I64_MAX`; hence valid local rows are `0..I64_MAX-1`. Reject `row >= row_count`, cumulative offset overflow, and `count > I64_MAX` before allocation/read advancement as administrative representational failure. Do not narrow, wrap, clamp, truncate, or impose a content cap. Canonical core bytes encode both pair cells as signed two's-complement i64 big-endian; the public digest encodes exactly the declared observation carrier cell(s). `PowerOwner` is a separate typed union and is never a `RowRef`: Player carries the exact containing `RunIdentity`; Actor carries a live Enemy `Handle` with that same run identity. The legacy public `powers.owner_table,owner_ref` facade maps exactly `(0,0)` to `PowerOwner::Player(current_run)` and `(7,actor_slot_local_row)` to the corresponding live Enemy handle through the current actor table. No other table-zero pair is valid; actor-row conversion checks bounds and exact handle resolution before use. Stale/cross-run player identities and actor handles reject; there is no arbitrary negative-row absence convention.

### A.2 Primitive, record, tagged-union grammar

Primitive widths are those in §2. `HashHex64` is exactly 64 lowercase ASCII hex digits in descriptor JSON. `Option<T>` is one-byte 0 or one-byte 1 followed by T; `Vec<T>` is U64 big-endian count followed by elements; all lengths/products are checked before allocation. Record fields appear in the table order below. The root `A1CS` schema version and exact core-schema identity version this complete nested grammar; nested records do not carry independent redundant version bytes. A grammar change creates a new core-schema identity/version. Every record is scoped to one `RunState`; references must resolve within that run and to the declared handle/table kind. Owner resolution is the explicit owner/handle/key field where present; otherwise a child inherits the owning parent field. A copied/captured snapshot is immutable from its capture checkpoint through the lifetime of its containing action/continuation; live refs resolve the current object at use time and are never silently converted to snapshots. A record lives exactly as long as the containing state/field/handle and is removed only at its declared owning transition. Vectors preserve their listed/game-semantic order; any uniqueness/canonical-set rule is explicit in the consuming field invariant. There are no implicit maps or host iteration: map-like values are represented as ordered vectors and must be unique, then sorted by the named key rule. Canonical records serialize each field in declaration order; no field may be omitted or added. Union encoding is U16 tag code then exactly one payload; unknown tags reject. Union tag codes are assigned 1..N by sorting tag names by unsigned UTF-8 bytes, independently of producer/source enum ordinals.

| Record | Ordered required fields (`name:type`) |
|---|---|
| `Act2StartState` | `act:U8`, `floor:I32`, `player_revision:U64`, `heal_result:I32`, `map:MapState`, `current_room:Option<Handle>`, `first_decision_pending:Bool` |
| `ActionArgument` | `kind:ParameterKind`, `ordinal:U32`, `value:ActionArgumentValue` |
| `ActorSnapshot` | `actor:Handle`, `actor_key:OwnerKey`, `hp:I32`, `max_hp:I32`, `block:I32`, `move_key:Option<OwnerKey>`, `capture_checkpoint:CheckpointKey` |
| `AttemptHistory` | `object:Handle`, `attempt:U32`, `result:AttemptState`, `checkpoint:CheckpointKey` |
| `BoardPosition` | `position:U32`, `face:FaceState`, `revealed:Option<ContentKey>`, `attempt:AttemptState` |
| `BossRelicResolution` | `offer_ids:Vec<Handle>`, `selected:Option<Handle>`, `skipped:Bool`, `children:Vec<Handle>`, `resolved:Bool` |
| `BottleLink` | `relic:Handle`, `card:Handle`, `bottle:BottleKind` |
| `CampfireOptionObject` | `option:OwnerKey`, `usable:Bool`, `commit:Option<CommitKey>` |
| `CancelledPayment` | `disposition:CancelDisposition` |
| `CanonicalAction` | `kind:ActionKind`, `source:Option<RowRef>`, `target:Option<RowRef>`, `arguments:Vec<ActionArgument>` |
| `CardFlag` | `flag:CardFlagKind`, `value:Bool` |
| `CardGrowth` | `growth_key:OwnerKey`, `amount:I32`, `checkpoint:CheckpointKey` |
| `CardHistory` | `instance:Handle`, `plays:U64`, `uses:U64`, `losses:U64`, `discards:U64`, `upgrades:U64`, `notifications:Vec<NotificationHistory>` |
| `CardMutation` | `card:Handle`, `field:CardMutationKind`, `before:I32`, `after:I32`, `cause:SiteKey` |
| `CardSnapshot` | `card_key:OwnerKey`, `upgrade_count:I32`, `misc:I32`, `base_cost:I32`, `turn_cost:I32`, `flags:Vec<CardFlag>`, `magic_values:Vec<I32>`, `damage_values:Vec<I32>`, `block_values:Vec<I32>`, `copy_origin:Option<CopyOrigin>`, `capture_checkpoint:CheckpointKey` |
| `CheckpointKey` | `site:SiteKey`, `checkpoint:AsciiKey`, `ordinal:U64` |
| `ChestObject` | `kind:ChestKind`, `reward:RewardState`, `key_state:KeyCommitState` |
| `ClaimState` | `state:ClaimStateKind`, `selected:Option<Handle>`, `commit:Option<CommitKey>` |
| `CombatBridgeState` | `combat:Handle`, `run_revision:U64`, `enemies:Vec<EnemyState>`, `moves:Vec<MovePayload>`, `cards:Vec<CombatCard>`, `piles:Vec<PileMembership>`, `energy:I32`, `block:Vec<ResourceSnapshot>`, `orbs:Vec<OrbState>`, `stance:Stance`, `history:Vec<HistoryEntry>`, `entered:Option<CoreAction>`, `ordinary:Vec<CoreAction>`, `preturn:Vec<CoreAction>`, `card_queue:Vec<CoreAction>`, `monster_queue:Vec<Handle>`, `effects:Vec<CoreAction>`, `interruption:Option<OwnerKey>` |
| `CombatCard` | `handle:Handle`, `snapshot:CardSnapshot`, `permanent_source:Option<Handle>`, `zone:CardZone`, `temporary_duration:Option<I32>`, `cost_layers:Vec<CostLayer>` |
| `CommitDisposition` | `tag:CommitDispositionTag`, `checkpoint:Option<CheckpointKey>` |
| `CommitKey` | `run:RunIdentity`, `invocation:U64`, `checkpoint:CheckpointKey`, `source_ordinal:U64` |
| `CommittedPayment` | `amount:I32`, `resource:ResourceKey`, `commit:CommitKey` |
| `CompositePrice` | `owner:OwnerKey`, `components:Vec<PriceComponent>`, `settled:I32`, `checkpoint:CheckpointKey` |
| `ContentKey` | `namespace:ContentNamespace`, `semantic_key:AsciiKey` |
| `ContinuationPhase` | `phase:ContinuationPhaseKind`, `source:SiteKey`, `parent:Option<Handle>`, `checkpoint:CheckpointKey` |
| `ContinuationState` | `id:Handle`, `parent:Option<Handle>`, `source:SiteKey`, `phase:ContinuationPhase`, `resume:CheckpointKey`, `captured:Vec<CapturedValue>`, `eligible:Vec<SelectionMember>`, `selected:Vec<SelectionMember>`, `remaining:Vec<SelectionMember>`, `payment:PaymentState`, `children:Vec<Handle>`, `commit_keys:Vec<CommitKey>`, `cancel:CancelDisposition`, `return_phase:RunPhase` |
| `CopyOrigin` | `source:Handle`, `site:SiteKey`, `capture_checkpoint:CheckpointKey`, `copied_fields:Vec<AsciiKey>` |
| `CoreAction` | `tag:CoreActionTag`, `owner:OwnerKey`, `source:Option<Handle>`, `targets:Vec<Handle>`, `parameters:Vec<CapturedValue>`, `parent:Option<Handle>`, `checkpoint:CheckpointKey` |
| `CostLayer` | `kind:CostLayerKind`, `value:I32`, `origin:SiteKey`, `active:Bool` |
| `CreationProfileV2` | `character:Character`, `ascension:U8`, `unlocks:UnlockProfile`, `neow_eligibility:NeowEligibility`, `ordinary_bosses_seen:Vec<OwnerKey>`, `creation_history:Vec<ProfileFact>`, `stored_note_card:Option<OwnerKey>`, `stored_note_upgrades:U32`, `note_exchange_available:Bool`, `overrides:Vec<ProfileOverride>` |
| `DamageComponent` | `amount:I32`, `owner:OwnerKey` |
| `DamagePrice` | `amount:I32` |
| `DeathRecord` | `cause:SiteKey`, `checkpoint:CheckpointKey`, `rescues_checked:Vec<OwnerKey>`, `death_committed:Bool` |
| `DecisionRecord` | `accepted_ordinal:U64`, `pre_public_hash:Hash32`, `pre_core_hash:Hash32`, `candidate_index:U64`, `semantic_action:CanonicalActionBytes`, `completion:ReplayCompletion` |
| `DeckState` | `permanent_instances:Vec<PermanentCard>`, `combat_instances:Vec<CombatCard>`, `generated_instances:Vec<CombatCard>`, `membership:Vec<PileMembership>`, `instance_generation:U64`, `copy_provenance:Vec<CopyOrigin>`, `bottle_links:Vec<BottleLink>` |
| `Descriptor` | `version:U16`, `identity:AsciiKey`, `fixed_rows:Vec<DescriptorFixedRow>`, `tables:Vec<DescriptorTable>`, `codebooks:Vec<DescriptorCodebook>`, `references:Vec<DescriptorReference>`, `visibility:Vec<DescriptorVisibility>`, `ordering:Vec<DescriptorOrdering>`, `actions:Vec<DescriptorAction>`, `parameters:Vec<DescriptorParameter>`, `core_schema:AsciiKey`, `replay_schema:AsciiKey`, `task:AsciiKey`, `compatibility:Vec<DescriptorCompatibility>`, `component_hashes:Vec<DescriptorComponentHash>` |
| `DescriptorAction` | `code:U32`, `name:AsciiKey`, `source_tables:Vec<U16>`, `target_tables:Vec<U16>`, `parameter_schema:AsciiKey` |
| `DescriptorCodebook` | `name:AsciiKey`, `wire_width:U8`, `members:Vec<DescriptorEnumMember>` |
| `DescriptorColumn` | `ordinal:U16`, `name:AsciiKey`, `wire:WireKind`, `domain:AsciiKey`, `unit:AsciiKey`, `sentinels:Vec<I64>`, `owner:AsciiKey`, `visibility:AsciiKey`, `lifetime:AsciiKey` |
| `DescriptorCompatibility` | `name:AsciiKey`, `value:AsciiKey` |
| `DescriptorComponentHash` | `name:AsciiKey`, `schema:AsciiKey`, `digest:HashHex64` |
| `DescriptorEnumMember` | `name:AsciiKey`, `code:U32` |
| `DescriptorFixedRow` | `name:AsciiKey`, `columns:Vec<DescriptorColumn>` |
| `DescriptorOrdering` | `table:AsciiKey`, `key_fields:Vec<AsciiKey>`, `direction:SortDirection` |
| `DescriptorParameter` | `code:U32`, `name:AsciiKey`, `value_type:AsciiKey` |
| `DescriptorReference` | `field:AsciiKey`, `table_ids:Vec<U16>`, `absent_table:I64`, `absent_row:I64` |
| `DescriptorTable` | `id:U16`, `name:AsciiKey`, `columns:Vec<DescriptorColumn>` |
| `DescriptorVisibility` | `field:AsciiKey`, `predicate:AsciiKey`, `checkpoint:AsciiKey`, `mask:MaskBehavior`, `lifetime:AsciiKey` |
| `EconomyState` | `gold_owner:ResourceKey`, `shop_visit:U64`, `stock_generation:U64`, `shop_stock:Vec<Handle>`, `prices:Vec<RegisteredPrice>`, `purge_price:Option<RegisteredPrice>`, `purchased_services:Vec<OwnerKey>`, `restock_phase:U32`, `card_pool:Vec<PoolEntry>`, `relic_pool:Vec<PoolEntry>`, `potion_pool:Vec<PoolEntry>`, `reward_pool:Vec<PoolEntry>`, `pity:Vec<PityCounter>`, `consumed_ids:Vec<ContentKey>`, `history:Vec<HistoryEntry>` |
| `EdgeRecord` | `from:Handle`, `to:Handle`, `ordinal:U64` |
| `EligibilityReason` | `member:SelectionMember`, `reason:OwnerKey`, `checkpoint:CheckpointKey` |
| `EligibilityState` | `policy:OwnerKey`, `eligible:Vec<SelectionMember>`, `excluded:Vec<SelectionMember>`, `reasons:Vec<EligibilityReason>` |
| `EncounterHistory` | `queue_family:OwnerKey`, `last_family_keys:Vec<OwnerKey>`, `selection_counters:Vec<HistoryCounter>` |
| `EnemyState` | `handle:Handle`, `enemy_key:OwnerKey`, `hp:I32`, `max_hp:I32`, `block:I32`, `formation:U32`, `move_history:Vec<OwnerKey>`, `selected_move:Option<OwnerKey>`, `payload:Option<MovePayload>`, `powers:Vec<PowerState>`, `death:EnemyDeathState`, `escape:Bool`, `split:Option<SplitState>`, `terminal_credit:Bool` |
| `EnumValue` | `owner:OwnerKey`, `code:U32` |
| `EventHistory` | `pool:OwnerKey`, `visit_count:U64`, `eligibility_facts:Vec<EligibilityReason>`, `removed_identities:Vec<OwnerKey>`, `attempts:Vec<AttemptHistory>` |
| `EventOptionObject` | `option:OwnerKey`, `eligible:Bool`, `payment:Option<PaymentKey>`, `commit:Option<CommitKey>` |
| `EventState` | `event_key:OwnerKey`, `event_instance:Handle`, `phase:EventPhase`, `offered_options:Vec<OwnerKey>`, `selected_option:Option<OwnerKey>`, `eligibility:EligibilityState`, `reveal_state:RevealState`, `attempt_count:U32`, `payment_state:PaymentState`, `committed_results:Vec<CommittedEffect>`, `history:EventHistory`, `resume_parent:Option<Handle>` |
| `FinalDelivery` | `retired_identity:RunIdentity`, `retired_generation:U64`, `observation:PublicObservation`, `outcome:TerminalOutcome`, `reward:I8`, `info:StepInfo` |
| `GoldComponent` | `amount:I32`, `owner:OwnerKey` |
| `GoldPrice` | `amount:I32` |
| `Handle` | `run:RunIdentity`, `kind:HandleKind`, `ordinal:U64` |
| `HistoryCounter` | `owner:OwnerKey`, `index:U32`, `value:I64` |
| `HistoryEntry` | `owner:OwnerKey`, `ordinal:U64`, `value:I64`, `members:Vec<OwnerKey>`, `checkpoint:CheckpointKey` |
| `HistoryMutation` | `history_key:OwnerKey`, `ordinal:U64`, `before:Option<I64>`, `after:I64`, `cause:SiteKey` |
| `HistoryState` | `encounters:EncounterHistory`, `rewards:RewardHistory`, `relics:RelicHistory`, `events:EventHistory`, `cards:Vec<CardHistory>`, `potions:PotionHistory`, `note:NoteHistory`, `mechanics:Vec<MechanicHistory>` |
| `HpLossComponent` | `amount:I32`, `owner:OwnerKey` |
| `HpLossPrice` | `amount:I32` |
| `InventoryState` | `relics:Vec<RelicInstance>`, `potion_slots:Vec<Option<PotionInstance>>`, `keys:KeyFlags`, `relic_order:Vec<Handle>`, `potion_order:Vec<U32>` |
| `KeyCommitState` | `acquired:Bool`, `commit:Option<CommitKey>` |
| `KeyFlags` | `ruby:Bool`, `sapphire:Bool`, `emerald:Bool` |
| `KeyMutation` | `key:KeyKind`, `before:Bool`, `after:Bool`, `cause:SiteKey` |
| `KnowledgeFact` | `fact:KnowledgeFactKind`, `subject:Option<Handle>`, `position:U64`, `value:I64`, `checkpoint:CheckpointKey` |
| `KnowledgeState` | `revealed_nodes:Vec<Handle>`, `announced_boss:Option<OwnerKey>`, `visible_objects:Vec<Handle>`, `event_facts:Vec<HistoryEntry>`, `known_draw_prefix:Vec<Handle>`, `known_draw_suffix:Vec<Handle>`, `scry_window:Vec<Handle>`, `remembered_faces:Vec<RememberedFace>`, `public_history:Vec<HistoryEntry>`, `facts:Vec<KnowledgeFact>`, `reveal_checkpoints:Vec<CheckpointKey>` |
| `MapHistoryEntry` | `node:Handle`, `fact:MapHistoryKind`, `checkpoint:CheckpointKey` |
| `MapMutation` | `node:Handle`, `field:MapMutationKind`, `before:I64`, `after:I64`, `cause:SiteKey` |
| `MapState` | `generation:U64`, `nodes:Vec<NodeRecord>`, `edges:Vec<EdgeRecord>`, `assignments:Vec<RoomAssignment>`, `current:Option<Handle>`, `burning_marker:Option<Handle>`, `boss:OwnerKey`, `visited:Vec<Handle>`, `revealed:Vec<Handle>`, `history:Vec<MapHistoryEntry>` |
| `MaxHpComponent` | `amount:I32`, `owner:OwnerKey` |
| `MaxHpPrice` | `amount:I32` |
| `MechanicHistory` | `site:SiteKey`, `mechanic:OwnerKey`, `counters:Vec<HistoryCounter>`, `members:Vec<OwnerKey>`, `checkpoint:CheckpointKey` |
| `MovePayload` | `move_key:OwnerKey`, `damage:Option<I32>`, `hits:Option<U32>`, `amounts:Vec<I32>`, `targets:Vec<Handle>` |
| `NativeByteWordState` | `algorithm:OwnerKey`, `words:Vec<U64>`, `cursor:U64` |
| `NativeRngState` | `contract_id:AsciiKey`, `partition:Partition`, `run_id:AsciiKey`, `role_id:AsciiKey`, `site_id:AsciiKey`, `scope:Vec<AsciiKey>`, `key:Hash32`, `next_word:U128` |
| `NativeSiteState` | `contract_id:AsciiKey`, `partition:Partition`, `run_id:AsciiKey`, `role_id:AsciiKey`, `site_id:AsciiKey`, `scope:Vec<AsciiKey>`, `next_invocation:U64`, `stream_created:Bool`, `conditioning:Vec<SiteConditionRef>` |
| `NeowEligibility` | `eligible:Bool`, `reasons:Vec<OwnerKey>`, `checkpoint:CheckpointKey` |
| `NodeRecord` | `handle:Handle`, `x:I32`, `y:I32`, `room:RoomKind`, `active:Bool`, `burning_marker:Bool`, `visited:Bool`, `revealed:Bool` |
| `NoteHistory` | `stored_card:Option<OwnerKey>`, `upgrades:U32`, `exchange_used:Bool`, `persistence_checkpoint:Option<CheckpointKey>` |
| `NotificationHistory` | `instance:Handle`, `site:SiteKey`, `notification:OwnerKey`, `count:U64`, `last_checkpoint:CheckpointKey` |
| `ObservationTable` | `table_id:U16`, `width:U16`, `values:Vec<I64>`, `known:Vec<Bool>` |
| `Offer` | `handle:Handle`, `kind:OfferKind`, `content:Option<ContentKey>`, `source:SiteKey`, `source_instance:Option<Handle>`, `target:Option<OfferTarget>`, `amount:Option<OfferAmount>`, `price:Option<RegisteredPrice>`, `status:OfferStatus`, `public_order:U64`, `claim_commit:Option<CommitKey>` |
| `OfferMutation` | `offer:Handle`, `before:OfferStatus`, `after:OfferStatus`, `cause:SiteKey` |
| `OrbEntry` | `orb_key:OwnerKey`, `passive:I32`, `evoke:I32`, `counter:I32`, `ordinal:U32` |
| `OrbState` | `orbs:Vec<OrbEntry>`, `capacity:U32` |
| `OrderedHandles` | `kind:HandleKind`, `members:Vec<Handle>` |
| `OwnerKey` | `namespace:ContentNamespace`, `semantic_key:AsciiKey` |
| `PaymentKey` | `resource:ResourceKey`, `amount:I32`, `checkpoint:CheckpointKey` |
| `PaymentStateRecord` | `payment:PaymentState`, `commit:Option<CommitKey>`, `cancel:CancelDisposition` |
| `PermanentCard` | `handle:Handle`, `snapshot:CardSnapshot`, `growth:Vec<CardGrowth>`, `bottle:Option<BottleKind>` |
| `PileMembership` | `card:Handle`, `pile:CardPile`, `ordinal:U64` |
| `PityCounter` | `site:SiteKey`, `owner:OwnerKey`, `counter:I32` |
| `PlayerState` | `character:Character`, `ascension:U8`, `hp:I32`, `max_hp:I32`, `gold:I32`, `block:I32`, `energy:I32`, `max_energy:I32`, `draw_per_turn:I32`, `potion_capacity:U32`, `orb_capacity:U32`, `stance:Stance`, `inventory:InventoryState`, `deck:DeckState`, `growth:Vec<CardGrowth>`, `history:Vec<OwnerKey>`, `powers:Vec<PowerState>` |
| `PoolConfig` | `batch_size:U64`, `worker_count:U32`, `partition:Partition`, `reset_mode:ResetMode`, `recording:RecordingMode` |
| `PoolEntry` | `key:OwnerKey`, `eligible:Bool`, `consumed:Bool`, `ordinal:U64` |
| `PotionHistory` | `sites:Vec<PotionSiteHistory>` |
| `PotionInstance` | `handle:Handle`, `potion_key:OwnerKey`, `slot:U32`, `potency:I32` |
| `PotionMutation` | `slot:U32`, `before:Option<Handle>`, `after:Option<Handle>`, `cause:SiteKey` |
| `PotionSiteHistory` | `site:SiteKey`, `offers:U64`, `consumed:Vec<OwnerKey>`, `phase:U32`, `checkpoint:CheckpointKey` |
| `PowerState` | `owner:PowerOwner`, `power_key:OwnerKey`, `amount:I32`, `ordinal:U32` |
| `ProfileFact` | `key:OwnerKey`, `value:ProfileValue` |
| `ProfileOverride` | `key:OwnerKey`, `value:ProfileValue` |
| `PublicObservation` | `fixed_context:Vec<I64>`, `fixed_context_known:Vec<Bool>`, `fixed_player:Vec<I64>`, `fixed_player_known:Vec<Bool>`, `tables:Vec<ObservationTable>` |
| `PublicRoomStateObject` | `owner:OwnerKey`, `fields:Vec<PublicTypedValue>` |
| `PublicTypedValue` | `key:OwnerKey`, `kind:ValueKind`, `integer:I64`, `reference:Option<RowRef>` |
| `ReadBatchResult` | `observations:Vec<PublicObservation>`, `infos:Vec<StepInfo>` |
| `RefusedPayment` | `reason:OwnerKey` |
| `RejectionState` | `contract_id:AsciiKey`, `partition:Partition`, `run_id:AsciiKey`, `role_id:AsciiKey`, `site_id:AsciiKey`, `scope:Vec<AsciiKey>`, `invocation:U64`, `operation:AsciiKey`, `rejected_words:Vec<U64>`, `active:Bool` |
| `RelicHistory` | `pool:OwnerKey`, `remaining_keys:Vec<OwnerKey>`, `consumed_rejected:Vec<OwnerKey>`, `owned_exclusions:Vec<OwnerKey>` |
| `RelicInstance` | `handle:Handle`, `relic_key:OwnerKey`, `counter:I32`, `used:Bool`, `properties:Vec<RelicProperty>` |
| `RelicMutation` | `relic:Handle`, `field:RelicMutationKind`, `before:I32`, `after:I32`, `cause:SiteKey` |
| `RelicProperty` | `property_key:OwnerKey`, `ordinal:U32`, `value:CapturedValue` |
| `RememberedFace` | `position:U32`, `content:OwnerKey`, `reveal_checkpoint:CheckpointKey` |
| `ReplayCompleteHashes` | `post_public_hash:Hash32`, `post_core_hash:Hash32` |
| `ReplayDiagnostic` | `stage:ReplayStage`, `record_ordinal:U64`, `component:AsciiKey`, `expected:Hash32`, `actual:Hash32`, `site:Option<SiteKey>`, `message_code:AsciiKey` |
| `ReplaySeed` | `kind:SeedKind`, `argument:Option<U64>`, `root:Hash32` |
| `ReservedPayment` | `amount:I32`, `resource:ResourceKey`, `checkpoint:CheckpointKey` |
| `ResetBatchResult` | `observations:Vec<PublicObservation>`, `infos:Vec<StepInfo>` |
| `ResetRequest` | `profile:CreationProfileV2`, `seed:SeedRequest` |
| `ResourceKey` | `owner:OwnerKey`, `resource:ResourceKind`, `instance:Option<Handle>` |
| `ResourceMutation` | `snapshot:ResourceSnapshot`, `cause:SiteKey` |
| `ResourceSnapshot` | `resource:ResourceKey`, `before:I32`, `after:I32`, `checkpoint:CheckpointKey` |
| `RewardContainerObject` | `offers:Vec<Handle>`, `claim:ClaimState` |
| `RewardHistory` | `site:SiteKey`, `generated_slots:U64`, `pity:Vec<PityCounter>`, `consumed:Vec<ContentKey>` |
| `RewardState` | `offers:Vec<Handle>`, `selected:Option<Handle>`, `claimed:Vec<Handle>`, `skipped:Vec<Handle>`, `pool:OwnerKey`, `checkpoint:CheckpointKey` |
| `RoomAssignment` | `node:Handle`, `room_key:OwnerKey`, `room_kind:RoomKind` |
| `RoomMutation` | `room:Handle`, `field:RoomMutationKind`, `before:I64`, `after:I64`, `cause:SiteKey` |
| `RoomPhaseState` | `phase:RoomPhase`, `continuation:Option<Handle>`, `checkpoint:CheckpointKey` |
| `RoomState` | `handle:Handle`, `kind:RoomKind`, `phase:RoomPhase`, `objects:Vec<RoomObject>`, `offers:Vec<Offer>`, `selectors:Vec<SelectionState>`, `event:Option<EventState>`, `combat:Option<CombatBridgeState>`, `parent:Option<Handle>`, `children:Vec<Handle>`, `commit_keys:Vec<CommitKey>` |
| `RowRef` | `table_id:I64`, `local_row:I64` |
| `RunIdentity` | `stream:U128`, `episode:U64` |
| `RunState` | `identity:RunIdentity`, `profile:CreationProfileV2`, `rules_version:AsciiKey`, `content_revision:Hash32`, `phase:RunPhase`, `act:U8`, `floor:I32`, `player:PlayerState`, `map:MapState`, `economy:EconomyState`, `room:Option<RoomState>`, `continuations:Vec<ContinuationState>`, `history:HistoryState`, `knowledge:KnowledgeState`, `rng:RunRngState`, `transition:Option<TransitionState>`, `terminal:Option<TerminalOutcome>`, `task_reward:I8`, `decision_generation:U64`, `next_handle_ordinal:U64`, `commit_ledger:Vec<CommitKey>` |
| `SchemaRef` | `identity:AsciiKey`, `digest:Hash32` |
| `SelectionState` | `owner:OwnerKey`, `eligible:Vec<SelectionMember>`, `selected:Vec<SelectionMember>`, `min:U64`, `max:U64`, `ordered:Bool`, `commit:CommitDisposition`, `cancel:CancelDisposition`, `parent:Handle`, `resume:CheckpointKey` |
| `ShopServiceObject` | `service:OwnerKey`, `offer:Option<Handle>`, `enabled:Bool`, `price:Option<RegisteredPrice>` |
| `SiteKey` | `namespace:SiteNamespace`, `owner:AsciiKey`, `operation:SiteOperation`, `variant:U32` |
| `SplitState` | `parent:Handle`, `children:Vec<Handle>`, `checkpoint:CheckpointKey`, `kill_credit:Bool` |
| `StepAction` | `candidate_index:U64`, `token:Token32` |
| `StepBatchResult` | `observations:Vec<PublicObservation>`, `rewards:Vec<I8>`, `terminated:Vec<Bool>`, `truncated:Vec<Bool>`, `infos:Vec<StepInfo>`, `finals:Vec<Option<FinalDelivery>>` |
| `StepInfo` | `episode:U64`, `decision_generation:U64`, `candidate_count:U64`, `terminal:Option<TerminalOutcome>` |
| `TerminalOutcome` | `tag:TerminalOutcomeKind`, `reward:I8`, `checkpoint:CheckpointKey` |
| `TransitionCheckpoint` | `tag:TransitionCheckpointTag`, `site:SiteKey`, `ordinal:U64` |
| `TransitionHook` | `hook:OwnerKey`, `state:TransitionHookState`, `checkpoint:CheckpointKey` |
| `TransitionHookState` | `hook:OwnerKey`, `state:HookStateKind`, `commit:Option<CommitKey>` |
| `TransitionState` | `boss_rewards:RewardState`, `mandatory_choices:Vec<Handle>`, `boss_relic:BossRelicResolution`, `hooks:Vec<TransitionHook>`, `act2:Act2StartState`, `checkpoint:CheckpointKey`, `first_decision_not_started:Bool`, `task_reward:I8`, `outcome:TerminalOutcome` |
| `TypedRowRefs` | `table_id:I64`, `rows:Vec<I64>` |
| `UnlockProfile` | `cards:UnlockSet`, `relics:UnlockSet`, `potions:UnlockSet`, `events:UnlockSet`, `encounters:UnlockSet` |
| `UnlockSet` | `all_unlocked:Bool`, `included:Vec<OwnerKey>`, `excluded:Vec<OwnerKey>` |
| `CanonicalActionBytes` | `utf8:Utf8ScalarString` |
| `PoolFault` | `slot:Option<U64>`, `stage:ApiStage`, `committed_prefix:U64`, `site:Option<SiteKey>`, `checkpoint:Option<CheckpointKey>`, `kind:ApiErrorKind` |
| `ApiError` | `kind:ApiErrorKind`, `slot:Option<U64>`, `stage:ApiStage`, `operation_ordinal:U64`, `committed_prefix:U64`, `checkpoint:Option<CheckpointKey>` |
| `ReplayHeaderV3` | `observation:SchemaRef`, `action:SchemaRef`, `content_site:SchemaRef`, `native_rng:SchemaRef`, `seed_adapter:SchemaRef`, `replay:SchemaRef`, `task:SchemaRef`, `delivery:SchemaRef`, `core_state:SchemaRef`, `descriptor_digest:Hash32`, `source_revision:RevisionString`, `rules_revision:RevisionString`, `seed:ReplaySeed`, `partition:Partition`, `run_identity:RunIdentity`, `creation_profile:CreationProfileV2`, `seed_adapter_id:AsciiKey`, `rng_contract_id:AsciiKey`, `run_id:AsciiKey`, `rng_component_schema:SchemaRef`, `rng_pool_schema:SchemaRef` |
| `CoreStateV5` | `version:U16`, `run:RunState`, `private_catalog_revision:Hash32` |
| `ReplayFileV3` | `magic:AsciiKey`, `version:U16`, `header:ReplayHeaderV3`, `records:Vec<DecisionRecord>`, `end_status:ReplayEndStatus`, `diagnostic:Option<ReplayDiagnostic>` |
| `SiteConditionRef` | `owner:OwnerKey`, `field:OwnerKey` |
| `MapDraftRngState` | `map_generation:U64`, `act:U8`, `mode:U8`, `ruleset_id:AsciiKey`, `accepted_draft_ordinal:U64`, `rejected_draft_count:U64`, `marker_drawn:Bool` |
| `RunRngState` | `contract_id:AsciiKey`, `seed_adapter_id:AsciiKey`, `seed_request:SeedRequest`, `root32:Hash32`, `partition:Partition`, `run_id:AsciiKey`, `streams:Vec<NativeRngState>`, `site_invocations:Vec<NativeSiteState>`, `rejections:Vec<RejectionState>`, `map_drafts:Vec<MapDraftRngState>` |

| Tagged union | Exact variants (`tag → payload type`; wire U16 codes derive by the sorted-tag rule above) |
|---|---|
| `ActionArgumentValue` | `ChosenPosition → U64`; `Count → U64`; `OptionArgument → OptionArgument`; `OrderedSelectionMember → SelectionMember`; `PublicPriceComponent → PriceComponent`; `PublicX → I32`; `SourcePublicFlag → Bool` |
| `CancelDisposition` | `CommitPartial → Vec<CommittedEffect>`; `NotCancellable → Unit`; `ReturnToParent → Unit`; `SourceTerminal → TerminalOutcome` |
| `CapturedValue` | `Actor → ActorSnapshot`; `Bool → Bool`; `Card → CardSnapshot`; `ContentKey → ContentKey`; `EnumValue → EnumValue`; `F32Bits → F32Bits`; `I32 → I32`; `OrderedHandles → OrderedHandles`; `Price → RegisteredPrice`; `Resource → ResourceSnapshot`; `RowRefs → TypedRowRefs`; `U32 → U32` |
| `CombatWork` | `CardQueueEntry → CoreAction`; `DungeonEffect → CoreAction`; `EnteredAction → CoreAction`; `MonsterSnapshot → Handle`; `OrdinaryAction → CoreAction`; `PreTurnAction → CoreAction`; `RoomEffect → CoreAction`; `Soul → CoreAction` |
| `CommittedEffect` | `Card → CardMutation`; `History → HistoryMutation`; `Key → KeyMutation`; `Map → MapMutation`; `Offer → OfferMutation`; `Potion → PotionMutation`; `Relic → RelicMutation`; `Resource → ResourceMutation`; `Room → RoomMutation` |
| `OfferAmount` | `Count → U64`; `Damage → I32`; `Healing → I32`; `MaxHp → I32`; `Potency → I32`; `Resource → I32`; `SelectionCount → U64` |
| `OfferTarget` | `Actor → Handle`; `Card → Handle`; `Node → Handle`; `Player → Unit`; `Potion → Handle`; `Relic → Handle`; `RoomObject → Handle` |
| `OptionArgument` | `Bool → Bool`; `Content → ContentKey`; `Enum → EnumValue`; `I32 → I32`; `Price → RegisteredPrice`; `Reference → RowRef`; `U32 → U32` |
| `PaymentState` | `Cancelled → CancelledPayment`; `Committed → CommittedPayment`; `NotDue → Unit`; `Refused → RefusedPayment`; `Reserved → ReservedPayment` |
| `PriceComponent` | `Damage → DamageComponent`; `Gold → GoldComponent`; `HpLoss → HpLossComponent`; `MaxHp → MaxHpComponent` |
| `ProfileValue` | `Bool → Bool`; `Content → ContentKey`; `History → Vec<OwnerKey>`; `I32 → I32`; `U32 → U32`; `U8 → U8` |
| `RegisteredPrice` | `Composite → CompositePrice`; `Damage → DamagePrice`; `Gold → GoldPrice`; `HpLoss → HpLossPrice`; `MaxHp → MaxHpPrice` |
| `ReplayCompletion` | `Complete → ReplayCompleteHashes`; `Interrupted → Unit` |
| `RoomObject` | `BoardPosition → BoardPosition`; `CampfireOption → CampfireOptionObject`; `Chest → ChestObject`; `EventOption → EventOptionObject`; `PublicState → PublicRoomStateObject`; `RewardContainer → RewardContainerObject`; `ShopService → ShopServiceObject` |
| `RunPhase` | `Act2Ready → TransitionCheckpoint`; `Combat → Handle`; `Creation → Unit`; `Dead → DeathRecord`; `Decision → DecisionKind`; `MapChoice → Unit`; `Neow → Unit`; `Room → Handle`; `Suspended → Handle` |
| `SeedRequest` | `Entropy32 → Unit`; `Root32 → Hash32`; `Seed64 → U64` |
| `SelectionMember` | `Actor → Handle`; `Card → Handle`; `Node → Handle`; `Offer → Handle`; `Option → OwnerKey`; `Potion → Handle`; `Resource → ResourceKey`; `RoomObject → Handle` |
| `PowerOwner` | `Actor → Handle`; `Player → RunIdentity` |

**Run-owned independent RNG state (normative).** `RunState.rng` is the sole reachable `RunRngState` aggregate, not one stream and not an opaque pool label. It retains the typed seed request, exact seed-adapter ID, 32-byte root, partition, canonical RunKey, and complete vectors of per-stream cursor/key records, semantic site-invocation counters and conditioning owner references, active rejection state, and map-draft lifecycle state. Each stream/site tuple is the complete accepted frame identity `(partition, run_id, role_id, site_id, scope...)`; equality is byte-exact after the accepted length-delimited frame encoding. Each collection is unique and sorted by that complete frame (rejections then by invocation); no scalar/global cursor substitutes. Root32 and Seed64 roots are recomputed from retained typed input; Entropy32 retains the exact captured root as the seed request. Every stream key must rederive from the retained root and exact tuple. A semantic site invocation lazily creates its site row and checked invocation ordinal; a stream row is inserted only on its first random-word request. Reusing the identical tuple resumes its cursor; a distinct role, site, scope, partition, or run key cannot alias it. Only the selected stream advances. Cursor `2^64` is valid exhausted state; a request at that endpoint fails administratively without wrap, reuse, or mutation. Site rows and streams persist for the owning run; scope reuse means resumption of that exact tuple, while retirement occurs only with run termination, never by silently dropping an advanced cursor. Conditioning is represented by canonical references to existing authoritative mechanic/history owner fields, not copied gameplay state. Rejection rows exist only during a live draw/fault and name a prior allocated tuple invocation. Map-draft rows are unique by MapState generation and retain checked accepted/rejected draft ordinals without duplicating the map payload. The complete aggregate is encoded into `CoreStateV5`; byte equality and from-start replay regeneration compare all root metadata, tuples, keys, counters, rejected words and draft state. No standalone restore/checkpoint API is introduced. These are consumer-state laws, not producer equality evidence.

A producer import may fill a declared `OwnerKey`, `ContentKey`, site owner/operation, enum member name, source-qualified numeric value, or source-specific legal domain. It may not change a record's field set/order, primitive width, option/reference shape, tagged union structure, canonical codec, ownership resolution rule, or lifetime boundary without a new consumer schema version. For source-derived variable keys the key string set is open pending the named import; each actual key is still strict nonempty ASCII and each mapping is explicit and unique.

### A.3 Consumer wire-code maps

Every pair below is `member=U32 wire code`; values are frozen consumer assignments, not source enum ordinals. A producer's member/key set is reconciled separately and cannot renumber these codes. The descriptor carries these pairs sorted by member-name bytes; code uniqueness is mandatory. `SiteNamespace`, `SiteOperation`, and content namespaces have open source-member equality until their named producer input is routed; the currently enumerated consumer assignments are proposal codes only where the names themselves are not accepted facts.

#### `ActionKind`
`PlayCard=1`; `EndTurn=2`; `UsePotion=3`; `DiscardPotion=4`; `TraverseMap=5`; `ChooseEventOption=6`; `Buy=7`; `Purge=8`; `ChooseCampfire=9`; `ClaimReward=10`; `SkipReward=11`; `TakeBossRelic=12`; `SkipBossRelic=13`; `SelectInstance=14`; `DeselectInstance=15`; `ConfirmSelection=16`; `CancelSelection=17`; `RevealBoard=18`; `SingingBowl=19`; `Proceed=20`; `SourceOption=21`

#### `ApiErrorKind`
`MalformedInput=1`; `StaleProvenance=2`; `ForeignProvenance=3`; `IllegalCandidate=4`; `SourceRefusal=5`; `AdministrativeFailure=6`; `PoolNotReady=7`; `PoolFaulted=8`; `PoolClosed=9`

#### `AttemptState`
`Available=1`; `Attempted=2`; `Matched=3`; `Missed=4`; `Removed=5`

#### `BottleKind`
`None=0`; `Attack=1`; `Skill=2`; `Power=3`

#### `CardFlagKind`
`Retain=1`; `SelfRetain=2`; `Exhaust=3`; `ExhaustOnce=4`; `ExhaustOnFire=5`; `Ethereal=6`; `FreeOnce=7`; `Autoplay=8`; `IgnoreEnergyTotal=9`

#### `CardMutationKind`
`Upgrade=1`; `Misc=2`; `BaseCost=3`; `TurnCost=4`; `Flags=5`; `Growth=6`; `Membership=7`

#### `CardPile`
`Draw=1`; `Hand=2`; `Discard=3`; `Exhaust=4`; `Limbo=5`; `InPlay=6`

#### `CardZone`
`Hand=1`; `Draw=2`; `Discard=3`; `Exhaust=4`; `Limbo=5`; `InPlay=6`; `OtherPublic=7`

#### `Character`
`Ironclad=1`; `Silent=2`; `Defect=3`; `Watcher=4`

#### `ChestKind`
`Ordinary=1`; `Boss=2`

#### `ClaimStateKind`
`Open=1`; `Selected=2`; `Claimed=3`; `Skipped=4`

#### `CommitDispositionTag`
`Mandatory=1`; `Optional=2`; `NoCommit=3`

#### `ContentNamespace`
`Card=1`; `Relic=2`; `Potion=3`; `Enemy=4`; `Encounter=5`; `Event=6`; `Power=7`; `Intent=8`; `Orb=9`; `SourceOption=10`; `RoomObject=11`; `PublicFact=12`; `EffectSite=13`

#### `ContinuationPhaseKind`
`Creation=1`; `Neow=2`; `Map=3`; `Room=4`; `Combat=5`; `Decision=6`; `Selection=7`; `Reward=8`; `Event=9`; `Shop=10`; `Campfire=11`; `Chest=12`; `Handoff=13`; `Terminal=14`

#### `CoreActionTag`
`EnteredAction=1`; `OrdinaryAction=2`; `PreTurnAction=3`; `CardQueueEntry=4`; `MonsterSnapshot=5`; `RoomEffect=6`; `DungeonEffect=7`; `Soul=8`

#### `CostKind`
`Ordinary=1`; `X=2`; `Unplayable=3`

#### `CostLayerKind`
`Base=1`; `Combat=2`; `Turn=3`; `UntilPlayed=4`; `FreeOnce=5`; `XCapture=6`

#### `DecisionKind`
`None=1`; `Neow=2`; `Map=3`; `Event=4`; `Shop=5`; `Campfire=6`; `Chest=7`; `Reward=8`; `BossRelic=9`; `Combat=10`; `DeckSelection=11`; `CombatSelection=12`; `MatchBoard=13`; `PotionSelection=14`; `Handoff=15`

#### `EnemyDeathState`
`Alive=1`; `Dying=2`; `Dead=3`

#### `EventPhase`
`Created=1`; `Revealed=2`; `OptionsPresented=3`; `Selected=4`; `Resolving=5`; `Suspended=6`; `Complete=7`

#### `FaceState`
`NotApplicable=0`; `FaceDown=1`; `Exposed=2`; `RememberedHidden=3`; `Removed=4`

#### `HandleKind`
`PermanentCard=1`; `CombatCard=2`; `Enemy=3`; `Relic=4`; `Offer=5`; `Continuation=6`; `EffectInvocation=7`; `Combat=8`; `Room=9`; `Potion=10`; `MapNode=11`; `Selector=12`

#### `HistoryFactKind`
`VisitedRoom=1`; `RevealedEvent=2`; `ChosenOption=3`; `RevealedBoard=4`; `PublicPlay=5`; `PublicDiscard=6`; `PublicCounter=7`

#### `HookStateKind`
`Pending=1`; `Entered=2`; `Committed=3`; `Complete=4`

#### `KeyKind`
`Ruby=1`; `Sapphire=2`; `Emerald=3`

#### `KnowledgeFactKind`
`DrawCount=1`; `DiscardCount=2`; `ExhaustCount=3`; `KnownNext=4`; `KnownBottom=5`; `ScryWindow=6`; `RememberedBoard=7`; `AnnouncedBoss=8`; `SourceRevealed=9`

#### `MapHistoryKind`
`Visited=1`; `Revealed=2`; `BurningMarked=3`; `Current=4`

#### `MapMutationKind`
`Visited=1`; `Revealed=2`; `BurningMarker=3`; `CurrentNode=4`

#### `MaskBehavior`
`NotEmitted=1`; `PresentUnknownZero=2`

#### `ObjectState`
`Neutral=0`; `Available=1`; `Disabled=2`; `Selected=3`; `ResolvingPublicStage=4`; `Completed=5`; `Removed=6`

#### `OfferKind`
`Card=1`; `Relic=2`; `Potion=3`; `Gold=4`; `Key=5`; `Hp=6`; `MaxHp=7`; `Removal=8`; `Upgrade=9`; `Transform=10`; `SourceOption=11`

#### `OfferPriceKind`
`None=0`; `Gold=1`; `HpLoss=2`; `Damage=3`; `MaxHp=4`; `RegisteredComposite=5`

#### `OfferStatus`
`Offered=1`; `Selected=2`; `Claimed=3`; `Skipped=4`; `Removed=5`; `Disabled=6`

#### `ParameterKind`
`Count=1`; `ChosenPosition=2`; `OrderedSelectionMember=3`; `PublicX=4`; `OptionArgument=5`; `PublicPriceComponent=6`; `SourcePublicFlag=7`

#### `Partition`
`Train=1`; `Selection=2`; `Final=3`; `Test=4`

#### `PoolStateKind`
`Uninitialized=1`; `Ready=2`; `Operating=3`; `Faulted=4`; `Closed=5`

#### `RecordingMode`
`Off=0`; `FromStart=1`

#### `RelationKind`
`RewardExclusive=1`; `RewardParent=2`; `PublicCopySource=3`; `PublicPermanentOwner=4`; `PublicSelectionSource=5`; `PublicKnownOrder=6`

#### `RelicMutationKind`
`Counter=1`; `Used=2`; `Ownership=3`

#### `ReplayEndStatus`
`Death=1`; `Act2Ready=2`; `Interrupted=3`; `ConstructionFailure=4`

#### `ReplayStage`
`Header=1`; `Initialization=2`; `PreObservation=3`; `Menu=4`; `Action=5`; `Transition=6`; `PostObservation=7`; `Final=8`

#### `ResetMode`
`SingleManual=1`; `VectorSameStep=2`

#### `ResourceKind`
`Hp=1`; `MaxHp=2`; `Gold=3`; `Block=4`; `Energy=5`; `MaxEnergy=6`; `PotionSlot=7`; `KeyFlag=8`

#### `RevealState`
`Hidden=1`; `PartiallyRevealed=2`; `Revealed=3`; `Remembered=4`

#### `RoomKind`
`Unknown=0`; `Monster=1`; `Elite=2`; `Boss=3`; `Event=4`; `Shop=5`; `Rest=6`; `Treasure=7`; `BossTreasure=8`

#### `RoomMutationKind`
`Phase=1`; `OfferState=2`; `ObjectState=3`; `Continuation=4`

#### `RoomObjectKind`
`BoardPosition=1`; `EventOption=2`; `CampfireOption=3`; `Chest=4`; `RewardContainer=5`; `ShopService=6`; `SourcePublicState=7`

#### `RoomPhase`
`Entering=1`; `Active=2`; `Resolving=3`; `Complete=4`; `Interrupted=5`

#### `SeedKind`
`Root32=1`; `Seed64=2`; `Entropy32=3`

#### `SiteNamespace`
`Card=1`; `Relic=2`; `Potion=3`; `Enemy=4`; `Event=5`; `Run=6`; `Acquisition=7`; `Power=8`

#### `SiteOperation`
`Use=1`; `ActualDraw=2`; `ActualExhaust=3`; `StartTurn=4`; `EndTurn=5`; `PreDraw=6`; `Equip=7`; `RewardBuild=8`; `Claim=9`; `Purchase=10`; `Transform=11`; `Remove=12`; `Copy=13`; `Insert=14`; `KeyCommit=15`; `Selection=16`; `Resume=17`; `Transition=18`; `Damage=19`; `Heal=20`; `Lifecycle=21`; `PoolSelect=22`; `Initialize=23`; `Restock=24`; `Preview=25`; `ResourceMutation=26`

#### `SlotStateKind`
`Uninitialized=1`; `Live=2`; `Terminal=3`

#### `SortDirection`
`Ascending=1`; `Descending=2`

#### `Stance`
`None=0`; `Neutral=1`; `Wrath=2`; `Calm=3`; `Divinity=4`

#### `TerminalOutcomeKind`
`Death=1`; `Act1CompleteAct2Ready=2`

#### `TransitionCheckpointTag`
`BossRewardResolved=1`; `MandatoryChoicesResolved=2`; `BossRelicResolved=3`; `HooksResolved=4`; `Act2Initialized=5`; `Act2Ready=6`

#### `ValueKind`
`Signed=1`; `Boolean=2`; `Enum=3`; `Binary32Bits=4`; `RowRef=5`

#### `WireKind`
`I64=1`; `I8Mask=2`; `RowRefPairI64=3`; `F32BitsInI64=4`

#### `ApiStage`
`Preflight=1`; `Construction=2`; `Dispatch=3`; `CoreTransition=4`; `Projection=5`; `Packing=6`; `OwnershipTransfer=7`; `ReplayVerify=8`; `Close=9`

### A.4 Canonical descriptor and manifest bytes

The descriptor is the exact `Descriptor` record above; its object keys are exactly its declared field names, recursively. Unknown or missing fields, duplicate JSON keys, duplicate IDs/names/codes, invalid nested record fields, unknown enum codes/tags, and out-of-order declared arrays reject. JSON object keys are sorted by unsigned ASCII byte order. Arrays are never reordered by the serializer: fixed rows follow the schema list, tables by ascending table ID, each table/fixed-row columns by contiguous ordinal from zero, codebooks by name bytes and members by name bytes, references/visibility by field bytes, ordering by table bytes, actions/parameters by numeric code, compatibility and component hashes by name bytes. Arrays whose keys repeat are invalid. The serializer emits no whitespace; only null, booleans, signed-i64 integers, strings, arrays, and objects are permitted; floats and integers outside `[-2^63,2^63-1]` reject. Integer form is shortest base-10, no plus, leading zero, or negative zero. Strings are Unicode scalar sequences encoded as UTF-8; quote and backslash are escaped as `\"` and `\\`; every U+0000..U+001F is `\u00xx` with lowercase hex; every other scalar is emitted directly as UTF-8, with no normalization. Parser input is accepted only if strict UTF-8 JSON parses without duplicate keys and byte-for-byte reserialization equals the original bytes. This descriptor/manifest JSON grammar is distinct from the action-JSON grammar in §10; action JSON rules do not implicitly define descriptor bytes.

A production descriptor additionally requires fixed rows exactly `context` (12 ordered columns) then `player` (16); table IDs exactly 1..18 with widths `[7,3,9,12,4,4,8,5,5,9,7,5,6,8,7,8,6,5]` (118 table columns total); one visibility record for every fully qualified fixed/table carrier field; every `ActionKind` and `ParameterKind` code/name pair; the exact complete `enums` codebook set with wire width 4 and exact member/code pairs in this appendix; the exact `union/<name>` codebook set with wire width 2 and lexicographically derived tag/code pairs; and exactly 10 component hashes. An empty descriptor is only a serializer test vector and fails production-descriptor validation. This shape gate does not close any producer-supplied member/domain/visibility/equality join.

Component bytes are themselves canonical encodings of their versioned closed component records. For component `(name,schema_id,bytes)`, `component_hash = SHA256(ASCII("spire-lab-act1-descriptor-component-v1") || 00 || U64BE(len(UTF8(schema_id))) || UTF8(schema_id) || U64BE(len(bytes)) || bytes)`. Descriptor `component_hashes` contains the exact lowercase 64-hex digest and schema ID for each component, sorted by ASCII name; the set is exactly `{observation, action, content-site, visibility, numeric-fields, native-rng, private-core, replay, task, delivery}` once all inputs close. While any import is unresolved, no final descriptor, component, or composed fingerprint is claimed; a structural test vector below is not a production descriptor.

For canonical descriptor byte string `D`, `descriptor_preimage = ASCII("spire-lab-act1-descriptor-v5") || 00 || U64BE(len(D)) || D`, and `descriptor_digest = SHA256(descriptor_preimage)`. No self-hash is included: component hashes are computed first, inserted into D, then D is hashed. Changing any field meaning/order, codebook, visibility, action/site relation, core/replay law, or imported component changes the descriptor version and identity. This is a precise preimage rule, not a claimed final hash.

#### Canonical vectors

- For input object `{"b":2,"a":"é"}`, canonical bytes are hex `7b2261223a22c3a9222c2262223a327d` (UTF-8 for `{"a":"é","b":2}`).
- String-escape vector for value containing quote, backslash, LF, TAB, slash, and `é` has canonical bytes `7b2273223a225c225c5c5c75303030615c75303030392fc3a9227d`; controls use lowercase `\u00xx`, while slash and non-ASCII scalar bytes remain literal.
- Wire-code checks: `SiteNamespace.Run=6`, `RoomObjectKind.Chest=4`, `ActionKind.SourceOption=21`; `CommittedEffect.Card` is the lexicographic tag code shown by its `union/CommittedEffect` codebook.
- Component hash check: schema ID `schema.v1` and component bytes `{}` yield `0fbaea9ccdb4ed8fbf9b55b411b2b9f9e4218988a4c79644d5a90eb9a83f3320`.
- Empty structural candidate-5 descriptor vector serializes to the exact bytes asserted by the private correction test. It is only a serializer vector: an empty table set is not a valid production Act 1 observation descriptor.
- RowRef vectors: absent `(0,-1)`; first row `(1,0)` with count 1; last representable slot row `(18,I64_MAX-1)` with count I64_MAX; reject `(0,-2)`, `(1,-1)`, unknown table 19, row equal to count, and segment count I64_MAX+1.

### A.5 Recursive grammar validation and boundary

The private correction validator resolves every referenced name through the finite primitive/enum/record/union registry, checks each record's complete ordered field manifest, every union's complete required variant manifest and every enum's frozen name/code manifest, rejects duplicate/malformed enum names/codes and validates nested values recursively. The public record tables above are the normative full expansion; no `Opaque`, arbitrary byte payload, untyped dictionary, missing-field fallback, or producer-defined structure is permitted. Producer-provided *values* and source-member sets remain open only at the explicitly identified joins in §11. Private negative controls omit nested `CheckpointKey.ordinal`, remove a required record field or `CommittedEffect.Resource` variant, mutate a frozen enum code, use an unknown nested union tag, and feed noncanonical descriptor bytes; each must fail. These structural controls establish only contract consistency, not runtime codec, source equality, or gameplay behavior.

<!-- END GENERATED NATIVE GRAMMAR APPENDIX -->
