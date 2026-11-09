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
    /// M34a a3: the numbers an option row displays (stage, option), in `PARAM_SEMANTICS` order.
    pub params:Option<fn(&RunState,&EventCaptures,u8,u8)->[Option<i32>;3]>,
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

/// The source's `AbstractDungeon.ascensionLevel >= 13` event band (M31 source check §D): Mind Bloom.
pub(in crate::run) fn a13(state:&RunState)->bool{state.ascension>=13}

/// The defaults of the optional hooks, for `Body{.., ..PLAIN}`.
/// Pure projection of the numbers printed in a standard event row.
fn params(event:EventId,state:&RunState,c:&EventCaptures,stage:u8,option:u8)->[Option<i32>;3]{
    let captured=|index:usize|c.values.get(index).copied().map(|value|value as i32);
    let high=state.ascension>=15;
    let one=|value|[value,None,None];
    match (event,stage,option) {
        (EventId::BigFish,0,0)=>one(captured(0)),
        (EventId::BigFish,0,1)=>one(Some(5)),
        (EventId::DeadAdventurer,0..=2,0)=>one(Some((if high{35}else{25})+25*i32::from(stage))),
        (EventId::GoldenIdol,1,1)=>one(captured(0)),
        (EventId::GoldenIdol,1,2)=>one(captured(1)),
        (EventId::Mushrooms,0,1)=>one(Some(((state.max_hp as f32)*0.25_f32) as i32)),
        (EventId::ScrapOoze,0..=8,0)=>[
            Some((if high{5}else{3})+i32::from(stage)),Some(25+10*i32::from(stage)),None],
        (EventId::ShiningLight,0,0)=>[captured(0),Some(2),None],
        (EventId::Cleric,0,0)=>one(captured(0)),
        (EventId::Ssssserpent,0,0)=>one(Some(if high{150}else{175})),
        (EventId::WingStatue,0,0)=>one(Some(7)),
        (EventId::WingStatue,0,1)=>[Some(50),Some(80),None],
        (EventId::WingStatue,0,2)=>one(Some(10)),
        (EventId::WorldOfGoop,0,0)=>[Some(75),Some(11),None],
        (EventId::WorldOfGoop,0,1)=>one(captured(0)),

        (EventId::Ghosts,0,0)=>[
            captured(0),Some(if high{3}else{5}),None],
        (EventId::Vampires,0,0)=>[
            captured(0),Some(5),None],
        (EventId::Vampires,0,1)=>one(Some(5)),
        (EventId::Mausoleum,0,0)=>one(Some(if high{100}else{50})),
        (EventId::Nest,0,0)=>one(Some(if high{50}else{99})),
        (EventId::Nest,0,1)=>one(Some(6)),
        (EventId::CursedTome,1..=3,0)=>one(Some(i32::from(stage))),
        (EventId::CursedTome,4,0)=>one(captured(0)),
        (EventId::CursedTome,4,1)=>one(Some(3)),
        (EventId::DrugDealer,0,1)=>one(Some(2)),
        (EventId::ForgottenAltar,0,1)=>[
            Some(5),captured(0),None],
        (EventId::Library,0,1)=>one(captured(0)),
        // Colosseum's opening 4200 is prose, not a gameplay quantity; the boss reward is 100 gold.
        (EventId::Colosseum,1,1)=>one(Some(100)),
        (EventId::MindBloom,0,2) if c.values.first()==Some(&1)=>one(Some(999)),
        (EventId::MoaiHead,0,0)=>one(captured(0)),
        (EventId::MoaiHead,0,1)=>one(Some(333)),
        (EventId::SensoryStone,0,1)=>[Some(5),Some(2),None],
        (EventId::SensoryStone,0,2)=>[Some(10),Some(3),None],
        (EventId::WindingHalls,0,0)=>[captured(0),Some(2),None],
        (EventId::WindingHalls,0,1)=>one(captured(1)),
        (EventId::WindingHalls,0,2)=>one(captured(2)),

        (EventId::WeMeetAgain,0,1)=>one(captured(1)),
        (EventId::FaceTrader,0,0)=>[captured(0),Some(if high{50}else{75}),None],
        (EventId::WomanInBlue,0,0)=>[Some(20),Some(1),None],
        (EventId::WomanInBlue,0,1)=>[Some(30),Some(2),None],
        (EventId::WomanInBlue,0,2)=>[Some(40),Some(3),None],
        (EventId::WomanInBlue,0,3) if high=>one(Some(((state.max_hp as f32)*0.05_f32).ceil() as i32)),
        (EventId::Designer,0,0|1)=>[
            Some(if high{50}else{40}),if option==1{Some(2)}else{None},None],
        (EventId::Designer,0,2|3)=>[
            Some(if high{75}else{60}),if option==3{Some(2)}else{None},None],
        (EventId::Designer,0,4)=>one(Some(if high{110}else{90})),
        (EventId::Designer,0,5)=>one(Some(if high{5}else{3})),
        (EventId::KnowingSkull,0,0..=2)=>[
            captured(usize::from(option)).map(|uses|uses+6),if option==1{Some(90)}else{None},None],
        (EventId::KnowingSkull,0,3)=>one(Some(6)),
        (EventId::GoldenShrine,0,0)=>one(Some(if high{50}else{100})),
        (EventId::GoldenShrine,0,1)=>one(Some(275)),
        (EventId::WheelOfChange,1,5)=>one(Some(((state.max_hp as f32)*if high{0.15_f32}else{0.10_f32}) as i32)),
        _=>[None;3],
    }
}

