//! M32 Act 2 behaviour rows (plan item 6; design §2.3, §6.4). Expected
//! selectors and payloads are taken from `m32-source-city-v1.md`; ticketed
//! cases pin `uniform_below` before calling the engine under test.
use super::super::enemies::{self, EnemyMemory};
use super::super::rng::{self, Domain, StreamId, StreamState};
use super::super::*;
use crate::content::{EncounterId, EnemyId};

fn ai_stream(actor: &str) -> StreamState {
    let id = StreamId::new(Domain::RunCombatEnemyAi, vec!["m32-act2".into(), actor.into()]).unwrap();
    StreamState::new(id, [32; 32], 0).unwrap()
}

fn stream_for_ticket_with_id(id: StreamId, bound: u128, wanted: u64) -> StreamState {
    for cursor in 0..100_000u128 {
        let stream = StreamState::new(id.clone(), [32; 32], cursor).unwrap();
        if rng::uniform_below(&stream, bound).unwrap().0 == wanted {
            return stream;
        }
    }
    panic!("could not construct a stream with ticket {wanted} / {bound}");
}

fn ticket_stream(bound: u128, wanted: u64) -> StreamState {
    stream_for_ticket_with_id(ai_stream("selector").id().clone(), bound, wanted)
}

fn selected(id: EnemyId, history: &[FullIntentId], memory: &EnemyMemory, stream: &StreamState, ascension: u8)
    -> enemies::MoveSelection {
    enemies::select_move_at(id, history, memory, stream, ascension).unwrap()
}

fn combat_at(encounter: EncounterId, ascension: u8) -> Combat {
    Combat::enter(super::m29_a2_owner::setup(encounter, ascension, 7, 0).unwrap()).unwrap()
}

fn slot(combat: &Combat, species: EnemyId) -> usize {
    combat.state.enemies.iter().position(|enemy| enemy.enemy_id == ActorEnemyId::Full(species))
        .unwrap_or_else(|| panic!("{species:?} absent from encounter"))
}

fn set_current(combat: &mut Combat, at: usize, name: &'static str, history: &[&'static str]) {
    let current = ActorIntentId::Full(FullIntentId::from_name(name));
    let actor = &mut combat.state.enemies[at];
    actor.intent_id = current.clone();
    actor.selected_move = current;
    actor.move_history = history.iter().map(|name| ActorIntentId::Full(FullIntentId::from_name(name))).collect();
    effects::refresh_enemy_values(&mut combat.state).unwrap();
}

fn install_ticket(combat: &mut Combat, at: usize, bound: u128, wanted: u64) -> (StreamState, StreamState) {
    let id = combat.state.enemies[at].full_ai_rng.as_ref().unwrap().id().clone();
    let stream = stream_for_ticket_with_id(id, bound, wanted);
    let (_, next) = rng::uniform_below(&stream, bound).unwrap();
    combat.state.enemies[at].full_ai_rng = Some(stream.clone());
    (stream, next)
}

fn public_intent(combat: &Combat, species: EnemyId) -> (String, Option<i32>) {
    let view = combat.public_full().unwrap();
    let actor = view.enemies.iter().find(|enemy| enemy.enemy_id == species).unwrap();
    (actor.intent.as_str().to_owned(), actor.intent_damage)
}

fn player_power(combat: &Combat, id: &str) -> i32 {
    combat.state.player_full_powers.iter().find(|power| power.id.0 == id).map_or(0, |power| power.state.amount)
}

fn enemy_power(combat: &Combat, at: usize, id: &str) -> i32 {
    combat.state.enemies[at].full_powers.iter().find(|power| power.id.0 == id).map_or(0, |power| power.state.amount)
}

