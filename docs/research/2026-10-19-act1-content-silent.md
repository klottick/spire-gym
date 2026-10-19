# Act 1 reachable Silent cards: paired normative records

Date: 2026-10-19. Bead: `spire-gli.2.3.19`.

**Author draft for independent review.** These are standalone source-derived
records for 75 green kinds (4 basic, 19 common, 33 uncommon, 19 rare) and the
dependent colorless Shiv. They replace missing public B07 records, not the whole
execution specification. Local rules below are normative fidelity requirements;
the explicitly unclosed composition fields in §10 remain blockers. No gameplay
test, original-class execution, complete-transitive clearance or human approval
is claimed.

## 1. Reachability and record interpretation

The run is standard, fully unlocked, unmodded Ironclad A0 Act 1. Prismatic Shard
can expose off-color non-basic rewards; Note for Yourself can retrieve a legally
stored prior-run card, including the four green basics. This does **not** start
Silent, import her starting deck/resources, add her home events, or enable
Silent-exclusive relics/potions. Shiv is a temporary dependent kind, not a new
permanent-reward or legal stored-Note kind. Acquisition route eligibility and the
native Note profile transaction are separate from these card payloads.

Each table row is a paired record. `a→b` is fresh base→once-upgraded; a single
value is identical in both. Every kind here accepts at most one ordinary upgrade;
upgrading an already upgraded instance does nothing. Upgrade changes only the
listed cost/scalars/flags, not type, color, rarity or target. Instance changes
from other sources are not erased by pretending an upgrade is a new card.

* `A/S/P`: Attack/Skill/Power. `EN/ALL/SELF/NONE` are the source target fields
  ENEMY/ALL_ENEMY/SELF/NONE, **not** an autoplay legality specification. EN requires
  an ordinary enemy target; ALL, SELF and NONE require no player-selected enemy.
  A hand choice remains a later native continuation even with target NONE.
* `D/B/M`: baseDamage/baseBlock/baseMagicNumber. Unlisted scalar fields are the
  source's unused `-1`, not zero payloads. Some magnitudes are use-site literals,
  not M; those are written in the effect column. Card Block/damage uses the
  corresponding calculated instance value, not an unconditional printed integer.
* Cost is fresh mutable combat cost; fresh turn cost equals it. `X=-1` and
  `unplayable=-2` are distinct sentinels. All numeric costs and fields are signed
  32-bit source integers; §9 states arithmetic/rounding and actual local clamps.
* `E`: inherent Exhaust. `I`: Innate. A missing E or I means false in both
  versions. All 76 kinds start non-Ethereal, with retain/selfRetain/freeToPlayOnce
  false, misc=0, and ordinary damage type NORMAL. Power-card removal on use is not
  the E flag. External exhaust/prevention/destination hooks remain separate.
* Green is the color for every row except Shiv. `isMultiDamage` is true exactly
  for Dagger Spray, All-Out Attack, Die Die Die and Grand Finale. Other multi-hit
  attacks are separate selected-target hits, not all-enemy matrices.
* Source IDs, including historical spellings, are authoritative. Display names
  in parentheses disambiguate them. Constructor tags are empty except Strike_G
  (STRIKE, STARTER_STRIKE), Defend_G (STARTER_DEFEND), Underhanded Strike (STRIKE),
  and Venomology (HEALING). No type is inferred from an inability to play.

Effect sequences describe local requests and their source-specific placement;
they do not promise that every later request survives lethal/terminal clearing.
No universal FIFO/drain/rollback rule is introduced.

## 2. Basic records: four distinct green kinds

Rarity BASIC; available here through legal Note storage, not Shard's ordinary
non-basic reward selection. Debug-only Strike/Defend branches are outside scope.

| Source ID (display) | Type/target; cost | Scalars; flags | Paired local payload |
|---|---|---|---|
| `Strike_G` (Strike) | A/EN; 1 | D=6→9 | One damage hit. |
| `Defend_G` (Defend) | S/SELF; 1 | B=5→8 | One player Block gain. |
| `Neutralize` | A/EN; 0 | D=3→4, M=1→2 | Damage, then M Weak. |
| `Survivor` | S/SELF; 1 | B=8→11 | Block, then selected discard of 1 remaining hand card. |

## 3. Common records: nineteen kinds

Rarity COMMON. Shared discard, poison, delayed, cost and damage qualifications in
§§6–9 are part of these records, not optional footnotes.

| Source ID (display) | Type/target; cost | Scalars; flags | Paired local payload |
|---|---|---|---|
| `Acrobatics` | S/NONE; 1 | M=3→4 | Request M draws, then selected discard 1. |
| `Backflip` | S/SELF; 1 | B=5→8 | Block, then draw 2. |
| `Bane` | A/EN; 1 | D=7→10 | At card use capture the target and both DamageInfo payloads; enqueue the first hit then a dedicated BaneAction. Its Poison, target-HP and initial-duration source-dying gates are specified in §7, not a generic second DamageAction. |
| `Blade Dance` | S/NONE; 1 | M=3→4 | Create M ordinary Shivs, not Shiv+ by the card upgrade. |
| `Cloak And Dagger` (Cloak and Dagger) | S/SELF; 1 | B=6, M=1→2 | Block, then create M ordinary Shivs. |
| `Dagger Spray` | A/ALL; 1 | D=4→6 | Two distinct all-enemy damage requests using the card's calculated matrix. |
| `Dagger Throw` | A/EN; 1 | D=9→12 | Damage, draw 1, selected discard 1. |
| `Deadly Poison` | S/EN; 1 | M=5→7 | Apply M Poison. |
| `Deflect` | S/SELF; 0 | B=4→7 | Block. |
| `Dodge and Roll` | S/SELF; 1 | B=4→6 | Block now; install Next Turn Block with the same calculated Block captured at use. |
| `Flying Knee` | A/EN; 1 | D=8→11 | Damage; install Energized 1. |
| `Outmaneuver` | S/NONE; 1 | — | Install Energized 2→3; these amounts are literals, not M. |
| `PiercingWail` (Piercing Wail) | S/ALL; 1 | M=6→8; E | Request −M Strength for each monster; separately request Shackled restoration M only for monsters without Artifact at use-time. See §7. |
| `Poisoned Stab` | A/EN; 1 | D=6→8, M=3→4 | Damage, then M Poison. |
| `Prepared` | S/NONE; 0 | M=1→2 | Draw M, then selected discard M. |
| `Quick Slash` | A/EN; 1 | D=8→12 | Damage, then draw 1. |
| `Slice` | A/EN; 0 | D=6→9 | One damage hit. |
| `Underhanded Strike` (Sneaky Strike) | A/EN; 2 | D=12→16 | Damage; at the later conditional-energy action, gain 2 if totalDiscardedThisTurn>0. This is not cost cancellation/refund. |
| `Sucker Punch` | A/EN; 1 | D=7→9, M=1→2 | Damage, then M Weak. |

