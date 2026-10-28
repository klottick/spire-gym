# M15 Silent A0 coverage ledger

Planning inventory at accepted M14 `a8261aa55e64d9c013f418b37711060003ca50b4`.
The historical gap classifications are retained alongside current native
implementation and review evidence. Read with the
[design](../superpowers/specs/2026-10-26-m15-silent-heart-design.md) and
[implementation plan](../superpowers/plans/2026-10-26-m15-silent-heart.md).

## Current implementation status (2026-10-28, native review passed)

T1–T9 have native implementation and focused evidence. All75 Green kinds have
real shared handlers/constructors, paired base/+ dispatch (Reflex/Tactician retain
ordinary unplayability), permanent obtain/upgrade/removal and full M15 admission.
All11 exclusive relics, three exclusive potions and the complete transitive
33-potion/L34 Colorless generator dependencies have executable shared owners.
The retained T9 core checkpoint passes194 tests. Its supplied key/Act4 diagnostics
are mechanics evidence, not normal-start evidence or milestone acceptance.

T10 versioned native publication/session/replay/batch and the T11 caller are
integrated with focused native25+unit4 and example85 checks. Pre-outcome source
freeze and all six original normal attempts are complete. All six ended in Death;
four earned all three keys, none reached Act3/Act4/Heart. Explicit native parity
passed all3143 original actions against both the original frozen source and the
final runtime with its qualified preservation repairs. The stable workspace
matrix passes2357 checks, zero failures, one default-ignored check in68 blocks.
The ignored consumer's explicit executions are separately recorded. These
workspace and full-history results are pre-correction evidence. One independent
senior completed the full review, then cleared all five Important and two Minor
findings after the bounded correction pass. The corrected source passes 294
focused checks and 15 existing native session/replay/batch integration tests.
The [review record](../superpowers/plans/2026-10-28-m15-native-runtime-review.md)
separates those revisions and records the current runtime-first direction.
Canonical normal completion remains unproven; the separately supplied Act4 win
is not normal-prefix proof. Main integration and Python delivery remain separate.
Historical checkpoint notes retain their as-of status; baseline classifications
below remain unchanged; each current card row now links
to its actual behavior checks. Qualified static facts are in the
[source qualification record](../research/2026-10-27-m15-source-qualifications.md).

## Grades, sources and counts

- **I**: actual native behavior at the accepted finite profile boundary, with
  regression anchors; wider compositions still need the listed work.
- **U**: native behavior exists but the finite Silent profile does not admit or
  publish it. A native registry entry alone does not earn I or U.
- **M**: missing runtime behavior, not merely missing from a producer; implementation
  is required. **E** names an exact unresolved evidence question. Evidence gaps
  and missing runtime behavior are separate columns, not a generic exemption.
- No row here claims original-runtime validation or current Python delivery.
  Public datasets are reference inputs with their existing grades.

The [card catalog](../../data/cards/README.md) has75 Silent rows: 4 Basic,
19 Common,33 Uncommon,19 Rare, all base/+; 71 nonbasic ordinary-pool candidates.
The runtime has19 green kinds (four starters +15 nonbasic), all admitted by M14;
M14's permanent universe is those19 plus Parasite. Its public universe additionally
contains Shiv/Dazed/Burn/Slimed/Wound/Void (26 distinct IDs total).
**56 ordinary green behaviors are missing**, with no named enum/registry/handler.
No ordinary green card is merely implemented-but-unadmitted at this baseline.

Each card row below references its existing `data/cards/cards.json` **id** for
paired base/+ values, flags, citations and uncertainties; this ledger does not
copy the dataset. The game ID is the catalog's attributed string, not a Rust ID.
For an M row, the native name shown is a **proposal**, with `as_str` the snake-case
of that proposed variant. Existing names/tokens retain their actual spellings
(e.g. StrikeGreen=`strike_g`). T is the plan task owning missing behavior.
Every row needs base/+ actual-play, upgrade/removal, visible-value, copy and
relevant interaction coverage before character closure.

Source owners: [IDs](../../crates/spire-core/src/content/ids/cards.rs),
[registry](../../crates/spire-core/src/content/cards.rs),
[dispatcher](../../crates/spire-core/src/combat/cards/mod.rs),
[Silent handlers](../../crates/spire-core/src/combat/cards/silent.rs),
[M14 admission](../../crates/spire-core/src/run/rooms/m14_content.rs),
[mutation owner](../../crates/spire-core/src/run/mutations.rs),
[M14 publication](../../crates/spire-env/src/run_schema_m14.rs).
Paired source details: [Silent study §§2–9](../research/2026-10-19-act1-content-silent.md).
Its source facts are static/qualified and its §10 composition gaps remain explicit.

## Every ordinary green kind and both variants

