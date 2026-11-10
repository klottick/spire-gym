//! Item-owned mechanics. Effects returned here are ordinary work for the shared
//! combat resolver; this module does not own or drain a second effect queue.
mod relics;
mod potions;
mod ironclad;
mod ironclad_turns;
mod silent;
pub(crate) mod dispatch;
pub(crate) mod laws;
pub(crate) mod run_laws;
pub(crate) mod lifecycle;
pub(crate) mod relic_effects;
pub(crate) use lifecycle::{lose_relic, obtain_relic, open_equip_selection, RelicObtained};
pub(crate) mod relic_pools;

#[cfg(test)]
pub(crate) mod tests;
#[cfg(test)]
mod m10_ironclad_relic_turns;
#[cfg(test)]
mod m15_silent_items;

use crate::{combat::{Combat, Effect, EnemyHandle, Hook, InputError}, content::{Character, PotionId, RelicId}, run::{RunError, RunState}};

#[derive(Clone,Debug,PartialEq,Eq)]
pub struct RelicCounterView {pub relic:RelicId,pub value:i32}
pub(crate) fn visible_counters(counters:&std::collections::BTreeMap<RelicId,i32>)->Vec<RelicCounterView>{
    [RelicId::NeowsLament,RelicId::Inserter].into_iter().filter_map(|relic|counters.get(&relic).map(|value|RelicCounterView{relic,value:*value})).collect()
}
pub(crate) fn complete_counter_view(counters:&std::collections::BTreeMap<RelicId,i32>)->Vec<RelicCounterView>{
    [RelicId::NeowsLament,RelicId::Inserter,RelicId::AncientTeaSet,RelicId::HappyFlower,RelicId::MawBank,
        RelicId::Nunchaku,RelicId::Omamori,RelicId::PenNib,RelicId::TinyChest,RelicId::InkBottle,
        RelicId::Matryoshka,RelicId::Sundial,RelicId::Girya,RelicId::IncenseBurner,RelicId::LizardTail,
        RelicId::WingBoots,RelicId::Circlet,RelicId::NlothsHungryFace]
        .into_iter().filter_map(|relic|counters.get(&relic).map(|value|RelicCounterView{relic,value:*value})).collect()
}
pub(crate) fn replace_burning_blood(state:&mut RunState)->Result<(),RunError>{relics::replace_burning_blood(state)}

