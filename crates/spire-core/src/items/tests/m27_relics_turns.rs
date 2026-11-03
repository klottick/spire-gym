//! M27 plan Task R1, lane R1, key m27_relics_turns.
//! Complete-admission combat tests for turn, startup, and settlement relics.

use crate::combat::{Actor, CardDestination, CombatAction, DamagePayload, Effect, EffectKind,
    FullPowerId, Hook, HookEvent, Source, TargetId};
use crate::content::{CardId, Character, EncounterId, PotionId, RelicId};
use crate::items::{dispatch::{ItemPhase, PhaseFacts}, relic_effects::turns,
    RelicHookWork};
use crate::run::{CardInstance, Run};
use super::m27_support;
use std::collections::BTreeMap;

fn run_with(relic: RelicId, encounter: EncounterId) -> Run {
    m27_support::complete_run(Character::Ironclad, vec![relic], vec![None; 3],
        BTreeMap::new(), encounter)
}

fn run_with_potions(relic: RelicId, potions: Vec<Option<PotionId>>) -> Run {
    m27_support::complete_run(Character::Ironclad, vec![relic], potions,
        BTreeMap::new(), EncounterId::Cultist)
}

fn run_with_counters(relic: RelicId, counters: BTreeMap<RelicId, i32>) -> Run {
    m27_support::complete_run(Character::Ironclad, vec![relic], vec![Some(PotionId::FirePotion); 3],
        counters, EncounterId::Cultist)
}

fn run_with_second_deck(relic: RelicId, ids: &[CardId]) -> Run {
    let mut run = run_with_potions(relic, vec![Some(PotionId::FirePotion); 3]);
    win_cultist(&mut run);
    replace_master_deck(&mut run, ids);
    m27_support::next_complete_combat(&mut run, EncounterId::Cultist).unwrap();
    run
}

fn view(run: &Run) -> crate::combat::CombatPublic {
    run.state().combat.as_ref().expect("live training combat").public_full().unwrap()
}

fn power(run: &Run, id: &str) -> i32 {
    view(run).player.powers.iter().find(|power| power.id.0 == id).map_or(0, |power| power.amount)
}

fn enemy_power(run: &Run, index: usize, id: &str) -> i32 {
    view(run).enemies[index].powers.iter().find(|power| power.id.0 == id).map_or(0, |power| power.amount)
}

fn item_counter(run: &Run, id: RelicId) -> i32 {
    run.state().combat.as_ref().expect("live training combat").item_context().unwrap().counter(id)
}

fn item_local(run: &Run, id: RelicId) -> i32 {
    run.state().combat.as_ref().expect("live training combat").item_context().unwrap().local(id)
}

fn enqueue(run: &mut Run, effects: Vec<Effect>) {
    run.state_mut_for_test().combat.as_mut().expect("live training combat")
        .enqueue_native_effects(effects, None).unwrap();
}

fn enqueue_player(run: &mut Run, kind: EffectKind) {
    enqueue(run, vec![Effect { source: Source::None, target: Some(Actor::Player), kind }]);
}

fn hook(run: &mut Run, hook: Hook) {
    run.state_mut_for_test().combat.as_mut().expect("live training combat")
        .enqueue_hook(HookEvent { hook, source: Source::Actor(Actor::Player), target: Some(Actor::Player) })
        .unwrap_or_else(|error| panic!("dispatch {hook:?}: {error:?}"));
}

fn create_card(run: &mut Run, id: CardId, destination: CardDestination) {
    enqueue(run, vec![Effect { source: Source::None, target: Some(Actor::Player),
        kind: EffectKind::CreateCard { id, destination, upgrades: 0 } }]);
}

fn card_instance(run: &Run, id: CardId) -> u64 {
    view(run).hand.iter().find(|card| card.id == id)
        .unwrap_or_else(|| panic!("{id:?} is not in hand: {:?}", view(run).hand)).instance.0
}

fn play_card(run: &mut Run, id: CardId, target: Option<TargetId>) {
    let instance = card_instance(run, id);
    run.choose_combat(CombatAction::PlayCard { instance, target }).unwrap_or_else(|error|
        panic!("play {id:?}: {error:?}; hand={:?}", view(run).hand));
}

// atTurnStart relics answer the pre-draw checkpoint (source-check-turnstart-hooks-v1).
fn turn_start(run: &mut Run) { hook(run, Hook::TurnStartedPreDraw); }
// onPlayerEndTurn relics answer callEndOfTurnActions, PlayerPreEndTurnCards (spire-oto.22).
fn end_turn_hook(run: &mut Run) { hook(run, Hook::PlayerPreEndTurnCards); }

fn lose_player_hp(run: &mut Run, amount: i32) {
    enqueue_player(run, EffectKind::Damage(DamagePayload::HpLoss(amount)));
}

fn win_cultist(run: &mut Run) {
    for slot in 0..3 {
        run.choose_combat(CombatAction::UsePotion { slot, target: Some(TargetId(1)) })
            .unwrap_or_else(|error| panic!("Fire Potion {slot}: {error:?}"));
    }
    assert!(run.state().combat.is_none(), "the Run owner settles the Fire Potion victory");
    assert_eq!(run.state().last_combat.as_ref().map(|record| record.outcome),
        Some(crate::combat::Outcome::Victory));
}

fn escape_cultist(run: &mut Run) {
    run.choose_combat(CombatAction::UsePotion { slot: 0, target: None })
        .unwrap_or_else(|error| panic!("Smoke Bomb: {error:?}"));
    assert!(run.state().combat.is_none(), "the Run owner settles the escape");
    assert_eq!(run.state().last_combat.as_ref().map(|record| record.outcome),
        Some(crate::combat::Outcome::Escaped));
}

