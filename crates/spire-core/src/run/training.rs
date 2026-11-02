//! Explicit finite Ironclad A0 combat entry. Public resources, never snapshots.
use super::*;
use crate::content::{CardId,RelicId,PotionId,EncounterId,Character};

pub const TRAINING_CARDS:&[CardId]=&[CardId::Armaments,CardId::SearingBlow,CardId::BodySlam,CardId::Carnage,CardId::Clash,CardId::Hemokinesis,CardId::Pummel,CardId::Uppercut,CardId::Immolate,CardId::SwordBoomerang,CardId::WildStrike,CardId::Entrench,CardId::GhostlyArmor,CardId::Intimidate,CardId::PowerThrough,CardId::RecklessCharge,CardId::SeeingRed,CardId::Shockwave,CardId::SpotWeakness,CardId::TrueGrit,CardId::Headbutt,CardId::Warcry,CardId::BattleTrance,CardId::Exhume,CardId::DualWield,CardId::Havoc,CardId::DoubleTap,CardId::Whirlwind,CardId::Inflame,CardId::Evolve,CardId::FireBreathing,CardId::FlameBarrier,CardId::Juggernaut,CardId::Rage,CardId::Rupture,CardId::Corruption,CardId::DarkEmbrace,CardId::FeelNoPain,CardId::Sentinel,CardId::SecondWind,CardId::SeverSoul,CardId::FiendFire,CardId::Barricade,CardId::Berserk,CardId::Brutality,CardId::DemonForm,CardId::Metallicize,CardId::Combust,CardId::Strike,CardId::Defend,CardId::Bash,CardId::Anger,CardId::IronWave,CardId::Cleave,CardId::PommelStrike,CardId::ShrugItOff,CardId::TwinStrike,CardId::Clothesline,CardId::HeavyBlade,CardId::PerfectedStrike,CardId::Thunderclap,CardId::Bloodletting,CardId::BurningPact,CardId::Disarm,CardId::Bludgeon,CardId::Feed,CardId::Reaper,CardId::Impervious,CardId::Offering,CardId::Flex,CardId::InfernalBlade,CardId::LimitBreak,CardId::BloodForBlood,CardId::Rampage,CardId::Dropkick];
pub const TRAINING_RELICS:&[RelicId]=&[RelicId::Vajra,RelicId::OddlySmoothStone,RelicId::BagOfPreparation,RelicId::BagOfMarbles,RelicId::BronzeScales,RelicId::BurningBlood,RelicId::BloodVial,RelicId::Anchor,RelicId::Lantern,RelicId::Strawberry,RelicId::Pear,RelicId::Mango,RelicId::SacredBark,RelicId::MarkOfBloom,RelicId::MagicFlower,RelicId::ToyOrnithopter,RelicId::NeowsLament,RelicId::BlackBlood,RelicId::EmptyCage,RelicId::SlaversCollar,RelicId::RedMask,RelicId::RedSkull,RelicId::PaperPhrog,RelicId::ChampionsBelt,RelicId::CharonsAshes,RelicId::SelfFormingClay,RelicId::Orichalcum,RelicId::Brimstone,RelicId::MarkOfPain,RelicId::RunicCube];
pub const TRAINING_POTIONS:&[PotionId]=&[PotionId::FirePotion,PotionId::BloodPotion,PotionId::FruitJuice,PotionId::BlockPotion,PotionId::EnergyPotion,PotionId::StrengthPotion,PotionId::SwiftPotion,PotionId::WeakPotion,PotionId::FearPotion,PotionId::DexterityPotion,PotionId::AncientPotion,PotionId::Elixir,PotionId::HeartOfIron];
// The original profile keeps exactly its accepted prefix and RNG keys.
pub const TRAINING_ENCOUNTERS:&[(EncounterId,u8,crate::combat::enemies::EncounterKind)]=
    crate::combat::enemies::A0_ENCOUNTER_SETUPS.split_at(43).0;
