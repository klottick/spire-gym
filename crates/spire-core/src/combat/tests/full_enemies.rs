use super::super::enemies::{self, EnemyMemory};
use super::super::rng::{self, Domain, StreamState};
use super::super::*;
use crate::content::EnemyId;
use super::super::card_state::permanent as live_card;

fn ai_stream(actor: &str) -> StreamState {
    stream(Domain::RunCombatEnemyAi, vec!["encounter-1".into(), actor.into()])
}

fn stream(domain: Domain, scope: Vec<String>) -> StreamState {
    let id = rng::StreamId::new(domain, scope).unwrap();
    rng::make_stream(&[17; 32], rng::Partition::Test, "enemy-first-join", &id).unwrap()
}

fn enemy(id: EnemyId, intent: FullIntentId, hp: i32) -> EnemyState {
    let mut e = EnemyState {
        instance_id: EnemyHandle::parse("actor-1").unwrap(), enemy_id: ActorEnemyId::Full(id),
        hp, max_hp: hp, block: 0, powers: vec![], full_powers: vec![],
        intent_id: ActorIntentId::Full(intent), move_history: vec![], selected_move: ActorIntentId::Full(intent),
        damage_entries: Default::default(), full_damage_entries: Default::default(),
        full_ai_rng: Some(ai_stream("actor-1")), full_memory: Some(match id {
            EnemyId::Cultist => EnemyMemory::Cultist { ascension: 0 },
            EnemyId::JawWorm => EnemyMemory::JawWorm { ascension: 0, variant: enemies::JawWormVariant::Act1 },
            EnemyId::GremlinNob => EnemyMemory::GremlinNob { ascension: 0 },
            EnemyId::LouseRed => EnemyMemory::LouseRed { curl_up_used: false },
            EnemyId::LouseGreen => EnemyMemory::LouseGreen { curl_up_used: false },
            EnemyId::AcidSlimeS | EnemyId::AcidSlimeM | EnemyId::AcidSlimeL => EnemyMemory::AcidSlime { split_triggered: false },
            EnemyId::SpikeSlimeS | EnemyId::SpikeSlimeM | EnemyId::SpikeSlimeL => EnemyMemory::SpikeSlime { split_triggered: false },
            EnemyId::SlimeBoss => EnemyMemory::SlimeBoss { split_triggered: false },
            EnemyId::Lagavulin|EnemyId::Sentry|EnemyId::Guardian|EnemyId::Hexaghost|EnemyId::SphericGuardian|EnemyId::Byrd|EnemyId::Chosen|EnemyId::Snecko|EnemyId::Centurion|EnemyId::Mystic|EnemyId::GremlinLeader|EnemyId::GremlinMad|EnemyId::GremlinSneaky|EnemyId::GremlinFat|EnemyId::GremlinShield|EnemyId::GremlinWizard|EnemyId::SlaverBlue|EnemyId::Taskmaster|EnemyId::SlaverRed|EnemyId::BookOfStabbing|EnemyId::Champ|EnemyId::Collector|EnemyId::TorchHead|EnemyId::BronzeAutomaton|EnemyId::BronzeOrb=>EnemyMemory::for_enemy(id).unwrap(),
            EnemyId::OrbWalker|EnemyId::Repulsor|EnemyId::Exploder|EnemyId::Spiker|EnemyId::Darkling=>EnemyMemory::for_enemy(id).unwrap(),
            EnemyId::SpireGrowth|EnemyId::Transient|EnemyId::Maw=>EnemyMemory::for_enemy(id).unwrap(),
            EnemyId::WrithingMass=>EnemyMemory::for_enemy(id).unwrap(),
            EnemyId::GiantHead|EnemyId::Nemesis=>EnemyMemory::for_enemy(id).unwrap(),
            EnemyId::Reptomancer|EnemyId::Dagger=>EnemyMemory::for_enemy(id).unwrap(),
            EnemyId::AwakenedOne=>EnemyMemory::for_enemy(id).unwrap(),
            EnemyId::TimeEater=>EnemyMemory::for_enemy(id).unwrap(),
            EnemyId::Deca|EnemyId::Donu|EnemyId::SpireShield|EnemyId::SpireSpear|EnemyId::CorruptHeart=>EnemyMemory::for_enemy(id).unwrap(),
            EnemyId::FungiBeast|EnemyId::Looter|EnemyId::Mugger|EnemyId::ShelledParasite|EnemyId::SnakePlant=>EnemyMemory::for_enemy(id).unwrap(),
            EnemyId::Unknown(_) => unreachable!(),
        }), intent_damage: None, life: super::super::model::ActorLife::Active,
    };
    e.move_history.push(ActorIntentId::Full(intent));
    e
}

fn jaw_combat() -> Combat {
    let scenario = super::support::base_scenario();
    let setup = FullCombatSetup {
        player: scenario.player,
        player_powers: vec![],
        shuffle_rng: stream(Domain::RunCombatShuffle, vec!["encounter-1".into()]),
        character: crate::content::Character::Ironclad,
        card_rng: stream(Domain::RunCombatCardRandom, vec!["encounter-1".into()]),
        enemies: vec![FullEnemySetup { initial_memory:None,
            instance_id: EnemyHandle::parse("jaw-1").unwrap(), enemy_id: EnemyId::JawWorm,
            profile: enemies::EnemyProfile::Standard,
            hp: 42, max_hp: 42, block: 0, intent: FullIntentId::Chomp, powers: vec![],
            ai_rng: ai_stream("jaw-1"), ascension: 0,
        }], enemy_damage_bases: Default::default(),
    };
    Combat::enter(CombatEntry { setup, opening_draw: 3, cards: vec![
        live_card(crate::content::CardId::Strike, CardInstanceId(1), 0),
        live_card(crate::content::CardId::Defend, CardInstanceId(2), 0),
        live_card(crate::content::CardId::Bash, CardInstanceId(3), 0),
    ] }).unwrap()
}

#[test]
fn setup_uses_supplied_player_deck_and_actor_scoped_streams() {
    let player = super::support::base_scenario().player;
    let mut run_rng = crate::run::RunRng::new([23; 32], rng::Partition::Test, 7);
    let entry = enemies::setup(crate::content::EncounterId::JawWorm, 1, 0, &mut run_rng,
        enemies::SetupContext { player: player.clone(), player_powers: vec![], character: crate::content::Character::Watcher,
            cards: vec![live_card(crate::content::CardId::Strike, CardInstanceId(501), 0)],
            opening_draw: 1, encounter_visit: 2 }).unwrap();
    assert_eq!(entry.setup.player, player, "enemy setup must not synthesize or overwrite run HP/resources");
    assert_eq!(entry.setup.character, crate::content::Character::Watcher);
    assert_eq!(entry.cards.iter().map(|card| card.instance).collect::<Vec<_>>(), vec![CardInstanceId(501)]);
    let jaw = &entry.setup.enemies[0];
    assert!((40..=44).contains(&jaw.hp));
    assert_eq!(jaw.hp, jaw.max_hp);
    assert_eq!(jaw.intent, FullIntentId::Chomp);
    assert_eq!(jaw.ai_rng.id().scope()[0], entry.setup.shuffle_rng.id().scope()[0]);
    assert_eq!(entry.setup.card_rng.id().domain(), Domain::RunCombatCardRandom);
    assert_eq!(entry.setup.card_rng.id().scope().len(), 1);
    assert_eq!(entry.setup.card_rng.id().scope()[0], entry.setup.shuffle_rng.id().scope()[0]);
    assert_ne!(entry.setup.card_rng.id(), entry.setup.shuffle_rng.id());
    assert_ne!(jaw.ai_rng.id().scope()[1], entry.setup.shuffle_rng.id().scope()[0]);
}

#[test]
fn cultist_and_jaw_worm_setup_support_all_twenty_one_ascensions_and_reject_twenty_one_atomically() {
    use crate::content::EncounterId;
    let context = || {
        let player = super::support::base_scenario().player;
        enemies::SetupContext { player, player_powers: vec![], character: crate::content::Character::Ironclad,
            cards: vec![live_card(crate::content::CardId::Strike, CardInstanceId(941), 0)],
            opening_draw: 1, encounter_visit: 41 }
    };
    for ascension in 0..=20 {
        for (encounter, id, expected_hp) in [
            (EncounterId::Cultist, EnemyId::Cultist, if ascension >= 7 { 50..=56 } else { 48..=54 }),
            (EncounterId::JawWorm, EnemyId::JawWorm, if ascension >= 7 { 42..=46 } else { 40..=44 }),
        ] {
            let mut rng = crate::run::RunRng::new([71; 32], rng::Partition::Test, ascension as u64);
            let entry = enemies::setup(encounter, 1, ascension, &mut rng, context()).unwrap();
            assert_eq!(entry.setup.enemies.len(), 1);
            let actor = &entry.setup.enemies[0];
            assert_eq!(actor.enemy_id, id);
            assert!(expected_hp.contains(&actor.hp), "{encounter:?} A{ascension}: {}", actor.hp);
            assert_eq!(actor.hp, actor.max_hp);
            assert_eq!(actor.intent, match id { EnemyId::Cultist => FullIntentId::Incantation,
                EnemyId::JawWorm => FullIntentId::Chomp, _ => unreachable!() });
            assert!(entry.setup.enemy_damage_bases.is_empty(),
                "Cultist/Jaw use canonical species damage, not sampled per-slot overrides");
            let expected_hp = actor.hp;
            let expected_intent = actor.intent;
            let mut combat = Combat::enter(entry).unwrap();
            let public = combat.public_full().unwrap();
            assert_eq!(public.enemies[0].hp, expected_hp);
            assert_eq!(public.enemies[0].intent, expected_intent);
            let expected_damage = match id {
                EnemyId::Cultist => None,
                EnemyId::JawWorm => Some(if ascension >= 2 { 12 } else { 11 }),
                _ => unreachable!(),
            };
            assert_eq!(public.enemies[0].intent_damage, expected_damage,
                "canonical public damage for {encounter:?} A{ascension}");
            assert_eq!(combat.snapshot().state.enemies[0].full_memory.as_ref().map(|memory| memory.ascension()),
                Some(ascension), "snapshot retains native per-actor ascension configuration");
            if id == EnemyId::JawWorm {
                let hp = public.player.hp;
                combat.step_full(&CombatAction::EndTurn).unwrap();
                assert_eq!(combat.public_full().unwrap().player.hp,
                    hp - if ascension >= 2 { 12 } else { 11 },
                    "first native Jaw Chomp at A{ascension}");
            }
        }
    }
    for encounter in [EncounterId::Cultist, EncounterId::JawWorm] {
        let mut rng = crate::run::RunRng::new([72; 32], rng::Partition::Test, 1);
        let before = rng.clone();
        assert!(enemies::setup(encounter, 1, 21, &mut rng, context()).is_err());
        assert_eq!(rng, before, "invalid ascension rejects before consuming setup RNG");
    }
}

#[test]
fn a20_cultist_incantation_and_later_attack_use_the_live_ritual_amount() {
    let mut run_rng = crate::run::RunRng::new([93; 32], rng::Partition::Test, 20);
    let context = enemies::SetupContext { player: super::support::base_scenario().player,
        player_powers: vec![], character: crate::content::Character::Ironclad,
        cards: vec![live_card(crate::content::CardId::Strike, CardInstanceId(942), 0)],
        opening_draw: 1, encounter_visit: 52 };
    let entry = enemies::setup(crate::content::EncounterId::Cultist, 1, 20, &mut run_rng, context).unwrap();
    let mut combat = Combat::enter(entry).unwrap();

    combat.step_full(&CombatAction::EndTurn).unwrap();
    assert!(combat.state.enemies[0].full_powers.iter().any(|power|
        power.id == FullPowerId("ritual") && power.state.amount == 5));
    assert_eq!(combat.state.enemies[0].full_memory,
        Some(EnemyMemory::Cultist { ascension: 20 }));

    let hp = combat.public_full().unwrap().player.hp;
    combat.step_full(&CombatAction::EndTurn).unwrap();
    assert_eq!(combat.public_full().unwrap().player.hp, hp - 6,
        "the first Attack resolves before Ritual grants its A20 Strength");
    assert!(combat.state.enemies[0].full_powers.iter().any(|power|
        power.id == FullPowerId("strength") && power.state.amount == 5));
    let hp = combat.public_full().unwrap().player.hp;
    combat.step_full(&CombatAction::EndTurn).unwrap();
    assert_eq!(combat.public_full().unwrap().player.hp, hp - 11,
        "the later A20 Attack uses canonical base six plus five Strength");
    assert_eq!(combat.state.enemies[0].full_damage_entries[&FullIntentId::Attack].base, 6);
    assert_eq!(combat.state.enemies[0].full_damage_entries[&FullIntentId::Attack].output, 16,
        "post-attack Ritual refreshes to ten cumulative Strength");
}

