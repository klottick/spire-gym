//! M28f event branch tests: source literals from plan-check-v1 §§2/4/6 and source-check-events-v1.
use super::super::*;
use super::m28_event_core::{at_event, base, before_event, count, leave, row, view};
use crate::content::{
    CardColor, CardId, CardRarity, Catalog, Character, EncounterId, EnemyId, EventId, PotionId, RelicId,
    RelicTier,
};
use crate::run::{CardInstance, GameAction, RoomChoiceKind, RoomChoiceStatus, RoomViewKind, Run};

fn nloth_event(character: Character, seed: u8, gift: bool) -> Run {
    let (mut run, node) = before_event(character, 2, EventId::Nloth, seed);
    run.state.relics.extend([RelicId::Anchor, RelicId::HornCleat]);
    if gift { run.state.relics.push(RelicId::NlothsGift); }
    run.step(&GameAction::Move(node)).unwrap();
    assert_eq!(view(&run).kind, RoomViewKind::Event(EventId::Nloth));
    run
}

fn moai_event(character: Character, seed: u8) -> Run {
    let (mut run, node) = before_event(character, 3, EventId::MoaiHead, seed);
    run.state.hp = base(character) / 2;
    run.step(&GameAction::Move(node)).unwrap();
    assert_eq!(view(&run).kind, RoomViewKind::Event(EventId::MoaiHead));
    run
}

fn nloth_offered_relic(run: &Run, option: u8) -> RelicId {
    view(run).choices.into_iter().find_map(|choice| match choice.kind {
        RoomChoiceKind::EventRelic { stage: 0, option: row_option, relic } if row_option == option => relic,
        _ => None,
    }).unwrap_or_else(|| panic!("N'loth option {option} did not show an offered relic"))
}

fn choose_nloth(run: &mut Run, option: u8) {
    let id = view(run).choices.into_iter().find_map(|choice| match choice.kind {
        RoomChoiceKind::EventRelic { stage: 0, option: row_option, .. } if row_option == option => Some(choice.id),
        _ => None,
    }).unwrap_or_else(|| panic!("N'loth option {option} did not publish a relic row"));
    run.step(&GameAction::Choose(id)).unwrap();
}

fn choose(run: &mut Run, stage: u8, option: u8) -> crate::run::RunTransition {
    let id = row(run, &RoomChoiceKind::EventOption { stage, option }).id;
    run.step(&GameAction::Choose(id)).unwrap()
}

fn select_instance(run: &mut Run, instance: u64) {
    let id = run.state.pending.as_ref().expect("event grid is open").selection.candidates
        .iter().find(|candidate| candidate.card.instance == instance)
        .unwrap_or_else(|| panic!("instance {instance} is not offered")).id;
    run.step(&GameAction::Select(id)).unwrap();
}

fn set_deck(run: &mut Run, ids: &[CardId]) {
    let first = run.state.deck.first().map_or(90_000, |card| card.instance + 100);
    run.state.deck = ids.iter().enumerate().map(|(index, id)| CardInstance {
        id: *id, instance: first + index as u64, upgrades: 0, misc: 0,
    }).collect();
    run.state.relic_cards.clear();
}

fn install_bottle(run: &mut Run, relic: RelicId, instance: u64) {
    run.state.relics.push(relic);
    run.state.relic_cards.insert(relic, instance);
}

fn assert_done(run: &Run) {
    assert_eq!(view(run).choices.iter().map(|choice| choice.kind.clone()).collect::<Vec<_>>(),
        vec![RoomChoiceKind::Leave]);
}

fn finish_done(run: &mut Run) {
    assert_done(run);
    leave(run);
    assert_eq!(view(run).kind, RoomViewKind::Map);
}

fn designer_with_option(option: u8, gold: u32, deck: &[CardId]) -> Run {
    for seed in 0..=u8::MAX {
        let (mut run, node) = before_event(Character::Ironclad, 2, EventId::Designer, seed);
        run.state.gold = gold;
        set_deck(&mut run, deck);
        run.step(&GameAction::Move(node)).unwrap();
        if view(&run).choices.iter().any(|choice|
            choice.kind == RoomChoiceKind::EventOption { stage: 0, option })
        {
            return run;
        }
    }
    panic!("no Designer construction exposed option {option}");
}

fn designer_with_gold(gold: u32) -> Run {
    let (mut run, node) = before_event(Character::Ironclad, 2, EventId::Designer, 37);
    run.state.gold = 500;
    run.step(&GameAction::Move(node)).unwrap();
    run.state.gold = gold;
    crate::run::rooms::standard::event_frame::publish(&mut run).unwrap();
    run
}

fn designer_variant_row(run: &Run, options: &[u8]) -> crate::run::RoomChoiceView {
    view(run).choices.into_iter().find(|choice| matches!(choice.kind,
        RoomChoiceKind::EventOption { stage: 0, option } if options.contains(&option)))
        .unwrap_or_else(|| panic!("no Designer row for {options:?}: {:?}", view(run).choices))
}

