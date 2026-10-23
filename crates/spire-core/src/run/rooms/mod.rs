//! Explicit authored room prefix; never a substitute for standard random pools.
use super::*;
use crate::content::{CardId,PotionId,RelicId,EventId};
mod profile;
mod neow;
pub(super) mod rewards;
mod shop;
mod events;
mod rest;
pub(super) mod selection;
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum RoomScenario { M4Act1Prefix }
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum RoomViewKind { Neow,Map,Combat,Rewards,CardReward,Shop,Campfire,Event(EventId),Treasure,CheckpointComplete }
#[derive(Clone,Debug,PartialEq,Eq)]
pub enum RoomChoiceKind { NeowMaxHp{amount:u32},NeowLament,Gold(u32),CardReward,Card{id:CardId,upgrades:u32},Potion(PotionId),Relic(RelicId),Sapphire,OpenChest,Rest{heal:u32},Smith,Recall,Purge,ShrineUpgrade,ShrineRemove,Leave,Proceed,Skip }
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum RoomChoiceStatus { Offered,Claimed,Skipped,Disabled }
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct RoomChoiceView {pub id:ChoiceId,pub kind:RoomChoiceKind,pub price:Option<u32>,pub status:RoomChoiceStatus}
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct RoomView {pub kind:RoomViewKind,pub choices:Vec<RoomChoiceView>}
#[derive(Clone,Debug,PartialEq,Eq)]
pub(crate) struct RoomLoop {pub screen:RoomView,pub purge_cost:u32,pub reward_parent:Option<RoomView>}