## 4. Uncommon records: thirty-three kinds

Rarity UNCOMMON.

| Source ID (display) | Type/target; cost | Scalars; flags | Paired local payload |
|---|---|---|---|
| `Accuracy` | P/SELF; 1 | M=4→6 | Install Accuracy M; Shiv identity-specific rewriting in §8. |
| `All Out Attack` (All-Out Attack) | A/ALL; 1 | D=10→14 | All-enemy damage, then random discard 1. |
| `Backstab` | A/EN; 0 | D=11→15; E,I | One damage hit. |
| `Blur` | S/SELF; 1 | B=5→8 | Block; install Blur 1, preserving Block at the relevant next start-turn removal checkpoint. |
| `Bouncing Flask` | S/ALL; 2 | M=3→4 | M separate Poison-3 applications to source-selected random enemies, with repeats allowed; §7 gives sampling checkpoints. |
| `Calculated Gamble` | S/NONE; 0 | E=true→false | Snapshot remaining hand size n at its action; for n>0, random-discard n then request n draws; n=0 enqueues neither discard nor draw. Both versions call the same non-extra-draw action branch. |
| `Caltrops` | P/SELF; 1 | M=3→5 | Install Thorns M; retaliate only on qualified incoming damage callbacks (§7), including fully blocked qualifying attacks. |
| `Catalyst` | S/EN; 1 | E | If Poison exists at action resolution, request an additional current amount→twice current amount as Poison. No Poison means no application; not direct power multiplication. |
| `Choke` | A/EN; 2 | D=12, M=3→5 | Damage; separately install Choked M. Its on-use hook requests M HP loss; it removes at afflicted start-turn. |
| `Concentrate` | S/SELF; 0 | M=3→2 | Selected discard M (all available when undersized), then gain 2 energy even if fewer were available. No extra minimum-hand legality override. |
| `Crippling Poison` (Crippling Cloud) | S/ALL; 2 | M=4→7; E | Unless monsters are basically dead, visit non-dead/non-dying monsters and enqueue M Poison then 2 Weak per monster. Separate applications. |
| `Dash` | A/EN; 2 | D=10→13, B=10→13 | Block first, then damage. It remains an Attack for type hooks. |
| `Distraction` | S/NONE; 1→0 | E | Fresh generated Skill from actual-character source pools; set its eligible turn cost to 0, then temporary hand creation. §8 defines filtering, not a green pool. |
| `Endless Agony` | A/EN; 0 | D=4→6; E | Damage on use. Actual draw separately creates one stat-equivalent temporary copy at queue top; creation is not a draw. |
| `Escape Plan` | S/SELF; 0 | B=3→5 | Draw 1 with a follow-up that scans DrawCardAction.drawnCards for any Skill; if found, enqueue one captured-B Block gain, then stop scanning. |
| `Eviscerate` | A/EN; 3 | D=7→9 | Three distinct hits. Each received discard notification lowers eligible turn cost by 1; draw resets it from combat cost minus turn discard counter. Exact hooks in §6. |
| `Expertise` | S/SELF; 1 | M=6→7 | At action resolution snapshot hand size h; enqueue one draw of M−h only when h<M; otherwise enqueue none, not a zero-count draw or a loop maintaining M after reactions. |
| `Finisher` | A/EN; 1 | D=6→8 | At follow-up resolution count ATTACK entries in cardsPlayedThisTurn, subtract 1, and enqueue that many hits if target exists with positive HP. Ordinary current Finisher is excluded by the subtraction. |
| `Flechettes` | A/EN; 1 | D=4→6 | At its action, enqueue one hit per current hand card whose type is SKILL, including unplayable Skills; not per playable card. |
| `Footwork` | P/SELF; 1 | M=2→3 | Apply M Dexterity. |
| `Heel Hook` | A/EN; 1 | D=5→8 | Always prepend damage; if target is non-null and already has Weak at helper entry, also enqueue gain 1 energy then draw 1 behind that damage. Weak only gates the energy/draw; no positive-HP predicate or post-hit Weak retest is added. |
| `Infinite Blades` | P/SELF; 1 | I=false→true | Install Infinite Blades 1; at start-turn create one ordinary Shiv per power amount unless monsters are basically dead. Upgrade does not increase amount. |
| `Leg Sweep` | S/EN; 2 | B=11→14, M=2→3 | Apply M Weak, then player Block. |
| `Masterful Stab` | A/EN; 0 | D=12→16 | Damage. Each qualifying per-instance tookDamage notification calls combat-cost update +1, not a global combat-loss-count formula (§6). |
| `Noxious Fumes` | P/SELF; 1 | M=2→3 | Install M; at player post-draw turn-start request amount Poison separately for each non-dead/non-dying monster, unless monsters are basically dead. |
| `Predator` | A/EN; 2 | D=15→20 | Damage; install Draw Card Next Turn 2. |
| `Reflex` | S/NONE; unplayable | M=2→3 | Ordinary canUse always false. Manual-discard callback appends draw M; direct use body also requests draw M if another origin reaches it, not an ordinary legality grant. |
| `Riddle With Holes` (Riddle with Holes) | A/EN; 2 | D=3→4 | Five distinct damage hits. |
| `Setup` | S/NONE; 1→0 | — | Choose one remaining hand instance, move toward draw-top; set one-play-free only when its mutable combat cost>0. Never a universal numeric-zero rewrite (§6). |
| `Skewer` | A/EN; X | D=7→10 | One hit per positive effective X; upgrade changes per-hit damage, not hit count (§8). |
| `Tactician` | S/NONE; unplayable | M=1→2 | Ordinary canUse always false; manual-discard callback prepends gain M energy. Its use body is empty. |
| `Terror` | S/EN; 1→0 | E | Apply 99 Vulnerable duration, not an infinite-duration administrative sentinel. |
| `Well Laid Plans` (Well-Laid Plans) | P/NONE; 1 | M=1→2 | Install Retain Cards M; optional end-turn selection up to amount, with Ethereal/Pyramid/Equilibrium distinctions (§6). |

