use super::*;
use crate::combat::{Actor, Combat, CombatAction, CombatCardData, CombatCardRef, CombatEntry,
    DamagePayload, EffectKind, EnemyHandle, FullCombatSetup, FullEnemySetup, FullIntentId,
    Hook, LiveCard, PlayerState, Source, TargetId};
use crate::combat::rng::{Domain, StreamId, StreamState};
use crate::content::{CardId, EnemyId};

fn full_entry(character: crate::content::Character, opening_draw: usize, count: u64) -> CombatEntry {
    let suffix = format!("starter-{character:?}");
    let shuffle_id = StreamId::new(Domain::RunCombatShuffle, vec![suffix.clone()]).unwrap();
    let ai_id = StreamId::new(Domain::RunCombatEnemyAi, vec![suffix.clone(), "enemy-1".into()]).unwrap();
    let card_stream_id = StreamId::new(Domain::RunCombatCardRandom, vec![suffix]).unwrap();
    let setup = FullCombatSetup {
        enemy_damage_bases: Default::default(),
        player: PlayerState { hp: 40, max_hp: 50, block: 0, energy: 3,
            max_energy: 3, draw_per_turn: 5, powers: vec![] },
        player_powers: vec![], character,
        shuffle_rng: StreamState::new(shuffle_id, [21; 32], 0).unwrap(),
        card_rng: StreamState::new(card_stream_id, [23; 32], 0).unwrap(),
        enemies: vec![FullEnemySetup { initial_memory:None, instance_id: EnemyHandle::parse("enemy-1").unwrap(), ascension: 0, profile: crate::combat::enemies::EnemyProfile::Standard,
            enemy_id: EnemyId::Cultist, hp: 30, max_hp: 48, block: 0,
            intent: FullIntentId::Incantation, powers: vec![],
            ai_rng: StreamState::new(ai_id, [22; 32], 0).unwrap() }],
    };
    let cards = (1..=count).map(|id| LiveCard { misc:0, instance: crate::combat::CardInstanceId(id),
        id: CardId::StrikeBlue, origin: CombatCardRef::Permanent(crate::combat::CardInstanceId(id)),
        upgrades: 0, local: CombatCardData::default(),copy_family:id,copy_provenance:crate::combat::card_state::CopyProvenance::Original }).collect();
    CombatEntry { setup, opening_draw, cards }
}

fn enter_with_items(entry: CombatEntry, items: CombatItemContext) -> Combat {
    let mut owner = Some(items);
    let combat = Combat::enter_with_item_context(entry, &mut owner).unwrap();
    assert!(owner.is_none(), "successful entry moves inventory into Combat");
    combat
}

fn combat_potion_facts<'a>(live_targets: &'a [EnemyHandle]) -> NativePotionUseContext<'a> {
    NativePotionUseContext { phase: PotionRoomPhase::Combat, turn_ended: false,
        we_meet_again: false, hp: 30, max_hp: 50, live_targets }
}

#[test]
fn ring_of_the_snake_draws_two_after_opening_hand_once_only_when_owned() {
    let entry = full_entry(crate::content::Character::Silent, 5, 10);
    let baseline = Combat::enter(full_entry(crate::content::Character::Silent, 7, 10)).unwrap();
    let expected_order = baseline.public_full().unwrap().hand.into_iter()
        .map(|card| card.instance).collect::<Vec<_>>();

    let mut combat = enter_with_items(entry.clone(),
        CombatItemContext::new(vec![RelicId::RingOfTheSnake], vec![]));
    let view = combat.public_full().unwrap();
    assert_eq!(view.hand.len(), 7);
    assert_eq!(view.hand.iter().map(|card| card.instance).collect::<Vec<_>>(),
        expected_order, "starter relic's extra draw follows the ordinary opening hand order");
    assert_eq!(combat.item_context().unwrap().relics(), &[RelicId::RingOfTheSnake]);

    let absent = Combat::enter(entry).unwrap();
    assert_eq!(absent.public_full().unwrap().hand.len(), 5, "other starts do not get Ring of the Snake");

    combat.enqueue_native_effects(vec![crate::combat::Effect { source: Source::None,
        target: Some(Actor::Enemy(EnemyHandle::parse("enemy-1").unwrap())),
        kind: EffectKind::Damage(DamagePayload::Fixed(30)) }], None).unwrap();
    let exit = combat.exit().unwrap();
    assert_eq!(exit.item_context.as_ref().unwrap().relics(), &[RelicId::RingOfTheSnake],
        "the one active inventory owner returns through successful combat Exit");
}

#[test]
fn cracked_core_channels_a_real_lightning_orb_before_ready_and_passive_uses_it() {
    let entry = full_entry(crate::content::Character::Defect, 1, 10);
    let combat = enter_with_items(entry.clone(),
        CombatItemContext::new(vec![RelicId::CrackedCore], vec![]));
    assert_eq!(combat.public_full().unwrap().character.orbs.iter().filter(|orb| orb.is_some()).count(), 1,
        "Cracked Core channels one actual entry orb before the ready observation");
    let mut combat = combat;
    combat.step_full(&CombatAction::EndTurn).unwrap();
    assert_eq!(combat.public_full().unwrap().enemies[0].hp, 27,
        "the entry orb performs its real 3-damage passive at the subsequent end turn");

    let absent = Combat::enter(entry).unwrap();
    assert!(absent.public_full().unwrap().character.orbs.iter().all(Option::is_none),
        "non-Defect ownership must not invent Cracked Core's starting orb");
}

#[test]
fn pure_water_generates_a_unique_temporary_miracle_that_plays_and_exhausts() {
    let entry = full_entry(crate::content::Character::Watcher, 5, 10);
    let mut combat = enter_with_items(entry,
        CombatItemContext::new(vec![RelicId::PureWater], vec![]));
    let generated = combat.public_full().unwrap().hand.into_iter().find(|card|
        card.id == CardId::Miracle).expect("Pure Water creates its actual Miracle in the opening hand");
    assert_eq!(generated.upgrades, 0);
    assert_eq!(generated.origin, CombatCardRef::CombatLocal(generated.instance.0));
    assert!(!(1..=10).contains(&generated.instance.0),
        "generated instance must not alias a permanent deck instance");

    combat.step_full(&CombatAction::PlayCard { instance: generated.instance.0, target: None }).unwrap();
    let ready = combat.public_full().unwrap();
    assert_eq!(ready.player.energy, 4, "base Miracle grants one energy without payment");
    assert!(ready.exhaust.iter().any(|card| card.instance == generated.instance
        && card.id == CardId::Miracle), "played Miracle exhausts its generated instance");
    assert!(!ready.hand.iter().any(|card| card.instance == generated.instance));
    assert_eq!(combat.item_context().unwrap().relics(), &[RelicId::PureWater]);

    let full_hand = enter_with_items(
        full_entry(crate::content::Character::Watcher, 10, 10),
        CombatItemContext::new(vec![RelicId::PureWater], vec![]));
    let full = full_hand.public_full().unwrap();
    assert_eq!(full.hand.len(), 10);
    assert_eq!(full.discard.iter().filter(|card| card.id == CardId::Miracle).count(), 1,
        "the generated temporary card respects hand capacity via discard overflow");
}

#[test]
fn starter_relic_entry_effects_are_not_replayed_at_ready_or_on_duplicate_start_hook() {
    let entry = full_entry(crate::content::Character::Silent, 5, 10);
    let mut combat = enter_with_items(entry,
        CombatItemContext::new(vec![RelicId::RingOfTheSnake], vec![]));
    let before = combat.snapshot();
    assert_eq!(combat.enqueue_hook(crate::combat::HookEvent { hook: Hook::CombatStarted,
        source: Source::None, target: None }), Err(RunError::InvalidAction));
    assert_eq!(crate::combat::snapshot::compare(&before, &combat.snapshot()), None,
        "a ready-boundary duplicate start hook cannot replay startup draws");
    assert_eq!(combat.public_full().unwrap().hand.len(), 7);
}

