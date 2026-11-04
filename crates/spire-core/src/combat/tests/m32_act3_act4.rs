//! M32 Acts 3–4 behaviour rows (plan item 7; lane D research and design §6.4–§6.7).
use super::super::enemies::{self, EnemyMemory};
use super::super::rng::{self, Domain, StreamState};
use super::super::*;
use crate::content::{CardId, EnemyId, EncounterId};

fn ai_stream(actor: &str) -> StreamState {
    let id = rng::StreamId::new(Domain::RunCombatEnemyAi, vec!["m32-act34".into(), actor.into()]).unwrap();
    rng::make_stream(&[32; 32], rng::Partition::Test, "m32-act34", &id).unwrap()
}

fn stream_with_ticket(actor: &str, bound: u128, wanted: u64) -> (StreamState, u64, StreamState) {
    let template = ai_stream(actor);
    (0..4096).find_map(|cursor| {
        let stream = StreamState::new(template.id().clone(), [0; 32], cursor).unwrap();
        let (ticket, next) = rng::uniform_below(&stream, bound).unwrap();
        (ticket == wanted).then_some((stream, ticket, next))
    }).unwrap_or_else(|| panic!("no {bound}-way stream ticket {wanted} found for {actor}"))
}

fn combat(encounter: EncounterId, ascension: u8, seed: u8) -> Combat {
    Combat::enter(super::m29_a2_owner::setup(encounter, ascension, seed, 0).unwrap()).unwrap()
}

fn set_intent(combat: &mut Combat, slot: usize, name: &'static str) {
    let selected = ActorIntentId::Full(FullIntentId::from_name(name));
    let enemy = &mut combat.state.enemies[slot];
    enemy.intent_id = selected.clone();
    enemy.selected_move = selected.clone();
    if let Some(last) = enemy.move_history.last_mut() { *last = selected; }
    else { enemy.move_history.push(selected); }
    effects::refresh_enemy_values(&mut combat.state).unwrap();
}

fn amount(enemy: &super::super::model::EnemyState, id: &str) -> i32 {
    enemy.full_powers.iter().find(|power| power.id.0 == id).map_or(0, |power| power.state.amount)
}

fn player_amount(combat: &Combat, id: &str) -> i32 {
    combat.state.player_full_powers.iter().find(|power| power.id.0 == id).map_or(0, |power| power.state.amount)
}

#[test]
fn m32_darkling_a17_harden_strength() {
    let memory = EnemyMemory::Darkling { center: false, nip: 9, first: true };
    let (stream, ticket, next) = stream_with_ticket("darkling-harden", 100, 0);
    assert_eq!(ticket, 0, "source branch is selected by the constructed base ticket");
    for ascension in [16, 17, 20] {
        let selected = enemies::select_move_at(EnemyId::Darkling, &[], &memory, &stream, ascension).unwrap();
        assert_eq!(selected.intent.as_str(), "darkling_harden", "A{ascension}: the move identity stays stable");
        assert_eq!(selected.stream, next, "A{ascension}: Harden consumes exactly the existing base draw");
    }

    let mut outcome = Vec::new();
    for (ascension, expected_strength) in [(16, 0), (17, 2), (20, 2)] {
        let mut combat = combat(EncounterId::ThreeDarklings, ascension, 33);
        set_intent(&mut combat, 0, "darkling_harden");
        let before = combat.public_full().unwrap();
        assert_eq!(before.enemies[0].intent.as_str(), "darkling_harden", "A{ascension}: public move identity");
        assert_eq!(before.enemies[0].intent_damage, None, "Harden has no intent damage");

        let effects = enemies::move_work_at(&combat.state.enemies[0], ascension).unwrap();
        combat.step_full(&CombatAction::EndTurn).unwrap();
        assert_eq!(combat.state.enemies[0].block, 12, "A{ascension}: Harden resolves Block 12");
        assert_eq!(amount(&combat.state.enemies[0], "strength"), expected_strength, "A{ascension}: Harden's Strength payload");
        assert!(matches!(&effects[0].kind, EffectKind::GainBlock(12)), "A{ascension}: Block precedes any Strength effect");
        assert_eq!(effects.len(), if expected_strength == 2 { 2 } else { 1 }, "A{ascension}: Harden effect queue length");
        if expected_strength == 2 {
            assert!(matches!(&effects[1].kind, EffectKind::ApplyFullPower { id, amount: 2 } if id.0 == "strength"),
                "A{ascension}: Strength +2 follows Block");
        } else {
            assert_eq!(effects.len(), 1, "A16 preserves the original Block-only payload");
        }
        outcome.push((combat.state.enemies[0].block, amount(&combat.state.enemies[0], "strength")));
    }
    assert_eq!(outcome[1], outcome[2], "A20 Harden matches A19");
}

