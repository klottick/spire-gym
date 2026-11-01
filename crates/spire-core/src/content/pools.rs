//! M26 complete producer pools (design §7): the one owner of every card pool.
//!
//! Membership is derived from the catalog and `card_tags`; each family's
//! literal size is checked, and an incomplete family or a definition-only
//! member is rejected before any caller draws. Accepted families keep their
//! recorded order (M10's red reward arrays and Infernal Blade list, M25's
//! Foreign Influence order); every other family is ordered by rarity
//! (Common → Uncommon → Rare), then `CardId::as_str`. Combat pools exclude
//! HEALING cards; Run pools do not.

use super::cards::{is_healing, CardColor, CardDefinition, CardRarity, CardType, NoteStorage, RegistryError};
use super::{CardId, Catalog, Character};

/// Combat generator families (`returnTrulyRandomCardInCombat` and L34).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GenerationPool { Attack, Skill, Power, Any, Colorless, Common }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CardPool {
    /// The character's Common/Uncommon/Rare cards, optionally one rarity and type.
    Ordinary { character: Character, rarity: Option<CardRarity>, card_type: Option<CardType> },
    /// Prismatic Shard support: ordinary cards of all four colours plus ordinary colourless.
    AnyColour { rarity: Option<CardRarity> },
    /// Ordinary colourless, including Bandage Up.
    Colourless { rarity: Option<CardRarity> },
    /// The random curse pool (removable curses only).
    Curse,
    /// The transform support for one input card (design §7.2).
    Transform { character: Character, input: CardId },
    /// A combat generator pool of the player's colour (HEALING excluded).
    Combat { character: Character, pool: GenerationPool },
    /// M25's Foreign Influence support (every ordinary non-HEALING Attack).
    ForeignInfluence,
    /// Every removable permanent card of any colour.
    NoteForYourself,
}

/// M10's accepted red reward arrays (the Ironclad `Ordinary` order); M10's
/// `red_candidates` reads them here.
pub(crate) const RED_COMMON: &[CardId] = &[CardId::Armaments,CardId::BodySlam,CardId::Clash,CardId::SwordBoomerang,CardId::WildStrike,CardId::TrueGrit,CardId::Headbutt,CardId::Warcry,CardId::Havoc,CardId::Anger,CardId::IronWave,CardId::Cleave,CardId::PommelStrike,CardId::ShrugItOff,CardId::TwinStrike,CardId::Clothesline,CardId::HeavyBlade,CardId::PerfectedStrike,CardId::Thunderclap,CardId::Flex];
pub(crate) const RED_UNCOMMON: &[CardId] = &[CardId::SearingBlow,CardId::Carnage,CardId::Hemokinesis,CardId::Pummel,CardId::Uppercut,CardId::Entrench,CardId::GhostlyArmor,CardId::Intimidate,CardId::PowerThrough,CardId::RecklessCharge,CardId::SeeingRed,CardId::Shockwave,CardId::SpotWeakness,CardId::BattleTrance,CardId::DualWield,CardId::Whirlwind,CardId::Inflame,CardId::Evolve,CardId::FireBreathing,CardId::FlameBarrier,CardId::Rage,CardId::Rupture,CardId::DarkEmbrace,CardId::FeelNoPain,CardId::Sentinel,CardId::SecondWind,CardId::SeverSoul,CardId::Metallicize,CardId::Combust,CardId::Bloodletting,CardId::BurningPact,CardId::Disarm,CardId::BloodForBlood,CardId::Rampage,CardId::Dropkick,CardId::InfernalBlade];
pub(crate) const RED_RARE: &[CardId] = &[CardId::Immolate,CardId::Exhume,CardId::DoubleTap,CardId::Juggernaut,CardId::Corruption,CardId::FiendFire,CardId::Barricade,CardId::Berserk,CardId::Brutality,CardId::DemonForm,CardId::Bludgeon,CardId::Feed,CardId::Reaper,CardId::Impervious,CardId::Offering,CardId::LimitBreak];
/// M10's accepted Infernal Blade list: the Ironclad combat Attack order.
pub(crate) const IRONCLAD_COMBAT_ATTACKS: &[CardId] = &[CardId::SearingBlow,CardId::BodySlam,CardId::Carnage,CardId::Clash,CardId::Hemokinesis,CardId::Pummel,CardId::Uppercut,CardId::Immolate,CardId::SwordBoomerang,CardId::WildStrike,CardId::RecklessCharge,CardId::Headbutt,CardId::Whirlwind,CardId::SeverSoul,CardId::FiendFire,CardId::Anger,CardId::IronWave,CardId::Cleave,CardId::PommelStrike,CardId::TwinStrike,CardId::Clothesline,CardId::HeavyBlade,CardId::PerfectedStrike,CardId::Thunderclap,CardId::Bludgeon,CardId::BloodForBlood,CardId::Rampage,CardId::Dropkick];

