# Act 1 native semantic and numeric contract closure

**Status: standalone candidate contract for review; not approved behavior, final schema, implementation authorization, or proof that every source mechanic is closed.**

This document closes a specific native-contract gap: it defines a checkable field-level wire contract, semantic-key assignment rule, complete action tuple shape, effect-site identity/binding record, and detached profile/time/handoff attachments. It deliberately does not claim an immutable final dictionary/hash while required content/site reconciliation and human choices remain open. It supplements the reviewed Act 1 specification; it does not widen any combat-v1 identity.

## 1. Evidence and decision labels

- **Source fact (F)** means a fact reported by the reviewed static/source records. The seven normative records linked below contain paired content or numeric findings; they are not runtime tests and do not prove every transitive composition.
- **Encoding proposal (P-E)** is an exact candidate representation/domain. It is not a claim that the game uses that representation or that the whole specification approved it.
- **Human option (H)** is a tagged alternative whose selection remains open. No option below silently supplies a default.
- **Unresolved source interface (U-S)** means the native contract accepts a symbolic named input from the RNG/maps or execution work; this document does not invent that answer.

No original game initializer or runtime was executed. The authorized archive identity was checked against its retained pin before and after a narrowly scoped static pass: 12 named shared class entries were extracted to private evidence and inspected with the pinned `javap` (`-p -s`); no dependencies were resolved and no class was initialized. Their declarations support the direct-member rows below, not behavior of every method or every subclass. Separately, retained source-method joins cited in §2.8 support the burning-elite node-marker fact. Other claims remain attributed to the linked reviewed public records and are not upgraded to runtime evidence.

Reviewed normative interfaces (all standalone public records): [enemy numeric and callback records](2026-10-19-act1-content-enemies.md), [Ironclad/colorless/Status/Curse and shared card rules](2026-10-19-act1-content-red-colorless.md), [Silent paired records](2026-10-19-act1-content-silent.md), [Neow/events](2026-10-19-act1-content-event-neow.md), [Watcher paired records](2026-10-19-act1-content-watcher.md), [economy/rewards/chests/potions](2026-10-19-act1-content-economy-potions.md), and [Defect paired records](2026-10-19-act1-content-defect.md). These link to primary-record scope; they do not imply transitive closure or an approved unified content catalog.

## 2. Exact field transport contract

### 2.1 Common carriers and validity

**P-E.** Every slot-local packed table is a variable-length sequence of rows with the fixed width below. Native numeric payload is signed `int64`; its same-shaped knowledge mask is `int8` restricted to `0` or `1`. Offsets are signed `int64[B+1]`, start at zero, nondecreasing, end at row count, and count rows (not scalar cells). For slot `s`, every row reference is local to `[offset[s], offset[s+1])`; `-1` is the only absent-row sentinel where a field permits it. Validate type, dimensions, mask, enums, refs, spans and all arithmetic before indexing or allocating. Unknown cells contain numeric zero with mask zero; known zero is mask one. A permitted absent sentinel is known. No padding/fake absent row, dense action carrier, guessed row/content maximum, paging, or truncation.

The public descriptor has fixed `context[12]` and `player[16]` rows plus these 18 tables. The order and exact column names are normative for this candidate:

| ID | Table (width) | Ordered columns; scalar meaning and candidate domain |
|---:|---|---|
| 1 | `nodes` (7) | `x,y`: signed source coordinates; `room_kind`: `RoomKind`; `visited,reachable`: boolean; `burning_marker`: boolean or known absent `-1` only if descriptor permits; `boss_code`: room-qualified `RoomObjectKind`/boss key, else `-1`. Coordinates use source int domain (U-S B02/B09); no coordinate clipping. |
| 2 | `edges` (3) | `from_node_ref,to_node_ref`: required local refs to table 1; `currently_selectable`: boolean from current legal traversal predicate. |
| 3 | `permanent_cards` (9) | `card_code`: public card-kind key; `upgrades`: card-kind-specific nonnegative upgrade domain; `misc`: card-kind-specific signed integer domain (often nonpublic, therefore masked unless public); `bottle_kind`: `BottleKind`; `removable,upgradeable`: booleans; `base_cost_kind`: `CostKind`; `base_cost`: signed-i32 source value, including ordinary zero and distinct -1/-2 sentinels, when card identity/cost is source-entitled; `public_position`: canonical visible ordinal. Never coerce instance history to a kind-wide counter. |
| 4 | `combat_cards` (12) | `card_code`; `upgrades`; `misc`; `zone`: `CardZone`; `position`: zone-local visible ordinal; `cost_kind`; `base_cost`; `turn_cost`: signed-i32 current source value, independently sourced; `public_owner_ref`: local owner ref to player sentinel or permitted table; `retained,exhaust_on_use,ethereal`: booleans. `base_cost` and `turn_cost` are distinct fields; cost-bearing cells are masked if card identity/value is not source-entitled. |
| 5 | `relics` (4) | `relic_code`; `public_position`; `counter`: relic-specific signed source-int domain; `used`: boolean. A relic's constructor default/counter lifetime does not define a universal bound. |
| 6 | `potions` (4) | `belt_slot`: actual slot index; `occupied`: boolean; `potion_code`: kind key or known `-1` for empty slot; `potency`: potion-specific signed source-int domain, masked when not public/applicable. Empty slots are real slots, not max-capacity padding. |
| 7 | `actors` (8) | `enemy_code`; `formation_position`; `hp,max_hp,block`: source-int domains with exact transition rules, not universal clipping; `intent_code`: owner-qualified intent key; `intent_damage`: source payload integer; `intent_hits`: source payload count. Hidden/suppressed intent and all derived payload fields are masked, not reconstructed. |
| 8 | `powers` (5) | `owner_table,owner_ref`: typed owner pair; `power_code`; `amount`: power-specific signed source-int domain; `public_position`: canonical order. Per-instance duration/amount is not a generic saturating counter. |
| 9 | `orbs` (5) | `position`; `orb_code`; `passive_amount,evoke_amount,counter`: orb-specific signed source-int domains. Orb slots are ordered; absence is not a fabricated row. |
| 10 | `offers` (9) | `offer_kind`; `content_code`; `target_table,target_ref`: typed optional target; `amount`: offer-specific amount domain; `price_kind`; `price_amount`: settled signed-i32 payment/display amount for an entitled price-bearing offer; raw source-float intermediates are excluded; `status`: `OfferStatus`; `public_position`. Offered, claimed, skipped and acquired are different lifecycle states. |
| 11 | `room_objects` (7) | `object_kind`; `public_position`; `face_state`: `FaceState`; `content_kind,content_code`: typed object content; `object_state`: `ObjectState`; `remaining_attempts`: exact source counter when known/public. No attempt field is decremented by transport. |
| 12 | `selection` (5) | `source_table,source_ref`: typed local source; `selected_ordinal`: selected-sequence index or `-1`; `selected,eligible`: booleans. Sequence order is preserved where source selection is ordered; selected subset is not sorted to canonicalize it. |
| 13 | `knowledge` (6) | `fact_kind`; `subject_table,subject_ref`; `position`; `value_kind`; `value`. Only legitimate revealed facts; the value's kind-specific domain is fixed by the fact dictionary. Hidden and present-unknown differ. |
| 14 | `properties` (8) | `owner_table,owner_ref`; `property_code`; `ordinal`; `value_kind`; `value`; `ref_table,ref_row`. Key is `(owner, property, ordinal)`; duplicate keys reject. This is a closed 45-code extension set in §3, not an open bag. |
| 15 | `history` (7) | `fact_kind`; `ordinal`; `subject_kind,subject_code`; `value_kind,value`; `visible_floor`. Public, currently entitled history only; private source counters and hidden RNG history never leak. |
| 16 | `candidates` (8) | `action_kind`; `source_table,source_ref`; `target_table,target_ref`; `option_code`; `parameter_start,parameter_count` into table 17's slot-local segment. Rows map privately to one exact current core choice. |
| 17 | `candidate_parameters` (6) | `parameter_kind`; `ordinal`; `value_kind`; `value`; `ref_table,ref_row`. Ordered within candidate; interpretation is action- and option-qualified. It is not caller-writable input. |
| 18 | `relations` (5) | `relation_kind`; `from_table,from_ref`; `to_table,to_ref`. Only enumerated typed relations in §3; duplicate rows preserve multiplicity only where the relation definition expressly says so. |

The 12 context fields, in order, are `phase,decision_kind,act,global_floor,current_node_ref,turn,terminal_outcome,parent_context_kind,selected_count,min_select,max_select,ordered`. The 16 player fields are `character,ascension,hp,max_hp,gold,block,energy,max_energy,draw_per_turn,potion_capacity,ruby,sapphire,emerald,stance,orb_capacity,mantra`. Both fixed rows use `int64` values and cellwise `int8` masks. These widths and values are **encoding proposals**; listed source helper bounds below are not request-side clamps.

`U=2^63-1` is an offset/count/carrier representational ceiling, not the game's content cap. Validate multiplication, additions, conversions, host-size and allocation before mutation. If legal state cannot be represented or allocated, fault administratively with provenance; do not turn exhaustion into a legal-rule rejection, row omission, shorter menu, defeat, retry, or gameplay content cap.

### 2.2 Source numerics versus carrier-only numeric bounds

A transport cell's signed-64 range is established only by the proposed carrier, not by source behavior. Source evidence and bounds are separate:

| Domain | Status and required mapping |
|---|---|
| Signed integer / count | A direct Java `int` field has representable source domain exactly `[-2^31, 2^31-1]`; its `int64` cell is the exact sign-extension and wider input is an encoding error, never narrowing. §2.6 identifies a source member or exact derived producer for every candidate field, and states the broad primitive domain. This is not a claim that every bit pattern is reachable: owner-specific transition predicates, sentinels, helper caps and narrower reachable ranges remain source-law obligations named per row where unproved. Collection ordinals/counts derive from source `int` size/index; they do not establish game-content maxima. |
| `binary32` | Preserve raw IEEE-754 binary32 bits when a source field/price/quota is a float; do not round-trip through decimal or binary64. Apply the source operation's rounding at its exact boundary. The normative records identify float-based costs/rounding where applicable; this proposal does not claim all float members or all input values have been audited. |
| boolean | Encode exactly `0` or `1`; reject other values. Source's boolean truth is not encoded as numeric truthiness. |
| counts and widths | Table row counts and offsets use `0..U` only as encoding limits. Candidate parameter count and selection count are additionally constrained by the exact currently offered source menu, not a constant cap. |
| mutable counters | No global clamp/saturating rule. Per-kind source counter transition and initial value must be specified; an observed caller/helper cap applies only at that helper. |
| helper-specific cap | The source `addEnergy`/`addBlock` helper caps at 999 at those call sites. It does not clip external inputs or establish 999 as a general cap for incoming Block, enemy Block, relic counters, gold, HP, energy grants outside that helper, or any packed value. |
| overflow / nonfinite | Source arithmetic behavior is a source-specific rule and may not be replaced by carrier wrapping. If a legal source value cannot fit or be represented exactly by selected encoding, raise an administrative numeric/encoding fault before presenting a misleading observation. Do not reject/truncate a legal game row as a gameplay rule. |

The schema now has a candidate mapping row for every exposed field; it is **not** a proof that every live mutable source member is exposed, every owner-specific legal/reachable subset is known, or every call boundary is joined. Reconcile the source-member inventory against the full normative records, properties, and source methods before asserting B09 closure. That is a specific residual, not a blanket “all fields fit int64” claim.

### 2.3 Value kinds, references, and binary32

`value_kind` is a closed tagged union: `1 signed-i64`, `2 boolean`, `3 enum-code`, `4 binary32-raw-u32-in-i64` (upper 32 bits zero), `5 local-row-ref`. A row reference uses `(ref_table,ref_row)` with known integer local row; scalar is zero for kind 5. Other kinds require both reference columns to be absent sentinel. Knowledge/history `value_kind` is restricted to the explicitly enumerated scalar subset and exact fact-specific dictionary. Reject unknown tags, nonzero unused bits, malformed refs, overflow, noncanonical duplicate properties, or reference to another slot even when globally in range. This generic primitive union does not permit a field to change wire kind: table 10 `price_amount` is settled signed-i32 only; source binary32 intermediates are excluded.

Binary32 validation treats bits as an encoding, not a float permission: the field's source contract specifies whether NaN/infinity/signed-zero are legal; until established, these values are not admitted as ordinary numeric content. The present price cell is not a float field: settled prices are signed-i32, while raw generation/intermediate floats stay source-private and are excluded. Any future explicitly public float property requires a separately proven owner/member and visibility row; no generic float escape hatch is implied.

### 2.4 Field-by-field domain signature

The following signature is positionally aligned with the ordered columns in §2.1. It assigns a validator tag to **each** packed field, rather than inferring a row-wide type. `key(NS)` means a code in the named frozen content namespace; `ref(T)` is a slot-local row in table T; `opt-ref(T...)` additionally permits the documented absent pair. `src-int(field)` and `src-count(field)` are source-domain values carried in int64, not a declaration that every such source is Java `int`. This is a complete candidate wire-tag assignment; it is not a complete proof of every source primitive/range.

