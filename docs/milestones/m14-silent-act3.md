# M14 finite native Silent through Act3

**Status: accepted at the declared finite native scope after one fresh independent final review and root acceptance.**
Normal progression across the declared42 cases verifies actual same-visit entry
and records for all three Act3 bosses and real ordinary/elite rooms. All42 normal
outcomes remain Death; canonical normal completion is **unproven**. Under the
[user-approved supplied-start amendment](../superpowers/plans/2026-10-26-m14-supplied-full-boss-approval.md),
all six separately labelled supplied-loadout full-HP boss fights reached actual
Victory, Proceed, four ending dialog stages and Act3NoKeys. No normal win is claimed.

Implementation evidence was assembled against accepted M13 commit
`c1942d20586680c16bd9b5d46c84c5c200be42db`. M14 was accepted after the fresh independent final review and root disposition against the frozen source. The [approved design](../superpowers/specs/2026-10-26-m14-silent-act3-design.md)
and [five-task plan](../superpowers/plans/2026-10-26-m14-silent-act3.md) retain
their original bytes. The [approval record](../superpowers/plans/2026-10-26-m14-silent-act3-approval.md)
and [shared coverage amendment](../superpowers/plans/2026-10-26-silent-normal-run-coverage-amendment.md)
resolve the predecessor condition and replace the historical per-route final-boss gate.

## Finite implementation boundary

`RoomScenario::M14SilentAct3` composes the accepted M13 normal Silent A0 start
and Acts1–2 with the shared M9 Act3 map, rooms, combat settlement and ending.
The canonical start is 70/70 HP, 99 gold, five StrikeGreen/five DefendGreen,
Survivor/Neutralize, Ring of the Snake, three empty potion slots and no keys.
It chooses actual short Neow and proceeds through generated maps. M12 still
stops at Act2Ready; M13 still stops at Act3Ready. M14 is a separate profile,
not an alias of either predecessor or the Ironclad M9/M10 profiles.

The permanent universe is the 19 accepted M12 green card kinds plus Parasite:
StrikeGreen, DefendGreen, Survivor, Neutralize, PoisonedStab, Backflip,
BladeDance, DeadlyPoison, CloakAndDagger, DaggerThrow, NoxiousFumes, Accuracy,
WellLaidPlans, QuickSlash, Reflex, Catalyst, Deflect, Tactician, Nightmare,
Parasite. Ordinary reward/shop producers use the separate green rows; Parasite
is a permanent curse acquired by actual Writhing Mass journal work. The six
generated/temporary kinds are Shiv, Dazed, Burn, Slimed, Wound and Void. Shiv is
an Attack. Awakened One's `awa_sludge` creates Void at BottomDraw; no Sludge
CardId is introduced.

The 18 relic kinds are RingOfTheSnake, NeowsLament, Lantern, Pear, Mango,
SneckoSkull, ToyOrnithopter, Strawberry, Anchor, BloodVial, BagOfPreparation,
EmptyCage, SlaversCollar, Vajra, OddlySmoothStone, BagOfMarbles, BronzeScales
and RedMask. The four potions remain Fire, Strength, Weak and Dexterity.

| Act3 producer | Captured rows |
|---|---|
| Ordinary card reward | Local settled-combat ordinal modulo3: PoisonedStab/Backflip/BladeDance; DeadlyPoison/CloakAndDagger/DaggerThrow; NoxiousFumes/Accuracy/WellLaidPlans |
| Potion reward | Fire/Strength/Weak at the same local ordinal |
| First local shop | QuickSlash/Reflex/Catalyst at50/75/75; unowned SneckoSkull at150; Dexterity at50 |
| Later local shop | Deflect/Tactician/Nightmare at50/75/150; unowned BagOfPreparation at150; Fire at50 |
| Elite relic | Lantern, Pear, Mango, SneckoSkull, ToyOrnithopter, Strawberry, Anchor, BloodVial, Vajra, OddlySmoothStone, BagOfPreparation, BagOfMarbles, BronzeScales; skip owned rows |
| Chest43 | First unowned Strawberry/Anchor/BloodVial, exclusive Sapphire, Proceed; all-owned relic rows disabled |
| Questions | Falling, Tomb of Lord Red Mask, then actual Monster capture |
| Boss50 Victory | Actual Proceed/dialog ending, no gold/card/potion loot or third boss chest |

