use super::super::*;
use super::full_runtime::entered_combat;

fn with_first_hand_card(id: crate::content::CardId, upgrades: u32) -> Combat {
    let mut combat = entered_combat(3, 3, |_| {});
    let card = combat.state.piles.hand[0].full_mut().unwrap();
    card.id = id;
    card.upgrades = upgrades;
    combat
}

fn with_defect_card(id: crate::content::CardId, upgrades: u32) -> Combat {
    let mut combat = with_first_hand_card(id, upgrades);
    combat.state.character = super::super::characters::CharacterState::for_character(
        crate::content::Character::Defect);
    combat
}

fn with_watcher_card(id: crate::content::CardId, upgrades: u32) -> Combat {
    let mut combat = with_first_hand_card(id, upgrades);
    combat.state.character = super::super::characters::CharacterState::for_character(
        crate::content::Character::Watcher);
    combat
}

fn with_silent_card(id: crate::content::CardId, upgrades: u32) -> Combat {
    let mut combat = with_first_hand_card(id, upgrades);
    combat.state.character = super::super::characters::CharacterState::for_character(
        crate::content::Character::Silent);
    combat
}

fn full_power(id: &'static str, amount: i32) -> FullPowerEntry {
    FullPowerEntry { id: FullPowerId(id), state: PowerState {
        amount, skip_first: false, monster_source_grace: false,
    } }
}

#[test]
fn silent_deadly_poison_applies_base_and_upgraded_amount_for_one_energy() {
    for (upgrades, amount) in [(0, 5), (1, 7)] {
        let mut combat = with_silent_card(crate::content::CardId::DeadlyPoison, upgrades);
        let instance = combat.state.piles.hand[0].full().unwrap().instance.0;
        combat.step_full(&CombatAction::PlayCard { instance, target: Some(TargetId(1)) }).unwrap();
        assert_eq!(combat.state.enemies[0].full_powers.iter()
            .find(|power| power.id == FullPowerId("poison")).map(|power| power.state.amount), Some(amount));
        assert_eq!(combat.public_full().unwrap().player.energy, 2);
    }
}

#[test]
fn silent_poisoned_stab_resolves_attack_before_base_or_upgraded_poison() {
    for (upgrades, damage, poison) in [(0, 6, 3), (1, 8, 4)] {
        let mut combat = with_silent_card(crate::content::CardId::PoisonedStab, upgrades);
        combat.state.enemies[0].block = 2;
        let instance = combat.state.piles.hand[0].full().unwrap().instance.0;
        let prepared = super::super::cards::prepare(&combat, &CombatAction::PlayCard {
            instance, target: Some(TargetId(1)),
        }).unwrap();
        assert!(matches!(prepared.effects[0].kind,
            EffectKind::Damage(DamagePayload::Fixed(amount)) if amount == damage));
        assert_eq!(prepared.effects.len(), 2);
        combat.step_full(&CombatAction::PlayCard { instance, target: Some(TargetId(1)) }).unwrap();
        assert_eq!(combat.public_full().unwrap().enemies[0].hp, 48 - (damage - 2));
        assert_eq!(combat.state.enemies[0].full_powers.iter()
            .find(|power| power.id == FullPowerId("poison")).map(|power| power.state.amount), Some(poison));
        assert_eq!(combat.public_full().unwrap().player.energy, 2);
    }
}

#[test]
fn catalyst_permanent_card_state_carries_its_exhaust_flag() {
    let card = super::super::card_state::permanent(crate::content::CardId::Catalyst,
        CardInstanceId(99), 0);
    assert!(card.local.exhaust_on_finish);
}

#[test]
fn catalyst_adds_twice_the_current_poison_and_exhausts_but_absence_is_noop() {
    for (upgrades, start, expected) in [(0, 4, 12), (1, 4, 12), (0, 6_000, 18_000)] {
        let mut combat = with_silent_card(crate::content::CardId::Catalyst, upgrades);
        combat.state.piles.hand[0].full_mut().unwrap().local.exhaust_on_finish = true;
        combat.state.enemies[0].full_powers.push(full_power("poison", start));
        let instance = combat.state.piles.hand[0].full().unwrap().instance.0;
        combat.step_full(&CombatAction::PlayCard { instance, target: Some(TargetId(1)) }).unwrap();
        assert_eq!(combat.state.enemies[0].full_powers.iter()
            .find(|power| power.id == FullPowerId("poison")).map(|power| power.state.amount), Some(expected));
        assert!(combat.state.piles.exhaust.iter().any(|card| card.handle().as_str() == instance.to_string()));
        assert_eq!(combat.public_full().unwrap().player.energy, 2);
    }

    let mut absent = with_silent_card(crate::content::CardId::Catalyst, 0);
    absent.state.piles.hand[0].full_mut().unwrap().local.exhaust_on_finish = true;
    let instance = absent.state.piles.hand[0].full().unwrap().instance.0;
    absent.step_full(&CombatAction::PlayCard { instance, target: Some(TargetId(1)) }).unwrap();
    assert!(!absent.state.enemies[0].full_powers.iter()
        .any(|power| power.id == FullPowerId("poison")));
    assert!(absent.state.piles.exhaust.iter().any(|card| card.handle().as_str() == instance.to_string()));
}

#[test]
fn poison_ticks_at_afflicted_enemy_start_ignoring_block_then_decrements_and_can_kill() {
    let mut combat = with_silent_card(crate::content::CardId::DeadlyPoison, 0);
    combat.state.enemies[0].full_powers.push(full_power("poison", 2));
    combat.state.enemies[0].hp = 10;
    combat.state.enemies[0].block = 8;
    combat.step_full(&CombatAction::EndTurn).unwrap();
    assert_eq!(combat.public_full().unwrap().enemies[0].hp, 8,
        "Poison HP_LOSS bypasses Block at the enemy's own start turn");
    assert_eq!(combat.state.enemies[0].full_powers.iter()
        .find(|power| power.id == FullPowerId("poison")).map(|power| power.state.amount), Some(1));

    let mut lethal = with_silent_card(crate::content::CardId::DeadlyPoison, 0);
    lethal.state.enemies[0].hp = 5;
    lethal.state.enemies[0].full_powers.push(full_power("poison", 5));
    lethal.step_full(&CombatAction::EndTurn).unwrap();
    assert_eq!(lethal.state.enemies[0].hp, 0);
    assert_eq!(lethal.state.outcome, Some(Outcome::Victory));
    assert_eq!(lethal.public_full().unwrap().player.hp, 40,
        "lethal Poison prevents the afflicted enemy's queued Attack");
}

#[test]
fn artifact_prevents_one_poison_application_without_erasing_existing_poison() {
    let mut combat = with_silent_card(crate::content::CardId::DeadlyPoison, 0);
    combat.state.enemies[0].full_powers.extend([
        full_power("poison", 2), full_power("artifact", 1),
    ]);
    let instance = combat.state.piles.hand[0].full().unwrap().instance.0;
    combat.step_full(&CombatAction::PlayCard { instance, target: Some(TargetId(1)) }).unwrap();
    assert_eq!(combat.state.enemies[0].full_powers.iter()
        .find(|power| power.id == FullPowerId("poison")).map(|power| power.state.amount), Some(2));
    assert!(!combat.state.enemies[0].full_powers.iter()
        .any(|power| power.id == FullPowerId("artifact")));
}

#[test]
fn defect_beam_cell_hits_then_applies_vulnerable_and_upgrades_both() {
    for (upgrades, damage, vulnerable) in [(0, 3, 1), (1, 4, 2)] {
        let mut combat = with_defect_card(crate::content::CardId::BeamCell, upgrades);
        let instance = combat.state.piles.hand[0].full().unwrap().instance.0;
        combat.step_full(&CombatAction::PlayCard { instance, target: Some(TargetId(1)) }).unwrap();
        assert_eq!(combat.public_full().unwrap().enemies[0].hp, 48 - damage);
        assert!(combat.state.enemies[0].full_powers.iter()
            .any(|power| power.id == FullPowerId("vulnerable") && power.state.amount == vulnerable));
        assert_eq!(combat.public_full().unwrap().player.energy, 3);
    }
}

#[test]
fn defect_go_for_the_eyes_applies_weak_from_live_attack_intent_and_not_nonattack() {
    for (upgrades, damage, weak) in [(0, 3, 1), (1, 4, 2)] {
        let mut attack = with_defect_card(crate::content::CardId::GoForTheEyes, upgrades);
        attack.state.enemies[0].intent_id = ActorIntentId::Full(FullIntentId::Attack);
        attack.state.enemies[0].selected_move = ActorIntentId::Full(FullIntentId::Attack);
        attack.state.enemies[0].full_damage_entries.get_mut(&FullIntentId::Attack).unwrap().base = 0;
        super::super::effects::refresh_enemy_values(&mut attack.state).unwrap();
        assert_eq!(attack.state.enemies[0].intent_damage, Some(0),
            "a live zero-damage Attack intent qualifies at the follow-up checkpoint");
        let instance = attack.state.piles.hand[0].full().unwrap().instance.0;
        attack.step_full(&CombatAction::PlayCard { instance, target: Some(TargetId(1)) }).unwrap();
        assert_eq!(attack.public_full().unwrap().enemies[0].hp, 48 - damage);
        assert!(attack.state.enemies[0].full_powers.iter()
            .any(|power| power.id == FullPowerId("weak") && power.state.amount == weak));
        assert_eq!(attack.public_full().unwrap().player.energy, 3, "Go for the Eyes costs zero");

        let mut nonattack = with_defect_card(crate::content::CardId::GoForTheEyes, upgrades);
        nonattack.state.enemies[0].intent_id = ActorIntentId::Full(FullIntentId::Incantation);
        nonattack.state.enemies[0].selected_move = ActorIntentId::Full(FullIntentId::Incantation);
        nonattack.state.enemies[0].intent_damage = None;
        let instance = nonattack.state.piles.hand[0].full().unwrap().instance.0;
        nonattack.step_full(&CombatAction::PlayCard { instance, target: Some(TargetId(1)) }).unwrap();
        assert_eq!(nonattack.state.enemies[0].full_powers.iter()
            .find(|power| power.id == FullPowerId("weak")).map(|power| power.state.amount), None);
    }
}

#[test]
fn defect_barrage_repeats_one_captured_hit_per_occupied_orb() {
    use super::super::characters::OrbKind;
    for (upgrades, hit, orbs) in [(0, 4, vec![None, None, None]),
        (0, 4, vec![Some(OrbKind::Frost), None, None]),
        (1, 6, vec![None, None, None]),
        (1, 6, vec![Some(OrbKind::Lightning), None, None]),
        (0, 4, vec![Some(OrbKind::Frost), None, Some(OrbKind::Lightning)]),
        (1, 6, vec![Some(OrbKind::Frost), Some(OrbKind::Frost), Some(OrbKind::Lightning)])] {
        let mut combat = with_defect_card(crate::content::CardId::Barrage, upgrades);
        combat.state.character.orb_slots = orbs;
        combat.state.enemies[0].block = 5;
        let count = combat.state.character.orb_slots.iter().flatten().count();
        let instance = combat.state.piles.hand[0].full().unwrap().instance.0;
        let prepared = super::super::cards::prepare(&combat, &CombatAction::PlayCard {
            instance, target: Some(TargetId(1)),
        }).unwrap();
        assert_eq!(prepared.effects.len(), 1,
            "Barrage defers its occupied-orb count until the action continuation executes");
        assert!(matches!(&prepared.effects[0].kind,
            EffectKind::CardContinuation(super::super::cards::DefectContinuation::BarrageStart { damage })
                if *damage == hit));
        combat.step_full(&CombatAction::PlayCard { instance, target: Some(TargetId(1)) }).unwrap();
        let total = hit * count as i32;
        assert_eq!(combat.public_full().unwrap().enemies[0].hp, 48 - (total - 5).max(0));
        assert_eq!(combat.public_full().unwrap().enemies[0].block, (5 - total).max(0));
        assert_eq!(combat.state.hook_events.iter().filter(|event|
            event.hook == Hook::Damage).count(), count,
            "one independently dispatched Damage hook per occupied orb");
        assert_eq!(combat.public_full().unwrap().player.energy, 2);
        assert!(combat.state.piles.discard.iter().any(|card| card.handle().as_str() == instance.to_string()),
            "the played Barrage instance completes normally");
    }
}

