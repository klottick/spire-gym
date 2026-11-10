use crate::{combat::rng::Partition, content::Character, run::*};

fn fresh_run() -> Run {
    Run::new(
        RunConfig { character: Character::Ironclad, ascension: 0 },
        CreationProfile::experienced(),
        RunRng::new([73; 32], Partition::Test, 0),
    ).unwrap()
}

#[test]
fn itemise_rewards_covers_standard_reward_law() {
    let run = fresh_run();
    let before = run.reward_snapshot();
    let events = [
        CommittedEvent::FloorEntered { act: 1, floor: 1 },
        CommittedEvent::FloorEntered { act: 1, floor: 1 },
        CommittedEvent::FloorEntered { act: 1, floor: 2 },
        CommittedEvent::FloorEntered { act: 4, floor: 50 },
        CommittedEvent::KeyAcquired(Key::Ruby),
        CommittedEvent::KeyAcquired(Key::Ruby),
        CommittedEvent::KeyAcquired(Key::Emerald),
        CommittedEvent::KeyAcquired(Key::Sapphire),
        CommittedEvent::Ended(RunTerminal::HeartVictory),
        CommittedEvent::Ended(RunTerminal::Death),
    ];
    let items = itemise_rewards(&before, &events).unwrap();
    assert_eq!(items, vec![
        RewardItem { kind: RewardItemKind::NewFloor { act: 1, floor: 1 }, amount: 1 },
        RewardItem { kind: RewardItemKind::NewFloor { act: 1, floor: 2 }, amount: 1 },
        RewardItem { kind: RewardItemKind::NewAct4Floor { floor: 50 }, amount: 10 },
        RewardItem { kind: RewardItemKind::FirstKey(Key::Ruby), amount: 5 },
        RewardItem { kind: RewardItemKind::FirstKey(Key::Emerald), amount: 5 },
        RewardItem { kind: RewardItemKind::FirstKey(Key::Sapphire), amount: 5 },
        RewardItem { kind: RewardItemKind::HeartVictory, amount: 100 },
    ]);
    assert_eq!(items.iter().map(|item| item.amount).sum::<i32>(), 127);

    for terminal in [RunTerminal::Death, RunTerminal::EmptyDeck] {
        let run = fresh_run();
        let snapshot = run.reward_snapshot();
        let terminal_event = CommittedEvent::Ended(terminal);
        assert_eq!(itemise_rewards(&snapshot, &[terminal_event.clone()]).unwrap(), vec![RewardItem {
            kind: if terminal == RunTerminal::Death { RewardItemKind::Death } else { RewardItemKind::EmptyDeck },
            amount: -1,
        }]);
        assert!(itemise_rewards(&snapshot, &[terminal_event.clone(), terminal_event]).unwrap().len() == 1);
    }

    let run = fresh_run();
    let snapshot = run.reward_snapshot();
    assert!(itemise_rewards(&snapshot, &[CommittedEvent::Ended(RunTerminal::Act3NoKeys)]).unwrap().is_empty());
    let non_reward = CommittedEvent::ActAdvanced { from: 1, to: 2 };
    assert!(itemise_rewards(&snapshot, &[non_reward]).unwrap().is_empty());
}