fn contextual_select(combat: &Combat, at: usize, history: &[FullIntentId], stream: &StreamState)
    -> enemies::MoveSelection {
    let owner = combat.state.enemies[at].instance_id.clone();
    let actors = combat.state.enemies.iter().filter_map(|enemy| {
        let ActorEnemyId::Full(species) = enemy.enemy_id else { return None };
        matches!(enemy.life, ActorLife::Active | ActorLife::HalfDead).then(|| enemies::act2::ActorSummary {
            owner: enemy.instance_id.clone(), species, hp: enemy.hp, max_hp: enemy.max_hp,
            slot: combat.state.actor_slots[&enemy.instance_id],
        })
    }).collect();
    let context = enemies::act2::EnemyRollContext { owner, turn: combat.state.turn, actors, ascension: combat.state.ascension };
    let enemy = &combat.state.enemies[at];
    let ActorEnemyId::Full(species) = enemy.enemy_id else { panic!("contextual selector requires a full actor") };
    enemies::act2::select_move(species, history,
        enemy.full_memory.as_ref().unwrap(), stream, &context).unwrap()
}

#[test]
fn m32_romeo_a17_double_cross_slash() {
    for (ascension, expected) in [(16, "romeo_agonizing_slash"), (17, "romeo_cross_slash"), (20, "romeo_cross_slash")] {
        let mut combat = combat_at(EncounterId::MaskedBandits, ascension);
        let at = slot(&combat, EnemyId::BanditLeader);
        set_current(&mut combat, at, "romeo_cross_slash", &["romeo_mock", "romeo_agonizing_slash", "romeo_cross_slash"]);
        combat.step_full(&CombatAction::EndTurn).unwrap();
        assert!(combat.state.fault.is_none(), "A{ascension}: {:?}", combat.state.fault);
        assert_eq!(public_intent(&combat, EnemyId::BanditLeader).0, expected, "live Romeo roll at A{ascension}");
    }

    let history = [
        FullIntentId::from_name("romeo_mock"),
        FullIntentId::from_name("romeo_agonizing_slash"),
        FullIntentId::from_name("romeo_cross_slash"),
    ];
    let stream = ai_stream("romeo-selector");
    for (ascension, expected) in [(16, "romeo_agonizing_slash"), (17, "romeo_cross_slash"), (20, "romeo_cross_slash")] {
        let choice = selected(EnemyId::BanditLeader, &history, &EnemyMemory::BanditLeader, &stream, ascension);
        assert_eq!(choice.intent.as_str(), expected, "A{ascension}: source follow-up after Cross Slash");
        assert_eq!(choice.stream, stream, "Romeo's fixed transitions do not draw");
    }
    let two_crosses = [history[0], history[1], history[2], FullIntentId::from_name("romeo_cross_slash")];
    assert_eq!(selected(EnemyId::BanditLeader, &two_crosses, &EnemyMemory::BanditLeader, &stream, 17).intent.as_str(),
        "romeo_agonizing_slash", "two Cross Slashes reach Agonizing Slash");
}

#[test]
fn m32_chosen_a17_opening_hex() {
    for (ascension, expected) in [(16, "chosen_poke"), (17, "chosen_hex"), (20, "chosen_hex")] {
        let mut combat = combat_at(EncounterId::Chosen, ascension);
        let (intent, damage) = public_intent(&combat, EnemyId::Chosen);
        assert_eq!(intent, expected, "setup opening at A{ascension}");
        if expected == "chosen_hex" {
            assert_eq!(damage, None, "Hex has no damage intent");
        }
        if ascension == 17 {
            combat.step_full(&CombatAction::EndTurn).unwrap();
            assert!(combat.state.fault.is_none(), "A{ascension}: {:?}", combat.state.fault);
            assert_eq!(player_power(&combat, "hex"), 1, "the actual Hex turn applies Hex 1");
        }
    }

    let stream = ai_stream("chosen-opener");
    for ascension in [16, 17, 20] {
        let choice = selected(EnemyId::Chosen, &[], &EnemyMemory::Chosen, &stream, ascension);
        assert_eq!(choice.intent.as_str(), if ascension == 16 { "chosen_poke" } else { "chosen_hex" }, "A{ascension}");
        assert_eq!(choice.stream, stream, "both fixed opener paths keep the existing no-draw stream");
    }
}