| Reference ID | Catalog game ID | Rarity/type | Native ID (proposed if M) | Baseline behavior / M14 admission/publication | M15 work |
|---|---|---|---|---|---|
| `DEFEND_GREEN` | `Defend_G` | Basic/Skill | `DefendGreen` | I finite / yes / yes | [Runtime checks](../../crates/spire-core/src/combat/tests/m12_silent_values.rs); [T9 obtain/upgrade/remove](../../crates/spire-core/src/run/tests/m15_silent_heart.rs); T10 linked; normal/final review pending |
| `NEUTRALIZE` | `Neutralize` | Basic/Attack | `Neutralize` | I finite / yes / yes | [Runtime checks](../../crates/spire-core/src/combat/tests/m12_silent_values.rs); [T9 obtain/upgrade/remove](../../crates/spire-core/src/run/tests/m15_silent_heart.rs); T10 linked; normal/final review pending |
| `STRIKE_GREEN` | `Strike_G` | Basic/Attack | `StrikeGreen` | I finite / yes / yes | [Runtime checks](../../crates/spire-core/src/combat/tests/m12_silent_values.rs); [T9 obtain/upgrade/remove](../../crates/spire-core/src/run/tests/m15_silent_heart.rs); T10 linked; normal/final review pending |
| `SURVIVOR` | `Survivor` | Basic/Skill | `Survivor` | I finite / yes / yes | [Runtime checks](../../crates/spire-core/src/combat/tests/m12_silent_values.rs); [T9 obtain/upgrade/remove](../../crates/spire-core/src/run/tests/m15_silent_heart.rs); T10 linked; normal/final review pending |
| `ACROBATICS` | `Acrobatics` | Common/Skill | `Acrobatics` | M / no / no | [Runtime checks](../../crates/spire-core/src/combat/tests/m15_silent_history.rs); [T9 obtain/upgrade/remove](../../crates/spire-core/src/run/tests/m15_silent_heart.rs); T10 linked; normal/final review pending |
| `BACKFLIP` | `Backflip` | Common/Skill | `Backflip` | I finite / yes / yes | [Runtime checks](../../crates/spire-core/src/combat/tests/m12_silent_values.rs); [T9 obtain/upgrade/remove](../../crates/spire-core/src/run/tests/m15_silent_heart.rs); T10 linked; normal/final review pending |
| `BANE` | `Bane` | Common/Attack | `Bane` | M / no / no | [Runtime checks](../../crates/spire-core/src/combat/tests/m15_silent_poison_death.rs); [T9 obtain/upgrade/remove](../../crates/spire-core/src/run/tests/m15_silent_heart.rs); T10 linked; normal/final review pending |
| `BLADE_DANCE` | `Blade Dance` | Common/Skill | `BladeDance` | I finite / yes / yes | [Runtime checks](../../crates/spire-core/src/combat/tests/m12_silent_values.rs); [T9 obtain/upgrade/remove](../../crates/spire-core/src/run/tests/m15_silent_heart.rs); T10 linked; normal/final review pending |
| `CLOAK_AND_DAGGER` | `Cloak And Dagger` | Common/Skill | `CloakAndDagger` | I finite / yes / yes | [Runtime checks](../../crates/spire-core/src/combat/tests/m12_silent_values.rs); [T9 obtain/upgrade/remove](../../crates/spire-core/src/run/tests/m15_silent_heart.rs); T10 linked; normal/final review pending |
| `DAGGER_SPRAY` | `Dagger Spray` | Common/Attack | `DaggerSpray` | M / no / no | [Runtime checks](../../crates/spire-core/src/combat/tests/m15_silent_direct.rs); [T9 obtain/upgrade/remove](../../crates/spire-core/src/run/tests/m15_silent_heart.rs); T10 linked; normal/final review pending |
| `DAGGER_THROW` | `Dagger Throw` | Common/Attack | `DaggerThrow` | I finite / yes / yes | [Runtime checks](../../crates/spire-core/src/combat/tests/m12_silent_values.rs); [T9 obtain/upgrade/remove](../../crates/spire-core/src/run/tests/m15_silent_heart.rs); T10 linked; normal/final review pending |
| `DEADLY_POISON` | `Deadly Poison` | Common/Skill | `DeadlyPoison` | I finite / yes / yes | [Runtime checks](../../crates/spire-core/src/combat/tests/m12_silent_values.rs); [T9 obtain/upgrade/remove](../../crates/spire-core/src/run/tests/m15_silent_heart.rs); T10 linked; normal/final review pending |
| `DEFLECT` | `Deflect` | Common/Skill | `Deflect` | I finite / yes / yes | [Runtime checks](../../crates/spire-core/src/combat/tests/m12_silent_values.rs); [T9 obtain/upgrade/remove](../../crates/spire-core/src/run/tests/m15_silent_heart.rs); T10 linked; normal/final review pending |
| `DODGE_AND_ROLL` | `Dodge and Roll` | Common/Skill | `DodgeAndRoll` | M / no / no | [Runtime checks](../../crates/spire-core/src/combat/tests/m15_silent_powers.rs); [T9 obtain/upgrade/remove](../../crates/spire-core/src/run/tests/m15_silent_heart.rs); T10 linked; normal/final review pending |
| `FLYING_KNEE` | `Flying Knee` | Common/Attack | `FlyingKnee` | M / no / no | [Runtime checks](../../crates/spire-core/src/combat/tests/m15_silent_powers.rs); [T9 obtain/upgrade/remove](../../crates/spire-core/src/run/tests/m15_silent_heart.rs); T10 linked; normal/final review pending |
| `OUTMANEUVER` | `Outmaneuver` | Common/Skill | `Outmaneuver` | M / no / no | [Runtime checks](../../crates/spire-core/src/combat/tests/m15_silent_powers.rs); [T9 obtain/upgrade/remove](../../crates/spire-core/src/run/tests/m15_silent_heart.rs); T10 linked; normal/final review pending |
| `PIERCING_WAIL` | `PiercingWail` | Common/Skill | `PiercingWail` | M / no / no | [Runtime checks](../../crates/spire-core/src/combat/tests/m15_silent_poison_death.rs); [T9 obtain/upgrade/remove](../../crates/spire-core/src/run/tests/m15_silent_heart.rs); T10 linked; normal/final review pending |
| `POISONED_STAB` | `Poisoned Stab` | Common/Attack | `PoisonedStab` | I finite / yes / yes | [Runtime checks](../../crates/spire-core/src/combat/tests/m12_silent_values.rs); [T9 obtain/upgrade/remove](../../crates/spire-core/src/run/tests/m15_silent_heart.rs); T10 linked; normal/final review pending |
| `PREPARED` | `Prepared` | Common/Skill | `Prepared` | M / no / no | [Runtime checks](../../crates/spire-core/src/combat/tests/m15_silent_history.rs); [T9 obtain/upgrade/remove](../../crates/spire-core/src/run/tests/m15_silent_heart.rs); T10 linked; normal/final review pending |
| `QUICK_SLASH` | `Quick Slash` | Common/Attack | `QuickSlash` | I finite / yes / yes | [Runtime checks](../../crates/spire-core/src/combat/tests/m12_silent_values.rs); [T9 obtain/upgrade/remove](../../crates/spire-core/src/run/tests/m15_silent_heart.rs); T10 linked; normal/final review pending |
| `SLICE` | `Slice` | Common/Attack | `Slice` | M / no / no | [Runtime checks](../../crates/spire-core/src/combat/tests/m15_silent_direct.rs); [T9 obtain/upgrade/remove](../../crates/spire-core/src/run/tests/m15_silent_heart.rs); T10 linked; normal/final review pending |
| `SNEAKY_STRIKE` | `Underhanded Strike` | Common/Attack | `SneakyStrike` | M / no / no | [Runtime checks](../../crates/spire-core/src/combat/tests/m15_silent_history.rs); [T9 obtain/upgrade/remove](../../crates/spire-core/src/run/tests/m15_silent_heart.rs); T10 linked; normal/final review pending |
| `SUCKER_PUNCH` | `Sucker Punch` | Common/Attack | `SuckerPunch` | M / no / no | [Runtime checks](../../crates/spire-core/src/combat/tests/m15_silent_direct.rs); [T9 obtain/upgrade/remove](../../crates/spire-core/src/run/tests/m15_silent_heart.rs); T10 linked; normal/final review pending |
| `ACCURACY` | `Accuracy` | Uncommon/Power | `Accuracy` | I finite / yes / yes | [Runtime checks](../../crates/spire-core/src/combat/tests/m15_silent_cost_copy.rs); [T9 obtain/upgrade/remove](../../crates/spire-core/src/run/tests/m15_silent_heart.rs); T10 linked; normal/final review pending |
| `ALL_OUT_ATTACK` | `All Out Attack` | Uncommon/Attack | `AllOutAttack` | M / no / no | [Runtime checks](../../crates/spire-core/src/combat/tests/m15_silent_history.rs); [T9 obtain/upgrade/remove](../../crates/spire-core/src/run/tests/m15_silent_heart.rs); T10 linked; normal/final review pending |
| `BACKSTAB` | `Backstab` | Uncommon/Attack | `Backstab` | M / no / no | [Runtime checks](../../crates/spire-core/src/combat/tests/m15_silent_direct.rs); [T9 obtain/upgrade/remove](../../crates/spire-core/src/run/tests/m15_silent_heart.rs); T10 linked; normal/final review pending |
| `BLUR` | `Blur` | Uncommon/Skill | `Blur` | M / no / no | [Runtime checks](../../crates/spire-core/src/combat/tests/m15_silent_powers.rs); [T9 obtain/upgrade/remove](../../crates/spire-core/src/run/tests/m15_silent_heart.rs); T10 linked; normal/final review pending |
| `BOUNCING_FLASK` | `Bouncing Flask` | Uncommon/Skill | `BouncingFlask` | M / no / no | [Runtime checks](../../crates/spire-core/src/combat/tests/m15_silent_poison_death.rs); [T9 obtain/upgrade/remove](../../crates/spire-core/src/run/tests/m15_silent_heart.rs); T10 linked; normal/final review pending |
| `CALCULATED_GAMBLE` | `Calculated Gamble` | Uncommon/Skill | `CalculatedGamble` | M / no / no | [Runtime checks](../../crates/spire-core/src/combat/tests/m15_silent_history.rs); [T9 obtain/upgrade/remove](../../crates/spire-core/src/run/tests/m15_silent_heart.rs); T10 linked; normal/final review pending |
| `CALTROPS` | `Caltrops` | Uncommon/Power | `Caltrops` | M / no / no | [Runtime checks](../../crates/spire-core/src/combat/tests/m15_silent_powers.rs); [T9 obtain/upgrade/remove](../../crates/spire-core/src/run/tests/m15_silent_heart.rs); T10 linked; normal/final review pending |
| `CATALYST` | `Catalyst` | Uncommon/Skill | `Catalyst` | I finite / yes / yes | [Runtime checks](../../crates/spire-core/src/combat/tests/m12_silent_values.rs); [T9 obtain/upgrade/remove](../../crates/spire-core/src/run/tests/m15_silent_heart.rs); T10 linked; normal/final review pending |
| `CHOKE` | `Choke` | Uncommon/Attack | `Choke` | M / no / no | [Runtime checks](../../crates/spire-core/src/combat/tests/m15_silent_powers.rs); [T9 obtain/upgrade/remove](../../crates/spire-core/src/run/tests/m15_silent_heart.rs); T10 linked; normal/final review pending |
| `CONCENTRATE` | `Concentrate` | Uncommon/Skill | `Concentrate` | M / no / no | [Runtime checks](../../crates/spire-core/src/combat/tests/m15_silent_history.rs); [T9 obtain/upgrade/remove](../../crates/spire-core/src/run/tests/m15_silent_heart.rs); T10 linked; normal/final review pending |
| `CRIPPLING_CLOUD` | `Crippling Poison` | Uncommon/Skill | `CripplingCloud` | M / no / no | [Runtime checks](../../crates/spire-core/src/combat/tests/m15_silent_poison_death.rs); [T9 obtain/upgrade/remove](../../crates/spire-core/src/run/tests/m15_silent_heart.rs); T10 linked; normal/final review pending |
| `DASH` | `Dash` | Uncommon/Attack | `Dash` | M / no / no | [Runtime checks](../../crates/spire-core/src/combat/tests/m15_silent_direct.rs); [T9 obtain/upgrade/remove](../../crates/spire-core/src/run/tests/m15_silent_heart.rs); T10 linked; normal/final review pending |
| `DISTRACTION` | `Distraction` | Uncommon/Skill | `Distraction` | M / no / no | [Runtime checks](../../crates/spire-core/src/combat/tests/m15_generation.rs); [T9 obtain/upgrade/remove](../../crates/spire-core/src/run/tests/m15_silent_heart.rs); T10 linked; normal/final review pending |
| `ENDLESS_AGONY` | `Endless Agony` | Uncommon/Attack | `EndlessAgony` | M / no / no | [Runtime checks](../../crates/spire-core/src/combat/tests/m15_silent_cost_copy.rs); [T9 obtain/upgrade/remove](../../crates/spire-core/src/run/tests/m15_silent_heart.rs); T10 linked; normal/final review pending |
| `ESCAPE_PLAN` | `Escape Plan` | Uncommon/Skill | `EscapePlan` | M / no / no | [Runtime checks](../../crates/spire-core/src/combat/tests/m15_silent_history.rs); [T9 obtain/upgrade/remove](../../crates/spire-core/src/run/tests/m15_silent_heart.rs); T10 linked; normal/final review pending |
| `EVISCERATE` | `Eviscerate` | Uncommon/Attack | `Eviscerate` | M / no / no | [Runtime checks](../../crates/spire-core/src/combat/tests/m15_silent_cost_copy.rs); [T9 obtain/upgrade/remove](../../crates/spire-core/src/run/tests/m15_silent_heart.rs); T10 linked; normal/final review pending |
| `EXPERTISE` | `Expertise` | Uncommon/Skill | `Expertise` | M / no / no | [Runtime checks](../../crates/spire-core/src/combat/tests/m15_silent_history.rs); [T9 obtain/upgrade/remove](../../crates/spire-core/src/run/tests/m15_silent_heart.rs); T10 linked; normal/final review pending |
| `FINISHER` | `Finisher` | Uncommon/Attack | `Finisher` | M / no / no | [Runtime checks](../../crates/spire-core/src/combat/tests/m15_silent_history.rs); [T9 obtain/upgrade/remove](../../crates/spire-core/src/run/tests/m15_silent_heart.rs); T10 linked; normal/final review pending |
| `FLECHETTES` | `Flechettes` | Uncommon/Attack | `Flechettes` | M / no / no | [Runtime checks](../../crates/spire-core/src/combat/tests/m15_silent_history.rs); [T9 obtain/upgrade/remove](../../crates/spire-core/src/run/tests/m15_silent_heart.rs); T10 linked; normal/final review pending |
| `FOOTWORK` | `Footwork` | Uncommon/Power | `Footwork` | M / no / no | [Runtime checks](../../crates/spire-core/src/combat/tests/m15_silent_powers.rs); [T9 obtain/upgrade/remove](../../crates/spire-core/src/run/tests/m15_silent_heart.rs); T10 linked; normal/final review pending |
| `HEEL_HOOK` | `Heel Hook` | Uncommon/Attack | `HeelHook` | M / no / no | [Runtime checks](../../crates/spire-core/src/combat/tests/m15_silent_poison_death.rs); [T9 obtain/upgrade/remove](../../crates/spire-core/src/run/tests/m15_silent_heart.rs); T10 linked; normal/final review pending |
| `INFINITE_BLADES` | `Infinite Blades` | Uncommon/Power | `InfiniteBlades` | M / no / no | [Runtime checks](../../crates/spire-core/src/combat/tests/m15_silent_powers.rs); [T9 obtain/upgrade/remove](../../crates/spire-core/src/run/tests/m15_silent_heart.rs); T10 linked; normal/final review pending |
| `LEG_SWEEP` | `Leg Sweep` | Uncommon/Skill | `LegSweep` | M / no / no | [Runtime checks](../../crates/spire-core/src/combat/tests/m15_silent_direct.rs); [T9 obtain/upgrade/remove](../../crates/spire-core/src/run/tests/m15_silent_heart.rs); T10 linked; normal/final review pending |
| `MASTERFUL_STAB` | `Masterful Stab` | Uncommon/Attack | `MasterfulStab` | M / no / no | [Runtime checks](../../crates/spire-core/src/combat/tests/m15_silent_cost_copy.rs); [T9 obtain/upgrade/remove](../../crates/spire-core/src/run/tests/m15_silent_heart.rs); T10 linked; normal/final review pending |
| `NOXIOUS_FUMES` | `Noxious Fumes` | Uncommon/Power | `NoxiousFumes` | I finite / yes / yes | [Runtime checks](../../crates/spire-core/src/combat/tests/m12_silent_values.rs); [T9 obtain/upgrade/remove](../../crates/spire-core/src/run/tests/m15_silent_heart.rs); T10 linked; normal/final review pending |
| `PREDATOR` | `Predator` | Uncommon/Attack | `Predator` | M / no / no | [Runtime checks](../../crates/spire-core/src/combat/tests/m15_silent_powers.rs); [T9 obtain/upgrade/remove](../../crates/spire-core/src/run/tests/m15_silent_heart.rs); T10 linked; normal/final review pending |
| `REFLEX` | `Reflex` | Uncommon/Skill | `Reflex` | I finite / yes / yes | [Runtime checks](../../crates/spire-core/src/combat/tests/m15_silent_history.rs); [T9 obtain/upgrade/remove](../../crates/spire-core/src/run/tests/m15_silent_heart.rs); T10 linked; normal/final review pending |
| `RIDDLE_WITH_HOLES` | `Riddle With Holes` | Uncommon/Attack | `RiddleWithHoles` | M / no / no | [Runtime checks](../../crates/spire-core/src/combat/tests/m15_silent_direct.rs); [T9 obtain/upgrade/remove](../../crates/spire-core/src/run/tests/m15_silent_heart.rs); T10 linked; normal/final review pending |
| `SETUP` | `Setup` | Uncommon/Skill | `Setup` | M / no / no | [Runtime checks](../../crates/spire-core/src/combat/tests/m15_silent_cost_copy.rs); [T9 obtain/upgrade/remove](../../crates/spire-core/src/run/tests/m15_silent_heart.rs); T10 linked; normal/final review pending |
| `SKEWER` | `Skewer` | Uncommon/Attack | `Skewer` | M / no / no | [Runtime checks](../../crates/spire-core/src/combat/tests/m15_silent_x_repeat.rs); [T9 obtain/upgrade/remove](../../crates/spire-core/src/run/tests/m15_silent_heart.rs); T10 linked; normal/final review pending |
| `TACTICIAN` | `Tactician` | Uncommon/Skill | `Tactician` | I finite / yes / yes | [Runtime checks](../../crates/spire-core/src/combat/tests/m15_silent_history.rs); [T9 obtain/upgrade/remove](../../crates/spire-core/src/run/tests/m15_silent_heart.rs); T10 linked; normal/final review pending |
| `TERROR` | `Terror` | Uncommon/Skill | `Terror` | M / no / no | [Runtime checks](../../crates/spire-core/src/combat/tests/m15_silent_poison_death.rs); [T9 obtain/upgrade/remove](../../crates/spire-core/src/run/tests/m15_silent_heart.rs); T10 linked; normal/final review pending |
| `WELL_LAID_PLANS` | `Well Laid Plans` | Uncommon/Power | `WellLaidPlans` | I finite / yes / yes | [Runtime checks](../../crates/spire-core/src/combat/tests/m15_silent_powers.rs); [T9 obtain/upgrade/remove](../../crates/spire-core/src/run/tests/m15_silent_heart.rs); T10 linked; normal/final review pending |
| `ADRENALINE` | `Adrenaline` | Rare/Skill | `Adrenaline` | M / no / no | [Runtime checks](../../crates/spire-core/src/combat/tests/m15_silent_direct.rs); [T9 obtain/upgrade/remove](../../crates/spire-core/src/run/tests/m15_silent_heart.rs); T10 linked; normal/final review pending |
| `AFTER_IMAGE` | `After Image` | Rare/Power | `AfterImage` | M / no / no | [Runtime checks](../../crates/spire-core/src/combat/tests/m15_silent_powers.rs); [T9 obtain/upgrade/remove](../../crates/spire-core/src/run/tests/m15_silent_heart.rs); T10 linked; normal/final review pending |
| `ALCHEMIZE` | `Venomology` | Rare/Skill | `Alchemize` | M / no / no | [Runtime checks](../../crates/spire-core/src/combat/tests/m15_generation.rs); [T9 obtain/upgrade/remove](../../crates/spire-core/src/run/tests/m15_silent_heart.rs); T10 linked; normal/final review pending |
| `A_THOUSAND_CUTS` | `A Thousand Cuts` | Rare/Power | `AThousandCuts` | M / no / no | [Runtime checks](../../crates/spire-core/src/combat/tests/m15_silent_powers.rs); [T9 obtain/upgrade/remove](../../crates/spire-core/src/run/tests/m15_silent_heart.rs); T10 linked; normal/final review pending |
| `BULLET_TIME` | `Bullet Time` | Rare/Skill | `BulletTime` | M / no / no | [Runtime checks](../../crates/spire-core/src/combat/tests/m15_silent_cost_copy.rs); [T9 obtain/upgrade/remove](../../crates/spire-core/src/run/tests/m15_silent_heart.rs); T10 linked; normal/final review pending |
| `BURST` | `Burst` | Rare/Skill | `Burst` | M / no / no | [Runtime checks](../../crates/spire-core/src/combat/tests/m15_silent_x_repeat.rs); [T9 obtain/upgrade/remove](../../crates/spire-core/src/run/tests/m15_silent_heart.rs); T10 linked; normal/final review pending |
| `CORPSE_EXPLOSION` | `Corpse Explosion` | Rare/Skill | `CorpseExplosion` | M / no / no | [Runtime checks](../../crates/spire-core/src/combat/tests/m15_silent_poison_death.rs); [T9 obtain/upgrade/remove](../../crates/spire-core/src/run/tests/m15_silent_heart.rs); T10 linked; normal/final review pending |
| `DIE_DIE_DIE` | `Die Die Die` | Rare/Attack | `DieDieDie` | M / no / no | [Runtime checks](../../crates/spire-core/src/combat/tests/m15_silent_direct.rs); [T9 obtain/upgrade/remove](../../crates/spire-core/src/run/tests/m15_silent_heart.rs); T10 linked; normal/final review pending |
| `DOPPELGANGER` | `Doppelganger` | Rare/Skill | `Doppelganger` | M / no / no | [Runtime checks](../../crates/spire-core/src/combat/tests/m15_silent_x_repeat.rs); [T9 obtain/upgrade/remove](../../crates/spire-core/src/run/tests/m15_silent_heart.rs); T10 linked; normal/final review pending |
| `ENVENOM` | `Envenom` | Rare/Power | `Envenom` | M / no / no | [Runtime checks](../../crates/spire-core/src/combat/tests/m15_silent_powers.rs); [T9 obtain/upgrade/remove](../../crates/spire-core/src/run/tests/m15_silent_heart.rs); T10 linked; normal/final review pending |
| `GLASS_KNIFE` | `Glass Knife` | Rare/Attack | `GlassKnife` | M / no / no | [Runtime checks](../../crates/spire-core/src/combat/tests/m15_silent_cost_copy.rs); [T9 obtain/upgrade/remove](../../crates/spire-core/src/run/tests/m15_silent_heart.rs); T10 linked; normal/final review pending |
| `GRAND_FINALE` | `Grand Finale` | Rare/Attack | `GrandFinale` | M / no / no | [Runtime checks](../../crates/spire-core/src/combat/tests/m15_silent_history.rs); [T9 obtain/upgrade/remove](../../crates/spire-core/src/run/tests/m15_silent_heart.rs); T10 linked; normal/final review pending |
| `MALAISE` | `Malaise` | Rare/Skill | `Malaise` | M / no / no | [Runtime checks](../../crates/spire-core/src/combat/tests/m15_silent_x_repeat.rs); [T9 obtain/upgrade/remove](../../crates/spire-core/src/run/tests/m15_silent_heart.rs); T10 linked; normal/final review pending |
| `NIGHTMARE` | `Night Terror` | Rare/Skill | `Nightmare` | I finite / yes / yes | [Runtime checks](../../crates/spire-core/src/combat/tests/m15_silent_cost_copy.rs); [T9 obtain/upgrade/remove](../../crates/spire-core/src/run/tests/m15_silent_heart.rs); T10 linked; normal/final review pending |
| `PHANTASMAL_KILLER` | `Phantasmal Killer` | Rare/Skill | `PhantasmalKiller` | M / no / no | [Runtime checks](../../crates/spire-core/src/combat/tests/m15_silent_powers.rs); [T9 obtain/upgrade/remove](../../crates/spire-core/src/run/tests/m15_silent_heart.rs); T10 linked; normal/final review pending |
| `STORM_OF_STEEL` | `Storm of Steel` | Rare/Skill | `StormOfSteel` | M / no / no | [Runtime checks](../../crates/spire-core/src/combat/tests/m15_silent_x_repeat.rs); [T9 obtain/upgrade/remove](../../crates/spire-core/src/run/tests/m15_silent_heart.rs); T10 linked; normal/final review pending |
| `TOOLS_OF_THE_TRADE` | `Tools of the Trade` | Rare/Power | `ToolsOfTheTrade` | M / no / no | [Runtime checks](../../crates/spire-core/src/combat/tests/m15_silent_powers.rs); [T9 obtain/upgrade/remove](../../crates/spire-core/src/run/tests/m15_silent_heart.rs); T10 linked; normal/final review pending |
| `UNLOAD` | `Unload` | Rare/Attack | `Unload` | M / no / no | [Runtime checks](../../crates/spire-core/src/combat/tests/m15_silent_history.rs); [T9 obtain/upgrade/remove](../../crates/spire-core/src/run/tests/m15_silent_heart.rs); T10 linked; normal/final review pending |
| `WRAITH_FORM` | `Wraith Form v2` | Rare/Power | `WraithForm` | M / no / no | [Runtime checks](../../crates/spire-core/src/combat/tests/m15_silent_powers.rs); [T9 obtain/upgrade/remove](../../crates/spire-core/src/run/tests/m15_silent_heart.rs); T10 linked; normal/final review pending |


