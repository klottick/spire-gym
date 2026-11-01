//! M25 complete-Watcher identities, catalog rows, card tags and capability (plan Task C1).
//! Every expected value is a literal from design §5.1–§5.3 (the M25 design's Watcher,
//! Foreign Influence and card-tag tables), `data/cards/cards.json` rarity/type rows,
//! `data/relics/README.md` tiers, or the content-lane contract's Binding 1/2/8, never a
//! value computed by the helper under test.
use crate::content::{
    card_tags, is_healing, is_strike, watcher, CardColor, CardId, CardRarity, CardTag, CardType,
    Catalog, Character, EncounterId, NoteStorage, RelicId, RelicTier,
};
use crate::items::CombatItemContext;
use super::super::*;
use std::collections::BTreeSet;

use CardId::*;

/// Design §5.1, by rarity and type (75 = 4/19/35/17 and 27/36/12).
const PURPLE: &[(CardRarity, CardType, &[CardId])] = &[
    (CardRarity::Basic, CardType::Attack, &[StrikePurple, Eruption]),
    (CardRarity::Basic, CardType::Skill, &[DefendPurple, Vigilance]),
    (CardRarity::Common, CardType::Attack, &[BowlingBash, Consecrate, CrushJoints, CutThroughFate, EmptyFist, FlurryOfBlows, FlyingSleeves, FollowUp, JustLucky, SashWhip]),
    (CardRarity::Common, CardType::Skill, &[Crescendo, EmptyBody, Evaluate, Halt, PressurePoints, Prostrate, Protect, ThirdEye, Tranquility]),
    (CardRarity::Uncommon, CardType::Attack, &[CarveReality, Conclude, FearNoEvil, ReachHeaven, SandsOfTime, SignatureMove, TalkToTheHand, Tantrum, Wallop, Weave, WheelKick, WindmillStrike]),
    (CardRarity::Uncommon, CardType::Skill, &[Collect, DeceiveReality, EmptyMind, ForeignInfluence, Indignation, InnerPeace, Meditate, Perseverance, Pray, Sanctity, SimmeringFury, Swivel, WaveOfTheHand, Worship, WreathOfFlame]),
    (CardRarity::Uncommon, CardType::Power, &[BattleHymn, Fasting, Foresight, LikeWater, MentalFortress, Nirvana, Rushdown, Study]),
    (CardRarity::Rare, CardType::Attack, &[Brilliance, LessonLearned, Ragnarok]),
    (CardRarity::Rare, CardType::Skill, &[Alpha, Blasphemy, ConjureBlade, DeusExMachina, Judgment, Omniscience, Scrawl, SpiritShield, Vault, Wish]),
    (CardRarity::Rare, CardType::Power, &[DevaForm, Devotion, Establishment, MasterReality]),
];

/// Design §5.3 Foreign Influence support, by colour and rarity (43/48/15 = 106).
const SUPPORT: &[(CardColor, CardRarity, &[CardId])] = &[
    (CardColor::Purple, CardRarity::Common, &[BowlingBash, Consecrate, CrushJoints, CutThroughFate, EmptyFist, FlurryOfBlows, FlyingSleeves, FollowUp, JustLucky, SashWhip]),
    (CardColor::Purple, CardRarity::Uncommon, &[CarveReality, Conclude, FearNoEvil, ReachHeaven, SandsOfTime, SignatureMove, TalkToTheHand, Tantrum, Wallop, Weave, WheelKick, WindmillStrike]),
    (CardColor::Purple, CardRarity::Rare, &[Brilliance, Ragnarok]),
    (CardColor::Red, CardRarity::Common, &[Anger, BodySlam, Clash, Cleave, Clothesline, Headbutt, HeavyBlade, IronWave, PerfectedStrike, PommelStrike, SwordBoomerang, Thunderclap, TwinStrike, WildStrike]),
    (CardColor::Red, CardRarity::Uncommon, &[BloodForBlood, Carnage, Dropkick, Hemokinesis, Pummel, Rampage, RecklessCharge, SearingBlow, SeverSoul, Uppercut, Whirlwind]),
    (CardColor::Red, CardRarity::Rare, &[Bludgeon, FiendFire, Immolate]),
    (CardColor::Green, CardRarity::Common, &[Bane, DaggerSpray, DaggerThrow, FlyingKnee, PoisonedStab, QuickSlash, Slice, SneakyStrike, SuckerPunch]),
    (CardColor::Green, CardRarity::Uncommon, &[AllOutAttack, Backstab, Choke, Dash, EndlessAgony, Eviscerate, Finisher, Flechettes, HeelHook, MasterfulStab, Predator, RiddleWithHoles, Skewer]),
    (CardColor::Green, CardRarity::Rare, &[DieDieDie, GlassKnife, GrandFinale, Unload]),
    (CardColor::Blue, CardRarity::Common, &[BallLightning, Barrage, BeamCell, Claw, ColdSnap, CompileDriver, GoForTheEyes, Rebound, Streamline, SweepingBeam]),
    (CardColor::Blue, CardRarity::Uncommon, &[Blizzard, Bullseye, DoomAndGloom, FTL, Melter, RipAndTear, Scrape, Sunder]),
    (CardColor::Blue, CardRarity::Rare, &[AllForOne, CoreSurge, Hyperbeam, MeteorStrike, ThunderStrike]),
    (CardColor::Colorless, CardRarity::Uncommon, &[DramaticEntrance, FlashOfSteel, MindBlast, SwiftStrike]),
    (CardColor::Colorless, CardRarity::Rare, &[HandOfGreed]),
];

