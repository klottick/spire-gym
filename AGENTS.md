# Spire Gym: current agent guidance

## Approved replanning and new execution constraints (2026-10-22)

- The user approved the36-milestone roadmap in
  `docs/superpowers/specs/2026-10-22-narrow-gym-milestones-design.md`.
  Final scope remains four characters/A0–20/Acts1–4/Heart. M1 authored IroncladA0
  damage/Block/Weak/Vulnerable combat is now accepted. Earlier broad family-wave
  scheduling notes below are history, not authority to dispatch more such work.
  The revised Beads migration/M1 plan is
  `docs/superpowers/plans/2026-10-22-roadmap-migration-and-m1-combat.md`;
  user-approved for native execution ("I approve, please do this"). Migration is
  applied: M1–36 are `spire-dxo.23`–`.58`; M1 `.23`/`.23.1` are closed after one
  isolated independent review with no findings, exact reviewed source and fresh
  checks. User approved
  `docs/superpowers/plans/2026-10-23-m2-shared-combat-mechanics.md` ("I approve");
  M2 `.24`/planning `.24.1`/implementation `.24.2` are now closed after one
  independent GPT-6 Luna milestone review in its own materialized worktree found
  no findings. Controller verified all201 reviewed files unchanged and fresh
  M1 5/0 + M2 12/0 + shared8/0/diffcheck. Final core344/7 retains exactly seven
  pre-M2 Poison/Catalyst/Artifact and Slaver/Fungi failures/messages; relevant old
  potion consumer passes after the approved invalid-Chomp-override fixture fix.
  Consumers43/0, other integrations21/0 and both examples pass; no workspace-green,
  family/full-run/Python-extension/original-runtime acceptance. Profile:
  `docs/milestones/m2-combat.md`. M3 `.25` and actual `.25.1` are open/unassigned
  for just-in-time planning, not product start. Native actor remains
  `opencode-session-recovery-20261021-v1`. M2 execution ledger:
  `.superpowers/sdd/2026-10-23-m2-shared-combat-mechanics/progress.md`.
  User permits concurrent GPT-6 Luna fresh eyes, with mandatory separate source
  worktree/build directory and no overlapping edits; one final milestone review.
  Original .7 owner safely
  released its own claim; old unfinished components retain scope/history/edges
  under `roadmap-v2-backlog`. No force release, rollback or overlapping ownership.
  M1 external5/0, accepted consumers43/0, other integrations21/0; core335/8 exactly
  matches eight pre-M1 draft failures. No workspace-green/full-run acceptance.
  Runnable profile/limits: `docs/milestones/m1-combat.md`. Current ledger:
  `.superpowers/sdd/2026-10-22-roadmap-migration-and-m1-combat/progress.md`.
- Every future subagent, including reviewers/investigators, MUST have a dedicated
  worktree and unique build directory. Do not dispatch into this controller's
  or another agent's live checkout. Materialize the reviewed current public
  tracked/untracked source baseline—HEAD alone lacks this uncommitted simulator.
  Workers own coherent integration in their worktrees; controller integrates
  complete candidates sequentially. No blind merges, commits or cleanup implied.
- Tests must be fast, deterministic and behavior-focused: small valid fixtures,
  exact game-rule values, bounded scenarios, no incidental debug/log goldens,
  duplicated per-card rule tests, sleeps or network setup. Do not weaken useful
  tests merely to meet a count/time target. Diagnose warm M1 commands above5s or
  warm workspace commands above30s, reporting build time separately; never use
  flaky wall-clock assertions as correctness gates. Follow the plan's actual
  pre-change/candidate baseline and conditional accepted-consumer checks.

## Product and current priority

- Deliver a headless gym: authoritative Rust simulation, native batching, thin
  Python/Gymnasium interfaces, public observations, legal actions/masks, rewards,
  reset/termination, replay, focused tests and small rollout examples.
- Actual ML code is outside this repo: no models, optimizers, training pipelines,
  learner checkpoints or learned-agent evaluation infrastructure.
- Read `README.md`, `docs/scope.md` and `docs/work-status.md`. Beads `spire-dxo`
  is the full-run gym PoC through Acts 1–4 and the Heart; no separate Act 1-only
  release gate. All four standard fully unlocked characters and Ascension 0–20
  are selected for this PoC, A20-primary, including normally reachable content.
  `spire-dxo.1` defines architecture and acceptance; do not re-ask or silently
  narrow the character/ascension scope.
