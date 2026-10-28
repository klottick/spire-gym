use super::super::*;
use crate::content::{CardColor, CardId, CardRarity, CardType, Catalog, EncounterId};
use super::full_runtime::entered_combat;
use model::OwnedCard;

fn id(token: &str) -> CardId {
    Catalog::standard().cards().iter().find(|row| row.id.as_str() == token)
        .unwrap_or_else(|| panic!("missing Colorless power card {token}")).id
}

fn combat(token: &str, upgrades: u32) -> Combat {
    let mut combat = entered_combat(9, 9, |card| {
        *card = permanent_live_card(id(token), card.instance, upgrades);
    });
    combat.state.player.hp = 999;
    combat.state.player.max_hp = 999;
    combat.state.enemies[0].hp = 500;
    combat.state.enemies[0].max_hp = 500;
    combat
}

fn three_enemy_combat(token: &str, upgrades: u32) -> Combat {
    let mut entry = super::m8_shapes::entry(EncounterId::ThreeShapes);
    entry.setup.player.max_energy = 9;
    entry.setup.player.energy = 9;
    entry.setup.player.hp = 999;
    entry.setup.player.max_hp = 999;
    for enemy in &mut entry.setup.enemies {
        enemy.hp = 500;
        enemy.max_hp = 500;
        enemy.block = 0;
    }
    entry.cards = vec![
        permanent_live_card(id(token), CardInstanceId(1), upgrades),
        permanent_live_card(CardId::DefendGreen, CardInstanceId(2), 0),
        permanent_live_card(CardId::DefendGreen, CardInstanceId(3), 0),
    ];
    entry.opening_draw = 3;
    Combat::enter(entry).unwrap()
}

fn add_to_hand(combat: &mut Combat, token: &str, instance: u64, upgrades: u32) {
    let card = permanent_live_card(id(token), CardInstanceId(instance), upgrades);
    combat.state.card_ids_issued.insert(card_state::handle(card.instance));
    combat.state.piles.hand.push(OwnedCard::Full(card));
}

fn add_to_draw(combat: &mut Combat, token: &str, instance: u64, upgrades: u32) {
    let card = permanent_live_card(id(token), CardInstanceId(instance), upgrades);
    combat.state.card_ids_issued.insert(card_state::handle(card.instance));
    combat.state.piles.draw.push(OwnedCard::Full(card));
}

fn play(combat: &mut Combat, instance: u64, target: Option<TargetId>) {
    combat.step_full(&CombatAction::PlayCard { instance, target }).unwrap();
}

fn power_amount(combat: &Combat, name: &str) -> i32 {
    combat.state.player_full_powers.iter().find(|power| power.id.0 == name)
        .map_or(0, |power| power.state.amount)
}

fn enemy_power(combat: &Combat, enemy: usize, name: &str) -> i32 {
    combat.state.enemies[enemy].full_powers.iter().find(|power| power.id.0 == name)
        .map_or(0, |power| power.state.amount)
}

#[test]
fn m15_colorless_power_cards_register_the_exact_supported_family_and_costs() {
    let rows = [
        ("mayhem", CardType::Power, 2, 1, 1, 1),
        ("panache", CardType::Power, 0, 0, 10, 14),
        ("sadistic_nature", CardType::Power, 0, 0, 5, 7),
        ("the_bomb", CardType::Skill, 2, 2, 40, 50),
    ];
    for (token, card_type, base_cost, upgraded_cost, base_magic, upgraded_magic) in rows {
        let definition = Catalog::standard().cards().iter().find(|row| row.id.as_str() == token)
            .unwrap_or_else(|| panic!("missing Colorless power card definition {token}"));
        assert_eq!(definition.color, CardColor::Colorless, "{token} color");
        assert_eq!(definition.rarity, CardRarity::Rare, "{token} rarity");
        assert_eq!(definition.card_type, card_type, "{token} type");
        assert!(!definition.definition_only, "{token} is executable");
        for (upgrades, cost) in [(0, base_cost), (1, upgraded_cost)] {
            let fight = combat(token, upgrades);
            let card = fight.state.piles.hand.iter().filter_map(OwnedCard::full)
                .find(|card| card.id == definition.id).unwrap();
            assert_eq!(cards::visible_cost(&fight.state, card), Some(cost), "{token}+{upgrades} cost");
            assert_eq!(cards::public_values(card).unwrap().magic_number,
                Some(if upgrades==0{base_magic}else{upgraded_magic}), "{token}+{upgrades} displayed value");
        }
    }
}

#[test]
fn m15_colorless_default_state_keeps_legacy_combat_debug_shape() {
    let fight=entered_combat(3,3,|_|{});
    assert!(!format!("{fight:?}").contains("colorless:"),
        "empty typed power state is omitted from the established Combat Debug output");
}

