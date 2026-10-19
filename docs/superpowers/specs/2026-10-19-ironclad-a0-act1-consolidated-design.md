# Ironclad A0 Act 1 — consolidated specification

Date: 2026-10-19. Author task: `spire-gli.2.3.8`.

**Whole-scope integration draft — BLOCKED, not approved or implementation-ready.**
This is the single review surface for standard Ironclad A0 Act 1. It preserves
the approved architecture, states candidate detailed contracts, and accounts for
required content even where a law is unresolved. It is not an implementation plan,
an implementation claim, or permission to install, build, benchmark, or train.
Main owns integration of independently reviewed follow-ups and the eventual human
presentation. This revision integrates only four main-authorized, independently
cleared and merged follow-ups, plus the earlier Note/Silent corrections. Technical
clearance of a proposal is not human approval of its behavior.

## 1. Reading this specification

The obligation is **one complete Act 1**, not a resource foundation or a collection
of independently shippable partial simulators. A blocked requirement stays in scope.
Successful sampled runs, enumerated IDs, and local static proofs cannot silently
close its dependencies.

Status notation used throughout:

| Mark | Meaning |
|---|---|
| A | Previously accepted architecture/explicit user constraint, restated here without asking for reapproval. |
| F | Retained factual finding, limited to its stated evidence and inputs; not universal interaction certification. |
| P | Specific recommendation for whole-spec review; not an approved default. |
| O | Required field/law not yet closed; no runnable fallback follows. |

Evidence is separately graded: **static** (inspected source structure),
**bytecode-confirmed** (the particular checked operand/branch/order), **model/proof**
(independent mathematical consequence), or **public-description** (primary values
without complete execution evidence). No claim here has original-game execution
or original-runtime validation. Future acceptance cases in section 15 are all
**UNEXECUTED**. A requirement can combine A constraints, F fields and O dependencies.

### 1.1 Evidence index

These seven documents are present on the clean main baseline. Their publication
does not upgrade any bounded claim or override their own review qualifications.

| ID | Baseline input | What it contributes; what it does not |
|---|---|---|
| E01 | [Map/seed and shuffle residuals](../../research/2026-10-18-act1-rng-map-closure.md) | Supplied-choice zero-elite witness, map-stream dependency, exact conditional three-item counts; not initialized-seed realization or a general shuffle law. |
| E02 | [Execution closure](../../research/2026-10-18-act1-execution-closure.md) | Common-path order, five repeat flags, scheduler/rescue/theft/Reaper boundaries; not all callers or timer outcomes. |
| E03 | [Continuations closure](../../research/2026-10-18-act1-continuations-closure.md) | Finite parent/child matrix, deferred keys, campfire order, Note and acquisition boundaries; not universal return/drain laws. |
| E04 | [Relic reconciliation](../../research/2026-10-18-act1-relics-closure.md) | All 141 candidate identities/tiers, 32 direct spawn predicates and hook entrypoints; not all primary/global mechanics. |
| E05 | [Native contract](../../research/2026-10-18-act1-native-contract-closure.md) | Ownership, site-policy record, numeric/API/replay/fault proposals; not final schema or approved detailed API. |
| E06 | [Runtime capability limit](../../research/2026-10-18-act1-runtime-closure.md) | Why current canaries do not authorize game execution; no mechanical inference from that limitation. |
| E07 | [Acts 2/3 research](../../research/2026-10-18-ironclad-a0-acts23-research.md) | Carryover/setup and clock dependencies at the successor boundary; not authorization for later-act gameplay here. |

Additional read-only historical inputs are identified by filename rather than
broken hyperlinks to unpublished files. **E08** is the user-approved
`2026-10-18-ironclad-a0-act1-architecture-design.md`, including the packed-table
amendment. **E09** is the coverage and six primary-card-family drafts (Ironclad,
colorless/status/curse, Silent, Defect, Watcher, Note). **E10** is the accepted
map-Java/map-random, generation-Java and corrected acquisition follow-up findings.
**E11** is the accepted enemy-Java and execution-follow-up findings. **E12** is the
profile, packed-interface and generation-context supporting proposals. Their
needed conclusions are restated here; none is a required external normative file.
Unintegrated detail from them is a named blocker, not a hidden dependency readers
must recover. These summaries are not new independent checks of private source.

#### E09 review qualification — bounded update, 2026-10-19

**E09-R** identifies the fresh independent Note/Silent supporting-draft review
(review SHA256 `7c96c117462c5591d9e5c56de7a22237bfb12bd4836eb155578bde3a21909edc`).
Its dispositions attach to these exact historical documents, not subsequent PRs:

| E09 input | Exact reviewed SHA256 | Current evidence disposition |
|---|---|---|
| `2026-10-18-ironclad-a0-act1-note-card-rules.md` | `30f5599d5cf42c7293dca24c61508c0231c1f0afcc57b730b64a9c67cd95ed76` | Approved only as an accurate bounded primary-rule/supporting-obligation draft. Not full Note/content/profile closure or whole-spec approval. |
| `2026-10-18-ironclad-a0-act1-silent-card-rules.md` | `e4a141078639ca5df84c54e2a7cea6f23897fb7456c3b7c85460e45ac8ca9238` | **Changes required; not factually cleared.** Important findings SI-I1 (Setup) and SI-I2 (Masterful Stab) prohibit the unconditional cost/history claims. Section 5.2 states the required constraints. |

E09 is therefore not blanket factual approval of its member drafts. E09-R's
source-derived corrections are static/bytecode-backed, not gameplay tests. The
consolidation author read the complete review and the retained source/bytecode
anchors for these two findings; that is not an independent clearance of this new
revision. Corrected author documents still need exact-head independent recheck
and main's integration disposition (B04/B07/B14). Note's bounded clearance does
not approve the proposed detached output/default/death/fault contracts (B11).

#### Reviewed integration inputs

Main authorized the following exact revisions after independent clearance and
normal two-parent merges. Their bytes match canonical main
`56669aa42f6dfaae808e633840b721f14c40fc48`. Immutable public links intentionally
avoid requiring new files in this author's older-base worktree.

