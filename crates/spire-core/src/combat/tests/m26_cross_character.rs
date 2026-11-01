//! Lane B matrix X: shared card laws and section 6.2 literal rows.
use super::super::*;
use super::m26_support::{self, CardCountChange, EnemyDelta, PublicDelta, CHARACTERS};
use crate::{
    content::{
        complete::{supports_generated, supports_permanent},
        pools::{self, CardPool, GenerationPool},
        CardColor, CardId, Catalog, Character, PotionId,
    },
    run::complete_upgrades_valid,
};

fn upgrades(id: CardId) -> Vec<u32> {
    if complete_upgrades_valid(id, 1) { vec![0, 1] } else { vec![0] }
}

fn owner(id: CardId) -> Character {
    match Catalog::standard().card(id).expect("complete catalog card").color {
        CardColor::Red => Character::Ironclad,
        CardColor::Green => Character::Silent,
        CardColor::Blue => Character::Defect,
        CardColor::Purple => Character::Watcher,
        CardColor::Colorless | CardColor::Curse | CardColor::Status => Character::Ironclad,
        CardColor::Shared => unreachable!("complete executable rows have no shared owner"),
    }
}

fn skip_orb(id: CardId) -> bool {
    // Literal section 6.2 sensitive rows from H0 seam-map section 8, including Focus-only cards.
    matches!(id,
        CardId::Zap | CardId::Dualcast | CardId::BallLightning | CardId::Barrage
        | CardId::ColdSnap | CardId::CompileDriver | CardId::Coolheaded | CardId::Recursion
        | CardId::Blizzard | CardId::Capacitor | CardId::Chaos | CardId::Chill | CardId::Consume
        | CardId::Darkness | CardId::DoomAndGloom | CardId::Fusion | CardId::Glacier | CardId::Loop
        | CardId::StaticDischarge | CardId::Storm | CardId::Tempest | CardId::Electrodynamics
        | CardId::Fission | CardId::MeteorStrike | CardId::MultiCast | CardId::Rainbow
        | CardId::ThunderStrike | CardId::Defragment | CardId::BiasedCognition)
}

fn skip_generator(id: CardId) -> bool {
    // Literal generator exceptions required by B1; they have separate pool assertions below.
    matches!(id,
        CardId::InfernalBlade | CardId::Distraction | CardId::WhiteNoise | CardId::CreativeAI
        | CardId::HelloWorld | CardId::Discovery | CardId::Chrysalis | CardId::Metamorphosis
        | CardId::JackOfAllTrades | CardId::Transmutation | CardId::Magnetism
        | CardId::ForeignInfluence | CardId::Alchemize)
}

#[test]
fn m26_cross_character_equivalence() {
    let rows = Catalog::standard().cards();
    let mut mismatches = Vec::new();
    for definition in rows.iter().filter(|row| supports_permanent(row.id) || supports_generated(row.id)) {
        let kind = definition.id;
        if skip_orb(kind) || skip_generator(kind) { continue; }
        let owning_character = owner(kind);
        for upgrade in upgrades(kind) {
            let expected = m26_support::play_delta(owning_character, kind, upgrade);
            if expected.fault.is_some() {
                mismatches.push(format!("{kind:?}|{upgrade}|{owning_character:?}|fault=None|actual={expected:?}"));
            }
            for character in CHARACTERS {
                let actual = m26_support::play_delta(character, kind, upgrade);
                if actual.fault.is_some() || actual != expected {
                    mismatches.push(format!("{kind:?}|{upgrade}|{character:?}|{expected:?}|{actual:?}"));
                }
            }
        }
    }

    // H0 section 6's accepted own-colour script rows: the overlay must leave each
    // character's own card law byte-identical in the stripped comparison.
    let own = [
        (Character::Ironclad, [CardId::Flex, CardId::PommelStrike, CardId::Inflame, CardId::Armaments, CardId::ShrugItOff]),
        (Character::Silent, [CardId::Backflip, CardId::DeadlyPoison, CardId::Footwork, CardId::Acrobatics, CardId::BladeDance]),
        (Character::Defect, [CardId::BallLightning, CardId::Coolheaded, CardId::Defragment, CardId::ChargeBattery, CardId::Loop]),
        (Character::Watcher, [CardId::FlurryOfBlows, CardId::Worship, CardId::Protect, CardId::Foresight, CardId::MentalFortress]),
    ];
    for (character, kinds) in own {
        for kind in kinds {
            // The finite M25 admission rejects off-colour permanent Defend in the
            // baseline deck, so keep the Watcher control inside its accepted roster.
            let defend = if character == Character::Watcher { CardId::DefendPurple } else { CardId::Defend };
            let without = m26_support::play_delta_with_seed_and_overlay_using_defend(
                character, kind, 0, [41; 32], false, false, defend,
            );
            let with = m26_support::play_delta_with_seed_and_overlay_using_defend(
                character, kind, 0, [41; 32], true, false, defend,
            );
            if without.fault.is_some() || with.fault.is_some() || without != with {
                mismatches.push(format!("{kind:?}|0|{character:?}|{without:?}|{with:?}"));
            }
        }
    }

    assert!(mismatches.is_empty(), "m26 cross-character rows (kind|upgrades|character|expected|actual):\n{}",
        mismatches.join("\n"));
}

