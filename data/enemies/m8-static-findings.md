# M8 task-local static findings

Inspected 2026-10-23 under explicit permission to resolve reference gaps and update
`data/*`. Independently worded facts from named methods in the user-owned archive.
The target is PC v2.3, but this archive's upstream build/channel is unverified:
**not target-version provenance, original-runtime validation or game acceptance**.
No game source, disassembly, binaries or private outputs are distributed. Existing
wiki-derived records retain their attribution and source-specific qualifications.

Archive SHA256: `cfad868ac8d65a88e71a0bf096fb09f78811e553effe0787c5309a655e081673`.
Existing JDK `javap` 21.0.12.1+1, static inspection only, Java option-injection
variables cleared. No original classes/initializers executed. Hashes identify
bytes, not mechanics proof. RNG-001 does not require original PRNG/cursor parity.

## Task1: Orb Walker, Shapes and related powers

Names below use package `com.megacrit.cardcrawl`.

- `monsters.beyond.OrbWalker.usePreBattleAction`, `takeTurn`, `getMove`:
  Generic Strength Up3 below A17,5 thereafter. Laser requests ordinary attack, then
  one Burn shuffled into draw and one in discard. Integer proposal below40 selects
  Claw unless both newest moves are Claw; otherwise Laser unless both newest are
  Laser. Redirect to the other move, not reroll or renormalized pool.
- `powers.GenericStrengthUpPower` constructor, `atEndOfRound`: exact ID
  `Generic Strength Up Power`; applies its amount as Strength at **end of round**,
  without decrementing itself. Earlier owner-end-turn wording is superseded.
- `monsters.beyond.Repulsor.takeTurn`, `getMove`: proposal below20 selects Attack
  only when newest move is not Attack; otherwise Daze. Daze requests two Dazed
  shuffled into draw, not discard/permanent deck.
- `monsters.beyond.Spiker.takeTurn`, `getMove`: before six executed buffs,
  below50 selects Attack only after non-Attack; otherwise buff+2 Thorns. Buff count
  increments on execution, not selection. After six buffs Attack is forced.
- `powers.ThornsPower.onAttacked`: a nonnull different attacker and type other
  than THORNS/HP_LOSS queues owner-attributed THORNS retaliation. Positive HP damage
  is not required; per-hit callback, not Sharp Hide's Attack-play trigger.
- `monsters.beyond.Exploder.takeTurn`, `getMove`: executed-turn counter selects
  Attack for first two turns, then Unknown-intent explosion. Ordinary Attack bases
  are distinct from the power's fixed explosion.
- `powers.ExplosivePower` constructor, `duringTurn`: exact ID `Explosive`.
  At countdown1 and owner not dying, queues default self-SuicideAction **before**
  owner-attributed fixed30 THORNS DamageAction to player; otherwise reduce1.
  Early ordinary killing is not an on-death explosion. Supersedes the secondary
  port's damage-then-suicide order and unresolved original damage type.
- `actions.common.SuicideAction` constructors, `update`: default delegates with
  true flag, clears Block and HP and calls actual death processing. Not replacement,
  escape or a fabricated player kill.
- `actions.common.MakeTempCardInDiscardAndDeckAction.update`: distinct draw/discard
  insertions for Orb Walker, no permanent acquisition.
- `helpers.MonsterHelper.spawnShapes`, `getAncientShape` and relevant mixed factory
  branch: three/four Shapes sample ordered members without replacement from two
  each of Repulsor/Exploder/Spiker. Mixed group order is independent shape,
  independent shape, Spheric Guardian. Independent helper is uniform over
  Spiker/Repulsor/Exploder, not a shrinking six-member pool.

Fixed THORNS requests do not use ordinary Strength/Weak/Vulnerable attack
calculation; Block still absorbs the typed damage. This does not establish every
other player-power/relic prevention interaction, every terminal callback
composition, all Ascension payloads or enemy constructor identities. Only the two
power constructors above close exact IDs. Native tests are simulator evidence,
never evidence of original-game execution.

## Task2: Darkling revival and conditional AI

Named `monsters.beyond.Darkling` constructor, `usePreBattleAction`, `getMove`,
`takeTurn`, `damage`, `die`, `changeState`; `powers.RegrowPower` constructor and
`actions.common.HealAction.update`. Only selected members retained privately.

