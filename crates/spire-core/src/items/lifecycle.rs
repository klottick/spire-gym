//! Relic lifecycle types (M27 design §7.1). Task M2 creates the types the lane stubs use;
//! Task M4 implements the obtain, loss and selection drivers behind them without editing
//! any lane file.
#![allow(dead_code)]
use crate::content::{PotionId, RelicId, RelicTier};
use crate::run::{ChoiceId, ContinuationTarget, EmptySelectionRule, ItemAdmission, RoomSelectionOperation, Run, RunError, RunState, RunTransition, SelectionCandidate};

/// Where an obtained relic comes from (plan Task M4; pool spawn conditions read it).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RelicSource { Reward, Elite, Boss, Shop, Chest, Event, Neow, Dig, Equip(RelicId) }

/// A card selection an equip opens (Empty Cage, Astrolabe, the bottles, …).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct SelectionSpec {
    /// Eligible permanent card instances, in deck order.
    pub eligible: Vec<u64>,
    /// Cards to choose.
    pub count: usize,
    /// Whether the player may choose none.
    pub cancellable: bool,
}

/// Offers an equip opens through the reward owner (Calling Bell, Orrery, Tiny House, Cauldron).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct RewardSpec {
    pub card_rewards: usize,
    pub relic_tiers: Vec<RelicTier>,
    pub potions: Vec<PotionId>,
    pub gold: u32,
}

/// Work an equip queues for the lifecycle driver.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct EquipWork {
    pub(crate) selections: Vec<SelectionSpec>,
    pub(crate) offers: Vec<RewardSpec>,
}

impl EquipWork {
    pub(crate) fn select(&mut self, spec: SelectionSpec) { self.selections.push(spec); }
    pub(crate) fn offer(&mut self, spec: RewardSpec) { self.offers.push(spec); }
}

/// The relic and the child work produced by its obtain-time owner.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct RelicObtained { pub selection:Option<SelectionSpec>, pub offers:Vec<RewardSpec> }

/// Acquire a relic through the Run's admission profile and its single equip owner.
pub(crate) fn obtain_relic(state:&mut RunState,id:RelicId,source:RelicSource)->Result<RelicObtained,RunError>{
    let _source=source;
    if state.item_admission==ItemAdmission::Profile {
        super::relics::acquire(state,id)?;
        return Ok(RelicObtained::default())
    }
    if state.combat.is_some()||state.active_combat.is_some(){return Err(RunError::InvalidAction)}
    // F7: the exhausted Boss-pool fallback Red Circlet is obtainable outside the ordinary relic union.
    if !crate::content::complete::relic_union(state.character).contains(&id)&&id!=RelicId::RedCirclet{return Err(RunError::InvalidAction)}
    if id!=RelicId::Circlet&&state.relics.contains(&id){return Err(RunError::InvalidAction)}

    let mut staged=state.clone();
    if id==RelicId::Circlet&&staged.relics.contains(&id){
        let counter=staged.relic_counters.entry(id).or_insert(1);
        *counter=counter.checked_add(1).ok_or(RunError::Unsupported("Circlet counter overflow"))?;
        *state=staged;
        return Ok(RelicObtained::default())
    }
    super::relics::acquire(&mut staged,id)?;
    // Design §7.4: an obtained relic leaves every pool tier (sources outside the pools too).
    if let Some(pools)=staged.relic_pools.as_mut(){super::relic_pools::remove(pools,id);}
    let mut work=EquipWork::default();
    super::dispatch::equip(id,&mut staged,&mut work)?;
    *state=staged;
    Ok(RelicObtained{selection:work.selections.into_iter().next(),offers:work.offers})
}

pub(crate) fn after_obtain(run:&mut Run,choice:ChoiceId,relic:RelicId,obtained:RelicObtained)->Result<RunTransition,RunError>{
    crate::run::rooms::after_obtain(run,choice,relic,obtained)
}

pub(crate) fn neow_boss_swap(state:&mut RunState,key:&[u64])->Result<RelicId,RunError>{
    neow_boss_swap_obtained(state,key).map(|(relic,_)|relic)
}

