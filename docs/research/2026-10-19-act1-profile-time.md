# Act 1 profile, Note persistence and headless time: decision draft

Date: 2026-10-19. Task: `spire-gli.2.3.7`.

**Status: source-grounded findings plus RECOMMENDED native contracts, not approved
behavior.** This standalone supplement resolves the former selection-versus-save
ambiguity into a concrete candidate contract. It does not approve the whole Act 1
specification, an implementation plan, implementation, or original-game execution.
“Committed” below means a native run-owned value, **never a write to a real account**.

## 1. Authority and evidence grades

Scope remains standard, unmodded, fully unlocked, Act-4-enabled Ironclad A0 Act 1,
including every legal Note/off-color route, death, skip and resolved Act2Ready.
Other characters' cards do not change the actual character or grant their starts.
The existing architecture requires Rust-owned gameplay, detached creation/final
values, explicit single reset, independent vector same-step reset, from-start
replay, invalid-whole-batch nonadvancement and administrative post-dispatch faults.

Evidence labels used here:

* **S-retained:** read-only retained static reconstruction, including previously
  reviewed acquisition/continuation facts; not a fresh runtime result.
* **B-new:** this lane freshly checked the named method in disassembly, using an
  exact unique archive entry. Selected dependencies also have reconstruction.
* **M-proposal:** an independently authored private model checked the proposed
  profile lifecycle/time arithmetic. Agreement checks the proposal, not the game.
* **P:** a native design choice recommended for whole-spec review. Not a source
  fact, approval, test result, or claim of original timer parity.

Archive SHA-256:
`cfad868ac8d65a88e71a0bf096fb09f78811e553effe0787c5309a655e081673`.
Inspection used pinned Temurin 21.0.12.1+1 and CFR 0.152; four input/tool pins were
checked before and after each new inspection batch. This establishes owned-byte
identity, not upstream channel/build identity. No game class, initializer,
instrumented original class or original runtime executed. No real saves/profiles,
launcher, downloads, installs, services, GPU, benchmarks or training were used.

Related merged findings, without reliance on unpublished drafts:
[continuations](2026-10-18-act1-continuations-closure.md),
[execution](2026-10-18-act1-execution-closure.md),
[native ownership](2026-10-18-act1-native-contract-closure.md), and
[Acts 2/3 and the clock obligation](2026-10-18-ironclad-a0-acts23-research.md).
The read-only earlier profile/Note/architecture drafts were context, not copied
implementation or separately approved profile defaults. Existing combat-v1 and
[RNG-001](../intentional-deviations.md) identities/meaning do not change.

## 2. Source facts: Note is not an immediate profile transaction

| ID / grade | Established boundary | Consequence; bounded qualification |
|---|---|---|
| N1 / B-new + S-retained | `NoteForYourself.initializeObtainCard` reads stored ID and upgrade count, makes a fresh card and applies upgrades. Missing preferences use Iron Wave / zero; an unknown library ID locally falls back to Iron Wave. | No saved misc, growth, temporary cost, bottle flags or old object identity is read. The original malformed-ID fallback is **not** the proposed native validation policy. Individual cards' construction/upgrade laws remain content responsibilities. |
| N2 / B-new + S-retained | The Take branch invokes held-relic obtain hooks on the incoming object, adds it to the master deck, then invokes master-deck-change hooks, **before** constructing outgoing selection. | This direct obtain is not the ordinary obtain-effect/Omamori wrapper. The updated deck, potentially including the incoming card itself, is the selection basis. The callback lane owns the transitive hook consequences. |
| N3 / S-retained | Outgoing candidates come from purgeable cards, then the bottle filter. The examined purge helper excludes Necronomicurse, Curse of the Bell and Ascender's Bane by identity; the bottle filter excludes the three bottle flags. | A card's color or later-act origin alone is not an exclusion. A fabricated temporary-only permanent card is not a legal stored input merely because a helper would accept it. |
| N4 / B-new + S-retained | `NoteForYourself.update` removes the selected actual master-deck object through object-removal, then retains that same object as `saveCard`. | Removal and its card/relic notifications precede saving the reference. This is not a frozen selection-time ID/upgrade tuple. Full removal-hook closure and global screen return belong to the continuation/execution lanes. |
| N5 / B-new + S-retained | `AbstractDungeon.nextRoomTransition` detects departure from this Note event with a saved object; it writes its then-current card ID and upgrade count and flushes preferences. | Persistence occurs on the subsequent room transition, not Take, outgoing selection, event-local COMPLETE or opening the map. No misc field is written. The write precedes later transition cleanup/room-entry work, so a later fault need not imply that nothing committed. |
| N6 / B-new | The A0 availability branch accepts non-Daily ascension level zero without consulting a prior win/cleared-Ascension history. | In-scope A0 Note eligibility is not a configurable `note_enabled` switch. This does not make the event guaranteed or close event selection probability. |
| N7 / B-new + S-retained | Ignore creates no selected outgoing saved reference. | No Note replacement is due merely because the event was visited. Source screen COMPLETE is not a universal durable-effect-completion predicate. |