#[test]
fn m32_mystic_a17_heal_cutoff_and_attack_guard() {
    for (ascension, expected) in [(16, "mystic_attack"), (17, "mystic_buff"), (20, "mystic_buff")] {
        let mut combat = combat_at(EncounterId::CenturionAndHealer, ascension);
        let mystic = slot(&combat, EnemyId::Mystic);
        for enemy in &mut combat.state.enemies { enemy.hp = enemy.max_hp; }
        set_current(&mut combat, mystic, "mystic_attack", &["mystic_attack"]);
        let (stream, next) = install_ticket(&mut combat, mystic, 100, 40);
        assert_eq!(rng::uniform_below(&stream, 100).unwrap().0, 40, "A{ascension} selector ticket");
        combat.step_full(&CombatAction::EndTurn).unwrap();
        assert!(combat.state.fault.is_none(), "A{ascension}: {:?}", combat.state.fault);
        assert_eq!(public_intent(&combat, EnemyId::Mystic).0, expected, "live repeat-Attack guard at A{ascension}");
        assert_eq!(player_power(&combat, "frail"), 2, "the current Attack applies Frail 2");
        assert_eq!(combat.state.enemies[mystic].full_ai_rng.as_ref().unwrap(), &next,
            "Mystic's live roll consumes one selector ticket");
    }

    for (ascension, expected, draws) in [
        (16, "mystic_heal", false),
        (17, "mystic_attack", true),
        (20, "mystic_attack", true),
    ] {
        let mut combat = combat_at(EncounterId::CenturionAndHealer, ascension);
        let mystic = slot(&combat, EnemyId::Mystic);
        let centurion = slot(&combat, EnemyId::Centurion);
        for enemy in &mut combat.state.enemies { enemy.hp = enemy.max_hp; }
        combat.state.enemies[mystic].hp -= 8;
        combat.state.enemies[centurion].hp -= 8;
        let stream = ticket_stream(100, 40);
        assert_eq!(rng::uniform_below(&stream, 100).unwrap().0, 40, "A{ascension} selector ticket");
        let choice = contextual_select(&combat, mystic, &[], &stream);
        assert_eq!(choice.intent.as_str(), expected, "live roster deficit is 16 at A{ascension}");
        assert_eq!(choice.stream, if draws { rng::uniform_below(&stream, 100).unwrap().1 } else { stream.clone() },
            "Heal is forced without a selector draw; ordinary branches consume one");
    }
}

#[test]
fn m32_shelled_parasite_a17_opening_fell() {
    for ascension in [17, 20] {
        let mut combat = combat_at(EncounterId::ShelledParasite, ascension);
        let (intent, damage) = public_intent(&combat, EnemyId::ShelledParasite);
        assert_eq!(intent, "shelled_parasite_fell", "forced setup opener at A{ascension}");
        assert_eq!(damage, Some(21), "Fell's source-table base damage");
        combat.step_full(&CombatAction::EndTurn).unwrap();
        assert!(combat.state.fault.is_none(), "A{ascension}: {:?}", combat.state.fault);
        assert_eq!(player_power(&combat, "frail"), 2, "Fell applies Frail 2");
    }

    let memory = EnemyMemory::ShelledParasite { first: true };
    let stream = ticket_stream(2, 0);
    assert_eq!(rng::uniform_below(&stream, 2).unwrap().0, 0, "A16 opening coin fixture");
    let at16 = selected(EnemyId::ShelledParasite, &[], &memory, &stream, 16);
    assert_eq!(at16.intent.as_str(), "shelled_parasite_double_strike");
    assert_eq!(at16.stream, rng::uniform_below(&stream, 2).unwrap().1);
    for ascension in [17, 20] {
        let choice = selected(EnemyId::ShelledParasite, &[], &memory, &stream, ascension);
        assert_eq!(choice.intent.as_str(), "shelled_parasite_fell", "A{ascension}");
        assert_eq!(choice.memory, EnemyMemory::ShelledParasite { first: false });
        assert_eq!(choice.stream, stream, "the forced opener drops the conditional coin draw");
    }
}

