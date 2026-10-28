use crate::{
    combat::rng::Partition,
    content::{CardColor, CardId, CardRarity, Catalog, PotionId, RelicId},
    run::{self, rooms::m15_content},
};
use std::collections::BTreeSet;

#[test]
fn m15_green_candidates_match_the_source_roster_and_canonical_order() {
    let expected = [
        (
            CardRarity::Basic,
            &[CardId::DefendGreen, CardId::Neutralize, CardId::StrikeGreen, CardId::Survivor][..],
        ),
        (
            CardRarity::Common,
            &[
                CardId::Acrobatics, CardId::Backflip, CardId::Bane, CardId::BladeDance,
                CardId::CloakAndDagger, CardId::DaggerSpray, CardId::DaggerThrow,
                CardId::DeadlyPoison, CardId::Deflect, CardId::DodgeAndRoll,
                CardId::FlyingKnee, CardId::Outmaneuver, CardId::PiercingWail,
                CardId::PoisonedStab, CardId::Prepared, CardId::QuickSlash,
                CardId::Slice, CardId::SneakyStrike, CardId::SuckerPunch,
            ][..],
        ),
        (
            CardRarity::Uncommon,
            &[
                CardId::Accuracy, CardId::AllOutAttack, CardId::Backstab, CardId::Blur,
                CardId::BouncingFlask, CardId::CalculatedGamble, CardId::Caltrops,
                CardId::Catalyst, CardId::Choke, CardId::Concentrate,
                CardId::CripplingCloud, CardId::Dash, CardId::Distraction,
                CardId::EndlessAgony, CardId::EscapePlan, CardId::Eviscerate,
                CardId::Expertise, CardId::Finisher, CardId::Flechettes,
                CardId::Footwork, CardId::HeelHook, CardId::InfiniteBlades,
                CardId::LegSweep, CardId::MasterfulStab, CardId::NoxiousFumes,
                CardId::Predator, CardId::Reflex, CardId::RiddleWithHoles,
                CardId::Setup, CardId::Skewer, CardId::Tactician, CardId::Terror,
                CardId::WellLaidPlans,
            ][..],
        ),
        (
            CardRarity::Rare,
            &[
                CardId::AThousandCuts, CardId::Adrenaline, CardId::AfterImage,
                CardId::Alchemize, CardId::BulletTime, CardId::Burst,
                CardId::CorpseExplosion, CardId::DieDieDie, CardId::Doppelganger,
                CardId::Envenom, CardId::GlassKnife, CardId::GrandFinale,
                CardId::Malaise, CardId::Nightmare, CardId::PhantasmalKiller,
                CardId::StormOfSteel, CardId::ToolsOfTheTrade, CardId::Unload,
                CardId::WraithForm,
            ][..],
        ),
    ];

    let catalog = Catalog::standard();
    for (rarity, reference) in expected {
        let actual = m15_content::green_candidates(rarity);
        assert_eq!(actual, reference, "wrong {rarity:?} Green roster or order");
        assert!(actual.windows(2).all(|pair| pair[0].as_str() < pair[1].as_str()));
        assert_eq!(actual.iter().copied().collect::<BTreeSet<_>>().len(), actual.len());
        for id in actual {
            let card = catalog.card(*id).unwrap_or_else(|| panic!("missing {id:?}"));
            assert_eq!(card.color, CardColor::Green, "{id:?}");
            assert_eq!(card.rarity, rarity, "{id:?}");
        }
    }
    assert_eq!(m15_content::green_candidates(CardRarity::Special), &[]);
    assert_eq!(m15_content::green_candidates(CardRarity::Status), &[]);
    assert_eq!(m15_content::green_candidates(CardRarity::Curse), &[]);
}

#[test]
fn m15_card_admission_keeps_all_permanent_green_ids_and_alchemize_reward_eligibility() {
    let basics = m15_content::green_candidates(CardRarity::Basic);
    let commons = m15_content::green_candidates(CardRarity::Common);
    let uncommons = m15_content::green_candidates(CardRarity::Uncommon);
    let rares = m15_content::green_candidates(CardRarity::Rare);
    assert_eq!((basics.len(), commons.len(), uncommons.len(), rares.len()), (4, 19, 33, 19));

    let all_green = basics.iter().chain(commons).chain(uncommons).chain(rares).copied().collect::<BTreeSet<_>>();
    assert_eq!(all_green.len(), 75);
    assert_eq!(m15_content::M15_GREEN_PERMANENT_CARDS.len(), 75);
    assert_eq!(m15_content::M15_GREEN_PERMANENT_CARDS.iter().copied().collect::<BTreeSet<_>>(), all_green);

    let ordinary = m15_content::M15_ORDINARY_GREEN_CARDS;
    assert_eq!(ordinary.len(), 71);
    assert_eq!(ordinary.iter().copied().collect::<BTreeSet<_>>(), commons.iter().chain(uncommons).chain(rares).copied().collect());
    assert!(ordinary.contains(&CardId::Alchemize), "HEALING excludes combat generation only");
    for excluded in [CardId::DefendGreen, CardId::Neutralize, CardId::StrikeGreen, CardId::Survivor, CardId::Parasite, CardId::Shiv] {
        assert!(!ordinary.contains(&excluded), "{excluded:?} is not an ordinary Green reward");
    }

    let permanent = m15_content::M15_PERMANENT_CARDS;
    assert_eq!(permanent.len(), 76);
    assert!(permanent.contains(&CardId::Parasite));
    assert!(!permanent.contains(&CardId::Shiv));
    assert_eq!(permanent.iter().copied().collect::<BTreeSet<_>>(), all_green.into_iter().chain([CardId::Parasite]).collect());

    let definition_only = m15_content::green_candidates(CardRarity::Rare)
        .iter()
        .copied()
        .filter(|id| Catalog::standard().card(*id).unwrap().definition_only)
        .collect::<Vec<_>>();
    assert!(definition_only.is_empty(), "all19 Rare bodies, including actual Alchemize, are now executable");
}

