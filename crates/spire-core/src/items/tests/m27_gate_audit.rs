//! M27 Task M6 gate audit (design §2 item 6, §§6.3, 7.5, 8.1, 15.2).
//! The item ledger is literal and independently checked against the complete catalog unions.

use crate::content::{complete, Catalog, CardType, Character, PotionId, RelicId};
use crate::combat::{CombatAction, TargetId};
use super::m27_support;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ItemId { Relic(RelicId), Potion(PotionId) }
impl ItemId {
    fn as_str(self) -> &'static str { match self { Self::Relic(id) => id.as_str(), Self::Potion(id) => id.as_str() } }
    fn source_name(self) -> String { match self { Self::Relic(id) => format!("RelicId::{id:?}"), Self::Potion(id) => format!("PotionId::{id:?}") } }
}
#[derive(Clone, Copy, Debug)]
struct LedgerRow { id: &'static str, item: ItemId, kind: &'static str, key: &'static str, file: &'static str, test: &'static str }

/// One row per complete union identity; ids, keys, source files, and tests are literal.
const LEDGER: &[LedgerRow] = &[
    LedgerRow { id: "holy_water", item: ItemId::Relic(RelicId::HolyWater), kind: "relic", key: "E", file: "src/items/tests/m27_existing_relics.rs", test: "m27_pure_water_phase" },
    LedgerRow { id: "chemical_x", item: ItemId::Relic(RelicId::ChemicalX), kind: "relic", key: "E", file: "src/items/tests/m27_existing_relics.rs", test: "m27_existing_relics_equal_accepted" },
    LedgerRow { id: "vajra", item: ItemId::Relic(RelicId::Vajra), kind: "relic", key: "E", file: "src/items/tests/m27_existing_relics.rs", test: "m27_existing_relics_equal_accepted" },
    LedgerRow { id: "oddly_smooth_stone", item: ItemId::Relic(RelicId::OddlySmoothStone), kind: "relic", key: "E", file: "src/items/tests/m27_existing_relics.rs", test: "m27_existing_relics_equal_accepted" },
    LedgerRow { id: "bag_of_preparation", item: ItemId::Relic(RelicId::BagOfPreparation), kind: "relic", key: "E", file: "src/items/tests/m27_existing_relics.rs", test: "m27_existing_relics_equal_accepted" },
    LedgerRow { id: "bag_of_marbles", item: ItemId::Relic(RelicId::BagOfMarbles), kind: "relic", key: "E", file: "src/items/tests/m27_existing_relics.rs", test: "m27_existing_relics_equal_accepted" },
    LedgerRow { id: "bronze_scales", item: ItemId::Relic(RelicId::BronzeScales), kind: "relic", key: "E", file: "src/items/tests/m27_existing_relics.rs", test: "m27_existing_relics_equal_accepted" },
    LedgerRow { id: "red_mask", item: ItemId::Relic(RelicId::RedMask), kind: "relic", key: "E", file: "src/items/tests/m27_existing_relics.rs", test: "m27_existing_relics_equal_accepted" },
    LedgerRow { id: "red_skull", item: ItemId::Relic(RelicId::RedSkull), kind: "relic", key: "E", file: "src/items/tests/m27_existing_relics.rs", test: "m27_existing_relics_equal_accepted" },
    LedgerRow { id: "paper_phrog", item: ItemId::Relic(RelicId::PaperPhrog), kind: "relic", key: "E", file: "src/items/tests/m27_existing_relics.rs", test: "m27_existing_relics_equal_accepted" },
    LedgerRow { id: "champions_belt", item: ItemId::Relic(RelicId::ChampionsBelt), kind: "relic", key: "E", file: "src/items/tests/m27_existing_relics.rs", test: "m27_existing_relics_equal_accepted" },
    LedgerRow { id: "charons_ashes", item: ItemId::Relic(RelicId::CharonsAshes), kind: "relic", key: "E", file: "src/items/tests/m27_existing_relics.rs", test: "m27_existing_relics_equal_accepted" },
    LedgerRow { id: "self_forming_clay", item: ItemId::Relic(RelicId::SelfFormingClay), kind: "relic", key: "E", file: "src/items/tests/m27_existing_relics.rs", test: "m27_existing_relics_equal_accepted" },
    LedgerRow { id: "orichalcum", item: ItemId::Relic(RelicId::Orichalcum), kind: "relic", key: "E", file: "src/items/tests/m27_existing_relics.rs", test: "m27_existing_relics_equal_accepted" },
    LedgerRow { id: "brimstone", item: ItemId::Relic(RelicId::Brimstone), kind: "relic", key: "E", file: "src/items/tests/m27_existing_relics.rs", test: "m27_existing_relics_equal_accepted" },
    LedgerRow { id: "mark_of_pain", item: ItemId::Relic(RelicId::MarkOfPain), kind: "relic", key: "E", file: "src/items/tests/m27_existing_relics.rs", test: "m27_existing_relics_equal_accepted" },
    LedgerRow { id: "runic_cube", item: ItemId::Relic(RelicId::RunicCube), kind: "relic", key: "E", file: "src/items/tests/m27_existing_relics.rs", test: "m27_existing_relics_equal_accepted" },
    LedgerRow { id: "snecko_skull", item: ItemId::Relic(RelicId::SneckoSkull), kind: "relic", key: "E", file: "src/items/tests/m27_existing_relics.rs", test: "m27_existing_relics_equal_accepted" },
    LedgerRow { id: "ninja_scroll", item: ItemId::Relic(RelicId::NinjaScroll), kind: "relic", key: "E", file: "src/items/tests/m27_existing_relics.rs", test: "m27_existing_relics_equal_accepted" },
    LedgerRow { id: "paper_krane", item: ItemId::Relic(RelicId::PaperKrane), kind: "relic", key: "E", file: "src/items/tests/m27_existing_relics.rs", test: "m27_existing_relics_equal_accepted" },
    LedgerRow { id: "the_specimen", item: ItemId::Relic(RelicId::TheSpecimen), kind: "relic", key: "E", file: "src/items/tests/m27_existing_relics.rs", test: "m27_existing_relics_equal_accepted" },
    LedgerRow { id: "tingsha", item: ItemId::Relic(RelicId::Tingsha), kind: "relic", key: "E", file: "src/items/tests/m27_existing_relics.rs", test: "m27_existing_relics_equal_accepted" },
    LedgerRow { id: "tough_bandages", item: ItemId::Relic(RelicId::ToughBandages), kind: "relic", key: "E", file: "src/items/tests/m27_relics_structural.rs", test: "m27_gambling_chip_discards_trigger_tough_bandages" },
    LedgerRow { id: "twisted_funnel", item: ItemId::Relic(RelicId::TwistedFunnel), kind: "relic", key: "E", file: "src/items/tests/m27_existing_relics.rs", test: "m27_existing_relics_equal_accepted" },
    LedgerRow { id: "hovering_kite", item: ItemId::Relic(RelicId::HoveringKite), kind: "relic", key: "E", file: "src/items/tests/m27_existing_relics.rs", test: "m27_existing_relics_equal_accepted" },
    LedgerRow { id: "ring_of_the_serpent", item: ItemId::Relic(RelicId::RingOfTheSerpent), kind: "relic", key: "E", file: "src/items/tests/m27_existing_relics.rs", test: "m27_existing_relics_equal_accepted" },
    LedgerRow { id: "wrist_blade", item: ItemId::Relic(RelicId::WristBlade), kind: "relic", key: "E", file: "src/items/tests/m27_existing_relics.rs", test: "m27_existing_relics_equal_accepted" },
    LedgerRow { id: "black_blood", item: ItemId::Relic(RelicId::BlackBlood), kind: "relic", key: "E", file: "src/items/tests/m27_existing_relics.rs", test: "m27_existing_relics_equal_accepted" },
    LedgerRow { id: "empty_cage", item: ItemId::Relic(RelicId::EmptyCage), kind: "relic", key: "E", file: "src/items/tests/m27_existing_relics.rs", test: "m27_existing_relics_equal_accepted" },
    LedgerRow { id: "slavers_collar", item: ItemId::Relic(RelicId::SlaversCollar), kind: "relic", key: "E", file: "src/items/tests/m27_existing_relics.rs", test: "m27_existing_relics_equal_accepted" },
    LedgerRow { id: "neows_lament", item: ItemId::Relic(RelicId::NeowsLament), kind: "relic", key: "E", file: "src/items/tests/m27_existing_relics.rs", test: "m27_existing_relics_equal_accepted" },
    LedgerRow { id: "burning_blood", item: ItemId::Relic(RelicId::BurningBlood), kind: "relic", key: "E", file: "src/items/tests/m27_existing_relics.rs", test: "m27_existing_relics_equal_accepted" },
    LedgerRow { id: "ring_of_the_snake", item: ItemId::Relic(RelicId::RingOfTheSnake), kind: "relic", key: "E", file: "src/items/tests/m27_existing_relics.rs", test: "m27_existing_relics_equal_accepted" },
    LedgerRow { id: "cracked_core", item: ItemId::Relic(RelicId::CrackedCore), kind: "relic", key: "E", file: "src/items/tests/m27_existing_relics.rs", test: "m27_existing_relics_equal_accepted" },
    LedgerRow { id: "pure_water", item: ItemId::Relic(RelicId::PureWater), kind: "relic", key: "E", file: "src/items/tests/m27_existing_relics.rs", test: "m27_g10_legacy_pure_water_pre_draw" },
    LedgerRow { id: "blood_vial", item: ItemId::Relic(RelicId::BloodVial), kind: "relic", key: "E", file: "src/items/tests/m27_existing_relics.rs", test: "m27_existing_relics_equal_accepted" },
    LedgerRow { id: "anchor", item: ItemId::Relic(RelicId::Anchor), kind: "relic", key: "E", file: "src/items/tests/m27_existing_relics.rs", test: "m27_existing_relics_equal_accepted" },
    LedgerRow { id: "lantern", item: ItemId::Relic(RelicId::Lantern), kind: "relic", key: "E", file: "src/items/tests/m27_existing_relics.rs", test: "m27_existing_relics_equal_accepted" },
    LedgerRow { id: "strawberry", item: ItemId::Relic(RelicId::Strawberry), kind: "relic", key: "E", file: "src/items/tests/m27_existing_relics.rs", test: "m27_existing_relics_equal_accepted" },
    LedgerRow { id: "pear", item: ItemId::Relic(RelicId::Pear), kind: "relic", key: "E", file: "src/items/tests/m27_existing_relics.rs", test: "m27_existing_relics_equal_accepted" },
    LedgerRow { id: "mango", item: ItemId::Relic(RelicId::Mango), kind: "relic", key: "E", file: "src/items/tests/m27_existing_relics.rs", test: "m27_existing_relics_equal_accepted" },
    LedgerRow { id: "sacred_bark", item: ItemId::Relic(RelicId::SacredBark), kind: "relic", key: "E", file: "src/items/tests/m27_existing_relics.rs", test: "m27_existing_relics_equal_accepted" },
    LedgerRow { id: "mark_of_bloom", item: ItemId::Relic(RelicId::MarkOfBloom), kind: "relic", key: "E", file: "src/items/tests/m27_existing_relics.rs", test: "m27_existing_relics_equal_accepted" },
    LedgerRow { id: "magic_flower", item: ItemId::Relic(RelicId::MagicFlower), kind: "relic", key: "E", file: "src/items/tests/m27_existing_relics.rs", test: "m27_existing_relics_equal_accepted" },
    LedgerRow { id: "toy_ornithopter", item: ItemId::Relic(RelicId::ToyOrnithopter), kind: "relic", key: "E", file: "src/items/tests/m27_existing_relics.rs", test: "m27_existing_relics_equal_accepted" },
    LedgerRow { id: "sozu", item: ItemId::Relic(RelicId::Sozu), kind: "relic", key: "E", file: "src/items/tests/m27_energy_law.rs", test: "m27_g1_sozu_energy_was_missing" },
    LedgerRow { id: "potion_belt", item: ItemId::Relic(RelicId::PotionBelt), kind: "relic", key: "E", file: "src/items/tests/m27_existing_relics.rs", test: "m27_existing_relics_equal_accepted" },
    LedgerRow { id: "data_disk", item: ItemId::Relic(RelicId::DataDisk), kind: "relic", key: "E", file: "src/items/tests/m27_existing_relics.rs", test: "m27_existing_relics_equal_accepted" },
    LedgerRow { id: "gold_plated_cables", item: ItemId::Relic(RelicId::GoldPlatedCables), kind: "relic", key: "E", file: "src/items/tests/m27_existing_relics.rs", test: "m27_existing_relics_equal_accepted" },
    LedgerRow { id: "emotion_chip", item: ItemId::Relic(RelicId::EmotionChip), kind: "relic", key: "E", file: "src/items/tests/m27_existing_relics.rs", test: "m27_existing_relics_equal_accepted" },
    LedgerRow { id: "symbiotic_virus", item: ItemId::Relic(RelicId::SymbioticVirus), kind: "relic", key: "E", file: "src/items/tests/m27_existing_relics.rs", test: "m27_existing_relics_equal_accepted" },
    LedgerRow { id: "frozen_core", item: ItemId::Relic(RelicId::FrozenCore), kind: "relic", key: "E", file: "src/items/tests/m27_existing_relics.rs", test: "m27_existing_relics_equal_accepted" },
    LedgerRow { id: "inserter", item: ItemId::Relic(RelicId::Inserter), kind: "relic", key: "E", file: "src/items/tests/m27_existing_relics.rs", test: "m27_existing_relics_equal_accepted" },
    LedgerRow { id: "nuclear_battery", item: ItemId::Relic(RelicId::NuclearBattery), kind: "relic", key: "E", file: "src/items/tests/m27_existing_relics.rs", test: "m27_existing_relics_equal_accepted" },
    LedgerRow { id: "runic_capacitor", item: ItemId::Relic(RelicId::RunicCapacitor), kind: "relic", key: "E", file: "src/items/tests/m27_existing_relics.rs", test: "m27_existing_relics_equal_accepted" },
    LedgerRow { id: "violet_lotus", item: ItemId::Relic(RelicId::VioletLotus), kind: "relic", key: "E", file: "src/items/tests/m27_existing_relics.rs", test: "m27_existing_relics_equal_accepted" },
    LedgerRow { id: "golden_eye", item: ItemId::Relic(RelicId::GoldenEye), kind: "relic", key: "E", file: "src/items/tests/m27_existing_relics.rs", test: "m27_existing_relics_equal_accepted" },
    LedgerRow { id: "teardrop_locket", item: ItemId::Relic(RelicId::TeardropLocket), kind: "relic", key: "E", file: "src/items/tests/m27_existing_relics.rs", test: "m27_existing_relics_equal_accepted" },
    LedgerRow { id: "damaru", item: ItemId::Relic(RelicId::Damaru), kind: "relic", key: "E", file: "src/items/tests/m27_existing_relics.rs", test: "m27_existing_relics_equal_accepted" },
    LedgerRow { id: "duality", item: ItemId::Relic(RelicId::Duality), kind: "relic", key: "E", file: "src/items/tests/m27_existing_relics.rs", test: "m27_existing_relics_equal_accepted" },
    LedgerRow { id: "cloak_clasp", item: ItemId::Relic(RelicId::CloakClasp), kind: "relic", key: "E", file: "src/items/tests/m27_existing_relics.rs", test: "m27_existing_relics_equal_accepted" },
    LedgerRow { id: "melange", item: ItemId::Relic(RelicId::Melange), kind: "relic", key: "E", file: "src/items/tests/m27_existing_relics.rs", test: "m27_existing_relics_equal_accepted" },
    LedgerRow { id: "akabeko", item: ItemId::Relic(RelicId::Akabeko), kind: "relic", key: "C1", file: "src/items/tests/m27_relics_turns.rs", test: "m27_akabeko_first_attack_bonus" },
    LedgerRow { id: "ancient_tea_set", item: ItemId::Relic(RelicId::AncientTeaSet), kind: "relic", key: "C3", file: "src/run/tests/m27_relics_run.rs", test: "m27_ancient_tea_set_rest_arms_energy" },
    LedgerRow { id: "art_of_war", item: ItemId::Relic(RelicId::ArtOfWar), kind: "relic", key: "C1", file: "src/items/tests/m27_relics_turns.rs", test: "m27_art_of_war_no_attack_energy" },
    LedgerRow { id: "the_boot", item: ItemId::Relic(RelicId::TheBoot), kind: "relic", key: "C2", file: "src/items/tests/m27_relics_play.rs", test: "m27_the_boot_four_damage_minimum" },
    LedgerRow { id: "centennial_puzzle", item: ItemId::Relic(RelicId::CentennialPuzzle), kind: "relic", key: "C2", file: "src/items/tests/m27_relics_play.rs", test: "m27_centennial_puzzle_first_hp_loss_draw" },
    LedgerRow { id: "ceramic_fish", item: ItemId::Relic(RelicId::CeramicFish), kind: "relic", key: "C3", file: "src/run/tests/m27_relics_run.rs", test: "m27_ceramic_fish_card_obtain_gold" },
    LedgerRow { id: "dream_catcher", item: ItemId::Relic(RelicId::DreamCatcher), kind: "relic", key: "C3", file: "src/run/tests/m27_relics_run.rs", test: "m27_dream_catcher_rest_card_reward" },
    LedgerRow { id: "happy_flower", item: ItemId::Relic(RelicId::HappyFlower), kind: "relic", key: "C1", file: "src/items/tests/m27_relics_turns.rs", test: "m27_happy_flower_third_turn_energy" },
    LedgerRow { id: "juzu_bracelet", item: ItemId::Relic(RelicId::JuzuBracelet), kind: "relic", key: "C3", file: "src/run/tests/m27_relics_run.rs", test: "m27_juzu_bracelet_monster_to_event" },
    LedgerRow { id: "maw_bank", item: ItemId::Relic(RelicId::MawBank), kind: "relic", key: "C3", file: "src/run/tests/m27_relics_run.rs", test: "m27_maw_bank_floor_gold_shop_disable" },
    LedgerRow { id: "meal_ticket", item: ItemId::Relic(RelicId::MealTicket), kind: "relic", key: "C3", file: "src/run/tests/m27_relics_run.rs", test: "m27_meal_ticket_shop_entry_heal" },
    LedgerRow { id: "nunchaku", item: ItemId::Relic(RelicId::Nunchaku), kind: "relic", key: "C2", file: "src/items/tests/m27_relics_play.rs", test: "m27_nunchaku_tenth_attack_energy" },
    LedgerRow { id: "omamori", item: ItemId::Relic(RelicId::Omamori), kind: "relic", key: "C3", file: "src/run/tests/m27_relics_run.rs", test: "m27_omamori_negates_through_the_card_owner" },
    LedgerRow { id: "pen_nib", item: ItemId::Relic(RelicId::PenNib), kind: "relic", key: "C2", file: "src/items/tests/m27_relics_play.rs", test: "m27_pen_nib_tenth_attack_doubles" },
    LedgerRow { id: "preserved_insect", item: ItemId::Relic(RelicId::PreservedInsect), kind: "relic", key: "C1", file: "src/items/tests/m27_relics_turns.rs", test: "m27_preserved_insect_elite_hp_floor" },
    LedgerRow { id: "regal_pillow", item: ItemId::Relic(RelicId::RegalPillow), kind: "relic", key: "C3", file: "src/run/tests/m27_relics_run.rs", test: "m27_regal_pillow_rest_heal_plus_fifteen" },
    LedgerRow { id: "smiling_mask", item: ItemId::Relic(RelicId::SmilingMask), kind: "relic", key: "C3", file: "src/run/tests/m27_relics_run.rs", test: "m27_smiling_mask_removal_price_fifty" },
    LedgerRow { id: "tiny_chest", item: ItemId::Relic(RelicId::TinyChest), kind: "relic", key: "C3", file: "src/run/tests/m27_relics_run.rs", test: "m27_tiny_chest_fourth_unknown_treasure" },
    LedgerRow { id: "war_paint", item: ItemId::Relic(RelicId::WarPaint), kind: "relic", key: "C3", file: "src/run/tests/m27_relics_run.rs", test: "m27_war_paint_two_skill_upgrades" },
    LedgerRow { id: "whetstone", item: ItemId::Relic(RelicId::Whetstone), kind: "relic", key: "C3", file: "src/run/tests/m27_relics_run.rs", test: "m27_whetstone_two_attack_upgrades" },
    LedgerRow { id: "blue_candle", item: ItemId::Relic(RelicId::BlueCandle), kind: "relic", key: "C2", file: "src/items/tests/m27_relics_play.rs", test: "m27_blue_candle_curse_play_exhaust" },
    LedgerRow { id: "bottled_flame", item: ItemId::Relic(RelicId::BottledFlame), kind: "relic", key: "J", file: "src/run/tests/m27_relic_selections.rs", test: "m27_bottled_flame_selected_attack_innate" },
    LedgerRow { id: "bottled_lightning", item: ItemId::Relic(RelicId::BottledLightning), kind: "relic", key: "J", file: "src/run/tests/m27_relic_selections.rs", test: "m27_bottled_lightning_selected_skill_innate" },
    LedgerRow { id: "bottled_tornado", item: ItemId::Relic(RelicId::BottledTornado), kind: "relic", key: "J", file: "src/run/tests/m27_relic_selections.rs", test: "m27_bottled_tornado_selected_power_innate" },
    LedgerRow { id: "the_courier", item: ItemId::Relic(RelicId::TheCourier), kind: "relic", key: "C3", file: "src/run/tests/m27_relics_run.rs", test: "m27_the_courier_shop_price_twenty_off" },
    LedgerRow { id: "darkstone_periapt", item: ItemId::Relic(RelicId::DarkstonePeriapt), kind: "relic", key: "C3", file: "src/run/tests/m27_relics_run.rs", test: "m27_darkstone_periapt_curse_max_hp_six" },
    LedgerRow { id: "eternal_feather", item: ItemId::Relic(RelicId::EternalFeather), kind: "relic", key: "C3", file: "src/run/tests/m27_relics_run.rs", test: "m27_eternal_feather_rest_room_heal" },
    LedgerRow { id: "frozen_egg", item: ItemId::Relic(RelicId::FrozenEgg), kind: "relic", key: "C3", file: "src/run/tests/m27_relics_run.rs", test: "m27_frozen_egg_power_obtain_upgrade" },
    LedgerRow { id: "gremlin_horn", item: ItemId::Relic(RelicId::GremlinHorn), kind: "relic", key: "C2", file: "src/items/tests/m27_relics_play.rs", test: "m27_gremlin_horn_nonfinal_kill_payoff" },
    LedgerRow { id: "horn_cleat", item: ItemId::Relic(RelicId::HornCleat), kind: "relic", key: "C1", file: "src/items/tests/m27_relics_turns.rs", test: "m27_horn_cleat_second_turn_block" },
    LedgerRow { id: "ink_bottle", item: ItemId::Relic(RelicId::InkBottle), kind: "relic", key: "C2", file: "src/items/tests/m27_relics_play.rs", test: "m27_ink_bottle_tenth_card_draw" },
    LedgerRow { id: "kunai", item: ItemId::Relic(RelicId::Kunai), kind: "relic", key: "C2", file: "src/items/tests/m27_relics_play.rs", test: "m27_kunai_third_attack_dexterity" },
    LedgerRow { id: "letter_opener", item: ItemId::Relic(RelicId::LetterOpener), kind: "relic", key: "C2", file: "src/items/tests/m27_relics_play.rs", test: "m27_letter_opener_third_skill_thorns" },
    LedgerRow { id: "matryoshka", item: ItemId::Relic(RelicId::Matryoshka), kind: "relic", key: "C3", file: "src/run/tests/m27_relics_run.rs", test: "m27_matryoshka_nonboss_chest_charge" },
    LedgerRow { id: "meat_on_the_bone", item: ItemId::Relic(RelicId::MeatOnTheBone), kind: "relic", key: "C1", file: "src/items/tests/m27_relics_turns.rs", test: "m27_meat_on_the_bone_half_hp_heal" },
    LedgerRow { id: "mercury_hourglass", item: ItemId::Relic(RelicId::MercuryHourglass), kind: "relic", key: "C1", file: "src/items/tests/m27_relics_turns.rs", test: "m27_mercury_hourglass_turn_start_thorns" },
    LedgerRow { id: "molten_egg", item: ItemId::Relic(RelicId::MoltenEgg), kind: "relic", key: "C3", file: "src/run/tests/m27_relics_run.rs", test: "m27_molten_egg_attack_obtain_upgrade" },
    LedgerRow { id: "mummified_hand", item: ItemId::Relic(RelicId::MummifiedHand), kind: "relic", key: "C2", file: "src/items/tests/m27_relics_play.rs", test: "m27_mummified_hand_power_cost_zero" },
    LedgerRow { id: "ornamental_fan", item: ItemId::Relic(RelicId::OrnamentalFan), kind: "relic", key: "C2", file: "src/items/tests/m27_relics_play.rs", test: "m27_ornamental_fan_third_attack_block" },
    LedgerRow { id: "pantograph", item: ItemId::Relic(RelicId::Pantograph), kind: "relic", key: "C1", file: "src/items/tests/m27_relics_turns.rs", test: "m27_pantograph_boss_start_heal" },
    LedgerRow { id: "question_card", item: ItemId::Relic(RelicId::QuestionCard), kind: "relic", key: "C3", file: "src/run/tests/m27_relics_run.rs", test: "m27_question_card_reward_choice_plus_one" },
    LedgerRow { id: "shuriken", item: ItemId::Relic(RelicId::Shuriken), kind: "relic", key: "C2", file: "src/items/tests/m27_relics_play.rs", test: "m27_shuriken_third_attack_strength" },
    LedgerRow { id: "singing_bowl", item: ItemId::Relic(RelicId::SingingBowl), kind: "relic", key: "C3", file: "src/run/tests/m27_relics_run.rs", test: "m27_singing_bowl_bowl_max_hp_two" },
    LedgerRow { id: "strike_dummy", item: ItemId::Relic(RelicId::StrikeDummy), kind: "relic", key: "C2", file: "src/items/tests/m27_relics_play.rs", test: "m27_strike_dummy_strike_tag_bonus" },
    LedgerRow { id: "sundial", item: ItemId::Relic(RelicId::Sundial), kind: "relic", key: "C1", file: "src/items/tests/m27_relics_turns.rs", test: "m27_sundial_third_shuffle_energy" },
    LedgerRow { id: "toxic_egg", item: ItemId::Relic(RelicId::ToxicEgg), kind: "relic", key: "C3", file: "src/run/tests/m27_relics_run.rs", test: "m27_toxic_egg_skill_obtain_upgrade" },
    LedgerRow { id: "white_beast_statue", item: ItemId::Relic(RelicId::WhiteBeastStatue), kind: "relic", key: "C3", file: "src/run/tests/m27_relics_run.rs", test: "m27_white_beast_statue_potion_chance_override" },
    LedgerRow { id: "bird_faced_urn", item: ItemId::Relic(RelicId::BirdFacedUrn), kind: "relic", key: "C2", file: "src/items/tests/m27_relics_play.rs", test: "m27_bird_faced_urn_power_use_heal" },
    LedgerRow { id: "calipers", item: ItemId::Relic(RelicId::Calipers), kind: "relic", key: "H", file: "src/items/tests/m27_energy_law.rs", test: "m27_calipers_keeps_fifteen_less" },
    LedgerRow { id: "captains_wheel", item: ItemId::Relic(RelicId::CaptainsWheel), kind: "relic", key: "C1", file: "src/items/tests/m27_relics_turns.rs", test: "m27_captains_wheel_third_turn_block" },
    LedgerRow { id: "dead_branch", item: ItemId::Relic(RelicId::DeadBranch), kind: "relic", key: "C2", file: "src/items/tests/m27_relics_play.rs", test: "m27_dead_branch_actual_exhaust_card" },
    LedgerRow { id: "du_vu_doll", item: ItemId::Relic(RelicId::DuVuDoll), kind: "relic", key: "C1", file: "src/items/tests/m27_relics_turns.rs", test: "m27_du_vu_doll_curse_count_strength" },
    LedgerRow { id: "fossilized_helix", item: ItemId::Relic(RelicId::FossilizedHelix), kind: "relic", key: "C1", file: "src/items/tests/m27_relics_turns.rs", test: "m27_fossilized_helix_first_hit_buffer" },
    LedgerRow { id: "gambling_chip", item: ItemId::Relic(RelicId::GamblingChip), kind: "relic", key: "J", file: "src/items/tests/m27_relics_structural.rs", test: "m27_gambling_chip_discards_trigger_tough_bandages" },
    LedgerRow { id: "ginger", item: ItemId::Relic(RelicId::Ginger), kind: "relic", key: "C2", file: "src/items/tests/m27_relics_play.rs", test: "m27_ginger_weak_prevention" },
    LedgerRow { id: "girya", item: ItemId::Relic(RelicId::Girya), kind: "relic", key: "C3", file: "src/run/tests/m27_relics_run.rs", test: "m27_girya_three_lifts_strength" },
    LedgerRow { id: "ice_cream", item: ItemId::Relic(RelicId::IceCream), kind: "relic", key: "H", file: "src/items/tests/m27_energy_law.rs", test: "m27_energy_law_all_characters" },
    LedgerRow { id: "incense_burner", item: ItemId::Relic(RelicId::IncenseBurner), kind: "relic", key: "C1", file: "src/items/tests/m27_relics_turns.rs", test: "m27_incense_burner_sixth_turn_intangible" },
    LedgerRow { id: "lizard_tail", item: ItemId::Relic(RelicId::LizardTail), kind: "relic", key: "H", file: "src/combat/tests/m27_rescue_escape.rs", test: "m27_rescue_order_fairy_then_lizard_tail_once" },
    LedgerRow { id: "old_coin", item: ItemId::Relic(RelicId::OldCoin), kind: "relic", key: "C3", file: "src/run/tests/m27_relics_run.rs", test: "m27_old_coin_equip_gold_300" },
    LedgerRow { id: "peace_pipe", item: ItemId::Relic(RelicId::PeacePipe), kind: "relic", key: "C3", file: "src/run/tests/m27_relics_run.rs", test: "m27_peace_pipe_toke_remove_card" },
    LedgerRow { id: "pocketwatch", item: ItemId::Relic(RelicId::Pocketwatch), kind: "relic", key: "C1", file: "src/items/tests/m27_relics_turns.rs", test: "m27_pocketwatch_later_turn_draw" },
    LedgerRow { id: "prayer_wheel", item: ItemId::Relic(RelicId::PrayerWheel), kind: "relic", key: "C3", file: "src/run/tests/m27_relics_run.rs", test: "m27_prayer_wheel_monster_second_card_row" },
    LedgerRow { id: "shovel", item: ItemId::Relic(RelicId::Shovel), kind: "relic", key: "C3", file: "src/run/tests/m27_relics_run.rs", test: "m27_shovel_dig_front_relic" },
    LedgerRow { id: "stone_calendar", item: ItemId::Relic(RelicId::StoneCalendar), kind: "relic", key: "C1", file: "src/items/tests/m27_relics_turns.rs", test: "m27_stone_calendar_seventh_turn_thorns" },
    LedgerRow { id: "thread_and_needle", item: ItemId::Relic(RelicId::ThreadAndNeedle), kind: "relic", key: "C1", file: "src/items/tests/m27_relics_turns.rs", test: "m27_thread_and_needle_start_plated_armor" },
    LedgerRow { id: "torii", item: ItemId::Relic(RelicId::Torii), kind: "relic", key: "C2", file: "src/items/tests/m27_relics_play.rs", test: "m27_torii_two_to_five_damage" },
    LedgerRow { id: "tungsten_rod", item: ItemId::Relic(RelicId::TungstenRod), kind: "relic", key: "C2", file: "src/items/tests/m27_relics_play.rs", test: "m27_tungsten_rod_positive_loss_minus_one" },
    LedgerRow { id: "turnip", item: ItemId::Relic(RelicId::Turnip), kind: "relic", key: "C2", file: "src/items/tests/m27_relics_play.rs", test: "m27_turnip_frail_prevention" },
    LedgerRow { id: "unceasing_top", item: ItemId::Relic(RelicId::UnceasingTop), kind: "relic", key: "J", file: "src/items/tests/m27_relics_structural.rs", test: "m27_unceasing_top_empty_hand_draw" },
    LedgerRow { id: "wing_boots", item: ItemId::Relic(RelicId::WingBoots), kind: "relic", key: "C3", file: "src/run/tests/m27_relics_run.rs", test: "m27_wing_boots_three_offpath_moves" },
    LedgerRow { id: "astrolabe", item: ItemId::Relic(RelicId::Astrolabe), kind: "relic", key: "J", file: "src/run/tests/m27_relic_selections.rs", test: "m27_astrolabe_three_transform_upgrade" },
    LedgerRow { id: "black_star", item: ItemId::Relic(RelicId::BlackStar), kind: "relic", key: "C3", file: "src/run/tests/m27_relics_run.rs", test: "m27_black_star_elite_second_relic" },
    LedgerRow { id: "busted_crown", item: ItemId::Relic(RelicId::BustedCrown), kind: "relic", key: "C3", file: "src/run/tests/m27_relics_run.rs", test: "m27_busted_crown_one_reward_choice" },
    LedgerRow { id: "calling_bell", item: ItemId::Relic(RelicId::CallingBell), kind: "relic", key: "J", file: "src/run/tests/m27_relic_selections.rs", test: "m27_calling_bell_curse_three_tiers" },
    LedgerRow { id: "coffee_dripper", item: ItemId::Relic(RelicId::CoffeeDripper), kind: "relic", key: "C3", file: "src/run/tests/m27_relics_run.rs", test: "m27_coffee_dripper_no_rest_plus_energy" },
    LedgerRow { id: "cursed_key", item: ItemId::Relic(RelicId::CursedKey), kind: "relic", key: "C3", file: "src/run/tests/m27_relics_run.rs", test: "m27_cursed_key_nonboss_chest_curse" },
    LedgerRow { id: "ectoplasm", item: ItemId::Relic(RelicId::Ectoplasm), kind: "relic", key: "C3", file: "src/run/tests/m27_relics_run.rs", test: "m27_ectoplasm_gold_gain_blocked" },
    LedgerRow { id: "fusion_hammer", item: ItemId::Relic(RelicId::FusionHammer), kind: "relic", key: "C3", file: "src/run/tests/m27_relics_run.rs", test: "m27_fusion_hammer_no_smith_plus_energy" },
    LedgerRow { id: "pandoras_box", item: ItemId::Relic(RelicId::PandorasBox), kind: "relic", key: "J", file: "src/run/tests/m27_relic_selections.rs", test: "m27_pandoras_box_starter_card_transform" },
    LedgerRow { id: "philosophers_stone", item: ItemId::Relic(RelicId::PhilosophersStone), kind: "relic", key: "H", file: "src/items/tests/m27_energy_law.rs", test: "m27_philosophers_stone_strength_and_energy" },
    LedgerRow { id: "runic_dome", item: ItemId::Relic(RelicId::RunicDome), kind: "relic", key: "J", file: "src/items/tests/m27_relics_structural.rs", test: "m27_runic_dome_hide_enemy_intents" },
    LedgerRow { id: "runic_pyramid", item: ItemId::Relic(RelicId::RunicPyramid), kind: "relic", key: "J", file: "src/items/tests/m27_relics_structural.rs", test: "m27_runic_pyramid_retain_hand" },
    LedgerRow { id: "snecko_eye", item: ItemId::Relic(RelicId::SneckoEye), kind: "relic", key: "J", file: "src/items/tests/m27_relics_structural.rs", test: "m27_snecko_eye_confused_on_draw" },
    LedgerRow { id: "tiny_house", item: ItemId::Relic(RelicId::TinyHouse), kind: "relic", key: "J", file: "src/run/tests/m27_relic_selections.rs", test: "m27_tiny_house_equip_reward_bundle" },
    LedgerRow { id: "velvet_choker", item: ItemId::Relic(RelicId::VelvetChoker), kind: "relic", key: "J", file: "src/items/tests/m27_relics_structural.rs", test: "m27_velvet_choker_six_play_limit" },
    LedgerRow { id: "the_abacus", item: ItemId::Relic(RelicId::TheAbacus), kind: "relic", key: "C1", file: "src/items/tests/m27_relics_turns.rs", test: "m27_the_abacus_shuffle_block" },
    LedgerRow { id: "cauldron", item: ItemId::Relic(RelicId::Cauldron), kind: "relic", key: "J", file: "src/run/tests/m27_relic_selections.rs", test: "m27_cauldron_five_potion_offers" },
    LedgerRow { id: "clockwork_souvenir", item: ItemId::Relic(RelicId::ClockworkSouvenir), kind: "relic", key: "C1", file: "src/items/tests/m27_relics_turns.rs", test: "m27_clockwork_souvenir_start_artifact" },
    LedgerRow { id: "dollys_mirror", item: ItemId::Relic(RelicId::DollysMirror), kind: "relic", key: "J", file: "src/run/tests/m27_relic_selections.rs", test: "m27_dollys_mirror_duplicate_card_instance" },
    LedgerRow { id: "frozen_eye", item: ItemId::Relic(RelicId::FrozenEye), kind: "relic", key: "J", file: "src/items/tests/m27_relics_structural.rs", test: "m27_frozen_eye_draw_order_visible" },
    LedgerRow { id: "hand_drill", item: ItemId::Relic(RelicId::HandDrill), kind: "relic", key: "C2", file: "src/items/tests/m27_relics_play.rs", test: "m27_hand_drill_block_break_vulnerable" },
    LedgerRow { id: "medical_kit", item: ItemId::Relic(RelicId::MedicalKit), kind: "relic", key: "C2", file: "src/items/tests/m27_relics_play.rs", test: "m27_medical_kit_status_play_exhaust" },
    LedgerRow { id: "membership_card", item: ItemId::Relic(RelicId::MembershipCard), kind: "relic", key: "C3", file: "src/run/tests/m27_relics_run.rs", test: "m27_membership_card_shop_price_half" },
    LedgerRow { id: "orange_pellets", item: ItemId::Relic(RelicId::OrangePellets), kind: "relic", key: "C2", file: "src/items/tests/m27_relics_play.rs", test: "m27_orange_pellets_complete_card_trio" },
    LedgerRow { id: "orrery", item: ItemId::Relic(RelicId::Orrery), kind: "relic", key: "J", file: "src/run/tests/m27_relic_selections.rs", test: "m27_orrery_five_card_rewards" },
    LedgerRow { id: "prismatic_shard", item: ItemId::Relic(RelicId::PrismaticShard), kind: "relic", key: "C3", file: "src/run/tests/m27_potion_rewards.rs", test: "m27_prismatic_shard_any_colour_card_reward" },
    LedgerRow { id: "sling_of_courage", item: ItemId::Relic(RelicId::SlingOfCourage), kind: "relic", key: "C1", file: "src/items/tests/m27_relics_turns.rs", test: "m27_sling_of_courage_elite_trigger_strength" },
    LedgerRow { id: "strange_spoon", item: ItemId::Relic(RelicId::StrangeSpoon), kind: "relic", key: "J", file: "src/items/tests/m27_relics_structural.rs", test: "m27_strange_spoon_eligible_exhaust_save" },
    LedgerRow { id: "toolbox", item: ItemId::Relic(RelicId::Toolbox), kind: "relic", key: "J", file: "src/items/tests/m27_relics_structural.rs", test: "m27_toolbox_three_colorless_choose" },
    LedgerRow { id: "lees_waffle", item: ItemId::Relic(RelicId::LeesWaffle), kind: "relic", key: "C3", file: "src/run/tests/m27_relics_run.rs", test: "m27_lees_waffle_max_hp_full_heal" },
    LedgerRow { id: "bloody_idol", item: ItemId::Relic(RelicId::BloodyIdol), kind: "relic", key: "C3", file: "src/run/tests/m27_relics_run.rs", test: "m27_bloody_idol_gold_gain_heal_five" },
    LedgerRow { id: "circlet", item: ItemId::Relic(RelicId::Circlet), kind: "relic", key: "C3", file: "src/run/tests/m27_relics_run.rs", test: "m27_circlet_rare_pool_fallback" },
    LedgerRow { id: "cultist_headpiece", item: ItemId::Relic(RelicId::CultistHeadpiece), kind: "relic", key: "C1", file: "src/items/tests/m27_relics_turns.rs", test: "m27_cultist_headpiece_speech_only" },
    LedgerRow { id: "enchiridion", item: ItemId::Relic(RelicId::Enchiridion), kind: "relic", key: "C1", file: "src/items/tests/m27_relics_turns.rs", test: "m27_enchiridion_class_power_zero" },
    LedgerRow { id: "face_of_cleric", item: ItemId::Relic(RelicId::FaceOfCleric), kind: "relic", key: "C1", file: "src/items/tests/m27_relics_turns.rs", test: "m27_face_of_cleric_victory_max_hp" },
    LedgerRow { id: "golden_idol", item: ItemId::Relic(RelicId::GoldenIdol), kind: "relic", key: "C3", file: "src/run/tests/m27_relics_run.rs", test: "m27_golden_idol_stored_gold_bonus" },
    LedgerRow { id: "gremlin_visage", item: ItemId::Relic(RelicId::GremlinVisage), kind: "relic", key: "C1", file: "src/items/tests/m27_relics_turns.rs", test: "m27_gremlin_visage_player_weak" },
    LedgerRow { id: "mutagenic_strength", item: ItemId::Relic(RelicId::MutagenicStrength), kind: "relic", key: "C1", file: "src/items/tests/m27_relics_turns.rs", test: "m27_mutagenic_strength_first_turn_strength" },
    LedgerRow { id: "necronomicon", item: ItemId::Relic(RelicId::Necronomicon), kind: "relic", key: "J", file: "src/items/tests/m27_relics_structural.rs", test: "m27_necronomicon_first_attack_repeat" },
    LedgerRow { id: "nilrys_codex", item: ItemId::Relic(RelicId::NilrysCodex), kind: "relic", key: "J", file: "src/items/tests/m27_relics_structural.rs", test: "m27_nilrys_codex_choose_one_of_three" },
    LedgerRow { id: "nloths_gift", item: ItemId::Relic(RelicId::NlothsGift), kind: "relic", key: "C3", file: "src/run/tests/m27_relics_run.rs", test: "m27_nloths_gift_rare_chance_tripled" },
    LedgerRow { id: "nloths_hungry_face", item: ItemId::Relic(RelicId::NlothsHungryFace), kind: "relic", key: "C3", file: "src/run/tests/m27_relics_run.rs", test: "m27_nloths_hungry_face_next_chest_relic_removal" },
    LedgerRow { id: "odd_mushroom", item: ItemId::Relic(RelicId::OddMushroom), kind: "relic", key: "C2", file: "src/items/tests/m27_relics_play.rs", test: "m27_odd_mushroom_vulnerable_damage" },
    LedgerRow { id: "spirit_poop", item: ItemId::Relic(RelicId::SpiritPoop), kind: "relic", key: "C3", file: "src/run/tests/m27_relics_run.rs", test: "m27_spirit_poop_score_minus_one" },
    LedgerRow { id: "ssserpent_head", item: ItemId::Relic(RelicId::SsserpentHead), kind: "relic", key: "C3", file: "src/run/tests/m27_relics_run.rs", test: "m27_ssserpent_head_event_room_gold_fifty" },
    LedgerRow { id: "warped_tongs", item: ItemId::Relic(RelicId::WarpedTongs), kind: "relic", key: "C1", file: "src/items/tests/m27_relics_turns.rs", test: "m27_warped_tongs_postdraw_upgrade" },
    LedgerRow { id: "fire_potion", item: ItemId::Potion(PotionId::FirePotion), kind: "potion", key: "P", file: "src/combat/tests/m27_potions.rs", test: "m27_potion_fire_potion_ironclad" },
    LedgerRow { id: "blood_potion", item: ItemId::Potion(PotionId::BloodPotion), kind: "potion", key: "P", file: "src/combat/tests/m27_potions.rs", test: "m27_potion_blood_potion_ironclad" },
    LedgerRow { id: "fruit_juice", item: ItemId::Potion(PotionId::FruitJuice), kind: "potion", key: "P", file: "src/combat/tests/m27_potions.rs", test: "m27_potion_fruit_juice_ironclad" },
    LedgerRow { id: "block_potion", item: ItemId::Potion(PotionId::BlockPotion), kind: "potion", key: "P", file: "src/combat/tests/m27_potions.rs", test: "m27_potion_block_potion_ironclad" },
    LedgerRow { id: "energy_potion", item: ItemId::Potion(PotionId::EnergyPotion), kind: "potion", key: "P", file: "src/combat/tests/m27_potions.rs", test: "m27_potion_energy_potion_ironclad" },
    LedgerRow { id: "strength_potion", item: ItemId::Potion(PotionId::StrengthPotion), kind: "potion", key: "P", file: "src/combat/tests/m27_potions.rs", test: "m27_potion_strength_potion_ironclad" },
    LedgerRow { id: "swift_potion", item: ItemId::Potion(PotionId::SwiftPotion), kind: "potion", key: "P", file: "src/combat/tests/m27_potions.rs", test: "m27_potion_swift_potion_ironclad" },
    LedgerRow { id: "weak_potion", item: ItemId::Potion(PotionId::WeakPotion), kind: "potion", key: "P", file: "src/combat/tests/m27_potions.rs", test: "m27_potion_weak_potion_ironclad" },
    LedgerRow { id: "fear_potion", item: ItemId::Potion(PotionId::FearPotion), kind: "potion", key: "P", file: "src/combat/tests/m27_potions.rs", test: "m27_potion_fear_potion_ironclad" },
    LedgerRow { id: "dexterity_potion", item: ItemId::Potion(PotionId::DexterityPotion), kind: "potion", key: "P", file: "src/combat/tests/m27_potions.rs", test: "m27_potion_dexterity_potion_ironclad" },
    LedgerRow { id: "ancient_potion", item: ItemId::Potion(PotionId::AncientPotion), kind: "potion", key: "P", file: "src/combat/tests/m27_potions.rs", test: "m27_potion_ancient_potion_ironclad" },
    LedgerRow { id: "elixir", item: ItemId::Potion(PotionId::Elixir), kind: "potion", key: "P", file: "src/combat/tests/m27_potions.rs", test: "m27_potion_elixir_ironclad" },
    LedgerRow { id: "heart_of_iron", item: ItemId::Potion(PotionId::HeartOfIron), kind: "potion", key: "P", file: "src/combat/tests/m27_potions.rs", test: "m27_potion_heart_of_iron_ironclad" },
    LedgerRow { id: "poison_potion", item: ItemId::Potion(PotionId::PoisonPotion), kind: "potion", key: "P", file: "src/combat/tests/m27_potions.rs", test: "m27_potion_poison_potion_silent" },
    LedgerRow { id: "cunning_potion", item: ItemId::Potion(PotionId::CunningPotion), kind: "potion", key: "P", file: "src/combat/tests/m27_potions.rs", test: "m27_potion_cunning_potion_silent" },
    LedgerRow { id: "ghost_in_a_jar", item: ItemId::Potion(PotionId::GhostInAJar), kind: "potion", key: "P", file: "src/combat/tests/m27_potions.rs", test: "m27_potion_ghost_in_a_jar_silent" },
    LedgerRow { id: "flex_potion", item: ItemId::Potion(PotionId::FlexPotion), kind: "potion", key: "P", file: "src/combat/tests/m27_potions.rs", test: "m27_potion_flex_potion_ironclad" },
    LedgerRow { id: "speed_potion", item: ItemId::Potion(PotionId::SpeedPotion), kind: "potion", key: "P", file: "src/combat/tests/m27_potions.rs", test: "m27_potion_speed_potion_ironclad" },
    LedgerRow { id: "regen_potion", item: ItemId::Potion(PotionId::RegenPotion), kind: "potion", key: "P", file: "src/combat/tests/m27_potions.rs", test: "m27_potion_regen_potion_ironclad" },
    LedgerRow { id: "liquid_bronze", item: ItemId::Potion(PotionId::LiquidBronze), kind: "potion", key: "P", file: "src/combat/tests/m27_potions.rs", test: "m27_potion_liquid_bronze_ironclad" },
    LedgerRow { id: "essence_of_steel", item: ItemId::Potion(PotionId::EssenceOfSteel), kind: "potion", key: "P", file: "src/combat/tests/m27_potions.rs", test: "m27_potion_essence_of_steel_ironclad" },
    LedgerRow { id: "cultist_potion", item: ItemId::Potion(PotionId::CultistPotion), kind: "potion", key: "P", file: "src/combat/tests/m27_potions.rs", test: "m27_potion_cultist_potion_ironclad" },
    LedgerRow { id: "blessing_of_the_forge", item: ItemId::Potion(PotionId::BlessingOfTheForge), kind: "potion", key: "P", file: "src/combat/tests/m27_potions.rs", test: "m27_potion_blessing_of_the_forge_ironclad" },
    LedgerRow { id: "snecko_oil", item: ItemId::Potion(PotionId::SneckoOil), kind: "potion", key: "P", file: "src/combat/tests/m27_potions.rs", test: "m27_potion_snecko_oil_ironclad" },
    LedgerRow { id: "attack_potion", item: ItemId::Potion(PotionId::AttackPotion), kind: "potion", key: "P", file: "src/combat/tests/m27_potions.rs", test: "m27_potion_attack_potion_ironclad" },
    LedgerRow { id: "skill_potion", item: ItemId::Potion(PotionId::SkillPotion), kind: "potion", key: "P", file: "src/combat/tests/m27_potions.rs", test: "m27_potion_skill_potion_ironclad" },
    LedgerRow { id: "power_potion", item: ItemId::Potion(PotionId::PowerPotion), kind: "potion", key: "P", file: "src/combat/tests/m27_potions.rs", test: "m27_potion_power_potion_ironclad" },
    LedgerRow { id: "colorless_potion", item: ItemId::Potion(PotionId::ColorlessPotion), kind: "potion", key: "P", file: "src/combat/tests/m27_potions.rs", test: "m27_potion_colorless_potion_ironclad" },
    LedgerRow { id: "smoke_bomb", item: ItemId::Potion(PotionId::SmokeBomb), kind: "potion", key: "P", file: "src/combat/tests/m27_potions.rs", test: "m27_potion_smoke_bomb_ironclad" },
    LedgerRow { id: "fairy_potion", item: ItemId::Potion(PotionId::FairyPotion), kind: "potion", key: "P", file: "src/combat/tests/m27_potions.rs", test: "m27_potion_fairy_potion_ironclad" },
    LedgerRow { id: "explosive_potion", item: ItemId::Potion(PotionId::ExplosivePotion), kind: "potion", key: "P", file: "src/combat/tests/m27_potions.rs", test: "m27_potion_explosive_potion_ironclad" },
    LedgerRow { id: "gamblers_brew", item: ItemId::Potion(PotionId::GamblersBrew), kind: "potion", key: "P", file: "src/combat/tests/m27_potions.rs", test: "m27_potion_gamblers_brew_ironclad" },
    LedgerRow { id: "liquid_memories", item: ItemId::Potion(PotionId::LiquidMemories), kind: "potion", key: "P", file: "src/combat/tests/m27_potions.rs", test: "m27_potion_liquid_memories_ironclad" },
    LedgerRow { id: "duplication_potion", item: ItemId::Potion(PotionId::DuplicationPotion), kind: "potion", key: "P", file: "src/combat/tests/m27_potions.rs", test: "m27_potion_duplication_potion_ironclad" },
    LedgerRow { id: "distilled_chaos", item: ItemId::Potion(PotionId::DistilledChaos), kind: "potion", key: "P", file: "src/combat/tests/m27_potions.rs", test: "m27_potion_distilled_chaos_ironclad" },
    LedgerRow { id: "entropic_brew", item: ItemId::Potion(PotionId::EntropicBrew), kind: "potion", key: "P", file: "src/combat/tests/m27_potions.rs", test: "m27_potion_entropic_brew_ironclad" },
    LedgerRow { id: "focus_potion", item: ItemId::Potion(PotionId::FocusPotion), kind: "potion", key: "P", file: "src/combat/tests/m27_potions.rs", test: "m27_potion_focus_potion_defect" },
    LedgerRow { id: "potion_of_capacity", item: ItemId::Potion(PotionId::PotionOfCapacity), kind: "potion", key: "P", file: "src/combat/tests/m27_potions.rs", test: "m27_potion_potion_of_capacity_defect" },
    LedgerRow { id: "essence_of_darkness", item: ItemId::Potion(PotionId::EssenceOfDarkness), kind: "potion", key: "P", file: "src/combat/tests/m27_potions.rs", test: "m27_potion_essence_of_darkness_defect" },
    LedgerRow { id: "stance_potion", item: ItemId::Potion(PotionId::StancePotion), kind: "potion", key: "P", file: "src/combat/tests/m27_potions.rs", test: "m27_potion_stance_potion_watcher" },
    LedgerRow { id: "ambrosia", item: ItemId::Potion(PotionId::Ambrosia), kind: "potion", key: "P", file: "src/combat/tests/m27_potions.rs", test: "m27_potion_ambrosia_watcher" },
    LedgerRow { id: "bottled_miracle", item: ItemId::Potion(PotionId::BottledMiracle), kind: "potion", key: "P", file: "src/combat/tests/m27_potions.rs", test: "m27_potion_bottled_miracle_watcher" },
];

const TEST_FILES: &[(&str, &str)] = &[
    ("src/items/tests/m27_existing_relics.rs", include_str!("m27_existing_relics.rs")),
    ("src/items/tests/m27_relics_structural.rs", include_str!("m27_relics_structural.rs")),
    ("src/items/tests/m27_energy_law.rs", include_str!("m27_energy_law.rs")),
    ("src/items/tests/m27_relics_turns.rs", include_str!("m27_relics_turns.rs")),
    ("src/run/tests/m27_relics_run.rs", include_str!("../../run/tests/m27_relics_run.rs")),
    ("src/items/tests/m27_relics_play.rs", include_str!("m27_relics_play.rs")),
    ("src/run/tests/m27_relic_selections.rs", include_str!("../../run/tests/m27_relic_selections.rs")),
    ("src/items/tests/m27_capability.rs", include_str!("m27_capability.rs")),
    ("src/combat/tests/m27_rescue_escape.rs", include_str!("../../combat/tests/m27_rescue_escape.rs")),
    ("src/run/tests/m27_potion_rewards.rs", include_str!("../../run/tests/m27_potion_rewards.rs")),
    ("src/combat/tests/m27_potions.rs", include_str!("../../combat/tests/m27_potions.rs")),
    ("src/run/tests/m27_relic_lifecycle.rs", include_str!("../../run/tests/m27_relic_lifecycle.rs")),
    ("src/run/tests/m27_potion_lifecycle.rs", include_str!("../../run/tests/m27_potion_lifecycle.rs")),
    ("src/items/tests/m27_potion_owner.rs", include_str!("m27_potion_owner.rs")),
    ("src/items/tests/m27_cross_character.rs", include_str!("m27_cross_character.rs")),
    ("src/items/tests/m27_inert.rs", include_str!("m27_inert.rs")),
    ("src/run/tests/m27_supplied.rs", include_str!("../../run/tests/m27_supplied.rs")),
];

const SOURCE_FILES: &[(&str, &str)] = &[
    ("combat/card_offers.rs", include_str!("../../combat/card_offers.rs")),
    ("combat/card_state.rs", include_str!("../../combat/card_state.rs")),
    ("combat/cards/colorless.rs", include_str!("../../combat/cards/colorless.rs")),
    ("combat/cards/colorless/choices.rs", include_str!("../../combat/cards/colorless/choices.rs")),
    ("combat/cards/colorless/direct.rs", include_str!("../../combat/cards/colorless/direct.rs")),
    ("combat/cards/colorless/powers.rs", include_str!("../../combat/cards/colorless/powers.rs")),
    ("combat/cards/curses.rs", include_str!("../../combat/cards/curses.rs")),
    ("combat/cards/defect.rs", include_str!("../../combat/cards/defect.rs")),
    ("combat/cards/defect/direct.rs", include_str!("../../combat/cards/defect/direct.rs")),
    ("combat/cards/defect/powers.rs", include_str!("../../combat/cards/defect/powers.rs")),
    ("combat/cards/defect/state.rs", include_str!("../../combat/cards/defect/state.rs")),
    ("combat/cards/ironclad.rs", include_str!("../../combat/cards/ironclad.rs")),
    ("combat/cards/ironclad/autoplay.rs", include_str!("../../combat/cards/ironclad/autoplay.rs")),
    ("combat/cards/ironclad/choices.rs", include_str!("../../combat/cards/ironclad/choices.rs")),
    ("combat/cards/ironclad/copies.rs", include_str!("../../combat/cards/ironclad/copies.rs")),
    ("combat/cards/ironclad/direct.rs", include_str!("../../combat/cards/ironclad/direct.rs")),
    ("combat/cards/ironclad/exhaust.rs", include_str!("../../combat/cards/ironclad/exhaust.rs")),
    ("combat/cards/ironclad/fatal.rs", include_str!("../../combat/cards/ironclad/fatal.rs")),
    ("combat/cards/ironclad/growth.rs", include_str!("../../combat/cards/ironclad/growth.rs")),
    ("combat/cards/ironclad/powers.rs", include_str!("../../combat/cards/ironclad/powers.rs")),
    ("combat/cards/ironclad/reactive.rs", include_str!("../../combat/cards/ironclad/reactive.rs")),
    ("combat/cards/ironclad/rules.rs", include_str!("../../combat/cards/ironclad/rules.rs")),
    ("combat/cards/ironclad/state.rs", include_str!("../../combat/cards/ironclad/state.rs")),
    ("combat/cards/ironclad/support.rs", include_str!("../../combat/cards/ironclad/support.rs")),
    ("combat/cards/ironclad/upgrade.rs", include_str!("../../combat/cards/ironclad/upgrade.rs")),
    ("combat/cards/mod.rs", include_str!("../../combat/cards/mod.rs")),
    ("combat/cards/shared.rs", include_str!("../../combat/cards/shared.rs")),
    ("combat/cards/silent.rs", include_str!("../../combat/cards/silent.rs")),
    ("combat/cards/silent/copy.rs", include_str!("../../combat/cards/silent/copy.rs")),
    ("combat/cards/silent/costs.rs", include_str!("../../combat/cards/silent/costs.rs")),
    ("combat/cards/silent/direct.rs", include_str!("../../combat/cards/silent/direct.rs")),
    ("combat/cards/silent/discard.rs", include_str!("../../combat/cards/silent/discard.rs")),
    ("combat/cards/silent/history.rs", include_str!("../../combat/cards/silent/history.rs")),
    ("combat/cards/silent/poison.rs", include_str!("../../combat/cards/silent/poison.rs")),
    ("combat/cards/silent/powers.rs", include_str!("../../combat/cards/silent/powers.rs")),
    ("combat/cards/silent/retain.rs", include_str!("../../combat/cards/silent/retain.rs")),
    ("combat/cards/silent/shiv.rs", include_str!("../../combat/cards/silent/shiv.rs")),
    ("combat/cards/silent/state.rs", include_str!("../../combat/cards/silent/state.rs")),
    ("combat/cards/silent/x_repeat.rs", include_str!("../../combat/cards/silent/x_repeat.rs")),
    ("combat/cards/watcher.rs", include_str!("../../combat/cards/watcher.rs")),
    ("combat/cards/watcher/m25.rs", include_str!("../../combat/cards/watcher/m25.rs")),
    ("combat/cards/watcher/m25_direct.rs", include_str!("../../combat/cards/watcher/m25_direct.rs")),
    ("combat/cards/watcher/m25_foreign.rs", include_str!("../../combat/cards/watcher/m25_foreign.rs")),
    ("combat/cards/watcher/m25_generation.rs", include_str!("../../combat/cards/watcher/m25_generation.rs")),
    ("combat/cards/watcher/m25_powers.rs", include_str!("../../combat/cards/watcher/m25_powers.rs")),
    ("combat/cards/watcher/m25_state.rs", include_str!("../../combat/cards/watcher/m25_state.rs")),
    ("combat/characters.rs", include_str!("../../combat/characters.rs")),
    ("combat/command.rs", include_str!("../../combat/command.rs")),
    ("combat/content.rs", include_str!("../../combat/content.rs")),
    ("combat/effects.rs", include_str!("../../combat/effects.rs")),
    ("combat/enemies/act1.rs", include_str!("../../combat/enemies/act1.rs")),
    ("combat/enemies/act1/thieves.rs", include_str!("../../combat/enemies/act1/thieves.rs")),
    ("combat/enemies/act2.rs", include_str!("../../combat/enemies/act2.rs")),
    ("combat/enemies/act2/automaton.rs", include_str!("../../combat/enemies/act2/automaton.rs")),
    ("combat/enemies/act2/book.rs", include_str!("../../combat/enemies/act2/book.rs")),
    ("combat/enemies/act2/champ.rs", include_str!("../../combat/enemies/act2/champ.rs")),
    ("combat/enemies/act2/chosen_snecko.rs", include_str!("../../combat/enemies/act2/chosen_snecko.rs")),
    ("combat/enemies/act2/collector.rs", include_str!("../../combat/enemies/act2/collector.rs")),
    ("combat/enemies/act2/flight.rs", include_str!("../../combat/enemies/act2/flight.rs")),
    ("combat/enemies/act2/gremlin_leader.rs", include_str!("../../combat/enemies/act2/gremlin_leader.rs")),
    ("combat/enemies/act2/ordinary_closure.rs", include_str!("../../combat/enemies/act2/ordinary_closure.rs")),
    ("combat/enemies/act2/partners.rs", include_str!("../../combat/enemies/act2/partners.rs")),
    ("combat/enemies/act2/slavers.rs", include_str!("../../combat/enemies/act2/slavers.rs")),
    ("combat/enemies/act2/spawn.rs", include_str!("../../combat/enemies/act2/spawn.rs")),
    ("combat/enemies/act2/stasis.rs", include_str!("../../combat/enemies/act2/stasis.rs")),
    ("combat/enemies/act3.rs", include_str!("../../combat/enemies/act3.rs")),
    ("combat/enemies/act3/awakened.rs", include_str!("../../combat/enemies/act3/awakened.rs")),
    ("combat/enemies/act3/darklings.rs", include_str!("../../combat/enemies/act3/darklings.rs")),
    ("combat/enemies/act3/elites.rs", include_str!("../../combat/enemies/act3/elites.rs")),
    ("combat/enemies/act3/ordinary.rs", include_str!("../../combat/enemies/act3/ordinary.rs")),
    ("combat/enemies/act3/partners.rs", include_str!("../../combat/enemies/act3/partners.rs")),
    ("combat/enemies/act3/reptomancer.rs", include_str!("../../combat/enemies/act3/reptomancer.rs")),
    ("combat/enemies/act3/shapes.rs", include_str!("../../combat/enemies/act3/shapes.rs")),
    ("combat/enemies/act3/time_eater.rs", include_str!("../../combat/enemies/act3/time_eater.rs")),
    ("combat/enemies/act3/writhing.rs", include_str!("../../combat/enemies/act3/writhing.rs")),
    ("combat/enemies/act4.rs", include_str!("../../combat/enemies/act4.rs")),
    ("combat/enemies/act4/heart.rs", include_str!("../../combat/enemies/act4/heart.rs")),
    ("combat/enemies/act4/shield_spear.rs", include_str!("../../combat/enemies/act4/shield_spear.rs")),
    ("combat/enemies/elites.rs", include_str!("../../combat/enemies/elites.rs")),
    ("combat/enemies/guardian.rs", include_str!("../../combat/enemies/guardian.rs")),
    ("combat/enemies/hexaghost.rs", include_str!("../../combat/enemies/hexaghost.rs")),
    ("combat/enemies/mod.rs", include_str!("../../combat/enemies/mod.rs")),
    ("combat/engine.rs", include_str!("../../combat/engine.rs")),
    ("combat/exhaust.rs", include_str!("../../combat/exhaust.rs")),
    ("combat/generation.rs", include_str!("../../combat/generation.rs")),
    ("combat/hooks.rs", include_str!("../../combat/hooks.rs")),
    ("combat/m22_diagnostics.rs", include_str!("../../combat/m22_diagnostics.rs")),
    ("combat/mod.rs", include_str!("../../combat/mod.rs")),
    ("combat/model.rs", include_str!("../../combat/model.rs")),
    ("combat/observe.rs", include_str!("../../combat/observe.rs")),
    ("combat/piles.rs", include_str!("../../combat/piles.rs")),
    ("combat/powers.rs", include_str!("../../combat/powers.rs")),
    ("combat/rng.rs", include_str!("../../combat/rng.rs")),
    ("combat/rules.rs", include_str!("../../combat/rules.rs")),
    ("combat/selection.rs", include_str!("../../combat/selection.rs")),
    ("combat/smoke.rs", include_str!("../../combat/smoke.rs")),
    ("combat/snapshot.rs", include_str!("../../combat/snapshot.rs")),
    ("combat/status.rs", include_str!("../../combat/status.rs")),
    ("combat/tests/actions_views.rs", include_str!("../../combat/tests/actions_views.rs")),
    ("combat/tests/differential.rs", include_str!("../../combat/tests/differential.rs")),
    ("combat/tests/differential_decode.rs", include_str!("../../combat/tests/differential_decode.rs")),
    ("combat/tests/encounter_ordinary.rs", include_str!("../../combat/tests/encounter_ordinary.rs")),
    ("combat/tests/encounter_thieves.rs", include_str!("../../combat/tests/encounter_thieves.rs")),
    ("combat/tests/entry_replay.rs", include_str!("../../combat/tests/entry_replay.rs")),
    ("combat/tests/exhaust.rs", include_str!("../../combat/tests/exhaust.rs")),
    ("combat/tests/full_cards.rs", include_str!("../../combat/tests/full_cards.rs")),
    ("combat/tests/full_enemies.rs", include_str!("../../combat/tests/full_enemies.rs")),
    ("combat/tests/full_runtime.rs", include_str!("../../combat/tests/full_runtime.rs")),
    ("combat/tests/m10_autoplay.rs", include_str!("../../combat/tests/m10_autoplay.rs")),
    ("combat/tests/m10_copy.rs", include_str!("../../combat/tests/m10_copy.rs")),
    ("combat/tests/m10_direct.rs", include_str!("../../combat/tests/m10_direct.rs")),
    ("combat/tests/m10_exhaust_actions.rs", include_str!("../../combat/tests/m10_exhaust_actions.rs")),
    ("combat/tests/m10_exhaust_hooks.rs", include_str!("../../combat/tests/m10_exhaust_hooks.rs")),
    ("combat/tests/m10_fatal.rs", include_str!("../../combat/tests/m10_fatal.rs")),
    ("combat/tests/m10_generation.rs", include_str!("../../combat/tests/m10_generation.rs")),
    ("combat/tests/m10_growth.rs", include_str!("../../combat/tests/m10_growth.rs")),
    ("combat/tests/m10_heart.rs", include_str!("../../combat/tests/m10_heart.rs")),
    ("combat/tests/m10_ironclad_potions.rs", include_str!("../../combat/tests/m10_ironclad_potions.rs")),
    ("combat/tests/m10_ironclad_relics.rs", include_str!("../../combat/tests/m10_ironclad_relics.rs")),
    ("combat/tests/m10_power_turns.rs", include_str!("../../combat/tests/m10_power_turns.rs")),
    ("combat/tests/m10_reactive.rs", include_str!("../../combat/tests/m10_reactive.rs")),
    ("combat/tests/m10_selection.rs", include_str!("../../combat/tests/m10_selection.rs")),
    ("combat/tests/m10_shield_spear.rs", include_str!("../../combat/tests/m10_shield_spear.rs")),
    ("combat/tests/m10_support.rs", include_str!("../../combat/tests/m10_support.rs")),
    ("combat/tests/m10_upgrade.rs", include_str!("../../combat/tests/m10_upgrade.rs")),
    ("combat/tests/m11_silent_copy.rs", include_str!("../../combat/tests/m11_silent_copy.rs")),
    ("combat/tests/m11_silent_discard.rs", include_str!("../../combat/tests/m11_silent_discard.rs")),
    ("combat/tests/m11_silent_poison.rs", include_str!("../../combat/tests/m11_silent_poison.rs")),
    ("combat/tests/m11_silent_retain.rs", include_str!("../../combat/tests/m11_silent_retain.rs")),
    ("combat/tests/m11_silent_shiv.rs", include_str!("../../combat/tests/m11_silent_shiv.rs")),
    ("combat/tests/m12_silent_values.rs", include_str!("../../combat/tests/m12_silent_values.rs")),
    ("combat/tests/m13_silent_interactions.rs", include_str!("../../combat/tests/m13_silent_interactions.rs")),
    ("combat/tests/m14_silent_interactions.rs", include_str!("../../combat/tests/m14_silent_interactions.rs")),
    ("combat/tests/m15_act4_interactions.rs", include_str!("../../combat/tests/m15_act4_interactions.rs")),
    ("combat/tests/m15_card_offers.rs", include_str!("../../combat/tests/m15_card_offers.rs")),
    ("combat/tests/m15_colorless_choices.rs", include_str!("../../combat/tests/m15_colorless_choices.rs")),
    ("combat/tests/m15_colorless_direct.rs", include_str!("../../combat/tests/m15_colorless_direct.rs")),
    ("combat/tests/m15_colorless_powers.rs", include_str!("../../combat/tests/m15_colorless_powers.rs")),
    ("combat/tests/m15_generation.rs", include_str!("../../combat/tests/m15_generation.rs")),
    ("combat/tests/m15_potion_actions.rs", include_str!("../../combat/tests/m15_potion_actions.rs")),
    ("combat/tests/m15_potion_generation.rs", include_str!("../../combat/tests/m15_potion_generation.rs")),
    ("combat/tests/m15_silent_cost_copy.rs", include_str!("../../combat/tests/m15_silent_cost_copy.rs")),
    ("combat/tests/m15_silent_direct.rs", include_str!("../../combat/tests/m15_silent_direct.rs")),
    ("combat/tests/m15_silent_history.rs", include_str!("../../combat/tests/m15_silent_history.rs")),
    ("combat/tests/m15_silent_items.rs", include_str!("../../combat/tests/m15_silent_items.rs")),
    ("combat/tests/m15_silent_poison_death.rs", include_str!("../../combat/tests/m15_silent_poison_death.rs")),
    ("combat/tests/m15_silent_powers.rs", include_str!("../../combat/tests/m15_silent_powers.rs")),
    ("combat/tests/m15_silent_x_repeat.rs", include_str!("../../combat/tests/m15_silent_x_repeat.rs")),
    ("combat/tests/m15_smoke.rs", include_str!("../../combat/tests/m15_smoke.rs")),
    ("combat/tests/m16_content.rs", include_str!("../../combat/tests/m16_content.rs")),
    ("combat/tests/m16_mutations.rs", include_str!("../../combat/tests/m16_mutations.rs")),
    ("combat/tests/m16_orb_interactions.rs", include_str!("../../combat/tests/m16_orb_interactions.rs")),
    ("combat/tests/m16_orbs.rs", include_str!("../../combat/tests/m16_orbs.rs")),
    ("combat/tests/m16_power_callbacks.rs", include_str!("../../combat/tests/m16_power_callbacks.rs")),
    ("combat/tests/m16_review_corrections.rs", include_str!("../../combat/tests/m16_review_corrections.rs")),
    ("combat/tests/m16_support.rs", include_str!("../../combat/tests/m16_support.rs")),
    ("combat/tests/m18_custody_support.rs", include_str!("../../combat/tests/m18_custody_support.rs")),
    ("combat/tests/m18_defect_interactions.rs", include_str!("../../combat/tests/m18_defect_interactions.rs")),
    ("combat/tests/m19_defect_interactions.rs", include_str!("../../combat/tests/m19_defect_interactions.rs")),
    ("combat/tests/m20_defect_direct.rs", include_str!("../../combat/tests/m20_defect_direct.rs")),
    ("combat/tests/m20_defect_generation.rs", include_str!("../../combat/tests/m20_defect_generation.rs")),
    ("combat/tests/m20_defect_interactions.rs", include_str!("../../combat/tests/m20_defect_interactions.rs")),
    ("combat/tests/m20_defect_powers.rs", include_str!("../../combat/tests/m20_defect_powers.rs")),
    ("combat/tests/m20_defect_state.rs", include_str!("../../combat/tests/m20_defect_state.rs")),
    ("combat/tests/m21_watcher_creation.rs", include_str!("../../combat/tests/m21_watcher_creation.rs")),
    ("combat/tests/m21_watcher_retain.rs", include_str!("../../combat/tests/m21_watcher_retain.rs")),
    ("combat/tests/m21_watcher_review.rs", include_str!("../../combat/tests/m21_watcher_review.rs")),
    ("combat/tests/m21_watcher_scry.rs", include_str!("../../combat/tests/m21_watcher_scry.rs")),
    ("combat/tests/m21_watcher_stances.rs", include_str!("../../combat/tests/m21_watcher_stances.rs")),
    ("combat/tests/m22_watcher_progression.rs", include_str!("../../combat/tests/m22_watcher_progression.rs")),
    ("combat/tests/m23_confused_cost.rs", include_str!("../../combat/tests/m23_confused_cost.rs")),
    ("combat/tests/m23_cost_law.rs", include_str!("../../combat/tests/m23_cost_law.rs")),
    ("combat/tests/m23_watcher_act2.rs", include_str!("../../combat/tests/m23_watcher_act2.rs")),
    ("combat/tests/m24_time_warp_barrier.rs", include_str!("../../combat/tests/m24_time_warp_barrier.rs")),
    ("combat/tests/m24_watcher_act3.rs", include_str!("../../combat/tests/m24_watcher_act3.rs")),
    ("combat/tests/m25_burn_retain_all_characters.rs", include_str!("../../combat/tests/m25_burn_retain_all_characters.rs")),
    ("combat/tests/m25_watcher_direct.rs", include_str!("../../combat/tests/m25_watcher_direct.rs")),
    ("combat/tests/m25_watcher_foreign_influence.rs", include_str!("../../combat/tests/m25_watcher_foreign_influence.rs")),
    ("combat/tests/m25_watcher_generated_construction.rs", include_str!("../../combat/tests/m25_watcher_generated_construction.rs")),
    ("combat/tests/m25_watcher_generation.rs", include_str!("../../combat/tests/m25_watcher_generation.rs")),
    ("combat/tests/m25_watcher_interactions.rs", include_str!("../../combat/tests/m25_watcher_interactions.rs")),
    ("combat/tests/m25_watcher_interactions_w.rs", include_str!("../../combat/tests/m25_watcher_interactions_w.rs")),
    ("combat/tests/m25_watcher_nested_autoplay.rs", include_str!("../../combat/tests/m25_watcher_nested_autoplay.rs")),
    ("combat/tests/m25_watcher_powers.rs", include_str!("../../combat/tests/m25_watcher_powers.rs")),
    ("combat/tests/m25_watcher_selection_during_play.rs", include_str!("../../combat/tests/m25_watcher_selection_during_play.rs")),
    ("combat/tests/m25_watcher_state.rs", include_str!("../../combat/tests/m25_watcher_state.rs")),
    ("combat/tests/m25_watcher_unplayable_in_hand.rs", include_str!("../../combat/tests/m25_watcher_unplayable_in_hand.rs")),
    ("combat/tests/m26_autoplay.rs", include_str!("../../combat/tests/m26_autoplay.rs")),
    ("combat/tests/m26_capability.rs", include_str!("../../combat/tests/m26_capability.rs")),
    ("combat/tests/m26_cards.rs", include_str!("../../combat/tests/m26_cards.rs")),
    ("combat/tests/m26_copies.rs", include_str!("../../combat/tests/m26_copies.rs")),
    ("combat/tests/m26_corrections.rs", include_str!("../../combat/tests/m26_corrections.rs")),
    ("combat/tests/m26_cross_character.rs", include_str!("../../combat/tests/m26_cross_character.rs")),
    ("combat/tests/m26_generation.rs", include_str!("../../combat/tests/m26_generation.rs")),
    ("combat/tests/m26_offcolour_laws.rs", include_str!("../../combat/tests/m26_offcolour_laws.rs")),
    ("combat/tests/m26_pools.rs", include_str!("../../combat/tests/m26_pools.rs")),
    ("combat/tests/m26_support.rs", include_str!("../../combat/tests/m26_support.rs")),
    ("combat/tests/m27_engine_2.rs", include_str!("../../combat/tests/m27_engine_2.rs")),
    ("combat/tests/m27_engine_3.rs", include_str!("../../combat/tests/m27_engine_3.rs")),
    ("combat/tests/m27_potions.rs", include_str!("../../combat/tests/m27_potions.rs")),
    ("combat/tests/m27_rescue_escape.rs", include_str!("../../combat/tests/m27_rescue_escape.rs")),
    ("combat/tests/m5_bosses.rs", include_str!("../../combat/tests/m5_bosses.rs")),
    ("combat/tests/m5_elites.rs", include_str!("../../combat/tests/m5_elites.rs")),
    ("combat/tests/m5_status.rs", include_str!("../../combat/tests/m5_status.rs")),
    ("combat/tests/m6_bosses.rs", include_str!("../../combat/tests/m6_bosses.rs")),
    ("combat/tests/m6_elites.rs", include_str!("../../combat/tests/m6_elites.rs")),
    ("combat/tests/m6_ordinary.rs", include_str!("../../combat/tests/m6_ordinary.rs")),
    ("combat/tests/m6_partners.rs", include_str!("../../combat/tests/m6_partners.rs")),
    ("combat/tests/m6_pressure.rs", include_str!("../../combat/tests/m6_pressure.rs")),
    ("combat/tests/m6_stasis.rs", include_str!("../../combat/tests/m6_stasis.rs")),
    ("combat/tests/m6_summons.rs", include_str!("../../combat/tests/m6_summons.rs")),
    ("combat/tests/m8_awakened.rs", include_str!("../../combat/tests/m8_awakened.rs")),
    ("combat/tests/m8_darklings.rs", include_str!("../../combat/tests/m8_darklings.rs")),
    ("combat/tests/m8_elites.rs", include_str!("../../combat/tests/m8_elites.rs")),
    ("combat/tests/m8_ordinary.rs", include_str!("../../combat/tests/m8_ordinary.rs")),
    ("combat/tests/m8_partners.rs", include_str!("../../combat/tests/m8_partners.rs")),
    ("combat/tests/m8_reptomancer.rs", include_str!("../../combat/tests/m8_reptomancer.rs")),
    ("combat/tests/m8_shapes.rs", include_str!("../../combat/tests/m8_shapes.rs")),
    ("combat/tests/m8_time_eater.rs", include_str!("../../combat/tests/m8_time_eater.rs")),
    ("combat/tests/m8_writhing.rs", include_str!("../../combat/tests/m8_writhing.rs")),
    ("combat/tests/m9_relics.rs", include_str!("../../combat/tests/m9_relics.rs")),
    ("combat/tests/mod.rs", include_str!("../../combat/tests/mod.rs")),
    ("combat/tests/observe.rs", include_str!("../../combat/tests/observe.rs")),
    ("combat/tests/piles.rs", include_str!("../../combat/tests/piles.rs")),
    ("combat/tests/public_projection.rs", include_str!("../../combat/tests/public_projection.rs")),
    ("combat/tests/resolution.rs", include_str!("../../combat/tests/resolution.rs")),
    ("combat/tests/rng_contract.rs", include_str!("../../combat/tests/rng_contract.rs")),
    ("combat/tests/rules_ai.rs", include_str!("../../combat/tests/rules_ai.rs")),
    ("combat/tests/selection.rs", include_str!("../../combat/tests/selection.rs")),
    ("combat/tests/shared_mechanics.rs", include_str!("../../combat/tests/shared_mechanics.rs")),
    ("combat/tests/support.rs", include_str!("../../combat/tests/support.rs")),
    ("combat/tests/turns.rs", include_str!("../../combat/tests/turns.rs")),
    ("combat/tests/whole_combat.rs", include_str!("../../combat/tests/whole_combat.rs")),
    ("combat/turns.rs", include_str!("../../combat/turns.rs")),
    ("combat/view.rs", include_str!("../../combat/view.rs")),
    ("content/cards.rs", include_str!("../../content/cards.rs")),
    ("content/complete.rs", include_str!("../../content/complete.rs")),
    ("content/defect.rs", include_str!("../../content/defect.rs")),
    ("content/encounters.rs", include_str!("../../content/encounters.rs")),
    ("content/events.rs", include_str!("../../content/events.rs")),
    ("content/ids.rs", include_str!("../../content/ids.rs")),
    ("content/ids/cards.rs", include_str!("../../content/ids/cards.rs")),
    ("content/ids/encounters.rs", include_str!("../../content/ids/encounters.rs")),
    ("content/ids/events.rs", include_str!("../../content/ids/events.rs")),
    ("content/ids/items.rs", include_str!("../../content/ids/items.rs")),
    ("content/items.rs", include_str!("../../content/items.rs")),
    ("content/mod.rs", include_str!("../../content/mod.rs")),
    ("content/pools.rs", include_str!("../../content/pools.rs")),
    ("content/watcher.rs", include_str!("../../content/watcher.rs")),
    ("items/dispatch.rs", include_str!("../../items/dispatch.rs")),
    ("items/ironclad.rs", include_str!("../../items/ironclad.rs")),
    ("items/ironclad_turns.rs", include_str!("../../items/ironclad_turns.rs")),
    ("items/laws.rs", include_str!("../../items/laws.rs")),
    ("items/lifecycle.rs", include_str!("../../items/lifecycle.rs")),
    ("items/m10_ironclad_relic_turns.rs", include_str!("../../items/m10_ironclad_relic_turns.rs")),
    ("items/m15_potion_generation_tests.rs", include_str!("../../items/m15_potion_generation_tests.rs")),
    ("items/m15_silent_items.rs", include_str!("../../items/m15_silent_items.rs")),
    ("items/main_relics.rs", include_str!("../../items/main_relics.rs")),
    ("items/mod.rs", include_str!("../../items/mod.rs")),
    ("items/potion_generation.rs", include_str!("../../items/potion_generation.rs")),
    ("items/potion_offers.rs", include_str!("../../items/potion_offers.rs")),
    ("items/potions.rs", include_str!("../../items/potions.rs")),
    ("items/relic_effects/accepted_law.rs", include_str!("../../items/relic_effects/accepted_law.rs")),
    ("items/relic_effects/mod.rs", include_str!("../../items/relic_effects/mod.rs")),
    ("items/relic_effects/play.rs", include_str!("../../items/relic_effects/play.rs")),
    ("items/relic_effects/run.rs", include_str!("../../items/relic_effects/run.rs")),
    ("items/relic_effects/structural.rs", include_str!("../../items/relic_effects/structural.rs")),
    ("items/relic_effects/turns.rs", include_str!("../../items/relic_effects/turns.rs")),
    ("items/relic_pools.rs", include_str!("../../items/relic_pools.rs")),
    ("items/relics.rs", include_str!("../../items/relics.rs")),
    ("items/run_laws.rs", include_str!("../../items/run_laws.rs")),
    ("items/silent.rs", include_str!("../../items/silent.rs")),
    ("items/tests.rs", include_str!("../../items/tests.rs")),
    ("items/tests/m16_defect.rs", include_str!("../../items/tests/m16_defect.rs")),
    ("items/tests/m20_defect.rs", include_str!("../../items/tests/m20_defect.rs")),
    ("items/tests/m25_watcher.rs", include_str!("../../items/tests/m25_watcher.rs")),
    ("items/tests/m27_capability.rs", include_str!("../../items/tests/m27_capability.rs")),
    ("items/tests/m27_cofiring.rs", include_str!("../../items/tests/m27_cofiring.rs")),
    ("items/tests/m27_cross_character.rs", include_str!("../../items/tests/m27_cross_character.rs")),
    ("items/tests/m27_dispatch.rs", include_str!("../../items/tests/m27_dispatch.rs")),
    ("items/tests/m27_energy_law.rs", include_str!("../../items/tests/m27_energy_law.rs")),
    ("items/tests/m27_existing_relics.rs", include_str!("../../items/tests/m27_existing_relics.rs")),
    ("items/tests/m27_inert.rs", include_str!("../../items/tests/m27_inert.rs")),
    ("items/tests/m27_potion_factory.rs", include_str!("../../items/tests/m27_potion_factory.rs")),
    ("items/tests/m27_potion_owner.rs", include_str!("../../items/tests/m27_potion_owner.rs")),
    ("items/tests/m27_relics_play.rs", include_str!("../../items/tests/m27_relics_play.rs")),
    ("items/tests/m27_relics_structural.rs", include_str!("../../items/tests/m27_relics_structural.rs")),
    ("items/tests/m27_relics_turns.rs", include_str!("../../items/tests/m27_relics_turns.rs")),
    ("items/tests/m27_seams.rs", include_str!("../../items/tests/m27_seams.rs")),
    ("items/tests/m27_support.rs", include_str!("../../items/tests/m27_support.rs")),
    ("lib.rs", include_str!("../../lib.rs")),
    ("run/combat.rs", include_str!("../../run/combat.rs")),
    ("run/decision.rs", include_str!("../../run/decision.rs")),
    ("run/diagnostics/m14_supplied_boss.rs", include_str!("../../run/diagnostics/m14_supplied_boss.rs")),
    ("run/diagnostics/m15_supplied_act4.rs", include_str!("../../run/diagnostics/m15_supplied_act4.rs")),
    ("run/diagnostics/m20.rs", include_str!("../../run/diagnostics/m20.rs")),
    ("run/diagnostics/m22.rs", include_str!("../../run/diagnostics/m22.rs")),
    ("run/diagnostics/m23.rs", include_str!("../../run/diagnostics/m23.rs")),
    ("run/diagnostics/m24.rs", include_str!("../../run/diagnostics/m24.rs")),
    ("run/diagnostics/m25.rs", include_str!("../../run/diagnostics/m25.rs")),
    ("run/diagnostics/m26.rs", include_str!("../../run/diagnostics/m26.rs")),
    ("run/diagnostics/m27.rs", include_str!("../../run/diagnostics/m27.rs")),
    ("run/diagnostics/mod.rs", include_str!("../../run/diagnostics/mod.rs")),
    ("run/engine.rs", include_str!("../../run/engine.rs")),
    ("run/map.rs", include_str!("../../run/map.rs")),
    ("run/mod.rs", include_str!("../../run/mod.rs")),
    ("run/model.rs", include_str!("../../run/model.rs")),
    ("run/mutations.rs", include_str!("../../run/mutations.rs")),
    ("run/observe.rs", include_str!("../../run/observe.rs")),
    ("run/profile.rs", include_str!("../../run/profile.rs")),
    ("run/public.rs", include_str!("../../run/public.rs")),
    ("run/reward.rs", include_str!("../../run/reward.rs")),
    ("run/rng.rs", include_str!("../../run/rng.rs")),
    ("run/rooms/act1.rs", include_str!("../../run/rooms/act1.rs")),
    ("run/rooms/act2.rs", include_str!("../../run/rooms/act2.rs")),
    ("run/rooms/act3.rs", include_str!("../../run/rooms/act3.rs")),
    ("run/rooms/act3_end.rs", include_str!("../../run/rooms/act3_end.rs")),
    ("run/rooms/act4.rs", include_str!("../../run/rooms/act4.rs")),
    ("run/rooms/boss.rs", include_str!("../../run/rooms/boss.rs")),
    ("run/rooms/events.rs", include_str!("../../run/rooms/events.rs")),
    ("run/rooms/events/act3.rs", include_str!("../../run/rooms/events/act3.rs")),
    ("run/rooms/m10_content.rs", include_str!("../../run/rooms/m10_content.rs")),
    ("run/rooms/m12_content.rs", include_str!("../../run/rooms/m12_content.rs")),
    ("run/rooms/m13_content.rs", include_str!("../../run/rooms/m13_content.rs")),
    ("run/rooms/m14_content.rs", include_str!("../../run/rooms/m14_content.rs")),
    ("run/rooms/m15_content.rs", include_str!("../../run/rooms/m15_content.rs")),
    ("run/rooms/m17_content.rs", include_str!("../../run/rooms/m17_content.rs")),
    ("run/rooms/m18_content.rs", include_str!("../../run/rooms/m18_content.rs")),
    ("run/rooms/m19_content.rs", include_str!("../../run/rooms/m19_content.rs")),
    ("run/rooms/m20_content.rs", include_str!("../../run/rooms/m20_content.rs")),
    ("run/rooms/m22_content.rs", include_str!("../../run/rooms/m22_content.rs")),
    ("run/rooms/m23_content.rs", include_str!("../../run/rooms/m23_content.rs")),
    ("run/rooms/m24_content.rs", include_str!("../../run/rooms/m24_content.rs")),
    ("run/rooms/m25_content.rs", include_str!("../../run/rooms/m25_content.rs")),
    ("run/rooms/mod.rs", include_str!("../../run/rooms/mod.rs")),
    ("run/rooms/neow.rs", include_str!("../../run/rooms/neow.rs")),
    ("run/rooms/profile.rs", include_str!("../../run/rooms/profile.rs")),
    ("run/rooms/rest.rs", include_str!("../../run/rooms/rest.rs")),
    ("run/rooms/rewards.rs", include_str!("../../run/rooms/rewards.rs")),
    ("run/rooms/selection.rs", include_str!("../../run/rooms/selection.rs")),
    ("run/rooms/shop.rs", include_str!("../../run/rooms/shop.rs")),
    ("run/rooms/transition.rs", include_str!("../../run/rooms/transition.rs")),
    ("run/tests/combat.rs", include_str!("../../run/tests/combat.rs")),
    ("run/tests/creation.rs", include_str!("../../run/tests/creation.rs")),
    ("run/tests/decisions.rs", include_str!("../../run/tests/decisions.rs")),
    ("run/tests/m10_act4.rs", include_str!("../../run/tests/m10_act4.rs")),
    ("run/tests/m10_content.rs", include_str!("../../run/tests/m10_content.rs")),
    ("run/tests/m10_copy.rs", include_str!("../../run/tests/m10_copy.rs")),
    ("run/tests/m10_exhaust.rs", include_str!("../../run/tests/m10_exhaust.rs")),
    ("run/tests/m10_gains.rs", include_str!("../../run/tests/m10_gains.rs")),
    ("run/tests/m10_growth.rs", include_str!("../../run/tests/m10_growth.rs")),
    ("run/tests/m10_keys.rs", include_str!("../../run/tests/m10_keys.rs")),
    ("run/tests/m10_potions.rs", include_str!("../../run/tests/m10_potions.rs")),
    ("run/tests/m10_settlement.rs", include_str!("../../run/tests/m10_settlement.rs")),
    ("run/tests/m10_upgrade.rs", include_str!("../../run/tests/m10_upgrade.rs")),
    ("run/tests/m11_silent.rs", include_str!("../../run/tests/m11_silent.rs")),
    ("run/tests/m12_silent_act1.rs", include_str!("../../run/tests/m12_silent_act1.rs")),
    ("run/tests/m13_silent_act2.rs", include_str!("../../run/tests/m13_silent_act2.rs")),
    ("run/tests/m14_silent_act3.rs", include_str!("../../run/tests/m14_silent_act3.rs")),
    ("run/tests/m14_supplied_boss.rs", include_str!("../../run/tests/m14_supplied_boss.rs")),
    ("run/tests/m14_supplied_boss/completion.rs", include_str!("../../run/tests/m14_supplied_boss/completion.rs")),
    ("run/tests/m14_supplied_boss/fixture.rs", include_str!("../../run/tests/m14_supplied_boss/fixture.rs")),
    ("run/tests/m14_supplied_boss/fixture_tests.rs", include_str!("../../run/tests/m14_supplied_boss/fixture_tests.rs")),
    ("run/tests/m15_content.rs", include_str!("../../run/tests/m15_content.rs")),
    ("run/tests/m15_silent_heart.rs", include_str!("../../run/tests/m15_silent_heart.rs")),
    ("run/tests/m15_silent_heart/supplied_act4.rs", include_str!("../../run/tests/m15_silent_heart/supplied_act4.rs")),
    ("run/tests/m16_defect_mutations.rs", include_str!("../../run/tests/m16_defect_mutations.rs")),
    ("run/tests/m16_defect_profile.rs", include_str!("../../run/tests/m16_defect_profile.rs")),
    ("run/tests/m17_defect_act1.rs", include_str!("../../run/tests/m17_defect_act1.rs")),
    ("run/tests/m18_defect_act2.rs", include_str!("../../run/tests/m18_defect_act2.rs")),
    ("run/tests/m19_defect_act3.rs", include_str!("../../run/tests/m19_defect_act3.rs")),
    ("run/tests/m20_defect_content.rs", include_str!("../../run/tests/m20_defect_content.rs")),
    ("run/tests/m20_diagnostic_support.rs", include_str!("../../run/tests/m20_diagnostic_support.rs")),
    ("run/tests/m20_supplied.rs", include_str!("../../run/tests/m20_supplied.rs")),
    ("run/tests/m21_watcher.rs", include_str!("../../run/tests/m21_watcher.rs")),
    ("run/tests/m22_watcher_act1.rs", include_str!("../../run/tests/m22_watcher_act1.rs")),
    ("run/tests/m23_watcher_act2.rs", include_str!("../../run/tests/m23_watcher_act2.rs")),
    ("run/tests/m24_watcher_act3.rs", include_str!("../../run/tests/m24_watcher_act3.rs")),
    ("run/tests/m25_watcher_content.rs", include_str!("../../run/tests/m25_watcher_content.rs")),
    ("run/tests/m25_watcher_heart.rs", include_str!("../../run/tests/m25_watcher_heart.rs")),
    ("run/tests/m26_capability.rs", include_str!("../../run/tests/m26_capability.rs")),
    ("run/tests/m26_card_mutations.rs", include_str!("../../run/tests/m26_card_mutations.rs")),
    ("run/tests/m26_pools.rs", include_str!("../../run/tests/m26_pools.rs")),
    ("run/tests/m26_supplied.rs", include_str!("../../run/tests/m26_supplied.rs")),
    ("run/tests/m27_act4_producers.rs", include_str!("../../run/tests/m27_act4_producers.rs")),
    ("run/tests/m27_capability.rs", include_str!("../../run/tests/m27_capability.rs")),
    ("run/tests/m27_potion_lifecycle.rs", include_str!("../../run/tests/m27_potion_lifecycle.rs")),
    ("run/tests/m27_potion_rewards.rs", include_str!("../../run/tests/m27_potion_rewards.rs")),
    ("run/tests/m27_potion_shop.rs", include_str!("../../run/tests/m27_potion_shop.rs")),
    ("run/tests/m27_relic_lifecycle.rs", include_str!("../../run/tests/m27_relic_lifecycle.rs")),
    ("run/tests/m27_relic_offers.rs", include_str!("../../run/tests/m27_relic_offers.rs")),
    ("run/tests/m27_relic_pools.rs", include_str!("../../run/tests/m27_relic_pools.rs")),
    ("run/tests/m27_relic_selections.rs", include_str!("../../run/tests/m27_relic_selections.rs")),
    ("run/tests/m27_relics_run.rs", include_str!("../../run/tests/m27_relics_run.rs")),
    ("run/tests/m27_run_seams.rs", include_str!("../../run/tests/m27_run_seams.rs")),
    ("run/tests/m27_supplied.rs", include_str!("../../run/tests/m27_supplied.rs")),
    ("run/tests/m5_act1.rs", include_str!("../../run/tests/m5_act1.rs")),
    ("run/tests/m5_rewards.rs", include_str!("../../run/tests/m5_rewards.rs")),
    ("run/tests/m5_transition.rs", include_str!("../../run/tests/m5_transition.rs")),
    ("run/tests/m6_events.rs", include_str!("../../run/tests/m6_events.rs")),
    ("run/tests/m6_rooms.rs", include_str!("../../run/tests/m6_rooms.rs")),
    ("run/tests/m7_rewards.rs", include_str!("../../run/tests/m7_rewards.rs")),
    ("run/tests/m7_rooms.rs", include_str!("../../run/tests/m7_rooms.rs")),
    ("run/tests/m7_transition.rs", include_str!("../../run/tests/m7_transition.rs")),
    ("run/tests/m8_acquisition.rs", include_str!("../../run/tests/m8_acquisition.rs")),
    ("run/tests/m8_events.rs", include_str!("../../run/tests/m8_events.rs")),
    ("run/tests/m8_power_census.rs", include_str!("../../run/tests/m8_power_census.rs")),
    ("run/tests/m8_rooms.rs", include_str!("../../run/tests/m8_rooms.rs")),
    ("run/tests/m9_boss.rs", include_str!("../../run/tests/m9_boss.rs")),
    ("run/tests/m9_end.rs", include_str!("../../run/tests/m9_end.rs")),
    ("run/tests/m9_rooms.rs", include_str!("../../run/tests/m9_rooms.rs")),
    ("run/tests/mod.rs", include_str!("../../run/tests/mod.rs")),
    ("run/tests/reward.rs", include_str!("../../run/tests/reward.rs")),
    ("run/tests/rng.rs", include_str!("../../run/tests/rng.rs")),
    ("run/tests/room_rewards.rs", include_str!("../../run/tests/room_rewards.rs")),
    ("run/tests/room_services.rs", include_str!("../../run/tests/room_services.rs")),
    ("run/tests/rooms.rs", include_str!("../../run/tests/rooms.rs")),
    ("run/tests/world.rs", include_str!("../../run/tests/world.rs")),
    ("run/training.rs", include_str!("../../run/training.rs")),
    ("run/world.rs", include_str!("../../run/world.rs")),
    ("trace.rs", include_str!("../../trace.rs")),
    ("items/tests/m27_gate_audit.rs", include_str!("m27_gate_audit.rs")),
];


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum OwnerClass { Owner, Projection, TransientInput }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct OwnerField { file: &'static str, structure: &'static str, field: &'static str, type_name: &'static str, class: OwnerClass }
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct ScannedField { file: String, structure: String, field: String, type_name: String }

/// Literal census of item stores and projections found by the source scan.
const OWNER_FIELDS: &[OwnerField] = &[
    OwnerField { file: "combat/view.rs", structure: "CombatInventoryView", field: "relic_counters", type_name: "Vec<RelicCounterView>", class: OwnerClass::Projection },
    OwnerField { file: "combat/view.rs", structure: "CombatInventoryView", field: "relics", type_name: "Vec<RelicId>", class: OwnerClass::Projection },
    OwnerField { file: "combat/view.rs", structure: "CombatInventoryView", field: "potions", type_name: "Vec<Option<PotionId>>", class: OwnerClass::Projection },
    OwnerField { file: "items/mod.rs", structure: "CombatItemContext", field: "potion_potencies", type_name: "Vec<Option<i32>>", class: OwnerClass::Owner },
    OwnerField { file: "items/mod.rs", structure: "CombatItemContext", field: "counters", type_name: "BTreeMap<RelicId,i32>", class: OwnerClass::Owner },
    OwnerField { file: "items/mod.rs", structure: "CombatItemContext", field: "relics", type_name: "Vec<RelicId>", class: OwnerClass::Owner },
    OwnerField { file: "items/mod.rs", structure: "CombatItemContext", field: "potions", type_name: "Vec<Option<PotionId>>", class: OwnerClass::Owner },
    OwnerField { file: "items/mod.rs", structure: "PotionUseEffects", field: "potions", type_name: "Vec<Option<PotionId>>", class: OwnerClass::TransientInput },
    OwnerField { file: "run/decision.rs", structure: "PublicPlayer", field: "relic_counters", type_name: "Vec<RelicCounterView>", class: OwnerClass::Projection },
    OwnerField { file: "run/decision.rs", structure: "PublicPlayer", field: "relics", type_name: "Vec<RelicId>", class: OwnerClass::Projection },
    OwnerField { file: "run/decision.rs", structure: "PublicPlayer", field: "potions", type_name: "Vec<Option<PotionId>>", class: OwnerClass::Projection },
    OwnerField { file: "run/diagnostics/m27.rs", structure: "CompleteItemSupply", field: "relics", type_name: "Vec<RelicId>", class: OwnerClass::TransientInput },
    OwnerField { file: "run/diagnostics/m27.rs", structure: "CompleteItemSupply", field: "potions", type_name: "Vec<Option<PotionId>>", class: OwnerClass::TransientInput },
    OwnerField { file: "run/diagnostics/m27.rs", structure: "CompleteItemSupply", field: "counters", type_name: "BTreeMap<RelicId,i32>", class: OwnerClass::TransientInput },
    OwnerField { file: "run/diagnostics/m27.rs", structure: "M27RunCapture", field: "relics", type_name: "Vec<RelicId>", class: OwnerClass::Projection },
    OwnerField { file: "run/model.rs", structure: "AuthoredRunStart", field: "relics", type_name: "Vec<RelicId>", class: OwnerClass::TransientInput },
    OwnerField { file: "run/model.rs", structure: "AuthoredRunStart", field: "potions", type_name: "Vec<Option<PotionId>>", class: OwnerClass::TransientInput },
    OwnerField { file: "run/model.rs", structure: "AuthoredRunStart", field: "relic_counters", type_name: "BTreeMap<RelicId,i32>", class: OwnerClass::TransientInput },
    OwnerField { file: "run/model.rs", structure: "RunState", field: "relics", type_name: "Vec<RelicId>", class: OwnerClass::Owner },
    OwnerField { file: "run/model.rs", structure: "RunState", field: "potions", type_name: "Vec<Option<PotionId>>", class: OwnerClass::Owner },
    OwnerField { file: "run/model.rs", structure: "RunState", field: "relic_counters", type_name: "BTreeMap<RelicId,i32>", class: OwnerClass::Owner },
    OwnerField { file: "run/model.rs", structure: "RunState", field: "potion_potencies", type_name: "Vec<Option<(PotionId,i32)>>", class: OwnerClass::Owner },
    OwnerField { file: "run/training.rs", structure: "CombatTrainingEntry", field: "relics", type_name: "Vec<RelicId>", class: OwnerClass::TransientInput },
    OwnerField { file: "run/training.rs", structure: "CombatTrainingEntry", field: "potions", type_name: "Vec<Option<PotionId>>", class: OwnerClass::TransientInput },
];

fn test_source(path: &str) -> &'static str {
    TEST_FILES.iter().find(|(candidate, _)| *candidate == path)
        .map(|(_, text)| *text).unwrap_or_else(|| panic!("test file missing from include_str map: {path}"))
}

