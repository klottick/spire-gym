//! M32 Act 1 behaviour rows (plan items 2 and 5; design §2.3, §6.4, §6.8). Expected moves are
//! worked from the source rules recorded in `m32-source-exordium-v1.md`; roll-dependent cases read
//! their stream ticket with `rng::uniform_below`, never the selector under test.
use super::super::enemies::{self, EnemyMemory};
use super::super::rng::{self, Domain, StreamState};
use super::super::*;
use crate::content::EnemyId;

fn ai_stream(actor: &str) -> StreamState {
    let id = rng::StreamId::new(Domain::RunCombatEnemyAi, vec!["encounter-1".into(), actor.into()]).unwrap();
    rng::make_stream(&[32; 32], rng::Partition::Test, "m32-act1", &id).unwrap()
}

fn select(id: EnemyId, history: &[FullIntentId], stream: &StreamState, ascension: u8)
    -> Result<enemies::MoveSelection, enemies::SelectError> {
    enemies::select_move_at(id, history, &EnemyMemory::for_enemy(id).unwrap(), stream, ascension)
}

// ---- G1: the Louse opening is an ordinary roll (design §6.8; LouseNormal/LouseDefensive getMove) ----

/// Source: `num < 25` → Grow (Red) / Spit Web (Green), else Bite; an empty history meets no repeat
/// guard, so the opening is that roll at every ascension. Engine ticket: `uniform_below(stream, 4)`,
/// 0 ↔ the 25% branch.
#[test]
fn m32_g1_louse_opening_roll() {
    for (id, nonattack) in [(EnemyId::LouseRed, FullIntentId::Grow), (EnemyId::LouseGreen, FullIntentId::SpitWeb)] {
        for ascension in [0, 16, 17, 20] {
            let (mut saw_bite, mut saw_nonattack) = (false, false);
            for i in 0..64 {
                let stream = ai_stream(&format!("louse-{i}"));
                let (roll, next) = rng::uniform_below(&stream, 4).unwrap();
                let expected = if roll == 0 { nonattack } else { FullIntentId::Bite };
                let selection = select(id, &[], &stream, ascension).unwrap();
                assert_eq!(selection.intent, expected, "{id:?} A{ascension} stream {i}");
                assert_eq!(selection.stream, next, "{id:?} A{ascension} stream {i}: the opening draws one ticket");
                saw_bite |= expected == FullIntentId::Bite;
                saw_nonattack |= expected == nonattack;
            }
            assert!(saw_bite && saw_nonattack, "{id:?} A{ascension}: the sweep shows both openings");
        }
    }
    // Generated setup (both Louse kinds through `enemies::setup`) publishes non-Bite openings too.
    let mut nonattack_openings = 0;
    for seed in 0..32u8 {
        let mut run_rng = crate::run::RunRng::new([seed; 32], rng::Partition::Test, 11);
        let entry = enemies::setup(crate::content::EncounterId::ThreeLouse, 1, 0, &mut run_rng,
            super::m29_a2_owner::context(1)).unwrap();
        for louse in &entry.setup.enemies {
            let nonattack = if louse.enemy_id == EnemyId::LouseRed { FullIntentId::Grow } else { FullIntentId::SpitWeb };
            assert!(louse.intent == FullIntentId::Bite || louse.intent == nonattack, "{:?}", louse.intent);
            nonattack_openings += usize::from(louse.intent == nonattack);
        }
    }
    assert!(nonattack_openings > 0, "generated Louse setups never opened with Grow / Spit Web");
}

/// A history that opens with the non-attack move is a legal source history (G1); the selector accepts it.
#[test]
fn m32_g1_louse_nonattack_opening_history_accepted() {
    let stream = ai_stream("louse-history");
    for (id, nonattack) in [(EnemyId::LouseRed, FullIntentId::Grow), (EnemyId::LouseGreen, FullIntentId::SpitWeb)] {
        for history in [vec![nonattack], vec![nonattack, FullIntentId::Bite]] {
            for ascension in [0, 17] {
                assert!(select(id, &history, &stream, ascension).is_ok(), "{id:?} {history:?} A{ascension}");
            }
        }
    }
}

fn stream_ticket(actor: &str, bound: u128, expected: u64) -> StreamState {
    for index in 0..20_000 {
        let stream = ai_stream(&format!("{actor}-{index}"));
        if rng::uniform_below(&stream, bound).unwrap().0 == expected {
            return stream;
        }
    }
    panic!("no {bound}-ticket stream found for {actor}={expected}");
}

fn stream_two_tickets(actor: &str, first_bound: u128, first: u64,
    second_bound: u128, second_min: u64) -> (StreamState, u64, u64) {
    for index in 0..100_000 {
        let stream = ai_stream(&format!("{actor}-{index}"));
        let (first_ticket, cursor) = rng::uniform_below(&stream, first_bound).unwrap();
        let (second_ticket, _) = rng::uniform_below(&cursor, second_bound).unwrap();
        if first_ticket == first && second_ticket >= second_min {
            return (stream, first_ticket, second_ticket);
        }
    }
    panic!("no two-ticket stream found for {actor}");
}

fn single_enemy_combat(encounter: crate::content::EncounterId, ascension: u8,
    id: EnemyId) -> (Combat, usize) {
    for seed in 0..128u8 {
        let mut entry = super::m29_a2_owner::setup(encounter, ascension, seed, 0).unwrap();
        if entry.setup.enemies.iter().any(|enemy| enemy.enemy_id == id) {
            if encounter == crate::content::EncounterId::ThreeLouse {
                let slot = entry.setup.enemies.iter().position(|enemy| enemy.enemy_id == id).unwrap();
                let combat = Combat::enter(entry).unwrap();
                return (combat, slot);
            }
            entry.setup.enemies.retain(|enemy| enemy.enemy_id == id);
            let combat = Combat::enter(entry).unwrap();
            return (combat, 0);
        }
    }
    panic!("{id:?} was absent from {encounter:?} in 128 setups at A{ascension}");
}