#[test]
fn cultist_and_jaw_worm_live_payloads_change_only_at_the_qualified_thresholds() {
    for (ascension, ritual, jaw_strength, jaw_block) in [
        (1, 3, 3, 6), (2, 4, 4, 6), (6, 4, 4, 6),
        (7, 4, 4, 6), (16, 4, 4, 6), (17, 5, 5, 9), (20, 5, 5, 9),
    ] {
        let mut cultist = enemy(EnemyId::Cultist, FullIntentId::Incantation, 52);
        cultist.full_memory = Some(EnemyMemory::Cultist { ascension });
        assert!(matches!(enemies::move_work(&cultist).unwrap().as_slice(), [Effect {
            kind: EffectKind::ApplyFullPower { id: FullPowerId("ritual"), amount }, ..
        }] if *amount == ritual), "Cultist Ritual amount at A{ascension}");

        let mut jaw = enemy(EnemyId::JawWorm, FullIntentId::Bellow, 44);
        jaw.full_memory = Some(EnemyMemory::JawWorm { ascension, variant: enemies::JawWormVariant::Act1 });
        assert!(matches!(enemies::move_work(&jaw).unwrap().as_slice(), [
            Effect { kind: EffectKind::ApplyFullPower { id: FullPowerId("strength"), amount }, .. },
            Effect { kind: EffectKind::GainBlock(block), .. },
        ] if *amount == jaw_strength && *block == jaw_block), "Jaw Bellow at A{ascension}");
    }
}

#[test]
fn a20_jaw_worm_resolves_chomp_bellow_and_thrashing_damage_from_one_actor_config() {
    let mut run_rng = crate::run::RunRng::new([94; 32], rng::Partition::Test, 21);
    let context = enemies::SetupContext { player: super::support::base_scenario().player,
        player_powers: vec![], character: crate::content::Character::Ironclad,
        cards: vec![live_card(crate::content::CardId::Strike, CardInstanceId(943), 0)],
        opening_draw: 1, encounter_visit: 53 };
    let entry = enemies::setup(crate::content::EncounterId::JawWorm, 1, 20, &mut run_rng, context).unwrap();
    let mut combat = Combat::enter(entry).unwrap();

    let hp = combat.public_full().unwrap().player.hp;
    combat.step_full(&CombatAction::EndTurn).unwrap();
    assert_eq!(combat.public_full().unwrap().player.hp, hp - 12, "A20 opening Chomp");

    {
        let jaw = &mut combat.state.enemies[0];
        jaw.intent_id = ActorIntentId::Full(FullIntentId::Bellow);
        jaw.selected_move = ActorIntentId::Full(FullIntentId::Bellow);
        jaw.move_history.push(ActorIntentId::Full(FullIntentId::Bellow));
        jaw.intent_damage = None;
    }
    combat.step_full(&CombatAction::EndTurn).unwrap();
    assert!(combat.state.enemies[0].full_powers.iter().any(|power|
        power.id == FullPowerId("strength") && power.state.amount == 5));
    assert_eq!(combat.state.enemies[0].block, 9, "A20 Bellow Block");
    let hp = combat.public_full().unwrap().player.hp;

    {
        let jaw = &mut combat.state.enemies[0];
        jaw.intent_id = ActorIntentId::Full(FullIntentId::Thrash);
        jaw.selected_move = ActorIntentId::Full(FullIntentId::Thrash);
        jaw.move_history.push(ActorIntentId::Full(FullIntentId::Thrash));
        jaw.intent_damage = Some(12);
    }
    combat.step_full(&CombatAction::EndTurn).unwrap();
    assert_eq!(combat.public_full().unwrap().player.hp, hp - 12,
        "A20 Thrash base seven plus five Strength");
    assert_eq!(combat.state.enemies[0].block, 5, "Thrash retains its unchanged five Block");
}

fn encounter_context(visit: u64, card_id: u64) -> enemies::SetupContext {
    let mut player = super::support::base_scenario().player;
    player.hp = 200;
    player.max_hp = 200;
    enemies::SetupContext { player, player_powers: vec![],
        character: crate::content::Character::Ironclad,
        cards: vec![live_card(crate::content::CardId::Strike, CardInstanceId(card_id), 0)],
        opening_draw: 1, encounter_visit: visit }
}

#[test]
fn blue_slaver_and_two_fungi_encounters_construct_real_actors_across_ascensions() {
    use crate::content::EncounterId;

    for ascension in 0..=20 {
        let visit = 300 + u64::from(ascension);
        let mut slaver_rng = crate::run::RunRng::new([121; 32], rng::Partition::Test, ascension as u64);
        let slaver = enemies::setup(EncounterId::BlueSlaver, 1, ascension, &mut slaver_rng,
            encounter_context(visit, 20_000 + u64::from(ascension)))
            .expect("Blue Slaver encounter setup must construct its real solo actor");
        assert_eq!(slaver.setup.enemies.len(), 1);
        let actor = &slaver.setup.enemies[0];
        assert_eq!(actor.enemy_id, EnemyId::SlaverBlue);
        assert_eq!(actor.instance_id.as_str(), format!("slaver_blue-{visit}-0"));
        assert!((if ascension >= 7 { 48..=52 } else { 46..=50 }).contains(&actor.hp));
        assert!(matches!(actor.intent.as_str(), "slaver_rake" | "slaver_stab"));

        Combat::enter(slaver).expect("solo actor must enter the real resolver at every supported ascension");

        let mut fungi_rng = crate::run::RunRng::new([122; 32], rng::Partition::Test, ascension as u64);
        let fungi = enemies::setup(EncounterId::TwoFungiBeasts, 1, ascension, &mut fungi_rng,
            encounter_context(visit, 21_000 + u64::from(ascension)))
            .expect("Two Fungi encounter setup must construct its ordered actors");
        assert_eq!(fungi.setup.enemies.len(), 2);
        for (slot, beast) in fungi.setup.enemies.iter().enumerate() {
            assert_eq!(beast.enemy_id.as_str(), "fungi_beast");
            assert_eq!(beast.instance_id.as_str(), format!("fungi_beast-{visit}-{slot}"));
            assert!((if ascension >= 7 { 24..=28 } else { 22..=28 }).contains(&beast.hp));
            assert!(matches!(beast.intent.as_str(), "bite" | "grow"));
            assert!(beast.powers.iter().any(|power|
                power.id == FullPowerId("spore_cloud") && power.state.amount == 2));
        }
        Combat::enter(fungi).expect("both fungi actors must enter the real resolver at every supported ascension");
    }
}

#[test]
fn act2_three_cultists_are_ordered_independent_and_resolve_real_ritual_and_later_attacks_at_all_ascensions() {
    use crate::content::EncounterId;
    for ascension in 0..=20 {
        let visit = 80 + u64::from(ascension);
        let mut run_rng = crate::run::RunRng::new([105; 32], rng::Partition::Test, ascension as u64);
        let entry = enemies::setup(EncounterId::ThreeCultists, 2, ascension, &mut run_rng,
            encounter_context(visit, 10_000 + u64::from(ascension))).unwrap();
        assert_eq!(entry.setup.enemies.len(), 3);
        assert!(entry.setup.enemy_damage_bases.is_empty());
        for (slot, actor) in entry.setup.enemies.iter().enumerate() {
            assert_eq!(actor.enemy_id, EnemyId::Cultist);
            assert_eq!(actor.profile, enemies::EnemyProfile::Standard);
            assert_eq!(actor.instance_id.as_str(), format!("cultist-{visit}-{slot}"));
            assert_eq!(actor.ascension, ascension);
            assert_eq!(actor.intent, FullIntentId::Incantation);
            assert!((if ascension >= 7 { 50..=56 } else { 48..=54 }).contains(&actor.hp));
            assert_eq!(actor.hp, actor.max_hp);
            assert_eq!(actor.ai_rng.id().scope()[1], format!("slot-{slot}-cultist-{visit}-{slot}"));
        }
        assert_ne!(entry.setup.enemies[0].ai_rng.id(), entry.setup.enemies[1].ai_rng.id());
        assert_ne!(entry.setup.enemies[1].ai_rng.id(), entry.setup.enemies[2].ai_rng.id());

        let mut combat = Combat::enter(entry).unwrap();
        let public = combat.public_full().unwrap();
        assert_eq!(public.enemies.iter().map(|enemy| enemy.intent).collect::<Vec<_>>(),
            vec![FullIntentId::Incantation; 3]);
        let ritual = if ascension >= 17 { 5 } else if ascension >= 2 { 4 } else { 3 };
        combat.step_full(&CombatAction::EndTurn).unwrap();
        assert!(combat.state.enemies.iter().all(|actor| actor.full_powers.iter().any(|power|
            power.id == FullPowerId("ritual") && power.state.amount == ritual)));
        assert!(combat.state.enemies.iter().all(|actor| actor.intent_id == ActorIntentId::Full(FullIntentId::Attack)));

        let hp = combat.public_full().unwrap().player.hp;
        combat.step_full(&CombatAction::EndTurn).unwrap();
        assert_eq!(combat.public_full().unwrap().player.hp, hp - 18,
            "each Cultist's first Attack remains base six before Ritual grants Strength");
        assert!(combat.state.enemies.iter().all(|actor| actor.full_powers.iter().any(|power|
            power.id == FullPowerId("strength") && power.state.amount == ritual)));

        let hp = combat.public_full().unwrap().player.hp;
        combat.step_full(&CombatAction::EndTurn).unwrap();
        assert_eq!(combat.public_full().unwrap().player.hp, hp - (3 * (6 + ritual)),
            "later Attacks use the retained per-actor Ritual Strength at A{ascension}");
    }
}

#[test]
fn act3_jaw_worm_horde_enters_with_hard_profile_buffs_and_resolves_selected_openers_at_all_ascensions() {
    use crate::content::EncounterId;
    for ascension in 0..=20 {
        let visit = 120 + u64::from(ascension);
        let mut run_rng = crate::run::RunRng::new([106; 32], rng::Partition::Test, ascension as u64);
        let entry = enemies::setup(EncounterId::JawWormHorde, 3, ascension, &mut run_rng,
            encounter_context(visit, 11_000 + u64::from(ascension))).unwrap();
        assert_eq!(entry.setup.enemies.len(), 3);
        assert!(entry.setup.enemy_damage_bases.is_empty());
        let strength = if ascension >= 17 { 5 } else if ascension >= 2 { 4 } else { 3 };
        let block = if ascension >= 17 { 9 } else { 6 };
        for (slot, actor) in entry.setup.enemies.iter().enumerate() {
            assert_eq!(actor.enemy_id, EnemyId::JawWorm);
            assert_eq!(actor.profile, enemies::EnemyProfile::JawWormHorde);
            assert_eq!(actor.instance_id.as_str(), format!("jaw_worm-{visit}-{slot}"));
            assert_eq!(actor.ascension, ascension);
            assert!(matches!(actor.intent, FullIntentId::Chomp | FullIntentId::Thrash | FullIntentId::Bellow));
            assert_eq!(actor.ai_rng.id().scope()[1], format!("slot-{slot}-jaw_worm-{visit}-{slot}"));
            let role = format!("jaw_worm-{visit}-{slot}");
            let expected_stream = run_rng.derive_combat_stream(Domain::RunCombatEnemyAi, 3,
                EncounterId::JawWormHorde.as_str(), visit, Some(slot as u32), &role).unwrap();
            let (bucket, _) = rng::uniform_below(&expected_stream, 100).unwrap();
            let expected_intent = match bucket {
                0..=24 => FullIntentId::Chomp,
                25..=54 => FullIntentId::Thrash,
                55..=99 => FullIntentId::Bellow,
                _ => unreachable!(),
            };
            assert_eq!(actor.intent, expected_intent,
                "hard Jaw Worm opening uses exact independent 25/30/45 buckets");
        }
        assert_ne!(entry.setup.enemies[0].ai_rng.id(), entry.setup.enemies[1].ai_rng.id());
        assert_ne!(entry.setup.enemies[1].ai_rng.id(), entry.setup.enemies[2].ai_rng.id());

        let mut combat = Combat::enter(entry).unwrap();
        let initial = combat.public_full().unwrap();
        assert_eq!(initial.enemies.len(), 3);
        assert!(initial.enemies.iter().all(|enemy| enemy.block == block),
            "all three hard Jaw Worms receive prebattle Bellow Block before ready");
        assert!(initial.enemies.iter().all(|enemy| enemy.powers.iter().any(|power|
            power.id == FullPowerId("strength") && power.amount == strength)),
            "all three hard Jaw Worms receive prebattle Bellow Strength before ready");
        let opening_damage = initial.enemies.iter().map(|enemy| enemy.intent_damage.unwrap_or(0)).sum::<i32>();
        combat.step_full(&CombatAction::EndTurn).unwrap();
        assert_eq!(initial.player.hp - combat.public_full().unwrap().player.hp, opening_damage,
            "selected hard-profile opening intents execute against the same roster");
        assert!(combat.state.enemies.iter().all(|actor| actor.move_history.len() == 2),
            "each original actor takes one real action; no phantom prebattle turn is recorded");
    }
}

