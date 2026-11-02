//! M27 Main-2 shared item owners. Lane rule bodies remain with their assigned lane.
use crate::combat::{self, Actor, Combat, CombatAction, Effect, EffectKind, FullPowerEntry,
    FullPowerId, GenerationCost, GenerationPool, GenerationWork, PowerEntry,
    PowerState, RelicEffect, Source, TargetId};
use crate::combat::content::PowerId;
use crate::content::{CardId, Character, EncounterId, RelicId};
use crate::items::{dispatch, laws, CombatItemContext};
use crate::run::Run;
use std::collections::BTreeMap;

fn full_combat(run: &mut Run) -> &mut Combat {
    run.state_mut_for_test().combat.as_mut().expect("complete Run has active combat")
}

fn complete_run(relics: Vec<RelicId>, encounter: EncounterId) -> Run {
    crate::items::tests::m27_support::complete_run(Character::Ironclad, relics, vec![None; 3],
        BTreeMap::new(), encounter)
}

fn full_power(id: &'static str, amount: i32) -> FullPowerEntry {
    FullPowerEntry { id: FullPowerId(id), state: PowerState { amount, skip_first: false,
        monster_source_grace: false } }
}

fn relic_effect(run: &mut Run, id: RelicId, kind: RelicEffect) {
    full_combat(run).enqueue_native_effects(vec![Effect { source: Source::Relic(id), target: None,
        kind: EffectKind::Relic(kind) }], None).unwrap();
}

#[test]
fn m27_engine_2_power_attempt_precedes_artifact_and_applied_callback() {
    let _ = dispatch::take_phase_log();
    let mut run = complete_run(vec![RelicId::Ginger, RelicId::Turnip], EncounterId::Cultist);
    for (id, amount) in [("weak", 2), ("frail", 1)] {
        let expected = {
            let current = full_combat(&mut run).item_context().unwrap();
            laws::power_attempt(current, &Actor::Player, id, amount)
        };
        if id == "frail" { full_combat(&mut run).state.player_full_powers.push(full_power("artifact", 1)); }
        let _ = dispatch::take_phase_log();
        full_combat(&mut run).enqueue_native_effects(vec![Effect { source: Source::Actor(Actor::Player),
            target: Some(Actor::Player), kind: EffectKind::ApplyFullPower { id: FullPowerId(id), amount } }], None).unwrap();
        let state = &full_combat(&mut run).state;
        let applied = state.player_full_powers.iter().find(|power| power.id.0 == id).map_or(0, |power| power.state.amount);
        assert_eq!(applied, if expected && id == "weak" { amount } else { 0 }, "{id} policy result");
        let artifact = state.player_full_powers.iter().find(|power| power.id.0 == "artifact").map_or(0, |power| power.state.amount);
        assert_eq!(artifact, if id == "frail" && !expected { 1 } else { 0 }, "Artifact follows the relic attempt gate");
        let applied_phase = dispatch::take_phase_log().contains(&dispatch::ItemPhase::PowerApplied);
        assert_eq!(applied_phase, expected && id == "weak", "PowerApplied is emitted only for a committed power stack");
    }
}