fn line_containing(source: &str, needle: &str) -> usize {
    source.lines().position(|line| line.contains(needle)).map_or(0, |line| line + 1)
}

fn macro_definition_body<'a>(source: &'a str, name: &str) -> Option<&'a str> {
    let declaration = format!("macro_rules! {name}");
    let declaration_start = source.find(&declaration)?;
    let open = declaration_start + source[declaration_start..].find('{')?;
    let bytes = source.as_bytes();
    let mut depth = 1usize;
    let mut index = open + 1;
    let mut quoted = false;
    let mut escaped = false;
    let mut line_comment = false;
    let mut block_comment = 0usize;
    while index < bytes.len() {
        if line_comment {
            if bytes[index] == b'\n' { line_comment = false; }
            index += 1;
            continue;
        }
        if block_comment > 0 {
            if bytes.get(index..index + 2) == Some(b"/*") { block_comment += 1; index += 2; }
            else if bytes.get(index..index + 2) == Some(b"*/") { block_comment -= 1; index += 2; }
            else { index += 1; }
            continue;
        }
        if quoted {
            if escaped { escaped = false; }
            else if bytes[index] == b'\\' { escaped = true; }
            else if bytes[index] == b'"' { quoted = false; }
            index += 1;
            continue;
        }
        if bytes.get(index..index + 2) == Some(b"//") { line_comment = true; index += 2; }
        else if bytes.get(index..index + 2) == Some(b"/*") { block_comment = 1; index += 2; }
        else if bytes[index] == b'"' { quoted = true; index += 1; }
        else if bytes[index] == b'{' { depth += 1; index += 1; }
        else if bytes[index] == b'}' {
            depth -= 1;
            if depth == 0 { return Some(&source[open + 1..index]); }
            index += 1;
        } else { index += 1; }
    }
    None
}

