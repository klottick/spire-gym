use super::super::*;
use crate::content::CardId;
use super::full_runtime::entered_combat;

#[test]
fn m12_silent_base_and_upgraded_public_values_follow_actual_card_state(){
    use CardId::*;
    // Literal source-qualified printed quantities. Accuracy's mutable Shiv
    // base is tested separately through actual play; no payload recomputation.
    for (id,damage,block,magic) in [
        (Neutralize,[Some(3),Some(4)],[None,None],[Some(1),Some(2)]),
        (Survivor,[None,None],[Some(8),Some(11)],[Some(1),Some(1)]),
        (PoisonedStab,[Some(6),Some(8)],[None,None],[Some(3),Some(4)]),
        (Backflip,[None,None],[Some(5),Some(8)],[Some(2),Some(2)]),
        (BladeDance,[None,None],[None,None],[Some(3),Some(4)]),
        (DeadlyPoison,[None,None],[None,None],[Some(5),Some(7)]),
        (CloakAndDagger,[None,None],[Some(6),Some(6)],[Some(1),Some(2)]),
        (DaggerThrow,[Some(9),Some(12)],[None,None],[Some(1),Some(1)]),
        (NoxiousFumes,[None,None],[None,None],[Some(2),Some(3)]),
        (Accuracy,[None,None],[None,None],[Some(4),Some(6)]),
        (WellLaidPlans,[None,None],[None,None],[Some(1),Some(2)]),
        (QuickSlash,[Some(8),Some(12)],[None,None],[Some(1),Some(1)]),
        (Reflex,[None,None],[None,None],[Some(2),Some(3)]),
        (Catalyst,[None,None],[None,None],[Some(2),Some(3)]),
        (Deflect,[None,None],[Some(4),Some(7)],[None,None]),
        (Tactician,[None,None],[None,None],[Some(1),Some(2)]),
        (Nightmare,[None,None],[None,None],[Some(3),Some(3)]),
    ]{
        for up in 0..=1usize{
            let combat=entered_combat(3,3,|card|*card=permanent_live_card(id,card.instance,up as u32));
            let view=combat.public_full().unwrap();let card=view.hand.iter().find(|card|card.id==id).unwrap();
            let value=card.values.as_ref().unwrap_or_else(||panic!("{id:?}+{up} has unknown public values"));
            assert_eq!((value.base_damage,value.base_block,value.magic_number),(damage[up],block[up],magic[up]));
            assert_eq!(value.cost_kind,if matches!(id,Reflex|Tactician){CardCostKind::Unplayable}else{CardCostKind::Fixed});
            assert_eq!(value.exhaust_on_finish,matches!(id,Catalyst|Nightmare));
        }
    }
}

#[test]
fn m12_actual_accuracy_shivs_publish_mutated_damage_and_exhaust_disposition(){
    let mut combat=entered_combat(3,3,|card|*card=permanent_live_card(CardId::BladeDance,card.instance,0));
    let effect=Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::ApplyFullPower{id:FullPowerId("accuracy"),amount:4}};
    super::super::powers::apply_effect(&mut combat.state,&effect).unwrap();
    let blade=combat.public_full().unwrap().hand.iter().find(|c|c.id==CardId::BladeDance).unwrap().instance.0;
    combat.step_full(&CombatAction::PlayCard{instance:blade,target:None}).unwrap();
    let view=combat.public_full().unwrap();let shivs=view.hand.iter().filter(|c|c.id==CardId::Shiv).collect::<Vec<_>>();assert_eq!(shivs.len(),3);
    for shiv in &shivs{let values=shiv.values.as_ref().expect("generated Shiv exposes its actual mutable values");assert_eq!(values.base_damage,Some(8));assert!(values.exhaust_on_finish);}
    let instance=shivs[0].instance.0;let before=view.enemies[0].hp;
    combat.step_full(&CombatAction::PlayCard{instance,target:Some(TargetId(1))}).unwrap();
    let after=combat.public_full().unwrap();assert_eq!(after.enemies[0].hp,before-8);assert_eq!(after.exhaust.iter().find(|c|c.instance.0==instance).unwrap().values.as_ref().unwrap().base_damage,Some(8));
}
