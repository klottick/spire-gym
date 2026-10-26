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

pub(super) fn action(
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

pub(super) fn action_m14(
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

#[cfg(test)]
mod tests {
    use super::action;
    use spire_core::{
        content::Character,
        run::map::{PublicMap, PublicMapNode},
        run::{
            Candidate, ChoiceId, CombatProgress, GameAction, MapNodeKind, NodeId, PublicDecision,
            PublicPlayer, RoomScenario, RoomView, RoomViewKind,
        },
    };

    fn node(id: u64, kind: MapNodeKind, edges: &[u64]) -> PublicMapNode {
        PublicMapNode {
            id: NodeId(id),
            floor: id as u32,
            x: id as u8,
            y: id as u8,
            kind,
            resolved_room: None,
            edges: edges.iter().copied().map(NodeId).collect(),
            visited: false,
            revealed: true,
            burning: None,
        }
    }

    fn map(candidate_ids: &[u64], nodes: Vec<PublicMapNode>) -> PublicMap {
        let mut all_nodes = vec![node(0, MapNodeKind::Rest, candidate_ids)];
        all_nodes.extend(nodes);
        PublicMap {
            boss: None,
            act: 2,
            nodes: all_nodes,
            current: Some(NodeId(0)),
            winged_moves: 0,
        }
    }

    fn decision(candidate_ids: &[u64], hp: u32, max_hp: u32, gold: u32) -> PublicDecision {
        PublicDecision {
            content_profile: Some(RoomScenario::M13SilentAct2),
            room: Some(RoomView { kind: RoomViewKind::Map, choices: Vec::new() }),
            reward_total: 0,
            act: 2,
            floor: 1,
            player: PublicPlayer {
                relic_counters: Vec::new(),
                character: Character::Silent,
                hp,
                max_hp,
                gold,
                relics: Vec::new(),
                potions: vec![None, None, None],
                keys: [false; 3],
            },
            combat: None,
            combat_progress: CombatProgress::default(),
            last_combat: None,
            objects: Vec::new(),
            candidates: candidate_ids
                .iter()
                .copied()
                .enumerate()
                .map(|(index, id)| Candidate {
                    id: ChoiceId(index as u64 + 1),
                    action: GameAction::Move(NodeId(id)),
                    label: "Move",
                    neow_offer: None,
                    object: None,
                })
                .collect(),
            offers: Vec::new(),
            selection: None,
            terminal: None,
        }
    }

    fn m14_map(candidate_ids: &[u64], nodes: Vec<PublicMapNode>) -> PublicMap {
        let mut map = map(candidate_ids, nodes);
        map.act = 3;
        map
    }

    fn m14_decision(candidate_ids: &[u64], hp: u32, max_hp: u32, gold: u32) -> PublicDecision {
        let mut decision = decision(candidate_ids, hp, max_hp, gold);
        decision.content_profile = Some(RoomScenario::M14SilentAct3);
        decision.act = 3;
        decision
    }

    fn m14_action(decision: &PublicDecision, map: &PublicMap) -> Result<GameAction, String> {
        m14_action_with_progress(decision, map, false, false)
    }

    fn m14_action_with_progress(
        decision: &PublicDecision,
        map: &PublicMap,
        elite_reached: bool,
        ordinary_reached: bool,
    ) -> Result<GameAction, String> {
        let mut caller = super::super::Caller::new_m14(8192).unwrap();
        caller.act3_elite_reached = elite_reached;
        caller.act3_ordinary_reached = ordinary_reached;
        super::super::action_for(decision, Some(map), &caller)
    }

    #[test]
    fn m14_revision_routes_minimizes_elites_beyond_the_required_one() {
        let mut nodes = vec![
            node(10, MapNodeKind::Elite, &[11]),
            node(11, MapNodeKind::Monster, &[100]),
            node(20, MapNodeKind::Treasure, &[21]),
        ];
        for id in 21..=29 {
            nodes.push(node(id, MapNodeKind::Treasure, &[id + 1]));
        }
        nodes.push(node(30, MapNodeKind::Elite, &[31]));
        for id in 31..=39 {
            nodes.push(node(id, MapNodeKind::Treasure, &[id + 1]));
        }
        nodes.extend([
            node(40, MapNodeKind::Elite, &[41]),
            node(41, MapNodeKind::Monster, &[100]),
            node(100, MapNodeKind::Boss, &[]),
        ]);
        let map = m14_map(&[10, 20], nodes);
        let decision = m14_decision(&[10, 20], 70, 70, 0);

        assert_eq!(m14_action(&decision, &map), Ok(GameAction::Move(NodeId(10))));
    }

    #[test]
    fn m14_revision_routes_low_health_prefers_rest_before_the_required_elite() {
        let mut nodes = vec![
            node(10, MapNodeKind::Rest, &[11]),
            node(11, MapNodeKind::Elite, &[12]),
            node(12, MapNodeKind::Monster, &[100]),
            node(20, MapNodeKind::Elite, &[21]),
        ];
        for id in 21..=36 {
            nodes.push(node(id, MapNodeKind::Treasure, &[id + 1]));
        }
        nodes.extend([
            node(37, MapNodeKind::Rest, &[38]),
            node(38, MapNodeKind::Monster, &[100]),
            node(100, MapNodeKind::Boss, &[]),
        ]);
        let map = m14_map(&[10, 20], nodes);
        let decision = m14_decision(&[10, 20], 20, 70, 0);

        assert_eq!(m14_action(&decision, &map), Ok(GameAction::Move(NodeId(10))));

        let exact_half = m14_decision(&[10, 20], 35, 70, 0);
        assert_eq!(m14_action(&exact_half, &map), Ok(GameAction::Move(NodeId(10))));
    }

    #[test]
    fn m14_revision_routes_uses_existing_resource_score_after_route_constraints() {
        let map = m14_map(
            &[10, 20],
            vec![
                node(10, MapNodeKind::Question, &[11]),
                node(11, MapNodeKind::Elite, &[12]),
                node(12, MapNodeKind::Monster, &[100]),
                node(20, MapNodeKind::Shop, &[21]),
                node(21, MapNodeKind::Elite, &[22]),
                node(22, MapNodeKind::Monster, &[100]),
                node(100, MapNodeKind::Boss, &[]),
            ],
        );
        let decision = m14_decision(&[10, 20], 70, 70, 150);

        assert_eq!(m14_action(&decision, &map), Ok(GameAction::Move(NodeId(20))));
    }

    #[test]
    fn m14_revision_routes_breaks_equal_ranks_by_smallest_offered_node_id() {
        let map = m14_map(
            &[20, 5],
            vec![
                node(20, MapNodeKind::Monster, &[30]),
                node(5, MapNodeKind::Monster, &[30]),
                node(30, MapNodeKind::Elite, &[40]),
                node(40, MapNodeKind::Boss, &[]),
            ],
        );
        let decision = m14_decision(&[20, 5], 70, 70, 0);

        assert_eq!(m14_action(&decision, &map), Ok(GameAction::Move(NodeId(5))));
    }

    #[test]
    fn m14_revision_routes_keeps_both_unmet_quotas_hard() {
        let map = m14_map(
            &[10, 20],
            vec![
                node(10, MapNodeKind::Elite, &[11]),
                node(11, MapNodeKind::Treasure, &[100]),
                node(20, MapNodeKind::Monster, &[21]),
                node(21, MapNodeKind::Treasure, &[100]),
                node(100, MapNodeKind::Boss, &[]),
            ],
        );
        let decision = m14_decision(&[10, 20], 70, 70, 0);

        assert_eq!(
            m14_action(&decision, &map),
            Err("no public caller rule for floor 1 in Map".into()),
        );
    }

    #[test]
    fn m14_revision_routes_accepts_a_route_after_both_quotas_are_satisfied() {
        let map = m14_map(
            &[10],
            vec![node(10, MapNodeKind::Shop, &[100]), node(100, MapNodeKind::Boss, &[])],
        );
        let decision = m14_decision(&[10], 70, 70, 150);

        assert_eq!(
            m14_action_with_progress(&decision, &map, true, true),
            Ok(GameAction::Move(NodeId(10))),
        );
    }

    #[test]
    fn low_health_prefers_recovery_before_the_first_elite_on_equal_length_routes() {
        let map = map(
            &[10, 20],
            vec![
                node(10, MapNodeKind::Rest, &[11]),
                node(11, MapNodeKind::Elite, &[12]),
                node(12, MapNodeKind::Monster, &[13]),
                node(20, MapNodeKind::Elite, &[21]),
                node(21, MapNodeKind::Rest, &[22]),
                node(22, MapNodeKind::Monster, &[13]),
                node(13, MapNodeKind::Boss, &[]),
            ],
        );
        let decision = decision(&[10, 20], 10, 70, 0);

        assert_eq!(action(&decision, &map, false, false), Ok(GameAction::Move(NodeId(10))));
    }

    #[test]
    fn high_value_route_without_elite_cannot_beat_a_required_elite_route() {
        let map = map(
            &[1, 2],
            vec![
                node(1, MapNodeKind::Treasure, &[3]),
                node(3, MapNodeKind::Question, &[5]),
                node(2, MapNodeKind::Elite, &[4]),
                node(4, MapNodeKind::Monster, &[5]),
                node(5, MapNodeKind::Boss, &[]),
            ],
        );
        let decision = decision(&[1, 2], 70, 70, 200);

        assert_eq!(action(&decision, &map, false, false), Ok(GameAction::Move(NodeId(2))));
    }

    #[test]
    fn missing_required_elite_route_does_not_fall_back_to_a_boss_shortcut() {
        let map = map(
            &[1],
            vec![
                node(1, MapNodeKind::Shop, &[2]),
                node(2, MapNodeKind::Treasure, &[3]),
                node(3, MapNodeKind::Boss, &[]),
            ],
        );
        let decision = decision(&[1], 70, 70, 200);

        assert_eq!(action(&decision, &map, false, false), Err("map has no offered route to a boss".into()));
    }

    #[test]
    fn prior_elite_satisfies_the_requirement_and_extra_elites_are_avoided() {
        let map = map(
            &[10, 20],
            vec![
                node(10, MapNodeKind::Monster, &[30]),
                node(20, MapNodeKind::Elite, &[21]),
                node(21, MapNodeKind::Monster, &[30]),
                node(30, MapNodeKind::Boss, &[]),
            ],
        );
        let decision = decision(&[10, 20], 70, 70, 0);

        assert_eq!(action(&decision, &map, true, false), Ok(GameAction::Move(NodeId(10))));
    }

    #[test]
    fn richer_gold_prefers_a_shop_over_a_question_with_equal_combat_routes() {
        let map = map(
            &[10, 20],
            vec![
                node(10, MapNodeKind::Shop, &[30]),
                node(20, MapNodeKind::Question, &[31]),
                node(30, MapNodeKind::Monster, &[40]),
                node(31, MapNodeKind::Monster, &[40]),
                node(40, MapNodeKind::Boss, &[]),
            ],
        );
        let decision = decision(&[10, 20], 70, 70, 150);

        assert_eq!(action(&decision, &map, true, true), Ok(GameAction::Move(NodeId(10))));
    }

    #[test]
    fn equal_scored_offered_routes_choose_the_smallest_public_node_id() {
        let map = map(
            &[20, 5],
            vec![
                node(20, MapNodeKind::Monster, &[30]),
                node(5, MapNodeKind::Monster, &[30]),
                node(30, MapNodeKind::Elite, &[40]),
                node(40, MapNodeKind::Boss, &[]),
            ],
        );
        let decision = decision(&[20, 5], 70, 70, 0);

        assert_eq!(action(&decision, &map, false, false), Ok(GameAction::Move(NodeId(5))));
    }

    #[test]
    fn current_health_changes_the_value_of_visible_recovery() {
        let map = map(&[10, 20], vec![
            node(10, MapNodeKind::Rest, &[11]),
            node(11, MapNodeKind::Question, &[30]),
            node(20, MapNodeKind::Treasure, &[21]),
            node(21, MapNodeKind::Question, &[30]),
            node(30, MapNodeKind::Boss, &[]),
        ]);
        let healthy = decision(&[10, 20], 70, 70, 0);
        let injured = decision(&[10, 20], 7, 70, 0);
        assert_eq!(action(&healthy, &map, true, true), Ok(GameAction::Move(NodeId(20))));
        assert_eq!(action(&injured, &map, true, true), Ok(GameAction::Move(NodeId(10))));
    }

    #[test]
    fn current_potion_stock_changes_the_visible_combat_risk_score() {
        let map = map(&[10, 20], vec![
            node(10, MapNodeKind::Monster, &[11]),
            node(11, MapNodeKind::Treasure, &[30]),
            node(20, MapNodeKind::Question, &[21]),
            node(21, MapNodeKind::Shop, &[30]),
            node(30, MapNodeKind::Boss, &[]),
        ]);
        let empty = decision(&[10, 20], 70, 70, 0);
        let mut stocked = empty.clone();
        stocked.player.potions = vec![Some(spire_core::content::PotionId::FirePotion); 3];
        assert_eq!(action(&empty, &map, true, true), Ok(GameAction::Move(NodeId(20))));
        assert_eq!(action(&stocked, &map, true, true), Ok(GameAction::Move(NodeId(10))));
    }

}
