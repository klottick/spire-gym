# M10 task-local Ironclad and Act4 findings

Date: 2026-10-24. Input to the approved M10 implementation plan, **not** native
mechanics acceptance or original-runtime validation. Scope is the named A0
Ironclad/Act4 prerequisites, not a new archive-wide audit.

## Evidence boundary

Owned archive SHA256:
`cfad868ac8d65a88e71a0bf096fb09f78811e553effe0787c5309a655e081673`.
Its upstream build/channel remains **unverified**. Existing pinned Temurin
21.0.12.1+1 `javap` was used read-only with Java option-injection variables
removed. Exact named ZIP entries were checked for uniqueness. No game class,
initializer, launcher or substituted runtime was executed; no installation or
whole-archive extraction. Raw selected-member outputs remain outside the public
repository. The following are independently worded semantic findings.

Existing [red-card findings](2026-10-19-act1-content-red-colorless.md) and
[M9 ending findings](2026-10-23-m9-act3-ending.md) retain their qualifications.
New findings below do not establish arbitrary composed callback behavior merely
because individual producers are known. Resolve a remaining named composition
at its owning task before writing an oracle/body, rather than imposing a broad
up-front source census.

## Upgrade ownership: Task2

`Armaments.use` and `ArmamentsAction.update` establish actual hand-only upgrading:
base grants5 Block and upgrades one eligible remaining hand card; upgraded
grants the same5 Block and upgrades every eligible remaining hand card. With no
eligible card the upgrade action completes; one eligible card is forced, more
requires a selection. Upgrading does not mutate the master deck. Noneligible
hand cards remain owned, including while selection is suspended.

`SearingBlow` constructors and `upgrade` independently confirm the distinction
already retained in the red-card report §4.2: a real upgrade adds4+the old count
to base damage, then increments the count. Successive ordinary upgrades give
12/16/21/27/34. The count-taking constructor initializes damage12 while setting
the supplied count: count alone is not a universal reconstruction invariant.
Stat-equivalent copying and permanent-entry reconstruction therefore need
different operations, not one formula used indiscriminately at every boundary.
Arithmetic is source-width signed integer arithmetic, not Python/int64 gameplay.

## Act4 door, transition and terminal source: Tasks21–22

- `SpireHeart.goToFinalAct` opens the event-version door. `DoorUnlockScreen.exit`
  marks the old room complete and selects TheEnding, not a direct Heart fight.
- The ordinary `AbstractDungeon` constructor invokes `dungeonTransitionSetup`
  before subclass map generation. That setup advances the act, clears act-local
  progression and requests A0 healing through the player's ordinary healing
  method using maxHP as the nominal amount. It is not an unconditional assignment
  of currentHP=maxHP; actual healing prevention remains relevant.
- `TheEnding.generateSpecialMap` creates a connected Rest→Shop→Elite→Boss route,
  plus a **TrueVictoryRoom** destination after the boss. Original room transition
  increments the floor on actual entry. Under the existing native collapsed
  Act3-ending mapping at floor50, playable Act4 rooms are51–54 and the final
  TrueVictoryRoom is55. The fourth all-key choice itself is not another floor.
- `CorruptHeart.die` calls ordinary death and final-boss victory bookkeeping,
  but does not replace the player-facing Proceed transition. `ProceedButton.update`
  for a TheEnding boss invokes `goToTrueVictoryRoom`, which schedules that actual
  room and normal room transition. `TrueVictoryRoom` owns presentation, not another
  combat/reward/menu that an agent must win. Native can omit its cutscene while
  preserving the explicit boss Proceed and real final entry/terminal.
- Thus the plan's conditional205 total was premature: with actual floors1–50,
  three keys, Act4 floors51–55 and HeartVictory, the selected reward policy totals
  **215 = 50 + 15 + 50 + 100**. This is conditional bookkeeping, not evidence that
  an in-repo policy completed that path. No extra readiness/door/animation bonus.

## Reward publication: Task22

The relevant generic reward producer is **`AbstractRoom.update`**, after
`endBattle` marks battle over; the M9 note's offsets599–1214 refer to `update`,
not the short `endBattle` method. This locator correction does not change the
qualified M9 suppression result or its native behavior.

`AbstractRoom.update` takes the real Elite path for Shield/Spear, constructing
non-daily25–35 gold, then invoking the room's actual drop/potion producer and
opening the reward screen. `MonsterRoomElite.dropReward` adds a real relic and
checks the burning source for Emerald. Shield/Spear is not a burning elite merely
because Act4 is enabled. Card reward construction still belongs to the opened
reward-screen owner; M10 uses its declared captured offers, not full standard
rarity/pity acceptance.

