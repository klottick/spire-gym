//! spire-d8g: engine-level checks behind the 2026-11-16 stress-campaign findings.
//!
//! D1/D4: the original's end-of-battle routine runs Meat on the Bone, then the player's victory
//! callbacks (relics, blights, powers), on a victory and on a Smoke Bomb escape alike; `smoked`
//! only suppresses rewards (data `MEAT_ON_THE_BONE` trigger `end_battle_before_player_victory_hooks`).
use super::super::{CombatAction, FullPowerEntry, FullPowerId, Outcome, PowerState, TargetId};
use crate::content::{Character, EncounterId, PotionId, RelicId};
use crate::items::tests::m27_support;
use crate::run::Run;
use std::collections::BTreeMap;

fn run_at(character: Character, relics: Vec<RelicId>, potions: Vec<Option<PotionId>>, encounter: EncounterId,
    hp: i32, max_hp: i32) -> Run {
    let mut run = m27_support::complete_run(character, relics, potions, BTreeMap::new(), encounter);
    let state = run.state_mut_for_test().combat.as_mut().unwrap().state_mut_for_test();
    state.player.max_hp = max_hp;
    state.player.hp = hp;
    run
}

fn escape(run: &mut Run) {
    run.choose_combat(CombatAction::UsePotion { slot: 0, target: None }).expect("Smoke Bomb escapes");
    assert!(run.state().combat.is_none(), "the Run owner settles the escape");
    assert_eq!(run.state().last_combat.as_ref().map(|record| record.outcome), Some(Outcome::Escaped));
}

fn win_cultist(run: &mut Run) {
    for slot in 0..3 {
        run.choose_combat(CombatAction::UsePotion { slot, target: Some(TargetId(1)) })
            .unwrap_or_else(|error| panic!("Fire Potion {slot}: {error:?}"));
    }
    assert!(run.state().combat.is_none(), "the Run owner settles the victory");
    assert_eq!(run.state().last_combat.as_ref().map(|record| record.outcome), Some(Outcome::Victory));
}

fn smoke() -> Vec<Option<PotionId>> { vec![Some(PotionId::SmokeBomb), None, None] }

/// D1 minimal case (lane F repros): Ironclad at 70/80 escapes; Burning Blood heals 6.
#[test]
fn d8g_d1_smoke_bomb_escape_runs_burning_blood() {
    let mut run = run_at(Character::Ironclad, vec![RelicId::BurningBlood], smoke(), EncounterId::JawWorm, 70, 80);
    escape(&mut run);
    assert_eq!((run.state().hp, run.state().max_hp), (76, 80));
}

#[test]
fn d8g_d1_smoke_bomb_escape_runs_black_blood() {
    let mut run = run_at(Character::Ironclad, vec![RelicId::BlackBlood], smoke(), EncounterId::JawWorm, 40, 80);
    escape(&mut run);
    assert_eq!((run.state().hp, run.state().max_hp), (52, 80));
}

#[test]
fn d8g_d1_smoke_bomb_escape_runs_meat_on_the_bone() {
    let mut run = run_at(Character::Ironclad, vec![RelicId::MeatOnTheBone], smoke(), EncounterId::JawWorm, 40, 80);
    escape(&mut run);
    assert_eq!((run.state().hp, run.state().max_hp), (52, 80), "40 <= 80/2 heals 12");
    let mut above = run_at(Character::Ironclad, vec![RelicId::MeatOnTheBone], smoke(), EncounterId::JawWorm, 41, 80);
    escape(&mut above);
    assert_eq!(above.state().hp, 41, "above half HP no heal");
}

#[test]
fn d8g_d1_smoke_bomb_escape_runs_face_of_cleric() {
    let mut run = run_at(Character::Ironclad, vec![RelicId::FaceOfCleric], smoke(), EncounterId::JawWorm, 40, 80);
    escape(&mut run);
    assert_eq!((run.state().hp, run.state().max_hp), (41, 81));
}

#[test]
fn d8g_d1_smoke_bomb_escape_runs_self_repair() {
    let mut run = run_at(Character::Defect, vec![], smoke(), EncounterId::JawWorm, 40, 80);
    run.state_mut_for_test().combat.as_mut().unwrap().state_mut_for_test().player_full_powers.push(FullPowerEntry {
        id: FullPowerId("repair"), state: PowerState { amount: 7, skip_first: false, monster_source_grace: false },
    });
    escape(&mut run);
    assert_eq!(run.state().hp, 47);
}

/// D4: Meat on the Bone tests its threshold before the victory heals (Burning Blood, Black Blood,
/// Self Repair). Heals commute, so only the threshold input changes: final = min(before + 12 + 6, max).
#[test]
fn d8g_d4_meat_on_the_bone_threshold_precedes_burning_blood_on_victory() {
    let fire = vec![Some(PotionId::FirePotion); 3];
    let mut run = run_at(Character::Ironclad, vec![RelicId::BurningBlood, RelicId::MeatOnTheBone], fire.clone(),
        EncounterId::Cultist, 40, 80);
    win_cultist(&mut run);
    assert_eq!(run.state().hp, 58, "40 -> Meat +12 -> Burning Blood +6");
    let mut above = run_at(Character::Ironclad, vec![RelicId::BurningBlood, RelicId::MeatOnTheBone], fire,
        EncounterId::Cultist, 41, 80);
    win_cultist(&mut above);
    assert_eq!(above.state().hp, 47, "41 > 40: Burning Blood only");
}

