# Current native encounter coverage

This inventory reconciles the57 concrete `EncounterId` values with the shared
setup owner, catalog, world domains, finite Run admission and numeric publication.
It describes the encounter-closure change against accepted M14 commit
`a8261aa55e64d9c013f418b37711060003ca50b4`. It is native source/test evidence;
it does not establish a matching built/imported Python extension, complete
standard pools, all-character A0–20 Runs or original-game parity.

## Supported setups and finite profiles

[Shared setup metadata](../crates/spire-core/src/combat/enemies/mod.rs) owns57
setups. The original43 retain keys1–43; Blue Slaver and Two Fungi append keys44/45,
and the remaining current Act1/2 setups append keys46–57.
These are setup RNG keys, distinct from observation vocabulary IDs. Every row
resolves through the [definition catalog](../crates/spire-core/src/content/encounters.rs)
and the shared actor dispatcher. Catalog omissions for Small/Large/Lots of Slimes,
six slime species and stale definition-only flags were corrected here.

`v1` below means the frozen43-entry standalone Ironclad A0 profile. `v2` means
that set plus the14 explicit additions. `Run` means selectable in the accepted
captured progression for that act: M5/M12 onward for Act1, M6/M13 onward for
Act2 (boss completion requires M7/M13 onward), M8/M14 for Act3 ordinary/elites,
M9/M14 for Act3 bosses, and M10 Ironclad Heart for Act4. Later Ironclad profiles
retain their predecessors. M14 Silent remains Acts1–3 only.

| Act | Encounter ID | Setup key | Admission |
|---:|---|---:|---|
|1|cultist|1|v1/v2, Run|
|1|jaw_worm|2|v1/v2, Run|
|1|two_louse|3|v1/v2, Run|
|1|small_slimes|4|v1/v2; internal Run gate admits it, captured route does not select it|
|1|large_slime|5|v1/v2, Run|
|1|lots_of_slimes|6|v1/v2, Run|
|1|slime_boss|7|v1/v2, Run|
|1|gremlin_nob|10|v1/v2, Run|
|1|lagavulin|11|v1/v2, Run|
|1|three_sentries|12|v1/v2, Run|
|1|three_louse|13|v1/v2, Run|
|1|guardian|14|v1/v2, Run|
|1|hexaghost|15|v1/v2, Run|
|1|blue_slaver|44|v2 only; new solo setup uses existing `slaver_blue` actor|
|1|two_fungi_beasts|45|v2 only; two independent new `fungi_beast` actors|
|1|red_slaver|46|v2 only; existing Red Slaver actor|
|1|looter|47|v2 only; real theft, death reward and escape|
|1|gremlin_gang|48|v2 only; four ordered draws without replacement from the member bag|
|1|exordium_thugs|49|v2 only; ordered weak-wildlife / strong-humanoid composition|
|1|exordium_wildlife|50|v2 only; ordered strong / weak wildlife composition|
|2|three_cultists|8|v1/v2, Run|
|2|spheric_guardian|16|v1/v2, Run|
|2|three_byrds|17|v1/v2, Run|
|2|chosen_and_byrds|18|v1/v2, Run|
|2|snecko|19|v1/v2, Run|
|2|centurion_and_healer|20|v1/v2, Run|
|2|gremlin_leader|21|v1/v2, Run|
|2|slavers|22|v1/v2, Run|
|2|book_of_stabbing|23|v1/v2, Run|
|2|champ|24|v1/v2, Run|
|2|collector|25|v1/v2, Run|
|2|bronze_automaton|26|v1/v2, Run|
|2|chosen|51|v2 only; existing Chosen actor|
|2|shelled_parasite|52|v2 only; Plated Armor, Leech and armor-loss stun|
|2|two_thieves|53|v2 only; Looter then Mugger, independent ledgers|
|2|sentry_and_sphere|54|v2 only; Sentry then Spheric Guardian; Sentry opens Bolt|
|2|snake_plant|55|v2 only; multi-hit attack, debuffs and Malleable|
|2|cultist_and_chosen|56|v2 only; Cultist then Chosen|
|2|shelled_parasite_and_fungi|57|v2 only; Parasite then Fungi|
|3|jaw_worm_horde|9|v1/v2, Run|
|3|orb_walker|27|v1/v2, Run|
|3|three_shapes|28|v1/v2, Run|
|3|four_shapes|29|v1/v2, Run|
|3|sphere_and_two_shapes|30|v1/v2, Run|
|3|three_darklings|31|v1/v2, Run|
|3|spire_growth|32|v1/v2, Run|
|3|transient|33|v1/v2, Run|
|3|maw|34|v1/v2, Run|
|3|writhing_mass|35|v1/v2, Run|
|3|giant_head|36|v1/v2, Run|
|3|nemesis|37|v1/v2, Run|
|3|reptomancer|38|v1/v2, Run|
|3|awakened_one|39|v1/v2, Run|
|3|time_eater|40|v1/v2, Run|
|3|donu_and_deca|41|v1/v2, Run|
|4|shield_and_spear|42|v1/v2, M10 fixed route|
|4|corrupt_heart|43|v1/v2, M10 fixed route|

The accepted playable captured sets remain12 (M12),24 (M13),40 (M14);
Small Slimes and the14 new encounters do not silently enter them. M12/M13
successor boss icons are metadata until their playable act is supported.
[Run admission](../crates/spire-core/src/run/combat.rs) and captured selectors
in [Act1](../crates/spire-core/src/run/rooms/act1.rs),
[Act2](../crates/spire-core/src/run/rooms/act2.rs) and
[Act3](../crates/spire-core/src/run/rooms/act3.rs) remain unchanged.

