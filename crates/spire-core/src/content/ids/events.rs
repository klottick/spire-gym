#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EventId { Neow, UpgradeShrine, Purifier, AncientWriting, Joust, Falling, TombOfLordRedMask,
    // M28 standard events (design §5.1), appended after the accepted seven; Rust identities, not
    // source IDs (E1). Act 1 ordinary.
    BigFish, DeadAdventurer, GoldenIdol, Mushrooms, LivingWall, ScrapOoze, ShiningLight, Cleric, Ssssserpent, WingStatue, WorldOfGoop,
    // Act 2 ordinary.
    Addict, Beggar, Colosseum, CursedTome, DrugDealer, ForgottenAltar, Ghosts, MaskedBandits, Nest, Library, Mausoleum, Vampires,
    // Act 3 ordinary.
    MindBloom, MoaiHead, MysteriousSphere, SensoryStone, WindingHalls,
    // Shrines.
    GoldenShrine, MatchAndKeep, Transmogrifier, WheelOfChange,
    // One-time.
    AccursedBlacksmith, BonfireSpirits, FaceTrader, FountainOfCleansing, Lab, NoteForYourself, WeMeetAgain, WomanInBlue, Designer, Duplicator, KnowingSkull, Nloth, SecretPortal,
    Unknown(&'static str) }
impl EventId { pub const fn as_str(self) -> &'static str { match self { Self::Falling=>"falling",Self::TombOfLordRedMask=>"tomb_of_lord_red_mask",Self::Neow => "neow", Self::UpgradeShrine=>"upgrade_shrine",Self::Purifier=>"purifier",Self::AncientWriting=>"ancient_writing",Self::Joust=>"joust",
    Self::BigFish=>"big_fish",Self::DeadAdventurer=>"dead_adventurer",Self::GoldenIdol=>"golden_idol",Self::Mushrooms=>"mushrooms",Self::LivingWall=>"living_wall",Self::ScrapOoze=>"scrap_ooze",Self::ShiningLight=>"shining_light",Self::Cleric=>"cleric",Self::Ssssserpent=>"ssssserpent",Self::WingStatue=>"wing_statue",Self::WorldOfGoop=>"world_of_goop",
    Self::Addict=>"addict",Self::Beggar=>"beggar",Self::Colosseum=>"colosseum",Self::CursedTome=>"cursed_tome",Self::DrugDealer=>"drug_dealer",Self::ForgottenAltar=>"forgotten_altar",Self::Ghosts=>"ghosts",Self::MaskedBandits=>"masked_bandits",Self::Nest=>"nest",Self::Library=>"library",Self::Mausoleum=>"mausoleum",Self::Vampires=>"vampires",
    Self::MindBloom=>"mind_bloom",Self::MoaiHead=>"moai_head",Self::MysteriousSphere=>"mysterious_sphere",Self::SensoryStone=>"sensory_stone",Self::WindingHalls=>"winding_halls",
    Self::GoldenShrine=>"golden_shrine",Self::MatchAndKeep=>"match_and_keep",Self::Transmogrifier=>"transmogrifier",Self::WheelOfChange=>"wheel_of_change",
    Self::AccursedBlacksmith=>"accursed_blacksmith",Self::BonfireSpirits=>"bonfire_spirits",Self::FaceTrader=>"face_trader",Self::FountainOfCleansing=>"fountain_of_cleansing",Self::Lab=>"lab",Self::NoteForYourself=>"note_for_yourself",Self::WeMeetAgain=>"we_meet_again",Self::WomanInBlue=>"woman_in_blue",Self::Designer=>"designer",Self::Duplicator=>"duplicator",Self::KnowingSkull=>"knowing_skull",Self::Nloth=>"nloth",Self::SecretPortal=>"secret_portal",
    Self::Unknown(s) => s } } }
