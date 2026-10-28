use crate::combat::{Actor, Combat, CombatAction, CombatCardData, CombatCardRef, CombatEntry,
    DamagePayload, Effect, EffectKind, EnemyHandle, FullCombatSetup, FullEnemySetup,
    FullIntentId, FullPowerEntry, Hook, LiveCard, PlayerState, PowerState, ResolvedEffectKind,
    Source, TargetId};
use crate::combat::rng::{Domain, StreamId, StreamState};
use crate::content::{CardId, Character, EnemyId, PotionId, RelicId};
use crate::items::{relic_hook_effects, silent_death_work, CombatItemContext};

fn full_power(id: &'static str, amount: i32) -> FullPowerEntry {
    FullPowerEntry { id: crate::combat::FullPowerId(id), state: PowerState {
        amount, skip_first: false, monster_source_grace: false,
    } }
}

fn entry(cards: &[CardId], opening_draw: usize, enemy_count: usize,
    enemy_id: EnemyId, intent: FullIntentId, enemy_powers: &[FullPowerEntry],
    player_powers: &[FullPowerEntry]) -> CombatEntry {
    let scope = format!("m15-silent-items-{}-{opening_draw}-{enemy_count}-{enemy_id:?}-{intent:?}", cards.len());
    let shuffle_id = StreamId::new(Domain::RunCombatShuffle, vec![scope.clone()]).unwrap();
    let card_id = StreamId::new(Domain::RunCombatCardRandom, vec![scope.clone()]).unwrap();
    let enemies = (0..enemy_count).map(|index| {
        let owner = EnemyHandle::parse(&format!("enemy-{}", index + 1)).unwrap();
        let ai_id = StreamId::new(Domain::RunCombatEnemyAi,
            vec![scope.clone(), owner.as_str().to_owned()]).unwrap();
        FullEnemySetup { initial_memory: None, instance_id: owner, ascension: 0,
            profile: crate::combat::enemies::EnemyProfile::Standard, enemy_id,
            hp: 48, max_hp: 48, block: 0, intent: intent.clone(),
            powers: enemy_powers.to_vec(),
            ai_rng: StreamState::new(ai_id, [0x53; 32], 0).unwrap() }
    }).collect();
    let live_cards = cards.iter().enumerate().map(|(index, id)| {
        let instance = crate::combat::CardInstanceId(index as u64 + 1);
        LiveCard { instance, id: *id, origin: CombatCardRef::Permanent(instance), upgrades: 0,
            local: CombatCardData::default(), copy_family: instance.0,
            copy_provenance: crate::combat::card_state::CopyProvenance::Original }
    }).collect();
    CombatEntry { setup: FullCombatSetup { enemy_damage_bases: Default::default(),
        player: PlayerState { hp: 35, max_hp: 80, block: 0, energy: 3, max_energy: 3,
            draw_per_turn: 5, powers: vec![] }, player_powers: player_powers.to_vec(),
        character: Character::Silent,
        shuffle_rng: StreamState::new(shuffle_id, [0x51; 32], 0).unwrap(),
        card_rng: StreamState::new(card_id, [0x52; 32], 0).unwrap(), enemies },
        opening_draw, cards: live_cards }
}

fn enter(entry: CombatEntry, relics: Vec<RelicId>, potions: Vec<Option<PotionId>>) -> Combat {
    let mut owner = Some(CombatItemContext::new(relics, potions).with_m15_profile());
    let combat = Combat::enter_with_item_context(entry, &mut owner).unwrap();
    assert!(owner.is_none(), "the entered combat takes the one native inventory owner");
    combat
}

fn power_amount(combat: &Combat, actor: &Actor, id: &str) -> Option<i32> {
    let view = combat.public_full().unwrap();
    let powers = match actor {
        Actor::Player => view.player.powers,
        Actor::Enemy(owner) => view.enemies.into_iter().find(|enemy| enemy.instance_id == *owner)
            .expect("owner is present").powers,
    };
    powers.into_iter().find(|power| power.id.0 == id).map(|power| power.amount)
}

fn play_and_discard(combat: &mut Combat, instance: u64) -> u64 {
    combat.step_full(&CombatAction::PlayCard { instance, target: Some(TargetId(1)) }).unwrap();
    let selection = combat.public_full().unwrap().selection.expect("Dagger Throw opens a real discard choice");
    let selected = selection.candidates.iter().find(|card| card.instance.0 != instance)
        .expect("another hand card can be manually discarded").instance.0;
    combat.step_full(&CombatAction::SelectCard { instance: selected }).unwrap();
    selected
}