- Reuse the implemented supplied-entry Ironclad A0 combat, native batching and
  Gymnasium facades. Full-run gameplay is not implemented. The long-term target
  remains four standard characters, Acts 1–4, Ascension 0–20, A20-primary.

## Beads and work ownership

- Use only the existing shared tracker, with your allocated actor:
  `/home/klottick/.local/bin/bd -C /home/klottick/repos/spire-lab ... --json`.
  Do not initialize another `.beads`, database, service or worktree-specific board.
- Before writes, check `context --json` and `ping --json`: database `spire`,
  project UUID `a61e792f-9730-47c0-b262-7dac036d6bf5`. Do not read credentials or
  alter service/configuration. Ordinary writes retain local Dolt history.
- Use `list --label poc-gym --status open,in_progress,blocked --json` for the
  current workstream, then `ready --label poc-gym --json` for ready tasks.
  Claim an assigned task normally; no force/reclaim/impersonation or overlapping
  ownership. In-progress means actual current work, not retained custody.
- Beads is authoritative for task state. Status docs are dated summaries, not
  duplicate task lists. Completed reviews do not establish source acceptance.
- `backlog-full-scope` is the broader specification backlog, not automatically
  required PoC work. `deferred-verification` is paused custom-verifier work.
  All unfinished historical full-spec, reference/upstream and runtime-investigation
  lanes are deferred and labelled `historical-backlog`, not pending dispatch.
  Outside the PoC, `maintenance-gym` and `preservation-gate` identify the two
  genuine follow-ups; do not reactivate archived research to fill the work queue.
  Do not resume historical workers/reviewers or replay whole audits from an old
  allocation. Delegate only with current user/applicable instruction authority.

## Full-run delivery and bounded verification

