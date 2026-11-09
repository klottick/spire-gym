//! M34a public view completeness (plan a1/a2): intent category, planned hits and the live Heart cap.
use super::super::*;
use crate::content::{EnemyId, EncounterId};

fn combat(encounter: EncounterId, ascension: u8, seed: u8) -> Combat {
    Combat::enter(super::m29_a2_owner::setup(encounter, ascension, seed, 0).unwrap()).unwrap()
}

#[test]
fn heart_cap_in_view_equals_heart_invincible() {
    for ascension in [0, 18, 19, 20] {
        let view = combat(EncounterId::CorruptHeart, ascension, 7).public_full().unwrap();
        let heart = view.enemies.iter().find(|enemy| enemy.enemy_id == EnemyId::CorruptHeart).unwrap();
        assert_eq!(heart.invincible_cap, Some(enemies::ascension::heart_invincible(ascension)), "A{ascension}");
    }
    assert_eq!(enemies::ascension::heart_invincible(18), 300);
    assert_eq!(enemies::ascension::heart_invincible(19), 200);
    // Only the Heart carries the cap.
    let view = combat(EncounterId::ShieldAndSpear, 19, 7).public_full().unwrap();
    assert!(view.enemies.iter().all(|enemy| enemy.invincible_cap.is_none()));
}

fn set_intent(combat: &mut Combat, slot: usize, name: &'static str) {
    let selected = ActorIntentId::Full(FullIntentId::from_name(name));
    let enemy = &mut combat.state.enemies[slot];
    enemy.intent_id = selected.clone();
    enemy.selected_move = selected.clone();
    if let Some(last) = enemy.move_history.last_mut() { *last = selected; } else { enemy.move_history.push(selected); }
    effects::refresh_enemy_values(&mut combat.state).unwrap();
}

/// Recheck r3: the oracle is the resolver's damage applications from one actor during its turn, in a
/// fixture where nothing stops the move early (player HP 999 from the shared setup context, no
/// player retaliation powers, the attacker cannot die). Returns (published hits, resolved hits).
pub(super) fn published_and_resolved_hits(encounter: EncounterId, ascension: u8, enemy: EnemyId, intent: &'static str,
    prepare: impl FnOnce(&mut model::EnemyState)) -> (Option<u32>, u32) {
    published_and_resolved_hits_seeded((encounter, 3), ascension, enemy, intent, prepare)
}

fn published_and_resolved_hits_seeded((encounter, seed): (EncounterId, u8), ascension: u8, enemy: EnemyId, intent: &'static str,
    prepare: impl FnOnce(&mut model::EnemyState)) -> (Option<u32>, u32) {
    let mut combat = combat(encounter, ascension, seed);
    let slot = combat.state.enemies.iter().position(|e| e.enemy_id == ActorEnemyId::Full(enemy))
        .unwrap_or_else(|| panic!("{enemy:?} not in {encounter:?}"));
    set_intent(&mut combat, slot, intent);
    prepare(&mut combat.state.enemies[slot]);
    effects::refresh_enemy_values(&mut combat.state).unwrap();
    let handle = combat.state.enemies[slot].instance_id.clone();
    let published = combat.public_full().unwrap().enemies.iter().find(|e| e.instance_id == handle).unwrap().intent_hits;
    let observed = combat.step_full_observed(&CombatAction::EndTurn);
    // A forced intent can leave the actor's AI history inconsistent, so the NEXT selection (the
    // RollMove queued after the move) may fault. That happens after the move resolved; any other
    // fault fails the fixture.
    if let Err(error) = &observed.result {
        let text = format!("{error:?}");
        assert!(text.contains("kind: RollMove") && text.contains(handle.as_str()), "{enemy:?} {intent}: {text}");
    }
    let resolved = observed.effects.iter().filter(|effect| effect.source == Source::Actor(Actor::Enemy(handle.clone()))
        && effect.target == Some(Actor::Player)
        && matches!(effect.kind, ResolvedEffectKind::Damage { category: DamageCategory::Normal, .. })).count() as u32;
    assert!(observed.effects.iter().any(|effect| matches!(&effect.kind, ResolvedEffectKind::EnemyMove { actor, .. } if *actor == Actor::Enemy(handle.clone()))),
        "{enemy:?} {intent}: the actor took its turn");
    assert!(combat.state.player.hp > 0, "fixture must not let the player die mid-move");
    (published, resolved)
}

