# Shared power and intent vocabulary (spire-85d)

Every observation encoder publishes the same power and intent vocabulary. The
engine owns the registry; encoders only derive from it.

- `crates/spire-core/src/combat/public_ids.rs` holds `PUBLIC_POWERS` and
  `PUBLIC_INTENTS`. These list every power and intent the public combat view
  can publish, including the masked intent `hidden`. In debug builds, the
  `full_player` and `full_enemies` view builders (`combat/view.rs`) assert that
  each published name is registered. So any core, env or fuzz test that reaches
  an unregistered producer fails loudly.
- `crates/spire-env/src/public_vocab.rs` is the only spire-env owner. Its
  `install` overwrites the `power` and `intent` vocabularies in both descriptor
  builders, `run_schema::vocabularies` and `run_schema_m10::spec_from_vocabulary`.
  Its `power` and `intent` lookups are the only encoder lookups. Row 0 is
  padding; row `i + 1` is registry entry `i`.
- Per-milestone power and intent lists have been removed: the M9 and M10 tables,
  the M12–M25 unions, the policy-view closure lists, and the M20 player-power
  admission list. No encoder can now reject a registered power or intent.
- `public_vocab` tests walk the full registry through every encoder (M9, M10,
  policy Run-v3, combat policy view-v1/v2, M12–M25 Run, M16 Defect combat and M21
  Watcher combat). `native_fault_regressions.rs` replays the retained M10 Flex
  (`lose_strength`) and M22 burning-elite Gremlin Nob (`regenerate`) prefixes
  at the session level. Since spire-oto.27 the burning-elite buff publishes
  `regenerate` (RegenerateMonsterPower, shared with the Awakened One); `regeneration`
  is only the player's RegenPower.

Limit (review F3): completeness is enforced dynamically. The debug view guard
fires only on paths a test, fuzz or debug run actually reaches; there is no
structural per-producer check. A release build that publishes an unregistered
name still fails loudly, with an encoder `Input("unregistered public power")`
or `Input("unregistered public intent")` error surfaced as a delivery fault.
It never aliases the name to padding or corrupts the observation. A debug-build
native fuzz pass, or a check per producer table, would make completeness
durable.

When a producer adds a power or intent, append it to the registry. Every
descriptor and fingerprint below changes with it.

## CHANGES — 2026-11-02 descriptor fingerprints

All power and intent vocabularies became the shared registry tables: 137 powers
and 184 intents, plus padding. Every Run and combat-view descriptor fingerprint
changed. Power and intent indices changed for every schema except the leading
M9 prefix. Retrain or re-derive any consumer that embeds these indices.