These are local ordering facts, not a transactional filesystem guarantee. In
particular, the source's two preference mutations and flush do not establish
crash-atomic storage or rollback. The native contract below performs no file I/O.

### 2.1 Proposed native Note lifecycle — RECOMMENDED (P)

Use a run-owned exchange record with these distinct states:

| State | Run-owned meaning | Detached non-policy output |
|---|---|---|
| `None` | No accepted exchange; Ignore stays here. | No staged replacement; original committed Note value. |
| `IncomingApplied` | The source-specific incoming mutations reached their actual checkpoints; outgoing selection has not resolved. | Progress tag only; no invented outgoing card or committed exchange. |
| `PendingTransition` | Outgoing removal finished; retain a stable owned reference/identity to that removed object, including its live rule-relevant fields. | A **staged snapshot**, explicitly noncommitted, of currently observed ID/upgrade plus exchange identity. |
| `Committed` | At the next source-qualified departure checkpoint, sample the retained object's current ID/upgrade and replace the native committed Note descriptor once. | Detached committed descriptor, revision and commit checkpoint; clear the pending reference. |
| `AbandonedBeforeTransition` | Genuine task death occurs before that checkpoint. | Retain prior committed descriptor; staged diagnostic snapshot may be reported as abandoned, never as the next Note value. |

The native commit is one typed in-memory replacement; do not emulate two separate
filesystem writes. This is a **P** representation choice, not source crash parity.
The stored descriptor contains exactly `card_kind` and `upgrade_count`; a staged
snapshot cannot become the later commit by mere aliasing. Preserve live scaling
through removal hooks, then omit it from the stored descriptor. Fresh retrieval
uses the card's construction/upgrade law: for example, Ritual Dagger/Genetic
Algorithm prior misc growth is not imported, unlike Searing Blow's upgrade count.
Do not reset an outgoing live card before its removal callbacks to achieve this.

### 2.2 Detached profile candidate and interruption — RECOMMENDED (P)

`ProfileCandidate` is a **partial-run candidate**, not a completed-account result.
Its required semantic fields are: profile-contract identity; captured creation
profile; current committed Note descriptor; monotonically increasing Note revision
(initially zero); optional exchange identity/staged snapshot/status; newly observed
boss-discovery additions; current-run Neow-eligibility evidence; and outcome/status.
Stable internal object/episode identities stay outside policy features. Transport
codes/offsets belong to the native-schema lane, not a second schema defined here.

Each successful read/step returns detached copies of this non-policy result. A
staged result is inspectable but **not installable as a creation profile**. A
successful terminal result freezes it before replacement/reset. Later observations,
steps, caller edits, reset and close cannot mutate previously delivered values.

| Interruption | Required candidate/final behavior (P) |
|---|---|
| Death before outgoing selection | Keep every already-applied incoming mutation; no invented outgoing selection, transition, Note commit or completed exchange. |
| Death after removal, before transition | Keep actual run mutations, abandon stage, retain old committed Note. Do not force transition to save it. |
| Death after Note commit | Retain that commit; death is not rollback. Return actual death final and a partial-run candidate. |
| Fault before commit | Quiesce/fault the whole pool; preserve native progress and diagnostics. No successful step/final/profile result. Earlier delivered staged outputs remain staged. |
| Fault after commit, including later room-entry, packing or export failure | Keep committed descriptor/revision internally and in fault provenance; no fabricated success, rollback, defeat, retry or reset. A bounded diagnostic copy may describe the known committed prefix but is not a normally delivered `ProfileCandidate` or completed profile. |
| Act1Complete/Act2Ready | Capture candidate and final only after boss rewards, relic take/skip and required children/handoff work resolve. Completion must not itself flush an unrelated pending Note exchange. +1 is for resolved Act1 handoff, not profile delivery. |

