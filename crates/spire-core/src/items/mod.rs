//! Item-owned mechanics. Effects returned here are ordinary work for the shared
//! combat resolver; this module does not own or drain a second effect queue.
mod relics;
mod potions;
mod ironclad;
mod ironclad_turns;
mod silent;

#[cfg(test)]
mod tests;
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
pub(crate) fn replace_burning_blood(state:&mut RunState)->Result<(),RunError>{relics::replace_burning_blood(state)}

/// Item ownership is transferred into active full combat and returned with its
/// exit payload. Outside combat, RunState remains the permanent authority.
#[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]
pub enum CombatStartupProfile{#[default] Legacy,M21Watcher}
#[derive(Clone, Default, PartialEq, Eq)]
pub struct CombatItemContext {
    pub(crate) battle:NativeBattleContext,
    startup_profile:CombatStartupProfile,
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
    lantern_armed: bool,
    heal_room: NativeHealRoom,
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
        if self.m21_profile(){value.field("startup_profile",&self.startup_profile).field("potion_potencies",&self.potion_potencies).field("last_potion_use",&self.last_potion_use);}value.finish()
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
        if !self.shared_generated_profile()||self.owns_relic(RelicId::MarkOfBloom){return None}
        let slot=self.potions.iter().position(|potion|*potion==Some(PotionId::FairyPotion))?;
        let percent=if self.owns_relic(RelicId::SacredBark){60.0_f32}else{30.0_f32};
        let amount=((max_hp as f32)*(percent/100.0_f32)) as i32;
        self.potions[slot]=None;Some((slot,amount.max(1)))
    }

    pub fn new(relics: Vec<RelicId>, potions: Vec<Option<PotionId>>) -> Self {
        let mut counters=std::collections::BTreeMap::new();
        if relics.contains(&RelicId::NeowsLament){counters.insert(RelicId::NeowsLament,3);}
        if relics.contains(&RelicId::Inserter){counters.insert(RelicId::Inserter,0);}
        Self { potion_potencies:vec![],last_potion_use:None,startup_profile:CombatStartupProfile::Legacy,counters, relics, potions, runic_capacitor_armed:false,orichalcum_trigger:false, kite_triggered:false,wrist_blade_bonus:0,
            lantern_armed: false, heal_room: NativeHealRoom::default(),battle:NativeBattleContext::default(),collar_activated:false,emotion_chip_armed:false }
    }

    /// Move permanent inventory into the single active combat owner.
    pub(crate) fn take_from_run(state: &mut RunState) -> Self {
        Self { potion_potencies:vec![],last_potion_use:None,startup_profile:CombatStartupProfile::Legacy,runic_capacitor_armed:false,battle:NativeBattleContext::default(),collar_activated:false,emotion_chip_armed:false,counters:std::mem::take(&mut state.relic_counters),relics: std::mem::take(&mut state.relics),
            potions: std::mem::take(&mut state.potions), orichalcum_trigger:false,kite_triggered:false,wrist_blade_bonus:0,lantern_armed: false,
            heal_room: NativeHealRoom::from_run(state) }
    }

    /// Return active ownership once without replacing another live inventory.
    pub(crate) fn return_to_run(self, state: &mut RunState) -> Result<(), RunError> {
        if !state.relics.is_empty() || !state.potions.is_empty() || !state.relic_counters.is_empty() {
            return Err(RunError::InvalidAction);
        }
        state.relics = self.relics;
        state.potions = self.potions;
        state.relic_counters=self.counters;
        Ok(())
    }