## 5. Rare records and dependent Shiv

The first nineteen rows have rarity RARE. Shiv has rarity SPECIAL and color
COLORLESS. A scalarless row still has the explicit literal payload below.

| Source ID (display) | Type/target; cost | Scalars; flags | Paired local payload |
|---|---|---|---|
| `A Thousand Cuts` | P/SELF; 2 | M=1→2 | Install Thousand Cuts M; after-card-played callback requests amount all-enemy THORNS damage, not another Attack play. |
| `Adrenaline` | S/SELF; 0 | E | Gain 1→2 energy, then draw 2. |
| `After Image` | P/SELF; 1 | I=false→true | Install After Image 1; on-use callback requests Block equal to power amount. Upgrade changes Innate, not Block. |
| `Venomology` (Alchemize) | S/SELF; 1→0 | E | Sample a limited-route potion at use, enqueue obtain; actual Ironclad potion eligibility, Sozu and belt resolution are distinct (§8). |
| `Bullet Time` | S/NONE; 3→2 | — | Enqueue No Draw then eligible current-hand turn-cost zeroing. Does not set Setup's free flag or rewrite X/unplayable sentinels. |
| `Burst` | S/SELF; 1 | M=1→2 | Install Burst M; next M eligible non-purge Skill uses each enqueue one additional play, not M extra plays of a single Skill. |
| `Corpse Explosion` | S/EN; 2 | M=6→9 | Apply M Poison, then separately CorpseExplosionPower 1. On qualifying death request owner's then-max-HP × explosion stacks as all-enemy THORNS (§7). |
| `Die Die Die` | A/ALL; 1 | D=13→17; E | One all-enemy damage request. |
| `Doppelganger` | S/SELF; X | E | Effective X gains a +1 upgraded bonus; if positive, install that much Energized then Draw Card Next Turn (§8). |
| `Envenom` | P/SELF; 2→1 | — | Install Envenom 1; onAttack with positive damageAmount, NORMAL damage and target other than owner prepends Poison equal to power amount. |
| `Glass Knife` | A/EN; 1 | D=8→12 | Two hits, then UUID-scoped baseDamage−2 floored at zero across matching battle instances (§8); not permanent master-deck loss. |
| `Grand Finale` | A/ALL; 0 | D=50→60 | Ordinary canUse also requires draw pile size=0; payload is one all-enemy hit. Discard pile need not be empty. Other origins' checks remain distinct. |
| `Malaise` | S/EN; X | E | For positive effective X (+1 upgraded), request −X Strength then X Weak; no restoration power (§8). |
| `Night Terror` (Nightmare) | S/NONE; 3→2 | M=3; E | Choose one remaining hand card; capture/reset a stat-equivalent template when constructing its independent delayed power; next start-turn request M temporary copies. |
| `Phantasmal Killer` | S/SELF; 1→0 | — | Install Phantasmal 1; next start-turn install Double Damage with initial duration 1 and reduce Phantasmal by 1. NORMAL damage ×2, not ×2 per stack. |
| `Storm of Steel` | S/NONE; 1 | — | Snapshot remaining hand size n; prepare one Shiv→Shiv+ template and construct its temporary-creation action before prepending discard n. Discard executes first; insertion later makes n stat-equivalent copies from that prepared template (§8). Upgrade changes template state, not n. |
| `Tools of the Trade` | P/SELF; 1→0 | — | Install Tools 1; post-draw start-turn request amount draws, then selected discard amount. Stacks increase both, not the number of independent selections. |
| `Unload` | A/EN; 1 | D=14→18 | Damage; snapshot all current non-Attack hand identities and prepend specific discards in iteration order (therefore reverse pending order). Recheck membership at each discard. |
| `Wraith Form v2` (Wraith Form) | P/SELF; 3 | M=2→3 | Apply IntangiblePlayer M, then independently Wraith Form −1; downside loses 1 Dexterity per power amount magnitude at end-turn (§7). |
| `Shiv` | A/EN; 0 | D=4→6 before Accuracy; E | One damage hit; constructor includes current Accuracy amount in baseDamage. See §8 for existing-instance rewrites and generated upgrades. |

## 6. Choices, discard, draw, retention and corrected instance state

### Setup and the non-equivalent cost layers

Setup's empty hand is a no-op; one remaining card is automatic; otherwise select
exactly one. Movement keeps the existing object. In **both** movement branches
only `cost>0` grants `freeToPlayOnce=true`. Here `cost` means mutable combat cost,
not immutable printed cost and not `costForTurn`. Neither cost number is zeroed.

| Selected instance at Setup resolution | Newly granted free flag? |
|---|---|
| Combat cost positive; turn cost zero or otherwise changed | Yes |
| Combat cost zero; turn-only cost positive | No |
| X combat sentinel −1 | No; no new X/payment bypass |
| Unplayable combat sentinel −2 | No; no new ordinary play permission |
| Guard fails but a free flag already exists | Existing flag is not cleared by Setup |

Ordinary reset restores turn cost from combat cost but leaves the free flag.
Use cleanup clears that flag **after after-use callbacks and before destination
processing**. This is not an unconditional “free until played” lifetime: Confusion
on an eligible draw clears the flag even when sampled cost equals current cost.
Forethought uses the same positive-combat-cost guard but moves to draw-bottom.
Forethought+ permits zero selections and preserves the selected **sequence**;
equal subsets in different selection orders are not interchangeable. No sorting
of card identities may replace those ordered choices. This cross-source rule
applies equally when the selected cards are green.

