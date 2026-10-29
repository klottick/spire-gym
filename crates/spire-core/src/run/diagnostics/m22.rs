//! Fixed, default-off constructed fixtures. No ordinary traversal claim or state importer.
use super::super::*;
use crate::{combat::{CombatAction,Outcome,rng::Partition},content::{CardId,Character,EncounterId,PotionId,RelicId}};
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum M22DiagnosticBoss{Guardian,Hexaghost,SlimeBoss}
impl M22DiagnosticBoss{pub fn encounter(self)->EncounterId{match self{Self::Guardian=>EncounterId::Guardian,Self::Hexaghost=>EncounterId::Hexaghost,Self::SlimeBoss=>EncounterId::SlimeBoss}}}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum M22DiagnosticBranch{HolyWater,VioletLotus,SacredBark,EmptyCage,SlaversCollar,Skip}
impl M22DiagnosticBranch{fn relic(self)->Option<RelicId>{Some(match self{Self::HolyWater=>RelicId::HolyWater,Self::VioletLotus=>RelicId::VioletLotus,Self::SacredBark=>RelicId::SacredBark,Self::EmptyCage=>RelicId::EmptyCage,Self::SlaversCollar=>RelicId::SlaversCollar,Self::Skip=>return None})}}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum M22Diagnostic{
    BossRewards{boss:M22DiagnosticBoss},BossRelicMenu{boss:M22DiagnosticBoss},
    BossChild{boss:M22DiagnosticBoss,branch:M22DiagnosticBranch},
    Act2Ready{boss:M22DiagnosticBoss,branch:M22DiagnosticBranch},
    ScryChoice,MeditateChoice,StanceChoice,BeforeStartupFault,
    TransitionResetFault,TransitionHealFault,
}
fn random()->RunRng{let mut root=[0;32];root[..8].copy_from_slice(&7u64.to_le_bytes());RunRng::new(root,Partition::Test,0)}
fn creation(boss:M22DiagnosticBoss)->CreationProfile{let mut p=CreationProfile::experienced();p.neow_eligible=false;p.seen_bosses[0]=match boss{M22DiagnosticBoss::Guardian=>vec!["Hexaghost","SlimeBoss"],M22DiagnosticBoss::Hexaghost=>vec!["Guardian","SlimeBoss"],M22DiagnosticBoss::SlimeBoss=>vec!["Guardian","Hexaghost"]};p}
fn canonical(boss:M22DiagnosticBoss)->Result<Run,RunError>{Run::new_room_scenario(RunConfig{character:Character::Watcher,ascension:0},creation(boss),random(),RoomScenario::M22WatcherAct1)}
fn choose(run:&mut Run,kind:RoomChoiceKind)->Result<(),RunError>{let id=run.decision()?.room.ok_or(RunError::InvalidAction)?.choices.iter().find(|r|r.kind==kind&&r.status==RoomChoiceStatus::Offered).ok_or(RunError::InvalidAction)?.id;run.step(&GameAction::Choose(id))?;Ok(())}
fn source_map(run:&mut Run,boss:M22DiagnosticBoss){
    let node=NodeId(0xF220_0016);run.state.floor=16;run.state.room="M22ConstructedBossSource";
    run.state.map=Some(RunMap{act:1,boss_icon:Some(boss.encounter()),current:Some(node),nodes:vec![MapNode{id:node,floor:16,label:"Boss",edges:vec![],x:0,y:15,kind:MapNodeKind::Boss,hidden_kind:None,visited:true,revealed:true,encounter:Some(boss.encounter()),burning:false}],..RunMap::default()});
    let record=CombatRecord{visit:0,encounter:boss.encounter(),outcome:Outcome::Victory};
    run.state.combat_progress=CombatProgress{entered:1,settled:1};run.state.last_combat=Some(record.clone());
    run.state.history.events.push(CommittedEvent::CombatFinished(record));
    run.state.history.rooms.push(VisitedRoom{act:1,floor:16,label:"M22ConstructedBossSource"});
}
fn boss_rewards(boss:M22DiagnosticBoss)->Result<Run,RunError>{
    let mut r=canonical(boss)?;choose(&mut r,RoomChoiceKind::NeowMaxHp{amount:7})?;source_map(&mut r,boss);
    // Explicitly constructed source record, not a forged played-combat receipt.
    let screen=rooms::view(&mut r.next_choice,RoomViewKind::Rewards,vec![(RoomChoiceKind::CardReward,None),(RoomChoiceKind::Proceed,None)])?;
    let mut cards=rooms::m22_content::reward_cards(true,0).into_iter().map(|id|(RoomChoiceKind::Card{id,upgrades:0},None)).collect::<Vec<_>>();cards.push((RoomChoiceKind::Skip,None));
    let captured=rooms::rewards::CapturedRewards{cards:rooms::view(&mut r.next_choice,RoomViewKind::CardReward,cards)?,return_to:rooms::rewards::RewardReturn::BossTreasure};
    let room=r.state.room_loop.as_mut().unwrap();room.screen=screen;room.active_rewards=Some(captured);r.candidates.clear();Ok(r)
}
fn boss_menu(boss:M22DiagnosticBoss)->Result<Run,RunError>{let mut r=boss_rewards(boss)?;choose(&mut r,RoomChoiceKind::Proceed)?;choose(&mut r,RoomChoiceKind::OpenChest)?;Ok(r)}
fn boss_child(boss:M22DiagnosticBoss,branch:M22DiagnosticBranch)->Result<Run,RunError>{
    let mut r=boss_menu(boss)?;r.state.hp=31;r.state.potions[0]=Some(PotionId::BottledMiracle);
    if let Some(id)=branch.relic(){choose(&mut r,RoomChoiceKind::Relic(id))?;}else{choose(&mut r,RoomChoiceKind::Skip)?;}
    Ok(r)
}
fn play_kind(r:&mut Run,kind:CardId)->Result<(),RunError>{
    let d=r.decision()?;let c=d.combat.as_ref().ok_or(RunError::InvalidAction)?;
    let action=d.candidates.iter().find(|row|matches!(row.action,GameAction::Combat(CombatAction::PlayCard{instance,..}) if c.hand.iter().any(|card|card.instance.0==instance&&card.id==kind))).ok_or(RunError::InvalidAction)?.action.clone();
    r.step(&action)?;Ok(())
}
fn combat_fixture(fixture:M22Diagnostic)->Result<Run,RunError>{
    let mut r=canonical(M22DiagnosticBoss::Guardian)?;choose(&mut r,RoomChoiceKind::NeowMaxHp{amount:7})?;
    if fixture==M22Diagnostic::ScryChoice{r.state.deck.iter_mut().for_each(|c|c.id=CardId::ThirdEye);}
    if fixture==M22Diagnostic::MeditateChoice{r.state.deck.iter_mut().for_each(|c|c.id=CardId::Meditate);}
    if fixture==M22Diagnostic::StanceChoice{r.state.potions[0]=Some(PotionId::StancePotion);}
    if fixture==M22Diagnostic::BeforeStartupFault{
        crate::items::acquire_relic(&mut r.state,RelicId::HolyWater)?;
        r.state.deck.last_mut().unwrap().instance=u64::MAX-1;r.state.next_instance=u64::MAX;
        return Ok(r)
    }
    let id=r.decision()?.candidates.iter().find_map(|c|if let GameAction::Move(id)=c.action{Some(id)}else{None}).ok_or(RunError::InvalidAction)?;
    r.step(&GameAction::Move(id))?;
    match fixture{
        M22Diagnostic::ScryChoice=>{play_kind(&mut r,CardId::ThirdEye)?;},
        M22Diagnostic::MeditateChoice=>{r.step(&GameAction::Combat(CombatAction::EndTurn))?;play_kind(&mut r,CardId::Meditate)?;},
        M22Diagnostic::StanceChoice=>{r.step(&GameAction::Combat(CombatAction::UsePotion{slot:0,target:None}))?;},_=>return Err(RunError::InvalidAction),
    }Ok(r)
}
/// Finite typed diagnostic stages only; callers must label the constructed provenance.
pub fn m22_diagnostic(fixture:M22Diagnostic)->Result<Run,RunError>{match fixture{
    M22Diagnostic::BossRewards{boss}=>boss_rewards(boss),M22Diagnostic::BossRelicMenu{boss}=>boss_menu(boss),
    M22Diagnostic::BossChild{boss,branch}=>boss_child(boss,branch),
    M22Diagnostic::Act2Ready{boss,branch}=>{let mut r=boss_child(boss,branch)?;while r.state.pending.is_some(){let id=r.decision()?.candidates.iter().find_map(|c|if let GameAction::Select(id)=c.action{Some(id)}else{None}).ok_or(RunError::InvalidAction)?;r.step(&GameAction::Select(id))?;}choose(&mut r,if branch==M22DiagnosticBranch::Skip{RoomChoiceKind::ConfirmSkip}else{RoomChoiceKind::Proceed})?;Ok(r)},
    M22Diagnostic::TransitionResetFault|M22Diagnostic::TransitionHealFault=>{
        let mut r=boss_child(M22DiagnosticBoss::Guardian,M22DiagnosticBranch::HolyWater)?;
        r.state.room_loop.as_mut().unwrap().transition=Some(ActTransition{from:1,to:if fixture==M22Diagnostic::TransitionResetFault{2}else{3},floor:17,reset_applied:false,heal_applied:false,initialized:false});
        if fixture==M22Diagnostic::TransitionResetFault{r.state.max_hp=0;}
        let result=rooms::transition::drive(&mut r);if !matches!(result,Err(RunError::Faulted(_))){return Err(RunError::InvalidAction)}Ok(r)
    },other=>combat_fixture(other),
}}