The non-endless TheEnding boss branch, like final TheBeyond, exits before drop,
potion and reward-screen publication. No claimable Heart gold/card/potion/relic
or extra boss chest is justified by generic gold-object construction. Preserve
actual Victory settlement and the separate source Proceed described above.

## Act4 actor bodies: Tasks19–20

Named constructors qualify exact IDs `SpireShield`, `SpireSpear`, `CorruptHeart`;
A0 HP110/160/750. Shield/Spear are Elite actors and have Artifact1 each. Shield's
prebattle action also applies `Surrounded` to the player. Surrounded and
`BackAttack` are distinct indicators; do not multiply their descriptions twice.

Shield: A0 Bash12 then Strength−1 for the no-orb Ironclad profile; Fortify30 to
the monster group; Smash34 and Block equal to the captured calculated damage
output, not actual HP removed. Selection counter advances when selecting, not
when executing: fair coin chooses Bash/Fortify at counter mod3=0, the next
selection chooses the other, mod3=2 selects Smash. The no-orb branch must not
erase the separately qualified conditional Focus branch for future profiles.

Spear: Burn Strike5×2 then two Burns into discard at A0; Skewer10×3; group
Strength2. Selection counter mod3=0 chooses Burn Strike unless newest history
is Burn Strike, otherwise buff; mod3=1 selects Skewer; mod3=2 fair coin chooses
Burn Strike/buff. This is not IID turn frequencies or a universal fixed cycle.

Task19's named-member readback further qualifies the encounter-only orientation
boundary: `AbstractPlayer.playCard` writes horizontal facing at manual
ENEMY/SELF_AND_ENEMY target selection while Surrounded is present; target-required
potion selection has its own equivalent write. No-target/ALL_ENEMY and direct
autoplay/repeat use do not write facing. Shield/Spear `die()` writes facing to the
surviving non-dead/non-dying partner immediately, then queues Surrounded removal
before that survivor's BackAttack removal. `applyBackAttack` is a Surrounded plus
relative-side predicate, not the mere presence of either marker. The public native
view reports the side/orientation-derived active rear attacker, not the raw
per-enemy marker. These are findings from the named owned archive methods only;
the archive build/channel is unverified and no original runtime was executed.

Damage checkpoints remain distinct: `calculateDamage(int)` multiplies at its
qualified intermediate checkpoint before later final modifiers, while
`applyPowers()` multiplies the already-calculated `DamageInfo.output`; both use
binary32 multiplication and truncation, not rounding. The native selected-intent
mapping uses its explicitly modeled checkpoint; do not infer arbitrary enemy
DamageInfo/callback composition from that public intent value.
The named `SpireShield.takeTurn` check is the player's active-Orb list
(`orbs.isEmpty`), not Defect identity or the configured count of empty Orb Slots:
no active Orb takes Strength−1 with no AI coin; at least one active Orb consumes
`AbstractDungeon.aiRng.randomBoolean`, applying Focus−1 only on true and otherwise
falling through to Strength−1. The attack/action work is queued before this immediate
coin check. The native Focus effect is narrowly admitted only from that Shield Bash
branch; the encounter actor-local enemy-AI stream represents this source-qualified
AI random choice in the deterministic simulator, not original RNG parity.
For the encounter's initial orientation, the named `AbstractCreature` constructor
explicitly initializes `flipHorizontal=false`; this maps to the simulator's
initial `PlayerFacing::Left` under the v1 qualified relative-side predicate (the
right-side Spear is initially behind). Do not infer this initial state from a
player marker or reset it on autoplay.


Heart: initial Debilitate is selected on entry and resolves as the first enemy
turn (not as pre-battle work); then a fair-coin attack, its opposite, and buff
repeat the three-selection pattern. A0 Blood Shots are twelve hits at2 each;
Echo is one hit at40. Debilitate
queues Vulnerable2, Weak2, Frail2, then one each Dazed/Slimed/Wound/Burn/Void in
that order into the draw pile via the actual insertion actions. Buff requests
negative-Strength compensation+2, then by prior buff count: Artifact2, Beat+1,
PainfulStabs, extraStrength10, thereafter extraStrength50. Buff count advances
when the buff body executes; selection count advances at selection.

Task20's qualified source notes retain the narrower implementation facts:
Blood Shots constructs twelve distinct2-damage entries; the first selected
Debilitate is queued by `takeTurn`, after the opening draw/player turn; and the
Heart buff computes negative-Strength compensation before its ordinal-specific
power. Invincible's `onAttackedToChangeDamage` receives the damage after Block,
caps that value by remaining allowance, and spends the capped value before target
HP overkill clamping. Thus an overkill hit may spend more Invincible allowance
than HP actually removed; this is not a claim that Reaper earns healing for that
overkill. Its receipt still reports actual post-Block HP lost, and Reaper healing
follows that receipt. These are static source-grounded rules from the named
CorruptHeart/InvinciblePower methods in the existing Task1 inspection; they are
not original-runtime execution/parity claims.

