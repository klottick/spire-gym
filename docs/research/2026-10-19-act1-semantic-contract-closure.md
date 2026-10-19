# Act 1 native semantic and numeric contract closure

**Status: standalone candidate contract for review; not approved behavior, final schema, implementation authorization, or proof that every source mechanic is closed.**

This document closes a specific native-contract gap: it defines a checkable field-level wire contract, semantic-key assignment rule, complete action tuple shape, effect-site identity/binding record, and detached profile/time/handoff attachments. It deliberately does not claim an immutable final dictionary/hash while required content/site reconciliation and human choices remain open. It supplements the reviewed Act 1 specification; it does not widen any combat-v1 identity.

## 1. Evidence and decision labels

- **Source fact (F)** means a fact reported by the reviewed static/source records. The seven normative records linked below contain paired content or numeric findings; they are not runtime tests and do not prove every transitive composition.
- **Encoding proposal (P-E)** is an exact candidate representation/domain. It is not a claim that the game uses that representation or that the whole specification approved it.
- **Human option (H)** is a tagged alternative whose selection remains open. No option below silently supplies a default.
- **Unresolved source interface (U-S)** means the native contract accepts a symbolic named input from the RNG/maps or execution work; this document does not invent that answer.

No original game class, initializer, or runtime was executed. The owned archive identity was retained from the authorized source-identity manifest; this lane performed no new archive extraction/decompilation because the needed operation-specific replay framing, packed widths, source-site families, and profile/time trace already exist in reviewed public records. Their assertions are attributed to those records, not upgraded to fresh source verification here.

Reviewed normative interfaces (all standalone public records): [enemy numeric and callback records](2026-10-19-act1-content-enemies.md), [Ironclad/colorless/Status/Curse and shared card rules](2026-10-19-act1-content-red-colorless.md), [Silent paired records](2026-10-19-act1-content-silent.md), [Neow/events](2026-10-19-act1-content-event-neow.md), [Watcher paired records](2026-10-19-act1-content-watcher.md), [economy/rewards/chests/potions](2026-10-19-act1-content-economy-potions.md), and [Defect paired records](2026-10-19-act1-content-defect.md). These link to primary-record scope; they do not imply transitive closure or an approved unified content catalog.

## 2. Exact field transport contract

### 2.1 Common carriers and validity

**P-E.** Every slot-local packed table is a variable-length sequence of rows with the fixed width below. Native numeric payload is signed `int64`; its same-shaped knowledge mask is `int8` restricted to `0` or `1`. Offsets are signed `int64[B+1]`, start at zero, nondecreasing, end at row count, and count rows (not scalar cells). For slot `s`, every row reference is local to `[offset[s], offset[s+1])`; `-1` is the only absent-row sentinel where a field permits it. Validate type, dimensions, mask, enums, refs, spans and all arithmetic before indexing or allocating. Unknown cells contain numeric zero with mask zero; known zero is mask one. A permitted absent sentinel is known. No padding/fake absent row, dense action carrier, guessed row/content maximum, paging, or truncation.

The public descriptor has fixed `context[12]` and `player[16]` rows plus these 18 tables. The order and exact column names are normative for this candidate:

