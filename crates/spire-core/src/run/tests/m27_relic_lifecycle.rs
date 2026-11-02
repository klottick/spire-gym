//! M27 plan Task M4, lane Main, key m27_relic_lifecycle.
use super::super::*;
use crate::content::{CardId, Character, EncounterId, PotionId, RelicId};
use crate::items::tests::m27_support::complete_run;
use crate::run::rooms::RoomLoop;
use crate::run::{CreationProfile, Run, RunConfig, RunError, RunRng};
use std::collections::BTreeMap;

/// The support fixture enters a complete training combat. Settle it with the same three Fire
/// Potion actions as `items/tests/m27_support.rs`, then install the requested noncombat slots.
fn settled_complete_run(character: Character, relics: Vec<RelicId>, potions: Vec<Option<PotionId>>,
    counters: BTreeMap<RelicId, i32>) -> Run {
    let initial_relics=relics.clone();
    let mut run = complete_run(character, relics, vec![Some(PotionId::FirePotion);3], counters,
        EncounterId::Cultist);
    for slot in 0..3 {
        run.choose_combat(crate::combat::CombatAction::UsePotion {
            slot,
            target: Some(crate::combat::TargetId(1)),
        }).unwrap_or_else(|error| panic!("settle complete fixture for {character:?} {initial_relics:?} with Fire Potion {slot}: {error:?}"));
    }
    assert!(run.state().combat.is_none() && run.state().active_combat.is_none());
    run.state.potions = potions;
    run
}

fn campfire_with_toke(run: &mut Run) -> ChoiceId {
    let node = NodeId(1);
    let choice = ChoiceId(100);
    run.next_choice = 101;
    run.state.act = 1;
    run.state.map = Some(RunMap {
        act: 1,
        current: Some(node),
        nodes: vec![MapNode {
            id: node, floor: 1, label: "Toke fixture", edges: vec![], x: 0, y: 0,
            kind: MapNodeKind::Rest, hidden_kind: None, visited: true, revealed: true,
            encounter: None, burning: false,
        }],
        ..RunMap::default()
    });
    run.state.room_loop = Some(RoomLoop {
        scenario: RoomScenario::M5Act1Captured, act1: None, act2: None, act3: None, act4: None,
        m10_elite_relic_cursor: 0,
        screen: RoomView { kind: RoomViewKind::Campfire, choices: vec![RoomChoiceView {
            id: choice, kind: RoomChoiceKind::Toke, price: None, status: RoomChoiceStatus::Offered,
        }] },
        purge_cost: 75, reward_parent: None, active_rewards: None, boss: None, transition: None,
        act2_boss_order: None, act3_boss_order: None, joust: None, joust_visits: 0,
        act3_event: None, act3_end: None, standard_event: None,
    });
    choice
}

fn shop_with_relic(run: &mut Run, relic: RelicId) -> ChoiceId {
    let choice = ChoiceId(200);
    run.next_choice = 201;
    run.state.gold = 200;
    run.state.room_loop = Some(RoomLoop {
        scenario: RoomScenario::M5Act1Captured, act1: None, act2: None, act3: None, act4: None,
        m10_elite_relic_cursor: 0,
        screen: RoomView { kind: RoomViewKind::Shop, choices: vec![RoomChoiceView {
            id: choice, kind: RoomChoiceKind::Relic(relic), price: Some(150),
            status: RoomChoiceStatus::Offered,
        }] },
        purge_cost: 75, reward_parent: None, active_rewards: None, boss: None, transition: None,
        act2_boss_order: None, act3_boss_order: None, joust: None, joust_visits: 0,
        act3_event: None, act3_end: None, standard_event: None,
    });
    choice
}