Bullet Time instead visits hand cards at its own action and uses the turn-cost
setter: only an already nonnegative turn cost is rewritten and clamped to zero.
It supplies no new free flag. Distraction's generated card uses the same setter
with a negative argument, which clamps eligible turn cost to zero. Sentinel
protection, ordinary canUse overrides, No Draw and free state remain independent.
No Draw is a debuff, removed at player end-turn; prevention/removal can change
its presence. It prevents draw requests from succeeding, not non-draw creation.

### Masterful Stab: location-sensitive notification state

A fresh-kind Masterful Stab starts at combat cost zero regardless of earlier
combat losses. Its notification updates **its own** combat cost +1 through the
shared cost updater, preserving the combat-versus-turn discount difference and
flooring costs at zero. The qualifying player damage path dispatches only after
positive final HP loss in COMBAT; this includes normal damage that loses HP, not
only DamageType.HP_LOSS. The dispatch visits **hand, then discard, then draw**,
not exhaust, limbo or every historical card. One loss of 5 HP gives one callback,
not five; wholly prevented/zero final loss does not qualify.

Therefore do not derive its price from a global combat HP-loss count. Exhaust
the instance with True Grit+, lose HP while it is exhausted, then Exhume that
same instance: neither Exhume branch catches up missed notifications. A fresh
kind copy and a stat-equivalent copy have different cost capture (§8). These are
source-derived discriminators, not executed gameplay/seed witnesses.

### Source-specific discard and draw transactions

* The selected/random DiscardAction first stops if monsters are basically dead.
  With hand size≤requested count it discards that fixed current size without a
  selection. Otherwise selected discard requires the requested count; random
  discard repeatedly samples remaining hand entries using cardRandomRng.
  Concentrate has no invented “must have M cards” legality restriction.
* Ordinary selected/random/all effect discards move each chosen card, call its
  manual-discard callback, then increment the counter. The all/undersized
  end-turn branch suppresses that callback. `incrementDiscard` increments the
  counter even for end-turn requests, but notifies hand→discard→draw cards and
  relic manual-discard hooks only when turnHasEnded is false and endOfTurn=false.
  Thus counter increments and callback eligibility are not identical predicates.
* Reflex appends its draw; Tactician prepends its energy. Unload's specific
  discard instead checks the captured card is still in the group, moves it,
  increments/notifies, **then** calls its manual-discard callback. Do not impose
  the selected-action callback order on this producer. Non-Attack includes Powers,
  Statuses and Curses, not just Skills; Reflex/energy callbacks never retype cards.
* Calculated Gamble captures n before discard reactions. Both base and upgraded
  card pass `false` to its action: n>0 queues discard n then draw n for execution,
  while n=0 enqueues neither discard nor draw. The action's unused extra-draw
  branch is not Calculated Gamble+'s effect. Storm of Steel captures n and prepares
  one template and its temporary-creation action before discard executes; discard
  runs first and insertion later makes n stat-equivalent copies (§8). Reaction-
  drawn cards are not recursively added to the original discard set or count.
* Eviscerate's `didDiscard` lowers eligible current turn cost by 1. Its on-draw
  hook instead sets eligible turn cost to combat cost−totalDiscardedThisTurn,
  floored at zero. Turn-start resets attributes and applies powers. Its makeCopy
  constructs a fresh kind, then, in combat, invokes this history-based setter on
  the **source**, not the returned fresh object; do not silently “repair” that
  source behavior into a returned-copy catch-up rule. Mixed cost writer order
  remains CARD-COST, not a universal global-counter price formula.
* DrawCardAction clears its shared draw history on its designated initial update,
  including No Draw/empty/full-hand exits. No Draw, nonpositive request, no cards
  in either pile, or a ten-card hand ends with the follow-up still scheduled.
  Requested count is limited by remaining hand capacity; shortage in draw pile
  schedules draw/reshuffle/remainder with history preserved across those pieces.
  Actual draw records the card before the player's draw callback. Merely moving,
  generating or revealing a card does not invoke the same draw transaction.
* Escape Plan's follow-up scans the then-current **shared draw-history list**,
  not a hidden top-card preview. A no-draw exit leaves no Skill to qualify.
  Nested/reaction-draw history ownership and interrupted follow-ups remain the
  explicit CARD-HISTORY gap; this is not a frozen final-hand-type shortcut.
* Finisher's hit count is sampled in its action, not from a preview; ordinary
  history includes the current attack and the source subtracts one. Flechettes
  samples the hand at its action, not at policy input. Expertise captures its
  deficit M−h once and enqueues exactly one draw only when h<M; otherwise enqueue
  none. Neither helper's absent child action may be replaced by a synthetic
  DrawCardAction(0): an actual default-history-clearing zero-count draw clears
  shared draw history on its initial update before exiting. Those absent children
  perform no such clear; other nested-history writers remain CARD-HISTORY.
  Repeat/interposed history can therefore matter without a proved nested outcome.

Well-Laid Plans' power requests its optional up-to-amount selection at player
end-turn only with a nonempty hand and without Runic Pyramid or Equilibrium.
Selected non-Ethereal cards receive temporary retain; selected Ethereal cards do
not. This is not inherent selfRetain, automatic selection, or Ethereal immunity.
Shared Ethereal/retain/Establishment/cleanup interleaving remains CARD-ORDER.
All real choices resume their parent once; payment, draws and already committed
discards must not be repeated upon resumption.

## 7. Poison, debuffs, retaliation and delayed powers

### Bane's captured follow-up and local gates

At card use Bane retains the selected target and constructs both DamageInfo
payloads, each owned by the player with the card's then-calculated damage and
damage type, before either hit resolves. The first request is an ordinary
DamageAction; the second is a dedicated BaneAction with its own captured target
and DamageInfo. Do not
recalculate that second payload from later card fields or substitute a generic
second DamageAction for its distinct local conditions.

Each entered BaneAction update exits immediately on a null target. It then checks
the target's current Poison; absence ends the action without a hit. With Poison
present, the initial-duration branch (duration equals its initial 0.01f) additionally
requires a non-null target with positive HP; **within that branch**, a non-THORNS
payload whose owner is dying cancels the action. This is not an every-update
source-death check, a universal
canUse restriction, or a generalized null-source/origin policy. At completion
after its duration update, it directly delivers the captured hit only if the
target is still non-null with positive HP. Following that damage call it checks
basically-dead cleanup and prepends its 0.1f wait. These entered local checkpoints are
known; recipient damage/prevention/rescue, terminal arbitration and whether later
work continues remain CARD-DEATH, not reasons to omit the local gates.