const ORDINARY: [CardRarity; 3] = [CardRarity::Common, CardRarity::Uncommon, CardRarity::Rare];
const COLOURS: [CardColor; 5] = [CardColor::Red, CardColor::Green, CardColor::Blue, CardColor::Purple, CardColor::Colorless];

pub fn cards(pool: CardPool) -> Result<Vec<CardId>, RegistryError> {
    let rows = members(pool)?;
    if rows.iter().any(|row| row.definition_only) || rows.len() != expected(pool)? {
        return Err(RegistryError::IncompletePool);
    }
    Ok(rows.into_iter().map(|row| row.id).collect())
}

fn color(character: Character) -> CardColor {
    match character {
        Character::Ironclad => CardColor::Red,
        Character::Silent => CardColor::Green,
        Character::Defect => CardColor::Blue,
        Character::Watcher => CardColor::Purple,
    }
}

fn rows(filter: impl Fn(&CardDefinition) -> bool) -> Vec<&'static CardDefinition> {
    Catalog::standard().cards().iter().filter(|row| filter(row)).collect()
}

fn rank(rarity: CardRarity) -> u8 {
    match rarity {
        CardRarity::Basic => 0, CardRarity::Common => 1, CardRarity::Uncommon => 2, CardRarity::Rare => 3,
        CardRarity::Special => 4, CardRarity::Status => 5, CardRarity::Curse => 6,
    }
}

