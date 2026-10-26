# M13 — finite Silent A0 through Act2 (accepted profile)

Status: **accepted at the finite native scope below** after one fresh independent
GPT-6 Luna MAX milestone review with no Critical, Important or Minor findings
and explicit root scope rulings. The [coverage amendment](../superpowers/plans/2026-10-26-silent-normal-run-coverage-amendment.md)
requires actual entry and a matching same-visit CombatRecord for all three Act2
boss species across the same six fixed runs, plus a real normal-start Act3Ready
completion. The frozen caller supplies all three species, three completions and
three genuine Deaths. Original losses and strict pre-amendment failures remain
preserved. The [approved design](../superpowers/specs/2026-10-26-m13-silent-act2-design.md)
and [plan](../superpowers/plans/2026-10-26-m13-silent-act2.md) retain their original
bytes as approval history; separate records state the later authorized changes.
Later milestone implementation requires its own approved plan and assigned source.

`RoomScenario::M13SilentAct2` extends the existing Rust Run, generated maps,
room services, encounter setup, combat resolver, settlement and act transition
owners. It starts from canonical Silent A0 resources and actual short Neow.
M12 retains its accepted Act2Ready cutoff. M13's supported source boundary is
Act3Ready at act3/floor34 after the Act2 boss reward, boss chest, mandatory
relic children or confirmed skip, ordinary act healing and successor map
initialization. Diagnostic room fixtures exercise this boundary separately from normal-start
outcome evidence. The bounded revision also reaches it normally in three cases.

The finite content remains M12's 19 permanent green card kinds, 13 relic kinds
and four potion kinds. Generated identities are Shiv, Dazed, Burn, Slimed and
the additional Wound produced by actual Book of Stabbing/Taskmaster mechanics.
There is no expanded ordinary card pool or captured curse producer.

Act2 has six ordinary encounter IDs (Spheric Guardian, Three Byrds, Centurion
and Healer, Snecko, Chosen and Byrds, Three Cultists), three elite IDs (Gremlin
Leader, Slavers, Book of Stabbing) and three boss IDs (Champ, Bronze Automaton,
Collector). Its actors add 21 identities to M12's 16. Cultist is already in
M12, so there are **22 reachable Act2 actor identities and 37 total M13
identities**. This clarifies the approved documents' count without changing
their named roster: the 21 additions include six ordinary actors, Gremlin
Leader and its five child species, the three Slavers, Book of Stabbing, and
five boss/summon species. Actual Three Cultists reuses Cultist.

The finite events are Neow, Upgrade Shrine, Purifier, Ancient Writing and
Joust. Actual Act2 question rooms choose Ancient Writing first, then Joust
at 50 gold or more, or Purifier below 50; later questions follow the shared
Monster path. Ancient Simplicity uses the existing upgrade owner for green
Strike/Defend basics. Existing Ironclad event behavior remains shared.

Act2 reward cards rotate M12's three captured triples by Act2-local settled
combat visit; the boss offers Nightmare or Skip. Potions rotate Fire,
Strength and Weak. Act2 has its own first/later shop counter, using the same
captured Silent shop row builder as M12. Its elite relic cursor starts locally,
skips owned relics and consumes successfully published offers even when declined.
Exhaustion returns `captured Silent Act2 elite relic sequence exhausted` before
another cursor advance; accepted settlement retains the source fault and its
combat custody. Stock is never replenished. Boss treasure offers Empty Cage
and Slaver's Collar with owned-item checks and explicit skip confirmation.

Four actual Act2 combat diagnostics pass against existing shared mechanics:
Artifact 3 blocks three Poison applications before base Deadly Poison plus
Snecko Skull applies six; Accuracy-buffed Shivs deal four HP through Flight
and decrement Flight per hit; Taskmaster-generated Wound moves through actual
Survivor/Dagger Throw selection without Reflex/Tactician triggers, while the
selected listener instances give their own draw/energy; Well-Laid Plans and
Nightmare retain the original and deliver three distinct temporary copies.
These diagnostics do not establish a normal-start completion.

The original caller baseline was frozen before outcomes. Its support source SHA-256 is
`5eec1989c5a0b19839d9c8b4aab48075fc904d9ba80f0cdcb3fea5e154bb927e`.
An earlier retained freeze differs only in the initial Act2 history exclusion
spelling: the shared history uses `Automaton`, while the encounter ID is
`BronzeAutomaton`. That configuration correction preceded every outcome;
the action policy was unchanged. The six seed/boss pairs were declared before
execution and were not replaced.