    pub(crate) fn with_startup_profile(mut self,profile:CombatStartupProfile)->Self{self.startup_profile=profile;self.reinitialize_potion_potencies();self}
    pub(crate) fn reinitialize_potion_potencies(&mut self){
        self.potion_potencies=if self.m21_profile(){self.potions.iter().map(|p|p.map(|id|self.initial_potency(id))).collect()}else{vec![]};
    }
    fn initial_potency(&self,id:PotionId)->i32{
        let base=match id{PotionId::StancePotion=>0,PotionId::Ambrosia|PotionId::BottledMiracle|PotionId::EnergyPotion|PotionId::DexterityPotion=>2,PotionId::BlockPotion=>12,PotionId::AncientPotion=>1,_=>0};
        base*if self.owns_relic(RelicId::SacredBark){2}else{1}
    }
    pub(crate) fn potion_potency(&self,slot:usize)->Option<i32>{self.potion_potencies.get(slot).copied().flatten()}
    pub(crate) fn consumed_potion_potency(&self,slot:usize,id:PotionId)->Option<i32>{
        self.last_potion_use.filter(|(s,p,_)|*s==slot&&*p==id&&self.potions.get(slot)==Some(&None)).map(|(_,_,amount)|amount)
    }
    pub(crate) fn m21_profile(&self)->bool{self.startup_profile==CombatStartupProfile::M21Watcher}
    pub(crate) fn owns_relic(&self, id: RelicId) -> bool { self.relics.contains(&id) }
    pub(crate) fn valid_potion_capacity(&self) -> bool {
        valid_potion_capacity(&self.relics, &self.potions)
    }
    pub(crate) fn validate_potion_obtain_input(&self, id: PotionId) -> Result<(), InputError> {
        validate_potion_obtain_input(&self.relics,&self.potions,id)
    }
    /// Ordinary first-empty-slot obtain. Sozu is checked by the accepted
    /// effect/run consumer at dispatch, never by this inventory owner.
    pub(crate) fn obtain_potion(&mut self, id: PotionId) -> Result<bool, InputError> {
        self.validate_potion_obtain_input(id)?;
        if let Some(slot) = self.potions.iter().position(Option::is_none) {
            self.potions[slot] = Some(id);
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
    pub(crate) fn relics(&self) -> &[RelicId] { &self.relics }
    pub(crate) fn potions(&self) -> &[Option<PotionId>] { &self.potions }
    pub(crate) fn counters(&self)->&std::collections::BTreeMap<RelicId,i32>{&self.counters}
    pub(crate) fn counter(&self,id:RelicId)->i32{self.counters.get(&id).copied().unwrap_or(0)}
    pub(crate) fn set_counter(&mut self,id:RelicId,value:i32){if value==0&&id!=RelicId::Inserter{self.counters.remove(&id);}else{self.counters.insert(id,value);}}
    pub(crate) fn with_counters(mut self,counters:std::collections::BTreeMap<RelicId,i32>)->Self{self.counters=counters;self}
    pub(crate) fn heal_room(&self) -> NativeHealRoom { self.heal_room }

    pub(crate) fn with_native_room(mut self, phase: PotionRoomPhase,
        map: Option<&crate::run::RunMap>) -> Self {
        self.heal_room = NativeHealRoom::from_map(phase, map);
        self
    }

    /// A typed independent combat room supplies the same healing entitlement as
    /// an entered Run combat node, without constructing a fake map.
    pub(crate) fn with_training_room(mut self)->Self{self.heal_room=NativeHealRoom{combat:true,current_map_node:true};self}

    fn with_potions(&self, potions: Vec<Option<PotionId>>) -> Self {
        Self { potion_potencies:if self.m21_profile(){potions.iter().enumerate().map(|(i,id)|id.map(|id|if self.potions.get(i)==Some(&Some(id)){self.potion_potency(i).unwrap_or_else(||self.initial_potency(id))}else{self.initial_potency(id)})).collect()}else{vec![]},
            last_potion_use:if self.m21_profile(){self.potions.iter().zip(&potions).enumerate().find_map(|(slot,(old,new))|if new.is_none(){old.map(|id|(slot,id,self.potion_potency(slot).unwrap_or_else(||self.initial_potency(id))))}else{None}).or(self.last_potion_use)}else{None},
            startup_profile:self.startup_profile,runic_capacitor_armed:self.runic_capacitor_armed,battle:self.battle,collar_activated:self.collar_activated,emotion_chip_armed:self.emotion_chip_armed,counters:self.counters.clone(),relics: self.relics.clone(), potions,
            orichalcum_trigger:self.orichalcum_trigger,kite_triggered:self.kite_triggered,wrist_blade_bonus:self.wrist_blade_bonus,lantern_armed: self.lantern_armed,
            heal_room: self.heal_room }
    }
}

fn valid_potion_capacity(relics: &[RelicId], potions: &[Option<PotionId>]) -> bool {
    if relics.contains(&RelicId::PotionBelt) { potions.len() == 5 } else { potions.len() == 3 }
}

fn validate_potion_obtain_input(relics:&[RelicId],potions:&[Option<PotionId>],id:PotionId)->Result<(),InputError>{
    if !valid_potion_capacity(relics,potions)||potions.iter().flatten().any(|id|crate::content::Catalog::standard().potion(*id).is_none())
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
    relics::on_run_hook(state, hook)
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

pub(crate) fn ordinary_heal_amount_in_room(relics: &[RelicId], amount: i32,
    room: NativeHealRoom) -> i32 {
    if relics.contains(&RelicId::MarkOfBloom) { return 0; }
    if relics.contains(&RelicId::MagicFlower) && room.is_live_combat_with_node() {
        let scaled = (amount as f32) * 1.5_f32;
        return (scaled + 0.5_f32).floor() as i32;
    }
    amount
}

/// Produce ordered native effects for a combat lifecycle checkpoint.
pub(crate) fn relic_hook_effects(context: &mut CombatItemContext, hook: Hook, hp: i32,max_hp:i32,block:i32) -> RelicHookWork {
    let mut work=relics::on_combat_hook(context, hook, hp,block);
    let silent=silent::on_combat_hook(context,hook);
    work.top.extend(silent.top);work.bottom.extend(silent.bottom);
    let ic=ironclad::on_hook(context,hook,hp,max_hp);
    work.top.extend(ic.top);work.bottom.extend(ic.bottom);work.direct_strength=ic.direct_strength;
    work.top.extend(ironclad_turns::relic_hook(context,hook));
    work
}

pub(crate) fn relic_damage_effects(context:&mut CombatItemContext,target:&crate::combat::Actor,hp_lost:i32,hp_after:i32,max_hp:i32)->Result<Vec<Effect>,crate::combat::InputError>{ironclad::on_damage(context,target,hp_lost,hp_after,max_hp)}
pub(crate) fn relic_hp_state_effects(context:&mut CombatItemContext,hp:i32,max_hp:i32)->Vec<Effect>{ironclad::on_hp_state(context,hp,max_hp)}
pub(crate) fn relic_exhaust_effects(context:&CombatItemContext,source:&crate::combat::Source,enemies:&[crate::combat::EnemyHandle])->Vec<Effect>{ironclad::on_exhaust(context,source,enemies)}

pub(crate) fn brimstone_enemy_work(context:&CombatItemContext,hook:Hook,enemies:&[EnemyHandle])->Vec<Effect>{ironclad_turns::brimstone_enemy_work(context,hook,enemies)}
pub(crate) fn runic_cube_draw(context:&CombatItemContext,positive_hp_lost:i32)->Option<Effect>{ironclad_turns::runic_cube_draw(context,positive_hp_lost)}

pub(crate) fn battle_start_enemy_effects(context:&CombatItemContext,targets:&[EnemyHandle])->Vec<Effect>{
    let mut work=relics::battle_start_enemy_effects(context,targets);
    work.extend(silent::battle_start_enemy_effects(context,targets));work
}

/// Relic callback bridge used by the shared Death hook. The caller snapshots
/// Poison and the live eligible count at callback time; Specimen target choice
/// remains deferred to the typed effect's ordinary resolver dispatch.
pub(crate) fn silent_death_work(context:&CombatItemContext,dead_owner:EnemyHandle,
    callback_poison:Option<i32>,live_eligible_count:usize)->Vec<Effect>{
    silent::death_work(context,dead_owner,callback_poison,live_eligible_count)
}

/// M15-only generated hand work runs through the ordinary CreateCard resolver
/// before Combat's existing opening draw.
pub(crate) fn silent_pre_draw_work(context:&CombatItemContext)->Vec<Effect>{silent::pre_draw_work(context)}

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
    if potion==Some(PotionId::ExplosivePotion){
        for effect in &mut prepared.effects {
            if let (Some(crate::combat::Actor::Enemy(owner)),crate::combat::EffectKind::Damage(
                crate::combat::DamagePayload::Fixed(amount)))=(&effect.target,&mut effect.kind){
                *amount=combat.normal_potion_damage_to(owner,*amount)?;
            }
        }
    }
    if context.owns_relic(RelicId::ToyOrnithopter) {
        prepared.effects.push(Effect { source: crate::combat::Source::None,
            target: Some(crate::combat::Actor::Player), kind: crate::combat::EffectKind::Heal(5) });
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
    combat.enqueue_native_effects(prepared.effects,
        Some(context.with_potions(prepared.potions)))
}

/// Native noncombat use while RunState is the sole owner of HP and potion slots.
/// This deliberately does not create a public action or advance room progression.
pub(crate) fn use_run_potion(state: &mut RunState, slot: usize,
    turn_ended: bool, we_meet_again: bool) -> Result<(), RunError> {
    if (crate::run::rooms::is_m15(state)||crate::run::rooms::is_m20(state))&&state.potions.get(slot).copied().flatten()==Some(PotionId::EntropicBrew){
        if turn_ended{return Err(RunError::InvalidAction)}
        return use_m15_entropic_brew_outside(state,slot,we_meet_again)
    }
    let mut observed=crate::combat::observe::run_enabled(state).then(Vec::new);
    let (hp,max_hp,potions)=prepare_run_potion_inner(state,slot,turn_ended,we_meet_again,|before,after,source| {
        if let Some(values)=observed.as_mut(){values.push((before,after,source.clone()));}
    })?;
    state.hp=hp;state.max_hp=max_hp;state.potions=potions;
    // Preparation is also used by legal menus. Publish its detached primitive
    // checkpoints only after this real use commits, never during menu queries.
    if let Some(values)=observed {for (before,after,source) in values {crate::combat::observe::run_heal_values(state,before,after,&source);}}
    Ok(())
}

/// Outside-combat first-empty-slot obtain. This inventory helper deliberately
/// has no Sozu branch; accepted generated obtains check Sozu at execution.
pub(crate) fn obtain_run_potion(state: &mut RunState, id: PotionId) -> Result<bool, RunError> {
    validate_potion_obtain_input(&state.relics,&state.potions,id)
        .map_err(|_|RunError::Unsupported("generated potion obtain input invalid"))?;
    if let Some(slot) = state.potions.iter().position(Option::is_none) {
        state.potions[slot] = Some(id);
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Apply a proposal after an outside continuation has already been accepted.
/// Any invalid proposal retains the first Run fault and the successful prefix.
pub(crate) fn apply_accepted_run_potion_obtain(state: &mut RunState, id: PotionId) -> Result<bool, RunError> {
    if let Some(fault) = &state.fault { return Err(RunError::Faulted(fault.clone())); }
    let prepared=validate_potion_obtain_input(&state.relics,&state.potions,id)
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
        apply_run_potion_use_hook(state);
        return Ok(());
    }

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
    apply_run_potion_use_hook(state);
    for id in proposals {
        apply_accepted_run_potion_obtain(state, id)?;
    }
    Ok(())
}

fn validate_m15_entropic_brew_outside(state:&RunState,slot:usize,we_meet_again:bool)->Result<(),RunError>{
    if let Some(fault) = &state.fault { return Err(RunError::Faulted(fault.clone())); }
    if state.terminal.is_some() { return Err(RunError::Terminal); }
    if state.combat.is_some() || !(state.character == Character::Silent
        || state.character == Character::Defect && crate::run::rooms::is_m20(state)) || we_meet_again
        || state.max_hp == 0 || state.max_hp > 999 || state.hp > state.max_hp
        || state.potions.get(slot).copied().flatten() != Some(PotionId::EntropicBrew)
        || !valid_potion_capacity(&state.relics, &state.potions)
        || !state.potions.iter().flatten().all(|owned| crate::content::Catalog::standard().potion(*owned).is_some()) {
        return Err(RunError::InvalidAction);
    }
    Ok(())
}
pub(crate) fn can_use_run_potion(state:&RunState,slot:usize,turn_ended:bool,we_meet_again:bool)->bool{
    prepare_run_potion(state,slot,turn_ended,we_meet_again).is_ok()
        ||(crate::run::rooms::is_m15(state)||crate::run::rooms::is_m20(state))&&!turn_ended&&validate_m15_entropic_brew_outside(state,slot,we_meet_again).is_ok()
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
    let owned = CombatItemContext::new(state.relics.clone(), state.potions.clone());
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
#[cfg(test)]
mod m15_potion_generation_tests;

/// Relic atPreBattle requests. The shared engine owns invocation and dispatch.
/// No slots or orbs are stored by this callback owner.
pub(crate) fn defect_pre_battle_effects(context:&mut CombatItemContext)->Result<Vec<Effect>,InputError>{
    if !context.m20_profile(){return Ok(vec![])}
    validate_defect_relic_counters(context)?;
    context.runic_capacitor_armed=context.owns_relic(RelicId::RunicCapacitor);
    let mut work=Vec::new();
    for relic in context.relics(){
        let orb=match relic{RelicId::SymbioticVirus=>crate::combat::OrbKind::Dark,RelicId::NuclearBattery=>crate::combat::OrbKind::Plasma,_=>continue};
        work.push(defect_relic_effect(crate::combat::CharacterEffect::Channel(orb)));
    }
    Ok(work)
}
/// Exactly one source relic atTurnStart pass, before player powers/orbs.
/// Counter commit occurs even when the later ordinary Slots request caps at10.
pub(crate) fn defect_turn_start_effects(context:&mut CombatItemContext)->Result<Vec<Effect>,InputError>{
    if !context.m20_profile(){return Ok(vec![])}
    validate_defect_relic_counters(context)?;
    let mut work=Vec::new();
    for index in 0..context.relics.len(){
        match context.relics[index]{
            RelicId::RunicCapacitor if context.runic_capacitor_armed=>{
                context.runic_capacitor_armed=false;work.push(defect_relic_effect(crate::combat::CharacterEffect::Slots(3)));
            },
            RelicId::Inserter=>{
                let old=context.counter(RelicId::Inserter);let next=if old<0{1}else{old+1};
                context.set_counter(RelicId::Inserter,if next==2{0}else{next});
                if next==2{work.push(defect_relic_effect(crate::combat::CharacterEffect::Slots(1)));}
            },
            _=>{},
        }
    }
    Ok(work)
}
/// `has_empty_orb` is the real slot owner's source callback-time predicate.
/// The ordinary channel later owns full-slot eviction and any accepted fault.
pub(crate) fn defect_end_turn_work(context:&CombatItemContext,has_empty_orb:bool)->Vec<Effect>{
    if context.m20_profile()&&context.owns_relic(RelicId::FrozenCore)&&has_empty_orb{
        vec![defect_relic_effect(crate::combat::CharacterEffect::Channel(crate::combat::OrbKind::Frost))]
    }else{vec![]}
}
fn defect_relic_effect(effect:crate::combat::CharacterEffect)->Effect{
    Effect{source:crate::combat::Source::None,target:Some(crate::combat::Actor::Player),kind:crate::combat::EffectKind::Character(effect)}
}
fn validate_defect_relic_counters(context:&CombatItemContext)->Result<(),InputError>{
    if context.counters.get(&RelicId::Inserter).is_some_and(|n|!context.owns_relic(RelicId::Inserter)||!(-1..=1).contains(n)){
        return Err(InputError::Invariant("M20 Inserter counter invalid".into()))
    }Ok(())
}