#[test]
fn m15_silent_startup_replacement_and_legacy_opening_draw_are_separate() {
    use crate::run::{CreationProfile, Run, RunConfig, RunRng};

    let mut run = Run::new(RunConfig { character: Character::Silent, ascension: 0 },
        CreationProfile::experienced(), RunRng::new([0x61; 32], crate::combat::rng::Partition::Test, 0)).unwrap();
    assert_eq!(run.state().relics.first(), Some(&RelicId::RingOfTheSnake));
    crate::items::acquire_relic(run.state_mut_for_test(), RelicId::RingOfTheSerpent).unwrap();
    assert_eq!(run.state().relics.first(), Some(&RelicId::RingOfTheSerpent));
    assert!(!run.state().relics.contains(&RelicId::RingOfTheSnake));
    assert!(!run.state().relic_counters.contains_key(&RelicId::RingOfTheSnake));

    let deck = [CardId::StrikeGreen; 10];
    let mut serpent = enter(entry(&deck, 5, 1, EnemyId::Cultist, FullIntentId::Incantation, &[], &[]),
        vec![RelicId::RingOfTheSerpent], vec![]);
    let view = serpent.public_full().unwrap();
    assert_eq!(view.player.powers.iter().find(|power| power.id.0 == "draw_per_turn").map(|power| power.amount), None);
    assert_eq!(serpent.state.player.draw_per_turn, 6, "Serpent increases master draw per turn by one");
    assert_eq!(view.hand.len(), 6, "Serpent draws one extra card in the real opening hand");
    serpent.step_full(&CombatAction::EndTurn).unwrap();
    assert_eq!(serpent.public_full().unwrap().hand.len(), 6,
        "Serpent's master draw bonus persists on the following player turn");
    serpent.enqueue_native_effects(vec![Effect { source: Source::Actor(Actor::Player),
        target: Some(Actor::Enemy(EnemyHandle::parse("enemy-1").unwrap())),
        kind: EffectKind::Damage(DamagePayload::Fixed(999)) }], None).unwrap();
    let exit = serpent.exit().unwrap();
    assert_eq!(exit.item_context.as_ref().unwrap().relics(), &[RelicId::RingOfTheSerpent],
        "the active context returns exact M15 relic custody through combat Exit");

    let ninja = enter(entry(&deck, 5, 1, EnemyId::Cultist, FullIntentId::Incantation, &[], &[]),
        vec![RelicId::NinjaScroll], vec![]);
    let view = ninja.public_full().unwrap();
    assert_eq!(view.hand.iter().filter(|card| card.id == CardId::Shiv).count(), 3,
        "Ninja Scroll creates three ordinary Shivs before the opening draw");
    assert_eq!(view.hand.iter().filter(|card| card.origin == CombatCardRef::Permanent(card.instance)).count(), 5);

    let combined = enter(entry(&deck, 5, 1, EnemyId::Cultist, FullIntentId::Incantation, &[], &[]),
        vec![RelicId::RingOfTheSerpent, RelicId::NinjaScroll], vec![]);
    assert_eq!(combined.state.player.draw_per_turn, 6);
    assert_eq!(combined.public_full().unwrap().hand.len(), 9,
        "the three pre-draw Shivs coexist with six actual opening cards");

    let old_profile = Combat::enter(entry(&deck, 5, 1, EnemyId::Cultist, FullIntentId::Incantation, &[], &[])).unwrap();
    assert_eq!(old_profile.public_full().unwrap().hand.len(), 5);
    assert!(!old_profile.public_full().unwrap().hand.iter().any(|card| card.id == CardId::Shiv),
        "old entries retain their original opening draw and receive no M15 startup work");

    let mut m15_item_without_profile = Some(CombatItemContext::new(vec![RelicId::NinjaScroll], vec![]));
    assert!(matches!(Combat::enter_with_item_context(
        entry(&deck, 5, 1, EnemyId::Cultist, FullIntentId::Incantation, &[], &[]),
        &mut m15_item_without_profile),
        Err(crate::combat::CombatEntryFailure::Rejected(crate::run::RunError::Unsupported(
            "M15 exclusive item requires M15 profile")))));
    assert!(m15_item_without_profile.is_some(), "profile gating rejects before inventory transfer");

    let mut wrong_character = entry(&deck, 5, 1, EnemyId::Cultist, FullIntentId::Incantation, &[], &[]);
    wrong_character.setup.character = Character::Ironclad;
    let mut inventory = Some(CombatItemContext::new(vec![RelicId::NinjaScroll], vec![]).with_m15_profile());
    assert!(matches!(Combat::enter_with_item_context(wrong_character, &mut inventory),
        Err(crate::combat::CombatEntryFailure::Rejected(crate::run::RunError::InvalidAction))));
    assert!(inventory.is_some(), "wrong-character validation rejects before ownership transfer");
}

#[test]
fn m15_silent_funnel_uses_shared_artifact_then_skull_power_dispatch() {
    let blocked = enter(entry(&[CardId::StrikeGreen; 5], 1, 2, EnemyId::Cultist,
        FullIntentId::Incantation, &[full_power("artifact", 1)], &[]),
        vec![RelicId::TwistedFunnel, RelicId::SneckoSkull], vec![]);
    for enemy in &blocked.public_full().unwrap().enemies {
        assert_eq!(power_amount(&blocked, &Actor::Enemy(enemy.instance_id.clone()), "artifact"), None,
            "Artifact consumes one startup Poison application");
        assert_eq!(power_amount(&blocked, &Actor::Enemy(enemy.instance_id.clone()), "poison"), None,
            "Artifact prevention happens before the positive-Poison Snecko Skull modifier");
    }

    let applied = enter(entry(&[CardId::StrikeGreen; 5], 1, 2, EnemyId::Cultist,
        FullIntentId::Incantation, &[], &[]),
        vec![RelicId::TwistedFunnel, RelicId::SneckoSkull], vec![]);
    for enemy in &applied.public_full().unwrap().enemies {
        assert_eq!(power_amount(&applied, &Actor::Enemy(enemy.instance_id.clone()), "poison"), Some(5),
            "each live enemy receives four Funnel Poison plus one Skull point");
    }
}