#[test]
fn m27_engine_2_allowed_unplayable_curse_and_status_are_free_and_exhaust() {
    for (relic, card_id) in [(RelicId::BlueCandle, CardId::Clumsy), (RelicId::MedicalKit, CardId::Wound)] {
        let mut run = complete_run(vec![relic], EncounterId::Cultist);
        let combat = full_combat(&mut run);
        let live = combat::complete_live_card(card_id, combat::CardInstanceId(900), 0).unwrap();
        let handle = combat::CardHandle::parse("900").unwrap();
        combat.state.card_ids_issued.insert(handle);
        combat.state.piles.hand.push(combat::model::OwnedCard::Full(live));
        combat.state.player.energy = 0;
        let owned = combat.state.piles.hand.iter().find(|owned| owned.full().is_some_and(|card| card.id == card_id))
            .expect("opening hand contains authored test card");
        let instance = owned.full().unwrap().instance.0;
        let definition = crate::content::Catalog::standard().card(card_id).unwrap();
        let allowed = laws::play_allowed(combat.item_context().unwrap(), dispatch::PhaseCard {
            id: card_id, card_type: definition.card_type, cost_paid: 0 }).allow == Some(true);
        let prepared = combat::cards::prepare(combat, &CombatAction::PlayCard { instance, target: None });
        assert_eq!(prepared.is_ok(), allowed, "{card_id:?} legality follows the owned item rule");
        if let Ok(prepared) = prepared {
            assert_eq!(prepared.paid, 0);
            assert_eq!(prepared.remaining_energy, 0);
            assert!(prepared.effects.is_empty());
            assert!(prepared.exhaust_on_finish);
            if card_id == CardId::Wound {
                run.choose_combat(CombatAction::PlayCard { instance, target: None }).unwrap();
                let combat = full_combat(&mut run);
                assert_eq!(combat.state.player.energy, 0);
                assert!(combat.state.piles.exhaust.iter().any(|owned| owned.handle().as_str() == instance.to_string()),
                    "accepted Status play exhausts at the ordinary finish checkpoint");
            }
        }
    }
}

#[test]
fn m27_engine_2_mummified_hand_zeroes_one_eligible_turn_cost() {
    let mut run = complete_run(vec![RelicId::MummifiedHand], EncounterId::Cultist);
    let combat = full_combat(&mut run);
    let mut hand = Vec::new();
    for owned in &mut combat.state.piles.hand {
        let Some(card) = owned.full_mut() else { continue };
        if hand.is_empty() { card.local.turn_cost = Some(0); }
        else if hand.len() == 1 { card.local.free_to_play_once = true; }
        hand.push(card.instance);
    }
    let eligible = combat.state.piles.hand.iter().filter_map(|owned| owned.full()).filter_map(|card| {
        let base = combat::cards::base_cost(&combat.state, card)?;
        (base > 0 && combat::card_state::effective_cost(card, base).ok()? > 0
            && !card.local.x_cost && !card.local.free_to_play_once && !card.local.free_play)
            .then_some((card.instance, card.local.turn_cost, base))
    }).collect::<Vec<_>>();
    assert!(!eligible.is_empty());
    relic_effect(&mut run, RelicId::MummifiedHand, RelicEffect::SetRandomHandCardCostZeroThisTurn);
    let after = full_combat(&mut run).state.piles.hand.iter().filter_map(|owned| owned.full()).collect::<Vec<_>>();
    let changed = eligible.iter().filter(|(instance, old, _)| after.iter().find(|card| card.instance == *instance)
        .is_some_and(|card| card.local.turn_cost == Some(0) && *old != Some(0))).count();
    assert_eq!(changed, 1, "one card with positive base and turn cost is selected");
    for (instance, old, _) in eligible {
        if old == Some(0) { assert_eq!(after.iter().find(|card| card.instance == instance).unwrap().local.turn_cost, old); }
    }
}

#[test]
fn m27_engine_2_orange_pellets_removes_only_classified_player_debuffs() {
    let mut run = complete_run(vec![RelicId::OrangePellets], EncounterId::Cultist);
    {
        let state = &mut full_combat(&mut run).state;
        state.player_full_powers.extend([full_power("weak", 2), full_power("strength", -2),
            full_power("artifact", 1), full_power("metallicize", 2)]);
        state.player.powers.push(PowerEntry { id: PowerId::Vulnerable,
            state: PowerState { amount: 2, skip_first: false, monster_source_grace: false } });
    }
    relic_effect(&mut run, RelicId::OrangePellets, RelicEffect::RemovePlayerDebuffs);
    let state = &full_combat(&mut run).state;
    for debuff in ["weak", "strength", "vulnerable"] {
        assert!(!state.player_full_powers.iter().any(|power| power.id.0 == debuff));
    }
    assert!(state.player_full_powers.iter().any(|power| power.id.0 == "artifact"));
    assert!(state.player_full_powers.iter().any(|power| power.id.0 == "metallicize"));
    assert!(state.player.powers.is_empty(), "the shared removal owner clears legacy and full lanes");
}

