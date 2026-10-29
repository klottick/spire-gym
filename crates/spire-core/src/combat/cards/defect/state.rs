//! M20 stateful blue cards over the shared pile, history and selection owners.
use super::*;
use crate::combat::{model::OwnedCard,selection::{SelectionOperation as O,SelectionSourceZone as Z,SelectionDisposition as D,SelectionParent as P,EmptySelection}};
#[derive(Clone,Debug,PartialEq,Eq)]
pub enum StateEffect {
    Ftl{damage:i32,threshold:usize},
    X{kind:CardId,count:i32,block:i32,free:bool,paid:bool},
    Hologram,Seek{count:usize},Recycle,AllForOne,Retrieve{card:CardHandle},Scrape,Reboot,
}
fn work(action:StateEffect)->(Option<PayloadTarget>,EffectKind){(None,EffectKind::Defect(DefectEffect::State(action)))}
pub(super) fn payload(state:&State,card:&LiveCard,target:Option<u64>)->Result<Option<(i32,Vec<(Option<PayloadTarget>,EffectKind)>)>,RunError>{
    use CardId::*;let up=card.upgrades>0;
    if !matches!(card.id,Hologram|Rebound|Equilibrium|ForceField|FTL|Recycle|ReinforcedBody|Tempest|MultiCast|Scrape|Reboot|Seek|AllForOne){return Ok(None)}
    if matches!(card.id,Rebound|FTL|Scrape|AllForOne)!=target.is_some(){return Err(RunError::InvalidAction)}
    let mut effects=Vec::new();let player=Some(PayloadTarget::Player);
    match card.id {
        Hologram=>{effects.push((player,EffectKind::GainBlock(modified_block(state,if up{5}else{3})?)));effects.push(work(StateEffect::Hologram));},
        Rebound=>{effects.push((Some(PayloadTarget::SelectedEnemy),EffectKind::Damage(DamagePayload::Fixed(modified_damage(state,target.unwrap(),if up{12}else{9})?))));effects.push((Some(PayloadTarget::Player),EffectKind::ApplyFullPower{id:FullPowerId("rebound"),amount:1}));},
        Equilibrium=>{effects.push((player,EffectKind::GainBlock(modified_block(state,if up{16}else{13})?)));effects.push((Some(PayloadTarget::Player),EffectKind::ApplyFullPower{id:FullPowerId("equilibrium"),amount:1}));},
        ForceField=>effects.push((player,EffectKind::GainBlock(modified_block(state,if up{16}else{12})?))),
        FTL=>effects.push((Some(PayloadTarget::SelectedEnemy),EffectKind::Defect(DefectEffect::State(StateEffect::Ftl{damage:modified_damage(state,target.unwrap(),if up{6}else{5})?,threshold:if up{4}else{3}})))),
        ReinforcedBody|Tempest|MultiCast=>{
            let chemical=state.item_context.as_ref().is_some_and(|items|items.owns_relic(crate::content::RelicId::ChemicalX));
            let count=super::super::captured_x(state,card)?+if chemical{2}else{0}+i32::from(up&&card.id!=ReinforcedBody);
            effects.push(work(StateEffect::X{kind:card.id,count,block:if card.id==ReinforcedBody{modified_block(state,if up{9}else{7})?}else{0},free:card.local.free_play||card.local.free_to_play_once,paid:false}));
        },
        Recycle=>effects.push(work(StateEffect::Recycle)),
        Seek=>effects.push(work(StateEffect::Seek{count:if up{2}else{1}})),
        AllForOne|Scrape=>{
            let base=if card.id==AllForOne{if up{14}else{10}}else{if up{10}else{7}};
            effects.push((Some(PayloadTarget::SelectedEnemy),EffectKind::Damage(DamagePayload::Fixed(modified_damage(state,target.unwrap(),base)?))));
            if card.id==Scrape{effects.push((None,EffectKind::Draw(if up{5}else{4})));effects.push(work(StateEffect::Scrape));}
            else{effects.push(work(StateEffect::AllForOne));}
        },
        Reboot=>{effects.push(work(StateEffect::Reboot));effects.push((None,EffectKind::ShuffleDraw));effects.push((None,EffectKind::Draw(if up{6}else{4})));},
        _=>unreachable!(),
    }
    Ok(Some((starting_cost(card.id,card.upgrades).unwrap(),effects)))
}
pub(super) fn public_values(card:&LiveCard)->Option<CardPublicValues>{
    use CardId::*;let up=card.upgrades>0;
    let (damage,block,magic)=match card.id {
        Hologram=>(None,Some(if up{5}else{3}),None),Rebound=>(Some(if up{12}else{9}),None,None),
        Equilibrium=>(None,Some(if up{16}else{13}),None),ForceField=>(None,Some(if up{16}else{12}),None),
        FTL=>(Some(if up{6}else{5}),None,Some(if up{4}else{3})),
        ReinforcedBody=>(None,Some(if up{9}else{7}),None),Tempest|MultiCast=>(None,None,Some(i32::from(up))),
        AllForOne=>(Some(if up{14}else{10}),None,None),Scrape=>(Some(if up{10}else{7}),None,Some(if up{5}else{4})),
        Seek=>(None,None,Some(if up{2}else{1})),Reboot=>(None,None,Some(if up{6}else{4})),Recycle=>(None,None,None),_=>return None,
    };
    Some(CardPublicValues{base_damage:damage,base_block:block,magic_number:magic,damage_bonus:None,
        cost_kind:if card.local.x_cost{CardCostKind::X}else{CardCostKind::Fixed},free_play:card.local.free_play||card.local.free_to_play_once,
        exhaust_on_finish:card.local.exhaust_on_finish,ethereal:card.local.ethereal,innate:card.local.innate,temporary_cost:card.local.temporary_cost})
}
pub(super) fn validate_work(state:&State,effect:&Effect,work:&StateEffect)->Result<(),InputError>{
    let Source::Card(source)=&effect.source else{return Err(InputError::Invariant("blue state source".into()))};
    let Some(card)=state.in_play.as_ref().filter(|owned|owned.handle()==*source).and_then(OwnedCard::full)else{return Err(InputError::Invariant("blue state in-play owner".into()))};
    use CardId::*;
    let valid=match work {
        StateEffect::Ftl{damage,threshold}=>card.id==FTL&&*damage>=0&&*threshold==if card.upgrades>0{4}else{3},
        StateEffect::X{kind,count,block,..}=>*kind==card.id&&matches!(kind,ReinforcedBody|Tempest|MultiCast)&&(0..=1002).contains(count)&&*block>=0,
        StateEffect::Hologram=>card.id==Hologram,StateEffect::Seek{count}=>card.id==Seek&&*count==if card.upgrades>0{2}else{1},
        StateEffect::Recycle=>card.id==Recycle,StateEffect::AllForOne|StateEffect::Retrieve{..}=>card.id==AllForOne,
        StateEffect::Scrape=>card.id==Scrape,StateEffect::Reboot=>card.id==Reboot,
    };
    if state.full_runtime&&valid{Ok(())}else{Err(InputError::Invariant("blue state payload/producer".into()))}
}
fn open(state:&mut State,source:&Source,candidates:Vec<CardHandle>,zone:Z,count:usize,operation:O,disposition:D)->Result<(),EffectError>{
    let Source::Card(parent)=source else{unreachable!()};
    crate::combat::effects::begin_selection_range(state,P::Card(parent.clone()),zone,candidates,count,count,disposition,EmptySelection::Continue)?;
    state.pending_selection.as_mut().unwrap().operation=operation;Ok(())
}
fn retrieve(state:&mut State,handle:&CardHandle)->Result<(),InputError>{crate::combat::piles::retrieve_ordinary_from_discard(state,handle).map(|_|())}
pub(super) fn apply(state:&mut State,effect:&Effect,work:&StateEffect)->Result<(),EffectError>{
    let child=|kind|Effect{source:effect.source.clone(),target:effect.target.clone(),kind};
    match work {
        StateEffect::Ftl{damage,threshold}=>{
            crate::combat::enemies::act2::bottom(state,child(EffectKind::Damage(DamagePayload::Fixed(*damage))));
            if state.silent.played_this_turn.len().saturating_sub(1)<*threshold{state.pending.push_front(child(EffectKind::Draw(1)));}
        },
        StateEffect::X{kind,count,block,free,paid}=>{
            if *count<=0||*kind==CardId::MultiCast&&state.character.orb_slots.first().copied().flatten().is_none(){return Ok(())}
            if !*paid&&!*free{state.player.energy=0;}
            if let Some(current)=state.current.as_mut(){current.kind=EffectKind::Defect(DefectEffect::State(StateEffect::X{kind:*kind,count:*count,block:*block,free:*free,paid:true}));}
            for index in (0..*count).rev(){
                let kind=match kind {CardId::ReinforcedBody=>EffectKind::GainBlock(*block),CardId::Tempest=>EffectKind::Character(C::Channel(OrbKind::Lightning)),CardId::MultiCast=>EffectKind::Character(C::EvokeFront{remove:index==*count-1}),_=>unreachable!()};
                state.pending.push_front(Effect{source:effect.source.clone(),target:Some(Actor::Player),kind});
            }
        },
        StateEffect::Hologram=>{
            let candidates=state.piles.discard.iter().map(OwnedCard::handle).collect::<Vec<_>>();
            if candidates.len()<=1{if let Some(card)=candidates.first(){retrieve(state,card)?;}}
            else{open(state,&effect.source,candidates,Z::Discard,1,O::HologramRetrieve,D::Hand)?;}
        },
        StateEffect::Seek{count}=>{
            let mut candidates=state.piles.draw.iter().map(OwnedCard::handle).collect::<Vec<_>>();
            if candidates.len()<=*count{for card in candidates{crate::combat::piles::move_draw_card(state,&card,CardDestination::Hand)?;}}
            else{
                candidates.sort_by_key(|handle|{let card=state.piles.draw.iter().find(|card|card.handle()==*handle).unwrap().full().unwrap();(card.id.as_str(),card.instance)});
                open(state,&effect.source,candidates,Z::Draw,*count,O::SeekRetrieve,D::Hand)?;
            }
        },
        StateEffect::Recycle=>{
            let candidates=state.piles.hand.iter().map(OwnedCard::handle).collect::<Vec<_>>();
            if candidates.len()<=1{if let Some(card)=candidates.first(){crate::combat::piles::recycle_hand_card(state,&effect.source,card)?;}}
            else{open(state,&effect.source,candidates,Z::Hand,1,O::Recycle,D::Exhaust)?;}
        },
        StateEffect::AllForOne=>{
            let cards=state.piles.discard.iter().filter_map(|owned|owned.full().filter(|card|
                card.local.free_to_play_once||!card.local.x_cost&&card.local.mutable_combat_cost.or(card.local.randomized_cost).or_else(||super::super::base_cost(state,card))==Some(0)).map(|_|owned.handle())).collect::<Vec<_>>();
            for card in cards.into_iter().rev(){state.pending.push_front(child(EffectKind::Defect(DefectEffect::State(StateEffect::Retrieve{card}))));}
        },
        StateEffect::Retrieve{card}=>retrieve(state,card)?,
        StateEffect::Scrape=>{
            for entry in state.silent.drawn_cards.clone(){
                let Some(instance)=entry.instance else{continue};
                let Some(index)=state.piles.hand.iter().position(|owned|owned.full().is_some_and(|card|card.instance==instance))else{continue};
                let card=state.piles.hand[index].full().unwrap();
                let turn=card.local.turn_cost.or_else(||crate::combat::card_state::combat_price(card,super::super::base_cost(state,card).unwrap_or(-2)).ok());
                if turn==Some(0)||card.local.free_to_play_once{continue}
                let card=state.piles.hand.remove(index);crate::combat::piles::discard_selected_card_ordered(state,card,true,true)?;
            }
        },
        StateEffect::Reboot=>crate::combat::piles::shuffle_all_to_draw(state)?,
    }Ok(())
}