#[test]
fn m28_moai_head_jump() {
    for (character, max_hp, loss) in [
        (Character::Ironclad, 80, 10),
        (Character::Silent, 70, 9),
        (Character::Defect, 75, 9),
        (Character::Watcher, 72, 9),
    ] {
        let (mut run, node) = before_event(character, 3, EventId::MoaiHead, 10);
        run.state.max_hp = max_hp;
        run.state.hp = max_hp / 2;
        run.step(&GameAction::Move(node)).unwrap();
        choose(&mut run, 0, 0);
        assert_eq!((run.state.max_hp, run.state.hp), (max_hp - loss, max_hp - loss), "{character:?}");
        finish_done(&mut run);
    }
}

#[test]
fn m28_moai_head_trade_idol() {
    let run = moai_event(Character::Defect, 11);
    let idol = row(&run, &RoomChoiceKind::EventOption { stage: 0, option: 1 });
    assert_eq!(idol.status, RoomChoiceStatus::Disabled, "Golden Idol is required");
    let (mut run, node) = before_event(Character::Defect, 3, EventId::MoaiHead, 111);
    run.state.relics.push(RelicId::GoldenIdol);
    run.step(&GameAction::Move(node)).unwrap();
    assert_eq!(view(&run).kind, RoomViewKind::Event(EventId::MoaiHead));
    let before = run.state.gold;
    choose(&mut run, 0, 1);
    assert_eq!(run.state.gold, before + 333);
    assert!(!run.state.relics.contains(&RelicId::GoldenIdol));
    finish_done(&mut run);
}

#[test]
fn m28_moai_head_leave() {
    let mut run = moai_event(Character::Silent, 12);
    let before = (run.state.hp, run.state.max_hp, run.state.gold, run.state.relics.clone());
    choose(&mut run, 0, 2);
    assert_eq!(view(&run).kind, RoomViewKind::Map);
    assert_eq!((run.state.hp, run.state.max_hp, run.state.gold, run.state.relics), before);
}

#[test]
fn m28_mysterious_sphere_fight() {
    let mut run = at_event(Character::Ironclad, 3, EventId::MysteriousSphere, 13);
    choose(&mut run, 0, 0);
    let combat = run.state.combat.as_ref().expect("Fight enters combat");
    assert_eq!(run.state.active_combat.as_ref().unwrap().encounter, EncounterId::TwoOrbWalkers);
    let enemies: Vec<_> = combat.public_full().unwrap().enemies.iter().map(|enemy| enemy.enemy_id).collect();
    assert_eq!(enemies, vec![EnemyId::OrbWalker, EnemyId::OrbWalker], "left slot then right slot");
    super::m28_world::win(&mut run);
    let rows = view(&run);
    assert_eq!(rows.kind, RoomViewKind::Rewards);
    let gold: Vec<_> = rows.choices.iter().filter_map(|choice| match choice.kind {
        RoomChoiceKind::Gold(amount) => Some(amount), _ => None,
    }).collect();
    assert_eq!(gold.len(), 1);
    assert!((45..=55).contains(&gold[0]), "one source-range gold row");
    let relics: Vec<_> = rows.choices.iter().filter_map(|choice| match choice.kind {
        RoomChoiceKind::Relic(relic) => Some(relic), _ => None,
    }).collect();
    assert_eq!(relics.len(), 1);
    assert_eq!(Catalog::standard().relic_tier(relics[0]), Some(RelicTier::Rare));
    assert!(![RelicId::BottledFlame, RelicId::BottledLightning, RelicId::BottledTornado, RelicId::Whetstone].contains(&relics[0]));
    assert_eq!(rows.choices.iter().filter(|choice| choice.kind == RoomChoiceKind::CardReward).count(), 1);
    assert!(rows.choices.iter().any(|choice| choice.kind == RoomChoiceKind::Proceed));
}

#[test]
fn m28_mysterious_sphere_leave() {
    let mut run = at_event(Character::Watcher, 3, EventId::MysteriousSphere, 14);
    let before = run.state.clone();
    let rng = run.state.rng.clone();
    choose(&mut run, 0, 1);
    assert_eq!(view(&run).kind, RoomViewKind::Map);
    assert_eq!((run.state.hp, run.state.max_hp, run.state.gold, run.state.deck, run.state.relics),
        (before.hp, before.max_hp, before.gold, before.deck, before.relics));
    assert_eq!(run.state.rng, rng, "Leave takes no event choice draw");
}

fn sensory_stone_rows(option: u8, count_rows: usize, hp_loss: u32, seed: u8) {
    let (mut run, node) = before_event(Character::Ironclad, 3, EventId::SensoryStone, seed);
    run.state.max_hp = 80;
    run.state.hp = 70;
    run.step(&GameAction::Move(node)).unwrap();
    choose(&mut run, 0, option);
    assert_eq!(run.state.hp, 70 - hp_loss, "option {option}");
    let rows = view(&run);
    assert_eq!(rows.kind, RoomViewKind::Rewards);
    let cards: Vec<_> = rows.choices.iter().filter(|choice| choice.kind == RoomChoiceKind::CardReward)
        .map(|choice| choice.id).collect();
    assert_eq!(cards.len(), count_rows, "only the event's colorless rows");
    assert!(!rows.choices.iter().any(|choice| matches!(choice.kind,
        RoomChoiceKind::Card { .. } | RoomChoiceKind::Relic(_) | RoomChoiceKind::Gold(_))));
    for id in cards {
        run.step(&GameAction::Choose(id)).unwrap();
        let offers = view(&run);
        let offered_cards: Vec<_> = offers.choices.iter().filter_map(|choice| match choice.kind {
            RoomChoiceKind::Card { id, .. } => Some(id), _ => None,
        }).collect();
        assert_eq!(offered_cards.len(), 3, "default Colorless row has three cards");
        for card in offered_cards {
            let info = Catalog::standard().card(card).unwrap();
            assert_eq!(info.color, CardColor::Colorless);
            assert!(matches!(info.rarity, CardRarity::Uncommon | CardRarity::Rare));
        }
        let skip = row(&run, &RoomChoiceKind::Skip).id;
        run.step(&GameAction::Choose(skip)).unwrap();
        assert_eq!(view(&run).kind, RoomViewKind::Rewards);
    }
    let proceed = row(&run, &RoomChoiceKind::Proceed).id;
    run.step(&GameAction::Choose(proceed)).unwrap();
    assert_eq!(view(&run).kind, RoomViewKind::Map);
}

