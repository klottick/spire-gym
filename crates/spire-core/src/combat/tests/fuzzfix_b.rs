//! Regressions for fuzz chunk B (spire-oto.10–.16, .19, .20).
use super::super::*;
use crate::{combat::{card_state, model::OwnedCard, CardInstanceId, CombatAction},
    content::{CardId, Character, EncounterId, PotionId, RelicId}, items::CombatItemContext, run::RunRng};
use CardId::*;

fn fixture(character: Character, cards: &[(CardId, u32)], relics: Vec<RelicId>, potions: Vec<Option<PotionId>>, energy: i32) -> Combat {
    fixture_in(EncounterId::Cultist, 1, character, cards, relics, potions, energy)
}

fn fixture_in(encounter: EncounterId, act: u8, character: Character, cards: &[(CardId, u32)], relics: Vec<RelicId>,
    potions: Vec<Option<PotionId>>, energy: i32) -> Combat {
    let mut entry = enemies::setup(encounter, act, 0, &mut RunRng::new([0x42; 32], rng::Partition::Test, 0),
        enemies::SetupContext {
            player: PlayerState { hp: 80, max_hp: 80, block: 0, energy, max_energy: energy, draw_per_turn: 5, powers: vec![] },
            player_powers: vec![], character, cards: vec![permanent_live_card(Defend, CardInstanceId(99), 0)],
            opening_draw: cards.len().min(10), encounter_visit: 0 }).unwrap();
    entry.cards = cards.iter().enumerate()
        .map(|(index, (id, upgrades))| complete_live_card(*id, CardInstanceId(index as u64 + 1), *upgrades).unwrap()).collect();
    entry.opening_draw = cards.len().min(10);
    let mut items = Some(CombatItemContext::new(relics, potions).with_complete_cards().with_complete_items());
    Combat::enter_with_item_context(entry, &mut items).unwrap()
}

fn take(combat: &mut Combat, instance: u64) -> OwnedCard {
    let handle = card_state::handle(CardInstanceId(instance));
    for pile in [&mut combat.state.piles.hand, &mut combat.state.piles.draw, &mut combat.state.piles.discard, &mut combat.state.piles.exhaust] {
        if let Some(at) = pile.iter().position(|owned| owned.handle() == handle) { return pile.remove(at); }
    }
    panic!("card {instance} is not in a pile")
}

/// Hand, draw (top is last) and discard, by fixture instance.
fn arrange(combat: &mut Combat, hand: &[u64], draw: &[u64], discard: &[u64]) {
    for instance in hand { let card = take(combat, *instance); combat.state.piles.hand.push(card); }
    for instance in draw { let card = take(combat, *instance); combat.state.piles.draw.push(card); }
    for instance in discard { let card = take(combat, *instance); combat.state.piles.discard.push(card); }
    combat.state.public.known_draw.clear();
    combat.state.public.known_draw_bottom.clear();
    piles::refresh_hand_values(&mut combat.state);
}

fn step(combat: &mut Combat, action: CombatAction, context: &str) {
    combat.step_full(&action).unwrap_or_else(|error| panic!("{context}: {error:?}; fault={:?}", combat.state.fault));
    assert!(combat.state.fault.is_none(), "{context}: {:?}", combat.state.fault);
}

fn player_power(combat: &Combat, id: &str) -> i32 {
    combat.state.player_full_powers.iter().find(|power| power.id.0 == id).map_or(0, |power| power.state.amount)
}

fn enemy_power(combat: &Combat, id: &str) -> i32 {
    combat.state.enemies[0].full_powers.iter().find(|power| power.id.0 == id).map_or(0, |power| power.state.amount)
}

