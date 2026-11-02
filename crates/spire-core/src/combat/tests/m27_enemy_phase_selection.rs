//! Melange opens Scry whenever a shuffle triggers it and suspends whatever
//! queue it interrupted: an enemy turn, a potion, or a post-selection draw.
use super::super::*;
use crate::{content::{CardId, Character, EncounterId, PotionId, RelicId}, items::CombatItemContext, run::RunRng};

fn watcher_shield_spear_with_melange() -> Combat { watcher_shield_spear_with(vec![None; 3]) }

fn watcher_shield_spear_with(potions: Vec<Option<PotionId>>) -> Combat {
    let cards = (1..=20).map(|instance| {
        let id = if instance % 2 == 0 { CardId::StrikePurple } else { CardId::DefendPurple };
        permanent_live_card(id, CardInstanceId(instance), 0)
    }).collect();
    let entry = enemies::setup(EncounterId::ShieldAndSpear, 4, 0,
        &mut RunRng::new([0xF9; 32], rng::Partition::Test, 0),
        enemies::SetupContext {
            player: PlayerState { hp: 80, max_hp: 80, block: 0, energy: 3, max_energy: 3,
                draw_per_turn: 5, powers: vec![] },
            player_powers: vec![], character: Character::Watcher, cards, opening_draw: 5, encounter_visit: 0,
        }).unwrap();
    let mut items = Some(CombatItemContext::new(
        vec![RelicId::PureWater, RelicId::CentennialPuzzle, RelicId::Melange], potions)
        .with_complete_cards().with_complete_items());
    let combat = Combat::enter_with_item_context(entry, &mut items).unwrap();
    assert!(items.is_none());
    combat
}

fn empty_draw_with_recycled_deck(combat: &mut Combat) {
    let mut all = std::mem::take(&mut combat.state.piles.hand);
    all.extend(std::mem::take(&mut combat.state.piles.draw));
    all.extend(std::mem::take(&mut combat.state.piles.discard));
    all.extend(std::mem::take(&mut combat.state.piles.exhaust));
    combat.state.piles.discard = all;
    combat.state.public.known_draw.clear();
    combat.state.public.known_draw_bottom.clear();
}

fn empty_draw_keeping_hand(combat: &mut Combat) {
    let mut rest = std::mem::take(&mut combat.state.piles.draw);
    rest.extend(std::mem::take(&mut combat.state.piles.exhaust));
    combat.state.piles.discard.extend(rest);
    combat.state.public.known_draw.clear();
    combat.state.public.known_draw_bottom.clear();
}

fn confirm_first_scry_candidate(combat: &mut Combat) {
    let candidate = combat.state.pending_selection.as_ref().unwrap().candidates[0].as_str().parse().unwrap();
    combat.step_full(&CombatAction::SelectCard { instance: candidate }).unwrap();
    combat.step_full(&CombatAction::ConfirmSelection).unwrap();
}

fn assert_scry_boundary(combat: &Combat, resume: Phase) {
    assert_eq!(combat.state.phase, Phase::Selection);
    assert_eq!(combat.state.pending_selection.as_ref().map(|selection|selection.resume.phase), Some(resume));
    let public=combat.public_full().unwrap();
    assert_eq!(public.phase, Phase::Selection);
    assert_eq!(public.selection_resume_phase, Some(resume));
    assert!(public.selection.as_ref().is_some_and(|selection|selection.operation==SelectionOperation::Scry));
    assert!(combat.legal_actions_full().unwrap().contains(&CombatAction::ConfirmSelection));
    assert!(combat.validate_runtime_for_test().is_ok());
}

#[test]
fn m27_centennial_melange_scry_is_offered_during_enemy_turn() {
    let mut combat = watcher_shield_spear_with_melange();
    empty_draw_with_recycled_deck(&mut combat);
    let result = combat.step_full(&CombatAction::EndTurn);
    assert!(result.is_ok(), "Centennial Puzzle's enemy-turn shuffle should pause for Melange Scry; got {result:?}, fault={:?}", combat.state.fault);
    assert_scry_boundary(&combat, Phase::EnemyTurn);

    let candidate = combat.state.pending_selection.as_ref().unwrap().candidates[0].as_str().parse().unwrap();
    combat.step_full(&CombatAction::SelectCard { instance: candidate }).unwrap();
    combat.step_full(&CombatAction::ConfirmSelection).unwrap();
    assert_eq!(combat.state.phase, Phase::Decision);
    assert_eq!(combat.state.turn, 2, "enemy turn and EndRound finish after Scry");
    assert!(combat.validate_runtime_for_test().is_ok());
}

