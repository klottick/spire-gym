//! M27 plan Task M5, A27 lane, key m27_potion_factory_helpers.

use crate::{
    combat::rng::Partition,
    content::{CardId, Character, EncounterId, PotionId, RelicId},
    items::{potion_generation, potion_offers},
    run::{AuthoredRunStart, CardAdmission, CompleteCombatStart, ItemAdmission, Run,
        RunAdmission, RunRng},
};

fn complete_run(character: Character, relics: Vec<RelicId>, potions: Vec<Option<PotionId>>) -> Run {
    let (strike, defend, signature) = match character {
        Character::Ironclad => (CardId::Strike, CardId::Defend, CardId::Bash),
        Character::Silent => (CardId::StrikeGreen, CardId::DefendGreen, CardId::Neutralize),
        Character::Defect => (CardId::StrikeBlue, CardId::DefendBlue, CardId::Zap),
        Character::Watcher => (CardId::StrikePurple, CardId::DefendPurple, CardId::Eruption),
    };
    let deck = [strike, strike, strike, defend, defend, signature].into_iter().enumerate()
        .map(|(index, id)| crate::run::CardInstance { instance: index as u64 + 1, id, upgrades: 0, misc: 0 })
        .collect();
    let start = AuthoredRunStart { hp: 80, max_hp: 80, gold: 0, deck, relics, potions,
        relic_counters: Default::default() };
    Run::prepare_complete_admission_combat(
        CompleteCombatStart::Supplied(start), character,
        RunAdmission::new(CardAdmission::Complete, ItemAdmission::Complete).unwrap(),
        EncounterId::Cultist, RunRng::new([0xA7; 32], Partition::Test, 0),
    ).unwrap()
}

fn settle_cultist_with_cards(run: &mut Run) {
    for _ in 0..64 {
        if run.state().combat.is_none() { return; }
        let actions = run.state().combat.as_ref().unwrap().legal_actions_full().unwrap();
        let action = actions.iter().find(|action| matches!(action,
            crate::combat::CombatAction::PlayCard { target: Some(_), .. }))
            .or_else(|| actions.iter().find(|action| matches!(action, crate::combat::CombatAction::EndTurn)))
            .or_else(|| actions.first())
            .cloned().expect("a live combat has a legal action");
        run.choose_combat(action).unwrap();
    }
    panic!("fixture Cultist combat did not settle within 64 actions");
}

#[test]
fn m27_potion_factory_helpers() {
    let classes = [
        (Character::Ironclad, PotionId::BloodPotion, PotionId::Elixir, PotionId::HeartOfIron),
        (Character::Silent, PotionId::PoisonPotion, PotionId::CunningPotion, PotionId::GhostInAJar),
        (Character::Defect, PotionId::FocusPotion, PotionId::PotionOfCapacity, PotionId::EssenceOfDarkness),
        (Character::Watcher, PotionId::BottledMiracle, PotionId::StancePotion, PotionId::Ambrosia),
    ];

    for (character, common, uncommon, rare) in classes {
        for (roll, proposal, expected) in [
            (64, 0, common),
            (65, 1, uncommon),
            (89, 1, uncommon),
            (90, 2, rare),
        ] {
            let mut script = [roll, proposal].into_iter();
            let mut bounds = Vec::new();
            let actual = potion_generation::rarity_first(character, |bound| {
                bounds.push(bound);
                Ok::<_, crate::run::RunError>(script.next().unwrap() % bound as usize)
            }).unwrap();
            assert_eq!(actual, expected, "{character:?} rarity roll {roll}");
            assert_eq!(bounds, [100, 33], "one rarity draw and one 33-pool proposal");
        }

        let pool = potion_generation::candidates(character, false).unwrap();
        assert_eq!(pool.len(), 33, "only the player's 33-potion class pool");
        for index in 0..33 {
            let mut bounds = Vec::new();
            let actual = potion_generation::flat(character, |bound| {
                bounds.push(bound);
                Ok::<_, crate::run::RunError>(index)
            }).unwrap();
            assert_eq!(actual, pool[index], "flat index {index} for {character:?}");
            assert_eq!(bounds, [33], "flat is one uniform proposal");
        }

        let fruit_index = pool.iter().position(|id| *id == PotionId::FruitJuice).unwrap();
        let mut unrestricted = [90, fruit_index].into_iter();
        assert_eq!(potion_generation::rarity_first(character, |bound| {
            Ok::<_, crate::run::RunError>(unrestricted.next().unwrap() % bound as usize)
        }).unwrap(), PotionId::FruitJuice, "unrestricted rarity-first can produce Fruit Juice");
        let (actual, bounds, used) = potion_generation::sample_scripted(
            character, true, &[90, fruit_index, 2]);
        assert_eq!(actual, rare, "limited generation excludes Fruit Juice");
        assert_eq!(bounds, [100, 33, 33], "limited generation discards its first proposal");
        assert_eq!(used, 3);
        assert_ne!(actual, PotionId::FruitJuice);
    }
}

