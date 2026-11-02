//! Act 2 ordinary event bodies (spec m28c/spec/core.md §4).
use super::*;
use super::super::event_exec::RelicChoice;
use super::super::event_frame::DONE;

/// The Addict / Pleading Vagrant (acts23 research :184; W rev 68706; PM source check m28c Q1-Q2):
/// no spawn gate; Offer Gold (85, the row price, disabled below it) rolls a relic tier, takes a
/// screenless relic of it, pays, then grants it; Rob takes such a relic and Shame; Leave. No A15 branch.
pub(in crate::run) const ADDICT:Body=Body{
    construct:|_|Ok(EventCaptures::default()),
    rows:|_,_,_|vec![(0,Some(85)),(1,None),(2,None)],
    choose:|_,_,_,option|Ok(match option{
        0=>vec![Outcome::BuyRelic{choice:RelicChoice::RandomTier,gold:85},Outcome::NextStage(DONE)],
        1=>vec![Outcome::ObtainRelic(RelicChoice::RandomTier),Outcome::ObtainCurse(CardId::Shame),Outcome::NextStage(DONE)],
        2=>vec![Outcome::Leave],
        _=>return Err(RunError::InvalidAction),
    }),
};