#[test]
fn m15_relic_and_potion_admission_tables_are_exact_and_bounded() {
    let stock = [
        RelicId::Lantern, RelicId::Pear, RelicId::Mango, RelicId::ToyOrnithopter,
        RelicId::Strawberry, RelicId::Anchor, RelicId::BloodVial, RelicId::Vajra,
        RelicId::OddlySmoothStone, RelicId::BagOfPreparation, RelicId::BagOfMarbles,
        RelicId::BronzeScales, RelicId::Orichalcum, RelicId::SneckoSkull,
        RelicId::NinjaScroll, RelicId::PaperKrane, RelicId::TheSpecimen,
        RelicId::Tingsha, RelicId::ToughBandages,
    ];
    assert_eq!(m15_content::M15_RELIC_STOCK, stock);
    assert_eq!(m15_content::M15_RELIC_STOCK.len(), 19);
    assert_eq!(m15_content::M15_BOSS_RELICS, [
        RelicId::EmptyCage, RelicId::SlaversCollar, RelicId::RingOfTheSerpent,
        RelicId::HoveringKite, RelicId::WristBlade,
    ]);
    assert_eq!(m15_content::M15_RELICS28.len(), 28);
    assert_eq!(m15_content::M15_RELICS28.iter().copied().collect::<BTreeSet<_>>().len(), 28);
    assert_eq!(m15_content::M15_RELICS28.iter().copied().collect::<BTreeSet<_>>(), stock.into_iter().chain([
        RelicId::RingOfTheSnake, RelicId::NeowsLament, RelicId::EmptyCage,
        RelicId::SlaversCollar, RelicId::RedMask, RelicId::TwistedFunnel,
        RelicId::RingOfTheSerpent, RelicId::HoveringKite, RelicId::WristBlade,
    ]).collect());
    assert_eq!(m15_content::M15_POTION_CYCLE, [
        PotionId::FirePotion, PotionId::StrengthPotion, PotionId::WeakPotion,
        PotionId::PoisonPotion, PotionId::CunningPotion, PotionId::GhostInAJar,
    ]);
}

#[test]
fn m15_shared_relic_cursor_skips_owned_items_without_refill_and_faults_at_exhaustion() {
    use m15_content::M15_RELIC_STOCK as stock;

    let mut owned = Vec::new();
    let mut cursor = 0;
    let mut acquired = Vec::new();
    for expected in stock {
        let (relic, next) = m15_content::capture_relic(cursor, &owned).unwrap();
        assert_eq!(relic, expected);
        assert_eq!(next, cursor + 1);
        assert!(!acquired.contains(&relic));
        acquired.push(relic);
        owned.push(relic);
        cursor = next;
    }
    assert_eq!(cursor, 19);
    assert!(matches!(m15_content::capture_relic(cursor, &owned), Err(run::RunError::Unsupported(_))));
    assert_eq!(m15_content::capture_relic(0, &owned).unwrap_err(), run::RunError::Unsupported("M15 normal relic stock exhausted"));
    assert!(matches!(m15_content::capture_relic(19, &[]), Err(run::RunError::Unsupported(_))));

    assert_eq!(m15_content::capture_relic(0, &[RelicId::Lantern, RelicId::Pear]).unwrap(), (RelicId::Mango, 3));
    assert_eq!(m15_content::capture_relic(18, &[RelicId::ToughBandages]).unwrap_err(), run::RunError::Unsupported("M15 normal relic stock exhausted"));
}

#[test]
fn m15_capture_sampling_is_distinct_scoped_and_preflights_every_requested_family() {
    assert_eq!(run::RunRngScope::parse("run.m15.card.offer"), Ok(run::RunRngScope::M15CardOffer));
    assert_eq!(run::RunRngScope::parse("run.m10.card.offer"), Ok(run::RunRngScope::M10CardOffer));

    let mut rng = run::RunRng::new([71; 32], Partition::Test, 9);
    let before = rng.clone();
    let row = m15_content::sample_capture_row(&mut rng, &[4, 21, 2], &[CardRarity::Common, CardRarity::Uncommon, CardRarity::Uncommon]).unwrap();
    assert_eq!(row.len(), 3);
    assert_ne!(row[1], row[2]);
    assert!(row.iter().all(|id| {
        Catalog::standard().card(*id).is_some_and(|card| card.color == CardColor::Green && !card.definition_only)
    }));
    assert_eq!(Catalog::standard().card(row[0]).unwrap().rarity, CardRarity::Common);
    assert_eq!(Catalog::standard().card(row[1]).unwrap().rarity, CardRarity::Uncommon);
    assert_eq!(Catalog::standard().card(row[2]).unwrap().rarity, CardRarity::Uncommon);
    assert_ne!(rng, before, "an accepted capture advances only its new M15-owned stream");

    let rare=m15_content::sample_capture_row(&mut rng,&[4,21],&[CardRarity::Rare;3]).unwrap();
    assert_eq!(rare.iter().copied().collect::<BTreeSet<_>>().len(),3);
    assert!(rare.iter().all(|id|Catalog::standard().card(*id).unwrap().rarity==CardRarity::Rare));
    let mut rejected = run::RunRng::new([71; 32], Partition::Test, 9);
    let before_rejected = rejected.clone();
    assert!(matches!(m15_content::sample_capture_row(&mut rejected, &[4, 21], &[CardRarity::Common,CardRarity::Status]), Err(run::RunError::Unsupported(_))));
    assert_eq!(rejected,before_rejected,"every family is checked before the first accepted row draw");
}