/// Every power spelling and power-vocabulary entry in core and env at base
/// 7b036e8d (FullPowerId literals, `id.0` comparisons, power predicates and the
/// env POWERS lists). M25's new spellings must not reuse any of them.
const BASE_POWER_SPELLINGS: &[&str] = &[
    "a_thousand_cuts", "accuracy", "after_image", "amplify", "angry", "artifact", "back_attack", "barricade",
    "battle_hymn", "beat_of_death", "berserk", "bias", "blur", "brutality", "buffer", "burst",
    "cannot_change_stance", "choked", "combust", "confused", "constricted", "corpse_explosion", "corruption",
    "creative_ai", "curiosity", "curl_up", "dark_embrace", "demon_form", "devotion", "dexterity",
    "double_damage", "double_tap", "draw", "draw_card_next_turn", "draw_per_turn", "draw_reduction",
    "duplication", "echo_form", "electrodynamics", "energized", "energized_blue", "enrage", "entangled",
    "envenom", "equilibrium", "establishment", "evolve", "explosive", "fading", "feel_no_pain",
    "fire_breathing", "flame_barrier", "flight", "focus", "foresight", "frail", "heatsinks", "hello_world",
    "hex", "infinite_blades", "intangible", "intangible_player", "invincible", "juggernaut", "life_link",
    "like_water", "lock_on", "loop", "lose_dexterity", "lose_strength", "magnetism", "malleable", "mantra",
    "master_reality", "mayhem", "mental_fortress", "metallicize", "minion", "mode_shift", "next_turn_block",
    "nirvana", "no_block", "no_draw", "nonlegacy_ward", "noxious_fumes", "painful_stabs", "panache",
    "panache_countdown", "phantasmal", "plated_armor", "poison", "rage", "reactive", "rebound", "regenerate",
    "regeneration", "regrow", "repair", "retain_cards", "ritual", "rupture", "rushdown", "sadistic_nature",
    "shackled", "sharp_hide", "shifting", "slow", "split", "spore_cloud", "stasis", "static_discharge",
    "storm", "strength", "strength_up", "surrounded", "the_bomb", "the_bomb_plus", "thievery", "thorns",
    "time_warp", "tools_of_the_trade", "unawakened", "vulnerable", "weak", "wraith_form",
];

const M25_NEW_KINDS: &[CardId] = &[
    BowlingBash, Crescendo, CrushJoints, Evaluate, FlyingSleeves, FollowUp, Halt, PressurePoints, Protect, SashWhip, Tranquility,
    Collect, Conclude, Fasting, FearNoEvil, ForeignInfluence, Indignation, InnerPeace, Pray, ReachHeaven, Sanctity,
    SignatureMove, SimmeringFury, Study, Swivel, TalkToTheHand, Tantrum, Wallop, WaveOfTheHand, WheelKick, WreathOfFlame,
    Alpha, Blasphemy, ConjureBlade, DeusExMachina, DevaForm, Judgment, LessonLearned, Omniscience, Ragnarok, Scrawl, SpiritShield, Vault, Wish,
    Insight, ThroughViolence, Beta, Omega, Expunger, BecomeAlmighty, FameAndFortune, LiveForever,
];

fn set(ids: impl IntoIterator<Item = CardId>) -> BTreeSet<CardId> { ids.into_iter().collect() }