#[test]
fn starter_entry_native_fault_retains_the_moved_item_context_and_first_fault() {
    let mut entry = full_entry(crate::content::Character::Silent, 5, 10);
    entry.cards[0].id = CardId::Unknown("unsupported-starter-fault-fixture");
    let mut combat = enter_with_items(entry,
        CombatItemContext::new(vec![RelicId::RingOfTheSnake], vec![]));
    assert_eq!(combat.item_context().unwrap().relics(), &[RelicId::RingOfTheSnake]);
    let fault = combat.step_full(&CombatAction::PlayCard { instance: 1, target: None }).unwrap_err();
    assert_eq!(combat.item_context().unwrap().relics(), &[RelicId::RingOfTheSnake],
        "a native card fault does not drop the active starter inventory");
    let before = combat.snapshot();
    assert_eq!(combat.install_item_context(CombatItemContext::default()), Err(fault.clone()));
    assert_eq!(crate::combat::snapshot::compare(&before, &combat.snapshot()), None);
    assert_eq!(combat.exit(), Err(fault));
}

#[test]
fn item_context_transfers_run_inventory_out_and_back_without_a_second_owner() {
    use crate::run::{CreationProfile, Run, RunConfig, RunRng};
    let mut run = Run::new(RunConfig { character: crate::content::Character::Silent, ascension: 0 },
        CreationProfile::experienced(), RunRng::new([0; 32], crate::combat::rng::Partition::Test, 0)).unwrap();
    let state = run.state_mut_for_test();
    state.relics = vec![RelicId::RingOfTheSnake];
    state.potions = vec![None, Some(PotionId::FirePotion)];
    let context = CombatItemContext::take_from_run(run.state_mut_for_test());
    assert!(run.state().relics.is_empty());
    assert!(run.state().potions.is_empty());
    run.state_mut_for_test().relics.push(RelicId::BurningBlood);
    assert_eq!(context.clone().return_to_run(run.state_mut_for_test()), Err(RunError::InvalidAction),
        "return must not silently replace inventory acquired by a competing owner");
    assert_eq!(run.state().relics, vec![RelicId::BurningBlood]);
    run.state_mut_for_test().relics.clear();
    context.return_to_run(run.state_mut_for_test()).unwrap();
    assert_eq!(run.state().relics, vec![RelicId::RingOfTheSnake]);
    assert_eq!(run.state().potions, vec![None, Some(PotionId::FirePotion)]);
}

#[test]
fn rejected_combat_entry_leaves_inventory_with_the_run_owner() {
    let mut entry = full_entry(crate::content::Character::Silent, 5, 10);
    entry.setup.player.hp = 0;
    let mut owner = Some(CombatItemContext::new(vec![RelicId::RingOfTheSnake], vec![]));
    assert!(matches!(Combat::enter_with_item_context(entry, &mut owner),
        Err(crate::combat::CombatEntryFailure::Rejected(RunError::InvalidAction))));
    assert_eq!(owner.as_ref().unwrap().relics(), &[RelicId::RingOfTheSnake],
        "pre-entry rejection must not consume the RunState's only inventory owner");
}

#[test]
fn startup_fault_returns_the_mutated_combat_and_never_restores_or_replays_inventory() {
    // With 999 issued identities, hook dispatch rejects Pure Water's generated
    // card while constructing the startup queue, before Ring's Draw dispatch.
    // This separately covers retention of a transferred context on a pre-draw
    // startup fault; the native combat fixture exercises a later insertion fault.
    let entry = full_entry(crate::content::Character::Silent, 5, 999);
    let mut owner = Some(CombatItemContext::new(
        vec![RelicId::RingOfTheSnake, RelicId::PureWater], vec![]));
    let failure = Combat::enter_with_item_context(entry, &mut owner).unwrap_err();
    assert!(owner.is_none(), "transferred inventory must not be copied back on a post-mutation fault");
    let crate::combat::CombatEntryFailure::Faulted { combat, fault } = failure else {
        panic!("expected retained post-startup native fault")
    };
    let mut combat = *combat;
    let before = combat.snapshot();
    let record = combat.fault_record().expect("faulted entry retains native diagnostic state");
    assert_eq!(combat.item_context().unwrap().relics(),
        &[RelicId::RingOfTheSnake, RelicId::PureWater]);
    let crate::run::RunFaultKind::Combat(native) = &fault.kind else { panic!("combat fault expected") };
    assert_eq!(record, native.record, "entry returns the same first fault as the combat state");
    assert_eq!(combat.step_full(&CombatAction::EndTurn), Err(RunError::Faulted(fault.clone())));
    assert_eq!(crate::combat::snapshot::compare(&before, &combat.snapshot()), None,
        "returned faulted combat cannot retry entry/startup or replay effects");
}

#[test]
fn blood_vial_heals_two_at_combat_start_once_and_caps_at_max_hp() {
    let mut entry = full_entry(crate::content::Character::Ironclad, 5, 10);
    entry.setup.player.hp = 40;
    let combat = enter_with_items(entry.clone(),
        CombatItemContext::new(vec![RelicId::BloodVial], vec![]));
    assert_eq!(combat.public_full().unwrap().player.hp, 42);

    entry.setup.player.hp = 50;
    let full = enter_with_items(entry,
        CombatItemContext::new(vec![RelicId::BloodVial], vec![]));
    assert_eq!(full.public_full().unwrap().player.hp, 50);

    let absent = Combat::enter(full_entry(crate::content::Character::Ironclad, 5, 10)).unwrap();
    assert_eq!(absent.public_full().unwrap().player.hp, 40,
        "combat startup does not synthesize Blood Vial ownership");
}

#[test]
fn anchor_adds_ten_block_and_lantern_adds_one_energy_only_for_first_turn() {
    let entry = full_entry(crate::content::Character::Ironclad, 5, 10);
    let anchor = enter_with_items(entry.clone(),
        CombatItemContext::new(vec![RelicId::Anchor], vec![]));
    let view = anchor.public_full().unwrap();
    assert_eq!(view.player.block, 10);
    assert_eq!(view.player.energy, 3);
    let absent = Combat::enter(full_entry(crate::content::Character::Ironclad, 5, 10)).unwrap();
    assert_eq!(absent.public_full().unwrap().player.block, 0,
        "combat startup does not synthesize Anchor ownership");

    let mut lantern = enter_with_items(entry,
        CombatItemContext::new(vec![RelicId::Lantern], vec![]));
    assert_eq!(lantern.public_full().unwrap().player.energy, 4,
        "Lantern's first-turn energy is resolved before the opening decision");
    lantern.step_full(&CombatAction::EndTurn).unwrap();
    assert_eq!(lantern.public_full().unwrap().player.energy, 3,
        "Lantern does not replay on later turn starts");
    let absent = Combat::enter(full_entry(crate::content::Character::Ironclad, 5, 10)).unwrap();
    assert_eq!(absent.public_full().unwrap().player.energy, 3,
        "combat startup does not synthesize Lantern ownership");
}

#[test]
fn blood_vial_and_lantern_startup_work_precedes_anchor_bottom_queued_block() {
    let mut context = CombatItemContext::new(
        vec![RelicId::Anchor, RelicId::Lantern, RelicId::BloodVial], vec![]);
    let startup = relic_hook_effects(&mut context, Hook::CombatStarted, 40, 80,0);
    assert_eq!(startup.top.iter().map(|effect| effect.kind.clone()).collect::<Vec<_>>(),
        vec![EffectKind::Heal(2)]);
    assert_eq!(startup.bottom.iter().map(|effect| effect.kind.clone()).collect::<Vec<_>>(),
        vec![EffectKind::GainBlock(10)]);

    let first_turn = relic_hook_effects(&mut context, Hook::TurnStarted, 40, 80,0);
    assert_eq!(first_turn.top.iter().map(|effect| effect.kind.clone()).collect::<Vec<_>>(),
        vec![EffectKind::GainEnergy(1)]);
    assert!(first_turn.bottom.is_empty());
    assert!(relic_hook_effects(&mut context, Hook::TurnStarted, 40, 80,0).top.is_empty());
}

fn run_with_vitals(hp: u32, max_hp: u32) -> crate::run::Run {
    use crate::run::{CreationProfile, Run, RunConfig, RunRng};
    let mut run = Run::new(RunConfig { character: crate::content::Character::Ironclad, ascension: 0 },
        CreationProfile::experienced(), RunRng::new([7; 32], crate::combat::rng::Partition::Test, 0)).unwrap();
    run.state_mut_for_test().hp = hp;
    run.state_mut_for_test().max_hp = max_hp;
    run
}

