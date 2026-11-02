//! M27 plan Task P1, lane P, key m27_potions.
//! One named complete-item potion case per union (potion, character) pair.

use crate::{
    combat::{self, Actor, CharacterEffect, CombatAction, DamagePayload, Effect, EffectKind,
        PotionEffect, Source, TargetId},
    content::{CardId, Character, EncounterId, PotionId, RelicId},
    items::{self, tests::m27_support},
    run::Run,
};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TargetRule {
    None,
    Enemy,
    Never,
}

fn complete_run(character: Character, potion: PotionId, relics: Vec<RelicId>) -> Run {
    m27_support::complete_run(character, relics, vec![Some(potion), None, None],
        BTreeMap::new(), EncounterId::JawWorm)
}

fn live_target(run: &Run) -> combat::EnemyHandle {
    run.state().combat.as_ref().unwrap().public_full().unwrap().enemies[0].instance_id.clone()
}

fn full_power(effects: &[Effect], id: &'static str, amount: i32) {
    assert!(effects.iter().any(|effect| matches!(&effect.kind,
        EffectKind::ApplyFullPower { id: actual, amount: actual_amount }
            if actual.0 == id && *actual_amount == amount)),
        "missing full power {id}={amount} in {effects:?}");
}

fn assert_prepared_value(potion: PotionId, effects: &[Effect], value: i32) {
    let amount = |predicate: &dyn Fn(&EffectKind) -> bool| {
        assert!(effects.iter().any(|effect| predicate(&effect.kind)),
            "{potion:?} missing expected output {value}: {effects:?}");
    };
    match potion {
        PotionId::FirePotion => amount(&|kind| matches!(kind, EffectKind::Damage(DamagePayload::Thorns(n)) if *n == value)),
        PotionId::BloodPotion => amount(&|kind| matches!(kind, EffectKind::Heal(n) if *n == value * 80 / 100)),
        PotionId::FruitJuice => amount(&|kind| matches!(kind, EffectKind::IncreaseMaxHpThenHeal(n) if *n == value)),
        PotionId::BlockPotion => amount(&|kind| matches!(kind, EffectKind::GainBlock(n) if *n == value)),
        PotionId::EnergyPotion => amount(&|kind| matches!(kind, EffectKind::GainEnergy(n) if *n == value)
            || matches!(kind, EffectKind::Watcher(combat::cards::watcher::WatcherEffect::PotionEnergy { amount, .. })
                if *amount == value)),
        PotionId::StrengthPotion => full_power(effects, "strength", value),
        PotionId::SwiftPotion => amount(&|kind| matches!(kind, EffectKind::Draw(n) if *n == value)),
        PotionId::WeakPotion => full_power(effects, "weak", value),
        PotionId::FearPotion => full_power(effects, "vulnerable", value),
        PotionId::DexterityPotion => full_power(effects, "dexterity", value),
        PotionId::AncientPotion => full_power(effects, "artifact", value),
        PotionId::Elixir => amount(&|kind| matches!(kind, EffectKind::SelectHandCardsAfterDraw { required: 0,
            disposition: combat::SelectionDisposition::Exhaust, .. })),
        PotionId::HeartOfIron => full_power(effects, "metallicize", value),
        PotionId::PoisonPotion => full_power(effects, "poison", value),
        PotionId::CunningPotion => {
            assert_eq!(effects.len(), value as usize, "Cunning Potion creates the specified number of Shivs");
            assert!(effects.iter().all(|effect| matches!(effect.kind,
                EffectKind::CreateCard { id: CardId::Shiv, upgrades: 1, .. })));
        }
        PotionId::GhostInAJar => full_power(effects, "intangible_player", value),
        PotionId::SmokeBomb => amount(&|kind| matches!(kind, EffectKind::SmokeEscape)),
        PotionId::AttackPotion | PotionId::SkillPotion | PotionId::PowerPotion | PotionId::ColorlessPotion => {
            assert!(effects.iter().any(|effect| matches!(&effect.kind, EffectKind::Generate(work)
                if work.count == 3 && work.offer == Some((potion != PotionId::ColorlessPotion, value as u8)))),
                "three distinct offers retain the specified copy count: {effects:?}");
        }
        PotionId::FlexPotion => {
            full_power(effects, "strength", value);
            full_power(effects, "lose_strength", value);
        }
        PotionId::SpeedPotion => {
            full_power(effects, "dexterity", value);
            full_power(effects, "lose_dexterity", value);
        }
        PotionId::RegenPotion => full_power(effects, "regeneration", value),
        PotionId::LiquidBronze => full_power(effects, "thorns", value),
        PotionId::EssenceOfSteel => full_power(effects, "plated_armor", value),
        PotionId::CultistPotion => full_power(effects, "ritual", value),
        PotionId::BlessingOfTheForge => amount(&|kind| matches!(kind,
            EffectKind::Potion(PotionEffect::UpgradeHand))),
        PotionId::SneckoOil => {
            amount(&|kind| matches!(kind, EffectKind::Draw(n) if *n == value));
            amount(&|kind| matches!(kind, EffectKind::Potion(PotionEffect::RandomizeHandCosts)));
        }
        PotionId::FairyPotion => unreachable!("Fairy is checked as never manually usable"),
        PotionId::ExplosivePotion => {
            assert_eq!(effects.len(), 1, "Jaw Worm supplies one live target");
            amount(&|kind| matches!(kind, EffectKind::Damage(DamagePayload::Fixed(n)) if *n == value));
        }
        PotionId::GamblersBrew => amount(&|kind| matches!(kind,
            EffectKind::Potion(PotionEffect::GamblersBrewSelectDiscard))),
        PotionId::LiquidMemories => amount(&|kind| matches!(kind,
            EffectKind::Potion(PotionEffect::LiquidMemories { requested }) if *requested == value as u8)),
        PotionId::DuplicationPotion => full_power(effects, "duplication", value),
        PotionId::DistilledChaos => {
            assert_eq!(effects.len(), value as usize);
            assert!(effects.iter().all(|effect| matches!(effect.kind,
                EffectKind::Potion(PotionEffect::DistilledChaosTopCard { .. }))));
        }
        PotionId::EntropicBrew => amount(&|kind| matches!(kind,
            EffectKind::Potion(PotionEffect::EntropicBrew { count: 3, .. }))),
        PotionId::FocusPotion => amount(&|kind| matches!(kind, EffectKind::Character(CharacterEffect::Focus(n)) if *n == value)),
        PotionId::PotionOfCapacity => amount(&|kind| matches!(kind, EffectKind::Character(CharacterEffect::Slots(n)) if *n == value)),
        PotionId::EssenceOfDarkness => amount(&|kind| matches!(kind,
            EffectKind::Defect(combat::cards::DefectEffect::EssenceOfDarkness { potency }) if *potency == value)),
        PotionId::StancePotion => amount(&|kind| matches!(kind,
            EffectKind::Watcher(combat::cards::watcher::WatcherEffect::OpenStanceChoice))),
        PotionId::Ambrosia => amount(&|kind| matches!(kind,
            EffectKind::Character(CharacterEffect::Stance(combat::Stance::Divinity)))),
        PotionId::BottledMiracle => amount(&|kind| matches!(kind,
            EffectKind::Watcher(combat::cards::watcher::WatcherEffect::CreateRequest { id: CardId::Miracle, count })
                if *count == value as u32)),
        PotionId::Unknown(_) => unreachable!("the complete standard potion union is finite"),
    }
}