#[test]
fn m15_silent_manual_discard_queues_bandages_tingsha_and_once_per_turn_kite() {
    let cards = [CardId::DaggerThrow, CardId::StrikeGreen, CardId::DefendGreen,
        CardId::StrikeGreen, CardId::StrikeGreen, CardId::StrikeGreen, CardId::StrikeGreen];
    let mut combat = enter(entry(&cards, cards.len(), 2, EnemyId::Cultist,
        FullIntentId::Incantation, &[], &[]),
        vec![RelicId::ToughBandages, RelicId::HoveringKite, RelicId::Tingsha], vec![]);
    let dagger = combat.public_full().unwrap().hand.iter().find(|card| card.id == CardId::DaggerThrow).unwrap().instance.0;
    let discarded = play_and_discard(&mut combat, dagger);
    let view = combat.public_full().unwrap();
    assert_eq!(view.player.block, 3, "Tough Bandages responds to the actual hand-selection discard");
    assert_eq!(view.player.energy, 3, "Kite's once-per-turn energy resolves after Dagger Throw's payment");
    let losses = view.enemies.iter().map(|enemy| 48 - enemy.hp).collect::<Vec<_>>();
    assert_eq!(losses.iter().sum::<i32>(), 12, "Dagger Throw deals nine and Tingsha adds one random three-damage hit");
    assert!(losses.iter().any(|loss| *loss == 3) || losses.contains(&12),
        "Tingsha selects a current random enemy rather than every enemy: {losses:?}");
    assert!(view.discard.iter().any(|card| card.instance.0 == discarded));
    assert_eq!(combat.state.hook_events.iter().filter(|event| event.hook == Hook::CardManuallyDiscarded).count(), 1);

    let mut context = CombatItemContext::new(vec![RelicId::HoveringKite], vec![]).with_m15_profile();
    let first = relic_hook_effects(&mut context, Hook::CardManuallyDiscarded, 35, 80, 0);
    assert_eq!(first.top.iter().filter(|effect| effect.kind == EffectKind::GainEnergy(1)).count(), 1);
    let second = relic_hook_effects(&mut context, Hook::CardManuallyDiscarded, 35, 80, 0);
    assert!(!second.top.iter().any(|effect| effect.kind == EffectKind::GainEnergy(1)),
        "a second real callback in the same turn cannot pay Kite twice");
    assert!(context.kite_triggered());
    relic_hook_effects(&mut context, Hook::TurnStarted, 35, 80, 0);
    assert!(!context.kite_triggered(), "Kite's once-per-turn custody resets at player turn start");
    let next_turn = relic_hook_effects(&mut context, Hook::CardManuallyDiscarded, 35, 80, 0);
    assert!(next_turn.top.iter().any(|effect| effect.kind == EffectKind::GainEnergy(1)));

    let mut cleanup = enter(entry(&[CardId::StrikeGreen; 5], 5, 1, EnemyId::Cultist,
        FullIntentId::Incantation, &[], &[]),
        vec![RelicId::ToughBandages, RelicId::HoveringKite, RelicId::Tingsha], vec![]);
    cleanup.step_full(&CombatAction::EndTurn).unwrap();
    assert_eq!(cleanup.state.hook_events.iter().filter(|event| event.hook == Hook::CardManuallyDiscarded).count(), 0,
        "end-turn hand cleanup is not a manual discard callback");
    assert_eq!(cleanup.public_full().unwrap().player.block, 0);
    assert!(cleanup.public_full().unwrap().enemies.iter().all(|enemy| enemy.hp == 48),
        "cleanup triggers no Tingsha damage");
}

