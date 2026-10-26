use super::super::*;
use super::full_runtime::entered_combat;
use crate::{
    content::{CardColor, CardId, CardRarity, CardType, Catalog, Character, EncounterId, NoteStorage, RelicId},
    combat::model::OwnedCard,
};

fn combat_with_card(id: CardId, upgrades: u32) -> Combat {
    let mut combat = entered_combat(3, 3, |_| {});
    let card = combat.state.piles.hand[0].full_mut().unwrap();
    *card = card_state::permanent(id, card.instance, upgrades);
    combat.state.character = characters::CharacterState::for_character(Character::Silent);
    combat
}

fn combat_with_owned_relic(id: CardId, upgrades: u32, relic: RelicId) -> Combat {
    let mut entry = super::m5_elites::entry(EncounterId::Cultist);
    entry.setup.character = Character::Silent;
    let instance = entry.cards[0].instance;
    entry.cards[0] = card_state::permanent(id, instance, upgrades);
    let mut items = Some(crate::items::CombatItemContext::new(vec![relic], vec![]));
    Combat::enter_with_item_context(entry, &mut items).expect("accepted explicit native relic context")
}

fn full_power(id: &'static str, amount: i32) -> FullPowerEntry {
    FullPowerEntry { id: FullPowerId(id), state: PowerState {
        amount, skip_first: false, monster_source_grace: false,
    } }
}

fn amount(combat: &Combat, slot: usize, id: &'static str) -> Option<i32> {
    combat.state.enemies[slot].full_powers.iter()
        .find(|power| power.id == FullPowerId(id)).map(|power| power.state.amount)
}

fn play(combat: &mut Combat, id: CardId, target: Option<TargetId>) -> Result<(), crate::run::RunError> {
    let instance = combat.state.piles.hand.iter().filter_map(OwnedCard::full)
        .find(|card| card.id == id).expect("requested card is in hand").instance.0;
    combat.step_full(&CombatAction::PlayCard { instance, target })
}

fn apply_enemy_poison(combat: &mut Combat, amount: i32) {
    let target = Actor::Enemy(combat.state.enemies[0].instance_id.clone());
    combat.enqueue_native_effects(vec![Effect {
        source: Source::Actor(Actor::Player), target: Some(target),
        kind: EffectKind::ApplyFullPower { id: FullPowerId("poison"), amount },
    }], None).unwrap();
}

fn commit_prepared_catalyst_with_prefix(combat: &mut Combat, prefix: Vec<Effect>) {
    let instance = combat.state.piles.hand.iter().filter_map(OwnedCard::full)
        .find(|card| card.id == CardId::Catalyst).expect("Catalyst remains in hand after preparation").instance.0;
    let prepared = cards::prepare(combat, &CombatAction::PlayCard {
        instance, target: Some(TargetId(1)),
    }).expect("actual legal Catalyst prepares before its queued action");
    assert_eq!(prepared.paid, 1);
    let handle = card_state::handle(prepared.instance);
    let source = Source::Card(handle.clone());
    let disposition = combat.state.piles.hand.get(prepared.hand_index).and_then(OwnedCard::full)
        .map_or(model::FinishDisposition::Natural, |card|
            cards::ironclad::exhaust::per_use_disposition(&combat.state, card));
    let card = combat.state.piles.hand.remove(prepared.hand_index);
    combat.state.in_play = Some(card);
    combat.state.player.energy = prepared.remaining_energy;
    combat.state.phase = Phase::PlayerAction;
    combat.state.play_context = Some(model::PlayContext {
        card: handle, target: prepared.target.clone(), energy_on_use: prepared.captured_energy,
        paid: prepared.paid, disposition,
    });
    combat.state.pending.push_back(Effect { source: source.clone(), target: prepared.target.clone(),
        kind: EffectKind::Hook(Hook::CardPlayed) });
    combat.state.pending.extend(prefix);
    combat.state.pending.extend(prepared.effects);
    combat.state.pending.push_back(Effect { source: source.clone(), target: None,
        kind: EffectKind::Hook(Hook::CardAfterUse) });
    combat.state.pending.push_back(Effect { source, target: None, kind: EffectKind::FinishCard });
    combat.state.pending.push_back(Effect { source: Source::None, target: None, kind: EffectKind::Decision });
    engine::resolve(&mut combat.state);
}

