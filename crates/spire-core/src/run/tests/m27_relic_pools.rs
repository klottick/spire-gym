//! Literal M27 relic pool laws (design §7.4, economy research §4.1).
use super::super::*;
use crate::{
    content::{CardId, Character, EncounterId, RelicId, RelicTier},
    items::{lifecycle::RelicSource, relic_pools::{self, RelicPools, Selector, TierTable}},
};

fn supplied(character: Character, relics: Vec<RelicId>) -> AuthoredRunStart {
    let deck = crate::content::Catalog::standard().starter_deck(character).unwrap().iter().enumerate()
        .map(|(i, id)| CardInstance { id: *id, instance: i as u64 + 1, upgrades: 0, misc: 0 }).collect();
    AuthoredRunStart { hp: 80, max_hp: 80, gold: 99, deck, relics, potions: vec![None; 3], relic_counters: Default::default() }
}

fn complete(character: Character, relics: Vec<RelicId>) -> Run {
    let start = supplied(character, relics);
    let admission = RunAdmission::new(CardAdmission::Complete, ItemAdmission::Complete).unwrap();
    Run::prepare_complete_admission_combat(CompleteCombatStart::Supplied(start), character, admission,
        EncounterId::Cultist, RunRng::new([7; 32], crate::combat::rng::Partition::Test, 0)).unwrap()
}

fn pools(common: &[RelicId], uncommon: &[RelicId], rare: &[RelicId], shop: &[RelicId], boss: &[RelicId]) -> RelicPools {
    relic_pools::explicit_for_test(common.to_vec(), uncommon.to_vec(), rare.to_vec(), shop.to_vec(), boss.to_vec())
}

fn set_pools(run: &mut Run, pools: RelicPools) { run.state.relic_pools = Some(pools); }

#[test]
fn m27_relic_pool_law_build_excludes_owned_and_other_classes_for_each_character() {
    let cases = [
        (Character::Ironclad, RelicId::BurningBlood, RelicId::Brimstone, RelicId::DataDisk),
        (Character::Silent, RelicId::RingOfTheSnake, RelicId::TwistedFunnel, RelicId::Brimstone),
        (Character::Defect, RelicId::CrackedCore, RelicId::DataDisk, RelicId::TwistedFunnel),
        (Character::Watcher, RelicId::PureWater, RelicId::VioletLotus, RelicId::DataDisk),
    ];
    for (character, starter, own_class, other_class) in cases {
        let run = complete(character, vec![starter, RelicId::Anchor]);
        let built = run.state.relic_pools.as_ref().expect("complete admission builds relic pools");
        let ids = [&built.common, &built.uncommon, &built.rare, &built.shop, &built.boss]
            .into_iter().flat_map(|tier| tier.iter().copied()).collect::<Vec<_>>();
        assert!(!ids.contains(&starter), "owned starter is removed for {character:?}");
        assert!(!ids.contains(&RelicId::Anchor), "owned shared relic is removed for {character:?}");
        assert!(ids.contains(&own_class), "own-class relic remains for {character:?}");
        assert!(!ids.contains(&other_class), "other-class relic is absent for {character:?}");
    }
}

#[test]
fn m27_relic_pool_law_front_back_boss_front_and_consumed_if_skipped() {
    let mut front = complete(Character::Ironclad, vec![RelicId::BurningBlood]);
    set_pools(&mut front, pools(&[RelicId::Anchor, RelicId::BloodVial], &[], &[], &[],
        &[RelicId::BlackBlood, RelicId::SacredBark]));
    assert_eq!(relic_pools::take(&mut front.state, RelicTier::Common, Selector::Front, RelicSource::Reward, &[1]), Ok(RelicId::Anchor));
    assert_eq!(front.state.relic_pools.as_ref().unwrap().common, vec![RelicId::BloodVial], "offered item is consumed when skipped");
    assert_eq!(relic_pools::take(&mut front.state, RelicTier::Common, Selector::Back, RelicSource::Reward, &[2]), Ok(RelicId::BloodVial));
    assert_eq!(relic_pools::take(&mut front.state, RelicTier::Boss, Selector::Back, RelicSource::Boss, &[3]), Ok(RelicId::BlackBlood), "boss always selects its front");
}