/// The exact preapproved full-HP supplied boss resources, before actual entry.
/// No normal Neow, earned path, keys or prior victory is implied.
pub fn m22_supplied_boss(boss:M22DiagnosticBoss)->Result<Run,RunError>{
    let config=RunConfig{character:Character::Watcher,ascension:0};let rng=random();let template=canonical(boss)?;
    let resources=AuthoredRunStart{hp:72,max_hp:72,gold:99,deck:[CardId::Eruption,CardId::Vigilance,CardId::EmptyFist,CardId::CarveReality].into_iter().flat_map(|id|[id;4]).enumerate().map(|(i,id)|CardInstance{instance:i as u64+1,id,upgrades:1,misc:0}).collect(),relics:vec![RelicId::PureWater,RelicId::VioletLotus,RelicId::Vajra,RelicId::OddlySmoothStone],potions:vec![None;3],relic_counters:Default::default()};
    // Reuse accepted complete resource validation, keeping the approved ID/order bytes.
    M21WatcherCombatStart::Supplied(resources.clone()).normalized()?;
    let mut r=Run::from_validated_combat_resources(config,resources,rng,17);r.state.authored_combat=false;r.state.room_loop=template.state.room_loop;
    r.state.floor=15;r.state.room="M22SuppliedBossEntry";let previous=NodeId(0xF220_0015);let node=NodeId(0xF220_0016);
    r.state.map=Some(RunMap{act:1,boss_icon:Some(boss.encounter()),current:Some(previous),nodes:vec![MapNode{id:previous,floor:15,label:"Rest",edges:vec![node],x:0,y:14,kind:MapNodeKind::Rest,hidden_kind:None,visited:true,revealed:true,encounter:None,burning:false},MapNode{id:node,floor:16,label:"Boss",edges:vec![],x:0,y:15,kind:MapNodeKind::Boss,hidden_kind:None,visited:false,revealed:true,encounter:None,burning:false}],..RunMap::default()});
    // Shared Act1 metadata selects the requested actual boss; no later icon patch.
    r.state.room_loop.as_mut().unwrap().act1=Some(rooms::act1::Act1Progress{boss_order:vec![boss.encounter();3],question_visits:0,shop_visits:0,elite_relic_cursor:0,combat_visits:0});
    r.state.room_loop.as_mut().unwrap().screen=RoomView{kind:RoomViewKind::Map,choices:vec![]};r.candidates.clear();Ok(r)
}