fn poison_prefix(combat: &Combat, amount: i32) -> Effect {
    Effect { source: Source::Actor(Actor::Player), target: Some(Actor::Enemy(combat.state.enemies[0].instance_id.clone())),
        kind: EffectKind::ApplyFullPower { id: FullPowerId("poison"), amount } }
}

fn remove_poison_prefix(combat: &Combat) -> Effect {
    Effect { source: Source::Actor(Actor::Player), target: Some(Actor::Enemy(combat.state.enemies[0].instance_id.clone())),
        kind: EffectKind::RemoveFullPower(FullPowerId("poison")) }
}

#[test]
fn m11_catalyst_samples_current_poison_when_its_queued_action_resolves() {
    for (upgrades, expected) in [(0, 10), (1, 15)] {
        let mut combat = combat_with_card(CardId::Catalyst, upgrades);
        apply_enemy_poison(&mut combat, 4);
        let prefix = poison_prefix(&combat, 1);
        commit_prepared_catalyst_with_prefix(&mut combat, vec![prefix]);
        assert_eq!(amount(&combat, 0, "poison"), Some(expected),
            "Catalyst reads P=5 when its action runs, after preparation captured the earlier P=4");
        assert!(combat.state.piles.exhaust.iter().any(|card|
            card.full().is_some_and(|live| live.id == CardId::Catalyst)));
        assert_eq!(combat.state.player.energy, 2);
    }
}

#[test]
fn m11_catalyst_can_sample_poison_added_after_its_preparation() {
    for (upgrades, expected) in [(0, 8), (1, 12)] {
        let mut combat = combat_with_card(CardId::Catalyst, upgrades);
        let prefix = poison_prefix(&combat, 4);
        commit_prepared_catalyst_with_prefix(&mut combat, vec![prefix]);
        assert_eq!(amount(&combat, 0, "poison"), Some(expected),
            "the queued Catalyst action sees P=4 applied after its no-Poison preparation");
    }
}

#[test]
fn m11_catalyst_does_not_apply_when_poison_is_removed_before_its_action() {
    let mut combat = combat_with_card(CardId::Catalyst, 1);
    apply_enemy_poison(&mut combat, 4);
    let prefix = remove_poison_prefix(&combat);
    commit_prepared_catalyst_with_prefix(&mut combat, vec![prefix]);
    assert_eq!(amount(&combat, 0, "poison"), None,
        "Catalyst observes no current Poison at its queued action and creates no application");
    assert!(combat.state.piles.exhaust.iter().any(|card|
        card.full().is_some_and(|live| live.id == CardId::Catalyst)));
    assert_eq!(combat.state.player.energy, 2);
}