| ID | Table | Ordered validator tags (same order as §2.1 columns) |
|---:|---|---|
| 1 | `nodes` | `x,y:src-int(map-coordinate)`; `room_kind:enum(room_kind)`; `visited,reachable:boolean`; `burning_marker:boolean-or-absent`; `boss_code:opt-key(room-object/boss)` |
| 2 | `edges` | `from_node_ref,to_node_ref:ref(1)`; `currently_selectable:boolean` |
| 3 | `permanent_cards` | `card_code:key(card)`; `upgrades:src-count(card-upgrade)`; `misc:src-int(card-instance-misc)`; `bottle_kind:enum(bottle_kind)`; `removable,upgradeable:boolean`; `base_cost_kind:enum(cost_kind)`; `base_cost:conditional-src-int(cost)`; `public_position:public-ordinal` |
| 4 | `combat_cards` | `card_code:key(card)`; `upgrades:src-count(card-upgrade)`; `misc:src-int(card-instance-misc)`; `zone:enum(combat_zone)`; `position:public-ordinal` (masked for unordered hidden zones); `cost_kind:enum(cost_kind)`; `base_cost,turn_cost:conditional-src-int(distinct-cost-fields)`; `public_owner_ref:opt-ref(3)`; `retained,exhaust_on_use,ethereal:boolean` |
| 5 | `relics` | `relic_code:key(relic)`; `public_position:public-ordinal`; `counter:src-int(relic-qualified-counter)`; `used:boolean` |
| 6 | `potions` | `belt_slot:src-count(actual-slot-index)`; `occupied:boolean`; `potion_code:opt-key(potion)`; `potency:conditional-src-int(potion-qualified-potency)` |
| 7 | `actors` | `enemy_code:key(enemy)`; `formation_position:public-ordinal`; `hp,max_hp,block:src-int(actor-resource)`; `intent_code:owner-qualified-key(intent)`; `intent_damage:conditional-src-int(intent-payload)`; `intent_hits:conditional-src-count(intent-payload)` |
| 8 | `powers` | `owner_table:enum(owner-table)`; `owner_ref:typed-ref(owner_table)`; `power_code:key(power)`; `amount:src-int(power-qualified-amount)`; `public_position:public-ordinal` |
| 9 | `orbs` | `position:public-ordinal`; `orb_code:key(orb)`; `passive_amount,evoke_amount,counter:src-int(orb-qualified-field)` |
| 10 | `offers` | `offer_kind:enum(offer_kind)`; `content_code:kind-qualified-key(content_kind)`; `target_table:enum(target-table)`; `target_ref:opt-ref(target_table)`; `amount:conditional-src-int(offer-qualified-amount)`; `price_kind:enum(price_kind)`; `price_amount:conditional-signed-i32-settled-amount(price-kind,owner,entitlement)`; `status:enum(offer_status)`; `public_position:public-ordinal` |
| 11 | `room_objects` | `object_kind:enum(room_object_kind)`; `public_position:public-ordinal`; `face_state:enum(face_state)`; `content_kind:enum(content_kind)`; `content_code:kind-qualified-key(content_kind)`; `object_state:enum(object_state)`; `remaining_attempts:conditional-src-count(object-qualified-attempts)` |
| 12 | `selection` | `source_table:enum(selection-source-table)`; `source_ref:typed-ref(source_table)`; `selected_ordinal:src-count-or-absent`; `selected,eligible:boolean` |
| 13 | `knowledge` | `fact_kind:enum(knowledge_fact)`; `subject_table:enum(knowledge-subject-table)`; `subject_ref:opt-ref(subject_table)`; `position:src-count(public-sequence-position)`; `value_kind:enum(value_kind[1..4])`; `value:fact-qualified-scalar` |
| 14 | `properties` | `owner_table:enum(property-owner-table)`; `owner_ref:typed-ref(owner_table)`; `property_code:enum(property_code[1..45])`; `ordinal:src-count(property-multiplicity-index)`; `value_kind:enum(value_kind[1..5])`; `value:property-qualified-scalar-or-zero-for-ref`; `ref_table:enum(property-ref-table)-or-absent`; `ref_row:ref-row-or-absent` |
| 15 | `history` | `fact_kind:enum(history_fact[1..7])`; `ordinal:src-count(public-history-position)`; `subject_kind:enum(history-subject-kind)`; `subject_code:kind-qualified-key(subject_kind)`; `value_kind:enum(value_kind[1..4])`; `value:history-fact-qualified-scalar`; `visible_floor:src-count(public-floor)` |
| 16 | `candidates` | `action_kind:enum(action_kind[1..21])`; `source_table:enum(action-source-table)`; `source_ref:typed-ref(source_table)`; `target_table:enum(action-target-table)-or-absent`; `target_ref:opt-ref(target_table)`; `option_code:owner-qualified-key(source-option)-or-none`; `parameter_start:slot-local-span-start(table17)`; `parameter_count:slot-local-span-count(table17)` |
| 17 | `candidate_parameters` | `parameter_kind:enum(parameter_kind[1..7])`; `ordinal:src-count(parameter-order)`; `value_kind:enum(value_kind[1..5])`; `value:action-option-qualified-scalar-or-zero-for-ref`; `ref_table:enum(parameter-ref-table)-or-absent`; `ref_row:ref-row-or-absent` |
| 18 | `relations` | `relation_kind:enum(relation_kind[1..6])`; `from_table:enum(relation-specific-from-table)`; `from_ref:typed-ref(from_table)`; `to_table:enum(relation-specific-to-table)`; `to_ref:typed-ref(to_table)` |

The validator tags above close carrier interpretation and point to each field's semantic domain family. They intentionally do **not** claim guessed source maxima. The residual is now field-qualified: exact source primitive, signedness, legal minimum/maximum, special sentinel set, float operation/rounding, overflow behavior, and source locator remain to be attached per source field for (a) map coordinates; (b) each card's upgrade/misc/base-cost/turn-cost fields; (c) each relic counter and potion potency; (d) actor HP/maxHP/Block and each intent payload; (e) each power/orb amount; (f) each offer amount/price and room-object attempt counter; (g) selected/knowledge/property/history/candidate parameter counts and tagged payloads. A source helper cap remains attached only to its call boundary. A later source-field map must have a row per live source field and explicit source-field → packed-field/property/excluded-field mapping; this signature is not offered as that missing row inventory.

The crosswalk replaces the missing *candidate-cell* inventory but not the missing complete source-to-schema equality. Exact open joins are: (1) each numeric field's set of methods writing/reading it and any owner-specific sentinel/range beyond the Java descriptor; (2) each source member absent from fixed/table columns, mapped to one exact property `(namespace,source key,property code,ordinal)` or a named private exclusion; (3) each conditional-visibility source predicate and its observation checkpoint; (4) every property-code owner/content tuple and its exact primitive/ref/domain; (5) current menu/action tuple producer and callback equality for tables 10–12, 16–18. A candidate source class family name is not a substitute for these joins.

### 2.8 Exact residual source joins by carrier

| Candidate carrier | Concrete joined facts in this revision | Exact remaining source equality/refinement |
|---|---|---|
| context[12] | `actNum`, `floorNum`, `turn` are source `int`; current map node is converted to a local typed ref; selection counts are source-list derived. | Decision/phase/parent/outcome are adapter states and need execution-lane dispatch predicates; current map-node availability, selection owner/min/max/ordered callback and legal menu membership require their exact source owner/method inventory. |
| player[16] | `chosenClass`, creature `currentHealth/maxHealth/gold/currentBlock`, energy `energy/energyMaster`, `gameHandSize`, `potionSlots`, `maxOrbs`, key booleans, stance object, and owner-filtered power amount are identified. | Reconcile start-turn draw caller; all field writers and operation-local clamps; exact Mantra power identity/callback; static settings-to-key lifecycle; legal/reachable ranges must not be inferred from descriptor width. |
| table 1 nodes | `x/y:I`, `room`, `taken:Z`, and node `hasEmeraldKey:Z` as the mechanic-bearing burning-elite designation. Retained `AbstractDungeon.generateMap`/`setEmeraldElite` methods select an elite node and set its node marker; `AbstractPlayer` pre-battle setup and `MonsterRoomElite` entry/reward logic read the current node marker. `Settings.hasEmeraldKey` is a distinct acquired run-key flag. | Exact public observation predicate/checkpoint for exposing the node designation; initialized zero-Elite image/history-conditioned measure; boss preview identity and current reachability/menu predicate. Node designation and acquired run key are distinct facts, not interchangeable fields. |
| table 2 edges | `getEdges()` endpoints converted to local node refs. | Source traversal predicate and exact menu-selection callback. |
| tables 3–4 cards | `cardID`, `timesUpgraded`, `misc`, `cost`, `costForTurn`, flags and source card predicates; 131 exact card ID/class pairs have literal dictionary rows. | Full four-character + colorless/Status/Curse/Note-reachable identity equality; all source operations mutating `misc`, upgrade/cost overflow and sentinels per owner; single `exhaust_on_use` field’s exact stage predicate; bottle identity UUID and visibility; source zone/order visibility; per-kind property vs exclusion mapping. |
| table 5 relics | `relicId`, `counter`, `usedUp`, actual owner-list order. | Full 141 primary relic ID aliases, all counter constructors/writers/sentinels/caps, per-relic properties and lifecycle hooks. No 999/global counter bound. |
| table 6 potions | `ID`, `slot`, potency accessor and actual list occupancy; 33 eligible source records are documented elsewhere. | Exact 33 source ID literal rows and alias collisions in this document; potency accessor inputs/overflow per potion; actual list/slot lifecycle and each potion property's visibility. |
| table 7 actors | `id`, HP/maxHP/block, intent/byte move and private `intentDmg/intentMultiAmt`; formation list index. | Full source enemy identity equality; per-owner move byte → semantic intent and legal payload/visibility; intent construction/update method bindings and all resource writers. |
| tables 8–9 powers/orbs | shared ID/amount/owner and `ID/passiveAmount/evokeAmount` members; list order; no common orb counter invented. | Full namespace identity and owner-qualified power amounts; subclass-only orb state; per-owner descriptor property and visibility relations. |
| table 10 offers | Offer record tagged to source object identity, current payload and list ordinal. | Exact source menu constructor → offer tuple → claim/pay callback per owner; price primitive is independently selected per offer and the generic row is not proof of any specific price mapping. |
| tables 11–12 objects/selections | Exact candidate tags/ref shape; no projection mutates source attempts/selection. | Room/event/campfire/chest source object/member inventories; every selector owner, ordered list, eligibility predicate, min/max and resume/cancel callback. |
| tables 13–15 knowledge/properties/history | Public/private separation and finite 9/45/7 codebooks. | Complete public owner-to-property/value map; actual knowledge reveal predicates; history fact production and visible floor; private-excluded field inventory. A closed code count does not prove owner/value equality. |
| tables 16–18 actions/parameters/relations | 21 action tags, typed refs, ordered parameter segments and six relation tags. | Exact current core candidate set equality; owner-qualified source action tuple/option code/parameter primitive and commit callback; every relation kind’s closed endpoint pair and multiplicity law. |

### 2.5 Fixed codebooks and typed-reference closure

These are closed candidate wire enums, not Java enum ordinals. Codes are literal and version-local:

| Codebook | Candidate numeric mapping |
|---|---|
| `room_kind` | `0 unknown, 1 monster, 2 elite, 3 boss, 4 event, 5 shop, 6 rest, 7 treasure, 8 boss_treasure` |
| `bottle_kind` | `0 none, 1 attack, 2 skill, 3 power` |
| `cost_kind` | `1 ordinary, 2 X, 3 unplayable` (ordinary includes zero; X is `-1`; unplayable is `-2`); derived from base cost only. Emit signed-i32 cost and kind together when source-entitled; otherwise mask both so a hidden cost is not disclosed by its tag. `turn_cost` is a separate signed-i32 value/mask and is not classified by this tag. |
| `combat_zone` | `1 hand, 2 draw, 3 discard, 4 exhaust, 5 in_play, 6 other_public`; hidden draw order is not an ordering feature |
| `offer_kind` | `1 card, 2 relic, 3 potion, 4 gold, 5 key, 6 hp, 7 max_hp, 8 removal, 9 upgrade, 10 transform, 11 source_option` |
| `price_kind` | `0 none, 1 settled_gold, 2 settled_hp_loss, 3 settled_damage, 4 settled_max_hp, 5 source_composite`; `price_amount` is settled signed-i32 only for registered entitled owners; composite components require separately registered properties, otherwise the offer is unemittable pending owner proof |
| `offer_status` | `1 offered, 2 selected, 3 claimed, 4 skipped, 5 removed, 6 disabled` |
| `room_object_kind` | `1 board_position, 2 event_option, 3 campfire_option, 4 chest, 5 reward_container, 6 source_public_state` |
| `face_state` | `0 not_applicable, 1 face_down, 2 exposed, 3 remembered_hidden, 4 removed` |
| `object_state` | `0 neutral, 1 available, 2 disabled, 3 selected, 4 resolving_public_stage, 5 completed, 6 removed` |
| `knowledge_fact` | `1 draw_count, 2 discard_count, 3 exhaust_count, 4 known_next_card, 5 known_bottom_card, 6 scry_window, 7 remembered_board, 8 announced_boss, 9 source_revealed` |
| `history_fact` | `1 visited_room, 2 revealed_event, 3 chosen_option, 4 revealed_board, 5 public_play, 6 public_discard, 7 public_counter` |
| `relation_kind` | `1 reward_exclusive, 2 reward_parent, 3 public_copy_source, 4 public_permanent_owner, 5 public_selection_source, 6 public_known_order` |
| `value_kind` | `1 signed-i64, 2 boolean, 3 enum-code, 4 binary32-raw-u32-in-i64, 5 local-row-ref`; tables 13/15 admit only 1–4 |
| fixed `context.phase` | `1 neow, 2 map, 3 room, 4 combat, 5 selection, 6 dead, 7 act2_ready` |
| fixed `context.terminal_outcome` | `0 none, 1 death, 2 act1_complete_act2_ready` |
| fixed `context.decision_kind` | `0 none, 1 neow, 2 map, 3 event, 4 shop, 5 campfire, 6 chest, 7 reward, 8 boss_relic, 9 combat, 10 deck_selection, 11 combat_selection, 12 match_board, 13 potion_selection, 14 handoff` |
| fixed `player.character` | `1 ironclad, 2 silent, 3 defect, 4 watcher` (creation admits only 1); ascension is exactly 0 in this Act-1 candidate |
| fixed `player.stance` | `0 none, 1 neutral, 2 wrath, 3 calm, 4 divinity` |

Typed reference pairs and permitted target tables (all row numbers are local to the same slot) are: table 2 endpoints→table 1; table 4 `public_owner_ref`→table 3; table 8 owner→player `(0,0)` or table 7; table 10 target→`{3,4,5,6,7,11}` or absent; table 12 source→`{3,4,6,10,11}`; table 13 subject→`{1,3,4,7,10,11}` or aggregate absence; table 14/17 kind-5 values→tables `1..13`; table 16 source→`{3,4,6,10,11,12}` or absent, target→player `(0,0)` or `{1,2,3,4,6,7,10,11}` or absent; table 18 endpoints→tables `1..13` with relation-qualified restrictions. Table 8's `(owner_table,owner_ref)` pair is valid only for `(0,0)` player or `(7,local_actor_row)` actor. A reference sentinel is exactly `(0,-1)`, never a bare `-1` with ambiguous type. Action-specific and relation-specific subsets are further constrained by their closed tuple dictionary and current source menu. This freezes candidate carrier allowlists; it does not establish source-private core membership. Exact content/owner-qualified option and intent codebooks remain source-catalog inputs, not numeric ordinals.


### 2.6 Source-field and numeric-domain crosswalk

This is the explicit 146-key projection join: 118 packed-table columns plus 12 context and 16 player cells. `Field key` is the independent candidate schema key, not a source name. Each row names either the actual source member/accessor or the exact adapter-derived producer and labels the primitive/domain and visibility. `F` rows use retained static/member evidence; `P-E` rows are representation rules; `U-S` rows name the precise caller/owner join still missing. A source object pointer is never serialized: source identities map through the literal dictionary and object relations map to slot-local row refs. The crosswalk does not claim that every `U-S` row is source-closed merely because its carrier/type is specified.