#[test]
fn m27_relic_pool_law_remove_and_excluded_candidates_are_consumed() {
    let mut run = complete(Character::Ironclad, vec![RelicId::BurningBlood]);
    set_pools(&mut run, pools(&[RelicId::TheCourier, RelicId::Anchor], &[RelicId::Girya], &[RelicId::Girya], &[], &[]));
    let courier_exclusions = [RelicId::OldCoin, RelicId::SmilingMask, RelicId::MawBank, RelicId::TheCourier];
    assert_eq!(relic_pools::exclude_take(&mut run.state, RelicTier::Common, Selector::Front, RelicSource::Reward,
        &courier_exclusions, &[11]), Ok(RelicId::Anchor));
    assert!(!run.state.relic_pools.as_ref().unwrap().common.contains(&RelicId::TheCourier));
    let black_star_exclusions = [RelicId::Girya, RelicId::PeacePipe, RelicId::Shovel];
    set_pools(&mut run, pools(&[RelicId::Girya, RelicId::Anchor], &[RelicId::Girya], &[RelicId::Girya], &[], &[]));
    assert_eq!(relic_pools::exclude_take(&mut run.state, RelicTier::Common, Selector::Front, RelicSource::Elite,
        &black_star_exclusions, &[12]), Ok(RelicId::Anchor));
    assert!(!run.state.relic_pools.as_ref().unwrap().common.contains(&RelicId::Girya));
    let pools = run.state.relic_pools.as_mut().unwrap();
    relic_pools::remove(pools, RelicId::Girya);
    assert!(pools.common.is_empty() && pools.uncommon.is_empty() && pools.rare.is_empty(), "obtained relics are removed from every tier");
}

#[test]
fn m27_relic_pool_law_spawn_failure_consumes_candidate_then_retries_back_at_original_tier() {
    let mut run = complete(Character::Ironclad, vec![RelicId::BurningBlood]);
    run.state.floor = 49;
    set_pools(&mut run, pools(&[RelicId::MawBank, RelicId::Anchor, RelicId::BloodVial], &[], &[], &[], &[]));
    assert_eq!(relic_pools::take(&mut run.state, RelicTier::Common, Selector::Front, RelicSource::Reward, &[4]), Ok(RelicId::BloodVial));
    assert_eq!(run.state.relic_pools.as_ref().unwrap().common, vec![RelicId::Anchor], "Maw Bank fails above floor 48 and is consumed before the back retry");
}

#[test]
fn m27_relic_pool_law_empty_boss_returns_red_circlet() {
    let mut run = complete(Character::Ironclad, vec![RelicId::BurningBlood]);
    set_pools(&mut run, pools(&[], &[RelicId::PaperPhrog], &[RelicId::Mango], &[RelicId::Brimstone], &[RelicId::BlackBlood]));
    assert_eq!(relic_pools::take(&mut run.state, RelicTier::Common, Selector::Back, RelicSource::Reward, &[5]), Ok(RelicId::PaperPhrog));
    set_pools(&mut run, pools(&[], &[], &[RelicId::Mango], &[RelicId::Brimstone], &[RelicId::BlackBlood]));
    assert_eq!(relic_pools::take(&mut run.state, RelicTier::Uncommon, Selector::Back, RelicSource::Reward, &[6]), Ok(RelicId::Mango));
    set_pools(&mut run, pools(&[], &[RelicId::PaperPhrog], &[RelicId::Mango], &[], &[RelicId::BlackBlood]));
    assert_eq!(relic_pools::take(&mut run.state, RelicTier::Shop, Selector::Back, RelicSource::Shop, &[7]), Ok(RelicId::PaperPhrog));
    set_pools(&mut run, pools(&[], &[], &[], &[RelicId::Brimstone], &[RelicId::BlackBlood]));
    assert_eq!(relic_pools::take(&mut run.state, RelicTier::Rare, Selector::Back, RelicSource::Reward, &[8]), Ok(RelicId::Circlet));
    set_pools(&mut run, pools(&[RelicId::Anchor], &[RelicId::PaperPhrog], &[RelicId::Mango], &[RelicId::Brimstone], &[]));
    let pools_before = run.state.relic_pools.clone();
    let rng_before = run.state.rng.clone();
    assert_eq!(relic_pools::take(&mut run.state, RelicTier::Boss, Selector::Back, RelicSource::Boss, &[9]),
        Ok(RelicId::RedCirclet));
    assert_eq!(run.state.relic_pools, pools_before, "empty boss fallback does not consume any tier");
    assert_eq!(run.state.rng, rng_before, "empty boss fallback consumes no randomness");
}

#[test]
fn m27_relic_pool_law_roll_tier_boundaries_for_both_tables() {
    let ordinary = [(0, RelicTier::Common), (49, RelicTier::Common), (50, RelicTier::Uncommon),
        (82, RelicTier::Uncommon), (83, RelicTier::Rare), (99, RelicTier::Rare)];
    let merchant = [(0, RelicTier::Common), (47, RelicTier::Common), (48, RelicTier::Uncommon),
        (81, RelicTier::Uncommon), (82, RelicTier::Rare), (99, RelicTier::Rare)];
    for (roll, expected) in ordinary { assert_eq!(relic_pools::roll_tier_with_roll(TierTable::Ordinary, roll), expected, "ordinary {roll}"); }
    for (roll, expected) in merchant { assert_eq!(relic_pools::roll_tier_with_roll(TierTable::Merchant, roll), expected, "merchant {roll}"); }
}