#[test]
fn m11_silent_poison_application_and_tick_order_are_source_owned() {
    for (upgrades, expected) in [(0, 5), (1, 7)] {
        let mut combat = combat_with_card(CardId::DeadlyPoison, upgrades);
        assert_eq!(play(&mut combat, CardId::DeadlyPoison, Some(TargetId(1))), Ok(()),
            "Deadly Poison is a real one-energy selected-enemy application");
        assert_eq!(amount(&combat, 0, "poison"), Some(expected));
        assert_eq!(combat.state.player.energy, 2);
    }

    for (upgrades, damage, poison) in [(0, 6, 3), (1, 8, 4)] {
        let mut combat = combat_with_card(CardId::PoisonedStab, upgrades);
        combat.state.enemies[0].block = 2;
        let instance = combat.state.piles.hand[0].full().unwrap().instance.0;
        let prepared = cards::prepare(&combat, &CombatAction::PlayCard {
            instance, target: Some(TargetId(1)),
        }).unwrap();
        assert_eq!(prepared.paid, 1);
        assert_eq!(prepared.effects.len(), 2);
        assert!(matches!(prepared.effects[0].kind,
            EffectKind::Damage(DamagePayload::Fixed(value)) if value == damage));
        assert!(matches!(prepared.effects[1].kind,
            EffectKind::ApplyFullPower { id: FullPowerId("poison"), amount } if amount == poison));
        assert_eq!(play(&mut combat, CardId::PoisonedStab, Some(TargetId(1))), Ok(()));
        assert_eq!(combat.state.enemies[0].hp, 48 - (damage - 2));
        assert_eq!(amount(&combat, 0, "poison"), Some(poison));
        assert_eq!(combat.state.player.energy, 2);
    }

    for (upgrades, start, expected) in [
        (0, 4, 8), (1, 4, 12), (0, 6_000, 12_000), (1, 6_000, 18_000),
    ] {
        let mut combat = combat_with_card(CardId::Catalyst, upgrades);
        combat.state.enemies[0].full_powers.push(full_power("poison", start));
        assert!(engine::validate_runtime(&combat.state).is_ok(),
            "the 6000 starting amount is below the source-qualified new-object cap");
        assert_eq!(play(&mut combat, CardId::Catalyst, Some(TargetId(1))), Ok(()));
        assert_eq!(amount(&combat, 0, "poison"), Some(expected));
        assert!(combat.state.piles.exhaust.iter().any(|card|
            card.full().is_some_and(|live| live.id == CardId::Catalyst)));
        assert_eq!(combat.state.player.energy, 2);
    }

    let mut absent = combat_with_card(CardId::Catalyst, 1);
    assert_eq!(play(&mut absent, CardId::Catalyst, Some(TargetId(1))), Ok(()));
    assert_eq!(amount(&absent, 0, "poison"), None,
        "no Poison means Catalyst queues no application");
    assert!(absent.state.piles.exhaust.iter().any(|card|
        card.full().is_some_and(|live| live.id == CardId::Catalyst)));

    for (id, color, rarity, card_type) in [
        (CardId::DeadlyPoison, CardColor::Green, CardRarity::Common, CardType::Skill),
        (CardId::PoisonedStab, CardColor::Green, CardRarity::Common, CardType::Attack),
        (CardId::Catalyst, CardColor::Green, CardRarity::Uncommon, CardType::Skill),
    ] {
        let definition = Catalog::standard().card(id).unwrap();
        assert_eq!((definition.color, definition.rarity, definition.card_type, definition.definition_only),
            (color, rarity, card_type, false));
    }
    assert!(card_state::permanent(CardId::Catalyst, CardInstanceId(99), 0).local.exhaust_on_finish);
    assert_eq!(powers::cleanse_debuff(&full_power("poison", 1)), Some(true));
}

#[test]
fn m11_silent_artifact_spends_one_charge_per_application_without_cleansing_poison() {
    let mut combat = combat_with_card(CardId::DeadlyPoison, 0);
    let target = Actor::Enemy(combat.state.enemies[0].instance_id.clone());
    combat.state.enemies[0].full_powers.extend([
        full_power("poison", 2), full_power("artifact", 2),
    ]);
    combat.enqueue_native_effects(vec![
        Effect { source: Source::Actor(Actor::Player), target: Some(target.clone()),
            kind: EffectKind::ApplyFullPower { id: FullPowerId("poison"), amount: 5 } },
        Effect { source: Source::Actor(Actor::Player), target: Some(target),
            kind: EffectKind::ApplyFullPower { id: FullPowerId("poison"), amount: 3 } },
    ], None).unwrap();
    assert_eq!(amount(&combat, 0, "poison"), Some(2));
    assert_eq!(amount(&combat, 0, "artifact"), None,
        "each separate debuff application consumes one Artifact charge");
}