/// Item ownership is transferred into active full combat and returned with its
/// exit payload. Outside combat, RunState remains the permanent authority.
#[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]
pub enum CombatStartupProfile{#[default] Legacy,M21Watcher}
#[derive(Clone, Default, PartialEq, Eq)]
pub struct CombatItemContext {
    pub(crate) battle:NativeBattleContext,
    startup_profile:CombatStartupProfile,
    watcher_progression:bool,
    /// M24 Watcher Acts1–3: the progression union also admits Red Mask (Tomb).
    watcher_act3:bool,
    /// M25 Watcher Acts 1–4 and Heart (`with_m25_profile`).
    watcher_heart:bool,
    /// M26 complete card capability overlay (`with_complete_cards`).
    complete_cards:bool,
    /// M30 G4 (PM ruling W1): the entering Run's master deck has a real acquisition order, so combat
    /// entry shuffles it as is and skips the finite M21–M25 Watcher canonical sort. Set by the Run
    /// owner for every Complete-admission Run (the Standard Run; the M26/M27 supplied diagnostics,
    /// whose decks are pre-sorted by that key, so their order is unchanged); combat entry clears it.
    master_order:bool,
    /// M27 complete item capability overlay (`with_complete_items`); valid only with
    /// `complete_cards` (design §5.2).
    complete_items:bool,
    /// Combat-local relic state (design §7.5), dropped at settlement.
    local:std::collections::BTreeMap<RelicId,i32>,
    /// Bottled relic card links moved with the inventory (design §7.5).
    relic_cards:std::collections::BTreeMap<RelicId,u64>,
    potion_potencies:Vec<Option<i32>>,
    last_potion_use:Option<(usize,PotionId,i32)>,
    pub(crate) collar_activated:bool,
    pub(crate) emotion_chip_armed:bool,
    counters: std::collections::BTreeMap<RelicId,i32>,
    relics: Vec<RelicId>,
    potions: Vec<Option<PotionId>>,
    pub(crate) orichalcum_trigger: bool,
    runic_capacitor_armed: bool,
    kite_triggered: bool,
    wrist_blade_bonus: i32,
    damage_card: Option<dispatch::PhaseCard>,
    lantern_armed: bool,
    heal_room: NativeHealRoom,
    /// M31 (design §6.2): the owning Run's ascension, read only for its potion base
    /// (`run::potion_base_slots`). `new` and `Default` give 0 (base 3, the finite profiles).
    ascension: u8,
}

#[derive(Clone,Copy,Default,PartialEq,Eq)]
pub(crate) struct NativeBattleContext {pub elite_trigger:bool,pub boss:bool,pub burning:Option<crate::combat::enemies::BurningBuff>,pub m15:bool,pub m20:bool}

impl std::fmt::Debug for NativeBattleContext{
    fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result{
        let mut value=f.debug_struct("NativeBattleContext");
        value.field("elite_trigger",&self.elite_trigger).field("boss",&self.boss).field("burning",&self.burning);
        if self.m15{value.field("m15",&self.m15);}if self.m20{value.field("m20",&true);}value.finish()
    }
}
impl std::fmt::Debug for CombatItemContext{
    fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result{
        let mut value=f.debug_struct("CombatItemContext");
        value.field("battle",&self.battle).field("collar_activated",&self.collar_activated)
            .field("counters",&self.counters).field("relics",&self.relics).field("potions",&self.potions)
            .field("orichalcum_trigger",&self.orichalcum_trigger).field("lantern_armed",&self.lantern_armed).field("heal_room",&self.heal_room);
        if self.runic_capacitor_armed{value.field("runic_capacitor_armed",&true);}
        if self.emotion_chip_armed{value.field("emotion_chip_armed",&self.emotion_chip_armed);}
        if self.kite_triggered{value.field("kite_triggered",&self.kite_triggered);}
        if self.wrist_blade_bonus!=0{value.field("wrist_blade_bonus",&self.wrist_blade_bonus);}
        if self.damage_card.is_some(){value.field("damage_card",&self.damage_card);}
        if self.m22_profile(){value.field("watcher_progression",&true);}
        if self.watcher_act3{value.field("watcher_act3",&true);}
        if self.watcher_heart{value.field("watcher_heart",&true);}
        if self.complete_cards{value.field("complete_cards",&true);}
        if self.master_order{value.field("master_order",&true);}
        if self.complete_items{value.field("complete_items",&true);}
        if self.ascension!=0{value.field("ascension",&self.ascension);}
        if !self.local.is_empty(){value.field("local",&self.local);}
        if !self.relic_cards.is_empty(){value.field("relic_cards",&self.relic_cards);}
        if self.m21_profile(){value.field("startup_profile",&self.startup_profile);}
        if self.m21_profile() || self.potion_potencies.iter().any(Option::is_some) || self.last_potion_use.is_some(){
            value.field("potion_potencies",&self.potion_potencies).field("last_potion_use",&self.last_potion_use);
        }
        value.finish()
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct RelicHookWork {
    pub top: Vec<Effect>,
    pub bottom: Vec<Effect>,
    pub direct_strength: i32,
}

impl RelicHookWork {
    pub(crate) fn is_empty(&self) -> bool { self.top.is_empty() && self.bottom.is_empty() && self.direct_strength==0 }
}

impl CombatItemContext {
    /// Damage-path rescue consumes the first live Fairy without potion-use hooks.
    pub(crate) fn consume_fairy_for_rescue(&mut self,max_hp:i32)->Option<(usize,i32)> {
        if !(self.shared_generated_profile()||self.complete_items())||self.owns_relic(RelicId::MarkOfBloom){return None}
        let slot=self.potions.iter().position(|potion|*potion==Some(PotionId::FairyPotion))?;
        let percent=if self.complete_items(){self.potion_potency(slot).unwrap_or(30)}
            else if self.owns_relic(RelicId::SacredBark){60}else{30};
        let amount=((max_hp as f32)*(percent as f32/100.0_f32)) as i32;
        self.potions[slot]=None;
        if let Some(potency)=self.potion_potencies.get_mut(slot){*potency=None;}
        Some((slot,amount.max(1)))
    }

    pub fn new(relics: Vec<RelicId>, potions: Vec<Option<PotionId>>) -> Self {
        let mut counters=std::collections::BTreeMap::new();
        if relics.contains(&RelicId::NeowsLament){counters.insert(RelicId::NeowsLament,3);}
        if relics.contains(&RelicId::Inserter){counters.insert(RelicId::Inserter,0);}
        Self { potion_potencies:vec![],last_potion_use:None,startup_profile:CombatStartupProfile::Legacy,watcher_progression:false,watcher_act3:false,watcher_heart:false,complete_cards:false,master_order:false,complete_items:false,local:Default::default(),relic_cards:Default::default(),counters, relics, potions, runic_capacitor_armed:false,orichalcum_trigger:false, kite_triggered:false,wrist_blade_bonus:0,damage_card:None,
            lantern_armed: false, heal_room: NativeHealRoom::default(),battle:NativeBattleContext::default(),collar_activated:false,emotion_chip_armed:false,ascension:0 }
    }

    /// Move permanent inventory into the single active combat owner.
    pub(crate) fn take_from_run(state: &mut RunState) -> Self {
        let complete_items=state.item_admission==crate::run::ItemAdmission::Complete;
        let relics=std::mem::take(&mut state.relics);
        let potions=std::mem::take(&mut state.potions);
        let stored=std::mem::take(&mut state.potion_potencies);
        let run_potencies=if complete_items{main_relics::reconcile_potion_potencies(&relics,&potions,&stored)}else{Vec::new()};
        let potion_potencies=if complete_items {
            if run_potencies.is_empty(){vec![None;potions.len()]}
            else{run_potencies.iter().map(|slot|slot.map(|(_,potency)|potency)).collect()}
        }else{Vec::new()};
        Self { potion_potencies,last_potion_use:None,startup_profile:CombatStartupProfile::Legacy,watcher_progression:false,watcher_act3:false,watcher_heart:false,complete_cards:false,master_order:false,complete_items:false,local:Default::default(),relic_cards:std::mem::take(&mut state.relic_cards),runic_capacitor_armed:false,battle:NativeBattleContext::default(),collar_activated:false,emotion_chip_armed:false,counters:std::mem::take(&mut state.relic_counters),relics,
            potions, orichalcum_trigger:false,kite_triggered:false,wrist_blade_bonus:0,damage_card:None,lantern_armed: false,
            heal_room: NativeHealRoom::from_run(state),ascension:state.ascension }
    }

    /// Return active ownership once without replacing another live inventory.
    pub(crate) fn return_to_run(self, state: &mut RunState) -> Result<(), RunError> {
        if !state.relics.is_empty() || !state.potions.is_empty() || !state.relic_counters.is_empty() || !state.relic_cards.is_empty() || !state.potion_potencies.is_empty() {
            return Err(RunError::InvalidAction);
        }
        let returned_potencies=if self.complete_items {
            let bark=self.relics.contains(&RelicId::SacredBark);
            self.potions.iter().enumerate().map(|(slot,potion)|potion.map(|id|{
                let potency=self.potion_potencies.get(slot).copied().flatten()
                    .unwrap_or_else(||main_relics::initial_potion_potency(id,bark));
                (id,potency)
            })).collect()
        }else{Vec::new()};
        state.relics = self.relics;
        state.potions = self.potions;
        state.relic_counters=self.counters;
        state.relic_cards=self.relic_cards;
        state.potion_potencies=returned_potencies;
        if !state.potion_potencies.iter().any(Option::is_some){state.potion_potencies.clear();}
        Ok(())
    }

    pub(crate) fn with_startup_profile(mut self,profile:CombatStartupProfile)->Self{self.startup_profile=profile;self.fill_potion_potencies();self}
    /// Gives every held potion without a potency its initial one; stored potencies
    /// (Run custody, F4) are kept. Contexts that do not track potency hold none.
    fn fill_potion_potencies(&mut self){
        if !self.potion_potency_enabled(){self.potion_potencies.clear();return}
        self.potion_potencies.resize(self.potions.len(),None);
        for slot in 0..self.potions.len(){
            if let (Some(id),None)=(self.potions[slot],self.potion_potencies[slot]){self.potion_potencies[slot]=Some(self.initial_potency(id));}
        }
    }
    fn initial_potency(&self,id:PotionId)->i32{
        main_relics::initial_potion_potency(id,self.owns_relic(RelicId::SacredBark))
    }
    fn potion_potency_enabled(&self)->bool{self.m21_profile()||self.complete_items}
    /// The (slot, potion) of the latest use, where this profile records it (`None` = untracked).
    pub(crate) fn recorded_potion_use(&self)->Option<Option<(usize,PotionId)>>{
        self.potion_potency_enabled().then(||self.last_potion_use.map(|(slot,id,_)|(slot,id)))
    }
    pub(crate) fn potion_potency(&self,slot:usize)->Option<i32>{self.potion_potencies.get(slot).copied().flatten()}
    pub(crate) fn consumed_potion_potency(&self,slot:usize,id:PotionId)->Option<i32>{
        self.last_potion_use.filter(|(s,p,_)|*s==slot&&*p==id&&self.potions.get(slot)==Some(&None)).map(|(_,_,amount)|amount)
    }
    pub(crate) fn with_m22_profile(mut self)->Self{self.watcher_progression=true;self.with_startup_profile(CombatStartupProfile::M21Watcher)}
    /// Test-only: the M24 profile flag is the one intended item-context difference
    /// between matched M23 and M24 combats; prefix tests normalize exactly it.
    #[cfg(test)] pub(crate) fn mark_watcher_act3_for_test(&mut self){self.watcher_act3=true;}
    /// Everything `with_m22_profile` does, plus the M24 relic union (Red Mask).
    pub(crate) fn with_m24_profile(self)->Self{let mut context=self.with_m22_profile();context.watcher_act3=true;context}
    /// Everything `with_m22_profile` does, plus the M25 flag (the 23-relic M25 union);
    /// `watcher_act3` stays false.
    pub(crate) fn with_m25_profile(self)->Self{let mut context=self.with_m22_profile();context.watcher_heart=true;context}
    pub(crate) fn m25_profile(&self)->bool{self.watcher_heart}
    /// M26: overlay the complete card capability on an existing character context.
    /// Relic and potion validation stay the base context's.
    pub(crate) fn with_complete_cards(mut self)->Self{self.complete_cards=true;self}
    pub(crate) fn complete_cards(&self)->bool{self.complete_cards}
    /// M30 G4: see `master_order`.
    pub(crate) fn with_master_order(mut self)->Self{self.master_order=true;self}
    pub(crate) fn master_order(&self)->bool{self.master_order}
    /// Combat entry clears the flag on transfer, so it never survives into combat state.
    pub(crate) fn clear_master_order(&mut self){self.master_order=false;}
    /// M27 complete item capability (design §5.2). Validation rejects it without
    /// `with_complete_cards()`.
    pub(crate) fn with_complete_items(mut self)->Self{self.complete_items=true;self.fill_potion_potencies();self}
    pub(crate) fn complete_items(&self)->bool{self.complete_items}
    /// Selects the single pre-draw visit used by item dispatch.
    /// `Combat::enter_with_item_context` queues `Hook::CombatStartedPreDraw` iff
    /// the active item context satisfies `m21_profile()`; it queues that hook
    /// immediately before `OpeningHand`. Other contexts visit startup work with
    /// `hook == None` (`combat/engine.rs`, combat-entry opening sequence).
    pub(crate) fn is_pre_draw_visit(&self, hook: Option<crate::combat::Hook>) -> bool {
        if self.m21_profile() {
            hook == Some(crate::combat::Hook::CombatStartedPreDraw)
        } else {
            hook.is_none()
        }
    }
    /// Combat-local relic state (0 when unset).
    pub(crate) fn local(&self,id:RelicId)->i32{self.local.get(&id).copied().unwrap_or(0)}
    pub(crate) fn set_local(&mut self,id:RelicId,value:i32){if value==0{self.local.remove(&id);}else{self.local.insert(id,value);}}
    /// The permanent card instance a bottle relic is linked to.
    pub(crate) fn relic_card(&self,id:RelicId)->Option<u64>{self.relic_cards.get(&id).copied()}
    /// Bottled relic links in relic order (M34a a4 public marker).
    pub(crate) fn bottled_cards(&self)->Vec<(RelicId,u64)>{self.relic_cards.iter().map(|(relic,instance)|(*relic,*instance)).collect()}
    /// M26 §6.3 class L: an M25 Watcher card law is reachable under the M25 profile or the
    /// complete card capability (seam map §3; one helper, no per-site character list).
    pub(crate) fn m25_cards(&self)->bool{self.watcher_heart||self.complete_cards}
    pub(crate) fn m22_profile(&self)->bool{self.watcher_progression}
    pub(crate) fn m21_profile(&self)->bool{self.startup_profile==CombatStartupProfile::M21Watcher}
    pub(crate) fn owns_relic(&self, id: RelicId) -> bool { self.relics.contains(&id) }
    /// The owning Run's base potion slots (M31 design §6.2): 3, or 2 from A11.
    pub(crate) fn potion_base(&self) -> usize { crate::run::potion_base_slots(self.ascension) }
    pub(crate) fn valid_potion_capacity(&self) -> bool {
        valid_potion_capacity(&self.relics, &self.potions, self.potion_base(), self.complete_items())
    }
    pub(crate) fn validate_potion_obtain_input(&self, id: PotionId) -> Result<(), InputError> {
        validate_potion_obtain_input(&self.relics,&self.potions,self.potion_base(),self.complete_items(),id)
    }
    /// Ordinary first-empty-slot obtain. Sozu is checked by the accepted
    /// effect/run consumer at dispatch, never by this inventory owner.
    pub(crate) fn obtain_potion(&mut self, id: PotionId) -> Result<bool, InputError> {
        self.validate_potion_obtain_input(id)?;
        if let Some(slot) = self.potions.iter().position(Option::is_none) {
            self.potions[slot] = Some(id);
            if self.complete_items {
                self.potion_potencies.resize(self.potions.len(),None);
                self.potion_potencies[slot]=Some(self.initial_potency(id));
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }
    pub(crate) fn with_battle_context(mut self,battle:NativeBattleContext)->Self{self.battle=battle;self}
    /// Mark the owned inventory as belonging to the opt-in M15 Run profile.
    /// Task9's Run entry seam is the only production caller.
    pub(crate) fn with_m15_profile(mut self)->Self{self.battle.m15=true;self}
    pub(crate) fn m15_profile(&self)->bool{self.battle.m15}
    pub(crate) fn with_m20_profile(mut self)->Self{self.battle.m20=true;self}
    pub(crate) fn m20_profile(&self)->bool{self.battle.m20}
    pub(crate) fn shared_generated_profile(&self)->bool{self.battle.m15||self.battle.m20}
    #[cfg(test)]
    pub(crate) fn kite_triggered(&self)->bool{self.kite_triggered}
    pub(crate) fn wrist_blade_bonus(&self)->i32{self.wrist_blade_bonus}
    pub(crate) fn set_wrist_blade_bonus(&mut self,value:i32){self.wrist_blade_bonus=value;}
    pub(crate) fn damage_card(&self)->Option<dispatch::PhaseCard>{self.damage_card}
    pub(crate) fn set_damage_card_for_projection(&mut self,card:dispatch::PhaseCard){self.damage_card=Some(card);}
    pub(crate) fn relics(&self) -> &[RelicId] { &self.relics }
    pub(crate) fn potions(&self) -> &[Option<PotionId>] { &self.potions }
    pub(crate) fn counters(&self)->&std::collections::BTreeMap<RelicId,i32>{&self.counters}
    pub(crate) fn counter(&self,id:RelicId)->i32{self.counters.get(&id).copied().unwrap_or(0)}
    pub(crate) fn set_counter(&mut self,id:RelicId,value:i32){if value==0&&id!=RelicId::Inserter{self.counters.remove(&id);}else{self.counters.insert(id,value);}}
    pub(crate) fn with_counters(mut self,counters:std::collections::BTreeMap<RelicId,i32>)->Self{self.counters=counters;self}
    pub(crate) fn discard_potion(&mut self,slot:usize)->Result<(),RunError>{
        if !self.complete_items||self.potions.get(slot).is_none_or(Option::is_none){return Err(RunError::InvalidAction)}
        self.potions[slot]=None;
        if let Some(potency)=self.potion_potencies.get_mut(slot){*potency=None;}
        if self.last_potion_use.is_some_and(|(used_slot,_,_)|used_slot==slot){self.last_potion_use=None;}
        Ok(())
    }
    pub(crate) fn heal_room(&self) -> NativeHealRoom { self.heal_room }

    pub(crate) fn with_native_room(mut self, phase: PotionRoomPhase,
        map: Option<&crate::run::RunMap>) -> Self {
        self.heal_room = NativeHealRoom::from_map(phase, map);
        self
    }

    /// A typed independent combat room supplies the same healing entitlement as
    /// an entered Run combat node, without constructing a fake map.
    pub(crate) fn with_training_room(mut self)->Self{self.heal_room=NativeHealRoom{combat:true,current_map_node:true};self}
    /// M34s-a: validate owned resources against the potion base of `ascension` (M31 A11).
    pub(crate) fn with_ascension(mut self,ascension:u8)->Self{self.ascension=ascension;self}

    fn with_potions(&self, potions: Vec<Option<PotionId>>) -> Self {
        Self { potion_potencies:if self.potion_potency_enabled(){potions.iter().enumerate().map(|(i,id)|id.map(|id|if self.potions.get(i)==Some(&Some(id)){self.potion_potency(i).unwrap_or_else(||self.initial_potency(id))}else{self.initial_potency(id)})).collect()}else{vec![]},
            last_potion_use:if self.potion_potency_enabled(){self.potions.iter().zip(&potions).enumerate().find_map(|(slot,(old,new))|if new.is_none(){old.map(|id|(slot,id,self.potion_potency(slot).unwrap_or_else(||self.initial_potency(id))))}else{None}).or(self.last_potion_use)}else{None},
            startup_profile:self.startup_profile,watcher_progression:self.watcher_progression,watcher_act3:self.watcher_act3,watcher_heart:self.watcher_heart,complete_cards:self.complete_cards,master_order:self.master_order,complete_items:self.complete_items,local:self.local.clone(),relic_cards:self.relic_cards.clone(),runic_capacitor_armed:self.runic_capacitor_armed,battle:self.battle,collar_activated:self.collar_activated,emotion_chip_armed:self.emotion_chip_armed,counters:self.counters.clone(),relics: self.relics.clone(), potions,
            orichalcum_trigger:self.orichalcum_trigger,kite_triggered:self.kite_triggered,wrist_blade_bonus:self.wrist_blade_bonus,damage_card:None,lantern_armed: self.lantern_armed,
            heal_room: self.heal_room,ascension:self.ascension }
    }
}

/// The live potion count against the Run's base (M31 design §6.3): `base` (3, or 2 from A11), or
/// `base + 2` with Potion Belt. F6: under complete admission a lost Potion Belt keeps its two slots,
/// so `base + 2` stays valid without the belt.
fn valid_potion_capacity(relics: &[RelicId], potions: &[Option<PotionId>], base: usize, complete: bool) -> bool {
    if relics.contains(&RelicId::PotionBelt) { potions.len() == base + 2 } else { potions.len() == base || complete && potions.len() == base + 2 }
}

fn validate_potion_obtain_input(relics:&[RelicId],potions:&[Option<PotionId>],base:usize,complete:bool,id:PotionId)->Result<(),InputError>{
    if !valid_potion_capacity(relics,potions,base,complete)||potions.iter().flatten().any(|id|crate::content::Catalog::standard().potion(*id).is_none())
        ||crate::content::Catalog::standard().potion(id).is_none(){
        return Err(InputError::Invariant("generated potion obtain input invalid".into()))
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct NativeHealRoom { combat: bool, current_map_node: bool }

impl NativeHealRoom {
    fn from_map(phase: PotionRoomPhase, map: Option<&crate::run::RunMap>) -> Self {
        let current_map_node = map.is_some_and(|map| map.current
            .is_some_and(|current| map.nodes.iter().any(|node| node.id == current)));
        Self { combat: phase == PotionRoomPhase::Combat, current_map_node }
    }

    fn from_run(state: &RunState) -> Self {
        Self::from_map(if state.combat.is_some() { PotionRoomPhase::Combat }
            else { PotionRoomPhase::OutsideCombat }, state.map.as_ref())
    }

    pub(crate) fn is_live_combat_with_node(self) -> bool { self.combat && self.current_map_node }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PotionRoomPhase { Combat, OutsideCombat }

/// Borrowed native facts for a single potion-use attempt. This is an ephemeral
/// projection supplied by the current owner, never another inventory/state.
#[derive(Clone, Copy, Debug)]
pub(crate) struct NativePotionUseContext<'a> {
    pub phase: PotionRoomPhase,
    pub turn_ended: bool,
    pub we_meet_again: bool,
    pub hp: u32,
    pub max_hp: u32,
    pub live_targets: &'a [EnemyHandle],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PotionUseEffects {
    pub effects: Vec<Effect>,
    pub potions: Vec<Option<PotionId>>,
}

/// Dispatch run-level item checkpoints that mutate persistent run resources.
/// Active-combat item effects use the same handlers with the combat-owned item
/// context and are committed through Combat's ordinary resolver.
pub fn on_hook(state: &mut RunState, hook: Hook) -> Result<(), RunError> {
    if hook==Hook::CombatSettled{relics::settlement_heal_bridge(state);}
    Ok(())
}

/// Native relic acquisition while RunState owns the permanent inventory.
pub(crate) fn acquire_relic(state: &mut RunState, id: RelicId) -> Result<(), RunError> {
    relics::acquire(state, id)
}

/// Apply the finite item-owned ordinary-heal callback to an already computed
/// requested amount. Other healing modifiers remain explicitly unsupported.
pub(crate) fn ordinary_heal_amount(relics: &[RelicId], amount: i32) -> i32 {
    ordinary_heal_amount_in_room(relics, amount, NativeHealRoom::default())
}

/// The one heal law (design §9.1) for an onVictory heal committed at combat settlement (Meat on
/// the Bone, Face of Cleric): the room is still COMBAT, so Magic Flower applies.
pub(crate) fn victory_heal_amount(state: &RunState, amount: i32) -> i32 {
    ordinary_heal_amount_in_room(&state.relics, amount,
        NativeHealRoom::from_map(PotionRoomPhase::Combat, state.map.as_ref()))
}

pub(crate) fn ordinary_heal_amount_in_room(relics: &[RelicId], amount: i32,
    room: NativeHealRoom) -> i32 {
    if relics.contains(&RelicId::MarkOfBloom) { return 0; }
    if relics.contains(&RelicId::MagicFlower) && room.is_live_combat_with_node() {
        let scaled = (amount as f32) * 1.5_f32;
        return (scaled + 0.5_f32).floor() as i32;
    }
    amount
}

/// Produce ordered native effects for a combat lifecycle checkpoint: the hook's phase
/// through the one relic dispatcher (`dispatch::combat_phase`).
pub(crate) fn relic_phase_effects(context: &mut CombatItemContext, facts: &dispatch::PhaseFacts<'_>) -> Result<RelicHookWork, InputError> {
    let Some(hook)=facts.hook else { return Ok(RelicHookWork::default()) };
    if hook==Hook::CombatSettled {context.emotion_chip_armed=false;}
    if context.m20_profile()&&hook==Hook::TurnStartedPreDraw&&facts.target==Some(&crate::combat::Actor::Player){validate_defect_relic_counters(context)?;}
    let Some(phase)=dispatch::hook_phase(hook) else { return Ok(RelicHookWork::default()) };
    let mut work=dispatch::combat_phase(context,phase,facts)?;
    if phase==dispatch::ItemPhase::CardUsed {
        let after=dispatch::combat_phase(context,dispatch::ItemPhase::AfterUse,facts)?;
        work.top.extend(after.top);work.bottom.extend(after.bottom);
    }
    Ok(work)
}

/// One relic slot of a player turn-start checkpoint (`dispatch::turn_start_phases`).
pub(crate) fn turn_start_relic_work(context: &mut CombatItemContext, phase: dispatch::ItemPhase, facts: &dispatch::PhaseFacts<'_>) -> Result<RelicHookWork, InputError> {
    if context.m20_profile()&&phase==dispatch::ItemPhase::TurnStartPreDraw{validate_defect_relic_counters(context)?;}
    dispatch::combat_phase(context,phase,facts)
}

/// The accepted tests' bridge entry points, kept as thin calls into the one dispatcher.
#[cfg(test)]
pub(crate) fn relic_hook_effects(context: &mut CombatItemContext, hook: Hook, hp: i32,max_hp:i32,block:i32) -> RelicHookWork {
    relic_phase_effects(context,&dispatch::PhaseFacts::new(Some(hook),hp,max_hp,block)).expect("accepted bridge hook")
}
#[cfg(test)]
pub(crate) fn silent_death_work(context:&CombatItemContext,dead_owner:EnemyHandle,callback_poison:Option<i32>,live_eligible_count:usize)->Vec<Effect>{
    let mut facts=dispatch::PhaseFacts::new(Some(Hook::Death),0,0,0);facts.dead_enemy=Some((&dead_owner,callback_poison,live_eligible_count));
    let work=dispatch::combat_phase(&mut context.clone(),dispatch::ItemPhase::EnemyDied,&facts).expect("accepted death bridge");
    work.top.into_iter().chain(work.bottom).collect()
}
#[cfg(test)]
pub(crate) fn defect_pre_battle_effects(context:&mut CombatItemContext)->Result<Vec<Effect>,InputError>{startup_work(context)}
#[cfg(test)]
pub(crate) fn defect_turn_start_effects(context:&mut CombatItemContext)->Result<Vec<Effect>,InputError>{
    let mut facts=dispatch::PhaseFacts::new(Some(Hook::TurnStartedPreDraw),0,0,0);facts.target=Some(&crate::combat::Actor::Player);
    let work=relic_phase_effects(context,&facts)?;
    Ok(work.top.into_iter().chain(work.bottom).collect())
}
#[cfg(test)]
pub(crate) fn defect_end_turn_work(context:&CombatItemContext,has_empty_orb:bool)->Vec<Effect>{
    let mut facts=dispatch::PhaseFacts::new(Some(Hook::PlayerPreEndTurnCards),0,0,0);facts.target=Some(&crate::combat::Actor::Player);facts.has_empty_orb=has_empty_orb;
    relic_phase_effects(&mut context.clone(),&facts).expect("accepted end-turn bridge").top
}

/// The combat-entry startup dispatch (before the opening hand): Defect atPreBattle work and
/// Ninja Scroll's generated Shivs, in acquisition order, as ordinary bottom work.
pub(crate) fn startup_work(context:&mut CombatItemContext)->Result<Vec<Effect>,InputError>{
    let mut facts=dispatch::PhaseFacts::new(None,0,0,0);
    facts.elite=context.battle.elite_trigger;facts.boss=context.battle.boss;
    startup_work_with_facts(context,&facts)
}

pub(crate) fn startup_work_with_facts(context:&mut CombatItemContext,facts:&dispatch::PhaseFacts<'_>)->Result<Vec<Effect>,InputError>{
    if context.m20_profile(){validate_defect_relic_counters(context)?;}
    let work=dispatch::combat_phase(context,dispatch::ItemPhase::CombatStartPreDraw,facts)?;
    Ok(work.top.into_iter().chain(work.bottom).collect())
}

/// Validate and prepare a potion use without mutating the caller's item owner.
/// The returned post-use slots are committed only alongside validated effects.
pub(crate) fn potion_use_effects(context: &CombatItemContext, slot: usize,
    target: Option<EnemyHandle>, facts: &NativePotionUseContext<'_>) -> Result<PotionUseEffects, RunError> {
    potions::use_potion(context, slot, target, facts)
}
pub(crate) fn potion_use_effects_for_character(context: &CombatItemContext, slot: usize,
    target: Option<EnemyHandle>, facts: &NativePotionUseContext<'_>, character: Character) -> Result<PotionUseEffects, RunError> {
    potions::use_potion_for_character(context, slot, target, facts, character)
}

/// Finite supported startup/use IDs, independent of registration metadata.
pub(crate) fn validate_combat_inventory(context: &CombatItemContext,character:crate::content::Character) -> Result<(), RunError> {
    if context.complete_items(){return validate_complete_inventory(context,character)}
    // M26 overlay potion-belt admission: under the complete capability a held potion is admitted
    // when it is in the player's class pool (Alchemize gains settle there); relics, counters and
    // belt shape stay the base context's rules, checked with those potions set aside.
    if context.complete_cards(){
        let pool=potion_generation::candidates(character,false).map_err(|_|RunError::InvalidAction)?;
        if context.potions.iter().flatten().any(|id|!pool.contains(id)){return Err(RunError::InvalidAction)}
        let mut base=context.clone();
        base.complete_cards=false;
        base.potions=base.potions.iter().map(|_|None).collect();
        return validate_combat_inventory(&base,character)
    }
    if context.m22_profile(){
        let mut seen=std::collections::BTreeSet::new();
        if !context.m21_profile()||character!=Character::Watcher||context.battle.m15||context.battle.m20||context.potions.len()!=3
            ||context.relics.iter().any(|id|!(if context.watcher_heart{crate::run::M25_WATCHER_HEART_RELICS}else if context.watcher_act3{crate::run::M24_WATCHER_ACT3_RELICS}else{crate::run::M22_WATCHER_ACT1_RELICS}).contains(id)||!seen.insert(*id))
            ||context.potions.iter().flatten().any(|id|!crate::run::M22_WATCHER_ACT1_POTIONS.contains(id))
            ||context.owns_relic(RelicId::PureWater)&&context.owns_relic(RelicId::HolyWater)
            ||context.counters.iter().any(|(id,n)|*id!=RelicId::NeowsLament||!context.owns_relic(*id)||!matches!(*n,1..=3|-2))
            ||context.owns_relic(RelicId::NeowsLament)&&!matches!(context.counters.get(&RelicId::NeowsLament),Some(1..=3)|Some(-2)){
            return Err(RunError::InvalidAction)
        }
        return Ok(())
    }
    if context.m21_profile(){
        let mut seen=std::collections::BTreeSet::new();
        if character!=Character::Watcher||context.battle.m15||context.battle.m20||context.potions.len()!=3||!context.counters.is_empty()
            ||context.relics.iter().any(|id|!crate::run::M21_WATCHER_RELICS.contains(id)||!seen.insert(*id))
            ||context.potions.iter().flatten().any(|id|!crate::run::M21_WATCHER_POTIONS.contains(id)) {return Err(RunError::InvalidAction)}
        return Ok(())
    }
    if context.m20_profile()&&character!=Character::Defect{return Err(RunError::InvalidAction)}
    let new_blue_relics=[RelicId::SymbioticVirus,RelicId::FrozenCore,RelicId::Inserter,RelicId::NuclearBattery,RelicId::RunicCapacitor];
    if (!context.m20_profile()&&context.relics.iter().any(|id|new_blue_relics.contains(id)))
        || (!context.m20_profile()&&context.potions.iter().flatten().any(|id|*id==PotionId::EssenceOfDarkness)){
        return Err(RunError::Unsupported("M20 exclusive item requires M20 profile"))
    }
    if new_blue_relics.iter().any(|id|context.relics.iter().filter(|owned|*owned==id).count()>1)
        ||context.owns_relic(RelicId::FrozenCore)&&context.owns_relic(RelicId::CrackedCore)
        ||context.counters.get(&RelicId::Inserter).is_some_and(|value|!context.owns_relic(RelicId::Inserter)||!(-1..=1).contains(value)){
        return Err(RunError::InvalidAction)
    }
    let defect_items=context.relics.iter().any(|id|matches!(id,RelicId::DataDisk|RelicId::GoldPlatedCables|RelicId::EmotionChip|RelicId::SymbioticVirus|RelicId::FrozenCore|RelicId::Inserter|RelicId::NuclearBattery|RelicId::RunicCapacitor))
        ||context.potions.iter().flatten().any(|id|matches!(id,PotionId::FocusPotion|PotionId::PotionOfCapacity|PotionId::EssenceOfDarkness));
    if defect_items&&character!=crate::content::Character::Defect{return Err(RunError::InvalidAction)}
    if !context.shared_generated_profile()&&context.relics.iter().any(|id|matches!(id,RelicId::Sozu|RelicId::PotionBelt)){
        return Err(RunError::Unsupported("generated-potion relic requires M15 profile"))
    }
    let has_m15_relic=context.relics.iter().any(|id|matches!(id,RelicId::NinjaScroll|RelicId::PaperKrane|RelicId::TheSpecimen
        |RelicId::Tingsha|RelicId::ToughBandages|RelicId::TwistedFunnel|RelicId::HoveringKite
        |RelicId::RingOfTheSerpent|RelicId::WristBlade));
    let has_new_shared_potion=context.potions.iter().flatten().any(|id|matches!(id,
        PotionId::FlexPotion|PotionId::SpeedPotion|PotionId::RegenPotion|PotionId::LiquidBronze
        |PotionId::EssenceOfSteel|PotionId::CultistPotion|PotionId::BlessingOfTheForge|PotionId::SneckoOil
        |PotionId::FairyPotion|PotionId::AttackPotion|PotionId::SkillPotion|PotionId::PowerPotion
        |PotionId::ColorlessPotion|PotionId::ExplosivePotion|PotionId::GamblersBrew|PotionId::LiquidMemories
        |PotionId::DuplicationPotion|PotionId::DistilledChaos|PotionId::SmokeBomb|PotionId::EntropicBrew));
    if has_new_shared_potion&&!context.shared_generated_profile(){return Err(RunError::Unsupported("M15 shared potion requires M15 profile"))}
    let has_m15_potion=context.potions.iter().flatten().any(|id|matches!(id,PotionId::PoisonPotion|PotionId::CunningPotion|PotionId::GhostInAJar));
    if (has_m15_relic||has_m15_potion)&&!context.m15_profile(){return Err(RunError::Unsupported("M15 exclusive item requires M15 profile"));}
    if (has_m15_relic||has_m15_potion)&&character!=crate::content::Character::Silent{return Err(RunError::InvalidAction);}
    if context.owns_relic(RelicId::BurningBlood)&&context.owns_relic(RelicId::BlackBlood){return Err(RunError::InvalidAction)}
    if context.owns_relic(RelicId::NeowsLament)&&!matches!(context.counters.get(&RelicId::NeowsLament),Some(1..=3)|Some(-2)){return Err(RunError::InvalidAction)}
    for id in &context.relics {
        // Approved blue-X modifier is diagnostic-only: ordinary M20 producer
        // and encoder vocabularies exclude it; old startup admission stays shut.
        if *id==RelicId::ChemicalX&&context.m20_profile(){continue}
        if !matches!(id, RelicId::BurningBlood | RelicId::RingOfTheSnake | RelicId::CrackedCore
            | RelicId::PureWater | RelicId::BloodVial | RelicId::Anchor | RelicId::Lantern
            | RelicId::Strawberry | RelicId::Pear | RelicId::Mango | RelicId::SacredBark
            | RelicId::MarkOfBloom | RelicId::MagicFlower | RelicId::ToyOrnithopter | RelicId::NeowsLament
            | RelicId::BlackBlood | RelicId::EmptyCage | RelicId::SlaversCollar | RelicId::RedMask
            | RelicId::Vajra | RelicId::OddlySmoothStone | RelicId::BagOfPreparation | RelicId::BagOfMarbles | RelicId::BronzeScales
            | RelicId::RedSkull | RelicId::PaperPhrog | RelicId::ChampionsBelt | RelicId::CharonsAshes | RelicId::SelfFormingClay
            | RelicId::Brimstone | RelicId::MarkOfPain | RelicId::RunicCube | RelicId::Orichalcum
            | RelicId::SneckoSkull | RelicId::NinjaScroll | RelicId::PaperKrane | RelicId::TheSpecimen
            | RelicId::Tingsha | RelicId::ToughBandages | RelicId::TwistedFunnel | RelicId::HoveringKite
            | RelicId::RingOfTheSerpent | RelicId::WristBlade | RelicId::Sozu | RelicId::PotionBelt
            | RelicId::DataDisk | RelicId::GoldPlatedCables | RelicId::EmotionChip
            | RelicId::SymbioticVirus | RelicId::FrozenCore | RelicId::Inserter | RelicId::NuclearBattery | RelicId::RunicCapacitor) {
            return Err(RunError::Unsupported("unimplemented combat relic"));
        }
    }
    for id in context.potions.iter().flatten() {
        if !matches!(id, PotionId::FirePotion | PotionId::BloodPotion | PotionId::FruitJuice
            | PotionId::BlockPotion | PotionId::EnergyPotion | PotionId::StrengthPotion
            | PotionId::SwiftPotion | PotionId::WeakPotion | PotionId::FearPotion
            | PotionId::DexterityPotion | PotionId::AncientPotion | PotionId::Elixir | PotionId::HeartOfIron
            | PotionId::PoisonPotion | PotionId::CunningPotion | PotionId::GhostInAJar
            | PotionId::FlexPotion | PotionId::SpeedPotion | PotionId::RegenPotion
            | PotionId::LiquidBronze | PotionId::EssenceOfSteel | PotionId::CultistPotion
            | PotionId::BlessingOfTheForge | PotionId::SneckoOil | PotionId::FairyPotion
            | PotionId::ExplosivePotion | PotionId::AttackPotion | PotionId::SkillPotion | PotionId::PowerPotion
            | PotionId::ColorlessPotion | PotionId::GamblersBrew
            | PotionId::DuplicationPotion | PotionId::DistilledChaos | PotionId::LiquidMemories
            | PotionId::SmokeBomb | PotionId::EntropicBrew | PotionId::FocusPotion | PotionId::PotionOfCapacity | PotionId::EssenceOfDarkness) {
            return Err(RunError::Unsupported("unimplemented combat potion"));
        }
    }
    Ok(())
}

/// The domain of each persistent relic counter (design §7.5). `None`: the relic keeps no
/// persistent counter, so a counter key for it is invalid.
pub(crate) fn counter_domain(id:RelicId)->Option<std::ops::RangeInclusive<i32>>{
    Some(match id{
        RelicId::PenNib|RelicId::Nunchaku|RelicId::InkBottle=>0..=9,
        RelicId::IncenseBurner=>0..=5,
        RelicId::HappyFlower|RelicId::Sundial|RelicId::Omamori|RelicId::Matryoshka=>0..=2,
        RelicId::WingBoots|RelicId::Girya|RelicId::TinyChest=>0..=3,
        RelicId::Inserter=>-1..=1,
        RelicId::NeowsLament=>-2..=3,
        RelicId::Circlet=>1..=i32::MAX,
        // Used / disabled / armed flags.
        RelicId::LizardTail|RelicId::MawBank|RelicId::NlothsHungryFace|RelicId::AncientTeaSet=>1..=1,
        _=>return None,
    })
}

/// The one complete-inventory validator (design §5.2), reached first from
/// `validate_combat_inventory` under `complete_items()`.
pub(crate) fn validate_complete_inventory(context:&CombatItemContext,character:Character)->Result<(),RunError>{
    if !context.complete_cards(){return Err(RunError::Unsupported("complete items require the complete card capability"))}
    let union=crate::content::complete::relic_union(character);
    let mut seen=std::collections::BTreeSet::new();
    // F7 makes Red Circlet the exhausted Boss fallback, so owned inventory admits it outside ordinary relic unions.
    if context.relics.iter().any(|id|!(union.contains(id)||*id==RelicId::RedCirclet)||!seen.insert(*id)){return Err(RunError::InvalidAction)}
    const REPLACEMENTS:[(RelicId,RelicId);4]=[(RelicId::BurningBlood,RelicId::BlackBlood),(RelicId::RingOfTheSnake,RelicId::RingOfTheSerpent),
        (RelicId::CrackedCore,RelicId::FrozenCore),(RelicId::PureWater,RelicId::HolyWater)];
    if REPLACEMENTS.iter().any(|(starter,boss)|context.owns_relic(*starter)&&context.owns_relic(*boss)){return Err(RunError::InvalidAction)}
    let potions=crate::content::complete::potion_union(character);
    if context.potions.iter().flatten().any(|id|!potions.contains(id))||!context.valid_potion_capacity(){return Err(RunError::InvalidAction)}
    if context.counters.iter().any(|(id,value)|!context.owns_relic(*id)||!counter_domain(*id).is_some_and(|domain|domain.contains(value))){return Err(RunError::InvalidAction)}
    if context.owns_relic(RelicId::NeowsLament)&&!matches!(context.counters.get(&RelicId::NeowsLament),Some(1..=3)|Some(-2)){return Err(RunError::InvalidAction)}
    Ok(())
}

/// Pure full-combat preparation used by both legal menus and accepted use.
pub(crate) fn prepare_combat_potion(combat: &Combat, slot: usize,
    target: Option<EnemyHandle>, we_meet_again: bool) -> Result<PotionUseEffects, RunError> {
    prepare_combat_potion_inner(combat, slot, target, we_meet_again, combat.native_generation_character())
}

fn prepare_combat_potion_inner(combat: &Combat, slot: usize,
    target: Option<EnemyHandle>, we_meet_again: bool, character: Option<Character>) -> Result<PotionUseEffects, RunError> {
    let context = combat.item_context().ok_or(RunError::InvalidAction)?;
    if context.potions().get(slot)==Some(&Some(PotionId::SmokeBomb))&&!combat.native_smoke_allowed(){return Err(RunError::InvalidAction)}
    let (phase, hp, max_hp, live_targets) = combat.native_potion_use_state()?;
    let facts = NativePotionUseContext { phase: PotionRoomPhase::Combat,
        turn_ended: phase == crate::combat::Phase::Terminal, we_meet_again,
        hp: hp as u32, max_hp: max_hp as u32, live_targets: &live_targets };
    let potion=context.potions().get(slot).copied().flatten();
    let mut prepared = match character {
        Some(character) => potion_use_effects_for_character(context, slot, target, &facts, character)?,
        None => potion_use_effects(context, slot, target, &facts)?,
    };
    // Frozen item profiles keep the accepted Toy Ornithopter effect in the
    // prepared potion prefix. Complete items route it through PotionUsed below.
    if context.owns_relic(RelicId::ToyOrnithopter)&&!context.complete_items(){
        prepared.effects.push(Effect { source: crate::combat::Source::None,
            target: Some(crate::combat::Actor::Player), kind: crate::combat::EffectKind::Heal(5) });
    }
    if potion==Some(PotionId::ExplosivePotion){
        for effect in &mut prepared.effects {
            if let (Some(crate::combat::Actor::Enemy(owner)),crate::combat::EffectKind::Damage(
                crate::combat::DamagePayload::Fixed(amount)))=(&effect.target,&mut effect.kind){
                *amount=combat.normal_potion_damage_to(owner,*amount)?;
            }
        }
    }
    combat.preflight_native_effects(&prepared.effects)?;
    Ok(prepared)
}

/// Resolve a combat-owned potion through the shared queue, committing its slot
/// only with the validated native effects.
pub(crate) fn use_combat_potion(combat: &mut Combat, slot: usize,
    target: Option<EnemyHandle>, we_meet_again: bool) -> Result<(), RunError> {
    let context = combat.item_context().cloned().ok_or(RunError::InvalidAction)?;
    let prepared = prepare_combat_potion(combat, slot, target, we_meet_again)?;
    let mut prepared=prepared;
    combat.capture_distilled_chaos_targets(&mut prepared.effects)?;
    let (_,hp,max_hp,_)=combat.native_potion_use_state()?;
    let mut next_items=context.with_potions(prepared.potions);
    let mut facts=dispatch::PhaseFacts::new(None,hp,max_hp,0);
    let potion_source=crate::combat::Source::Potion{slot,id:context.potions().get(slot).copied().flatten().ok_or(RunError::InvalidAction)?};
    facts.source=Some(&potion_source);
    // M27 seam: potion-used; the returned bottom work follows all potion effects.
    let work=dispatch::combat_phase(&mut next_items,dispatch::ItemPhase::PotionUsed,&facts)
        .map_err(|_|RunError::InvalidAction)?;
    combat.enqueue_native_effects_with_relic_work(prepared.effects,Some(next_items),work)
}

/// Native noncombat use while RunState is the sole owner of HP and potion slots.
/// This deliberately does not create a public action or advance room progression.
pub(crate) fn use_run_potion(state: &mut RunState, slot: usize,
    turn_ended: bool, we_meet_again: bool) -> Result<(), RunError> {
    if (crate::run::rooms::is_m15(state)||crate::run::rooms::is_m20(state)||state.item_admission==crate::run::ItemAdmission::Complete)
        &&state.potions.get(slot).copied().flatten()==Some(PotionId::EntropicBrew){
        if turn_ended{return Err(RunError::InvalidAction)}
        return use_m15_entropic_brew_outside(state,slot,we_meet_again)
    }
    let mut observed=crate::combat::observe::run_enabled(state).then(Vec::new);
    let (hp,max_hp,potions)=prepare_run_potion_inner(state,slot,turn_ended,we_meet_again,|before,after,source| {
        if let Some(values)=observed.as_mut(){values.push((before,after,source.clone()));}
    })?;
    state.hp=hp;state.max_hp=max_hp;state.potions=potions;
    if state.item_admission==crate::run::ItemAdmission::Complete {
        state.potion_potencies=main_relics::reconcile_potion_potencies(&state.relics,&state.potions,&state.potion_potencies);
    }
    // Preparation is also used by legal menus. Publish its detached primitive
    // checkpoints only after this real use commits, never during menu queries.
    if let Some(values)=observed {for (before,after,source) in values {crate::combat::observe::run_heal_values(state,before,after,&source);}}
    Ok(())
}

/// Outside-combat first-empty-slot obtain. This inventory helper deliberately
/// has no Sozu branch; accepted generated obtains check Sozu at execution.
pub(crate) fn obtain_run_potion(state: &mut RunState, id: PotionId) -> Result<bool, RunError> {
    validate_potion_obtain_input(&state.relics,&state.potions,crate::run::potion_base_slots(state.ascension),state.item_admission==crate::run::ItemAdmission::Complete,id)
        .map_err(|_|RunError::Unsupported("generated potion obtain input invalid"))?;
    if let Some(slot) = state.potions.iter().position(Option::is_none) {
        state.potions[slot] = Some(id);
        if state.item_admission==crate::run::ItemAdmission::Complete {
            state.potion_potencies=main_relics::reconcile_potion_potencies(&state.relics,&state.potions,&state.potion_potencies);
        }
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Apply a proposal after an outside continuation has already been accepted.
/// Any invalid proposal retains the first Run fault and the successful prefix.
pub(crate) fn apply_accepted_run_potion_obtain(state: &mut RunState, id: PotionId) -> Result<bool, RunError> {
    if let Some(fault) = &state.fault { return Err(RunError::Faulted(fault.clone())); }
    let prepared=validate_potion_obtain_input(&state.relics,&state.potions,crate::run::potion_base_slots(state.ascension),state.item_admission==crate::run::ItemAdmission::Complete,id)
        .map_err(|_|RunError::Unsupported("generated potion obtain input invalid"))
        .and_then(|_|if state.relics.contains(&RelicId::Sozu){Ok(false)}else{obtain_run_potion(state,id)});
    match prepared {
        Ok(obtained) => Ok(obtained),
        Err(_) => {
            if state.fault.is_none() {
                state.fault = Some(crate::run::RunFault {
                    kind: crate::run::RunFaultKind::Invariant("accepted potion obtain failed"),
                    checkpoint: "run.m15.potion.obtain",
                    committed_events: state.history.events.clone(),
                });
            }
            Err(RunError::Faulted(state.fault.clone().unwrap()))
        }
    }
}

/// Shared accepted outside-combat Entropic Brew. Sozu consumes Brew before any
/// proposal draw; otherwise all unrestricted proposals are captured before the
/// slot is vacated and first-empty-slot obtains begin.
pub(crate) fn use_m15_entropic_brew_outside(
    state: &mut RunState, slot: usize, we_meet_again: bool,
) -> Result<(), RunError> {
    validate_m15_entropic_brew_outside(state,slot,we_meet_again)?;

    if state.relics.contains(&RelicId::Sozu) {
        state.potions[slot] = None;
        if state.item_admission==crate::run::ItemAdmission::Complete {
            state.potion_potencies=main_relics::reconcile_potion_potencies(&state.relics,&state.potions,&state.potion_potencies);
        }
        apply_run_potion_use_hook(state);
        return Ok(());
    }

    // One proposal per live slot under every admission (research table: full/unused proposals
    // are not evidence of fewer source draws).
    let count = state.potions.len();
    let semantic = [state.floor as u64, slot as u64];
    let proposals = match potion_generation::sample_many_for_run(
        state, state.character, false, count, &semantic,
    ) {
        Ok(proposals) => proposals,
        Err(_) => {
            if state.fault.is_none() {
                state.fault = Some(crate::run::RunFault {
                    kind: crate::run::RunFaultKind::Invariant("accepted potion generation failed"),
                    checkpoint: "run.m15.potion.generate",
                    committed_events: state.history.events.clone(),
                });
            }
            return Err(RunError::Faulted(state.fault.clone().unwrap()));
        }
    };
    state.potions[slot] = None;
    if state.item_admission==crate::run::ItemAdmission::Complete {
        state.potion_potencies=main_relics::reconcile_potion_potencies(&state.relics,&state.potions,&state.potion_potencies);
    }
    apply_run_potion_use_hook(state);
    for id in proposals {
        apply_accepted_run_potion_obtain(state, id)?;
    }
    Ok(())
}

fn validate_m15_entropic_brew_outside(state:&RunState,slot:usize,we_meet_again:bool)->Result<(),RunError>{
    if let Some(fault) = &state.fault { return Err(RunError::Faulted(fault.clone())); }
    if state.terminal.is_some() { return Err(RunError::Terminal); }
    let complete=state.item_admission==crate::run::ItemAdmission::Complete;
    if state.combat.is_some() || !(complete || state.character == Character::Silent
        || state.character == Character::Defect && crate::run::rooms::is_m20(state)) || we_meet_again
        || state.max_hp == 0 || state.max_hp > 999 || state.hp > state.max_hp
        || state.potions.get(slot).copied().flatten() != Some(PotionId::EntropicBrew)
        || !valid_potion_capacity(&state.relics, &state.potions, crate::run::potion_base_slots(state.ascension), complete)
        || !state.potions.iter().flatten().all(|owned| crate::content::Catalog::standard().potion(*owned).is_some()) {
        return Err(RunError::InvalidAction);
    }
    if complete {
        let pool=potion_generation::candidates(state.character,false).map_err(|_|RunError::InvalidAction)?;
        if state.potions.iter().flatten().any(|id|!pool.contains(id)){return Err(RunError::InvalidAction)}
    }
    Ok(())
}
pub(crate) fn can_use_run_potion(state:&RunState,slot:usize,turn_ended:bool,we_meet_again:bool)->bool{
    prepare_run_potion(state,slot,turn_ended,we_meet_again).is_ok()
        ||(crate::run::rooms::is_m15(state)||crate::run::rooms::is_m20(state)||state.item_admission==crate::run::ItemAdmission::Complete)
            &&!turn_ended&&validate_m15_entropic_brew_outside(state,slot,we_meet_again).is_ok()
}
pub(crate) fn prepare_run_potion(state:&RunState,slot:usize,turn_ended:bool,we_meet_again:bool)->Result<(u32,u32,Vec<Option<PotionId>>),RunError>{
    prepare_run_potion_inner(state,slot,turn_ended,we_meet_again,|_,_,_|{})
}
fn prepare_run_potion_inner(state:&RunState,slot:usize,turn_ended:bool,we_meet_again:bool,
    mut observe_heal:impl FnMut(u32,u32,&crate::combat::Source))->Result<(u32,u32,Vec<Option<PotionId>>),RunError>{
    if let Some(fault) = &state.fault { return Err(RunError::Faulted(fault.clone())); }
    if state.terminal.is_some() { return Err(RunError::Terminal); }
    if state.combat.is_some() { return Err(RunError::InvalidAction); }
    if state.max_hp == 0 || state.max_hp > 999 || state.hp > state.max_hp {
        return Err(RunError::InvalidAction);
    }
    let facts = NativePotionUseContext { phase: PotionRoomPhase::OutsideCombat,
        turn_ended, we_meet_again, hp: state.hp, max_hp: state.max_hp,
        live_targets: &[] };
    let owned = run_potion_context(state);
    let prepared = potion_use_effects(&owned, slot, None, &facts)?;
    let mut hp = state.hp;
    let mut max_hp = state.max_hp;
    for effect in &prepared.effects {
        match &effect.kind {
            crate::combat::EffectKind::Heal(amount) => {
                let before=hp;
                let amount = ordinary_heal_amount(owned.relics(), *amount).max(0) as u32;
                if hp > 0 { hp = hp.saturating_add(amount).min(max_hp); }
                observe_heal(before,hp,&effect.source);
            }
            crate::combat::EffectKind::IncreaseMaxHpThenHeal(amount) => {
                let before=hp;
                max_hp = max_hp.checked_add(*amount as u32)
                    .filter(|value| *value <= 999)
                    .ok_or(RunError::Unsupported("maximum HP overflow"))?;
                let heal = ordinary_heal_amount(owned.relics(), *amount).max(0) as u32;
                if hp > 0 { hp = hp.saturating_add(heal).min(max_hp); }
                observe_heal(before,hp,&effect.source);
            }
            _ => return Err(RunError::InvalidAction),
        }
    }
    if let Some((before, after)) = run_potion_use_heal(owned.relics(), hp, max_hp) {
        hp = after;
        observe_heal(before, hp, &crate::combat::Source::None);
    }
    Ok((hp,max_hp,prepared.potions))
}

fn run_potion_context(state:&RunState)->CombatItemContext{
    let mut owned=CombatItemContext::new(state.relics.clone(),state.potions.clone());
    owned.ascension=state.ascension;
    if state.item_admission==crate::run::ItemAdmission::Complete {
        let potencies=main_relics::reconcile_potion_potencies(&state.relics,&state.potions,&state.potion_potencies);
        owned.potion_potencies=if potencies.is_empty(){vec![None;state.potions.len()]}
            else{potencies.iter().map(|slot|slot.map(|(_,potency)|potency)).collect()};
        owned=owned.with_complete_items();
    }
    owned
}

fn run_potion_use_heal(relics: &[RelicId], hp: u32, max_hp: u32) -> Option<(u32, u32)> {
    if !relics.contains(&RelicId::ToyOrnithopter) { return None; }
    let heal = ordinary_heal_amount(relics, 5) as u32;
    Some((hp, if hp > 0 { hp.saturating_add(heal).min(max_hp) } else { hp }))
}

fn apply_run_potion_use_hook(state: &mut RunState) {
    let Some((before, after)) = run_potion_use_heal(&state.relics, state.hp, state.max_hp) else { return; };
    state.hp = after;
    if crate::combat::observe::run_enabled(state) {
        crate::combat::observe::run_heal_values(state, before, after, &crate::combat::Source::None);
    }
}

pub(crate) mod potion_generation;
pub(crate) mod potion_offers;
pub(crate) mod main_relics;
#[cfg(test)]
mod m15_potion_generation_tests;

/// Combat phase work of the M20 Defect relics (bodies unchanged): atPreBattle at the startup
/// dispatch, the one atTurnStart pass before player powers and orbs, and Frozen Core's
/// end-of-turn Frost. Counter commit occurs even when the later Slots request caps at 10.
pub(crate) fn defect_phase(id:RelicId,context:&mut CombatItemContext,phase:dispatch::ItemPhase,facts:&dispatch::PhaseFacts<'_>,work:&mut RelicHookWork)->Result<bool,InputError>{
    if !matches!(id,RelicId::SymbioticVirus|RelicId::NuclearBattery|RelicId::RunicCapacitor|RelicId::Inserter|RelicId::EmotionChip|RelicId::FrozenCore){return Ok(false)}
    let player=facts.target==Some(&crate::combat::Actor::Player);
    // Emotion Chip (atTurnStart, bottom) is armed by HP loss in every profile that owns it.
    if id==RelicId::EmotionChip{
        if facts.hook==Some(Hook::TurnStartedPreDraw)&&player&&context.emotion_chip_armed{
            context.emotion_chip_armed=false;
            work.bottom.push(crate::combat::Effect{source:crate::combat::Source::Actor(crate::combat::Actor::Player),target:Some(crate::combat::Actor::Player),
                kind:crate::combat::EffectKind::Character(crate::combat::CharacterEffect::PassiveGroup(crate::combat::OrbPassiveGroup::All))});
        }
        return Ok(true)
    }
    if !(context.m20_profile()||context.complete_items()){return Ok(true)}
    match (id,phase,facts.hook) {
        (RelicId::RunicCapacitor,dispatch::ItemPhase::CombatStartPreDraw,None)=>context.runic_capacitor_armed=true,
        (RelicId::SymbioticVirus,dispatch::ItemPhase::CombatStartPreDraw,None)=>work.bottom.push(defect_relic_effect(crate::combat::CharacterEffect::Channel(crate::combat::OrbKind::Dark))),
        (RelicId::NuclearBattery,dispatch::ItemPhase::CombatStartPreDraw,None)=>work.bottom.push(defect_relic_effect(crate::combat::CharacterEffect::Channel(crate::combat::OrbKind::Plasma))),
        (RelicId::RunicCapacitor,_,Some(Hook::TurnStartedPreDraw)) if player&&context.runic_capacitor_armed=>{
            context.runic_capacitor_armed=false;work.bottom.push(defect_relic_effect(crate::combat::CharacterEffect::Slots(3)));
        }
        (RelicId::Inserter,_,Some(Hook::TurnStartedPreDraw)) if player=>{
            let old=context.counter(RelicId::Inserter);let next=if old<0{1}else{old+1};
            context.set_counter(RelicId::Inserter,if next==2{0}else{next});
            if next==2{work.bottom.push(defect_relic_effect(crate::combat::CharacterEffect::Slots(1)));}
        }
        (RelicId::FrozenCore,_,Some(Hook::PlayerPreEndTurnCards)) if player&&facts.has_empty_orb=>
            work.top.push(defect_relic_effect(crate::combat::CharacterEffect::Channel(crate::combat::OrbKind::Frost))),
        _=>{}
    }
    Ok(true)
}
fn defect_relic_effect(effect:crate::combat::CharacterEffect)->Effect{
    Effect{source:crate::combat::Source::None,target:Some(crate::combat::Actor::Player),kind:crate::combat::EffectKind::Character(effect)}
}
fn validate_defect_relic_counters(context:&CombatItemContext)->Result<(),InputError>{
    if context.counters.get(&RelicId::Inserter).is_some_and(|n|!context.owns_relic(RelicId::Inserter)||!(-1..=1).contains(n)){
        return Err(InputError::Invariant("M20 Inserter counter invalid".into()))
    }Ok(())
}

/// M34s-b P6 (final review M2): every field of the combat item context a public root carries, classified by exhaustive
/// destructuring (a new field fails to compile until classified). No field holds an already-sampled hidden value: the
/// profile flags and the battle context are public entry facts; relics, potions, potencies, counters, local relic state,
/// bottled card links, the last potion use, the played card's phase facts and the trigger/armed latches are public item
/// state; the heal room and ascension are public Run facts.
#[cfg(test)]
#[test]
fn combat_item_context_census_classifies_every_field() {
    let CombatItemContext { battle, startup_profile: _, watcher_progression: _, watcher_act3: _, watcher_heart: _,
        complete_cards: _, master_order: _, complete_items: _, local: _, relic_cards: _, potion_potencies: _,
        last_potion_use: _, collar_activated: _, emotion_chip_armed: _, counters: _, relics: _, potions: _,
        orichalcum_trigger: _, runic_capacitor_armed: _, kite_triggered: _, wrist_blade_bonus: _, damage_card: _,
        lantern_armed: _, heal_room: NativeHealRoom { combat: _, current_map_node: _ }, ascension: _ } = CombatItemContext::default();
    let NativeBattleContext { elite_trigger: _, boss: _, burning: _, m15: _, m20: _ } = battle;
}