#[test]
fn native_outside_blood_use_heals_by_truncated_percent_and_consumes_once() {
    let mut run = run_with_vitals(30, 81);
    run.state_mut_for_test().potions[0] = Some(PotionId::BloodPotion);
    use_run_potion(run.state_mut_for_test(), 0, false, false).unwrap();
    assert_eq!((run.state().hp, run.state().max_hp), (46, 81));
    assert_eq!(run.state().potions[0], None);

    let mut bark = run_with_vitals(30, 81);
    bark.state_mut_for_test().relics.push(RelicId::SacredBark);
    bark.state_mut_for_test().potions[0] = Some(PotionId::BloodPotion);
    use_run_potion(bark.state_mut_for_test(), 0, true, false).unwrap();
    assert_eq!(bark.state().hp, 62, "outside-combat turn-ended does not reject this special potion");

    let mut capped = run_with_vitals(80, 81);
    capped.state_mut_for_test().potions[0] = Some(PotionId::BloodPotion);
    use_run_potion(capped.state_mut_for_test(), 0, false, false).unwrap();
    assert_eq!(capped.state().hp, 81);

    let mut dead = run_with_vitals(0, 1);
    dead.state_mut_for_test().potions[0] = Some(PotionId::BloodPotion);
    use_run_potion(dead.state_mut_for_test(), 0, false, false).unwrap();
    assert_eq!(dead.state().hp, 0, "ordinary healing never revives zero HP");
    assert_eq!(dead.state().potions[0], None);
}

#[test]
fn native_outside_fruit_juice_increases_max_hp_before_heal_and_handles_cap_or_death() {
    let mut base = run_with_vitals(48, 50);
    base.state_mut_for_test().potions[0] = Some(PotionId::FruitJuice);
    use_run_potion(base.state_mut_for_test(), 0, false, false).unwrap();
    assert_eq!((base.state().hp, base.state().max_hp), (53, 55));
    assert_eq!(base.state().potions[0], None);

    let mut bark = run_with_vitals(58, 60);
    bark.state_mut_for_test().relics.push(RelicId::SacredBark);
    bark.state_mut_for_test().potions[0] = Some(PotionId::FruitJuice);
    use_run_potion(bark.state_mut_for_test(), 0, false, false).unwrap();
    assert_eq!((bark.state().hp, bark.state().max_hp), (68, 70));

    let mut dead = run_with_vitals(0, 50);
    dead.state_mut_for_test().potions[0] = Some(PotionId::FruitJuice);
    use_run_potion(dead.state_mut_for_test(), 0, false, false).unwrap();
    assert_eq!((dead.state().hp, dead.state().max_hp), (0, 55),
        "increaseMaxHp advances before ordinary heal's dying guard");
}

#[test]
fn mark_of_bloom_suppresses_run_blood_and_fruit_heals_without_reversing_max_hp_or_slot_commit() {
    let mut blood = run_with_vitals(30, 81);
    blood.state_mut_for_test().relics.push(RelicId::MarkOfBloom);
    blood.state_mut_for_test().potions[0] = Some(PotionId::BloodPotion);
    use_run_potion(blood.state_mut_for_test(), 0, false, false).unwrap();
    assert_eq!((blood.state().hp, blood.state().max_hp), (30, 81));
    assert_eq!(blood.state().potions[0], None);

    let mut fruit = run_with_vitals(30, 50);
    fruit.state_mut_for_test().relics.push(RelicId::MarkOfBloom);
    fruit.state_mut_for_test().potions[0] = Some(PotionId::FruitJuice);
    use_run_potion(fruit.state_mut_for_test(), 0, false, false).unwrap();
    assert_eq!((fruit.state().hp, fruit.state().max_hp), (30, 55),
        "Bloom suppresses the delegated heal but cannot undo the earlier maxHP increase");
    assert_eq!(fruit.state().potions[0], None, "a valid accepted potion remains consumed");
}

#[test]
fn mark_of_bloom_suppresses_burning_blood_and_max_hp_acquisition_healing() {
    let mut run = run_with_vitals(30, 50);
    run.state_mut_for_test().relics.push(RelicId::MarkOfBloom);
    on_hook(run.state_mut_for_test(), Hook::CombatSettled).unwrap();
    assert_eq!(run.state().hp, 30, "Burning Blood delegates to the same ordinary heal policy");

    let mut obtain = run_with_vitals(20, 50);
    obtain.state_mut_for_test().relics.push(RelicId::MarkOfBloom);
    crate::items::acquire_relic(obtain.state_mut_for_test(), RelicId::Strawberry).unwrap();
    assert_eq!((obtain.state().hp, obtain.state().max_hp), (20, 57));
}

#[test]
fn mark_of_bloom_suppresses_native_combat_heal_and_blood_vial_but_preserves_owned_context() {
    let mut combat = entered_combat(30, 30);
    combat.install_item_context(CombatItemContext::new(vec![RelicId::MarkOfBloom], vec![])).unwrap();
    combat.enqueue_native_effects(vec![crate::combat::Effect { source: Source::Actor(Actor::Player),
        target: Some(Actor::Player), kind: EffectKind::Heal(10) }], None).unwrap();
    assert_eq!(combat.public_full().unwrap().player.hp, 30);

    let entry = full_entry(crate::content::Character::Ironclad, 5, 10);
    let combat = enter_with_items(entry,
        CombatItemContext::new(vec![RelicId::MarkOfBloom, RelicId::BloodVial], vec![]));
    assert_eq!(combat.public_full().unwrap().player.hp, 40,
        "the startup Blood Vial Heal is suppressed by the owned Bloom callback");
    assert!(combat.item_context().unwrap().owns_relic(RelicId::MarkOfBloom));
}

#[test]
fn toy_ornithopter_heals_five_once_only_after_successful_manual_potion_use() {
    let mut run = run_with_vitals(20, 50);
    run.state_mut_for_test().relics.push(RelicId::ToyOrnithopter);
    run.state_mut_for_test().potions[0] = Some(PotionId::BloodPotion);
    use_run_potion(run.state_mut_for_test(), 0, false, false).unwrap();
    assert_eq!(run.state().hp, 35, "Blood's 20% (10) precedes Ornithopter's fixed 5");
    assert_eq!(run.state().potions[0], None);

    let mut bark = run_with_vitals(20, 60);
    bark.state_mut_for_test().relics.extend([RelicId::SacredBark, RelicId::ToyOrnithopter]);
    bark.state_mut_for_test().potions[0] = Some(PotionId::BloodPotion);
    use_run_potion(bark.state_mut_for_test(), 0, false, false).unwrap();
    assert_eq!(bark.state().hp, 49, "Bark doubles Blood only, not Ornithopter's 5");

    let mut invalid = run_with_vitals(20, 50);
    invalid.state_mut_for_test().relics.push(RelicId::ToyOrnithopter);
    let before = invalid.state().clone();
    assert_eq!(use_run_potion(invalid.state_mut_for_test(), 0, false, false), Err(RunError::InvalidAction));
    assert_eq!(*invalid.state(), before, "failed use does not invoke onUsePotion");
}

#[test]
fn toy_ornithopter_manual_combat_potion_heal_uses_same_queue_and_consumes_once() {
    let mut entry = full_entry(crate::content::Character::Ironclad, 5, 10);
    entry.setup.player.hp = 20;
    let mut combat = enter_with_items(entry, CombatItemContext::new(
        vec![RelicId::ToyOrnithopter], vec![Some(PotionId::BloodPotion)]));
    use_combat_potion(&mut combat, 0, None, false).unwrap();
    assert_eq!(combat.public_full().unwrap().player.hp, 35,
        "combat Blood heals 20%, then the manual-use relic queues exactly five");
    assert_eq!(combat.item_context().unwrap().potions(), &[None]);
}

#[test]
fn magic_flower_rounds_binary32_for_live_combat_only_including_small_odd_amounts() {
    let context = CombatItemContext::new(vec![RelicId::MagicFlower], vec![])
        .with_native_room(PotionRoomPhase::Combat, None);
    let mut no_node = enter_with_items(full_entry(crate::content::Character::Ironclad, 5, 10), context);
    no_node.enqueue_native_effects(vec![crate::combat::Effect { source: Source::None,
        target: Some(Actor::Player), kind: EffectKind::Heal(1) }], None).unwrap();
    assert_eq!(no_node.public_full().unwrap().player.hp, 41,
        "COMBAT without a current map node leaves the requested heal unchanged");

    let room = test_map_with_current_node();
    let context = CombatItemContext::new(vec![RelicId::MagicFlower], vec![])
        .with_native_room(PotionRoomPhase::Combat, Some(&room));
    let mut live = enter_with_items(full_entry(crate::content::Character::Ironclad, 5, 10), context);
    live.enqueue_native_effects(vec![crate::combat::Effect { source: Source::None,
        target: Some(Actor::Player), kind: EffectKind::Heal(1) }], None).unwrap();
    assert_eq!(live.public_full().unwrap().player.hp, 42,
        "Java-style round(1×binary32(1.5)) heals two");
}