#[test]
fn defect_compile_driver_counts_distinct_live_orb_kinds_after_hit() {
    use super::super::characters::OrbKind;
    for (upgrades, orbs, draws, damage) in [
        (0, vec![None, None, None], 0, 7),
        (1, vec![Some(OrbKind::Frost), Some(OrbKind::Frost), None], 1, 10),
        (0, vec![Some(OrbKind::Frost), Some(OrbKind::Lightning), Some(OrbKind::Frost)], 2, 7),
    ] {
        let mut combat = with_defect_card(crate::content::CardId::CompileDriver, upgrades);
        combat.state.character.orb_slots = orbs;
        let instance = combat.state.piles.hand[0].full().unwrap().instance.0;
        let draw_order = arrange_draw_fixture(&mut combat, instance);
        combat.step_full(&CombatAction::PlayCard { instance, target: Some(TargetId(1)) }).unwrap();
        assert_eq!(combat.public_full().unwrap().enemies[0].hp, 48 - damage);
        assert_eq!(combat.public_full().unwrap().player.energy, 2);
        assert_eq!(combat.public_full().unwrap().hand.iter().map(|card| card.instance.0).collect::<Vec<_>>(),
            draw_order.iter().take(draws).map(|handle| handle.as_str().parse::<u64>().unwrap()).collect::<Vec<_>>());
    }
}

#[test]
fn defect_sweeping_beam_hits_all_living_enemies_then_draws() {
    for (upgrades, damage) in [(0, 6), (1, 9)] {
        let mut combat = with_defect_card(crate::content::CardId::SweepingBeam, upgrades);
        let mut second = combat.state.enemies[0].clone();
        second.instance_id = EnemyHandle::parse("second-enemy").unwrap();
        second.full_ai_rng = Some(super::full_runtime::full_stream(
            super::super::rng::Domain::RunCombatEnemyAi, vec!["run-combat-1", "enemy-scope-2"]));
        second.hp = 50;
        second.max_hp = 50;
        combat.state.enemies.push(second);
        let instance = combat.state.piles.hand[0].full().unwrap().instance.0;
        let draw_order = arrange_draw_fixture(&mut combat, instance);
        let prepared = super::super::cards::prepare(&combat, &CombatAction::PlayCard {
            instance, target: None,
        }).unwrap();
        assert!(matches!(prepared.effects.last().map(|effect| &effect.kind), Some(EffectKind::Draw(1))));
        combat.step_full(&CombatAction::PlayCard { instance, target: None }).unwrap();
        assert_eq!(combat.public_full().unwrap().enemies.iter().map(|enemy| enemy.hp).collect::<Vec<_>>(),
            vec![48 - damage, 50 - damage]);
        assert_eq!(combat.public_full().unwrap().player.energy, 2);
        assert_eq!(combat.public_full().unwrap().hand.iter().map(|card| card.instance.0).collect::<Vec<_>>(),
            draw_order.iter().take(1).map(|handle| handle.as_str().parse::<u64>().unwrap()).collect::<Vec<_>>());
    }
}

#[test]
fn lethal_barrage_stops_after_the_first_hit_and_prunes_remaining_hits() {
    use super::super::characters::OrbKind;
    let mut combat = with_defect_card(crate::content::CardId::Barrage, 0);
    combat.state.enemies[0].hp = 4;
    combat.state.character.orb_slots = vec![Some(OrbKind::Frost), Some(OrbKind::Lightning), None];
    let instance = combat.state.piles.hand[0].full().unwrap().instance.0;
    combat.step_full(&CombatAction::PlayCard { instance, target: Some(TargetId(1)) }).unwrap();
    assert_eq!(combat.state.outcome, Some(Outcome::Victory));
    assert_eq!(combat.public_full().unwrap().enemies[0].hp, 0);
    assert!(combat.state.pending.is_empty(), "terminal cleanup prunes remaining Barrage hits");
    assert_eq!(combat.state.hook_events.iter().filter(|event|
        event.hook == Hook::Damage).count(), 1,
        "terminal cleanup prevents the second queued Barrage hit");
}

#[test]
fn barrage_rejects_a_zero_remaining_hit_continuation_without_damage() {
    use super::super::cards::DefectContinuation;
    let mut combat = with_defect_card(crate::content::CardId::Barrage, 0);
    let instance = combat.state.piles.hand[0].full().unwrap().instance.0;
    let source = Source::Card(super::super::card_state::handle(
        combat.state.piles.hand[0].full().unwrap().instance));
    combat.state.in_play = Some(combat.state.piles.hand.remove(0));
    let enemy = combat.state.enemies[0].instance_id.clone();
    let before_hp = combat.state.enemies[0].hp;
    let status = dispatch_native(&mut combat, Effect { source, target: Some(Actor::Enemy(enemy)),
        kind: EffectKind::CardContinuation(DefectContinuation::BarrageNext { damage: 4, remaining: 0 }) });
    assert_eq!(combat.state.enemies[0].hp, before_hp,
        "zero remaining hits must not apply the captured damage");
    assert_eq!(status, TransitionStatus::Fault,
        "a continuation with no hits left is invalid, not a free extra hit");
    assert_eq!(combat.state.in_play.as_ref().unwrap().full().unwrap().instance.0, instance);
}

fn arrange_draw_fixture(combat: &mut Combat, in_hand: u64) -> Vec<CardHandle> {
    let cards = std::mem::take(&mut combat.state.piles.hand);
    let mut hand = Vec::new();
    let mut draw = Vec::new();
    for card in cards {
        if card.full().unwrap().instance.0 == in_hand { hand.push(card); }
        else { draw.push(card); }
    }
    assert_eq!(hand.len(), 1, "fixture retains exactly the played card in hand");
    assert!(combat.state.piles.draw.is_empty(), "fixture starts from entered three-card deck");
    combat.state.piles.hand = hand;
    combat.state.piles.draw = draw;
    assert_eq!(combat.state.card_ids_issued.len(),
        combat.state.piles.hand.len() + combat.state.piles.draw.len()
            + combat.state.piles.discard.len() + combat.state.piles.exhaust.len());
    super::super::engine::validate_runtime(&combat.state).unwrap();
    combat.public_full().unwrap();
    combat.state.piles.draw.iter().rev().map(|card| card.handle()).collect()
}

fn dispatch_native(combat: &mut Combat, effect: Effect) -> TransitionStatus {
    combat.state.pending.push_back(effect);
    super::super::engine::resolve(&mut combat.state).status
}

fn make_hand_fixture(combat: &mut Combat, count: u64) {
    use super::super::model::OwnedCard;
    combat.state.piles.hand = (1..=count).map(|id| {
        let instance = CardInstanceId(id);
        OwnedCard::Full(super::super::card_state::permanent(
            crate::content::CardId::Strike, instance, 0))
    }).collect();
    combat.state.piles.draw.clear();
    combat.state.piles.discard.clear();
    combat.state.piles.exhaust.clear();
    combat.state.in_play = None;
    combat.state.card_ids_issued = (1..=count)
        .map(|id| CardHandle::parse(&id.to_string()).unwrap()).collect();
}

#[test]
fn iron_wave_orders_block_then_attack_and_pays_once() {
    let mut combat = with_first_hand_card(crate::content::CardId::IronWave, 0);
    combat.step_full(&CombatAction::PlayCard { instance: combat.state.piles.hand[0].full().unwrap().instance.0,
        target: Some(TargetId(1)) }).unwrap();
    let view = combat.public_full().unwrap();
    assert_eq!(view.player.block, 5);
    assert_eq!(view.enemies[0].hp, 43);
    assert_eq!(view.player.energy, 2);
    assert!(view.discard.iter().any(|c| c.id == crate::content::CardId::IronWave));
}

#[test]
fn upgraded_iron_wave_uses_one_upgraded_block_and_damage_value() {
    let mut combat = with_first_hand_card(crate::content::CardId::IronWave, 1);
    combat.step_full(&CombatAction::PlayCard { instance: combat.state.piles.hand[0].full().unwrap().instance.0,
        target: Some(TargetId(1)) }).unwrap();
    let view = combat.public_full().unwrap();
    assert_eq!(view.player.block, 7);
    assert_eq!(view.enemies[0].hp, 41);
    assert_eq!(view.player.energy, 2);
}

#[test]
fn silent_common_cards_resolve_upgrades_and_shiv_creation() {
    let cases = [
        (crate::content::CardId::Backflip, 0, 5, 2),
        (crate::content::CardId::Backflip, 1, 8, 2),
        (crate::content::CardId::Deflect, 0, 4, 0),
        (crate::content::CardId::Deflect, 1, 7, 0),
    ];
    for (id, upgrades, block, draws) in cases {
        let mut combat = with_first_hand_card(id, upgrades);
        let instance = combat.state.piles.hand[0].full().unwrap().instance.0;
        arrange_draw_fixture(&mut combat, instance);
        combat.step_full(&CombatAction::PlayCard { instance, target: None }).unwrap();
        let view = combat.public_full().unwrap();
        assert_eq!(view.player.block, block, "{id:?} upgrade {upgrades}");
        assert_eq!(view.hand.len(), draws as usize, "{id:?} draws");
    }

    for (upgrades, damage) in [(0, 8), (1, 12)] {
        let mut combat = with_first_hand_card(crate::content::CardId::QuickSlash, upgrades);
        let instance = combat.state.piles.hand[0].full().unwrap().instance.0;
        arrange_draw_fixture(&mut combat, instance);
        let prepared = super::super::cards::prepare(&combat, &CombatAction::PlayCard {
            instance, target: Some(TargetId(1)),
        }).unwrap();
        assert!(matches!(&prepared.effects[0].kind, EffectKind::Damage(DamagePayload::Fixed(value)) if *value == damage));
        assert_eq!(prepared.effects[1].kind, EffectKind::Draw(1));
        combat.step_full(&CombatAction::PlayCard { instance, target: Some(TargetId(1)) }).unwrap();
        assert_eq!(combat.public_full().unwrap().enemies[0].hp, 48 - damage);
        assert_eq!(combat.public_full().unwrap().hand.len(), 1);
    }

    for (upgrades, block, shivs) in [(0, 6, 1), (1, 6, 2)] {
        let mut combat = with_first_hand_card(crate::content::CardId::CloakAndDagger, upgrades);
        let instance = combat.state.piles.hand[0].full().unwrap().instance.0;
        let prepared = super::super::cards::prepare(&combat, &CombatAction::PlayCard { instance, target: None }).unwrap();
        assert!(prepared.effects.iter().filter(|effect| matches!(&effect.kind, EffectKind::CreateCard { id: crate::content::CardId::Shiv, .. }))
            .all(|effect| effect.target == Some(Actor::Player)), "generated-card effects target the player");
        combat.step_full(&CombatAction::PlayCard { instance, target: None }).unwrap();
        assert_eq!(combat.public_full().unwrap().player.block, block);
        let produced = combat.state.piles.hand.iter().filter_map(|card| card.full())
            .filter(|card| card.id == crate::content::CardId::Shiv).collect::<Vec<_>>();
        assert_eq!(produced.len(), shivs);
        assert!(produced.iter().all(|card| card.local.exhaust_on_finish));
        assert!(produced.iter().all(|card| matches!(card.origin, crate::combat::CombatCardRef::CombatLocal(_))));
    }

    let mut overflow = with_first_hand_card(crate::content::CardId::CloakAndDagger, 1);
    make_hand_fixture(&mut overflow, 10);
    overflow.state.piles.hand[0].full_mut().unwrap().id = crate::content::CardId::CloakAndDagger;
    overflow.state.piles.hand[0].full_mut().unwrap().upgrades = 1;
    super::super::engine::validate_runtime(&overflow.state).unwrap();
    overflow.step_full(&CombatAction::PlayCard { instance: 1, target: None }).unwrap();
    assert_eq!(overflow.state.piles.hand.iter().filter(|card| card.full().is_some_and(|c| c.id == crate::content::CardId::Shiv)).count(), 1);
    assert_eq!(overflow.state.piles.discard.iter().filter(|card| card.full().is_some_and(|c| c.id == crate::content::CardId::Shiv)).count(), 1,
        "generated hand overflow redirects Shiv to discard");

    let shiv = overflow.state.piles.hand.iter().find(|card| card.full().is_some_and(|c| c.id == crate::content::CardId::Shiv))
        .unwrap().full().unwrap().instance.0;
    overflow.step_full(&CombatAction::PlayCard { instance: shiv, target: Some(TargetId(1)) }).unwrap();
    assert_eq!(overflow.public_full().unwrap().enemies[0].hp, 44, "ordinary Shiv deals four");
    assert!(overflow.state.piles.exhaust.iter().any(|card| card.handle().as_str() == shiv.to_string()),
        "generated Shiv exhausts after play");
}

