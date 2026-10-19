# Act 1 source identity and numeric-domain joins

**Status: bounded source-join supplement; assignment .44 remains incomplete.** This is a research contract, not final native schema, runtime validation, whole-spec approval, or implementation authorization. The archive is the pinned working reference described in the consolidated specification; evidence below is static/source-record evidence only. No original game class or initializer was executed.

## 1. Identity and code assignment contract

A source identity is the exact owner-qualified source key, not localized display text, a Java enum ordinal, an allocation identity, nor a wire number. A native canonical key is stable, nonlocalized ASCII `snake_case` within its namespace. Source aliases are byte-sensitive and case-sensitive; no normalization, trimming, case-folding, punctuation rewrite, or “first matching alias” is permitted. Duplicate display labels in distinct namespaces/owners remain distinct identities. Dynamic per-instance handles and numeric wire allocation remain the native owner’s responsibility.

A catalog row has these normative columns: `(namespace, canonical_key, exact_source_identity, exact_aliases, lifecycle_owner, public_eligibility, evidence_grade)`. Each source identity and each `(namespace, owner, alias)` must resolve to exactly one canonical key. Unsupported or unproven identities fail closed. Catalog order is native canonical UTF-8 byte order, not historical host/JRE enumeration order. This is a selected native policy, not a source-order claim.

The finite namespace families required by this Act 1 envelope are cards, relics, potions, enemies, encounters, powers, events, rooms, rewards, choices/source-options, properties, intents, orbs, room objects, and public facts. Their reachability includes Ironclad A0, legally Note-reachable other-character cards, Prismatic Shard reward cards and dependent generated/control cards, transformed/generated cards, Act 1 routes and required Act 2 initialization. It does not include other-character starts/runs or Act 2 decisions. A generator must not widen this universe to every ID returned by an arbitrary runtime lookup.

## 2. Concrete identities whose source rows are closed in accepted records

The table gives exact source-facing IDs and distinct owner-qualified keys for the full local enemy universe. Move identifiers remain subordinate to the enemy key; e.g. move byte `1` is not a global intent key. The records establish 25 local enemy identities; they do not establish a global all-acts monster namespace.

| Canonical key | Exact source identity | Owner / emission eligibility |
|---|---|---|
| `cultist` | `Cultist` | Act 1 encounter factories; emit only when actor is in the current public formation. |
| `jaw_worm` | `JawWorm` | Act 1 encounter factories; same condition. |
| `red_louse` | `FuzzyLouseNormal` | Act 1 encounter factories; same condition. |
| `green_louse` | `FuzzyLouseDefensive` | Act 1 encounter factories; same condition. |
| `acid_slime_s` | `AcidSlime_S` | Act 1 encounter or dependent split; same condition. |
| `acid_slime_m` | `AcidSlime_M` | Act 1 encounter or dependent split; same condition. |
| `acid_slime_l` | `AcidSlime_L` | Act 1 encounter, dependent split, or boss summon; same condition. |
| `spike_slime_s` | `SpikeSlime_S` | Act 1 encounter; same condition. |
| `spike_slime_m` | `SpikeSlime_M` | Act 1 encounter or dependent split; same condition. |
| `spike_slime_l` | `SpikeSlime_L` | Act 1 encounter; same condition. |
| `blue_slaver` | `SlaverBlue` | Act 1 encounter; same condition. |
| `red_slaver` | `SlaverRed` | Act 1 encounter; same condition. |
| `looter` | `Looter` | Act 1 encounter; same condition. |
| `fungi_beast` | `FungiBeast` | Act 1 encounter; same condition. |
| `fat_gremlin` | `GremlinFat` | Act 1 encounter; same condition. |
| `mad_gremlin` | `GremlinWarrior` | Act 1 encounter; same condition. |
| `sneaky_gremlin` | `GremlinThief` | Act 1 encounter; same condition. |
| `shield_gremlin` | `GremlinTsundere` | Act 1 encounter; same condition. |
| `gremlin_wizard` | `GremlinWizard` | Act 1 encounter; same condition. |
| `gremlin_nob` | `GremlinNob` | Act 1 elite; same condition. |
| `lagavulin` | `Lagavulin` | Act 1 elite; same condition. |
| `sentry` | `Sentry` | Individual actors in the Act 1 three-Sentry elite encounter. |
| `slime_boss` | `SlimeBoss` | Act 1 boss; same condition. |
| `hexaghost` | `Hexaghost` | Act 1 boss; same condition. |
| `the_guardian` | `TheGuardian` | Act 1 boss; same condition. |

The elite encounter key `three_sentries` denotes the encounter factory/formation, not a monster kind; its three actor instances each use `sentry`. For the two Louse kinds, the source class aliases are `LouseNormal` and `LouseDefensive` respectively, while the exact source IDs above remain `FuzzyLouseNormal` and `FuzzyLouseDefensive`.

