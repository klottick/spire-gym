use std::{
    cmp::Reverse,
    collections::{BTreeMap, BTreeSet},
};

use spire_core::run::map::PublicMap;
use spire_core::run::{GameAction, MapNodeKind, NodeId, PublicDecision};

type RouteState = (NodeId, bool, bool, bool);
type Memo = BTreeMap<RouteState, Option<RouteRank>>;

#[derive(Clone, Copy)]
struct RouteRank {
    // Elites reached after the existing quota was already satisfied.
    extra_elites: u32,
    // Active only for M14's low-health, unmet-quota preference.
    rest_before_first_elite: bool,
    resource_score: i64,
}

#[derive(Clone, Copy)]
struct RouteProfile {
    m14_rank: bool,
    prefer_rest_before_first_elite: bool,
}

impl RouteProfile {
    const M13: Self = Self {
        m14_rank: false,
        prefer_rest_before_first_elite: false,
    };

    fn m14(decision: &PublicDecision, elite_reached: bool) -> Self {
        let at_most_half_health =
            u64::from(decision.player.hp) * 2 <= u64::from(decision.player.max_hp);
        Self {
            m14_rank: true,
            prefer_rest_before_first_elite: !elite_reached && at_most_half_health,
        }
    }

    fn key(self, rank: RouteRank) -> (Reverse<u32>, bool, i64) {
        if self.m14_rank {
            // M14 prioritizes fewer extra elites, then the visible recovery
            // condition, before reusing the established resource score.
            (Reverse(rank.extra_elites), rank.rest_before_first_elite, rank.resource_score)
        } else {
            (Reverse(0), false, rank.resource_score)
        }
    }
}

#[derive(Clone, Copy)]
struct RouteWeights {
    missing_hp: u32,
    max_hp: u32,
    score_scale: i64,
    potion_stock: i64,
    gold: u32,
}

pub(crate) fn action(
    decision: &PublicDecision,
    map: &PublicMap,
    elite_reached: bool,
    ordinary_reached: bool,
) -> Result<GameAction, String> {
    let weights = RouteWeights::from_decision(decision);
    let mut memo = Memo::new();
    let mut visiting = BTreeSet::new();

    decision
        .candidates
        .iter()
        .filter_map(|candidate| {
            let GameAction::Move(id) = &candidate.action else {
                return None;
            };
            path_rank(
                map,
                *id,
                elite_reached,
                ordinary_reached,
                false,
                RouteProfile::M13,
                weights,
                &mut memo,
                &mut visiting,
            )
            .map(|rank| (rank.resource_score, *id, candidate.action.clone()))
        })
        .max_by_key(|(score, id, _)| (*score, Reverse(*id)))
        .map(|(_, _, action)| action)
        .ok_or_else(|| "map has no offered route to a boss".into())
}

pub(crate) fn action_m14(
    decision: &PublicDecision,
    map: &PublicMap,
    elite_reached: bool,
    ordinary_reached: bool,
) -> Result<GameAction, String> {
    let weights = RouteWeights::from_decision(decision);
    let profile = RouteProfile::m14(decision, elite_reached);
    let mut memo = Memo::new();
    let mut visiting = BTreeSet::new();

    decision
        .candidates
        .iter()
        .filter_map(|candidate| {
            let GameAction::Move(id) = &candidate.action else {
                return None;
            };
            path_rank(
                map,
                *id,
                elite_reached,
                ordinary_reached,
                false,
                profile,
                weights,
                &mut memo,
                &mut visiting,
            )
            .map(|rank| (rank, *id, candidate.action.clone()))
        })
        .max_by_key(|(rank, id, _)| {
            let (extra_elites, rest_before_first_elite, resource_score) =
                profile.key(*rank);
            (extra_elites, rest_before_first_elite, resource_score, Reverse(*id))
        })
        .map(|(_, _, action)| action)
        .ok_or_else(|| "map has no offered route to a boss".into())
}

