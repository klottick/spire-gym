# M15 named source qualifications

These are independently worded, bounded static findings for the approved
[M15 plan](../superpowers/plans/2026-10-26-m15-silent-heart.md), extending the
[Silent study](2026-10-19-act1-content-silent.md),
[red/colorless study](2026-10-19-act1-content-red-colorless.md) and
[potion study](2026-10-19-act1-content-economy-potions.md). They are neither
original-runtime validation nor proof of native implementation. Raw evidence,
input manifests and attributed corrections are retained privately outside
managed sessions.

The inspected archive SHA-256 is
`cfad868ac8d65a88e71a0bf096fb09f78811e553effe0787c5309a655e081673`;
the provisioned `javap` is21.0.12.1. The upstream build/channel is unverified.
No game/class execution, broad archive extraction or installation occurred.

## G1, G2 and G5: complete actual-character domains

`TheSilent.getCardPool` → `CardLibrary.addGreenCards` →
`AbstractDungeon.initializeCardPools` supplies nonbasic green source groups.
The named green constructors confirm the ledger's complete26 Attack,33
nonhealing Skill and11 Power sets and their9/13/4,10/14/9,0/6/5 rarity splits.
Alchemize is the HEALING-tagged Rare Skill; unplayable Reflex/Tactician remain
eligible. Untyped nonhealing support is their70-kind union. Distraction uses
the full33 Skill domain, including itself. The helper concatenates rarity
groups and chooses an identity without a rarity lottery. Native group/ID
ordering remains the approved deterministic ordering, without Java collection
or cursor parity.

`PotionHelper.initialize/getPotions(SILENT,false)` and all relevant constructor
rarities confirm33 identities:30 shared plus Poison/Cunning/Ghost, at17/9/7.
Rarity-first sampling uses65/25/10, then uniform eligible identity within the
selected rarity. Limited generation excludes only FruitJuice, giving17/9/6;
outside Brew permits it. Proposal construction and inventory obtain remain
distinct. No sampler may substitute the currently implemented subset.

Typed potion callers pass their nonnull type into `DiscoveryAction(CardType,
amount)`. Its screen flag is `cardType != null`, so typed offers permit skip;
untyped Discovery and Colorless use their separate mandatory path. Three
distinct offers preserve accepted order; selection creates one/two independent
stat-equivalent copies with the guarded turn-cost-zero request. An initial
private report misread the conditional; root and senior independently checked
the retained branch, constructor and callers. The original report and its
attributed correction are preserved separately. The approved typed-skip rule
is unchanged.

## S1–S3: costs, copies and history

Mutable combat cost, turn cost and a one-play-free flag are distinct. Their
named update/reset/upgrade writers retain the source-specific guards. Masterful
Stab receives one callback per positive player-loss event only in hand, discard
and draw. Eviscerate's copy constructor mutates the live source's eligible turn
cost before a stat-equivalent copy captures fields. This needs a state-aware,
card-specific pre-copy seam; the generic copy helper remains independent.

Same-family copies remain distinct objects with shared mutation-family identity.
GlassKnife captures both hits before its family damage mutation. Nightmare
captures and resets a template; EndlessAgony creates a fresh-family child on
actual draw. Creation, retrieval and draw are different operations.

New player-turn setup clears play/discard counters. Play history includes the
current card before its use work. Ordinary manual discard calls the card hook
before increment/notifications; Unload's specific-card discard increments and
notifies first. Both keep a captured ownership set rather than recursively
discarding reaction-drawn cards.

Each new root draw clears shared draw history even for zero, NoDraw, full-hand
or empty-pile exits. Internal draw/recycle pieces preserve that history. An
absent Expertise/Gamble child performs no clear. Evolve appends a new clearing
draw; the root draw's EscapePlan follow-up runs first. Thus a one-card Status
draw does not gain Block from Evolve's later Skill, while a two-card root draw
of Status then Skill does, including its shortage/reshuffle composition. One
shared resolver can represent these checkpoints without animation timing.

## S4–S6: callbacks, death and items

On-use callbacks capture existing player powers, relics and monster powers
after the card body enqueues work. AfterImage and Choke enqueue their work
before the later use cleanup. ThousandCuts captures at the distinct
`onAfterCardPlayed` checkpoint and enqueues damage after use cleanup; Beat of
Death's later after-use callback appends behind that pending damage. A newly
queued Power installation does not receive its own earlier callback. Tools
uses the post-draw checkpoint, appending its draw before its selected discard.

Death visits monster powers before player relics. CorpseExplosion captures
callback maxHP times stacks; Specimen captures current Poison afterward but
chooses its recipient later. Area-damage last-enemy cleanup can remove the
pending transfer while preserving damage/heal/Block/use actions. Awakened
One's first-phase clear cancels pending card-queue items, including an
undispatched Burst repeat; it does not erase already queued action effects.
These named classes need the shared native lifecycle owner, rather than a
universal drain or cancellation rule.