#[test]
fn m28_sensory_stone_one() { sensory_stone_rows(0, 1, 0, 15); }

#[test]
fn m28_sensory_stone_two() { sensory_stone_rows(1, 2, 5, 16); }

#[test]
fn m28_sensory_stone_three() { sensory_stone_rows(2, 3, 10, 17); }

#[test]
fn m28_winding_halls_madness() {
    for (character, max_hp, damage) in [
        (Character::Ironclad, 80, 10),
        (Character::Silent, 70, 9),
        (Character::Defect, 75, 9),
        (Character::Watcher, 72, 9),
    ] {
        let (mut run, node) = before_event(character, 3, EventId::WindingHalls, 18);
        run.state.max_hp = max_hp;
        run.state.hp = max_hp;
        run.step(&GameAction::Move(node)).unwrap();
        let old = count(&run, CardId::Madness);
        choose(&mut run, 0, 0);
        assert_eq!(run.state.hp, max_hp - damage, "{character:?}");
        assert_eq!(count(&run, CardId::Madness), old + 2, "{character:?}");
        finish_done(&mut run);
    }
}

#[test]
fn m28_winding_halls_rest() {
    for (character, heal) in [
        (Character::Ironclad, 20),
        (Character::Silent, 18),
        (Character::Defect, 19),
        (Character::Watcher, 18),
    ] {
        let (mut run, node) = before_event(character, 3, EventId::WindingHalls, 19);
        run.state.max_hp = base(character);
        run.state.hp = 1;
        run.step(&GameAction::Move(node)).unwrap();
        let old = count(&run, CardId::Writhe);
        choose(&mut run, 0, 1);
        assert_eq!(run.state.hp, 1 + heal, "{character:?}");
        assert_eq!(count(&run, CardId::Writhe), old + 1, "{character:?}");
        finish_done(&mut run);
    }

    let mut blocked = at_event(Character::Ironclad, 3, EventId::WindingHalls, 20);
    blocked.state.hp = 1;
    blocked.state.relics.push(RelicId::Omamori);
    blocked.state.relic_counters.insert(RelicId::Omamori, 2);
    let old = count(&blocked, CardId::Writhe);
    choose(&mut blocked, 0, 1);
    assert_eq!((count(&blocked, CardId::Writhe), blocked.state.relic_counters.get(&RelicId::Omamori)),
        (old, Some(&1)), "Omamori blocks the curse and spends one charge");
    finish_done(&mut blocked);
}

#[test]
fn m28_winding_halls_maze() {
    for character in [Character::Ironclad, Character::Silent, Character::Defect, Character::Watcher] {
        let max_hp = base(character);
        let (mut run, node) = before_event(character, 3, EventId::WindingHalls, 21);
        run.state.max_hp = max_hp;
        run.state.hp = max_hp;
        run.step(&GameAction::Move(node)).unwrap();
        choose(&mut run, 0, 2);
        assert_eq!((run.state.max_hp, run.state.hp), (max_hp - 4, max_hp - 4), "{character:?}");
        finish_done(&mut run);
    }
}

#[test]
fn m28_secret_portal_boss() {
    let mut run = at_event(Character::Defect, 3, EventId::SecretPortal, 22);
    let previous_floor = run.state.floor;
    choose(&mut run, 0, 0);
    // The event room closes to the map view, as before any map-entered fight; the boss combat runs.
    assert_eq!(view(&run).kind, RoomViewKind::Map);
    assert_eq!(run.state.floor, previous_floor + 1);
    let map = run.state.map.as_ref().unwrap();
    let current = map.current.unwrap();
    assert_eq!(map.nodes.iter().find(|node| node.id == current).unwrap().kind, MapNodeKind::Boss);
    assert!(run.state.combat.is_some());
    assert!(run.state.fault.is_none());
}

#[test]
fn m28_secret_portal_leave() {
    let mut run = at_event(Character::Ironclad, 3, EventId::SecretPortal, 23);
    let before = (run.state.hp, run.state.max_hp, run.state.gold, run.state.deck.clone(), run.state.relics.clone());
    let rng = run.state.rng.clone();
    choose(&mut run, 0, 1);
    assert_eq!(view(&run).kind, RoomViewKind::Map);
    assert_eq!((run.state.hp, run.state.max_hp, run.state.gold, run.state.deck, run.state.relics), before);
    assert_eq!(run.state.rng, rng);
}