#[test]
fn m25_identity_catalog_rows_and_capability() {
    let catalog = Catalog::standard();
    let all = catalog.cards().iter().map(|row| row.id).collect::<Vec<_>>();
    assert_eq!(set(all.iter().copied()).len(), all.len(), "one catalog row per identity");

    // 75 purple ordinary rows with the design §5.1 rarity and type.
    let purple_rows = catalog.cards().iter().filter(|row| row.color == CardColor::Purple).map(|row| row.id).collect::<Vec<_>>();
    let expected_purple = set(PURPLE.iter().flat_map(|(_, _, ids)| ids.iter().copied()));
    assert_eq!(set(purple_rows.iter().copied()), expected_purple);
    assert_eq!(purple_rows.len(), 75);
    let mut rarity = [0usize; 4];
    let mut kind = [0usize; 3];
    for (expected_rarity, expected_type, ids) in PURPLE {
        for id in *ids {
            let row = catalog.card(*id).expect("purple row");
            assert_eq!((row.rarity, row.card_type, row.note_storage), (*expected_rarity, *expected_type, NoteStorage::RemovablePermanent), "{id:?}");
            rarity[match row.rarity { CardRarity::Basic => 0, CardRarity::Common => 1, CardRarity::Uncommon => 2, _ => 3 }] += 1;
            kind[match row.card_type { CardType::Attack => 0, CardType::Skill => 1, _ => 2 }] += 1;
        }
    }
    assert_eq!(rarity, [4, 19, 35, 17]);
    assert_eq!(kind, [27, 36, 12]);

    // The 8 children are generated-only and the 3 Wish options choice-only.
    let children: &[(CardId, CardType)] = &[(Miracle, CardType::Skill), (Smite, CardType::Attack), (Safety, CardType::Skill),
        (Insight, CardType::Skill), (ThroughViolence, CardType::Attack), (Beta, CardType::Skill), (Omega, CardType::Power), (Expunger, CardType::Attack)];
    assert_eq!(watcher::M25_WATCHER_GENERATED, &children.iter().map(|(id, _)| *id).collect::<Vec<_>>()[..]);
    let options: &[(CardId, CardType)] = &[(BecomeAlmighty, CardType::Power), (FameAndFortune, CardType::Skill), (LiveForever, CardType::Power)];
    assert_eq!(watcher::M25_WATCHER_CHOICES, &options.iter().map(|(id, _)| *id).collect::<Vec<_>>()[..]);
    for (id, card_type) in children.iter().chain(options) {
        let row = catalog.card(*id).expect("child or option row");
        assert_eq!((row.color, row.rarity, row.card_type, row.note_storage), (CardColor::Colorless, CardRarity::Special, *card_type, NoteStorage::TemporaryOnly), "{id:?}");
        assert!(!watcher::supports_m25_permanent(*id), "{id:?} is never permanent");
    }
    for (id, _) in children { assert!(watcher::supports_m25_generated(*id), "{id:?}"); }
    for (id, _) in options { assert!(!watcher::supports_m25_generated(*id) && !watcher::supports_generated(*id), "{id:?} is choice-only"); }

    // Capability predicates: M25 exact, M21 unchanged.
    let m25_permanent = set(all.iter().copied().filter(|id| watcher::supports_m25_permanent(*id)));
    assert_eq!(m25_permanent, expected_purple);
    assert_eq!(watcher::M25_WATCHER_CARDS.len(), 75);
    assert_eq!(&watcher::M25_WATCHER_CARDS[..28], watcher::M21_WATCHER_CARDS, "built from the M21 list");
    let m25_generated = set(all.iter().copied().filter(|id| watcher::supports_m25_generated(*id)));
    assert_eq!(m25_generated, set(expected_purple.iter().copied().chain(children.iter().map(|(id, _)| *id))));
    assert_eq!(m25_generated.len(), 83);
    let m21_permanent = set(all.iter().copied().filter(|id| watcher::supports_permanent(*id)));
    assert_eq!(m21_permanent, set([StrikePurple, DefendPurple, Eruption, Vigilance, EmptyFist, FlurryOfBlows, Prostrate, Worship, Brilliance, Devotion,
        ThirdEye, CutThroughFate, JustLucky, Weave, Foresight, Nirvana, WindmillStrike, Perseverance, SandsOfTime, Establishment, Meditate,
        CarveReality, DeceiveReality, MasterReality, BattleHymn, MentalFortress, Rushdown, LikeWater]));
    let m21_generated = set(all.iter().copied().filter(|id| watcher::supports_generated(*id)));
    assert_eq!(m21_generated, set(m21_permanent.iter().copied().chain([Miracle, Smite, Safety])));
    for id in M25_NEW_KINDS { assert!(!watcher::supports_generated(*id), "{id:?} stays outside the finite M21 profile"); }

    // One card-tag owner: six HEALING kinds (plus Bandage Up, which M26 design §5.1 adds),
    // thirteen STRIKE kinds, Defect metadata unchanged.
    assert_eq!(set(all.iter().copied().filter(|id| is_healing(*id))), set([Feed, Reaper, Alchemize, SelfRepair, LessonLearned, Wish, BandageUp]));
    assert!(!is_healing(HandOfGreed));
    assert_eq!(set(all.iter().copied().filter(|id| is_strike(*id))), set([Strike, StrikeGreen, StrikeBlue, StrikePurple, PerfectedStrike, PommelStrike,
        TwinStrike, WildStrike, SwiftStrike, SneakyStrike, ThunderStrike, MeteorStrike, WindmillStrike]));
    assert!(!is_strike(Thunderclap));
    for row in catalog.cards() {
        if let Some(metadata) = row.blue_metadata { assert_eq!(card_tags(row.id), metadata.tags, "{:?}", row.id); }
        else { assert!(card_tags(row.id).iter().all(|tag| matches!(tag, CardTag::Healing | CardTag::Strike)), "{:?}", row.id); }
    }

    // The M20 Defect and colourless generator pools are unchanged.
    use super::super::generation::{pool, GenerationPool};
    let colorless = pool(Some(Character::Watcher), GenerationPool::Colorless).expect("L34");
    let expected_l34 = ["apotheosis","blind","chrysalis","dark_shackles","deep_breath","discovery","dramatic_entrance","enlightenment","finesse","flash_of_steel","forethought","good_instincts","hand_of_greed","impatience","jack_of_all_trades","madness","magnetism","master_of_strategy","mayhem","metamorphosis","mind_blast","panacea","panache","panic_button","purity","sadistic_nature","secret_technique","secret_weapon","swift_strike","the_bomb","thinking_ahead","transmutation","trip","violence"];
    assert_eq!(colorless.iter().map(|id| id.as_str()).collect::<Vec<_>>(), expected_l34);
    for (kind, len) in [(GenerationPool::Any, 70), (GenerationPool::Attack, 23), (GenerationPool::Skill, 34), (GenerationPool::Power, 13), (GenerationPool::Common, 18)] {
        let blue = pool(Some(Character::Defect), kind).expect("M20 Defect pool");
        assert_eq!(blue.len(), len, "{kind:?}");
        assert!(blue.iter().all(|id| catalog.card(*id).unwrap().color == CardColor::Blue), "{kind:?}");
        assert!(!blue.contains(&SelfRepair), "{kind:?} excludes HEALING Self Repair");
    }

    // The four new relics: tiers and Watcher exclusivity.
    for (id, tier) in [(RelicId::Damaru, RelicTier::Common), (RelicId::Duality, RelicTier::Uncommon), (RelicId::CloakClasp, RelicTier::Rare), (RelicId::Melange, RelicTier::Shop)] {
        assert_eq!(catalog.relic_tier(id), Some(tier), "{id:?}");
        assert_eq!(catalog.relic_eligible_for(id, Character::Watcher), Some(true));
        assert_eq!(catalog.relic_eligible_for(id, Character::Ironclad), Some(false));
    }
    assert_eq!([RelicId::Damaru, RelicId::Duality, RelicId::CloakClasp, RelicId::Melange].map(|id| id.as_str()), ["damaru", "duality", "cloak_clasp", "melange"]);

    // No new identity collides with an existing spelling.
    let spellings = all.iter().map(|id| id.as_str()).collect::<BTreeSet<_>>();
    assert_eq!(spellings.len(), all.len(), "card spellings are unique");
    for id in M25_NEW_KINDS { assert!(catalog.card(*id).is_some(), "{id:?} has a catalog row"); }
    let relic_spellings = catalog_relics().iter().map(|id| id.as_str()).collect::<BTreeSet<_>>();
    assert_eq!(relic_spellings.len(), catalog_relics().len(), "relic spellings are unique");
    let powers = watcher::M25_WATCHER_POWER_IDS.iter().copied().collect::<BTreeSet<_>>();
    assert_eq!(powers.len(), 12);
    assert_eq!(powers, BTreeSet::from(["mark", "block_return", "vigor", "wave_of_the_hand", "free_attack", "simmering_fury", "study", "energy_down", "deva_form", "collect", "omega", "blasphemer"]));
    for name in &powers { assert!(!BASE_POWER_SPELLINGS.contains(name), "{name} collides with an existing power spelling"); }

    // Foreign Influence support: 106 = 43/48/15, 82 non-Watcher, HEALING excluded, catalog order.
    let support = watcher::foreign_influence_support();
    assert_eq!(set(support.iter().copied()), set(SUPPORT.iter().flat_map(|(_, _, ids)| ids.iter().copied())));
    assert_eq!(support.len(), 106);
    for (color, rarity_tier, ids) in SUPPORT {
        for id in *ids { let row = catalog.card(*id).unwrap(); assert_eq!((row.color, row.rarity, row.card_type), (*color, *rarity_tier, CardType::Attack), "{id:?}"); }
    }
    let count = |tier| support.iter().filter(|id| catalog.card(**id).unwrap().rarity == tier).count();
    assert_eq!([count(CardRarity::Common), count(CardRarity::Uncommon), count(CardRarity::Rare)], [43, 48, 15]);
    assert_eq!(support.iter().filter(|id| catalog.card(**id).unwrap().color != CardColor::Purple).count(), 82);
    for excluded in [Feed, Reaper, LessonLearned] { assert!(!support.contains(&excluded), "{excluded:?} is HEALING"); }
    assert!(support.contains(&HandOfGreed));
    let order = all.iter().copied().filter(|id| support.contains(id)).collect::<Vec<_>>();
    assert_eq!(support, &order[..], "catalog row order");
}

