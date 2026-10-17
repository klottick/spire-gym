use super::super::*;
use super::super::rng::{make_stream, Domain, Partition, StreamId, RNG_CONTRACT_ID};

pub(super) fn base_scenario() -> Scenario {
    let stream = |domain, scope: Vec<&str>| {
        make_stream(&[0; 32], Partition::Test, "combat-behavior-plan",
            &StreamId::new(domain, scope.into_iter().map(str::to_owned).collect()).unwrap()).unwrap()
    };
    Scenario {
        encounter_id: EnemyKind::Cultist,
        player: PlayerState { hp: 40, max_hp: 50, block: 0, energy: 3,
            max_energy: 3, draw_per_turn: 5, powers: vec![] },
        enemies: vec![EnemySetup { instance_id: EnemyHandle::parse("enemy-1").unwrap(),
            enemy_id: EnemyKind::Cultist, hp: 48, max_hp: 48, block: 0, powers: vec![],
            intent_id: MoveId::Incantation, move_history: vec![MoveId::Incantation] }],
        piles: Piles { hand: [("1", CardKind::Strike), ("2", CardKind::Defend), ("3", CardKind::Bash)]
            .into_iter().map(|(id, card_id)| CardInstance {
                instance_id: CardHandle::parse(id).unwrap(), card_id, upgraded: false }).collect(),
            draw: vec![], discard: vec![], exhaust: vec![] },
        rng: CombatRng { contract_id: RNG_CONTRACT_ID.into(),
            shuffle: stream(Domain::CombatShuffle, vec!["combat-plan-1"]),
            enemy_ai: stream(Domain::CombatEnemyAi, vec!["combat-plan-1", "enemy-scope-1"]),
            enemy_instance_id: EnemyHandle::parse("enemy-1").unwrap() },
    }
}