#[test]
fn d8g_d4_meat_on_the_bone_threshold_precedes_burning_blood_on_escape() {
    let mut run = run_at(Character::Ironclad, vec![RelicId::BurningBlood, RelicId::MeatOnTheBone], smoke(),
        EncounterId::JawWorm, 40, 80);
    escape(&mut run);
    assert_eq!(run.state().hp, 58);
}

/// D4: Meat on the Bone runs before every victory callback, so Face of Cleric's +1 Max HP is not
/// part of its threshold even when Face of Cleric was obtained first (41 > 81/2 in the original's
/// float comparison; 41 <= 82/2 would heal).
#[test]
fn d8g_d4_meat_on_the_bone_threshold_precedes_face_of_cleric() {
    let mut run = run_at(Character::Ironclad, vec![RelicId::FaceOfCleric, RelicId::MeatOnTheBone], smoke(),
        EncounterId::JawWorm, 41, 81);
    escape(&mut run);
    assert_eq!((run.state().hp, run.state().max_hp), (42, 82));
}

/// D2 differential: the target-independent base cost equals the cost the payload producers name with a target
/// available (the previous derivation), for every physical card kind at upgrades 0 and 1 (Searing Blow at every
/// level) in each character's live complete combat. Only kinds with no payload price there (unplayable kinds) differ
/// in availability, and they stay unpriced.
#[test]
fn d8g_d2_base_cost_matches_the_targeted_payload_cost() {
    use super::super::cards::{base_cost, probed_cost};
    use crate::content::CardId;
    for character in Character::ALL {
        let run = m27_support::complete_run(character, vec![], vec![None; 3], BTreeMap::new(), EncounterId::JawWorm);
        let state = &run.state().combat.as_ref().unwrap().state;
        assert!(state.enemies.iter().any(|enemy| enemy.targetable()));
        let mut priced = 0;
        for &id in CardId::ALL {
            let levels: Vec<u32> = if id == CardId::SearingBlow { (0..=73).collect() } else { vec![0, 1] };
            for upgrades in levels {
                if !crate::content::complete::supports_generated(id) || !crate::run::complete_upgrades_valid(id, upgrades) { continue }
                let card = super::super::complete_live_card(id, super::super::CardInstanceId(999), upgrades).unwrap();
                let (old, new) = (probed_cost(state, &card), base_cost(state, &card));
                assert_eq!(new.is_some(), old.is_some(), "{character:?} {id:?}+{upgrades}: availability");
                if old.is_some() { assert_eq!(new, old, "{character:?} {id:?}+{upgrades}"); priced += 1; }
            }
        }
        assert!(priced > 700, "{character:?}: {priced} priced faces");
    }
}

/// D2: with no enemy targetable (Awakened One's rebirth window) every Attack in each character's combat generation
/// pool keeps its printed cost; the previous payload derivation lost it for most of them.
#[test]
fn d8g_d2_generation_pool_attacks_keep_their_cost_with_no_targetable_enemy() {
    use super::super::cards::{base_cost, probed_cost};
    use crate::content::pools::{cards, CardPool, GenerationPool};
    let mut lost = Vec::new();
    for character in Character::ALL {
        let mut run = m27_support::complete_run(character, vec![], vec![None; 3], BTreeMap::new(), EncounterId::JawWorm);
        let state = run.state_mut_for_test().combat.as_mut().unwrap().state_mut_for_test();
        for enemy in &mut state.enemies { enemy.hp = 0; enemy.life = super::super::ActorLife::HalfDead; }
        let state = &*state;
        assert!(!state.enemies.iter().any(|enemy| enemy.targetable()));
        for id in cards(CardPool::Combat { character, pool: GenerationPool::Attack }).unwrap() {
            for upgrades in [0, 1] {
                let card = super::super::complete_live_card(id, super::super::CardInstanceId(999), upgrades).unwrap();
                assert!(card.local.x_cost || base_cost(state, &card).is_some(), "{character:?} {id:?}+{upgrades}");
                if upgrades == 0 && !card.local.x_cost && probed_cost(state, &card).is_none() { lost.push(character); }
            }
        }
    }
    // The previous target-validated derivation lost the cost of every single-target Attack without a static entry
    // here (all-enemy Attacks still priced untargeted; Defect's Attacks all have static entries).
    let count = |character| lost.iter().filter(|lost| **lost == character).count();
    assert_eq!(Character::ALL.map(count), [22, 17, 0, 13]);
}

/// D3: a Distilled Chaos burst into Time Warp. The burst's three top-card actions all run before its first card is
/// used. When the 12th card is child `k`, the original's early end retires the remaining queued children through
/// the use-card cleanup without their effects (a Power leaves the piles, an exhaust card exhausts, Strange Spoon may
/// save it), queues +2 Strength per monster, and the turn ends once the queues drain (lane D Q1;
/// `TimeWarpPower.onAfterUseCard`, `GameActionManager.callEndTurnEarlySequence`, `UseCardAction.update`).
/// Draw pile, top first: Strike (1), Inflame (2), Impervious (3), then Defends.
fn time_warp_burst(twelfth: u64, relics: Vec<RelicId>) -> super::super::Combat {
    use super::fuzzfix_b::{arrange, fixture_in, step};
    use crate::content::CardId::*;
    let mut cards = vec![(Strike, 0), (Inflame, 0), (Impervious, 0)];
    cards.extend([(Defend, 0); 8]);
    let mut combat = fixture_in(EncounterId::TimeEater, 3, Character::Ironclad, &cards, relics,
        vec![Some(PotionId::DistilledChaos), None, None], 3);
    arrange(&mut combat, &[], &[4, 5, 6, 7, 8, 9, 10, 11, 3, 2, 1], &[]);
    combat.state.enemies[0].full_powers.iter_mut().find(|power| power.id.0 == "time_warp").unwrap().state.amount = 12 - twelfth as i32;
    step(&mut combat, CombatAction::UsePotion { slot: 0, target: None }, "Distilled Chaos into Time Warp");
    assert_eq!(combat.state.turn, 2, "the 12th card ends the turn once");
    let enemy = &combat.state.enemies[0];
    assert_eq!(enemy.full_powers.iter().find(|power| power.id.0 == "time_warp").unwrap().state.amount, 0);
    assert_eq!(enemy.full_powers.iter().find(|power| power.id.0 == "strength").map_or(0, |power| power.state.amount), 2);
    assert!(combat.state.pending_time_warp.is_none() && combat.state.queued_autoplay.is_empty()
        && combat.state.play_frames.is_empty() && combat.state.in_play.is_none());
    assert_eq!(combat.state.hook_events.iter().filter(|event| event.hook == super::super::Hook::TurnEnded).count(), 1);
    super::super::engine::validate_runtime(&combat.state).expect("valid boundary after the early end");
    combat
}

