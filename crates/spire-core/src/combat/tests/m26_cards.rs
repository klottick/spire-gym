//! M26 Lane A card and curse behavior (design §4.2–4.3; plan Task A1).
//! Expected values are literal source values. Disclosed policy choices are
//! named in the test comments that bind them (F8–F11).
use super::super::*;
use crate::{content::{CardId, Character, EncounterId}, items::CombatItemContext};
use crate::combat::model::OwnedCard;
use CardId::*;

fn card(id: CardId, instance: u64, upgrades: u32) -> LiveCard {
    complete_live_card(id, CardInstanceId(instance), upgrades)
        .unwrap_or_else(|error| panic!("complete card {id:?}+{upgrades}: {error:?}"))
}

/// M0 §6 stripped matrix shape and seed, using Jaw Worm's supported Act 1 setup.
/// m8_shapes::entry currently fixes depth 3, where Jaw Worm is rejected.
fn fight(cards: Vec<LiveCard>, opening_draw: usize) -> Combat {
    let mut rng = crate::run::RunRng::new([41; 32], rng::Partition::Test, 0);
    let mut entry = enemies::setup(EncounterId::JawWorm, 1, 0, &mut rng, enemies::SetupContext {
        player: PlayerState {
            hp: 80, max_hp: 80, block: 0, energy: 10, max_energy: 10,
            draw_per_turn: 5, powers: vec![],
        },
        player_powers: vec![], character: Character::Ironclad,
        cards: vec![permanent_live_card(Defend, CardInstanceId(99), 0)],
        opening_draw: 1, encounter_visit: 0,
    }).unwrap_or_else(|error| panic!("Jaw Worm setup: {error:?}"));
    entry.cards = cards;
    entry.opening_draw = opening_draw;
    let mut items = Some(CombatItemContext::new(vec![], vec![None; 3]).with_complete_cards());
    Combat::enter_with_item_context(entry, &mut items)
        .unwrap_or_else(|error| panic!("complete-card combat entry: {error:?}"))
}

fn play(combat: &mut Combat, instance: u64, target: Option<u64>) {
    combat.step_full(&CombatAction::PlayCard { instance, target: target.map(TargetId) })
        .unwrap_or_else(|error| panic!("play {instance}: {error:?}; fault={:?}", combat.fault_record()));
}

fn hand_card(combat: &Combat, instance: u64) -> &LiveCard {
    combat.state.piles.hand.iter().find(|owned| owned.handle() == card_state::handle(CardInstanceId(instance)))
        .and_then(OwnedCard::full).expect("card is in hand")
}

fn owned_card(combat: &Combat, instance: u64) -> &LiveCard {
    combat.state.piles.all(combat.state.in_play.as_ref())
        .filter_map(OwnedCard::full).find(|live| live.instance == CardInstanceId(instance))
        .expect("card remains in a combat pile")
}

fn in_pile(pile: &[OwnedCard], instance: u64, id: CardId) -> bool {
    pile.iter().any(|owned| owned.handle() == card_state::handle(CardInstanceId(instance))
        && owned.full().is_some_and(|live| live.id == id))
}

fn player_power(combat: &Combat, id: &'static str) -> i32 {
    combat.state.player_full_powers.iter().find(|power| power.id.0 == id)
        .map_or(0, |power| power.state.amount)
}

fn add_draw_card(combat: &mut Combat, id: CardId, instance: u64) {
    let owned = OwnedCard::Full(card(id, instance, 0));
    assert!(combat.state.card_ids_issued.insert(owned.handle()), "fresh draw identity");
    combat.state.piles.draw.push(owned);
    piles::refresh_hand_values(&mut combat.state);
}

#[derive(Clone, Copy)]
enum FixturePile { Hand, Draw, Exhaust }