For the four-character paired primary-card families, the source class families and local record boundaries are: Ironclad (3 basic, 20 common, 36 uncommon, 16 rare); Silent (4 basic, 71 nonbasic, plus generated Shiv); Defect (4 basic, 71 nonbasic, 2 Status dependencies, 4 orb kinds); Watcher (4 basic, 71 nonbasic, 8 physical dependencies and 3 nonphysical Wish options). These counts are coverage summaries, **not** identity equality. Exact card IDs and aliases are the `AbstractCard.cardID` values, including the distinct character starter spellings, Status/Curses, Note event cards, Shard-eligible cards, and named generated/control kinds in the accepted paired records. A display-name or class-name fallback is not a key. The previously accepted 131-row card ID/class dictionary is only its documented Ironclad/colorless/Status/Curse subset and is not used as proof of this larger set.

The reviewed event/reward records establish 25 event identities and option behavior; the economy/potion record establishes 33 eligible potion constructors; the relic reconciliation established 141 candidate relic identities. These finite inventories are useful source-set fixtures, but this supplement does not reproduce their complete literal aliases/owner rows. In particular, option identity is event-owner-qualified, potion aliases are factory-ID-qualified, and relic aliases are relic-owner-qualified; counts alone do not freeze them. This is an explicit incomplete join, not an import contract that asks the native implementer to infer missing IDs from another document.

## 3. Numeric-domain/member join rules

The source primitive domain, reachable values, local helper caps, selected native representation, and public visibility are separate facts. Direct Java `int` members have source representability `[-2^31, 2^31-1]`; where Java `int` arithmetic is executed, overflow wraps at that operation in source behavior. A wider native carrier must sign-extend and preserve the resulting value; it must not narrow an out-of-range input. A Java `byte` intent/move discriminator is signed `[-128,127]` and must be decoded under its owning monster before it becomes a semantic key. Boolean fields are exactly `0/1`. Collection sizes/indexes derive from source `int`, but imply no fixed content limit. Allocation/carrier failure is administrative, never a reason to clip a legal source value, shorten a menu, omit a row, or fabricate defeat.