fn test_map_with_current_node() -> crate::run::ActMap {
    let mut run = run_with_vitals(40, 50);
    let mut map = crate::run::map::generate(1, &mut run.state_mut_for_test().rng,
        crate::run::map::MapEligibility::new(true, false)).unwrap();
    map.current = Some(map.nodes[0].id);
    map
}

#[test]
fn magic_flower_receives_blood_percentage_after_binary32_truncation_and_not_fruit_max_hp() {
    let room = test_map_with_current_node();
    let context = CombatItemContext::new(vec![RelicId::MagicFlower], vec![Some(PotionId::BloodPotion)])
        .with_native_room(PotionRoomPhase::Combat, Some(&room));
    let mut entry = full_entry(crate::content::Character::Ironclad, 5, 10);
    entry.setup.player.hp = 20;
    let mut blood = enter_with_items(entry, context);
    use_combat_potion(&mut blood, 0, None, false).unwrap();
    assert_eq!(blood.public_full().unwrap().player.hp, 35,
        "Blood truncates 20% of 50 to 10, then Flower rounds 15");

    let context = CombatItemContext::new(vec![RelicId::MagicFlower], vec![Some(PotionId::FruitJuice)])
        .with_native_room(PotionRoomPhase::Combat, Some(&room));
    let mut fruit = enter_with_items(full_entry(crate::content::Character::Ironclad, 5, 10), context);
    use_combat_potion(&mut fruit, 0, None, false).unwrap();
    let player = fruit.public_full().unwrap().player;
    assert_eq!((player.hp, player.max_hp), (48, 55),
        "Fruit adds maxHP 5, while its separate heal 5 becomes 8");
}

#[test]
fn magic_flower_is_outside_unchanged_and_bloom_still_suppresses_live_combat_heal() {
    let mut outside = run_with_vitals(20, 50);
    outside.state_mut_for_test().relics.extend([RelicId::MagicFlower]);
    outside.state_mut_for_test().potions[0] = Some(PotionId::BloodPotion);
    use_run_potion(outside.state_mut_for_test(), 0, false, false).unwrap();
    assert_eq!(outside.state().hp, 30, "outside-combat Blood receives no Flower multiplier");

    let room = test_map_with_current_node();
    let context = CombatItemContext::new(
        vec![RelicId::MagicFlower, RelicId::MarkOfBloom], vec![])
        .with_native_room(PotionRoomPhase::Combat, Some(&room));
    let mut combat = enter_with_items(full_entry(crate::content::Character::Ironclad, 5, 10), context);
    combat.enqueue_native_effects(vec![crate::combat::Effect { source: Source::None,
        target: Some(Actor::Player), kind: EffectKind::Heal(10) }], None).unwrap();
    assert_eq!(combat.public_full().unwrap().player.hp, 40,
        "Bloom's zero callback remains effective alongside Flower in live combat");
}

#[test]
fn six_combat_potions_prepare_base_and_bark_payloads_with_typed_targets() {
    let target = EnemyHandle::parse("enemy-1").unwrap();
    let facts = combat_potion_facts(std::slice::from_ref(&target));
    let cases = [
        (PotionId::BlockPotion, EffectKind::GainBlock(12), EffectKind::GainBlock(24), false),
        (PotionId::EnergyPotion, EffectKind::GainEnergy(2), EffectKind::GainEnergy(4), false),
        (PotionId::StrengthPotion, EffectKind::ApplyFullPower { id: crate::combat::FullPowerId("strength"), amount: 2 },
            EffectKind::ApplyFullPower { id: crate::combat::FullPowerId("strength"), amount: 4 }, false),
        (PotionId::SwiftPotion, EffectKind::Draw(3), EffectKind::Draw(6), false),
        (PotionId::WeakPotion, EffectKind::ApplyFullPower { id: crate::combat::FullPowerId("weak"), amount: 3 },
            EffectKind::ApplyFullPower { id: crate::combat::FullPowerId("weak"), amount: 6 }, true),
        (PotionId::FearPotion, EffectKind::ApplyFullPower { id: crate::combat::FullPowerId("vulnerable"), amount: 3 },
            EffectKind::ApplyFullPower { id: crate::combat::FullPowerId("vulnerable"), amount: 6 }, true),
    ];
    for (potion, base, bark, targeted) in cases {
        let requested_target = targeted.then(|| target.clone());
        for (owns_bark, expected) in [(false, base), (true, bark)] {
            let relics = owns_bark.then_some(RelicId::SacredBark).into_iter().collect();
            let context = CombatItemContext::new(relics, vec![Some(potion)]);
            let prepared = potion_use_effects(&context, 0, requested_target.clone(), &facts).unwrap();
            assert_eq!(prepared.effects[0].kind, expected, "{potion:?}, Bark={owns_bark}");
            assert_eq!(prepared.effects[0].target,
                targeted.then(|| Actor::Enemy(target.clone())).or(Some(Actor::Player)));
            assert_eq!(prepared.potions, vec![None]);
        }
    }
}

#[test]
fn six_combat_potions_reject_outside_ended_event_and_invalid_targets_without_slot_mutation() {
    let target = EnemyHandle::parse("enemy-1").unwrap();
    let facts = combat_potion_facts(std::slice::from_ref(&target));
    for potion in [PotionId::BlockPotion, PotionId::EnergyPotion, PotionId::StrengthPotion,
        PotionId::SwiftPotion, PotionId::WeakPotion, PotionId::FearPotion] {
        let context = CombatItemContext::new(vec![], vec![Some(potion)]);
        let outside = NativePotionUseContext { phase: PotionRoomPhase::OutsideCombat, ..facts };
        let ended = NativePotionUseContext { turn_ended: true, ..facts };
        let event = NativePotionUseContext { we_meet_again: true, ..facts };
        assert_eq!(potion_use_effects(&context, 0,
            matches!(potion, PotionId::WeakPotion | PotionId::FearPotion).then(|| target.clone()),
            &outside), Err(RunError::InvalidAction), "{potion:?} outside combat");
        assert_eq!(potion_use_effects(&context, 0,
            matches!(potion, PotionId::WeakPotion | PotionId::FearPotion).then(|| target.clone()),
            &ended), Err(RunError::InvalidAction), "{potion:?} after turn end");
        assert_eq!(potion_use_effects(&context, 0, None, &event), Err(RunError::InvalidAction),
            "{potion:?} in We Meet Again");
        assert_eq!(context.potions(), &[Some(potion)]);
    }

    for potion in [PotionId::WeakPotion, PotionId::FearPotion] {
        let context = CombatItemContext::new(vec![], vec![Some(potion)]);
        assert_eq!(potion_use_effects(&context, 0, None, &facts), Err(RunError::InvalidAction));
        let foreign = EnemyHandle::parse("not-live").unwrap();
        assert_eq!(potion_use_effects(&context, 0, Some(foreign), &facts), Err(RunError::InvalidAction));
        assert_eq!(context.potions(), &[Some(potion)]);
    }
    let context = CombatItemContext::new(vec![], vec![Some(PotionId::BlockPotion)]);
    assert_eq!(potion_use_effects(&context, 0, Some(target.clone()), &facts), Err(RunError::InvalidAction));
}