fn assert_hits(encounter: EncounterId, ascension: u8, enemy: EnemyId, intent: &'static str,
    prepare: impl FnOnce(&mut model::EnemyState), expected: Option<u32>) {
    let (published, resolved) = published_and_resolved_hits(encounter, ascension, enemy, intent, prepare);
    assert_eq!(published, expected, "{enemy:?} {intent} A{ascension}: published hits");
    assert_eq!(published.unwrap_or(0), resolved, "{enemy:?} {intent} A{ascension}: published hits = resolved applications");
}

#[test]
fn engine_owned_hit_counts_are_published_at_their_thresholds() {
    assert_hits(EncounterId::CorruptHeart, 3, EnemyId::CorruptHeart, "heart_blood_shots", |_| {}, Some(12));
    assert_hits(EncounterId::CorruptHeart, 4, EnemyId::CorruptHeart, "heart_blood_shots", |_| {}, Some(15));
    assert_hits(EncounterId::CorruptHeart, 20, EnemyId::CorruptHeart, "heart_echo", |_| {}, Some(1));
    assert_hits(EncounterId::CorruptHeart, 0, EnemyId::CorruptHeart, "heart_buff", |_| {}, None);
    assert_hits(EncounterId::ShieldAndSpear, 2, EnemyId::SpireSpear, "spear_skewer", |_| {}, Some(3));
    assert_hits(EncounterId::ShieldAndSpear, 3, EnemyId::SpireSpear, "spear_skewer", |_| {}, Some(4));
    assert_hits(EncounterId::ShieldAndSpear, 0, EnemyId::SpireSpear, "spear_burn_strike", |_| {}, Some(2));
    for stabs in [1, 2, 5] {
        assert_hits(EncounterId::BookOfStabbing, 18, EnemyId::BookOfStabbing, "book_multi",
            |e| e.full_memory = Some(enemies::EnemyMemory::BookOfStabbing { stab_count: stabs }), Some(stabs));
    }
    assert_hits(EncounterId::BookOfStabbing, 0, EnemyId::BookOfStabbing, "book_big", |_| {}, Some(1));
}

// ---- a1: census oracle (`m34a_intent_census_v1.json`, generated from the lane-I census) ----------

use super::super::intents::{self, IntentCategory};

fn census() -> Vec<serde_json::Value> {
    let value: serde_json::Value = serde_json::from_str(include_str!("m34a_intent_census_v1.json")).unwrap();
    value["rows"].as_array().unwrap().clone()
}

fn census_category(row: &serde_json::Value, ascension: u8) -> &str {
    row["category"].as_array().unwrap().iter()
        .find(|band| (band["from"].as_u64().unwrap()..=band["to"].as_u64().unwrap()).contains(&u64::from(ascension)))
        .unwrap()["intent"].as_str().unwrap()
}

fn census_row<'a>(rows: &'a [serde_json::Value], enemy: EnemyId, intent: &str) -> Option<&'a serde_json::Value> {
    rows.iter().find(|row| row["enemy"] == enemy.as_str() && row["intent"] == intent)
}

#[test]
fn intent_category_is_a_function_of_its_key() {
    let rows = census();
    let mut keys = std::collections::BTreeSet::new();
    for row in &rows {
        assert!(keys.insert((row["enemy"].as_str().unwrap(), row["intent"].as_str().unwrap())), "duplicate census key {row}");
        let bands = row["category"].as_array().unwrap();
        let mut next = 0;
        for band in bands {
            assert_eq!(band["from"].as_u64().unwrap(), next, "{row}: bands partition 0..=20");
            next = band["to"].as_u64().unwrap() + 1;
            assert!(IntentCategory::ALL.iter().any(|c| c.as_str() == band["intent"]), "{row}: source category");
        }
        assert_eq!(next, 21, "{row}: bands cover A20");
    }
}

#[test]
fn every_engine_enemy_move_has_a_census_category() {
    let rows = census();
    // The core table and the census are the same set of pairs, with the same category at every ascension.
    assert_eq!(intents::INTENT_ROWS.len(), rows.len(), "core table size = census size");
    for row in intents::INTENT_ROWS {
        let census = census_row(&rows, row.enemy, row.intent).unwrap_or_else(|| panic!("{:?} {} not in census", row.enemy, row.intent));
        for ascension in 0..=20 {
            assert_eq!(row.category.at(ascension).as_str(), census_category(census, ascension), "{:?} {} A{ascension}", row.enemy, row.intent);
        }
    }
    // Every registered intent name an enemy can select is some row's move (`hidden` masks; the gremlin
    // flee is published as an Escaped actor, never as an intent).
    for name in public_ids::PUBLIC_INTENTS.iter().chain(public_ids::EVENT_INTENTS) {
        // `hidden` masks; the gremlin flee is an Escaped actor, never an intent. The other five are
        // frozen-vocabulary names no producer selects (on ea409f82 they occur only in public_ids.rs and
        // one test fixture): Lagavulin/Sentry use `attack`/`stunned`/`bolt`/`beam`.
        if ["hidden", "gremlin_escape", "stun", "lag_attack", "sentry_bolt", "sentry_beam", "charge_up"].contains(name) { continue; }
        assert!(intents::INTENT_ROWS.iter().any(|row| row.intent == *name), "registered intent {name} has no census row");
    }
}

