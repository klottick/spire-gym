//! M25 Watcher interactions with Time Eater, Act3 bosses, and multi-hit owners.
use super::super::*;
use crate::{
    content::{CardId, Character, EncounterId},
    items::CombatItemContext,
};
use super::{m5_elites::hit, m8_shapes::entry};

use CardId::*;

fn handle(id: u64) -> CardHandle {
    card_state::handle(CardInstanceId(id))
}

fn watcher(mut e: CombatEntry, deck: &[(CardId, u32)], energy: i32) -> Combat {
    e.setup.character = Character::Watcher;
    e.cards = deck.iter().enumerate().map(|(i, (id, upgrades))|
        m25_watcher_live_card(*id, CardInstanceId(i as u64 + 1), *upgrades).unwrap()).collect();
    e.opening_draw = deck.len().min(10);
    e.setup.player.energy = energy;
    e.setup.player.max_energy = energy;
    e.setup.player.draw_per_turn = 1;
    let mut context = Some(CombatItemContext::new(vec![], vec![None; 3]).with_m25_profile());
    let combat = Combat::enter_with_item_context(e, &mut context).unwrap();
    assert!(context.is_none());
    combat
}

fn time_eater(count: i32, deck: &[(CardId, u32)], energy: i32) -> Combat {
    let mut combat = watcher(entry(EncounterId::TimeEater), deck, energy);
    combat.state.enemies[0].full_powers.iter_mut()
        .find(|power| power.id.0 == "time_warp").unwrap().state.amount = count;
    set_intent(&mut combat, 0, "time_ripple");
    combat
}

fn play(combat: &mut Combat, instance: u64, target: Option<u64>) {
    combat.step_full(&CombatAction::PlayCard {
        instance,
        target: target.map(TargetId),
    }).unwrap_or_else(|error| panic!("play {instance}: {error:?}; legal={:?}; fault={:?}",
        combat.legal_actions_full(), combat.fault_record()));
}

fn end(combat: &mut Combat) {
    combat.step_full(&CombatAction::EndTurn).unwrap();
}

fn set_intent(combat: &mut Combat, slot: usize, name: &'static str) {
    let intent = ActorIntentId::Full(FullIntentId::from_name(name));
    let enemy = &mut combat.state.enemies[slot];
    enemy.intent_id = intent.clone();
    enemy.selected_move = intent.clone();
    enemy.move_history.push(intent);
    effects::refresh_enemy_values(&mut combat.state).unwrap();
}

fn amount(combat: &Combat, slot: usize, id: &str) -> i32 {
    combat.state.enemies[slot].full_powers.iter()
        .find(|power| power.id.0 == id).map_or(0, |power| power.state.amount)
}

fn player_power(combat: &Combat, id: &str) -> i32 {
    combat.state.player_full_powers.iter()
        .find(|power| power.id.0 == id).map_or(0, |power| power.state.amount)
}

fn full_power(id: &'static str, amount: i32) -> FullPowerEntry {
    FullPowerEntry { id: FullPowerId(id), state: PowerState {
        amount, skip_first: false, monster_source_grace: false,
    } }
}

fn hook_count(combat: &Combat, hook: Hook) -> usize {
    combat.state.hook_events.iter().filter(|event| event.hook == hook).count()
}

fn damage_hooks(combat: &Combat, slot: usize) -> usize {
    let target = Actor::Enemy(combat.state.enemies[slot].instance_id.clone());
    combat.state.hook_events.iter()
        .filter(|event| event.hook == Hook::Damage && event.target.as_ref() == Some(&target))
        .count()
}

fn place(combat: &mut Combat, instance: u64, zone: SelectionSourceZone) {
    let id = handle(instance);
    let take = |pile: &mut Vec<model::OwnedCard>| pile.iter().position(|card| card.handle() == id).map(|i| pile.remove(i));
    let card = take(&mut combat.state.piles.hand)
        .or_else(|| take(&mut combat.state.piles.draw))
        .or_else(|| take(&mut combat.state.piles.discard))
        .or_else(|| take(&mut combat.state.piles.exhaust))
        .expect("requested owned card exists");
    match zone {
        SelectionSourceZone::Hand => combat.state.piles.hand.push(card),
        SelectionSourceZone::Draw => combat.state.piles.draw.push(card),
        SelectionSourceZone::Discard => combat.state.piles.discard.push(card),
        _ => unreachable!(),
    }
    piles::refresh_hand_values(&mut combat.state);
}