fn macro_has_test_function(body: &str) -> bool {
    if !body.contains("#[test]") { return false; }
    let mut cursor = 0;
    while let Some(relative) = body[cursor..].find("fn ") {
        let after_fn = cursor + relative + 3;
        let signature = body[after_fn..].trim_start();
        if let Some(metavariable) = signature.strip_prefix('$') {
            let length = metavariable.chars().take_while(|ch| ch.is_ascii_alphanumeric() || *ch == '_').map(char::len_utf8).sum::<usize>();
            if length > 0 && metavariable[length..].trim_start().starts_with('(') { return true; }
        }
        cursor = after_fn;
    }
    false
}

fn macro_invocation_for_test<'a>(source: &'a str, test: &str) -> Option<&'a str> {
    for (start, _) in source.match_indices(test) {
        if start > 0 && source[..start].chars().next_back().is_some_and(|ch| ch.is_ascii_alphanumeric() || ch == '_') { continue; }
        if source[start + test.len()..].chars().next().is_some_and(|ch| ch.is_ascii_alphanumeric() || ch == '_') { continue; }
        let before = &source[..start];
        let open = before.rfind("!(")?;
        if source[open + 2..start].contains(')') { continue; }
        let prefix = before[..open].trim_end();
        let name_start = prefix.rfind(|ch: char| !ch.is_ascii_alphanumeric() && ch != '_').map_or(0, |index| index + 1);
        if name_start < prefix.len() { return Some(&prefix[name_start..]); }
    }
    None
}