- Constructor literal `Darkling`; A0HP48–56, Nip7–11 rolled once per actor,
  Chomp8 twice. Harden12. Slot index1 cannot Chomp; the two original outer
  identities retain eligibility when other actors are down.
- Opening integer0–49 Harden,50–99 Nip. Later0–39 chooses Chomp only for an outer
  actor not after Chomp, otherwise requests40–99 again.40–69 chooses Harden unless
  newest is Harden, then Nip.70–99 chooses Nip unless both newest moves are Nip,
  then requests0–99 again. Preserve recursive redirects, not rounded flat rates.
- On first zeroHP, mark half-dead; call current powers' onDeath and relics'
  onMonsterDeath callbacks, then clear **all** powers. This is not final combat
  settlement, nor proof of every separate Fatal/card-kill reward callback.
- With another Darkling not half-dead, select Regrowing, a no-op spent turn; its
  next roll while half-dead selects Reincarnate. Reincarnate requests healing
  `floor(maxHP/2)`, then REVIVE clears half-dead, reapplies RegrowPower and invokes
  relic spawn callbacks. Same actor, original maxHP and Nip; no fresh constructor.
- When all Darklings are simultaneously half-dead, release the cannot-lose guard
  and call actual final death processing on the group. Do not issue one room reward
  per temporary downing or revive a defeated group.
- `RegrowPower` constructor exact ID `Life Link`, technical BUFF. These are facts
  for the owned bytes only; upstream build/channel and original-runtime behavior
  remain unverified. Unimplemented relic/card callback consumers are not made
  supported by this static finding.

## Task3: Spire Growth, Transient and Maw

Named constructors, `getMove`, `takeTurn`, `usePreBattleAction` where present;
ConstrictedPower constructor/atEndOfTurn, ShiftingPower constructor/onAttacked,
FadingPower constructor/duringTurn and GainStrengthPower.atEndOfTurn. Named caller
closure: GameActionManager.getNextAction, AbstractCreature.applyTurnPowers/
applyEndOfTurnTriggers and AbstractMonster.damage's post-Block callback sequence.
Only selected members retained privately. Same unverified build/runtime limits.

- Constructor IDs: Spire Growth=`Serpent`, Transient=`Transient`, Maw=`Maw`.
  A0HP170/999/300, respectively. ConstrictedPower=`Constricted` (DEBUFF),
  ShiftingPower=`Shifting`, FadingPower=`Fading`.
- Growth A0: proposal below50 selects Quick16 unless both newest are Quick.
  Otherwise, if actual player Constricted is absent and newest is not Constrict,
  select Constrict10. Otherwise Smash22 unless both newest are Smash, then Quick.
  Artifact preventing a power application does not make that power present.
  The owned A17 branch adds2 Constrict damage and prioritizes eligible Constrict;
  this resolves that owned-byte dispute, not the attributed wiki's target-version
  provenance. Preserve the conflicting wiki A2 label as a source-specific claim.
- Constricted is source-attributed fixed THORNS at player end, still Blockable,
  not an extra NORMAL attack at enemy action time. Its named callback does not
  itself decrement the amount or establish all source-death/removal compositions.
- Transient starts Attack30 at A0 and adds10 per executed action. Shifting's
  positive onAttacked argument is **post-Block callback damage before the HP clamp**,
  not pre-Block attempted damage and not necessarily final clamped HP lost. It has
  no NORMAL-only predicate. It queues negative Strength; when Artifact is absent
  it also queues Shackled ahead of that loss. Artifact may prevent the loss.
  Shackled restores at affected-owner end-turn and then removes itself.
- GameActionManager queues takeTurn before applying duringTurn powers. Fading5
  therefore lets the fifth attack resolve before default self-SuicideAction; a
  player-lethal fifth attack is genuine Death, not successful survival. Fading6
  is the owned A17 branch. Expiry is actual self-death, not replacement/escape or
  a fabricated player-attributed kill. No original-runtime validation claimed.
- Maw starts internal turnCount1; every actual getMove increments it. Roar repeats
  until actually executed, setting the roar latch. Thereafter proposal below50
  selects Nom unless newest is Nom; otherwise after Slam/Nom choose Drool, else
  Slam. Nom deals5 per hit for `floor(internalTurnCount/2)`, equivalent to
  `ceil(selectedEnemyTurn/2)` in the ordinary sequence. Roar gives Weak3/Frail3,
  Slam25, Drool Strength3 at A0. Public reads/card count do not increment this clock.

