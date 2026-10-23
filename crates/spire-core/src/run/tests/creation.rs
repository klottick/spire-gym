use crate::{content::Character, run::*};

#[test]
fn all_84_starts() {
    for character in Character::ALL {
        for ascension in 0..=20 {
            let run = Run::new(RunConfig { character, ascension }, CreationProfile::experienced(),
                RunRng::new([ascension; 32], crate::combat::rng::Partition::Test, 0)).unwrap();
            assert_eq!(run.state().character, character);
            assert_eq!(run.state().ascension, ascension);
            assert!(run.decision().unwrap().candidates.len() >= 2);
            let count=(match character { Character::Silent=>12,_=>10 })+usize::from(ascension>=10);
            assert_eq!(run.state().deck.len(), count);
            assert_eq!(run.state().deck.iter().map(|c| c.instance).collect::<std::collections::HashSet<_>>().len(), count);
            let starters=match character {
                Character::Ironclad=>vec![crate::content::CardId::Strike;5].into_iter().chain(vec![crate::content::CardId::Defend;4]).chain([crate::content::CardId::Bash]).collect::<Vec<_>>(),
                Character::Silent=>vec![crate::content::CardId::StrikeGreen;5].into_iter().chain(vec![crate::content::CardId::DefendGreen;5]).chain([crate::content::CardId::Survivor,crate::content::CardId::Neutralize]).collect(),
                Character::Defect=>vec![crate::content::CardId::StrikeBlue;4].into_iter().chain(vec![crate::content::CardId::DefendBlue;4]).chain([crate::content::CardId::Zap,crate::content::CardId::Dualcast]).collect(),
                Character::Watcher=>vec![crate::content::CardId::StrikePurple;4].into_iter().chain(vec![crate::content::CardId::DefendPurple;4]).chain([crate::content::CardId::Eruption,crate::content::CardId::Vigilance]).collect(),
            };
            assert_eq!(run.state().deck.iter().take(starters.len()).map(|c|c.id).collect::<Vec<_>>(),starters);
            let (expected_hp,expected_max)=match (character,ascension) {
                (Character::Ironclad,0..=5)=>(80,80), (Character::Ironclad,6..=13)=>(72,80), (Character::Ironclad,14..=20)=>(68,75),
                (Character::Silent,0..=5)=>(70,70), (Character::Silent,6..=13)=>(63,70), (Character::Silent,14..=20)=>(59,66),
                (Character::Defect,0..=5)=>(75,75), (Character::Defect,6..=13)=>(68,75), (Character::Defect,14..=20)=>(64,71),
                (Character::Watcher,0..=5)=>(72,72), (Character::Watcher,6..=13)=>(65,72), (Character::Watcher,14..=20)=>(61,68),
                (_,_)=>unreachable!(),
            };
            assert_eq!((run.state().hp,run.state().max_hp,run.state().gold),(expected_hp,expected_max,99));
            assert!(run.state().neow_eligible);
            assert!(run.state().act4_enabled);
            assert!(run.state().seen_bosses.iter().all(|seen|!seen.is_empty()));
            assert_eq!(run.state().stored_note.0,crate::content::CardId::IronWave);
            assert_eq!(run.state().stored_note.1,0);
            assert_eq!(run.state().potions.len(),if ascension>=11 {2} else {3});
            assert_eq!(run.state().deck.iter().any(|c| c.id==crate::content::CardId::AscendersBane),ascension>=10);
            assert_eq!(run.state().relics.len(),1);
            assert_eq!(run.state().keys,[false;3]);
        }
    }
}

#[test]
fn invalid_ascension_rejects() {
    assert!(Run::new(RunConfig { character: Character::Watcher, ascension: 21 }, CreationProfile::experienced(),
        RunRng::new([0; 32], crate::combat::rng::Partition::Test, 0)).is_err());
}

#[test]
fn duplicate_instance_survives_remove() {
    let mut run = Run::new(RunConfig { character: Character::Ironclad, ascension: 0 }, CreationProfile::experienced(),
        RunRng::new([0; 32], crate::combat::rng::Partition::Test, 0)).unwrap();
    let strikes=run.state().deck.iter().filter(|c|c.id==crate::content::CardId::Strike).map(|c|c.instance).collect::<Vec<_>>();
    assert_eq!(strikes.len(),5);
    run.remove_card(strikes[0]).unwrap();
    assert_eq!(run.state().deck.iter().filter(|c|c.id==crate::content::CardId::Strike).count(),4);
    assert!(!run.state().deck.iter().any(|c|c.instance==strikes[0]));
    assert!(strikes[1..].iter().all(|id|run.state().deck.iter().any(|c|c.instance==*id)));
}
