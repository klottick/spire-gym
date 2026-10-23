# M7 — Ironclad through captured Act2 into Act3 readiness

**Accepted at the declared captured M7 scope after ONE independent milestone review.**
Ironclad/A0, explicit captured content over native generated maps. This is not
complete standard pools, Act3 gameplay, all-character/Ascension coverage or
original-runtime validation. The final four-character/A0–20/Acts1–4/Heart target
is unchanged. [Written plan](../superpowers/plans/2026-10-23-m7-ironclad-act2.md).

## Run it

```sh
cargo run -p spire-core --example m7_act2 --locked --offline -- --seed 7 --scenario all
cargo run -p spire-core --example m7_act2 --locked --offline -- --seed 11 --scenario complete
cargo test -p spire-core --test m7_act2 --locked --offline
```

Scenarios `complete`, `death`, `all`; default seed7/all/budget4096. Budgets0/1
are unfinished nonzero exits, not gameplay losses, wins or truncations. A genuine
Death is valid executed gameplay and exits zero, labelled **not Act3 completion**;
faults/invalid arguments/unfinished budgets still fail execution. Deliberate real
nonattacking Death is separately labelled a diagnostic, never a game win.

## Observed normal-start outcomes — 2026-10-23

| Seed / Act2 boss | Accepted actions | Final HP | Reward | Actual outcome |
|---|---:|---:|---:|---|
|7 Champ|543|0|32|real Death at Act2 floor33; no chest/Act3|
|7 Automaton|560|105|34|nonterminal/no-action Act3Ready34|
|7 Collector|567|105|34|same scoped boundary|
|11 Champ|566|105|34|same scoped boundary|
|11 Automaton|575|105|34|same scoped boundary|
|11 Collector|593|105|34|same scoped boundary|

The user clarified that a hard-game loss is not a simulator failure. These are
the two seeds already named in the written plan; no search/substitution was used.
Seed11 supplies actual normal-start proof for every boss/reward/handoff alternative;
seed7 Champ's natural loss is explicitly tested/replayed, not tuned away, hidden or
called completion. Its reward32 is floor33 credit minus the real−1 Death event.
Winning paths include real ordinary/elite fights, Ancient Writing, actual chest
resolution and preserved public resources. A deliberately nonattacking Cultist
loss takes10 actions/floor1/reward0 (floor1 credit minus Death1), without injection.

## Profile and source ownership

`Run::new_room_scenario(...,RoomScenario::M7Act2Captured)` uses ordinary80/80HP,
99gold, five Strikes/four Defends/Bash, Burning Blood, three empty potion slots,
no keys. Supplied independent short-Neow eligibility and prior boss discovery
history select reproducible icons, not injected resources. Public examples take
offered Neow maxHP8 and genuinely offered relics/cards, never hidden simulator
clones, forced outcomes, easier-seed searches or game-value changes.

- Reuse the accepted [M5 Act1](m5-act1.md) topology/content and [M6 Act2](m6-act2.md)
  mechanics/captures. M5 still stops nonterminal/no-action at Act2Ready; M6 still
  stops after actual floor26 treasure Proceed at Act2PrefixComplete. Only M7
  continues all generated late-Act2 edges through boss33; no hidden edge pruning.
- Act2 ordinary visits: SphericGuardian, ThreeByrds, Centurion/Mystic, then
  Snecko/ChosenByrd/ThreeCultists/CenturionMystic cycle with last-two exclusions.
  Elites cycle Leader/Slavers/Book, rotated by the initialized icon, no immediate
  repeat. These are captures; HP/AI/summons/damage are actual native mechanics.
- Questions capture Ancient Writing, then Joust at gold≥50 otherwise Purifier,
  then ordinary Monster. Shops retain later-M5 stock/prices and carried purge
  cost. Ordinary chests retain first-unowned Strawberry/Anchor/BloodVial versus
  Sapphire. Elite captured relic cursor advances on offer even if skipped;
  exhaustion is an explicit unsupported boundary, never fabricated replenishment.
- Ordinary/elite rewards retain M5 card triples and Fire/Strength/Weak indexed by
  Act2-local settled visits. Gold10–20/25–35, burning/Collar/Emerald/relic outcomes
  depend on actual room origin, not species alone. Global combat history persists.
- Boss33 fights the initialized public icon: Champ, BronzeAutomaton or Collector,
  with actual Boss source context. Accepted entry discovers its canonical name
  once and consumes its selected private order entry; merely showing an icon
  discovers nothing. A0 never adds a second boss.
- Native boss victory waits for actual queued add deaths/escapes and Stasis return
  before the one Run settlement/census owner. Player Death exports card custody
  but grants no boss rewards/chest/act advance. Earned player healing is retained.