| Seed | Act1 target | Act2 target | Actual outcome | Actions |
|---|---|---|---|---:|
| 7 | Guardian | Champ | Death in Champ combat, floor33 | 672 |
| 7 | Hexaghost | Bronze Automaton | Death to Slavers, floor23 | 426 |
| 7 | Slime Boss | Collector | Death to Book of Stabbing, floor23 | 436 |
| 11 | Guardian | Bronze Automaton | Death to Slavers, floor23 | 430 |
| 11 | Hexaghost | Collector | Death to Book of Stabbing, floor23 | 436 |
| 11 | Slime Boss | Champ | Death to Gremlin Leader, floor23 | 457 |

All six original baseline runs start normally, choose actual Lament, win their selected Act1 boss,
initialize the requested Act2 boss icon, and enter ordinary/elite Act2 combat.
For that original baseline, only seed7 Guardian→Champ enters an Act2 boss.
Bronze Automaton/Collector entry coverage and every normal-start Act3Ready
completion were unmet. The required integration test fails explicitly for five missing entries
and zero completions; it replays every recorded public decision, map, action
and transition before asserting the gate. Natural Death, caller budget0/1,
invalid actions and retained native faults stay separate. No policy tuning,
seed search, resource injection or enemy-value changes followed the outcomes.

Run the exact fixed examples with existing offline dependencies:

```sh
cargo run -p spire-core --example m13_silent_act2 --locked --offline -- --seed 7 --scenario all
cargo run -p spire-core --example m13_silent_act2 --locked --offline -- --seed 11 --scenario all
cargo test -p spire-core --test m13_silent_act2 --locked --offline
```

The current examples identify `m13-public-caller-v2` and report genuine Death
or Act3Ready with a successful process exit. The amended integration target
checks all six outcomes and aggregate real species coverage/completion; map
icons and early Deaths never supply missing boss-entry coverage.
Its optional `SPIRE_M13_TRACE_DIR` writes full public traces during verification.

The user subsequently approved [one bounded public-caller revision](../superpowers/plans/2026-10-26-m13-public-caller-revision-approval.md).
It keeps the sole shared caller loop, all six seed/boss pairs and every existing
acceptance assertion. No engine, resources, enemy values, reward/shop/deck
priorities or ordinary combat priorities changed. The original caller/wrapper,
670-file candidate and six loss traces remain independently retained.

Revision `m13-public-caller-v2` compares only offered public paths using current
HP/max HP, potion stock, gold and visible room types. It requires ordinary and
elite traversal, values recovery before the required elite and refuses a
requirement-dropping shortcut. It handles current survival pressure before
scaling and uses threat-based potion targets; attack retargeting applies only
under survival pressure. Its score is a bounded heuristic with no simulated
future encounters, draws or game actions. Published intent damage is per hit;
CombatPublic exposes no hit count. The pressure hint is explicitly a lower bound,
not proof of turn safety. No private hit lookup, boss case table or resolver copy
was added.

Fifteen supplied public-state unit tests passed before any revised fixed case
was evaluated. The revised caller/helpers/wrapper and rules were frozen at
2026-10-26 14:54:04.792758 UTC. Support source SHA-256:
`82c149a4fb1852509167988df3dfd134e65b20ee0e366bbf14b8e85d6002b917`.
Rules receipt SHA-256:
`faeb71d6c9900b1a12831c7fc197c5c07b1dd5a0429d7f0b84d97d32b3ad2b26`.
No policy edit followed these outcomes:

| Seed | Act1 target | Act2 target | Revised actual outcome | Actions |
|---|---|---|---|---:|
| 7 | Guardian | Champ | Death to Gremlin Leader, floor23; no Champ entry | 409 |
| 7 | Hexaghost | Bronze Automaton | Death during actual Bronze Automaton, floor33 | 633 |
| 7 | Slime Boss | Collector | Act3Ready, act3/floor34, HP87/87 | 663 |
| 11 | Guardian | Bronze Automaton | Death to Slavers, floor23; no Automaton entry | 422 |
| 11 | Hexaghost | Collector | Act3Ready, act3/floor34, HP87/87 | 705 |
| 11 | Slime Boss | Champ | Act3Ready, act3/floor34, HP87/87 | 723 |