/// spire-oto.19 (F20): every X-cost card autoplayed with Chemical X uses the
/// captured energy plus 2 and spends nothing (pm-ruling-autoplay-x-v2), on each
/// top-card autoplay route: Distilled Chaos (3 energy captured), Havoc (3 - 1 paid
/// for Havoc = 2) and Mayhem (the turn-2 recharge, 3). The recorded fuzz failures
/// were `Unsupported("Chemical X modifier is not represented for Whirlwind/Skewer X
/// payment")`.
#[test]
fn fuzzfix_b_every_autoplayed_x_card_adds_chemical_x_and_spends_nothing() {
    type Observe = fn(&Combat, i32, usize) -> i32;
    let cases: [(CardId, Character, Observe); 10] = [
        (Whirlwind, Character::Ironclad, |c, hp, _| (hp - c.state.enemies[0].hp) / 5),
        (Skewer, Character::Silent, |c, hp, _| (hp - c.state.enemies[0].hp) / 7),
        (Malaise, Character::Silent, |c, _, _| enemy_power(c, "weak")),
        (Doppelganger, Character::Silent, |c, _, _| player_power(c, "energized")),
        (Tempest, Character::Defect, |c, _, _| c.state.defect.lightning_channeled as i32),
        (ReinforcedBody, Character::Defect, |c, _, _| c.state.player.block / 7),
        (Collect, Character::Watcher, |c, _, _| player_power(c, "collect")),
        (ConjureBlade, Character::Watcher, |c, _, _| piles::owned_full_cards(&c.state)
            .find(|card| card.id == Expunger).map_or(-1, |card| card.misc)),
        (Transmutation, Character::Ironclad, |c, _, hand| (c.state.piles.hand.len() - hand) as i32),
        // One Lightning orb, evoked X times for 8 each (Mayhem: plus its 3 end-of-turn passive, floored away).
        (MultiCast, Character::Defect, |c, hp, _| (hp - c.state.enemies[0].hp) / 8),
    ];
    for (card, character, observe) in cases {
        let lightning = |combat: &mut Combat| if card == MultiCast {
            combat.state.character.orb_slots = vec![Some(characters::OrbKind::Lightning.into()), None, None];
        };
        // Distilled Chaos: the X card is the only card left to autoplay, so no filler card adds to the observation.
        let mut combat = fixture(character, &[(card, 0), (Defend, 0), (Defend, 0), (Defend, 0)],
            vec![RelicId::ChemicalX], vec![Some(PotionId::DistilledChaos), None, None], 3);
        arrange(&mut combat, &[2, 3, 4], &[1], &[]);
        lightning(&mut combat);
        let (hp, hand) = (combat.state.enemies[0].hp, combat.state.piles.hand.len());
        step(&mut combat, CombatAction::UsePotion { slot: 0, target: None }, &format!("Distilled Chaos {card:?} + Chemical X"));
        assert_eq!(observe(&combat, hp, hand), 5, "Distilled Chaos {card:?}: X = 3 captured energy + 2 Chemical X");
        assert_eq!(combat.state.player.energy, 3, "Distilled Chaos {card:?}: an autoplayed X card spends nothing");

        let mut combat = fixture(character, &[(card, 0), (Havoc, 0), (Defend, 0), (Defend, 0)], vec![RelicId::ChemicalX], vec![None; 3], 3);
        arrange(&mut combat, &[2, 3, 4], &[1], &[]);
        lightning(&mut combat);
        let (hp, hand) = (combat.state.enemies[0].hp, combat.state.piles.hand.len() - 1);
        step(&mut combat, CombatAction::PlayCard { instance: 2, target: None }, &format!("Havoc {card:?} + Chemical X"));
        assert_eq!(observe(&combat, hp, hand), 4, "Havoc {card:?}: X = 2 energy after Havoc + 2 Chemical X");
        assert_eq!(combat.state.player.energy, 2, "Havoc {card:?}: an autoplayed X card spends nothing");

        let mut combat = fixture(character, &[(card, 0), (Mayhem, 0), (Defend, 0), (Defend, 0), (Defend, 0), (Defend, 0), (Defend, 0)],
            vec![RelicId::ChemicalX], vec![None; 3], 3);
        arrange(&mut combat, &[2], &[1, 3, 4, 5, 6, 7], &[]);
        lightning(&mut combat);
        let hp = combat.state.enemies[0].hp;
        play_powers_and_end_turn(&mut combat, &[2]);
        assert_eq!(observe(&combat, hp, 5), 5, "Mayhem {card:?}: X = 3 recharged energy + 2 Chemical X");
        assert_eq!(combat.state.player.energy, 3, "Mayhem {card:?}: an autoplayed X card spends nothing");
    }
}

/// spire-oto.11 (F12): the cleanse classifier shared by Champ, Awakened, Time
/// Eater and Orange Pellets classifies every power the engine can publish, at
/// either sign, so no cleanse can meet an unclassified power.
#[test]
fn fuzzfix_b_cleanse_classifies_every_public_power() {
    let unclassified = public_ids::PUBLIC_POWERS.iter().flat_map(|id| [-1, 1].map(|amount| (*id, amount)))
        .filter(|(id, amount)| powers::cleanse_debuff(&FullPowerEntry { id: FullPowerId(id),
            state: PowerState { amount: *amount, skip_first: false, monster_source_grace: false } }).is_none())
        .map(|(id, _)| id).collect::<std::collections::BTreeSet<_>>();
    assert!(unclassified.is_empty(), "unclassified cleanse powers: {unclassified:?}");
}

/// spire-oto.20 (chunk A review m2): a direct manual-discard callback is gated on
/// its tagged producer and the moved card, not on the phase it resolves in.
#[test]
fn fuzzfix_b_direct_manual_discard_callback_is_not_phase_gated() {
    let mut combat = fixture(Character::Silent, &[(DefendGreen, 0), (DefendGreen, 0)], vec![RelicId::Tingsha], vec![None; 3], 3);
    arrange(&mut combat, &[2], &[], &[1]);
    let callback = |instance: u64| Effect { source: Source::Card(card_state::handle(CardInstanceId(instance))), target: None,
        kind: EffectKind::ManualDiscardCallback { origin: ManualDiscardOrigin::Direct } };
    for phase in [Phase::PlayerStart, Phase::Decision, Phase::PlayerAction, Phase::Selection, Phase::EnemyTurn, Phase::RoundEnd] {
        combat.state.phase = phase;
        assert_eq!(engine::validate_effect(&callback(1), &combat.state), Ok(()), "{phase:?}");
        assert!(engine::validate_effect(&callback(2), &combat.state).is_err(), "{phase:?}: the card was not moved to discard");
    }
}

/// A refresh-only value as on a card that has never been in the hand.
fn forget_values(combat: &mut Combat, instance: u64) {
    let handle = card_state::handle(CardInstanceId(instance));
    let card = combat.state.piles.draw.iter_mut().find(|owned| owned.handle() == handle).and_then(OwnedCard::full_mut).unwrap();
    card.local.base_block = None;
    card.local.base_damage = None;
}

/// Turn 1: play the powers in `hand` (instance order), then end the turn.
fn play_powers_and_end_turn(combat: &mut Combat, powers: &[u64]) {
    combat.state.player.energy = 9;
    for instance in powers { step(combat, CombatAction::PlayCard { instance: *instance, target: None }, "turn 1 power"); }
    step(combat, CombatAction::EndTurn, "end turn 1");
    assert_eq!(combat.state.turn, 2);
}