- `spire-dxo.3` holds one reviewed implementation plan/start decision;
  `.4`–`.22` cover the native run/gameplay/API/replay/batching and final acceptance
  components. Their Beads contain scope, exclusions, checks and dependency edges.
  Blocked/unassigned components are not author assignments or approved code start.
  The user approved `docs/superpowers/specs/2026-10-22-full-run-gym-poc-design.md`
  on 2026-10-22 ("the spec looks good"); `.1` is closed. The approved plan is
  `docs/superpowers/plans/2026-10-22-full-run-gym-poc.md`. The user approved the
  plan (“lgtm”) and authorized start (“You can start”); `.3` is closed and the 20
  planned dependency additions are applied. The user consented to a fresh worktree;
  this native-created `full-run-gym-poc-20261022` checkout is the primary workspace.
  Main and historical worktrees remain untouched. Fresh offline Rust baseline:
  169 passed, 0 failed before product edits. Do not re-ask plan/start/worktree consent.
  Registry foundation `.5` is accepted/closed after independent component review,
  one scoped fix/re-review and fresh controller verification (174 Rust passes,
  0 failures before Task2). Next `.4` returned from `luna-poc-run-20261022-v1`
  with reported A6 rounding concern corrected before review. Independent review
  required scoped RNG and live coordinator/continuation fixes; `.4` is now
  accepted/closed after fix1 scoped re-review and fresh controller Rust189/0 on
  unchanged reviewed bytes. One Minor shared-hook warning remains for later
  consumers/final review, not whole-run acceptance. Next `.6` runtime author
  returned incomplete/tests-only and released claim; bounded separate Sol arbiter
  identified four same-task migration slices. Original Luna author now implements
  first component-review fixround1 after authored multi-pick/CombatExit and
  OwnedCard/OwnedPiles compatibility. Public legacy3variant types/ONE resolver
  preserved; controller229/0 verifies the final correction but is not acceptance.
  First independent runtime review requires real hook dispatch (not event logging).
  Delivered separate Sol ruling also confirms full actor/intent/power and
  per-actor RNG representability gaps. Fixround1 originals are addressed after
  scoped review and controller236/0; new Important full-power refresh/validation
  mismatch resolved by scoped re-review2; `.6` accepted/closed with fresh
  controller239/0 and diffcheck on unchanged reviewed bytes. Historical TDD gaps
  remain explicit. Actual family mechanics now assigned ONLY to disjoint Luna
  authors .7cards/characters, .8items, .9allActenemies; no bridge/later assignment.
  Shared roots remain controller-integrated exact requests, no overlapping edits.
  Shared fault join is applied and
  preflight/budget failures corrected; do not replay the old failed-test task.
  Prior missing-red history stays honest. `.11` native representative
  map/world component accepted/closed after independent review+3scoped fixes/
  reviews and external2/run30/workspace223 on unchanged world files. Actual
  room/encounter/Neow/keys/Act4 effects stay required under later owners; no
  full-run or original-runtime claim. Minor native-consumer warnings go to final
  review. Three mechanics authors active; .10bridge/later components blocked.
  Their first returns were only a copy helper, metadata test and no-code context
  blocker. Same actors now implement bounded first executable handlers/tests and
  applicable shared patches under mechanics-first-join-decisions.md; registration
  is not a prerequisite to writing owned modules. Do not equate definition_only
  flags with effect proof, sample first slices as complete pools, or call family
  components accepted. Card prepare/commit now integrated with6card/140combat
  passes and earlier247workspace; author continues real character starter state.
  Native enemy join+borrow correction now pass4focused tests; combat144/2red
  includes active card Neutralize/Survivor additions. Enemy owner now implements
  actual Louse formation/AI/hit-trigger bodies/root patch now joined, but no Louse
  green:26remaining compile errors are character types. Original character root
  request withheld for checkpoint/player-end/Weak-oracle/rounding corrections;
  same cardauthor revises. Enemyowner returned actual selectedCharacter/cardRNG
  setup adapter, focused compile still45missing shared type/API errors, no test
  execution. Revised item and character root joins now applied. Initial character
  compile5errors: controller fixture duplicates/shadowing corrected; same author
  handles owned numeric type and exact differential/full-only legacy guard patch.
  First joined snapshot now freshly verified workspace276/0+diffcheck, with real
  cards14/enemies9/items10/fullBlock2 tests. Original block limit/CurlUp HP/fault/
  fullExit oracle failures corrected; history preserved. Same3authors now active
  next common-card+Miracle/Slimed handler, remaining3starter relic entry and Slime
  formation/AI/status/split batches under mechanics-next-native-batches.md.
  Next-batch drafts returned unexecuted/compile-blocked. Cardowner now authors
  actual applicable generation/energy/Copy wiring instead of returned prose-only
  request. Delivered separate Sol mechanics-native-join-ruling.md pins parent
  tombstones+appendchildren/stable slots, childRNG/queued continuation and typed
  failed-entry retainedCombat ownership. Item/enemy owners now author actual
  applicable source/test requests under that memo. Item entry return needs exact
  source-order patch/legitimate internal fault-state test correction now returned
  as FINAL unified diff and controller-integrated; no tests yet, itemowner idle.
  Card handoff ruling delivered (task-4-card-join-handoff-ruling.md); sameowner
  now prepares verified scratch/unified patch with correct overflow/fixtures.
  Enemy broad split patch failed; sameowner now prepares SMALL StageA typedintent/
  canonicaldamage wiring, explicit unsupported Split until still-required real
  StageB tombstone/RNG/queue replacement (task-6-slime-staged-join.md). No no-op
  success or family acceptance. Arbiter idle; no review/source census/duplicate
  insertion authority. Final card8file and SlimeStageA5file roots now integrated
  (both reverseapplycheck0); actual cards22/12fail,enemy16/3fail,items18/0 and
  nativeentryfixture0/1fail were corrected without altering mechanics to satisfy
  wrongoracles. Latest fresh controller workspace317/0+diffcheck on joined second
  snapshot (cards36/enemies19/items18/nativepostdrawfault), no authorsactive during
  run. Same3original authors now active real Silent/Shiv, health/acquisition/
  startup items and REQUIRED real SplitStageB per mechanics-next-after-types.md.
  Split remains explicitUnsupported until StageB; green guard isn't mechanics
  acceptance. Arbiter idle, roots controller only;317/0 predates these edits.
  Silent batch shared5file afterdrawselection/constructor join now applied and
  reversechecked; actual filter stops8pendingStageBfields beforetests. Source
  defects traced: QuickSlashdoc8/12 vs source6/9, Cloak missingPlayertarget,
  full12card called legacy11helper. Itemhook3file+B0enemydata5file joins now
  applied: actual controller cards37/2,enemy19/2,items23/0. Samecardowner fixed
  owned defects after behaviorRED; author focused39/0, workspace core261/2 fails
  only two real Split tests. B1owned replacement body and exact5sharedfile patch
  now integrated after hashes/applycheck/reversecheck0; controller enemy22/0 and
  freshworkspace328/0+diffcheck. Actual Split/stagedfault/childcursor behavior,
  not temporaryUnsupported. Same3owners active next Ironclad attack/potionfidelity/
  SlimeBoss batches under mechanics-next-after-split.md;328/0 predates these edits.
  Tracker .9 still originalactor in_progress at return; sameauthor reconciles OWN
  claim normally, controller did not release/impersonate. No family acceptance.
  Itemowner returned partialFirePlayer-source change with compileblocker; current
  LegacyCard reference gone, controller singlefocusedrun actualCurlUpRED5Blockvs0.
  Sameowner active narrowed finiteTHORNS/Bark rootpatch+producer/tests; required
  BloodPotion/FruitJuice/healmodifiers stillunfinished. Othercard/boss owners active,
  no duplicate sharedroot edits or repeatedknownfailedbuilds;328/0 prebatch only.
  SlimeBoss ownedbatch+exact3file roots now integrated/hash/reversechecked; enemy
  filter compile3pendingThorns errors,0tests. Bossowner normalreleaseconfirmedidle.
  Cardnumeric request WITHHELD stale16vsboss35/outputcapmiss30012/legacy1515
  executiongap; sameowner revises full-only completepath. Thorns request WITHHELD
  target-before-binding and wrongVulnerableTHORNS amplification (Silent522NORMAL
  only); sameitemowner corrects typedpatch+fixture20raw/block7->13HP, retains all
  powers. No candidate silentlyapplied or sourcefactsnerfed; card/itemauthorsactive.
  Corrected dynamicFINAL4file and THORNSFINAL nowintegrated, numericreversecheck0
  beforefinitejoin, overlappingcontexts preserveboss/fullabsorption. Controller
  actualcards44/1,enemy25/1,items24/1,numeric2/0,THORNSdiagnostic1/0. Remaining
  wrongoracles lethalcardinplayretention/bosscanonicalnames/oldFixed20assertion;
  same3originalowners active OWNTESTONLY corrections, no productnerfs. RealBoss
  descendants/settlement and FireCurlUp13HPloss pass, no currentworkspacegreen.
  Card/item OWNTESTONLY corrections returned: authorcards45/0 anditems25/0,
  exactlethalExitinplayowner/noduplicate andThornsPlayer/slotassertions preserved;
  controllerreadconfirmed, no productionchanges. Thoseowners idle/released.
  Boss testcorrection stillactive; awaitreturnthenONEfreshworkspace, no repeated
  focusedruns/newcontent while pending.328/0prebatch, no currentgreen/acceptance.
  Boss test-only canonicalintent/ownedGoop correction nowreturned; freshcontroller
  workspace343/0+diffcheck afterALLauthorreturns (cards45/enemies26/items25).
  Same3originalowners active DefectFrost, requiredBark/BloodPotion/FruitJuice and
  actualCultist/Act1JawA0-20 thresholds per mechanics-next-after-boss-thorns.md.
  Concrete nonA0setuprejection prioritized; bounded publicCultist/Jaw references
  supplied REFERENCE-informed only, no roundedAIoverride/ambiguousLouseA17guess,
  privateaudit or reducedprofile.343/0predatesnewedits, no familyacceptance.
  ItemBark/health return API/registration-only,nohandlers/tests. Controllerapplied
  ONLY2ID/registrysections afterpreimages; sharedmaxHPbehavior notapplied, b/tmp
  patchpathsneedrepair. Sameowner active executabletests/nativePotionUseContext
  handoff(task-5-health-potion-executable-handoff.md), no roomlabelguesses or
  duplicateinventory; FruitmustenqueueordinaryHeal aftermaxHP. Enemyascowner
  returned21asctests-only, compileblockedFrostexhaustivematches/noactualRED, idle
  untilcardFrostrootjoin. Card/itemactive,343/0prebatch/no currentgreen.
  Itemowner nowreturned2ownedtests BarkFire40 andBlood/Fruitpreparation; single
  focusE0004 Frostdiagnostic beforetests,noactualRED/handlers/useAPI. Itemactor
  releasedidle alongsideascowner, onlyFrostcardauthoractive. AwaitactualFrost
  sharedreturn/join then runbothsetsONCE, no unchangedredispatch/buildstorm or
  wildcardforeignfix;343/0prebatch/no familyacceptance.
  Frostownerreturned realcards/orbs+exact1diagnosticpatch (isolated54/0notproof).
  ControllerSHA/apply/reversecheck0thenACTUALfull_cards54/0. Items25/2runtimeRED
  Bark20vs40/Bloodunhandled; sameitemowner active realnativecontext/healthbody/
  queuedHeal/rootpacket. AscsingletestexecuteswrongA0Louseoverridemapexpectation,
  sameenemyowner fixesoracleFIRSTthenobserveA1rejectionbeforeprofilebody. Sampled
  overridecontractunchanged, no fakecanonicaloverride. Cardidle,item/enemyactive;
  no currentworkspacegreen or familyacceptance,343/0prebatch only.
  Ascension4file+health5file exactjoins nowAPPLIED/reversecheck0. Controllerwrapper
  repeatedsame2itemcompileerrors across4filters (zero testsnotproof); sameowner
  fixedtype/borrow thenwrongBarkHP60>max48fixture tovalidLouse15/15+Block30:
  Fire40=>HP5/noCurl. Freshcontrollerenemy30/0 and finalworkspace367/0+diffcheck
  afterALLreturns(cards54/enemies30/items34), actualCultistJawA0-20/liveA20 and
  Bark/Blood/Fruit/nativequeuedHeal. Otherasc/consumerprovenance/healmodifiersstill
  unfinished, nofamilyacceptance. Same3owners active Watchercommons/Bloomhealing/
  Act2ThreeCultists+Act3hardJawformations per mechanics-next-after-asc-health.md;
  367/0predatesnewedits, notfuture/currentconcurrentgreen.
  Encounterauthorreturnedproposalonly/newapprovalgate; controllerexplicitly
  approvedit asmatchingexistinguserspec/plan/specificbrief, sameowneractualwork
  resumed,no perencounterdesigncycle. Watcher/Bloomauthorsreturnedtests-only
  blockedmissingBloomID; exact2ID/catalogjoinnowappliedSHA/apply/reverse0.
  Actualwatcher0/5anditems34/3runtimeREDs missinghandlers/healsuppression; same
  originalownersactivebody+finitehealpolicy/shareddelegationpatch. All3active,
  367/0prebatch/no currentworkspacegreen or familyacceptance.
  Watcherbodyreturned afterrealRED, authorfocused7/0/no sharedpatch. Bloomowned
  finitehealpolicy/bodyreturned, exact1Healrootpatch nowSHA/apply/reversechecked
  andintegrated; legacyunchanged/FruitqueuedHealpreserved. Singleitemrunstops9
  activeenemyOWNJawWormvariantmigrationerrors/0tests; noforeignedits/bugverdict
  onin-flightcandidate. Card/itemidle/released, awaitactualenemyreturn/sharedjoin
  thenjointtests, no polling/buildstorm/newidlework.367/0prebatch/no acceptance.
  Enemy returned actual ThreeCultists/JawHorde and core317/0, but applied its own
  four-file shared request contrary to controller-only ownership. Bounded author
  provenance correction confirms commands/paths; incorrect controller attribution
  preserved as corrected history. Own .9 claim normally released; all3 idle.
  Controller inspected current bodies/queue/tests, reversecheck0; no revert or
  second apply. Fresh workspace381/0+diffcheck includes cards61/enemies34/items37,
  real Watcher/Bloom/Acts2-3 A0-20 behavior, not family/full-run acceptance. Missing
  multiAct preimplementation runtimeRED remains explicit. Allscope unchanged;
  future shared requests author-unapplied/controller-only, no downstream release.
  User explicitly requested resume after unnecessary controller scheduling pause.
  .7/.8/.9 conditionally reopened/allocated to SAME actors and original sessions
  actually dispatched: Defect five common attack follow-ups, MagicFlower/
  ToyOrnithopter native lifecycle, GremlinNob A0-20/Enrage. See
  mechanics-next-after-watcher-bloom-multiact.md for exact scope/ownership/facts.
  Own registry-family ID files are assigned; no ID-onlywait, no sharedroot apply
  by authors, no new per-content approvalgate. Nob thresholds3/8/18 use bounded
  public revision56364 reference-informed only.381/0 predates new concurrent work,
  no currentgreen/familyacceptance/downstreamrelease. Controller continues after
  concrete handoffs rather than turning verified releases into indefinite pauses.
  ItemFlower/Toybody returned; controllercheckedSHA/currentapply then exact1file
  Healroot joined/reverse0. Toy2runtimeREDs historical but wrongoracles:20+10+5
  required35not41; two combat55 expectations exceedfixturemax50. Sameitemowner
  activeOWNTESTONLY arithmetic/headroomfix (combatHP20/max50 ->Toy/Flower35 vs
  baseline30), no product/capnerf. FlowerREDunexecuted dueactiveDefectmigration;
  no repeatedknownbuild. Card/enemyactive;381/0prior/no currentgreen/acceptance.
  Itemtestcorrectionsreturned/controllerreadconfirmed: outside35, combatToy/
  FlowerBloodHP20max50->35; Bark49/Fruit48max55unchanged. No production/capchanges
  or repeatedbuild; itemidle/released awaiting actualcard/enemyjoins thenjoint
  verification. Card/enemystillactive;381/0prioronly, notcurrentgreen/acceptance.
  Defectexact4filefollowupjoinAPPLIED afterALL4SHA/applycheck/reverse0 preserving
  Flowerroot. Actualcontrollercards67/0/items42/0nowexecute. StaticBarrageStart
  queuesNextremaining0foroneorb/Nextalwayshits; existingtestonly2/3orbs. SAME
  cardowneractivebounded0/1regressionFIRSTthenminimalfix/exactstrictrootrequest
  ifneeded, no newcards. EnemyNobactive,itemidle; awaitreturns/jointworkspace,
  381/0prioronly/no familyacceptance. Futurelive-roombridgecallerremainsrequired.
  Nob4filerootnowjoined/currentapply/reverse0 preservingDefect/Flower; actual
  enemy36/1 waswrongHPoracleignoredDefendBlock10, originalownerOWNTESTONLYraw14/
  Block10/HPloss4/Vuln2->37/0. Barrageoneorbextra-hitruntimeREDfixedownedStart;
  separatezeroNextRED strict1..9rootnowjoinedSHA/apply/reverse0. ALLreturns then
  freshworkspace396/0+diffcheck(cards68/enemies37/items42). SAME3originalsessions
  actuallyresumednextper mechanics-next-after-defect-flower-nob.md: SilentPoison/
  Catalyst+singleArtifactcheckpoint(cardowneronly), sixfinitecombatpotions and
  BlueSlaver/TwoFungiA0-20+SporeCloud. Newascpublic61769/57207reference-informed;
  sharedrootscontrolleronly.396/0predatesnewedits/notconcurrentgreen oracceptance.
  BlueSlaver/TwoFungi author returnedtests-only: singlefocusstopsE0004activeitem
  potionmatchmissing6arms, zeroenemytests/no runtimeRED/body/rootpacket. Ownclaim
  releasedidle, card/itemactive. Awaitactualitemreturn/joins thenONEnamedsetup
  regressionforRegistryRED beforeoriginalenemybody; noforeignfix/buildstorm/
  unchangedredispatch.396/0prioronly/no acceptance ordownstreamrelease.
  User notedBarrage>3slots. Controllerread: countoccupied supportsruntimecapacity
  <=10/remaining1..9, buttests0-3only; 5/10occupied+sparse10casesaddedtocardbrief,
  carrytooriginalcardhandoff/no paralleltesteditor. Itemsockettransporterrorleft
  ownclaimin_progress; SAMEsessionactuallyresumedrecovery/preservepartialwork.
  Actual6potionarmsnowpresent, oldenemyE0004notcurrentproof. Cardactive/item
  recoveredactive/enemyidleawaitactualhandoff;396/0prioronly/no newgreen.
  Recovereditemownerreturned6actualpotionbodies/registry/tests afterreal2handlerRED;
  payloadprepPASS butconsumerfixturefixes/Swiftcaprecycleunverified. Latestfocus
  stops pendingcardApplyPoison BEFOREtests, no newitemroot/Artifactduplicate.
  Item/enemyidle/released, ONLYcardPoison/Artifactactive. Awaitactualcardpacket,
  thenitems+ONEnamedenemysetupRegistryRED andsameenemybody; carryuserBarrage>3
  regressionrequirementatcardhandoff.396/0prioronly/no newgreen/acceptance.
  Shared roots controller only. No family acceptance;276/0 predates these edits,
  not current concurrent green. Prior
  counts are snapshot evidence only, not family acceptance.
  Shared roots remain controller-integrated requests, not overlapping ownership.
  189/0 is the accepted pre-wave baseline, not current in-progress verification.
  No blocked downstream component is an author assignment.
  The first author returned NEEDS_CONTEXT: the initial task incorrectly required a
  preapproved complete full-game manifest before engine work. Task 1 is a tested
  registry foundation with exact four-character starts and honest incomplete-pool
  errors; card/item/enemy/event owners extend their own registry families alongside
  real mechanics. Full normally reachable scope is unchanged at final acceptance.
  Do not create a manifest/source-census gate or sample a reduced partial pool.
  Authors release their own completed/blocked active claims normally for review;
  controllers must not force-close or impersonate an author. `.5` was closed by
  its actual allocated author after controller acceptance.
  TIME-001 is resolved: assume Portal's 800-second condition
  always satisfied, keeping other eligibility and normal random selection; no
  clock, frame or wait subsystem. The user corrected the unsupported
  Reaper lost-heal inference; the global frame/clock proposal is withdrawn, not a
  requirement. Do not resume a source/timer audit or implement that historical
  proposal. Preserve earned Reaper healing when combat ends.
  Design approval alone permits planning; the later plan/start approval above
  authorizes scoped implementation after workspace setup, not installs or commits.