| Field key | Source identity/member or exact derived producer | Primitive/tag | Source domain and numeric/overflow contract | Public visibility | Evidence grade / exact remaining join |
|---|---|---|---|---|---|
| context.phase | adapter-owned decision/episode state; no corresponding game member | enum | fixed closed codebook in §2.5; reject all other codes | conditional | P-E: contract state emitted by the environment; source dispatch mapping remains in §4.2/§8 |
| context.decision_kind | adapter-owned decision/episode state; no corresponding game member | enum | fixed closed codebook in §2.5; reject all other codes | conditional | P-E: contract state emitted by the environment; source dispatch mapping remains in §4.2/§8 |
| context.act | AbstractDungeon.actNum:I | i32 | source Java int; sign-extend; no clip | conditional | F: static member declaration; candidate field is active act, not a new counter |
| context.global_floor | AbstractDungeon.floorNum:I | i32 | source Java int; sign-extend; no clip | conditional | F: static member declaration; valid range/reentry transitions are separate source rules |
| context.current_node_ref | AbstractDungeon.getCurrMapNode() → slot-local nodes row | typed-ref | typed local table-1 row or absent pair (0,-1) | conditional | P-E derived reference; never expose a JVM pointer |
| context.turn | GameActionManager.turn:I | i32 | source Java int; sign-extend; preserve source transitions | conditional | F: retained static member declaration |
| context.terminal_outcome | adapter-owned decision/episode state; no corresponding game member | enum | fixed closed codebook in §2.5; reject all other codes | conditional | P-E: contract state emitted by the environment; source dispatch mapping remains in §4.2/§8 |
| context.parent_context_kind | adapter-owned decision/episode state; no corresponding game member | enum | fixed closed codebook in §2.5; reject all other codes | conditional | P-E: contract state emitted by the environment; source dispatch mapping remains in §4.2/§8 |
| context.selected_count | current selection owner’s selected list and selector min/max | derived-i64 | source list size/min/max are i32; preserve exact values, reject malformed adapter value | conditional | U-S: bind each selector class/field/callback to the execution-lane menu inventory |
| context.min_select | current selection owner’s selected list and selector min/max | derived-i64 | source list size/min/max are i32; preserve exact values, reject malformed adapter value | conditional | U-S: bind each selector class/field/callback to the execution-lane menu inventory |
| context.max_select | current selection owner’s selected list and selector min/max | derived-i64 | source list size/min/max are i32; preserve exact values, reject malformed adapter value | conditional | U-S: bind each selector class/field/callback to the execution-lane menu inventory |
| context.ordered | adapter-owned selection descriptor | derived-bool | exactly 0 or 1 | conditional | P-E: orderedness is selection-owner metadata, not Java truthiness |
| player.character | AbstractPlayer.chosenClass | source-object | finite PlayerClass→character code map; standard creation permits IRONCLAD only | always | F: chosenClass is the actual run character; off-color cards do not replace it |
| player.ascension | AbstractDungeon.ascensionLevel:I | i32 | exactly source A0 value 0 in this profile | always | F: source run setting; not an agent-selected knob |
| player.hp | AbstractCreature.currentHealth:I | i32 | full signed i32 source field; no observation clamp; source damage/heal transitions apply | always | F: retained AbstractCreature field declaration; local heal/damage limits do not define all HP bounds |
| player.max_hp | AbstractCreature.maxHealth:I | i32 | full signed i32 source field; no universal cap | always | F: retained AbstractCreature field declaration; increaseMaxHp and source death paths remain operation-specific |
| player.gold | AbstractCreature.gold:I | i32 | full signed i32 field; no invented cap; gain/loss hooks own transitions | always | F: retained AbstractCreature field declaration; payment and loss are source operations |
| player.block | AbstractCreature.currentBlock:I | i32 | full signed i32 field; AbstractCreature.addBlock’s source result cap 999 applies only at that method; other assignment paths remain distinct | always | F: retained field and reviewed addBlock method record; not a general carrier cap |
| player.energy | EnergyManager.energy:I | i32 | full signed i32 field; source helper effects/caps remain call-site specific | always | F: targeted javap field descriptor; no universal 999 mapping |
| player.max_energy | EnergyManager.energyMaster:I | i32 | full signed i32 field; source transition-specific | always | F: targeted javap field descriptor |
| player.draw_per_turn | AbstractPlayer.gameHandSize:I | i32 | source Java int; adapter labels the source hand-draw request only where the setup path confirms this field is the request | conditional | U-S: cross-check all start-turn draw callers before treating gameHandSize as a universal draw count |
| player.potion_capacity | AbstractPlayer.potionSlots:I | i32 | source Java int; source-defined slot count, no padding to a guessed maximum | always | F: targeted javap field descriptor; capacity and actual list shape are distinct |
| player.ruby | Settings.hasRubyKey:Z | bool | 0/1 | always | F: retained AbstractDungeon static bytecode reads this source flag |
| player.sapphire | Settings.hasSapphireKey:Z | bool | 0/1 | always | F: retained AbstractDungeon static bytecode reads this source flag |
| player.emerald | Settings.hasEmeraldKey:Z | bool | 0/1 | always | F: retained AbstractDungeon static bytecode reads this source flag |
| player.stance | AbstractPlayer.stance:Lcom/megacrit/cardcrawl/stances/AbstractStance; (identity read from stance.ID) | source-object | finite owner-qualified stance code; no enum ordinal or display-name encoding | always | F: retained AbstractPlayer declaration and source stance ID read; candidate codebook is an explicit encoding proposal |
| player.orb_capacity | AbstractPlayer.maxOrbs:I | i32 | source Java int; no padding or carrier-derived cap | always | F: targeted AbstractPlayer declaration |
| player.mantra | AbstractPower.amount:I filtered by exact MantraPower source ID | i32 | power-specific signed i32; omit/mask when MantraPower is absent | conditional | F: AbstractPower.amount is source int; player-owned MantraPower is one owner-qualified power, not a global player scalar |
| table.1.x | MapRoomNode.x:I | i32 | full signed i32; sign-extend; no coordinate clipping | always | F: targeted javap field descriptor |
| table.1.y | MapRoomNode.y:I | i32 | full signed i32; sign-extend; no coordinate clipping | always | F: targeted javap field descriptor |
| table.1.room_kind | MapRoomNode.room:AbstractRoom | source-object | derive code from exact room subclass identity; 0 only for actual null/unknown per mask policy | always | F/P-E: room is an object reference, not an enum member |
| table.1.visited | MapRoomNode.taken:Z | bool | 0/1 | always | F: targeted javap field descriptor |
| table.1.reachable | current node + MapRoomNode edges + traversal guard | derived-bool | exact source legal-traversal predicate 0/1 | always | U-S: exact traversal callback/menu join belongs execution lane |
| table.1.burning_marker | `MapRoomNode.hasEmeraldKey:Z`, set for the selected elite node by retained `AbstractDungeon.generateMap` → `setEmeraldElite` methods; consumed by `AbstractPlayer` pre-battle setup and `MonsterRoomElite` entry/reward logic | bool | 0/1; this node designation is the burning-elite marker, not the acquired run-key flag | conditional | F: retained method-source evidence for map generation/selection and pre-battle/room consumption; exact public observation predicate/checkpoint remains U-S. Initialized zero-Elite image/history-conditioned measure remains open. |
| table.1.boss_code | MapRoomNode.room plus Act1 boss selection/room construction identity | source-object | kind-qualified boss key or absent; do not infer a hidden future boss from visible boss-room kind | unknown-mask-zero | U-S: map-preview boss identity source owner/member is not bound in retained map record |
| table.2.from_node_ref | MapRoomNode.getEdges()/source edge list | typed-ref | required slot-local table-1 row | always | F/P-E: edge endpoint is a derived local ref |
| table.2.to_node_ref | MapRoomNode.getEdges()/source edge list | typed-ref | required slot-local table-1 row | always | F/P-E: edge endpoint is a derived local ref |
| table.2.currently_selectable | active source node + reachable edge predicate | derived-bool | exact current menu membership 0/1; no stale-edge acceptance | always | U-S: exact current-choice callback mapping belongs execution lane |
| table.3.card_code | AbstractCard.cardID:Ljava/lang/String; | source-key | exact byte-sensitive source ID alias → frozen card key; reject unknown ID | always | F: all per-kind exact aliases are in the literal dictionary rows; shared field is the source cardID |
| table.3.upgrades | AbstractCard.timesUpgraded:I | i32 | exact signed i32; Searing Blow increments at each source upgrade operation; do not infer a saturating count cap | always | F: javap descriptors and paired card records; source arithmetic wraps only where a source Java-int operation does |
| table.3.misc | AbstractCard.misc:I | i32 | exact signed i32 including source sentinels/state; preserve source int wrap at source arithmetic operations | conditional | F: javap descriptors and paired card records; source arithmetic wraps only where a source Java-int operation does |
| table.3.bottle_kind | player relic bottle relation to card UUID | enum | 0 absent; 1 Attack Bottle; 2 Skill Bottle; 3 Power Bottle; derive from live bottle relic and card instance UUID, not card kind | conditional | P-E/U-S: projection rule as stated; source caller-specific predicates remain separately bound |
| table.3.removable | current source purge/transform selector predicate | derived-bool | exact current predicate result 0/1; source selector exceptions remain per owner | conditional | P-E/U-S: projection rule as stated; source caller-specific predicates remain separately bound |
| table.3.upgradeable | AbstractCard.canUpgrade():Z | bool | exact live predicate result 0/1 | conditional | P-E/U-S: projection rule as stated; source caller-specific predicates remain separately bound |
| table.3.base_cost_kind | AbstractCard.cost:I, classified only for a source-entitled card instance | enum | 1 ordinary for `cost>=0` (including zero), 2 X for `-1`, 3 unplayable for `-2`; other negative states retain source value but are not assigned a kind absent an owner-specific fact; mask kind with cost when identity/value is not entitled | conditional | F/P-E: source member and sentinels; source entitlement predicate and owner-specific negative-state reachability remain open |
| table.3.base_cost | AbstractCard.cost:I | i32 | exact signed i32, including ordinary zero, X `-1`, unplayable `-2`; never replace sentinel with zero. Mask with kind if card identity/value is not source-entitled. | conditional | F: javap descriptors and paired card records; source arithmetic wraps only where a source Java-int operation does; visibility is source-entitlement gated |
| table.3.public_position | CardGroup.group list index (permitted public group) | derived-i64 | nonnegative actual public sequence ordinal; count/offset arithmetic checked, no guessed maximum | always | P-E/U-S: projection rule as stated; source caller-specific predicates remain separately bound |
| table.4.card_code | AbstractCard.cardID:Ljava/lang/String; | source-key | exact byte-sensitive source ID alias → frozen card key; reject unknown ID | always | F: all per-kind exact aliases are in the literal dictionary rows; shared field is the source cardID |
| table.4.upgrades | AbstractCard.timesUpgraded:I | i32 | exact signed i32; Searing Blow increments at each source upgrade operation; do not infer a saturating count cap | always | F: javap descriptors and paired card records; source arithmetic wraps only where a source Java-int operation does |
| table.4.misc | AbstractCard.misc:I | i32 | exact signed i32 including source sentinels/state; preserve source int wrap at source arithmetic operations | conditional | F: javap descriptors and paired card records; source arithmetic wraps only where a source Java-int operation does |
| table.4.zone | CardGroup type membership | enum | closed candidate combat-zone mapping; group membership is unique at snapshot checkpoint | always | P-E/U-S: projection rule as stated; source caller-specific predicates remain separately bound |
| table.4.position | CardGroup.group list index | derived-i64 | nonnegative actual zone ordinal; masked for hidden-order zones unless source visibility grants order | conditional | P-E/U-S: projection rule as stated; source caller-specific predicates remain separately bound |
| table.4.cost_kind | AbstractCard.cost:I | enum | classifies `base_cost` only: ordinary including zero, X `-1`, unplayable `-2`; emit with base cost only under source entitlement, otherwise mask both. `turn_cost` is never classified by this tag. | conditional | F/P-E: field mapping/codebook; source setter/cost guards remain origin-specific |
| table.4.base_cost | AbstractCard.cost:I | i32 | exact signed i32; preserve ordinary zero and distinct `-1`/`-2` sentinel values. Mask with kind when identity/value is not source-entitled. | conditional | F: javap descriptors and paired card records; source arithmetic wraps only where a source Java-int operation does |
| table.4.turn_cost | AbstractCard.costForTurn:I | i32 | exact signed i32 when source-entitled; preserve independently of base-cost kind and `freeToPlayOnce`. Guarded negative setter behavior does not justify coercing or classifying this field; mask when not entitled. | conditional | F: javap descriptor and card-method records; setter guards/mutable free-once predicates are owner/call-site-specific; source arithmetic wraps only where a source Java-int operation does |
| table.4.public_owner_ref | no inherited owner member on AbstractCard | typed-ref | optional slot-local provenance relation; absent pair unless a source-owned public owner is proven | conditional | P-E/U-S: projection rule as stated; source caller-specific predicates remain separately bound |
| table.4.retained | AbstractCard.retain:Z OR selfRetain:Z | bool | source predicate is explicitly retain \|\| selfRetain; 0/1 | conditional | F: javap descriptors and paired card records; source arithmetic wraps only where a source Java-int operation does |
| table.4.exhaust_on_use | AbstractCard.exhaust:Z, exhaustOnUseOnce:Z, exhaustOnFire:Z | bool | U-S: define whether this column is a union or which stage’s flag it means; never collapse these source flags silently | conditional | F: javap descriptors and paired card records; source arithmetic wraps only where a source Java-int operation does |
| table.4.ethereal | AbstractCard.isEthereal:Z | bool | 0/1 | conditional | F: javap descriptors and paired card records; source arithmetic wraps only where a source Java-int operation does |
| table.5.relic_code | AbstractRelic.relicId:Ljava/lang/String; | source-key | exact case-sensitive source ID alias → relic key | always | F: targeted javap field descriptor |
| table.5.public_position | AbstractPlayer.relics list index | derived-i64 | nonnegative actual inventory ordinal | always | P-E: order is container order, not catalog order |
| table.5.counter | AbstractRelic.counter:I | i32 | full signed i32 including owner-specific sentinels; no universal clamp | conditional | F: targeted javap field descriptor; individual relic laws remain source-owner-qualified |
| table.5.used | AbstractRelic.usedUp:Z | bool | 0/1 | always | F: targeted javap field descriptor |
| table.6.belt_slot | AbstractPotion.slot:I | i32 | actual signed source slot index; empty real slots retain their slot ordinal | always | F: targeted javap field descriptor |
| table.6.occupied | AbstractPlayer.potions list entry presence | derived-bool | 0/1; no padding beyond source potionSlots | always | P-E: container occupancy is derived from list entries |
| table.6.potion_code | AbstractPotion.ID:Ljava/lang/String; | source-key | exact source factory ID → potion key; absent only for empty actual slot | conditional | F: targeted javap field descriptor; 33 eligible records are enumerated in the linked economy/potion source record; literal ID-to-key equality is not yet imported into this candidate |
| table.6.potency | AbstractPotion.potency:I then getPotency():I | i32 | accessor result as source int; Sacred Bark multiplication/overflow follows accessor; no universal positive bound | conditional | F: targeted javap descriptor + potion source record; no effect when potion absent |
| table.7.enemy_code | AbstractCreature.id:Ljava/lang/String; | source-key | exact source enemy identity alias → enemy key | always | F: retained shared creature declaration and 25-key source table |
| table.7.formation_position | MonsterGroup.monsters list index | derived-i64 | nonnegative actual living/dead actor position; do not compact actors and lose formation order | always | P-E: source container index |
| table.7.hp | AbstractCreature.currentHealth:I | i32 | full signed i32; source damage/death/rescue transitions govern, no display clamp | always | F: retained AbstractCreature field descriptor |
| table.7.max_hp | AbstractCreature.maxHealth:I | i32 | full signed i32; increaseMaxHp and special setters are distinct | always | F: retained AbstractCreature field descriptor |
| table.7.block | AbstractCreature.currentBlock:I | i32 | full signed i32; addBlock caps result at 999 only at that method; other paths are not clipped by this row contract | always | F: retained field and reviewed method-specific cap |
| table.7.intent_code | AbstractMonster.intent:Lcom/megacrit/cardcrawl/monsters/AbstractMonster$Intent; plus AbstractMonster.nextMove:B and owner ID | enum | owner-qualified exact move key, not byte ordinal/display text | conditional | F: targeted javap declarations; movement→semantic dictionary requires each owner's move record |
| table.7.intent_damage | AbstractMonster.intentDmg:I | i32 | source calculated payload; full signed i32; binary32 calculation/conversion occurs at source method boundaries | conditional | F: targeted javap descriptor; hidden/suppressed intent mask=0 |
| table.7.intent_hits | AbstractMonster.intentMultiAmt:I | i32 | source count/payload i32; not a generic nonnegative clamp | conditional | F: targeted javap descriptor; hidden/suppressed intent mask=0 |
| table.8.owner_table | AbstractPower.owner:AbstractCreature | source-object | owner tag 0 player / 7 actor; owner ref is typed and slot-local | always | F: power owner is an object reference; wire tag is a proposal |
| table.8.owner_ref | AbstractPower.owner:AbstractCreature | typed-ref | (0,0) player or table-7 actor row; no global pointer | always | F/P-E: owner pair is explicit |
| table.8.power_code | AbstractPower.ID:Ljava/lang/String; | source-key | exact case-sensitive power ID alias → power key | always | F: targeted javap field descriptor |
| table.8.amount | AbstractPower.amount:I | i32 | full signed i32; source power-specific mutation/expiry applies; no generic saturation | conditional | F: targeted javap field descriptor; source setter effects are owner-specific |
| table.8.public_position | AbstractCreature.powers list index | derived-i64 | nonnegative actual power order | always | P-E: source list position |
| table.9.position | AbstractPlayer.orbs list index | derived-i64 | nonnegative actual occupied slot index | always | P-E: actual slots only |
| table.9.orb_code | AbstractOrb.ID:Ljava/lang/String; | source-key | exact source orb ID alias → orb key | always | F: targeted javap field descriptor |
| table.9.passive_amount | AbstractOrb.passiveAmount:I | i32 | full signed i32 after source focus/lock-on hooks | conditional | F: targeted javap field descriptor; source calculation stages retain binary32 then narrowing semantics |
| table.9.evoke_amount | AbstractOrb.evokeAmount:I | i32 | full signed i32 after source hooks | conditional | F: targeted javap field descriptor |
| table.9.counter | no common AbstractOrb counter member in AbstractOrb declaration | not-applicable | mask=0 for source orb classes with no separately defined public counter; do not alias amount/evokeAmount | unknown-mask-zero | F/U-S: each supported subclass must be checked before any subclass-only counter is added |
| table.10.offer_kind | typed adapter sum of live CardRewardScreen/ShopScreen/EventRoom/Campfire/Chest/CombatRewardScreen branches | enum | closed §2.5 codebook, one current source offer type | always | F: economy/event records establish local owner-specific payload; U-S: complete owner→field→callback union and menu membership remain execution/content joins |
| table.10.content_code | current offered object's exact cardID/relicId/AbstractPotion.ID or explicit non-content offer tag | source-key | key is namespace-qualified; no display-name normalization | always | F: economy/event records establish local owner-specific payload; U-S: complete owner→field→callback union and menu membership remain execution/content joins |
| table.10.target_table | source continuation's target type (player/actor/card/object) or absent tag | enum | closed typed table tag or absent sentinel | always | F: economy/event records establish local owner-specific payload; U-S: complete owner→field→callback union and menu membership remain execution/content joins |
| table.10.target_ref | source continuation's exact selected target object mapped to its slot-local row | typed-ref | (0,-1) absent or current exact typed local row | conditional | F: economy/event records establish local owner-specific payload; U-S: complete owner→field→callback union and menu membership remain execution/content joins |
| table.10.amount | owner-specific offer constructor/payload (Offer source owner in the active menu) | i32 | signed source int32 where payload is integral; no universal cap | conditional | F: economy/event records establish local owner-specific payload; U-S: complete owner→field→callback union and menu membership remain execution/content joins |
| table.10.price_kind | owner-specific current settled payment branch (gold/HP-loss/damage/maxHP/composite) | enum | closed §2.5 codebook; emit only after owner payment semantics and exposure are registered | conditional | F: economy/event records establish local owner-specific payload; U-S: complete owner→field→callback union, menu membership and entitlement remain execution/content joins |
| table.10.price_amount | settled payment/display integer by registered owner: shop card `AbstractCard.price:I`; shop relic/potion members `StoreRelic.price`/`StorePotion.price` (Java int; initialized from `getPrice`, then integer discount/jitter settlement); purge member `ShopScreen.actualPurgeCost` (Java int); any other event/reward owner must independently name its exact settled member/callback before emission | i32 | one wire payload kind only: signed i32 settled amount, exact as paid/displayed. Float price-generation/intermediate arithmetic is source-method-local binary32 and excluded from this cell; no binary32 bits here. Mask unless source entitlement exposes offer and amount. | conditional | F: retained ShopScreen/StoreRelic/StorePotion source methods and economy record establish listed shop owner/member stages; U-S: complete event/reward/composite owner registry, exact callback equality and entitlement predicate remain open. Never infer an absent owner payload. |
| table.10.status | adapter offer lifecycle state tied to current source continuation | enum | closed lifecycle code; generated/claimed/skipped remain distinct | always | F: economy/event records establish local owner-specific payload; U-S: complete owner→field→callback union and menu membership remain execution/content joins |
| table.10.public_position | current owner menu/reward list ordinal | derived-i64 | exact actual list ordinal; never sort offered order | always | F: economy/event records establish local owner-specific payload; U-S: complete owner→field→callback union and menu membership remain execution/content joins |
| table.11.object_kind | source room/event/campfire/chest owner → object_kind adapter tag | adapter-value | closed §2.5 codebook or exact source field value for this owner; no source field is decremented by projection | conditional | P-E/U-S: current object-specific member/callback join is required; source object identity and continuation ownership are not a generic class |
| table.11.public_position | source room/event/campfire/chest owner → public_position adapter tag | adapter-value | closed §2.5 codebook or exact source field value for this owner; no source field is decremented by projection | conditional | P-E/U-S: current object-specific member/callback join is required; source object identity and continuation ownership are not a generic class |
| table.11.face_state | source room/event/campfire/chest owner → face_state adapter tag | adapter-value | closed §2.5 codebook or exact source field value for this owner; no source field is decremented by projection | conditional | P-E/U-S: current object-specific member/callback join is required; source object identity and continuation ownership are not a generic class |
| table.11.content_kind | source room/event/campfire/chest owner → content_kind adapter tag | adapter-value | closed §2.5 codebook or exact source field value for this owner; no source field is decremented by projection | conditional | P-E/U-S: current object-specific member/callback join is required; source object identity and continuation ownership are not a generic class |
| table.11.content_code | source room/event/campfire/chest owner → content_code adapter tag | adapter-value | closed §2.5 codebook or exact source field value for this owner; no source field is decremented by projection | conditional | P-E/U-S: current object-specific member/callback join is required; source object identity and continuation ownership are not a generic class |
| table.11.object_state | source room/event/campfire/chest owner → object_state adapter tag | adapter-value | closed §2.5 codebook or exact source field value for this owner; no source field is decremented by projection | conditional | P-E/U-S: current object-specific member/callback join is required; source object identity and continuation ownership are not a generic class |
| table.11.remaining_attempts | source room/event/campfire/chest owner → remaining_attempts adapter tag | adapter-value | closed §2.5 codebook or exact source field value for this owner; no source field is decremented by projection | conditional | P-E/U-S: current object-specific member/callback join is required; source object identity and continuation ownership are not a generic class |
| table.12.source_table | current source selector and ordered selection instance list → source_table | enum | selector-specific min/max/ordering and predicate; local row references only; count/ordinal source int32 | conditional | U-S: selector owner, actual predicate, and commit/cancel callback IDs must be joined to execution inventory |
| table.12.source_ref | current source selector and ordered selection instance list → source_ref | typed-ref | selector-specific min/max/ordering and predicate; local row references only; count/ordinal source int32 | conditional | U-S: selector owner, actual predicate, and commit/cancel callback IDs must be joined to execution inventory |
| table.12.selected_ordinal | current source selector and ordered selection instance list → selected_ordinal | derived-i64 | selector-specific min/max/ordering and predicate; local row references only; count/ordinal source int32 | conditional | U-S: selector owner, actual predicate, and commit/cancel callback IDs must be joined to execution inventory |
| table.12.selected | current source selector and ordered selection instance list → selected | derived-bool | selector-specific min/max/ordering and predicate; local row references only; count/ordinal source int32 | always | U-S: selector owner, actual predicate, and commit/cancel callback IDs must be joined to execution inventory |
| table.12.eligible | current source selector and ordered selection instance list → eligible | derived-bool | selector-specific min/max/ordering and predicate; local row references only; count/ordinal source int32 | always | U-S: selector owner, actual predicate, and commit/cancel callback IDs must be joined to execution inventory |
| table.13.fact_kind | environment-maintained legitimate-knowledge ledger → fact_kind | enum | fact-specific scalar tag and source exposure checkpoint; hidden underlying state is not copied into this public table | conditional | P-E: this is a simulator knowledge record, not asserted as an original-game member |
| table.13.subject_table | environment-maintained legitimate-knowledge ledger → subject_table | enum | fact-specific scalar tag and source exposure checkpoint; hidden underlying state is not copied into this public table | conditional | P-E: this is a simulator knowledge record, not asserted as an original-game member |
| table.13.subject_ref | environment-maintained legitimate-knowledge ledger → subject_ref | typed-ref | fact-specific scalar tag and source exposure checkpoint; hidden underlying state is not copied into this public table | conditional | P-E: this is a simulator knowledge record, not asserted as an original-game member |
| table.13.position | environment-maintained legitimate-knowledge ledger → position | adapter-value | fact-specific scalar tag and source exposure checkpoint; hidden underlying state is not copied into this public table | conditional | P-E: this is a simulator knowledge record, not asserted as an original-game member |
| table.13.value_kind | environment-maintained legitimate-knowledge ledger → value_kind | enum | fact-specific scalar tag and source exposure checkpoint; hidden underlying state is not copied into this public table | conditional | P-E: this is a simulator knowledge record, not asserted as an original-game member |
| table.13.value | environment-maintained legitimate-knowledge ledger → value | adapter-value | fact-specific scalar tag and source exposure checkpoint; hidden underlying state is not copied into this public table | unknown-mask-zero | P-E: this is a simulator knowledge record, not asserted as an original-game member |
| table.14.owner_table | owner-qualified source property code 1..45 → owner_table | enum | closed tagged scalar/reference union; exact domain is property-code × owner-key specific; unused branches reject | conditional | U-S: complete public property-code→content-owner/member mapping is required; §3.3 only defines the code vocabulary |
| table.14.owner_ref | owner-qualified source property code 1..45 → owner_ref | typed-ref | closed tagged scalar/reference union; exact domain is property-code × owner-key specific; unused branches reject | conditional | U-S: complete public property-code→content-owner/member mapping is required; §3.3 only defines the code vocabulary |
| table.14.property_code | owner-qualified source property code 1..45 → property_code | enum | closed tagged scalar/reference union; exact domain is property-code × owner-key specific; unused branches reject | conditional | U-S: complete public property-code→content-owner/member mapping is required; §3.3 only defines the code vocabulary |
| table.14.ordinal | owner-qualified source property code 1..45 → ordinal | adapter-value | closed tagged scalar/reference union; exact domain is property-code × owner-key specific; unused branches reject | conditional | U-S: complete public property-code→content-owner/member mapping is required; §3.3 only defines the code vocabulary |
| table.14.value_kind | owner-qualified source property code 1..45 → value_kind | enum | closed tagged scalar/reference union; exact domain is property-code × owner-key specific; unused branches reject | conditional | U-S: complete public property-code→content-owner/member mapping is required; §3.3 only defines the code vocabulary |
| table.14.value | owner-qualified source property code 1..45 → value | adapter-value | closed tagged scalar/reference union; exact domain is property-code × owner-key specific; unused branches reject | conditional | U-S: complete public property-code→content-owner/member mapping is required; §3.3 only defines the code vocabulary |
| table.14.ref_table | owner-qualified source property code 1..45 → ref_table | enum | closed tagged scalar/reference union; exact domain is property-code × owner-key specific; unused branches reject | conditional | U-S: complete public property-code→content-owner/member mapping is required; §3.3 only defines the code vocabulary |
| table.14.ref_row | owner-qualified source property code 1..45 → ref_row | typed-ref | closed tagged scalar/reference union; exact domain is property-code × owner-key specific; unused branches reject | conditional | U-S: complete public property-code→content-owner/member mapping is required; §3.3 only defines the code vocabulary |
| table.15.fact_kind | environment-maintained public-history ledger → fact_kind | enum | public-only source checkpoint history; int32 source values sign-extend; sequence ordinals are carrier ordinals | conditional | P-E: no private source RNG/history counters are projected |
| table.15.ordinal | environment-maintained public-history ledger → ordinal | derived-i64 | public-only source checkpoint history; int32 source values sign-extend; sequence ordinals are carrier ordinals | conditional | P-E: no private source RNG/history counters are projected |
| table.15.subject_kind | environment-maintained public-history ledger → subject_kind | enum | public-only source checkpoint history; int32 source values sign-extend; sequence ordinals are carrier ordinals | conditional | P-E: no private source RNG/history counters are projected |
| table.15.subject_code | environment-maintained public-history ledger → subject_code | source-key | public-only source checkpoint history; int32 source values sign-extend; sequence ordinals are carrier ordinals | conditional | P-E: no private source RNG/history counters are projected |
| table.15.value_kind | environment-maintained public-history ledger → value_kind | enum | public-only source checkpoint history; int32 source values sign-extend; sequence ordinals are carrier ordinals | conditional | P-E: no private source RNG/history counters are projected |
| table.15.value | environment-maintained public-history ledger → value | derived-i64 | public-only source checkpoint history; int32 source values sign-extend; sequence ordinals are carrier ordinals | conditional | P-E: no private source RNG/history counters are projected |
| table.15.visible_floor | environment-maintained public-history ledger → visible_floor | derived-i64 | public-only source checkpoint history; int32 source values sign-extend; sequence ordinals are carrier ordinals | conditional | P-E: no private source RNG/history counters are projected |
| table.16.action_kind | exact current source menu choice → typed action action_kind | enum | exact action tuple in §3.2 plus live menu membership; index/start/count are slot-local checked i64 spans | always | U-S: source action-class/callback membership and owner-qualified option IDs are execution/source-registry joins |
| table.16.source_table | exact current source menu choice → typed action source_table | enum | exact action tuple in §3.2 plus live menu membership; index/start/count are slot-local checked i64 spans | always | U-S: source action-class/callback membership and owner-qualified option IDs are execution/source-registry joins |
| table.16.source_ref | exact current source menu choice → typed action source_ref | typed-ref | exact action tuple in §3.2 plus live menu membership; index/start/count are slot-local checked i64 spans | always | U-S: source action-class/callback membership and owner-qualified option IDs are execution/source-registry joins |
| table.16.target_table | exact current source menu choice → typed action target_table | enum | exact action tuple in §3.2 plus live menu membership; index/start/count are slot-local checked i64 spans | always | U-S: source action-class/callback membership and owner-qualified option IDs are execution/source-registry joins |
| table.16.target_ref | exact current source menu choice → typed action target_ref | typed-ref | exact action tuple in §3.2 plus live menu membership; index/start/count are slot-local checked i64 spans | always | U-S: source action-class/callback membership and owner-qualified option IDs are execution/source-registry joins |
| table.16.option_code | exact current source menu choice → typed action option_code | source-key | exact action tuple in §3.2 plus live menu membership; index/start/count are slot-local checked i64 spans | always | U-S: source action-class/callback membership and owner-qualified option IDs are execution/source-registry joins |
| table.16.parameter_start | exact current source menu choice → typed action parameter_start | derived-i64 | exact action tuple in §3.2 plus live menu membership; index/start/count are slot-local checked i64 spans | always | U-S: source action-class/callback membership and owner-qualified option IDs are execution/source-registry joins |
| table.16.parameter_count | exact current source menu choice → typed action parameter_count | derived-i64 | exact action tuple in §3.2 plus live menu membership; index/start/count are slot-local checked i64 spans | always | U-S: source action-class/callback membership and owner-qualified option IDs are execution/source-registry joins |
| table.17.parameter_kind | action-owner parameter tuple in current source menu → parameter_kind | enum | option-qualified scalar/reference tagged union; ordinal/count are nonnegative checked spans | conditional | U-S: parameter-owner/domain restrictions must join each source action and exact current choice |
| table.17.ordinal | action-owner parameter tuple in current source menu → ordinal | derived-i64 | option-qualified scalar/reference tagged union; ordinal/count are nonnegative checked spans | conditional | U-S: parameter-owner/domain restrictions must join each source action and exact current choice |
| table.17.value_kind | action-owner parameter tuple in current source menu → value_kind | enum | option-qualified scalar/reference tagged union; ordinal/count are nonnegative checked spans | conditional | U-S: parameter-owner/domain restrictions must join each source action and exact current choice |
| table.17.value | action-owner parameter tuple in current source menu → value | adapter-value | option-qualified scalar/reference tagged union; ordinal/count are nonnegative checked spans | conditional | U-S: parameter-owner/domain restrictions must join each source action and exact current choice |
| table.17.ref_table | action-owner parameter tuple in current source menu → ref_table | enum | option-qualified scalar/reference tagged union; ordinal/count are nonnegative checked spans | conditional | U-S: parameter-owner/domain restrictions must join each source action and exact current choice |
| table.17.ref_row | action-owner parameter tuple in current source menu → ref_row | typed-ref | option-qualified scalar/reference tagged union; ordinal/count are nonnegative checked spans | conditional | U-S: parameter-owner/domain restrictions must join each source action and exact current choice |
| table.18.relation_kind | public ownership/selection/known-order relation registry → relation_kind | enum | closed relation tag plus exact endpoint-table allowlist; slot-local row refs | conditional | P-E/U-S: §3.3 names relation kinds but endpoint equality requires the owner-specific relation registry |
| table.18.from_table | public ownership/selection/known-order relation registry → from_table | enum | closed relation tag plus exact endpoint-table allowlist; slot-local row refs | conditional | P-E/U-S: §3.3 names relation kinds but endpoint equality requires the owner-specific relation registry |
| table.18.from_ref | public ownership/selection/known-order relation registry → from_ref | typed-ref | closed relation tag plus exact endpoint-table allowlist; slot-local row refs | conditional | P-E/U-S: §3.3 names relation kinds but endpoint equality requires the owner-specific relation registry |
| table.18.to_table | public ownership/selection/known-order relation registry → to_table | enum | closed relation tag plus exact endpoint-table allowlist; slot-local row refs | conditional | P-E/U-S: §3.3 names relation kinds but endpoint equality requires the owner-specific relation registry |
| table.18.to_ref | public ownership/selection/known-order relation registry → to_ref | typed-ref | closed relation tag plus exact endpoint-table allowlist; slot-local row refs | conditional | P-E/U-S: §3.3 names relation kinds but endpoint equality requires the owner-specific relation registry |