/// spire-oto.20: Mayhem's play-top runs after the turn's draw, after the recharge with
/// Deva Form's synchronous gain, so Collect's X is 3 + 1.
#[test]
fn fuzzfix_b_mayhem_x_sees_the_recharge_with_deva_form() {
    let mut combat = fixture(Character::Watcher, &[(Mayhem, 0), (DevaForm, 0), (Collect, 0),
        (Defend, 0), (Defend, 0), (Defend, 0), (Defend, 0), (Defend, 0)], vec![], vec![None; 3], 3);
    arrange(&mut combat, &[1, 2], &[3, 4, 5, 6, 7, 8], &[]);
    play_powers_and_end_turn(&mut combat, &[1, 2]);
    assert_eq!(player_power(&combat, "collect"), 4, "Collect X = 3 recharge + 1 Deva Form");
    assert_eq!(combat.state.player.energy, 4, "the autoplayed Collect spends nothing");
}

/// spire-oto.20: Mayhem plays the card on top after the turn's draw, not the card the draw takes.
#[test]
fn fuzzfix_b_mayhem_plays_the_top_card_after_the_turn_draw() {
    let mut combat = fixture(Character::Ironclad, &[(Mayhem, 0), (Defend, 0),
        (Strike, 0), (Strike, 0), (Strike, 0), (Strike, 0), (Strike, 0)], vec![], vec![None; 3], 3);
    arrange(&mut combat, &[1], &[2, 3, 4, 5, 6, 7], &[]);
    play_powers_and_end_turn(&mut combat, &[1]);
    let hp = combat.state.enemies[0].hp;
    assert_eq!(combat.state.player.block, 5, "the sixth card, Defend, is Mayhem's");
    assert_eq!(combat.state.enemies[0].hp, hp);
    assert_eq!(combat.state.piles.hand.iter().filter_map(OwnedCard::full).filter(|card| card.id == Strike).count(), 5);
}

/// spire-oto.20: a Void drawn at turn start queues its energy loss after Mayhem's play-top,
/// so Mayhem's X is taken before the loss; the queued card is used after it.
#[test]
fn fuzzfix_b_void_energy_loss_follows_mayhem_play_top() {
    let mut combat = fixture(Character::Ironclad, &[(Mayhem, 0), (Whirlwind, 0),
        (Void, 0), (Strike, 0), (Strike, 0), (Strike, 0), (Strike, 0)], vec![], vec![None; 3], 3);
    arrange(&mut combat, &[1], &[2, 3, 4, 5, 6, 7], &[]);
    let hp = combat.state.enemies[0].hp;
    play_powers_and_end_turn(&mut combat, &[1]);
    assert_eq!(hp - combat.state.enemies[0].hp, 15, "Whirlwind X = 3, captured before Void's loss");
    assert_eq!(combat.state.player.energy, 2, "Void's loss still applies");
}

/// spire-oto.20: a play-top with an empty draw pile shuffles the discard and plays its top.
#[test]
fn fuzzfix_b_mayhem_play_top_shuffles_an_empty_draw_pile() {
    let mut combat = fixture(Character::Ironclad, &[(Mayhem, 0), (Defend, 0),
        (Strike, 0), (Strike, 0), (Strike, 0), (Strike, 0), (Strike, 0)], vec![], vec![None; 3], 3);
    arrange(&mut combat, &[1, 2], &[3, 4, 5, 6, 7], &[]);
    play_powers_and_end_turn(&mut combat, &[1]);
    assert_eq!(combat.state.player.block, 5, "the shuffled discard's only card, Defend");
}

/// spire-oto.14 (F15): PlayTopCardAction refreshes Stack before queueing it. The recorded
/// fuzz fault was `Unsupported("Stack Block was not refreshed on hand entry")`.
#[test]
fn fuzzfix_b_mayhem_refreshes_stack_when_it_takes_it() {
    let mut combat = fixture(Character::Defect, &[(Mayhem, 0), (Stack, 0), (Defend, 0), (Defend, 0), (Defend, 0),
        (Strike, 0), (Strike, 0), (Strike, 0), (Strike, 0), (Strike, 0)], vec![], vec![None; 3], 3);
    arrange(&mut combat, &[1, 3, 4, 5], &[2, 6, 7, 8, 9, 10], &[]);
    forget_values(&mut combat, 2);
    play_powers_and_end_turn(&mut combat, &[1]);
    assert_eq!(combat.state.player.block, 3, "Stack Block = the three discarded Defends");
}

/// spire-oto.13 (F14): Blizzard recounts Frost at use on every play path. The recorded
/// fuzz fault was `Unsupported("Blizzard damage was not refreshed from Frost history")`.
#[test]
fn fuzzfix_b_autoplayed_blizzard_counts_frost_at_use() {
    let mut combat = fixture(Character::Defect, &[(Blizzard, 0), (Defend, 0)], vec![], vec![Some(PotionId::DistilledChaos), None, None], 3);
    arrange(&mut combat, &[2], &[1], &[]);
    forget_values(&mut combat, 1);
    combat.state.defect.frost_channeled = 2;
    let hp = combat.state.enemies[0].hp;
    step(&mut combat, CombatAction::UsePotion { slot: 0, target: None }, "Distilled Chaos Blizzard");
    assert_eq!(hp - combat.state.enemies[0].hp, 4, "2 Frost x 2");
}