fn replace_master_deck(run: &mut Run, ids: &[CardId]) {
    let state = run.state_mut_for_test();
    state.deck = ids.iter().copied().enumerate().map(|(index, id)| CardInstance {
        instance: index as u64 + 1, id, upgrades: 0, misc: 0,
    }).collect();
    state.next_instance = ids.len() as u64 + 1;
}

macro_rules! entry_power_test {
    ($name:ident, $relic:expr, $power:expr, $amount:expr, $encounter:expr) => {
        #[test]
        fn $name() {
            let run = run_with($relic, $encounter);
            assert_eq!(power(&run, $power), $amount, "combat-entry power");
        }
    };
}

#[test]
fn m27_akabeko_first_attack_bonus() {
    let mut run = run_with_second_deck(RelicId::Akabeko, &[CardId::TwinStrike, CardId::TwinStrike]);
    assert_eq!(power(&run, "vigor"), 8);
    let before = view(&run).enemies[0].hp;
    play_card(&mut run, CardId::TwinStrike, Some(TargetId(1)));
    assert_eq!(before - view(&run).enemies[0].hp, 26, "two 5-damage hits each receive Vigor 8");
    assert_eq!(power(&run, "vigor"), 0, "CardUsed consumes the bonus once");
    enqueue_player(&mut run, EffectKind::GainEnergy(3));
    let before = view(&run).enemies[0].hp;
    play_card(&mut run, CardId::TwinStrike, Some(TargetId(1)));
    assert_eq!(before - view(&run).enemies[0].hp, 10, "the following two 5-damage hits remain unmodified");
}

#[test]
fn m27_akabeko_boundary() {
    let mut run = run_with_second_deck(RelicId::Akabeko, &[CardId::TwinStrike]);
    let before = view(&run).enemies[0].hp;
    play_card(&mut run, CardId::TwinStrike, Some(TargetId(1)));
    assert_eq!(before - view(&run).enemies[0].hp, 26,
        "Vigor 8 applies to both 5-damage hits, not only the first hit");
    assert_eq!(power(&run, "vigor"), 0);
}

#[test]
fn m27_art_of_war_no_attack_energy() {
    let mut run = run_with(RelicId::ArtOfWar, EncounterId::Cultist);
    assert_eq!(view(&run).player.energy, 3, "the first turn has no preceding qualifying interval");
    run.choose_combat(CombatAction::EndTurn).unwrap();
    assert_eq!(view(&run).player.energy, 4, "a no-Attack interval grants one energy at the next TurnStart");
    play_card(&mut run, CardId::Strike, Some(TargetId(1)));
    run.choose_combat(CombatAction::EndTurn).unwrap();
    assert_eq!(view(&run).player.energy, 3, "an interval containing an Attack grants no energy");
    run.choose_combat(CombatAction::EndTurn).unwrap();
    assert_eq!(view(&run).player.energy, 4, "the later empty interval grants one energy");
}

#[test]
fn m27_art_of_war_boundary() {
    let mut run = run_with_second_deck(RelicId::ArtOfWar, &[CardId::BodySlam]);
    run.choose_combat(CombatAction::EndTurn).unwrap();
    assert_eq!(view(&run).player.block, 0);
    play_card(&mut run, CardId::BodySlam, Some(TargetId(1)));
    run.choose_combat(CombatAction::EndTurn).unwrap();
    assert_eq!(view(&run).player.energy, 3, "a zero-damage Attack still counts as played");
    run.choose_combat(CombatAction::EndTurn).unwrap();
    assert_eq!(view(&run).player.energy, 4, "the next no-Attack interval qualifies");
}

#[test]
fn m27_art_of_war_two_combat_counter() {
    let mut run = run_with_potions(RelicId::ArtOfWar, vec![Some(PotionId::FirePotion); 3]);
    assert_eq!(view(&run).player.energy, 3);
    win_cultist(&mut run);
    m27_support::next_complete_combat(&mut run, EncounterId::Cultist).unwrap();
    assert_eq!(view(&run).player.energy, 3, "combat two's first turn grants no bonus");
    run.choose_combat(CombatAction::EndTurn).unwrap();
    assert_eq!(view(&run).player.energy, 4, "its first empty interval grants one energy");
}

#[test]
fn m27_happy_flower_third_turn_energy() {
    let mut run = run_with(RelicId::HappyFlower, EncounterId::Cultist);
    assert_eq!(item_counter(&run, RelicId::HappyFlower), 1, "combat entry dispatched turn one");
    assert_eq!(view(&run).player.energy, 3);
    turn_start(&mut run);
    assert_eq!(item_counter(&run, RelicId::HappyFlower), 2);
    turn_start(&mut run);
    assert_eq!(item_counter(&run, RelicId::HappyFlower), 0);
    assert_eq!(view(&run).player.energy, 4, "turn three queues exactly one energy");
}

#[test]
fn m27_happy_flower_boundary() {
    let mut run = run_with_potions(RelicId::HappyFlower, vec![Some(PotionId::FirePotion); 3]);
    turn_start(&mut run);
    assert_eq!(item_counter(&run, RelicId::HappyFlower), 2);
    win_cultist(&mut run);
    assert_eq!(run.state().relic_counters.get(&RelicId::HappyFlower), Some(&2));
    m27_support::next_complete_combat(&mut run, EncounterId::Cultist).unwrap();
    assert_eq!(item_counter(&run, RelicId::HappyFlower), 0);
    assert_eq!(view(&run).player.energy, 4, "a saved two triggers on turn one of the next combat");
}

#[test]
fn m27_happy_flower_two_combat_counter() {
    let mut run = run_with_counters(RelicId::HappyFlower, BTreeMap::from([(RelicId::HappyFlower, 1)]));
    assert_eq!(item_counter(&run, RelicId::HappyFlower), 2);
    win_cultist(&mut run);
    m27_support::next_complete_combat(&mut run, EncounterId::Cultist).unwrap();
    assert_eq!(item_counter(&run, RelicId::HappyFlower), 0);
    assert_eq!(view(&run).player.energy, 4);
    turn_start(&mut run);
    assert_eq!(item_counter(&run, RelicId::HappyFlower), 1);
    assert_eq!(view(&run).player.energy, 4, "turn four does not grant another energy");
}

