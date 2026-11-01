//! Lane B matrix G: seeded player-pool membership for every shared generator.
use super::m26_support::{self, CHARACTERS};
use crate::{
    content::{pools::{self, CardPool, GenerationPool}, CardId, Character},
};

#[derive(Clone, Copy)]
struct GeneratorSpec { id: CardId, pool: GenerationPool }

fn generators() -> [GeneratorSpec; 11] {
    use CardId::*;
    [
        GeneratorSpec { id: InfernalBlade, pool: GenerationPool::Attack },
        GeneratorSpec { id: Distraction, pool: GenerationPool::Skill },
        GeneratorSpec { id: WhiteNoise, pool: GenerationPool::Power },
        GeneratorSpec { id: Discovery, pool: GenerationPool::Any },
        GeneratorSpec { id: Chrysalis, pool: GenerationPool::Skill },
        GeneratorSpec { id: Metamorphosis, pool: GenerationPool::Attack },
        GeneratorSpec { id: HelloWorld, pool: GenerationPool::Common },
        GeneratorSpec { id: CreativeAI, pool: GenerationPool::Power },
        GeneratorSpec { id: JackOfAllTrades, pool: GenerationPool::Colorless },
        GeneratorSpec { id: Transmutation, pool: GenerationPool::Colorless },
        GeneratorSpec { id: Magnetism, pool: GenerationPool::Colorless },
    ]
}

fn expected(character: Character, generator: GeneratorSpec) -> Result<Vec<CardId>, String> {
    pools::cards(CardPool::Combat { character, pool: generator.pool })
        .map_err(|error| format!("{:?}: {error:?}", generator.pool))
}

#[test]
fn m26_generation_player_pool() {
    let mut mismatches = Vec::new();
    for character in CHARACTERS {
        for generator in generators() {
            let pool = match expected(character, generator) {
                Ok(pool) => pool,
                Err(error) => {
                    mismatches.push(format!("{:?}|0|{character:?}|pool available|{error}", generator.id));
                    continue;
                }
            };
            for seed in 1u8..=200 {
                match m26_support::generated_cards(character, generator.id, [seed; 32]) {
                    Ok(created) => {
                        for (id, upgrades) in created {
                            if !pool.contains(&id) {
                                mismatches.push(format!("{:?}|{seed}|{character:?}|{:?} member|{id:?}+{upgrades}",
                                    generator.id, generator.pool));
                            }
                        }
                    }
                    Err(error) => mismatches.push(format!("{:?}|{seed}|{character:?}|{:?} member|{error}",
                        generator.id, generator.pool)),
                }
            }
        }
    }
    assert!(mismatches.is_empty(), "m26 generation pool rows (kind|seed|character|expected|actual):\n{}",
        mismatches.join("\n"));
}

use crate::{
    combat::{Combat, CombatAction, CardInstanceId, CombatCardRef, model::OwnedCard},
    content::{CardType, Catalog},
    items::CombatItemContext,
};

fn m26_generation_items(character: Character) -> CombatItemContext {
    let items = CombatItemContext::new(vec![], vec![None; 3]);
    match character {
        Character::Ironclad => items,
        Character::Silent => items.with_m15_profile(),
        Character::Defect => items.with_m20_profile(),
        Character::Watcher => items.with_m25_profile(),
    }
    .with_complete_cards()
}

fn m26_generation_fixture(character: Character, card: CardId) -> Result<Combat, String> {
    let master_reality = crate::combat::complete_live_card(CardId::MasterReality, CardInstanceId(1), 0)
        .map_err(|error| format!("MasterReality construction: {error:?}"))?;
    let mut entry = m26_support::make_entry(character, master_reality, [0x26; 32])?;
    let generated = crate::combat::complete_live_card(card, CardInstanceId(7), 0)
        .map_err(|error| format!("{card:?} construction: {error:?}"))?;
    entry.cards.push(generated);
    if card == CardId::DualWield {
        let strike = crate::combat::complete_live_card(CardId::Strike, CardInstanceId(8), 0)
            .map_err(|error| format!("Strike construction: {error:?}"))?;
        entry.cards.push(strike);
    } else {
        for instance in 8..=10 {
            let defend = crate::combat::complete_live_card(CardId::Defend, CardInstanceId(instance), 0)
                .map_err(|error| format!("Defend construction: {error:?}"))?;
            entry.cards.push(defend);
        }
    }
    entry.opening_draw = entry.cards.len();
    let mut context = Some(m26_generation_items(character));
    Combat::enter_with_item_context(entry, &mut context)
        .map_err(|error| format!("{character:?} {card:?} entry: {error:?}"))
}