#[test]
fn m11_silent_poison_tick_is_hp_loss_before_current_stack_decrement() {
    let mut combat = combat_with_card(CardId::DeadlyPoison, 0);
    combat.state.enemies[0].hp = 10;
    combat.state.enemies[0].block = 8;
    apply_enemy_poison(&mut combat, 2);
    let owner = Actor::Enemy(combat.state.enemies[0].instance_id.clone());
    combat.state.pending.push_back(Effect { source: Source::None, target: Some(owner.clone()),
        kind: EffectKind::Hook(Hook::EnemyTurnStarted) });
    combat.state.pending.push_back(Effect { source: Source::None, target: None, kind: EffectKind::Decision });
    engine::resolve(&mut combat.state);
    assert!(combat.state.fault.is_none(), "{:?}", combat.fault_record());
    assert_eq!((combat.state.enemies[0].hp, combat.state.enemies[0].block), (8, 8),
        "enemy Poison uses HP_LOSS and does not consume Block");
    assert_eq!(amount(&combat, 0, "poison"), Some(1));

    combat.state.pending.push_back(Effect { source: Source::None, target: Some(owner.clone()),
        kind: EffectKind::Hook(Hook::EnemyTurnStarted) });
    combat.state.pending.push_back(Effect { source: Source::None, target: None, kind: EffectKind::Decision });
    engine::resolve(&mut combat.state);
    assert!(combat.state.fault.is_none(), "{:?}", combat.fault_record());
    assert_eq!(combat.state.enemies[0].hp, 7);
    assert_eq!(combat.state.enemies[0].block, 8);
    assert_eq!(amount(&combat, 0, "poison"), None,
        "the one remaining stack ticks, then the zero power is removed");
}

#[test]
fn m11_silent_lethal_poison_tick_decrements_before_death_settlement() {
    let mut combat = combat_with_card(CardId::DeadlyPoison, 0);
    combat.state.enemies[0].hp = 5;
    apply_enemy_poison(&mut combat, 5);
    let move_count = combat.state.enemies[0].move_history.len();

    assert_eq!(combat.step_full(&CombatAction::EndTurn), Ok(()));
    assert_eq!(combat.state.enemies[0].hp, 0);
    assert_eq!(amount(&combat, 0, "poison"), Some(4),
        "HP_LOSS completion decrements the current Poison before basically-dead settlement");
    assert_eq!(combat.state.enemies[0].move_history.len(), move_count,
        "a lethal owner-start tick prevents that actor's queued move");
    assert_eq!(combat.state.outcome, Some(Outcome::Victory));
    assert_eq!(combat.state.hook_events.iter().filter(|event| event.hook == Hook::Death).count(), 1);
    assert_eq!(combat.state.hook_events.iter().filter(|event| event.hook == Hook::CombatSettled).count(), 1);
}

#[test]
fn m11_silent_poison_tick_uses_captured_damage_and_reduces_the_current_stack() {
    let mut combat = combat_with_card(CardId::DeadlyPoison, 0);
    combat.state.enemies[0].hp = 20;
    apply_enemy_poison(&mut combat, 5);
    let owner = Actor::Enemy(combat.state.enemies[0].instance_id.clone());
    hooks::dispatch(&mut combat.state, Hook::EnemyTurnStarted, &Source::None, Some(&owner)).unwrap();
    assert!(matches!(combat.state.pending.front().map(|effect| &effect.kind),
        Some(EffectKind::Damage(DamagePayload::PoisonTickHpLoss { owner: captured, amount: 5 }))
            if captured == &combat.state.enemies[0].instance_id));

    combat.state.enemies[0].full_powers.iter_mut()
        .find(|power| power.id == FullPowerId("poison")).unwrap().state.amount = 9;
    engine::resolve(&mut combat.state);

    assert!(combat.state.fault.is_none(), "{:?}", combat.fault_record());
    assert_eq!(combat.state.enemies[0].hp, 15,
        "the tick keeps its amount captured when the owner-start hook ran");
    assert_eq!(amount(&combat, 0, "poison"), Some(8),
        "the queued reduction applies to the current stack, not the captured amount");
}