fn move_card(combat: &mut Combat, instance: u64, destination: FixturePile) {
    let handle = card_state::handle(CardInstanceId(instance));
    let take = |pile: &mut Vec<OwnedCard>| pile.iter().position(|owned| owned.handle() == handle)
        .map(|index| pile.remove(index));
    let owned = take(&mut combat.state.piles.hand)
        .or_else(|| take(&mut combat.state.piles.draw))
        .or_else(|| take(&mut combat.state.piles.discard))
        .or_else(|| take(&mut combat.state.piles.exhaust))
        .expect("fixture card has one visible owner");
    match destination {
        FixturePile::Hand => combat.state.piles.hand.push(owned),
        FixturePile::Draw => combat.state.piles.draw.push(owned),
        FixturePile::Exhaust => combat.state.piles.exhaust.push(owned),
    }
    piles::refresh_hand_values(&mut combat.state);
}

fn card_in(combat: &Combat, id: CardId) -> Vec<(u64, CombatCardRef)> {
    combat.state.piles.all(combat.state.in_play.as_ref())
        .filter_map(OwnedCard::full).filter(|live| live.id == id)
        .map(|live| (live.instance.0, live.origin.clone())).collect()
}

#[test]
fn m26_cards_colourless_values() {
    // Design §4.2 literal costs and effects, both faces.
    let rows = [
        (BandageUp, 0, 4, 6), (Apparition, 1, 1, 1), (Bite, 1, 7, 8),
        (Jax, 0, 2, 3), (RitualDagger, 1, 15, 15),
    ];
    for (id, cost, base, upgraded) in rows {
        for (upgrades, value) in [(0, base), (1, upgraded)] {
            let mut combat = fight(vec![card(id, 1, upgrades)], 1);
            combat.state.player.hp = 70;
            let public_card = hand_card(&combat, 1).clone();
            assert_eq!(cards::visible_cost(&combat.state, &public_card), Some(cost), "{id:?}+{upgrades} cost");
            let enemy_before = combat.state.enemies[0].hp;
            match id {
                BandageUp | Apparition | Jax => play(&mut combat, 1, None),
                Bite | RitualDagger => play(&mut combat, 1, Some(1)),
                _ => unreachable!(),
            }
            match id {
                BandageUp => {
                    assert_eq!(combat.state.player.hp, 70 + value, "Bandage Up heals {value}");
                    assert!(in_pile(&combat.state.piles.exhaust, 1, id), "Bandage Up exhausts");
                }
                Apparition => {
                    assert_eq!(player_power(&combat, "intangible"), 1, "Apparition grants 1 Intangible");
                    assert!(in_pile(&combat.state.piles.exhaust, 1, id), "Apparition exhausts");
                    assert_eq!(public_card.local.ethereal, upgrades == 0, "base Apparition alone is Ethereal");
                }
                Bite => {
                    assert_eq!(combat.state.enemies[0].hp, enemy_before - value, "Bite damage {value}");
                    assert_eq!(combat.state.player.hp, if upgrades == 0 { 72 } else { 73 }, "Bite healing 2 → 3");
                    assert!(in_pile(&combat.state.piles.discard, 1, id), "Bite does not exhaust");
                }
                Jax => {
                    assert_eq!(combat.state.player.hp, 67, "J.A.X. loses 3 HP");
                    assert_eq!(player_power(&combat, "strength"), value, "J.A.X. Strength {value}");
                    assert!(in_pile(&combat.state.piles.discard, 1, id), "J.A.X. does not exhaust");
                }
                RitualDagger => {
                    assert_eq!(enemy_before - combat.state.enemies[0].hp, 15, "Ritual Dagger deals its misc 15");
                    assert!(in_pile(&combat.state.piles.exhaust, 1, id), "Ritual Dagger exhausts");
                    assert_eq!(owned_card(&combat, 1).misc, 15, "non-Fatal Ritual Dagger keeps misc 15");
                }
                _ => unreachable!(),
            }
        }
    }
    // F7: Fatal growth stays on the same Ritual Dagger family (+3 base, +5 upgraded).
    for (upgrades, grown) in [(0, 18), (1, 20)] {
        let mut combat = fight(vec![card(RitualDagger, 1, upgrades)], 1);
        combat.state.enemies[0].hp = 1;
        play(&mut combat, 1, Some(1));
        assert_eq!(owned_card(&combat, 1).misc, grown, "Fatal raises Ritual Dagger misc to {grown}");
    }
}