fn prepare_for_rule(run: &Run, potion: PotionId, rule: TargetRule, target: &combat::EnemyHandle,
    bark_value: i32) {
    let combat = run.state().combat.as_ref().expect("complete live combat");
    assert_eq!(combat.item_context().unwrap().potion_potency(0), Some(bark_value));
    let result = match rule {
        TargetRule::Enemy => {
            assert!(items::prepare_combat_potion(combat, 0, None, false).is_err(),
                "{potion:?} requires an enemy target");
            let prepared = items::prepare_combat_potion(combat, 0, Some(target.clone()), false)
                .unwrap_or_else(|error| panic!("{potion:?} rejects a live enemy: {error:?}"));
            assert!(items::prepare_combat_potion(combat, 0,
                Some(combat::EnemyHandle::parse("potion-grid-not-live").unwrap()), false).is_err(),
                "{potion:?} rejects a target that is not a living enemy");
            Some(prepared)
        }
        TargetRule::None => {
            assert!(items::prepare_combat_potion(combat, 0, Some(target.clone()), false).is_err(),
                "{potion:?} does not accept an enemy target");
            Some(items::prepare_combat_potion(combat, 0, None, false)
                .unwrap_or_else(|error| panic!("{potion:?} accepts a target-free use: {error:?}")))
        }
        TargetRule::Never => {
            assert!(items::prepare_combat_potion(combat, 0, None, false).is_err(),
                "Fairy in a Bottle is never manually usable");
            assert!(items::prepare_combat_potion(combat, 0, Some(target.clone()), false).is_err());
            None
        }
    };
    let chosen_target = if rule == TargetRule::Enemy { Some(target.clone()) } else { None };
    assert!(items::prepare_combat_potion(combat, 0, chosen_target, true).is_err(),
        "{potion:?} is rejected during We Meet Again");
    if let Some(prepared) = result {
        assert_eq!(prepared.potions[0], None, "manual use consumes this potion slot");
        assert_prepared_value(potion, &prepared.effects, bark_value);
    }
}