| ID | Table (width) | Ordered columns; scalar meaning and candidate domain |
|---:|---|---|
| 1 | `nodes` (7) | `x,y`: signed source coordinates; `room_kind`: `RoomKind`; `visited,reachable`: boolean; `burning_marker`: boolean or known absent `-1` only if descriptor permits; `boss_code`: room-qualified `RoomObjectKind`/boss key, else `-1`. Coordinates use source int domain (U-S B02/B09); no coordinate clipping. |
| 2 | `edges` (3) | `from_node_ref,to_node_ref`: required local refs to table 1; `currently_selectable`: boolean from current legal traversal predicate. |
| 3 | `permanent_cards` (9) | `card_code`: public card-kind key; `upgrades`: card-kind-specific nonnegative upgrade domain; `misc`: card-kind-specific signed integer domain (often nonpublic, therefore masked unless public); `bottle_kind`: `BottleKind`; `removable,upgradeable`: booleans; `base_cost_kind`: `CostKind`; `base_cost`: source cost domain only when that cost kind carries a numeric base; `public_position`: canonical visible ordinal. Never coerce instance history to a kind-wide counter. |
| 4 | `combat_cards` (12) | `card_code`; `upgrades`; `misc`; `zone`: `CardZone`; `position`: zone-local visible ordinal; `cost_kind`; `base_cost`; `turn_cost`: current displayed/source turn-cost domain, independently sourced; `public_owner_ref`: local owner ref to player sentinel or permitted table; `retained,exhaust_on_use,ethereal`: booleans. `base_cost` and `turn_cost` are distinct fields. |
| 5 | `relics` (4) | `relic_code`; `public_position`; `counter`: relic-specific signed source-int domain; `used`: boolean. A relic's constructor default/counter lifetime does not define a universal bound. |
| 6 | `potions` (4) | `belt_slot`: actual slot index; `occupied`: boolean; `potion_code`: kind key or known `-1` for empty slot; `potency`: potion-specific signed source-int domain, masked when not public/applicable. Empty slots are real slots, not max-capacity padding. |
| 7 | `actors` (8) | `enemy_code`; `formation_position`; `hp,max_hp,block`: source-int domains with exact transition rules, not universal clipping; `intent_code`: owner-qualified intent key; `intent_damage`: source payload integer; `intent_hits`: source payload count. Hidden/suppressed intent and all derived payload fields are masked, not reconstructed. |
| 8 | `powers` (5) | `owner_table,owner_ref`: typed owner pair; `power_code`; `amount`: power-specific signed source-int domain; `public_position`: canonical order. Per-instance duration/amount is not a generic saturating counter. |
| 9 | `orbs` (5) | `position`; `orb_code`; `passive_amount,evoke_amount,counter`: orb-specific signed source-int domains. Orb slots are ordered; absence is not a fabricated row. |
| 10 | `offers` (9) | `offer_kind`; `content_code`; `target_table,target_ref`: typed optional target; `amount`: offer-specific amount domain; `price_kind`; `price_amount`: exact source price representation (binary32 values use raw bits, see §2.3); `status`: `OfferStatus`; `public_position`. Offered, claimed, skipped and acquired are different lifecycle states. |
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
| Signed integer / count | For a field whose retained source record identifies a Java `int`, the candidate scalar domain is exactly `[-2^31, 2^31-1]` and its `int64` cell is the exact sign-extension of that 32-bit value; a wider input is an encoding error, never wrap/clamp. For fields not evidenced as Java `int`, `int64` is carrier-only until their source type is established. The paired records specify actual card/enemy/economy values, costs, counters and selected helper rules, but not every field's primitive source type or narrower legal domain. Each literal field still needs `(source identity/field, source method boundary, primitive type, legal domain/cap if any, rounding, overflow consequence)` before final freeze. Unknown fields remain `U-S`, not silently signed64 gameplay. |
| `binary32` | Preserve raw IEEE-754 binary32 bits when a source field/price/quota is a float; do not round-trip through decimal or binary64. Apply the source operation's rounding at its exact boundary. The normative records identify float-based costs/rounding where applicable; this proposal does not claim all float members or all input values have been audited. |
| boolean | Encode exactly `0` or `1`; reject other values. Source's boolean truth is not encoded as numeric truthiness. |
| counts and widths | Table row counts and offsets use `0..U` only as encoding limits. Candidate parameter count and selection count are additionally constrained by the exact currently offered source menu, not a constant cap. |
| mutable counters | No global clamp/saturating rule. Per-kind source counter transition and initial value must be specified; an observed caller/helper cap applies only at that helper. |
| helper-specific cap | The source `addEnergy`/`addBlock` helper caps at 999 at those call sites. It does not clip external inputs or establish 999 as a general cap for incoming Block, enemy Block, relic counters, gold, HP, energy grants outside that helper, or any packed value. |
| overflow / nonfinite | Source arithmetic behavior is a source-specific rule and may not be replaced by carrier wrapping. If a legal source value cannot fit or be represented exactly by selected encoding, raise an administrative numeric/encoding fault before presenting a misleading observation. Do not reject/truncate a legal game row as a gameplay rule. |

The schema is not numeric-domain-complete for every live mutable source field until the source-field map is independently reconciled against the full normative records and source-method evidence. That is a specific B09 residual, not a blanket “all fields fit int64” claim.

### 2.3 Value kinds, references, and binary32

`value_kind` is a closed tagged union: `1 signed-i64`, `2 boolean`, `3 enum-code`, `4 binary32-raw-u32-in-i64` (upper 32 bits zero), `5 local-row-ref`. A row reference uses `(ref_table,ref_row)` with known integer local row; scalar is zero for kind 5. Other kinds require both reference columns to be absent sentinel. Knowledge/history `value_kind` is restricted to the explicitly enumerated scalar subset and exact fact-specific dictionary. Reject unknown tags, nonzero unused bits, malformed refs, overflow, noncanonical duplicate properties, or reference to another slot even when globally in range.