#[test]
fn m15_mayhem_upgrade_changes_base_cost_and_preserves_the_existing_positive_delta() {
    let mut card=card_state::permanent(id("mayhem"),CardInstanceId(90),0);
    card.local.turn_cost=None;
    card.local.temporary_cost=2;
    cards::ironclad::rules::upgrade(&mut card).unwrap();
    assert_eq!(card.upgrades,1);
    assert_eq!(card.local.mutable_combat_cost,Some(1));
    assert_eq!(card.local.turn_cost,Some(3),"temporary positive turn-cost delta survives the base upgrade");
    assert_eq!(card.local.temporary_cost,0,"upgraded cost state uses the shared base/turn representation");
}

#[test]
fn m15_panache_counts_card_uses_not_hits_and_resets_at_start_turn() {
    let mut fight = combat("panache", 0);
    play(&mut fight, 3, None);
    assert_eq!(power_amount(&fight, "panache"), 10);

    add_to_hand(&mut fight, "twin_strike", 50, 0);
    for instance in 51..=54 { add_to_hand(&mut fight, "defend_g", instance, 0); }
    play(&mut fight, 50, Some(TargetId(1)));
    let after_two_hits = fight.state.enemies[0].hp;
    for instance in 51..=53 { play(&mut fight, instance, None); }
    assert_eq!(fight.state.enemies[0].hp, after_two_hits,
        "four later card uses do not trigger Panache despite Twin Strike's two hits");
    play(&mut fight, 54, None);
    assert_eq!(fight.state.enemies[0].hp, after_two_hits - 10,
        "the fifth later card use queues one all-enemy Thorns hit");

    let mut upgraded = combat("panache", 1);
    play(&mut upgraded, 3, None);
    assert_eq!(power_amount(&upgraded, "panache"), 14);
    add_to_hand(&mut upgraded, "defend_g", 60, 0);
    play(&mut upgraded, 60, None);
    upgraded.step_full(&CombatAction::EndTurn).unwrap();
    for instance in 61..=65 { add_to_hand(&mut upgraded, "defend_g", instance, 0); }
    let before = upgraded.state.enemies[0].hp;
    for instance in 61..=64 { play(&mut upgraded, instance, None); }
    assert_eq!(upgraded.state.enemies[0].hp, before, "start turn resets remaining uses to five");
    play(&mut upgraded, 65, None);
    assert_eq!(upgraded.state.enemies[0].hp, before - 14);
}

#[test]
fn m15_mayhem_autoplays_the_top_card_with_a_captured_random_target_and_no_payment_or_exhaust() {
    let mut fight = three_enemy_combat("mayhem", 0);
    play(&mut fight, 1, None);
    add_to_draw(&mut fight, "bash", 50, 0);
    fight.state.player.max_energy = 1;
    fight.state.player.energy = 1;
    let before = fight.state.enemies.iter().map(|enemy| enemy.hp).collect::<Vec<_>>();

    fight.step_full(&CombatAction::EndTurn).unwrap();

    let damage = fight.state.enemies.iter().zip(before)
        .map(|(enemy, hp)| hp - enemy.hp).collect::<Vec<_>>();
    assert_eq!(damage.iter().filter(|amount| **amount > 0).count(), 1,
        "Mayhem captures one living target for the selected top card");
    assert_eq!(damage.iter().sum::<i32>(), 8, "the actual Bash payload resolves");
    assert_eq!(fight.state.player.energy, 1, "the two-cost child consumes no energy");
    assert!(fight.state.piles.discard.iter().any(|card| card.handle().as_str() == "50"),
        "Mayhem does not impose Havoc's exhaust disposition");
    assert!(!fight.state.piles.exhaust.iter().any(|card| card.handle().as_str() == "50"));
}

#[test]
fn m15_mayhem_keeps_the_shared_autoplay_frame_across_a_child_selection() {
    let mut fight=three_enemy_combat("mayhem",0);
    play(&mut fight,1,None);
    for instance in 51..=53{add_to_draw(&mut fight,"defend_g",instance,0);}
    add_to_draw(&mut fight,"acrobatics",50,0);
    fight.state.player.max_energy=1;
    fight.state.player.energy=1;

    fight.step_full(&CombatAction::EndTurn).unwrap();

    assert_eq!(fight.state.phase,Phase::Selection);
    assert_eq!(fight.state.play_frames.last().map(|frame|frame.child.as_str()),Some("50"));
    let chosen=fight.state.pending_selection.as_ref().unwrap().candidates[0].as_str().parse().unwrap();
    fight.step_full(&CombatAction::SelectCard{instance:chosen}).unwrap();
    assert_eq!(fight.state.phase,Phase::Decision);
    assert!(fight.state.play_frames.is_empty());
    assert!(fight.state.piles.discard.iter().any(|card|card.handle().as_str()=="50"));
}