#[test]
fn m27_engine_2_dead_branch_generates_one_ordinary_any_card_to_hand() {
    let mut run = complete_run(vec![RelicId::DeadBranch], EncounterId::Cultist);
    let before = full_combat(&mut run).state.piles.hand.len();
    full_combat(&mut run).enqueue_native_effects(vec![Effect { source: Source::Relic(RelicId::DeadBranch),
        target: Some(Actor::Player), kind: EffectKind::Generate(GenerationWork {
            pool: GenerationPool::Any, count: 1, unique: false, destination: combat::CardDestination::Hand,
            cost: GenerationCost::Ordinary, upgraded: false, offer: None, captured: false, cards: vec![], next: 0,
        }) }], None).unwrap();
    let after = full_combat(&mut run).state.piles.hand.len();
    assert_eq!(after, before + 1);
    assert!(full_combat(&mut run).public_full().unwrap().hand.iter().any(|card|
        crate::content::complete::supports_generated(card.id)));
}

#[test]
fn m27_engine_2_pen_nib_doubles_attack_and_expires_after_use() {
    let mut run = complete_run(vec![RelicId::PenNib], EncounterId::Cultist);
    {
        let state = &mut full_combat(&mut run).state;
        state.player_full_powers.push(full_power("double_damage", 1));
    }
    full_combat(&mut run).enqueue_native_effects(vec![Effect { source: Source::Relic(RelicId::PenNib),
        target: Some(Actor::Player), kind: EffectKind::ApplyFullPower { id: FullPowerId("pen_nib"), amount: 1 } }], None).unwrap();
    assert!(full_combat(&mut run).public_full().unwrap().player.powers.iter()
        .any(|power| power.id.0 == "pen_nib" && power.amount == 1), "Pen Nib uses the visible full-power projection");
    let first = full_combat(&mut run).public_full().unwrap().hand.into_iter()
        .find(|card| card.id == CardId::Strike).expect("opening hand has Strike").instance;
    let before = full_combat(&mut run).state.enemies[0].hp;
    run.choose_combat(CombatAction::PlayCard { instance: first.0, target: Some(TargetId(1)) }).unwrap();
    let after_first = full_combat(&mut run).state.enemies[0].hp;
    assert_eq!(before - after_first, 24, "Pen Nib and double damage multiply for four times Strike damage");
    assert!(!full_combat(&mut run).state.player_full_powers.iter().any(|power| power.id.0 == "pen_nib"));
    assert!(full_combat(&mut run).state.player_full_powers.iter().any(|power| power.id.0 == "double_damage"));
    let second = full_combat(&mut run).public_full().unwrap().hand.into_iter()
        .find(|card| card.id == CardId::Strike).expect("a second Strike remains in the opening hand").instance;
    let before_second = full_combat(&mut run).state.enemies[0].hp;
    run.choose_combat(CombatAction::PlayCard { instance: second.0, target: Some(TargetId(1)) }).unwrap();
    assert_eq!(before_second - full_combat(&mut run).state.enemies[0].hp, 12,
        "the remaining double-damage power applies alone to the next Attack");
}

#[test]
fn m27_engine_2_new_relic_effects_require_complete_owned_context() {
    let mut run = complete_run(vec![RelicId::MummifiedHand], EncounterId::Cultist);
    let combat = full_combat(&mut run);
    combat.state.item_context = Some(CombatItemContext::new(vec![RelicId::MummifiedHand], vec![None; 3]));
    let effect = Effect { source: Source::Relic(RelicId::MummifiedHand), target: None,
        kind: EffectKind::Relic(RelicEffect::SetRandomHandCardCostZeroThisTurn) };
    assert!(combat.preflight_native_effects(std::slice::from_ref(&effect)).is_err());
    let pen = Effect { source: Source::Relic(RelicId::PenNib), target: Some(Actor::Player),
        kind: EffectKind::ApplyFullPower { id: FullPowerId("pen_nib"), amount: 1 } };
    assert!(combat.preflight_native_effects(&[pen]).is_err(), "unowned frozen context cannot activate Pen Nib");
    let _ = laws::power_attempt(combat.item_context().unwrap(), &Actor::Player, "weak", 1);
}