#[test]
fn six_combat_potions_use_real_block_energy_draw_strength_and_debuff_consumers() {
    let mut entry = full_entry(crate::content::Character::Ironclad, 5, 10);
    entry.setup.player_powers.push(crate::combat::FullPowerEntry {
        id: crate::combat::FullPowerId("dexterity"),
        state: crate::combat::PowerState { amount: 9, skip_first: false, monster_source_grace: false },
    });
    let mut block = enter_with_items(entry, CombatItemContext::new(vec![], vec![Some(PotionId::BlockPotion)]));
    use_combat_potion(&mut block, 0, None, false).unwrap();
    assert_eq!(block.public_full().unwrap().player.block, 12,
        "Block Potion is a direct gain, not card Block scaled by Dexterity");
    assert_eq!(block.item_context().unwrap().potions(), &[None]);

    let mut energy = enter_with_items(full_entry(crate::content::Character::Ironclad, 5, 10),
        CombatItemContext::new(vec![], vec![Some(PotionId::EnergyPotion)]));
    use_combat_potion(&mut energy, 0, None, false).unwrap();
    let view = energy.public_full().unwrap();
    assert_eq!((view.player.energy, view.player.max_energy), (5, 3));
    assert_eq!(energy.item_context().unwrap().potions(), &[None]);

    let mut strength = enter_with_items(full_entry(crate::content::Character::Ironclad, 5, 10),
        CombatItemContext::new(vec![], vec![Some(PotionId::StrengthPotion)]));
    use_combat_potion(&mut strength, 0, None, false).unwrap();
    let view = strength.public_full().unwrap();
    let instance = view.hand[0].instance.0;
    let target = view.targets[0].0;
    strength.step_full(&CombatAction::PlayCard { instance, target: Some(target) }).unwrap();
    assert_eq!(strength.public_full().unwrap().enemies[0].hp, 22,
        "Strength Potion changes the subsequent Strike's real damage");
    assert_eq!(strength.item_context().unwrap().potions(), &[None]);

    let mut swift = enter_with_items(full_entry(crate::content::Character::Ironclad, 5, 10),
        CombatItemContext::new(vec![], vec![Some(PotionId::SwiftPotion)]));
    let before = swift.public_full().unwrap();
    assert_eq!(before.hand.len(), 5);
    use_combat_potion(&mut swift, 0, None, false).unwrap();
    let after = swift.public_full().unwrap();
    assert_eq!(after.hand.len(), 8);
    assert_eq!(after.draw_count, before.draw_count - 3);
    assert_eq!(swift.item_context().unwrap().potions(), &[None]);

    let mut entry = full_entry(crate::content::Character::Ironclad, 5, 10);
    entry.setup.enemies[0].enemy_id = EnemyId::JawWorm;
    entry.setup.enemies[0].hp = 42;
    entry.setup.enemies[0].max_hp = 42;
    entry.setup.enemies[0].intent = FullIntentId::Chomp;
    let target = EnemyHandle::parse("enemy-1").unwrap();
    let mut weak = enter_with_items(entry, CombatItemContext::new(vec![], vec![Some(PotionId::WeakPotion)]));
    use_combat_potion(&mut weak, 0, Some(target), false).unwrap();
    assert_eq!(weak.public_full().unwrap().enemies[0].intent_damage, Some(8),
        "Weak changes the enemy's actual future attack output");
    assert_eq!(weak.item_context().unwrap().potions(), &[None]);

    let target = EnemyHandle::parse("enemy-1").unwrap();
    let mut fear = enter_with_items(full_entry(crate::content::Character::Ironclad, 5, 10),
        CombatItemContext::new(vec![], vec![Some(PotionId::FearPotion)]));
    use_combat_potion(&mut fear, 0, Some(target), false).unwrap();
    let view = fear.public_full().unwrap();
    let instance = view.hand[0].instance.0;
    let target = view.targets[0].0;
    fear.step_full(&CombatAction::PlayCard { instance, target: Some(target) }).unwrap();
    assert_eq!(fear.public_full().unwrap().enemies[0].hp, 21,
        "Fear's Vulnerable changes the subsequent Strike damage");
    assert_eq!(fear.item_context().unwrap().potions(), &[None]);
}

#[test]
fn swift_potion_draw_respects_hand_cap_and_recycles_owned_discard_cards() {
    let mut capped = enter_with_items(full_entry(crate::content::Character::Ironclad, 9, 10),
        CombatItemContext::new(vec![], vec![Some(PotionId::SwiftPotion)]));
    use_combat_potion(&mut capped, 0, None, false).unwrap();
    let capped_view = capped.public_full().unwrap();
    assert_eq!((capped_view.hand.len(), capped_view.draw_count), (10, 0));

    let mut recycled = enter_with_items(full_entry(crate::content::Character::Ironclad, 5, 5),
        CombatItemContext::new(vec![], vec![Some(PotionId::SwiftPotion)]));
    let before = recycled.public_full().unwrap();
    let card = before.hand[0].instance.0;
    let target = before.targets[0].0;
    recycled.step_full(&CombatAction::PlayCard { instance: card, target: Some(target) }).unwrap();
    let before_potion = recycled.public_full().unwrap();
    assert_eq!((before_potion.hand.len(), before_potion.discard.len(), before_potion.draw_count), (4, 1, 0));
    use_combat_potion(&mut recycled, 0, None, false).unwrap();
    let after = recycled.public_full().unwrap();
    assert_eq!((after.hand.len(), after.discard.len(), after.draw_count), (5, 0, 0),
        "Swift uses ordinary draw/recycle and moves an owned discard card into hand");
    assert!(after.hand.iter().any(|owned| owned.instance.0 == card));
}

#[test]
fn native_outside_potion_rejections_preserve_resources_slots_and_retained_fault() {
    let mut run = run_with_vitals(20, 50);
    run.state_mut_for_test().potions[0] = Some(PotionId::BloodPotion);
    let before = run.state().clone();
    assert_eq!(use_run_potion(run.state_mut_for_test(), 0, false, true), Err(RunError::InvalidAction));
    assert_eq!(*run.state(), before);
    assert_eq!(use_run_potion(run.state_mut_for_test(), 8, false, false), Err(RunError::InvalidAction));
    assert_eq!(*run.state(), before);

    let mut overflow = run_with_vitals(900, 999);
    overflow.state_mut_for_test().potions[0] = Some(PotionId::FruitJuice);
    let before_overflow = overflow.state().clone();
    assert_eq!(use_run_potion(overflow.state_mut_for_test(), 0, false, false), Err(RunError::InvalidAction));
    assert_eq!(*overflow.state(), before_overflow, "maxHP bounds reject before healing or slot commit");

    let mut active = run_with_vitals(20, 50);
    active.state_mut_for_test().potions[0] = Some(PotionId::BloodPotion);
    active.state_mut_for_test().combat = Some(entered_combat(3, 3));
    let before_active = active.state().clone();
    assert_eq!(use_run_potion(active.state_mut_for_test(), 0, false, false), Err(RunError::InvalidAction));
    assert_eq!(*active.state(), before_active,
        "outside helper cannot race the active combat's inventory/resource owner");

    let fault = crate::run::RunFault { kind: crate::run::RunFaultKind::UnsupportedBoundary("potion fixture"),
        checkpoint: "run.potion.test", committed_events: vec![] };
    run.state_mut_for_test().fault = Some(fault.clone());
    let before_faulted = run.state().clone();
    assert_eq!(use_run_potion(run.state_mut_for_test(), 0, false, false), Err(RunError::Faulted(fault)));
    assert_eq!(*run.state(), before_faulted);
}

#[test]
fn combat_potion_rejected_by_event_gate_keeps_inventory_and_snapshot_unchanged() {
    let mut combat = entered_combat(3, 3);
    combat.install_item_context(CombatItemContext::new(vec![RelicId::SacredBark],
        vec![Some(PotionId::FirePotion)])).unwrap();
    let before = combat.snapshot();
    assert_eq!(use_combat_potion(&mut combat, 0,
        Some(EnemyHandle::parse("enemy-1").unwrap()), true), Err(RunError::InvalidAction));
    assert_eq!(crate::combat::snapshot::compare(&before, &combat.snapshot()), None);
}

#[test]
fn strawberry_pear_and_mango_acquisition_raise_max_hp_then_heal_by_the_same_amount() {
    let cases = [(RelicId::Strawberry, 7), (RelicId::Pear, 10), (RelicId::Mango, 14)];
    for (id, amount) in cases {
        let mut run = run_with_vitals(20, 50);
        crate::items::acquire_relic(run.state_mut_for_test(), id).unwrap();
        assert_eq!(run.state().max_hp, 50 + amount);
        assert_eq!(run.state().hp, 20 + amount);
        assert_eq!(run.state().relics, vec![RelicId::BurningBlood, id],
            "the acquired relic appends to, rather than replaces, permanent inventory");
    }

    let mut near_cap = run_with_vitals(48, 50);
    crate::items::acquire_relic(near_cap.state_mut_for_test(), RelicId::Strawberry).unwrap();
    assert_eq!((near_cap.state().hp, near_cap.state().max_hp), (55, 57));

    let mut at_cap = run_with_vitals(50, 50);
    crate::items::acquire_relic(at_cap.state_mut_for_test(), RelicId::Mango).unwrap();
    assert_eq!((at_cap.state().hp, at_cap.state().max_hp), (64, 64));

    let mut zero_hp = run_with_vitals(0, 50);
    crate::items::acquire_relic(zero_hp.state_mut_for_test(), RelicId::Pear).unwrap();
    assert_eq!((zero_hp.state().hp, zero_hp.state().max_hp), (0, 60),
        "the increaseMaxHp child cannot use its heal child to revive zero HP");
}