#[test]
fn multi_act_enemy_setup_rejects_wrong_act_invalid_ascension_and_invalid_context_without_rng_draws() {
    use crate::content::EncounterId;
    let mut cases = vec![
        (EncounterId::ThreeCultists, 1, 0, encounter_context(150, 12_001)),
        (EncounterId::JawWormHorde, 3, 21, encounter_context(151, 12_002)),
        (EncounterId::ThreeCultists, 2, 0, encounter_context(152, 12_003)),
    ];
    cases[2].3.cards.clear();
    for (encounter, act, ascension, context) in cases {
        let mut run_rng = crate::run::RunRng::new([107; 32], rng::Partition::Test, 1);
        let before = run_rng.clone();
        assert!(enemies::setup(encounter, act, ascension, &mut run_rng, context).is_err());
        assert_eq!(run_rng, before, "rejected encounter profile must not consume setup RNG");
    }
}

#[test]
fn gremlin_nob_encounter_builds_a_real_solo_act1_elite_before_any_new_species_wiring() {
    use crate::content::EncounterId;
    for ascension in 0..=20 {
        let mut run_rng = crate::run::RunRng::new([108; 32], rng::Partition::Test, ascension as u64);
        let entry = enemies::setup(EncounterId::GremlinNob, 1, ascension, &mut run_rng,
            encounter_context(180 + u64::from(ascension), 12_180 + u64::from(ascension)))
            .expect("ordinary Gremlin Nob setup must be implemented");

        assert_eq!(entry.setup.enemies.len(), 1);
        let nob = &entry.setup.enemies[0];
        assert_eq!(nob.enemy_id, EnemyId::GremlinNob);
        assert_eq!(nob.ascension, ascension);
        assert!((if ascension >= 8 { 85..=90 } else { 82..=86 }).contains(&nob.hp));
        assert_eq!(nob.hp, nob.max_hp);
        assert_eq!(nob.intent.as_str(), "bellow");
        assert!(nob.powers.iter().all(|power| power.id != FullPowerId("enrage")),
            "Nob is not enraged before its initial Bellow resolves");
        assert!(entry.setup.enemy_damage_bases.is_empty());

        let mut combat = Combat::enter(entry).unwrap();
        assert_eq!(combat.public_full().unwrap().enemies[0].intent.as_str(), "bellow");
        assert_eq!(combat.public_full().unwrap().enemies[0].intent_damage, None);
        combat.step_full(&CombatAction::EndTurn).unwrap();
        let enrage = if ascension >= 18 { 3 } else { 2 };
        assert!(combat.state.enemies[0].full_powers.iter().any(|power|
            power.id == FullPowerId("enrage") && power.state.amount == enrage),
            "actual initial Bellow applies Enrage at A{ascension}");
        if ascension >= 18 {
            assert_eq!(combat.state.enemies[0].intent_id.as_str(), "skull_bash");
        } else {
            assert!(matches!(combat.state.enemies[0].intent_id.as_str(), "skull_bash" | "bull_rush"));
        }
    }
}

#[test]
fn gremlin_nob_a18_move_sequence_is_history_driven_and_a20_enrage_uses_live_skill_card_type() {
    use crate::content::{CardId, EncounterId};
    let memory = EnemyMemory::GremlinNob { ascension: 18 };
    let ai = ai_stream("nob-a18-sequence");
    let mut history = vec![];
    for expected in ["bellow", "skull_bash", "bull_rush", "bull_rush", "skull_bash", "bull_rush"] {
        let selected = enemies::select_move(EnemyId::GremlinNob, &history, &memory, &ai).unwrap();
        assert_eq!(selected.intent.as_str(), expected);
        history.push(selected.intent);
    }

    let mut run_rng = crate::run::RunRng::new([109; 32], rng::Partition::Test, 20);
    let mut player = super::support::base_scenario().player;
    player.max_energy = 5;
    let context = enemies::SetupContext { player,
        player_powers: vec![], character: crate::content::Character::Ironclad,
        cards: vec![live_card(CardId::Defend, CardInstanceId(12_201), 0),
            live_card(CardId::Strike, CardInstanceId(12_202), 0),
            super::super::card_state::generated(CardId::Slimed, CardInstanceId(12_203), 0).unwrap(),
            live_card(CardId::Defend, CardInstanceId(12_204), 0),
            live_card(CardId::Defend, CardInstanceId(12_205), 0)],
        opening_draw: 5, encounter_visit: 201 };
    let entry = enemies::setup(EncounterId::GremlinNob, 1, 20, &mut run_rng, context).unwrap();
    let mut combat = Combat::enter(entry).unwrap();
    let opening_defend = combat.state.piles.hand.iter().find_map(|owned|
        owned.full().filter(|card| card.id == CardId::Defend).map(|card| card.instance.0)).unwrap();
    combat.step_full(&CombatAction::PlayCard { instance: opening_defend, target: None }).unwrap();
    assert!(combat.state.enemies[0].full_powers.iter().all(|power| power.id != FullPowerId("strength")),
        "a Skill before Bellow creates Enrage does not grant Strength");
    combat.step_full(&CombatAction::EndTurn).unwrap();
    assert_eq!(combat.state.enemies[0].full_powers.iter().find(|power|
        power.id == FullPowerId("enrage")).unwrap().state.amount, 3);

    let defend = combat.state.piles.hand.iter().find_map(|owned|
        owned.full().filter(|card| card.id == CardId::Defend).map(|card| card.instance.0)).unwrap();
    combat.state.player.energy = 0;
    assert!(combat.step_full(&CombatAction::PlayCard { instance: defend, target: None }).is_err());
    assert!(combat.state.enemies[0].full_powers.iter().all(|power| power.id != FullPowerId("strength")),
        "failed payment does not dispatch CardPlayed or Enrage");

    combat.state.player.energy = 5;
    combat.step_full(&CombatAction::PlayCard { instance: defend, target: None }).unwrap();
    assert_eq!(combat.state.enemies[0].full_powers.iter().find(|power|
        power.id == FullPowerId("strength")).unwrap().state.amount, 3,
        "the real in-play Skill invokes Enrage once");

    let second_defend = combat.state.piles.hand.iter().find_map(|owned|
        owned.full().filter(|card| card.id == CardId::Defend).map(|card| card.instance.0)).unwrap();
    combat.step_full(&CombatAction::PlayCard { instance: second_defend, target: None }).unwrap();
    assert_eq!(combat.state.enemies[0].full_powers.iter().find(|power|
        power.id == FullPowerId("strength")).unwrap().state.amount, 6,
        "each accepted Skill invokes Enrage once");

    let strike = combat.state.piles.hand.iter().find_map(|owned|
        owned.full().filter(|card| card.id == CardId::Strike).map(|card| card.instance.0)).unwrap();
    combat.step_full(&CombatAction::PlayCard { instance: strike, target: Some(TargetId(1)) }).unwrap();
    assert_eq!(combat.state.enemies[0].full_powers.iter().find(|power|
        power.id == FullPowerId("strength")).unwrap().state.amount, 6,
        "Attack does not trigger Enrage");

    let slimed = combat.state.piles.hand.iter().find_map(|owned|
        owned.full().filter(|card| card.id == CardId::Slimed).map(|card| card.instance.0)).unwrap();
    combat.step_full(&CombatAction::PlayCard { instance: slimed, target: None }).unwrap();
    assert_eq!(combat.state.enemies[0].full_powers.iter().find(|power|
        power.id == FullPowerId("strength")).unwrap().state.amount, 6,
        "Status does not trigger Enrage");
    let hp = combat.public_full().unwrap().player.hp;
    assert_eq!(combat.public_full().unwrap().player.block, 10,
        "both accepted Defend Skills contribute their real Block before Skull Bash");
    assert_eq!(combat.public_full().unwrap().enemies[0].intent_damage, Some(14),
        "A20 Skull Bash raw damage is base eight plus six Strength before player Block");
    combat.step_full(&CombatAction::EndTurn).unwrap();
    assert_eq!(combat.public_full().unwrap().player.hp, hp - 4,
        "Skull Bash raw fourteen is absorbed by ten real Block, losing four HP");
    assert_eq!(combat.public_full().unwrap().player.block, 0,
        "Skull Bash consumes the player's remaining Block");
    assert!(combat.state.player_full_powers.iter().any(|power|
        power.id == FullPowerId("vulnerable") && power.state.amount == 2),
        "Skull Bash applies its player Vulnerable after the attack");
}

#[test]
fn gremlin_nob_ascension_boundaries_control_hp_damage_and_enrage_without_a0_fallback() {
    use crate::content::EncounterId;
    for (ascension, hp_range, bash, rush, enrage) in [
        (0, 82..=86, 6, 14, 2), (2, 82..=86, 6, 14, 2),
        (3, 82..=86, 8, 16, 2), (7, 82..=86, 8, 16, 2),
        (8, 85..=90, 8, 16, 2), (17, 85..=90, 8, 16, 2),
        (18, 85..=90, 8, 16, 3), (20, 85..=90, 8, 16, 3),
    ] {
        let mut rng = crate::run::RunRng::new([110; 32], rng::Partition::Test, ascension as u64);
        let entry = enemies::setup(EncounterId::GremlinNob, 1, ascension, &mut rng,
            encounter_context(220 + u64::from(ascension), 12_220 + u64::from(ascension))).unwrap();
        assert!(hp_range.contains(&entry.setup.enemies[0].hp));
        let bases = enemies::damage_bases_at_ascension(EnemyId::GremlinNob, ascension).unwrap();
        assert_eq!(bases.get(&FullIntentId::from_name("skull_bash")).unwrap(), &bash);
        assert_eq!(bases.get(&FullIntentId::from_name("bull_rush")).unwrap(), &rush);
        let mut combat = Combat::enter(entry).unwrap();
        combat.step_full(&CombatAction::EndTurn).unwrap();
        assert_eq!(combat.state.enemies[0].full_powers.iter().find(|power|
            power.id == FullPowerId("enrage")).unwrap().state.amount, enrage);
    }

    for (encounter, act, ascension) in [
        (EncounterId::GremlinNob, 2, 0), (EncounterId::GremlinNob, 1, 21),
    ] {
        let mut rng = crate::run::RunRng::new([111; 32], rng::Partition::Test, 0);
        let before = rng.clone();
        assert!(enemies::setup(encounter, act, ascension, &mut rng,
            encounter_context(250, 12_250)).is_err());
        assert_eq!(rng, before);
    }

    let memory = EnemyMemory::GremlinNob { ascension: 0 };
    let stream = ai_stream("nob-two-rush-forced-bash");
    let forced = enemies::select_move(EnemyId::GremlinNob,
        &[FullIntentId::from_name("bellow"), FullIntentId::from_name("bull_rush"),
            FullIntentId::from_name("bull_rush")], &memory, &stream).unwrap();
    assert_eq!(forced.intent.as_str(), "skull_bash");
    assert_eq!(forced.stream, stream, "forced second Bull Rush redirect does not draw");
    assert!(enemies::select_move(EnemyId::GremlinNob,
        &[FullIntentId::from_name("skull_bash")], &memory, &stream).is_err(),
        "a history without initial Bellow is rejected");
}

