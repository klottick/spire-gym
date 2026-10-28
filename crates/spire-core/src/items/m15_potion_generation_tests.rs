use crate::{
    combat::InputError,
    content::{Catalog, Character, PotionId, PotionRarity},
    items::potion_generation,
};
use std::collections::HashSet;

#[test]
fn m15_potion_factory_exposes_the_complete_silent_roster_and_limited_exclusion() {
    let expected = [
        PotionId::PoisonPotion, PotionId::CunningPotion, PotionId::GhostInAJar,
        PotionId::BlockPotion, PotionId::DexterityPotion, PotionId::EnergyPotion,
        PotionId::ExplosivePotion, PotionId::FirePotion, PotionId::StrengthPotion,
        PotionId::SwiftPotion, PotionId::WeakPotion, PotionId::FearPotion,
        PotionId::AttackPotion, PotionId::SkillPotion, PotionId::PowerPotion,
        PotionId::ColorlessPotion, PotionId::FlexPotion, PotionId::SpeedPotion,
        PotionId::BlessingOfTheForge, PotionId::RegenPotion, PotionId::AncientPotion,
        PotionId::LiquidBronze, PotionId::GamblersBrew, PotionId::EssenceOfSteel,
        PotionId::DuplicationPotion, PotionId::DistilledChaos, PotionId::LiquidMemories,
        PotionId::CultistPotion, PotionId::FruitJuice, PotionId::SneckoOil,
        PotionId::FairyPotion, PotionId::SmokeBomb, PotionId::EntropicBrew,
    ];
    assert_eq!(expected.iter().copied().collect::<HashSet<_>>().len(), 33,
        "the actual Silent potion identities are unique");
    assert_eq!(potion_generation::candidates(Character::Silent, false).unwrap(), &expected);
    let limited: Vec<_> = expected.into_iter().filter(|id| *id != PotionId::FruitJuice).collect();
    assert_eq!(potion_generation::candidates(Character::Silent, true).unwrap(), limited);
    assert_eq!(limited.len(), 32);
    assert!(!limited.contains(&PotionId::FruitJuice));
    for id in expected {
        assert!(Catalog::standard().potion(id).is_some(), "unregistered generated ID: {id:?}");
        assert_eq!(Catalog::standard().potion_eligible_for(id, Character::Silent), Some(true));
    }
    assert_eq!(expected.iter().filter(|id| Catalog::standard().potion_rarity(**id) == Some(PotionRarity::Common)).count(), 17);
    assert_eq!(expected.iter().filter(|id| Catalog::standard().potion_rarity(**id) == Some(PotionRarity::Uncommon)).count(), 9);
    assert_eq!(expected.iter().filter(|id| Catalog::standard().potion_rarity(**id) == Some(PotionRarity::Rare)).count(), 7);
    assert!(matches!(potion_generation::candidates(Character::Ironclad, true), Err(InputError::Unsupported(_))));
}

#[test]
fn m15_limited_factory_rolls_rarity_before_discarding_flat_proposal_and_rejects_wrong_rarity() {
    let (potion, bounds, used) = potion_generation::sample_scripted(
        Character::Silent, true, &[70, 0, 3, 19]);
    assert_eq!(potion, PotionId::RegenPotion);
    assert_eq!(bounds, [100, 33, 33, 33]);
    assert_eq!(used, 4);
}

#[test]
fn m15_limited_factory_proposes_from_full_pool_then_rejects_fruit_juice() {
    let (potion, bounds, used) = potion_generation::sample_scripted(
        Character::Silent, true, &[90, 0, 28, 2]);
    assert_eq!(potion, PotionId::GhostInAJar);
    assert_eq!(bounds, [100, 33, 33, 33]);
    assert_eq!(used, 4);
}

#[test]
fn m15_unrestricted_factory_keeps_fruit_juice_and_rejects_after_one_rarity_roll() {
    let (uncommon, bounds, used) = potion_generation::sample_scripted(
        Character::Silent, false, &[70, 1]);
    assert_eq!(uncommon, PotionId::CunningPotion);
    assert_eq!(bounds, [100, 33]);
    assert_eq!(used, 2);

    let (fruit, bounds, used) = potion_generation::sample_scripted(
        Character::Silent, false, &[99, 28]);
    assert_eq!(fruit, PotionId::FruitJuice);
    assert_eq!(bounds, [100, 33]);
    assert_eq!(used, 2);
}