/// Privileged, read-only diagnostic projection; never a normal policy observation.
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct M22TransitionCheckpoint{pub from:u8,pub to:u8,pub floor:u32,pub reset_applied:bool,pub heal_applied:bool,pub initialized:bool}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum M22DiagnosticZone{Hand,Draw,Discard,Exhaust,InPlay,PlayFrame,QueuedAutoplay,Stasis,PlayedPower,Inactive}
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct M22DiagnosticCard{pub zone:M22DiagnosticZone,pub instance:crate::combat::CardInstanceId,pub id:CardId,pub origin:crate::combat::CombatCardRef}
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct M22CreationCheckpoint{pub id:CardId,pub count:u32,pub next:u32,pub template_upgrades:u32}
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct M22CombatCheckpoint{
    pub cards:Vec<M22DiagnosticCard>,pub draw_count:usize,
    pub inventory_relics:Vec<RelicId>,pub inventory_potions:Vec<Option<PotionId>>,
    pub inventory_counters:std::collections::BTreeMap<RelicId,i32>,
    pub current_creation:Option<M22CreationCheckpoint>,pub pending_opening_hand:Option<usize>,
    pub fault:Option<crate::combat::FaultRecord>,
}
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct M22DiagnosticCheckpoint{
    pub transition:Option<M22TransitionCheckpoint>,pub combat:Option<M22CombatCheckpoint>,
    pub run_fault:Option<RunFault>,pub combat_progress:CombatProgress,
    pub run_relics:Vec<RelicId>,pub run_potions:Vec<Option<PotionId>>,
    pub permanent_cards:Vec<CardInstance>,pub rng:RunRng,pub next_choice:u64,pub next_instance:u64,
}
/// Reads actual owners even after a fault. It cannot install or resume state.
pub fn m22_diagnostic_checkpoint(run:&Run)->Result<M22DiagnosticCheckpoint,RunError>{
    if !rooms::is_m22(&run.state){return Err(RunError::InvalidAction)}
    Ok(M22DiagnosticCheckpoint{
        transition:run.state.room_loop.as_ref().and_then(|r|r.transition.as_ref()).map(|f|M22TransitionCheckpoint{from:f.from,to:f.to,floor:f.floor,reset_applied:f.reset_applied,heal_applied:f.heal_applied,initialized:f.initialized}),
        combat:run.state.combat.as_ref().map(crate::combat::Combat::m22_diagnostic_checkpoint),
        run_fault:run.state.fault.clone(),combat_progress:run.state.combat_progress,
        run_relics:run.state.relics.clone(),run_potions:run.state.potions.clone(),permanent_cards:run.state.deck.clone(),rng:run.state.rng.clone(),next_choice:run.next_choice,next_instance:run.state.next_instance,
    })
}