#[test]
fn jaw_horde_opening_profile_preserves_later_conditional_restrictions_and_act1_still_requires_chomp_first() {
    let hard = EnemyMemory::JawWorm { ascension: 20, variant: enemies::JawWormVariant::HordeHard };
    let after_bellow = enemies::select_move(EnemyId::JawWorm, &[FullIntentId::Bellow], &hard,
        &ai_stream("hard-jaw-bellow-history")).unwrap();
    assert!(matches!(after_bellow.intent, FullIntentId::Chomp | FullIntentId::Thrash),
        "hard Jaw Bellow repeat restriction remains active after its randomized opener");

    let after_two_thrashes = enemies::select_move(EnemyId::JawWorm,
        &[FullIntentId::Thrash, FullIntentId::Thrash], &hard,
        &ai_stream("hard-jaw-thrash-history")).unwrap();
    assert!(matches!(after_two_thrashes.intent, FullIntentId::Chomp | FullIntentId::Bellow),
        "hard Jaw cannot select a third consecutive Thrash");

    let act1 = EnemyMemory::JawWorm { ascension: 20, variant: enemies::JawWormVariant::Act1 };
    assert!(enemies::select_move(EnemyId::JawWorm, &[FullIntentId::Thrash], &act1,
        &ai_stream("act1-jaw-invalid-first-history")).is_err(),
        "Act1 Jaw history retains its forced initial Chomp contract");
}

#[test]
fn cultist_incantation_is_a_real_queued_ritual_effect_then_attack() {
    let cultist = enemy(EnemyId::Cultist, FullIntentId::Incantation, 50);
    let work = enemies::move_work(&cultist).unwrap();
    assert_eq!(work, vec![Effect { source: Source::Actor(Actor::Enemy(cultist.instance_id.clone())),
        target: Some(Actor::Enemy(cultist.instance_id.clone())),
        kind: EffectKind::ApplyFullPower { id: FullPowerId("ritual"), amount: 3 } }]);

    let history = vec![FullIntentId::Incantation];
    let first = enemies::select_move(EnemyId::Cultist, &history, &EnemyMemory::Cultist { ascension: 0 },
        &ai_stream("actor-1")).unwrap();
    assert_eq!(first.intent, FullIntentId::Attack);
    assert_eq!(first.stream.word_index(), 0, "deterministic Cultist continuation draws no AI word");

    let attack = enemy(EnemyId::Cultist, FullIntentId::Attack, 50);
    let work = enemies::move_work(&attack).unwrap();
    assert_eq!(work, vec![Effect { source: Source::Actor(Actor::Enemy(attack.instance_id.clone())),
        target: Some(Actor::Player), kind: EffectKind::Damage(DamagePayload::FullEnemyMove(FullIntentId::Attack)) }]);
}

#[test]
fn jaw_worm_has_distinct_chomp_damage_and_actor_owned_conditional_ai_draws() {
    let jaw = enemy(EnemyId::JawWorm, FullIntentId::Thrash, 42);
    let work = enemies::move_work(&jaw).unwrap();
    assert_eq!(work, vec![
        Effect { source: Source::Actor(Actor::Enemy(jaw.instance_id.clone())), target: Some(Actor::Player),
            kind: EffectKind::Damage(DamagePayload::FullEnemyMove(FullIntentId::Thrash)) },
        Effect { source: Source::Actor(Actor::Enemy(jaw.instance_id.clone())), target: Some(Actor::Enemy(jaw.instance_id.clone())),
            kind: EffectKind::GainBlock(5) },
    ]);

    let history = vec![FullIntentId::Chomp];
    let first = enemies::select_move(EnemyId::JawWorm, &history, &EnemyMemory::JawWorm { ascension: 0, variant: enemies::JawWormVariant::Act1 },
        &ai_stream("actor-1")).unwrap();
    let second = enemies::select_move(EnemyId::JawWorm, &history, &EnemyMemory::JawWorm { ascension: 0, variant: enemies::JawWormVariant::Act1 },
        &ai_stream("actor-2")).unwrap();
    assert!(matches!(first.intent, FullIntentId::Thrash | FullIntentId::Bellow));
    assert_eq!(first.stream.id().scope()[1], "actor-1");
    assert_eq!(second.stream.id().scope()[1], "actor-2");
    assert_eq!(first.memory, EnemyMemory::JawWorm { ascension: 0, variant: enemies::JawWormVariant::Act1 });

    let mut combat = jaw_combat();
    let hp = combat.public_full().unwrap().player.hp;
    combat.step_full(&CombatAction::EndTurn).unwrap();
    assert_eq!(combat.public_full().unwrap().player.hp, hp - 11,
        "the shared queue resolves Jaw Worm Chomp damage, not Cultist's six-damage attack");
    assert!(matches!(&combat.state.enemies[0].intent_id,
        ActorIntentId::Full(FullIntentId::Thrash | FullIntentId::Bellow)));
    let public = combat.public_full().unwrap();
    assert_eq!(public.enemies[0].intent, match &combat.state.enemies[0].intent_id {
        ActorIntentId::Full(intent) => *intent, _ => unreachable!(),
    });
    assert_eq!(public.enemies[0].intent_damage, combat.state.enemies[0].intent_damage);
    assert!(format!("{public:?}").find("move_history").is_none(), "private AI history is not public");
}

#[test]
fn cultist_turn_resolves_ritual_then_attacks_before_strengthening() {
    let mut combat = super::full_runtime::entered_combat(3, 3, |_| {});
    combat.step_full(&CombatAction::EndTurn).unwrap();
    let cultist = &combat.state.enemies[0];
    assert!(cultist.full_powers.iter().any(|power|
        power.id == FullPowerId("ritual") && power.state.amount == 3 && !power.state.skip_first));
    assert_eq!(cultist.intent_id, ActorIntentId::Full(FullIntentId::Attack));
    assert_eq!(combat.public_full().unwrap().enemies[0].intent, FullIntentId::Attack);

    let hp = combat.state.player.hp;
    combat.step_full(&CombatAction::EndTurn).unwrap();
    assert_eq!(combat.state.player.hp, hp - 6,
        "the queued Attack resolves at six before the turn-end Ritual gain applies");
    assert!(combat.state.enemies[0].full_powers.iter().any(|power|
        power.id == FullPowerId("strength") && power.state.amount == 3));
    assert_eq!(combat.state.enemies[0].full_damage_entries[&FullIntentId::Attack].output, 9);
}

#[test]
fn two_louse_setup_builds_independent_red_and_green_actors_and_publishes_real_moves() {
    let player = super::support::base_scenario().player;
    let mut rng = crate::run::RunRng::new([31; 32], rng::Partition::Test, 11);
    let entry = enemies::setup(crate::content::EncounterId::TwoLouse, 1, 0, &mut rng,
        enemies::SetupContext { player, player_powers: vec![], character: crate::content::Character::Silent, cards: vec![
            live_card(crate::content::CardId::Strike, CardInstanceId(601), 0),
            live_card(crate::content::CardId::Defend, CardInstanceId(602), 0),
            live_card(crate::content::CardId::Bash, CardInstanceId(603), 0),
        ], opening_draw: 3, encounter_visit: 8 }).unwrap();
    assert_eq!(entry.setup.character, crate::content::Character::Silent);
    assert_eq!(entry.setup.card_rng.id().domain(), Domain::RunCombatCardRandom);
    assert_eq!(entry.setup.card_rng.id().scope().len(), 1);
    assert_eq!(entry.setup.card_rng.id().scope()[0], entry.setup.shuffle_rng.id().scope()[0]);
    assert_eq!(entry.setup.enemies.len(), 2);
    assert!(matches!(entry.setup.enemies[0].enemy_id, EnemyId::LouseRed | EnemyId::LouseGreen));
    assert!(matches!(entry.setup.enemies[1].enemy_id, EnemyId::LouseRed | EnemyId::LouseGreen));
    for (slot, louse) in entry.setup.enemies.iter().enumerate() {
        let (min, max) = if louse.enemy_id == EnemyId::LouseRed { (10, 15) } else { (11, 17) };
        assert!((min..=max).contains(&louse.hp));
        assert_eq!(louse.max_hp, louse.hp);
        assert_eq!(louse.intent, FullIntentId::Bite);
        assert!(louse.ai_rng.id().scope()[1].starts_with(&format!("slot-{slot}-")));
        assert!(louse.powers.iter().any(|power|
            power.id == FullPowerId("curl_up") && (3..=7).contains(&power.state.amount)));
    }

    let mut combat = Combat::enter(entry).unwrap();
    assert!(combat.state.enemies.iter().all(|louse|
        (5..=7).contains(&louse.full_damage_entries[&FullIntentId::Bite].base)));
    combat.step_full(&CombatAction::EndTurn).unwrap();
    assert!(combat.public_full().unwrap().enemies.iter().all(|enemy| matches!(
        enemy.intent, FullIntentId::Bite | FullIntentId::Grow | FullIntentId::SpitWeb)));
    assert_eq!(combat.public_full().unwrap().enemies.len(), 2);
    for (public, private) in combat.public_full().unwrap().enemies.iter().zip(&combat.state.enemies) {
        assert_eq!(public.intent, match &private.intent_id {
            ActorIntentId::Full(intent) => *intent, _ => unreachable!(),
        });
    }
}

#[test]
fn louse_ai_redirects_third_repeat_and_species_payloads_differ() {
    let red = enemies::EnemyMemory::for_enemy(EnemyId::LouseRed).unwrap();
    let green = enemies::EnemyMemory::for_enemy(EnemyId::LouseGreen).unwrap();
    let stream = ai_stream("red-louse");
    let red_repeat = enemies::select_move(EnemyId::LouseRed,
        &[FullIntentId::Bite, FullIntentId::Grow, FullIntentId::Grow], &red, &stream).unwrap();
    assert_eq!(red_repeat.intent, FullIntentId::Bite);
    assert_eq!(red_repeat.stream, stream, "forced third-repeat redirect consumes no RNG");
    let green_repeat = enemies::select_move(EnemyId::LouseGreen,
        &[FullIntentId::Bite, FullIntentId::SpitWeb, FullIntentId::SpitWeb], &green, &stream).unwrap();
    assert_eq!(green_repeat.intent, FullIntentId::Bite);

    let red_grow = enemy(EnemyId::LouseRed, FullIntentId::Grow, 12);
    assert_eq!(enemies::move_work(&red_grow).unwrap(), vec![Effect {
        source: Source::Actor(Actor::Enemy(red_grow.instance_id.clone())),
        target: Some(Actor::Enemy(red_grow.instance_id.clone())),
        kind: EffectKind::ApplyFullPower { id: FullPowerId("strength"), amount: 3 },
    }]);
    let green_web = enemy(EnemyId::LouseGreen, FullIntentId::SpitWeb, 12);
    assert_eq!(enemies::move_work(&green_web).unwrap(), vec![Effect {
        source: Source::Actor(Actor::Enemy(green_web.instance_id.clone())),
        target: Some(Actor::Player),
        kind: EffectKind::ApplyFullPower { id: FullPowerId("weak"), amount: 2 },
    }]);
}

#[test]
fn curl_up_is_queued_once_only_for_nonlethal_owner_attributed_normal_damage() {
    let mut combat = two_louse_combat();
    let target = combat.state.enemies[0].instance_id.clone();
    let curl_up = combat.state.enemies[0].full_powers.iter()
        .find(|power| power.id == FullPowerId("curl_up")).unwrap().state.amount;
    let hp_before = combat.state.enemies[0].hp;
    let owner = Actor::Player;
    combat.state.pending.push_back(Effect {
        source: Source::Actor(owner.clone()), target: Some(Actor::Enemy(target.clone())),
        kind: EffectKind::Damage(DamagePayload::Fixed(1)),
    });
    super::super::engine::resolve(&mut combat.state);
    let louse = combat.state.enemies.iter().find(|enemy| enemy.instance_id == target).unwrap();
    let hp_after_first_hit = louse.hp;
    assert_eq!(hp_after_first_hit, hp_before - 1, "the first normal hit deals its one HP damage");
    assert_eq!(louse.block, curl_up, "Curl Up queues its rolled Block after a qualifying hit");
    assert!(!louse.full_powers.iter().any(|power| power.id == FullPowerId("curl_up")),
        "one qualifying damage consumes Curl Up");
    combat.state.pending.push_back(Effect {
        source: Source::Actor(owner), target: Some(Actor::Enemy(target.clone())),
        kind: EffectKind::Damage(DamagePayload::Fixed(1)),
    });
    super::super::engine::resolve(&mut combat.state);
    let louse = combat.state.enemies.iter().find(|enemy| enemy.instance_id == target).unwrap();
    assert_eq!(louse.block, curl_up - 1, "the second, separately resolved hit consumes one remaining Block");
    assert_eq!(louse.hp, hp_after_first_hit, "the second hit is fully absorbed by the remaining Curl Up Block");
    assert!(!louse.full_powers.iter().any(|power| power.id == FullPowerId("curl_up")));
    assert!(matches!(louse.full_memory, Some(EnemyMemory::LouseRed { curl_up_used: true }) |
        Some(EnemyMemory::LouseGreen { curl_up_used: true })));
}