/// The defaults of the optional hooks, for `Body{.., ..PLAIN}`.
pub(in crate::run) const PLAIN:Body=Body{construct:|_|Ok(EventCaptures::default()),rows:|_,_,_|vec![],choose:|_,_,_,_|Err(RunError::InvalidAction),selected:None,card:None,card_stage:None,shows:None,params:None};

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

/// The meaning of one published event param (M34a a3; census `m34a/event-display-census-v1`).
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct ParamMeaning { pub event:EventId, pub stage:u8, pub option:u8, pub index:u8, pub meaning:&'static str }

/// Every published event param's meaning (M34a a3), keyed by (event, stage, option, index).
macro_rules! meaning { ($event:ident,$stage:expr,$option:expr,$index:expr,$label:literal) => {
    ParamMeaning{event:EventId::$event,stage:$stage,option:$option,index:$index,meaning:$label}
} }
pub const PARAM_SEMANTICS:&[ParamMeaning]=&[
    meaning!(BigFish,0,0,0,"heal_amount"), meaning!(BigFish,0,1,0,"max_hp_gained"),
    meaning!(DeadAdventurer,0,0,0,"ambush_chance_percent"), meaning!(DeadAdventurer,1,0,0,"ambush_chance_percent"), meaning!(DeadAdventurer,2,0,0,"ambush_chance_percent"),
    meaning!(GoldenIdol,1,1,0,"damage_taken"), meaning!(GoldenIdol,1,2,0,"max_hp_lost"),
    meaning!(Mushrooms,0,1,0,"heal_amount"),
    meaning!(ScrapOoze,0,0,0,"damage_taken"), meaning!(ScrapOoze,0,0,1,"relic_chance_percent"),
    meaning!(ScrapOoze,1,0,0,"damage_taken"), meaning!(ScrapOoze,1,0,1,"relic_chance_percent"),
    meaning!(ScrapOoze,2,0,0,"damage_taken"), meaning!(ScrapOoze,2,0,1,"relic_chance_percent"),
    meaning!(ScrapOoze,3,0,0,"damage_taken"), meaning!(ScrapOoze,3,0,1,"relic_chance_percent"),
    meaning!(ScrapOoze,4,0,0,"damage_taken"), meaning!(ScrapOoze,4,0,1,"relic_chance_percent"),
    meaning!(ScrapOoze,5,0,0,"damage_taken"), meaning!(ScrapOoze,5,0,1,"relic_chance_percent"),
    meaning!(ScrapOoze,6,0,0,"damage_taken"), meaning!(ScrapOoze,6,0,1,"relic_chance_percent"),
    meaning!(ScrapOoze,7,0,0,"damage_taken"), meaning!(ScrapOoze,7,0,1,"relic_chance_percent"),
    meaning!(ScrapOoze,8,0,0,"damage_taken"), meaning!(ScrapOoze,8,0,1,"relic_chance_percent"),
    meaning!(ShiningLight,0,0,0,"damage_taken"), meaning!(ShiningLight,0,0,1,"upgrade_count"),
    meaning!(Cleric,0,0,0,"heal_amount"), meaning!(Ssssserpent,0,0,0,"gold_gained"),
    meaning!(WingStatue,0,0,0,"damage_taken"), meaning!(WingStatue,0,1,0,"gold_min"), meaning!(WingStatue,0,1,1,"gold_max"), meaning!(WingStatue,0,2,0,"damage_threshold"),
    meaning!(WorldOfGoop,0,0,0,"gold_gained"), meaning!(WorldOfGoop,0,0,1,"damage_taken"), meaning!(WorldOfGoop,0,1,0,"gold_lost"),
    meaning!(Ghosts,0,0,0,"max_hp_lost"), meaning!(Ghosts,0,0,1,"apparition_count"),
    meaning!(Vampires,0,0,0,"max_hp_lost"), meaning!(Vampires,0,0,1,"bite_count"), meaning!(Vampires,0,1,0,"bite_count"),
    meaning!(Mausoleum,0,0,0,"writhe_chance_percent"), meaning!(Nest,0,0,0,"gold_gained"), meaning!(Nest,0,1,0,"damage_taken"),
    meaning!(CursedTome,1,0,0,"hp_lost"), meaning!(CursedTome,2,0,0,"hp_lost"), meaning!(CursedTome,3,0,0,"hp_lost"), meaning!(CursedTome,4,0,0,"hp_lost"), meaning!(CursedTome,4,1,0,"hp_lost"),
    meaning!(DrugDealer,0,1,0,"transform_count"), meaning!(ForgottenAltar,0,1,0,"max_hp_gained"), meaning!(ForgottenAltar,0,1,1,"damage_taken"), meaning!(Library,0,1,0,"heal_amount"),
    meaning!(Colosseum,1,1,0,"gold_gained"), meaning!(MindBloom,0,2,0,"gold_gained"),
    meaning!(MoaiHead,0,0,0,"max_hp_lost"), meaning!(MoaiHead,0,1,0,"gold_gained"),
    meaning!(SensoryStone,0,1,0,"hp_lost"), meaning!(SensoryStone,0,1,1,"card_count"), meaning!(SensoryStone,0,2,0,"hp_lost"), meaning!(SensoryStone,0,2,1,"card_count"),
    meaning!(WindingHalls,0,0,0,"damage_taken"), meaning!(WindingHalls,0,0,1,"madness_count"), meaning!(WindingHalls,0,1,0,"heal_amount"), meaning!(WindingHalls,0,2,0,"max_hp_lost"),
    meaning!(WeMeetAgain,0,1,0,"gold_gift_amount"),
    meaning!(FaceTrader,0,0,0,"damage_taken"), meaning!(FaceTrader,0,0,1,"gold_gained"),
    meaning!(WomanInBlue,0,0,0,"gold_cost"), meaning!(WomanInBlue,0,0,1,"potion_count"),
    meaning!(WomanInBlue,0,1,0,"gold_cost"), meaning!(WomanInBlue,0,1,1,"potion_count"),
    meaning!(WomanInBlue,0,2,0,"gold_cost"), meaning!(WomanInBlue,0,2,1,"potion_count"), meaning!(WomanInBlue,0,3,0,"hp_lost"),
    meaning!(Designer,0,0,0,"gold_cost"), meaning!(Designer,0,1,0,"gold_cost"), meaning!(Designer,0,1,1,"card_count"),
    meaning!(Designer,0,2,0,"gold_cost"), meaning!(Designer,0,3,0,"gold_cost"), meaning!(Designer,0,3,1,"card_count"),
    meaning!(Designer,0,4,0,"gold_cost"), meaning!(Designer,0,5,0,"hp_lost"),
    meaning!(KnowingSkull,0,0,0,"hp_lost"), meaning!(KnowingSkull,0,1,0,"hp_lost"), meaning!(KnowingSkull,0,1,1,"gold_gained"), meaning!(KnowingSkull,0,2,0,"hp_lost"), meaning!(KnowingSkull,0,3,0,"hp_lost"),
    meaning!(GoldenShrine,0,0,0,"gold_gained"), meaning!(GoldenShrine,0,1,0,"gold_gained"),
    meaning!(WheelOfChange,1,5,0,"hp_lost"),
];