fn growth_opening_stream(seed: u8) -> StreamState {
    let run_rng = crate::run::RunRng::new([seed; 32], rng::Partition::Test, 0);
    let role = format!("{}-0-0", EnemyId::SpireGrowth.as_str());
    run_rng.derive_combat_stream(Domain::RunCombatEnemyAi, 3, EncounterId::SpireGrowth.as_str(), 0, Some(0), &role).unwrap()
}

#[test]
fn m32_spire_growth_a17_forced_constrict() {
    let (seed, opening_stream, opening_ticket, opening_next) = (0..=u8::MAX).find_map(|seed| {
        let stream = growth_opening_stream(seed);
        let (ticket, next) = rng::uniform_below(&stream, 100).unwrap();
        (ticket == 0).then_some((seed, stream, ticket, next))
    }).expect("a setup seed with the distinguishing ticket 0 exists");
    assert_eq!(opening_ticket, 0, "the same source roll selects Quick below A17 and is overridden at A17");

    for (ascension, expected) in [(16, "growth_quick"), (17, "growth_constrict"), (20, "growth_constrict")] {
        let entry = super::m29_a2_owner::setup(EncounterId::SpireGrowth, ascension, seed, 0).unwrap();
        assert_eq!(entry.setup.enemies[0].intent.as_str(), expected, "A{ascension}: setup opening");
        assert_eq!(entry.setup.enemies[0].ai_rng, opening_next, "A{ascension}: setup consumes one base AI ticket");
    }
    for ascension in [16, 17] {
        let result = enemies::select_move_at(EnemyId::SpireGrowth, &[], &EnemyMemory::SpireGrowth, &opening_stream, ascension);
        assert!(matches!(result, Err(enemies::SelectError::Invalid(InputError::Unsupported(message))) if message == "Growth selection needs actual player power context"),
            "A{ascension}: context-free dispatcher remains Unsupported by contract");
    }

    let mut live_outcome = Vec::new();
    for (ascension, expected) in [(16, "growth_quick"), (17, "growth_constrict"), (20, "growth_constrict")] {
        let mut live = combat(EncounterId::SpireGrowth, ascension, seed);
        set_intent(&mut live, 0, "growth_quick");
        let (stream, ticket, next) = stream_with_ticket(&format!("growth-live-{ascension}"), 100, 0);
        assert_eq!(ticket, 0);
        live.state.enemies[0].full_ai_rng = Some(stream);
        live.step_full(&CombatAction::EndTurn).unwrap();
        assert_eq!(live.state.enemies[0].intent_id.as_str(), expected, "A{ascension}: live roll");
        assert_eq!(live.state.enemies[0].full_ai_rng.as_ref().unwrap(), &next, "A{ascension}: forced Constrict keeps the one base draw");
        assert_eq!(live.public_full().unwrap().enemies[0].intent.as_str(), expected, "A{ascension}: public live intent");
        live_outcome.push(live.state.enemies[0].intent_id.as_str().to_owned());
    }
    assert_eq!(live_outcome[1], live_outcome[2], "A20 live selection matches A19");

    let mut payload_outcome = Vec::new();
    for (ascension, expected) in [(16, 10), (17, 12), (20, 12)] {
        let mut payload = combat(EncounterId::SpireGrowth, ascension, seed);
        set_intent(&mut payload, 0, "growth_constrict");
        assert_eq!(payload.public_full().unwrap().enemies[0].intent.as_str(), "growth_constrict", "A{ascension}: public setup payload intent");
        let work = enemies::move_work_at(&payload.state.enemies[0], ascension).unwrap();
        assert!(matches!(work[0].kind, EffectKind::ApplyFullPower { id, amount } if id.0 == "constricted" && amount == expected),
            "A{ascension}: Constricted payload is {expected}");
        payload.step_full(&CombatAction::EndTurn).unwrap();
        assert_eq!(player_amount(&payload, "constricted"), expected, "A{ascension}: resolved Constricted amount");
        payload_outcome.push(player_amount(&payload, "constricted"));
    }
    assert_eq!(payload_outcome[1], payload_outcome[2], "A20 payload matches A19");
}

