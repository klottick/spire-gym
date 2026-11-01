//! Deus Ex Machina is Unplayable: only its on-draw trigger acts. A copy that reaches the hand without
//! a draw is simply unplayable; legal actions and the public view stay valid.
use super::super::*;
use crate::{content::{CardId, Character, EncounterId}, items::CombatItemContext};

fn watcher_entry(mut entry: CombatEntry, cards: Vec<LiveCard>) -> Combat {
    entry.setup.character = Character::Watcher;
    entry.setup.player.hp = 300;
    entry.setup.player.max_hp = 300;
    entry.cards = cards;
    entry.opening_draw = 0;
    let mut items = Some(CombatItemContext::new(vec![], vec![None; 3]).with_m25_profile());
    Combat::enter_with_item_context(entry, &mut items).expect("Watcher combat enters")
}

fn deus_unplayable_in_hand(c: &Combat, instance: u64) {
    let hand = c.state.piles.hand.iter().map(|card| { let live = card.full().unwrap(); (live.instance.0, live.id) }).collect::<Vec<_>>();
    let legal = c.legal_actions_full().unwrap_or_else(|error| panic!("legal actions with Deus Ex Machina in hand {hand:?}: {error:?}"));
    assert!(!legal.iter().any(|action| matches!(action, CombatAction::PlayCard { instance: i, .. } if *i == instance)));
    assert!(legal.contains(&CombatAction::EndTurn));
    assert!(c.public_full().is_ok());
}

#[test]
fn m25_deus_ex_machina_in_hand_without_draw_is_unplayable() {
    for upgrades in 0..=1 {
        let mut c = watcher_entry(super::m8_shapes::entry(EncounterId::OrbWalker), vec![
            m25_watcher_live_card(CardId::DeusExMachina, CardInstanceId(1), upgrades).unwrap(),
            m25_watcher_live_card(CardId::DefendPurple, CardInstanceId(2), 0).unwrap(),
        ]);
        let deus = c.state.piles.draw.remove(c.state.piles.draw.iter().position(|card| card.full().unwrap().instance == CardInstanceId(1)).unwrap());
        c.state.piles.hand.push(deus);
        piles::refresh_hand_values(&mut c.state);
        deus_unplayable_in_hand(&c, 1);
    }
}

#[test]
fn m25_stasis_returned_deus_ex_machina_is_unplayable() {
    let mut c = watcher_entry(super::m6_ordinary::entry(EncounterId::BronzeAutomaton), vec![m25_watcher_live_card(CardId::DeusExMachina, CardInstanceId(1), 0).unwrap()]);
    let boss = Actor::Enemy(c.state.enemies[0].instance_id.clone());
    c.enqueue_native_effects(vec![Effect { source: Source::Actor(boss.clone()), target: Some(boss), kind: EffectKind::EnemyMove }], None).unwrap();
    assert_eq!(c.state.enemies.len(), 3, "Bronze Automaton spawned its orbs");
    super::m6_stasis::capture(&mut c, 1);
    let orb = c.state.enemies[1].instance_id.clone();
    assert_eq!(c.state.stasis[&orb].full().unwrap().id, CardId::DeusExMachina);
    super::m5_elites::hit(&mut c, 1, 100);
    assert!(c.state.piles.hand.iter().any(|card| card.full().unwrap().instance == CardInstanceId(1)), "returned to hand without a draw");
    deus_unplayable_in_hand(&c, 1);
}