#[test]
fn m11_silent_poison_tick_still_damages_after_poison_is_removed_before_delivery() {
    let mut combat = combat_with_card(CardId::DeadlyPoison, 0);
    combat.state.enemies[0].hp = 20;
    apply_enemy_poison(&mut combat, 5);
    let owner = Actor::Enemy(combat.state.enemies[0].instance_id.clone());
    hooks::dispatch(&mut combat.state, Hook::EnemyTurnStarted, &Source::None, Some(&owner)).unwrap();
    combat.state.enemies[0].full_powers.retain(|power| power.id != FullPowerId("poison"));
    engine::resolve(&mut combat.state);

    assert!(combat.state.fault.is_none(), "{:?}", combat.fault_record());
    assert_eq!(combat.state.enemies[0].hp, 15,
        "the already captured HP_LOSS is not cancelled by later Poison removal");
    assert_eq!(amount(&combat, 0, "poison"), None,
        "the queued one-stack reduction is harmless when no current stack remains");
}

#[test]
fn m11_silent_poison_lethal_darkling_keeps_its_half_death_callback() {
    let entry = super::m8_shapes::entry(EncounterId::ThreeDarklings);
    let mut combat = Combat::enter(entry).unwrap();
    combat.state.enemies[0].hp = 1;
    combat.state.enemies[0].full_powers.push(full_power("poison", 1));
    let owner = Actor::Enemy(combat.state.enemies[0].instance_id.clone());
    hooks::dispatch(&mut combat.state, Hook::EnemyTurnStarted, &Source::None, Some(&owner)).unwrap();
    engine::resolve(&mut combat.state);

    assert!(combat.state.fault.is_none(), "{:?}", combat.fault_record());
    assert_eq!((combat.state.enemies[0].hp, combat.state.enemies[0].life), (0, ActorLife::HalfDead));
    assert_eq!(combat.state.outcome, None, "downing one Darkling is not final combat death");
    assert_eq!(combat.state.hook_events.iter().filter(|event| event.hook == Hook::HalfDeath).count(), 1,
        "the source-owned HalfDeath lifecycle hook is preserved");
    assert_eq!(combat.state.hook_events.iter().filter(|event| event.hook == Hook::Death).count(), 0,
        "the Poison completion marker does not fabricate final actor death during HalfDead revival");
    assert_eq!(combat.state.hook_events.iter().filter(|event| event.hook == Hook::CombatSettled).count(), 0);
}

#[test]
fn m11_silent_poison_lethal_reptomancer_keeps_minion_cleanup_and_death_callbacks() {
    let entry = super::m8_shapes::entry(EncounterId::Reptomancer);
    let mut combat = Combat::enter(entry).unwrap();
    combat.state.enemies[1].hp = 1;
    combat.state.enemies[1].full_powers.push(full_power("poison", 1));
    let owner = Actor::Enemy(combat.state.enemies[1].instance_id.clone());
    hooks::dispatch(&mut combat.state, Hook::EnemyTurnStarted, &Source::None, Some(&owner)).unwrap();
    engine::resolve(&mut combat.state);

    assert!(combat.state.fault.is_none(), "{:?}", combat.fault_record());
    assert!(combat.state.enemies.iter().all(|enemy| enemy.hp == 0),
        "the parent's existing Reptomancer cleanup still resolves its live Daggers");
    assert_eq!(combat.state.outcome, Some(Outcome::Victory));
    assert_eq!(combat.state.hook_events.iter().filter(|event| event.hook == Hook::Death).count(), 3,
        "two Dagger cleanup callbacks and the parent's deferred Poison death callback remain");
    assert_eq!(combat.state.hook_events.iter().filter(|event| event.hook == Hook::CombatSettled).count(), 1);
}