pub const TRAINING_ENCOUNTERS_V2:&[(EncounterId,u8,crate::combat::enemies::EncounterKind)]=
    crate::combat::enemies::A0_ENCOUNTER_SETUPS.split_at(57).0;

#[derive(Clone,Debug,PartialEq,Eq)]
pub struct CombatTrainingEntry {
    pub encounter:EncounterId,pub hp:u32,pub max_hp:u32,
    /// Unordered multiset: the engine canonicalizes before its seeded shuffle.
    pub deck:Vec<(CardId,u32)>,pub relics:Vec<RelicId>,pub potions:Vec<Option<PotionId>>,
    /// Typed public counter, only when Neow's Lament is equipped.
    pub neows_lament_remaining:Option<i32>,
}
impl CombatTrainingEntry {
    pub fn validate(&self)->Result<(),RunError>{self.validate_encounters(TRAINING_ENCOUNTERS)}
    pub fn validate_v2(&self)->Result<(),RunError>{self.validate_encounters(TRAINING_ENCOUNTERS_V2)}
    fn validate_encounters(&self,encounters:&[(EncounterId,u8,crate::combat::enemies::EncounterKind)])->Result<(),RunError>{
        if self.hp==0||self.hp>self.max_hp||self.max_hp>999||!(1..=128).contains(&self.deck.len())||self.potions.len()!=3
            ||!encounters.iter().any(|(e,_,_)|*e==self.encounter){return Err(RunError::InvalidAction)}
        if self.deck.iter().any(|(c,u)|!TRAINING_CARDS.contains(c)||*u>if *c==CardId::SearingBlow{10}else{1}){
            return Err(RunError::Unsupported("card or upgrade outside combat training profile"))
        }
        let mut seen=std::collections::BTreeSet::new();
        if self.relics.iter().any(|r|!TRAINING_RELICS.contains(r)||!seen.insert(*r))
            ||self.relics.contains(&RelicId::BurningBlood)&&self.relics.contains(&RelicId::BlackBlood)
            ||self.potions.iter().flatten().any(|p|!TRAINING_POTIONS.contains(p)){
            return Err(RunError::InvalidAction)
        }
        if let Some(n)=self.neows_lament_remaining {
            if !self.relics.contains(&RelicId::NeowsLament)||!matches!(n,1..=3|-2){return Err(RunError::InvalidAction)}
        }
        Ok(())
    }
}
/// Public durable settlement facts. The associated numeric decision owns the
/// post-cleanup permanent deck/inventory/max HP. No terminal resolver is leaked.
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct CombatCheckpoint {pub record:CombatRecord,pub hp_before_healing:u32,pub hp_after_cleanup:u32,
    /// Earned but not claimed; unlike ordinary gold these receipts are not multiplied.
    pub stolen_gold_rewards:Vec<u32>,pub all_enemies_escaped:bool,pub mugged:bool}