Binary32 validation treats bits as an encoding, not a float permission: the field's source contract specifies whether NaN/infinity/signed-zero are legal; until established, these values are not admitted as ordinary numeric content. A public price with source binary32 semantics is carried by exact 32-bit pattern in signed64 (upper bits zero); the optional property descriptor identifies that same carrier, never decimal display text.

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
| 10 | `offers` | `offer_kind:enum(offer_kind)`; `content_code:kind-qualified-key(content_kind)`; `target_table:enum(target-table)`; `target_ref:opt-ref(target_table)`; `amount:conditional-src-int(offer-qualified-amount)`; `price_kind:enum(price_kind)`; `price_amount:conditional-src-int-or-binary32-bits(price-kind)`; `status:enum(offer_status)`; `public_position:public-ordinal` |
| 11 | `room_objects` | `object_kind:enum(room_object_kind)`; `public_position:public-ordinal`; `face_state:enum(face_state)`; `content_kind:enum(content_kind)`; `content_code:kind-qualified-key(content_kind)`; `object_state:enum(object_state)`; `remaining_attempts:conditional-src-count(object-qualified-attempts)` |
| 12 | `selection` | `source_table:enum(selection-source-table)`; `source_ref:typed-ref(source_table)`; `selected_ordinal:src-count-or-absent`; `selected,eligible:boolean` |
| 13 | `knowledge` | `fact_kind:enum(knowledge_fact)`; `subject_table:enum(knowledge-subject-table)`; `subject_ref:opt-ref(subject_table)`; `position:src-count(public-sequence-position)`; `value_kind:enum(value_kind[1..4])`; `value:fact-qualified-scalar` |
| 14 | `properties` | `owner_table:enum(property-owner-table)`; `owner_ref:typed-ref(owner_table)`; `property_code:enum(property_code[1..45])`; `ordinal:src-count(property-multiplicity-index)`; `value_kind:enum(value_kind[1..5])`; `value:property-qualified-scalar-or-zero-for-ref`; `ref_table:enum(property-ref-table)-or-absent`; `ref_row:ref-row-or-absent` |
| 15 | `history` | `fact_kind:enum(history_fact[1..7])`; `ordinal:src-count(public-history-position)`; `subject_kind:enum(history-subject-kind)`; `subject_code:kind-qualified-key(subject_kind)`; `value_kind:enum(value_kind[1..4])`; `value:history-fact-qualified-scalar`; `visible_floor:src-count(public-floor)` |
| 16 | `candidates` | `action_kind:enum(action_kind[1..21])`; `source_table:enum(action-source-table)`; `source_ref:typed-ref(source_table)`; `target_table:enum(action-target-table)-or-absent`; `target_ref:opt-ref(target_table)`; `option_code:owner-qualified-key(source-option)-or-none`; `parameter_start:slot-local-span-start(table17)`; `parameter_count:slot-local-span-count(table17)` |
| 17 | `candidate_parameters` | `parameter_kind:enum(parameter_kind[1..7])`; `ordinal:src-count(parameter-order)`; `value_kind:enum(value_kind[1..5])`; `value:action-option-qualified-scalar-or-zero-for-ref`; `ref_table:enum(parameter-ref-table)-or-absent`; `ref_row:ref-row-or-absent` |
| 18 | `relations` | `relation_kind:enum(relation_kind[1..6])`; `from_table:enum(relation-specific-from-table)`; `from_ref:typed-ref(from_table)`; `to_table:enum(relation-specific-to-table)`; `to_ref:typed-ref(to_table)` |

The validator tags above close carrier interpretation and point to each field's semantic domain family. They intentionally do **not** claim guessed source maxima. The residual is now field-qualified: exact source primitive, signedness, legal minimum/maximum, special sentinel set, float operation/rounding, overflow behavior, and source locator remain to be attached per source field for (a) map coordinates; (b) each card's upgrade/misc/base-cost/turn-cost fields; (c) each relic counter and potion potency; (d) actor HP/maxHP/Block and each intent payload; (e) each power/orb amount; (f) each offer amount/price and room-object attempt counter; (g) selected/knowledge/property/history/candidate parameter counts and tagged payloads. A source helper cap remains attached only to its call boundary. A later source-field map must have a row per live source field and explicit source-field → packed-field/property/excluded-field mapping; this signature is not offered as that missing row inventory.

### 2.5 Fixed codebooks and typed-reference closure

These are closed candidate wire enums, not Java enum ordinals. Codes are literal and version-local:

| Codebook | Candidate numeric mapping |
|---|---|
| `room_kind` | `0 unknown, 1 monster, 2 elite, 3 boss, 4 event, 5 shop, 6 rest, 7 treasure, 8 boss_treasure` |
| `bottle_kind` | `0 none, 1 attack, 2 skill, 3 power` |
| `cost_kind` | `1 ordinary, 2 X, 3 unplayable`; numeric cost cells are masked unless the kind is ordinary |
| `combat_zone` | `1 hand, 2 draw, 3 discard, 4 exhaust, 5 in_play, 6 other_public`; hidden draw order is not an ordering feature |
| `offer_kind` | `1 card, 2 relic, 3 potion, 4 gold, 5 key, 6 hp, 7 max_hp, 8 removal, 9 upgrade, 10 transform, 11 source_option` |
| `price_kind` | `0 none, 1 gold, 2 hp_loss, 3 damage, 4 max_hp, 5 source_composite`; component properties carry composite prices |
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

## 3. Finite dictionaries, aliases, actions, and references

### 3.1 Dictionary identity and source aliases

**P-E.** Content dictionary namespaces are closed IDs `1..13`: `card`, `relic`, `potion`, `enemy`, `encounter`, `event`, `power`, `intent`, `orb`, `source_option`, `room_object`, `public_fact`, `effect_site`. Within a frozen namespace, each canonical semantic key is lower ASCII snake-case, stable and nonlocalized. Assign codes `1..N` after bytewise UTF-8 sort of the complete finite key set; `0` means unknown only in fields that expressly allow unknown, and `-1` means absent only in fields expressly permitting absence. No enum cast, runtime hash iteration, display string, address, catalog position or source alias becomes a code.

Each dictionary record is exactly `(namespace, canonical_key, source_identity_set, alias_set, evidence_grade, visibility, lifecycle_owner)`. Canonical key is unique within namespace; each source identity maps to exactly one key; each alias maps to exactly one key **within its declared owner/namespace**. Conflicting aliases fail catalog validation; never choose “first match.” Aliases are exact case-sensitive source-facing identifiers only for lookup/traceability and are not additional game kinds. This binding method is complete; the literal whole-game dictionary remains an input artifact that must be generated and equality-checked against all records before identity freeze.

Finite source-owned dictionary families are: every card instance kind in the paired Ironclad, Silent, Defect, Watcher and colorless/Status/Curse/SPECIAL records; every relic kind in the economy/relic records; all 33 eligible potion kinds; the 25 enemy kinds; event identities and their options in Neow/event records; encounters and formation/member roles in enemy records; power, intent, orb, source-option, room-object and public-fact keys present in those same records. The sources distinguish character-specific basics as distinct kinds, every Note-reachable off-color basic, generated Status/Curses and event-stored special cards. A color/rarity/title alias does not merge kinds. The closed set is not “all IDs returned by a runtime card lookup”: unsupported/unreachable identities are absent, not silently accepted.

Candidate alias rules (proposal): exact source enum/library spelling aliases only; aliases are normalized nowhere. For names that differ by spaces/punctuation, author the canonical snake-case key explicitly and record the original exact spellings as separate aliases. Duplicate display names across owner/namespace remain distinct by canonical key. Unknown external strings reject creation input; source fallback to Iron Wave for malformed stored ID is not adopted as native validation behavior.

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

Property codes `1..45` have the following closed meanings (the code numbers are stable only within this proposal): `1 base_damage`, `2 displayed_damage`, `3 base_block`, `4 displayed_block`, `5 magic`, `6 captured_x`, `7 cost_until_played`, `8 cost_combat`, `9 cost_free_once`, `10 autoplay`, `11 ignore_total`, `12 purge`, `13 public_source_site`, `14 public_copy_origin`, `15 retained_duration`, `16 delayed_duration`, `17 uses`, `18 gold_price`, `19 hp_price`, `20 max_hp_price`, `21 selected_order`, `22 can_skip`, `23 can_confirm`, `24 scry_count`, `25 public_discard_count`, `26 public_hp_loss_count`, `27 public_play_count`, `28 public_power_count`, `29 mantra`, `30 orb_slots`, `31 focus`, `32 kill_growth`, `33 potion_capacity`, `34 key_kind`, `35 burning_buff_public`, `36 source_option_stage`, `37 draw_order_known`, `38 outgoing_card_public`, `39 upgrades`, `40 source_visible_counter`, `41 nominal_price_binary32`, `42 intent_visible`, `43 payment_status`, `44 transform_input_kind`, `45 public_turn_cost_modifier`. Code 40 permits only explicitly registered `(content_key, subkey)` pairs; it is not an escape hatch. D04 time fields are not inserted as an unqualified property.

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