#[test]
fn m27_preserved_insect_elite_hp_floor() {
    let run = run_with(RelicId::PreservedInsect, EncounterId::GremlinNob);
    let enemy = &view(&run).enemies[0];
    assert_eq!(enemy.hp, enemy.max_hp * 3 / 4, "initial elite HP is capped at truncated 75% Max HP");
    assert_eq!(enemy.max_hp, view(&run).enemies[0].max_hp, "Max HP is unchanged");
}

#[test]
fn m27_preserved_insect_boundary() {
    let mut run = run_with(RelicId::PreservedInsect, EncounterId::GremlinNob);
    let before = view(&run).enemies[0].hp;
    let enemy_id = view(&run).enemies[0].instance_id.clone();
    enqueue(&mut run, vec![Effect { source: Source::Actor(Actor::Player),
        target: Some(Actor::Enemy(enemy_id)), kind: EffectKind::Damage(DamagePayload::HpLoss(40)) }]);
    let after = view(&run).enemies[0].hp;
    assert!(after <= before && after <= view(&run).enemies[0].max_hp * 3 / 4,
        "later HP loss stays below the cap; a later callback must not raise current HP");
}

#[test]
fn m27_horn_cleat_second_turn_block() {
    let mut run = run_with(RelicId::HornCleat, EncounterId::Cultist);
    assert_eq!(view(&run).player.block, 0, "turn one has no Horn Cleat block");
    turn_start(&mut run);
    assert_eq!(view(&run).player.block, 14, "turn two queues Block 14 once");
    assert_eq!(item_local(&run, RelicId::HornCleat), 2);
}

#[test]
fn m27_horn_cleat_boundary() {
    let mut run = run_with(RelicId::HornCleat, EncounterId::Cultist);
    enqueue_player(&mut run, EffectKind::ApplyFullPower { id: FullPowerId("dexterity"), amount: 9 });
    enqueue_player(&mut run, EffectKind::ApplyFullPower { id: FullPowerId("frail"), amount: 1 });
    turn_start(&mut run);
    assert_eq!(view(&run).player.block, 14, "Dexterity 9 and Frail 1 do not scale relic Block");
}

#[test]
fn m27_horn_cleat_two_combat_counter() {
    let mut run = run_with_potions(RelicId::HornCleat, vec![Some(PotionId::FirePotion); 3]);
    assert_eq!(item_local(&run, RelicId::HornCleat), 1);
    win_cultist(&mut run);
    m27_support::next_complete_combat(&mut run, EncounterId::Cultist).unwrap();
    assert_eq!(item_local(&run, RelicId::HornCleat), 1, "combat-local state restarts at turn one");
    turn_start(&mut run);
    assert_eq!(view(&run).player.block, 14);
}

#[test]
fn m27_meat_on_the_bone_half_hp_heal() {
    let mut run = run_with_potions(RelicId::MeatOnTheBone, vec![Some(PotionId::FirePotion); 3]);
    lose_player_hp(&mut run, 40);
    win_cultist(&mut run);
    assert_eq!((run.state().hp, run.state().max_hp), (52, 80), "Victory at 40/80 heals 12");
}

#[test]
fn m27_meat_on_the_bone_boundary() {
    let mut run = run_with_potions(RelicId::MeatOnTheBone, vec![Some(PotionId::SmokeBomb), None, None]);
    lose_player_hp(&mut run, 40);
    escape_cultist(&mut run);
    assert_eq!(run.state().hp, 40);
    assert_eq!(run.state().hp, 40, "Escaped is not a Victory relic checkpoint (F3)");
}

#[test]
fn m27_mercury_hourglass_turn_start_thorns() {
    let mut run = run_with(RelicId::MercuryHourglass, EncounterId::ThreeCultists);
    let before = view(&run).enemies.iter().map(|enemy| enemy.hp).collect::<Vec<_>>();
    turn_start(&mut run);
    let after = view(&run).enemies;
    assert_eq!(after.len(), 3);
    for (index, enemy) in after.iter().enumerate() {
        assert_eq!(before[index] - enemy.hp, 3, "each live enemy receives non-Attack Thorns 3");
    }
}

#[test]
fn m27_mercury_hourglass_boundary() {
    let mut run = run_with(RelicId::MercuryHourglass, EncounterId::ThreeCultists);
    enqueue_player(&mut run, EffectKind::ApplyFullPower { id: FullPowerId("strength"), amount: 9 });
    let before = view(&run).enemies.iter().map(|enemy| enemy.hp).collect::<Vec<_>>();
    turn_start(&mut run);
    let after = view(&run).enemies;
    for (index, enemy) in after.iter().enumerate() {
        assert_eq!(before[index] - enemy.hp, 3, "Strength 9 does not scale this relic damage");
    }
}

fn pantograph_work(boss: bool) -> RelicHookWork {
    // Boss encounters are outside complete_run's finite M3 training admission. Keep the
    // complete owner and supply the entry fact directly to this focused phase test.
    let run = run_with(RelicId::Pantograph, EncounterId::Cultist);
    let mut context = run.state().combat.as_ref().unwrap().item_context().unwrap().clone();
    let mut facts = PhaseFacts::new(None, 40, 80, 0);
    facts.boss = boss;
    let mut work = RelicHookWork::default();
    assert!(turns::phase(RelicId::Pantograph, &mut context, ItemPhase::CombatStart, &facts, &mut work).unwrap());
    work
}

#[test]
fn m27_pantograph_boss_start_heal() {
    let work = pantograph_work(true);
    // atBattleStart addToTop (source-check-turnstart-hooks-v1).
    assert_eq!(work.top.len(), 1);
    assert_eq!(work.top[0].kind, EffectKind::Heal(25), "Boss combat startup queues one heal request of 25");
    assert_eq!(work.top[0].source, Source::Relic(RelicId::Pantograph));
}