#[test]
fn m32_snake_plant_a17_spores_guard() {
    for (ascension, expected) in [(16, "snake_plant_enfeebling_spores"), (17, "snake_plant_chomp_chomp"), (20, "snake_plant_chomp_chomp")] {
        let mut combat = combat_at(EncounterId::SnakePlant, ascension);
        let at = slot(&combat, EnemyId::SnakePlant);
        set_current(&mut combat, at, "snake_plant_chomp_chomp", &[
            "snake_plant_chomp_chomp", "snake_plant_enfeebling_spores", "snake_plant_chomp_chomp",
        ]);
        let (live_stream, next) = install_ticket(&mut combat, at, 100, 65);
        assert_eq!(rng::uniform_below(&live_stream, 100).unwrap().0, 65, "live A{ascension} ticket");
        combat.step_full(&CombatAction::EndTurn).unwrap();
        assert!(combat.state.fault.is_none(), "A{ascension}: {:?}", combat.state.fault);
        assert_eq!(public_intent(&combat, EnemyId::SnakePlant).0, expected, "live next intent at A{ascension}");
        assert_eq!(combat.state.enemies[at].full_ai_rng.as_ref().unwrap(), &next, "one live selector draw");
    }

    let history = [
        FullIntentId::from_name("snake_plant_chomp_chomp"),
        FullIntentId::from_name("snake_plant_enfeebling_spores"),
        FullIntentId::from_name("snake_plant_chomp_chomp"),
    ];
    let stream = ticket_stream(100, 65);
    assert_eq!(rng::uniform_below(&stream, 100).unwrap().0, 65, "high-branch separating ticket");

    for (ascension, expected) in [(16, "snake_plant_enfeebling_spores"), (17, "snake_plant_chomp_chomp"), (20, "snake_plant_chomp_chomp")] {
        let choice = selected(EnemyId::SnakePlant, &history, &EnemyMemory::SnakePlant, &stream, ascension);
        assert_eq!(choice.intent.as_str(), expected, "A{ascension}");
        assert_eq!(choice.stream, rng::uniform_below(&stream, 100).unwrap().1, "one base selector draw at A{ascension}");
    }

}

#[test]
fn m32_snecko_a17_tail_whip_weak() {
    let stream = ticket_stream(100, 0);
    assert_eq!(rng::uniform_below(&stream, 100).unwrap().0, 0, "Tail Whip ticket");
    for ascension in [16, 17, 20] {
        let choice = selected(EnemyId::Snecko, &[FullIntentId::from_name("snecko_glare")], &EnemyMemory::Snecko,
            &stream, ascension);
        assert_eq!(choice.intent.as_str(), "snecko_tail_whip", "A{ascension}");
        assert_eq!(choice.stream, rng::uniform_below(&stream, 100).unwrap().1, "unchanged one-ticket selector");

        let mut combat = combat_at(EncounterId::Snecko, ascension);
        let at = slot(&combat, EnemyId::Snecko);
        set_current(&mut combat, at, "snecko_tail_whip", &["snecko_tail_whip"]);
        let (intent, damage) = public_intent(&combat, EnemyId::Snecko);
        assert_eq!(intent, "snecko_tail_whip");
        assert_eq!(damage, Some(10), "public intent damage is resolved before queued debuffs");
        let selected_actor = combat.state.enemies[at].clone();
        let hp = combat.state.player.hp;
        combat.step_full(&CombatAction::EndTurn).unwrap();
        assert!(combat.state.fault.is_none(), "A{ascension}: {:?}", combat.state.fault);
        assert_eq!(hp - combat.state.player.hp, 10, "Tail damage resolves before its debuffs at A{ascension}");
        assert_eq!(player_power(&combat, "weak"), if ascension >= 17 { 2 } else { 0 }, "Tail Weak at A{ascension}");
        assert_eq!(player_power(&combat, "vulnerable"), 2, "Tail Vulnerable at A{ascension}");
        if ascension >= 17 {
            let work = enemies::move_work_at(&selected_actor, ascension).unwrap();
            assert!(matches!(work[0].kind, EffectKind::Damage(_)), "Tail damage is queued first");
            assert!(matches!(&work[1].kind, EffectKind::ApplyFullPower { id, amount: 2 } if id.0 == "weak"),
                "Weak 2 is queued after damage");
            assert!(matches!(&work[2].kind, EffectKind::ApplyFullPower { id, amount: 2 } if id.0 == "vulnerable"),
                "Vulnerable 2 is queued after Weak 2");
        }
    }
}