#[test]
fn m32_deca_a19_square_plated_armor() {
    let mut outcomes = Vec::new();
    for (ascension, expected_armor) in [(18, 0), (19, 3), (20, 3)] {
        let mut live = combat(EncounterId::DonuAndDeca, ascension, 43);
        live.state.enemies[0].full_memory = Some(EnemyMemory::Deca { attacking: false });
        set_intent(&mut live, 0, "deca_square");
        let view = live.public_full().unwrap();
        assert_eq!(view.enemies[0].intent.as_str(), "deca_square", "A{ascension}: public Square intent");
        assert_eq!(view.enemies[0].intent_damage, None, "Square has no damage");

        let mut probe = live.clone();
        let deca = probe.state.enemies[0].instance_id.clone();
        let donu = probe.state.enemies[1].instance_id.clone();
        let work = enemies::act3::take_turn(&mut probe.state, &deca).unwrap();
        let expected_work = if expected_armor == 0 {
            vec![(&deca, false), (&donu, false)]
        } else {
            vec![(&deca, false), (&deca, true), (&donu, false), (&donu, true)]
        };
        assert_eq!(work.len(), expected_work.len(), "A{ascension}: group queue length");
        for (index, (target, is_armor)) in expected_work.into_iter().enumerate() {
            assert_eq!(work[index].target, Some(Actor::Enemy(target.clone())), "A{ascension}: target order {index}");
            if is_armor {
                assert!(matches!(work[index].kind, EffectKind::ApplyFullPower { id, amount: 3 } if id.0 == "plated_armor"),
                    "A{ascension}: Plated Armor immediately follows that member's Block");
            } else {
                assert!(matches!(work[index].kind, EffectKind::GainBlock(16)), "A{ascension}: Block 16 precedes armor");
            }
        }

        live.step_full(&CombatAction::EndTurn).unwrap();
        assert_eq!((live.state.enemies[0].block, amount(&live.state.enemies[0], "plated_armor")), (16 + expected_armor, expected_armor), "A{ascension}: Deca resolves Square and its end-turn armor");
        assert_eq!((live.state.enemies[1].block, amount(&live.state.enemies[1], "plated_armor")), (16 + expected_armor, expected_armor), "A{ascension}: living Donu receives Square and its end-turn armor");
        outcomes.push((live.state.enemies[0].block, amount(&live.state.enemies[0], "plated_armor"), live.state.enemies[1].block, amount(&live.state.enemies[1], "plated_armor")));
    }
    assert_eq!(outcomes[1], outcomes[2], "A20 Deca/Donu Square matches A19");

    let mut one_dead = combat(EncounterId::DonuAndDeca, 19, 43);
    one_dead.state.enemies[0].full_memory = Some(EnemyMemory::Deca { attacking: false });
    set_intent(&mut one_dead, 0, "deca_square");
    one_dead.state.enemies[1].hp = 0;
    one_dead.step_full(&CombatAction::EndTurn).unwrap();
    assert_eq!((one_dead.state.enemies[0].block, amount(&one_dead.state.enemies[0], "plated_armor")), (19, 3));
    assert_eq!((one_dead.state.enemies[1].block, amount(&one_dead.state.enemies[1], "plated_armor")), (0, 0), "dead Donu skips both queued effects");
}