Shared affordability, belt, ownership, purge and permanent-instance rules apply.
Elite publication commits its cursor even if the offer is skipped. Exhaustion
returns `RunError::Unsupported("captured Silent Act3 elite relic sequence exhausted")`
without another cursor advance. Earlier inventories can consume the finite
stock; five further unowned elite offers are not guaranteed. Accepted combat
sources remain operational faults when publication fails; they are not Death
or completion. Routes are not pruned and stock is not refilled to hide this limit.

The 16 Act3 encounter IDs are OrbWalker, ThreeShapes, SpireGrowth, Transient,
FourShapes, Maw, SphereAndTwoShapes, JawWormHorde, ThreeDarklings, WrithingMass,
GiantHead, Nemesis, Reptomancer, AwakenedOne, TimeEater and DonuAndDeca.
JawWormHorde retains its earlier setup-table index and RNG key.

The actual Act3 actor union has 20 species: OrbWalker, Repulsor, Exploder,
Spiker, SphericGuardian, JawWorm, Darkling, SpireGrowth, Transient, Maw,
WrithingMass, GiantHead, Nemesis, Reptomancer, Dagger, Cultist, AwakenedOne,
TimeEater, Deca and Donu. SphericGuardian/JawWorm/Cultist are reused; 17 are new.
Including shape alternatives, summoned Daggers and boss partners yields 54
cumulative species: M12's16, 21 additional Act2 and 17 additional Act3. Act2
itself has22 species including reused Cultist; overlapping act counts are not
added as disjoint sets.

## Endings and supplied diagnostics

Real boss50 settlement exposes Act3BossComplete, then Proceed and actual dialog
choices. Missing any key or disabled Act4 takes the fourth dialog to
`RunTerminal::Act3NoKeys`, once, with terminal reward contribution0. This gives
no floor51 or Heart bonus. Genuine Death and EmptyDeck remain distinct losses.

With all keys and Act4 enabled, the shared core stops after dialog3 at
Act4Ready, act3/floor50: alive, nonterminal, zero actions, no heal/new act or
readiness bonus. All-key lifecycle coverage is supplied diagnostic evidence,
not a normal-start key/Heart trace. M14's session exposes a healthy inspectable,
resettable cutoff; stepping rejects with
`UnsupportedBoundary("M14 stops before the Act4 door")`. M9's unsupported
publication and M10's actual door behavior retain their prior contracts.

Focused supplied Run diagnostics exercise actual Act2 boss settlement and
successor initialization, every Act3 capture, local stock and skipped/exhausted
cursor custody, Falling duplicate-instance removal/EmptyDeck, Tomb all-gold
including0/owned222, actual Writhing permanent gain through Victory/Death and
paid purge, all three actual boss lifecycles across eight key combinations,
and first-fault retention at gain/event/reward/ending frontiers. They do not
supply missing normal-start boss coverage.

## Native interface

| Identity | Value |
|---|---|
| Profile | `M14SilentAct3` |
| Observation | `spire-gym-m14-silent-act3-observation-v1` |
| Replay | `spire-gym-m14-silent-act3-replay-v1` |
| Rules | `spire-gym-native-m14-rules-v1` |
| Content | `spire-gym-m14-silent-a0-acts1-3-captured-v1` |

`RunSession::new_m14`, `RunBatch::new_m14`, `m14_run_observation_spec` and
`encode_run_m14` reuse the session, row encoder, portable semantic replay and
ordered scheduler. Default capacities remain `RunCapacities::m10_default()`.
The profile does not advertise the Ironclad combat-policy sidecar
(`combat_view_id` is null).

The existing `boss_history` field gains a complete typed triple, with all history
installed before Run construction:

```json
{"boss_history":{"m14-silent-act3":{"act1":"guardian","act2":"champ","act3":"awakened-one"}}}
```

Act3 alternatives are `awakened-one`, `time-eater` and `donu-and-deca`.
All27 typed combinations are supported; malformed/partial/extra/unknown/wrong
profile histories reject before construction. Old scalar histories, M13 pair
JSON, existing IDs/fingerprints and canonical source boss-history spellings
remain separate contracts. Rust exhaustive matches need an additive enum arm.

## Current supplied full-boss ending evidence

The approved `cfg(test)` fixture builds each complete boss history before
construction and supplies the declared owned resources before the first
published decision or combat entry. It obtains fourteen cards through the
shared mutation owner before removing the twelve original starter instances,
acquires eleven relics through the native item owner in addition to Ring of
the Snake, validates the three Dexterity Potions, and stages a generated
Act3 floor49 predecessor. A real Move enters the full-HP floor50 boss.
The fixture returns after entry without further state writes. All subsequent
actions come from the unchanged v5 public caller through `Run::step`.

