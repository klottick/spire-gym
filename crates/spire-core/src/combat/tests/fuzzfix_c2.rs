//! Regressions for fuzz chunk C2 (spire-oto.22): the monster turn in source order
//! (source-check-endturn-v1 Q2, source-check-endturn-v2 B/C, source-check-monsterturn-v3).
//! MonsterStartTurnAction queues every monster's start-of-turn powers before the monster queue
//! is filled; each queued monster's actions drain before the next monster's takeTurn; after the
//! last move MonsterGroup.applyEndOfTurnPowers runs each live monster's end-turn powers in group
//! and list order, then the player's and the monsters' atEndOfRound powers.
use super::super::*;
use super::fuzzfix_b::{arrange, fixture, fixture_in};
use crate::combat::observe::{ResolvedEffect, ResolvedEffectKind};
use crate::{combat::CombatAction, content::{CardId, Character, EncounterId, RelicId}};
use CardId::*;

const ALL: [Character; 4] = [Character::Ironclad, Character::Silent, Character::Defect, Character::Watcher];

fn power(id: &'static str, amount: i32) -> FullPowerEntry {
    FullPowerEntry { id: FullPowerId(id), state: PowerState { amount, skip_first: false, monster_source_grace: false } }
}

fn end_turn(combat: &mut Combat) -> Vec<ResolvedEffect> {
    let observed = combat.step_full_observed(&CombatAction::EndTurn);
    observed.result.unwrap_or_else(|error| panic!("EndTurn: {error:?}; fault={:?}", combat.state.fault));
    assert!(combat.state.fault.is_none(), "EndTurn fault: {:?}", combat.state.fault);
    observed.effects
}

fn at(effects: &[ResolvedEffect], what: &str, matches: impl Fn(&ResolvedEffect) -> bool) -> u32 {
    effects.iter().find(|event| matches(event)).unwrap_or_else(|| panic!("no {what} in {effects:?}")).sequence
}

fn enemy(combat: &Combat, index: usize) -> Actor { Actor::Enemy(combat.state.enemies[index].instance_id.clone()) }

fn moved(actor: Actor) -> impl Fn(&ResolvedEffect) -> bool {
    move |event| matches!(&event.kind, ResolvedEffectKind::EnemyMove { actor: mover, .. } if *mover == actor)
}

fn damaged(actor: Actor) -> impl Fn(&ResolvedEffect) -> bool {
    move |event| event.target.as_ref() == Some(&actor) && matches!(event.kind, ResolvedEffectKind::Damage { .. })
}

fn blocked(actor: Actor) -> impl Fn(&ResolvedEffect) -> bool {
    move |event| event.target.as_ref() == Some(&actor) && matches!(event.kind, ResolvedEffectKind::Block { .. })
}

fn healed(actor: Actor) -> impl Fn(&ResolvedEffect) -> bool {
    move |event| event.target.as_ref() == Some(&actor) && matches!(event.kind, ResolvedEffectKind::Heal { .. })
}

fn power_change(actor: Actor, id: &'static str) -> impl Fn(&ResolvedEffect) -> bool {
    move |event| event.target.as_ref() == Some(&actor)
        && matches!(&event.kind, ResolvedEffectKind::Power { id: changed, .. } if changed.0 == id)
}

fn drawn(event: &ResolvedEffect) -> bool {
    matches!(event.kind, ResolvedEffectKind::CardMove { after: observe::PublicZone::Hand, .. })
}

fn two_louse(character: Character, relics: Vec<RelicId>) -> Combat {
    let cards = [(Strike, 0), (Strike, 0), (Defend, 0), (Defend, 0), (Defend, 0), (Defend, 0), (Defend, 0), (Defend, 0)];
    let mut combat = fixture_in(EncounterId::TwoLouse, 1, character, &cards, relics, vec![None; 3], 3);
    arrange(&mut combat, &[1, 2], &[3, 4, 5, 6, 7, 8], &[]);
    combat
}

/// MonsterStartTurnAction runs every monster's start-of-turn powers before the monster queue is
/// filled, so the second louse's Poison resolves before the first louse moves (base: each
/// monster's Poison resolved just before its own move).
#[test]
fn fuzzfix_c2_every_poison_resolves_before_the_first_move() {
    for character in ALL {
        let mut combat = two_louse(character, vec![]);
        combat.state.enemies[1].full_powers.push(power("poison", 3));
        let (first, second) = (enemy(&combat, 0), enemy(&combat, 1));
        let effects = end_turn(&mut combat);
        assert!(at(&effects, "second louse's Poison", damaged(second.clone())) < at(&effects, "first move", moved(first)),
            "{character:?}: {effects:?}");
        assert!(effects.iter().any(moved(second)), "{character:?}: the poisoned louse still moves");
    }
}

