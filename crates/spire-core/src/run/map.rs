//! Native representative act-map generation. This is deliberately not an
//! original-seed replica; see the Task 8 report for the qualified policy.
use super::{MapNode, MapNodeKind, Run, RunError, RunMap, RunRng, RunRngScope, RunState};

pub type ActMap = RunMap;

/// Inputs governing whether this generated map may include the Act 4 burning-Elite marker.
/// The flags are intentionally opaque outside this module's constructor contract.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MapEligibility { act4_enabled: bool, emerald_key_owned: bool }
impl MapEligibility {
    pub const fn new(act4_enabled: bool, emerald_key_owned: bool) -> Self { Self { act4_enabled, emerald_key_owned } }
    pub(crate) fn from_run(state: &RunState) -> Self {
        Self { act4_enabled: state.act4_enabled, emerald_key_owned: state.keys[2] }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublicMapNode {
    pub id: super::NodeId,
    pub floor: u32,
    pub x: u8,
    pub y: u8,
    pub kind: MapNodeKind,
    pub resolved_room: Option<MapNodeKind>,
    pub edges: Vec<super::NodeId>,
    pub visited: bool,
    pub revealed: bool,
    pub burning: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublicMap {
    pub act: u8,
    pub nodes: Vec<PublicMapNode>,
    pub current: Option<super::NodeId>,
    pub winged_moves: u8,
}

pub(crate) fn public_view(map: &RunMap) -> PublicMap {
    let nodes = map.nodes.iter().map(|node| {
        let resolved_room = if node.kind == MapNodeKind::Question && node.visited && node.revealed {
            node.hidden_kind
        } else { None };
        let burning = if node.kind == MapNodeKind::Elite { Some(node.burning) } else { None };
        PublicMapNode { id: node.id, floor: node.floor, x: node.x, y: node.y, kind: node.kind,
            resolved_room, edges: node.edges.clone(), visited: node.visited, revealed: node.revealed, burning }
    }).collect();
    PublicMap { act: map.act, nodes, current: map.current, winged_moves: map.winged_moves }
}

impl ActMap {
    /// Return the sanitized map facts intended for an ordinary external consumer.
    /// Hidden question outcomes, selected encounter identities and private histories
    /// are not included.
    pub fn public_view(&self) -> PublicMap {
        public_view(self)
    }
}

// `ActMap` is the public alias for this private-storage type. Keep its standard
// Debug path identical to the supported sanitized view rather than exposing the
// raw RunMap representation.
impl std::fmt::Debug for super::RunMap {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&public_view(self), formatter)
    }
}

// Run's derived raw-state Debug view would expose hidden map knowledge. Its
// human-readable form is therefore the same sanitized view used by consumers.
impl std::fmt::Debug for Run {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Run")
            .field("decision", &self.decision())
            .field("map", &self.public_map())
            .finish()
    }
}

const WIDTH: usize = 7;
const LAST_ROW: usize = 14;
const BOSS_ROW: usize = 15;

fn node_id(x: usize, y: usize) -> super::NodeId {
    super::NodeId((y * WIDTH + x) as u64)
}

fn round_half_up(n: usize, percent: usize) -> usize {
    (n * percent + 50) / 100
}

/// Generate a connected representative map with scoped topology and room draws.
/// Act 4's special routing is owned by Task 12; ordinary maps are generated for
/// Acts 1–3. Any rejected draft is private and consumes only its semantic scopes.
pub fn generate(act: u8, rng: &mut RunRng, eligibility: MapEligibility) -> Result<ActMap, RunError> {
    if !(1..=3).contains(&act) {
        return Err(RunError::Unsupported("ordinary map generation is defined for Acts 1-3; Act 4 routing belongs to Task 12"));
    }
    let mut draft = 0u64;
    loop {
        match generate_draft(act, draft, rng, eligibility)? {
            Some(map) => return Ok(map),
            None => draft = draft.checked_add(1).ok_or(RunError::Unsupported("map draft ordinal exhausted"))?,
        }
    }
}