/// Hits are published exactly for attack-category moves (`Some(1)` single hits; `None` otherwise).
#[test]
fn intent_hits_are_published_for_attack_categories_only() {
    for row in intents::INTENT_ROWS {
        for ascension in 0..=20 {
            let attack = matches!(row.category.at(ascension), IntentCategory::Attack | IntentCategory::AttackBuff
                | IntentCategory::AttackDebuff | IntentCategory::AttackDefend);
            assert_eq!(row.hits != intents::HitsRule::NoDamage, attack, "{:?} {} A{ascension}", row.enemy, row.intent);
        }
    }
}

/// Observed selection, table-driven over every encounter: at each decision of a passive fight (the
/// player ends turn; HP 999), every active, unhidden enemy publishes the census category for its
/// selected move at the combat ascension, and hits iff the census says the move deals damage.
#[test]
fn actor_records_census_category_at_move_selection() {
    let rows = census();
    let (mut seen, mut failures, mut natural) = (std::collections::BTreeSet::new(), std::collections::BTreeSet::new(), 0);
    for &(encounter, _, _) in enemies::A0_ENCOUNTER_SETUPS {
        for ascension in [0u8, 17, 20] {
            for seed in 0..2u8 {
                let mut combat = combat(encounter, ascension, seed);
                for _turn in 0..12 {
                    let view = combat.public_full().unwrap();
                    if view.outcome.is_some() { break; }
                    for enemy in view.enemies.iter().filter(|e| !e.replaced && !e.escaped && e.hp > 0) {
                        let name = enemy.intent.as_str();
                        let row = census_row(&rows, enemy.enemy_id, name)
                            .unwrap_or_else(|| panic!("{encounter:?} A{ascension}: {:?} {name} not in census", enemy.enemy_id));
                        assert_eq!(enemy.intent_category.map(IntentCategory::as_str), Some(census_category(row, ascension)),
                            "{encounter:?} A{ascension}: {:?} {name}", enemy.enemy_id);
                        assert_eq!(enemy.intent_hits.is_some(), row["hits"]["kind"] != "none",
                            "{encounter:?} A{ascension}: {:?} {name} hits shape", enemy.enemy_id);
                        seen.insert((enemy.enemy_id.as_str(), name));
                    }
                    // r3 oracle on natural turns (covers spawn-only actors): an actor whose published move
                    // ran this turn, with nothing stopping it early (it survives, the player survives),
                    // made exactly its published number of NORMAL damage applications.
                    let published: Vec<_> = view.enemies.iter().map(|e| (e.instance_id.clone(), e.intent, e.intent_hits)).collect();
                    let observed = combat.step_full_observed(&CombatAction::EndTurn);
                    if observed.result.is_err() || combat.state.player.hp <= 0 { break; }
                    for (handle, intent, hits) in published {
                        let actor = Actor::Enemy(handle.clone());
                        let moved = observed.effects.iter().any(|effect| matches!(&effect.kind,
                            ResolvedEffectKind::EnemyMove { actor: a, move_id } if *a == actor && move_id.as_str() == intent.as_str()));
                        let survived = combat.state.enemies.iter().any(|e| e.instance_id == handle && e.life == ActorLife::Active && e.hp > 0);
                        if !moved || !survived { continue; }
                        let resolved = observed.effects.iter().filter(|effect| effect.source == Source::Actor(actor.clone())
                            && effect.target == Some(Actor::Player)
                            && matches!(effect.kind, ResolvedEffectKind::Damage { category: DamageCategory::Normal, .. })).count() as u32;
                        if hits.unwrap_or(0) != resolved {
                            failures.insert(format!("{encounter:?} A{ascension}: {handle:?} {} published {hits:?} resolved {resolved}", intent.as_str()));
                        }
                        natural += 1;
                    }
                }
            }
        }
    }
    assert!(failures.is_empty(), "{} natural-turn hit mismatches:\n{}", failures.len(), failures.into_iter().collect::<Vec<_>>().join("\n"));
    assert!(seen.len() > 120 && natural > 500, "the passive sweep observed {} pairs, {natural} natural moves", seen.len());
}