fn designer_with_bottle(option: u8, gold: u32, deck: &[CardId], relic: RelicId, bottle_index: usize) -> Run {
    for seed in 0..=u8::MAX {
        let (mut run, node) = before_event(Character::Ironclad, 2, EventId::Designer, seed);
        run.state.gold = gold;
        set_deck(&mut run, deck);
        let instance = run.state.deck[bottle_index].instance;
        install_bottle(&mut run, relic, instance);
        run.step(&GameAction::Move(node)).unwrap();
        if view(&run).choices.iter().any(|choice|
            choice.kind == RoomChoiceKind::EventOption { stage: 0, option })
        {
            return run;
        }
    }
    panic!("no bottled Designer construction exposed option {option}");
}

#[test]
fn m28_designer_adjust_chosen() {
    let mut run = designer_with_option(0, 500, &[CardId::Bash, CardId::Defend]);
    let chosen = run.state.deck[0].instance;
    assert_eq!(designer_variant_row(&run, &[0, 1]).price, Some(40));
    choose(&mut run, 0, 0);
    select_instance(&mut run, chosen);
    assert_eq!(run.state.deck.iter().find(|card| card.instance == chosen).unwrap().upgrades, 1);
    assert_eq!(run.state.gold, 460);
    finish_done(&mut run);
}

#[test]
fn m28_designer_adjust_random_two() {
    let mut run = designer_with_option(1, 500, &[CardId::Bash, CardId::Defend]);
    let old_instances: Vec<_> = run.state.deck.iter().map(|card| card.instance).collect();
    assert_eq!(designer_variant_row(&run, &[0, 1]).price, Some(40));
    choose(&mut run, 0, 1);
    assert_eq!(run.state.gold, 460);
    assert_eq!(run.state.deck.iter().map(|card| card.upgrades).sum::<u32>(), 2);
    assert!(old_instances.iter().all(|instance| run.state.deck.iter().any(|card| card.instance == *instance)));
    finish_done(&mut run);
}

#[test]
fn m28_designer_clean_remove() {
    let mut run = designer_with_option(2, 500, &[CardId::Strike, CardId::Defend, CardId::Bash]);
    let offered: Vec<_> = run.state.deck.iter().map(|card| card.instance).collect();
    assert_eq!(designer_variant_row(&run, &[2, 3]).price, Some(60));
    choose(&mut run, 0, 2);
    select_instance(&mut run, offered[0]);
    assert_eq!(run.state.gold, 440);
    assert_eq!(run.state.deck.len(), 2);
    assert!(!run.state.deck.iter().any(|card| card.instance == offered[0]));
    finish_done(&mut run);
}

#[test]
fn m28_designer_clean_transform_two() {
    let mut run = designer_with_option(3, 500, &[CardId::Strike, CardId::Defend, CardId::Bash]);
    let offered: Vec<_> = run.state.deck.iter().map(|card| card.instance).collect();
    assert_eq!(designer_variant_row(&run, &[2, 3]).price, Some(60));
    choose(&mut run, 0, 3);
    select_instance(&mut run, offered[0]);
    select_instance(&mut run, offered[1]);
    assert_eq!(run.state.gold, 440);
    assert_eq!(run.state.deck.len(), 3);
    assert!(offered[..2].iter().all(|instance| !run.state.deck.iter().any(|card| card.instance == *instance)));
    finish_done(&mut run);
}

#[test]
fn m28_designer_full_service() {
    let mut run = designer_with_option(4, 500, &[CardId::Strike, CardId::Defend, CardId::Bash]);
    let removed = run.state.deck[0].instance;
    assert_eq!(row(&run, &RoomChoiceKind::EventOption { stage: 0, option: 4 }).price, Some(90));
    choose(&mut run, 0, 4);
    select_instance(&mut run, removed);
    assert_eq!(run.state.gold, 410);
    assert_eq!(run.state.deck.len(), 2);
    assert!(!run.state.deck.iter().any(|card| card.instance == removed));
    assert_eq!(run.state.deck.iter().map(|card| card.upgrades).sum::<u32>(), 2, "upgrades up to two remaining cards");
    finish_done(&mut run);
}

#[test]
fn m28_designer_punch() {
    let mut run = designer_with_option(5, 500, &[CardId::Strike, CardId::Defend]);
    // A row price is a gold price; Punch's 3 HP cost shows none.
    assert_eq!(row(&run, &RoomChoiceKind::EventOption { stage: 0, option: 5 }).price, None);
    let gold = run.state.gold;
    let hp = run.state.hp;
    choose(&mut run, 0, 5);
    assert_eq!((run.state.hp, run.state.gold), (hp - 3, gold));
    finish_done(&mut run);
}