fn m26_generation_resolve_choices(combat: &mut Combat) -> Result<(), String> {
    for _ in 0..100 {
        if let Some(fault) = combat.fault_record() {
            return Err(format!("resolution fault: {fault:?}"));
        }
        if combat.state.phase != crate::combat::Phase::Selection
            && combat.state.pending_selection.is_none()
            && combat.state.silent.card_offer.is_none()
            && combat.state.in_play.is_none()
        {
            return Ok(());
        }
        let actions = combat.legal_actions_full().map_err(|error| format!("choice actions: {error:?}"))?;
        let action = actions.iter().find(|action| matches!(action, CombatAction::ChooseCardOffer { index: 0 }))
            .or_else(|| actions.iter().find(|action| matches!(action, CombatAction::ConfirmSelection)))
            .or_else(|| actions.iter().find(|action| matches!(action, CombatAction::SelectCard { .. })))
            .cloned().ok_or_else(|| "choice has no first offered card/action".to_owned())?;
        combat.step_full(&action).map_err(|error| format!("choice {action:?}: {error:?}"))?;
    }
    Err("choice resolution exceeded 100 actions".to_owned())
}

fn m26_generation_play(combat: &mut Combat, instance: u64) -> Result<(), String> {
    let actions = combat.legal_actions_full().map_err(|error| format!("legal actions: {error:?}"))?;
    let action = actions.into_iter().find(|action| {
        matches!(action, CombatAction::PlayCard { instance: candidate, .. } if *candidate == instance)
    }).ok_or_else(|| format!("card instance {instance} has no legal play action"))?;
    combat.step_full(&action).map_err(|error| format!("play instance {instance}: {error:?}"))?;
    m26_generation_resolve_choices(combat)
}

fn m26_generation_local_instances(combat: &Combat) -> std::collections::BTreeSet<u64> {
    super::super::piles::owned_full_cards(&combat.state)
        .filter(|card| matches!(card.origin, CombatCardRef::CombatLocal(_)))
        .map(|card| card.instance.0)
        .collect()
}

fn m26_generation_zone(combat: &Combat, instance: u64) -> &'static str {
    let in_pile = |pile: &[OwnedCard]| pile.iter().any(|owned| {
        owned.full().is_some_and(|card| card.instance.0 == instance)
    });
    if in_pile(&combat.state.piles.hand) { "hand" }
    else if in_pile(&combat.state.piles.draw) { "draw" }
    else if in_pile(&combat.state.piles.discard) { "discard" }
    else if in_pile(&combat.state.piles.exhaust) { "exhaust" }
    else if combat.state.in_play.as_ref().and_then(OwnedCard::full).is_some_and(|card| card.instance.0 == instance) { "in-play" }
    else { "other" }
}

