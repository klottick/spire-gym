//! Exact initialization checks only. Prescribed gameplay stays ignored/held.
use super::super::*;
use crate::{content::{CardId,Character,EncounterId,PotionId,RelicId},combat::Outcome};
use diagnostics::{m20_supplied_act4,M20DiagnosticLoadout as L};
#[test]
fn m20_diagnostic_support_both_fixed_starts_are_exact_before_the_real_door(){
    let blue=[CardId::Defragment,CardId::Defragment,CardId::Capacitor,CardId::Loop,CardId::BiasedCognition,CardId::Electrodynamics,CardId::EchoForm,CardId::Glacier,CardId::Glacier,CardId::Coolheaded,CardId::Coolheaded,CardId::BallLightning,CardId::DoomAndGloom,CardId::Consume,CardId::Fission,CardId::Buffer,CardId::SelfRepair,CardId::Seek];
    let relics=vec![RelicId::CrackedCore,RelicId::DataDisk,RelicId::GoldPlatedCables,RelicId::Mango,RelicId::BagOfPreparation,RelicId::Anchor,RelicId::OddlySmoothStone,RelicId::SlaversCollar];
    for loadout in [L::Defect,L::Strong]{
        let r=m20_supplied_act4(loadout).unwrap();let d=r.decision().unwrap();
        assert_eq!(d.content_profile,Some(RoomScenario::M20DefectHeart));assert_eq!(r.state.character,Character::Defect);assert_eq!((r.state.act,r.state.floor,r.state.ascension),(3,50,0));
        assert_eq!((r.state.hp,r.state.max_hp),if loadout==L::Strong{(250,250)}else{(89,89)},"Mango real obtain is inclusive before Strong override");assert_eq!(r.state.gold,200);assert_eq!(r.state.keys,[true;3]);assert_eq!(r.state.relics,relics);assert_eq!(r.state.potions,vec![Some(PotionId::FocusPotion),Some(PotionId::PotionOfCapacity),Some(PotionId::WeakPotion)]);
        let expected=if loadout==L::Strong{vec![CardId::SearingBlow;10]}else{blue.to_vec()};assert_eq!(r.state.deck.iter().map(|card|card.id).collect::<Vec<_>>(),expected);
        for (index,card)in r.state.deck.iter().enumerate(){assert_eq!(card.instance,index as u64+1);assert_eq!(card.upgrades,if loadout==L::Strong{15}else{1});assert_eq!(card.misc,0);}
        assert_eq!(r.state.next_instance,r.state.deck.len()as u64+1);assert!(r.state.combat.is_none()&&r.state.active_combat.is_none()&&r.state.pending.is_none());assert!(r.combat_checkpoint().is_none());
        assert_eq!(r.state.combat_progress,CombatProgress{entered:1,settled:1});assert_eq!(r.state.last_combat,Some(CombatRecord{visit:0,encounter:EncounterId::AwakenedOne,outcome:Outcome::Victory}));
        assert_eq!(d.room.unwrap().choices.iter().map(|row|row.kind.clone()).collect::<Vec<_>>(),vec![RoomChoiceKind::Act3Continue{stage:4}]);assert_eq!(r.public_map().unwrap().act,3);
        assert_eq!(r,m20_supplied_act4(loadout).unwrap(),"initial RNG and every owner are deterministic");assert!(r.state.terminal.is_none());
    }
}
#[test]
fn m20_diagnostic_support_ordinary_profile_has_no_red_obtain_or_supplied_switch(){
    let mut r=super::m20_defect_content::run();let before=r.clone();assert!(mutations::obtain_card(&mut r.state,CardId::SearingBlow,15).is_err());assert_eq!(r,before);
    assert_ne!(r,m20_supplied_act4(L::Defect).unwrap());assert_eq!((r.state.act,r.state.floor),(1,0));assert_eq!(r.state.keys,[false;3]);
}
