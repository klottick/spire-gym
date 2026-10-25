# M10 — captured Ironclad A0 through the Heart and native Run tools

> **M10 accepted at its declared captured profile.** ONE fresh independent GPT-6 Luna MAX review
> found no findings, and the controller verified all 608 reviewed files unchanged in BOTH checkouts.
> Native implementation and the preservation matrix remain verified on the final 607-file runtime
> baseline, including all 16 Task 28 Python tests. This is not a normal-start win or standard-pool claim.

Prepared from the accepted M9 profile, approved M10 plan, qualified Task 29 fact draft, and the native
controller's verified final-source matrix and result summaries. It adds no mechanics research or
implementation claims beyond that evidence.

This accepted profile extends the [accepted M9 profile](m9-act3.md) with captured Ironclad A0
card/item mechanics, actual key sources, the fourth all-key choice, fixed Act 4 route, Shield/Spear,
Corrupt Heart, and opt-in M10 numeric/session/replay tools. It does not mean a policy has won.

## Scope and use

- M10 is **Ironclad, Ascension 0 only**. Ordinary start: 80/80 HP, 99 gold, five Strikes, four Defends,
  Bash, Burning Blood, three empty potion slots, and no keys. Boss history is context, not resources or
  injected room state; reset has no deck/HP/gold/relic/potion/key override.
- Captured M10 offers are not the full standard card/relic/potion/shop/event, rarity, or pity generators.
  They provide bounded playable content; each handler and acquisition has a native owner.
- All characters, A1–20, complete standard-profile reachability, original-game execution, Java/RNG-cursor
  parity, and winning strategy are outside M10. Strategy tuning and seed search are excluded. M11 Silent
  may be planned only after M10 acceptance; no implementation follows automatically.

### Opt-in usage

M10 is explicit; existing defaults remain M9. Rust callers use
`RunSession::new_m10(config)` or `RunBatch::new_m10(configs, workers)`. Python
callers can opt in with `RunEnv("ironclad", 0, profile="m10-ironclad-heart", partition="test")`;
`NativeRunSession` accepts the same `profile` keyword. The default profile for
`RunEnv`/`NativeRunSession` and the existing `RunSession::new` remains M9.

Run the Python example with its explicit profile/history/seed/budget options:

```sh
UV_PROJECT_ENVIRONMENT=target/m9-gym-dev-venv uv run --locked --no-sync python -B examples/run_m10_gymnasium.py --profile m10-ironclad-heart --history time-eater-unseen --seed 11 --budget 8192
```

The separate native `m10_heart` CLI is fixed to M10 and does not accept profile,
character, ascension, or history options.

## Ironclad red card identities and source support

The profile contains all **75** ordinary red card kinds (3 Basic + 20 Common + 36 Uncommon + 16 Rare;
72 nonbasic). These are typed native identities with source-backed base/upgraded mechanics, not only
vocabulary rows. The full roster makes the declared boundary auditable:

| Rarity | Kinds |
|---|---|
| Basic (3) | Bash; Strike; Defend |
| Common (20) | Anger; Armaments; Body Slam; Clash; Cleave; Clothesline; Flex; Havoc; Headbutt; Heavy Blade; Iron Wave; Perfected Strike; Pommel Strike; Shrug It Off; Sword Boomerang; Thunderclap; True Grit; Twin Strike; Warcry; Wild Strike |
| Uncommon (36) | Battle Trance; Blood for Blood; Bloodletting; Burning Pact; Carnage; Combust; Dark Embrace; Disarm; Dropkick; Dual Wield; Entrench; Evolve; Feel No Pain; Fire Breathing; Flame Barrier; Ghostly Armor; Hemokinesis; Infernal Blade; Inflame; Intimidate; Metallicize; Power Through; Pummel; Rage; Rampage; Reckless Charge; Rupture; Searing Blow; Second Wind; Seeing Red; Sentinel; Sever Soul; Shockwave; Spot Weakness; Uppercut; Whirlwind |
| Rare (16) | Barricade; Berserk; Bludgeon; Brutality; Corruption; Demon Form; Double Tap; Exhume; Feed; Fiend Fire; Immolate; Impervious; Juggernaut; Limit Break; Offering; Reaper |

Native support covers play, upgrade/removal, card-instance state, copy/growth/cost/disposition,
generated Statuses, selections, autoplay/repeats, exhaust callbacks, and source-attributed HP-loss,
draw, Block, attack, and turn hooks. Infernal Blade uses the complete **28 eligible nonbasic Attack**
support (excluding Feed/Reaper), with actual generated-card ownership, ordinary turn cost, source-local
RNG, upgrade, and hand-overflow behavior. This does not generalize handlers to other characters or
claim all shared generators complete. See the [M10 research note](../research/2026-10-24-m10-act4-and-ironclad.md)
for named Act 4/Heart findings and the [approved M10 plan](../superpowers/plans/2026-10-24-m10-ironclad-heart.md)
for detailed implementation scope.