#[test]
fn m28_designer_disabled_states() {
    let low_gold = designer_with_gold(39);
    for options in [&[0, 1][..], &[2, 3][..], &[4][..]] {
        let choice = designer_variant_row(&low_gold, options);
        assert_eq!(choice.status, RoomChoiceStatus::Disabled, "insufficient gold: {choice:?}");
    }
    assert_eq!(designer_variant_row(&low_gold, &[0, 1]).price, Some(40));
    assert_eq!(designer_variant_row(&low_gold, &[2, 3]).price, Some(60));
    assert_eq!(row(&low_gold, &RoomChoiceKind::EventOption { stage: 0, option: 4 }).price, Some(90));

    let no_upgrades = designer_with_option(0, 500, &[
        CardId::AscendersBane, CardId::CurseOfTheBell, CardId::Necronomicurse,
    ]);
    assert_eq!(designer_variant_row(&no_upgrades, &[0, 1]).status, RoomChoiceStatus::Disabled);

    let one_unbottled = designer_with_bottle(3, 500, &[CardId::Bash, CardId::Defend], RelicId::BottledFlame, 0);
    assert_eq!(row(&one_unbottled, &RoomChoiceKind::EventOption { stage: 0, option: 3 }).status, RoomChoiceStatus::Disabled,
        "Clean Up transform needs two unbottled cards");
    assert_eq!(row(&one_unbottled, &RoomChoiceKind::EventOption { stage: 0, option: 4 }).status, RoomChoiceStatus::Offered,
        "Full Service accepts one unbottled card");
    let one_unbottled_remove = designer_with_bottle(2, 500, &[CardId::Bash, CardId::Defend], RelicId::BottledFlame, 0);
    assert_eq!(row(&one_unbottled_remove, &RoomChoiceKind::EventOption { stage: 0, option: 2 }).status, RoomChoiceStatus::Offered,
        "Clean Up remove accepts one unbottled card");
    let zero_unbottled = designer_with_bottle(2, 500, &[CardId::Bash], RelicId::BottledFlame, 0);
    assert_eq!(row(&zero_unbottled, &RoomChoiceKind::EventOption { stage: 0, option: 2 }).status, RoomChoiceStatus::Disabled);
    assert_eq!(row(&zero_unbottled, &RoomChoiceKind::EventOption { stage: 0, option: 4 }).status, RoomChoiceStatus::Disabled,
        "Full Service requires an unbottled card");
}

#[test]
fn m28_designer_clean_remove_softlock() {
    let ids = [CardId::AscendersBane, CardId::CurseOfTheBell, CardId::Necronomicurse];
    let mut run = designer_with_option(2, 500, &ids);
    let deck = run.state.deck.clone();
    choose(&mut run, 0, 2);
    assert_eq!(run.state.gold, 440);
    assert_eq!(run.state.deck, deck);
    assert!(run.state.pending.is_none());
    assert_done(&run);
    finish_done(&mut run);
}

#[test]
fn m28_designer_clean_transform_softlock() {
    let mut run = designer_with_option(3, 500, &[
        CardId::Strike, CardId::AscendersBane, CardId::CurseOfTheBell, CardId::Necronomicurse,
    ]);
    let deck = run.state.deck.clone();
    choose(&mut run, 0, 3);
    assert_eq!(run.state.gold, 440);
    assert_eq!(run.state.deck, deck, "a one-card grid cannot satisfy the two-card selection");
    assert!(run.state.pending.is_none());
    assert_done(&run);
    finish_done(&mut run);
}

#[test]
fn m28_designer_full_service_softlock() {
    let mut run = designer_with_option(4, 500, &[
        CardId::AscendersBane, CardId::CurseOfTheBell, CardId::Necronomicurse,
    ]);
    let deck = run.state.deck.clone();
    choose(&mut run, 0, 4);
    assert_eq!(run.state.gold, 410);
    assert_eq!(run.state.deck, deck, "the short grid prevents both removal and follow-up upgrades");
    assert!(run.state.pending.is_none());
    assert_done(&run);
    finish_done(&mut run);
}

#[test]
fn m28_duplicator_duplicate() {
    let mut run = at_event(Character::Ironclad, 2, EventId::Duplicator, 24);
    set_deck(&mut run, &[CardId::Bash]);
    let original = run.state.deck[0].instance;
    run.state.deck[0].upgrades = 1;
    install_bottle(&mut run, RelicId::BottledFlame, original);
    choose(&mut run, 0, 0);
    select_instance(&mut run, original);
    assert_eq!(run.state.deck.len(), 2);
    let copy = run.state.deck.iter().find(|card| card.instance != original).unwrap();
    assert_eq!((copy.id, copy.upgrades), (CardId::Bash, 1), "duplicate keeps its upgrade");
    assert_eq!(run.state.relic_cards.get(&RelicId::BottledFlame), Some(&original),
        "only the original remains bottled");
    finish_done(&mut run);
}

#[test]
fn m28_duplicator_duplicate_omamori() {
    let mut run = at_event(Character::Silent, 2, EventId::Duplicator, 25);
    set_deck(&mut run, &[CardId::Writhe]);
    let original = run.state.deck[0].instance;
    run.state.relics.push(RelicId::Omamori);
    run.state.relic_counters.insert(RelicId::Omamori, 2);
    choose(&mut run, 0, 0);
    select_instance(&mut run, original);
    assert_eq!(run.state.deck.len(), 1, "the copied Curse is negated");
    assert_eq!(run.state.deck[0].instance, original);
    assert_eq!(run.state.relic_counters.get(&RelicId::Omamori), Some(&1));
    finish_done(&mut run);
}

#[test]
fn m28_duplicator_duplicate_molten_egg() {
    let mut run = at_event(Character::Ironclad, 2, EventId::Duplicator, 26);
    set_deck(&mut run, &[CardId::Strike]);
    let original = run.state.deck[0].instance;
    run.state.relics.push(RelicId::MoltenEgg);
    choose(&mut run, 0, 0);
    select_instance(&mut run, original);
    assert_eq!(run.state.deck.iter().find(|card| card.instance == original).unwrap().upgrades, 0);
    assert_eq!(run.state.deck.iter().find(|card| card.instance != original).unwrap().upgrades, 1);
    finish_done(&mut run);
}