fn install_history(combat: &mut Combat, slot: usize, history: &[FullIntentId]) {
    let selected = ActorIntentId::Full(*history.last().expect("nonempty history"));
    let enemy = &mut combat.state.enemies[slot];
    enemy.intent_id = selected.clone();
    enemy.selected_move = selected;
    enemy.move_history = history.iter().copied().map(ActorIntentId::Full).collect();
    effects::refresh_enemy_values(&mut combat.state).unwrap();
}

fn setup_initial_stream(combat: &Combat, slot: usize) -> StreamState {
    let cursor = combat.state.enemies[slot].full_ai_rng.as_ref().expect("setup owns enemy AI stream");
    StreamState::new(cursor.id().clone(), *cursor.key(), 0).unwrap()
}

fn power_amount_on_player(combat: &Combat, id: &str) -> i32 {
    combat.state.player_full_powers.iter().find(|power| power.id.0 == id)
        .map_or(0, |power| power.state.amount)
}

// ---- Acid Slime S: the A17 force applies only to the empty-history opening ----

#[test]
fn m32_acid_slime_s_a17_opening_lick() {
    let stream = stream_ticket("acid-s-opening", 2, 0);
    let (ticket, after_ticket) = rng::uniform_below(&stream, 2).unwrap();
    assert_eq!(ticket, 0, "A16 opening coin ticket selects Tackle");
    let a16 = select(EnemyId::AcidSlimeS, &[], &stream, 16).unwrap();
    assert_eq!(a16.intent, FullIntentId::AcidTackle);
    assert_eq!(a16.stream, after_ticket, "A16 opening consumes its coin");
    let a17 = select(EnemyId::AcidSlimeS, &[], &stream, 17).unwrap();
    assert_eq!(a17.intent, FullIntentId::AcidLick, "A17 empty history is forced Lick");
    assert_eq!(a17.stream, stream, "forced A17 opening makes no coin draw");

    for ascension in [16, 17] {
        let (mut combat, slot) = single_enemy_combat(crate::content::EncounterId::LotsOfSlimes,
            ascension, EnemyId::AcidSlimeS);
        let setup_stream = setup_initial_stream(&combat, slot);
        let (setup_ticket, setup_after) = rng::uniform_below(&setup_stream, 2).unwrap();
        let setup_expected = if ascension >= 17 || setup_ticket == 1 {
            FullIntentId::AcidLick
        } else { FullIntentId::AcidTackle };
        assert_eq!(combat.state.enemies[slot].intent_id, ActorIntentId::Full(setup_expected),
            "A{ascension} setup opening follows the source law");
        assert_eq!(combat.state.enemies[slot].full_ai_rng.as_ref().unwrap().word_index(),
            if ascension >= 17 { 0 } else { setup_after.word_index() },
            "A{ascension} setup opening draw count");
        if ascension >= 17 {
            assert_eq!(combat.state.enemies[slot].intent_id.as_str(), "acid_lick",
                "setup opening at A{ascension} is the forced source Lick");
            assert_eq!(combat.state.enemies[slot].full_ai_rng.as_ref().unwrap().word_index(), 0,
                "forced opening consumes no selector draw");
        }
        let current = FullIntentId::AcidTackle;
        install_history(&mut combat, slot, &[current]);
        combat.state.enemies[slot].full_ai_rng = Some(stream.clone());
        combat.step_full(&CombatAction::EndTurn).unwrap();
        assert!(combat.state.fault.is_none(), "A{ascension}: {:?}", combat.state.fault);
        assert_eq!(combat.public_full().unwrap().enemies[slot].intent.as_str(), "acid_lick",
            "later turns alternate at A{ascension}");
    }
}

// ---- Acid Slime M: A17 changes its bands and repeat guards ----

#[test]
fn m32_acid_slime_m_a17_bands_and_guards() {
    const HALF: u64 = 1 << 23;
    const FORTY_PERCENT: u64 = 6_710_887;
    let (stream, base_ticket, redirect_ticket) = stream_two_tickets(
        "acid-m-a17", 100, 35, 1 << 24, FORTY_PERCENT);
    assert_eq!(base_ticket, 35, "ticket 35 lies in A16's middle and A17's first band");
    assert!(redirect_ticket >= FORTY_PERCENT, "ticket {redirect_ticket} selects the non-redirect");
    assert!(redirect_ticket >= HALF, "the same ticket is also false for A17's 50% redirect");
    let history = [FullIntentId::AcidTackle, FullIntentId::AcidTackle];
    let (_, after_base) = rng::uniform_below(&stream, 100).unwrap();
    let (_, after_redirect) = rng::uniform_below(&after_base, 1 << 24).unwrap();

    for (ascension, expected, expected_stream) in [
        (16, FullIntentId::AcidLick, &after_redirect),
        (17, FullIntentId::AcidSpit, &after_base),
        (20, FullIntentId::AcidSpit, &after_base),
    ] {
        let selected = select(EnemyId::AcidSlimeM, &history, &stream, ascension).unwrap();
        assert_eq!(selected.intent, expected, "A{ascension}");
        assert_eq!(&selected.stream, expected_stream, "A{ascension} selector draw count");

        let (mut combat, slot) = single_enemy_combat(crate::content::EncounterId::SmallSlimes,
            ascension, EnemyId::AcidSlimeM);
        let setup_stream = setup_initial_stream(&combat, slot);
        let setup_ticket = rng::uniform_below(&setup_stream, 100).unwrap().0;
        let setup_expected = if setup_ticket < if ascension >= 17 { 40 } else { 30 } {
            FullIntentId::AcidSpit
        } else if setup_ticket < if ascension >= 17 { 80 } else { 70 } {
            FullIntentId::AcidTackle
        } else { FullIntentId::AcidLick };
        assert_eq!(combat.state.enemies[slot].intent_id, ActorIntentId::Full(setup_expected),
            "A{ascension} setup opening ticket {setup_ticket}");
        install_history(&mut combat, slot, &history);
        combat.state.enemies[slot].full_ai_rng = Some(stream.clone());
        combat.step_full(&CombatAction::EndTurn).unwrap();
        assert!(combat.state.fault.is_none(), "A{ascension}: {:?}", combat.state.fault);
        let public = &combat.public_full().unwrap().enemies[slot];
        assert_eq!(public.intent, expected, "A{ascension} live roll");
        assert_eq!(public.intent_damage, (expected == FullIntentId::AcidSpit).then_some(8),
            "A{ascension} public resolved intent damage");
        assert_eq!(combat.state.enemies[slot].full_ai_rng.as_ref().unwrap(), expected_stream,
            "A{ascension} live route consumes the selector's source-law tickets");
    }
}