#[test]
fn m15_tingsha_rng_fault_retains_prior_bandages_and_kite_work_without_retry() {
    let cards = [CardId::DaggerThrow, CardId::StrikeGreen, CardId::DefendGreen,
        CardId::StrikeGreen, CardId::StrikeGreen, CardId::StrikeGreen, CardId::StrikeGreen];
    let mut combat = enter(entry(&cards, cards.len(), 2, EnemyId::Cultist,
        FullIntentId::Incantation, &[], &[]),
        vec![RelicId::ToughBandages, RelicId::HoveringKite, RelicId::Tingsha], vec![]);
    let stream = combat.state.card_rng.as_ref().unwrap();
    combat.state.card_rng = Some(StreamState::new(stream.id().clone(), *stream.key(), 1_u128 << 64).unwrap());
    let dagger = combat.public_full().unwrap().hand.iter().find(|card| card.id == CardId::DaggerThrow).unwrap().instance.0;
    combat.step_full(&CombatAction::PlayCard { instance: dagger, target: Some(TargetId(1)) }).unwrap();
    let selected = combat.public_full().unwrap().selection.unwrap().candidates.iter()
        .find(|card| card.instance.0 != dagger).unwrap().instance.0;
    let error = combat.step_full(&CombatAction::SelectCard { instance: selected }).unwrap_err();
    assert!(matches!(error, crate::run::RunError::Faulted(_)));
    assert_eq!(combat.state.player.block, 3, "the earlier Bandages child remains committed");
    assert_eq!(combat.state.player.energy, 3, "Kite's earlier queued gain remains committed");
    assert!(combat.item_context().unwrap().kite_triggered(), "the callback's once-per-turn flag is retained");
    assert_eq!(combat.state.current.as_ref().unwrap().kind, EffectKind::JuggernautRandomDamage(3));
    assert!(combat.state.pending.iter().any(|effect| effect.kind == EffectKind::FinishCard),
        "the suspended Dagger Throw still owns its queued finish after the callback fault");
    assert_eq!(combat.state.fault_selection.as_ref().unwrap().selected,
        vec![crate::combat::card_state::handle(crate::combat::CardInstanceId(selected))]);
    let faulted = combat.snapshot();
    assert_eq!(combat.step_full(&CombatAction::SelectCard { instance: selected }), Err(error));
    assert_eq!(combat.snapshot(), faulted, "the Tingsha RNG failure cannot retry prior children");
}

#[test]
fn m15_paper_krane_weak_output_and_wrist_blade_source_cost_predicate_share_damage() {
    let jawworm = entry(&[CardId::Shiv; 4], 1, 1, EnemyId::JawWorm,
        FullIntentId::Chomp, &[full_power("weak", 1)], &[]);
    let ordinary = Combat::enter(jawworm.clone()).unwrap();
    let krane = enter(jawworm, vec![RelicId::PaperKrane], vec![]);
    assert_eq!(ordinary.public_full().unwrap().enemies[0].intent_damage, Some(8));
    assert_eq!(krane.public_full().unwrap().enemies[0].intent_damage, Some(6),
        "PaperKrane applies the source Weak output multiplier of 0.6");

    let mut blade = enter(entry(&[CardId::Shiv, CardId::StrikeGreen, CardId::Whirlwind], 3, 1,
        EnemyId::Cultist, FullIntentId::Incantation, &[], &[]), vec![RelicId::WristBlade], vec![]);
    let strike = blade.state.piles.hand.iter_mut().filter_map(crate::combat::model::OwnedCard::full_mut)
        .find(|card| card.id == CardId::StrikeGreen).unwrap();
    strike.local.free_play = true;
    let whirlwind = blade.state.piles.hand.iter_mut().filter_map(crate::combat::model::OwnedCard::full_mut)
        .find(|card| card.id == CardId::Whirlwind).unwrap();
    whirlwind.local.x_cost = true;
    for id in [CardId::Shiv, CardId::StrikeGreen, CardId::Whirlwind] {
        let instance = blade.public_full().unwrap().hand.iter().find(|card| card.id == id).unwrap().instance.0;
        blade.step_full(&CombatAction::PlayCard { instance,
            target: if id == CardId::Whirlwind { None } else { Some(TargetId(1)) } }).unwrap();
    }
    assert_eq!(blade.public_full().unwrap().enemies[0].hp, 15,
        "WristBlade adds four to the zero-cost Shiv and free non-X Strike, then excludes X-cost Whirlwind");
}

#[test]
fn m15_specimen_dispatches_captured_poison_to_a_later_live_target() {
    let mut combat = enter(entry(&[CardId::StrikeGreen; 5], 1, 3, EnemyId::Cultist,
        FullIntentId::Incantation, &[], &[]),
        vec![RelicId::TheSpecimen, RelicId::SneckoSkull], vec![]);
    let dead_owner = EnemyHandle::parse("enemy-1").unwrap();
    let later_dead = EnemyHandle::parse("enemy-2").unwrap();
    let final_live = EnemyHandle::parse("enemy-3").unwrap();
    let work = silent_death_work(combat.item_context().unwrap(), dead_owner.clone(), Some(5), 2);
    assert_eq!(work.len(), 1);
    assert_eq!(silent_death_work(combat.item_context().unwrap(), dead_owner.clone(), Some(0), 2).len(), 0);
    assert_eq!(silent_death_work(combat.item_context().unwrap(), dead_owner.clone(), Some(5), 0).len(), 0,
        "a callback with no eligible living target emits no typed transfer");
    let kill = |owner: EnemyHandle| Effect { source: Source::Actor(Actor::Player),
        target: Some(Actor::Enemy(owner)), kind: EffectKind::Damage(DamagePayload::Fixed(999)) };
    combat.enqueue_native_effects(vec![kill(dead_owner.clone()), kill(later_dead.clone()), work[0].clone()], None).unwrap();
    assert_eq!(combat.public_full().unwrap().enemies.iter().find(|enemy| enemy.instance_id == dead_owner).unwrap().hp, 0);
    assert_eq!(combat.public_full().unwrap().enemies.iter().find(|enemy| enemy.instance_id == later_dead).unwrap().hp, 0);
    assert_eq!(power_amount(&combat, &Actor::Enemy(final_live), "poison"), Some(6),
        "the callback captured Poison 5, then the shared resolver chooses the only live enemy and Skull adds one");
}