The native Heart integration exercises that distinction through the actual
Reaper and Charon's Ashes listeners: observed damage receipts distinguish Normal
from source-null THORNS, and earned healing is summed from actual HP loss. It also
composes real Havoc parent/child card-use ownership and multi-hit Whirlwind with
Beat of Death; the former yields one callback per completed use (not per hit),
while an unplayable Havoc child yields no child callback. These bounded native
cases characterize simulator queue/listener composition, not original-runtime
event parity.

## Heart powers and draw ordering: Tasks10/20/24

`BeatOfDeathPower` has exact ID `BeatOfDeath`, BUFF classification, A0 amount1.
Its after-use callback queues owner-attributed THORNS damage to player at bottom.
`UseCardAction.update` owns after-use notification; its dispatch/terminal/pruning
composition must be matched before choosing a lethal/nested-play oracle.

`InvinciblePower` has exact ID `Invincible`, A0 allowance/reset quota300 and
priority99. Its incoming-damage-change callback caps the passed post-Block value
to remaining allowance and spends that capped value, without a damage-type
exclusion inside that callback. `AbstractMonster.damage` calls it after Block
and attacker damage-change notifications. Its owner-start callback restores the
quota, not player-start by convention. Remaining allowance is the actual public
amount used in its description, so qualified native public views can expose it;
do not expose private future buff history alongside it.

`BrutalityPower.atStartOfTurnPostDraw` queues additional draw before its HP-loss
request. The printed-text ordering discrepancy is resolved for this exact callback,
not by rewriting the retained wiki description or claiming arbitrary callback
drains finish before another hook is invoked.

## Remaining task-local prerequisites (not silent completions)

- Task19's principal facing cases are now qualified by the isolated Luna factual
  assistant's named-member readback below. Random-target replacement composition
  remains separate if an actual producer requires it; do not infer it from an
  ordinary selected target.
- Tasks7/9/11–16: exact source-specific nested-repeat, Fatal/healing, exhaust and
  item compositions beyond the cited existing reports. Close only the named
  load-bearing members needed by the next batch, before tests assert new timing.
- Task21: transition heal prevention/reset order uses existing mutation owners;
  include the real later final room without changing M9's collapsed pre-door cutoff.

No new reference data IDs/grades have been changed by this note alone. The owning
mechanics task updates affected records/citations when integrating its native IDs.

## Isolated Luna facing/after-use supplement

Same owned-static evidence grade and unchanged archive/tool identities; no runtime
validation or product changes. This is the controller's semantic synthesis of
the dedicated factual assistant's named-member handoff, not another audit.

- Manual enemy-target and self-and-enemy-target cards turn the Surrounded player
  toward the selected enemy in `AbstractPlayer.playCard`27–111, before queuing the
  card. This includes targeted Skills, not only Attack animation. Alternate
  no-target/ALL_ENEMY paths115–136 do not perform that write.
- Target-required potion selection independently turns toward a live selected
  enemy in `PotionPopUp.updateTargetMode`165–211, before actual potion use227–238.
  Cancellation does not use/consume the potion. The no-target potion route is not
  silently treated as a selected attack.
- Autoplay/repeats call `AbstractPlayer.useCard` through the card-queue owner,
  without this target-choice write. `useFastAttackAnimation` changes animation
  fields, not facing. Use the orientation established by a prior actual target
  choice/partner-death checkpoint; a nested pile selector does not turn the player.
- Both partner `die` members0–136 scan not-dead/not-dying survivors. While the
  player still has Surrounded, they immediately face the player toward the survivor
  and queue removal of Surrounded and that survivor's BackAttack indicator. If no
  live survivor remains, this loop does not perform the partner refresh.
- `AbstractMonster.applyBackAttack`0–61 requires Surrounded plus the actor being
  behind the current player orientation. With normal orientation the right actor
  qualifies; with flipped orientation the left qualifies. Calculated damage uses
  binary32 multiplication by1.5 followed by integer truncation at that checkpoint,
  not round-to-nearest or a second multiplier from Surrounded. `calculateDamage`
  places this after ordinary give/receive/stance modifiers and before final
  give/receive passes; `applyPowers` similarly adjusts the actual damage outputs.