fn boss_choice_with_cage(run:&mut Run)->ChoiceId{
    campfire_with_toke(run);
    let choice=ChoiceId(100);
    run.next_choice=101;
    run.state.floor=17;
    let map=run.state.map.as_mut().unwrap();
    map.act=1;
    map.current=Some(NodeId(1));
    map.nodes[0].floor=17;
    map.nodes[0].kind=MapNodeKind::Boss;
    map.nodes[0].visited=true;
    run.state.room_loop.as_mut().unwrap().screen=RoomView{kind:RoomViewKind::BossRelicChoice,choices:vec![RoomChoiceView{
        id:choice,kind:RoomChoiceKind::Relic(RelicId::EmptyCage),price:None,status:RoomChoiceStatus::Offered,
    }]};
    run.m27_test_set_boss_progress(None,false);
    choice
}

fn obtain(run:&mut Run,id:RelicId,source:crate::items::lifecycle::RelicSource)->Result<crate::items::RelicObtained,RunError>{
    crate::items::obtain_relic(&mut run.state,id,source)
}

#[test]
fn m27_campfire_toke_removes_one_card() {
    let mut run = settled_complete_run(Character::Ironclad, vec![], vec![None; 3], BTreeMap::new());
    let choice = campfire_with_toke(&mut run);
    let card = run.state.deck[0].clone();
    crate::items::dispatch::take_run_event_log();
    run.step(&GameAction::Choose(choice)).expect("Toke opens its one-card removal selection");
    let pending = run.state.pending.as_ref().expect("Toke selection is pending");
    assert_eq!(pending.selection.required, 1);
    assert_eq!(pending.selection.candidates.len(), run.state.deck.len());
    let selected = pending.selection.candidates.iter().find(|candidate| candidate.card.instance == card.instance).unwrap().id;
    run.step(&GameAction::Select(selected)).expect("Toke removes the selected card");
    assert!(crate::items::dispatch::take_run_event_log().contains(&"CardRemoved"),"Toke uses the shared removal event");
    assert!(!run.state.deck.iter().any(|candidate| candidate.instance == card.instance));
    assert_eq!(run.state.last_selection, Some(vec![card]));
    assert_eq!(run.state.last_resumed_parent, Some(choice));
    assert!(run.state.pending.is_none());
    assert_eq!(run.state.room_loop.as_ref().unwrap().screen.kind, RoomViewKind::Map);
}

#[test]
fn m27_item_mutation_rejections_are_atomic() {
    let mut owned = settled_complete_run(Character::Ironclad, vec![], vec![None; 3], BTreeMap::new());
    obtain(&mut owned,RelicId::Strawberry,crate::items::lifecycle::RelicSource::Reward).unwrap();
    let before = format!("{:?}", owned.state);
    assert_eq!(obtain(&mut owned,RelicId::Strawberry,crate::items::lifecycle::RelicSource::Reward), Err(RunError::InvalidAction));
    assert_eq!(format!("{:?}", owned.state), before, "owned relic rejection leaves complete RunState unchanged");

    let mut other_class = settled_complete_run(Character::Ironclad, vec![], vec![None; 3], BTreeMap::new());
    let before = format!("{:?}", other_class.state);
    assert_eq!(obtain(&mut other_class,RelicId::RingOfTheSerpent,crate::items::lifecycle::RelicSource::Reward), Err(RunError::InvalidAction));
    assert_eq!(format!("{:?}", other_class.state), before);

    let mut active = complete_run(Character::Ironclad, vec![], vec![Some(PotionId::FirePotion); 3],
        BTreeMap::new(), EncounterId::Cultist);
    let before = format!("{:?}", active.state);
    assert_eq!(crate::items::obtain_relic(&mut active.state,RelicId::Strawberry,crate::items::lifecycle::RelicSource::Reward), Err(RunError::InvalidAction));
    assert_eq!(format!("{:?}", active.state), before);

    let mut active_only = complete_run(Character::Ironclad, vec![], vec![Some(PotionId::FirePotion); 3],
        BTreeMap::new(), EncounterId::Cultist);
    let active_combat = active_only.state.active_combat.clone().unwrap();
    active_only.state.combat = None;
    active_only.state.active_combat = Some(active_combat);
    let before = format!("{:?}", active_only.state);
    assert_eq!(crate::items::obtain_relic(&mut active_only.state,RelicId::Strawberry,crate::items::lifecycle::RelicSource::Reward), Err(RunError::InvalidAction));
    assert_eq!(format!("{:?}", active_only.state), before);

    let mut replacement = settled_complete_run(Character::Ironclad, vec![], vec![None; 3], BTreeMap::new());
    let before = format!("{:?}", replacement.state);
    assert_eq!(crate::items::obtain_relic(&mut replacement.state,RelicId::BlackBlood,crate::items::lifecycle::RelicSource::Boss), Err(RunError::InvalidAction));
    assert_eq!(format!("{:?}", replacement.state), before);

    assert_eq!(crate::items::lose_relic(&mut replacement.state,RelicId::Strawberry),Err(RunError::InvalidAction),"losing an unowned relic is rejected");
    assert_eq!(format!("{:?}",replacement.state),before,"unowned loss leaves complete RunState unchanged");

    let mut circlet=settled_complete_run(Character::Ironclad,vec![],vec![None;3],BTreeMap::new());
    obtain(&mut circlet,RelicId::Circlet,crate::items::lifecycle::RelicSource::Reward).unwrap();
    assert_eq!(circlet.state.relics.iter().filter(|id|**id==RelicId::Circlet).count(),1);
    assert_eq!(circlet.state.relic_counters.get(&RelicId::Circlet),Some(&1));
    obtain(&mut circlet,RelicId::Circlet,crate::items::lifecycle::RelicSource::Reward).unwrap();
    assert_eq!(circlet.state.relics.iter().filter(|id|**id==RelicId::Circlet).count(),1,"Circlet duplicates stack without appending");
    assert_eq!(circlet.state.relic_counters.get(&RelicId::Circlet),Some(&2));
}