#[test]
fn m15_silent_potions_use_the_owned_belt_bark_accuracy_overflow_and_expiry() {
    let poison_entry = entry(&[CardId::StrikeGreen; 5], 1, 1, EnemyId::Cultist,
        FullIntentId::Incantation, &[], &[]);
    let outside = crate::items::NativePotionUseContext { phase: crate::items::PotionRoomPhase::OutsideCombat,
        turn_ended: false, we_meet_again: false, hp: 35, max_hp: 80, live_targets: &[] };
    for potion in [PotionId::PoisonPotion, PotionId::CunningPotion, PotionId::GhostInAJar] {
        let owned = CombatItemContext::new(vec![], vec![Some(potion)]).with_m15_profile();
        assert!(matches!(crate::items::potion_use_effects(&owned, 0, None, &outside),
            Err(crate::run::RunError::InvalidAction)), "{potion:?} is combat-only");
    }

    let mut missing_target = enter(poison_entry.clone(), vec![], vec![Some(PotionId::PoisonPotion)]);
    assert_eq!(missing_target.step_full(&CombatAction::UsePotion { slot: 0, target: None }),
        Err(crate::run::RunError::InvalidAction));
    assert_eq!(missing_target.item_context().unwrap().potions()[0], Some(PotionId::PoisonPotion),
        "target rejection leaves the real belt slot untouched");

    let plain = enter(poison_entry.clone(), vec![], vec![Some(PotionId::PoisonPotion)]);
    let mut plain = plain;
    plain.step_full(&CombatAction::UsePotion { slot: 0, target: Some(TargetId(1)) }).unwrap();
    let enemy = Actor::Enemy(EnemyHandle::parse("enemy-1").unwrap());
    assert_eq!(power_amount(&plain, &enemy, "poison"), Some(6));
    assert_eq!(plain.item_context().unwrap().potions()[0], None, "the live potion belt consumes this one slot");
    assert_eq!(plain.step_full(&CombatAction::UsePotion { slot: 0, target: Some(TargetId(1)) }),
        Err(crate::run::RunError::InvalidAction), "a consumed slot cannot be used again");

    let mut bark = enter(poison_entry.clone(), vec![RelicId::SacredBark, RelicId::ToyOrnithopter],
        vec![Some(PotionId::PoisonPotion)]);
    bark.state.player.hp = 30;
    let used = bark.step_full_observed(&CombatAction::UsePotion { slot: 0, target: Some(TargetId(1)) });
    used.result.unwrap();
    assert_eq!(power_amount(&bark, &enemy, "poison"), Some(12));
    assert_eq!(bark.public_full().unwrap().player.hp, 35, "Toy Ornithopter's supplied callback heals five");
    assert_eq!(bark.item_context().unwrap().potions()[0], None);
    let poison_seq = used.effects.iter().find(|event| matches!((&event.target, &event.kind),
        (Some(Actor::Enemy(_)), ResolvedEffectKind::Power { id, after: Some(12), .. }) if id.0 == "poison"))
        .unwrap().sequence;
    let heal_seq = used.effects.iter().find(|event| matches!(event.kind, ResolvedEffectKind::Heal { .. }))
        .unwrap().sequence;
    assert!(poison_seq < heal_seq, "Ornithopter runs after the potion's own shared effect");

    let mut plain_cunning = enter(entry(&[CardId::StrikeGreen; 10], 1, 1, EnemyId::Cultist,
        FullIntentId::Incantation, &[], &[full_power("accuracy", 2)]),
        vec![], vec![Some(PotionId::CunningPotion)]);
    plain_cunning.step_full(&CombatAction::UsePotion { slot: 0, target: None }).unwrap();
    let view = plain_cunning.public_full().unwrap();
    assert_eq!(view.hand.iter().filter(|card| card.id == CardId::Shiv && card.upgrades == 1).count(), 3,
        "ordinary Cunning Potion creates three upgraded Shivs");
    assert!(view.hand.iter().filter(|card| card.id == CardId::Shiv)
        .all(|card| card.values.as_ref().and_then(|values| values.base_damage) == Some(8)),
        "Accuracy applies to the three ordinary generated Shivs");

    let mut cunning = enter(entry(&[CardId::StrikeGreen; 10], 8, 1, EnemyId::Cultist,
        FullIntentId::Incantation, &[], &[full_power("accuracy", 3)]),
        vec![RelicId::SacredBark], vec![Some(PotionId::CunningPotion)]);
    cunning.step_full(&CombatAction::UsePotion { slot: 0, target: None }).unwrap();
    let view = cunning.public_full().unwrap();
    assert_eq!(view.hand.len(), 10);
    assert_eq!(view.hand.iter().filter(|card| card.id == CardId::Shiv && card.upgrades == 1).count(), 2,
        "Sacred Bark creates six upgraded Shivs through the ordinary generated-card path");
    assert_eq!(view.discard.iter().filter(|card| card.id == CardId::Shiv && card.upgrades == 1).count(), 4,
        "hand-cap overflow preserves upgraded Shivs in discard");
    assert!(view.hand.iter().filter(|card| card.id == CardId::Shiv)
        .all(|card| card.values.as_ref().and_then(|values| values.base_damage) == Some(9)),
        "Accuracy is attached to generated Shivs as the shared create-card effect commits");
    assert!(cunning.state.piles.discard.iter().filter_map(crate::combat::model::OwnedCard::full)
        .filter(|card| card.id == CardId::Shiv).all(|card| card.local.base_damage == Some(9)),
        "Accuracy is retained on upgraded Shivs routed directly to discard by the hand cap");
    assert_eq!(cunning.item_context().unwrap().potions()[0], None);

    let mut ghost = enter(poison_entry, vec![], vec![Some(PotionId::GhostInAJar)]);
    ghost.step_full(&CombatAction::UsePotion { slot: 0, target: None }).unwrap();
    assert_eq!(power_amount(&ghost, &Actor::Player, "intangible_player"), Some(1));
    ghost.step_full(&CombatAction::EndTurn).unwrap();
    assert_eq!(power_amount(&ghost, &Actor::Player, "intangible_player"), None,
        "the source duration expires at the player's round-end power checkpoint");

    let mut bark_ghost = enter(entry(&[CardId::StrikeGreen; 5], 1, 1, EnemyId::Cultist,
        FullIntentId::Incantation, &[], &[]), vec![RelicId::SacredBark],
        vec![Some(PotionId::GhostInAJar)]);
    bark_ghost.step_full(&CombatAction::UsePotion { slot: 0, target: None }).unwrap();
    assert_eq!(power_amount(&bark_ghost, &Actor::Player, "intangible_player"), Some(2));
    let bark_ended = bark_ghost.step_full_observed(&CombatAction::EndTurn);
    bark_ended.result.unwrap();
    assert_eq!(power_amount(&bark_ghost, &Actor::Player, "intangible_player"), Some(1),
        "one round-end decrements Sacred Bark's two-turn Intangible once");
    assert_eq!(bark_ended.effects.iter().filter(|event| matches!(&event.kind,
        ResolvedEffectKind::Power { id, before: Some(_), .. } if id.0 == "intangible_player")).count(), 1,
        "one TurnEnded checkpoint queues one duration decrement");
    bark_ghost.step_full(&CombatAction::EndTurn).unwrap();
    assert_eq!(power_amount(&bark_ghost, &Actor::Player, "intangible_player"), None,
        "Bark's two-round Ghost duration expires after its second round-end checkpoint");
}

