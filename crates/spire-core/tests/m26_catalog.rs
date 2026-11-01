use spire_core::content::{
    is_healing, CardColor, CardId, CardRarity, CardType, Catalog, NoteStorage,
};

#[test]
fn m26_identity_catalog_rows() {
    let catalog = Catalog::standard();
    let expected = [
        (CardId::BandageUp, "bandage_up", CardColor::Colorless, CardRarity::Uncommon, CardType::Skill, NoteStorage::RemovablePermanent),
        (CardId::Apparition, "apparition", CardColor::Colorless, CardRarity::Special, CardType::Skill, NoteStorage::RemovablePermanent),
        (CardId::Bite, "bite", CardColor::Colorless, CardRarity::Special, CardType::Attack, NoteStorage::RemovablePermanent),
        (CardId::Jax, "jax", CardColor::Colorless, CardRarity::Special, CardType::Skill, NoteStorage::RemovablePermanent),
        (CardId::RitualDagger, "ritual_dagger", CardColor::Colorless, CardRarity::Special, CardType::Attack, NoteStorage::RemovablePermanent),
        (CardId::Clumsy, "clumsy", CardColor::Curse, CardRarity::Curse, CardType::Curse, NoteStorage::RemovablePermanent),
        (CardId::CurseOfTheBell, "curse_of_the_bell", CardColor::Curse, CardRarity::Curse, CardType::Curse, NoteStorage::NonremovableCurse),
        (CardId::Decay, "decay", CardColor::Curse, CardRarity::Curse, CardType::Curse, NoteStorage::RemovablePermanent),
        (CardId::Doubt, "doubt", CardColor::Curse, CardRarity::Curse, CardType::Curse, NoteStorage::RemovablePermanent),
        (CardId::Injury, "injury", CardColor::Curse, CardRarity::Curse, CardType::Curse, NoteStorage::RemovablePermanent),
        (CardId::Necronomicurse, "necronomicurse", CardColor::Curse, CardRarity::Curse, CardType::Curse, NoteStorage::NonremovableCurse),
        (CardId::Normality, "normality", CardColor::Curse, CardRarity::Curse, CardType::Curse, NoteStorage::RemovablePermanent),
        (CardId::Pain, "pain", CardColor::Curse, CardRarity::Curse, CardType::Curse, NoteStorage::RemovablePermanent),
        (CardId::Shame, "shame", CardColor::Curse, CardRarity::Curse, CardType::Curse, NoteStorage::RemovablePermanent),
        (CardId::Writhe, "writhe", CardColor::Curse, CardRarity::Curse, CardType::Curse, NoteStorage::RemovablePermanent),
    ];

    assert_eq!(catalog.cards().len(), 369);
    for (id, spelling, color, rarity, card_type, note_storage) in expected {
        assert_eq!(id.as_str(), spelling);
        let definition = catalog.card(id).unwrap_or_else(|| panic!("missing catalog row for {spelling}"));
        assert_eq!(definition.color, color, "{spelling} color");
        assert_eq!(definition.rarity, rarity, "{spelling} rarity");
        assert_eq!(definition.card_type, card_type, "{spelling} type");
        assert_eq!(definition.note_storage, note_storage, "{spelling} note storage");
        assert!(!definition.definition_only, "{spelling} must be supported");
    }

    let regret = catalog.card(CardId::Regret).expect("Regret catalog row");
    assert!(!regret.definition_only);
    assert_eq!(regret.note_storage, NoteStorage::RemovablePermanent);
    let ascenders_bane = catalog.card(CardId::AscendersBane).expect("Ascender's Bane catalog row");
    assert!(!ascenders_bane.definition_only);
    assert_eq!(ascenders_bane.note_storage, NoteStorage::NonremovableCurse);

    assert!(catalog.cards().iter().all(|definition| definition.id.as_str() != "pride"));
    assert!(is_healing(CardId::BandageUp));
    // Disclosed decision F4 (seam map H0 §7): Bite and Ritual Dagger carry no card tags.
    assert!(spire_core::content::card_tags(CardId::Bite).is_empty(), "F4: Bite has no tags");
    assert!(spire_core::content::card_tags(CardId::RitualDagger).is_empty(), "F4: Ritual Dagger has no tags");
}