#[test]
fn m32_book_of_stabbing_a18_big_stab_counts() {
    let multi = FullIntentId::from_name("book_multi");
    let big = FullIntentId::from_name("book_big");
    let memory = EnemyMemory::BookOfStabbing { stab_count: 3 };
    let high_stream = ticket_stream(100, 15);
    let low_stream = ticket_stream(100, 14);
    assert_eq!(rng::uniform_below(&high_stream, 100).unwrap().0, 15, "high-branch boundary");
    assert_eq!(rng::uniform_below(&low_stream, 100).unwrap().0, 14, "low-branch boundary");

    for (ascension, expected_big_count, expected_multi_count) in [(17, 3, 4), (18, 4, 5), (20, 4, 5)] {
        let mut combat = combat_at(EncounterId::BookOfStabbing, ascension);
        let at = slot(&combat, EnemyId::BookOfStabbing);
        combat.state.enemies[at].full_memory = Some(memory.clone());
        set_current(&mut combat, at, "book_multi", &["book_multi", "book_multi"]);
        let (high_live, high_next) = install_ticket(&mut combat, at, 100, 15);
        assert_eq!(rng::uniform_below(&high_live, 100).unwrap().0, 15);
        combat.step_full(&CombatAction::EndTurn).unwrap();
        assert!(combat.state.fault.is_none(), "A{ascension}: {:?}", combat.state.fault);
        assert_eq!(public_intent(&combat, EnemyId::BookOfStabbing), ("book_big".to_owned(), Some(24)),
            "live roll selects Big with its public base damage at A{ascension}");
        assert_eq!(combat.state.enemies[at].full_ai_rng.as_ref().unwrap(), &high_next);
        assert_eq!(combat.state.enemies[at].full_memory, Some(EnemyMemory::BookOfStabbing { stab_count: expected_big_count }),
            "live Big counter at A{ascension}");

        let (low_live, low_next) = install_ticket(&mut combat, at, 100, 14);
        assert_eq!(rng::uniform_below(&low_live, 100).unwrap().0, 14);
        combat.step_full(&CombatAction::EndTurn).unwrap();
        assert!(combat.state.fault.is_none(), "A{ascension}: {:?}", combat.state.fault);
        assert_eq!(public_intent(&combat, EnemyId::BookOfStabbing), ("book_multi".to_owned(), Some(7)),
            "hit count is not public; Multi-Stab's public intent damage is its base");
        assert_eq!(combat.state.enemies[at].full_ai_rng.as_ref().unwrap(), &low_next);
        assert_eq!(combat.state.enemies[at].full_memory, Some(EnemyMemory::BookOfStabbing { stab_count: expected_multi_count }),
            "live next Multi counter at A{ascension}");

        let hp = combat.state.player.hp;
        let events = combat.state.damage_events_this_combat;
        combat.step_full(&CombatAction::EndTurn).unwrap();
        assert!(combat.state.fault.is_none(), "A{ascension}: {:?}", combat.state.fault);
        assert_eq!(hp - combat.state.player.hp, expected_multi_count as i32 * 7,
            "resolved Multi-Stab damage at A{ascension}");
        assert_eq!(combat.state.damage_events_this_combat - events, expected_multi_count,
            "each Multi-Stab hit resolves as one damage effect at A{ascension}");
    }

    for (ascension, expected_big_count) in [(17, 3), (18, 4), (20, 4)] {
        let high = selected(EnemyId::BookOfStabbing, &[multi, multi], &memory, &high_stream, ascension);
        assert_eq!(high.intent, big, "A{ascension} ticket 15 selects Big after two Multi-Stabs");
        assert_eq!(high.memory, EnemyMemory::BookOfStabbing { stab_count: expected_big_count }, "Big selection count at A{ascension}");
        let low = selected(EnemyId::BookOfStabbing, &[multi], &memory, &low_stream, ascension);
        assert_eq!(low.intent, big, "A{ascension} ticket 14 selects Big when the last move was not Big");
        assert_eq!(low.memory, EnemyMemory::BookOfStabbing { stab_count: expected_big_count }, "low-branch Big count at A{ascension}");
        let next = selected(EnemyId::BookOfStabbing, &[multi, multi, big], &high.memory, &low_stream, ascension);
        assert_eq!(next.intent, multi, "the low ticket after Big selects Multi at A{ascension}");
        assert_eq!(next.memory, EnemyMemory::BookOfStabbing { stab_count: expected_big_count + 1 },
            "next Multi increments its counter at A{ascension}");
    }
}