### Application and Poison

ApplyPowerAction rejects null/dead-or-escaped targets and handles source hooks,
prevention and Artifact **per application**. Artifact tests a debuff's type,
not the color/type of the whole card. Poison, Weak, negative Strength, Shackled,
Choked, Corpse Explosion and Wraith Form's downside are separate debuff requests.
An already-present ordinary matching ID stacks using the action amount; Nightmare
(`Night Terror`) is explicitly excluded from matching-ID stacking. Removing Poison
does not remove Corpse Explosion, and preventing one request is not a blanket
veto of a multi-effect card. Sadistic Nature and other reactions can interpose;
their complete resulting totals are not established here.

Poison schedules a captured-amount HP_LOSS damage action at the afflicted
creature's start-turn, only in COMBAT and while monsters are not basically dead.
The loss action itself stops outside COMBAT; at completion it damages a positive-
HP target, then decrements the **currently present** Poison by one, removing it
at zero, checks basically-dead cleanup and prepends its wait. Damage is not an
Attack hit or a Block gain; its recipient HP_LOSS/prevention path must be retained.
Noxious Fumes instead produces new enemy Poison at the player's **post-draw**
start-turn. Those two clocks must not be collapsed.

Catalyst samples current Poison at its own action and enqueues another Poison
application of P→2P. If unmodified and accepted against the same surviving power,
the resulting total is 2P→3P. The application carries that raw stack amount
separately from its constructed Poison object's amount. Artifact/pre-application
reactions or power replacement mean this is **not** an unconditional arithmetic
assignment of that total. No existing Poison yields no new application, not a
free initial stack. The upgraded action
assumes its ordinary non-null target; no generic null-origin policy is inferred.

Bouncing Flask samples an initial target at card use using cardRandomRng. Each
non-final bounce action preselects the next target before the current Poison
application resolves, and prepends the next bounce behind its current application/
wait. The next target can therefore become stale after intervening reactions;
there is no automatic “all targets chosen after prior application” rule. The
selected target's positive HP gates its application. A null target stops the
bounce; basically-dead detection clears post-combat work and stops it. Sampling
with null exception and aliveOnly=true uses current monster-list entries excluding
halfDead/isDying/isEscaping; no exclusion of the previous target is passed. Empty
support returns null; singleton still requests a bounded index. Conditional RNG
and interposed death scheduling remain CARD-RNG/CARD-DEATH, not an IID allocation
of total 9/12 Poison to a fixed initial roster.

Corpse Explosion onDeath requires owner HP≤0 and monsters not basically dead.
Its payload uses owner **maxHealth at that callback** times current explosion
amount, a pure matrix with null damage owner and THORNS type. It does not copy
current Poison or remaining HP. Chained death, split/half-dead callback arbitration,
kill credit and last-enemy cancellation remain CARD-DEATH; the local callback is
not a proof that every nominal explosion reaches every enemy.

Piercing Wail visits the monster list twice: first queues negative Strength for
all, then, based on Artifact presence **at card use**, queues positive Shackled
restoration only for those then lacking Artifact. The first applications have
not resolved before that second pass. Each queued application still has its own
resolution-time prevention/hooks. Shackled itself is a debuff; at end-turn it
requests its amount of Strength back then removes itself. Cleansing/prevention
can therefore break naive loss/restoration symmetry. Malaise installs no such
restoration. Do not decide restoration eligibility by recomputing Artifact only
after Wail's Strength requests have consumed it.

### Play and damage callbacks

After Image uses onUseCard; Thousand Cuts uses onAfterCardPlayed; Choked uses
onUseCard and requests HP loss from its owner, then removes at its start-turn.
They are not one undifferentiated “after everything” event, and a newly installed
power is not assumed to receive its parent card's earlier callbacks. Callback
suppression, mixed repeats and terminal clearing remain shared execution fields.

Caltrops' Thorns requires an incoming type other than THORNS/HP_LOSS, non-null
attacker, and attacker different from owner. It does **not** require positive
unblocked damageAmount; a fully blocked qualifying Attack still retaliates.
It prepends THORNS damage to the attacker and returns the incoming amount
unchanged. It neither recursively retaliates to THORNS nor triggers on every
explicit HP-loss cost. Envenom instead requires positive damageAmount and NORMAL
type, target other than owner; it prepends its Poison application. Multi-hit
attacks preserve each hit and callback rather than combining their damage first.

### Separate delayed lifetimes

| Installed state | Local delivery/lifetime |
|---|---|
| Energized | onEnergyRecharge gains its stored amount synchronously, then requests removal; constructor and stacking cap at 999. |
| Draw Card Next Turn | post-draw start-turn appends amount draw then removal; priority 20. |
| Next Turn Block | start-turn appends stored-amount Block then removal; Dodge and Roll has already captured its calculated card Block, not next-turn base Block recalculation. |
| Tools of the Trade | post-draw start-turn appends amount draw then amount selected discard; priority 25. |
| Infinite Blades | start-turn creates amount ordinary Shiv templates; stacks add amount. |
| Noxious Fumes | post-draw start-turn applications; stacks add amount. |
| Nightmare | start-turn appends creation from stored template then removal by power ID; separate selected templates remain independent powers. |
| Phantasmal | start-turn appends Double Damage (new power amount 1, application amount=current Phantasmal amount), then reduces Phantasmal by 1. New application and stacking an existing Double Damage are distinct. |
| Double Damage | NORMAL damage multiplies by binary32 2.0 once, independent of stack amount; amount controls duration, not repeated doubling. Player-source end-round reduces by 1/removes at zero. |
| Blur | amount controls duration, reduced/removed at end-round; start-turn Block-removal logic tests presence. It is not Barricade or immunity to hits. |
| IntangiblePlayer | final-receive modifier maps values>1 to 1; priority 75; end-round reduces/removes amount. Other HP-loss paths and modifier ordering remain shared. |
| Wraith Form | independent DEBUFF, signed amount initially −1; end-turn requests Dexterity change equal to amount. Its callback has no isPlayer guard. Stacks add negative amounts; Intangible expiry does not remove it. |

