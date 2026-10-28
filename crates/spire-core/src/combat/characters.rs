//! Character-local stance and orb mechanics used by the shared combat resolver.
use super::{Actor, DamagePayload, Effect, EffectKind, Source};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stance { Neutral, Wrath, Calm }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OrbKind { Lightning, Frost, Dark, Plasma }

/// One owned orb object. Only Dark carries accumulated damage.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct OrbState { pub kind: OrbKind, pub dark_damage: Option<i32> }
impl OrbState {
    pub const fn new(kind: OrbKind) -> Self {
        Self { kind, dark_damage: if matches!(kind, OrbKind::Dark) { Some(6) } else { None } }
    }
}
impl From<OrbKind> for OrbState { fn from(kind: OrbKind) -> Self { Self::new(kind) } }
impl std::fmt::Debug for OrbState {
    fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result {
        if self.kind==OrbKind::Dark { f.debug_struct("Dark").field("damage",&self.dark_damage).finish() }
        else { self.kind.fmt(f) }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CharacterState {
    pub stance: Stance,
    pub orb_slots: Vec<Option<OrbState>>,
    pub focus: i32,
}

impl CharacterState {
    pub fn for_character(character: crate::content::Character) -> Self {
        let orb_slots = if character == crate::content::Character::Defect { vec![None; 3] } else { Vec::new() };
        Self { stance: Stance::Neutral, orb_slots, focus: 0 }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OrbPassivePhase { Start, End }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OrbPassiveGroup { NaturalStart, NaturalEnd, All, Dark }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CharacterEffect {
    Stance(Stance), Focus(i32), Channel(OrbKind), EvokeFront { remove: bool },
    EvokeAt { index: usize }, EndTurn, StartTurn, Slots(i32),
    ChannelState { orb: OrbState, evict: bool }, Recursion, Fission { evoke: bool },
    PassiveGroup(OrbPassiveGroup), Lightning { amount: i32 }, OrbEnergy(i32),
}

pub(super) fn enqueue(state: &mut super::model::State, source: Source, action: CharacterEffect) {
    state.pending.push_front(Effect { source, target: Some(Actor::Player), kind: EffectKind::Character(action) });
}

pub(super) fn apply(state: &mut super::model::State, source: &Source,
    action: CharacterEffect) -> Result<(), super::EffectError>
{
    match action {
        CharacterEffect::Focus(delta) => {
            state.pending.push_front(Effect {source:source.clone(),target:Some(Actor::Player),
                kind:EffectKind::ApplyFullPower{id:super::FullPowerId("focus"),amount:delta}});
        }
        CharacterEffect::Stance(next) => {
            let previous=state.character.stance;
            if previous==next {return Ok(())}
            if previous==Stance::Calm {
                state.player.energy=state.player.energy.checked_add(2).filter(|n|*n<=999)
                    .ok_or_else(||super::InputError::Unsupported("stance energy outside supported bounds".into()))?;
            }
            state.character.stance=next;
            super::effects::refresh_enemy_values(state)?;
        }
        CharacterEffect::Channel(kind)=>channel(state,source,OrbState::new(kind),true)?,
        CharacterEffect::ChannelState{orb,evict}=>channel(state,source,orb,evict)?,
        CharacterEffect::Slots(delta)=>change_slots(state,delta)?,
        CharacterEffect::EvokeFront{remove}=>evoke(state,source,0,remove)?,
        CharacterEffect::EvokeAt{index}=>evoke(state,source,index,true)?,
        CharacterEffect::EndTurn=>passive_group(state,source,OrbPassiveGroup::NaturalEnd)?,
        CharacterEffect::StartTurn=>passive_group(state,source,OrbPassiveGroup::NaturalStart)?,
        CharacterEffect::PassiveGroup(group)=>passive_group(state,source,group)?,
        CharacterEffect::Recursion=>{
            if let Some(orb)=state.character.orb_slots.first().copied().flatten() {
                enqueue(state,source.clone(),CharacterEffect::ChannelState{orb,evict:false});
                enqueue(state,source.clone(),CharacterEffect::EvokeFront{remove:true});
            }
        }
        CharacterEffect::Fission{evoke:invokes}=>{
            let count=state.character.orb_slots.iter().flatten().count() as i32;
            if count>0 {
                state.pending.push_front(Effect{source:source.clone(),target:None,kind:EffectKind::Draw(count)});
                enqueue(state,source.clone(),CharacterEffect::OrbEnergy(count));
                if invokes {
                    for _ in 0..state.character.orb_slots.len(){enqueue(state,source.clone(),CharacterEffect::EvokeFront{remove:true});}
                } else {state.character.orb_slots.fill(None);}
            }
        }
        CharacterEffect::Lightning{amount}=>lightning(state,source,amount)?,
        CharacterEffect::OrbEnergy(amount)=>{state.player.energy=state.player.energy.saturating_add(amount).min(999);}
    }
    Ok(())
}

pub(super) fn apply_focus(state:&mut super::model::State,delta:i32)->Result<(),super::InputError>{
    state.character.focus=state.character.focus.saturating_add(delta).clamp(-999,999);
    Ok(())
}

pub(super) fn change_slots(state:&mut super::model::State,delta:i32)->Result<(),super::InputError>{
    if state.character.orb_slots.len()>10 {return Err(super::InputError::Invariant("orb capacity exceeds ten".into()))}
    let capacity=(state.character.orb_slots.len() as i64+i64::from(delta)).clamp(0,10) as usize;
    state.character.orb_slots.resize(capacity,None);Ok(())
}

pub(super) fn channel(state:&mut super::model::State,source:&Source,orb:OrbState,evict:bool)->Result<(),super::EffectError>{
    if (orb.kind==OrbKind::Dark)!=orb.dark_damage.is_some() {
        return Err(super::InputError::Invariant("orb payload ownership".into()).into())
    }
    if state.character.orb_slots.len()>10 {return Err(super::InputError::Invariant("orb capacity exceeds ten".into()).into())}
    if let Some(slot)=state.character.orb_slots.iter_mut().find(|slot|slot.is_none()) {*slot=Some(orb);}
    else if evict&&!state.character.orb_slots.is_empty() {
        enqueue(state,source.clone(),CharacterEffect::ChannelState{orb,evict});
        enqueue(state,source.clone(),CharacterEffect::EvokeFront{remove:true});
    }
    Ok(())
}

fn evoke(state:&mut super::model::State,source:&Source,index:usize,remove:bool)->Result<(),super::EffectError>{
    let Some(orb)=state.character.orb_slots.get(index).copied().flatten() else{return Ok(())};
    match orb.kind {
        OrbKind::Frost=>state.pending.push_front(Effect{source:source.clone(),target:Some(Actor::Player),kind:EffectKind::GainBlock((5+state.character.focus).max(0))}),
        OrbKind::Plasma=>enqueue(state,source.clone(),CharacterEffect::OrbEnergy(2)),
        OrbKind::Lightning=>enqueue(state,source.clone(),CharacterEffect::Lightning{amount:(8+state.character.focus).max(0)}),
        OrbKind::Dark=>{
            // Dark's local selector deliberately includes half-dead actors. Ties
            // retain source slot order and do not consume random words.
            let target=state.enemies.iter().filter(|e|e.life==super::ActorLife::HalfDead||(e.life==super::ActorLife::Active&&e.hp>0))
                .enumerate().min_by_key(|(index,e)|(e.hp,*index)).map(|(_,e)|e.instance_id.clone());
            if let Some(target)=target {
                state.pending.push_front(Effect{source:source.clone(),target:Some(Actor::Enemy(target)),
                    kind:EffectKind::Damage(DamagePayload::Orb(orb.dark_damage.expect("validated Dark")))});
            }
        }
    }
    if remove {state.character.orb_slots.remove(index);state.character.orb_slots.push(None);}
    Ok(())
}

fn cables(state:&super::model::State)->bool {
    state.item_context.as_ref().is_some_and(|items|items.owns_relic(crate::content::RelicId::GoldPlatedCables))
}

pub(super) fn passive_group(state:&mut super::model::State,source:&Source,group:OrbPassiveGroup)->Result<(),super::EffectError>{
    let phases:&[OrbPassivePhase]=match group {
        OrbPassiveGroup::NaturalStart=>&[OrbPassivePhase::Start],
        OrbPassiveGroup::NaturalEnd=>&[OrbPassivePhase::End],
        _=>&[OrbPassivePhase::Start,OrbPassivePhase::End],
    };
    let mut work=Vec::new();
    for index in 0..state.character.orb_slots.len(){
        if group==OrbPassiveGroup::Dark&&!state.character.orb_slots[index].is_some_and(|orb|orb.kind==OrbKind::Dark){continue}
        for phase in phases{if let Some(effect)=passive_work(state,source,index,*phase)?{work.push(effect);}}
    }
    let extra=cables(state)&&state.character.orb_slots.first().copied().flatten()
        .is_some_and(|orb|group!=OrbPassiveGroup::Dark||orb.kind==OrbKind::Dark);
    if extra{for phase in phases{if let Some(effect)=passive_work(state,source,0,*phase)?{work.push(effect);}}}
    // Queued Impulse/DarkImpulse invoke hooks now; their child actions are
    // addToBottom work and can follow the already queued ordinary draw. Natural
    // end hooks instead finish before the shared monster-turn continuation.
    if matches!(group,OrbPassiveGroup::All|OrbPassiveGroup::Dark){
        for effect in work{super::enemies::act2::bottom(state,effect);}
    }else{for effect in work.into_iter().rev(){state.pending.push_front(effect);}}
    Ok(())
}

pub(super) fn passive_work(state:&mut super::model::State,source:&Source,index:usize,phase:OrbPassivePhase)->Result<Option<Effect>,super::EffectError>{
    let Some(orb)=state.character.orb_slots.get(index).copied().flatten() else{return Ok(None)};
    let kind=match (orb.kind,phase) {
        (OrbKind::Dark,OrbPassivePhase::End)=>{
            state.character.orb_slots[index].as_mut().unwrap().dark_damage=Some(orb.dark_damage.expect("validated Dark").wrapping_add((6+state.character.focus).max(0)));return Ok(None)
        }
        (OrbKind::Frost,OrbPassivePhase::End)=>EffectKind::GainBlock((2+state.character.focus).max(0)),
        (OrbKind::Lightning,OrbPassivePhase::End)=>EffectKind::Character(CharacterEffect::Lightning{amount:(3+state.character.focus).max(0)}),
        (OrbKind::Plasma,OrbPassivePhase::Start)=>EffectKind::Character(CharacterEffect::OrbEnergy(1)),
        _=>return Ok(None),
    };
    Ok(Some(Effect{source:source.clone(),target:Some(Actor::Player),kind}))
}
pub(super) fn invoke_passive(state:&mut super::model::State,source:&Source,index:usize,phase:OrbPassivePhase)->Result<(),super::EffectError>{
    if let Some(work)=passive_work(state,source,index,phase)?{state.pending.push_front(work);}Ok(())
}

fn lightning(state:&mut super::model::State,source:&Source,amount:i32)->Result<(),super::EffectError>{
    let targets=state.enemies.iter().filter(|e|e.hp>0&&e.life==super::ActorLife::Active).map(|e|e.instance_id.clone()).collect::<Vec<_>>();
    if targets.is_empty(){return Ok(())}
    let all=state.player_full_powers.iter().any(|p|p.id.0=="electrodynamics"&&p.state.amount>0);
    let selected=if all {targets} else {
        let stream=state.card_rng.as_ref().ok_or_else(||super::InputError::Invariant("card random stream missing".into()))?;
        let (index,next)=if targets.len()==1 {let (_,next)=super::rng::draw_u64(stream)?;(0,next)}
            else {super::rng::uniform_below(stream,targets.len() as u128)?};
        state.card_rng=Some(next);vec![targets[index as usize].clone()]
    };
    for target in selected.into_iter().rev(){
        state.pending.push_front(Effect{source:source.clone(),target:Some(Actor::Enemy(target)),kind:EffectKind::Damage(DamagePayload::Orb(amount))});
    }
    Ok(())
}