| Source owner/member or derived field | Read/write/default/reset and numeric law | Visibility and mapping |
|---|---|---|
| `MapRoomNode.x`, `.y` | Source signed `int`; read as coordinates, no clipping. Defaults/construction and map writers remain map-owner-specific. | Public for exposed map nodes. Map node identity is a row reference, not serialized object identity. |
| `MapRoomNode.hasEmeraldKey` | Boolean node marker set by burning-elite map selection and read at elite prebattle/room reward paths. It means the node’s burning designation. | Conditional on public map-observation policy. **Not** `Settings.hasEmeraldKey`, the separately acquired player key. |
| `Settings.hasRubyKey`, `.hasSapphireKey`, `.hasEmeraldKey` | Boolean acquired-key state; writes at their respective acquisition choices; independent from map marker. | Public only when the actual run-key state is entitled. Never alias the burning-node marker to this property. |
| `AbstractCard.cardID` | Exact source string identity; constructor/transform/copy chooses kind. Aliases are exact and namespace-scoped. | Key resolves to a canonical card key; no localized display name or class-name fallback. |
| `AbstractCard.timesUpgraded` | Signed `int`; constructor starts at owner-specified value; upgrade writers increment, with Java overflow at that operation. Searing Blow and specialized constructors/copies have nonuniform construction/replay behavior. No generic maximum. | Public for entitled card instances; instance value, not a kind-global counter. |
| `AbstractCard.misc` | Signed `int`, including owner-defined negative sentinels and state; constructor/default, reset and mutator laws vary per class. Java arithmetic wraps where performed. | Conditional: mask when card instance’s relevant state is not legitimately visible. Preserve instance/receiver identity. |
| `AbstractCard.cost` | Signed `int`. `-1` is X, `-2` is unplayable, `0` is an ordinary cost; other negative values are not silently reclassified. Constructor/copy/upgrade and mutators are class/operation-specific. | Entitlement-gated; base cost and its derived kind are masked together if hidden. Never substitute sentinel with zero. |
| `AbstractCard.costForTurn` | Distinct signed `int`, mutable separately from base cost; setter guards and ordered writers govern. ResetAttributes restores turn cost from mutable combat cost, not printed base cost. | Independently entitlement-gated; never derive this from `cost_kind`, free flag, or base cost. |
| Cost/free mutations | `setCostForTurn` negative ordinary requests clamp to zero only under its own guard; negative X/unplayable states are preserved. Positive mutable combat cost can independently set `freeToPlayOnce`. Confusion/randomize-hand, Enlightenment, Madness and Corruption use different branches. Eviscerate copy mutates the source; Blood for Blood copy updates the returned copy. | Expose separate base cost, turn cost, free flag and captured-X semantics only at their own public checkpoints. No generic total precedence rule. |
| `AbstractCreature.currentHealth`, `maxHealth`, `currentBlock`, `gold` | Signed `int` members. Constructor/start defaults are owner-specific; damage/heal/maxHP/payment methods mutate at their semantic effect boundary. Do not infer universal nonnegative range from normal UI. `addBlock`’s processed result caps current Block at 999 at that helper only. | Public run/player fields; actor fields only for public actors. No universal 999 resource cap. |
| Strength amount | StrengthPower constructor/stack/reduce clamps result to `[-999,999]` on those paths; addition/subtraction precedes clamp. Other powers/counters do not inherit this cap. | Owner-qualified power rows; preserve signed amount and owner, no generic counter saturation. |
| `EnergyManager.energy`, `energyMaster` | Signed `int` members. `addEnergy` helper cap at 999 applies only on that method path; direct/setter paths remain distinct. | Current energy and energy master are separate fields and visibility. No 999 input clamp. |
| `AbstractRelic.counter`, `usedUp` | `counter` is signed `int`, potentially owner-defined negative/special values; constructor and per-relic reset/write behavior differ. `usedUp` is boolean. No global bound. | Relic identity and owner inventory position are public; counter is conditional on that relic’s source-visible predicate. |
| `AbstractPotion.ID`, `slot`, `potency` / `getPotency()` | ID is exact factory identity; slot is signed source `int`; potency is signed `int` and the accessor result is authoritative. Sacred Bark multiplication follows source accessor semantics, including Java overflow at the source arithmetic boundary. | Slot/occupied state public; identity and potency only for an occupied, entitled slot. No generic positive potency assumption. |
| `AbstractMonster.intent`, `nextMove`, `intentDmg`, `intentMultiAmt` | `intent` object plus signed-byte local move ID are owner-qualified; payloads are signed `int`. Intent update/constructor and payload calculation order are monster-specific; float-derived payloads use their source binary32 boundary before narrowing. | Mask hidden/suppressed intent and derived payload together. Never emit byte ordinals as global intent codes. |
| `AbstractPower.ID`, `.owner`, `.amount` | Exact string identity, object owner reference, signed `int` amount; constructors, stacking, decrement and removal are power-specific. | Public power key and owner-qualified order; amount conditional, never a universal saturating count. |
| `AbstractOrb.ID`, `.passiveAmount`, `.evokeAmount` | Exact orb ID and signed `int` values after applicable focus/lock-on hooks. Calculation stages may be binary32 before source narrowing. Base class has no common `counter` member. | Actual occupied slot order; no synthetic counter or absent-orb rows. |
| `ShopScreen` settled prices | Generated discounts/jitter use float intermediates and source-specific rounding order; once settled, shop card/relic/potion price fields and `actualPurgeCost` are signed `int` paid/displayed amounts. Preserve source settlement and integer overflow behavior. | A public price requires a registered owner/member and entitled live offer. The carrier is settled signed-i32, not float. |
| Other event/reward/composite prices | No generic price owner or property may be inferred from price-like display text or intermediate float. Exact member, settlement, payer, write point, and reveal checkpoint required per owner. | Do not emit until that owner proof exists. Property 41 is reserved and unemittable absent actual owner/emission proof. |
| Float intermediates | Binary32 operands/operation ordering/rounding boundary are source-method-specific. Do not replace with binary64/exact-rational arithmetic or round only at a later observation. Preserve NaN/infinity/signed-zero policy only where source field permits. | Raw intermediate values are not public prices; native administrative overflow must not truncate a settled legal value. |
| `property_code=41` | No proven actual owner/member/emission path in accepted evidence. | Exclude it from public emitted property rows and from the native consumer’s recognized/emittable property set. Do not serialize “unknown property 41” as an extension. |

These are source facts only at their stated member/helper/owner boundaries. They do not imply every possible input is naturally reachable. The native implementation may choose a wider safe representation but may not clamp source state. Carrier i64 and signed-i32 settled price are representation choices, not source behavior.

## 4. Visibility, ownership, lifetime, and absent values

Every emitted field requires a complete `(source owner, source member or exact producer, read checkpoint, writer/mutator set, constructor/default/reset, lifetime, visibility predicate, destination)` join. The source object pointer remains private; instance references become slot-local typed refs. Current container order is an observation only when the container is public at that checkpoint. Hidden draw order is not a stable catalog order; Frozen Eye may reveal current draw order, whereas Runic Dome suppresses enemy intent. A face-down board location may be public while its value remains unknown. An absent row, a present-but-unknown field, and known zero are distinct states.

