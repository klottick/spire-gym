//! Authored M1 fixtures and caller helpers, not game rules or normal-run setup.
use spire_core::{
    combat::{
        Combat, CombatAction, CombatEntry, CombatPublic, Outcome, FullCombatSetup, FullEnemySetup, FullIntentId,
        FullPowerEntry, FullPowerId, EnemyHandle, PlayerState, PowerState, CardInstanceId,
        permanent_live_card,
        enemies::EnemyProfile,
        rng::{self, Domain, Partition, StreamId, StreamState},
    },
    content::{CardId, Character, EnemyId},
    run::RunError,
};

pub const ROSTER: [CardId; 4] = [CardId::Strike, CardId::Defend, CardId::Bash, CardId::Clothesline];

fn stream(seed: u64, domain: Domain, actor: Option<&str>) -> StreamState {
    let mut root = [0; 32];
    root[..8].copy_from_slice(&seed.to_le_bytes());
    let mut scope = vec!["m1-authored-combat".to_owned()];
    if let Some(actor) = actor { scope.push(actor.to_owned()); }
    rng::make_stream(&root, Partition::Test, "m1", &StreamId::new(domain, scope).unwrap()).unwrap()
}

pub fn fixture(enemy: EnemyId, seed: u64) -> CombatEntry {
    let (hp, intent, powers) = match enemy {
        EnemyId::JawWorm => (42, FullIntentId::Chomp, vec![]),
        EnemyId::LouseGreen => (15, FullIntentId::Bite, vec![FullPowerEntry {
            id: FullPowerId("curl_up"),
            state: PowerState { amount: 5, skip_first: false, monster_source_grace: false },
        }]),
        EnemyId::GremlinNob => (85, FullIntentId::Bellow, vec![]),
        _ => panic!("not an M1 fixture enemy"),
    };
    CombatEntry {
        setup: FullCombatSetup {
            player: PlayerState { hp: 80, max_hp: 80, block: 0, energy: 3,
                max_energy: 3, draw_per_turn: 4, powers: vec![] },
            player_powers: vec![], character: Character::Ironclad,
            card_rng: stream(seed, Domain::RunCombatCardRandom, None),
            shuffle_rng: stream(seed, Domain::RunCombatShuffle, None),
            enemies: vec![FullEnemySetup {
                instance_id: EnemyHandle::parse("m1-enemy").unwrap(), enemy_id: enemy,
                ascension: 0, profile: EnemyProfile::Standard, hp, max_hp: hp,
                block: 0, intent, powers,
                ai_rng: stream(seed, Domain::RunCombatEnemyAi, Some("m1-enemy")),
            }],
            enemy_damage_bases: if enemy == EnemyId::LouseGreen {
                [((0, FullIntentId::Bite), 6)].into_iter().collect()
            } else { Default::default() },
        },
        opening_draw: ROSTER.len(),
        cards: ROSTER.iter().enumerate().map(|(i, id)|
            permanent_live_card(*id, CardInstanceId(i as u64 + 1), 0)).collect(),
    }
}

/// Choose only from the current public menu. Policy lives in this caller.
pub fn card_action(combat: &Combat, id: CardId) -> Result<CombatAction, RunError> {
    let hand = combat.public_full()?.hand;
    combat.legal_actions_full()?.into_iter().find(|action| match action {
        CombatAction::PlayCard { instance, .. } => hand.iter().any(|c| c.id == id && c.instance.0 == *instance),
        _ => false,
    }).ok_or(RunError::InvalidAction)
}

#[derive(Debug)]
pub enum DemoError {
    Combat(RunError),
    BudgetExceeded { accepted: usize },
}

impl From<RunError> for DemoError {
    fn from(error: RunError) -> Self { Self::Combat(error) }
}

impl std::fmt::Display for DemoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Combat(error) => write!(f, "combat error: {error:?}"),
            Self::BudgetExceeded { accepted } => write!(f, "unfinished scenario: action budget exhausted after {accepted} accepted actions"),
        }
    }
}

/// A bounded external caller policy, never an automatic simulator choice.
/// The observer receives public observations only, not snapshots or RNG state.
pub fn run_fight(combat: &mut Combat, attack: bool, budget: usize,
    mut observe: impl FnMut(&CombatPublic, &[CombatAction], Option<&CombatAction>))
    -> Result<Outcome, DemoError> {
    for accepted in 0..=budget {
        let view = combat.public_full()?;
        let menu = combat.legal_actions_full()?;
        if let Some(outcome) = view.outcome {
            observe(&view, &menu, None);
            return Ok(outcome);
        }
        if accepted == budget { return Err(DemoError::BudgetExceeded { accepted }); }
        let mut action = CombatAction::EndTurn;
        if attack {
            for id in [CardId::Bash, CardId::Clothesline, CardId::Strike, CardId::Defend] {
                if let Some(candidate) = menu.iter().find(|candidate| match candidate {
                    CombatAction::PlayCard { instance, .. } => view.hand.iter().any(|card|
                        card.id == id && card.instance.0 == *instance),
                    _ => false,
                }) {
                    action = candidate.clone();
                    break;
                }
            }
        }
        observe(&view, &menu, Some(&action));
        combat.step_full(&action)?;
    }
    unreachable!("the inclusive budget loop returns before stepping past the limit")
}
