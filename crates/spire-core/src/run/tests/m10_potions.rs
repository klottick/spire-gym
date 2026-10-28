use crate::{combat::{self,CombatAction,DamagePayload,Effect,EffectKind,Source,Actor,rng::Partition},content::{CardId,Character,EncounterId,PotionId,RelicId},run::*};

#[test]
fn m10_elixir_and_heart_of_iron_are_not_usable_outside_combat(){
    for potion in [PotionId::Elixir,PotionId::HeartOfIron]{
        let start=AuthoredRunStart{hp:50,max_hp:80,gold:0,deck:vec![CardInstance{ misc:0,instance:1,id:CardId::Defend,upgrades:0}],
            relics:vec![],potions:vec![Some(potion),None,None],relic_counters:Default::default()};
        let run=Run::from_authored_combat(RunConfig{character:Character::Ironclad,ascension:0},start,
            RunRng::new([31;32],Partition::Test,0)).unwrap();
        assert_eq!(crate::items::prepare_run_potion(run.state(),0,false,false),Err(RunError::InvalidAction),"{potion:?}");
    }
}

#[test]
fn m10_elixir_uses_transferred_run_inventory_and_returns_consumed_belt_to_death_settlement(){
    let start=AuthoredRunStart{hp:80,max_hp:80,gold:0,
        deck:(1..=12).map(|instance|CardInstance{ misc:0,instance,id:CardId::Defend,upgrades:0}).collect(),
        relics:vec![RelicId::CharonsAshes],potions:vec![Some(PotionId::Elixir),None,None],relic_counters:Default::default()};
    let mut run=Run::from_authored_combat(RunConfig{character:Character::Ironclad,ascension:0},start,
        RunRng::new([29;32],Partition::Test,0)).unwrap();
    run.enter_combat(EncounterId::JawWorm).unwrap();
    assert!(run.state.potions.is_empty(),"Run relinquishes the inventory to the live combat owner");
    assert_eq!(run.state.combat.as_ref().unwrap().item_context().unwrap().potions(),&[Some(PotionId::Elixir),None,None]);

    run.choose_combat(CombatAction::UsePotion{slot:0,target:None}).unwrap();
    let selection=run.state.combat.as_ref().unwrap().public_full().unwrap().selection.unwrap();
    assert_eq!(selection.parent,combat::SelectionParent::Potion{slot:0,id:PotionId::Elixir});
    let selected=selection.candidates[0].instance.0;
    run.choose_combat(CombatAction::SelectCard{instance:selected}).unwrap();
    run.choose_combat(CombatAction::ConfirmSelection).unwrap();
    let live=run.state.combat.as_ref().unwrap();
    assert_eq!(live.item_context().unwrap().potions(),&[None,None,None]);
    assert_eq!(live.public_full().unwrap().exhaust.iter().filter(|card|card.instance.0==selected).count(),1);
    assert!(run.state.deck.iter().any(|card|card.instance==selected),"temporary exhaust does not remove the permanent acquired card");

    let combat=run.state.combat.as_mut().unwrap();
    combat.enqueue_native_effects(vec![Effect{source:Source::None,target:Some(Actor::Player),kind:EffectKind::Damage(DamagePayload::HpLoss(999))}],None).unwrap();
    let settled=run.settle_combat().unwrap();
    assert_eq!(settled.terminal,Some(RunTerminal::Death));
    assert_eq!(run.state.potions,vec![None,None,None],"settlement returns the exact consumed belt state");
    assert_eq!(run.state.relics,vec![RelicId::CharonsAshes]);
    assert!(run.state.combat.is_none());
    assert_eq!(run.state.combat_progress.settled,1);
}