### 2.7 Numeric operation and overflow law

For every source member whose descriptor is Java `int` (`I`), legal source field input/output values are exactly signed 32-bit `[-2^31,2^31−1]`; the candidate `int64` value is the sign extension of that exact source value. Do not accept an out-of-range caller value and narrow it. Where the source method executes Java integer add/subtract/multiply, the source operation wraps modulo 2^32 at that operation boundary; this is **source behavior**, not an administrative transport failure and not a gameplay cap. Where source code clamps, it clamps only in that method. In particular, `AbstractCreature.addBlock` and the documented energy helper caps do not generalize to other resources. The environment does not emulate overflow as a legal action rejection, and transport/allocation exhaustion does not imitate source integer overflow.

The cost sentinels are separate: source card base `cost == -1` means X and `cost == -2` means unplayable in the reviewed card records; ordinary `0` remains a known ordinary cost, distinct from both sentinels. When the card identity/value is source-entitled, emit the signed-i32 base value (including sentinels) and its derived kind together; when not entitled, mask both so kind cannot leak a hidden instance cost. `costForTurn` is a distinct mutable signed-i32 field and may be changed by guarded setters; its value/mask is independent and `cost_kind` never classifies it. Preserve the source guard on negative setter inputs, including the rule that a negative argument does not rewrite an already-negative turn cost. A positive mutable combat cost can independently set `freeToPlayOnce`; do not infer it from a zero turn cost or erase the positive-cost history. A free-to-play-once flag, autoplay/ignore-total flags, X capture, printed cost, base cost, and turn cost are independent source fields/predicates. This candidate does not infer generic payment from a cost code. Every source action that reads these values still needs the exact source-owner tuple/callback join listed in §3.2/§8.