#[test]
fn m27_engine_2_frozen_empty_context_keeps_generic_paths_inert() {
    let mut run = complete_run(vec![], EncounterId::Cultist);
    let baseline = full_combat(&mut run).clone();
    let mut frozen = baseline.clone();
    frozen.state.item_context = Some(CombatItemContext::new(vec![], vec![None; 3]));
    let mut baseline_public = baseline.public_full().unwrap();
    let mut frozen_public = frozen.public_full().unwrap();
    baseline_public.inventory = None;
    frozen_public.inventory = None;
    assert_eq!(baseline_public, frozen_public, "an empty frozen inventory adds no public observation fields");
    let mut baseline_debug = baseline.state.clone();
    let mut frozen_debug = frozen.state.clone();
    baseline_debug.item_context = None;
    frozen_debug.item_context = None;
    assert_eq!(format!("{baseline_debug:?}"), format!("{frozen_debug:?}"),
        "the frozen context adds no combat Debug fields");
    let combat = full_combat(&mut run);
    combat.state.item_context = Some(CombatItemContext::new(vec![], vec![None; 3]));
    combat.enqueue_native_effects(vec![Effect { source: Source::Actor(Actor::Player), target: Some(Actor::Player),
        kind: EffectKind::ApplyFullPower { id: FullPowerId("weak"), amount: 1 } }], None).unwrap();
    assert!(combat.state.player_full_powers.iter().any(|power| power.id.0 == "weak" && power.state.amount == 1),
        "without a relevant complete relic, the ordinary full power path still applies");

    let clumsy = combat::complete_live_card(CardId::Clumsy, combat::CardInstanceId(901), 0).unwrap();
    combat.state.card_ids_issued.insert(combat::CardHandle::parse("901").unwrap());
    combat.state.piles.hand.push(combat::model::OwnedCard::Full(clumsy));
    let rejected = combat::cards::prepare(combat, &CombatAction::PlayCard { instance: 901, target: None });
    assert!(matches!(rejected, Err(crate::run::RunError::InvalidAction)),
        "an unplayable Curse remains blocked without a complete Blue Candle context");

    let enemy = combat.state.enemies[0].clone();
    assert_eq!(combat::cards::normal_damage(&combat.state, 6, 0, false, &enemy).unwrap(), 6,
        "the ordinary damage path has no Pen Nib multiplier without its visible power");

    for (source, kind) in [
        (RelicId::MummifiedHand, EffectKind::Relic(RelicEffect::SetRandomHandCardCostZeroThisTurn)),
        (RelicId::OrangePellets, EffectKind::Relic(RelicEffect::RemovePlayerDebuffs)),
        (RelicId::DeadBranch, EffectKind::Generate(GenerationWork { pool: GenerationPool::Any, count: 1,
            unique: false, destination: combat::CardDestination::Hand, cost: GenerationCost::Ordinary,
            upgraded: false, offer: None, captured: false, cards: vec![], next: 0 })),
        (RelicId::PenNib, EffectKind::ApplyFullPower { id: FullPowerId("pen_nib"), amount: 1 }),
    ] {
        let effect = Effect { source: Source::Relic(source), target: if source == RelicId::DeadBranch || source == RelicId::PenNib {
            Some(Actor::Player) } else { None }, kind };
        assert!(combat.preflight_native_effects(&[effect]).is_err(), "{source:?} stays unreachable in the frozen context");
    }
}
