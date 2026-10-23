use spire_core::combat::CONTENT_ID;
use spire_core::content::{
    CardColor, CardId, CardRarity, CardType, Catalog, Character, ContentProducer,
    RegistryError,
};

fn multiplicity(catalog: &Catalog, character: Character, id: CardId) -> usize {
    catalog.starter_deck(character).unwrap().iter().filter(|candidate| **candidate == id).count()
}

#[test]
fn exact_standard_base_starts_resolve() {
    let catalog = Catalog::standard();
    let starts = [
        (Character::Ironclad, CardId::Strike, CardId::Defend, 5, 4, CardId::Bash, 10),
        (Character::Silent, CardId::StrikeGreen, CardId::DefendGreen, 5, 5, CardId::Survivor, 12),
        (Character::Defect, CardId::StrikeBlue, CardId::DefendBlue, 4, 4, CardId::Zap, 10),
        (Character::Watcher, CardId::StrikePurple, CardId::DefendPurple, 4, 4, CardId::Eruption, 10),
    ];
    for (character, strike, defend, strikes, defends, special, total) in starts {
        let deck = catalog.starter_deck(character).expect("standard start is registered");
        assert_eq!(deck.len(), total, "{} start size", character.as_str());
        assert_eq!(multiplicity(catalog, character, strike), strikes);
        assert_eq!(multiplicity(catalog, character, defend), defends);
        assert_eq!(multiplicity(catalog, character, special), 1);
        if character == Character::Silent {
            assert_eq!(multiplicity(catalog, character, CardId::Neutralize), 1);
        } else if character == Character::Defect {
            assert_eq!(multiplicity(catalog, character, CardId::Dualcast), 1);
        } else if character == Character::Watcher {
            assert_eq!(multiplicity(catalog, character, CardId::Vigilance), 1);
        }
        assert!(deck.iter().all(|id| catalog.card(*id).is_some()));
        assert!(catalog.starter_relic(character).and_then(|id| catalog.relic(id)).is_some());
    }
    assert!(catalog.card(CardId::AscendersBane).is_some());
    assert!(catalog.card(CardId::IronWave).is_some());
    assert_eq!(catalog.card(CardId::AscendersBane).unwrap().rarity, CardRarity::Curse);
    assert!(catalog.card(CardId::AscendersBane).unwrap().definition_only);
}

#[test]
fn named_producers_keep_basic_and_off_color_rules_distinct() {
    let catalog = Catalog::standard();
    assert_eq!(catalog.eligibility(CardId::Anger, ContentProducer::OrdinaryReward, Character::Ironclad), Ok(true));
    assert_eq!(catalog.eligibility(CardId::Anger, ContentProducer::OrdinaryReward, Character::Silent), Ok(false));
    assert_eq!(catalog.eligibility(CardId::Anger, ContentProducer::ShardOrdinaryReward, Character::Silent), Ok(true));
    assert_eq!(catalog.eligibility(CardId::Strike, ContentProducer::ShardOrdinaryReward, Character::Silent), Ok(false));
    assert_eq!(catalog.eligibility(CardId::AscendersBane, ContentProducer::ShardOrdinaryReward, Character::Silent), Ok(false));
    assert_eq!(catalog.eligibility(CardId::Wound, ContentProducer::ShardOrdinaryReward, Character::Silent), Ok(false));
    assert_eq!(catalog.eligibility(CardId::Miracle, ContentProducer::ShardOrdinaryReward, Character::Silent), Ok(false));
    assert_eq!(catalog.eligibility(CardId::Strike, ContentProducer::NoteForYourself, Character::Silent), Ok(true));
    assert_eq!(catalog.eligibility(CardId::Zap, ContentProducer::NoteForYourself, Character::Ironclad), Ok(true));
    assert_eq!(catalog.eligibility(CardId::Anger, ContentProducer::NoteForYourself, Character::Silent), Ok(true));
    assert_eq!(catalog.eligibility(CardId::Madness, ContentProducer::NoteForYourself, Character::Ironclad), Ok(true));
    assert_eq!(catalog.eligibility(CardId::Regret, ContentProducer::NoteForYourself, Character::Silent), Ok(true));
    assert_eq!(catalog.eligibility(CardId::Miracle, ContentProducer::NoteForYourself, Character::Ironclad), Ok(false));
    assert_eq!(catalog.eligibility(CardId::Wound, ContentProducer::NoteForYourself, Character::Ironclad), Ok(false));
    assert_eq!(catalog.eligibility(CardId::AscendersBane, ContentProducer::NoteForYourself, Character::Ironclad), Ok(false));
    assert_eq!(catalog.eligibility(CardId::Strike, ContentProducer::PureWater, Character::Watcher), Ok(false));
    assert_eq!(catalog.eligibility(CardId::Miracle, ContentProducer::PureWater, Character::Watcher), Ok(true));
    assert_eq!(catalog.eligibility(CardId::Anger, ContentProducer::Unregistered("future"), Character::Silent), Err(RegistryError::UnknownProducer));
    assert_eq!(catalog.card(CardId::Anger).unwrap().color, CardColor::Red);
    assert_eq!(catalog.card(CardId::Anger).unwrap().rarity, CardRarity::Common);
    assert_eq!(catalog.card(CardId::Anger).unwrap().card_type, CardType::Attack);
    assert_eq!(catalog.eligibility(CardId::Unknown("unlisted"), ContentProducer::NoteForYourself, Character::Ironclad), Err(RegistryError::UnknownDefinition));
}

#[test]
fn unpopulated_selectable_pools_fail_closed() {
    let catalog = Catalog::standard();
    assert_eq!(catalog.selectable_pool(ContentProducer::OrdinaryReward, Character::Ironclad), Err(RegistryError::IncompletePool));
    assert!(catalog.card(CardId::Unknown("missing")).is_none());
}

#[test]
fn legacy_ids_unchanged() {
    assert_eq!(CONTENT_ID, "spire-lab-combat-reference-v2");
    assert_eq!(CardId::Strike.as_str(), "strike");
    assert_eq!(CardId::Defend.as_str(), "defend");
    assert_eq!(CardId::Bash.as_str(), "bash");
}