fn instance_of(combat: &Combat, id: CardId) -> u64 {
    piles::owned_full_cards(&combat.state).find(|card| card.id == id)
        .expect("owned card exists").instance.0
}

#[test]
fn m25_w15_timewarp_turn_control() {
    // m25_powers_stacked_turn_endings already asserts every Binding 7 Vault ruling
    // item: SHA-256 4dfdfb39f2f6b93ef600f2d344bf6ade9b1ee2ad6171cfe5e6cb7f68a40e96a7.
    // Extend that coverage with the additional turn-control card and play-source cases.
    // Meditate retrieves a discard card, requests Calm, then ends the turn
    // (watcher research:111); Time Warp counts the completed card use and resets at 12
    // while queuing Strength +2 (M8 static findings:365-370).
    let mut meditate = time_eater(11, &[(Meditate, 0), (DefendPurple, 0), (StrikePurple, 0)], 3);
    place(&mut meditate, 2, SelectionSourceZone::Discard);
    place(&mut meditate, 3, SelectionSourceZone::Discard);
    let played_before = hook_count(&meditate, Hook::CardPlayed);
    play(&mut meditate, 1, None);
    assert!(meditate.public_full().unwrap().selection.is_some(), "Meditate selects its discard return");
    meditate.step_full(&CombatAction::SelectCard { instance: 2 }).unwrap();
    meditate.step_full(&CombatAction::ConfirmSelection).unwrap();
    assert_eq!((meditate.state.turn, amount(&meditate, 0, "time_warp"), amount(&meditate, 0, "strength")), (2, 0, 2));
    assert_eq!(hook_count(&meditate, Hook::CardPlayed) - played_before, 1);
    assert_eq!(hook_count(&meditate, Hook::TurnEnded), 1, "Meditate and the twelfth-card barrier end once");

    // Omniscience costs one card play; the selected card is played twice free
    // (implementation-notes.json:76-78), so 8 + 1 + 2 reaches 11. One ordinary
    // twelfth card then resets the counter and requests the one normal early end.
    let mut omni = time_eater(8, &[(Omniscience, 0), (DefendPurple, 0), (StrikePurple, 0)], 9);
    place(&mut omni, 2, SelectionSourceZone::Draw);
    let omni_plays = hook_count(&omni, Hook::CardPlayed);
    play(&mut omni, 1, None);
    assert_eq!(amount(&omni, 0, "time_warp"), 8, "Omniscience's selection resolves before its after-use counter");
    omni.step_full(&CombatAction::SelectCard { instance: 2 }).unwrap();
    assert_eq!(amount(&omni, 0, "time_warp"), 11, "Omniscience's two child plays count separately");
    assert_eq!(hook_count(&omni, Hook::CardPlayed) - omni_plays, 3);
    play(&mut omni, 3, Some(1));
    assert_eq!((omni.state.turn, amount(&omni, 0, "time_warp"), amount(&omni, 0, "strength")), (2, 0, 2));
    assert_eq!(hook_count(&omni, Hook::CardPlayed) - omni_plays, 4);
    assert_eq!(hook_count(&omni, Hook::TurnEnded), 1);

    // Foreign Influence offers distinct attacks; its selected output is a real M25
    // combat card (watcher research:106; C5 F1). At 11, playing that output is the
    // twelfth card and must run the same Time Warp reset/Strength producer.
    let mut foreign = time_eater(10, &[(ForeignInfluence, 0), (DefendPurple, 0)], 9);
    play(&mut foreign, 1, None);
    foreign.state.silent.card_offer.as_mut().expect("Foreign Influence offer").cards = vec![Anger, BodySlam, Cleave];
    foreign.step_full(&CombatAction::ChooseCardOffer { index: 0 }).unwrap();
    assert_eq!(amount(&foreign, 0, "time_warp"), 11);
    let anger = instance_of(&foreign, Anger);
    play(&mut foreign, anger, Some(1));
    assert_eq!((foreign.state.turn, amount(&foreign, 0, "time_warp"), amount(&foreign, 0, "strength")), (2, 0, 2));
    assert_eq!(hook_count(&foreign, Hook::CardPlayed), 2, "Foreign Influence and its played output each count once");
    assert_eq!(hook_count(&foreign, Hook::TurnEnded), 1);
}

