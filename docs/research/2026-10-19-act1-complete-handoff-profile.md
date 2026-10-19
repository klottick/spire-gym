# Act 1 creation profile and resolved Act 2 handoff

Date: 2026-10-19. Companion contract for the policy-amended Act 1 specification.

**Status: bounded source facts plus explicit native engineering contract.** This is
not whole-spec approval, an implementation plan, an implementation authorization,
or Act 2 gameplay. Source descriptions below are static/reconstruction facts from
the pinned owned archive, not original-game execution. Selected native rules are
normative proposals for whole-spec review. All future gameplay cases remain
**UNEXECUTED**.

## 1. Boundary and inputs

This document closes the profile and successor-state engineering joins for
A1C-REQ-028/029 and B05/B10/B11. The selected endpoint is a resolved Act 1 boss
successor, after the Act 2 map and required initialized state exist, and strictly
before the first Act 2 player decision. Boss lethal damage, reward-screen closure,
the first fade flag, or `dungeon.beaten` alone is not success. No Act 2 decision or
player action is taken here.

Normative baseline: [consolidated spec](../superpowers/specs/2026-10-19-ironclad-a0-act1-consolidated-design.md)
at main `2e090d78297a236bfd549797e460db40d47e709f`, SHA-256
`06c5bca48b579c27388bd23a8d4e0d062109efc02061ad95e4950a0788bcbf9e`.
Relevant reviewed inputs: [E07 Acts 2/3 setup](2026-10-18-ironclad-a0-acts23-research.md);
[E15 profile/history](2026-10-19-act1-profile-time.md);
[E16 continuations](2026-10-19-act1-continuations.md);
[E20 enemy records](2026-10-19-act1-content-enemies.md);
[E25 Neow/events](2026-10-19-act1-content-event-neow.md);
[E26 economy/potions](2026-10-19-act1-content-economy-potions.md);
[E28 execution](2026-10-19-act1-transitive-execution-closure.md); and [E29 partial
native candidate](https://github.com/klottick/spire-gym/blob/d3e9b7a3565932e83235936d083e906f087ab078/docs/research/2026-10-19-act1-semantic-contract-closure.md) (immutable author head `d3e9b7a3565932e83235936d083e906f087ab078`,
document SHA `db5061114732ba8ea18099e81249df1fdfc015055ed7ab528c8ab5121ff4d599`);
and the independently reviewed, main-routed [RNG and representative map contract]
(https://github.com/klottick/spire-lab/blob/35808a1b61a79566ec5a18cd50fea85ae60eadc0/docs/research/2026-10-19-act1-complete-rng-maps.md)
(merged main `35808a1b61a79566ec5a18cd50fea85ae60eadc0`, document SHA-256
`e24d7f0667ccf6397a9ee84e5a1ccc72d40294f3c5cb9269f98b4f61d22447c1`). This last
input is accepted only for its bounded RNG/maps producer; it does not close this
document's remaining semantic/native joins or the whole spec.
These are bounded inputs, not a new source audit. Owned archive
SHA-256 is `cfad868ac8d65a88e71a0bf096fb09f78811e553effe0787c5309a655e081673`.
No original class, game, initializer, or gameplay was executed.

Evidence marks: **F** bounded source fact; **S** selected native policy; **E**
explicit native engineering contract proposed here; **I** named producer import not
yet bound to a reviewed support-lane result. A proposal is not a source claim.

## 2. Creation profile: versioned, owned, closed grammar

### 2.1 Exact record

`CreationProfileV1` is a fully materialized immutable value. It is validated and
deep-copied before entropy acquisition, semantic run identity/ordinal reservation,
slot abandonment, or any episode mutation. There are no absent fields, implicit
per-field defaults, account/save imports, mutable global preference reads, or
partial pending/committed profile states.

| Field | Type/domain | Owner and effect |
|---|---|---|
| `contract_id` | exact ASCII `spire-lab-act1-creation-profile-v1` | Version gate; any other value rejects. |
| `ruleset` | exact tuple `standard=true, modded=false, character=Ironclad, ascension=0, act4_enabled=true, fully_unlocked=true` | Fixed task profile, not user-tunable. “Unlocked” is not history. |
| `prior_ironclad_neow_eligible` | boolean | Selects eligible Neow history branch only; independent of boss discoveries. `true` is experienced preset. |
| `bosses_seen` | three sorted, duplicate-free sets over `Act1={Guardian,Hexaghost,SlimeBoss}`, `Act2={Champ,Automaton,Collector}`, `Act3={AwakenedOne,DonuAndDeca,TimeEater}` | Run owns per-act history. Act1 initializes from input; Act1 discoveries add at their prebattle hook; Acts2/3 sets initialize from input and remain separate. No single `all_seen` bit. |
| `stored_note_card` | `StoredCardV1 { card_id: enum, upgrade_count: nonnegative signed-i32 source count }` | Per-episode initial Note payload. Default is `Iron Wave, 0`; not a feature before legal reveal. |

No other fields are legal. In particular, no custom deck/resources, start HP/gold,
Ascension override, unlock progression, character switch, Note enable/disable,
next boss, map, key, RNG cursor, combat state, bottle, temporary cost, misc growth,
or saved game path. Extra/unknown keys, wrong types, duplicate set members, invalid
enum values, out-of-domain counts, integer overflow, or unsupported contract ID
reject the **whole addressed batch** before any row advances. No fallback is applied.

The fixed standard-start values are not knobs: Ironclad A0 starts at 80/80 HP and
99 gold, Burning Blood, five Ironclad Strikes, four Ironclad Defends, Bash, three
empty potion slots, and Ruby/Sapphire/Emerald all absent. Starting deck instances
are new identities with zero upgrades. These are per the accepted baseline's
bounded E09/E12 anchor; this author does not upgrade it to an independent source
numeric certification.

### 2.2 Named defaults and independent episode semantics

`ExperiencedA0V1` is the only implicit convenience selection when the caller omits
the profile argument as a whole: it expands before initialization to the exact
record `standard fully-unlocked ruleset; prior_ironclad_neow_eligible=true;
Act1={Guardian,Hexaghost,SlimeBoss}; Act2={Champ,Automaton,Collector};
Act3={AwakenedOne,DonuAndDeca,TimeEater}; stored_note_card={IronWave,0}`.
The resolved value is captured in replay. A supplied raw record has no omitted
field defaults. A supported explicit override changes only the Neow boolean,
per-act seen sets, and/or legal stored-card record. It does not require account
provenance, a training episode, or proof the profile was played. The preset is a
creation baseline for every independent episode—not inherited state.

Every explicit reset and vector same-step replacement receives its own resolved
profile input (or the pool's explicitly configured, already resolved preset value).
Replacement never reads the retired episode's Note, discoveries, Neow outcome,
unlock/history mutations, or profile candidate. The detached completed final is
captured before replacement and is not an alias to the successor episode. Invalid
last-row profile rejects every row before RNG/identity/time advancement. A valid
reset of a live run is administrative abandonment, not death, truncation, or a
gameplay choice. This does not imply runtime reset rollback after dispatch.

The source's Neow eligibility is related to standard death/abandon at floor >=16,
not prior boss victory; the user-selected experienced default is not that source
history. `project seed present` is not original custom-seeded mode. Act1 first
unseen boss priority is Guardian, Hexaghost, Slime Boss; the all-seen branch uses
the selected native random contract, not a source-seed parity demand. Boss
discovery is recorded at the actual prebattle hook, not at announcement, boss-list
construction, or victory; later death does not erase it. Acts2/3 discovery sets
are carried as explicit profile history, not collapsed into an inherited flag.

## 3. Stored Note card grammar and validation

### 3.1 Stored value is deliberately minimal

`StoredCardV1` stores only a canonical supported card ID and the source signed-i32
upgrade count to replay on a freshly constructed card. Negative counts reject.
There is no card instance UUID, persisted `misc` field, bottle flag, cost override,
turn/free flag, retain flag, potion, or card-specific combat/permanent growth.
`misc` is deliberately omitted: retrieval starts from the constructor's default
and does not import old growth. Retrieval applies exactly `upgrade_count` calls
under that card's legal upgrade behavior. Thus ordinary once-upgrade cards allow
0/1; `Searing Blow` is repeatedly upgradeable and is not limited to one by a
generic boolean. The count is not reduced to a boolean. Other unupgradeable and
nonrepeatable kinds allow only their actual domain (normally 0 or 0/1); invalid
counts reject, never clamp or silently replay a different value. Check source
signed-i32 conversion and that card's actual upgrade/mutation arithmetic before
initialization. Do not impose an arbitrary collection/count cap; any tighter
domain must be established by the exact card law and legal native transport.

### 3.2 Identity whitelist

The `card_id` is a canonical semantic identity, not localized text, color, rarity,
substring, Java class name, or “any ID accepted by source fallback.” The finite
candidate grammar consists of the physical permanent card kinds in Appendix A of
the baseline: all Ironclad, Silent, Defect, Watcher and ordinary colorless card
identities; the twelve other-character basic Note identities; the four removable
event cards `Apparition`, `Bite`, `J.A.X.`, `Ritual Dagger`; and only otherwise
reachable ordinary removable curse identities. Aliases normalize at validation
to one canonical ID; duplicate aliases within one profile are not a set and are
rejected as malformed input. It does not admit Status cards, generated temporary
physical specials (`Shiv`, `Miracle`, `Smite`, `Safety`, `Insight`, `Through
Violence`, `Beta`, `Omega`, `Expunger`), wish-only options, `Curse of the Bell`,
Ascender's Bane, Necronomicurse, Pride, nonphysical options, arbitrary library
IDs, or a merely temporary copy. A source ID must also resolve to a standard
permanent instance whose actual removal eligibility is valid; legal off-color
does not mean another character's start or event pool is imported.

The removable-curse set is exactly the ordinary removable curse support (Clumsy,
Decay, Doubt, Injury, Normality, Pain, Parasite, Regret, Shame, Writhe) where the
canonical content catalog marks it as a permanent card and `canRemove`/Note
candidate rules accept that instance. `Curse of the Bell`, Ascender's Bane and
Necronomicurse are rejected. Source Note's observed missing preference fallback
to Iron Wave/0 and unknown-library-ID fallback are **not** native behavior. The
native input is validated atomically and malformed/unsupported IDs fail closed.

The whitelist is checked against the named content manifest at initialization:
`supported_stored_card_ids` must equal the catalog's legal permanent/removable
card IDs after exclusions above, and each member must have its exact upgrade
domain. This is an equality binding, not “all colors,” rarity filtering, or a
numeric cap. Content owner must provide the literal ID-to-kind/upgrade-law rows;
the accepted 131 ID/class crosswalk is only the Ironclad/colorless/Status/Curse
subset and is not falsely treated as complete catalog evidence. Until main routes
the reviewed complete catalog/numeric delivery, this public whitelist grammar is
the precise legal *set definition*, while literal final enum codes and every
ID-specific upgrade maximum remain an explicit integration dependency (B09/B10),
not invented source facts. No user decision is needed to select enum widths or
alias handling.

The executable consumer contract is `StoredCardCatalogV1`, supplied by the
main-routed accepted catalog/numeric producer; it is not defined by a second local
card list. It is an immutable versioned manifest with `catalog_revision`,
`source_archive_sha256`, and finite `rows`. Each row has exactly `canonical_id`,
`accepted_aliases` (including canonical spelling), `permanent_instance`,
`note_candidate`, `removable`, `upgrade_kind` (`none`, `once`, `repeatable`),
`max_upgrade_count` (nullable only for a genuinely unbounded source law), and
`checked_mutation_domain` (source member(s), widths, checked read/write/overflow
predicate). Duplicate canonical IDs/aliases or unknown row fields reject.

The consumer validates before episode mutation: (1) schema/revision and archive
digest match the accepted routed producer receipt; (2) only a listed alias
normalizes to its canonical ID, and unknown IDs, shared aliases, or noncanonical
input where canonical encoding is required reject; (3) `permanent_instance`,
`note_candidate`, and `removable` are all true; count is nonnegative and signed-i32
representable; and the row law is exact (`none` => 0, `once` => 0/1,
`repeatable` => the published finite domain or published checked unbounded law);
(4) each replayed mutation satisfies that row's checked arithmetic domain, with no
wrap, saturation, clamp, partial replay, or fallback; (5) the canonical row set
equals the §3.2 semantic whitelist and every alias/eligibility/upgrade/arithmetic
field equals its routed producer receipt. Missing, extra, duplicate, changed, or
stale rows reject the whole addressed batch before RNG/identity/time advancement.
Negative controls are missing/stale receipt, omitted/extra ID, duplicate row or
alias, alias collision, temporary/status/nonremovable row, wrong eligibility,
forged upgrade law/count/max, signed-i32 overflow, and arithmetic-domain violation;
all reject atomically. These are contract controls, not executed catalog tests.
Literal rows and arithmetic limits remain unbound until the accepted receipt is
routed; this defines the equality predicate without inventing that data.

### 3.3 In-run Note ordering is unaffected

Source Note Take calls the incoming card's obtain hooks, inserts it into the
master deck, then calls deck-change hooks **before** constructing outgoing
selection; incoming itself may be selected. This direct path does not run the
ordinary Fast/Show obtain constructor and does not consume Omamori. Outgoing
selection uses purgeable permanent instances, then excludes bottled instances.
Selection is from the live deck after the incoming addition. Removal uses the
actual object-removal hooks, then the source retains that same live object as the
pending save reference. The departure/next-room-transition checkpoint persists
its then-current ID and upgrade count—not the selection-time tuple, misc growth,
or a full card object. Preserve the order and source live-reference fact. Native
profile capture must not introduce mandatory `IncomingApplied`/`PendingTransition`
export API; it can own in-run progress privately and expose only a minimal honest
optional result.

Configured creation values are caller-known initialization inputs; this does not
authorize policy metadata, observation table length/order, reward/menu ordering,
or diagnostics to re-expose the hidden Note ID. Until the event reveals it, no
policy observation, candidate, feature, final metadata, or reset side channel may
identify that stored card. Once legally revealed, show only the legitimate
revealed value. Caller input is not itself observation permission.

## 4. Boss victory is a multi-stage transition, not a terminal

### 4.1 Ordered phases and commitment checkpoints

The source facts are bounded by E16/E20/E26/E28; selected native settlement is
semantic. Preserve durable changes at their actual boundaries, and do not create
a global rollback transaction. Each row names who owns the transition:

| Phase / owner | Required state transition | Completion predicate / exclusions |
|---|---|---|
| `BossCombat` / combat + run | Resolve lethal packet, boss-specific death/split/cannot-lose and terminal logic; apply actual death/rescue rules. Live permanent mutations (Feed/gold/maxHP/card growth/relic counters/potions) already committed remain. Mark boss discovery at prebattle hook if reached. | An actual player death terminates −1. Boss dying/`isDying`, queued lethal packet, split child, combat/room escape, room victory latch or combat reward construction is not +1. A legal escape settles its room/combat path and the run continues; it does not end this task episode. No post-death heal. |
| `BossCombatRewards` / reward owner | Generate the boss combat reward rows under boss rarity/context, with their own card history/pity and other reward law. Each card row is an independent legal Take/Skip; Take consumes its row before deferred obtain resolves. Whole-list Proceed can leave legal rows unclaimed. Resolve all accepted obtains and any mandatory child before leaving this phase. | Do not call boss-combat rewards “boss chest”; TreasureRoomBoss is a distinct later phase. A skip is valid and does not undo offer history. |
| `BossTreasureEntry` / chest owner | On entry, run the boss-chest opening hooks (`onChestOpen(true)`); construct three sequential boss-tier relic offers from the live boss pool (each consumes its pool entry), then open boss-relic selection. | No ordinary chest size/tier/gold roll, no sapphire-key attachment, no ordinary `onChestOpenAfter`, no Matryoshka extra relic, and Cursed Key's ordinary-chest curse. Do not emit chest gold or force a key. |
| `BossRelicChoice` / choice owner | Take one offered relic or skip; the boss chest itself can also be proceeded past with no pick (record no-pick). Normal take closes selection before ordinary relic arrival/obtain/onEquip. Black Blood replacement is a distinct instant path. | Skip/no-pick is legal; offers are not claims. There is no implicit pick. Do not run onEquip while the choice is still open. |
| `BossRelicChild` / relic + continuation owners | If taken, install exactly once and apply its ordered immediate effects; suspend on any real child selection/reward (e.g. Astrolabe purge/transform, Empty Cage removals, Pandora starter replacement confirmation, Calling Bell curse then its screenless relic offers, Tiny House's upgrade/maxHP/gold/potion/reward rows). Resolve or legally skip every exposed child according to that source's actual policy, resume the owning parent/checkpoint once. Applied grants/removals/rewards are durable at their own checkpoint. | A local flag, screen close, room COMPLETE, queued child, or reward row remaining does not prove global readiness. Preserve empty/automatic/grid size branches and no fabricated choice. Skippable card/relic/potion rows need not be claimed. |
| `NextDungeonProceed` / handoff owner | After permitted choices, next-dungeon Proceed records no-pick where appropriate; a three-relic warning can delay this handler once. Then start fade and mark dungeon beaten. | This still has not constructed TheCity; fade start / beaten flag is not +1. Do not drain unrelated work or discard committed mutations. |
| `DungeonTransition` / main game | Complete the actual main-game transition screen; construct the named successor with the same player object/state. Run shared transition setup, then TheCity/Act2 initialization in defined order below. | An administrative fault quiesces the pool, retains actual committed prefix/provenance, emits no success final/reward/autoreset. It is not death, defeat, retry, rollback or successful profile delivery. |
| `Act2Ready` / run + env | After required map/state initialization, retain the fully initialized successor privately and freeze a detached terminal final of the completed episode; reward +1 exactly once, `terminated=true,truncated=false`. Do not publish an Act2 observation/menu or apply an Act2 action. | Checkpoint is strictly before any Act2 player decision. No Act2 policy play or hidden successor leakage. |

Task-terminal arbitration applies at every phase: actual player death is −1 and
`terminated=true,truncated=false`, even if earlier rewards/keys/Note commits remain
durable; no later transition heal can revive it. A legally available escape is a
nonterminal combat/room outcome: that step receives reward 0, resolves the
source-specific room/combat settlement, and the same episode/run continues with
later legal decisions. It is neither task termination nor a task final/autoreset,
victory, death, or administrative fault. Act1 boss escape is not manufactured
where the source offers no boss escape route; that boss-specific absence does not
remove escape mechanics elsewhere. Skip, no-pick, empty offer, administrative
numeric/resource failure, and post-dispatch fault are also not death. A fault
quiesces and returns no successful final or autoreset; it preserves the committed
prefix and fault boundary without rollback.
Success is the sole `Act2Ready` outcome. A source one-use latch/pool consumption
commits only at its named consumer: boss offer generation consumes all three
offered pool entries even when skipped, boss relic install/child effects commit
once, selected key effects commit at their key-effect update, and visited-room/
encounter queue consumption follows its room-specific departure rule. No later
death erases these mutations.

Boss treasure's three boss relics are not three boss rewards to auto-collect. Boss
card row Take/Skip, boss chest relic Take/Skip/no-pick, and proceeding past chest
are distinct decisions. Boss relic children are resolved only after install. Other
boss-combat reward rows may remain unclaimed if the source allows Proceed; do not
invent mandatory claiming. Resolve all mandatory source child work and the selected
semantic immediate-causal effects before `Act2Ready`; do not require unrelated
queue/list emptiness as a substitute for consumer-specific completion.

### 4.2 Transition effects that persist / reset

The existing player is carried. The new-act constructor does not create a new
Ironclad deck. Shared transition setup advances to Act 2, clears/rebuilds act-local
path, event/shrine/encounter/boss collections and question-room probability state,
resets the shared potion-drop modifier, and requests heal equal to max HP through
normal heal processing. Applicable heal modifiers, caps, dying guards, and prior
relic/history conditions apply; do not write `hp=max_hp` or assume every relic
produces an unconditional all-heal. Reset does not revive a dead player.

| State family | Transition treatment |
|---|---|
| Player HP/maxHP, gold, permanent deck instances/upgrades, relic instances and counters, current potions/belt, acquired keys and durable run effects | Carry exact post-effects values and identities from same player/run; preserve all actually committed mutations and consumed one-use flags. Apply only real transition heal/relic hooks. Do not replenish potions or clear deck. |
| Act number/global floor/path/current room | Advance to Act 2 / next global floor by source transition; clear Act1 path/local current-room ownership and create Act2's empty starting map node. Preserve visited/history facts as named below, not old selectable map nodes. |
| Act-local encounter queues, boss candidate list, ordinary events, six repeatable shrines, question-room probabilities, and Act2-derived offer collections | Initialize Act2-local collections/state as described below. Preserve the run-owned consumed-offer/depletion ledger across the act boundary; rebuilding a derived collection must reapply that ledger. Act1 encounter queue heads are not reused as Act2 queues. |
| Relic pool depletion and committed offer history | Carry the current run-owned consumption history across Act1→Act2, independently of owned relic identities. Every generated offer consumes its source pool entry at the producer commit boundary, including an offered-but-skipped or no-pick boss relic. A newly derived Act2 available collection excludes all already-consumed IDs for that pool role as well as owned-ID exclusions; never restore an unowned consumed relic. |
| Remaining run-wide one-time events | Carry the live remaining list; do not repopulate consumed events. Filter by Act2 eligibility when generating a future event, not by exposing hidden future IDs now. |
| Discovery history | Carry Act1 seen-boss additions and existing independent Act2/Act3 seen sets. The new Act2 boss-selection setup depends on Act2's seen set; it is hidden from policy until legally announced/revealed. Boss lists do not become a public “all seen” feature by accident. |
| Acquired ruby/sapphire/emerald flags | Carry actual owned keys; optional and deferred key effects commit only at their actual consumer. The burning map-node designation is distinct from acquired Emerald. Act1 boss chest has no Sapphire attachment. |
| Run play clock | The selected policy satisfies pure elapsed eligibility; this Act1 contract adds no elapsed-time field, clock/wait-only action, or frame clock. No time value is needed to initialize Act2 at this cutoff. |
| RNG state | Replace any source cursor assumption with reviewed native producer import; do not carry twelve original source streams, source seed offsets, Java shuffle cursors, or card-RNG counter bands as policy. Run/RNG replay provenance follows named import only. |
| Temporary Act1 combat/room/reward/UI work | Must have reached source-specific resolved consumer or be legally abandoned by the actual transition. Never copy queues, screens, temporary card piles, actor timers or stale candidate tokens into Act2. Durable effects already applied remain. No arbitrary private carry-all blob. |

## 5. Act 2 initialization producer contract

The following distinguishes fixed source-informed Act2 setup facts **F** from
concrete native successor contract **E**. These are construction requirements,
not Act2 gameplay mechanics or permission to expose future choices.

### 5.1 Required order

1. Finish boss reward, boss relic selection/no-pick, all mandatory selected relic
   children, and the source-specific boss treasure Proceed. Preserve legal skips.
2. Complete transition-screen boundary, then shared dungeon transition setup on
   the same player: set Act=2; reset act-local path/event/shrine/encounter/boss
   state and question-room probabilities; reset room potion-drop modifier; issue
   normal max-HP heal request through hooks. Do not recreate deck/player.
3. Apply Act2 ruleset/chance parameters, then initialize Act2 encounter pools/
   queues and boss candidates, ordinary event and six recurring shrine pools, and
   Act2 card/potion/relic derived offer collections. Carry the run-owned relic
   consumption/offer-history ledger and reapply its exclusions to any derived relic
   collection; also keep the one-time event remainder. Do not reset run histories
   merely because a collection is reconstructed.
4. Generate/validate the Act2 map and burning Elite designation through the
   accepted map producer. The visible map contains only map facts permitted at
   this boundary; future encounters, boss order, hidden room/event identities,
   offer pools and RNG state remain private.
5. Create the Act2 empty entry node/current position and set its initial state.
   Do not enter the node, roll its encounter/event, perform its room-entry hooks,
   construct/publish an Act2 action menu, or select an action. Mark `Act2Ready`
   only after these state components are coherent.

**Selected native constructor order** is the numbered sequence above: shared
transition setup; Act2 ruleset/chance parameters; encounter/boss, event/shrine,
card/potion/relic producers (with carried depletion applied); map validation and
burning marker; empty entry node; then readiness validation. The dependencies are
that Act2 context exists before its producers, the map producer receives Act2
parameters, and the entry node refers to the validated map. Producer-internal
order not required by those dependencies is private and must not affect policy-visible
state.

**Source ordering is a separate static fact, not the selected RNG schedule:** in
the inspected `AbstractDungeon` constructor, shared transition setup is followed
by `generateMonsters`, `initializeBoss`, `initializeEventList`,
`initializeShrineList`, `initializeCardPools`, and `initializePotions`; control then
returns to `TheCity`'s constructor body, which initializes level-specific chances,
generates the map, and installs the empty node. This method/call ordering does not
prove that every source pool initializer is invoked on an Act transition, and it
does not override selected native depletion carry. Native independent RNG requires
no host-cursor, source-call-order, or cross-system draw parity.

### 5.2 Act 2 initial state domains (source-informed)

| Owner/state | Required initialized domain/value | Visibility at Act2Ready |
|---|---|---|
| Act/floor/entry | `act=2`; global floor `17` at the first Act2 entry; fresh path; Act2 start node exists with no Act2 room visited. Native floor counter retains the checked source integer domain. | Internal successor only at this terminal; no Act2 fields are inserted into the Act1 policy final. |
| Map | TheCity topology, room assignments, edges, and valid burning-Elite marker generated as one Act2 map; every exposed map satisfies the selected valid-map predicate (including >=1 Elite) by deterministic reject/regenerate before exposure. New map is not a patched Act1 map. | Retained in private successor; not projected into the Act1 final. Burning marker is not acquired Emerald key. No hidden future encounter/event/boss result. |
| Encounter queues and Act2 boss order | Weak and strong lists: two weak entries, separately filtered first strong, then twelve more strong; ten elite entries. Weak/strong/elite candidates and weights are the E07 §2 Act2 pool rows. For boss selection, first-unseen priority is **Champ → Automaton → Collector**; if none is unseen, native policy shuffles the E07-listed candidates `(Automaton, Collector, Champ)` and BossRoom consumes the ordered head. E07's forced unseen identity is duplicated in the internal boss list, not a second fight. | Queue order/cursors and future boss candidates stay private. All-seen ordering is a selected unbiased shuffle, but its RNG binding is **blocked**: accepted registry has `setup.boss_order` only at `(act=1)` and `encounter.kind` at room scope `(act,floor,room_ordinal,enemy_ordinal)`; neither is an accepted Act2 constructor-order site. Required controller amendment: one registry row under existing role `run.encounter`, with a controller-assigned stable literal site ID (none invented here), scope `(act=2)`, invocation `0`, operation unbiased shuffle over E07's listed sequence `(Automaton, Collector, Champ)`, and output/commit the ordered candidate list for later head consumption; no draw in first-unseen branch. Until accepted/routed, all-seen Act2 readiness has an explicit unresolved site binding. No Act2 field is projected into Act1 final. |
| Encounter history | Preserve Act1 visited rooms and encounter history; initialize Act2's own encounter queue/history and first-strong exclusion from actual last Act2 weak entry (not Act1 last encounter). Ordinary append rejects last two; elite append rejects only immediate prior; first strong uses source-specific exclusions, not ordinary rule. Room departure consumes its Act2 queue head, event fight does not. | Private successor history; do not publish hidden queue entries or underlying queue cursor. |
| Events/shrines | Reinitialize the 13 Act2 ordinary event IDs: Addict; Back to Basics; Beggar; Colosseum; Cursed Tome; Drug Dealer; Forgotten Altar; Ghosts; Masked Bandits; Nest; The Library; The Mausoleum; Vampires. Reinitialize six repeatable shrines: Match and Keep, Wheel of Change, Golden Shrine, Transmogrifier, Purifier, Upgrade Shrine. Combine only the remaining one-time-event list under eligibility at a future event-selection site. | All pools private. Do not expose event identities/order or eligibility for hidden future rooms. |
| Card reward state | Carry the live ordinary-card rarity-pity integer `b` across the Act boundary; new-run initial `b=5`, Rare resets to5, Common sets `max(-40,b-1)`, Uncommon leaves it unchanged. Boss forced-Rare generation still applies the caller's listed pity transition; boss rows already generated are not copied. Card rarity and identity are distinct named sites. | Current unoffered rewards absent; pity and site counters private. Only actually generated current offers may be shown. |
| Shop/economy | Carry `paid_purge_count` and current gold; purge price is `75+25*paid_purge_count` gold under the accepted deterministic E26 expression. Reset Act2-local `shop_visit_ordinal=0`; no active shop/phase/restock inventory exists at handoff. On each actual Act2 shop, site scope is `(act=2,shop_visit,phase,slot)`; initial stock and each restock use their accepted `run.shop` sites. | No stock/prices/offers before shop entry; visit, phase, slot, inventory exclusions, and random state private. Shop/relic/potion/card IDs remain hidden until legally offered. |
| Relic pools and depletion | For each role `Common,Uncommon,Rare,Shop,Boss`, carry ordered-consumption history across acts. At Act2 pool construction, stable-canonical full pre-removal members are shuffled first by the accepted `run.relic/relic.pool.init` site, then current owned IDs and all previously consumed IDs for that role are removed while preserving surviving order. Every selected candidate removed by a rejection predicate is committed consumed at that selector checkpoint; an accepted/generated offer is consumed regardless of later take/skip/no-pick. Pool choice itself is front/end consumption, not another identity draw. | Exact counts, site scope, shuffle and selector transaction law are in §6.5; literal member rows and all caller predicates remain accepted catalog/content joins. No consumed unowned relic can reappear; pools/history hidden. |
| Potion state | Carry exact current belt/capacity and `PotionAppearanceOffset` (run-start value0; update +10 on failed eligible appearance and −10 on success; no clamp/reset; claim/skip/use do not change it). Separately reset `AbstractRoom.blizzardPotionMod=0` at act transition. Keep Sozu and current relic/potion counters in PlayerCore; do not refill the belt. | Belt visible normally; offset, modifier, pool/catalog and future proposal private. Named `run.potion` sites and caller scopes are bound in §6.5. |
| Question-room state | Reset Act2 binary32 history ramps to `F(.10),F(.03),F(.02)`; initialize `question_room_roll_ordinal=0`. The corresponding no-override 24-bit pushforward is Monster/Shop/Treasure/Event `1,677,722/503,316/335,545/14,260,633` over `16,777,216`; these are initial derived masses, not mutable ramp fields. Keep Tiny Chest/Juzu ownership and Tiny Chest counter in carried relic/player state. Full E25 transitions are in §6.5. | Ramps, ordinals, overrides and future rooms hidden. No reset on each `?` roll; update fields only at the exact E25 checkpoint. |
| Room probabilities | Act2 map parameters from accepted map producer: exact rational Shop `5/100`, Rest `12/100`, Elite `8/100`, Event `22/100`, Monster residual; fixed-row exceptions are not ordinary bucket probabilities. Chest size 50/33/17; ordinary relic tier 50/33/17; colorless rare parameter .3; A0 card-upgrade .25 remain their separate producers. | Only realized Act2 map facts may be visible after legal Act2 observation; all parameters/candidates stay private at this Act1 terminal. |
| One-time event history | Continue only the remaining source run-wide one-time event IDs; already consumed remain absent. Act2-specific eligible additions (including Designer, Duplicator, Knowing Skull, N'loth) are evaluated at their proper future event generation. | Hidden pool not exposed; revealed/history facts only. |
| Native random state | Carry exact accepted RNG contract/seed-adapter/run identity/root/partition and current role-site states, including `run.map.draft`, `run.encounter`, `run.relic`, `run.card.acquire`, `run.event`, `run.potion`, `run.shop`, and `run.handoff` scopes as joined in §6.5. Act2 starts map-draft and question-room roll ordinals at0; rejected complete map drafts increment only their checked map ordinal. | Root/seed adapter/run key, keys, cursors, counters, hidden samples, rejected drafts, and diagnostics remain private; no fields projected into the Act1 final. |
| Current phase | `Act2Ready`, no current Act2 combat/event/shop/reward child, no pending Act1 continuation, and no Act2 menu/candidate constructed or published. | Act1 task final is terminal with zero candidates; successor is initialized privately and not exposed as policy data. See §6. |

Act2 source encounter pools and numerical room parameters above are retained facts
from the merged E07 report and baseline §7/§17. They are not source RNG parity
obligations. The source's two weak + first strong + twelve additional strong list
is fifteen regular entries, not fourteen. The one-time events are not part of the
13 ordinary-event count. Boss-seen fully experienced branch shuffles its candidate
list in source; native selection remains under the accepted independent RNG policy.

## 6. Concrete owned records and delivery

### 6.1 `TransitionCheckpointV1`

This is a private run-owned checkpoint record, not a public profile and not a
generic source-state snapshot:

| Field | Domain / checkpoint owner | Reset or visibility |
|---|---|---|
| `phase` | `BossCombat`, `BossCombatRewards`, `BossTreasureEntry`, `BossRelicChoice`, `BossRelicChild`, `NextDungeonProceed`, `DungeonTransition`, `Act2Initialize`, `Act2Ready`, `Dead`, `Faulted` | Monotone only along allowed transitions; one terminal outcome. Private phase is not a feature except current phase's legal public projection. |
| `run_key` | stable stream identity + episode ordinal, no slot/worker | Never changes in this episode; hidden. |
| `act1_boss_id`, `boss_discovery_committed` | exact Act1 boss identity + boolean | Commit only at prebattle discovery hook. Retain even if later death. Internal/history projection obeys legitimate visibility. |
| `reward_rows` | owned source-qualified reward identities/status and source parent | Built once; each claim/skip commits at its source checkpoint; clears/retains under the exact consumer, not list-empty heuristic. Hidden absent legal current screen. |
| `boss_relic_offers` | three ordered consumed boss-pool identities and claim/skip/no-pick state | Hidden until boss choice; offered members remain consumed on skip. No alias to acquired relic identities. |
| `continuation` | parent kind/identity, source ID, captured args, ordered selections, resume checkpoint, child status and once-only keys | Acyclic source-qualified ownership; clear on actual resume/completion. Never copied to successor after resolved handoff. |
| `durable_mutation_prefix` | player/run-owned actual values: HP/maxHP/gold/deck/relics/counters/potions/keys/histories | Applied in place; no rollback on later death/fault; never reconstructed from rewards/logs. |
| `handoff_effects` | actual source-qualified transitions/heal child IDs and resolved status | Every applicable effect resolves once; no all-heal assignment. Clear after consumer. |
| `transition_state` | act/floor/room, Act2 map/queues/events/shrines/reward/shop/relic/potion pools and histories listed §5 | Reset only the named Act1-local families; carry only named run-owned fields. Hidden fields stay private. |
| `terminal` | `None`, `Death`, `Act1CompleteAct2Ready`, `AdministrativeFault` | Set once. Death gets −1; successful ready gets +1; fault and all other non-death endings get 0 and no success final. |

Fields use checked source/native domains; transport must not narrow counters or
introduce a generic arbitrary cap. Completion checkpoint and task-captured Act1
final are detached before any vector replacement. `TransitionCheckpointV1` is not
an importable restoration API; replay reconstructs from start and accepted choices.

### 6.2 `SuccessorStateV1`

The owned successor is an explicit product of these components, not “the entire
dungeon object,” an opaque private blob, or an Act1 object graph:

1. `PlayerCore`: character/rules identity, current/max HP, gold, persistent deck
   entries `(instance_id, canonical_card_id, upgrade_count, actual persistent
   misc only where that card law owns it, bottle flags)`, relic entries
   `(canonical_relic_id,counter,used/charge state)`, potion capacity and each live
   slot `(occupied,potion_id,potency)`, and acquired key bits. Other profile-wide
   history and pool counters live only in the named `Progression` or `Act2Pools`
   fields below; no unenumerated player counter bag is carried.
2. `Progression`: act=2, next global floor, prior visited-room/reward/Neow/boss
   history, independent Act1/Act2/Act3 boss-seen sets, remaining one-time event
   identities, and current entry-node reference.
3. `Act2Map`: topology/typed rooms/edges/burning marker/current empty node and
   marker-to-buff pending status; valid map producer identity/version. No map
   generation input seed or rejected drafts in observations.
4. `Act2Pools`: encounter queue/history, three boss candidates/selection history,
   ordinary event pool, six shrine pool, card reward/pity history, relic tier
   pools/depletion, potion state, shop stock not yet generated (empty), question
   room state, room probability inputs, and stable IDs. Future entries are
   private and pools obey their own ownership.
5. `NativeRandomProvenance`: accepted schema/site/domain versions and run-owned
   opaque native random states required to continue exact deterministic replay.
   Exact state layout is imported from reviewed RNG lane, not defined as source
   cursors here and never exposed as policy information.
6. `PhaseAndWork`: phase `Act2Ready`, current location at the legal empty entry
   node, no Act1 continuation/action/screen, no Act2 room event/encounter entered,
   and no Act2 menu/candidate constructed. The first Act2 action has not happened.

Any content identity, counter, instance ID, map field, pool, history, or RNG state
not in these named components does not cross the handoff by default. Add a field
only with a source-backed owner and initialization/reset disposition. Player
resources are not duplicated in combat overlay; stale transient objects do not
survive. `SuccessorStateV1` is owned by the Rust run and inaccessible as a policy
observation except for its permitted public projection.

### 6.3 Finite construction, carry, and reset matrix

This matrix makes the successor constructor auditable without claiming opaque
source pools or rejected RNG proposals are public state. `Static` is immutable
ruleset/content data, not a per-run depleted pool; `Rebuild` creates/reset the
successor's run-owned instance; `Carry` preserves the same run-owned value/identity.
Source anchors are static inspection of the pinned archive's
`AbstractDungeon`/`TheCity` initialization and transition-setup bytecode. No class
initializer or game runtime was executed.

| Component / exact fields | Owner / input | Transition disposition | Initial value/domain at `Act2Ready` | Visibility |
|---|---|---|---|---|
| Player core: character/rules, HP/maxHP, gold, deck identities/card IDs/upgrades/misc/bottles, relic identities/counters/charges, potion slots/capacity, keys | Existing player/run | Carry post-boss committed values and same player identity; only selected normal transition healing mutates resources. | Exact post-effect values; never synthesize deck/relic/potion replacement. | Successor private. |
| Progression: act, global floor, visited/reward/Neow history, three boss-seen sets, remaining one-time events, entry reference | Run owner | Carry run-owned histories; advance act/floor; install Act2 entry. Do not clear history on dungeon replacement. | `act=2`, first Act2 entry floor 17; prior committed history plus Act1 commits. | Private. |
| Transition scratch: `pathX`, `pathY`, event/shrine lists, normal/elite encounter lists, boss list | Shared transition owner | Clear old lists and paths; TheCity/Act2 producer repopulates. | Empty during transition gap, then Act2-owned values before ready. | Private. |
| Question history: `monster_ramp`, `shop_ramp`, `treasure_ramp`; question roll ordinal | E25 event producer + accepted `run.event` | Initialize each act's history once to binary32 `F(.10/.03/.02)` at transition setup; do not reset on each room roll. Increment question-room ordinal once per actual `?` roll. Apply the exact snapshot→24-bit cell draw→ordered writes→Tiny Chest→Juzu→history updates in §6.5. | Binary32 field semantics; initial derived grid tuple in §5.2. Tiny Chest counter belongs to carried relic instance and is not reset at Act boundary. | Ramps/snapshot/ordinal/private RNG hidden; no future room exposure. |
| `AbstractRoom.blizzardPotionMod` | Shared transition owner | Explicitly set integer `0` at shared transition setup; does not refill player belt. | `0`. | Modifier private; owned belt only under normal observation. |
| Source-only `TheCity` room-chance constructor tuple `(Shop,Rest,Treasure,Event,Elite)` | Static source provenance (`TheCity.initializeLevelSpecificChances`) | Record source fact only: binary32 `(.05,.12,0,.22,.08)`; these labels are semantic tuple positions, not asserted Java field identifiers, and values are **not stored as selected native map sampler fields**. | Source constructor fact; no native successor state member. | Not exposed or used as the native map quota vector. |
| Selected native Act2 map quota vector | Accepted map ruleset/`run.map.draft` producer | Use exact rationals `Shop=5/100, Rest=12/100, Elite=8/100, Event=22/100`, Monster residual; derive integer quotas from actual Act2 graph counts per accepted map §4. No ordinary Treasure bucket (fixed-row chest remains). | Ruleset-owned sampler constants, distinct from source binary32 tuple above. | Parameters/draft private; only legally revealed map facts later. |
| Non-map producer parameters | Chest, relic, card and potion producer owners respectively | Keep chest `50/33/17`, ordinary relic tier `50/33/17`, colorless rare `.3`, A0 card upgrade `.25` (`ascension<12`) on their named producer semantics; do not pass them into the map sampler. | Separate producer parameters/checkpoints, not `TheCity` map quota fields. | Hidden until the corresponding legal offer/room result. |
| Normal/elite queues and boss candidates | Encounter producer + carried boss history | Clear Act1 lists; Act2 creates two weak, first strong, twelve more strong, ten elite entries; first-strong exclusions derive from actual Act2 weak result. First-unseen boss priority Champ→Automaton→Collector; all-seen ordering site is pending registry amendment described in §5.2. | E07 candidate/weight definitions §5.2; queue and candidate order private. | Private; do not project a hidden future boss. |
| Ordinary events, repeatable shrines, remaining one-time events | Event producer / progression | Rebuild ordinary and shrine lists; carry live remaining one-time set without restoring consumed IDs; evaluate eligibility only at future selection. | 13 ordinary and six shrine IDs §5.2 plus exact remaining one-time history. | Private including order/future eligible subset. |
| Card pools: common/uncommon/rare/colorless/curse; static source definitions | Card-content producer + reward-history owner | Clear/rebuild run-local rarity groups from character/content definitions; do not carry Act1 group depletion. Carry ordinary rarity-pity integer `b` unchanged across acts; initialize to5 only for a new run. | Five groups exist; actual ID membership awaits accepted catalog. Pity `b∈[-40,5]`; rare->5, common->`max(-40,b-1)`, uncommon unchanged, once per generated rarity result before identity retry. | Pools/history hidden; only generated offers shown. |
| Legacy source card-rarity fields `cardBlizzStartOffset`, `cardBlizzRandomizer`, `cardBlizzGrowth`, `cardBlizzMaxOffset` | Source provenance only; selected native reward producer owns semantic `b` | Static bytecode shows `dungeonTransitionSetup` does not write these fields and full-run reset sets randomizer from start offset. They are not the selected native pity state; successor carries the accepted producer's `b`, not an assumed Java field mirror. | The four source fields are signed Java ints; native consumes the accepted `b` contract above. No source parity or numeric member law is inferred. | Private; no pity/cursor in final observation. |
| Potion appearance and carried resources | Potion producer + carried player | Carry `PotionAppearanceOffset` unchanged across act; reset distinct room modifier `AbstractRoom.blizzardPotionMod=0`; retain belt/capacity and Sozu/relic state. | Run-start offset0; success subtract10, failure add10; no clamp/reset; claim/skip/use do not change it. Named potion sites/scopes in §6.5. | Belt visible normally; offset/modifier/catalog/site state hidden. |
| Relic pool state: five roles; current order; rejection/offer ledger; owned exclusions | Run-owned relic history + accepted `run.relic` producer + player relics | Carry consumption ledger. At Act2 init shuffle canonical full pre-removal lists first, then remove owned and previously consumed IDs preserving surviving order. Each selector rejection consumes at its caller checkpoint; each offered identity consumes regardless of take/skip/no-pick. | Pre-removal counts Common/Uncommon/Rare/Shop/Boss =33/30/28/17/22. Front/end selector and caller-specific rejection/fallback law bind to accepted RNG §3.4. Exact literal member/predicate joins remain pending catalog/content route. | Pool arrays/ledger/order hidden; acquired relics visible normally. |
| Shop visit, paid-purge history, stock/price | Shop/economy producer + progression | Carry `paid_purge_count` and gold; Act2 `shop_visit_ordinal=0`. No active shop/phase/stock/restock at handoff. Generate inventory only at actual shop entry. | Purge `75+25*paid_purge_count`; accepted deterministic E26 price expression at its checkpoint. Restock ordinal visit-local; exact increment/commit timing remains open in §7. | No stock/price/offers before legal visit; all visit/phase/history private. |
| Act2 map, draft, burning marker, entry node | Accepted map producer | Build `MapDraftInput(mode=Act2Handoff,act=2,...)`; generate/reject whole drafts under exact role/site counters, retain only `ValidMap`; then install `(0,-1)` `EmptyRoom` without entering it. | Typed dimensions, fields, room codes, counters and validator bound in §6.5. | Whole successor map private at Act1 terminal; no seed/draft/cursor in final. |
| Native random provenance | Accepted RNG producer | Carry exact run root/seed adapter/partition/RunKey and required `NativeRngState` values plus current semantic ordinals; exact Act2 roles/sites/scopes in §6.5. Never source Java objects/cursors. | Exact record fields and map/relic counter law in §6.5; unlisted producer state remains an explicit gap. | Root/seed/key/counters/site/scopes/rejected proposals private. |
| Phase/work | Handoff owner | Ready only after required child and every named producer validator succeeds. | `Act2Ready`; no Act1 continuation/screen/action, Act2 room entry/menu/candidate/decision. | Act1 terminal projection has zero candidates and no successor fields. |

Construction has a selected native order and a distinct source order as stated in
§5.1. Under the native order, Act2 parameters and pool producers finish before the
validated map and empty entry are installed; `Act2Ready` follows only after all
components validate. Static inspection establishes the source superclass and
subclass call sequence, not that each source initializer is a transition reset
contract. Source `initializeCardPools` rebuilds run-local rarity groups, not
immutable static card definitions. Regardless of any derived relic collection
rebuild, the selected native run-owned consumed-offer ledger carries across the act
boundary and is reapplied before offers. The potion modifier's actual transition
reset value is zero; other potion-producer state is a separate named import.

### 6.4 Concrete child and transition completion protocol

This is the finite handoff-side interface. Content callback/site/effect values
remain pending execution import; this protocol does not reimplement that lane.

`ParentChildCheckpointV1` has exactly: `checkpoint_id`, `parent_id`, nullable
`child_id`, `source_owner` (boss reward, boss relic, relic child, transition
effect), `kind` (choice, automatic effect, reward row, nested child, transition
effect), `requiredness` (mandatory-to-resume or optional), `status`, `commit_id`,
nullable `result_code`, nullable `resume_checkpoint_id`, and `once_key`. IDs are
episode-local, not content IDs. Statuses are `Pending`, `Offered`, `Committed`,
`Skipped`, `NoChoice`, `Resolved`, `Resumed`, or terminal `Faulted`. `Committed`
requires a commit ID; `Skipped` is legal only for optional/skippable work;
`NoChoice` requires producer declaration; `Resolved` requires its producer result
and every mandatory descendant resolved; `Resumed` requires exactly one parent
resume at the named checkpoint. `once_key` is unique per effect/commit/resume;
duplicate application/resume faults. Parent/child links are acyclic, and nested
work names the suspended parent checkpoint.

`ResolvedTransitionEffectsV1` has ordered `effects`, each with `effect_id`,
`once_key`, `consumer_site`, `requiredness`, `status`, `commit_id`, `result_code`,
and optional `child_checkpoint_id`, plus `next_act`, `successor_build_id`, and
`producer_revision`. Statuses are `NotApplicable`, `Pending`, `Committed`,
`Resolved`, `Faulted`. Applicable effects transition Pending -> Committed ->
Resolved exactly once; nonapplicable effects are explicitly `NotApplicable`; child
work must resolve before its effect resolves. Exact site/effect IDs, code set and
payload/result codes are `PendingImport(execution producer revision)`, not
invented. The accepted producer must provide an exhaustive finite set including
transition heal/relic hooks and selected immediate causal effects; unknown
applicable work blocks completion.

The handoff may enter `DungeonTransition` only when each boss reward row has
explicit `Committed`, `Skipped`, or producer-authorized `NoChoice`; obtains'
children resolve; the chest has exactly three offered/consumed relic identities
and explicit take/skip/no-pick; selected relic mandatory descendants resolve; and
each suspended parent resumes exactly once. Optional descendants may be skipped;
offered rows are not implicitly claimed. `Act2Ready` is true iff simultaneously:
(a) no actual death or administrative fault has won task-terminal arbitration and
the run remains live (a prior nonterminal room/combat escape does not terminate or
invalidate the episode); (b) boss reward/chest
predicates above hold; (c) mandatory child work is resolved and each suspended
parent resumed once; (d) each applicable transition effect is resolved exactly
once and all others are explicitly nonapplicable; (e) destination is Act2 with
same carried player identity; (f) map, empty node, queues, event/shrine/card/
potion/relic/shop/history and RNG components pass their producer validators; and
(g) no Act1 screen/continuation/action/child/effect remains, and no Act2 room
entry/menu/action has begun. Collection emptiness is neither necessary nor
sufficient: optional rows can remain as resolved skip provenance; an empty
collection missing a required result is not ready.

Task-terminal outcomes are exclusive and set once: `Death` => -1, terminated;
`Act1CompleteAct2Ready` => +1, terminated; `AdministrativeFault` => 0, quiesced,
and no success final. `None` means the task episode remains live. A room/combat
escape is not in this episode terminal domain: it resolves at its producer's
room/combat boundary with step reward 0, no task final, no task autoreset, and run
continuation to later legal decisions. Do not create a task-terminal escape phase
or outcome. Fault preserves committed prefix and does not fabricate a successor.
This defines consumer record shape/predicate, not evidence that the execution
producer has supplied its exact site/effect rows.

### 6.5 Accepted RNG/map and Act2 semantic-state joins

This subsection consumes only the accepted bounded producer in main
`35808a1b61a79566ec5a18cd50fea85ae60eadc0`, document SHA-256
`e24d7f0667ccf6397a9ee84e5a1ccc72d40294f3c5cb9269f98b4f61d22447c1`.
It is a consumer binding, not source-runtime evidence. It does not import source
Java RNG objects, source draw correlations, or unreviewed content-member rows.

**Typed records and exact Act2 map projection.** The successor stores the accepted
`RunRoot32[32]`, seed-adapter ID (`spire-lab-act1-seed-sha256-candidate1`,
`spire-lab-act1-root32-candidate1`, or `spire-lab-act1-entropy32-candidate1`),
RNG contract ID
`spire-lab-act1-rng-hmac-sha256-v1`, partition (`train|selection|final|test`),
canonical RunKey `a1:<32-lowercase-hex-stream-identity>:<canonical-u64-episode>`;
and the immutable `NativeRngState` values needed for subsequent replay. Each
`NativeRngState` is `(contract_id,partition,run_id,role_id,site_id,scope,key[32],
next_word:u128)` with drawable cursor `0..2^64−1` and explicit exhausted endpoint
`2^64`; all IDs are exact printable non-space ASCII (1..255 bytes). Role/site
identity and scope are fields, not inferred from array position. `MapDraftInput`
is `(mode:u8,act:u8,ruleset_id,run_id,draft_ordinal:u64,emerald_key_owned:bool,
act4_enabled:bool)`; `MapMode` encodes `1=Act1Start`, `2=Act2Handoff`; for this
handoff mode is2, act is2, and ruleset is `spire-lab-act2-map-candidate1`. Its
dimensions are 7 columns, floors 0..14, plus
boss `(3,15)`. `MapNode=(x:u8,y:u8,room_kind:u8,visited:bool,reachable:bool,
burning_eligible:bool)`; room codes `1 Monster,2 Elite,3 Shop,4 Rest,5 Event,
6 Treasure,7 Boss`. `MapEdge` has coordinate endpoints, unique and sorted by
source then destination. `ValidMap=(mode,act,nodes,edges,fixed_rooms,burning_node?,
start_nodes,boss_node,validator_version:u16=1)` with canonical sorted nodes/edges,
fixed-row entries, sorted starts, and boss `(3,15)`. No RNG key, draft ordinal,
rejected draft, or transcript is in `ValidMap`.
`MapDraft` is the private candidate form carrying the candidate graph/room fields
plus `draft_ordinal:u64` and relevant per-site `NativeRngState` handles; only
successful full validation converts it to `ValidMap`.

**Act2 map initialization transaction.** Begin `draft_ordinal=0`; independently
derive role `run.map.draft` site streams for `map.topology`, `map.room_bucket`,
and, only when eligible, `map.burning_elite`, each scoped `(act=2,draft_ordinal)`
plus the site-specific subscopes in the accepted registry. Generate six distinct
starts, route topology, exact-rational room quotas/assignment, fixed rows, and
burning marker as specified by accepted map §4. Validate the complete draft,
including ≥1 assigned Elite, fixed rows, branch/merge, reachability, edge/type
constraints and canonical IDs. Invalid drafts are discarded wholly; checked
u64-increment the ordinal and regenerate. No patching, cap, or partial-field carry.
Retain only the first `ValidMap`; then create current node `(0,-1,EmptyRoom)`.
The Act2 successor constructor order is therefore: establish accepted run identity
and role derivation context; initialize/reset named Act2 semantic state below;
construct and validate `MapDraft` until `ValidMap`; install empty node; validate
whole `Act2Ready`. The map contract says no first Act2 decision/room entry occurs.

**Concrete producer-to-successor state joins.** `Carry` means same run-owned
semantic value, `Reset` is an explicit Act2-local initial value, `Derive` means
rebuild only the named collection while preserving history. These states are not
all encoded in RNG cursors; mechanics state and RNG state are distinct:

| Accepted producer/state | Exact successor fields and operation | Initial/carry law and checkpoint | Visibility / unbound remainder |
|---|---|---|---|
| `run.map.draft` | Map input/output types and fields above; draft ordinal; `NativeRngState` per `run.map.draft` role/site. | Act2 mode, act2 ruleset, ordinal0; increment only for a wholly rejected complete draft. `map.topology`, `map.room_bucket`, `map.burning_elite` have distinct site IDs and draft scope. | Entire map private at Act1 terminal; later map visibility follows game rules. No draft/rejection/cursor exposed. |
| `run.encounter` | Act2 encounter queue heads/contents and boss candidate/order state; per-site `NativeRngState` for `encounter.kind`, `encounter.composition`, `encounter.enemy_hp`, `encounter.opening`; scope `(act=2,floor,room_ordinal,enemy_ordinal)`. | Initialize the private Act2 queue from E07/§5.2: two weak, separately filtered first strong, twelve further strong, ten elite; use its Act2 candidates/weights/history rules. First-strong exclusion uses actual last Act2 weak result, never Act1. Room departure consumes queue head; Event combat does not. Boss first-unseen order is Champ→Automaton→Collector with no draw and forced singleton duplication. For all-seen, selected operation is unbiased shuffle of E07's listed sequence `(Automaton,Collector,Champ)` but there is no accepted Act2 initialization site; do not use Act1-only `setup.boss_order` or room-scoped `encounter.kind`. The precise controller registry amendment is in §5.2; readiness remains blocked on accepting/routing that row. | Queues, future boss/encounter candidates, hidden selections and streams private. E07 facts are joined; all-seen site binding is explicitly absent. No locally invented site ID. |
| `run.relic` | Per-role ordered pool arrays `Common,Uncommon,Rare,Shop,Boss`; run-owned consumed-ID history per role; separate RNG state for `relic.pool.init`, `relic.tier`, `relic.identity`, `relic.end`, `relic.screenless`, `relic.noncampfire`, `relic.neow`, `relic.shop`. | Pre-removal list counts 33/30/28/17/22. Canonical stable IDs -> shuffle full list -> remove current owned IDs and carried consumed IDs preserving order. A candidate removed by caller rejection commits to consumed history; generated offer consumes irrespective of take/skip/no-pick. Front/end selection consumes from array, not identity RNG. No-pick never restores. Each call uses accepted `(act,visit,site,invocation)` identity and caller-selected exact site/scope. | Arrays/history/streams hidden. Literal relic ID rows and callers' content-specific `canSpawn`/role-membership predicates await accepted catalog/content joins; counts and selection law are bound. |
| `run.card.acquire` | Ordinary card-pity semantic integer `b`; rarity and identity are separate sites, with caller-qualified pairs for ordinary monster, elite, boss, colorless reward helper, Prayer Wheel second offer, Dream Catcher Rest offer, each Orrery offer, fifth-screen auto-card row, Neow-discarded auto-card row, and Tiny House reward-screen row. | Carry `b` across Act1→Act2. New-run `b=5`, range `[-40,5]`; generated rarity result applies once before identity retries: Rare sets5, Common `max(-40,b-1)`, Uncommon unchanged. Ordinary/elite/other thresholds and boss-forced-rare behavior follow accepted RNG §2/§3; identity retry stays at chosen rarity and excludes IDs already in same offer. Site scope `(act,source_event,site,invocation,slot)`. | Pity/site state hidden. Literal supported card IDs, aliases, upgrade laws remain accepted `.44/.46` I1 gap. Caller-specific counts/eligibility and exact candidate rows use those accepted catalog/content joins. |
| `run.event` / `QuestionRoomState` | Separate binary32 ramp fields `(monster_ramp,shop_ramp,treasure_ramp)`; `question_room_roll_ordinal:u64`; source-owned Tiny Chest counter remains in its carried relic instance; Juzu/Tiny Chest ownership read from carried relics. `NativeRngState` sites include `event.room.probability` and `event.room.override`; question roll ordinal maps to accepted `event_visit` scope. | Act2 initializes ramps to `F(.10),F(.03),F(.02)` and roll ordinal0. For each actual `?` entry, snapshot prior ramp values; compute `m=trunc(F(monster_ramp×100))`, `s=0` iff the room being left is ShopRoom, otherwise `trunc(F(shop_ramp×100))`, and `t=trunc(F(treasure_ramp×100))`. Draw `k∈[0,2^24)` once at `event.room.probability`; `cell=trunc(F(100×(k/2^24)))`. E25 later writes are ordered: start Event; A=`[0,min(100,m))` writes Monster; B=`[min(99,m),min(100,m+s))` writes Shop; C=`[min(99,m+s),min(100,m+s+t))` writes Treasure. Thus C wins B and B wins A, including cell-99 saturation. Increment Tiny Chest's carried counter once per roll; exactly4 resets it to0 and forces Treasure after the base draw. Then Juzu converts any remaining selected Monster to Event; scripted fights are unaffected. Commit that final room result once; an actual EventRoom's branch/ID are later separate event sites. Update ramps after overrides: Monster resets to `F(.10)` for a primitive Monster not replaced by Tiny Chest (including Juzu conversion), otherwise `F(old+F(.10))`; Shop resets to `F(.03)` iff final room is Shop, otherwise `F(old+F(.03))` (including a Shop-suppressed roll while leaving Shop); Treasure resets to `F(.02)` iff final room is Treasure (including Tiny Chest), otherwise `F(old+F(.02))`. Increment question-room roll ordinal once; no invalid action triggers another roll. `F` is IEEE binary32 rounding; the source tuple is separate from the initial 24-bit pushforward `(1_677_722,503_316,335_545,14_260_633)`. | Ramp fields/counters, snapshots, stream state and future outcomes hidden; carried relic counter is not duplicated/reset at Act boundary. E25 §3 supplies the complete mutation and precedence law, now joined here. |
| `run.potion` | Semantic appearance offset (integer accumulator; native width remains unbound), current belt/capacity in carried PlayerCore, distinct `AbstractRoom.blizzardPotionMod` room field reset to0; sites `potion.reward.appearance`, `potion.reward.identity`, `potion.shop.identity`, `potion.event.flat`, `potion.brew.combat`, `potion.brew.outside`, `potion.use.alchemize`, `potion.use.entropic_brew`. | Carry appearance offset unchanged across Act boundary; run starts0. Combat-drop eligible chance is `40+offset` for Elite/Event combat, ordinary and boss only when no escape, otherwise base0; White Beast overrides100, then ≥4 existing room reward rows overrides0. Draw integer0..99, success iff below chance; success adds offer and subtracts10; failure adds10. No clamp/reset; claim/skip/use/identity do not mutate offset. Potion identity/caller sites have scope `(act,room_or_visit,source,invocation,slot)`. Caller laws remain distinct: combat Brew consumes/discards flat proposal then retains chosen rarity rejecting Fruit Juice/wrong rarity; outside Brew Sozu check precedes proposals; Alchemize samples before Sozu/full-belt obtain; caller-specific duplicate/full-slot rules follow accepted §2. | Belt normally visible; offset, modifier, catalogs and future proposals hidden. Accepted source provides 33-ID catalog, rarity masses and caller laws. **Still open:** native field width and exact callback→field commit mapping; caller-to-checkpoint wiring must preserve the accepted predicates above. Do not substitute belt state for offset. |
| `run.shop` | `paid_purge_count` (nonnegative checked count; concrete native width unbound), current gold, Act2 `shop_visit_ordinal=0`, private phase/restock ordinal and stock/inventory exclusions; distinct `shop.card.stock`, `shop.relic.stock`, `shop.potion.stock`, `shop.price`, `shop.discount`, `shop.restock`, `shop.purge` states. | Carry paid purge count and gold; no active shop/phase/stock at readiness. Purge price `75+25*paid_purge_count`. At actual visit use semantic scope `(act,shop_visit,phase,slot)`; each restock has its own ordinal/identity. Initial/restock stock uses accepted typed card/relic/potion sites; purchase/payment and purge payment are not offer draws. Price expressions are deterministic E26 checkpoints, not RNG draws. | No stock/prices/offers before Shop room; visit/history/streams hidden. **Still open:** exact shop visit increment and restock/purchase stock-commit timing fields/checkpoints from accepted economy/native schema; deterministic price expression is bound, not numeric pool reconstruction. |
| `run.handoff` | Boss reward/relic child sites `boss.reward.*`, `boss.relic.*`, transition site `transition.act2.map`; scope `(act,handoff_ordinal,site)`. | Boss rewards use caller-qualified card/relic sites; mandatory children finish before transition. Handoff ordinal is semantic/checkpoint-owned. Act2 map generation uses separate `run.map.draft` role/site/draft ordinal. Transition itself draws no word absent a named child site. | Handoff/child ordinal and reward state private; exact callback/effect mapping remains I3 producer binding. |

**E25 question-room commit rows (finite state transition).** The selector snapshots
the three old ramp fields before drawing, then commits the resolved room and history
once; these are semantic state fields, not RNG cursor aliases:

| Own state/result | Read/write and exact condition | Checkpoint / commit |
|---|---|---|
| `monster_ramp:binary32` | If primitive selection is Monster and Tiny Chest did not replace it, write `F(.10)` (Juzu conversion still resets); otherwise write `F(old+F(.10))`. Tiny-Chest-forced Treasure increments. | After base-cell writes, Tiny Chest override, then Juzu conversion; same question-room resolution commit. |
| `shop_ramp:binary32` | If final resolved room is Shop, write `F(.03)`; otherwise `F(old+F(.03))`. Leaving-Shop suppression changes this roll's shop threshold to zero, not this update rule. | After overrides and final room resolution. |
| `treasure_ramp:binary32` | If final resolved room is Treasure, write `F(.02)` (including Tiny Chest); otherwise `F(old+F(.02))`. | After overrides and final room resolution. |
| Carried Tiny Chest counter | If owned, increment once per `?` roll; exactly4 writes0 and forces Treasure after the primitive result was consumed. It is not reset on Act transition. | In this same resolution, before Juzu tests the remaining selected room. |
| `question_room_roll_ordinal:u64` / `run.event` scope | Increment once per actual question-room roll; bind that semantic visit to `(act,event_visit,site,invocation)`. Invalid external actions do not roll or increment. | Once with the resolved question-room commit; event branch/ID construction, if the result is Event, is a later independent site/checkpoint. |
| Resolved room result | Starting Event, apply Monster A, then Shop B, then Treasure C writes; later matching interval overwrites earlier. Consume the one primitive `k` before Tiny Chest/Juzu; Juzu converts selected Monster to Event but does not alter scripted fights. | Commit exactly one final room result after overrides; no second roll for conversion or invalid input. |
| Shared run RNG/replay | Root32, seed-adapter ID, contract ID, partition, RunKey, immutable role/site/scope/key/cursor state, rules/content/site descriptors, replay semantic decisions. Act2 roles are `run.map.draft`, `run.encounter`, `run.card.acquire`, `run.relic`, `run.event`, `run.potion`, `run.shop`, `run.combat`, and `run.handoff` where a named producer actually draws. | Exact IDs/frame/key/block/lane/cursor and checked episode/site caller ordinals from accepted RNG §§1–2. Derive streams by exact role/site/scope; no shared parent cursor. Lazy derivation is allowed for a site not yet called; do not fabricate an advanced cursor or eagerly consume a word. Retain current run identity at Act boundary; Act2 scopes use same RunKey, not new episode/reset identity. | All replay/RNG provenance private. No source Java cursor parity, hidden RNG projection, or reset of current episode identity at handoff. |

**Readiness and public projection assertions.** `Act2Ready` now requires exactly one accepted
`ValidMap(mode=Act2Handoff,act=2,validator_version=1)` plus the empty entry node;
map draft state is not a successor policy field. Required component validators
check: ordered relic arrays contain no owned or previously consumed same-role ID;
card pity is in `[-40,5]`; event numerators sum to `16,777,216` and are the exact
Act2 tuple above; potion room modifier is0 and carried belt matches PlayerCore;
shop has carried gold/purge history but no instantiated stock/phase; all RNG state
matches accepted contract/partition/RunKey and no cursor exceeds `2^64`. Any
unlisted site required by Act2 producers is an initialization failure/open join,
not a default-zero stream. Terminal Act1 `final_obs` excludes map, pool arrays,
history numerators, appearance offset, shop ordinals, root/seed/key/counter/site
state, and rejected proposals; successor remains detached and private.

### 6.6 Phase visibility and cutoff

Before `Act2Ready`, the policy may observe only the current Act1 decision data
per A1C-REQ-033; no full profile object, Note ID, boss pool, hidden Act2 map,
encounters/events, rewards, shop stock, RNG, seed, cursor, rejected drafts,
transition checkpoint, diagnostic, or private instance identity. At terminal
delivery, `final_obs` is an immutable detached snapshot of the completed task
state; it contains no hidden Act2 successor fields. The successor remains owned
privately for an explicitly supported follow-on continuation, but that is not
provided by this Act1 task API unless a separate reviewed API defines it.

Vector same-step autoreset owns two distinct values: retired Act1 `final_obs`,
`reward=+1`, terminal outcome and episode identity; and new episode ordinary
reset observation/identity. `final_obs` and attached profile/result records are
deeply detached; reset/step/close or caller mutation cannot alter them. The next
episode uses its configured independent profile, never an inherited Note. Single
manual mode stays terminal with zero candidates until explicit reset. A post-
dispatch administrative failure returns no successful batch/final/reset; it
faults and quiesces the pool with actual commit provenance. Actual death produces
−1 only once, `terminated=true,truncated=false`; escape, skip, no-pick, non-death
fault or resource failure is not death and receives 0.

The native retired-delivery owner also retains a detached `TerminalFinalV1`:
`{run_identity, task_outcome, task_reward, terminated, truncated=false,
final_observation, completed_successor: SuccessorStateV1}`. This is an owned
historical final paired to the completed episode, not a pointer into the live slot,
new episode, or caller memory. It remains stable across replacement/reset/close and
is destroyed only with its owning detached delivery. `completed_successor` is
native-private and is not projected into policy observation/info or an importable
restoration API; it exists so the actual Act2-ready successor is retained rather
than substituted with the reset episode. Vector finals retain one such value per
retired terminal row in returned-row order. A fault produces no successful
`TerminalFinalV1`; its diagnostic is not a successful final.

## 7. Named imports and remaining joins

These contracts are the handoff producer side; do not block on other authors, but
do not claim final binding before main routes accepted revisions:

* **Accepted RNG/map producer (consumed):** exact accepted revision is main
  `35808a1b61a79566ec5a18cd50fea85ae60eadc0`, document SHA-256
  `e24d7f0667ccf6397a9ee84e5a1ccc72d40294f3c5cb9269f98b4f61d22447c1`. §6.5
  binds its exact seed/RunKey/`NativeRngState`, role/site registry, typed map
  draft/output, Act2 scope/counter law, relic shuffle/depletion, ordinary card
  pity, potion appearance history, shop purge/visit starts, question-room initial
  grid and hidden-state boundary. E25's binary32 ramp mutations, Shop suppression,
  Tiny Chest/Juzu order, cell write precedence and result commit are joined in
  §6.5. Remaining residuals are the all-seen Act2 boss-order site registry row
  specified in §5.2, native potion field/caller commit mapping, shop commit
  ordinals, and relic literal catalog/member predicates. No source Java
  cursor/correlation or `runSeed+200` parity is imported.
* **Execution import (I):** exact accepted producer revision must instantiate
  `ParentChildCheckpointV1` and `ResolvedTransitionEffectsV1` under §6.4, binding
  the exhaustive legal site/effect/result-code set and mapping each content
  callback's requiredness, commit, child, and resume into those fields. The record
  shape, statuses, once-only rules, and `Act2Ready` predicate are fixed here;
  concrete effect IDs/sites/payloads, callback-to-checkpoint rows, and producer
  revision remain pending until an accepted execution result is main-routed. This
  author does not reimplement every content callback or settle that lane's causal
  mechanics.
* **Catalog/numeric import (I):** exact legal stored-card ID aliases/upgrade
  domains and source member/domain/read/write/overflow facts from `catalog-numeric`,
  then routed into native contract dictionary. The profile grammar above specifies
  semantic set/exclusions, but does not make a partial accepted card crosswalk a
  full manifest.
* **Native schema owner:** consumes `CreationProfileV1`, `TransitionCheckpointV1`,
  `ParentChildCheckpointV1`, `ResolvedTransitionEffectsV1`, and `SuccessorStateV1`.
  The required producer-to-native projection is: profile fields are validated
  inputs only; checkpoint and successor are run-private; only the current legal
  Act1 observation is policy-visible; terminal `final_obs` is detached Act1 state
  with zero candidates and no successor/profile/Note hidden fields; successor
  stays private for continuation. Exact native property names/types/widths,
  visibility dictionary, packed encoding, codec/hash, API names, and restoration
  format must be supplied by a concrete accepted native schema revision and remain
  pending that handoff. This projection does not claim that revision exists.

The open integration work is exact member/catalog joins, final native enum codes
and packed projection, acceptance/routing of the finite all-seen Act2 boss-order
site row in §5.2, relic literal content predicates, and the remaining
potion/shop semantic commit checkpoints identified in §6.5, plus cross-lane
consistency review. Map/RNG constructor, E07 first-unseen priority, E25 question
history mutation/precedence, relic depletion transaction law, and named
card/potion/shop/question initial states are now bound. No true new product policy question remains
for this bounded work: fields/widths/aliases are engineering choices. Any newly
discovered product choice is routed to main with finite options and impact, not
guessed here.

## 8. Completion / non-completion ledger

| Spec requirement | Concrete closure in this delivery | Still open / explicit nonclaim |
|---|---|---|
| A1C-REQ-028: task reward and successor cutoff | Exact phase/terminal ordering; +1 only after resolved initialized Act2 pre-first-decision; −1 actual death; otherwise 0; no truncation/Act2 action. | Execution callback join and native API binding; future T25/T33 unexecuted. |
| A1C-REQ-029: profile/history | Exact `CreationProfileV1`, experienced preset, independent overrides/reset, fixed start, Note minimum record, discovery/Neow separation and no inheritance. | Literal complete card/upgrade manifest and native codec review; B11 not declared closed. |
| B05 successor/children | Ordered checkpoints; concrete child/effect record fields, legal statuses, once-only resume/commit, and necessary/sufficient `Act2Ready` predicate. | Exact content callback/site/effect/result-code mapping and accepted execution producer revision; no generic work-drain promise. |
| B10 native state/visibility | Successor component construction/carry/reset matrix and required private/public projection; accepted typed RNG/map records and hidden boundary joined in §6.5. | Exact native property/type/width dictionary, packed codec/hash, API and restoration schema remain pending native owner. |
| B11 stored profile | Versioned profile grammar, validation algorithm, exact manifest equality predicate/negative controls, independent reset. | Literal card/alias/upgrade/arithmetic rows remain pending accepted catalog/numeric producer receipt; no false local catalog. |
| Act2 initialization | Source-anchored list resets and level values; accepted map/RNG record, E07 encounter queue and first-unseen boss priority, relic depletion transaction law, card pity, potion appearance, shop purge/visit, full E25 question-room state transition and hidden cutoff; potion modifier explicitly 0. | Exact accepted registry row for all-seen Act2 boss ordering; potion semantic callback→field commit/domain; shop visit/restock/purchase commit ordinals; relic literal member/caller predicates; no Act2 gameplay. |

Future A1C-T01, T24, T25, T29, T31, T33 and T35 remain unexecuted acceptance
cases; this is specification work, not gameplay evidence. Whole-spec independent
review/human approval, reviewed implementation plan, and separate implementation
start are still required.