fn handle(instance: u64) -> super::super::CardHandle { super::super::card_state::handle(super::super::CardInstanceId(instance)) }
fn in_pile(pile: &[super::super::model::OwnedCard], instance: u64) -> bool { pile.iter().any(|owned| owned.handle() == handle(instance)) }
fn strength(combat: &super::super::Combat) -> i32 { super::fuzzfix_b::player_power(combat, "strength") }

#[test]
fn d8g_d3_time_warp_on_the_first_distilled_chaos_card_retires_the_other_two() {
    use super::fuzzfix_b::used_target;
    let combat = time_warp_burst(1, vec![]);
    assert!(used_target(&combat, 1).is_some(), "Strike is the 12th card");
    assert_eq!((used_target(&combat, 2), used_target(&combat, 3)), (None, None), "retired without use");
    assert_eq!(strength(&combat), 0, "Inflame's effect did not run");
    assert!(in_pile(&combat.state.played_powers, 2), "a retired Power leaves the piles as a used Power does");
    let piles = &combat.state.piles;
    assert!(!in_pile(&piles.hand, 2) && !in_pile(&piles.draw, 2) && !in_pile(&piles.discard, 2) && !in_pile(&piles.exhaust, 2));
    assert!(in_pile(&piles.exhaust, 3), "a retired exhaust card exhausts");
}

#[test]
fn d8g_d3_time_warp_on_the_second_distilled_chaos_card_retires_the_third() {
    use super::fuzzfix_b::used_target;
    let combat = time_warp_burst(2, vec![]);
    assert!(used_target(&combat, 1).is_some() && used_target(&combat, 2).is_some());
    assert_eq!(used_target(&combat, 3), None);
    assert_eq!(strength(&combat), 2, "Inflame, the 12th card, was used");
    assert!(in_pile(&combat.state.played_powers, 2) && in_pile(&combat.state.piles.exhaust, 3));
}

#[test]
fn d8g_d3_time_warp_on_the_third_distilled_chaos_card_retires_nothing() {
    use super::fuzzfix_b::used_target;
    let combat = time_warp_burst(3, vec![]);
    assert!((1..=3).all(|instance| used_target(&combat, instance).is_some()));
    assert_eq!(strength(&combat), 2);
    assert!(in_pile(&combat.state.piles.exhaust, 3), "Impervious, the 12th card, exhausts after use");
}

/// D3 cleanup: Strange Spoon rolls for a retired exhaust card, as for a used one (`UseCardAction.update`).
#[test]
fn d8g_d3_retired_exhaust_card_rolls_strange_spoon() {
    let plain = time_warp_burst(1, vec![]);
    let spoon = time_warp_burst(1, vec![RelicId::StrangeSpoon]);
    assert_ne!(spoon.state.card_rng, plain.state.card_rng, "the Spoon roll draws from the card stream");
    let piles = &spoon.state.piles;
    assert!(in_pile(&piles.exhaust, 3) != (in_pile(&piles.discard, 3) || in_pile(&piles.hand, 3) || in_pile(&piles.draw, 3)),
        "Impervious is either exhausted or saved");
}

/// D3, nested: the 12th card is played by a card that Distilled Chaos played (a repeat copy or Havoc's top card), so a
/// card-owned frame sits above the potion's frame. The turn is still potion-owned: the early end retires the potion's
/// other queued cards and the turn ends once the card-owned work unwinds. Counter 10: the potion's first card is the
/// 11th. Draw pile, top first: `top`, then instances 2.. in order; `hand` holds the setup card played first.
fn time_warp_nested(character: Character, setup: crate::content::CardId, top: &[crate::content::CardId]) -> (super::super::Combat, i32) {
    use super::fuzzfix_b::{arrange, fixture_in, step};
    let filler = if character == Character::Silent { crate::content::CardId::DefendGreen } else { crate::content::CardId::Defend };
    let mut cards = vec![(setup, 0)];
    cards.extend(top.iter().map(|card| (*card, 0)));
    cards.extend([(filler, 0); 6]);
    let mut combat = fixture_in(EncounterId::TimeEater, 3, character, &cards, vec![],
        vec![Some(PotionId::DistilledChaos), None, None], 3);
    let mut draw: Vec<u64> = (2..=cards.len() as u64).collect();
    draw.reverse();
    arrange(&mut combat, &[1], &draw, &[]);
    step(&mut combat, CombatAction::PlayCard { instance: 1, target: None }, "setup card");
    combat.state.enemies[0].full_powers.iter_mut().find(|power| power.id.0 == "time_warp").unwrap().state.amount = 10;
    let hp = combat.state.enemies[0].hp;
    step(&mut combat, CombatAction::UsePotion { slot: 0, target: None }, "Distilled Chaos into Time Warp");
    assert_eq!(combat.state.turn, 2, "the 12th card ends the turn once");
    let enemy = &combat.state.enemies[0];
    assert_eq!(enemy.full_powers.iter().find(|power| power.id.0 == "time_warp").unwrap().state.amount, 0);
    assert_eq!(enemy.full_powers.iter().find(|power| power.id.0 == "strength").map_or(0, |power| power.state.amount), 2);
    assert!(combat.state.pending_time_warp.is_none() && combat.state.queued_autoplay.is_empty()
        && combat.state.play_frames.is_empty() && combat.state.in_play.is_none());
    assert_eq!(combat.state.hook_events.iter().filter(|event| event.hook == super::super::Hook::TurnEnded).count(), 1);
    super::super::engine::validate_runtime(&combat.state).expect("valid boundary after the early end");
    let lost = hp - combat.state.enemies[0].hp;
    (combat, lost)
}

