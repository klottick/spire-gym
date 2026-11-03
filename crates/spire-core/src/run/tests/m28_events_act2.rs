//! M28e Act 2 event bodies and event combats (chunk plan §4 M28e; plan-check m28e/plan-check-v1).
use super::super::*;
use super::m28_event_core::{at_event, before_event, count, leave, row, view};
use crate::content::{CardId, CardTag, Character, EncounterId, EventId, PotionId, RelicId};
use crate::run::{CardInstance, GameAction, RoomChoiceKind, RoomChoiceStatus, RoomViewKind, Run, RunTransition};
use rooms::standard::event_exec::Outcome;
use rooms::standard::event_frame;

/// N3 (PM source check m28e/source-check-event-escape-v1 ‡): a rewards-disabled event fight (the
/// Colosseum's first) still runs the EventRoom potion roll (`addPotionToRewards` precedes the
/// `rewardAllowed` check) after a victory or a Smoke Bomb escape; `Colosseum.reopen` then clears the
/// rewards, so only the RNG and the potion offset change and the event reopens at its next stage.
#[test]
fn m28_event_combat_rewards_disabled_still_rolls_potion() {
    for escape in [false, true] {
        let mut run = at_event(Character::Ironclad, 3, EventId::GoldenShrine, 50 + u8::from(escape));
        run.state.potions[0] = Some(crate::content::PotionId::SmokeBomb);
        let (offset, potions, gold) = (run.state.potion_offset, run.state.potions.clone(), run.state.gold);
        event_frame::choose_constructed(&mut run, vec![Outcome::Combat { encounter: EncounterId::OrbWalker, rewards: None, next_stage: 0, elite: false }]).unwrap();
        let before = run.state.rng.clone();
        if escape { run.choose_combat(crate::combat::CombatAction::UsePotion { slot: 0, target: None }).unwrap(); } else { super::m28_world::win(&mut run); }
        assert!(run.state.combat.is_none() && run.state.fault.is_none());
        assert_eq!((run.state.potion_offset - offset).abs(), 10, "escape={escape}: the suppressed potion roll moved the offset");
        assert_ne!(run.state.rng, before, "escape={escape}: the potion roll drew");
        let rows = view(&run);
        assert_eq!(rows.kind, RoomViewKind::Event(EventId::GoldenShrine), "escape={escape}: no reward screen");
        assert!(rows.choices.iter().all(|c| matches!(c.kind, RoomChoiceKind::EventOption { stage: 0, .. }) && c.status == RoomChoiceStatus::Offered), "{rows:?}");
        if !escape { assert_eq!(run.state.potions, potions, "nothing is granted"); }
        assert_eq!(run.state.gold, gold, "escape={escape}: no gold");
    }
}

/// Source order (PM, check m28e/source-check-open-b-v1 item 12; Mind Bloom and Dead Adventurer add theirs the same
/// way): an event fight's tier relic rows leave the pools when the fight starts (`addRelicToRewards` before
/// `enterCombat`), not at settlement; the reward screen shows the relics taken then, and a Smoke Bomb escape takes
/// nothing more.
#[test]
fn m28_event_combat_relic_rows_taken_at_fight_start() {
    use crate::content::{Catalog, RelicTier};
    use rooms::standard::event_exec::{RelicChoice, RewardSpec};
    for escape in [false, true] {
        let mut run = at_event(Character::Defect, 3, EventId::GoldenShrine, 60 + u8::from(escape));
        run.state.potions[0] = Some(crate::content::PotionId::SmokeBomb);
        let pools = run.state.relic_pools.clone();
        let spec = RewardSpec { gold: 100, gold_range: None, relics: vec![RelicChoice::Tier(RelicTier::Rare), RelicChoice::Tier(RelicTier::Uncommon)] };
        event_frame::choose_constructed(&mut run, vec![Outcome::Combat { encounter: EncounterId::OrbWalker, rewards: Some(spec), next_stage: rooms::standard::event_frame::DONE, elite: true }]).unwrap();
        assert_ne!(run.state.relic_pools, pools, "escape={escape}: both rows taken when the fight starts");
        let started = run.state.relic_pools.clone();
        if escape { run.choose_combat(crate::combat::CombatAction::UsePotion { slot: 0, target: None }).unwrap(); } else { super::m28_world::win(&mut run); }
        assert_eq!(run.state.relic_pools, started, "escape={escape}: settlement takes nothing more");
        if !escape {
            let relics: Vec<_> = view(&run).choices.iter().filter_map(|c| match c.kind { RoomChoiceKind::Relic(r) => Some(r), _ => None }).collect();
            let tiers: Vec<_> = relics.iter().map(|r| Catalog::standard().relic_tier(*r)).collect();
            assert_eq!(tiers, vec![Some(RelicTier::Rare), Some(RelicTier::Uncommon)], "Rare row then Uncommon row");
        }
    }
}

/// PM-approved shared outcome (We Meet Again): `LosePotion(slot)` empties a held slot; an empty slot is rejected
/// before any mutation.
#[test]
fn m28_event_outcome_lose_potion() {
    use crate::content::PotionId;
    let mut run = at_event(Character::Watcher, 2, EventId::GoldenShrine, 70);
    run.state.potions[1] = Some(PotionId::FirePotion);
    event_frame::choose_constructed(&mut run, vec![Outcome::LosePotion(1), Outcome::NextStage(rooms::standard::event_frame::DONE)]).unwrap();
    assert_eq!(run.state.potions[1], None);
    let mut run = at_event(Character::Watcher, 2, EventId::GoldenShrine, 71);
    run.state.potions = vec![None; run.state.potions.len()];
    let before = run.clone();
    assert_eq!(event_frame::choose_constructed(&mut run, vec![Outcome::LosePotion(0), Outcome::GainGold(5)]).unwrap_err(), crate::run::RunError::InvalidAction);
    assert!(run == before, "rejected atomically");
}

/// PM-approved shared outcome (Vampires, G1): `RemoveTagged(StarterStrike)` removes every starter Strike, upgraded
/// ones included, in one step; a deck of only Strikes does not end the Run before the list's later obtains.
#[test]
fn m28_event_outcome_remove_tagged_starter_strikes() {
    use crate::content::{CardId, CardTag};
    for (seed, character, strike) in [(72, Character::Ironclad, CardId::Strike), (73, Character::Silent, CardId::StrikeGreen),
        (74, Character::Defect, CardId::StrikeBlue), (75, Character::Watcher, CardId::StrikePurple)] {
        let mut run = at_event(character, 2, EventId::GoldenShrine, seed);
        run.state.deck[0].upgrades = 1;
        let others: Vec<_> = run.state.deck.iter().filter(|c| c.id != strike).cloned().collect();
        assert!(run.state.deck.iter().filter(|c| c.id == strike).count() >= 4);
        event_frame::choose_constructed(&mut run, vec![Outcome::RemoveTagged(CardTag::StarterStrike), Outcome::NextStage(rooms::standard::event_frame::DONE)]).unwrap();
        assert_eq!(run.state.deck, others, "{character:?}: only the starter Strikes left");
        let mut run = at_event(character, 2, EventId::GoldenShrine, seed);
        run.state.deck.retain(|c| c.id == strike);
        event_frame::choose_constructed(&mut run, vec![Outcome::RemoveTagged(CardTag::StarterStrike), Outcome::ObtainCard { id: CardId::Bite, upgrades: 0 }, Outcome::NextStage(rooms::standard::event_frame::DONE)]).unwrap();
        assert_eq!((run.state.terminal, run.state.deck.iter().map(|c| c.id).collect::<Vec<_>>()), (None, vec![CardId::Bite]), "{character:?}");
    }
}

/// Shared outcome for The Library's Read (PM check m28e/source-check-open-a-v1 item 7): `CardBoard{count}` generates
/// `count` distinct ids at the choice, from the character's pools, reading the card-rarity offset without moving it;
/// the ids become the frame's board.
#[test]
fn m28_event_outcome_card_board() {
    for (seed, character) in [(80, Character::Ironclad), (81, Character::Silent), (82, Character::Defect), (83, Character::Watcher)] {
        let mut run = at_event(character, 2, EventId::GoldenShrine, seed);
        let offset = run.state.card_rarity_offset;
        event_frame::choose_constructed(&mut run, vec![Outcome::CardBoard { count: 20 }, Outcome::NextStage(1)]).unwrap();
        let ids = run.state.room_loop.as_ref().unwrap().standard_event.as_ref().unwrap().captured.card_ids.clone();
        assert_eq!(ids.len(), 20, "{character:?}");
        assert_eq!(ids.iter().collect::<std::collections::BTreeSet<_>>().len(), 20, "{character:?}: distinct ids");
        let pool = crate::content::pools::cards(crate::content::pools::CardPool::Ordinary { character, rarity: None, card_type: None }).unwrap();
        assert!(ids.iter().all(|id| pool.contains(id)), "{character:?}: the character's pools");
        assert_eq!(run.state.card_rarity_offset, offset, "{character:?}: the offset is read, not advanced");
    }
}