All three successful paths record real Act2 boss Victory, actual card reward,
boss chest and mandatory relic child resolution before successor initialization.
All three boss species have actual entries across the matrix, but the prescribed
seed7 Guardian→Champ and seed11 Guardian→Bronze Automaton routes die early.
Before the later coverage amendment, the original per-route
`m13_initial_boss_targets_select_generated_maps_and_actual_entries` assertion
reported those two missing entries and failed. That 16-pass/one-failure result,
its exact source and full traces remain immutable evidence. The user's explicit
amendment supersedes only that per-route boss-entry criterion. The current test
runs and reports all six cases, requires same-visit CombatEntered/CombatRecord
pairs for aggregate coverage of Champ, Bronze Automaton and Collector, and keeps
all actual completion/reward/chest/transition assertions. Missing species or
normal completion still fails. The amended focused core target passes 17/0;
the actual outcomes/action counts above are unchanged. No new caller revision,
seed/resource change or outcome replacement occurred.

The unchanged native schema/replay/batch/session targets recorded 26 passes,
zero failures (including the same 15 unit cases included by the session target).
Every step of all six revised transcripts agrees across the native session and
semantic JSON replay. The three real readiness paths now make the healthy
Act3Ready zero-action/rejection/reset/reproduction assertions executable; they
pass. The frozen revised non-Markdown source/mode root is
`a9c3a15021e6c89cfdc5d54be0a50addd2da914cb5be20cbd5123cc007ca8619`.
Full revised public traces and statuses are retained separately from the originals.

The final amended native preservation run used
`cargo test --workspace --locked --offline --no-fail-fast`, two Cargo jobs and
one test thread. It recorded **1,581 passed, one failed, zero ignored** across
57 target result blocks. The sole failure is the retained historical
`combat::tests::full_enemies::blue_slaver_and_two_fungi_encounters_construct_real_actors_across_ascensions`.
Every M13 test occurrence passes: 52/0, including the same 15 supplied public-state
unit cases included by both the core-caller and native-session targets. This is
not a green-workspace claim. The other shared milestone, environment, replay,
batch, report and default-feature workspace targets pass.

All 15 final CLI invocations matched expected process exits. M13's six outcomes
and action counts remain exactly those in the revised table. M12 retains its
six Act2Ready counts (269/280/314 and 267/265/268), and all 16 M9/M10 scenario
summary lines match the independently verified accepted-M12 baseline. The
existing caller's genuine Death controls, budgets0/1 and invalid-input checks
retain their distinct behavior.

All 674 source hashes and Git modes remained stable across that final run. Its
tested non-Markdown source/mode root is
`a472a57ff643ca305202e4c4c84f7ee4d52c2597e94afe9b14093a79ffcbba29`.
Only this profile's administrative result text was completed afterward; frozen
caller/rules, every production/test/dependency byte and all previous evidence
remain unchanged. Complete source/mode manifests, patch/preimages, commands and
exit statuses are retained independently. The subsequent independent review and root acceptance are recorded below;
this matrix receipt alone is not acceptance.

The native adapter adds `RunSession::new_m13`, `RunBatch::new_m13`,
`m13_run_observation_spec` and `encode_run_m13` on the existing session,
encoder, replay and scheduler owners. `RunSessionConfig` keeps one
`boss_history` field. Its additive `RunBossHistory::M13SilentAct2 { act1, act2 }`
uses `RunAct1BossTarget` and `RunAct2BossTarget` before Run construction.
The JSON form is
`{"m13-silent-act2":{"act1":"guardian","act2":"champ"}}`; malformed,
partial, unknown and mismatched profile histories reject. All six previous
history string spellings and M9/M10/M12 observation fingerprints remain stable.
The new Rust enum variant requires new arms in exhaustive consumer matches;
this is not unchanged exhaustive-match source compatibility or a Rust ABI claim.

| Identity | Value |
|---|---|
| Profile | `M13SilentAct2` |
| Observation | `spire-gym-m13-silent-act2-observation-v1` |
| Replay | `spire-gym-m13-silent-act2-replay-v1` |
| Rules | `spire-gym-native-m13-rules-v1` |
| Content | `spire-gym-m13-silent-a0-acts1-2-captured-v1` |