/// spire-oto.14/.20: every top-card action of a Distilled Chaos burst runs before the first
/// card is used: the burst's cards are the original top three, and Stack's value is the
/// discard count before the burst.
#[test]
fn fuzzfix_b_distilled_chaos_takes_the_whole_burst_before_using_it() {
    let mut combat = fixture(Character::Defect, &[(PommelStrike, 0), (Defend, 0), (Stack, 0), (Strike, 0), (Defend, 0), (Defend, 0)],
        vec![], vec![Some(PotionId::DistilledChaos), None, None], 3);
    arrange(&mut combat, &[], &[4, 3, 2, 1], &[5, 6]);
    forget_values(&mut combat, 3);
    step(&mut combat, CombatAction::UsePotion { slot: 0, target: None }, "Distilled Chaos burst");
    assert_eq!(combat.state.player.block, 7, "Defend 5 + Stack 2 (discard before the burst)");
    let hand = combat.state.piles.hand.iter().filter_map(OwnedCard::full).map(|card| card.id).collect::<Vec<_>>();
    assert_eq!(hand, vec![Strike], "Pommel Strike drew the card below the burst");
}

/// Advance the card-random stream by `draws`, to choose a deterministic target sequence.
fn advance_card_rng(combat: &mut Combat, draws: usize) {
    let mut stream = combat.state.card_rng.clone().unwrap();
    for _ in 0..draws { stream = crate::combat::rng::draw_u64(&stream).unwrap().1; }
    combat.state.card_rng = Some(stream);
}

/// The target each card was used against, by fixture instance (None: not used).
fn used_target(combat: &Combat, instance: u64) -> Option<Option<Actor>> {
    let source = Source::Card(card_state::handle(CardInstanceId(instance)));
    combat.state.hook_events.iter().find(|event| event.hook == Hook::CardPlayed && event.source == source).map(|event| event.target.clone())
}

fn enemy_index(combat: &Combat, target: &Option<Actor>) -> usize {
    let Some(Actor::Enemy(owner)) = target else { panic!("not an enemy target: {target:?}") };
    combat.state.enemies.iter().position(|enemy| &enemy.instance_id == owner).unwrap()
}

fn in_pile(pile: &[OwnedCard], instance: u64) -> bool {
    pile.iter().any(|owned| owned.handle() == card_state::handle(CardInstanceId(instance)))
}

/// spire-oto.20 dead-target ruling (source-check-dead-target-v1) items 2 and 3: a queued
/// single-enemy card whose captured target died is not used (no play hook, no damage) and
/// takes its normal destination; a queued ALL_ENEMY card plays normally.
#[test]
fn fuzzfix_b_queued_card_with_a_dead_target_is_not_used_and_an_all_enemy_card_still_plays() {
    let make = |draws: usize| {
        let mut combat = fixture_in(EncounterId::ThreeCultists, 2, Character::Silent,
            &[(StrikeGreen, 0), (StrikeGreen, 0), (DaggerSpray, 0), (DefendGreen, 0)],
            vec![], vec![Some(PotionId::DistilledChaos), None, None], 3);
        arrange(&mut combat, &[4], &[3, 2, 1], &[]);
        advance_card_rng(&mut combat, draws);
        combat
    };
    // Choose a burst whose three captured targets are the same Cultist.
    let draws = (0..64).find(|draws| {
        let mut probe = make(*draws);
        step(&mut probe, CombatAction::UsePotion { slot: 0, target: None }, "probe burst");
        let targets = [1, 2, 3].map(|card| used_target(&probe, card).unwrap());
        targets[0] == targets[1] && targets[1] == targets[2]
    }).expect("a card-random state with one shared burst target");
    let mut combat = make(draws);
    let mut probe = combat.clone();
    step(&mut probe, CombatAction::UsePotion { slot: 0, target: None }, "probe burst");
    let doomed = enemy_index(&probe, &used_target(&probe, 1).unwrap());
    combat.state.enemies[doomed].hp = 1;
    let before = combat.state.enemies.iter().map(|enemy| enemy.hp).collect::<Vec<_>>();
    step(&mut combat, CombatAction::UsePotion { slot: 0, target: None }, "Distilled Chaos burst");
    assert!(combat.state.enemies[doomed].hp <= 0, "the first Strike kills the shared target");
    assert_eq!(used_target(&combat, 2), None, "the second Strike's target died: it is not used");
    assert!(in_pile(&combat.state.piles.discard, 2), "the unused Strike takes its normal destination");
    assert!(used_target(&combat, 3).is_some(), "Dagger Spray (ALL_ENEMY) ignores its dead captured target");
    for (index, enemy) in combat.state.enemies.iter().enumerate().filter(|(index, _)| *index != doomed) {
        assert_eq!(before[index] - enemy.hp, 8, "only Dagger Spray hit Cultist {index}");
    }
    assert_eq!(combat.state.hook_events.iter().filter(|event| event.hook == Hook::CardPlayed).count(), 2);
    assert_eq!(combat.state.outcome, None);
}