/// M27's `we_meet_again` potion lock (PM decompile, review-v1 M1): `AbstractPotion.canDiscard()` and the
/// out-of-combat potions' `canUse()` test `getCurrRoom().event instanceof WeMeetAgain`, and the room stays
/// the event room after `openMap` until the next room is entered. So every potion is locked from the event's
/// start, on the map after Leave included, and unlocked by the next room entry.
#[test]
fn m28_we_meet_again_locks_potions_until_the_next_room() {
    let (mut run, node) = super::m28_event_core::before_event(Character::Ironclad, 2, EventId::WeMeetAgain, 84);
    run.state.potions = vec![Some(PotionId::FruitJuice), Some(PotionId::BloodPotion), Some(PotionId::EntropicBrew)];
    assert!(!run.state.we_meet_again);
    run.step(&GameAction::Move(node)).unwrap();
    let potion_actions = |run: &Run| run.decision().unwrap().candidates.iter()
        .filter(|c| matches!(c.action, GameAction::UsePotion { .. } | GameAction::DiscardPotion { .. })).count();
    assert!(run.state.we_meet_again, "set while the event is live");
    assert_eq!(potion_actions(&run), 0, "no potion use or discard on the event page");
    choose_at(&mut run, 0, 3);
    assert_eq!(view(&run).kind, RoomViewKind::Map);
    assert!(run.state.we_meet_again, "the event is still the current room's on the map");
    assert_eq!(potion_actions(&run), 0, "no potion use or discard on the map after Leave");
    for slot in 0..3 {
        let before = run.clone();
        assert_eq!(run.step(&GameAction::UsePotion { slot, target: None }), Err(crate::run::RunError::InvalidAction));
        assert_eq!(run.step(&GameAction::DiscardPotion { slot }), Err(crate::run::RunError::InvalidAction));
        assert!(run == before, "slot {slot}: rejected atomically");
    }
    let next = run.decision().unwrap().candidates.iter().find_map(|c| match c.action { GameAction::Move(id) => Some(id), _ => None }).unwrap();
    run.state.map.as_mut().unwrap().nodes.iter_mut().find(|n| n.id == next).unwrap().kind = MapNodeKind::Rest;
    run.step(&GameAction::Move(next)).unwrap();
    assert_eq!(view(&run).kind, RoomViewKind::Campfire);
    assert!(!run.state.we_meet_again, "cleared by the next room entry");
    let actions: Vec<_> = run.decision().unwrap().candidates.into_iter().map(|c| c.action).collect();
    assert!(actions.contains(&GameAction::UsePotion { slot: 0, target: None }) && (0..3).all(|slot| actions.contains(&GameAction::DiscardPotion { slot })), "{actions:?}");
    let max_hp = run.state.max_hp;
    run.step(&GameAction::UsePotion { slot: 0, target: None }).unwrap();
    assert_eq!((run.state.max_hp, run.state.potions[0]), (max_hp + 5, None), "Fruit Juice works in the next room");
    run.step(&GameAction::DiscardPotion { slot: 1 }).unwrap();
    assert_eq!(run.state.potions[1], None);
}

/// PM ruling (Drug Dealer bottled cards): removing or transforming a bottled card drops its bottle mapping (the
/// source's mark travels with the card), and the next combat starts without a fault or the bottled placement.
#[test]
fn m28_removed_bottled_card_leaves_no_bottle_mapping() {
    use crate::content::{CardId, RelicId};
    let mut run = at_event(Character::Ironclad, 3, EventId::GoldenShrine, 90);
    let bash = run.state.deck.iter().find(|c| c.id == CardId::Bash).unwrap().instance;
    run.state.relics.push(RelicId::BottledFlame);
    run.state.relic_cards.insert(RelicId::BottledFlame, bash);
    event_frame::choose_constructed(&mut run, vec![Outcome::RemoveSpecific(bash), Outcome::Combat { encounter: EncounterId::OrbWalker, rewards: None, next_stage: 0, elite: false }]).unwrap();
    assert!(run.state.relic_cards.is_empty(), "no stale bottle mapping: {:?}", run.state.relic_cards);
    assert!(run.state.fault.is_none() && run.state.combat.is_some(), "the next combat starts");
}

/// PM ruling (bottled cards): upgrade grids (`getUpgradableCards()`, e.g. Accursed Blacksmith's Forge) offer a bottled
/// card; remove and transform grids other than Drug Dealer's strip it (Transmogrifier here).
#[test]
fn m28_bottled_cards_in_upgrade_and_transform_grids() {
    use crate::content::{CardId, RelicId};
    for (event, option, offered) in [(EventId::AccursedBlacksmith, 0, true), (EventId::Transmogrifier, 0, false)] {
        let mut run = at_event(Character::Ironclad, 2, event, 91);
        let bash = run.state.deck.iter().find(|c| c.id == CardId::Bash).unwrap().instance;
        run.state.relics.push(RelicId::BottledFlame);
        run.state.relic_cards.insert(RelicId::BottledFlame, bash);
        let id = view(&run).choices.into_iter().find(|c| c.kind == RoomChoiceKind::EventOption { stage: 0, option }).unwrap().id;
        run.step(&crate::run::GameAction::Choose(id)).unwrap();
        let candidates = &run.state.pending.as_ref().expect("selection opened").selection.candidates;
        assert_eq!(candidates.iter().any(|c| c.card.instance == bash), offered, "{event:?}");
    }
}

/// Shared outcome for Forgotten Altar (PM check m28e/source-check-mechanics-c-v1 item 3, `instantObtain` with the equip
/// flag false): `ReplaceRelic` puts the new relic in the old relic's slot, with no equip effect; it needs the old relic
/// owned and the new one unowned, checked before any mutation.
#[test]
fn m28_event_outcome_replace_relic_in_slot() {
    use crate::content::RelicId;
    let mut run = at_event(Character::Silent, 2, EventId::GoldenShrine, 92);
    run.state.relics.extend([RelicId::GoldenIdol, RelicId::Anchor]);
    let mut expected = run.state.relics.clone();
    let slot = expected.iter().position(|r| *r == RelicId::GoldenIdol).unwrap();
    expected[slot] = RelicId::BloodyIdol;
    let (hp, gold) = (run.state.hp, run.state.gold);
    event_frame::choose_constructed(&mut run, vec![Outcome::ReplaceRelic { old: RelicId::GoldenIdol, new: RelicId::BloodyIdol }, Outcome::NextStage(rooms::standard::event_frame::DONE)]).unwrap();
    assert_eq!((run.state.relics.clone(), run.state.hp, run.state.gold), (expected, hp, gold));
    let before = run.clone();
    assert_eq!(event_frame::choose_constructed(&mut run, vec![Outcome::ReplaceRelic { old: RelicId::GoldenIdol, new: RelicId::BloodyIdol }]).unwrap_err(), crate::run::RunError::InvalidAction);
    assert!(run == before);
}

/// Global (replaces lane N's encounter-named guard): an event combat whose enemy setup is unavailable at the Run's
/// ascension is the typed `Unsupported`, checked before any mutation, for every event encounter (M28d's Three
/// Sentries at A15 here), never a fault after acceptance. Supported pairs (Gremlin Nob at A15) still start.
/// M29c lands the Sentry, so the unavailable witness moves to Masked Bandits (M29d) at A15 (name kept).
/// M29d lands Masked Bandits: the witness is A17, above its encounter ceiling (16).
#[test]
fn m28_event_combat_unsupported_ascension_is_rejected_up_front() {
    let mut run = at_event(Character::Ironclad, 1, EventId::GoldenShrine, 94);
    run.state.ascension = 17;
    let before = run.clone();
    let result = event_frame::choose_constructed(&mut run, vec![Outcome::Combat { encounter: EncounterId::MaskedBandits, rewards: None, next_stage: 0, elite: true }]);
    assert!(matches!(result, Err(crate::run::RunError::Unsupported(_))), "{result:?}");
    assert!(run == before && run.state.fault.is_none(), "rejected atomically, no fault");
    event_frame::choose_constructed(&mut run, vec![Outcome::Combat { encounter: EncounterId::GremlinNob, rewards: None, next_stage: 0, elite: true }]).unwrap();
    assert!(run.state.combat.is_some());
}

// ---- Lane B: the remaining eleven non-combat M28e event bodies ----

fn event_frame(run: &Run) -> &event_frame::StandardEventFrame {
    run.state.room_loop.as_ref().unwrap().standard_event.as_ref().unwrap()
}

/// The event row at (`stage`, `option`), whatever it reveals (option, card or potion row).
fn event_row(run: &Run, stage: u8, option: u8) -> crate::run::RoomChoiceView {
    view(run).choices.into_iter().find(|c| matches!(c.kind, RoomChoiceKind::EventOption { stage: s, option: o }
        | RoomChoiceKind::EventCard { stage: s, option: o, .. } | RoomChoiceKind::EventPotion { stage: s, option: o, .. } if s == stage && o == option))
        .unwrap_or_else(|| panic!("no event row at stage {stage}, option {option}"))
}