#[test]
fn dagger_throw_draws_then_selects_drawn_card_to_discard() {
    let mut combat = with_first_hand_card(crate::content::CardId::DaggerThrow, 1);
    let instance = combat.state.piles.hand[0].full().unwrap().instance.0;
    let draw_order = arrange_draw_fixture(&mut combat, instance);
    assert_eq!(draw_order.len(), 2);
    let selected_drawn = draw_order[0].clone();
    let before_rng = combat.state.rng.clone();
    let prepared = super::super::cards::prepare(&combat, &CombatAction::PlayCard {
        instance, target: Some(TargetId(1)),
    }).unwrap();
    assert!(matches!(&prepared.effects[0].kind, EffectKind::Damage(DamagePayload::Fixed(12))));
    assert_eq!(prepared.effects[1].kind, EffectKind::Draw(1));
    assert!(matches!(prepared.effects[2].kind, EffectKind::SelectHandCardsAfterDraw { required: 1, .. }));
    combat.step_full(&CombatAction::PlayCard { instance, target: Some(TargetId(1)) }).unwrap();
    assert_eq!(combat.public_full().unwrap().enemies[0].hp, 36);
    assert_eq!(combat.public_full().unwrap().hand.len(), 1);
    assert_eq!(combat.state.pending_selection.as_ref().unwrap().candidates, vec![selected_drawn.clone()]);
    let selection_rng = combat.state.rng.clone();
    assert_eq!(selection_rng, before_rng, "drawn card and selection consume no RNG");
    combat.step_full(&CombatAction::SelectCard { instance: selected_drawn.as_str().parse().unwrap() }).unwrap();
    assert!(combat.state.piles.discard.iter().any(|card| card.handle() == selected_drawn));
    assert!(combat.state.pending_selection.is_none());
    assert!(combat.state.in_play.is_none(), "parent card resumes and completes once");
    assert_eq!(combat.state.player.energy, 2);
}

#[test]
fn invalid_dagger_throw_target_preserves_full_combat_state() {
    let mut combat = with_first_hand_card(crate::content::CardId::DaggerThrow, 0);
    let instance = combat.state.piles.hand[0].full().unwrap().instance.0;
    let before = combat.state.clone();
    assert_eq!(combat.step_full(&CombatAction::PlayCard { instance, target: Some(TargetId(99)) }),
        Err(crate::run::RunError::InvalidAction));
    assert_eq!(combat.state, before);
}

fn player_strength(combat: &mut Combat, amount: i32) {
    combat.state.player_full_powers.push(FullPowerEntry {
        id: FullPowerId("strength"),
        state: PowerState { amount, skip_first: false, monster_source_grace: false },
    });
}

#[test]
fn clothesline_damage_precedes_upgraded_weak_and_uses_strength() {
    for (upgrades, expected_damage, expected_weak) in [(0, 14, 2), (1, 16, 3)] {
        let mut combat = with_first_hand_card(crate::content::CardId::Clothesline, upgrades);
        player_strength(&mut combat, 2);
        let instance = combat.state.piles.hand[0].full().unwrap().instance.0;
        let prepared = super::super::cards::prepare(&combat, &CombatAction::PlayCard {
            instance, target: Some(TargetId(1)),
        }).unwrap();
        assert!(matches!(&prepared.effects[0].kind, EffectKind::Damage(DamagePayload::Fixed(n)) if *n == expected_damage));
        assert!(matches!(&prepared.effects[1].kind, EffectKind::ApplyFullPower { id: FullPowerId("weak"), amount }
            if *amount == expected_weak));
        combat.step_full(&CombatAction::PlayCard { instance, target: Some(TargetId(1)) }).unwrap();
        let enemy = &combat.public_full().unwrap().enemies[0];
        assert_eq!(enemy.hp, 48 - expected_damage);
        assert!(enemy.powers.iter().any(|power| power.id == FullPowerId("weak") && power.amount == expected_weak));
    }
}

#[test]
fn heavy_blade_multiplies_only_strength_contribution() {
    for (upgrades, expected_damage) in [(0, 20), (1, 24)] {
        let mut combat = with_first_hand_card(crate::content::CardId::HeavyBlade, upgrades);
        player_strength(&mut combat, 2);
        let instance = combat.state.piles.hand[0].full().unwrap().instance.0;
        combat.step_full(&CombatAction::PlayCard { instance, target: Some(TargetId(1)) }).unwrap();
        assert_eq!(combat.public_full().unwrap().enemies[0].hp, 48 - expected_damage);
    }

    let mut high_strength = with_first_hand_card(crate::content::CardId::HeavyBlade, 1);
    player_strength(&mut high_strength, 400);
    high_strength.state.enemies[0].hp = 40;
    high_strength.state.enemies[0].block = 7;
    let instance = high_strength.state.piles.hand[0].full().unwrap().instance.0;
    let prepared = super::super::cards::prepare(&high_strength, &CombatAction::PlayCard {
        instance, target: Some(TargetId(1)),
    }).unwrap();
    assert!(matches!(&prepared.effects[0].kind, EffectKind::Damage(DamagePayload::Fixed(2014))));
    high_strength.step_full(&CombatAction::PlayCard { instance, target: Some(TargetId(1)) }).unwrap();
    assert_eq!(high_strength.state.enemies[0].hp, 0);
    assert_eq!(high_strength.state.enemies[0].block, 0);
    assert_eq!(high_strength.state.outcome, Some(Outcome::Victory));
    assert!(high_strength.state.pending.is_empty(), "terminal settlement prunes later work");
    let exit = high_strength.exit().unwrap();
    assert_eq!(exit.in_play.as_ref().map(|card| card.instance.0), Some(instance),
        "terminal full-combat exit retains the lethal card in its in-play slot");
    let in_other_zone = exit.hand.iter().chain(&exit.draw).chain(&exit.discard).chain(&exit.exhaust)
        .filter(|card| card.instance.0 == instance).count();
    assert_eq!(in_other_zone, 0, "the lethal card is not duplicated into another zone");
}

#[test]
fn lethal_clothesline_does_not_apply_queued_weak() {
    let mut combat = with_first_hand_card(crate::content::CardId::Clothesline, 0);
    combat.state.enemies[0].hp = 12;
    let instance = combat.state.piles.hand[0].full().unwrap().instance.0;
    combat.step_full(&CombatAction::PlayCard { instance, target: Some(TargetId(1)) }).unwrap();
    assert_eq!(combat.state.enemies[0].hp, 0);
    assert!(combat.state.enemies[0].full_powers.iter().all(|power| power.id.0 != "weak"),
        "terminal cleanup prunes the post-hit Weak effect");
}

#[test]
fn perfected_strike_counts_distinct_strike_tags_in_hand_draw_and_discard_only() {
    for (upgrades, expected_damage) in [(0, 14), (1, 18)] {
        let mut combat = with_first_hand_card(crate::content::CardId::PerfectedStrike, upgrades);
        make_hand_fixture(&mut combat, 5);
        let mut cards = std::mem::take(&mut combat.state.piles.hand);
        cards[0].full_mut().unwrap().id = crate::content::CardId::PerfectedStrike;
        cards[0].full_mut().unwrap().upgrades = upgrades;
        cards[1].full_mut().unwrap().id = crate::content::CardId::Thunderclap;
        cards[2].full_mut().unwrap().id = crate::content::CardId::PommelStrike;
        cards[3].full_mut().unwrap().id = crate::content::CardId::TwinStrike;
        cards[4].full_mut().unwrap().id = crate::content::CardId::Strike;
        combat.state.piles.hand = vec![cards.remove(0), cards.remove(0)];
        combat.state.piles.draw = vec![cards.remove(0)];
        combat.state.piles.discard = vec![cards.remove(0)];
        combat.state.piles.exhaust = vec![cards.remove(0)];
        super::super::engine::validate_runtime(&combat.state).unwrap();
        let instance = combat.state.piles.hand[0].full().unwrap().instance.0;
        assert_eq!(combat.state.piles.hand[0].full().unwrap().id, crate::content::CardId::PerfectedStrike);
        let prepared = super::super::cards::prepare(&combat, &CombatAction::PlayCard {
            instance, target: Some(TargetId(1)),
        }).unwrap();
        assert!(matches!(&prepared.effects[0].kind, EffectKind::Damage(DamagePayload::Fixed(n))
            if *n == expected_damage));
        combat.step_full(&CombatAction::PlayCard { instance, target: Some(TargetId(1)) }).unwrap();
        assert_eq!(combat.public_full().unwrap().enemies[0].hp, 48 - expected_damage);
        assert!(matches!(combat.state.in_play, None));
        assert!(combat.state.piles.exhaust.iter().any(|card| card.handle().as_str() == "5"),
            "the STRIKE-tagged card in exhaust is not counted or moved by calculation");
    }
}

#[test]
fn thunderclap_hits_each_living_enemy_before_applying_vulnerable() {
    for (upgrades, damage) in [(0, 4), (1, 7)] {
        let mut combat = with_first_hand_card(crate::content::CardId::Thunderclap, upgrades);
        let mut second = combat.state.enemies[0].clone();
        second.instance_id = EnemyHandle::parse("second-enemy").unwrap();
        second.full_ai_rng = Some(super::full_runtime::full_stream(
            super::super::rng::Domain::RunCombatEnemyAi, vec!["run-combat-1", "enemy-scope-2"]));
        second.hp = 50;
        second.max_hp = 50;
        combat.state.enemies.push(second);
        let instance = combat.state.piles.hand[0].full().unwrap().instance.0;
        let prepared = super::super::cards::prepare(&combat, &CombatAction::PlayCard { instance, target: None }).unwrap();
        assert_eq!(prepared.effects.len(), 4);
        assert!(prepared.effects[..2].iter().all(|effect| matches!(&effect.kind, EffectKind::Damage(DamagePayload::Fixed(n)) if *n == damage)));
        assert!(prepared.effects[2..].iter().all(|effect| matches!(&effect.kind,
            EffectKind::ApplyFullPower { id: FullPowerId("vulnerable"), amount: 1 })));
        combat.step_full(&CombatAction::PlayCard { instance, target: None }).unwrap();
        let view = combat.public_full().unwrap();
        assert_eq!(view.enemies.iter().map(|enemy| enemy.hp).collect::<Vec<_>>(), vec![48 - damage, 50 - damage]);
        assert!(view.enemies.iter().all(|enemy| enemy.powers.iter().any(|power|
            power.id == FullPowerId("vulnerable") && power.amount == 1)));
    }
}