#[test]
fn m28_duplicator_ignore() {
    let mut run = at_event(Character::Watcher, 2, EventId::Duplicator, 27);
    let before = run.state.clone();
    choose(&mut run, 0, 1);
    assert_eq!(view(&run).kind, RoomViewKind::Map);
    assert_eq!((run.state.hp, run.state.max_hp, run.state.gold, run.state.deck, run.state.relics),
        (before.hp, before.max_hp, before.gold, before.deck, before.relics));
}

#[test]
fn m28_knowing_skull_costs_escalate_per_option() {
    let mut run = at_event(Character::Defect, 2, EventId::KnowingSkull, 28);
    let hp = run.state.hp;
    let gold = run.state.gold;
    for expected_cost in [6_u32, 7, 8] {
        let old_hp = run.state.hp;
        choose(&mut run, 0, 1);
        assert_eq!(run.state.hp, old_hp - expected_cost);
    }
    assert_eq!(run.state.gold, gold + 270, "three fixed +90 rewards");
    assert_eq!(run.state.hp, hp - 21);

    for expected_cost in [6_u32, 7, 8] {
        let old_hp = run.state.hp;
        let old_potions = run.state.potions.iter().filter(|potion| potion.is_some()).count();
        choose(&mut run, 0, 0);
        assert_eq!(run.state.hp, old_hp - expected_cost);
        assert_eq!(run.state.potions.iter().filter(|potion| potion.is_some()).count(), old_potions + 1);
    }
    for expected_cost in [6_u32, 7, 8] {
        let old_hp = run.state.hp;
        let old_deck = run.state.deck.len();
        choose(&mut run, 0, 2);
        assert_eq!(run.state.hp, old_hp - expected_cost);
        assert_eq!(run.state.deck.len(), old_deck + 1);
        let card = run.state.deck.last().unwrap().id;
        let info = Catalog::standard().card(card).unwrap();
        assert_eq!((info.color, info.rarity), (CardColor::Colorless, CardRarity::Uncommon));
    }
}

#[test]
fn m28_knowing_skull_potion_full_slots_and_sozu() {
    let mut full = at_event(Character::Ironclad, 2, EventId::KnowingSkull, 29);
    full.state.potions = vec![Some(PotionId::FirePotion); 3];
    let before = full.state.potions.clone();
    let hp = full.state.hp;
    choose(&mut full, 0, 0);
    assert_eq!(full.state.hp, hp - 6);
    assert_eq!(full.state.potions, before, "full slots discard the drawn potion");

    let mut sozu = at_event(Character::Watcher, 2, EventId::KnowingSkull, 30);
    sozu.state.relics.push(RelicId::Sozu);
    let before = sozu.state.potions.clone();
    let hp = sozu.state.hp;
    choose(&mut sozu, 0, 0);
    assert_eq!(sozu.state.hp, hp - 6);
    assert_eq!(sozu.state.potions, before, "Sozu suppresses the draw and obtain");
}

#[test]
fn m28_knowing_skull_card() {
    let mut run = at_event(Character::Silent, 2, EventId::KnowingSkull, 31);
    let hp = run.state.hp;
    let old_deck = run.state.deck.len();
    choose(&mut run, 0, 2);
    assert_eq!((run.state.hp, run.state.deck.len()), (hp - 6, old_deck + 1));
    let card = run.state.deck.last().unwrap().id;
    let info = Catalog::standard().card(card).unwrap();
    assert_eq!((info.color, info.rarity), (CardColor::Colorless, CardRarity::Uncommon));
}

#[test]
fn m28_knowing_skull_leave() {
    let mut run = at_event(Character::Ironclad, 2, EventId::KnowingSkull, 32);
    let hp = run.state.hp;
    choose(&mut run, 0, 1);
    let after_gold = run.state.hp;
    assert_eq!(after_gold, hp - 6);
    choose(&mut run, 0, 3);
    assert_eq!(view(&run).kind, RoomViewKind::Map);
    assert_eq!(run.state.hp, after_gold - 6, "Leave always costs six after another option was used");
}

#[test]
fn m28_nloth_trade_a() {
    let mut run = nloth_event(Character::Silent, 33, false);
    let offered = nloth_offered_relic(&run, 0);
    let old_count = run.state.relics.len();
    choose_nloth(&mut run, 0);
    assert_eq!(run.state.relics.len(), old_count);
    assert!(!run.state.relics.contains(&offered), "the offered relic is traded");
    assert!(run.state.relics.contains(&RelicId::NlothsGift));
    finish_done(&mut run);
}

#[test]
fn m28_nloth_trade_b() {
    let mut run = nloth_event(Character::Defect, 34, false);
    let offered = nloth_offered_relic(&run, 1);
    let old_count = run.state.relics.len();
    choose_nloth(&mut run, 1);
    assert_eq!(run.state.relics.len(), old_count);
    assert!(!run.state.relics.contains(&offered), "the offered relic is traded");
    assert!(run.state.relics.contains(&RelicId::NlothsGift));
    finish_done(&mut run);
}