- After-use notification belongs to `UseCardAction.update`10–144, before clearing
  free/autoplay flags and before purge/Power/ordinary disposition. Each actual
  repeated play constructs its own use action; an outer repeat request alone is
  not another Beat callback. The explicit use-trigger suppression flag matters.

Queued-card pruning is not action-list clearing: lethal-monster handling's
`cleanCardQueue` and battle-end `clear` have different owners and effects. Final
named-member follow-up qualifies the actual damage dispatch: Beat's ordinary
`DamageAction` retains owner Heart/type THORNS, and THORNS bypasses its
dying-source/dead-target cancellation checks. A dying Heart therefore does not by
itself cancel Beat damage. Lethal-monster cleanup preserves UseCard and DAMAGE
work. Separately, the player DeathScreen update bypasses room/action-manager
dispatch after the lethal action, rather than promising that all queued children
complete. Remaining queued-action versus animation-driven battle-end clearing
order is not original-runtime qualified. Task20 needs an explicit deterministic
headless settlement ruling at its owning checkpoint, not a clock subsystem,
precredited future heal or universal simultaneous-lethal winner.

## Task3 random-spot insertion conflict

The existing red-card report's nonempty random-spot support is independently
confirmed along WildStrike/RecklessCharge → MakeTempCardInDrawPileAction →
ShowCardAndAddToDrawPileEffect → Soul.onToDeck → CardGroup.addToRandomSpot.
For draw size n>0 the insertion indices are0..n−1; empty insertion appends.
Current native `piles::insert_generated(ShuffledDraw)` instead samples n+1 gaps,
including a top-gap outcome absent from this source route. This is a genuine
shared-helper mismatch, not permission to invent a second old-rules simulator.
Before changing it, Task3 records actual affected callers and replay/rules-identity
compatibility; old artifacts remain preserved and v1 ABI vocabulary/fingerprint
must not be silently widened. This finding alone does not qualify other source
routes or original RNG parity. This finding was recorded before the helper change;
Task3's integrated candidate now uses the corrected shared insertion law with the
explicit rules-identity amendment below, not an unchanged-old-behavior claim.

Actual native callers currently include Hex and the OrbLaser/Repulsor status
effects. This is a reachable earlier-profile correction, not just new-card
behavior. Controller's explicit compatibility amendment retains the numeric-v1
descriptor/ABI, record schema and native HMAC algorithm/domain IDs while changing
corrected M9-session rules identity to `spire-gym-native-m9-rules-v2`; old rules-v1
records remain preserved and are rejected by strict identity preflight instead
of silently replayed under revised mechanics. Existing callers' source routes
are now independently qualified: `HexPower.onUseCard`51–71 creates the random-spot
Dazed draw action; `Repulsor.takeTurn`104–151 uses the same constructor; OrbWalker
Laser162–229 uses `MakeTempCardInDiscardAndDeckAction`, whose update31–78 creates
one random-spot draw Burn and one separate discard Burn. The draw-effect overload
115–125 and random-spot branch92–112 reach the same CardGroup endpoint. These
named source chains close the helper-change prerequisite without asserting that
opening shuffle, explicit bottom insertion or other uninspected routes share it.

Integrated Task3 verification confirms this is observable, not metadata-only:
corrected M9seed7/all-three histories naturally die at action697/floor42/reward41,
where preserved rules-v1 records had778/floor47/reward46. Seed11 Time/Donu losses
are814/831 actions rather than813/830 at the same floor47/reward46. Existing
M7/M8 seeded action counts also change in some paths. Old records/results remain
separate historical evidence; no policy/seed/resource tuning or old-trace
compatibility claim accompanies the shared-rule fix.

## Task3 per-hit selector closure

`SwordBoomerang.use`34–51 bottom-queues one random-hit action per magic-number
hit. Each `AttackDamageRandomEnemyAction.update`27–45/91–112 chooses at its own
dispatch from current encounter-list order, recalculates card damage against that
target, and top-queues the actual NORMAL player hit. That hit and its reactions
resolve before the next random selection; later targets are not pre-sampled.
`MonsterGroup.getRandomMonster`371–426 excludes half-dead/dying/escaping actors,
returns no target when none qualify, and allows repeat identities. Native stable
encounter order maps to this list; sorting arbitrary target IDs is not the rule.

The original singleton selector still calls the source card RNG; the existing
native HMAC bounded-one operation does not advance its cursor. Preserve the
selected native scoped RNG contract and distinguish it from original RNG call
parity. Pummel's separate fixed-target actions check current target HP and do not
transfer remaining hits to a survivor; the final action uses ordinary DamageAction
guards. Existing player-death dispatch/terminal qualifications above still apply;
these selector facts do not establish a new universal terminal-winner law.