| Schema | Schema ID | Old fingerprint | New fingerprint |
| --- | --- | --- | --- |
| M9 Run-v1 | `spire-gym-run-observation-v1` | `0893bce86e4b3abf11aa384c501503a91ae37f688ec605ab40b814ad365cb999` | `270163f700c7ca19556898d14c22ca8d641b0ef6f5c757d80d0ca7983989981c` |
| M10 Run-v2 | `spire-gym-run-observation-v2` | `7664874a7a483d1f2e0439ea7b319d1a4be4b9d276d0cb0f908654feda9393db` | `dccc7624c45898bc33e3fa682b620fd78ce70fe558d6db7624a888a7ed068c21` |
| M10 policy Run-v3 | `spire-gym-run-observation-v3` | `30905db1b7a520482d3ea8aa2e63ddb7f78b7c081de89d87268151f29cf4c7f8` | `28d92d89c437eea7fdc260240585bca466dee1fd74b9b3eb38513d2b34b0be64` |
| Combat policy view-v1 | `spire-gym-combat-policy-view-v1` | `02009686282c592f971eece3d5853eb54f2a482dde9dae1d0cf380ecdaf8fd78` | `08bb0e2244325c5ae33caea9b3c0f78ceda8cc79ea9100e77618b9d7a8d880db` |
| Combat policy view-v2 | `spire-gym-combat-policy-view-v2` | `02433813e1297244060d9852f342c6eb4c9c4d62f3dde9763ff82191867eceec` | `a27083bad27ffdd1fafea4c8de2b1580edad3c9d10200695694e15858b6122f7` |
| M12 Silent Act1 | `spire-gym-m12-silent-act1-observation-v1` | `366c572af8141146995e849acc737a33484623790e2b3d661e290fadfb6c9fac` | `ae0b995eade57ef1dccc39bd9cad6064d8bd33a1cfbc37a312e4b76ac01ceb20` |
| M13 Silent Act2 | `spire-gym-m13-silent-act2-observation-v1` | `3f1e21d50106b8ca71d7cd67a2de7bfa4074455dbbb459629d8e2ac24cf39cf8` | `cd7f78f7bc8ae8566f727f431f8a8d840303cdcb556173b31bf169cea8c34ecf` |
| M14 Silent Act3 | `spire-gym-m14-silent-act3-observation-v1` | `50a685721e5db56ed7a5332160f3747087f96ce738da020415b1427159377aee` | `72e75d8357ebbbba2b905f381262a99a4385c670d9fa20fa8cd008f4b90fb827` |
| M15 Silent Heart | `spire-gym-m15-silent-heart-observation-v1` | `254fb75b404fc558070468c6397324e68f962f23b33aca819b54fecf142ea11a` | `ab9fd38b68e7bd276526e136caf58fbbf6f0ef5916eb67fcba20fd91b54004f1` |
| M16 Defect combat | `spire-gym-m16-defect-combat-observation-v1` | `7bf5a854cd5fb4d05b1645960c1c39d1e23f937e1e8edda02aad13ba904a0183` | `754e496dadc127be2f8409a88f9e85393b629919e449e5c459a712a288b5ed58` |
| M17 Defect Act1 | `spire-gym-m17-defect-act1-observation-v1` | `2176f835eca1384b2070f3d7d001158ef1fead23be5d6d7eea138b57f6bff999` | `eeb45fcedd185710aeb0e5cb58ca619cc5927861cebda39b1612ff1500887a92` |
| M18 Defect Act2 | `spire-gym-m18-defect-act2-observation-v1` | `e097a8202f7cf7433f0ce57a39807bb74d370fba75a9aa84af65d726269564bb` | `6a4bb8526357f0264ae0cad8ccaad8015a0a3d360a678d236d94afe21a4ac967` |
| M19 Defect Act3 | `spire-gym-m19-defect-act3-observation-v1` | `149092e11432e182920b5f07e0728dac45e880296815d9b84fba9b6180c2817b` | `b2aa0b85a83a51171524a7487ab7cb3ed450e00811bd7f00c472e358a0220b15` |
| M20 Defect Heart | `spire-gym-m20-defect-heart-observation-v1` | `8c03211b4a3c589cfb55ba1078ddd447a444f3f53d8e9bfdab9ab85ab80e512d` | `3f11018081cfe0a2e9e69c87d57a57bd4043df2496c6d2c539dd74b0bf20fb56` |
| M21 Watcher combat | `spire-gym-m21-watcher-combat-observation-v1` | `271bd249d8da1a9db5e3e8372af8c3b8df3b7b8266bb0e3663744d52085e2c2c` | `99f84de7f3eb8a7dab49b3662c8ccb425a7b60e2ea9244938302b390458c890c` |
| M22 Watcher Act1 | `spire-gym-m22-watcher-act1-observation-v1` | `5d1c77658141b2dfefbe95e17b85899ad6b77fb370e402bcbbe771146821e087` | `2e4bcade98a3ab7beb402fca027aaed460bae6a26ff545754e3fa3b7e9e6b9bb` |
| M23 Watcher Act2 | `spire-gym-m23-watcher-act2-observation-v1` | `5d5524ff7f60575dea528d42808f7a0a5e67c41bfd0e3b36f7f83b663c9f9d7e` | `5b5f5ed8438b94d14bdb2cfa362229ddcb54df0a1091ba3ccb6ecfbdb87ff8bb` |
| M24 Watcher Act3 | `spire-gym-m24-watcher-act3-observation-v1` | `453b433051801b5cfbfd40bedbd70a8d2d53590e1ebffb4969c8cb17bc5b7564` | `3116abae686d6bc3aca5585c9d9fb87c89956543ac0213d324b2aa92ad9239b0` |
| M25 Watcher Heart | `spire-gym-m25-watcher-heart-observation-v1` | `d1197df6ed3d786ffe0413ec88beeb33db44873c8032f6c67d926a109af93e78` | `09355f51be8b906ecfe1bfd3465f400107ec387d4d5ebe1964f80a6810e77550` |