// ---- Acid Slime L: the large selector's bands and all three history tests change at A17 ----

#[test]
fn m32_acid_slime_l_a17_bands_and_guards() {
    const FORTY_PERCENT: u64 = 6_710_887;
    let (stream, base_ticket, redirect_ticket) = stream_two_tickets(
        "acid-l-a17", 100, 35, 1 << 24, FORTY_PERCENT);
    assert_eq!(base_ticket, 35, "ticket 35 lies in A16's middle and A17's first band");
    assert!(redirect_ticket >= FORTY_PERCENT, "ticket {redirect_ticket} selects the non-redirect");
    let history = [FullIntentId::AcidTackle, FullIntentId::AcidTackle];
    let (_, after_base) = rng::uniform_below(&stream, 100).unwrap();
    let (_, after_redirect) = rng::uniform_below(&after_base, 1 << 24).unwrap();

    for (ascension, expected, expected_stream) in [
        (16, FullIntentId::AcidLick, &after_redirect),
        (17, FullIntentId::AcidSpit, &after_base),
        (20, FullIntentId::AcidSpit, &after_base),
    ] {
        let selected = select(EnemyId::AcidSlimeL, &history, &stream, ascension).unwrap();
        assert_eq!(selected.intent, expected, "A{ascension}");
        assert_eq!(&selected.stream, expected_stream, "A{ascension} selector draw count");

        let (mut combat, slot) = single_enemy_combat(crate::content::EncounterId::LargeSlime,
            ascension, EnemyId::AcidSlimeL);
        let setup_stream = setup_initial_stream(&combat, slot);
        let setup_ticket = rng::uniform_below(&setup_stream, 100).unwrap().0;
        let setup_expected = if setup_ticket < if ascension >= 17 { 40 } else { 30 } {
            FullIntentId::AcidSpit
        } else if setup_ticket < 70 {
            FullIntentId::AcidTackle
        } else { FullIntentId::AcidLick };
        assert_eq!(combat.state.enemies[slot].intent_id, ActorIntentId::Full(setup_expected),
            "A{ascension} setup opening ticket {setup_ticket}");
        install_history(&mut combat, slot, &history);
        combat.state.enemies[slot].full_ai_rng = Some(stream.clone());
        combat.step_full(&CombatAction::EndTurn).unwrap();
        assert!(combat.state.fault.is_none(), "A{ascension}: {:?}", combat.state.fault);
        let public = &combat.public_full().unwrap().enemies[slot];
        assert_eq!(public.intent, expected, "A{ascension} live roll");
        assert_eq!(public.intent_damage, (expected == FullIntentId::AcidSpit).then_some(12),
            "A{ascension} public resolved intent damage");
        assert_eq!(combat.state.enemies[slot].full_ai_rng.as_ref().unwrap(), expected_stream,
            "A{ascension} live route consumes the selector's source-law tickets");
    }
}

// ---- Spike Slime M: one preceding Lick is sufficient to force Flame Tackle at A17 ----

#[test]
fn m32_spike_slime_m_a17_single_lick_guard() {
    let stream = stream_ticket("spike-m-a17", 100, 40);
    let (ticket, after_ticket) = rng::uniform_below(&stream, 100).unwrap();
    assert_eq!(ticket, 40, "ticket 40 is in the high band");
    let history = [FullIntentId::SpikeLick];
    for (ascension, expected, expected_stream) in [
        (16, FullIntentId::SpikeLick, &after_ticket),
        (17, FullIntentId::SpikeFlameTackle, &stream),
        (20, FullIntentId::SpikeFlameTackle, &stream),
    ] {
        let selected = select(EnemyId::SpikeSlimeM, &history, &stream, ascension).unwrap();
        assert_eq!(selected.intent, expected, "A{ascension}");
        assert_eq!(&selected.stream, expected_stream, "A{ascension} forced-branch draw count");

        let (mut combat, slot) = single_enemy_combat(crate::content::EncounterId::SmallSlimes,
            ascension, EnemyId::SpikeSlimeM);
        install_history(&mut combat, slot, &history);
        combat.state.enemies[slot].full_ai_rng = Some(stream.clone());
        combat.step_full(&CombatAction::EndTurn).unwrap();
        assert!(combat.state.fault.is_none(), "A{ascension}: {:?}", combat.state.fault);
        let public = &combat.public_full().unwrap().enemies[slot];
        assert_eq!(public.intent, expected, "A{ascension} live roll");
        assert_eq!(public.intent_damage,
            (expected == FullIntentId::SpikeFlameTackle).then_some(10),
            "A{ascension} public resolved intent damage");
        assert_eq!(combat.state.enemies[slot].full_ai_rng.as_ref().unwrap(), expected_stream,
            "A{ascension} live route stream");
    }
}