## Exclusive items, events, and captured offers

The 11 Ironclad-exclusive relics are **Burning Blood, Black Blood, Magic Flower, Red Skull, Paper
Phrog, Champion Belt, Charon's Ashes, Self-Forming Clay, Brimstone, Mark of Pain, and Runic Cube**.
The three exclusive potions are **Blood Potion, Elixir, and Heart of Iron**. Each has real
acquisition/use and applicable combat/turn/healing/selection ownership; catalog presence is not proof.
Brimstone is shop-only outside finite normal drops. The M10 boss-choice capture uses declared old/new
rows, with owned or ineligible choices disabled.

Captured offers are one Common plus two distinct Uncommons for ordinary rewards, three distinct Rares
for a boss reward, and one each Common/Uncommon/Rare in shops. M10 samples from each declared complete
eligible red support using owned scoped RNG. These are **captured-offer rules**, not standard
rarity/rare-pity/shop algorithms. Events cover only qualified shared branches, source eligibility, and
actual card/item acquisition/upgrades. No Ironclad-only event is invented; full shared-event/standard-
pool closure is not claimed.

Finite relic stock is **19 consumed offers**, correcting superseded 18-acquisition arithmetic: at most
15 Act 1–3 elite + 3 opened-chest + 1 Act 4 elite offers. Opening a chest consumes its row even when
Sapphire or Proceed skips the relic. The M10 stock includes the source-qualified nineteenth normal-
drop kind, Orichalcum. No refill, refund, duplicate fallback, or route pruning is allowed; Brimstone,
boss/shop/event sources are outside the finite sequence.

## Keys, Act 4, and Heart settlement

- **Recall** is a real campfire choice (+5 once, no rest/Smith). **Sapphire** consumes its chest relic
  row at OpenChest, grants +5, and retains actual optional Proceed. **Emerald** requires Victory at the
  real burning Elite; a marker or Death is insufficient.
  Only the first distinct key pays +5; no duplicate reward is manufactured.
- Preserve M9's missing-key Act3NoKeys ending and all-key pre-door **Act 3/floor 50** cutoff. M10 alone
  enables the fourth actual all-key choice and **Rest51 → Shop52 → Elite53 → Heart54 → TrueVictoryRoom55**;
  the choice is not another floor. Transition requests ordinary maxHP healing, subject to prevention,
  and adds no door bonus. Rooms and decisions are real, not direct jumps or auto-selected screens.
- Shield/Spear uses its actual Act 4 Elite source, actor pair, captured reward, and source-owned
  completion; do not inherit ordinary Act 3 reward contents. Non-endless Heart publishes no ordinary
  boss gold/card/potion/relic screen or extra chest. HeartVictory is not combat Death: it requires
  actual boss Victory settlement, explicit **Proceed**, TrueVictoryRoom55 entry, and terminal settlement.
- The conditional true-route reward is **215** only if events happen: floors 1–50 (+50), three
  distinct keys (+15), Act 4 entries 51–55 (+50), and HeartVictory (+100). No door/readiness/animation
  bonus or unobserved route reward is included.
- The accepted typed-event policy unit diagnostic sums those supplied rows to 215; it does not execute
  the route. Separately, the supplied native key/Act 4/Heart chain's actual final Proceed yields 110.
  Neither the policy diagnostic nor that supplied chain is normal-start victory evidence.

## Native example and evidence actually observed

The fixed public-only example is a tool demonstration, not a strategy benchmark:

```sh
cargo run -p spire-core --example m10_heart --locked --offline -- --seed 7 --scenario all
cargo run -p spire-core --example m10_heart --locked --offline -- --seed 11 --scenario all
```

The native CLI has a fixed M10 Ironclad A0 profile; callers cannot set profile, character, ascension,
or boss history. It accepts only seed (`u64`), budget (`0..=8192`), and scenario (`complete|death|all`)
options; the default budget is 8192. Normal-start scenarios run each of the three fixed M9 boss-history
variants. `all` also includes the nonattacking diagnostic; `complete` means only “omit that diagnostic,”
not a claim of route completion; `death` selects only the nonattacking diagnostic. Budgets 0/1 produce
an unfinished-caller error, 8193 is rejected, and an unsupported scenario is an argument error. Genuine
gameplay Death exits successfully; unfinished budget is not Death, success, or Gym truncation.