/// spire-oto.20 dead-target ruling item 4: when a burst card kills the last monster the
/// combat ends and no remaining queued card is used.
#[test]
fn fuzzfix_b_victory_mid_burst_uses_no_remaining_queued_card() {
    let mut combat = fixture(Character::Silent, &[(StrikeGreen, 0), (StrikeGreen, 0), (DefendGreen, 0), (DefendGreen, 0)],
        vec![], vec![Some(PotionId::DistilledChaos), None, None], 3);
    arrange(&mut combat, &[4], &[3, 2, 1], &[]);
    combat.state.enemies[0].hp = 1;
    step(&mut combat, CombatAction::UsePotion { slot: 0, target: None }, "lethal Distilled Chaos burst");
    assert_eq!(combat.state.outcome, Some(Outcome::Victory));
    assert!(used_target(&combat, 1).is_some());
    assert_eq!((used_target(&combat, 2), used_target(&combat, 3)), (None, None), "no card is used after the victory");
    assert_eq!(combat.state.player.block, 0);
}

/// spire-oto.20 dead-target ruling item 5: Omniscience's card and its repeat are random-target
/// queue items; each picks a live target when it is dequeued, so a repeat never meets the
/// target the first play killed.
#[test]
fn fuzzfix_b_omniscience_repeat_picks_a_live_target_at_dequeue() {
    for draws in 0..12 {
        let mut combat = fixture_in(EncounterId::ThreeCultists, 2, Character::Watcher,
            &[(Omniscience, 0), (StrikePurple, 0), (DefendPurple, 0)], vec![], vec![None; 3], 10);
        arrange(&mut combat, &[1, 3], &[2], &[]);
        advance_card_rng(&mut combat, draws);
        for enemy in &mut combat.state.enemies { enemy.hp = 1; }
        step(&mut combat, CombatAction::PlayCard { instance: 1, target: None }, "Omniscience");
        step(&mut combat, CombatAction::SelectCard { instance: 2 }, "Omniscience selects Strike");
        let dead = combat.state.enemies.iter().filter(|enemy| enemy.hp <= 0).count();
        assert_eq!(dead, 2, "draws {draws}: the Strike and its repeat each kill a live Cultist");
    }
}

/// B1 review I3: Mayhem's queued cards are card-queue items, so the shared card-queue
/// cancel covers them. Time Warp's 12th card (the first Mayhem Strike) ends the turn, and
/// the second Mayhem Strike is discarded unused, as a Havoc or Distilled Chaos card would be.
#[test]
fn fuzzfix_b_time_warp_cancels_the_rest_of_a_mayhem_burst() {
    let mut combat = fixture_in(EncounterId::TimeEater, 3, Character::Ironclad,
        &[(Mayhem, 0), (Strike, 0), (Strike, 0), (Defend, 0), (Defend, 0), (Defend, 0), (Defend, 0), (Defend, 0)],
        vec![], vec![None; 3], 3);
    arrange(&mut combat, &[1], &[2, 3, 4, 5, 6, 7, 8], &[]);
    combat.state.player.energy = 9;
    step(&mut combat, CombatAction::PlayCard { instance: 1, target: None }, "Mayhem");
    combat.state.player_full_powers.iter_mut().find(|power| power.id.0 == "mayhem").unwrap().state.amount = 2;
    combat.state.enemies[0].full_powers.iter_mut().find(|power| power.id.0 == "time_warp").unwrap().state.amount = 11;
    step(&mut combat, CombatAction::EndTurn, "Mayhem burst into Time Warp");
    assert!(used_target(&combat, 3).is_some(), "the first Mayhem Strike is the 12th card");
    assert_eq!(used_target(&combat, 2), None, "Time Warp cancels the second Mayhem Strike");
    // Time Warp ends turn 2; the discarded Strike is shuffled back and drawn on turn 3.
    assert_eq!(combat.state.turn, 3);
    assert!(in_pile(&combat.state.piles.hand, 2) && !in_pile(&combat.state.piles.exhaust, 2),
        "the cancelled card was discarded, not exhausted or kept in queue custody");
    assert!(combat.state.queued_autoplay.is_empty());
}

/// B1 review m2 (out-of-hand values, PM ruling Q2): a top-card action refreshes Mind Blast
/// to the draw pile left after it is taken; Omniscience plays a never-refreshed Mind Blast
/// (and its stat-equivalent repeat) with the constructor value, 0.
#[test]
fn fuzzfix_b_mind_blast_out_of_hand_uses_its_refreshed_or_constructor_value() {
    let mut chaos = fixture(Character::Ironclad, &[(MindBlast, 0), (Defend, 0), (Defend, 0), (Defend, 0), (Defend, 0), (Defend, 0)],
        vec![], vec![Some(PotionId::DistilledChaos), None, None], 3);
    arrange(&mut chaos, &[], &[2, 3, 4, 5, 6, 1], &[]);
    forget_values(&mut chaos, 1);
    let hp = chaos.state.enemies[0].hp;
    step(&mut chaos, CombatAction::UsePotion { slot: 0, target: None }, "Distilled Chaos Mind Blast");
    assert_eq!(hp - chaos.state.enemies[0].hp, 5, "refreshed when taken: five cards left in the draw pile");

    let mut omniscience = fixture(Character::Watcher, &[(Omniscience, 0), (MindBlast, 0), (Defend, 0)], vec![], vec![None; 3], 10);
    arrange(&mut omniscience, &[1], &[3, 2], &[]);
    forget_values(&mut omniscience, 2);
    let hp = omniscience.state.enemies[0].hp;
    step(&mut omniscience, CombatAction::PlayCard { instance: 1, target: None }, "Omniscience");
    step(&mut omniscience, CombatAction::SelectCard { instance: 2 }, "Omniscience selects Mind Blast");
    assert!(used_target(&omniscience, 2).is_some(), "Mind Blast was used");
    assert_eq!(omniscience.state.enemies[0].hp, hp, "never refreshed: constructor damage 0, twice");
}

