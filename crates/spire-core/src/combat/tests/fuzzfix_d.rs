//! Regressions for fuzz chunk D (spire-oto.26): addToBot work raised during the monster turn
//! resolves before the next monster's takeTurn or, after the last move, before
//! MonsterGroup.applyEndOfTurnPowers; that pass and the monsters' atEndOfRound visit monsters in
//! group (slot) order (C2 review M1/M2/M4; source-check-monsterturn-v3).
use super::super::*;
use super::fuzzfix_b::{arrange, fixture, fixture_in};
use crate::combat::observe::{ResolvedEffect, ResolvedEffectKind};
use crate::{combat::CombatAction, content::{CardId, Character, EncounterId, RelicId}, };
use CardId::*;

const ALL: [Character; 4] = [Character::Ironclad, Character::Silent, Character::Defect, Character::Watcher];

fn power(id: &'static str, amount: i32) -> FullPowerEntry {
    FullPowerEntry { id: FullPowerId(id), state: PowerState { amount, skip_first: false, monster_source_grace: false } }
}

fn observed(combat: &mut Combat, action: CombatAction) -> Vec<ResolvedEffect> {
    let observed = combat.step_full_observed(&action);
    observed.result.unwrap_or_else(|error| panic!("{action:?}: {error:?}; fault={:?}", combat.state.fault));
    assert!(combat.state.fault.is_none(), "{action:?} fault: {:?}", combat.state.fault);
    observed.effects
}

fn end_turn(combat: &mut Combat) -> Vec<ResolvedEffect> { observed(combat, CombatAction::EndTurn) }

fn at(effects: &[ResolvedEffect], what: &str, matches: impl Fn(&ResolvedEffect) -> bool) -> u32 {
    effects.iter().find(|event| matches(event)).unwrap_or_else(|| panic!("no {what} in {effects:?}")).sequence
}

fn enemy(combat: &Combat, index: usize) -> Actor { Actor::Enemy(combat.state.enemies[index].instance_id.clone()) }

fn amount(combat: &Combat, index: usize, id: &str) -> i32 {
    combat.state.enemies[index].full_powers.iter().find(|power| power.id.0 == id).map_or(0, |power| power.state.amount)
}

fn moved(actor: Actor) -> impl Fn(&ResolvedEffect) -> bool {
    move |event| matches!(&event.kind, ResolvedEffectKind::EnemyMove { actor: mover, .. } if *mover == actor)
}

fn gained_block(actor: Actor) -> impl Fn(&ResolvedEffect) -> bool {
    move |event| event.target.as_ref() == Some(&actor) && matches!(event.kind, ResolvedEffectKind::Block { before, after } if after > before)
}

fn blocked(actor: Actor) -> impl Fn(&ResolvedEffect) -> bool {
    move |event| event.target.as_ref() == Some(&actor) && matches!(event.kind, ResolvedEffectKind::Block { .. })
}

fn power_change(actor: Actor, id: &'static str) -> impl Fn(&ResolvedEffect) -> bool {
    move |event| event.target.as_ref() == Some(&actor)
        && matches!(&event.kind, ResolvedEffectKind::Power { id: changed, .. } if changed.0 == id)
}

fn two_louse_with(character: Character, relics: Vec<RelicId>, draw: &[(CardId, u32)]) -> Combat {
    let mut cards = vec![(Strike, 0), (Strike, 0)];
    cards.extend_from_slice(draw);
    let mut combat = fixture_in(EncounterId::TwoLouse, 1, character, &cards, relics, vec![None; 3], 3);
    let draw = (3..=cards.len() as u64).collect::<Vec<_>>();
    arrange(&mut combat, &[1, 2], &draw, &[]);
    combat
}