#[test]
fn targeted_ironclad_attacks_reject_invalid_targets_without_mutation() {
    for id in [crate::content::CardId::Clothesline, crate::content::CardId::HeavyBlade,
        crate::content::CardId::PerfectedStrike] {
        let mut combat = with_first_hand_card(id, 0);
        let before = combat.state.clone();
        let instance = combat.state.piles.hand[0].full().unwrap().instance.0;
        assert_eq!(combat.step_full(&CombatAction::PlayCard { instance, target: Some(TargetId(99)) }),
            Err(crate::run::RunError::InvalidAction), "{id:?}");
        assert_eq!(combat.state, before, "{id:?}");
    }
    let mut combat = with_first_hand_card(crate::content::CardId::Thunderclap, 0);
    let before = combat.state.clone();
    let instance = combat.state.piles.hand[0].full().unwrap().instance.0;
    assert_eq!(combat.step_full(&CombatAction::PlayCard { instance, target: Some(TargetId(1)) }),
        Err(crate::run::RunError::InvalidAction));
    assert_eq!(combat.state, before);
}

#[test]
fn invalid_iron_wave_target_does_not_change_combat() {
    let mut combat = with_first_hand_card(crate::content::CardId::IronWave, 0);
    let before = combat.public_full().unwrap();
    assert_eq!(combat.step_full(&CombatAction::PlayCard { instance: combat.state.piles.hand[0].full().unwrap().instance.0,
        target: Some(TargetId(99)) }), Err(crate::run::RunError::InvalidAction));
    assert_eq!(combat.public_full().unwrap(), before);
}

#[test]
fn unknown_full_card_is_not_silently_playable() {
    let mut combat = with_first_hand_card(crate::content::CardId::Unknown("unwired_test_card"), 0);
    assert!(matches!(combat.step_full(&CombatAction::PlayCard {
        instance: combat.state.piles.hand[0].full().unwrap().instance.0, target: None,
    }), Err(crate::run::RunError::Faulted(_))));
}

#[test]
fn colored_full_starter_strikes_and_defends_have_native_effects() {
    for id in [crate::content::CardId::Strike, crate::content::CardId::StrikeGreen,
        crate::content::CardId::StrikeBlue, crate::content::CardId::StrikePurple] {
        let mut combat = with_first_hand_card(id, 0);
        let instance = combat.state.piles.hand[0].full().unwrap().instance.0;
        combat.step_full(&CombatAction::PlayCard { instance, target: Some(TargetId(1)) }).unwrap();
        assert_eq!(combat.public_full().unwrap().enemies[0].hp, 42, "{id:?}");
        assert_eq!(combat.public_full().unwrap().player.energy, 2, "{id:?}");
    }
    for id in [crate::content::CardId::Defend, crate::content::CardId::DefendGreen,
        crate::content::CardId::DefendBlue, crate::content::CardId::DefendPurple] {
        let mut combat = with_first_hand_card(id, 0);
        let instance = combat.state.piles.hand[0].full().unwrap().instance.0;
        combat.step_full(&CombatAction::PlayCard { instance, target: None }).unwrap();
        let view = combat.public_full().unwrap();
        assert_eq!(view.player.block, 5, "{id:?}");
        assert_eq!(view.player.energy, 2, "{id:?}");
    }
}

#[test]
fn defend_preflight_rejects_live_enemy_target_without_state_change() {
    let combat = with_first_hand_card(crate::content::CardId::Defend, 0);
    let before = combat.state.clone();
    let action = CombatAction::PlayCard {
        instance: combat.state.piles.hand[0].full().unwrap().instance.0,
        target: Some(TargetId(1)),
    };
    assert!(matches!(super::super::cards::prepare(&combat, &action),
        Err(crate::run::RunError::InvalidAction)));
    assert_eq!(combat.state, before);
}

#[test]
fn neutralize_applies_weak_after_damage_and_weak_reduces_later_hit() {
    let mut combat = with_first_hand_card(crate::content::CardId::Neutralize, 0);
    let second = combat.state.piles.hand[1].full_mut().unwrap();
    second.id = crate::content::CardId::StrikeGreen;
    let neutralize = combat.state.piles.hand[0].full().unwrap().instance.0;
    combat.step_full(&CombatAction::PlayCard { instance: neutralize, target: Some(TargetId(1)) }).unwrap();
    let after_neutralize = combat.public_full().unwrap();
    assert_eq!(after_neutralize.enemies[0].hp, 45);
    assert!(after_neutralize.enemies[0].powers.iter().any(|p| p.id.0 == "weak" && p.amount == 1));

    let strike = combat.state.piles.hand[0].full().unwrap().instance.0;
    combat.step_full(&CombatAction::PlayCard { instance: strike, target: Some(TargetId(1)) }).unwrap();
    assert_eq!(combat.public_full().unwrap().enemies[0].hp, 39,
        "enemy Weak does not reduce the player's outgoing Strike");

    // Give the Cultist an actual Attack intent so the applied Weak changes its
    // outgoing NORMAL hit, not merely a stored status projection.
    let enemy = &mut combat.state.enemies[0];
    enemy.intent_id = ActorIntentId::Full(FullIntentId::Attack);
    enemy.selected_move = ActorIntentId::Full(FullIntentId::Attack);
    enemy.move_history.push(ActorIntentId::Full(FullIntentId::Attack));
    super::super::effects::refresh_enemy_values(&mut combat.state).unwrap();
    combat.step_full(&CombatAction::EndTurn).unwrap();
    assert_eq!(combat.public_full().unwrap().player.hp, 36,
        "Cultist's 6 NORMAL attack is reduced to 4 by its Weak");
}

#[test]
fn player_weak_reduces_its_own_attack() {
    let mut combat = with_first_hand_card(crate::content::CardId::Strike, 0);
    combat.state.player_full_powers.push(FullPowerEntry {
        id: FullPowerId("weak"), state: PowerState { amount: 1, skip_first: false, monster_source_grace: false },
    });
    let card = combat.state.piles.hand[0].full().unwrap().instance.0;
    combat.step_full(&CombatAction::PlayCard { instance: card, target: Some(TargetId(1)) }).unwrap();
    assert_eq!(combat.public_full().unwrap().enemies[0].hp, 44,
        "6 NORMAL attack with player Weak is floored once to 4");
}

#[test]
fn normal_damage_modifiers_keep_binary32_intermediates_until_one_floor() {
    let mut combat = with_first_hand_card(crate::content::CardId::IronWave, 0);
    combat.state.player_full_powers.push(FullPowerEntry {
        id: FullPowerId("weak"), state: PowerState { amount: 1, skip_first: false, monster_source_grace: false },
    });
    combat.state.character.stance = super::super::characters::Stance::Wrath;
    combat.state.enemies[0].full_powers.push(FullPowerEntry {
        id: FullPowerId("vulnerable"), state: PowerState { amount: 1, skip_first: false, monster_source_grace: false },
    });
    let card = combat.state.piles.hand[0].full().unwrap().instance.0;
    let prepared = super::super::cards::prepare(&combat,
        &CombatAction::PlayCard { instance: card, target: Some(TargetId(1)) }).unwrap();
    assert!(prepared.effects.iter().any(|effect|
        effect.kind == EffectKind::Damage(DamagePayload::Fixed(11))));
}

#[test]
fn end_turn_lightning_kills_last_enemy_before_its_attack() {
    let mut combat = with_first_hand_card(crate::content::CardId::Zap, 1);
    combat.state.character.orb_slots = vec![Some(super::super::characters::OrbKind::Lightning)];
    combat.state.character.focus = 0;
    combat.state.enemies[0].hp = 3;
    combat.state.enemies[0].intent_id = ActorIntentId::Full(FullIntentId::Attack);
    combat.state.enemies[0].selected_move = ActorIntentId::Full(FullIntentId::Attack);
    super::super::effects::refresh_enemy_values(&mut combat.state).unwrap();
    let player_hp = combat.state.player.hp;

    combat.step_full(&CombatAction::EndTurn).unwrap();

    let after = combat.public_full().unwrap();
    assert_eq!(after.enemies[0].hp, 0, "passive Lightning deals 3 before MonsterStart");
    assert_eq!(after.player.hp, player_hp, "a killed last enemy never executes its queued attack");
    assert_eq!(combat.state.character.orb_slots, vec![Some(super::super::characters::OrbKind::Lightning)],
        "passive evokes but does not remove the orb");
    assert!(combat.state.card_rng.as_ref().unwrap().word_index() > 0,
        "a live target consumes the separate card-random stream");
}

#[test]
fn survivor_blocks_then_waits_for_explicit_discard_selection() {
    let mut combat = with_first_hand_card(crate::content::CardId::Survivor, 1);
    let survivor = combat.state.piles.hand[0].full().unwrap().instance.0;
    let selected = combat.state.piles.hand[1].full().unwrap().instance.0;
    combat.step_full(&CombatAction::PlayCard { instance: survivor, target: None }).unwrap();
    let choosing = combat.public_full().unwrap();
    assert_eq!(choosing.player.block, 11);
    assert_eq!(choosing.player.energy, 2);
    assert_eq!(choosing.selection.unwrap().candidates.len(), 2);

    combat.step_full(&CombatAction::SelectCard { instance: selected }).unwrap();
    let after = combat.public_full().unwrap();
    assert_eq!(after.player.energy, 2);
    assert!(after.discard.iter().any(|card| card.instance.0 == selected));
    assert!(after.discard.iter().any(|card| card.instance.0 == survivor));
}

#[test]
fn eruption_hits_neutral_before_wrath_and_doubles_a_followup_attack() {
    let mut combat = with_first_hand_card(crate::content::CardId::Eruption, 0);
    combat.state.piles.hand[1].full_mut().unwrap().id = crate::content::CardId::StrikePurple;
    let eruption = combat.state.piles.hand[0].full().unwrap().instance.0;
    combat.step_full(&CombatAction::PlayCard { instance: eruption, target: Some(TargetId(1)) }).unwrap();
    assert_eq!(combat.public_full().unwrap().enemies[0].hp, 39);
    assert_eq!(combat.state.character.stance, super::super::characters::Stance::Wrath);

    let strike = combat.state.piles.hand[0].full().unwrap().instance.0;
    combat.step_full(&CombatAction::PlayCard { instance: strike, target: Some(TargetId(1)) }).unwrap();
    assert_eq!(combat.public_full().unwrap().enemies[0].hp, 27);
}

#[test]
fn vigilance_enters_calm_then_upgraded_eruption_exits_for_two_energy() {
    let mut combat = with_first_hand_card(crate::content::CardId::Vigilance, 0);
    combat.state.piles.hand[0].full_mut().unwrap().upgrades = 1;
    let eruption_card = combat.state.piles.hand[1].full_mut().unwrap();
    eruption_card.id = crate::content::CardId::Eruption;
    eruption_card.upgrades = 1;
    let vigilance = combat.state.piles.hand[0].full().unwrap().instance.0;
    combat.step_full(&CombatAction::PlayCard { instance: vigilance, target: None }).unwrap();
    assert_eq!(combat.public_full().unwrap().player.block, 12);
    assert_eq!(combat.state.character.stance, super::super::characters::Stance::Calm);

    let eruption = combat.state.piles.hand[0].full().unwrap().instance.0;
    combat.step_full(&CombatAction::PlayCard { instance: eruption, target: Some(TargetId(1)) }).unwrap();
    assert_eq!(combat.public_full().unwrap().enemies[0].hp, 39);
    assert_eq!(combat.public_full().unwrap().player.energy, 2);
    assert_eq!(combat.state.character.stance, super::super::characters::Stance::Wrath);
}

