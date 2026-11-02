//! Shrine event bodies (spec m28c/spec/core.md §4).
use super::*;
use super::super::event_frame::DONE;

/// Golden Shrine (event-neow research :273; W rev 68546): Pray +100 gold; Desecrate +275 gold and
/// Regret; Leave.
pub(in crate::run) const GOLDEN_SHRINE:Body=Body{
    construct:|_|Ok(EventCaptures::default()),
    rows:|_,_,_|vec![(0,None),(1,None),(2,None)],
    choose:|_,_,_,option|Ok(match option{
        0=>vec![Outcome::GainGold(100),Outcome::NextStage(DONE)],
        1=>vec![Outcome::GainGold(275),Outcome::ObtainCurse(CardId::Regret),Outcome::NextStage(DONE)],
        2=>vec![Outcome::Leave],
        _=>return Err(RunError::InvalidAction),
    }),
};
