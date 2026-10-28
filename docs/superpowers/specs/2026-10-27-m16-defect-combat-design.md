# M16 Defect A0 combat mechanics design

Status: planning proposal; one independent planning review returned READY.
Same-reviewer acknowledgment of these documentation corrections and actual
primary-session user approval remain pending. No implementation or milestone
acceptance is implied.
Planning Bead: `spire-dxo.38.1`; roadmap outcome: `spire-dxo.38`.
Read baseline: `e5b07da5609a3a2654b3bced3f8388ea96324a8f`.

## Outcome and approach

Deliver the [M16 roadmap outcome](2026-10-22-narrow-gym-milestones-design.md#defect-milestones1620-a0)
on the existing authoritative Rust engine: four orb kinds, channel/passive/evoke,
Focus, capacity changes through the real ten-slot cap, representative real power
and item interactions, combat-persistent mutations and Genetic Algorithm's
permanent learned Block. Demonstrate actual authored fights, canonical Defect
starting resources, consecutive combat ownership and a real Death. This is a
finite native combat profile, not normal-start Act1 traversal or complete Defect.

| Approach | Benefit | Cost / decision |
|---|---|---|
| Extend shared orb, card-instance, item and Run owners with a finite representative profile | Makes each promised mechanic executable and preserves accepted behavior | Requires explicit callback and permanent-misc integration. Recommended planner choice. |
| Implement all 75 blue cards and exclusive items now | Earlier character-wide coverage | Pulls generators, selections and M20 breadth into M16; unnecessary for this gate. |
| Add only Dark/Plasma metadata and fixture-only slot changes | Small patch | Does not deliver real producers, power interactions or persistent mutation; rejected. |

The user selected one shared Rust engine. The finite roster and interface details
below are planner recommendations, not claims of human approval. Planning now was
explicitly requested; no extra methodology selection, product work, commit or
publication follows from preparing this package.

## Present implementation and source grades

Source inspection at the read baseline establishes (payload support is distinct
from numeric public-value support; the existing `cards::public_values` dispatch
currently handles Ironclad/Silent, so Task1 also fills real blue values):

| Area | Actual state | Owner / gap |
|---|---|---|
| Lightning/Frost | Real passive/evoke; channel and eviction; shared Orb damage / direct Frost Block | `combat/characters.rs::apply/invoke`; preserve and extend |
| Focus | Stored and used in orb arithmetic, but public producer validation admits Shield Bash -1 only; adjustment requires occupied orb | `combat/engine.rs::validate_effect`, `characters.rs::apply_focus`, `powers.rs`; genuine gap for empty/zero slots and positive producers |
| Capacity/Barrage | Three starting slots, invariant <=10; Barrage already counts occupied slots, with >3 and sparse-ten regressions | `cards/mod.rs::apply_defect_continuation`, `tests/full_cards.rs`; no capacity-changing producer exists |
| Dark/Plasma/powers | No native orb variants or blue Power producers | Definitions alone will not close these gaps |
| Permanent identity | Permanent origin, copy families, temporary upgrade census and Parasite gain journal exist | `combat/card_state.rs`, `run/combat.rs`; no Run misc field or Genetic mutation journal |
| Native transport | Run encoder has two orb kinds; Gym66 sidecar omits orb group and has capacity1; policy-v3 is M10 Ironclad only | `spire-env/src/run_observation.rs`, `combat_policy.rs`; require explicit opt-in extension |

The [Defect paired records](../../research/2026-10-19-act1-content-defect.md)
supply local card/orb/mutation rules from owned static bytes, with explicit §9
composition residuals. [Data guides](../../../data/README.md) retain community and
supporting-static grades; rows are not implementation or runtime proof. The owned
archive SHA-256 is `cfad868ac8d65a88e71a0bf096fb09f78811e553effe0787c5309a655e081673`;
its upstream build/channel is unverified. Named static follow-ups qualify the
callback cases in the companion evidence; no game runtime is invoked. Do not
publish source/bytecode or claim exact original-runtime/seed parity. RNG-001 and
TIME-001 remain in force.

The [coverage ledger](../../milestones/m16-defect-combat-coverage-ledger.md)
records implemented/partial/missing coverage and the named static checkpoints.

## Exact finite content and exclusions

`content/defect.rs` (new) owns `M16_DEFECT_CARDS: &[CardId]` and
`supports_permanent(CardId) -> bool`, `supports_generated(CardId) -> bool`.
These are admission facts, not another rules dispatch or a standard blue pool.
The same 29 kinds are admitted permanently and by explicitly requested fresh,
stat-equivalent and same-family copies, in base and once-upgraded forms:

- Existing: StrikeBlue, DefendBlue, Zap, Dualcast, BallLightning, ColdSnap,
  Coolheaded, Barrage, CompileDriver.
- Orb/state: Darkness, Fusion, Capacitor, Consume, Defragment, BiasedCognition,
  Loop, Electrodynamics, Bullseye, Recursion, Fission.
- Callbacks/repeats: StaticDischarge, Storm, Heatsinks, Amplify, EchoForm.
- Mutable cards: GeneticAlgorithm, Claw, SteamBarrier, Streamline.

Append new IDs with explicit stable spellings: `darkness`, `fusion`, `capacitor`,
`consume`, `defragment`, `biased_cognition`, `loop`, `electrodynamics`, `bullseye`,
`recursion`, `fission`, `static_discharge`, `storm`, `heatsinks`, `amplify`,
`echo_form`, `genetic_algorithm`, `claw`, `steam_barrier`, `streamline`.
Use the native `Bullseye` identity for the source LockOn card; power ID is
`lock_on`. Existing supported blue kinds outside these 29 retain their behavior
but are not silently included in this finite profile.

Real item witnesses: CrackedCore (existing), DataDisk, GoldPlatedCables,
EmotionChip; FocusPotion and PotionOfCapacity (proposed native names). Catalog
metadata, exclusive-character admission, active use, counters/reset and inventory
custody must all work. Shared existing items used by tests remain supported by
their established contracts. No random card generator is added; explicit copy
construction does not imply a generator pool. No new event, reward/shop sampler,
Neow/map loop, Act1/2 boss gate, fullitem/full-blue completeness, Python install,
ML, seed search or winning-policy tuning belongs to M16. Full Defect character
coverage is M20; acquisition offers are M17–20's declared producers.

## Authoritative state and rule seams

Keep `combat/characters.rs` the owner; a focused `characters/orbs.rs` may hold the
four-orb operations, without moving Watcher rules or copying the resolver.
Proposed `OrbState { kind: OrbKind, dark_damage: Option<i32> }` replaces kind-only
entries in `CharacterState.orb_slots: Vec<Option<OrbState>>`. Only Dark has a
stored payload, initially6. `OrbKind` appends Dark and Plasma. There is one owned
slot vector, not parallel payload/history arrays. Public projections may expose
kind and payload without exposing resolver identity.

Extend `CharacterEffect` with typed capacity and passive work. Keep existing
channel/evoke/EndTurn operations as adapters. Proposed internal seams:

- `channel(state: &mut State, source: &Source, orb: OrbState, evict: bool)
  -> Result<(), EffectError>`; fresh channel constructs state, Recursion transfers
  the captured object. Full positive capacity queues front evoke then retry;
  zero capacity installs nothing. First empty slot wins. Only successful install
  is a channel; passives/evokes/repeats do not impersonate channels.
- `invoke_passive(state: &mut State, source: &Source, index: usize,
  phase: OrbPassivePhase) -> Result<(), EffectError>`, where phase is Start/End.
- `change_slots(state: &mut State, delta: i32) -> Result<(), InputError>`;
  positive single-slot steps append empty slots through10; negative steps remove
  the last slot without evoke, floor0. No slot operation creates an orb.

Logical front is index0, including an empty front in a sparse diagnostic layout;
front invoke/Recursion does not search ahead for a later occupied orb. Preserve
Barrage's occupied count at its continuation, not vector length or use time.
Compile Driver counts distinct kinds, never distinct Dark payloads: Dark D6 and
Dark D12 together count as one kind.

| Orb | Passive | Evoke / Focus interaction |
|---|---|---|
| Lightning | End: max(0,3+F) orb damage | max(0,8+F); random eligible living target, or all eligible targets with Electro |
| Frost | End: max(0,2+F) Block | max(0,5+F); bypass card Dexterity/Frail calculation, retain ordinary gain hooks |
| Dark | End: add max(0,6+F) to that orb's stored D | Damage D to first source-order minimum-current-HP eligible monster; ties not random; future Focus does not rewrite D |
| Plasma | Start: +1 energy; no natural end effect | +2 energy; Focus-independent |

Use existing card RNG and enemy-life predicates for Lightning, including
half-dead/escaping/dying distinctions. Dark's source-local selector differs and
must not be replaced by the generic random-target filter. Electro changes target
mode, not number of copies of damage. Orb damage uses `DamagePayload::Orb` and
the existing damage owner, not an Attack/Fixed shortcut. Lock-On duration stacks
and expires via shared power lifecycle; multiplier is binary32 ×1.5 then source
integer conversion, independent of duration. Artifact may block the debuff.
Strength/Weak/Wrath/Vulnerable and attack-only counters do not modify orb damage;
shared defensive/death reactions still apply. Keep accepted damage behavior for
old producers and validate new ones by typed source/context.

Both positive and negative Focus changes are allowed without occupied orbs and
at zero slots; zero-slot Bias decay still applies negative Focus. Use one
authoritative Focus value and a consistent public power projection; do not independently add it
to both CharacterState and a full-power amount. Clamp stack/reduction to
[-999,999], remove zero representation, and route negative applications through
Artifact. Consume requests +2/+3 Focus then removes one slot, even at zero.
Defragment supplies +1/+2. Biased Cognition supplies +4/+5 followed by Bias1;
blocked Bias does not revoke the positive Focus. Start-turn Bias reduces by its
stacked intensity. FocusPotion adds2, Capacity potion adds2 slots, doubled by
existing Sacred Bark potency logic when that shared relic is present.

Dualcast evokes twice, removing only on the second; Recursion preserves Dark D.
Fission captures occupied n at action execution; base removes without evoke,
upgrade schedules individual evokes; both subsequently gain n energy and draw n.
Plasma evokes add energy separately. Lethal damage may cancel later work through
the existing terminal owner; do not aggregate or always drain it.

## Callback, numeric and ownership checkpoints

Natural orb phases must join `turns.rs`, `hooks.rs` and item hooks once. Pre-draw
power callbacks, energy recharge, natural start-orb work, draw, end-card cleanup
and natural end-orb work have distinct phases. The exact ordinary-turn
`EnergyManager.recharge` caller remains a named source checkpoint: established
relics/cards/powers/orbs invocation order does not locate recharge. Resolve it
before the normal-turn Plasma implementation/test, keeping the separate initial
combat energy path distinct. Loop invokes front start+end hooks
1/2 times; Darkness+ triggers existing Dark passives after its channel. Cables
adds only its source-defined extra front passive at eligible group call sites;
never globally double `invoke_passive`, which would incorrectly double Loop.
EmotionChip arms on actual positive HP loss and consumes/reset its combat-local
flag at the source-defined player-start callback. Static Discharge instead uses
the earlier onAttacked damage value with nonself nonnull owner, excludes
THORNS/HP_LOSS and can trigger even if later HP-loss reduction prevents loss.
The coverage ledger and retained named-static addendum record the established
placement; an unresolved member chain blocks its affected producer before
implementation, not authorization to guess or open a broad audit.

Storm and Heatsinks observe actual notified Power uses, not installed-count
polling. Storm intensity1 queues Lightning, Heatsinks1/2 queues draw. Amplify1/2
and Echo intensity1 queue same-family autoplay/purge repeats through the shared
play-frame owner. Echo uses the source turn-position/copies guard, Amplify only
nonpurge Power uses; neither recursively repeats a purge copy. Echo's base
Ethereal/upgraded non-Ethereal and Amplify's end-turn expiry are real. Repeats
notify real callbacks and pay through accepted shared cost/repeat semantics;
do not multiply a payload scalar. M15's proposed cost/history work is not an
accepted API: reconcile actual accepted source first and add only the blue
callbacks missing there. This finite roster does not require random generation.

Source arithmetic uses signed Java int32 (including wrapping field mutation),
not invented 999 caps for Dark D, Genetic misc, Claw damage or Steam baseBlock.
Focus has its actual ±999 clamp; added energy caps999; slot cap10 and hand cap10
retain separate meanings. Use signed storage, explicit wrapping field arithmetic
where source int addition applies, and exact binary32 conversion for Lock-On.
Validate stored mutation separately from calculated nonnegative damage/Block.
The old `Orb` payload validator's <=1515 restriction is not a gameplay Dark cap:
replace with producer-specific validation and preserve old producer guarantees.
An unsupported transport/capacity/continuation is a retained operational fault,
not a gameplay loss, truncation, silent clamp or retry. Numeric boundary tests
must assert source-qualified conversion before exposing wrapped payloads; the
static addendum must identify any still-unresolved conversion rather than
claiming arbitrary injected extremes are reachable gameplay.

Persistent card state stays in `combat/card_state.rs` and Run's permanent deck:

- Add signed `misc` (GA default1, others0) to Run `CardInstance`, live card and
  relevant public projections. Add combat-local mutable base Block and combat
  cost fields only where shared accepted state lacks them. Preserve old public
  descriptors by explicit projection; do not serialize new columns into them.
- GA captures old calculated Block, queues growth2/3 to matching permanent family
  and deduplicated combat instances, then gains captured Block. Upgrade does not
  reset misc. Fresh-identity generated/stat copies cannot mutate an unrelated
  master; same-family repeats can. Grow at the action checkpoint, not victory.
- Proposed `PermanentCardMutation { ordinal: u64, instance: CardInstanceId,
  id: CardId, misc: i32 }` journal records absolute values. Reuse the existing
  gain-journal synchronization pattern without merging Parasite admission into
  mutation authority. `Run::sync_permanent_gains` joins mutation preflight/commit
  after each attempted combat action, including accepted-prefix fault and Death.
  Track a distinct applied-mutation ordinal in ActiveCombat; do not replay deltas.
  Preflight identity, kind, contiguous ordinal and all touched owners before
  changing anything. Re-observation/retry/settlement cannot grow twice.
- Settlement's full census still includes hand/draw/discard/exhaust/in-play,
  play frames, queued autoplay, Stasis, played powers and inactive cards. Validate
  original ownership and learned misc; returning the card is not a second growth.
- Claw adds2 to playing card then Claws in hand/draw/discard; no global future
  bonus or master mutation. Steam Barrier loses1 stored baseBlock without a
  floor; upgrade adds2 to current wear. Streamline mutates same-family combat
  cost/turn cost with the source zero-turn-price branch; all reset next combat.

## Native profile, public view and demonstration

Add `M16DefectCombatStart::{Canonical, Supplied(AuthoredRunStart)}` and
`Run::from_m16_defect_combat(start: M16DefectCombatStart, rng: RunRng)
-> Result<Run, RunError>`. Canonical uses Run::new's Defect A0 resources:
75/75HP,99gold, four StrikeBlue, four DefendBlue, Zap, Dualcast, CrackedCore,
three empty potion slots, no keys. Supplied validates exactly this finite
profile's resources and carries a diagnostic label. Both route through
`from_validated_combat_resources`, `enter_owned`, shared `enemies::setup`,
`Combat::enter_with_item_context`, resolver and `settle_combat`. Do not weaken
existing authored constructors or add a RoomScenario/normal act profile.

Expose public Dark stored damage, exact empty-slot positions/capacity, Focus,
mutable visible costs/Block and learned misc; hide draw order, RNG, future enemy
moves, family IDs and ordinal bookkeeping. Extend `run_observation.rs` with
`RunEncodingProfile::M16Defect` over `encode_run_with_schema`, and add
`run_schema_m16.rs` plus explicit `combat_policy.rs` opt-in functions:
`defect_combat_observation_spec(&RunCapacities) -> Result<RunObservationSpec,RunSessionError>`
and `encode_defect_combat(&PublicDecision,&RunCapacities) -> Result<RunObservation,RunSessionError>`.
Schema ID: `spire-gym-m16-defect-combat-observation-v1`; profile ID:
`M16DefectCombat`. Preserve the shared action relationship: action row i is
`PublicDecision.candidates[i]` in its original order, with no filtering,
reordering or paging. The descriptor action space has `n = caps.actions`;
`actions` has that row capacity and `action_mask` shape is `[caps.actions]`.
Mask entries are1 exactly for candidate rows and0 for unused padding; action-row
references use the existing shared descriptor vocabularies and public row
references. This adds no dispatch format, session token or batch API.
Orb columns: kind, dark_damage; vocab empty0/lightning1/frost2/
dark3/plasma4. Known empty slot differs from padding; dark_damage is known only
for Dark. Append `misc` to this profile's permanent/combat-card groups, keep old
schemas unchanged. Use ten orb rows by default; smaller capacity reports an
explicit capacity fault, never drops a high-index orb. Retain the existing
combat-v1, Run-v1/v2, policy-v3, M12–14 descriptors and default sidecar unchanged.
The new misc and Dark payload columns use the full signed int32 domain in
native int64 transport; unknown fields retain known0 rather than fake zero.
No new Python facade or session/scheduler copy is needed or claimed. M18 can
compose this vocabulary/state through its own opt-in profile.

Create native `m16_defect_combat` example with explicit `starter|supplied|all`,
seed7, budget8192. Starter checks canonical resources and plays actual combat;
supplied scenarios demonstrate all four kinds, cap/sparse Barrage, callbacks and
same-instance two-fight learning; include natural Death. Fixed scenarios and
semantic action choices are frozen before execution. No fake combat result,
strategy/seed/resource search or claim that supplied resources were earned.

## Dependency and acceptance gates

| Work | Independence / dependency |
|---|---|
| Owned four-orb state, capacity, local passive/evoke, GA journal | Designable against e5b07da now; extend shared owners rather than wait for imaginary M17 APIs |
| Blue repeats/cost mutation | M15 v3 design/plan is user-approved and execution has started; its source/API remains unaccepted. Consume actual accepted play/cost/history owners after acceptance/source reconciliation |
| Encounter setup and death/HP/gold custody | Task68 PR53 is merged to main and accepted at `d1fa706638a31a854fdcaeed9a9a4642abf716eb`; reconcile that accepted source before integration, never modify its owned tree |
| M17 | Unassigned; canonical normal start, finite admission/offers and Act1 completion remain missing assumptions, not M16 deliverables |
| M18 | Design/plan is now user-approved and uses the same29 IDs/shared admission seam. Execution still requires accepted M16/M17 and root release; this is not M16 approval or an accepted M17 API |

The accepted Task68 read-only export is
`/home/klottick/.local/share/spire-gym/accepted/encounters-d1fa706638a31a854fdcaeed9a9a4642abf716eb`;
root reports its native check as2043 passed/0failed. This is newer dependency
context, not a test run or source-baseline change by this planning task.

Planning may finish now. Implementation requires actual user approval and recorded
accepted upstream identities per the roadmap delivery order; independent mechanics
must not be blocked by a nonexistent M17 implementation. Before writing code,
record M15/task68 acceptance, compare their real source and update only affected
seams/tests. If they differ materially, root resolves scope; do not silently fork
an older implementation or reimplement their shared machinery.

Acceptance requires behavior-focused paired base/upgraded tests, meaningful faults,
one fixed executable authored demonstration, native projection agreement and
one fresh independent final milestone review. Run focused TDD on changed owners,
then one stable integrated workspace check. Known baseline failures retain exact
revision-qualified names/results; no test-count goal or repeated baseline loops.
No Python delivery, original-runtime fidelity, normal-run Act1/2 or M20 completion
claim follows from this gate. See the [implementation plan](../plans/2026-10-27-m16-defect-combat.md)
for concrete tests and five Review Focus classes.