fn finish_potion_selection(run: &mut Run) {
    for _ in 0..64 {
        let Some(combat) = run.state().combat.as_ref() else { return; };
        if combat.state.phase != combat::Phase::Selection { return; }
        let action = combat.legal_actions_full().unwrap().into_iter().next()
            .expect("potion selection publishes a legal continuation");
        run.choose_combat(action).unwrap_or_else(|error| panic!("resolve potion selection: {error:?}"));
    }
    panic!("potion selection did not resolve within 64 legal choices");
}

fn potion_case(character: Character, potion: PotionId, base_value: i32, bark_value: i32,
    target_rule: TargetRule) {
    let mut base = complete_run(character, potion, vec![]);
    if potion == PotionId::EnergyPotion {
        base.state_mut_for_test().combat.as_mut().unwrap().state.player.energy = 0;
    }
    let target = live_target(&base);
    let base_combat = base.state().combat.as_ref().unwrap();
    assert_eq!(base_combat.item_context().unwrap().potion_potency(0), Some(base_value),
        "{potion:?} base potency for {character:?} is Appendix B's literal");
    let base_result = match target_rule {
        TargetRule::Enemy => Some(items::prepare_combat_potion(base_combat, 0, Some(target.clone()), false)
            .unwrap_or_else(|error| panic!("base {potion:?} {character:?}: {error:?}"))),
        TargetRule::None => Some(items::prepare_combat_potion(base_combat, 0, None, false)
            .unwrap_or_else(|error| panic!("base {potion:?} {character:?}: {error:?}"))),
        TargetRule::Never => {
            assert!(items::prepare_combat_potion(base_combat, 0, None, false).is_err());
            // Fairy has no manual output or Toy Ornithopter callback.
            assert!(items::prepare_combat_potion(base_combat, 0, None, true).is_err());
            None
        }
    };
    if let Some(prepared) = base_result {
        assert_eq!(prepared.potions[0], None);
        assert_prepared_value(potion, &prepared.effects, base_value);
    }
    prepare_for_rule(&base, potion, target_rule, &target, base_value);

    // End the complete combat without consuming its potion; every manual use is refused after
    // the turn/combat has ended, even when that potion normally also works outside combat.
    let ended_target = target.clone();
    let combat = base.state().combat.as_ref().unwrap();
    let mut ended = combat.clone();
    ended.enqueue_native_effects(vec![Effect {
        source: Source::Actor(Actor::Player), target: Some(Actor::Enemy(ended_target.clone())),
        kind: EffectKind::Damage(DamagePayload::Fixed(10_000)),
    }], None).expect("finish the test combat through the resolver");
    assert_eq!(ended.public_full().unwrap().outcome, Some(combat::Outcome::Victory));
    assert!(items::prepare_combat_potion(&ended, 0,
        (target_rule == TargetRule::Enemy).then_some(ended_target), false).is_err(),
        "{potion:?} cannot be used once the combat turn has ended");

    if target_rule == TargetRule::Never { return; }
    let mut bark = complete_run(character, potion, vec![RelicId::SacredBark, RelicId::ToyOrnithopter]);
    let bark_target = live_target(&bark);
    prepare_for_rule(&bark, potion, target_rule, &bark_target, bark_value);
    let combat = bark.state_mut_for_test().combat.as_mut().unwrap();
    combat::tests::m27_state::set_player_hp(combat, 30);
    if potion == PotionId::EnergyPotion {
        combat.state.player.energy = 0;
    }
    let before = combat.public_full().unwrap().player.hp;
    let target_action = (target_rule == TargetRule::Enemy).then_some(TargetId(1));
    bark.choose_combat(CombatAction::UsePotion { slot: 0, target: target_action })
        .unwrap_or_else(|error| panic!("Bark potion use {potion:?} {character:?}: {error:?}"));
    finish_potion_selection(&mut bark);
    let after_hp = bark.state().combat.as_ref().map(|combat| combat.public_full().unwrap().player.hp)
        .unwrap_or(bark.state().hp as i32);
    let expected_delta = match potion {
        PotionId::BloodPotion => 37,
        PotionId::FruitJuice => 15,
        _ => 5,
    };
    assert_eq!(before, 30);
    assert_eq!(after_hp - before, expected_delta,
        "Toy Ornithopter contributes exactly +5 after manual {potion:?}; Bark does not double it");
    if let Some(combat) = bark.state().combat.as_ref() {
        if potion == PotionId::EntropicBrew {
            let items = combat.item_context().unwrap();
            assert_eq!(items.potions().iter().flatten().count(), 3,
                "combat Entropic Brew obtains one limited proposal per live belt slot");
            let pool = items::potion_generation::candidates(character, true).unwrap();
            assert!(items.potions().iter().flatten().all(|id| pool.contains(id)));
        } else {
            assert_eq!(combat.item_context().unwrap().potions()[0], None, "manual use empties its held slot");
        }
    } else {
        assert_eq!(bark.state().potions[0], None, "Smoke Bomb returns its consumed slot after escape");
    }
    if potion == PotionId::SmokeBomb {
        assert_eq!(bark.state().last_combat.as_ref().map(|record| record.outcome),
            Some(combat::Outcome::Escaped));
    }
}