// ---- Spike Slime L: large Lick retains Frail payload but has the same A17 guard ----

#[test]
fn m32_spike_slime_l_a17_single_lick_guard() {
    let stream = stream_ticket("spike-l-a17", 100, 30);
    let (ticket, after_ticket) = rng::uniform_below(&stream, 100).unwrap();
    assert_eq!(ticket, 30, "ticket 30 is in the high band");
    let history = [FullIntentId::SpikeLick];
    for (ascension, expected, expected_stream) in [
        (16, FullIntentId::SpikeLick, &after_ticket),
        (17, FullIntentId::SpikeFlameTackle, &stream),
        (20, FullIntentId::SpikeFlameTackle, &stream),
    ] {
        let selected = select(EnemyId::SpikeSlimeL, &history, &stream, ascension).unwrap();
        assert_eq!(selected.intent, expected, "A{ascension}");
        assert_eq!(&selected.stream, expected_stream, "A{ascension} forced-branch draw count");

        let (mut combat, slot) = single_enemy_combat(crate::content::EncounterId::LargeSlime,
            ascension, EnemyId::SpikeSlimeL);
        install_history(&mut combat, slot, &history);
        combat.state.enemies[slot].full_ai_rng = Some(stream.clone());
        combat.step_full(&CombatAction::EndTurn).unwrap();
        assert!(combat.state.fault.is_none(), "A{ascension}: {:?}", combat.state.fault);
        let public = &combat.public_full().unwrap().enemies[slot];
        assert_eq!(public.intent, expected, "A{ascension} live roll");
        assert_eq!(public.intent_damage,
            (expected == FullIntentId::SpikeFlameTackle).then_some(18),
            "A{ascension} public resolved intent damage");
        assert_eq!(combat.state.enemies[slot].full_ai_rng.as_ref().unwrap(), expected_stream,
            "A{ascension} live route stream");
    }
}

// ---- LouseNormal: one preceding Grow is barred at A17 ----

#[test]
fn m32_louse_red_a17_single_grow_guard() {
    let stream = stream_ticket("red-louse-a17", 4, 0);
    let (ticket, after_ticket) = rng::uniform_below(&stream, 4).unwrap();
    assert_eq!(ticket, 0, "ticket zero selects the low-band non-attack without the guard");
    let history = [FullIntentId::Grow];
    for (ascension, expected, expected_stream) in [
        (16, FullIntentId::Grow, &after_ticket),
        (17, FullIntentId::Bite, &after_ticket),
        (20, FullIntentId::Bite, &after_ticket),
    ] {
        let selected = select(EnemyId::LouseRed, &history, &stream, ascension).unwrap();
        assert_eq!(selected.intent, expected, "A{ascension}");
        assert_eq!(&selected.stream, expected_stream, "A{ascension}: ordinary opening-band ticket only");

        let (mut combat, slot) = single_enemy_combat(crate::content::EncounterId::ThreeLouse,
            ascension, EnemyId::LouseRed);
        install_history(&mut combat, slot, &history);
        combat.state.enemies[slot].full_ai_rng = Some(stream.clone());
        let bite_damage = combat.state.enemies[slot].full_damage_entries[&FullIntentId::Bite].output;
        combat.step_full(&CombatAction::EndTurn).unwrap();
        assert!(combat.state.fault.is_none(), "A{ascension}: {:?}", combat.state.fault);
        let public = &combat.public_full().unwrap().enemies[slot];
        assert_eq!(public.intent, expected, "A{ascension} live roll");
        let grow_strength = if ascension >= 17 { 4 } else { 3 };
        assert_eq!(public.intent_damage, (expected == FullIntentId::Bite).then_some(bite_damage + grow_strength),
            "A{ascension} public resolved intent damage");
    }
}

// ---- LouseDefensive: one preceding Spit Web is barred at A17 ----

#[test]
fn m32_louse_green_a17_single_spit_web_guard() {
    let stream = stream_ticket("green-louse-a17", 4, 0);
    let (ticket, after_ticket) = rng::uniform_below(&stream, 4).unwrap();
    assert_eq!(ticket, 0, "ticket zero selects the low-band non-attack without the guard");
    let history = [FullIntentId::SpitWeb];
    for (ascension, expected, expected_stream) in [
        (16, FullIntentId::SpitWeb, &after_ticket),
        (17, FullIntentId::Bite, &after_ticket),
        (20, FullIntentId::Bite, &after_ticket),
    ] {
        let selected = select(EnemyId::LouseGreen, &history, &stream, ascension).unwrap();
        assert_eq!(selected.intent, expected, "A{ascension}");
        assert_eq!(&selected.stream, expected_stream, "A{ascension}: ordinary opening-band ticket only");

        let (mut combat, slot) = single_enemy_combat(crate::content::EncounterId::ThreeLouse,
            ascension, EnemyId::LouseGreen);
        install_history(&mut combat, slot, &history);
        combat.state.enemies[slot].full_ai_rng = Some(stream.clone());
        let bite_damage = combat.state.enemies[slot].full_damage_entries[&FullIntentId::Bite].output;
        combat.step_full(&CombatAction::EndTurn).unwrap();
        assert!(combat.state.fault.is_none(), "A{ascension}: {:?}", combat.state.fault);
        let public = &combat.public_full().unwrap().enemies[slot];
        assert_eq!(public.intent, expected, "A{ascension} live roll");
        assert_eq!(public.intent_damage, (expected == FullIntentId::Bite).then_some(bite_damage),
            "A{ascension} public resolved intent damage");
    }
}

fn select_with_memory(id: EnemyId, history: &[FullIntentId], memory: &EnemyMemory,
    stream: &StreamState, ascension: u8) -> Result<enemies::MoveSelection, enemies::SelectError> {
    enemies::select_move_at(id, history, memory, stream, ascension)
}

