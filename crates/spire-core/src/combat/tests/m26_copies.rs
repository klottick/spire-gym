//! Lane B matrices Y and generated/status playback.
use super::super::*;
use super::m26_support::{self, PublicDelta, CHARACTERS};
use crate::{
    combat::{card_state::CopyProvenance, model::OwnedCard, rng::Partition},
    content::{complete::{supports_generated, supports_permanent}, CardId, Character, EncounterId, Catalog},
    items::CombatItemContext,
    run::{complete_upgrades_valid, RunRng},
};

fn upgrades(id: CardId) -> Vec<u32> {
    if complete_upgrades_valid(id, 1) { vec![0, 1] } else { vec![0] }
}

fn assert_copy_row(
    mismatches: &mut Vec<String>,
    id: CardId,
    upgrade: u32,
    source_misc: i32,
    canonical: &LiveCard,
) {
    let source_result = crate::combat::complete_live_card(id, CardInstanceId(1), upgrade);
    let Ok(mut source) = source_result else {
        mismatches.push(format!("{id:?}|{upgrade}|copy|source constructs|{source_result:?}"));
        return;
    };
    source.misc = source_misc;
    source.copy_family = 9_999;
    source.local.ethereal = !canonical.local.ethereal;
    source.local.exhaust_on_finish = !canonical.local.exhaust_on_finish;
    source.local.innate = !canonical.local.innate;
    source.local.x_cost = !canonical.local.x_cost;
    let copy = source.stat_equivalent_copy(CardInstanceId(9_000));
    let expected = (
        id, upgrade, source_misc, CardInstanceId(9_000), CombatCardRef::CombatLocal(9_000),
        9_000, CopyProvenance::StatEquivalent,
        canonical.local.ethereal, canonical.local.exhaust_on_finish,
        canonical.local.innate, canonical.local.x_cost,
    );
    let actual = (
        copy.id, copy.upgrades, copy.misc, copy.instance, copy.origin.clone(),
        copy.copy_family, copy.copy_provenance, copy.local.ethereal,
        copy.local.exhaust_on_finish, copy.local.innate, copy.local.x_cost,
    );
    if actual != expected {
        mismatches.push(format!("{id:?}|{upgrade}|stat_equivalent_copy|{expected:?}|{actual:?}"));
    }
}

fn combat_with(
    character: Character,
    cards: &[(CardId, u32, i32)],
    seed: [u8; 32],
) -> Result<Combat, String> {
    let mut live = cards.iter().enumerate().map(|(index, (id, upgrade, misc))| {
        let mut card = crate::combat::complete_live_card(*id, CardInstanceId(index as u64 + 1), *upgrade)
            .map_err(|error| format!("{id:?}+{upgrade} construction: {error:?}"))?;
        card.misc = *misc;
        Ok(card)
    }).collect::<Result<Vec<_>, String>>()?;
    let next = live.len() as u64 + 1;
    for instance in next..next + 5 {
        live.push(permanent_live_card(CardId::Defend, CardInstanceId(instance), 0));
    }
    let mut entry = enemies::setup(EncounterId::JawWorm, 1, 0,
        &mut RunRng::new(seed, Partition::Test, 0),
        enemies::SetupContext {
            player: PlayerState { hp: 80, max_hp: 80, block: 0, energy: 10,
                max_energy: 10, draw_per_turn: 5, powers: vec![] },
            player_powers: vec![], character, cards: live.clone(), opening_draw: live.len(), encounter_visit: 0,
        }).map_err(|error| format!("Jaw Worm setup: {error:?}"))?;
    entry.cards = live;
    entry.opening_draw = entry.cards.len();
    let items = CombatItemContext::new(vec![], vec![None; 3]);
    let items = match character {
        Character::Ironclad => items,
        Character::Silent => items.with_m15_profile(),
        Character::Defect => items.with_m20_profile(),
        Character::Watcher => items.with_m25_profile(),
    }.with_complete_cards();
    let mut context = Some(items);
    let mut combat = Combat::enter_with_item_context(entry, &mut context)
        .map_err(|error| format!("combat entry: {error:?}"))?;
    for enemy in &mut combat.state.enemies { enemy.hp = 500; enemy.max_hp = 500; enemy.block = 0; }
    Ok(combat)
}

