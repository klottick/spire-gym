//! Explicit finite Ironclad A0 combat entry. Public resources, never snapshots.
use super::*;
use crate::content::{CardId,RelicId,PotionId,EncounterId,Character};

pub const TRAINING_CARDS:&[CardId]=&[CardId::Armaments,CardId::SearingBlow,CardId::BodySlam,CardId::Carnage,CardId::Clash,CardId::Hemokinesis,CardId::Pummel,CardId::Uppercut,CardId::Immolate,CardId::SwordBoomerang,CardId::WildStrike,CardId::Entrench,CardId::GhostlyArmor,CardId::Intimidate,CardId::PowerThrough,CardId::RecklessCharge,CardId::SeeingRed,CardId::Shockwave,CardId::SpotWeakness,CardId::TrueGrit,CardId::Headbutt,CardId::Warcry,CardId::BattleTrance,CardId::Exhume,CardId::DualWield,CardId::Havoc,CardId::DoubleTap,CardId::Whirlwind,CardId::Inflame,CardId::Evolve,CardId::FireBreathing,CardId::FlameBarrier,CardId::Juggernaut,CardId::Rage,CardId::Rupture,CardId::Corruption,CardId::DarkEmbrace,CardId::FeelNoPain,CardId::Sentinel,CardId::SecondWind,CardId::SeverSoul,CardId::FiendFire,CardId::Barricade,CardId::Berserk,CardId::Brutality,CardId::DemonForm,CardId::Metallicize,CardId::Combust,CardId::Strike,CardId::Defend,CardId::Bash,CardId::Anger,CardId::IronWave,CardId::Cleave,CardId::PommelStrike,CardId::ShrugItOff,CardId::TwinStrike,CardId::Clothesline,CardId::HeavyBlade,CardId::PerfectedStrike,CardId::Thunderclap,CardId::Bloodletting,CardId::BurningPact,CardId::Disarm,CardId::Bludgeon,CardId::Feed,CardId::Reaper,CardId::Impervious,CardId::Offering,CardId::Flex,CardId::InfernalBlade,CardId::LimitBreak,CardId::BloodForBlood,CardId::Rampage,CardId::Dropkick];
pub const TRAINING_RELICS:&[RelicId]=&[RelicId::Vajra,RelicId::OddlySmoothStone,RelicId::BagOfPreparation,RelicId::BagOfMarbles,RelicId::BronzeScales,RelicId::BurningBlood,RelicId::BloodVial,RelicId::Anchor,RelicId::Lantern,RelicId::Strawberry,RelicId::Pear,RelicId::Mango,RelicId::SacredBark,RelicId::MarkOfBloom,RelicId::MagicFlower,RelicId::ToyOrnithopter,RelicId::NeowsLament,RelicId::BlackBlood,RelicId::EmptyCage,RelicId::SlaversCollar,RelicId::RedMask,RelicId::RedSkull,RelicId::PaperPhrog,RelicId::ChampionsBelt,RelicId::CharonsAshes,RelicId::SelfFormingClay,RelicId::Orichalcum,RelicId::Brimstone,RelicId::MarkOfPain,RelicId::RunicCube];
pub const TRAINING_POTIONS:&[PotionId]=&[PotionId::FirePotion,PotionId::BloodPotion,PotionId::FruitJuice,PotionId::BlockPotion,PotionId::EnergyPotion,PotionId::StrengthPotion,PotionId::SwiftPotion,PotionId::WeakPotion,PotionId::FearPotion,PotionId::DexterityPotion,PotionId::AncientPotion,PotionId::Elixir,PotionId::HeartOfIron];
pub use crate::combat::enemies::A0_ENCOUNTER_SETUPS as TRAINING_ENCOUNTERS;

#[derive(Clone,Debug,PartialEq,Eq)]
pub struct CombatTrainingEntry {
    pub encounter:EncounterId,pub hp:u32,pub max_hp:u32,
    /// Unordered multiset: the engine canonicalizes before its seeded shuffle.
    pub deck:Vec<(CardId,u32)>,pub relics:Vec<RelicId>,pub potions:Vec<Option<PotionId>>,
    /// Typed public counter, only when Neow's Lament is equipped.
    pub neows_lament_remaining:Option<i32>,
}
impl CombatTrainingEntry {
    pub fn validate(&self)->Result<(),RunError>{
        if self.hp==0||self.hp>self.max_hp||self.max_hp>999||!(1..=128).contains(&self.deck.len())||self.potions.len()!=3
            ||!TRAINING_ENCOUNTERS.iter().any(|(e,_,_)|*e==self.encounter){return Err(RunError::InvalidAction)}
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
pub struct CombatCheckpoint {pub record:CombatRecord,pub hp_before_healing:u32,pub hp_after_cleanup:u32}

impl Run {
    /// Prepare resources without entering combat. The caller must retain this
    /// owner before `enter_training_combat`, which can accept effects then fault.
    pub fn prepare_combat_training(mut entry:CombatTrainingEntry,rng:RunRng)->Result<Self,RunError>{
        entry.validate()?;
        entry.deck.sort_by_key(|(id,upgrades)|(id.as_str(),*upgrades));entry.relics.sort_by_key(|id|id.as_str());
        let (_,act,_)=TRAINING_ENCOUNTERS.iter().find(|(e,_,_)|*e==entry.encounter).unwrap();
        let deck=entry.deck.into_iter().enumerate().map(|(i,(id,upgrades))|CardInstance{id,upgrades,instance:i as u64+1}).collect::<Vec<_>>();
        let next_instance=deck.len() as u64+1;
        let mut counters=std::collections::BTreeMap::new();
        if entry.relics.contains(&RelicId::NeowsLament){counters.insert(RelicId::NeowsLament,entry.neows_lament_remaining.unwrap_or(3));}
        let resources=AuthoredRunStart{hp:entry.hp,max_hp:entry.max_hp,gold:0,deck,relics:entry.relics,potions:entry.potions,relic_counters:counters};
        let mut run=Self::from_validated_combat_resources(RunConfig{character:Character::Ironclad,ascension:0},resources,rng,next_instance);
        run.state.training_encounter=Some(entry.encounter);run.state.act=*act;run.state.room="CombatTraining";
        Ok(run)
    }
    pub fn enter_training_combat(&mut self,encounter:EncounterId)->Result<RunTransition,RunError>{
        if self.state.training_encounter!=Some(encounter)||self.state.combat_progress.entered!=0{return Err(RunError::InvalidAction)}
        let (_,_,kind)=TRAINING_ENCOUNTERS.iter().find(|(e,_,_)|*e==encounter).ok_or(RunError::InvalidAction)?;
        let kind=match kind{crate::combat::enemies::EncounterKind::Monster=>MapNodeKind::Monster,crate::combat::enemies::EncounterKind::Elite=>MapNodeKind::Elite,crate::combat::enemies::EncounterKind::Boss=>MapNodeKind::Boss};
        self.enter_owned(encounter,CombatOrigin::Training{kind})
    }
    pub fn combat_checkpoint(&self)->Option<CombatCheckpoint>{self.state.last_combat_checkpoint.clone()}
}