Existing I-row regression owners are `combat/tests/m12_silent_values.rs` and
`m11_silent_{poison,discard,shiv,retain,copy}.rs`; native progression in
`run/tests/m12_silent_act1.rs`, `m13_silent_act2.rs`, `m14_silent_act3.rs` and
`spire-env/tests/run_m14_schema.rs`. These are source/test anchors, not fresh
results from this documentation task. Accuracy's application refreshes existing
four piles, draw refreshes hand, and manual selected discard refreshes hand;
other newly reached copy/movement cases need qualification rather than an
unsupported assertion that the accepted path is defective. Reflex/Tactician's
existing selected-discard callbacks are real; new random/all/Unload producers
need source-specific notification ordering. End-turn cleanup must remain excluded.

## Generated, status, copy and selection paths

| Resource/path | Baseline distinction | Missing M15 work / assertion owner |
|---|---|---|
| Shiv and Shiv+ | I: generated Attack,0 cost,4/6 damage, Exhaust; not ordinary reward | T4/5/7: InfiniteBlades ordinary Shiv, Storm/Cunning upgrades, Accuracy, copy identity and hand overflow |
| Dazed/Burn/Slimed/Wound/Void | I: generated encounter/status rules, separate permanent admission; Burn+ is enemy upgrade, not a new green kind | T8/9: new generation/autoplay/no-draw/selection routes keep actual status behavior; no invented Sludge ID |
| Parasite | I: permanent WrithingMass gain and purge/maxHP callbacks; not ordinary offer | T8/9: retain Run gain/removal owner, Victory/Death and fault custody |
| Nightmare | I finite captured-template/next-turn delivery3 with independent installation cursor | T5: new cost/family/delayed-copy compositions, no alias to later source card |
| Fresh/stat-equivalent/same-family copy | I for accepted M10/M11 paths, not a universal field-copy guarantee | T5: EndlessAgony/GlassKnife/Burst/Storm and cost reset; T8: generated colorless and potion templates |
| Hand selected discard | I: suspended native owner, exact continuation; cleanup/exhaust distinct | T3: random/all/undersized/Unload and draw-reaction order; exactly-once payment/resume |
| Retain/end-turn selection | I finite WellLaidPlans, optional up-to power amount; identity entitlements | T4/5: Blur/Tools/Intangible end-turn interleaving, no Ethereal immunity |
| Generated card offer | M: no public/pending offer owner distinct from pile cards | T8/10: typed three-offer source, optionality, select/skip, templates not issued card handles, replay/capacity |
| Distraction | M: no native card or selector | G1/T8: full actual-character Skill support; HEALING excluded, X/unplayable not forced playable |
| Alchemize and nested potion use | M: no native factory/obtain effect | G2–G5/T8:33 factory IDs,32 limited, full belt no replacement prompt; resolver-native error/Run fault owners, real actual inventory |
| Transitive Colorless children | M:34 eligible parent kinds have zero executable handlers | G3/T8: typed source supports, own generated cards/powers/statuses, instance/cost/selection/Run gains; do not omit generator kinds |

