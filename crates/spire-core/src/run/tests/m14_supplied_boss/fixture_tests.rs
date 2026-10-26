//! Prepublication checks for the narrowly supplied full-boss fixture.
use super::*;
use crate::{
    content::{Character, EnemyId},
    run::CommittedEvent,
};
use std::collections::BTreeSet;

const EXPECTED_CASES: [(u64, frozen::Scenario); 6] = [
    (7, frozen::Scenario { act1_boss: EncounterId::Guardian, act2_boss: EncounterId::Champ, act3_boss: EncounterId::AwakenedOne }),
    (7, frozen::Scenario { act1_boss: EncounterId::Hexaghost, act2_boss: EncounterId::BronzeAutomaton, act3_boss: EncounterId::TimeEater }),
    (7, frozen::Scenario { act1_boss: EncounterId::SlimeBoss, act2_boss: EncounterId::Collector, act3_boss: EncounterId::DonuAndDeca }),
    (11, frozen::Scenario { act1_boss: EncounterId::Hexaghost, act2_boss: EncounterId::Collector, act3_boss: EncounterId::AwakenedOne }),
    (11, frozen::Scenario { act1_boss: EncounterId::SlimeBoss, act2_boss: EncounterId::Champ, act3_boss: EncounterId::TimeEater }),
    (11, frozen::Scenario { act1_boss: EncounterId::Guardian, act2_boss: EncounterId::BronzeAutomaton, act3_boss: EncounterId::DonuAndDeca }),
];

// Independent literal oracles from the approved supplied-start proposal
// (SHA-256 9e4f1790f3c33a8c92b874677e955ca694220c36b9ff8927020bf112121f3833).
const EXPECTED_DECK: [CardId; 14] = [
    CardId::NoxiousFumes, CardId::NoxiousFumes, CardId::Catalyst, CardId::Catalyst,
    CardId::Backflip, CardId::Backflip, CardId::Backflip,
    CardId::Deflect, CardId::Deflect, CardId::Deflect,
    CardId::DeadlyPoison, CardId::DeadlyPoison,
    CardId::CloakAndDagger, CardId::WellLaidPlans,
];

const EXPECTED_ACQUIRE_RELICS: [RelicId; 11] = [
    RelicId::Pear, RelicId::Mango, RelicId::Strawberry, RelicId::SlaversCollar,
    RelicId::SneckoSkull, RelicId::OddlySmoothStone, RelicId::BagOfPreparation,
    RelicId::Lantern, RelicId::Anchor, RelicId::BronzeScales, RelicId::RedMask,
];

fn expected_act3_history(target: EncounterId) -> Vec<&'static str> {
    let mut history = match target {
        EncounterId::AwakenedOne => vec!["TimeEater", "DonuAndDeca"],
        EncounterId::TimeEater => vec!["AwakenedOne", "DonuAndDeca"],
        EncounterId::DonuAndDeca => vec!["AwakenedOne", "TimeEater"],
        other => panic!("unexpected supplied Act3 boss {other:?}"),
    };
    history.push(match target {
        EncounterId::AwakenedOne => "AwakenedOne",
        EncounterId::TimeEater => "TimeEater",
        EncounterId::DonuAndDeca => "DonuAndDeca",
        _ => unreachable!(),
    });
    history
}

fn expected_relics() -> Vec<RelicId> {
    std::iter::once(RelicId::RingOfTheSnake)
        .chain(ACQUIRE_RELICS)
        .collect()
}

#[test]
fn fixture_cases_keep_exact_original_boss_histories_and_order() {
    assert_eq!(CASES, EXPECTED_CASES);
    assert_eq!(DECK, EXPECTED_DECK);
    assert_eq!(ACQUIRE_RELICS, EXPECTED_ACQUIRE_RELICS);
    assert_eq!(POTIONS, [Some(PotionId::DexterityPotion); 3]);

    for (seed, scenario) in CASES {
        let prepared = fixture::prepare(seed, scenario).unwrap();
        let history = &prepared.run.state.seen_bosses;
        let act1_act2 = frozen::m13_support::Scenario {
            act1_boss: scenario.act1_boss,
            act2_boss: scenario.act2_boss,
        };
        assert_eq!(history[0], frozen::m13_support::act1_seen_bosses(act1_act2));
        assert_eq!(history[1], frozen::m13_support::act2_seen_bosses(act1_act2));
        assert_eq!(history[2], expected_act3_history(scenario.act3_boss));
    }
}