The packed observation contains no persistent source `float` cell in the 18 tables. Card damage/Block, potion percentage, event chance and price-generation float calculations are method-local binary32 operations: preserve their exact source operation sequence, rounding/narrowing point, and threshold comparison where those results feed an observed integer/boolean. Do not recast them as a persistent binary32 source member or as a binary64/exact-rational accumulator. The detached time attachment remains the separately tagged raw-binary32 case. Integer gold/card/relic/potion price fields are `int`, not raw float bits; a property code cannot relabel them as float.

Boolean source members have JVM descriptor `Z` and encode `0/1`. Enum/object references are not Java ordinals: code them through the explicit owner-qualified source-key map or fail closed. Java byte move IDs are source `byte` (`B`, signed `[-128,127]`) and are promoted only after owner-qualified intent decoding; they are not accepted as arbitrary i32 enum codes. Counts read from Java collection sizes/indices are source `int` ordinals, but those are container representation bounds, not game-content maxima. A count beyond source collection/host/allocation representation faults administratively; no legal menu is shortened.

Known direct field joins now include MapRoomNode `x/y/room/taken/hasEmeraldKey`, AbstractCard `cardID/cost/costForTurn/timesUpgraded/misc/baseDamage/baseBlock/baseMagicNumber/retain/selfRetain/exhaust/exhaustOnUseOnce/exhaustOnFire/isEthereal/freeToPlayOnce/isInAutoplay/ignoreEnergyOnUse`, AbstractCreature `currentHealth/maxHealth/currentBlock/gold`, AbstractRelic `relicId/counter/usedUp`, AbstractPotion `ID/slot/potency` plus `getPotency`, AbstractPower `ID/amount/owner`, AbstractOrb `ID/passiveAmount/evokeAmount`, AbstractMonster `intent/nextMove/intentDmg/intentMultiAmt`, EnergyManager `energy/energyMaster`, AbstractPlayer `chosenClass/potionSlots/maxOrbs/gameHandSize`, AbstractDungeon `actNum/floorNum/ascensionLevel`, and GameActionManager `turn`. The rows label field-specific method/predicate joins where the underlying cell is not a direct member.


## 3. Finite dictionaries, aliases, actions, and references

### 3.1 Dictionary identity and source aliases

**P-E.** Content dictionary namespaces are closed IDs `1..13`: `card`, `relic`, `potion`, `enemy`, `encounter`, `event`, `power`, `intent`, `orb`, `source_option`, `room_object`, `public_fact`, `effect_site`. Within a frozen namespace, each canonical semantic key is lower ASCII snake-case, stable and nonlocalized. Assign codes `1..N` after bytewise UTF-8 sort of the complete finite key set; `0` means unknown only in fields that expressly allow unknown, and `-1` means absent only in fields expressly permitting absence. No enum cast, runtime hash iteration, display string, address, catalog position or source alias becomes a code.

Each dictionary record is exactly `(namespace, canonical_key, source_identity_set, alias_set, evidence_grade, visibility, lifecycle_owner)`. Canonical key is unique within namespace; each source identity maps to exactly one key; each alias maps to exactly one key **within its declared owner/namespace**. Conflicting aliases fail catalog validation; never choose “first match.” Aliases are exact case-sensitive source-facing identifiers only for lookup/traceability and are not additional game kinds. This binding method is complete; the literal whole-game dictionary remains an input artifact that must be generated and equality-checked against all records before identity freeze.

Finite source-owned dictionary families are: every card instance kind in the paired Ironclad, Silent, Defect, Watcher and colorless/Status/Curse/SPECIAL records; every relic kind in the economy/relic records; all 33 eligible potion kinds; the 25 enemy kinds; event identities and their options in Neow/event records; encounters and formation/member roles in enemy records; power, intent, orb, source-option, room-object and public-fact keys present in those same records. The sources distinguish character-specific basics as distinct kinds, every Note-reachable off-color basic, generated Status/Curses and event-stored special cards. A color/rarity/title alias does not merge kinds. The closed set is not “all IDs returned by a runtime card lookup”: unsupported/unreachable identities are absent, not silently accepted.

Candidate alias rules (proposal): exact source enum/library spelling aliases only; aliases are normalized nowhere. For names that differ by spaces/punctuation, author the canonical snake-case key explicitly and record the original exact spellings as separate aliases. Duplicate display names across owner/namespace remain distinct by canonical key. Unknown external strings reject creation input; source fallback to Iron Wave for malformed stored ID is not adopted as native validation behavior.


#### Literal source-key inventory

The following rows are concrete source-identity proposals for the retained Ironclad/colorless/Status/Curse card ledger only. `source_id` is the exact `AbstractCard.cardID` value; the class alias is retained independently, and the semantic key is a proposed stable nonlocalized key. This is **not** a complete 13-namespace dictionary: Silent/Defect/Watcher, all relics, potions, enemies/encounters, events/options, powers/intents/orbs/room objects/public facts/effect sites still require their own literal identity joins. Exact alias collisions below are checked by namespace and byte-sensitive alias, not normalized display name.