fn resolve_selection(combat: &mut Combat) -> Result<(), String> {
    for _ in 0..50 {
        if combat.fault_record().is_some() { return Err(format!("fault: {:?}", combat.fault_record())); }
        if combat.state.phase != Phase::Selection && combat.state.pending_selection.is_none()
            && combat.state.silent.card_offer.is_none() && combat.state.in_play.is_none() {
            return Ok(());
        }
        let actions = combat.legal_actions_full().map_err(|error| format!("actions: {error:?}"))?;
        let action = actions.iter().find(|action| matches!(action, CombatAction::ConfirmSelection))
            .or_else(|| actions.iter().find(|action| matches!(action, CombatAction::SelectCard { .. })))
            .or_else(|| actions.iter().find(|action| matches!(action, CombatAction::ChooseCardOffer { .. })))
            .cloned().ok_or_else(|| "selection has no legal candidate".to_owned())?;
        combat.step_full(&action).map_err(|error| format!("{action:?}: {error:?}"))?;
    }
    Err("selection bound".to_owned())
}

fn play_instance_unresolved(combat: &mut Combat, instance: u64) -> Result<(), String> {
    let actions = combat.legal_actions_full().map_err(|error| format!("actions: {error:?}"))?;
    let action = actions.iter().find(|action| {
        matches!(action, CombatAction::PlayCard { instance: candidate, .. } if *candidate == instance)
    }).cloned().ok_or_else(|| {
        let hand = combat.state.piles.hand.iter().filter_map(OwnedCard::full)
            .map(|card| (card.id, card.instance.0, card.upgrades)).collect::<Vec<_>>();
        format!("instance {instance} has no legal PlayCard; phase={:?}, hand={hand:?}, actions={actions:?}", combat.state.phase)
    })?;
    combat.step_full(&action).map_err(|error| format!("play {instance}: {error:?}"))?;
    Ok(())
}

fn play_instance(combat: &mut Combat, instance: u64) -> Result<(), String> {
    play_instance_unresolved(combat, instance)?;
    resolve_selection(combat)
}

fn find_card(combat: &Combat, id: CardId, local_only: bool) -> Option<LiveCard> {
    combat.state.piles.all(combat.state.in_play.as_ref())
        .filter_map(OwnedCard::full)
        .find(|card| card.id == id && (!local_only || matches!(card.origin, CombatCardRef::CombatLocal(_))))
        .cloned()
}

fn move_to_hand(combat: &mut Combat, id: CardId) -> Option<LiveCard> {
    if let Some(card) = combat.state.piles.hand.iter().filter_map(OwnedCard::full)
        .find(|card| card.id == id).cloned() { return Some(card); }
    let moved = if let Some(index) = combat.state.piles.draw.iter().position(|owned| owned.full().is_some_and(|card| card.id == id)) {
        Some(combat.state.piles.draw.remove(index))
    } else if let Some(index) = combat.state.piles.discard.iter().position(|owned| owned.full().is_some_and(|card| card.id == id)) {
        Some(combat.state.piles.discard.remove(index))
    } else if let Some(index) = combat.state.piles.exhaust.iter().position(|owned| owned.full().is_some_and(|card| card.id == id)) {
        Some(combat.state.piles.exhaust.remove(index))
    } else { None };
    if let Some(owned) = moved {
        let live = owned.full()?.clone();
        combat.state.piles.hand.push(owned);
        piles::refresh_hand_values(&mut combat.state);
        return Some(live);
    }
    None
}