#[test]
fn m27_relic_pool_law_black_blood_needs_burning_blood() {
    let mut run = complete(Character::Ironclad, vec![RelicId::Anchor]);
    set_pools(&mut run, pools(&[], &[], &[], &[], &[RelicId::BlackBlood, RelicId::SacredBark]));
    assert_eq!(relic_pools::take(&mut run.state, RelicTier::Boss, Selector::Front, RelicSource::Boss, &[10]), Ok(RelicId::SacredBark));
    assert!(!run.state.relic_pools.as_ref().unwrap().boss.contains(&RelicId::BlackBlood), "failed replacement candidate remains consumed");
}

#[test]
fn m27_relic_pool_law_each_boss_replacement_needs_its_starter() {
    let mut defect = complete(Character::Defect, vec![]);
    assert!(!relic_pools::spawn_allowed_for_test(&defect.state, RelicId::FrozenCore, RelicSource::Boss));
    defect.state.relics.push(RelicId::CrackedCore);
    assert!(relic_pools::spawn_allowed_for_test(&defect.state, RelicId::FrozenCore, RelicSource::Boss));

    let mut watcher = complete(Character::Watcher, vec![]);
    assert!(!relic_pools::spawn_allowed_for_test(&watcher.state, RelicId::HolyWater, RelicSource::Boss));
    watcher.state.relics.push(RelicId::PureWater);
    assert!(relic_pools::spawn_allowed_for_test(&watcher.state, RelicId::HolyWater, RelicSource::Boss));

    let mut silent = complete(Character::Silent, vec![]);
    assert!(!relic_pools::spawn_allowed_for_test(&silent.state, RelicId::RingOfTheSerpent, RelicSource::Boss));
    silent.state.relics.push(RelicId::RingOfTheSnake);
    assert!(relic_pools::spawn_allowed_for_test(&silent.state, RelicId::RingOfTheSerpent, RelicSource::Boss));
}