/// Cards used this combat, copies included (a repeat copy is a new instance).
fn all_plays(combat: &super::super::Combat) -> usize {
    combat.state.hook_events.iter().filter(|event| event.hook == super::super::Hook::CardPlayed).count()
}

fn card_plays(combat: &super::super::Combat, instance: u64) -> usize {
    let source = super::super::Source::Card(handle(instance));
    combat.state.hook_events.iter().filter(|event| event.hook == super::super::Hook::CardPlayed && event.source == source).count()
}

/// I1 (a): Double Tap's copy of the potion's Strike is the 12th card. It is queued in front of the potion's other cards
/// (`DoubleTapPower.onUseCard`), so the Strike hits twice and both Defends are retired unused.
#[test]
fn d8g_d3_time_warp_on_a_double_tap_copy_of_a_distilled_chaos_card() {
    use crate::content::CardId::*;
    let (combat, lost) = time_warp_nested(Character::Ironclad, DoubleTap, &[Strike, Defend, Defend]);
    assert_eq!(card_plays(&combat, 2), 1, "the potion's Strike, the 11th card");
    assert_eq!((all_plays(&combat), lost), (3, 12), "Double Tap, the Strike and its copy, the 12th card");
    assert_eq!((card_plays(&combat, 3), card_plays(&combat, 4)), (0, 0), "the potion's other cards are retired unused");
    assert!(in_pile(&combat.state.piles.discard, 3) && in_pile(&combat.state.piles.discard, 4));
}

/// I1 (a), Burst: Burst's copy of the potion's Defend is the 12th card, as with Double Tap.
#[test]
fn d8g_d3_time_warp_on_a_burst_copy_of_a_distilled_chaos_card() {
    use crate::content::CardId::*;
    let (combat, lost) = time_warp_nested(Character::Silent, Burst, &[DefendGreen, StrikeGreen, StrikeGreen]);
    assert_eq!(card_plays(&combat, 2), 1, "the potion's Defend, the 11th card");
    assert_eq!((all_plays(&combat), lost), (3, 0), "Burst, the Defend and its copy, the 12th card");
    assert_eq!((card_plays(&combat, 3), card_plays(&combat, 4)), (0, 0), "the potion's Strikes are retired unused");
}

/// I1 (b): Havoc, played by the potion, is the 11th card and the card it plays is the 12th. Exactly one more card is
/// used after Havoc and the other two are retired. Which one is used depends on the order of Havoc's top card
/// against the potion's queued cards: the original appends it behind them, the engine plays it first. That is the
/// ordering difference tracked as spire-d4h, so this test accepts either.
#[test]
fn d8g_d3_time_warp_on_havocs_top_card_under_distilled_chaos() {
    use crate::content::CardId::*;
    let (combat, _) = time_warp_nested(Character::Ironclad, Inflame, &[Havoc, Defend, Defend, Defend]);
    assert_eq!(card_plays(&combat, 2), 1, "Havoc, the 11th card");
    assert_eq!(all_plays(&combat), 3, "Inflame, Havoc and one 12th card");
    assert_eq!((3..=5).map(|instance| card_plays(&combat, instance)).sum::<usize>(), 1, "one 12th card; two retired");
}

/// E2: an enemy dies once. Corpse Explosion's THORNS damage kills a poisoned enemy whose own poison tick is already
/// queued; the original's `die()` guard (`isDying`) and Poison's `currentHealth > 0` check stop a second death, so its
/// death callbacks run once (lane D Q5). The queued completion used to emit a second Death hook; on a Bronze Orb
/// holding a Stasis card that faulted (`Stasis return death/custody`; the env repros `tests::d8g_runs` cover it).
#[test]
fn d8g_e2_a_dying_enemy_receives_no_second_death() {
    use super::super::{Actor, DamagePayload, Effect, EffectKind, Hook, Source};
    use super::fuzzfix_b::{fixture_in, step};
    use crate::content::CardId::*;
    let mut combat = fixture_in(EncounterId::ThreeCultists, 2, Character::Silent, &[(Defend, 0), (Defend, 0)],
        vec![], vec![None; 3], 3);
    let owner = combat.state.enemies[0].instance_id.clone();
    let enemy = Actor::Enemy(owner.clone());
    // As queued by the orb's own start-of-turn poison tick, after a Corpse Explosion hit kills it.
    combat.state.pending.push_back(Effect { source: Source::Actor(Actor::Player), target: Some(enemy.clone()),
        kind: EffectKind::Damage(DamagePayload::Thorns(999)) });
    combat.state.pending.push_back(Effect { source: Source::Actor(enemy.clone()), target: Some(enemy.clone()),
        kind: EffectKind::PoisonTickComplete });
    super::super::engine::resolve(&mut combat.state);
    assert!(combat.state.fault.is_none(), "{:?}", combat.state.fault);
    let deaths = combat.state.hook_events.iter().filter(|event| event.hook == Hook::Death && event.target == Some(enemy.clone())).count();
    assert_eq!(deaths, 1, "one death callback");
    step(&mut combat, CombatAction::EndTurn, "the fight continues");
}