fn catalog_relics() -> Vec<RelicId> {
    use RelicId::*;
    [Vajra, OddlySmoothStone, BagOfPreparation, BagOfMarbles, BronzeScales, BurningBlood, RingOfTheSnake, CrackedCore, PureWater, BloodVial,
        Anchor, Lantern, Strawberry, Pear, Mango, SacredBark, MarkOfBloom, MagicFlower, ToyOrnithopter, NeowsLament, BlackBlood, EmptyCage,
        SlaversCollar, RedMask, RedSkull, PaperPhrog, ChampionsBelt, CharonsAshes, SelfFormingClay, Orichalcum, Brimstone, MarkOfPain, RunicCube,
        SneckoSkull, NinjaScroll, PaperKrane, TheSpecimen, Tingsha, ToughBandages, TwistedFunnel, HoveringKite, RingOfTheSerpent, WristBlade,
        Sozu, PotionBelt, DataDisk, GoldPlatedCables, EmotionChip, SymbioticVirus, FrozenCore, Inserter, NuclearBattery, RunicCapacitor,
        ChemicalX, VioletLotus, GoldenEye, TeardropLocket, HolyWater, Damaru, Duality, CloakClasp, Melange]
        .into_iter().filter(|id| Catalog::standard().relic(*id).is_some()).collect()
}

/// Design §5.1/§5.2 costs, base → upgraded (`None` = unplayable, `X` = -1 with the X marker).
/// The 28 accepted M21 kinds and three children keep their accepted M21 costs.
const X: Option<i32> = Some(-1);
const COSTS: &[(CardId, Option<i32>, Option<i32>)] = &[
    (StrikePurple, Some(1), Some(1)), (DefendPurple, Some(1), Some(1)), (Eruption, Some(2), Some(1)), (Vigilance, Some(2), Some(2)),
    (EmptyFist, Some(1), Some(1)), (FlurryOfBlows, Some(0), Some(0)), (Prostrate, Some(0), Some(0)), (Worship, Some(2), Some(2)),
    (Brilliance, Some(1), Some(1)), (Devotion, Some(1), Some(1)), (ThirdEye, Some(1), Some(1)), (CutThroughFate, Some(1), Some(1)),
    (JustLucky, Some(0), Some(0)), (Weave, Some(0), Some(0)), (Foresight, Some(1), Some(1)), (Nirvana, Some(1), Some(1)),
    (WindmillStrike, Some(2), Some(2)), (Perseverance, Some(1), Some(1)), (SandsOfTime, Some(4), Some(4)), (Establishment, Some(1), Some(1)),
    (Meditate, Some(1), Some(1)), (CarveReality, Some(1), Some(1)), (DeceiveReality, Some(1), Some(1)), (MasterReality, Some(1), Some(0)),
    (BattleHymn, Some(1), Some(1)), (MentalFortress, Some(1), Some(1)), (Rushdown, Some(1), Some(0)), (LikeWater, Some(1), Some(1)),
    (BowlingBash, Some(1), Some(1)), (Consecrate, Some(0), Some(0)), (Crescendo, Some(1), Some(0)), (CrushJoints, Some(1), Some(1)),
    (EmptyBody, Some(1), Some(1)), (Evaluate, Some(1), Some(1)), (FlyingSleeves, Some(1), Some(1)), (FollowUp, Some(1), Some(1)),
    (Halt, Some(0), Some(0)), (PressurePoints, Some(1), Some(1)), (Protect, Some(2), Some(2)), (SashWhip, Some(1), Some(1)),
    (Tranquility, Some(1), Some(0)),
    (Collect, X, X), (Conclude, Some(1), Some(1)), (EmptyMind, Some(1), Some(1)), (Fasting, Some(2), Some(2)), (FearNoEvil, Some(1), Some(1)),
    (ForeignInfluence, Some(0), Some(0)), (Indignation, Some(1), Some(1)), (InnerPeace, Some(1), Some(1)), (Pray, Some(1), Some(1)),
    (ReachHeaven, Some(2), Some(2)), (Sanctity, Some(1), Some(1)), (SignatureMove, Some(2), Some(2)), (SimmeringFury, Some(1), Some(1)),
    (Study, Some(2), Some(1)), (Swivel, Some(2), Some(2)), (TalkToTheHand, Some(1), Some(1)), (Tantrum, Some(1), Some(1)),
    (Wallop, Some(2), Some(2)), (WaveOfTheHand, Some(1), Some(1)), (WheelKick, Some(2), Some(2)), (WreathOfFlame, Some(1), Some(1)),
    (Alpha, Some(1), Some(1)), (Blasphemy, Some(1), Some(1)), (ConjureBlade, X, X), (DeusExMachina, None, None), (DevaForm, Some(3), Some(3)),
    (Judgment, Some(1), Some(1)), (LessonLearned, Some(2), Some(2)), (Omniscience, Some(4), Some(3)), (Ragnarok, Some(3), Some(3)),
    (Scrawl, Some(1), Some(0)), (SpiritShield, Some(2), Some(2)), (Vault, Some(3), Some(2)), (Wish, Some(3), Some(3)),
    (Miracle, Some(0), Some(0)), (Smite, Some(1), Some(1)), (Safety, Some(1), Some(1)),
    (Insight, Some(0), Some(0)), (ThroughViolence, Some(0), Some(0)), (Beta, Some(2), Some(1)), (Omega, Some(3), Some(3)), (Expunger, Some(1), Some(1)),
];

