# Reachable Watcher cards: paired normative records

Date: 2026-10-19. Bead: `spire-gli.2.3.21`.

**Draft for independent review.** These records supply the missing B07 primary
card content: four distinct basics, 71 nonbasics (19 common, 35 uncommon, 17 rare),
eight physical dependent specials, and three nonphysical Wish options. They do not
claim full transitive execution closure, human approval, or implementation authority.
The exact outstanding fields are listed in [section 9](#9-exact-open-fields).

## 1. Scope, evidence, and how to read the records

The run is standard, unmodded, fully unlocked, Act-4-enabled **Ironclad A0 Act 1**.
Prismatic Shard rewards and legal A Note For Yourself retrieval are acquisition
routes, not changes to the run character. Watcher basics retain their own kind
identities; they do not join the ordinary reward pool. There is no Watcher start,
Pure Water, initial Miracle, Watcher-only potion/relic pool, or imported home event.
Fixed dependent specials can be created by the listed cards without granting their
parent character's starting resources. Legal Note storage is not a universal
permission to store generated specials or option displays.

Evidence grades used here:

* **P**: previously independently reviewed public StS 1 primary descriptions,
  independently restated here. [Roster](https://slaythespire.gg/cards/watcher) and
  the individual linked card pages support the paired primary values, not complete
  engine timing. The old reviewed draft is read-only evidence, not a required
  unpublished import. No bulk prose was copied from it.
* **S**: static inspection of owned archive SHA-256
  `cfad868ac8d65a88e71a0bf096fb09f78811e553effe0787c5309a655e081673`.
  Retained constructor/bytecode evidence covers 74 purple kinds; the missing
  Foreign Influence card body was inspected separately. Retained action/copy bodies
  and narrowly inspected missing members support sections 6–8.
  The private claim ledger maps each record to exact members, hashes and logs.
  This pins owned bytes, not an upstream version/channel or executed runtime.
* **V**: private standard-library guard, inventory and document checks. These
  check evidence handling and written coverage, not original game behavior.

The tables' primary pairs are **P**, with **S** constructor support for purple
base fields. More precise S findings below override incomplete P prose—especially
Halt+, Expunger, Mantra, Spirit Shield and Omniscience. No original game class,
initializer, launcher, substituted game runtime or gameplay test was executed.

Notation: `a/b` means **base/upgraded**, including cost and flags. A single value
applies to both variants. `A/S/P` = Attack/Skill/Power. Targets are semantic:
`enemy` requires one selected enemy; `all` affects enemies without selecting one;
`random` delegates target choice to the effect; `self` concerns the player, piles
or a non-enemy modal selection. This is not a claim that every self effect has the
source target enum SELF: Wish, Omniscience and several others use NONE; Vault uses
ALL; Ragnarok uses ALL_ENEMY but makes separate random attacks.

`E` = Exhaust-on-play, `R` = inherent self-Retain, `I` = Innate, `Eth` = Ethereal.
An absent flag is false in both variants; `–/R`, for example, adds Retain only on
upgrade. Powers leave play through the Power route, not the ordinary discard
route. Other ordinary cards discard unless the record names Exhaust or Tantrum's
self-return; origin-specific purge/exhaust overrides remain separate. Base and
upgraded instances are independently specified; upgrading a parent does not
implicitly upgrade its children. All printed damage/Block precedes modifiers.
Clause order identifies local requests, not a universal drain-between-clauses law.

## 2. Four Note-reachable basics

| Kind | Type; target; cost | Flags | Base/upgraded payload and state |
|---|---|---|---|
| [Watcher Strike](https://slaythespire.gg/cards/watcher/Strike) | A; enemy; 1 | — | Attack for 6/9. Kind `Strike_P`, not Ironclad Strike; STRIKE and STARTER_STRIKE tags. |
| [Watcher Defend](https://slaythespire.gg/cards/watcher/Defend) | S; self; 1 | — | Block 5/8. Kind `Defend_P`; STARTER_DEFEND tag. |
| [Eruption](https://slayspiredb.com/sts1/cards/eruption) | A; enemy; 2/1 | — | Attack for 9, then request Wrath. A Neutral first hit is not retroactively doubled. |
| [Vigilance](https://slayspiredb.com/sts1/cards/vigilance) | S; self; 2 | — | Block 8/12, then request Calm. |

## 3. Nineteen common nonbasics

| Kind | Type; target; cost | Flags | Base/upgraded payload and state |
|---|---|---|---|
| [Bowling Bash](https://slayspiredb.com/sts1/cards/bowling-bash) | A; enemy; 1 | — | 7/10 per hit; enqueue one hit against the selected target for each combat monster not dead or escaped when `use` counts it. Not one multiplied hit. |
| [Consecrate](https://slayspiredb.com/sts1/cards/consecrate) | A; all; 0 | — | Attack each enemy for 5/8. |
| [Crescendo](https://slayspiredb.com/sts1/cards/crescendo) | S; self; 1/0 | R,E | Request Wrath. |
| [Crush Joints](https://slayspiredb.com/sts1/cards/crush-joints) | A; enemy; 1 | — | Attack for 8/10; previous-card Skill predicate requests 1/2 Vulnerable on that enemy. See combat-history boundary below. |
| [Cut Through Fate](https://slayspiredb.com/sts1/cards/cut-through-fate) | A; enemy; 1 | — | Attack for 7/9; Scry 2/3; draw 1 after the Scry continuation. |
| [Empty Body](https://slayspiredb.com/sts1/cards/empty-body) | S; self; 1 | — | Block 7/10; request Neutral stance. EMPTY tag. |
| [Empty Fist](https://slayspiredb.com/sts1/cards/empty-fist) | A; enemy; 1 | — | Attack for 9/14; request Neutral. EMPTY tag. |
| [Evaluate](https://slayspiredb.com/sts1/cards/evaluate) | S; self; 1 | — | Block 6/10; create one base Insight in draw pile through the insertion route. |
| [Flurry of Blows](https://slayspiredb.com/sts1/cards/flurry-of-blows) | A; enemy; 0 | — | Attack for 4/6. Actual stance change notifies discard instances to request their return to hand. |
| [Flying Sleeves](https://slayspiredb.com/sts1/cards/flying-sleeves) | A; enemy; 1 | R | Two separate 4/6-damage hits. |
| [Follow-Up](https://slayspiredb.com/sts1/cards/follow-up) | A; enemy; 1 | — | Attack for 7/11; previous-card Attack predicate requests 1 energy. |
| [Halt](https://slayspiredb.com/sts1/cards/halt) | S; self; 0 | — | Base Block 3/4; in Wrath an additional separately modified 9/14 component is included in **one** gain. Unmodified totals in Wrath are 12/18. See section 8. |
| [Just Lucky](https://slayspiredb.com/sts1/cards/just-lucky) | A; enemy; 0 | — | Scry 1/2; Block 2/3; attack for 3/4. The pending choice precedes later payload requests. |
| [Pressure Points](https://slayspiredb.com/sts1/cards/pressure-points) | S; enemy, then all; 1 | — | Request 8/11 Mark on target; trigger all enemies' existing Mark amounts as HP loss. Application prevention does not erase old Mark. Kind `PathToVictory`. |
| [Prostrate](https://slayspiredb.com/sts1/cards/prostrate) | S; self; 0 | — | Request 2/3 Mantra; Block 4. |
| [Protect](https://slayspiredb.com/sts1/cards/protect) | S; self; 2 | R | Block 12/16. |
| [Sash Whip](https://slayspiredb.com/sts1/cards/sash-whip) | A; enemy; 1 | — | Attack for 8/10; previous-card Attack predicate requests 1/2 Weak on target. |
| [Third Eye](https://slayspiredb.com/sts1/cards/third-eye) | S; self; 1 | — | Block 7/9; Scry 3/5. |
| [Tranquility](https://slayspiredb.com/sts1/cards/tranquility) | S; self; 1/0 | R,E | Request Calm. Kind `ClearTheMind`. |

## 4. Thirty-five uncommon nonbasics

| Kind | Type; target; cost | Flags | Base/upgraded payload and state |
|---|---|---|---|
| [Battle Hymn](https://slayspiredb.com/sts1/cards/battle-hymn) | P; self; 1 | –/I | Per start-turn trigger create one base Smite in hand; stacks increase count, not child upgrade. |
| [Carve Reality](https://slayspiredb.com/sts1/cards/carve-reality) | A; enemy; 1 | — | Attack for 6/10; create one base Smite in hand. |
| [Collect](https://slayspiredb.com/sts1/cards/collect) | S; self; X | E | Install delayed creation of one **Miracle+** per turn for effective X/X+1 future turns. Chemical X and payment are separate; section 7. |
| [Conclude](https://slayspiredb.com/sts1/cards/conclude) | A; all; 1 | — | Attack each enemy for 12/16; request end turn. |
| [Deceive Reality](https://slayspiredb.com/sts1/cards/deceive-reality) | S; self; 1 | — | Block 4/7; create one base Safety in hand. |
| [Empty Mind](https://slayspiredb.com/sts1/cards/empty-mind) | S; self; 1 | — | Draw 2/3; request Neutral. EMPTY tag. |
| [Fasting](https://slayspiredb.com/sts1/cards/fasting) | P; self; 2 | — | Request 3/4 Strength, then 3/4 Dexterity, then 1 Energy Down debuff. Energy Down requests that many energy lost at start turn; Artifact can block the debuff, not the preceding buffs. Kind `Fasting2`. |
| [Fear No Evil](https://slayspiredb.com/sts1/cards/fear-no-evil) | A; enemy; 1 | — | Attack for 8/11; request Calm if action-time intent is ATTACK, ATTACK_BUFF, ATTACK_DEBUFF or ATTACK_DEFEND. Hit is queued ahead of that stance request. Hidden intent must not leak through legality. |
| [Foreign Influence](https://slayspiredb.com/sts1/cards/foreign-influence) | S; self; 0 | E | Offer three distinct eligible any-color Attacks; selected instance enters hand, or discard at hand size 10. Upgraded parent sets chosen instance's turn cost to 0. Source policy below. |
| [Foresight](https://slayspiredb.com/sts1/cards/foresight) | P; self; 1 | — | Start-turn Scry 3/4; explicitly request empty-deck shuffle first if draw pile is empty. Kind `Wireheading`. |
| [Indignation](https://slayspiredb.com/sts1/cards/indignation) | S; all/self; 1 | — | In Wrath request 3/5 Vulnerable on enemies; otherwise request Wrath. Mutually exclusive branches, evaluated for that resolution. |
| [Inner Peace](https://slayspiredb.com/sts1/cards/inner-peace) | S; self; 1 | — | In Calm draw 3/4; otherwise request Calm. Not both branches. |
| [Like Water](https://slayspiredb.com/sts1/cards/like-water) | P; self; 1 | — | Player pre-end-turn-card hook, if then in Calm, requests its installed Block amount, initially 5/7. Stacking adds amounts with explicit upper clamp 999. |
| [Meditate](https://slayspiredb.com/sts1/cards/meditate) | S; self; 1 | — | Retrieve 1/2 discard instances, granting temporary Retain to moved cards; request Calm; request end turn. Exact full-hand/undersized branches in section 6. |
| [Mental Fortress](https://slayspiredb.com/sts1/cards/mental-fortress) | P; self; 1 | — | Actual stance change requests Block 4/6; stacks add amount. No same-stance trigger. |
| [Nirvana](https://slayspiredb.com/sts1/cards/nirvana) | P; self; 1 | — | `onScry` requests Block 3/4; stacks add amount. This notification can occur before an empty-pile return. |
| [Perseverance](https://slayspiredb.com/sts1/cards/perseverance) | S; self; 1 | R | Initially Block 5/7. Each actual `onRetained` increases this instance's base Block by 2/3. Upgrade after growth adds 2 Block and changes future growth by +1; old growth is not recalculated. |
| [Pray](https://slayspiredb.com/sts1/cards/pray) | S; self; 1 | — | Request 3/4 Mantra; create one base Insight in draw pile. |
| [Reach Heaven](https://slayspiredb.com/sts1/cards/reach-heaven) | A; enemy; 2 | — | Attack for 10/15; create one base Through Violence in draw pile. |
| [Rushdown](https://slayspiredb.com/sts1/cards/rushdown) | P; self; 1/0 | — | Actual transition into Wrath requests draw 2; stacks add draw count. Kind `Adaptation`. |
| [Sanctity](https://slayspiredb.com/sts1/cards/sanctity) | S; self; 1 | — | Block 6/9; previous-card Skill predicate requests draw 2. |
| [Sands of Time](https://slayspiredb.com/sts1/cards/sands-of-time) | A; enemy; 4 | R | Attack for 20/26. Each actual `onRetained` queues a combat-cost reduction of 1 on this instance; nonnegative combat cost floors at 0. |
| [Signature Move](https://slayspiredb.com/sts1/cards/signature-move) | A; enemy; 2 | — | Attack for 30/40. Its specific `canUse` rejects if another Attack instance is in hand; see origin qualification below. |
| [Simmering Fury](https://slayspiredb.com/sts1/cards/simmering-fury) | S; self; 1 | — | Install next-turn Wrath and next-turn draw 2/3 as distinct powers. Kind `Vengeance`; these are not one indivisible start-turn action. |
| [Study](https://slayspiredb.com/sts1/cards/study) | P; self; 2/1 | — | End-turn create one base Insight in draw pile per stacked count. |
| [Swivel](https://slayspiredb.com/sts1/cards/swivel) | S; self; 2 | — | Block 8/11; gain one Free Attack entitlement. It has no automatic turn-end expiration. Qualifying non-purge Attack use decrements a positive count. |
| [Talk to the Hand](https://slayspiredb.com/sts1/cards/talk-to-the-hand) | A; enemy; 1 | E | Attack for 5/7; install enemy Block Return debuff of 2/3. Its own preceding hit does not receive the newly installed effect. Artifact can block installation. |
| [Tantrum](https://slayspiredb.com/sts1/cards/tantrum) | A; enemy; 1 | self-return | Enqueue 3/4 separate hits of 3, then request Wrath. Ordinary cleanup returns the same instance to draw pile; not creation of a fresh Tantrum. |
| [Wallop](https://slayspiredb.com/sts1/cards/wallop) | A; enemy; 2 | — | Attack for 9/12; after target damage returns, positive target `lastDamageTaken` requests that much Block. Exact post-lethal survival of this queued Block is not established. |
| [Wave of the Hand](https://slayspiredb.com/sts1/cards/wave-of-the-hand) | S; self/all; 1 | — | Positive `onGainedBlock` amount requests 1/2 Weak on each enemy; stack amounts add. Removal is queued at end round, not immediately on next card. |
| [Weave](https://slayspiredb.com/sts1/cards/weave) | A; enemy; 0 | — | Attack for 4/6; each discard-pile Scry notification requests return of this instance to hand. |
| [Wheel Kick](https://slayspiredb.com/sts1/cards/wheel-kick) | A; enemy; 2 | — | Attack for 15/20; draw 2. |
| [Windmill Strike](https://slayspiredb.com/sts1/cards/windmill-strike) | A; enemy; 2 | R | Initially attack for 7/10. Each actual `onRetained` adds 4/5 base damage to this combat instance. Upgrade after growth adds 3 damage and changes future growth by +1. STRIKE tag. |
| [Worship](https://slayspiredb.com/sts1/cards/worship) | S; self; 2 | –/R | Request 5 Mantra; upgrade adds Retain, not more Mantra. |
| [Wreath of Flame](https://slayspiredb.com/sts1/cards/wreath-of-flame) | S; self; 1 | — | Gain 5/8 Vigor: add that amount to NORMAL damage in its damage-give hook; an Attack-use callback queues removal. Multi-hit capture and mixed-repeat timing are not inferred from the word “next.” |

## 5. Seventeen rare nonbasics

| Kind | Type; target; cost | Flags | Base/upgraded payload and state |
|---|---|---|---|
| [Alpha](https://slayspiredb.com/sts1/cards/alpha) | S; self; 1 | E; –/I | Create one base Beta in draw pile. Upgrade affects Innate only. |
| [Blasphemy](https://slayspiredb.com/sts1/cards/blasphemy) | S; self; 1 | E; –/R | Request Divinity; install End Turn Death BUFF. At start turn it queues 99,999 HP loss and its own removal; not an unconditional defeat assignment. |
| [Brilliance](https://slayspiredb.com/sts1/cards/brilliance) | A; enemy; 1 | — | Attack base 12/16 plus the combat Mantra-gained accumulator, before damage modifiers. Current unspent Mantra is not that accumulator. |
| [Conjure Blade](https://slayspiredb.com/sts1/cards/conjure-blade) | S; self; X | E | Create one Expunger in draw pile, storing effective captured X/X+1 with Chemical X +2. One child is requested even at zero effective count; detailed sentinel/payment rules in section 7. |
| [Deus Ex Machina](https://slayspiredb.com/sts1/cards/deus-ex-machina) | S; self; unplayable | E | Actual draw callback requests 2/3 base Miracles in hand and self-exhaust. Movement/creation without the draw callback does not trigger it. Still a Skill, not a Status. |
| [Deva Form](https://slayspiredb.com/sts1/cards/deva-form) | P; self; 3 | Eth/– | Energy-recharge hook grants an increasing amount; initial grant 1 and increment 1. Upgrade removes Ethereal, not cost or initial grant. Exact stacking in section 8. |
| [Devotion](https://slayspiredb.com/sts1/cards/devotion) | P; self; 1 | — | Post-draw start-turn request of stacked Mantra amount, initially 2/3, with special no-existing-Mantra branch at amount ≥10. |
| [Establishment](https://slayspiredb.com/sts1/cards/establishment) | P; self; 1 | –/I | End-turn action discounts eligible hand cards by stacked amount (1 per installation). Selection is by `retain` or `selfRetain` at action execution, not a generic non-discarded-card test. |
| [Judgment](https://slayspiredb.com/sts1/cards/judgment) | S; enemy; 1 | — | If action-time target HP ≤30/40 and target is a monster, request Instant Kill. Source spelling `Judgement`. Not a 30/40-damage Attack. |
| [Lesson Learned](https://slayspiredb.com/sts1/cards/lesson-learned) | A; enemy; 2 | E | Attack for 10/13; qualifying fatal branch upgrades one eligible permanent-deck instance immediately. HEALING tag excludes it from healing-excluding generators. |
| [Master Reality](https://slayspiredb.com/sts1/cards/master-reality) | P; self; 1/0 | — | Enable route-specific creation upgrades; not retroactive upgrading of existing combat/permanent cards. Multiple routes and copy paths are not interchangeable. |
| [Omniscience](https://slayspiredb.com/sts1/cards/omniscience) | S; self; 4/3 | E | Select one draw-pile instance; set its Exhaust flag, remove it, enqueue it and one stat-equivalent purge-on-use copy. Each child queue request is `randomTarget=true, immediateCard=false, autoplayCard=true, ignoreEnergyTotal=false`; section 6 distinguishes the later derived card state. |
| [Ragnarok](https://slayspiredb.com/sts1/cards/ragnarok) | A; random; 3 | — | Enqueue 5/6 random-enemy attack actions, each with base damage 5/6. Each action owns its target/damage calculation; not one target or combined hit. |
| [Scrawl](https://slayspiredb.com/sts1/cards/scrawl) | S; self; 1/0 | E | Request enough draws to fill a ten-card hand. Completed draws still depend on draw prevention, available cards and intervening draw callbacks. |
| [Spirit Shield](https://slayspiredb.com/sts1/cards/spirit-shield) | S; self; 2 | — | At `use`, count hand instances other than this object; base Block = count ×3/4; apply Block modifiers once, request one gain. Not Dexterity once per card. |
| [Vault](https://slayspiredb.com/sts1/cards/vault) | S; self; 3/2 | E | Request the source's extra player turn/end-turn path, suppressing the ordinary enemy turn. Does not waive end/start-turn effects. Exact mixed continuation arbitration remains open. |
| [Wish](https://slayspiredb.com/sts1/cards/wish) | S; self; 3 | E | Choose one of: 3/4 Strength, 25/30 gold, 6/8 Plated Armor. Option order is Strength → gold → Plated Armor. These are option callbacks, not three acquired cards. HEALING tag. |

### 5.1 Eight physical dependent specials

These have colorless SPECIAL identities, not ordinary Watcher reward rarity.
Their reachable creation routes do not establish Note eligibility.

| Kind | Type; target; cost | Flags | Base/upgraded payload and state |
|---|---|---|---|
| [Safety](https://slayspiredb.com/sts1/cards/safety) | S; self; 1 | R,E | Block 12/16. Deceive Reality's immediate Block is separate. |
| [Smite](https://slayspiredb.com/sts1/cards/smite) | A; enemy; 1 | R,E | Attack for 12/16. |
| [Insight](https://slayspiredb.com/sts1/cards/insight) | S; self; 0 | R,E | Draw 2/3. |
| [Through Violence](https://slayspiredb.com/sts1/cards/through-violence) | A; enemy; 0 | R,E | Attack for 20/30. |
| [Miracle](https://slayspiredb.com/sts1/cards/miracle) | S; self; 0 | R,E | Gain 1/2 energy. Generation does not grant starting Miracles. |
| [Beta](https://slayspiredb.com/sts1/cards/beta) | S; self; 2/1 | E | Create one base Omega in draw pile. |
| [Omega](https://slayspiredb.com/sts1/cards/omega) | P; all; 3 | — | End-turn triggered damage 50/60 to each enemy. Not a played Attack; Wrath/Divinity's NORMAL multipliers are not blanket multipliers on this effect. |
| [Expunger](https://slayspiredb.com/sts1/cards/expunger) | A; enemy; 1 | — | Stored count h gives h separate hits of 9/15. Cost stays 1; upgrading damage does not add a hit to an already captured h. |

### 5.2 Three Wish option records

| Option kind | Type; source cost/target | Base/upgraded callback payload |
|---|---|---|
| Become Almighty (`BecomeAlmighty`) | Power; −2/NONE | Request Strength 3/4. |
| Fame and Fortune (`FameAndFortune`) | Skill; −2/NONE | Request gold 25/30, not the visual coin count (which is twice the payload). |
| Live Forever (`LiveForever`) | Power; −2/NONE | Request Plated Armor 6/8. |

Wish constructs options in the listed order and explicitly upgrades all three for
Wish+. Choosing invokes exactly one option's callback; it does not play that
display as a hand card, pay −2 energy, or run ordinary Power-card cleanup. Gold
goes through gain-gold authority, so Ectoplasm/prevention and live-resource hooks
remain applicable. No automatic strategic choice or random branch is permitted.

## 6. Stance, Mantra, Retain, Scry, and ordered choices

### 6.1 Stance transitions (S)

Combat starts Neutral, without Watcher resources. An actual different-stance
request not blocked by Cannot Change Stance has this local sequence:

1. Notify installed player powers, then relics, with old/new stances.
2. Run old stance exit; assign new stance; run new stance entry.
3. Update entry history and notify the player's stance machinery, discard-pile
   stance-reactive cards, and player stance callback.

Same-ID requests skip those transition effects and notifications. A Neutral exit
while already Neutral gives no Calm energy and no Mental Fortress/Rushdown/Flurry
trigger. Local callback invocation order does **not** mean their queued actions
finish before the stance assignment or before subsequent callbacks.

| Stance | Local law |
|---|---|
| Neutral | No inherent stance damage multiplier or energy grant. |
| Calm | Actual exit queues gain 2 energy. Entry itself grants none. |
| Wrath | NORMAL damage-give and damage-receive hooks multiply binary32 input by 2. No same-Wrath re-entry benefit. |
| Divinity | Actual entry queues gain 3 energy; NORMAL damage-give hook multiplies by 3. Start-turn hook queues Neutral; it does not synchronously exit before every other start-turn callback. |

Calm→Divinity therefore requests both exit energy 2 and entry energy 3; a same-
Divinity request grants neither fresh entry energy nor transition reactions.
NORMAL is the source damage category, not “every HP change.” Direct HP loss,
Mark and triggered THORNS damage do not acquire a stance multiplier by prose.
Eruption/Tantrum hits are requested before their Wrath transition. Damage capture,
recalculation and terminal interruption are still source-specific.

### 6.2 Mantra is not generic modulo arithmetic (S)

Maintain **current Mantra amount**, **Mantra-gained combat history**, and stance as
different fields. Constructing `MantraPower(owner,a)` increments the combat history
by a before its later application; Brilliance reads that accumulator when damage
is calculated, adds it to base damage before modifiers, then restores base damage.
Do not reconstruct history from remaining Mantra or from number of Divinity entries.

For stacking an existing Mantra power, the source adds the incoming stack, then
checks the resulting amount **once**. At ≥10 it requests Divinity and subtracts
10 exactly once; a nonpositive remainder also requests removal. A 9+3 stack leaves
2; a 9+23 stack leaves 22, not 2, and requests one stance change, not three. The
check is not a loop; this does not prove that every illustrative large operand is
reachable at a given point in a run.

First installation follows Apply Power's install path and the default no-op
initial-application hook, not that stacking check. Devotion's post-draw hook has
its own important exception: **with no existing Mantra and stacked Devotion amount
≥10, it requests Divinity directly instead of constructing Mantra**. That branch
does not increment history through a Mantra constructor or store excess Mantra.
Otherwise Devotion constructs/applies its full 2/3-per-install stacked amount.
Existing-Mantra gains while already Divine still consume one threshold; the
same-stance request does not produce another entry-energy grant. No invented cap,
discard-all-excess rule or repeated-threshold energy is added.

### 6.3 Retain, growth and discounts (S)

`selfRetain` is inherent; `retain` is a temporary grant. The inspected restore
action invokes `onRetained`, restores eligible instances to hand, then clears the
temporary flag. This is not a rule that every card left in hand by Runic Pyramid
or an all-hand non-discard effect received a Retain callback. Ethereal exhaustion
is a separate end-turn action and is not canceled merely by spelling “Retain.”
Exact cross-source end-turn ordering remains W-END below.

Perseverance and Windmill mutate combat base fields additively at each actual
callback. Upgrading does not retroactively apply the larger growth rate to prior
retentions. Sands queues Reduce Cost on that exact card; Establishment separately
queues an end-turn scan of current hand cards with either Retain flag, applying
its stacked discount. They can both affect Sands, through distinct operations.
The inspected `modifyCostForCombat` is not a “preserve cost difference” operation:
if turn cost is positive, add the discount to that turn cost, floor at zero, then
assign combat cost to that result. Otherwise, if combat cost is nonnegative, add
the discount to combat cost with zero floor and set turn cost to zero. If neither
condition holds, do nothing (the ordinary X/unplayable-sentinel case). These are
not permanent deck edits or universal cost-precedence rules.

Meditate's local return action:

* Empty discard or nonpositive requested count: no return; later Calm/end-turn
  requests are still separate queued work.
* Discard size ≤ requested count: traverse discard group order; set Retain on all
  candidates first; move each only while hand size <10. An overflow candidate
  remains in discard **with the Retain flag set** in this branch.
* Otherwise open mandatory selection for 1/2 instances. On resumption traverse the
  selected-card list; move and set Retain only while hand size <10. Overflow
  selections remain in discard without receiving a new flag in this branch.

The selected-list traversal order matters when only one hand slot remains.
Mapping user click order to that list is not proved here; W-CHOICE retains that
specific missing field. It must not silently become an unordered set or an
arbitrary sort. The bulk undersized path is not an externally chosen permutation.

### 6.4 Scry and continuation boundaries (S)

Scry aborts before notifications if enemies are already basically dead. Otherwise
its initial step notifies powers' `onScry` **before** checking for an empty draw
pile. Thus Nirvana can request Block even when nothing is revealed. Ordinary Scry
does not itself reshuffle: reveal at most the requested top count available, allow
discard of any subset including none, and do not reorder the kept cards. Golden
Eye adds 2 to the requested count at Scry action construction; it is not a Watcher
exclusive resource import. Foresight explicitly supplies a separate empty-pile
shuffle before its Scry request.

Selected cards move to discard in selected-list order. After that local selection
handling, the Scry update traverses the then-current discard group and invokes
`triggerOnScry`; Weave queues a return action. Newly Scry-discarded Weave is
therefore not excluded just because it was in draw at Scry start. The method can
also reach that traversal on update passes without a nonempty selection; this is
not evidence for a universal exactly-one-Weave-notification-per-Scry rule. Modal
update counts, selected-list construction and return-action de-duplication require
the W-CHOICE/W-CALLBACK fields below.

Cut Through Fate must resume its pending Scry before its draw; Just Lucky before
its later Block/hit. A resumed choice does not repay, repeat the hit or reroll the
parent. Expose only revealed ordered instances and legitimate remembered knowledge,
not hidden remainder/order or RNG state. Choosing none is a valid confirmation,
not automatic strategic selection.

### 6.5 Omniscience and combat-history predicates (S)

Omniscience opens a one-instance draw selection, sorting its display
alphabetically and then by rarity/status type rather than granting hidden top-order
knowledge. Empty draw finishes without a selection. After selection it marks the
original Exhaust, removes it from draw/souls, and enqueues original then a
**stat-equivalent copy** with purge-on-use. Those requests are assembled before
either child's later resolution; do not replace them with “resolve the first
fully, then decide whether to construct the second.” Stat-equivalent copy makes a
fresh UUID; it is not the separate same-instance copy operation. Each NewQueue
request independently captures current energy **when that NewQueue action
executes**, and specifies `randomTarget=true, immediateCard=false,
autoplayCard=true`. NewQueue constructs a CardQueueItem with
`ignoreEnergyTotal=false`; it passes the captured energy, random-target flag and
autoplay flag through. Thus the queue record is exactly
`autoplayCard=true, ignoreEnergyTotal=false` for both children—not the
`true/true` ignore-total/autoplay queue route used by repeat powers.

When GameActionManager later processes each item, it first assigns the card's
`isInAutoplay` from `autoplayCard`, then runs `canUse`. Only after that gate passes
does it assign `energyOnUse` from the captured queue field and derive the distinct
card field `ignoreEnergyOnUse=true` from `isInAutoplay`; otherwise that card field
would follow the queue's `ignoreEnergyTotal`. The manager's derived per-card value
does not rewrite the queue flag. `AbstractPlayer.useCard` uses autoplay for its
autoplay/free-payment path, while captured X remains a separate amount and the
original remains marked Exhaust. This supports the ordinary no-energy-payment
conclusion, but not a false queue provenance claim or a universal legality bypass.
Nested modal choices still suspend the actual execution path; terminal/turn-ending
arbitration is not closed.

The Omniscience queued child plays are autoplayed without ordinary hand-energy
payment, but their queue items have `ignoreEnergyTotal=false`. The manager derives
the card's distinct `ignoreEnergyOnUse=true` only after assigning autoplay and
passing `canUse`; this later card field is not the queue provenance flag. By
contrast, the five repeat powers (Double Tap, Burst, Amplify, Duplication, Echo)
copy energy-on-use and explicitly enqueue both queue flags true:
`ignoreEnergyTotal=true, autoplayCard=true`. Preserve this source distinction.
Autoplay is not a universal legality bypass. Signature Move's inspected `canUse`
first checks the shared gate, then rejects another Attack object in hand even if
that object is unplayable. Its result depends on the actual selected instance/copy
and origin.

Crush Joints, Follow-Up and Sanctity action predicates require at least two combat
history entries and inspect entry `size−2`, expecting the current play at the end;
they do not clear history at turn boundary. Sash Whip uses the corresponding
previous-Attack predicate. Glow checks of the latest pre-play entry are not the
payload predicate. Intervening/repeat history mutations before a queued predicate
remain W-CALLBACK, not an assumed immutable captured “previous card.”

## 7. Creation, payment and copy capture

### 7.1 Source-specific producers

| Producer | Child and destination, unchanged by parent upgrade unless stated |
|---|---|
| Battle Hymn; Carve Reality | Base Smite to hand; Hymn creates its stacked count on start-turn trigger only if monsters are not basically dead. |
| Deceive Reality | Base Safety to hand. |
| Evaluate; Pray; Study | Base Insight to draw; Study's end-turn trigger creates its stacked count. |
| Reach Heaven | Base Through Violence to draw. |
| Alpha; Beta | Base Beta / base Omega to draw. Alpha+ does not directly create Beta+; Beta+ does not directly create Omega+. |
| Deus Ex Machina | Actual draw requests 2/3 base Miracles and self-exhaust; the self-exhaust front insertion is ahead of the earlier creation request in pending order. |
| Collect | Delayed upgraded Miracle, one each eligible recharge trigger, not X immediate cards; stacked duration clamps at 999. |
| Conjure Blade | Newly constructed Expunger with captured hit count, through draw-pile temporary creation. |
| Foreign Influence | Any-color Attack offer policy below; selected copy to hand/discard, + turn cost 0. |
| Omniscience | Existing selected original plus stat-equivalent purge copy, not a fresh random kind. |
| Meditate; Flurry; Weave; Tantrum | Existing-instance movement; no kind sampling or constructor-state reset. |
| Wish | Three option displays, one callback; no acquired physical option. |

The actual run character remains **Ironclad** in downstream generation. A Watcher
producer's color is not a replacement character context. If Foreign Influence
offers a context-sensitive producer, that nested producer uses its own source
policy and actual Ironclad character, not the offer's color. Shard widens only its
specified acquisition route; it does not globally widen potions, relics, random
temporary cards, basics or healing-tagged specials.

Foreign Influence is an explicit any-color exception. Each proposal first rolls
0…99 with card-random RNG: 0…54 common, 55…84 uncommon, 85…99 rare. The type/rarity
library filter excludes HEALING, Curse and Status and locked kinds (all ordinary
unlocks are enabled here); it has **no color predicate**. It shuffles its candidate
group with card-random RNG, then the rarity-selection helper sorts candidates and
selects an index with card RNG. Duplicate card IDs retry the **whole** proposal,
including rarity. Therefore 55/30/15 describes proposal rarity, not necessarily
each accepted slot after duplicate rejection, and not a universal reward law.
Colorless Attacks of eligible rarity are not excluded merely for being colorless.
Special/BASIC rarity does not enter these three rarity proposals. Lesson Learned
and Wish have HEALING tags; tags, not English healing text, govern that exclusion.
Exact seeded shuffle/sort-tie history and accepted joint law remain W-RNG.

Creation actions/effects own hand size 10 overflow, destination, copied state and
Master Reality hook boundaries. A chosen Foreign Influence card uses a
stat-equivalent copy and goes to discard if hand is full. That does not prove
every other producer's overflow/hook order. Master Reality cannot be implemented
as one idealized upgrade at every constructor: offer, chosen-card copy, temporary
creation and effect paths are distinct. Existing shared route-specific extra-
upgrade/Searing Blow questions are not settled by these Watcher records.

### 7.2 X capture is separate from energy spent (S)

Keep queue capture `energyOnUse`, source-local effective count, free-one-play flag,
energy total at action execution, child upgrade and child hit count distinct.
The source stores X cost as −1; unplayable is −2, not zero.

For **Conjure Blade**, base forwards captured `energyOnUse`; upgraded forwards
that integer **plus 1**. The action uses the forwarded value unless it equals −1,
in which case it reads current energy total. Chemical X then adds 2. It constructs
one base Expunger, calls `setX(effective)`, and requests creation in draw pile even
when effective is zero. Unless the passed free-one-play flag is true, it separately
spends the **current total**, not the stored hit count. In particular a synthetic
upgraded sentinel −1 becomes 0 before the action's fallback check; do not assume
that hypothetical input behaves as “current total +1.” Reachable queue callers
must establish their actual capture rather than erase sentinel distinctions.

For **Collect**, effective count is captured energy if not −1, otherwise action-time
energy; Chemical X adds 2, upgraded parent adds 1. A positive result installs that
many turns of upgraded-Miracle generation. A zero/nonpositive result installs
nothing; payment is inside that positive-result branch. There, nonfree payment
separately consumes current total. A free play does not automatically mean X=0;
a captured X does not establish how much was paid.

Collect's installed duration is capped at 999 on construction and stacking. Each
energy-recharge callback requests one upgraded Miracle, then queues removal if
duration ≤1, otherwise a reduction by 1. Stacking lengthens this single duration;
it does not multiply the number created per recharge. This is an evidenced source
cap, unlike an invented limit on Expunger hits.

Expunger's fresh constructor sets cost 1 and damage 9, leaving current/base magic
count at the inherited **−1** until `setX` is called (a nonpositive loop count
requests no hits). `setX(a)` stores a as current/base hit count,
**adding 1 if the child was already upgraded at that moment**. Upgrading an already
captured child adds 6 damage only; it does not rerun `setX` or add a hit. Ordinary
`makeCopy` constructs a fresh unset-count Expunger; its stat-equivalent override
explicitly restores both stored count fields (and associated description state).
Thus constructor-copy, stat-equivalent copy, “upgrade then capture,” and “capture
then upgrade” are different operations. Conjure Blade constructs a base child and
captures before any later creation upgrade; that upgrade does not recapture X.
Expunger never re-reads energy when played. No guessed hit cap or inherent Exhaust
is added.

## 8. Numerical and conditional execution rules

### 8.1 Source domains and rounding (S)

Card base/current damage, Block, magic amounts, times-upgraded, X capture, energy,
power amounts and the combat Mantra history are Java signed **32-bit integer**
fields in the inspected source, not unbounded mathematical integers. Costs include
the −1/−2 sentinels. Damage/Block modifier intermediates are Java **binary32 float**;
stance multipliers are binary32 2.0 and 3.0. Preserve conversion/rounding boundaries,
not exact-rational accumulation followed by one convenient rounding.

The ordinary target-specific card damage calculation applies player relic damage
modification, player damage-give powers, player stance, target damage-receive
powers, player final-give powers, target final-receive powers; negative final
intermediate clamps to 0, then `MathUtils.floor` produces the integer. Ordinary
card Block applies player `modifyBlock`, then `modifyBlockLast`, clamps negative
result to 0 and floors once. Hook order within collections matters. This does not
complete every hook body, incoming-damage path or captured-copy override.

Halt is exceptional **before** that final gain: independently calculate modified
Block for nominal 3/4 and nominal 9/14, including their separate floors; the
action-time Wrath check chooses their sum versus the first alone, then requests
one Block gain. Dexterity/Frail can therefore modify both components despite there
being only one gain event. The old P draft's unchanged additional 9 was incomplete;
S upgrade/use/bytecode establish additional 14 for Halt+.

Spirit Shield excludes the playing object from hand count, multiplies integer
count by 3/4, then applies the ordinary Block-modifier path once at use. The
similarly named SpiritShieldAction is not the card's inspected use route. Retain
growth changes base integer fields; modifying cost is not changing X or stored h.

Do not promote source overflow to a native gameplay wrap, cap, defeat or retry.
Exact behavior at overflowing source operands, nonfinite float conversion or
administratively exhausted native numeric/allocation capacity is **W-NUM**; the
native architecture requires an administrative fault rather than dropped/truncated
legal content. This exception is not a claim that the source's only numeric limits
are administrative. The explicit Like Water stack clamp 999 remains a real
source rule, as is Collect's duration clamp; a blanket 999 cap on other Watcher
powers is unsupported.

### 8.2 Ongoing resources and non-Attack payloads (S)

**Deva Form:** carry increment `a` and next gain `g`, initially a=1,g=1. Recharge
grants g synchronously, then adds a to g. A further Deva stack increases a by the
stack amount and increases g by **1**, not by that full amount. Two ordinary
installations before recharge yield gains 2,4,6…; installing the second after one
recharge yields next gain 3, then 5…. These are state-conditional laws, not one
universal independent progression for every copy. Fasting Energy Down is a
separate start-turn loss request, so its relative resolution cannot be hidden by
folding it into the Deva increment.

**Talk to the Hand:** enemy Block Return is a stacking DEBUFF. Its on-attacked
hook qualifies damage info that is neither THORNS nor HP_LOSS, with nonnull owner
different from that enemy. It does not require positive damage in this hook;
qualifying zero/fully blocked attacks can still request Block. Each qualifying hit
requests stacked amount for the player. Complete on-attacked invocation and
lethal cancellation remain shared fields; do not generalize this to arbitrary HP
loss or Mark.

**Pressure Points:** Mark is a stacking DEBUFF. Its trigger queues HP_LOSS equal
to its stored amount with null owner when called by `PathToVictory`. Trigger Marks
traverses monster/power collections in their current order; it does not pick one
enemy. Artifact can prevent the new application while old marks elsewhere still
trigger. HP loss is not Block-reduced NORMAL attack damage, and no stance damage
multiplier is applied to its nominal Mark amount.

**Wallop:** the inspected action calls target damage, then reads positive
`lastDamageTaken` and queues a gain at the front before its post-combat clear.
That field is not universally “HP removed” or a guaranteed surviving gain. The
in-flight target damage may invoke retaliation/death hooks; current-action work and
pending-action filtering are distinct. No guaranteed final-enemy Wallop heal-like
outcome is inferred.

**Lesson Learned:** after direct target damage, the local fatal branch requires
target dying or HP≤0, excludes half-dead and Minion. Collect every permanent-deck
instance for which `canUpgrade()` is true; if nonempty select an index using misc
RNG, upgrade it and perform bottled-upgrade bookkeeping immediately, before later
combat cleanup. Duplicate kinds are distinct eligible instances; Searing Blow uses
its own repeated-upgrade predicate. An empty set draws nothing. This is not a
promise of synchronized mutation of all corresponding combat copies or of
arbitrary reaction-kill credit.

**Ragnarok:** each queued action chooses again through the explicit card-random
RNG overload. If monsters are basically dead, return no target without that index
draw. Otherwise the null-exception/alive-only path constructs candidates in group
order excluding `halfDead`, `isDying` and `isEscaping`; an empty candidate group
returns null without a draw. A nonempty group requests index 0…n−1, **including a
request at n=1**. The RNG wrapper increments its counter once per such request.
Then the action recalculates the card's damage against that chosen target and
queues the hit. This is not captured total damage or a preselected sequence of
targets. Exact root-state/conditional joint measure of those bounded draws remains
W-RNG; eligible-target exclusions and empty/singleton call boundaries are not
unknown merely because that measure remains open.

### 8.3 Blasphemy and Judgment are not administrative outcomes (S)

End Turn Death inherits BUFF classification and nonstacking amount −1. Artifact's
DEBUFF prevention does not block it; debuff cleansing is not permission to remove
it. At its start-turn callback it requests VFX, HP loss 99,999 with owner/source
player, then self-removal. HP loss enters actual damage/prevention/rescue machinery.

The inspected player damage path clamps a >1 amount to 1 if IntangiblePlayer is
present at damage time. Buffer's damage-change hook can reduce a positive amount
to zero while requesting consumption; Tungsten Rod's later HP-loss hook subtracts
1 from a positive amount. Thus surviving via prevention is not inherently illegal:
for example, Intangible still present plus Rod can reduce this nominal loss to 0.
With Intangible alone and no other changes, HP>1 survives a one-HP loss. Presence
and expiry **at this boundary**, not presence when Blasphemy was played, matters.

If the resulting player hit is lethal, the retained branch considers held Fairy
before unused Lizard Tail unless Mark of the Bloom prevents that rescue branch.
Rescue uses actual heal hooks; it is not rollback or proof of positive final HP
under all modifiers. Pending removal and other queued work are not guaranteed to
survive a death transition. Vault/start-turn/death arbitration and all other
prevention combinations remain W-END/W-TERMINAL. Future tests must not assert
“unpreventable death next action” or translate Blasphemy to reward −1 directly.

Judgment's threshold action requests Instant Kill, not an ordinary damage hit.
That retained action sets target HP to zero, updates the health bar, then invokes
target damage with a null-owner, zero-amount HP_LOSS damage info. Downstream
monster damage/death/half-dead/revival/victory callbacks therefore remain
authoritative; no claim that this action itself unconditionally calls `die` is
made. It does not request a 30/40-damage hit against target Block. Its threshold
is action-time HP, not a preview captured when the player selected the card.

## 9. Exact open fields

These remain **blockers**, not generic “all CARD-X” disclaimers and not optional
content cuts. Local records above can be used without inventing answers to them.
The [shared execution report](2026-10-18-act1-execution-closure.md) explains why a
local request does not close all downstream containers/callbacks.

| Field | Exact obligation still open |
|---|---|
| W-CALLBACK | Full transitive lifecycle/cost/legality/copy hook closure; previous-card history mutation between enqueue and predicate; mixed Vigor/Swivel/repeat capture; all return-to-hand overflow and duplicate notifications. A direct callback body is not a complete caller proof. |
| W-CHOICE | Source UI selected-list construction/order for multi-select Meditate and Scry; modal Scry update-pass count, repeated Weave return de-duplication; nested Omniscience choice/target order and cancellation. Preserve order as unresolved, not sorted by implementation convenience. |
| W-END | Whole caller ordering among Ethereal, Retain restore, Establishment, Pyramid/Equilibrium/Well-Laid Plans, delayed powers, recharge, opening draw, Divinity exit, Blasphemy and Vault. Local hook names/order above remain facts; universal end/start-turn arbitration is not proved. |
| W-TERMINAL | Exact continuation ownership when retaliation/player death, last-enemy death, revive, pending clears or modal interruption meet Wallop, Mark, Judgment, Lesson Learned, Blasphemy and repeated/turn-ending cards. No invented queue drain or completed-outcome guarantee. |
| W-RNG | Full source-conditioned shuffle/sort-tie law and accepted joint Foreign Influence offer law; bounded-draw root-state/conditional joint measure for Ragnarok (support and empty/singleton call sites are specified above); draw-pile insertion endpoints/joint ordering and RNG history for every creator and Tantrum. No assumed uniform length+1 gaps or uniform full permutation. |
| W-CREATE | Complete per-route Master Reality/copy/temporary-creation/effect hook matrix, including hand overflow ordering, offers versus chosen instances, already-upgraded/Searing Blow behavior, and unrelated nested producers' actual-character policy. The named fixed children are not a universal generator. |
| W-NUM | Complete source numeric-domain/overflow reachability and every downstream modifier/cap path, beyond the explicit int32/binary32/floor and local clamps established above; native administrative exhaustion contract must remain distinct from source mechanics. |

This supplies **finite paired primary coverage**, not full B07 transitive closure,
whole-Act1 approval, a selected timer policy or implementation-start permission.
No unpublished document or private import is required to read the numeric records;
private evidence supports review but does not substitute for a missing public rule.

## 10. Future acceptance cases — UNEXECUTED

No case below was run as gameplay, source execution, simulator or reference test.
“Probe” rows require the named open-field decision before fixing an outcome.

| Case | Discriminating expectation or required probe |
|---|---|
| W-A01 finite paired coverage | 4 basics +19 common +35 uncommon +17 rare +8 physical specials =83; three Wish options are additional nonphysical records. Every row has base/upgraded cost, target, flags and payload. |
| W-A02 identity | Watcher Note Strike/Defend retain `Strike_P`/`Defend_P`; acquired cards do not change Ironclad, starting resources or exclusive pools. |
| W-A03 transitions | Same Calm yields no exit energy; Calm→Neutral requests 2; Calm→Divinity requests 2+3; same Divinity grants no entry 3. Notification execution versus queued payload remains distinct. |
| W-A04 Mantra | Existing amount 9 stacked by 3 gives remainder 2 and one transition; hypothetical 9+23 gives 22, not modulo 2. Devotion amount ≥10 with no Mantra takes its direct branch without constructor history increment. |
| W-A05 Halt correction | No modifiers: base Wrath gain 12; upgraded Wrath gain 18. With Dexterity/Frail, independently modify/floor nominal components, then one gain. Never use old +9 for Halt+. |
| W-A06 Spirit Shield | Ordinary hand contains source plus three others: base Block input 9, upgraded 12, then modifiers once. No per-card Dexterity or use of unrelated action-time group-size helper. |
| W-A07 growth | Retain Windmill base twice, then upgrade: damage base 18, future growth 5. Perseverance corresponding Block 11, future growth 3. No permanent-deck edit or retroactive growth. |
| W-A08 retain full-hand | Compare Meditate+'s undersized automatic branch with selected branch at one free slot; overflow Retain flags differ. Exact selected-list order remains W-CHOICE, not an arbitrary set. |
| W-A09 empty Scry | No basically-dead enemies, empty draw: Nirvana notification is requested; no ordinary Scry reshuffle. Foresight supplies its own empty-deck shuffle. Probe modal repeated Weave notifications separately. |
| W-A10 creation variants | Evaluate+/Pray+/Study+ produce base Insight; Collect produces Miracle+; Deus+ requests three base Miracles only on draw. Test route-specific Master Reality upgrades separately. |
| W-A11 Conjure capture | Captured X=3 with Chemical X: base h=5, upgraded h=6; nonfree action payment uses current total separately. At effective h=0 still request one Expunger. No cap or extra count from later upgrade. |
| W-A12 Expunger copies | Set base child's h=3 then upgrade: damage15,h3. Upgrade first then setX(3): damage15,h4. Stat-equivalent copy preserves captured h; constructor copy does not imply that preservation. |
| W-A13 Omniscience | Chosen original marked Exhaust; second request is a stat-equivalent purge copy constructed before resolution. Children do not pay remaining energy normally. Probe nested choice and turn-ending interruption without inventing full drain. |
| W-A14 Deva stacks | Two initial installations yield 2,4,6… recharge gains; installing second after first recharge yields next 3 then5. Fasting loss remains a separate timed request. |
| W-A15 Blasphemy | BUFF installation is not Artifact-prevented. Damage-time Intangible/Buffer/Rod and Fairy-before-Tail rescue must be honored conditionally. Vault/expiry/terminal outcomes remain probes, not direct defeat assertions. |
| W-A16 Mark/Judgment | Artifact-prevented new Mark leaves old Mark triggers; Judgment compares action-time HP then requests kill, not damage. Resolve downstream death arbitration without skipping it. |
| W-A17 hit structure | Bowling Bash counts non-dead/non-escaped monsters at use and enqueues that many target hits; Ragnarok+ enqueues six distinct random-attack actions, not one 36-damage hit. Exact per-hit RNG law remains W-RNG. |
| W-A18 current-character policy | Foreign Influence's explicit any-color exception does not turn nested context-bound generation into Watcher generation. Track duplicate-retry rarity proposals and both RNG roles, not accepted IID 55/30/15. |
| W-A19 Wish order | Strength, gold, Plated Armor option order and + amounts 4/30/8; choose exactly one callback, no acquired option, no payment of −2 or ordinary Power-use history. |
| W-A20 fault/privacy boundary | Invalid choice advances no authoritative state/RNG; administrative numeric/allocation failure does not become death/truncation. Observations expose legal choices/captured count and known state, not hidden draw order, seed or internal identity. |

## 11. Validation and review boundary

Actually executed: private failure-first guard tests with missing/duplicate ZIP,
path-escape, hash/size and Java-option negative controls; four pinned inputs checked
before/after static batches; exact unique class selection; retained source/member
reconciliation; finite row/paired-record checks; local public-link, whitespace and
single-path publication checks. A rejected wrong-package inspection was preserved
and corrected; it was an evidence-tool failure, not a gameplay result.

Application tests were not rerun for this prose-only author lane. The controller's
current-wave baseline is 124 Rust pass, 828 Python pass and two CUDA-environment
failures; the user's documentation-only exception is not a green suite, a waiver
of new failures, or permission for product/test/dependency changes. Independent
review and exact-head main readback remain pending; the Bead stays in progress.