#[test]
fn invalid_or_active_run_relic_acquisition_is_nonmutating() {
    let mut unknown = run_with_vitals(31, 50);
    let before = unknown.state().clone();
    assert_eq!(crate::items::acquire_relic(unknown.state_mut_for_test(),
        RelicId::Unknown("unregistered-acquisition-fixture")), Err(RunError::Registry("unknown relic")));
    assert_eq!(*unknown.state(), before);

    let mut active = run_with_vitals(31, 50);
    active.state_mut_for_test().combat = Some(entered_combat(40, 10));
    let before = active.state().clone();
    assert_eq!(crate::items::acquire_relic(active.state_mut_for_test(), RelicId::Strawberry),
        Err(RunError::InvalidAction));
    assert_eq!(*active.state(), before, "RunState is not a second inventory owner during combat");

    let mut overflow = run_with_vitals(40, u32::MAX);
    let before = overflow.state().clone();
    assert_eq!(crate::items::acquire_relic(overflow.state_mut_for_test(), RelicId::Mango),
        Err(RunError::Unsupported("maximum HP overflow")));
    assert_eq!(*overflow.state(), before, "unrepresentable maxHP increase is transactional");
}

fn entered_combat(player_hp: i32, enemy_hp: i32) -> Combat {
    entered_combat_with_card(player_hp, enemy_hp, CardId::Strike)
}

fn entered_combat_with_card(player_hp: i32, enemy_hp: i32, card_id: CardId) -> Combat {
    let shuffle_id = StreamId::new(Domain::RunCombatShuffle, vec!["item-test".into()]).unwrap();
    let ai_id = StreamId::new(Domain::RunCombatEnemyAi,
        vec!["item-test".into(), "enemy-1".into()]).unwrap();
    let shuffle = StreamState::new(shuffle_id, [1; 32], 0).unwrap();
    let ai = StreamState::new(ai_id.clone(), [2; 32], 0).unwrap();
    let card_stream_id = StreamId::new(Domain::RunCombatCardRandom, vec!["item-test".into()]).unwrap();
    let card_rng = StreamState::new(card_stream_id, [3; 32], 0).unwrap();
    let setup = FullCombatSetup {
        enemy_damage_bases: Default::default(),
        player: PlayerState { hp: player_hp, max_hp: 50, block: 0, energy: 3,
            max_energy: 3, draw_per_turn: 5, powers: vec![] },
        player_powers: vec![], character: crate::content::Character::Ironclad,
        shuffle_rng: shuffle, card_rng,
        enemies: vec![FullEnemySetup { initial_memory:None, instance_id: crate::combat::EnemyHandle::parse("enemy-1").unwrap(), ascension: 0, profile: crate::combat::enemies::EnemyProfile::Standard,
            enemy_id: EnemyId::Cultist, hp: enemy_hp, max_hp: 48, block: 0, intent: FullIntentId::Incantation,
            powers: vec![], ai_rng: ai }],
    };
    let live = LiveCard { misc:0, instance: crate::combat::CardInstanceId(1), id: card_id,
        origin: CombatCardRef::Permanent(crate::combat::CardInstanceId(1)), upgrades: 0,
        local: CombatCardData::default(),copy_family:1,copy_provenance:crate::combat::card_state::CopyProvenance::Original };
    Combat::enter(CombatEntry { setup, opening_draw: 1, cards: vec![live] }).unwrap()
}

#[test]
fn burning_blood_effect_is_owned_survival_gated_and_capped_by_native_heal() {
    let owned = CombatItemContext::new(vec![RelicId::BurningBlood], vec![]);
    let absent = CombatItemContext::default();
    let effects = relic_hook_effects(&mut owned.clone(), Hook::CombatSettled, 30, 80,0);
    assert_eq!(effects.top.len(), 1);
    assert_eq!(effects.top[0].target, Some(Actor::Player));
    assert_eq!(effects.top[0].kind, EffectKind::Heal(6));
    assert_eq!(effects.top[0].source, Source::None);
    assert!(effects.bottom.is_empty());
    assert!(relic_hook_effects(&mut absent.clone(), Hook::CombatSettled, 30, 80,0).is_empty());
    assert!(relic_hook_effects(&mut owned.clone(), Hook::CombatSettled, 0, 80,0).is_empty());
    assert!(relic_hook_effects(&mut owned.clone(), Hook::TurnEnded, 30, 80,0).is_empty());

    let mut combat = entered_combat(48, 1);
    combat.install_item_context(owned).unwrap();
    combat.enqueue_native_effects(vec![crate::combat::Effect { source: Source::None,
        target: Some(Actor::Enemy(crate::combat::EnemyHandle::parse("enemy-1").unwrap())),
        kind: crate::combat::EffectKind::Damage(DamagePayload::Fixed(20)) }], None).unwrap();
    let result = combat.public_full().unwrap();
    assert_eq!(result.player.hp, 50, "six healing is clamped to maximum HP");
    assert_eq!(result.outcome, Some(crate::combat::Outcome::Victory));

    let mut once_only = entered_combat(30, 1);
    once_only.install_item_context(CombatItemContext::new(vec![RelicId::BurningBlood], vec![])).unwrap();
    once_only.enqueue_native_effects(vec![crate::combat::Effect { source: Source::None,
        target: Some(Actor::Enemy(EnemyHandle::parse("enemy-1").unwrap())),
        kind: EffectKind::Damage(DamagePayload::Fixed(20)) }], None).unwrap();
    assert_eq!(once_only.public_full().unwrap().player.hp, 36,
        "the settled item hook resolves once, before terminal, without a second fixed heal");

    let mut no_relic = entered_combat(30, 1);
    no_relic.enqueue_native_effects(vec![crate::combat::Effect { source: Source::None,
        target: Some(Actor::Enemy(crate::combat::EnemyHandle::parse("enemy-1").unwrap())),
        kind: crate::combat::EffectKind::Damage(DamagePayload::Fixed(20)) }], None).unwrap();
    assert_eq!(no_relic.public_full().unwrap().player.hp, 30,
        "full mode must not inherit legacy unconditional terminal healing");
}

#[test]
fn burning_blood_context_survives_full_exit_after_exactly_one_owned_heal() {
    let mut combat = entered_combat_with_card(27, 8, CardId::Bash);
    combat.install_item_context(CombatItemContext::new(vec![RelicId::BurningBlood], vec![])).unwrap();
    combat.step_full(&CombatAction::PlayCard { instance: 1, target: Some(TargetId(1)) }).unwrap();
    let exit = combat.exit().unwrap();
    assert_eq!((exit.hp, exit.max_hp), (33, 50), "Burning Blood heals 6 exactly once before Exit");
    assert!(exit.item_context.as_ref().unwrap().owns_relic(RelicId::BurningBlood));
    assert_eq!(exit.in_play.as_ref().map(|card| card.id), Some(CardId::Bash),
        "terminal handoff retains the lethal card independently of relic healing");
}

#[test]
fn burning_blood_does_not_rescue_death_and_context_is_still_exported() {
    let mut combat = entered_combat(1, 30);
    combat.install_item_context(CombatItemContext::new(vec![RelicId::BurningBlood], vec![])).unwrap();
    combat.enqueue_native_effects(vec![crate::combat::Effect { source: Source::None,
        target: Some(Actor::Player), kind: EffectKind::Damage(DamagePayload::Fixed(1)) }], None).unwrap();
    let exit = combat.exit().unwrap();
    assert_eq!(exit.outcome, crate::combat::Outcome::Death);
    assert_eq!(exit.hp, 0, "victory healing cannot rescue lethal damage");
    assert!(exit.item_context.as_ref().unwrap().owns_relic(RelicId::BurningBlood));
}

