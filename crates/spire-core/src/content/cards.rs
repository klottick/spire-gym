use super::{CardId, Character};
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CardType { Attack, Skill, Power, Status, Curse }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CardColor { Red, Green, Blue, Purple, Colorless, Status, Curse, Shared }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CardRarity { Basic, Common, Uncommon, Rare, Special, Status, Curse }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NoteStorage { RemovablePermanent, TemporaryOnly, NonremovableCurse }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContentProducer { OrdinaryReward, ShardOrdinaryReward, NoteForYourself, PureWater, Unregistered(&'static str) }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RegistryError { UnknownDefinition, UnknownProducer, IncompletePool, MissingStarterDeck }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CardCost { Energy(u8), X }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CardTarget { Enemy, AllEnemies, SelfTarget, None }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CardKeywords {
    pub exhaust: bool,
    pub innate: bool,
    pub ethereal: bool,
}

impl CardKeywords {
    pub const NONE: Self = Self { exhaust: false, innate: false, ethereal: false };
    pub const EXHAUST: Self = Self { exhaust: true, innate: false, ethereal: false };
    pub const EXHAUST_INNATE: Self = Self { exhaust: true, innate: true, ethereal: false };
    pub const INNATE: Self = Self { exhaust: false, innate: true, ethereal: false };
    pub const ETHEREAL: Self = Self { exhaust: false, innate: false, ethereal: true };
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CardTag { Strike, StarterStrike, StarterDefend, Healing }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CardFaceMetadata {
    pub cost: CardCost,
    pub target: CardTarget,
    pub keywords: CardKeywords,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BlueCardMetadata {
    pub base: CardFaceMetadata,
    pub upgraded: CardFaceMetadata,
    pub tags: &'static [CardTag],
}

#[derive(Clone, Copy)]
pub struct CardDefinition {
    pub id: CardId,
    pub color: CardColor,
    pub rarity: CardRarity,
    pub card_type: CardType,
    pub note_storage: NoteStorage,
    /// Definitions are registered, but executable support belongs to later
    /// mechanics components. No foundation row claims a run handler.
    pub definition_only: bool,
    /// Source-backed base/upgraded catalog facts for the blue card family.
    pub blue_metadata: Option<BlueCardMetadata>,
}

impl fmt::Debug for CardDefinition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CardDefinition")
            .field("id", &self.id)
            .field("color", &self.color)
            .field("rarity", &self.rarity)
            .field("card_type", &self.card_type)
            .field("note_storage", &self.note_storage)
            .field("definition_only", &self.definition_only)
            .finish()
    }
}

impl CardDefinition {
    pub fn eligible_for(&self, producer: ContentProducer, character: Character) -> Result<bool, RegistryError> {
        let character_color = match character { Character::Ironclad => CardColor::Red, Character::Silent => CardColor::Green, Character::Defect => CardColor::Blue, Character::Watcher => CardColor::Purple };
        let ordinary_pool_candidate = matches!(self.color, CardColor::Red | CardColor::Green | CardColor::Blue | CardColor::Purple | CardColor::Colorless)
            && !matches!(self.rarity, CardRarity::Basic | CardRarity::Status | CardRarity::Curse | CardRarity::Special);
        Ok(match producer {
            ContentProducer::OrdinaryReward => self.color == character_color && ordinary_pool_candidate
                && matches!(self.rarity, CardRarity::Common | CardRarity::Uncommon | CardRarity::Rare),
            ContentProducer::ShardOrdinaryReward => ordinary_pool_candidate
                && matches!(self.rarity, CardRarity::Common | CardRarity::Uncommon | CardRarity::Rare),
            ContentProducer::NoteForYourself => self.note_storage == NoteStorage::RemovablePermanent,
            ContentProducer::PureWater => self.id == CardId::Miracle,
            ContentProducer::Unregistered(_) => return Err(RegistryError::UnknownProducer),
        })
    }
}

pub(crate) const CARDS: &[CardDefinition] = &[
    supported(CardId::Armaments,CardColor::Red,CardRarity::Common,CardType::Skill,NoteStorage::RemovablePermanent),
    supported(CardId::SearingBlow,CardColor::Red,CardRarity::Uncommon,CardType::Attack,NoteStorage::RemovablePermanent),
    supported(CardId::BodySlam,CardColor::Red,CardRarity::Common,CardType::Attack,NoteStorage::RemovablePermanent),
    supported(CardId::Carnage,CardColor::Red,CardRarity::Uncommon,CardType::Attack,NoteStorage::RemovablePermanent),
    supported(CardId::Clash,CardColor::Red,CardRarity::Common,CardType::Attack,NoteStorage::RemovablePermanent),
    supported(CardId::Hemokinesis,CardColor::Red,CardRarity::Uncommon,CardType::Attack,NoteStorage::RemovablePermanent),
    supported(CardId::Pummel,CardColor::Red,CardRarity::Uncommon,CardType::Attack,NoteStorage::RemovablePermanent),
    supported(CardId::Uppercut,CardColor::Red,CardRarity::Uncommon,CardType::Attack,NoteStorage::RemovablePermanent),
    supported(CardId::Immolate,CardColor::Red,CardRarity::Rare,CardType::Attack,NoteStorage::RemovablePermanent),
    supported(CardId::SwordBoomerang,CardColor::Red,CardRarity::Common,CardType::Attack,NoteStorage::RemovablePermanent),
    supported(CardId::WildStrike,CardColor::Red,CardRarity::Common,CardType::Attack,NoteStorage::RemovablePermanent),
    supported(CardId::Entrench,CardColor::Red,CardRarity::Uncommon,CardType::Skill,NoteStorage::RemovablePermanent),
    supported(CardId::GhostlyArmor,CardColor::Red,CardRarity::Uncommon,CardType::Skill,NoteStorage::RemovablePermanent),
    supported(CardId::Intimidate,CardColor::Red,CardRarity::Uncommon,CardType::Skill,NoteStorage::RemovablePermanent),
    supported(CardId::PowerThrough,CardColor::Red,CardRarity::Uncommon,CardType::Skill,NoteStorage::RemovablePermanent),
    supported(CardId::RecklessCharge,CardColor::Red,CardRarity::Uncommon,CardType::Attack,NoteStorage::RemovablePermanent),
    supported(CardId::SeeingRed,CardColor::Red,CardRarity::Uncommon,CardType::Skill,NoteStorage::RemovablePermanent),
    supported(CardId::Shockwave,CardColor::Red,CardRarity::Uncommon,CardType::Skill,NoteStorage::RemovablePermanent),
    supported(CardId::SpotWeakness,CardColor::Red,CardRarity::Uncommon,CardType::Skill,NoteStorage::RemovablePermanent),
    supported(CardId::TrueGrit,CardColor::Red,CardRarity::Common,CardType::Skill,NoteStorage::RemovablePermanent),
    supported(CardId::Headbutt,CardColor::Red,CardRarity::Common,CardType::Attack,NoteStorage::RemovablePermanent),
    supported(CardId::Warcry,CardColor::Red,CardRarity::Common,CardType::Skill,NoteStorage::RemovablePermanent),
    supported(CardId::BattleTrance,CardColor::Red,CardRarity::Uncommon,CardType::Skill,NoteStorage::RemovablePermanent),
    supported(CardId::Exhume,CardColor::Red,CardRarity::Rare,CardType::Skill,NoteStorage::RemovablePermanent),
    supported(CardId::DualWield,CardColor::Red,CardRarity::Uncommon,CardType::Skill,NoteStorage::RemovablePermanent),
    supported(CardId::Havoc,CardColor::Red,CardRarity::Common,CardType::Skill,NoteStorage::RemovablePermanent),
    supported(CardId::DoubleTap,CardColor::Red,CardRarity::Rare,CardType::Skill,NoteStorage::RemovablePermanent),
    supported(CardId::Whirlwind,CardColor::Red,CardRarity::Uncommon,CardType::Attack,NoteStorage::RemovablePermanent),
    supported(CardId::Inflame,CardColor::Red,CardRarity::Uncommon,CardType::Power,NoteStorage::RemovablePermanent),
    supported(CardId::Evolve,CardColor::Red,CardRarity::Uncommon,CardType::Power,NoteStorage::RemovablePermanent),
    supported(CardId::FireBreathing,CardColor::Red,CardRarity::Uncommon,CardType::Power,NoteStorage::RemovablePermanent),
    supported(CardId::FlameBarrier,CardColor::Red,CardRarity::Uncommon,CardType::Skill,NoteStorage::RemovablePermanent),
    supported(CardId::Juggernaut,CardColor::Red,CardRarity::Rare,CardType::Power,NoteStorage::RemovablePermanent),
    supported(CardId::Rage,CardColor::Red,CardRarity::Uncommon,CardType::Skill,NoteStorage::RemovablePermanent),
    supported(CardId::Rupture,CardColor::Red,CardRarity::Uncommon,CardType::Power,NoteStorage::RemovablePermanent),
    supported(CardId::Corruption,CardColor::Red,CardRarity::Rare,CardType::Power,NoteStorage::RemovablePermanent),
    supported(CardId::DarkEmbrace,CardColor::Red,CardRarity::Uncommon,CardType::Power,NoteStorage::RemovablePermanent),
    supported(CardId::FeelNoPain,CardColor::Red,CardRarity::Uncommon,CardType::Power,NoteStorage::RemovablePermanent),
    supported(CardId::Sentinel,CardColor::Red,CardRarity::Uncommon,CardType::Skill,NoteStorage::RemovablePermanent),
    supported(CardId::SecondWind,CardColor::Red,CardRarity::Uncommon,CardType::Skill,NoteStorage::RemovablePermanent),
    supported(CardId::SeverSoul,CardColor::Red,CardRarity::Uncommon,CardType::Attack,NoteStorage::RemovablePermanent),
    supported(CardId::FiendFire,CardColor::Red,CardRarity::Rare,CardType::Attack,NoteStorage::RemovablePermanent),
    supported(CardId::Barricade,CardColor::Red,CardRarity::Rare,CardType::Power,NoteStorage::RemovablePermanent),
    supported(CardId::Berserk,CardColor::Red,CardRarity::Rare,CardType::Power,NoteStorage::RemovablePermanent),
    supported(CardId::Brutality,CardColor::Red,CardRarity::Rare,CardType::Power,NoteStorage::RemovablePermanent),
    supported(CardId::DemonForm,CardColor::Red,CardRarity::Rare,CardType::Power,NoteStorage::RemovablePermanent),
    supported(CardId::Metallicize,CardColor::Red,CardRarity::Uncommon,CardType::Power,NoteStorage::RemovablePermanent),
    supported(CardId::Combust,CardColor::Red,CardRarity::Uncommon,CardType::Power,NoteStorage::RemovablePermanent),
    supported(CardId::Void,CardColor::Status,CardRarity::Status,CardType::Status,NoteStorage::TemporaryOnly),
    supported(CardId::Parasite,CardColor::Curse,CardRarity::Curse,CardType::Curse,NoteStorage::RemovablePermanent),
    supported(CardId::Strike, CardColor::Red, CardRarity::Basic, CardType::Attack, NoteStorage::RemovablePermanent),
    supported(CardId::Defend, CardColor::Red, CardRarity::Basic, CardType::Skill, NoteStorage::RemovablePermanent),
    supported(CardId::Bash, CardColor::Red, CardRarity::Basic, CardType::Attack, NoteStorage::RemovablePermanent),
    supported(CardId::StrikeGreen, CardColor::Green, CardRarity::Basic, CardType::Attack, NoteStorage::RemovablePermanent),
    supported(CardId::DefendGreen, CardColor::Green, CardRarity::Basic, CardType::Skill, NoteStorage::RemovablePermanent),
    supported(CardId::Neutralize, CardColor::Green, CardRarity::Basic, CardType::Attack, NoteStorage::RemovablePermanent),
    supported(CardId::Survivor, CardColor::Green, CardRarity::Basic, CardType::Skill, NoteStorage::RemovablePermanent),
    with_blue_metadata(supported(CardId::StrikeBlue, CardColor::Blue, CardRarity::Basic, CardType::Attack, NoteStorage::RemovablePermanent), blue_metadata(CardCost::Energy(1), CardCost::Energy(1), CardTarget::Enemy, CardKeywords::NONE, CardKeywords::NONE, &[CardTag::Strike, CardTag::StarterStrike])),
    with_blue_metadata(supported(CardId::DefendBlue, CardColor::Blue, CardRarity::Basic, CardType::Skill, NoteStorage::RemovablePermanent), blue_metadata(CardCost::Energy(1), CardCost::Energy(1), CardTarget::SelfTarget, CardKeywords::NONE, CardKeywords::NONE, &[CardTag::StarterDefend])),
    with_blue_metadata(supported(CardId::Zap, CardColor::Blue, CardRarity::Basic, CardType::Skill, NoteStorage::RemovablePermanent), blue_metadata(CardCost::Energy(1), CardCost::Energy(0), CardTarget::SelfTarget, CardKeywords::NONE, CardKeywords::NONE, &[])),
    with_blue_metadata(supported(CardId::Dualcast, CardColor::Blue, CardRarity::Basic, CardType::Skill, NoteStorage::RemovablePermanent), blue_metadata(CardCost::Energy(1), CardCost::Energy(0), CardTarget::None, CardKeywords::NONE, CardKeywords::NONE, &[])),
    supported(CardId::StrikePurple, CardColor::Purple, CardRarity::Basic, CardType::Attack, NoteStorage::RemovablePermanent),
    supported(CardId::DefendPurple, CardColor::Purple, CardRarity::Basic, CardType::Skill, NoteStorage::RemovablePermanent),
    supported(CardId::Eruption, CardColor::Purple, CardRarity::Basic, CardType::Attack, NoteStorage::RemovablePermanent),
    supported(CardId::Vigilance, CardColor::Purple, CardRarity::Basic, CardType::Skill, NoteStorage::RemovablePermanent),
    supported(CardId::Anger, CardColor::Red, CardRarity::Common, CardType::Attack, NoteStorage::RemovablePermanent),
    supported(CardId::IronWave, CardColor::Red, CardRarity::Common, CardType::Attack, NoteStorage::RemovablePermanent),
    supported(CardId::Cleave, CardColor::Red, CardRarity::Common, CardType::Attack, NoteStorage::RemovablePermanent),
    supported(CardId::PommelStrike, CardColor::Red, CardRarity::Common, CardType::Attack, NoteStorage::RemovablePermanent),
    supported(CardId::ShrugItOff, CardColor::Red, CardRarity::Common, CardType::Skill, NoteStorage::RemovablePermanent),
    supported(CardId::TwinStrike, CardColor::Red, CardRarity::Common, CardType::Attack, NoteStorage::RemovablePermanent),
    supported(CardId::Backflip, CardColor::Green, CardRarity::Common, CardType::Skill, NoteStorage::RemovablePermanent),
    supported(CardId::DaggerThrow, CardColor::Green, CardRarity::Common, CardType::Attack, NoteStorage::RemovablePermanent),
    supported(CardId::Reflex, CardColor::Green, CardRarity::Uncommon, CardType::Skill, NoteStorage::RemovablePermanent),
    supported(CardId::Tactician, CardColor::Green, CardRarity::Uncommon, CardType::Skill, NoteStorage::RemovablePermanent),
    supported(CardId::Deflect, CardColor::Green, CardRarity::Common, CardType::Skill, NoteStorage::RemovablePermanent),
    supported(CardId::QuickSlash, CardColor::Green, CardRarity::Common, CardType::Attack, NoteStorage::RemovablePermanent),
    supported(CardId::CloakAndDagger, CardColor::Green, CardRarity::Common, CardType::Skill, NoteStorage::RemovablePermanent),
    supported(CardId::Shiv, CardColor::Colorless, CardRarity::Special, CardType::Attack, NoteStorage::TemporaryOnly),
    supported(CardId::Accuracy, CardColor::Green, CardRarity::Uncommon, CardType::Power, NoteStorage::RemovablePermanent),
    supported(CardId::BladeDance, CardColor::Green, CardRarity::Common, CardType::Skill, NoteStorage::RemovablePermanent),
    supported(CardId::Clothesline, CardColor::Red, CardRarity::Common, CardType::Attack, NoteStorage::RemovablePermanent),
    supported(CardId::HeavyBlade, CardColor::Red, CardRarity::Common, CardType::Attack, NoteStorage::RemovablePermanent),
    supported(CardId::PerfectedStrike, CardColor::Red, CardRarity::Common, CardType::Attack, NoteStorage::RemovablePermanent),
    supported(CardId::Thunderclap, CardColor::Red, CardRarity::Common, CardType::Attack, NoteStorage::RemovablePermanent),
    with_blue_metadata(supported(CardId::BallLightning, CardColor::Blue, CardRarity::Common, CardType::Attack, NoteStorage::RemovablePermanent), blue_metadata(CardCost::Energy(1), CardCost::Energy(1), CardTarget::Enemy, CardKeywords::NONE, CardKeywords::NONE, &[])),
    with_blue_metadata(supported(CardId::ColdSnap, CardColor::Blue, CardRarity::Common, CardType::Attack, NoteStorage::RemovablePermanent), blue_metadata(CardCost::Energy(1), CardCost::Energy(1), CardTarget::Enemy, CardKeywords::NONE, CardKeywords::NONE, &[])),
    with_blue_metadata(supported(CardId::Coolheaded, CardColor::Blue, CardRarity::Common, CardType::Skill, NoteStorage::RemovablePermanent), blue_metadata(CardCost::Energy(1), CardCost::Energy(1), CardTarget::SelfTarget, CardKeywords::NONE, CardKeywords::NONE, &[])),
    with_blue_metadata(supported(CardId::Leap, CardColor::Blue, CardRarity::Common, CardType::Skill, NoteStorage::RemovablePermanent), blue_metadata(CardCost::Energy(1), CardCost::Energy(1), CardTarget::SelfTarget, CardKeywords::NONE, CardKeywords::NONE, &[])),
    supported(CardId::Consecrate, CardColor::Purple, CardRarity::Common, CardType::Attack, NoteStorage::RemovablePermanent),
    supported(CardId::EmptyBody, CardColor::Purple, CardRarity::Common, CardType::Skill, NoteStorage::RemovablePermanent),
    supported(CardId::EmptyFist, CardColor::Purple, CardRarity::Common, CardType::Attack, NoteStorage::RemovablePermanent),
    supported(CardId::EmptyMind, CardColor::Purple, CardRarity::Uncommon, CardType::Skill, NoteStorage::RemovablePermanent),
    with_blue_metadata(supported(CardId::BeamCell, CardColor::Blue, CardRarity::Common, CardType::Attack, NoteStorage::RemovablePermanent), blue_metadata(CardCost::Energy(0), CardCost::Energy(0), CardTarget::Enemy, CardKeywords::NONE, CardKeywords::NONE, &[])),
    with_blue_metadata(supported(CardId::GoForTheEyes, CardColor::Blue, CardRarity::Common, CardType::Attack, NoteStorage::RemovablePermanent), blue_metadata(CardCost::Energy(0), CardCost::Energy(0), CardTarget::Enemy, CardKeywords::NONE, CardKeywords::NONE, &[])),
    with_blue_metadata(supported(CardId::Barrage, CardColor::Blue, CardRarity::Common, CardType::Attack, NoteStorage::RemovablePermanent), blue_metadata(CardCost::Energy(1), CardCost::Energy(1), CardTarget::Enemy, CardKeywords::NONE, CardKeywords::NONE, &[])),
    with_blue_metadata(supported(CardId::CompileDriver, CardColor::Blue, CardRarity::Common, CardType::Attack, NoteStorage::RemovablePermanent), blue_metadata(CardCost::Energy(1), CardCost::Energy(1), CardTarget::Enemy, CardKeywords::NONE, CardKeywords::NONE, &[])),
    with_blue_metadata(supported(CardId::SweepingBeam, CardColor::Blue, CardRarity::Common, CardType::Attack, NoteStorage::RemovablePermanent), blue_metadata(CardCost::Energy(1), CardCost::Energy(1), CardTarget::AllEnemies, CardKeywords::NONE, CardKeywords::NONE, &[])),
    supported(CardId::DeadlyPoison, CardColor::Green, CardRarity::Common, CardType::Skill, NoteStorage::RemovablePermanent),
    supported(CardId::PoisonedStab, CardColor::Green, CardRarity::Common, CardType::Attack, NoteStorage::RemovablePermanent),
    supported(CardId::Catalyst, CardColor::Green, CardRarity::Uncommon, CardType::Skill, NoteStorage::RemovablePermanent),
    supported(CardId::NoxiousFumes, CardColor::Green, CardRarity::Uncommon, CardType::Power, NoteStorage::RemovablePermanent),
    supported(CardId::WellLaidPlans, CardColor::Green, CardRarity::Uncommon, CardType::Power, NoteStorage::RemovablePermanent),
    supported(CardId::Bloodletting, CardColor::Red, CardRarity::Uncommon, CardType::Skill, NoteStorage::RemovablePermanent),
    supported(CardId::BurningPact, CardColor::Red, CardRarity::Uncommon, CardType::Skill, NoteStorage::RemovablePermanent),
    supported(CardId::Disarm, CardColor::Red, CardRarity::Uncommon, CardType::Skill, NoteStorage::RemovablePermanent),
    supported(CardId::Bludgeon, CardColor::Red, CardRarity::Rare, CardType::Attack, NoteStorage::RemovablePermanent),
    supported(CardId::Feed,CardColor::Red,CardRarity::Rare,CardType::Attack,NoteStorage::RemovablePermanent),
    supported(CardId::Reaper,CardColor::Red,CardRarity::Rare,CardType::Attack,NoteStorage::RemovablePermanent),
    supported(CardId::Impervious, CardColor::Red, CardRarity::Rare, CardType::Skill, NoteStorage::RemovablePermanent),
    supported(CardId::Offering, CardColor::Red, CardRarity::Rare, CardType::Skill, NoteStorage::RemovablePermanent),
    supported(CardId::Flex, CardColor::Red, CardRarity::Common, CardType::Skill, NoteStorage::RemovablePermanent),
    supported(CardId::InfernalBlade,CardColor::Red,CardRarity::Uncommon,CardType::Skill,NoteStorage::RemovablePermanent),
    supported(CardId::LimitBreak, CardColor::Red, CardRarity::Rare, CardType::Skill, NoteStorage::RemovablePermanent),
    supported(CardId::BloodForBlood, CardColor::Red, CardRarity::Uncommon, CardType::Attack, NoteStorage::RemovablePermanent),
    supported(CardId::Rampage, CardColor::Red, CardRarity::Uncommon, CardType::Attack, NoteStorage::RemovablePermanent),
    supported(CardId::Dropkick, CardColor::Red, CardRarity::Uncommon, CardType::Attack, NoteStorage::RemovablePermanent),
    supported(CardId::Slimed, CardColor::Status, CardRarity::Status, CardType::Status, NoteStorage::TemporaryOnly),
    supported(CardId::Dazed, CardColor::Status, CardRarity::Status, CardType::Status, NoteStorage::TemporaryOnly),
    supported(CardId::Burn, CardColor::Status, CardRarity::Status, CardType::Status, NoteStorage::TemporaryOnly),
    card(CardId::AscendersBane, CardColor::Curse, CardRarity::Curse, CardType::Curse, NoteStorage::NonremovableCurse),
    supported(CardId::Miracle, CardColor::Colorless, CardRarity::Special, CardType::Skill, NoteStorage::TemporaryOnly),
    supported(CardId::Wound, CardColor::Status, CardRarity::Status, CardType::Status, NoteStorage::TemporaryOnly),
    supported(CardId::Madness, CardColor::Colorless, CardRarity::Uncommon, CardType::Skill, NoteStorage::RemovablePermanent),
    card(CardId::Regret, CardColor::Curse, CardRarity::Curse, CardType::Curse, NoteStorage::RemovablePermanent),
    supported(CardId::Nightmare, CardColor::Green, CardRarity::Rare, CardType::Skill, NoteStorage::RemovablePermanent),
    supported(CardId::DaggerSpray,CardColor::Green,CardRarity::Common,CardType::Attack,NoteStorage::RemovablePermanent),
    supported(CardId::Slice,CardColor::Green,CardRarity::Common,CardType::Attack,NoteStorage::RemovablePermanent),
    supported(CardId::SuckerPunch,CardColor::Green,CardRarity::Common,CardType::Attack,NoteStorage::RemovablePermanent),
    supported(CardId::Backstab,CardColor::Green,CardRarity::Uncommon,CardType::Attack,NoteStorage::RemovablePermanent),
    supported(CardId::Dash,CardColor::Green,CardRarity::Uncommon,CardType::Attack,NoteStorage::RemovablePermanent),
    supported(CardId::LegSweep,CardColor::Green,CardRarity::Uncommon,CardType::Skill,NoteStorage::RemovablePermanent),
    supported(CardId::RiddleWithHoles,CardColor::Green,CardRarity::Uncommon,CardType::Attack,NoteStorage::RemovablePermanent),
    supported(CardId::Adrenaline,CardColor::Green,CardRarity::Rare,CardType::Skill,NoteStorage::RemovablePermanent),
    supported(CardId::DieDieDie,CardColor::Green,CardRarity::Rare,CardType::Attack,NoteStorage::RemovablePermanent),
    supported(CardId::Acrobatics,CardColor::Green,CardRarity::Common,CardType::Skill,NoteStorage::RemovablePermanent),
    supported(CardId::Prepared,CardColor::Green,CardRarity::Common,CardType::Skill,NoteStorage::RemovablePermanent),
    supported(CardId::SneakyStrike,CardColor::Green,CardRarity::Common,CardType::Attack,NoteStorage::RemovablePermanent),
    supported(CardId::AllOutAttack,CardColor::Green,CardRarity::Uncommon,CardType::Attack,NoteStorage::RemovablePermanent),
    supported(CardId::CalculatedGamble,CardColor::Green,CardRarity::Uncommon,CardType::Skill,NoteStorage::RemovablePermanent),
    supported(CardId::Concentrate,CardColor::Green,CardRarity::Uncommon,CardType::Skill,NoteStorage::RemovablePermanent),
    supported(CardId::EscapePlan,CardColor::Green,CardRarity::Uncommon,CardType::Skill,NoteStorage::RemovablePermanent),
    supported(CardId::Expertise,CardColor::Green,CardRarity::Uncommon,CardType::Skill,NoteStorage::RemovablePermanent),
    supported(CardId::Finisher,CardColor::Green,CardRarity::Uncommon,CardType::Attack,NoteStorage::RemovablePermanent),
    supported(CardId::Flechettes,CardColor::Green,CardRarity::Uncommon,CardType::Attack,NoteStorage::RemovablePermanent),
    supported(CardId::GrandFinale,CardColor::Green,CardRarity::Rare,CardType::Attack,NoteStorage::RemovablePermanent),
    supported(CardId::Unload,CardColor::Green,CardRarity::Rare,CardType::Attack,NoteStorage::RemovablePermanent),
    supported(CardId::DodgeAndRoll,CardColor::Green,CardRarity::Common,CardType::Skill,NoteStorage::RemovablePermanent),
    supported(CardId::FlyingKnee,CardColor::Green,CardRarity::Common,CardType::Attack,NoteStorage::RemovablePermanent),
    supported(CardId::Outmaneuver,CardColor::Green,CardRarity::Common,CardType::Skill,NoteStorage::RemovablePermanent),
    supported(CardId::Blur,CardColor::Green,CardRarity::Uncommon,CardType::Skill,NoteStorage::RemovablePermanent),
    supported(CardId::Caltrops,CardColor::Green,CardRarity::Uncommon,CardType::Power,NoteStorage::RemovablePermanent),
    supported(CardId::Choke,CardColor::Green,CardRarity::Uncommon,CardType::Attack,NoteStorage::RemovablePermanent),
    supported(CardId::Footwork,CardColor::Green,CardRarity::Uncommon,CardType::Power,NoteStorage::RemovablePermanent),
    supported(CardId::InfiniteBlades,CardColor::Green,CardRarity::Uncommon,CardType::Power,NoteStorage::RemovablePermanent),
    supported(CardId::Predator,CardColor::Green,CardRarity::Uncommon,CardType::Attack,NoteStorage::RemovablePermanent),
    supported(CardId::AThousandCuts,CardColor::Green,CardRarity::Rare,CardType::Power,NoteStorage::RemovablePermanent),
    supported(CardId::AfterImage,CardColor::Green,CardRarity::Rare,CardType::Power,NoteStorage::RemovablePermanent),
    supported(CardId::Envenom,CardColor::Green,CardRarity::Rare,CardType::Power,NoteStorage::RemovablePermanent),
    supported(CardId::PhantasmalKiller,CardColor::Green,CardRarity::Rare,CardType::Skill,NoteStorage::RemovablePermanent),
    supported(CardId::ToolsOfTheTrade,CardColor::Green,CardRarity::Rare,CardType::Power,NoteStorage::RemovablePermanent),
    supported(CardId::WraithForm,CardColor::Green,CardRarity::Rare,CardType::Power,NoteStorage::RemovablePermanent),
    supported(CardId::Bane,CardColor::Green,CardRarity::Common,CardType::Attack,NoteStorage::RemovablePermanent),
    supported(CardId::BouncingFlask,CardColor::Green,CardRarity::Uncommon,CardType::Skill,NoteStorage::RemovablePermanent),
    supported(CardId::PiercingWail,CardColor::Green,CardRarity::Common,CardType::Skill,NoteStorage::RemovablePermanent),
    supported(CardId::CripplingCloud,CardColor::Green,CardRarity::Uncommon,CardType::Skill,NoteStorage::RemovablePermanent),
    supported(CardId::HeelHook,CardColor::Green,CardRarity::Uncommon,CardType::Attack,NoteStorage::RemovablePermanent),
    supported(CardId::Terror,CardColor::Green,CardRarity::Uncommon,CardType::Skill,NoteStorage::RemovablePermanent),
    supported(CardId::CorpseExplosion,CardColor::Green,CardRarity::Rare,CardType::Skill,NoteStorage::RemovablePermanent),
    supported(CardId::Blind,CardColor::Colorless,CardRarity::Uncommon,CardType::Skill,NoteStorage::RemovablePermanent),
    supported(CardId::DarkShackles,CardColor::Colorless,CardRarity::Uncommon,CardType::Skill,NoteStorage::RemovablePermanent),
    supported(CardId::DeepBreath,CardColor::Colorless,CardRarity::Uncommon,CardType::Skill,NoteStorage::RemovablePermanent),
    supported(CardId::DramaticEntrance,CardColor::Colorless,CardRarity::Uncommon,CardType::Attack,NoteStorage::RemovablePermanent),
    supported(CardId::Finesse,CardColor::Colorless,CardRarity::Uncommon,CardType::Skill,NoteStorage::RemovablePermanent),
    supported(CardId::FlashOfSteel,CardColor::Colorless,CardRarity::Uncommon,CardType::Attack,NoteStorage::RemovablePermanent),
    supported(CardId::GoodInstincts,CardColor::Colorless,CardRarity::Uncommon,CardType::Skill,NoteStorage::RemovablePermanent),
    supported(CardId::Impatience,CardColor::Colorless,CardRarity::Uncommon,CardType::Skill,NoteStorage::RemovablePermanent),
    supported(CardId::MindBlast,CardColor::Colorless,CardRarity::Uncommon,CardType::Attack,NoteStorage::RemovablePermanent),
    supported(CardId::Panacea,CardColor::Colorless,CardRarity::Uncommon,CardType::Skill,NoteStorage::RemovablePermanent),
    supported(CardId::PanicButton,CardColor::Colorless,CardRarity::Uncommon,CardType::Skill,NoteStorage::RemovablePermanent),
    supported(CardId::SwiftStrike,CardColor::Colorless,CardRarity::Uncommon,CardType::Attack,NoteStorage::RemovablePermanent),
    supported(CardId::Trip,CardColor::Colorless,CardRarity::Uncommon,CardType::Skill,NoteStorage::RemovablePermanent),
    supported(CardId::HandOfGreed,CardColor::Colorless,CardRarity::Rare,CardType::Attack,NoteStorage::RemovablePermanent),
    supported(CardId::MasterOfStrategy,CardColor::Colorless,CardRarity::Rare,CardType::Skill,NoteStorage::RemovablePermanent),
    supported(CardId::Apotheosis,CardColor::Colorless,CardRarity::Rare,CardType::Skill,NoteStorage::RemovablePermanent),
    supported(CardId::Enlightenment,CardColor::Colorless,CardRarity::Uncommon,CardType::Skill,NoteStorage::RemovablePermanent),
    supported(CardId::Forethought,CardColor::Colorless,CardRarity::Uncommon,CardType::Skill,NoteStorage::RemovablePermanent),
    supported(CardId::Purity,CardColor::Colorless,CardRarity::Uncommon,CardType::Skill,NoteStorage::RemovablePermanent),
    supported(CardId::SecretTechnique,CardColor::Colorless,CardRarity::Rare,CardType::Skill,NoteStorage::RemovablePermanent),
    supported(CardId::SecretWeapon,CardColor::Colorless,CardRarity::Rare,CardType::Skill,NoteStorage::RemovablePermanent),
    supported(CardId::ThinkingAhead,CardColor::Colorless,CardRarity::Rare,CardType::Skill,NoteStorage::RemovablePermanent),
    supported(CardId::Violence,CardColor::Colorless,CardRarity::Rare,CardType::Skill,NoteStorage::RemovablePermanent),
    supported(CardId::MasterfulStab,CardColor::Green,CardRarity::Uncommon,CardType::Attack,NoteStorage::RemovablePermanent),
    supported(CardId::Eviscerate,CardColor::Green,CardRarity::Uncommon,CardType::Attack,NoteStorage::RemovablePermanent),
    supported(CardId::Setup,CardColor::Green,CardRarity::Uncommon,CardType::Skill,NoteStorage::RemovablePermanent),
    supported(CardId::BulletTime,CardColor::Green,CardRarity::Rare,CardType::Skill,NoteStorage::RemovablePermanent),
    supported(CardId::EndlessAgony,CardColor::Green,CardRarity::Uncommon,CardType::Attack,NoteStorage::RemovablePermanent),
    supported(CardId::GlassKnife,CardColor::Green,CardRarity::Rare,CardType::Attack,NoteStorage::RemovablePermanent),
    supported(CardId::Skewer,CardColor::Green,CardRarity::Uncommon,CardType::Attack,NoteStorage::RemovablePermanent),
    supported(CardId::Burst,CardColor::Green,CardRarity::Rare,CardType::Skill,NoteStorage::RemovablePermanent),
    supported(CardId::Doppelganger,CardColor::Green,CardRarity::Rare,CardType::Skill,NoteStorage::RemovablePermanent),
    supported(CardId::Malaise,CardColor::Green,CardRarity::Rare,CardType::Skill,NoteStorage::RemovablePermanent),
    supported(CardId::StormOfSteel,CardColor::Green,CardRarity::Rare,CardType::Skill,NoteStorage::RemovablePermanent),
    supported(CardId::Mayhem,CardColor::Colorless,CardRarity::Rare,CardType::Power,NoteStorage::RemovablePermanent),
    supported(CardId::Panache,CardColor::Colorless,CardRarity::Rare,CardType::Power,NoteStorage::RemovablePermanent),
    supported(CardId::SadisticNature,CardColor::Colorless,CardRarity::Rare,CardType::Power,NoteStorage::RemovablePermanent),
    supported(CardId::TheBomb,CardColor::Colorless,CardRarity::Rare,CardType::Skill,NoteStorage::RemovablePermanent),
    supported(CardId::Distraction,CardColor::Green,CardRarity::Uncommon,CardType::Skill,NoteStorage::RemovablePermanent),
    supported(CardId::Alchemize,CardColor::Green,CardRarity::Rare,CardType::Skill,NoteStorage::RemovablePermanent),
    supported(CardId::Discovery,CardColor::Colorless,CardRarity::Uncommon,CardType::Skill,NoteStorage::RemovablePermanent),
    supported(CardId::Chrysalis,CardColor::Colorless,CardRarity::Rare,CardType::Skill,NoteStorage::RemovablePermanent),
    supported(CardId::Metamorphosis,CardColor::Colorless,CardRarity::Rare,CardType::Skill,NoteStorage::RemovablePermanent),
    supported(CardId::JackOfAllTrades,CardColor::Colorless,CardRarity::Uncommon,CardType::Skill,NoteStorage::RemovablePermanent),
    supported(CardId::Transmutation,CardColor::Colorless,CardRarity::Rare,CardType::Skill,NoteStorage::RemovablePermanent),
    supported(CardId::Magnetism,CardColor::Colorless,CardRarity::Rare,CardType::Power,NoteStorage::RemovablePermanent),
    with_blue_metadata(supported(CardId::Darkness, CardColor::Blue, CardRarity::Uncommon, CardType::Skill, NoteStorage::RemovablePermanent), blue_metadata(CardCost::Energy(1), CardCost::Energy(1), CardTarget::SelfTarget, CardKeywords::NONE, CardKeywords::NONE, &[])),
    with_blue_metadata(supported(CardId::Fusion, CardColor::Blue, CardRarity::Uncommon, CardType::Skill, NoteStorage::RemovablePermanent), blue_metadata(CardCost::Energy(2), CardCost::Energy(1), CardTarget::SelfTarget, CardKeywords::NONE, CardKeywords::NONE, &[])),
    with_blue_metadata(supported(CardId::Capacitor, CardColor::Blue, CardRarity::Uncommon, CardType::Power, NoteStorage::RemovablePermanent), blue_metadata(CardCost::Energy(1), CardCost::Energy(1), CardTarget::SelfTarget, CardKeywords::NONE, CardKeywords::NONE, &[])),
    with_blue_metadata(supported(CardId::Consume, CardColor::Blue, CardRarity::Uncommon, CardType::Skill, NoteStorage::RemovablePermanent), blue_metadata(CardCost::Energy(2), CardCost::Energy(2), CardTarget::SelfTarget, CardKeywords::NONE, CardKeywords::NONE, &[])),
    with_blue_metadata(supported(CardId::Defragment, CardColor::Blue, CardRarity::Uncommon, CardType::Power, NoteStorage::RemovablePermanent), blue_metadata(CardCost::Energy(1), CardCost::Energy(1), CardTarget::SelfTarget, CardKeywords::NONE, CardKeywords::NONE, &[])),
    with_blue_metadata(supported(CardId::BiasedCognition, CardColor::Blue, CardRarity::Rare, CardType::Power, NoteStorage::RemovablePermanent), blue_metadata(CardCost::Energy(1), CardCost::Energy(1), CardTarget::SelfTarget, CardKeywords::NONE, CardKeywords::NONE, &[])),
    with_blue_metadata(supported(CardId::Loop, CardColor::Blue, CardRarity::Uncommon, CardType::Power, NoteStorage::RemovablePermanent), blue_metadata(CardCost::Energy(1), CardCost::Energy(1), CardTarget::SelfTarget, CardKeywords::NONE, CardKeywords::NONE, &[])),
    with_blue_metadata(supported(CardId::Electrodynamics, CardColor::Blue, CardRarity::Rare, CardType::Power, NoteStorage::RemovablePermanent), blue_metadata(CardCost::Energy(2), CardCost::Energy(2), CardTarget::SelfTarget, CardKeywords::NONE, CardKeywords::NONE, &[])),
    with_blue_metadata(supported(CardId::Bullseye, CardColor::Blue, CardRarity::Uncommon, CardType::Attack, NoteStorage::RemovablePermanent), blue_metadata(CardCost::Energy(1), CardCost::Energy(1), CardTarget::Enemy, CardKeywords::NONE, CardKeywords::NONE, &[])),
    with_blue_metadata(supported(CardId::Recursion, CardColor::Blue, CardRarity::Common, CardType::Skill, NoteStorage::RemovablePermanent), blue_metadata(CardCost::Energy(1), CardCost::Energy(0), CardTarget::SelfTarget, CardKeywords::NONE, CardKeywords::NONE, &[])),
    with_blue_metadata(supported(CardId::Fission, CardColor::Blue, CardRarity::Rare, CardType::Skill, NoteStorage::RemovablePermanent), blue_metadata(CardCost::Energy(0), CardCost::Energy(0), CardTarget::None, CardKeywords::EXHAUST, CardKeywords::EXHAUST, &[])),
    with_blue_metadata(supported(CardId::StaticDischarge, CardColor::Blue, CardRarity::Uncommon, CardType::Power, NoteStorage::RemovablePermanent), blue_metadata(CardCost::Energy(1), CardCost::Energy(1), CardTarget::SelfTarget, CardKeywords::NONE, CardKeywords::NONE, &[])),
    with_blue_metadata(supported(CardId::Storm, CardColor::Blue, CardRarity::Uncommon, CardType::Power, NoteStorage::RemovablePermanent), blue_metadata(CardCost::Energy(1), CardCost::Energy(1), CardTarget::SelfTarget, CardKeywords::NONE, CardKeywords::INNATE, &[])),
    with_blue_metadata(supported(CardId::Heatsinks, CardColor::Blue, CardRarity::Uncommon, CardType::Power, NoteStorage::RemovablePermanent), blue_metadata(CardCost::Energy(1), CardCost::Energy(1), CardTarget::SelfTarget, CardKeywords::NONE, CardKeywords::NONE, &[])),
    with_blue_metadata(supported(CardId::Amplify, CardColor::Blue, CardRarity::Rare, CardType::Skill, NoteStorage::RemovablePermanent), blue_metadata(CardCost::Energy(1), CardCost::Energy(1), CardTarget::SelfTarget, CardKeywords::NONE, CardKeywords::NONE, &[])),
    with_blue_metadata(supported(CardId::EchoForm, CardColor::Blue, CardRarity::Rare, CardType::Power, NoteStorage::RemovablePermanent), blue_metadata(CardCost::Energy(3), CardCost::Energy(3), CardTarget::SelfTarget, CardKeywords::ETHEREAL, CardKeywords::NONE, &[])),
    with_blue_metadata(supported(CardId::GeneticAlgorithm, CardColor::Blue, CardRarity::Uncommon, CardType::Skill, NoteStorage::RemovablePermanent), blue_metadata(CardCost::Energy(1), CardCost::Energy(1), CardTarget::SelfTarget, CardKeywords::EXHAUST, CardKeywords::EXHAUST, &[])),
    with_blue_metadata(supported(CardId::Claw, CardColor::Blue, CardRarity::Common, CardType::Attack, NoteStorage::RemovablePermanent), blue_metadata(CardCost::Energy(0), CardCost::Energy(0), CardTarget::Enemy, CardKeywords::NONE, CardKeywords::NONE, &[])),
    with_blue_metadata(supported(CardId::SteamBarrier, CardColor::Blue, CardRarity::Common, CardType::Skill, NoteStorage::RemovablePermanent), blue_metadata(CardCost::Energy(0), CardCost::Energy(0), CardTarget::SelfTarget, CardKeywords::NONE, CardKeywords::NONE, &[])),
    with_blue_metadata(supported(CardId::Streamline, CardColor::Blue, CardRarity::Common, CardType::Attack, NoteStorage::RemovablePermanent), blue_metadata(CardCost::Energy(2), CardCost::Energy(2), CardTarget::Enemy, CardKeywords::NONE, CardKeywords::NONE, &[])),
    with_blue_metadata(supported(CardId::ChargeBattery, CardColor::Blue, CardRarity::Common, CardType::Skill, NoteStorage::RemovablePermanent), blue_metadata(CardCost::Energy(1), CardCost::Energy(1), CardTarget::SelfTarget, CardKeywords::NONE, CardKeywords::NONE, &[])),
    with_blue_metadata(supported(CardId::Hologram, CardColor::Blue, CardRarity::Common, CardType::Skill, NoteStorage::RemovablePermanent), blue_metadata(CardCost::Energy(1), CardCost::Energy(1), CardTarget::SelfTarget, CardKeywords::EXHAUST, CardKeywords::NONE, &[])),
    with_blue_metadata(supported(CardId::Rebound, CardColor::Blue, CardRarity::Common, CardType::Attack, NoteStorage::RemovablePermanent), blue_metadata(CardCost::Energy(1), CardCost::Energy(1), CardTarget::Enemy, CardKeywords::NONE, CardKeywords::NONE, &[])),
    with_blue_metadata(supported(CardId::Stack, CardColor::Blue, CardRarity::Common, CardType::Skill, NoteStorage::RemovablePermanent), blue_metadata(CardCost::Energy(1), CardCost::Energy(1), CardTarget::SelfTarget, CardKeywords::NONE, CardKeywords::NONE, &[])),
    with_blue_metadata(supported(CardId::Turbo, CardColor::Blue, CardRarity::Common, CardType::Skill, NoteStorage::RemovablePermanent), blue_metadata(CardCost::Energy(0), CardCost::Energy(0), CardTarget::SelfTarget, CardKeywords::NONE, CardKeywords::NONE, &[])),
    with_blue_metadata(supported(CardId::Aggregate, CardColor::Blue, CardRarity::Uncommon, CardType::Skill, NoteStorage::RemovablePermanent), blue_metadata(CardCost::Energy(1), CardCost::Energy(1), CardTarget::SelfTarget, CardKeywords::NONE, CardKeywords::NONE, &[])),
    with_blue_metadata(supported(CardId::AutoShields, CardColor::Blue, CardRarity::Uncommon, CardType::Skill, NoteStorage::RemovablePermanent), blue_metadata(CardCost::Energy(1), CardCost::Energy(1), CardTarget::SelfTarget, CardKeywords::NONE, CardKeywords::NONE, &[])),
    with_blue_metadata(supported(CardId::Blizzard, CardColor::Blue, CardRarity::Uncommon, CardType::Attack, NoteStorage::RemovablePermanent), blue_metadata(CardCost::Energy(1), CardCost::Energy(1), CardTarget::AllEnemies, CardKeywords::NONE, CardKeywords::NONE, &[])),
    with_blue_metadata(supported(CardId::BootSequence, CardColor::Blue, CardRarity::Uncommon, CardType::Skill, NoteStorage::RemovablePermanent), blue_metadata(CardCost::Energy(0), CardCost::Energy(0), CardTarget::SelfTarget, CardKeywords::EXHAUST_INNATE, CardKeywords::EXHAUST_INNATE, &[])),
    with_blue_metadata(supported(CardId::Chaos, CardColor::Blue, CardRarity::Uncommon, CardType::Skill, NoteStorage::RemovablePermanent), blue_metadata(CardCost::Energy(1), CardCost::Energy(1), CardTarget::SelfTarget, CardKeywords::NONE, CardKeywords::NONE, &[])),
    with_blue_metadata(supported(CardId::Chill, CardColor::Blue, CardRarity::Uncommon, CardType::Skill, NoteStorage::RemovablePermanent), blue_metadata(CardCost::Energy(0), CardCost::Energy(0), CardTarget::SelfTarget, CardKeywords::EXHAUST, CardKeywords::EXHAUST_INNATE, &[])),
    with_blue_metadata(supported(CardId::DoomAndGloom, CardColor::Blue, CardRarity::Uncommon, CardType::Attack, NoteStorage::RemovablePermanent), blue_metadata(CardCost::Energy(2), CardCost::Energy(2), CardTarget::AllEnemies, CardKeywords::NONE, CardKeywords::NONE, &[])),
    with_blue_metadata(supported(CardId::DoubleEnergy, CardColor::Blue, CardRarity::Uncommon, CardType::Skill, NoteStorage::RemovablePermanent), blue_metadata(CardCost::Energy(1), CardCost::Energy(0), CardTarget::SelfTarget, CardKeywords::EXHAUST, CardKeywords::EXHAUST, &[])),
    with_blue_metadata(supported(CardId::Equilibrium, CardColor::Blue, CardRarity::Uncommon, CardType::Skill, NoteStorage::RemovablePermanent), blue_metadata(CardCost::Energy(2), CardCost::Energy(2), CardTarget::SelfTarget, CardKeywords::NONE, CardKeywords::NONE, &[])),
    with_blue_metadata(supported(CardId::ForceField, CardColor::Blue, CardRarity::Uncommon, CardType::Skill, NoteStorage::RemovablePermanent), blue_metadata(CardCost::Energy(4), CardCost::Energy(4), CardTarget::SelfTarget, CardKeywords::NONE, CardKeywords::NONE, &[])),
    with_blue_metadata(supported(CardId::FTL, CardColor::Blue, CardRarity::Uncommon, CardType::Attack, NoteStorage::RemovablePermanent), blue_metadata(CardCost::Energy(0), CardCost::Energy(0), CardTarget::Enemy, CardKeywords::NONE, CardKeywords::NONE, &[])),
    with_blue_metadata(supported(CardId::Glacier, CardColor::Blue, CardRarity::Uncommon, CardType::Skill, NoteStorage::RemovablePermanent), blue_metadata(CardCost::Energy(2), CardCost::Energy(2), CardTarget::SelfTarget, CardKeywords::NONE, CardKeywords::NONE, &[])),
    with_blue_metadata(supported(CardId::HelloWorld, CardColor::Blue, CardRarity::Uncommon, CardType::Power, NoteStorage::RemovablePermanent), blue_metadata(CardCost::Energy(1), CardCost::Energy(1), CardTarget::SelfTarget, CardKeywords::NONE, CardKeywords::INNATE, &[])),
    with_blue_metadata(supported(CardId::Melter, CardColor::Blue, CardRarity::Uncommon, CardType::Attack, NoteStorage::RemovablePermanent), blue_metadata(CardCost::Energy(1), CardCost::Energy(1), CardTarget::Enemy, CardKeywords::NONE, CardKeywords::NONE, &[])),
    with_blue_metadata(supported(CardId::Overclock, CardColor::Blue, CardRarity::Uncommon, CardType::Skill, NoteStorage::RemovablePermanent), blue_metadata(CardCost::Energy(0), CardCost::Energy(0), CardTarget::SelfTarget, CardKeywords::NONE, CardKeywords::NONE, &[])),
    with_blue_metadata(supported(CardId::Recycle, CardColor::Blue, CardRarity::Uncommon, CardType::Skill, NoteStorage::RemovablePermanent), blue_metadata(CardCost::Energy(1), CardCost::Energy(0), CardTarget::SelfTarget, CardKeywords::NONE, CardKeywords::NONE, &[])),
    with_blue_metadata(supported(CardId::ReinforcedBody, CardColor::Blue, CardRarity::Uncommon, CardType::Skill, NoteStorage::RemovablePermanent), blue_metadata(CardCost::X, CardCost::X, CardTarget::SelfTarget, CardKeywords::NONE, CardKeywords::NONE, &[])),
    with_blue_metadata(supported(CardId::Reprogram, CardColor::Blue, CardRarity::Uncommon, CardType::Skill, NoteStorage::RemovablePermanent), blue_metadata(CardCost::Energy(1), CardCost::Energy(1), CardTarget::None, CardKeywords::NONE, CardKeywords::NONE, &[])),
    with_blue_metadata(supported(CardId::RipAndTear, CardColor::Blue, CardRarity::Uncommon, CardType::Attack, NoteStorage::RemovablePermanent), blue_metadata(CardCost::Energy(1), CardCost::Energy(1), CardTarget::AllEnemies, CardKeywords::NONE, CardKeywords::NONE, &[])),
    with_blue_metadata(supported(CardId::Scrape, CardColor::Blue, CardRarity::Uncommon, CardType::Attack, NoteStorage::RemovablePermanent), blue_metadata(CardCost::Energy(1), CardCost::Energy(1), CardTarget::Enemy, CardKeywords::NONE, CardKeywords::NONE, &[])),
    with_blue_metadata(supported(CardId::SelfRepair, CardColor::Blue, CardRarity::Uncommon, CardType::Power, NoteStorage::RemovablePermanent), blue_metadata(CardCost::Energy(1), CardCost::Energy(1), CardTarget::SelfTarget, CardKeywords::NONE, CardKeywords::NONE, &[CardTag::Healing])),
    with_blue_metadata(supported(CardId::Skim, CardColor::Blue, CardRarity::Uncommon, CardType::Skill, NoteStorage::RemovablePermanent), blue_metadata(CardCost::Energy(1), CardCost::Energy(1), CardTarget::None, CardKeywords::NONE, CardKeywords::NONE, &[])),
    with_blue_metadata(supported(CardId::Sunder, CardColor::Blue, CardRarity::Uncommon, CardType::Attack, NoteStorage::RemovablePermanent), blue_metadata(CardCost::Energy(3), CardCost::Energy(3), CardTarget::Enemy, CardKeywords::NONE, CardKeywords::NONE, &[])),
    with_blue_metadata(supported(CardId::Tempest, CardColor::Blue, CardRarity::Uncommon, CardType::Skill, NoteStorage::RemovablePermanent), blue_metadata(CardCost::X, CardCost::X, CardTarget::SelfTarget, CardKeywords::EXHAUST, CardKeywords::EXHAUST, &[])),
    with_blue_metadata(supported(CardId::WhiteNoise, CardColor::Blue, CardRarity::Uncommon, CardType::Skill, NoteStorage::RemovablePermanent), blue_metadata(CardCost::Energy(1), CardCost::Energy(0), CardTarget::None, CardKeywords::EXHAUST, CardKeywords::EXHAUST, &[])),
    with_blue_metadata(supported(CardId::AllForOne, CardColor::Blue, CardRarity::Rare, CardType::Attack, NoteStorage::RemovablePermanent), blue_metadata(CardCost::Energy(2), CardCost::Energy(2), CardTarget::Enemy, CardKeywords::NONE, CardKeywords::NONE, &[])),
    with_blue_metadata(supported(CardId::Buffer, CardColor::Blue, CardRarity::Rare, CardType::Power, NoteStorage::RemovablePermanent), blue_metadata(CardCost::Energy(2), CardCost::Energy(2), CardTarget::SelfTarget, CardKeywords::NONE, CardKeywords::NONE, &[])),
    with_blue_metadata(supported(CardId::CoreSurge, CardColor::Blue, CardRarity::Rare, CardType::Attack, NoteStorage::RemovablePermanent), blue_metadata(CardCost::Energy(1), CardCost::Energy(1), CardTarget::Enemy, CardKeywords::EXHAUST, CardKeywords::EXHAUST, &[])),
    with_blue_metadata(supported(CardId::CreativeAI, CardColor::Blue, CardRarity::Rare, CardType::Power, NoteStorage::RemovablePermanent), blue_metadata(CardCost::Energy(3), CardCost::Energy(2), CardTarget::SelfTarget, CardKeywords::NONE, CardKeywords::NONE, &[])),
    with_blue_metadata(supported(CardId::Hyperbeam, CardColor::Blue, CardRarity::Rare, CardType::Attack, NoteStorage::RemovablePermanent), blue_metadata(CardCost::Energy(2), CardCost::Energy(2), CardTarget::AllEnemies, CardKeywords::NONE, CardKeywords::NONE, &[])),
    with_blue_metadata(supported(CardId::MachineLearning, CardColor::Blue, CardRarity::Rare, CardType::Power, NoteStorage::RemovablePermanent), blue_metadata(CardCost::Energy(1), CardCost::Energy(1), CardTarget::SelfTarget, CardKeywords::NONE, CardKeywords::INNATE, &[])),
    with_blue_metadata(supported(CardId::MeteorStrike, CardColor::Blue, CardRarity::Rare, CardType::Attack, NoteStorage::RemovablePermanent), blue_metadata(CardCost::Energy(5), CardCost::Energy(5), CardTarget::Enemy, CardKeywords::NONE, CardKeywords::NONE, &[CardTag::Strike])),
    with_blue_metadata(supported(CardId::MultiCast, CardColor::Blue, CardRarity::Rare, CardType::Skill, NoteStorage::RemovablePermanent), blue_metadata(CardCost::X, CardCost::X, CardTarget::None, CardKeywords::NONE, CardKeywords::NONE, &[])),
    with_blue_metadata(supported(CardId::Rainbow, CardColor::Blue, CardRarity::Rare, CardType::Skill, NoteStorage::RemovablePermanent), blue_metadata(CardCost::Energy(2), CardCost::Energy(2), CardTarget::SelfTarget, CardKeywords::EXHAUST, CardKeywords::NONE, &[])),
    with_blue_metadata(supported(CardId::Reboot, CardColor::Blue, CardRarity::Rare, CardType::Skill, NoteStorage::RemovablePermanent), blue_metadata(CardCost::Energy(0), CardCost::Energy(0), CardTarget::SelfTarget, CardKeywords::EXHAUST, CardKeywords::EXHAUST, &[])),
    with_blue_metadata(supported(CardId::Seek, CardColor::Blue, CardRarity::Rare, CardType::Skill, NoteStorage::RemovablePermanent), blue_metadata(CardCost::Energy(0), CardCost::Energy(0), CardTarget::None, CardKeywords::EXHAUST, CardKeywords::EXHAUST, &[])),
    with_blue_metadata(supported(CardId::ThunderStrike, CardColor::Blue, CardRarity::Rare, CardType::Attack, NoteStorage::RemovablePermanent), blue_metadata(CardCost::Energy(3), CardCost::Energy(3), CardTarget::AllEnemies, CardKeywords::NONE, CardKeywords::NONE, &[CardTag::Strike])),
];

const fn card(id: CardId, color: CardColor, rarity: CardRarity, card_type: CardType, note_storage: NoteStorage) -> CardDefinition {
    CardDefinition { id, color, rarity, card_type, note_storage, definition_only: true, blue_metadata: None }
}

const fn supported(id: CardId, color: CardColor, rarity: CardRarity, card_type: CardType, note_storage: NoteStorage) -> CardDefinition {
    CardDefinition { id, color, rarity, card_type, note_storage, definition_only: false, blue_metadata: None }
}

const fn blue_metadata(
    base_cost: CardCost,
    upgraded_cost: CardCost,
    target: CardTarget,
    base_keywords: CardKeywords,
    upgraded_keywords: CardKeywords,
    tags: &'static [CardTag],
) -> BlueCardMetadata {
    BlueCardMetadata {
        base: CardFaceMetadata { cost: base_cost, target, keywords: base_keywords },
        upgraded: CardFaceMetadata { cost: upgraded_cost, target, keywords: upgraded_keywords },
        tags,
    }
}

const fn with_blue_metadata(
    mut definition: CardDefinition,
    metadata: BlueCardMetadata,
) -> CardDefinition {
    definition.blue_metadata = Some(metadata);
    definition
}

const IC: &[CardId] = &[CardId::Strike, CardId::Strike, CardId::Strike, CardId::Strike, CardId::Strike,
    CardId::Defend, CardId::Defend, CardId::Defend, CardId::Defend, CardId::Bash];
const SILENT: &[CardId] = &[CardId::StrikeGreen, CardId::StrikeGreen, CardId::StrikeGreen, CardId::StrikeGreen, CardId::StrikeGreen,
    CardId::DefendGreen, CardId::DefendGreen, CardId::DefendGreen, CardId::DefendGreen, CardId::DefendGreen, CardId::Survivor, CardId::Neutralize];
const DEFECT: &[CardId] = &[CardId::StrikeBlue, CardId::StrikeBlue, CardId::StrikeBlue, CardId::StrikeBlue,
    CardId::DefendBlue, CardId::DefendBlue, CardId::DefendBlue, CardId::DefendBlue, CardId::Zap, CardId::Dualcast];
const WATCHER: &[CardId] = &[CardId::StrikePurple, CardId::StrikePurple, CardId::StrikePurple, CardId::StrikePurple,
    CardId::DefendPurple, CardId::DefendPurple, CardId::DefendPurple, CardId::DefendPurple, CardId::Eruption, CardId::Vigilance];
pub(crate) static STARTERS: &[(Character, &[CardId])] = &[
    (Character::Ironclad, IC), (Character::Silent, SILENT), (Character::Defect, DEFECT), (Character::Watcher, WATCHER),
];

#[cfg(test)]
mod tests {
    use super::{
        CardColor, CardCost, CardKeywords, CardRarity, CardTag, CardTarget, CardType,
    };
    use crate::content::{defect::M20_DEFECT_CARDS, CardId, Catalog};

    #[test]
    fn m20_blue_catalog_has_all_source_roster_metadata() {
        let catalog = Catalog::standard();
        assert_eq!(M20_DEFECT_CARDS.len(), 75);
        let mut rarity_counts = [0usize; 4];
        let mut type_counts = [0usize; 3];

        for id in M20_DEFECT_CARDS {
            let definition = catalog.card(*id).expect("M20 blue card catalog row");
            assert_eq!(definition.color, CardColor::Blue);
            let metadata = definition.blue_metadata.expect("base/upgraded blue metadata");
            assert_eq!(metadata.base.target, metadata.upgraded.target);

            match definition.rarity {
                CardRarity::Basic => rarity_counts[0] += 1,
                CardRarity::Common => rarity_counts[1] += 1,
                CardRarity::Uncommon => rarity_counts[2] += 1,
                CardRarity::Rare => rarity_counts[3] += 1,
                _ => panic!("unexpected M20 blue rarity for {id:?}"),
            }
            match definition.card_type {
                CardType::Attack => type_counts[0] += 1,
                CardType::Skill => type_counts[1] += 1,
                CardType::Power => type_counts[2] += 1,
                _ => panic!("unexpected M20 blue type for {id:?}"),
            }
        }
        assert_eq!(rarity_counts, [4, 18, 36, 17]);
        assert_eq!(type_counts, [24, 37, 14]);

        let zap = catalog.card(CardId::Zap).unwrap().blue_metadata.unwrap();
        assert_eq!(zap.base.cost, CardCost::Energy(1));
        assert_eq!(zap.upgraded.cost, CardCost::Energy(0));
        let boot = catalog.card(CardId::BootSequence).unwrap().blue_metadata.unwrap();
        assert_eq!(boot.base.keywords, CardKeywords::EXHAUST_INNATE);
        assert_eq!(boot.upgraded.keywords, CardKeywords::EXHAUST_INNATE);
        let chill = catalog.card(CardId::Chill).unwrap().blue_metadata.unwrap();
        assert_eq!(chill.base.keywords, CardKeywords::EXHAUST);
        assert_eq!(chill.upgraded.keywords, CardKeywords::EXHAUST_INNATE);
        let echo = catalog.card(CardId::EchoForm).unwrap().blue_metadata.unwrap();
        assert_eq!(echo.base.keywords, CardKeywords::ETHEREAL);
        assert_eq!(echo.upgraded.keywords, CardKeywords::NONE);
        let self_repair = catalog.card(CardId::SelfRepair).unwrap().blue_metadata.unwrap();
        assert_eq!(self_repair.tags, &[CardTag::Healing]);
        let reprogram = catalog.card(CardId::Reprogram).unwrap().blue_metadata.unwrap();
        assert_eq!(reprogram.base.target, CardTarget::None);
        assert_eq!(reprogram.upgraded.target, CardTarget::None);
        let seek = catalog.card(CardId::Seek).unwrap().blue_metadata.unwrap();
        assert_eq!(seek.base.keywords, CardKeywords::EXHAUST);
        assert_eq!(seek.upgraded.keywords, CardKeywords::EXHAUST);
    }

    #[test]
    fn blue_metadata_preserves_legacy_catalog_debug_shape() {
        let zap = Catalog::standard().card(CardId::Zap).unwrap();
        assert_eq!(
            format!("{zap:?}"),
            "CardDefinition { id: Zap, color: Blue, rarity: Basic, card_type: Skill, note_storage: RemovablePermanent, definition_only: false }",
        );
    }
}