#[derive(Clone, Copy)]
struct OrbLiteral {
    id: CardId,
    cost: (i32, i32),
    enemy_hp: (i32, i32),
    block: (i32, i32),
    focus: (i32, i32),
    power: Option<(&'static str, i32, i32)>,
    draw: (i32, i32),
    exhaust: (bool, bool),
    slots: (usize, usize),
}

// Literal values are from data/cards/cards.json (card text, cost and flags),
// defect source rows in H0 frontier orb-cards.tsv, and M1b's accepted zero-slot law.
fn orb_literals() -> [OrbLiteral; 29] {
    use CardId::*;
    [
        OrbLiteral { id: Zap, cost: (1,0), enemy_hp: (0,0), block: (0,0), focus: (0,0), power: None, draw: (0,0), exhaust: (false,false), slots: (0,0) },
        OrbLiteral { id: Dualcast, cost: (1,0), enemy_hp: (0,0), block: (0,0), focus: (0,0), power: None, draw: (0,0), exhaust: (false,false), slots: (0,0) },
        OrbLiteral { id: BallLightning, cost: (1,1), enemy_hp: (-7,-10), block: (0,0), focus: (0,0), power: None, draw: (0,0), exhaust: (false,false), slots: (0,0) },
        OrbLiteral { id: Barrage, cost: (1,1), enemy_hp: (0,0), block: (0,0), focus: (0,0), power: None, draw: (0,0), exhaust: (false,false), slots: (0,0) },
        OrbLiteral { id: ColdSnap, cost: (1,1), enemy_hp: (-6,-9), block: (0,0), focus: (0,0), power: None, draw: (0,0), exhaust: (false,false), slots: (0,0) },
        OrbLiteral { id: CompileDriver, cost: (1,1), enemy_hp: (-7,-10), block: (0,0), focus: (0,0), power: None, draw: (0,0), exhaust: (false,false), slots: (0,0) },
        OrbLiteral { id: Coolheaded, cost: (1,1), enemy_hp: (0,0), block: (0,0), focus: (0,0), power: None, draw: (1,2), exhaust: (false,false), slots: (0,0) },
        OrbLiteral { id: Recursion, cost: (1,0), enemy_hp: (0,0), block: (0,0), focus: (0,0), power: None, draw: (0,0), exhaust: (false,false), slots: (0,0) },
        OrbLiteral { id: Blizzard, cost: (1,1), enemy_hp: (0,0), block: (0,0), focus: (0,0), power: None, draw: (0,0), exhaust: (false,false), slots: (0,0) },
        OrbLiteral { id: Capacitor, cost: (1,1), enemy_hp: (0,0), block: (0,0), focus: (0,0), power: None, draw: (0,0), exhaust: (false,false), slots: (2,3) },
        OrbLiteral { id: Chaos, cost: (1,1), enemy_hp: (0,0), block: (0,0), focus: (0,0), power: None, draw: (0,0), exhaust: (false,false), slots: (0,0) },
        OrbLiteral { id: Chill, cost: (0,0), enemy_hp: (0,0), block: (0,0), focus: (0,0), power: None, draw: (0,0), exhaust: (true,true), slots: (0,0) },
        OrbLiteral { id: Consume, cost: (2,2), enemy_hp: (0,0), block: (0,0), focus: (2,3), power: None, draw: (0,0), exhaust: (false,false), slots: (0,0) },
        OrbLiteral { id: Darkness, cost: (1,1), enemy_hp: (0,0), block: (0,0), focus: (0,0), power: None, draw: (0,0), exhaust: (false,false), slots: (0,0) },
        OrbLiteral { id: DoomAndGloom, cost: (2,2), enemy_hp: (-10,-14), block: (0,0), focus: (0,0), power: None, draw: (0,0), exhaust: (false,false), slots: (0,0) },
        OrbLiteral { id: Fusion, cost: (2,1), enemy_hp: (0,0), block: (0,0), focus: (0,0), power: None, draw: (0,0), exhaust: (false,false), slots: (0,0) },
        OrbLiteral { id: Glacier, cost: (2,2), enemy_hp: (0,0), block: (7,10), focus: (0,0), power: None, draw: (0,0), exhaust: (false,false), slots: (0,0) },
        OrbLiteral { id: Loop, cost: (1,1), enemy_hp: (0,0), block: (0,0), focus: (0,0), power: Some(("loop",1,2)), draw: (0,0), exhaust: (false,false), slots: (0,0) },
        OrbLiteral { id: StaticDischarge, cost: (1,1), enemy_hp: (0,0), block: (0,0), focus: (0,0), power: Some(("static_discharge",1,2)), draw: (0,0), exhaust: (false,false), slots: (0,0) },
        OrbLiteral { id: Storm, cost: (1,1), enemy_hp: (0,0), block: (0,0), focus: (0,0), power: Some(("storm",1,1)), draw: (0,0), exhaust: (false,false), slots: (0,0) },
        OrbLiteral { id: Tempest, cost: (10,10), enemy_hp: (0,0), block: (0,0), focus: (0,0), power: None, draw: (0,0), exhaust: (true,true), slots: (0,0) },
        OrbLiteral { id: Electrodynamics, cost: (2,2), enemy_hp: (0,0), block: (0,0), focus: (0,0), power: Some(("electrodynamics",1,1)), draw: (0,0), exhaust: (false,false), slots: (0,0) },
        OrbLiteral { id: Fission, cost: (0,0), enemy_hp: (0,0), block: (0,0), focus: (0,0), power: None, draw: (0,0), exhaust: (true,true), slots: (0,0) },
        OrbLiteral { id: MeteorStrike, cost: (5,5), enemy_hp: (-24,-30), block: (0,0), focus: (0,0), power: None, draw: (0,0), exhaust: (false,false), slots: (0,0) },
        // M20's literal zero-slot law: no front orb means no count and no X payment.
        OrbLiteral { id: MultiCast, cost: (0,0), enemy_hp: (0,0), block: (0,0), focus: (0,0), power: None, draw: (0,0), exhaust: (false,false), slots: (0,0) },
        OrbLiteral { id: Rainbow, cost: (2,2), enemy_hp: (0,0), block: (0,0), focus: (0,0), power: None, draw: (0,0), exhaust: (true,false), slots: (0,0) },
        OrbLiteral { id: ThunderStrike, cost: (3,3), enemy_hp: (0,0), block: (0,0), focus: (0,0), power: None, draw: (0,0), exhaust: (false,false), slots: (0,0) },
        OrbLiteral { id: Defragment, cost: (1,1), enemy_hp: (0,0), block: (0,0), focus: (1,2), power: None, draw: (0,0), exhaust: (false,false), slots: (0,0) },
        OrbLiteral { id: BiasedCognition, cost: (1,1), enemy_hp: (0,0), block: (0,0), focus: (4,5), power: Some(("bias",1,1)), draw: (0,0), exhaust: (false,false), slots: (0,0) },
    ]
}

fn orb_expected(row: OrbLiteral, upgraded: bool) -> PublicDelta {
    let pick = |pair: (i32, i32)| if upgraded { pair.1 } else { pair.0 };
    let pick_bool = |pair: (bool, bool)| if upgraded { pair.1 } else { pair.0 };
    let slots = if upgraded { row.slots.1 } else { row.slots.0 };
    let draw = pick(row.draw) as usize;
    let (power_id, power_amount) = row.power
        .map(|(id, base, plus)| (id, if upgraded { plus } else { base }))
        .map_or_else(|| (String::new(), 0), |(id, amount)| (id.to_owned(), amount));
    let player_powers = if power_amount == 0 { vec![] } else { vec![(power_id, power_amount)] };
    let played = CardCountChange { id: row.id, upgrades: u32::from(upgraded), count: -1 };
    let defends = CardCountChange { id: CardId::Defend, upgrades: 0, count: draw as i32 };
    let draw_change = CardCountChange { id: CardId::Defend, upgrades: 0, count: -(draw as i32) };
    let finished = CardCountChange { id: row.id, upgrades: u32::from(upgraded), count: 1 };
    let becomes_power = matches!(row.id,
        CardId::Capacitor | CardId::Loop | CardId::StaticDischarge | CardId::Storm
        | CardId::Electrodynamics | CardId::Defragment | CardId::BiasedCognition);
    let mut hand = if draw == 0 { vec![played] } else { vec![played, defends] };
    hand.sort();
    PublicDelta {
        player_hp: 0,
        player_block: pick(row.block),
        energy: -pick(row.cost),
        player_powers,
        focus: pick(row.focus),
        enemies: vec![EnemyDelta {
            id: "jaw_worm".to_owned(),
            hp: pick(row.enemy_hp),
            block: 0,
            powers: vec![],
        }],
        stance: (Stance::Neutral, Stance::Neutral),
        hand,
        draw: if draw == 0 { vec![] } else { vec![draw_change] },
        // Played Powers move to the tracked played-powers area, not a PublicDelta pile.
        discard: if pick_bool(row.exhaust) || becomes_power { vec![] } else { vec![finished.clone()] },
        exhaust: if pick_bool(row.exhaust) { vec![finished] } else { vec![] },
        created_cards: vec![],
        playable: true,
        fault: None,
        orbs: Some(vec![None; slots]),
    }
}

#[derive(Clone, Copy)]
struct GeneratorSpec { id: CardId, pool: GenerationPool }

fn generator_specs() -> [GeneratorSpec; 12] {
    use CardId::*;
    [
        GeneratorSpec { id: InfernalBlade, pool: GenerationPool::Attack },
        GeneratorSpec { id: Distraction, pool: GenerationPool::Skill },
        GeneratorSpec { id: WhiteNoise, pool: GenerationPool::Power },
        GeneratorSpec { id: CreativeAI, pool: GenerationPool::Power },
        GeneratorSpec { id: HelloWorld, pool: GenerationPool::Common },
        GeneratorSpec { id: Discovery, pool: GenerationPool::Any },
        GeneratorSpec { id: Chrysalis, pool: GenerationPool::Skill },
        GeneratorSpec { id: Metamorphosis, pool: GenerationPool::Attack },
        GeneratorSpec { id: JackOfAllTrades, pool: GenerationPool::Colorless },
        GeneratorSpec { id: Transmutation, pool: GenerationPool::Colorless },
        GeneratorSpec { id: Magnetism, pool: GenerationPool::Colorless },
        GeneratorSpec { id: ForeignInfluence, pool: GenerationPool::Attack },
    ]
}

fn expected_generator_pool(character: Character, generator: GeneratorSpec) -> Result<Vec<CardId>, String> {
    let pool = if generator.id == CardId::ForeignInfluence {
        CardPool::ForeignInfluence
    } else {
        CardPool::Combat { character, pool: generator.pool }
    };
    pools::cards(pool).map_err(|error| format!("{pool:?}: {error:?}"))
}

const SHARED_LIMITED_POTIONS: [PotionId; 29] = [
    PotionId::BlockPotion, PotionId::DexterityPotion, PotionId::EnergyPotion,
    PotionId::ExplosivePotion, PotionId::FirePotion, PotionId::StrengthPotion,
    PotionId::SwiftPotion, PotionId::WeakPotion, PotionId::FearPotion,
    PotionId::AttackPotion, PotionId::SkillPotion, PotionId::PowerPotion,
    PotionId::ColorlessPotion, PotionId::FlexPotion, PotionId::SpeedPotion,
    PotionId::BlessingOfTheForge, PotionId::RegenPotion, PotionId::AncientPotion,
    PotionId::LiquidBronze, PotionId::GamblersBrew, PotionId::EssenceOfSteel,
    PotionId::DuplicationPotion, PotionId::DistilledChaos, PotionId::LiquidMemories,
    PotionId::CultistPotion, PotionId::SneckoOil, PotionId::FairyPotion,
    PotionId::SmokeBomb, PotionId::EntropicBrew,
];

fn limited_potion_pool(character: Character) -> Vec<PotionId> {
    let class = match character {
        Character::Ironclad => [PotionId::BloodPotion, PotionId::Elixir, PotionId::HeartOfIron],
        Character::Silent => [PotionId::PoisonPotion, PotionId::CunningPotion, PotionId::GhostInAJar],
        Character::Defect => [PotionId::FocusPotion, PotionId::PotionOfCapacity, PotionId::EssenceOfDarkness],
        Character::Watcher => [PotionId::BottledMiracle, PotionId::StancePotion, PotionId::Ambrosia],
    };
    SHARED_LIMITED_POTIONS.into_iter().chain(class).collect()
}

#[test]
fn m26_cross_character_sensitive_rows() {
    let mut mismatches = Vec::new();

    for row in orb_literals() {
        for (upgraded, upgrade) in [(false, 0), (true, 1)] {
            if !complete_upgrades_valid(row.id, upgrade) { continue; }
            let expected = orb_expected(row, upgraded);
            for character in [Character::Ironclad, Character::Silent, Character::Watcher] {
                let actual = m26_support::play_delta_with_orbs(character, row.id, upgrade);
                if actual != expected {
                    mismatches.push(format!("{:?}|{upgrade}|{character:?}|{expected:?}|{actual:?}", row.id));
                }
            }
        }
    }

    // F13 generator rows and F16 limited Alchemize use the player's character pool.
    for character in CHARACTERS {
        for generator in generator_specs() {
            let expected = match expected_generator_pool(character, generator) {
                Ok(pool) => pool,
                Err(error) => {
                    mismatches.push(format!("{:?}|0|{character:?}|pool available|{error}", generator.id));
                    continue;
                }
            };
            match m26_support::generated_cards(character, generator.id, [41; 32]) {
                Ok(created) => for (id, upgrades) in created {
                    if !expected.contains(&id) {
                        mismatches.push(format!("{:?}|{upgrades}|{character:?}|member of {:?}|created {id:?}",
                            generator.id, generator.pool));
                    }
                },
                Err(error) => mismatches.push(format!("{:?}|0|{character:?}|created pool member|{error}", generator.id)),
            }
        }

        let potion_pool = limited_potion_pool(character);
        for seed in 0u8..200 {
            match m26_support::alchemize_potion(character, [seed; 32]) {
                Ok(Some(potion)) if potion_pool.contains(&potion) => {}
                Ok(Some(potion)) => mismatches.push(format!("Alchemize|0|{character:?}|member of limited class pool|{potion:?} seed={seed}")),
                Ok(None) => mismatches.push(format!("Alchemize|0|{character:?}|one potion|none seed={seed}")),
                Err(error) => mismatches.push(format!("Alchemize|0|{character:?}|limited class pool|{error} seed={seed}")),
            }
        }
    }

    assert!(mismatches.is_empty(), "m26 sensitive rows (kind|upgrades|character|expected|actual):\n{}",
        mismatches.join("\n"));
}

/// Main (M3 review): design §14.3 inertness for EVERY own-colour kind, base and upgraded,
/// not only the H0 script cards. A kind the character's base profile does not admit is
/// recorded by the baseline fault and counted, never compared.
#[test]
fn m26_cross_character_own_inert() {
    let mut mismatches = Vec::new();
    let mut compared = 0usize;
    let mut skipped = Vec::new();
    for character in CHARACTERS {
        let defend = if character == Character::Watcher { CardId::DefendPurple } else { CardId::Defend };
        for definition in Catalog::standard().cards().iter()
            .filter(|row| supports_permanent(row.id) && owner(row.id) == character
                && definition_colour_is_own(row.id, character)) {
            for upgrade in upgrades(definition.id) {
                let without = m26_support::play_delta_with_seed_and_overlay_using_defend(
                    character, definition.id, upgrade, [41; 32], false, skip_orb(definition.id), defend);
                if without.fault.as_deref().is_some_and(|f| f.starts_with("combat entry") || f.starts_with("card construction")) {
                    skipped.push(format!("{:?}+{upgrade}", definition.id)); continue;
                }
                let with = m26_support::play_delta_with_seed_and_overlay_using_defend(
                    character, definition.id, upgrade, [41; 32], true, skip_orb(definition.id), defend);
                compared += 1;
                if without.fault.is_some() || with.fault.is_some() || without != with {
                    mismatches.push(format!("{:?}|{upgrade}|{character:?}|{without:?}|{with:?}", definition.id));
                }
            }
        }
    }
    // Literal floor: red 72+Basic, green 71+, blue 71+, purple 71+ rows; base and upgraded.
    eprintln!("own-colour inertness: {compared} compared; baseline profile rejects {}: {skipped:?}", skipped.len());
    assert!(compared >= 500, "own-colour inertness compared only {compared} rows");
    assert!(mismatches.is_empty(), "own-colour overlay inertness (kind|upgrades|character|without|with):\n{}", mismatches.join("\n"));
}

fn definition_colour_is_own(id: CardId, character: Character) -> bool {
    use crate::content::CardColor::*;
    let colour = Catalog::standard().card(id).unwrap().color;
    matches!((colour, character), (Red, Character::Ironclad) | (Green, Character::Silent) | (Blue, Character::Defect) | (Purple, Character::Watcher))
}
