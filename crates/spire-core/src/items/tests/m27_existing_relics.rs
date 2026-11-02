//! M27 plan Task M3, lane Main, key m27_existing_relics.
//! Empty handoff scaffold for accepted-relic regression tests owned by Main.

use crate::content::{CardId, Character, EncounterId, RelicId};
use std::collections::BTreeMap;

const M27_CHARACTERS: [Character; 4] = [Character::Ironclad, Character::Silent, Character::Defect, Character::Watcher];

fn accepted_character(relic: RelicId) -> Character {
    let catalog = crate::content::Catalog::standard();
    let preferred = match relic {
        RelicId::ChemicalX => Some(Character::Defect),
        RelicId::PotionBelt | RelicId::Sozu => Some(Character::Silent),
        _ => None,
    };
    preferred.or_else(|| M27_CHARACTERS.into_iter().find(|character|
        catalog.relic_eligible_for(relic, *character) == Some(true))
    )
        .unwrap_or_else(|| panic!("accepted relic {relic:?} has no eligible character"))
}

fn accepted_context(character: Character, relic: RelicId) -> crate::items::CombatItemContext {
    let potion_slots = if relic == RelicId::PotionBelt { 5 } else { 3 };
    let context = crate::items::CombatItemContext::new(vec![relic], vec![None; potion_slots]);
    match character {
        Character::Ironclad => context,
        Character::Silent => context.with_m15_profile(),
        Character::Defect => context.with_m20_profile(),
        Character::Watcher => context.with_m25_profile(),
    }
}

fn complete_context(character: Character, relic: RelicId) -> crate::items::CombatItemContext {
    let potion_slots = if relic == RelicId::PotionBelt { 5 } else { 3 };
    crate::run::complete_heart_context(
        crate::items::CombatItemContext::new(vec![relic], vec![None; potion_slots]), character,
    ).with_complete_items()
}

fn entered_combat(character: Character, context: crate::items::CombatItemContext)
    -> crate::combat::Combat {
    let mut entry = super::full_entry(character, 5, 10);
    if character == Character::Watcher {
        entry.cards = (1..=10).map(|instance|
            crate::combat::m25_watcher_live_card(CardId::DefendPurple,
                crate::combat::CardInstanceId(instance), 0).unwrap()
        ).collect();
    }
    let mut owner = Some(context);
    crate::combat::Combat::enter_with_item_context(entry, &mut owner)
        .unwrap_or_else(|error| panic!("{character:?} accepted/complete combat entry: {error:?}"))
}

fn public_entry(character: Character, context: crate::items::CombatItemContext)
    -> crate::combat::CombatPublic {
    entered_combat(character, context).public_full().expect("full public combat at ready")
}

fn accepted_phase_work(context: &mut crate::items::CombatItemContext, hook: crate::combat::Hook)
    -> crate::items::RelicHookWork {
    let enemy = crate::combat::EnemyHandle::parse("m27-accepted-target").unwrap();
    let mut facts = crate::items::dispatch::PhaseFacts::new(Some(hook), 40, 80, 0);
    facts.enemies = std::slice::from_ref(&enemy);
    facts.live_enemies = std::slice::from_ref(&enemy);
    facts.hand_size = 3;
    facts.draw_size = 5;
    facts.discard_size = 1;
    facts.dead_enemy = Some((&enemy, Some(3), 0));
    crate::items::relic_phase_effects(context, &facts).unwrap()
}

const M27_ACCEPTED_HOOKS: [crate::combat::Hook; 13] = [
    crate::combat::Hook::CombatStartedPreDraw,
    crate::combat::Hook::CombatStarted,
    crate::combat::Hook::TurnStartedPreDraw,
    crate::combat::Hook::TurnStarted,
    crate::combat::Hook::CardPlayed,
    crate::combat::Hook::CardAfterUse,
    crate::combat::Hook::AttackResolved,
    crate::combat::Hook::CardManuallyDiscarded,
    crate::combat::Hook::DrawPileShuffled,
    crate::combat::Hook::PlayerPreEndTurnCards,
    crate::combat::Hook::CloakClaspBeforeCleanup { hand_size: 3 },
    crate::combat::Hook::PlayerEndTurn,
    crate::combat::Hook::CombatSettled,
];

