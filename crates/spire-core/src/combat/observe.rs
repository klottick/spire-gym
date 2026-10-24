//! Detached, opt-in observations; never part of cloned gameplay state.
use super::*;
use super::model::State;
use std::{cell::RefCell, marker::PhantomData, rc::Rc};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Observed<T> { pub result:T, pub effects:Vec<ResolvedEffect>, pub boundaries:Vec<CombatBoundary> }
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedEffect { pub sequence:u32, pub source:Source, pub target:Option<Actor>, pub kind:ResolvedEffectKind }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DamageCategory { Normal, Orb, Thorns, HpLoss }
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResolvedEffectKind {
    Damage { incoming:i32, absorbed_block:i32, hp_before:i32, hp_after:i32, hp_lost:i32, overkill:Option<i32>, category:DamageCategory },
    Heal { hp_before:i32, hp_after:i32, amount:i32 },
    Block { before:i32, after:i32 },
    Energy { before:i32, after:i32 },
    Power { id:FullPowerId, before:Option<i32>, after:Option<i32> },
    EnemyMove { actor:Actor, move_id:ActorIntentId },
    LifeChange { actor:Actor, before:Option<PublicLife>, after:PublicLife },
    CardMove { card:CardHandle, before:PublicZone, after:PublicZone },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PublicLife { Active, Defeated, Replaced, Escaped, HalfDead }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PublicZone { Unknown, Hand, Discard, Exhaust, InPlay, Stasis }
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PublicCombatSnapshot { Legacy(PublicView), Native(CombatPublic) }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CombatBoundaryKind { Entry, Selection, Terminal }
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CombatBoundary { pub kind:CombatBoundaryKind, pub public:PublicCombatSnapshot, pub parent_card:Option<CardHandle> }

struct Collector { owner:usize, run:Option<usize>, run_state:Option<usize>, entry:bool, effects:Vec<ResolvedEffect>, boundaries:Vec<CombatBoundary> }
thread_local! { static CAPTURE:RefCell<Option<Collector>> = const { RefCell::new(None) }; }

// Guards cannot migrate to a different thread's collector. Addresses are only
// compared as identities; no pointer is ever dereferenced.
pub(crate) struct Session { previous:Option<Collector>, active:bool, _local:PhantomData<Rc<()>> }
impl Session {
    pub(crate) fn combat(combat: &Combat) -> Self {
        Self::start(identity(&combat.state), None,None)
    }
    pub(crate) fn run(owner:usize,state:usize) -> Self { Self::start(0,Some(owner),Some(state)) }
    fn start(owner:usize,run:Option<usize>,run_state:Option<usize>) -> Self {
        let collector = Collector { owner, run, run_state, entry:false, effects:Vec::new(), boundaries:Vec::new() };
        let previous = CAPTURE.with(|slot| slot.replace(Some(collector)));
        Self { previous, active:true, _local:PhantomData }
    }
    pub(crate) fn finish<T>(mut self, result:T) -> Observed<T> {
        let collector = CAPTURE.with(|slot| slot.replace(self.previous.take())).expect("active observer session");
        self.active=false;
        Observed { result, effects:collector.effects, boundaries:collector.boundaries }
    }
}
impl Drop for Session {
    fn drop(&mut self) { if self.active { CAPTURE.with(|slot| { slot.replace(self.previous.take()); }); } }
}
pub(crate) struct Binding { previous:Option<usize>, _local:PhantomData<Rc<()>> }
impl Drop for Binding {
    fn drop(&mut self) {
        if let Some(previous)=self.previous { CAPTURE.with(|slot| {
            if let Some(c)=slot.borrow_mut().as_mut() { c.owner=previous; }
        }); }
    }
}
pub(crate) fn bind_run(run:usize,combat:&Combat)->Binding {
    let previous=CAPTURE.with(|slot| {
        let mut slot=slot.borrow_mut();
        let c=slot.as_mut().filter(|c|c.run==Some(run))?;
        Some(std::mem::replace(&mut c.owner,identity(&combat.state)))
    });
    Binding { previous,_local:PhantomData }
}
pub(crate) struct EntryPermit { previous:Option<bool>, _local:PhantomData<Rc<()>> }
impl Drop for EntryPermit {
    fn drop(&mut self) {
        if let Some(previous)=self.previous { CAPTURE.with(|slot| {
            if let Some(c)=slot.borrow_mut().as_mut() { c.entry=previous; }
        }); }
    }
}
pub(crate) fn permit_entry(run:usize)->EntryPermit {
    let previous=CAPTURE.with(|slot| {
        let mut slot=slot.borrow_mut();
        let c=slot.as_mut().filter(|c|c.run==Some(run))?;
        Some(std::mem::replace(&mut c.entry,true))
    });
    EntryPermit { previous,_local:PhantomData }
}
pub(super) fn bind_entry(combat:&Combat)->Binding {
    let previous=CAPTURE.with(|slot| {
        let mut slot=slot.borrow_mut();
        let c=slot.as_mut().filter(|c|c.entry)?;
        Some(std::mem::replace(&mut c.owner,identity(&combat.state)))
    });
    Binding { previous,_local:PhantomData }
}
fn identity(state:&State) -> usize { std::ptr::from_ref(state) as usize }
pub(super) fn enabled(state:&State) -> bool {
    CAPTURE.with(|slot| slot.borrow().as_ref().is_some_and(|c| c.owner == identity(state)))
}
pub(super) fn emit(state:&State, source:&Source, target:Option<&Actor>, kind:impl FnOnce()->ResolvedEffectKind) {
    CAPTURE.with(|slot| {
        let mut slot = slot.borrow_mut();
        if let Some(c) = slot.as_mut().filter(|c| c.owner == identity(state)) {
            c.effects.push(ResolvedEffect { sequence:c.effects.len() as u32, source:source.clone(),
                target:target.cloned(), kind:kind() });
        }
    });
}
pub(crate) fn run_heal(state:&crate::run::RunState,before:u32) {
    run_heal_values(state,before,state.hp,&Source::None);
}
pub(crate) fn run_enabled(state:&crate::run::RunState)->bool {
    CAPTURE.with(|slot|slot.borrow().as_ref().is_some_and(|c|c.run_state==Some(std::ptr::from_ref(state) as usize)))
}
pub(crate) fn run_heal_values(state:&crate::run::RunState,before:u32,after:u32,source:&Source) {
    CAPTURE.with(|slot| {
        if let Some(c)=slot.borrow_mut().as_mut().filter(|c|c.run_state==Some(std::ptr::from_ref(state) as usize)) {
            c.effects.push(ResolvedEffect {sequence:c.effects.len() as u32,source:source.clone(),target:Some(Actor::Player),
                kind:ResolvedEffectKind::Heal {hp_before:before as i32,hp_after:after as i32,amount:after as i32-before as i32}});
        }
    });
}
pub(crate) fn boundary(combat:&Combat) {
    if !enabled(&combat.state) { return; }
    let kind = match combat.state.phase {
        Phase::Selection => CombatBoundaryKind::Selection,
        Phase::Terminal => CombatBoundaryKind::Terminal,
        _ => return,
    };
    capture_boundary(combat,kind);
}
pub(super) fn capture_boundary(combat:&Combat,kind:CombatBoundaryKind) {
    if !enabled(&combat.state) {return;}
    let public = if combat.state.full_runtime {
        combat.public_full().ok().map(PublicCombatSnapshot::Native)
    } else { combat.public_view().ok().map(PublicCombatSnapshot::Legacy) };
    if let Some(public) = public {
        let boundary = CombatBoundary { kind, public,
            parent_card:combat.state.pending_selection.as_ref().map(|s|s.parent.clone()) };
        CAPTURE.with(|slot| { slot.borrow_mut().as_mut().unwrap().boundaries.push(boundary); });
    }
}

// Only enabled observation copies these public primitive values. Draw contents,
// pending work, RNG and future selections are deliberately absent.
pub(super) struct Mark { actors:Vec<ActorMark>, energy:i32, cards:std::collections::BTreeMap<CardHandle,PublicZone> }
struct ActorMark { actor:Actor, block:i32, powers:Vec<(FullPowerId,i32)>, life:PublicLife }
fn life(hp:i32,life:ActorLife)->PublicLife {
    match life { ActorLife::Replaced=>PublicLife::Replaced,ActorLife::Escaped=>PublicLife::Escaped,ActorLife::HalfDead=>PublicLife::HalfDead,
        ActorLife::Active if hp<=0=>PublicLife::Defeated,ActorLife::Active=>PublicLife::Active }
}
fn values(state:&State)->Mark {
    let powers=|legacy:&[PowerEntry],full:&[FullPowerEntry]| legacy.iter().map(|p|(FullPowerId(p.id.as_str()),p.state.amount))
        .chain(full.iter().map(|p|(p.id,p.state.amount))).collect();
    let mut actors=vec![ActorMark { actor:Actor::Player,block:state.player.block,
        powers:powers(&state.player.powers,&state.player_full_powers),life:life(state.player.hp,ActorLife::Active) }];
    actors.extend(state.enemies.iter().map(|e|ActorMark { actor:Actor::Enemy(e.instance_id.clone()),block:e.block,
        powers:powers(&e.powers,&e.full_powers),life:life(e.hp,e.life) }));
    let mut cards=std::collections::BTreeMap::new();
    for (pile,zone) in [(&state.piles.hand,PublicZone::Hand),(&state.piles.discard,PublicZone::Discard),(&state.piles.exhaust,PublicZone::Exhaust)] {
        cards.extend(pile.iter().map(|c|(c.handle(),zone)));
    }
    if let Some(card)=&state.in_play {cards.insert(card.handle(),PublicZone::InPlay);}
    cards.extend(state.stasis.values().map(|c|(c.handle(),PublicZone::Stasis)));
    Mark { actors,energy:state.player.energy,cards }
}
pub(super) fn mark(state:&State)->Option<Mark> { enabled(state).then(||values(state)) }
pub(super) fn changes(state:&State,before:Option<Mark>,source:&Source) {
    let Some(before)=before else {return};
    let after=values(state);
    for a in &after.actors {
        let b=before.actors.iter().find(|b|b.actor==a.actor);
        if let Some(b)=b {
            if a.block!=b.block {emit(state,source,Some(&a.actor),||ResolvedEffectKind::Block {before:b.block,after:a.block});}
        }
        if b.map(|b|b.life)!=Some(a.life) {emit(state,source,Some(&a.actor),||ResolvedEffectKind::LifeChange {actor:a.actor.clone(),before:b.map(|b|b.life),after:a.life});}
        let old=b.map_or(&[][..],|b|b.powers.as_slice());
        let ids:std::collections::BTreeSet<_>=old.iter().chain(&a.powers).map(|(id,_)|*id).collect();
        for id in ids {
            let before=old.iter().find(|(i,_)|*i==id).map(|(_,n)|*n);
            let after=a.powers.iter().find(|(i,_)|*i==id).map(|(_,n)|*n);
            if before!=after {emit(state,source,Some(&a.actor),||ResolvedEffectKind::Power {id,before,after});}
        }
    }
    if before.energy!=after.energy {emit(state,source,Some(&Actor::Player),||ResolvedEffectKind::Energy {before:before.energy,after:after.energy});}
    let ids:std::collections::BTreeSet<_>=before.cards.keys().chain(after.cards.keys()).collect();
    for card in ids {
        let b=before.cards.get(card).copied().unwrap_or(PublicZone::Unknown);
        let a=after.cards.get(card).copied().unwrap_or(PublicZone::Unknown);
        if a!=b {emit(state,source,None,||ResolvedEffectKind::CardMove {card:card.clone(),before:b,after:a});}
    }
}
impl Combat {
    pub fn step_observed(&mut self, action:&Action) -> Observed<Result<Transition,StepError>> {
        let session = Session::combat(self);
        let result = self.step(action);
        if result.is_ok() { boundary(self); }
        session.finish(result)
    }
    pub fn step_full_observed(&mut self, action:&CombatAction) -> Observed<Result<(),crate::run::RunError>> {
        let session = Session::combat(self);
        let result = self.step_full(action);
        if result.is_ok() { boundary(self); }
        session.finish(result)
    }
}