fn generated_child_delta(
    character: Character,
    child: CardId,
    producer: CardId,
    needs_next_turn: bool,
) -> Result<(PublicDelta, LiveCard), String> {
    let mut combat = combat_with(character, &[(producer, 0, 0)], [41; 32])?;
    let parent = combat.state.piles.hand.iter().filter_map(OwnedCard::full)
        .find(|card| card.id == producer).map(|card| card.instance.0)
        .ok_or_else(|| format!("{producer:?} not in opening hand"))?;
    play_instance(&mut combat, parent)?;
    if needs_next_turn {
        combat.step_full(&CombatAction::EndTurn).map_err(|error| format!("{producer:?} EndTurn: {error:?}"))?;
        resolve_selection(&mut combat)?;
    }
    let card = move_to_hand(&mut combat, child)
        .or_else(|| find_card(&combat, child, true))
        .ok_or_else(|| format!("{producer:?} did not create {child:?}"))?;
    let expected = m26_support::play_delta_live_high_hp(Character::Ironclad, card.clone(), false);
    let before = combat.clone();
    let playable = combat.legal_actions_full().map_err(|error| format!("{child:?} actions: {error:?}"))?
        .iter().any(|action| matches!(action, CombatAction::PlayCard { instance, .. }
            if *instance == card.instance.0));
    if playable {
        play_instance(&mut combat, card.instance.0)?;
    }
    let actual = m26_support::delta_between(&before, &combat, true, false);
    if expected.fault.is_some() { return Err(format!("{child:?} owning result fault: {expected:?}")); }
    if !playable {
        return Ok((m26_support::delta_between(&combat, &combat, false, false), card));
    }
    Ok((actual, card))
}

fn dual_wield_delta(character: Character) -> Result<PublicDelta, String> {
    let mut combat = combat_with(character, &[(CardId::DualWield, 0, 0), (CardId::Eruption, 0, 0)], [41; 32])?;
    play_instance(&mut combat, 1)?;
    let copy = find_card(&combat, CardId::Eruption, true).ok_or_else(|| "Dual Wield made no Eruption copy".to_owned())?;
    let before = combat.clone();
    play_instance(&mut combat, copy.instance.0)?;
    Ok(m26_support::delta_between(&before, &combat, true, false))
}

fn nightmare_delta(character: Character) -> Result<PublicDelta, String> {
    let mut combat = combat_with(character, &[(CardId::Nightmare, 0, 0), (CardId::Eruption, 0, 0)], [41; 32])?;
    play_instance_unresolved(&mut combat, 1)?;
    let actions = combat.legal_actions_full().map_err(|error| format!("Nightmare selection actions: {error:?}"))?;
    let target = actions.into_iter().find(|action| matches!(action, CombatAction::SelectCard { instance }
        if combat.state.piles.hand.iter().any(|owned| owned.full().is_some_and(|card|
            card.instance.0 == *instance && card.id == CardId::Eruption))))
        .ok_or_else(|| "Nightmare has no legal Eruption selection".to_owned())?;
    combat.step_full(&target).map_err(|error| format!("Nightmare select Eruption: {error:?}"))?;
    resolve_selection(&mut combat)?;
    combat.step_full(&CombatAction::EndTurn).map_err(|error| format!("Nightmare EndTurn: {error:?}"))?;
    resolve_selection(&mut combat)?;
    let copy = find_card(&combat, CardId::Eruption, true).ok_or_else(|| "Nightmare made no Eruption copy".to_owned())?;
    let before = combat.clone();
    play_instance(&mut combat, copy.instance.0)?;
    Ok(m26_support::delta_between(&before, &combat, true, false))
}

fn anger_copy_delta(character: Character) -> Result<PublicDelta, String> {
    let mut combat = combat_with(character, &[(CardId::Anger, 0, 0)], [41; 32])?;
    play_instance(&mut combat, 1)?;
    let copy = move_to_hand(&mut combat, CardId::Anger).ok_or_else(|| "Anger made no copy".to_owned())?;
    let before = combat.clone();
    play_instance(&mut combat, copy.instance.0)?;
    Ok(m26_support::delta_between(&before, &combat, true, false))
}

fn power_copy_delta(character: Character, power: CardId, copied: CardId) -> Result<PublicDelta, String> {
    let mut combat = combat_with(character, &[(power, 0, 0), (copied, 0, 0)], [41; 32])?;
    play_instance(&mut combat, 1)?;
    let before = combat.clone();
    play_instance(&mut combat, 2)?;
    Ok(m26_support::delta_between(&before, &combat, true, false))
}