#[test]
fn m32_time_eater_a19_ripple_slam_haste() {
    let mut ripple_outcomes = Vec::new();
    let mut slam_outcomes = Vec::new();
    let mut haste_outcomes = Vec::new();
    for (ascension, expected_frail, expected_slimed, expected_block) in [(18, 0, 0, 0), (19, 1, 2, 32), (20, 1, 2, 32)] {
        let mut ripple = combat(EncounterId::TimeEater, ascension, 53);
        set_intent(&mut ripple, 0, "time_ripple");
        let view = ripple.public_full().unwrap();
        assert_eq!(view.enemies[0].intent.as_str(), "time_ripple", "A{ascension}: public Ripple intent");
        assert_eq!(view.enemies[0].intent_damage, None, "Ripple has no damage");
        let work = enemies::move_work_at(&ripple.state.enemies[0], ascension).unwrap();
        let expected_len = if expected_frail == 0 { 3 } else { 4 };
        assert_eq!(work.len(), expected_len, "A{ascension}: Ripple queue length");
        assert!(matches!(work[0].kind, EffectKind::GainBlock(20)), "Ripple starts with Block 20");
        for (index, id) in [(1, "vulnerable"), (2, "weak")].into_iter().chain((expected_frail > 0).then_some((3, "frail"))) {
            assert!(matches!(work[index].kind, EffectKind::ApplyFullPower { id: power, amount: 1 } if power.0 == id),
                "A{ascension}: Ripple applies {id} in source order");
        }
        ripple.step_full(&CombatAction::EndTurn).unwrap();
        assert_eq!(ripple.state.enemies[0].block, 20, "A{ascension}: Ripple Block resolves");
        assert_eq!((player_amount(&ripple, "vulnerable"), player_amount(&ripple, "weak"), player_amount(&ripple, "frail")), (1, 1, expected_frail), "A{ascension}: Ripple debuffs resolve");
        ripple_outcomes.push((ripple.state.enemies[0].block, player_amount(&ripple, "vulnerable"), player_amount(&ripple, "weak"), player_amount(&ripple, "frail")));

        let mut slam = combat(EncounterId::TimeEater, ascension, 54);
        set_intent(&mut slam, 0, "time_slam");
        let view = slam.public_full().unwrap();
        assert_eq!(view.enemies[0].intent.as_str(), "time_slam", "A{ascension}: public Head Slam intent");
        assert_eq!(view.enemies[0].intent_damage, Some(32), "A{ascension}: Head Slam resolved damage");
        let work = enemies::move_work_at(&slam.state.enemies[0], ascension).unwrap();
        assert_eq!(work.len(), 2 + expected_slimed, "A{ascension}: Head Slam queue length");
        assert!(matches!(work[0].kind, EffectKind::Damage(DamagePayload::FullEnemyMove(intent)) if intent.as_str() == "time_slam"), "Head Slam damage resolves first");
        assert!(matches!(work[1].kind, EffectKind::ApplyFullPower { id, amount: 1 } if id.0 == "draw_reduction"), "Draw Reduction follows damage");
        for card_effect in work.iter().skip(2) {
            assert!(matches!(card_effect.kind, EffectKind::CreateCard { id: CardId::Slimed, destination: super::super::command::CardDestination::Discard, upgrades: 0 }),
                "A{ascension}: two unupgraded Slimed cards follow Draw Reduction");
        }
        let hp_before = slam.state.player.hp;
        slam.step_full(&CombatAction::EndTurn).unwrap();
        let slimed = slam.state.piles.discard.iter().filter_map(|card| card.full()).filter(|card| card.id == CardId::Slimed).count();
        assert_eq!(slam.state.player.hp, hp_before - 32, "A{ascension}: Head Slam resolved damage");
        assert_eq!(player_amount(&slam, "draw_reduction"), 1, "A{ascension}: Draw Reduction resolves");
        assert_eq!(slimed, expected_slimed as usize, "A{ascension}: Slimed cards resolve into discard");
        slam_outcomes.push((slam.state.player.hp, player_amount(&slam, "draw_reduction"), slimed));

        let mut haste = combat(EncounterId::TimeEater, ascension, 55);
        haste.state.enemies[0].hp = 239;
        haste.state.enemies[0].full_memory = Some(EnemyMemory::TimeEater { haste_used: true });
        haste.state.enemies[0].full_powers.push(FullPowerEntry { id: FullPowerId("shackled"), state: PowerState { amount: 1, skip_first: false, monster_source_grace: false } });
        haste.state.enemies[0].full_powers.push(FullPowerEntry { id: FullPowerId("weak"), state: PowerState { amount: 2, skip_first: false, monster_source_grace: false } });
        set_intent(&mut haste, 0, "time_haste");
        let view = haste.public_full().unwrap();
        assert_eq!(view.enemies[0].intent.as_str(), "time_haste", "A{ascension}: public Haste intent");
        assert_eq!(view.enemies[0].intent_damage, None, "Haste has no damage");
        let work = enemies::move_work_at(&haste.state.enemies[0], ascension).unwrap();
        assert_eq!(work.len(), if expected_block == 0 { 3 } else { 4 }, "A{ascension}: Haste queue length");
        assert!(matches!(work[0].kind, EffectKind::TimeEaterCleanse), "Haste begins with cleansing debuffs");
        assert!(matches!(work[1].kind, EffectKind::RemoveFullPower(id) if id.0 == "shackled"), "Shackled removal follows cleanse");
        assert!(matches!(work[2].kind, EffectKind::Heal(1)), "Haste heals to half before its A19 Block");
        if expected_block > 0 { assert!(matches!(work[3].kind, EffectKind::GainBlock(32)), "A19 Haste Block follows healing"); }
        haste.step_full(&CombatAction::EndTurn).unwrap();
        assert_eq!(haste.state.enemies[0].hp, 240, "A{ascension}: Haste heals to half HP");
        assert_eq!(haste.state.enemies[0].block, expected_block, "A{ascension}: Haste Block resolves after healing");
        assert_eq!(amount(&haste.state.enemies[0], "shackled"), 0, "A{ascension}: Haste removes Shackled");
        assert_eq!(amount(&haste.state.enemies[0], "weak"), 0, "A{ascension}: Haste cleanses debuffs");
        haste_outcomes.push((haste.state.enemies[0].hp, haste.state.enemies[0].block, amount(&haste.state.enemies[0], "shackled"), amount(&haste.state.enemies[0], "weak")));
    }
    assert_eq!(ripple_outcomes[1], ripple_outcomes[2], "A20 Ripple matches A19");
    assert_eq!(slam_outcomes[1], slam_outcomes[2], "A20 Head Slam matches A19");
    assert_eq!(haste_outcomes[1], haste_outcomes[2], "A20 Haste matches A19");

    for ascension in [18, 19, 20] {
        let mut live = combat(EncounterId::TimeEater, ascension, 56);
        live.state.enemies[0].hp = 239;
        set_intent(&mut live, 0, "time_ripple");
        let before = live.state.enemies[0].full_ai_rng.clone().unwrap();
        live.step_full(&CombatAction::EndTurn).unwrap();
        assert_eq!(live.state.enemies[0].intent_id.as_str(), "time_haste", "A{ascension}: live low-HP route selects Haste");
        // Design §6.7 / RNG-001: the forced Haste branch makes no AI draw (unchanged from base at every A).
        assert_eq!(live.state.enemies[0].full_ai_rng.as_ref().unwrap(), &before, "A{ascension}: forced Haste consumes no AI draw");
    }
}

