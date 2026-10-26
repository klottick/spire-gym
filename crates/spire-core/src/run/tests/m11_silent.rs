use crate::{
    content::{CardId, Character, EncounterId, PotionId, RelicId},
    run::{AuthoredRunStart, CreationProfile, Run, RunConfig, RunError, RunRng},
    combat::rng::Partition,
};

fn rng(seed: u64) -> RunRng {
    let mut root = [0; 32];
    root[..8].copy_from_slice(&seed.to_le_bytes());
    RunRng::new(root, Partition::Test, 0)
}

fn silent_run(seed: u64) -> Run {
    Run::new(
        RunConfig { character: Character::Silent, ascension: 0 },
        CreationProfile::experienced(),
        rng(seed),
    )
    .unwrap()
}

fn authored_from(run: &Run) -> AuthoredRunStart {
    let decision = run.decision().unwrap();
    AuthoredRunStart {
        hp: decision.player.hp,
        max_hp: decision.player.max_hp,
        gold: decision.player.gold,
        deck: run.state().deck.clone(),
        relics: decision.player.relics,
        potions: decision.player.potions,
        relic_counters: run.state().relic_counters.clone(),
    }
}

#[test]
fn m11_silent_authored_factory_accepts_canonical_run_new_start() {
    let source = silent_run(7);
    let decision = source.decision().unwrap();
    assert_eq!(decision.player.character, Character::Silent);
    assert_eq!((decision.player.hp, decision.player.max_hp, decision.player.gold), (70, 70, 99));
    assert_eq!(source.state().deck.len(), 12);
    assert_eq!(
        source.state().deck.iter().map(|card| (card.id, card.upgrades)).collect::<Vec<_>>(),
        vec![
            (CardId::StrikeGreen, 0),
            (CardId::StrikeGreen, 0),
            (CardId::StrikeGreen, 0),
            (CardId::StrikeGreen, 0),
            (CardId::StrikeGreen, 0),
            (CardId::DefendGreen, 0),
            (CardId::DefendGreen, 0),
            (CardId::DefendGreen, 0),
            (CardId::DefendGreen, 0),
            (CardId::DefendGreen, 0),
            (CardId::Survivor, 0),
            (CardId::Neutralize, 0),
        ],
        "the literal canonical roster is five Strike+, five Defend+, Survivor, Neutralize"
    );
    assert_eq!(
        source.state().deck.iter().map(|card| card.instance).collect::<Vec<_>>(),
        (1..=12).collect::<Vec<_>>(),
        "the twelve starter instances are unique and retain their canonical order"
    );
    assert_eq!(source.state().deck.iter().filter(|card| card.id == CardId::StrikeGreen).count(), 5);
    assert_eq!(source.state().deck.iter().filter(|card| card.id == CardId::DefendGreen).count(), 5);
    assert_eq!(source.state().deck[10].id, CardId::Survivor);
    assert_eq!(source.state().deck[11].id, CardId::Neutralize);
    assert_eq!(decision.player.relics, vec![RelicId::RingOfTheSnake]);
    assert_eq!(decision.player.potions, vec![None, None, None]);
    assert!(source.state().selected_neow.is_none(), "reading the public Neow decision must not select an offer");
    assert_eq!(decision.player.keys, [false; 3]);

    let authored = Run::from_authored_combat(
        RunConfig { character: Character::Silent, ascension: 0 },
        authored_from(&source),
        rng(7),
    );
    assert!(authored.is_ok(), "canonical Silent A0 resources should pass the authored-combat bridge: {authored:?}");
    let mut authored = authored.unwrap();
    assert_eq!(authored.state().keys, [false; 3]);
    assert_eq!(authored.state().deck, source.state().deck);
    assert_eq!(authored.state().relics, source.state().relics);
    assert_eq!(authored.state().potions, source.state().potions);
    assert_eq!(authored.state().gold, 99);
    assert_eq!(authored.state().hp, 70);
    assert_eq!(authored.state().max_hp, 70);
    assert_eq!(authored.state().combat_progress.entered, 0);
    assert_eq!(authored.state().next_instance, source.state().next_instance);
    assert!(authored.enter_combat(EncounterId::JawWorm).is_ok());
    assert!(authored.state().relics.is_empty() && authored.state().potions.is_empty(),
        "Run inventory is detached while the existing Combat owner holds it");
    let active = authored.decision().unwrap();
    let combat = active.combat.as_ref().expect("actual owned combat entered");
    assert_eq!(combat.hand.len(), 7, "Ring of the Snake's existing hook follows the five-card base draw");
    assert_eq!(combat.inventory.as_ref().unwrap().relics, vec![RelicId::RingOfTheSnake]);
    assert_eq!(combat.inventory.as_ref().unwrap().potions, vec![None, None, None]);
    assert!(!active.candidates.is_empty());
}

#[test]
fn m11_silent_authored_factory_rejects_changed_initial_resources() {
    let source = silent_run(7);
    let canonical = authored_from(&source);
    let mut cases = Vec::new();

    let mut changed = canonical.clone();
    changed.hp -= 1;
    cases.push(("hp", changed));
    let mut changed = canonical.clone();
    changed.max_hp += 1;
    cases.push(("max hp", changed));
    let mut changed = canonical.clone();
    changed.gold += 1;
    cases.push(("gold", changed));
    let mut changed = canonical.clone();
    changed.deck[0].id = CardId::Strike;
    cases.push(("starter card identity", changed));
    let mut changed = canonical.clone();
    changed.deck[0].instance += 1;
    cases.push(("starter permanent instance", changed));
    let mut changed = canonical.clone();
    changed.deck[0].upgrades = 1;
    cases.push(("starter upgrade", changed));
    let mut changed = canonical.clone();
    changed.deck.swap(0, 1);
    cases.push(("starter ordering", changed));
    let mut changed = canonical.clone();
    changed.relics.clear();
    cases.push(("starter relic", changed));
    let mut changed = canonical.clone();
    changed.potions[0] = Some(PotionId::FirePotion);
    cases.push(("potion slot", changed));
    let mut changed = canonical;
    changed.relic_counters.insert(RelicId::RingOfTheSnake, 1);
    cases.push(("relic counter", changed));

    for (label, start) in cases {
        let result = Run::from_authored_combat(
            RunConfig { character: Character::Silent, ascension: 0 },
            start,
            rng(7),
        );
        assert!(result.is_err(), "modified canonical {label} was accepted");
    }
}

#[test]
fn m11_silent_authored_factory_keeps_ironclad_m3_acceptance() {
    let start = AuthoredRunStart {
        hp: 80,
        max_hp: 80,
        gold: 99,
        deck: vec![crate::run::CardInstance { instance: 1, id: CardId::Strike, upgrades: 0 }],
        relics: vec![],
        potions: vec![None; 3],
        relic_counters: Default::default(),
    };
    let accepted = Run::from_authored_combat(
        RunConfig { character: Character::Ironclad, ascension: 0 },
        start,
        RunRng::new([41; 32], Partition::Test, 0),
    );
    assert!(accepted.is_ok());
}

#[test]
fn m11_silent_authored_factory_keeps_non_silent_character_rejection() {
    let source = silent_run(11);
    let start = authored_from(&source);
    let result = Run::from_authored_combat(
        RunConfig { character: Character::Defect, ascension: 0 },
        start,
        rng(11),
    );
    assert_eq!(result, Err(RunError::Unsupported("authored M3 requires Ironclad A0")));
}