#[test]
fn m26_cards_curse_rules() {
    // Design §4.3: every admitted curse is unplayable.
    let curses = [Clumsy, CurseOfTheBell, Decay, Doubt, Injury, Necronomicurse,
        Normality, Pain, Parasite, Regret, Shame, Writhe, AscendersBane];
    for id in curses {
        let combat = fight(vec![card(id, 1, 0)], 1);
        assert!(!combat.legal_actions_full().unwrap().iter().any(|action|
            matches!(action, CombatAction::PlayCard { instance: 1, .. })), "{id:?} is absent from the action mask");
    }
    for id in [Clumsy, AscendersBane] {
        let mut combat = fight(vec![card(id, 1, 0)], 1);
        combat.step_full(&CombatAction::EndTurn).unwrap();
        assert!(in_pile(&combat.state.piles.exhaust, 1, id), "Ethereal {id:?} exhausts");
    }
    let writhe = fight(vec![card(Defend, 1, 0), card(Writhe, 2, 0)], 1);
    assert!(writhe.state.piles.hand.iter().any(|owned| owned.full().is_some_and(|live| live.id == Writhe)),
        "Innate Writhe is in the opening hand");

    let mut held = fight(vec![card(Decay, 1, 0), card(Doubt, 2, 0), card(Shame, 3, 0),
        card(Regret, 4, 0), card(Injury, 5, 0)], 5);
    held.state.player.block = 1;
    let work = status::end_turn_work(&held.state);
    assert_eq!(work.len(), 4, "only Decay, Doubt, Shame and Regret auto-use");
    held.enqueue_native_effects(work, None).unwrap();
    assert_eq!(held.state.player.hp, 74, "Decay 2 loses 1 after Block; Regret loses hand size 5");
    assert_eq!(player_power(&held, "weak"), 1, "Doubt grants 1 Weak");
    assert_eq!(player_power(&held, "frail"), 1, "Shame grants 1 Frail");

    let unchanged = fight(vec![card(Injury, 1, 0), card(CurseOfTheBell, 2, 0)], 2);
    assert!(status::end_turn_work(&unchanged.state).is_empty(), "Injury and Curse of the Bell do nothing in combat");
}