#[test]
fn m15_sadistic_nature_triggers_per_accepted_debuff_after_each_targets_artifact_check() {
    for (upgrades, damage) in [(0, 5), (1, 7)] {
        let mut fight = three_enemy_combat("sadistic_nature", upgrades);
        play(&mut fight, 1, None);
        for enemy in 0..2 {
            fight.state.enemies[enemy].full_powers.push(FullPowerEntry {
                id: FullPowerId("artifact"),
                state: PowerState { amount: 1, skip_first: false, monster_source_grace: false },
            });
        }
        let first = Actor::Enemy(fight.state.enemies[0].instance_id.clone());
        let second = Actor::Enemy(fight.state.enemies[1].instance_id.clone());
        fight.enqueue_native_effects(vec![
            Effect { source: Source::Actor(Actor::Player), target: Some(first.clone()),
                kind: EffectKind::ApplyFullPower { id: FullPowerId("weak"), amount: 1 } },
            Effect { source: Source::Actor(Actor::Player), target: Some(second.clone()),
                kind: EffectKind::ApplyFullPower { id: FullPowerId("weak"), amount: 1 } },
        ], None).unwrap();
        assert_eq!(fight.state.enemies[0].hp, 500, "first target Artifact blocks the debuff and reaction");
        assert_eq!(fight.state.enemies[1].hp, 500, "second target Artifact is independent");
        assert_eq!(enemy_power(&fight, 0, "artifact"), 0);
        assert_eq!(enemy_power(&fight, 1, "artifact"), 0);

        fight.enqueue_native_effects(vec![
            Effect { source: Source::Actor(Actor::Player), target: Some(first.clone()),
                kind: EffectKind::ApplyFullPower { id: FullPowerId("weak"), amount: 1 } },
            Effect { source: Source::Actor(Actor::Player), target: Some(first.clone()),
                kind: EffectKind::ApplyFullPower { id: FullPowerId("vulnerable"), amount: 1 } },
            Effect { source: Source::Actor(Actor::Player), target: Some(first),
                kind: EffectKind::ApplyFullPower { id: FullPowerId("shackled"), amount: 1 } },
        ], None).unwrap();
        assert_eq!(fight.state.enemies[0].hp, 500 - 2 * damage,
            "each accepted ordinary debuff reacts once while Shackled restoration is excluded");
        assert_eq!(fight.state.enemies[1].hp, 500, "reactions remain target-local");
    }
}

#[test]
fn m15_the_bomb_keeps_staggered_three_turn_timers_and_upgrade_payloads() {
    let mut fight = combat("the_bomb", 0);
    play(&mut fight, 3, None);
    fight.step_full(&CombatAction::EndTurn).unwrap();
    assert_eq!(fight.state.enemies[0].hp, 500, "first end turn reduces 3 to 2");

    add_to_hand(&mut fight, "the_bomb", 50, 1);
    play(&mut fight, 50, None);
    fight.step_full(&CombatAction::EndTurn).unwrap();
    assert_eq!(fight.state.enemies[0].hp, 500, "second end turn reduces the first to 1 and the second to 2");

    fight.step_full(&CombatAction::EndTurn).unwrap();
    assert_eq!(fight.state.enemies[0].hp, 460, "the base cast detonates at its own third end turn for 40");
    fight.step_full(&CombatAction::EndTurn).unwrap();
    assert_eq!(fight.state.enemies[0].hp, 410, "the staggered upgraded cast later detonates for 50");
}

#[test]
fn m15_the_bomb_terminal_cancels_other_owned_detonations() {
    let mut fight = combat("the_bomb", 0);
    play(&mut fight, 3, None);
    add_to_hand(&mut fight, "the_bomb", 50, 1);
    play(&mut fight, 50, None);
    fight.state.enemies[0].hp = 40;
    for _ in 0..3 { fight.step_full(&CombatAction::EndTurn).unwrap(); }
    assert_eq!(fight.state.outcome, Some(Outcome::Victory));
    assert!(fight.state.pending.is_empty(), "terminal settlement cancels queued later Bomb work");
    assert!(fight.state.colorless.bombs.is_empty(), "terminal settlement releases each retained Bomb owner");
}

#[test]
fn m15_bomb_detonates_before_the_enemy_can_deliver_lethal_damage() {
    let mut fight=combat("the_bomb",0);
    play(&mut fight,3,None);
    fight.state.colorless.bombs[0].remaining=1;
    fight.state.enemies[0].hp=40;
    fight.state.player.hp=1;
    add_to_hand(&mut fight,"burn",50,0);
    fight.step_full(&CombatAction::EndTurn).unwrap();
    assert_eq!(fight.state.outcome,Some(Outcome::Victory));
    assert_eq!(fight.state.player.hp,1);
}

