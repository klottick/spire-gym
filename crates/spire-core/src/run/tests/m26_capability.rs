//! M26 Run card admission (plan Task M1, design §5.3).
use super::super::*;
use crate::{combat,content::{CardId,Character,EncounterId}};
use CardId::*;
fn rng()->RunRng{RunRng::new([7;32],combat::rng::Partition::Test,0)}
fn card(instance:u64,id:CardId,upgrades:u32)->CardInstance{CardInstance{instance,id,upgrades,misc:0}}
fn supplied(deck:Vec<CardInstance>)->AuthoredRunStart{
    AuthoredRunStart{hp:80,max_hp:80,gold:99,deck,relics:vec![],potions:vec![None;3],relic_counters:Default::default()}
}
fn mixed_deck()->Vec<CardInstance>{
    vec![card(1,StrikePurple,0),card(2,Zap,1),card(3,Apparition,0),card(4,Regret,0),card(5,SearingBlow,3),card(6,Strike,0)]
}

#[test]
fn m26_capability_run_admission(){
    let mut run=Run::prepare_complete_combat(CompleteCombatStart::Supplied(supplied(mixed_deck())),Character::Ironclad,EncounterId::Cultist,rng()).unwrap();
    assert_eq!(run.state.card_admission,CardAdmission::Complete);
    assert!(format!("{:?}",run.state).ends_with(", card_admission: Complete }"));
    run.enter_training_combat(EncounterId::Cultist).unwrap();
    assert!(run.state.combat.as_ref().unwrap().item_context().unwrap().complete_cards(),"Complete Run enters with the overlay");
    // Every character enters its Heart context plus the overlay (never the frozen M16 path).
    for character in [Character::Silent,Character::Defect,Character::Watcher]{
        let mut run=Run::prepare_complete_combat(CompleteCombatStart::Supplied(supplied(mixed_deck())),character,EncounterId::Cultist,rng()).unwrap();
        run.enter_training_combat(EncounterId::Cultist).unwrap_or_else(|e|panic!("{character:?}: {e:?}"));
        let items=run.state.combat.as_ref().unwrap().item_context().unwrap();
        assert!(items.complete_cards());
        assert_eq!((items.m15_profile(),items.m20_profile(),items.m25_profile()),
            (character==Character::Silent,character==Character::Defect,character==Character::Watcher),"{character:?} Heart context");
    }

    // A default Run's admission is Profile and its text omits the field.
    let default=Run::new(RunConfig{character:Character::Ironclad,ascension:0},CreationProfile::experienced(),rng()).unwrap();
    assert_eq!(default.state.card_admission,CardAdmission::Profile);
    assert!(!format!("{:?}",default.state).contains("card_admission"));
    let mut m21=Run::prepare_m21_watcher_combat(M21WatcherCombatStart::Canonical,EncounterId::Cultist,rng()).unwrap();
    m21.enter_training_combat(EncounterId::Cultist).unwrap();
    let items=m21.state.combat.as_ref().unwrap().item_context().unwrap();
    assert!(!items.complete_cards()&&!format!("{items:?}").contains("complete_cards"));

    // Rejections happen before any Run state is built.
    let ritual=CardInstance{instance:7,id:RitualDagger,upgrades:0,misc:15};
    assert!(CompleteCombatStart::Supplied(supplied(vec![card(1,Strike,0),ritual.clone(),CardInstance{instance:8,id:GeneticAlgorithm,upgrades:0,misc:1}])).normalized(Character::Silent).is_ok());
    for bad in [card(9,Burn,0),card(9,Shiv,0),card(9,Regret,1),card(9,Doubt,1),card(9,FameAndFortune,0),card(9,Strike,2),
        CardInstance{misc:14,..ritual.clone()},CardInstance{misc:0,..ritual},card(0,Strike,0),card(1,Strike,0)]{
        let mut deck=mixed_deck();deck.push(bad.clone());
        assert!(CompleteCombatStart::Supplied(supplied(deck.clone())).normalized(Character::Ironclad).is_err(),"rejects {bad:?}");
        assert!(Run::prepare_complete_combat(CompleteCombatStart::Supplied(supplied(deck)),Character::Ironclad,EncounterId::Cultist,rng()).is_err());
    }
}
