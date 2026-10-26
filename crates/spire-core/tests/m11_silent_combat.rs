#[path = "../examples/m11_silent_combat.rs"]
#[allow(dead_code)]
mod m11_cli;

use spire_core::{
    combat::{Outcome, rng::Partition},
    content::{CardId, Character, EncounterId, RelicId},
    run::{
        AuthoredRunStart, CardInstance, CommittedEvent, CombatProgress, CreationProfile,
        GameAction, Run, RunConfig, RunRng, RunTerminal,
    },
};
use std::collections::BTreeMap;

fn rng(seed: u64) -> RunRng {
    let mut root = [0; 32];
    root[..8].copy_from_slice(&seed.to_le_bytes());
    RunRng::new(root, Partition::Test, 0)
}

fn start_from_public_decision(decision: &spire_core::run::PublicDecision) -> AuthoredRunStart {
    assert!(decision.player.relic_counters.is_empty());
    AuthoredRunStart {
        hp: decision.player.hp,
        max_hp: decision.player.max_hp,
        gold: decision.player.gold,
        deck: decision.objects.iter().map(|card| CardInstance {
            instance: card.instance,
            id: card.id,
            upgrades: card.upgrades,
        }).collect(),
        relics: decision.player.relics.clone(),
        potions: decision.player.potions.clone(),
        relic_counters: BTreeMap::new(),
    }
}

fn silent_seed(seed: u64) -> Run {
    Run::new(
        RunConfig { character: Character::Silent, ascension: 0 },
        CreationProfile::experienced(),
        rng(seed),
    ).unwrap()
}

#[test]
fn m11_silent_public_start_enters_and_settles_real_owned_combat() {
    for seed in [7, 11] {
        let normal = silent_seed(seed);
        let start = normal.decision().unwrap();
        assert_eq!((start.act, start.floor), (1, 0));
        assert_eq!((start.player.character, start.player.hp, start.player.max_hp, start.player.gold),
            (Character::Silent, 70, 70, 99));
        assert_eq!(start.objects.iter().map(|card| (card.instance, card.id, card.upgrades)).collect::<Vec<_>>(), vec![
            (1, CardId::StrikeGreen, 0), (2, CardId::StrikeGreen, 0),
            (3, CardId::StrikeGreen, 0), (4, CardId::StrikeGreen, 0),
            (5, CardId::StrikeGreen, 0), (6, CardId::DefendGreen, 0),
            (7, CardId::DefendGreen, 0), (8, CardId::DefendGreen, 0),
            (9, CardId::DefendGreen, 0), (10, CardId::DefendGreen, 0),
            (11, CardId::Survivor, 0), (12, CardId::Neutralize, 0),
        ]);
        assert_eq!(start.player.relics, vec![RelicId::RingOfTheSnake]);
        assert_eq!(start.player.potions, vec![None, None, None]);
        assert_eq!(start.player.keys, [false; 3]);
        assert_eq!(start.offers.len(), 4, "the public Neow choice is visible but is not selected");
        assert!(start.candidates.iter().all(|candidate| matches!(candidate.action, GameAction::Choose(_))));
        assert!(start.combat.is_none());

        let mut authored = Run::from_authored_combat(
            RunConfig { character: Character::Silent, ascension: 0 },
            start_from_public_decision(&start),
            rng(seed),
        ).unwrap();
        let entry = authored.enter_combat(EncounterId::JawWorm).unwrap();
        assert_eq!(entry.committed_events, vec![CommittedEvent::CombatEntered {
            visit: 0,
            encounter: EncounterId::JawWorm,
        }]);

        let active = authored.decision().unwrap();
        let combat = active.combat.as_ref().expect("the bridge entered native combat");
        assert_eq!(combat.hand.len(), 7, "five base draws plus Ring of the Snake's existing two-card hook");
        assert_eq!(combat.inventory.as_ref().unwrap().relics, vec![RelicId::RingOfTheSnake]);
        assert_eq!(active.player.keys, [false; 3]);
        assert_eq!(active.combat_progress, CombatProgress { entered: 1, settled: 0 });
        assert!(!active.candidates.is_empty());
        assert!(active.candidates.iter().all(|candidate| matches!(candidate.action, GameAction::Combat(_))));

        let mut accepted = 0usize;
        let record = loop {
            let decision = authored.decision().unwrap();
            if let Some(record) = decision.last_combat {
                break record;
            }
            assert!(accepted < 8192, "seed {seed} exceeded the explicit caller budget");
            let action = decision.candidates.first().expect("native public legal actions remain available").action.clone();
            assert!(matches!(action, GameAction::Combat(_)), "the caller uses only combat candidates");
            let transition = authored.step(&action).unwrap();
            accepted += 1;
            if transition.committed_events.iter().any(|event| matches!(event, CommittedEvent::CombatFinished(_))) {
                assert!(transition.committed_events.iter().any(|event| matches!(event,
                    CommittedEvent::CombatFinished(completed) if completed == &authored.decision().unwrap().last_combat.clone().unwrap())));
            }
        };
        assert!(accepted > 0);
        assert_eq!(record.encounter, EncounterId::JawWorm);
        assert!(matches!(record.outcome, Outcome::Victory | Outcome::Death));

        let settled = authored.decision().unwrap();
        assert_eq!(settled.combat_progress, CombatProgress { entered: 1, settled: 1 });
        assert_eq!(settled.last_combat, Some(record.clone()));
        assert!(settled.combat.is_none());
        assert_eq!(settled.objects, start.objects, "settlement preserves permanent instance identities");
        assert_eq!(settled.player.relics, vec![RelicId::RingOfTheSnake], "the transferred relic returns exactly once");
        assert_eq!(settled.player.potions, vec![None, None, None]);
        match record.outcome {
            Outcome::Victory => assert_eq!(settled.terminal, None),
            Outcome::Death => assert_eq!(settled.terminal, Some(RunTerminal::Death)),
        }

        if record.outcome == Outcome::Victory {
            let evolved_hp = settled.player.hp;
            authored.enter_combat(EncounterId::Cultist).unwrap();
            let next = authored.decision().unwrap();
            assert_eq!(next.combat_progress, CombatProgress { entered: 2, settled: 1 });
            assert_eq!(next.player.hp, evolved_hp, "entry does not restore canonical starter HP after settlement");
            assert_eq!(next.combat.as_ref().unwrap().hand.len(), 7);
        }

    }
}

#[test]
fn m11_example_cli_accepts_default_budget_and_u64_seeds() {
    let options = m11_cli::parse_args([
        "--seed".to_owned(),
        u64::MAX.to_string(),
        "--scenario".to_owned(),
        "all".to_owned(),
    ]).unwrap();
    assert_eq!(options.seed, u64::MAX);
    assert_eq!(options.budget, 8192);
    assert_eq!(options.scenario, "all");
    assert!(m11_cli::parse_args(["--seed".to_owned(), "7".to_owned(), "--budget".to_owned(), "0".to_owned()]).is_err());
    assert!(m11_cli::parse_args(["--seed".to_owned(), "7".to_owned(), "--scenario".to_owned(), "invalid".to_owned()]).is_err());
}