// ---- Gremlin Wizard: a Blast history repeats Blast without changing charge memory ----

#[test]
fn m32_gremlin_wizard_a17_blast_repeats() {
    let stream = ai_stream("wizard-after-blast");
    let history = [FullIntentId::from_name("gremlin_blast")];
    let memory = EnemyMemory::GremlinWizard { charge: 0 };
    for (ascension, expected) in [
        (16, "gremlin_charge"),
        (17, "gremlin_blast"),
        (20, "gremlin_blast"),
    ] {
        let selected = select_with_memory(EnemyId::GremlinWizard, &history, &memory, &stream, ascension).unwrap();
        assert_eq!(selected.intent.as_str(), expected, "A{ascension}");
        assert_eq!(selected.memory, memory, "Blast leaves charge memory at zero");
        assert_eq!(selected.stream, stream, "fixed history transition draws no selector ticket");

        let (mut combat, slot) = single_enemy_combat(crate::content::EncounterId::GremlinGang,
            ascension, EnemyId::GremlinWizard);
        install_history(&mut combat, slot, &history);
        combat.state.enemies[slot].full_memory = Some(memory.clone());
        combat.state.enemies[slot].full_ai_rng = Some(stream.clone());
        combat.step_full(&CombatAction::EndTurn).unwrap();
        assert!(combat.state.fault.is_none(), "A{ascension}: {:?}", combat.state.fault);
        let public = &combat.public_full().unwrap().enemies[slot];
        assert_eq!(public.intent.as_str(), expected, "A{ascension} Gremlin Gang route");
        assert_eq!(public.intent_damage, (expected == "gremlin_blast").then_some(30),
            "A{ascension} resolved Blast damage");
        assert_eq!(combat.state.enemies[slot].full_memory,
            Some(EnemyMemory::GremlinWizard { charge: 0 }), "Blast resets charge memory to zero");
    }

    // Rally selects a Wizard through the Gremlin Leader's real spawn route at A18.
    let mut entry = super::m29_a2_owner::setup(crate::content::EncounterId::GremlinLeader, 18, 4, 0).unwrap();
    entry.setup.enemies.retain(|enemy| enemy.enemy_id == EnemyId::GremlinLeader);
    let mut leader_combat = Combat::enter(entry).unwrap();
    install_history(&mut leader_combat, 0, &[FullIntentId::from_name("leader_rally")]);
    leader_combat.state.enemies[0].full_ai_rng = Some(stream_ticket("leader-rally-wizard", 8, 7));
    leader_combat.step_full(&CombatAction::EndTurn).unwrap();
    let wizard_slot = leader_combat.state.enemies.iter().position(|enemy|
        enemy.enemy_id == ActorEnemyId::Full(EnemyId::GremlinWizard))
        .expect("A18 Rally's first pool ticket creates a Wizard minion");
    assert!(leader_combat.state.enemies[wizard_slot].full_powers.iter().any(|power| power.id.0 == "minion"),
        "the Wizard arrived through the Gremlin Leader minion route");
    install_history(&mut leader_combat, wizard_slot, &history);
    leader_combat.state.enemies[wizard_slot].full_memory = Some(memory);
    leader_combat.step_full(&CombatAction::EndTurn).unwrap();
    assert!(leader_combat.state.fault.is_none(), "A18 minion: {:?}", leader_combat.state.fault);
    assert_eq!(leader_combat.state.enemies[wizard_slot].intent_id.as_str(), "gremlin_blast",
        "A18 spawned Wizard repeats Blast from its move history");
    assert_eq!(leader_combat.state.enemies[wizard_slot].full_memory,
        Some(EnemyMemory::GremlinWizard { charge: 0 }));
}

// ---- Gremlin Fat: Smash adds Frail after Weak at A17+ ----

#[test]
fn m32_gremlin_fat_a17_smash_frail() {
    let current = FullIntentId::from_name("gremlin_smash");
    for (ascension, expected_powers) in [
        (16, vec![("weak", 1)]),
        (17, vec![("weak", 1), ("frail", 1)]),
        (20, vec![("weak", 1), ("frail", 1)]),
    ] {
        let (mut combat, slot) = single_enemy_combat(crate::content::EncounterId::GremlinGang,
            ascension, EnemyId::GremlinFat);
        install_history(&mut combat, slot, &[current]);
        let owner = combat.state.enemies[slot].instance_id.clone();
        let queued = enemies::act2::take_turn(&mut combat.state, &owner).unwrap();
        let queued_powers = queued.iter().filter_map(|effect| match &effect.kind {
            EffectKind::ApplyFullPower { id, amount } => Some((id.0, *amount)),
            _ => None,
        }).collect::<Vec<_>>();
        assert_eq!(queued_powers, expected_powers, "A{ascension} queue order after Smash damage");

        combat.step_full(&CombatAction::EndTurn).unwrap();
        assert!(combat.state.fault.is_none(), "A{ascension}: {:?}", combat.state.fault);
        assert_eq!(power_amount_on_player(&combat, "weak"), 1, "A{ascension}");
        assert_eq!(power_amount_on_player(&combat, "frail"), i32::from(ascension >= 17), "A{ascension}");
    }
}

// ---- Taskmaster: A18 adds Strength after the three Wounds ----