These named callbacks/priorities are not a completed ordering among every power,
relic, pre-turn action, draw, retained card and terminal event. Wraith downside
can itself be prevented/cleansed independently of Intangible; do not assume both
are a single lifetime or that Dexterity cannot become negative.

## 8. Generation, copying, mutation and X

### Source-specific creation and actual character

Distraction concatenates eligible entries from srcCommonCardPool,
srcUncommonCardPool and srcRareCardPool, selecting SKILL and excluding HEALING
tags, then draws one bounded list index using cardRandomRng. There is no preceding
rarity-weight draw here. In this standard run these are the actual Ironclad's
source pools, initialized from his pool, not the producing card's green color and
not Shard's expanded reward catalog. It makes a fresh kind copy, applies its
turn-cost setter, then requests temporary creation. A generated X Skill remains
X; ordinary unplayability is not bypassed. Exact initialized support/order and
root-seed joint membership remain CARD-RNG, not an invented all-color sampler.

Alchemize samples `returnRandomPotion(true)` at use before obtain execution;
the limited route excludes Fruit Juice. Potion initialization uses actual chosen
character, so this run does not gain Silent-only Poison/Cunning/Ghost-in-a-Jar
eligibility merely by playing a green card. Sozu is checked by ObtainPotionAction
at execution; an attempted/generated potion is not automatically a successful
obtain. Belt insertion is a real run-resource mutation at its resolution, not a
victory-only reward or a deferred simulation ledger. Detailed potion weights,
full-belt continuation, obtain hooks and interruption are the explicit potion
interface gap in §10, not supplied by this card's constructor alone.

Blade Dance, Cloak and Dagger and Infinite Blades specify ordinary Shiv templates.
Storm of Steel's BladeFuryAction first snapshots n; its one prepared template is
constructed before discard executes. Its upgraded branch constructs a Shiv, makes
a fresh kind copy and upgrades that copy; its base branch prepares an ordinary Shiv.
It then constructs/prepends MakeTempCardInHandAction with that template and n,
and only afterward constructs/prepends the DiscardAction. Thus discard executes
first, but template preparation and the temporary-action constructor's Master
Reality checkpoint have already happened. The insertion stage later creates
n stat-equivalent copies from that template, not n fresh late templates after
discard. No reaction-dependent gameplay witness is inferred from this order.

The common temporary-hand action upgrades a non-Status/non-Curse template at
action **construction** when Master Reality is present; this is a separate source
of generated upgrades, including the Storm of Steel checkpoint just described.
At update, its overflow calculation assigns available spaces up to ten to hand
and excess to discard via distinct insertion effects; overflow is not deletion,
nor a manual-discard callback. Effect completion/cancellation is CARD-ORDER.

Accuracy's constructor and stack hook rewrite Shiv instances in hand/draw/discard/
exhaust to baseDamage=4+amount or 6+amount by upgraded state. Its draw/discard
hook rewrites hand Shivs. Shiv construction also reads current Accuracy. This
is identity-specific, not +amount to every zero-cost Attack, not an ordinary
Strength modifier, and not a permanent master-deck change. Cross-writer timing
with other baseDamage mutations remains CARD-COPY/CARD-ORDER.

### Copy categories and mutation targets

Fresh `makeCopy` constructors restore the kind's initial state. The shared
stat-equivalent copy reapplies upgrades then restores baseDamage/baseBlock/
baseMagicNumber, combat/turn costs and cost-modified flags, misc, free flag and
selected metadata; it is not a live alias to the original object. Same-instance
copy additionally shares UUID. These categories must not be conflated with
permanent card acquisition or with reconstructing global history.

Endless Agony's draw callback captures a stat-equivalent copy immediately for a
top-queued temporary-creation action. Nightmare captures its selected card in
the NightmarePower constructor, then resets the template's calculated attributes
and turn cost to combat values. Reset does not itself clear the free flag. At
next start-turn it requests three creations from that template; later mutations
of the original hand card are not a live lookup. Independent Night Terror powers
do not merge templates by matching ID. Master Reality can affect temporary
creation at its own later construction checkpoint.

Glass Knife's two DamageInfo payloads are constructed before its later mutation.
ModifyDamageAction addresses matching UUID battle instances: cardInUse, draw,
discard, exhaust, limbo and hand; a set prevents double processing of the same
object. It changes each baseDamage by −2 with a zero floor, not current Strength
and not master-deck base damage. A repeat sharing UUID can therefore participate;
a separately generated new-UUID copy is not automatically the same mutation
target. Exact mixed-copy/cleanup timing remains CARD-COPY/CARD-ORDER.

Burst consumes one charge for a non-purge Skill in its on-use hook, makes a
same-instance copy, recalculates damage for a retained target when present, marks
the copy purgeOnUse, copies energyOnUse, and explicitly queues autoplayCard=true
and ignoreEnergyTotal=true. Purge prevents that repeat consuming Burst again.
Remaining Burst is removed at player end-turn. These flags are not universal
play-legality permission or an independent ordinary energy payment. Other repeat
powers' overlapping entitlements remain CARD-ORDER/CARD-COPY.

### Three X wrappers, not a generic zero-cost rule

Let C be captured energyOnUse when not −1, otherwise energy total at wrapper
resolution. Let K=2 with Chemical X, otherwise 0. The wrapper computes:

| Card | Positive effect amount | Payload |
|---|---|---|
| Skewer | C+K | That many distinct hits of captured calculated D. |
| Doppelganger | C+K+(upgraded?1:0) | Energized amount, then next-turn draw amount. |
| Malaise | C+K+(upgraded?1:0) | Negative Strength amount, then equal Weak. |

Only a positive result schedules these payloads. **Within that positive branch**,
when the wrapper's captured freeToPlayOnce is false, it consumes the then-current
total energy, not “effect amount” and not necessarily C. Chemical X's +2 and the
upgrade bonus are not additional energy charged. A free wrapper still uses its
captured/resolved effect amount. Zero-X is a legal input when ordinary legality
permits; upgraded Doppelganger/Malaise at C=K=0 still have effect 1. Setup supplies
no new free flag to X cards. Repeat captured X and queued flags must be preserved
without inventing a second ordinary payment. Full mixed-origin legality remains
CARD-COST/CARD-ORDER.