All six frozen native normal-start seed/history episodes remain genuine early losses: seed 7 (all
histories), Death action 177/floor 11/reward 10; seed 11 (all histories), Death action 110/floor 7/reward 6.
Separate default masked-caller CLI traces ended in Death after 27 and 67 actions; these are different
caller episodes, not strategy-quality measurements or full-route proof. The nonattacking native
diagnostic is Death action 10/floor 1/reward 0. These are valid gameplay outcomes, not tool errors or
Heart-victory evidence. No normal-start no-key ending, Shield/Spear, Heart, or HeartVictory traversal
was observed. The supplied native key/Act4/Heart chain is separate diagnostic evidence: its actual
`finalProceed` reward is 110, **not** the typed-event policy unit diagnostic's 215 and not a normal-start
route result. Neither that chain nor a synthetic Act4Ready classifier substitutes for normal-start
traversal.

### Numeric, session, replay, and native batch contracts

M10 is an explicit opt-in over the same native RunSession owner. Identifiers:

| Contract | M10 value |
|---|---|
| Profile | `M10IroncladHeart` |
| Observation | `spire-gym-run-observation-v2` |
| Replay | `spire-gym-run-replay-v2` |
| Rules | `spire-gym-native-m10-rules-v1` |
| Content | `spire-gym-m10-ironclad-a0-captured-v1` |
| Seed adapter | unchanged `spire-gym-run-u64-le-v1` |

Seeds are unsigned 64-bit and adapted from little-endian bytes into the same owned RNG
root/partition/ordinal scheme. Initial `None` uses ordinal zero; continuation advances the ordinal;
explicit reseed starts at zero. Replay records semantic actions, rewards, terminals, typed identities,
and portable fault summaries—not snapshots—and M10 replay-v2 is never silently read as M9.

Numeric v2 is separate from frozen M9 numeric-v1. It adds entitled card values/cost/playing flags,
enemy side/hit count, public facing/Surrounded/Heart allowance, optional-selection bounds/operation,
and known-draw aliases only to represented public card rows. Optional values have known masks;
occupied and padding differ. Hidden draw contents/order, future offers, RNG/cursors, private handles,
and unentitled values are never known-zero. `m10_default` initially matches M9; no truncation/paging/
variable spaces.

Native `RunBatch` retains ordered M10 sessions on one persistent scheduler. Whole-input reset/step
preflight is atomic. Valid dispatched work may partially commit: all workers quiesce, each owner keeps
its accepted prefix/first indexed fault, undelivered peer rows are marked delivery-lost, and
diagnostics/reproduction remain available even with recording disabled. No rollback, retry, autoreset,
or Python vector facade. Reset capacity fault and idempotent close are verified. Budget exhaustion is
unfinished/error—not Death or Gym truncation; gameplay Death remains valid.

### M9 compatibility amendment: rules v2, frozen v1 ABI

Correct shared random-spot insertion from `n+1` gaps to the actual `n` positions changes seeded M9
outcomes. Current M9 rules identity is `spire-gym-native-m9-rules-v2`; replay wire schema remains
`spire-gym-run-replay-v1`, numeric observation-v1/action vocabulary and fingerprint
`0893bce86e4b3abf11aa384c501503a91ae37f688ec605ab40b814ad365cb999` are unchanged, as is the HMAC
algorithm/root adaptation. Current binaries reject preserved `m9-rules-v1` records at preflight.
Old records/outcomes remain preserved and require their old source/binary for historical replay. No
silent compatibility mode, record rewrite, unchanged-old-trace claim, or second legacy resolver.

Current corrected-rules M9 traces: seed 7 (all histories) Death action 697/floor 42/reward 41; seed 11
Awakened One Death 711/floor 41/reward 40, Time Eater Death 814/floor 47/reward 46, Donu-Deca Death
831/floor 47/reward 46. Preserved historical seed-7 v1 is action 778/floor 47/reward 46. Keep them
separate.

## Verification and acceptance status

### Integrated and controller-verified through the Task 29 matrix

- Task 26 native batch tests passed **5/0**; `spire-env` passed **102/0**. Its workspace snapshot had
  **1,401 passes** and exactly the seven preserved historical core failures, with every other target
  passing. These are bounded results, not a green-workspace claim.
- Task 27's source-matched, refreshed binding passed **40 M10 tests** plus **195 existing environment
  tests** (**235 total**); touched-file Ruff lint, Ruff format, and `ty` checks passed. The controller
  verified the imported module/build/source-artifact provenance and Torch absence. The 195 existing
  tests alone were M9/pre-body coverage; the 40 M10 tests are the new profile and ownership coverage.
