//! Standard event pools, eligibility and the selection law (design §5.2). State lives on
//! `RunState::event_pools`, present only under `RoomScenario::Standard`.
use super::*;
use crate::content::{CardType, Catalog, EventPool};

/// The event lists of a standard Run: `ordinary` and `shrines` are rebuilt at each act start;
/// `one_time` is built once at Run creation and never replenished; `seen` records every event
/// constructed, in order.
#[derive(Clone,Debug,PartialEq,Eq)]
pub(crate) struct EventPools { pub ordinary:Vec<EventId>, pub shrines:Vec<EventId>, pub one_time:Vec<EventId>, pub seen:Vec<EventId> }

/// Catalog rows of `pool` available in `act`, in catalog (source) order.
fn rows(pool:EventPool,act:u8)->Vec<EventId>{
    Catalog::standard().events().iter().filter(|row|row.pool==pool&&row.acts&(1<<act)!=0).map(|row|row.id).collect()
}
fn one_time_rows()->Vec<EventId>{Catalog::standard().events().iter().filter(|row|row.pool==EventPool::OneTime).map(|row|row.id).collect()}

impl EventPools {
    /// Run creation: Act 1's ordinary and shrine lists and every one-time row.
    pub(crate) fn new(act:u8)->Self{Self{ordinary:rows(EventPool::Ordinary,act),shrines:rows(EventPool::Shrine,act),one_time:one_time_rows(),seen:vec![]}}
    /// Act start: rebuild ordinary and shrines; one-time rows and history persist.
    pub(crate) fn rebuild(&mut self,act:u8){self.ordinary=rows(EventPool::Ordinary,act);self.shrines=rows(EventPool::Shrine,act);}
    fn remove(&mut self,id:EventId){
        for list in [&mut self.ordinary,&mut self.shrines,&mut self.one_time]{if let Some(i)=list.iter().position(|e|*e==id){list.remove(i);return}}
    }
}

/// `isCursed` (event-neow research :287; E9): a curse other than Ascender's Bane, Curse of the Bell
/// and Necronomicurse.
fn is_cursed(state:&RunState)->bool{
    state.deck.iter().any(|card|mutations::removable(card.id)
        &&Catalog::standard().card(card.id).is_some_and(|row|row.card_type==CardType::Curse))
}

/// `event_eligible` (design §5.2): the act mask of the catalog row, then the row's rule. `node` is
/// the event room's map node (Colosseum reads its row).
pub(crate) fn eligible(state:&RunState,id:EventId,node:NodeId)->bool{
    let Some(row)=Catalog::standard().event(id) else {return false};
    if row.acts&(1<<state.act)==0{return false}
    match id{
        EventId::DeadAdventurer|EventId::Mushrooms=>state.floor>6,
        EventId::Cleric=>state.gold>=35,
        EventId::FountainOfCleansing=>is_cursed(state),
        EventId::WomanInBlue|EventId::Joust=>state.gold>=50,
        EventId::Designer|EventId::Beggar=>state.gold>=75,
        EventId::KnowingSkull=>state.hp>12,
        // Below A15 and at most the highest unlocked Ascension, taken as 20 (UNLOCK-001).
        EventId::NoteForYourself=>state.ascension<15,
        // The 800-second playtime gate is treated as met (TIME-001); the act mask is Act 3.
        EventId::SecretPortal=>true,
        EventId::Nloth=>state.relics.len()>=2,
        EventId::MoaiHead=>(state.hp as f32)/(state.max_hp as f32)<=0.5_f32||state.relics.contains(&RelicId::GoldenIdol),
        // E10: strictly beyond half the map's row count (integer row index; the boss node is not a
        // map row, so a standard map has 15 rows and Colosseum needs y >= 8).
        EventId::Colosseum=>state.map.as_ref().is_some_and(|map|{
            let rows=map.nodes.iter().filter(|n|n.kind!=MapNodeKind::Boss).map(|n|n.y).max().map_or(0,|y|y+1);
            map.nodes.iter().any(|n|n.id==node&&n.y>rows/2)
        }),
        _=>true,
    }
}

/// The outcome of the selection law: an event to construct, or the E6 monster fallback.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub(crate) enum Selected { Event(EventId), Monster }

/// The selection law (design §5.2, Act 1 research §3.1, E6) with its draws supplied: `shrine` is
/// the branch draw's result (`< 0.25`), `pick(len)` an index below `len`. Removes the picked ID
/// from its list and records it in `seen`.
pub(crate) fn select_with(pools:&mut EventPools,state:&RunState,node:NodeId,shrine:bool,mut pick:impl FnMut(usize)->Result<usize,RunError>)->Result<Selected,RunError>{
    let ordinary=|pools:&EventPools|pools.ordinary.iter().copied().filter(|id|eligible(state,*id,node)).collect::<Vec<_>>();
    let mut chosen=None;
    let mut delegated=false;
    if !shrine{
        let list=ordinary(pools);
        if list.is_empty(){delegated=true}else{chosen=Some(list[pick(list.len())?])}
    }
    if chosen.is_none(){
        if !pools.shrines.is_empty()||!pools.one_time.is_empty(){
            // Remaining shrines, then the eligible one-time rows (E6a: eligible-empty falls back).
            let list=pools.shrines.iter().chain(pools.one_time.iter()).copied().filter(|id|eligible(state,*id,node)).collect::<Vec<_>>();
            if !list.is_empty(){chosen=Some(list[pick(list.len())?])}
        }else if !pools.ordinary.is_empty()&&!delegated{
            // Both raw shrine lists empty: the ordinary pick once (E6c ends a second delegation).
            let list=ordinary(pools);
            if !list.is_empty(){chosen=Some(list[pick(list.len())?])}
        }
    }
    let Some(id)=chosen else {return Ok(Selected::Monster)};
    pools.remove(id);
    pools.seen.push(id);
    Ok(Selected::Event(id))
}

/// The selection law with its draws (design §8): `M28EventBranch` (binary32 `< 0.25` is the shrine
/// branch) and `M28EventPick`, keyed by act, floor and node.
pub(crate) fn select(state:&mut RunState,node:NodeId)->Result<Selected,RunError>{
    let mut pools=state.event_pools.clone().ok_or(RunError::InvalidAction)?;
    let key=[u64::from(state.act),u64::from(state.floor),node.0];
    let mut rng=state.rng.clone();
    let roll=((rng.draw_scoped(RunRngScope::M28EventBranch,&key)>>40) as f32)/16_777_216.0;
    let selected=select_with(&mut pools,state,node,roll<0.25_f32,|len|rng.bounded_scoped_checked(RunRngScope::M28EventPick,&key,len as u64))?;
    state.rng=rng;
    state.event_pools=Some(pools);
    Ok(selected)
}
