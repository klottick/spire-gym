# M12 — Silent A0 through Act1 (accepted profile)

Status: accepted at the finite native scope below after one fresh independent
GPT-6 Luna MAX milestone review and root integration verification. The reviewer
found no Critical, Important or Minor findings; all nine considered scope choices
and the documented Rust enum compatibility tradeoff received explicit root rulings.
The [approval record](../superpowers/plans/2026-10-26-m12-silent-act1-approval.md)
identifies the exact reviewed design/plan and execution condition satisfied by
Gym66's main merge at `533dad1c8aec86df624b3742b6b3ad1dbf1d0033`.

## Shared progression and normal start

`RoomScenario::M12SilentAct1` adds a finite captured Silent A0 profile to the same
Rust Run owner used by the Ironclad milestones. The same generated topology,
encounter setup, combat resolver, rooms, rewards, shop, selections, inventory
custody, boss chest and act transition execute Silent's episode.

The entry derives resources from `Run::new` and `Catalog`: 70/70 HP, 99 gold,
five StrikeGreen, five DefendGreen, Survivor, Neutralize, Ring of the Snake,
three empty potion slots and no keys. The caller selects actual short Neow:
Max HP +7 or Neow's Lament. Full Neow is outside this profile. Ring's existing
CombatStarted hook adds two cards to the ordinary five-card opening draw.
There is no separate Silent starter, room loop or resolver.

Act1's supported ordinary captures are Cultist, Jaw Worm, Two Louse, Large Slime,
Three Louse and Lots of Slimes; elites are Gremlin Nob, Lagavulin and Three
Sentries; bosses are Guardian, Hexaghost and Slime Boss. These are captured
encounters on generated topology, not the full standard encounter sampler.
The initial CreationProfile's unseen-boss history selects each boss before map
generation; choosing a scenario does not mutate a generated map.

## Finite captured content

The profile exposes 19 permanent green card kinds, 13 possible relic kinds and
four potion kinds. These are declared supported captures, not complete standard
Silent reward/shop/drop pools. The public core constants
`M12_SILENT_ACT1_{CARDS,RELICS,POTIONS}` identify their finite universe.

| Producer | Captured rows |
|---|---|
| Ordinary card rewards | Rotate Poisoned Stab / Backflip / Blade Dance; Deadly Poison / Cloak and Dagger / Dagger Throw; Noxious Fumes / Accuracy / Well-Laid Plans |
| Boss card reward | Nightmare and explicit Skip; one supported green Rare, not three distinct rare offers |
| First/later merchant cards | Quick Slash / Reflex / Catalyst for 50/75/75 gold; Deflect / Tactician / Nightmare for 50/75/150 gold |
| Elite relic offers | Lantern, Pear, Mango, Snecko Skull, Toy Ornithopter, Strawberry, Anchor, Blood Vial; cursor consumes offers even if declined and skips already-owned relics |
| Merchant relic/potion | First Snecko Skull / Dexterity Potion, later Bag of Preparation / Fire Potion; fixed 150/50 gold, with owned/full-belt checks |
| Ordinary treasure | Strawberry / Anchor / Blood Vial captures and actual Sapphire-versus-relic choice |
| Boss treasure | Empty Cage / Slaver's Collar, or explicit skip/confirmation; actual Empty Cage mandatory child selections |
| Combat potion rewards | Fire, Strength, Weak by zero-based combat visit modulo 3 |
| Shared events | Upgrade Shrine / Purifier, with real captured permanent upgrade/removal choices |

The four basic kinds are StrikeGreen, DefendGreen, Survivor and Neutralize.
Generated Shivs and Nightmare copies remain combat-local; Shiv is not a permanent
reward. Actual Act1 encounters can produce Dazed, Burn and Slimed. Shared handlers
retain accepted M11 card mechanics and their source qualifications.