#[test]
fn m26_generation_master_reality_every_character() {
    // M25 I-1 Master Reality creation law applied to M26 generators.
    let mut failures = Vec::new();
    for character in CHARACTERS {
        for generator in generators() {
            let result = (|| -> Result<Vec<(CardId, u32, &'static str)>, String> {
                let mut combat = m26_generation_fixture(character, generator.id)?;
                m26_generation_play(&mut combat, 1)?;
                if !combat.state.player_full_powers.iter().any(|power| power.id.0 == "master_reality") {
                    return Err("Master Reality was not installed by its played card".to_owned());
                }
                let before = m26_generation_local_instances(&combat);
                m26_generation_play(&mut combat, 7)?;
                if matches!(generator.id, CardId::HelloWorld | CardId::CreativeAI | CardId::Magnetism) {
                    combat.step_full(&CombatAction::EndTurn).map_err(|error| format!("EndTurn: {error:?}"))?;
                    m26_generation_resolve_choices(&mut combat)?;
                }
                let created = super::super::piles::owned_full_cards(&combat.state)
                    .filter(|card| matches!(card.origin, CombatCardRef::CombatLocal(_)) && !before.contains(&card.instance.0))
                    .filter(|card| Catalog::standard().card(card.id).is_some_and(|row| !matches!(row.card_type, CardType::Curse | CardType::Status)))
                    .map(|card| (card.id, card.upgrades, m26_generation_zone(&combat, card.instance.0)))
                    .collect::<Vec<_>>();
                if created.is_empty() {
                    return Err("assertion failed: generator made a new non-Curse/Status card".to_owned());
                }
                let wrong = created.iter().filter(|(_, upgrades, _)| *upgrades != 1).collect::<Vec<_>>();
                if !wrong.is_empty() {
                    return Err(format!("assertion failed: every newly created non-Curse/Status card has upgrades == 1; actual={wrong:?}"));
                }
                Ok(created)
            })();
            if let Err(error) = result {
                failures.push(format!("character={character:?} generator={:?}: {error}", generator.id));
            }
        }

        let dual_wield = (|| -> Result<Vec<(CardId, u32, &'static str)>, String> {
            let mut combat = m26_generation_fixture(character, CardId::DualWield)?;
            m26_generation_play(&mut combat, 1)?;
            if !combat.state.player_full_powers.iter().any(|power| power.id.0 == "master_reality") {
                return Err("Master Reality was not installed by its played card".to_owned());
            }
            let before = m26_generation_local_instances(&combat);
            m26_generation_play(&mut combat, 7)?;
            let created = super::super::piles::owned_full_cards(&combat.state)
                .filter(|card| matches!(card.origin, CombatCardRef::CombatLocal(_)) && !before.contains(&card.instance.0))
                .filter(|card| Catalog::standard().card(card.id).is_some_and(|row| !matches!(row.card_type, CardType::Curse | CardType::Status)))
                .map(|card| (card.id, card.upgrades, m26_generation_zone(&combat, card.instance.0)))
                .collect::<Vec<_>>();
            if created.is_empty() { return Err("assertion failed: Dual Wield copied the base Strike".to_owned()); }
            let wrong = created.iter().filter(|(_, upgrades, _)| *upgrades != 1).collect::<Vec<_>>();
            if !wrong.is_empty() {
                return Err(format!("assertion failed: every newly created non-Curse/Status card has upgrades == 1; actual={wrong:?}"));
            }
            Ok(created)
        })();
        if let Err(error) = dual_wield {
            failures.push(format!("character={character:?} generator=DualWield(base Strike): {error}"));
        }
    }
    assert!(failures.is_empty(), "M25 I-1 Master Reality creation law applied to M26 generators; RED mismatches:\n{}",
        failures.join("\n"));
}

#[test]
fn m26_generation_master_reality_curse_return_stays_base() {
    // M25 I-1 Master Reality creation law applied to M26 generators.
    for character in CHARACTERS {
        let master_reality = crate::combat::complete_live_card(CardId::MasterReality, CardInstanceId(1), 0)
            .expect("Master Reality construction");
        let mut entry = m26_support::make_entry(character, master_reality, [0x26; 32])
            .expect("Necronomicurse fixture");
        let curse = crate::combat::complete_live_card(CardId::Necronomicurse, CardInstanceId(7), 0)
            .expect("Necronomicurse construction");
        entry.cards.push(curse);
        entry.opening_draw = entry.cards.len();
        let mut context = Some(m26_generation_items(character));
        let mut combat = Combat::enter_with_item_context(entry, &mut context)
            .unwrap_or_else(|error| panic!("{character:?} Necronomicurse entry: {error:?}"));
        m26_generation_play(&mut combat, 1).unwrap_or_else(|error| panic!("{character:?} Master Reality play: {error}"));
        let index = combat.state.piles.hand.iter().position(|owned|
            owned.full().is_some_and(|card| card.instance == CardInstanceId(7)))
            .unwrap_or_else(|| panic!("{character:?} Necronomicurse is not in hand"));
        let permanent = combat.state.piles.hand.remove(index);
        let source = permanent.full().expect("Necronomicurse full card").clone();
        combat.state.piles.exhaust.push(permanent);
        super::super::piles::refresh_hand_values(&mut combat.state);
        let mut work = Vec::new();
        super::super::cards::curses::on_exhaust(&combat.state, &source, &mut work);
        combat.enqueue_native_effects(work, None)
            .unwrap_or_else(|error| panic!("{character:?} Necronomicurse return: {error:?}"));
        let returned = super::super::piles::owned_full_cards(&combat.state)
            .find(|card| card.id == CardId::Necronomicurse && card.origin == CombatCardRef::CombatLocal(card.instance.0))
            .unwrap_or_else(|| panic!("{character:?} returned Necronomicurse is missing"));
        assert_eq!(returned.upgrades, 0, "{character:?} returned Necronomicurse remains base under Master Reality");
    }
}