fn choose_at(run: &mut Run, stage: u8, option: u8) -> RunTransition {
    let id = event_row(run, stage, option).id;
    run.step(&GameAction::Choose(id)).unwrap()
}

fn assert_done(run: &Run) {
    assert_eq!(event_frame(run).stage, event_frame::DONE);
    assert_eq!(view(run).choices.iter().map(|choice| choice.kind.clone()).collect::<Vec<_>>(), vec![RoomChoiceKind::Leave]);
}

fn finish_done(run: &mut Run) {
    assert_done(run);
    leave(run);
    assert_eq!(view(run).kind, RoomViewKind::Map);
}

fn event_card(run: &mut Run, stage: u8, option: u8) -> RunTransition {
    let choice = view(run).choices.into_iter().find(|choice| matches!(choice.kind,
        RoomChoiceKind::EventCard { stage: s, option: o, .. } if s == stage && o == option))
        .unwrap_or_else(|| panic!("no event card row at stage {stage}, option {option}"));
    run.step(&GameAction::Choose(choice.id)).unwrap()
}

fn select_instance(run: &mut Run, instance: u64) -> RunTransition {
    let pending = run.state.pending.as_ref().expect("event selection is open");
    let id = pending.selection.candidates.iter().find(|candidate| candidate.card.instance == instance)
        .unwrap_or_else(|| panic!("card instance {instance} was not offered: {:?}", pending.selection.candidates)).id;
    run.step(&GameAction::Select(id)).unwrap()
}

fn set_deck(run: &mut Run, ids: &[CardId]) {
    let first = run.state.deck.first().map_or(10_000, |card| card.instance + 100);
    run.state.deck = ids.iter().enumerate().map(|(i, id)| CardInstance {
        id: *id, instance: first + i as u64, upgrades: 0, misc: 0,
    }).collect();
    run.state.relic_cards.clear();
}

fn enter_with(character: Character, event: EventId, seed: u8, setup: impl FnOnce(&mut Run)) -> Run {
    let (mut run, node) = before_event(character, 2, event, seed);
    setup(&mut run);
    run.step(&GameAction::Move(node)).unwrap();
    assert_eq!(view(&run).kind, RoomViewKind::Event(event));
    run
}

fn tome_to_final(run: &mut Run) {
    choose_at(run, 0, 0); // Read
    choose_at(run, 1, 0); // Continue 1
    choose_at(run, 2, 0); // Continue 2
    choose_at(run, 3, 0); // Continue 3
    assert_eq!(event_frame(run).stage, 4);
}

fn strike_id(character: Character) -> CardId {
    match character {
        Character::Ironclad => CardId::Strike,
        Character::Silent => CardId::StrikeGreen,
        Character::Defect => CardId::StrikeBlue,
        Character::Watcher => CardId::StrikePurple,
    }
}

fn potion_is_in_class_pool(character: Character, potion: PotionId) -> bool {
    use PotionId::*;
    let shared = [BlockPotion, DexterityPotion, EnergyPotion, ExplosivePotion, FirePotion, StrengthPotion,
        SwiftPotion, WeakPotion, FearPotion, AttackPotion, SkillPotion, PowerPotion, ColorlessPotion,
        FlexPotion, SpeedPotion, BlessingOfTheForge, RegenPotion, AncientPotion, LiquidBronze,
        GamblersBrew, EssenceOfSteel, DuplicationPotion, DistilledChaos, LiquidMemories, CultistPotion,
        FruitJuice, SneckoOil, FairyPotion, SmokeBomb, EntropicBrew];
    if shared.contains(&potion) { return true; }
    match character {
        Character::Ironclad => [BloodPotion, Elixir, HeartOfIron].contains(&potion),
        Character::Silent => [PoisonPotion, CunningPotion, GhostInAJar].contains(&potion),
        Character::Defect => [FocusPotion, PotionOfCapacity, EssenceOfDarkness].contains(&potion),
        Character::Watcher => [BottledMiracle, StancePotion, Ambrosia].contains(&potion),
    }
}

fn buy_woman_potions(character: Character, option: u8, amount: usize, price: u32, seed: u8) {
    let mut run = enter_with(character, EventId::WomanInBlue, seed, |run| run.state.gold = 100);
    let deck = run.state.deck.clone();
    let hp = run.state.hp;
    let max_hp = run.state.max_hp;
    for (option, price) in [(0, 20), (1, 30), (2, 40)] {
        assert_eq!(row(&run, &RoomChoiceKind::EventOption { stage: 0, option }).price, Some(price));
    }
    let before_potions = run.state.potions.iter().filter(|slot| slot.is_some()).count();
    choose_at(&mut run, 0, option);
    assert_eq!(run.state.gold, 100 - price);
    assert_eq!((run.state.hp, run.state.max_hp, &run.state.deck), (hp, max_hp, &deck));
    assert_eq!(view(&run).kind, RoomViewKind::Rewards);
    assert!(event_frame(&run).complete);
    let potions: Vec<_> = view(&run).choices.iter().filter_map(|choice| match choice.kind {
        RoomChoiceKind::Potion(id) => Some((choice.id, id)),
        _ => None,
    }).collect();
    assert_eq!(potions.len(), amount);
    assert!(potions.iter().all(|(_, id)| potion_is_in_class_pool(character, *id)), "{character:?}: {potions:?}");
    for (choice, potion) in &potions {
        run.step(&GameAction::Choose(*choice)).unwrap();
        assert!(run.state.potions.iter().any(|slot| *slot == Some(*potion)));
    }
    assert_eq!(run.state.potions.iter().filter(|slot| slot.is_some()).count(), before_potions + amount);
    let proceed = row(&run, &RoomChoiceKind::Proceed).id;
    run.step(&GameAction::Choose(proceed)).unwrap();
    assert_eq!(view(&run).kind, RoomViewKind::Map);
}

#[test]
fn m28_beggar_pay_for_removal() {
    let mut run = enter_with(Character::Ironclad, EventId::Beggar, 91, |run| run.state.gold = 120);
    let deck = run.state.deck.clone();
    assert_eq!(row(&run, &RoomChoiceKind::EventOption { stage: 0, option: 0 }).price, Some(75));
    choose_at(&mut run, 0, 0);
    assert_eq!(run.state.gold, 45);
    assert_eq!(run.state.deck, deck, "the paid selection is still pending");
    assert!(run.state.pending.is_some());
    let selected = run.state.pending.as_ref().unwrap().selection.candidates[0].card.instance;
    select_instance(&mut run, selected);
    assert_eq!(run.state.deck.len(), deck.len() - 1);
    assert!(!run.state.deck.iter().any(|card| card.instance == selected));
    finish_done(&mut run);
}

#[test]
fn m28_beggar_softlock_001_empty_purge_group() {
    let mut run = enter_with(Character::Ironclad, EventId::Beggar, 92, |run| {
        run.state.gold = 100;
        set_deck(run, &[CardId::AscendersBane, CardId::CurseOfTheBell, CardId::Necronomicurse]);
    });
    let deck = run.state.deck.clone();
    choose_at(&mut run, 0, 0);
    assert_eq!(run.state.gold, 25);
    assert_eq!(run.state.deck, deck);
    assert!(run.state.pending.is_none());
    finish_done(&mut run);
}

#[test]
fn m28_beggar_leave() {
    let mut run = at_event(Character::Silent, 2, EventId::Beggar, 93);
    let state = run.state.clone();
    choose_at(&mut run, 0, 1);
    assert_eq!(view(&run).kind, RoomViewKind::Map);
    assert_eq!((run.state.gold, run.state.hp, run.state.max_hp, run.state.deck, run.state.relics),
        (state.gold, state.hp, state.max_hp, state.deck, state.relics));
}

#[test]
fn m28_cursed_tome_read() {
    let mut run = at_event(Character::Ironclad, 2, EventId::CursedTome, 94);
    let before = (run.state.hp, run.state.gold, run.state.max_hp, run.state.deck.clone(), run.state.relics.clone());
    choose_at(&mut run, 0, 0);
    assert_eq!(event_frame(&run).stage, 1);
    assert_eq!((run.state.hp, run.state.gold, run.state.max_hp, &run.state.deck, &run.state.relics),
        (before.0, before.1, before.2, &before.3, &before.4));
    assert_eq!(view(&run).choices.iter().map(|choice| choice.kind.clone()).collect::<Vec<_>>(),
        vec![RoomChoiceKind::EventOption { stage: 1, option: 0 }]);
}

#[test]
fn m28_cursed_tome_leave() {
    let mut run = at_event(Character::Silent, 2, EventId::CursedTome, 95);
    let state = run.state.clone();
    choose_at(&mut run, 0, 1);
    assert_eq!(view(&run).kind, RoomViewKind::Map);
    assert_eq!((run.state.hp, run.state.gold, run.state.max_hp, run.state.deck, run.state.relics),
        (state.hp, state.gold, state.max_hp, state.deck, state.relics));
}