fn sorted(mut rows: Vec<&'static CardDefinition>) -> Vec<&'static CardDefinition> {
    rows.sort_by_key(|row| (rank(row.rarity), row.id.as_str()));
    rows
}

/// A recorded order must be a permutation of the derived membership.
fn recorded(derived: Vec<&'static CardDefinition>, order: &[CardId]) -> Result<Vec<&'static CardDefinition>, RegistryError> {
    let permutation = derived.len() == order.len() && order.iter().all(|id| derived.iter().any(|row| row.id == *id));
    if !permutation { return Err(RegistryError::IncompletePool) }
    Ok(order.iter().map(|id| *derived.iter().find(|row| row.id == *id).expect("checked permutation")).collect())
}

fn ordinary_rarities(rarity: Option<CardRarity>) -> Result<Vec<CardRarity>, RegistryError> {
    match rarity {
        None => Ok(ORDINARY.to_vec()),
        Some(rarity) if ORDINARY.contains(&rarity) => Ok(vec![rarity]),
        Some(_) => Err(RegistryError::IncompletePool),
    }
}

fn members(pool: CardPool) -> Result<Vec<&'static CardDefinition>, RegistryError> {
    Ok(match pool {
        CardPool::Ordinary { character, rarity, card_type } => {
            let mut out = Vec::new();
            for rarity in ordinary_rarities(rarity)? {
                let derived = sorted(rows(|row| row.color == color(character) && row.rarity == rarity));
                let family = match (character, rarity) {
                    (Character::Ironclad, CardRarity::Common) => recorded(derived, RED_COMMON)?,
                    (Character::Ironclad, CardRarity::Uncommon) => recorded(derived, RED_UNCOMMON)?,
                    (Character::Ironclad, _) => recorded(derived, RED_RARE)?,
                    _ => derived,
                };
                out.extend(family.into_iter().filter(|row| card_type.is_none_or(|kind| row.card_type == kind)));
            }
            out
        }
        CardPool::AnyColour { rarity } => {
            let rarities = ordinary_rarities(rarity)?;
            sorted(rows(|row| COLOURS.contains(&row.color) && rarities.contains(&row.rarity)))
        }
        CardPool::Colourless { rarity } => {
            let rarities = ordinary_rarities(rarity)?;
            sorted(rows(|row| row.color == CardColor::Colorless && rarities.contains(&row.rarity)))
        }
        CardPool::Curse => sorted(rows(|row| row.color == CardColor::Curse && row.note_storage == NoteStorage::RemovablePermanent)),
        CardPool::Transform { character, input } => {
            let definition = Catalog::standard().card(input).filter(|row| !row.definition_only).ok_or(RegistryError::UnknownDefinition)?;
            let base = match definition.color {
                CardColor::Curse => CardPool::Curse,
                CardColor::Colorless => CardPool::Colourless { rarity: None },
                _ => CardPool::Ordinary { character, rarity: None, card_type: None },
            };
            let mut family = members(base)?;
            family.retain(|row| row.id != input);
            family
        }
        CardPool::Combat { pool: GenerationPool::Colorless, .. } => {
            let mut family = rows(|row| row.color == CardColor::Colorless && ORDINARY.contains(&row.rarity) && !is_healing(row.id));
            family.sort_by_key(|row| row.id.as_str());
            family
        }
        CardPool::Combat { character, pool } => {
            let derived = sorted(rows(|row| row.color == color(character) && ORDINARY.contains(&row.rarity) && !is_healing(row.id)));
            let family = derived.into_iter().filter(|row| match pool {
                GenerationPool::Attack => row.card_type == CardType::Attack,
                GenerationPool::Skill => row.card_type == CardType::Skill,
                GenerationPool::Power => row.card_type == CardType::Power,
                GenerationPool::Common => row.rarity == CardRarity::Common,
                GenerationPool::Any | GenerationPool::Colorless => true,
            }).collect::<Vec<_>>();
            if (character, pool) == (Character::Ironclad, GenerationPool::Attack) { recorded(family, IRONCLAD_COMBAT_ATTACKS)? } else { family }
        }
        CardPool::ForeignInfluence => rows(|row| row.card_type == CardType::Attack && COLOURS.contains(&row.color)
            && ORDINARY.contains(&row.rarity) && !is_healing(row.id)),
        CardPool::NoteForYourself => sorted(rows(|row| !row.definition_only && row.note_storage == NoteStorage::RemovablePermanent)),
    })
}

/// Literal permanent sizes, Common/Uncommon/Rare, as Attack/Skill/Power (plan Task M2).
fn ordinary_size(color: CardColor, rarity: CardRarity, card_type: CardType) -> usize {
    let index = match rarity { CardRarity::Common => 0, CardRarity::Uncommon => 1, _ => 2 };
    let table: [[usize; 3]; 3] = match color {
        CardColor::Red => [[14, 11, 5], [6, 17, 5], [0, 8, 6]],
        CardColor::Green => [[9, 13, 4], [10, 14, 10], [0, 6, 5]],
        CardColor::Blue => [[10, 8, 5], [8, 20, 6], [0, 8, 6]],
        CardColor::Purple => [[10, 12, 3], [9, 15, 10], [0, 8, 4]],
        CardColor::Colorless => [[0, 4, 1], [0, 16, 10], [0, 0, 4]],
        _ => return 0,
    };
    match card_type { CardType::Attack => table[0][index], CardType::Skill => table[1][index], CardType::Power => table[2][index], _ => 0 }
}

fn ordinary_total(colors: &[CardColor], rarities: &[CardRarity], card_type: Option<CardType>) -> usize {
    let types = card_type.map_or(vec![CardType::Attack, CardType::Skill, CardType::Power], |kind| vec![kind]);
    let mut total = 0;
    for color in colors { for rarity in rarities { for kind in &types { total += ordinary_size(*color, *rarity, *kind); } } }
    total
}

fn expected(pool: CardPool) -> Result<usize, RegistryError> {
    Ok(match pool {
        CardPool::Ordinary { character, rarity, card_type } => ordinary_total(&[color(character)], &ordinary_rarities(rarity)?, card_type),
        CardPool::AnyColour { rarity } => ordinary_total(&COLOURS, &ordinary_rarities(rarity)?, None),
        CardPool::Colourless { rarity } => ordinary_total(&[CardColor::Colorless], &ordinary_rarities(rarity)?, None),
        CardPool::Curse => 10,
        CardPool::Transform { character, input } => {
            let definition = Catalog::standard().card(input).ok_or(RegistryError::UnknownDefinition)?;
            let base = match definition.color {
                CardColor::Curse => CardPool::Curse,
                CardColor::Colorless => CardPool::Colourless { rarity: None },
                _ => CardPool::Ordinary { character, rarity: None, card_type: None },
            };
            expected(base)? - usize::from(members(base)?.iter().any(|row| row.id == input))
        }
        CardPool::Combat { pool: GenerationPool::Colorless, .. } => 34,
        CardPool::Combat { character, pool } => {
            // Attack, Skill, Power, Any, Common (design §7.1).
            let sizes = match character {
                Character::Ironclad => [28, 28, 14, 70, 20],
                Character::Silent => [26, 33, 11, 70, 19],
                Character::Defect => [23, 34, 13, 70, 18],
                Character::Watcher => [24, 33, 12, 69, 19],
            };
            match pool {
                GenerationPool::Attack => sizes[0], GenerationPool::Skill => sizes[1], GenerationPool::Power => sizes[2],
                GenerationPool::Any => sizes[3], GenerationPool::Common => sizes[4], GenerationPool::Colorless => 34,
            }
        }
        CardPool::ForeignInfluence => 106,
        CardPool::NoteForYourself => 349,
    })
}