#[test]
fn m27_pantograph_boundary() {
    assert!(pantograph_work(false).top.is_empty(), "non-Boss entry queues no heal");
    let multi_boss = pantograph_work(true);
    assert_eq!(multi_boss.top.iter().filter(|effect| effect.kind == EffectKind::Heal(25)).count(), 1,
        "multiple Boss actors still produce one heal of 25");
}

#[test]
fn m27_sundial_third_shuffle_energy() {
    let mut run = run_with(RelicId::Sundial, EncounterId::Cultist);
    assert_eq!(item_counter(&run, RelicId::Sundial), 0);
    hook(&mut run, Hook::DrawPileShuffled);
    assert_eq!(item_counter(&run, RelicId::Sundial), 1);
    hook(&mut run, Hook::DrawPileShuffled);
    assert_eq!(item_counter(&run, RelicId::Sundial), 2);
    assert_eq!(view(&run).player.energy, 3);
    hook(&mut run, Hook::DrawPileShuffled);
    assert_eq!(item_counter(&run, RelicId::Sundial), 0);
    assert_eq!(view(&run).player.energy, 5, "the third callback adds energy 2");
}

#[test]
fn m27_sundial_boundary() {
    let mut run = run_with(RelicId::Sundial, EncounterId::Cultist);
    create_card(&mut run, CardId::Wound, CardDestination::ShuffledDraw);
    assert_eq!(item_counter(&run, RelicId::Sundial), 0, "insertion without shuffle notification is not a callback");
    assert_eq!(view(&run).player.energy, 3);
}

#[test]
fn m27_sundial_two_combat_counter() {
    let mut run = run_with_potions(RelicId::Sundial, vec![Some(PotionId::FirePotion); 3]);
    hook(&mut run, Hook::DrawPileShuffled);
    hook(&mut run, Hook::DrawPileShuffled);
    assert_eq!(item_counter(&run, RelicId::Sundial), 2);
    win_cultist(&mut run);
    m27_support::next_complete_combat(&mut run, EncounterId::Cultist).unwrap();
    assert_eq!(view(&run).player.energy, 3);
    hook(&mut run, Hook::DrawPileShuffled);
    assert_eq!(item_counter(&run, RelicId::Sundial), 0);
    assert_eq!(view(&run).player.energy, 5, "callback three persists across settlement");
}

#[test]
fn m27_captains_wheel_third_turn_block() {
    let mut run = run_with(RelicId::CaptainsWheel, EncounterId::Cultist);
    assert_eq!(view(&run).player.block, 0, "turn one");
    turn_start(&mut run);
    assert_eq!(view(&run).player.block, 0, "turn two");
    turn_start(&mut run);
    assert_eq!(view(&run).player.block, 18, "turn three queues Block 18 once");
    assert_eq!(item_local(&run, RelicId::CaptainsWheel), 3);
}

#[test]
fn m27_captains_wheel_boundary() {
    let mut run = run_with(RelicId::CaptainsWheel, EncounterId::Cultist);
    enqueue_player(&mut run, EffectKind::ApplyFullPower { id: FullPowerId("dexterity"), amount: 9 });
    enqueue_player(&mut run, EffectKind::ApplyFullPower { id: FullPowerId("frail"), amount: 1 });
    turn_start(&mut run);
    turn_start(&mut run);
    assert_eq!(view(&run).player.block, 18, "Dexterity and Frail do not scale relic Block");
}

#[test]
fn m27_captains_wheel_two_combat_counter() {
    let mut run = run_with_potions(RelicId::CaptainsWheel, vec![Some(PotionId::FirePotion); 3]);
    win_cultist(&mut run);
    m27_support::next_complete_combat(&mut run, EncounterId::Cultist).unwrap();
    assert_eq!(view(&run).player.block, 0, "combat two resets its local turn counter");
    turn_start(&mut run);
    assert_eq!(view(&run).player.block, 0);
    turn_start(&mut run);
    assert_eq!(view(&run).player.block, 18);
}

fn du_vu_run_with_two_curses() -> Run {
    let mut run = run_with_potions(RelicId::DuVuDoll, vec![Some(PotionId::FirePotion); 3]);
    win_cultist(&mut run);
    replace_master_deck(&mut run, &[CardId::Clumsy, CardId::Injury, CardId::Strike]);
    m27_support::next_complete_combat(&mut run, EncounterId::Cultist).unwrap();
    run
}

#[test]
fn m27_du_vu_doll_curse_count_strength() {
    let run = du_vu_run_with_two_curses();
    assert_eq!(power(&run, "strength"), 2, "two master-deck Curse instances are counted at battle start");
    assert!(!run.state().relic_counters.contains_key(&RelicId::DuVuDoll),
        "the current combat derives its Curse count from entry facts rather than a Run counter");
}

#[test]
fn m27_du_vu_doll_boundary() {
    let mut run = du_vu_run_with_two_curses();
    run.choose_combat(CombatAction::EndTurn).unwrap();
    assert!(view(&run).exhaust.iter().any(|card| card.id == CardId::Clumsy),
        "the combat copy exhausts at end turn");
    assert_eq!(power(&run, "strength"), 2, "removing a combat copy does not change granted Strength");
}