fn miracle_count<'a>(effects: impl Iterator<Item = &'a crate::combat::Effect>) -> usize {
    effects.map(|effect| match &effect.kind {
        crate::combat::EffectKind::CreateCard { id: CardId::Miracle, .. } => 1,
        crate::combat::EffectKind::Watcher(crate::combat::cards::watcher::WatcherEffect::CreateRequest {
            id: CardId::Miracle, count,
        }) => *count as usize,
        crate::combat::EffectKind::Watcher(crate::combat::cards::watcher::WatcherEffect::Create {
            template, count, ..
        }) if template.id == CardId::Miracle => *count as usize,
        _ => 0,
    }).sum()
}

#[test]
fn m27_existing_relics_equal_accepted() {
    use crate::items::relic_effects::ACCEPTED_RELICS;
    assert_eq!(ACCEPTED_RELICS.len(), 62, "the accepted relic inventory remains the contracted set");
    for relic in ACCEPTED_RELICS {
        let character = accepted_character(*relic);
        let accepted_public = public_entry(character, accepted_context(character, *relic));
        let complete_public = public_entry(character, complete_context(character, *relic));
        assert_eq!(accepted_public, complete_public,
            "{relic:?} ready public delta under its accepted and complete contexts");
        let mut accepted = accepted_context(character, *relic);
        let mut complete = complete_context(character, *relic);
        let accepted_start = crate::items::startup_work(&mut accepted).unwrap();
        let complete_start = crate::items::startup_work(&mut complete).unwrap();
        if matches!(relic, RelicId::PureWater | RelicId::HolyWater) {
            let expected = if *relic == RelicId::PureWater { 1 } else { 3 };
            let accepted_pre_draw = accepted_phase_work(&mut accepted, crate::combat::Hook::CombatStartedPreDraw);
            let complete_pre_draw = accepted_phase_work(&mut complete, crate::combat::Hook::CombatStartedPreDraw);
            let accepted_count = miracle_count(accepted_start.iter()
                .chain(accepted_pre_draw.top.iter()).chain(accepted_pre_draw.bottom.iter()));
            let complete_count = miracle_count(complete_start.iter()
                .chain(complete_pre_draw.top.iter()).chain(complete_pre_draw.bottom.iter()));
            assert_eq!(accepted_count, expected, "accepted {relic:?} pre-draw value");
            assert_eq!(complete_count, expected, "complete {relic:?} pre-draw value");
            assert_eq!(miracle_count(complete_start.iter()), 0,
                "complete {relic:?} waits for its queued pre-draw hook");
            assert_eq!(miracle_count(complete_pre_draw.top.iter().chain(&complete_pre_draw.bottom)), expected,
                "complete {relic:?} fires on its one queued pre-draw hook");
        } else {
            assert_eq!(accepted_start, complete_start, "{relic:?} startup delta");
            for hook in M27_ACCEPTED_HOOKS {
                if *relic == RelicId::Orichalcum && matches!(hook,
                    crate::combat::Hook::PlayerEndTurn|crate::combat::Hook::CloakClaspBeforeCleanup { .. }) {
                    continue;
                }
                assert_eq!(accepted_phase_work(&mut accepted, hook), accepted_phase_work(&mut complete, hook),
                    "{relic:?} {hook:?} delta against its accepted context");
            }
            if *relic == RelicId::Orichalcum {
                let accepted_end = accepted_phase_work(&mut accepted, crate::combat::Hook::PlayerEndTurn);
                let accepted_cleanup = accepted_phase_work(&mut accepted,
                    crate::combat::Hook::CloakClaspBeforeCleanup { hand_size: 3 });
                let complete_end = accepted_phase_work(&mut complete, crate::combat::Hook::PlayerEndTurn);
                let complete_cleanup = accepted_phase_work(&mut complete,
                    crate::combat::Hook::CloakClaspBeforeCleanup { hand_size: 3 });
                assert_eq!(accepted_end.top.iter().chain(&accepted_cleanup.top).cloned().collect::<Vec<_>>(),
                    complete_end.top.iter().chain(&complete_cleanup.top).cloned().collect::<Vec<_>>(),
                    "M2 Orichalcum migration keeps one six-Block event across both checkpoints");
                assert!(complete_end.is_empty(), "complete Orichalcum does not grant Block on PlayerEndTurn");
                assert_eq!(complete_cleanup.top.iter().filter(|effect|
                    effect.kind == crate::combat::EffectKind::GainBlock(6)).count(), 1,
                    "complete Orichalcum keeps one accepted six-Block end-turn event at cleanup");
            }
        }
    }

    let vajra = public_entry(Character::Ironclad, complete_context(Character::Ironclad, RelicId::Vajra));
    assert_eq!(vajra.player.powers.iter().find(|power| power.id.0 == "strength").map(|power| power.amount), Some(1),
        "Vajra grants its accepted +1 Strength at combat start");

    let anchor = public_entry(Character::Ironclad, complete_context(Character::Ironclad, RelicId::Anchor));
    assert_eq!(anchor.player.block, 10, "Anchor resolves its accepted 10 Block at combat start");
    let mut anchor_work = complete_context(Character::Ironclad, RelicId::Anchor);
    let anchor_start = accepted_phase_work(&mut anchor_work, crate::combat::Hook::CombatStarted);
    assert!(anchor_start.bottom.iter().any(|effect| effect.kind == crate::combat::EffectKind::GainBlock(10)),
        "Anchor's accepted 10 Block stays bottom queued");

    for (character, relic, source) in [
        (Character::Ironclad, RelicId::BagOfPreparation, "Bag of Preparation"),
        (Character::Silent, RelicId::RingOfTheSnake, "Ring of the Snake"),
    ] {
        let public = public_entry(character, complete_context(character, relic));
        assert_eq!(public.hand.len(), 7, "{source} keeps its two accepted turn-one draws");
    }

    for (relic, expected_heal) in [(RelicId::BurningBlood, 6), (RelicId::BlackBlood, 12)] {
        for context in [accepted_context(Character::Ironclad, relic),
            complete_context(Character::Ironclad, relic)] {
            let mut entry = super::full_entry(Character::Ironclad, 5, 10);
            entry.setup.player.hp = 40;
            entry.setup.player.max_hp = 80;
            let mut owner = Some(context);
            let mut combat = crate::combat::Combat::enter_with_item_context(entry, &mut owner)
                .unwrap_or_else(|error| panic!("{relic:?} settlement context entry: {error:?}"));
            assert_eq!(combat.public_full().unwrap().player.hp, 40);
            combat.enqueue_hook(crate::combat::HookEvent { hook: crate::combat::Hook::CombatSettled,
                source: crate::combat::Source::Actor(crate::combat::Actor::Player),
                target: Some(crate::combat::Actor::Player) }).unwrap();
            assert_eq!(combat.public_full().unwrap().player.hp, 40 + expected_heal,
                "{relic:?} applies its accepted {expected_heal} HP settlement heal");
        }
    }

    // Accepted literal scenarios: Vajra and Bag of Preparation, m9_relics.rs:25-44;
    // Ring of the Snake, m9_relics.rs:1-22; Orichalcum, m10_ironclad_relics.rs:33-53;
    // Burning Blood/Black Blood, shared_mechanics.rs:307-318; Watcher startup values,
    // m25_watcher.rs:196-208 and m23_watcher_act2.rs:398-405.
}