const M10_COMMON: &[&str] = &["armaments", "body_slam", "clash", "sword_boomerang", "wild_strike", "true_grit", "headbutt", "warcry", "havoc", "anger", "iron_wave", "cleave", "pommel_strike", "shrug_it_off", "twin_strike", "clothesline", "heavy_blade", "perfected_strike", "thunderclap", "flex"];
const M10_UNCOMMON: &[&str] = &["searing_blow", "carnage", "hemokinesis", "pummel", "uppercut", "entrench", "ghostly_armor", "intimidate", "power_through", "reckless_charge", "seeing_red", "shockwave", "spot_weakness", "battle_trance", "dual_wield", "whirlwind", "inflame", "evolve", "fire_breathing", "flame_barrier", "rage", "rupture", "dark_embrace", "feel_no_pain", "sentinel", "second_wind", "sever_soul", "metallicize", "combust", "bloodletting", "burning_pact", "disarm", "blood_for_blood", "rampage", "dropkick", "infernal_blade"];
const M10_RARE: &[&str] = &["immolate", "exhume", "double_tap", "juggernaut", "corruption", "fiend_fire", "barricade", "berserk", "brutality", "demon_form", "bludgeon", "feed", "reaper", "impervious", "offering", "limit_break"];
const M15_COMMON: &[&str] = &["acrobatics", "backflip", "bane", "blade_dance", "cloak_and_dagger", "dagger_spray", "dagger_throw", "deadly_poison", "deflect", "dodge_and_roll", "flying_knee", "outmaneuver", "piercing_wail", "poisoned_stab", "prepared", "quick_slash", "slice", "sneaky_strike", "sucker_punch"];
const M15_UNCOMMON: &[&str] = &["accuracy", "all_out_attack", "backstab", "blur", "bouncing_flask", "calculated_gamble", "caltrops", "catalyst", "choke", "concentrate", "crippling_cloud", "dash", "distraction", "endless_agony", "escape_plan", "eviscerate", "expertise", "finisher", "flechettes", "footwork", "heel_hook", "infinite_blades", "leg_sweep", "masterful_stab", "noxious_fumes", "predator", "reflex", "riddle_with_holes", "setup", "skewer", "tactician", "terror", "well_laid_plans"];
const M15_RARE: &[&str] = &["a_thousand_cuts", "adrenaline", "after_image", "alchemize", "bullet_time", "burst", "corpse_explosion", "die_die_die", "doppelganger", "envenom", "glass_knife", "grand_finale", "malaise", "nightmare", "phantasmal_killer", "storm_of_steel", "tools_of_the_trade", "unload", "wraith_form"];
const INFERNAL_BLADE_M10: &[&str] = &["searing_blow", "body_slam", "carnage", "clash", "hemokinesis", "pummel", "uppercut", "immolate", "sword_boomerang", "wild_strike", "reckless_charge", "headbutt", "whirlwind", "sever_soul", "fiend_fire", "anger", "iron_wave", "cleave", "pommel_strike", "twin_strike", "clothesline", "heavy_blade", "perfected_strike", "thunderclap", "bludgeon", "blood_for_blood", "rampage", "dropkick"];
const SILENT_ATTACK: &[&str] = &["bane", "dagger_spray", "dagger_throw", "flying_knee", "poisoned_stab", "quick_slash", "slice", "sneaky_strike", "sucker_punch", "all_out_attack", "backstab", "choke", "dash", "endless_agony", "eviscerate", "finisher", "flechettes", "heel_hook", "masterful_stab", "predator", "riddle_with_holes", "skewer", "die_die_die", "glass_knife", "grand_finale", "unload"];
const SILENT_SKILL: &[&str] = &["acrobatics", "backflip", "blade_dance", "cloak_and_dagger", "deadly_poison", "deflect", "dodge_and_roll", "outmaneuver", "piercing_wail", "prepared", "blur", "bouncing_flask", "calculated_gamble", "catalyst", "concentrate", "crippling_cloud", "distraction", "escape_plan", "expertise", "leg_sweep", "reflex", "setup", "tactician", "terror", "adrenaline", "bullet_time", "burst", "corpse_explosion", "doppelganger", "malaise", "nightmare", "phantasmal_killer", "storm_of_steel"];
const SILENT_POWER: &[&str] = &["accuracy", "caltrops", "footwork", "infinite_blades", "noxious_fumes", "well_laid_plans", "a_thousand_cuts", "after_image", "envenom", "tools_of_the_trade", "wraith_form"];
const SILENT_ANY: &[&str] = &["acrobatics", "backflip", "bane", "blade_dance", "cloak_and_dagger", "dagger_spray", "dagger_throw", "deadly_poison", "deflect", "dodge_and_roll", "flying_knee", "outmaneuver", "piercing_wail", "poisoned_stab", "prepared", "quick_slash", "slice", "sneaky_strike", "sucker_punch", "accuracy", "all_out_attack", "backstab", "blur", "bouncing_flask", "calculated_gamble", "caltrops", "catalyst", "choke", "concentrate", "crippling_cloud", "dash", "distraction", "endless_agony", "escape_plan", "eviscerate", "expertise", "finisher", "flechettes", "footwork", "heel_hook", "infinite_blades", "leg_sweep", "masterful_stab", "noxious_fumes", "predator", "reflex", "riddle_with_holes", "setup", "skewer", "tactician", "terror", "well_laid_plans", "a_thousand_cuts", "adrenaline", "after_image", "bullet_time", "burst", "corpse_explosion", "die_die_die", "doppelganger", "envenom", "glass_knife", "grand_finale", "malaise", "nightmare", "phantasmal_killer", "storm_of_steel", "tools_of_the_trade", "unload", "wraith_form"];
const DEFECT_ATTACK: &[&str] = &["ball_lightning", "barrage", "beam_cell", "claw", "cold_snap", "compile_driver", "go_for_the_eyes", "rebound", "streamline", "sweeping_beam", "blizzard", "bullseye", "doom_and_gloom", "ftl", "melter", "rip_and_tear", "scrape", "sunder", "all_for_one", "core_surge", "hyperbeam", "meteor_strike", "thunder_strike"];
const DEFECT_SKILL: &[&str] = &["charge_battery", "coolheaded", "hologram", "leap", "recursion", "stack", "steam_barrier", "turbo", "aggregate", "auto_shields", "boot_sequence", "chaos", "chill", "consume", "darkness", "double_energy", "equilibrium", "force_field", "fusion", "genetic_algorithm", "glacier", "overclock", "recycle", "reinforced_body", "reprogram", "skim", "tempest", "white_noise", "amplify", "fission", "multi_cast", "rainbow", "reboot", "seek"];
const DEFECT_POWER: &[&str] = &["capacitor", "defragment", "heatsinks", "hello_world", "loop", "static_discharge", "storm", "biased_cognition", "buffer", "creative_ai", "echo_form", "electrodynamics", "machine_learning"];
const DEFECT_ANY: &[&str] = &["ball_lightning", "barrage", "beam_cell", "charge_battery", "claw", "cold_snap", "compile_driver", "coolheaded", "go_for_the_eyes", "hologram", "leap", "rebound", "recursion", "stack", "steam_barrier", "streamline", "sweeping_beam", "turbo", "aggregate", "auto_shields", "blizzard", "boot_sequence", "bullseye", "capacitor", "chaos", "chill", "consume", "darkness", "defragment", "doom_and_gloom", "double_energy", "equilibrium", "force_field", "ftl", "fusion", "genetic_algorithm", "glacier", "heatsinks", "hello_world", "loop", "melter", "overclock", "recycle", "reinforced_body", "reprogram", "rip_and_tear", "scrape", "skim", "static_discharge", "storm", "sunder", "tempest", "white_noise", "all_for_one", "amplify", "biased_cognition", "buffer", "core_surge", "creative_ai", "echo_form", "electrodynamics", "fission", "hyperbeam", "machine_learning", "meteor_strike", "multi_cast", "rainbow", "reboot", "seek", "thunder_strike"];
const DEFECT_COMMON: &[&str] = &["ball_lightning", "barrage", "beam_cell", "charge_battery", "claw", "cold_snap", "compile_driver", "coolheaded", "go_for_the_eyes", "hologram", "leap", "rebound", "recursion", "stack", "steam_barrier", "streamline", "sweeping_beam", "turbo"];
const L34: &[&str] = &["apotheosis", "blind", "chrysalis", "dark_shackles", "deep_breath", "discovery", "dramatic_entrance", "enlightenment", "finesse", "flash_of_steel", "forethought", "good_instincts", "hand_of_greed", "impatience", "jack_of_all_trades", "madness", "magnetism", "master_of_strategy", "mayhem", "metamorphosis", "mind_blast", "panacea", "panache", "panic_button", "purity", "sadistic_nature", "secret_technique", "secret_weapon", "swift_strike", "the_bomb", "thinking_ahead", "transmutation", "trip", "violence"];
const FOREIGN_INFLUENCE: &[&str] = &["searing_blow", "body_slam", "carnage", "clash", "hemokinesis", "pummel", "uppercut", "immolate", "sword_boomerang", "wild_strike", "reckless_charge", "headbutt", "whirlwind", "sever_soul", "fiend_fire", "anger", "iron_wave", "cleave", "pommel_strike", "twin_strike", "dagger_throw", "quick_slash", "clothesline", "heavy_blade", "perfected_strike", "thunderclap", "ball_lightning", "cold_snap", "consecrate", "empty_fist", "beam_cell", "go_for_the_eyes", "barrage", "compile_driver", "sweeping_beam", "poisoned_stab", "bludgeon", "blood_for_blood", "rampage", "dropkick", "dagger_spray", "slice", "sucker_punch", "backstab", "dash", "riddle_with_holes", "die_die_die", "sneaky_strike", "all_out_attack", "finisher", "flechettes", "grand_finale", "unload", "flying_knee", "choke", "predator", "bane", "heel_hook", "dramatic_entrance", "flash_of_steel", "mind_blast", "swift_strike", "hand_of_greed", "masterful_stab", "eviscerate", "endless_agony", "glass_knife", "skewer", "bullseye", "claw", "streamline", "rebound", "blizzard", "doom_and_gloom", "ftl", "melter", "rip_and_tear", "scrape", "sunder", "all_for_one", "core_surge", "hyperbeam", "meteor_strike", "thunder_strike", "flurry_of_blows", "brilliance", "cut_through_fate", "just_lucky", "weave", "windmill_strike", "sands_of_time", "carve_reality", "bowling_bash", "crush_joints", "flying_sleeves", "follow_up", "sash_whip", "conclude", "fear_no_evil", "reach_heaven", "signature_move", "talk_to_the_hand", "tantrum", "wallop", "wheel_kick", "ragnarok"];