#[test]
fn m15_specimen_actual_death_follows_corpse_explosion_and_shared_poison() {
    let mut setup = entry(&[CardId::StrikeGreen; 5], 1, 3, EnemyId::Cultist,
        FullIntentId::Incantation, &[], &[]);
    setup.setup.enemies[0].powers = vec![full_power("poison", 5), full_power("corpse_explosion", 1)];
    for enemy in &mut setup.setup.enemies[1..] { enemy.hp = 100; enemy.max_hp = 100; }
    let mut combat = enter(setup, vec![RelicId::TheSpecimen, RelicId::SneckoSkull], vec![]);
    combat.enqueue_native_effects(vec![Effect {
        source: Source::Actor(Actor::Player),
        target: Some(Actor::Enemy(EnemyHandle::parse("enemy-1").unwrap())),
        kind: EffectKind::Damage(DamagePayload::Fixed(999)),
    }], None).unwrap();
    let view = combat.public_full().unwrap();
    assert_eq!(view.enemies[0].hp, 0);
    assert_eq!(view.enemies[1].hp, 52);
    assert_eq!(view.enemies[2].hp, 52);
    let poison: i32 = view.enemies[1..].iter().flat_map(|enemy| &enemy.powers)
        .filter(|power| power.id.0 == "poison").map(|power| power.amount).sum();
    assert_eq!(poison, 6, "actual death captures Poison5, queues Corpse first, then Specimen uses shared Skull application");
}