The exact inventory is14 cards, all upgraded once: NoxiousFumes×2,Catalyst×2,
Backflip×3,Deflect×3,DeadlyPoison×2,CloakAndDagger×1,WellLaidPlans×1.
RingOfTheSnake plus Pear,Mango,Strawberry,SlaversCollar,SneckoSkull,
OddlySmoothStone,BagOfPreparation,Lantern,Anchor,BronzeScales,RedMask produce
101/101 HP and ordinary native startup hooks. Gold is99, keys are all false,
Act4 is enabled, and the caller budget is8192. No combat energy, hand, enemy HP,
outcome or RNG is patched. Boss starts are AwakenedOne300 plus two native
Cultists, TimeEater456, and Donu/Deca250 each, with every actor at full HP.

All six cases ran once in the declared order, with every Result, initial
resource/public boundary, actual entry transition, full Caller trace and
endpoint retained separately from the normal42 evidence. Six unique read-only
files total54,591,812bytes. There were no faults, operational errors or budget
exhaustions. Every case has a matching same-visit Victory record, actual offered
Proceed, dialog stages1–4, zero ending reward and act3/floor50/Act3NoKeys.

| Seed | Act1 / Act2 / Act3 history targets | Result | Caller actions |
|---|---|---|---|
|7|Guardian / Champ / AwakenedOne|Act3NoKeys|141|
|7|Hexaghost / BronzeAutomaton / TimeEater|Act3NoKeys|81|
|7|SlimeBoss / Collector / DonuAndDeca|Act3NoKeys|94|
|11|Hexaghost / Collector / AwakenedOne|Act3NoKeys|150|
|11|SlimeBoss / Champ / TimeEater|Act3NoKeys|42|
|11|Guardian / BronzeAutomaton / DonuAndDeca|Act3NoKeys|82|

These are six **supplied-loadout full-boss wins**, including both seeds for each
boss alternative. They do not establish canonical normal-start completion or
a public supplied-start training API. The qualification is **normal boss entry
verified; supplied-loadout full-boss win/end verified; canonical normal
completion unproven**. Existing normal results, caller policy, engine, encoders,
original approved plans and all earlier candidate evidence remain unchanged.

Fixture checks recorded five behavioral failures against the canonical start,
then5/0 with the approved fixture. The completion oracle rejects missing or
mismatched records, Death, partial/out-of-order dialog, wrong endpoints and
bonus/Heart advancement. All accepted M13 full Debug transcript digests passed.
A source/mode/loadout/case/rule freeze preceded all six plays:703 files,576
non-Markdown inputs, full root
`2b7d117263e784447a0ad00695e54627bc255562c82d3af72feb1a32ebeef2fd`,
non-Markdown root
`3c6f993614380dfa1c7e63cfb085b607ea3ec3c42657fc7ca4a197db27d40472`.
The single final stable workspace matrix recorded **2,001 passed,1 failed,
0 ignored across62 result blocks**. The sole failure is the previously reported
`combat::tests::full_enemies::blue_slaver_and_two_fungi_encounters_construct_real_actors_across_ascensions`,
which rejects Blue Slaver with `Registry("encounter setup not implemented")`.
It is outside this finite M14 profile and remains reported; the full workspace
is not claimed green. M14 core caller target88/0, native batch86/0 and native
schema5/session4/replay4 pass. Embedded shared helper tests occur in multiple
targets; those repeated executions are not additional unique mechanics coverage.

All42 normal results, action counts and endpoints match the retained first
suite. For all42 cases, native single sessions and batches with workers1/2
match the complete core transcript, terminal, rewards, observations and portable
semantic replay. Cases are processed one at a time. The six supplied traces
also reproduced byte for byte in a separate final-preservation directory.
All six accepted M13 full Debug digests pass, all15 predecessor CLI commands
match accepted exits and ordered semantic lines, and both prescribed M14 CLI
commands reproduce all six original v5 normal summaries exactly. The final
matrix took4:34 wall time; measured maximum resident set size was367,252KiB.
No source policy, fixture, gameplay, encoder or original approved plan changed
after the supplied freeze; only this qualified report changed. No Python
extension delivery or native-session supplied-start API is claimed.

Canonical normal-start completion remains unproven. M14 exposes no public supplied-start API or Python-extension delivery claim, and does not claim full Silent pools, an Act4 door or a Heart encounter. M15 remains unstarted.

## Retained normal-start and earlier verification evidence

