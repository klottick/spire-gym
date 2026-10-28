use super::super::*;
use super::full_runtime::entered_combat;
use crate::content::{CardId, Catalog, Character};
use model::OwnedCard;

fn id(token: &str) -> CardId {
    Catalog::standard().cards().iter().find(|row| row.id.as_str() == token)
        .unwrap_or_else(|| panic!("missing executable green card {token}")).id
}

fn fight(token: &str, upgrades: u32) -> Combat {
    let mut combat = entered_combat(12, 12, |card| {
        *card = permanent_live_card(id(token), card.instance, upgrades);
    });
    combat.state.character = characters::CharacterState::for_character(Character::Silent);
    combat.state.enemies[0].hp = 500;
    combat.state.enemies[0].max_hp = 500;
    combat
}

fn add(combat: &mut Combat, token: &str, instance: u64) {
    let card = permanent_live_card(id(token), CardInstanceId(instance), 0);
    assert!(combat.state.card_ids_issued.insert(card_state::handle(card.instance)));
    combat.state.piles.hand.push(OwnedCard::Full(card));
}

fn play(combat: &mut Combat, instance: u64, target: bool) {
    combat.step_full(&CombatAction::PlayCard {
        instance, target: target.then_some(TargetId(1)),
    }).unwrap();
}

fn power(combat: &Combat, id: &str) -> i32 {
    combat.state.player_full_powers.iter().find(|entry| entry.id.0 == id)
        .map_or(0, |entry| entry.state.amount)
}

fn enemy_power(combat: &Combat, id: &str) -> i32 {
    combat.state.enemies[0].full_powers.iter().find(|entry| entry.id.0 == id)
        .map_or(0, |entry| entry.state.amount)
}

fn select_first(combat: &mut Combat) {
    let instance = combat.state.pending_selection.as_ref().unwrap().candidates[0]
        .as_str().parse().unwrap();
    combat.step_full(&CombatAction::SelectCard { instance }).unwrap();
}

#[test]
fn m15_silent_x_attacks_and_upgraded_zero_x_keep_source_amounts() {
    for (upgrades, per_hit) in [(0, 7), (1, 10)] {
        let mut skewer = fight("skewer", upgrades);
        skewer.state.player.energy = 3;
        play(&mut skewer, 3, true);
        assert_eq!(skewer.state.enemies[0].hp, 500 - 3 * per_hit,
            "Skewer resolves the captured X count with its base/upgraded hit value");
        assert_eq!(skewer.state.player.energy, 0, "the physical X play pays once");
        assert!(skewer.state.play_frames.is_empty());
    }

    let mut doppelganger = fight("doppelganger", 1);
    doppelganger.state.player.energy = 0;
    play(&mut doppelganger, 3, false);
    assert_eq!(power(&doppelganger, "draw_card_next_turn"), 1,
        "Doppelganger+ applies its +1 even when captured X is zero");
    assert_eq!(power(&doppelganger, "energized"), 1);

    let mut malaise = fight("malaise", 1);
    malaise.state.player.energy = 0;
    play(&mut malaise, 3, true);
    assert_eq!(enemy_power(&malaise, "strength"), -1,
        "Malaise+ applies its +1 at captured X zero");
    assert_eq!(enemy_power(&malaise, "weak"), 1);
    assert_eq!(malaise.state.piles.exhaust.iter().filter(|card| card.handle().as_str() == "3").count(), 1);
}

#[test]
fn m15_silent_free_x_wrapper_keeps_captured_energy() {
    let mut skewer = fight("skewer", 0);
    skewer.state.player.energy = 3;
    skewer.state.piles.hand.iter_mut().find(|card| card.handle().as_str() == "3")
        .unwrap().full_mut().unwrap().local.free_play = true;
    play(&mut skewer, 3, true);
    assert_eq!(skewer.state.enemies[0].hp, 479,
        "a free wrapper preserves the pre-payment X value");
    assert_eq!(skewer.state.player.energy, 3, "the free wrapper does not spend current energy");
    assert!(!skewer.state.piles.discard.iter().find(|card| card.handle().as_str() == "3")
        .unwrap().full().unwrap().local.free_play, "ordinary commit clears autoplay free_play");
}