#[test]
fn m26_cards_end_of_turn_order() {
    // Disclosed decision F8: one Burn-owner packet per card in hand order,
    // after PlayerPreEndTurnCards power work and before Ethereal/discard cleanup.
    let mut doubt = card(Doubt, 4, 0);
    doubt.local.self_retain = true;
    let mut combat = fight(vec![card(Burn, 1, 0), card(Decay, 2, 0), card(Burn, 3, 0), doubt,
        card(Shame, 5, 0), card(Regret, 6, 0), card(Clumsy, 7, 0)], 7);
    for instance in [1, 2, 3, 4, 5, 6, 7] { move_card(&mut combat, instance, FixturePile::Hand); }
    combat.state.player_full_powers.push(FullPowerEntry { id: FullPowerId("metallicize"),
        state: PowerState { amount: 2, skip_first: false, monster_source_grace: false } });
    let source_order = status::end_turn_work(&combat.state).iter().filter_map(|effect| {
        let Source::Card(handle) = &effect.source else { return None };
        combat.state.piles.hand.iter().find(|owned| owned.handle() == *handle)
            .and_then(OwnedCard::full).map(|live| live.id)
    }).collect::<Vec<_>>();
    assert_eq!(source_order, vec![Burn, Decay, Burn, Doubt, Shame, Regret],
        "hand-order auto-use, interleaved with Burn; retained Doubt still triggers");

    // Ask the shared EndPlayerTurn owner to schedule the actual sequence.
    turns::apply_effect(&mut combat.state, &Effect { source: Source::None, target: None,
        kind: EffectKind::EndPlayerTurn }).unwrap();
    assert_eq!(combat.state.pending.front().map(|effect| &effect.kind),
        Some(&EffectKind::Hook(Hook::PlayerPreEndTurnCards)));
    engine::dispatch_one(&mut combat.state);
    assert_eq!(combat.state.pending.front().map(|effect| &effect.kind), Some(&EffectKind::GainBlock(2)),
        "Metallicize work is queued before the curse packets");
    engine::dispatch_one(&mut combat.state);
    assert_eq!(combat.state.player.block, 2, "PlayerPreEndTurnCards power work resolves first");
    let queued_sources = combat.state.pending.iter().take(6).filter_map(|effect| {
        let Source::Card(handle) = &effect.source else { return None };
        combat.state.piles.hand.iter().find(|owned| owned.handle() == *handle)
            .and_then(OwnedCard::full).map(|live| live.id)
    }).collect::<Vec<_>>();
    assert_eq!(queued_sources, vec![Burn, Decay, Burn, Doubt, Shame, Regret]);
    let discard = combat.state.pending.iter().position(|effect| effect.kind == EffectKind::DiscardHand).unwrap();
    assert!(discard > 0, "Ethereal exhaust and ordinary discard are after the curse packets");
    assert!(combat.state.piles.hand.iter().any(|owned| owned.full().is_some_and(|live| live.id == Clumsy)),
        "Ethereal remains in hand until cleanup");
    assert!(!combat.state.hook_events.iter().any(|event| event.hook == Hook::CardPlayed),
        "end-of-turn auto-use emits no CardPlayed");

    // Retain an actual curse through the End Turn selection; it still auto-uses.
    // pre-M26-review: the fixture had no draw pile, so the discarded curses were reshuffled and
    // redrawn. Five Defends (8-12) now stay in the draw pile for the next turn.
    let mut integrated = fight(vec![card(Burn, 1, 0), card(Decay, 2, 0), card(Burn, 3, 0),
        card(Doubt, 4, 0), card(Shame, 5, 0), card(Regret, 6, 0), card(Clumsy, 7, 0),
        card(Defend, 8, 0), card(Defend, 9, 0), card(Defend, 10, 0), card(Defend, 11, 0), card(Defend, 12, 0)], 7);
    for instance in [1, 2, 3, 4, 5, 6, 7] { move_card(&mut integrated, instance, FixturePile::Hand); }
    for instance in [8, 9, 10, 11, 12] { move_card(&mut integrated, instance, FixturePile::Draw); }
    integrated.state.player_full_powers.push(FullPowerEntry { id: FullPowerId("retain_cards"),
        state: PowerState { amount: 1, skip_first: false, monster_source_grace: false } });
    integrated.step_full(&CombatAction::EndTurn).unwrap();
    assert!(integrated.state.pending_selection.as_ref().unwrap().candidates
        .contains(&card_state::handle(CardInstanceId(4))));
    integrated.step_full(&CombatAction::SelectCard { instance: 4 }).unwrap();
    integrated.step_full(&CombatAction::ConfirmSelection).unwrap();
    // pre-M26-review (final review v1 I2; M25 R4 generalized). Original lines:
    // assert!(integrated.state.piles.hand.iter().any(|owned| owned.full().is_some_and(|live| live.id == Doubt)),
    //     "retained Doubt remains in hand");
    assert!(!integrated.state.piles.hand.iter().any(|owned| owned.full().is_some_and(|live| live.id == Doubt)),
        "a retained Doubt discards itself, like Burn");
    assert!(in_pile(&integrated.state.piles.discard, 4, Doubt), "the retained Doubt is in the discard pile");
    assert!(in_pile(&integrated.state.piles.exhaust, 7, Clumsy), "Ethereal cleanup exhausts Clumsy");
    assert!(!integrated.state.hook_events.iter().any(|event| event.hook == Hook::CardPlayed));
}