#[test]
fn m15_potion_generation_tests_shared_power_pairs_use_live_slots_and_end_turn_callbacks() {
    for bark in [false,true] {
        let factor=if bark{2}else{1};
        for (potion,power,amount) in [
            (PotionId::FlexPotion,"strength",5),(PotionId::SpeedPotion,"dexterity",5),
            (PotionId::RegenPotion,"regeneration",5),(PotionId::LiquidBronze,"thorns",3),
            (PotionId::EssenceOfSteel,"plated_armor",4),(PotionId::CultistPotion,"ritual",1),
        ] {
            let relics=if bark {vec![RelicId::SacredBark]}else{vec![]};
            let mut c=enter(entry(&[CardId::DefendGreen;8],1,1,EnemyId::Cultist,
                FullIntentId::Incantation,&[],&[]),relics,vec![Some(potion)]);
            c.step_full(&CombatAction::UsePotion{slot:0,target:None}).unwrap();
            assert_eq!(c.item_context().unwrap().potions()[0],None);
            assert_eq!(power_amount(&c,&Actor::Player,power),Some(amount*factor));
            let observed=c.step_full_observed(&CombatAction::EndTurn);observed.result.unwrap();
            match potion {
                PotionId::FlexPotion|PotionId::SpeedPotion=>assert_eq!(power_amount(&c,&Actor::Player,power),None),
                PotionId::RegenPotion=>{
                    assert_eq!(c.state.player.hp,35+5*factor);
                    assert_eq!(power_amount(&c,&Actor::Player,power),Some(5*factor-1));
                }
                PotionId::EssenceOfSteel=>assert!(observed.effects.iter().any(|effect|
                    effect.target==Some(Actor::Player)&&matches!(effect.kind,ResolvedEffectKind::Block{before:0,after} if after==4*factor))),
                PotionId::CultistPotion=>assert_eq!(power_amount(&c,&Actor::Player,"strength"),Some(factor)),
                _=>(),
            }
        }
    }
    for (potion,power) in [(PotionId::FlexPotion,"strength"),(PotionId::SpeedPotion,"dexterity")] {
        let mut c=enter(entry(&[CardId::DefendGreen;8],1,1,EnemyId::Cultist,
            FullIntentId::Incantation,&[],&[full_power("artifact",1)]),vec![],vec![Some(potion)]);
        c.step_full(&CombatAction::UsePotion{slot:0,target:None}).unwrap();
        assert_eq!(power_amount(&c,&Actor::Player,"artifact"),None,"Artifact blocks the separate stat-loss debuff");
        c.step_full(&CombatAction::EndTurn).unwrap();
        assert_eq!(power_amount(&c,&Actor::Player,power),Some(5));
    }
}

#[test]
fn m15_potion_generation_tests_bronze_and_steel_share_real_damage_callbacks() {
    for (damage,expected_armor,expected_enemy_hp) in [
        (DamagePayload::Fixed(10),3,45),(DamagePayload::Thorns(10),4,48),(DamagePayload::HpLoss(10),4,48),
    ] {
        let mut c=enter(entry(&[CardId::Burn;8],1,1,EnemyId::Cultist,
            FullIntentId::Incantation,&[],&[]),vec![],vec![Some(PotionId::LiquidBronze),Some(PotionId::EssenceOfSteel)]);
        c.step_full(&CombatAction::UsePotion{slot:0,target:None}).unwrap();
        c.step_full(&CombatAction::UsePotion{slot:1,target:None}).unwrap();
        let source=if matches!(damage,DamagePayload::Thorns(_)) {
            Source::Card(c.state.piles.hand[0].handle()) // Actual Burn THORNS producer.
        } else {Source::Actor(Actor::Enemy(EnemyHandle::parse("enemy-1").unwrap()))};
        c.enqueue_native_effects(vec![Effect{source,
            target:Some(Actor::Player),kind:EffectKind::Damage(damage)}],None).unwrap();
        assert_eq!(c.state.player.hp,25);
        assert_eq!(power_amount(&c,&Actor::Player,"plated_armor"),Some(expected_armor));
        assert_eq!(c.state.enemies[0].hp,expected_enemy_hp);
    }
}

#[test]
fn m15_potion_generation_tests_forge_upgrades_hand_once_and_snecko_randomizes_after_draw() {
    for bark in [false,true] {
        let relics=if bark{vec![RelicId::SacredBark]}else{vec![]};
        let mut forge=enter(entry(&[CardId::SearingBlow,CardId::GlassKnife,CardId::DefendGreen],3,1,
            EnemyId::Cultist,FullIntentId::Incantation,&[],&[]),relics.clone(),vec![Some(PotionId::BlessingOfTheForge)]);
        forge.step_full(&CombatAction::UsePotion{slot:0,target:None}).unwrap();
        for card in &forge.state.piles.hand {
            let card=card.full().unwrap();assert_eq!(card.upgrades,1);assert_eq!(card.local.temporary_upgrades,1);
            if card.id==CardId::GlassKnife{assert_eq!(card.local.base_damage,Some(12));}
        }
        let mut oil_entry=entry(&[CardId::DefendGreen,CardId::Reflex,CardId::Whirlwind,
            CardId::GlassKnife,CardId::DefendGreen,CardId::DefendGreen,CardId::DefendGreen,CardId::DefendGreen,
            CardId::DefendGreen,CardId::DefendGreen,CardId::DefendGreen,CardId::DefendGreen],1,1,
            EnemyId::Cultist,FullIntentId::Incantation,&[],&[]);
        for card in &mut oil_entry.cards { *card=crate::combat::permanent_live_card(card.id,card.instance,card.upgrades); }
        let mut oil=enter(oil_entry,relics,vec![Some(PotionId::SneckoOil)]);
        let before=oil.state.card_rng.clone();
        oil.step_full(&CombatAction::UsePotion{slot:0,target:None}).unwrap();
        assert_eq!(oil.state.piles.hand.len(),if bark{10}else{6});
        assert_ne!(oil.state.card_rng,before);
        for card in oil.state.piles.hand.iter().filter_map(crate::combat::model::OwnedCard::full) {
            if card.local.x_cost||card.id==CardId::Reflex {assert_eq!(card.local.randomized_cost,None);}
            else {
                if let Some(cost)=card.local.randomized_cost {
                    assert!((0..=3).contains(&cost));assert_eq!(card.local.turn_cost,Some(cost));
                    if card.id==CardId::GlassKnife{assert_eq!(card.local.mutable_combat_cost,Some(cost));}
                } else {assert_eq!(card.local.mutable_combat_cost,if card.id==CardId::GlassKnife{Some(1)}else{None});}
            }
        }
    }
}