## Exclusive relic coverage: all11

[Relic reference](../../data/relics/README.md) and its `relics.json` identify all11;
all11 records currently have `local_static:null`. Their effect summaries and
citations are reference grade, not private-source mechanics clearance. Existing
native [item IDs](../../crates/spire-core/src/content/ids/items.rs),
[relic owner](../../crates/spire-core/src/items/relics.rs) and
[Poison owner](../../crates/spire-core/src/combat/powers.rs) establish the two I
rows. S6 names the exact missing evidence for new callbacks.

| Catalog key / game ID | Native ID (proposed if M) | Baseline behavior/admission | M15 source and interaction |
|---|---|---|---|
| Ring of the Snake / `Ring of the Snake` | `RingOfTheSnake` | I starter; draw2 opening once; registry definition_only flag does not negate actual hook | Existing starter; T7 Serpent replacement removes its bonus; [runtime/acquisition](../../crates/spire-core/src/combat/tests/m15_silent_items.rs), [M15 tiers](../../crates/spire-core/src/run/tests/m15_silent_heart.rs) |
| Snecko Skull / `Snake Skull` | `SneckoSkull` | I admitted; positive Poison application+1 after Artifact guard | Normal stock; T6/7 multi-application/transfer, not tick/decay+1; [runtime/acquisition](../../crates/spire-core/src/combat/tests/m15_silent_items.rs), [M15 tiers](../../crates/spire-core/src/run/tests/m15_silent_heart.rs) |
| Ninja Scroll / `Ninja Scroll` | `NinjaScroll` | M/no | Normal uncommon;3 ordinary startup Shivs, S6 timing versus opening draw; [runtime/acquisition](../../crates/spire-core/src/combat/tests/m15_silent_items.rs), [M15 tiers](../../crates/spire-core/src/run/tests/m15_silent_heart.rs) |
| Paper Krane / `Paper Crane` | `PaperKrane` | M/no | Normal uncommon; weak enemy damage multiplier0.6; T7/9 Heart multi-hit; [runtime/acquisition](../../crates/spire-core/src/combat/tests/m15_silent_items.rs), [M15 tiers](../../crates/spire-core/src/run/tests/m15_silent_heart.rs) |
| The Specimen / `The Specimen` | `TheSpecimen` | M/no | Normal rare; source-selected Poison transfer at death; S5/S6 target/amount/Artifact/Skull/half-dead; [runtime/acquisition](../../crates/spire-core/src/combat/tests/m15_silent_items.rs), [M15 tiers](../../crates/spire-core/src/run/tests/m15_silent_heart.rs) |
| Tingsha / `Tingsha` | `Tingsha` | M/no | Normal rare;3 random-target damage per qualifying manual discard; S6 RNG/callback order; [runtime/acquisition](../../crates/spire-core/src/combat/tests/m15_silent_items.rs), [M15 tiers](../../crates/spire-core/src/run/tests/m15_silent_heart.rs) |
| Tough Bandages / `Tough Bandages` | `ToughBandages` | M/no | Normal rare;3 Block per qualifying discard, no cleanup substitute; [runtime/acquisition](../../crates/spire-core/src/combat/tests/m15_silent_items.rs), [M15 tiers](../../crates/spire-core/src/run/tests/m15_silent_heart.rs) |
| Twisted Funnel / `TwistedFunnel` | `TwistedFunnel` | M/no | Shop-only; startup Poison4 per enemy; S6 startup order and Skull/Artifact; [runtime/acquisition](../../crates/spire-core/src/combat/tests/m15_silent_items.rs), [M15 tiers](../../crates/spire-core/src/run/tests/m15_silent_heart.rs) |
| Hovering Kite / `HoveringKite` | `HoveringKite` | M/no | Boss; energy1 first qualifying discard each turn, reset/custody needed; [runtime/acquisition](../../crates/spire-core/src/combat/tests/m15_silent_items.rs), [M15 tiers](../../crates/spire-core/src/run/tests/m15_silent_heart.rs) |
| Ring of the Serpent / `Ring of the Serpent` | `RingOfTheSerpent` | M/no | Boss, requires/replaces Snake, not eligible BossSwap; draw+1 each turn incl opening; [runtime/acquisition](../../crates/spire-core/src/combat/tests/m15_silent_items.rs), [M15 tiers](../../crates/spire-core/src/run/tests/m15_silent_heart.rs) |
| Wrist Blade / `WristBlade` | `WristBlade` | M/no | Boss; eligible0-turn-cost Attack damage+4; X/free/copy details S6; [runtime/acquisition](../../crates/spire-core/src/combat/tests/m15_silent_items.rs), [M15 tiers](../../crates/spire-core/src/run/tests/m15_silent_heart.rs) |

No Silent relic is U at baseline: two implemented/admitted, nine missing.
The design's19-offer stock includes six ordinary exclusive kinds; shop/starter/
boss sources remain separate. Source tier and replacement rules must be qualified
before enabling acquisition. Never offer an exclusive as an unrelated generic tier.


## Potion coverage and transitive dependencies