#[test]
fn m28_cursed_tome_continue_1() {
    let mut run = at_event(Character::Ironclad, 2, EventId::CursedTome, 96);
    let hp = run.state.hp;
    choose_at(&mut run, 0, 0);
    choose_at(&mut run, 1, 0);
    assert_eq!((run.state.hp, event_frame(&run).stage), (hp - 1, 2));
}

#[test]
fn m28_cursed_tome_continue_2() {
    let mut run = at_event(Character::Ironclad, 2, EventId::CursedTome, 97);
    let hp = run.state.hp;
    choose_at(&mut run, 0, 0);
    choose_at(&mut run, 1, 0);
    choose_at(&mut run, 2, 0);
    assert_eq!((run.state.hp, event_frame(&run).stage), (hp - 3, 3));
}

#[test]
fn m28_cursed_tome_continue_3() {
    let mut run = at_event(Character::Ironclad, 2, EventId::CursedTome, 98);
    let hp = run.state.hp;
    choose_at(&mut run, 0, 0);
    choose_at(&mut run, 1, 0);
    choose_at(&mut run, 2, 0);
    choose_at(&mut run, 3, 0);
    assert_eq!((run.state.hp, event_frame(&run).stage), (hp - 6, 4));
    assert_eq!(view(&run).choices.iter().map(|choice| choice.kind.clone()).collect::<Vec<_>>(), vec![
        RoomChoiceKind::EventOption { stage: 4, option: 0 }, RoomChoiceKind::EventOption { stage: 4, option: 1 },
    ]);
}

#[test]
fn m28_cursed_tome_take_a0_a14() {
    for (seed, owned, expected) in [
        (99, vec![RelicId::Necronomicon, RelicId::Enchiridion, RelicId::NilrysCodex], RelicId::Circlet),
        (100, vec![RelicId::Necronomicon, RelicId::NilrysCodex], RelicId::Enchiridion),
    ] {
        let mut run = at_event(Character::Ironclad, 2, EventId::CursedTome, seed);
        run.state.hp = 60;
        run.state.relics.extend(owned);
        let start_relics = run.state.relics.len();
        tome_to_final(&mut run);
        choose_at(&mut run, 4, 0);
        assert_eq!(run.state.hp, 44, "three pages plus A0 Take cost 10");
        assert_eq!(run.state.relics.len(), start_relics, "the relic remains unclaimed on the reward screen");
        assert_eq!(view(&run).kind, RoomViewKind::Rewards);
        let reward = row(&run, &RoomChoiceKind::Relic(expected));
        if expected == RelicId::Circlet {
            run.step(&GameAction::Choose(reward.id)).unwrap();
            assert!(run.state.relics.contains(&expected));
            assert_eq!(run.state.relics.len(), start_relics + 1);
        } else {
            assert!(!run.state.relics.contains(&expected), "Proceed can skip the relic");
            assert_eq!(run.state.relics.len(), start_relics);
        }
        let proceed = row(&run, &RoomChoiceKind::Proceed).id;
        run.step(&GameAction::Choose(proceed)).unwrap();
        assert_eq!(view(&run).kind, RoomViewKind::Map);
    }
}

#[test]
fn m28_cursed_tome_stop() {
    let mut run = at_event(Character::Defect, 2, EventId::CursedTome, 101);
    run.state.hp = 60;
    let relics = run.state.relics.clone();
    tome_to_final(&mut run);
    choose_at(&mut run, 4, 1);
    assert_eq!(run.state.hp, 51, "1+2+3 page loss plus 3 Stop loss");
    assert_eq!(run.state.relics, relics);
    finish_done(&mut run);
}

#[test]
fn m28_drug_dealer_take_jax() {
    let mut run = at_event(Character::Silent, 2, EventId::DrugDealer, 102);
    let deck = run.state.deck.len();
    let gold = run.state.gold;
    let hp = run.state.hp;
    choose_at(&mut run, 0, 0);
    assert_eq!((count(&run, CardId::Jax), run.state.deck.len(), run.state.gold, run.state.hp),
        (1, deck + 1, gold, hp));
    finish_done(&mut run);
}

#[test]
fn m28_drug_dealer_transform_two() {
    let mut run = enter_with(Character::Ironclad, EventId::DrugDealer, 103, |run| set_deck(run, &[CardId::Strike, CardId::Defend, CardId::Bash]));
    let before = run.state.deck.clone();
    choose_at(&mut run, 0, 1);
    let candidates = run.state.pending.as_ref().unwrap().selection.candidates.clone();
    let first = candidates[0].card.instance;
    let second = candidates[1].card.instance;
    select_instance(&mut run, first);
    select_instance(&mut run, second);
    assert_eq!(run.state.deck.len(), before.len());
    assert!(!run.state.deck.iter().any(|card| card.instance == first || card.instance == second));
    finish_done(&mut run);
}

#[test]
fn m28_drug_dealer_transform_disabled_with_one_candidate() {
    let mut run = enter_with(Character::Ironclad, EventId::DrugDealer, 104, |run| set_deck(run, &[CardId::Strike]));
    let transform = row(&run, &RoomChoiceKind::EventOption { stage: 0, option: 1 });
    assert_eq!(transform.status, RoomChoiceStatus::Disabled);
    let before = run.state.clone();
    assert_eq!(run.step(&GameAction::Choose(transform.id)), Err(crate::run::RunError::InvalidAction));
    assert_eq!(run.state, before);
}

#[test]
fn m28_drug_dealer_mutagenic_strength() {
    for (seed, owned, expected) in [(105, false, RelicId::MutagenicStrength), (106, true, RelicId::Circlet)] {
        let mut run = at_event(Character::Watcher, 2, EventId::DrugDealer, seed);
        if owned { run.state.relics.push(RelicId::MutagenicStrength); }
        let old = run.state.relics.len();
        choose_at(&mut run, 0, 2);
        assert_eq!(run.state.relics.len(), old + 1);
        assert!(run.state.relics.contains(&expected));
        finish_done(&mut run);
    }
}

#[test]
fn m28_forgotten_altar_exchange_idol() {
    for (seed, already_bloody, expected) in [(107, false, RelicId::BloodyIdol), (108, true, RelicId::Circlet)] {
        let mut run = enter_with(Character::Ironclad, EventId::ForgottenAltar, seed, |run| {
            run.state.relics.push(RelicId::GoldenIdol);
            if already_bloody { run.state.relics.push(RelicId::BloodyIdol); }
        });
        let old = run.state.relics.len();
        choose_at(&mut run, 0, 0);
        assert!(run.state.relics.contains(&expected));
        if already_bloody {
            assert!(run.state.relics.contains(&RelicId::GoldenIdol), "Bloody Idol already owned: keep Golden Idol");
            assert_eq!(run.state.relics.len(), old + 1);
        } else {
            assert!(!run.state.relics.contains(&RelicId::GoldenIdol));
            assert_eq!(run.state.relics.len(), old);
        }
        finish_done(&mut run);
    }
}

#[test]
fn m28_forgotten_altar_offer_disabled_without_golden_idol() {
    let run = at_event(Character::Silent, 2, EventId::ForgottenAltar, 109);
    assert_eq!(event_row(&run, 0, 0).status, RoomChoiceStatus::Disabled);
}

#[test]
fn m28_forgotten_altar_sacrifice_a0_a14() {
    for (character, max_hp, damage) in [(Character::Ironclad, 80, 20), (Character::Silent, 70, 18),
        (Character::Defect, 75, 19), (Character::Watcher, 72, 18)] {
        let mut run = enter_with(character, EventId::ForgottenAltar, 110, |run| {
            run.state.max_hp = max_hp;
            run.state.hp = max_hp;
        });
        let gold = run.state.gold;
        choose_at(&mut run, 0, 1);
        assert_eq!((run.state.max_hp, run.state.hp, run.state.gold), (max_hp + 5, max_hp + 5 - damage, gold), "{character:?}");
        finish_done(&mut run);
    }
}

#[test]
fn m28_forgotten_altar_decay() {
    let mut run = at_event(Character::Defect, 2, EventId::ForgottenAltar, 111);
    let old = run.state.deck.len();
    choose_at(&mut run, 0, 2);
    assert_eq!((count(&run, CardId::Decay), run.state.deck.len()), (1, old + 1));
    finish_done(&mut run);
}

#[test]
fn m28_ghosts_accept_a0_a14() {
    for (character, max_hp, loss) in [(Character::Ironclad, 80, 40), (Character::Silent, 70, 35),
        (Character::Defect, 75, 38), (Character::Watcher, 72, 36)] {
        let mut run = enter_with(character, EventId::Ghosts, 112, |run| {
            run.state.max_hp = max_hp;
            run.state.hp = max_hp;
        });
        let old_deck = run.state.deck.len();
        choose_at(&mut run, 0, 0);
        assert_eq!((run.state.max_hp, run.state.hp), (max_hp - loss, max_hp - loss), "{character:?}");
        assert_eq!((count(&run, CardId::Apparition), run.state.deck.len()), (5, old_deck + 5));
        finish_done(&mut run);
    }
}