| Namespace | Exact source ID | Canonical semantic key (proposal) | Additional exact alias | Evidence row |
|---|---|---|---|---|
| card | `Bash` | `bash` | `—` | R01 |
| card | `Defend_R` | `defend_red` | `Defend_Red` | R02 |
| card | `Strike_R` | `strike_red` | `Strike_Red` | R03 |
| card | `Anger` | `anger` | `—` | R04 |
| card | `Armaments` | `armaments` | `—` | R05 |
| card | `Body Slam` | `bodyslam` | `BodySlam` | R06 |
| card | `Clash` | `clash` | `—` | R07 |
| card | `Cleave` | `cleave` | `—` | R08 |
| card | `Clothesline` | `clothesline` | `—` | R09 |
| card | `Flex` | `flex` | `—` | R10 |
| card | `Havoc` | `havoc` | `—` | R11 |
| card | `Headbutt` | `headbutt` | `—` | R12 |
| card | `Heavy Blade` | `heavyblade` | `HeavyBlade` | R13 |
| card | `Perfected Strike` | `perfectedstrike` | `PerfectedStrike` | R14 |
| card | `Pommel Strike` | `pommelstrike` | `PommelStrike` | R15 |
| card | `Shrug It Off` | `shrugitoff` | `ShrugItOff` | R16 |
| card | `Sword Boomerang` | `swordboomerang` | `SwordBoomerang` | R17 |
| card | `Thunderclap` | `thunderclap` | `ThunderClap` | R18 |
| card | `True Grit` | `truegrit` | `TrueGrit` | R19 |
| card | `Twin Strike` | `twinstrike` | `TwinStrike` | R20 |
| card | `Warcry` | `warcry` | `—` | R21 |
| card | `Wild Strike` | `wildstrike` | `WildStrike` | R22 |
| card | `Iron Wave` | `ironwave` | `IronWave` | R24 |
| card | `Battle Trance` | `battletrance` | `BattleTrance` | R25 |
| card | `Blood for Blood` | `bloodforblood` | `BloodForBlood` | R26 |
| card | `Bloodletting` | `bloodletting` | `—` | R27 |
| card | `Burning Pact` | `burningpact` | `BurningPact` | R28 |
| card | `Carnage` | `carnage` | `—` | R29 |
| card | `Combust` | `combust` | `—` | R30 |
| card | `Dark Embrace` | `darkembrace` | `DarkEmbrace` | R31 |
| card | `Disarm` | `disarm` | `—` | R32 |
| card | `Dropkick` | `dropkick` | `—` | R33 |
| card | `Dual Wield` | `dualwield` | `DualWield` | R34 |
| card | `Entrench` | `entrench` | `—` | R35 |
| card | `Evolve` | `evolve` | `—` | R36 |
| card | `Feel No Pain` | `feelnopain` | `FeelNoPain` | R37 |
| card | `Fire Breathing` | `firebreathing` | `FireBreathing` | R38 |
| card | `Flame Barrier` | `flamebarrier` | `FlameBarrier` | R39 |
| card | `Ghostly Armor` | `ghostlyarmor` | `GhostlyArmor` | R40 |
| card | `Hemokinesis` | `hemokinesis` | `—` | R41 |
| card | `Infernal Blade` | `infernalblade` | `InfernalBlade` | R42 |
| card | `Inflame` | `inflame` | `—` | R43 |
| card | `Intimidate` | `intimidate` | `—` | R44 |
| card | `Metallicize` | `metallicize` | `—` | R45 |
| card | `Power Through` | `powerthrough` | `PowerThrough` | R46 |
| card | `Pummel` | `pummel` | `—` | R47 |
| card | `Rage` | `rage` | `—` | R48 |
| card | `Rampage` | `rampage` | `—` | R49 |
| card | `Reckless Charge` | `recklesscharge` | `RecklessCharge` | R50 |
| card | `Rupture` | `rupture` | `—` | R51 |
| card | `Searing Blow` | `searingblow` | `SearingBlow` | R52 |
| card | `Second Wind` | `secondwind` | `SecondWind` | R53 |
| card | `Seeing Red` | `seeingred` | `SeeingRed` | R54 |
| card | `Sentinel` | `sentinel` | `—` | R55 |
| card | `Sever Soul` | `seversoul` | `SeverSoul` | R56 |
| card | `Shockwave` | `shockwave` | `—` | R57 |
| card | `Spot Weakness` | `spotweakness` | `SpotWeakness` | R58 |
| card | `Uppercut` | `uppercut` | `—` | R59 |
| card | `Whirlwind` | `whirlwind` | `—` | R60 |
| card | `Barricade` | `barricade` | `—` | R61 |
| card | `Berserk` | `berserk` | `—` | R62 |
| card | `Bludgeon` | `bludgeon` | `—` | R63 |
| card | `Brutality` | `brutality` | `—` | R64 |
| card | `Corruption` | `corruption` | `—` | R65 |
| card | `Demon Form` | `demonform` | `DemonForm` | R66 |
| card | `Double Tap` | `doubletap` | `DoubleTap` | R67 |
| card | `Exhume` | `exhume` | `—` | R68 |
| card | `Feed` | `feed` | `—` | R69 |
| card | `Fiend Fire` | `fiendfire` | `FiendFire` | R70 |
| card | `Immolate` | `immolate` | `—` | R71 |
| card | `Impervious` | `impervious` | `—` | R72 |
| card | `Juggernaut` | `juggernaut` | `—` | R73 |
| card | `Limit Break` | `limitbreak` | `LimitBreak` | R74 |
| card | `Offering` | `offering` | `—` | R75 |
| card | `Reaper` | `reaper` | `—` | R76 |
| card | `Bandage Up` | `bandageup` | `BandageUp` | C01 |
| card | `Blind` | `blind` | `—` | C02 |
| card | `Dark Shackles` | `darkshackles` | `DarkShackles` | C03 |
| card | `Deep Breath` | `deepbreath` | `DeepBreath` | C04 |
| card | `Discovery` | `discovery` | `—` | C05 |
| card | `Dramatic Entrance` | `dramaticentrance` | `DramaticEntrance` | C06 |
| card | `Enlightenment` | `enlightenment` | `—` | C07 |
| card | `Finesse` | `finesse` | `—` | C08 |
| card | `Flash of Steel` | `flashofsteel` | `FlashOfSteel` | C09 |
| card | `Forethought` | `forethought` | `—` | C10 |
| card | `Good Instincts` | `goodinstincts` | `GoodInstincts` | C11 |
| card | `Impatience` | `impatience` | `—` | C12 |
| card | `Jack Of All Trades` | `jackofalltrades` | `JackOfAllTrades` | C13 |
| card | `Madness` | `madness` | `—` | C14 |
| card | `Mind Blast` | `mindblast` | `MindBlast` | C15 |
| card | `Panacea` | `panacea` | `—` | C16 |
| card | `PanicButton` | `panicbutton` | `—` | C17 |
| card | `Purity` | `purity` | `—` | C18 |
| card | `Swift Strike` | `swiftstrike` | `SwiftStrike` | C19 |
| card | `Trip` | `trip` | `—` | C20 |
| card | `Apotheosis` | `apotheosis` | `—` | C21 |
| card | `Chrysalis` | `chrysalis` | `—` | C22 |
| card | `HandOfGreed` | `handofgreed` | `—` | C23 |
| card | `Magnetism` | `magnetism` | `—` | C24 |
| card | `Master of Strategy` | `masterofstrategy` | `MasterOfStrategy` | C25 |
| card | `Mayhem` | `mayhem` | `—` | C26 |
| card | `Metamorphosis` | `metamorphosis` | `—` | C27 |
| card | `Panache` | `panache` | `—` | C28 |
| card | `Sadistic Nature` | `sadisticnature` | `SadisticNature` | C29 |
| card | `Secret Technique` | `secrettechnique` | `SecretTechnique` | C30 |
| card | `Secret Weapon` | `secretweapon` | `SecretWeapon` | C31 |
| card | `The Bomb` | `thebomb` | `TheBomb` | C32 |
| card | `Thinking Ahead` | `thinkingahead` | `ThinkingAhead` | C33 |
| card | `Transmutation` | `transmutation` | `—` | C34 |
| card | `Violence` | `violence` | `—` | C35 |
| card | `Burn` | `burn` | `—` | S01 |
| card | `Dazed` | `dazed` | `—` | S02 |
| card | `Slimed` | `slimed` | `—` | S03 |
| card | `Void` | `voidcard` | `VoidCard` | S04 |
| card | `Wound` | `wound` | `—` | S05 |
| card | `Ghostly` | `apparition` | `Apparition` | N01 |
| card | `Bite` | `bite` | `—` | N02 |
| card | `J.A.X.` | `jax` | `JAX` | N03 |
| card | `RitualDagger` | `ritualdagger` | `—` | N04 |
| card | `Shiv` | `shiv` | `—` | N05 |
| card | `Safety` | `safety` | `—` | N06 |
| card | `Smite` | `smite` | `—` | N07 |
| card | `Insight` | `insight` | `—` | N08 |
| card | `ThroughViolence` | `throughviolence` | `—` | N09 |
| card | `Miracle` | `miracle` | `—` | N10 |
| card | `Beta` | `beta` | `—` | N11 |
| card | `Omega` | `omega` | `—` | N12 |
| card | `Expunger` | `expunger` | `—` | N13 |
| card | `BecomeAlmighty` | `becomealmighty` | `—` | N14 |
| card | `LiveForever` | `liveforever` | `—` | N15 |
| card | `FameAndFortune` | `fameandfortune` | `—` | N16 |

#### Concrete owner-to-site slice (39 inventoried direct-card producers)

The retained generator analysis names these 39 direct-card producer owners. The stable source-facing identity for this slice is the owner label as printed in the linked normative records; the site suffix is the proposed `(operation, variant=0)` pair. This is an exact named slice, not a claim that the owner labels exhaust all legal content kinds or that numeric owner codes are frozen.

| Site operation | Owner labels and proposed canonical owner keys |
|---|---|
| `use/0` | Infernal Blade (`infernal_blade`), Distraction (`distraction`), White Noise (`white_noise`), Discovery (`discovery`), Chrysalis (`chrysalis`), Metamorphosis (`metamorphosis`), Foreign Influence (`foreign_influence`), Jack of All Trades (`jack_of_all_trades`), Transmutation (`transmutation`), Wild Strike (`wild_strike`), Reckless Charge (`reckless_charge`), Power Through (`power_through`), Immolate (`immolate`), Overclock (`overclock`), TURBO (`turbo`), Blade Dance (`blade_dance`), Cloak and Dagger (`cloak_and_dagger`), Storm of Steel (`storm_of_steel`), Carve Reality (`carve_reality`), Deceive Reality (`deceive_reality`), Evaluate (`evaluate`), Pray (`pray`), Reach Heaven (`reach_heaven`), Conjure Blade (`conjure_blade`), Anger (`anger`), Alchemize (`alchemize`) |
| `start-turn/0` | Creative AI (`creative_ai`), Hello World (`hello_world`), Magnetism (`magnetism`), Infinite Blades (`infinite_blades`), Battle Hymn (`battle_hymn`), Collect (`collect`), Alpha (`alpha`), Beta (`beta`) |
| `end-turn/0` | Study (`study`) |
| `actual-draw/0` | Deus Ex Machina (`deus_ex_machina`), Endless Agony (`endless_agony`) |
| `copy/0` | Dual Wield (`dual_wield`), Nightmare (`nightmare`) |

These labels bind to candidate semantic keys by the explicit parenthesized key, not capitalization-normalization at runtime. The source alias strings and exact source class/field locators must still be independently joined against the seven records; the key list is evidence-derived candidate content, not a newly inspected archive catalog. It also does not include the separate `insert/0` child sites, which are distinct effects. This slice therefore supplies named owner/site rows and exact operation distinctions while remaining **not a complete source-site equality proof**.

### 3.2 Action tuple dictionary

`action_kind` is the finite set below. A candidate row is a tagged sum; source/target ref and parameter list are constrained by the action, not by a generic “look up card” operation. The parameter order is part of action identity. Parameters use tags: `1 count`, `2 chosen-position`, `3 ordered-selection-member`, `4 captured/public-X`, `5 option-argument`, `6 public-price-component`, `7 source-public-flag`.

| Code | Action key | Exact tuple payload after common source/target | Legal shape |
|---:|---|---|---|
| 1 | `play_card` | source combat-card ref; optional target actor ref; `X` parameter only where this exact card/source exposes it | Candidate-specific legality and current payment/captured-X boundary; never infer X from printed cost alone. |
| 2 | `end_turn` | no target/parameters | Only when exact current core menu contains it. |
| 3 | `use_potion` | potion ref; optional target; option args only for that potion's own choice | Belt ownership and current phase checked by core. |
| 4 | `discard_potion` | potion ref | No gameplay effect beyond source-legal discard. |
| 5 | `traverse_map` | edge ref | Exact current reachable/selectable edge. |
| 6 | `choose_event_option` | event-option object ref; option key | Current event owner and live branch predicate. |
| 7 | `buy` | offer ref; price component only if source exposes a choice | Current price/payment and purchase eligibility, no stale row purchase. |
| 8 | `purge` | purge offer/source ref; selected card ref | Confirmation/cost is part of source continuation. |
| 9 | `choose_campfire` | campfire-option ref; option key | Initial filter and source-appended options preserve order. |
| 10 | `claim_reward` | reward ref; optional target ref | Generation/offer and actual claim are different checkpoints. |
| 11 | `skip_reward` | reward ref | Current skippable offer only; skip does not restore consumed source pool. |
| 12 | `take_boss_relic` | boss-relic offer ref | Exact current offered relic only. |
| 13 | `skip_boss_relic` | boss-relic offer ref | Exact current source skip branch. |
| 14 | `select_instance` | selection source ref; selected instance ref; position | Sequence position is meaningful; source-defined order retained. |
| 15 | `deselect_instance` | selection source ref; selected instance ref | Only a currently selected member. |
| 16 | `confirm_selection` | selection source ref; ordered count/member list | Count/range/mandatory/ordered rules from that selection. |
| 17 | `cancel_selection` | selection source ref | Only if source says cancellation exists; never a fabricated universal cancel. |
| 18 | `reveal_board` | board object ref; position | Only source-authorized reveal action. |
| 19 | `singing_bowl` | option code, count if source menu has that parameter | Uses actual option; not interchangeable with reward claim. |
| 20 | `proceed` | source room/object ref; exact continuation option if multiple | Dispatches the current continuation, not a universal return-to-map. |
| 21 | `choose_source_option` | owner-qualified source-option ref/key and exact typed parameters | No arbitrary callback or free-form payload. |

The public candidate table provides a canonical display ordering: action code, visible source order, visible target order, option code, then ordered parameters. This ordering does not change selected sequence semantics or hide private identity. The caller supplies only local candidate index plus opaque current token; it cannot edit action/parameter rows. Current schema's machine-range carrier `0..U-1` is not an action-space allocation or legal bound; valid action indices are exactly `[0,N)`.

### 3.3 Property, relation, and visibility dictionaries

Property codes `1..45` have the following closed meanings (the code numbers are stable only within this proposal): `1 base_damage`, `2 displayed_damage`, `3 base_block`, `4 displayed_block`, `5 magic`, `6 captured_x`, `7 cost_until_played`, `8 cost_combat`, `9 cost_free_once`, `10 autoplay`, `11 ignore_total`, `12 purge`, `13 public_source_site`, `14 public_copy_origin`, `15 retained_duration`, `16 delayed_duration`, `17 uses`, `18 gold_price`, `19 hp_price`, `20 max_hp_price`, `21 selected_order`, `22 can_skip`, `23 can_confirm`, `24 scry_count`, `25 public_discard_count`, `26 public_hp_loss_count`, `27 public_play_count`, `28 public_power_count`, `29 mantra`, `30 orb_slots`, `31 focus`, `32 kill_growth`, `33 potion_capacity`, `34 key_kind`, `35 burning_buff_public`, `36 source_option_stage`, `37 draw_order_known`, `38 outgoing_card_public`, `39 upgrades`, `40 source_visible_counter`, `41 reserved_unemittable_pending_actual_public_owner_proof`, `42 intent_visible`, `43 payment_status`, `44 transform_input_kind`, `45 public_turn_cost_modifier`. Code 40 permits only explicitly registered `(content_key, subkey)` pairs; it is not an escape hatch. Code 41 is not a generic binary32 escape hatch and emits no property row under this candidate. D04 time fields are not inserted as an unqualified property.

Knowledge codes `1..9`: `draw_count`, `discard_count`, `exhaust_count`, `known_next`, `known_bottom`, `scry_window`, `remembered_board`, `announced_boss`, `source_revealed`. History codes `1..7`: `visited_room`, `revealed_event`, `chosen_option`, `revealed_board`, `public_play`, `public_discard`, `public_counter`. Relation codes `1..6`: `reward_exclusive`, `reward_parent`, `public_copy_source`, `public_permanent_owner`, `public_selection_source`, `public_known_order`. Exact relation endpoints are typed and restricted to the consolidated schema's permitted table sets; invalid relation-kind/table combinations reject.