Baseline [potion use owner](../../crates/spire-core/src/items/potions.rs) has13
named kinds:10 shared and three red. Silent M12–14 admit Fire/Strength/Weak/
Dexterity only. Thus six shared kinds are **U**, while red-exclusive use support
is not ordinary Silent acquisition. Catalog `definition_only` is not a reliable
substitute for reading real handlers. There is no potion dataset to duplicate;
reuse the [economy/potion study §§5–8](../research/2026-10-19-act1-content-economy-potions.md).

| Kinds / proposed native IDs | Baseline classification | M15 obligation |
|---|---|---|
| FirePotion, StrengthPotion, WeakPotion, DexterityPotion | I admitted finite Silent | Preserve actual use/target/belt and M15 publication |
| FruitJuice, BlockPotion, EnergyPotion, SwiftPotion, FearPotion, AncientPotion | U implemented use, not finite Silent admission | Admit actual generator outputs and publish; Fruit excluded limited, allowed outside Brew |
| BloodPotion, Elixir, HeartOfIron | I red-only use, ineligible Silent factory | Keep old behavior; do not insert into Silent factory |
| Poison Potion / `PoisonPotion` (`Poison Potion`) | M | Common;6/12 Poison, targeted ordinary application |
| Cunning Potion / `CunningPotion` (`CunningPotion`) | M | Uncommon;3/6 upgraded Shivs, actual overflow/Accuracy |
| Ghost in a Jar / `GhostInAJar` (`GhostInAJar`) | M | Rare;1/2 Intangible, source expiry and Heart multi-hit |
| ExplosivePotion (`Explosive Potion`) | M | Shared NORMAL AoE, distinct from Fire THORNS |
| AttackPotion (`AttackPotion`), SkillPotion (`SkillPotion`), PowerPotion (`PowerPotion`) | M | G5: exact expected Silent26/33/11 supports, three distinct ordered offers, typed skip,1/2 chosen copies; qualify named source before enabling |
| ColorlessPotion (`ColorlessPotion`) | M | Full34 nonhealing uncommon/rare support G3 |
| FlexPotion (`SteroidPotion`), SpeedPotion (`SpeedPotion`) | M | Positive stat plus separate loss debuff; Artifact independently |
| BlessingOfTheForge (`BlessingOfTheForge`) | M | Once-only hand upgrade at action, no doubled upgrade |
| RegenPotion (`Regen Potion`), LiquidBronze (`LiquidBronze`), EssenceOfSteel (`EssenceOfSteel`) | M | Source heal/Thorns/PlatedArmor lifetimes and positive-loss predicates |
| GamblersBrew (`GamblersBrew`), LiquidMemories (`LiquidMemories`) | M | Optional discard/draw vs mandatory discard retrieval, real selectors and hand limits |
| DuplicationPotion (`DuplicationPotion`), DistilledChaos (`DistilledChaos`) | M | Real repeated uses/autoplay, captured energy/targets, source duration/cleanup |
| CultistPotion (`CultistPotion`), SneckoOil (`SneckoOil`) | M | Ritual; whole resulting hand cost randomization, X/unplayable guarded |
| FairyPotion (`FairyPotion`), SmokeBomb (`SmokeBomb`) | M | Automatic rescue/nonmanual use; source escape restrictions/Run settlement |
| EntropicBrew (`EntropicBrew`) | M | Live slot-count proposals; combat limited vs outside unrestricted; no full-belt prompt |

The shared M rows collectively name all20 missing shared potion kinds;
with three exclusive kinds, **23 of33 Silent factory identities are missing**.
The proposed native spelling is a planning decision; parentheses retain the
exact reference factory ID. All new use handlers need ordinary and SacredBark
interaction coverage at the cited grade, with actual consumed inventory. Bark/
Sozu/PotionBelt compositions can be supplied diagnostics without declaring those
relics normal M15 acquisitions; do not add a source mechanism based on unowned
hypothetical state. Their reachable admission is separately explicit.

### G1: Distraction support

Named static lookup: `Distraction.use`, `Alchemize.<init>`,
`AbstractDungeon.returnTrulyRandomCardInCombat(CardType)` / `initializeCardPools`,
`TheSilent.getCardPool`, `CardLibrary.addGreenCards`, `CardGroup.addToTop/addToBottom`.
It concatenates Common/Uncommon/Rare actual-character src pools filtered SKILL
and not HEALING, then one bounded list index, no rarity draw or self exclusion.
Alchemize is explicitly HEALING, so excluded. Native deterministic sorting within
rarity is permitted; Java HashMap iteration/seed cursor parity is not required.

Expected fully unlocked, unmodded Silent projection is **33 =10/14/9**:

- Common: Acrobatics, Backflip, BladeDance, CloakAndDagger, DeadlyPoison, Deflect,
  DodgeAndRoll, Outmaneuver, PiercingWail, Prepared.
- Uncommon: Blur, BouncingFlask, CalculatedGamble, Catalyst, Concentrate,
  CripplingCloud, Distraction, EscapePlan, Expertise, LegSweep, Reflex, Setup,
  Tactician, Terror.
- Rare: Adrenaline, BulletTime, Burst, CorpseExplosion, Doppelganger, Malaise,
  Nightmare, PhantasmalKiller, StormOfSteel.

The other33 constructors' runtime tags/types were not checked by the bounded
planning lookup: this exact expected set is a catalog projection, not a fully
constructor-verified initialized session. Task1 must confirm those named
constructors' relevant fields/tags before enabling it; no game execution or
incidental iteration reconstruction required. Missing native behavior for these
eligible kinds is runtime work, not a source exemption. Generated X/unplayable
cards keep sentinels and ordinary legality; being a generated Skill is not a
free play permission.

### G2: Alchemize and complete potion support

Named static lookup confirms `AbstractDungeon.initializePotions` →
`PotionHelper.initialize(PlayerClass)` / `getPotions(SILENT,false)`:
30 shared + Poison/Cunning/Ghost =33; constructor rarities common/uncommon/rare.
Combined with existing shared16/8/6 roster gives Silent **17/9/7**, limited
**17/9/6** after excluding rare FruitJuice. `Alchemize.use` queues
`ObtainPotionAction(returnRandomPotion(true))`; `EntropicBrew.use` chooses the
limited helper in combat and unrestricted outside.

This is qualified static evidence, not a game run or native factory. Exact
Silent conditional law at `AbstractDungeon.returnRandomPotion` overloads and
`PotionHelper` remains to be checked as named Task1 work before pinning per-ID
probabilities; do not assume every helper is flat or use an Ironclad list.
Existing study specifies rarity65/25/10 and uniform eligible IDs within tier,
source first-discarded limited proposal and full-belt distinctions. A direct
native conditional-law test should use those verified inputs, not seeded Java
cursor witnesses. No arbitrary retry cap is allowed.

### G3: Colorless Potion and recursive card generation

The study's35 ordinary colorless identities minus HEALING BandageUp gives
**34 =19 Uncommon +15 Rare**. **0 have executable native handlers**. Madness is
one registry-only definition with no payload; the other33 lack named IDs/rows.
A supported row or passing data test does not close this runtime gap.

| Uncommon19 | Rare15 |
|---|---|
| Blind, DarkShackles, DeepBreath, Discovery, DramaticEntrance, Enlightenment, Finesse, FlashOfSteel, Forethought, GoodInstincts, Impatience, JackOfAllTrades, Madness, MindBlast, Panacea, PanicButton, Purity, SwiftStrike, Trip | Apotheosis, Chrysalis, HandOfGreed, Magnetism, MasterOfStrategy, Mayhem, Metamorphosis, Panache, SadisticNature, SecretTechnique, SecretWeapon, TheBomb, ThinkingAhead, Transmutation, Violence |

Names are proposed native variants; exact game IDs/base/+ and source notes stay
in the existing card catalog (e.g. `Jack Of All Trades`, `HandOfGreed`,
`PanicButton`). Task8 owns these34 actual behaviors, their source-qualified
recursive supports and generated dispositions. Discovery/Jack/Magnetism/
Transmutation and Chrysalis/Metamorphosis/Mayhem need actual current-character
supports, copy/cost and repeat/turn owners; HandOfGreed needs durable gold through
Run's mutation/gain owner. Do not stop the dependency graph at a reference row,
and do not generate an unsupported colorless kind and call the parent complete.
S1–S5 apply to their shared mechanics; unresolved exact rules are blockers.

### G4: Queued obtain and durable resources

Ordinary obtain inserts first empty slot or returns false if full, with no Sozu
test in that helper. Combat ObtainPotionAction checks Sozu at execution and
attempts the already generated proposal once; no resample/replacement chooser.
Outside obtain effect also completes without a full-belt prompt. Reward claims
have separate full-belt retention behavior; a source-specified Sozu claim can
consume an offer without obtain. Manual use invokes payload/use hooks then
vacates its own slot; generated obtains may execute later. Discard is not use.
The combat obtain helper returns `Result<bool,InputError>` and the shared
resolver converts invalid accepted ID/inventory through `EffectError::Invalid`,
retaining the actual current effect/source and first fault. Full valid belt is
`Ok(false)`. The outside-Run adapter uses Run errors before acceptance and a
retained `run.m15.potion.obtain` invariant fault after acceptance; no blanket
`RunError` conversion is added to the combat resolver.
Combat Brew produces live total-slot-count proposals, not free-slot-count or
Bark-potency-count. Outside Brew checks Sozu before sampling. Retain actual belt,
accepted prefix and pending owner on fault, and return remaining resources on
Victory/Death. Fairy rescue and Smoke escape must use actual source guards and
accepted Run/combat terminal owners. These native paths are currently missing.

### G5: Typed Attack/Skill/Power potion selectors