macro_rules! case {
    ($name:ident, $character:ident, $potion:ident, $base:literal, $bark:literal, $target:ident) => {
        #[test]
        fn $name() {
            potion_case(Character::$character, PotionId::$potion, $base, $bark, TargetRule::$target);
        }
    };
}

macro_rules! shared_cases {
    ($potion:ident, $base:literal, $bark:literal, $target:ident;
        $ic:ident, $si:ident, $de:ident, $wa:ident) => {
        case!($ic, Ironclad, $potion, $base, $bark, $target);
        case!($si, Silent, $potion, $base, $bark, $target);
        case!($de, Defect, $potion, $base, $bark, $target);
        case!($wa, Watcher, $potion, $base, $bark, $target);
    };
}

shared_cases!(FirePotion, 20, 40, Enemy;
    m27_potion_fire_potion_ironclad, m27_potion_fire_potion_silent,
    m27_potion_fire_potion_defect, m27_potion_fire_potion_watcher);
case!(m27_potion_blood_potion_ironclad, Ironclad, BloodPotion, 20, 40, None);
shared_cases!(FruitJuice, 5, 10, None;
    m27_potion_fruit_juice_ironclad, m27_potion_fruit_juice_silent,
    m27_potion_fruit_juice_defect, m27_potion_fruit_juice_watcher);