#[test]
fn m27_melange_scry_resumes_remaining_enemy_move_and_end_round() {
    let mut combat = watcher_shield_spear_with_melange();
    empty_draw_with_recycled_deck(&mut combat);
    let spear = combat.state.enemies.iter().find(|enemy|
        enemy.enemy_id == ActorEnemyId::Full(crate::content::EnemyId::SpireSpear)).unwrap().instance_id.clone();
    let before_hp = combat.state.player.hp;
    combat.state.turn = 2;
    combat.state.phase = Phase::EnemyTurn;
    combat.state.pending.clear();
    combat.state.pending.push_back(Effect { source: Source::Relic(RelicId::CentennialPuzzle), target: Some(Actor::Player), kind: EffectKind::Draw(3) });
    combat.state.pending.push_back(Effect { source: Source::None, target: Some(Actor::Enemy(spear.clone())), kind: EffectKind::EnemyMove });
    combat.state.pending.push_back(Effect { source: Source::None, target: None, kind: EffectKind::EndRound });

    assert_eq!(engine::resolve(&mut combat.state).status, TransitionStatus::Selection);
    assert_scry_boundary(&combat, Phase::EnemyTurn);
    assert!(combat.state.pending.iter().any(|effect| effect.kind == EffectKind::EnemyMove
        && effect.target == Some(Actor::Enemy(spear.clone()))), "the Spear move stays queued behind the modal decision");

    let candidate = combat.state.pending_selection.as_ref().unwrap().candidates[0].as_str().parse().unwrap();
    combat.step_full(&CombatAction::SelectCard { instance: candidate }).unwrap();
    combat.step_full(&CombatAction::ConfirmSelection).unwrap();

    assert_eq!(combat.state.phase, Phase::Decision, "the following player decision is ready");
    assert_eq!(combat.state.turn, 3, "the suspended enemy turn and EndRound ran exactly once");
    assert!(combat.state.player.hp < before_hp, "the queued Spear move resolves after Scry");
    assert!(!combat.state.pending.iter().any(|effect| effect.kind == EffectKind::EndRound));
    assert!(combat.validate_runtime_for_test().is_ok());
}

#[test]
fn m27_snecko_oil_draw_shuffle_opens_melange_scry_and_resumes_the_potion() {
    let mut combat = watcher_shield_spear_with(vec![Some(PotionId::SneckoOil), None, None]);
    empty_draw_with_recycled_deck(&mut combat);
    let result = combat.step_full(&CombatAction::UsePotion { slot: 0, target: None });
    assert!(result.is_ok(), "Snecko Oil's draw shuffle should pause for Melange Scry; got {result:?}, fault={:?}", combat.state.fault);
    assert_eq!(combat.item_context().unwrap().potions()[0], None);
    assert_scry_boundary(&combat, Phase::Decision);
    assert!(combat.state.pending.iter().any(|effect| effect.kind == EffectKind::Potion(PotionEffect::RandomizeHandCosts)),
        "the potion's cost randomization stays queued behind the modal decision");

    confirm_first_scry_candidate(&mut combat);
    assert_eq!(combat.state.phase, Phase::Decision);
    assert!(combat.state.pending.is_empty(), "the interrupted potion finished after Scry");
    assert_eq!(combat.state.piles.hand.len(), 5);
    assert!(combat.validate_runtime_for_test().is_ok());
}

#[test]
fn m27_gamblers_brew_redraw_shuffle_opens_melange_scry_and_resumes_the_draw() {
    let mut combat = watcher_shield_spear_with(vec![Some(PotionId::GamblersBrew), None, None]);
    empty_draw_keeping_hand(&mut combat);
    let hand = combat.state.piles.hand.iter().map(|card| card.handle()).collect::<Vec<_>>();
    combat.step_full(&CombatAction::UsePotion { slot: 0, target: None }).unwrap();
    for card in hand.iter().take(3) {
        combat.step_full(&CombatAction::SelectCard { instance: card.as_str().parse().unwrap() }).unwrap();
    }
    let result = combat.step_full(&CombatAction::ConfirmSelection);
    assert!(result.is_ok(), "Gambler's Brew's redraw shuffle should pause for Melange Scry; got {result:?}, fault={:?}", combat.state.fault);
    assert_scry_boundary(&combat, Phase::Decision);

    confirm_first_scry_candidate(&mut combat);
    assert_eq!(combat.state.phase, Phase::Decision);
    assert!(combat.state.pending.is_empty());
    assert_eq!(combat.state.piles.hand.len(), hand.len(), "all three discarded cards are redrawn after Scry");
    assert!(combat.validate_runtime_for_test().is_ok());
}