fn macro_generates_test(source: &str, name: &str, visited: &mut BTreeSet<String>) -> bool {
    if !visited.insert(name.to_owned()) { return false; }
    let Some(body) = macro_definition_body(source, name) else { return false; };
    if macro_has_test_function(body) { return true; }
    let mut cursor = 0;
    while let Some(relative) = body[cursor..].find("!(") {
        let open = cursor + relative;
        let prefix = body[..open].trim_end();
        let name_start = prefix.rfind(|ch: char| !ch.is_ascii_alphanumeric() && ch != '_').map_or(0, |index| index + 1);
        if name_start < prefix.len() && macro_generates_test(source, &prefix[name_start..], visited) { return true; }
        cursor = open + 2;
    }
    false
}

fn source_test_form(source: &str, test: &str) -> Option<&'static str> {
    if source.contains(&format!("fn {test}")) { return Some("fn"); }
    let macro_name = macro_invocation_for_test(source, test)?;
    macro_generates_test(source, macro_name, &mut BTreeSet::new()).then_some("macro")
}

fn definition_line(source: &str, item: ItemId) -> Option<(usize, &str)> {
    let prefix = match item { ItemId::Relic(id) => format!("RelicDefinition{{id:RelicId::{id:?}"), ItemId::Potion(id) => format!("PotionDefinition{{id:PotionId::{id:?}") };
    source.lines().enumerate().find_map(|(line, text)| {
        let normalized: String = text.chars().filter(|ch| !ch.is_whitespace()).collect();
        normalized.contains(&prefix).then_some((line + 1, text))
    })
}