fn necronomicurse_copy(character: Character) -> Result<LiveCard, String> {
    let source = crate::combat::complete_live_card(CardId::Necronomicurse, CardInstanceId(1), 0)
        .map_err(|error| format!("Necronomicurse construction: {error:?}"))?;
    let mut entry = m26_support::make_entry(character, source, [41; 32])?;
    entry.opening_draw = entry.cards.len();
    let items = CombatItemContext::new(vec![], vec![None; 3]);
    let items = match character {
        Character::Ironclad => items,
        Character::Silent => items.with_m15_profile(),
        Character::Defect => items.with_m20_profile(),
        Character::Watcher => items.with_m25_profile(),
    }.with_complete_cards();
    let mut context = Some(items);
    let mut combat = Combat::enter_with_item_context(entry, &mut context)
        .map_err(|error| format!("Necronomicurse entry: {error:?}"))?;
    let index = combat.state.piles.hand.iter().position(|owned| owned.full().is_some_and(|card| card.id == CardId::Necronomicurse))
        .ok_or_else(|| "Necronomicurse not in hand".to_owned())?;
    let permanent = combat.state.piles.hand.remove(index);
    let source = permanent.full().ok_or_else(|| "Necronomicurse full card missing".to_owned())?.clone();
    combat.state.piles.exhaust.push(permanent);
    let mut work = Vec::new();
    cards::curses::on_exhaust(&combat.state, &source, &mut work);
    combat.enqueue_native_effects(work, None).map_err(|error| format!("Necronomicurse return: {error:?}"))?;
    let returned = find_card(&combat, CardId::Necronomicurse, true)
        .ok_or_else(|| "returned Necronomicurse is not combat-local".to_owned())?;
    if returned.instance == CardInstanceId(1) || returned.upgrades != 0
        || returned.origin != CombatCardRef::CombatLocal(returned.instance.0) {
        return Err(format!("returned copy must be a fresh base combat-local card: {returned:?}"));
    }
    Ok(returned)
}

#[test]
fn m26_copies_every_kind() {
    let rows = Catalog::standard().cards();
    let mut mismatches = Vec::new();
    for definition in rows.iter().filter(|row| supports_permanent(row.id) || supports_generated(row.id)) {
        let id = definition.id;
        for upgrade in upgrades(id) {
            let canonical = match crate::combat::complete_live_card(id, CardInstanceId(8_000), upgrade) {
                Ok(card) => card,
                Err(error) => {
                    mismatches.push(format!("{id:?}|{upgrade}|canonical|constructs|{error:?}"));
                    continue;
                }
            };
            let misc = if id == CardId::RitualDagger { 21 }
                else if id == CardId::GeneticAlgorithm { 4 }
                else { 73 };
            assert_copy_row(&mut mismatches, id, upgrade, misc, &canonical);
        }
    }
    for (id, upgrade, misc) in [
        (CardId::SearingBlow, 3, 0),
        (CardId::RitualDagger, 0, 21),
        (CardId::GeneticAlgorithm, 0, 4),
    ] {
        if let Ok(canonical) = crate::combat::complete_live_card(id, CardInstanceId(8_000), upgrade) {
            assert_copy_row(&mut mismatches, id, upgrade, misc, &canonical);
        } else {
            mismatches.push(format!("{id:?}|{upgrade}|canonical|constructs|missing"));
        }
    }

    // Each real producer creates or repeats an off-colour card; compare the
    // resulting play delta with its owning-colour row.
    for character in CHARACTERS {
        for (producer, copied, owning) in [
            (CardId::DualWield, CardId::Eruption, Character::Watcher),
            (CardId::Nightmare, CardId::Eruption, Character::Watcher),
            (CardId::Anger, CardId::Anger, Character::Ironclad),
        ] {
            let actual = match producer {
                CardId::DualWield => dual_wield_delta(character),
                CardId::Nightmare => nightmare_delta(character),
                _ => anger_copy_delta(character),
            };
            let expected = m26_support::play_delta(owning, copied, 0);
            match actual {
                Ok(actual) if actual == expected && actual.fault.is_none() => {}
                Ok(actual) => mismatches.push(format!("{copied:?}|0|{character:?}|{expected:?}|{actual:?}")),
                Err(error) => mismatches.push(format!("{producer:?}|0|{character:?}|owner delta {expected:?}|{error}")),
            }
        }
        for (power, copied, owning) in [
            (CardId::Amplify, CardId::Defragment, Character::Defect),
            (CardId::EchoForm, CardId::BallLightning, Character::Defect),
        ] {
            let actual = power_copy_delta(character, power, copied);
            let expected = power_copy_delta(owning, power, copied);
            match (actual, expected) {
                (Ok(actual), Ok(expected)) if actual == expected && actual.fault.is_none() => {}
                (Ok(actual), Ok(expected)) => mismatches.push(format!("{copied:?}|0|{character:?}|{expected:?}|{actual:?}")),
                (Err(error), expected) => mismatches.push(format!("{power:?}|0|{character:?}|owner delta {expected:?}|{error}")),
                (Ok(actual), Err(error)) => mismatches.push(format!("{copied:?}|0|{character:?}|owner delta error {error}|{actual:?}")),
            }
        }

        match necronomicurse_copy(character) {
            Ok(_) => {}
            Err(error) => mismatches.push(format!("Necronomicurse|0|{character:?}|fresh base CombatLocal copy|{error}")),
        }
    }

    assert!(mismatches.is_empty(), "m26 copied-card rows (kind|upgrades|character|expected|actual):\n{}",
        mismatches.join("\n"));
}