- Task 28 added **16 Python environment tests**; the fresh source-matched environment passed **251
  tests total** (**235 prior + 16 Task 28**). Changed-file Ruff lint, Ruff format, and `ty` checks all
  passed. The actual optional Elixir selector is verified: fixed seed 11/Awakened One history after 62
  actions at floor 5, owned potion slot 1, and legal `UsePotion` leading to an optional selector with
  min 0/max 5, five eligible cards, zero selected, and legal `ConfirmSelection`.
- The normal Confirm0 branch is preserved in v2 record/replay; exported arrays match and the potion is
  consumed exactly once. The accepted-child Gym tuple fault retains the original unprintable
  `BaseException`, nested-busy rejection, first fault and accepted prefix; selector replay remains
  available and no retry occurs. These are test-only additions: no production change, new API, strategy
  change, seed, or state injection. Task 28 is fully verified; this selector is not unobserved.
- Task 28's typed-event policy unit diagnostic reaches215 from supplied rows only; it is separate from
  the source-backed supplied chain's final Proceed reward110 and neither proves normal-start route
  traversal. The six fixed native normal-start traces remain Death at actions110/177; the separate
  Python masked-caller Death traces at27/67 are not strategy-quality or full-route evidence.
- The final **607-file** controller preservation matrix is source-stable and records workspace
  **1,402 passed / exactly 7 failed** (core **1,149/7**, the same seven named historical failures
  below). Focused native filters passed: M10 **453/0**, M9 **32/0**, M8 **115/0**, M7 **16/0**, M6
  **95/0**. Runtime was 21.93s. All ten native examples plus M9 seed 7, M10 seed 7, nonattacking, and
  expected-error cases for budgets 0/1/8193 and invalid scenario matched their expected exits
  (**17 commands**). Enemy/power reference tests passed 27/20 and the power builder regenerated 133
  records. The accepted citation fix leaves all 133 power records, citation arrays/grades, null IDs,
  and non-citation fields unchanged; only the local research-source snapshot hash was added. The source
  remained unchanged after verification.

The seven preserved workspace failures in the verified matrix are:

1. `full_cards::artifact_prevents_one_poison_application_without_erasing_existing_poison`
2. `full_cards::catalyst_adds_twice_the_current_poison_and_exhausts_but_absence_is_noop`
3. `full_cards::catalyst_permanent_card_state_carries_its_exhaust_flag`
4. `full_cards::poison_ticks_at_afflicted_enemy_start_ignoring_block_then_decrements_and_can_kill`
5. `full_cards::silent_deadly_poison_applies_base_and_upgraded_amount_for_one_energy`
6. `full_cards::silent_poisoned_stab_resolves_attack_before_base_or_upgraded_poison`
7. `full_enemies::blue_slaver_and_two_fungi_encounters_construct_real_actors_across_ascensions`

### Accepted profile and declared limits

- The implementation and native preservation matrix are complete. ONE fresh independent GPT-6 Luna
  MAX review (`ses_e8bdccfd2ffe9wlTc8FVR7oHE3`) found no Critical, Important, or Minor findings and
  satisfied all five review-focus classes. The controller explicitly ruled all six declined categories
  within the approved profile. Both checkouts retained all 608 reviewed file hashes at source root
  `92c07f4a38de0f1a2d5a76bc5d8a96167bb876d87db04aa66cc5b8fa8eb200e1` before administrative
  acceptance updates. The reviewer independently passed M10 core453, environment102, report12, and
  Python251. M10 `.32.2`/`.32` are normally closed; no product/test/data/runtime changes followed review.
- Full normal-start traversal into Shield/Spear, the Heart, or HeartVictory remains unobserved; this is
  an explicit profile limitation, not evidence of a win or a Task 28 test gap. A zero `known_draw`
  high-water means not observed, not a positive bound.
- The accepted power-reference fix restores annotation-driven generation of the Task 19 facing source
  metadata. All 133 power records—including citation arrays/grades, null identities, and non-citation
  fields—remain unchanged; only the local research-source snapshot hash was added. Controller checks
  passed builder133, power reference20, and enemy reference27. This is reference-integrity evidence,
  not mechanics proof.

The full Python collection has known Torch/readiness and historical diff-schema or metadata-fixture
failures. Do not install Torch or rewrite baselines to manufacture a green label; report actual results.
Source findings use named members of an owned archive with unverified build/distribution channel. No
original game execution or Java RNG/cursor parity is claimed.

**Acceptance state:** M10 is accepted at its declared captured profile after ONE clean independent
review and exact-source verification. The six normal-start traces remain early gameplay Deaths;
normal-start HeartVictory remains unobserved. M11 planning-only `.33.1` is open/unassigned for ONE
written Silent combat plan and actual user review before implementation; no product start follows.