#[test]
fn m27_no_inert_items() {
    let characters = [Character::Ironclad, Character::Silent, Character::Defect, Character::Watcher];
    let all_relics: BTreeSet<&str> = characters.iter().flat_map(|character| complete::relic_union(*character))
        .map(RelicId::as_str).collect();
    let all_potions: BTreeSet<&str> = characters.iter().flat_map(|character| complete::potion_union(*character))
        .map(PotionId::as_str).collect();
    let relic_rows: Vec<_> = LEDGER.iter().filter(|row| row.kind == "relic").collect();
    let potion_rows: Vec<_> = LEDGER.iter().filter(|row| row.kind == "potion").collect();
    let expected_relics: BTreeSet<_> = relic_rows.iter().map(|row| row.id).collect();
    let expected_potions: BTreeSet<_> = potion_rows.iter().map(|row| row.id).collect();
    let mut reds = Vec::new();

    if relic_rows.len() != 179 || expected_relics.len() != relic_rows.len() || expected_relics != all_relics {
        let missing: Vec<_> = all_relics.difference(&expected_relics).copied().collect();
        let extra: Vec<_> = expected_relics.difference(&all_relics).copied().collect();
        reds.push(format!("union ledger relics: rows={} unique={} expected=179 actual={} missing={missing:?} extra={extra:?}", relic_rows.len(), expected_relics.len(), all_relics.len()));
    }
    if potion_rows.len() != 42 || expected_potions.len() != potion_rows.len() || expected_potions != all_potions {
        let missing: Vec<_> = all_potions.difference(&expected_potions).copied().collect();
        let extra: Vec<_> = expected_potions.difference(&all_potions).copied().collect();
        reds.push(format!("union ledger potions: rows={} unique={} expected=42 actual={} missing={missing:?} extra={extra:?}", potion_rows.len(), expected_potions.len(), all_potions.len()));
    }

    let accepted_relics = crate::items::relic_effects::ACCEPTED_RELICS;
    let accepted_rows: Vec<_> = LEDGER.iter().filter(|row| row.key == "E").collect();
    let accepted_ids: BTreeSet<_> = accepted_relics.iter().map(|id| id.as_str()).collect();
    let e_row_ids: BTreeSet<_> = accepted_rows.iter().filter_map(|row| match row.item {
        ItemId::Relic(id) => Some(id.as_str()),
        ItemId::Potion(_) => None,
    }).collect();
    if accepted_rows.len() != 62 || e_row_ids.len() != accepted_rows.len()
        || accepted_ids.len() != 62 || e_row_ids != accepted_ids {
        let missing: Vec<_> = accepted_ids.difference(&e_row_ids).copied().collect();
        let extra: Vec<_> = e_row_ids.difference(&accepted_ids).copied().collect();
        reds.push(format!("E accepted relic rows: observed {} rows and {} unique relic IDs vs {} unique ACCEPTED_RELICS; expected exactly 62 E rows equal to ACCEPTED_RELICS; missing={missing:?} extra={extra:?}", accepted_rows.len(), e_row_ids.len(), accepted_ids.len()));
    }

    let definitions = include_str!("../../content/items.rs");
    let catalog = Catalog::standard();
    for row in LEDGER {
        if !matches!(row.key, "K" | "D" | "C1" | "C2" | "C3" | "J" | "E" | "L" | "P" | "H" | "X" | "I" | "S") {
            reds.push(format!("{} {} coverage key {:?}: observed unsupported key; expected one of K,D,C1,C2,C3,J,E,L,P,H,X,I,S", row.kind, row.id, row.key));
        }
        if row.id != row.item.as_str() {
            reds.push(format!("{} literal id mismatch: observed {:?}, expected {:?}", row.id, row.id, row.item.as_str()));
        }
        let source = test_source(row.file);
        if source_test_form(source, row.test).is_none() {
            let line = line_containing(source, row.test);
            reds.push(format!("{} {} {}:{} observed no literal function or #[test]-generating macro for `{}` (candidate token line {line}); expected the named test function in the file", row.kind, row.id, row.file, line, row.test));
        }
        let source_name = row.item.source_name();
        let accepted_member = match row.item {
            ItemId::Relic(id) => accepted_relics.contains(&id),
            ItemId::Potion(_) => false,
        };
        if row.key == "E" {
            if !accepted_member {
                reds.push(format!("{} {} {} expected an E-key relic present in ACCEPTED_RELICS; observed membership={accepted_member}", row.kind, row.id, row.file));
            }
        } else if !source.contains(&source_name) && !source.contains(row.id) {
            let line = line_containing(source, row.test);
            reds.push(format!("{} {} {}:{} observed no `{}` or `{}` item name; expected the effect test file to name this item", row.kind, row.id, row.file, line, source_name, row.id));
        }
        let (line, definition) = match definition_line(definitions, row.item) {
            Some(found) => found,
            None => {
                reds.push(format!("{} {} src/content/items.rs:0 observed no registry definition; expected one executable definition", row.kind, row.id));
                continue;
            }
        };
        let normalized: String = definition.chars().filter(|ch| !ch.is_whitespace()).collect();
        let is_definition_only = normalized.contains("definition_only:true");
        let catalog_definition_only = match row.item {
            ItemId::Relic(id) => catalog.relic(id).is_some_and(|definition| definition.definition_only),
            ItemId::Potion(id) => catalog.potion(id).is_some_and(|definition| definition.definition_only),
        };
        if is_definition_only || catalog_definition_only {
            reds.push(format!("{} {} src/content/items.rs:{line} observed definition_only=true (source={is_definition_only}, catalog={catalog_definition_only}); expected false", row.kind, row.id));
        }
    }
    assert!(reds.is_empty(), "m27_no_inert_items REDs:\n{}", reds.join("\n"));
}

