use super::*;
use crate::combat::Outcome;

#[derive(Clone, Debug)]
pub(super) struct Proof {
    pub full_hp_start: bool,
    pub entry: Option<(u64, EncounterId)>,
    pub target: EncounterId,
    pub victories: Vec<(usize, CombatRecord)>,
    pub proceed: Vec<usize>,
    pub dialogs: Vec<(usize, u8)>,
    pub ending: Vec<usize>,
    pub endpoint: Option<(u8, u32, RunTerminal)>,
    pub result_no_keys: bool,
    pub forbidden_advance: bool,
    pub ending_reward: i32,
}

pub(super) fn qualifies(proof: &Proof) -> bool {
    let Some((visit, encounter))=proof.entry else { return false; };
    if !proof.full_hp_start || encounter != proof.target || !proof.result_no_keys
        || proof.forbidden_advance || proof.ending_reward != 0
        || proof.endpoint != Some((3,50,RunTerminal::Act3NoKeys))
        || proof.victories.len()!=1 || proof.proceed.len()!=1
        || proof.dialogs.len()!=4 || proof.ending.len()!=1 { return false; }
    let (victory_index, record)=&proof.victories[0];
    let proceed=proof.proceed[0];
    record.visit==visit && record.encounter==encounter && record.outcome==Outcome::Victory
        && *victory_index < proceed
        && proof.dialogs.iter().map(|(_,stage)|*stage).eq(1..=4)
        && proceed < proof.dialogs[0].0
        && proof.dialogs.windows(2).all(|pair|pair[0].0 < pair[1].0)
        && proof.ending[0]==proof.dialogs[3].0
}

#[test]
fn fixture_completion_oracle_requires_full_real_ordered_ending() {
    let good = Proof {
        full_hp_start: true, entry: Some((0, EncounterId::TimeEater)), target: EncounterId::TimeEater,
        victories: vec![(10, CombatRecord { visit: 0, encounter: EncounterId::TimeEater, outcome: Outcome::Victory })],
        proceed: vec![11], dialogs: vec![(12,1),(13,2),(14,3),(15,4)], ending: vec![15],
        endpoint: Some((3,50,RunTerminal::Act3NoKeys)), result_no_keys: true,
        forbidden_advance: false, ending_reward: 0,
    };
    assert!(qualifies(&good));
    for axis in 0..16 {
        let mut bad=good.clone();
        match axis {
            0=>bad.full_hp_start=false, 1=>bad.entry=None,
            2=>bad.victories[0].1.visit=1, 3=>bad.victories[0].1.encounter=EncounterId::DonuAndDeca,
            4=>bad.victories[0].1.outcome=Outcome::Death,
            5=>bad.proceed.clear(), 6=>{bad.dialogs.remove(2);},
            7=>bad.dialogs.swap(0,1), 8=>bad.ending.clear(),
            9=>bad.endpoint=Some((3,51,RunTerminal::Act3NoKeys)),
            10=>bad.result_no_keys=false, 11=>bad.forbidden_advance=true,
            12=>bad.ending_reward=1, 13=>bad.target=EncounterId::AwakenedOne,
            14=>bad.proceed[0]=9, 15=>bad.ending[0]=14, _=>unreachable!(),
        };
        assert!(!qualifies(&bad), "negative oracle axis {axis}");
    }
}