## Task4: Writhing Mass and permanent Parasite

Named `WrithingMass` constructor/usePreBattleAction/getMove/takeTurn/damage,
ReactivePower constructor/onAttacked, MalleablePower constructor/onAttacked/
atEndOfTurn, Parasite constructor/onRemoveFromMasterDeck and the named obtain
actions/effect. Shared caller closure used AbstractMonster.damage,
AbstractCreature.addPower, AbstractPower constructor/compareTo, GameActionManager
queue methods, TwinStrike.use, AbstractPlayer.useCard, UseCardAction,
RollMoveAction and GainBlockAction. Existing retained members were reused; no
original classes executed. Same owned-byte/unverified-build evidence boundary.

- Constructor ID `WrithingMass`, A0HP160. Ordinary bases: Big32, Multi7 three
  hits, Block attack15, Debuff attack10 with Weak2/Vulnerable2. **The Block amount
  uses its damage entry's base**,15 at A0/16 at A2, not Strength-modified output or
  unconditional16. The wiki-derived fixed16 claim remains attributed in the record;
  owned bytes do not establish that every target-PCv2.3 build matches them.
- Opening proposals0–32 Multi,33–65 Block,66–99 Debuff; no opening Big. Later:
  0–9 Multi unless newest Multi, then retry10–99;10–19 Parasite unless used or
  newest Parasite, then a0.1f coin chooses Multi or retries20–99;20–39 Debuff unless
  newest Debuff, then a0.4f coin retries0–19 or40–99;40–69 Block unless newest
  Block, then retry0–69;70–99 Big unless newest Big, then retry0–39. These are
  recursive redirects, not flat post-restriction frequencies.
- Named Random.randomBoolean(float) compares RandomXS128.nextFloat strictly
  against the float literal. That helper returns one of2^24 equally spaced
  fractions. Thus0.1f accepts1677722/16777216 outcomes;0.4f accepts6710887/16777216.
  Native RNG need not reproduce original Java seeds, streams or incidental calls.
- Reactive exact ID `Compulsive`: nonnull attacker, damage type neither THORNS
  nor HP_LOSS, positive callback damage strictly less than current HP. It queues
  RollMove at bottom. Native Lightning Orb damage retains its previously qualified
  THORNS classification; a distinct native payload does not invent another
  original DamageType. Fully blocked/lethal hits do not qualify.
- Malleable exact ID `Malleable`: nonnull attacker, exactly NORMAL, positive
  callback damage strictly less than current HP. Queue current amount as Block
  at bottom for an enemy, synchronously increment amount. Enemy resets to base
  at owner-end-turn; player branch resets at end-round, not a universal start-turn.
- This setup inserts Reactive then Malleable. Named add/traversal paths do not
  sort by the available comparator. For actual prequeued TwinStrike: both hits
  and card cleanup precede appended RollMove/Block4/RollMove/Block5. Do not infer
  this timing for every dynamically-produced later hit.
- Parasite-used latches on execution **before** AddCardToDeckAction, not selection
  or successful later publication. Its ShowCardAndObtainEffect invokes obtain/
  master-deck callbacks and the actual permanent SoulGroup.obtain path; not a
  current-combat pile insertion. Arbitrary prevention relics remain unqualified.
- Parasite is an unplayable(-2), non-upgradeable Curse. Permanent removal calls
  decreaseMaxHealth3, floors maxHP at1 and clamps only excess current HP:88/88→85/85,
  40/88→40/85. Combat-copy exhaustion is not permanent removal.

## Task5: Giant Head / Nemesis / Slow / enemy Intangible

Named GiantHead constructor/usePreBattleAction/getMove/takeTurn, Nemesis
constructor/getMove/takeTurn/damage, SlowPower constructor/onAfterUseCard/
atDamageReceive/atEndOfRound, IntangiblePower constructor/atDamageFinalReceive/
atEndOfTurn. Caller closure: DamageInfo constructor/applyPowers, AbstractPlayer.
useCard, UseCardAction.update, AbstractMonster.damage, AbstractCreature.addPower/
applyEndOfTurnTriggers, MonsterGroup.applyEndOfTurnPowers, GameActionManager.
getNextAction/addToBottom, ReducePowerAction.update, LoseHPAction.update,
DamageAction.update, Lightning.onEvoke and the named orb single/all-hit actions,
ThornsPower.onAttacked, FirePotion.use. Existing static tools only, no original
execution. Same owned-archive SHA and unverified upstream build/channel limits.