## Task4 support composition prerequisites

The isolated factual assistant closes these named seams at the same owned-static
grade; no support-card bodies or new runtime claims follow from the facts alone.

- Entrench queues `DoubleYourBlockAction`; its update24–52 passes current Block
  directly to `addBlock`. Do not apply Dexterity/Frail/card NoBlock again. The
  shared gain checkpoint still invokes actual gain callbacks and caps player
  Block999 (`AbstractCreature.addBlock`1386–1489). “Direct doubling” does not mean
  bypassing every relic/power callback. Use the native direct GainBlock owner, not
  a second printed-card Block calculation; zero and near-cap cases remain real.
- `PowerThrough.use`32–52 queues two generated Wounds into hand before the
  ordinary modified Block15/20 action. `MakeTempCardInHandAction.update`129–184
  fills available hand slots and sends excess to discard, without evicting cards
  or emitting actual-draw callbacks. Test room2/1/0 after actual parent movement,
  preserving valid generated identities and the one census owner.
- `SpotWeaknessAction.update`29–69 checks retained target's intent base damage
  at effect dispatch: >=0 grants Strength3/4, including mixed/base-zero attacks;
  negative/null does not. Use the current native damage-entry base, not modified
  preview output or enum-only Attack classification. Candidate legality must not
  reveal the conditional outcome. Targetless public plays remain invalid/atomic;
  the source UI no-target thought is not a grant. These named members do not
  establish a special selected-target-disappearance rule.
- Other Task4 literals/flags reuse the existing red-card report. Reckless Charge
  uses the already-qualified corrected random-spot route; Ghostly Armor uses the
  existing ordinary Ethereal owner. No repeat audit or extra cleanup owner needed.

## Task5 owned-selection prerequisites

Isolated named static readback supplements, not replaces, the existing red-card
report. No new card bodies, Task11 exhaust callbacks or runtime parity inferred.

- True Grit queues Block before exhausting from the then-current hand. Base uses
  source card-RNG random selection; upgraded uses one required choice. The playing
  parent is already outside that hand. Empty is no-op. Native mapping is current
  eligible-hand order plus existing scoped HMAC bounded selection, including its
  singleton no-advance contract, not original Java RNG/cursor parity.
- Headbutt queues its hit before current-discard selection and skips that move
  when battle-ending. Empty source is no-op, singleton forced, multiple required
  choice. Move the actual existing instance to draw top, not a synthetic copy.
- Warcry completes its1/2 requested draws before capturing the resulting current
  hand for required topdeck selection, then finishes by ordinary exhaust. Existing
  draw cap/recycle/NoDraw governs those draws; an empty source needs no chooser.
- Battle Trance queues draw3/4 before applying NoDraw. Existing NoDraw can already
  suppress those requested draws; a new debuff must not suppress its own preceding
  draw. Artifact can block NoDraw, whose flagged end-of-turn removal allows the
  next turn's normal drawing. Do not eagerly set a card-side flag in prepare.
- Exhume's action checks full hand before selecting eligible existing exhaust
  instances, excluding all Exhume IDs. Empty is no-op, singleton forced, multiple
  required choice. Preserve actual local state/identity and do not replay old draw
  callbacks or reconstruct the returned card.

Public source-zone candidates (hand/discard/exhaust) remain visible when entitled;
only after an actual topdeck choice/move does that identity become newly entitled
known draw-top state. Neither source-zone candidate visibility nor a required
selector authorizes revealing the unknown draw tail. Shared optional0..N selection
must retain its real parent until explicit ConfirmSelection (including zero),
without giving existing required selectors spurious confirm/cancel actions. Potion
parent support is typed infrastructure now; Elixir consumption/body stays Task16.

## Task6 copy-source prerequisites

Qualified static readback of DualWieldAction and AbstractCard copy methods:

- Dual Wield is cost1, net1/2 copies of an eligible current-hand Attack/Power,
  not a Skill, and is not inherently Exhaust. No eligible card is a no-op. Sole
  eligible source stays active; the multi-choice path requests1+copy-count children
  for the selected source. The selected-original custody follow-up is closed below.
- The route is stat-equivalent creation, not autoplay. Actual inserted children
  pass through MakeTempCardInHandAction's copy path; overflow goes to discard.
- `makeStatEquivalentCopy` replays existing upgrades on the fresh kind, then
  restores actual source upgraded/count/base damage/Block/magic/misc, costs and
  cost-modified flags, free-play-once and other explicitly copied metadata. Actual
  mutable Searing Blow damage must survive; count alone is not reconstruction.