#[test]
fn generated_miracle_is_playable_and_its_generated_exhaust_state_is_used() {
    let mut combat = with_first_hand_card(crate::content::CardId::Strike, 0);
    let produced = Effect { source: Source::None, target: Some(Actor::Player),
        kind: EffectKind::CreateCard { id: crate::content::CardId::Miracle,
            destination: CardDestination::Hand, upgrades: 1 } };
    assert_eq!(dispatch_native(&mut combat, produced), TransitionStatus::Decision);
    let miracle = combat.state.piles.hand.iter().filter_map(|owned| owned.full())
        .find(|card| card.id == crate::content::CardId::Miracle).unwrap().clone();
    assert_eq!(miracle.upgrades, 1);
    assert!(miracle.local.exhaust_on_finish);
    assert_eq!(miracle.origin, CombatCardRef::CombatLocal(miracle.instance.0));
    combat.step_full(&CombatAction::PlayCard { instance: miracle.instance.0, target: None }).unwrap();
    assert_eq!(combat.public_full().unwrap().player.energy, 5);
    assert!(combat.state.piles.exhaust.iter().any(|owned| owned.handle() ==
        super::super::card_state::handle(miracle.instance)));
}

#[test]
fn enemy_generated_slimed_discard_preserves_native_owner_and_exhaust_state() {
    let mut combat = with_first_hand_card(crate::content::CardId::Strike, 0);
    let enemy = combat.state.enemies[0].instance_id.clone();
    let before = combat.state.card_ids_issued.len();
    let produced = Effect { source: Source::Actor(Actor::Enemy(enemy.clone())), target: Some(Actor::Player),
        kind: EffectKind::CreateCard { id: crate::content::CardId::Slimed,
            destination: CardDestination::Discard, upgrades: 0 } };
    assert_eq!(dispatch_native(&mut combat, produced), TransitionStatus::Decision);
    let slimed = combat.state.piles.discard.iter().filter_map(|owned| owned.full())
        .find(|card| card.id == crate::content::CardId::Slimed).unwrap();
    assert!(slimed.local.exhaust_on_finish);
    assert_eq!(slimed.origin, CombatCardRef::CombatLocal(slimed.instance.0));
    assert_eq!(combat.state.card_ids_issued.len(), before + 1);
    assert_eq!(combat.state.enemies[0].instance_id, enemy);
}

#[test]
fn generated_slimed_from_enemy_is_discard_owned_and_exhausts_when_played() {
    let mut combat = with_first_hand_card(crate::content::CardId::Strike, 0);
    let enemy = combat.state.enemies[0].instance_id.clone();
    let produced = Effect { source: Source::Actor(Actor::Enemy(enemy.clone())), target: Some(Actor::Player),
        kind: EffectKind::CreateCard { id: crate::content::CardId::Slimed,
            destination: CardDestination::Hand, upgrades: 0 } };
    assert_eq!(dispatch_native(&mut combat, produced), TransitionStatus::Decision);
    let slimed = combat.state.piles.hand.iter().filter_map(|owned| owned.full())
        .find(|card| card.id == crate::content::CardId::Slimed).unwrap().clone();
    assert!(slimed.local.exhaust_on_finish);
    assert_eq!(slimed.origin, CombatCardRef::CombatLocal(slimed.instance.0));
    combat.step_full(&CombatAction::PlayCard { instance: slimed.instance.0, target: None }).unwrap();
    assert!(combat.state.piles.exhaust.iter().any(|owned| owned.handle() ==
        super::super::card_state::handle(slimed.instance)));
    assert_eq!(combat.state.enemies[0].instance_id, enemy);
}

#[test]
fn generated_hand_overflow_discards_one_owned_card_without_fault() {
    let mut combat = with_first_hand_card(crate::content::CardId::Strike, 0);
    make_hand_fixture(&mut combat, 10);
    assert_eq!(combat.public_full().unwrap().hand.len(), 10, "fixture is valid before effect");
    let effect = Effect { source: Source::Actor(Actor::Enemy(combat.state.enemies[0].instance_id.clone())),
        target: Some(Actor::Player), kind: EffectKind::CreateCard {
            id: crate::content::CardId::Miracle, destination: CardDestination::Hand, upgrades: 1 } };
    assert_eq!(dispatch_native(&mut combat, effect), TransitionStatus::Decision);
    assert!(combat.state.fault.is_none());
    assert_eq!(combat.state.piles.hand.len(), 10);
    let generated = combat.state.piles.discard.iter().filter_map(|owned| owned.full())
        .find(|card| card.id == crate::content::CardId::Miracle).unwrap();
    assert_eq!(generated.upgrades, 1);
    assert!(generated.local.exhaust_on_finish);
    assert_eq!(generated.origin, CombatCardRef::CombatLocal(generated.instance.0));
    assert_eq!(combat.state.card_ids_issued.len(), 11);
}

#[test]
fn generated_draw_card_is_next_card_drawn_without_rng_or_reordering() {
    let mut combat = with_first_hand_card(crate::content::CardId::Strike, 0);
    let played = combat.state.piles.hand[0].full().unwrap().instance.0;
    let existing_order = arrange_draw_fixture(&mut combat, played);
    assert_eq!(existing_order.len(), 2);
    let existing_top = existing_order[0].clone();
    let rng_before = combat.state.rng.clone();
    let effect = Effect { source: Source::None, target: Some(Actor::Player),
        kind: EffectKind::CreateCard { id: crate::content::CardId::Slimed,
            destination: CardDestination::Draw, upgrades: 0 } };
    assert_eq!(dispatch_native(&mut combat, effect), TransitionStatus::Decision);
    let generated = combat.state.piles.draw.last().unwrap().handle();
    assert_ne!(generated, existing_top);
    let draw = Effect { source: Source::None, target: Some(Actor::Player), kind: EffectKind::Draw(2) };
    assert_eq!(dispatch_native(&mut combat, draw), TransitionStatus::Decision);
    assert_eq!(combat.state.piles.hand.len(), 3, "two requested cards were drawn");
    assert_eq!(combat.state.piles.hand[1].handle(), generated);
    assert_eq!(combat.state.piles.hand[2].handle(), existing_top);
    assert_eq!(combat.state.rng, rng_before);
}

#[test]
fn generated_identity_overflow_retains_current_and_pending_work_without_mutation() {
    let mut combat = with_first_hand_card(crate::content::CardId::Strike, 0);
    make_hand_fixture(&mut combat, 1);
    let maximum = CardInstanceId(u64::MAX);
    let max_card = super::super::card_state::permanent(crate::content::CardId::Strike, maximum, 0);
    combat.state.piles.hand[0] = super::super::model::OwnedCard::Full(max_card);
    combat.state.card_ids_issued = [CardHandle::parse(&u64::MAX.to_string()).unwrap()].into();
    assert_eq!(combat.public_full().unwrap().hand[0].instance, maximum);
    let source = Source::Actor(Actor::Enemy(combat.state.enemies[0].instance_id.clone()));
    let attempted = Effect { source: source.clone(), target: Some(Actor::Player),
        kind: EffectKind::CreateCard { id: crate::content::CardId::Slimed,
            destination: CardDestination::Discard, upgrades: 0 } };
    let queued = Effect { source, target: None, kind: EffectKind::Decision };
    let piles_before = combat.state.piles.clone();
    let ids_before = combat.state.card_ids_issued.clone();
    let rng_before = combat.state.rng.clone();
    combat.state.pending.push_back(attempted.clone());
    combat.state.pending.push_back(queued.clone());
    assert_eq!(super::super::engine::resolve(&mut combat.state).status, TransitionStatus::Fault);
    assert_eq!(combat.state.piles, piles_before);
    assert_eq!(combat.state.card_ids_issued, ids_before);
    assert_eq!(combat.state.rng, rng_before);
    assert_eq!(combat.state.current, Some(attempted.clone()));
    let expected_pending: std::collections::VecDeque<Effect> = vec![queued].into();
    assert_eq!(combat.state.pending, expected_pending);
    assert_eq!(combat.state.fault.as_ref().unwrap().kind, FaultKind::Unsupported);
    let fault = combat.state.fault.clone();
    let resolved = combat.state.resolved_effects;
    super::super::engine::resolve(&mut combat.state);
    assert_eq!(combat.state.fault, fault);
    assert_eq!(combat.state.current, Some(attempted));
    assert_eq!(combat.state.resolved_effects, resolved, "faulted work is never retried");
}

#[test]
fn energy_overflow_retains_effect_and_queue_without_wrapping_or_partial_mutation() {
    let mut combat = entered_combat(999, 999, |_| {});
    assert_eq!(combat.public_full().unwrap().player.energy, 999, "fixture is valid");
    let source = Source::Card(super::super::card_state::handle(
        combat.state.piles.hand[0].full().unwrap().instance));
    let attempted = Effect { source: source.clone(), target: Some(Actor::Player), kind: EffectKind::GainEnergy(1) };
    let queued = Effect { source, target: None, kind: EffectKind::Decision };
    let energy_before = combat.state.player.energy;
    let piles_before = combat.state.piles.clone();
    combat.state.pending.push_back(attempted.clone());
    combat.state.pending.push_back(queued.clone());
    assert_eq!(super::super::engine::resolve(&mut combat.state).status, TransitionStatus::Fault);
    assert_eq!(combat.state.player.energy, energy_before);
    assert_eq!(combat.state.piles, piles_before);
    assert_eq!(combat.state.current, Some(attempted));
    let expected_pending: std::collections::VecDeque<Effect> = vec![queued].into();
    assert_eq!(combat.state.pending, expected_pending);
    assert_eq!(combat.state.fault.as_ref().unwrap().kind, FaultKind::Unsupported);
}

#[test]
fn copy_dispatch_rejects_non_in_play_owner_before_issuing_instance() {
    let mut combat = with_first_hand_card(crate::content::CardId::Anger, 1);
    let source = super::super::card_state::handle(combat.state.piles.hand[0].full().unwrap().instance);
    let attempted = Effect { source: Source::Card(source.clone()), target: None,
        kind: EffectKind::CopyCard { source, destination: CardDestination::Discard } };
    let piles_before = combat.state.piles.clone();
    let ids_before = combat.state.card_ids_issued.clone();
    combat.state.pending.push_back(attempted.clone());
    assert_eq!(super::super::engine::resolve(&mut combat.state).status, TransitionStatus::Fault);
    assert_eq!(combat.state.piles, piles_before);
    assert_eq!(combat.state.card_ids_issued, ids_before);
    assert_eq!(combat.state.current, Some(attempted));
    assert_eq!(combat.state.fault.as_ref().unwrap().kind, FaultKind::Invariant);
}