#[test]
fn m27_shared_accepted_relics_all_characters() {
    use crate::items::relic_effects::ACCEPTED_RELICS;
    let catalog = crate::content::Catalog::standard();
    let shared = ACCEPTED_RELICS.iter().copied().filter(|relic|
        M27_CHARACTERS.into_iter().all(|character| catalog.relic_eligible_for(*relic, character) == Some(true)))
        .collect::<Vec<_>>();
    assert_eq!(shared.len(), 22, "the accepted shared-relic set stays finite");
    for relic in shared {
        for character in M27_CHARACTERS {
            let context = complete_context(character, relic);
            crate::items::validate_combat_inventory(&context, character)
                .unwrap_or_else(|error| panic!("complete {character:?} shared {relic:?} admission: {error:?}"));
            assert!(context.owns_relic(relic), "complete {character:?} inventory retains shared {relic:?}");
            for hook in M27_ACCEPTED_HOOKS {
                let mut ironclad = complete_context(Character::Ironclad, relic);
                let mut other = context.clone();
                assert_eq!(accepted_phase_work(&mut ironclad, hook), accepted_phase_work(&mut other, hook),
                    "shared {relic:?} {hook:?} work for {character:?} matches Ironclad");
            }
        }
    }

    let mut ironclad = complete_context(Character::Ironclad, RelicId::Anchor);
    let anchor = accepted_phase_work(&mut ironclad, crate::combat::Hook::CombatStarted);
    assert!(anchor.bottom.iter().any(|effect| effect.kind == crate::combat::EffectKind::GainBlock(10)),
        "Ironclad keeps Anchor's accepted 10 Block delta on the bottom side");
}