- Write one concise full-run design, not a specification/reviewer cycle per card,
  field or fixture. All-four-character A0–20 full runs through the Heart and
  external ML are fixed boundaries. Document content reachability without hidden
  cuts; implementation interfaces and acceptance cases remain design decisions.
- Verify production behavior with existing Rust/Python tests, small deterministic
  fixtures, seeded replay and focused regressions. Every verification activity must
  name the product behavior or concrete defect it protects. Counts/hashes alone
  are not mechanics proof, and shortcuts/no-op content are not a valid PoC.
- Do not build Java parsers, symbolic executors, semantic-law DSLs, general
  validators, receipt pipelines or broad-copy/mutation farms to verify this work.
  Reuse qualified facts; inspect a missing mechanic only when a concrete task needs
  it. New verification tooling/dependencies or a wider audit need an explicit
  scope decision tied to a demonstrated product blocker.
- Normal TDD iterations are expected. Repeated review-level failures of the same
  verification approach require a bounded root-cause decision, not endless checker
  patches. Use component-boundary reviews and one integrated review; no review
  swarm per assertion. Keep unsupported behavior/fidelity limitations explicit.

## Preservation and verification

- Historical specs, private sources, failures, checkpoints, bytecode, worktrees
  and unfamiliar/uncommitted files are not disposable. Inspect before overwriting;
  no broad deletion, reset, checkout/revert, worktree removal or blind merge.
- Original game source/binaries/assets and private verification artifacts stay
  outside the public repo. Never copy historical private material wholesale.
- No commits, pushes, publication, dependency installs/removals, service/security
  changes or original game execution without separate explicit authority.
- Keep source-grounded versus original-runtime-validated claims distinct. Use
  focused tests and disclose unsupported mechanics; deferred proof is not proof.
  Do not build a general verification platform instead of the requested gym path.
- For environment usage/tests, see `docs/combat-environments.md` and
  `docs/development.md`. Existing test counts are historical, not current reruns.
  CUDA/device smoke checks are not a prerequisite for Rust/gym correctness.