#[test]
fn m32_taskmaster_a18_whip_strength() {
    let whip = FullIntentId::from_name("taskmaster_whip");
    for (ascension, wounds, strength) in [(17, 2, 0), (18, 3, 1), (20, 3, 1)] {
        let (mut combat, slot) = single_enemy_combat(crate::content::EncounterId::Slavers,
            ascension, EnemyId::Taskmaster);
        install_history(&mut combat, slot, &[whip]);
        assert_eq!(combat.public_full().unwrap().enemies[slot].intent.as_str(), "taskmaster_whip");
        assert_eq!(combat.public_full().unwrap().enemies[slot].intent_damage, Some(7), "resolved Whip damage");

        let owner = combat.state.enemies[slot].instance_id.clone();
        let queued = enemies::act2::take_turn(&mut combat.state, &owner).unwrap();
        let wound_positions = queued.iter().enumerate().filter_map(|(index, effect)|
            matches!(&effect.kind, EffectKind::CreateCard { id, .. } if *id == crate::content::CardId::Wound)
                .then_some(index)).collect::<Vec<_>>();
        assert_eq!(wound_positions.len(), wounds as usize, "A{ascension} Wounds");
        let strength_position = queued.iter().position(|effect|
            matches!(&effect.kind, EffectKind::ApplyFullPower { id, amount: 1 } if id.0 == "strength"));
        assert_eq!(strength_position.is_some(), ascension >= 18, "A{ascension} Strength presence");
        if let Some(position) = strength_position {
            assert!(position > *wound_positions.last().unwrap(), "Strength is queued after every Wound");
        }

        combat.step_full(&CombatAction::EndTurn).unwrap();
        assert!(combat.state.fault.is_none(), "A{ascension}: {:?}", combat.state.fault);
        let public = combat.public_full().unwrap();
        assert_eq!(public.discard.iter().filter(|card| card.id == crate::content::CardId::Wound).count(),
            wounds as usize, "A{ascension} Wounds reach discard");
        let taskmaster = &combat.state.enemies[slot];
        assert_eq!(taskmaster.full_powers.iter().find(|power| power.id.0 == "strength")
            .map_or(0, |power| power.state.amount), strength, "A{ascension} self Strength");
    }
}

// ---- Slaver Red: one preceding Scrape is barred by the final guard at A17 ----

#[test]
fn m32_slaver_red_a17_single_scrape_guard() {
    let stream = stream_ticket("red-slaver-a17", 100, 20);
    let (ticket, after_ticket) = rng::uniform_below(&stream, 100).unwrap();
    assert_eq!(ticket, 20, "ticket 20 reaches the final fallback");
    let history = [FullIntentId::from_name("slaver_stab"), FullIntentId::from_name("slaver_scrape")];
    let memory = EnemyMemory::SlaverRed { used_entangle: true };
    for (ascension, expected) in [
        (16, FullIntentId::from_name("slaver_scrape")),
        (17, FullIntentId::from_name("slaver_stab")),
        (20, FullIntentId::from_name("slaver_stab")),
    ] {
        let selected = select_with_memory(EnemyId::SlaverRed, &history, &memory, &stream, ascension).unwrap();
        assert_eq!(selected.intent, expected, "A{ascension}");
        assert_eq!(selected.stream, after_ticket, "A{ascension} uses one base ticket");

        let (mut combat, slot) = single_enemy_combat(crate::content::EncounterId::RedSlaver,
            ascension, EnemyId::SlaverRed);
        install_history(&mut combat, slot, &history);
        combat.state.enemies[slot].full_memory = Some(memory.clone());
        combat.state.enemies[slot].full_ai_rng = Some(stream.clone());
        combat.step_full(&CombatAction::EndTurn).unwrap();
        assert!(combat.state.fault.is_none(), "A{ascension}: {:?}", combat.state.fault);
        assert_eq!(power_amount_on_player(&combat, "vulnerable"), if ascension >= 17 { 2 } else { 1 },
            "Scrape payload applies before the next selection");
        let public = &combat.public_full().unwrap().enemies[slot];
        assert_eq!(public.intent, expected, "A{ascension} live roll");
        assert_eq!(public.intent_damage,
            Some((if expected.as_str() == "slaver_scrape" { 9 } else { 14 }) * 3 / 2),
            "A{ascension} public resolved intent damage after Scrape's Vulnerable");
        assert_eq!(combat.state.enemies[slot].full_ai_rng.as_ref().unwrap(), &after_ticket,
            "A{ascension} live selector stream");
    }
}

fn split_parent_for_acid_child_tickets(root: &EnemyHandle) -> (StreamState, StreamState, StreamState) {
    let acid_large = EnemyHandle::parse(&format!("{}/split/1", root.as_str())).unwrap();
    let acid_medium = EnemyHandle::parse(&format!("{}/split/1/split/0", root.as_str())).unwrap();
    for index in 0..100_000 {
        let parent = ai_stream(&format!("slime-boss-split-{index}"));
        let acid_large_input = enemies::split_child_stream(&parent, &acid_large, 1).unwrap();
        let (acid_large_ticket, acid_large_after) = rng::uniform_below(&acid_large_input, 100).unwrap();
        if acid_large_ticket != 35 { continue; }
        let acid_medium_input = enemies::split_child_stream(&acid_large_after, &acid_medium, 0).unwrap();
        let (acid_medium_ticket, acid_medium_after) = rng::uniform_below(&acid_medium_input, 100).unwrap();
        if acid_medium_ticket == 35 {
            return (parent, acid_large_after, acid_medium_after);
        }
    }
    panic!("could not derive A17-distinguishing Split child tickets");
}

// ---- Slime Boss → Large → Medium split children use the inherited A17 AI rules ----

