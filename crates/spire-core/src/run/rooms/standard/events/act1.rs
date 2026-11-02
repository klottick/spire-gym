//! Act 1 ordinary event bodies (spec m28c/spec/core.md §4).
use super::*;
use super::super::event_exec::RelicChoice;
use super::super::event_frame::DONE;

/// Big Fish (event-neow research :257, :347-353; W rev 68316). Banana heals max HP / 3 (integer
/// division, captured at construction); Donut +5 max HP; Box a random-tier screenless relic and
/// Regret (relic first: the curse is shown and obtained by a later effect, F1).
pub(in crate::run) const BIG_FISH:Body=Body{
    construct:|cx|Ok(EventCaptures{values:vec![cx.state.max_hp/3],..Default::default()}),
    rows:|_,_,_|vec![(0,None),(1,None),(2,None)],
    choose:|_,c,_,option|Ok(match option{
        0=>vec![Outcome::Heal(c.values[0]),Outcome::NextStage(DONE)],
        1=>vec![Outcome::GainMaxHp(5),Outcome::NextStage(DONE)],
        2=>vec![Outcome::ObtainRelic(RelicChoice::RandomTier),Outcome::ObtainCurse(CardId::Regret),Outcome::NextStage(DONE)],
        _=>return Err(RunError::InvalidAction),
    }),
};