#[test]
fn m15_sadistic_callback_appends_damage_after_already_queued_actions() {
    let mut fight=combat("sadistic_nature",0);
    play(&mut fight,3,None);
    let target=Actor::Enemy(fight.state.enemies[0].instance_id.clone());
    fight.state.pending.push_back(Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::GainBlock(3)});
    let application=Effect{source:Source::Actor(Actor::Player),target:Some(target.clone()),kind:EffectKind::ApplyFullPower{id:FullPowerId("weak"),amount:1}};
    powers::apply_effect(&mut fight.state,&application).unwrap();
    assert_eq!(fight.state.pending.front().unwrap().kind,EffectKind::GainBlock(3));
    assert_eq!(fight.state.pending.back().unwrap().kind,EffectKind::Damage(DamagePayload::Thorns(5)));
    assert_eq!(enemy_power(&fight,0,"weak"),1);
    // Notification does not depend on a stack amount changing (Strength clamp).
    fight.state.enemies[0].full_powers.push(FullPowerEntry{id:FullPowerId("strength"),state:PowerState{amount:-999,skip_first:false,monster_source_grace:false}});
    let application=Effect{kind:EffectKind::ApplyFullPower{id:FullPowerId("strength"),amount:-1},..application};
    let before=fight.state.pending.len();
    powers::apply_effect(&mut fight.state,&application).unwrap();
    assert_eq!(fight.state.pending.len(),before+1);
}

#[test]
fn m15_public_delayed_bomb_timers_are_visible_independently_with_base_plus_identity(){
    fn timers(combat:&Combat)->Vec<(&'static str,i32)>{combat.public_full().unwrap().player.powers.into_iter().filter(|power|matches!(power.id.0,"the_bomb"|"the_bomb_plus")).map(|power|(power.id.0,power.amount)).collect()}
    let mut fight=combat("the_bomb",0);play(&mut fight,3,None);assert_eq!(timers(&fight),vec![("the_bomb",3)]);
    fight.step_full(&CombatAction::EndTurn).unwrap();add_to_hand(&mut fight,"the_bomb",50,1);play(&mut fight,50,None);
    assert_eq!(timers(&fight),vec![("the_bomb",2),("the_bomb_plus",3)]);let before=fight.snapshot();assert_eq!(fight.public_full().unwrap(),fight.public_full().unwrap());assert_eq!(fight.snapshot(),before);
    fight.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(timers(&fight),vec![("the_bomb",1),("the_bomb_plus",2)]);
    fight.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(timers(&fight),vec![("the_bomb_plus",1)]);assert_eq!(fight.state.enemies[0].hp,460);
    fight.step_full(&CombatAction::EndTurn).unwrap();assert!(timers(&fight).is_empty());assert_eq!(fight.state.enemies[0].hp,410);
    let mut reverse=combat("the_bomb",1);play(&mut reverse,3,None);add_to_hand(&mut reverse,"the_bomb",50,0);play(&mut reverse,50,None);
    assert_eq!(timers(&reverse),vec![("the_bomb_plus",3),("the_bomb",3)],"independent public timers preserve their callback creation order across variants");
}
#[test]
fn m15_public_delayed_panache_countdown_is_visible_and_separate_from_damage(){
    fn visible(combat:&Combat,name:&str)->Option<i32>{combat.public_full().unwrap().player.powers.iter().find(|power|power.id.0==name).map(|power|power.amount)}
    let mut fight=combat("panache",1);play(&mut fight,3,None);assert_eq!(visible(&fight,"panache"),Some(14));assert_eq!(visible(&fight,"panache_countdown"),Some(5));
    for (instance,expected) in [(50,4),(51,3),(52,2),(53,1),(54,5)]{add_to_hand(&mut fight,"deflect",instance,0);play(&mut fight,instance,None);assert_eq!(visible(&fight,"panache_countdown"),Some(expected));assert_eq!(visible(&fight,"panache"),Some(14));}
    assert_eq!(fight.state.enemies[0].hp,486);fight.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(visible(&fight,"panache_countdown"),Some(5));
}
#[test]
fn m15_public_delayed_invalid_timer_rejects_without_alias_or_mutation(){
    let mut fight=combat("the_bomb",0);play(&mut fight,3,None);fight.state.colorless.bombs[0].damage=41;let before=fight.snapshot();
    assert_eq!(fight.public_full(),Err(crate::run::RunError::InvalidAction));assert_eq!(fight.snapshot(),before);
}