/// Accepted Watcher entry (m24_watcher_act3.rs `m24_curse_parasite_…`): actual setup via
/// `m8_shapes::entry`, character Watcher, permanents built with the shared constructor as
/// Run decks are, and the profile initializer applied at combat entry.
fn enter(context: fn(crate::items::CombatItemContext) -> crate::items::CombatItemContext, deck: &[(CardId, u32)]) -> Result<super::super::Combat, super::super::CombatEntryFailure> {
    use super::super::{permanent_live_card, CardInstanceId, Combat};
    let mut e = super::m8_shapes::entry(crate::content::EncounterId::OrbWalker);
    e.setup.character = Character::Watcher;
    e.cards = deck.iter().enumerate().map(|(i, (id, up))| permanent_live_card(*id, CardInstanceId(i as u64 + 1), *up)).collect();
    e.opening_draw = deck.len().min(10);
    let mut items = Some(context(crate::items::CombatItemContext::new(vec![RelicId::PureWater], vec![None; 3])));
    Combat::enter_with_item_context(e, &mut items)
}

#[test]
fn m25_identity_construction_admission() {
    use crate::items::CombatItemContext as Items;
    assert_eq!(set(COSTS.iter().map(|(id, _, _)| *id)), set(watcher::M25_WATCHER_CARDS.iter().chain(watcher::M25_WATCHER_GENERATED).copied()));
    assert_eq!(COSTS.len(), 83);
    // Under M25 every kind constructs, base and upgraded, with its cost and Watcher initialization.
    let deck = COSTS.iter().flat_map(|(id, _, _)| [(*id, 0), (*id, 1)]).chain([(Parasite, 0)]).collect::<Vec<_>>();
    let combat = enter(Items::with_m25_profile, &deck).expect("the complete Watcher roster enters an M25 combat");
    for (index, (id, up)) in deck.iter().enumerate() {
        let card = super::super::piles::owned_full_cards(&combat.state).find(|card| card.instance.0 == index as u64 + 1).expect("entered card");
        assert_eq!((card.id, card.upgrades), (*id, *up));
        if *id == Parasite {
            assert!(!card.local.m21 && card.local.mutable_combat_cost.is_none(), "a Curse keeps the shared unplayable fields and no Watcher state");
            continue;
        }
        let (_, base, upgraded) = COSTS.iter().find(|(kind, _, _)| kind == id).unwrap();
        assert!(card.local.m21, "{id:?}+{up} has Watcher initialization");
        assert_eq!(card.local.mutable_combat_cost, if *up == 0 { *base } else { *upgraded }, "{id:?}+{up} cost");
        assert_eq!(card.local.turn_cost, card.local.mutable_combat_cost, "{id:?}+{up} turn price");
        assert_eq!(card.local.x_cost, matches!(id, Collect | ConjureBlade), "{id:?} X marker");
    }
    // Choice-only options, off-colour permanents and starting Statuses still reject under M25.
    for rejected in [BecomeAlmighty, FameAndFortune, LiveForever, Strike, Dazed] {
        assert!(enter(Items::with_m25_profile, &[(rejected, 0)]).is_err(), "{rejected:?} rejects under M25");
    }
    // The finite M22/M23 and M24 contexts still reject every M25 addition; Statuses reject everywhere.
    let finite: [fn(Items) -> Items; 2] = [Items::with_m22_profile, Items::with_m24_profile];
    for context in finite {
        for id in M25_NEW_KINDS.iter().filter(|id| !watcher::M25_WATCHER_CHOICES.contains(id)).chain(&[Consecrate, EmptyBody, EmptyMind]) {
            assert!(enter(context, &[(*id, 0)]).is_err(), "{id:?} stays outside the finite profile");
        }
        assert!(enter(context, &[(Dazed, 0)]).is_err(), "a starting Status rejects");
        assert!(enter(context, &[(Parasite, 0), (StrikePurple, 0)]).is_ok(), "M24 Curse law and M21 kinds unchanged");
    }
}

fn generation_watcher(e: CombatEntry, deck: &[(CardId, u32)], energy: i32, relics: Vec<RelicId>) -> Combat {
    generation_watcher_opening_draw(e, deck, energy, relics, deck.len().min(10))
}

fn generation_watcher_opening_draw(mut e: CombatEntry, deck: &[(CardId, u32)], energy: i32, relics: Vec<RelicId>, opening_draw: usize) -> Combat {
    e.setup.character = Character::Watcher;
    e.cards = deck.iter().enumerate().map(|(i, (id, up))|
        m25_watcher_live_card(*id, CardInstanceId(i as u64 + 1), *up).unwrap()).collect();
    e.opening_draw = opening_draw;
    e.setup.player.energy = energy;
    e.setup.player.max_energy = energy;
    e.setup.player.draw_per_turn = 1;
    let mut items = Some(CombatItemContext::new(relics, vec![None; 3]).with_m25_profile());
    let combat = Combat::enter_with_item_context(e, &mut items).unwrap();
    assert!(items.is_none());
    combat
}

fn gen_entry(encounter: EncounterId) -> CombatEntry { super::m8_shapes::entry(encounter) }

fn gen_play(combat: &mut Combat, instance: u64, target: Option<u64>) {
    combat.step_full(&CombatAction::PlayCard { instance, target: target.map(TargetId) })
        .unwrap_or_else(|error| panic!("play {instance}: {error:?}; legal={:?}; fault={:?}", combat.legal_actions_full(), combat.fault_record()));
}