/// C1 review M3: a monster killed by Thorns during its move fires Gremlin Horn, whose addToBot
/// Energy and Draw resolve after that monster's queued actions and before the next monster's
/// takeTurn. The second louse then dies on Thorns too: Victory with the Horn card in hand
/// (base ce139820: the Draw waited behind every remaining move, so the hand was empty).
#[test]
fn fuzzfix_c2_gremlin_horn_draw_resolves_before_the_next_monster_moves() {
    for character in ALL {
        let mut combat = two_louse(character, vec![RelicId::GremlinHorn]);
        // G1 (M32): retargeted seed 0x42→0x02 for enemy AI so both empty-history Lice open on Bite.
        let source = crate::run::RunRng::new([0x02; 32], rng::Partition::Test, 0);
        let retargeted = combat.state.enemies.iter().map(|enemy| {
            let slot = combat.state.actor_slots[&enemy.instance_id] as u32;
            let id = match enemy.enemy_id { ActorEnemyId::Full(id) => id, _ => unreachable!() };
            let before = source.derive_combat_stream(rng::Domain::RunCombatEnemyAi, 1,
                EncounterId::TwoLouse.as_str(), 0, Some(slot), enemy.instance_id.as_str()).unwrap();
            let selection = enemies::select_move(id, &[], enemy.full_memory.as_ref().unwrap(), &before).unwrap();
            assert_eq!(selection.intent, FullIntentId::Bite, "{character:?}: retargeted opening");
            (enemy.instance_id.clone(), selection.intent, selection.memory, selection.stream)
        }).collect::<Vec<_>>();
        for (owner, intent, memory, stream) in retargeted {
            let enemy = combat.state.enemies.iter_mut().find(|enemy| enemy.instance_id == owner).unwrap();
            enemy.intent_id = ActorIntentId::Full(intent);
            enemy.selected_move = ActorIntentId::Full(intent);
            enemy.move_history = vec![ActorIntentId::Full(intent)];
            enemy.full_memory = Some(memory);
            enemy.full_ai_rng = Some(stream);
        }
        for enemy in &mut combat.state.enemies { enemy.hp = 1; }
        combat.state.player_full_powers.push(power("thorns", 5));
        let second = enemy(&combat, 1);
        let effects = end_turn(&mut combat);
        assert_eq!(combat.state.outcome, Some(Outcome::Victory), "{character:?}");
        assert!(at(&effects, "Horn draw", drawn) < at(&effects, "second move", moved(second)), "{character:?}: {effects:?}");
        assert_eq!(combat.state.piles.hand.len(), 1, "{character:?}: the first Horn draw resolved; the last death ends the combat");
    }
}

/// After the last move, each live monster's end-turn powers run in group order and power-list
/// order (Metallicize and Plated Armor at the pre-card slot, then Shackled = GainStrengthPower,
/// Intangible and Regenerate at atEndOfTurn), all before the player's atEndOfRound powers
/// (Blur, Weak). Base: they ran after the player's round work, and Plated Armor only for two
/// species inside their own move.
#[test]
fn fuzzfix_c2_monster_end_turn_pass_follows_every_move_and_precedes_the_round() {
    for character in ALL {
        let mut combat = two_louse(character, vec![]);
        for enemy in &mut combat.state.enemies { (enemy.hp, enemy.max_hp) = (40, 50); }
        combat.state.enemies[0].full_powers.extend([power("metallicize", 3), power("strength", -2), power("shackled", 2),
            power("intangible", 1), power("regenerate", 4)]);
        combat.state.enemies[1].full_powers.push(power("plated_armor", 4));
        combat.state.player_full_powers.extend([power("blur", 1), power("weak", 2)]);
        let (first, second) = (enemy(&combat, 0), enemy(&combat, 1));
        let effects = end_turn(&mut combat);
        let last_move = at(&effects, "second move", moved(second.clone()));
        let order = [
            at(&effects, "first Metallicize", blocked(first.clone())),
            at(&effects, "first Shackled restore", power_change(first.clone(), "strength")),
            at(&effects, "first Intangible", power_change(first.clone(), "intangible")),
            at(&effects, "first Regenerate", healed(first.clone())),
            at(&effects, "second Plated Armor", blocked(second.clone())),
            at(&effects, "player Blur", power_change(Actor::Player, "blur")),
            at(&effects, "player Weak", power_change(Actor::Player, "weak")),
        ];
        assert!(last_move < order[0] && order.windows(2).all(|pair| pair[0] < pair[1]), "{character:?}: {order:?} {effects:?}");
    }
}