#[test]
fn fixture_prepares_full_hp_combat_and_the_real_matching_entry_for_all_six_cases() {
    for (seed, scenario) in CASES {
        let prepared = fixture::prepare(seed, scenario).unwrap();
        let entry = prepared.entry.as_ref().expect("fixture records its actual boss Move entry");
        let expected_entry = entry.committed_events.iter().find(|event| {
            matches!(event, CommittedEvent::CombatEntered { encounter, .. } if *encounter == scenario.act3_boss)
        });
        assert!(expected_entry.is_some(), "seed {seed}, boss {:?}", scenario.act3_boss);

        let state = &prepared.run.state;
        assert_eq!((state.character, state.ascension, state.act, state.floor), (Character::Silent, 0, 3, 50));
        assert_eq!((state.hp, state.max_hp), (101, 101));
        assert_eq!(state.gold, 99);
        assert_eq!(state.keys, [false; 3]);
        assert!(state.act4_enabled);
        assert_eq!(state.combat_progress.entered, 1);
        assert_eq!(state.combat_progress.settled, 0);
        assert_eq!(state.active_combat.as_ref().unwrap().encounter, scenario.act3_boss);
        let view = state.combat.as_ref().unwrap().public_full().unwrap();
        assert!(!view.enemies.is_empty());
        assert!(view.enemies.iter().all(|enemy| enemy.hp > 0 && enemy.hp == enemy.max_hp));
        match scenario.act3_boss {
            EncounterId::AwakenedOne => {
                let boss = view.enemies.iter().find(|enemy| enemy.enemy_id == EnemyId::AwakenedOne).unwrap();
                assert_eq!(boss.hp, 300);
                assert_eq!(view.enemies.iter().filter(|enemy| enemy.enemy_id == EnemyId::Cultist).count(), 2);
            }
            EncounterId::TimeEater => {
                let boss = view.enemies.iter().find(|enemy| enemy.enemy_id == EnemyId::TimeEater).unwrap();
                assert_eq!(boss.hp, 456);
                assert_eq!(view.enemies.len(), 1);
            }
            EncounterId::DonuAndDeca => {
                for id in [EnemyId::Donu, EnemyId::Deca] {
                    let boss = view.enemies.iter().find(|enemy| enemy.enemy_id == id).unwrap();
                    assert_eq!(boss.hp, 250);
                }
                assert_eq!(view.enemies.len(), 2);
            }
            other => panic!("unexpected supplied Act3 boss {other:?}"),
        }
    }
}

#[test]
fn fixture_records_only_the_current_entry_without_prior_combat_or_neow_lament() {
    for (seed, scenario) in CASES {
        let prepared = fixture::prepare(seed, scenario).unwrap();
        let state = &prepared.run.state;
        let combat_entries = state.history.events.iter().filter(|event| {
            matches!(event, CommittedEvent::CombatEntered { encounter, .. } if *encounter == scenario.act3_boss)
        }).count();
        assert_eq!(combat_entries, 1, "seed {seed}");
        assert!(!state.history.events.iter().any(|event| matches!(event, CommittedEvent::CombatFinished(_))));
        assert!(state.last_combat.is_none());
        assert!(state.selected_neow.is_none());
        assert!(!prepared.run.state.combat.as_ref().unwrap().item_context().unwrap()
            .relics().contains(&RelicId::NeowsLament));
    }
}

#[test]
fn fixture_owns_the_exact_fourteen_upgraded_cards_with_unique_permanent_ids() {
    for (seed, scenario) in CASES {
        let prepared = fixture::prepare(seed, scenario).unwrap();
        let state = &prepared.run.state;

        assert_eq!(state.deck.len(), DECK.len(), "seed {seed}");
        let owned_cards = state.deck.iter().map(|card| card.id).collect::<Vec<_>>();
        assert_eq!(owned_cards.as_slice(), DECK.as_slice());
        assert!(state.deck.iter().all(|card| card.upgrades == 1));
        assert!(state.deck.iter().all(|card| crate::run::M14_SILENT_ACT3_CARDS.contains(&card.id)));

        let ids = state.deck.iter().map(|card| card.instance).collect::<BTreeSet<_>>();
        assert_eq!(ids.len(), DECK.len(), "permanent card identities are unique");
        assert!(ids.iter().all(|id| *id > 0));
        assert!(state.next_instance > *ids.iter().max().unwrap());
    }
}

#[test]
fn fixture_keeps_inventory_custody_and_uses_native_start_hooks() {
    let expected_relics = expected_relics();

    for (seed, scenario) in CASES {
        let prepared = fixture::prepare(seed, scenario).unwrap();
        let state = &prepared.run.state;
        let combat = state.combat.as_ref().unwrap();
        let items = combat.item_context().unwrap();
        let view = combat.public_full().unwrap();

        assert_eq!(items.relics(), expected_relics.as_slice(), "relic custody for seed {seed}");
        assert!(items.relics().iter().all(|id| *id == RelicId::RingOfTheSnake
            || crate::run::M14_SILENT_ACT3_RELICS.contains(id)));
        assert_eq!(items.potions(), POTIONS.as_slice());
        assert!(items.potions().iter().flatten().all(|id| crate::run::M14_SILENT_ACT3_POTIONS.contains(id)));
        assert_eq!(view.inventory.as_ref().unwrap().relics.as_slice(), expected_relics.as_slice());
        assert_eq!(view.inventory.as_ref().unwrap().potions.as_slice(), POTIONS.as_slice());

        assert_eq!((view.player.energy, view.player.max_energy), (5, 4));
        assert_eq!(view.hand.len(), 9, "Ring and Bag of Preparation extend the ordinary opening draw");
        assert_eq!(view.player.block, 10, "Anchor supplies native first-turn block");
        assert!(view.player.powers.iter().any(|power| power.id.0 == "thorns" && power.amount == 3));
        assert!(view.player.powers.iter().any(|power| power.id.0 == "dexterity" && power.amount == 1));
        // Red Mask custody is checked above. Its startup Weak can be absorbed
        // by native Artifact; the Donu/Deca owner test records that case.
    }
}