- Stat-equivalent copies have new family identities; `makeSameInstanceOf` adds
  source UUID preservation to that route. This private provenance is not an agent
  handle or another pile registry. Source-local Exhaust/Ethereal/Innate/per-play
  exhaust mutations are not explicitly restored by the copy method: constructor
  and replayed upgrade semantics determine those flags, unlike arbitrary native
  whole-local-state cloning.
- Dual Wield's own upgrade changes quantity, not child upgrade status. The
  separate Master Reality creation checkpoint is source-qualified but not a new
  supported M10 producer or a reason to fabricate unearned upgrades. Preserve
  existing native public-copy adapter compatibility while adding the actual typed
  source-copy route; future state producers extend it in their owning tasks.

These facts do not establish Task6 implementation, original-runtime parity or
Master Reality/Rampage support. Raw named outputs stay private; exact field-level
handoff and selected-source custody supplement remain isolated assistance evidence.

Selected-source follow-up: `HandCardSelectScreen.selectHoveredCard`1863–1880
transfers the actual source from hand to selectedCards. `DualWieldAction.update`
195–241 queues its1+dupeAmount children, returns only cannotDuplicate ineligible
cards, then clears the selected group; `returnCards`249–272 does not restore the
chosen original. `CardGroup.removeCard(AbstractCard)`456–483 has a callback only
for MASTER_DECK, not this hand removal. There is no discard/exhaust callback.
Task6 therefore retains that original in typed inactive native custody for exact
permanent census/exit/fault ownership while keeping it outside active piles. Sole
eligible path still leaves its original active plus1/2 children. All actual
children use new stat-equivalent families; no inherited original UUID or extra
active original is silently substituted to simplify native bookkeeping.

## Task7 autoplay/X source handoff (remaining composition qualifications explicit)

Isolated named-member inspection qualifies these chains, not a general scheduler
audit or original-runtime parity. Task7 has not started implementation.

- Havoc chooses its random enemy target at actual use, before the queued top-card
  extraction resolves. The later PlayTopCardAction uses that target, recycles a
  nonempty discard if draw is empty, extracts the actual card without a draw
  callback, arms exhaust-on-use-once and queues autoplay. Target/legality are
  checked on dequeue; ordinary child energy is waived. Native accepted execution
  must not sample RNG during read-only legal-action preparation.
- Double Tap queues a same-instance-family repeat of a qualifying non-purge
  Attack, retains original target and captured energyOnUse, marks purge and free
  repeat, and spends one charge when queued. Do not replace it with raw payload
  duplication or consume another charge recursively. Source family identity is
  distinct from the stat-equivalent new-family route closed in Task6.
- Whirlwind captures per-target damage at use, and its action uses energyOnUse
  unless sentinel−1 requests current energy. Positive rounds queue ordinary
  per-enemy hits; non-free execution then spends current total energy at that
  action checkpoint, not a fixed positive card cost or necessarily the captured X.
  Free is not zero X. Chemical X's source checkpoint is not a newly supported
  relic/producer or an original RNG/call-count claim.

Havoc failed-child destination remains a concrete prerequisite: extraction arms
the exhaust flag before canUse, and failed use still queues suppressed-use cleanup.
The actual called CardGroup transfer methods must decide whether an absent source
hand entry prevents destination insertion; merely seeing hand.moveToExhaustPile
does not settle it. Also distinguish Time Warp's established no13th-new-input
boundary from the fate of an already queued repeat. Both narrow called-owner
follow-ups are assigned to the same factual assistant; no new discard/exhaust or
queued-child suppression oracle is asserted from the first handoff alone.

Both follow-ups are now source-qualified. `UseCardAction` snapshots intrinsic or
exhaust-once flags even when dontTriggerOnUseCard suppresses use and after-use
callbacks. Its final disposition is not suppressed. `CardGroup.moveToExhaustPile`
3676–3726 does not require membership in the receiver hand group: it runs actual
exhaust callbacks and inserts the same object into exhaust. Discard transfer likewise
does not require an earlier hand removal to succeed. Thus Havoc's failed ordinary
Status/Curse exhausts without a use/end-turn payload; default discard applies only
to a non-exhaust, non-purge autoplay route without Havoc's imposed flag. This narrows
the earlier report's generic unplayable-autoplay conclusion, not all card families.

`TimeWarpPower.onAfterUseCard`76–144 calls the actual early-end sequence at12.
`GameActionManager.callEndTurnEarlySequence`1344–1405 clears the existing CARD queue
and limbo after scheduling dontTriggerOnUseCard cleanup actions for queued autoplay
entries. Their payloads do not execute or wait until next turn. Double Tap repeats
purge; Havoc children exhaust with applicable exhaust callbacks. This does not
clear unrelated accepted EFFECT work. Task7 must retain each queued child and its
real disposition through cleanup/fault boundaries instead of losing its identity
or imposing a global queue wipe. Animation-mediated tail remains unverified.