use spire_core::content::{
    pools::{cards as m26_cards, CardPool, GenerationPool},
    Character, ContentProducer, RegistryError,
};

fn spellings(ids: &[CardId]) -> Vec<&'static str> {
    ids.iter().map(|id| id.as_str()).collect()
}

fn assert_spellings(ids: &[CardId], expected: &[&str], context: &str) {
    assert_eq!(spellings(ids), expected.to_vec(), "{context}");
}

fn rarity_rank(id: CardId) -> u8 {
    match Catalog::standard().card(id).expect("pool member has a catalog row").rarity {
        CardRarity::Basic => 0,
        CardRarity::Common => 1,
        CardRarity::Uncommon => 2,
        CardRarity::Rare => 3,
        CardRarity::Special => 4,
        CardRarity::Curse => 5,
        CardRarity::Status => 6,
    }
}

fn assert_rarity_name_order(ids: &[CardId], context: &str) {
    let keys = ids.iter().map(|id| (rarity_rank(*id), id.as_str())).collect::<Vec<_>>();
    assert!(keys.windows(2).all(|pair| pair[0] <= pair[1]), "{context}: {keys:?}");
}

fn get_pool(pool: CardPool) -> Vec<CardId> {
    m26_cards(pool).unwrap_or_else(|error| panic!("pool {pool:?}: {error:?}"))
}