#[test]
fn curl_up_does_not_trigger_for_orb_damage() {
    let mut combat = two_louse_combat();
    let target = combat.state.enemies[0].instance_id.clone();
    let hp_before = combat.state.enemies[0].hp;
    combat.state.pending.push_back(Effect {
        source: Source::Actor(Actor::Player), target: Some(Actor::Enemy(target.clone())),
        kind: EffectKind::Damage(DamagePayload::Orb(1)),
    });
    super::super::engine::resolve(&mut combat.state);
    let louse = combat.state.enemies.iter().find(|enemy| enemy.instance_id == target).unwrap();
    assert_eq!(louse.hp, hp_before - 1);
    assert_eq!(louse.block, 0, "Orb damage is not normal damage and must not trigger Curl Up");
    assert!(louse.full_powers.iter().any(|power| power.id == FullPowerId("curl_up")));
    assert!(matches!(louse.full_memory, Some(EnemyMemory::LouseRed { curl_up_used: false }) |
        Some(EnemyMemory::LouseGreen { curl_up_used: false })));
}

#[test]
fn curl_up_excludes_preblocked_lethal_and_unowned_damage() {
    let mut preblocked = two_louse_combat();
    let target = preblocked.state.enemies[0].instance_id.clone();
    preblocked.state.enemies[0].block = 2;
    preblocked.state.pending.push_back(Effect {
        source: Source::Actor(Actor::Player), target: Some(Actor::Enemy(target.clone())),
        kind: EffectKind::Damage(DamagePayload::Fixed(1)),
    });
    super::super::engine::resolve(&mut preblocked.state);
    let louse = preblocked.state.enemies.iter().find(|enemy| enemy.instance_id == target).unwrap();
    assert_eq!(louse.block, 1);
    assert_eq!(louse.hp, louse.max_hp);
    assert!(louse.full_powers.iter().any(|power| power.id == FullPowerId("curl_up")));

    let mut lethal = two_louse_combat();
    let target = lethal.state.enemies[0].instance_id.clone();
    lethal.state.enemies[0].hp = 1;
    lethal.state.pending.push_back(Effect {
        source: Source::Actor(Actor::Player), target: Some(Actor::Enemy(target.clone())),
        kind: EffectKind::Damage(DamagePayload::Fixed(1)),
    });
    super::super::engine::resolve(&mut lethal.state);
    let louse = lethal.state.enemies.iter().find(|enemy| enemy.instance_id == target).unwrap();
    assert_eq!(louse.hp, 0);
    assert!(louse.full_powers.iter().any(|power| power.id == FullPowerId("curl_up")));

    let mut unowned = two_louse_combat();
    let target = unowned.state.enemies[0].instance_id.clone();
    let hp_before = unowned.state.enemies[0].hp;
    unowned.state.pending.push_back(Effect {
        source: Source::None, target: Some(Actor::Enemy(target.clone())),
        kind: EffectKind::Damage(DamagePayload::Fixed(1)),
    });
    super::super::engine::resolve(&mut unowned.state);
    let louse = unowned.state.enemies.iter().find(|enemy| enemy.instance_id == target).unwrap();
    assert_eq!(louse.hp, hp_before - 1);
    assert_eq!(louse.block, 0);
    assert!(louse.full_powers.iter().any(|power| power.id == FullPowerId("curl_up")));
}

fn two_louse_combat() -> Combat {
    let player = super::support::base_scenario().player;
    let mut rng = crate::run::RunRng::new([41; 32], rng::Partition::Test, 12);
    Combat::enter(enemies::setup(crate::content::EncounterId::TwoLouse, 1, 0, &mut rng,
        enemies::SetupContext { player, player_powers: vec![], character: crate::content::Character::Defect, cards: vec![
            live_card(crate::content::CardId::Strike, CardInstanceId(701), 0),
            live_card(crate::content::CardId::Defend, CardInstanceId(702), 0),
            live_card(crate::content::CardId::Bash, CardInstanceId(703), 0),
        ], opening_draw: 3, encounter_visit: 9 }).unwrap()).unwrap()
}

#[test]
fn small_slimes_setup_is_one_of_the_qualified_ordered_pair_formations() {
    let player = super::support::base_scenario().player;
    let mut rng = crate::run::RunRng::new([53; 32], rng::Partition::Test, 17);
    let entry = enemies::setup(crate::content::EncounterId::SmallSlimes, 1, 0, &mut rng,
        enemies::SetupContext { player, player_powers: vec![], character: crate::content::Character::Ironclad,
            cards: vec![live_card(crate::content::CardId::Strike, CardInstanceId(801), 0)],
            opening_draw: 1, encounter_visit: 2 }).unwrap();
    let pair = entry.setup.enemies.iter().map(|enemy| enemy.enemy_id).collect::<Vec<_>>();
    assert!(matches!(pair.as_slice(),
        [EnemyId::SpikeSlimeS, EnemyId::AcidSlimeM] |
        [EnemyId::AcidSlimeS, EnemyId::SpikeSlimeM]));
    for enemy in &entry.setup.enemies {
        let hp = match enemy.enemy_id {
            EnemyId::AcidSlimeM | EnemyId::SpikeSlimeM => 28..=32,
            EnemyId::AcidSlimeS => 8..=12,
            EnemyId::SpikeSlimeS => 10..=14,
            _ => panic!("Small Slimes contains only the qualified small/medium actors"),
        };
        assert!(hp.contains(&enemy.hp));
        assert_eq!(enemy.max_hp, enemy.hp);
        match enemy.enemy_id {
            EnemyId::AcidSlimeM => assert!(matches!(enemy.intent,
                FullIntentId::AcidSpit | FullIntentId::AcidTackle | FullIntentId::AcidLick)),
            EnemyId::SpikeSlimeM => assert!(matches!(enemy.intent,
                FullIntentId::SpikeFlameTackle | FullIntentId::SpikeLick)),
            EnemyId::AcidSlimeS => assert!(matches!(enemy.intent, FullIntentId::AcidTackle | FullIntentId::AcidLick)),
            EnemyId::SpikeSlimeS => assert_eq!(enemy.intent, FullIntentId::SpikeTackle),
            _ => (),
        }
    }
    assert_ne!(entry.setup.enemies[0].instance_id, entry.setup.enemies[1].instance_id);
    assert_ne!(entry.setup.enemies[0].ai_rng.id(), entry.setup.enemies[1].ai_rng.id());
}

#[test]
fn medium_acid_spit_orders_damage_before_one_player_owned_slimed_request() {
    let acid = enemy(EnemyId::AcidSlimeM, FullIntentId::AcidSpit, 30);
    let work = enemies::move_work(&acid).unwrap();
    assert_eq!(work, vec![
        Effect { source: Source::Actor(Actor::Enemy(acid.instance_id.clone())),
            target: Some(Actor::Player),
            kind: EffectKind::Damage(DamagePayload::FullEnemyMove(FullIntentId::AcidSpit)) },
        Effect { source: Source::Actor(Actor::Enemy(acid.instance_id.clone())),
            target: Some(Actor::Player),
            kind: EffectKind::CreateCard { id: crate::content::CardId::Slimed,
                destination: crate::combat::GeneratedCardDestination::Discard, upgrades: 0 } },
    ]);
}

#[test]
fn large_acid_and_spike_attacks_request_two_slimed_cards_after_damage() {
    for (id, intent, damage) in [
        (EnemyId::AcidSlimeL, FullIntentId::AcidSpit, 11),
        (EnemyId::SpikeSlimeL, FullIntentId::SpikeFlameTackle, 16),
    ] {
        let actor = enemy(id, intent, 66);
        let effects = enemies::move_work(&actor).unwrap();
        assert_eq!(effects.len(), 3);
        assert_eq!(effects[0].kind, EffectKind::Damage(DamagePayload::FullEnemyMove(intent)));
        assert!(effects[1..].iter().all(|effect| effect.target == Some(Actor::Player)
            && matches!(effect.kind, EffectKind::CreateCard { id: crate::content::CardId::Slimed,
                destination: crate::combat::GeneratedCardDestination::Discard, upgrades: 0 })));
        assert_eq!(enemies::damage_bases(id).unwrap()[&intent], damage);
    }
}

#[test]
fn full_damage_path_accepts_qualified_base_sixteen_without_widening_legacy_helper() {
    assert_eq!(super::super::effects::refreshed_damage(16, &[], &[], &[], &[], false, true, false).unwrap(), 16);
    assert!(super::super::rules::normal_damage(16, 0, false).is_err());
    assert!(super::super::rules::normal_damage_modifiers(16, 0, false, false, false).is_err());
}

#[test]
fn canonical_species_damage_bases_preserve_legacy_values_and_size_specific_slime_values() {
    for (id, expected) in [
        (EnemyId::Cultist, vec![(FullIntentId::Attack, 6)]),
        (EnemyId::JawWorm, vec![(FullIntentId::Chomp, 11), (FullIntentId::Thrash, 7)]),
        (EnemyId::AcidSlimeS, vec![(FullIntentId::AcidTackle, 3)]),
        (EnemyId::AcidSlimeM, vec![(FullIntentId::AcidSpit, 7), (FullIntentId::AcidTackle, 10)]),
        (EnemyId::AcidSlimeL, vec![(FullIntentId::AcidSpit, 11), (FullIntentId::AcidTackle, 16)]),
        (EnemyId::SpikeSlimeS, vec![(FullIntentId::SpikeTackle, 5)]),
        (EnemyId::SpikeSlimeM, vec![(FullIntentId::SpikeFlameTackle, 8)]),
        (EnemyId::SpikeSlimeL, vec![(FullIntentId::SpikeFlameTackle, 16)]),
    ] {
        let expected = expected.into_iter().collect::<std::collections::BTreeMap<_, _>>();
        assert_eq!(enemies::damage_bases(id).unwrap(), expected);
    }
}

#[test]
fn typed_slime_intents_round_trip_their_canonical_names() {
    for (id, name) in [
        (FullIntentId::AcidTackle, "acid_tackle"),
        (FullIntentId::AcidSpit, "acid_spit"),
        (FullIntentId::AcidLick, "acid_lick"),
        (FullIntentId::AcidSplit, "acid_split"),
        (FullIntentId::SpikeTackle, "spike_tackle"),
        (FullIntentId::SpikeFlameTackle, "spike_flame_tackle"),
        (FullIntentId::SpikeLick, "spike_lick"),
        (FullIntentId::SpikeSplit, "spike_split"),
    ] {
        assert_eq!(id.as_str(), name);
        assert_eq!(FullIntentId::from_name(name), id);
    }
}