The sections below retain the results and disposition at each historical
freeze. Their INCOMPLETE labels describe those earlier gates; the supplied
amendment above changes only the completion requirement and does not rewrite
any normal outcome or missing-coverage history.

### Frozen-v5 42-case suite — original completion gate failed

The user approved the independently reviewed42-case amendment under proposal
SHA-256 `a9e4a8d06361f9a97b703e1f03f7ea2d380ba1e4f02dfe21f1b1d2e6f05ab8f2`.
It retains the original six inputs and adds exactly seeds12–23, three triples
per seed, selected before outcomes. Seed7 and even12–22 use Guardian/Champ/
AwakenedOne, Hexaghost/BronzeAutomaton/TimeEater, SlimeBoss/Collector/DonuAndDeca.
Seed11 and odd13–23 use Guardian/BronzeAutomaton/DonuAndDeca,
Hexaghost/Collector/AwakenedOne, SlimeBoss/Champ/TimeEater, in that order.

One shared compiled case table drives core and native test enumeration. The
v5 policy, priorities/ties, gameplay, canonical resources, RNG and8192 budget
remain byte-identical. Only enumeration, assertions and reporting changed.
Table tests recorded1 pass/2 behavioral failures against the old six rows,
then3/0; aggregate oracle tests recorded2 passes/1 budget failure, then3/0.
The assertion now explicitly rejects budget exhaustion even when other rows
supply coverage. Missing boss species, ordinary/elite coverage and completion
remain failures. All six accepted M13 full Debug digests passed.

All42 rows ran in declared order, without an early coverage-success stop.
Every Result/full public Caller transcript was written into a new empty retained
directory with `SPIRE_M14_TRACE_DIR` set, before derived assertions. Verification
found42 unique nonempty complete files totaling3,651,975,983bytes, with SHA-256
receipts. The original six full files are byte-identical to retained v5 traces.
The30 earlier v1–v5 traces remain separately immutable.

Every cell below is a genuine **Death**, with no operational error or budget
exhaustion. Cells show accepted actions and endpoint act/floor; columns identify
the Act1 target and its predeclared triple above. This reports all42 outcomes.

| Seed | Guardian actions, endpoint | Hexaghost actions, endpoint | SlimeBoss actions, endpoint |
|---|---|---|---|
|7|1157, Act3 floor45|1216, Act3 floor47|1245, Act3 floor47|
|11|991, Act3 floor41|1048, Act3 floor45|927, Act3 floor40|
|12|1170, Act3 floor47|1260, Act3 floor50|1115, Act3 floor47|
|13|946, Act3 floor39|761, Act2 floor33|1005, Act3 floor41|
|14|430, Act2 floor23|877, Act3 floor38|796, Act3 floor38|
|15|1141, Act3 floor50|1120, Act3 floor48|1147, Act3 floor50|
|16|1112, Act3 floor48|1095, Act3 floor48|1062, Act3 floor48|
|17|1030, Act3 floor50|1051, Act3 floor50|1021, Act3 floor50|
|18|999, Act3 floor42|1099, Act3 floor47|902, Act3 floor42|
|19|956, Act3 floor41|932, Act3 floor41|1054, Act3 floor41|
|20|916, Act3 floor39|889, Act3 floor39|890, Act3 floor39|
|21|470, Act2 floor24|1049, Act3 floor46|766, Act2 floor33|
|22|1166, Act3 floor47|1198, Act3 floor48|1227, Act3 floor48|
|23|1333, Act3 floor50|1343, Act3 floor50|1283, Act3 floor48|

Actual same-visit Act3 boss records are TimeEater3, DonuAndDeca3 and
AwakenedOne2; all eight boss records are Death. These come from seeds12,
15,17 and23, through actual normal progression. Actual ordinary coverage is
OrbWalker, ThreeShapes, SpireGrowth, Transient, FourShapes, Maw and
SphereAndTwoShapes; elite coverage is GiantHead, Nemesis and Reptomancer.
**All boss-entry/record alternatives are now covered, but normal Act3NoKeys
completions remain zero.** At its first evaluation, the one core suite test failed solely on that
missing completion. The later approved amendment removes only that normal-win
predicate; completion is proved separately by the supplied suite above. All42 core transcript replay checks complete before that aggregate
assertion. Death remains a valid outcome and is never relabeled as completion.