#[test]
fn m25_w16_curiosity_new_powers() {
    // Curiosity queues Strength 1 for each actual Power use (M8 static findings:401-404).
    // The M24 row already covers its accepted old powers and Skill exclusions; this adds
    // Fasting, Study, Deva Form and the generated Omega Power from Beta.
    let mut combat = watcher(entry(EncounterId::AwakenedOne), &[
        (Fasting, 0), (Study, 0), (DevaForm, 0),
    ], 9);
    assert_eq!((combat.state.enemies[2].hp, amount(&combat, 2, "curiosity"), amount(&combat, 2, "strength")), (300, 1, 0));
    for (instance, card, expected) in [(1, Fasting, 1), (2, Study, 2), (3, DevaForm, 3)] {
        play(&mut combat, instance, None);
        assert_eq!(amount(&combat, 2, "strength"), expected, "{card:?} is one actual Power use");
    }
    assert_eq!(combat.state.played_powers.len(), 3);

    // Beta's generated Omega is its own actual Power use; Beta itself is a Skill.
    let mut omega_combat = watcher(entry(EncounterId::AwakenedOne), &[(Beta, 0)], 9);
    play(&mut omega_combat, 1, None);
    assert_eq!(amount(&omega_combat, 2, "strength"), 0, "Beta is a Skill, not a Curiosity trigger");
    let omega = instance_of(&omega_combat, Omega);
    place(&mut omega_combat, omega, SelectionSourceZone::Hand);
    play(&mut omega_combat, omega, None);
    assert_eq!(amount(&omega_combat, 2, "strength"), 1, "Omega is one actual Power use");
    assert_eq!(player_power(&omega_combat, "omega"), 50, "Omega stores its 50 end-turn damage amount");
    assert_eq!(omega_combat.state.played_powers.len(), 1);

    // These three cards are Skills and must not trigger Curiosity.
    let mut skills = watcher(entry(EncounterId::AwakenedOne), &[
        (Wish, 0), (SimmeringFury, 0), (WreathOfFlame, 0),
    ], 9);
    play(&mut skills, 1, None);
    assert!(skills.legal_actions_full().unwrap().contains(&CombatAction::ChooseCardOffer { index: 0 }));
    skills.step_full(&CombatAction::ChooseCardOffer { index: 0 }).unwrap();
    play(&mut skills, 2, None);
    play(&mut skills, 3, None);
    assert_eq!(amount(&skills, 2, "strength"), 0, "Wish, Simmering Fury, and Wreath of Flame are Skills");
    assert!(skills.state.played_powers.is_empty());
    assert!(engine::validate_runtime(&combat.state).is_ok());
}