#[test]
fn m32_split_children_a17_ai() {
    let (mut combat, boss_slot) = single_enemy_combat(crate::content::EncounterId::SlimeBoss,
        17, EnemyId::SlimeBoss);
    let boss_handle = combat.state.enemies[boss_slot].instance_id.clone();
    let (parent_stream, acid_large_after, acid_medium_after) = split_parent_for_acid_child_tickets(&boss_handle);
    combat.state.enemies[boss_slot].full_ai_rng = Some(parent_stream);
    combat.state.enemies[boss_slot].full_memory = Some(EnemyMemory::SlimeBoss { split_triggered: true });
    combat.state.enemies[boss_slot].hp = combat.state.enemies[boss_slot].max_hp / 2;
    install_history(&mut combat, boss_slot, &[FullIntentId::SlimeBossSplit]);
    combat.step_full(&CombatAction::EndTurn).unwrap();
    assert!(combat.state.fault.is_none(), "Slime Boss split: {:?}", combat.state.fault);

    let acid_large_handle = EnemyHandle::parse(&format!("{}/split/1", boss_handle.as_str())).unwrap();
    let acid_large_slot = combat.state.enemies.iter().position(|enemy|
        enemy.instance_id == acid_large_handle).expect("Slime Boss creates its Acid Slime L child");
    assert_eq!(combat.state.enemies[acid_large_slot].enemy_id, ActorEnemyId::Full(EnemyId::AcidSlimeL));
    assert_eq!(combat.state.enemies[acid_large_slot].intent_id.as_str(), "acid_spit",
        "A17 Acid Slime L opening uses ticket 35 in its <40 band");
    assert_eq!(combat.state.enemies[acid_large_slot].full_ai_rng.as_ref(), Some(&acid_large_after));

    combat.state.enemies[acid_large_slot].full_memory = Some(EnemyMemory::AcidSlime { split_triggered: true });
    combat.state.enemies[acid_large_slot].hp = combat.state.enemies[acid_large_slot].max_hp / 2;
    install_history(&mut combat, acid_large_slot, &[FullIntentId::AcidSplit]);
    combat.step_full(&CombatAction::EndTurn).unwrap();
    assert!(combat.state.fault.is_none(), "Acid Slime L split: {:?}", combat.state.fault);

    let acid_medium_handle = EnemyHandle::parse(&format!("{}/split/1/split/0", boss_handle.as_str())).unwrap();
    let acid_medium = combat.state.enemies.iter().find(|enemy| enemy.instance_id == acid_medium_handle)
        .expect("Acid Slime L creates its Acid Slime M child");
    assert_eq!(acid_medium.enemy_id, ActorEnemyId::Full(EnemyId::AcidSlimeM));
    assert_eq!(acid_medium.intent_id.as_str(), "acid_spit",
        "A17 Acid Slime M opening uses ticket 35 in its <40 band");
    assert_eq!(acid_medium.full_ai_rng.as_ref(), Some(&acid_medium_after));
    engine::validate_runtime(&combat.state).unwrap();
}

/// Source SlaverRed.getMove: the final "not Scrape" fallback (A17: last move not Scrape; A0: not two Scrapes) also
/// applies while Entangle is unused, when num < 75 (lane B SlaverRed §1–§2). Found by the c1 traversal gate.
#[test]
fn m32_slaver_red_a17_scrape_guard_before_entangle() {
    let stream = stream_ticket("red-slaver-a17-unused-entangle", 100, 20);
    let (ticket, after_ticket) = rng::uniform_below(&stream, 100).unwrap();
    assert_eq!(ticket, 20, "ticket 20 is below the Entangle cutoff 75");
    let history = [FullIntentId::from_name("slaver_stab"), FullIntentId::from_name("slaver_scrape")];
    let memory = EnemyMemory::SlaverRed { used_entangle: false };
    for (ascension, expected) in [
        (16, FullIntentId::from_name("slaver_scrape")),
        (17, FullIntentId::from_name("slaver_stab")),
        (20, FullIntentId::from_name("slaver_stab")),
    ] {
        let selected = select_with_memory(EnemyId::SlaverRed, &history, &memory, &stream, ascension).unwrap();
        assert_eq!(selected.intent, expected, "A{ascension}");
        assert_eq!(selected.stream, after_ticket, "A{ascension} uses one base ticket");
    }
}

// ---- Acid Slime M/L: each A17 guard and redirect sub-rule (final review I1; AcidSlime_M/L.getMove, lane B §1–§2) ----
// Mapping (source): band 1 redirect true → Tackle, false → Lick; band 2 true → Spit, false → Lick; band 3 true → Spit,
// false → Tackle. 0.4f = 6_710_887 / 2^24, 0.6f = 10_066_330 / 2^24 (engine form of randomBoolean(p)); 0.5 is a coin.

const ACID_P24: u128 = 1 << 24;
const ACID_P40: u64 = 6_710_887;
const ACID_P50: u64 = 1 << 23;
const ACID_P60: u64 = 10_066_330;

fn acid_find_stream(actor: &str, wanted: impl Fn(u64, u64, u64) -> bool) -> StreamState {
    // wanted(base ticket of 100, second ticket of 2^24, second ticket of 2)
    (0..200_000).map(|index| ai_stream(&format!("{actor}-{index}"))).find(|stream| {
        let (base, cursor) = rng::uniform_below(stream, 100).unwrap();
        wanted(base, rng::uniform_below(&cursor, ACID_P24).unwrap().0, rng::uniform_below(&cursor, 2).unwrap().0)
    }).unwrap_or_else(|| panic!("no Acid stream found for {actor}"))
}

fn acid_after(stream: &StreamState, second_bound: Option<u128>) -> StreamState {
    let (_, cursor) = rng::uniform_below(stream, 100).unwrap();
    second_bound.map_or(cursor.clone(), |bound| rng::uniform_below(&cursor, bound).unwrap().1)
}