Snecko Skull is Silent-exclusive and Common. Replacing the old captured
Ironclad-exclusive Rare Magic Flower is a declared eligible capture change, not
tier-preserving random generation. Black Blood, Brimstone, Mark of Pain, Runic
Cube and Ironclad-exclusive potions do not enter these offers. No Silent-exclusive
event is invented.

## Public boundary and executable evidence

The native public adapter reuses RunSession/RunBatch and semantic replay owners
with distinct M12 identities. The controller verified 17 public M12 tests (schema, actual selection/card values,
session, replay and batch). The
controller has freshly verified the native caller integration suite (4/0),
including stepwise semantic replay of all six normal-start boss alternatives
and both genuine Death controls. M11's authored-combat bridge remains diagnostic
and does not substitute for normal Neow/map progression.

The normal caller policy was frozen before fixed runs (support source SHA-256
`ed049190f700430175fabf7624dfa9eac4a3294ba4b14af7d4a09a7f45d490f6`).
It selects Lament, follows public routes through one elite, then uses public
room choices/current combat intents and cards. It was not tuned after outcomes.

| Seed | Guardian | Hexaghost | Slime Boss |
|---|---|---|---|
| 7 | Act2Ready, 269 actions | Act2Ready, 280 actions | Act2Ready, 314 actions |
| 11 | Act2Ready, 267 actions | Act2Ready, 265 actions | Act2Ready, 268 actions |

All six observed native cases record Victory against the requested Act1 boss,
traverse an elite and reach act2/floor17 at 77/77 HP after ordinary act healing.
The final public map's boss is the initialized Act2 boss, not a substituted
Act1 encounter; committed CombatEntered/CombatFinished records prove Act1's
actual selected boss. Separate nonattacking controls at both seeds produce
native Death at floor1 after nine accepted actions and HP0. Budget0/1 are
nonterminal BudgetExhausted caller outcomes. These are finite deterministic
examples, not a policy-quality or all-seed success claim.

Run the native examples with existing offline dependencies:

```sh
cargo run -p spire-core --example m12_silent_act1 --locked --offline -- --seed 7 --scenario all
cargo run -p spire-core --example m12_silent_act1 --locked --offline -- --seed 11 --scenario all
cargo run -p spire-core --example m12_silent_act1 --locked --offline -- --seed 7 --scenario death
cargo run -p spire-core --example m12_silent_act1 --locked --offline -- --budget 1
```

The cutoff is actual Act2Ready after the Act1 boss reward, boss chest children
and ordinary act healing: act2/floor17, nonterminal, no legal actions and no
readiness bonus. Genuine Death is a gameplay terminal; caller budget exhaustion
and operational faults are separate. No injected late-act resources/keys/maps,
enemy adjustments, easier seed search or forced-win policy is permitted. The
verification record preserves each fixed7/11 outcome.

## Limits and source grades

M12 does not claim full green pools, rarity/pity rules, all exclusive items,
full Neow/events, standard encounter RNG, Acts2–4 gameplay, higher ascension,
original-game execution or Java PRNG parity. M15 must close complete Silent A0
coverage or expose an explicit blocker. Inputs retain source grades/attribution
in the [data guide](../../data/README.md), domain guides,
[accepted M11 profile](m11-silent-combat.md) and qualified Silent content report.
Reference membership alone is not runtime proof.

No Python transport copy or extension refresh is included; native Rust tests do
not establish refreshed Python delivery. Existing M1–M10 and Gym66 frozen/opt-in
contracts remain preservation requirements.

## Native configuration compatibility

The shared native session gains additive `RunBossHistory` Act1 alternatives,
with `M9BossHistory` retained as a compatibility alias. Old variant names,
serialized spellings, existing profile behavior and numeric fingerprints remain
preservation contracts. Rust consumers that exhaustively match the former three
variants must add the new Act1 arms; this is not unchanged exhaustive-match
source compatibility or a frozen Rust ABI. Profile/history combinations are
validated before execution. The two existing test matches need only unreachable
Act1 arms; their old behavior assertions remain intact.