#[test]
fn m25_w17_mark_intangible_and_half_dead() {
    // Pressure Points' Mark is HP loss, not Attack damage (implementation-notes.json:83-85;
    // watcher research:87,502-507). Nemesis Intangible caps positive damage at one
    // (M8 static findings:181-192); M14's accepted Poison HP_LOSS case asserts that cap
    // for a real Nemesis turn (m14_silent_interactions.rs:510-526).
    let mut nemesis = watcher(entry(EncounterId::Nemesis), &[(PressurePoints, 0), (DefendPurple, 0)], 3);
    set_intent(&mut nemesis, 0, "nemesis_burn");
    end(&mut nemesis);
    assert_eq!(amount(&nemesis, 0, "intangible"), 1);
    place(&mut nemesis, 1, SelectionSourceZone::Hand);
    let hp = nemesis.state.enemies[0].hp;
    play(&mut nemesis, 1, Some(1));
    assert_eq!((hp - nemesis.state.enemies[0].hp, amount(&nemesis, 0, "mark"), amount(&nemesis, 0, "intangible")), (1, 8, 1));

    // Existing Marks on other Darklings also trigger; a lethal Mark HP loss reaches
    // Darkling half-death, not ordinary Victory. The lifecycle is the same one pinned by
    // m25_state_judgment_and_omniscience under F9 (C3 report; M8 findings Task2).
    let mut darklings = watcher(entry(EncounterId::ThreeDarklings), &[(PressurePoints, 0)], 9);
    darklings.state.enemies[0].hp = 40;
    darklings.state.enemies[1].hp = 8;
    darklings.state.enemies[2].hp = 40;
    darklings.state.enemies[0].full_powers.push(full_power("mark", 2));
    darklings.state.enemies[2].full_powers.push(full_power("mark", 3));
    let before = darklings.state.enemies.iter().map(|enemy| enemy.hp).collect::<Vec<_>>();
    play(&mut darklings, 1, Some(2));
    assert_eq!((before[0] - darklings.state.enemies[0].hp, darklings.state.enemies[0].life), (2, ActorLife::Active));
    assert_eq!((darklings.state.enemies[1].hp, darklings.state.enemies[1].life, amount(&darklings, 1, "mark")), (0, ActorLife::HalfDead, 0));
    assert_eq!((before[2] - darklings.state.enemies[2].hp, darklings.state.outcome), (3, None));

    // F9's half-dead target boundary: a Judgment card does not expose a half-dead
    // Darkling as a legal target. The accepted C3 test covers an active Darkling's
    // Judgment kill entering half-death through the shared lifecycle.
    let mut judgment = watcher(entry(EncounterId::ThreeDarklings), &[(Judgment, 0)], 3);
    let first_hp = judgment.state.enemies[0].hp;
    hit(&mut judgment, 0, first_hp);
    assert_eq!(judgment.state.enemies[0].life, ActorLife::HalfDead);
    let targets = judgment.legal_actions_full().unwrap().into_iter().filter_map(|action| match action {
        CombatAction::PlayCard { instance: 1, target: Some(target) } => Some(target.0),
        _ => None,
    }).collect::<Vec<_>>();
    assert_eq!(targets, vec![2, 3], "only the two active Darklings are Judgment targets");
}

fn assert_writhing_reactions(combat: &mut Combat, instance: u64, target: Option<u64>, hits: usize, block: i32, final_malleable: i32) {
    let hp = combat.state.enemies[0].hp;
    let moves = combat.state.enemies[0].move_history.len();
    let damages = damage_hooks(combat, 0);
    play(combat, instance, target);
    assert!(combat.state.enemies[0].hp < hp, "every fixture card deals positive NORMAL damage");
    assert_eq!(damage_hooks(combat, 0) - damages, hits, "one actual enemy damage callback per source hit");
    assert_eq!(combat.state.enemies[0].move_history.len() - moves, hits, "Reactive queues one move per qualifying hit");
    assert_eq!((combat.state.enemies[0].block, amount(combat, 0, "malleable")), (block, final_malleable),
        "Malleable queues its current Block and increments once per hit");
}

#[test]
fn m25_w18_writhing_new_multi_hits() {
    // Writhing Mass A0 starts HP160 with Malleable4; Reactive and Malleable each run once
    // for each qualifying NORMAL hit (M8 findings:141,156-163). Hit counts and attack
    // faces are literal research/design values: Tantrum 3 (watcher:125), Flying Sleeves 2
    // (:83), Ragnarok 5 (:150), Expunger's stored X (design §5.1), Bowling Bash one hit
    // per live monster (:74; this encounter has one).
    let mut tantrum = watcher(entry(EncounterId::WrithingMass), &[(Tantrum, 0)], 9);
    assert_writhing_reactions(&mut tantrum, 1, Some(1), 3, 12, 6);

    let mut sleeves = watcher(entry(EncounterId::WrithingMass), &[(FlyingSleeves, 0)], 9);
    assert_writhing_reactions(&mut sleeves, 1, Some(1), 2, 7, 5);

    let mut ragnarok = watcher(entry(EncounterId::WrithingMass), &[(Ragnarok, 0)], 9);
    assert_writhing_reactions(&mut ragnarok, 1, None, 5, 25, 8);

    let mut conjure = watcher(entry(EncounterId::WrithingMass), &[(ConjureBlade, 0)], 3);
    play(&mut conjure, 1, None);
    let expunger = instance_of(&conjure, Expunger);
    assert_eq!(piles::owned_full_cards(&conjure.state).find(|card| card.instance.0 == expunger).unwrap().misc, 3);
    place(&mut conjure, expunger, SelectionSourceZone::Hand);
    conjure.state.player.energy = 1;
    assert_writhing_reactions(&mut conjure, expunger, Some(1), 3, 12, 6);

    let mut bowling = watcher(entry(EncounterId::WrithingMass), &[(BowlingBash, 0)], 9);
    assert_writhing_reactions(&mut bowling, 1, Some(1), 1, 3, 4);

    // Pressure Points' HP_LOSS is not an Attack callback: Mark changes HP while
    // neither Reactive nor Malleable reacts (M8 findings:156-163; watcher:87,502-507).
    let mut mark = watcher(entry(EncounterId::WrithingMass), &[(PressurePoints, 0)], 9);
    let before = mark.state.enemies[0].hp;
    let moves = mark.state.enemies[0].move_history.len();
    play(&mut mark, 1, Some(1));
    assert_eq!((before - mark.state.enemies[0].hp, mark.state.enemies[0].block,
        amount(&mark, 0, "malleable"), mark.state.enemies[0].move_history.len() - moves), (8, 0, 3, 0));
}