fn gen_place(combat: &mut Combat, instance: u64, zone: SelectionSourceZone) {
    let handle = card_state::handle(CardInstanceId(instance));
    let take = |pile: &mut Vec<model::OwnedCard>| pile.iter().position(|card| card.handle() == handle).map(|i| pile.remove(i));
    let card = take(&mut combat.state.piles.hand)
        .or_else(|| take(&mut combat.state.piles.draw))
        .or_else(|| take(&mut combat.state.piles.discard))
        .or_else(|| take(&mut combat.state.piles.exhaust))
        .expect("requested test card exists");
    match zone {
        SelectionSourceZone::Hand => combat.state.piles.hand.push(card),
        SelectionSourceZone::Draw => combat.state.piles.draw.push(card),
        SelectionSourceZone::Discard => combat.state.piles.discard.push(card),
        _ => unreachable!(),
    }
    piles::refresh_hand_values(&mut combat.state);
}

fn gen_cards(combat: &Combat, id: CardId) -> Vec<&card_state::LiveCard> {
    piles::owned_full_cards(&combat.state).filter(|card| card.id == id).collect()
}

fn gen_power(combat: &Combat, id: &str) -> i32 {
    combat.state.player_full_powers.iter().find(|power| power.id.0 == id).map_or(0, |power| power.state.amount)
}

#[test]
fn m25_generation_children_and_options() {
    // Evaluate and Pray create a base Insight after their own payload resolves.
    for up in [0, 1] {
        let mut evaluate = generation_watcher(gen_entry(EncounterId::OrbWalker), &[(Evaluate, up)], 9, vec![]);
        let before_block = evaluate.state.player.block;
        gen_play(&mut evaluate, 1, None);
        assert_eq!(evaluate.state.player.block - before_block, if up == 0 { 6 } else { 10 });
        let insight = gen_cards(&evaluate, Insight);
        assert_eq!(insight.len(), 1);
        assert_eq!((insight[0].upgrades, insight[0].local.magic_number, insight[0].local.exhaust_on_finish, insight[0].local.self_retain), (0, Some(2), true, true));

        let mut pray = generation_watcher(gen_entry(EncounterId::OrbWalker), &[(Pray, up)], 9, vec![]);
        gen_play(&mut pray, 1, None);
        assert_eq!(pray.state.character.mantra_gained, if up == 0 { 3 } else { 4 });
        let insight = gen_cards(&pray, Insight);
        assert_eq!(insight.len(), 1);
        assert_eq!(insight[0].upgrades, 0);
    }

    // Study stacks once per Power and creates at PlayerEndTurn, before the enemy acts.
    let mut study = generation_watcher(gen_entry(EncounterId::OrbWalker), &[(Study, 0), (Study, 1)], 9, vec![]);
    gen_play(&mut study, 1, None);
    gen_play(&mut study, 2, None);
    assert_eq!(gen_power(&study, "study"), 2);
    study.step_full(&CombatAction::EndTurn).unwrap();
    let insights = gen_cards(&study, Insight);
    assert_eq!(insights.len(), 2);
    assert!(insights.iter().all(|card| card.upgrades == 0));

    // Reach Heaven, Alpha and Beta create the next base child in the draw pile.
    for up in [0, 1] {
        let mut reach = generation_watcher(gen_entry(EncounterId::OrbWalker), &[(ReachHeaven, up)], 9, vec![]);
        let hp = reach.state.enemies[0].hp;
        gen_play(&mut reach, 1, Some(1));
        assert_eq!(hp - reach.state.enemies[0].hp, if up == 0 { 10 } else { 15 });
        let violence = gen_cards(&reach, ThroughViolence);
        assert_eq!(violence.len(), 1);
        assert_eq!((violence[0].upgrades, violence[0].local.base_damage, violence[0].local.magic_number, violence[0].local.exhaust_on_finish, violence[0].local.self_retain), (0, Some(20), None, true, true));
    }
    let mut alpha = generation_watcher(gen_entry(EncounterId::OrbWalker), &[(Alpha, 0)], 9, vec![]);
    gen_play(&mut alpha, 1, None);
    let beta = *gen_cards(&alpha, Beta).first().expect("Alpha creates Beta");
    assert_eq!((beta.upgrades, beta.local.mutable_combat_cost, beta.local.exhaust_on_finish), (0, Some(2), true));
    let beta_instance = beta.instance.0;
    gen_place(&mut alpha, beta_instance, SelectionSourceZone::Hand);
    gen_play(&mut alpha, beta_instance, None);
    let omega = *gen_cards(&alpha, Omega).first().expect("Beta creates Omega");
    assert_eq!((omega.upgrades, omega.local.mutable_combat_cost, omega.local.magic_number), (0, Some(3), Some(50)));

    // Omega's end-turn damage is non-attack damage: no Strength, Vulnerable,
    // stance multiplier or Double Damage applies to the source amount.
    let mut omega_combat = generation_watcher(gen_entry(EncounterId::FourShapes), &[(Omega, 1)], 9, vec![]);
    omega_combat.state.character.stance = Stance::Divinity;
    omega_combat.state.player_full_powers.push(FullPowerEntry { id: FullPowerId("strength"), state: PowerState { amount: 20, skip_first: false, monster_source_grace: false } });
    omega_combat.state.player_full_powers.push(FullPowerEntry { id: FullPowerId("double_damage"), state: PowerState { amount: 1, skip_first: false, monster_source_grace: false } });
    for enemy in &mut omega_combat.state.enemies {
        enemy.hp = 200;
        enemy.block = 0;
        enemy.full_powers.push(FullPowerEntry { id: FullPowerId("vulnerable"), state: PowerState { amount: 2, skip_first: false, monster_source_grace: false } });
    }
    gen_play(&mut omega_combat, 1, None);
    let before = omega_combat.state.enemies.iter().map(|enemy| (enemy.hp, enemy.block)).collect::<Vec<_>>();
    omega_combat.enqueue_hook(HookEvent { hook: Hook::PlayerEndTurn, source: Source::Actor(Actor::Player), target: Some(Actor::Player) }).unwrap();
    assert_eq!(omega_combat.state.enemies.iter().zip(before).map(|(enemy, (hp, block))|
        hp - enemy.hp + block - enemy.block).collect::<Vec<_>>(), vec![60, 60, 60, 60]);

    // Conjure Blade stores its X (+ upgrade) on one Expunger; Expunger reuses
    // normal mutable Attack damage and its own upgrade only changes per-hit damage.
    for (up, hits) in [(0, 3), (1, 4)] {
        let mut conjure = generation_watcher(gen_entry(EncounterId::OrbWalker), &[(ConjureBlade, up)], 3, vec![]);
        gen_play(&mut conjure, 1, None);
        let expunger = *gen_cards(&conjure, Expunger).first().expect("Conjure Blade creates Expunger");
        assert_eq!((expunger.misc, expunger.local.base_damage, expunger.upgrades), (hits, Some(9), 0));
        let instance = expunger.instance.0;
        gen_place(&mut conjure, instance, SelectionSourceZone::Hand);
        conjure.state.player.energy = 1;
        let hp = conjure.state.enemies[0].hp;
        gen_play(&mut conjure, instance, Some(1));
        assert_eq!(hp - conjure.state.enemies[0].hp, hits * 9);
    }

    // Wish's shared offer is mandatory and ordered. Options inherit Wish's
    // upgrade, Master Reality does not upgrade them, and none becomes a card.
    for up in [0, 1] {
        for (index, option) in [BecomeAlmighty, FameAndFortune, LiveForever].into_iter().enumerate() {
            let mut wish = generation_watcher(gen_entry(EncounterId::OrbWalker), &[(Wish, up)], 9, vec![]);
            wish.state.player_full_powers.push(FullPowerEntry { id: FullPowerId("master_reality"), state: PowerState { amount: -1, skip_first: false, monster_source_grace: false } });
            gen_play(&mut wish, 1, None);
            let offer = wish.state.silent.card_offer.as_ref().expect("Wish opens a mandatory shared offer");
            assert_eq!(offer.cards, vec![BecomeAlmighty, FameAndFortune, LiveForever]);
            assert!(!offer.optional);
            let runtime = engine::validate_runtime(&wish.state);
            assert!(runtime.is_ok(), "Wish choice remains a valid selection state: {runtime:?}");
            let delivery = Effect { source: offer.source.clone(), target: Some(Actor::Player), kind: EffectKind::DeliverCardOffer { id: option, copies: 1, next: 0, zero_cost: false } };
            let delivery_validation = card_offers::validate_work(&wish.state, &delivery);
            assert!(delivery_validation.is_ok(), "Wish option delivery validates through the shared offer owner: {delivery_validation:?}");
            // FameAndFortune gold goes through the shared combat gold owner
            // (thieves::gain_gold -> current_combat_gold), admitted under M25.
            wish.step_full(&CombatAction::ChooseCardOffer { index: index as u32 }).unwrap();
            assert!(gen_cards(&wish, BecomeAlmighty).is_empty() && gen_cards(&wish, FameAndFortune).is_empty() && gen_cards(&wish, LiveForever).is_empty());
            match option {
                BecomeAlmighty => assert_eq!(gen_power(&wish, "strength"), if up == 0 { 3 } else { 4 }),
                FameAndFortune => assert_eq!(wish.current_combat_gold(), Some(if up == 0 { 25 } else { 30 })),
                LiveForever => {
                    let armor = if up == 0 { 6 } else { 8 };
                    assert_eq!(gen_power(&wish, "plated_armor"), armor);
                    let block = wish.state.player.block;
                    wish.enqueue_hook(HookEvent { hook: Hook::PlayerPreEndTurnCards, source: Source::Actor(Actor::Player), target: Some(Actor::Player) }).unwrap();
                    assert_eq!(wish.state.player.block - block, armor, "Plated Armor's shared end-turn hook admits Watcher");
                }
                _ => unreachable!("the fixed Wish choice list contains only the three options"),
            }
        }
    }
}

