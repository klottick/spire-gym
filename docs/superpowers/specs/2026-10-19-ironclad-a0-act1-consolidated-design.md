# Ironclad A0 Act 1 — consolidated specification

Date: 2026-10-19. Author task: `spire-gli.2.3.38`.

**Whole-scope integration draft — BLOCKED, not approved or implementation-ready.**
This is the single review surface for standard Ironclad A0 Act 1. It preserves
the approved architecture, states candidate detailed contracts, and accounts for
required content even where a law is unresolved. It is not an implementation plan,
an implementation claim, or permission to install, build, benchmark, or train.
Main owns integration of independently reviewed follow-ups and the eventual human
presentation. This revision preserves the fourteen previously integrated main-
authorized follow-ups and earlier Note/Silent corrections, and substantively
integrates accepted PR24 (E27) and corrected PR23 (E28). Technical clearance of a
source claim or proposal is not human approval of its behavior.

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

#### Newly integrated reviewed inputs (E27–E28)

The following exact author revisions were independently cleared for bounded research
claims and normally merged into canonical main at this integration base. They provide
static/bytecode evidence, not original execution or whole-spec closure.

| ID | Accessible input / exact reviewed author head | Merge identity | Integrated contribution and limit |
|---|---|---|---|
| E27 | [Initialized RNG and map closure](../../research/2026-10-19-act1-initialized-rng-map-closure.md) · [reviewed revision](https://github.com/klottick/spire-gym/blob/8f13d5231bfcc119aec871a5b0a726a91c008454/docs/research/2026-10-19-act1-initialized-rng-map-closure.md) · `8f13d5231bfcc119aec871a5b0a726a91c008454` | PR24 merge `dc1759cdab02bb653ebda9887c2709e2c3459f11` | Same-root twelve-stream initialization, Act1 fresh `s+1` map stream, five consecutive relic-source seed calls, and linked Neow/card/curse/potion/event cursor transitions. Conditional fixed-input wiring only; no root/history prior, evaluated complete joints, target host/JRE catalog order, or initialized zero-Elite witness/exclusion. |
| E28 | [Transitive execution closure](../../research/2026-10-19-act1-transitive-execution-closure.md) · [reviewed revision](https://github.com/klottick/spire-gym/blob/4167152c5d0fce47586fe83f3d1cfc1f976af609/docs/research/2026-10-19-act1-transitive-execution-closure.md) · `4167152c5d0fce47586fe83f3d1cfc1f976af609` | PR23 merge `b637b580a920942e392eab44c35c2a9869219874` | Shared manager/power/card-play checkpoints, player-vs-monster damage and rescue/death/escape paths, source-specific nested commits/returns, and conditional Reaper/Match timer facts, including corrected marker-vs-discard callbacks and Entrench. Static bounded transitions only; no exhaustive reachable composition, initialized exceptional-history proof, natural timer outcome, or native clock choice. |

The independent exact-head reviews found PR24 clear for its bounded scope and PR23
clear only after its Entrench/end-turn callback correction. PR24 author model checks
passed 4/4, with retained map and shuffle suites at 7/7 and 9/9; its independent
reviewer reran all three suites successfully. These checks validate helper/model
arithmetic only. PR23 author performed read-only static/bytecode reconciliation
without a new JVM invocation; the independent review/recheck spot-checked exact
source transitions. TX01–TX18 remain unexecuted future cases. The accepted heads
above are the only authorities; earlier PR23 head
`7b8a05c9f2bbd0ffed67657a03bad2e36614c309` was not merge-fit. Private author/reviewer
ledgers and controller merge receipts retain detailed commands and checks.

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
normal two-parent merges. E13–E19 match canonical main
`8a2148d99da114703978418a2698c4086d070fb0`; E20 matches canonical main
`a8db45e57b14ebd50f98b3210b93eb6a89de7b19`. Immutable public links intentionally
avoid requiring new files in this author's older-base worktree.

| ID | Cleared public input / exact author head | Integrated contribution and limit |
|---|---|---|
| E13 | [Initialized map/empty marker](https://github.com/klottick/spire-gym/blob/db15eb93eeb788b7683151b4bc5ebd10ac37c268/docs/research/2026-10-19-act1-map-seed.md) | Initialized underfill model, zero-Elite rank criterion and exact conditional counter/state failure branch; no initialized zero-Elite existence or native policy approval. |
| E14 | [Finite shuffle laws](https://github.com/klottick/spire-gym/blob/815a372a88447d38bbb5e9efcf0abb5530c279e0/docs/research/2026-10-19-act1-shuffle-laws.md) | Complete conditional finite mapper, weighted posterior and linked-pool composition; input measure, independence boundaries and catalog order remain choices, actual full ensemble unevaluated. |
| E15 | [Profile/time decision contract](https://github.com/klottick/spire-gym/blob/55f07f7382d9857c6fe3930cb5c6c200afd6ebe9/docs/research/2026-10-19-act1-profile-time.md) | Source history/Note/timer facts and concrete staged/committed, preset and 60-Hz/exact-elapsed/wait recommendations; all new native choices remain P. |
| E16 | [Source-specific continuations](https://github.com/klottick/spire-gym/blob/2cbcd2d325f0e4d4f2294612cbfff131919e2288/docs/research/2026-10-19-act1-continuations.md) | Closed local parent/child, desktop prevention, Bowl, Match, empty-selection and transition contracts; not global terminal arbitration, exceptional reachability or final handoff-schema approval. |
| E17 | [Concrete native candidate](https://github.com/klottick/spire-gym/blob/b18b43c5b11ca9cd1533b7eb016ff60d6bbb2c3a/docs/research/2026-10-19-act1-native-contract.md) | R1 operation-specific reset preflight and R2 exact byte/digest framing corrected; packed layouts, ownership and sites remain candidate contracts. PR-body provenance Minor dispositioned before merge. NC04/NC10–NC12 and human choices remain open. |
| E18 | [Relic primary/shared rules](https://github.com/klottick/spire-gym/blob/e95fd2e2ac269f79d2ca519773b5bfe3a51ca70e/docs/research/2026-10-19-act1-relic-mechanics.md) | All141 local primary laws,13 shared consumers and10 current Chemical X actions; corrected Red Skull/Tiny House and final generated-X qualification. Local coverage is not transitive closure. |
| E19 | [Execution direct families](https://github.com/klottick/spire-gym/blob/f515954193bac8d36eb8e425051ba5940c5a4000/docs/research/2026-10-19-act1-execution.md) | Complete direct classfile families plus bounded legality/cost/copy/origin/lifecycle/rescue facts; corrected Transmutation self-output and Eviscerate source receiver. No global mixed-callback/cost/terminal/timer closure. |
| E20 | [Enemy normative records](https://github.com/klottick/spire-gym/blob/e56d4ace59219d07d01dd69f117c9534f9fbe941/docs/research/2026-10-19-act1-content-enemies.md) | All25 A0 identity/HP/setup/move/payload and conditional AI records, reaction/split/boss/theft checkpoints, factories/event variants, burning/key and nine fixed Status sites; Strength/Block helper bounds corrected. EN-U1–U6 remain. |

E17–E19 and E20 match the canonical heads stated above after normal two-parent
merges. Only final corrected heads above are authority; older
relic-review unqualified generated-turn-zero wording is superseded by E18's
explicit negative-X exception, consistent with E19, not a second competing law.

#### Final normative-content integration cutoff — canonical main `9645d1758833c2b99c6dc34320307c98475767fe`

The following seven public documents are integrated from their exact cleared
author heads and ordinary main merges. Relative links resolve against canonical
main; immutable links pin each reviewed author revision. These records establish
local requirements within their stated evidence limits, not complete transitive
closure, gameplay execution, or human approval.

| ID | Accessible document / exact reviewed head | Local coverage integrated here; residual limit |
|---|---|---|
| E20 | [Enemy records](../../research/2026-10-19-act1-content-enemies.md) · [reviewed revision](https://github.com/klottick/spire-gym/blob/e56d4ace59219d07d01dd69f117c9534f9fbe941/docs/research/2026-10-19-act1-content-enemies.md) · `e56d4ace59219d07d01dd69f117c9534f9fbe941` | All25 local A0 enemy records and bounded setup/reaction/factory/key/status facts, as integrated in §7/Appendix F; not global scheduling or root-conditioned RNG closure. |
| E21 | [Silent paired records](../../research/2026-10-19-act1-content-silent.md) · [reviewed revision](https://github.com/klottick/spire-gym/blob/60f97e2bbc8b4c2b35622f039bd972685fd73142/docs/research/2026-10-19-act1-content-silent.md) · `60f97e2bbc8b4c2b35622f039bd972685fd73142` | 4 basics, 71 nonbasics and dependent Shiv with paired costs/targets/payloads plus named local callbacks; no Silent run/start, and no global target/callback/terminal closure. |
| E22 | [Defect paired records](../../research/2026-10-19-act1-content-defect.md) · [reviewed revision](https://github.com/klottick/spire-gym/blob/40c1a2f93fb65bfa0b19d1ca1618edbdb3ccd476/docs/research/2026-10-19-act1-content-defect.md) · `40c1a2f93fb65bfa0b19d1ca1618edbdb3ccd476` | 4 basics, 71 nonbasics, two Status dependencies and four orb kinds with paired local mechanics, including Lock-On lifecycle; not full orb/callback/terminal composition. |
| E23 | [Watcher paired records](../../research/2026-10-19-act1-content-watcher.md) · [reviewed revision](https://github.com/klottick/spire-gym/blob/1f3abc31306410a0a4ea48ab73bea1351a40fd21/docs/research/2026-10-19-act1-content-watcher.md) · `1f3abc31306410a0a4ea48ab73bea1351a40fd21` | 4 basics, 71 nonbasics, eight physical dependencies and three nonphysical Wish options; queue provenance is distinct from manager-derived play fields; not global queue/terminal closure. |
| E24 | [Red/colorless/dependent records](../../research/2026-10-19-act1-content-red-colorless.md) · [reviewed revision](https://github.com/klottick/spire-gym/blob/6ec9659534b19ff57cd9699bfa9fb09793e977d0/docs/research/2026-10-19-act1-content-red-colorless.md) · `6ec9659534b19ff57cd9699bfa9fb09793e977d0` | 75 red, 35 ordinary colorless, five Status, eleven Curse, four stored-event and twelve dependent/control records; local Feed/heal, Mayhem, Flame Barrier, Exhume and Entrench cases do not establish universal callbacks. |
| E25 | [Neow/events/question-room records](../../research/2026-10-19-act1-content-event-neow.md) · [reviewed revision](https://github.com/klottick/spire-gym/blob/f880dc8c02e2a208860802b0ddb594acd942ce7e/docs/research/2026-10-19-act1-content-event-neow.md) · `f880dc8c02e2a208860802b0ddb594acd942ce7e` | Numeric/conditional records for Neow, evolving question-room law, all25 events, choices, payments and return paths; initialized/history-conditioned joint measure and exceptional legal reachability remain open. |
| E26 | [Economy/reward/chest/potion records](../../research/2026-10-19-act1-content-economy-potions.md) · [reviewed revision](https://github.com/klottick/spire-gym/blob/637723b1bf8157a20186f5c4c221bf7e25cc35a1/docs/research/2026-10-19-act1-content-economy-potions.md) · `637723b1bf8157a20186f5c4c221bf7e25cc35a1` | Numeric card/relic/potion/reward/chest/merchant/Dig records and all33 potion local effects/guards; seeded-shuffle joint measure, transitive potion actions and global commit/scheduler composition remain open. |

Private controller merge receipts bind E21–E26 respectively to normal merge commits
`faba306317a36e17aaded8c98d79ef75c9ed2ec8`,
`848aaec89e3b3d5638ba6f9c65904341a75f2650`,
`63b1dcf43e1b56334555dee47d545e5448711268`,
`9645d1758833c2b99c6dc34320307c98475767fe`,
`ae7750c38e2084abdd5c7f3b772e33e0d15c4e6b`, and
`e2d5e2db19b434898166badfdebe1680e09595ae`. No sibling working-tree content is
used as authority.

The independent whole-draft review at prior head
`fb7e9f8ff3d9665e3120f0b772623d5125146576` found zero findings **as an accurate
BLOCKED draft**, explicitly not a complete human-approval-ready mechanics/API
specification. It does not clear this revised head. PR16 enemy records and PR17–22
normative tables are integrated above only within their exact independently cleared
scope. They address the paired-primary and numeric-content subgaps, but do not close
the remaining transitive, source-proof, initialized-measure or human-choice gates.

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
| A1C-D07 | Recommend E17's concrete ownership/finite registry,18-table widths/masks/local refs, int64 candidate carrier, separate identities, reset/delivery/fault and exact replay-byte contracts (§10.3/§11–13). Keep packed-first/candidate-aware facade. | Fixed small action heads lose choices; dense carrier masks are infeasible. Different layouts/framing require versioned descriptors, not implicit combat-v1 replacement. Concrete representation still needs complete semantic catalogs/property/numeric/source bindings. | Accept/revise these P representations separately from fidelity D03–D06; reconcile time fields/wait opcode/profile attachments/private-core encoding, B09/B10/B11. No final fingerprint exists. |
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
| Omniscience and selected playback | Draw-pile original queues with exhaust; stat-equivalent autoplay copies purge and have distinct UUIDs, unlike five repeats' same-UUID linkage. | Mixed-repeat/callback law; queueExtraCard has zero direct invocations in E19's archive inventory, not a universal producer; A1C-B04 |
| Generated/copy/upgrade/draw/movement | Each has its own creation/history/cost/hook policy. Movement into hand is not automatically draw or obtain. | Full stat-equivalent-copy overrides and cost precedence; A1C-B04/A1C-B07 |

X is not fixed-cost zero or unplayable. Turn-zero setters do not erase negative X
cost semantics. Chemical X is source-sensitive; Transmutation's capture/payment
and Conjure Blade's child payload must not be inferred from Whirlwind alone.
Descriptions of generated-card discounts do not declare every generated X card
numeric-zero or free. E18/E19 establish the ten action-level distinctions below.
Corruption, Confusion/Snecko Oil, until-played discounts, base/combat cost changes,
upgrades and free plays need an explicit precedence matrix. A missing matrix is
not permission to apply effects in the order convenient for an implementation.

### 4.1.1 Closed direct gates, cost helpers and copy receivers (F; E19)

The base legality gate checks Status/Curse exceptions (Kit/Candle), supplied
target/group and then affordability permissions: turn-ended, powers, Entangled
Attack, relics, blights and hand cards. Only the final energy comparison admits
sufficient energy, free play **or autoplay**. Autoplay is not an earlier permission
bypass. NoSkills is the only nonbase power canPlayCard; Normality is the card
permission (three recorded plays), Choker the relic permission (six), and TimeMaze
the separate blight family. Later-act archive declarations do not create Act1 routes.

All eight card canUse overrides are accounted for: Secret Technique/Weapon add
draw-pile Skill/Attack availability, Grand Finale requires empty draw, Signature
Move no other hand Attack object, Clash all hand Attacks; these five also require
base acceptance. Reflex, Tactician and Deus reject. The base's supplied dying-target
test is not a universal nonnull check. Earlier rejection records no normal history;
an accepted manager entry records onPlay/history **before** later invalid-target
handling can omit player use/payment. Player use recalculates damage, adjusts X,
enters card use, constructs immediate onUse callbacks, notifies other cards,
removes from hand, then pays ordinary energy. Pending payload is not resolved yet.

| Mutation/copy interface | Required state/receiver law, not global precedence |
|---|---|
| setCostForTurn | Existing turn cost must be nonnegative; negative ordinary request clamps0. Negative X/unplayable sentinels are not rewritten. |
| updateCost / modifyCostForCombat / upgradeBaseCost | Preserve each helper's combat/turn difference and positive-versus-nonpositive branch. Ordinary Curse/Status exclusion has Pride/Slimed exceptions; no context-free total order with other writers. |
| resetAttributes | Restore turn from mutable combat cost, not printed cost; clear turn modifier, not all flags. |
| Confusion versus randomize-hand | Eligible nonnegative sampled0–3 equal to combat cost leaves differing turn cost alone; Confusion still clears one-shot free flag, hand randomization does not. |
| Enlightenment / Madness / Corruption | Enlightenment's >1 turn/combat branches differ. Madness prefers positive turn cost, otherwise positive combat cost, and zeroes both. Corruption's draw setter, refresh/application combat modifier, exhaust mark and Skill payment exemption are separate actual calls. |
| Eviscerate makeCopy | Construct fresh cost3 result; under dungeon/map/combat guard, turn setter mutates **SOURCE** using source combat cost minus totalDiscardedThisTurn, not returned card or combat-total history. Observe both receivers. |
| Blood for Blood makeCopy | With a player, history-aware cost update by −damagedThisCombat targets **RETURNED** fresh object, not source. |
| Searing Blow makeCopy | Counter-accepting constructor stores upgrades but begins damage12; it does not replay damage upgrades. Stat-equivalent route separately replays upgrades/restores selected source stats. |
| Stat-equivalent / same-instance | First call source makeCopy, including source side effects; then upgrades/selected stats/costs/misc/bottles/free flag come from then-current source. Not every mutable destination/play flag survives. Same-instance is a distinct object sharing UUID, not alias. |
| Expunger | Sole stat-equivalent override additionally restores X magic values and clones description list. Source/permanent growth owner is a separate attribution law. |

E19's439-class declaration/direct-instruction inventory is complete at those
families, not proof of every constructor, transitive callback, legal mixed mutation
or reflective path. Draw own-card callback precedes power callbacks and refresh;
newly applied powers sort, with Confusion priority0. Preserve actual ordered-list
mutations rather than invent a total priority table.

### 4.1.2 Ten current X actions (F; E18/E19)

Unless stated otherwise, non-sentinel energyOnUse replaces live effect basis;
Chemical X adds2 **once per action**, before upgrade increments/positive test.
Positive-count nonfree branch pays **live current energy**, not captured basis.
Ordinary player use caps X to live energy only without ignore-total; autoplay
sets one-shot-free for X. This does not rewrite generated children.

| Card | Captured effect/payload and exceptional payment gate |
|---|---|
| Whirlwind | Count all-enemy hits with captured damage array/type. |
| Skewer | Count targeted hits with captured damage/type. |
| Reinforced Body | Count separate Block actions of supplied per-action Block. |
| Malaise | Adjusted count+1 if upgraded; separate negative Strength then Weak actions. |
| Doppelganger | Adjusted count+1 if upgraded for next-turn energy/draw powers. |
| Tempest | Adjusted count+1 if upgraded for Lightning channel actions. |
| Collect | Adjusted count+1 if upgraded for Collect power. |
| Multi-Cast | Current orb required before any boost/payment; upgraded adds1, count−1 no-removal evocations then final removal/evoke. No orb means neither payload nor payment. |
| Transmutation | Card use raises capture to at least live energy; adjusted count colorless proposals. Fresh copy, source+ upgrade, guarded setter request0, then temporary-hand child; positive count gates nonfree live-energy payment. No self-ID exclusion: self-generated Transmutation stays X/no new free flag through fresh/upgrade/stat-copy/available-hand path absent unrelated modifiers. Ordinary nonnegative output becomes turn0, not universally base0/free. |
| Conjure Blade | Upgraded card supplies captured energy+1 upstream; Chemical X adjusts action count. Even zero builds Expunger and enqueues draw-pile creation; nonfree live-energy payment is **outside** positive-count gate. |

These local branches do not prove all generated-card/orb/target/death interactions
or initialized-root realizability. Producer free-payment state does not confer a
free flag on each new child; generic zero-output language is expressly superseded.

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
positive post-modifier HP in arbitrary injected states. E19 narrows the legal
conditional: on a non-dying player with amount-preserving/boosting reachable
modifiers it remains positive; no concrete power onHeal override exists in this
archive. Bloom/dying forced fixtures are not fresh Act1 zero-heal witnesses.
All88 direct damage invokes and every override/reaction are not thereby closed.
Parasite's object-removal max-HP decrease floors maximum at one and clamps HP; that
local helper does not itself invoke death. No later queued heal revives established
death. Reaper retaliation-before-queued-heal is evidenced; a lost-heal outcome or
timer threshold is not (A1C-B04/A1C-B12).

### 4.3 Direct lifecycle dispatch and retained proof limits (F; E19)

Actual draw invokes the card while still draw-top, changes membership, then power
and relic draw notifications; no-argument wrapper refresh follows. Direct draw(n)
does not imply DrawCardAction's No Draw/capacity/soul/shuffle guards. Manual discard
moves/refreshes, triggers card manual-discard, then increments count; Eviscerate and
relic broadcast additionally require neither turn-ended nor end-turn discard.
Evolve reacts to Status (with No Draw check), Fire Breathing to Status or Curse.
No producer may replace **no child enqueued** by a zero-amount draw/discard child:
constructor/update/history effects must be established at that source boundary.
Compound attacks likewise retain captured DamageInfo/target and local checkpoints,
not a generic second-damage action inferred from text. Pending normative card
records remain unaccepted; these are exactness constraints, not new primary tables.

End-turn marker calls relic/blight, pre-card powers, orb producer, hand-for-playing
hooks and stance; room endTurn separately runs powers, queues clear/discard, resets
draw/discard/hand attributes and queues enemy turn. Retained objects leave to limbo;
restoration is queued, ordinary bulk discard suppressed by Pyramid/Equilibrium,
then shuffled remaining-hand clone gets end-player-turn callbacks. Ethereal queues
targeted exhaust above discard; queued exhaust draws do not enlarge that clone.
Restore calls onRetained before insertion, clears temporary retain only; ordinary
Pyramid holding is not retain. The no-gameplay-RNG-argument clone shuffle law stays
open, not silently uniform.

After monster work: surviving-monster end turn, player end round, surviving-monster
end round; then stance/relic/blight start, hand predraw, draw/hand/discard start,
powers/orbs, history/Block reset and queued normal draw. Postdraw **producer calls**
can occur before that draw executes. Initial room start omits that same hand-predraw
pass; do not reuse the repeated-turn sequence wholesale. NewQueue deduplicates
physical objects, has one null end marker and inserts immediate work at index1 if
nonempty (repeated insertions reverse producer order). Ordinary actions precede
preturn, then card queue; mixed repeats need actual collection order.

Direct family inventories include real draw/exhaust/manual discard/retain/copy/
play/use/after-use/scry/turn/death dispatches; similarly named onRetain/onDiscard
have no declarations/direct invokes, triggerAtStartOfTurn has no invokes, and
onDrawOrDiscard has four declarations including two bases, not five overrides.
Fiend Fire captures count, Second Wind captures non-Attack objects; reactions do
not recursively grow either snapshot. LoseHP/Blasphemy use damage HP_LOSS; maxHP
reduction is no hard death assignment. Judgement's InstantKill enemy path and
monster Suicide are not universal player rescue/Fatal handlers. HP/counters/card
notifications commit before rescue/death. Direct-family completeness never closes
every nested override, applyPowers/glow, mixed reaction, entered tail or terminal
dispatch. B04/B07/B12 retain those exact composition obligations.

### 4.4 Card admission, history, use and payment checkpoints (F; E28)

A queued card is not yet an accepted play. At manager dispatch, legality (`canUse`)
precedes the ordinary notification branch; only an admitted play records the inspected
play/onPlay history. On the accepted route, captured energy and ignore-total are set,
then player powers, monster powers, player relics, stance, blights and hand/discard/
draw cards receive their ordered notifications. A later null/dead/escaped target check
can reject use after this history commit; that late rejection omits player use/payment
and does not roll history back. The explicitly suppressed-notification branch is
distinct from normal accepted notification.

Player use then recalculates damage, applies the source-specific X cap only when
ignore-total is false, invokes card `use`, constructs `UseCardAction` (including its
constructor-time callbacks), notifies other cards in hand, removes the played instance
from hand, and applies ordinary energy debit subject to the actual free/autoplay/
Corruption-Skill exceptions. Payload actions commonly remain queued after this use
entry. Five repeat powers explicitly set autoplay and ignore-total true while copying
energy; this does not generalize to other queue producers. Preserve the already stated
Forethought+ selected order, Setup/Forethought positive mutable combat-cost free-flag
guard without numeric zeroing, per-instance Masterful Stab notifications, Eviscerate
source receiver, Transmutation negative-X/self-generation path, and Omniscience queue
flags versus later manager-derived play flags. No total cost precedence or all mixed
callback law follows.

### 4.5 Shared manager, queue and power-selection checkpoints (F; E28)

The manager does not recursively drain a selected action: in its executing branch it
updates at most the current action, and later manager updates choose subsequent work.
Selection priority is current entered action first, then index 0 of ordinary `actions`,
then index 0 of `preTurnActions`, then card-queue work, then monster snapshots/turn
work; the turn-ended branch is later still. `addToTop` inserts at index 0,
`addToBottom` appends, and repeated top insertions reverse producer order.
`addToTurnStart` inserts at its own index 0 but does not outrank ordinary actions.
Card-queue immediate insertion uses index 1 when nonempty (otherwise index 0), with
a separately managed single end-turn marker. The marker calls
`triggerOnEndOfTurnForPlayingCard`; later `DiscardAtEndOfTurnAction` clones and calls
`Collections.shuffle` on the remaining hand, then invokes the distinct
`triggerOnEndOfPlayerTurn` callback. Entrench overrides only that later callback and
front-queues `ExhaustAllEtherealAction` after bulk discard work was queued; Entrench
is not itself Ethereal, and the cleanup remains pending work. Repeated top insertions
reverse producer order. `Collections.shuffle` has no explicit gameplay RNG argument,
so no seeded/native-order/uniform shuffle law is claimed. These are list-specific
rules, not a universal FIFO across effects, souls, room objects or other containers.

For power application through `ApplyPowerAction`, source `onApplyPower` callbacks can
precede target validity and named relic prevention; Artifact is checked after those
prevention hooks. Existing matching IDs stack at the application branch (except
`Night Terror`); a newly added power is appended, the list is sorted by
`this.priority - other.priority`, then initial/modify callbacks follow. Stable equal-key
order is scoped to that sort operation. Direct `addPower` instead stacks matching IDs
or appends without sorting. Lifecycle passes visit the current list; do not snapshot or
re-sort every callback family by editorial rule.

**E28 update to A1C-REQ-009 (F/O):** Preserve these current/pending/container
distinctions and selection priorities; callback-created work remains pending until
the manager reaches its container on a later dispatch. Selective post-combat filtering and later full manager
clear are separate transitions: neither unwinds the already-entered call nor proves
all other work drained. Exhaustive callback and reachable-combination closure remains
A1C-B04/B05/B07.

### 4.6 Distinct player/monster damage, rescue and terminal checkpoints (F; E28)

Player damage and monster damage are different state machines. For the inspected
player-target route, clamp negative input, apply player Intangible and Block, then
apply source relic, owner-power, target relic and target-power damage modifiers. The
later attack notifications are source relic, owner powers, target powers, then target
relics; the owner-sensitive callback block is skipped for null-owner damage.
`onLoseHpLast` precedes capture of `lastDamageTaken`; later `onLoseHp` amount changes
therefore do not rewrite that capture. Positive post-hook loss commits HP-loss/damage
counters and notifies then-current hand, discard and draw card instances once each
(excluding exhaust), before HP/bloodied updates and rescue/death. If lethal and Bloom
is absent, Fairy is consumed before unused Lizard Tail; their healing is synchronous
through normal heal modifiers, and earlier counters/notifications are not rolled
back. Otherwise death is committed; a later queued heal is not resurrection.

Monster damage separately checks Intangible and dying/escaping, runs its distinct
owner/target callback order (including the local target-power `wasHPLost` pass not
guarded by positive damage), captures `lastDamageTaken` before HP subtraction, and
has no player card-history or rescue pass. First `die(triggerRelics)` sets `isDying`
and conditionally calls target-power death and player relic monster-death hooks; it
is not `isDead`. Death-timer settlement, group/cannotLose checks, room `endBattle`,
and manager clearing are later checkpoints. Escape has its own three-second timer and
`escaped` state, not death/Fatal/victory credit. Split children are later actors, not
immediate replacement kill credit; `cannotLose` suicide can suppress kill hooks. Player healing visits relic modifiers then power `onHeal`; monster
healing instead checks dying, visits powers only, then adds/caps HP. These bounded
base paths do not enumerate all damage producers, overrides, Fatal attribution,
simultaneous callbacks, splits, rescues or terminal interposition; retain B04/B06/B07.

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
the full paired-content records; E21–E24 now supply those local primary records,
while their shared execution/composition residuals remain A1C-B07.

### 5.2 Required Note/Silent review constraints (F, bounded; E09-R)

These independently worded corrections override any incompatible implication from
E09; they do not close the shared cost/copy/callback matrices or certify all Silent
mechanics. The locators name inspected members, not a required private artifact.

| Finding | Known constraint | Evidence locator / remaining boundary |
|---|---|---|
| SI-I1 — Setup / Forethought | Setup moves selected existing instance to draw-top; Forethought to bottom. Both singleton and selection-return branches grant `freeToPlayOnce` only for positive mutable **combat cost** `cost`, not printed cost or `costForTurn`. No numeric-zero rewrite. Combat zero with positive turn-only cost, X/unplayable get no new flag, but failed guard does not clear an existing one. Reset preserves it; actual after-use cleanup clears it; other explicit writers can clear it earlier. Forethought ordinary requires one where selection exists; + permits any ordered sequence including empty. Preserve iteration/insertion order, not sorted subset. No legality/payment bypass. | E19's Setup/Forethought/card-reset/use-cleanup anchors extend E09-R. Exact mixed overrides/free/copy precedence remains B04/B07. |
| SI-I2 — Masterful Stab | Fresh ordinary construction starts cost zero. Each qualifying damage notification increments that instance's cost by one through its cost-update helper, not by the number of HP points. The inspected positive-loss combat dispatch notifies hand, discard and draw, **not exhaust**. Exhume returns the existing exhausted instance without missed-notification catch-up. Preserve location and per-instance cost state; a global combat HP-loss count is not a replacement. Ordinary fresh copy and stat-equivalent copy are distinct routes, not universal historical reconstruction. | `MasterfulStab` constructor/`tookDamage/makeCopy`; `AbstractPlayer.damage/updateCardsOnDamage`; `ExhumeAction.update`; `AbstractCard.updateCost`. Other prevention, cost-layer and copy composition remains B04/B07. |

Future discriminators under T09–T11 must retain the selected/card-instance identity,
field/flag state and exact notification boundary. In particular, a legally acquired
Masterful Stab exhausted by True Grit+, followed by positive HP loss while exhausted
and retrieval by Exhume, must not receive that missed increment. This is a
source-derived counterexample to a global formula, not an executed trajectory or
initialized-root-seed witness. Note's NT-01–15 and Silent's SI-01–22 remain
unexecuted future cases; neither inventory nor E09-R's bounded clearance runs them.

### 5.3 Integrated paired primary records and local discriminators (F; E20–E24)

The exact paired record tables and their full local conditional payloads are in the
linked E21–E24 documents above; this section states the cross-family requirements
needed at this whole-spec boundary rather than duplicating four catalogs. The
integrated local record sets are 75 Silent kinds plus dependent Shiv; 75 Defect
kinds plus two Status/four orb kinds; 75 Watcher kinds plus physical dependencies
and nonphysical Wish choices; and 75 red, 35 ordinary colorless, five Status,
eleven Curse, four stored-event and twelve dependent/control records. Counts
describe primary record coverage, not transitive execution closure.

| Integrated source fact | Required distinction at the consolidated boundary | Still not proved |
|---|---|---|
| Silent Gamble/Expertise and Bane | “No child action enqueued” is not equivalent to constructing a zero-amount draw child: the latter can run its initial history-clearing update. Bane captures target and both DamageInfo payloads at use; its follow-up has distinct null/Poison and completion-time positive-HP gates. Only its initial-duration branch additionally requires a positive-HP target and cancels a non-THORNS payload when its owner is dying. | Shared history mutation composition, arbitrary Bane origins, all nested callbacks and terminal outcomes; do not generalize that branch's source-death check to every update. |
| Defect Lock-On and potion X guard | Lock-On amount is duration, while the orb modifier remains ×1.5; qualifying duration 2 then 3 stacks to 5 and expires at its named end-round lifecycle. `setCostForTurn(0)` rewrites only an existing nonnegative turn cost; negative X/unplayable sentinels remain negative. | Global hook ordering, all generated-X/copy precedence and every orb/action composition. |
| Neow and seeded history | Neow full offers consume category selections at construction; drawback selection precedes filtered reward selection; activation owns payload draws. Colored duplicate retries retain rarity, while colorless identity uses card RNG after Neow rarity draws; potion offers share successive potion RNG calls. Tiny Chest ordinary acquisition initializes its counter at 0; the fourth room roll still consumes its primitive float before forcing treasure. | Initialized/history-conditioned joint measure for offer and shuffle histories; no independent-stream or uniform-permutation assumption follows. |
| Potion and Dig routes | Potion source is actual Ironclad character, not printed color. After copy/upgrade, `setCostForTurn(0)` changes only an existing nonnegative turn cost; X-cost −1 stays negative and no generic free-play flag is set. Distilled Chaos captures each queued action's target during potion use, before any queued card resolves. Campfire Dig rolls common/uncommon/rare 50/33/17, uses the normal/front relic selector and offers one relic; it is not a chest and has no chest gold/tier/key hooks. | Downstream action retargeting/death/cancel composition, relic seeded-shuffle measure and complete commit/terminal arbitration. |
| Watcher Omniscience | The queued original/copy requests preserve `randomTarget=true, immediateCard=false, autoplayCard=true, ignoreEnergyTotal=false`; the card manager later derives `ignoreEnergyTotal=true` for the card play. That derived value does not rewrite or falsify queue provenance. | Nested selection ordering/cancellation, mixed repeats and global queue/terminal resolution. |
| Red and colorless lifecycle | Feed's qualifying Fatal raises max HP then requests ordinary healing through hooks/cap (not fixed current-HP gain). Mayhem's callback is pre-normal-draw and queues deferred wrappers; completion of their nested plays relative to normal draw remains open. Flame Barrier retaliates at the hit owner only for a nonnull other-owner hit that is neither THORNS nor HP_LOSS, then removes at the next start turn. Exhume transfers an existing eligible exhausted instance, with explicit empty/hand-full/no-candidate branches and no history catch-up. Entrench itself is not Ethereal; its own end-player-turn card-callback override front-queues Ethereal cleanup. | Whole hook/queue arbitration, death/revival, exceptional initialized reachability and all combinations with other callbacks. |

Every row is a source-bounded fact from the exact linked, reviewed revision, not a
newly performed source audit. T09–T18 discriminate these cases where applicable;
all remain future, unexecuted acceptance obligations.

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

### 6.2 Initialized RNG ownership and linked consumption (F; E27)

For a fixed numeric run root `s`, the inspected initializer constructs twelve distinct
stream objects—monster, event, merchant, card, treasure, relic, monster-HP, potion,
AI, shuffle, card-randomizer and miscellaneous—from that same root. Their initial
expanded XS pair and zero wrapper counter are equal, but object state advances
separately after construction; this is common-root initialization, not twelve IID
seeded domains. For Act 1, Exordium replaces the map wrapper with a fresh stream from
`s + 1` using signed-64 wrap semantics. Map topology and its direct-XS bucket shuffle
therefore share that replacement stream/cursor, not the earlier relic stream.

| Source transition | Normative relation to retain | Still not established |
|---|---|---|
| Relic pool initialization | Populate Common, Uncommon, Rare, Shop, Boss inputs; consume five consecutive `relic`-stream `randomLong()` outputs in that order, each seeding a separate Java shuffle. Java shuffle rejection advances only that fresh Java generator. Apply owned-ID deletion after all five shuffles. | Exact input iteration order of hash-backed catalogs on the target host/JRE; root/history measure or evaluated joint posterior. Never sort silently or replace linked source words with five independent seeds. |
| Neow and card/curse | Neow categories/colored identity/rarity use event-owned state as called; duplicate colored identity retries preserve selected rarity. Colorless identity and deferred curse use the advancing card stream; a previously generated then removed automatic Neow card row still consumes its calls. | Full initialized/history-conditioned action measure, exact posterior over complete offer histories, or independent-stream law. |
| Potion offers | Successive flat-ID Neow/Lab/Woman potion choices consume the same advancing potion stream; duplicates remain allowed. | Unconditional history-weighted offer joint or a different native cursor policy. |
| Question room and event | The question-room float is consumed before Tiny Chest/Juzu overrides; event branch then eligible ID use counter-linked copies/advances of event state, branch before ID. | Complete event/history source law and evaluated joint distribution; no independent resampling after override. |

For any fixed `(s, ordered catalogs, legal prior history and caller transcript)`,
these source transitions are deterministic conditional wiring and compose with the
finite kernels in §10.1. This does not supply a measure over root seeds, profiles,
player choices or histories, nor all producer/context transitions needed to evaluate
every observable joint. Seed UI conversion and timestamp paths do not define a
standard-run prior. The conditional zero-Elite criterion still needs a witness or
exclusion with topology and bucket derived from that same `s+1` stream. The seed image is the audited 64-bit input image, not all nonzero 128-bit XS states;
seed 0 aliases signed minimum after expansion. No user-seed prior follows from that
constructor. These claims are static/bytecode-backed; no initializer/runtime ran.

### 6.3 Destination and construction laws

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

E27 connects this map transition to run initialization: all twelve named streams
begin from common root `s`, but Act1 map generation replaces the map object with a
fresh `s+1` state; the map topology, direct-XS bucket shuffle and marker therefore
share that replacement path and do not continue the relic cursor. This narrows the
conditional source wiring only. The root/history measure, host/JRE relic catalog
order and a same-root initialized zero-Elite witness/exclusion remain B02/B03.

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

**A1C-REQ-019 (F/O; E20):** Appendix F is the integrated local normative A0 record
for all25 scoped enemy kinds: source identity, ordinary HP/setup, move IDs/intents,
requested payloads and conditional selection. Move history records selection,
including interrupted choices; never renormalize a finite primary/secondary law.
Local reaction, spawn, boss, Looter, factory/event, burning/key, preview and fixed
Status checkpoints in F are requirements, not a complete transitive scheduler.
Strength application clamps resulting amount to[−999,999] after stacking/reduction;
processed Block gain floors, adds then caps current Block at999 for every creature.
These are helper application bounds, not request clipping, universal counter caps,
or permission to saturate every numeric field. E20 supplies source-corrected local
facts; global callback, timing, nested-return, terminal and initialized RNG/root
composition remain B04/B05/B07/B12 as named.

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
closure. AppendixE now integrates E18's141 local primary rules and13 actual shared
consumers, plus §4.1.2's10 current X action laws. Each still requires
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

E28 adds shared commit checkpoints; retain exact source boundaries rather than
interpreting screen close or queued child as completion:

| Source | Mutation/commit checkpoint | Return/settlement boundary |
|---|---|---|
| Card reward | Take removes the selected row before deferred Fast obtain; Skip leaves the offer available. | Reopen reuses that row; later insertion is not implied by Take or close. |
| Shop card/relic | Card purchase enqueues Fast obtain before spend/stock mutation; relic payment/install is committed before an `onEquip` child and restock completion. | Child resolves at its own consumer; no second charge/purchase on resume. |
| Neow transform | Single transform generates before removal; two-card transform removes both inputs before generation. | Parent resumes through its own no-screen consumer. |
| Delayed relic / event | Installed relic update consumes its pending latch/result at the relic-update pass; event result/Proceed effects are event-specific. | Child parent may be absent; source-owned consumer/return route applies, not a universal screen-stack or map return. |
| Note | Incoming hooks and master-deck insertion precede outgoing candidate construction; selected outgoing removal precedes later-room transition persistence. | Direct incoming does not invoke Omamori obtain-effect constructors; selection alone is not persisted departure. |
| Boss successor | Reward/skip and child effects precede next-dungeon transition; later game transition constructs the successor using the same player and applies per-act reset/heal hooks. | `+1` waits for resolved successor-ready boundary; exact generated-map payload remains D08/B05/B10. |

These checkpoints do not close exceptional initialized-history reachability, full
consumer composition or universal terminal arbitration.

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

### 8.6 Integrated numeric and conditional content tables (F; E25–E26)

E25 supplies the actual Neow category/reward payloads, question-room probability
updates, and all25 event branch/eligibility/payment/risk/return records. E26 supplies
the numeric card/reward/pity, shop/restock/price, relic/potion, Dig and correlated
chest tables. These replace roster-only treatment for the local B08 content
families; the linked tables are the detailed normative records.

| Boundary | Integrated numeric/conditional discriminator | Residual boundary |
|---|---|---|
| Neow | Shortlist is fixed +8 max HP or Lament. Full category0 has six equiprobable reward choices and category1 has five; the four category2 drawbacks are equiprobable before filtered reward selection. At max HP79, `b=7` and +2b is14 (not15); ordinary colored triple slots have exact finite common/uncommon masses and no rare; potion offers are three flat-ID calls with repeats allowed. | Category/drawback/payload joint laws over initialized histories; generated then discarded card offer still mutates ordinary card history. No independent RNG streams. |
| Question rooms and events | Probability state updates on room transitions with binary32 rounding; primitive room roll precedes Juzu/Tiny Chest overrides. Tiny Chest forces treasure on its fourth post-equip room roll. Event records include actual entry thresholds, amounts, conditional risks and result routes (for example Dead Adventurer search risk25/50/75% and Cleric entry G≥35 with 35-gold heal or 50-gold removal). | Joint room→shrine→event-ID law and initialized legal-history witnesses for exceptional selectors; not every branch is proven reachable from a fresh root. |
| Cards, merchants, rewards | Ordinary card rarity starts rare/uncommon/common 0/35/65; elite 5/40/55, with per-generated-slot pity mutation before fixed-tier identity retries. Shard's ordinary reward support is320 with rarity counts76/160/84. Shop initial inventory and each price/discount/restock/removal expression use their documented separate rounding order; removal base starts75 and rises25 per paid removal. | Seeded permutation measure, complete shared history/callback integration, source-specific empty-stock and exceptional reachability composition. |
| Chests and relic offers | Chest type is50/33/17; tier and gold presence share their roll, and gold amount uses separately rounded binary32 0.9G/1.1G endpoints. Base relic and sapphire-key competition differs from extra Matryoshka relics; keys commit at their effect checkpoint. | Joint relic pool initialization/depletion under declared input measure, deferred-key/terminal composition and all caller histories. |
| Potions | Ironclad has33 eligible candidates (17/9/7 by tier); ordinary combat/shop/Courier is rarity-first65/25/10, whereas named event factories are flat-ID. Combat Brew uses Fruit-Juice-excluded rare support6; it proposes once per live slot, including the consumed Brew slot, before each obtain check. Bark payload/use-count distinction, Sozu, belt capacity, negative-X guard, Dig and Distilled Chaos target capture retain source-specific records. | All33 full per-use behavior under mixed callbacks/terminal interruption, seeded identity joint law and whole potion/execution/continuation closure. |

The table is a compact index to concrete primary values and paired conditional
records, not a replacement for E25/E26. B08 is narrowed to residual source-proof,
initialized/joint-measure, exceptional-reachability and cross-system composition
obligations listed here and in §17.

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

E28 narrows the source checkpoint without selecting a native clock or natural
outcome: after its local initial wait, Reaper visits current non-dying/non-escaping
monsters in list order, applies damage, sums each target's captured `lastDamageTaken`,
and queues VFX plus a HealAction only for a positive sum; non-fast mode inserts a wait
before that heal, and the action also inserts its own top-priority wait. At the
basically-dead filter, HealAction and damage-typed ordinary work are retained; later
`endBattle`/manager clear can still intervene, or the heal can run first. This is a
conditional race surface, not a natural lost-heal or guaranteed-heal witness. The
ordinary desktop caller rejects raw frame deltas above 0.1 seconds before game update;
that caller bound is not a native scheduler recommendation. Match uses strict `< 0`
timer gates: a pair revealed on the second selection resolves only on a later
underflow; the fifth resolved pair enters cleanup with a positive wait under the
admitted desktop delta, while exact-zero later differs from negative. These are
source-condition facts, not observed frame frequencies. D04 and B12 still require
a human clock decision and complete timer/settlement registry.

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
approved shortcut. E27 now supplies the fixed-root source wiring for initialization,
map replacement, five relic seed calls and selected linked caller cursors (§6.2); it
does not supply the root/history measure or every producer/context transition. E14
supplies the complete **conditional** finite laws below;
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

### 10.3 Finite source-policy registry and ownership proposal (P; E17/D07)

Run owns creation,resources,room/map/history/pools,combat,continuations,knowledge
and RNG. Combat exclusively borrows persistent resources and owns temporary cards/
piles/enemies/work. Standalone combat gets explicit owned resource context. Private
handles carry(run,typed kind,checked u64 creation ordinal), never public allocation
order or accidental RNG scope. Kinds1–9: permanent card,combat card,enemy,relic,
offer,continuation,effect invocation,combat,room. Copies have new identity and exact
source/capture/permanent-growth-owner record; temporary copy ownership is not
inherited automatically. Source-qualified live lookup differs from captured scalar.

Continuation owns source/invocation,parent,tag,resume checkpoint,captured values,
children,selection order,play context,cancel disposition and completed checkpoint
keys(invocation,checkpoint,source iteration). No undo log. Frame tags1–14: Neow,
Shop,Event,Combat,Reward,RelicObtain,DeckSelection,CombatSelection,PotionUse,ChestKey,
Campfire,BossHandoff,Play,ProfileTransition. Single-parent/acyclic/same-run graph
is not universal LIFO. Play origin hand/autoplay/repeat/special is independent of
free,ignore-total,autoplay,purge,history,captured-X validity/energy,target and source
payment/cleanup checkpoints. Storage does not prove callbacks or parent returns.

SiteKey=(namespace,owner,operation,variant), four u32. Namespaces1–8: card,relic,
potion,enemy,event,run-system,acquisition,power. Operations1–25: use,actual-draw,
actual-exhaust,start-turn,end-turn,pre-draw,equip,reward-build,claim,purchase,
transform,remove,copy,insert,key-commit,selection,resume,transition,damage,heal,
lifecycle,pool-select,initialize,restock,preview. Variant is frozen local site
ordinal, not invocation count. Entry fields: request bounds,basis,eligible IDs,
exact kernel,duplicates/retries,history checkpoints,RNG binding,construction/cost/
copy,destination/capacity,payment/commit/cancel/terminal and parent/resume law.
Basis1–8: CurrentCharacter,Colorless,ExplicitLibrarySubset,FixedIdentity,
ExistingInstance,StoredProfile,CurrentCharacterPotions,SourceAcquisition.

Every field is Fact(value,locator,grade), Proposed(value,decision),
Partial(known,exact residuals,locators), or Open(question,dependency). No row-level
Fact inheritance. Readiness1 incomplete/2 review-required/3 review-closed stays
**1** for this candidate until required semantics/choices close. Independent
caller/override/roster reconciliation must detect sites missing from both producer
flags and registry. Missing required laws block initialization before entropy/IDs;
late lookup failure is administrative content fault, never default pool/reroll/
inert effect/Skip/death. No runtime wildcard owner×operation extension.

All39 direct card owners receive separate source-qualified records:

| Operations | Owners / distinct child policies |
|---|---|
| use/0 random card | Infernal Blade,Distraction,White Noise,Discovery,Chrysalis,Metamorphosis,Foreign Influence,Jack of All Trades,Transmutation; §6's distinct support/offer/discount laws. |
| start-turn/0 random | Creative AI,Hello World,Magnetism with their own intensity/guard/pool. |
| use/0 fixed | Wild Strike,Reckless Charge,Power Through,Immolate,Overclock,TURBO,Blade Dance,Cloak and Dagger,Storm of Steel,Carve Reality,Deceive Reality,Evaluate,Pray,Reach Heaven,Alpha,Beta,Conjure Blade. |
| turn/draw fixed | Infinite Blades,Battle Hymn,Collect start-turn/0; Study end-turn/0; Deus actual-draw/0. |
| existing-instance | Anger use/0; Dual Wield,Nightmare copy/0; Endless Agony actual-draw/0; Nightmare also start-turn/0 materialization. |
| potion | Alchemize use/0; child acquisition7/5 claim/3, not card insertion. |

Applicable cards additionally have insert/0 carrying captured parameters: ordinary
hand,selected-offer copy,draw-pile,fixed hand,discard or existing-instance copy
route—not a new selection. Branch/Toolbox/Mark of Pain separate insert/0; Mirror
claim/0; four card potions insert/0; Brew acquisition7/5 claim/3 with combat/outside
provenance. Relic sites: Branch actual-exhaust,Toolbox pre-draw,Mirror copy,
Astrolabe/Pandora transform,Bell equip0/1,Cursed Key lifecycle,Cauldron equip,
Tiny House equip0/1/2,Orrery equip,Dream lifecycle. Prayer Wheel/Question Card/Crown/
Shard have individual reward-build; Courier restock0/1/2; typed/colorless potions
use; Brew use0/1; Memories/Elixir/Gambler/Chaos selection. Five repeat powers have
individual lifecycle records; Sentries,acid slimes,Slime Boss,Hexaghost need fixed
card lifecycle variants. AppendixE's13 shared relic consumers need their actual
damage/reward/lifecycle/use keys, neither inert nor transitively closed by a key.

System owners6/1–8: map(topology,bucket,marker,buff),encounter(weak/strong/elite/boss,
composition/HP/opening),event(ordinary/shrine/eligible/one-time),relic pools(five
initialization,normal/end/screenless/noncampfire),combat(setup/draw/discard/exhaust/
shuffle/turn/victory/death/escape),campfire(menu/each option),chest(ordinary/boss/
rewards/deferred keys),handoff(reward/relic/successor). Acquisition owners7/1–6:
card reward ten caller variants; merchant stock/purchase/purge; five transforms;
eight potion-offer callers; effect-card/direct-Note/relic/potion obtain and object/
string/iterator remove/preview; ordered/unordered/board/fixed selection. Every
appendixC event has selection0/resume0/transition0 plus explicit Note incoming/
outgoing/profile,Wheel reveal/confirm,Match board/picks/pair/cleanup,Face missing
pool,Dead Adventurer category/combat,Mushrooms combat sites. Full branch/variant
equality and semantic dictionaries remain B04/B07/B08/B10.

Candidate RNG prefix run.a1.candidate1. roles1–11: map(act),encounter(act),
relic-pools(initial),card-acquisition(act),potion-acquisition(act),merchant(act,visit),
event(act),combat-generation(combat),combat-placement(combat),combat-ai(combat,enemy
semantic position),combat-hp(same). ASCII canonical decimal semantic ordinals obey
HMAC-v1's1–255-byte fields. Position entry:formation or spawn:semantic-ordinal:
child-position cannot depend on allocation renaming. Each site still needs
KernelBinding(kernel,conditioning state,history transition,correlated sites,input
measure,cursor relation); it may require sharing/joint state or rejecting the split.
Role names do not approve independence. §10.1–10.2 update E17's older shuffle-open
wording only at their conditional mathematical scope. No finalized new RNG domain
registry/semantic catalog hash is asserted.

## 11. Packed observation and candidate proposal

**A1C-REQ-032 (A/P/O):** Use fixed-width numeric rows in variable-length packed
tables, separately versioned from combat-v1. Legal piles/decks/menus can grow:
Hello World plus sustained defense against a non-scaling Louse can grow combat
piles; Frozen Eye can make growing ordered sequences public. Ten-card hand capacity
is not a deck/pile bound. Do not impose a guessed 256/512/4096 limit, truncate,
page away choices or force gameplay resampling on allocation failure.

Proposed transport for each table t: `int64 values[M_t,W_t]`, native-endian,
C-contiguous; `int64 offsets[B+1]`, starting zero, nondecreasing, ending M_t;
aligned `int8 known[M_t,W_t]` containing 0/1 for every column. Adjacent offset
difference is the sole length; offsets count rows. An all-empty table retains
`[0,W_t]` shape. Cross-table references are tagged **slot-local public rows**,
absent tagged(0,−1); never global offsets/native IDs. Validate against the referenced slot
segment. Knowledge mask shares its table's offsets.

### 11.1 Candidate descriptor, not an assigned fingerprint

E17's concrete P layout replaces this draft's earlier width6 candidate, **not** any
combat-v1 interface. Semantics/content and D04 clock/wait extension are unfrozen;
no final fingerprint exists. Fixed context[B,12]: phase,decision_kind,act,
global_floor,current_node_ref,turn,terminal_outcome,parent_context_kind,
selected_count,min_select,max_select,ordered. Fixed player[B,16]: character,
ascension,hp,max_hp,gold,block,energy,max_energy,draw_per_turn,potion_capacity,
ruby,sapphire,emerald,stance,orb_capacity,mantra. Both have same-shaped int8 known.
All18 packed tables always exist:

| ID/table | W | Ordered int64 columns |
|---|---:|---|
| 1 nodes | 7 | x,y,room_kind,visited,reachable,burning_marker,boss_code |
| 2 edges | 3 | from_node_ref,to_node_ref,currently_selectable |
| 3 permanent_cards | 9 | card_code,upgrades,misc,bottle_kind,removable,upgradeable,base_cost_kind,base_cost,public_position |
| 4 combat_cards | 12 | card_code,upgrades,misc,zone,position,cost_kind,base_cost,turn_cost,public_owner_ref,retained,exhaust_on_use,ethereal |
| 5 relics | 4 | relic_code,public_position,counter,used |
| 6 potions | 4 | belt_slot,occupied,potion_code,potency |
| 7 actors | 8 | enemy_code,formation_position,hp,max_hp,block,intent_code,intent_damage,intent_hits |
| 8 powers | 5 | owner_table,owner_ref,power_code,amount,public_position |
| 9 orbs | 5 | position,orb_code,passive_amount,evoke_amount,counter |
| 10 offers | 9 | offer_kind,content_code,target_table,target_ref,amount,price_kind,price_amount,status,public_position |
| 11 room_objects | 7 | object_kind,public_position,face_state,content_kind,content_code,object_state,remaining_attempts |
| 12 selection | 5 | source_table,source_ref,selected_ordinal,selected,eligible |
| 13 knowledge | 6 | fact_kind,subject_table,subject_ref,position,value_kind,value |
| 14 properties | 8 | owner_table,owner_ref,property_code,ordinal,value_kind,value,ref_table,ref_row |
| 15 history | 7 | fact_kind,ordinal,subject_kind,subject_code,value_kind,value,visible_floor |
| 16 candidates | 8 | action_kind,source_table,source_ref,target_table,target_ref,option_code,parameter_start,parameter_count |
| 17 candidate_parameters | 6 | parameter_kind,ordinal,value_kind,value,ref_table,ref_row |
| 18 relations | 5 | relation_kind,from_table,from_ref,to_table,to_ref |

Let U=2^63−1. Counts/offsets/spans0..U, index<segment length; check host-size,
products/additions/byte arithmetic. Core transport bytes=252B+144(B+1)+9Σ(M_tW_t),
excluding detached finals/metadata/facade/copies. Unknown=value0/known0, distinct
from known zero. Specified optional sentinel−1 has known1. No fake absent rows.
Signed64 transport does not replace source integer/binary32 laws or prove legal
numeric coverage. Source clamps apply at their actual helper boundary, e.g.
addEnergy/addBlock999, **not** incoming request clipping. No universal counter cap;
source per-field domain/application audit remains B09. Encoding exhaustion faults.

Proposed enums (start1 unless stated): phase Neow/Map/Room/Combat/Selection/Dead/
Act2Ready; outcome0 none/1 Death/2 Act1CompleteAct2Ready; decision0 none then Neow/
Map/Event/Shop/Campfire/Chest/Reward/BossRelic/Combat/DeckSelection/CombatSelection/
MatchBoard/PotionSelection/Handoff. Character1–4 Ironclad/Silent/Defect/Watcher,
creation only1,ascension0. Stance0 none then Neutral/Wrath/Calm/Divinity. Room0
unknown then monster/elite/boss/event/shop/rest/treasure/boss-treasure. Zone hand/
draw/discard/exhaust/in-play/other-public; cost ordinary/X/unplayable (numeric cells
masked for last two); bottle0 none then Attack/Skill/Power. Offer card/relic/potion/
gold/key/HP/maxHP/removal/upgrade/transform/source-option; price0 none then gold/
HP-loss/damage/maxHP/source-composite; status offered/selected/claimed/skipped/
removed/disabled. Disabled row has no candidate. Object kind board-position/event-
option/campfire-option/chest/reward-container/source-public-state; face0 n/a then
down/exposed/remembered-hidden/removed; object state0 neutral then available/
disabled/selected/resolving-public/completed/removed. Further stage details need
owner-qualified vocabulary. Real belt slots include empty occupied0,code−1 and
masked potency, not machine-max padding. No-combat/selection cells mask; persistent
terminal resources stay public.

Ref=(table,local row), absent(0,−1), player(0,0) only for ActorRef. Table0 otherwise
not addressable, except property owner context(0,1), never candidate target/actor.
Typed targets: edges→1; combat permanent owner→3; powers→player/7; offers→3/4/5/6/7/
11 or absent; selection→3/4/6/10/11; knowledge→1/3/4/7/10/11 or absent; relations→
1–13 with exact relation kinds; candidate source→3/4/6/10/11/12 or absent,target→
player/1/2/3/4/6/7/10/11 or absent. Candidate spans address local17; tag-set membership
still needs exact core menu validation. Unknown refs mask both cells0. Accessor
alone adds submitting-slot offset; globally in-range neighbor refs are invalid.

Value kinds1–5: signed integer,boolean,enum,binary32 raw-u32-bits,row reference.
Ref scalar0 with typed ref; other kinds absent ref. Properties own actual1–13 or
player/context, never recursive properties/candidates; property/parameter values
may reference1–13. Knowledge/history scalar kinds1–4. Property codes1–45:
base-damage,displayed-damage,base-block,displayed-block,magic,captured-X,
cost-until-played,cost-combat,cost-free-once,autoplay,ignore-total,purge,
source-site-public,copy-origin-public,retained-duration,delayed-duration,uses,
gold-price,HP-price,maxHP-price,selected-order,can-skip,can-confirm,scry-count,
public-discard-count,public-HP-loss-count,public-play-count,public-Power-count,
Mantra,orb-slots,Focus,kill-growth,potion-capacity,key-kind,burning-buff-public,
source-option-stage,draw-order-known,outgoing-card-public,upgrades,source-visible-
counter,nominal-price-binary32,intent-visible,payment-status,transform-input-kind,
public-turn-cost-modifier. Code40 requires frozen content-qualified ordinal subkeys,
not an escape hatch. D04 elapsed/running fields/wait opcode remain B10 integration,
not silently assigned to40 or source-option action21.

Knowledge kinds1–9: draw/discard/exhaust counts,known-next,known-bottom,scry-window,
remembered-board,announced-boss,source-revealed. History1–7: visited-room,revealed-
event,chosen-option,revealed-board,public-play,public-discard,public-counter.
Relations1–6: reward-exclusive,reward-parent,public-copy-source,public-permanent-
owner,public-selection-source,public-known-order. Knowledge precedes lengths/order/
refs/masks. Unknown draw order is public multiset, not private sequence. Nodes sort
(y,x),edges(from,to); preserve significant source order otherwise sort full public
tuples/known bits, never private tie-breakers. Equivalent duplicates retain
multiplicity; different public futures need representation, not hidden ID/merge.
Every public field needs fixed/property/relation or evidenced nonpublic/irrelevant
disposition at catalog freeze;45 codes alone do not establish that inventory.

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

E17 action codes1–21: play-card,end-turn,use-potion,discard-potion,traverse-map,
choose-event-option,buy,purge,choose-campfire,claim-reward,skip-reward,take-boss-relic,
skip-boss-relic,select-instance,deselect-instance,confirm-selection,cancel-selection,
reveal-board,Singing-Bowl,proceed,choose-source-option. The last needs a frozen
owner-qualified option and exact core choice; no arbitrary callback. Parameter
kinds1–7: count,chosen-position,ordered-selection-member,X-public-value,option-
argument,public-price-component,source-public-flag. Empty span(0,0). Caller submits
index/token, never edited candidate/parameter rows. Order action kind,visible
source/target order,option,ordered parameters; source public_position preserves
strategic option order. Carrier0≤a<U; live legality0≤a<N≤U. No U-sized allocation.

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

E17's concrete lifecycle/metadata/replay proposals below narrow B10's representation
gap; semantic/profile/time integration and human D07 disposition remain. These
names are recommendations, not a current
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

### 12.1 Operation-specific lifecycle, identities and delivery (P; E17)

Pool states1–5: Uninitialized,Ready,Operating,Faulted,Closed. Slot states at usable
boundaries: Uninitialized,Live,Terminal. Ready pool can contain uninitialized
neighbors after a proper subset reset; uninitialized slots have no fake observation.

| Operation | Quiescent pool / addressed slots | Required provenance |
|---|---|---|
| step_batch | Ready / Live only | Current episode/decision token, actionable nonterminal phase, candidate bound/private mapping/core membership. |
| reset_batch | Uninitialized or Ready / Uninitialized,Live,Terminal | **No step token/menu/candidate/actionable-phase requirement.** Every requested profile/time policy,seed tag/width/range,clock/config,stream identity,next ordinal,duplicate RunKey and reservation validated first. |
| read_batch | Ready / Live,Terminal | No action/token predicate; addressed uninitialized slot rejects, no implicit initialization. |

All operations validate exact dtype/shape/copy/index and ascending unique nonempty
addressed slots,compatibility,lease,reference/span/byte arithmetic and required
identity/token/bookkeeping/minimal-error capacity. Reservations consume no semantic
ordinal. **Reset finishes all-row validation/reservations before entropy,identity
installation/ordinal consumption,live abandonment or mutation.** First subset
initialization,manual terminal reset and live administrative abandonment are legal.
Unaddressed slots remain unchanged. Operating/Faulted/Closed reject step/reset;
faulted read is not successful gameplay. Dispatch failure after valid preflight is
pool fault/no rollback, not a promise of atomic runtime reset.

Create validates positive batch/workers,effective limit,complete catalogs/bindings/
profile/time and reset/recording modes, but installs no run until reset. Read/reset
return addressed detached observations/metadata without fabricated reward/final.
Live reset is abandonment, not death/truncation/accepted gameplay action; optional
lifecycle record separately identifies old/new runs. SingleManual mode1 has one
slot,readable terminal/zero candidates until explicit reset; VectorSameStep mode2
captures terminal before replacement. Subset B counts addressed returned rows;
final_slots indexes that returned order, not underlying pool indices.

RunKey=(caller stream128,episode u64), independent of slot/worker. Explicit reset
and replacement use next checked ordinal; duplicate live keys reject. Seed kinds:
Root32 exact bytes unchanged; Seed64 unsigned64 maps to SHA256(UTF8
spire-lab/act1/seed/candidate1 + NUL + big-endian u64 seed); Entropy32 captures32
system bytes **after preflight**, failure administrative not replacement seed.
HMAC episode ASCII a1:stream-32-lowercase-hex:canonical-decimal-ordinal; partitions
train/selection/final/test unchanged. Replacement retains configured root/profile/
partition, no new entropy or prior Note output. Replay uses separate owner.

Proposed metadata: run_identity uint64[B,3]=(stream high,low,ordinal), same format
stepped/final identities; decision_generation uint64[B]; token uint8[B,32] opaque
pool/run/decision provenance; delivery_identity uint64[3]=(unique pool high,low,
checked operation ordinal); slot_indices int64[B]. Read may advance delivery
bookkeeping, never decisions/RNG. final_present bool[B],ascending final_slots
int64[K],independent K-segment terminal batch with zero candidates and retired
reward/outcome. K0/KB exact shapes. Profile attachment schema_id/hash32,presence[B],
owned length-delimited row bytes, with §9.1 semantic policy still proposed. All
identities/tokens are non-policy. Lease covers packing/export/facade packaging;
caller edit or later step/reset/close cannot corrupt live/cache/past exports.

Candidate identities (P, not released aliases) are spire-lab-act1- followed by
observation-candidate1,candidate-action-candidate1,content-sites-candidate1,
rng-bindings-candidate1,seed-sha256-candidate1,replay-from-start-candidate1;
task is spire-lab-ironclad-a0-act1-sparse-candidate1, delivery
spire-lab-act1-delivery-candidate1. Hash layout/dictionaries/references/order,
task/action/final/attachments in canonical descriptor JSON excluding its own hash;
include separately hashed content/site/kernel manifests. No final hash until B10
closes. Content namespaces1–13: card,relic,potion,enemy,encounter,event,power,intent,
orb,source-option,room-object,public-fact,effect-site. Authored ASCII semantic keys
sort bytewise to codes1..count (u32 owners),0 unknown/−1 absent where allowed.
Character basics distinct; aliases no extra kind. Options/intents owner-qualified;
effects sorted numeric SiteKeys. Adding/reordering/meaning changes identity/hash,
not implicit recode or combat-v1 widening.

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

### 12.2 Exact proposed replay framing and digest (P; E17/R2)

u8/u16/u64 are1/2/8 bytes, multibyte big-endian; i64 two's-complement big-endian.
HASH raw32 bytes. BYTES(x)=u64(byte length)+x, STR=BYTES(strict Unicode-scalar UTF8),
no BOM,overlong/surrogate,normalization,NUL terminator,padding/alignment. SCHEMA=
STR(nonempty identity)+HASH(component descriptor). ATTACHMENT=SCHEMA+BYTES(payload).
OPT(T)=00 absent or01+T present, no other tag; absent differs from present-empty.
Payload validated by its schema, not silently reserialized/defaulted. Below “+” is
byte concatenation and “repeat n” exact repetitions, never an implicit length.

- FILE: literal hex41315250 (A1RP),u16(1),BYTES(HEADER),u64(record count),records,
  u8(end status),OPT(diagnostic ATTACHMENT),exact EOF. Status0 clean-prefix/1 terminal
  forbids diagnostics;2 interrupted requires present diagnostics.
- HEADER: eight SCHEMA pairs in observation/action/content-site/RNG/root-adapter/
  replay/task/delivery order; HASH(composed descriptor);STR(source revision),
  STR(rules revision);u8(seed kind),seed argument(u64 only for Seed64),root32,
  u8(partition1 train/2 selection/3 final/4 test),stream16 high-byte-first,u64 episode;
  ordered OPT attachments creation snapshot/profile policy/time policy,then
  OPT(private-core SCHEMA). Root32/Entropy32 have no seed-argument bytes. Revisions
  are exact nonempty strings; absent core schema expressly disclaims full-core comparison.
- RECORD: u8(1)+BYTES(DECISION). DECISION: u64 accepted ordinal,HASH(pre-public),
  u64 local candidate index,BYTES(semantic action JSON),u8 completion. Completion0
  has no post fields;1 adds HASH(post-public)+OPT(HASH post-core),whose presence
  must equal header core schema. Ordinals start0 consecutively. Only last may be
  incomplete and only status2; interrupted can also follow a complete prefix.
- Lengths/counts0..U and checked enclosing remaining bytes/products/conversions
  before allocation/read advancement; Seed64/episode/accepted ordinals retain full
  u64. Each framed child consumes exactly its length. Reject unknown tags/versions,
  truncation,suffix,noncanonical bytes—no alternate parser/ignored extension.
  Zero records allowed with complete header,including initialization fault;
  entropy/header failure cannot fabricate a root/header. Terminal status requires
  actual regenerated terminal last boundary. Finalized file count is not a promise
  a partially written export is parseable.

Canonical JSON: unique ASCII keys byte-sorted,ordered arrays,Unicode scalars,
base10 integers−2^63..2^64−1. No booleans/null/floats/insignificant whitespace/
duplicate keys/leading zero/plus/negative zero. Only quote/backslash escape normally;
controls U0000..001F use lowercase backslash-u00xx; other scalars including slash/
DEL/nonASCII remain raw UTF8, no normalization. Optional tuple member[] or[value],
action root ordered array. Its exact semantic fields/dictionaries remain B10/NC11–12;
framing fixture[] is not a legal action. Opaque attachments need not be JSON.

Public SHA256 input: descriptor raw32; context **all12 i64 values then all12 known
bytes**; player all16 values then all16 known; tables1..18 each u64 segment row
count,all row-major values,then all same-order known bytes. No outer BYTES,table
tags,fixed lengths,offsets,batch dimensions,tokens/RunKeys/attachments/private
state. Empty segment contributes8 zero bytes. Unknown values must be zero.
Same-step post digest is retired terminal, not replacement; replacement gets its
own creation record. Private-core encoding stays **D NC11/NC12** pending complete
authoritative hidden/RNG inventory; public equality cannot stand in for core.

Six structural goldens below pin framing, not legal states/runnable game replays.
Hex “00*n” means n zero bytes; plus means concat:

| Golden | Exact preimage | SHA256 |
|---|---|---|
| G01 | 00*8 | af5570f5a1810b7af78caf4bc70a660f0df51e42baf91d4de5b2328de0e83dfc |
| G02 | 0000000000000002c3a9 | 1d9b2f68e31ef2b6730c67d7729ca6a523e7f19d8987ca7c72eb93cb3bb9d979 |
| G03 | 01+000000000000003b+00*48+0000000000000002+5b5d+00 | cbba1c720f971d132277b6a1547aa68dee3929bd387ab1b9bca27be9ee76611d |
| G04 | 00*428 | fced892c73c66d68fdbb01bb68464549f19996231e05ceddce888520d3b70227 |
| G05 | 00*32+ffffffffffffffff+00*88+01+00*155+0000000000000001+00*199 | 9970458824150d3f8e19e7bcb2d608b9405f2cc31f6fedfd0136e5464b3eea46 |
| G06 | 41315250000100000000000001b8+H+00*10 | 01ed74ddb35dcc8f98e2542a0b85707c5cbbfc3506f1149f720e0950a5979a19 |

H=(000000000000000178+00*32) repeated8,then00*32,then(000000000000000178)
repeated2,then01+00*32+01+00*28. H440 bytes,FILE464; IDs x/zero hashes are
unrecognized fixture schemas. G03 is68 bytes/59-byte decision,incomplete[];
G04 empty public digest428 bytes; G05 signed context−1/known1 and one unknown
node row is491 bytes. Optional attachment absent00 versus present01+STR(x)+zero
HASH+BYTES(empty) discriminates presence. Reviewer independently reconstructed all
six; this is representation evidence, not execution of gameplay or approved schema.

Replay validates identities/inputs,initializes normally,regenerates menu/digests,
checks index→semantic action and applies ordinary choice. Earliest mismatch returns
component/provenance,never repaired path. Recording off has identical gameplay.
External export/allocation failure remains external provenance,not a reproducible
gameplay event. Snapshot2 remains combat-only; no run checkpoint import is added.

## 13. Batch validity and administrative failure

**A1C-REQ-038 (A/P):** Validate every addressed slot/input type/shape/range and
operation-specific lifecycle/reservations before transition. Token/phase/menu/core
membership are **step-only** predicates; reset instead applies §12.1's complete
creation validation/reservations before entropy/identity/abandonment/mutation.
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
not permission to resume a faulted pool. E17 permits diagnostics/close only, never
reset/read-as-success/retry. Detached earlier successful arrays remain valid.

Proposed privileged fault envelope: int64 width12 with same-width int8 known,
operation,stage,kind,pool_slot,association,accepted_ordinal_ref,site_namespace,
site_owner,site_operation,site_variant,checkpoint,cursor_state. Operations1–6 new/
reset/read/step/replay/close; stages1–7 initialize/core/project/pack/export/facade/
bookkeeping; kinds1–9 content/invariant/RNG-exhaustion/numeric/allocation/worker/
encoding/binding/facade. Association0 none/1 current/2 newly-retired/3 both.
Ordinal ref points to aligned uint64 attachment,not narrowing cast; absent−1.
Cursor_state0 absent/1 available uses exact u128 high/low pair including HMAC exhausted
2^64 sentinel. Owned schema-qualified attachments carry semantic invocation/domain/
scope/original cause. Retain first observed and append concurrent causes without
claiming deterministic scheduler-first error. Pre-reserved minimal error under OOM
retains delivery/operation/stage/kind/association/cause category,not promised text.
Fault diagnostics are not ordinary profile/final success; all workers quiesce before
error escapes. Caller failure after delivered result cannot retroactively fault it.

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
| A1C-T04 | Preserve run5988/map5989 MODEL requested3/assigned2 certificate, rank-barrier criterion and same-seed topology/bucket proof obligation, distinct from initialized zero existence. Disabled marker gates draw nothing; enabled empty increments wrapper only before failure, singleton draws; approved native administrative classification, no reroll/forced Elite/death. Group buff/Insect order remains separate. For one root, distinguish twelve same-root stream objects from IID domains, fresh wrapped `s+1` map replacement from relic continuation, five consecutive relic words and post-shuffle deletion; no initialized zero-Elite witness/exclusion is implied. |
| A1C-T05 | §10.1 finite first-return rejection/final-state and descending composition under approved measure; missing faces0–5 including deterministic-result seed consumption; weighted duplicate-ID board/reveal posterior; empty/singleton/ordered duplicate-kind upgrade prefixes/Tiny House second bottle check; postshuffle-owned-deletion2779/M counterexample, five-pool joint/posterior/requested-tier depletion; declared input order versus silent sorting. Reconstruct five linked relic seed words and Neow/card/deferred-curse/potion/event cursor transitions from a shared root; root/history measure and complete evaluated joints remain open. |
| A1C-T06 | All weak/strong/elite/event/boss families and ordered compositions; first-strong exclusions, last-two/elite history and queue consumption on departure, not event fights. |
| A1C-T07 | AppendixF every identity/HP/setup/move-intent-payload/initial and redirected AI; finite float grid counts, post-Entangle execution flag, interrupted-selection history; factory/ordered instances, event elites, burning application/rounding then Insect, Emerald offer vs obtain, nine fixed Status sites, Strength endpoint and enemy Block999. Root/history-conditioned joints remain T05/B03. |
| A1C-T08 | CurlUp strict preHP/lethal/block gate; Mad/Enrage/SharpHide/SporeCloud/Protect/Wizard/escape checkpoints; asleep/event Lagavulin; living split vs lethal, parent HP at Split execution, fresh ordered children outside queue snapshot; cannot-lose suicide suppresses kill hooks; Guardian queued quota/change/Block; Hex capture/Inferno checkpoints; Looter ledger/decrement/HP/timed escape/refund. Transitive callbacks/terminal/timers remain open. |
| A1C-T09 | Each E21–E24 paired base/upgrade primary record plus every CARD-* shared record; local coverage cannot close by counts alone or successful random runs. Silent no-child versus zero-child history, Bane captured payload/checkpoints, and SI-I2: one qualifying multi-HP loss versus multiple losses, zero-loss prevention, notified hand/draw/discard locations, fresh versus stat-equivalent copy and per-instance cost layers. |
| A1C-T10 | §4.1.1 eight legality overrides/permission gates and accepted-history-before-later-target rejection. Distinguish current action from later ordinary→preTurn→card-queue→monster selection; top/bottom and repeated insertion order; ApplyPowerAction new-power stable-priority sort from direct unsorted `addPower`. Five repeats true/true,energy capture,physical object≠same UUID;mixed queue index1 reversal. Ten X branches captured versus live/free;no-orb Multi-Cast,zero Conjure,Chemical once;Transmutation ordinary output versus self-generated X through fresh/upgrade/stat-copy/hand,then later source capture/payment. Setup/Forethought positive mutable combat cost guard versus turn-only/X/unplayable and pre-existing flag;reset/use/Confusion clearing distinct. |
| A1C-T11 | Draw/movement/creation, No Draw, Void/Deus/Endless Agony; discard causes/Reflex/Tactician; exhaust Spoon/Branch/Embrace/Fiend Fire snapshot. SI-I2: legally acquire Masterful Stab, exhaust via True Grit+, lose positive HP while it is exhausted, Exhume the same instance; no missed-notification cost increment, unlike a global combat-history formula. |
| A1C-T12 | Retain/Ethereal/Pyramid/Establishment; Poison/debuff/Artifact/Explosion; Focus/orb zero/full/Dark/Plasma; stances/Mantra/Scry/extra turns. |
| A1C-T13 | Non-final Feed/Greed/Lesson/Genetic/Dagger durable mutation; temporary copy/upgrade isolation; later death preserves committed resources. |
| A1C-T14 | Distinguish player and monster damage modifiers/callbacks, `lastDamageTaken` capture, positive-loss commit, hand/discard/draw notification versus exhaust, Fairy-before-Tail synchronous rescue and delayed death/escape/endBattle; include split/cannotLose/Fatal controls. Under approved time contract, Reaper nonfinal/last hit/retaliation/rescue/death/no-heal with queued/current/first-update healing, VFX counts and death timer variations; no universal loss or always-drain. Strict-zero/negative and next-visit retirement must match the chosen schedule. |
| A1C-T15 | Each of twenty generation rows at actual inner effect site, including off-color autoplay/repeat; Source support/mass/retry/history and no pity mutation. |
| A1C-T16 | Master Reality Searing Blow hand9/10 versus Foreign Influence/overflow;Searing counter constructor versus upgrade replay/restored damage;Expunger extra X copy restoration. Eviscerate makeCopy mutates source with this-turn discards under guard,returned fresh3 unchanged versus BloodForBlood returned-object combat-loss update;stat copy reads then-current source. n positions excluding new top,selection before insertion. |
| A1C-T17 | All33 potion paired/use/Bark/phase/target/skip laws; Alchemize/Brew live capacity, proposal-before-obtain, combat/outside Sozu and Fruit Juice, own consumed slot/self-production; negative/X cost guard; Dig's separate one-relic route; Distilled Chaos targets captured before any queued card resolves. |
| A1C-T18 | E25 exact Neow categories/drawback/reward quantities and source RNG ownership; question-room history and Tiny Chest fourth-roll override after its primitive draw; all25 event eligibility/branches, no invented initial Leave; Scrap Ooze26/100, Wheel reveal gold once. Match first/second observation, ID match/five delayed attempts/fresh-copy obtain/fifth cleanup entry positive versus later strict-zero landing, no sixth attempt or obtain drain. Woman unlabelled open()/Proceed versus Lab/event-fight direct map. Guarded empty event returns versus committed empty-grid reachability witness/exclusion. Reconcile source-local child commit/consumer/resume checkpoints without assuming drain/rollback. Initialized/history-conditioned joint laws remain open. |
| A1C-T19 | Merchant proposal/retry/stock/sale/prices/discounts/removal/Courier; cancel purge versus confirmed result/no double charge; child onEquip before restock, Grid Peek/nonempty-reward return override and close/reenter same inventory, not universal automatic Shop screen return. |
| A1C-T20 | E26 ordinary/Sharded reward support and pity-before-claim, correlated chest type/tier/gold, and documented shop price/rounding/restock records; Skip/reopen same row, Take consumes before insertion, Bowl distinct +2maxHP then heal2 at full/damaged/refused-heal state with null/last/mixed associated rows. Empty card/Dream offer omitted. Dream/Orrery/Tiny House sources retain history of discarded automatic rows. |
| A1C-T21 | AppendixE141 local/13 shared records plus32 spawn predicates and X/H/G/P/V composition. Red Skull integer half/direct initial versus queued asymmetric COMBAT transitions;Tiny House50 BASE gold reward/one potion OFFER before guarded claim. Statue3/4 rows;Calipers8/20 with Blur/Barricade;IceCream998+3→999 at helper;Ginger/Turnip before Artifact;Pyramid/retain/Ethereal;Spoon destination/no draw for purge/Power;Frog/Mushroom factors;Idol direct/theft/Treasure distinctions;Drill exact Block depletion;Collar/EventRoom vs BlackStar. Counter lifetimes and skipped/rejected requested-tier pool consumption remain exact. |
| A1C-T22 | Chest correlated tier/gold, separately rounded amount endpoints, base-linked sapphire versus extra relics, deferred emerald/ruby, legal skip/key deferral. |
| A1C-T23 | Campfire initial filter→Recall append→scan, Dripper/Hammer plus usable Recall, empty Smith/Toke targets, Rest/onRest/Dream child. |
| A1C-T24 | Fast/Show prevented curse admitted delta0/0.1 and rejected>0.1: one charge, no acquisition/deck-change/insertion; Show entered card update/bookkeeping versus Fast guard, no universal prefilter/phantom curse. Note direct incoming has no Omamori charge; retrieve-and-store same removable card, removal/live reference/ID-upgrade-only transition save. Under approved output choice, detached stage/one commit/revision/abandonment and death/fault before/after; synthetic reference mutation is not natural witness. |
| A1C-T25 | Boss chest/no-pick/legal skips and relic-owned children: Astrolabe0/1/3/4, Cage0/1/2/3 phase differences, Pandora0/nonzero starters, Bell prevented curse still relic offers. +1 after resolved children/transition heal/per-act reset and chosen entry/map cutoff, not close/fade. |
| A1C-T26 | Context12/player16/eighteen exact W/masks;offset lengths[2,0,3]→[0,2,2,5],slot2 local1→global3 only in accessor. Typed refs/spans/masks/unknown0/known−1,ordered choices,empty[0,W],large legal multiplicity and actual helper-domain numeric audit;no request clipping/implicit counter cap. |
| A1C-T27 | Hidden-state/private-handle renaming preserves values/order/lengths/masks/menu; Frozen Eye/Scry/board legitimate reveals change only entitled facts; Dome has no intent-derived damage leak. |
| A1C-T28 | First/last candidate, negative/one-past, stale/cross-slot token/global-offset confusion; invalid final batch row leaves every state/RNG/token/reward/ordinal unchanged. |
| A1C-T29 | R1 four reset discriminators: first proper subset with uninitialized neighbors,SingleManual terminal zero-menu reset,live abandonment without step token,invalid last profile/seed/config/ordinal/reservation rejects all before entropy/ID/abandonment. Step retains token/live/menu predicate. SAME_STEP mixed/consecutive K0/KB finals map returned order;no action applied to replacement,detached profile isolation. |
| A1C-T30 | Retain/mutate exports/finals through step/reset/close; injected core/encoding/binding/facade allocation faults quiesce pool, preserve committed prefix and current-delivery provenance, no success result. |
| A1C-T31 | R2 exact byte grammar/G01–G06,canonical JSON/UTF8/lengths/signed cells/known order;malformed tags/truncation/suffix/U count/optional presence reject,Seed64 and ordinals retain u64. Public post digest retired terminal not replacement;absent core schema disclaims core equivalence. From-start profile/time/semantic mapping earliest mismatch;worker/packing/recording equivalence. All8 boss-seen subsets,prebattle hook,floor15/16;approved clock/wait800 boundary,stopClock,invalid dwell nonadvancement,no hidden-Portal gate/offer reroll. |
| A1C-T32 | Approved facade carrier/Sequence/space/tree/wrappers/partial reset; no O(2^63) allocation, exact caller-RNG bounded sampling; all legacy combat-v1 regressions and known-answer RNG fixtures. |
| A1C-T33 | Generated end-to-end trajectories through every applicable room/route, legal death/skip/key deferral and full handoff, including rare off-color/Note/relic chains; no production filtering or seed-search miss used as exclusion. |
| A1C-T34 | Structural hot-path/ownership/source-policy review and fresh independent integrated review; scope/evidence/administrative limits disclosed separately from any later performance or transfer claim. |

E28's **TX01–TX18** are additional future discriminators, all UNEXECUTED; they refine
(not replace) A1C-T04/T05/T10/T14/T18/T25/T33 for manager priority, insertion/ties,
accepted-history versus late-target rejection, player/monster damage and rescue,
nested commit/return, and Entrench's later end-player-turn callback.

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
| A1C-REQ-003 | §3/§10.3 owners | E05/E08/E17 | T30,T34 | B10 |
| A1C-REQ-004 | §3/§10.3 single authority/overlay | E05/E08/E17/E19 | T10,T13,T32 | B04,B10 |
| A1C-REQ-005 | §3 live durability | E02/E03/E05 | T13,T14,T30 | B04,B05 |
| A1C-REQ-006 | §4 phases/resolution | E03/E05/E08 | T10,T25,T33 | B04,B05,B12 |
| A1C-REQ-007 | §4/§8.2–8.5 continuations | E03/E05/E16 | T18–T25 | B05 |
| A1C-REQ-008 | §4.1–4.1.2/§5.2 origins/cost/copy/X | E02/E10/E11/E09-R/E18/E19 | T09,T10,T11,T16 | B04,B07 |
| A1C-REQ-009 | §4.2–4.5 scheduler/lifecycle | E02/E10/E11/E19/E28 | T08,T10,T11,T14,TX01–TX04,TX17–TX18 | B04,B05,B12 |
| A1C-REQ-010 | §4.2–4.3/4.6/appendixE damage/rescue | E02/E03/E11/E18/E19/E28 | T08,T13,T14,TX05–TX11 | B04,B06,B07,B12 |
| A1C-REQ-011 | §5/appendix A cards; §5.2–5.3 corrections and paired records | E09 (qualified by E09-R)/E10/E21–E24 | T09–T13 | B04,B07 |
| A1C-REQ-012 | §5 off-color mechanics; §5.2–5.3 corrections | E08/E09 (qualified by E09-R)/E10/E21–E23 | T09–T12,T15,T24 | B04,B07 |
| A1C-REQ-013 | §5 specials/Note and E20–E24 dependencies | E03/E09 (qualified by E09-R)/E12/E20–E24 | T09,T11,T24 | B07,B11 |
| A1C-REQ-014 | §6/§10.3 shared resolver | E05/E08/E10/E17 | T15,T17 | B10 |
| A1C-REQ-015 | §6/§10.3 finite site policies | E05/E10/E17/E18/E19/E27/E28 | T04,T05,T15–T21,T34 | B03,B04,B05,B06,B08,B10 |
| A1C-REQ-016 | §6.2/§7/§7.1 map kernel/initialized model | E01/E10/E13/E27 | T02,T03,T04,T05 | B02,B03,B10 |
| A1C-REQ-017 | §7/§7.1/appendixF burning/empty cursors | E01/E10/E11/E13/E20 | T04,T07,T22 | B02,B04 |
| A1C-REQ-018 | §7/appendixF encounters and factories | E09/E11/E20 | T06,T07,T08 | B07,B08 |
| A1C-REQ-019 | §7/appendixF complete local enemy records | E02/E11/E19/E20 | T07,T08,T14 | B03,B04,B07,B12 |
| A1C-REQ-020 | §8/§8.2/§8.6/§9 Neow history/children | E03/E09/E10/E15/E16/E25 | T01,T31 | B05,B08,B11 |
| A1C-REQ-021 | §8/§8.5–8.6/appendix C/F events | E03/E10/E14/E16/E20/E25 | T07,T18,T33 | B03,B05,B08,B11 |
| A1C-REQ-022 | §8/§8.2–8.6 merchant | E03/E10/E16/E26 | T19,T21 | B05,B06,B08 |
| A1C-REQ-023 | §8/§8.3–8.6 rewards/transforms | E03/E10/E16/E25/E26 | T13,T20,T24 | B04,B05,B08 |
| A1C-REQ-024 | §8.1/§8.6/§4.1.2/§10.1/appendixD/E relics | E04/E10/E14/E18/E26 | T05,T10,T21,T25 | B03,B06,B08 |
| A1C-REQ-025 | §8.1/§8.6/appendix D potions | E09/E10/E16/E26 | T10,T14,T17 | B04,B05,B08 |
| A1C-REQ-026 | §8.1/§8.2 camps | E03/E04/E16 | T23 | B05,B06,B08 |
| A1C-REQ-027 | §8.1/§8.2 keys/chests | E03/E10/E16 | T22,T23,T25 | B04,B05,B08 |
| A1C-REQ-028 | §9 handoff/task | E03/E05/E07/E08/E16/E28 | T25,T29,T33,TX14 | B05,B10 |
| A1C-REQ-029 | §9/§9.1 profile; §1.1 bounded Note disposition | E03/E05/E09 (qualified by E09-R)/E12/E15/E16 | T01,T24,T29,T31 | B07,B10,B11 |
| A1C-REQ-030 | §9.2 clock/settlement recommendation | E02/E07/E15/E16/E28 | T14,T18,T24,T25,T31,TX10,TX12 | B04,B12 |
| A1C-REQ-031 | §6.2/§10–10.3 RNG/finite laws/registry | E01/E05/E08/E10/E11/E13/E14/E17/E27 | T02–T08,T15–T22,T31,T32 | B02,B03,B10 |
| A1C-REQ-032 | §11 exact candidate widths/growth | E05/E08/E12/E17 | T26,T32 | B09,B10 |
| A1C-REQ-033 | §11/appendixE/F knowledge and intent | E05/E08/E09/E17/E18/E20 | T18,T24,T27 | B07,B10 |
| A1C-REQ-034 | §11 action/menu | E05/E08/E12/E17 | T10,T26,T28,T32 | B05,B10 |
| A1C-REQ-035 | §12/§12.1 API | E05/E12/E17 | T26,T28–T32 | B10 |
| A1C-REQ-036 | §12/§12.1 reset/finals | E05/E08/E12/E17 | T24,T29,T30 | B10,B11 |
| A1C-REQ-037 | §12.2/§9.1–9.2 replay | E05/E08/E15/E17 | T31 | B10,B11,B12 |
| A1C-REQ-038 | §12.1/§13 operation-specific preflight | E05/E08/E17 | T01,T28,T29,T31 | B10 |
| A1C-REQ-039 | §13 exact fault/parallel delivery | E05/E08/E17 | T26,T29–T31 | B10 |
| A1C-REQ-040 | §14 compatibility/performance | E05/E08/E12 | T32,T34 | B10,B13 |
| A1C-REQ-041 | §15 acceptance | E08 and this draft | T01–T34,TX01–TX18 | B13,B14 |

In the ledger only, Tnn and Bnn abbreviate `A1C-Tnn` and `A1C-Bnn`.
Their complete definitions are sections 15 and 17; no external task tracker is
needed to understand a requirement's state.

## 17. Exact blocker and integration ledger

All items are **OPEN** with the narrowed residuals below. Ranked priority is for approval
readback, not an implementation sequence or authority to perform new research.
Dependencies name assigned interfaces for main to reconcile. Main-routed, exact-head
reviewed and merged inputs E13–E28 are integrated within bounded claims; no
unreviewed contract draft or sibling working-copy result was polled or adopted. PR25
remains HELD and is not an input.

| ID / rank | Missing closure, smallest acceptable resolution | Main-owned integration interface |
|---|---|---|
| A1C-B01 / 1 | Adopt evidence target/limits in D01; do not silently claim a verified retail build, historical JRE or original-runtime equivalence. Original gameplay remains unexecuted and is not a test pass requirement invented by this draft. | Whole-spec human fidelity decision; E06 preserved |
| A1C-B02 / 1 | E13 closes initialized underfill MODEL, rank criterion and conditional empty-marker counter+1/raw-state+0 failure; E27 fixes the twelve same-root stream objects and fresh wrapped `s+1` map source. Still need a declared root/history input measure, initialized zero-Elite witness/exclusion over the declared domain with topology and bucket from that same stream, and D02 native map-law/administrative initialization policy. Search minimum2 is not a minimum theorem. | E27 exact accepted head integrated; root/history and zero-Elite proof plus D02 remain |
| A1C-B03 / 1 | E14 closes conditional finite mapper/posterior/observable laws; E27 adds five consecutive relic-stream words, source-owner/cursor links for Neow/card/deferred curse/potion/event, and caller-specific map reseeding. Still choose input measure, joint/independence boundaries and ordered relic catalog; close evaluated producer/context joints for unconditional boards/offers and any required root/profile/history ensemble; bind exact sites to native policy. No IID source words, sorted catalog, or target-host hash order is inferred. | E27 adds concrete conditional wiring, not root/history measure, complete evaluated joints, host order, or B10 bindings; D03/B03 remain open |
| A1C-B04 / 1 | E19 closes direct method/invoke families and corrected local distinctions; E28 adds bounded manager current/pending selection, insertion and power-sort checkpoints, card admission/use/payment ordering, damage/heal/death/escape transitions, and conditional Reaper timer/queue facts. Still required: exhaustive reachable transitive constructors/callbacks/applyPowers/glow, mixed-origin cost/copy/history and queue composition, every damage/Fatal/split/terminal arbitration, and the full mechanic-bearing timer registry. No queueExtraCard direct caller is inferred; no global total cost order or natural Reaper outcome is proved. | E28 accepted shared endpoints are partial; transitive callback/site and terminal composition, B12 timer registry and D04 remain |
| A1C-B05 / 1 | E16 closes local parent/child and selected screen/continuation paths; E28 adds action/pre-turn/card/monster dispatch and source-specific before→commit→consumer→return checkpoints. Still required: global container/Peek/terminal arbitration; initialized-history witnesses/exclusions for mandatory empty selectors, zero-Cage phase and stale/mistyped records; initialized event-list exhaustion; exact successor-ready payload/cutoff. The superseded phantom prevented-Show-curse obligation is not reinstated. | E28 narrows shared scheduler/commit endpoints only; reachability, exhaustive continuation arbitration and D08 remain |
| A1C-B06 / 1 | E18 closes141 primary laws,13 actual shared consumers and10 local X action/payment laws (§4.1.2/appendixE),including corrected Skull/House/self-generated X. Remaining per-row X/H/G/P/V:all transitive payloads,mixed callback/cancel/terminal,source return/exceptional inventory reachability,joint history kernels and native visibility. Local R helper facts remain established,not proof of all combined states. | Reviewed relic input integrated;exact appendixE residual endpoints cross execution/continuation/content/native |
| A1C-B07 / 2 | **Primary paired-record gap addressed:** E21–E24 supply accepted local paired card/dependent records; E20 supplies enemy primary records. E28 adds shared transition checkpoints but not their exhaustive composition. B07 remains OPEN for per-route CARD-* attribution, transitive constructors/callbacks, mixed-origin/cost/copy/queue combinations, damage/Fatal/split/terminal laws, and initialized-root/history reachability where needed; local records/counts are not whole-mechanics closure. | E28 adds bounded shared-source evidence only; substantive transitive/site and exceptional-history obligations remain |
| A1C-B08 / 2 | **Local primary numeric-content gap addressed:** E20/E25/E26 provide enemy, event, Neow and economy/potion values/conditions; E28 clarifies nested commit/consumer/resume checkpoints, not local numeric catalogs. B08 remains OPEN for initialized/history-conditioned joints, exceptional reachability, standard-start certification, cross-system reward/claim/Bark/child/commit/timing composition and remaining exact per-site obligations (B03–B06/B09/B12); local tables do not prove every combination. | Accepted local tables and E28 checkpoints are integrated; root/history, reachability and cross-system composition remain |
| A1C-B09 / 2 | E17 fixes candidate signed64 rows/U counts/checked conversions,not full source numeric bounds audit. E18 actual addEnergy/addBlock999 applies at helper,not clipping requests or all counters. Still map every legal field to actual source integer/binary32/cap/overflow and encoding domain,including growing state, using E20–E26 numeric records. | D07 per-field domain/application audit with reviewed content,not admin cap substitution |
| A1C-B10 / 2 | E17 supplies concrete ownership/registry,18 widths/enums/local refs/carrier,operation-specific reset/delivery/fault and exact ordered replay grammar/digest/six goldens. Still D:NC11/NC12 full semantic catalogs/public-field/site equality,all action tuples,private-core hidden/RNG encoding;NC04 exact joint domain bindings/canonical semantic scopes;NC10 profile/time payload,clock/wait extension;successor cutoff. No final descriptor/fingerprint/schema freeze or human D07 approval. | Reviewed native candidate integrated;final whole-spec semantic/schema/fidelity reconciliation,not missing generic byte framing |
| A1C-B11 / 2 | E15 supplies explicit history facts and decision-ready lifecycle/preset/output/death/fault/reset/replay recommendation, now §9/9.1. Need D05/D06 human disposition, full legal stored-card/upgrade catalog and native schema/diagnostic integration; later-act discovery hooks remain bounded evidence. No automatic inheritance. Neither technical review nor Note bounded approval chooses native behavior. | Reviewed profile/time input integrated; content/native interfaces and whole-spec readback main-owned |
| A1C-B12 / 1 | E15/E16 establish named timer facts and desktop bound; E28 adds conditional Reaper queue/filter/clear alternatives, Match strict-underflow distinctions and the admitted desktop delta bound. §9.2 60-Hz/exact-elapsed/dwell/wait remain P. Need D04 fidelity choice, complete mechanic-bearing timer/settlement/terminal registry and exact native Reaper/Match outcomes under that chosen law. Natural Reaper success/loss and frame distribution remain unproved; no source parity follows from a model. | E28 narrows source checkpoints only; full timer registry, natural outcomes and D04 remain |
| A1C-B13 / 3 | T01–T34 and TX01–TX18 remain UNEXECUTED. Written spec must first define precise expected laws; later separately approved implementation supplies actual gameplay/binding/fault/replay/legacy/generated-route evidence. Not executing product cases in this documentation wave is not a demand to implement before written-spec approval. Documentation checks never discharge later acceptance. | Future implementation acceptance only; coordinated plan/start authorization still separate |
| A1C-B14 / 1 | Prior fb7e9f8 review has zero findings only as BLOCKED-draft merge fitness, not user-approval readiness. This integrated head needs fresh independent contradiction/coverage/source-policy recheck; D01–D08 and accessible complete-spec human approval remain outstanding, then separate coordinated plan review/start. | Main; no ready/merge authority follows from this author update |

The ledger deliberately separates missing evidence (B02–B08), candidate contract
choices (B09–B12), unexecuted acceptance (B13), evidence target (B01) and review/
human gates (B14). Accepting a recommendation cannot silently count as proving a
source law. Conversely, lack of original-runtime validation does not erase bounded
static facts already obtained safely.

### 17.1 Integrated evidence inputs — scope and remaining interfaces

E20–E28 are integrated from exact reviewed author heads and merge receipts in §1.1.
Their accessible links provide bounded primary records or shared transitions; this
section records their disposition, not a claim of transitive completeness.

| Input | Integrated local primary coverage | Remaining interface (still OPEN) |
|---|---|---|
| E20 enemies | All25 local enemy identity/HP/setup/move/payload/conditional-AI records and bounded reaction/split/boss/preview/fixed-Status/factory laws. | Global scheduler, nested callbacks, terminal/timer and initialized RNG/root composition (B02–B05/B12). |
| E21 Silent | 4 basics, 71 nonbasics and dependent Shiv paired records, plus Setup/Forethought, Masterful Stab, Bane and named discard/Poison/Retain/copy distinctions. | Mixed CARD-* laws and callbacks, target/death composition, remaining initialized-root and source-policy closure (B04/B07). |
| E22 Defect | 4 basics, 71 nonbasics, two Status and four orb kinds, including costs, payloads, cost receivers, Lock-On and local orb rules. | Full mixed orb/callback/cost/terminal and source-generator composition (B04/B07/B10). |
| E23 Watcher | 4 basics, 71 nonbasics, eight physical dependencies and three Wish option records, including queue-versus-derived Omniscience fields. | Nested choices, global queue/terminal, cross-family callbacks and initialized joint measure (B03/B04/B07/B12). |
| E24 red/colorless/dependent | 75 red, 35 ordinary colorless, five Status, eleven Curse, four stored-event and twelve dependent/control records, including named Feed/Mayhem/Flame Barrier/Exhume/Entrench branches. | Cross-system lifecycle/callback/terminal composition, stored-card reachability and remaining shared CARD-* laws (B04/B05/B07/B11). |
| E25 Neow/events | Numeric Neow categories/payloads and RNG ownership, question-room update/override law, and all25 event local conditions, amounts, branches and returns. | Initialized/history-conditioned joint measures, exceptional legal-history witnesses, parent/child arbitration (B03/B05/B08). |
| E26 economy/potions | Numeric economy, reward/pity, shop/price/restock, chest correlations, Dig and all33 local potion records including guards/targets. | Seeded-shuffle joint laws, full potion callback/terminal composition, standard-start numeric certification and native numeric-domain audit (B03/B04/B08/B09/B12). |
| E27 initialized RNG/map | Twelve same-root stream objects; Act1 `s+1` map replacement; five ordered relic seed calls; linked Neow/card/curse/potion/event cursor edges. | Root/history prior, evaluated full joints, host/JRE catalog order, initialized zero-Elite witness/exclusion and native bindings (B02/B03/B10). |
| E28 shared execution | Manager/power/card-play checkpoints; distinct player/monster damage, rescue/death/escape; nested commit/consumer; local Reaper/Match timer conditions and Entrench callback. | Exhaustive transitive content/callback/terminal composition, selector reachability, natural timer outcomes and chosen native clock (B04/B05/B06/B07/B12). |

No unreviewed sibling content or working-tree-only records are needed for these
dispositions. Their linked source documents are normative inputs; they do not
substitute for human selection among unresolved D01–D08 proposals.

### 17.2 Three distinct gap classes for final readback

1. **Fact/proof/integration:** B02 initialized root image; B03 root/history measure,
   catalog order and required producer joints; B04–B08 actual remaining per-site,
   transitive and exceptional-history composition; B09 exact numeric application
   domains; B10 complete semantic/public/private-core dictionaries and bindings.
   Human approval of a layout cannot prove these laws. E20–E26 close accepted local
   enemy/card/numeric primary records; E27–E28 add conditional wiring/shared
   checkpoints only. Remaining transitive/source-proof and initialized-measure
   residuals stay open.
2. **Human choices:** D01 evidence target,D02 native empty-map law,D03 measure/order/
   independence,D04 timing/settlement deviation,D05 profile/preset validation,D06 Note
   output,D07 concrete representation/ownership/wire/API,D08 successor cutoff. Reviewed
   candidate schemas do not approve these or fill missing content laws. Accepted
   Rust/single-authority/uncapped/invalid-batch/no-rollback/compatibility architecture
   remains accepted,not a repeated approval question.
3. **Implementation acceptance:** B13/T01–T34 stay UNEXECUTED future obligations after
   separate approved plan/start. Written-spec approval requires precise expected
   laws and independent whole-head review/human readback (B14),not product tests
   before authorization. Document checker/golden arithmetic is not gameplay evidence.

## Appendix A. Finite reachable-card inventory and closure boundaries

These names are identity labels, not localized substring tests or assigned native
enum values. Aliases need explicit catalog mapping. E09 inventories and E10 pool
findings support these groups; E21–E24 provide paired local primary records, while
execution/source-policy composition still requires A1C-REQ-011 through A1C-REQ-015
and A1C-B07.
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

## Appendix B. Enemy identity/HP cross-index (full local laws in Appendix F)

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
anchors. Do not apply an ordinary constructor HP roll to split children. AppendixF
now supplies the accepted local payload/AI records; paired primary card/enemy
records are not an open roster gap. Remaining B07 concerns transitive enemy/card
composition, not unspecified enemy payload or an assumed zero-damage default.

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

Thirteen no-primary-override rows have the E18 shared/global/site laws in appendixE:
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

## Appendix E. Relic primary laws and bounded composition (F; E18)

These141 constructor-ID rows supply local primary conditions/mutations rather than
another roster. “Queue” means enqueue, not executed payload. Actual source numeric
clamps/rounding remain at helper boundaries; maxHP increases invoke healing, not
just a displayed max change. Parent/preview/obtain and entered/current/pending work
remain distinct. Column X=execution/cancel/terminal composition,H=acquisition/
return/empty history,G=source generation/joint kernel,P=transitive power/card/potion
payload,V=native public projection,R=stated local resource helper traced. R is not
all-combination closure. These residuals remain obligations even though each row
has a known local rule; no count or callback name certifies all141 transitive laws.

### E.1 Thirteen shared consumers

| Relic | Source-local law / exact boundary |
|---|---|
| Paper Frog / Odd Mushroom | Vulnerable NORMAL incoming factor1.75 on nonplayer owner with Frog;1.25 on player with Mushroom; otherwise ordinary1.5. Neither adds Vulnerable nor modifies THORNS/HP_LOSS by this branch. |
| White Beast Statue | Set potion appearance100,then ≥4 existing reward rows suppress0; still roll0..99. Success appends/offset−10,failure offset+10. Not identity/Bark/Sozu/capacity override. |
| Calipers | Next-turn Block reduction15 floor0 only absent both Barricade/Blur; either suppresses loss. Start hooks called but their queued Block not yet resolved; reduction before queued draw. |
| Ginger / Turnip | ApplyPowerAction player Weak/Frail respectively prevented at application checkpoint after target/source notifications,before Artifact/insertion. No cleanse or universal direct-add immunity. |
| Ice Cream | Recharge adds remaining energy rather than replacing; Conserve branch skipped. addEnergy ceiling999; battle prep copies recharge then current0,not intercombat energy carry. |
| Runic Pyramid | Suppress bulk end-turn discard and RetainCards selection,not explicit retain restore/hooks or Ethereal exhaust. Ordinary held cards gain no retain flag/onRetained. Clone shuffle/transitive ordering remains G/X. |
| Chemical X | Per-action +2 effect after captured override,before positive test/upgraded increment; not spendable energy. Ten branches §4.1.2,including no-orb Multi-Cast and zero-count Conjure. |
| Frozen Eye | Draw/exhaust views bypass ordinary copied-list sorting; draw reversed copy shows upcoming order. No pile mutation or next-shuffle reveal. |
| Strange Spoon | One fair Boolean only in exhaust-marked non-Power ordinary UseCard cleanup after after-use/flag clearing; purge/Power return before draw. Success follows rebound→shuffle-back→return-hand→discard,not always discard. Independent exhaust/Ethereal unaffected. |
| Golden Idol | Store/recompute rounded binary32 base-gold×.25 bonus for non-theft gold reward outside actual TreasureRoom. Claim base+bonus through gainGold. Not direct event/Coin/Fish gain or stolen refund multiplier; Ectoplasm can block gain. |
| Spirit Poop | Full-game score flag/bonus−1 regardless victory Boolean; not Act1 task reward/HP/gold/debuff. Ordinary ownership still affects relic-count predicates. |

### E.2 All141 local primary records

| Constructor ID | Local requirement | Residual |
|---|---|---|
| `Burning Blood` | Living victory hook requests heal6. | R,X |
| `Akabeko` | Battle start queues Vigor8; NORMAL damage addition persists until Attack onUse queues removal,not one damage packet. | X,P |
| `Anchor` | Queue Block10 at battle-start bottom; entry clears used display. | R,X |
| `Ancient Tea Set` | Rest-room entry arms−2; prebattle resets first-turn latch; first turn consumes to−1 and top-queues energy2 without requiring Rest. | R,X,H |
| `Art of War` | First turn grants none; subsequent turn gains energy1 if previous interval had no Attack onUse,then rearm. | R,X |
| `Bag of Marbles` | Battle-start Vulnerable1 queued per monster. | X,P |
| `Bag of Preparation` | Battle-start draw2 queued. | X,P |
| `Blood Vial` | Top-queue heal2 at battle start. | R,X |
| `Bronze Scales` | Queue Thorns3 at battle start. | X,P |
| `Centennial Puzzle` | Rearm prebattle; first positive combat wasHPLost consumes latch immediately,queues draw3. | X,P |
| `CeramicFish` | Actual onObtainCard calls gainGold9,not offer/movement. | R,H |
| `Dream Catcher` | Sleep after heal/onRest produces ordinary reward cards; no empty offer screen. | H,G |
| `Happy Flower` | Equip0; turn increments,third resets0/queues energy1;persistent across combat;−1 advances1. | R,X |
| `Juzu Bracelet` | Room helper converts rolled MONSTER→EVENT with branch chance reset,not a ban on event-authored fights. | G,H |
| `Lantern` | Rearm prebattle; first-turn top energy1 consumes latch. | R,X |
| `MawBank` | Active room entry gain12; shop onSpendGold permanently disables−2,not every event loss. | R,H |
| `MealTicket` | justEnteredRoom actual ShopRoom heal15. | R,H |
| `Nunchaku` | Every tenth persistent Attack onUse resets counter/queues energy1. | R,X |
| `Oddly Smooth Stone` | Queue Dexterity1 at battle start. | X,P |
| `Omamori` | Two initial charges; named curse effect constructors prevent/consume,zero used. Direct Note bypass;§8.3. | H,X |
| `Orichalcum` | End turn Block0 or explicit trigger top-queues6/clears trigger;turn clears flag;positive gain callback stops pulse/floors supplied float. | R,X |
| `Pen Nib` | Persistent Attack counter:ninth queues power,tenth0;battle start restores at9. Power doubles NORMAL until Attack queues removal. | X,P |
| `Potion Belt` | Equip capacity+2 and two actual empty slots. | R,H |
| `PreservedInsect` | eliteTrigger battle-start initial current HP capped at truncated75% maxHP;maxHP unchanged,no spawn hook. | X,P |
| `Red Skull` | Bloodied iff integer HP≤maxHP/2. Battle start resets active;delayed initial action directly adds Strength3 if inactive/bloodied. Later entry queues+3/activates only inactive in COMBAT;exit queues−3 only active in COMBAT but clears active regardless;victory clears. No lethal-tail guarantee. | X,P |
| `Regal Pillow` | Sleep adds15 to Rest heal amount,not every heal. | R,H |
| `Smiling Mask` | Shop purge50 overrides price paths;room callback presentation only. | H |
| `Strawberry` | Equip maxHP+7 with helper heal. | R,H |
| `Boot` | Owner-present positive non-THORNS/non-HP_LOSS attack-change amount<5 becomes5,not base+5. | X,P |
| `Tiny Chest` | Equip0;fourth EventHelper roll resets0/forces TREASURE before chance updates;roll still consumed. | G,H |
| `Toy Ornithopter` | Potion use heal5 queued in COMBAT,immediate otherwise. | R,X,H |
| `Vajra` | Battle-start Strength1 queued. | X,P |
| `War Paint` | Equip canUpgrade Skill-instance shuffle;first≤2 upgraded/bottle links refreshed,duplicate kinds allowed. | H,G |
| `Whetstone` | Same prefix for canUpgrade Attack instances. | H,G |
| `Blue Candle` | Base Curse legality exception;onUse queues HP_LOSS1 and marks card/cleanup exhaust,not all-permission bypass. | X,P |
| `Bottled Flame` | Delayed eligible Attack selection/flag;opening top-placement shared with initializeDeck,not flash. | H,X,G |
| `Bottled Lightning` | Skill bottle;spawn nonbasic filter differs from equipment's typed purgeables. | H,X,G |
| `Bottled Tornado` | Power bottle;unequip clears selected permanent flag. | H,X,G |
| `Darkstone Periapt` | Actual CURSE-color obtain maxHP+6/helper heal,not prevented offer. | R,H |
| `Eternal Feather` | RestRoom entry heal3×floor(deck-size/5),even without legal/chosen Rest. | R,H |
| `Frozen Egg 2` | Preview/obtain canUpgrade unupgraded Power upgrades;equip revisits existing reward cards. | H,P |
| `Gremlin Horn` | onMonsterDeath currentHP0 and group not basically-dead queues energy1 then draw1;no final-group payoff. | R,X,P |
| `HornCleat` | Battle counter0;second turn Block14 once;victory clears spent/sets−1. | R,X |
| `InkBottle` | Persistent tenth onUse resets0/draw1;battle restores armed display9. | X,P |
| `Kunai` | Turn counter;each third Attack onUse resets/queues Dex1;victory−1. | X,P |
| `Letter Opener` | Each third turn Skill onUse queues all-enemy THORNS5/reset;victory−1. | X,P |
| `Matryoshka` | Two nonboss chest charges;before base relic add common/uncommon reward using strict.75 branch,then mark used when exhausted. | H,G |
| `Meat on the Bone` | endBattle before ordinary victory hooks:living HP≤integer half heals12. | R,X |
| `Mercury Hourglass` | Turn-start all-enemy THORNS3 queued. | X,P |
| `Molten Egg 2` | Attack version of Egg guard;already-upgraded Searing Blow gains no new acquisition upgrade. | H,P |
| `Mummified Hand` | Power onUse selects hand instance combat>0,turn>0,not free,not queued;immediate turn0;empty no-op. | X,G,P |
| `Ornamental Fan` | Each third turn Attack onUse resets/queues Block4;victory−1. | R,X |
| `Pantograph` | If any BOSS actor,battle start queues one heal25 then returns,not per actor. | R,X |
| `Paper Frog` | E.1 enemy Vulnerable NORMAL1.75. | X,P |
| `Pear` | Equip maxHP+10/helper heal. | R,H |
| `Question Card` | Reward-count callback+1 only actual helper callers. | H,G |
| `Self Forming Clay` | Each positive combat wasHPLost queues NextTurnBlock3;power later queues gain/removal. | R,X,P |
| `Shuriken` | Third turn Attack onUse resets/queues Strength1;victory−1. | X,P |
| `Singing Bowl` | Enabled card screen maxHP+2 then heal2/exact associated row removal;§8.4,not discovery. | R,H |
| `StrikeDummy` | Damage modifier+3 for STRIKE tag,not name substring/all Attacks. | X,P |
| `Sundial` | Persistent equip0;third actual onShuffle resets/queues energy2. | R,X,G |
| `The Courier` | Shop.8 multiplier and source-specific restock;room hook presentation. | H,G |
| `Toxic Egg 2` | Skill Egg;preview and actual obtain both guarded against already upgraded. | H,P |
| `White Beast Statue` | E.1 appearance override before row suppression/history,not claim guarantee. | H,G |
| `Bird Faced Urn` | Power onUse top-queues heal2. | R,X |
| `Calipers` | E.1 conditional Block loss15. | R,X |
| `CaptainsWheel` | Battle counter0;third turn Block18 once;victory clears spent/counter. | R,X |
| `Champion Belt` | Player-source/nonself Vulnerable ApplyPower without target Artifact notifies before insertion,queues Weak1. | X,P |
| `Charon's Ashes` | Actual exhaust top-queues all-enemy THORNS3,no Branch basically-dead guard. | X,P |
| `Dead Branch` | Actual exhaust while group not basically-dead samples identity now,bottom hand generation later. | X,G,P |
| `Du-Vu Doll` | Equip/deck-change recount CURSE-type permanent instances;positive cached count battle-start Strength. | H,X,P |
| `FossilizedHelix` | Battle-start Buffer1;entry clears display,payload is prevention not resurrection. | X,P |
| `Gambling Chip` | Rearm pre-draw;first postdraw consumes activation/queues any-number manual discard/redraw. | X,P |
| `Ginger` | E.1 player Weak prevention before Artifact. | X,P |
| `Girya` | Lift usable persistent count<3;effect increments,not option construction;battle queues count Strength if nonzero. | H,X,P |
| `Ice Cream` | E.1 recharge carry with actual energy ceiling999,prep reset. | R,X |
| `Incense Burner` | Persistent equip0;sixth turn resets/queues Intangible1;−1 advances1. | X,P |
| `Lizard Tail` | Unused−1 damage rescue after Fairy;sync heal max(1,halfmax),mark−2,real modifiers. | R,X |
| `Magic Flower` | Heal amount rounds×1.5 only live room COMBAT,then power modifiers/cap;can include endBattle healing before phase change. | R,X,P |
| `Mango` | Equip maxHP+14/helper heal. | R,H |
| `Old Coin` | Equip gainGold300,actual guards apply. | R,H |
| `Peace Pipe` | Add Toke enabled for nonbottled purgeables;selected object removal hooks. | H |
| `Pocketwatch` | Reset battle count/first-turn;onPlay counts,later postdraw≤3 queues draw3 then clears;no first-turn bonus;victory−1. | X,P |
| `Prayer Wheel` | Second card offer ordinary MonsterRoom only,not elite/boss/EventRoom;later offer sees earlier history. | H,G |
| `Shovel` | Add Dig;effect generates actual relic,not option constructor. | H,G |
| `StoneCalendar` | Battle counter0;turn increments;end exactly7 queues all-enemy THORNS52;victory reset−1. | X,P |
| `Thread and Needle` | Battle-start PlatedArmor4 queued. | X,P |
| `Torii` | Owner-present non-THORNS/non-HP_LOSS amount2..5→1 before Tungsten final-loss stage. | X,P |
| `TungstenRod` | Final positive loss−1 including applicable HP_LOSS,not once per turn. | X,P |
| `Turnip` | E.1 player Frail prevention before Artifact. | X,P |
| `Unceasing Top` | Refresh empty hand/pending actions,active turn/combat,no screen/NoDraw/temp-disable,draw or discard nonempty→draw1. Prebattle disable,turn reenable. | X,P |
| `WingedGreaves` | Three charges permit otherwise-unconnected same-next-row;selecting such edge consumes,normal edge free,last−2. | H,V |
| `Astrolabe` | Purgeable≤3 automatic,>3 select3,zero no transform;each object removal before upgraded transform/queued obtain. | H,G |
| `Black Blood` | Boss instant slot0 replacement;living victory heal12;canSpawn requires Burning Blood absent after starter-removing Neow swap. | R,H,X |
| `Black Star` | Extra noncampfire relic reward actual MonsterRoomElite only,not EventRoom eliteTrigger. | H,G |
| `Busted Crown` | Equip/unequip master energy±1;actual reward-count helper−2. | R,H,G |
| `Calling Bell` | Fixed curse confirmation then screen-down once-only screenless C/U/R offers;prevented curse does not suppress relics. | H,G,X |
| `Coffee Dripper` | Master energy±1;disable exact RestOption,not room-entry heals/Recall. | R,H |
| `Cursed Key` | Master energy±1;nonboss chest-open queues random curse,bossChest none;skipping chest no open hook. | R,H,G |
| `Ectoplasm` | Master energy±1;gainGold early return prevents actual gain and its callbacks,not generating offer. | R,H |
| `Empty Cage` | Set room incomplete;zero completes internal latch only;≤2 automatic object removal,else grid2;actual delete completes room. | H,X |
| `Fusion Hammer` | Master energy±1;disable ordinary SmithOption,not every upgrade source. | R,H |
| `Mark of Pain` | Master energy±1;queue two Wounds in combat draw pile with actual insertion law,not hand/permanent deck. | R,X,G,P |
| `Pandora's Box` | Iterator removes starter-tagged Strike/Defend,one replacement/preview per input,positive-count confirmation later Fast obtains;no object-removal hooks. | H,G,X |
| `Philosopher's Stone` | Master energy±1;initial and spawned monsters immediately Strength+1/modify notification,including split children. | R,X,P |
| `Runic Cube` | Every positive combat wasHPLost top-queues draw1. | X,P |
| `Runic Dome` | Master energy±1;suppress visible intents/reconstructive fields,not AI selection/actions. | R,V |
| `Runic Pyramid` | E.1 suppress bulk discard/selection,not Ethereal/retain distinction. | X,P |
| `SacredBark` | Equip reinitializes held potion data;potency helper doubles plus explicit choice/duplication paths,not double-all-fields. | H,P |
| `SlaversCollar` | Explicit beforeEnergyPrep eliteTrigger/BOSS temporarily master energy+1;activated victory removes;no general equip+1. | R,X |
| `Snecko Eye` | Master draw±2;prebattle Confusion queued;draw mutation not global initial rewrite. | X,G,P |
| `Sozu` | Master energy±1;reward can consume without obtain,shop refuses unpaid,low-level obtain has no universal Sozu gate. | R,H,P |
| `Tiny House` | One upgrade prefix,maxHP+5,**50 base-gold reward and one potion offer**,then ordinary-context card reward. Not instant gold/potion gain or forced boss-combat rare;bonus/claim/guard separate. | R,H,G |
| `Velvet Choker` | Master energy±1;battle/turn0,onPlay increments≤6,permission rejects≥6;victory−1,autoplay actual gate applies. | R,X,P |
| `Brimstone` | Each turn top-queue player Strength2 and monsters1;top insertions reverse encounter producer order. | X,P |
| `Cauldron` | Five flat-player-pool potion offers;remove auto card row after its generation/history. | H,G,P |
| `Chemical X` | E.1/§4.1.2 ten current count/capture/live-payment laws,not ordinary Brilliance legacy-name inference. | X,P,G |
| `ClockworkSouvenir` | Battle-start Artifact1 queued. | X,P |
| `DollysMirror` | Permanent selected stat-equivalent copy,clear bottles,scheduled obtain;selection complete not insertion. | H,X,P |
| `Frozen Eye` | E.1 copied view ordering/public knowledge,not hidden next-shuffle knowledge. | V |
| `HandDrill` | Damage exhausting positive monster Block,including exact equality,queues Vulnerable2;not Calipers/generic loseBlock or HP_LOSS bypass. | X,P |
| `Lee's Waffle` | MaxHP+7 includes associated helper heal,then new-max-sized heal request;false argument does not disable helper healing. | R,H |
| `Medical Kit` | Otherwise-unplayable Status legality exception;onUse marks exhaust,does not convert type to Skill. | X,P |
| `Membership Card` | Shop.5 discount immediately reprices remaining stock;room callback presentation only. | H |
| `OrangePellets` | Reset type flags each turn;onUse completes Attack/Skill/Power triple→queue RemoveDebuffs/reset immediately,can repeat triple. | X,P |
| `Orrery` | Four explicit plus automatic fifth card offer,changing history/five separate choices,not one five-card menu. | H,G |
| `PrismaticShard` | Non-Defect with zero master slots gains one on equip;reward colors widen,not generation/merchant/transform character authority. | H,G,P |
| `Sling` | eliteTrigger battle start Strength2 queued,not restricted to actual elite-room type. | X,P |
| `Strange Spoon` | E.1 one conditional cleanup Boolean with actual ordinary destination precedence. | X,G |
| `TheAbacus` | Actual onShuffle bottom-queues Block6,not arbitrary movement. | R,X,G |
| `Toolbox` | Pre-draw choose-one colorless action queued;choice remains required work. | X,G,P |
| `NeowsBlessing` | Three initial battle charges set initial current monster HP1,maxHP unchanged;last used−2. | X,P |
| `Golden Idol` | E.1 stored qualified gold reward bonus,not universal gain modifier. | R,H |
| `Odd Mushroom` | E.1 player Vulnerable NORMAL1.25. | X,P |
| `WarpedTongs` | Postdraw random canUpgrade non-Status hand shuffle/first upgrade,empty no-op;temporary instance,not permanent upgrade. | X,G,P |
| `Spirit Poop` | E.1 full-game score−1,not Act1 reward;ordinary ownership remains. | R,V |
| `FaceOfCleric` | Victory maxHP+1/helper heal. | R,X |
| `CultistMask` | Inspected primary speech/sound actions only,no numeric buff;ordinary ownership still present. | X |
| `GremlinMask` | Battle-start Weak1 through ApplyPowerAction,Ginger before Artifact. | X,P |
| `NlothsMask` | One nonboss chest-after-open charge removes first relic reward/linked row,then−2;Matryoshka/base ordering matters. | H |
| `SsserpentHead` | Face Trader ownership:actual EventRoom entry gainGold50,not Ssssserpent reward. | R,H |

Relic-specific remaining payload endpoints include Vigor/Nib/NextTurnBlock;
Strength/Dexterity/damage/Block pipelines;debuff stacking/Artifact/direct applies;
Thorns/Buffer/PlatedArmor/Intangible/Confusion/RemoveDebuffs;draw/discard/exhaust;
orb/Expunger/generated Status/choice/potion/Bark laws. G retains exact upgrade,
pool,room-history,reward-count,merchant/restock,hand-selection/Spoon/Confusion and
unseeded hand-shuffle joint bindings. H retains actual parent/death/exceptional
inventory reachability;V requires no hidden-order/intent/counter leakage. E16/E14/
E19 close their stated sublaws,not the rest of these compositions. Victory-named
hook does not prove win-only guard: endBattle Meat then player victory checks
!isDying;escape/smoke outcome needs actual caller trace,not blanket hook exclusion.

## Appendix F. Enemy A0 local normative records (F; E20)

Scope: standard, unmodded, fully unlocked Ironclad A0 Act1, Act4 available; owned
archive identity and limitation are §1. `U[a,b]` is inclusive uniform integer;
ordinary HP starts current=max. `hit(n)×k` means k distinct NORMAL damage actions,
not one multiplied hit. `+B/+Str` are requested amounts, not bypasses of modifiers,
Artifact, hooks or cancellation. Move IDs are kind-local. Unless stated otherwise,
ordinary next choice is queued RollMoveAction; a random roll may still be consumed
when a selected branch ignores it. Mad/Sneaky repeats instead queue deterministic
SetMoveAction; Fat still rolls. No A17 Frail, extra Gang leader/Minion or later-act
enemy path is imported.

### F.1 All 25 kind records

| Kind / source identity | Ordinary A0 HP and setup | Move-ID / source intent: payload | AI / checkpoint |
|---|---|---|---|
| Cultist / `Cultist` | U[48,54], none | 3 BUFF Incantation Ritual3; 1 ATTACK Dark Strike hit6 | First3, then1; first end-round Ritual suppressed. |
| Jaw Worm / `JawWorm` | U[40,44], none, not hard | 1 ATTACK Chomp11; 2 DEFEND_BUFF Bellow +Str3 then+B6; 3 ATTACK_DEFEND Thrash7 then+B5 | Initial1; finite history law F.2; no Act3 setup. |
| Red Louse / `FuzzyLouseNormal` (class `LouseNormal`) | U[10,15], Curl Up U[3,7], Bite U[5,7] | 3 ATTACK Bite(fixed rolled base); 4 BUFF Grow +Str3 | Nonattack/Bite25/75; third identical selection redirects. |
| Green Louse / `FuzzyLouseDefensive` (class `LouseDefensive`) | U[11,17], Curl Up U[3,7], Bite U[5,7] | 3 ATTACK Bite(fixed rolled base); 4 DEBUFF Spit Web Weak2 | Same finite redirect; no Grow. HP then Bite; Curl Up separate HP-stream prebattle roll after initial intent. |
| Acid Slime S / `AcidSlime_S` | U[8,12], none | 1 ATTACK Tackle3; 2 DEBUFF Lick Weak1 | Initial fair1/2; then alternate1↔2; no Split. |
| Acid Slime M / `AcidSlime_M` | U[28,32], none | 1 ATTACK_DEBUFF Corrosive Spit7 + Slimed1 discard; 2 ATTACK Tackle10; 4 DEBUFF Lick Weak1 | Finite Acid law F.2; split-created uses inherited HP. |
| Acid Slime L / `AcidSlime_L` | U[65,69], Split | 1 ATTACK_DEBUFF Corrosive Spit11 + Slimed2 discard; 2 ATTACK Tackle16; 4 DEBUFF Lick Weak2; 3 UNKNOWN Split→two Acid M | Acid law until living half-HP interrupt; boss-created inherits HP. |
| Spike Slime S / `SpikeSlime_S` | U[10,14], none | 1 ATTACK Tackle5 | Always1; no Slimed/Split. |
| Spike Slime M / `SpikeSlime_M` | U[28,32], none | 1 ATTACK_DEBUFF Flame Tackle8 + Slimed1 discard; 4 DEBUFF Lick Frail1 | Tackle0–29/Lick30–99; third identical redirects; no Split. |
| Spike Slime L / `SpikeSlime_L` | U[64,70], Split | 1 ATTACK_DEBUFF Flame Tackle16 + Slimed2 discard; 4 DEBUFF Lick Frail2; 3 UNKNOWN Split→two Spike M | Same 30/70 redirect until living half-HP interrupt. |
| Blue Slaver / `SlaverBlue` | U[46,50], none | 1 ATTACK Stab12; 4 ATTACK_DEBUFF Rake7 then Weak1 | Rake0–39/Stab40–99; third identical redirects including initial selection. |
| Red Slaver / `SlaverRed` | U[46,50], none | 1 ATTACK Stab13; 3 ATTACK_DEBUFF Scrape8 then Vulnerable1; 2 STRONG_DEBUFF Entangle1 | Initial1; exact execution-set Entangle flag and 45/55 post-Entangle law F.2. |
| Looter / `Looter` | U[44,48], Thievery15, ledger0, attack count0 | 1 ATTACK Mug10 + separate theft15; 4 ATTACK Lunge12 + theft15; 2 DEFEND Smoke+B6; 3 ESCAPE | Two Mugs; then fair Smoke/Lunge; Lunge→Smoke→Escape. Ledger/theft/HP and reward ownership F.4. |
| Fungi Beast / `FungiBeast` | U[22,28], Spore Cloud2 | 1 ATTACK Bite6; 2 BUFF Grow+Str3 | Bite0–59/Grow60–99; third Bite→Grow, consecutive Grow→Bite; final-death guard F.3. |
| Fat Gremlin / `GremlinFat` | U[13,17], none | 2 ATTACK_DEBUFF Smash4 then Weak1 | Repeated ordinary Gang choice rolls; no A17 Frail; escape99 is not ordinary member-death reaction. |
| Mad Gremlin / `GremlinWarrior` | U[20,24], Angry1 | 1 ATTACK Scratch4 | Repeated via SetMoveAction, not RollMoveAction; Angry keys to qualifying damage, not Attack play. |
| Shield Gremlin / `GremlinTsundere` | U[12,15], none | 1 DEFEND Protect+B7 selected ally/self; 2 ATTACK Shield Bash6 | Initial Protect even alone; later count of living/non-escaping actors selects Protect if>1 else Bash. Resolution targeting F.3. |
| Sneaky Gremlin / `GremlinThief` | U[10,14], none | 1 ATTACK Puncture9 | Repeated SetMoveAction; no Thievery ledger; escape99 is not ordinary death broadcast. |
| Gremlin Wizard / `GremlinWizard` | U[21,25], charge1 | 2 UNKNOWN Charging, no damage, increment; 1 ATTACK Ultimate Blast25, reset0 | First two Charging then Blast; thereafter three Charges per Blast; charge3 directly selects Blast. |
| Gremlin Nob / `GremlinNob` | U[82,86], no initial Enrage; ordinary/event Vulnerable enabled | 3 BUFF Bellow Enrage2; 2 ATTACK_DEBUFF Skull Bash6 then Vulnerable2; 1 ATTACK Bull Rush14 | First3; then Bash0–32/Rush33–99; after two Rush force Bash; Bash may repeat. Non-Vulnerable constructor is outside scoped factory. |
| Lagavulin / `Lagavulin` | U[109,111]; ordinary queued Block8 then Metallicize8; event awake has Block0/no Metallicize | 5 SLEEP, no hit; 4 STUN, no hit; 3 ATTACK18; 1 STRONG_DEBUFF Siphon Dexterity−1 then Strength−1 | Ordinary sleep/wake and event Siphon-first F.3; natural third sleep queues wake/opening+Attack selection for next action. |
| Sentry / `Sentry` | U[38,42] each, Artifact1 each | 3 DEBUFF Bolt Dazed2 discard; 4 ATTACK Beam9 | Ordered three-actor factory opens Bolt/Beam/Bolt; each actor alternates independently, deaths do not renumber phase. |
| Guardian / `TheGuardian` | 240; Mode Shift30, loss0, open, latch false | 6 DEFEND Charging Up+B9; 2 ATTACK Fierce Bash32; 7 STRONG_DEBUFF Vent Steam Weak2 then Vulnerable2; 5 ATTACK Whirlwind5×4; 1 BUFF Defensive Mode Sharp Hide3; 3 ATTACK Roll Attack9; 4 ATTACK_BUFF Twin Slam8×2 then remove Sharp Hide | Initial6→2→7→5→6; threshold replaces pending offensive move with1 when queued change resolves; then1→3→4→5→6. Quota/Block timing F.5. |
| Hexaghost / `Hexaghost` | 250; no starting power, inactive, future-Sear flag false | 5 UNKNOWN Activate captures Divider base floor(H/12)+1; 1 ATTACK captured base×6; 4 ATTACK_DEBUFF Sear6 + Burn1 discard (Burn+ after Inferno); 2 ATTACK Tackle5×2; 3 DEFEND_BUFF +B12 then +Str2; 6 ATTACK_DEBUFF Inferno2×6 | Activate→Divider once, then Sear→Tackle→Sear→Inflame→Tackle→Sear→Inferno repeat; actor orb-count state. Capture/Burn checkpoints F.5. |
| Slime Boss / `SlimeBoss` | 140, Split | 4 STRONG_DEBUFF Goop Spray Slimed3 discard; 2 UNKNOWN Preparing; 1 ATTACK Slam35; 3 UNKNOWN Split→Spike L left, Acid L right | Initial4→2→1→4 until living HP≤70 schedules Split; children inherit split-time HP as current/max. |

### F.2 Conditional local selection laws (finite, not renormalized IID)

Let r be uniform integer0–99, primary mass=count/100; `last(X)` and `two(X)` are
last one/two selected move IDs. Chance-float comparisons use `k/2^24 < binary32(p)`
on k=0..16777215. Exact accepted counts: qC=.5625→9437184; qT=.357→5989466;
qB=.416→6979322; qA=.4→6710887.

| Actor and state | Finite conditional law |
|---|---|
| Jaw Worm, first | Chomp regardless r. Later: r0–24 Chomp unless last Chomp, then Bellow with qC else Thrash; r25–54 Thrash unless two Thrash, then Chomp with qT else Bellow; r55–99 Bellow unless last Bellow, then Chomp with qB else Thrash. After Chomp masses (C,T,B)=(0,.30+.25(1−qC),.45+.25qC); after Bellow=(.25+.45qB,.30+.45(1−qB),0); after one Thrash=(.25,.30,.45); after two Thrashes=(.25+.30qT,0,.45+.30(1−qT)). Post-Chomp Bellow=.590625. |
| Acid M/L | r0–29 Spit unless two Spit, then fair Tackle/Lick; r30–69 Tackle unless last Tackle, then Spit with qA else Lick; r70–99 Lick unless two Lick, then Spit with qA else Tackle. After Tackle Spit=.30+.40qA, not 46/54 or 50/50. Split is separate interrupt. |
| Louse / Blue Slaver / Spike M/L / Fungi / Nob | Louse primary nonattack/Bite1/4–3/4; two same redirects to other. Blue Rake2/5–Stab3/5; Spike Tackle3/10–Lick7/10; Fungi Bite3/5–Grow2/5; two of either redirects to other. Nob after Bellow Bash33/100–Rush67/100; two Rush forces Bash, no Bash repeat restriction. |
| Red Slaver | First Stab and clear first-selection flag. Before Entangle used: r75–99 Entangle; r0–74 Scrape unless two Scrape, then Stab. `usedEntangle` flips in Entangle takeTurn, not selection or successful debuff. Afterwards unrestricted law is r55–99 Stab45/100, r0–54 Scrape55/100; two Stab forces Scrape, two Scrape forces Stab. No new Entangle and no reversed55/45. |

### F.3 Reactions, wake and split checkpoints

Curl Up once: positive NORMAL damage, nonnull owner, damage strictly below pre-subtraction
HP; latch immediately, closing/Block/removal bottom-queued. Fully blocked, zero, other
type and lethal hit do not trigger; it is not synchronous protection against queued later
hits. Mad Angry1: positive owner-attributed damage except HP_LOSS/THORNS queues +Str1
top; no surviving-hit requirement at power, later dead-target application may reject.
Nob Enrage2: qualifying Skill onUse callback queues +Str2 top once Enrage exists;
UseCardAction dontTriggerOnUseCard gate matters. Guardian Sharp Hide3: qualifying Attack
callback appends player THORNS hit3 bottom; multihit is one callback, repeats may callback
separately. Fungi Spore Cloud2 checks battle-ending status before queueing Vulnerable2
top with null source; final basically-dead group suppresses it, including Thorns kills.
Cultist Ritual3 skips first end-round callback, later queues +Str3. Shield Protect chooses
uniformly eligible actor instances at action resolution excluding self/dying/ESCAPE;
empty set→self; does not separately exclude halfDead/isEscaping. Intent choice is separate:
count non-dying/non-escaping actors>1 keeps Protect else Bash. Wizard starts charge1;
Charging increments, reaching3 selects Blast; Blast resets0 before selecting Charging.
Gremlin escape99 queues EscapeAction, but bounded direct caller scan found broadcast outside
ordinary Act1 roots, no direct escapeNext call, no Gang leader/Minion; no ordinary member
kill flight broadcast. This caller boundary is not reflection/native closure.

Ordinary Lagavulin setup Block8 then Metallicize8. First two sleeping actions no attack;
third also no hit, queues opening plus Attack selection for following action. Surviving
damage wake via override selects STUN4 and queues opening; fully blocked/zero HP-loss does
not wake. After surviving wake: Stunned→Attack→Attack→Siphon. Awake event has out/trigger
flags, no intrinsic Block/Metallicize, starts Siphon; cycle Siphon→Attack→Attack→Siphon.
Opening guarded for dying actor, reduces Metallicize by8 only and does not synchronously
clear remaining Block. Siphon queues Dex−1 then Str−1 separately; Artifact1 can block the
first, leaving second eligible. Source power notification precedes Artifact; successful
application/stack invokes onModifyPower.

For Slime Boss and both large slimes, local damage override after base damage: non-dying
currentHP≤maxHP/2 selects Split immediately and queues SetMove; large slime latches. For
positive integer HP this is ≤floor(max/2); lethal damage already marks dying and suppresses
live split. Any damage type can qualify locally, but each producer must enter override.
Split takeTurn constructs children using then-current parent HP for both current/max. Boss:
Spike L left, Acid L right; large slime two same-type M left then right. No M→S. Children
fresh identity/powers, zero inherited Poison, non-Minions; first AI is ordinary initial.
Cannot-lose brackets suicide/spawn; die(false) suppresses base onDeath/relic onMonsterDeath,
so replacement is not attributed kill/Fatal. Spawn relic callbacks precede child init,
power application and positional insertion; Philosopher's Stone is concrete +Str1. Battle
start callbacks do not universally rerun. Group action queue is snapshot: children do not
act in already-built round queue. Actual child kills can qualify for their hooks; Fatal/
Poison/multitarget/rescue arbitration stays B04/B07/B12.

### F.4 Looter transaction and escape boundary

First Mug; first selects another Mug; during second Mug takeTurn fair chance-float .5 selects
Smoke or Lunge. No later repeat choice. Lunge→Smoke→Escape. Smoke queues Block6 then
EscapeAction, which sets room mugged flag, not die. Each Mug/Lunge queues separate ledger
add `min(15, player gold at ledger resolution)`, then gold-carrying DamageAction. Damage
initial update subtracts up to15 current gold directly; later timed completion applies HP
hit. Ledger increment, gold decrement and HP hit are three commit points, theft does not
require Block penetration/positive HP damage and bypasses loseGold callbacks. Ordinary queue
has no demonstrated player interposition. Death's STOLEN_GOLD reward uses ledger, not later
recomputed loss; not instant restitution. Golden Idol does not multiply it; claim uses
gainGold restrictions. Escaped Looter has no refund/Fatal credit. Common ordinary gold
suppression asks all retained monsters escaped; mixed Thugs with killed survivor differs.
Escape/Smoke is not player victory/death or drain-all-work guarantee. Timer/FAST_MODE/Reaper
and legal asynchronous interposition remain B12/B04.

### F.5 Boss, factory, burning, intent/payload and fixed Status facts

Guardian's open, unlatched surviving HP decrease contributes to initial quota30; Blocked
damage contributes0. Threshold hit resets accumulator/latches/appends visual/change-state
work bottom; no immediate Block or published Defensive Mode. Defensive change appends Mode
Shift removal and Block20, next quota+10, selects/publishes Defensive Mode1 and closes actor.
Earlier queued damage may precede Block; excess not carried; later quotas30,40,50,… with no
cap at50. Defensive turn appends Sharp Hide3 and selects Roll Attack; Roll→Twin Slam. Slam
appends offensive change, two hit8 actions, then Sharp Hide removal. Offensive change
appends new Mode Shift/reset and residual-Block loss behind already queued hits.

Hexaghost Activate captures current player H and sets Divider base floor(H/12)+1 before
selecting Divider (H=11/12/23/24→1/2/2/3). Do not recapture at resolution; Strength/other
combat modifications remain separate. Inferno queues six hit2 actions before BurnIncrease,
sets future-Sear flag. BurnIncrease upgrades existing draw/discard Burns only (including
traveling members), not hand/exhaust/limbo; later gate creates/inserts three Burn+. Pending
future producers are not existing cards; interruption at each checkpoint is durable.

Factories, ordered construction and relative weight:

| Pool/key | Weight | Construction law |
|---|---:|---|
| Weak Cultist / Jaw Worm / 2 Louse / Small Slimes | 2 each | One Cultist; non-hard Jaw; two independent fair red/green slots; fair SpikeS+AcidM or AcidS+SpikeM left/right. |
| Strong Blue Slaver / Looter / Large Slime / 3 Louse / 2 Fungi | 2 each | One Blue; one Looter; fair AcidL/SpikeL; three ordered fair-color Louses; two Fungi. |
| Strong Red Slaver | 1 | One Red Slaver. |
| Strong Gremlin Gang / Lots of Slimes | 1 each | Gang successive uniform remaining-member indices: Mad×2,Sneaky×2,Fat×2,Shield×1,Wizard×1; draw4, sample order slots, no final shuffle. Lots exhaust SpikeS×3/AcidS×2 through successive uniform remaining members. |
| Strong Exordium Thugs / Wildlife | 1.5 each | Thugs left Louse colors each1/6, SpikeM/AcidM each1/3; right Cultist/Looter each1/3, Blue/Red each1/6 via separate category/subtype. Wildlife left fair Fungi/Jaw, right Thugs left weak-wildlife law. |
| Elite Nob / Lagavulin / 3 Sentries | 1 each | Vulnerable-enabled ordinary Nob; asleep Lagavulin; ordered Sentries Bolt/Beam/Bolt. |
| Boss Guardian / Hexaghost / Slime Boss | — | One of each boss kind. |

Gang samples remaining members, not equiprobable kinds or replacement; Lots is not IID
types. Category-then-subtype is not flat kind mass. Candidate helpers may construct before
category selection; discarded construction incidental RNG is not required seed parity.
Ordinary queue starts 3 weak, first-strong, then12 further strong (16); elite list10.
Ordinary append retries original weighted proposal until differs from last2; elite rejects
only preceding elite. First-strong separately rejects 3 Louse after weak 2 Louse, and Large
Slime/Lots after Small Slimes; no ordinary last-two filter there. Unreachable leftover
Looter/Blue exclusions are not promoted to reachable law. Weights sort ascending stably;
binary32 total/cumulative strict cutoff on 24-bit draw. Weak total8, strong total16 exact
fractions; eligible repeat rejection yields p(i)/sum eligible, not new uniform. Elite
Nob/Lagavulin/Sentries finite counts5592406/5592405/5592405 of16777216, not equal thirds.
MonsterRoom departure consumes ordinary head; MonsterRoomElite departure consumes elite
head. Question room as MonsterRoom uses ordinary route; event eliteTrigger does not consume.
Boss discovery first unseen Guardian→Hexaghost→Slime; all-seen source Java shuffle seeded
monsterRng (singleton duplicated), boss entry removes; seen at prebattle, not announcement/
victory. Initialized/history-conditioned shuffle measure remains B03.

Event variants: Dead Adventurer selects event-owned 3 Sentries/Nob/awake Lagavulin by U[0,2],
not ordinary elite queue. Sentries ordered phases; Nob Vulnerable-enabled; event Lagavulin
Siphon first with Block0/no Metallicize. Dead Adventurer shuffles GOLD/NOTHING/RELIC once via
miscRng-seeded Java Random, but no uniform permutation/joint law claimed; search risks25/50/75,
Leave between searches, three successes end search. Event ordinary gold U[25,35], adds
unsearched gold30/relic categories before combat, suppresses searched duplicate; eliteTrigger
allows Insect/Sling/Collar, not MonsterRoomElite.dropReward, Black Star extra relic or burning
room setup. Mushrooms: three Fungi, gold U[20,30] plus Odd Mushroom (Circlet if owned); Odd
Mushroom is loot, not in-fight protection. Eat noncombat heals trunc(.25×maxHP), then requests
Parasite. Prayer Wheel extra ordinary-MonsterRoom offer does not follow event combat. Scrap
Ooze is not enemy/factory. Other event damage is not a monster Attack or queue consumer.

For marked elite with Final Act available, one U[0,3] to every actor: Strength+2; maxHP bonus
round(binary32(.25×old maxHP)) then heal bonus; Metallicize4; or monster Regeneration3. Bonus
rounding is nearest (halves up) for these HP values, not truncate(1.25×HP). Prebattle ordinary
setup queues before burning; combat-wait applies burn before battle-start relics. Preserved
Insect then caps current HP to trunc(binary32(.75×new maxHP)), no maxHP change or local min1;
Sentry38→max48/current36, 39→max49/current36. Lagavulin Metallicize8+4, opening reduces8,
leaves4. Metallicize pre-end-turn hook queues equal Block; monster Regeneration queues Heal3
at end turn when not halfDead/dying/dead, does not decrement amount. Elite reward adds relics,
then offers Emerald Key iff Final Act available, absent key, marked room and nonempty rewards.
Offer, choose and durable possession differ; obtain-key effect commits; skip/defer preserved.
Event elite identity does not imply marked-room reward. Empty map marker has no root witness;
see B02, no retry/forced placement/defeat.

Intent preview is distinct from payload: selected move contains kind-local ID/category/base/
multihit; createIntent publishes and computes preview, applyPowers separately recalculates.
Scoped preview processes attacker give, target receive, stance, final give/receive in source
list order with binary32 intermediates, then floor/nonnegative clamp. It is not every target
hook. takeTurn queues DamageActions with DamageInfo references, not copied displayed intentDmg;
later target.damage sees then-current Block/reaction/death. Neither immutable preview-as-hit
nor universal per-hit recalculation may be assumed. Dome suppresses visible intent/derived
preview, not already observed move; no leak through menu/legality.

| Fixed enemy Status/mutation site | Requested output / destination / checkpoint |
|---|---|
| Sentry Bolt | Dazed2→discard |
| Acid M Spit | Slimed1→discard after hit queued |
| Acid L Spit | Slimed2→discard after hit queued |
| Spike M Flame Tackle | Slimed1→discard after hit queued |
| Spike L Flame Tackle | Slimed2→discard after hit queued |
| Slime Boss Goop | Slimed3→discard, no hit |
| Hexaghost Sear | Burn1→discard, Burn+ only after own Inferno flag; after hit queued |
| Hexaghost Inferno creation | Burn+3→discard at later BurnIncrease gate after six hits queued |
| Hexaghost Inferno mutation | Upgrade existing draw/discard Burns; no obtain/draw event |

These are fixed Status producers, not character/Shard pools or permanent deck acquisition.
Dazed Ethereal; Slimed cost1 Status; Burn/Burn+ end-turn2/4 damage. Creation is not draw or
Skill play. Named small-count discard commits pile membership at effect construction; later
visual soul travel is not second insertion. It does not prove all callbacks finish on battle
end. Enemy source applies signed32 storage fields and signed-byte move IDs; do not import Java
overflow as gameplay or invent bounds beyond actual helper laws. StrengthPower constructor,
stack and reduce clamp resulting amount[−999,999] after arithmetic; AbstractCreature.addBlock
floors processed gain, adds and caps currentBlock999 for all creatures. This is not universal
minimum-zero, direct-write or all-counter cap. Native allocation/numeric exhaustion is admin,
not defeat or dropped damage.

EN-U1–EN-U6 remain explicit: mixed origin/cost/copy and transitive Enrage/Hide/Curl/Angry/
Poison/Fatal callbacks; full prevention/heal/lethal/terminal and all work-container ordering;
original timer/FAST_MODE/Reaper and legal asynchronous theft interposition; nested reward/
obtain/key durable continuation; initialized/history-conditioned Java shuffle joints; and
zero-Elite root witness or exclusion. No runtime/gameplay acceptance ran; all T cases remain
future discriminators, and this appendix is not an all-transitive closure claim.

## 18. Draft handoff status

This integration draft supplies whole-scope requirements and bounded reviewed E13–E28
advances, not closed mechanics/API evidence or a human-approval-ready rulebook.
The former phantom ordinary Show-curse obligation is superseded; accepted paired
primary records and local numeric tables are integrated. Remaining root/history
measures, catalog/joint laws, exceptional reachability, transitive/source composition,
numeric domains, timer/profile/handoff interfaces and schema reconciliation remain required.
Concrete native contracts and all141 local relic laws do not erase those gaps. B01–B14 remain
open at their narrowed boundaries; D01–D08 remain unapproved recommendations or
choices for main's readback; T01–T34 are unexecuted. The author may correct this
document only through bounded main-directed follow-ups. Independent review,
human whole-spec approval, coordinated implementation-plan review and separate
start authorization remain distinct. No PR readiness, merge or parent-task closure
is implied by publishing this draft.