Fault diagnostics must identify the last reached checkpoint and revision, distinguishing
“commit known reached” from “output delivery failed.” If allocation prevents a
diagnostic copy, preserve fault provenance without promising a recoverable result.
No ordinary API can keep stepping/reading a faulted pool as though it succeeded.
This reconciles durable partial commit with the ban on manufactured completed
profiles. Caller-side persistence, if any, is outside gameplay and never automatic.

Finite alternatives for the human review, **none approved**:

| Alternative | Tradeoff | Recommendation |
|---|---|---|
| Selection-only result, tagged pending | Small interface, but callers can mistake removal for persistence; cannot represent a later live-object change without another result. | Do not use as the sole output. |
| Committed-only result at departure | Harder to misuse; hides useful exchange progress and death-before-transition distinction unless separately reported. | Acceptable smaller alternative if all progress tags remain explicit. |
| Separate staged snapshot and committed revision | Represents actual boundaries, abandonment and partial-fault provenance; more statuses to specify/test. | **RECOMMENDED.** |

## 3. Meaningful creation history, defaults and replay

### 3.1 Source history is not “fully unlocked”

| ID / grade | Source finding | Native implication (P where a choice) |
|---|---|---|
| H1 / B-new | Standard-run Neow reads the character-specific saved spirits value; zero selects the two-option shortlist, nonzero the ordinary four-option branch. The inspected standard death and saved-run-abandon paths set eligibility according to floor **at least 16**, not a required boss victory. | Represent the relevant prior Ironclad eligibility as a boolean, not a “bosses defeated” count or a user-picked blessing list. Reaching the boss-floor boundary is the meaningful distinction; the source predicates are the exact anchor. |
| H2 / B-new | `Settings.isStandardRun` excludes Daily, trial and explicit original `seedSet`; the separate original seeded Neow branch supplies eligibility. | A reproducible **project RNG seed does not mean original custom-seeded mode**. Keep standard-run history semantics; do not force four blessings whenever native replay has a seed. |
| H3 / B-new | Exordium prioritizes the first unseen boss: Guardian, then Hexaghost, then Slime Boss. Only after all are seen does the normal branch shuffle all three. A forced singleton is duplicated internally, not two A0 fights. | Fully unlocked and all-bosses-seen are distinct inputs. Do not override unseen priority with a uniform boss roll. The exact all-seen shuffle law is another lane's obligation. |
| H4 / B-new | Each of those three bosses marks its own discovery in `usePreBattleAction`; `UnlockTracker` stores discovery separately from unlock progression. | Record a discovery addition at that hook's actual checkpoint, even if the player later dies. Neither boss announcement, boss selection nor victory is the substitute checkpoint. |
| H5 / S-retained | Acts 2/3 have their own discovery priorities, as recorded in the merged Acts 2/3 report. | Carry per-act discovery sets, not one `all_seen` boolean. This document freshly verifies only the Act 1 three-boss hook/priority slice, not every later-act discovery hook. |

### 3.2 Creation profile candidate — RECOMMENDED (P)

Require an owned `RunCreationProfile` with an explicit contract identity and:

* fixed standard/unmodded/fully-unlocked/Act4-enabled Ironclad A0 rules;
* prior Ironclad Neow eligibility, independently of discovery;
* per-act seen-boss sets: Act 1's three above; Act 2 Champ/Automaton/Collector;
  Act 3 Awakened One/Donu-and-Deca/Time Eater;
* stored Note card kind and nonnegative upgrade count, validated against the
  supported standard permanent/removable-card vocabulary and that kind's actual
  upgrade law. Searing Blow is not limited to one upgrade by a generic boolean.