- IDs: `GiantHead`, `Nemesis`, SlowPower `Slow` (DEBUFF), enemy IntangiblePower
  `Intangible`. A0 HP500/185; HeadCount13/GlareWeak1; NemesisTri6×3/Scythe45/
  TriBurn3 to discard.
- Head count starts5, decrements per getMove including opener to4. While >1,
  ticket0–49 chooses Glare unless newest2 both Glare, then Count;50–99 chooses
  Count unless newest2 both Count, then Glare. No secondary reroll. After action4
  the next roll reaches0, choosing Time for action5. Time decrements count to-6
  floor; base30-5*count produces30/35/40/45/50/55/60 then60. Cap is base, not
  Strength/Weak-modified output. Higher-Ascension findings are not implied.
- Slow starts0. onAfterUseCard queues+1 after that card's already queued payload;
  no card-type restriction, but dontTriggerOnUseCard suppresses this callback.
  NORMAL receive float multiplier is1+0.1*amount, additive not compounded, in
  target power-list order after attacker Strength/Weak and stance. One final
  binary32 floor; do not floor before Slow. End-round sets0 without removal.
  THORNS (including Lightning/potions) and HP_LOSS are excluded.
- Nemesis decrements cooldown per selection including opener. Opener0–49 Tri,
  50–99 Burn. Later0–29 selects Scythe if newest notScythe and cooldown≤0, setting
  cooldown2. Otherwise fair coin true chooses Burn if newest2 bothTri elseTri;
  false chooses Burn if newest notBurn elseTri.30–64 selects Tri unless newest2
  bothTri; then fair true chooses Scythe/set2 if cooldown≤0 elseBurn; falseBurn.
  65–99 selects Burn unless newestBurn; then fair true chooses Scythe/set2 if
  cooldown≤0, otherwiseTri. These are direct branches, not renormalized rates.
- No initial Intangible. takeTurn queues payload, then absent-only Intangible1,
  then RollMove. Native represents negative cooldown values as0 because only
  the≤0 predicate matters; no original seed/cursor/internal-integer parity claim.
- Intangible justApplied starts true. After enemy queue/actions drain, ordinary
  getNextAction invokes group end powers, invoking each living owner's end-turn
  callback. The first clears justApplied only, the next reduces1/removes. Thus
  player1 unprotected,2 protected,3 unprotected,4 protected in ordinary turns.
- Intangible final-receive callback caps values>1 at1, no type filter. Crucially,
  Nemesis.damage independently replaces every positive DamageInfo output by1
  before ordinary damage/Block. Named NORMAL, HP_LOSS, retaliation THORNS,
  Lightning Orb THORNS and FirePotion paths therefore cap per positive hit, zero
  remains0. HP_LOSS still bypasses Block. This does not cover direct HP mutation,
  general player Intangible, or arbitrary interrupted/skipped scheduler paths.

## Task6: Reptomancer / SnakeDagger

Named Reptomancer constructor/usePreBattleAction/getMove/takeTurn/canSpawn/damage/
die; SnakeDagger constructor/getMove/takeTurn/damage; MonsterHelper.getEncounter,
SpawnMonsterAction constructor/update, MonsterGroup.addMonster/queueMonsters,
SuicideAction constructor/update, AbstractMonster.die overloads, LoseHPAction,
DamageAction, GameActionManager addToTop/getNextAction/clearPostCombatActions and
the relevant AbstractRoom.update/endBattle transition. Existing selected static
tools only, no original execution. Same owned bytes/unverified upstream build.

- Constructor IDs are `Reptomancer` and `Dagger` (class SnakeDagger); controller
  separately checked those named retained constructor literals. A0 parentHP180–190,
  Snake13×2+Weak1, Big30; DaggerHP20–25, Wound9+one Wounddiscard then Explode25.
- Source array has4 tracked children. Initial group/action order is trackedslot1
  leftDagger,parent,trackedslot0 rightDagger. POSX210,-220,180,-250 and POSY75,115,
  345,335 by tracked index0..3. Smart insertion counts actors left of child; action
  order is slot3,slot1,parent,slot2,slot0, not array order. Reusing a dead/escaped
  source slot does not reuse its actor handle or erase historical actors.
