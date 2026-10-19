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
document SHA `db5061114732ba8ea18099e81249df1fdfc015055ed7ab528c8ab5121ff4d599`).
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
| `BossCombat` / combat + run | Resolve lethal packet, boss-specific death/split/cannot-lose and terminal logic; apply actual death/rescue rules. Live permanent mutations (Feed/gold/maxHP/card growth/relic counters/potions) already committed remain. Mark boss discovery at prebattle hook if reached. | An actual player death terminates −1. Boss dying/`isDying`, queued lethal packet, split child, escape, room victory latch or combat reward construction is not +1. No post-death heal. |
| `BossCombatRewards` / reward owner | Generate the boss combat reward rows under boss rarity/context, with their own card history/pity and other reward law. Each card row is an independent legal Take/Skip; Take consumes its row before deferred obtain resolves. Whole-list Proceed can leave legal rows unclaimed. Resolve all accepted obtains and any mandatory child before leaving this phase. | Do not call boss-combat rewards “boss chest”; TreasureRoomBoss is a distinct later phase. A skip is valid and does not undo offer history. |
| `BossTreasureEntry` / chest owner | On entry, run the boss-chest opening hooks (`onChestOpen(true)`); construct three sequential boss-tier relic offers from the live boss pool (each consumes its pool entry), then open boss-relic selection. | No ordinary chest size/tier/gold roll, no sapphire-key attachment, no ordinary `onChestOpenAfter`, no Matryoshka extra relic, and Cursed Key's ordinary-chest curse. Do not emit chest gold or force a key. |
| `BossRelicChoice` / choice owner | Take one offered relic or skip; the boss chest itself can also be proceeded past with no pick (record no-pick). Normal take closes selection before ordinary relic arrival/obtain/onEquip. Black Blood replacement is a distinct instant path. | Skip/no-pick is legal; offers are not claims. There is no implicit pick. Do not run onEquip while the choice is still open. |
| `BossRelicChild` / relic + continuation owners | If taken, install exactly once and apply its ordered immediate effects; suspend on any real child selection/reward (e.g. Astrolabe purge/transform, Empty Cage removals, Pandora starter replacement confirmation, Calling Bell curse then its screenless relic offers, Tiny House's upgrade/maxHP/gold/potion/reward rows). Resolve or legally skip every exposed child according to that source's actual policy, resume the owning parent/checkpoint once. Applied grants/removals/rewards are durable at their own checkpoint. | A local flag, screen close, room COMPLETE, queued child, or reward row remaining does not prove global readiness. Preserve empty/automatic/grid size branches and no fabricated choice. Skippable card/relic/potion rows need not be claimed. |
| `NextDungeonProceed` / handoff owner | After permitted choices, next-dungeon Proceed records no-pick where appropriate; a three-relic warning can delay this handler once. Then start fade and mark dungeon beaten. | This still has not constructed TheCity; fade start / beaten flag is not +1. Do not drain unrelated work or discard committed mutations. |
| `DungeonTransition` / main game | Complete the actual main-game transition screen; construct the named successor with the same player object/state. Run shared transition setup, then TheCity/Act2 initialization in defined order below. | An administrative fault quiesces the pool, retains actual committed prefix/provenance, emits no success final/reward/autoreset. It is not death, defeat, retry, rollback or successful profile delivery. |
| `Act2Ready` / run + env | After required map/state initialization, retain the fully initialized successor privately and freeze a detached terminal final of the completed episode; reward +1 exactly once, `terminated=true,truncated=false`. Do not publish an Act2 observation/menu or apply an Act2 action. | Checkpoint is strictly before any Act2 player decision. No Act2 policy play or hidden successor leakage. |

Terminal arbitration applies at every phase: actual player death is −1 and
`terminated=true,truncated=false`, even if earlier rewards/keys/Note commits remain
durable; no later transition heal can revive it. A legally available escape is a
distinct nondeath outcome with reward 0, never boss victory or +1; Act1 boss escape
is not manufactured where the source does not offer a legal escape route. Skip,
no-pick, empty offer, administrative numeric/resource failure, and post-dispatch
fault are also not death. A fault quiesces and returns no successful final or
autoreset; it preserves the committed prefix and fault boundary without rollback.
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
| Act-local encounter queues, boss candidate list, ordinary events, six repeatable shrines, question-room probabilities, card/potion pools and local reward state | Reinitialize for Act2 at the described setup steps. Act1 queue heads/pools are not reused as Act2 pools. |
| Remaining run-wide one-time events | Carry the live remaining list; do not repopulate consumed events. Filter by Act2 eligibility when generating a future event, not by exposing hidden future IDs now. |
| Discovery history | Carry Act1 seen-boss additions and existing independent Act2/Act3 seen sets. The new Act2 boss-selection setup depends on Act2's seen set; it is hidden from policy until legally announced/revealed. Boss lists do not become a public “all seen” feature by accident. |
| Acquired ruby/sapphire/emerald flags | Carry actual owned keys; optional and deferred key effects commit only at their actual consumer. The burning map-node designation is distinct from acquired Emerald. Act1 boss chest has no Sapphire attachment. |
| Run play clock | Pure elapsed eligibility is selected satisfied; no clock/wait-only action is added. Do not invent Act2 policy time behavior in this contract. Preserve only whatever resolved run-owned logical clock fields the selected final run schema requires; no frame clock is an Act1 completion gate. |
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
3. Construct Act2 encounter pools/queues and the Act2 boss candidate state; create
   Act2 ordinary event and six recurring shrine pools; rebuild card, potion and
   relic offer pools and Act2 chance parameters. Keep the one-time event remainder.
4. Generate/validate the Act2 map and burning Elite designation through the
   accepted map producer. The visible map contains only map facts permitted at
   this boundary; future encounters, boss order, hidden room/event identities,
   offer pools and RNG state remain private.
5. Create the Act2 empty entry node/current position and set its initial state.
   Do not enter the node, roll its encounter/event, perform its room-entry hooks,
   construct/publish an Act2 action menu, or select an action. Mark `Act2Ready`
   only after these state components are coherent.

The source Acts2/3 record says queues and boss list are built before act-specific
map; native selected map RNG is independent, so this is an initialization phase
ordering dependency, not a demand to recreate source cursor correlations. Exact
per-subconstructor order among nonobservable pools must be frozen by the named
producer contract; no policy-visible field may be derived by hidden allocation
order.

### 5.2 Act 2 initial state domains (source-informed)

| Owner/state | Required initialized domain/value | Visibility at Act2Ready |
|---|---|---|
| Act/floor/entry | `act=2`; global floor is the next Act2 entry floor (source successor from the Act1 boss); fresh path; Act2 start node exists with no Act2 room visited. Exact native floor counter representation is the run owner contract. | Internal successor only at this terminal; no Act2 fields are inserted into the Act1 policy final. |
| Map | TheCity topology, room assignments, edges, and valid burning-Elite marker generated as one Act2 map; every exposed map satisfies the selected valid-map predicate (including >=1 Elite) by deterministic reject/regenerate before exposure. New map is not a patched Act1 map. | Retained in private successor; not projected into the Act1 final. Burning marker is not acquired Emerald key. No hidden future encounter/event/boss result. |
| Encounter queues | Weak and strong lists: two weak entries, the separately filtered first strong, then twelve more strong; ten elite entries. Act2 boss candidate pool/order built from its seen history. Source nominal weak: Spheric Guardian, Chosen, Shell Parasite, 3 Byrds, 2 Thieves (weight 2 each). Strong nominal weights: Chosen and Byrds 2; Sentry and Sphere 2; Snake Plant 6; Snecko 4; Centurion and Healer 6; Cultist and Chosen 3; 3 Cultists 3; Shelled Parasite and Fungi 3. Elites: Gremlin Leader, Slavers, Book of Stabbing (1 each). Boss identities: Automaton, Collector, Champ. | Queues, future list order, conditional draws and unselected identities are private. The actual future encounter is not an observation. No Act2 fields are projected into the Act1 final. |
| Encounter history | Preserve Act1 visited rooms and encounter history; initialize Act2's own encounter queue/history and first-strong exclusion from actual last Act2 weak entry (not Act1 last encounter). Ordinary append rejects last two; elite append rejects only immediate prior; first strong uses source-specific exclusions, not ordinary rule. Room departure consumes its Act2 queue head, event fight does not. | Private successor history; do not publish hidden queue entries or underlying queue cursor. |
| Events/shrines | Reinitialize the 13 Act2 ordinary event IDs: Addict; Back to Basics; Beggar; Colosseum; Cursed Tome; Drug Dealer; Forgotten Altar; Ghosts; Masked Bandits; Nest; The Library; The Mausoleum; Vampires. Reinitialize six repeatable shrines: Match and Keep, Wheel of Change, Golden Shrine, Transmogrifier, Purifier, Upgrade Shrine. Combine only the remaining one-time-event list under eligibility at a future event-selection site. | All pools private. Do not expose event identities/order or eligibility for hidden future rooms. |
| Rewards/card state | Rebuild Act2 card generation/reward state and offer histories; keep prior deck and any run-wide reward/pity history that source/native policy defines. Boss reward rows are consumed/skipped as decided, not copied into Act2. | No unoffered card or reward candidate. Public current rewards only after actual generation. |
| Shop/economy | Reinitialize Act2 shop visit state and Act2 applicable room/economy parameters; carry current gold and existing price/removal histories that are run-owned. The shop itself does not exist until a shop room is visited. | No stock, prices, relic/potion/card offers, or shop RNG before a shop visit. |
| Relic pools | Rebuild applicable relic pools from Act2 context with owned-ID removals and source pool/depletion rules; ordinary Act2 relic chances Common/Uncommon/Rare=50/33/17; boss pool is initialized for boss selection/reward role; preserve acquired relic list/counters separately. | Pool members, order, depletion and future offers hidden. Relics already owned remain visible. |
| Potions | Reinitialize Act2 potion-drop probability modifier to its normal reset value, with source shared modifier reset at transition; retain actual current belt contents/capacity and Potion counters/Sozu effects. | Current belt visible under normal observation; no future potion proposal/identity. |
| Room probabilities | Act2 room chance parameters from source construction: Shop .05, Rest .12, Event .22, Elite .08 (binary32 source anchors). Ordinary treasure parameter zero does not remove the fixed map chest. Chest size 50/33/17; ordinary relic tier 50/33/17; colorless rare parameter .3; card-upgrade parameter .25. These are map/producer inputs, not independent per-node probabilities or all producer rules. | Only realized public map types; no hidden chance values or unrevealed chest/reward outcomes. |
| One-time event history | Continue only the remaining source run-wide one-time event IDs; already consumed remain absent. Act2-specific eligible additions (including Designer, Duplicator, Knowing Skull, N'loth) are evaluated at their proper future event generation. | Hidden pool not exposed; revealed/history facts only. |
| Native random state | Bind Act2 map, encounter, boss, event, relic, card, potion, shop and combat-generation roles to exact accepted native versioned imports, with independent reproducible system boundaries and preserved mechanic conditioning/history. | All seeds, stream keys, cursors, hidden future samples, rejected proposals and RNG diagnostics are private. |
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
   slot `(occupied,potion_id,potency)`, acquired key bits, run-level permanent
   counters/history.
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

### 6.3 Phase visibility and cutoff

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

* **RNG import (I):** `AcceptedRunRngV1` from the named `rng-maps` lane, carrying
  its actual reviewed seed adapter, semantic roles/sites, unbiased bounded
  primitive/shuffle, domain versions, independence boundaries, and deterministic
  reset/replay rules. This contract consumes separate Act2 producer roles for
  map, encounter/boss, event, card, relic, potion/shop and combat setup as the
  reviewed producer specifies; it does not import original twelve stream objects,
  runSeed+200 map parity, source Java shuffles, or exact source cross-system
  cursor relations. Bind the Act2 map constructor's actual reviewed input/output
  after route; before that route this is explicitly **I**, not resolved.
* **Execution import (I):** `ResolvedTransitionEffectsV1` and
  `ResolvedParentChildCheckpointV1` from the named `execution` lane, binding
  actual boss reward/skip, relic child, effect consumer/resume, death/fault,
  semantic immediate-causal settlement and no-unrelated-drain rules. This
  document fixes the handoff outcome/order contract; it does not reimplement
  every content callback or settle another lane's incomplete transitive mechanics.
* **Catalog/numeric import (I):** exact legal stored-card ID aliases/upgrade
  domains and source member/domain/read/write/overflow facts from `catalog-numeric`,
  then routed into native contract dictionary. The profile grammar above specifies
  semantic set/exclusions, but does not make a partial accepted card crosswalk a
  full manifest.
* **Native schema owner:** `CreationProfileV1`, `TransitionCheckpointV1`, and
  `SuccessorStateV1` are the concrete producer records for the native owner to
  consume. This author does not claim packed encoding, final property/visibility
  dictionary, private core codec, hash, final API names, or restoration import.

The open integration work is exact member/catalog joins, final native enum codes
and packed projection, map/RNG constructor binding after reviewed deliveries,
and cross-lane consistency review. No true new product policy question remains
for this bounded work: fields/widths/aliases are engineering choices. Any newly
discovered product choice is routed to main with finite options and impact, not
guessed here.

## 8. Completion / non-completion ledger

| Spec requirement | Concrete closure in this delivery | Still open / explicit nonclaim |
|---|---|---|
| A1C-REQ-028: task reward and successor cutoff | Exact phase/terminal ordering; +1 only after resolved initialized Act2 pre-first-decision; −1 actual death; otherwise 0; no truncation/Act2 action. | Execution callback join and native API binding; future T25/T33 unexecuted. |
| A1C-REQ-029: profile/history | Exact `CreationProfileV1`, experienced preset, independent overrides/reset, fixed start, Note minimum record, discovery/Neow separation and no inheritance. | Literal complete card/upgrade manifest and native codec review; B11 not declared closed. |
| B05 successor/children | Ordered boss combat/reward/chest/relic-child/proceed/main-transition checkpoints, required nested completion and source-specific transition state. | Cross-lane exact callback/continuation composition and source-reachability residuals; no generic work-drain promise. |
| B10 native state/visibility | Concrete checkpoint/successor components, owner/reset/visibility and private-vs-public boundary. | Complete native dictionaries/packed fields/visibility/site/RNG contracts and final hash remain with native owner. |
| B11 stored profile | Versioned profile grammar, legal-set semantics, stored fields, validation and independent reset. | Full literal card and per-card upgrade-law equality is explicitly waiting for reviewed catalog input, not disguised as complete. |
| Act2 initialization | Exact listed Act2 setup domains and source-informed initialization ordering, state carry/reset table, hidden cutoff. | RNG/map producer exact reviewed bindings and full Acts2 implementation are not claimed; no Act2 gameplay. |

Future A1C-T01, T24, T25, T29, T31, T33 and T35 remain unexecuted acceptance
cases; this is specification work, not gameplay evidence. Whole-spec independent
review/human approval, reviewed implementation plan, and separate implementation
start are still required.