#[test]
fn anger_deals_damage_and_creates_a_distinct_stat_equivalent_combat_copy() {
    let mut combat = with_first_hand_card(crate::content::CardId::Anger, 1);
    combat.state.piles.hand[1].full_mut().unwrap().id = crate::content::CardId::Anger;
    combat.state.piles.hand[1].full_mut().unwrap().upgrades = 2;
    let source = combat.state.piles.hand[0].full().unwrap().clone();
    let sibling = combat.state.piles.hand[1].full().unwrap().clone();
    let source_id = source.instance.0;
    combat.step_full(&CombatAction::PlayCard { instance: source_id, target: Some(TargetId(1)) }).unwrap();
    let view = combat.public_full().unwrap();
    assert_eq!(view.enemies[0].hp, 40, "upgraded Anger deals 8 damage from inherited 48 HP");
    assert_eq!(view.player.energy, 3, "Anger costs zero");
    let copied = combat.state.piles.discard.iter().find_map(|owned| owned.full())
        .expect("Anger copy is in discard");
    assert_eq!(copied.id, crate::content::CardId::Anger);
    assert_eq!(copied.upgrades, source.upgrades);
    assert!(matches!(copied.origin, crate::combat::CombatCardRef::CombatLocal(_)));
    assert_ne!(copied.instance, source.instance);
    assert!(matches!(source.origin, crate::combat::CombatCardRef::Permanent(_)));
    let same_kind_sibling = combat.state.piles.hand.iter().filter_map(|owned| owned.full())
        .find(|live| live.instance == sibling.instance).unwrap();
    assert_eq!(same_kind_sibling.upgrades, 2);
    assert_eq!(same_kind_sibling.origin, sibling.origin);
    assert_ne!(copied.instance, same_kind_sibling.instance);
}

#[test]
fn cleave_hits_each_living_enemy_with_upgraded_damage() {
    let mut combat = with_first_hand_card(crate::content::CardId::Cleave, 1);
    let mut second = combat.state.enemies[0].clone();
    second.instance_id = EnemyHandle::parse("second-enemy").unwrap();
    second.full_ai_rng = Some(super::full_runtime::full_stream(
        super::super::rng::Domain::RunCombatEnemyAi, vec!["run-combat-1", "enemy-scope-2"]));
    second.hp = 50;
    second.max_hp = 50;
    combat.state.enemies.push(second);
    let cleave = combat.state.piles.hand[0].full().unwrap().instance.0;
    combat.step_full(&CombatAction::PlayCard { instance: cleave, target: None }).unwrap();
    let view = combat.public_full().unwrap();
    assert_eq!(view.enemies.iter().map(|enemy| enemy.hp).collect::<Vec<_>>(), vec![37, 39]);
    assert_eq!(view.player.energy, 2);
}

#[test]
fn pommel_strike_orders_damage_before_upgraded_two_card_draw() {
    let mut combat = with_first_hand_card(crate::content::CardId::PommelStrike, 1);
    let pommel = combat.state.piles.hand[0].full().unwrap().instance.0;
    arrange_draw_fixture(&mut combat, pommel);
    let draw_before = combat.state.piles.draw.len();
    combat.step_full(&CombatAction::PlayCard { instance: pommel, target: Some(TargetId(1)) }).unwrap();
    let view = combat.public_full().unwrap();
    assert_eq!(view.enemies[0].hp, 38, "upgraded Pommel Strike deals 10 from inherited 48 HP");
    assert_eq!(view.player.energy, 2);
    assert_eq!(combat.state.piles.draw.len(), draw_before - 2);
    assert_eq!(view.hand.len(), 2, "two cards are drawn after the played card leaves hand");
}

#[test]
fn pommel_damage_resolves_before_native_two_card_draw() {
    let mut combat = with_first_hand_card(crate::content::CardId::PommelStrike, 1);
    let instance = combat.state.piles.hand[0].full().unwrap().instance.0;
    arrange_draw_fixture(&mut combat, instance);
    let action = CombatAction::PlayCard { instance, target: Some(TargetId(1)) };
    let prepared = super::super::cards::prepare(&combat, &action).unwrap();
    assert_eq!(prepared.effects.len(), 2);
    assert!(matches!(&prepared.effects[0].kind, EffectKind::Damage(DamagePayload::Fixed(10))));
    assert_eq!(prepared.effects[1].kind, EffectKind::Draw(2));
    let draw_before = combat.state.piles.draw.len();

    assert_eq!(dispatch_native(&mut combat, prepared.effects[0].clone()), TransitionStatus::Decision);
    assert_eq!(combat.state.enemies[0].hp, 38, "actual 10 damage occurs first");
    assert_eq!(combat.state.piles.draw.len(), draw_before, "damage has not drawn cards");
    assert_eq!(dispatch_native(&mut combat, prepared.effects[1].clone()), TransitionStatus::Decision);
    assert_eq!(combat.state.piles.draw.len(), 0, "the upgraded payload draws two");
    assert_eq!(combat.state.piles.hand.len(), 3, "both distinct cards enter hand");
}

#[test]
fn shrug_it_off_gains_upgraded_block_then_draws_one() {
    let mut combat = with_first_hand_card(crate::content::CardId::ShrugItOff, 1);
    let shrug = combat.state.piles.hand[0].full().unwrap().instance.0;
    arrange_draw_fixture(&mut combat, shrug);
    let draw_before = combat.state.piles.draw.len();
    combat.step_full(&CombatAction::PlayCard { instance: shrug, target: None }).unwrap();
    let view = combat.public_full().unwrap();
    assert_eq!(view.player.block, 11);
    assert_eq!(view.player.energy, 2);
    assert_eq!(combat.state.piles.draw.len(), draw_before - 1);
    assert_eq!(view.hand.len(), 1);
}

#[test]
fn shrug_block_resolves_before_native_one_card_draw() {
    let mut combat = with_first_hand_card(crate::content::CardId::ShrugItOff, 1);
    let instance = combat.state.piles.hand[0].full().unwrap().instance.0;
    arrange_draw_fixture(&mut combat, instance);
    let action = CombatAction::PlayCard { instance, target: None };
    let prepared = super::super::cards::prepare(&combat, &action).unwrap();
    assert_eq!(prepared.effects.len(), 2);
    assert_eq!(prepared.effects[0].kind, EffectKind::GainBlock(11));
    assert_eq!(prepared.effects[1].kind, EffectKind::Draw(1));
    let draw_before = combat.state.piles.draw.len();

    assert_eq!(dispatch_native(&mut combat, prepared.effects[0].clone()), TransitionStatus::Decision);
    assert_eq!(combat.state.player.block, 11, "actual block resolves first");
    assert_eq!(combat.state.piles.draw.len(), draw_before, "block has not drawn a card");
    assert_eq!(dispatch_native(&mut combat, prepared.effects[1].clone()), TransitionStatus::Decision);
    assert_eq!(combat.state.piles.draw.len(), 1, "the native payload draws one");
    assert_eq!(combat.state.piles.hand.len(), 2);
}

#[test]
fn twin_strike_resolves_two_separate_upgraded_hits() {
    let mut combat = with_first_hand_card(crate::content::CardId::TwinStrike, 1);
    let twin = combat.state.piles.hand[0].full().unwrap().instance.0;
    combat.step_full(&CombatAction::PlayCard { instance: twin, target: Some(TargetId(1)) }).unwrap();
    assert_eq!(combat.public_full().unwrap().enemies[0].hp, 34, "two separate 7-damage hits from inherited 48 HP");
    assert_eq!(combat.public_full().unwrap().player.energy, 2);
}

#[test]
fn twin_strike_stops_after_first_hit_kills_target() {
    let mut combat = with_first_hand_card(crate::content::CardId::TwinStrike, 0);
    combat.state.enemies[0].hp = 5;
    let player_hp = combat.state.player.hp;
    let twin = combat.state.piles.hand[0].full().unwrap().instance.0;
    combat.step_full(&CombatAction::PlayCard { instance: twin, target: Some(TargetId(1)) }).unwrap();
    assert_eq!(combat.public_full().unwrap().enemies[0].hp, 0);
    assert_eq!(combat.public_full().unwrap().player.hp, player_hp, "lethal first hit prevents enemy action");
}

#[test]
fn anger_invalid_target_does_not_pay_or_create_a_copy() {
    let mut combat = with_first_hand_card(crate::content::CardId::Anger, 0);
    let before = combat.public_full().unwrap();
    let instance = combat.state.piles.hand[0].full().unwrap().instance.0;
    assert_eq!(combat.step_full(&CombatAction::PlayCard { instance, target: Some(TargetId(99)) }),
        Err(crate::run::RunError::InvalidAction));
    assert_eq!(combat.public_full().unwrap(), before);
    assert!(combat.state.piles.discard.is_empty());
}

#[test]
fn anger_base_deals_six_and_creates_one_combat_local_copy() {
    let mut combat = with_first_hand_card(crate::content::CardId::Anger, 0);
    let source = combat.state.piles.hand[0].full().unwrap().instance.0;
    combat.step_full(&CombatAction::PlayCard { instance: source, target: Some(TargetId(1)) }).unwrap();
    assert_eq!(combat.public_full().unwrap().enemies[0].hp, 42);
    assert_eq!(combat.state.piles.discard.iter().filter(|card|
        card.full().is_some_and(|live| live.id == crate::content::CardId::Anger)).count(), 2,
        "played source and its combat-local copy are both in discard");
}

#[test]
fn cleave_base_deals_eight_to_each_enemy() {
    let mut combat = with_first_hand_card(crate::content::CardId::Cleave, 0);
    let mut second = combat.state.enemies[0].clone();
    second.instance_id = EnemyHandle::parse("second-enemy").unwrap();
    second.full_ai_rng = Some(super::full_runtime::full_stream(
        super::super::rng::Domain::RunCombatEnemyAi, vec!["run-combat-1", "enemy-scope-2"]));
    combat.state.enemies.push(second);
    let cleave = combat.state.piles.hand[0].full().unwrap().instance.0;
    combat.step_full(&CombatAction::PlayCard { instance: cleave, target: None }).unwrap();
    assert_eq!(combat.public_full().unwrap().enemies.iter().map(|enemy| enemy.hp).collect::<Vec<_>>(), vec![40, 40]);
}

#[test]
fn pommel_strike_base_deals_nine_then_draws_one() {
    let mut combat = with_first_hand_card(crate::content::CardId::PommelStrike, 0);
    let pommel = combat.state.piles.hand[0].full().unwrap().instance.0;
    arrange_draw_fixture(&mut combat, pommel);
    let draw_before = combat.state.piles.draw.len();
    combat.step_full(&CombatAction::PlayCard { instance: pommel, target: Some(TargetId(1)) }).unwrap();
    assert_eq!(combat.public_full().unwrap().enemies[0].hp, 39);
    assert_eq!(combat.state.piles.draw.len(), draw_before - 1);
    assert_eq!(combat.public_full().unwrap().hand.len(), 1);
}

#[test]
fn shrug_it_off_base_gains_eight_block_then_draws_one() {
    let mut combat = with_first_hand_card(crate::content::CardId::ShrugItOff, 0);
    let shrug = combat.state.piles.hand[0].full().unwrap().instance.0;
    arrange_draw_fixture(&mut combat, shrug);
    let draw_before = combat.state.piles.draw.len();
    combat.step_full(&CombatAction::PlayCard { instance: shrug, target: None }).unwrap();
    assert_eq!(combat.public_full().unwrap().player.block, 8);
    assert_eq!(combat.state.piles.draw.len(), draw_before - 1);
}

#[test]
fn miracle_gains_energy_then_exhausts_and_slimed_is_playable_then_exhausts() {
    let mut miracle_combat = with_first_hand_card(crate::content::CardId::Miracle, 0);
    miracle_combat.state.piles.hand[0].full_mut().unwrap().local.exhaust_on_finish = true;
    let miracle = miracle_combat.state.piles.hand[0].full().unwrap().instance.0;
    miracle_combat.step_full(&CombatAction::PlayCard { instance: miracle, target: None }).unwrap();
    assert_eq!(miracle_combat.public_full().unwrap().player.energy, 4);
    assert!(miracle_combat.state.piles.exhaust.iter().any(|card|
        card.full().is_some_and(|live| live.instance.0 == miracle)));

    let mut slimed_combat = with_first_hand_card(crate::content::CardId::Slimed, 0);
    slimed_combat.state.piles.hand[0].full_mut().unwrap().local.exhaust_on_finish = true;
    let slimed = slimed_combat.state.piles.hand[0].full().unwrap().instance.0;
    slimed_combat.step_full(&CombatAction::PlayCard { instance: slimed, target: None }).unwrap();
    assert_eq!(slimed_combat.public_full().unwrap().player.energy, 2);
    assert!(slimed_combat.state.piles.exhaust.iter().any(|card|
        card.full().is_some_and(|live| live.instance.0 == slimed)));
}