#[test]
fn m32_acid_slime_a17_band2_two_tackle_guard() {
    use FullIntentId::{AcidLick, AcidSpit, AcidTackle};
    // Band 2 (A16: 30–69; A17: 40–79 M, 40–69 L). A16 redirects after the LAST Tackle (0.4); A17 only after TWO.
    let stream = acid_find_stream("acid-band2-one-tackle", |base, _, _| base == 50);
    let (a16_ticket, _) = rng::uniform_below(&acid_after(&stream, None), ACID_P24).unwrap();
    let a16_expected = if a16_ticket < ACID_P40 { AcidSpit } else { AcidLick };
    for id in [EnemyId::AcidSlimeM, EnemyId::AcidSlimeL] {
        let selected = select(id, &[AcidTackle], &stream, 16).unwrap();
        assert_eq!((selected.intent, selected.stream), (a16_expected, acid_after(&stream, Some(ACID_P24))),
            "{id:?} A16: one Tackle redirects (ticket {a16_ticket})");
        for ascension in [17, 20] {
            let selected = select(id, &[AcidTackle], &stream, ascension).unwrap();
            assert_eq!((selected.intent, selected.stream), (AcidTackle, acid_after(&stream, None)),
                "{id:?} A{ascension}: one Tackle is not the two-Tackle guard; no redirect draw");
        }
    }
}

#[test]
fn m32_acid_slime_a17_band2_redirect_probabilities() {
    use FullIntentId::{AcidLick, AcidSpit, AcidTackle};
    let history = [AcidTackle, AcidTackle];
    // M: A16 0.4 (2^24 ticket ≥ 0.4 → Lick); A17 coin 0.5 (2-ticket 0 → Spit).
    let m_spit = acid_find_stream("acid-m-b2-coin-0", |base, p24, coin| base == 50 && p24 >= ACID_P40 && coin == 0);
    // M at A17 with coin 1 → Lick even though its 2^24 ticket is < 0.6 (separates 0.5 from 0.6).
    let m_lick = acid_find_stream("acid-m-b2-coin-1", |base, p24, coin| base == 50 && p24 < ACID_P60 && coin == 1);
    // L: A17 0.6 with a 2^24 ticket in [0.5, 0.6) → Spit; A16 0.4 → Lick; a coin of 1 would also give Lick.
    let l_spit = acid_find_stream("acid-l-b2-p60", |base, p24, coin| base == 50
        && (ACID_P50..ACID_P60).contains(&p24) && coin == 1);
    for (id, stream, a16, a17, second) in [
        (EnemyId::AcidSlimeM, &m_spit, AcidLick, AcidSpit, 2u128),
        (EnemyId::AcidSlimeM, &m_lick, AcidSpit, AcidLick, 2),
        (EnemyId::AcidSlimeL, &l_spit, AcidLick, AcidSpit, ACID_P24),
    ] {
        let (a16_ticket, _) = rng::uniform_below(&acid_after(stream, None), ACID_P24).unwrap();
        let a16 = if stream == &m_lick { if a16_ticket < ACID_P40 { AcidSpit } else { AcidLick } } else { a16 };
        let selected = select(id, &history, stream, 16).unwrap();
        assert_eq!((selected.intent, selected.stream), (a16, acid_after(stream, Some(ACID_P24))), "{id:?} A16 0.4");
        for ascension in [17, 20] {
            let selected = select(id, &history, stream, ascension).unwrap();
            assert_eq!((selected.intent, selected.stream), (a17, acid_after(stream, Some(second))),
                "{id:?} A{ascension} two-Tackle redirect");
        }
    }
}

#[test]
fn m32_acid_slime_a17_band3_last_lick_guard() {
    use FullIntentId::{AcidLick, AcidSpit, AcidTackle};
    // Band 3 (≥ 70; A17 M ≥ 80): A16 redirects only after TWO Licks; A17 after the LAST Lick (0.4 Spit / Tackle).
    let history = [AcidTackle, AcidLick];
    let tackle = acid_find_stream("acid-b3-tackle", |base, p24, _| base == 90 && p24 >= ACID_P40);
    let spit = acid_find_stream("acid-b3-spit", |base, p24, _| base == 90 && p24 < ACID_P40);
    for id in [EnemyId::AcidSlimeM, EnemyId::AcidSlimeL] {
        for (stream, a17) in [(&tackle, AcidTackle), (&spit, AcidSpit)] {
            let selected = select(id, &history, stream, 16).unwrap();
            assert_eq!((selected.intent, selected.stream), (AcidLick, acid_after(stream, None)),
                "{id:?} A16: one Lick is not the two-Lick guard; no redirect draw");
            for ascension in [17, 20] {
                let selected = select(id, &history, stream, ascension).unwrap();
                assert_eq!((selected.intent, selected.stream), (a17, acid_after(stream, Some(ACID_P24))),
                    "{id:?} A{ascension}: last Lick redirects at 0.4");
            }
        }
    }
}

#[test]
fn m32_acid_slime_l_a17_band1_redirect() {
    use FullIntentId::{AcidLick, AcidSpit, AcidTackle};
    // Band 1 after two Spits: A16 (both sizes) and A17 M use a coin; A17 L uses 0.6 (Tackle / Lick).
    let history = [AcidSpit, AcidSpit];
    let stream = acid_find_stream("acid-b1-l-p60", |base, p24, coin| base == 10
        && (ACID_P50..ACID_P60).contains(&p24) && coin == 1);
    for (id, ascension, expected, second) in [
        (EnemyId::AcidSlimeL, 16, AcidLick, 2u128),
        (EnemyId::AcidSlimeL, 17, AcidTackle, ACID_P24),
        (EnemyId::AcidSlimeL, 20, AcidTackle, ACID_P24),
        (EnemyId::AcidSlimeM, 16, AcidLick, 2),
        (EnemyId::AcidSlimeM, 17, AcidLick, 2),
    ] {
        let selected = select(id, &history, &stream, ascension).unwrap();
        assert_eq!((selected.intent, selected.stream), (expected, acid_after(&stream, Some(second))),
            "{id:?} A{ascension} band-1 redirect after two Spits");
    }
}