#[test]
fn m11_silent_ordinary_self_hp_loss_cannot_impersonate_a_queued_poison_tick() {
    let mut combat = combat_with_card(CardId::DeadlyPoison, 0);
    combat.state.enemies[0].hp = 5;
    apply_enemy_poison(&mut combat, 5);
    let owner = Actor::Enemy(combat.state.enemies[0].instance_id.clone());
    combat.enqueue_native_effects(vec![Effect { source: Source::Actor(owner.clone()),
        target: Some(owner), kind: EffectKind::Damage(DamagePayload::HpLoss(5)) }], None).unwrap();

    assert_eq!(combat.state.enemies[0].hp, 0);
    assert_eq!(combat.state.outcome, Some(Outcome::Victory),
        "ordinary self HP_LOSS uses the ordinary settlement path even when its amount matches Poison");
    assert_eq!(amount(&combat, 0, "poison"), Some(5), "ordinary HP_LOSS does not decay Poison");
    assert_eq!(combat.state.hook_events.iter().filter(|event| event.hook == Hook::Death).count(), 1);
    assert_eq!(combat.state.hook_events.iter().filter(|event| event.hook == Hook::CombatSettled).count(), 1);
}

#[test]
fn m11_silent_poison_tick_fault_retains_committed_damage_and_pending_decrement() {
    let mut combat = combat_with_card(CardId::DeadlyPoison, 0);
    combat.state.enemies[0].hp = 20;
    apply_enemy_poison(&mut combat, 3);
    let owner = Actor::Enemy(combat.state.enemies[0].instance_id.clone());
    combat.state.resolved_effects = 99_997;
    combat.state.pending.extend([
        Effect { source: Source::Actor(owner.clone()), target: Some(owner.clone()), kind: EffectKind::Hook(Hook::EnemyTurnStarted) },
        Effect { source: Source::Actor(owner.clone()), target: Some(owner.clone()), kind: EffectKind::EnemyMove },
    ]);
    engine::dispatch_one(&mut combat.state);
    engine::dispatch_one(&mut combat.state);
    assert_eq!(combat.state.enemies[0].hp, 17,
        "the HP_LOSS is committed before the callback dispatch guard trips");
    engine::dispatch_one(&mut combat.state);
    assert_eq!(combat.state.fault.as_ref().map(|fault| fault.kind), Some(FaultKind::EffectGuard));
    assert_eq!(combat.state.current.as_ref().map(|effect| &effect.kind),
        Some(&EffectKind::Hook(Hook::Damage)));
    assert_eq!(amount(&combat, 0, "poison"), Some(3),
        "the accepted post-damage fault occurs before the current stack is decremented");
    assert_eq!(combat.state.pending.front().map(|effect| &effect.kind),
        Some(&EffectKind::ReduceFullPower { id: FullPowerId("poison"), amount: 1 }));
    assert_eq!(combat.state.pending.get(1).map(|effect| &effect.kind),
        Some(&EffectKind::PoisonTickComplete));
    assert_eq!(combat.state.pending.get(2).map(|effect| &effect.kind), Some(&EffectKind::EnemyMove));
    let retained = combat.state.clone();
    engine::dispatch_one(&mut combat.state);
    assert_eq!(combat.state, retained, "a retained fault cannot retry damage or poison decay");
}

#[test]
fn m11_silent_snecko_skull_and_noxious_fumes_have_native_content_rows() {
    let relic = Catalog::standard().relic(RelicId::SneckoSkull).expect("owned relic row");
    assert!(!relic.definition_only);
    let fumes = Catalog::standard().card(CardId::NoxiousFumes).expect("card row");
    assert_eq!((fumes.color, fumes.rarity, fumes.card_type, fumes.note_storage, fumes.definition_only),
        (CardColor::Green, CardRarity::Uncommon, CardType::Power, NoteStorage::RemovablePermanent, false));
}