#[test]
fn m26_normality_counts_autoplay_and_late_draw() {
    // Normality counts CardPlayed uses: Double Tap repeat.
    let mut double_tap = fight(vec![card(Normality, 1, 0), card(DoubleTap, 2, 0),
        card(Strike, 3, 0), card(Defend, 4, 0)], 4);
    play(&mut double_tap, 2, None);
    play(&mut double_tap, 3, Some(1));
    let double_tap_counts = double_tap.state.silent.played_this_turn.len() == 3
        && !double_tap.legal_actions_full().unwrap().iter().any(|action|
            matches!(action, CombatAction::PlayCard { instance: 4, .. }));

    // Normality counts the repeated Skill supplied by Burst.
    let mut burst = fight(vec![card(Normality, 1, 0), card(Burst, 2, 0),
        card(Defend, 3, 0), card(Defend, 4, 0)], 4);
    play(&mut burst, 2, None);
    play(&mut burst, 3, None);
    let burst_counts = burst.state.silent.played_this_turn.len() == 3
        && !burst.legal_actions_full().unwrap().iter().any(|action|
            matches!(action, CombatAction::PlayCard { instance: 4, .. }));

    // Normality counts the top card actually played by Havoc.
    let mut havoc = fight(vec![card(Normality, 1, 0), card(Havoc, 2, 0),
        card(Defend, 3, 0), card(Defend, 4, 0)], 4);
    add_draw_card(&mut havoc, Defend, 5);
    play(&mut havoc, 2, None);
    play(&mut havoc, 3, None);
    let havoc_counts = havoc.state.silent.played_this_turn.len() == 3
        && !havoc.legal_actions_full().unwrap().iter().any(|action|
            matches!(action, CombatAction::PlayCard { instance: 4, .. }));

    // Disclosed decision F9 counting: Mayhem's real next-turn top-card autoplay is a play.
    let mut mayhem = fight(vec![card(Mayhem, 1, 0), card(Normality, 2, 0),
        card(Defend, 3, 0), card(Defend, 4, 0), card(Defend, 5, 0)], 5);
    add_draw_card(&mut mayhem, Strike, 90);
    mayhem.state.player_full_powers.push(FullPowerEntry { id: FullPowerId("retain_cards"),
        state: PowerState { amount: 1, skip_first: false, monster_source_grace: false } });
    play(&mut mayhem, 1, None);
    mayhem.step_full(&CombatAction::EndTurn).unwrap();
    assert!(mayhem.state.pending_selection.as_ref().unwrap().candidates
        .contains(&card_state::handle(CardInstanceId(2))), "retain selection can keep Normality");
    mayhem.step_full(&CombatAction::SelectCard { instance: 2 }).unwrap();
    mayhem.step_full(&CombatAction::ConfirmSelection).unwrap();
    assert_eq!(mayhem.state.silent.played_this_turn.len(), 1, "Mayhem's Strike autoplay starts the next turn's count");
    let manual = mayhem.state.piles.hand.iter().filter_map(OwnedCard::full)
        .filter(|live| live.id == Defend).map(|live| live.instance.0).collect::<Vec<_>>();
    assert!(manual.len() >= 3, "Mayhem's turn has at least three manual Defends in hand");
    play(&mut mayhem, manual[0], None);
    play(&mut mayhem, manual[1], None);
    let mayhem_counts = mayhem.state.silent.played_this_turn.len() == 3
        && !mayhem.legal_actions_full().unwrap().iter().any(|action|
            matches!(action, CombatAction::PlayCard { instance, .. } if *instance == manual[2]));

    // Drawing Normality as the third play's payload blocks the next manual play.
    let mut late = fight(vec![card(Defend, 1, 0), card(Defend, 2, 0),
        card(FlashOfSteel, 3, 0), card(Defend, 4, 0)], 4);
    add_draw_card(&mut late, Normality, 90);
    play(&mut late, 1, None);
    play(&mut late, 2, None);
    play(&mut late, 3, Some(1));
    let late_draw = late.state.silent.played_this_turn.len() == 3
        && late.state.piles.hand.iter().any(|owned| owned.full().is_some_and(|live| live.id == Normality))
        && !late.legal_actions_full().unwrap().iter().any(|action|
            matches!(action, CombatAction::PlayCard { instance: 4, .. }));

    assert_eq!((double_tap_counts, burst_counts, havoc_counts, mayhem_counts, late_draw),
        (true, true, true, true, true), "all autoplay/duplicate plays count; fourth play and late-draw play are blocked");
}