#[test]
fn m28_nloth_trade_with_gift_owned() {
    let mut run = nloth_event(Character::Watcher, 35, true);
    let offered = nloth_offered_relic(&run, 1);
    let old_count = run.state.relics.len();
    choose_nloth(&mut run, 1);
    assert_eq!(run.state.relics.len(), old_count + 1);
    assert!(run.state.relics.contains(&RelicId::NlothsGift));
    assert!(run.state.relics.contains(&RelicId::Circlet));
    assert!(run.state.relics.contains(&offered), "Gift already owned keeps the offered relic");
    finish_done(&mut run);
}

#[test]
fn m28_nloth_leave() {
    let mut run = nloth_event(Character::Ironclad, 36, false);
    let offers = [nloth_offered_relic(&run, 0), nloth_offered_relic(&run, 1)];
    assert_ne!(offers[0], offers[1], "N'loth shows two distinct relics");
    let before = run.state.clone();
    let rng = run.state.rng.clone();
    choose(&mut run, 0, 2);
    assert_eq!(view(&run).kind, RoomViewKind::Map);
    assert_eq!((run.state.hp, run.state.max_hp, run.state.gold, run.state.deck, run.state.relics),
        (before.hp, before.max_hp, before.gold, before.deck, before.relics));
    assert_eq!(run.state.rng, rng);
}

// Review-fixes I1 (PM decompile, m28f review-fixes): N'loth trades through `AbstractPlayer.loseRelic`,
// which runs the relic's `onUnequip` before removing it.

/// N'loth with exactly `[relic, Anchor]` owned (so both are offered), after `setup`.
fn nloth_owning(character: Character, relic: RelicId, seed: u8, setup: impl FnOnce(&mut Run)) -> Run {
    let (mut run, node) = before_event(character, 2, EventId::Nloth, seed);
    run.state.relics = vec![relic, RelicId::Anchor];
    run.state.relic_counters.clear();
    setup(&mut run);
    run.step(&GameAction::Move(node)).unwrap();
    assert_eq!(view(&run).kind, RoomViewKind::Event(EventId::Nloth));
    run
}

fn trade_relic(run: &mut Run, relic: RelicId) {
    let id = view(run).choices.into_iter().find(|choice|
        matches!(choice.kind, RoomChoiceKind::EventRelic { stage: 0, relic: Some(offered), .. } if offered == relic))
        .unwrap_or_else(|| panic!("N'loth did not offer {relic:?}")).id;
    run.step(&GameAction::Choose(id)).unwrap();
    assert!(!run.state.relics.contains(&relic), "{relic:?} is traded");
    assert!(run.state.relics.contains(&RelicId::NlothsGift));
}

/// From the map, enter the next room as a Monster room; the opening combat view.
fn fight_next(run: &mut Run) -> crate::combat::CombatPublic {
    assert_eq!(view(run).kind, RoomViewKind::Map);
    let target = run.decision().unwrap().candidates.iter()
        .find_map(|c| match c.action { GameAction::Move(n) => Some(n), _ => None }).unwrap();
    run.state.map.as_mut().unwrap().nodes.iter_mut().find(|n| n.id == target).unwrap().kind = MapNodeKind::Monster;
    run.step(&GameAction::Move(target)).unwrap();
    run.state.combat.as_ref().expect("the Monster room enters combat").public_full().unwrap()
}

/// The next combat with `relic` kept (N'loth's Leave) and with it traded away.
fn kept_and_traded(character: Character, relic: RelicId, seed: u8) -> (crate::combat::CombatPublic, crate::combat::CombatPublic) {
    let mut kept = nloth_owning(character, relic, seed, |_| {});
    choose(&mut kept, 0, 2);
    let mut traded = nloth_owning(character, relic, seed, |_| {});
    trade_relic(&mut traded, relic);
    leave(&mut traded);
    (fight_next(&mut kept), fight_next(&mut traded))
}

#[test]
fn m28_nloth_trade_necronomicon_removes_first_curse_without_hooks() {
    let mut first = 0;
    let mut second = 0;
    let mut run = nloth_owning(Character::Ironclad, RelicId::Necronomicon, 40, |run| {
        first = run.state.next_instance;
        second = first + 1;
        run.state.next_instance += 2;
        run.state.deck.insert(1, CardInstance { id: CardId::Necronomicurse, instance: first, upgrades: 0, misc: 0 });
        run.state.deck.push(CardInstance { id: CardId::Necronomicurse, instance: second, upgrades: 0, misc: 0 });
    });
    let mut expected = run.state.deck.clone();
    expected.retain(|card| card.instance != first);
    let max_hp = run.state.max_hp;
    trade_relic(&mut run, RelicId::Necronomicon);
    // `onUnequip` removes the first Necronomicurse by a direct `group.remove`: no copy returns.
    assert_eq!(run.state.deck, expected);
    assert_eq!(count(&run, CardId::Necronomicurse), 1);
    assert_eq!(run.state.max_hp, max_hp);
    finish_done(&mut run);
}

#[test]
fn m28_nloth_trade_energy_relic_loses_one_energy_once() {
    for relic in [RelicId::CursedKey, RelicId::BustedCrown, RelicId::Sozu] {
        let (kept, traded) = kept_and_traded(Character::Ironclad, relic, 41);
        assert_eq!((kept.player.max_energy, traded.player.max_energy), (4, 3), "{relic:?}");
    }
}