#[test]
fn m27_pure_water_phase() {
    use crate::items::{CombatItemContext, dispatch::PhaseFacts};

    for (relic, expected_miracles) in [(RelicId::PureWater, 1), (RelicId::HolyWater, 3)] {
        let mut context = crate::run::complete_heart_context(
            CombatItemContext::new(vec![relic], vec![None; 3]),
            Character::Watcher,
        ).with_complete_items();
        let facts = PhaseFacts::new(None, 80, 80, 0);
        let startup = crate::items::startup_work_with_facts(&mut context, &facts)
            .unwrap_or_else(|error| panic!("{relic:?} complete startup dispatch: {error:?}"));

        assert_eq!(miracle_count(startup.iter()), 0,
            "{relic:?} waits for the one pre-draw visit selected by the queued hook");
        let pre_draw = accepted_phase_work(&mut context, crate::combat::Hook::CombatStartedPreDraw);
        assert_eq!(miracle_count(pre_draw.top.iter().chain(&pre_draw.bottom)), expected_miracles,
            "{relic:?} creates its Miracles at the one queued pre-draw hook visit");

        let run = super::m27_support::complete_run(
            Character::Watcher, vec![relic], vec![None; 3], BTreeMap::new(), EncounterId::Cultist,
        );
        let combat = run.state().combat.as_ref().expect("complete Run entered combat");
        let public = combat.public_full().expect("full public combat");
        assert_eq!(public.hand.iter().filter(|card| card.id == CardId::Miracle).count(), expected_miracles,
            "{relic:?} Miracles are in hand before the opening draw completes");
        assert_eq!(public.hand.len(), 5 + expected_miracles,
            "{relic:?} adds its Miracles without taking opening-draw slots");
        assert_eq!(public.draw_count, 1,
            "{relic:?} leaves one of six starter cards after the five-card opening draw");
    }
}

#[test]
fn m27_g10_legacy_pure_water_pre_draw() {
    let context = crate::items::CombatItemContext::new(vec![RelicId::PureWater], vec![None; 3]);
    let mut startup_context = context.clone();
    let startup = crate::items::startup_work(&mut startup_context)
        .expect("legacy Pure Water startup dispatch succeeds");
    assert_eq!(miracle_count(startup.iter()), 1,
        "legacy Pure Water fires at its hook-free pre-draw visit");

    let mut legacy_holy_context = crate::items::CombatItemContext::new(vec![RelicId::HolyWater], vec![None; 3]);
    let legacy_holy_startup = crate::items::startup_work(&mut legacy_holy_context)
        .expect("legacy Holy Water startup dispatch succeeds");
    // G10 (M27): both startup relics use the hook-free visit in Legacy contexts (design §7.6; PM ruling G10).
    assert_eq!(miracle_count(legacy_holy_startup.iter()), 3,
        "legacy Holy Water fires all three Miracles at its hook-free pre-draw visit");

    let combat = entered_combat(Character::Ironclad, context);
    let public = combat.public_full().expect("full public legacy combat");
    assert_eq!(public.hand.iter().filter(|card| card.id == CardId::Miracle).count(), 1,
        "legacy Pure Water creates exactly one Miracle before the opening draw");
    assert_eq!(public.hand.len(), 6, "the Miracle does not consume an opening-draw slot");
    assert_eq!(public.draw_count, 5, "the opening draw still draws five cards");
}