pub(super) fn public(run:&Run)->Result<RoomView,RunError>{
    let mut screen=run.state.room_loop.as_ref().ok_or(RunError::InvalidAction)?.screen.clone();
    for row in &mut screen.choices {if row.status==RoomChoiceStatus::Offered {
        if row.price.is_some_and(|p|p>run.state.gold)||matches!(row.kind,RoomChoiceKind::Potion(_))&&!run.state.potions.contains(&None){row.status=RoomChoiceStatus::Disabled;}
        if matches!(row.kind,RoomChoiceKind::Smith|RoomChoiceKind::ShrineUpgrade)&&!run.state.deck.iter().any(|c|c.upgrades==0&&mutations::supported(c.id)){row.status=RoomChoiceStatus::Disabled;}
        if matches!(row.kind,RoomChoiceKind::Recall)&&run.state.keys[0]{row.status=RoomChoiceStatus::Disabled;}
        if matches!(row.kind,RoomChoiceKind::Rest{..}){row.kind=RoomChoiceKind::Rest{heal:(run.state.max_hp as f32*0.3_f32) as u32};}
        if run.state.pending.is_some(){row.status=RoomChoiceStatus::Disabled;}
    }}Ok(screen)
}
pub(super) fn candidates(run:&Run)->Result<Vec<Candidate>,RunError>{
    let view=public(run)?;
    if let Some(pending)=&run.state.pending {
        let mut candidates=run.candidates.clone();
        if matches!(pending.parent,ContinuationTarget::Room{operation:RoomSelectionOperation::ShopPurge{..}|RoomSelectionOperation::Smith,..}){
            candidates.push(Candidate{id:ChoiceId(u64::MAX),action:GameAction::Cancel,label:"Cancel selection",neow_offer:None,object:None});
        }return Ok(candidates)
    }
    if view.kind==RoomViewKind::Neow{return Ok(run.candidates.clone())}
    let mut candidates:Vec<_>=if view.kind==RoomViewKind::Map {
        run.state.map.as_ref().unwrap().nodes.iter().filter(|n|world::preflight_move(&run.state,n.id).is_ok()).map(|n|Candidate{
            id:ChoiceId(n.id.0),action:GameAction::Move(n.id),label:"Move",neow_offer:None,object:None}).collect()
    }else{view.choices.iter().filter(|c|c.status==RoomChoiceStatus::Offered).map(|c|Candidate{id:c.id,action:GameAction::Choose(c.id),label:"Room choice",neow_offer:None,object:None}).collect()};
    for (slot,potion) in run.state.potions.iter().enumerate(){if potion.is_some(){
        candidates.push(Candidate{id:ChoiceId(u64::MAX-slot as u64),action:GameAction::DiscardPotion{slot:slot as u32},label:"Discard potion",neow_offer:None,object:None});
        if crate::items::prepare_run_potion(&run.state,slot,false,false).is_ok(){candidates.push(Candidate{id:ChoiceId(u64::MAX-slot as u64),action:GameAction::UsePotion{slot:slot as u32,target:None},label:"Use potion",neow_offer:None,object:None});}
    }}
    Ok(candidates)
}
pub(super) fn step(run:&mut Run,action:&GameAction)->Result<RunTransition,RunError>{
    if !candidates(run)?.iter().any(|c|&c.action==action){return Err(RunError::InvalidAction)}
    match action {
        GameAction::Choose(id) if public(run)?.kind==RoomViewKind::Neow=>neow::choose(run,*id),
        GameAction::Move(id)=>profile::enter(run,*id),
        GameAction::DiscardPotion{slot}=>{run.state.potions[*slot as usize]=None;Ok(empty_transition())},
        GameAction::UsePotion{slot,target:None}=>{crate::items::use_run_potion(&mut run.state,*slot as usize,false,false)?;Ok(empty_transition())},
        GameAction::Select(id)=>run.select(*id),
        GameAction::Cancel=>{run.state.pending=None;run.candidates.clear();Ok(empty_transition())},
        GameAction::Choose(id)=>match public(run)?.kind {
            RoomViewKind::Shop=>shop::choose(run,*id),RoomViewKind::Campfire=>rest::choose(run,*id),RoomViewKind::Event(_)=>events::choose(run,*id),_=>rewards::choose(run,*id),
        },
        _=>Err(RunError::Unsupported("room handler pending")),
    }
}
pub(super) fn empty_transition()->RunTransition{RunTransition{reward:0,terminal:None,committed_events:vec![]}}
pub(super) fn token(next:&mut u64)->Result<ChoiceId,RunError>{let id=*next;*next=next.checked_add(1).ok_or(RunError::Unsupported("choice identity exhausted"))?;Ok(ChoiceId(id))}
pub(super) fn view(next:&mut u64,kind:RoomViewKind,rows:Vec<(RoomChoiceKind,Option<u32>)>)->Result<RoomView,RunError>{
    let mut staged=*next;let choices=rows.into_iter().map(|(kind,price)|Ok(RoomChoiceView{id:token(&mut staged)?,kind,price,status:RoomChoiceStatus::Offered})).collect::<Result<_,RunError>>()?;
    *next=staged;Ok(RoomView{kind,choices})
}
pub(super) fn finish(run:&mut Run)->Result<RunTransition,RunError>{
    if run.state.deck.is_empty(){return run.commit_terminal(RunTerminal::EmptyDeck)}
    let kind=if run.state.map.as_ref().and_then(|m|m.current)==Some(NodeId(59)){RoomViewKind::CheckpointComplete}else{RoomViewKind::Map};
    run.state.room_loop.as_mut().unwrap().screen=RoomView{kind,choices:vec![]};run.candidates.clear();Ok(empty_transition())
}
impl Run {
    pub fn new_room_scenario(config:RunConfig,profile:CreationProfile,rng:RunRng,_scenario:RoomScenario)->Result<Self,RunError>{
        if config.character!=crate::content::Character::Ironclad||config.ascension!=0||profile.neow_eligible{return Err(RunError::Unsupported("M4 requires Ironclad A0 short Neow"))}
        let mut run=Self::new(config,profile,rng)?;
        let choices=run.candidates.iter().map(|c|RoomChoiceView{id:c.id,kind:match c.neow_offer.as_ref().unwrap().reward{
            NeowReward::Lament=>RoomChoiceKind::NeowLament,_=>RoomChoiceKind::NeowMaxHp{amount:(run.state.max_hp as f32*0.1_f32) as u32}},price:None,status:RoomChoiceStatus::Offered}).collect();
        run.state.room_loop=Some(RoomLoop{screen:RoomView{kind:RoomViewKind::Neow,choices},purge_cost:75,reward_parent:None});Ok(run)
    }
}