fn generate_draft(act: u8, draft: u64, rng: &mut RunRng, eligibility: MapEligibility) -> Result<Option<ActMap>, RunError> {
    // Sample six distinct starts in canonical column order, then walk each route
    // with noncrossing adjacent transitions. Route ordering is stable.
    let mut starts: Vec<usize> = (0..WIDTH).collect();
    for i in 0..6 {
        let j = i + rng.bounded_scoped(RunRngScope::MapTopology, &[act as u64, draft, 0, i as u64], (WIDTH - i) as u64);
        starts.swap(i, j);
    }
    starts.truncate(6);
    starts.sort_unstable();
    let mut routes: Vec<Vec<usize>> = starts.iter().map(|&x| vec![x]).collect();
    for row in 0..LAST_ROW {
        for route in 0..routes.len() {
            let x = *routes[route].last().expect("route has a start");
            let mut choices: Vec<usize> = (x.saturating_sub(1)..=(x + 1).min(WIDTH - 1)).collect();
            choices.retain(|&next| {
                (0..route).all(|prior| {
                    let px = routes[prior][row];
                    let pn = routes[prior][row + 1];
                    (x as isize - px as isize) * (next as isize - pn as isize) >= 0
                })
            });
            if choices.is_empty() { return Ok(None); }
            let index = rng.bounded_scoped(RunRngScope::MapTopology,
                &[act as u64, draft, (row + 1) as u64, route as u64], choices.len() as u64);
            routes[route].push(choices[index]);
        }
    }

    let mut edges: Vec<(usize, usize)> = Vec::new();
    let mut active = vec![[false; WIDTH]; LAST_ROW + 1];
    for route in &routes {
        for row in 0..=LAST_ROW { active[row][route[row]] = true; }
        for row in 0..LAST_ROW {
            edges.push((node_id(route[row], row).0 as usize, node_id(route[row + 1], row + 1).0 as usize));
        }
    }
    edges.sort_unstable();
    edges.dedup();
    let row14: Vec<usize> = (0..WIDTH).filter(|&x| active[LAST_ROW][x]).collect();
    let boss = node_id(3, BOSS_ROW);
    for x in row14 { edges.push((node_id(x, LAST_ROW).0 as usize, boss.0 as usize)); }

    let mut parents = vec![Vec::<usize>::new(); (BOSS_ROW + 1) * WIDTH];
    let mut children = vec![Vec::<usize>::new(); (BOSS_ROW + 1) * WIDTH];
    for &(from, to) in &edges { parents[to].push(from); children[from].push(to); }
    if !(1..=LAST_ROW).any(|y| (0..WIDTH).any(|x| active[y][x] && parents[node_id(x,y).0 as usize].len() > 1))
        || !(0..LAST_ROW).any(|y| (0..WIDTH).any(|x| active[y][x] && children[node_id(x,y).0 as usize].len() > 1)) {
        return Ok(None);
    }

    let active_quota = (0..=LAST_ROW).filter(|&y| y != 13).map(|y| active[y].iter().filter(|&&v| v).count()).sum::<usize>();
    let assignable = (0..=LAST_ROW).map(|y| active[y].iter().filter(|&&v| v).count()).sum::<usize>()
        - [0usize, 8, 14].iter().map(|&y| active[y].iter().filter(|&&v| v).count()).sum::<usize>();
    let quotas = [round_half_up(active_quota, 5), round_half_up(active_quota, 12),
        round_half_up(active_quota, 8), round_half_up(active_quota, 22)];
    let monsters = assignable.checked_sub(quotas.iter().sum()).ok_or(RunError::Unsupported("room quota exceeds active map nodes"))?;
    let mut bucket = Vec::with_capacity(assignable);
    bucket.extend(std::iter::repeat(MapNodeKind::Shop).take(quotas[0]));
    bucket.extend(std::iter::repeat(MapNodeKind::Rest).take(quotas[1]));
    bucket.extend(std::iter::repeat(MapNodeKind::Elite).take(quotas[2]));
    bucket.extend(std::iter::repeat(MapNodeKind::Question).take(quotas[3]));
    bucket.extend(std::iter::repeat(MapNodeKind::Monster).take(monsters));
    for i in (1..bucket.len()).rev() {
        let j = rng.bounded_scoped(RunRngScope::MapRoomBucket, &[act as u64, draft, bucket.len() as u64, i as u64], (i + 1) as u64);
        bucket.swap(i, j);
    }

    let mut kinds = vec![None; (BOSS_ROW + 1) * WIDTH];
    for x in 0..WIDTH { if active[0][x] { kinds[node_id(x,0).0 as usize] = Some(MapNodeKind::Monster); } }
    for x in 0..WIDTH { if active[8][x] { kinds[node_id(x,8).0 as usize] = Some(MapNodeKind::Treasure); } }
    for x in 0..WIDTH { if active[14][x] { kinds[node_id(x,14).0 as usize] = Some(MapNodeKind::Rest); } }
    for y in 0..=LAST_ROW {
        for x in 0..WIDTH {
            if !active[y][x] || kinds[node_id(x,y).0 as usize].is_some() { continue; }
            let eligible = |kind: MapNodeKind| {
                let row_ok = match kind {
                    MapNodeKind::Shop | MapNodeKind::Question => (1..=13).contains(&y),
                    MapNodeKind::Rest => (5..=12).contains(&y),
                    MapNodeKind::Elite => (5..=13).contains(&y),
                    MapNodeKind::Monster => (1..=14).contains(&y),
                    _ => false,
                };
                if !row_ok { return false; }
                let parent_same = parents[node_id(x,y).0 as usize].iter().any(|p| kinds[*p] == Some(kind));
                let sibling_same = parents[node_id(x,y).0 as usize].iter().flat_map(|p| children[*p].iter())
                    .any(|s| *s != node_id(x,y).0 as usize && kinds[*s] == Some(kind));
                let parent_restricted = matches!(kind, MapNodeKind::Rest | MapNodeKind::Treasure | MapNodeKind::Shop | MapNodeKind::Elite);
                let sibling_restricted = matches!(kind, MapNodeKind::Rest | MapNodeKind::Monster | MapNodeKind::Question | MapNodeKind::Elite | MapNodeKind::Shop);
                !(parent_restricted && parent_same) && !(sibling_restricted && sibling_same)
            };
            if let Some(index) = (0..bucket.len()).find(|&i| eligible(bucket[i])) {
                let chosen = bucket.remove(index);
                kinds[node_id(x,y).0 as usize] = Some(chosen);
            }
        }
    }
    // Unassigned nodes are deterministically Monster, never patched to Elite.
    for y in 0..=LAST_ROW { for x in 0..WIDTH {
        let id = node_id(x,y).0 as usize;
        if active[y][x] && kinds[id].is_none() { kinds[id] = Some(MapNodeKind::Monster); }
    }}
    if !(0..=LAST_ROW).any(|y| (0..WIDTH).any(|x| kinds[node_id(x,y).0 as usize] == Some(MapNodeKind::Elite))) {
        return Ok(None);
    }

    let mut burning_candidates: Vec<usize> = (0..=LAST_ROW).flat_map(|y| (0..WIDTH).map(move |x| (x,y)))
        .filter(|&(x,y)| kinds[node_id(x,y).0 as usize] == Some(MapNodeKind::Elite)).map(|(x,y)| node_id(x,y).0 as usize).collect();
    let marker_eligible = eligibility.act4_enabled && !eligibility.emerald_key_owned;
    let burning = if marker_eligible && !burning_candidates.is_empty() {
        let i = rng.bounded_scoped(RunRngScope::MapBurningElite,
            &[act as u64, draft, burning_candidates.len() as u64], burning_candidates.len() as u64);
        Some(burning_candidates.swap_remove(i))
    } else { None };

    let mut nodes = Vec::new();
    for y in 0..=LAST_ROW { for x in 0..WIDTH {
        if !active[y][x] { continue; }
        let id = node_id(x,y);
        let kind = kinds[id.0 as usize].expect("every active node assigned");
        let hidden_kind = None;
        let outgoing = children[id.0 as usize].iter().map(|n| super::NodeId(*n as u64)).collect();
        nodes.push(MapNode { id, floor: y as u32 + 1, label: kind.label(), edges: outgoing,
            x: x as u8, y: y as u8, kind, hidden_kind, visited: false, revealed: y == 0,
            encounter: None, burning: burning == Some(id.0 as usize) });
    }}
    nodes.push(MapNode { id: boss, floor: 16, label: "Boss", edges: vec![], x: 3, y: 15,
        kind: MapNodeKind::Boss, hidden_kind: None, visited: false, revealed: false,
        encounter: None, burning: false });
    nodes.sort_by_key(|n| (n.y, n.x));
    Ok(Some(RunMap { act, draft_ordinal: draft, nodes, current: None, winged_moves: 0, encounter_history: Vec::new(),
        regular_encounter_history: Vec::new(), elite_encounter_history: Vec::new(),
        question_ramps: [0x3DCCCCCD, 0x3CF5C28F, 0x3CA3D70A], question_roll_ordinal: 0 }))
}