Table/column visibility registry assigns each semantic field one and only one disposition: `always-public`, `conditionally-public(predicate-key)`, `present-unknown(mask=0)`, `private-excluded`, or `not-applicable`. All content-defined dynamic property values must resolve to an actual fixed column, one of these 45 properties, a typed relation, or an explicitly nonpublic field. Missing disposition or duplicate mapping fails closed. Frozen Eye exposes current draw order only; without it, hidden order does not leak through ordering/refs. Runic Dome masks intent and derived damage. Face-down board position is visible while unknown value remains masked. This registry proposal is exact in shape, but the literal per-content field-to-property rows and all source alias/site reconciliation remain B10/NC11/NC12 closure obligations; counts are not proof of equality.

## 4. Site identity, policy and equality

### 4.1 Stable site identity and finite inventory equality

A source site key is `(namespace:u32, owner:u32, operation:u32, variant:u32)`. Namespaces: `1 card, 2 relic, 3 potion, 4 enemy, 5 event, 6 run-system, 7 acquisition, 8 power`. Operation codes: `1 use, 2 actual-draw, 3 actual-exhaust, 4 start-turn, 5 end-turn, 6 pre-draw, 7 equip, 8 reward-build, 9 claim, 10 purchase, 11 transform, 12 remove, 13 copy, 14 insert, 15 key-commit, 16 selection, 17 resume, 18 transition, 19 damage, 20 heal, 21 lifecycle, 22 pool-select, 23 initialize, 24 restock, 25 preview`. `variant` is a fixed local site ordinal, never an invocation, row, card instance, seed, or dynamic counter. This numeric tuple is not a final RNG ID and must not be regenerated from runtime enumeration.

Each site record must provide: key; exact owner/source operation; result kind (offer/fixed creation/copy/transform/acquisition/movement/effect); context basis; request bounds; eligible identity set or fixed result; ordered selection/retry law; read/write history and once-only checkpoint; payment/cost/copy/construction; RNG binding; destination/capacity; continuation parent/resume/cancel; terminal/fault boundary; visibility; source locators/grade; and readiness. Every member is Fact, Proposed, Partial with named open fields, or Open. No inherited row fact and no wildcard fallback.

Equality proof is set-based, not count-based. Build the independent expected set from every producer invocation/constructor/callback/claim/insert/transform/removal/acquisition and every source owner-operation call; build the registry's source-qualified records separately; compare both directions after alias resolution. Report `missing_from_registry`, `unmatched_registry_entry`, `duplicate_alias`, `duplicate_site_key`, `owner_mismatch`, `operation_mismatch`, and `unsupported_namespace` as distinct failures. For a direct card owner, creation, insertion and later acquisition are different sites when their effects differ. Nested producer changes owner to the actual inner source. A content count (including the retained 39 direct card owners, 141 relic primaries, 33 potions, 25 enemies/events) is a coverage checksum only, never an equality proof.

The candidate inventory names explicitly differentiated route families: 39 direct card producers; applicability-qualified insertions; potion card-production/use and claim routes; relic actual-exhaust/pre-draw/copy/transform/equip/lifecycle/reward-build/restock routes; source-specific repeat-power lifecycle; enemy fixed-status and encounter lifecycle; all event select/resume/transition plus Note/Match/FaceTrader/Dead Adventurer/Mushrooms child sites; seven acquisition groups; map, encounter, event, five relic pools, combat, campfire, chest and handoff system sites. Exact per-site records are not all established by those families: require the two-way equality above before a final registry. Public document inventories are incomplete as to literal full keys; this report does not label NC11 closed.

### 4.2 RNG binding is symbolic and source-law preserving

An `RngBinding` is an explicit tagged interface, not an invented domain assignment:

```
RngBinding = NoRandomDraw(evidence)
           | Bound { domain_key, canonical_scope_fields, draw_checkpoint,
                     primitive_and_bound, ordered_call_index,
                     input_measure_id, conditioning_state_id,
                     cursor_relation_id, correlated_site_ids,
                     history_transition_id, retry_consumption,
                     exhaustion = AdministrativeFault }
           | Unresolved { named_source_question, owner_lane, required_inputs }
```

Canonical scope fields may use semantic run/map/visit/combat/actor/site/invocation identities; they may never include worker, slot, allocation address, execution completion order, current policy token, or public row number. The actual semantic mapping, input measure, joint law, cursor relationship, and linked-site dependencies are accepted from the RNG/maps and execution evidence interfaces; if those lanes have not established them, the record remains `Unresolved` and the descriptor is not runtime-ready. A candidate role name does not imply an independent stream. Shared/conditional RNG can require a shared cursor, joint input state, or rejection of the proposed split. HMAC domain identifiers/compatibility pins remain unchanged; no final new domain registry or schema digest is asserted.

**Named incoming obligations to controller:** map/topology/bucket/marker source scope and initialized/history-conditioned measure; relic five-pool seed/cursor joint and catalog order; event/Neow/question-room/candidate draw relationships; combat generation/placement/AI/HP linked domains; canonical site-to-source-call mapping and exact action/callback checkpoint. These belong to sibling RNG/maps and execution lanes. This author neither polls siblings nor fills any answer by assumption.

## 5. Detached profile, Note, time, replay, and handoff

### 5.1 Profile and Note attachment

**P-E; selection remains H.** `ProfileCandidateV1` is a detached partial-run observation, never automatically an account write or a creation profile. Schema fields are: profile schema identity; complete captured `RunCreationProfile` (explicit prior Ironclad Neow eligibility, per-act boss-seen sets, stored Note card kind and valid upgrade count); Note committed descriptor `(card_key, upgrade_count, revision)`; exchange status `None | IncomingApplied | PendingTransition | Committed | AbandonedBeforeTransition`; optional exchange identity and detached staged snapshot; boss-discovery additions; current-run Neow eligibility evidence; outcome/status. A staged snapshot is not committed and cannot be used as a next-run profile without explicit caller conversion. No omitted raw-profile field defaults. `ExperiencedA0` may be a separately named/versioned preset only if the whole-spec human selects it; it is not inferred from “fully unlocked.”

Note's reported source checkpoints remain distinct: incoming obtain/hooks and deck insertion precede outgoing selection; removal callbacks precede retaining the selected live instance; the committed card ID/upgrades are sampled on a later qualified room transition. The native candidate contract does not alias a staged snapshot to the later commit. A genuine death before transition abandons the stage while retaining prior committed Note; a death after commit keeps it. Administrative fault preserves internally reached mutations and checkpoint/revision, quiesces the pool, and returns no normal success/final/profile result. Earlier detached outputs remain valid. This is candidate in-memory semantics, not crash-atomic preference parity or caller-side persistence.

Every delivered profile/final/replay attachment owns its bytes. Mutating a caller buffer, later step/reset/close, or same-step replacement cannot mutate previous outputs. `ProfileCandidate` does not leak hidden source IDs or creation history into policy-visible ordering. Replay stores resolved detached profile and profile contract identity, not a live object, local file, account reference, or output-derived implicit default.

### 5.2 Time contract is explicitly a variant

The source evidence distinguishes binary32 playtime from scheduler time: gameplay dungeon updates add frame delta unless source `stopClock`; Secret Portal filtering in Act 3 checks playtime at least 800 seconds; this is not Act1 content or guaranteed selection. Reaper has separate source timer/action dispatch dependencies; no natural heal-loss result or native frame distribution is established.

The profile/time attachment therefore contains tagged `TimePolicy` (policy identity + payload) and `TimeState` (exact state under that policy). Whole-spec options, none selected here:

| Tag | Required payload | Explicitly does not mean |
|---|---|---|
| `SourceBinary32Frames` | initial raw binary32 playtime; admitted frame-delta bit sequence; stop-clock state/checkpoints; recorded frame inputs | host CPU/wall time, fixed FPS, or parity without source scheduling proof |
| `CanonicalVirtual60Hz` | initial elapsed exact integer units; frame count; fixed delta identity; explicit stop-clock/checkpoint state | original frame distribution or source binary32 drift parity |
| `ExplicitElapsedAtSettledDecision` | checked elapsed integer input with accepted choice; policy-defined scheduler settlement; replayed accepted dwell | caller-controlled automatic frame count or skipping a live mechanic-bearing timer |
| `NoElapsedTimeRules` | explicit deviation identity and proof that no in-scope successor/content rule reads elapsed time | source parity or hiding the 800-second dependency |

For `ExplicitElapsedAtSettledDecision`, legality requires a named settled/quiescent predicate and no active mechanic timer awaiting source progress; a bare generic wait is not admitted. A proposed exact integer unit/time conversion can be carried as a candidate only after selecting that variant. Any post-dispatch integer overflow/allocation issue faults administratively, not as a legal time cap. Replay includes time policy identity and all accepted time/frame inputs. Invalid batch rows do not advance time/RNG/state. No host timing, permanent always-on, permanent zero, or Reaper-specific forced outcome is silently chosen.

### 5.3 Exact successor-ready handoff payload variants

`Act2ReadyV1` is a detached native successor payload with fields: actual character/rules identity; authoritative player HP/maxHP/gold; ordered permanent deck instances with kind/upgrades/live misc/provenance; relic instances/counters and order; potion slots/capacity; keys; survived one-time-event and relevant encounter/reward histories; committed Note descriptor/revision; elapsed-time policy/state; RNG/root/domain provenance and canonical cursor or replay references; per-act reset/room modifiers; source-qualified handoff checkpoint; and an `entry_boundary` tag. It excludes private account/save paths, hidden unoffered pools as policy features, arbitrary external run state, and mutable pointers.

The unresolved cutoff is represented as an exact sum, not a hidden default:

- `ResolvedAct1BeforeTransition`: all Act1 boss rewards/skip/claim, boss-relic take/skip and every once-only child/continuation have resolved; payload is before Act2 transition mutations.
- `ResolvedAct2EntryPreMap`: same player has passed required successor transition effects: act advance, per-act list/room-probability initialization, room potion modifier reset, normal A0 maxHP-sized heal request through source heal hooks, and entry into TheCity before map construction.
- `ResolvedAct2EntryWithMap`: previous effects plus initialized successor map and empty starting node, with map/root law supplied by the RNG/maps interface.

All choices must retain the exact source transition checkpoint, and +1 is emitted once only at the human-selected accepted boundary. A boundary requiring unresolved source continuation or RNG is tagged blocked, not approximated. The payload is detached; source pool fault after an internally reached checkpoint does not fabricate a delivered successful payload. No Act2 gameplay/Heart win is claimed. These three are alternatives for human selection; the document recommends no one as fact.

### 5.4 API, replay, reset, faults and immutable delivery attachments

Preserve operation-specific predicates: `step` requires live slot, current decision token, current candidate index and exact core menu membership; `reset` does not require a step token or live menu and may initialize an uninitialized slot, explicitly reset terminal slot, or abandon a live slot after all-row validation; `read` returns detached state for initialized live/terminal slots without gameplay/RNG/token advancement. Every invalid addressed row rejects the entire batch before state/RNG/reward/profile/time/identity/ordinal changes. After dispatch, runtime/encoding/allocation/binding/facade failure faults and quiesces the whole pool, retains already reached mutations and fault provenance, returns no success batch, never rolls back, retries, resets, truncates, fabricates defeat, or drops a row. This is administrative representation failure, not a game rule/cap.

Replay framing remains exactly the reviewed 2026-10-19 candidate's grammar and six structural goldens: `A1RP`, versioned header, eight schema pairs, composed descriptor hash, explicit source/rules revisions, seed/partition/stream/episode, ordered optional creation/profile/time/core attachments, length-framed accepted decision records, exact end status/optional diagnostic, strict canonical JSON and exact EOF. Public digest order remains descriptor hash; fixed context values then masks; player values then masks; tables 1–18 in order, each row count then row-major values then same-order masks. G01–G06 are byte/framing goldens only, not gameplay-replay fixtures. Existing replay framing, goldens, seed/HMAC/RNG/snapshot/combat-observation pins are not changed. The private-core envelope and generic primitive/framing proposals in §5.5 do not define a complete payload encoding: complete source-specific tag payload schemas remain unencoded, and authoritative hidden-state ownership/field equality plus replay-attachment integration remain unresolved. Full content/site descriptor bytes remain unresolved; no final hash/fingerprint is claimed until NC11/12 and the unresolved bindings close.

Successful reset/read/step/replay deliveries contain detached, schema-qualified attachment bytes and identity; same-step vector reset snapshots the retired episode's final observation/resources/outcome/identity before installing replacement. Final slots index returned-row order. No action is applied to replacement. Manual terminal slots persist until explicit reset. Replay is from-start only, checks canonical semantic choices/current menus, records accepted time input where selected, and reports first mismatch without repair or imported-state substitution.


### 5.5 Canonical private-core encoding candidate — bounded carrier, not whole-core codec

**P-E; not source-parity evidence and not an implementation approval.** The proposed carrier is a tagged binary record, never opaque JSON, object serialization, pointer dump, hash-map iteration, or display text. Only the outer envelope and generic primitive/framing productions below are exact P-E grammar. No tag payload schema is currently claimed as a fully deterministic, emission-ready canonical payload; the entire private core remains unencoded/unfreezable. The tag rows below retain semantic member inventories and partial typed-layout candidates, not complete payload schemas. Any field whose primitive width, identifier framing, reference scope, discriminator domain, or list element grammar is absent remains unencoded and therefore unfreezable; this includes the explicitly unencoded tags listed below. For the generic productions, integers are big-endian two’s-complement at the stated width; lengths are unsigned big-endian; strings are strict UTF-8 with no normalization and a `u32` byte length. Reject malformed UTF-8, duplicate singleton tags, duplicate map keys, unknown tags, invalid lengths, trailing bytes, out-of-range source-width values, or noncanonical ordering before accepting any decoded payload fragment.

Envelope bytes are ASCII `A1CORE` (6 bytes), exactly one `u16 format-version=1`, `u32 record count`, followed by records sorted by ascending `(u16 tag, u32 owner ordinal, u32 instance ordinal)`. Each record is `u16 tag`, `u32 owner ordinal`, `u32 instance ordinal`, `u64 payload length`, payload bytes. No padding; the payload length must consume exactly the record payload. There is no row-level or payload-level version field: a tag's payload grammar is fixed by the envelope format version. Owner/instance ordinals are canonical within the containing run/owner list, not RNG/site IDs; singleton records use ordinal zero. The closed tag registry consists exactly of `0x0001 PlayerState`, `0x0002 MapState`, `0x0003 CardInstance`, `0x0004 RelicInstance`, `0x0005 PotionInstance`, `0x0006 ActorState`, `0x0007 PowerState`, `0x0008 OrbState`, `0x0009 PendingContinuation`, `0x000A SourceRngState`, `0x000B HistoryState`, `0x000C RoomState`, `0x000D EconomyState`, `0x000E ProfileStage`, `0x000F TimeState`, `0x0010 HandoffCheckpoint`. All 16 numeric tag values are fixed above; Unknown tags reject and no extension is silently ignored.