fn path_rank(
    map: &PublicMap,
    id: NodeId,
    elite_reached: bool,
    ordinary_reached: bool,
    rest_before_first_elite: bool,
    profile: RouteProfile,
    weights: RouteWeights,
    memo: &mut Memo,
    visiting: &mut BTreeSet<RouteState>,
) -> Option<RouteRank> {
    let key = (id, elite_reached, ordinary_reached, rest_before_first_elite);
    if let Some(score) = memo.get(&key) {
        return *score;
    }
    if !visiting.insert(key) {
        return None;
    }

    let score = map.nodes.iter().find(|node| node.id == id).and_then(|node| {
        let next_elite_reached = elite_reached || node.kind == MapNodeKind::Elite;
        let next_ordinary_reached = ordinary_reached || node.kind == MapNodeKind::Monster;
        if node.kind == MapNodeKind::Boss {
            return (next_elite_reached && next_ordinary_reached).then_some(RouteRank {
                extra_elites: 0,
                rest_before_first_elite,
                resource_score: 0,
            });
        }

        let next_rest_before_first_elite = rest_before_first_elite
            || (profile.prefer_rest_before_first_elite
                && !elite_reached
                && node.kind == MapNodeKind::Rest);
        let best_continuation = node
            .edges
            .iter()
            .filter_map(|next| {
                path_rank(
                    map,
                    *next,
                    next_elite_reached,
                    next_ordinary_reached,
                    next_rest_before_first_elite,
                    profile,
                    weights,
                    memo,
                    visiting,
                )
            })
            .max_by_key(|rank| profile.key(*rank))?;
        Some(RouteRank {
            extra_elites: best_continuation.extra_elites
                + u32::from(node.kind == MapNodeKind::Elite && elite_reached),
            rest_before_first_elite: best_continuation.rest_before_first_elite,
            resource_score: weights.node_score(node.kind, elite_reached)
                + best_continuation.resource_score,
        })
    });

    visiting.remove(&key);
    memo.insert(key, score);
    score
}

impl RouteWeights {
    fn from_decision(decision: &PublicDecision) -> Self {
        let max_hp = decision.player.max_hp;
        Self {
            missing_hp: max_hp.saturating_sub(decision.player.hp),
            max_hp,
            // Four times max HP keeps HP fractions and the 3/2 risk multiplier exact.
            score_scale: i64::from(max_hp.max(1)) * 4,
            potion_stock: decision
                .player
                .potions
                .iter()
                .filter(|potion| potion.is_some())
                .count()
                .min(3) as i64,
            gold: decision.player.gold,
        }
    }

    fn node_score(self, kind: MapNodeKind, elite_reached: bool) -> i64 {
        let risk = (12 * self.score_scale + self.ratio_score(24)
            - 3 * self.potion_stock * self.score_scale)
        .max(3 * self.score_scale);
        match kind {
            MapNodeKind::Monster => -if elite_reached { risk } else { risk * 3 / 2 },
            MapNodeKind::Elite if elite_reached => -120 * self.score_scale,
            MapNodeKind::Elite => -(25 * self.score_scale + self.ratio_score(40)
                - 5 * self.potion_stock * self.score_scale),
            MapNodeKind::Rest if elite_reached => 10 * self.score_scale + self.ratio_score(40),
            MapNodeKind::Rest => 12 * self.score_scale + self.ratio_score(50),
            MapNodeKind::Treasure => 14 * self.score_scale,
            MapNodeKind::Shop if self.gold >= 150 => 16 * self.score_scale,
            MapNodeKind::Shop if self.gold >= 50 => 8 * self.score_scale,
            MapNodeKind::Shop => 0,
            MapNodeKind::Question | MapNodeKind::Event => 4 * self.score_scale,
            MapNodeKind::Boss => 0,
        }
    }

    fn ratio_score(self, coefficient: i64) -> i64 {
        if self.max_hp == 0 {
            return 0;
        }
        coefficient * i64::from(self.missing_hp) * 4
    }
}