fn category_at(encounter: EncounterId, enemy: EnemyId, intent: &'static str, ascension: u8) -> Option<IntentCategory> {
    let mut combat = combat(encounter, ascension, 1);
    let slot = combat.state.enemies.iter().position(|e| e.enemy_id == ActorEnemyId::Full(enemy)).unwrap();
    set_intent(&mut combat, slot, intent);
    combat.public_full().unwrap().enemies[slot].intent_category
}

#[test]
fn darkling_harden_switches_at_a17() {
    assert_eq!(category_at(EncounterId::ThreeDarklings, EnemyId::Darkling, "darkling_harden", 16), Some(IntentCategory::Defend));
    assert_eq!(category_at(EncounterId::ThreeDarklings, EnemyId::Darkling, "darkling_harden", 17), Some(IntentCategory::DefendBuff));
}

#[test]
fn deca_square_switches_at_a19() {
    assert_eq!(category_at(EncounterId::DonuAndDeca, EnemyId::Deca, "deca_square", 18), Some(IntentCategory::Defend));
    assert_eq!(category_at(EncounterId::DonuAndDeca, EnemyId::Deca, "deca_square", 19), Some(IntentCategory::DefendBuff));
}

/// An (encounter, seed) whose opening formation holds `enemy` at `ascension`.
fn encounter_with(enemy: EnemyId, ascension: u8) -> Option<(EncounterId, u8)> {
    (0..16u8).flat_map(|seed| enemies::A0_ENCOUNTER_SETUPS.iter().map(move |(id, _, _)| (*id, seed))).find(|(id, seed)| {
        super::m29_a2_owner::setup(*id, ascension, *seed, 0).ok()
            .and_then(|entry| Combat::enter(entry).ok())
            .is_some_and(|combat| combat.state.enemies.iter().any(|e| e.enemy_id == ActorEnemyId::Full(enemy)))
    })
}

/// a2 (recheck r3): for every census attack row, at each ascension where its count can change, the
/// published hits equal the resolver's damage applications from that actor in a no-early-stop fixture.
#[test]
fn actor_intent_hits_equal_resolved_damage_applications() {
    let rows = census();
    let (mut checked, mut failures, mut spawn_only) = (0, Vec::new(), std::collections::BTreeSet::new());
    for table in intents::INTENT_ROWS {
        let (enemy, intent) = (table.enemy, table.intent);
        let name = enemy.as_str();
        let row = census_row(&rows, enemy, intent).unwrap();
        if row["hits"]["kind"] == "none" { continue; }
        let ascensions: Vec<u8> = match row["hits"]["kind"].as_str().unwrap() {
            "ascension" => row["hits"]["bands"].as_array().unwrap().iter().map(|b| b["from"].as_u64().unwrap() as u8).collect(),
            _ => vec![0, 20],
        };
        for ascension in ascensions {
            // Spawn-only actors (Torch Head, Bronze Orb, daggers, split slimes, summoned gremlins) never
            // open a formation; the passive sweep checks their hits on their natural turns.
            let Some(encounter) = encounter_with(enemy, ascension) else { spawn_only.insert(name); continue };
            let (published, resolved) = published_and_resolved_hits_seeded(encounter, ascension, enemy, intent, |e| {
                if let Some(enemies::EnemyMemory::Maw { turns, .. }) = e.full_memory.as_mut() { *turns = 5; }
            });
            let expected = match row["hits"]["kind"].as_str().unwrap() {
                "fixed" => row["hits"]["n"].as_u64().map(|n| n as u32),
                "ascension" => row["hits"]["bands"].as_array().unwrap().iter()
                    .find(|b| (b["from"].as_u64().unwrap()..=b["to"].as_u64().unwrap()).contains(&u64::from(ascension)))
                    .and_then(|b| b["n"].as_u64()).map(|n| n as u32),
                _ => published,
            };
            if published != expected || published.unwrap_or(0) != resolved {
                failures.push(format!("{name} {intent} A{ascension}: published {published:?} census {expected:?} resolved {resolved}"));
            }
            checked += 1;
        }
    }
    assert!(failures.is_empty(), "{} hit mismatches:\n{}", failures.len(), failures.join("\n"));
    assert!(checked > 60, "checked {checked} attack rows (spawn-only: {spawn_only:?})");
}