/// GainStrengthPower.atEndOfTurn has no guard: Shackled restores Strength in every profile
/// (base: only when the draw-history flag was set).
#[test]
fn fuzzfix_c2_shackled_restores_without_the_history_flag() {
    for character in ALL {
        let mut combat = fixture(character, &[(Strike, 0)], vec![], vec![None; 3], 3);
        combat.state.silent.history_enabled = false;
        combat.state.enemies[0].full_powers.extend([power("strength", -3), power("shackled", 3)]);
        end_turn(&mut combat);
        let powers = &combat.state.enemies[0].full_powers;
        assert!(!powers.iter().any(|power| matches!(power.id.0, "strength" | "shackled") && power.state.amount != 0),
            "{character:?}: {powers:?}");
    }
}

/// Each TheBombPower is its own priority-5 power-list entry and ticks at its list position in
/// the player's atEndOfTurn pass: a Bomb installed before Ritual detonates before Ritual's
/// Strength, and one installed after it detonates after (base: every Bomb after the pass).
#[test]
fn fuzzfix_c2_bomb_ticks_at_its_power_list_position() {
    for character in ALL {
        for bomb_first in [true, false] {
            let mut combat = fixture(character, &[(TheBomb, 0)], vec![], vec![None; 3], 3);
            (combat.state.enemies[0].hp, combat.state.enemies[0].max_hp) = (200, 200);
            if !bomb_first { combat.state.player_full_powers.push(power("ritual", 2)); }
            combat.step_full(&CombatAction::PlayCard { instance: 1, target: None }).unwrap();
            if bomb_first { combat.state.player_full_powers.push(power("ritual", 2)); }
            combat.state.colorless.bombs[0].remaining = 1;
            let cultist = enemy(&combat, 0);
            let effects = end_turn(&mut combat);
            let (bomb, ritual) = (at(&effects, "Bomb", damaged(cultist)), at(&effects, "Ritual", power_change(Actor::Player, "strength")));
            assert_eq!(bomb < ritual, bomb_first, "{character:?} bomb_first={bomb_first}: {effects:?}");
            assert!(combat.state.colorless.bombs.is_empty() && !combat.state.player_full_powers.iter().any(|power| power.id.0 == "bomb_timer"),
                "{character:?}: the detonated Bomb leaves the list");
        }
    }
}

/// Two Bombs are two list entries: the public view shows one row per timer and no engine-only
/// entry, and each detonation removes exactly its own entry.
#[test]
fn fuzzfix_c2_two_bombs_keep_two_hidden_entries_and_two_public_rows() {
    let mut combat = fixture(Character::Silent, &[(TheBomb, 0), (TheBomb, 1)], vec![], vec![None; 3], 4);
    (combat.state.enemies[0].hp, combat.state.enemies[0].max_hp) = (500, 500);
    combat.step_full(&CombatAction::PlayCard { instance: 1, target: None }).unwrap();
    combat.step_full(&CombatAction::PlayCard { instance: 2, target: None }).unwrap();
    let entries = |combat: &Combat| combat.state.player_full_powers.iter().filter(|power| power.id.0 == "bomb_timer").count();
    assert_eq!(entries(&combat), 2);
    let view = view::full_player(&combat.state).unwrap();
    let rows = view.powers.iter().map(|power| power.id.0).filter(|id| id.starts_with("the_bomb") || *id == "bomb_timer").collect::<Vec<_>>();
    assert_eq!(rows, ["the_bomb", "the_bomb_plus"]);
    combat.state.colorless.bombs[0].remaining = 1;
    end_turn(&mut combat);
    assert_eq!(combat.state.enemies[0].hp, 460);
    assert_eq!((entries(&combat), combat.state.colorless.bombs.len(), combat.state.colorless.bombs[0].damage), (1, 1, 50));
}