#[test]
fn m28_ghosts_decline() {
    let mut run = at_event(Character::Watcher, 2, EventId::Ghosts, 113);
    let state = run.state.clone();
    choose_at(&mut run, 0, 1);
    assert_eq!(view(&run).kind, RoomViewKind::Map);
    assert_eq!((run.state.hp, run.state.max_hp, run.state.gold, run.state.deck, run.state.relics),
        (state.hp, state.max_hp, state.gold, state.deck, state.relics));
}

#[test]
fn m28_the_nest_smash_and_grab_a0_a14() {
    let mut run = at_event(Character::Ironclad, 2, EventId::Nest, 114);
    let gold = run.state.gold;
    choose_at(&mut run, 0, 0);
    assert_eq!(run.state.gold, gold + 99);
    finish_done(&mut run);
}

#[test]
fn m28_the_nest_stay_in_line() {
    let mut run = enter_with(Character::Silent, EventId::Nest, 115, |run| run.state.hp = 50);
    let before = run.state.deck.len();
    choose_at(&mut run, 0, 1);
    assert_eq!((run.state.hp, count(&run, CardId::RitualDagger), run.state.deck.len()), (44, 1, before + 1));
    finish_done(&mut run);
}

#[test]
fn m28_the_library_read_card() {
    let mut run = at_event(Character::Defect, 2, EventId::Library, 116);
    let old_len = run.state.deck.len();
    let old_hp = run.state.hp;
    choose_at(&mut run, 0, 0);
    assert_eq!(event_frame(&run).stage, 1);
    let ids = event_frame(&run).captured.card_ids.clone();
    assert_eq!(ids.len(), 20);
    assert_eq!(ids.iter().collect::<std::collections::BTreeSet<_>>().len(), 20);
    let selected = 11_u8;
    let expected = ids[usize::from(selected)];
    let expected_count = count(&run, expected);
    let row = view(&run).choices.into_iter().find(|choice| matches!(choice.kind,
        RoomChoiceKind::EventCard { stage: 1, option, card: Some(id) } if option == selected && id == expected)).unwrap();
    run.step(&GameAction::Choose(row.id)).unwrap();
    assert_eq!((count(&run, expected), run.state.deck.len(), run.state.hp),
        (expected_count + 1, old_len + 1, old_hp));
    assert!(run.state.deck.iter().any(|card| card.id == expected));
    assert_done(&run);
    finish_done(&mut run);
}

#[test]
fn m28_the_library_sleep_a0_a14() {
    for (character, max_hp, heal) in [(Character::Ironclad, 80, 26), (Character::Silent, 70, 23),
        (Character::Defect, 75, 25), (Character::Watcher, 72, 24)] {
        let mut run = enter_with(character, EventId::Library, 117, |run| {
            run.state.max_hp = max_hp;
            run.state.hp = 1;
        });
        let deck = run.state.deck.clone();
        let gold = run.state.gold;
        let relics = run.state.relics.clone();
        choose_at(&mut run, 0, 1);
        assert_eq!((run.state.hp, run.state.max_hp, run.state.gold), (1 + heal, max_hp, gold), "{character:?}");
        assert_eq!((&run.state.deck, &run.state.relics), (&deck, &relics));
        finish_done(&mut run);
    }
}

#[test]
fn m28_the_mausoleum_open_a0_a14() {
    let mut saw_writhe = false;
    let mut saw_clean = false;
    for seed in 0..100_u8 {
        let mut run = at_event(Character::Silent, 2, EventId::Mausoleum, seed);
        let before_writhe = count(&run, CardId::Writhe);
        let relics = run.state.relics.len();
        choose_at(&mut run, 0, 0);
        let writhe_delta = count(&run, CardId::Writhe) - before_writhe;
        assert!(writhe_delta <= 1);
        assert_eq!(run.state.relics.len(), relics + 1, "one screenless relic for seed {seed}");
        saw_writhe |= writhe_delta == 1;
        saw_clean |= writhe_delta == 0;
        finish_done(&mut run);
    }
    assert!(saw_writhe && saw_clean, "A0 seeds reach Writhe and relic-only arms: {saw_writhe}/{saw_clean}");
}

#[test]
fn m28_the_mausoleum_leave() {
    let mut run = at_event(Character::Watcher, 2, EventId::Mausoleum, 118);
    let state = run.state.clone();
    choose_at(&mut run, 0, 1);
    assert_eq!(view(&run).kind, RoomViewKind::Map);
    assert_eq!((run.state.hp, run.state.max_hp, run.state.gold, run.state.deck, run.state.relics),
        (state.hp, state.max_hp, state.gold, state.deck, state.relics));
}

#[test]
fn m28_vampires_pay_hp() {
    for (character, max_hp, loss) in [(Character::Ironclad, 80, 24), (Character::Silent, 70, 21),
        (Character::Defect, 75, 23), (Character::Watcher, 72, 22)] {
        let mut run = enter_with(character, EventId::Vampires, 119, |run| {
            run.state.max_hp = max_hp;
            run.state.hp = max_hp;
        });
        let old: Vec<_> = run.state.deck.iter().filter(|card| !crate::content::card_tags(card.id).contains(&CardTag::StarterStrike))
            .map(|card| card.id).collect();
        let gold = run.state.gold;
        choose_at(&mut run, 0, 0);
        let mut expected = old;
        expected.extend([CardId::Bite; 5]);
        assert_eq!((run.state.max_hp, run.state.hp, run.state.gold), (max_hp - loss, max_hp - loss, gold), "{character:?}");
        assert_eq!(run.state.deck.iter().map(|card| card.id).collect::<Vec<_>>(), expected, "{character:?}");
        finish_done(&mut run);
    }
}

#[test]
fn m28_vampires_give_blood_vial() {
    let mut run = enter_with(Character::Silent, EventId::Vampires, 120, |run| run.state.relics.push(RelicId::BloodVial));
    assert!(view(&run).choices.iter().any(|choice| choice.kind == RoomChoiceKind::EventOption { stage: 0, option: 1 }));
    let hp = run.state.hp;
    let max_hp = run.state.max_hp;
    let old: Vec<_> = run.state.deck.iter().filter(|card| !crate::content::card_tags(card.id).contains(&CardTag::StarterStrike))
        .map(|card| card.id).collect();
    choose_at(&mut run, 0, 1);
    let mut expected = old;
    expected.extend([CardId::Bite; 5]);
    assert!(!run.state.relics.contains(&RelicId::BloodVial));
    assert_eq!((run.state.hp, run.state.max_hp), (hp, max_hp));
    assert_eq!(run.state.deck.iter().map(|card| card.id).collect::<Vec<_>>(), expected);
    finish_done(&mut run);
}

#[test]
fn m28_vampires_give_blood_vial_option_requires_relic() {
    let run = at_event(Character::Ironclad, 2, EventId::Vampires, 121);
    assert!(!view(&run).choices.iter().any(|choice| choice.kind == RoomChoiceKind::EventOption { stage: 0, option: 1 }));
}

#[test]
fn m28_vampires_refuse() {
    let mut run = at_event(Character::Defect, 2, EventId::Vampires, 122);
    let state = run.state.clone();
    choose_at(&mut run, 0, 2);
    assert_eq!(view(&run).kind, RoomViewKind::Map);
    assert_eq!((run.state.hp, run.state.max_hp, run.state.gold, run.state.deck, run.state.relics),
        (state.hp, state.max_hp, state.gold, state.deck, state.relics));
}

#[test]
fn m28_we_meet_again_give_potion() {
    let mut run = enter_with(Character::Ironclad, EventId::WeMeetAgain, 123, |run| {
        run.state.potions = vec![None, Some(PotionId::FirePotion), None];
    });
    let gold = run.state.gold;
    let hp = run.state.hp;
    let max_hp = run.state.max_hp;
    let deck = run.state.deck.clone();
    let relics = run.state.relics.len();
    assert_eq!(event_frame(&run).captured.values[0], 2, "the only held potion slot is preselected");
    choose_at(&mut run, 0, 0);
    assert_eq!(run.state.potions, vec![None, None, None]);
    assert_eq!((run.state.gold, run.state.hp, run.state.max_hp, run.state.deck.len()), (gold, hp, max_hp, deck.len()));
    assert_eq!(run.state.relics.len(), relics + 1);
    finish_done(&mut run);
}

#[test]
fn m28_we_meet_again_give_gold() {
    for (seed, gold) in [(124, 49), (125, 50), (126, 120), (127, 300)] {
        let mut run = enter_with(Character::Silent, EventId::WeMeetAgain, seed, |run| run.state.gold = gold);
        let gold_row = event_row(&run, 0, 1);
        if gold < 50 {
            assert_eq!(gold_row.status, RoomChoiceStatus::Disabled);
            continue;
        }
        let amount = event_frame(&run).captured.values[1];
        assert!((50..=gold.min(150)).contains(&amount), "G={gold}: {amount}");
        let relics = run.state.relics.len();
        choose_at(&mut run, 0, 1);
        assert_eq!(run.state.gold, gold - amount);
        assert_eq!(run.state.relics.len(), relics + 1);
        finish_done(&mut run);
    }
}

