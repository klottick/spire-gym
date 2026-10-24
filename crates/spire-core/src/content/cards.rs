use super::{CardId, Character};

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

#[derive(Clone, Copy, Debug)]
pub struct CardDefinition {
    pub id: CardId,
    pub color: CardColor,
    pub rarity: CardRarity,
    pub card_type: CardType,
    pub note_storage: NoteStorage,
    /// Definitions are registered, but executable support belongs to later
    /// mechanics components. No foundation row claims a run handler.
    pub definition_only: bool,
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
    supported(CardId::StrikeBlue, CardColor::Blue, CardRarity::Basic, CardType::Attack, NoteStorage::RemovablePermanent),
    supported(CardId::DefendBlue, CardColor::Blue, CardRarity::Basic, CardType::Skill, NoteStorage::RemovablePermanent),
    supported(CardId::Zap, CardColor::Blue, CardRarity::Basic, CardType::Skill, NoteStorage::RemovablePermanent),
    supported(CardId::Dualcast, CardColor::Blue, CardRarity::Basic, CardType::Skill, NoteStorage::RemovablePermanent),
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
    supported(CardId::Deflect, CardColor::Green, CardRarity::Common, CardType::Skill, NoteStorage::RemovablePermanent),
    supported(CardId::QuickSlash, CardColor::Green, CardRarity::Common, CardType::Attack, NoteStorage::RemovablePermanent),
    supported(CardId::CloakAndDagger, CardColor::Green, CardRarity::Common, CardType::Skill, NoteStorage::RemovablePermanent),
    supported(CardId::Shiv, CardColor::Colorless, CardRarity::Special, CardType::Attack, NoteStorage::TemporaryOnly),
    supported(CardId::Clothesline, CardColor::Red, CardRarity::Common, CardType::Attack, NoteStorage::RemovablePermanent),
    supported(CardId::HeavyBlade, CardColor::Red, CardRarity::Common, CardType::Attack, NoteStorage::RemovablePermanent),
    supported(CardId::PerfectedStrike, CardColor::Red, CardRarity::Common, CardType::Attack, NoteStorage::RemovablePermanent),
    supported(CardId::Thunderclap, CardColor::Red, CardRarity::Common, CardType::Attack, NoteStorage::RemovablePermanent),
    supported(CardId::BallLightning, CardColor::Blue, CardRarity::Common, CardType::Attack, NoteStorage::RemovablePermanent),
    supported(CardId::ColdSnap, CardColor::Blue, CardRarity::Common, CardType::Attack, NoteStorage::RemovablePermanent),
    supported(CardId::Coolheaded, CardColor::Blue, CardRarity::Common, CardType::Skill, NoteStorage::RemovablePermanent),
    supported(CardId::Leap, CardColor::Blue, CardRarity::Common, CardType::Skill, NoteStorage::RemovablePermanent),
    supported(CardId::Consecrate, CardColor::Purple, CardRarity::Common, CardType::Attack, NoteStorage::RemovablePermanent),
    supported(CardId::EmptyBody, CardColor::Purple, CardRarity::Common, CardType::Skill, NoteStorage::RemovablePermanent),
    supported(CardId::EmptyFist, CardColor::Purple, CardRarity::Common, CardType::Attack, NoteStorage::RemovablePermanent),
    supported(CardId::EmptyMind, CardColor::Purple, CardRarity::Uncommon, CardType::Skill, NoteStorage::RemovablePermanent),
    supported(CardId::BeamCell, CardColor::Blue, CardRarity::Common, CardType::Attack, NoteStorage::RemovablePermanent),
    supported(CardId::GoForTheEyes, CardColor::Blue, CardRarity::Common, CardType::Attack, NoteStorage::RemovablePermanent),
    supported(CardId::Barrage, CardColor::Blue, CardRarity::Common, CardType::Attack, NoteStorage::RemovablePermanent),
    supported(CardId::CompileDriver, CardColor::Blue, CardRarity::Common, CardType::Attack, NoteStorage::RemovablePermanent),
    supported(CardId::SweepingBeam, CardColor::Blue, CardRarity::Common, CardType::Attack, NoteStorage::RemovablePermanent),
    card(CardId::DeadlyPoison, CardColor::Green, CardRarity::Common, CardType::Skill, NoteStorage::RemovablePermanent),
    card(CardId::PoisonedStab, CardColor::Green, CardRarity::Common, CardType::Attack, NoteStorage::RemovablePermanent),
    card(CardId::Catalyst, CardColor::Green, CardRarity::Uncommon, CardType::Skill, NoteStorage::RemovablePermanent),
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
    card(CardId::Madness, CardColor::Colorless, CardRarity::Uncommon, CardType::Skill, NoteStorage::RemovablePermanent),
    card(CardId::Regret, CardColor::Curse, CardRarity::Curse, CardType::Curse, NoteStorage::RemovablePermanent),
];

const fn card(id: CardId, color: CardColor, rarity: CardRarity, card_type: CardType, note_storage: NoteStorage) -> CardDefinition {
    CardDefinition { id, color, rarity, card_type, note_storage, definition_only: true }
}

const fn supported(id: CardId, color: CardColor, rarity: CardRarity, card_type: CardType, note_storage: NoteStorage) -> CardDefinition {
    CardDefinition { id, color, rarity, card_type, note_storage, definition_only: false }
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