#[test]
fn m32_spire_shield_a18_smash_block_99() {
    let mut outcomes = Vec::new();
    for (ascension, expected_block) in [(17, 38), (18, 99), (20, 99)] {
        let mut live = combat(EncounterId::ShieldAndSpear, ascension, 61);
        let shield = &mut live.state.enemies[0];
        shield.full_memory = Some(EnemyMemory::SpireShield { selections: 3 });
        shield.move_history = ["shield_bash", "shield_fortify", "shield_smash"].into_iter()
            .map(|name| ActorIntentId::Full(FullIntentId::from_name(name))).collect();
        set_intent(&mut live, 0, "shield_smash");
        let view = live.public_full().unwrap();
        assert_eq!(view.enemies[0].intent.as_str(), "shield_smash", "A{ascension}: public Smash intent");
        assert_eq!(view.enemies[0].intent_damage, Some(38), "A{ascension}: resolved Smash damage stays 38");
        let mut probe = live.clone();
        let owner = probe.state.enemies[0].instance_id.clone();
        let work = enemies::act4::move_work(&mut probe.state, &owner).unwrap();
        assert_eq!(work.len(), 2, "A{ascension}: Smash queue length");
        assert!(matches!(work[0].kind, EffectKind::Damage(DamagePayload::FullEnemyMove(intent)) if intent.as_str() == "shield_smash"), "Smash damage precedes its Block");
        assert!(matches!(work[1].kind, EffectKind::GainBlock(block) if block == expected_block), "A{ascension}: Smash grants fixed expected Block");
        live.step_full(&CombatAction::EndTurn).unwrap();
        assert_eq!(live.state.enemies[0].block, expected_block, "A{ascension}: Smash Block resolves");
        outcomes.push((view.enemies[0].intent_damage, live.state.enemies[0].block));
    }
    assert_eq!(outcomes[1], outcomes[2], "A20 Shield Smash matches A19");
}