#[test]
fn m15_potion_generation_tests_fairy_rescues_damage_without_manual_use_or_ornithopter() {
    for (bark,bloom) in [(false,false),(true,false),(false,true)] {
        let mut relics=vec![RelicId::ToyOrnithopter];if bark{relics.push(RelicId::SacredBark);}if bloom{relics.push(RelicId::MarkOfBloom);}
        let mut c=enter(entry(&[CardId::DefendGreen;8],1,1,EnemyId::Cultist,
            FullIntentId::Incantation,&[],&[]),relics,vec![Some(PotionId::FairyPotion)]);
        let before=c.snapshot();
        assert_eq!(c.step_full(&CombatAction::UsePotion{slot:0,target:None}),Err(crate::run::RunError::InvalidAction));
        assert_eq!(c.snapshot(),before);
        c.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Enemy(EnemyHandle::parse("enemy-1").unwrap())),
            target:Some(Actor::Player),kind:EffectKind::Damage(DamagePayload::Fixed(999))}],None).unwrap();
        assert_eq!(c.state.player.hp,if bloom{0}else if bark{48}else{24},"rescue heals30/60% without Ornithopter's use callback");
        assert_eq!(c.item_context().unwrap().potions()[0],if bloom{Some(PotionId::FairyPotion)}else{None});
        assert_eq!(c.public_full().unwrap().outcome,if bloom{Some(crate::combat::Outcome::Death)}else{None});
    }
    let mut tiny=entry(&[CardId::DefendGreen;8],1,1,EnemyId::Cultist,FullIntentId::Incantation,&[],&[]);
    tiny.setup.player.hp=1;tiny.setup.player.max_hp=1;
    let mut c=enter(tiny,vec![],vec![Some(PotionId::FairyPotion),Some(PotionId::FairyPotion)]);
    let hit=Effect{source:Source::Actor(Actor::Enemy(EnemyHandle::parse("enemy-1").unwrap())),
        target:Some(Actor::Player),kind:EffectKind::Damage(DamagePayload::Fixed(1))};
    c.enqueue_native_effects(vec![hit.clone(),hit],None).unwrap();
    assert_eq!(c.state.player.hp,1,"each actual damage checkpoint uses the minimum-one rescue");
    assert_eq!(c.item_context().unwrap().potions(),&[None,None]);
}

#[test]
fn m15_potion_generation_tests_forge_uses_shared_cost_damage_and_exhaust_upgrade_writers() {
    let ids=[CardId::Apotheosis,CardId::Madness,CardId::SecretTechnique,CardId::SecretWeapon,
        CardId::ThinkingAhead,CardId::Skewer,CardId::Nightmare,CardId::Terror,CardId::MindBlast,CardId::Shiv];
    let mut setup=entry(&ids,10,1,EnemyId::Cultist,FullIntentId::Incantation,&[],&[]);
    for card in &mut setup.cards { *card=crate::combat::permanent_live_card(card.id,card.instance,card.upgrades); }
    let mut c=enter(setup,vec![],vec![Some(PotionId::BlessingOfTheForge)]);
    c.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),
        kind:EffectKind::ApplyFullPower{id:crate::combat::FullPowerId("accuracy"),amount:3}}],None).unwrap();
    c.step_full(&CombatAction::UsePotion{slot:0,target:None}).unwrap();
    for card in c.state.piles.hand.iter().filter_map(crate::combat::model::OwnedCard::full) {
        let expected=match card.id {CardId::Apotheosis=>Some(1),CardId::Madness|CardId::Terror=>Some(0),
            CardId::Nightmare=>Some(2),CardId::MindBlast=>Some(1),_=>None};
        if let Some(cost)=expected {assert_eq!(crate::combat::card_state::effective_cost(card,crate::combat::cards::base_cost(&c.state,card).unwrap()),Ok(cost),"{:?} source upgraded payment",card.id);}
        if matches!(card.id,CardId::SecretTechnique|CardId::SecretWeapon|CardId::ThinkingAhead){assert!(!card.local.exhaust_on_finish,"upgraded retrieval/ThinkingAhead do not Exhaust");}
        if card.id==CardId::Skewer{assert_eq!(card.local.base_damage,Some(10));}
        if card.id==CardId::Shiv{assert_eq!(card.local.base_damage,Some(9));}
    }
}

#[test]
fn m15_regeneration_heals_before_end_turn_burn_damage() {
    let mut c=enter(entry(&[CardId::Burn],1,1,EnemyId::Cultist,FullIntentId::Incantation,&[],&[]),vec![],vec![Some(PotionId::RegenPotion)]);
    c.state.player.hp=1;
    c.step_full(&CombatAction::UsePotion{slot:0,target:None}).unwrap();
    c.step_full(&CombatAction::EndTurn).unwrap();
    assert_eq!(c.state.player.hp,4);
    assert_eq!(c.state.outcome,None);
}
