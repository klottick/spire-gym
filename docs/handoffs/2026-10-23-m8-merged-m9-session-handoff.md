# Session handoff: M8 merged; M9 planning next

**Later preservation note (2026-10-23):** The user approved saving this handoff and
its `AGENTS.md` pointer in a local documentation commit on
`docs/m8-session-handoff-20261023`, updating local `main` to merged `origin/main`,
and locally ignoring known tracker/evidence files without deleting them. The
uncommitted-document and older-local-main descriptions below record the original
handoff snapshot. No push, worktree/branch/stash removal or M9 implementation was
authorized by this cleanup.

**Prepared 2026-10-23 at the user's request to continue in a different session.**
This is a continuation guide, not approval of an unwritten M9 plan or a new
implementation assignment. Git/Beads/GitHub state below was checked during handoff
preparation. Test results are the recorded merge verification, not new test runs
performed merely to write this document.

## 1. Read this first

1. **M8 is accepted and merged. Do not redo its implementation or reviews.**
2. [PR #42](https://github.com/klottick/spire-gym/pull/42) is **MERGED**, merge
   commit `4874fb398128c142bd56f10a12e6fca939a88f93`.
3. The next deliverable is **ONE written M9 implementation plan for user review**.
   M9 planning task `spire-dxo.31.1` is **open/unassigned**. No M9 plan exists yet;
   there is no M9 implementation child, code/start approval or active worker.
4. The current workspace already contains the merged product tree. Its branch tip
   is the reviewed feature commit, not the merge commit, but their **trees are
   identical**. Do not reset/clean it or edit the main checkout to “catch up.”
5. Full core tests intentionally retain **seven historical failures**. They are
   not new M8 regressions and must not be hidden, weakened or called green.
6. Real player **Death is valid gameplay**, not automatically a simulator failure.
   Do not search easier seeds or alter game values/resources to force wins.
7. The PR/three-review/merge authorization was a **completed bounded request**.
   It does not authorize more commits, pushes, merges, installs, private audits,
   cleanup or M9 implementation.
8. Preserve every worktree, the named stash and ignored evidence. This handoff and
   its new `AGENTS.md` pointer are **local, uncommitted documentation changes**.

## 2. Project objective and binding scope

Deliver a headless Slay the Spire 1 gym: **one authoritative Rust simulator**, native
batching, thin Python/Gymnasium interfaces, public observations/legal actions,
rewards/reset/termination/replay and small runnable examples. External users own
models, optimizers, training pipelines and learned-agent evaluation; those do not
belong in this repository. No graphics/client/audio/game UI is required.

The approved final target remains **all four fully unlocked standard characters,
Ascension 0–20, Acts 1–4 and the Heart**, with A20 the primary performance target.
Do not re-ask or silently narrow that final scope. Early milestones accept explicit
narrow profiles, not the entire game.

Primary documents:

- [AGENTS.md](../../AGENTS.md): current instructions; newest dated entries supersede
  older historical progress paragraphs.
- [Approved 36-milestone roadmap](../superpowers/specs/2026-10-22-narrow-gym-milestones-design.md):
  binding progressive delivery sequence; specifically M8/M9/M10.
- [Repository scope](../scope.md), [work status](../work-status.md),
  [README](../../README.md): orientation, not authoritative task ownership.
- [Accepted M8 profile](../milestones/m8-act3.md): exact supported mechanics,
  captures, outcomes and exclusions.
- [Approved M8 plan](../superpowers/plans/2026-10-23-m8-act3-mechanics.md): twelve
  completed tasks and their interfaces. It is not the next plan to execute.
- [M7 profile](../milestones/m7-act2.md), [M6 profile](../milestones/m6-act2.md),
  [M5 profile](../milestones/m5-act1.md): endpoints that future work must preserve.

**Stale-text warning:** some committed pages still say “M8 uncommitted,” “M8 active”
or “M7 is the current boundary.” For example, M8 profile's last sentence and parts
of the reference index predate publication. Those statements are historical:
PR42/Beads/publication ledger and this handoff establish the later merged state.
Do not interpret them as lost work or permission to repeat publication/reviews.

## 3. Exact repository/workspace state

| Item | Value at handoff |
|---|---|
| Repository | `klottick/spire-gym` |
| Current isolated controller worktree | `/home/klottick/.local/share/opencode/worktree/503d4a/full-run-gym-poc-20261022` |
| Current branch | `feat/m8-act3-mechanics` |
| Current HEAD / remote feature tip | `2b5637b619c91b6ef22b7ee29b5da996174bfc90` |
| Conventional Commit | `feat(run): add Act 3 mechanics and captured prefix` |
| PR42 merged into `main` | `4874fb398128c142bd56f10a12e6fca939a88f93` |
| Fetched `origin/main` | `4874fb398128c142bd56f10a12e6fca939a88f93` |
| Local `main` | `61c1d042b454a82addafa8ad08b46a939d5704bc` — deliberately not moved |
| Previous `feat/run-act2-prefix` | `ddd4df2138db8bc1860170bb079743184dbf6335` — retained |
| Main checkout / shared tracker root | `/home/klottick/repos/spire-lab` — not the editing workspace |
| Named preservation stash | `a6daee8f9290c330aefe4b26f74ab6b24edf7b62` — retained |

Immediately before this handoff, the feature worktree/index were clean. Only this
new document and the discovery pointer in `AGENTS.md` are intentionally left
uncommitted afterward. **No simulator, tests, data or dependency files are changed
by handoff preparation.** No handoff commit/push is authorized or performed.

PR42 contained100 explicit public paths,4662 insertions/368 deletions. All462
tracked files matched the reviewed snapshot in the controller and three PR-review
worktrees. After merge, `git diff 4874fb3 2b5637b` was empty: the merged tree is the
exact reviewed tree, not an unreviewed conflict resolution. No branch was deleted.

Use non-destructive checks if state has changed since this snapshot:

```sh
git status --short --branch
git rev-parse HEAD origin/main main refs/stash
git diff 4874fb398128c142bd56f10a12e6fca939a88f93 2b5637b619c91b6ef22b7ee29b5da996174bfc90 --
gh pr view 42 --json state,mergeCommit,headRefOid,url
```

The installed `gh` version does **not** accept `baseRefOid` in `pr view --json`.

## 4. Task tracker and session ownership

Use **only** the existing shared Beads tracker:

```sh
/home/klottick/.local/bin/bd -C /home/klottick/repos/spire-lab context --json
/home/klottick/.local/bin/bd -C /home/klottick/repos/spire-lab ping --json
/home/klottick/.local/bin/bd -C /home/klottick/repos/spire-lab list --label poc-gym --status open,in_progress,blocked --json
/home/klottick/.local/bin/bd -C /home/klottick/repos/spire-lab ready --label poc-gym --json
```

Before writes verify database `spire`, project UUID
`a61e792f-9730-47c0-b262-7dac036d6bf5`, and healthy ping. Preserve ordinary local
Dolt history. Do not initialize a new board/database/service or inspect credentials.

| Bead | State / meaning |
|---|---|
| `spire-dxo` | Open full gym epic; broad final target unchanged |
| `.23`–`.30` | M1–M8 accepted/closed at their declared scopes |
| `spire-dxo.30.1` | M8 planning closed after actual user plan approval |
| `spire-dxo.30.2` | M8 native implementation closed after review/verification |
| `spire-dxo.62` | User-requested M8 PR/three-review/merge follow-up closed |
| `spire-dxo.31` | M9 outcome open, activated for planning |
| `spire-dxo.31.1` | **M9 planning only, open/unassigned**; accepted M8 prerequisite |

Outgoing native actor: `opencode-session-recovery-20261021-v1`. Its two recent
owned tasks are closed; do not reopen/impersonate it or force-release old claims.
Use an explicitly allocated actor for the new session and claim the unassigned
planning task normally when actually beginning authorized planning. Do not claim
the whole outcome as implementation. Check current task state first; another
session may have started after this handoff.

The historical `.4`–`.22` broad component/family lanes and archived research are
not a ready-made dispatch queue. Do not restart old authors/audits from old notes.

## 5. Next deliverable: M9 written plan, not product code

Roadmap M9 requires normal-start IroncladA0 progression through all three ordinary
acts and their **real Act3 boss alternatives, room consequences and rewards**,
correct Act3 completion/non-goal versus Death, reproducible replay and the thin gym
interface following the extended path. Act3 completion must not be called Heart
victory. Act4/Heart delivery remains M10.

**No M9 implementation plan has been written/approved.** On authorized continuation,
the next session should:

1. Read the roadmap, accepted profiles, current native interfaces and tracker.
2. Reuse M8's implemented boss/lifecycle/event bodies rather than rebuild them.
3. Identify actual late-Act3 room/boss entry, reward/settlement, terminal/public API
   and replay gaps. Distinguish API exposure from unsupported Python run/batching
   work; the roadmap's interface requirement must be explicitly addressed, not
   assumed satisfied by existing combat-only bindings.
4. Write one bounded M9 plan with concrete files/interfaces, true normal-start
   acceptance cases, qualified source prerequisites and preservation tests.
5. Present it to the user for approval before an implementation child, product
   code, builds/tests as M9 implementation, or author dispatch begins.

These are **planning leads, not decisions already approved**:

- M8's router currently admits35–43 only; topology includes future boss50. M9 must
  design a distinct profile/continuation so M8 still stops at actual chest43.
- The current Run combat bridge admits M8 ordinary/elites, not Act3 bosses merely
  because standalone boss setup works. Follow actual origin, item custody, entry,
  permanent-gain sync, settlement and reward ownership.
- `rooms/boss.rs` and `rooms/transition.rs` currently specialize Acts1/2 source
  chests and1→2/2→3 handoffs. Do not blindly reuse a boss chest/relic or heal at the
  end of Act3; qualify the relevant end-of-act behavior needed by M9.
- `RunTerminal::Act3NoKeys` and `HeartVictory` already exist in the model. Enum
  presence is not proof of reachable implemented progression. Plan the actual
  reachable non-goal and supported all-key boundary without inventing Heart victory
  or silently starting M10's Act4 implementation.
- The established policy may genuinely lose before a boss. Preserve those losses;
  do not convert an operational fault into Death or replace from-start evidence
  with a late fixture. Any new policy change needs a concrete public-state reason
  and an approved, bounded acceptance design—not endless win tuning.

Do not re-ask settled final character/Ascension scope or repeat a per-card design
cycle. Resolve real ambiguities in the written plan; seek user decisions only
where authority/scope genuinely requires them.

## 6. M8 acceptance and additional PR reviews

M8's original single fresh milestone review:

- GPT6 Luna session `ses_e952da62bffeTLac42Fw88P0XH`.
- Worktree `m8-final-review-luna-20261023` under the current worktree's parent.
- No Critical/Important/Minor findings; five required focus areas satisfied.
- Controller verified all462 candidate files unchanged before acceptance checks.

The user then explicitly requested **three different fresh GPT6 Luna agents after
creating a PR**, plus work towards merge. This authorized a bounded additional
publication/review follow-up; it did not invalidate acceptance or create a standing
three-review requirement for every future milestone.

| PR reviewer | Fresh session | Primary lens | Report |
|---|---|---|---|
| A | `ses_e95262386ffeDGGFjvLO37X4Br` | Combat/queues/revival/spawns | `pr42-review-a.md` |
| B | `ses_e9526234dffeoJJCdPxntRMmCI` | Run/events/resources/faults | `pr42-review-b.md` |
| C | `ses_e95262345ffeKxN1OjjmMZNRaU` | Public API/privacy/replay/regressions | `pr42-review-c.md` |

Each independently reviewed the whole PR and all five focus classes, not just its
lens. Each had an exact-HEAD462-file verified worktree
`m8-pr42-luna-{a,b,c}-20261023`, unique matching `/tmp/opencode/` build target and
`CARGO_BUILD_JOBS=2`. All returned **no findings**, with no product edits. Reports
were read fully and preserved; no fixes/re-reviews/deferred minors were necessary.

The five focus areas: pending revival versus final death; accepted twelfth-card
Time Warp work; permanent Parasite identity; distinct spawn/expiry/cleanup/partner
lifecycles; actual chest43/source/resource/privacy/predecessor boundaries.

[Public attributed review summary](https://github.com/klottick/spire-gym/pull/42#issuecomment-6375911757).
These were local agent reviews, **not fabricated GitHub account approvals**.
GitHub reported MERGEABLE/CLEAN and an empty check rollup: **no configured/reported
CI checks**, not “CI passed.” Normal exact-head-guarded merge used no admin bypass,
force push or branch/worktree deletion. Merged at2026-10-23T22:22:39Z.

Do not spawn another M8 review, poll/resume completed reviewers or repurpose their
worktrees for implementation. No workers remain active from this session.

## 7. Where detailed local evidence lives

Current ignored evidence workspace, **preserve it**:

```text
/home/klottick/.local/share/opencode/worktree/503d4a/full-run-gym-poc-20261022/
  .superpowers/sdd/2026-10-23-m8-act3-mechanics/
```

Key files inside it:

- `progress.md`: full twelve-task RED→GREEN/source/interface/final-acceptance ledger;
  every `Ruling:` records a decision and its risk. Read relevant entries, not the
  entire session transcript.
- `publication-progress.md`: later PR42 publication, three additional reviews,
  fresh checks and completed merge; supersedes prepublication “uncommitted” notes.
- `review-report.md`: original milestone reviewer report.
- `pr42-review-{a,b,c}.md`: copied additional reports, also retained in their own
  worktrees at the same relative evidence directory.
- `candidate-manifest.json`: original accepted462-file candidate manifest; five
  administrative files subsequently changed for acceptance/publication.
- `pr42-manifest.json`: exact committed reviewed snapshot and all462 tracked hashes.
- `baseline-workspace.log`, `baseline-failure-names.txt`: pre-M8 historical failures.
- `pr42-premerge-*.log`: complete final publication verification outputs.
- `pr42-postmerge-*.log`, `pr42-merged.json`: merged-tree equality and postmerge checks.
- `publication-allowlist.json`, `publication-close.json`, `.30` closure receipts
  and M9 planning creation receipt: bounded publication/task evidence.
- `stash-disposition-report.md`: independent stash assessment; not an apply/drop order.

These files are **ignored and local**, not in the merged repository. A new checkout
gets the committed product, not these logs or this uncommitted handoff. Read them
in place or copy only needed nonprivate evidence to a dedicated workspace after
checking ownership. Never `git add .` to publish them.

## 8. Implemented mechanics that must survive continuation

All use the existing resolver/actors/Run coordinator—not separate simulators.

- **Shapes/Orb Walker:** native groups and independent helper samples; Orb Walker
  growth at end-round, actual Burn. Exploder queues source suicide before fixed30
  THORNS damage; final-actor accepted damage can still produce real player Death.
  Spiker has per-hit Thorns; Minion/HP0 are not generic cleanup rules.
- **Darklings:** original center identity, actual conditional AI, HalfDead callbacks,
  all-down arbitration, Regrowing/Reincarnate over distinct turns, same actor/RNG
  identity and floor(maxHP/2) revival; no early Victory or duplicated settlement.
- **Growth/Transient/Maw:** actual player Constricted context, post-Block/pre-overkill
  Shifting with Shackled/Strength/Artifact ordering, fifth Transient attack before
  expiry, Maw selected-turn Nom. Do not flatten their special timing into counters
  advanced by reads or unexecuted moves.
- **Writhing:** per-qualifying-hit conditional rerolls, Malleable source queue timing,
  actual permanent Parasite. Owned-static A0 Block move is15damage+15Block, debuff
  attack10; retain attributed wiki16 conflict. Orb follows existing Lightning THORNS
  exclusions, not a fabricated fourth original damage type.
- **Giant Head/Nemesis:** after-use Slow, one final rounding floor, countdown;
  exact history/cooldown/redirect law and absent-only Intangible callbacks/per-hit cap.
- **Reptomancer/Daggers:** source tracked slots distinct from physical/action slots,
  prepared spawn custody survives source-qualified parent cleanup; real child can
  insert afterward and keep combat live. Dagger9+Wound then25 before self-HP-loss.
- **Awakened:** First/Rebirth/Second, typed cleanse/explicit Shackled handling,
  positive Strength retention, full heal/forced Echo and final Cultist escape. Last
  actor at0HP awaiting Rebirth still admits a decision/ordinary EndTurn; a real
  diagnostic exposed this and an actual-card regression protects the correction.
- **Inactive Powers:** resolved Power moves once as the same `OwnedCard` into
  `State.played_powers`, not duplicate handle storage; included in public history,
  exit and exact Run card census. Inflame actually grants2/3Strength.
- **Void:** only actual draw triggers clamped LoseEnergy1; Ethereal at end-turn.
  Sludge uses bottom insertion, not random placement or an unnecessary RNG draw.
- **Time Eater:** strict below-half Haste latch spent at selection; typed cleanse/
  delta heal. Time Warp increments after accepted payload/selection, before
  disposition, persists across turns, resets at12 and retains an owned barrier.
  Current card finishes before one ordinary EndPlayerTurn; no13th input or broad
  queue wipe. Faults retain work; existing native sequential terminal priority stays.
  Draw Reduction is one fewer card with application grace then duration decay,
  derived from active power without mutating authoritative base draw size.
- **Deca/Donu:** actual Deca-first independent execution-advanced cycles, Artifact2,
  attacks10×2/Dazed2, Block16 and Strength3. Publish full group targets in order;
  later action-specific guards apply. Surviving target buffs remain earned after
  source death; killing one alone neither wins nor resets the other.
- **Falling/Tomb/Red Mask:** detached fixed instance captures and staged RNG before
  movement; missing types disabled/all missing Leave; ordinary chosen-instance
  removal, not arbitrary purge or curse removal. Tomb allgold including0 precedes
  actual RedMask acquisition; ownedMask222 and resultLeave once. Startup Weak1
  enumerates initial group, subject to Artifact, not every turn/later summons.

### Permanent gains, faults and sources are especially important

Combat permanent-gain journal acceptance, Run identity allocation/publication and
acknowledgment are distinct frontiers. Synchronize accepted Parasite gains before
another decision **or a later combat fault**, not only Victory. Census exclusions
must be exact recorded identities/origins, never all generated cards. A later paid
Parasite removal really invokes maxHP−3; combat exhaust is not permanent purge.

Preflight failures leave state/RNG/token frontiers unchanged. An accepted failure
retains original source/current/pending work/inventory/latest cursors and freezes
with the first fault. Do not retry mutations, reroll offers, discard custody or
fabricate loss/completion. Pending children must finish before source Proceed.

## 9. Current progression and explicit captures

| Profile | Preserved endpoint |
|---|---|
| M4 | Authored nine-floor room prefix |
| M5 | Real Act1/boss/reward/chest → nonterminal/no-action `Act2Ready`, floor17 |
| M6 | Same-player Act2 prefix → actual chest26 Proceed/no actions |
| M7 | Real all3Act2 boss alternatives/rewards/chest → `Act3Ready`, floor34/no actions |
| M8 | Same-player Map at34, generated playable35–43 → actual Treasure43 Proceed, `Act3PrefixComplete`, alive/nonterminal/no actions |

M8 topology contains future boss50, but does not admit44/50 or a3→4 transition.
Merely reaching floor43, exhausting loot, or having no action is not completion.
Even potion actions are unavailable at the actual cutoff.

M8 ordinary captures: Orb Walker → Three Shapes → strong rotation Growth,
Transient, Four Shapes, Maw, Sphere-and-Two-Shapes, Hard Jaw Horde, Darklings,
Writhing; recent-family/first-strong exclusions apply. Elites rotate Head/Nemesis/
Reptomancer offset by public initialized boss icon. Questions: Falling, Tomb, then
ordinary Monster. These are **explicit captures, not complete standard pools**.

Existing finite source rewards:10–20 ordinary gold,25–35elite gold; Fire/Strength/
Weak potion cycle; first Act3 ordinary card row `[Inflame,ShrugItOff,Anger]`, then
existing rows. Finite eight-item elite sequence skips owned relics; exhaustion
fails explicitly, not refill/fabricated loot. Chest scans Strawberry/Anchor/
BloodVial versus Sapphire; optional Proceed remains available. Later shop captures
Clothesline50/StrengthPotion50/Anchor150 and inherited purge; owned M8 stock Disabled.
Act3-local settlement counts include Death and are separate from global history.

## 10. Observed normal-start outcomes to preserve

Normal public start:80/80HP,99gold,5Strikes/4Defends/Bash, Burning Blood, empty3-slot
potion belt, no keys. Supported short-Neow profile and valid prior boss histories
select Guardian/Automaton/Act3 icon; no injected resources or fight results.

| Seed / Act3 icon | Actions | Final HP | Floor / reward | Outcome |
|---|---:|---:|---|---|
|11 Awakened One|711|0|41 /40|Actual Giant Head Death |
|11 Time Eater|721|91/119|43 /43|Alive scoped prefix completion |
|11 Donu/Deca|738|85/119|43 /43|Alive scoped prefix completion |
|7 Awakened One|694|0|40 /39|Actual Giant Head Death |
|7 Time Eater|685|0|40 /39|Actual Nemesis Death |
|7 Donu/Deca|689|0|40 /39|Actual Reptomancer Death |

The icon selects the elite/capture context; **it does not mean that boss was
fought on the normal prefix**. Standalone diagnostics exercise all three bosses.
All normal paths demonstrate actual early Act3 fights, Falling removal and Tomb/
RedMask; completed11 paths beat their elite and actually obtain/play Inflame.
Nonattacking normal-start Death is10actions/floor1/reward0: floor+1 thenDeath−1.

The sixteen supplied diagnostics use999HP/4energy/Strength20/upgraded finite deck,
TimeWarp11 and a named initial Writhing Parasite move. They execute real bodies
and lifecycle observations but are **not** normal-start proof. Never reuse those
resources as a hidden public episode fixture.

## 11. Verification baseline and commands

Recorded fresh premerge result on the exact committed candidate:

| Lane | Result |
|---|---|
| Core library / workspace stops there |647passed /7historical failures |
| Native M8 |111passed |
| Native M6 / M7 |92 /16passed |
| Public M1–M8 integrations together |56passed (M8 alone6) |
| Consumers (`spire-env`/`spire-python`, plus CLI) |43passed |
| Other core integrations |30passed |
| M7 example reporting test |1passed |
| Offline references |70passed |
| Python performance unit checks |22passed |
| Card / power builders `--check` |Identical370 /133records |
| Examples / scenarios |All8examples; both M8 seeds/all16diagnostics; budgets0/1 reject unfinished execution |

Postmerge tree equality verified; fresh identical-tree core647/7, nativeM8 111/0
and publicM8 6/0 repeated. Typical warm core0.16s/public matrix≈1s; builds reported
separately. These are historical observations, not wall-clock correctness gates.

### Runnable commands after planning/implementation authorization as appropriate

Use existing tools/dependencies, locked/offline. The handoff itself does not grant
installs/rebuilds of Python extensions or a new verification project.

```sh
# Core: expected nonzero because of the SAME seven old failures.
cargo test --workspace --locked --offline
cargo test -p spire-core --lib m8_ --locked --offline
cargo test -p spire-core --lib m6_ --locked --offline
cargo test -p spire-core --lib m7_ --locked --offline

# Accepted public profiles together.
cargo test -p spire-core --test m1_combat --test m2_combat --test m3_run --test m4_rooms --test m5_act1 --test m6_act2 --test m7_act2 --test m8_act3 --locked --offline

# Existing consumers, other integrations and reporting.
cargo test -p spire-env -p spire-python -p spire-cli --locked --offline
cargo test -p spire-core --test run_items --test run_map_api --test run_content --test trace_protocol --test performance --locked --offline
cargo test -p spire-core --example m7_act2 --locked --offline

# Normal prefix, real nonattacking Death and labelled supplied diagnostics.
cargo run -p spire-core --example m8_act3 --locked --offline -- --scenario all --seed 11 --budget 8192
cargo run -p spire-core --example m8_act3 --locked --offline -- --scenario all --seed 7 --budget 8192

# Separate reference/performance integrity lanes; no pytest/network needed.
python3.12 -B -m unittest discover -s tests/python -p 'test_*reference*.py' -v
python3.12 -B data/cards/build_dataset.py --check
python3.12 -B data/powers/build_dataset.py --check
python3.12 -B -m unittest discover -s tests/performance -q
git diff --check
```

Examples are `m1_combat`, `m2_combat`, `m3_run`, `m4_rooms`, `m5_act1`, `m6_act2`,
`m7_act2`, `m8_act3`. M8 defaults11/all/8192; scenarios prefix/diagnostics/death/all.
Real Death exits0 with explicit noncompletion label. Faults, invalid args, budgets0/1
and over8192 exit nonzero; budget exhaustion is not a fabricated game outcome.

### Exact seven existing failure names

```text
combat::tests::full_cards::catalyst_permanent_card_state_carries_its_exhaust_flag
combat::tests::full_cards::catalyst_adds_twice_the_current_poison_and_exhausts_but_absence_is_noop
combat::tests::full_cards::artifact_prevents_one_poison_application_without_erasing_existing_poison
combat::tests::full_cards::silent_poisoned_stab_resolves_attack_before_base_or_upgraded_poison
combat::tests::full_cards::poison_ticks_at_afflicted_enemy_start_ignoring_block_then_decrements_and_can_kill
combat::tests::full_enemies::blue_slaver_and_two_fungi_encounters_construct_real_actors_across_ascensions
combat::tests::full_cards::silent_deadly_poison_applies_base_and_upgraded_amount_for_one_energy
```

Compare **underlying assertions**, not only names/counts. Ignore only thread IDs,
source line locations and the movable first-panic Rust backtrace hint. The ledger
documents earlier migrations' extra Debug-field normalization; do not introduce
broad normalization that hides new product-state differences. No M8 broad repair
of these old drafts was authorized or required. If M9 genuinely touches their
behavior, qualify and address the actual scoped defect instead of preserving a bug
merely for the count; preserve honest baseline history either way.

## 12. Native code map for the next planner

All paths below are relative to repository root.

| Area | Start here / why |
|---|---|
| Run facade and lifecycle | `crates/spire-core/src/run/{mod,engine,model,decision}.rs`; public `Run`, state, action/decision/terminal/fault contracts |
| Run/combat ownership | `crates/spire-core/src/run/combat.rs`; entry whitelist, `CombatOrigin`, item custody, gain synchronization/census, actual settlement |
| Room routing/profile isolation | `crates/spire-core/src/run/rooms/mod.rs`; scenario gates, actions/public filtering, completion/child routing |
| Existing Act3 prefix | `crates/spire-core/src/run/rooms/act3.rs`;35–43 capture/entry/source cutoff and Act3-local progress |
| Detached Act3 events | `crates/spire-core/src/run/rooms/events/act3.rs`; prepare/accepted/mutation/publication frame |
| Earlier acts and reward precedents | `crates/spire-core/src/run/rooms/{act1,act2,boss,rewards,shop,selection,transition}.rs` |
| Permanent mutations/rewards/RNG | `crates/spire-core/src/run/{mutations,reward,rng}.rs`; checked identities, hooks, once-only floor/key/terminal credits, scoped Falling RNG |
| Native topology/public map | `crates/spire-core/src/run/{map,world}.rs`; topology, preflight, encounter origins and privacy |
| Shared combat owners | `crates/spire-core/src/combat/{model,engine,effects,turns,hooks,powers,view}.rs` |
| Species setup/AI dispatch | `crates/spire-core/src/combat/enemies/mod.rs`, `act3.rs`, `act3/*.rs`; native bosses already implemented |
| Existing spawn custody | `crates/spire-core/src/combat/enemies/act2/spawn.rs`; extended by Reptomancer, not a second scheduler |
| Item/startup behavior | `crates/spire-core/src/items/{mod,relics,potions}.rs`; single owner, RedMask initial targets/Artifact |
| Current public M8 caller | `crates/spire-core/examples/m8_support/mod.rs`, `examples/m8_act3.rs`; explicit public versus diagnostic policy |
| Public integration evidence | `crates/spire-core/tests/m8_act3.rs`; start/resources/replay/privacy/loss/budgets/diagnostics |
| Native regressions | `crates/spire-core/src/combat/tests/m8_*.rs`, `src/run/tests/m8_*.rs` |
| Previous full-act pattern | `crates/spire-core/examples/m7_support/mod.rs`, `tests/m7_act2.rs`, native `m7_*` tests |
| Thin native env/bindings | `crates/spire-env/src/lib.rs`, `crates/spire-python/`, `python/spire_lab/gymnasium/`; existing bounded supplied-combat slice, not already full Run batching |

`RunTerminal` currently includes HeartVictory/Death/EmptyDeck/Act3NoKeys. Reward
code credits newly entered floors1 (Act4 floors10), distinct keys5, Death/EmptyDeck−1,
HeartVictory100 and Act3NoKeys0. Enum/reward match arms do **not** prove actual
end-to-end terminal reachability. M9 planning must trace the real room owner.

## 13. Source/reference policy and intentional deviations

Start new mechanics research with [data/README.md](../../data/README.md), relevant
domain guide/record/citations/uncertainties, then native code and milestone profile.
Do not create alternate dataset paths or repeat whole-library research.

- Cards370records; relics180; enemies65kinds/196moves/67contexts; powers133.
-27enemy constructor labels remain provisional;12power constructor IDs unresolved.
- Source finding index: `data/enemies/m8-static-findings.md`, with affected domain
  records/citations. Reference integrity is not implemented mechanics proof.
- Owned archive SHA256
  `cfad868ac8d65a88e71a0bf096fb09f78811e553effe0787c5309a655e081673` identifies
  the inspected bytes only. Its upstream **build/channel is UNVERIFIED**. Do not
  label it proven PCv2.3 or original-runtime validation.
- M8 had separately bounded named-method static lookup permission with existing
  tools; not original execution/install/whole-archive audit/private publication.
  Do not automatically expand this to M9. Name a real unresolved task-local source
  blocker and obtain any necessary separately bounded authority before access.
- User permitted resolving relevant `data/*` gaps and updating affected records
  after lookup; that is not blanket private audit, overlapping worker-edit or
  source-material publication permission. Preserve grades/disagreements.
- **RNG-001:** conditional laws/history/redirects/meaningful dependencies matter;
  original Java PRNG/seed/cursor/call parity is not required. Canonical permanent-ID
  ordering/native streams are deliberate. Do not flatten nominal AI probabilities.
- **TIME-001:** Portal's800-second condition is assumed satisfied, with other
  eligibility/ordinary selection retained. No clock/frame/wait subsystem or revived
  historical Reaper timer rationale.
- No universal original simultaneous-lethal/autoplay/foreign-flag parity claim.
  Preserve declared native sequential terminal/fault contracts and named exceptions.
- Wiki-derived data/snapshots are **CC BY-SA4.0**; builders/tests are project MIT.
  Do not apply MIT to wiki data or publish original game binaries/source/assets.

## 14. Preservation and collaboration rules

- **Never** reset/revert/discard unfamiliar edits, delete broad paths, pop/drop the
  stash, prune/remove worktrees, or force-push. Inspect before changing anything.
- Stash `a6daee8…f7b62` preserved41overlapping old-main public files during an
  earlier bounded merge. Independent user-requested assessment found no unique
  material needing salvage; assessment did not authorize deletion. Keep it.
- Local main is intentionally older than origin/main. Do not “repair” the main
  checkout or its many local tracker/evidence files to begin M9.
- Controller's accepted M8 follow-up task is closed. A different session should
  not continue committing on the merged branch by habit; establish its planning
  workspace/base safely, preserving these local handoff files.
- Future helpers/reviewers need **dedicated worktrees and unique build targets**;
  native creation can start at a stale base, so verify exact intended source. A
  clean fresh worker checkout can be switched non-destructively to the intended
  commit; uncommitted controller work needs explicit materialization/hash checks.
- No concurrent overlapping product edits. Native controller integrates complete
  candidates sequentially. Useful isolated Luna reading assistance is permitted
  by the selected method; do not spawn redundant researchers/review swarms.
- Default future approved execution method: native single-owner/TDD, useful isolated
  Luna factual assistance and **one** final independent milestone review. The three
  extra PR42 reviewers were specifically user-requested, not a new standing policy.
- Tests should be deterministic, small, behavior-focused and exact-value based.
  No sleeps/network fixtures/debug goldens, nominal no-op content or test weakening.
  Diagnose warm focused commands above5s / workspace above30s, separating build
  time; never assert wall-clock timing as correctness.
- Do not build Java parsers/symbolic executors/general validators/receipt platforms
  or launch whole-source audits to verify one mechanic. Use existing tools and
  named task-local evidence. No installs/removals/service/config/security changes.
- Optional CUDA/readiness tests are not a prerequisite for Rust/gym correctness.
  Historical Python readiness results are separate from this Rust/reference matrix;
  do not claim the whole Python extension was rebuilt/revalidated by M8.

The user uses Paseo: deliver server documents as absolute clickable `file:///home/...`
Markdown links, not only workspace-relative links or terminal instructions.

## 15. Suggested new-session startup prompt

> Read `AGENTS.md` and
> `docs/handoffs/2026-10-23-m8-merged-m9-session-handoff.md` in the existing isolated
> `full-run-gym-poc-20261022` worktree. M8/PR42 are accepted and merged; do not redo
> them. Verify current Git/Beads state and preserve the local handoff edits, every
> worktree/stash/evidence and seven historical failures. The next deliverable is
> ONE M9 written plan for my review, not implementation. Reuse accepted native
> mechanics and qualified references, retain natural losses and prior endpoints,
> and do not infer new commit/push/install/private-audit/cleanup authority.

If starting in another directory, first locate this handoff in the absolute
controller path above. A new worktree based on committed main will not include
this uncommitted document/AGENTS pointer or ignored evidence automatically.

## 16. What this handoff preparation changed

- Added this comprehensive document.
- Added a newest-entry `AGENTS.md` pointer/current merged-state warning.
- No M9 plan, product implementation, test/data/dependency changes, new agents,
  Git commit/push/merge or cleanup. No ongoing work is assigned to the old session.
- Verified document links/code entry paths and local state; did not rerun gameplay
  suites for documentation-only work. Recorded verification evidence remains local.

The next session owns fresh planning decisions. This document preserves context;
it is not a substitute for checking live tracker state or user approval gates.
