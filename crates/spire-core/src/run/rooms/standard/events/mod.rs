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
}

/// The landed body of `event`; `None` until its chunk lands (the six accepted events keep their
/// own handlers and never reach here).
pub(in crate::run) fn body(event:EventId)->Option<&'static Body>{
    match event{
        EventId::BigFish=>Some(&act1::BIG_FISH),
        EventId::Addict=>Some(&act2::ADDICT),
        EventId::MindBloom=>Some(&act3::MIND_BLOOM),
        EventId::GoldenShrine=>Some(&shrines::GOLDEN_SHRINE),
        _=>None,
    }
}

/// The typed interim of an event whose body has not landed (chunk plan §3–§4).
pub(in crate::run) fn interim(event:EventId)->&'static str{
    use EventId::*;
    match event{
        DeadAdventurer|GoldenIdol|Mushrooms|LivingWall|ScrapOoze|ShiningLight|Cleric|Ssssserpent|WingStatue|WorldOfGoop
        |MatchAndKeep|Transmogrifier|WheelOfChange|AccursedBlacksmith|BonfireSpirits|FaceTrader|FountainOfCleansing|Lab|NoteForYourself
            =>"M28 event body lands in M28d",
        Beggar|Colosseum|CursedTome|DrugDealer|ForgottenAltar|Ghosts|MaskedBandits|Nest|Library|Mausoleum|Vampires|WeMeetAgain|WomanInBlue
            =>"M28 event body lands in M28e",
        MoaiHead|MysteriousSphere|SensoryStone|WindingHalls|SecretPortal|Designer|Duplicator|KnowingSkull|Nloth=>"M28 event body lands in M28f",
        _=>"event handler unavailable",
    }
}