#[test]
fn m27_relic_pool_law_spawn_restrictions_use_floors_acts_deck_and_source() {
    let mut run = complete(Character::Ironclad, vec![RelicId::BurningBlood]);
    let state = &mut run.state;
    state.floor = 48;
    for id in [RelicId::TheCourier, RelicId::MawBank, RelicId::OldCoin, RelicId::SmilingMask,
        RelicId::AncientTeaSet, RelicId::CeramicFish, RelicId::DarkstonePeriapt, RelicId::DreamCatcher,
        RelicId::FrozenEgg, RelicId::MealTicket, RelicId::MeatOnTheBone, RelicId::MoltenEgg, RelicId::Omamori,
        RelicId::PotionBelt, RelicId::PrayerWheel, RelicId::QuestionCard, RelicId::RegalPillow,
        RelicId::SingingBowl, RelicId::ToxicEgg, RelicId::JuzuBracelet] {
        assert!(relic_pools::spawn_allowed_for_test(state, id, RelicSource::Reward), "{id:?} at floor 48");
        state.floor = 49;
        assert!(!relic_pools::spawn_allowed_for_test(state, id, RelicSource::Reward), "{id:?} at floor 49");
        state.floor = 48;
    }
    for id in [RelicId::TheCourier, RelicId::MawBank, RelicId::OldCoin, RelicId::SmilingMask] {
        assert!(!relic_pools::spawn_allowed_for_test(state, id, RelicSource::Shop), "{id:?} is not a merchant offer");
    }
    for (id, last_allowed) in [(RelicId::Matryoshka, 40), (RelicId::WingBoots, 40), (RelicId::TinyChest, 35), (RelicId::PreservedInsect, 52)] {
        state.floor = last_allowed;
        assert!(relic_pools::spawn_allowed_for_test(state, id, RelicSource::Reward), "{id:?} at {last_allowed}");
        state.floor = last_allowed + 1;
        assert!(!relic_pools::spawn_allowed_for_test(state, id, RelicSource::Reward), "{id:?} above {last_allowed}");
    }
    state.floor = 0;
    state.act = 1;
    assert!(relic_pools::spawn_allowed_for_test(state, RelicId::Ectoplasm, RelicSource::Reward));
    state.act = 2;
    assert!(!relic_pools::spawn_allowed_for_test(state, RelicId::Ectoplasm, RelicSource::Reward));
    state.act = 1;
    state.floor = 47;
    assert!(relic_pools::spawn_allowed_for_test(state, RelicId::Girya, RelicSource::Dig));
    // Main-3: ordinary sources keep the campfire relics (economy research §4.1 applies the
    // exclusion only to the noncampfire selector family, i.e. Black Star's `exclude_take`).
    assert!(relic_pools::spawn_allowed_for_test(state, RelicId::Girya, RelicSource::Reward));
    assert!(relic_pools::spawn_allowed_for_test(state, RelicId::Shovel, RelicSource::Elite));
    state.floor = 48;
    assert!(!relic_pools::spawn_allowed_for_test(state, RelicId::Shovel, RelicSource::Dig));
    state.relics.extend([RelicId::PeacePipe, RelicId::Shovel]);
    state.floor = 47;
    assert!(!relic_pools::spawn_allowed_for_test(state, RelicId::Girya, RelicSource::Dig), "two campfire relics already owned");
    state.relics.retain(|id| *id != RelicId::PeacePipe && *id != RelicId::Shovel);
    assert!(!relic_pools::spawn_allowed_for_test(state, RelicId::BottledFlame, RelicSource::Reward), "starter Attack is Basic");
    assert!(!relic_pools::spawn_allowed_for_test(state, RelicId::BottledLightning, RelicSource::Reward), "starter Skills are Basic");
    state.deck.push(CardInstance { id: CardId::Clothesline, instance: 99, upgrades: 0, misc: 0 });
    assert!(relic_pools::spawn_allowed_for_test(state, RelicId::BottledFlame, RelicSource::Reward));
    state.deck.push(CardInstance { id: CardId::Deflect, instance: 100, upgrades: 0, misc: 0 });
    state.deck.push(CardInstance { id: CardId::Inflame, instance: 101, upgrades: 0, misc: 0 });
    assert!(relic_pools::spawn_allowed_for_test(state, RelicId::BottledLightning, RelicSource::Reward));
    assert!(relic_pools::spawn_allowed_for_test(state, RelicId::BottledTornado, RelicSource::Reward));
    assert!(!relic_pools::spawn_allowed_for_test(state, RelicId::BottledFlame, RelicSource::Event));
    assert!(!relic_pools::spawn_allowed_for_test(state, RelicId::BottledLightning, RelicSource::Equip(RelicId::CallingBell)));
    assert!(!relic_pools::spawn_allowed_for_test(state, RelicId::BottledTornado, RelicSource::Event));
    assert!(!relic_pools::spawn_allowed_for_test(state, RelicId::BottledFlame, RelicSource::Equip(RelicId::CallingBell)));
    assert!(!relic_pools::spawn_allowed_for_test(state, RelicId::Whetstone, RelicSource::Event));
    assert!(!relic_pools::spawn_allowed_for_test(state, RelicId::TheCourier, RelicSource::Shop));
}

#[test]
fn m27_relic_pool_law_profile_state_debug_and_rng_remain_inert() {
    let rng = RunRng::new([7; 32], crate::combat::rng::Partition::Test, 0);
    let mut profile = CreationProfile::experienced();
    profile.neow_eligible = false;
    let run = Run::new(RunConfig { character: Character::Ironclad, ascension: 0 }, profile, rng.clone()).unwrap();
    assert!(run.state.relic_pools.is_none());
    assert!(!format!("{:?}", run.state).contains("relic_pools"));
    let accepted = Run::prepare_complete_combat(CompleteCombatStart::Supplied(supplied(Character::Ironclad, vec![])),
        Character::Ironclad, EncounterId::Cultist, rng.clone()).unwrap();
    assert!(format!("{:?}", accepted.state).ends_with(", card_admission: Complete }"), "accepted profile state Debug suffix remains byte-stable");
    assert!(!format!("{:?}", accepted.state).contains("relic_pools"));
    let mut actual = run.state.rng.clone();
    let mut expected = rng;
    for (scope, key) in [(RunRngScope::M27RelicPools, &[1][..]), (RunRngScope::M27RelicTier, &[2][..]), (RunRngScope::M27PotionOffer, &[3][..])] {
        assert_eq!(actual.draw_scoped(scope, key), expected.draw_scoped(scope, key), "profile made no {scope:?} draw");
    }
    for (label, scope) in [("run.m27.relic.pools", RunRngScope::M27RelicPools), ("run.m27.relic.tier", RunRngScope::M27RelicTier),
        ("run.m27.potion.offer", RunRngScope::M27PotionOffer)] {
        assert_eq!(RunRngScope::parse(label), Ok(scope));
    }
    assert!(RunRngScope::M26CardCurse < RunRngScope::M27RelicPools);
    assert!(RunRngScope::M27RelicPools < RunRngScope::M27RelicTier);
    assert!(RunRngScope::M27RelicTier < RunRngScope::M27PotionOffer);
}