- M21 Watcher combat view at `combat_policy_capacities()`: `e7e49e88d50c947fde064a764153a5846ba3dc6c8298116f03e8126383b51e34` → `bbb3ad92bc04542773435bf30194a6ef7ac533807931832628bb41d7b47af861` (the table row above uses `RunCapacities::m10_default()`).
- M16 Defect combat at `combat_policy_capacities()`: `2b53f59fac1899e231300914b0b88aabc4e26add3492584de479b60e5fafce0a` → `6092fd366225add31b35bde2fb0f8984f943773f8399404d89de410ddf7618d7`.

Schema IDs are unchanged; only descriptor bytes changed. Diagnostic-only test
descriptors (M20, M22–M25 diagnostic specs) change with their parents. Dated
milestone profiles and reconciliation records keep the fingerprints accepted at
their time. Combat-v1 (`schema.rs`, legacy three-power slice) has no power or
intent name vocabulary and is unchanged.

## 2026-11-02 combat-card capacity (spire-oto.2)

`combat_policy_capacities().combat_cards` (and the `known_draw` rows that share it)
rose from 512 to the shared Run default of 1024. A generated CatalogCombat
three_shapes stall reached 515 visible rows (505 Repulsor Dazed) and faulted with
`Capacity`. Overflow is still a retained fault at the new bound. Schema IDs and
vocabularies are unchanged; only the row count and descriptor bytes changed.
The middle column is intermediate: the value with spire-oto.2 alone. The same change
also adds the spire-oto.8 phase columns below, so the last column is the shipped value.

| View | Previous fingerprint | Intermediate (spire-oto.2 alone) | Shipped (with spire-oto.8) |
| --- | --- | --- | --- |
| Combat policy view-v1 | `08bb0e2244325c5ae33caea9b3c0f78ceda8cc79ea9100e77618b9d7a8d880db` | `c70a6ef99bd638e0677a0d00287d64a22489625cc94ea288f09d795f08d3342c` | `4c73f7de04d7b2875f473f8034c1cbacf984ea2d52d0a9428289c30cf1dff463` |
| Combat policy view-v2 | `a27083bad27ffdd1fafea4c8de2b1580edad3c9d10200695694e15858b6122f7` | `2877c28eff6b785b31d388c521da4fbcf56439e1e0d688b8c8dfa40137c15293` | `5ba729fd1b10a067fb8fc5d777d78b47fc9526ec0af0030d6868a32b52a7824b` |
| M21 Watcher combat at `combat_policy_capacities()` | `bbb3ad92bc04542773435bf30194a6ef7ac533807931832628bb41d7b47af861` | `ce4e099fc33298229852508f34d26de790774f28c063ab5d2530f0b401e1d122` | `46fef9f50c0e3212dcd0b3d244cdae663c633713e40f578976c06303f267d771` |
| M16 Defect combat at `combat_policy_capacities()` | `6092fd366225add31b35bde2fb0f8984f943773f8399404d89de410ddf7618d7` | `feeffa64c463aee521944f9a85e53446289226fd18518a45c5ebb72581236ef2` | `feeffa64c463aee521944f9a85e53446289226fd18518a45c5ebb72581236ef2` (spire-oto.8 does not change M16) |

The same chunk (spire-oto.7) makes the Watcher public value owner report Deus Ex
Machina as cost kind `unplayable`, so the card-metadata descriptor fingerprint
moved from `cb753725f58cec83633d9a51eb4941f622523843fad6ecff0874f79016308112` to
`b60b66c69bce54a6652442e54bc42a2a446d05ec7efa39bb1b11cf4b8c6ff5fb`, and a visible Deus Ex
Machina row encodes the unplayable cost kind.

## 2026-11-02 interrupting selection phase (spire-oto.8)

