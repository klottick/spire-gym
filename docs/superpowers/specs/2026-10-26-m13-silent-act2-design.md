# M13 Silent through Act2 design

Date: 2026-10-26. Status: **proposed finite design for the combined planning/review/approval gate**. No M13 product implementation is authorized until the primary session approves the reviewed design and plan.

## Intent and established decisions

M13 carries a normal-start Silent A0 Run through Act2, including real character/Act2 interactions, then stops after actual Act2 boss rewards and Act3 initialization. The approved roadmap requires from-start Acts1–2 and forbids later-act state injection. M12 is complete and accepted at its narrower finite boundary: normal-start Silent through Act1, ending at Act2Ready.

The user selected one shared Rust Run/combat engine and rejected copied character progression. That architecture is established. This proposal chooses the M13 finite content rows, tests, public identity, and stop condition; those details remain proposals for user approval.

M12 is accepted and merged at `afacd0934c55ceac159e684cfcb3f2565e7cb6b6`. Its accepted evidence records six seed-7/11 Act1 boss routes reaching Act2Ready, natural Death controls, public native session/replay/batch agreement, and 1,529 passing workspace tests with the one historical Blue Slaver setup failure. M12’s profile, schema and cutoff remain frozen.

## Current owners and proposed approaches

The shared lifecycle already owns short Neow, generated maps, Act1 and Act2 routing, room/event/reward/shop/chest custody, combat setup/entry/settlement, and the Act1→Act2→Act3 transition. Core sources include `crates/spire-core/src/run/rooms/act1.rs`, `act2.rs`, `transition.rs`, `rewards.rs`, `shop.rs`, and `boss.rs`. `M12SilentAct1` supplies the existing finite Silent card/item rows and Act1 state. M6/M7 supply shared Act2 route mechanics, but M7’s card/relic/boss rewards are Ironclad captures and cannot become Silent offers. `spire-env` owns the one Run session, observation, replay, and batch boundary.

| Approach | What changes | Tradeoff and decision |
|---|---|---|
| A. Add a distinct finite `M13SilentAct2` profile on current owners | Extend shared Run/Act2 progression and add versioned M13 session, observation, replay, and batch identities; reuse finite M12 Silent content through M13-owned Act2 hooks | Recommended. It composes M12’s normal-start Act1 and M6/M7’s shared Act2 path while preserving accepted M12/M9/M10 contracts. The finite capture and explicit exhaustion boundaries remain visible. |
| B. Extend `M12SilentAct1` through Act2 | Widen M12 room, codec, replay and profile boundaries | Rejected because it rewrites M12’s accepted Act2Ready boundary and versioned identity. |
| C. Reuse only M7’s later-act captures or defer public Run integration | Inject an Act2 start or stop before the M13 public caller/replay path | Rejected because it does not establish M13’s from-start Silent outcome or the evolving public caller path. |

## Goals

- Start with canonical Silent A0 resources and real short Neow, then run the same Run through actual generated Act1 and Act2 maps.
- Select each Act1 and Act2 boss alternative before construction through typed initial boss-history inputs; validate the generated public map icons and actual entered `CombatRecord` encounters.
- Support a finite, explicitly captured Silent content slice through every advertised M13 state. Extend the schema for the actual Act2 actors, statuses and public Act3Ready map icon.
- Exercise actual Act2 interactions across the already implemented Poison, Snecko Skull, Shiv/Accuracy, discard, Wound, retain and Nightmare/copy mechanics. Keep those interaction diagnostics distinct from normal-start outcome evidence.
- Preserve real shared ownership, finite offer eligibility, acted-on source cursors, boss-reward children, reset/replay/batch semantics, and fault custody.
- Stop at `Act3Ready`, after actual Act2 boss rewards, mandatory boss-relic children or confirmed skip, once-only act healing, and actual Act3 map/boss-order initialization.

## Non-goals and anti-goals

