use crate::{combat::{CombatAction,TargetId},content::{CardId,Character,EncounterId},run::*};

#[test]
fn m10_rampage_combat_growth_does_not_mutate_run_master_deck_and_resets_next_entry(){
    let start=AuthoredRunStart{hp:80,max_hp:80,gold:99,
        deck:(1..=4).map(|instance|CardInstance{instance,id:CardId::Strike,upgrades:0}).collect(),
        relics:vec![],potions:vec![None;3],relic_counters:Default::default()};
    let mut run=Run::from_authored_combat(RunConfig{character:Character::Ironclad,ascension:0},start,
        RunRng::new([83;32],crate::combat::rng::Partition::Test,0)).unwrap();
    let deck=vec![CardInstance{instance:1,id:CardId::Rampage,upgrades:0},
        CardInstance{instance:2,id:CardId::Strike,upgrades:0},CardInstance{instance:3,id:CardId::Strike,upgrades:0},
        CardInstance{instance:4,id:CardId::Strike,upgrades:0}];
    run.state.deck=deck.clone();
    run.enter_combat(EncounterId::JawWorm).unwrap();
    let mut saw_grown=false;
    for _ in 0..40 {
        let Some(combat)=run.decision().unwrap().combat else{break};
        if let Some(grown)=combat.discard.iter().find(|card|card.instance.0==1&&card.id==CardId::Rampage) {
            saw_grown|=grown.values.as_ref().unwrap().base_damage==Some(13);
        }
        let playable=combat.hand.iter().find(|card|matches!(card.id,CardId::Rampage|CardId::Strike)
            &&card.effective_cost.is_some_and(|cost|cost<=combat.player.energy));
        if let Some(playable)=playable {
            run.choose_combat(CombatAction::PlayCard{instance:playable.instance.0,target:Some(TargetId(1))}).unwrap();
        } else {
            run.choose_combat(CombatAction::EndTurn).unwrap();
        }
        if run.state.combat.is_none(){break}
    }
    assert!(run.state.combat.is_none(),"bounded native combat must settle through the Run coordinator");
    assert!(saw_grown,"the actual in-combat card owner exposes its post-hit mutable damage");
    assert_eq!(run.state.deck,deck);
    run.enter_combat(EncounterId::Cultist).unwrap();
    let next=run.decision().unwrap().combat.unwrap();
    assert_eq!(next.hand.iter().find(|card|card.instance.0==1).unwrap().values.as_ref().unwrap().base_damage,Some(8),
        "fresh combat entry reconstructs Rampage from the unchanged permanent descriptor");
}
