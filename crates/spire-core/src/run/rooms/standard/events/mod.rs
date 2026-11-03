//! Standard event bodies (design §5.3–§5.4). A body is three functions over the frame's captured
//! values: `construct` fixes them, `rows` lists a stage's options, `choose` maps an option to the
//! executor's outcomes. Bodies never mutate `RunState`.
use super::*;
use super::event_frame::EventCaptures;
use super::event_exec::Outcome;
mod act1;
mod act2;
mod act3;
mod shrines;
mod one_time;
#[cfg(test)]
pub(in crate::run) use act3::MIND_BLOOM_BOSSES;

/// The draw context of `construct` (`M28EventBody`, keyed by act, floor, node and event).
pub(in crate::run) struct BodyContext<'a> { pub state:&'a RunState, pub rng:&'a mut RunRng, pub key:[u64;5] }
impl BodyContext<'_> {
    pub(in crate::run) fn draw(&mut self,tag:u64,bound:usize)->Result<usize,RunError>{
        let key=[&self.key[..],&[tag]].concat();
        self.rng.bounded_scoped_checked(RunRngScope::M28EventBody,&key,bound as u64)
    }
}

pub(in crate::run) struct Body {
    pub construct:fn(&mut BodyContext)->Result<EventCaptures,RunError>,
    /// A stage's option rows: (option, gold price shown on the row).
    pub rows:fn(&RunState,&EventCaptures,u8)->Vec<(u8,Option<u32>)>,
    /// An option's outcomes; `InvalidAction` for an option the stage does not offer.
    pub choose:fn(&RunState,&EventCaptures,u8,u8)->Result<Vec<Outcome>,RunError>,
    /// An `EventSelectOp::Choose` result: the chosen cards' outcomes, run before the rest of the
    /// list (X3: Bonfire Spirits' offering, Note For Yourself's stored card).
    pub selected:Option<fn(&RunState,&EventCaptures,u8,&[CardInstance])->Result<Vec<Outcome>,RunError>>,
    /// A card board (X7, D2): the identity a stage's option row shows, if revealed.
    pub card:Option<fn(&EventCaptures,u8,u8)->Option<CardId>>,
    /// Identifies the stages whose rows are card options; other stages remain event options.
    pub card_stage:Option<fn(u8)->bool>,
    /// D2: the gift an option row's button names (We Meet Again's preselected potion and card).
    pub shows:Option<fn(&RunState,&EventCaptures,u8,u8)->Shows>,
}

/// What an option row reveals besides its price (D2).
pub(in crate::run) enum Shows{
    Nothing,
    /// The held potion the row gives up; `None` when no potion was preselected.
    Potion(Option<crate::content::PotionId>),
    /// The deck card instance the row gives up; `None` when no card was preselected.
    Card(Option<u64>),
    /// The owned relic the row offers (N'loth).
    Relic(Option<RelicId>),
}

/// The source's `AbstractDungeon.ascensionLevel >= 15` event band (PM source check
/// m28d/source-check-act-dependence-v1): the A15 amounts below are read from it.
pub(in crate::run) fn a15(state:&RunState)->bool{state.ascension>=15}

/// The defaults of the optional hooks, for `Body{.., ..PLAIN}`.
pub(in crate::run) const PLAIN:Body=Body{construct:|_|Ok(EventCaptures::default()),rows:|_,_,_|vec![],choose:|_,_,_,_|Err(RunError::InvalidAction),selected:None,card:None,card_stage:None,shows:None};

/// The body of `event`; `None` only for the six accepted events, which keep their own handlers and
/// never reach here.
pub(in crate::run) fn body(event:EventId)->Option<&'static Body>{
    match event{
        EventId::BigFish=>Some(&act1::BIG_FISH),
        EventId::Addict=>Some(&act2::ADDICT),
        EventId::MindBloom=>Some(&act3::MIND_BLOOM),
        EventId::GoldenShrine=>Some(&shrines::GOLDEN_SHRINE),
        _=>act1::body(event).or_else(||act2::body(event)).or_else(||act3::body(event)).or_else(||shrines::body(event)).or_else(||one_time::body(event)),
    }
}

/// The error of an event without a standard body (the accepted six use their own handlers).
pub(in crate::run) const NO_HANDLER:&str="event handler unavailable";