#[test]
fn acid_spit_inserts_a_fresh_slimed_instance_into_discard_after_its_hit() {
    // This declared single-actor combat fixture isolates the qualified medium
    // Acid Spit behavior; it is not presented as a Small Slimes formation.
    let run_rng = crate::run::RunRng::new([59; 32], rng::Partition::Test, 18);
    let ai_rng = run_rng.derive_combat_stream(rng::Domain::RunCombatEnemyAi, 1,
        "small_slimes", 3, Some(0), "acid-slime-m-3-0").unwrap();
    let shuffle_rng = run_rng.derive_combat_stream(rng::Domain::RunCombatShuffle, 1,
        "small_slimes", 3, None, "shuffle").unwrap();
    let card_rng = run_rng.derive_combat_stream(rng::Domain::RunCombatCardRandom, 1,
        "small_slimes", 3, None, "card").unwrap();
    let setup = FullCombatSetup { player: super::support::base_scenario().player,
        player_powers: vec![], character: crate::content::Character::Silent, card_rng,
        shuffle_rng, enemies: vec![FullEnemySetup { initial_memory:None, instance_id: EnemyHandle::parse("acid-slime-m-3-0").unwrap(), ascension: 0,
            profile: enemies::EnemyProfile::Standard,
            enemy_id: EnemyId::AcidSlimeM, hp: 30, max_hp: 30, block: 0,
            intent: FullIntentId::AcidSpit, powers: vec![], ai_rng }], enemy_damage_bases: Default::default() };
    let entry = CombatEntry { setup, opening_draw: 1,
        cards: vec![live_card(crate::content::CardId::Strike, CardInstanceId(851), 0)] };
    let mut combat = Combat::enter(entry).unwrap();
    let player_hp = combat.state.player.hp;
    let acid = &mut combat.state.enemies[0];
    acid.intent_id = ActorIntentId::Full(FullIntentId::AcidSpit);
    acid.selected_move = ActorIntentId::Full(FullIntentId::AcidSpit);
    acid.move_history.push(ActorIntentId::Full(FullIntentId::AcidSpit));
    let acid_handle = acid.instance_id.clone();
    enemies::act(&mut combat.state, &acid_handle).unwrap();
    super::super::engine::resolve(&mut combat.state);
    assert_eq!(combat.state.player.hp, player_hp - 7);
    assert_eq!(combat.state.piles.discard.len(), 1);
    let generated = combat.state.piles.discard[0].full().expect("generated status uses native LiveCard ownership");
    assert_eq!(generated.id, crate::content::CardId::Slimed);
    assert_eq!(generated.upgrades, 0);
    assert!(matches!(generated.origin, crate::combat::CombatCardRef::CombatLocal(_)));
    assert!(combat.state.card_ids_issued.contains(&super::super::card_state::handle(generated.instance)));
}

#[test]
fn medium_spike_selection_redirects_third_repeat_and_publishes_distinct_lick() {
    let stream = ai_stream("spike-medium");
    let memory = EnemyMemory::for_enemy(EnemyId::SpikeSlimeM).unwrap();
    let redirected = enemies::select_move(EnemyId::SpikeSlimeM,
        &[FullIntentId::SpikeFlameTackle, FullIntentId::SpikeFlameTackle], &memory, &stream).unwrap();
    assert_eq!(redirected.intent, FullIntentId::SpikeLick);
    assert_eq!(redirected.stream, stream, "forced repeat redirect consumes no AI draw");

    let acid_memory = EnemyMemory::for_enemy(EnemyId::AcidSlimeM).unwrap();
    let acid_repeat = enemies::select_move(EnemyId::AcidSlimeM,
        &[FullIntentId::AcidSpit, FullIntentId::AcidSpit], &acid_memory, &stream).unwrap();
    assert!(matches!(acid_repeat.intent, FullIntentId::AcidTackle | FullIntentId::AcidLick),
        "two consecutive spits redirect to the qualified fair alternatives");

    let spike = enemy(EnemyId::SpikeSlimeM, FullIntentId::SpikeLick, 30);
    assert_eq!(enemies::move_work(&spike).unwrap(), vec![Effect {
        source: Source::Actor(Actor::Enemy(spike.instance_id.clone())), target: Some(Actor::Player),
        kind: EffectKind::ApplyFullPower { id: FullPowerId("frail"), amount: 1 },
    }]);
}

#[test]
fn large_slime_split_retires_parent_and_appends_children_with_late_hp() {
    let player = super::support::base_scenario().player;
    let mut run_rng = crate::run::RunRng::new([67; 32], rng::Partition::Test, 19);
    let mut entry = enemies::setup(crate::content::EncounterId::LargeSlime, 1, 0, &mut run_rng,
        enemies::SetupContext { player, player_powers: vec![], character: crate::content::Character::Defect,
            cards: vec![live_card(crate::content::CardId::Strike, CardInstanceId(901), 0)],
            opening_draw: 1, encounter_visit: 4 }).unwrap();
    let survivor_rng = run_rng.derive_combat_stream(rng::Domain::RunCombatEnemyAi, 1,
        "large_slime", 4, Some(1), "stable-survivor").unwrap();
    entry.setup.enemies.push(FullEnemySetup { initial_memory:None, instance_id: EnemyHandle::parse("stable-survivor").unwrap(), ascension: 0,
        profile: enemies::EnemyProfile::Standard,
        enemy_id: EnemyId::Cultist, hp: 50, max_hp: 50, block: 0, intent: FullIntentId::Incantation,
        powers: vec![], ai_rng: survivor_rng });
    let mut combat = Combat::enter(entry).unwrap();
    let parent = combat.state.enemies[0].instance_id.clone();
    let survivor_id = combat.state.enemies[1].instance_id.clone();
    let parent_id = match combat.state.enemies[0].enemy_id {
        ActorEnemyId::Full(EnemyId::AcidSlimeL) => EnemyId::AcidSlimeL,
        ActorEnemyId::Full(EnemyId::SpikeSlimeL) => EnemyId::SpikeSlimeL,
        _ => unreachable!("Large Slime encounter contains a large Acid or Spike Slime"),
    };
    let split_intent = if parent_id == EnemyId::AcidSlimeL { FullIntentId::AcidSplit } else { FullIntentId::SpikeSplit };
    let threshold = combat.state.enemies[0].max_hp / 2;
    combat.state.enemies[0].hp = threshold + 2;
    let tackle = if parent_id == EnemyId::AcidSlimeL { FullIntentId::AcidTackle } else { FullIntentId::SpikeFlameTackle };
    combat.state.enemies[0].intent_id = ActorIntentId::Full(tackle);
    combat.state.enemies[0].selected_move = ActorIntentId::Full(tackle);
    combat.state.enemies[0].move_history = vec![ActorIntentId::Full(tackle)];

    combat.state.pending.push_back(Effect { source: Source::Actor(Actor::Player),
        target: Some(Actor::Enemy(parent.clone())), kind: EffectKind::Damage(DamagePayload::Fixed(2)) });
    super::super::engine::resolve(&mut combat.state);
    let parent_state = combat.state.enemies.iter().find(|enemy| enemy.instance_id == parent).unwrap();
    assert_eq!(parent_state.hp, threshold);
    assert_eq!(parent_state.intent_id, ActorIntentId::Full(split_intent),
        "a surviving threshold hit publishes Split immediately");
    assert_eq!(combat.public_full().unwrap().enemies[0].intent, split_intent);
    assert_eq!(combat.state.enemies.len(), 2);
    assert_eq!(combat.state.enemies[1].instance_id, survivor_id);

    // Damage after threshold selection but before the Split turn must be
    // reflected in each child's inherited HP.
    combat.state.pending.push_back(Effect { source: Source::Actor(Actor::Player),
        target: Some(Actor::Enemy(parent.clone())), kind: EffectKind::Damage(DamagePayload::Fixed(4)) });
    super::super::engine::resolve(&mut combat.state);
    assert_eq!(combat.state.enemies[0].hp, threshold - 4);
    let hp_before_split = combat.state.enemies[0].hp;
    let parent_stream_before_turn = combat.state.enemies[0].full_ai_rng.clone();
    let player_hp_before_turn = combat.state.player.hp;
    combat.state.pending.push_back(Effect { source: Source::None, target: None,
        kind: EffectKind::MonsterStart });
    super::super::engine::resolve(&mut combat.state);
    assert!(combat.state.fault.is_none(), "a qualified Split and current enemy phase must resolve");
    assert_eq!(combat.state.enemies.len(), 4, "parent tombstone, survivor, then two appended children");
    assert_eq!(combat.state.enemies[0].instance_id, parent);
    assert_eq!(combat.state.enemies[0].hp, 0, "replacement is not death-credit damage");
    assert_eq!(combat.state.enemies[0].full_ai_rng, parent_stream_before_turn,
        "stale queued parent RollMove is skipped without drawing");
    assert_eq!(combat.state.enemies[1].instance_id, survivor_id);
    let left = &combat.state.enemies[2];
    let right = &combat.state.enemies[3];
    assert_eq!(left.enemy_id, ActorEnemyId::Full(if parent_id == EnemyId::AcidSlimeL {
        EnemyId::AcidSlimeM
    } else { EnemyId::SpikeSlimeM }));
    assert_eq!(right.enemy_id, left.enemy_id);
    assert_eq!((left.hp, left.max_hp), (hp_before_split, hp_before_split));
    assert_eq!((right.hp, right.max_hp), (hp_before_split, hp_before_split));
    assert_ne!(left.instance_id, right.instance_id);
    assert_ne!(left.full_ai_rng, right.full_ai_rng);
    assert_eq!(left.move_history.len(), 1);
    assert_eq!(right.move_history.len(), 1);
    assert!(left.full_powers.is_empty() && right.full_powers.is_empty());
    assert!(left.full_memory.is_some() && right.full_memory.is_some());
    assert_eq!(combat.state.player.hp, player_hp_before_turn,
        "the frozen enemy-phase roster does not grant either child a same-phase attack");
    assert_eq!(combat.state.phase, Phase::Decision,
        "the native EndTurn continuation finishes after skipping the retired parent");
    assert_eq!(combat.public_full().unwrap().targets, vec![
        (TargetId(1), false), (TargetId(2), true), (TargetId(3), true), (TargetId(4), true)]);
    assert_eq!(combat.state.outcome, None);
}

#[test]
fn split_child_stream_is_semantic_and_ignores_parent_cursor() {
    let parent = ai_stream("large-parent");
    let advanced = StreamState::new(parent.id().clone(), *parent.key(), 47).unwrap();
    let left = EnemyHandle::parse("large-parent/split/0").unwrap();
    let right = EnemyHandle::parse("large-parent/split/1").unwrap();
    let left_stream = enemies::split_child_stream(&parent, &left, 0).unwrap();
    assert_eq!(left_stream, enemies::split_child_stream(&advanced, &left, 0).unwrap());
    assert_ne!(left_stream, enemies::split_child_stream(&parent, &right, 1).unwrap());
    assert_eq!(left_stream.id().domain(), Domain::RunCombatEnemyAi);
    assert_eq!(left_stream.id().scope()[0], parent.id().scope()[0]);
    assert_eq!(left_stream.id().scope()[1], left.as_str());
}

#[test]
fn split_child_initialization_fault_retains_staged_child_and_does_not_retry() {
    let player = super::support::base_scenario().player;
    let mut run_rng = crate::run::RunRng::new([79; 32], rng::Partition::Test, 23);
    let mut entry = enemies::setup(crate::content::EncounterId::LargeSlime, 1, 0, &mut run_rng,
        enemies::SetupContext { player, player_powers: vec![], character: crate::content::Character::Defect,
            cards: vec![live_card(crate::content::CardId::Strike, CardInstanceId(931), 0)],
            opening_draw: 1, encounter_visit: 6 }).unwrap();
    let parent = entry.setup.enemies[0].instance_id.clone();
    let colliding_child = EnemyHandle::parse(&format!("{}/split/1", parent.as_str())).unwrap();
    let survivor_rng = run_rng.derive_combat_stream(rng::Domain::RunCombatEnemyAi, 1,
        "large_slime", 6, Some(1), "collision-survivor").unwrap();
    entry.setup.enemies.push(FullEnemySetup { initial_memory:None, instance_id: colliding_child, ascension: 0,
        profile: enemies::EnemyProfile::Standard,
        enemy_id: EnemyId::Cultist, hp: 50, max_hp: 50, block: 0,
        intent: FullIntentId::Incantation, powers: vec![], ai_rng: survivor_rng });
    let mut combat = Combat::enter(entry).unwrap();
    let parent_id = match combat.state.enemies[0].enemy_id {
        ActorEnemyId::Full(id @ (EnemyId::AcidSlimeL | EnemyId::SpikeSlimeL)) => id,
        _ => unreachable!("Large Slime encounter supplies a splitting parent"),
    };
    let split_intent = if parent_id == EnemyId::AcidSlimeL { FullIntentId::AcidSplit } else { FullIntentId::SpikeSplit };
    combat.state.enemies[0].hp = combat.state.enemies[0].max_hp / 2;
    combat.state.enemies[0].intent_id = ActorIntentId::Full(split_intent);
    combat.state.enemies[0].selected_move = ActorIntentId::Full(split_intent);
    combat.state.enemies[0].move_history.push(ActorIntentId::Full(split_intent));
    match combat.state.enemies[0].full_memory.as_mut().unwrap() {
        EnemyMemory::AcidSlime { split_triggered } | EnemyMemory::SpikeSlime { split_triggered } => *split_triggered = true,
        _ => unreachable!(),
    }
    combat.state.pending.push_back(Effect { source: Source::None, target: None,
        kind: EffectKind::MonsterStart });
    super::super::engine::resolve(&mut combat.state);

    let progress = combat.state.replacement_progress.as_ref().unwrap();
    assert_eq!(combat.state.fault.as_ref().unwrap().kind, FaultKind::Invariant);
    assert_eq!(progress.parent, parent);
    assert_eq!(progress.next_child, 1);
    assert_eq!(progress.prepared.len(), 1);
    assert!(progress.latest_child_rng.is_some());
    assert_eq!(combat.state.enemies.len(), 2, "prepared children commit atomically");
    assert_eq!(combat.state.enemies[0].life, super::super::model::ActorLife::Active);
    assert!(matches!(combat.state.current.as_ref().map(|effect| &effect.kind),
        Some(EffectKind::ReplaceEnemy { parent: current_parent, children })
            if current_parent == &parent && children.len() == 2));
    assert!(combat.state.pending.iter().any(|effect| effect.kind == EffectKind::RollMove
        && effect.target == Some(Actor::Enemy(parent.clone()))));

    let before = combat.snapshot();
    let Err(crate::run::RunError::Faulted(packet)) = combat.step_full(&CombatAction::EndTurn) else {
        panic!("faulted replacement must not retry")
    };
    let crate::run::RunFaultKind::Combat(native) = packet.kind else { panic!("native combat diagnostic expected") };
    let diagnostic = native.replacement_progress.unwrap();
    assert_eq!(diagnostic.next_child, 1);
    assert_eq!(diagnostic.prepared_children.len(), 1);
    assert!(diagnostic.latest_child_cursor.is_some());
    assert_eq!(super::super::snapshot::compare(&before, &combat.snapshot()), None);
}