#[test]
fn fire_potion_prepares_one_targeted_twenty_damage_effect_and_consumes_its_slot() {
    let context = CombatItemContext::new(vec![], vec![None, Some(PotionId::FirePotion)]);
    let target = EnemyHandle::parse("enemy-1").unwrap();
    let facts = combat_potion_facts(std::slice::from_ref(&target));
    let use_effects = potion_use_effects(&context, 1, Some(target.clone()), &facts).unwrap();
    assert_eq!(use_effects.effects.len(), 1);
    assert_eq!(use_effects.effects[0].target, Some(Actor::Enemy(target)));
    assert_eq!(use_effects.effects[0].source, Source::Actor(Actor::Player));
    assert_eq!(use_effects.effects[0].kind,
        EffectKind::Damage(DamagePayload::Thorns(20)));
    assert_eq!(use_effects.potions, vec![None, None]);
    assert_eq!(context.potions(), &[None, Some(PotionId::FirePotion)],
        "preparation is nonmutating until the native resolver accepts the effects");

    let mut combat = entered_combat(30, 30);
    combat.install_item_context(context).unwrap();
    use_combat_potion(&mut combat, 1,
        Some(EnemyHandle::parse("enemy-1").unwrap()), false).unwrap();
    let view = combat.public_full().unwrap();
    assert_eq!(view.enemies[0].hp, 10);
    assert_eq!(combat.item_context().unwrap().potions(), &[None, None]);
}

#[test]
fn sacred_bark_doubles_fire_potion_thorns_from_owned_relic_context() {
    let context = CombatItemContext::new(vec![RelicId::SacredBark],
        vec![Some(PotionId::FirePotion)]);
    let target = EnemyHandle::parse("enemy-1").unwrap();
    let facts = combat_potion_facts(std::slice::from_ref(&target));
    let prepared = potion_use_effects(&context, 0, Some(target.clone()), &facts).unwrap();
    assert_eq!(prepared.effects.len(), 1);
    assert_eq!(prepared.effects[0].source, Source::Actor(Actor::Player));
    assert_eq!(prepared.effects[0].kind,
        EffectKind::Damage(DamagePayload::Thorns(40)));
    assert_eq!(prepared.potions, vec![None]);
    assert_eq!(context.potions(), &[Some(PotionId::FirePotion)],
        "preparation must not consume the active owner before effect acceptance");

    let mut entry = full_entry(crate::content::Character::Ironclad, 5, 10);
    let enemy = &mut entry.setup.enemies[0];
    enemy.enemy_id = EnemyId::LouseRed;
    enemy.intent = FullIntentId::Bite;
    enemy.hp = 15;
    enemy.max_hp = 15;
    entry.setup.enemy_damage_bases.insert((0, FullIntentId::Bite), 5);
    enemy.powers.push(crate::combat::FullPowerEntry {
        id: crate::combat::FullPowerId("curl_up"),
        state: crate::combat::PowerState { amount: 5, skip_first: false, monster_source_grace: false },
    });
    let context = CombatItemContext::new(vec![RelicId::SacredBark], vec![Some(PotionId::FirePotion)]);
    let mut combat = enter_with_items(entry, context);
    combat.enqueue_native_effects(vec![crate::combat::Effect { source: Source::None,
        target: Some(Actor::Enemy(target.clone())), kind: EffectKind::GainBlock(30) }], None).unwrap();
    use_combat_potion(&mut combat, 0, Some(target), false).unwrap();
    let view = combat.public_full().unwrap();
    assert_eq!((view.enemies[0].hp, view.enemies[0].block), (5, 0));
    assert!(view.enemies[0].powers.iter().any(|power| power.id.0 == "curl_up"));
    assert_eq!(combat.item_context().unwrap().potions(), &[None]);
    assert!(combat.item_context().unwrap().owns_relic(RelicId::SacredBark));
}

#[test]
fn blood_potion_and_fruit_juice_have_owned_finite_preparation_handlers() {
    for potion in [PotionId::BloodPotion, PotionId::FruitJuice] {
        let context = CombatItemContext::new(vec![], vec![Some(potion)]);
        let facts = combat_potion_facts(&[]);
        let prepared = potion_use_effects(&context, 0, None, &facts);
        assert!(prepared.is_ok(), "{potion:?} must prepare its native healing operation");
    }
}

#[test]
fn blood_potion_uses_binary32_percentage_truncation_and_no_minimum() {
    for (max_hp, bark, expected) in [(81, false, 16), (81, true, 32), (1, false, 0)] {
        let relics = if bark { vec![RelicId::SacredBark] } else { vec![] };
        let context = CombatItemContext::new(relics, vec![Some(PotionId::BloodPotion)]);
        let facts = NativePotionUseContext { phase: PotionRoomPhase::OutsideCombat,
            turn_ended: false, we_meet_again: false, hp: max_hp, max_hp, live_targets: &[] };
        let prepared = potion_use_effects(&context, 0, None, &facts).unwrap();
        assert_eq!(prepared.effects, vec![crate::combat::Effect { source: Source::Actor(Actor::Player),
            target: Some(Actor::Player), kind: EffectKind::Heal(expected) }]);
        assert_eq!(prepared.potions, vec![None]);
        assert_eq!(context.potions(), &[Some(PotionId::BloodPotion)]);
    }
}

#[test]
fn fruit_juice_requests_max_hp_first_with_bark_doubling() {
    for (bark, expected) in [(false, 5), (true, 10)] {
        let relics = if bark { vec![RelicId::SacredBark] } else { vec![] };
        let context = CombatItemContext::new(relics, vec![Some(PotionId::FruitJuice)]);
        let facts = NativePotionUseContext { phase: PotionRoomPhase::Combat,
            turn_ended: false, we_meet_again: false, hp: 40, max_hp: 50,
            live_targets: &[] };
        let prepared = potion_use_effects(&context, 0, None, &facts).unwrap();
        assert_eq!(prepared.effects, vec![crate::combat::Effect { source: Source::Actor(Actor::Player),
            target: Some(Actor::Player), kind: EffectKind::IncreaseMaxHpThenHeal(expected) }]);
    }
}

#[test]
fn special_health_potion_gates_distinguish_combat_turn_end_and_event() {
    let context = CombatItemContext::new(vec![], vec![Some(PotionId::BloodPotion)]);
    let combat_ended = NativePotionUseContext { phase: PotionRoomPhase::Combat,
        turn_ended: true, we_meet_again: false, hp: 20, max_hp: 50, live_targets: &[] };
    assert_eq!(potion_use_effects(&context, 0, None, &combat_ended), Err(RunError::InvalidAction));
    let outside_ended = NativePotionUseContext { phase: PotionRoomPhase::OutsideCombat,
        turn_ended: true, ..combat_ended };
    assert!(potion_use_effects(&context, 0, None, &outside_ended).is_ok());
    let event = NativePotionUseContext { we_meet_again: true, ..outside_ended };
    assert_eq!(potion_use_effects(&context, 0, None, &event), Err(RunError::InvalidAction));
}

#[test]
fn invalid_potion_slot_or_missing_fire_target_changes_no_owned_slot() {
    let context = CombatItemContext::new(vec![], vec![None, Some(PotionId::FirePotion)]);
    let target = EnemyHandle::parse("enemy-1").unwrap();
    let facts = combat_potion_facts(std::slice::from_ref(&target));
    assert_eq!(potion_use_effects(&context, 0, Some(target.clone()), &facts),
        Err(RunError::InvalidAction));
    assert_eq!(potion_use_effects(&context, 8, Some(target.clone()), &facts),
        Err(RunError::InvalidAction));
    assert_eq!(potion_use_effects(&context, 1, None, &facts), Err(RunError::InvalidAction));
    assert_eq!(context.potions(), &[None, Some(PotionId::FirePotion)]);
}