- canSpawn counts ALL other non-dying group actors and requires≤3. getMove also
  disallows a third consecutive selectedSpawn. Four tracked source positions are
  an independent bound; A0construct1 perSpawn, scanning indices0up for vacancy.
  Constructor HP RNG and parent-array reservation precede queued group insertion.
- Opening Spawn; later integer0–32Snake unless newestSnake then recursive33–99
  inclusive;33–65Spawn unless two-newestSpawn or canSpawnfalse, then directSnake;
  66–99Big unless newestBig then recursive0–65inclusive. No flat thirds or universal
  no-repeat claim; direct fallback Snake may repeat a prior Snake.
- Initial others and spawned children receive MinionPower on their source paths.
  It does not make living actors invisible to combat readiness or give independent
  reward settlement for each Dagger. Full generic Fatal/reward pipeline is not
  audited by this finding.
- Dagger queues Wound9 then status, later NORMAL25 attack then selfHP_LOSS equal
  to currentHP captured at takeTurn queue construction, before that attack resolves.
  Self-loss invokes ordinary death callbacks if it kills; a lethal player hit
  precedes that self-loss. Do not reuse Exploder's suicide-first THORNS lifecycle.
- Parent die delegates ordinary death then adds HideHealthBar/Suicide to TOP for
  each other nondead/nondying grouped actor. The default SuicideAction delegates
  relicTrigger=true, sets gold/HP0 and calls die(true), including named death
  power/relic callbacks. Reverse group death order follows repeated top insertion;
  no invented lethal NORMAL damage or blanket Minion-ignore predicate.
- Qualified queued-state edge: one child constructed/reserved, SpawnMonsterAction
  already queued, then parent killed while two initial Daggers remain. Its immediate
  basically-dead check sees those living actors and does not clear postcombat
  actions. Top suicides do not clear the action list. getNextAction consumes the
  pending spawn without parent-alive guard; AbstractRoom completion waits for an
  empty action list. Thus this child inserts after parent cleanup and keeps combat
  live. This is static method-order evidence for that concrete ordinary queue, not
  every fault/mod/interruption. Native injected faults retain constructor/slot/
  prepared-child custody until actual insertion acknowledgment.

## Task10: Falling / Tomb of Lord Red Mask / Red Mask

Named Falling constructor/setCards/buttonEffect; CardHelper hasCardWithType/
returnCardOfType; CardGroup getGroupWithoutBottledCards/removeCard; TombRedMask
constructor/buttonEffect; AbstractRoom.spawnRelicAndObtain/AbstractRelic.obtain;
RedMask constructor/atBattleStart; AbstractPlayer.applyStartOfCombatLogic and
ApplyPowerAction.update. Existing selected static tools, same owned archive SHA
and unverified upstream build/channel. No execution or private source publication.

- IDs `Falling`, `Tomb of Lord Red Mask`, relic `Red Mask` (SPECIAL). Falling
  constructor setCards caches flags and exact object references before intro
  Continue. Attack/Skill/Power eligibility/sample call order; choices later present
  Skill/Power/Attack. Nonempty candidate lists call random(size-1), even singleton;
  this does not establish whether original bound0 advances RNG. Native RNG-001
  retains uniform instance support and canonical instance order, not original
  master-deck order seed mapping. Repeated UI reads/clicks do not resample.
- Original getGroupWithoutBottledCards excludes all three bottle flags; native M8
  has no such flags and does not claim them. Original eligible candidate traversal
  preserves master-deck order. Duplicate card kinds remain distinct objects.
  Missing types disabled; all types absent offers Leave, not free curse removal.
- Falling removes only its cached object with ordinary master-deck onRemove/
  relic onMasterDeckChange callbacks. It does NOT itself issue EmptyDeck damage
  or maxHP effects; native EmptyDeck-1 settlement is the declared separate gym
  rule. Source initial Continue is presentation around the already captured
  instances, not another sampling checkpoint.
- Tomb owned Mask option grants exactly222 gold; noMask option displays/currently
  pays ALL current gold, including0, then actually obtains one RedMask. Constructor
  preview is not acquisition. Effects precede RESULT, which offers Leave to map;
  no repeated payout/grant on the result screen. Original callback rechecks current
  ownership; native stale source choices fail closed rather than duplicate grant.
- Red Mask atBattleStart occurs once through owned relic startup iteration, not
  every turn. It enumerates the entire current group in order without a producer
  live filter and queues player-source Weak1 per target. ApplyPower execution's
  own target guard and ordinary Artifact/DEBUFF prevention apply separately.
  Initial membership is snapshotted; later summons do not receive a repeat pass.