#[test]
fn m28_nloth_trade_snecko_eye_loses_two_cards_once() {
    let (kept, traded) = kept_and_traded(Character::Ironclad, RelicId::SneckoEye, 42);
    assert_eq!((kept.hand.len(), traded.hand.len()), (7, 5));
}

#[test]
fn m28_nloth_trade_ring_of_the_serpent_loses_one_card_once() {
    let (kept, traded) = kept_and_traded(Character::Silent, RelicId::RingOfTheSerpent, 43);
    assert_eq!((kept.hand.len(), traded.hand.len()), (6, 5));
}

#[test]
fn m28_nloth_trade_bottle_clears_its_card_flag() {
    for (relic, card) in [(RelicId::BottledFlame, CardId::Bash), (RelicId::BottledLightning, CardId::Defend),
        (RelicId::BottledTornado, CardId::Inflame)]
    {
        let mut run = nloth_owning(Character::Ironclad, relic, 44, |run| {
            let instance = run.state.next_instance;
            run.state.next_instance += 1;
            run.state.deck.push(CardInstance { id: card, instance, upgrades: 0, misc: 0 });
            run.state.relic_cards.insert(relic, instance);
        });
        let deck = run.state.deck.clone();
        trade_relic(&mut run, relic);
        assert!(run.state.relic_cards.is_empty(), "{relic:?}: its card is no longer bottled");
        assert_eq!(run.state.deck, deck, "{relic:?}: the card stays in the deck");
    }
}

#[test]
fn m28_nloth_trade_no_op_relic_changes_nothing_else() {
    let mut run = nloth_owning(Character::Defect, RelicId::Vajra, 45, |_| {});
    let before = run.state.clone();
    trade_relic(&mut run, RelicId::Vajra);
    assert_eq!(run.state.relics, vec![RelicId::Anchor, RelicId::NlothsGift]);
    assert_eq!((&run.state.deck, run.state.hp, run.state.max_hp, run.state.gold, &run.state.potions,
        &run.state.relic_cards, &run.state.relic_counters),
        (&before.deck, before.hp, before.max_hp, before.gold, &before.potions,
        &before.relic_cards, &before.relic_counters));
    leave(&mut run);
    let traded = fight_next(&mut run);
    assert_eq!((traded.player.max_energy, traded.hand.len()), (3, 5));
}

// Review-fixes M3: the reviewer's probes (m28f-review-v1 §M3), as tests.

#[test]
fn m28_knowing_skull_lethal_payment_ends_the_run() {
    let mut run = at_event(Character::Ironclad, 2, EventId::KnowingSkull, 28);
    run.state.hp = 6;
    let gold = run.state.gold;
    let result = choose(&mut run, 0, 1);
    assert_eq!(result.terminal, Some(RunTerminal::Death));
    assert_eq!((run.state.hp, run.state.gold), (0, gold), "the payment kills before the gold");
}

#[test]
fn m28_knowing_skull_tungsten_rod_reduces_the_payment() {
    let mut run = at_event(Character::Ironclad, 2, EventId::KnowingSkull, 28);
    run.state.relics.push(RelicId::TungstenRod);
    let hp = run.state.hp;
    choose(&mut run, 0, 1);
    assert_eq!(hp - run.state.hp, 5, "HP_LOSS 6 through Tungsten Rod");
}

#[test]
fn m28_nloth_gift_and_circlet_owned_stacks_the_circlet() {
    let (mut run, node) = before_event(Character::Silent, 2, EventId::Nloth, 33);
    run.state.relics.extend([RelicId::Anchor, RelicId::HornCleat, RelicId::NlothsGift, RelicId::Circlet]);
    run.state.relic_counters.insert(RelicId::Circlet, 1);
    run.step(&GameAction::Move(node)).unwrap();
    let offered = nloth_offered_relic(&run, 0);
    choose_nloth(&mut run, 0);
    assert_eq!(run.state.relic_counters.get(&RelicId::Circlet), Some(&2));
    assert_eq!(run.state.relics.iter().filter(|relic| **relic == RelicId::Circlet).count(), 1);
    assert!(run.state.relics.contains(&offered));
    finish_done(&mut run);
}

#[test]
fn m28_designer_clean_remove_bottled_only_strands() {
    // Unbottled count 1 enables Clean Up remove; the grid (purgeable and unbottled) is empty (Q1).
    let mut run = designer_with_bottle(2, 500, &[CardId::Bash, CardId::AscendersBane], RelicId::BottledFlame, 0);
    let remove = row(&run, &RoomChoiceKind::EventOption { stage: 0, option: 2 });
    assert_eq!(remove.status, RoomChoiceStatus::Offered);
    let deck = run.state.deck.clone();
    run.step(&GameAction::Choose(remove.id)).unwrap();
    assert_eq!(run.state.gold, 440);
    assert_eq!(run.state.deck, deck);
    assert!(run.state.pending.is_none());
    finish_done(&mut run);
}

#[test]
fn m28_mysterious_sphere_relic_is_drawn_at_the_fight_click() {
    let mut run = at_event(Character::Ironclad, 3, EventId::MysteriousSphere, 13);
    let pools = run.state.relic_pools.clone();
    choose(&mut run, 0, 0);
    assert!(run.state.combat.is_some());
    assert_ne!(run.state.relic_pools, pools, "the Rare relic leaves its pool before the combat (D3)");
}