/// C2 review M1 (confirmed): Poison wakes a sleeping Lagavulin in the pre-turn. Lagavulin.damage
/// queues ChangeState OPEN and its ReducePower Metallicize 8 at the bottom; both resolve before
/// its takeTurn, so the end-turn pass has no Metallicize left (base: the reduce waited behind
/// EndRound and Lagavulin kept 8 Block into the player's turn).
#[test]
fn fuzzfix_d_poisoned_sleeping_lagavulin_wakes_without_end_turn_block() {
    for character in ALL {
        let mut combat = fixture_in(EncounterId::Lagavulin, 1, character, &[(Strike, 0)], vec![], vec![None; 3], 3);
        assert_eq!(amount(&combat, 0, "metallicize"), 8, "{character:?}: asleep");
        combat.state.enemies[0].full_powers.push(power("poison", 5));
        let lagavulin = enemy(&combat, 0);
        let effects = end_turn(&mut combat);
        assert!(at(&effects, "Metallicize reduce", power_change(lagavulin.clone(), "metallicize"))
            < at(&effects, "Lagavulin move", moved(lagavulin)), "{character:?}: {effects:?}");
        assert_eq!((combat.state.enemies[0].block, amount(&combat, 0, "metallicize")), (0, 0), "{character:?}");
    }
}

