use super::super::*;
use crate::{
    combat::{rng::Partition, model::OwnedCard},
    content::{CardId, Character, EncounterId, PotionId},
    items::CombatItemContext,
    run::RunRng,
};
use std::collections::{BTreeMap, BTreeSet};

pub(super) const CHARACTERS: [Character; 4] = [
    Character::Ironclad, Character::Silent, Character::Defect, Character::Watcher,
];

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct CardCountChange {
    pub id: CardId,
    pub upgrades: u32,
    pub count: i32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct EnemyDelta {
    pub id: String,
    pub hp: i32,
    pub block: i32,
    pub powers: Vec<(String, i32)>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct PublicDelta {
    pub player_hp: i32,
    pub player_block: i32,
    pub energy: i32,
    pub player_powers: Vec<(String, i32)>,
    pub focus: i32,
    pub enemies: Vec<EnemyDelta>,
    pub stance: (Stance, Stance),
    pub hand: Vec<CardCountChange>,
    pub draw: Vec<CardCountChange>,
    pub discard: Vec<CardCountChange>,
    pub exhaust: Vec<CardCountChange>,
    pub created_cards: Vec<(CardId, u32)>,
    pub playable: bool,
    pub fault: Option<String>,
    pub orbs: Option<Vec<Option<(OrbKind, Option<i32>)>>>,
}

#[derive(Clone)]
struct Snapshot {
    hp: i32,
    block: i32,
    energy: i32,
    powers: Vec<(String, i32)>,
    focus: i32,
    enemies: Vec<(String, i32, i32, Vec<(String, i32)>)>,
    stance: Stance,
    orbs: Vec<Option<(OrbKind, Option<i32>)>>,
    hand: Vec<CardCountChange>,
    draw: Vec<CardCountChange>,
    discard: Vec<CardCountChange>,
    exhaust: Vec<CardCountChange>,
    local_instances: BTreeSet<u64>,
}

fn changes(before: &[(String, i32)], after: &[(String, i32)]) -> Vec<(String, i32)> {
    let mut all = BTreeMap::<String, i32>::new();
    for (id, amount) in before { *all.entry(id.clone()).or_default() -= amount; }
    for (id, amount) in after { *all.entry(id.clone()).or_default() += amount; }
    all.into_iter().filter(|(_, amount)| *amount != 0).collect()
}

fn power_values(powers: &[FullPowerEntry]) -> Vec<(String, i32)> {
    let mut values = powers.iter().map(|power| (power.id.0.to_owned(), power.state.amount)).collect::<Vec<_>>();
    values.sort();
    values
}

fn zone(pile: &[OwnedCard]) -> Vec<CardCountChange> {
    let mut counts = BTreeMap::<(CardId, u32), i32>::new();
    for live in pile.iter().filter_map(OwnedCard::full) {
        *counts.entry((live.id, live.upgrades)).or_default() += 1;
    }
    counts.into_iter().map(|((id, upgrades), count)| CardCountChange { id, upgrades, count }).collect()
}

fn local_instances(combat: &Combat) -> BTreeSet<u64> {
    combat.state.piles.all(combat.state.in_play.as_ref())
        .filter_map(OwnedCard::full)
        .filter(|card| matches!(card.origin, CombatCardRef::CombatLocal(_)))
        .map(|card| card.instance.0)
        .collect()
}

fn snapshot(combat: &Combat) -> Snapshot {
    let state = &combat.state;
    Snapshot {
        hp: state.player.hp,
        block: state.player.block,
        energy: state.player.energy,
        powers: power_values(&state.player_full_powers),
        focus: state.character.focus,
        enemies: state.enemies.iter().map(|enemy| (
            enemy.enemy_id.as_str().to_owned(), enemy.hp, enemy.block,
            power_values(&enemy.full_powers),
        )).collect(),
        stance: state.character.stance,
        orbs: state.character.orb_slots.iter().map(|slot| slot.map(|orb| (orb.kind, orb.dark_damage))).collect(),
        hand: zone(&state.piles.hand),
        draw: zone(&state.piles.draw),
        discard: zone(&state.piles.discard),
        exhaust: zone(&state.piles.exhaust),
        local_instances: local_instances(combat),
    }
}

fn delta(before: Snapshot, combat: &Combat, playable: bool, include_orbs: bool, prior_fault: Option<String>) -> PublicDelta {
    let after = snapshot(combat);
    let enemies = before.enemies.iter().zip(&after.enemies).map(|(old, new)| EnemyDelta {
        id: new.0.clone(),
        hp: new.1 - old.1,
        block: new.2 - old.2,
        powers: changes(&old.3, &new.3),
    }).collect();
    let make_pile_delta = |old: &[CardCountChange], new: &[CardCountChange]| {
        let mut counts = BTreeMap::<(CardId, u32), i32>::new();
        for card in old { *counts.entry((card.id, card.upgrades)).or_default() -= card.count; }
        for card in new { *counts.entry((card.id, card.upgrades)).or_default() += card.count; }
        counts.into_iter().filter_map(|((id, upgrades), count)| {
            (count != 0).then_some(CardCountChange { id, upgrades, count })
        }).collect::<Vec<_>>()
    };
    let mut created_cards = combat.state.piles.all(combat.state.in_play.as_ref())
        .filter_map(OwnedCard::full)
        .filter(|card| matches!(card.origin, CombatCardRef::CombatLocal(_))
            && !before.local_instances.contains(&card.instance.0))
        .map(|card| (card.id, card.upgrades))
        .collect::<Vec<_>>();
    created_cards.sort();
    let fault = prior_fault.or_else(|| combat.fault_record().map(|fault| format!("{fault:?}")));
    PublicDelta {
        player_hp: after.hp - before.hp,
        player_block: after.block - before.block,
        energy: after.energy - before.energy,
        player_powers: changes(&before.powers, &after.powers),
        focus: after.focus - before.focus,
        enemies,
        stance: (before.stance, after.stance),
        hand: make_pile_delta(&before.hand, &after.hand),
        draw: make_pile_delta(&before.draw, &after.draw),
        discard: make_pile_delta(&before.discard, &after.discard),
        exhaust: make_pile_delta(&before.exhaust, &after.exhaust),
        created_cards,
        playable,
        fault,
        orbs: include_orbs.then_some(after.orbs),
    }
}

fn base_items(character: Character, complete: bool) -> CombatItemContext {
    let items = CombatItemContext::new(vec![], vec![None; 3]);
    let items = match character {
        Character::Ironclad => items,
        Character::Silent => items.with_m15_profile(),
        Character::Defect => items.with_m20_profile(),
        Character::Watcher => items.with_m25_profile(),
    };
    if complete { items.with_complete_cards() } else { items }
}

fn ensure_in_hand(combat: &mut Combat, instance: u64) -> Result<(), String> {
    if combat.state.piles.hand.iter().any(|owned| owned.full().is_some_and(|card| card.instance.0 == instance)) {
        return Ok(());
    }
    let card = if let Some(index) = combat.state.piles.draw.iter()
        .position(|owned| owned.full().is_some_and(|card| card.instance.0 == instance)) {
        Some(combat.state.piles.draw.remove(index))
    } else if let Some(index) = combat.state.piles.discard.iter()
        .position(|owned| owned.full().is_some_and(|card| card.instance.0 == instance)) {
        Some(combat.state.piles.discard.remove(index))
    } else if let Some(index) = combat.state.piles.exhaust.iter()
        .position(|owned| owned.full().is_some_and(|card| card.instance.0 == instance)) {
        Some(combat.state.piles.exhaust.remove(index))
    } else {
        None
    };
    let Some(card) = card else { return Err(format!("fixture card instance {instance} is missing")); };
    combat.state.piles.hand.push(card);
    piles::refresh_hand_values(&mut combat.state);
    Ok(())
}

fn keep_only_card_in_hand(combat: &mut Combat, instance: u64) -> Result<(), String> {
    ensure_in_hand(combat, instance)?;
    let hand = std::mem::take(&mut combat.state.piles.hand);
    for owned in hand {
        if owned.full().is_some_and(|card| card.instance.0 == instance) {
            combat.state.piles.hand.push(owned);
        } else {
            combat.state.piles.draw.push(owned);
        }
    }
    piles::refresh_hand_values(&mut combat.state);
    Ok(())
}

fn keep_enemy_alive(combat: &mut Combat) {
    for enemy in &mut combat.state.enemies {
        enemy.hp = 500;
        enemy.max_hp = 500;
        enemy.block = 0;
    }
}

pub(super) fn make_entry(character: Character, first: LiveCard, seed: [u8; 32]) -> Result<CombatEntry, String> {
    make_entry_with_defend(character, first, seed, CardId::Defend)
}

fn make_entry_with_defend(
    character: Character, first: LiveCard, seed: [u8; 32], defend: CardId,
) -> Result<CombatEntry, String> {
    let mut cards = vec![first];
    for instance in 2..=6 {
        let card = if character == Character::Watcher && defend == CardId::DefendPurple {
            crate::combat::m25_watcher_live_card(defend, CardInstanceId(instance), 0)
                .map_err(|error| format!("finite Watcher Defend construction: {error:?}"))?
        } else {
            permanent_live_card(defend, CardInstanceId(instance), 0)
        };
        cards.push(card);
    }
    enemies::setup(
        EncounterId::JawWorm,
        1,
        0,
        &mut RunRng::new(seed, Partition::Test, 0),
        enemies::SetupContext {
            player: PlayerState {
                hp: 80, max_hp: 80, block: 0, energy: 10, max_energy: 10,
                draw_per_turn: 5, powers: vec![],
            },
            player_powers: vec![],
            character,
            cards: cards.clone(),
            opening_draw: 1,
            encounter_visit: 0,
        },
    ).map(|mut entry| {
        entry.cards = cards;
        entry.opening_draw = 1;
        entry
    }).map_err(|error| format!("Jaw Worm setup: {error:?}"))
}

pub(super) fn play_delta(character: Character, card: CardId, upgrades: u32) -> PublicDelta {
    play_delta_with_seed_and_overlay(character, card, upgrades, [41; 32], true, false)
}

pub(super) fn play_delta_with_orbs(character: Character, card: CardId, upgrades: u32) -> PublicDelta {
    play_delta_with_seed_and_overlay(character, card, upgrades, [41; 32], true, true)
}

pub(super) fn play_delta_live(character: Character, card: LiveCard, include_orbs: bool) -> PublicDelta {
    play_delta_live_seeded(character, card, include_orbs, [41; 32], true)
}

pub(super) fn play_delta_with_seed(
    character: Character, card: CardId, upgrades: u32, seed: [u8; 32],
) -> PublicDelta {
    play_delta_with_seed_and_overlay(character, card, upgrades, seed, true, false)
}

pub(super) fn play_delta_with_seed_and_overlay(
    character: Character, card: CardId, upgrades: u32, seed: [u8; 32],
    complete: bool, include_orbs: bool,
) -> PublicDelta {
    play_delta_with_seed_and_overlay_using_defend(
        character, card, upgrades, seed, complete, include_orbs, CardId::Defend,
    )
}

pub(super) fn play_delta_with_seed_and_overlay_using_defend(
    character: Character, card: CardId, upgrades: u32, seed: [u8; 32],
    complete: bool, include_orbs: bool, defend: CardId,
) -> PublicDelta {
    let finite_watcher = character == Character::Watcher
        && crate::content::Catalog::standard().card(card)
            .is_some_and(|definition| definition.color == crate::content::CardColor::Purple);
    let live = if finite_watcher {
        match crate::combat::m25_watcher_live_card(card, CardInstanceId(1), upgrades) {
            Ok(live) => live,
            Err(error) => return failed_delta(format!("finite Watcher construction: {error:?}"), include_orbs),
        }
    } else if complete {
        match crate::combat::complete_live_card(card, CardInstanceId(1), upgrades) {
            Ok(live) => live,
            Err(error) => return failed_delta(format!("card construction: {error:?}"), include_orbs),
        }
    } else if character == Character::Watcher {
        match crate::combat::m25_watcher_live_card(card, CardInstanceId(1), upgrades) {
            Ok(live) => live,
            Err(error) => return failed_delta(format!("finite Watcher construction: {error:?}"), include_orbs),
        }
    } else {
        permanent_live_card(card, CardInstanceId(1), upgrades)
    };
    play_delta_live_seeded_with_overlay_and_defend(character, live, seed, complete, include_orbs, defend)
}

pub(super) fn play_delta_live_seeded(
    character: Character, card: LiveCard, include_orbs: bool, seed: [u8; 32], complete: bool,
) -> PublicDelta {
    play_delta_live_seeded_with_overlay_and_defend(character, card, seed, complete, include_orbs, CardId::Defend)
}

fn play_delta_live_seeded_with_overlay_and_defend(
    character: Character, card: LiveCard, seed: [u8; 32], complete: bool, include_orbs: bool, defend: CardId,
) -> PublicDelta {
    let id = card.id;
    let instance = card.instance.0;
    let entry = match make_entry_with_defend(character, card, seed, defend) {
        Ok(entry) => entry,
        Err(error) => return failed_delta(error, include_orbs),
    };
    let mut context = Some(base_items(character, complete));
    let mut combat = match Combat::enter_with_item_context(entry, &mut context) {
        Ok(combat) => combat,
        Err(error) => return failed_delta(format!("combat entry: {error:?}"), include_orbs),
    };
    if let Err(error) = keep_only_card_in_hand(&mut combat, instance) {
        return failed_delta(error, include_orbs);
    }
    keep_enemy_alive(&mut combat);
    let before = snapshot(&combat);
    let actions = match combat.legal_actions_full() {
        Ok(actions) => actions,
        Err(error) => return delta(before, &combat, false, include_orbs, Some(format!("legal actions: {error:?}"))),
    };
    let play_action = actions.into_iter().find(|action| {
        matches!(action, CombatAction::PlayCard { instance: candidate, .. } if *candidate == instance)
    });
    let Some(play_action) = play_action else {
        return delta(before, &combat, false, include_orbs, None);
    };
    let mut playable = true;
    let mut fault = match combat.step_full(&play_action) {
        Ok(()) => None,
        Err(error) => Some(format!("play {id:?}: {error:?}")),
    };
    if fault.is_none() {
        for _ in 0..50 {
            if combat.fault_record().is_some() {
                break;
            }
            if !matches!(combat.state.phase, Phase::Selection)
                && combat.state.pending_selection.is_none()
                && combat.state.silent.card_offer.is_none()
                && combat.state.in_play.is_none()
            {
                break;
            }
            let actions = match combat.legal_actions_full() {
                Ok(actions) => actions,
                Err(error) => {
                    fault = Some(format!("selection legal actions: {error:?}"));
                    break;
                }
            };
            let action = actions.iter().find(|action| matches!(action, CombatAction::ConfirmSelection))
                .or_else(|| actions.iter().find(|action| matches!(action, CombatAction::SelectCard { .. })))
                .or_else(|| actions.iter().find(|action| matches!(action, CombatAction::ChooseCardOffer { .. })))
                .cloned();
            let Some(action) = action else {
                fault = Some("selection has no legal candidate".to_owned());
                break;
            };
            if let Err(error) = combat.step_full(&action) {
                fault = Some(format!("selection {action:?}: {error:?}"));
                break;
            }
        }
        if fault.is_none() && (combat.state.phase == Phase::Selection
            || combat.state.pending_selection.is_some()
            || combat.state.silent.card_offer.is_some()
            || combat.state.in_play.is_some())
        {
            fault = Some("selection bound".to_owned());
        }
    }
    if let Some(record) = combat.fault_record() {
        if fault.is_none() { fault = Some(format!("{record:?}")); }
    }
    let result = delta(before, &combat, playable, include_orbs, fault);
    playable = result.playable;
    let _ = playable;
    result
}

pub(super) fn play_delta_live_high_hp(character: Character, card: LiveCard, include_orbs: bool) -> PublicDelta {
    let entry = match make_entry(character, card.clone(), [41; 32]) {
        Ok(entry) => entry,
        Err(error) => return failed_delta(error, include_orbs),
    };
    let mut context = Some(base_items(character, true));
    let mut combat = match Combat::enter_with_item_context(entry, &mut context) {
        Ok(combat) => combat,
        Err(error) => return failed_delta(format!("combat entry: {error:?}"), include_orbs),
    };
    if let Err(error) = keep_only_card_in_hand(&mut combat, card.instance.0) {
        return failed_delta(error, include_orbs);
    }
    keep_enemy_alive(&mut combat);
    let before = snapshot(&combat);
    let instance = card.instance.0;
    let actions = match combat.legal_actions_full() {
        Ok(actions) => actions,
        Err(error) => return delta(before, &combat, false, include_orbs, Some(format!("legal actions: {error:?}"))),
    };
    let Some(action) = actions.into_iter().find(|action| {
        matches!(action, CombatAction::PlayCard { instance: candidate, .. } if *candidate == instance)
    }) else { return delta(before, &combat, false, include_orbs, None); };
    let mut fault = combat.step_full(&action).err().map(|error| format!("play {:?}: {error:?}", card.id));
    if fault.is_none() {
        if let Err(error) = resolve_to_play_phase(&mut combat) { fault = Some(error); }
    }
    delta(before, &combat, true, include_orbs, fault)
}

pub(super) fn delta_between(before: &Combat, after: &Combat, playable: bool, include_orbs: bool) -> PublicDelta {
    delta(snapshot(before), after, playable, include_orbs, None)
}

fn failed_delta(message: String, include_orbs: bool) -> PublicDelta {
    PublicDelta {
        player_hp: 0,
        player_block: 0,
        energy: 0,
        player_powers: vec![],
        focus: 0,
        enemies: vec![],
        stance: (Stance::Neutral, Stance::Neutral),
        hand: vec![],
        draw: vec![],
        discard: vec![],
        exhaust: vec![],
        created_cards: vec![],
        playable: false,
        fault: Some(message),
        orbs: include_orbs.then(Vec::new),
    }
}



fn resolve_to_play_phase(combat: &mut Combat) -> Result<(), String> {
    for _ in 0..50 {
        if combat.fault_record().is_some() {
            return Err(format!("combat fault: {:?}", combat.fault_record()));
        }
        if combat.state.phase != Phase::Selection
            && combat.state.pending_selection.is_none()
            && combat.state.silent.card_offer.is_none()
            && combat.state.in_play.is_none()
        {
            return Ok(());
        }
        let actions = combat.legal_actions_full().map_err(|error| format!("selection actions: {error:?}"))?;
        let action = actions.iter().find(|action| matches!(action, CombatAction::ConfirmSelection))
            .or_else(|| actions.iter().find(|action| matches!(action, CombatAction::SelectCard { .. })))
            .or_else(|| actions.iter().find(|action| matches!(action, CombatAction::ChooseCardOffer { .. })))
            .cloned().ok_or_else(|| "selection has no legal candidate".to_owned())?;
        combat.step_full(&action).map_err(|error| format!("selection {action:?}: {error:?}"))?;
    }
    Err("selection bound".to_owned())
}

pub(super) fn generated_cards(
    character: Character, generator: CardId, seed: [u8; 32],
) -> Result<Vec<(CardId, u32)>, String> {
    let live = crate::combat::complete_live_card(generator, CardInstanceId(1), 0)
        .map_err(|error| format!("{generator:?} construction: {error:?}"))?;
    let entry = make_entry(character, live, seed)?;
    let mut context = Some(base_items(character, true));
    let mut combat = Combat::enter_with_item_context(entry, &mut context)
        .map_err(|error| format!("{character:?} {generator:?} entry: {error:?}"))?;
    ensure_in_hand(&mut combat, 1)?;
    let action = combat.legal_actions_full().map_err(|error| format!("legal actions: {error:?}"))?
        .into_iter().find(|action| matches!(action, CombatAction::PlayCard { instance: 1, .. }))
        .ok_or_else(|| format!("{generator:?} is not playable"))?;
    combat.step_full(&action).map_err(|error| format!("{generator:?} play: {error:?}"))?;
    resolve_to_play_phase(&mut combat)?;

    if matches!(generator, CardId::HelloWorld | CardId::CreativeAI | CardId::Magnetism) {
        combat.step_full(&CombatAction::EndTurn).map_err(|error| format!("{generator:?} EndTurn: {error:?}"))?;
        resolve_to_play_phase(&mut combat)?;
    }

    let mut generated = combat.state.piles.all(combat.state.in_play.as_ref())
        .filter_map(OwnedCard::full)
        .filter(|card| matches!(card.origin, CombatCardRef::CombatLocal(_)))
        .map(|card| (card.id, card.upgrades))
        .collect::<Vec<_>>();
    generated.sort();
    if generated.is_empty() {
        return Err(format!("{generator:?} made no combat-local cards"));
    }
    Ok(generated)
}

pub(super) fn alchemize_potion(character: Character, seed: [u8; 32]) -> Result<Option<PotionId>, String> {
    let live = crate::combat::complete_live_card(CardId::Alchemize, CardInstanceId(1), 0)
        .map_err(|error| format!("Alchemize construction: {error:?}"))?;
    let mut entry = make_entry(character, live, seed)?;
    let stream = &entry.setup.card_rng;
    entry.setup.card_rng = crate::combat::rng::StreamState::new(stream.id().clone(), seed, 0)
        .map_err(|error| format!("Alchemize random stream: {error:?}"))?;
    let mut context = Some(base_items(character, true));
    let mut combat = Combat::enter_with_item_context(entry, &mut context)
        .map_err(|error| format!("{character:?} Alchemize entry: {error:?}"))?;
    ensure_in_hand(&mut combat, 1)?;
    let action = combat.legal_actions_full().map_err(|error| format!("Alchemize legal actions: {error:?}"))?
        .into_iter().find(|action| matches!(action, CombatAction::PlayCard { instance: 1, .. }))
        .ok_or_else(|| format!("{character:?} Alchemize is not playable"))?;
    combat.step_full(&action).map_err(|error| format!("{character:?} Alchemize play: {error:?}"))?;
    resolve_to_play_phase(&mut combat)?;
    Ok(combat.item_context().and_then(|items| items.potions().first().copied().flatten()))
}