impl Run {
    /// Prepare resources without entering combat. The caller must retain this
    /// owner before `enter_training_combat`, which can accept effects then fault.
    pub fn prepare_combat_training(entry:CombatTrainingEntry,rng:RunRng)->Result<Self,RunError>{
        entry.validate()?;Self::prepare_validated_training(entry,0,rng)
    }
    /// Additive encounter closure profile; original v1 validation stays frozen.
    pub fn prepare_combat_training_v2(entry:CombatTrainingEntry,rng:RunRng)->Result<Self,RunError>{
        Self::prepare_combat_training_v2_with_gold(entry,0,rng)
    }
    /// Currency is already owned, just like the supplied HP and deck.
    pub fn prepare_combat_training_v2_with_gold(entry:CombatTrainingEntry,gold:u32,rng:RunRng)->Result<Self,RunError>{
        entry.validate_v2()?;Self::prepare_validated_training(entry,gold,rng)
    }
    fn prepare_validated_training(mut entry:CombatTrainingEntry,gold:u32,rng:RunRng)->Result<Self,RunError>{
        entry.deck.sort_by_key(|(id,upgrades)|(id.as_str(),*upgrades));entry.relics.sort_by_key(|id|id.as_str());
        let (_,act,_)=TRAINING_ENCOUNTERS_V2.iter().find(|(e,_,_)|*e==entry.encounter).unwrap();
        let deck=entry.deck.into_iter().enumerate().map(|(i,(id,upgrades))|CardInstance{ misc:0,id,upgrades,instance:i as u64+1}).collect::<Vec<_>>();
        let next_instance=deck.len() as u64+1;
        let mut counters=std::collections::BTreeMap::new();
        if entry.relics.contains(&RelicId::NeowsLament){counters.insert(RelicId::NeowsLament,entry.neows_lament_remaining.unwrap_or(3));}
        let resources=AuthoredRunStart{hp:entry.hp,max_hp:entry.max_hp,gold,deck,relics:entry.relics,potions:entry.potions,relic_counters:counters};
        let mut run=Self::from_validated_combat_resources(RunConfig{character:Character::Ironclad,ascension:0},resources,rng,next_instance);
        run.state.training_encounter=Some(entry.encounter);run.state.act=*act;run.state.room="CombatTraining";
        Ok(run)
    }
    pub fn enter_training_combat(&mut self,encounter:EncounterId)->Result<RunTransition,RunError>{
        if self.state.training_encounter!=Some(encounter)||self.state.combat_progress.entered!=0{return Err(RunError::InvalidAction)}
        let (_,_,kind)=TRAINING_ENCOUNTERS_V2.iter().find(|(e,_,_)|*e==encounter).ok_or(RunError::InvalidAction)?;
        let kind=match kind{crate::combat::enemies::EncounterKind::Monster=>MapNodeKind::Monster,crate::combat::enemies::EncounterKind::Elite=>MapNodeKind::Elite,crate::combat::enemies::EncounterKind::Boss=>MapNodeKind::Boss};
        self.enter_owned(encounter,CombatOrigin::Training{kind})
    }
    pub fn combat_checkpoint(&self)->Option<CombatCheckpoint>{self.state.last_combat_checkpoint.clone()}
}