## 9. Numeric domains and rounding obligations

Printed costs, magnitudes, turn-duration counts, counters, captured X, base fields and power
amounts are **signed Java int (32-bit)** in the inspected source. Negative
sentinels, negative Strength/Dexterity and the −1 Wraith amount are meaningful;
they must not be stored as unsigned positive amounts. The source's integer
addition/multiplication occurs in that domain before any later local clamp.
Native numeric/resource exhaustion is administrative, not game death, a turn/hit
cap, silent saturation, dropped legal cards or an invented retry.

Card damage and Block calculations use **binary32** intermediates. The inspected
targeted damage path orders relic damage modification, player give powers,
stance give, target receive powers, player final-give, target final-receive,
then clamps negative results to zero and floors into the integer card payload.
The all-enemy path calculates per-monster values rather than multiplying one
total. Block applies player modifyBlock then modifyBlockLast, clamps at zero and
floors. Dexterity adds its signed amount with a zero floor (confirmed in bytecode
despite a malformed reconstructed local variable); Strength adds to NORMAL damage.
Weak gives NORMAL damage ×0.75f; ordinary Vulnerable receives NORMAL damage
×1.5f (Paper Frog against an enemy ×1.75f; Odd Mushroom on player ×1.25f).
No Silent-exclusive Paper Crane acquisition is added to this Ironclad run.
Do not floor after each power or replace binary32 with an exact-rational/float64
universal calculation. Later recipient Block/prevention/Intangible/HP hooks are
not thereby erased or treated as another identical card-calculation stage.

Local source bounds are heterogeneous:

* Poison construction caps the new power object's amount at 9999. Matching-ID
  stacking instead uses the separate application amount and the generic additive
  stack path, with **no corresponding 9999 stack clamp**. Catalyst carries raw
  P/2P as that application amount even if its new object's amount was capped.
  Do not clamp either Catalyst's accepted stack increment or total Poison to 9999.
* Strength, Dexterity and Shackled clamp constructor/stack/reduction amounts to
  [−999,999] in their inspected methods. Energized caps constructor/stack at 999.
* Masterful Stab's shared updateCost floors combat and resulting turn costs at
  zero while preserving their prior difference. setCostForTurn affects only an
  already nonnegative turn cost and floors negative requested values to zero.