One final named duration/counter qualification is assigned before Task7: Double
Tap's unused-charge end-of-turn removal and immediate-vs-queued decrement timing.
No unused-next-turn-charge or selection-visible counter oracle is guessed from
the card's general text alone.

That final qualification is now closed: `DoubleTapPower.onUseCard`138–158 directly
decrements remaining amount when queuing an eligible original Attack's repeat;
at zero it separately queues removal. Purge guards prevent recursive consumption.
`atEndOfTurn(true)`160–174 queues removal of any unused remainder at the owning
player end-of-turn trigger (`AbstractCreature`1764–1788). Thus a nested selection
can expose an immediately reduced counter while removal remains pending; it is
not the number of queued repeats, and unused charges do not persist next turn.

## Task8 growth prerequisites

Named static-source handoff, not implementation/runtime evidence:

- Flex queues positive Strength before a separate LoseStrength debuff. Artifact
  can block installing that loss without undoing the gain; Artifact present only
  at expiry can block the queued negative Strength application while loss-power
  removal still completes. Keep the temporary loss record separate from Strength.
- Limit Break reads current signed Strength when its action resolves, then queues
  an equal signed application. Negative/zero behavior uses ordinary power/Artifact
  handling, not a positive-only clamp. Base Exhaust, upgraded non-Exhaust.
- Blood for Blood's cost decrease follows each positive final HP-loss event, not
  HP quantity. Actual notification visits hand/discard/draw, excluding exhaust,
  limbo and retained inactive census custody. Positive HP_LOSS/THORNS qualify;
  blocked/prevented zero does not. Fresh-kind creation catches up from combat
  damage-event history; stat-equivalent copies restore actual source costs.
- Rampage queues its hit before same-family damage growth. The actual traversal
  includes card-in-use, hand/draw/discard/exhaust/limbo, not the master deck or
  inactive replacement journal. Stat-equivalent new-family copies stay separate.

Dropkick's full update closes the initial handoff contradiction: its null-target
and non-Vulnerable branches skip only rewards and join the unconditional hit
insertion. A valid target therefore receives captured5/8 damage regardless of
Vulnerable; action-time Vulnerable adds energy1/draw1, resolving Hit→Energy→Draw
through front insertion. Native targeted-play preflight must reject a missing
target atomically: the source null path still queues a hit and later dereferences
the target, not a legal no-op. This supersedes the earlier handoff and the older
red report's overly broad conditional-hit wording, while retaining their literal
damage/reward values. No Task8 body, runtime parity or general all-owned-cards
callback is established by this source-only note.

Cost-floor follow-up: `updateCost(-1)` computes the old base/turn difference, clamps
new base to0, and only updates either price/modified flag if the base changed.
Thus `(base0,turn2)` stays `(0,2)` on a damage callback; `(4,6)` becomes `(3,5)`.
Blood for Blood's upgrade is different: it calls `upgradeBaseCost(cost−1)` when
already discounted, then clamps its own negative base back to0. That helper
recalculates a positive old turn price before the card-local clamp. Upgrade cases
`(0,2)`, `(1,2)`, `(2,2)` become `(0,1)`, `(0,1)`, `(1,1)` respectively; an old
turn cost0 remains0. Native temporary Armaments upgrades must retain legal cost
state/census at this floor, not reject a stale reduction4 against upgraded cap3 or
reuse the ordinary damage-callback no-op for the distinct upgrade route.

## Task9 Fatal and earned-heal prerequisites

Named static readback, same unverified archive-build/runtime limits:

- Feed performs its hit, then tests `(isDying || HP<=0) && !halfDead && !Minion`.
  Minion means the actual power, not an encounter/name exclusion. It increases
  maxHP3/4 and requests ordinary healing synchronously before returning to queued
  work. Healing uses existing modifiers/cap and does not revive an already dying
  player. A queued retaliation therefore differs from synchronous committed Death;
  neither creates a universal simultaneous-lethal winner. Normal native target
  preflight still rejects absent/invalid targets before accepting payment.
- Reaper iterates eligible room slots in order, calls each damage operation, and
  sums positive resolved `lastDamageTaken`, excluding Block/prevention/overkill.
  Its damage loop completes before queued retaliation dispatches; the subsequent
  single ordinary heal sits behind that queued damage. Do not interleave queued
  retaliation between synchronous loop hits merely because native attacks share
  a resolver. Retain accepted progress/results through an actual later fault.