Raw profiles have **no omitted-field defaults**. Provide a named, versioned
`ExperiencedA0` preset, **RECOMMENDED** as the convenience default: prior Neow
eligibility true, all nine ordinary bosses seen, unupgraded Iron Wave stored.
Resolve that preset into the complete detached value before initialization/replay.
This is an experienced-profile proposal, **not a consequence of unlock flags** or
a claim that first-use game preferences have those history values. Also accept
explicit false Neow eligibility and partial/empty discovery sets without requiring
the caller to reconstruct/authenticate a prior real account. A named novice-history
preset (false, empty sets, Iron Wave zero) is a finite optional alternative, not a
locked-content mode. Neither preset excludes legal variants from content coverage.

No arbitrary starting-deck/resources/orb/stance/other-character override; no
redundant A0 cleared-Ascension array, note toggle, next-boss override, real-profile
path or imported checkpoint. Do not infer Note storage support from rarity/color
alone: the twelve distinct other-character basics and removable later-act event
cards remain required; temporary-only cards, standard-unreachable records and
the three nonpurgeable curses do not gain routes through arbitrary input injection.
The exact legal stored-card ID/upgrade catalog is a required content interface,
not an assertion that this profile supplement has audited all card upgrades.

Validate all requested rows, kinds, counts, history values and transport arithmetic
before RNG draws, episode reservation/installation or elapsed-time advancement.
Malformed data rejects the **whole batch** unchanged. Native malformed-ID rejection
rather than the original Iron Wave fallback is a deliberate **P** boundary; it must
be included in the final specification's error contract, not called a source rule.

### 3.3 History output, reset and replay — RECOMMENDED (P)

Keep `boss_discoveries_added` separate from `neow_eligibility_evidence`. On a genuine
Act1 death, the latter can give the observed floor predicate and resulting boolean.
On Act2Ready report `BossFloorReached`, **not `FullRunCompleted`**: future completion
history is not yet available. A caller may explicitly build a subsequent creation
profile using the committed Note and discovered-boss union; it must explicitly
choose the next run's prior-Neow boolean. No implicit “apply all output as account”
or assumption that Act1 termination is a full-run death/win.

Every explicit reset and vector replacement takes its configured detached creation
profile, not the preceding slot's candidate/history/Note. Capturing final data must
precede replacement. A fault yields no normal terminal auto-reset. Replay records
the resolved profile, rules/content/observation/task/RNG/profile/time identities,
accepted semantic choices and accepted elapsed inputs. It reconstructs from the
start, never rereads mutable callers, defaults, files or global discovery state.
Commit revisions/checkpoints are replay comparison outputs, not imported authority.
Profile results and replay secrets are non-policy. Note contents become policy
information only when the event actually reveals them, not at creation or through
candidate ordering. Discovery/history fields likewise need legitimate public
projection rather than blindly exporting the entire creation object.

## 4. Source clock and Reaper: explicit dependency trace