#[test]
fn m27_obtain_equip_resource_laws() {
    for (id, hp, max_hp) in [
        (RelicId::Strawberry, 57, 87),
        (RelicId::Pear, 60, 90),
        (RelicId::Mango, 64, 94),
    ] {
        let mut run = settled_complete_run(Character::Ironclad, vec![], vec![None; 3], BTreeMap::new());
        run.state.hp = 50;
        run.state.max_hp = 80;
        obtain(&mut run,id,crate::items::lifecycle::RelicSource::Reward).unwrap();
        assert_eq!((run.state.hp, run.state.max_hp), (hp, max_hp), "{id:?}");
    }

    let mut bloom = settled_complete_run(Character::Ironclad, vec![RelicId::MarkOfBloom],
        vec![None; 3], BTreeMap::new());
    bloom.state.hp = 50;
    bloom.state.max_hp = 80;
    obtain(&mut bloom,RelicId::Strawberry,crate::items::lifecycle::RelicSource::Reward).unwrap();
    assert_eq!((bloom.state.hp, bloom.state.max_hp), (50, 87));

    let original = [Some(PotionId::FirePotion), Some(PotionId::BlockPotion), Some(PotionId::WeakPotion)];
    let mut belt = settled_complete_run(Character::Ironclad, vec![], original.to_vec(), BTreeMap::new());
    let choice = shop_with_relic(&mut belt, RelicId::PotionBelt);
    belt.step(&GameAction::Choose(choice)).expect("shop obtain equips Potion Belt");
    assert_eq!(belt.state.potions, [original.as_slice(), &[None, None]].concat(),
        "Potion Belt appends two empty slots and preserves existing slot order");

    let mut defect = settled_complete_run(Character::Defect, vec![], vec![None; 3], BTreeMap::new());
    obtain(&mut defect,RelicId::NeowsLament,crate::items::lifecycle::RelicSource::Neow).unwrap();
    obtain(&mut defect,RelicId::Inserter,crate::items::lifecycle::RelicSource::Reward).unwrap();
    assert_eq!(defect.state.relic_counters.get(&RelicId::NeowsLament), Some(&3));
    assert_eq!(defect.state.relic_counters.get(&RelicId::Inserter), Some(&0));
}