shared_cases!(BlockPotion, 12, 24, None;
    m27_potion_block_potion_ironclad, m27_potion_block_potion_silent,
    m27_potion_block_potion_defect, m27_potion_block_potion_watcher);
shared_cases!(EnergyPotion, 2, 4, None;
    m27_potion_energy_potion_ironclad, m27_potion_energy_potion_silent,
    m27_potion_energy_potion_defect, m27_potion_energy_potion_watcher);
shared_cases!(StrengthPotion, 2, 4, None;
    m27_potion_strength_potion_ironclad, m27_potion_strength_potion_silent,
    m27_potion_strength_potion_defect, m27_potion_strength_potion_watcher);
shared_cases!(SwiftPotion, 3, 6, None;
    m27_potion_swift_potion_ironclad, m27_potion_swift_potion_silent,
    m27_potion_swift_potion_defect, m27_potion_swift_potion_watcher);
shared_cases!(WeakPotion, 3, 6, Enemy;
    m27_potion_weak_potion_ironclad, m27_potion_weak_potion_silent,
    m27_potion_weak_potion_defect, m27_potion_weak_potion_watcher);
shared_cases!(FearPotion, 3, 6, Enemy;
    m27_potion_fear_potion_ironclad, m27_potion_fear_potion_silent,
    m27_potion_fear_potion_defect, m27_potion_fear_potion_watcher);
shared_cases!(DexterityPotion, 2, 4, None;
    m27_potion_dexterity_potion_ironclad, m27_potion_dexterity_potion_silent,
    m27_potion_dexterity_potion_defect, m27_potion_dexterity_potion_watcher);
shared_cases!(AncientPotion, 1, 2, None;
    m27_potion_ancient_potion_ironclad, m27_potion_ancient_potion_silent,
    m27_potion_ancient_potion_defect, m27_potion_ancient_potion_watcher);
case!(m27_potion_elixir_ironclad, Ironclad, Elixir, 0, 0, None);
case!(m27_potion_heart_of_iron_ironclad, Ironclad, HeartOfIron, 6, 12, None);
case!(m27_potion_poison_potion_silent, Silent, PoisonPotion, 6, 12, Enemy);
case!(m27_potion_cunning_potion_silent, Silent, CunningPotion, 3, 6, None);
case!(m27_potion_ghost_in_a_jar_silent, Silent, GhostInAJar, 1, 2, None);
shared_cases!(SmokeBomb, 0, 0, None;
    m27_potion_smoke_bomb_ironclad, m27_potion_smoke_bomb_silent,
    m27_potion_smoke_bomb_defect, m27_potion_smoke_bomb_watcher);
shared_cases!(AttackPotion, 1, 2, None;
    m27_potion_attack_potion_ironclad, m27_potion_attack_potion_silent,
    m27_potion_attack_potion_defect, m27_potion_attack_potion_watcher);
shared_cases!(SkillPotion, 1, 2, None;
    m27_potion_skill_potion_ironclad, m27_potion_skill_potion_silent,
    m27_potion_skill_potion_defect, m27_potion_skill_potion_watcher);
shared_cases!(PowerPotion, 1, 2, None;
    m27_potion_power_potion_ironclad, m27_potion_power_potion_silent,
    m27_potion_power_potion_defect, m27_potion_power_potion_watcher);
shared_cases!(ColorlessPotion, 1, 2, None;
    m27_potion_colorless_potion_ironclad, m27_potion_colorless_potion_silent,
    m27_potion_colorless_potion_defect, m27_potion_colorless_potion_watcher);
shared_cases!(FlexPotion, 5, 10, None;
    m27_potion_flex_potion_ironclad, m27_potion_flex_potion_silent,
    m27_potion_flex_potion_defect, m27_potion_flex_potion_watcher);