| ID / grade | Source dependency | What it establishes; what it does not |
|---|---|---|
| T1 / B-new + S-retained | `CardCrawlGame` dispatches dungeon updates during gameplay, with a separate transition-screen path; new-run reset clears playtime and stop-clock state. `AbstractDungeon.update` adds graphics frame delta to binary32 playtime unless `stopClock`, before its screen dispatch. | Run playtime is not CPU usage, floor count, action count or just combat time. A screen being open does not by itself bypass that increment. No host/frame distribution is established. |
| T2 / B-new + S-retained | `AbstractDungeon.getShrine` admits Secret Portal only in TheBeyond with playtime **>=800 seconds**. Its selection remains part of the filtered one-time pool. | Meeting the threshold makes it eligible, not guaranteed; it does not retroactively change an already sampled event. The portal is still not Act 1 content. |
| T3 / B-new | `Reaper.use` schedules a zero-duration Reaper VFX action before its vampire multi-target damage action. The damage action starts at `ACTION_DUR_FAST` (initialized to 0.25 seconds), ticks frame delta and performs damage at completion. | Damage, retaliation and later healing are distinct checkpoints. No lost-heal outcome follows from the constant alone. |
| T4 / B-new | After damage, the vampire action appends zero-duration flying-orb VFX actions, conditionally a 0.3-second wait outside fast mode, then a HealAction if healing is positive. When all monsters are basically dead it filters pending post-combat actions. It separately inserts a 0.1-second wait at the front regardless of that condition. | VFX actions still need dispatch/update if retained: zero duration is not zero scheduler visits. The last-kill filter removes those pending VFX/waits while preserving healing, but is not the later full clear. The observed VFX emission bound is not a gameplay damage/healing cap. |
| T5 / B-new | `WaitAction` shortens waits above 0.1 seconds in fast mode; `AbstractGameAction.tickDuration` completes on a strictly negative residual. `HealAction` invokes healing on its **first update**, before its later duration completes. | Exactly zero and negative are different; action dequeue and first update are different. A heal need not wait for its own duration to finish, but must actually reach first update. |
| T6 / B-new | `GameActionManager.update` updates an unfinished detached current action, or on a later update clears it and chooses subsequent work. `AbstractRoom.update` advances monsters before ordinary manager dispatch; modal branches can change manager dispatch. | A single FIFO loop or “keep draining until empty this frame” is not the source schedule. Entered callbacks are not unwound by clearing pending work. |
| T7 / B-new | Monster death timers are decremented by frame delta; ordinary `die` adds 1.8 seconds normally / 1.0 fast, while boss-victory logic adds its own delay. Negative death timer can set dead and invoke room `endBattle` when all are dead and loss is permitted. | Basically-dead versus dead differs. These values depend on prior timer state, overrides and update order; not a fabricated Reaper-specific cutoff. |
| T8 / B-new | `endBattle` marks battle over, runs victory hooks, then clears manager work; its later room-completion delay is separate. Final-boss logic can stop the play clock if the final-act/key route is unavailable. | Retained pending heal can still be cleared before its first update. Play clock stop is not a universal scheduler pause or an Act1 boss stop rule. |
| T9 / B-new | The inspected `ReaperEffect` body emits sound/lightning visuals; `FlyingOrbEffect` updates its own visual movement/completion and does not invoke player healing. `VFXAction` enqueues the effect and ticks its own duration independently. | Do not wait for an orb to visually reach the player as the heal's commit point. These bodies do not certify every transitive visual class as removable. |

This trace closes the named timer dependencies, **not** all transitive gameplay
callbacks/monster overrides/effect sites. Guardian Sharp Hide retaliation is an
already established ordering exposure. Natural original Reaper heal-loss and its
frame distribution remain **unproved**. Death-screen dispatch, rescue/heal modifiers,
room children and shared terminal arbitration are interfaces to the execution and
continuation lanes, not duplicated full audits here. No safe-runner/sandbox survey
or original runtime is needed or authorized to make these bounded static findings.

## 5. Concrete time contract — RECOMMENDED, not approved (P)

Use **two explicit, separately versioned quantities**, never host CPU/wall time:

1. **Scheduler time:** canonical virtual 60-Hz frames, `FAST_MODE=false` fixed for
   this rules profile; each scheduled gameplay update receives the same nearest-
   ties-to-even binary32 representation of 1/60 second (bits `0x3c888889`). Timer
   subtraction/comparison remains binary32, not a host floating-point timestamp.
   Preserve the named source update order, guards,
   strict comparisons, first-update effects, current-versus-pending distinction
   and source-qualified cancellation. A completed action is retired/chosen on the
   next manager visit, not recursively drained. Even a zero-duration VFX action
   gets its actual first update. Pure rendering may be omitted, but not a callback,
   timer or dispatch visit on which gameplay depends. Record completed frame counts
   at decision/commit boundaries. Never assign a special Reaper heal-success rule.
2. **Run elapsed time:** exact nonnegative integer units of **1/3,000,000 second**.
   Start at zero; each running-clock canonical dungeon frame contributes 50,000
   units. Explicit accepted decision dwell contributes `elapsed_us * 3`. Stop-clock
   suppresses this contribution, not scheduler progress. No host delay, worker count,
   Python call cost or diagnostic recording cost contributes. Freeze the task's
   captured time after its terminal boundary; do not simulate death-screen idling
   as gameplay. Act2Ready retains the run's running/stopped flag and elapsed value
   for later continuation, rather than forcing the carried run's clock stopped.

Use checked signed-64-bit nonnegative storage/transport for elapsed units and
microseconds. Exhaustion is an administrative numeric fault, not a gameplay time
cap. Preflight accepted input multiplications/sums before dispatch where possible;
post-dispatch automatic accumulation exhaustion obeys pool-fault/no-rollback rules.
The exact Portal threshold is 2,400,000,000 units. This deliberately avoids long-run
binary32 accumulation drift for eligibility; it is **not binary32 playtime parity**.
The 60-Hz schedule is a declared deterministic surrogate, not measured original
FPS, a distribution over original frames, or permission from RNG-001.