All57 are A0 setups. The low-level constructor additionally supports A1–20 for
Cultist, Jaw Worm, Three Cultists, Jaw Worm Horde, Gremlin Nob, Blue Slaver and
Two Fungi Beasts. Other nonzero-Ascension setups reject explicitly. Standalone
resource entry and accepted normal profiles remain A0. A0–20 actor construction
is a narrower claim than A0–20 Run delivery.

## Standard-pool closure and normal progression

Every concrete ID in the broad weighted [world domains](../crates/spire-core/src/run/world.rs)
now has a shared A0 setup. The two Act4 IDs use their fixed route. The prior12
Act1/2 gaps are included by the primary session's expanded repair scope; their
older exclusion from finite milestone profiles does not make them future content
under this task. The new support boundary is all57 defined standard encounter IDs.

Captured normal Run selectors and admission remain frozen, so this setup/actor
closure does not silently switch their queues to the broad standard domains.
The remaining delivery work includes selecting complete standard content in a
new Run profile, broader character/Ascension coverage and event-owned contexts
outside these57 IDs. The reference library's67 encounter-context records and65
species are a different inventory; they do not establish native admission.

`Unknown` is an open fallback rejected by finite admission, not a58th supported
encounter. An opaque/Unknown intent category is separate: existing Split, charge
and other nonattack mechanics execute through the shared resolver. A display
label does not establish a missing actor.

## Publication and compatibility

The default `ironclad-combat-v1` catalog stays at43 encounters with its accepted
catalog descriptor, support order, policy-view-v1 and fingerprints. Explicit
`"profile_id":"ironclad-combat-v2"` selects57 encounters and a versioned catalog,
content profile and policy-view-v2. Its enemy vocabulary appends Fungi Beast, Looter, Mugger, Shelled Parasite and
Snake Plant. Its power suffix adds Spore Cloud, Thievery and Plated Armor; its
intent suffix names the new actor moves. Existing Malleable is reused without
changing Writhing Mass's accepted reset value. Existing encounter IDs for
`blue_slaver`/`two_fungi_beasts` already exist in the M10-derived vocabulary;
existing Slaver intents and Fungi Bite/Grow need no renamed tokens.

The [catalog/session](../crates/spire-env/src/combat_training.rs) selects the
version and the [policy view](../crates/spire-env/src/combat_policy.rs) reuses the
[versioned Run encoder](../crates/spire-env/src/run_observation.rs). Resources,
setup, item custody, turns and settlement still belong to the same Run/combat
owners. Default combat-v1, M9/M10, policy-v3 and M12–M14 descriptors/replays remain
frozen; their vocabularies do not gain Fungi. The existing JSON binding transports
the explicit catalog field without another Python gameplay loop. No extension
build/import or Python-delivery claim follows from native checks.

V2 optionally accepts already-owned `gold` per entry (default0, native u32
bounds); v1 rejects that field. Ledger accrual, currency subtraction and the HP
hit are distinct shared effects. Run publishes committed currency after attempted
actions, including later faults. Actual thief death earns a stolen-gold reward;
it does not refund currency immediately. Escaped thieves retain their lost gold,
and all-retained-escaped suppresses ordinary room gold. The existing room reward
owner claims earned receipts. Standalone v2's terminal `last_exit` reports earned
unclaimed `stolen_gold_rewards`, `all_enemies_escaped` and `mugged`; its single
combat boundary does not add a reward-room gameplay loop. These facts distinguish
an escaped clear from enemy kills even when both end combat.

## Evidence and validation boundaries

The historical constructor test originally failed before reaching either actor.
Its incorrect `blue_slaver` actor and unprefixed `rake`/`stab` assertions were
corrected to the established `SlaverBlue`/`slaver_blue` and
`slaver_rake`/`slaver_stab` identities; the original A0–20 range remains.
It now checks real resolver entry for both encounters at each ascension.

[Full enemy tests](../crates/spire-core/src/combat/tests/full_enemies.rs) exercise
new HP/ascension setup, independent actor streams, actual damage, A2/A17 growth,
Slaver Weak/repeat rules, nonfinal/final Spore Cloud, Artifact and enemy-turn
Thorns ordering. [Core standalone tests](../crates/spire-core/tests/combat_training.rs)
exercise all old43 encounters to actual Death/Victory, pin original RNG keys,
check catalog closure and admit new encounters only through v2.
[Native transport tests](../crates/spire-env/tests/combat_training.rs) cover
version selection, default rejection, numeric publication and real turns.
[Complete-entry tests](../crates/spire-env/tests/encounter_closure.rs) exercise
all57 resets/turns and actual gold/receipt/escape/Death publication.
[Thief actor tests](../crates/spire-core/src/combat/tests/encounter_thieves.rs)
check per-actor custody and faults, including second-Mug Thorns deaths before
RollMove. [Ordinary actor tests](../crates/spire-core/src/combat/tests/encounter_ordinary.rs)
check Parasite/Plant AI, armor, Stun history, Malleable and Life Suck healing.
These supplement existing per-family and captured-profile tests; they are not
exhaustive interaction or original-runtime proofs. Genuine losses, including
M14's unproven canonical normal-start completion, remain qualified in the
[accepted profile](milestones/m14-silent-act3.md).

New named static facts and their exact provenance limits are recorded in
[encounter findings](research/2026-10-26-encounter-closure.md). Existing enemy
references retain the grades in the [enemy guide](../data/enemies/README.md).