#[test]
fn m28_we_meet_again_give_card() {
    let mut run = enter_with(Character::Defect, EventId::WeMeetAgain, 128, |run| set_deck(run, &[CardId::IronWave, CardId::StrikeBlue]));
    let sacrificed = run.state.deck.iter().find(|card| card.id == CardId::IronWave).unwrap().instance;
    assert_eq!(event_frame(&run).captured.cards, vec![sacrificed]);
    let relics = run.state.relics.len();
    let gold = run.state.gold;
    let hp = run.state.hp;
    choose_at(&mut run, 0, 2);
    assert!(!run.state.deck.iter().any(|card| card.instance == sacrificed));
    assert_eq!(run.state.deck.iter().map(|card| card.id).collect::<Vec<_>>(), vec![CardId::StrikeBlue]);
    assert_eq!((run.state.relics.len(), run.state.gold, run.state.hp), (relics + 1, gold, hp));
    finish_done(&mut run);
}

#[test]
fn m28_we_meet_again_disabled_without_gift() {
    let run = enter_with(Character::Ironclad, EventId::WeMeetAgain, 129, |run| run.state.gold = 100);
    assert_eq!(event_row(&run, 0, 0).status, RoomChoiceStatus::Disabled, "no held potion");
    assert_eq!(event_row(&run, 0, 1).status, RoomChoiceStatus::Offered);
    assert_eq!(event_row(&run, 0, 2).status, RoomChoiceStatus::Disabled, "no eligible card");

    let run = enter_with(Character::Ironclad, EventId::WeMeetAgain, 138, |run| run.state.gold = 49);
    assert_eq!(event_row(&run, 0, 0).status, RoomChoiceStatus::Disabled, "no held potion");
    assert_eq!(event_row(&run, 0, 1).status, RoomChoiceStatus::Disabled, "gold below 50");
    assert_eq!(event_row(&run, 0, 2).status, RoomChoiceStatus::Disabled, "no eligible card");
}

#[test]
fn m28_we_meet_again_attack_leave() {
    let mut run = at_event(Character::Watcher, 2, EventId::WeMeetAgain, 130);
    let state = run.state.clone();
    choose_at(&mut run, 0, 3);
    assert_eq!(view(&run).kind, RoomViewKind::Map);
    assert_eq!((run.state.gold, run.state.hp, run.state.max_hp, run.state.deck, run.state.relics, run.state.potions),
        (state.gold, state.hp, state.max_hp, state.deck, state.relics, state.potions));
}

#[test]
fn m28_woman_in_blue_buy_one_potion() {
    for (seed, character) in [(131, Character::Ironclad), (132, Character::Silent), (133, Character::Defect), (134, Character::Watcher)] {
        buy_woman_potions(character, 0, 1, 20, seed);
    }
}

#[test]
fn m28_woman_in_blue_buy_two_potions() {
    buy_woman_potions(Character::Silent, 1, 2, 30, 135);
}

#[test]
fn m28_woman_in_blue_buy_three_potions() {
    buy_woman_potions(Character::Defect, 2, 3, 40, 136);
}

#[test]
fn m28_woman_in_blue_leave() {
    let mut run = enter_with(Character::Watcher, EventId::WomanInBlue, 137, |run| run.state.gold = 50);
    let state = run.state.clone();
    choose_at(&mut run, 0, 3);
    assert_eq!(view(&run).kind, RoomViewKind::Map);
    assert_eq!((run.state.gold, run.state.hp, run.state.max_hp, run.state.deck, run.state.relics, run.state.potions),
        (state.gold, state.hp, state.max_hp, state.deck, state.relics, state.potions));
}

/// Check C item 3: Offer Idol puts Bloody Idol in Golden Idol's relic slot (`instantObtain`, equip flag false).
#[test]
fn m28_forgotten_altar_bloody_idol_takes_golden_idol_slot() {
    let mut run = at_event(Character::Defect, 2, EventId::ForgottenAltar, 95);
    run.state.relics.extend([RelicId::GoldenIdol, RelicId::Anchor, RelicId::Vajra]);
    let slot = run.state.relics.iter().position(|r| *r == RelicId::GoldenIdol).unwrap();
    let mut expected = run.state.relics.clone();
    expected[slot] = RelicId::BloodyIdol;
    let id = row(&run, &RoomChoiceKind::EventOption { stage: 0, option: 0 }).id;
    run.step(&GameAction::Choose(id)).unwrap();
    assert_eq!(run.state.relics, expected);
}

/// PM ruling (bottled cards): Drug Dealer's transform grid keeps a bottled card; transforming it leaves no bottle
/// mapping and the next combat starts without a fault.
#[test]
fn m28_drug_dealer_transforms_a_bottled_card() {
    let mut run = at_event(Character::Ironclad, 2, EventId::DrugDealer, 96);
    let bash = run.state.deck.iter().find(|c| c.id == CardId::Bash).unwrap().instance;
    run.state.relics.push(RelicId::BottledFlame);
    run.state.relic_cards.insert(RelicId::BottledFlame, bash);
    let id = row(&run, &RoomChoiceKind::EventOption { stage: 0, option: 1 }).id;
    run.step(&GameAction::Choose(id)).unwrap();
    let pending = run.state.pending.as_ref().expect("transform grid");
    let pick = |run: &Run, instance: u64| run.state.pending.as_ref().unwrap().selection.candidates.iter().find(|c| c.card.instance == instance).map(|c| c.id);
    assert!(pending.selection.candidates.iter().any(|c| c.card.instance == bash), "the bottled card is offered");
    let other = pending.selection.candidates.iter().find(|c| c.card.instance != bash).unwrap().card.instance;
    for instance in [bash, other] { let id = pick(&run, instance).unwrap(); run.step(&GameAction::Select(id)).unwrap(); }
    assert!(!run.state.deck.iter().any(|c| c.instance == bash) && run.state.relic_cards.is_empty(), "transformed away, no mapping");
    let mut run2 = run.clone();
    leave(&mut run2);
    let mut fight = at_event(Character::Ironclad, 2, EventId::GoldenShrine, 97);
    fight.state.deck = run.state.deck.clone();
    fight.state.relics = run.state.relics.clone();
    fight.state.relic_cards = run.state.relic_cards.clone();
    event_frame::choose_constructed(&mut fight, vec![Outcome::Combat { encounter: EncounterId::MaskedBandits, rewards: None, next_stage: 0, elite: false }]).unwrap();
    assert!(fight.state.fault.is_none() && fight.state.combat.is_some(), "the next combat starts without a fault");
}

pub(super) fn option_row(run: &mut Run, stage: u8, option: u8) {
    let id = row(run, &RoomChoiceKind::EventOption { stage, option }).id;
    run.step(&GameAction::Choose(id)).unwrap();
}
fn relic_tiers(run: &Run) -> Vec<Option<crate::content::RelicTier>> {
    view(run).choices.iter().filter_map(|c| match c.kind { RoomChoiceKind::Relic(r) => Some(crate::content::Catalog::standard().relic_tier(r)), _ => None }).collect()
}

/// The diagnostic Run placed on a row-7 node whose row-8 child is the event room: the first row the Colosseum gate (E10) admits.
pub(super) fn colosseum(character: Character, seed: u8) -> Run {
    let (mut run, node) = colosseum_before(character, seed);
    run.step(&GameAction::Move(node)).unwrap();
    assert_eq!(view(&run).kind, RoomViewKind::Event(EventId::Colosseum));
    run
}

fn colosseum_before(character: Character, seed: u8) -> (Run, crate::run::NodeId) {
    let (mut run, _) = before_event(character, 2, EventId::Colosseum, seed);
    let map = run.state.map.as_mut().unwrap();
    let from = map.nodes.iter().find(|n| n.y == 7 && !n.edges.is_empty()).unwrap().clone();
    let node = from.edges[0];
    map.nodes.iter_mut().find(|n| n.id == node).unwrap().kind = MapNodeKind::Event;
    map.current = Some(from.id);
    run.state.floor = from.floor;
    (run, node)
}

/// The first Slavers fight keeps the EventRoom potion roll on a Smoke Bomb escape (source-check-event-escape-v1, N3).
#[test]
fn m28_event_combat_colosseum_first_fight_smoke_bomb() {
    use crate::content::EnemyId;
    let mut run = colosseum(Character::Silent, 104);
    run.state.potions[0] = Some(PotionId::SmokeBomb);
    let (gold, potion_offset, rng) = (run.state.gold, run.state.potion_offset, run.state.rng.clone());
    option_row(&mut run, 0, 0);
    let enemies: Vec<_> = run.state.combat.as_ref().unwrap().public_full().unwrap().enemies
        .iter().map(|enemy| enemy.enemy_id).collect();
    assert_eq!(enemies, vec![EnemyId::SlaverBlue, EnemyId::SlaverRed]);

    run.choose_combat(crate::combat::CombatAction::UsePotion { slot: 0, target: None }).unwrap();
    assert!(run.state.combat.is_none() && run.state.fault.is_none());
    assert_eq!(run.state.potions[0], None);
    assert_eq!((run.state.potion_offset - potion_offset).abs(), 10, "N3's suppressed event potion roll advances its offset");
    assert_ne!(run.state.rng, rng, "N3 draws on the event potion stream");
    assert_eq!(run.state.gold, gold, "the first fight pays no rewards");
    let rows = view(&run);
    assert_eq!(rows.kind, RoomViewKind::Event(EventId::Colosseum));
    assert_eq!(rows.choices.iter().map(|choice| choice.kind.clone()).collect::<Vec<_>>(), vec![
        RoomChoiceKind::EventOption { stage: 1, option: 0 },
        RoomChoiceKind::EventOption { stage: 1, option: 1 },
    ]);
}