Schema tests cover every finite identity, all 12 actual Act2 setups and their
initial/two-turn public rows, actual summons/statuses/intents, the retained
Cultist and foreign-ID rejection. Codec-only fixtures cover Act3 map icons;
they do not prove a native session reaches or resets at Act3Ready. Every action,
reward, terminal and observation in the original six Death transcripts and
the revised mixed-outcome transcripts agrees
with the native session and semantic JSON replay. Batch prefixes agree with
single sessions using one and two workers. The Ironclad combat-policy sidecar
remains explicitly unsupported for M13.

The original pre-revision 670-file native workspace verification used locked offline dependencies,
two Cargo jobs and one test thread. It recorded **1,548 passed, 4 failed,
0 ignored**, with every workspace target executed using `--no-fail-fast`.
The failures are:

- The historical
  `combat::tests::full_enemies::blue_slaver_and_two_fungi_encounters_construct_real_actors_across_ascensions`.
- `m13_initial_boss_targets_select_generated_maps_and_actual_entries`:
  five prescribed boss entries and all normal completions remain unmet.
- `m13_session_matches_six_fixed_core_transcripts_and_act3ready_cutoff`:
  transcript agreement passes, but there is no natural Act3Ready completion.
- `m13_act3ready_is_healthy_resettable_zero_action_boundary`:
  a natural ready session is unavailable, so the cutoff/reset assertions remain
  unexercised.

In that pre-revision workspace, the 22 M13 tests comprise 19 passes and those
three failed outcome/boundary gates. Five core room/custody tests, four actual combat interactions, three
schema tests, two replay tests, two batch tests, typed configuration, standalone
transcript agreement and Death/budget/invalid controls pass. This is not a green
workspace or a completed milestone. Fifteen final CLI invocations matched their
expected exits, including the unchanged six M12 Act2Ready examples, recorded
M9/M10 seed7/11 outcomes and M13 fixed/Death/budget/invalid controls.

All 670 source hashes stayed unchanged across that original verification. Its tested
non-Markdown source/mode root is
`b5caf86c937179d9f63fca627a09837c513a5310b5a95309be738f84c41a7e4e`;
the original retained candidate then completed this evidence profile. The later
user-approved caller revision has its separate freeze and focused results above;
the old workspace count is not represented as a revised-source workspace run.
The separate final amended preservation result is recorded above. Source manifests, explicit
commands/statuses, the original caller receipts, six full public traces and
isolated-author preimages are retained outside managed sessions. Root owns
candidate disposition and the one final milestone review; no final review,
acceptance, commit, staging, push or publication is asserted here.

This is native Rust source work. It does not establish a built or imported
Python extension, full Silent pools, Acts3–4 gameplay, Heart, A1–20, Java RNG/map
parity, original-game execution or policy quality. Source grades remain those
of the [reference guide](../../data/README.md) and accepted M11/M12 profiles.
M15 still owns complete Silent A0 catalog closure or explicit runtime/source
blockers. A frozen caller's genuine losses cannot be promoted into acceptance.

## Independent review and root acceptance

One fresh GPT-6 Luna MAX milestone reviewer found no Critical, Important or
Minor findings. All five focus classes were assessed. Seven bounded locked/offline
checks passed 18 tests; the reviewer also verified all 674 source hashes/Git modes
and reconstructed the patch exactly from the accepted M12 baseline. Review
report SHA-256: `36ecbbf10408cc0e5c31c0806a68e4d43053bd8d3b9b72530571b51af07b1566`.

The reviewed source root was
`72d0e0842828808f83a3779d830f474b2a7a5e474b40adef28555e00e8c53011`.
Root accepted the finite profile after explicit rulings on the ten executor
choices and declined scope. Only acceptance documentation and the already-approved
M14 planning packet changed afterward; all runtime, tests, caller, dependencies
and Git modes remain identical to tested runtime root
`a472a57ff643ca305202e4c4c84f7ee4d52c2597e94afe9b14093a79ffcbba29`.

This accepts the declared native profile and the amended normal evidence, not
all three boss victories, full Silent pools, Acts3–4 gameplay, Python artifact
delivery, original-game parity or a strategy guarantee. The M14 design and plan
have separate user approval; the primary session supplies its accepted baseline
and live execution assignment.
