#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RelicId { Vajra, OddlySmoothStone, BagOfPreparation, BagOfMarbles, BronzeScales,
    BurningBlood, RingOfTheSnake, CrackedCore, PureWater, BloodVial, Anchor, Lantern,
    Strawberry, Pear, Mango, SacredBark, MarkOfBloom, MagicFlower, ToyOrnithopter, NeowsLament, BlackBlood, EmptyCage, SlaversCollar, RedMask,
    RedSkull, PaperPhrog, ChampionsBelt, CharonsAshes, SelfFormingClay, Orichalcum, Brimstone, MarkOfPain, RunicCube, Unknown(&'static str), SneckoSkull,
    NinjaScroll, PaperKrane, TheSpecimen, Tingsha, ToughBandages, TwistedFunnel, HoveringKite, RingOfTheSerpent, WristBlade,
    Sozu, PotionBelt }
impl RelicId { pub const fn as_str(self) -> &'static str { match self {
    Self::Vajra=>"vajra",Self::OddlySmoothStone=>"oddly_smooth_stone",Self::BagOfPreparation=>"bag_of_preparation",Self::BagOfMarbles=>"bag_of_marbles",Self::BronzeScales=>"bronze_scales",
    Self::RedMask=>"red_mask",Self::RedSkull=>"red_skull",Self::PaperPhrog=>"paper_phrog",Self::ChampionsBelt=>"champions_belt",Self::CharonsAshes=>"charons_ashes",Self::SelfFormingClay=>"self_forming_clay",Self::Orichalcum=>"orichalcum",Self::Brimstone=>"brimstone",Self::MarkOfPain=>"mark_of_pain",Self::RunicCube=>"runic_cube",
    Self::BlackBlood=>"black_blood",Self::EmptyCage=>"empty_cage",Self::SlaversCollar=>"slavers_collar",
    Self::BurningBlood => "burning_blood", Self::RingOfTheSnake => "ring_of_the_snake", Self::CrackedCore => "cracked_core", Self::PureWater => "pure_water", Self::SneckoSkull => "snecko_skull",
    Self::BloodVial => "blood_vial", Self::Anchor => "anchor", Self::Lantern => "lantern",
    Self::Strawberry => "strawberry", Self::Pear => "pear", Self::Mango => "mango", Self::SacredBark => "sacred_bark", Self::MarkOfBloom => "mark_of_bloom",
    Self::MagicFlower => "magic_flower", Self::ToyOrnithopter => "toy_ornithopter", Self::NeowsLament=>"neows_lament", Self::Unknown(s) => s,
    Self::NinjaScroll=>"ninja_scroll", Self::PaperKrane=>"paper_krane", Self::TheSpecimen=>"the_specimen",
    Self::Tingsha=>"tingsha", Self::ToughBandages=>"tough_bandages", Self::TwistedFunnel=>"twisted_funnel",
    Self::HoveringKite=>"hovering_kite", Self::RingOfTheSerpent=>"ring_of_the_serpent", Self::WristBlade=>"wrist_blade",
    Self::Sozu=>"sozu",Self::PotionBelt=>"potion_belt"
}}}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PotionId { FirePotion, BloodPotion, FruitJuice, BlockPotion, EnergyPotion, StrengthPotion,
    SwiftPotion, WeakPotion, FearPotion, DexterityPotion, AncientPotion, Elixir, HeartOfIron, Unknown(&'static str),
    PoisonPotion, CunningPotion, GhostInAJar,
    SmokeBomb,
    AttackPotion, SkillPotion, PowerPotion, ColorlessPotion,
    FlexPotion, SpeedPotion, RegenPotion, LiquidBronze, EssenceOfSteel, CultistPotion, BlessingOfTheForge, SneckoOil, FairyPotion,
    ExplosivePotion, GamblersBrew, LiquidMemories, DuplicationPotion, DistilledChaos, EntropicBrew }
impl PotionId { pub const fn as_str(self) -> &'static str { match self {
    Self::FirePotion => "fire_potion", Self::BloodPotion => "blood_potion", Self::FruitJuice => "fruit_juice",
    Self::BlockPotion => "block_potion", Self::EnergyPotion => "energy_potion", Self::StrengthPotion => "strength_potion",
    Self::SwiftPotion => "swift_potion", Self::WeakPotion => "weak_potion", Self::FearPotion => "fear_potion",
    Self::DexterityPotion => "dexterity_potion", Self::AncientPotion => "ancient_potion", Self::Elixir => "elixir",
    Self::HeartOfIron => "heart_of_iron", Self::Unknown(s) => s,
    Self::AttackPotion=>"attack_potion",Self::SkillPotion=>"skill_potion",Self::PowerPotion=>"power_potion",Self::ColorlessPotion=>"colorless_potion",
    Self::SmokeBomb=>"smoke_bomb",
    Self::FairyPotion=>"fairy_potion",
    Self::BlessingOfTheForge=>"blessing_of_the_forge", Self::SneckoOil=>"snecko_oil",
    Self::FlexPotion=>"flex_potion", Self::SpeedPotion=>"speed_potion", Self::RegenPotion=>"regen_potion",
    Self::LiquidBronze=>"liquid_bronze", Self::EssenceOfSteel=>"essence_of_steel", Self::CultistPotion=>"cultist_potion",
    Self::PoisonPotion=>"poison_potion", Self::CunningPotion=>"cunning_potion", Self::GhostInAJar=>"ghost_in_a_jar"
    ,Self::ExplosivePotion=>"explosive_potion",Self::GamblersBrew=>"gamblers_brew",Self::LiquidMemories=>"liquid_memories",
    Self::DuplicationPotion=>"duplication_potion",Self::DistilledChaos=>"distilled_chaos"
,Self::EntropicBrew=>"entropic_brew"
} } }