**Named source frontier, assigned to Task1 before Task8:** `AttackPotion.use`,
`SkillPotion.use`, `PowerPotion.use`; their `DiscoveryAction` constructors,
`generateCardChoices` and `update`/combat reward-screen selection path;
`AbstractDungeon.returnTrulyRandomCardInCombat(CardType)` / `initializeCardPools`,
`TheSilent.getCardPool`, `CardLibrary.addGreenCards`, and relevant green
constructors' type/rarity/HEALING tags (including `Alchemize.<init>`). Qualify
actual-character source pools, constructor flags/skip semantics, and repeated
identity handling. The existing economy study §7.1's28/28/14 type counts are
Ironclad-only and are not used as Silent expectations.

Expected fully unlocked/unmodded Silent nonbasic catalog projections are
**Attack26 (9/13/4), Skill33 (10/14/9), Power11 (0/6/5)** by
Common/Uncommon/Rare. Alchemize's confirmed HEALING tag excludes it; unplayable
Reflex/Tactician remain eligible Skills. Basics, Shiv, status, curse and SPECIAL
kinds are outside the source pools, and no card is removed merely because its
native handler or ordinary play condition is missing. Exact expected sets:

| Potion/type | Common | Uncommon | Rare |
|---|---|---|---|
| Attack26 | Bane, DaggerSpray, DaggerThrow, FlyingKnee, PoisonedStab, QuickSlash, Slice, SneakyStrike, SuckerPunch | AllOutAttack, Backstab, Choke, Dash, EndlessAgony, Eviscerate, Finisher, Flechettes, HeelHook, MasterfulStab, Predator, RiddleWithHoles, Skewer | DieDieDie, GlassKnife, GrandFinale, Unload |
| Skill33 | Acrobatics, Backflip, BladeDance, CloakAndDagger, DeadlyPoison, Deflect, DodgeAndRoll, Outmaneuver, PiercingWail, Prepared | Blur, BouncingFlask, CalculatedGamble, Catalyst, Concentrate, CripplingCloud, Distraction, EscapePlan, Expertise, LegSweep, Reflex, Setup, Tactician, Terror | Adrenaline, BulletTime, Burst, CorpseExplosion, Doppelganger, Malaise, Nightmare, PhantasmalKiller, StormOfSteel |
| Power11 | — | Accuracy, Caltrops, Footwork, InfiniteBlades, NoxiousFumes, WellLaidPlans | AfterImage, AThousandCuts, Envenom, ToolsOfTheTrade, WraithForm |

These sets are explicit expected assertions, not newly verified instantiated
source fields: the planning fix reads the existing catalog/study and does not
perform another private audit. Task1 must qualify the named selector path and
relevant constructor tags/types before a typed sampler is enabled. Unavailable
exact membership/flag rules remain a completion blocker; missing runtime
behavior is planned work, never permission to sample a smaller set.

Native order is Common→Uncommon→Rare, IDs sorted by `CardId::as_str` within each
rarity. Preserve each selector's conditional ordered uniform-without-replacement
law over its complete support: after k distinct offers, next eligible support
has size N-k; no rarity lottery or eligibility reroll after publication. Java
HashMap/cursor parity is excluded. Task8's named support test compares all three
complete expected sets and native order, then three distinct ordered offers.
The study's typed path exposes optional skip: assert `optional == true`, a legal
skip creates no copy and resumes once, selection creates1/2 copies of the chosen
identity and inspection/select do not reroll. Confirm constructor/update flag
meaning at the named frontier before asserting this oracle; do not transfer the
flag to ColorlessPotion or Discovery's distinct source path.

## Events and ordinary character eligibility

| Existing event | Native behavior and finite admission | M15 requirement / evidence limit |
|---|---|---|
| Neow | I short-Neow start; full Neow not admitted | Canonical Silent resources and existing actual two short choices; no boss-swap claim |
| UpgradeShrine | I first captured Act1 Question | T9 actual green upgrade legality/instance selection |
| Purifier | I second Act1 Question / ineligible-Joust fallback | T9 green removal, Parasite callback/EmptyDeck and duplicate identity |
| AncientWriting | I first Act2 Question; recognizes green basics | T9 full new green roster through shared eligibility; actual upgrade/removal |
| Joust | I second Act2 Question if act2/gold>=50 | Keep existing loss/gold/preflight rules; no claim of original full event pool |
| Falling | I first Act3 Question | T9 typed Attack/Skill/Power eligible card loss; disabled empty types, duplicate-instance custody |
| TombOfLordRedMask | I second Act3 Question | All-gold exchange/owned+222, RedMask source separate from normal stock |

[Native event registry](../../crates/spire-core/src/content/events.rs) has these
seven definitions, with six ordinary room handlers. No Silent-exclusive event is
registered; this does **not** prove that the original character-exclusive set is
empty. Existing [event research](../research/2026-10-19-act1-content-event-neow.md)
is Ironclad A0 Act1 author-draft evidence, not a Silent Acts1–4 matrix. Exact
unresolved evidence: per-act `initializeEventList`/selection, character predicates
and applicable `canSpawn` for Silent; inspect only the named eligibility facts
needed to rule on exclusive requirements. Full shared event-pool delivery stays
M26–28. If a genuinely Silent-exclusive event exists, it becomes required M15
work; absence from the current registry is no scope waiver. Act4's fixed route
has no question/event node. This outstanding exclusive-eligibility ruling is S8.

## Exact unresolved composition evidence

These are the existing study's explicit unclosed fields narrowed to M15 inputs.
Accepted local rules remain usable; no universal FIFO, copy, death or rollback
law is invented. Task1 resolves the named missing rule before its owning test
asserts exact composed outcomes; runtime missing mechanics remain implementation.

| Key | Exact unresolved condition and named source frontier | Affected work |
|---|---|---|
| S1 cost | `AbstractCard` costForTurn/free/reset/upgrade and `UseCardAction` cleanup, MasterfulStab/Eviscerate callbacks in actual incoming loss/discard routes | T3/5: mixed writers/normal versus autoplay/X legality; own instance/location, no global-history replacement |
| S2 copy | `makeCopy`/`makeStatEquivalentCopy`/same-instance UUID, `ModifyDamageAction`, Burst/Nightmare/EndlessAgony and generated-colorless copies | T5/8: exactly captured mutable fields, family mutation vs new identity; no aliases or master-deck growth |
| S3 history | `DrawCardAction` shared history/initial-clear/follow-up across nested reaction draws; `GameActionManager` cardsPlayed/discard resets; Unload-specific counter/callback order | T3/5: EscapePlan/Expertise/Gamble/Finisher and Eviscerate; no absent child replaced with Draw(0) |
| S4 order | Named power callbacks and player/relic on-use/after-play/start/end-turn dispatch, Tools draw/discard, Blur/Intangible/Wraith, retain/Ethereal | T4/5/7/9: source-qualified priority/append/prepend and suspension; accepted headless scheduler checkpoints remain one owner |
| S5 death | `DamageAction`/HP-loss/matrix recipients, `AbstractMonster.die`/half-dead, CorpseExplosionPower/TheSpecimen, shared split/rebirth/last-enemy cleanup | T6/7/9: maxHP/current Poison at actual callback, transfer/chain survival, Heart allowance and terminal interruption |
| S6 relic | NinjaScroll startup vs opening draw, HoveringKite trigger reset, Tingsha target draw, ToughBandages callback, WristBlade eligible-cost/damage stage, PaperCrane modifier, Specimen transfer, TwistedFunnel startup, RingOfTheSerpent replacement | T7: catalog summaries have no local_static clearance; named relic methods must qualify precise native oracle |
| S7 numeric/RNG | recipient-side binary32/rounding and caps, signed amount/overflow, actual generator supports/conditional target laws G1–G3/G5 | T2/4/5/6/8/9; no game-damage cap, silent saturation, hidden RNG leak or Java PRNG parity claim |
| S8 event | Silent-only event eligibility through per-act event lists, actual character predicates and relevant spawn rules | T1/9; full shared generation later, real exclusive content cannot be excluded by missing registry |

## Integration and acceptance ledger

| Surface | Baseline / independent owner | Required M15 evidence |
|---|---|---|
| Keys/door | M14 all-key supplied Act4Ready only; M10 full actual door implemented | T9 actual earned keys+same-visit Act3 victory/fourth choice, missing-key ending, heal/route once |
| Shield/Spear and Heart | Existing shared setup keys42/43 and resolver mechanics | T9 Silent Artifact/Poison/Specimen/facing, Intangible/multi-hit, Beat per actual use and allowance |
| Native schemas/tools | M14 finite schema/session/replay/batch, M10 facing/allowance groups | T10 complete M15 vocabulary, new generated offers, privacy/capacity, typed27 histories, portable replay and ordered worker agreement |
| Encounter source | Accepted baseline43 setups; `.68` independently expanded to all57 defined IDs, new gold theft/recovery and powers | Reconcile its accepted source/PR before execution, preserve old keys/frozen Run selection; use shared gold custody for HandOfGreed/escape interactions. No duplicate encounter audit. |
| Normal traversal | M14 normal42 all Death; supplied6 win/end, canonical completion unproven; old caller does not deliberately take keys | T11 freezes public missing-key Recall/Sapphire/Emerald and burning-route objective, legal checks/distance-NodeId ties/fourth ending choice before same six attempts; exact earned reach or explicitly unproven mandatory gate, no outcome tuning |
| Supplied diagnostics | Existing source-qualified M10 and M14 fixtures | T9 actual mechanics/endings only, never relabel normal; no automatic M14 amendment transfer |
| Python | M12–14 native-only, extension identity not established for new source | Native-only recommendation; no build/import/delivery claim |

Task11's separate `M15CallerOutcome::HeartVictory` requires the actual final
native Heart terminal after legal Proceed and its accepted trace receipt;
entry/pending victory, no-key/Death/EmptyDeck and budget outcomes stay distinct.
The existing CallerOutcome enum and M13/M14 exhaustive/source contracts remain
unchanged; this settlement label does not upgrade supplied evidence to normal.