### Waiting and sampling contract (P)

At an actual settled player decision, submit nonnegative integer `elapsed_us`
alongside the action/token. Omission explicitly means zero **decision dwell**,
not frozen automatic time. Preflight every row before changing time. Charge dwell
immediately before applying the accepted action, without regenerating its menu,
rerolling existing offers or processing a second player turn. Reject dwell at
non-settled/suspended/terminal/faulted states; do not wait past unresolved mandatory
effects. The scheduler owns automatic frames, never a caller-supplied frame count.

For a policy-facing finite menu, recommend a single `WaitTo800Seconds` timing
candidate at running-clock settled decisions below the known threshold; it advances
only the exact remaining elapsed units, preserves the parent decision and generates
a new token. Its accompanying `elapsed_us` must be zero, avoiding double charging;
the exact-unit jump need not be rounded to a whole microsecond. This explicit
headless timing abstraction adds no combat turn, reward,
RNG draw, offer refresh or key. It must not depend on whether a hidden future room
is Portal. Once the threshold is reached the candidate disappears; no infinite
microsecond action enumeration or dense machine-range mask. This candidate may be
offered already in Act 1 for forward-compatible carryover, though the Portal itself
remains Act 3 only. Explicit dwell remains available to controlled/replay callers;
the finite wait is how a policy can intentionally meet the known content gate.

Only **quiescent** decision dwell can use this elapsed-only shortcut. It cannot
accelerate a Reaper/death race: those are automatic scheduler frames with no player
decision inserted between damage and healing. If an apparently settled UI can still
run a mechanic-bearing timer/callback, its source-specific continuation must first
define settlement or require the canonical frame path; do not classify it as idle.
The exact all-content settlement registry remains an execution-lane dependency.

At event generation, test the elapsed value reached **at that sampling checkpoint**,
including prior accepted dwell and automatic frames. Waiting afterward cannot
replace an already generated event. Keep elapsed/clock-running state public in the
run schema; a policy's timing lever cannot be hidden in non-policy diagnostics.
Keep scheduler internals private unless an actual source rule reveals them.

Replay records time-contract identity, fixed scheduler mode, every accepted dwell
or wait candidate, and time/frame-count comparison checkpoints. Replay ignores host
timing and rejects mismatched time identities. Invalid/rejected requests add no
time or replay action. Different worker counts must yield identical per-run time,
choices, commits and observations. Opt-in recording cannot change the schedule.

### Finite alternatives and costs (all P, none approved)

| Option | Strength | Cost / limitation | Disposition |
|---|---|---|---|
| Fixed 60-Hz source-order scheduler + explicit dwell/exact playtime | Reproducible, preserves named timing dependencies; gives waiting and Portal a concrete replay law. | Adds frame work; exact playtime differs from binary32 accumulation; full mechanic-bearing settlement/timer registry still required. | **RECOMMENDED** candidate above. |
| Recorded per-frame delta stream with source binary32 playtime | Can compare a supplied frame trace without pretending CPU time is game time. | Large records, explicit mode/screen/input timing needed; supplied trace is not proof of natural original timing; no authorized original capture exists. | Finite comparison alternative, not default. |
| Source-order semantic checkpoints with a declared Reaper completion/cancellation policy | Faster and simpler; can retain explicit elapsed/wait for Portal. | Must explicitly choose the Reaper race behavior and document a timing deviation; “always drain/heal” is not established. | Not recommended until that narrower fidelity decision is written. |

Always-zero run time, always-enabled Portal, CPU/wall-time gates, hidden per-step
time charges, unconditional heal/drain and choosing a lost-heal threshold without
evidence are **not** alternatives selected or approved here.

## 6. Acceptance obligations and exact remaining uncertainty

The following are finite **future native acceptance cases**, not executed gameplay
tests. Private M-proposal checks cover only selected lifecycle/arithmetic rows.