- Other event eligibility/pools, bottled overrides, arbitrary startup callbacks,
  native injected faults and original RNG/runtime/version parity are not closed.

## Task9: Deca / Donu

Named MonsterHelper.getEncounter; Deca/Donu constructor/usePreBattleAction/getMove/
takeTurn/die, GainBlockAction.update, ApplyPowerAction.update and actual
AbstractCreature.isDeadOrEscaped. Same archive/static existing-tool scope and
unverified upstream build/channel; no original execution/private source published.

- IDs `Deca`/`Donu`, each250HP/Artifact2 at A0; actual group order Deca then Donu.
  Each is an independent boss, neither is a Minion or a merged shared actor.
- Deca own isAttacking starts true: Beam10x2 then twoDazeddiscard, then Square
  Block16 to group. Donu starts false: Circle groupStrength3 then Beam10x2. Each
  getMove only reads its own flag; takeTurn publication flips it. Repeated selection
  without execution does not advance the cycle. Ally death never resets it.
- Neither producer filters living actors while publishing group buffs: every
  current list entry gets its own action in list order. Later GainBlock execution
  skips isDying/isDead, not halfDead/isEscaping/source death. ApplyPower execution
  checks isDeadOrEscaped before and after callbacks; named helper means isDying,
  halfDead or monster isEscaping, not a direct isDead test. These different guards
  are not a universal definition of living. Ordinary pair has no escape/revival;
  arbitrary foreign actor/flag compositions remain unverified.
- Already published actions for a surviving target are not canceled merely by
  source death. Named die methods call ordinary death and gate victory on actual
  group defeat; with one survivor they do not clear normal actions. Both main
  bosses must be defeated. No generic paired-death shortcut or replacement.
- Artifact only prevents DEBUFF applications; group positiveStrength3 is a BUFF
  and Block16 is not a debuff. Surviving attacks use live Strength, not baked bases.
  Static findings do not prove original runtime, injected-fault parity or upstream
  target-version provenance.

## Task8: Time Eater / Time Warp / Draw Reduction

Named TimeEater constructor/usePreBattleAction/getMove/takeTurn/damage/die;
TimeWarpPower constructor/onAfterUseCard; DrawReductionPower constructor/
onInitialApplication/atEndOfRound/onRemove. Caller closure: AbstractPlayer
useCard/draw/updateInput, UseCardAction constructor/update, GameActionManager
getNextAction/callEndTurnEarlySequence/callEndOfTurnActions, MonsterGroup
applyEndOfTurnPowers, AbstractRoom update/endTurn and its inner action,
EndTurnButton.disable, NewQueueCardAction update/queueContainsEndTurnCard,
RemoveDebuffsAction and EndTurnAction. Same owned archive SHA and unverified
upstream build/channel; static existing tools, no original execution/public source.

- IDs `TimeEater`, `Time Warp`, `Draw Reduction`. A0 HP456; Reverberate7x3,
  HeadSlam26+DrawReduction1, RippleBlock20+Weak1/Vulnerable1.
- Haste strictly below halfHP, not at228; latch spent when selected. Its action
  removes DEBUFFs, explicitly Shackled, then heals delta228-currentHP. Existing
  Block is not explicitly cleared by this action. Positive Strength survives,
  negative Strength is a debuff; arbitrary harmful BUFFs are not generic debuffs.
- Ordinary selector also supplies opener.0-44Reverberate unless last2 both that,
  then recursively redraw50-99 inclusive;45-79HeadSlam unless lastHeadSlam then
  float0.66f trueReverberate/falseRipple;80-99Ripple unless lastRipple then recursive
  redraw0-74inclusive. Literal0.66f confirmed; qualified24-bit finite law gives
  11072963/16777216 true outcomes, not rounded66% or2/3. RNG-001 still applies.
- Time Warp starts0, persists across turns, increments onAfterUseCard. Exactly12
  resets0 before early-end request and queuedStrength2 for every encounter actor;
  that loop has no dead filter. dontTriggerOnUseCard suppresses this callback.
  Current payload/selection precedes after-use, which precedes disposition. Potions
  and rejected actions are not card uses. Normal early-end sequence preserves
  ordinary actions and current UseCardAction; queued autoplay is marked suppressed,
  cardQueue/limbo are cleared, then one null-card sentinel initiates normal end-turn
  actions after current cleanup. Never substitute an ordinary-action queue wipe or
  a thirteenth card-input opportunity. Arbitrary autoplay producers are not closed.