/// B1 review m2: Necronomicon's copy is refreshed (applyPowers) before it is queued, unlike
/// Double Tap/Burst/Echo Form copies, which keep the original's values.
#[test]
fn fuzzfix_b_necronomicon_refreshes_its_copy() {
    let mut combat = fixture(Character::Ironclad, &[(MindBlast, 0), (Defend, 0), (Defend, 0), (Defend, 0)],
        vec![RelicId::Necronomicon], vec![None; 3], 3);
    arrange(&mut combat, &[1], &[2, 3, 4], &[]);
    combat.state.enemies[0].max_hp = 500;
    combat.state.enemies[0].hp = 500;
    // A stale value on the original: the copy must not inherit it.
    combat.state.piles.hand.iter_mut().filter_map(OwnedCard::full_mut).find(|card| card.id == MindBlast).unwrap().local.base_damage = Some(40);
    step(&mut combat, CombatAction::PlayCard { instance: 1, target: Some(TargetId(1)) }, "Mind Blast with Necronomicon");
    assert_eq!(500 - combat.state.enemies[0].hp, 40 + 3, "the original keeps 40; the refreshed copy deals the draw count, 3");
}

/// Like `fixture_in`, with a five-card opening draw so relic draws have cards to take.
fn fixture_five(encounter: EncounterId, act: u8, character: Character, cards: &[(CardId, u32)], relics: Vec<RelicId>) -> Combat {
    let mut entry = enemies::setup(encounter, act, 0, &mut RunRng::new([0x42; 32], rng::Partition::Test, 0),
        enemies::SetupContext {
            player: PlayerState { hp: 80, max_hp: 80, block: 0, energy: 3, max_energy: 3, draw_per_turn: 5, powers: vec![] },
            player_powers: vec![], character, cards: vec![permanent_live_card(Defend, CardInstanceId(99), 0)],
            opening_draw: 5, encounter_visit: 0 }).unwrap();
    entry.cards = cards.iter().enumerate()
        .map(|(index, (id, upgrades))| complete_live_card(*id, CardInstanceId(index as u64 + 1), *upgrades).unwrap()).collect();
    entry.opening_draw = 5;
    let mut items = Some(CombatItemContext::new(relics, vec![None; 3]).with_complete_cards().with_complete_items());
    Combat::enter_with_item_context(entry, &mut items).unwrap()
}

/// B1 review I2 (source-check-turnstart-hooks-v1): Art of War and Happy Flower are atTurnStart
/// relics whose energy is queued before the turn draw and before Mayhem's wrapper, so the
/// Whirlwind Mayhem plays sees 3 + 1.
#[test]
fn fuzzfix_b_turn_start_relic_energy_precedes_mayhem_x() {
    for relic in [RelicId::ArtOfWar, RelicId::HappyFlower] {
        let mut combat = fixture(Character::Ironclad, &[(Mayhem, 0), (Whirlwind, 0),
            (Defend, 0), (Defend, 0), (Defend, 0), (Defend, 0), (Defend, 0)], vec![relic], vec![None; 3], 3);
        arrange(&mut combat, &[1], &[2, 3, 4, 5, 6, 7], &[]);
        // Happy Flower's third turn start is turn 2: its counter reads 2 after turn 1.
        if relic == RelicId::HappyFlower { combat.state.item_context.as_mut().unwrap().set_counter(relic, 2); }
        let hp = combat.state.enemies[0].hp;
        play_powers_and_end_turn(&mut combat, &[1]);
        assert_eq!((hp - combat.state.enemies[0].hp) / 5, 4, "{relic:?}: Whirlwind X = 3 + 1");
        assert_eq!(combat.state.player.energy, 4, "{relic:?}: the autoplayed Whirlwind spends nothing");
    }
}

/// The source check's own example: Deva Form played on the previous turn, Art of War
/// triggered, Mayhem, Collect on top: Collect X = 3 + 1 (Deva) + 1 (Art of War) = 5; once Deva
/// Form has already recharged once its grant is 2, and X = 6.
#[test]
fn fuzzfix_b_collect_x_sees_art_of_war_and_deva_form() {
    let mut combat = fixture(Character::Watcher, &[(Mayhem, 0), (DevaForm, 0), (Collect, 0),
        (Defend, 0), (Defend, 0), (Defend, 0), (Defend, 0), (Defend, 0)], vec![RelicId::ArtOfWar], vec![None; 3], 3);
    arrange(&mut combat, &[1, 2], &[3, 4, 5, 6, 7, 8], &[]);
    play_powers_and_end_turn(&mut combat, &[1, 2]);
    assert_eq!(player_power(&combat, "collect"), 5, "first Deva recharge: 3 + 1 + 1");

    let mut cards = vec![(Mayhem, 0), (DevaForm, 0), (Collect, 0)];
    cards.extend([(Defend, 0); 9]);
    let mut combat = fixture(Character::Watcher, &cards, vec![RelicId::ArtOfWar], vec![None; 3], 3);
    // Turn 2 draws Mayhem and four Defends; turn 3 draws five Defends, leaving Collect on top.
    arrange(&mut combat, &[2], &[3, 4, 5, 6, 7, 8, 1, 9, 10, 11, 12], &[]);
    play_powers_and_end_turn(&mut combat, &[2]);
    combat.state.player.energy = 9;
    step(&mut combat, CombatAction::PlayCard { instance: 1, target: None }, "turn 2 Mayhem");
    combat.state.player.energy = 0;
    step(&mut combat, CombatAction::EndTurn, "end turn 2");
    assert_eq!(combat.state.turn, 3);
    assert_eq!(player_power(&combat, "collect"), 6, "second Deva recharge: 3 + 2 + 1");
}