const CURSES: &[&str] = &[
    "clumsy", "decay", "doubt", "injury", "normality", "pain", "parasite", "regret", "shame", "writhe",
];
const COLORLESS_ORDINARY: &[&str] = &[
    "bandage_up", "blind", "dark_shackles", "deep_breath", "discovery",
    "dramatic_entrance", "enlightenment", "finesse", "flash_of_steel",
    "forethought", "good_instincts", "impatience", "jack_of_all_trades",
    "madness", "mind_blast", "panacea", "panic_button", "purity",
    "swift_strike", "trip", "apotheosis", "chrysalis", "hand_of_greed",
    "magnetism", "master_of_strategy", "mayhem", "metamorphosis", "panache",
    "sadistic_nature", "secret_technique", "secret_weapon", "the_bomb",
    "thinking_ahead", "transmutation", "violence",
];

#[test]
fn m26_pools_permanent_sizes_and_order() {
    use CardRarity::*;
    use CardType::*;

    let typed = [
        (Character::Ironclad, [14, 11, 5], [6, 17, 5], [0, 8, 6], [20, 36, 16]),
        (Character::Silent, [9, 13, 4], [10, 14, 10], [0, 6, 5], [19, 33, 19]),
        (Character::Defect, [10, 8, 5], [8, 20, 6], [0, 8, 6], [18, 36, 17]),
        (Character::Watcher, [10, 12, 3], [9, 15, 10], [0, 8, 4], [19, 35, 17]),
    ];
    let rarities = [Common, Uncommon, Rare];
    let types = [(Attack, 0usize), (Skill, 1usize), (Power, 2usize)];

    for (character, attack_counts, skill_counts, power_counts, totals) in typed {
        let expected_color = match character {
            Character::Ironclad => CardColor::Red,
            Character::Silent => CardColor::Green,
            Character::Defect => CardColor::Blue,
            Character::Watcher => CardColor::Purple,
        };
        let by_type = [attack_counts, skill_counts, power_counts];
        let all = get_pool(CardPool::Ordinary { character, rarity: None, card_type: None });
        assert_eq!(all.len(), totals.iter().sum::<usize>(), "{character:?} total");

        let expected_all: Vec<&str> = match character {
            Character::Ironclad => M10_COMMON.iter().chain(M10_UNCOMMON).chain(M10_RARE).copied().collect(),
            Character::Silent => M15_COMMON.iter().chain(M15_UNCOMMON).chain(M15_RARE).copied().collect(),
            _ => {
                let mut expected = Catalog::standard().cards().iter()
                    .filter(|row| row.color == expected_color
                        && matches!(row.rarity, Common | Uncommon | Rare)
                        && !row.definition_only)
                    .map(|row| row.id).collect::<Vec<_>>();
                expected.sort_by_key(|id| (rarity_rank(*id), id.as_str()));
                expected.iter().map(|id| id.as_str()).collect()
            }
        };
        assert_spellings(&all, &expected_all, &format!("{character:?} ordinary C/U/R order"));

        for (index, rarity) in rarities.into_iter().enumerate() {
            let family = get_pool(CardPool::Ordinary { character, rarity: Some(rarity), card_type: None });
            assert_eq!(family.len(), totals[index], "{character:?} {rarity:?} total");
            assert!(family.iter().all(|id| {
                let row = Catalog::standard().card(*id).unwrap();
                row.color == expected_color && row.rarity == rarity && !row.definition_only
            }), "{character:?} {rarity:?} members");

            let recorded = match character {
                Character::Ironclad => Some([M10_COMMON, M10_UNCOMMON, M10_RARE][index]),
                Character::Silent => Some([M15_COMMON, M15_UNCOMMON, M15_RARE][index]),
                _ => None,
            };
            if let Some(expected) = recorded {
                assert_spellings(&family, expected, &format!("{character:?} {rarity:?} recorded order"));
            } else {
                let mut expected = Catalog::standard().cards().iter()
                    .filter(|row| row.color == expected_color && row.rarity == rarity && !row.definition_only)
                    .map(|row| row.id).collect::<Vec<_>>();
                expected.sort_by_key(|id| id.as_str());
                assert_eq!(family, expected, "{character:?} {rarity:?} sorted source family");
            }

            for (card_type, type_index) in types {
                let typed_family = get_pool(CardPool::Ordinary {
                    character, rarity: Some(rarity), card_type: Some(card_type),
                });
                if let Some(recorded) = recorded {
                    let expected = recorded.iter().copied().filter(|spelling| {
                        Catalog::standard().cards().iter()
                            .find(|row| row.id.as_str() == *spelling)
                            .expect("recorded source card is in the catalog")
                            .card_type == card_type
                    }).collect::<Vec<_>>();
                    assert_spellings(&typed_family, &expected,
                        &format!("{character:?} {rarity:?} {card_type:?} recorded filter"));
                } else {
                    let mut expected = Catalog::standard().cards().iter()
                        .filter(|row| row.color == expected_color && row.rarity == rarity
                            && row.card_type == card_type && !row.definition_only)
                        .map(|row| row.id).collect::<Vec<_>>();
                    expected.sort_by_key(|id| id.as_str());
                    assert_eq!(typed_family, expected,
                        "{character:?} {rarity:?} {card_type:?} sorted source filter");
                }
                assert_eq!(typed_family.len(), by_type[type_index][index],
                    "{character:?} {card_type:?} {rarity:?}");
            }
        }
    }

    let colourless = get_pool(CardPool::Colourless { rarity: None });
    assert_eq!(colourless.len(), 35);
    assert_spellings(&colourless, COLORLESS_ORDINARY, "colourless ordinary C/U/R order");
    for (index, rarity) in rarities.into_iter().enumerate() {
        let family = get_pool(CardPool::Colourless { rarity: Some(rarity) });
        assert_eq!(family.len(), [0, 20, 15][index], "colourless {rarity:?}");
        let expected = COLORLESS_ORDINARY.iter().copied().filter(|spelling| {
            Catalog::standard().cards().iter()
                .find(|row| row.id.as_str() == *spelling)
                .expect("colourless source card is in catalog").rarity == rarity
        }).collect::<Vec<_>>();
        assert_spellings(&family, &expected, &format!("colourless {rarity:?} sorted source family"));
        assert!(family.iter().all(|id| {
            let row = Catalog::standard().card(*id).unwrap();
            row.color == CardColor::Colorless && row.rarity == rarity && !row.definition_only
        }), "colourless {rarity:?} members");
        let type_counts = [(Attack, [0, 4, 1]), (Skill, [0, 16, 10]), (Power, [0, 0, 4])];
        for (card_type, expected) in type_counts {
            let actual = family.iter().filter(|id|
                Catalog::standard().card(**id).unwrap().card_type == card_type
            ).count();
            assert_eq!(actual, expected[index], "colourless {card_type:?} {rarity:?}");
        }
    }
    assert_eq!(colourless.iter().map(|id| Catalog::standard().card(*id).unwrap().rarity)
        .filter(|rarity| *rarity == Common).count(), 0);
    assert!(colourless.iter().all(|id| Catalog::standard().card(*id).unwrap().color == CardColor::Colorless));
    assert!(colourless.contains(&CardId::BandageUp), "Bandage Up remains in permanent colourless acquisition");

    let any_colour_sizes = [(Some(Common), 76), (Some(Uncommon), 160), (Some(Rare), 84), (None, 320)];
    for (rarity, expected_len) in any_colour_sizes {
        let family = get_pool(CardPool::AnyColour { rarity });
        assert_eq!(family.len(), expected_len, "AnyColour {rarity:?}");
        let mut expected = Catalog::standard().cards().iter()
            .filter(|row| matches!(row.color, CardColor::Red | CardColor::Green | CardColor::Blue
                | CardColor::Purple | CardColor::Colorless)
                && matches!(row.rarity, Common | Uncommon | Rare)
                && rarity.is_none_or(|expected| row.rarity == expected)
                && !row.definition_only)
            .map(|row| row.id).collect::<Vec<_>>();
        expected.sort_by_key(|id| (rarity_rank(*id), id.as_str()));
        assert_eq!(family, expected, "AnyColour {rarity:?} sorted source family");
    }

    let curses = get_pool(CardPool::Curse);
    assert_eq!(curses.len(), 10);
    assert_spellings(&curses, CURSES, "random Curse support order");

    let note = get_pool(CardPool::NoteForYourself);
    assert_eq!(note.len(), 349);
    let mut expected_note = Catalog::standard().cards().iter()
        .filter(|row| row.note_storage == NoteStorage::RemovablePermanent)
        .map(|row| row.id).collect::<Vec<_>>();
    expected_note.sort_by_key(|id| (rarity_rank(*id), id.as_str()));
    assert_eq!(note, expected_note, "NoteForYourself sorted removable-permanent source family");
    for excluded in [
        CardId::CurseOfTheBell, CardId::Necronomicurse, CardId::AscendersBane,
        CardId::Shiv, CardId::Miracle, CardId::Smite, CardId::Safety, CardId::Insight,
        CardId::ThroughViolence, CardId::Beta, CardId::Omega, CardId::Expunger,
        CardId::Burn, CardId::Dazed, CardId::Slimed, CardId::Wound, CardId::Void,
        CardId::FameAndFortune, CardId::BecomeAlmighty, CardId::LiveForever,
    ] {
        assert!(!note.contains(&excluded), "{excluded:?} is not NoteForYourself-removable");
    }

    let ironclad_rare = get_pool(CardPool::Ordinary {
        character: Character::Ironclad, rarity: Some(Rare), card_type: None,
    });
    assert!(ironclad_rare.contains(&CardId::Feed));
    assert!(ironclad_rare.contains(&CardId::Reaper));
    assert!(get_pool(CardPool::Colourless { rarity: Some(Uncommon) }).contains(&CardId::BandageUp));
    assert_eq!(get_pool(CardPool::Colourless { rarity: Some(Common) }), Vec::<CardId>::new());
    for rarity in [CardRarity::Basic, CardRarity::Special, CardRarity::Status, CardRarity::Curse] {
        assert_eq!(m26_cards(CardPool::Ordinary {
            character: Character::Ironclad, rarity: Some(rarity), card_type: None,
        }), Err(RegistryError::IncompletePool), "Ordinary rejects {rarity:?}");
        assert_eq!(m26_cards(CardPool::AnyColour { rarity: Some(rarity) }),
            Err(RegistryError::IncompletePool), "AnyColour rejects {rarity:?}");
        assert_eq!(m26_cards(CardPool::Colourless { rarity: Some(rarity) }),
            Err(RegistryError::IncompletePool), "Colourless rejects {rarity:?}");
    }
}