#[test]
fn exhausted_split_child_cursor_is_retained_without_mutating_parent_stream() {
    let mut combat = large_acid_combat();
    let parent = combat.state.enemies[0].instance_id.clone();
    let parent_stream = combat.state.enemies[0].full_ai_rng.clone().unwrap();
    let parent_stream_before = parent_stream.clone();
    let child = EnemyHandle::parse(&format!("{}/split/0", parent.as_str())).unwrap();
    let child_stream = enemies::split_child_stream(&parent_stream, &child, 0).unwrap();
    let latest = StreamState::new(child_stream.id().clone(), *child_stream.key(), 1_u128 << 64).unwrap();
    combat.state.replacement_progress = Some(super::super::enemies::ReplacementProgress {
        parent: parent.clone(), children: vec![EnemyId::AcidSlimeM; 2],
        inherited_hp: combat.state.enemies[0].hp, next_child: 0, prepared: vec![],
        latest_child_rng: Some(child_stream),
    });

    super::super::engine::retain_error(&mut combat.state,
        EffectError::Random(super::super::rng::RngError::Exhausted { state: latest }));

    assert_eq!(combat.state.fault.as_ref().unwrap().kind, FaultKind::RngExhausted);
    assert_eq!(combat.state.replacement_progress.as_ref().unwrap()
        .latest_child_rng.as_ref().unwrap().word_index(), 1_u128 << 64);
    assert_eq!(combat.state.enemies[0].full_ai_rng.as_ref(), Some(&parent_stream_before),
        "a child cursor is never written into the parent actor stream");
}

#[test]
fn slime_boss_real_turn_cycle_deals_goop_slam_and_selects_preparation() {
    let mut combat = Combat::enter(slime_boss_entry(9, [103; 32])).unwrap();
    assert_eq!(combat.state.enemies.len(), 1);
    assert_eq!(combat.state.enemies[0].enemy_id.as_str(), "slime_boss");
    assert_eq!((combat.state.enemies[0].hp, combat.state.enemies[0].max_hp), (140, 140));
    assert_eq!(combat.state.enemies[0].intent_id,
        ActorIntentId::Full(FullIntentId::SlimeBossSticky));
    assert_eq!(combat.public_full().unwrap().enemies[0].intent, FullIntentId::SlimeBossSticky);
    let initial_goop = enemies::move_work(&combat.state.enemies[0]).unwrap();
    assert_eq!(initial_goop.len(), 3);
    assert!(initial_goop.iter().all(|effect| matches!(effect.kind,
        EffectKind::CreateCard { id: crate::content::CardId::Slimed,
            destination: super::super::GeneratedCardDestination::Discard, upgrades: 0 })));
    let boss = combat.state.enemies[0].instance_id.clone();
    let hp = combat.state.player.hp;

    for (turn, expected_intent, expected_hp, expected_goop) in [
        ("goop", FullIntentId::SlimeBossPrepSlam, hp, 3),
        ("prepare", FullIntentId::SlimeBossSlam, hp, 3),
        ("slam", FullIntentId::SlimeBossSticky, hp - 35, 3),
        ("goop", FullIntentId::SlimeBossPrepSlam, hp - 35, 6),
    ] {
        combat.state.pending.push_back(Effect { source: Source::None, target: None,
            kind: EffectKind::MonsterStart });
        super::super::engine::resolve(&mut combat.state);
        assert!(combat.state.fault.is_none(), "{turn} turn resolves without a fault");
        assert_eq!(combat.state.enemies[0].instance_id, boss);
        assert_eq!(combat.state.player.hp, expected_hp, "{turn} payload HP");
        let actual_intent = match combat.state.enemies[0].intent_id {
            ActorIntentId::Full(intent) => intent,
            ActorIntentId::Legacy(_) => panic!("full Slime Boss intent must remain typed"),
        };
        assert_eq!(actual_intent, expected_intent, "{turn} next typed intent");
        assert_eq!(actual_intent.as_str(), expected_intent.as_str(), "{turn} canonical public intent name");
        assert_eq!(FullIntentId::from_name(actual_intent.as_str()), actual_intent,
            "{turn} public intent name round-trips to its typed identity");
        assert_eq!(combat.public_full().unwrap().enemies[0].intent, expected_intent,
            "{turn} public projection matches the selected native intent");
        let count_goop = |pile: &[super::super::model::OwnedCard]| pile.iter().filter(|card|
            card.full().is_some_and(|card| card.id == crate::content::CardId::Slimed)).count();
        let zones = (count_goop(&combat.state.piles.hand), count_goop(&combat.state.piles.draw),
            count_goop(&combat.state.piles.discard), count_goop(&combat.state.piles.exhaust),
            combat.state.in_play.as_ref().is_some_and(|card|
                card.full().is_some_and(|card| card.id == crate::content::CardId::Slimed)) as usize);
        assert_eq!(zones.0 + zones.1 + zones.2 + zones.3 + zones.4, expected_goop,
            "{turn} total owned Slimed across hand/draw/discard/exhaust/in-play; zones={zones:?}");
    }
}

#[test]
fn slime_boss_slam_uses_full_enemy_damage_range_without_changing_legacy_range() {
    assert_eq!(super::super::effects::refreshed_damage(35, &[], &[], &[], &[], false, true, false).unwrap(), 35);
    assert!(super::super::rules::normal_damage_modifiers(35, 0, false, false, false).is_err(),
        "the legacy numeric contract remains unchanged");
}

#[test]
fn slime_boss_threshold_split_inherits_late_hp_and_descendants_can_settle() {
    let mut combat = Combat::enter(slime_boss_entry(10, [107; 32])).unwrap();
    let boss = combat.state.enemies[0].instance_id.clone();
    let player_hp = combat.state.player.hp;
    combat.state.enemies[0].hp = 74;
    combat.state.pending.push_back(Effect { source: Source::Actor(Actor::Player),
        target: Some(Actor::Enemy(boss.clone())), kind: EffectKind::Damage(DamagePayload::Fixed(4)) });
    super::super::engine::resolve(&mut combat.state);
    assert_eq!(combat.state.enemies[0].hp, 70);
    assert_eq!(combat.state.enemies[0].intent_id.as_str(), "split",
        "the nonlethal half-HP threshold immediately interrupts the current intent");

    combat.state.pending.push_back(Effect { source: Source::Actor(Actor::Player),
        target: Some(Actor::Enemy(boss.clone())), kind: EffectKind::Damage(DamagePayload::Fixed(6)) });
    super::super::engine::resolve(&mut combat.state);
    assert_eq!(combat.state.enemies[0].hp, 64);
    combat.state.pending.push_back(Effect { source: Source::None, target: None,
        kind: EffectKind::MonsterStart });
    super::super::engine::resolve(&mut combat.state);
    assert!(combat.state.fault.is_none(), "boss replacement resolves through B1");
    assert_eq!(combat.state.enemies[0].life, super::super::model::ActorLife::Replaced);
    assert_eq!(combat.state.enemies[0].hp, 0);
    assert_eq!(combat.state.enemies[1].enemy_id.as_str(), "spike_slime_l");
    assert_eq!(combat.state.enemies[2].enemy_id.as_str(), "acid_slime_l");
    assert_eq!((combat.state.enemies[1].hp, combat.state.enemies[1].max_hp), (64, 64));
    assert_eq!((combat.state.enemies[2].hp, combat.state.enemies[2].max_hp), (64, 64));
    assert_eq!(combat.state.player.hp, player_hp, "children do not attack during the frozen boss phase");
    assert_eq!(combat.public_full().unwrap().targets, vec![
        (TargetId(1), false), (TargetId(2), true), (TargetId(3), true)]);
    assert_eq!(combat.state.outcome, None, "parent replacement cannot create a victory gap");

    // Both large descendants cross their own actual half-HP threshold, then
    // split on a later frozen roster into four medium descendants.
    let large_handles = [combat.state.enemies[1].instance_id.clone(), combat.state.enemies[2].instance_id.clone()];
    for handle in &large_handles {
        let large = combat.state.enemies.iter_mut().find(|enemy| &enemy.instance_id == handle).unwrap();
        large.hp = 36;
        combat.state.pending.push_back(Effect { source: Source::Actor(Actor::Player),
            target: Some(Actor::Enemy(handle.clone())), kind: EffectKind::Damage(DamagePayload::Fixed(4)) });
    }
    super::super::engine::resolve(&mut combat.state);
    for handle in &large_handles {
        let large = combat.state.enemies.iter().find(|enemy| &enemy.instance_id == handle).unwrap();
        assert_eq!(large.hp, 32);
        let expected_split = match large.enemy_id.as_str() {
            "acid_slime_l" => "acid_split", "spike_slime_l" => "spike_split",
            other => panic!("unexpected large child {other}"),
        };
        assert_eq!(large.intent_id.as_str(), expected_split);
    }
    let hp_before_descendant_splits = combat.state.player.hp;
    combat.state.pending.push_back(Effect { source: Source::None, target: None,
        kind: EffectKind::MonsterStart });
    super::super::engine::resolve(&mut combat.state);
    assert!(combat.state.fault.is_none(), "both large Slime descendants execute their own Split");
    assert_eq!(combat.state.player.hp, hp_before_descendant_splits,
        "newly appended children do not act in the already-frozen phase");
    assert_eq!(combat.state.enemies.len(), 7);
    assert_eq!(combat.state.enemies.iter().filter(|enemy|
        enemy.life == super::super::model::ActorLife::Active && enemy.hp > 0).count(), 4);
    assert_eq!(combat.state.enemies[3].enemy_id.as_str(), "spike_slime_m");
    assert_eq!(combat.state.enemies[4].enemy_id.as_str(), "spike_slime_m");
    assert_eq!(combat.state.enemies[5].enemy_id.as_str(), "acid_slime_m");
    assert_eq!(combat.state.enemies[6].enemy_id.as_str(), "acid_slime_m");
    assert_eq!(combat.state.outcome, None);

    let targets = combat.state.enemies.iter().filter(|enemy| enemy.hp > 0)
        .map(|enemy| enemy.instance_id.clone()).collect::<Vec<_>>();
    for target in targets {
        combat.state.pending.push_back(Effect { source: Source::Actor(Actor::Player),
            target: Some(Actor::Enemy(target)), kind: EffectKind::Damage(DamagePayload::Fixed(32)) });
    }
    combat.state.pending.push_back(Effect { source: Source::None, target: None,
        kind: EffectKind::Decision });
    super::super::engine::resolve(&mut combat.state);
    assert_eq!(combat.state.outcome, Some(super::super::Outcome::Victory),
        "the room settles only after the last active descendant dies");
}

