# Act 1 card records: Ironclad, colorless, Status, Curse and special dependencies

Date: 2026-10-19. Issue: `spire-gli.2.3.18`. Base: `cf5f410725d45794baea61e9d85e04c797e089ad`.

**Specification draft; independent review pending.** This document supplies actual
paired primary records, not an inventory substituted for mechanics. It specifies
75 red, 35 ordinary colorless, five Status, eleven Curse, four stored-event and
twelve dependent-special/control records. Silent/Defect/Watcher ordinary and basic
records remain their respective content lanes' responsibility. Local payload
coverage is not full transitive callback, terminal or whole-spec closure.
No implementation, whole-spec approval or original-runtime validation is claimed.

## 1. Authority, evidence and reading the records

Scope is standard, unmodded, fully unlocked, Act-4-enabled Ironclad A0 Act 1,
including legal Shard and Note content. Off-color possession does not change the
actual character, grant another start, or import later-act home events.

Evidence is pinned to owned archive SHA-256
`cfad868ac8d65a88e71a0bf096fb09f78811e553effe0787c5309a655e081673`.
This identifies bytes, not an independently established upstream build/channel.
The independently authored records combine previously reviewed paired-description
research, retained static constructor/generator/execution/acquisition evidence, and
targeted new static inspection of missing local card/action/power members. Nothing
below is copied source, a mechanical translation, or an executed gameplay result.

Evidence families, with non-sensitive class/member locators:

| Key | Evidence and boundary |
|---|---|
| P | Reviewed primary paired-description research; [Ironclad compendium](https://slaythespire.wiki.gg/wiki/Cards_List), [colorless descriptions](https://slaythespire.wiki.gg/wiki/Colorless_Cards), and [StS1 event/special descriptions](https://slayspiredb.com/sts1/cards/ritual-dagger). Paired constants are not scheduler proof. |
| S | Pinned static card constructors and selected `use`, `upgrade`, calculation and lifecycle methods; relevant named action/power members below. Static source reconstruction, not runtime observations. |
| G | Reviewed retained selectors, copy helpers, temporary-card actions/effects, `CardGroup.addToRandomSpot`, and source-specific Master Reality checkpoints. |
| A | Reviewed retained Note, object removal, transform and obtain paths; [merged continuation findings](2026-10-18-act1-continuations-closure.md) and [profile/time findings and proposals](2026-10-19-act1-profile-time.md). |
| X | [Merged bounded execution findings](2026-10-18-act1-execution-closure.md); separate current correction evidence for conditional free flags and instance-local loss notifications. The latter is adopted only for the named checked facts, not approval of its author's unreviewed full document. |

Private `content-red-colorless/claim-ledger.md` maps every row to retained
constructors/paired evidence and the exceptional members; the two source manifests
record 84 targeted dependencies (21 reused full sources, 63 newly reconstructed
classes with disassembly), plus a separately reused library-floor dependency.
These are evidence counts, not 84 independently proven
transitive mechanics. Four archive/tool identities were checked before/after;
JVM option injection variables were removed. No game class, initializer, launcher,
substituted original runtime, real profile or save ran or was accessed.

### Record conventions

* `a/b` is base/upgraded; a single value is unchanged. Cost is printed energy,
  before modifications. `X` and unplayable (`U`, source sentinel −2) are not zero.
* `A/S/P` = Attack/Skill/Power. `E` = selected enemy, `All` = all eligible enemies,
  `Rnd` = automatic random target, `Self` = no enemy choice. `None` is no target;
  these semantic targets do not turn source `NONE` into `SELF` metadata. Spot
  Weakness's source target is `SELF_AND_ENEMY`: the player chooses the enemy.
* `D(n)` = ordinary Attack damage, `B(n)` = calculated card Block, `T(n)` =
  non-Attack/THORNS damage, `L(n)` = explicit player HP loss. Neither T nor L is
  automatically modified like D. `draw` means actual draw, not creation/retrieval.
* `ex`, `eth`, `inn`, `ret` = exhaust-on-use, Ethereal, Innate, intrinsic Retain.
  Unlisted keywords are absent. Ordinary non-Power A/S cards discard after use
  unless origin/modifiers change that destination. Power removal is not Exhaust.
  Status/Curse special paths are described separately.
* Red section headings give rarity; ordinary colorless is COLORLESS, Status is
  COLORLESS/COMMON, curses are CURSE color. A row's changed field is its upgrade
  delta; unchanged fields persist. Except repeat-upgrade Searing Blow and supplied
  Burn+, ordinary cards have one meaningful upgrade; curses/other statuses have none.
* Effect ordering below is local request/capture order. Intervening callbacks,
  revalidation and terminal cancellation are not silently replaced with universal
  FIFO, recursive drain, rollback or guaranteed completion. Exact remaining common
  rules have anchors in [§8](#8-exact-unclosed-common-rule-anchors).

## 2. Ironclad paired records

### Basics — three

| ID | Kind (class if different) | Type/target | Cost | Base/upgraded payload and local condition |
|---|---|---|---|---|
| R01 | Bash | A/E | 2 | D(8/10), then Vulnerable 2/3. |
| R02 | Defend (`Defend_Red`) | S/Self | 1 | B(5/8). |
| R03 | Strike (`Strike_Red`) | A/E | 1 | D(6/9); STRIKE tag. |

### Common — twenty

| ID | Kind | Type/target | Cost | Base/upgraded payload and local condition |
|---|---|---|---|---|
| R04 | Anger | A/E | 0 | D(6/8); request one stat-equivalent combat copy in discard, not a fresh printed-state Anger or permanent acquisition. |
| R05 | Armaments | S/Self | 1 | B(5); upgrade one eligible remaining hand card / every eligible remaining hand card for combat. Zero eligible: no upgrade; single eligible: forced. |
| R06 | Body Slam | A/E | 1/0 | Set base damage to current Block in `use`, calculate target damage, request hit; Block is not consumed. Same-play later Fan/Rage gains are not advance credit. |
| R07 | Clash | A/E | 0 | Ordinary `canUse` requires every hand card be Attack; D(14/18). Inert/unplayable non-Attacks still fail that predicate. |
| R08 | Cleave | A/All | 1 | D(8/11) per enemy, per-target damage calculation. |
| R09 | Clothesline | A/E | 2 | D(12/14), Weak 2/3. |
| R10 | Flex | S/Self | 0 | Strength +2/+4; separately apply matching end-turn Strength-loss debuff. Artifact can prevent that debuff; it does not reverse the gain. |
| R11 | Havoc | S/None | 1/0 | Request draw-top autoplay, no child payment, exhaust destination; empty draw can reshuffle discard. Nested target/legality/X/destination are origin-specific, §5. |
| R12 | Headbutt | A/E | 1 | D(9/12); move one discard instance to draw-top; empty→none, singleton→forced, multiple→choose. Retrieval action returns without movement if room is battle-ending. |
| R13 | Heavy Blade | A/E | 2 | D(14) with signed Strength contribution ×3/×5; not multiplication of the entire attack or all later modifiers. |
| R14 | Perfected Strike | A/E | 2 | Base 6 plus 2/3 per STRIKE-tagged instance currently in hand+draw+discard at calculation. Not master deck, exhaust or limbo; §4. |
| R15 | Pommel Strike | A/E | 1 | D(9/10); draw 1/2. |
| R16 | Shrug It Off | S/Self | 1 | B(8/11); draw 1. |
| R17 | Sword Boomerang | A/Rnd (source All) | 1 | 3/4 separate sequential hits of D(3); reselect eligible random living target per hit, not one target for all hits. Repeated target identities are allowed. |
| R18 | Thunderclap (`ThunderClap`) | A/All | 1 | D(4/7) to enemies; then Vulnerable 1 to enemies, not applied before its own hits. |
| R19 | True Grit | S/Self | 1 | B(7/9); exhaust one random remaining hand instance / one selected remaining instance; empty hand gives no exhaust. |
| R20 | Twin Strike | A/E | 1 | Two separate hits D(5/7). |
| R21 | Warcry | S/Self | 0 | Draw 1/2; select one resulting hand card to draw-top; ex. |
| R22 | Wild Strike | A/E | 1 | D(12/17); create one base Wound at a random draw-pile insertion position; §6 gives endpoints. |
| R24 | Iron Wave | A/E | 1 | B(5/7), then D(5/7). Calculate ordinary card Block once, not twice. |

### Uncommon — thirty-six

| ID | Kind | Type/target | Cost | Base/upgraded payload and local condition |
|---|---|---|---|---|
| R25 | Battle Trance | S/None | 0 | Draw 3/4, then apply current-turn No Draw; Artifact can block the debuff. Existing No Draw still governs the requested draws. |
| R26 | Blood for Blood | A/E | 4/3 | D(18/22); each positive notified HP-loss event reduces mutable combat cost by one, minimum 0. Fresh-kind copy accounts for `damagedThisCombat`; upgrade/copy exceptions in §4. |
| R27 | Bloodletting | S/Self | 0 | L(3), then energy +2/+3. Prevented loss is not an invented loss event; later energy is not a survival guarantee. |
| R28 | Burning Pact | S/None | 1 | Select one remaining hand instance to exhaust, then independently request draw2/3. Empty hand exhausts none but does not cancel the draw; singleton is forced. |
| R29 | Carnage | A/E | 2 | D(20/28); eth. |
| R30 | Combust | P/Self | 1 | Install end-turn L(1) then all-enemy T(5/7). Stacking adds damage and one to the separate self-loss amount; §5. |
| R31 | Dark Embrace | P/Self | 2/1 | Each actual exhaustion requests draw 1 per installed intensity; stacks. |
| R32 | Disarm | S/E | 1 | Strength −2/−3; ex. Debuff prevention applies to the negative Strength application. |
| R33 | Dropkick | A/E | 1 | D(5/8); action checks target Vulnerable before inserting its hit, energy+1, draw1 at queue front in reverse insertion order: hit then energy then draw if condition passed. |
| R34 | Dual Wield | S/None | 1 | Select remaining hand Attack or Power; retain equivalent original plus 1/2 added combat copies. Choice branch replaces selected hand object with equivalent copies; §4. |
| R35 | Entrench | S/Self | 2/1 | Gain current Block again through direct doubling path; no Dexterity/Frail/No Block recalculation of B(current Block). Entrench itself is not Ethereal; its separate end-player-turn card callback queues the Ethereal cleanup described in §5. |
| R36 | Evolve | P/Self | 1 | Each actual Status draw requests draw 1/2 per accumulated intensity only when its owner lacks No Draw at the callback; no Curse/movement/creation trigger. |
| R37 | Feel No Pain | P/Self | 1 | Each actual exhaustion requests raw power Block 3/4 per accumulated amount. Not B(3/4). |
| R38 | Fire Breathing | P/Self | 1 | Each actual Status/Curse draw requests all-enemy T(6/10); stacks. |
| R39 | Flame Barrier | S/Self | 2 | B(12/16); retaliation T(4/6) when the hit callback has a nonnull owner other than this power's owner and damage type is neither THORNS nor HP_LOSS; target that hit owner. At the player's next start turn it queues power removal; not current-turn-end removal. |
| R40 | Ghostly Armor | S/Self | 1 | B(10/13); eth. |
| R41 | Hemokinesis | A/E | 1 | Capture D(15/20), request L(2) before hit. Queued Rupture Strength does not recalculate this captured hit. |
| R42 | Infernal Blade | S/None | 1/0 | Generate one eligible actual-character Attack with ordinary turn cost 0; ex. X remains X. Exact 28-kind support and Master Reality routes in §6. |
| R43 | Inflame | P/Self | 1 | Strength +2/+3. |
| R44 | Intimidate | S/All | 0 | Weak 1/2 per enemy; ex. |
| R45 | Metallicize | P/Self | 1 | Pre-end-turn-card power hook requests raw Block 3/4; stacks, not ordinary card Block. |
| R46 | Power Through | S/Self | 1 | Request two base Wounds into hand, then B(15/20); full-hand excess goes to discard. |
| R47 | Pummel | A/E | 1 | 4/5 separate D(2) hits; ex. |
| R48 | Rage | S/Self | 0 | Install current-turn Attack-use trigger granting raw power Block 3/5; stacks; removed at end turn. |
| R49 | Rampage | A/E | 1 | D(8 + prior combat growth); then +5/+8 base damage to battle instances sharing its UUID, lower-clamped at 0. No master-deck mutation. |
| R50 | Reckless Charge | A/E | 0 | D(7/10); insert one base Dazed randomly in draw pile. |
| R51 | Rupture | P/Self | 1 | Positive HP-loss notification whose damage owner is player requests Strength +1/+2; one trigger per event, not per HP; §5. |
| R52 | Searing Blow | A/E | 2 | Base 12; after n actual upgrades, 12+n(n+7)/2 (first upgrade 16). Repeat-upgrade/copy constructor distinction in §4. |
| R53 | Second Wind | S/Self | 1 | Snapshot remaining non-Attacks; exhaust that set before its separate B(5/7) events, one per snapshotted card; no inherent self-exhaust. |
| R54 | Seeing Red | S/None | 1/0 | Energy +2; ex. |
| R55 | Sentinel | S/Self | 1 | B(5/8); this instance's actual Exhaust callback requests energy +2/+3, whether played or manually exhausted. |
| R56 | Sever Soul | A/E | 2 | Queue remaining-hand non-Attack exhaustion before D(16/22). Individual exhausts are pushed to front; not the reference discrepancy that defers them behind hit. |
| R57 | Shockwave | S/All | 2 | Weak 3/5 and Vulnerable 3/5 as separate debuff applications per enemy; ex. |
| R58 | Spot Weakness | S/E | 1 | At action resolution, nonnull target with intent base damage ≥0 gives Strength +3/+4. No attacking-intent legality gate; hidden intent cannot filter candidates. |
| R59 | Uppercut | A/E | 2 | D(13), Weak 1/2, Vulnerable 1/2. |
| R60 | Whirlwind | A/All | X | Effective X rounds of per-enemy D(5/8); Chemical X adds2; captured energy and payment are distinct, §5. |

### Rare — sixteen

| ID | Kind | Type/target | Cost | Base/upgraded payload and local condition |
|---|---|---|---|---|
| R61 | Barricade | P/Self | 3/2 | Prevent ordinary turn-start Block clearing; non-stacking presence, not a second multiplication of Block. |
| R62 | Berserk | P/Self | 0 | Self Vulnerable 2/1; install start-turn energy +1 per intensity. |
| R63 | Bludgeon | A/E | 3 | D(32/42). |
| R64 | Brutality | P/Self | 0 | Install post-normal-draw hook: draw1, then L(1), each per accumulated intensity; + adds inn. Does not lose HP before its queued draw. |
| R65 | Corruption | P/Self | 3/2 | Non-stacking Skill cost/free-play treatment plus Skill exhaust-on-use. Status and Curse remain their own types; do not assign all negative-cost cards numeric0. |
| R66 | Demon Form | P/None | 3 | Post-normal-draw hook requests Strength +2/+3; stacks. |
| R67 | Double Tap | S/Self | 1 | Next 1/2 qualifying Attacks this turn each gain one extra play. Charges count affected Attacks, not repeats of one Attack; §5. |
| R68 | Exhume | S/None | 1/0 | At action resolution, hand size10, empty exhaust, or no non-Exhume exhaust candidate yields no transfer; one eligible candidate is forced, otherwise choose one. Move that existing instance to hand, excluding every Exhume kind; ex. No reconstruction or missed-history catch-up. |
| R69 | Feed | A/E | 1 | D(10/12); qualifying Fatal immediately raises max HP by3/4, then requests an ordinary heal of that amount through relic/power heal hooks and current-HP cap; current-HP gain is not invariantly3/4. ex. No half-dead/Minion benefit; splitting alone is not Fatal. |
| R70 | Fiend Fire | A/E | 2 | `FiendFireAction.update` captures remaining hand size (not `use`); random one-card Exhaust actions consume that original shrinking hand before queued reactions refill it; one D(7/10) hit per captured count; ex. |
| R71 | Immolate | A/All | 2 | D(21/28), then create one base Burn in discard. |
| R72 | Impervious | S/Self | 2 | B(30/40); ex. |
| R73 | Juggernaut | P/Self | 2 | Each positive `onGainedBlock(float)` notification requests random-enemy T(5/7); stacks. Zero notification does not trigger. |
| R74 | Limit Break | S/Self | 1 | Add current signed Strength again, including negative; base ex, + removes ex. Does not double pending Flex expiry amount. |
| R75 | Offering | S/Self | 0 | L(6), energy+2, draw3/5; ex. |
| R76 | Reaper | A/All | 2 | D(4/5); later heal nominal sum of actual enemy HP removed, excluding Block/overkill; ex. Retaliation can precede heal; final outcome/timer remains CARD-RC-TERMINAL. |

Record identifiers are opaque (R23 is unused): the 75 identities are R01–R22 and
R24–R76. No roster count is evidence of composed closure.

## 3. Ordinary colorless paired records

### Uncommon — twenty

| ID | Kind | Type/target | Cost | Base/upgraded payload and local condition |
|---|---|---|---|---|
| C01 | Bandage Up | S/Self | 0 | Heal4/6; ex. Ordinary acquisition includes it; L34 combat generators exclude its HEALING tag. |
| C02 | Blind | S/E→All | 0 | Weak2; + changes target from one chosen enemy to every enemy, not duration. |
| C03 | Dark Shackles | S/E | 0 | Strength−9/−15; ex. Queue matching end-turn restoration only if nonnull target lacks Artifact at `use`; not after observing whether loss succeeded. |
| C04 | Deep Breath | S/Self | 0 | If discard nonempty, move/shuffle it into draw and explicitly shuffle combined draw pile; then draw1/2. Empty discard skips both shuffle requests. |
| C05 | Discovery | S/Self | 1 | Offer three distinct I70 kinds; mandatory choose one, turn-cost0, hand overflow→discard. Base ex; + removes ex. |
| C06 | Dramatic Entrance | A/All | 0 | D(8/12); inn, ex. |
| C07 | Enlightenment | S/Self | 0 | Current hand turn costs>1 become1; + independently changes current hand combat costs>1 to1. Each predicate is separate; not an aura or printed-cost predicate. |
| C08 | Finesse | S/Self | 0 | B(2/4), draw1. |
| C09 | Flash of Steel | A/E | 0 | D(3/6), draw1. |
| C10 | Forethought | S/None | 0 | Move one / zero-or-more selected hand instances to draw-bottom in selected order; set one-play-free only where mutable combat cost>0. No numeric cost assignment; §4. |
| C11 | Good Instincts | S/Self | 0 | B(6/9). |
| C12 | Impatience | S/None | 0 | Play is permitted with Attacks; effect-resolution hand scan draws2/3 only if no Attack then present. Glow is not legality. |
| C13 | Jack of All Trades | S/None | 0 | Generate1/2 independently sampled L34 cards, ordinary costs, into hand; ex. |
| C14 | Madness | S/Self | 1/0 | If any hand turn cost>0, select among those instances; otherwise select among combat cost>0 (including temporary0). Set chosen combat/turn cost0; ex. |
| C15 | Mind Blast | A/E | 2/1 | inn; `applyPowers` sets base damage to draw-pile size; later target calculation uses that field; `use` consumes captured damage, not a new draw-count query. |
| C16 | Panacea | S/Self | 0 | Artifact1/2; ex. Not removal of existing debuffs. |
| C17 | Panic Button | S/Self | 0 | B(30/40), then No Block2; ex. No Block's last card-Block modifier returns0; does not suppress raw power/orb/relic Block or direct Entrench doubling. |
| C18 | Purity | S/None | 0 | Select zero through3/5 other hand cards to exhaust; ex. Own cleanup exhaust is separate from selected-card exhausts. |
| C19 | Swift Strike | A/E | 0 | D(7/10); STRIKE tag, not a red basic. |
| C20 | Trip | S/E→All | 0 | Vulnerable2; + changes target, not duration. |

### Rare — fifteen

| ID | Kind | Type/target | Cost | Base/upgraded payload and local condition |
|---|---|---|---|---|
| C21 | Apotheosis | S/None | 2/1 | Combat-upgrade each canUpgrade instance in hand, draw, discard, exhaust, once in that group order; ex. Not playing/limbo instance, permanent deck or future-created aura. |
| C22 | Chrysalis | S/None | 2 | Sample3/5 S28 Skills before insertion; positive combat/turn costs set0; random draw-pile insertions; ex. + changes count, not child upgrades. |
| C23 | Hand of Greed | A/E | 2 | D(20/25); qualifying Fatal immediately grants20/25 gold, subject to gain prevention. No ex; half-dead/Minion exclusion as in GreedAction. |
| C24 | Magnetism | P/Self | 2/1 | Start-turn pre-normal-draw hook independently generates one L34 card per intensity, ordinary cost; dead-monster guard. |
| C25 | Master of Strategy | S/None | 0 | Draw3/4; ex. |
| C26 | Mayhem | P/Self | 2/1 | Pre-normal-draw `atStartOfTurn` callback queues one deferred wrapper per intensity; each wrapper later queues a draw-top autoplay, no payment and no Havoc-imposed exhaust. This is not a post-normal-draw callback; full queue completion relative to normal draw is not asserted. |
| C27 | Metamorphosis | S/None | 2 | Sample3/5 A28 Attacks before insertion; positive combat/turn costs set0; random draw-pile insertions; ex. |
| C28 | Panache | P/Self | 0 | Install remaining-play counter5 and T(10/14) payload. Each later use decrements; at0 reset5 and request all-enemy damage. Stack adds damage, not a new counter/reset; §5. |
| C29 | Sadistic Nature | P/Self | 0 | Matching debuff application notification requests target T(5/7); source must be owner, target another creature without Artifact; excludes restoration ID `Shackled`; §5. |
| C30 | Secret Technique | S/None | 0 | `canUse` requires a draw-pile Skill. Retrieval-time zero→none, one→forced, multiple→choose one; full hand redirects selected card to discard, otherwise hand. Not draw. Base ex; + removes ex. |
| C31 | Secret Weapon | S/None | 0 | Same legality/retrieval/overflow branches, Attack filter. Base ex; + removes ex. |
| C32 | The Bomb | S/Self | 2 | Install independent three-end-turn countdown with all-enemy T(40/50). Not a Power card, no ex. First eligible end turn changes3→2; then2→1; at1 queue reduction and detonation. |
| C33 | Thinking Ahead | S/None | 0 | Request draw2; queue later hand-to-top selection only if hand nonempty at `use`. Ordinary use includes its playing hand instance at that check; non-hand autoplay need not. Base ex; + removes ex. |
| C34 | Transmutation | S/Self | X | Positive effective X independent L34 outputs; + explicitly upgrades each child, then applies ordinary turn-cost0 setter; ex. Self-generated X child remains X and not automatically free. §§5–6. |
| C35 | Violence | S/None | 0 | Randomly transfer up to3/4 Attack instances from draw, without replacement; full hand redirects each selected card to discard. No actual draw; ex. Sampling qualification §6. |

## 4. Identity, costs, copying, upgrades and growth

### 4.1 Distinct operations and source fields

An ordinary fresh-kind copy uses that kind's construction law. A stat-equivalent
copy additionally retains source base damage/Block/magic, mutable combat and turn
costs, upgrade count/state, target, misc, bottle/cost flags and one-play-free flag;
it is not an alias or automatic permanent-source relation. Same-instance copy also
carries UUID. Preserve the creation route: repeating a card is not adding another
cycling card, and moving/retrieving an object is not constructing it again.

Anger requests a stat-equivalent discard copy. Dual Wield uses stat-equivalent
copies but has distinct identity branches: with one eligible Attack/Power it leaves
that object and adds1/2; with a genuine selection it removes the selected object
into selection state and adds2/3 equivalent objects, preserving the intended net
+1/+2 count, not the exact selected object's continued hand identity. Its noneligible
cards return afterward. Neither route changes the master deck. Rampage growth
targets all battle instances with its UUID, including card-in-use and cycling,
exhaust and limbo groups, never master-deck cards merely sharing kind/name.

Ordinary `upgradeBaseCost` preserves turn-minus-combat difference when turn cost
is positive, leaves an already-zero turn cost zero, then clamps a negative turn
cost to0. Blood for Blood specifically reduces the current combat cost by1 when
below4 (then lower-clamps), otherwise sets3; damage rises18→22. Its `tookDamage`
reduces existing notified instances, while `makeCopy` applies the player's earlier
`damagedThisCombat` history. This is not the Masterful Stab rule: Stab's fresh
constructor does **not** reconstruct missed loss history; exhaust-pile absence can
cause missed notifications, and Exhume returns the same object without catch-up.

Forethought's predicate reads **mutable combat cost**, not printed or payable
cost. Positive combat cost/turn-cost0 still receives one-play-free. Combat-cost0
with positive turn cost does not newly receive it; X/unplayable do not receive it.
Movement/reset restores turn fields but does not itself clear this flag. Ordinary
use cleanup clears it after after-use callbacks and before purge handling; other
writers may clear it earlier. Selected A then B produces A before B among those
bottom cards when eventually drawn. Ordered/empty upgraded selection is real;
canonicalizing it to an unordered subset changes outcomes.

Madness uses a two-stage eligible-instance predicate, not printed cost or payable
cost after free flags. Its preferred positive-turn-cost set can include combat-cost0
cards temporarily raised above0. With no positive turn costs it instead uses positive
combat costs, so temporary0 can qualify. Enlightenment changes turn cost>1; its +
independently changes combat cost>1. Combat0/turn3 becomes combat0/turn1; combat3/turn0
becomes combat1/turn0 under +. Confusion/Snecko can later
replace eligible ordinary costs; generated/moved entry is not itself an actual draw.
Cross-source precedence beyond these named bodies remains CARD-RC-COST.

### 4.2 Repeated upgrade and permanent misc

Searing Blow's actual upgrade adds4+old upgrade count, then increments count.
Thus n=0..4 gives12,16,21,27,34. Its count-taking fresh constructor starts damage12
while assigning the supplied count; that constructor alone does **not** replay the
earlier damage gains. Stat-equivalent copying restores the source's base damage
after its intermediate upgrades. Note reconstruction instead fresh-constructs and
replays stored upgrades. Do not replace these operations with an unconditional
“damage is a function of any supplied count” invariant for every intermediate copy.
Combat upgrades do not rewrite the permanent source's count. Master Reality's
route-specific additional upgrades are in §6, not a universal +1 rule.

Ritual Dagger begins misc/base damage15; upgrade changes Fatal growth3→5, not
initial15. A qualifying Dagger hit updates misc on matching-UUID permanent and
battle instances at resolution, then refreshes base damage from misc. A temporary
copy with a new UUID does not grow a same-named permanent Dagger. Same-instance
repeat provenance must be retained. Growth is not target-modified displayed damage,
Strength, upgrade count or an end-of-victory reward. Note stores ID/count, not
earlier misc: a retrieved upgraded Dagger starts15, future growth5. Remove outgoing
cards with their live fields intact before omitting misc from the stored descriptor.

Perfected Strike counts the STRIKE tag in hand/draw/discard at each recalculation.
The tags cover four basic Strikes, Perfected/Pommel/Twin/Wild/Swift/Sneaky/Thunder/
Meteor/Windmill Strike, not arbitrary localized names. Its own ordinary hand
calculation normally includes itself; a limbo repeat does not magically count as
being in those piles. Capture/recalculation sites and later pile changes must not
be replaced with one permanent-deck count; CARD-RC-CAPTURE retains composition gaps.

## 5. Hook, play-origin and conditional resolution rules

| Family | Established local contract; exact non-generalization |
|---|---|
| Exhaust | Actual movement notifies relics, powers, then exhausted card, before pile move. Branch samples in its earlier relic callback; Embrace queues later draw; Sentinel queues own energy. Spoon's coin belongs to exhaust-on-use non-Power cleanup, not manual/Ethereal exhaust or purge copies. |
| Multi-exhaust | Fiend Fire captures count in `FiendFireAction.update` (after its card-use method queued that action), then front-schedules random remaining-hand one-card exhaustion, completing the original shrinking set before bottom-queued draw/generation reactions. Second Wind snapshots non-Attacks, front-schedules exhausts then separate Block events; Sever Soul front-schedules eligible exhausts ahead of hit. No universal left-to-right exhaustion order follows. |
| Block | Direct B calculations pass power modifyBlock then modifyBlockLast and lower-clamp/source-floor conversion (§8). FNP/Rage/Metallicize use raw power Block; Entrench doubles existing Block directly. Juggernaut responds to positive gain notifications, not merely playing a Block-looking card. |
| Temporary Strength | Flex's gain and expiry debuff are separate applications. Dark Shackles tests existing Artifact in `use` before queued applications; its restoration debuff ID is `Shackled`, capped−999..999 in its own constructor/stack path. Do not infer application-time restoration eligibility from later Artifact consumption. |
| Combust/Rupture | Combust stacks damage and one self-loss unit per stack call, then requests one grouped loss before all-enemy damage if monsters not basically dead. One base+one upgraded gives loss2/damage12. Rupture's actual hook requires positive loss and owner==player; its Strength application is front-queued. Printed “card-caused” is not a complete causal ancestry test. |
| Start/end hooks | Berserk/Magnetism are pre-normal-draw. Brutality/Demon Form are post-normal-draw. Mayhem's `atStartOfTurn` callback runs pre-normal-draw and queues deferred wrappers which later queue its top-card play; this does not assert when all nested work completes relative to normal draw. Metallicize is pre-end-turn-card. During the separate end-player-turn card-callback pass, Entrench's override front-queues `ExhaustAllEtherealAction`; Entrench itself is not Ethereal. This named hook schedule is not a claim that all queued children commit before the next hook is invoked. |
| Panache | New installation starts counter5 and does not count its installing use retrospectively. Existing Panache counts later qualifying uses, including another Panache's use; stacking adds10/14 damage without resetting remaining count. Start-turn resets5. Damage is non-Attack and queued, not an extra Attack play. |
| Bomb | Each cast has separate identity, countdown3 and captured40/50 payload. Eligible end-turn hook queues reduction; amount1 additionally queues detonation. Differently aged bombs do not merge or borrow newest payload. Monsters-basically-dead suppresses that hook's work. |
| Sadistic Nature | `onApplyPower` tests DEBUFF, excludes `Shackled`, requires owner as source, nonowner target, no target Artifact at notification. Compound Weak+Vulnerable are separate notifications. Exact notification placement versus immunity/stacking/Artifact removal is CARD-RC-DEBUFF, not a blanket promise to trigger for every attempted debuff. |
| Impatience/Dropkick/Spot | Each condition is evaluated in its action, not by revealing success in legal actions. Impatience checks current hand types; Dropkick checks Vulnerable before its queued hit; Spot tests intent base damage≥0, including0 and mixed Attack intents. Hidden intent is not a policy-visible success preview. |
| Return-to-draw choices | Warcry always queues hand-to-top after draw; Thinking Ahead gates queuing at its own use-time hand nonempty check. When that action resolves: empty→no move, singleton→forced move, larger hand→mandatory one-card choice. Secret searches separately enforce a matching kind in draw at `canUse`, then rebuild candidate instances at retrieval; no eligible→no move, singleton→forced, larger→mandatory selection; full-hand target goes to discard. None of these movements is actual draw. |
| Havoc/Mayhem | Both nested plays waive payment but retain source-specific legality/targeting. Havoc imposes exhaust, Mayhem does not. Ordinary unplayable Status/Curse can finish in discard on autoplay without end-turn payload; Medical Kit/Blue Candle change play/cleanup, not type. Ethereal is not automatic exhaust merely on autoplay. |
| Double Tap/repeats | Double Tap, Burst, Amplify, Duplication, Echo carry first `energyOnUse` and explicitly queue autoplay=true/ignore-total=true. Repeat pays no new energy and is purge-on-use, not another persistent cycling card. Reactions and state changes between plays matter; mixed sources/dead targets/cancellation remain CARD-RC-ORIGIN. |
| Whirlwind/Transmutation | Effective count uses captured energy unless sentinel−1 selects current total, Chemical X adds2; positive count requests effects. Free play avoids spending energy; otherwise positive-effect action spends current total, not a fixed one-energy cost. Transmutation `use` first raises captured value to at least current total. Child count and payment must not share a generic fixed-cost0 flag. |
| Fatal/heal | Feed/Greed/Dagger check hit-attributed dying-or-nonpositive-HP target, exclude half-dead/Minion. Split flag alone is not Fatal; later eligible offspring kills can qualify. Feed's qualifying action raises max HP, then `increaseMaxHp` routes the same nominal amount through ordinary healing hooks; modifiers/prevention and the max-HP ceiling determine actual current-HP gain. Magic Flower, when its combat predicate holds, multiplies the heal request by1.5 and rounds via the source helper. Durable mutation occurs at its actual action checkpoint. Bite's fixed healing is not proportional to HP removed; Reaper's is. Lethal retaliation is not rescued by advance heal credit. |

Raw gain, damage and HP-loss producers still pass their own creature/action
machinery. A local predicate is not complete prevention, rescue, cap or terminal
arbitration. See the exact missing fields in §8.

## 6. Source-specific generation, insertion and upgrades

### Supports are defined by these records, not producer color

* **A28** = red nonbasic Attacks above except Feed and Reaper. **S28** = all red
  nonbasic Skills above. **P14** = all red Powers above. **I70** = their disjoint
  union. Uniform identity masses are1/28,1/28,1/14,1/70 respectively.
* **L34** = C01–C35 except Bandage Up. Equal identity mass1/34, **not** uniform
  rarity then identity. Hand of Greed is eligible despite durable gold.
* Actual Ironclad initializes these pools; Shard widens ordinary reward acquisition,
  not these combat selectors. Playing off-color Distraction/White Noise/Creative AI
  or Branch after an off-color exhaust still uses actual-character Skill/Power/all
  support. Basics, Statuses, curses and SPECIAL/control cards are not candidates.
* Ordinary permanent colorless acquisition includes all35; ordinary red acquisition
  includes Feed/Reaper and72 nonbasics. Permanent transform has separate actual-
  character/colorless/curse laws; no combat HEALING filter is imported into it.

| Source | Exact local sample/choice and mutation schedule |
|---|---|
| Infernal Blade | One A28 sample during use, ordinary turn-cost0 setter, then queued placement. Upgrading source changes1→0 payment only. |
| Discovery | Three distinct I70 IDs: ordered uniform sampling without replacement by duplicate rejection; no reward rarity/pity update. Mandatory one choice, not strategic skip. Copy chosen card, apply route's upgrade checkpoint, turn discount, placement. |
| Chrysalis/Metamorphosis | 3/5 independent S28/A28 identities; duplicates allowed. All identities selected before queued one-card insertions. Positive combat/turn costs become0; negative X untouched. Source+ changes count only. |
| Jack/Magnetism | Independent L34 sample per output/intensity; duplicates/self-generation permitted. Jack samples all before additions; Magnetism samples in start hook and has dead-monster guard. Ordinary output costs. |
| Transmutation | L34 per positive effective-X output, duplicates/self-generation allowed; samples before child additions. Source+ explicitly upgrades each child before ordinary turn discount. `setCostForTurn(0)` changes only nonnegative turn costs: a generated Transmutation keeps combat/turn−1 and receives no new one-play-free flag. It later captures/pays its own X normally unless another origin waives payment. |
| Branch/Toolbox | Branch samples I70 at Exhaust relic callback before later reactions. Toolbox offers three distinct L34, mandatory one, no source turn discount. Neither uses exhausted card/previous owner color. |
| Violence | Build Attack-instance candidate group from draw; randomize candidates and select/remove one for each iteration, at most3/4. Full hand moves selected card to discard, otherwise hand. No strategy choice, no draw callbacks. Exact composed shuffle-conditioned measure is CARD-RC-SHUFFLE, not asserted from the word “random”. |
| Wild Strike/Reckless Charge and one-card insertion children | With current draw size n>0, insertion index uniform0..n−1, mass1/n. Draw-top is append/index n, so new-top gap excluded. Empty pile insertion deterministic. Later insertions use resulting current size; not n+1 gaps or whole-pile shuffle. |
| Deep Breath | Nonempty discard requests EmptyDeckShuffle then explicit Shuffle on combined draw, followed by draw. Retained source establishes requests and shuffle hooks, not a new IID uniform permutation theorem. No Draw does not delete the already-requested shuffle. |

### Master Reality: actual stored card versus animation copy

This table assumes Master Reality present at **both actual checkpoints**; do not
cache its presence for an entire nested invocation. Ordinary cards stop after one
upgrade; repeat-upgradable Searing Blow exposes both checkpoints.

| Route | Actual stored child's extra Master Reality upgrades |
|---|---|
| Hand action → available hand | Action constructor upgrades input; hand effect upgrades stored object again: two possible Searing Blow upgrades. |
| Discovery/card-producing potion/Toolbox → hand | Choice action upgrade, then hand effect upgrade: two. |
| Foreign Influence → hand | No action-level MR upgrade; hand effect only: one. Producer's own upgraded turn discount is separate. |
| Metamorphosis one-card draw insertion | Action copy upgraded; draw effect copy upgraded and that latter object stored: two. Source's3/5 means separate amount-one actions, not one aggregate action. |
| Full-hand coordinate discard overflow | Effect upgrades an animation clone but stores its input. Hand-action/Discovery/Toolbox keeps earlier action-level upgrade only: one; Foreign Influence has none: zero. |

At hand size9, two independently allocated potion copies of selected Searing Blow
can become +2 in hand and +1 in discard. Source+ Transmutation's explicit child
upgrade is additional to the applicable MR route. Status/Curse exemptions remain
separate. These are static path consequences, not captured gameplay or approval of
all mixed copy/relic/cancellation interactions.

## 7. Status, Curse, Note-event and special/control records

### 7.1 Status — five identities, one supplied variant

All have no player upgrade permission (`canUpgrade=false`); Burn's directly
invocable supplied upgrade changes its numeric field but does not make it smithable.

| ID | Kind | Cost/target | Base/upgraded fields and conditional hooks |
|---|---|---|---|
| S01 | Burn | U/None | End-turn queued automatic payload T(2/4), guarded by suppressed-use flag; ordinary allowed play does not apply that end-turn damage. + supplied by enemy upgrade, not player smith. |
| S02 | Dazed | U/None | eth; no use payload or upgraded form. End-turn hand cleanup exhausts, not draw/creation. |
| S03 | Slimed | 1/Self | ex; empty use payload; no upgraded form. Type remains STATUS, not Skill; Corruption/Letter Opener/Skill restrictions do not automatically apply. |
| S04 | Void (`VoidCard`) | U/None | eth; actual `triggerWhenDrawn` queues energy−1, not insertion/retrieval-to-hand. No upgraded form. |
| S05 | Wound | U/None | No use/end-turn/Ethereal payload, no upgraded form; cycles normally until another rule moves/exhausts it. |

Status creation retains origin/destination: Immolate→discard Burn;
Wild Strike→draw Wound; Power Through→hand Wounds; Reckless Charge→draw Dazed;
TURBO→discard Void; Overclock→discard Burn. Enemy-supplied statuses use their own
producer. Hexaghost's Burn-increase path upgrades existing draw/discard Burns at
its first checkpoint, then later inserts three new upgraded Burns; pending future
producers are not already pile members. Soul animation is not a second insertion.

### 7.2 Curses — ten random candidates plus Bell

All cost U, target None, no upgrade and no ordinary use payload unless below.
K01–K10 rarity CURSE form the equal-ID ten-candidate random-curse support;
K11 rarity SPECIAL is excluded. Source-specific acquisition/removal still matters.

| ID | Kind | Local hook/payload; base=only form |
|---|---|---|
| K01 | Clumsy | eth; end-player-turn hook front-queues exhaustion of this hand instance. |
| K02 | Decay | End-turn auto-use suppressed-trigger path front-queues player T(2), not direct HP loss. |
| K03 | Doubt | End-turn auto-use applies Weak1 (source-monster timing flag true); actual draw queues reset of suppressed-use flag. |
| K04 | Injury | Inert payload; still Curse identity for acquisition/hand predicates. |
| K05 | Normality | While present in hand, ordinary play beyond turn count3 is disallowed. Counter does not reset when this instance leaves/reenters; autoplay bypass is source-specific. |
| K06 | Pain | On another card use while this instance is in hand, queue L(1); own play is excluded. Not attack/THORNS damage or one loss per point dealt. |
| K07 | Parasite | Object-removal from permanent deck reduces maxHP3; combat Exhaust is not permanent removal. Transform using object removal invokes it; other removal overloads cannot be assumed equivalent. |
| K08 | Regret | At end-turn enqueue checkpoint capture hand size, including itself, into magic field; later suppressed auto-use requests that L(amount). Do not recalculate after discard/other callbacks. |
| K09 | Shame | End-turn suppressed auto-use front-queues Frail1 with source-monster timing flag true. |
| K10 | Writhe | inn; no additional use payload. |
| K11 | Curse of the Bell (`CurseOfTheBell`) | Inert use; ordinary purgeable selection excludes it. Calling Bell acquisition, not random curse; combat Exhaust is still possible. |

Medical Kit permits Status use/exhaust and Blue Candle Curse use/exhaust with its
L(1) consequence, without retyping. Their legality/payment/hook composition is
CARD-RC-ORIGIN, not permission to apply every ordinary Skill rule. End-turn automatic
Burn/Decay/Doubt/Regret/Shame payloads are suppressed-use paths, not counted ordinary
card plays. Burn/Decay are Block-interacting damage; Pain/Regret explicit HP loss.

Curse offers are not obtains. Omamori's ordinary effect-wrapper prevention,
Darkstone's +6 maxHP, Ceramic Fish's +9 gold, Du-Vu/curse counts and object-removal
hooks must run only at their actual source checkpoints. Note's direct incoming
obtain bypasses the ordinary Omamori effect constructor; no global prevent-any-curse
transaction is established. Ascender's Bane has no standard A0 start; Pride's
Endless route and Necronomicurse's Act2 Tome route are outside current Act1 scope
and not legal Note output through the inspected purge filter.

### 7.3 Stored-event cards — four SPECIAL colorless kinds

| ID | Kind | Type/target | Cost | Base/upgraded fields and payload |
|---|---|---|---|---|
| N01 | Apparition | S/Self | 1 | Intangible1; ex; base eth, + removes eth only. Duration is not a one-hit token. |
| N02 | Bite | A/E | 1 | D(7/8), fixed heal2/3, not HP-removed scaling; no ex. |
| N03 | J.A.X. (`JAX`) | S/Self | 0 | L(3), Strength+2/+3; no ex. |
| N04 | Ritual Dagger | A/E | 1 | Initial misc/base damage15; qualifying Fatal grows3/5; ex. Upgrade does not add initial damage; UUID/growth law §4. |

A legal Note descriptor is a kind/count obtainable as a **permanent removable
card** in a prior standard run, not an arbitrary registered class or previous
combat snapshot. In this lane that includes ordinary red/colorless, ten removable
curses and these four event kinds; other-color basic/nonbasic records are separate
identities owned by their lanes. Temporary-only Statuses, generated specials and
Wish option controls are not legal merely because Note's UI helper can inspect a
fabricated master deck. Bell/Bane/Necronomicurse and outgoing bottled objects are
excluded by the actual purge/bottle filters. Apparition/Bite/JAX/Dagger retrieval
does not put Council of Ghosts/Vampires/Augmenter/Nest in Act1's event pool.

Note fresh-constructs, replays stored upgrades, applies incoming direct obtain hooks
and inserts before selecting outgoing card; retrieved card may itself be outgoing.
Object removal happens before retaining outgoing live reference. ID/count persist
only at next source-qualified room transition; misc/cost/bottle/UUID/history are
not stored. These local facts do not approve a native profile default, malformed-ID
fallback, time policy or loss-interruption behavior; see merged profile draft.

### 7.4 Dependent SPECIAL/control records — not ordinary generator candidates

These make the dependency vocabulary explicit without granting other-character
starts. Parent-card ordinary records belong to Silent/Watcher lanes. All physical
rows here are COLORLESS/SPECIAL; Wish controls are choice presentations, not an
extra permanent/cycling card acquisition or ordinary card play.

| ID | Dependency | Type/target | Cost | Base/upgraded fields and primary behavior |
|---|---|---|---|---|
| N05 | Shiv | A/E | 0 | D(4/6); ex; Blade Dance/Cloak/Storm/Infinite Blades source counts and Accuracy are separate. |
| N06 | Safety | S/Self | 1 | B(12/16); ret, ex; Deceive Reality creates base child. |
| N07 | Smite | A/E | 1 | D(12/16); ret, ex; Battle Hymn/Carve Reality create base child. |
| N08 | Insight | S/Self | 0 | Draw2/3; ret, ex; Evaluate/Pray/Study create base child in draw. |
| N09 | Through Violence | A/E | 0 | D(20/30); ret, ex; Reach Heaven creates base child in draw. |
| N10 | Miracle | S/None | 0 | Energy+1/+2; ret, ex. Deus Ex Machina creates base; Collect creates upgraded future children. No starting Miracle is granted. |
| N11 | Beta | S/None | 2/1 | Create base Omega in draw; ex. Alpha creates base Beta; source upgrade does not directly upgrade child. |
| N12 | Omega | P/Self | 3 | End-turn all-enemy T(50/60); not ordinary Attack damage. |
| N13 | Expunger | A/E | 1 | Stored hit count copies separately from cost; each hit D(9/15); no ex. Conjure Blade constructs even zero-hit child; + child damage does not recapture X or add a hit on this route. |
| N14 | Become Almighty | P/None (choice) | U (not paid separately) | Wish choice grants Strength3/4; no ordinary use-history increment for choosing presentation. |
| N15 | Live Forever | P/None (choice) | U (not paid separately) | Wish choice grants Plated Armor6/8, not Metallicize or immediate Block. |
| N16 | Fame and Fortune | S/None (choice) | U (not paid separately) | Wish choice gains25/30 gold, subject to gold-gain rules; not three simultaneously granted Wish outcomes. |

Conjure Blade passes captured X, source+ adds1, Chemical X adds2, then sets child
count before Master Reality's later upgrade checkpoints. Expunger's stat-copy
override preserves stored count in addition to common copied fields. Alpha+/Beta+
change Innate/payment, not directly Beta+/Omega+. Parent/child upgrades, intrinsic
Retain and actual retention-trigger effects must remain distinct. Full stance,
orb, retain/scry and other-color parent mechanics are not redefined here.

## 8. Exact unclosed common-rule anchors

The records specify concrete local facts; these anchors identify **missing composed
fields**, not blanket “CARD-X applies” substitutes for their payloads. They remain
required whole-spec obligations, not approved approximations or deferred scope.

### CARD-RC-ORIGIN

Missing: complete origin×legality×target revalidation×destination matrix for
ordinary/Havoc/Mayhem/potion/Omniscience/mixed repeats, including Normality, Choker,
Entangled, autoplay failure, Medical Kit/Blue Candle and nested mandatory choices.
Five-repeat captured-energy/true-flag fact is established; it does not settle these
cross-products or authorize universal bypass.

### CARD-RC-COST

Missing: complete ordering of all cost setters, Corruption, free-once writers,
Confusion/redraw, X affordability and upgrade/copy operations in mixed-source
histories. Forethought's positive-combat-cost predicate, cleanup reset and named
Madness/Enlightenment/upgrade operations above are established, not missing values.

### CARD-RC-CAPTURE

Missing: complete caller inventory establishing every recalculation versus use
capture for dynamic damage and repeats; e.g. Mind Blast cached draw count,
Perfected Strike hand/draw/discard tag count, Body Slam use-time Block, Hemokinesis
pre-loss damage and same-instance growth. Do not fabricate one snapshot for all.

### CARD-RC-DEBUFF

Missing: composed `ApplyPowerAction` notification order for immunity, Artifact,
reapplication/stacking, Sadistic's separate compound applications and restoration
exclusion, plus exact duration/retention/rescue interactions. Dark Shackles's
use-time Artifact guard and Sadistic's local predicate are explicit facts.

### CARD-RC-ENDTURN

Missing: complete combined hand-hook/automatic-use/Ethereal/retain/cleanup queue
law under every reachable ordering, interruption and generated replacement. Local
Regret captures size when queued, Clumsy front-exhausts, Burn queues bottom damage,
Decay/Doubt/Shame front-queue their payloads; these are not interchangeable scans.
No newly created card is recursively added to an already captured exhaust set.

### CARD-RC-SHUFFLE

Missing: complete finite joint measure/native source policy for every composed
shuffle (notably Violence candidate-building and repeated shuffles, Deep Breath,
end-turn clone shuffles), including correlations with later random selections.
Named uniform bounded-index laws, duplicate rejection and nonempty insertion
0..n−1 above are retained; marginally legal outcomes are not a root-seed witness.

### CARD-RC-TERMINAL

Missing: complete damage/prevention/rescue/Fatal/split/escape/modal interruption
and action/effect-list arbitration for these producers. Reaper's Sharp Hide-before-
heal mechanism is known, but natural lost-heal outcome, original timing threshold
and approved native headless fidelity choice are not established. A retained heal
action is not guaranteed execution, victory cleanup is not rollback, and advance
healing cannot be credited to survive lethal retaliation.

### CARD-RC-NUMERIC

Established domains, not a generic “all exact integers” assertion:

| Site | Source numeric domain / rounding |
|---|---|
| Base/current damage, Block, magic, costs, misc, upgrade counts, hit/card counts | JVM signed32 integer fields; X−1 and U−2 have distinct semantic domains. Source integer additions/multiplications can overflow; no invented gameplay pile/turn/count cap follows. |
| Card Block calculation | Convert base integer to binary32; iterate `modifyBlock(card)` then `modifyBlockLast` in installed order; lower-clamp0 then source conversion H below. No aggregation rounding: Second Wind calculates one source Block amount and emits separate events. |
| Card damage calculation | Binary32 intermediates; relic modifiers, owner damage-give powers, stance, target receive powers, owner final-give, target final-receive; lower-clamp0 then H below. Preview without target omits receive stages; All-enemy arrays have per-target results. This is not permission to reorder/simplify those float operations. |
| Heavy Blade | Strength amount temporarily multiplied by3/5 as signed32 before common float calculation, restored by integer division afterward. Small signed examples follow; large multiplication overflow is not proven harmless. |
| Searing Blow | Per-upgrade signed32 increment4+n and count increment, not evaluation of a potentially overflowing closed-form formula. The polynomial is an ordinary-range mathematical consequence, not a new source arithmetic domain. |
| Rampage/Dagger | Signed32 base/misc increments; Rampage lower-clamps negative resulting damage0, Dagger copies misc into baseDamage. UUID attribution is independent of numeric width. |
| Shackled | Its own constructor/stack clamp−999..999; do not project that cap onto misc/card count/Searing Blow upgrades. |
| Strength | `StrengthPower` constructor, stack and reduction clamp the resulting amount to−999..999; this is a Strength-power-specific rule, not a universal stat/power cap. |
| Parasite removal | Its permanent-deck removal requests max-HP decrease3; `AbstractCreature.decreaseMaxHealth` floors resulting max HP at1 and caps current HP down to that result. This local removal path does not establish a universal max-HP floor for unrelated rules. |
| `AbstractCreature.addBlock` | After callbacks and source floor conversion, resulting `currentBlock` is capped above at999. This operation-specific cap is not a universal Block/gain/power limit. |
| Juggernaut | Positive float gain-notification predicate, not integer card baseBlock>0. Downstream actual Block cap/modification remains creature-specific. |

**Source-floor domain matters.** Retained `MathUtils.floor(float)` reconstruction
and bytecode establish H(t): promote binary32 t to binary64, add16384, perform JVM
double-to-signed32 conversion (toward zero, saturating outside range; NaN→0), then
subtract16384. After the card's lower clamp, finite0≤t<2147467264 has H(t)=floor(t).
At t=2147467264 and above, conversion saturation yields2147467263 instead; a blanket
unbounded mathematical floor would be wrong. Positive infinity gives that same
value; NaN survives the `<0` clamp and yields−16384. These last cases describe the
source arithmetic, **not** evidence that a legal card history reaches nonfinite
values. No source-runtime invocation was used to establish them. The private
arithmetic model checked the threshold and ordinary-domain controls, not gameplay
reachability. This is not `Math.round`, binary64-throughout combat, or permission
to silently idealize large source values.

Missing: exhaustive creature/power/relic clamp/rounding/overflow reachability
inventory for composed damage, HP/maxHP, Strength, healing, Block and permanent
growth, plus an approved native behavior for representational exhaustion. The
existing architecture requires administrative fault rather than truncation/fake
death/retry; that architecture is not proof that source overflow cannot be legally
reached. No arbitrary native numeric cap or binary64 substitution is approved here.

## 9. Validation, discriminating future cases and handoff

Actually run: private standard-library guard/arithmetic tests, missing/duplicate-entry,
containment/overwrite, changed-pin and duplicate-record negative controls; targeted
static javap/CFR inspections; four-pin integrity checks; record completeness,
source-locator mapping and public path/link/whitespace/privacy checks. Two static
name assumptions failed closed and are retained, then corrected using actual
caller references/exact ZIP preflight. These are **not gameplay tests**.

Future acceptance cases (not executed, not original-run witnesses):

| Case | Discriminator |
|---|---|
| Paired coverage | 75/35/5/11/4+12 distinct records; all cost/target/keyword deltas, not just base damage. |
| Signed and rounded | Heavy Blade Strength±2 gives20/24 and8/4 before other modifiers; ordinary B5 with Dexterity2 gives7, Frail-only gives3, not double-scaled Iron Wave. |
| Upgrade vs copy | Searing Blow0..4 gives12/16/21/27/34 after actual upgrades; count-taking constructor, stat-copy, Note replay and two-stage MR must be distinct tests. |
| Conditional free flag | Positive combat cost with turn0 receives flag; combat0 with raised turn cost does not; X/U unchanged. Ordered Forethought+ A,B versus B,A differs; empty selection is legal. Self-generated Transmutation remains X/not automatically free despite the ordinary zero-turn-cost setter. |
| Dynamic capture | Impatience playable with Attack but no draw unless action-time hand lacks Attacks; Spot has same candidate set for hidden attacking/nonattacking intent; Dropkick snapshots Vulnerable at its own action. |
| Growth identity | Rampage same-UUID battle instances versus new-UUID copies; Dagger matching permanent source and Note misc reset; Stab exhaust→loss→Exhume has no retroactive history catch-up. |
| Distinct hooks | Brutality post-draw queues draw before loss; Panache stacking changes damage not remaining counter; staggered Bombs keep age/payload. |
| Destination | Inserting into size1 places below existing top; size0 deterministic. Potion copies at hand9 show two-stage/overflow MR difference without deleting either card. |
| Status/curse | Burn2/4 and Decay2 use damage, Regret captured count and Pain use HP loss; moved Void loses no energy merely for movement. Parasite combat Exhaust differs from permanent removal; Bell is not random/storable purge output. |
| Fatal/terminal | Nonfinal Feed/Greed/Dagger mutations commit immediately when eligible; split/Minion/half-dead exclusions and Reaper lethal retaliation/cancellation need independently settled shared expectations. |

Self-review and checks cannot clear the independent-review gate. PR remains draft,
own issue in progress for main readback. Only this document is public; source,
helpers, raw evidence and failed attempts remain private. No product/test/dependency
changes or per-author product-suite rerun occurred. The current-wave exception for
two known CUDA-environment baseline failures is documentation-only, not a green
suite or waiver of a new failure, factual gap or reviewer finding. Whole-spec human
approval, implementation-plan review and implementation-start approval remain
separate and ungranted.