#[test]
fn m27_entropic_brew_outside_combat_all_characters() {
    for character in [Character::Ironclad, Character::Silent, Character::Defect, Character::Watcher] {
        let mut run = complete_run(character, vec![], vec![
            Some(PotionId::EntropicBrew), Some(PotionId::BlockPotion), None,
        ]);
        let before = run.state().clone();
        let mut expected = before.clone();
        let floor = expected.floor as u64;
        // Main-10 m5: one proposal per live slot (3); was empty slots + 1 (2).
        potion_generation::sample_many_for_run(&mut expected, character, false, 3, &[floor, 0]).unwrap();
        crate::items::use_m15_entropic_brew_outside(run.state_mut_for_test(), 0, false).unwrap();
        let after = run.state();
        assert_eq!(after.potions.iter().flatten().count(), 3, "every empty slot fills for {character:?}");
        assert_eq!(after.potions[1], Some(PotionId::BlockPotion), "existing potion remains in place");
        assert!(after.potions.iter().flatten().all(|id|
            potion_generation::candidates(character, false).unwrap().contains(id)), "class pool for {character:?}");
        assert_eq!(after.rng, expected.rng, "one proposal draws per live potion slot");
    }

    let mut sozu = complete_run(Character::Ironclad, vec![RelicId::Sozu], vec![
        Some(PotionId::EntropicBrew), None, None,
    ]);
    let rng = sozu.state().rng.clone();
    crate::items::use_m15_entropic_brew_outside(sozu.state_mut_for_test(), 0, false).unwrap();
    assert_eq!(sozu.state().potions, [None, None, None]);
    assert_eq!(sozu.state().rng, rng, "Sozu is checked before proposal generation");
}

#[test]
fn m27_sacred_bark_potency_reinit_f4() {
    assert_eq!(crate::items::main_relics::initial_potion_potency(PotionId::PoisonPotion, false), 6);
    assert_eq!(crate::items::main_relics::initial_potion_potency(PotionId::PoisonPotion, true), 12);
    assert_eq!(crate::items::main_relics::initial_potion_potency(PotionId::Ambrosia, false), 2);
    assert_eq!(crate::items::main_relics::initial_potion_potency(PotionId::Ambrosia, true), 4);
    let mut run = complete_run(Character::Ironclad, vec![], vec![None; 3]);
    crate::items::obtain_run_potion(run.state_mut_for_test(), PotionId::BlockPotion).unwrap();
    assert_eq!(run.state().potion_potencies[0], Some((PotionId::BlockPotion, 12)));

    crate::items::lifecycle::obtain_relic(run.state_mut_for_test(), RelicId::SacredBark,
        crate::items::lifecycle::RelicSource::Reward).unwrap();
    assert_eq!(run.state().potion_potencies[0], Some((PotionId::BlockPotion, 24)));
    assert!(format!("{:?}", run.state()).contains("potion_potencies"));

    crate::items::lifecycle::lose_relic(run.state_mut_for_test(), RelicId::SacredBark).unwrap();
    assert_eq!(run.state().potion_potencies[0], Some((PotionId::BlockPotion, 24)),
        "losing Sacred Bark preserves already stored potency");
    let context = crate::items::CombatItemContext::take_from_run(run.state_mut_for_test()).with_complete_items();
    assert_eq!(context.potion_potency(0), Some(24), "the stored value moves into combat custody");
    context.return_to_run(run.state_mut_for_test()).unwrap();
    assert_eq!(run.state().potion_potencies[0], Some((PotionId::BlockPotion, 24)),
        "the stored value returns to Run custody");

    let mut supplied_bark = complete_run(Character::Ironclad, vec![RelicId::SacredBark], vec![
        Some(PotionId::BlockPotion), None, None,
    ]);
    crate::items::lifecycle::lose_relic(supplied_bark.state_mut_for_test(), RelicId::SacredBark).unwrap();
    assert_eq!(supplied_bark.state().potion_potencies[0], Some((PotionId::BlockPotion, 24)),
        "loss before the first combat still retains the held potion's potency");
}

#[test]
fn m27_sacred_bark_lost_potency_reaches_combat_use() {
    // F4 end to end (Main-4 integration of A26's potency reader with A27's stored potency): Block Potion held when
    // Sacred Bark is obtained gives 24 Block in combat after Bark is lost; the Watcher path also passes its startup profile.
    for character in [Character::Ironclad, Character::Watcher] {
        let mut run = complete_run(character, vec![], vec![Some(PotionId::BlockPotion), None, None]);
        crate::items::lifecycle::obtain_relic(run.state_mut_for_test(), RelicId::SacredBark,
            crate::items::lifecycle::RelicSource::Reward).unwrap();
        crate::items::lifecycle::lose_relic(run.state_mut_for_test(), RelicId::SacredBark).unwrap();
        run.state_mut_for_test().training_encounter = Some(EncounterId::Cultist);
        run.enter_training_combat(EncounterId::Cultist).unwrap();
        assert_eq!(run.state().combat.as_ref().unwrap().item_context().unwrap().potion_potency(0), Some(24), "{character:?}");
        let block = run.state().combat.as_ref().unwrap().public_full().unwrap().player.block;
        run.choose_combat(crate::combat::CombatAction::UsePotion { slot: 0, target: None }).unwrap();
        assert_eq!(run.state().combat.as_ref().unwrap().public_full().unwrap().player.block - block, 24,
            "{character:?} Block Potion keeps its Bark potency after Bark is lost");
    }
}