#[test]
fn zap_channels_lightning_and_dualcast_invokes_then_removes_it() {
    let mut combat = with_first_hand_card(crate::content::CardId::Zap, 1);
    combat.state.character.orb_slots = vec![None; 3];
    let zap = combat.state.piles.hand[0].full().unwrap().instance.0;
    combat.step_full(&CombatAction::PlayCard { instance: zap, target: None }).unwrap();
    assert_eq!(combat.public_full().unwrap().player.energy, 3);
    assert_eq!(combat.state.character.orb_slots[0], Some(super::super::characters::OrbKind::Lightning));

    combat.state.piles.hand[0].full_mut().unwrap().id = crate::content::CardId::Dualcast;
    combat.state.character.focus = 2;
    let dualcast = combat.state.piles.hand[0].full().unwrap().instance.0;
    combat.step_full(&CombatAction::PlayCard { instance: dualcast, target: None }).unwrap();
    assert_eq!(combat.public_full().unwrap().enemies[0].hp, 28);
    assert_eq!(combat.state.character.orb_slots[0], None);
}

#[test]
fn defect_cold_snap_and_ball_lightning_order_damage_before_channel() {
    use super::super::characters::{CharacterEffect, OrbKind};
    for (id, damage, orb) in [
        (crate::content::CardId::ColdSnap, 6, OrbKind::Frost),
        (crate::content::CardId::BallLightning, 7, OrbKind::Lightning),
    ] {
        let mut combat = with_defect_card(id, 0);
        let instance = combat.state.piles.hand[0].full().unwrap().instance.0;
        let prepared = super::super::cards::prepare(&combat, &CombatAction::PlayCard {
            instance, target: Some(TargetId(1)),
        }).unwrap();
        assert!(matches!(&prepared.effects[0].kind,
            EffectKind::Damage(DamagePayload::Fixed(value)) if *value == damage), "{id:?}");
        assert!(matches!(&prepared.effects[1].kind,
            EffectKind::Character(CharacterEffect::Channel(actual)) if *actual == orb), "{id:?}");
        if orb == OrbKind::Frost {
            let diagnostic = super::differential::effect(&Effect { source: Source::None,
                target: Some(Actor::Player), kind: EffectKind::Character(CharacterEffect::Channel(orb)) });
            assert_eq!(diagnostic["payload"]["channel"], "frost");
        }
        let rng_before = combat.state.card_rng.as_ref().unwrap().clone();
        combat.step_full(&CombatAction::PlayCard { instance, target: Some(TargetId(1)) }).unwrap();
        assert_eq!(combat.public_full().unwrap().enemies[0].hp, 48 - damage, "{id:?}");
        assert_eq!(combat.state.character.orb_slots[0], Some(orb), "{id:?}");
        assert_eq!(combat.state.player.energy, 2, "{id:?}");
        assert_eq!(combat.state.card_rng.as_ref().unwrap(), &rng_before,
            "ordinary channeling does not pick a Lightning target");
    }
}

#[test]
fn defect_cold_snap_and_ball_lightning_use_qualified_upgraded_values() {
    use super::super::characters::OrbKind;
    for (id, damage, orb) in [
        (crate::content::CardId::ColdSnap, 9, OrbKind::Frost),
        (crate::content::CardId::BallLightning, 10, OrbKind::Lightning),
    ] {
        let mut combat = with_defect_card(id, 1);
        let instance = combat.state.piles.hand[0].full().unwrap().instance.0;
        combat.step_full(&CombatAction::PlayCard { instance, target: Some(TargetId(1)) }).unwrap();
        assert_eq!(combat.public_full().unwrap().enemies[0].hp, 48 - damage, "{id:?}");
        assert_eq!(combat.state.character.orb_slots[0], Some(orb), "{id:?}");
        assert_eq!(combat.state.player.energy, 2, "{id:?} retains cost 1 when upgraded");
    }
}

#[test]
fn defect_coolheaded_channels_before_drawing_its_base_or_upgraded_count() {
    use super::super::characters::{CharacterEffect, OrbKind};
    for (upgrades, count) in [(0, 1), (1, 2)] {
        let mut combat = with_defect_card(crate::content::CardId::Coolheaded, upgrades);
        let instance = combat.state.piles.hand[0].full().unwrap().instance.0;
        let draw_order = arrange_draw_fixture(&mut combat, instance);
        let prepared = super::super::cards::prepare(&combat, &CombatAction::PlayCard {
            instance, target: None,
        }).unwrap();
        assert!(matches!(&prepared.effects[0].kind,
            EffectKind::Character(CharacterEffect::Channel(OrbKind::Frost))));
        assert!(matches!(&prepared.effects[1].kind, EffectKind::Draw(n) if *n == count));
        combat.step_full(&CombatAction::PlayCard { instance, target: None }).unwrap();
        assert_eq!(combat.state.character.orb_slots[0], Some(OrbKind::Frost));
        assert_eq!(combat.public_full().unwrap().hand.iter().map(|card| card.instance.0)
            .collect::<Vec<_>>(), draw_order.iter().take(count as usize)
            .map(|handle| handle.as_str().parse::<u64>().unwrap()).collect::<Vec<_>>());
        assert_eq!(combat.public_full().unwrap().player.energy, 2);
    }
}

#[test]
fn defect_leap_gains_qualified_base_and_upgraded_block() {
    for (upgrades, block) in [(0, 9), (1, 12)] {
        let mut combat = with_defect_card(crate::content::CardId::Leap, upgrades);
        let instance = combat.state.piles.hand[0].full().unwrap().instance.0;
        combat.step_full(&CombatAction::PlayCard { instance, target: None }).unwrap();
        assert_eq!(combat.public_full().unwrap().player.block, block);
        assert_eq!(combat.public_full().unwrap().player.energy, 2);
    }
}

#[test]
fn frost_orb_passive_and_evoke_supply_focus_adjusted_block() {
    use super::super::characters::{CharacterEffect, OrbKind};
    let mut passive = with_defect_card(crate::content::CardId::Leap, 0);
    passive.state.character.focus = 3;
    passive.state.character.orb_slots = vec![Some(OrbKind::Frost), None, None];
    dispatch_native(&mut passive, Effect { source: Source::None, target: Some(Actor::Player),
        kind: EffectKind::Character(CharacterEffect::EndTurn) });
    assert_eq!(passive.public_full().unwrap().player.block, 5, "Frost passive is 2+Focus");
    assert_eq!(passive.state.character.orb_slots, vec![Some(OrbKind::Frost), None, None]);

    let mut evoke = with_defect_card(crate::content::CardId::Leap, 0);
    evoke.state.character.focus = 3;
    evoke.state.character.orb_slots = vec![Some(OrbKind::Frost), None, None];
    dispatch_native(&mut evoke, Effect { source: Source::None, target: Some(Actor::Player),
        kind: EffectKind::Character(CharacterEffect::EvokeFront { remove: true }) });
    assert_eq!(evoke.public_full().unwrap().player.block, 8, "Frost evoke is 5+Focus");
    assert_eq!(evoke.state.character.orb_slots, vec![None, None, None]);
}

#[test]
fn watcher_empty_fist_hits_before_neutral_and_calm_exit_energy() {
    use super::super::characters::{CharacterEffect, Stance};
    for (upgrades, damage) in [(0, 9), (1, 14)] {
        let mut combat = with_watcher_card(crate::content::CardId::EmptyFist, upgrades);
        combat.state.character.stance = Stance::Calm;
        let instance = combat.state.piles.hand[0].full().unwrap().instance.0;
        let prepared = super::super::cards::prepare(&combat, &CombatAction::PlayCard {
            instance, target: Some(TargetId(1)),
        }).unwrap();
        assert!(matches!(&prepared.effects[0].kind,
            EffectKind::Damage(DamagePayload::Fixed(amount)) if *amount == damage));
        assert!(matches!(&prepared.effects[1].kind,
            EffectKind::Character(CharacterEffect::Stance(Stance::Neutral))));
        combat.step_full(&CombatAction::PlayCard { instance, target: Some(TargetId(1)) }).unwrap();
        assert_eq!(combat.public_full().unwrap().enemies[0].hp, 48 - damage);
        assert_eq!(combat.state.character.stance, Stance::Neutral);
        assert_eq!(combat.public_full().unwrap().player.energy, 4,
            "the Calm exit grants 2 after the cost-1 play");
    }

    let mut wrath = with_watcher_card(crate::content::CardId::EmptyFist, 0);
    wrath.state.character.stance = Stance::Wrath;
    let instance = wrath.state.piles.hand[0].full().unwrap().instance.0;
    let prepared = super::super::cards::prepare(&wrath, &CombatAction::PlayCard {
        instance, target: Some(TargetId(1)),
    }).unwrap();
    assert!(matches!(&prepared.effects[0].kind,
        EffectKind::Damage(DamagePayload::Fixed(18))), "Wrath modifies the hit before stance exit");
    wrath.step_full(&CombatAction::PlayCard { instance, target: Some(TargetId(1)) }).unwrap();
    assert_eq!(wrath.public_full().unwrap().enemies[0].hp, 30);
    assert_eq!(wrath.state.character.stance, Stance::Neutral);
    assert_eq!(wrath.public_full().unwrap().player.energy, 2, "Wrath exit grants no Calm energy");
}

#[test]
fn watcher_empty_body_and_mind_resolve_before_calm_exit() {
    use super::super::characters::{CharacterEffect, Stance};
    for (upgrades, block) in [(0, 7), (1, 10)] {
        let mut combat = with_watcher_card(crate::content::CardId::EmptyBody, upgrades);
        combat.state.character.stance = Stance::Calm;
        let instance = combat.state.piles.hand[0].full().unwrap().instance.0;
        let prepared = super::super::cards::prepare(&combat, &CombatAction::PlayCard {
            instance, target: None,
        }).unwrap();
        assert!(matches!(&prepared.effects[0].kind, EffectKind::GainBlock(n) if *n == block));
        assert!(matches!(&prepared.effects[1].kind,
            EffectKind::Character(CharacterEffect::Stance(Stance::Neutral))));
        combat.step_full(&CombatAction::PlayCard { instance, target: None }).unwrap();
        assert_eq!(combat.public_full().unwrap().player.block, block);
        assert_eq!(combat.state.character.stance, Stance::Neutral);
        assert_eq!(combat.public_full().unwrap().player.energy, 4);
    }

    for (upgrades, count) in [(0, 2), (1, 3)] {
        let mut combat = with_watcher_card(crate::content::CardId::EmptyMind, upgrades);
        combat.state.character.stance = Stance::Calm;
        let instance = combat.state.piles.hand[0].full().unwrap().instance.0;
        let draw_order = arrange_draw_fixture(&mut combat, instance);
        let prepared = super::super::cards::prepare(&combat, &CombatAction::PlayCard {
            instance, target: None,
        }).unwrap();
        assert!(matches!(&prepared.effects[0].kind, EffectKind::Draw(n) if *n == count));
        assert!(matches!(&prepared.effects[1].kind,
            EffectKind::Character(CharacterEffect::Stance(Stance::Neutral))));
        combat.step_full(&CombatAction::PlayCard { instance, target: None }).unwrap();
        assert_eq!(combat.public_full().unwrap().hand.iter().map(|card| card.instance.0)
            .collect::<Vec<_>>(), draw_order.iter().take(count as usize)
            .map(|handle| handle.as_str().parse::<u64>().unwrap()).collect::<Vec<_>>());
        assert_eq!(combat.state.character.stance, Stance::Neutral);
        assert_eq!(combat.public_full().unwrap().player.energy, 4);
    }
}