M13 is not a full or standard-random Silent pool, full Silent A0 completion, Acts 3–4 gameplay, Heart, Ascension 1–20, Python binding/facade/extension delivery, original-game execution, or Java RNG/map parity. M15 remains responsible for completing all Silent A0 cards, exclusive relics/potions, events and interactions, or recording a specific blocker. Its coverage ledger must enumerate the Silent A0 catalog members and source owners still outside M11/M12/M13, then accept each supported row or state the exact runtime/source blocker; M13 does not mark the remaining catalog complete.

Do not clone Run progression, Act2 room logic, combat resolution, settlement, or the public Run facade. Do not borrow M7 Ironclad cards, Black Blood or Magic Flower for Silent offers. Do not initialize a Run at Act2, inject later cards/keys/resources/maps, mutate a map or boss history after Run construction, change enemy values, search for easier seeds, tune a policy after observing fixed-run outcomes, turn Death/fault/budget exhaustion into Act3Ready, refill exhausted captured rows, or claim win rate, full pool or original-runtime parity.

## Exact finite capture proposal

`M13SilentAct2` keeps M12’s 19 permanent Silent card kinds, four generated/status kinds (`Shiv`, `Dazed`, `Burn`, `Slimed`), 13 relic kinds and four potion kinds. It adds `Wound` as an actual generated Act2 combat card: Book of Stabbing and Taskmaster can create it. Dazed is already generated by the captured enemies. The declared A0 Act2 route has no captured curse producer, so curses remain outside the profile. The 19/13/4 rows remain the previously accepted M12 data; they are not a complete ordinary pool or proof of standard rarity rules.

M13’s Act2 route captures the shared M6/M7 finite mechanics:

- Six ordinary encounter IDs: `SphericGuardian`, `ThreeByrds`, `CenturionAndHealer`, `Snecko`, `ChosenAndByrds`, and `ThreeCultists`. `CenturionAndHealer` is one encounter ID for the Centurion/Mystic pair.
- Three elite encounter IDs: `GremlinLeader`, `Slavers`, and `BookOfStabbing`.
- Three boss encounter IDs: `Champ`, `BronzeAutomaton`, and `Collector`.
- The public Act2 enemy vocabulary includes the six ordinary actors; `GremlinLeader` plus its five child species (`GremlinMad`, `GremlinSneaky`, `GremlinFat`, `GremlinShield`, `GremlinWizard`); the three Slavers actors (`SlaverBlue`, `Taskmaster`, `SlaverRed`); `BookOfStabbing`; and the five boss/summon identities (`Champ`, `Collector`, `TorchHead`, `BronzeAutomaton`, `BronzeOrb`). This is 21 distinct Act2 enemy identities. Existing shared power/intent rows cover the observed Act2 mechanics (`artifact`, `barricade`, `flight`, `hex`, `confused`, `minion`, `angry`, `entangled`, `painful_stabs`, `stasis`, and actual Act2 intents); M13 must still test every emitted kind.
- The finite M13 event vocabulary is `Neow`, `UpgradeShrine`, `Purifier`, `AncientWriting`, and `Joust`: it retains M12's reachable Act1 rows and adds the captured Act2 rows. Act2 question rooms publish Ancient Writing first, then Joust when the shared 50-gold eligibility check permits it or Purifier otherwise; later questions use the actual shared Monster path. The M13 mapper and real-room tests cover both conditional second-question events; M12's mapper and vocabulary remain unchanged. No other event row is added.
- Ordinary card rewards reuse the three captured M12 triples in order by Act2-local settled combat visit; the boss card reward is Nightmare plus Skip. Act2 potion reward order is Fire, Strength, Weak by Act2-local settled combat visit.
- Eight captured elite relics reuse the existing M12 Silent row: Lantern, Pear, Mango, Snecko Skull, Toy Ornithopter, Strawberry, Anchor, and Blood Vial. The Act2 cursor is act-local and skips already owned relics. A successful offer's next cursor is committed with its published reward screen, even if the player proceeds without selecting the relic. Exhausted lookup returns an error before any new cursor can commit; it preserves the current cursor and retained source fault, with no replenishment or duplicate grant.
- An Act2 shop uses its own `Act2Progress.shop_visits`: the first visit uses M12’s first finite shop row (Quick Slash / Reflex / Catalyst, 50/75/75; Snecko Skull if unowned; Dexterity Potion for 50), later visits use M12’s later row (Deflect / Tactician / Nightmare, 50/75/150; Bag of Preparation if unowned; Fire Potion for 50). Existing gold checks, ownership, potion-belt custody, purge and Leave remain shared.
- Ordinary chests reuse the actual Strawberry / Anchor / Blood Vial capture with Sapphire exclusion. Act2 boss treasure reuses Empty Cage / Slaver’s Collar or explicit skip/confirmation, without Black Blood. Boss/room proceeds wait for mandatory child ownership and transition checkpoints.

