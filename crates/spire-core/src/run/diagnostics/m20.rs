use super::super::*;
use crate::{combat::{Outcome,rng::Partition},content::{CardId,Character,EncounterId,PotionId,RelicId}};
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum M20DiagnosticLoadout { Defect,Strong }
const DECK:&[CardId]=&[CardId::Defragment,CardId::Defragment,CardId::Capacitor,CardId::Loop,
    CardId::BiasedCognition,CardId::Electrodynamics,CardId::EchoForm,CardId::Glacier,CardId::Glacier,
    CardId::Coolheaded,CardId::Coolheaded,CardId::BallLightning,CardId::DoomAndGloom,CardId::Consume,
    CardId::Fission,CardId::Buffer,CardId::SelfRepair,CardId::Seek];
const RELICS:&[RelicId]=&[RelicId::CrackedCore,RelicId::DataDisk,RelicId::GoldPlatedCables,RelicId::Mango,
    RelicId::BagOfPreparation,RelicId::Anchor,RelicId::OddlySmoothStone,RelicId::SlaversCollar];
/// Return the frozen floor50 supplied source BEFORE its real dialog4/Act4 door.
/// Synthetic keys/prior Victory are declared inputs, never earned traversal.
pub fn m20_supplied_act4(loadout:M20DiagnosticLoadout)->Result<Run,RunError>{
    let mut root=[0;32];root[..8].copy_from_slice(&7u64.to_le_bytes());
    let mut profile=CreationProfile::experienced();profile.neow_eligible=false;
    profile.seen_bosses=[vec!["Hexaghost","SlimeBoss"],vec!["Automaton","Collector"],vec!["TimeEater","DonuAndDeca"]];
    let mut run=Run::new_room_scenario(RunConfig{character:Character::Defect,ascension:0},profile,
        RunRng::new(root,Partition::Test,0),RoomScenario::M20DefectHeart)?;
    let ids=if loadout==M20DiagnosticLoadout::Defect{DECK.to_vec()}else{vec![CardId::SearingBlow;10]};
    run.state.deck=ids.into_iter().enumerate().map(|(index,id)|{
        let instance=index as u64+1;let mut live=crate::combat::permanent_live_card(id,crate::combat::CardInstanceId(instance),0);
        for _ in 0..if loadout==M20DiagnosticLoadout::Defect{1}else{15}{crate::combat::cards::ironclad::rules::upgrade(&mut live)?;}
        Ok(CardInstance{instance,id,upgrades:live.upgrades,misc:live.misc})
    }).collect::<Result<Vec<_>,RunError>>()?;
    run.state.next_instance=run.state.deck.len() as u64+1;
    for id in RELICS.iter().skip(1){crate::items::acquire_relic(&mut run.state,*id)?;}
    if loadout==M20DiagnosticLoadout::Strong{run.state.hp=250;run.state.max_hp=250;}
    run.state.gold=200;run.state.potions=vec![Some(PotionId::FocusPotion),Some(PotionId::PotionOfCapacity),Some(PotionId::WeakPotion)];
    run.state.keys=[true;3];run.state.act=3;run.state.floor=50;run.state.room="M20SuppliedAct3Source";
    let node=NodeId(0xF200_0050);let record=CombatRecord{visit:0,encounter:EncounterId::AwakenedOne,outcome:Outcome::Victory};
    run.state.combat_progress=CombatProgress{entered:1,settled:1};run.state.last_combat=Some(record.clone());
    run.state.history=RunHistory{rooms:vec![VisitedRoom{act:3,floor:50,label:"M20SuppliedAct3Source"}],events:vec![CommittedEvent::CombatFinished(record.clone())]};
    run.state.reward_ledger=Default::default();
    run.state.map=Some(RunMap{act:3,boss_icon:Some(EncounterId::AwakenedOne),current:Some(node),nodes:vec![MapNode{id:node,floor:50,label:"Boss",edges:vec![],x:0,y:15,kind:MapNodeKind::Boss,hidden_kind:None,visited:true,revealed:true,encounter:Some(EncounterId::AwakenedOne),burning:false}],..RunMap::default()});
    let room=run.state.room_loop.as_mut().unwrap();room.act1=None;room.act2=None;room.act3=None;
    room.screen=rooms::view(&mut run.next_choice,RoomViewKind::Act3Ending,vec![(RoomChoiceKind::Act3Continue{stage:4},None)])?;
    room.act3_end=Some(rooms::act3_end::Act3EndFrame{node,record,phase:rooms::act3_end::Act3EndPhase::Dialog(4),accepted:None});
    run.candidates.clear();Ok(run)
}