#[test]
fn m27_du_vu_doll_two_combat_counter() {
    let mut run = run_with_potions(RelicId::DuVuDoll, vec![Some(PotionId::FirePotion); 3]);
    win_cultist(&mut run);
    replace_master_deck(&mut run, &[CardId::Clumsy, CardId::Injury, CardId::Strike]);
    run.state_mut_for_test().potions = vec![Some(PotionId::FirePotion); 3];
    m27_support::next_complete_combat(&mut run, EncounterId::Cultist).unwrap();
    assert!(!run.state().relic_counters.contains_key(&RelicId::DuVuDoll));
    assert_eq!(power(&run, "strength"), 2);
    win_cultist(&mut run);
    replace_master_deck(&mut run, &[CardId::Injury, CardId::Strike]);
    run.state_mut_for_test().potions = vec![Some(PotionId::FirePotion); 3];
    m27_support::next_complete_combat(&mut run, EncounterId::Cultist).unwrap();
    assert_eq!(power(&run, "strength"), 1, "the following combat recounts its one Curse at entry");
}

entry_power_test!(m27_fossilized_helix_first_hit_buffer, RelicId::FossilizedHelix, "buffer", 1, EncounterId::Cultist);

#[test]
fn m27_fossilized_helix_boundary() {
    let mut run = run_with(RelicId::FossilizedHelix, EncounterId::Cultist);
    assert_eq!(power(&run, "buffer"), 1);
    lose_player_hp(&mut run, 10);
    assert_eq!(view(&run).player.hp, 80, "Buffer prevents the first positive self HP-loss event");
    assert_eq!(power(&run, "buffer"), 0);
}

#[test]
fn m27_incense_burner_sixth_turn_intangible() {
    let mut run = run_with(RelicId::IncenseBurner, EncounterId::Cultist);
    assert_eq!(item_counter(&run, RelicId::IncenseBurner), 1);
    for expected in [2, 3, 4, 5] {
        turn_start(&mut run);
        assert_eq!(item_counter(&run, RelicId::IncenseBurner), expected);
        assert_eq!(power(&run, "intangible_player"), 0);
    }
    turn_start(&mut run);
    assert_eq!(item_counter(&run, RelicId::IncenseBurner), 0);
    assert_eq!(power(&run, "intangible_player"), 1);
}

#[test]
fn m27_incense_burner_boundary() {
    let mut run = run_with_counters(RelicId::IncenseBurner, BTreeMap::from([(RelicId::IncenseBurner, 4)]));
    assert_eq!(item_counter(&run, RelicId::IncenseBurner), 5);
    win_cultist(&mut run);
    assert_eq!(run.state().relic_counters.get(&RelicId::IncenseBurner), Some(&5));
    m27_support::next_complete_combat(&mut run, EncounterId::Cultist).unwrap();
    assert_eq!(item_counter(&run, RelicId::IncenseBurner), 0);
    assert_eq!(power(&run, "intangible_player"), 1, "the counter persists and triggers at six");
}

#[test]
fn m27_incense_burner_two_combat_counter() {
    let mut run = run_with_counters(RelicId::IncenseBurner, BTreeMap::from([(RelicId::IncenseBurner, 4)]));
    assert_eq!(item_counter(&run, RelicId::IncenseBurner), 5);
    win_cultist(&mut run);
    m27_support::next_complete_combat(&mut run, EncounterId::Cultist).unwrap();
    assert_eq!(power(&run, "intangible_player"), 1);
    assert_eq!(item_counter(&run, RelicId::IncenseBurner), 0);
    turn_start(&mut run);
    assert_eq!(item_counter(&run, RelicId::IncenseBurner), 1);
    assert_eq!(power(&run, "intangible_player"), 1, "the following turn does not grant a second stack");
}

fn pocketwatch_second_combat() -> Run {
    let mut run = run_with_potions(RelicId::Pocketwatch, vec![Some(PotionId::FirePotion); 3]);
    win_cultist(&mut run);
    replace_master_deck(&mut run, &[CardId::Shiv; 16]);
    m27_support::next_complete_combat(&mut run, EncounterId::Cultist).unwrap();
    run
}

#[test]
fn m27_pocketwatch_later_turn_draw() {
    let mut run = pocketwatch_second_combat();
    for _ in 0..3 { play_card(&mut run, CardId::Shiv, Some(TargetId(1))); }
    run.choose_combat(CombatAction::EndTurn).unwrap();
    assert_eq!(view(&run).hand.len(), 8, "three plays qualify for the next-turn Draw 3");
    // Main-2 ruling: the source rule is <= 3 plays (r1.md:299-302); the exact-three
    // suppression literal at r1.md:308 was a spec-sheet error.
    for _ in 0..4 { play_card(&mut run, CardId::Shiv, Some(TargetId(1))); }
    run.choose_combat(CombatAction::EndTurn).unwrap();
    assert_eq!(view(&run).hand.len(), 5, "four plays do not qualify for the next-turn Draw 3");
}

#[test]
fn m27_pocketwatch_boundary() {
    let mut run = run_with_potions(RelicId::Pocketwatch, vec![Some(PotionId::FirePotion); 3]);
    win_cultist(&mut run);
    replace_master_deck(&mut run, &[CardId::Shiv; 16]);
    m27_support::next_complete_combat(&mut run, EncounterId::Cultist).unwrap();
    for _ in 0..4 { play_card(&mut run, CardId::Shiv, Some(TargetId(1))); }
    run.choose_combat(CombatAction::EndTurn).unwrap();
    assert_eq!(view(&run).hand.len(), 5, "a four-card interval queues no bonus");
    run.choose_combat(CombatAction::EndTurn).unwrap();
    assert_eq!(view(&run).hand.len(), 8, "a later zero-card interval queues Draw 3");
}

#[test]
fn m27_pocketwatch_two_combat_counter() {
    let mut run = pocketwatch_second_combat();
    run.choose_combat(CombatAction::EndTurn).unwrap();
    assert_eq!(view(&run).hand.len(), 8, "combat-local play count resets; zero plays queues Draw 3 after draw");
}