/// D3 flag (a), lane D Q8: a queued autoplay card that fails `canUse` is retired by the use-card cleanup with its use
/// hooks suppressed: a Power goes to the Soul (no discard, exhaust or Strange Spoon roll); a non-Power with an
/// exhaust flag (Havoc's one-use exhaust) rolls Strange Spoon, proc → discard, else exhaust. Havoc is the third card
/// played under Normality, so its top card cannot be used.
fn havoc_suppressed(top: crate::content::CardId, relics: Vec<RelicId>) -> super::super::Combat {
    use super::fuzzfix_b::{arrange, fixture, step};
    use crate::content::CardId::*;
    let mut combat = fixture(Character::Ironclad, &[(Normality, 0), (Defend, 0), (Defend, 0), (Havoc, 0), (top, 0),
        (Strike, 0), (Strike, 0), (Strike, 0)], relics, vec![None; 3], 9);
    arrange(&mut combat, &[1, 2, 3, 4], &[6, 7, 8, 5], &[]);
    step(&mut combat, CombatAction::PlayCard { instance: 2, target: None }, "Defend");
    step(&mut combat, CombatAction::PlayCard { instance: 3, target: None }, "Defend");
    step(&mut combat, CombatAction::PlayCard { instance: 4, target: None }, "Havoc, the third card");
    assert_eq!(super::fuzzfix_b::used_target(&combat, 5), None, "the top card fails canUse under Normality");
    assert!(combat.state.queued_autoplay.is_empty() && combat.state.in_play.is_none());
    combat
}

#[test]
fn d8g_q8_havoc_power_that_cannot_be_used_goes_to_the_soul() {
    for relics in [vec![], vec![RelicId::StrangeSpoon]] {
        let combat = havoc_suppressed(crate::content::CardId::Inflame, relics.clone());
        let piles = &combat.state.piles;
        assert!(in_pile(&combat.state.played_powers, 5), "{relics:?}: Inflame retired to the Soul");
        assert!(!in_pile(&piles.exhaust, 5) && !in_pile(&piles.discard, 5) && !in_pile(&piles.hand, 5) && !in_pile(&piles.draw, 5));
        assert_eq!(strength(&combat), 0, "Inflame's effect did not run");
    }
    let plain = havoc_suppressed(crate::content::CardId::Inflame, vec![]);
    let spoon = havoc_suppressed(crate::content::CardId::Inflame, vec![RelicId::StrangeSpoon]);
    assert_eq!(plain.state.card_rng, spoon.state.card_rng, "no Strange Spoon roll for a Power");
}

#[test]
fn d8g_q8_havoc_status_that_cannot_be_used_exhausts_or_spoon_saves_it() {
    let plain = havoc_suppressed(crate::content::CardId::Wound, vec![]);
    assert!(in_pile(&plain.state.piles.exhaust, 5), "Havoc's one-use exhaust");
    let spoon = havoc_suppressed(crate::content::CardId::Wound, vec![RelicId::StrangeSpoon]);
    assert_ne!(spoon.state.card_rng, plain.state.card_rng, "Strange Spoon rolls");
    assert!(in_pile(&spoon.state.piles.exhaust, 5) != in_pile(&spoon.state.piles.discard, 5), "exhausted or saved to discard");
}

/// E5, lane D Q10: inside the original's monster damage routine, block is removed, then the player's relic modifiers
/// (The Boot raises unblocked 1–4 to 5), then attacker powers, then the target's powers (Invincible caps the damage to
/// its amount, spends that cap and clamps at 0), then HP. The Boot used to apply after the cap, so Invincible was
/// overspent below 0 (validator: "Invincible is Heart-owned and bounded by its A0 allowance").
fn heart_hit(damage: i32, relics: Vec<RelicId>) -> (i32, i32) {
    use super::super::{Actor, DamagePayload, Effect, EffectKind, Source};
    use super::fuzzfix_b::fixture_in;
    use crate::content::CardId::*;
    let mut combat = fixture_in(EncounterId::CorruptHeart, 4, Character::Ironclad, &[(Strike, 0), (Defend, 0)], relics, vec![None; 3], 3);
    let heart = &mut combat.state.enemies[0];
    heart.block = 0;
    heart.full_powers.iter_mut().find(|power| power.id.0 == "invincible").unwrap().state.amount = 2;
    let (owner, hp) = (heart.instance_id.clone(), heart.hp);
    combat.state.pending.push_back(Effect { source: Source::Actor(Actor::Player), target: Some(Actor::Enemy(owner)),
        kind: EffectKind::Damage(DamagePayload::Fixed(damage)) });
    super::super::engine::resolve(&mut combat.state);
    assert!(combat.state.fault.is_none(), "{:?}", combat.state.fault);
    super::super::engine::validate_runtime(&combat.state).expect("Invincible stays within its allowance");
    let heart = &combat.state.enemies[0];
    (hp - heart.hp, heart.full_powers.iter().find(|power| power.id.0 == "invincible").unwrap().state.amount)
}