* Glass Knife's mutation floors baseDamage at zero. Generic power stacking adds
  unless current amount is its nonstacking −1 sentinel; several named powers
  override this (notably Wraith Form's additive negative amount).
* Corpse Explosion uses integer maxHP×stacks before constructing its pure damage
  matrix. Catalyst+ uses integer 2P before the Poison constructor bound. They are
  not floating-point percentage damage or generic “unbounded mathematical int.”

Ordinary scalar upgrades add the listed D/B/M delta to the current base field;
they do not reconstruct an untouched printed base after earlier mutation. A
cost-changing upgrade sets the new combat cost, preserving the prior turn-minus-
combat difference only when the old turn cost was positive; a nonpositive turn
cost is not raised by that branch, and the resulting turn cost is floored at zero.
Thus the fresh paired cost is not a claim about every modified midcombat instance.

These facts do not settle native compatibility policy at source integer overflow,
all legal modifier interactions, or every recipient-side numeric writer. The
remaining exact fields are listed below rather than silently approximated.

## 10. Evidence ledger, verification and residual field map

All facts are pinned to owned archive SHA256
`cfad868ac8d65a88e71a0bf096fb09f78811e553effe0787c5309a655e081673`;
upstream build/channel identity is not established. No original game class,
initializer, reconstructed game runtime or real profile was executed/accessed.

Non-public evidence locators are identifiers, **not required document imports or
broken local hyperlinks**. All normative facts claimed here are stated above.

| Evidence group | Source anchors / grade |
|---|---|
| SI-P | Retained supporting-note-silent-review primary: cards.green constructors/use/upgrade and overrides for all 75 green classes; cards.tempCards.Shiv. Static CFR plus javap, not runtime. |
| SI-S | Same review shared: AbstractCard cost/reset/copy/damage/Block; AbstractPlayer damage/discard dispatch; GameActionManager incrementDiscard/history; named unique actions/powers in §§6–8; AbstractDungeon actual-character source pools. |
| SI-C | Same review closure: ApplyPowerAction, MakeTempCardInHandAction, PoisonLoseHpAction, ThornsPower, GetAllInBattleInstances, ExhumeAction. |
| SI-R | Frozen initial-review and correction-recheck-v1 full reports plus 13-class source/bytecode set: Setup/Forethought guards, selection order, Confusion, cost/reset/copy cleanup, Masterful Stab and Exhume. Old Silent draft remains unchanged and **not factually cleared**. |
| SI-N | This author lane's 27 targeted new classes: missing condition/count actions, ModifyDamageAction, Accuracy/delayed powers, DamageInfo, numeric powers, specific discard, MonsterGroup and AbstractPower. Exact unique ZIP entries, pinned javap/CFR and stripped Java environment; four pins before/after both batches. |
| SI-B | Retained followup-v1 execution BaneAction.ALL identity/source/bytecode and independent normative review confirmation: card-use payload capture, null/Poison gates, initial-duration positive-target source-dying cancellation and completion-time positive-target hit. Static evidence only. |

The retained initial review inspected 148 classes; its narrow recheck inspected
13, with overlap. Those are evidence counts, **not 161 distinct mechanics proofs**.
Its clearance for two corrected facts in execution PR13 at
`f00825447630032b21d29abb8938ddea8bcf8b46` is not whole-PR approval or transitive
approval of this document. This author read the actual relevant source anchors;
neither that clearance nor the old 71-row prose substitutes for those anchors.

Private checks validate the 76 distinct public IDs against retained constructors,
record source/method locators, verify class/tool-output integrity and exact archive
membership, and recheck the four pinned input/tools. Failure-first and negative
controls test the standard-library helpers; **they are not gameplay tests**.
Self-review also checks links, whitespace, sole-path scope and privacy. Product
suites were not rerun per author for this prose-only change. The controller's
recorded 124 Rust / 828 Python passes and two known CUDA-environment failures are
a qualified documentation-only baseline exception, not an all-green suite or a
waiver for new failures.

| Shared field key | Exact unclosed fields; affected local records |
|---|---|
| CARD-COST | Precedence/order of Confusion, upgrades, persistent discounts, free flags, turn reset and every copy/autoplay writer; nonordinary-origin legality/payment. Setup/Masterful/Eviscerate local hooks above are fixed, not deferred. |
| CARD-COPY | All cross-source retained fields/UUID attribution and mixed Burst/other-repeat/generated/permanent-copy compositions; exact mutation versus cleanup checkpoints. Nightmare capture, Endless Agony creation and Glass Knife UUID target family are fixed locally. |
| CARD-HISTORY | Nested draw-history ownership/replacement across interposed draws; full play-origin/history insertion/suppression matrix and turn-end reset ordering. Escape Plan/Finisher may not be replaced by guessed global summaries. |
| CARD-ORDER | Complete shared callback priority/tie/order and suspension/cancellation laws across pre-turn, power/relic, effect/soul insertion and end-turn retain/Ethereal/Establishment paths. Source-specific append/prepend facts do not prove universal FIFO or drain. |
| CARD-DEATH | Prevention/rescue, recipient damage/death, half-dead/split, chained Explosion, last-enemy action filtering, terminal interruption and durable effects for every origin. No fixed nested-death total or universal rollback claim. |
| CARD-RNG | Fully initialized per-generator supports/order, bounded-index conditional/joint laws, shared stream evolution and initialized-root-seed membership with reaction-dependent targets. Distraction/Flask filters and sampling checkpoints above are fixed; no universal IID/pool rule is substituted. |
| CARD-NUMERIC | Every recipient-side modifier/cap and native policy beyond source 32-bit range, not a generic integer promise. The explicit local clamps and binary32/floor domain above remain required. |
| POTION-INTERFACE | Complete Ironclad limited-potion weights/support, belt/full-capacity obtain continuation, hook ordering and terminal durability. Alchemize's actual-character authority, sampling-before-obtain and Sozu checkpoint are fixed. |

These gaps remain within full Act 1 scope. They are not permission to use
placeholders, drop legally reachable records or begin product implementation.
For common execution context see the already published
[bounded execution research](2026-10-18-act1-execution-closure.md); it too explicitly
leaves transitive scheduling open. Whole-spec approval, plan review and start
authorization remain separate human gates.

## 11. Discriminating future acceptance cases (not executed)

| Case | Required distinction |
|---|---|
| S01 paired records | All 4/19/33/19 green kinds and Shiv; exact source IDs, target fields, sentinels, scalar upgrades, E/I transitions and tags. |
| S02 Setup layers | Positive combat/zero turn; zero combat/positive turn; X; unplayable; preexisting free flag; Confusion clearing; reset versus use cleanup. |
| S03 inactive history | Masterful Stab in hand versus exhaust during positive loss; Exhume same identity without catch-up; one five-HP loss versus five one-HP losses; fresh versus stat-copy costs. |
| S04 ordered movement | Forethought+ same subset in opposite orders and empty selection; no sorting or repeated parent payment. |
| S05 discard sources | Selected/random/all and Unload-specific callback/counter order; Reflex/Tactician; Concentrate undersized/empty; no recursive reaction-drawn discard sweep. |
| S06 draw provenance | Escape Plan No Draw/full hand/empty piles and nested draws; Expertise one captured deficit; Endless Agony creation is not another draw. Nested history needs CARD-HISTORY closure before exact totals. |
| S07 count snapshots | Finisher counts attacks minus current entry, not hits/all plays; Flechettes counts unplayable Skills at action time; Eviscerate per-pile notifications/on-draw reconciliation/source-copy side effect. |
| S08 separate poison | Flask's next target sampled before current application; independent Artifact consumption; Catalyst no-Poison and constructor-cap-versus-stack cases; player Fumes versus afflicted tick clocks. |
| S09 distinct death effects | Corpse Poison prevented/removed independently from explosion; maxHP at callback versus currentHP; no fixed chain/last-enemy total before CARD-DEATH closure. |
| S10 delayed state | Dodge and Roll calculated capture; Tools stacked draw/discard; Nightmare template reset and independent captures; Phantasmal duration versus multiplier; Intangible expiry versus Wraith downside. |
| S11 identity/mutation | Ordinary Shiv versus Shiv+ versus Master Reality; Accuracy pile rewrites; Glass Knife shared UUID versus new UUID; no permanent deck mutation. |
| S12 X and repeat | Captured C differs from current energy; Chemical X; zero-X upgraded bonuses; free flag; Burst copy queue booleans/purge and no invented repeat payment. |
| S13 character authority | Ironclad Distraction support and Alchemize limited potion pool despite green producer; Sozu after sampling; real belt mutation, not victory reconciliation. |
| S14 numeric damage | Binary32 modifier chain/final floor, separate Dagger Spray/Riddle/Skewer hits, blocked Caltrops retaliation, positive NORMAL Envenom versus THORNS/HP_LOSS. |
| S15 no-child versus zero-child | Start with nonempty shared draw history: empty-hand Calculated Gamble (both versions) and Expertise at h=M or h>M enqueue no draw and perform no child-history clear; contrast an actually enqueued default-history-clearing DrawCardAction(0), whose initial update clears history. Expertise at h<M enqueues one positive-deficit draw. No specific nested-history gameplay outcome is asserted. |
| S16 Bane local checkpoints | Capture target and both DamageInfo payloads before either hit; distinguish null target, absent Poison during update, completion-time nonpositive target HP, and the initial-duration positive-target branch's non-THORNS/owner-isDying cancellation. Do not promote that checkpoint to every update, generic origins or universal legality; recipient/rescue/terminal outcomes remain unproved. |
| S17 helper/template ordering | Heel Hook without Weak still enqueues damage; entry-time Weak gates only energy then draw behind it, without a helper positive-HP test or post-hit retest. Storm of Steel prepares one template and runs its temporary-action construction/Master Reality checkpoint before discard executes, then later inserts stat-equivalent copies. No reaction-dependent gameplay witness is claimed. |

Exact assertions involving the listed shared gaps require their independent
closure first. This case list is neither a test run nor a proof of legal root-seed
reachability for each synthetic state.