| Case | Required discriminator |
|---|---|
| A01 | Obtain hooks and incoming deck change happen before outgoing candidates; incoming itself may qualify. Distinct direct-obtain/Omamori behavior; screen return delegated to its actual owner. |
| A02 | Ignore leaves Note unchanged; first-use Iron Wave zero differs from native malformed-ID rejection. A0 availability does not require a prior win. |
| A03 | Outgoing bottle/nonpurgeable restrictions and removal notifications; if a callback changes the retained object, commit samples its later ID/upgrade. No saved misc. A synthetic mutation discriminator is not proof of a naturally reachable mutation. |
| A04 | Stage delivered after selection remains noncommitted and detached after commit; repeated reads/transition completion do not increment revision twice. |
| A05 | Death before selection, after removal/before departure, and after commit produce the three distinct outputs in §2.2; no forced save/return. |
| A06 | Inject faults before commit, immediately after commit, at room entry and during export; durable prefixes persist, no success/fake death/auto-reset/retry. Old outputs remain valid. |
| A07 | All eight Act1 seen-boss subsets follow first-unseen priority, or the reviewed all-seen shuffle law. Pre-battle discovery survives later death; announcement alone does not add discovery. |
| A08 | Prior Neow false/true gives shortlist/ordinary menu independent of unlock and seen sets. Floor15/16 genuine death evidence differs; project seeded replay does not force original custom-seeded semantics. |
| A09 | Cross-character basics, each removable event card, scaling-reset cards and repeated Searing Blow upgrades retain legal identities without importing another start or home event. Exact card upgrade cases belong to the content catalog. |
| A10 | Caller/profile edits, sibling slots and preceding finals cannot change a live run/replacement. Terminal candidate is captured before same-step reset. Replay uses resolved original inputs, not current preset defaults. |
| A11 | Time just below/equal/above 800 seconds in Act3 versus Act1/2; eligible is not guaranteed. Waiting before event sampling can affect eligibility, waiting after cannot reroll. |
| A12 | Running/stopped play clock; fixed-frame versus zero-duration action; exact-zero versus negative residual; normal/fast source differences. Native mode is fixed normal; unsupported alternate mode must not silently change it. |
| A13 | Reaper nonfinal hit, last kill, retaliation/rescue, player death, no-heal amount and heal queued/current/first-update states; vary VFX count and death-timer state under the declared canonical schedule. Assert exact chosen native outcomes, not original parity or universal heal loss. |
| A14 | Wait candidate offered independently of hidden future events; no RNG/turn/reward/menu refresh; no candidate above threshold. Invalid elapsed type/negative/overflow in the last batch row leaves every row unchanged. |
| A15 | Serial/parallel/from-start equivalence includes accepted dwell, automatic frame counts, time thresholds and Note revisions; external packing faults are not falsely replayed as deterministic game faults. |

**Executed verification:** 22 private standard-library tests passed: five input/
output guards (including changed pins, missing/duplicate entries, containment,
exclusive outputs and JVM environment stripping), and eleven independent proposal
checks for staged/committed/death/fault/detachment/reset and elapsed/threshold/replay
arithmetic, plus six document structure/negative-control checks. The modeled
live-reference mutation is a synthetic discriminator, not a legal-run witness.
Their initial missing-module failures were retained. They do not test
source gameplay, Reaper outcomes, numeric exhaustion or the public bindings.
New inspection covered 27 unique named classes with 46 successful javap/CFR
invocations. An attempted wrong-package vampire-action entry failed closed with
zero matches; it and the corrected inspection were preserved. No product builds or
suites were run for this documentation-only lane.

**Remaining required work, not disguised defaults:** independent review of these
recommendations; integration with the other lanes' actual Note parent-return,
shared callback/terminal and numeric-schema contracts; legal stored-card/upgrade
catalog closure; all-seen boss shuffle law; later-act discovery-hook confirmation;
and the complete mechanic-bearing timer/settlement registry before claiming a
whole-game canonical scheduler. Original frame/delta distribution and natural
Reaper heal-loss remain unknown. Safe static evidence cannot label them observed.

**Small human choices for the whole-spec review:** accept separate staged/committed
profile candidates and explicit experienced preset? Accept the canonical-normal
60-Hz scheduler with exact run elapsed time and explicit timing/wait inputs, including
the disclosed nonparity and finite wait abstraction, or choose one listed alternative?
These recommendations are not new approval gates; controller presents them within
the single complete Act 1 specification. Whole-spec approval, subsequent plan
review and separate implementation-start authorization remain outstanding.
