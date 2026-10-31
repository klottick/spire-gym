//! Explicit authored room prefix; never a substitute for standard random pools.
use super::*;
use crate::content::{CardId,PotionId,RelicId,EventId};
mod profile;
pub(crate) mod m10_content;
pub(crate) mod m12_content;
pub(crate) mod m13_content;
pub(crate) mod m14_content;
pub(crate) mod m15_content;
pub(crate) mod m17_content;
pub(crate) mod m18_content;
pub(crate) mod m19_content;
pub(crate) mod m20_content;
pub mod m22_content;
pub mod m23_content;
pub mod m24_content;
mod neow;
pub(super) mod act1;
pub(super) mod act2;
pub(super) mod act3;
pub(super) mod act3_end;
pub(super) mod act4;
mod boss;
pub(super) mod transition;
pub(super) mod rewards;
pub(super) mod shop;
pub(super) mod events;
mod rest;
pub(super) mod selection;
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum RoomScenario { M4Act1Prefix, M5Act1Captured, M6Act2Prefix, M7Act2Captured, M8Act3Prefix, M9IroncladAct3, M10IroncladHeart, M12SilentAct1, M13SilentAct2, M14SilentAct3, M15SilentHeart, M17DefectAct1, M18DefectAct2, M19DefectAct3, M20DefectHeart, M22WatcherAct1, M23WatcherAct2, M24WatcherAct3 }
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum RoomViewKind { Neow,Map,Combat,Rewards,CardReward,Shop,Campfire,Event(EventId),Treasure,CheckpointComplete,BossTreasure,BossRelicChoice,BossRelicChild,ProceedWithoutRelic,Act2Ready,Act2PrefixComplete,Act3Ready,Act3PrefixComplete,Act3BossComplete,Act3Ending,Act4Ready,HeartVictory,TrueVictory }
#[derive(Clone,Debug,PartialEq,Eq)]
pub enum RoomChoiceKind { NeowMaxHp{amount:u32},NeowLament,Gold(u32),CardReward,Card{id:CardId,upgrades:u32},Potion(PotionId),Relic(RelicId),Sapphire,Emerald,OpenChest,Rest{heal:u32},Smith,Recall,Purge,ShrineUpgrade,ShrineRemove,AncientElegance,AncientSimplicity,JoustMurderer,JoustOwner,JoustContinue,JoustResult{owner_wins:bool},FallingLose{card_type:crate::content::CardType,instance:Option<u64>},TombGold,TombOfferGold,Leave,Proceed,Skip,ConfirmSkip,CancelSkip,Act3Continue{stage:u8} }
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum RoomChoiceStatus { Offered,Claimed,Skipped,Disabled }
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct RoomChoiceView {pub id:ChoiceId,pub kind:RoomChoiceKind,pub price:Option<u32>,pub status:RoomChoiceStatus}
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct RoomView {pub kind:RoomViewKind,pub choices:Vec<RoomChoiceView>}
#[derive(Clone,Debug,PartialEq,Eq)]
pub(crate) struct RoomLoop {pub scenario:RoomScenario,pub act1:Option<act1::Act1Progress>,pub act2:Option<act2::Act2Progress>,pub act3:Option<act3::Act3Progress>,pub act4:Option<act4::Act4Progress>,pub m10_elite_relic_cursor:usize,pub screen:RoomView,pub purge_cost:u32,pub reward_parent:Option<RoomView>,pub active_rewards:Option<rewards::CapturedRewards>,pub boss:Option<boss::BossProgress>,pub transition:Option<ActTransition>,pub act2_boss_order:Option<Vec<crate::content::EncounterId>>,pub act3_boss_order:Option<Vec<crate::content::EncounterId>>,pub joust:Option<events::JoustProgress>,pub joust_visits:u64,pub act3_event:Option<events::act3::Act3EventFrame>,pub act3_end:Option<act3_end::Act3EndFrame>}