The separate encounter runner's57 closure is pending at this planning freeze,
not accepted M15 source. Its stated anticipated gold interface keeps theft debit,
per-actor carried gold and earned unclaimed stolen-gold receipt distinct;
escape loses carried gold, actual death earns recovery, and all-escaped combat
suppresses ordinary gold. M15 reads the accepted final API before implementation;
this ledger neither accepts that draft nor creates a competing gold journal.
README PR52 changes two Markdown files only; accepted runtime baseline is unchanged.

Planning source facts were checked against an independently retained accepted
source export. New named lookup outputs use an owned archive SHA256
`cfad868ac8d65a88e71a0bf096fb09f78811e553effe0787c5309a655e081673`
and javap21.0.12.1; archive upstream build/distribution channel is **unverified**.
Lookups confirmed only the named Distraction/initialization/tag and potion
initialization/constructor/use facts above. No game/class execution occurred;
no private bytecode is published. Source-fact reports/input manifests are retained
with the execution brief outside managed sessions and frozen for root's review.

M15 cannot be marked character-complete while M/E obligations above remain
unresolved. Missing ordinary mechanics are planned work; unavailable exact
source/runtime rules explicitly prevent completion. Full standard pool/shared
content scope is a separate roadmap gate, never implied by one successful deck
or one supplied Heart fixture.

## Execution record — 2026-10-27, in progress

The primary user approved the v3 package; the separate
[approval record](../superpowers/plans/2026-10-27-m15-silent-heart-approval.md)
preserves that authorization. Execution uses accepted encounter PR53 baseline
`d1fa706638a31a854fdcaeed9a9a4642abf716eb`. The inventory above remains its
revision-qualified planning baseline. The rows below record implementation
progress without changing the full character obligations or claiming acceptance.

| Kinds / obligation | Current native work and evidence | Remaining gate |
|---|---|---|
| DaggerSpray, Slice, SuckerPunch, Backstab, Dash, LegSweep, RiddleWithHoles, Adrenaline, DieDieDie | Task2 shared direct payloads, registry, permanent/generated constructors and public values. Five `m15_direct` tests cover both variants, actual targets/payment/hits/Block/Weak/Artifact/draw/Exhaust/Innate; meaningful missing-registry RED retained, then GREEN. Existing `m12_silent_values`: two passes. | M15 admission/publication, integrated final verification and independent review pending. |
| Acrobatics, Prepared, SneakyStrike, AllOutAttack, CalculatedGamble, Concentrate, EscapePlan, Expertise, Finisher, Flechettes, GrandFinale, Unload | Task3 typed continuations and private action-time play/discard/draw history on the shared resolver. Eight `m15_silent_history` tests cover both variants, absent versus zero draw, undersized/manual/specific discard, actual count snapshots and qualified EscapePlan/Evolve/reshuffle schedules. Initial five behavior failures retained; focused GREEN. | Further cost/item compositions remain Tasks5/7; M15 admission/publication and final review pending. |
| Existing M11 selected-discard fault custody | Regression initially detected changed queued draw identity. New history support now opts into explicit continuations while old profiles retain their original `Draw` packet. All14 `m11_silent_discard` tests pass after the fix, with the old assertion unchanged. | M15 Run must explicitly enable its history path; no old-profile fault/schema change is authorized. |
| G1/G2/G5 membership | Named initialized pools and all relevant green/potion constructor fields confirm Silent26/33/11, untyped70, Distraction33, potion33 at17/9/7 and limited32 at17/9/6. Native declared support order remains as approved. | Full behavior dependencies and source-specific samplers remain Task8 work; none enabled by reference facts alone. |
| G5 typed optionality | The typed constructor stores its supplied nonnull type; `DiscoveryAction.update` passes `cardType != null` as screen skippability. Typed Attack/Skill/Power offers allow skip. An initial private report misread this conditional; original evidence is preserved with an attributed correction after root and senior verification. | Approved optional=true rule stands; no amendment. Untyped/colorless retain their own mandatory path. |
| S1–S3 | Named source writers/copy paths/history checkpoints qualified, including Eviscerate's source-side copy mutation and the two exact nested draw schedules. | Task5 must implement the actual state-aware copy seam; no universal scheduler/cost theorem is claimed. |
| S4–S8/G3/G4 | Named boundary evidence retained; S7 reuses accepted signed/binary32 owners, S8 finds no Silent-only event gate in the per-act/shared selectors. Local callback/death/relic frontiers have concrete implementation seams and bounded follow-up work. | Exact callback/death compositions still under qualification; subsequent runtime obligations remain open. |

The current combined direct/history filter records13 passes; it is a scoped
native check. External receipts retain exact source hashes, toolchain, commands,
failures and results independently of the managed checkout. No normal M15
outcomes, source/caller outcome freeze, final workspace matrix, final independent
review, milestone acceptance or Python delivery is claimed.


### Recovery integration checkpoint — 2026-10-27

The retained partial source was resumed at accepted encounter baseline
`d1fa706638a31a854fdcaeed9a9a4642abf716eb`. This checkpoint is implementation
progress, not M15 acceptance or normal-start traversal evidence.

- Task5's six cost/copy kinds pass 13 focused tests in the recovery checkout;
  the original legacy free/X payment test and M11/M10 copy/autoplay regressions
  pass. Ordinary `free_play` clears at the existing commit; Setup's distinct
  `free_to_play_once` clears at FinishCard. X/repeat work remains open.
- Task7's retained nine-relic/three-potion implementation was integrated against
  its exact preimages. The shared Death hook now captures Specimen Poison after
  monster power callbacks and defers target choice to its existing typed effect.
  Nine item tests passed at that checkpoint, including a new actual lethal-hit
  composition with Corpse Explosion and Snecko Skull; predecessor poison,
  copy, Heart and Silent interaction checks passed. M15 Run admission is pending.
- Task8 HandOfGreed uses the same Fatal gate as Feed and the accepted combat
  currency owner. Base/upgraded actual gold, minion exclusion, real Darkling and
  Awakened half-death, stale action and gold-overflow prefix checks pass. Existing
  colorless-direct, Feed and thief checks pass. Run-level gain synchronization
  remains part of Task9 evidence.
- Six shared potion kinds now have real belt payloads: FlexPotion, SpeedPotion,
  RegenPotion, LiquidBronze, EssenceOfSteel and CultistPotion. Paired Sacred Bark,
  independent Artifact/stat-loss, player end-turn resources and qualifying damage
  checks pass. Player Plated Armor extends the existing shared damage callback;
  both predecessor Shelled Parasite Plated Armor tests pass. This does not enable
  a partial potion factory or satisfy the remaining generated potion obligations.

The complete generation/obtain/colorless dependencies, remaining X/repeat work,
M15 profile/earned keys/Act4, native transport and frozen public-caller evidence
remain open. No normal episodes have been attempted and no final workspace
matrix, final milestone review, Python or original-runtime result is claimed.


## Historical recovery checkpoint: generation before full factory/T9

This paragraph records an earlier recovery checkpoint; the current dated status above supersedes its open-work index.

The baseline columns above retain the accepted M14 inventory. They are not the
current recovery implementation index. The recovery branch is still an unaccepted
M15 candidate; Run admission/publication, remaining potion dependencies and final
normal evidence are open.

- T5 X/repeat and T7 exclusive item behavior are integrated through the shared
  card queue, damage, discard and real belt owners. Existing legacy payment and
  fault assertions remain in their preservation lanes.
- T8 direct/choice/power/generation colorless families now have native handlers,
  including actual Fatal gold for Hand of Greed. The complete flat L34 support
  excludes Bandage Up. Green generation uses exact rarity-ordered 26/33/11
  supports and their 70-kind union; Alchemize and basics are excluded by source
  eligibility, while Distraction, Reflex and Tactician remain eligible.
- [Generation tests](../../crates/spire-core/src/combat/tests/m15_generation.rs)
  cover actual card/potion parents, source-specific optionality, no reroll,
  captured X, shuffled creation without draw callbacks, source RNG exhaustion,
  and successful child custody before a later issuance fault. Generated
  templates use shared card constructors, copying and pile insertion.
- Five shared potion actions use the existing autoplay and selection owners,
  including a typed Potion parent in PlayFrame. The original Card Debug shape
  is retained. The ordinary generated-card path keeps special Shiv construction
  separate from ordinary Colorless templates, preserving its established flags.
- Named retained callback facts place Bomb, Wraith downside and player end-turn
  potion powers before card callbacks; Blur and Intangible remain round-end.
  Sadistic Nature observes source application before the Artifact guard and
  appends damage at the action tail. Timing/order REDs and focused GREENs are
  retained privately; no original-runtime fidelity is claimed.
- Alchemize remains explicitly unavailable pending the full potion factory;
  Smoke, Entropic Brew, obtain integration and Run settlement remain T8 work.
  T9–T11 still require complete admission, keys/Act4, versioned native tools,
  frozen normal inputs/outcomes and independent final review. No normal M15
  episodes or final workspace matrix have been run at this checkpoint.


2026-10-28 exact-fact addendum: Task9 prose Heart800 is corrected for the A0 implementation assertion to750, matching the shared accepted setup. Named retained CorruptHeart constructor switches750→800 only at ascension9. Approved plan/spec bytes and frozen M10 remain unchanged; the failed new800 assertion/log and manager disposition are retained for final review. Static source only; upstream channel unverified.


### Current transitive execution and evidence anchors (2026-10-28)