## Native public contracts

`RunSession::new_m12` and `RunBatch::new_m12` use the same native session, token,
record, replay, batching and encoder owners as the existing profiles. Configure
Silent/A0 with `RunBossHistory::{Act1GuardianUnseen, Act1HexaghostUnseen,
Act1SlimeBossUnseen}`. The constructor rejects foreign characters/ascensions and
history/profile combinations before creating the episode. Reset/replay and the
public boundaries agree with all six fixed native transcripts. Native sessions
retain general seed/ordinal behavior; the bounded example accepts only7/11.

| Identity | Value |
|---|---|
| Profile | `M12SilentAct1` |
| Observation | `spire-gym-m12-silent-act1-observation-v1` |
| Replay | `spire-gym-m12-silent-act1-replay-v1` |
| Rules | `spire-gym-native-m12-rules-v1` |
| Content | `spire-gym-m12-silent-a0-act1-captured-v1` |

Actual base/upgraded green card values and mutable generated Shiv values pass
through one shared public card-value projection. Actual discard/Reflex reshuffle,
Tactician, optional retain/Confirm0 and Nightmare capture/delayed-copy snapshots
encode through the same row owner. Nightmare uses its accepted private captured
template continuation; it does not create a public `nightmare` power. Common
status value fields retain their existing unknown markers; status identity and
legal actions are represented. No hidden future draws or private template rows
are exposed.

Act2Ready publishes a healthy empty action mask, nonterminal/no bonus, with
`terminated=false` and `truncated=false`. A further step rejects at the supported
profile cutoff without freezing; decision/reset/close remain available. Batch
preflight and replay agree. Metadata distinguishes the 12 playable Act1 captures
and 16 actors/split offspring from three Act2 boss icons that describe only the
initialized successor map. It does not advertise Act2 gameplay or Blue Slaver.

Gym66's separate Ironclad combat-policy sidecar remains Ironclad-specific. M12
uses its full Run observation for combat; probing that sidecar returns an explicit
unsupported boundary without freezing, and metadata reports `combat_view_id: null`.
Existing M9/M10/Gym66 schemas and behavior remain separately preserved.

## Final native verification before milestone review

The 42-check integrated matrix completed with tested source stable: workspace
1,529 passed and one failed. The sole failure is the retained
`combat::tests::full_enemies::blue_slaver_and_two_fungi_encounters_construct_real_actors_across_ascensions`
Blue Slaver setup case. Every other workspace target passed. This is not a
workspace-green claim. Native M12 tests are 18/0 (14 core, 4 caller); public M12 tests
are 17/0. Preservation filters remain M11 62, M10 453, M9 32, M8 115, M7 16 and
M6 95, all with zero failures. Existing M9/M10 fixed CLI summaries are unchanged.
All declared old/new example/CLI invocations matched expected success/error exits,
including M12's six actual boss outcomes, two Death controls and budget/invalid
controls. The optional PyO3 binding source compiled with its existing extension
feature; no wheel was built, installed or imported for M12 and no refreshed Python
delivery is claimed. The subsequent latest-main reconciliation changed Markdown
only; all 539 tested non-Markdown source/test/dependency paths stayed byte-identical.

## Independent review and root acceptance

One fresh GPT-6 Luna MAX milestone review, distinct from authors and the planning
reviewer, passed 51 bounded probes and verified all 655 reviewed source hashes
and Git modes. Report SHA-256: `ea7c349b9d934cc732f8f5e59f3a42b4a76d5a9419fb8fdfd3fdd537225d74c6`. Root verified the
same source in an independent publication worktree: 1,529 workspace tests passed
with the sole retained Blue Slaver setup failure, and the seed-7/11 examples
produced all six declared Act2Ready outcomes. Only administrative Markdown
updates followed; runtime, tests and dependencies stayed byte-identical. This
accepts the finite profile, not full Silent pools, Acts2–4 gameplay or Python
extension delivery. The approved design and plan remain unchanged records.