#[test]
fn d8g_e5_the_boot_raises_damage_before_invincible_caps_it() {
    assert_eq!(heart_hit(3, vec![RelicId::TheBoot]), (2, 0), "Boot 3→5, then Invincible caps 5→2");
    assert_eq!(heart_hit(3, vec![]), (2, 0));
    assert_eq!(heart_hit(46, vec![RelicId::TheBoot]), (2, 0), "Clothesline's 46 into Invincible 2");
    assert_eq!(heart_hit(1, vec![RelicId::TheBoot]), (2, 0), "Boot 1→5, capped to 2");
}

/// Select one card of an open choice and confirm it when the choice stays open.
fn choose(combat: &mut super::super::Combat, instance: u64, context: &str) {
    use super::super::Phase;
    super::fuzzfix_b::step(combat, CombatAction::SelectCard { instance }, context);
    if combat.state.phase == Phase::Selection && combat.state.pending_selection.as_ref().is_some_and(|selection|
        selection.selected.contains(&handle(instance))) {
        super::fuzzfix_b::step(combat, CombatAction::ConfirmSelection, context);
    }
}

/// E7, lane D Q12: a repeater queues its copy in the card queue at use, and the action queue (the original's own
/// selection) drains first. Burst + Survivor: Survivor's discard choice resolves, then the copy plays and opens its own.
#[test]
fn d8g_e7_burst_copy_of_survivor_opens_its_discard_after_the_originals() {
    use super::super::{engine, Phase};
    use super::fuzzfix_b::{arrange, fixture, step};
    use crate::content::CardId::*;
    let mut combat = fixture(Character::Silent, &[(Burst, 0), (Survivor, 0), (StrikeGreen, 0), (StrikeGreen, 0), (DefendGreen, 0),
        (StrikeGreen, 0), (StrikeGreen, 0), (StrikeGreen, 0), (StrikeGreen, 0), (StrikeGreen, 0)], vec![], vec![None; 3], 3);
    arrange(&mut combat, &[1, 2, 3, 4, 5], &[6, 7, 8, 9, 10], &[]);
    step(&mut combat, CombatAction::PlayCard { instance: 1, target: None }, "Burst");
    step(&mut combat, CombatAction::PlayCard { instance: 2, target: None }, "Survivor");
    assert_eq!((combat.state.phase, combat.state.player.block), (Phase::Selection, 8), "Survivor's own choice is open");
    engine::validate_runtime(&combat.state).expect("valid selection boundary");
    choose(&mut combat, 3, "Survivor discards a Strike");
    assert_eq!((combat.state.phase, combat.state.player.block), (Phase::Selection, 16), "then the copy plays and opens its choice");
    engine::validate_runtime(&combat.state).expect("valid selection boundary for the copy");
    choose(&mut combat, 4, "the copy discards a Strike");
    assert_eq!(combat.state.phase, Phase::Decision);
    assert!(in_pile(&combat.state.piles.discard, 3) && in_pile(&combat.state.piles.discard, 4) && in_pile(&combat.state.piles.hand, 5));
    assert!(combat.state.play_frames.is_empty() && combat.state.queued_autoplay.is_empty());
    engine::validate_runtime(&combat.state).unwrap();
}

/// E7: Burst's copy of Havoc is queued while Havoc's top card (Survivor) has its discard choice open. The choice
/// resolves, then the copy plays the next top card. The selection boundary used to fault with
/// `Invariant("Burst repeat child/parent ownership")`: the suspended frame's child is the top card, not the copy.
#[test]
fn d8g_e7_burst_copy_of_havoc_waits_behind_the_top_cards_selection() {
    use super::super::{engine, Phase};
    use super::fuzzfix_b::{arrange, fixture, step};
    use crate::content::CardId::*;
    let mut combat = fixture(Character::Silent, &[(Burst, 0), (Havoc, 0), (StrikeGreen, 0), (StrikeGreen, 0), (DefendGreen, 0),
        (Survivor, 0), (StrikeGreen, 0), (DefendGreen, 0), (DefendGreen, 0)], vec![], vec![None; 3], 3);
    arrange(&mut combat, &[1, 2, 3, 4, 5], &[8, 9, 7, 6], &[]);
    let hp = combat.state.enemies[0].hp;
    step(&mut combat, CombatAction::PlayCard { instance: 1, target: None }, "Burst");
    step(&mut combat, CombatAction::PlayCard { instance: 2, target: None }, "Havoc plays Survivor");
    assert_eq!((combat.state.phase, combat.state.player.block), (Phase::Selection, 8), "Survivor's choice is open");
    engine::validate_runtime(&combat.state).expect("valid selection boundary");
    choose(&mut combat, 3, "Survivor discards a Strike");
    assert_eq!(combat.state.phase, Phase::Decision);
    assert_eq!(hp - combat.state.enemies[0].hp, 6, "the copy of Havoc plays the next top card, a Strike");
    assert_eq!(combat.state.silent.played_this_turn.iter().filter(|id| **id == Havoc).count(), 2);
    assert!(in_pile(&combat.state.piles.discard, 3), "Survivor's discard");
    assert!(in_pile(&combat.state.piles.exhaust, 6) && in_pile(&combat.state.piles.exhaust, 7), "Havoc's played cards exhaust");
    assert!(combat.state.play_frames.is_empty() && combat.state.queued_autoplay.is_empty());
    engine::validate_runtime(&combat.state).unwrap();
}