#[test]
fn m32_spire_spear_a18_burns_top_of_draw() {
    let mut outcomes = Vec::new();
    for (ascension, expected_destination) in [
        (17, super::super::command::CardDestination::Discard),
        (18, super::super::command::CardDestination::Draw),
        (20, super::super::command::CardDestination::Draw),
    ] {
        let mut live = combat(EncounterId::ShieldAndSpear, ascension, 62);
        set_intent(&mut live, 1, "spear_burn_strike");
        let view = live.public_full().unwrap();
        assert_eq!(view.enemies[1].intent.as_str(), "spear_burn_strike", "A{ascension}: public Burn Strike intent");
        assert_eq!(view.enemies[1].intent_damage, Some(9), "A{ascension}: resolved Burn Strike damage after Spear Strength");
        let mut probe = live.clone();
        let owner = probe.state.enemies[1].instance_id.clone();
        let work = enemies::act4::move_work(&mut probe.state, &owner).unwrap();
        assert_eq!(work.len(), 4, "A{ascension}: Burn Strike queues two hits and two Burns");
        assert!(work[..2].iter().all(|effect| matches!(effect.kind, EffectKind::Damage(DamagePayload::FullEnemyMove(intent)) if intent.as_str() == "spear_burn_strike")), "Burn Strike hits precede card creation");
        for card_effect in &work[2..] {
            assert!(matches!(card_effect.kind, EffectKind::CreateCard { id: CardId::Burn, destination, upgrades: 0 } if destination == expected_destination),
                "A{ascension}: two unupgraded Burns use the source destination");
        }
        let card_rng_before = live.state.card_rng.clone();
        live.step_full(&CombatAction::EndTurn).unwrap();
        assert_eq!(live.state.card_rng, card_rng_before, "A{ascension}: Burn creation and next-turn draw consume no card RNG");
        let count = |cards: &[super::super::model::OwnedCard]| cards.iter().filter_map(|card| card.full())
            .filter(|card| card.id == CardId::Burn && card.upgrades == 0).count();
        let hand = count(&live.state.piles.hand);
        let draw = count(&live.state.piles.draw);
        let discard = count(&live.state.piles.discard);
        if expected_destination == super::super::command::CardDestination::Draw {
            assert_eq!((hand, draw, discard), (2, 0, 0), "A{ascension}: both top-inserted unupgraded Burns are drawn next turn");
        } else {
            assert_eq!((hand, draw, discard), (0, 0, 2), "A{ascension}: A17 Burns remain in discard");
        }
        outcomes.push((hand, draw, discard));
    }
    assert_eq!(outcomes[1], outcomes[2], "A20 Spear Burn Strike matches A19");
}