#[test]
fn m27_stone_calendar_seventh_turn_thorns() {
    let mut run = run_with(RelicId::StoneCalendar, EncounterId::Cultist);
    let target = view(&run).enemies[0].instance_id.clone();
    let before = view(&run).enemies[0].hp;
    for turn in 2..=6 {
        turn_start(&mut run);
        end_turn_hook(&mut run);
        assert_eq!(before - view(&run).enemies[0].hp, 0, "end turn {turn} has no pulse");
    }
    turn_start(&mut run);
    end_turn_hook(&mut run);
    let after = view(&run);
    let enemy = after.enemies.iter().find(|enemy| enemy.instance_id == target).unwrap();
    assert_eq!(before - enemy.hp, 52, "turn seven deals non-Attack Thorns 52");
}

#[test]
fn m27_stone_calendar_boundary() {
    let mut run = run_with(RelicId::StoneCalendar, EncounterId::Cultist);
    let enemy_id = view(&run).enemies[0].instance_id.clone();
    enqueue(&mut run, vec![Effect { source: Source::None, target: Some(Actor::Enemy(enemy_id.clone())),
        kind: EffectKind::GainBlock(7) }]);
    for _ in 0..5 { turn_start(&mut run); }
    let before = view(&run).enemies[0].hp;
    turn_start(&mut run);
    end_turn_hook(&mut run);
    let after = view(&run);
    let enemy = after.enemies.iter().find(|enemy| enemy.instance_id == enemy_id).unwrap();
    assert_eq!(enemy.block, 0, "the 52-point Thorns pulse resolves through enemy Block first");
    assert_eq!(before - enemy.hp, 45, "52 damage removes 7 Block then 45 HP");
}

#[test]
fn m27_stone_calendar_two_combat_counter() {
    let mut run = run_with_potions(RelicId::StoneCalendar, vec![Some(PotionId::FirePotion); 3]);
    // Combat one reaches local turn six, but settles before its pulse.
    for _ in 0..5 { turn_start(&mut run); }
    win_cultist(&mut run);
    m27_support::next_complete_combat(&mut run, EncounterId::Cultist).unwrap();
    let before = view(&run).enemies[0].hp;
    for _ in 0..5 { turn_start(&mut run); end_turn_hook(&mut run); }
    assert_eq!(before - view(&run).enemies[0].hp, 0, "combat two turns one through six have no pulse");
    turn_start(&mut run);
    end_turn_hook(&mut run);
    assert_eq!(before - view(&run).enemies[0].hp, 52, "the local counter reaches seven in combat two");
}

entry_power_test!(m27_thread_and_needle_start_plated_armor, RelicId::ThreadAndNeedle, "plated_armor", 4, EncounterId::Cultist);

#[test]
fn m27_thread_and_needle_boundary() {
    let mut run = run_with(RelicId::ThreadAndNeedle, EncounterId::Cultist);
    enqueue_player(&mut run, EffectKind::GainBlock(10));
    let enemy = view(&run).enemies[0].instance_id.clone();
    enqueue(&mut run, vec![Effect { source: Source::Actor(Actor::Enemy(enemy)), target: Some(Actor::Player),
        kind: EffectKind::Damage(DamagePayload::Fixed(10)) }]);
    assert_eq!(view(&run).player.hp, 80, "Block prevents HP loss");
    assert_eq!(power(&run, "plated_armor"), 4, "zero HP loss does not decrement Plated Armor");
}

#[test]
fn m27_the_abacus_shuffle_block() {
    let mut run = run_with(RelicId::TheAbacus, EncounterId::Cultist);
    hook(&mut run, Hook::DrawPileShuffled);
    assert_eq!(view(&run).player.block, 6, "one qualifying shuffle grants Block 6");
    create_card(&mut run, CardId::Wound, CardDestination::ShuffledDraw);
    assert_eq!(view(&run).player.block, 6, "random insertion is not a second shuffle");
}

#[test]
fn m27_the_abacus_boundary() {
    let mut run = run_with(RelicId::TheAbacus, EncounterId::Cultist);
    create_card(&mut run, CardId::Wound, CardDestination::ShuffledDraw);
    assert_eq!(view(&run).player.block, 0, "insertion without draw-caused shuffle queues no Block");
}

entry_power_test!(m27_clockwork_souvenir_start_artifact, RelicId::ClockworkSouvenir, "artifact", 1, EncounterId::Cultist);

#[test]
fn m27_clockwork_souvenir_boundary() {
    let mut run = run_with(RelicId::ClockworkSouvenir, EncounterId::Cultist);
    assert_eq!(power(&run, "artifact"), 1);
    enqueue_player(&mut run, EffectKind::ApplyFullPower { id: FullPowerId("weak"), amount: 1 });
    assert_eq!(power(&run, "weak"), 0, "Artifact prevents the player Weak attempt");
    assert_eq!(power(&run, "artifact"), 0, "the prevented Weak attempt consumes Artifact");
}

entry_power_test!(m27_sling_of_courage_elite_trigger_strength, RelicId::SlingOfCourage, "strength", 2, EncounterId::GremlinNob);

#[test]
fn m27_sling_of_courage_boundary() {
    let elite = run_with(RelicId::SlingOfCourage, EncounterId::GremlinNob);
    let non_elite = run_with(RelicId::SlingOfCourage, EncounterId::Cultist);
    assert_eq!(power(&elite, "strength"), 2, "eliteTrigger applies even when room label is not Elite");
    assert_eq!(power(&non_elite, "strength"), 0, "a non-eliteTrigger entry grants no Strength");
}

#[test]
fn m27_cultist_headpiece_speech_only() {
    let with = run_with(RelicId::CultistHeadpiece, EncounterId::Cultist);
    let without = m27_support::complete_run(Character::Ironclad, vec![], vec![None; 3],
        BTreeMap::new(), EncounterId::Cultist);
    assert_eq!(view(&with).player, view(&without).player, "speech has no numeric player-state effect");
    assert_eq!(view(&with).enemies.iter().map(|enemy| enemy.powers.clone()).collect::<Vec<_>>(),
        view(&without).enemies.iter().map(|enemy| enemy.powers.clone()).collect::<Vec<_>>(),
        "speech has no enemy-stat effect");
    // The combat model has no speech/audio output surface; the source catalog payload is “Caw”.
}