- Last-enemy selective cleanup preserves earned HealAction and DAMAGE work, not
  just a fabricated Victory result. Earlier lethal retaliation cannot be rescued
  by pre-crediting the pending heal. Magic Flower/Mark of the Bloom remain the
  existing heal owners; TIME-001 stays closed, with no frame/clock subsystem.
- Feed's committed maxHP returns through the existing CombatExit/single Run-player
  settlement owner, including Death. Retained accepted faults must preserve that
  committed mutation, not rerun its heal or create a second inventory. Extend a
  durable-effect seam only where this actual committed delta needs custody;
  the Parasite-only card journal is not a generic maxHP ledger.

The exact qualified Fatal predicate does not by itself grant a legal attack on an
already dying/escaping target. Keep public preflight controls distinct from narrow
post-damage predicate diagnostics. These facts alone do not establish implemented
Task9 bodies or passing runtime tests; execution state is recorded in the ledger.

## Task10 player-power checkpoint prerequisites

Qualified named static callbacks, not runtime parity or new enemy-power scope:

- Barricade guards the actual player-start Block-loss path. It is not generic
  immunity to explicit loseBlock, nor evidence for an enemy reset change.
- Berserk separately applies self-Vulnerable before its energy power. Artifact
  blocks only the debuff application. Energy comes from the ordinary next-player
  start callback, not application-time or a fabricated first-turn trigger.
- Brutality queues its extra draw before HP_LOSS at the post-normal-draw power
  callback. Demon Form queues Strength at that same post-draw checkpoint. Neither
  triggers retroactively on mid-turn application. Brutality's upgraded Innate is
  card-opening entitlement, not a larger callback amount.
- Player Metallicize queues Block from pre-end-turn-card powers, before card
  processing; generic later TurnEnded is not its owner. Preserve independently
  qualified existing enemy behavior instead of shifting both by analogy.
- Combust skips basically-dead encounters, otherwise queues its HP_LOSS before
  THORNS all-enemy damage with null damage owner. Do not silently substitute a
  player-owned NORMAL packet. Its damage stacks by incoming amount, but HP cost
  increases by1 per extra copy, requiring separate owned state. Ordinary and
  Time-Warp-forced end share the same callback, not two producers. Existing
  committed-damage/terminal guards determine whether later damage executes.

Task10 code has not started. Raw callback outputs remain private, and unsupported
enemy lifecycle details are not filled in from these player-only findings.

## Exhaust and reactive composition: Tasks11–13

These later task-local findings retain the archive/build/runtime qualifications
above. They summarize the named exhaust actions, player draw/Block/damage callers,
and the six corresponding reactive powers; they are not a new family-wide audit.

- Exhaust movement and notification are character-neutral. Card/power-specific
  listeners supply effects; they do not own separate pile movement. Intrinsic
  Ethereal and per-use finish disposition are distinct properties.
- Second Wind and Sever Soul capture non-Attack references in current hand order;
  repeated front insertion executes them in reverse. Fiend Fire instead captures
  a count once, then each random exhaust selects from the current group. Listener
  invocation during transfer is distinct from execution of its queued children;
  already-front-queued batch exhaust/hit work precedes those children.
- Actual draw invokes the card callback before installed powers in their order.
  Void queues energy loss; Confused synchronously randomizes eligible ordinary
  costs and clears free-play. Evolve queues Status-only draws, checking No Draw;
  Fire Breathing queues null-owner THORNS for Status or Curse. Remaining ordinary
  requested draws precede these queued child actions. Creation in hand is not draw.
- Flame Barrier reacts to foreign-owner NORMAL damage even if fully blocked or
  zero. Native Orb is a Lightning/THORNS adapter category, not a fourth original
  DamageInfo type. Retaliation is player-owned THORNS; removal is at the next
  pre-draw player start. Its played Block uses ordinary card Block modifiers.
- Juggernaut's predicate is a positive post-relic Block notification, before the
  cap, not net Block increase. Its queued random action retains its damage and
  chooses a living target when dispatched. Scoped native target RNG is not an
  original-cursor equivalence claim.
- Rage queues raw Block once per Attack use after that card's payload, not per
  hit, and expires at the actual player-end checkpoint. Rupture requires positive
  HP loss with player DamageInfo ownership, without a damage-type exclusion:
  player-power HP_LOSS and self-owned held-Burn THORNS therefore qualify, whereas
  blocked-to-zero, zero and enemy-owned loss do not.
- In the headless resolver, callback work appended behind real queued actions
  must still precede its artificial Decision sentinel. This preserves a ready
  boundary only after work drains; it does not introduce an original-game clock.