#[test]
fn m25_generation_all_watcher_cards_base_plus_census() {
    use crate::items::CombatItemContext as Items;
    let identities = watcher::M25_WATCHER_CARDS.iter().chain(watcher::M25_WATCHER_GENERATED).copied().collect::<BTreeSet<_>>();
    assert_eq!(identities.len(), 83);
    assert_eq!(identities, set(COSTS.iter().map(|(id, _, _)| *id)));
    assert_eq!(COSTS.len(), 83, "design §5.1/§5.2 literal cost census");
    let deck = COSTS.iter().flat_map(|(id, _, _)| [(*id, 0), (*id, 1)]).collect::<Vec<_>>();
    let combat = enter(Items::with_m25_profile, &deck).expect("every permanent and generated kind enters under M25");
    for (index, (id, up)) in deck.iter().enumerate() {
        let instance = index as u64 + 1;
        let card = piles::owned_full_cards(&combat.state).find(|card| card.instance.0 == instance).expect("census card is owned");
        let (_, base, upgraded) = COSTS.iter().find(|(kind, _, _)| kind == id).unwrap();
        let cost = if *up == 0 { *base } else { *upgraded };
        assert_eq!((card.id, card.upgrades, card.local.mutable_combat_cost, card.local.turn_cost), (*id, *up, cost, cost), "{id:?}+{up} cost");
        let public = cards::public_values(card).expect("M25 card exposes public values");
        assert_eq!((public.base_damage, public.base_block, public.magic_number), (card.local.base_damage, card.local.base_block, card.local.magic_number), "{id:?}+{up} public values");
        assert_eq!(public.cost_kind, if matches!(id, Collect | ConjureBlade) { CardCostKind::X } else { CardCostKind::Fixed }, "{id:?}+{up} cost kind");
        if *up == 0 {
            let temporary = card_state::upgrade_live_card(&combat.state, card).unwrap_or_else(|error| panic!("temporary upgrade {id:?}: {error:?}"));
            let expected = m25_watcher_live_card(*id, CardInstanceId(20_000 + instance), 1).unwrap();
            assert_eq!((temporary.upgrades, temporary.local.temporary_upgrades, temporary.local.base_damage, temporary.local.base_block, temporary.local.magic_number, temporary.local.mutable_combat_cost),
                (1, 1, expected.local.base_damage, expected.local.base_block, expected.local.magic_number, expected.local.mutable_combat_cost), "{id:?} temporary upgrade");
            let mut source = card.clone();
            let copy = cards::make_stat_equivalent_copy(&mut source, CardInstanceId(30_000 + instance), cards::copy_history(&combat.state)).unwrap();
            assert_eq!((copy.id, copy.upgrades, copy.local.base_damage, copy.local.base_block, copy.local.magic_number, copy.local.mutable_combat_cost),
                (card.id, card.upgrades, card.local.base_damage, card.local.base_block, card.local.magic_number, card.local.mutable_combat_cost), "{id:?} stat-equivalent copy");
        }
    }
    // The older M21-backed profiles keep their exact admission boundary and
    // still accept the three original generated children.
    assert!(enter(Items::with_m22_profile, &[(Miracle, 0), (Smite, 0), (Safety, 0)]).is_ok());
    for id in [Insight, ThroughViolence, Beta, Omega, Expunger, BecomeAlmighty, FameAndFortune, LiveForever] {
        assert!(enter(Items::with_m22_profile, &[(id, 0)]).is_err(), "M21-backed profile still rejects {id:?}");
    }
}