pub(super) fn public(run:&Run)->Result<RoomView,RunError>{
    let mut screen=run.state.room_loop.as_ref().ok_or(RunError::InvalidAction)?.screen.clone();
    for row in &mut screen.choices {if row.status==RoomChoiceStatus::Offered {
        if row.price.is_some_and(|p|p>run.state.gold)||matches!(row.kind,RoomChoiceKind::Potion(_))&&!run.state.potions.contains(&None){row.status=RoomChoiceStatus::Disabled;}
        if matches!(row.kind,RoomChoiceKind::Smith|RoomChoiceKind::ShrineUpgrade)&&!run.state.deck.iter().any(|c|mutations::can_upgrade(&run.state,c)){row.status=RoomChoiceStatus::Disabled;}
        if matches!(row.kind,RoomChoiceKind::Recall)&&run.state.keys[0]{row.status=RoomChoiceStatus::Disabled;}
        if uses_captured_act1(&run.state)&&matches!(row.kind,RoomChoiceKind::Relic(id) if run.state.relics.contains(&id)){row.status=RoomChoiceStatus::Disabled;}
        if matches!(row.kind,RoomChoiceKind::Rest{..}){row.kind=RoomChoiceKind::Rest{heal:(run.state.max_hp as f32*0.3_f32) as u32};}
        if run.state.pending.is_some(){row.status=RoomChoiceStatus::Disabled;}
    }}Ok(screen)
}
pub(super) fn candidates(run:&Run)->Result<Vec<Candidate>,RunError>{
    let view=public(run)?;
    if matches!(view.kind,RoomViewKind::Act2Ready|RoomViewKind::Act2PrefixComplete|RoomViewKind::Act3Ready|RoomViewKind::Act3PrefixComplete|RoomViewKind::Act4Ready){return Ok(vec![])}
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
        if crate::items::can_use_run_potion(&run.state,slot,false,false){candidates.push(Candidate{id:ChoiceId(u64::MAX-slot as u64),action:GameAction::UsePotion{slot:slot as u32,target:None},label:"Use potion",neow_offer:None,object:None});}
    }}
    Ok(candidates)
}
pub(super) fn step(run:&mut Run,action:&GameAction)->Result<RunTransition,RunError>{
    if !candidates(run)?.iter().any(|c|&c.action==action){return Err(RunError::InvalidAction)}
    match action {
        GameAction::Choose(id) if public(run)?.kind==RoomViewKind::Neow=>neow::choose(run,*id),
        GameAction::Move(id)=>if uses_act4(&run.state)&&run.state.act==4{act4::enter(run,*id)}else if uses_captured_act3(&run.state)&&run.state.act==3{act3::enter(run,*id)}else if uses_captured_act2(&run.state)&&run.state.act==2{act2::enter(run,*id)}else if uses_captured_act1(&run.state){act1::enter(run,*id)}else{profile::enter(run,*id)},
        GameAction::DiscardPotion{slot}=>{run.state.potions[*slot as usize]=None;Ok(empty_transition())},
        GameAction::UsePotion{slot,target:None}=>{crate::items::use_run_potion(&mut run.state,*slot as usize,false,false)?;Ok(empty_transition())},
        GameAction::Select(id)=>run.select(*id),
        GameAction::Cancel=>{run.state.pending=None;run.candidates.clear();Ok(empty_transition())},
        GameAction::Choose(id)=>match public(run)?.kind {
            RoomViewKind::Act3BossComplete|RoomViewKind::Act3Ending=>act3_end::choose(run,*id),
            RoomViewKind::HeartVictory=>act4::choose_heart_proceed(run,*id),
            RoomViewKind::BossTreasure|RoomViewKind::BossRelicChoice|RoomViewKind::BossRelicChild|RoomViewKind::ProceedWithoutRelic=>boss::choose(run,*id),
            RoomViewKind::Shop=>shop::choose(run,*id),RoomViewKind::Campfire=>rest::choose(run,*id),RoomViewKind::Event(_)=>events::choose(run,*id),_=>rewards::choose(run,*id),
        },
        _=>Err(RunError::Unsupported("room handler pending")),
    }
}
pub(super) fn empty_transition()->RunTransition{RunTransition{reward:0,terminal:None,committed_events:vec![]}}
pub(in crate::run) fn is_m5(state:&RunState)->bool{state.room_loop.as_ref().is_some_and(|r|r.scenario==RoomScenario::M5Act1Captured)}
pub(in crate::run) fn is_m6(state:&RunState)->bool{state.room_loop.as_ref().is_some_and(|r|r.scenario==RoomScenario::M6Act2Prefix)}
pub(in crate::run) fn is_m7(state:&RunState)->bool{state.room_loop.as_ref().is_some_and(|r|r.scenario==RoomScenario::M7Act2Captured)}
pub(in crate::run) fn is_m8(state:&RunState)->bool{state.room_loop.as_ref().is_some_and(|r|r.scenario==RoomScenario::M8Act3Prefix)}
pub(in crate::run) fn is_m9(state:&RunState)->bool{state.room_loop.as_ref().is_some_and(|r|r.scenario==RoomScenario::M9IroncladAct3)}
pub(in crate::run) fn is_m10(state:&RunState)->bool{state.room_loop.as_ref().is_some_and(|r|r.scenario==RoomScenario::M10IroncladHeart)}
pub(in crate::run) fn uses_captured_act3(state:&RunState)->bool{is_m8(state)||uses_full_act3(state)}
pub(in crate::run) fn uses_captured_act2(state:&RunState)->bool{is_m6(state)||is_m7(state)||is_m13(state)||is_m18(state)||is_m23(state)||uses_captured_act3(state)}
/// Captured Act2 continues through boss33/treasure34 and the (2,3,34) transition.
pub(in crate::run) fn uses_act2_boss(state:&RunState)->bool{is_m7(state)||is_m13(state)||is_m18(state)||is_m23(state)||uses_captured_act3(state)}
pub(in crate::run) fn is_m18(state:&RunState)->bool{state.room_loop.as_ref().is_some_and(|r|r.scenario==RoomScenario::M18DefectAct2)}
pub(in crate::run) fn is_m19(state:&RunState)->bool{state.room_loop.as_ref().is_some_and(|r|r.scenario==RoomScenario::M19DefectAct3)}
pub(crate) fn is_m20(state:&RunState)->bool{state.room_loop.as_ref().is_some_and(|r|r.scenario==RoomScenario::M20DefectHeart)}
pub(in crate::run) fn uses_defect_content(state:&RunState)->bool{is_m17(state)||is_m18(state)||is_m19(state)||is_m20(state)}
pub(in crate::run) fn is_m17(state:&RunState)->bool{state.room_loop.as_ref().is_some_and(|r|r.scenario==RoomScenario::M17DefectAct1)}
pub(in crate::run) fn is_m12(state:&RunState)->bool{state.room_loop.as_ref().is_some_and(|r|r.scenario==RoomScenario::M12SilentAct1)}
pub(in crate::run) fn is_m13(state:&RunState)->bool{state.room_loop.as_ref().is_some_and(|r|r.scenario==RoomScenario::M13SilentAct2)}
pub(crate) fn is_m15(state:&RunState)->bool{state.room_loop.as_ref().is_some_and(|r|r.scenario==RoomScenario::M15SilentHeart)}
pub(in crate::run) fn uses_act4(state:&RunState)->bool{is_m10(state)||is_m15(state)||is_m20(state)}
pub(in crate::run) fn is_m14(state:&RunState)->bool{state.room_loop.as_ref().is_some_and(|r|r.scenario==RoomScenario::M14SilentAct3)}
pub(in crate::run) fn uses_silent_content(state:&RunState)->bool{is_m12(state)||is_m13(state)||is_m14(state)||is_m15(state)}
pub(in crate::run) fn uses_full_act3(state:&RunState)->bool{is_m9(state)||is_m10(state)||is_m14(state)||is_m15(state)||is_m19(state)||is_m20(state)||is_m24(state)}
pub(crate) fn is_m22(state:&RunState)->bool{state.room_loop.as_ref().is_some_and(|r|r.scenario==RoomScenario::M22WatcherAct1)}
pub(crate) fn is_m23(state:&RunState)->bool{state.room_loop.as_ref().is_some_and(|r|r.scenario==RoomScenario::M23WatcherAct2)}
pub(crate) fn is_m24(state:&RunState)->bool{state.room_loop.as_ref().is_some_and(|r|r.scenario==RoomScenario::M24WatcherAct3)}
/// Finite Watcher normal profiles share the M22 producer/startup owner.
pub(crate) fn uses_watcher_content(state:&RunState)->bool{is_m22(state)||is_m23(state)||is_m24(state)}
pub(in crate::run) fn uses_captured_act1(state:&RunState)->bool{is_m22(state)||is_m17(state)||is_m5(state)||is_m10(state)||is_m12(state)||is_m13(state)||uses_captured_act2(state)}
pub(super) fn token(next:&mut u64)->Result<ChoiceId,RunError>{let id=*next;*next=next.checked_add(1).ok_or(RunError::Unsupported("choice identity exhausted"))?;Ok(ChoiceId(id))}
pub(super) fn view(next:&mut u64,kind:RoomViewKind,rows:Vec<(RoomChoiceKind,Option<u32>)>)->Result<RoomView,RunError>{
    let mut staged=*next;let choices=rows.into_iter().map(|(kind,price)|Ok(RoomChoiceView{id:token(&mut staged)?,kind,price,status:RoomChoiceStatus::Offered})).collect::<Result<_,RunError>>()?;
    *next=staged;Ok(RoomView{kind,choices})
}
pub(super) fn finish(run:&mut Run)->Result<RunTransition,RunError>{
    if run.state.deck.is_empty(){return run.commit_terminal(RunTerminal::EmptyDeck)}
    if uses_captured_act3(&run.state)&&run.state.act==3{return act3::finish_room(run)}
    if uses_act4(&run.state)&&run.state.act==4{return act4::finish_room(run)}
    if uses_captured_act2(&run.state)&&run.state.act==2{return act2::finish_room(run)}
    let kind=if !uses_captured_act1(&run.state)&&run.state.map.as_ref().and_then(|m|m.current)==Some(NodeId(59)){RoomViewKind::CheckpointComplete}else{RoomViewKind::Map};
    run.state.room_loop.as_mut().unwrap().screen=RoomView{kind,choices:vec![]};run.candidates.clear();Ok(empty_transition())
}
impl Run {
    pub fn new_room_scenario(config:RunConfig,profile:CreationProfile,rng:RunRng,scenario:RoomScenario)->Result<Self,RunError>{
        let silent=matches!(scenario,RoomScenario::M12SilentAct1|RoomScenario::M13SilentAct2|RoomScenario::M14SilentAct3|RoomScenario::M15SilentHeart);
        let character=if matches!(scenario,RoomScenario::M22WatcherAct1|RoomScenario::M23WatcherAct2|RoomScenario::M24WatcherAct3){crate::content::Character::Watcher}else if matches!(scenario,RoomScenario::M17DefectAct1|RoomScenario::M18DefectAct2|RoomScenario::M19DefectAct3|RoomScenario::M20DefectHeart){crate::content::Character::Defect}else if silent{crate::content::Character::Silent}else{crate::content::Character::Ironclad};
        if config.character!=character||config.ascension!=0||profile.neow_eligible{return Err(RunError::Unsupported(match scenario{RoomScenario::M24WatcherAct3=>"M24 requires Watcher A0 short Neow",RoomScenario::M23WatcherAct2=>"M23 requires Watcher A0 short Neow",RoomScenario::M22WatcherAct1=>"M22 requires Watcher A0 short Neow",RoomScenario::M20DefectHeart=>"M20 requires Defect A0 short Neow",RoomScenario::M19DefectAct3=>"M19 requires Defect A0 short Neow",RoomScenario::M18DefectAct2=>"M18 requires Defect A0 short Neow",RoomScenario::M17DefectAct1=>"M17 requires Defect A0 short Neow",RoomScenario::M12SilentAct1=>"M12 requires Silent A0 short Neow",RoomScenario::M13SilentAct2=>"M13 requires Silent A0 short Neow",RoomScenario::M14SilentAct3=>"M14 requires Silent A0 short Neow",RoomScenario::M15SilentHeart=>"M15 requires Silent A0 short Neow",_=>"M4 requires Ironclad A0 short Neow"}))}
        let mut run=Self::new(config,profile,rng)?;
        let choices=run.candidates.iter().map(|c|RoomChoiceView{id:c.id,kind:match c.neow_offer.as_ref().unwrap().reward{
            NeowReward::Lament=>RoomChoiceKind::NeowLament,_=>RoomChoiceKind::NeowMaxHp{amount:(run.state.max_hp as f32*0.1_f32) as u32}},price:None,status:RoomChoiceStatus::Offered}).collect();
        run.state.room_loop=Some(RoomLoop{scenario,act1:None,act2:None,act3:None,act4:None,m10_elite_relic_cursor:0,screen:RoomView{kind:RoomViewKind::Neow,choices},purge_cost:75,reward_parent:None,active_rewards:None,boss:None,transition:None,act2_boss_order:None,act3_boss_order:None,joust:None,joust_visits:0,act3_event:None,act3_end:None});Ok(run)
    }
}