#[test]
fn m27_replacement_and_loss() {
    for (character,starter,boss) in [
        (Character::Ironclad,RelicId::BurningBlood,RelicId::BlackBlood),
        (Character::Silent,RelicId::RingOfTheSnake,RelicId::RingOfTheSerpent),
        (Character::Defect,RelicId::CrackedCore,RelicId::FrozenCore),
        (Character::Watcher,RelicId::PureWater,RelicId::HolyWater),
    ] {
        let mut run=settled_complete_run(Character::Ironclad,vec![],vec![None;3],BTreeMap::new());
        run.state.character=character;
        run.state.relics.push(starter);
        run.state.relic_counters.insert(starter,1);
        obtain(&mut run,boss,crate::items::lifecycle::RelicSource::Boss).expect("boss replacement obtains in starter slot");
        assert_eq!(run.state.relics.first(),Some(&boss),"{starter:?} -> {boss:?} replaces slot zero");
        assert!(!run.state.relics.contains(&starter));
        assert!(!run.state.relic_counters.contains_key(&starter),"{starter:?} counter is removed");
    }

    let potions=[Some(PotionId::FirePotion),Some(PotionId::BlockPotion),Some(PotionId::WeakPotion),None,None];
    let mut run=settled_complete_run(Character::Silent,
        vec![RelicId::BottledFlame,RelicId::Omamori],potions[..3].to_vec(),
        BTreeMap::from([(RelicId::Omamori,1)]));
    run.state.relics.push(RelicId::PotionBelt);
    run.state.potions=potions.to_vec();
    run.state.relic_cards.insert(RelicId::BottledFlame,run.state.deck[0].instance);
    crate::items::lose_relic(&mut run.state,RelicId::BottledFlame).unwrap();
    assert!(!run.state.relic_cards.contains_key(&RelicId::BottledFlame),"losing a bottle clears its card link");
    crate::items::lose_relic(&mut run.state,RelicId::Omamori).unwrap();
    assert!(!run.state.relic_counters.contains_key(&RelicId::Omamori),"loss clears all persistent counters for the relic");
    let before=run.state.potions.clone();
    // F6: there is no source unequip rule; Potion Belt's catalog law permanently adds two slots.
    crate::items::lose_relic(&mut run.state,RelicId::PotionBelt).unwrap();
    assert_eq!(run.state.potions,before,"F6 keeps Potion Belt's capacity and potion order after loss");
    assert_eq!(run.state.potions.len(),5);
}