/// The source check's lethal example: Mercury Hourglass's damage is queued before Mayhem's
/// wrapper; killing the lone enemy ends combat and clears the wrapper, so no target is drawn.
/// With a survivor, the wrapper's live-target pick sees the post-damage set: a Mayhem Strike
/// is always used (review I2: 5 of 16 card-RNG offsets lost it).
#[test]
fn fuzzfix_b_mercury_hourglass_resolves_before_mayhem_picks_a_target() {
    let cards = [(Mayhem, 0), (Strike, 0), (Defend, 0), (Defend, 0), (Defend, 0), (Defend, 0), (Defend, 0)];
    let mut lethal = fixture(Character::Ironclad, &cards, vec![RelicId::MercuryHourglass], vec![None; 3], 3);
    arrange(&mut lethal, &[1], &[2, 3, 4, 5, 6, 7], &[]);
    lethal.state.player.energy = 9;
    step(&mut lethal, CombatAction::PlayCard { instance: 1, target: None }, "Mayhem");
    lethal.state.enemies[0].hp = 3;
    let card_rng = lethal.state.card_rng.clone();
    step(&mut lethal, CombatAction::EndTurn, "Mercury Hourglass kills at turn 2 start");
    assert_eq!(lethal.state.outcome, Some(Outcome::Victory));
    assert_eq!(lethal.state.card_rng, card_rng, "no Mayhem target was drawn");
    assert_eq!(used_target(&lethal, 2), None);

    for draws in 0..16 {
        let mut combat = fixture_in(EncounterId::ThreeCultists, 2, Character::Ironclad, &cards,
            vec![RelicId::MercuryHourglass], vec![None; 3], 3);
        arrange(&mut combat, &[1], &[2, 3, 4, 5, 6, 7], &[]);
        advance_card_rng(&mut combat, draws);
        combat.state.player.energy = 9;
        step(&mut combat, CombatAction::PlayCard { instance: 1, target: None }, "Mayhem");
        for (index, enemy) in combat.state.enemies.iter_mut().enumerate() {
            enemy.hp = if index == 0 { 3 } else { 200 };
            enemy.max_hp = enemy.max_hp.max(enemy.hp);
        }
        step(&mut combat, CombatAction::EndTurn, "end turn 1");
        let target = used_target(&combat, 2).unwrap_or_else(|| panic!("draws {draws}: the Mayhem Strike is used"));
        assert_ne!(enemy_index(&combat, &target), 0, "draws {draws}: the dead Cultist is never picked");
    }
}

/// B1 review I1, turn 1 (battle start): atBattleStart bottom work (Bag of Preparation), then
/// atTurnStart relics (Mercury Hourglass bottom; Lantern top), then atTurnStartPostDraw relics
/// (Gambling Chip). The Chip's selection opens after the others resolved.
#[test]
fn fuzzfix_b_turn_one_start_relics_precede_post_draw_relics() {
    let mut cards = vec![(Strike, 0); 4];
    cards.extend([(Defend, 0); 5]);
    let combat = fixture_five(EncounterId::Cultist, 1, Character::Ironclad, &cards,
        vec![RelicId::GamblingChip, RelicId::MercuryHourglass, RelicId::Lantern, RelicId::BagOfPreparation]);
    assert!(combat.state.pending_selection.is_some(), "Gambling Chip's selection is open");
    assert_eq!(combat.state.enemies[0].hp, combat.state.enemies[0].max_hp - 3, "Mercury Hourglass already hit");
    assert_eq!(combat.state.player.energy, 4, "Lantern's energy already arrived");
    assert_eq!(combat.state.piles.hand.len(), 7, "Bag of Preparation already drew");
}

/// B1 review I1, turn 2: the atTurnStart relics run before the turn draw, and the
/// atTurnStartPostDraw relics (Pocketwatch's three draws) after it, after TurnStarted.
#[test]
fn fuzzfix_b_turn_two_post_draw_relics_follow_the_draw() {
    let mut combat = fixture_five(EncounterId::Cultist, 1, Character::Ironclad, &[(Defend, 0); 14],
        vec![RelicId::Pocketwatch, RelicId::ArtOfWar]);
    combat.state.player.energy = 0;
    let before = combat.state.hook_events.len();
    step(&mut combat, CombatAction::EndTurn, "end turn 1");
    let hooks = combat.state.hook_events[before..].iter()
        .filter(|event| matches!(event.hook, Hook::TurnStartedPreDraw | Hook::TurnStarted | Hook::CardDrawn))
        .map(|event| event.hook).collect::<Vec<_>>();
    let mut expected = vec![Hook::TurnStartedPreDraw];
    expected.extend([Hook::CardDrawn; 5]);
    expected.push(Hook::TurnStarted);
    expected.extend([Hook::CardDrawn; 3]);
    assert_eq!(hooks, expected);
    assert_eq!(combat.state.player.energy, 4, "Art of War: turn 1 played no Attack");
    assert_eq!(combat.state.piles.hand.len(), 8);
}