/// Work queued by a power callback during the last monster's move (Fire Breathing on the Gremlin
/// Horn draw of a Burn after a Thorns kill) resolves before applyEndOfTurnPowers: its damage
/// hits the surviving louse's HP before Metallicize gives Block (base: after EndRound, so the
/// pass's Block absorbed it).
#[test]
fn fuzzfix_d_power_callback_of_the_last_move_precedes_the_monster_pass() {
    for character in ALL {
        let mut combat = two_louse_with(character, vec![RelicId::GremlinHorn], &[(Defend, 0), (Burn, 0)]);
        // G1 (M32): retargeted seed 0x42→0x02 for enemy AI so both empty-history Lice open on Bite.
        let source = crate::run::RunRng::new([0x02; 32], super::super::rng::Partition::Test, 0);
        let retargeted = combat.state.enemies.iter().map(|enemy| {
            let slot = combat.state.actor_slots[&enemy.instance_id] as u32;
            let id = match enemy.enemy_id { ActorEnemyId::Full(id) => id, _ => unreachable!() };
            let before = source.derive_combat_stream(super::super::rng::Domain::RunCombatEnemyAi, 1,
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
        (combat.state.enemies[0].hp, combat.state.enemies[0].max_hp) = (40, 40);
        combat.state.enemies[0].full_powers.push(power("metallicize", 6));
        combat.state.enemies[1].hp = 1;
        combat.state.player_full_powers.extend([power("thorns", 5), power("fire_breathing", 6)]);
        end_turn(&mut combat);
        assert_eq!(combat.state.enemies[1].hp, 0, "{character:?}: the last mover died on Thorns");
        // Thorns 5 on the first louse's bite, then Fire Breathing 6 before the pass's Block 6.
        assert_eq!((combat.state.enemies[0].hp, combat.state.enemies[0].block), (29, 6), "{character:?}");
    }
}

/// C2 review M2: a monster placed in a lower slot than its position in the enemy list (a Gremlin
/// Leader minion summoned into slot 0) runs its end-turn powers and its atEndOfRound powers first
/// (base: list order).
#[test]
fn fuzzfix_d_monster_pass_and_round_end_follow_slot_order() {
    for character in ALL {
        let mut combat = two_louse_with(character, vec![], &[(Defend, 0); 6]);
        // G1 (M32): retargeted seed 0x42→0x02 for enemy AI so both empty-history Lice open on Bite.
        let source = crate::run::RunRng::new([0x02; 32], super::super::rng::Partition::Test, 0);
        let retargeted = combat.state.enemies.iter().map(|enemy| {
            let slot = combat.state.actor_slots[&enemy.instance_id] as u32;
            let id = match enemy.enemy_id { ActorEnemyId::Full(id) => id, _ => unreachable!() };
            let before = source.derive_combat_stream(super::super::rng::Domain::RunCombatEnemyAi, 1,
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
        combat.state.enemies.swap(0, 1);
        let (slot0, slot1) = (enemy(&combat, 1), enemy(&combat, 0));
        assert!(combat.state.actor_slots[&combat.state.enemies[1].instance_id] < combat.state.actor_slots[&combat.state.enemies[0].instance_id]);
        for enemy in &mut combat.state.enemies {
            (enemy.hp, enemy.max_hp) = (40, 40);
            enemy.full_powers.extend([power("metallicize", 2), power("ritual", 1)]);
        }
        let effects = end_turn(&mut combat);
        assert!(at(&effects, "slot 0 Metallicize", blocked(slot0.clone())) < at(&effects, "slot 1 Metallicize", blocked(slot1.clone())),
            "{character:?}: {effects:?}");
        assert!(at(&effects, "slot 0 Ritual", power_change(slot0, "strength")) < at(&effects, "slot 1 Ritual", power_change(slot1, "strength")),
            "{character:?}: {effects:?}");
    }
}

/// Characterization (C2 review M4): the pass precedes the player's atEndOfRound powers on its own.
#[test]
fn fuzzfix_d_monster_pass_precedes_player_blur_and_weak() {
    for character in ALL {
        let mut combat = two_louse_with(character, vec![], &[(Defend, 0); 6]);
        for enemy in &mut combat.state.enemies { (enemy.hp, enemy.max_hp) = (40, 40); }
        combat.state.enemies[0].full_powers.push(power("metallicize", 3));
        combat.state.player_full_powers.extend([power("blur", 1), power("weak", 2)]);
        let effects = end_turn(&mut combat);
        let block = at(&effects, "Metallicize", blocked(enemy(&combat, 0)));
        assert!(block < at(&effects, "Blur", power_change(Actor::Player, "blur")), "{character:?}: {effects:?}");
        assert!(block < at(&effects, "Weak", power_change(Actor::Player, "weak")), "{character:?}: {effects:?}");
    }
}

/// C2 review M4 / D review m2: RegenerateMonsterPower skips a half-dead owner, so not even a
/// Heal is queued. The Awakened One killed by Thorns during its own move is half-dead at the
/// end-turn pass and keeps Regenerate (a dying Darkling clears its powers); a live owner heals.
#[test]
fn fuzzfix_d_regenerate_skips_a_half_dead_awakened_one() {
    let mut combat = fixture_in(EncounterId::AwakenedOne, 3, Character::Silent, &[(Defend, 0)], vec![], vec![None; 3], 3);
    let awakened = combat.state.enemies.iter().position(|enemy| enemy.full_powers.iter().any(|power| power.id.0 == "regenerate")).unwrap();
    combat.state.enemies[awakened].hp = 1;
    let cultist = (0..combat.state.enemies.len()).find(|index| *index != awakened).unwrap();
    combat.state.enemies[cultist].hp = combat.state.enemies[cultist].max_hp - 10;
    combat.state.enemies[cultist].full_powers.push(power("regenerate", 4));
    combat.state.player_full_powers.push(power("thorns", 3));
    let (owner, live) = (enemy(&combat, awakened), enemy(&combat, cultist));
    let effects = end_turn(&mut combat);
    assert_eq!(combat.state.enemies[awakened].life, ActorLife::HalfDead, "{effects:?}");
    assert!(amount(&combat, awakened, "regenerate") > 0, "the half-dead owner keeps Regenerate");
    assert!(!effects.iter().any(|event| event.target.as_ref() == Some(&owner) && matches!(event.kind, ResolvedEffectKind::Heal { .. })),
        "half-dead: no Heal queued: {effects:?}");
    assert!(effects.iter().any(|event| event.target.as_ref() == Some(&live) && matches!(event.kind, ResolvedEffectKind::Heal { amount: 4, .. })),
        "live: healed 4: {effects:?}");
}

/// Characterization (C2 review M4): Malleable resets in the end-turn pass at its list position,
/// after a Metallicize listed before it (before C2 the reset ended the Snake Plant's own move).
#[test]
fn fuzzfix_d_malleable_resets_in_the_pass_after_the_move() {
    let mut combat = fixture_in(EncounterId::SnakePlant, 2, Character::Ironclad, &[(Strike, 0), (Strike, 0)], vec![], vec![None; 3], 3);
    let base = amount(&combat, 0, "malleable");
    combat.state.enemies[0].full_powers.insert(0, power("metallicize", 2));
    combat.step_full(&CombatAction::PlayCard { instance: 1, target: Some(TargetId(1)) }).unwrap();
    combat.step_full(&CombatAction::PlayCard { instance: 2, target: Some(TargetId(1)) }).unwrap();
    assert!(amount(&combat, 0, "malleable") > base);
    let plant = enemy(&combat, 0);
    let effects = end_turn(&mut combat);
    let reset = at(&effects, "reset", power_change(plant.clone(), "malleable"));
    let metallicize = at(&effects, "pass Metallicize", gained_block(plant.clone()));
    assert!(at(&effects, "move", moved(plant)) < metallicize && metallicize < reset, "{effects:?}");
    assert_eq!(amount(&combat, 0, "malleable"), base);
}

/// Characterization (C2 review M4): the engine-only Bomb entries never reach the observed stream.
#[test]
fn fuzzfix_d_bomb_entries_stay_out_of_the_observed_stream() {
    for character in ALL {
        let mut combat = fixture(character, &[(TheBomb, 0)], vec![], vec![None; 3], 3);
        (combat.state.enemies[0].hp, combat.state.enemies[0].max_hp) = (200, 200);
        let mut effects = observed(&mut combat, CombatAction::PlayCard { instance: 1, target: None });
        combat.state.colorless.bombs[0].remaining = 1;
        effects.extend(end_turn(&mut combat));
        assert!(effects.iter().any(|event| matches!(event.kind, ResolvedEffectKind::Damage { .. })), "{character:?}: detonated");
        assert!(!effects.iter().any(|event| matches!(&event.kind, ResolvedEffectKind::Power { id, .. } if id.0 == "bomb_timer")),
            "{character:?}: {effects:?}");
    }
}

fn hand_cost(combat: &Combat, instance: u64) -> Option<i32> {
    let card = combat.state.piles.hand.iter().filter_map(|owned| owned.full())
        .find(|card| card.instance == CardInstanceId(instance)).unwrap_or_else(|| panic!("card {instance} not in hand"));
    cards::visible_cost(&combat.state, card)
}

/// spire-oto.23: EstablishmentPowerAction calls modifyCostForCombat on every retained hand card,
/// whatever its colour. Payload-priced complete Ironclad and Silent cards (no local cost field)
/// get the discount too (base: unchanged), and Blood for Blood keeps its HP-loss reduction law.
#[test]
fn fuzzfix_d_establishment_discounts_payload_priced_cards_of_every_colour() {
    for character in ALL {
        let mut combat = fixture(character, &[(Bash, 0), (Dash, 0), (BloodForBlood, 0)], vec![], vec![None; 3], 3);
        combat.state.player_full_powers.push(power("establishment", 1));
        for owned in &mut combat.state.piles.hand { owned.full_mut().unwrap().local.retain = true; }
        let before = [hand_cost(&combat, 1), hand_cost(&combat, 2), hand_cost(&combat, 3)];
        assert_eq!(before, [Some(2), Some(2), Some(4)], "{character:?}");
        end_turn(&mut combat);
        assert_eq!([hand_cost(&combat, 1), hand_cost(&combat, 2), hand_cost(&combat, 3)], [Some(1), Some(1), Some(3)], "{character:?}");
        let blood = combat.state.piles.hand.iter_mut().filter_map(|owned| owned.full_mut())
            .find(|card| card.id == BloodForBlood).unwrap();
        card_state::reduce_blood_for_blood_cost(blood);
        assert_eq!(hand_cost(&combat, 3), Some(2), "{character:?}: a later HP loss still lowers Blood for Blood");
    }
}

fn damages(actor: Actor) -> impl Fn(&&ResolvedEffect) -> bool {
    move |event| event.target.as_ref() == Some(&actor) && matches!(event.kind, ResolvedEffectKind::Damage { .. })
}

/// C2 review / D review m3: addToBot work raised by a card's effects precedes an already queued
/// card (GameActionManager uses the card queue only once the action queue is empty). Louse Curl Up
/// Block from the first Strike absorbs the Double Tap repeat (base: the Block came after it).
#[test]
fn fuzzfix_d_curl_up_block_precedes_the_double_tap_repeat() {
    for character in ALL {
        let mut combat = two_louse_with(character, vec![], &[(Defend, 0); 6]);
        for enemy in &mut combat.state.enemies { (enemy.hp, enemy.max_hp) = (40, 40); }
        let curl = amount(&combat, 0, "curl_up");
        assert!(curl > 0, "{character:?}: the louse has Curl Up");
        combat.state.player_full_powers.push(power("double_tap", 1));
        let louse = enemy(&combat, 0);
        let effects = observed(&mut combat, CombatAction::PlayCard { instance: 1, target: Some(TargetId(1)) });
        let hits = effects.iter().filter(damages(louse.clone())).collect::<Vec<_>>();
        assert_eq!(hits.len(), 2, "{character:?}: {effects:?}");
        assert!(matches!(hits[1].kind, ResolvedEffectKind::Damage { absorbed_block, .. } if absorbed_block == curl.min(6)),
            "{character:?}: the repeat hits Curl Up's Block: {effects:?}");
    }
}

fn belt_fixture(character: Character) -> Combat {
    let mut items = Some(crate::items::CombatItemContext::new(vec![RelicId::ChampionsBelt], vec![None; 3]).with_complete_cards());
    let mut entry = enemies::setup(EncounterId::Cultist, 1, 0, &mut crate::run::RunRng::new([0x42; 32], rng::Partition::Test, 0),
        enemies::SetupContext {
            player: PlayerState { hp: 80, max_hp: 80, block: 0, energy: 3, max_energy: 3, draw_per_turn: 5, powers: vec![] },
            player_powers: vec![], character, cards: vec![permanent_live_card(Defend, CardInstanceId(99), 0)],
            opening_draw: 1, encounter_visit: 0 }).unwrap();
    entry.cards = vec![complete_live_card(Bash, CardInstanceId(1), 0).unwrap()];
    entry.opening_draw = 1;
    Combat::enter_with_item_context(entry, &mut items).unwrap_or_else(|error| panic!("{character:?}: {error:?}"))
}

/// D review m3: Champion Belt's Weak (addToBot) resolves before the Double Tap repeat of Bash.
#[test]
fn fuzzfix_d_champion_belt_weak_precedes_the_double_tap_repeat() {
    for character in ALL {
        let mut combat = belt_fixture(character);
        (combat.state.enemies[0].hp, combat.state.enemies[0].max_hp) = (200, 200);
        combat.state.player_full_powers.push(power("double_tap", 1));
        let cultist = enemy(&combat, 0);
        let effects = observed(&mut combat, CombatAction::PlayCard { instance: 1, target: Some(TargetId(1)) });
        let second = effects.iter().filter(damages(cultist.clone())).nth(1).unwrap_or_else(|| panic!("{character:?}: {effects:?}")).sequence;
        assert!(at(&effects, "Belt Weak", power_change(cultist.clone(), "weak")) < second, "{character:?}: {effects:?}");
        assert_eq!(amount(&combat, 0, "weak"), 2, "{character:?}");
    }
}

/// D review m3: a Lagavulin woken in the player's end turn (Combust) opens before the monster
/// turn: ChangeState and its Metallicize reduce follow the discard and precede its move (base:
/// the reduce waited behind the monster moves).
#[test]
fn fuzzfix_d_end_turn_wake_opens_before_the_monster_turn() {
    for character in ALL {
        let mut combat = fixture_in(EncounterId::Lagavulin, 1, character, &[(Combust, 0)], vec![], vec![None; 3], 3);
        combat.step_full(&CombatAction::PlayCard { instance: 1, target: None }).unwrap();
        combat.state.enemies[0].block = 0; // Combust's damage must reach HP to wake it
        let lagavulin = enemy(&combat, 0);
        let effects = end_turn(&mut combat);
        assert!(at(&effects, "Metallicize reduce", power_change(lagavulin.clone(), "metallicize"))
            < at(&effects, "Lagavulin move", moved(lagavulin)), "{character:?}: {effects:?}");
        assert_eq!((combat.state.enemies[0].block, amount(&combat, 0, "metallicize")), (0, 0), "{character:?}");
    }
}