#[test]
fn m15_silent_burst_repeats_next_one_or_two_skills_and_suspends_once_per_use() {
    for (upgrades, repetitions) in [(0, 1), (1, 2)] {
        let mut combat = fight("burst", upgrades);
        combat.state.player.energy = 9;
        add(&mut combat, "setup", 100);
        add(&mut combat, "backflip", 101);
        play(&mut combat, 3, false);
        assert_eq!(power(&combat, "burst"), repetitions,
            "Burst installs its source-qualified one/two Skill uses");

        play(&mut combat, 100, false);
        assert!(combat.state.pending_selection.is_some(), "Setup suspends its original use");
        select_first(&mut combat);
        assert!(combat.state.pending_selection.is_some(), "the repeated Setup has its own selection");
        select_first(&mut combat);
        assert!(combat.state.pending_selection.is_none());
        assert!(combat.state.play_frames.is_empty(), "the repeat resumes the suspended parent once");
        assert_eq!(combat.state.silent.played_this_turn.iter().filter(|id| **id == CardId::Setup).count(), 2);
        assert_eq!(power(&combat, "burst"), repetitions - 1,
            "the purge repeat does not consume a second Burst charge");

        play(&mut combat, 101, false);
        let expected_backflip_uses = if repetitions == 2 { 2 } else { 1 };
        assert_eq!(combat.state.silent.played_this_turn.iter().filter(|id| **id == CardId::Backflip).count(),
            expected_backflip_uses, "each remaining charge repeats one actual Skill");
        assert_eq!(power(&combat, "burst"), 0);
        assert_eq!(combat.state.player.energy, 6,
            "repeat children do not pay card energy");
        assert!(combat.state.play_frames.is_empty());
    }
}

#[test]
fn m15_silent_burst_x_child_uses_parent_capture_without_child_payment() {
    let mut combat = fight("burst", 1);
    combat.state.player.energy = 4;
    add(&mut combat, "doppelganger", 100);
    play(&mut combat, 3, false);
    play(&mut combat, 100, false);
    assert_eq!(combat.state.player.energy, 0, "the original X Skill pays its captured three");
    assert_eq!(power(&combat, "draw_card_next_turn"), 6,
        "the purge child uses captured X=3 while live energy is zero");
    assert_eq!(power(&combat, "energized"), 6);
    assert_eq!(combat.state.silent.played_this_turn.iter().filter(|id| **id == CardId::Doppelganger).count(), 2);
    assert_eq!(power(&combat, "burst"), 1, "the repeat child does not consume a second charge");
    assert!(combat.state.play_frames.is_empty());

    let mut free = fight("doppelganger", 0);
    free.state.player.energy = 2;
    free.state.piles.hand.iter_mut().find(|card| card.handle().as_str() == "3")
        .unwrap().full_mut().unwrap().local.free_play = true;
    play(&mut free, 3, false);
    assert_eq!(power(&free, "draw_card_next_turn"), 2,
        "free_play waives payment while preserving captured X=2");
    assert_eq!(free.state.player.energy, 2);
}