/// Coverage against source-check-turnstart-hooks-v1: Enchiridion (atPreBattle) and Pen Nib
/// (atBattleStart) fire once per combat. Watcher contexts also raise the pre-draw hook, which
/// used to fire both a second time.
#[test]
fn fuzzfix_b_battle_start_relics_fire_once_for_every_character() {
    // Complete Watcher runs use the M21 Watcher startup (run/training.rs, `with_m25_profile`).
    let context = |character: Character, relics: Vec<RelicId>| {
        let context = CombatItemContext::new(relics, vec![None; 3]);
        let context = if character == Character::Watcher { context.with_m25_profile() } else { context };
        context.with_complete_cards().with_complete_items()
    };
    for character in [Character::Ironclad, Character::Watcher] {
        let mut entry = enemies::setup(EncounterId::Cultist, 1, 0, &mut RunRng::new([0x42; 32], rng::Partition::Test, 0),
            enemies::SetupContext {
                player: PlayerState { hp: 80, max_hp: 80, block: 0, energy: 3, max_energy: 3, draw_per_turn: 5, powers: vec![] },
                player_powers: vec![], character, cards: vec![permanent_live_card(Defend, CardInstanceId(99), 0)],
                opening_draw: 3, encounter_visit: 0 }).unwrap();
        entry.cards = (1..=3).map(|instance| complete_live_card(Defend, CardInstanceId(instance), 0).unwrap()).collect();
        entry.opening_draw = 3;
        let combat = Combat::enter_with_item_context(entry, &mut Some(context(character, vec![RelicId::Enchiridion]))).unwrap();
        let generated = combat.state.piles.hand.iter().filter_map(OwnedCard::full).filter(|card| card.instance.0 > 3).count();
        assert_eq!(generated, 1, "{character:?}: one Enchiridion Power card");

        let mut entry = enemies::setup(EncounterId::Cultist, 1, 0, &mut RunRng::new([0x42; 32], rng::Partition::Test, 0),
            enemies::SetupContext {
                player: PlayerState { hp: 80, max_hp: 80, block: 0, energy: 3, max_energy: 3, draw_per_turn: 5, powers: vec![] },
                player_powers: vec![], character, cards: vec![permanent_live_card(Defend, CardInstanceId(99), 0)],
                opening_draw: 1, encounter_visit: 0 }).unwrap();
        entry.cards = vec![complete_live_card(Defend, CardInstanceId(1), 0).unwrap()];
        entry.opening_draw = 1;
        let mut items = Some(context(character, vec![RelicId::PenNib])
            .with_counters(std::collections::BTreeMap::from([(RelicId::PenNib, 9)])));
        let combat = Combat::enter_with_item_context(entry, &mut items).unwrap();
        assert_eq!(player_power(&combat, "pen_nib"), 1, "{character:?}: one Pen Nib power");
    }
}

/// Review v2 I1: Red Skull's battle-start check is atBattleStart addToBot. It reads HP when it
/// resolves, after the top-queued heals, so Blood Vial lifting 40/80 to 42/80 means it never
/// activates and the later Clockwork Souvenir's Artifact is kept.
#[test]
fn fuzzfix_b_red_skull_checks_hp_after_the_battle_start_heals() {
    let enter = |hp: i32, relics: Vec<RelicId>| {
        let mut entry = enemies::setup(EncounterId::Cultist, 1, 0, &mut RunRng::new([0x42; 32], rng::Partition::Test, 0),
            enemies::SetupContext {
                player: PlayerState { hp, max_hp: 80, block: 0, energy: 3, max_energy: 3, draw_per_turn: 5, powers: vec![] },
                player_powers: vec![], character: Character::Ironclad, cards: vec![permanent_live_card(Defend, CardInstanceId(99), 0)],
                opening_draw: 1, encounter_visit: 0 }).unwrap();
        entry.cards = vec![complete_live_card(Defend, CardInstanceId(1), 0).unwrap()];
        entry.opening_draw = 1;
        let mut items = Some(CombatItemContext::new(relics, vec![None; 3]).with_complete_cards().with_complete_items());
        Combat::enter_with_item_context(entry, &mut items).unwrap()
    };
    // Every relic-bar order: the heal and the Artifact are top work, the check is bottom work.
    let (skull, vial, clock) = (RelicId::RedSkull, RelicId::BloodVial, RelicId::ClockworkSouvenir);
    for relics in [[skull, vial, clock], [skull, clock, vial], [vial, skull, clock], [vial, clock, skull],
        [clock, skull, vial], [clock, vial, skull]] {
        let combat = enter(40, relics.to_vec());
        assert_eq!((combat.state.player.hp, player_power(&combat, "strength"), player_power(&combat, "artifact")), (42, 0, 1),
            "{relics:?}");
    }
    let combat = enter(30, vec![RelicId::RedSkull, RelicId::BloodVial]);
    assert_eq!((combat.state.player.hp, player_power(&combat, "strength")), (32, 3), "still bloodied after the heal");
}

/// Review v2 scan: Energy Down is an atStartOfTurn power whose loss is queued (addToBot), after
/// the start-of-turn relics' queued energy. Recharge 3, Art of War +1, then lose 4: 0.
#[test]
fn fuzzfix_b_energy_down_loss_follows_the_start_relic_energy() {
    let mut combat = fixture(Character::Ironclad, &[(Defend, 0); 8], vec![RelicId::ArtOfWar], vec![None; 3], 3);
    combat.state.player_full_powers.push(FullPowerEntry { id: FullPowerId("energy_down"),
        state: PowerState { amount: 4, skip_first: false, monster_source_grace: false } });
    step(&mut combat, CombatAction::EndTurn, "end turn 1");
    assert_eq!(combat.state.player.energy, 0, "3 + 1 - 4");
    assert_eq!(player_power(&combat, "energy_down"), 4, "Energy Down persists");
}