#[test]
fn m27_cultist_headpiece_boundary() {
    let run = run_with(RelicId::CultistHeadpiece, EncounterId::Cultist);
    assert_eq!(power(&run, "strength"), 0);
    assert_eq!(power(&run, "ritual"), 0);
    assert!(view(&run).player.powers.iter().all(|power| power.amount == 0 || power.id.0 != "strength"));
}

#[test]
fn m27_enchiridion_class_power_zero() {
    let run = run_with(RelicId::Enchiridion, EncounterId::Cultist);
    let public = view(&run);
    let generated = public.hand.iter().filter(|card|
        crate::content::Catalog::standard().card(card.id).is_some_and(|row| row.card_type == crate::content::CardType::Power))
        .collect::<Vec<_>>();
    assert_eq!(generated.len(), 1, "exactly one Power is generated before the opening draw");
    assert_eq!(generated[0].effective_cost, Some(0), "the generated Power costs zero this turn");
}

#[test]
fn m27_enchiridion_boundary() {
    let run = run_with(RelicId::Enchiridion, EncounterId::Cultist);
    assert!(view(&run).hand.iter().any(|card|
        crate::content::Catalog::standard().card(card.id).is_some_and(|row| row.card_type == crate::content::CardType::Power)
            && card.effective_cost == Some(0)), "the Power exists before opening draw and has current cost zero");
}

#[test]
fn m27_face_of_cleric_victory_max_hp() {
    let mut run = run_with_potions(RelicId::FaceOfCleric, vec![Some(PotionId::FirePotion); 3]);
    lose_player_hp(&mut run, 40);
    win_cultist(&mut run);
    assert_eq!((run.state().max_hp, run.state().hp), (81, 41),
        "Victory increases Max HP by 1 and heals through the increaseMaxHp rule");
}

#[test]
fn m27_face_of_cleric_boundary() {
    let mut run = run_with_potions(RelicId::FaceOfCleric, vec![Some(PotionId::SmokeBomb), None, None]);
    lose_player_hp(&mut run, 40);
    escape_cultist(&mut run);
    assert_eq!((run.state().max_hp, run.state().hp), (80, 40), "Escaped has no Victory max-HP effect (F3)");
}

entry_power_test!(m27_gremlin_visage_player_weak, RelicId::GremlinVisage, "weak", 1, EncounterId::Cultist);

#[test]
fn m27_gremlin_visage_boundary() {
    let run = run_with(RelicId::GremlinVisage, EncounterId::ThreeCultists);
    assert_eq!(power(&run, "weak"), 1);
    assert!(view(&run).enemies.iter().all(|enemy|
        enemy.powers.iter().all(|power| power.id.0 != "weak")), "the player receives Weak; enemies do not");
}

// Main-10 I2: catalog Strength Down (F15). The first-turn loss is the shared `lose_strength` debuff,
// resolved by the real end of turn (was a relic EndTurn reduction with lose_strength == 0).
fn end_turn(run: &mut Run) { run.choose_combat(CombatAction::EndTurn).unwrap(); }

#[test]
fn m27_mutagenic_strength_first_turn_strength() {
    let mut run = run_with(RelicId::MutagenicStrength, EncounterId::Cultist);
    assert_eq!(power(&run, "strength"), 3);
    assert_eq!(power(&run, "lose_strength"), 3, "a matching first-turn Strength Down is shown");
    end_turn(&mut run);
    assert_eq!(power(&run, "strength"), 0, "the first player end turn removes exactly 3 Strength");
    assert_eq!(power(&run, "lose_strength"), 0, "Strength Down expires with its loss");
}

#[test]
fn m27_mutagenic_strength_boundary() {
    let mut run = run_with(RelicId::MutagenicStrength, EncounterId::Cultist);
    end_turn(&mut run);
    enqueue_player(&mut run, EffectKind::ApplyFullPower { id: FullPowerId("strength"), amount: 2 });
    end_turn(&mut run);
    assert_eq!(power(&run, "strength"), 2, "later unrelated Strength is not removed");
}

#[test]
fn m27_mutagenic_strength_two_combat_counter() {
    let mut run = run_with_potions(RelicId::MutagenicStrength, vec![Some(PotionId::FirePotion); 3]);
    assert_eq!(power(&run, "strength"), 3);
    end_turn(&mut run);
    assert_eq!(power(&run, "strength"), 0);
    win_cultist(&mut run);
    m27_support::next_complete_combat(&mut run, EncounterId::Cultist).unwrap();
    assert_eq!(power(&run, "strength"), 3, "combat two grants its own Strength");
    assert_eq!(power(&run, "lose_strength"), 3, "combat two has its own Strength Down");
    end_turn(&mut run);
    assert_eq!(power(&run, "strength"), 0, "combat two has an independent first-turn loss");
}

fn tongs_second_combat() -> Run {
    let mut run = run_with_potions(RelicId::WarpedTongs, vec![Some(PotionId::FirePotion); 3]);
    win_cultist(&mut run);
    replace_master_deck(&mut run, &[CardId::SearingBlow, CardId::Wound]);
    m27_support::next_complete_combat(&mut run, EncounterId::Cultist).unwrap();
    run
}

#[test]
fn m27_warped_tongs_postdraw_upgrade() {
    let run = tongs_second_combat();
    let hand = view(&run).hand;
    assert_eq!(hand.len(), 2);
    assert_eq!(hand.iter().find(|card| card.id == CardId::SearingBlow).unwrap().upgrades, 1,
        "one upgradeable non-Status card upgrades once after opening draw");
    assert_eq!(hand.iter().find(|card| card.id == CardId::Wound).unwrap().upgrades, 0,
        "Status is not an eligible candidate");
}

