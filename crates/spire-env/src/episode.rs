//! Logical identity and staged sequence updates, independent of slot placement.
use std::sync::Arc;
use spire_core::combat::*;
use spire_core::combat::rng::{Domain, Partition, RNG_CONTRACT_ID, StreamId, make_stream};
use crate::{EnvError, IntegerSeed, ScenarioCatalog, ScenarioTemplate};
use crate::error::{reserved, setup};

/// Immutable supplied job, including its actual integer seed and derivation root.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EpisodeJob {
    template: Arc<ScenarioTemplate>,
    catalog_fingerprint: [u8;32],
    seed: IntegerSeed,
    root: [u8;32],
    partition: Partition,
    ordinal: u64,
}

impl EpisodeJob {
    pub fn template(&self) -> &ScenarioTemplate { &self.template }
    pub fn catalog_fingerprint(&self) -> [u8;32] { self.catalog_fingerprint }
    pub fn seed(&self) -> &IntegerSeed { &self.seed }
    pub fn root(&self) -> [u8;32] { self.root }
    pub fn partition(&self) -> Partition { self.partition }
    pub fn ordinal(&self) -> u64 { self.ordinal }
    pub fn episode_id(&self) -> String { format!("episode-{}", self.ordinal) }

    pub fn instantiate(&self) -> Result<Combat, EnvError> {
        Combat::new(&self.scenario()?).map_err(|e| setup(format!("supplied combat construction failed: {e:?}")))
    }

    pub(crate) fn scenario(&self) -> Result<Scenario, EnvError> {
        let t = &self.template;
        let enemy = EnemyHandle::parse("enemy-0").expect("fixed valid enemy handle");
        let first = match t.encounter { EnemyKind::Cultist => MoveId::Incantation, EnemyKind::JawWorm => MoveId::Chomp };
        let cards = |pile: &[crate::CardSpec], offset: usize| -> Result<Vec<CardInstance>, EnvError> {
            let mut result = reserved(pile.len())?;
            for (i, card) in pile.iter().enumerate() {
                let handle = offset.checked_add(i).ok_or_else(|| setup("card handle overflow"))?;
                result.push(CardInstance {
                    instance_id: CardHandle::parse(&handle.to_string()).expect("canonical decimal handle"),
                    card_id: card.kind, upgraded: card.upgraded,
                });
            }
            Ok(result)
        };
        let stream = |domain, scope| {
            let id = StreamId::new(domain, scope).map_err(|e| setup(format!("stream identity: {e:?}")))?;
            make_stream(&self.root, self.partition, &self.episode_id(), &id)
                .map_err(|e| setup(format!("stream derivation: {e:?}")))
        };
        Ok(Scenario {
            encounter_id: t.encounter,
            player: PlayerState { hp:t.player_hp, max_hp:t.player_max_hp, block:0,
                energy:t.energy_per_turn, max_energy:t.energy_per_turn, draw_per_turn:t.draw_per_turn, powers:vec![] },
            enemies:vec![EnemySetup { instance_id:enemy.clone(), enemy_id:t.encounter,
                hp:t.enemy_hp, max_hp:t.enemy_hp, block:0, powers:vec![], intent_id:first, move_history:vec![first] }],
            piles:Piles { hand:cards(&t.hand,0)?, draw:cards(&t.draw,t.hand.len())?, discard:vec![], exhaust:vec![] },
            rng:CombatRng { contract_id:RNG_CONTRACT_ID.into(),
                shuffle:stream(Domain::CombatShuffle,vec![t.id.clone()])?,
                enemy_ai:stream(Domain::CombatEnemyAi,vec![t.id.clone(),"enemy-0".into()])?,
                enemy_instance_id:enemy },
        })
    }
}

// These internal staged interfaces are consumed by the pool task, not exposed as
// a public alternate reset/RNG backend. Remove the temporary dead-code allowance
// when the pool becomes their production consumer.
#[allow(dead_code)]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct EpisodeSequence {
    state: Option<SequenceState>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SequenceState {
    seed: IntegerSeed,
    root: [u8;32],
    // None is exhausted, distinct from the valid last ordinal u64::MAX.
    next_ordinal: Option<u64>,
}

#[allow(dead_code)]
#[derive(Debug)]
pub(crate) struct PreparedEpisode {
    pub job: EpisodeJob,
    pub sequence: EpisodeSequence,
    pub effective_seed: Option<IntegerSeed>,
}

#[allow(dead_code)]
impl EpisodeSequence {
    pub(crate) fn next_ordinal(&self) -> Option<u64> {
        self.state.as_ref().and_then(|s| s.next_ordinal)
    }

    /// Pure staging: caller installs `sequence` only after all reset rows succeed.
    pub(crate) fn prepare_reset(&self, seed: Option<&IntegerSeed>, scenario_id: Option<&str>,
        catalog: &ScenarioCatalog, partition: Partition) -> Result<PreparedEpisode, EnvError> {
        self.prepare_with_entropy(seed, scenario_id, catalog, partition, &mut |bytes| {
            getrandom::fill(bytes).map_err(|e| setup(format!("initial seed entropy failed: {e}")))
        })
    }

    // Only the OS-backed entry point above is available to production callers.
    fn prepare_with_entropy(&self, seed: Option<&IntegerSeed>, scenario_id: Option<&str>,
        catalog: &ScenarioCatalog, partition: Partition,
        entropy: &mut impl FnMut(&mut [u8;32]) -> Result<(), EnvError>) -> Result<PreparedEpisode, EnvError> {
        let ordinal = if seed.is_some() || self.state.is_none() { 0 } else {
            self.state.as_ref().and_then(|s| s.next_ordinal).ok_or_else(|| setup("episode ordinal exhausted"))?
        };
        // Reject overrides/counters before consulting entropy or changing ownership.
        let template = catalog.select(ordinal, scenario_id)?;
        let effective_seed = match (seed, &self.state) {
            (Some(seed), _) => Some(seed.clone()),
            (None, None) => {
                let mut bytes = [0;32];
                entropy(&mut bytes)?;
                Some(IntegerSeed::from_be_bytes(&bytes)?)
            }
            (None, Some(_)) => None,
        };
        let (seed, root) = match &effective_seed {
            Some(seed) => (seed.clone(), seed.root()),
            None => {
                let state = self.state.as_ref().expect("initialized continuation");
                (state.seed.clone(), state.root)
            }
        };
        Ok(PreparedEpisode {
            job:EpisodeJob { template, catalog_fingerprint:catalog.fingerprint(), seed:seed.clone(), root, partition, ordinal },
            sequence:Self { state:Some(SequenceState { seed, root, next_ordinal:ordinal.checked_add(1) }) },
            effective_seed,
        })
    }

    #[cfg(test)]
    pub(crate) fn prepare_reset_with_entropy(&self, seed: Option<&IntegerSeed>, scenario_id: Option<&str>,
        catalog: &ScenarioCatalog, partition: Partition,
        entropy: &mut impl FnMut(&mut [u8;32]) -> Result<(), EnvError>) -> Result<PreparedEpisode, EnvError> {
        self.prepare_with_entropy(seed, scenario_id, catalog, partition, entropy)
    }
}

#[cfg(test)]
pub(crate) fn exhaustion_test_sequence(seed: &IntegerSeed) -> EpisodeSequence {
    EpisodeSequence { state:Some(SequenceState { seed:seed.clone(), root:seed.root(), next_ordinal:Some(u64::MAX) }) }
}