#[test]
fn m15_silent_storm_snapshots_before_reflex_draws_and_overflows_upgraded_shivs() {
    let mut combat = fight("storm_of_steel", 1);
    combat.state.player.energy = 9;
    add(&mut combat, "reflex", 100);
    add(&mut combat, "reflex", 101);
    for instance in 102..107 { add(&mut combat, "wound", instance); }
    assert_eq!(combat.state.piles.hand.len(), 10, "Storm begins with a full hand including itself");

    play(&mut combat, 3, false);

    let shivs = combat.state.piles.hand.iter().chain(&combat.state.piles.discard)
        .filter_map(OwnedCard::full).filter(|card| card.id == CardId::Shiv).collect::<Vec<_>>();
    assert_eq!(shivs.len(), 9, "Storm creates one Shiv for each card captured before Reflex reacts");
    assert!(shivs.iter().all(|card| card.upgrades == 1), "Storm+ creates upgraded Shivs");
    assert_eq!(combat.state.piles.hand.iter().filter_map(OwnedCard::full).filter(|card| card.id == CardId::Shiv).count(), 6);
    assert_eq!(combat.state.piles.discard.iter().filter_map(OwnedCard::full).filter(|card| card.id == CardId::Shiv).count(), 3,
        "the shared generated-card insertion owner preserves hand overflow in discard");
    assert_eq!(combat.state.piles.hand.len(), 10);
    assert!(combat.state.pending_selection.is_none());
    assert!(combat.state.play_frames.is_empty());
}

#[test]
fn m15_awakened_clear_cancels_queued_repeats_but_preserves_action_work() {
    use crate::content::EncounterId;

    let mut combat = Combat::enter(super::m8_shapes::entry(EncounterId::AwakenedOne)).unwrap();
    let owner = combat.state.enemies[2].instance_id.clone();
    combat.state.enemies[2].hp = 0;
    combat.state.enemies[2].full_memory = Some(EnemyMemory::AwakenedOne {
        phase: enemies::AwakenedPhase::First,
        first: false,
    });

    let parent = card_state::handle(CardInstanceId(500));
    let child = card_state::handle(CardInstanceId(501));
    combat.state.pending.push_back(Effect {
        source: Source::Card(parent.clone()), target: None,
        kind: EffectKind::Silent(cards::SilentEffect::BurstRepeat {
            child: child.clone(), target: None, energy_on_use: 2, cancelled: false,
        }),
    });
    combat.state.pending.push_back(Effect {
        source: Source::Card(parent), target: None,
        kind: EffectKind::Ironclad(cards::IroncladEffect::DoubleTapRepeat {
            child: card_state::handle(CardInstanceId(502)), target: None,
            energy_on_use: 2, cancelled: false,
        }),
    });
    combat.state.pending.push_back(Effect {
        source: Source::None, target: Some(Actor::Player),
        kind: EffectKind::GainBlock(3),
    });

    assert!(enemies::act3::awakened::down(&mut combat.state, &owner, &Source::None));
    let burst = combat.state.pending.iter().find(|effect|matches!(effect.kind,
        EffectKind::Silent(cards::SilentEffect::BurstRepeat{..}))).unwrap();
    let double_tap = combat.state.pending.iter().find(|effect|matches!(effect.kind,
        EffectKind::Ironclad(cards::IroncladEffect::DoubleTapRepeat{..}))).unwrap();
    assert!(matches!(burst.kind, EffectKind::Silent(cards::SilentEffect::BurstRepeat{cancelled:true,..})));
    assert!(matches!(double_tap.kind, EffectKind::Ironclad(cards::IroncladEffect::DoubleTapRepeat{cancelled:true,..})));
    assert!(combat.state.pending.iter().any(|effect|effect.kind==EffectKind::GainBlock(3)),
        "already queued action work remains after the card queue is cancelled");
    assert_eq!(combat.state.enemies[2].life, ActorLife::HalfDead);
}

#[test]
fn m15_cost_pair_burst_remains_one_when_upgraded() {
    for upgrades in [0, 1] {
        let mut combat = fight("burst", upgrades);
        combat.state.player.energy = 3;
        play(&mut combat, 3, false);
        assert_eq!(combat.state.player.energy, 2,
            "Burst costs one energy at upgrade level {upgrades}");
    }
}

#[test]
fn m15_cost_pair_storm_of_steel_remains_one_when_upgraded() {
    for upgrades in [0, 1] {
        let mut combat = fight("storm_of_steel", upgrades);
        combat.state.player.energy = 3;
        play(&mut combat, 3, false);
        assert_eq!(combat.state.player.energy, 2,
            "Storm of Steel costs one energy at upgrade level {upgrades}");
    }
}