/// Finite Watcher authored resources, without hidden combat state or progression.
#[derive(Clone,Debug,PartialEq,Eq)]
pub enum M21WatcherCombatStart{Canonical,Supplied(AuthoredRunStart)}
/// The admission profile a supplied Watcher start is validated against.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum WatcherAdmission{M21,M25}
pub const M21_WATCHER_RELICS:&[RelicId]=&[RelicId::PureWater,RelicId::VioletLotus,RelicId::GoldenEye,RelicId::TeardropLocket,RelicId::SacredBark,RelicId::Vajra,RelicId::OddlySmoothStone];
pub const M21_WATCHER_POTIONS:&[PotionId]=&[PotionId::StancePotion,PotionId::Ambrosia,PotionId::BottledMiracle,PotionId::BlockPotion,PotionId::EnergyPotion,PotionId::DexterityPotion,PotionId::AncientPotion];
impl M21WatcherCombatStart{
    /// Normalize public multisets once, keeping physical permanent identities.
    pub fn normalized(self)->Result<Self,RunError>{self.normalized_for(WatcherAdmission::M21)}
    /// Supplied-resource validation under an admission profile; `M21` is `normalized()`.
    pub fn normalized_for(self,admission:WatcherAdmission)->Result<Self,RunError>{
        let Self::Supplied(mut resources)=self else{return Ok(self)};
        if resources.hp==0||resources.hp>resources.max_hp||resources.max_hp>999||!(1..=128).contains(&resources.deck.len())||resources.potions.len()!=3{return Err(RunError::InvalidAction)}
        let mut ids=std::collections::BTreeSet::new();
        if resources.deck.iter().any(|c|c.instance==0||!ids.insert(c.instance)||c.upgrades>1||c.misc!=0||!match admission{
            WatcherAdmission::M21=>crate::content::watcher::supports_permanent(c.id),
            WatcherAdmission::M25=>crate::run::M25_WATCHER_HEART_CARDS.contains(&c.id)&&(c.id!=CardId::Parasite||c.upgrades==0),
        }){
            return Err(RunError::Unsupported(match admission{WatcherAdmission::M21=>"card outside finite M21 permanent profile",WatcherAdmission::M25=>"card outside the M25 permanent union"}))
        }
        if ids.last().is_none_or(|id|*id==u64::MAX){return Err(RunError::Unsupported("permanent identity exhausted"))}
        let context=crate::items::CombatItemContext::new(resources.relics.clone(),resources.potions.clone()).with_counters(resources.relic_counters.clone());
        let context=match admission{WatcherAdmission::M21=>context.with_startup_profile(crate::items::CombatStartupProfile::M21Watcher),WatcherAdmission::M25=>context.with_m25_profile()};
        crate::items::validate_combat_inventory(&context,Character::Watcher)?;
        resources.deck.sort_by_key(|c|(c.id.as_str(),c.upgrades,c.instance));resources.relics.sort_by_key(|r|r.as_str());Ok(Self::Supplied(resources))
    }
}
/// M26 complete-card supplied start (design §5.3): character-independent, a new type
/// beside `WatcherAdmission` (not a rename). Reached only from tests and the default-off
/// diagnostics; nothing public selects `CardAdmission::Complete` in M26.
#[derive(Clone,Debug,PartialEq,Eq)]
pub enum CompleteCombatStart{Supplied(AuthoredRunStart)}
impl CompleteCombatStart{
    /// Validate supplied resources against the complete permanent union (upgrade limits of
    /// design §8.1, obtain `misc`) and the character's Heart inventory context plus the overlay.
    pub fn normalized(self,character:Character)->Result<Self,RunError>{self.normalized_for(character,ItemAdmission::Profile)}
    /// M27 design §5.3: the same normalization with the item admission as a parameter. Under
    /// `Complete` the supplied relics, potions and counters go through the one complete
    /// validator, and the supplied relic order is kept: it is the acquisition order (§6.1).
    pub(crate) fn normalized_for(self,character:Character,items:ItemAdmission)->Result<Self,RunError>{
        let Self::Supplied(mut resources)=self;
        let complete_items=items==ItemAdmission::Complete;
        if resources.hp==0||resources.hp>resources.max_hp||resources.max_hp>999||!(1..=128).contains(&resources.deck.len())||!complete_items&&resources.potions.len()!=3{return Err(RunError::InvalidAction)}
        let mut ids=std::collections::BTreeSet::new();
        if resources.deck.iter().any(|c|c.instance==0||!ids.insert(c.instance)||!crate::content::complete::supports_permanent(c.id)
            ||!complete_upgrades_valid(c.id,c.upgrades)||c.misc!=complete_obtain_misc(c.id)){
            return Err(RunError::Unsupported("card outside the complete permanent union"))
        }
        if ids.last().is_none_or(|id|*id==u64::MAX){return Err(RunError::Unsupported("permanent identity exhausted"))}
        let context=crate::items::CombatItemContext::new(resources.relics.clone(),resources.potions.clone()).with_counters(resources.relic_counters.clone());
        let context=complete_heart_context(context,character);
        let context=if complete_items{context.with_complete_items()}else{context};
        crate::items::validate_combat_inventory(&context,character)?;
        resources.deck.sort_by_key(|c|(c.id.as_str(),c.upgrades,c.instance));
        if !complete_items{resources.relics.sort_by_key(|r|r.as_str());}
        Ok(Self::Supplied(resources))
    }
}
/// Upgrade limits under complete admission (design §8.1): Searing Blow any count the
/// `u32` rule represents, curses 0, every other card 0 or 1 where it upgrades.
pub(crate) fn complete_upgrades_valid(id:CardId,upgrades:u32)->bool{
    upgrades==0||id==CardId::SearingBlow||upgrades==1&&crate::combat::cards::ironclad::rules::can_upgrade(id,0)
}
/// Initial `misc` at obtain: Genetic Algorithm 1, Ritual Dagger 15, otherwise 0.
pub(crate) fn complete_obtain_misc(id:CardId)->i32{
    match id{CardId::GeneticAlgorithm=>1,CardId::RitualDagger=>15,_=>0}
}
/// The character's complete Heart combat context (M10, M15, M20, M25) with the overlay.
pub(crate) fn complete_heart_context(context:crate::items::CombatItemContext,character:Character)->crate::items::CombatItemContext{
    match character{
        Character::Ironclad=>context,
        Character::Silent=>context.with_m15_profile(),
        Character::Defect=>context.with_m20_profile(),
        Character::Watcher=>context.with_m25_profile(),
    }.with_complete_cards()
}
impl Run{
    /// A supplied complete-card start for one A0 training encounter: the Run's admission is
    /// `Complete`, so its combat context carries `with_complete_cards()` (tests/diagnostics only).
    pub(crate) fn prepare_complete_combat(start:CompleteCombatStart,character:Character,encounter:EncounterId,rng:RunRng)->Result<Self,RunError>{
        Self::prepare_complete_admission_combat(start,character,RunAdmission::new(CardAdmission::Complete,ItemAdmission::Profile)?,encounter,rng)
    }
    /// M27: the same start under an explicit admission pair (complete cards are required; an
    /// invalid pair is rejected by `RunAdmission::new` before any Run state exists).
    pub(crate) fn prepare_complete_admission_combat(start:CompleteCombatStart,character:Character,admission:RunAdmission,encounter:EncounterId,rng:RunRng)->Result<Self,RunError>{
        if admission.cards!=CardAdmission::Complete{return Err(RunError::InvalidAction)}
        let (act,_)=m21_watcher_combat_encounter_metadata(encounter).ok_or(RunError::InvalidAction)?;
        let CompleteCombatStart::Supplied(resources)=start.normalized_for(character,admission.items)?;
        let next=resources.deck.iter().map(|c|c.instance).max().unwrap().checked_add(1).ok_or(RunError::Unsupported("permanent identity exhausted"))?;
        let mut run=Self::from_validated_combat_resources(RunConfig{character,ascension:0},resources,rng,next);
        run.state.card_admission=CardAdmission::Complete;run.state.item_admission=admission.items;run.state.training_encounter=Some(encounter);run.state.act=act;run.state.room="M26SuppliedCombat";
        if admission.items==ItemAdmission::Complete{run.state.relic_pools=Some(crate::items::relic_pools::build(&run.state,&[])?);}
        Ok(run)
    }
}
pub fn m21_watcher_combat_encounter_metadata(encounter:EncounterId)->Option<(u8,crate::combat::enemies::EncounterKind)>{
    TRAINING_ENCOUNTERS_V2.iter().find(|(id,_,_)|*id==encounter).map(|(_,act,kind)|(*act,*kind))
}
impl Run{
    pub fn prepare_m21_watcher_combat(start:M21WatcherCombatStart,encounter:EncounterId,rng:RunRng)->Result<Self,RunError>{
        let (act,_)=m21_watcher_combat_encounter_metadata(encounter).ok_or(RunError::InvalidAction)?;
        let config=RunConfig{character:Character::Watcher,ascension:0};
        let (resources,label)=match start.normalized()?{
            M21WatcherCombatStart::Canonical=>{
                let canonical=Self::new(config,CreationProfile::experienced(),rng.clone())?;let s=canonical.state;
                (AuthoredRunStart{hp:s.hp,max_hp:s.max_hp,gold:s.gold,deck:s.deck,relics:s.relics,potions:s.potions,relic_counters:s.relic_counters},"M21CanonicalCombat")
            },
            M21WatcherCombatStart::Supplied(resources)=>(resources,"M21SuppliedCombat"),
        };
        let next=resources.deck.iter().map(|c|c.instance).max().unwrap().checked_add(1).ok_or(RunError::Unsupported("permanent identity exhausted"))?;
        let mut run=Self::from_validated_combat_resources(config,resources,rng,next);run.state.training_encounter=Some(encounter);run.state.act=act;run.state.room=label;Ok(run)
    }
}