- Draw Reduction is DEBUFF/turn-based. Initial application reduces draw size by1
  independently of amount. First end-round only clears justApplied; later callbacks
  reduce duration1, removal restores size1. Enemy HeadSlam1 therefore affects the
  immediately upcoming start draw once, not twice. Retain wiki intensity/duration
  discrepancy; other draw-modifier/replacement compositions remain unverified.
- Named room/queue methods do not prove universal simultaneous-lethal arbitration
  or native injected-fault recovery. Preserve native sequential terminal/fault
  ownership without calling it original-runtime parity or target-version proof.

## Task7: Awakened One / Inflame / Void

Named MonsterHelper.getEncounter; AwakenedOne constructor/usePreBattleAction/
getMove/takeTurn/changeState/damage/die; CuriosityPower constructor/onUseCard;
RegenerateMonsterPower constructor/atEndOfTurn; StrengthPower.updateDescription;
Inflame constructor/use/upgrade; VoidCard constructor/triggerWhenDrawn/use/upgrade.
Caller closure: AbstractPlayer.useCard/draw/loseEnergy, UseCardAction constructor/
update, CardGroup.empower, ClearCardQueueAction, MakeTempCardInDrawPileAction
constructor/update, LoseEnergyAction/EnergyPanel.useEnergy, HealAction/CanLoseAction.
Existing selected static tools only, no original execution. Same archive/upstream
build qualifications. Independently worded facts; no original code distributed.

- Encounter order Cultist/Cultist/AwakenedOne; A0 boss300HP, Curiosity1,
  Regenerate10 and Unawakened. Constructor IDs AwakenedOne, Curiosity, Regenerate,
  Inflame and Void. Cultists are actual actors, not Minion-tagged substitutes.
- First openerSlash20; later0–24Soul6x4 unless newestSoul thenSlash;25–99Slash
  unless newest2Slash thenSoul. After Rebirth forcedDarkEcho40; later0–49Sludge18
  unless newest2Sludge thenTackle10x3,50–99Tackle unless newest2Tackle thenSludge.
- Curiosity checks actual POWER use. Unsuppressed UseCardAction construction
  dispatches after card.use has queued the card payload, then queues bossStrength
  equal to Curiosity amount, before the later use/disposition action. No in-hand,
  preview, per-hit or arbitrary suppressed-autoplay acceptance is implied.
- Regenerate is nondecaying owner-end-turn Heal(amount), only while not halfDead/
  dying/dead. The boss is last in this captured formation; normal end-round follows
  its own payload/queued effects. This is not the player's decaying Regen lifecycle.
- First depletion marks halfDead while cannotLose, invokes power onDeath/relic
  onMonsterDeath callbacks, clears cardQueue (NOT ordinary actions), and filters
  powers. **Remove DEBUFF**, plus explicit Curiosity/Unawakened/Shackled IDs;
  preserve other buffs including positiveStrength. Strength's negative/zero form
  is DEBUFF and removed. Controller personally checked the branch to removal;
  an assistant's initial inverted reading was corrected before any native oracle.
- Rebirth is later queued ChangeState: phase2maxHP300, clearHalfDead, max-HP Heal,
  then CanLose. Same actor/powers not removed by that explicit filter persist;
  only final phase allows actual death and source EscapeActions for living Cultists.
  Early dead Cultists are not recreated; escape is not a fabricated death.
- Inflame1energy grants2/3Strength and is a real POWER. Actual UseCardAction uses
  empower/Soul presentation, not discard/Exhaust. Native owned inactive Power zone
  preserves permanent identity for public history/exit/Run census without copying
  or inventing a new acquisition. Legacy snapshot format is not extended by it.
- Void: STATUS, unplayable(-2), Ethereal, no-op upgrade. Actual draw queues loss1,
  clamped at0; placement/creation/copy/reveal alone does not. Sludge's actual
  MakeTempCardInDrawPile call has randomSpot/autoPosition/toBottom true; toBottom
  wins, so it inserts at BOTTOM, not random. No permanent acquisition follows.
- These are named A0/source-path findings, not original runtime, archive PCv2.3
  provenance, every card/fault composition or all injected foreign-object behavior.