The pre-outcome freeze has698 source files/Git modes and572 non-Markdown files:
full root `c676a26413a0f7961cdddf40a8b90a69993e6096543bf91ac703f3ad17830442`,
non-Markdown source/mode root
`e8fd6a8d1b0a2a96aaf4ea8610b3b9eb1307d52299b5eb71a7bcf2766a9f8f49`,
manifest SHA-256
`768fc6fcb8c8b2fe2b4c949f83f55acc7217a2843a2b2ee9eca1797475b21dd4`,
and compiled case-table SHA-256
`fbd8f19889e7daf8492be96177cf9a45380203875dd269e5f4a176afdd507847`.
The frozen v5 policy/rules identity remains
`469634bf0fdee0f345dd50854b104d9583375584cef8f6e6410c84b4324a71c6`.

Preflight recorded47,833,300,992bytes available RAM and837,447,720,960bytes
available evidence storage. The measured suite wall time was3:02.00. No
resource limit prevented retention. The native42-case single/batch workers1/2
and semantic replay target compiles and processes one case at a time; at this
first suite evaluation it was **not executed** because the then-required
completion gate failed. Its former
six-case native verification is retained separately below. No new full workspace
or CLI preservation run immediately followed this failed suite. The later
supplied amendment authorizes the final42-case native matrix; no Python
delivery is claimed.

The suite is exhausted and stopped. No additional seed, policy revision,
later start, stronger loadout, acceptance review, publication or Bead closure
is implied. Only this qualified Markdown report changed after evaluation;
source policy and gameplay remain frozen. Exact source/case manifests, diff,
preimages, commands, resources, Results and all traces remain independently
retained for root disposition.

### Retained bounded caller-development phase v3–v5 — exhausted

The user approved at most three further M14-only candidates, under proposal
SHA-256 `bd1dd6df9ac5e2cbbb60d75f9e52b4155b542a297cbc02b53472b58d94091952`.
Each candidate used meaningful supplied-state RED/GREEN checks, the six accepted
M13 full Debug digest checks, and an exact source/rules/Git-mode freeze before
its one six-case evaluation. All18 phase Results and full caller transcripts
remain separately retained (1,466,385,201bytes). All18 are genuine Deaths,
without operational errors or budget exhaustion. The earlier v1/v2 evidence
remains immutable. No seeds, engine/content/resources/RNG,8192 budget or coverage
gates changed; no private simulator or scripted case-specific wins were added.

V3 added public-hand-aware discard/retain/copy choices and current-cost setup/
poison ordering. Saved v2 evidence had140 empty Retain confirmations and81
selected positive-block cards whose printed cost fit current energy; that is
a public-state screen, not proof that a PlayCard action was legal at the
selection prompt or that a different choice would have prevented Death.
V3 still had zero boss coverage; its actual Act3 elite coverage was Nemesis.

V4 used fewer duplicate offensive acquisitions, offered Snecko Skull priority,
ratio-safe Strike purges, funded poison-before-Catalyst, and published target
power preferences. Saved v3 evidence showed a25-card deck first drawing/playing
Fumes on turn4 after HP59→41→33. This supported a preparation hypothesis,
without proving an optimal deck or a winning alternative. V4 reached two real
Act3 elite species and floor45, but still no boss entry/completion.

V5 adds current published Constricted and held Burn2/4 damage to its threat
heuristic, uses stronger printed block when passive Fumes is active, and
prefers legal attack/draw choices before optional setup/paid defense against
the published Shifting power. It recomputes after real actions; it predicts
no future enemy move, hidden hits, exact damage reduction or kill. Current
policy label: `m14-shared-public-caller-v5`. The engine and native identities
remain unchanged.

| Seed | Fixed Act1 / Act2 / Act3 targets | Actual v5 result | Actions | Endpoint | HP |
|---|---|---|---:|---|---|
|7|Guardian / Champ / AwakenedOne|Death|1157|Act3 floor45|0/87|
|7|Hexaghost / BronzeAutomaton / TimeEater|Death|1216|Act3 floor47|0/101|
|7|SlimeBoss / Collector / DonuAndDeca|Death|1245|Act3 floor47|0/101|
|11|Guardian / BronzeAutomaton / DonuAndDeca|Death|991|Act3 floor41|0/101|
|11|Hexaghost / Collector / AwakenedOne|Death|1048|Act3 floor45|0/101|
|11|SlimeBoss / Champ / TimeEater|Death|927|Act3 floor40|0/87|

V5 actual Act3 ordinary entry/record coverage is OrbWalker, ThreeShapes,
SpireGrowth, Transient, FourShapes and Maw. Actual elite entry/record coverage
is GiantHead, Nemesis and Reptomancer. **Actual Act3 boss entry/record coverage
is empty and normal Act3NoKeys completions are zero.** Encounter identities
come from real records; a terminal floor does not identify its enemy. The
current milestone gate fails. No v6 or post-result policy change followed.