fn normalized_type(raw: &str) -> String {
    raw.chars().filter(|ch| !ch.is_whitespace()).collect::<String>()
        .replace("std::collections::", "").replace("crate::content::", "").replace("crate::items::", "")
}

fn candidate_field_type(type_name: &str) -> bool {
    matches!(type_name, "Vec<RelicId>" | "Vec<Option<PotionId>>" | "Vec<RelicCounterView>"
        | "BTreeMap<RelicId,i32>" | "Vec<Option<i32>>" | "Vec<Option<(PotionId,i32)>>")
}

fn strip_comments(text: &str) -> String {
    let mut output = String::with_capacity(text.len());
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if i + 1 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'/' {
            while i < bytes.len() && bytes[i] != b'\n' { output.push(' '); i += 1; }
        } else if i + 1 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'*' {
            output.push(' '); output.push(' '); i += 2;
            let mut depth = 1usize;
            while i < bytes.len() && depth > 0 {
                if i + 1 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'*' { depth += 1; output.push(' '); output.push(' '); i += 2; }
                else if i + 1 < bytes.len() && bytes[i] == b'*' && bytes[i + 1] == b'/' { depth -= 1; output.push(' '); output.push(' '); i += 2; }
                else { let ch = text[i..].chars().next().unwrap(); output.push_str(&" ".repeat(ch.len_utf8())); i += ch.len_utf8(); }
            }
        } else {
            let ch = text[i..].chars().next().unwrap(); output.push(ch); i += ch.len_utf8();
        }
    }
    output
}