/// E8, lane D Q13: an autoplayed card keeps its own selection; the queued copy waits for it. Duplication Potion's copy
/// of Havoc is queued while Havoc's top card (Secret Technique) has its draw-pile choice open. The boundary used to
/// fault with `Invariant("Duplication repeat retains its actual card and captured child")`.
#[test]
fn d8g_e8_duplication_copy_of_havoc_waits_behind_secret_techniques_selection() {
    use super::super::{engine, Phase};
    use super::fuzzfix_b::{arrange, fixture, step};
    use crate::content::CardId::*;
    let mut combat = fixture(Character::Ironclad, &[(Havoc, 0), (SecretTechnique, 0), (Defend, 0), (Defend, 0), (Strike, 0), (Strike, 0)],
        vec![], vec![Some(PotionId::DuplicationPotion), None, None], 3);
    arrange(&mut combat, &[1, 6], &[3, 4, 5, 2], &[]);
    let hp = combat.state.enemies[0].hp;
    step(&mut combat, CombatAction::UsePotion { slot: 0, target: None }, "Duplication Potion");
    step(&mut combat, CombatAction::PlayCard { instance: 1, target: None }, "Havoc plays Secret Technique");
    assert_eq!(combat.state.phase, Phase::Selection, "Secret Technique's draw-pile choice is open");
    engine::validate_runtime(&combat.state).expect("valid selection boundary");
    choose(&mut combat, 4, "Secret Technique takes a Defend");
    assert_eq!(combat.state.phase, Phase::Decision);
    assert!(in_pile(&combat.state.piles.hand, 4), "the chosen Defend is in hand");
    assert_eq!(hp - combat.state.enemies[0].hp, 6, "the copy of Havoc plays the next top card, a Strike");
    assert!(in_pile(&combat.state.piles.exhaust, 2) && in_pile(&combat.state.piles.exhaust, 5), "Havoc's played cards exhaust");
    assert!(combat.state.play_frames.is_empty() && combat.state.queued_autoplay.is_empty());
    engine::validate_runtime(&combat.state).unwrap();
}

/// E9, lane D Q14: Nightmare's delivery and Foresight's Scry both run in the pre-draw power pass, in power-list order,
/// before the turn draw. Returns the hand size when the Scry opens and the turn-2 hand once it resolves.
fn nightmare_and_foresight(foresight_first: bool) -> (usize, Vec<crate::content::CardId>) {
    use super::super::{engine, Phase};
    use super::fuzzfix_b::{arrange, fixture, step};
    use crate::content::CardId::*;
    let mut combat = fixture(Character::Silent, &[(Foresight, 0), (Nightmare, 0), (DefendGreen, 0), (StrikeGreen, 0), (StrikeGreen, 0),
        (StrikeGreen, 0), (StrikeGreen, 0), (StrikeGreen, 0), (StrikeGreen, 0)], vec![], vec![None; 3], 10);
    arrange(&mut combat, &[1, 2, 3], &[4, 5, 6, 7, 8, 9], &[]);
    for instance in if foresight_first { [1, 2] } else { [2, 1] } {
        step(&mut combat, CombatAction::PlayCard { instance, target: None }, "power");
        if instance == 2 { choose(&mut combat, 3, "Nightmare copies Defend"); }
    }
    step(&mut combat, CombatAction::EndTurn, "end turn 1");
    assert_eq!((combat.state.turn, combat.state.phase), (2, Phase::Selection), "Foresight's start-of-turn Scry is open");
    engine::validate_runtime(&combat.state).expect("valid start-of-turn Scry boundary");
    let at_scry = combat.state.piles.hand.len();
    step(&mut combat, CombatAction::ConfirmSelection, "Scry keeps every card");
    assert_eq!(combat.state.phase, Phase::Decision);
    engine::validate_runtime(&combat.state).unwrap();
    (at_scry, combat.state.piles.hand.iter().map(|card| card.full().unwrap().id).collect())
}

#[test]
fn d8g_e9_nightmare_and_foresight_run_in_power_list_order_before_the_draw() {
    use crate::content::CardId::*;
    let (at_scry, hand) = nightmare_and_foresight(true);
    assert_eq!(at_scry, 0, "Foresight first: the Scry opens before the copies arrive");
    assert_eq!(hand[..3], [DefendGreen; 3], "{hand:?}");
    assert_eq!(hand.len(), 8, "three copies, then the five-card draw");
    let (at_scry, hand) = nightmare_and_foresight(false);
    assert_eq!(at_scry, 3, "Nightmare first: the copies arrive before the Scry");
    assert_eq!(hand[..3], [DefendGreen; 3], "{hand:?}");
    assert_eq!(hand.len(), 8);
}

/// E10, lane D Q15: the original's end of turn moves retained cards to limbo and queues their restore at the top, ahead
/// of the exhaust listeners' bottom-queued children (Dead Branch's card, Dark Embrace's draw), so those children see the
/// restored card and respect the 10-card hand (overflow to discard; the draw stops). The children used to run first
/// and the restore pushed the hand to 11 (`Invariant("hand exceeds cap")` at the next draw).
#[test]
fn d8g_e10_retained_card_is_restored_before_end_of_turn_exhaust_children() {
    use super::super::{engine, Phase};
    use super::fuzzfix_b::{arrange, fixture, step};
    use crate::content::CardId::*;
    let mut cards = vec![(Perseverance, 0), (Carnage, 0), (Carnage, 0), (Strike, 0)];
    cards.extend([(Defend, 0); 10]);
    let mut combat = fixture(Character::Ironclad, &cards, vec![RelicId::DeadBranch], vec![None; 3], 3);
    arrange(&mut combat, &[1, 2, 3, 4], &[5, 6, 7, 8, 9, 10, 11, 12, 13, 14], &[]);
    combat.state.player_full_powers.push(FullPowerEntry { id: FullPowerId("dark_embrace"),
        state: PowerState { amount: 9, skip_first: false, monster_source_grace: false } });
    step(&mut combat, CombatAction::EndTurn, "end turn 1");
    assert_eq!((combat.state.turn, combat.state.phase), (2, Phase::Decision));
    let piles = &combat.state.piles;
    assert_eq!(piles.hand.len(), 10, "the hand never exceeds 10");
    assert_eq!(piles.hand[0].handle(), handle(1), "Perseverance is restored first");
    assert_eq!(piles.draw.len(), 2, "the first Dark Embrace draws 8, the second and the turn draw find a full hand");
    assert!(in_pile(&piles.exhaust, 2) && in_pile(&piles.exhaust, 3) && in_pile(&piles.discard, 4));
    assert_eq!(piles.discard.len(), 2, "the second Dead Branch card overflows to discard");
    engine::validate_runtime(&combat.state).unwrap();
}

