use crate::content::{CardId, Character};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RunConfig { pub character: Character, pub ascension: u8 }

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CreationProfile {
    pub neow_eligible: bool,
    pub seen_bosses: [Vec<&'static str>; 3],
    pub stored_note: (CardId, u32),
    pub fully_unlocked: bool,
    pub act4_enabled: bool,
}
impl CreationProfile {
    pub fn experienced() -> Self { Self {
        neow_eligible: true,
        seen_bosses: [vec!["Guardian", "Hexaghost", "SlimeBoss"], vec!["Champ", "Automaton", "Collector"], vec!["AwakenedOne", "DonuAndDeca", "TimeEater"]],
        stored_note: (CardId::IronWave, 0), fully_unlocked: true, act4_enabled: true,
    }}
}