The capture reuses actual existing content/mechanics only. M13 adds public support for Wound and cross-act room/reward plumbing, not a larger invented card pool. This exact finite proposal, including exclusions, needs approval.

## Character/Act2 interactions

The finite M13 tests exercise these source-grounded combinations against actual Act2 encounter setups:

1. Poison and Snecko Skull against Act2 Artifact enemies (Spheric Guardian and Bronze Automaton): each actual setup starts with three Artifact, so the first three Poison applications are blocked and consume those charges; the first unblocked base Deadly Poison then applies five plus Snecko Skull's one, for six.
2. Blade Dance/Cloak and Dagger Shivs with Accuracy against Byrd Flight and the shared normal-hit path. An unupgraded Shiv plus base Accuracy is eight pre-mitigation; while Flight is positive the Byrd loses four HP, and three successful hits reduce Flight 3→2→1→0. This is a two-HP-loss increase over an unbuffed Shiv while flying. Exercise Shivs from both Blade Dance and Cloak and Dagger.
3. Survivor/Dagger Throw manual discard listeners plus actual Wound generated by Book of Stabbing or Taskmaster. Wound is separately shown to be selectable and discardable; discarding it does not trigger Reflex or Tactician. Discarding the Reflex instance itself triggers its base two-card draw, and discarding an explicitly upgraded Tactician itself grants two energy (base Tactician grants one).
4. Well-Laid Plans retain and Nightmare capture/copy/hand ownership in Act2 combat.
5. Act2-local card/potion reward visit ordering, elite cursor ownership/skip/exhaustion, and first/later shop stock through the actual Run room owners.

Supplied-combat or room diagnostics prove these bounded interactions only; they do not replace the six normal-start boss paths. Full card/exclusive-item/event coverage remains a named M15 closure obligation or blocker.

## Shared core and exact public boundary proposal

