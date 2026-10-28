use crate::{run::*,content::{CardId,Character,EncounterId,PotionId,RelicId},combat::{CombatAction,rng::Partition}};
#[test]
fn m16_defect_item_use_and_reset(){
    for bark in [false,true]{
        let relics=if bark{vec![RelicId::DataDisk,RelicId::SacredBark]}else{vec![RelicId::DataDisk]};
        let start=AuthoredRunStart{hp:75,max_hp:75,gold:99,deck:(1..=4).map(|instance|CardInstance{id:CardId::Claw,instance,upgrades:1,misc:0}).collect(),relics,
            potions:vec![Some(PotionId::FocusPotion),Some(PotionId::PotionOfCapacity),None],relic_counters:Default::default()};
        let mut run=Run::from_m16_defect_combat(M16DefectCombatStart::Supplied(start),RunRng::new([7;32],Partition::Test,0)).unwrap();
        run.enter_combat(EncounterId::Cultist).unwrap();
        assert_eq!(run.decision().unwrap().combat.unwrap().character.focus,1,"Data Disk without an occupied orb");
        run.choose_combat(CombatAction::UsePotion{slot:0,target:None}).unwrap();
        run.choose_combat(CombatAction::UsePotion{slot:1,target:None}).unwrap();
        let d=run.decision().unwrap();assert_eq!(d.player.potions,vec![None;3]);
        let c=d.combat.unwrap();assert_eq!(c.character.focus,if bark{5}else{3});assert_eq!(c.character.orbs.len(),if bark{7}else{5});
        assert!(run.choose_combat(CombatAction::UsePotion{slot:0,target:None}).is_err());
        for _ in 0..24 {
            let d=run.decision().unwrap();if d.combat.is_none(){break;}
            let action=d.candidates.iter().find(|c|matches!(c.action,GameAction::Combat(CombatAction::PlayCard{..})))
                .or_else(||d.candidates.iter().find(|c|matches!(c.action,GameAction::Combat(CombatAction::EndTurn)))).unwrap().action.clone();
            run.step(&action).unwrap();
        }
        assert_eq!(run.decision().unwrap().last_combat.unwrap().outcome,crate::combat::Outcome::Victory);
        run.enter_combat(EncounterId::Cultist).unwrap();
        let next=run.decision().unwrap();let c=next.combat.unwrap();
        assert_eq!(c.character.focus,1);assert_eq!(c.character.orbs.len(),3);
        assert_eq!(next.player.potions,vec![None;3]);
        assert!(c.hand.iter().all(|card|card.values.as_ref().unwrap().base_damage==Some(5)),"Claw combat growth resets");
        assert_eq!(crate::content::Catalog::standard().potion_eligible_for(PotionId::FocusPotion,Character::Ironclad),Some(false));
    }
}

#[test]
fn m16_defect_potion_accepted_prefix_fault_keeps_consumption(){
    use crate::items::*;
    let mut combat=crate::combat::tests::m16_support::ready_fixture();
    // Test-only owned fixture installs the exact context before public preparation.
    crate::combat::tests::m16_support::set_defect_items(&mut combat,CombatItemContext::new(vec![RelicId::ToyOrnithopter],vec![Some(PotionId::FocusPotion),None,None]));
    let context=combat.item_context().unwrap().clone();let prepared=prepare_combat_potion(&combat,0,None,false).unwrap();
    let items=context.with_potions(prepared.potions);
    crate::combat::tests::m16_support::guarded_potion_continuation(&mut combat,prepared.effects,items);
    assert_eq!(combat.item_context().unwrap().potions(),&[None;3]);
    let frozen=combat.clone();assert!(combat.step_full(&CombatAction::UsePotion{slot:0,target:None}).is_err());assert_eq!(combat,frozen);
}
