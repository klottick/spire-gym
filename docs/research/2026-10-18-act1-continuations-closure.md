# Act 1 continuation boundaries: bounded source findings and open contracts

Date: 2026-10-18. Research task: `spire-gli.2.2.4`.

**Status: bounded static findings; not whole-Act-1 closure or product approval.**
This note consolidates continuation/interface obligations for Neow, shops, boss
rewards, events, chests and campfires. It is independently readable and does not
depend on unpublished files. Mechanics below are static findings from the pinned
owned game archive and retained reconstruction, except where explicitly labelled
as a proposed native contract. No game code or initializer was executed. No
substituted game runtime, real profile/save, launcher or native library was used.

## 1. Evidence boundary and confidence

Static evidence establishes local branch/order facts, not that every legal path
reaches a branch or that all outstanding child effects finish before death or
handoff. The previous acquisition investigation covered 127 classes/25 event
branches, but did not close global screen returns, terminal arbitration, all relic
primary hooks, or original-runtime timing. Source identities and scoped evidence
are retained privately; public claims below are qualified summaries, not source
excerpts or translated algorithms.

The existing reviewed acquisition findings establish useful anchors:

* Neow candidates are grouped by reward category; category selection and its
  category-local candidate choice are distinct. Drawbacks precede reward
  activation, and nested choices have their own activation/update continuation.
* Shop proposals, restocks, card/relic obtain hooks, and reward claims have
  separate mutations; an offer is not a durable obtain. The reviewed merchant
  correction preserves normal/front selection for exhausted-tier promotions.
* An ordinary chest invokes ordered hooks, constructs linked reward records, and
  allows claim/skip; boss chests take a distinct path. Sapphire and Emerald key
  effects are deferred, and the Ruby recall path schedules a key effect before
  the room is marked complete.
* Event-owned fights remain event rooms. A local event/reward screen completion
  does not certify universal death/rescue arbitration or child-effect drainage.
* For Note for Yourself, incoming-card obtain/hooks precede outgoing selection;
  the outgoing live reference is persisted as card ID and upgrade count only on
  the subsequent room transition. This is not an immediate account/profile write.
* The reviewed Wheel correction is explicit: its gold mutation occurs at result
  reveal; later confirmation does not grant it again. Chest gold keeps endpoint
  multiplication before interpolation and binary32 rounding; do not replace it
  with factorized jitter.

These are local source conclusions. They do not establish a general callback
scheduler, natural reachability for every exceptional branch, or seeded RNG
parity. Shared scheduler/callback closure, RNG laws, and the all-141-relic audit
belong to other assigned work and are dependencies, not findings claimed here.

## 2. Finite parent/child and commit matrix

“Parent” means the active run decision/screen context; “child” is a nested choice,
effect, selection or reward. “Commit” below means the local mutation/order evidenced
for that path, not a universal durability guarantee. Cases marked **proposal** are
contract options, not adopted behavior.

