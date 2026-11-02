//! Act 3 ordinary event bodies (spec m28c/spec/core.md §4).
use super::*;
use crate::content::{EncounterId, RelicTier};
use super::super::event_exec::{RelicChoice, RewardSpec};
use super::super::event_frame::DONE;

/// Mind Bloom's three Act 1 bosses, one drawn uniformly at construction (design §5.8).
pub(in crate::run) const MIND_BLOOM_BOSSES:[EncounterId;3]=[EncounterId::Guardian,EncounterId::Hexaghost,EncounterId::SlimeBoss];

/// Mind Bloom (acts23 research :203, :210-213; W rev 68646). I am War: the drawn Act 1 boss (a
/// BOSS-type fight, G13), then the EventRoom rewards plus 50 gold (A0; 25 from A13) and a Rare-tier
/// relic, with an ordinary card offer (design §5.7; PM source check m28c Q3). I am Awake: upgrade every
/// card and Mark of the Bloom. Third option by floor (S: floor % 50 <= 40): I am Rich (999 gold,
/// two Normality) or I am Healthy (heal max HP, captured, and Doubt).
pub(in crate::run) const MIND_BLOOM:Body=Body{
    construct:|cx|{
        let boss=MIND_BLOOM_BOSSES[cx.draw(0,MIND_BLOOM_BOSSES.len())?];
        Ok(EventCaptures{values:vec![u32::from(cx.state.floor%50<=40),cx.state.max_hp],encounter:Some(boss),..Default::default()})
    },
    rows:|_,_,_|vec![(0,None),(1,None),(2,None)],
    choose:|_,c,_,option|Ok(match option{
        0=>vec![Outcome::Combat{encounter:c.encounter.ok_or(RunError::InvalidAction)?,rewards:Some(RewardSpec{gold:50,relic_tiers:vec![RelicTier::Rare]}),next_stage:DONE}],
        1=>vec![Outcome::UpgradeAll,Outcome::ObtainRelic(RelicChoice::Specific(RelicId::MarkOfBloom)),Outcome::NextStage(DONE)],
        2 if c.values[0]==1=>vec![Outcome::GainGold(999),Outcome::ObtainCurse(CardId::Normality),Outcome::ObtainCurse(CardId::Normality),Outcome::NextStage(DONE)],
        2=>vec![Outcome::Heal(c.values[1]),Outcome::ObtainCurse(CardId::Doubt),Outcome::NextStage(DONE)],
        _=>return Err(RunError::InvalidAction),
    }),
};