#[test]
fn m27_prismatic_shard_master_orb_slot() {
    for (character, expected_slots) in [
        (Character::Ironclad, 1), (Character::Silent, 1),
        (Character::Watcher, 1), (Character::Defect, 3),
    ] {
        let mut run = complete_run(character, vec![], vec![Some(PotionId::FirePotion); 3]);
        crate::items::lifecycle::obtain_relic(run.state_mut_for_test(), RelicId::PrismaticShard,
            crate::items::lifecycle::RelicSource::Shop).unwrap();
        run.state_mut_for_test().training_encounter = Some(EncounterId::Cultist);
        run.enter_training_combat(EncounterId::Cultist).unwrap();
        let slots = run.state().combat.as_ref().unwrap().public_full().unwrap().character.orbs.len();
        assert_eq!(slots, expected_slots, "first complete combat for {character:?}");
        settle_cultist_with_cards(&mut run);
        crate::items::tests::m27_support::next_complete_combat(&mut run, EncounterId::Cultist).unwrap();
        let later_slots = run.state().combat.as_ref().unwrap().public_full().unwrap().character.orbs.len();
        assert_eq!(later_slots, expected_slots, "subsequent complete combat for {character:?}");
    }
}

#[test]
fn m27_potion_appearance_law() {
    let mut run = complete_run(Character::Ironclad, vec![], vec![None; 3]);
    assert_eq!(potion_offers::combat_appearance_scripted(
        run.state_mut_for_test(), potion_offers::RewardRoom::Elite, 0, &[39, 0, 0]), Ok(Some(PotionId::BloodPotion)));
    assert_eq!(run.state().potion_offset, -10, "the successful offer lowers the next chance by ten");
    assert_eq!(potion_offers::combat_appearance_scripted(
        run.state_mut_for_test(), potion_offers::RewardRoom::Event, 0, &[30]), Ok(None));
    assert_eq!(run.state().potion_offset, 0, "roll 30 fails when chance is exactly 30");

    let mut escaped = complete_run(Character::Ironclad, vec![], vec![None; 3]);
    assert_eq!(potion_offers::combat_appearance_scripted(
        escaped.state_mut_for_test(), potion_offers::RewardRoom::Ordinary { enemy_escaped: true }, 0, &[0]), Ok(None));
    assert_eq!(escaped.state().potion_offset, 10, "escaped enemy uses chance zero from offset zero");

    let mut statue = complete_run(Character::Ironclad, vec![RelicId::WhiteBeastStatue], vec![None; 3]);
    assert_eq!(potion_offers::combat_appearance_scripted(
        statue.state_mut_for_test(), potion_offers::RewardRoom::Boss, 0, &[99, 90, 2]), Ok(Some(PotionId::HeartOfIron)));

    let mut full_rows = complete_run(Character::Ironclad, vec![RelicId::WhiteBeastStatue], vec![None; 3]);
    assert_eq!(potion_offers::combat_appearance_scripted(
        full_rows.state_mut_for_test(), potion_offers::RewardRoom::Elite, 4, &[99]), Ok(None));
}

#[test]
fn m27_review_m6_escaped_potion_miss_keeps_offset_history() {
    let mut escaped = complete_run(Character::Ironclad, vec![], vec![None; 3]);
    escaped.state_mut_for_test().potion_offset = -20;
    assert_eq!(potion_offers::combat_appearance_scripted(
        escaped.state_mut_for_test(), potion_offers::RewardRoom::Ordinary { enemy_escaped: true }, 0, &[0]),
        Ok(None));
    assert_eq!(escaped.state().potion_offset, -10,
        "an escaped enemy's miss advances the existing offset by ten");

    let mut statue = complete_run(Character::Ironclad, vec![RelicId::WhiteBeastStatue], vec![None; 3]);
    statue.state_mut_for_test().potion_offset = -20;
    assert!(potion_offers::combat_appearance_scripted(
        statue.state_mut_for_test(), potion_offers::RewardRoom::Ordinary { enemy_escaped: true }, 0, &[0, 0, 0])
        .unwrap().is_some(), "White Beast Statue makes the escaped-enemy roll hit");
    assert_eq!(statue.state().potion_offset, -30,
        "a successful escaped-enemy offer advances the existing offset by minus ten");
}