Fixed members already identified in the accepted source crosswalk include run act/floor/ascension/turn; player class/resources/energy/slots/orbs/keys/stance; map coordinates/room/taken/node marker; card identity/cost/upgrade/misc/flags; relic identity/counter/used state; potion identity/slot/potency; actor resources/intent payload; power identity/owner/amount; and orb identity/passive/evoke amounts. Those keys are not a complete source-member inventory: per-owner reader/writer/default/reset and public predicates remain required, especially subclass-only properties, offers, room objects, action tuples and relation endpoints.

The accepted property vocabulary is finite and version-local. Property 41’s stated reserved label has no owner proof and therefore has no emission path. Property 40 is not an open extension bag: only separately registered owner/subkey pairs may use it. Public facts/history are environment knowledge ledgers, not permission to reveal private RNG/history fields. All hidden-state and owner rules are fail-closed.

## 5. Numeric rounding and failure boundaries

* Integer arithmetic follows the actual source operation sequence. Java signed-int add/subtract/multiply wraps modulo `2^32` at that operation; no neighboring helper’s clamp is generalized.
* Float-driven source calculations retain binary32 precision and operation order, then round/narrow only where the source does. Card damage/Block, potion percentages, chances and price generation are not universal float fields.
* In particular, settled shop price is already an integer source result; do not expose or recompute it from a float wire value. Preserve ties/rounding order and integer discount application as the relevant source owner defines them.
* `AbstractCreature.addBlock` and energy helper bounds are exact helper-local contracts, not field-wide caps. Strength’s `[-999,999]` is limited to the named StrengthPower constructor/stack/reduce paths.
* A legal source value outside an elected carrier’s exact range, nonrepresentable conversion, failed checked allocation, or arithmetic necessary for representation faults administratively with provenance. It never becomes clipping, omission, a fabricated game outcome, retry, or a legal-input rejection.

## 6. Actual equality evidence and remaining obligations

Accepted E29 reports a 146-cell candidate crosswalk and a 131-card bounded alias table; neither is proof that all source-owned fields or legal identities are equal to a complete emitted set. Static declarations alone also do not prove a source member is read, written, reset, owner-qualified, or public. The full-source set reconciliation must independently enumerate retained/static actual declarations and read/write/constructor/reset/use sites, derive the in-scope expected set from actual reachability and accepted normative records, and compare it to the independently maintained delivered rows. A checker built with the same authored row list on both sides is circular and invalid.

For that reason this document does **not** claim full catalog or numeric closure. The following concrete gaps remain open for review/main routing:

1. Full literal canonical-key/source-ID/alias/owner/public-eligibility rows for all four card families, legal off-color/Note/Shard/generated/transform dependencies, 141 relics, 33 potions, 25 events and their choices, encounters, powers, rooms, rewards, properties, intents, orbs, and source options. Existing accepted primary tables remain evidence, but their counts and display labels are not reproduced here as complete aliases.
2. Per-owner read/write/mutator and constructor/default/reset rows for each required live member, including exact valid negative/X sentinels and reachable domain where established; the crosswalk’s source-qualified gap rows remain open rather than being filled by a generic `int32` label.
3. Exact visibility predicate/checkpoint and lifetime for every public member/property and private exclusion, including each offers/choices/property relation owner. Property 41 is explicitly excluded pending proof.
4. Independently grounded expected-vs-delivered source-field and alias equality, and failure-first negative controls for missing fields, aliases, duplicate aliases, wrong owner, domain, visibility, rounding/overflow, plus preserved failing attempts and honest evidence grade. This draft does not represent a count/signature check as equality.

The task is therefore **not complete**. Do not treat this document as an API/codec, final source registry, native implementation input without these gaps, or a statement that prior accepted evidence proves them. Native owner may consume only the concrete facts above as qualified source inputs; native numeric wire IDs/instance IDs remain its responsibility.

## 7. Accepted public evidence locators

These links are normative source-record references and their own evidence grades/limitations apply: [consolidated Act 1 specification §§12–13 and B07/B09/B10/B11](../superpowers/specs/2026-10-19-ironclad-a0-act1-consolidated-design.md); [semantic/native contract closure (E29)](2026-10-19-act1-semantic-contract-closure.md); [Ironclad, colorless, Status, Curse and dependent cards](2026-10-19-act1-content-red-colorless.md); [Silent cards](2026-10-19-act1-content-silent.md); [Defect cards/orbs](2026-10-19-act1-content-defect.md); [Watcher cards/choices](2026-10-19-act1-content-watcher.md); [enemy/encounter records](2026-10-19-act1-content-enemies.md); [events and Neow](2026-10-19-act1-content-event-neow.md); [economy, rewards, and potions](2026-10-19-act1-content-economy-potions.md); [relic source records](2026-10-18-act1-relics-closure.md). They are public inputs, not private evidence dependencies for the normative rules stated above.