/// Colosseum (acts23 research :187; checks m28e B item 12, C item 7, escape N3): fight 1 (Blue, Red Slaver) has no
/// reward screen and reopens the event at [Escape, Fight Nobs] after the EventRoom potion roll; fight 2 (Taskmaster,
/// Gremlin Nob, eliteTrigger) takes a Rare then an Uncommon relic row at fight start and pays 100 gold with an ordinary
/// card offer; Escape leaves. A Smoke Bomb escape from fight 2 shows Proceed only, to the map.
#[test]
fn m28_event_combat_colosseum_both_fights() {
    use crate::content::{EnemyId, RelicTier};
    for (seed, character, smoke) in [(100, Character::Ironclad, false), (101, Character::Watcher, false), (102, Character::Silent, true)] {
        let mut run = colosseum(character, seed);
        let (gold, offset) = (run.state.gold, run.state.potion_offset);
        assert_eq!(view(&run).choices.len(), 1, "one start row (intro and Continue collapse under D1)");
        option_row(&mut run, 0, 0);
        let enemies: Vec<_> = run.state.combat.as_ref().unwrap().public_full().unwrap().enemies.iter().map(|e| e.enemy_id).collect();
        assert_eq!(enemies, vec![EnemyId::SlaverBlue, EnemyId::SlaverRed]);
        super::m28_world::win(&mut run);
        assert_eq!((view(&run).kind, run.state.gold), (RoomViewKind::Event(EventId::Colosseum), gold), "no rewards after fight 1");
        assert_eq!((run.state.potion_offset - offset).abs(), 10, "N3 potion roll");
        let options: Vec<_> = view(&run).choices.iter().map(|c| c.kind.clone()).collect();
        assert_eq!(options, vec![RoomChoiceKind::EventOption { stage: 1, option: 0 }, RoomChoiceKind::EventOption { stage: 1, option: 1 }]);
        if smoke { run.state.potions[0] = Some(PotionId::SmokeBomb); }
        let pools = run.state.relic_pools.clone();
        option_row(&mut run, 1, 1);
        assert_ne!(run.state.relic_pools, pools, "relic rows taken at fight start");
        let enemies: Vec<_> = run.state.combat.as_ref().unwrap().public_full().unwrap().enemies.iter().map(|e| e.enemy_id).collect();
        assert_eq!(enemies, vec![EnemyId::Taskmaster, EnemyId::GremlinNob]);
        assert!(rooms::standard::event_frame::combat_elite(&run.state), "eliteTrigger");
        if smoke {
            run.choose_combat(crate::combat::CombatAction::UsePotion { slot: 0, target: None }).unwrap();
            assert_eq!(view(&run).choices.iter().map(|c| c.kind.clone()).collect::<Vec<_>>(), vec![RoomChoiceKind::Proceed]);
            option_proceed(&mut run);
            assert_eq!((view(&run).kind, run.state.gold), (RoomViewKind::Map, gold), "smoked: nothing granted");
            continue;
        }
        super::m28_world::win(&mut run);
        assert_eq!(relic_tiers(&run), vec![Some(RelicTier::Rare), Some(RelicTier::Uncommon)]);
        assert!(view(&run).choices.iter().any(|c| c.kind == RoomChoiceKind::Gold(100)));
        assert_eq!(view(&run).choices.iter().filter(|c| c.kind == RoomChoiceKind::CardReward).count(), 1);
        let id = row(&run, &RoomChoiceKind::Gold(100)).id;
        run.step(&GameAction::Choose(id)).unwrap();
        assert_eq!(run.state.gold, gold + 100);
        option_proceed(&mut run);
        assert_eq!(view(&run).kind, RoomViewKind::Map);
    }
    // Escape after fight 1 leaves with nothing more.
    let mut run = colosseum(Character::Defect, 103);
    option_row(&mut run, 0, 0);
    super::m28_world::win(&mut run);
    let gold = run.state.gold;
    option_row(&mut run, 1, 0);
    assert_eq!((view(&run).kind, run.state.gold), (RoomViewKind::Map, gold));
}

fn option_proceed(run: &mut Run) {
    let id = row(run, &RoomChoiceKind::Proceed).id;
    run.step(&GameAction::Choose(id)).unwrap();
}

/// Masked Bandits (acts23 research :192; checks m28e A, B items 13a/13b): Pay loses all gold (zero included); Fight
/// is Pointy, Romeo, Bear in that slot order, 25–35 gold drawn at the fight choice and Red Mask (Circlet if owned) as
/// reward rows plus the ordinary EventRoom card offer; a Smoke Bomb escape shows Proceed only, to the map.
#[test]
fn m28_event_combat_masked_bandits_returns() {
    use crate::content::EnemyId;
    for gold in [0, 99] {
        let mut run = at_event(Character::Silent, 2, EventId::MaskedBandits, 110);
        run.state.gold = gold;
        option_row(&mut run, 0, 0);
        assert_eq!(run.state.gold, 0);
        leave(&mut run);
    }
    let mut seen = std::collections::BTreeSet::new();
    for seed in 111..=140u8 {
        let owned = seed % 2 == 0;
        let mut run = at_event(Character::Ironclad, 2, EventId::MaskedBandits, seed);
        if owned { run.state.relics.push(RelicId::RedMask); }
        let gold = run.state.gold;
        option_row(&mut run, 0, 1);
        let enemies: Vec<_> = run.state.combat.as_ref().unwrap().public_full().unwrap().enemies.iter().map(|e| (e.enemy_id, e.logical_slot)).collect();
        assert_eq!(enemies, vec![(EnemyId::BanditPointy, 0), (EnemyId::BanditLeader, 1), (EnemyId::BanditBear, 2)]);
        assert!(!rooms::standard::event_frame::combat_elite(&run.state));
        super::m28_world::win(&mut run);
        let kinds: Vec<_> = view(&run).choices.iter().map(|c| c.kind.clone()).collect();
        let bonus = kinds.iter().find_map(|k| match k { RoomChoiceKind::Gold(n) => Some(*n), _ => None }).expect("event gold row");
        assert!((25..=35).contains(&bonus), "{bonus}");
        seen.insert(bonus);
        assert!(kinds.contains(&RoomChoiceKind::Relic(if owned { RelicId::Circlet } else { RelicId::RedMask })), "{kinds:?}");
        assert_eq!(kinds.iter().filter(|k| **k == RoomChoiceKind::CardReward).count(), 1);
        let id = row(&run, &RoomChoiceKind::Gold(bonus)).id;
        run.step(&GameAction::Choose(id)).unwrap();
        assert_eq!(run.state.gold, gold + bonus);
        option_proceed(&mut run);
        assert_eq!(view(&run).kind, RoomViewKind::Map);
    }
    assert!(seen.len() >= 5, "the gold draw varies: {seen:?}");
    let mut run = at_event(Character::Defect, 2, EventId::MaskedBandits, 141);
    run.state.potions[0] = Some(PotionId::SmokeBomb);
    let gold = run.state.gold;
    option_row(&mut run, 0, 1);
    run.choose_combat(crate::combat::CombatAction::UsePotion { slot: 0, target: None }).unwrap();
    assert_eq!(view(&run).choices.iter().map(|c| c.kind.clone()).collect::<Vec<_>>(), vec![RoomChoiceKind::Proceed]);
    option_proceed(&mut run);
    assert_eq!((view(&run).kind, run.state.gold, run.state.relics.contains(&RelicId::RedMask)), (RoomViewKind::Map, gold, false));
}

// ---- Review v1 fixes (m28e-review-v1 I1, I2, M2, M4) ----

/// I1 (check A items 6-7): The Library calls the plain `getCard(rarity)`, which reads the active character's
/// pools; Prismatic Shard's every-colour pools belong to the card-reward builder only.
#[test]
fn m28_the_library_board_ignores_prismatic_shard() {
    for (seed, character) in [(0, Character::Ironclad), (1, Character::Silent), (2, Character::Defect), (3, Character::Watcher)] {
        let mut run = enter_with(character, EventId::Library, seed, |run| run.state.relics.push(RelicId::PrismaticShard));
        choose_at(&mut run, 0, 0);
        let ids = event_frame(&run).captured.card_ids.clone();
        assert_eq!(ids.len(), 20);
        let pool = crate::content::pools::cards(crate::content::pools::CardPool::Ordinary { character, rarity: None, card_type: None }).unwrap();
        let off: Vec<_> = ids.iter().filter(|id| !pool.contains(id)).collect();
        assert!(off.is_empty(), "{character:?} with Prismatic Shard: off-colour {off:?}");
    }
}