/// D2 (verifier I1, PM-accepted): a card's cost is a static card field, so cost-reading effects see it with no enemy
/// targetable, as in the original. Ironclad under Confusion vs Awakened One: Cleave kills both Cultists and downs form
/// 1 (rebirth window, nothing targetable); Shrug It Off then draws Body Slam, whose cost Confusion rolls on the card RNG.
/// With the old target-validated cost the roll was skipped (no randomized cost, card RNG not advanced).
#[test]
fn d8g_d2_confusion_rolls_a_drawn_cost_with_no_targetable_enemy() {
    use super::fuzzfix_b::{arrange, fixture_in, step};
    use crate::content::CardId::*;
    let mut combat = fixture_in(EncounterId::AwakenedOne, 3, Character::Ironclad,
        &[(Cleave, 0), (ShrugItOff, 0), (BodySlam, 0), (Defend, 0), (Defend, 0)], vec![], vec![None; 3], 3);
    arrange(&mut combat, &[1, 2], &[4, 5, 3], &[]);
    for enemy in &mut combat.state.enemies { enemy.hp = 1; enemy.block = 0; }
    combat.state.player_full_powers.push(FullPowerEntry { id: FullPowerId("confused"),
        state: PowerState { amount: 1, skip_first: false, monster_source_grace: false } });
    step(&mut combat, CombatAction::PlayCard { instance: 1, target: None }, "Cleave");
    assert!(combat.state.enemies.iter().all(|enemy| !enemy.targetable()), "rebirth window: nothing targetable");
    let rng = combat.state.card_rng.clone();
    step(&mut combat, CombatAction::PlayCard { instance: 2, target: None }, "Shrug It Off draws Body Slam");
    let body_slam = combat.state.piles.hand.iter().find(|card| card.handle() == handle(3)).expect("Body Slam drawn");
    assert!(body_slam.full().unwrap().local.randomized_cost.is_some(), "Confusion rolled Body Slam's cost");
    assert_ne!(combat.state.card_rng, rng, "the roll used the card RNG");
}

/// E2 (verifier M1), the Death guard alone: a Death callback for an enemy whose death already ran does nothing (the
/// original's `die()` returns at once for a dying monster). The second Death here does not come from a poison tick.
#[test]
fn d8g_e2_a_second_death_callback_for_a_dead_enemy_does_nothing() {
    use super::super::{Actor, DamagePayload, Effect, EffectKind, Hook, Source};
    use super::fuzzfix_b::fixture_in;
    use crate::content::CardId::*;
    let mut combat = fixture_in(EncounterId::ThreeCultists, 2, Character::Silent, &[(Defend, 0), (Defend, 0)],
        vec![], vec![None; 3], 3);
    let enemy = Actor::Enemy(combat.state.enemies[0].instance_id.clone());
    combat.state.pending.push_back(Effect { source: Source::Actor(Actor::Player), target: Some(enemy.clone()),
        kind: EffectKind::Damage(DamagePayload::Thorns(999)) });
    combat.state.pending.push_back(Effect { source: Source::Actor(Actor::Player), target: Some(enemy.clone()),
        kind: EffectKind::Hook(Hook::Death) });
    super::super::engine::resolve(&mut combat.state);
    assert!(combat.state.fault.is_none(), "{:?}", combat.state.fault);
    let deaths = combat.state.hook_events.iter().filter(|event| event.hook == Hook::Death && event.target == Some(enemy.clone())).count();
    assert_eq!(deaths, 1, "one death callback");
}

/// E2 (verifier M1), the poison guard alone: a poison tick whose owner another hit already killed (its Death callback is
/// queued) causes no death and no settlement of its own (the original's Poison HP loss checks `currentHealth > 0`). The
/// completion is applied directly so the queued Death and the Death guard play no part.
#[test]
fn d8g_e2_a_poison_tick_on_an_enemy_already_dying_queues_no_death_or_settlement() {
    use super::super::{Actor, Effect, EffectKind, Hook, Source};
    use super::fuzzfix_b::fixture;
    use crate::content::CardId::*;
    let mut combat = fixture(Character::Silent, &[(Defend, 0), (Defend, 0)], vec![], vec![None; 3], 3);
    let enemy = Actor::Enemy(combat.state.enemies[0].instance_id.clone());
    combat.state.enemies[0].hp = 0;
    let death = Effect { source: Source::Actor(Actor::Player), target: Some(enemy.clone()), kind: EffectKind::Hook(Hook::Death) };
    combat.state.pending.clear();
    combat.state.pending.push_back(death.clone());
    super::super::effects::apply_effect(&mut combat.state, &Effect { source: Source::Actor(enemy.clone()), target: Some(enemy.clone()),
        kind: EffectKind::PoisonTickComplete }).unwrap();
    assert_eq!(combat.state.pending.iter().cloned().collect::<Vec<_>>(), vec![death], "only the killing hit's Death is queued");
}
