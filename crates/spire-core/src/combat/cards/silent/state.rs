//! Typed green continuations and source-checkpoint history, owned by combat.
use super::*;
use crate::combat::model::OwnedCard;

#[derive(Clone, Default, PartialEq, Eq)]
pub struct SilentState {
    /// New profile/card history support opts into explicit draw continuations.
    /// Old accepted profiles retain their original effect/fault queue bytes.
    pub history_enabled: bool,
    pub(in crate::combat) smoked:bool,
    pub(in crate::combat) source_character:Option<crate::content::Character>,
    pub retain_block_next_start: bool,
    pub played_this_turn: Vec<crate::content::CardId>,
    pub discarded_this_turn: u32,
    pub drawn_cards: Vec<crate::content::CardId>,
    pub(in crate::combat) card_offer:Option<crate::combat::card_offers::PendingCardOffer>,
}

impl std::fmt::Debug for SilentState {
    fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result {
        let mut value=f.debug_struct("SilentState");
        value.field("history_enabled",&self.history_enabled);
        value.field("retain_block_next_start",&self.retain_block_next_start);
        value.field("played_this_turn",&self.played_this_turn);
        value.field("discarded_this_turn",&self.discarded_this_turn);
        value.field("drawn_cards",&self.drawn_cards);
        if self.smoked{value.field("smoked",&self.smoked);}
        if self.source_character.is_some(){value.field("source_character",&self.source_character);}
        if self.card_offer.is_some(){value.field("card_offer",&self.card_offer);}
        value.finish()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SilentEffect {
    DrawContinue(i32),
    BaneHit{damage:i32},
    FlaskStart{bounces:u32},
    FlaskBounce{target:Option<EnemyHandle>,remaining:u32},
    ToolsDiscard { count:usize, turn:u32 },
    Setup,
    BulletTime,
    GlassKnifeAfterHits,
    EndlessAgonyCopy { card: CardHandle, template: Box<super::LiveCard> },
    Discard { count: usize, random: bool },
    DiscardSpecific { card: CardHandle },
    Gamble,
    Unload,
    Expertise { size: usize },
    EscapePlanBlock { block: i32 },
    AttackCountHits { damage: i32 },
    SkillCountHits { damage: i32 },
    SneakyEnergy,
    BurstRepeat { child: CardHandle, target: Option<Actor>, energy_on_use: i32, cancelled: bool },
    StormOfSteel { cards: Vec<CardHandle>, shiv_upgrades: u32 },
}

pub(in crate::combat) fn increment_discard(state: &mut State, notify_cards: bool) -> Result<(), InputError> {
    state.silent.discarded_this_turn=state.silent.discarded_this_turn.checked_add(1).filter(|count|*count<=i32::MAX as u32)
        .ok_or_else(||InputError::Unsupported("native discard counter exhausted".into()))?;
    if notify_cards { super::costs::manual_discard(state)?; }
    Ok(())
}

pub(in crate::combat) fn validate_work(state:&State,effect:&Effect,work:&SilentEffect)->Result<(),InputError>{
    if !state.full_runtime {return Err(InputError::Invariant("Silent work requires native combat".into()))}
    let valid=match work {
        SilentEffect::BaneHit{damage}=>*damage>=0,
        SilentEffect::FlaskStart{bounces}=>(3..=4).contains(bounces),
        SilentEffect::FlaskBounce{target,remaining}=>(1..=4).contains(remaining)&&target.as_ref().is_none_or(|owner|state.enemies.iter().any(|e|e.instance_id==*owner)),
        SilentEffect::Discard{count,..}=>(1..=10).contains(count),
        SilentEffect::Expertise{size}=>(1..=10).contains(size),
        SilentEffect::EscapePlanBlock{block}=>*block>=0,
        SilentEffect::AttackCountHits{damage}|SilentEffect::SkillCountHits{damage}=>*damage>=0,
        SilentEffect::DiscardSpecific{card}=>state.card_ids_issued.contains(card),
        SilentEffect::EndlessAgonyCopy{card,template}=>state.card_ids_issued.contains(card)
            &&effect.source==Source::Card(card.clone())&&effect.target==Some(Actor::Player)
            &&template.id==crate::content::CardId::EndlessAgony
            &&crate::combat::card_state::handle(template.instance)==*card,
        SilentEffect::StormOfSteel{cards,shiv_upgrades}=>*shiv_upgrades<=1&&cards.len()<=10
            &&cards.iter().collect::<std::collections::BTreeSet<_>>().len()==cards.len()
            &&cards.iter().all(|handle|state.piles.hand.iter().any(|owned|owned.handle()==*handle)),
        _=>true,
    };
    if !valid{return Err(InputError::Invariant("invalid Silent continuation payload".into()))}
    if let SilentEffect::ToolsDiscard{count,turn}=work {
        return if *count>0&&*count<=i32::MAX as usize&&*turn==state.turn&&effect.source==Source::Actor(Actor::Player)&&state.in_play.is_none() {Ok(())}else{Err(InputError::Invariant("Tools discard continuation owner".into()))}
    }
    if let SilentEffect::DrawContinue(count)=work {
        if !(0..=10).contains(count){return Err(InputError::Invariant("draw continuation count".into()))}
        return Ok(())
    }
    if matches!(work,SilentEffect::EndlessAgonyCopy{..}) { return Ok(()); }
    if let SilentEffect::StormOfSteel{..}=work {
        let Source::Card(owner)=&effect.source else{return Err(InputError::Invariant("Storm of Steel source absent".into()))};
        return if state.in_play.as_ref().is_some_and(|card|card.handle()==*owner
            &&card.full().is_some_and(|live|live.id==crate::content::CardId::StormOfSteel)) {Ok(())}
        else {Err(InputError::Invariant("Storm of Steel owner is not in play".into()))};
    }
    if let SilentEffect::BurstRepeat{child,target,energy_on_use,..}=work {
        let Source::Card(parent)=&effect.source else{return Err(InputError::Invariant("Burst repeat parent source absent".into()))};
        let parent_owned=state.in_play.as_ref().is_some_and(|card|card.handle()==*parent)
            ||state.play_frames.iter().any(|frame|matches!(&frame.parent_owner,crate::combat::model::PlayFrameParentOwner::Card(owner) if owner==parent)&&frame.child==*child&&frame.parent_finish_checkpoint_done);
        let child_skill=state.queued_autoplay.get(child).and_then(OwnedCard::full).is_some_and(|card|
            crate::content::Catalog::standard().card(card.id).is_some_and(|definition|definition.card_type==crate::content::CardType::Skill));
        let target_exists=target.as_ref().is_none_or(|actor|match actor{
            Actor::Player=>false,
            Actor::Enemy(owner)=>state.enemies.iter().any(|enemy|enemy.instance_id==*owner),
        });
        return if state.full_runtime&&parent_owned&&child_skill&&*energy_on_use>=-1&&target_exists {Ok(())}
            else {Err(InputError::Invariant("Burst repeat child/parent ownership".into()))};
    }
    let Source::Card(owner)=&effect.source else{return Err(InputError::Invariant("Silent continuation card source absent".into()))};
    if !state.in_play.as_ref().is_some_and(|card|card.handle()==*owner){return Err(InputError::Invariant("Silent continuation parent absent".into()))}
    let parent_id=state.in_play.as_ref().and_then(|owned|owned.full()).map(|card|card.id);
    let expected=match work {
        SilentEffect::Setup=>Some(crate::content::CardId::Setup),
        SilentEffect::BulletTime=>Some(crate::content::CardId::BulletTime),
        SilentEffect::GlassKnifeAfterHits=>Some(crate::content::CardId::GlassKnife),
        _=>None,
    };
    if expected.is_some_and(|id|parent_id!=Some(id)){
        return Err(InputError::Invariant("Silent cost continuation has the wrong parent card".into()))
    }
    Ok(())
}