#[test]
fn m26_pools_transform_supports() {
    let cases = [
        (Character::Ironclad, CardId::Strike, 72, None),
        (Character::Ironclad, CardId::Anger, 71, Some(CardId::Anger)),
        (Character::Silent, CardId::StrikePurple, 71, None),
        (Character::Ironclad, CardId::Apparition, 35, None),
        (Character::Ironclad, CardId::BandageUp, 34, Some(CardId::BandageUp)),
        (Character::Silent, CardId::Regret, 9, Some(CardId::Regret)),
        (Character::Watcher, CardId::CurseOfTheBell, 10, None),
    ];
    for (character, input, count, excluded) in cases {
        let transformed = get_pool(CardPool::Transform { character, input });
        assert_eq!(transformed.len(), count, "{character:?} transform {input:?}");
        if let Some(excluded) = excluded {
            assert!(!transformed.contains(&excluded), "{input:?} is removed from its transform support");
        }
        let definition = Catalog::standard().card(input).expect("transform input has a definition");
        let mut base: Vec<&str> = Vec::new();
        if definition.color == CardColor::Curse {
            base.extend_from_slice(CURSES);
        } else if definition.color == CardColor::Colorless {
            base.extend_from_slice(COLORLESS_ORDINARY);
        } else {
            match character {
                Character::Ironclad => {
                    base.extend_from_slice(M10_COMMON);
                    base.extend_from_slice(M10_UNCOMMON);
                    base.extend_from_slice(M10_RARE);
                }
                Character::Silent => {
                    base.extend_from_slice(M15_COMMON);
                    base.extend_from_slice(M15_UNCOMMON);
                    base.extend_from_slice(M15_RARE);
                }
                _ => unreachable!("this transform case uses a literal Ironclad or Silent family"),
            }
        }
        let expected = base.into_iter().filter(|id| *id != input.as_str()).collect::<Vec<_>>();
        assert_spellings(&transformed, &expected, &format!("{character:?} transform {input:?} literal order"));
    }
    assert_spellings(
        &get_pool(CardPool::Transform { character: Character::Watcher, input: CardId::CurseOfTheBell }),
        CURSES,
        "CurseOfTheBell transforms over all random curses, including itself",
    );
    assert_eq!(
        get_pool(CardPool::Transform { character: Character::Ironclad, input: CardId::Anger })
            .iter().filter(|id| **id != CardId::Anger).count(), 71,
    );
    assert_eq!(
        get_pool(CardPool::Transform { character: Character::Ironclad, input: CardId::BandageUp })
            .iter().filter(|id| **id != CardId::BandageUp).count(), 34,
    );
    assert_eq!(
        get_pool(CardPool::Transform { character: Character::Silent, input: CardId::Regret })
            .iter().filter(|id| **id != CardId::Regret).count(), 9,
    );
}