Skull adjusts a player-origin Poison proposal before source-power application
callbacks; Artifact later prevents a debuff without changing the final stack.
The accepted prior Artifact/Skull final-stack result remains intact. New
application reactions must use the qualified callback checkpoint; the old
result ruling is not a universal ordering theorem.

The named relic constructors confirm starter Snake; Common Skull; Uncommon
Scroll/Krane; Rare Specimen/Tingsha/Bandages; Shop Funnel; Boss Kite/Serpent/
WristBlade. Serpent requires and replaces Snake, increasing master draw by one
while removing Snake's opening bonus. Scroll queues three ordinary Shivs at
pre-opening-draw startup. Kite resets each turn; Tingsha chooses a live random
recipient at its damage-action checkpoint; cleanup is not manual discard.
WristBlade's local predicate is zero turn cost, or a free flag with non-X
combat cost. PaperKrane modifies weakened enemy NORMAL damage by0.6.

Poison/Cunning/Ghost constructor potencies are6/3/1, doubled by Bark at potion
construction. Cunning supplies upgraded Shivs and retains overflow in discard;
Ghost's player Intangible uses the named end-round expiry. Player Intangible
caps NORMAL, THORNS and the named direct/Poison HP-loss routes at one before
Block/HP commit; this does not generalize to unrelated HP-loss producers.

## G3/G4 and S7/S8: shared boundaries

The qualified35 ordinary colorless identities minus HEALING BandageUp give34
eligible kinds. Their recursive generators use actual typed/untyped Silent
supports or that complete colorless support. Temporary creation keeps its
source-specific copy, upgrade and cost checkpoints. HandOfGreed uses qualified
Fatal guards and the accepted earned-gold/settlement owner.

Ordinary potion insertion scans to the first empty slot and can fail at full
capacity without a replacement prompt. Consumer-specific Sozu checks, sampling,
manual-use/slot-vacate hooks and accepted fault custody remain distinct. The
approved native error mappings do not claim that the original has Rust types.

New mechanics reuse the accepted signed/binary32 damage and Block owners and
source-specific clamps. Native arithmetic/resource exhaustion is an explicit
operational fault, never an invented gameplay cap or silent saturation.
Per-act event/shrine lists, shared special lists and named selection/dispatch
guards contain no Silent-only event gate. This rules out an invented exclusive
event requirement; full shared event generation remains outside this captured
profile, and actual green eligibility still needs native event tests.

No source fact enables a handler, changes a frozen rules identity or satisfies
normal-start traversal. The coverage ledger retains all runtime obligations.


## Recovery addendum: named Colorless and Smoke callbacks

Additional retained static lookups resolve The Bomb at the player pre-card
end-turn power pass; Panache on-use damage follows the card's queued payload and
precedes after-use; Sadistic Nature's actual `SadisticPower` observes source
applications before the target Artifact guard, independently checks Artifact,
excludes Shackled and appends its damage. These are named callback facts, not a
universal queue-order theorem.

Smoke's ordinary gate also scans every retained monster for Back Attack or boss
type, without filtering that scan by life state. The source escape timer calls
battle end independently of draining queued work. Battle end invokes the
player's onVictory callbacks, including Slaver's Collar cleanup, then clears the
action manager. The smoke reward screen bypasses reward setup and presents
Proceed; internal drop rows can still have been constructed. Committed theft
has no automatic escape refund.

The native M15 implementation represents escape as its own ending at an explicit
resolver checkpoint. It runs the shared battle-end callbacks and clears pending
work at that checkpoint. It does not model the source's frame clock or certify
whether queued healing would have finished before the source timer. The native
ending is distinct from enemy defeat and from a Run victory or key claim. Run
settlement and final profile evidence remain separate implementation obligations.

The supplied archive's upstream channel remains unverified. Private method dumps
and hash manifests are retained outside managed sessions; no original game or
class execution, Python artifact delivery, or wider runtime fidelity is claimed.


### A0 Heart HP implementation-fact correction (2026-10-28)

The approved Task9 test prose names Heart800. The retained named CorruptHeart constructor sets HP750 below ascension9 and HP800 at ascension9 or above. M15 is A0 and reuses the accepted shared Heart750/Invincible300 owner; the new M15 assertion uses750. The original failed800 assertion/log and manager disposition are independently retained. Approved design/plan bytes and accepted M10 tests/setup remain unchanged. This is a static-source qualification with upstream build/channel unverified, not original-runtime evidence or a new character-specific Heart rule.