Replay framing remains exactly the reviewed 2026-10-19 candidate's grammar and six structural goldens: `A1RP`, versioned header, eight schema pairs, composed descriptor hash, explicit source/rules revisions, seed/partition/stream/episode, ordered optional creation/profile/time/core attachments, length-framed accepted decision records, exact end status/optional diagnostic, strict canonical JSON and exact EOF. Public digest order remains descriptor hash; fixed context values then masks; player values then masks; tables 1–18 in order, each row count then row-major values then same-order masks. G01–G06 are byte/framing goldens only, not gameplay-replay fixtures. Existing replay framing, goldens, seed/HMAC/RNG/snapshot/combat-observation pins are not changed. Private core encoding and full content/site descriptor bytes remain unresolved; no final hash/fingerprint is claimed until NC11/12 and the unresolved bindings close.

Successful reset/read/step/replay deliveries contain detached, schema-qualified attachment bytes and identity; same-step vector reset snapshots the retired episode's final observation/resources/outcome/identity before installing replacement. Final slots index returned-row order. No action is applied to replacement. Manual terminal slots persist until explicit reset. Replay is from-start only, checks canonical semantic choices/current menus, records accepted time input where selected, and reports first mismatch without repair or imported-state substitution.

## 6. Closure matrix, failures-first checks, and residuals

| Obligation | Before | New contract delta | Status after this document |
|---|---|---|---|
| NC11 semantic dictionary | Namespace/count and enumeration idea, not exact alias collision/equality rule | Closed 13 namespace IDs; canonical key/alias record and two-way equality failure classes; finite content families linked to all seven normative records | **Partial.** Literal generated catalog and every source alias/site still require inventory equality. |
| NC12 public fields/private core | 18 widths and 45 property codes, but no per-field domain/visibility association | Every packed table/column has explicit carrier/domain/visibility/cross-reference semantics; value tags and private-core boundary explicit | **Partial.** Full source field-to-domain and content visibility equality; private hidden/RNG core encoding remain. |
| Action tuples | 21 action labels but generic field interpretation incomplete | Every code has exact source/target/parameter shape and validation ownership | **Candidate complete**, pending exact current core-choice membership/callback binding from execution lane and cross-document review. |
| Effect sites | Key syntax/families, no full public/private exact identity binding | Stable tuple, required site record, finite inventory families and bidirectional equality test | **Partial.** Literal site set and site-to-core/private code equality require exact reconciliation. |
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
| Effect-site equality and full private core | No full equality evidence supplied | Required before final schema hash; RNG/maps and execution own source bindings. |

Required future acceptance rows, all **UNEXECUTED gameplay cases**: all18 widths/typed references/empty shapes and large legal growing sequences; every property/content alias positive and collision negative; both-way source-owner/site and action-to-core membership equality; every source integer/binary32/boolean/count field's domain vs carrier boundary; helper-cap non-clipping controls; detached Note before/after commit and death/fault/reset/same-step output isolation; each profile/time/handoff variant round-trip from-start replay; invalid final-row reset/step atomicity; whole-pool quiescence/fault without rollback; all six framing goldens plus malformed byte controls; exact compatibility regressions; exact map/relic/event/combat RNG source-scope/cursor joints after their owning lane resolves them.

These are acceptance obligations, not claims they were run. Current research tests are document/schema guard checks only. They do not discharge future implementation acceptance or human approval.

## 8. Explicit controller handoff

Main must reconcile this candidate with the independent RNG/maps and execution documents and the complete normative source catalog. Required named inputs: (1) RNG/maps: source scope, root/history measure, ordered catalog construction, joint/linked cursor bindings; (2) execution: actual dispatch/site call equality, exact core choice tags, continuation/checkpoint IDs, time settlement/timer predicates, and successor transition children; (3) content integration: literal finite canonical IDs and source aliases, per-field source numeric/visibility map and transitive site equality; (4) human: profile/history preset, Note/fault output variant, time policy, Act2-ready boundary and map payload choice. No sibling answer is assumed by this document.

The final schema hash/fingerprint is intentionally withheld. This proposal does not close B09 numeric-source mapping, B10 semantic/site/private-core bindings, B11 human profile choice, B12 time settlement, B14 whole-spec human approval, or any implementation/plan/start gate. It is a concrete candidate contract and a precise integration checklist—not “complete by manifest counts.”