#[test]
fn m26_pools_combat_all_characters() {
    let characters = [
        (Character::Ironclad, CardColor::Red, [28, 28, 14, 70, 20]),
        (Character::Silent, CardColor::Green, [26, 33, 11, 70, 19]),
        (Character::Defect, CardColor::Blue, [23, 34, 13, 70, 18]),
        (Character::Watcher, CardColor::Purple, [24, 33, 12, 69, 19]),
    ];
    let pools = [
        (GenerationPool::Attack, CardType::Attack, 0usize),
        (GenerationPool::Skill, CardType::Skill, 1usize),
        (GenerationPool::Power, CardType::Power, 2usize),
        (GenerationPool::Any, CardType::Attack, 3usize),
        (GenerationPool::Common, CardType::Attack, 4usize),
    ];

    for (character, color, expected_sizes) in characters {
        for (pool, card_type, slot) in pools {
            let ids = get_pool(CardPool::Combat { character, pool });
            assert_eq!(ids.len(), expected_sizes[slot], "{character:?} {pool:?}");
            let recorded = match (character, pool) {
                (Character::Ironclad, GenerationPool::Attack) => Some(INFERNAL_BLADE_M10),
                (Character::Silent, GenerationPool::Attack) => Some(SILENT_ATTACK),
                (Character::Silent, GenerationPool::Skill) => Some(SILENT_SKILL),
                (Character::Silent, GenerationPool::Power) => Some(SILENT_POWER),
                (Character::Silent, GenerationPool::Any) => Some(SILENT_ANY),
                (Character::Defect, GenerationPool::Attack) => Some(DEFECT_ATTACK),
                (Character::Defect, GenerationPool::Skill) => Some(DEFECT_SKILL),
                (Character::Defect, GenerationPool::Power) => Some(DEFECT_POWER),
                (Character::Defect, GenerationPool::Any) => Some(DEFECT_ANY),
                (Character::Defect, GenerationPool::Common) => Some(DEFECT_COMMON),
                _ => None,
            };
            if let Some(expected) = recorded {
                assert_spellings(&ids, expected, &format!("{character:?} {pool:?} accepted order"));
            } else {
                let excluded = |id| match character {
                    Character::Ironclad => matches!(id, CardId::Feed | CardId::Reaper),
                    Character::Silent => id == CardId::Alchemize,
                    Character::Defect => id == CardId::SelfRepair,
                    Character::Watcher => matches!(id, CardId::LessonLearned | CardId::Wish),
                };
                let mut expected = Catalog::standard().cards().iter()
                    .filter(|row| row.color == color && !row.definition_only
                        && matches!(row.rarity, CardRarity::Common | CardRarity::Uncommon | CardRarity::Rare)
                        && !is_healing(row.id)
                        && !excluded(row.id)
                        && (pool == GenerationPool::Any || pool == GenerationPool::Common
                            || row.card_type == card_type)
                        && (pool != GenerationPool::Common || row.rarity == CardRarity::Common))
                    .map(|row| row.id).collect::<Vec<_>>();
                expected.sort_by_key(|id| (rarity_rank(*id), id.as_str()));
                assert_eq!(ids, expected, "{character:?} {pool:?} sorted source family");
            }
            assert!(ids.iter().all(|id| {
                let row = Catalog::standard().card(*id).unwrap();
                row.color == color && !row.definition_only
                    && matches!(row.rarity, CardRarity::Common | CardRarity::Uncommon | CardRarity::Rare)
                    && (pool == GenerationPool::Any || pool == GenerationPool::Common || row.card_type == card_type)
                    && (pool != GenerationPool::Common || row.rarity == CardRarity::Common)
            }), "{character:?} {pool:?} membership metadata");
        }
    }

    let infernal = get_pool(CardPool::Combat {
        character: Character::Ironclad, pool: GenerationPool::Attack,
    });
    assert_spellings(&infernal, INFERNAL_BLADE_M10, "M10 Infernal Blade accepted Attack order");
    for character in [Character::Ironclad, Character::Silent, Character::Defect, Character::Watcher] {
        let l34 = get_pool(CardPool::Combat { character, pool: GenerationPool::Colorless });
        assert_spellings(&l34, L34, &format!("{character:?} L34 order"));
    }
    assert_spellings(&get_pool(CardPool::ForeignInfluence), FOREIGN_INFLUENCE, "M25 Foreign Influence order");

    let mut every_combat_pool = Vec::new();
    for (character, _, _) in characters {
        for pool in [
            GenerationPool::Attack, GenerationPool::Skill, GenerationPool::Power,
            GenerationPool::Any, GenerationPool::Colorless, GenerationPool::Common,
        ] {
            every_combat_pool.extend(get_pool(CardPool::Combat { character, pool }));
        }
    }
    every_combat_pool.extend(get_pool(CardPool::ForeignInfluence));
    for excluded in [
        CardId::Feed, CardId::Reaper, CardId::LessonLearned, CardId::Wish,
        CardId::Alchemize, CardId::SelfRepair, CardId::BandageUp,
    ] {
        assert!(!every_combat_pool.contains(&excluded), "{excluded:?} is excluded from every combat pool");
    }
}