#[test]
fn m11_silent_snecko_skull_adds_once_to_each_eligible_application_not_tick_or_decay() {
    let mut combat = combat_with_owned_relic(CardId::Catalyst, 0, RelicId::SneckoSkull);
    assert!(combat.item_context().unwrap().owns_relic(RelicId::SneckoSkull));
    apply_enemy_poison(&mut combat, 4);
    assert_eq!(amount(&combat, 0, "poison"), Some(5), "one eligible application gets one Skull point");

    assert_eq!(play(&mut combat, CardId::Catalyst, Some(TargetId(1))), Ok(()));
    assert_eq!(amount(&combat, 0, "poison"), Some(11),
        "Catalyst samples P=5, then its separate P application receives exactly one Skull point");

    let owner = Actor::Enemy(combat.state.enemies[0].instance_id.clone());
    combat.state.pending.extend([
        Effect { source: Source::Actor(owner.clone()), target: Some(owner.clone()),
            kind: EffectKind::Hook(Hook::EnemyTurnStarted) },
        Effect { source: Source::None, target: None, kind: EffectKind::Decision },
    ]);
    engine::resolve(&mut combat.state);
    assert!(combat.state.fault.is_none(), "{:?}", combat.fault_record());
    assert_eq!(amount(&combat, 0, "poison"), Some(10), "tick decrement receives no Skull bonus");

    let mut blocked = combat_with_owned_relic(CardId::DeadlyPoison, 0, RelicId::SneckoSkull);
    blocked.state.enemies[0].full_powers.extend([
        full_power("poison", 2), full_power("artifact", 1),
    ]);
    assert_eq!(play(&mut blocked, CardId::DeadlyPoison, Some(TargetId(1))), Ok(()));
    assert_eq!(amount(&blocked, 0, "poison"), Some(2), "Artifact-blocked Poison has no Skull increment");
    assert_eq!(amount(&blocked, 0, "artifact"), None);

    let mut absent = combat_with_owned_relic(CardId::Catalyst, 0, RelicId::SneckoSkull);
    assert_eq!(play(&mut absent, CardId::Catalyst, Some(TargetId(1))), Ok(()));
    assert_eq!(amount(&absent, 0, "poison"), None, "absent Catalyst emits no eligible application");
}

#[test]
fn m11_silent_noxious_fumes_uses_post_draw_power_disposition_and_live_targets() {
    for (upgrades, expected) in [(0, 2), (1, 3)] {
        let mut combat = combat_with_card(CardId::NoxiousFumes, upgrades);
        let instance = combat.state.piles.hand[0].full().unwrap().instance.0;
        let prepared = cards::prepare(&combat, &CombatAction::PlayCard { instance, target: None }).unwrap();
        assert_eq!(prepared.paid, 1);
        assert_eq!(prepared.target, None, "Noxious Fumes is targetless");
        assert_eq!(play(&mut combat, CardId::NoxiousFumes, None), Ok(()));
        assert_eq!(combat.state.player.energy, 2);
        assert_eq!(combat.state.player_full_powers.iter().find(|power|
            power.id == FullPowerId("noxious_fumes")).map(|power| power.state.amount), Some(expected));
        assert!(combat.state.played_powers.iter().any(|card|
            card.full().is_some_and(|live| live.instance.0 == instance && live.id == CardId::NoxiousFumes)));
        assert_eq!(amount(&combat, 0, "poison"), None,
            "playing the Power installs its callback but does not immediately poison enemies");

        let hp = combat.state.enemies[0].hp;
        combat.state.enemies[0].full_powers.push(full_power("poison", 2));
        assert_eq!(combat.step_full(&CombatAction::EndTurn), Ok(()));
        assert_eq!(combat.state.enemies[0].hp, hp - 2, "the old stack ticks during enemy start first");
        assert_eq!(amount(&combat, 0, "poison"), Some(expected + 1),
            "Noxious Fumes applies its full amount separately after the next player draw");
        let drawn = combat.state.hook_events.iter().rposition(|event| event.hook == Hook::CardDrawn)
            .expect("player draw emits a CardDrawn checkpoint");
        let started = combat.state.hook_events.iter().rposition(|event|
            event.hook == Hook::TurnStarted && event.target == Some(Actor::Player))
            .expect("player start callback is recorded");
        assert!(drawn < started, "player's next Draw is before Noxious Fumes at TurnStarted");

        let owner = Actor::Enemy(combat.state.enemies[0].instance_id.clone());
        combat.enqueue_native_effects(vec![Effect { source: Source::Actor(Actor::Player),
            target: Some(owner), kind: EffectKind::Damage(DamagePayload::Fixed(999)) }], None).unwrap();
        let exit = combat.exit().unwrap();
        assert_eq!(exit.played_powers.iter().map(|card| (card.instance.0, card.id)).collect::<Vec<_>>(),
            vec![(instance, CardId::NoxiousFumes)], "the Power card has one owned exit record");
    }
}