- Boss reward rows capture95–105 gold, `[Bludgeon,Impervious,Offering]` plus Skip
  for the rare-card child, and the current act-local potion capture. Claiming is
  optional; an opened mandatory child owns control until resolved. No elite
  relic/Emerald bonus on Boss source. Actual source Proceed enters separate
  BossTreasure34, credits that new floor once and dispatches RoomEntered.
- Boss chest retains BlackBlood/EmptyCage/SlaversCollar rows plus Skip. Owned or
  ineligible rows are Disabled, not refilled. Black Blood requires first-slot
  Burning Blood and replaces it once; with earlier Collar only BlackBlood and
  EmptyCage are offered. This is a declared captured choice set, NOT a standard
  three-new-relic roll. Open/take/skip/cancel/confirmation and mandatory Empty Cage
  removals are real source children. Zero/short-deck EmptyDeck remains a loss.
- Resolved chest source Proceed/confirmed skip validates the actual act2 source
  before replacing the completed1→2 transition frame. The same player requests
  ordinary A0 healing once (MarkOfBloom can suppress it), retains permanent cards,
  identity frontier, inventory/counters, gold/keys, purge cost and global history.
  Act-local progress resets; third-act map/boss order use separate native scopes.
- End at `Act3Ready`: act3/floor34, alive/nonterminal, no actions INCLUDING potion
  actions, no readiness/act bonus. The generated future map starts35/boss50, but
  M7 offers no Act3 move/encounter/reward. No-key total reward34; actual distinct
  keys add5 each. Boss wins/menus add0 beyond real new-floor credit; Death adds−1.

## Limits and evidence

The public caller uses only observations, legal menus and visible map topology;
replay checks accepted actions, decisions, maps and individual reward deltas.
Unvisited resolved rooms/future offers/AI/actor RNG/hidden card order remain private.
Invalid preflight rejects unchanged; operational faults retain accepted work and
the original error/owner, never rollback/reroll or masquerade as gameplay losses.

Existing represented card/relic/potion handlers are reused, not complete families.
Permanent cards captured here are Strike/Defend/Bash, IronWave/ShrugItOff/Anger,
Clothesline/PommelStrike/Disarm, Cleave/TwinStrike/Thunderclap and the three rare
boss captures. Relics are the starting BurningBlood/short-Neow Lament, finite elite
Lantern/Pear/Mango/MagicFlower/ToyOrnithopter/Strawberry/Anchor/BloodVial sequence,
ordinary/shop captures and BlackBlood/EmptyCage/Collar boss choices. Potion captures
are Fire/Strength/Weak. These lists are not standard full-game pools.
Reaper and unrepresented bottled/curse/local-copy flags remain unsupported. Generic
healing/custody tests do not certify those handlers. Source/reference evidence is
not original execution. Native HMAC/tie/traversal contracts do not promise original
Java PRNG/seed/cursor/comparator parity (RNG-001).

Initial seed7 callers completed Automaton but lost to both Champ and Collector.
Their traces and the bounded M7-only public priority regressions are preserved in
the local execution ledger. Existing M6 policy/game mechanics remained unchanged.
The user outcome correction ended further tuning to force seed7 Champ victory.
Current public checks pass8/0 plus one example reporting regression; internal M7
checks pass16/0. The ONE fresh independent GPT-6 Luna review found no Critical,
Important or Minor findings, assessed all five focus classes and recommended this
scoped acceptance. Controller verified all380 reviewed public files unchanged in
both worktrees and freshly reran public50/0, M7internal16/0, M6native90/0,
consumers43/0, other integrations21/0, reporting1/0, all seven examples, both seeded
scenarios, budget0/1 error paths and diffcheck. Only administrative acceptance text
changed afterward; no reviewed product bytes changed or second review was needed.

Six declined categories were explicitly evaluated, not dropped: original-runtime/
Java parity; wider roadmap; unsupported catalogs/handlers/flags; historical drafts/
whole-workspace; exhaustive seeds/routes/inventories/fault-state enumeration; and
the separate reference/performance/unrelated lanes. Acceptance certifies this
declared native profile and bounded evidence, not any of those broader claims.

The user-provided reference library lives in the separate main checkout under
`data/{cards,relics,enemies,powers}/`. It is reference input, not an implemented
registry or original-runtime proof; known source conflicts, provisional IDs and
nominal AI chances retain their qualifications. No files from those concurrent
reference lanes were edited/copied into this product snapshot.

The fresh pre-M7 core baseline was518 passes with seven historical Poison/Catalyst/
Artifact and BlueSlaver/TwoFungi failures; final core534/7 preserves the exact same
underlying assertions/messages. No whole-workspace-green claim. M7 `.29`/`.29.2`
are closed; M8 `.30`/planning-only `.30.1` are open/unassigned, no product start.
The accepted M7 snapshot is now locally committed as `2d9a47e` under the user's
separate pre-M8 commit request. The reference library is also available in this
checkout; see [its index](../../data/README.md). These administrative snapshots
do not change the reviewed simulator or imply a push, install or cleanup.