#[test]
fn m26_pools_selectable_and_eligibility() {
    let catalog = Catalog::standard();
    let mut expected_watcher = catalog.cards().iter()
        .filter(|row| row.color == CardColor::Purple
            && matches!(row.rarity, CardRarity::Common | CardRarity::Uncommon | CardRarity::Rare)
            && !row.definition_only)
        .map(|row| row.id).collect::<Vec<_>>();
    expected_watcher.sort_by_key(|id| (rarity_rank(*id), id.as_str()));

    let watcher = catalog.selectable_pool(ContentProducer::OrdinaryReward, Character::Watcher)
        .expect("M26 G1 Watcher ordinary rewards are complete");
    assert_eq!(watcher.len(), 71);
    assert_eq!(watcher, expected_watcher, "Watcher OrdinaryReward is the sorted purple C/U/R family");
    for id in &watcher {
        assert_eq!(catalog.eligibility(*id, ContentProducer::OrdinaryReward, Character::Watcher), Ok(true), "{id:?} OrdinaryReward");
        assert_eq!(catalog.eligibility(*id, ContentProducer::ShardOrdinaryReward, Character::Watcher), Ok(true), "{id:?} ShardOrdinaryReward");
    }

    let shard = catalog.selectable_pool(ContentProducer::ShardOrdinaryReward, Character::Silent)
        .expect("M26 ShardOrdinaryReward is complete");
    assert_eq!(shard.len(), 320);
    assert_rarity_name_order(&shard, "ShardOrdinaryReward");
    assert_eq!(catalog.selectable_pool(ContentProducer::NoteForYourself, Character::Ironclad)
        .expect("M26 NoteForYourself selector is complete").len(), 349);
    assert_eq!(
        catalog.selectable_pool(ContentProducer::PureWater, Character::Silent),
        Ok(vec![CardId::Miracle]),
    );
}