#[test]
fn m11_silent_noxious_fumes_turn_hook_only_enqueues_for_live_enemies() {
    let mut entry = super::m5_elites::entry(EncounterId::ThreeLouse);
    entry.setup.character = Character::Silent;
    let mut combat = Combat::enter(entry).unwrap();
    combat.state.player_full_powers.push(full_power("noxious_fumes", 2));
    combat.state.enemies[0].hp = 0;
    combat.state.enemies[0].full_powers.push(full_power("poison", 5));
    hooks::dispatch(&mut combat.state, Hook::TurnStarted, &Source::Actor(Actor::Player),
        Some(&Actor::Player)).unwrap();
    let targets = combat.state.pending.iter().filter_map(|effect| {
        matches!(effect.kind, EffectKind::ApplyFullPower { id: FullPowerId("poison"), amount: 2 })
            .then(|| effect.target.clone())
    }).collect::<Vec<_>>();
    assert_eq!(targets, vec![Some(Actor::Enemy(combat.state.enemies[1].instance_id.clone())),
        Some(Actor::Enemy(combat.state.enemies[2].instance_id.clone()))]);
}

#[test]
fn m11_silent_poisoned_stab_kill_keeps_queued_poison_on_the_dead_target_a_noop() {
    let mut entry = super::m5_elites::entry(EncounterId::ThreeLouse);
    entry.setup.character = Character::Silent;
    entry.cards[0] = card_state::permanent(CardId::PoisonedStab, entry.cards[0].instance, 0);
    entry.setup.enemies[0].hp = 6;
    let survivor_hp = entry.setup.enemies[1..].iter().map(|enemy| enemy.hp).collect::<Vec<_>>();
    let mut combat = Combat::enter(entry).unwrap();
    let instance = combat.state.piles.hand[0].full().unwrap().instance.0;

    assert_eq!(play(&mut combat, CardId::PoisonedStab, Some(TargetId(1))), Ok(()));
    assert_eq!(combat.state.fault, None, "the source ApplyPowerAction dead-target guard is a no-op");
    assert_eq!(combat.state.enemies[0].hp, 0);
    assert_eq!(amount(&combat, 0, "poison"), None);
    assert_eq!(combat.state.enemies[1..].iter().map(|enemy| enemy.hp).collect::<Vec<_>>(), survivor_hp);
    assert_eq!(combat.state.player.energy, 2, "the accepted card payment remains real");
    assert!(combat.state.piles.discard.iter().any(|card|
        card.full().is_some_and(|live| live.instance.0 == instance && live.id == CardId::PoisonedStab)),
        "the Attack completes its ordinary discard disposition");
    assert_eq!(combat.state.outcome, None, "surviving enemies keep combat live");
    assert_eq!(combat.state.hook_events.iter().filter(|event|
        event.hook == Hook::Death && event.target == Some(Actor::Enemy(combat.state.enemies[0].instance_id.clone()))).count(), 1);
    assert_eq!(combat.state.hook_events.iter().filter(|event| event.hook == Hook::CombatSettled).count(), 0);
    assert!(engine::validate_runtime(&combat.state).is_ok());
}