- Add `RoomScenario::M13SilentAct2`; keep `Run::new_room_scenario(config, profile, rng, scenario)` and all shared lifecycle/combat owners. M13 requires Silent A0 and short Neow. Extend the current `uses_captured_act1`/`uses_captured_act2` profile gates so M13 uses Act1 and the full Act2 path through floor 33, but does not start Act3.
- Add `m13_content.rs` for M13’s act-local reward/shop selection and explicit exhaustion messages. Reuse the accepted M12 content table/functions where appropriate; do not fork map, enemy, card, reward, or combat rules.
- Keep the one `RunSessionConfig.boss_history` field. Add typed `RunAct1BossTarget { Guardian, Hexaghost, SlimeBoss }` and `RunAct2BossTarget { Champ, BronzeAutomaton, Collector }` selectors and one additive composite `RunBossHistory::M13SilentAct2 { act1, act2 }` variant. Existing M9/M10 and M12 enum variants, M9 compatibility alias and old text/JSON spellings remain intact. M13 accepts only the composite variant; M12 continues to accept only its existing Act1 variants. Encode the new composite in replay JSON as `"boss_history":{"m13-silent-act2":{"act1":"guardian","act2":"champ"}}`; old histories remain their current string spellings. Reject a partial, unknown, or mismatched composite. Before constructing the Run, write the selected targets into `CreationProfile.seen_bosses[0]` and `[1]`. Leave Act3 history at canonical experienced-profile defaults; M13 does not override it.
- Add `RunProfile::M13SilentAct2`, `RunSession::new_m13(config: RunSessionConfig) -> Result<RunSession, RunSessionError>`, and `RunBatch::new_m13(configs: Vec<RunSessionConfig>, workers: usize) -> Result<RunBatch, RunBatchError>`. The shared session still owns tokens, reset, fault custody, replay and public actions.
- Add `m13_run_observation_spec(caps: &RunCapacities) -> Result<RunObservationSpec, RunSessionError>` and `encode_run_m13(decision: &PublicDecision, map: Option<&PublicMap>, caps: &RunCapacities) -> Result<RunObservation, RunSessionError>`. Use the existing row/group/schema packagers with a distinct M13 finite vocabulary; do not copy the observation encoder. Add Wound, the 21 Act2 enemies, 12 Act2 encounters, `Neow`/`UpgradeShrine`/`Purifier`/`AncientWriting`/`Joust`, the Act3 map-only boss icons and `Act3Ready` to this M13 vocabulary. Continue to reject foreign cards/items/encounters/operations and hidden state.
- Give M13 new identities: `spire-gym-m13-silent-act2-observation-v1`, `spire-gym-m13-silent-act2-replay-v1`, `spire-gym-native-m13-rules-v1`, `spire-gym-m13-silent-a0-acts1-2-captured-v1`, profile `M13SilentAct2`. M13 Act3Ready is a healthy nonterminal zero-action boundary with no completion reward; reset and close remain available. An attempted Act3 action rejects before mutation. M12’s Act2Ready boundary and all prior schema/replay identities/fingerprints stay unchanged.
- No Python facade or extension refresh is included. Native source acceptance is not Python-delivery or original-runtime proof.

## Normal-start evidence proposal

Freeze one caller that uses only public observations, current public map topology, legal actions, visible cards/intents and offered choices before the fixed outcomes are run. The normal-start acceptance matrix contains six deterministic routes, declared now: seed 7 maps Guardian→Champ, Hexaghost→BronzeAutomaton, SlimeBoss→Collector; seed 11 maps Guardian→BronzeAutomaton, Hexaghost→Collector, SlimeBoss→Champ. This covers each Act1 and Act2 target for both seeds without expanding to all 18 pairings.

Every route starts with the canonical M12 Silent resources and actual short-Neow choice; the typed history selects map icons before construction. Assert the Act1 generated map icon, actual selected Act1 boss entry, real Act1 settlement and transition, the actual successor Act2 map icon, at least one ordinary Act2 combat and an elite entry, actual selected Act2 boss entry and combat record, actual boss reward/chest resolution, and the final Act3Ready state when the gameplay outcome reaches it. All routes must enter the declared Act2 boss to count as boss-coverage evidence. At least one normal-start route must record an Act2 boss Victory, resolve its real boss reward/chest and transition, and reach Act3Ready; if all fixed routes die after boss entry, that normal completion requirement remains unmet. A genuine Death after actual boss entry remains a real Death, not Act3Ready. A Death before boss entry leaves that boss-coverage requirement unmet; do not replace it with a fixture or seed search. Do not promise all six wins before observing the approved fixed runs.

Also retain separate nonattacking Death, invalid/stale action, wrong profile/history, zero/small caller budget, and accepted delivery-fault diagnostics. Replay must reproduce public decisions, maps and reward deltas; batch and single sessions must agree across worker counts. If finite elite stock is exhausted after actual owned-item filtering, preserve the exact source fault and the existing cursor: successful published offers have already advanced it, while the failing lookup itself cannot advance it; do not grant a replacement or disguise it as Death. A workspace run is preservation evidence, not a green claim; M12's historical baseline remains qualified.

## Open to the implementer

The plan may settle small code-local choices such as helper placement and normal caller tie-breaks. The six boss-pair mapping, 19/13/4 finite catalog, named interactions, M13 identities, no-injection rule, and M15 remaining-coverage obligation are fixed by this proposed design and are approval items for the primary session.