#[test]
fn m15_rarity_rejection_has_no_retry_cap() {
    let mut draws = vec![95, 0];
    draws.extend(std::iter::repeat(3).take(130));
    draws.push(32);
    let (potion, bounds, used) = potion_generation::sample_scripted(
        Character::Silent, true, &draws);
    assert_eq!(potion, PotionId::EntropicBrew);
    assert_eq!(used, 133);
    assert_eq!(bounds.len(), 133);
    assert!(bounds.iter().all(|bound| *bound == 33 || *bound == 100));
}

#[test]
fn m15_generated_obtain_uses_first_empty_then_returns_false_for_a_full_valid_belt() {
    let mut combat_items = crate::items::CombatItemContext::new(
        vec![crate::content::RelicId::Sozu],
        vec![Some(PotionId::BlockPotion), None, Some(PotionId::FirePotion)],
    );
    assert_eq!(combat_items.obtain_potion(PotionId::StrengthPotion), Ok(true));
    assert_eq!(combat_items.potions(), &[Some(PotionId::BlockPotion), Some(PotionId::StrengthPotion), Some(PotionId::FirePotion)]);

    let mut full = crate::items::CombatItemContext::new(
        vec![crate::content::RelicId::PotionBelt],
        vec![Some(PotionId::BlockPotion), Some(PotionId::FirePotion), Some(PotionId::SwiftPotion),
            Some(PotionId::StrengthPotion), Some(PotionId::EnergyPotion)],
    );
    let before = full.clone();
    assert_eq!(full.obtain_potion(PotionId::FruitJuice), Ok(false));
    assert_eq!(full, before, "full valid belt is a no-op and Sozu belongs to the resolver");
}

#[test]
fn m15_generated_obtain_rejects_invalid_ids_and_belt_shapes_at_the_helper_boundary() {
    let mut items = crate::items::CombatItemContext::new(vec![], vec![None, None]);
    assert!(matches!(items.obtain_potion(PotionId::StrengthPotion),
        Err(InputError::Invariant(message)) if message == "generated potion obtain input invalid"));

    let mut items = crate::items::CombatItemContext::new(vec![], vec![None, None, None]);
    assert!(matches!(items.obtain_potion(PotionId::Unknown("bad")),
        Err(InputError::Invariant(message)) if message == "generated potion obtain input invalid"));
}

#[test]
fn m15_run_potion_factory_scope_is_registered_without_rekeying_old_scopes() {
    use crate::run::RunRngScope;
    assert_eq!(RunRngScope::parse("run.m15.potion.generate"), Ok(RunRngScope::M15PotionGeneration));
    assert_eq!(RunRngScope::parse("run.m10.card.offer"), Ok(RunRngScope::M10CardOffer));
}

fn m15_outside_run() -> crate::run::Run {
    let mut profile = crate::run::CreationProfile::experienced();
    profile.neow_eligible = false;
    crate::run::Run::new(
        crate::run::RunConfig { character: Character::Silent, ascension: 0 },
        profile,
        crate::run::RunRng::new([41; 32], crate::combat::rng::Partition::Test, 0),
    ).unwrap()
}

#[test]
fn m15_run_obtain_uses_first_empty_full_valid_belt_and_rejects_unaccepted_bad_ids() {
    let mut run = m15_outside_run();
    let state = run.state_mut_for_test();
    state.potions = vec![Some(PotionId::BlockPotion), None, Some(PotionId::SwiftPotion)];
    assert_eq!(crate::items::obtain_run_potion(state, PotionId::StrengthPotion), Ok(true));
    assert_eq!(state.potions, [Some(PotionId::BlockPotion), Some(PotionId::StrengthPotion), Some(PotionId::SwiftPotion)]);

    let mut run = m15_outside_run();
    let state = run.state_mut_for_test();
    state.potions = vec![Some(PotionId::BlockPotion), Some(PotionId::FirePotion), Some(PotionId::SwiftPotion)];
    let before = state.potions.clone();
    assert_eq!(crate::items::obtain_run_potion(state, PotionId::FruitJuice), Ok(false));
    assert_eq!(state.potions, before);
    assert_eq!(crate::items::obtain_run_potion(state, PotionId::Unknown("bad")),
        Err(crate::run::RunError::Unsupported("generated potion obtain input invalid")));
}