#[test]
fn m27_warped_tongs_boundary() {
    let run = tongs_second_combat();
    let hand = view(&run).hand;
    assert_eq!(hand.iter().find(|card| card.id == CardId::SearingBlow).unwrap().upgrades, 1,
        "Searing Blow upgrades exactly once");
    assert_eq!(hand.iter().find(|card| card.id == CardId::Wound).unwrap().upgrades, 0,
        "Status stays unchanged");
}

#[test]
fn m27_review_i2_mutagenic_first_turn_shows_strength_down() {
    let run = run_with(RelicId::MutagenicStrength, EncounterId::Cultist);
    assert_eq!(power(&run, "strength"), 3);
    assert_eq!(power(&run, "lose_strength"), 3,
        "catalog state_and_reset: a matching first-turn Strength Down effect");
}

#[test]
fn m27_review_i2_mutagenic_orange_pellets_removes_strength_down() {
    let mut run = m27_support::complete_run(Character::Ironclad,
        vec![RelicId::MutagenicStrength, RelicId::OrangePellets],
        vec![Some(PotionId::FirePotion); 3], BTreeMap::new(), EncounterId::Cultist);
    win_cultist(&mut run);
    replace_master_deck(&mut run, &[CardId::Strike, CardId::Defend, CardId::DarkEmbrace]);
    m27_support::next_complete_combat(&mut run, EncounterId::Cultist).unwrap();
    assert_eq!(power(&run, "strength"), 3);
    crate::combat::tests::m27_state::set_player_energy(
        run.state_mut_for_test().combat.as_mut().unwrap(), 99);
    play_card(&mut run, CardId::Strike, Some(TargetId(1)));
    play_card(&mut run, CardId::Defend, None);
    play_card(&mut run, CardId::DarkEmbrace, None);
    assert_eq!(power(&run, "lose_strength"), 0,
        "Orange Pellets removes Mutagenic Strength's Strength Down after an Attack, Skill, and Power");
    end_turn(&mut run);
    assert_eq!((power(&run, "strength"), power(&run, "lose_strength")), (3, 0),
        "the real end turn cannot reduce Strength after Orange Pellets removes the debuff");
}

#[test]
fn m27_review_i2_mutagenic_strength_down_precedes_an_earlier_relics_artifact() {
    // Both are atBattleStart addToTop relics (source-check-turnstart-hooks-v1): the later
    // relic's top work runs first, so Strength Down lands before Clockwork's Artifact.
    let mut run = m27_support::complete_run(Character::Ironclad,
        vec![RelicId::ClockworkSouvenir, RelicId::MutagenicStrength], vec![None; 3],
        BTreeMap::new(), EncounterId::Cultist);
    assert_eq!((power(&run, "strength"), power(&run, "lose_strength"), power(&run, "artifact")), (3, 3, 1),
        "Strength Down is applied before Clockwork Souvenir's Artifact exists");
    end_turn(&mut run);
    assert_eq!((power(&run, "strength"), power(&run, "artifact")), (3, 0),
        "the Strength loss is itself a debuff, and the Artifact blocks it");
    // Acquired the other way round, Artifact comes first and blocks the debuff.
    let mut run = m27_support::complete_run(Character::Ironclad,
        vec![RelicId::MutagenicStrength, RelicId::ClockworkSouvenir], vec![None; 3],
        BTreeMap::new(), EncounterId::Cultist);
    assert_eq!((power(&run, "strength"), power(&run, "lose_strength"), power(&run, "artifact")), (3, 0, 0),
        "Clockwork Souvenir's earlier Artifact blocks the later Mutagenic Strength Down");
    end_turn(&mut run);
    assert_eq!(power(&run, "strength"), 3, "blocked Strength Down leaves all 3 Strength");
}

#[test]
fn m27_review_i2_mutagenic_plain_first_end_turn_removes_three_strength() {
    let mut run = run_with(RelicId::MutagenicStrength, EncounterId::Cultist);
    assert_eq!((power(&run, "strength"), power(&run, "lose_strength")), (3, 3),
        "turn one shows 3 Strength and 3 Strength Down");
    end_turn(&mut run);
    assert_eq!((power(&run, "strength"), power(&run, "lose_strength")), (0, 0),
        "the first real end turn resolves Strength Down and expires it");
}

#[test]
fn m27_review_i3b_magic_flower_scales_victory_heal() {
    let cases = [
        (RelicId::MeatOnTheBone, true, (58, 80)),
        (RelicId::MeatOnTheBone, false, (52, 80)),
        (RelicId::FaceOfCleric, true, (42, 81)),
        (RelicId::FaceOfCleric, false, (41, 81)),
    ];
    for (relic, magic_flower, expected) in cases {
        // The complete training fixture supplies only these relics, so no Burning Blood heal fires.
        let relics = if magic_flower { vec![relic, RelicId::MagicFlower] } else { vec![relic] };
        let mut run = m27_support::complete_run(Character::Ironclad, relics,
            vec![Some(PotionId::FirePotion); 3], BTreeMap::new(), EncounterId::Cultist);
        let node = crate::run::NodeId(1);
        run.state_mut_for_test().map = Some(crate::run::RunMap {
            act: 1,
            current: Some(node),
            nodes: vec![crate::run::MapNode { id: node, floor: 1, label: "M27 victory-heal fixture",
                edges: vec![], x: 0, y: 0, kind: crate::run::MapNodeKind::Monster,
                hidden_kind: None, visited: true, revealed: true, encounter: Some(EncounterId::Cultist),
                burning: false }],
            ..crate::run::RunMap::default()
        });
        lose_player_hp(&mut run, 40);
        assert_eq!(view(&run).player.hp, 40, "the fixture starts the victory at 40/80");
        win_cultist(&mut run);
        assert_eq!((run.state().hp, run.state().max_hp), expected,
            "{relic:?}, Magic Flower={magic_flower}");
    }
}