#[test]
fn m25_w19_slow_new_multi_hits() {
    // Giant Head Slow starts at zero and increments once in after-use, after the card's
    // queued payload (M8 static findings:190-203). The three multi-hit cards cause 2, 5,
    // and 3 damage callbacks, but Slow advances by one after each card.
    let mut combat = watcher(entry(EncounterId::GiantHead), &[
        (FlyingSleeves, 0), (Ragnarok, 0), (Tantrum, 0),
    ], 9);
    assert_eq!((combat.state.enemies[0].hp, amount(&combat, 0, "slow")), (500, 0));
    for (instance, target, expected_hits, expected_slow) in [
        (1, Some(1), 2, 1), (2, None, 5, 2), (3, Some(1), 3, 3),
    ] {
        let damages = damage_hooks(&combat, 0);
        let cards_played = hook_count(&combat, Hook::CardPlayed);
        play(&mut combat, instance, target);
        assert_eq!(damage_hooks(&combat, 0) - damages, expected_hits);
        assert_eq!(amount(&combat, 0, "slow"), expected_slow, "one Slow increment after card {instance}");
        assert_eq!(hook_count(&combat, Hook::CardPlayed) - cards_played, 1);
    }
    assert_eq!(amount(&combat, 0, "slow"), 3, "ten separate hits are three completed card uses");
}

#[test]
fn m25_w20_awakened_rebirth_mark_and_omega() {
    // Awakened One starts at 300 HP with Curiosity and its phase-one cannot-lose guard;
    // first depletion removes DEBUFFs including Mark, Block Return, and Energy Down, then
    // rebirth heals the same actor to 300 (M8 static findings:395-410). C4/a4/report.md:15
    // classifies those three M25 IDs as debuffs; this cross-owner regression must fail if
    // only Awakened's branch is special-cased for Mark. Omega deals non-Attack damage at
    // player end turn (implementation-notes.json:8-10).
    let mut combat = watcher(entry(EncounterId::AwakenedOne), &[(PressurePoints, 0), (Beta, 0)], 9);
    combat.state.enemies[2].hp = 58;
    combat.state.enemies[2].full_powers.extend([
        full_power("block_return", 1), full_power("energy_down", 1),
    ]);
    play(&mut combat, 1, Some(3));
    assert_eq!((combat.state.enemies[2].hp, amount(&combat, 2, "mark")), (50, 8));
    play(&mut combat, 2, None); // Beta creates Omega through its normal child route.
    let omega = instance_of(&combat, Omega);
    place(&mut combat, omega, SelectionSourceZone::Hand);
    play(&mut combat, omega, None);
    assert_eq!(player_power(&combat, "omega"), 50, "Omega stores its 50 end-turn damage amount");
    assert_eq!(combat.state.enemies[2].hp, 50);

    // Omega's 50 end-turn damage depletes phase one. The shared Awakened lifecycle
    // rebirths before settlement; there is no premature Victory and Mark is removed.
    end(&mut combat);
    assert_eq!((combat.state.enemies[2].hp, combat.state.enemies[2].max_hp,
        combat.state.enemies[2].life, amount(&combat, 2, "mark"),
        amount(&combat, 2, "block_return"), amount(&combat, 2, "energy_down"), combat.state.outcome),
        (300, 300, ActorLife::Active, 0, 0, 0, None));
    assert_eq!(hook_count(&combat, Hook::CombatSettled), 0);
    assert!(engine::validate_runtime(&combat.state).is_ok());
}
