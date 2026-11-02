use spire_env::RunObservation;

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Policy {
    Uniform,
    First,
    EndTurnAsap,
    PotionHeavy,
    ChooseAll,
    ChooseNone,
}

pub const POLICIES: [Policy; 6] = [
    Policy::Uniform,
    Policy::First,
    Policy::EndTurnAsap,
    Policy::PotionHeavy,
    Policy::ChooseAll,
    Policy::ChooseNone,
];

impl Policy {
    pub fn name(self) -> &'static str {
        match self {
            Self::Uniform => "uniform",
            Self::First => "first",
            Self::EndTurnAsap => "end_turn_asap",
            Self::PotionHeavy => "potion_heavy",
            Self::ChooseAll => "choose_all",
            Self::ChooseNone => "choose_none",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        POLICIES.into_iter().find(|policy| policy.name() == name)
    }

    pub fn for_seed(seed: u64) -> Self {
        POLICIES[(mix64(seed) as usize) % POLICIES.len()]
    }
}

/// SplitMix64 finalizer, used only for deterministic harness choices.
pub fn mix64(mut z: u64) -> u64 {
    z = z.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// Choose a published action index using only the numeric public observation,
/// seed and decision step. The token is carried by `Decision` but is never read.
pub fn choose_action(observation: &RunObservation, _token: [u64; 2], policy: Policy, seed: u64, step: usize) -> Option<usize> {
    let legal = observation.action_mask.iter().enumerate()
        .filter_map(|(index, mask)| (*mask != 0).then_some(index))
        .collect::<Vec<_>>();
    if legal.is_empty() { return None; }
    let verbs = observation.group("actions");
    let verb_at = |index: usize| -> i64 {
        let Some(group) = verbs else { return -1 };
        let columns = group.shape.last().copied().unwrap_or(0);
        if columns == 0 { return -1; }
        group.values.get(index.saturating_mul(columns)).copied().unwrap_or(-1)
    };
    let is_deselect = |index: usize| {
        let verb = verb_at(index);
        let source_group = observation.group("actions").and_then(|group| {
            let columns = group.shape.last().copied().unwrap_or(0);
            (columns > 1).then(|| group.values.get(index.saturating_mul(columns) + 1).copied()).flatten()
        }).unwrap_or(-1);
        verb == 15 || (verb == 13 && source_group == 4)
    };
    let safe = legal.iter().copied().filter(|index| !is_deselect(*index)).collect::<Vec<_>>();
    let choices = if safe.is_empty() { &legal } else { &safe };
    let choose_verb = |wanted: &[i64]| choices.iter().copied().find(|index| wanted.contains(&verb_at(*index)));
    let random = || choices[(mix64(seed ^ (step as u64).wrapping_mul(0xD6E8_FEB8_6659_FD93)) as usize) % choices.len()];
    match policy {
        Policy::Uniform => Some(random()),
        Policy::First => choices.first().copied().or_else(|| choose_verb(&[11, 4])).or_else(|| legal.first().copied()),
        Policy::EndTurnAsap => choose_verb(&[10]).or_else(|| Some(random())),
        Policy::PotionHeavy => choose_verb(&[6]).or_else(|| Some(random())),
        Policy::ChooseAll => choose_verb(&[3, 9]).or_else(|| choose_verb(&[4, 11])).or_else(|| Some(random())),
        Policy::ChooseNone => choose_verb(&[4, 5, 11, 13]).or_else(|| Some(random())),
    }
}

/// Choose using action intent tags without claiming that they are an encoded
/// observation. CompleteItemsRun has no accepted profile encoder, but its
/// candidates still expose the same policy-relevant action kinds.
#[cfg(feature = "diagnostics")]
pub(crate) fn choose_action_with_kinds(mask: &[i8], kinds: &[i64], policy: Policy, seed: u64, step: usize) -> Option<usize> {
    let legal = mask.iter().enumerate().filter_map(|(index, mask)| (*mask != 0).then_some(index)).collect::<Vec<_>>();
    if legal.is_empty() { return None; }
    let kind_at = |index: usize| kinds.get(index).copied().unwrap_or(-1);
    let safe = legal.iter().copied().filter(|index| kind_at(*index) != 15).collect::<Vec<_>>();
    let choices = if safe.is_empty() { &legal } else { &safe };
    let choose_kind = |wanted: &[i64]| choices.iter().copied().find(|index| wanted.contains(&kind_at(*index)));
    let random = || choices[(mix64(seed ^ (step as u64).wrapping_mul(0xD6E8_FEB8_6659_FD93)) as usize) % choices.len()];
    match policy {
        Policy::Uniform => Some(random()),
        Policy::First => choices.first().copied().or_else(|| choose_kind(&[11, 4])).or_else(|| legal.first().copied()),
        Policy::EndTurnAsap => choose_kind(&[10]).or_else(|| Some(random())),
        Policy::PotionHeavy => choose_kind(&[6]).or_else(|| Some(random())),
        Policy::ChooseAll => choose_kind(&[3, 9]).or_else(|| choose_kind(&[4, 11])).or_else(|| Some(random())),
        Policy::ChooseNone => choose_kind(&[4, 5, 11, 13]).or_else(|| Some(random())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use spire_env::RunFeatureGroup;

    #[test]
    fn selection_policies_skip_watcher_deselect_and_confirm() {
        let observation = RunObservation {
            groups: vec![RunFeatureGroup { key: "actions", shape: vec![3, 9],
                values: [13, 4, 3, 0, -1, 0, 0, 0, -1,
                    9, 4, 4, 0, -1, 0, 0, 0, -1,
                    11, 0, -1, 0, -1, 0, 0, 0, -1].to_vec(),
                known: vec![1; 27], occupied: Some(vec![1; 3]) }],
            action_mask: vec![1, 1, 1],
        };
        assert_eq!(choose_action(&observation, [0, 0], Policy::First, 0, 0), Some(1));
        assert_eq!(choose_action(&observation, [0, 0], Policy::ChooseNone, 0, 0), Some(2));
    }

    #[test]
    fn selection_policies_skip_complete_combat_deselect_without_source_group() {
        let observation = RunObservation {
            groups: vec![RunFeatureGroup { key: "actions", shape: vec![2, 9],
                values: [15, 0, 0, 0, 0, 0, 0, 0, 0,
                    9, 0, 0, 0, 0, 0, 0, 0, 0].to_vec(),
                known: vec![1; 18], occupied: Some(vec![1; 2]) }],
            action_mask: vec![1, 1],
        };
        assert_eq!(choose_action(&observation, [0, 0], Policy::First, 0, 0), Some(1));
    }
}