#[test]
fn m26_pain_stacks_and_kills_once() {
    // Disclosed decision F10: Pain reacts to every other CardPlayed; both stacks
    // resolve through ordinary HP-loss damage and terminal death work.
    let mut combat = fight(vec![card(Pain, 1, 0), card(Pain, 2, 0), card(Defend, 3, 0)], 3);
    combat.state.player.hp = 2;
    play(&mut combat, 3, None);
    assert_eq!(combat.state.player.hp, 0, "two Pain stacks each lose 1 HP");
    assert_eq!(combat.state.outcome, Some(Outcome::Death), "Pain killing the player is normal combat Death");
    assert_eq!(combat.state.hook_events.iter().filter(|event|
        event.hook == Hook::Death && event.target == Some(Actor::Player)).count(), 1,
        "the lethal Pain stack produces one player Death hook");
    assert!(combat.fault_record().is_none(), "Pain death is not a combat fault");
}

#[test]
fn m26_necronomicurse_full_hand_and_repeat() {
    // Disclosed decision F11: the permanent source stays in Exhaust; a fresh
    // base copy redirects to Discard when the hand is already at its cap of 10.
    let mut full = fight((1..=10).map(|instance| card(Defend, instance, 0))
        .chain([card(Necronomicurse, 11, 0)]).collect(), 10);
    move_card(&mut full, 11, FixturePile::Exhaust);
    while full.state.piles.hand.len() < 10 {
        let instance = 100 + full.state.piles.hand.len() as u64;
        let owned = OwnedCard::Full(card(Defend, instance, 0));
        assert!(full.state.card_ids_issued.insert(owned.handle()));
        full.state.piles.hand.push(owned);
    }
    let permanent_card = owned_card(&full, 11).clone();
    let mut work = Vec::new();
    cards::curses::on_exhaust(&full.state, &permanent_card, &mut work);
    assert_eq!(work.len(), 1, "exhausting Necronomicurse requests one copy");
    full.enqueue_native_effects(work, None).unwrap();
    assert_eq!(full.state.piles.hand.len(), 10, "full hand stays at its cap");
    assert!(in_pile(&full.state.piles.exhaust, 11, Necronomicurse), "permanent Necronomicurse remains exhausted");
    let returned = full.state.piles.discard.iter().find_map(OwnedCard::full)
        .filter(|live| live.id == Necronomicurse).expect("full-hand return goes to discard");
    assert!(matches!(&returned.origin, CombatCardRef::CombatLocal(_)), "returned copy is combat-local");
    assert_eq!(returned.upgrades, 0, "returned copy is base");

    // Fiend Fire exhausts the permanent instance. Draw an existing upgraded
    // True Grit next, then choose the returned copy for a second exhaust.
    let mut repeated = fight(vec![card(FiendFire, 1, 0), card(Necronomicurse, 2, 0),
        card(Defend, 3, 0), card(Defend, 4, 0), card(Defend, 5, 0), card(Defend, 6, 0),
        card(Defend, 7, 0), card(Defend, 8, 0), card(Defend, 9, 0), card(TrueGrit, 10, 1)], 9);
    repeated.state.enemies[0].hp = 500;
    repeated.state.enemies[0].max_hp = 500;
    move_card(&mut repeated, 10, FixturePile::Draw);
    move_card(&mut repeated, 1, FixturePile::Hand);
    move_card(&mut repeated, 2, FixturePile::Hand);
    play(&mut repeated, 1, Some(1));
    repeated.enqueue_native_effects(vec![Effect { source: Source::None, target: Some(Actor::Player),
        kind: EffectKind::Draw(1) }], None).unwrap();
    play(&mut repeated, 10, None);
    let necro_copy = repeated.state.piles.hand.iter().find_map(OwnedCard::full)
        .filter(|live| live.id == Necronomicurse).map(|live| live.instance.0)
        .expect("Fiend Fire returned a copy to hand");
    repeated.step_full(&CombatAction::SelectCard { instance: necro_copy }).unwrap();
    let copies = card_in(&repeated, Necronomicurse);
    assert_eq!(copies.iter().filter(|(_, origin)| matches!(origin, CombatCardRef::CombatLocal(_))).count(), 2,
        "Fiend Fire and True Grit exhausts produce two fresh combat-local copies");
    assert!(copies.iter().any(|(instance, origin)| *instance == 2
        && *origin == CombatCardRef::Permanent(CardInstanceId(2)))
        && in_pile(&repeated.state.piles.exhaust, 2, Necronomicurse),
        "permanent instance remains in the exhaust census");
    assert!(repeated.fault_record().is_none());
}