#[test]
fn watcher_upgraded_empty_mind_draws_only_to_the_hand_limit_before_calm_exit() {
    use super::super::characters::Stance;
    let mut combat = with_watcher_card(crate::content::CardId::EmptyMind, 1);
    combat.state.character.stance = Stance::Calm;
    make_hand_fixture(&mut combat, 12);
    combat.state.piles.hand[0].full_mut().unwrap().id = crate::content::CardId::EmptyMind;
    combat.state.piles.hand[0].full_mut().unwrap().upgrades = 1;
    let instance = combat.state.piles.hand[0].full().unwrap().instance.0;
    let mut cards = std::mem::take(&mut combat.state.piles.hand);
    let first_draw = cards.pop().unwrap();
    let second_draw = cards.pop().unwrap();
    combat.state.piles.hand = cards;
    combat.state.piles.draw = vec![first_draw, second_draw];
    super::super::engine::validate_runtime(&combat.state).unwrap();
    let drawn = combat.state.piles.draw.last().unwrap().handle();
    combat.step_full(&CombatAction::PlayCard { instance, target: None }).unwrap();
    assert_eq!(combat.public_full().unwrap().hand.len(), 10);
    assert_eq!(combat.state.piles.draw.len(), 1);
    assert!(combat.state.piles.hand.iter().any(|card| card.handle() == drawn));
    assert_eq!(combat.state.character.stance, Stance::Neutral);
    assert_eq!(combat.public_full().unwrap().player.energy, 4);
}

#[test]
fn watcher_consecrate_hits_each_living_enemy_without_a_target() {
    for (upgrades, damage) in [(0, 10), (1, 16)] {
        let mut combat = with_watcher_card(crate::content::CardId::Consecrate, upgrades);
        combat.state.character.stance = super::super::characters::Stance::Wrath;
        let mut second = combat.state.enemies[0].clone();
        second.instance_id = EnemyHandle::parse("second-enemy").unwrap();
        second.full_ai_rng = Some(super::full_runtime::full_stream(
            super::super::rng::Domain::RunCombatEnemyAi, vec!["run-combat-1", "enemy-scope-2"]));
        second.hp = 50;
        second.max_hp = 50;
        combat.state.enemies.push(second);
        let instance = combat.state.piles.hand[0].full().unwrap().instance.0;
        let prepared = super::super::cards::prepare(&combat, &CombatAction::PlayCard {
            instance, target: None,
        }).unwrap();
        assert_eq!(prepared.effects.len(), 2);
        assert!(prepared.effects.iter().all(|effect|
            matches!(&effect.kind, EffectKind::Damage(DamagePayload::Fixed(n)) if *n == damage)));
        combat.step_full(&CombatAction::PlayCard { instance, target: None }).unwrap();
        let view = combat.public_full().unwrap();
        assert_eq!(view.enemies.iter().map(|enemy| enemy.hp).collect::<Vec<_>>(),
            vec![48 - damage, 50 - damage]);
        assert_eq!(view.player.energy, 3, "Consecrate costs zero");
    }
}

#[test]
fn watcher_lethal_empty_fist_retains_wrath_when_neutral_exit_is_pruned() {
    use super::super::characters::Stance;
    let mut combat = with_watcher_card(crate::content::CardId::EmptyFist, 0);
    combat.state.character.stance = Stance::Wrath;
    combat.state.enemies[0].hp = 18;
    let instance = combat.state.piles.hand[0].full().unwrap().instance.0;
    combat.step_full(&CombatAction::PlayCard { instance, target: Some(TargetId(1)) }).unwrap();
    assert_eq!(combat.state.enemies[0].hp, 0);
    assert_eq!(combat.state.outcome, Some(Outcome::Victory));
    assert_eq!(combat.state.character.stance, Stance::Wrath,
        "lethal damage interrupts the later Neutral request");
    assert!(combat.state.pending.is_empty());
}

#[test]
fn watcher_neutral_request_from_neutral_does_not_grant_calm_exit_energy() {
    let mut combat = with_watcher_card(crate::content::CardId::EmptyBody, 0);
    let instance = combat.state.piles.hand[0].full().unwrap().instance.0;
    combat.step_full(&CombatAction::PlayCard { instance, target: None }).unwrap();
    assert_eq!(combat.state.character.stance, super::super::characters::Stance::Neutral);
    assert_eq!(combat.public_full().unwrap().player.energy, 2,
        "same-stance Neutral request grants no exit energy");
}

#[test]
fn watcher_cards_reject_invalid_targets_without_mutation() {
    for (id, target) in [
        (crate::content::CardId::EmptyFist, None),
        (crate::content::CardId::EmptyFist, Some(TargetId(99))),
        (crate::content::CardId::EmptyBody, Some(TargetId(1))),
        (crate::content::CardId::EmptyMind, Some(TargetId(1))),
        (crate::content::CardId::Consecrate, Some(TargetId(1))),
    ] {
        let mut combat = with_watcher_card(id, 0);
        let instance = combat.state.piles.hand[0].full().unwrap().instance.0;
        let before = combat.state.clone();
        assert_eq!(combat.step_full(&CombatAction::PlayCard { instance, target }),
            Err(crate::run::RunError::InvalidAction), "{id:?} target {target:?}");
        assert_eq!(combat.state, before, "{id:?} target {target:?}");
    }
}

#[test]
fn frost_focus_passive_and_evoke_use_mixed_orb_order_and_channel_overflow() {
    use super::super::characters::{CharacterEffect, OrbKind};
    let mut combat = with_defect_card(crate::content::CardId::Coolheaded, 1);
    combat.state.character.focus = 1;
    combat.state.character.orb_slots = vec![Some(OrbKind::Frost), Some(OrbKind::Lightning), Some(OrbKind::Frost)];
    let instance = combat.state.piles.hand[0].full().unwrap().instance.0;
    let draw_order = arrange_draw_fixture(&mut combat, instance);
    let before_rng = combat.state.card_rng.as_ref().unwrap().clone();
    let prepared = super::super::cards::prepare(&combat, &CombatAction::PlayCard {
        instance, target: None,
    }).unwrap();
    assert!(matches!(&prepared.effects[0].kind,
        EffectKind::Character(CharacterEffect::Channel(OrbKind::Frost))));
    assert!(matches!(&prepared.effects[1].kind, EffectKind::Draw(2)));
    combat.step_full(&CombatAction::PlayCard { instance, target: None }).unwrap();
    assert_eq!(combat.public_full().unwrap().player.block, 6, "front Frost evokes for 5+Focus");
    assert_eq!(combat.state.character.orb_slots, vec![Some(OrbKind::Lightning), Some(OrbKind::Frost), Some(OrbKind::Frost)]);
    assert_eq!(combat.state.card_rng.as_ref().unwrap(), &before_rng,
        "evoking Frost during channel overflow does not run Lightning");
    assert_eq!(combat.public_full().unwrap().hand.iter().map(|card| card.instance.0)
        .collect::<Vec<_>>(), draw_order.iter().take(2)
        .map(|handle| handle.as_str().parse::<u64>().unwrap()).collect::<Vec<_>>());

    let mut passive = with_defect_card(crate::content::CardId::Leap, 0);
    passive.state.character.focus = 2;
    passive.state.character.orb_slots = vec![Some(OrbKind::Frost), Some(OrbKind::Lightning), Some(OrbKind::Frost)];
    dispatch_native(&mut passive, Effect { source: Source::None, target: Some(Actor::Player),
        kind: EffectKind::Character(CharacterEffect::EndTurn) });
    assert_eq!(passive.public_full().unwrap().player.block, 8, "two Frost passives each supply 2+Focus Block");
    assert_eq!(passive.public_full().unwrap().enemies[0].hp, 43, "existing Lightning passive remains 3+Focus damage");
    assert_eq!(passive.state.character.orb_slots, vec![Some(OrbKind::Frost), Some(OrbKind::Lightning), Some(OrbKind::Frost)]);
}

#[test]
fn upgraded_coolheaded_respects_ten_card_hand_limit() {
    let mut combat = with_defect_card(crate::content::CardId::Coolheaded, 1);
    make_hand_fixture(&mut combat, 12);
    combat.state.piles.hand[0].full_mut().unwrap().id = crate::content::CardId::Coolheaded;
    combat.state.piles.hand[0].full_mut().unwrap().upgrades = 1;
    let instance = combat.state.piles.hand[0].full().unwrap().instance.0;
    let mut cards = std::mem::take(&mut combat.state.piles.hand);
    let first_draw = cards.pop().unwrap();
    let second_draw = cards.pop().unwrap();
    combat.state.piles.hand = cards;
    combat.state.piles.draw = vec![first_draw, second_draw];
    super::super::engine::validate_runtime(&combat.state).unwrap();
    let drawn = combat.state.piles.draw.last().unwrap().handle();
    combat.step_full(&CombatAction::PlayCard { instance, target: None }).unwrap();
    assert_eq!(combat.public_full().unwrap().hand.len(), 10);
    assert_eq!(combat.state.piles.draw.len(), 1, "the second requested draw remains when hand is full");
    assert!(combat.state.piles.hand.iter().any(|card| card.handle() == drawn));
}

#[test]
fn defect_cards_reject_wrong_or_invalid_targets_without_mutation() {
    for (id, target) in [
        (crate::content::CardId::ColdSnap, None),
        (crate::content::CardId::ColdSnap, Some(TargetId(99))),
        (crate::content::CardId::BallLightning, None),
        (crate::content::CardId::BallLightning, Some(TargetId(99))),
        (crate::content::CardId::Coolheaded, Some(TargetId(1))),
        (crate::content::CardId::Leap, Some(TargetId(1))),
        (crate::content::CardId::BeamCell, None),
        (crate::content::CardId::BeamCell, Some(TargetId(99))),
        (crate::content::CardId::GoForTheEyes, None),
        (crate::content::CardId::GoForTheEyes, Some(TargetId(99))),
        (crate::content::CardId::Barrage, None),
        (crate::content::CardId::Barrage, Some(TargetId(99))),
        (crate::content::CardId::CompileDriver, None),
        (crate::content::CardId::CompileDriver, Some(TargetId(99))),
        (crate::content::CardId::SweepingBeam, Some(TargetId(1))),
    ] {
        let mut combat = with_defect_card(id, 0);
        let instance = combat.state.piles.hand[0].full().unwrap().instance.0;
        let before = combat.state.clone();
        assert_eq!(combat.step_full(&CombatAction::PlayCard { instance, target }),
            Err(crate::run::RunError::InvalidAction), "{id:?} target {target:?}");
        assert_eq!(combat.state, before, "{id:?} target {target:?}");
    }
}

#[test]
fn lethal_cold_snap_prunes_its_later_frost_channel() {
    let mut combat = with_defect_card(crate::content::CardId::ColdSnap, 0);
    combat.state.enemies[0].hp = 6;
    let instance = combat.state.piles.hand[0].full().unwrap().instance.0;
    combat.step_full(&CombatAction::PlayCard { instance, target: Some(TargetId(1)) }).unwrap();
    assert_eq!(combat.state.enemies[0].hp, 0);
    assert_eq!(combat.state.outcome, Some(Outcome::Victory));
    assert!(combat.state.character.orb_slots.iter().all(Option::is_none),
        "lethal damage interrupts the later Frost channel");
    assert!(combat.state.pending.is_empty());
}