#[test]
fn m27_selection_continuation_framework() {
    let mut run=settled_complete_run(Character::Ironclad,vec![],vec![None;3],BTreeMap::new());
    let choice=boss_choice_with_cage(&mut run);
    run.step(&GameAction::Choose(choice)).expect("Complete boss Cage obtain opens shared selector");
    let pending=run.state.pending.as_ref().unwrap();
    assert_eq!(pending.selection.required,2);
    assert_eq!(pending.selection.candidates.len(),run.state.deck.len());
    assert_eq!(pending.selection.empty_rule,crate::run::EmptySelectionRule::Fault);
    let first=run.state.pending.as_ref().unwrap().selection.candidates[0].id;
    run.state.pending.as_mut().unwrap().selection.required=1;
    let malformed=format!("{:?}",run.state);
    assert_eq!(run.step(&GameAction::Select(first)),Err(RunError::InvalidAction));
    assert_eq!(format!("{:?}",run.state),malformed,"wrong-size selection leaves complete RunState unchanged");
    run.state.pending.as_mut().unwrap().selection.required=2;
    let before_cards:Vec<_>=run.state.deck.clone();
    let first_card=run.state.pending.as_ref().unwrap().selection.candidates[0].card.clone();
    run.step(&GameAction::Select(first)).unwrap();
    let second=run.state.pending.as_ref().unwrap().selection.candidates[0].id;
    let second_card=run.state.pending.as_ref().unwrap().selection.candidates[0].card.clone();
    run.step(&GameAction::Select(second)).unwrap();
    assert_eq!(run.state.deck.len(),before_cards.len()-2);
    assert!(!run.state.deck.contains(&first_card)&&!run.state.deck.contains(&second_card));
    assert_eq!(run.state.last_selection,Some(vec![first_card,second_card]));
    assert_eq!(run.state.last_resumed_parent,Some(choice));
    assert!(run.state.pending.is_none());
    assert_eq!(run.state.room_loop.as_ref().unwrap().screen.kind,RoomViewKind::BossRelicChild);

    let mut empty=settled_complete_run(Character::Ironclad,vec![],vec![None;3],BTreeMap::new());
    empty.state.deck=vec![CardId::CurseOfTheBell,CardId::Necronomicurse,CardId::AscendersBane].into_iter().enumerate()
        .map(|(index,id)|crate::run::CardInstance{instance:index as u64+1,id,upgrades:0,misc:0}).collect();
    let empty_choice=boss_choice_with_cage(&mut empty);
    empty.step(&GameAction::Choose(empty_choice)).unwrap();
    assert!(empty.state.pending.is_none(),"§3.1 empty group opens no selector");
    assert_eq!(empty.state.deck.len(),3);
    assert_eq!(empty.state.last_selection,Some(vec![]));

    let mut small=settled_complete_run(Character::Ironclad,vec![],vec![None;3],BTreeMap::new());
    small.state.deck.truncate(2);
    let small_choice=boss_choice_with_cage(&mut small);
    let transition=small.step(&GameAction::Choose(small_choice)).unwrap();
    assert_eq!(transition.terminal,Some(crate::run::RunTerminal::EmptyDeck),"§3.1 removes all eligible cards when eligible <= count");
    assert!(small.state.pending.is_none());
    assert!(small.state.deck.is_empty());
}

#[test]
fn m27_complete_counter_view() {
    let counters=BTreeMap::from([(RelicId::PenNib,4),(RelicId::Omamori,1),(RelicId::NeowsLament,2)]);
    assert_eq!(crate::items::complete_counter_view(&counters),vec![
        crate::items::RelicCounterView{relic:RelicId::NeowsLament,value:2},
        crate::items::RelicCounterView{relic:RelicId::Omamori,value:1},
        crate::items::RelicCounterView{relic:RelicId::PenNib,value:4},
    ]);
    assert_eq!(crate::items::visible_counters(&counters),vec![crate::items::RelicCounterView{relic:RelicId::NeowsLament,value:2}],
        "the legacy visible view remains only Neow's Lament and Inserter");
    let complete=BTreeMap::from([(RelicId::Circlet,2),(RelicId::NlothsHungryFace,1)]);
    assert_eq!(crate::items::complete_counter_view(&complete),vec![
        crate::items::RelicCounterView{relic:RelicId::Circlet,value:2},
        crate::items::RelicCounterView{relic:RelicId::NlothsHungryFace,value:1},
    ],"complete counters follow catalog order");
    let catalog=[
        (RelicId::NeowsLament,2),(RelicId::Inserter,0),(RelicId::AncientTeaSet,1),(RelicId::HappyFlower,0),
        (RelicId::MawBank,1),(RelicId::Nunchaku,5),(RelicId::Omamori,1),(RelicId::PenNib,4),
        (RelicId::TinyChest,2),(RelicId::InkBottle,3),(RelicId::Matryoshka,1),(RelicId::Sundial,2),
        (RelicId::Girya,1),(RelicId::IncenseBurner,4),(RelicId::LizardTail,1),(RelicId::WingBoots,2),
        (RelicId::Circlet,3),(RelicId::NlothsHungryFace,1),
    ];
    let all: BTreeMap<_,_>=catalog.into_iter().collect();
    assert_eq!(crate::items::complete_counter_view(&all),catalog.into_iter()
        .map(|(relic,value)|crate::items::RelicCounterView{relic,value}).collect::<Vec<_>>());
}