/// I2 (D2): We Meet Again's buttons show the potion, gold amount and card its constructor preselected, so the
/// rows reveal them: the potion's identity, the gold as the row price, and the card's identity and instance.
#[test]
fn m28_we_meet_again_rows_show_the_preselected_gifts() {
    let mut run = enter_with(Character::Ironclad, EventId::WeMeetAgain, 21, |run| {
        run.state.potions = vec![Some(PotionId::FirePotion), Some(PotionId::FruitJuice), None];
        run.state.gold = 140;
        run.state.deck.push(CardInstance { id: CardId::Inflame, instance: 9001, upgrades: 1, misc: 0 });
        run.state.deck.push(CardInstance { id: CardId::IronWave, instance: 9002, upgrades: 0, misc: 0 });
    });
    let captured = event_frame(&run).captured.clone();
    let potion = run.state.potions[(captured.values[0] - 1) as usize];
    let card = run.state.deck.iter().find(|c| c.instance == captured.cards[0]).unwrap().clone();
    assert!(potion.is_some() && (50..=140).contains(&captured.values[1]));
    let rows = view(&run).choices;
    assert_eq!(rows[0].kind, RoomChoiceKind::EventPotion { stage: 0, option: 0, potion });
    assert_eq!((rows[1].kind.clone(), rows[1].price), (RoomChoiceKind::EventOption { stage: 0, option: 1 }, Some(captured.values[1])));
    assert_eq!(rows[2].kind, RoomChoiceKind::EventCard { stage: 0, option: 2, card: Some(card.id) });
    assert_eq!((rows[3].kind.clone(), rows[3].price), (RoomChoiceKind::EventOption { stage: 0, option: 3 }, None));
    assert!(rows.iter().all(|r| r.status == RoomChoiceStatus::Offered), "{rows:?}");
    let candidates = run.decision().unwrap().candidates;
    let object = candidates.iter().find(|c| c.action == GameAction::Choose(rows[2].id)).unwrap().object.clone().unwrap();
    assert_eq!((object.instance, object.id, object.upgrades), (card.instance, card.id, card.upgrades), "the card row names its instance");
    assert!(candidates.iter().filter(|c| c.action != GameAction::Choose(rows[2].id)).all(|c| c.object.is_none()));
    let gold = run.state.gold;
    choose_at(&mut run, 0, 1);
    assert_eq!(run.state.gold, gold - captured.values[1], "the price is paid once, by the gift");

    // Nothing to give: the rows reveal nothing and stay disabled.
    let run = enter_with(Character::Ironclad, EventId::WeMeetAgain, 129, |run| { run.state.gold = 49; set_deck(run, &[CardId::Strike]); });
    let rows = view(&run).choices;
    assert_eq!(rows[0].kind, RoomChoiceKind::EventPotion { stage: 0, option: 0, potion: None });
    assert_eq!((rows[1].kind.clone(), rows[1].price), (RoomChoiceKind::EventOption { stage: 0, option: 1 }, None));
    assert_eq!(rows[2].kind, RoomChoiceKind::EventCard { stage: 0, option: 2, card: None });
    assert!(rows[..3].iter().all(|r| r.status == RoomChoiceStatus::Disabled), "{rows:?}");
}

/// M2: above A0 the Colosseum's first fight has no enemy facts, so the event would open with no legal row; the
/// Move is rejected up front as the typed interim (the global ascension gate), atomically. Masked Bandits keeps Pay.
/// M29c lands the Colosseum's species: the witness moves above its first fight's ceiling (A17, the M32 reason).
#[test]
fn m28_colosseum_above_a0_is_rejected_at_the_move() {
    let (mut run, node) = colosseum_before(Character::Ironclad, 7);
    run.state.ascension = 17;
    let before = run.clone();
    let result = run.step(&GameAction::Move(node));
    assert!(matches!(result, Err(crate::run::RunError::Unsupported(_))), "{result:?}");
    assert!(run == before && run.state.fault.is_none(), "rejected atomically, no fault");

    // M29d lands Masked Bandits (ceiling 16): the fight is gated above the ceiling.
    let (mut run, node) = before_event(Character::Ironclad, 2, EventId::MaskedBandits, 7);
    run.state.ascension = 17;
    run.step(&GameAction::Move(node)).unwrap();
    assert_eq!(event_row(&run, 0, 0).status, RoomChoiceStatus::Offered, "Pay stays legal");
    assert_eq!(event_row(&run, 0, 1).status, RoomChoiceStatus::Disabled, "the fight is gated");
}

/// M4: amounts captured at construction ignore a max-HP change on the page (Fruit Juice, the D1 window).
#[test]
fn m28_captured_amounts_ignore_fruit_juice_on_the_page() {
    // (event, option, max HP and HP after the choice) from M=80, HP=30; the juice makes M=85, HP=35.
    for (event, option, after) in [(EventId::Ghosts, 0, (45, 35)), (EventId::Vampires, 0, (61, 35)),
        (EventId::Library, 1, (85, 61)), (EventId::ForgottenAltar, 1, (90, 20))] {
        let mut run = enter_with(Character::Ironclad, event, 9, |run| {
            run.state.max_hp = 80; run.state.hp = 30;
            run.state.potions[0] = Some(PotionId::FruitJuice);
        });
        run.step(&GameAction::UsePotion { slot: 0, target: None }).unwrap();
        assert_eq!((run.state.max_hp, run.state.hp), (85, 35), "{event:?}");
        choose_at(&mut run, 0, option);
        assert_eq!((run.state.max_hp, run.state.hp), after, "{event:?}: the amount captured from M=80");
    }
}

/// M4: Vampires' five Bites pass through the obtain hooks, so Molten Egg upgrades every one.
#[test]
fn m28_vampires_bites_with_molten_egg() {
    let mut run = enter_with(Character::Ironclad, EventId::Vampires, 11, |run| run.state.relics.push(RelicId::MoltenEgg));
    choose_at(&mut run, 0, 0);
    let bites: Vec<_> = run.state.deck.iter().filter(|c| c.id == CardId::Bite).map(|c| c.upgrades).collect();
    assert_eq!(bites, vec![1; 5]);
}

/// M4 (check C4): The Mausoleum obtains Writhe before the relic, so only an Omamori already held negates it; an
/// Omamori obtained by this choice arrives after the curse.
#[test]
fn m28_the_mausoleum_writhe_with_omamori_held_or_obtained() {
    let mut run = at_event(Character::Ironclad, 2, EventId::Mausoleum, 12);
    run.state.ascension = 15;
    run.state.relics.push(RelicId::Omamori);
    run.state.relic_counters.insert(RelicId::Omamori, 2);
    let deck = run.state.deck.len();
    choose_at(&mut run, 0, 0);
    assert_eq!((run.state.deck.len(), run.state.relic_counters.get(&RelicId::Omamori)), (deck, Some(&1)), "held: Writhe negated");

    let mut run = at_event(Character::Ironclad, 2, EventId::Mausoleum, 12);
    run.state.ascension = 15;
    let pools = run.state.relic_pools.as_mut().unwrap();
    for tier in [&mut pools.common, &mut pools.uncommon, &mut pools.rare] { tier.retain(|r| *r != RelicId::Omamori); tier.insert(0, RelicId::Omamori); }
    let writhe = count(&run, CardId::Writhe);
    choose_at(&mut run, 0, 0);
    assert!(run.state.relics.contains(&RelicId::Omamori), "the relic roll obtained Omamori");
    assert_eq!(count(&run, CardId::Writhe), writhe + 1, "obtained in this choice: Writhe is kept");
    assert_eq!(run.state.relic_counters.get(&RelicId::Omamori).copied().unwrap_or(2), 2, "no charge spent");
}

/// M4 (PM ruling, Drug Dealer bottled cards): a bottled card counts toward Transform's two (`getPurgeableCards`
/// does not filter bottles) and is offered in the grid.
#[test]
fn m28_drug_dealer_bottled_card_counts_toward_two() {
    let mut run = enter_with(Character::Ironclad, EventId::DrugDealer, 13, |run| {
        set_deck(run, &[CardId::Bash, CardId::Strike]);
        let bash = run.state.deck[0].instance;
        run.state.relics.push(RelicId::BottledFlame);
        run.state.relic_cards.insert(RelicId::BottledFlame, bash);
    });
    let bash = run.state.deck[0].instance;
    assert_eq!(event_row(&run, 0, 1).status, RoomChoiceStatus::Offered);
    choose_at(&mut run, 0, 1);
    let grid: Vec<_> = run.state.pending.as_ref().unwrap().selection.candidates.iter().map(|c| c.card.instance).collect();
    assert!(grid.contains(&bash) && grid.len() == 2, "{grid:?}");
}
