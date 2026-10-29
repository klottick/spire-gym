//! Generated templates suspend on the same native resolver as pile selections.
use super::*;
use super::model::State;
use crate::content::CardId;

#[derive(Clone,Debug,PartialEq,Eq)]
pub(super) struct PendingCardOffer {
    pub cards:Vec<CardId>,pub optional:bool,pub copies:u8,pub zero_cost:bool,
    pub source:Source,pub resume:Phase,
}

pub(super) fn validate_owner(state:&State,source:&Source)->Result<(),InputError>{
    let valid=match source {
        Source::Card(owner)=>state.in_play.as_ref().is_some_and(|card|card.handle()==*owner)
            &&state.pending.iter().any(|effect|effect.kind==EffectKind::FinishCard&&effect.source==Source::Card(owner.clone())),
        Source::Potion{slot,id}=>state.item_context.as_ref().and_then(|items|items.potions().get(*slot))
            .is_some_and(|owned|owned.is_none()||*owned==Some(*id)),
        Source::Actor(Actor::Player)=>state.in_play.is_none(),
        _=>false,
    };
    if valid{Ok(())}else{Err(InputError::Invariant("generated offer parent owner absent".into()))}
}

pub(super) fn validate_work(state:&State,effect:&Effect)->Result<(),InputError>{
    if !state.full_runtime||effect.target!=Some(Actor::Player){return Err(InputError::Invariant("generated offer native target".into()))}
    validate_owner(state,&effect.source)?;
    let valid_kind=|id:CardId|crate::content::Catalog::standard().card(id)
        .is_some_and(|definition|matches!(definition.card_type,crate::content::CardType::Attack|crate::content::CardType::Skill|crate::content::CardType::Power))
        &&super::card_state::generated(id,CardInstanceId(0),0).is_some();
    let valid=match &effect.kind {
        EffectKind::CardOffer{cards,copies,..}=>cards.len()==3&&(1..=2).contains(copies)
            &&cards.iter().enumerate().all(|(i,id)|valid_kind(*id)&&!cards[..i].contains(id))
            &&state.pending_selection.is_none()&&state.silent.card_offer.is_none(),
        EffectKind::DeliverCardOffer{id,copies,next,..}=>valid_kind(*id)&&(1..=2).contains(copies)&&next<=copies,
        _=>false,
    };
    if valid{Ok(())}else{Err(InputError::Invariant("generated offer kinds/count/ownership".into()))}
}

pub(super) fn apply(state:&mut State,effect:&Effect)->Result<(),EffectError>{
    match &effect.kind {
        EffectKind::CardOffer{cards,optional,copies,zero_cost}=>{
            let resume=state.phase;
            state.silent.card_offer=Some(PendingCardOffer{cards:cards.clone(),optional:*optional,copies:*copies,
                zero_cost:*zero_cost,source:effect.source.clone(),resume});
            state.phase=Phase::Selection;
        }
        EffectKind::DeliverCardOffer{id,copies,next,zero_cost}=>{
            let mut template=super::card_state::generated_with_history(*id,CardInstanceId(0),0,super::card_state::CardConstructionHistory::from_state(state))
                .ok_or_else(||InputError::Unsupported("generated offer template unavailable".into()))?;
            for index in *next..*copies {
                let instance=super::piles::next_generated_instance(state)?;
                let history=super::cards::copy_history(state);
                let mut card=super::cards::make_stat_equivalent_copy(&mut template,instance,history)?;
                // These offers contain ordinary Attack/Skill/Power templates.
                // Reflex/Tactician and X retain their source negative costs.
                if *zero_cost&&!card.local.x_cost&&!matches!(card.id,CardId::Reflex|CardId::Tactician){
                    card.local.turn_cost=Some(0);
                }
                super::piles::insert_generated(state,card,CardDestination::Hand)?;
                if let Some(current)=state.current.as_mut(){
                    current.kind=EffectKind::DeliverCardOffer{id:*id,copies:*copies,next:index+1,zero_cost:*zero_cost};
                }
            }
        }
        _=>return Err(InputError::Invariant("non-offer work reached generated offer owner".into()).into()),
    }
    Ok(())
}

impl Combat {
    pub(super) fn choose_card_offer(&mut self,action:&CombatAction)->Result<(),crate::run::RunError>{
        if self.state.fault.is_some(){return Err(super::engine::retained_fault(&self.state))}
        if self.state.phase!=Phase::Selection{return Err(crate::run::RunError::InvalidAction)}
        let pending=self.state.silent.card_offer.as_ref().ok_or(crate::run::RunError::InvalidAction)?;
        let chosen=match action {
            CombatAction::ChooseCardOffer{index}=>Some(*pending.cards.get(*index as usize).ok_or(crate::run::RunError::InvalidAction)?),
            CombatAction::SkipCardOffer if pending.optional=>None,
            _=>return Err(crate::run::RunError::InvalidAction),
        };
        super::engine::validate_runtime(&self.state).map_err(|_|crate::run::RunError::InvalidAction)?;
        let pending=self.state.silent.card_offer.take().expect("validated offer owner");
        let mark=super::observe::mark(&self.state);
        self.state.resolved_effects=0;self.state.attempted_action=Some(action.clone());self.state.phase=pending.resume;
        if let Some(id)=chosen {
            self.state.pending.push_front(Effect{source:pending.source.clone(),target:Some(Actor::Player),
                kind:EffectKind::DeliverCardOffer{id,copies:pending.copies,next:0,zero_cost:pending.zero_cost}});
        }
        let transition=super::engine::resolve(&mut self.state);
        super::observe::changes(&self.state,mark,&pending.source);self.refresh_legacy_cache();
        if self.state.fault.is_some(){return Err(super::engine::retained_fault(&self.state))}
        self.state.attempted_action=None;
        match transition.status {
            TransitionStatus::Decision|TransitionStatus::Selection|TransitionStatus::Terminal=>Ok(()),
            TransitionStatus::Fault=>Err(crate::run::RunError::InvalidAction),
        }
    }
}