#[test]
fn m25_generation_full_hand_overflow() {
    // Deus Ex Machina is drawn into a ten-card hand. Its Miracles overflow to
    // discard. Master Reality was installed before the creation action was built,
    // so MakeTempCardInHandAction's constructor check upgrades the overflow copies
    // too (amended after final review v1 I-1; pre-review asserted upgrades == 0,
    // original bytes retained in the M25 evidence fault-I1/original-bytes).
    let mut deck = vec![(DeusExMachina, 0), (MasterReality, 0)];
    deck.extend((0..8).map(|_| (StrikePurple, 0)));
    let mut deus = enter(CombatItemContext::with_m25_profile, &deck).expect("ten-card Watcher setup enters");
    gen_play(&mut deus, 2, None);
    let deus_instance = gen_cards(&deus, DeusExMachina).first().unwrap().instance;
    gen_place(&mut deus, deus_instance.0, SelectionSourceZone::Draw);
    let mut filler = 20;
    while deus.state.piles.hand.len() < 9 {
        deus.state.piles.hand.push(model::OwnedCard::Full(m25_watcher_live_card(DefendPurple, CardInstanceId(filler), 0).unwrap()));
        deus.state.card_ids_issued.insert(card_state::handle(CardInstanceId(filler)));
        filler += 1;
    }
    assert_eq!(deus.state.piles.hand.len(), 9);
    deus.enqueue_native_effects(vec![Effect { source: Source::Actor(Actor::Player), target: Some(Actor::Player), kind: EffectKind::Draw(1) }], None).unwrap();
    let miracles = deus.state.piles.discard.iter().filter_map(model::OwnedCard::full).filter(|card| card.id == Miracle).collect::<Vec<_>>();
    assert_eq!(miracles.len(), 2);
    assert!(miracles.iter().all(|card| card.upgrades == 1), "construction-time Master Reality upgrades overflowed Miracles");
    assert!(deus.state.piles.exhaust.iter().any(|card| card.handle()==card_state::handle(deus_instance)), "drawn Deus Ex Machina exhausts");

    // Collect's scheduled Miracle+ also reaches the discard at the full-hand
    // boundary and remains a single source upgrade.
    let mut collect = generation_watcher(gen_entry(EncounterId::OrbWalker), &[(MasterReality, 0), (Collect, 1)], 3, vec![]);
    gen_play(&mut collect, 1, None);
    collect.state.player.energy = 1;
    gen_play(&mut collect, 2, None);
    for instance in 10..20 {
        collect.state.piles.hand.push(model::OwnedCard::Full(m25_watcher_live_card(DefendPurple, CardInstanceId(instance), 0).unwrap()));
        collect.state.card_ids_issued.insert(card_state::handle(CardInstanceId(instance)));
    }
    collect.state.turn = 2;
    collect.enqueue_hook(HookEvent { hook: Hook::TurnStartedPreDraw, source: Source::Actor(Actor::Player), target: Some(Actor::Player) }).unwrap();
    let overflow = collect.state.piles.discard.iter().filter_map(model::OwnedCard::full).find(|card| card.id == Miracle).expect("Collect Miracle+ overflows");
    assert_eq!(overflow.upgrades, 1);
    assert_eq!(collect.state.piles.hand.len(), 10);

    // All draw-pile producers ignore a full hand and use the shared shuffled
    // draw insertion route.
    let mut draw = generation_watcher(gen_entry(EncounterId::OrbWalker), &[(Evaluate, 0), (ReachHeaven, 0), (Alpha, 0), (ConjureBlade, 0)], 9, vec![]);
    for instance in 10..16 {
        draw.state.piles.hand.push(model::OwnedCard::Full(m25_watcher_live_card(DefendPurple, CardInstanceId(instance), 0).unwrap()));
        draw.state.card_ids_issued.insert(card_state::handle(CardInstanceId(instance)));
    }
    gen_play(&mut draw, 1, None);
    gen_play(&mut draw, 2, Some(1));
    gen_play(&mut draw, 3, None);
    draw.state.player.energy = 0;
    gen_play(&mut draw, 4, None);
    assert_eq!(draw.state.piles.hand.len(), 6);
    for id in [Insight, ThroughViolence, Beta, Expunger] {
        assert_eq!(draw.state.piles.draw.iter().filter_map(model::OwnedCard::full).filter(|card| card.id == id).count(), 1, "{id:?} still enters draw pile at hand cap");
    }
    let omega_child = draw.state.piles.draw.iter().filter_map(model::OwnedCard::full).find(|card| card.id == Beta).unwrap().instance.0;
    gen_place(&mut draw, omega_child, SelectionSourceZone::Hand);
    draw.state.player.energy = 2;
    gen_play(&mut draw, omega_child, None);
    assert_eq!(draw.state.piles.draw.iter().filter_map(model::OwnedCard::full).filter(|card| card.id == Omega).count(), 1);
}
