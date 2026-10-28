use crate::{combat::{CombatAction,TargetId},content::{CardId,Character,EncounterId,PotionId},run::*};

#[test]
fn m10_dual_wield_replacement_survives_real_run_exit_without_deck_deletion_or_gain(){
    let start=AuthoredRunStart{hp:80,max_hp:80,gold:99,
        deck:(1..=4).map(|instance|CardInstance{ misc:0,instance,id:CardId::Strike,upgrades:0}).collect(),
        relics:vec![],potions:vec![Some(PotionId::FirePotion),None,None],relic_counters:Default::default()};
    let mut run=Run::from_authored_combat(RunConfig{character:Character::Ironclad,ascension:0},start,
        RunRng::new([29;32],crate::combat::rng::Partition::Test,0)).unwrap();
    let deck=vec![CardInstance{ misc:0,instance:1,id:CardId::DualWield,upgrades:0},
        CardInstance{ misc:0,instance:2,id:CardId::SearingBlow,upgrades:3},
        CardInstance{ misc:0,instance:3,id:CardId::Inflame,upgrades:0},CardInstance{ misc:0,instance:4,id:CardId::Bash,upgrades:0}];
    run.state.deck=deck.clone();
    run.enter_combat(EncounterId::JawWorm).unwrap();
    let ready=run.decision().unwrap();
    let initial=ready.combat.unwrap();
    assert_eq!(initial.hand.len(),4);
    assert_eq!(initial.hand.iter().find(|card|card.id==CardId::DualWield).unwrap().instance.0,1);
    run.choose_combat(CombatAction::PlayCard{instance:1,target:None}).unwrap();
    run.choose_combat(CombatAction::SelectCard{instance:2}).unwrap();
    assert_eq!(run.state.deck,deck);
    let ready=run.decision().unwrap();
    let copy=ready.combat.unwrap().hand.iter().find(|card|card.id==CardId::SearingBlow).unwrap().instance.0;
    assert_ne!(copy,2,"multi-eligible selected permanent was replaced by a fresh instance");
    run.choose_combat(CombatAction::PlayCard{instance:copy,target:Some(TargetId(1))}).unwrap();
    run.choose_combat(CombatAction::UsePotion{slot:0,target:Some(TargetId(1))}).unwrap();
    assert!(run.state.combat.is_none(),"actual combat victory settled through Run owner");
    assert_eq!(run.state.deck,deck,"retired source remains in the permanent master deck and copies add no permanent entries");
    run.enter_combat(EncounterId::Cultist).unwrap();
    let next=run.decision().unwrap().combat.unwrap();
    assert_eq!(next.hand.iter().find(|card|card.instance.0==2).unwrap().upgrades,3);
    assert_eq!(run.state.deck,deck);
}