All baseline M/U potion rows now use executable shared owners; red-only
Blood/Elixir/Heart remain outside the Silent factory. The one
[potion factory](../../crates/spire-core/src/items/potion_generation.rs) owns33
Silent IDs,32 eligible limited outputs, source rarities17/9/7, one65/25/10
rarity draw and uncapped full33 rejection proposals; Fruit is rejected only by
limited callers. The [factory/obtain tests](../../crates/spire-core/src/items/m15_potion_generation_tests.rs)
pin tier boundaries, the initial discarded proposal, Fruit rejection, full/live
belt behavior, Sozu, once-only Toy healing, prefix custody and malformed obtains.
[Actual action tests](../../crates/spire-core/src/combat/tests/m15_potion_actions.rs)
cover Explosive, Gambler, Memories, Duplication and typed potion-parent Chaos;
[power/Forge/Oil/Fairy tests](../../crates/spire-core/src/combat/tests/m15_silent_items.rs)
and [Brew tests](../../crates/spire-core/src/combat/tests/m15_potion_generation.rs)
cover the other shared payloads with Bark and real slots. The
[Smoke tests](../../crates/spire-core/src/combat/tests/m15_smoke.rs) and
[Run tests](../../crates/spire-core/src/run/tests/m15_silent_heart.rs) cover escape
restrictions, battle-end callbacks, committed theft custody and Proceed-only
settlement. All33 factory identities are dispatched by actual use in
[generation checks](../../crates/spire-core/src/combat/tests/m15_generation.rs);
Fairy correctly rejects manual use and is tested separately through lethal damage.

All34 L34 Colorless kinds have real base/+ actions. Their direct, choice and
power behaviors are checked in
[direct](../../crates/spire-core/src/combat/tests/m15_colorless_direct.rs),
[choice](../../crates/spire-core/src/combat/tests/m15_colorless_choices.rs),
[power](../../crates/spire-core/src/combat/tests/m15_colorless_powers.rs) and
[full-pool generation](../../crates/spire-core/src/combat/tests/m15_generation.rs)
lanes. Transmutation retains captured X; Chrysalis/Metamorphosis/Jack/Discovery/
Magnetism share constructors, copying, one resolver, real pile insertion and
retained source/cursor faults. Generated templates do not acquire physical
handles until delivery. Typed potion offers permit source-qualified skip;
Colorless/Discovery use their distinct mandatory rule. No incomplete support
is enabled by silently removing a candidate.

The [current Run owner](../../crates/spire-core/src/run/rooms/m15_content.rs)
admits76 permanent kinds (75 Green plus Parasite),28 normal relics and33 potion
IDs. Its19 stock captures are shared across chests/elites; five boss options and
Twisted Funnel shop sales are separate source tiers. Four-act shop captures commit
their detached stream once. Real Recall/Sapphire/burning-elite rewards, all eight
actual boss-record key masks, Shield/Spear and reward-child closure, Heart750,
final Proceed55/+110 once, Death-1 and exact first-fault custody are checked in
the supplied [Run diagnostics](../../crates/spire-core/src/run/tests/m15_silent_heart.rs).
[Act4 composition checks](../../crates/spire-core/src/combat/tests/m15_act4_interactions.rs)
exercise Artifact, Flask/Specimen/Poison survivor facing, Burst/Beat/AfterImage/
Choked/Cuts, actual multi-hit Intangible, the shared300 Heart allowance and
last-enemy Corpse/Specimen pruning. These are supplied diagnostics, not six
normal-start outcomes. Native publication, fixed public caller evaluation and
final acceptance remain separate open obligations.


Publication addendum: existing Panache remaining-use and independent Bomb timers
are projected read-only through M15 power rows. `panache_countdown` carries
remaining uses; `panache` retains its damage amount. Each `the_bomb`/`the_bomb_plus`
row carries remaining turns with captured40/50damage respectively, preserving
timer callback order without exposing private cast identities. Actual-play
[visibility checks](../../crates/spire-core/src/combat/tests/m15_colorless_powers.rs)
cover missing-state RED/GREEN, mixed-variant order, purity and invalid-state
rejection. Native metadata and its explicit capacity/isolation checks remain
part of the Task10 integration. These views add no mechanics or state journal.


### Original frozen normal evidence (2026-10-28)

The [candidate profile](m15-silent-heart.md) reports all six original seed7/11
attempts at8192 budget from the pre-outcome762-file source/caller/rules/resource
freeze. They ended in Death at Act1floor8 (two), Act2floor23 (two), Act2floor33
(two); no operational error, failed action or budget exhaustion occurred. Four
rows earned Ruby/Emerald/Sapphire in Act1 through real source choices (+5each).
None entered Act3/Act4/Heart. Full public traces, semantic receipts, source/modes
and binary hashes are independently retained. All original results remain fixed;
no policy/resource tuning or improved-outcome rerun is authorized. Native actual
parity has been explicitly released but is not yet counted passed. Supplied
Act4 mechanics and native implementation do not close the normal-start gate;
M15 remains incomplete pending primary disposition and independent final review.


Native release result: the explicitly released six-receipt consumer passed once
against the complete frozen762-file export:3143 original actions/public snapshots,
scalar sessions, paired workers1/2 batches, records/reproductions and portable
replay agreed. Default-suite ignore remains visible and is separately qualified;
it does not substitute for the explicit passed execution. Normal completion
remains unproved. Public Rust enum extensions required two reporting/test-only
compile guards outside the evaluated M15 driver/corelib/native-consumer closure;
exact post-freeze diffs are retained, old assertions/arms preserved and no normal
outcome was rerun.


Post-freeze preservation qualification: an unchanged M10 Blood for Blood
Confused/HP-loss assertion failed in the workspace matrix (2instead of1). Only
that red card's Confused draw assignment now keeps its accepted randomized/
temporary price representation; all other cards retain the identical branch.
The original assertion and19 Blood-for-Blood checks are unchanged and pass.
This is a shared core-file compatibility delta, explicitly distinct from the
two earlier reporting/test-only compile bridges. The red card is excluded from
all admitted/generated M15 pools; original frozen source/binary/six outcomes and
v1 native agreement remain immutable. One final-source exact-action agreement
pass is authorized using the same original receipts, without policy calls or
new outcomes. No milestone acceptance or gate amendment follows from the repair.


### Stable workspace preservation result (2026-10-28)

After the qualified reporter/test enum guards and narrow Blood for Blood repair,
`cargo test --workspace --locked --offline` completed:2357 passed, zero failed,
one default-ignored check across68 test-result blocks. The earlier two compile
failures (zero completed blocks) and first runtime failure (1652 passed, one
unchanged Blood for Blood assertion failed) remain retained. Focused post-repair
checks passed19 Blood for Blood,11 pressure,14 M15 cost-copy and22 generation.
The successful source/mode identity is separately bound from the original frozen
evaluation revision. The same six original receipts have been released for one
final-runtime exact-action agreement, with no caller or new outcome evaluation.
The default-ignored consumer is distinct from its explicitly passed original-
source execution. Neither workspace preservation nor supplied Act4 diagnostics
close the normal-start gap. M15 remains incomplete; source acceptance and final
technical review have not been granted.


Final-runtime agreement result: the unchanged consumer executed once against the
stabilized762-file runtime revision and the same six original semantic receipts.
All3143 actions, public snapshots, core/scalar, workers1/2 batches, records,
reproductions and portable replay agreed with the original evidence. One explicit
check passed, zero failed, zero ignored. Both source exports, original twelve
artifacts and binaries remain independently retained; the final source hash is
qualified separately from the original evaluation hash. No caller/policy or new
outcome evaluation occurred. The successful workspace/native checks leave the
strict M15 normal-start gate unmet and do not imply source or milestone acceptance.


### Supplied runtime-first result (2026-10-28, separate primary amendment)

The [actual-primary amendment](../superpowers/plans/2026-10-28-m15-supplied-act4-runtime-amendment.md)
authorizes bounded supplied accuracy diagnostics, accepting genuine Death. The
first seed7 supplied Silent run reached HeartVictory in85 accepted public actions:
Rest51 → Shop52 → actual full-HP Shield110/Spear160 Victory/reward closure →
full-HP Heart750 Victory → legal Proceed55/+110 once. It ended52/84HP, reward150
from supplied start, with no error/failed action/budget cutoff. The fallback
was not triggered and never run; its10distinct SearingBlow+15/250HP definition
and both initial RNG/input snapshots were frozen before Run1.

This is supplied core legal-play proof, explicitly distinct from all six
immutable normal losses and both3143-action native agreement revisions. Prior
Act3 Victory/keys/history are supplied initialization, not earned/played proof.
Production runtime/caller/normal admission are unchanged; only test fixtures
and honest result documentation extend the preceding source. Setup boundary
RED/GREEN and one explicit ignored-driver execution are retained. The earlier
workspace2357/0/1 matrix was not rerun, and supplied native transport replay was
not executed under the bounded instruction. Canonical normal completion remains
unproved. Complete implementation/source/native accuracy and remaining obligations
await independent senior review managed by the primary session; victory itself
does not establish acceptance.


### Independent review v1 correction candidate (2026-10-28)

Five Important and two Minor findings prompted one bounded F1–F7 pass. Concrete
[shared finish](../../crates/spire-core/src/combat/piles.rs) and deferred-parent
owners now distinguish original Power retirement from purged repeat custody.
The power owner separates new Double Damage constructor1 from an existing stack
delta. Opening draws use the opt-in shared callback path, and Endless Agony's
typed continuation captures before Confused while retaining queued physical
issuance/fault custody. Grand Finale's unconditional price remains separate
from its play predicate. The common value publisher includes effective one-play
waivers; the M15 replay parser admits the encoder's two derived capacity names
with their proper owning capacity. No copied mechanics owner or new journal.

The original7core/1codec counterexamples reproduced; stabilized focused checks
pass M15combat175, autoplay32, copy8, retain10, turns17, power-turns28, pressure11,
native-schema11, codec2 (zero failures/ignored). Source-boundary and expanded
callback/expiry/cost/reset/waiver/fault cases are included. Original test
assertions remain unchanged, and expanded fixture errors are explicitly retained
and qualified. No new normal/supplied caller, fallback, native full-history
series or broad matrix was executed. All prior frozen candidates and outcomes
remain revision-bound; neither the85win nor earlier native/workspace receipts
are relabelled as post-fix proof. Same-reviewer scoped accuracy recheck is pending.