Any selection may now suspend whatever continuation is resolving: an enemy turn,
a potion and its autoplayed child (Snecko Oil, Distilled Chaos), a card's work after
its own selection (Burning Pact's draw), or the turn-start queue after the after-draw
relic phase. Melange's Scry, which opens whenever a shuffle triggers it as in the
original, is the main source. Every pending selection stores one suspension record
when it opens: the interrupted phase, the in-play card, the top play frame with its
card or potion owner, and the once-per-resolution resolver marks already spent.
Validators check ownership through that record, and every commit path, modal or
not, restores exactly it before the interrupted queue resumes.
`CombatPublic` gains `phase` and `selection_resume_phase` for every
profile. The Watcher `combat_state` group appends `phase,selection_resume_phase`
(M21–M24 at the end; M25 after `previous_card_type`) with the `combat_phase`
vocabulary padding=0, decision=1, selection=2, player_action=3, enemy_turn=4,
round_end=5, player_start=6, terminal=7. Combat-policy lifecycle metadata carries
the current and resume phases. The resume phase is that record's phase. M9, M10 and M12–M20 tensors are unchanged.

| View | New fingerprint |
| --- | --- |
| Combat policy view-v1 | `4c73f7de04d7b2875f473f8034c1cbacf984ea2d52d0a9428289c30cf1dff463` |
| Combat policy view-v2 | `5ba729fd1b10a067fb8fc5d777d78b47fc9526ec0af0030d6868a32b52a7824b` |
| M21 Watcher combat | `46fef9f50c0e3212dcd0b3d244cdae663c633713e40f578976c06303f267d771` |
| M22 Watcher Act1 | `ead825f19203b3da9df19c39e79c7fb5d5f2efaa6d35c2971587fed7e0af1d9a` |
| M23 Watcher Act2 | `13dbacc4553ad50bef32e5e23ee05d0c6257aad55e0b5b2a134332784fed7613` |
| M24 Watcher Act3 | `3044b3d2b3ce6edc4343d393cdf1d4eee7733a94752fcd5aac9dd1a021da0332` |
| M25 Watcher Heart | `a1e637bf29fc28620a9c2e487cfd950462d8680967eb68bb9620e3a38bc5ca67` |

The `CombatPublic` Debug text prints the two fields only while a selection is open,
so the accepted M13 Silent caller transcripts (`m13_caller_transcripts_unchanged_by_m14_extension`)
were re-pinned. With those field lines removed, each of the six transcripts hashes
to its previous pin; nothing else changed:

| Case | Previous transcript SHA-256 | New transcript SHA-256 |
| --- | --- | --- |
| 0 | `370ea52171b0c7958d6927cecae5327b82b151504448f51aeff8f758f7d902fa` | `b0242fa54e673cc2b5d4240beb30cbedf9763e8d02e19a3e369e4414a0242622` |
| 1 | `d715a1e13a6d08472918b9bd637b9ad308437ad1e5cc6cc7128fcfb87c887165` | `d6a2e5c262ad0049e6384fde4d316f414f27671c2fb554ca79405e5334012588` |
| 2 | `c30e3d6336a9d0059cee1beeeb19c83f6c767d80cd13483a7e3f152b2005165e` | `2b25945623c55c78b74eba0cfa0fc0b8d46535c2aacb2c509538d23d7f5575da` |
| 3 | `749e206bc53aace53973bea3a5f91305a31baf1d0e8f28265ea5f863b92ced47` | `907f70ed5071e1d4f41c608aa65d6b0022270d553af87eac865cdd614dc885d4` |
| 4 | `2c42be5086c090b30d5d379512c7c559d809f8055a58e51c4483db0e05d89a34` | `325bfaa6615757e24f66bd8c57c5837c9c3bfd68a624d23b95cbce5bfef7b123` |
| 5 | `700cb47702ab4ce25ef72d600288798933dac99d7da7e624bc7eb22d7ff7b09e` | `9b8330c1788027c7e35d2ae73c3f19a341bd52dc79623d5304d3f87e29836c33` |

## 2026-11-02 turn-start order and out-of-hand card values (spire-oto.11, .13, .14, .19, .20)

No observation descriptor fingerprint changes: no observation name, vocabulary or row
count changed. The opt-in [card metadata](card-metadata.md) descriptor moved from
`b60b66c69bce54a6652442e54bc42a2a446d05ec7efa39bb1b11cf4b8c6ff5fb` to
`effbb810d345e906f6e20b47bd972e35d72ef48141378d54bd28b54afac5f406`. Mind Blast, Stack and
Blizzard now resolve their static cost and target, so they left `unresolved_cards`.
Behaviour that observations and transcripts can show changed as follows.

**Turn start.** Each relic and player power runs at its source hook
(`source-check-turnstart-hooks-v1`), for every character:
- **Turn 2 and later:**
  1. start-of-turn (atTurnStart) relics, such as Art of War, Happy Flower, Mercury Hourglass
     and Lantern;
  2. card, power and orb start callbacks, including Mayhem's wrappers;
  3. the recharge, synchronous when the turn draw is built;
  4. the draw;
  5. post-draw (atTurnStartPostDraw) relics, such as Pocketwatch and Gambling Chip;
  6. post-draw powers, such as Brutality and Devotion;
  7. Mayhem's play-top actions.

  All the queued work of steps 1 and 2 resolves before the draw. Start-of-turn relics
  were raised after the draw before.
- **Turn 1 (battle start):**
  1. atBattleStart relics;
  2. start-of-turn relics;
  3. post-draw relics;
  4. card, power and orb callbacks.

  Each step's bottom work follows the previous step's. There are no post-draw power
  callbacks on turn 1. Before, the post-draw relics ran before the start-of-turn relics.
- **Recharge:** the recharge law, then the power onEnergyRecharge callbacks in power
  order (Collect, Energized, Energized Blue, Deva Form's growing gain). Collect's Miracle
  is now created at the recharge, not with the start callbacks.
- **Energy Down:** its loss is queued with the other start-of-turn power work (it still
  persists), after the start-of-turn relics' queued energy. Before, it was subtracted at
  the recharge.

**Mayhem.** Mayhem plays the card on top after the turn's draw, after the post-draw relics
and powers. Its target is picked after the start-of-turn relics resolved; a lethal Mercury
Hourglass ends combat before any target is drawn. X is the energy when its play-top action
queues the card, so it includes Art of War, Happy Flower, Berserk, Deva Form and Ice Cream.

**Relic queue sides.** Battle-start and start-of-turn relics queue their work at the top
or the bottom as the source does:
- **Now top:** Akabeko, Clockwork Souvenir, Du-Vu Doll, Girya, Mutagenic Strength,
  Pantograph, Sling of Courage, Thread and Needle, Preserved Insect and Philosopher's Stone.
  Among top relics, the later relic's work runs first.
- **Now bottom:** Bag of Preparation, Ring of the Snake, Bag of Marbles, Red Mask, Twisted
  Funnel, Mark of Pain, Damaru, Runic Capacitor, Inserter, Emotion Chip and Self-Forming
  Clay.
- **Moved slot:**
  - Cracked Core channels at atPreBattle, in relic order with Symbiotic Virus and Nuclear
    Battery.
  - Teardrop Locket enters Calm at battle start, for every holder.
  - Pen Nib and Enchiridion fire once per combat. Watcher contexts fired them twice.
- **Red Skull:** its battle-start bloodied check is queued at the bottom and reads HP when
  it resolves, after top-queued heals such as Blood Vial and Pantograph. When it activates,
  it still adds +3 Strength directly. Pending-queue Debug text shows the new
  `RelicEffect::RedSkullCheck`.
- **Consequence:** with Clockwork Souvenir acquired before Mutagenic Strength, Strength Down
  now lands before the Artifact.

**Autoplay bursts.** Havoc, Mayhem and Distilled Chaos share one top-card action:
- an empty draw pile is shuffled from the discard and the action retries (new for
  Mayhem and Distilled Chaos);
- every top-card action of a burst runs before the first queued card is used.

**Card values.** The top-card action refreshes the card's values (Stack, Mind Blast)
before queueing it, and so does Necronomicon's copy. Blizzard counts Frost when it is
used.

**Targets.** Omniscience's card and its repeat each pick a fresh live random target
when they are dequeued; before, both were drawn at selection commit.

**Energy cap.** Energized saturates at 999 energy for every character; before,
Ironclad and Silent faulted above 999.

**Cleanse.** The shared cleanse classifier covers every public power. Shackled is a
debuff. Orange Pellets also removes negative Focus and Constricted.

The internal effect shapes `MayhemPlayTopCard`, `OmnisciencePlay` and
`OmniscienceRepeat` changed. Pending-queue Debug text, state fingerprints and replay
transcripts that pass through a Mayhem turn or an Omniscience play differ from
earlier ones; replays recorded before this change are not expected to reproduce
them.

## 2026-11-02 power-list order and potion-action fixes (spire-oto.10, .12, .15, .21)

No observation descriptor fingerprint changes: no observation name, vocabulary or row count
changed. Behaviour that observations and transcripts can show changed as follows.

**Power-list order.** Applying a new power stable-sorts the owner's list by source priority, so
each power follows every power of equal or lower priority (`source-check-power-priority-v1`). The default priority is 5,
with these overrides:

| Priority | Powers |
| ---: | --- |
| 0 | Confused |
| 6 | Double Damage, Pen Nib |
| 10 | Frail |
| 20 | Draw Card |
| 25 | Establishment, Tools of the Trade |
| 50 | Flight, Reactive |
| 75 | Intangible |
| 99 | Invincible, Weak |
| 105 | Constricted |

So a monster's Weak, for example, is listed after its Poison or Vulnerable, whichever was
applied first. Adding a new power re-sorts the whole list (a stable sort), and stacking a power
keeps its place. Two Strength paths append without sorting, as the source's direct add does:
Philosopher's Stone, and Red Skull's battle-start Strength. Such an entry keeps its place only
until the next new power re-sorts the list. For example, a Byrd with Philosopher's Stone shows
[Flight, Strength]; after Poison it shows [Strength, Poison, Flight].

**Start-of-turn power callbacks.** The start-of-turn and post-draw power callbacks run in that
list order, for every character. Before, they were grouped by engine module. Nightmare's copies
take part: each Nightmare holds its own Night Terror list entry, which observations do not show,
so its copies follow powers applied before it. Around them:
- Divinity's exit runs before them.
- The orb start pass (start passives, Gold-Plated Cables) runs after them.

Magnetism, Creative AI and Hello World choose their card when the callback runs, under every
profile. Before, Magnetism outside the M20 profile chose its card later, when the card was
created. End-of-turn power slots are unchanged (spire-oto.22).

**Fixes.**
- A Meditate played by Distilled Chaos ends the turn after the potion's remaining work.
- A Distilled Chaos burst continues when a burst Alchemize refills the potion's slot.
- Well-Laid Plans opens no retain choice when Runic Pyramid or Equilibrium already retains
  the hand. A retain choice adds to any retention set earlier that turn.

## CHANGES — 2026-11-09/10 Standard profile (M34)

No existing descriptor fingerprint changed. The captured M9–M25, policy Run-v3 and Gym66 combat-view encoders kept their
bytes, their M10 `intent_hits` table and their Heart cap literal 300. A native test proves the table and the cap equal the
engine at A0. The new Standard schema `spire-gym-standard-run-observation-v1`
([Standard Run API](standard-run-api.md)) uses this shared registry as follows:

- **Powers** use the shared `PUBLIC_POWERS` order (137 + padding). **Intents** use `PUBLIC_INTENTS` (184, ending in `hidden`)
  followed by the seven `EVENT_INTENTS`. M34b pins both Standard families as frozen v1 lists, by length and SHA-256
  (`standard_vocab_manifest_is_append_only`). A power or intent added to the shared registry later is appended at the END of
  the Standard family, after the event intents, so no Standard index moves. Its Standard row may then differ from its row in
  the other encoders, which append it inside the shared block.
- **Intent category** (`intent_category`, 17 source `AbstractMonster.Intent` values + padding) is new and Standard-only. It is
  keyed per (monster, move), with ascension conditions only where the source has one (Darkling Harden A17, Deca Square A19).
  The census oracle is the checked-in `crates/spire-core/src/combat/tests/m34a_intent_census_v1.json`.
- **Intent hits and Heart cap** (`enemies.intent_hits`, `combat_state.heart_cap`) are Standard-only live engine values:
  - hits are the selected move's planned damage applications, including AI memory (Book of Stabbing's count grows to
    `i32::MAX`);
  - the cap is the present Heart's Invincible cap (300, or 200 from A19);
  - hidden intents (Runic Dome) leave category, kind, damage and hits unknown.
- The other open Standard families (cards, relics, potions, events, encounters, actors, rooms, choice kinds, selection
  operations) come from the append-only manifest `standard-vocab-v1`, not from this registry.

M34c (exact checkpoints, replay to a decision) adds no observation column and changes no fingerprint.

## CHANGES — 2026-11-10 public knowledge and the Standard combat view (M34s-a)

Descriptors: the Standard Run fingerprint changes once, `efbc1253…` → `bf8e1d111bbf2171594231b99d8d6b701244249abfbb90a11e86c4b4a405f7ae`,
and the Standard rules id moves to `spire-gym-native-standard-rules-v2` (gameplay changes G-a4 and R-a1 below). The new
Standard combat view `spire-gym-standard-combat-view-v1` has fingerprint
`5d8147e13ded030c7faa501a8202eb2588915c2e1330b30cac5b06afc9f9916e`. No captured (M9–M25), policy Run-v3, combat-view v1/v2,
Gym66 or M21 descriptor or fingerprint changes, but some of their VALUES change: values change only as listed below, on
exactly the surfaces named (source-located census: M34s-a evidence `i1-census/surface-census-v1.md`):

1. **Standard draw rows** (Standard only). The zone family gains `draw` (appended to `standard-vocab-v1`). Every unknown
   draw-pile card is a `combat_cards` row in zone `draw`, `zone_position` −1 (known 0), after every other row, sorted by
   the canonical card key (published row without zone and zone_position, `permanent_row` last); ties are interchangeable.
   The `zone_position` domain is now −1..; semantics `draw_rows`.
2. **Card lifecycle columns** (Standard only), appended to `combat_cards`, each the player-visible fact it encodes:
   `autoplay_free` (an autoplay waiver; displayed cost 0), `free_play_once` (a one-use Forethought/Setup waiver; displayed
   cost 0), `rolled_cost` (the Confusion/Snecko Oil roll shown when drawn), `modified_combat_cost` (a cost changed for this
   combat, e.g. Madness, Enlightenment, Force Field), `hp_loss_discount` (Blood for Blood's HP-loss discount) and
   `cost_zero_this_turn` (Infernal Blade's cost 0 for the turn); semantics `card_lifecycle`.
3. **Intangible display** (every surface where the player can have Intangible while an attack intent is visible:
   the Standard Run observation, the Standard combat view and `standard-combat-v1` sessions through Wraith Form, Ghost in a Jar, Incense Burner and Council of Ghosts' Apparition; the
   `M15SilentHeart` Run through Wraith Form and Ghost in a Jar. Not reachable on M9, M10 (Run-v2, policy Run-v3, combat
   sidecar), M12–M14, M16–M25, the M21 Watcher combat sessions or Gym66 `ironclad-combat-v1/v2`). `intent_damage` applies the player's
   Intangible cap (1), as the original `calculateDamage`. Damage dealt is unchanged.
4. **Back Attack display** (every surface that admits Shield and Spear: the Standard Run observation, the Standard combat view and `standard-combat-v1` sessions; the M10 Run-v2, the M10 policy
   Run-v3 and the M10 combat sidecar (combat-policy view v1); the M15, M20 and M25 Runs; the M21 Watcher combat sessions;
   and **Gym66 `ironclad-combat-v1` (combat-policy view v1) and `ironclad-combat-v2` (view v2)**, whose entry catalogs admit
   `shield_and_spear`. Not reachable on M9, M12–M14, M16–M19, M22–M24). The displayed ×1.5 is truncated on the
   unfloored value before the floor, as the original `calculateDamage` (e.g. Weak and Back Attack show 5 where 4 is dealt;
   the dealt value is unchanged).
5. **Gameplay, disclosed (G-a4).** A card played by a non-purge autoplay (Havoc, Mayhem, Distilled Chaos, Omniscience's
   first play) no longer keeps its free-play waiver after its use, as the original `UseCardAction` clears it; if that card
   comes back into play it costs its normal energy. Observable where such a card can return: the Standard Run observation, the Standard combat view and `standard-combat-v1` sessions; the M10
   Run-v2, policy Run-v3 and combat sidecar, and **Gym66 `ironclad-combat-v1/v2`** (Havoc, then Exhume); the M25 Run
   (Omniscience, then Tantrum's shuffle-back). Not reachable on M9, M12–M24 or the M21 Watcher combat sessions.
   A retained record of those surfaces that reaches such a replayed card now stops with a typed `IncompatibleReplay`
   (rules ids are unchanged); none of the 323 retained frozen-replay inputs does.
6. **Selection order, disclosed (R-a1).** Secret Technique and Secret Weapon list their draw-pile candidates in the shared
   grid order (card id, instance), as Seek and Omniscience already did, instead of physical draw order; their candidate
   action indices change on the Standard Run observation, the Standard combat view and `standard-combat-v1` sessions (the only surfaces admitting them; no captured, M21 or Gym66 profile does).