fn close_struct_brace(source: &str, open: usize) -> Option<usize> {
    let clean = strip_comments(source);
    let bytes = clean.as_bytes();
    let mut depth = 0usize;
    let mut i = open;
    while i < bytes.len() {
        if bytes[i] == b'"' {
            i += 1;
            while i < bytes.len() {
                if bytes[i] == b'\\' { i = (i + 2).min(bytes.len()); }
                else if bytes[i] == b'"' { i += 1; break; }
                else { i += 1; }
            }
        } else if bytes[i] == b'{' { depth += 1; i += 1; }
        else if bytes[i] == b'}' { depth -= 1; i += 1; if depth == 0 { return Some(i - 1); } }
        else { i += 1; }
    }
    None
}

fn scanned_fields(path: &str, source: &str) -> Vec<ScannedField> {
    let mut result = Vec::new();
    let mut cursor = 0;
    while let Some(relative) = source[cursor..].find("struct ") {
        let start = cursor + relative;
        let line_start = source[..start].rfind('\n').map_or(0, |index| index + 1);
        let prefix = source[line_start..start].trim();
        let visibility = prefix.is_empty() || prefix.ends_with("pub")
            || prefix.split_whitespace().next_back().is_some_and(|word| word.starts_with("pub("));
        if !visibility { cursor = start + 7; continue; }
        let name_start = start + "struct ".len();
        let name_end = source[name_start..].find(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_')).map_or(source.len(), |offset| name_start + offset);
        if name_end == name_start { cursor = start + 7; continue; }
        let structure = &source[name_start..name_end];
        let remainder = &source[name_end..];
        let Some(open_relative) = remainder.find('{') else { cursor = name_end; continue; };
        if remainder.find(';').is_some_and(|semi| semi < open_relative) { cursor = name_end; continue; }
        let open = name_end + open_relative;
        let Some(close) = close_struct_brace(source, open) else { cursor = open + 1; continue; };
        let body = strip_comments(&source[open + 1..close]);
        let mut segment_start = 0usize;
        let mut angle = 0i32;
        let mut paren = 0i32;
        let mut bracket = 0i32;
        for (index, ch) in body.char_indices() {
            match ch { '<' => angle += 1, '>' => angle = (angle - 1).max(0), '(' => paren += 1, ')' => paren = (paren - 1).max(0), '[' => bracket += 1, ']' => bracket = (bracket - 1).max(0), ',' if angle == 0 && paren == 0 && bracket == 0 => {
                if let Some(field) = scanned_field_segment(path, structure, &body[segment_start..index]) { result.push(field); }
                segment_start = index + ch.len_utf8();
            }, _ => {} }
        }
        if let Some(field) = scanned_field_segment(path, structure, &body[segment_start..]) { result.push(field); }
        cursor = close + 1;
    }
    result
}

fn scanned_field_segment(path: &str, structure: &str, segment: &str) -> Option<ScannedField> {
    let (left, right) = segment.split_once(':')?;
    let field = left.split_whitespace().last()?.trim_matches(|ch: char| !ch.is_ascii_alphanumeric() && ch != '_');
    if !matches!(field, "relics" | "potions" | "relic_counters" | "counters" | "potion_potencies") { return None; }
    let type_name = normalized_type(right.trim().trim_end_matches('}'));
    if !candidate_field_type(&type_name) { return None; }
    Some(ScannedField { file: path.to_owned(), structure: structure.to_owned(), field: field.to_owned(), type_name })
}

#[test]
fn m27_single_item_owners() {
    let expected: BTreeSet<_> = OWNER_FIELDS.iter().map(|row| ScannedField {
        file: row.file.to_owned(), structure: row.structure.to_owned(), field: row.field.to_owned(), type_name: row.type_name.to_owned(),
    }).collect();
    let actual: BTreeSet<_> = SOURCE_FILES.iter().flat_map(|(path, text)| scanned_fields(path, text)).collect();
    let mut reds = Vec::new();
    if OWNER_FIELDS.len() != expected.len() { reds.push(format!("owner table duplicates: rows={} unique={}", OWNER_FIELDS.len(), expected.len())); }
    if actual != expected {
        for field in actual.difference(&expected) { reds.push(format!("unclassified source store {:?}.{}:{} type {} observed; expected one literal owner/projection/transient row", field.file, field.structure, field.field, field.type_name)); }
        for field in expected.difference(&actual) { reds.push(format!("owner table site {:?}.{}:{} type {} not found in source; expected declaration at that site", field.file, field.structure, field.field, field.type_name)); }
    }
    for field in OWNER_FIELDS {
        let allowed = match field.class {
            OwnerClass::Owner => (field.structure == "RunState" && field.file == "run/model.rs")
                || (field.structure == "CombatItemContext" && field.file == "items/mod.rs"),
            OwnerClass::Projection => matches!(field.structure, "CombatInventoryView" | "PublicPlayer" | "M27RunCapture"),
            OwnerClass::TransientInput => matches!(field.structure, "AuthoredRunStart" | "CombatTrainingEntry" | "CompleteItemSupply" | "PotionUseEffects"),
        };
        if !allowed { reds.push(format!("owner classification {}.{}:{} observed {:?}; expected class matches its ownership role", field.file, field.structure, field.field, field.class)); }
    }
    assert!(reds.is_empty(), "m27_single_item_owners structural REDs:\n{}", reds.join("\n"));

    let accepted_counter_view: Vec<crate::items::RelicCounterView> = Vec::new();
    let complete_pen_nib_view = vec![crate::items::RelicCounterView { relic: RelicId::PenNib, value: 5 }];
    for character in [Character::Ironclad, Character::Silent, Character::Defect, Character::Watcher] {
        let start_counters = BTreeMap::from([(RelicId::PenNib, 4)]);
        let mut run = m27_support::complete_run(character,
            vec![RelicId::PenNib, RelicId::SacredBark],
            vec![Some(PotionId::BlockPotion), Some(PotionId::SmokeBomb), Some(PotionId::FirePotion)],
            start_counters, crate::content::EncounterId::JawWorm);
        assert!(run.state().relics.is_empty() && run.state().potions.is_empty()
            && run.state().relic_counters.is_empty() && run.state().potion_potencies.is_empty(),
            "{character:?}: RunState relinquishes all four item stores on combat entry");
        let combat = run.state().combat.as_ref().expect("complete combat active");
        let items = combat.item_context().expect("complete combat owns item context");
        assert_eq!(items.relics(), &[RelicId::PenNib, RelicId::SacredBark]);
        assert_eq!(items.potions(), &[Some(PotionId::BlockPotion), Some(PotionId::SmokeBomb), Some(PotionId::FirePotion)]);
        assert_eq!(items.counters(), &BTreeMap::from([(RelicId::PenNib, 4)]));
        assert_eq!(items.potion_potencies, vec![Some(24), Some(0), Some(40)],
            "{character:?}: Sacred Bark potency stays in the combat owner");

        run.choose_combat(CombatAction::UsePotion { slot: 0, target: None })
            .unwrap_or_else(|error| panic!("{character:?} Block Potion: {error:?}"));
        let attack = {
            let combat = run.state().combat.as_ref().unwrap();
            let public = combat.public_full().unwrap();
            let catalog = Catalog::standard();
            let card = public.hand.iter().find(|card| catalog.card(card.id)
                .is_some_and(|definition| definition.card_type == CardType::Attack))
                .unwrap_or_else(|| panic!("{character:?} has no starter Attack in the opening hand"));
            let action = CombatAction::PlayCard { instance: card.instance.0, target: Some(TargetId(1)) };
            assert!(combat.legal_actions_full().unwrap().contains(&action), "{character:?} Attack is legal");
            action
        };
        run.choose_combat(attack).unwrap_or_else(|error| panic!("{character:?} Pen Nib Attack: {error:?}"));
        let live_items = run.state().combat.as_ref().unwrap().item_context().unwrap();
        assert_eq!(live_items.counter(RelicId::PenNib), 5, "{character:?}: one Attack advances Pen Nib once");
        assert_eq!(live_items.potions(), &[None, Some(PotionId::SmokeBomb), Some(PotionId::FirePotion)]);
        assert_eq!(crate::items::complete_counter_view(live_items.counters()), complete_pen_nib_view,
            "{character:?}: complete owner view includes the live Pen Nib counter");
        let first = run.decision().unwrap();
        let combat_inventory = first.combat.as_ref().unwrap().inventory.as_ref().unwrap();
        assert_eq!(combat_inventory.potions, [None, Some(PotionId::SmokeBomb), Some(PotionId::FirePotion)]);
        assert_eq!(crate::items::visible_counters(live_items.counters()), accepted_counter_view,
            "{character:?}: accepted visible counter projection omits Pen Nib");
        assert_eq!(combat_inventory.relic_counters, accepted_counter_view,
            "{character:?}: CombatPublic keeps the accepted visible counter projection");
        assert_eq!(first.player.relic_counters, accepted_counter_view,
            "{character:?}: PublicPlayer keeps the accepted visible counter projection");
        assert_eq!(combat_inventory.relics, live_items.relics(), "{character:?}: public combat view projects combat-owned relics");
        assert_eq!(first.player.potions, combat_inventory.potions, "{character:?}: decision projects combat-owned potion slots");
        assert_eq!(first.player.relic_counters, combat_inventory.relic_counters,
            "{character:?}: decision projects combat-owned counters");
        let second = run.decision().unwrap();
        assert_eq!(second, first, "{character:?}: rereading the live public decision does not apply a second settlement or advance the counter");
        assert_eq!(run.state().combat_progress.settled, 0);

        let active_items = run.state().combat.as_ref().unwrap().item_context().unwrap();
        let expected_relics = active_items.relics().to_vec();
        let expected_counters = active_items.counters().clone();
        let mut expected_potions = active_items.potions().to_vec();
        expected_potions[1] = None;
        let mut expected_potencies = active_items.potion_potencies.clone();
        expected_potencies[1] = None;
        run.choose_combat(CombatAction::UsePotion { slot: 1, target: None })
            .unwrap_or_else(|error| panic!("{character:?} Smoke Bomb escape: {error:?}"));
        assert_eq!(run.state().combat, None, "{character:?}: escape settles the active combat");
        assert_eq!(run.state().last_combat.as_ref().map(|record| record.outcome), Some(crate::combat::Outcome::Escaped));
        assert_eq!(run.state().relics, vec![RelicId::PenNib, RelicId::SacredBark]);
        assert_eq!(run.state().relics, expected_relics, "{character:?}: combat relic list returns once");
        assert_eq!(run.state().potions, vec![None, None, Some(PotionId::FirePotion)]);
        assert_eq!(run.state().potions, expected_potions, "{character:?}: used slots return empty");
        assert_eq!(run.state().relic_counters, BTreeMap::from([(RelicId::PenNib, 5)]));
        assert_eq!(run.state().relic_counters, expected_counters, "{character:?}: combat counter returns once");
        assert_eq!(run.state().potion_potencies, vec![None, None, Some((PotionId::FirePotion, 40))]);
        let expected_run_potencies = expected_potions.iter().enumerate().map(|(slot, potion)| potion.map(|id| {
            let potency = expected_potencies.get(slot).copied().flatten().unwrap_or_else(|| if id == PotionId::FirePotion { 40 } else { 0 });
            (id, potency)
        })).collect::<Vec<_>>();
        assert_eq!(run.state().potion_potencies, expected_run_potencies,
            "{character:?}: potency is paired with each remaining potion exactly once");
        assert_eq!((run.state().combat_progress.entered, run.state().combat_progress.settled), (1, 1));
        let settled_first = run.decision().unwrap();
        let settled_second = run.decision().unwrap();
        assert_eq!(settled_second, settled_first, "{character:?}: a public decision read does not settle again");
        assert_eq!(settled_first.player.relic_counters, accepted_counter_view,
            "{character:?}: RunState public view retains the accepted counter projection");
        assert_eq!(run.state().relic_counters.get(&RelicId::PenNib), Some(&5));
        assert_eq!(run.state().combat_progress.settled, 1);

        m27_support::next_complete_combat(&mut run, crate::content::EncounterId::JawWorm)
            .unwrap_or_else(|error| panic!("{character:?} next complete combat: {error:?}"));
        assert!(run.state().relics.is_empty() && run.state().potions.is_empty()
            && run.state().relic_counters.is_empty() && run.state().potion_potencies.is_empty(),
            "{character:?}: second combat takes the returned item state once");
        let next = run.decision().unwrap();
        let next_inventory = next.combat.as_ref().unwrap().inventory.as_ref().unwrap();
        let next_items = run.state().combat.as_ref().unwrap().item_context().unwrap();
        assert_eq!(next_items.counter(RelicId::PenNib), 5,
            "{character:?}: re-entered combat owner receives the settled Pen Nib counter");
        assert_eq!(crate::items::complete_counter_view(next_items.counters()), complete_pen_nib_view,
            "{character:?}: complete owner view includes Pen Nib after next entry");
        assert_eq!(crate::items::visible_counters(next_items.counters()), accepted_counter_view,
            "{character:?}: accepted visible projection still omits Pen Nib after next entry");
        assert_eq!(next_inventory.relic_counters, accepted_counter_view);
        assert_eq!(next.player.relic_counters, accepted_counter_view);
        assert_eq!(next_inventory.potions, [None, None, Some(PotionId::FirePotion)]);
        assert_eq!(run.decision().unwrap(), next, "{character:?}: rereading the next public decision leaves the counter unchanged");
        assert_eq!((run.state().combat_progress.entered, run.state().combat_progress.settled), (2, 1),
            "{character:?}: the second combat is entered but no second settlement is applied");
        assert_eq!(run.state().combat.as_ref().unwrap().item_context().unwrap().counter(RelicId::PenNib), 5);
    }
}