#[test]
fn m27_obtain_relic_profile_is_acquire() {
    let mut run=Run::new(RunConfig{character:Character::Ironclad,ascension:0},CreationProfile::experienced(),
        RunRng::new([9;32],crate::combat::rng::Partition::Test,0)).unwrap();
    let mut expected=run.state.clone();
    crate::items::acquire_relic(&mut expected,RelicId::Strawberry).unwrap();
    obtain(&mut run,RelicId::Strawberry,crate::items::lifecycle::RelicSource::Reward).unwrap();
    assert_eq!(format!("{:?}",run.state),format!("{:?}",expected),"Profile obtain delegates to acquire exactly");
}

#[test]
fn m27_review_i3a_lees_waffle_max_hp_with_mark_of_bloom() {
    let source = crate::items::lifecycle::RelicSource::Reward;
    let mut bloom = settled_complete_run(Character::Ironclad, vec![RelicId::MarkOfBloom],
        vec![None; 3], BTreeMap::new());
    bloom.state.hp = 12;
    bloom.state.max_hp = 80;
    obtain(&mut bloom, RelicId::LeesWaffle, source).unwrap();
    assert_eq!((bloom.state.hp, bloom.state.max_hp), (12, 87),
        "Mark of the Bloom prevents Lee's Waffle healing but not its 7 Max HP gain");

    let mut control = settled_complete_run(Character::Ironclad, vec![], vec![None; 3], BTreeMap::new());
    control.state.hp = 12;
    control.state.max_hp = 80;
    obtain(&mut control, RelicId::LeesWaffle, source).unwrap();
    assert_eq!((control.state.hp, control.state.max_hp), (87, 87),
        "without Mark of the Bloom Lee's Waffle heals to the new maximum");
}

#[test]
fn m27_review_m2_lost_potion_belt_keeps_five_slot_capacity() {
    let mut run = settled_complete_run(Character::Ironclad, vec![],
        vec![None; 3], BTreeMap::new());
    obtain(&mut run, RelicId::PotionBelt, crate::items::lifecycle::RelicSource::Reward).unwrap();
    assert_eq!(run.state.potions.len(), 5);
    assert_eq!(crate::items::lose_relic(&mut run.state, RelicId::PotionBelt), Ok(()),
        "losing Potion Belt keeps the complete Run's five potion slots valid");
    assert!(!run.state.relics.contains(&RelicId::PotionBelt));
    assert_eq!(run.state.potions.len(), 5, "losing Potion Belt preserves its two added slots");
    assert_eq!(crate::items::obtain_run_potion(&mut run.state, PotionId::FirePotion), Ok(true),
        "an outside-combat obtain fills an empty slot after Potion Belt is lost");
    assert_eq!(run.state.potions[0], Some(PotionId::FirePotion));
    run.state.room_loop = None;
    run.state.map = None;
    crate::items::tests::m27_support::next_complete_combat(&mut run, EncounterId::Cultist)
        .expect("complete combat entry accepts the five-slot belt after loss");
    assert_eq!(run.state.combat.as_ref().unwrap().item_context().unwrap().potions()[0],
        Some(PotionId::FirePotion));
}

#[test]
fn m27_review_m3_empty_boss_pool_red_circlet_is_obtainable() {
    let mut run = settled_complete_run(Character::Ironclad, vec![], vec![None; 3], BTreeMap::new());
    run.state.relic_pools = Some(crate::items::relic_pools::explicit_for_test(
        vec![], vec![], vec![], vec![], vec![]));
    let relic = crate::items::relic_pools::take(&mut run.state, crate::content::RelicTier::Boss,
        crate::items::relic_pools::Selector::Front, crate::items::lifecycle::RelicSource::Boss, &[1])
        .unwrap();
    assert_eq!(relic, RelicId::RedCirclet,
        "the empty Boss pool falls back to Red Circlet (F7)");
    assert_eq!(obtain(&mut run, relic, crate::items::lifecycle::RelicSource::Boss).is_ok(), true,
        "items::obtain_relic accepts the exhausted-pool Red Circlet from a Boss source");
    assert!(run.state.relics.contains(&RelicId::RedCirclet),
        "the exhausted-pool Red Circlet can be obtained and owned");
}