shared_cases!(SpeedPotion, 5, 10, None;
    m27_potion_speed_potion_ironclad, m27_potion_speed_potion_silent,
    m27_potion_speed_potion_defect, m27_potion_speed_potion_watcher);
shared_cases!(RegenPotion, 5, 10, None;
    m27_potion_regen_potion_ironclad, m27_potion_regen_potion_silent,
    m27_potion_regen_potion_defect, m27_potion_regen_potion_watcher);
shared_cases!(LiquidBronze, 3, 6, None;
    m27_potion_liquid_bronze_ironclad, m27_potion_liquid_bronze_silent,
    m27_potion_liquid_bronze_defect, m27_potion_liquid_bronze_watcher);
shared_cases!(EssenceOfSteel, 4, 8, None;
    m27_potion_essence_of_steel_ironclad, m27_potion_essence_of_steel_silent,
    m27_potion_essence_of_steel_defect, m27_potion_essence_of_steel_watcher);
shared_cases!(CultistPotion, 1, 2, None;
    m27_potion_cultist_potion_ironclad, m27_potion_cultist_potion_silent,
    m27_potion_cultist_potion_defect, m27_potion_cultist_potion_watcher);
shared_cases!(BlessingOfTheForge, 0, 0, None;
    m27_potion_blessing_of_the_forge_ironclad, m27_potion_blessing_of_the_forge_silent,
    m27_potion_blessing_of_the_forge_defect, m27_potion_blessing_of_the_forge_watcher);
shared_cases!(SneckoOil, 5, 10, None;
    m27_potion_snecko_oil_ironclad, m27_potion_snecko_oil_silent,
    m27_potion_snecko_oil_defect, m27_potion_snecko_oil_watcher);
shared_cases!(FairyPotion, 30, 60, Never;
    m27_potion_fairy_potion_ironclad, m27_potion_fairy_potion_silent,
    m27_potion_fairy_potion_defect, m27_potion_fairy_potion_watcher);
shared_cases!(ExplosivePotion, 10, 20, None;
    m27_potion_explosive_potion_ironclad, m27_potion_explosive_potion_silent,
    m27_potion_explosive_potion_defect, m27_potion_explosive_potion_watcher);
shared_cases!(GamblersBrew, 0, 0, None;
    m27_potion_gamblers_brew_ironclad, m27_potion_gamblers_brew_silent,
    m27_potion_gamblers_brew_defect, m27_potion_gamblers_brew_watcher);
shared_cases!(LiquidMemories, 1, 2, None;
    m27_potion_liquid_memories_ironclad, m27_potion_liquid_memories_silent,
    m27_potion_liquid_memories_defect, m27_potion_liquid_memories_watcher);
shared_cases!(DuplicationPotion, 1, 2, None;
    m27_potion_duplication_potion_ironclad, m27_potion_duplication_potion_silent,
    m27_potion_duplication_potion_defect, m27_potion_duplication_potion_watcher);
shared_cases!(DistilledChaos, 3, 6, None;
    m27_potion_distilled_chaos_ironclad, m27_potion_distilled_chaos_silent,
    m27_potion_distilled_chaos_defect, m27_potion_distilled_chaos_watcher);
shared_cases!(EntropicBrew, 0, 0, None;
    m27_potion_entropic_brew_ironclad, m27_potion_entropic_brew_silent,
    m27_potion_entropic_brew_defect, m27_potion_entropic_brew_watcher);
case!(m27_potion_focus_potion_defect, Defect, FocusPotion, 2, 4, None);
case!(m27_potion_potion_of_capacity_defect, Defect, PotionOfCapacity, 2, 4, None);
case!(m27_potion_essence_of_darkness_defect, Defect, EssenceOfDarkness, 1, 2, None);
case!(m27_potion_stance_potion_watcher, Watcher, StancePotion, 0, 0, None);
case!(m27_potion_ambrosia_watcher, Watcher, Ambrosia, 2, 4, None);
case!(m27_potion_bottled_miracle_watcher, Watcher, BottledMiracle, 2, 4, None);