#[test]
fn m15_accepted_run_obtain_fault_keeps_successful_prefix_and_first_fault() {
    let mut run = m15_outside_run();
    let state = run.state_mut_for_test();
    state.potions = vec![None, None, None];
    assert_eq!(crate::items::apply_accepted_run_potion_obtain(state, PotionId::StrengthPotion), Ok(true));
    let failed = crate::items::apply_accepted_run_potion_obtain(state, PotionId::Unknown("bad"));
    let Err(crate::run::RunError::Faulted(first)) = failed else { panic!("accepted bad obtain must retain a Run fault") };
    assert_eq!(first.kind, crate::run::RunFaultKind::Invariant("accepted potion obtain failed"));
    assert_eq!(first.checkpoint, "run.m15.potion.obtain");
    assert_eq!(state.potions, [Some(PotionId::StrengthPotion), None, None]);

    assert_eq!(crate::items::apply_accepted_run_potion_obtain(state, PotionId::SwiftPotion),
        Err(crate::run::RunError::Faulted(first.clone())));
    assert_eq!(state.fault, Some(first));
    assert_eq!(state.potions, [Some(PotionId::StrengthPotion), None, None]);
}

#[test]
fn m15_outside_brew_uses_live_three_or_five_slots_and_sozu_skips_generation() {
    for (slot_count, potion_belt) in [(3usize, false), (5usize, true)] {
        let mut run = m15_outside_run();
        let state = run.state_mut_for_test();
        if potion_belt { state.relics.push(crate::content::RelicId::PotionBelt); }
        state.potions = vec![None; slot_count];
        state.potions[0] = Some(PotionId::EntropicBrew);
        let rng_before = state.rng.clone();

        crate::items::use_m15_entropic_brew_outside(state, 0, false).unwrap();

        assert!(state.potions.iter().all(Option::is_some), "all {slot_count} proposals fit the vacated Brew slot and remaining empty slots");
        assert_ne!(state.rng, rng_before, "outside Brew samples before applying its obtains");
    }

    let mut run = m15_outside_run();
    let state = run.state_mut_for_test();
    state.relics.push(crate::content::RelicId::Sozu);
    state.potions = vec![Some(PotionId::EntropicBrew), None, None];
    let rng_before = state.rng.clone();
    crate::items::use_m15_entropic_brew_outside(state, 0, false).unwrap();
    assert_eq!(state.potions, [None, None, None]);
    assert_eq!(state.rng, rng_before, "outside Sozu check precedes every generation draw");
}

#[test]
fn m15_factory_keeps_source_rarity_boundaries_at_64_65_89_and_90() {
    for (draws, expected) in [
        (&[0, 0][..], PotionId::PoisonPotion),
        (&[64, 0, 1][..], PotionId::PoisonPotion),
        (&[65, 1][..], PotionId::CunningPotion),
        (&[89, 1][..], PotionId::CunningPotion),
        (&[90, 2][..], PotionId::GhostInAJar),
        (&[99, 2][..], PotionId::GhostInAJar),
    ] {
        assert_eq!(potion_generation::sample_scripted(Character::Silent, false, draws).0, expected);
    }
}

#[test]
fn m15_outside_brew_runs_ornithopter_once_even_when_sozu_skips_generation() {
    for sozu in [false, true] {
        let mut run = m15_outside_run();
        let state = run.state_mut_for_test();
        state.relics.push(crate::content::RelicId::ToyOrnithopter);
        if sozu { state.relics.push(crate::content::RelicId::Sozu); }
        state.hp -= 10;
        state.potions = vec![Some(PotionId::EntropicBrew), None, None];
        let before_hp = state.hp;
        let before_rng = state.rng.clone();

        crate::items::use_m15_entropic_brew_outside(state, 0, false).unwrap();

        assert_eq!(state.hp, before_hp + 5, "the manually used Brew triggers one ordinary use callback");
        assert_eq!(state.potions[0].is_some(), !sozu, "Sozu changes proposals, not the used-slot callback");
        if sozu { assert_eq!(state.rng, before_rng); }
    }
}

#[test]
fn m15_accepted_outside_obtain_checks_live_sozu_without_suppressing_invalid_input(){
    let mut run=m15_outside_run();let state=run.state_mut_for_test();state.relics.push(crate::content::RelicId::Sozu);state.potions=vec![None;3];
    assert_eq!(crate::items::apply_accepted_run_potion_obtain(state,PotionId::StrengthPotion),Ok(false));
    assert_eq!(state.potions,vec![None;3]);
    let failed=crate::items::apply_accepted_run_potion_obtain(state,PotionId::Unknown("bad_under_sozu"));
    assert!(matches!(failed,Err(crate::run::RunError::Faulted(_))));
    assert_eq!(state.fault.as_ref().unwrap().checkpoint,"run.m15.potion.obtain");
}