Before v5 evaluation, all696 source files/Git modes were frozen: full root
`7018ebdea65ed55dc32382d2e5133a5ae8786930a23eae9afd57073286686af0`,
570-file non-Markdown source/mode root
`87b3c8dc248692a87c50f421411a47a0374b2a509b76adf317cfd0b1c04e4d4c`,
manifest SHA-256
`7fe3a7930df73f8d9eceff4350a630697b350d599a8e3755ecd436b9da01436f`,
and rules SHA-256
`469634bf0fdee0f345dd50854b104d9583375584cef8f6e6410c84b4324a71c6`.
All64 focused revision checks passed; these include35 new phase cases plus
29 v2 supplied cases. All six accepted M13 full transcript digests passed
before each candidate; original full traces remain byte-identical.

One final phase-end stable workspace matrix recorded **1,902 passed,2 failed,
0 ignored across62 result blocks**. Its only failures are the historical Blue
Slaver case named below and the M14 normal-start coverage gate. The core M14
caller target records81 passes/1 gate failure. Native schema5/session4/replay4
pass; native batch records83 passes, including79 embedded shared caller/helper
cases and4 batch cases. Full recorded core/single/native batch agreement across
workers1/2 and semantic replay passes on the six genuine Death endpoints.
It supplies no missing normal boss or completion evidence.

M14-associated targets record193 passes/1 gate failure, including158 executions
of79 shared caller/helper cases embedded in two targets; those duplicate
executions are not additional unique mechanics coverage. The35 new phase
cases are caller tests, not new engine-mechanics validation. The original core
Run10/combat5 and supplied native cutoff1 diagnostics still pass. M13's core
target passes81 cases including the same helpers; its six full Debug transcripts
retain their accepted identities. All15 predecessor CLI commands preserve the
accepted exits/ordered lines. Both prescribed M14 CLI runs exit0 and reproduce
the six v5 action counts, endpoints, HP and outcomes above.

Source bytes/modes stayed fixed throughout that final matrix and CLI set.
Only this qualified Markdown report changed afterward; the runtime root is
unchanged. All frozen candidates, approvals, rules, losses, traces, commands,
logs, hashes, patches and preimages are independently retained. Development
stopped after three unsuccessful candidates. M14 remains **INCOMPLETE**, with
no final acceptance reviewer, acceptance, publication or Bead closure. Root
owns the next disposition. No Python-extension delivery is claimed.

### Retained approved caller v2 revision

After the original six losses, the user explicitly approved one bounded
M14-only caller revision. It adds public deck-ratio/copy-cap acquisition,
ratio-aware permanent removal, finite upgrade priorities, a shared route rank
that minimizes extra elites and prefers visible recovery at low health, and
legal current-intent defense before optional setup. The public policy label is
`m14-shared-public-caller-v2`. The engine, content, native identities, canonical
resources, six triples,8192 budget and coverage gates stayed unchanged. This
is an approved strategy revision; it does not establish a simulator defect or
promise successful routes. No third revision or outcome-driven tuning followed.

The approved addendum SHA-256 is
`76141f6184025f9c0961af1e86793ce55ae77b8f81a063b48167db83a105f6df`.
Supplied-state REDs were retained before integration. All29 supplied revision
cases passed (16 acquisition,6 route,7 combat/reporting). The M13 target passed
46 cases, including embedded caller helpers; its six full Debug transcripts
(302,696,735bytes) remained byte-identical to the accepted predecessor files
before any new M14 outcome.

All692 source files/Git modes were frozen before v2 evaluation. The566-file
non-Markdown source/mode root is
`c59bcdf4f6aeeb54f5a8647b49003797d78f7a3e086e3a610e64df5d521a6e52`;
the pre-outcome full-source root is
`cb6135769ca5aeab4b296056944e10c3e9b4b6be22eb52a8a201609a31e2269b`.
The manifest SHA-256 is
`44fc7b6684d3845579d9bb22a4c0a01feadd668d9b4b07d51c7c8f721323e140`,
and deterministic rules SHA-256 is
`21f46d767dd7127fc25646d93ee73d2b2b2e5e2f764626754cc168fc10d354f1`.

