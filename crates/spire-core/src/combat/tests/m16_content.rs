use super::super::*;
use super::full_runtime::entered_combat;
use crate::content::{CardId::*,CardType,Catalog,Character};
#[test]
fn m16_all_29_base_upgraded_cards_have_real_uses_values_costs_and_dispositions(){
    let rows=[(StrikeBlue,1,1),(DefendBlue,1,1),(Zap,1,0),(Dualcast,1,0),(BallLightning,1,1),(ColdSnap,1,1),(Coolheaded,1,1),(Barrage,1,1),(CompileDriver,1,1),(Darkness,1,1),(Fusion,2,1),(Capacitor,1,1),(Consume,2,2),(Defragment,1,1),(BiasedCognition,1,1),(Loop,1,1),(Electrodynamics,2,2),(Bullseye,1,1),(Recursion,1,0),(Fission,0,0),(StaticDischarge,1,1),(Storm,1,1),(Heatsinks,1,1),(Amplify,1,1),(EchoForm,3,3),(GeneticAlgorithm,1,1),(Claw,0,0),(SteamBarrier,0,0),(Streamline,2,2)];
    assert_eq!(crate::content::defect::M16_DEFECT_CARDS.len(),29);
    for (id,base,upgraded) in rows {for (up,cost) in [(0,base),(1,upgraded)]{
        let mut combat=entered_combat(20,20, |_|{});combat.state.character=CharacterState::for_character(Character::Defect);combat.state.silent.source_character=Some(Character::Defect);
        combat.state.enemies[0].max_hp=999;combat.state.enemies[0].hp=999;
        combat.state.character.orb_slots=vec![Some(OrbState{kind:OrbKind::Dark,dark_damage:Some(12)}),Some(OrbKind::Frost.into()),Some(OrbKind::Plasma.into())];
        let instance=combat.state.piles.hand[0].full().unwrap().instance;
        *combat.state.piles.hand[0].full_mut().unwrap()=permanent_live_card(id,instance,up);
        let definition=Catalog::standard().card(id).unwrap();assert!(!definition.definition_only,"{id:?}");
        let before=combat.public_full().unwrap();let card=before.hand.iter().find(|c|c.instance==instance).unwrap();
        assert_eq!(card.effective_cost,Some(cost),"{id:?}+{up}");assert!(card.values.is_some(),"{id:?}+{up}");
        let target=(definition.card_type==CardType::Attack).then_some(TargetId(1));
        combat.step_full(&CombatAction::PlayCard{instance:instance.0,target}).unwrap();
        assert_eq!(combat.state.player.energy,20-cost+if id==Fission{if up>0{5}else{3}}else if id==Electrodynamics&&up>0{2}else{0},"{id:?}+{up}");
        let present=|pile:&Vec<super::super::model::OwnedCard>|pile.iter().any(|c|c.full().is_some_and(|c|c.instance==instance));
        if definition.card_type==CardType::Power{assert!(present(&combat.state.played_powers),"{id:?}+{up}");}
        else if matches!(id,Fission|GeneticAlgorithm){assert!(present(&combat.state.piles.exhaust),"{id:?}+{up}");}
        else{assert!(present(&combat.state.piles.discard),"{id:?}+{up}");}
        if definition.card_type==CardType::Attack{assert!(combat.state.enemies[0].hp<999,"{id:?}+{up}");}
    }}
}