| Parent → child | Resume / local commit established | Boundary that remains open |
|---|---|---|
| Neow → reward option → card/relic/transform selection | Category and local candidate selection are separate; drawback activation precedes reward activation; a nested selection can precede its own later activation/update. | Exact complete parent-screen stack and effect-drain rule across every Neow child are not closed. No return may bypass pending durable effects. |
| Shop → buy / purge / card choice | Purchase and purge mutations are distinct; restock and re-proposal are separate. A card-reward child can be opened by an acquisition path. | Exact close/reopen behavior for every child, purchase callback, empty-stock path, and terminal interruption is not globally proven. Do not treat current offer state as ownership. |
| Boss chest → relic selection → continue | Boss chest is not an ordinary chest: ordinary sapphire attachment/Matryoshka behavior differs. Boss relic selection is a child before map/next-act continuation. | The complete boss-relic → transition → Act2Ready ordering and all effects/observations between them remain an integration gate. |
| Relic offer → claim → obtain/equip hooks | A visible reward record is not itself ownership; the source claim schedules/enters relic obtain behavior, and relic-specific hooks may further mutate resources or open choices. | The assigned 141-relic audit owns the full primary-hook roster. This report does not close that roster or treat an unprocessed obtain child as committed. |
| Event → event choice → screenless obtain or selection | Local event effects and choices are source-specific; Wheel gold is already mutated at reveal. A selection may schedule card obtain work. | Generic `closeCurrentScreen` retains/reopens selected prior screens, while grid and card-reward branches have special cases; this is not proof that every event/child is covered or that terminal events resume. |
| Event → event-owned combat → reward/skip → map | Event-room combat follows event-specific routing; Proceed has a direct-map branch for the two reviewed event-combat classes rather than ordinary combat-reward reopening. | Death, escape, smoke legality, reward suppression, and already-running child completion are not globally resolved here. Preserve legal death and skip; never force a reward or completion. |
| Chest → hooks → reward(s)/key → claim or skip | Hook order can add/remove reward items; key flags change in the later key-effect update, not when the offer is built or claimed. Skipping is legal. | Key-effect dispatch under terminal arbitration and the exact “ready” boundary are shared-scheduler dependencies. Do not mark a key at offer/claim construction. |
| Curse obtain / prevention → deck removal | Reviewed paths include Omamori checks in card-obtain effects; Note’s incoming card path is direct and does not go through that effect constructor. Parasite’s master-deck object-removal hook reduces max HP by 3; the inspected max-HP routine floors max HP at 1 and clamps current HP, without itself invoking death. | Omamori’s already-done effect dispatch and complete curse caller family belong to shared scheduler/acquisition closure. Do not universalize the effect wrapper to direct obtains or model the local Parasite max-HP reduction alone as a death. Other removal hooks remain distinct. |
| Campfire → Rest → heal/hooks → optional card reward → room complete | Rest heals before `onRest` callbacks; Dream Catcher can create a card-reward child. Coffee Dripper’s predicate rejects the exact RestOption; Fusion Hammer’s rejects the exact SmithOption. Recall schedules the Ruby effect and locally completes the room; Ruby ownership itself is deferred. | The campfire applies every owned relic’s usability predicate to every option; the two named prohibitions do not imply the entire menu is unusable. If no option is usable, the source locally marks the room complete. All other relic combinations remain a separate roster audit. |
| Campfire → Smith / Dig / purge / relic option | Smith’s candidate list is based on current upgradeable cards; other options have their own selection and resource laws. Campfire buttons are filtered through relic usability predicates. | Empty/depleted inventory and exceptions must retain the source decision’s legal skip/leave behavior; no invented forced option, no silent empty-list fallback. Exact all-relic cross-effects await the separate roster audit. |
| Card reward → card / skip / Singing Bowl | Ordinary card rewards expose a dedicated Singing Bowl action; draft/codex/discovery/choose-one screens hide it. Its button handler calls the max-HP increase with amount 2, removes its associated reward item, records the bowl choice, closes the current screen, then hides the button. | Exact downstream semantics of the max-HP helper and unusual reward-record combinations are not independently traced here. Do not infer the action from the relic’s hover callback. |
| Match-and-Keep → reveal pair → match/miss → cleanup → map | Static event source has a finite board, five attempt counter, delayed pair resolution, matched-card obtain scheduling, cleanup phase and explicit map transition. A successful match does not equate to the event having drained all obtain effects. | Exact UI input/reveal timing, attempt consumption around final-pair timing, malformed/exhausted inventories, and parent resumption remain unverified. Do not expose hidden card identities/order before reveal or turn UI timing into a gameplay timer. |
| Any parent → death / terminal | **Preservation requirement:** death is a legal terminal outcome; already-applied durable effects remain applied. | No universal return, rollback, forced completion, or “drain everything” rule is established. Faults are not defeats. |
| Act1Complete → Act2Ready | **Proposal:** distinguish completed Act 1 from the explicit successor-ready handoff and retain the resolved run-owned state needed by continuation. | Exact boundary, output schema and death/fault behavior need main’s whole-spec decision; this research does not invent a product API or mark all offered/acquired items as committed. |

### Screen-return interpretation

The inspected screen manager records a prior screen, performs screen-specific
cleanup, and can reopen selected card reward, combat reward, shop, grid and boss
reward screens. Grid closure can preserve the combat-reward parent; card-reward
closure calls its own close hook; event-owned combat can bypass ordinary reward
reopening. These branches are evidence that nested continuation is stateful, not
evidence of a universal stack discipline. A native model should represent the
active parent and pending child explicitly and resume only after the child’s
source-defined commit/skip and required durable effects resolve. The exact
representation is a **proposal** for whole-spec review, not a source API.