/// Final review m4: rows written from the decompiled source (lane m4, `m34a/lane-m4/independent-intent-rows-v1.json`),
/// independently of the census and its generator; the three below-threshold rows are added from the same source constants.
/// (EnemyId, engine intent, source category, ascension, planned hits, source locator under `monsters/`.)
#[test]
fn independent_source_rows_match_the_intent_table() {
    let rows: &[(EnemyId, &str, &str, u8, Option<u32>, &str)] = &[
        (EnemyId::GremlinNob, "bellow", "buff", 0, None, "exordium/GremlinNob.java:124"),
        (EnemyId::Hexaghost, "inferno", "attack_debuff", 0, Some(6), "exordium/Hexaghost.java:71,243"),
        (EnemyId::Guardian, "whirlwind", "attack", 0, Some(4), "exordium/TheGuardian.java:64,173"),
        (EnemyId::Lagavulin, "sleep", "sleep", 0, None, "exordium/Lagavulin.java:134"),
        (EnemyId::Sentry, "bolt", "debuff", 0, None, "exordium/Sentry.java:130"),
        (EnemyId::SlimeBoss, "split", "unknown", 0, None, "exordium/SlimeBoss.java:147"),
        (EnemyId::JawWorm, "bellow", "defend_buff", 0, None, "exordium/JawWorm.java:151"),
        (EnemyId::Byrd, "byrd_peck", "attack", 2, Some(6), "city/Byrd.java:79-84,176"),
        (EnemyId::Chosen, "chosen_hex", "strong_debuff", 0, None, "city/Chosen.java:150"),
        (EnemyId::ShelledParasite, "shelled_parasite_double_strike", "attack", 0, Some(2), "city/ShelledParasite.java:174"),
        (EnemyId::Taskmaster, "taskmaster_whip", "attack_debuff", 0, Some(1), "city/Taskmaster.java:77"),
        (EnemyId::BronzeAutomaton, "automaton_flail", "attack", 0, Some(2), "city/BronzeAutomaton.java:162"),
        (EnemyId::GremlinLeader, "leader_rally", "unknown", 0, None, "city/GremlinLeader.java:142"),
        (EnemyId::Centurion, "centurion_fury", "attack", 0, Some(3), "city/Centurion.java:40-41,59-64,137"),
        (EnemyId::TimeEater, "time_reverb", "attack", 0, Some(3), "beyond/TimeEater.java:176"),
        (EnemyId::Nemesis, "nemesis_tri", "attack", 0, Some(3), "beyond/Nemesis.java:143"),
        (EnemyId::Donu, "donu_beam", "attack", 0, Some(2), "beyond/Donu.java:120"),
        (EnemyId::Reptomancer, "repto_snake", "attack_debuff", 0, Some(2), "beyond/Reptomancer.java:169"),
        (EnemyId::WrithingMass, "writhing_multi", "attack", 0, Some(3), "beyond/WrithingMass.java:40,144"),
        (EnemyId::Darkling, "darkling_chomp", "attack", 0, Some(2), "beyond/Darkling.java:156"),
        (EnemyId::Maw, "maw_roar", "strong_debuff", 0, None, "beyond/Maw.java:115"),
        (EnemyId::CorruptHeart, "heart_blood_shots", "attack", 4, Some(15), "ending/CorruptHeart.java:71,74-78,175"),
        (EnemyId::SpireShield, "shield_fortify", "defend", 0, None, "ending/SpireShield.java:111,122"),
        (EnemyId::SpireSpear, "spear_skewer", "attack", 3, Some(4), "ending/SpireSpear.java:40,55-60,121"),
        (EnemyId::Byrd, "byrd_peck", "attack", 1, Some(5), "city/Byrd.java:79-84,176"),
        (EnemyId::CorruptHeart, "heart_blood_shots", "attack", 3, Some(12), "ending/CorruptHeart.java:71,74-78,175"),
        (EnemyId::SpireSpear, "spear_skewer", "attack", 2, Some(3), "ending/SpireSpear.java:40,55-60,121"),
    ];
    assert!(rows.len() >= 20);
    for &(enemy, intent, category, ascension, hits, source) in rows {
        assert_eq!(intents::category(enemy, intent, ascension).map(|c| c.as_str()), Some(category), "{enemy:?} {intent} A{ascension} ({source})");
        assert_eq!(intents::hits_for(enemy, intent, ascension, None), hits, "{enemy:?} {intent} A{ascension} hits ({source})");
    }
}
