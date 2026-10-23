use spire_core::{content::Character, run::{CreationProfile, Run, RunConfig, RunRng}};

#[test]
fn run_starts_with_owned_burning_blood_and_empty_potion_slots() {
    let run = Run::new(RunConfig { character: Character::Ironclad, ascension: 0 },
        CreationProfile::experienced(), RunRng::new([0; 32],
            spire_core::combat::rng::Partition::Test, 0)).unwrap();
    let player = run.decision().unwrap().player;
    assert_eq!(player.relics, [spire_core::content::RelicId::BurningBlood]);
    assert_eq!(player.potions, [None, None, None]);
}