## 3. Explicit profile-output boundary options

The existing profile design proposes detached input profiles, a detached
non-policy profile-effect result, no automatic update of later vector-reset
profiles, and no manufactured completed profile after a runtime fault. Source
research adds an important distinction: Note’s selection and its next-transition
file persistence are different points. The output contract is therefore still
open. Main should explicitly choose and specify one boundary rather than silently
equating “outgoing card selected,” “event locally complete,” and “profile saved.”

Options for whole-spec review (none is approved by this report):

1. **Selection result:** return a detached proposed profile delta at outgoing-card
   selection, tagged as pending the source transition boundary. A caller must not
   treat it as persisted/committed.
2. **Transition result:** expose the profile delta only after the modeled
   next-room-transition boundary has applied the source’s card-ID/upgrade write.
   Define what death before that boundary returns.
3. **Separate staged and committed outputs:** retain the proposed exchange and a
   distinct committed result, with explicit event/transition status. Define fault
   behavior without rolling back already-applied run effects or fabricating a
   successful profile result.

For all options, a vector auto-reset must not implicitly propagate a prior slot’s
profile delta; a later run receives its configured detached creation profile.
Replay must use the captured profile and accepted decisions, not reread external
files. Malformed profile IDs and profile-write failure are administrative contract
questions, not game deaths or permission to fabricate a fallback.

## 4. Bounded choice/interface requirements

* Singing Bowl is a distinct explicit reward action where the source screen
  enables it. Its local action is a +2 max-HP call plus removal of the associated
  card reward, followed by the source’s screen-close path. Keep card/skip/bowl
  actions distinct; the exact max-HP helper contract and unusual reward-item
  cases remain to be independently traced.
* Match-and-Keep policy observations may reveal only the cards currently exposed
  by the event’s reveal rules. Selection/attempt state is explicit; no constructor
  shuffle order or unrevealed identities leak into observations. The precise
  selection convention and attempt timing require source confirmation.
* Campfire candidate menus reflect the source’s option-level usability checks.
  Coffee Dripper rejects RestOption; Fusion Hammer rejects SmithOption. The
  menu then evaluates all relic predicates across options, and the source marks
  the room complete only when none remain usable. These facts do not close every
  relic combination. Empty upgrade/purge lists do not authorize a fabricated
  selection.
* Chest, key, event, reward and relic changes commit at their actual resolution
  sites. Legal skip/death/key deferral remain legal. Never force COMPLETE merely
  because the UI has no currently visible choice while a durable child remains.
  Omamori prevention is attached to specific obtain-effect paths, not every
  direct add; Parasite removal preserves its source max-HP loss without treating
  that mutation alone as lethal.
* Neow/shop/boss/event/chest/campfire parent identity and child resolution must
  survive close/reopen/return. Preserve each source’s direct-map special case and
  pending effects; a universal “return to map” shortcut is not supported.

These are implementation-independent acceptance obligations, not executed tests.
Candidate ordering, choice identities and hidden knowledge still need the complete
run schema and separate RNG contract.

## 5. Evidence grades, verification, and remaining limits

Evidence is static reconstruction plus focused bytecode checks recorded in the
private continuation lane. Relevant retained source members include screen
close/reopen, proceed routing, event-local acquisition, Neow/shop/campfire,
Match-and-Keep, key effects, Note, and card reward. The prior independent
acquisition review corrected and rechecked merchant promotion direction, Wheel
gold timing, and chest binary32 endpoint arithmetic. One new focused static
bytecode inspection covered the exact unique `SingingBowlButton` class; no
numerical model or runtime measurement was performed.

No game code, initializer, substituted runtime, real profile/save, service,
network probe, or native library ran. This note does not close the 141-relic
primary-hook audit, universal callback/effect closure, full RNG laws, exhaustive
screen-return stack, other relic campfire predicates beyond the two named boss
relics, Singing Bowl max-HP helper semantics and unusual reward-item cases,
Match-and-Keep
input/attempt UI details, empty-inventory exceptions, or
the exact Act1Complete/Act2Ready owned-profile contract. Those remain explicit
dependencies for the whole-Act-1 specification and fresh independent review.