| Seed | Fixed Act1 / Act2 / Act3 targets | Actual v2 result | Actions | Endpoint | HP |
|---|---|---|---:|---|---|
|7|Guardian / Champ / AwakenedOne|Death|864|Act3 floor39|0/87|
|7|Hexaghost / BronzeAutomaton / TimeEater|Death|842|Act3 floor39|0/87|
|7|SlimeBoss / Collector / DonuAndDeca|Death|848|Act3 floor39|0/87|
|11|Guardian / BronzeAutomaton / DonuAndDeca|Death|733|Act2 floor33|0/87|
|11|Hexaghost / Collector / AwakenedOne|Death|698|Act2 floor33|0/87|
|11|SlimeBoss / Champ / TimeEater|Death|718|Act2 floor33|0/87|

Every Result and full public Caller transcript was exported before assertions:
six separate v2 files totaling406,620,695bytes. All outcomes are genuine Deaths,
with no operational errors or budget exhaustion. Actual Act2 boss entry/record
coverage includes all three alternatives. Actual Act3 ordinary entry/record
coverage is OrbWalker, ThreeShapes and SpireGrowth. **Actual Act3 elite and
boss entry/record coverage is empty; normal Act3NoKeys completions are zero.**
The unchanged milestone gate fails. The first target recorded46 passes and
one gate failure; all selected cases were attempted and preserved.

One stable v2 workspace matrix on those frozen bytes recorded **1,762 passed,
2 failed,0 ignored across62 result blocks**. The only failures were the
historical Blue Slaver test named below and the M14 normal-start coverage gate.
The core Run10 and combat5 diagnostic cases and supplied native cutoff1 pass.
Native schema5/session4/replay4 pass. The native batch target passes48 cases:
4 batch cases and44 embedded shared caller helpers. It verifies every action,
reward, terminal, encoded observation, reproduction and semantic replay against
core and single sessions across workers1/2. Its six endpoints remain Death;
transport agreement supplies no missing normal completion.

M14-associated targets record123 passes and one gate failure, including88
executions of44 shared caller/helper cases embedded in two targets. Of those,
58 executions come from the29 new v2 supplied cases; duplicate executions are
not additional unique mechanics coverage. The broader workspace also embeds
these helpers in the M13 core/session targets.

All15 prescribed predecessor CLI commands retain their accepted exits and
ordered semantic lines. Both prescribed M14 CLI executions exit0 and reproduce
the exact six v2 outcomes, action counts, endpoints and HP above. CLI success
means the declared runs executed; it does not pass the milestone gate. Source
bytes and Git modes stayed unchanged through verification. Only this qualified
Markdown report was updated afterward; the runtime root stays unchanged.

The original and revised source exports, rules, approvals, losses, full traces,
commands, logs, hashes, patches and preimages remain separately retained outside
managed sessions. The resulting packet remains **INCOMPLETE**, without final
acceptance review or publication. No Python extension delivery is claimed.

### Retained original caller v1 evidence

The following original results and identities describe the first frozen caller,
before the separately approved v2 exception. Its source export and six losses
remain immutable. They are historical evidence, not the current source identity.

The caller and all 689 source files/Git modes were frozen before the first
M14 route evaluation. The tested 563-file non-Markdown source/mode root is
`adaaf9c737b64ba7d0647a5cb085f818da3803f04b06aa230426d580b2198227`;
the pre-outcome full-source root is
`780a31915f5a8af6d385a8f007d897bc9f494373e144c9a6da9a77249884162f`.
The shared caller source SHA-256 is
`cb64cf77128874dfeeffa2cc067440e98c3f6c212eeb7d3a86920a4e3717f07f`;
deterministic rules SHA-256 is
`3f408ec98b55a9e09c192f60c4b116427e0f48c27d4dd281c2d8c02193dd6fde`.
The public route scorer remains byte-identical to accepted M13. The one shared
loop retains M13's4096 maximum; M14 uses default/maximum8192 and rejects8193.
Falling chooses the lowest existing public card value, then smallest captured
permanent instance. Tomb takes offered gold only when Mask is owned, otherwise
Leave; actual ending choices use Proceed/Continue. No policy, resource, enemy, seed or matrix change was made during that
original freeze/evaluation. The approved v2 exception is recorded above.

| Seed | Fixed Act1 / Act2 / Act3 targets | Actual result | Actions | Endpoint | HP |
|---|---|---|---:|---|---|
|7|Guardian / Champ / AwakenedOne|Death|409|Act2 floor23|0/77|
|7|Hexaghost / BronzeAutomaton / TimeEater|Death|633|Act2 floor33|0/87|
|7|SlimeBoss / Collector / DonuAndDeca|Death|769|Act3 floor39|0/87|
|11|Guardian / BronzeAutomaton / DonuAndDeca|Death|422|Act2 floor23|0/77|
|11|Hexaghost / Collector / AwakenedOne|Death|840|Act3 floor40|0/87|
|11|SlimeBoss / Champ / TimeEater|Death|828|Act3 floor40|0/87|