#[test]
fn fire_potion_has_player_ownership_and_does_not_trigger_louse_curl_up() {
    let mut entry = full_entry(crate::content::Character::Watcher, 5, 10);
    let enemy = &mut entry.setup.enemies[0];
    enemy.enemy_id = EnemyId::LouseRed;
    enemy.intent = FullIntentId::Bite;
    enemy.hp = 40;
    entry.setup.enemy_damage_bases.insert((0, FullIntentId::Bite), 5);
    enemy.powers = vec![
        crate::combat::FullPowerEntry { id: crate::combat::FullPowerId("curl_up"),
            state: crate::combat::PowerState { amount: 5, skip_first: false, monster_source_grace: false } },
        crate::combat::FullPowerEntry { id: crate::combat::FullPowerId("vulnerable"),
            state: crate::combat::PowerState { amount: 1, skip_first: false, monster_source_grace: false } },
    ];
    entry.setup.player_powers = vec![
        crate::combat::FullPowerEntry { id: crate::combat::FullPowerId("strength"),
            state: crate::combat::PowerState { amount: 9, skip_first: false, monster_source_grace: false } },
        crate::combat::FullPowerEntry { id: crate::combat::FullPowerId("weak"),
            state: crate::combat::PowerState { amount: 2, skip_first: false, monster_source_grace: false } },
    ];

    let context = CombatItemContext::new(vec![], vec![Some(PotionId::FirePotion)]);
    let target = EnemyHandle::parse("enemy-1").unwrap();
    let facts = combat_potion_facts(std::slice::from_ref(&target));
    let prepared = potion_use_effects(&context, 0, Some(target.clone()), &facts).unwrap();
    assert_eq!(prepared.effects[0].source, Source::Actor(Actor::Player),
        "Fire Potion's damage is owned by the player, not an unowned fixed packet");
    assert_eq!(prepared.effects[0].kind, EffectKind::Damage(DamagePayload::Thorns(20)),
        "Fire Potion carries its non-Attack THORNS type through the native queue");

    let mut combat = enter_with_items(entry, context);
    combat.enqueue_native_effects(vec![crate::combat::Effect { source: Source::None,
        target: Some(Actor::Player),
        kind: EffectKind::Character(crate::combat::CharacterEffect::Stance(crate::combat::Stance::Wrath)) }],
        None).unwrap();
    combat.enqueue_native_effects(vec![crate::combat::Effect { source: Source::None,
        target: Some(Actor::Enemy(target.clone())), kind: EffectKind::GainBlock(7) }], None).unwrap();
    combat.enqueue_native_effects(prepared.effects,
        Some(CombatItemContext::new(vec![], prepared.potions))).unwrap();
    let enemy = &combat.public_full().unwrap().enemies[0];
    assert_eq!(enemy.hp, 27,
        "Vulnerable's x1.5 gate is NORMAL-only: raw THORNS 20 loses 13 HP after 7 Block");
    assert_eq!(enemy.block, 0, "THORNS damage does not trigger the Louse's Curl Up");
    assert!(enemy.powers.iter().any(|power| power.id.0 == "curl_up"),
        "Curl Up remains available after THORNS damage");
}

#[test]
fn thorns_damage_rejects_an_unowned_source_without_mutating_combat() {
    let mut combat = entered_combat(30, 30);
    let before = combat.snapshot();
    let effect = crate::combat::Effect { source: Source::None,
        target: Some(Actor::Enemy(EnemyHandle::parse("enemy-1").unwrap())),
        kind: EffectKind::Damage(DamagePayload::Thorns(20)) };
    assert_eq!(combat.enqueue_native_effects(vec![effect], None), Err(RunError::InvalidAction));
    assert_eq!(crate::combat::snapshot::compare(&before, &combat.snapshot()), None,
        "the full-runtime THORNS source guard rejects before HP, Block, or queue mutation");
}

#[test]
fn invalid_native_potion_target_preserves_combat_rng_and_owned_slot() {
    let context = CombatItemContext::new(vec![], vec![Some(PotionId::FirePotion)]);
    let mut combat = entered_combat(30, 30);
    combat.install_item_context(context).unwrap();
    let before = combat.snapshot();
    let target = EnemyHandle::parse("not-an-actor").unwrap();
    let facts = combat_potion_facts(&[]);
    assert_eq!(potion_use_effects(combat.item_context().unwrap(), 0,
        Some(target), &facts), Err(RunError::InvalidAction));
    assert_eq!(crate::combat::snapshot::compare(&before, &combat.snapshot()), None,
        "invalid target must not consume a potion, mutate HP, or advance combat RNG");
}

#[test]
fn potion_use_after_terminal_is_rejected_without_consuming_the_owned_slot() {
    let context = CombatItemContext::new(vec![], vec![Some(PotionId::FirePotion)]);
    let mut combat = entered_combat(30, 1);
    combat.install_item_context(context).unwrap();
    combat.enqueue_native_effects(vec![crate::combat::Effect {
        source: Source::None,
        target: Some(Actor::Enemy(EnemyHandle::parse("enemy-1").unwrap())),
        kind: EffectKind::Damage(DamagePayload::Fixed(1)),
    }], None).unwrap();
    assert_eq!(combat.public_full().unwrap().outcome, Some(crate::combat::Outcome::Victory));
    let before = combat.snapshot();
    assert_eq!(use_combat_potion(&mut combat, 0,
        Some(EnemyHandle::parse("enemy-1").unwrap()), false), Err(RunError::InvalidAction));
    assert_eq!(crate::combat::snapshot::compare(&before, &combat.snapshot()), None);
}

#[test]
fn item_context_install_is_once_only_and_snapshot_compares_ownership() {
    let mut combat = entered_combat(30, 30);
    combat.install_item_context(CombatItemContext::default()).unwrap();
    let installed = combat.snapshot();
    assert_eq!(combat.install_item_context(CombatItemContext::new(
        vec![RelicId::BurningBlood], vec![])), Err(RunError::InvalidAction));
    assert_eq!(crate::combat::snapshot::compare(&installed, &combat.snapshot()), None);
    assert_eq!(combat.try_snapshot_v2(), Err(crate::combat::RestoreError::Compatibility));

    let mut other = entered_combat(30, 30);
    other.install_item_context(CombatItemContext::new(vec![RelicId::BurningBlood], vec![])).unwrap();
    assert_eq!(crate::combat::snapshot::compare(&installed, &other.snapshot())
        .map(|divergence| divergence.component), Some("item_context"));
}

#[test]
fn item_boundary_retries_preserve_first_typed_combat_fault() {
    let mut combat = entered_combat_with_card(30, 30, CardId::Unknown("unwired_item_fixture"));
    let first_fault = combat.step_full(&CombatAction::PlayCard {
        instance: 1, target: Some(TargetId(1)),
    }).unwrap_err();
    let RunError::Faulted(fault) = &first_fault else { panic!("first action must retain its native fault") };
    assert_eq!(fault.checkpoint, "combat.runtime");
    let crate::run::RunFaultKind::Combat(native) = &fault.kind else { panic!("combat fault packet expected") };
    assert!(native.current.is_none());
    assert!(native.pending.is_empty());
    let before_retry = combat.snapshot();
    assert_eq!(combat.install_item_context(CombatItemContext::default()), Err(first_fault.clone()));
    assert_eq!(crate::combat::snapshot::compare(&before_retry, &combat.snapshot()), None);
    assert_eq!(combat.enqueue_native_effects(vec![], None), Err(first_fault));
    assert_eq!(crate::combat::snapshot::compare(&before_retry, &combat.snapshot()), None);
}

#[test]
fn run_hook_heals_only_owned_living_player_and_respects_max_hp() {
    use crate::run::{CreationProfile, Run, RunConfig, RunRng};
    use crate::content::Character;
    let mut ironclad = Run::new(RunConfig { character: Character::Ironclad, ascension: 0 },
        CreationProfile::experienced(), RunRng::new([0; 32], crate::combat::rng::Partition::Test, 0)).unwrap();
    ironclad.state_mut_for_test().hp = 30;
    on_hook(ironclad.state_mut_for_test(), Hook::CombatSettled).unwrap();
    assert_eq!(ironclad.state().hp, 36);

    let max_hp = ironclad.state().max_hp;
    ironclad.state_mut_for_test().hp = max_hp - 2;
    on_hook(ironclad.state_mut_for_test(), Hook::CombatSettled).unwrap();
    assert_eq!(ironclad.state().hp, ironclad.state().max_hp);

    ironclad.state_mut_for_test().hp = 0;
    on_hook(ironclad.state_mut_for_test(), Hook::CombatSettled).unwrap();
    assert_eq!(ironclad.state().hp, 0, "settlement cannot revive a dead run");

    let mut silent = Run::new(RunConfig { character: Character::Silent, ascension: 0 },
        CreationProfile::experienced(), RunRng::new([0; 32], crate::combat::rng::Partition::Test, 0)).unwrap();
    silent.state_mut_for_test().hp = 30;
    on_hook(silent.state_mut_for_test(), Hook::CombatSettled).unwrap();
    assert_eq!(silent.state().hp, 30, "no phantom Burning Blood on other starts");
}
#[path="tests/m16_defect.rs"] mod m16_defect;