#[test]
fn slime_boss_split_initialization_fault_keeps_first_ordered_child_staged() {
    let player = super::support::base_scenario().player;
    let mut run_rng = crate::run::RunRng::new([109; 32], rng::Partition::Test, 31);
    let mut entry = enemies::setup(crate::content::EncounterId::SlimeBoss, 1, 0, &mut run_rng,
        enemies::SetupContext { player, player_powers: vec![], character: crate::content::Character::Watcher,
            cards: vec![live_card(crate::content::CardId::Strike, CardInstanceId(951), 0)],
            opening_draw: 1, encounter_visit: 11 }).unwrap();
    let parent = entry.setup.enemies[0].instance_id.clone();
    let collision = EnemyHandle::parse(&format!("{}/split/1", parent.as_str())).unwrap();
    let ai_rng = run_rng.derive_combat_stream(rng::Domain::RunCombatEnemyAi, 1,
        "slime_boss", 11, Some(1), "collision").unwrap();
    entry.setup.enemies.push(FullEnemySetup { initial_memory:None, instance_id: collision, ascension: 0,
        profile: enemies::EnemyProfile::Standard,
        enemy_id: EnemyId::Cultist, hp: 50, max_hp: 50, block: 0,
        intent: FullIntentId::Incantation, powers: vec![], ai_rng });
    let mut combat = Combat::enter(entry).unwrap();
    combat.state.enemies[0].hp = 72;
    combat.state.pending.push_back(Effect { source: Source::Actor(Actor::Player),
        target: Some(Actor::Enemy(parent.clone())), kind: EffectKind::Damage(DamagePayload::Fixed(2)) });
    super::super::engine::resolve(&mut combat.state);
    assert_eq!(combat.state.enemies[0].intent_id.as_str(), "split");
    combat.state.pending.push_back(Effect { source: Source::None, target: None,
        kind: EffectKind::MonsterStart });
    super::super::engine::resolve(&mut combat.state);

    let progress = combat.state.replacement_progress.as_ref().expect("replacement frame remains after child collision");
    assert_eq!(combat.state.fault.as_ref().unwrap().kind, FaultKind::Invariant);
    assert_eq!(progress.parent, parent);
    assert_eq!(progress.children.iter().map(|id| id.as_str()).collect::<Vec<_>>(),
        vec!["spike_slime_l", "acid_slime_l"]);
    assert_eq!(progress.next_child, 1);
    assert_eq!(progress.prepared.len(), 1);
    assert_eq!(progress.prepared[0].enemy_id.as_str(), "spike_slime_l");
    assert_eq!(combat.state.enemies[0].life, super::super::model::ActorLife::Active,
        "parent/vector commit is withheld when the second child cannot initialize");
}

fn slime_boss_entry(visit: u64, seed: [u8; 32]) -> super::super::CombatEntry {
    let player = super::support::base_scenario().player;
    let mut run_rng = crate::run::RunRng::new(seed, rng::Partition::Test, 37);
    enemies::setup(crate::content::EncounterId::SlimeBoss, 1, 0, &mut run_rng,
        enemies::SetupContext { player, player_powers: vec![], character: crate::content::Character::Ironclad,
            cards: vec![live_card(crate::content::CardId::Strike, CardInstanceId(961), 0)],
            opening_draw: 1, encounter_visit: visit }).unwrap()
}

#[test]
fn lethal_large_slime_damage_does_not_select_split_or_create_children() {
    let mut combat = large_acid_combat();
    let parent = combat.state.enemies[0].instance_id.clone();
    let (large_id, memory, split_intent) = match combat.state.enemies[0].enemy_id {
        ActorEnemyId::Full(EnemyId::AcidSlimeL) => (EnemyId::AcidSlimeL,
            EnemyMemory::AcidSlime { split_triggered: false }, FullIntentId::AcidSplit),
        ActorEnemyId::Full(EnemyId::SpikeSlimeL) => (EnemyId::SpikeSlimeL,
            EnemyMemory::SpikeSlime { split_triggered: false }, FullIntentId::SpikeSplit),
        _ => unreachable!("Large Slime encounter contains a large Acid or Spike Slime"),
    };
    combat.state.enemies[0].enemy_id = ActorEnemyId::Full(large_id);
    combat.state.enemies[0].full_memory = Some(memory);
    combat.state.enemies[0].hp = 1;
    let tackle = if large_id == EnemyId::AcidSlimeL { FullIntentId::AcidTackle } else { FullIntentId::SpikeFlameTackle };
    combat.state.enemies[0].intent_id = ActorIntentId::Full(tackle);
    combat.state.enemies[0].selected_move = ActorIntentId::Full(tackle);
    combat.state.enemies[0].move_history = vec![ActorIntentId::Full(tackle)];
    combat.state.pending.push_back(Effect { source: Source::Actor(Actor::Player),
        target: Some(Actor::Enemy(parent.clone())), kind: EffectKind::Damage(DamagePayload::Fixed(1)) });
    super::super::engine::resolve(&mut combat.state);
    assert_eq!(combat.state.enemies.len(), 1, "lethal damage is not a split replacement");
    assert_eq!(combat.state.enemies[0].hp, 0);
    assert_ne!(combat.state.enemies[0].intent_id, ActorIntentId::Full(split_intent));
}

fn large_acid_combat() -> Combat {
    let player = super::support::base_scenario().player;
    let mut rng = crate::run::RunRng::new([71; 32], rng::Partition::Test, 21);
    Combat::enter(enemies::setup(crate::content::EncounterId::LargeSlime, 1, 0, &mut rng,
        enemies::SetupContext { player, player_powers: vec![], character: crate::content::Character::Watcher,
            cards: vec![live_card(crate::content::CardId::Strike, CardInstanceId(911), 0)],
            opening_draw: 1, encounter_visit: 5 }).unwrap()).unwrap()
}

fn closure_entry(id:crate::content::EncounterId,ascension:u8)->CombatEntry{
    let mut rng=crate::run::RunRng::new([123;32],rng::Partition::Test,u64::from(ascension));
    enemies::setup(id,1,ascension,&mut rng,encounter_context(400,22_000)).unwrap()
}

#[test]
fn closure_slaver_turns_preserve_damage_weak_and_a17_repeat_threshold(){
    for ascension in [0,1,2,6,7,16,17,20] {
        for (intent,damage) in [("slaver_stab",if ascension>=2{13}else{12}),("slaver_rake",if ascension>=2{8}else{7})] {
            let mut entry=closure_entry(crate::content::EncounterId::BlueSlaver,ascension);
            entry.setup.enemies[0].intent=FullIntentId::from_name(intent);
            let mut combat=Combat::enter(entry).unwrap();
            assert_eq!(combat.public_full().unwrap().enemies[0].intent_damage,Some(damage));
            combat.step_full(&CombatAction::EndTurn).unwrap();
            assert_eq!(combat.state.player.hp,200-damage);
            let weak=combat.state.player_full_powers.iter().find(|power|power.id.0=="weak").map_or(0,|p|p.state.amount);
            assert_eq!(weak,if intent=="slaver_rake"{if ascension>=17{2}else{1}}else{0});
            if intent=="slaver_rake"&&ascension>=17 {
                assert_eq!(combat.state.enemies[0].intent_id.as_str(),"slaver_stab");
            }
        }
    }
}

#[test]
fn closure_fungi_real_grow_then_bite_use_ascension_strength_and_independent_actors(){
    for ascension in [0,1,2,6,7,16,17,20] {
        let mut entry=closure_entry(crate::content::EncounterId::TwoFungiBeasts,ascension);
        for enemy in &mut entry.setup.enemies {enemy.intent=FullIntentId::Grow;}
        assert_ne!(entry.setup.enemies[0].ai_rng.id(),entry.setup.enemies[1].ai_rng.id());
        let mut combat=Combat::enter(entry).unwrap();
        combat.step_full(&CombatAction::EndTurn).unwrap();
        let strength=if ascension>=17{5}else if ascension>=2{4}else{3};
        assert_eq!(combat.state.player.hp,200);
        for enemy in &combat.state.enemies {
            assert!(enemy.full_powers.iter().any(|p|p.id.0=="strength"&&p.state.amount==strength));
            assert_eq!(enemy.intent_id.as_str(),"bite");
            assert_eq!(enemy.intent_damage,Some(6+strength));
        }
        combat.step_full(&CombatAction::EndTurn).unwrap();
        assert_eq!(combat.state.player.hp,200-2*(6+strength));
    }
}

#[test]
fn closure_spore_cloud_nonfinal_death_has_null_source_and_artifact_applies(){
    for artifact in [false,true] {
        let mut entry=closure_entry(crate::content::EncounterId::TwoFungiBeasts,0);
        if artifact {entry.setup.player_powers.push(FullPowerEntry{id:FullPowerId("artifact"),state:PowerState{amount:1,skip_first:false,monster_source_grace:false}});}
        let mut combat=Combat::enter(entry).unwrap();
        for slot in 0..2 {
            let owner=combat.state.enemies[slot].instance_id.clone();
            combat.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Enemy(owner)),kind:EffectKind::Damage(DamagePayload::Fixed(99))}],None).unwrap();
            let cloud=combat.state.player_full_powers.iter().find(|p|p.id.0=="vulnerable");
            if artifact {assert!(cloud.is_none());assert!(!combat.state.player_full_powers.iter().any(|p|p.id.0=="artifact"));}
            else {let cloud=cloud.expect("only the nonfinal death applies cloud");assert_eq!(cloud.state.amount,2);assert!(!cloud.state.monster_source_grace);}
        }
        assert_eq!(combat.state.outcome,Some(Outcome::Victory));
    }
}

#[test]
fn closure_spore_cloud_thorns_death_changes_the_next_enemy_hit(){
    let mut entry=closure_entry(crate::content::EncounterId::TwoFungiBeasts,0);
    for enemy in &mut entry.setup.enemies {enemy.hp=1;enemy.max_hp=1;enemy.intent=FullIntentId::Bite;}
    entry.setup.player_powers.push(FullPowerEntry{id:FullPowerId("thorns"),state:PowerState{amount:1,skip_first:false,monster_source_grace:false}});
    let mut combat=Combat::enter(entry).unwrap();
    combat.step_full(&CombatAction::EndTurn).unwrap();
    assert_eq!(combat.state.outcome,Some(Outcome::Victory));
    assert_eq!(combat.state.player.hp,185,"first Bite6, then nonfinal Spore Cloud makes second Bite9");
}

#[test]
fn closure_fungi_and_slaver_selection_keep_proposals_history_and_owned_rng(){
    let id=rng::StreamId::new(Domain::RunCombatEnemyAi,vec!["fixture".into(),"elite".into()]).unwrap();
    // Reuse the accepted RNG-boundary fixtures. Ticket39 is below both cutoffs;
    // ticket40 is Stab for Slaver and Bite for Fungi; ticket75 proposes Grow.
    for (cursor,ticket) in [(8,39),(51,40),(16,75)] {
        let stream=rng::StreamState::new(id.clone(),[0;32],cursor).unwrap();
        assert_eq!(rng::uniform_below(&stream,100).unwrap().0,ticket);
        for ascension in [0,17] {
            let memory=EnemyMemory::for_enemy_at_ascension(EnemyId::FungiBeast,ascension).unwrap();
            for (history,want) in [(vec![],if ticket<60{FullIntentId::Bite}else{FullIntentId::Grow}),
                (vec![FullIntentId::Bite;2],FullIntentId::Grow),
                (vec![FullIntentId::Grow],FullIntentId::Bite)] {
                let selected=enemies::select_move(EnemyId::FungiBeast,&history,&memory,&stream).unwrap();
                assert_eq!(selected.intent,want);assert_eq!(selected.memory,memory);
                assert_eq!(selected.stream.word_index(),cursor+1);
            }
            let memory=EnemyMemory::for_enemy_at_ascension(EnemyId::SlaverBlue,ascension).unwrap();
            for (history,want) in [(vec!["slaver_rake"],if ascension>=17||ticket>=40{"slaver_stab"}else{"slaver_rake"}),
                (vec!["slaver_stab";2],"slaver_rake")] {
                let history=history.into_iter().map(FullIntentId::from_name).collect::<Vec<_>>();
                let selected=enemies::select_move(EnemyId::SlaverBlue,&history,&memory,&stream).unwrap();
                assert_eq!(selected.intent.as_str(),want);assert_eq!(selected.memory,memory);
                assert_eq!(selected.stream.word_index(),cursor+1);
            }
        }
    }
}