| ID | Cleared public input / exact author head | Integrated contribution and limit |
|---|---|---|
| E13 | [Initialized map/empty marker](https://github.com/klottick/spire-gym/blob/db15eb93eeb788b7683151b4bc5ebd10ac37c268/docs/research/2026-10-19-act1-map-seed.md) | Initialized underfill model, zero-Elite rank criterion and exact conditional counter/state failure branch; no initialized zero-Elite existence or native policy approval. |
| E14 | [Finite shuffle laws](https://github.com/klottick/spire-gym/blob/815a372a88447d38bbb5e9efcf0abb5530c279e0/docs/research/2026-10-19-act1-shuffle-laws.md) | Complete conditional finite mapper, weighted posterior and linked-pool composition; input measure, independence boundaries and catalog order remain choices, actual full ensemble unevaluated. |
| E15 | [Profile/time decision contract](https://github.com/klottick/spire-gym/blob/55f07f7382d9857c6fe3930cb5c6c200afd6ebe9/docs/research/2026-10-19-act1-profile-time.md) | Source history/Note/timer facts and concrete staged/committed, preset and 60-Hz/exact-elapsed/wait recommendations; all new native choices remain P. |
| E16 | [Source-specific continuations](https://github.com/klottick/spire-gym/blob/2cbcd2d325f0e4d4f2294612cbfff131919e2288/docs/research/2026-10-19-act1-continuations.md) | Closed local parent/child, desktop prevention, Bowl, Match, empty-selection and transition contracts; not global terminal arbitration, exceptional reachability or final handoff-schema approval. |

The independent whole-draft review at prior head
`fb7e9f8ff3d9665e3120f0b772623d5125146576` found zero findings **as an accurate
BLOCKED draft**, explicitly not a complete human-approval-ready mechanics/API
specification. It does not clear this revised head. Other execution/native/relic
revisions and seven normative-content author lanes remain pending main's exact
reviewed integration directions; neither their existence nor rosters close B07/B08.

All owned-source findings use archive SHA256
`cfad868ac8d65a88e71a0bf096fb09f78811e553effe0787c5309a655e081673`.
Those bytes do not establish an upstream release/build/channel or historical
host-JRE identity. Source, reconstructions, helpers, fixtures and raw logs remain
private; this document supplies independently worded facts/contracts only.

### 1.2 Decision index for human review

These are the material choices, not questions about unchanged architecture. Main
must first reconcile the factual follow-ups, then present the remaining choices.
Every recommendation below is **P**, including a recommendation to retain a blocker.

| ID | Decision and recommendation | Alternatives / cost | Required disposition |
|---|---|---|---|
| A1C-D01 | Target the pinned owned bytes and disclose static-only evidence. | Claiming a named retail build/runtime equivalence has no current evidence. Requiring that evidence delays promotion; documentation cannot manufacture it. | Accept the bounded target/evidence statement or require additional authorized evidence; A1C-B01. |
| A1C-D02 | Preserve the empty-elite failure boundary; do not repair maps. Keep initialized-seed realizability open until reviewed proof. | Forced elite insertion or regeneration changes the law. Conditioning away failing maps also changes it. A content/setup administrative error is preferable to fake defeat, but its exact approved applicability still needs resolution. | Resolve original-root image versus project semantic-choice support and initialization failure classification; A1C-B02. |
| A1C-D03 | Preserve §10.1's exact finite mapper and posterior. Recommend an initialized semantic block for five relic pools and linked draws; U64 only at explicitly isolated event/upgrade boundaries. Separately recommend ascending UTF-8 constructor-ID input order as a disclosed deviation from unresolved source-host hash-map order. | Independent U64 seeds discard source correlation; full nonzero-state blocks change initialized support; actual-root/history measure needs a declared prior and earlier calls; ideal uniform permutations change mapper bias/support. Sorted input is not neutral under that mapper. | Choose measure, each joint/independence boundary and ordered catalog separately; no choice is approved. A1C-B03. |
| A1C-D04 | Recommend §9.2's fixed normal-mode 60-Hz source-order scheduler, separate exact elapsed units, explicit decision dwell and finite WaitTo800Seconds candidate. No special Reaper success/loss rule. | Recorded frame deltas retain supplied-trace timing but cost records and prove no natural distribution. Semantic checkpoints require an explicit timing deviation. Recommended exact elapsed differs from binary32 playtime; canonical frames are a surrogate, not original FPS. | Approve or replace the concrete time/wait abstraction and close settlement/timer registry; Reaper native outcomes still require exact scheduling cases. A1C-B12. |
| A1C-D05 | Require complete detached history/Note fields in raw creation profiles, with no omitted-field defaults. Recommend a versioned ExperiencedA0 convenience preset resolving prior Neow eligibility=true, all nine ordinary bosses seen and Iron Wave/0 before initialization. | Explicit raw fields avoid hidden history. A novice-history preset (false/empty/Iron Wave0) is optional. Experienced is a proposed convenience default, not a consequence of fully unlocked or source first-use preferences; explicit partial/empty histories stay supported. | Approve preset/default and malformed-input rejection deviation; validate kind-specific stored-card domain. A1C-B11. |
| A1C-D06 | Recommend §9.1's separate staged snapshot and committed Note revision in a detached partial-run ProfileCandidate, with source-qualified departure commit and no automatic cross-reset inheritance. | Committed-only is a smaller alternative if progress remains explicit. Selection-only hides live-reference changes and risks false persistence. Recommended death/fault behavior preserves actual commits without manufacturing a completed account or successful fault output. | Approve lifecycle/output choice and integrate exact schema/diagnostics; technical review is not policy approval. A1C-B11. |
| A1C-D07 | Keep native packed transport primary; offer a candidate-aware Sequence facade with a stable int64 index carrier. | Small fixed Discrete loses legal choices; moving spaces complicate clients; machine-range Discrete is incompatible with dense action heads/masks and many generic learners. | Exact descriptor, carrier, wrapper contract and metadata; A1C-B09/A1C-B10. |
| A1C-D08 | Define success as a fully resolved successor-ready run, including applicable transition hooks/heal, but stop before any Act 2 policy action. | Merely killing the boss is too early. Generating the next act before versus after the boundary changes available state/RNG work and needs one explicit cutoff. | Exact successor construction/checkpoint and observation boundary; A1C-B05/A1C-B10. |

## 2. Scope and completion boundary

**A1C-REQ-001 (A):** Simulate standard, unmodded, fully unlocked, Act-4-enabled
Ironclad, non-Ascension A0, from real run creation through Neow, generated Act 1,
every legal choice/death/skip, and resolved `Act1Complete/Act2Ready`. Include all
legally reachable off-color and stored-card mechanics before that boundary.

The normal starting-state candidate is 80/80 HP, 99 gold, Burning Blood, five
Ironclad Strikes, four Ironclad Defends, Bash, three empty potion slots and no
keys (E09/E12; exact generated-start numeric certification is A1C-B08). These are
not arbitrary deck/resource configuration knobs. Support both Neow variants and
legal boss-discovery histories. Fully unlocked does not mean all bosses seen.

| Included route | Required closure | Not thereby included |
|---|---|---|
| Normal cards/rewards/shops/transforms | Ironclad ordinary cards, colorless sources, upgrades/removal/acquisition | A universal sampler for all routes |
| Prismatic Shard | Qualifying all-color rewards; one non-Defect orb slot; dependent mechanics | Other characters' starts, exclusive relics or potions; widened ordinary shop stock |
| A Note For Yourself | Legal cross-character basics/nonbasics, removable prior-run event cards, upgrades, incoming/outgoing hooks and profile transition | Later-act home events, arbitrary save import or temporary-only stored cards |
| Neow boss swap / boss chest | Eligible boss relics, all immediate mutations and nested selections/rewards | Omitting a relic because its ordinary benefit mostly applies later |
| Temporary generation/copies/statuses/curses | Dependent cards, powers, delayed effects, histories, targets and selections | Making temporary cards permanent or including every SPECIAL in random pools |
| Optional keys/skip/escape/death | Source-legal choices and actual outcome distinctions | Forced key collection, guaranteed victory, forced rewards or escape-as-victory |

Out of scope: other-character **starts/runs**, other Ascensions, Act 2 policy
play onward, custom/Daily/Endless/mods, locked-profile progression, rendering,
audio, real accounts/saves/Steam/cloud, learner implementation and training.
Their mechanics are not excluded where a legal in-scope route already needs them.

**A1C-REQ-002 (A/F):** Maintain the source-first independently authored boundary
and evidence pin above. Do not publish original/translated executable algorithms,
or execute original classes/initializers/runtime under this specification's
research authority. E06 is a capability limit, not a reason to re-survey sandboxes
or waive fidelity. The initial synthesis ran no gameplay acceptance cases.

## 3. Native authority and state

**A1C-REQ-003 (A):** `spire-core` owns rules, gameplay state, legality, ordering,
source policies and RNG. `spire-env` owns logical episodes, public projection,
candidate indexing, task reward, native batching and replay/fault provenance.
`spire-python` owns checked numeric transport and detached delivery. Python/policy
selects only among published candidates; no gameplay callbacks or repairs.

**A1C-REQ-004 (A/P):** One persistent run is the sole mutable authority. A temporary
combat overlay uses the same detailed engine as standalone combat with an explicit
resource context. It must not hold a separately writable HP/deck/gold/belt copy to
reconcile at victory. Proposed state partition:

| Run-owned | Combat-owned | Continuation/knowledge-owned |
|---|---|---|
| Validated creation profile; actual character; HP/max HP/gold; permanent deck instances; relics/counters; potion slots; keys | Enemies and selected moves; combat card instances/piles; energy/Block/powers/orbs/stance; turn and combat histories; active work | Parent/source/resume point; captured parameters and ordered selections; legal learned/revealed facts |
| Map/topology/progression; encounter/event/relic/reward histories and pools; run RNG/semantic identities | Combat RNG scopes; action/card/monster/pre-turn work and entered action; temporary cost/stat/scaling changes | Pending durable children, once-only boundaries, source-specific commit/cancel; not a duplicate resource authority |

Private permanent, combat, generated, effect, continuation and choice identities
are distinct. Copy construction allocates a distinct instance with explicit source,
capture time and copied fields; it does not accidentally alias permanent growth.
No learned feature contains an arbitrary native identity.

**A1C-REQ-005 (A/F):** Durable mutations apply live at their actual effect boundary,
including non-final kills. Feed/max HP, Hand of Greed/Wish gold, Genetic Algorithm
and Ritual Dagger growth, Lesson Learned upgrades, potion obtain/consumption and
relic counters do not wait for combat end. No global commit transaction exists.
Later death or administrative failure does not undo applied mutations. Queued work
is not proof of application. Never reconstruct persistent state from final piles,
reward totals or diagnostic logs.

## 4. Decisions, execution and interruption

**A1C-REQ-006 (A/P):** Use explicit typed phases: creation/Neow, map choice, room,
combat, source-qualified decision/selection, suspended child, death and completed
handoff. Run retains phase authority during combat. An accepted action resolves
automatic work to the next genuine decision, actual death or resolved handoff;
never auto-pick a strategic option. No effect/step/turn cap is a game rule.

**A1C-REQ-007 (A/F):** A suspended decision retains parent identity, resume point,
already generated offers, payment/captured X, ordered selected instances, remaining
eligibility/count, and committed/cancelled effects. Resume neither rerolls nor
repays. UI-only confirmations may collapse only after proving that commitment,
knowledge, legal choices, hook order and random timing are unchanged. There is no
universal optional Skip, forced empty-list completion or return-to-map shortcut.

### 4.1 Play origin and payment

**A1C-REQ-008 (F/O):** Preserve origin, legality, free/ignore flags, energy-on-use,
captured X, histories, copied state, target policy and destination independently.
This finite matrix specifies required distinctions, not a universal bypass:

| Origin family | Retained contract | Exact residual |
|---|---|---|
| Ordinary hand play | On the inspected successful common path, history/callbacks precede later dead/null-target handling; payload precedes cleanup/on-use callbacks, hand removal and ordinary payment. | Complete canUse/free/cost override order and alternate branches; A1C-B04 |
| Draw-top autoplay: Havoc, Mayhem, Distilled Chaos | Inner card uses its own effect-site policy; no inference from outer card/potion color. Havoc's exhaust destination differs from ordinary autoplay. | Per-source restrictions, target reassignment, unplayable cards, X and cleanup; A1C-B04 |
| Double Tap, Burst, Amplify, Duplication, Echo repeats | Copy original `energyOnUse`; explicitly set autoplay and ignore-total true. Repeated flags are not merely copied from the original. | Overlapping entitlements, copy fields, target death, cancel and once-only history; A1C-B04 |
| Omniscience and selected playback | Retain selected original identity/parent and its source-defined twice-play/exhaust handling. | Exact queue-extra callers and mixed-repeat law; A1C-B04 |
| Generated/copy/upgrade/draw/movement | Each has its own creation/history/cost/hook policy. Movement into hand is not automatically draw or obtain. | Full stat-equivalent-copy overrides and cost precedence; A1C-B04/A1C-B07 |

X is not fixed-cost zero or unplayable. Turn-zero setters do not erase negative X
cost semantics. Chemical X is source-sensitive; Transmutation's capture/payment
and Conjure Blade's child payload must not be inferred from Whirlwind alone.
Descriptions of generated-card discounts do not declare every generated X card
numeric-zero or free. The pending execution revision must supply its reviewed
per-source X/copy receiver matrix; none is silently imported in this pass.
Corruption, Confusion/Snecko Oil, until-played discounts, base/combat cost changes,
upgrades and free plays need an explicit precedence matrix. A missing matrix is
not permission to apply effects in the order convenient for an implementation.

### 4.2 Work and lethal boundaries

**A1C-REQ-009 (F/O):** Distinguish entered/current action, pending ordinary and
pre-turn actions, card queue, monster snapshot queue, room objects, dungeon effects,
souls and timer-driven completion. Do not replace them with one universal FIFO.
Last-enemy pending filtering and later victory/full clearing are different steps;
neither unwinds an entered call or certifies all other work drained. Monster
updates and manager/room/effect dispatch have distinct positions (E02/E11).

Actual Exhaust locally calls relics, powers, then the card callback before movement.
Dead Branch samples at its relic callback; Dark Embrace queues its later draw.
Fiend Fire snapshots the remaining-hand count and schedules that many random
one-card exhausts ahead of hits; those exhausts finish on the shrinking original
hand before queued Branch/Embrace children arrive. A repeated play can see the
later hand. End-turn cloned-hand shuffle/Ethereal cleanup does not recursively
sweep newly generated reaction cards. These are bounded static rules, not all
transitive lifecycle/callback closure.

**A1C-REQ-010 (A/F/O):** Keep Attack damage, non-Attack damage, direct HP loss,
max-HP changes, Block and healing distinct. Specify modifier order, floors/caps,
per-hit/area traversal, Artifact/debuff notifications, Intangible/Buffer/Torii/
Tungsten Rod, retaliation, resurrection and Fatal attribution. One inspected lethal
damage route checks held Fairy before unused Lizard Tail; rescue healing runs
synchronously through heal modifiers. A raw minimum-one heal does not prove
positive post-modifier HP. Other lethal origins and prevention callers remain open.
Parasite's object-removal max-HP decrease floors maximum at one and clamps HP; that
local helper does not itself invoke death. No later queued heal revives established
death. Reaper retaliation-before-queued-heal is evidenced; a lost-heal outcome or
timer threshold is not (A1C-B04/A1C-B12).

## 5. Card and mechanic closure

**A1C-REQ-011 (A/O):** Every catalog identity in appendix A needs paired base/upgrade
cost, type/target, legal origins, ordered payload, keyword/lifetime, copy/scaling,
history, RNG/site and interruption rules. A primary description is not that full
record. The appendices make the dependency closure finite and visible, not claim
367 fully certified mechanics. No production pool may omit a required identity,
reroll an unsupported result or turn it inert to make a run appear complete.

**A1C-REQ-012 (A/F/O):** All three 71-card off-color nonbasic families are required,
not deferred to other-character milestones. Include Poison/discard/Retain; orb
slots/Focus/Evoke; stances/Mantra/Scry; X and delayed effects. Note alone can deliver
Zap/Dualcast without Shard: zero slots, empty slots and full occupied slots differ.
Only Shard's route grants its specified slot, not Defect starting resources.

**A1C-REQ-013 (F/O):** Special, status, curse and Note identities use their actual
rules, not ordinary pool metadata. Generated Shiv/Miracle/etc are not random-pool
members merely because reachable. Note adds twelve distinct basics and Apparition,
Bite, J.A.X., Ritual Dagger without adding their later-act events. Curse of the Bell
is reachable via Calling Bell but is not an ordinary random curse or removable
stored-card candidate. A0 does not start with Ascender's Bane; Pride's Endless route
and Necronomicurse's non-removable route do not justify arbitrary Note injection.

### 5.1 Shared mechanic records still required

The following is a finite closure checklist under A1C-B07; it must receive exact
laws, not just examples or a generic “all interactions” assertion.

| Mechanic record | Required fields / discriminator |
|---|---|
| CARD-COST | Printed/base/turn/until-played/free/X/unplayable layers, signed sentinels, upgrade and copy precedence; Setup's positive-`cost` free-flag guard and Masterful Stab's per-instance notifications (§5.2); Blood for Blood counts HP-loss events, Force Field qualifying Power history, Eviscerate actual discard history. |
| CARD-COPY | New identity versus stat-equivalent copy, source capture, all per-instance growth/cost/flags, permanent attribution; Anger/Dual Wield/Nightmare/Omniscience are not one route. |
| CARD-DRAW | Successful draw versus retrieval/insertion; No Draw; full hand; reshuffle/Innate/bottles; Endless Agony/Deus/Void and Evolve/Fire Breathing; setup before opening draw. |
| CARD-DISCARD | Explicit, random, effect and cleanup causes; Reflex/Tactician and counters; snapshots for Gamble/Storm of Steel/Unload/Scrape; no recursive inclusion of reaction arrivals. |
| CARD-EXHAUST | Selected/random/Ethereal/on-use/purge/Power differences; Spoon applies only at its played non-Power exhaust-cleanup checkpoint; Branch/Embrace/Charon's Ashes/Sentinel order. |
| CARD-RETAIN | Inherent Retain, selected Retain, Equilibrium, Pyramid and Ethereal differences; Establishment, Sands of Time, Perseverance and Windmill mutation/expiry. |
| CARD-BLOCK | Card versus power/relic/potion/orb/direct doubling/Wallop sources; Dexterity/Frail/No Block; separate repeated gains for Juggernaut and Wave of the Hand. |
| CARD-DEBUFF | Artifact per separate application, duration/cleansing, temporary Strength restoration versus permanent loss, Sadistic Nature/Champion Belt reactions; Poison/Catalyst/Explosion independence. |
| CARD-ORB | Capacity/zero/full/removal; channel success/history; Focus floors; Lightning/Frost/Dark/Plasma payloads; Dark capture/ties; Lock-On/rounding; Chaos law; Loop/Recursion/Fission/Multi-Cast order. |
| CARD-STANCE | Neutral/Calm/Wrath/Divinity entry/exit/same-state, energy, attack-only multipliers, Mantra threshold/excess/history, notifications and next-turn exit; Eruption hits before entering Wrath. |
| CARD-SCRY | Revealed prefix, legal discard subsets/order, empty/short piles, Weave/Nirvana reactions, retained knowledge and ensuing draws; no hidden remainder ordering. |
| CARD-DELAY | Next-turn energy/draw/copies, turn countdowns, extra turn versus enemy turn, Blasphemy lethal mechanism, Deva/Wraith/Biased downsides, Mayhem timing and reset. |
| CARD-FATAL | Actual attributed kill, Minion/split/suicide, simultaneous death and permanent Feed/Greed/Dagger/Lesson mutations; no reward on split alone. |
| CARD-HISTORY | Play versus hit counts, previous-card predicates across turns, self/repeat inclusion, HP-loss/discard/Power/Mantra/channel histories, public versus hidden portions; Masterful Stab's notified locations and retained instance state, not universal global-count catch-up (§5.2). |

Primary numeric anchors retained from E09 include Strike 6/9, Defend 5/8, Bash
8/10 plus Vulnerable 2/3; Searing Blow at upgrade count n has base
`12 + n(n+7)/2`; Heavy Blade multiplies signed Strength by 3/5; Genetic Algorithm
begins at one Block and grows permanently by 2/3 per qualifying use; Ritual Dagger
begins at 15 damage with Fatal growth 3/5. These anchors are not a substitute for
the full paired-content records; their unintegrated primary detail is A1C-B07.

### 5.2 Required Note/Silent review constraints (F, bounded; E09-R)

These independently worded corrections override any incompatible implication from
E09; they do not close the shared cost/copy/callback matrices or certify all Silent
mechanics. The locators name inspected members, not a required private artifact.

| Finding | Known constraint | Evidence locator / remaining boundary |
|---|---|---|
| SI-I1 — Setup | Move the selected existing hand instance to draw-pile top. Both singleton and selected-card routes grant `freeToPlayOnce` only if its current base/combat `cost` field is positive, not by testing `costForTurn`. This is a one-play-free flag, not a numeric-zero rewrite. Positive `cost` with changed turn cost qualifies; base zero with positive turn-only cost, X and unplayable sentinels receive no new flag. Selection does not make an unplayable card playable or redefine an X card's payment/effect. Ordinary attribute reset leaves this flag intact; use cleanup clears it. | `SetupAction.update` (both branches); `AbstractCard.resetAttributes/freeToPlay`; `UseCardAction.update`. Full override/copy/free/cleanup precedence remains B04/B07. |
| SI-I2 — Masterful Stab | Fresh ordinary construction starts cost zero. Each qualifying damage notification increments that instance's cost by one through its cost-update helper, not by the number of HP points. The inspected positive-loss combat dispatch notifies hand, discard and draw, **not exhaust**. Exhume returns the existing exhausted instance without missed-notification catch-up. Preserve location and per-instance cost state; a global combat HP-loss count is not a replacement. Ordinary fresh copy and stat-equivalent copy are distinct routes, not universal historical reconstruction. | `MasterfulStab` constructor/`tookDamage/makeCopy`; `AbstractPlayer.damage/updateCardsOnDamage`; `ExhumeAction.update`; `AbstractCard.updateCost`. Other prevention, cost-layer and copy composition remains B04/B07. |

Future discriminators under T09–T11 must retain the selected/card-instance identity,
field/flag state and exact notification boundary. In particular, a legally acquired
Masterful Stab exhausted by True Grit+, followed by positive HP loss while exhausted
and retrieval by Exhume, must not receive that missed increment. This is a
source-derived counterexample to a global formula, not an executed trajectory or
initialized-root-seed witness. Note's NT-01–15 and Silent's SI-01–22 remain
unexecuted future cases; neither inventory nor E09-R's bounded clearance runs them.

## 6. Shared generation and source-specific randomness

**A1C-REQ-014 (A/P):** One native shared mechanism resolves producing-site policy
against **actual run character**. Printed color, Note origin, outer autoplay,
copied source or deck composition never supplies current character. Source-specific
policies distinguish temporary generation, fixed creation, copy, movement,
permanent reward, merchant proposal, transform, potion proposal and acquisition.

**A1C-REQ-015 (F/P/O):** A policy record is required at each effect site, including
indirect callers. Required fields: stable site ID/producer/route/result kind;
context basis; exhaustive eligible IDs/exclusions; selection stages and finite
masses; duplicate/retry and history reads/writes; RNG family/scope/draw boundary;
construction/upgrade/cost/copy; destination/capacity; commit/blocked behavior;
continuation; evidence and readiness. Each field is established, partial with
named open subfields, or open. Unknown is not an empty dependency list.

Recommended catalog preflight rejects missing/open required records before run
initialization. A supposedly closed missing lookup during execution is an
administrative content fault with site provenance, not another character's pool,
an arbitrary card, dropped effect, automatic skip or defeat. No current record is
declared implementation-ready solely by this draft.

### 6.1 Twenty temporary-generation/potion source laws

Retained E10 static findings close eligible supports, conditional masses and
duplicate rules in this profile; they supersede older tooltip-only pool unknowns.
The remaining O fields are transitive cost/callback/cancel/terminal integration,
not a claim that all twenty candidate pools are still unknown.

Definitions: A28 = ordinary Ironclad Attacks except Feed/Reaper; S28 = all ordinary
Ironclad Skills; P14 = all ordinary Ironclad Powers; I70 = their union; C20 =
Ironclad commons; L34 = ordinary colorless except Bandage Up. “Uniform” below
describes the finite semantic bounded choice, not an initialized original-seed
ensemble. Ordinary combat selectors filter HEALING before selection; that excludes
Alchemize/Self Repair/Lesson Learned/Wish in their respective actual-character
contexts, not Hand of Greed. Basics, statuses, curses and SPECIALs including
Expunger are outside these ordinary supports.

| Site family | Selection / choice | Construction and local timing |
|---|---|---|
| Infernal Blade | One uniform A28 | Sample in use; turn-zero discount; then queued hand placement; source+ changes payment. |
| Distraction | One uniform S28 | Same route; X remains X; source+ changes payment. |
| White Noise | One uniform P14 | Turn-zero, not permanently free; source+ changes payment. |
| Creative AI | One P14 per intensity at start hook | Ordinary child cost; each power loops its full intensity in retained power-list order before normal draw is queued. |
| Hello World | One C20 per intensity | Ordinary child cost; suppress if monsters basically dead; source+ is Innate. |
| Discovery | Ordered three distinct I70 IDs; mandatory one | Uniform without replacement by kind; chosen turn-zero; source+ removes source Exhaust. |
| Chrysalis | 3/5 uniform S28 proposals, repeats allowed | All identities sampled before queued one-card insertions; positive base and turn costs zero for combat. |
| Metamorphosis | 3/5 uniform A28 proposals, repeats allowed | Same selection-before-insertion pattern, not whole-pile shuffle. |
| Foreign Influence | Three distinct all-color ordinary non-healing Attack IDs; zero/one choice | 43/48/15 IDs by common/uncommon/rare, initial tier 55/30/15 of 100; duplicate re-proposes **rarity and ID**. Source+ discounts chosen card. |
| Jack of All Trades | 1/2 uniform L34, repeats allowed | All proposals in use before additions; ordinary child costs. |
| Magnetism | One uniform L34 per intensity | Before normal draw; ordinary child costs; basically-dead suppression. |
| Transmutation | Effective captured X uniform L34 proposals | Sample before additions; source+ explicitly upgrades children before turn discount; Chemical X adds two to effect count. |
| Dead Branch | One uniform I70 on actual Exhaust | Sample during relic callback before queued placement; basically-dead suppression; exhausted color irrelevant. |
| Toolbox | Three distinct L34; mandatory one | Pre-draw choice; ordinary chosen cost; prebattle shuffled/Innate/bottle setup precedes it, but other hooks can change the hand first. |
| Attack Potion | Three distinct A28; optional choice | One selected identity, two separate copies with Bark; turn-zero. |
| Skill Potion | Three distinct S28; optional choice | Alchemize remains excluded even in genuine Silent context; not a reward exclusion. |
| Power Potion | Three distinct P14; optional choice | Source-character Power pool; not printed-card color. |
| Colorless Potion | Three distinct L34; **mandatory** choice | Its constructor route differs from typed potions; Bark copies selected ID, not offers. |
| Alchemize | Rarity-first Ironclad potion proposal, Fruit Juice excluded | Sample in use before obtain action checks Sozu/full belt; live persistent belt on resolution. |
| Entropic Brew | One proposal per live **slot capacity**, not vacancy count | Consume Brew, sample all proposals, then obtains; combat excludes Fruit Juice and samples even under Sozu; outside combat checks Sozu before proposals and permits Fruit Juice. |

For ordinary distinct offers, later accepted IDs are uniform among remaining
support. Foreign Influence instead reconditions each remaining ID's original
proposal mass p by `p/(1-s)`, where s is accepted-ID mass; later rarity marginals
are not fixed 55/30/15. Separate invocations do not ban earlier outputs. These
paths do not mutate card-reward pity. Do not independently repartition rarity to
erase the within-offer dependencies.

### 6.2 Destination and construction laws

Hand placement selects/constructs before checking capacity ten; excess goes to
discard, not deletion. Sequential children see then-live occupancy. Nonempty draw
insertion chooses uniformly among positions below the current top: n positions
for n current cards, **not n+1 gaps**; empty insertion is deterministic. Identities
for Chrysalis/Metamorphosis are selected before any insertion. Proposed public
draw orientation is top-first, independent of private storage; this is a descriptor
choice under A1C-D07, not a claim about source container orientation.

Master Reality checks occur at distinct construction sites. Hand-action routes
and Discovery/potion/Toolbox routes can upgrade at an earlier constructor/choice
checkpoint and again at actual hand insertion; ordinary upgrades saturate but
Searing Blow can gain two. Foreign Influence has only the hand-effect checkpoint.
Overflow's coordinate discard effect upgrades an animation copy, not its inserted
input: earlier upgrade remains, but that extra upgrade does not. Thus Bark at hand
size nine can put Searing Blow +2 in hand and +1 in discard; Foreign Influence
overflow can have no actual Master Reality upgrade. Do not normalize these quirks.
Conjure Blade creates Expunger even at zero captured payload; source+ adds one,
Chemical X adds two, child upgrades change 9 to 15 damage, not stored hit count.

Blood for Blood construction applies previous combat HP-loss-event history.
Force Field can consult qualifying prior Power plays. Eviscerate's copy path can
mutate its receiver's turn cost; full-hand Foreign Influence+ overflow need not
retain the earlier zero discount. These are exact route caveats, not a universal
“new cards start at printed cost” rule. Full shared cost/copy closure is A1C-B04.

## 7. Map, encounters and enemies

**A1C-REQ-016 (A/F/O):** Preserve the actual map's construction/conditional law,
not uniform valid maps or matching aggregate room counts. Retained E10 map facts:

| Stage | Required behavior |
|---|---|
| Topology | 7 columns × 15 allocated rows; six ordered complete walks; only the second start must differ from the first. Interior initial neighbor mass 1/3 each; boundary has two legal neighbors at 1/2 each, not clamped thirds. |
| Repairs | Retain repeated parent occurrences despite unique outgoing edges; ordered short-ancestor repair scans the initial target's parent list even after retargeting. Preserve the evidenced x-versus-y ancestor quirk and finite extremal-parent search, not an idealized graph ancestor rule. Crossing snaps left bound then right bound after that scan; no fresh valid-edge sample. |
| Bottom trim/activity | Scan lowest x first and retain first edge to a destination; remove outgoing duplicates only, not parent occurrences or allocated nodes. Activity for counts/assignment is outgoing edges. Final-row boss connector is not an ordinary allocated/quota node. |
| Quotas | N counts active nodes except row 13, including active fixed rows. Binary32 N×.05/.12/.08/.22 then Math.round gives shop/rest/elite/event counts. No A1 multiplier. Ordinary untyped count U includes row 13 but excludes fixed rows. |
| Bucket | Append quota entries then Monster padding to U; one retained shuffled bucket, not independent cell draws. Fixed rows 0 Monster, 8 Treasure, 14 Rest are pretyped; even inactive fixed cells can be typed without being quota-active. |
| Assignment | Row-major first compatible bucket occurrence; remove only that occurrence and retain skipped order. Parent restrictions: Rest/Treasure/Shop/Elite; sibling restrictions: Rest/Monster/Event/Elite/Shop. Optional Elite/Rest forbidden y≤4, Rest also y≥13. Fixed rows bypass ordinary coloring restrictions. |
| Fallback | Entire ordinary pass first; only afterward fill all active null nodes with Monster, consuming no bucket item. Earlier failures are not visible as Monsters during assignment. Quotas are targets, not guaranteed final totals. |

The retained repair matrix is relative to current column x: a candidate right of
x redraws x−1/x equally; left redraws x/x+1 equally; equal redraws x−1/x/x+1 in
thirds. Off-grid repair proposals reflect inward: at x=0 the equal branch yields
0 with mass1/3, 1 with mass2/3, while the right branch collapses to0; symmetric
rules apply at x=6. This differs from the initial boundary two-way law. Repeat
the conditional operation only for qualifying occurrences in the finite original
parent scan, not until a desired graph invariant holds. The ancestor helper's
initial comparison uses its first x versus second y; its extremal chains keep
first equal-x occurrence and test node identity. Parameter five permits up to six
ascents; repair requires successor-row minus ancestor-row<3, with no effective
large-gap correction. These are compatibility facts, not instructions to fix a
suspected typo. Historical library applicability and final native semantic registry
remain A1C-B02/A1C-B03/A1C-B10. A zero-elite supplied-choice witness is not an
initialized-root-seed witness or a successful playable map.

**A1C-REQ-017 (F/O):** With Act 4 available and emerald absent at marker placement,
select uniformly among all assigned elite cells, not first row then node. No
regeneration was found on the traced fresh path. At zero candidates the inspected
bound is invalid and throws; native treatment and initialized-seed image remain
A1C-D02, not a forced elite, retry, removed marker or defeat.

The marker does not preselect a buff. Combat preparation draws one of four equally
weighted families for the group: Strength +2, rounded max-HP increment 25% with
matching heal, Metallicize 4, Regeneration 3. All current enemies receive the same
family in list order. Max-HP preparation precedes Preserved Insect's current-HP
reduction on the inspected route; marker-placement emerald testing is not a new
later buff-time emerald test. The future buff is not pre-revealed by the map.

### 7.1 Initialized-map evidence and conditional failure (E13)

**F/model, not runtime:** numeric run seed 5988 gives map seed 5989 on the inspected
Act-1 `run seed + 1` initialization. The source-corresponding independent models
give N=41, ordinary capacity U=37 and initial bucket Shop2/Rest5/Elite3/Event9/
Monster18. Only `(3,5)` and `(6,12)` receive Elites; `(5,13)` and `(6,13)` fail
ordinary assignment, leaving one Rest and one Elite. Deferred fill makes both holes
Monsters. The marker selects `(6,12)`. This refutes quota-equals-assigned even on
initialized sampling; it is not a played original run, zero-Elite witness or proof
of a universal two-Elite minimum. Search of map seeds 1–200,000 found no zero case;
that finite miss is not exclusion or a probability law.

**F/proof:** for a positive Elite quota, label bucket tokens by their actual
shuffled positions, and let e be the first Elite token's position. Run the ordinary
scan with all Elite tokens suppressed, retaining order, other rules and delayed
fill. At each ordinary Elite-eligible node v let r(v) be its chosen token's original
position, or infinity for failure. Zero Elites occurs **iff every r(v)<e**. Before
the first Elite the two scans agree, and no earlier Elite parent/sibling can block
one; the first violation would assign an Elite. With bucket size U and quota q>0,
zero requires at least q holes in rows1–4 and none later, but early holes alone are
not sufficient. Initialized closure still needs a root witness or exclusion proof
where topology G(s) and bucket B(s) come from the **same** initialized seed/cursor,
not separately legal choices. No native map measure is selected by this proof.

The wrapper's call counter is not its underlying XS state cursor. Fresh map RNG
replacement occurs immediately before generation; earlier scene/relic activity
does not consume it. Initialization uses a 64-bit seed image, with zero aliasing
signed minimum, not arbitrary uniform128 state. Six walks use 90 ordinary calls
plus conditional retry/repair calls. Bucket shuffle uses XS directly and does not
increment the wrapper counter; ordinary assignment/fill uses neither. In the
5989 model, topology ends at counter/raw98/98, bucket at98/134, marker at99/135.
These are model counts, not original runtime measurements or new RNG-001 parity
requirements. Visual jitter uses a different RNG object.

| Marker condition | Source-local F contract |
|---|---|
| Act4 unavailable, or emerald already held | No marker selection call, regardless of candidate count. |
| Enabled, positive candidates | Scan Elite-room instances in grid row/column order; uniform bounded index, including an actual draw for a singleton. Nonactive cells do not acquire Elite rooms on this ordinary path. |
| Enabled, no candidates | Increment wrapper counter once, then invalid bound throws before **any** raw XS step, list access or marker change. Map assignment, first-room reset and fade request have already occurred. No local repair/regeneration/rollback/death conversion was found through the immediate factory. |

This establishes the empty branch conditionally, not initialized-root existence or
an observed crash. **P/D02:** represent it as an administrative initialization fault,
never −1, forced Elite, reroll or successful markerless map. Applicability and the
native map law still need explicit disposition. Outer launcher presentation is not
certified by this local trace.

**A1C-REQ-018 (F/O):** Generate real encounter queues, ordered compositions, HP,
initial move/history and opening shuffle/draw/hooks. Act 1 initial normal queue
has three weak plus one first-strong plus twelve further strong entries (**16**),
and ten elite entries; these are initialization lengths, not gameplay caps.
Ordinary append rejects either last-two family; elite append rejects the previous
elite. First-strong is separate: last weak two Louses excludes three Louses;
Small Slimes excludes Large/Lots. Do not add last-two filtering there by analogy.
Leaving normal/elite rooms consumes its queue head; event fights do not.

| Family | Required encounters / formation distinctions |
|---|---|
| Weak | Cultist, Jaw Worm, two Louses, Small Slimes; equal weight two. Small slime is left/small then right/medium with opposite types. |
| Strong | Blue Slaver2, Red Slaver1, Looter2, Large Slime2, three Louses2, two Fungi Beasts2, Gremlin Gang1, Lots of Slimes1, Exordium Thugs1.5, Exordium Wildlife1.5; total16. These source weights are not uniform ten-way. |
| Elite | Nob, Lagavulin, three Sentries; finite 24-bit initial masses 5592406/5592405/5592405 out of 16777216 before immediate-repeat rejection. |
| Boss | Guardian, Hexaghost, Slime Boss; discovery-conditioned priority or seeded shuffle, not assumed uniform thirds. |
| Event | Dead Adventurer's three elite variants and Mushrooms' Fungi fight, with event-owned setup/rewards/history. |

Gang samples four ordered instances without replacement from Mad×2/Sneaky×2/Fat×2/
Shield×1/Wizard×1, with no final shuffle. Lots exhausts a shrinking Spike(S)×3/
Acid(S)×2 pool. Louse colors use separate fair choices. Thugs' left weak-wildlife
slot chooses Louse/Spike(M)/Acid(M) in thirds, then fair Louse color; right chooses
Cultist/Slaver/Looter in thirds, then fair Slaver subtype. Wildlife's left is
Fungi/Jaw Worm equally, right uses that weak-wildlife law. Large slime type is fair.
Category-then-subtype is not equal mass per monster kind. Weak quarters and these
strong weights are exact fractions of the inspected finite float grid; elite
thirds have the unequal finite counts above.

**A1C-REQ-019 (F/O):** Enemy move history updates at selection, not only execution.
Retain finite redirect laws instead of renormalizing allowed moves. In particular
Red Slaver's post-Entangle unrestricted Stab/Scrape is 45/55, not 55/45; Jaw Worm
and Acid redirects use their finite float-grid comparisons. Lagavulin event starts
awake with no intrinsic Block/Metallicize; ordinary third sleep selects next-turn
Attack, not an immediate attack. Siphon applies Dexterity−1 then Strength−1 as
separate Artifact-eligible debuffs. Wake removes eight Metallicize, not all stacks.

Slime split intent is not immediate replacement; living half-HP threshold schedules
split, lethal damage suppresses it. Children get parent's split-time current HP
as both current/max, fresh powers, ordered spawn and non-Minion status. Monster
queue snapshots do not automatically append new children. Guardian's mode/Block
transition is queued; Sharp Hide timing is not a precomputed damage surcharge.
Hexaghost captures Divider from starting player HP and Inferno has separate Burn
mutation/insertion checkpoints. Looter ledger increment, actual gold theft and HP
hit are distinct; ordinary inspected path has no player decision between ledger
and theft, and refund uses the ledger, not recomputed net loss. Escapes are not
victory. All spawn/Fatal/rescue/timer/terminal cross-effects remain A1C-B04/A1C-B07.

Appendix B lists all 25 kinds and HP anchors. Full A0 payload/AI tables and
setup/reaction integration must still be consolidated and checked
under A1C-B07/A1C-B08; no absent value in this draft licenses a guessed law.

## 8. Neow, rooms, events and acquisition

**A1C-REQ-020 (A/F/O):** Support prior-run Neow-eligible four-option Neow and the
shortlist variant. Shortlist offers +8 max HP or Neow's Lament for the normal
Ironclad baseline. Full menu retains six first-category and five second-category
possibilities, conditional third-option drawback/reward exclusions, and boss swap.
Category selection differs from category-local candidate selection. Drawback
activation precedes reward activation. Boss swap removes the starter before spawn
predicates, so Black Blood cannot replace Burning Blood there, although it can be
eligible at a later boss reward. Every child card/relic/transform/potion selection
must resolve and resume Neow once. E16 closes the local return/transform distinctions
in §8.2; global arbitration remains B05 and exact option weights/exclusion/quantities
remain B08, not random generic blessings. Eligibility history is detailed in §9.

**A1C-REQ-021 (A/F/O):** All 25 Act 1 event families in appendix C are required,
including actual choices that cannot initially Leave, repeated risks, legal deaths,
resource refusal and nested fight/reward returns. Shrine branch is 1/4 before
filtered selection, not uniform over 25. Eligibility uses floor/gold/curse/profile/
history; selected IDs leave their applicable pools. Preserve empty-pool behavior
when established, not a universal reroll. Event-owned fights remain EventRoom for
reward rarity, Prayer Wheel, elite drops/keys and queue accounting. Their Proceed
has source-specific direct-map routing; ordinary combat return is not universal.

Question rooms have evolving monster/shop/treasure probabilities, Juzu/Tiny Chest
and preceding-room restrictions. They are not one fixed event/random-room table.
The complete probability update/filter/empty-pool records are A1C-B08.

**A1C-REQ-022 (F/O):** Shops support repeated purchases, removal and nested obtains
until legal Leave. Initial colored slots are two Attacks/two Skills/one Power plus
the source's colorless stock; Shard does not widen them. Duplicate second Attack/
Skill re-proposes rarity and ID, unlike reward fixed-rarity identity retry.
Common Power proposals fall through to uncommon. Relic/potion slots, sale,
discount timing, Smiling Mask precedence, removal-price history and Courier
restocks retain their own binary32 expression/truncation/rounding order. Prices
are not generic chest jitter. A proposal is not a purchase or obtain. Full belt/
Sozu refusal, payment/onEquip and each child resume have exact source contracts.
Courier/Maw Bank/Smiling Mask/Old Coin shop-room exclusions do not make them
globally unreachable. Full slot/price/restock/empty-stock laws remain A1C-B08.

**A1C-REQ-023 (F/O):** Offers, claims and mutations differ. Ordinary permanent-card
support is 72 Ironclad nonbasics without combat HEALING exclusion. Qualifying Shard
rewards use rarity-only union 320, with common/uncommon/rare counts 76/160/84,
not equal color then card. Within selected rarity, reject previously offered IDs
without rerolling rarity. Ownership does not deplete card reward support.

Reward rarity offset starts five; rare resets five, common decrements to floor
−40, uncommon leaves unchanged. It mutates per **generated slot** before fixed-tier
identity retries. Initial ordinary rare/uncommon/common is 0/35/65%, elite 5/40/55%;
these are not universal source probabilities. Automatically generated then removed
rewards can already have changed pity. Dream Catcher/Prayer Wheel/Orrery/Tiny House/
Neow retain source count, room and history distinctions. Skip does not reverse
offer history. Singing Bowl is the separate +2 maxHP then heal2 action in §8.4,
not all choose-one screens. E16 closes its downstream helper/row behavior; natural
reachability of abnormal records and global completion arbitration remain B05.

Colored transforms use flat actual-character ordinary 72, ordinary colorless 35,
excluding input ID when in support; Shard and input off-color identity do not widen
them. Curse transforms and object/string/iterator removal callbacks differ.
Egg previews, actual obtains, Pandora/Astrolabe transforms, bottle selection and
direct Note insertion cannot be collapsed into a universal acquire/remove call.

### 8.1 Relics, potions and camps

**A1C-REQ-024 (F/O):** E04's finite 141 candidates are required: 1 starter, 33 common,
30 uncommon, 28 rare, 22 boss, 17 shop, 10 special (appendix D). Ordinary pools are
shared plus Ironclad-red, not every character's exclusive relics. Pool initialization
shuffle, owned removal, offer consumption even on skip, spawn predicates and source
selector restrictions all matter. Constructor/hook enumeration is not primary-rule
closure. All 141 require trigger, payload, counter/reset, shared consumer and
ordering records. Prioritize the thirteen global-site residuals explicitly listed
in appendix D; absence of subclass override does not mean inertness.

Relic draw correction: normal consumes front for nonempty tiers. End consumes back
for common/uncommon/rare/shop but **front for boss**. Rejected candidates are consumed;
normal rejection dispatches to end at the **requested tier**, and end rejection
retries through itself. Depleted common/uncommon/shop promotes to uncommon/rare/
uncommon using **normal/front**, even from an end call. Empty rare gives Circlet;
empty boss Red Circlet. These fallback identities are outside the 141 candidate
roster. No inexhaustible replacement stream is implied.

**A1C-REQ-025 (F/O):** Ironclad potion support is exactly the 33 candidates in appendix
D: tier sizes 17/9/7. Combat drop/shop/Courier selection is rarity-first 65/25/10;
Lab/Woman/Neow/Cauldron are flat-ID source families; Tiny House retains its supplied
RNG policy. Potion appearance pity is separate from identity/claim. For Alchemize
and combat Brew, Fruit Juice is excluded with retained-rarity rejection: each
common has mass 65/(100×17), uncommon 25/(100×9), rare 10/(100×6). Outside-combat
Brew has seven eligible rares. Brew can produce itself; healing potions are not
blanket banned. Obtain fills first live empty slot; no victory deferral or forced
replacement. Sozu prevents new obtains, not retention/use of held potions.

Fairy is not manually drinkable. Smoke Bomb has source/encounter restrictions and
escape is not victory. Bark needs per-potion rules, not universal multiplication.
Use/discard phases, Elixir/Gambler/Liquid Memories selections and Distilled Chaos
play origins remain explicit B04/B05/B08 obligations. E16 closes local full-belt/
Sozu reward-versus-Shop distinctions in §8.4, not all33 potency/Bark/phase records.

**A1C-REQ-026 (F/O):** Campfire construction order is **initial options → held-relic
filter → eligible Recall append → final usability scan**. Filter Rest/Smith and
relic-added options, stopping at first rejection per option. Coffee Dripper rejects
Rest, Fusion Hammer rejects Smith, not the whole final menu. Recall remains
possible when eligible. Rest heals before onRest; Dream Catcher opens a child.
Smith selects currently upgradeable instances; Lift/Toke/Dig have actual Girya/
Peace Pipe/Shovel state and restrictions. E16 establishes that a final scan with
none usable completes the room; Smith/Toke cancellation returns to campfire menu.
Never invent an option or force key collection. Other per-relic/numeric/global
arbitration obligations remain B05/B06/B08.

**A1C-REQ-027 (A/F/O):** Keys are optional. Emerald is not relic-linked. Sapphire
competes with its associated base chest relic, not every extra Matryoshka relic.
Ruby Recall/other key claims schedule effects; flags commit during later key-effect
updates, not offer/claim construction or room-local COMPLETE. Chest hooks and
reward records are ordered; Cursed Key/Hungry Face/Matryoshka remain distinct.
Chest type uses 50/33/17 source weights. Tier and gold presence share their source
roll; gold amount preserves separately rounded binary32 .9G/1.1G endpoints before
finite interpolation and rounding, not G times independently sampled jitter.
Boss chest uses a distinct path, not ordinary sapphire/Matryoshka rules.

### 8.2 Parent/child obligations

| Parent | Child / required return and commitment distinction | Open dependency |
|---|---|---|
| Neow | Drawback before reward; grids consumed once by activated reward update; event resumes with no screen. Single transform generates before removal; double removes both before generating. Potion reward Proceed uses Neow's map branch; discarded automatic card offer already changed history. | A1C-B05/A1C-B08 |
| Merchant | Confirmed purge result pays/advances price/removes/disables service; cancel does none. Card purchase constructs obtain before spend/restock; relic onEquip may open a child before restock finishes. Children cannot repay/reroll. Orrery/Cauldron labelled screens do not guarantee a Shop return marker: close may reach NONE; merchant reopens the **same** inventory. | A1C-B05/A1C-B06 |
| Event | Event update consumes results only with no screen up. Dead Adventurer/Mushrooms combat and Lab reward Proceed open map directly; Woman in Blue uses unlabelled open()/Proceed returning to event/result, not labelled Cancel or Lab's exception. | A1C-B05/A1C-B08 |
| Chest/reward | Card row remains on Skip and can reopen without resampling; Take consumes it before Fast obtain resolves. Whole-list Proceed can leave rewards. Relic claim uses instant onEquip, refuses while Grid, and marks linked sapphire ignored/done where applicable; actual key flags commit later. | A1C-B05/A1C-B06 |
| Campfire | Heal precedes onRest/Dream child; empty generated offer opens no screen. Smith/Toke cancellation returns to menu. No usable final option completes room without a fabricated choice; appended Recall retains its own eligibility and deferred key. | A1C-B05/A1C-B06 |
| Boss relic | Offer closes before ordinary arrival/onEquip and relic-owned children. Astrolabe/Cage/Pandora/Bell/Tiny House branches below precede resolved handoff; chest/no-pick and child rewards can be skipped where source permits. | A1C-B05/A1C-B06/A1C-B10 |
| Any active parent | Actual death interrupts according to source dispatch; already committed effects persist; administrative fault is not death | A1C-B04/A1C-B05 |

The following E16 contracts supersede the older open local-dispatch obligations;
global terminal arbitration and the named exceptional reachability remain open.
Wheel gold commits at reveal, not confirmation; its other outcomes have later
callbacks. Neither room COMPLETE nor screen closure proves universal return.

### 8.3 Entered effects, curse prevention and removal (F; E16)

On the normal desktop caller, raw/ordinary delta share the backend field and
`CardCrawlGame.render` returns **before game update when raw delta>0.1f**. Rejected
frames do not accumulate a later oversized subtraction. Admitted delta is at most
0.1f. Dungeon active top-level effects update before done-removal, then ordinary
effects do likewise; pending queues transfer afterward without update. Room relic
objects update before removal; installed relic updates occur later in the overlay,
after Proceed/Cancel handling. Grid/card screens update, with extra room dispatch
only under Peek; event/Neow are not universal concurrent result consumers. Closing
a grid with nonempty combat rewards overrides its remembered return to that screen;
otherwise specific remembered screen/swap rules apply, or NONE, not automatic map.
Native semantic parents must reproduce these conditions, not assume a UI stack.

| Obtain route | Constructor/entered-update distinction and durable result |
|---|---|
| Fast curse prevented by nonzero Omamori | Constructor consumes one charge/marks done; entered Fast update has its own done guard. No obtain/deck-change hook or insertion follows. |
| Show curse prevented by nonzero Omamori | Constructor consumes charge/marks done but completes bookkeeping, seen/poof work and duration initialization (0.5 fast/2 normal). Show's entered update calls card update and subtracts at most0.1; remainder stays positive, then dispatcher removes it. No obtain/deck-change hook or insertion, and no later accumulation. |
| Unprevented effect reaches obtain checkpoint | Relic obtain hooks, Soul obtain and deck-change hooks are ordered; Soul's call inserts into master deck immediately, not at flying animation completion. |
| Note direct incoming | No Omamori constructor/charge; actual obtain hooks, direct insertion, then deck-change hooks precede outgoing selection. Persistence is a different later boundary. |

**Supersession:** an oversized direct Show update could enter acquisition only
outside the admitted normal desktop domain. It is **not a phantom-curse fidelity
obligation or an unresolved ordinary caller outcome**. Do not restore such a stale
obligation from E03 or a generic delta argument. Equally, do not implement a universal
done prefilter: constructor bookkeeping and Show's entered card update still differ
from Fast. Native time choices cannot invent prevented curse insertion.

Object removal deletes the selected object, invokes its removal hook, then relic
deck-change hooks; string-ID/iterator removal does not inherit that sequence.
Parasite's maxHP−3 helper floors at1/clamps HP without local damage/death. Independent
site payment/damage remains separate: Shop pays at confirmed purge, Cleric before
its nonempty mandatory grid, Wing damages before later selection. Bonfire reward
precedes removal. Living Wall/Transmogrifier and each Astrolabe input remove before
replacement generation; Neow's single/double order differs as above. Note removes
through object hooks before retaining the live outgoing reference. No general
remove-before-generate or nonlethal-removal claim follows.

### 8.4 Exact Bowl, relic-child and exceptional-size contracts (F; E16)

| Site | Required local law |
|---|---|
| Singing Bowl | Enabled ordinary card offer only, not discovery/codex/draft/choose-one. Increase maxHP2, then request heal2 through modifiers/cap/dying guard; no full heal or max-only substitution. Remove only associated reward item, then close/hide. Null item at Neow/Dream is a no-op removal but still grants the legal +2/heal; last row becoming empty does not itself position rewards or establish readiness. Take/Skip/Bowl remain distinct. Absent/mistyped records are local counterfactuals, not legal stale candidates. |
| Bottles / Mirror | Typed bottle group must be nonempty; equipped relic consumes selected flag attachment. Mirror lacks a local empty-deck shortcut; selected stat-equivalent copy clears bottles and schedules Show obtain after local selection completes. |
| Astrolabe | Zero purgeables: no transform/grid. One–three: automatic all. More: select three. Remove each input before upgraded generation; timer/local flags do not prove drain. |
| Empty Cage | Zero: selection-complete flag set but room remains the earlier INCOMPLETE phase. One–two: automatic object removals and COMPLETE. More: grid waits for two. Preserve this difference without claiming initialized deadlock. |
| Pandora / Bell | Pandora iterator-removes starter-tagged cards, previews replacements then confirmation schedules Fast obtains; zero starters skips grid. Bell curse confirmation schedules Fast; later relic update opens three screenless relic offers even if curse was prevented. Removed automatic reward generation can already mutate pity. |
| Empty offers / belts | Zero-card helper omits its reward row; Dream omits empty screen. Full-belt potion reward stays available; Sozu reward claim consumes without insertion, unlike Shop refusal without payment/restock. |
| Mandatory grids | Generic fixed-count grid neither clamps count nor auto-skips. Purifier/Transmogrifier committed grids and Wing's later purge have no local empty guard. Note can lack a guard if incoming is unremovable and nothing else qualifies; a valid removable incoming otherwise supplies a candidate. Local paths need legal-history witness/exclusion before deciding native exceptional treatment, not invented Leave. |

### 8.5 Match and Keep's visible decisions and strict timers (F/O; E14/E16)

At A0 construct twelve instances and five attempts; Intro/rules lead into Play
without an invented initial strategic Leave. First face-down pick reveals exactly
one card, retains five attempts and cannot be picked again while face up. Second
pick reveals another, starts match1s/miss1.25s wait, but decrements nothing yet.
Strict underflow during the resolution branch removes a matching pair and schedules
one Show obtain from the **first card's fresh ordinary copy**, or turns a mismatch
down while preserving learned memory. Both consume one attempt and clear picks.
ID equality, not exclusive original/copy pairing, defines a match.

The fifth resolved pair sets game-done and wait1s. Next Play enters cleanup before
waiting for that timer to expire; with admitted delta≤0.1f its cleanup-entry wait is
positive. Cleanup moves remaining board cards once, then needs positive wait to
cross strictly below zero to show Complete; its button opens map. An obtain already
queued by the fifth match is not proof of insertion or drain. Five matches still
leave two objects; no ordinary sixth-match/empty-board branch is inferred.

Exactly zero is distinct: pair wait landing at zero allows the next Play input
branch instead of underflow resolution; cleanup can later land at zero and fail
its guarded underflow completion path. Nonpositive **initial** cleanup entry is not
the ordinary fifth-pair path under the desktop bound. These are conditional static
branches, not measured runtime frequencies. D04 must choose the native schedule
without claiming parity from erasing them. Public cells/order/memory and shuffle
posteriors follow §10.1; no second hidden identity leaks before the first observation.

## 9. Handoff, task and profile/time

**A1C-REQ-028 (A/P/O):** Task reward is exactly +1 once at resolved
`Act1Complete/Act2Ready`, −1 on actual death, otherwise zero. Both end with
terminated=true and truncated=false. No shaping, potion penalties, rollout/turn/
time cap, administrative defeat or retry. Act 1 success is not Heart victory.

Completion requires boss victory hooks/rewards, legal reward skips/claims, boss
relic take/skip and every resulting child/counter/key/acquisition effect at its
proper boundary. Retain one valid native successor state: actual player/resources,
permanent instances, relic counters/belt/keys, surviving one-time events, applicable
histories and RNG provenance. Act setup requests healing through the normal heal
method, not unconditional HP=max assignment; per-act resets do not refill the deck
or replenish consumed one-time events. A1C-D08 must pin which successor initialization
work has occurred before +1; no implementation may choose the cutoff ad hoc.

E16 closes the source path: boss-combat Proceed transitions to boss treasure;
next-dungeon Proceed records no-pick/warning where applicable and starts beaten/
fade state, but does not yet construct TheCity. Main-game transition-screen
completion constructs the successor with the **same player**. Base transition
advances act, reinitializes per-act lists/room probabilities, resets room potion
modifier and requests the A0 maxHP-sized heal through hooks; TheCity builds map
and empty starting node. Original card-RNG counter bands are not new RNG-001 parity
requirements. **P/D08:** successor-ready must include required child commits and
heal/reset effects; exact generated-Act2-map versus defined-entry payload remains
B05/B10. This local trace does not authorize Act2 gameplay or claim its map laws
closed. Legal unclaimed rewards need not all be taken to complete.

**A1C-REQ-029 (A/F/P/O):** Creation profile is an owned, validated value captured
before entropy draw/episode allocation. It includes only supported rule-relevant
history, not an imported account/save or arbitrary starting state. Explicit facts
are prior Ironclad Neow eligibility, per-act seen boss sets and Note ID/upgrades
(P/D05; E15). The source standard death/abandon predicate uses floor≥16, not boss
victory; native reproducibility seeds do not imply original custom-seeded Neow
mode. Act1 first-unseen order is Guardian, Hexaghost, Slime Boss; only all-seen
uses the seeded three-boss shuffle. Each boss adds discovery at its prebattle
hook, not announcement or victory; later death does not undo it. Later-act sets
remain separate carryover. A0 Note has no prior-win/cleared-Ascension gate.

Stored records retain ID and valid upgrade count only, not bottle/cost/modifiers/
misc growth. Ordinary upgrade bounds differ from repeatedly upgradeable Searing
Blow. Genetic Algorithm/Ritual Dagger prior-run scaling resets on reconstruction.
Incoming Note obtain/hooks/insertion/master-deck notification happen **before**
outgoing selection; incoming can itself be selected. Outgoing removal runs its own
hooks and saves a live reference; ID/upgrades are persisted on next room transition,
not selection. Do not prematurely reset live growth before removal hooks. Bottled/
non-removable eligibility and malformed-ID/count validation must be explicit.

Source missing preferences use Iron Wave/0; unknown-ID fallback to Iron Wave is
not proposed native validation. **P/D05:** raw profile fields are mandatory; resolve
any named preset into a full detached value before initialization/replay. Validate
standard permanent/removable kinds and each kind's upgrade domain, including
repeated Searing Blow, not color/rarity alone or a generic upgraded boolean.
Malformed kind/count/history rejects the whole batch before entropy/episode/time
allocation; rejection instead of fallback is a disclosed proposed deviation. No
note_enabled, arbitrary deck/resources/stance/orbs, next-boss override or real-save
path. Exact supported stored-card catalog remains a normative-content dependency.

### 9.1 Concrete detached profile recommendation (all new contracts P; E15)

Recommend a run-owned exchange state and a separate committed descriptor containing
only kind/upgrade count. This is typed **in-memory** commitment, not crash-atomic
parity with the source's separate preference writes/flush or real account I/O.

| Exchange state | Proposed ownership/output boundary |
|---|---|
| None | Ignore/no accepted exchange; prior committed descriptor unchanged. |
| IncomingApplied | Incoming mutations reached actual checkpoints; no invented outgoing result. Output progress tag only. |
| PendingTransition | Outgoing object removal/hooks finished; retain owned live identity/state. Deliver a detached staged ID/upgrades snapshot explicitly noncommitted. |
| Committed | Source-qualified next departure samples then-current retained ID/upgrades once, increments revision and clears pending reference. Earlier staged exports do not change by alias. |
| AbandonedBeforeTransition | Genuine death before departure abandons stage and retains prior commit, without forcing a transition. |

Recommend `ProfileCandidate` as a **partial-run**, non-policy result: contract
identity, captured creation profile, committed Note/revision (initially0), optional
exchange identity/stage/status, boss-discovery additions, current-run Neow evidence
and outcome status. Native codes/widths stay B10. Successful reads/steps deliver
detached copies; successful terminal capture freezes before replacement. Staged
output is not installable as a creation profile. Caller mutation/step/reset/close
cannot alter past deliveries. Note descriptor is policy-visible only at legitimate
event reveal; creation/history objects must not leak through ordering or metadata.

| Interruption | Recommended behavior, not approved native policy |
|---|---|
| Death before selection / after removal before departure | Preserve applied incoming/removal mutations, respectively no invented outgoing or abandon stage; neither forces Note save. Return actual death, not fake completion. |
| Death after commit | Preserve that commit and actual death final/partial-run candidate; death does not roll back or imply a completed account. |
| Fault before/after commit | Quiesce/fault pool with actual checkpoint/revision and committed prefix retained internally. No normal success/final/profile result, rollback, death conversion, retry or autoreset. Earlier exports remain valid. Bounded fault diagnostic copy is not ordinary ProfileCandidate; allocation failure need not allow such a copy. |
| Act2Ready | Capture after resolved handoff; do not flush an unrelated pending exchange. Report BossFloorReached, not FullRunCompleted. |

Boss additions and Neow eligibility evidence are different outputs. A genuine
Act1 death may report the observed floor predicate; Act2Ready cannot assert the
future full-run history. Caller may explicitly union discoveries/use committed
Note for a later profile but must choose next run's prior-Neow value. Every explicit
reset and vector replacement uses its configured creation profile, never preceding
slot output. Replay captures resolved profile and profile/time identities, semantic
choices and checkpoints, not current presets/files or imported revisions. A local
continuation acceptance row cannot override this still-unapproved death/fault output
choice. No automatic cross-episode inheritance is adopted.

**A1C-REQ-030 (O/P):** Native time must be deterministic and replayable if it affects
rules. E07 establishes Secret Portal's source eligibility at playtime≥800 seconds
in Act 3. That does **not** add Portal to Act 1; it makes successor-carried time a
real interface dependency. Creation elapsed-time input, advancement/wait semantics,
validity and replay must be chosen explicitly, not process speed, always-on or
zero-time policy. Reaper also exposes update/timer ordering but has no established
frame outcome. A1C-D04/A1C-B12 blocks clock/checkpoint finalization; adding a Wait
action without approved semantics would create a new gameplay choice.

### 9.2 Concrete time/settlement recommendation (F facts, P native choice; E15/E16)

**F:** source binary32 playtime adds admitted graphics delta while stopClock is
false, before screen dispatch; open screen alone is not clock stop. Final-boss
clock logic is not an Act1 boss stop rule. Portal tests elapsed≥800 at event
filtering in TheBeyond, not guaranteed selection or retroactive offer replacement.

**F:** Reaper schedules zero-duration VFX before its 0.25s vampire damage action.
After damage, orb VFX, conditional normal-mode0.3s wait and positive HealAction
are queued; postcombat pending filtering retains healing, unlike later endBattle
full clear. A front0.1s wait is separate. Actions finish on **negative**, not zero,
timer residual; Heal applies on first update, not its duration completion or visual
orb arrival. Unfinished current action update and later retirement/next-choice
visits differ; room monster updates precede ordinary manager dispatch. Monster
death timers (ordinary additions1.8s normal/1s fast, with other prior state/boss
logic) can trigger endBattle/victory hooks/full clear. These dependencies explain
the race without proving natural Reaper heal loss or a universal cutoff. Inspected
Reaper/orb visuals do not heal, but zero-duration work still needs its dispatch.

**P/D04 recommendation, NOT default/approved:**

| Quantity/input | Concrete candidate law and disclosed cost |
|---|---|
| Scheduler | Normal mode fixed, virtual60-Hz gameplay updates with binary32 delta1/60 (ties-to-even bits0x3c888889). Preserve source order, guards, strict comparisons, entered/pending work and first-update effects. Retire/select on subsequent manager visit, never recursive same-frame drain. Omit only rendering proven mechanic-free. Record frame-count checkpoints. Canonical frames are a deterministic surrogate, not original FPS/parity. |
| Elapsed time | Separate exact nonnegative signed-int64 units of1/3,000,000s; initial0, running-clock canonical dungeon frame adds50,000. Accepted dwell adds elapsed_us×3. stopClock suppresses elapsed contribution, not scheduler progress. Portal threshold2,400,000,000 units. Exact accumulation intentionally differs from source binary32 drift. No host/worker/Python/debug cost. Freeze task-captured time at terminal but carry successor running flag/value. |
| Decision dwell | At genuinely settled decision only, nonnegative integer elapsed_us accompanies accepted action/token; omission means zero dwell, not frozen automatic frames. Whole-batch preflight before charging immediately prior to action. No offer/menu regeneration, RNG or second turn; reject suspended/unresolved/terminal/faulted dwell. Caller cannot choose automatic frame count or intervene in Reaper's automatic race. |
| Finite policy wait | Recommend one WaitTo800Seconds candidate below threshold at running-clock settled decisions, including Act1 for carryover. Advance exact remaining units, retain parent decision, renew token; accompanying dwell must be0. No turn/reward/RNG/key/offer refresh, hidden-future-Portal condition or infinite timing-action enumeration. Candidate disappears at threshold. This is an explicit proposed timing abstraction/new choice, not already approved architecture. |
| Bounds/replay/knowledge | Checked nonnegative int64 inputs/units; preflight feasible arithmetic, post-dispatch accumulation exhaustion faults administratively, never time cap. Public elapsed/running state supports public timing control; scheduler internals stay private. Record time identity, accepted dwell/wait and frame/commit comparisons. Invalid rows advance nothing; recording/workers do not change schedule. |

Elapsed-only dwell/wait is legitimate **only** at quiescent decisions: any live
mechanic-bearing timer/callback needs its defined source continuation/canonical
frame path. The complete settlement/timer registry remains B04/B12, not closed by
choosing60Hz. Native schedule must respect §8.3's closed prevention facts and §8.5's
strict-zero distinctions. No Reaper-specific success/loss override is selected.
Recorded per-frame source-binary32 traces and explicitly declared semantic
checkpoint deviations remain D04 alternatives, not hidden fallbacks.

## 10. RNG contract and effect-site registry

**A1C-REQ-031 (A/F/P/O):** Preserve `spire-lab-rng-hmac-sha256-v1`, snapshot2 and
`hmac="=0.12.1"`. RNG-001 waives original seed/cursor and incidental stream parity,
not mechanic-relevant conditional/joint laws, finite arithmetic, retained offers,
history or without-replacement behavior. Same numeric seeds need not match the
original game/reference. Never attribute a mechanical mismatch generically to RNG.

Every actual random site requires a reviewed semantic domain/scope, draw time,
distribution/input measure, conditioning/history, intentional shared cursor or
joint dependency, and exhaustion policy. Fixed creation/copy need no random ID
draw, but can still have source placement/hook draws. Scope must not use worker,
slot, pointer, allocation, live token or completion order. Invalid input consumes
nothing. Exhaustion is an administrative error with provenance, not loss/reset.

Candidate families (P, **not frozen strings or independent streams**):

| Family | Semantic scope candidate | Required dependency retained |
|---|---|---|
| Map | Run + map-generation identity | Ordered walks/repair; retained bucket and fallback; not per-cell IID |
| Encounter selection/composition/HP/AI | Run + visit/combat + semantic actor/selection ordinal as appropriate | Queue rejection/history, composition multiplicities, selection-time move history |
| Card reward | Run + reward-event identity | Ordered slots/pity before ID retry; source-specific counts/modifiers |
| Relic initialization/offers | Run + initialization/offer identity | Five-pool joint law, owned removals, consumed rejection and depletion direction |
| Merchant stock/restock | Run + shop-visit/stock phase | Ordered proposals/retries, type/rarity/price/restock distinctions |
| Event selection/outcome | Run + visit + site/attempt | Pool removal/eligibility, repeated risk, pre-shuffled outcome prefixes |
| Combat generation/placement/target | Run + combat identity, with reviewed finer scopes only | Source-local offers, staged construction/insertion, repeated targets and effects |
| Instance upgrade/shuffle | Run + producing source/invocation | Distinct instance labels, prefix law and observable duplicate collapse |

The exact site inventory/codes, canonical derivation bytes and which families
share cursors are A1C-B10. One independent stream per card/source/slot is not an
approved shortcut. E14 now supplies the complete **conditional** finite laws below;
B03 is their input measure/order/joint-context disposition and actual-ensemble
gap, not a claim that only the three-item table exists. No exhaustive2^48 table
or source RNG port is required by this mathematical specification.

### 10.1 Exact finite mapper, observables and posterior (F/math; E14)

For Java-seeded shuffles let M=2^48, a=25214903917, c=11. States s are integers
in[0,M); one transition T sends s to(as+c)modM. Output bits of post-state t are
u(t)=floor(t/2^17). A seed word x induces g(x)=(x XOR a)modM. For each positive
bound b≤2^31−1 define accepted sets A(b,j), j in[0,b), and rejection set R(b):

| b | Acceptance/output relation |
|---|---|
| Power of two | Every post-state accepted, j=floor(b×u/2^31). |
| Other | L=b×floor(2^31/b); accept u<L with j=u mod b, reject u≥L. |

With D(S) the diagonal indicator of set S, the bounded-draw final-state kernel is
**Q(b,j)=Σ over r=0,…,M−1 of (T D(R(b)))^r T D(A(b,j))**. The LCG's full M-cycle and
nonempty acceptance set give a finite first-accepted path for every initial state;
ΣjQ(b,j) is row-stochastic. Rejected advances cannot be discarded or replaced by
independent normalized output marginals.

Initially label each of n ordered list instances distinctly. Let S(b,j) exchange
positions b−1 and j, and U(b)=Σj Q(b,j)⊗S(b,j). For input-state law ν and list A,
the joint state/list law is **(ν⊗δA)U(n)U(n−1)…U(2)**, with identity product for
n≤1. Marginalize final Java state only when no later dependency needs it; call the
list law Kν(A,p). An observable's probability sums **actual K weights** over its
fiber, not equal permutations. This completely defines arbitrary finite-size
conditional shuffles within source index bounds, without claiming an efficient
future sampling algorithm.

U64 seed input gives each state2^16 preimages, hence ν=1/M; all event masses are
multiples of1/M. For n≥3 ideal labeled uniformity is impossible because n! has an
odd divisor; n≥17 also has n!>M, so some labeled permutations are outside the seed
image. Fixed seed is a point mass; other input measures push through g instead.
None of these facts prove original root/history input is U64 or independent.

For retained latent object z with joint weight w(z), legal public transcript h
restricts it to C(h). The next observable Y has mass
**Σ[z∈C(h),Y(z)=y]w(z) / Σ[z∈C(h)]w(z)**. Zero denominator means no conditional
law under that chosen measure, not retry. Preserve linked producer/context state
before projection. Public-history-only policy action likelihood cancels among
compatible states; hidden-informed/external conditioning needs its own likelihood.
Reveals, misses, matched removals, upgrade results, rejected/accepted offers and
skips condition this retained weighted object, never a fresh uniform compatible
shuffle. Hidden rejection traces remain private and are summed out.

| Observable/source | Exact construction and required discriminator |
|---|---|
| Face Trader | Filter owned IDs **before** shuffle from ordered CultistMask/FaceOfCleric/GremlinMask/NlothsMask/SsserpentHead; empty uses singleton Circlet. Consume one misc source long even at0/1 missing. Obtain first-entry copy. Apply head projection of K with history-conditioned input: under U64 two missing are exactly halves, five cannot all equal1/5. Repeated trades rebuild current missing list, not successive heads of one old list. |
| Match board | Six originals in order Rare/Uncommon/Common/Uncommon-colorless/random-curse/current-character event starter(Bash); preview hooks precede stat-equivalent six-copy suffix. Shuffle twelve distinct labels once with misc long, then place index i at(column i mod4,row i mod3), not row-major. Project complete revealed records; match by ID. Duplicate-ID fibers sum weighted permutations; six distinct pairs have7,484,400 ID boards and cannot be uniformly weighted under U64. Retain original cells and revealed memory after removal/miss. Board-generation/context and seed law stay joint, not assumed independent. |
| Upgrade prefix | Filter master-deck **instance encounter order** by canUpgrade (Attack for Whetstone, Skill for War Paint). One misc seed long even empty/singleton. Shining Light/Whetstone/War Paint upgrade first min(2,n), Tiny House first min(1,n). Sum exact ordered prefix weights, then any desired unordered/kind projection; never collapse same-kind instances first. Tiny House checks bottle links on first **and second** when available despite only one upgrade, then its potion source receives following misc state. |
| Five relic pools | Pre-removal ordered counts33/30/28/17/22 in Common/Uncommon/Rare/Shop/Boss order. Five successive relic source longs seed five separate Java shuffles; Java rejection is local, not a new source-long draw. Ordered inputs arise from shared hash-map then actual-character hash-map population, not display sorting. Apply source-specified owned-ID deletions **after** shuffle, preserving survivors; floor≥1 adds currently owned IDs to that deletion list. Offers push the joint retained pools through exact requested-tier front/end/depletion/predicate rules in §8.1, including caller checks after promotion; skip never restores consumption. |

For fixed ordered catalogs A and deletion context O, let η(s1,…,s5) be the joint
folded-seed law and J(A,s) the deterministic kernel output. The pool law is the
η-weighted image of **delete_O(J(A_C,s1),…,J(A_B,s5))**, retaining source/context
state for later draws. Random A/O must remain joint with η, especially if ownership
reveals earlier offers. No product of marginal tables is justified automatically.
Postshuffle deletion does not commute: for U64 shuffle of(0,1,2), delete1;
survivor orders(0,2)/(2,0) have numerators140737488352549/140737488358107 overM,
not M/2 each from shuffling survivors alone. The difference2779/M is exact, not noise.

### 10.2 Decision-ready input measures/order, not approved defaults (P; D03)

An initialized semantic block means one declared U64 **input word**, transformed
to the source initialized state, then linked state/word relations—not five IID
words. Its conditional mathematical relation can be fixed without adopting an
executable source RNG. On64-bit words define Z(x)=x XOR(x>>33) and
H=Z∘multiply(0xc4ceb9fe1a85ec53)∘Z∘multiply(0xff51afd7ed558ccd)∘Z modulo2^64.
Replace word0 by2^63 before initializing σ(r)=(H(r),H(H(r))). Thus zero/minimum
alias, with2^64−1 distinct initialized states and doubled alias weight under U64.
For state(u,v), the next pair is(v,z), with
z=(I XOR R17)(I XOR L23)u XOR(I XOR R26)v; I is identity and L/R zero-fill bit
shifts. Call this pair-state map X; emitted word is(v+z)mod2^64. Five such advances,
folding each word through g, define η;
retain the final state for linked draws. For intervening positive **bounded-integer
primitive** b, include first-accepted raw transitions using u=floor(word/2),
L=b×floor(2^63/b), accept u<L, result u mod b. Int wrappers first convert inclusive
endpoints. This is **not** wrapper-long/double-scaling arithmetic or the Java Q.
The finite nonzero state graph may formally include a wholly rejecting-cycle
no-return marker; no general later-bound termination theorem or reachable failure
is asserted. Five initialization unbounded longs require no such rejection proof.

| Input option | Fidelity consequence / disposition |
|---|---|
| U64 per explicitly independent invocation | Exact finite per-call bias retained; five independent pool seeds would impose product ν^5 and discard source joint/cursor information. Independence from board generation or later Tiny House potion is not already approved. |
| Initialized semantic block | Recommended for relic initialization and linked draws; preserves initialized within-block image/cursor relationships without unrelated root coupling. Needs declared block boundary, ordered inputs and linked callers. |
| Uniform full nonzero128 state block | Keeps sequential relation but changes initialized support/weights. X is invertible; each emitted word has2^64 preimages except zero loses one. This is not the initialized law or proof of product marginals. |
| Actual root/profile/history ensemble | Declare prior, policy likelihood and earlier draws/rejections, then push jointly to source entry state/catalog/context. No evaluated full Act1 ensemble is supplied; source determinism chooses no user prior. |
| Ideal uniform permutations | Explicit extra deviation replacing mapper bias/support, not authorized by seed nonparity. |

Recommend isolated-event/upgrade U64 only with reviewed independence boundaries;
otherwise keep a joint conditional input. Separately propose ascending UTF-8-byte
constructor-ID relic catalogs, explicitly deviating from unestablished historical
host hash-map order. Source-host order is a valid alternative only when established.
The law is complete **conditional on those declared inputs**, not approved by
choosing a name or assuming deterministic HMAC is IID over an unspecified population.
Native sampling still uses the pinned HMAC contract; semantic site derivations,
efficient exact sampling and remaining producer/context laws are B03/B10, not
permission to substitute a convenient universal sampler.

## 11. Packed observation and candidate proposal

**A1C-REQ-032 (A/P/O):** Use fixed-width numeric rows in variable-length packed
tables, separately versioned from combat-v1. Legal piles/decks/menus can grow:
Hello World plus sustained defense against a non-scaling Louse can grow combat
piles; Frozen Eye can make growing ordered sequences public. Ten-card hand capacity
is not a deck/pile bound. Do not impose a guessed 256/512/4096 limit, truncate,
page away choices or force gameplay resampling on allocation failure.

Proposed transport for each table t: `int64 values[M_t,W_t]`, native-endian,
C-contiguous; `int64 offsets[B+1]`, starting zero, nondecreasing, ending M_t;
optional aligned `int8 known[M_t,K_t]` containing only 0/1. Adjacent offset
difference is the sole length; offsets count rows. An all-empty table retains
`[0,W_t]` shape. Cross-table references are tagged **slot-local public rows**,
absent −1; never global offsets/native IDs. Validate against the referenced slot
segment. Knowledge mask shares its table's offsets.

### 11.1 Candidate descriptor, not an assigned fingerprint

The following is a concrete minimum field proposal, **not a complete frozen
descriptor**. Exact code vocabularies, bounds, all rows/widths and extension
relationships are A1C-B09/A1C-B10. No hash is invented for an unfinished schema.

| Table / fixed block | Candidate fields and width commitment |
|---|---|
| Player/context (fixed) | Phase, room/decision kind, HP/max HP/gold, keys, potion capacity, public floor/node, energy/Block/turn, stance, orb capacity, selection min/max/remaining and orderedness; D04 proposal adds public elapsed units/running-clock state and finite wait availability; exact width open |
| Permanent cards | Kind, upgrades, public scaling/overrides, bottled/property flags and multiplicity/relations where sound; width open |
| Combat cards / public pile knowledge | Kind, upgrades, public base/turn/payable cost classification, damage/Block/scaling, zone, visible position/known-order facts and sound public source relation; width open |
| Relics / potions | Catalog kind, public counters/state; potion physical slot and phase-use facts; no unobserved counter leak; widths open |
| Map nodes / edges / visits | Public coordinates/typed room/marker/reveal/visited flags and directed edges; boss connector separately identified; widths open |
| Enemies / powers / orbs | Public kind/position/HP/Block/powers; legitimately visible intent; orb kind/order/payload and duration state; widths open |
| Room objects / choices / memory | Visible stock prices/offers, public board positions/reveals and source selection progress; no hidden faces encoded; widths open |
| Candidates | Proposed width **6**: kind, source-table tag, source-local-row, target-local-row, parameter-start, parameter-count. Target table is declared by candidate kind; no overloaded ambiguous reference. |
| Candidate parameters | Fixed-width typed numeric rows carrying ordered choice parameters/option codes; own per-slot offsets, local start/count; exact width and codes open |

A descriptor must enumerate stable table IDs, column order/meaning/dtype/bounds,
mask mapping, enums, absent/unknown semantics, canonical ordering, candidate kind
and parameter vocabulary, ownership and task/final identity. Enum values are
explicit reviewed codes, never compiler enum casts. Proposed machine-range int64
transport is not proof every legal gameplay numeric field fits. Audit each source
numeric law versus transport and distinguish real mechanic caps from resource
exhaustion. Checked rows/widths/bytes/offsets/conversions/allocations are mandatory.

**A1C-REQ-033 (A/F):** Project only legitimate current public knowledge. Include
resources/inventory, visible map/announced boss, revealed room/offer/choice facts,
current combat and relevant learned facts. Exclude seeds/RNG/cursors, hidden future
encounters/question rooms, unoffered pools, future shuffles, arbitrary IDs/tokens,
worker/slot allocation and diagnostics. Table lengths/order/references are features
too; no covert hidden-state-dependent sorting.

Frozen Eye reveals current draw order, not the next reshuffle. Without it, preserve
unordered public card facts without hidden ordering; known top/bottom placements
and Scry have their own reveal lifetimes. Runic Dome suppresses intent and damage
features that reconstruct it. Match face-down position exists but value is unknown;
retain legally revealed memory. Absent, present-unknown and known-zero differ.
Canonical menu order preserves visible significant positions, otherwise complete
public semantic keys; equal names do not collapse different costs/scaling/bottles/
provenance. Forethought+ retains the **selected sequence**, not sorted subset.

**A1C-REQ-034 (A/P/O):** At a live decision with N candidates, legal action is local
index `[0,N)` plus current native provenance. Candidate rows privately resolve to
exact legal core choices. Terminal observation has zero candidates. Staged
multi-card choices expose actual count/commit/cancel/order semantics without
enumerating all subsets or introducing next-page policy actions. Native stale or
cross-slot token rejects; integer-only facade cannot detect old intent when an
old index is coincidentally legal in the new menu. Tokens are not replay identity
or learned features. Reads preserve menu/token/RNG; accepted changes rebuild.

### 11.2 Standard-space facade recommendation (P)

Native packed transport remains the efficient bulk interface. A single facade
could expose `Dict(fixed, tables)`, each variable table a stacked numeric Sequence
of row dictionaries (`values`, optional `known`), omitting native offsets. Vector
tables become tuples of B detached segment dictionaries, not object-per-card
arrays or padding to a guessed game maximum. The candidate index carrier proposal
is `Discrete(2^63−1, start=0, dtype=int64)`, vector carrier its O(B) declaration.
It is a representation range, **not** that many actions/logits/mask entries.
Actual legality remains `[0,N)`. Never allocate in proportion to the carrier.

Candidate-aware policy sampling uses bounded integer draws from published positive
counts and its own explicit policy RNG, never gameplay RNG or float-scaled sampling.
Proposed scalar input: built-in int or NumPy int64 scalar, not bool/float/other
integer dtype/scalar array. Proposed vector input: native-endian NumPy int64[B],
detached before validation, no lists/coercion. Detailed choices await A1C-D07.

This is not universal learner/wrapper compatibility: dense action heads/masks,
one-hot/FlattenObservation, generic Sequence shared-memory vectorization and
SAME_STEP-rejecting wrappers can be incompatible. E12's installed Gymnasium 1.4.0
inspection is historical static library context, not executed facade acceptance.
Later checks must cover exact spaces/tree/empty rows, applicable passive/lifecycle
checks, single RecordEpisodeStatistics/identity TransformObservation, vector
RecordEpisodeStatistics/DictInfoToList, mixed finals and partial resets. No upstream
checker patch or illegal-action repair is permitted to claim acceptance.

## 12. API, reset, replay and delivery

**A1C-REQ-035 (A/P/O):** Proposed native operations are:

| Operation | Required contract |
|---|---|
| Create pool | Validate batch/workers/partition/reset mode/recording and all descriptors; no hidden game/profile process singleton |
| Reset selected unique slots | Owned profile/seed request per slot; preflight/stage/reserve before installation; no duplicate index or partial invalid-input acceptance |
| Read selected slots | Detached current decision/metadata; no state/RNG/token allocation advance |
| Step configured slots | One local candidate/token per row; whole-batch preflight; one native dispatch/delivery, ordered results. If D04 selected, accepted elapsed_us and WaitTo800Seconds use §9.2's preflight/zero-dwell/settlement rules, not caller-controlled automatic frames. |
| Replay from start | Validate versioned creation record; reproduce accepted choices through normal menus, not state import |
| Close | Quiesce/release pool; delivered arrays/finals stay valid |

Exact signatures, partial-reset API, metadata columns/error codes/leases and
descriptor version IDs are A1C-B10. These names are recommendations, not a current
product API. Single step returns observation/reward/end flags/info; vector rewards
are proposed float32[B], flags bool[B], logical slot order. Ordinary truncation is
always false for this uncapped task.

**A1C-REQ-036 (A/P):** Single terminal state requires explicit reset. Vector uses
same-step autoreset: retain independently owned final observation/resources/
context/outcome/identity before replacement. Reward/end flags belong to the
stepped episode; ordinary returned observation belongs to current/replacement.
Never apply the old action to replacement, drop a finished row, or create fake
finals for live rows. Profile effects do not implicitly alter reset input.

Recommended native finals: bool[B] presence, ascending final_slots[K], independent
K-segment packed batch and matching retired-episode metadata. Facade recommendation:
`final_obs` length-B object array of complete single observations or None with
matching `_final_obs`; masked final_info records describe retired episodes,
ordinary metadata describes current episodes. Exact info keys are A1C-B10.

All observations, candidates, profiles and finals are detached from live state,
scratch and cached menus. A NumPy write-protection bit alone does not prove
ownership. Retained output survives subsequent step/reset/close and caller
mutation cannot corrupt native authority. Views retain their detached owner.

**A1C-REQ-037 (A/P/O):** Replay is opt-in from-start, not arbitrary snapshot restore.
Record detached resolved creation profile, root/seed-adapter/partition, semantic
run identity, rules/content/schema/task/RNG versions and accepted semantic choices
with canonical mapping. Recreate through ordinary initialization and menu checks;
tokens need not reproduce. Record approved logical time inputs/advances when part
of rules. Mismatch is a provenance-bearing error, not another action or state
substitution. Recording disabled must not change gameplay. External nondeterministic
allocation/export failures cannot be fabricated as deterministic clean replays.

## 13. Batch validity and administrative failure

**A1C-REQ-038 (A):** Validate every slot/input type/shape/range/readiness/phase/token/
menu member and required pre-dispatch reservations before any gameplay transition.
One invalid row rejects the entire batch with no state, RNG, reward, token,
profile, history or logical episode/ordinal allocation advance. No coercion,
clamp/modulo, invalid-row drop, no-op mapping or strategy retry. This is preflight
atomicity, not rollback after dispatch.

**A1C-REQ-039 (A/P):** Every slot exclusively owns mutable state. Assign semantic
identities before parallel dispatch, use pool-private persistent workers (direct
native iteration at effective-one), publish logical slot order. Equivalent run
and accepted choices agree across worker counts, neighbors and packing.

Post-dispatch core/encoding/allocation/binding/facade delivery failure quiesces
workers and administratively faults the pool, retains applied mutations and first
fault slot/stage/cause/RNG provenance, and emits **no successful batch**. No rollback,
retry, silent reset, fabricated defeat/truncation or duplicated action after export
failure. Keep operation lease through final/table packing and facade delivery so
failure attaches to current/newly retired records, never previously delivered ones.
True runtime construction failure has the same honest no-rollback/fault boundary;
invalid reset input still rejects before installation. Exact recoverability is
not permission to resume a faulted pool; any post-fault inspection API is A1C-B10.

## 14. Compatibility and structural performance constraints

**A1C-REQ-040 (A):** Existing supplied-ready combat semantics/APIs/schema/fingerprint
remain unchanged. Preserve `spire-lab-combat-reference-v2`,
`spire-lab-env-seed-sha256-v1`, `spire-lab-combat-observation-v1`, fingerprint
`8fd66d212259d2bdd27139988a772e1a6c223f8d65524f38c74174a0d0843357`, plus
the RNG pins in section 10. Run descriptors/action/replay/task identities are
separate. Same-engine extensions require standalone regression coverage, not a
second approximate engine or widening combat-v1 arrays under the old identity.

Rules/automatic resolution/projection/menus/encoding remain Rust-native; one
external batch call, no Python callback per slot/card/effect. Share immutable
catalogs; reuse private scratch/cached decisions soundly; no routine full-run clones,
per-step worker creation, global mutable gameplay lock or always-on trace/JSON/I/O.
Work may scale with required current public state/menu size, not accumulated debug
history. No combinatorial subset expansion when faithful staging suffices.
Honest bulk copies are preferable to unsafe aliasing. No GPU simulator, learner,
benchmark or speed claim follows. Memory budget includes fixed bytes plus each
table's rows×width×dtype, offsets/masks, independent finals, retained exports and
facade containers. Legal public growth is not a reason to discard information.

External reference comparison later uses the selected Daniel Ziegler fork at
`84ab3ead9d88c01c4be09a6773fb9a947f57d5ff`, not an unpinned oracle. Compare
equivalent resource/state/action and semantic-choice boundaries; separately test
conditional laws. Classify product defect, evidenced reference defect/limit,
approved deviation or unresolved question. Agreement is not original-runtime
validation; missing reference coverage is not a pass. Do not copy implementations.

## 15. Finite acceptance obligations — all UNEXECUTED

**A1C-REQ-041 (A/O):** The complete checkpoint needs exact authored cases, generated
trajectories, public-knowledge checks, real binding/ownership/fault tests and fresh
independent integrated review. Statistical frequencies and comparator agreement
supplement but do not replace exact laws. No row below reports a test run in this
documentation task. Each future case must record expected state, durable commits,
source/RNG boundary, observation/menu and task outcome; open laws are resolved
before asserting their expected result, not converted into permissive tests.
Actual implementation tests belong after separate plan/start authorization; written
spec approval needs the precise expected contracts and review, not premature product
execution to manufacture a pass.

| Case ID | Discriminating acceptance obligation |
|---|---|
| A1C-T01 | Generated standard start; both prior-eligibility Neow branches, drawback before reward, every category/child; invalid profile leaves entropy/identity unchanged. |
| A1C-T02 | Supplied map-choice topology: boundary two-way versus clamping; repeated-parent repairs, coordinate quirk, crossing snap and lowest-x trim. |
| A1C-T03 | Quota binary32 rounding with row-13 omission; row-major retained bucket, skipped entries, deferred fallback and fixed-row exceptions. |
| A1C-T04 | Preserve run5988/map5989 MODEL requested3/assigned2 certificate, rank-barrier criterion and same-seed topology/bucket proof obligation, distinct from initialized zero existence. Disabled marker gates draw nothing; enabled empty increments wrapper only before failure, singleton draws; approved native administrative classification, no reroll/forced Elite/death. Group buff/Insect order remains separate. |
| A1C-T05 | §10.1 finite first-return rejection/final-state and descending composition under approved measure; missing faces0–5 including deterministic-result seed consumption; weighted duplicate-ID board/reveal posterior; empty/singleton/ordered duplicate-kind upgrade prefixes/Tiny House second bottle check; postshuffle-owned-deletion2779/M counterexample, five-pool joint/posterior/requested-tier depletion; declared input order versus silent sorting. |
| A1C-T06 | All weak/strong/elite/event/boss families and ordered compositions; first-strong exclusions, last-two/elite history and queue consumption on departure, not event fights. |
| A1C-T07 | Each of 25 enemy HP/setup/AI/move-history records; Slaver 45/55, Jaw/Acid redirects, Lagavulin event Block0/Siphon Artifact, Sentry order. |
| A1C-T08 | Slime split versus lethal/Fatal/spawn snapshot, Guardian queued mode/Sharp Hide, Hexaghost Divider/Inferno/in-flight Burn, Looter theft/refund/escape. |
| A1C-T09 | Each card's base/upgrade primary record plus every CARD-* shared record; coverage cannot close by counts alone or successful random runs. SI-I2: one qualifying multi-HP loss versus multiple losses, zero-loss prevention, notified hand/draw/discard locations, fresh versus stat-equivalent copy and per-instance cost layers. |
| A1C-T10 | Hand/autoplay/five repeats/Omniscience matrices with target death, mixed entitlements, X=0/nonzero/Chemical X, flags and exact payment/history/copy/destination. SI-I1: Setup singleton and selection routes with positive `cost`/changed turn cost (including turn zero), base zero/positive turn-only cost, X and unplayable; distinguish flag from numeric fields, survival across ordinary reset and clearing on use. |
| A1C-T11 | Draw/movement/creation, No Draw, Void/Deus/Endless Agony; discard causes/Reflex/Tactician; exhaust Spoon/Branch/Embrace/Fiend Fire snapshot. SI-I2: legally acquire Masterful Stab, exhaust via True Grit+, lose positive HP while it is exhausted, Exhume the same instance; no missed-notification cost increment, unlike a global combat-history formula. |
| A1C-T12 | Retain/Ethereal/Pyramid/Establishment; Poison/debuff/Artifact/Explosion; Focus/orb zero/full/Dark/Plasma; stances/Mantra/Scry/extra turns. |
| A1C-T13 | Non-final Feed/Greed/Lesson/Genetic/Dagger durable mutation; temporary copy/upgrade isolation; later death preserves committed resources. |
| A1C-T14 | Fairy-before-Tail inspected branch and other lethal/prevention/heal origins; modifier-zero rescue/repeated damage. Under approved time contract, Reaper nonfinal/last hit/retaliation/rescue/death/no-heal with queued/current/first-update healing, VFX counts and death timer variations; no universal loss or always-drain. Strict-zero/negative and next-visit retirement must match the chosen schedule. |
| A1C-T15 | Each of twenty generation rows at actual inner effect site, including off-color autoplay/repeat; Source support/mass/retry/history and no pity mutation. |
| A1C-T16 | Master Reality Searing Blow at hand 9/10, Foreign Influence and overflow; Eviscerate copy cost; n-position insertion excluding new top; selection before insertion. |
| A1C-T17 | Alchemize/Brew live capacity, proposal-before-obtain, combat/outside Sozu and Fruit Juice, own consumed slot/self-production; each potion Bark/phase/target/skip law. |
| A1C-T18 | All25 event eligibility/branches, no invented initial Leave; Scrap Ooze26/100, Wheel reveal gold once. Match first/second observation, ID match/five delayed attempts/fresh-copy obtain/fifth cleanup entry positive versus later strict-zero landing, no sixth attempt or obtain drain. Woman unlabelled open()/Proceed versus Lab/event-fight direct map. Guarded empty event returns versus committed empty-grid reachability witness/exclusion. |
| A1C-T19 | Merchant proposal/retry/stock/sale/prices/discounts/removal/Courier; cancel purge versus confirmed result/no double charge; child onEquip before restock, Grid Peek/nonempty-reward return override and close/reenter same inventory, not universal automatic Shop screen return. |
| A1C-T20 | Ordinary/Sharded reward/pity-before-claim; Skip/reopen same row, Take consumes before insertion, Bowl distinct +2maxHP then heal2 at full/damaged/refused-heal state with null/last/mixed associated rows. Empty card/Dream offer omitted. Dream/Orrery/Tiny House sources retain history of discarded automatic rows. |
| A1C-T21 | Each 141 relic primary/global record plus 32 spawn predicates; consumed rejected/skip offers; normal→end/requested-tier and depleted→normal/front, boss front/fallback. |
| A1C-T22 | Chest correlated tier/gold, separately rounded amount endpoints, base-linked sapphire versus extra relics, deferred emerald/ruby, legal skip/key deferral. |
| A1C-T23 | Campfire initial filter→Recall append→scan, Dripper/Hammer plus usable Recall, empty Smith/Toke targets, Rest/onRest/Dream child. |
| A1C-T24 | Fast/Show prevented curse admitted delta0/0.1 and rejected>0.1: one charge, no acquisition/deck-change/insertion; Show entered card update/bookkeeping versus Fast guard, no universal prefilter/phantom curse. Note direct incoming has no Omamori charge; retrieve-and-store same removable card, removal/live reference/ID-upgrade-only transition save. Under approved output choice, detached stage/one commit/revision/abandonment and death/fault before/after; synthetic reference mutation is not natural witness. |
| A1C-T25 | Boss chest/no-pick/legal skips and relic-owned children: Astrolabe0/1/3/4, Cage0/1/2/3 phase differences, Pandora0/nonzero starters, Bell prevented curse still relic offers. +1 after resolved children/transition heal/per-act reset and chosen entry/map cutoff, not close/fade. |
| A1C-T26 | Descriptor/table shapes including empty/mixed segments, slot-local refs, ordered-selection parameters, numeric/offset/allocation checks and actual legal large growing piles. |
| A1C-T27 | Hidden-state/private-handle renaming preserves values/order/lengths/masks/menu; Frozen Eye/Scry/board legitimate reveals change only entitled facts; Dome has no intent-derived damage leak. |
| A1C-T28 | First/last candidate, negative/one-past, stale/cross-slot token/global-offset confusion; invalid final batch row leaves every state/RNG/token/reward/ordinal unchanged. |
| A1C-T29 | Single explicit reset; vector mixed/consecutive SAME_STEP endings, K=0/K=B, final/replacement identity/resources and no action applied twice; detached profile reset isolation. |
| A1C-T30 | Retain/mutate exports/finals through step/reset/close; injected core/encoding/binding/facade allocation faults quiesce pool, preserve committed prefix and current-delivery provenance, no success result. |
| A1C-T31 | From-start detached-profile/time replay; version mismatch, recording/worker/packing equivalence includes frame/time/commit revisions. All8 Act1 seen subsets, prebattle versus announcement, prior Neow floor15/16 evidence independent of unlock/project seed. Under approved time choice, running/stopped clock,800s boundary and Wait independent of hidden rooms, no offer reroll, elapsed overflow/invalid-last-row nonadvancement; waiting after sampling cannot replace an event. |
| A1C-T32 | Approved facade carrier/Sequence/space/tree/wrappers/partial reset; no O(2^63) allocation, exact caller-RNG bounded sampling; all legacy combat-v1 regressions and known-answer RNG fixtures. |
| A1C-T33 | Generated end-to-end trajectories through every applicable room/route, legal death/skip/key deferral and full handoff, including rare off-color/Note/relic chains; no production filtering or seed-search miss used as exclusion. |
| A1C-T34 | Structural hot-path/ownership/source-policy review and fresh independent integrated review; scope/evidence/administrative limits disclosed separately from any later performance or transfer claim. |

## 16. Requirements traceability ledger

IDs are stable within this consolidated document. A range in a cell means the
explicit listed cases/blockers, not an open-ended promise. All requirements remain
required; **no requirement is claimed fully accepted by this integration draft**.
Architecture constraints marked A retain their earlier acceptance, not new API
or whole-spec approval. Evidence references below do not mean tests executed.

| Requirement | Contract location | Evidence basis | Future cases | Remaining blocker |
|---|---|---|---|---|
| A1C-REQ-001 | §2 scope/start | E08/E09/E12 | T01,T33 | B07,B08,B11 |
| A1C-REQ-002 | §1/§2 evidence boundary | E01–E07/E08 | T34 | B01,B14 |
| A1C-REQ-003 | §3 owners | E05/E08 | T30,T34 | B10 |
| A1C-REQ-004 | §3 single authority/overlay | E05/E08 | T10,T13,T32 | B04,B10 |
| A1C-REQ-005 | §3 live durability | E02/E03/E05 | T13,T14,T30 | B04,B05 |
| A1C-REQ-006 | §4 phases/resolution | E03/E05/E08 | T10,T25,T33 | B04,B05,B12 |
| A1C-REQ-007 | §4/§8.2–8.5 continuations | E03/E05/E16 | T18–T25 | B05 |
| A1C-REQ-008 | §4.1 origins/payment; §5.2 corrections | E02/E10/E11/E09-R | T09,T10,T11,T16 | B04,B07 |
| A1C-REQ-009 | §4.2 scheduler | E02/E10/E11 | T08,T11,T14 | B04,B12 |
| A1C-REQ-010 | §4.2 damage/rescue | E02/E03/E11 | T08,T13,T14 | B04,B07,B12 |
| A1C-REQ-011 | §5/appendix A cards; §5.2 corrections | E09 (qualified by E09-R)/E10 | T09–T13 | B04,B07 |
| A1C-REQ-012 | §5 off-color mechanics; §5.2 corrections | E08/E09 (qualified by E09-R)/E10 | T09–T12,T15,T24 | B04,B07 |
| A1C-REQ-013 | §5 specials/Note | E03/E09 (qualified by E09-R)/E12 | T09,T11,T24 | B07,B11 |
| A1C-REQ-014 | §6 shared resolver | E05/E08/E10 | T15,T17 | B10 |
| A1C-REQ-015 | §6 site policies | E05/E10 | T15–T21,T34 | B04,B05,B06,B08,B10 |
| A1C-REQ-016 | §7/§7.1 map kernel/initialized model | E01/E10/E13 | T02,T03,T04 | B02,B03,B10 |
| A1C-REQ-017 | §7/§7.1 burning/empty cursors | E01/E10/E11/E13 | T04,T22 | B02,B04 |
| A1C-REQ-018 | §7 encounters | E09/E11 | T06,T07,T08 | B07,B08 |
| A1C-REQ-019 | §7 enemy mechanics | E02/E11 | T07,T08,T14 | B04,B07 |
| A1C-REQ-020 | §8/§8.2/§9 Neow history/children | E03/E09/E10/E15/E16 | T01,T31 | B05,B08,B11 |
| A1C-REQ-021 | §8/§8.5/appendix C events | E03/E10/E14/E16 | T18,T33 | B03,B05,B08,B11 |
| A1C-REQ-022 | §8/§8.2–8.4 merchant | E03/E10/E16 | T19,T21 | B05,B06,B08 |
| A1C-REQ-023 | §8/§8.3–8.4 rewards/transforms | E03/E10/E16 | T13,T20,T24 | B04,B05,B08 |
| A1C-REQ-024 | §8.1/§10.1/appendix D relics | E04/E10/E14 | T05,T21,T25 | B03,B06 |
| A1C-REQ-025 | §8.1/§8.4/appendix D potions | E09/E10/E16 | T10,T14,T17 | B04,B05,B08 |
| A1C-REQ-026 | §8.1/§8.2 camps | E03/E04/E16 | T23 | B05,B06,B08 |
| A1C-REQ-027 | §8.1/§8.2 keys/chests | E03/E10/E16 | T22,T23,T25 | B04,B05,B08 |
| A1C-REQ-028 | §9 handoff/task | E03/E05/E07/E08/E16 | T25,T29,T33 | B05,B10 |
| A1C-REQ-029 | §9/§9.1 profile; §1.1 bounded Note disposition | E03/E05/E09 (qualified by E09-R)/E12/E15/E16 | T01,T24,T29,T31 | B07,B10,B11 |
| A1C-REQ-030 | §9.2 clock/settlement recommendation | E02/E07/E15/E16 | T14,T18,T24,T25,T31 | B04,B12 |
| A1C-REQ-031 | §10/§10.1–10.2 RNG/finite laws | E01/E05/E08/E10/E11/E13/E14 | T02–T08,T15–T22,T31,T32 | B02,B03,B10 |
| A1C-REQ-032 | §11 schema/growth | E05/E08/E12 | T26,T32 | B09,B10 |
| A1C-REQ-033 | §11 knowledge | E05/E08/E09 | T18,T24,T27 | B07,B10 |
| A1C-REQ-034 | §11 action/menu | E05/E08/E12 | T10,T26,T28,T32 | B05,B10 |
| A1C-REQ-035 | §12 API | E05/E12 | T26,T28–T32 | B10 |
| A1C-REQ-036 | §12 reset/finals | E05/E08/E12 | T24,T29,T30 | B10,B11 |
| A1C-REQ-037 | §12/§9.1–9.2 replay | E05/E08/E15 | T31 | B10,B11,B12 |
| A1C-REQ-038 | §13 preflight | E05/E08 | T01,T28,T31 | B10 |
| A1C-REQ-039 | §13 fault/parallel | E05/E08 | T26,T29–T31 | B10 |
| A1C-REQ-040 | §14 compatibility/performance | E05/E08/E12 | T32,T34 | B10,B13 |
| A1C-REQ-041 | §15 acceptance | E08 and this draft | T01–T34 | B13,B14 |

In the ledger only, Tnn and Bnn abbreviate `A1C-Tnn` and `A1C-Bnn`.
Their complete definitions are sections 15 and 17; no external task tracker is
needed to understand a requirement's state.

## 17. Exact blocker and integration ledger

All items are **OPEN** with the narrowed residuals below. Ranked priority is for approval
readback, not an implementation sequence or authority to perform new research.
Dependencies name assigned interfaces for main to reconcile. Only main-routed
reviewed E13–E16 and earlier supporting-review inputs are integrated; no unfinished
normative author or pending execution/native/relic revision was polled or adopted.

| ID / rank | Missing closure, smallest acceptable resolution | Main-owned integration interface |
|---|---|---|
| A1C-B01 / 1 | Adopt evidence target/limits in D01; do not silently claim a verified retail build, historical JRE or original-runtime equivalence. Original gameplay remains unexecuted and is not a test pass requirement invented by this draft. | Whole-spec human fidelity decision; E06 preserved |
| A1C-B02 / 1 | E13 closes initialized underfill MODEL, rank criterion and conditional empty-marker counter+1/raw-state+0 failure. Still need initialized zero-Elite root witness/exclusion over declared domain with same-seed topology/bucket, and D02 native map-law/administrative initialization policy. Search minimum2 is not a minimum theorem. | Reviewed map input integrated; remaining fidelity/proof disposition main-owned, D02 |
| A1C-B03 / 1 | E14 closes conditional finite mapper/posterior/observable and linked-pool compositions. Choose input measure, joint/independence boundaries and ordered relic catalog; provide joint producer/context laws for unconditional boards/offers, any required actual root/profile/history ensemble evaluation and efficient exact native sampling contract. No IID source words or source-host hash order inferred. | Reviewed shuffle input integrated; D03 and B10 site registry, not a missing general kernel |
| A1C-B04 / 1 | Finite direct caller/override inventories and transitive legality/free/cost/copy/queue-extra plus draw/discard/exhaust/turn/death/prevention/heal callback records. Specify entered versus pending commit/cancel; close all matrix branches without universal FIFO/drain. Preserve §5.2 SI-I1/SI-I2 constraints in the corrected matrices and obtain exact-revision recheck; no pending author PR is clearance. | `execution` (`spire-gli.2.3.3`), shared CARD-* and D04 boundary |
| A1C-B05 / 1 | E16 closes the local parent/child, desktop Omamori prevention, removal/Bowl/Match/empty-size and handoff trace in §8–9. Remaining: global admitted/current/pending and Peek/terminal arbitration; legal-history witnesses/exclusions for unguarded mandatory empties, zero-Cage phase and stale/mistyped record conditions; initialized event-list exhaustion; exact successor-ready payload/cutoff. No phantom prevented Show curse remains an ordinary-path proof obligation. | Reviewed continuation input integrated; execution/native/content interfaces and D08 remain |
| A1C-B06 / 1 | All 141 relic primary/global consumer records with exact trigger/payload/state/counter/reset/order and acquire/unequip effects; thirteen no-override rows explicitly covered, not only 141/32 counts. | `relic-mechanics` (`spire-gli.2.3.5`) |
| A1C-B07 / 2 | Publish and integrate actual paired base/upgraded card values/targets/payload/state/copy/growth and all25 enemy payload/AI/setup/spawn records, not roster/HP anchors. Close fourteen CARD-* laws with exact per-route attribution or named per-ID fields. Seven assignments below are not results. Note bounded clearance and corrected consolidated SI-I1/SI-I2 wording do not factually clear old Silent or certify shared matrices. | Main supplies exact independently cleared normative/card/enemy and execution heads; no private normative imports or blanket E09 approval |
| A1C-B08 / 2 | Public numerical Neow categories/exclusions/quantities; all25 event eligibility/branch/payment/risk; question-room history odds; merchant slots/prices/discount rounding/restock/purge; combat/card/gold/chest/relic/potion drop/phase/Bark/transform and standard-start records. E16 closes specific local return/payment/Bowl/belt laws, not all those numerical records. Retained strong weights/source supports remain established at their bounds. | Main integrates reviewed events-Neow/economy-potions/enemy records below with remaining execution/relic/native interfaces |
| A1C-B09 / 2 | Per-field numeric bounds, actual gameplay cap/overflow and conversion audit; final packed representation covers all legal rows and meaningful ordered growth. int64 alone is not this proof. | `native-contract` (`spire-gli.2.3.6`), D07 |
| A1C-B10 / 2 | Complete concrete run/combat/continuation/site registry and RNG derivations/scopes; freeze row fields/widths/enums/masks/references/ordering; API/error/reset/replay/info/fault lease/carrier/facade contracts and new identities. Illustrative families/width-six candidate row are not a full schema. | `native-contract` (`spire-gli.2.3.6`), D07/D08 and other lanes' commit interfaces |
| A1C-B11 / 2 | E15 supplies explicit history facts and decision-ready lifecycle/preset/output/death/fault/reset/replay recommendation, now §9/9.1. Need D05/D06 human disposition, full legal stored-card/upgrade catalog and native schema/diagnostic integration; later-act discovery hooks remain bounded evidence. No automatic inheritance. Neither technical review nor Note bounded approval chooses native behavior. | Reviewed profile/time input integrated; content/native interfaces and whole-spec readback main-owned |
| A1C-B12 / 1 | E15/E16 close named timer dependencies/desktop bound; §9.2 gives concrete60Hz/exact-elapsed/dwell/wait P. Need D04 fidelity choice, complete mechanic-bearing timer/settlement/terminal registry and exact native Reaper/Match checkpoint outcomes under that chosen law. Natural original Reaper loss/frame distribution remain unproved; no source parity from model choice. | Reviewed timing inputs integrated; execution/native and D04 remain |
| A1C-B13 / 3 | T01–T34 remain UNEXECUTED. Written spec must first define precise expected laws; later separately approved implementation supplies actual gameplay/binding/fault/replay/legacy/generated-route evidence. Not executing product cases in this documentation wave is not a demand to implement before written-spec approval. Documentation checks never discharge later acceptance. | Future implementation acceptance only; coordinated plan/start authorization still separate |
| A1C-B14 / 1 | Prior fb7e9f8 review has zero findings only as BLOCKED-draft merge fitness, not user-approval readiness. This integrated head needs fresh independent contradiction/coverage/source-policy recheck; D01–D08 and accessible complete-spec human approval remain outstanding, then separate coordinated plan review/start. | Main; no ready/merge authority follows from this author update |

The ledger deliberately separates missing evidence (B02–B08), candidate contract
choices (B09–B12), unexecuted acceptance (B13), evidence target (B01) and review/
human gates (B14). Accepting a recommendation cannot silently count as proving a
source law. Conversely, lack of original-runtime validation does not erase bounded
static facts already obtained safely.

### 17.1 Pending normative-content interfaces — assignments, not evidence

These seven authors are running under main's assignments. Only their single public
document ownership and required gaps are recorded here; their unfinished facts,
private outputs or PRs have **not** been read or integrated. Filenames below are
descriptive future outputs, not claims that files exist in main or required hidden
normative imports. Main must route exact reviewed heads and reconcile shared laws.

| Assigned owner | Sole public document under docs/research/ | Required public record gap |
|---|---|---|
| spire-gli.2.3.18 / red-neutral | 2026-10-19-act1-content-red-colorless.md | Ironclad/colorless/status/curse/dependent special/removable event kinds: paired values, target/payload/upgrade/copy/growth/rounding and exact shared-law residuals. |
| spire-gli.2.3.19 / Silent | 2026-10-19-act1-content-silent.md | All legal green basics/nonbasics/Shiv, paired numeric/conditional rules; corrected Setup/Masterful Stab, discard/Poison/Retain/X/copy boundaries, no Silent start. |
| spire-gli.2.3.20 / Defect | 2026-10-19-act1-content-defect.md | Blue basics/nonbasics/special dependencies and orb/slot/Focus/Evoke/channel/resource laws; actual Ironclad generation context and growth/copy capture. |
| spire-gli.2.3.21 / Watcher | 2026-10-19-act1-content-watcher.md | Purple basics/nonbasics/options/specials; paired stance/Mantra/Retain/Scry/X/Expunger/Conjure/selection/copy and death residuals. |
| spire-gli.2.3.22 / enemies | 2026-10-19-act1-content-enemies.md | All25 A0 HP/setup/intent/AI-history/payload/spawn/split/escape and encounter/event/elite/boss distinctions; exact outstanding arbitration fields. |
| spire-gli.2.3.23 / events-Neow | 2026-10-19-act1-content-event-neow.md | Neow categories/drawback/exclusions/quantities, evolving question-room probabilities, all25 event numeric/eligibility/payment/risk/source records. |
| spire-gli.2.3.24 / economy-potions | 2026-10-19-act1-content-economy-potions.md | Merchant stock/price/rounding/retry/restock/purge; reward/chest/drop laws; all33 eligible potion base/Bark/phase/target/use/duration/Sozu/belt/commit records. |

These assignments fill B07/B08 only when actual reviewed records replace missing
fields. Inventories, a green document checker, parallel progress or an unreviewed
claim of completeness cannot substitute. Pending native/relic/execution corrections
likewise remain interfaces, not incorporated findings from those author lanes.

## Appendix A. Finite reachable-card inventory and closure boundaries

These names are identity labels, not localized substring tests or assigned native
enum values. Aliases need explicit catalog mapping. E09 inventories and E10 pool
findings support these groups; **all primary/execution records still require
A1C-REQ-011 through A1C-REQ-015**, with A1C-B07 preserving missing paired detail.
This is a complete-scope draft inventory, not a declaration of a complete rules
catalog. Source/type filters in section 6 determine which subset a site can sample.

| Group | Required identities |
|---|---|
| Ironclad basic (3) | Bash; Defend; Strike |
| Ironclad common (20) | Anger; Armaments; Body Slam; Clash; Cleave; Clothesline; Flex; Havoc; Headbutt; Heavy Blade; Iron Wave; Perfected Strike; Pommel Strike; Shrug It Off; Sword Boomerang; Thunderclap; True Grit; Twin Strike; Warcry; Wild Strike |
| Ironclad uncommon (36) | Battle Trance; Blood for Blood; Bloodletting; Burning Pact; Carnage; Combust; Dark Embrace; Disarm; Dropkick; Dual Wield; Entrench; Evolve; Feel No Pain; Fire Breathing; Flame Barrier; Ghostly Armor; Hemokinesis; Infernal Blade; Inflame; Intimidate; Metallicize; Power Through; Pummel; Rage; Rampage; Reckless Charge; Rupture; Searing Blow; Second Wind; Seeing Red; Sentinel; Sever Soul; Shockwave; Spot Weakness; Uppercut; Whirlwind |
| Ironclad rare (16) | Barricade; Berserk; Bludgeon; Brutality; Corruption; Demon Form; Double Tap; Exhume; Feed; Fiend Fire; Immolate; Impervious; Juggernaut; Limit Break; Offering; Reaper |
| Silent common (19) | Acrobatics; Backflip; Bane; Blade Dance; Cloak and Dagger; Dagger Spray; Dagger Throw; Deadly Poison; Deflect; Dodge and Roll; Flying Knee; Outmaneuver; Piercing Wail; Poisoned Stab; Prepared; Quick Slash; Slice; Sneaky Strike; Sucker Punch |
| Silent uncommon (33) | Accuracy; All-Out Attack; Backstab; Blur; Bouncing Flask; Calculated Gamble; Caltrops; Catalyst; Choke; Concentrate; Crippling Cloud; Dash; Distraction; Endless Agony; Escape Plan; Eviscerate; Expertise; Finisher; Flechettes; Footwork; Heel Hook; Infinite Blades; Leg Sweep; Masterful Stab; Noxious Fumes; Predator; Reflex; Riddle with Holes; Setup; Skewer; Tactician; Terror; Well-Laid Plans |
| Silent rare (19) | A Thousand Cuts; Adrenaline; After Image; Alchemize; Bullet Time; Burst; Corpse Explosion; Die Die Die; Doppelganger; Envenom; Glass Knife; Grand Finale; Malaise; Nightmare; Phantasmal Killer; Storm of Steel; Tools of the Trade; Unload; Wraith Form |
| Defect common (18) | Ball Lightning; Barrage; Beam Cell; Charge Battery; Claw; Cold Snap; Compile Driver; Coolheaded; Go for the Eyes; Hologram; Leap; Rebound; Recursion; Stack; Steam Barrier; Streamline; Sweeping Beam; TURBO |
| Defect uncommon (36) | Aggregate; Auto-Shields; Blizzard; Boot Sequence; Bullseye; Capacitor; Chaos; Chill; Consume; Darkness; Defragment; Doom and Gloom; Double Energy; Equilibrium; FTL; Force Field; Fusion; Genetic Algorithm; Glacier; Heatsinks; Hello World; Loop; Melter; Overclock; Recycle; Reinforced Body; Reprogram; Rip and Tear; Scrape; Self Repair; Skim; Static Discharge; Storm; Sunder; Tempest; White Noise |
| Defect rare (17) | All for One; Amplify; Biased Cognition; Buffer; Core Surge; Creative AI; Echo Form; Electrodynamics; Fission; Hyperbeam; Machine Learning; Meteor Strike; Multi-Cast; Rainbow; Reboot; Seek; Thunder Strike |
| Watcher common (19) | Bowling Bash; Consecrate; Crescendo; Crush Joints; Cut Through Fate; Empty Body; Empty Fist; Evaluate; Flurry of Blows; Flying Sleeves; Follow-Up; Halt; Just Lucky; Pressure Points; Prostrate; Protect; Sash Whip; Third Eye; Tranquility |
| Watcher uncommon (35) | Battle Hymn; Carve Reality; Collect; Conclude; Deceive Reality; Empty Mind; Fasting; Fear No Evil; Foreign Influence; Foresight; Indignation; Inner Peace; Like Water; Meditate; Mental Fortress; Nirvana; Perseverance; Pray; Reach Heaven; Rushdown; Sanctity; Sands of Time; Signature Move; Simmering Fury; Study; Swivel; Talk to the Hand; Tantrum; Wallop; Wave of the Hand; Weave; Wheel Kick; Windmill Strike; Worship; Wreath of Flame |
| Watcher rare (17) | Alpha; Blasphemy; Brilliance; Conjure Blade; Deus Ex Machina; Deva Form; Devotion; Establishment; Judgment; Lesson Learned; Master Reality; Omniscience; Ragnarok; Scrawl; Spirit Shield; Vault; Wish |
| Colorless uncommon (20) | Bandage Up; Blind; Dark Shackles; Deep Breath; Discovery; Dramatic Entrance; Enlightenment; Finesse; Flash of Steel; Forethought; Good Instincts; Impatience; Jack of All Trades; Madness; Mind Blast; Panacea; Panic Button; Purity; Swift Strike; Trip |
| Colorless rare (15) | Apotheosis; Chrysalis; Hand of Greed; Magnetism; Master of Strategy; Mayhem; Metamorphosis; Panache; Sadistic Nature; Secret Technique; Secret Weapon; The Bomb; Thinking Ahead; Transmutation; Violence |
| Note extra basics (12) | Silent Strike; Silent Defend; Neutralize; Survivor; Defect Strike; Defect Defend; Zap; Dualcast; Watcher Strike; Watcher Defend; Eruption; Vigilance |
| Note removable event kinds (4) | Apparition; Bite; J.A.X.; Ritual Dagger |
| Dependent physical specials (9) | Shiv; Miracle; Smite; Safety; Insight; Through Violence; Beta; Omega; Expunger |
| Status kinds (5) | Burn (including enemy-created Burn+); Dazed; Slimed; Void; Wound |
| Ordinary curse candidates (10) | Clumsy; Decay; Doubt; Injury; Normality; Pain; Parasite; Regret; Shame; Writhe |
| Separate non-removable curse (1) | Curse of the Bell |
| Strategic Wish representations (3) | Plated Armor choice; Strength choice; gold choice — option effects, not three permanently acquired physical cards |

This reconciliation counts 367 identities/option representations, not 367 ordinary
reward cards. Shard support320 is 72+71+71+71+35 and excludes basics/SPECIAL/status/
curse. Defect/Silent/Watcher own basics are distinct despite repeated printed names.
Base/upgraded card variants are state on these identities, not additional pool IDs.

Required special primary anchors (public-description unless section 6 supplies
more): Shiv cost0 Attack 4/6 Exhaust; Miracle cost0 Retain energy1/2 Exhaust;
Smite cost1 Retain Attack12/16 Exhaust; Safety cost1 Retain Block12/16 Exhaust;
Insight cost0 Retain draw2/3 Exhaust; Through Violence cost0 Retain Attack20/30
Exhaust; Beta cost2/1 creates Omega and Exhausts; Omega cost3 Power end-turn50/60
area damage; Expunger cost1 Attack9/15 per stored hit. Source-upgraded generation
does not universally upgrade children: Blade Dance+ gives more base Shivs, Storm
of Steel+ gives Shiv+, Collect creates Miracle+, Alpha+ adds Innate rather than
Beta+, and Conjure's stored hit count is independent of child damage upgrade.

Status/curse anchors: Burn is turn-end damage2/4, Decay damage2; Pain/Regret are
HP loss and require separate snapshots; Dazed/Clumsy/Void are Ethereal; Void loses
one energy on actual draw; Slimed cost1 Exhausts without Skill identity; Wound/
Injury have no extra payload; Doubt/Shame apply Weak/Frail at cleanup; Writhe is
Innate; Normality's three-play restriction depends on its current presence in hand.
No universal Unplayable sentinel bypass is inferred for autoplay/Medical Kit/Candle.

## Appendix B. Enemy identity/HP inventory

Uniform inclusive integer HP anchors from E11 apply before actual setup modifiers;
fixed HP does not imply the original omitted an RNG helper call. These 25 kinds
include spawned children/formation members, not 25 distinct encounters.

| Kind(s) | A0 HP anchor | Required distinction |
|---|---|---|
| Cultist; Jaw Worm | 48–54; 40–44 | Initial Ritual suppression/attack cycle; conditional redirects and history |
| Red Louse; Green Louse | 10–15; 11–17 | Separate Bite5–7 and prebattle Curl Up3–7 rolls; green Weak versus red Strength |
| Acid Slime S/M/L | 8–12; 28–32; 65–69 | Small initial fair choice, medium/large redirects/status/split |
| Spike Slime S/M/L | 10–14; 28–32; 64–70 | Small attack-only; medium/large Lick/status/split |
| Blue Slaver; Red Slaver | 46–50; 46–50 | Rake/Stab versus executed-once Entangle history |
| Looter; Fungi Beast | 44–48; 22–28 | Ledger/theft/escape/refund versus Grow/death Spore Cloud |
| Fat/Mad/Shield/Sneaky/Wizard Gremlins | 13–17 / 20–24 / 12–15 / 10–14 / 21–25 | Weak, damage reaction, live ally shield target, attack, charge; ordered Gang instances |
| Gremlin Nob; Lagavulin; Sentry | 82–86; 109–111; 38–42 | Enrage/attack history; sleep/event/Siphon; alternating ordered Dazed attacks |
| Guardian; Hexaghost; Slime Boss | 240; 250; 140 | Queued mode quota; Divider/Inferno; staged split and offspring |

Burning/Preserved Insect/event setup and legal other effects modify these initial
anchors. Do not apply an ordinary constructor HP roll to split children. A named
payload not numerically fixed here remains part of B07, not a zero-damage default.

## Appendix C. Event branch closure inventory

These 11 ordinary, six recurring-shrine and eight applicable one-time families
are the required finite set. E16's closed local continuation/empty conditions below
do not supply all missing numeric/generation records in B08. They are not executable
event algorithms or evidence that every branch supports initial Leave. Unguarded
mandatory-empty local conditions need initialized witness/exclusion, not a fallback.

| Event | Required branch/continuation family |
|---|---|
| Big Fish | Heal/maxHP/relic with Regret; no initial Leave. Curse and spawned relic resolve separately; Continue is not effect drain. |
| Dead Adventurer | Leave/search/repeated risk; pre-shuffled categories and interrupted event elite. Successful tokens removed from fight rewards; reward Proceed direct-map, not elite-room keys/queue law. |
| Golden Idol | Take/decline, then mandatory injury/HP/maxHP choice; relic scheduled before second choice, no new free Leave there. |
| Hypnotizing Colored Mushrooms | Fight/eat, no initial Leave; eat heals before queued Parasite. Event fight rewards remain skippable/direct-map. |
| Living Wall | Remove/transform/upgrade; upgrade initially disabled without upgradeables. Each route also gates grid on nonbottled purgeable count, including Grow; zero goes to result without selection mutation. No invented initial Leave. |
| Scrap Ooze | Repeated HP cost and rising relic chance, stop/take; initial nominal25 accepts 26 of100 integers |
| Shining Light | Enter disabled with no upgradeables; Leave remains. Damage precedes up-to-two exact instance prefix; singleton upgrades once. |
| The Cleric | Gold-eligible heal/remove/Leave. Remove with zero nonbottled purgeables opens no grid and spends no removal gold, then Proceed; nonempty committed branch pays before grid. |
| The Ssssserpent | Agree confirmation before gold/Doubt scheduling, or Leave; never Ssserpent Head. |
| Wing Statue | Damage precedes later mandatory purge with no local empty guard; separate gold branch has permanent base-damage deck gate, initial Leave exists. No post-damage invented Leave. |
| World of Goop | Damage before recovered gold; alternative loses precomputed capped gold, possibly0. No card target required. |
| Golden Shrine | Ordinary gold versus larger gold with Regret, or leave |
| Match and Keep | Twelve instances/six pairs, five attempts, reveal/match/miss/final cleanup/obtain; memory projection |
| Purifier | Initial remove/Leave; committed noncancellable grid has no local empty guard. No automatic post-commit Skip. |
| Transmogrifier | Initial transform/Leave; committed mandatory grid has no empty guard. Selected object removal precedes generation/Show obtain. |
| Upgrade Shrine | Upgrade disabled without upgradeables, Leave remains; selected permanent upgrade refreshes bottle link. |
| Wheel of Change | Gold at reveal once; other later callbacks. Purge empty subset opens no grid/replacement; relic outcome offers skippable rewards, not automatic obtain. |
| Ominous Forge | Upgrade disabled without upgradeables; rummage/Leave remain. Warped Tongs and queued Pain are distinct work. |
| Bonfire Spirits | No nonbottled purgeables: Choose→Complete without reward. Otherwise rarity reward (curse Spirit Poop/Circlet) precedes object removal/hooks; no arbitrary sacrifice or initial Leave. |
| Face Trader | Paid HP/gold branch, missing-face trade or decline; missing set shuffle/Circlet fallback |
| The Divine Fountain | Cleanse/Leave; actual curse objects retain bottle/unremovable exclusions. Empty removable subset supplies no replacement. |
| Lab | Potion offers despite full belt, automatic card suppressed, claims can fail, Proceed direct-map. |
| A Note For Yourself | Take/Ignore after reveal. Direct incoming hooks/insertion before outgoing purgeable/bottle filter; valid removable incoming supplies a candidate, otherwise no local empty guard. Ignore creates no outgoing write; departure persistence separate from selection. |
| We Meet Again! | Missing eligible potion/gold/nonbasic-noncurse card disables that sacrifice; Leave remains. One actual sacrifice precedes one spawned relic, not a three-relic menu. |
| The Woman in Blue | Paid quantity or A0 Leave; ordinary unlabelled CombatRewardScreen.open()/Proceed returns to event/result, not Lab direct-map or labelled open(String)/Cancel. Sozu/belt claim distinctions remain. |

Act 2/3-only home events are not additional Act 1 candidates. Secret Portal's
clock dependency is carried for future entry, not inserted into this table.

## Appendix D. Relic and potion scope reconciliation

The full per-ID tier/constructor/hook roster is public E04 evidence; this document
does not depend on a private catalog. The following restates the required candidate
names so readers need not infer scope from counts.

| Relic group | Candidates |
|---|---|
| Starter (1) | Burning Blood |
| Common (33) | Akabeko; Anchor; Ancient Tea Set; Art of War; Bag of Marbles; Bag of Preparation; Blood Vial; Bronze Scales; Centennial Puzzle; Ceramic Fish; Dream Catcher; Happy Flower; Juzu Bracelet; Lantern; Maw Bank; Meal Ticket; Nunchaku; Oddly Smooth Stone; Omamori; Orichalcum; Pen Nib; Potion Belt; Preserved Insect; Red Skull; Regal Pillow; Smiling Mask; Strawberry; The Boot; Tiny Chest; Toy Ornithopter; Vajra; War Paint; Whetstone |
| Uncommon (30) | Blue Candle; Bottled Flame; Bottled Lightning; Bottled Tornado; Darkstone Periapt; Eternal Feather; Frozen Egg; Gremlin Horn; Horn Cleat; Ink Bottle; Kunai; Letter Opener; Matryoshka; Meat on the Bone; Mercury Hourglass; Molten Egg; Mummified Hand; Ornamental Fan; Pantograph; Paper Phrog; Pear; Question Card; Self-Forming Clay; Shuriken; Singing Bowl; Strike Dummy; Sundial; The Courier; Toxic Egg; White Beast Statue |
| Rare (28) | Bird-Faced Urn; Calipers; Captain's Wheel; Champion Belt; Charon's Ashes; Dead Branch; Du-Vu Doll; Fossilized Helix; Gambling Chip; Ginger; Girya; Ice Cream; Incense Burner; Lizard Tail; Magic Flower; Mango; Old Coin; Peace Pipe; Pocketwatch; Prayer Wheel; Shovel; Stone Calendar; Thread and Needle; Torii; Tungsten Rod; Turnip; Unceasing Top; Wing Boots |
| Boss (22) | Astrolabe; Black Blood; Black Star; Busted Crown; Calling Bell; Coffee Dripper; Cursed Key; Ectoplasm; Empty Cage; Fusion Hammer; Mark of Pain; Pandora's Box; Philosopher's Stone; Runic Cube; Runic Dome; Runic Pyramid; Sacred Bark; Slaver's Collar; Snecko Eye; Sozu; Tiny House; Velvet Choker |
| Shop (17) | Brimstone; Cauldron; Chemical X; Clockwork Souvenir; Dolly's Mirror; Frozen Eye; Hand Drill; Lee's Waffle; Medical Kit; Membership Card; Orange Pellets; Orrery; Prismatic Shard; Sling of Courage; Strange Spoon; The Abacus; Toolbox |
| Source-bound special (10) | Neow's Lament; Golden Idol; Odd Mushroom; Warped Tongs; Spirit Poop; Face of Cleric; Cultist Headpiece; Gremlin Visage; N'loth's Hungry Face; Ssserpent Head |

Thirteen no-primary-override rows still require shared/global/site consumers:
Paper Frog (display Paper Phrog), White Beast Statue, Calipers, Ginger, Ice Cream,
Turnip, Runic Pyramid, Chemical X, Frozen Eye, Strange Spoon, Golden Idol, Odd
Mushroom, Spirit Poop. Relic IDs with aliases are not extra kinds: Frozen/Molten/
Toxic Egg use `... Egg 2`, Wing Boots uses `WingedGreaves`, Neow's Lament uses
`NeowsBlessing`, Abacus uses `TheAbacus`. E04 gives all exact identity mappings.
Thirty-two spawn predicates include floor limits, four shop exclusions, bottle
type/deck predicates, the Girya/Pipe/Shovel combination cap, Black Blood's Burning
Blood requirement and Ectoplasm Act≤1. Screenless and noncampfire selectors add
their own exclusions; canSpawn alone is not source eligibility.

| Potion tier | Required candidates |
|---|---|
| Common (17) | Attack Potion; Blessing of the Forge; Block Potion; Blood Potion; Colorless Potion; Dexterity Potion; Energy Potion; Explosive Potion; Fear Potion; Fire Potion; Power Potion; Skill Potion; Speed Potion; Flex Potion; Strength Potion; Swift Potion; Weak Potion |
| Uncommon (9) | Ancient Potion; Distilled Chaos; Duplication Potion; Elixir; Essence of Steel; Gambler's Brew; Liquid Bronze; Liquid Memories; Regen Potion |
| Rare (7) | Cultist Potion; Entropic Brew; Fairy in a Bottle; Fruit Juice; Heart of Iron; Smoke Bomb; Snecko Oil |

Each needs phase/target, potency/Bark, consumption/use callbacks, selections,
automatic trigger, resource commit and failure/cancellation record. Nine other-
character exclusives stay excluded from Ironclad potion generation: Poison/Cunning/
Ghost in a Jar, Focus/Potion of Capacity/Essence of Darkness, Bottled Miracle/
Stance/Ambrosia. Off-color Alchemize does not change this boundary.

## 18. Draft handoff status

This integration draft supplies whole-scope requirements and the reviewed E13–E16
advances, not closed mechanics/API evidence or a human-approval-ready rulebook.
The former phantom ordinary Show-curse obligation is superseded; complete normative
content/numeric/native/relic/execution integration is still required. B01–B14 remain
open at their narrowed boundaries; D01–D08 are recommendations or
choices for main's readback; T01–T34 are unexecuted. The author may correct this
document only through bounded main-directed follow-ups. Independent review,
human whole-spec approval, coordinated implementation-plan review and separate
start authorization remain distinct. No PR readiness, merge or parent-task closure
is implied by publishing this draft.