#[derive(Clone, Copy)]
struct ChildProducer { child: CardId, parent: CardId, next_turn: bool }

fn child_producers() -> [ChildProducer; 9] {
    use CardId::*;
    [
        ChildProducer { child: Shiv, parent: BladeDance, next_turn: false },
        ChildProducer { child: Miracle, parent: Collect, next_turn: true },
        ChildProducer { child: Smite, parent: BattleHymn, next_turn: true },
        ChildProducer { child: Safety, parent: DeceiveReality, next_turn: false },
        ChildProducer { child: Insight, parent: Evaluate, next_turn: false },
        ChildProducer { child: ThroughViolence, parent: ReachHeaven, next_turn: false },
        ChildProducer { child: Beta, parent: Alpha, next_turn: false },
        ChildProducer { child: Omega, parent: Beta, next_turn: false },
        ChildProducer { child: Expunger, parent: ConjureBlade, next_turn: false },
    ]
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct StatusResult {
    playable: bool,
    hp_delta: Option<i32>,
    energy_delta: Option<i32>,
    hand: Vec<CardId>,
    discard: Vec<CardId>,
    exhaust: Vec<CardId>,
    fault: Option<String>,
}

fn resolve_status(character: Character, id: CardId) -> Result<StatusResult, String> {
    let live = crate::combat::complete_live_card(CardId::Defend, CardInstanceId(1), 0)
        .map_err(|error| format!("status fixture: {error:?}"))?;
    let entry = m26_support::make_entry(character, live, [41; 32])?;
    let items = CombatItemContext::new(vec![], vec![None; 3]);
    let items = match character {
        Character::Ironclad => items,
        Character::Silent => items.with_m15_profile(),
        Character::Defect => items.with_m20_profile(),
        Character::Watcher => items.with_m25_profile(),
    }.with_complete_cards();
    let mut context = Some(items);
    let mut combat = Combat::enter_with_item_context(entry, &mut context)
        .map_err(|error| format!("{id:?} entry: {error:?}"))?;
    let before_hp = combat.state.player.hp;
    let before_energy = combat.state.player.energy;
    combat.enqueue_native_effects(vec![Effect { source: Source::None, target: Some(Actor::Player),
        kind: EffectKind::CreateCard { id, destination: CardDestination::Hand, upgrades: 0 } }], None)
        .map_err(|error| format!("{id:?} accepted CreateCard producer: {error:?}"))?;
    let legal = combat.legal_actions_full().map_err(|error| format!("{id:?} actions: {error:?}"))?
        .iter().any(|action| matches!(action, CombatAction::PlayCard { instance, .. }
            if combat.state.piles.hand.iter().any(|owned| owned.handle().as_str() == instance.to_string()
                && owned.full().is_some_and(|card| card.id == id))));

    if id == CardId::Void {
        let index = combat.state.piles.hand.iter().position(|owned| owned.full().is_some_and(|card| card.id == id))
            .ok_or_else(|| "Void creation missing from hand".to_owned())?;
        let void = combat.state.piles.hand.remove(index);
        combat.state.piles.draw.clear();
        combat.state.piles.draw.push(void);
        piles::refresh_hand_values(&mut combat.state);
        combat.enqueue_native_effects(vec![Effect { source: Source::None, target: Some(Actor::Player),
            kind: EffectKind::Draw(1) }], None).map_err(|error| format!("Void draw: {error:?}"))?;
    } else if id == CardId::Burn {
        let work = status::end_turn_work(&combat.state);
        combat.enqueue_native_effects(work, None).map_err(|error| format!("Burn end-turn owner: {error:?}"))?;
    } else {
        combat.step_full(&CombatAction::EndTurn).map_err(|error| format!("{id:?} EndTurn: {error:?}"))?;
        resolve_selection(&mut combat)?;
    }
    let zone = |pile: &[OwnedCard]| pile.iter().filter_map(OwnedCard::full)
        .filter(|card| card.id == id).map(|card| card.id).collect::<Vec<_>>();
    Ok(StatusResult {
        playable: legal,
        hp_delta: (id == CardId::Burn).then_some(combat.state.player.hp - before_hp),
        energy_delta: (id == CardId::Void).then_some(combat.state.player.energy - before_energy),
        hand: zone(&combat.state.piles.hand),
        discard: zone(&combat.state.piles.discard),
        exhaust: zone(&combat.state.piles.exhaust),
        fault: combat.fault_record().map(|fault| format!("{fault:?}")),
    })
}

#[test]
fn m26_generated_and_status_every_character() {
    let mut mismatches = Vec::new();
    for character in CHARACTERS {
        for producer in child_producers() {
            match generated_child_delta(character, producer.child, producer.parent, producer.next_turn) {
                Ok((actual, card)) => {
                    let owner = match producer.child {
                        CardId::Shiv | CardId::Safety => Character::Silent,
                        CardId::Miracle | CardId::Smite | CardId::Insight | CardId::ThroughViolence
                            | CardId::Beta | CardId::Omega | CardId::Expunger => Character::Watcher,
                        _ => unreachable!("all child producers have a literal owning color"),
                    };
                    match generated_child_delta(owner, producer.child, producer.parent, producer.next_turn) {
                        Ok((expected, _)) if expected.fault.is_none() && actual == expected => {}
                        Ok((expected, _)) => mismatches.push(format!("{:?}|{}|{character:?}|{expected:?}|{actual:?}",
                            producer.child, card.upgrades)),
                        Err(error) => mismatches.push(format!("{:?}|{}|{character:?}|owner result|{error}",
                            producer.child, card.upgrades)),
                    }
                }
                Err(error) => mismatches.push(format!("{:?}|0|{character:?}|created and resolved|{error}", producer.child)),
            }
        }

        let statuses = [
            (CardId::Burn, StatusResult { playable: false, hp_delta: Some(-2), energy_delta: None,
                hand: vec![CardId::Burn], discard: vec![], exhaust: vec![], fault: None }),
            (CardId::Dazed, StatusResult { playable: false, hp_delta: None, energy_delta: None,
                hand: vec![], discard: vec![], exhaust: vec![CardId::Dazed], fault: None }),
            (CardId::Slimed, StatusResult { playable: true, hp_delta: None, energy_delta: None,
                hand: vec![], discard: vec![CardId::Slimed], exhaust: vec![], fault: None }),
            (CardId::Wound, StatusResult { playable: false, hp_delta: None, energy_delta: None,
                hand: vec![], discard: vec![CardId::Wound], exhaust: vec![], fault: None }),
            (CardId::Void, StatusResult { playable: false, hp_delta: None, energy_delta: Some(-1),
                hand: vec![CardId::Void], discard: vec![], exhaust: vec![], fault: None }),
        ];
        for (status, expected) in statuses {
            match resolve_status(character, status) {
                Ok(actual) if actual == expected => {}
                Ok(actual) => mismatches.push(format!("{status:?}|0|{character:?}|{expected:?}|{actual:?}")),
                Err(error) => mismatches.push(format!("{status:?}|0|{character:?}|{expected:?}|{error}")),
            }
        }
    }
    assert!(mismatches.is_empty(), "m26 generated/status rows (kind|upgrades|character|expected|actual):\n{}",
        mismatches.join("\n"));
}