All six are genuine normal terminal outcomes, without operational errors or
budget exhaustion. All three Act2 boss species retain actual entry/record
coverage. Actual Act3 ordinary entry/record coverage is OrbWalker, ThreeShapes
and SpireGrowth; actual elite entry/record coverage is GiantHead and Nemesis.
**Actual Act3 boss entry/record coverage is empty and normal Act3NoKeys
completions are zero.** The amended milestone gate therefore fails. Visible
boss icons, supplied boss victories and supplied key/endings are not promoted
into missing normal evidence. Every actual reached stage/history/source is
checked, without requiring invented states after Death.

The first fixed-route test exited101 after preserving every Result and complete
public caller transcript (six files,332,881,257bytes) before global assertions.
Both prescribed M14 CLI seed7/11 executions exited0 and reproduced the same
six action counts, endpoints and outcomes. CLI success here means the declared
runs executed, not that the milestone completion gate passed. Trace export via
`SPIRE_M14_TRACE_DIR` is optional for ordinary consumers; the first evidence run
set it to independently retained storage. When requested, export errors remain
errors and all selected cases are attempted before aggregate failure.

One stable locked/offline native workspace matrix used Cargo jobs2, one test
thread, default features and `--no-fail-fast`. It recorded **1,646 passed,
2 failed,0 ignored across62 result blocks**. Its only failures were:

- Historical `combat::tests::full_enemies::blue_slaver_and_two_fungi_encounters_construct_real_actors_across_ascensions`.
- `m14_six_fixed_triples_enter_real_generated_bosses`, because all three actual
  Act3 boss entry/record alternatives and normal Act3NoKeys remain missing.

The M14-associated workspace executions are65 passes/1 gate failure, including
30 executions of the15 pre-existing shared caller/helper cases embedded in two
new targets; these are not30 additional unique M14 mechanics cases. The core
Run10 and actual combat5 diagnostic cases, supplied native cutoff1 case,
native schema5/session4/replay4 cases and batch4 cases pass. The core caller's
remaining two controls pass. The native batch target has19 passes including
its15 shared helper cases. It compares every recorded action, reward, terminal,
encoded observation and final reproduction with core and single sessions,
across workers1/2, then JSON round-trip/semantic replay. **Its six normal
endpoints are all Death; it supplies no normal Act3NoKeys trace.** Capacity
fault diagnostics retain the first fault and accepted action/peer work; replay
with sufficient transport capacity preserves the recorded fault metadata.

All689 hashes/modes stayed unchanged throughout the matrix. All six full M13
transcripts (302,696,735bytes) are byte-identical to accepted predecessor traces;
the existing M13 target passes17 tests and pins their full Debug SHA-256 values.
All15 predecessor CLI commands match the accepted M13 preservation receipt's
exit codes and ordered semantic lines, including unchanged M12 counts
269/280/314 and267/265/268, M9/M10 losses and invalid-input behavior. The native
schema tests pin accepted M9/M10/M12/M13 fingerprints and IDs. No Python
extension feature, build, import or delivery verification was performed.

Exact argv/environment/exits, hashes, original losses, first traces, full source,
Git modes, patch and preimages are retained outside managed sessions. The
qualified source packet is INCOMPLETE and remains for root disposition; no
further caller retuning, acceptance review, publication or outcome closure is implied.

## Source grades and remaining work

The [reference index](../../data/README.md),
[M8 static findings](../../data/enemies/m8-static-findings.md), accepted
[M11](m11-silent-combat.md), [M12](m12-silent-act1.md),
[M13](m13-silent-act2.md), and shared
[Act3 ending research](../research/2026-10-23-m9-act3-ending.md) retain their
individual evidence grades. Native setup/mechanics tests are distinct from
original-runtime validation; no new private-source publication is implied.

This is finite native Rust Silent A0 source work. It does not deliver complete
Silent reward/shop/event pools, all Silent cards or exclusive items, Act4/Heart,
A1–20, Java RNG/map parity, a built/imported Python extension, or strategy quality.
M15 must enumerate the remaining Silent A0 card/exclusive relic/potion/event and
interaction catalog against the cited source owners, implement closure or name
exact source/runtime blockers. The finite elite-stock exhaustion above is one
concrete unresolved content boundary; omitted pools and finite stock cannot
count as complete Silent A0 coverage.
