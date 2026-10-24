#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RelicId { Vajra, OddlySmoothStone, BagOfPreparation, BagOfMarbles, BronzeScales,
    BurningBlood, RingOfTheSnake, CrackedCore, PureWater, BloodVial, Anchor, Lantern,
    Strawberry, Pear, Mango, SacredBark, MarkOfBloom, MagicFlower, ToyOrnithopter, NeowsLament, BlackBlood, EmptyCage, SlaversCollar, RedMask, Unknown(&'static str) }
impl RelicId { pub const fn as_str(self) -> &'static str { match self {
    Self::Vajra=>"vajra",Self::OddlySmoothStone=>"oddly_smooth_stone",Self::BagOfPreparation=>"bag_of_preparation",Self::BagOfMarbles=>"bag_of_marbles",Self::BronzeScales=>"bronze_scales",
    Self::RedMask=>"red_mask",
    Self::BlackBlood=>"black_blood",Self::EmptyCage=>"empty_cage",Self::SlaversCollar=>"slavers_collar",
    Self::BurningBlood => "burning_blood", Self::RingOfTheSnake => "ring_of_the_snake", Self::CrackedCore => "cracked_core", Self::PureWater => "pure_water",
    Self::BloodVial => "blood_vial", Self::Anchor => "anchor", Self::Lantern => "lantern",
    Self::Strawberry => "strawberry", Self::Pear => "pear", Self::Mango => "mango", Self::SacredBark => "sacred_bark", Self::MarkOfBloom => "mark_of_bloom",
    Self::MagicFlower => "magic_flower", Self::ToyOrnithopter => "toy_ornithopter", Self::NeowsLament=>"neows_lament", Self::Unknown(s) => s
}}}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PotionId { FirePotion, BloodPotion, FruitJuice, BlockPotion, EnergyPotion, StrengthPotion,
    SwiftPotion, WeakPotion, FearPotion, DexterityPotion, AncientPotion, Unknown(&'static str) }
impl PotionId { pub const fn as_str(self) -> &'static str { match self {
    Self::FirePotion => "fire_potion", Self::BloodPotion => "blood_potion", Self::FruitJuice => "fruit_juice",
    Self::BlockPotion => "block_potion", Self::EnergyPotion => "energy_potion", Self::StrengthPotion => "strength_potion",
    Self::SwiftPotion => "swift_potion", Self::WeakPotion => "weak_potion", Self::FearPotion => "fear_potion",
    Self::DexterityPotion => "dexterity_potion", Self::AncientPotion => "ancient_potion", Self::Unknown(s) => s
} } }