/// As `neow_boss_swap`, also returning the obtain's equip continuation for `after_obtain`
/// (the M27-carried Neow boss-swap equip offer; M28 design §6.5).
pub(crate) fn neow_boss_swap_obtained(state:&mut RunState,key:&[u64])->Result<(RelicId,RelicObtained),RunError>{
    if state.item_admission!=ItemAdmission::Complete||state.combat.is_some()||state.active_combat.is_some()||state.pending.is_some(){
        return Err(RunError::InvalidAction)
    }
    let starter=*state.relics.first().ok_or(RunError::InvalidAction)?;
    let mut staged=state.clone();
    lose_relic(&mut staged,starter)?;
    let relic=super::relic_pools::take(&mut staged,RelicTier::Boss,super::relic_pools::Selector::Front,RelicSource::Neow,key)?;
    let obtained=obtain_relic(&mut staged,relic,RelicSource::Neow)?;
    *state=staged;
    Ok((relic,obtained))
}

/// Complete-profile relic loss is one atomic inventory mutation; Profile has no loss path.
pub(crate) fn lose_relic(state:&mut RunState,id:RelicId)->Result<(),RunError>{
    if state.item_admission==ItemAdmission::Profile{return Err(RunError::InvalidAction)}
    if state.combat.is_some()||state.active_combat.is_some()||!state.relics.contains(&id){return Err(RunError::InvalidAction)}
    let mut staged=state.clone();
    if id==RelicId::SacredBark {
        staged.potion_potencies=super::main_relics::reconcile_potion_potencies(&staged.relics,&staged.potions,&staged.potion_potencies);
    }
    on_unequip(&mut staged,id);
    staged.relics.retain(|owned|*owned!=id);
    staged.relic_counters.remove(&id);
    staged.relic_cards.remove(&id);
    *state=staged;
    Ok(())
}

/// Source `AbstractPlayer.loseRelic` runs the relic's `onUnequip` first (PM decompile, m28f
/// review-fixes I1); every loss (N'loth, Golden Idol, Neow's boss swap) comes through `lose_relic`.
/// The non-empty overrides need no state for energy or hand size: the `energyMaster` −1 relics
/// (Cursed Key, Mark of Pain, Philosopher's Stone, Runic Dome, Busted Crown, Coffee Dripper,
/// Ectoplasm, Fusion Hammer, Sozu, Velvet Choker) and `masterHandSize` (Snecko Eye −2, Ring of the
/// Serpent −1) are derived from the relics owned at each combat entry. The bottles' flag is their
/// `relic_cards` mapping, dropped by the caller. Every other relic's `onUnequip` is empty.
fn on_unequip(state:&mut RunState,id:RelicId){
    // Necronomicon: the first Necronomicurse leaves by a direct `group.remove`, so no removal hook
    // (its returned copy) runs.
    if id==RelicId::Necronomicon{
        if let Some(index)=state.deck.iter().position(|card|card.id==crate::content::CardId::Necronomicurse){state.deck.remove(index);}
    }
}

/// Open the shared room-continuation selector for an equip that needs card instances.
pub(crate) fn open_equip_selection(run:&mut Run,choice:ChoiceId,relic:RelicId,spec:SelectionSpec,after:Vec<RewardSpec>)->Result<RunTransition,RunError>{
    if run.state().item_admission!=ItemAdmission::Complete||!run.state().relics.contains(&relic)
        ||spec.eligible.len()<=spec.count||spec.count==0{return Err(RunError::InvalidAction)}
    let node=crate::run::rooms::selection_node(run.state())?;
    let mut seen=std::collections::BTreeSet::new();
    let mut staged=run.clone();
    let selection_ids=crate::run::rooms::reserve_selection_ids(&mut staged,spec.eligible.len())?;
    let candidates=spec.eligible.iter().zip(selection_ids).map(|(instance,id)|{
        if !seen.insert(*instance){return Err(RunError::InvalidAction)}
        let card=staged.state().deck.iter().find(|card|card.instance==*instance).cloned().ok_or(RunError::InvalidAction)?;
        Ok(SelectionCandidate{id,card})
    }).collect::<Result<Vec<_>,RunError>>()?;
    crate::run::rooms::defer_equip_offers(&mut staged,choice,relic,after)?;
    staged.begin_selection(ContinuationTarget::Room{node,choice,operation:RoomSelectionOperation::RelicEquip{relic}},
        candidates,spec.count.min(spec.eligible.len()),EmptySelectionRule::Fault)?;
    *run=staged;
    Ok(RunTransition{reward:0,terminal:None,committed_events:vec![]})
}