#[test]
fn m32_champ_a19_forge_cutoff() {
    for (ascension, expected) in [(18, "champ_slap"), (19, "champ_stance"), (20, "champ_stance")] {
        let mut combat = combat_at(EncounterId::Champ, ascension);
        let at = slot(&combat, EnemyId::Champ);
        combat.state.enemies[at].full_memory = Some(EnemyMemory::Champ { turns: 0, forges: 0, angered: false });
        set_current(&mut combat, at, "champ_gloat", &["champ_gloat"]);
        let (stream, next) = install_ticket(&mut combat, at, 100, 20);
        assert_eq!(rng::uniform_below(&stream, 100).unwrap().0, 20, "the ticket separates A18 from A19");
        combat.step_full(&CombatAction::EndTurn).unwrap();
        assert!(combat.state.fault.is_none(), "A{ascension}: {:?}", combat.state.fault);
        assert_eq!(public_intent(&combat, EnemyId::Champ).0, expected, "live cutoff at A{ascension}");
        assert_eq!(combat.state.enemies[at].full_ai_rng.as_ref().unwrap(), &next, "ordinary Champ tree uses one ticket");
        if ascension >= 19 {
            set_current(&mut combat, at, "champ_stance", &["champ_stance"]);
            let block_before = combat.state.enemies[at].block;
            combat.step_full(&CombatAction::EndTurn).unwrap();
            assert!(combat.state.fault.is_none(), "A{ascension}: {:?}", combat.state.fault);
            assert_eq!(combat.state.enemies[at].block - block_before, 27,
                "Stance queues Block 20, then Metallicize 7 grants its end-turn Block");
            assert_eq!(enemy_power(&combat, at, "metallicize"), 7, "A19 Defensive Stance Metallicize");
        }
    }
}

#[test]
fn m32_automaton_a19_boost_after_beam() {
    for (ascension, expected) in [(18, "automaton_stun"), (19, "automaton_boost"), (20, "automaton_boost")] {
        let mut combat = combat_at(EncounterId::BronzeAutomaton, ascension);
        let at = slot(&combat, EnemyId::BronzeAutomaton);
        combat.state.enemies[at].full_memory = Some(EnemyMemory::BronzeAutomaton { turns: 0 });
        set_current(&mut combat, at, "automaton_beam", &["automaton_beam"]);
        let stream_before = combat.state.enemies[at].full_ai_rng.clone().unwrap();
        combat.step_full(&CombatAction::EndTurn).unwrap();
        assert!(combat.state.fault.is_none(), "A{ascension}: {:?}", combat.state.fault);
        assert_eq!(public_intent(&combat, EnemyId::BronzeAutomaton).0, expected, "live post-Beam selection at A{ascension}");
        assert_eq!(combat.state.enemies[at].full_ai_rng.as_ref().unwrap(), &stream_before,
            "the deterministic after-Beam branch uses no selector draw");
        assert_eq!(combat.state.enemies[at].full_memory,
            Some(EnemyMemory::BronzeAutomaton { turns: 0 }), "the after-Beam early return does not advance turns");
        if ascension >= 19 {
            let block_before = combat.state.enemies[at].block;
            set_current(&mut combat, at, "automaton_boost", &["automaton_beam", "automaton_boost"]);
            combat.step_full(&CombatAction::EndTurn).unwrap();
            assert!(combat.state.fault.is_none(), "A{ascension}: {:?}", combat.state.fault);
            assert_eq!(combat.state.enemies[at].block - block_before, 12, "A19 Boost block");
            assert_eq!(enemy_power(&combat, at, "strength"), 4, "A19 Boost strength");
        }
    }
}