Payload primitives are `i8`/`i32`/`i64` signed big-endian at the declared width; `u8` booleans are exactly 0 or 1; `u32`-length-prefixed strings are strict UTF-8 (no normalization); fixed identifiers are exactly their stated byte width. A byte string is `u32 byte_length` then exactly that many bytes. A list is `u32 item_count`, then for each item `u32 item_length` plus exactly that item's declared payload; count is administrative framing only, not a content cap. A local child reference is `u32 owner ordinal,u32 instance ordinal` within the same envelope and must resolve to exactly one record of its declared tag. Optional is `u8 present` (0 absent, 1 followed by exactly one value); other values reject. The closed `Value` union arm codes are `1 i32`, `2 i64`, `3 bool-u8`, `4 enum-u16`, `5 strict-UTF8`, `6 length-framed-bytes`, `7 same-envelope-child-ref`; each arm has exactly the width/grammar just stated and no trailing bytes. The closed `Owner` union codes are `1 Player` (no payload), `2 Actor` (one child ref to tag 6), `3 Room` (one child ref to tag 0x000C), `4 Offer` (one child ref to tag 0x000D); unknown arms reject. The closed zone codes are `1 hand,2 draw,3 discard,4 exhaust,5 in-play,6 other-public`; no source-private zone is mapped to these public codes. The profile status codes are `0 none,1 incoming-applied,2 pending-transition,3 committed,4 abandoned-before-transition`; `TimePolicy` codes are `1 SourceBinary32Frames,2 CanonicalVirtual60Hz,3 ExplicitElapsedAtSettledDecision,4 NoElapsedTimeRules`; `Act2Boundary` codes are `1 ResolvedAct1BeforeTransition,2 ResolvedAct2EntryPreMap,3 ResolvedAct2EntryWithMap`. No implicit zero/default is permitted outside an arm that explicitly defines it.

| Tag | Candidate payload members and encoding disposition |
|---|---|
| PlayerState | character source ID; HP/maxHP/gold/block `i32`; energy/max-energy `i32`; potion-slot count `u32`; three key booleans; stance source ID; orb capacity `i32`; ordered length-framed child-ref lists for deck/relics/potions. |
| MapState | current map source key; ordered node list. Node: `i32 x,y`, room source ID, `taken:u8`, node marker `hasEmeraldKey:u8`; ordered edge-index list of `u32`; optional current-node `u32` index. Node marker is the burning-elite designation, not acquired run-key state. Never include RNG-derived fields not owned by the map interface. |
| CardInstance | exact card source ID; UUID exactly 16 bytes; `i32 timesUpgraded,misc,cost,costForTurn,baseDamage,baseBlock,baseMagicNumber`; nine source flags in listed order as `u8`; zone `u8`; zone ordinal `u32`. |
| RelicInstance | exact `relicId`; `i32 counter`; `u8 usedUp`; owner-list ordinal `u32`. |
| PotionInstance | exact potion `ID`; `i32 potency,slot`; `u8 obtained,discarded`; source owner-list ordinal `u32`. |
| ActorState | exact actor ID; `i32 currentHealth,maxHealth,currentBlock`; owner-qualified intent source ID and signed `i8 nextMove`; `i32 intentDmg,intentMultiAmt`; four flags `dead,dying,escaped,halfDead` as `u8`; formation ordinal `u32`. Hidden intent remains private and is not copied to public observation absent its visibility predicate. |
| PowerState | exact power ID; closed `Owner` union; `i32 amount,priority`; exact power-type source ID, not enum ordinal. |
| OrbState | exact orb ID; `i32 passiveAmount,evokeAmount,basePassiveAmount,baseEvokeAmount`; orb-list ordinal `u32`. No invented common counter. |
| PendingContinuation | **Unencoded/unfreezable pending source schema.** The carrier record can be framed, but source-owned continuation kind/stage/payload/ordered action descriptor registry is unresolved; no opaque payload, arbitrary callback, or guessed continuation discriminant is permitted. |
| SourceRngState | **Unencoded/unfreezable source payload schema.** No RNG owner, algorithm, state width, cursor, or canonical ordering is selected here; no arbitrary byte bag is called deterministic canonical state. |
| HistoryState | **Unencoded/unfreezable source payload schema** pending exact source owner, entry tags, widths and public/private checkpoint equality. |
| RoomState | **Unencoded/unfreezable source payload schema** pending exact room owner/object/option variant registry and field equality. |
| EconomyState | **Unencoded/unfreezable source payload schema** pending exact pool/price/chance owner-member and cursor/source-order joins. |
| ProfileStage | **Unencoded/unfreezable payload schema.** Retain the semantic candidate members for future human review: profile schema identity; committed Note `(card-key,upgrade-count,revision)`; exchange status/identity; optional detached staged descriptor; ordered boss-discovery additions; current-run Neow eligibility evidence; outcome/status. Their field wire types, widths, identifier grammars, optional/list-element layouts, and complete status/outcome domains are not specified here. No omitted profile field defaults. |
| TimeState | **Unencoded/unfreezable payload schema.** Retain the §5.2 `TimePolicy` and variant meanings as proposals for future human review only. The policy choice is not selected; checkpoint-list element grammar and fixed-delta, settled-decision, and deviation identity wire types/widths remain unspecified. No variant payload is currently emission-ready. |
| HandoffCheckpoint | **Unencoded/unfreezable payload schema.** Retain the §5.3 Act2 boundary alternatives as semantic proposals for future human review only. Source transition checkpoint ID type/width/reference scope and detached successor payload schema are not specified; RNG/map inputs and boundary choice remain open. No variant payload is currently emission-ready. |

The envelope and generic primitive/framing productions are exact P-E fragments only; generic framing does not supply omitted tag/member types. No tag payload schema is currently claimed as a fully deterministic, emission-ready canonical payload. `PendingContinuation`, `SourceRngState`, `HistoryState`, `RoomState`, `EconomyState`, `ProfileStage`, `TimeState`, and `HandoffCheckpoint` are explicitly unencoded/unfreezable payload schemas. The remaining tag rows are candidate layouts, not complete frozen payload schemas; any unstated field width, identifier encoding, reference scope, discriminator domain, or list item grammar remains unencoded at that field. Semantic profile, time and handoff variants remain proposals for future human review, with no choice or default selected. The full private core remains unencoded/unfreezable; full authoritative source-state equality and human choices remain open. This does not invent RNG bindings, continuation ownership, or product callbacks. Administrative list/length limits reject malformed/unrepresentable encoding and never cap legal game content.


## 6. Closure matrix, failures-first checks, and residuals

| Obligation | Before | New contract delta | Status after this document |
|---|---|---|---|
| NC11 semantic dictionary | Namespace/count and enumeration idea, not exact alias collision/equality rule | Closed 13 namespace IDs; 131 exact card ID/class aliases for the retained Ironclad/colorless/Status/Curse ledger independently joined; canonical key/alias collision checks and two-way equality failure classes | **Partial.** Other character, relic, potion, enemy/encounter, event/option, power/intent/orb/room/public-fact IDs and every source alias/site still require independent inventory equality. |
| NC12 public fields/private core | 18 widths and 45 property codes, but no per-field domain/visibility association | All 146 fixed/table cells have a source member or named derived producer, primitive/domain candidate, visibility and evidence/open-join label; private-core envelope and generic framing productions are explicit; eight tag payload schemas are explicitly unencoded/unfreezable, with other tag rows retained only as partial layout candidates | **Partial.** Source-member inventory vs schema/property/exclusion equality, owner-specific numeric transition/visibility predicates, the 45 property-owner value map and complete hidden-state equality remain open. The full private core is unencoded/unfreezable; no tag payload is claimed as a complete deterministic codec. This is not source closure. |
| Action tuples | 21 action labels but generic field interpretation incomplete | Every code has exact source/target/parameter shape and validation ownership | **Candidate complete**, pending exact current core-choice membership/callback binding from execution lane and cross-document review. |
| Effect sites | Key syntax/families, no full public/private exact identity binding | Stable tuple, required site record, finite inventory families and bidirectional equality test | **Partial.** Literal site set and site-to-core/private code equality require exact reconciliation. The 39 direct-card owners remain a named slice, not completeness. |
| NC04 RNG | Candidate names might be misread as independent stream selection | `RngBinding` tagged symbolic interface requires measure, conditioning, cursor, correlated sites and exact checkpoint; unresolved fails closed | **Open**, correctly delegated; no factual answer invented. |
| NC10 Note/profile | Detached candidate idea and source lifecycle findings | Exact attachment members/statuses, commit/revision/staging/death/fault/reset/delivery behavior and no implicit inheritance | **Candidate complete**, but profile catalog, diagnostics transport, human selection and independent review remain. |
| NC10 time | Candidate clock law exists, not approval | Four exact policy tags with payload contracts and gating; no host/always-on/zero default | **Human choice open**; settlement/timer registry is execution interface. |
| NC10 successor | successor-ready payload option not exact | Three exact entry-boundary variant payload/cutoff contracts, map variant requires map interface | **Human choice/source interface open**; no Act2 map law assumed. |
| Schema identity | Candidate identities/goldens but requested no final hash absent full binding | Preserved framing and compatibility pins; explicitly withheld final schema/hash | **Not frozen** until every literal dictionary/site/field/action/core binding and human choice is reconciled. |

### Failure-first schema and property checks

These are exact future contract tests, **not executed here**. The private lane's stdlib checks exercise the same predicates against this document with negative controls.

1. Replace one required table column, reorder any two columns, change a width, remove a table, or alter one `value_kind`: reject descriptor, rather than accept a different layout under an old identity.
2. Put a global-but-not-slot-local reference in a row, negative/overflow span, duplicate property key, nonzero unused reference cell, mask value 2, unknown enum, or upper bits in binary32 carrier: reject before access/packing.
3. Delete one source alias, give one alias two canonical keys, duplicate one site key, remove a producer-only site, or add a registry-only site: fail the named bidirectional NC11 comparison; equal counts do not pass.
4. Replace a `Bound` RNG interface with independent source/site streams while keeping the same marginal offer law but changing cursor/joint relation: fail on `input_measure_id`, `cursor_relation_id`, `correlated_site_ids`, or call checkpoint mismatch.
5. Mutate a detached staged Note result after export; step/reset/close; or inject failure before/after Note transition commit: prior exported bytes unchanged, staged != committed, revision increments once, no normal result after fault.
6. Reset an uninitialized/terminal/live slot without step token (valid), then give the final batch row malformed profile/time/seed/ordinal (invalid): whole request rejects unchanged before entropy/identity/abandonment/time. A token requirement on reset is a negative-control failure.
7. Try Act2Ready payload on all three entry tags: the first excludes transition fields, second excludes generated map, third includes map only if the named RNG interface is resolved. No variant leaks unoffered hidden pool into public observation.
8. Flip `TimePolicy` tag or alter accepted elapsed input without replay update: first semantic/time mismatch is reported; invalid batch does not advance time; representation overflow faults administratively, not as an episode terminal.
9. Re-run unchanged six replay framing goldens and reject truncated, trailing, unknown tag/version, malformed optional attachment and noncanonical UTF-8/JSON bytes. Golden success does not certify a complete site/schema hash.
10. Change any old compatibility pin (`spire-lab-rng-hmac-sha256-v1`, snapshot2, `spire-lab-combat-reference-v2`, `spire-lab-env-seed-sha256-v1`, `spire-lab-combat-observation-v1`, fingerprint `8fd66d212259d2bdd27139988a772e1a6c223f8d65524f38c74174a0d0843357`, or `hmac="=0.12.1"`): fail compatibility gate; this work changes none.

## 7. Source-to-contract ledger and future acceptance

| Claim | Evidence tier used | Exact limit / remaining interface |
|---|---|---|
| Table widths, column order, action codes, 45 property codes, replay grammar and six goldens | Retained reviewed contract proposal | Encoding specification only; not source-domain completeness, executable schema, or approval. |
| Per-card paired base/upgraded numeric and instance records | Retained normative records for Ironclad/colorless, Silent, Defect, Watcher | Do not infer transitive callbacks or universal numeric maxima; each record's explicit exceptions/open fields continue to govern. |
| Enemy, Neow/event, economy/reward/chest/potion numeric and branch records | Retained normative records | Local facts, not joint initialized RNG laws, all legal history witnesses, or complete callback closure. |
| Energy/Block 999 helper cap | Retained source-grounded record | Applies at named helpers only; no universal field/input cap. |
| Profile/Note/time facts | Retained profile-time source inspection/report | Fresh author reading is not re-proving source; original runtime remains unexecuted. Human profile/time variants remain unselected. |
| Effect-site equality and full private-core equality | No full equality evidence supplied; §5.5 is a candidate encoding only | Exact source site equality and hidden-state field/owner equality required before final schema hash; RNG/maps and execution own source bindings. |

Required future acceptance rows, all **UNEXECUTED gameplay cases**: all18 widths/typed references/empty shapes and large legal growing sequences; every property/content alias positive and collision negative; both-way source-owner/site and action-to-core membership equality; every source integer/binary32/boolean/count field's domain vs carrier boundary; helper-cap non-clipping controls; detached Note before/after commit and death/fault/reset/same-step output isolation; each profile/time/handoff variant round-trip from-start replay; invalid final-row reset/step atomicity; whole-pool quiescence/fault without rollback; all six framing goldens plus malformed byte controls; exact compatibility regressions; exact map/relic/event/combat RNG source-scope/cursor joints after their owning lane resolves them.

These are acceptance obligations, not claims they were run. Current research tests are document/schema guard checks only. They do not discharge future implementation acceptance or human approval.

## 8. Explicit controller handoff

Main must reconcile this candidate with the independent RNG/maps and execution documents and the complete normative source catalog. Required named inputs: (1) RNG/maps: source scope, root/history measure, ordered catalog construction, joint/linked cursor bindings; (2) execution: actual dispatch/site call equality, exact core choice tags, continuation/checkpoint IDs, time settlement/timer predicates, and successor transition children; (3) content integration: literal finite canonical IDs and source aliases, per-field source numeric/visibility map and transitive site equality; (4) human: profile/history preset, Note/fault output variant, time policy, Act2-ready boundary and map payload choice. No sibling answer is assumed by this document.

The final schema hash/fingerprint is intentionally withheld. This proposal does not close B09 numeric-source mapping, B10 semantic/site/private-core bindings, B11 human profile choice, B12 time settlement, B14 whole-spec human approval, or any implementation/plan/start gate. It is a concrete candidate contract and a precise integration checklist—not “complete by manifest counts.”
