//! Act 3 ordinary event bodies (spec m28c/spec/core.md §4; M28f: PM check m28f/source-check-events-v1).
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
    choose:|state,c,_,option|Ok(match option{
        0=>vec![Outcome::Combat{encounter:c.encounter.ok_or(RunError::InvalidAction)?,rewards:Some(RewardSpec{gold:if a13(state){25}else{50},gold_range:None,relics:vec![RelicChoice::Tier(RelicTier::Rare)]}),next_stage:DONE,elite:false}],
        1=>vec![Outcome::UpgradeAll,Outcome::ObtainRelic(RelicChoice::Specific(RelicId::MarkOfBloom)),Outcome::NextStage(DONE)],
        2 if c.values[0]==1=>vec![Outcome::GainGold(999),Outcome::ObtainCurse(CardId::Normality),Outcome::ObtainCurse(CardId::Normality),Outcome::NextStage(DONE)],
        2=>vec![Outcome::Heal(c.values[1]),Outcome::ObtainCurse(CardId::Doubt),Outcome::NextStage(DONE)],
        _=>return Err(RunError::InvalidAction),
    }),
    ..PLAIN
};

/// `MathUtils.round` of a binary32 share of max HP (PM check m28f/source-check-events-v1).
fn share(max_hp:u32,fraction:f32)->u32{crate::run::rooms::shop::libgdx_round((max_hp as f32)*fraction)}

/// The Moai Head (PM check m28f/source-check-events-v1). Construction captures round(12.5% M), 18% at
/// A15. Jump Inside loses that max HP, then heals the resulting max HP; Trade (disabled without Golden
/// Idol) removes the Idol, then gains 333 gold directly; Leave.
pub(in crate::run) const MOAI_HEAD:Body=Body{
    construct:|cx|Ok(EventCaptures{values:vec![share(cx.state.max_hp,if a15(cx.state){0.18_f32}else{0.125_f32})],..Default::default()}),
    rows:|_,_,stage|if stage==0{vec![(0,None),(1,None),(2,None)]}else{vec![]},
    choose:|state,c,stage,option|{
        if stage!=0{return Err(RunError::InvalidAction)}
        Ok(match option{
            0=>{let a=*c.values.first().ok_or(RunError::InvalidAction)?;vec![Outcome::LoseMaxHp(a),Outcome::Heal(state.max_hp.saturating_sub(a).max(1)),Outcome::NextStage(DONE)]}
            1 if state.relics.contains(&RelicId::GoldenIdol)=>vec![Outcome::LoseRelic(RelicId::GoldenIdol),Outcome::GainGold(333),Outcome::NextStage(DONE)],
            2=>vec![Outcome::Leave],
            _=>return Err(RunError::InvalidAction),
        })
    },
    ..PLAIN
};

/// Mysterious Sphere (PM check m28f/source-check-events-v1). Fight: two Orb Walkers; the gold row
/// (uniform 45..=55) and a screenless Rare relic row are fixed before the fight (D1 collapses the
/// Continue page), then the EventRoom potion roll and card offer after it. Leave.
pub(in crate::run) const MYSTERIOUS_SPHERE:Body=Body{
    rows:|_,_,stage|if stage==0{vec![(0,None),(1,None)]}else{vec![]},
    choose:|_,_,stage,option|{
        if stage!=0{return Err(RunError::InvalidAction)}
        Ok(match option{
            0=>vec![Outcome::Combat{encounter:EncounterId::TwoOrbWalkers,
                rewards:Some(RewardSpec{gold:0,gold_range:Some((45,55)),relics:vec![RelicChoice::Screenless(RelicTier::Rare)]}),next_stage:DONE,elite:false}],
            1=>vec![Outcome::Leave],
            _=>return Err(RunError::InvalidAction),
        })
    },
    ..PLAIN
};

/// Sensory Stone (PM check m28f/source-check-events-v1). One, two or three colorless card offers;
/// two and three cost 5 and 10 HP_LOSS. The memory text's miscRng shuffle picks text only: no draw.
pub(in crate::run) const SENSORY_STONE:Body=Body{
    rows:|_,_,stage|if stage==0{vec![(0,None),(1,None),(2,None)]}else{vec![]},
    choose:|_,_,stage,option|{
        if stage!=0{return Err(RunError::InvalidAction)}
        Ok(match option{
            0=>vec![Outcome::ColourlessRewards(1)],
            // The source loses HP after building the rows; no row draw reads HP (frame control is last).
            1=>vec![Outcome::LoseHp(5),Outcome::ColourlessRewards(2)],
            2=>vec![Outcome::LoseHp(10),Outcome::ColourlessRewards(3)],
            _=>return Err(RunError::InvalidAction),
        })
    },
    ..PLAIN
};

/// Winding Halls (PM check m28f/source-check-events-v1). Construction captures round(12.5% M) damage,
/// round(25% M) heal and round(5% M) max HP (A15: 18%, 20%, 5%). Embrace Madness: NORMAL damage without
/// an owner, then two Madness; Rest: heal, then Writhe; the maze: lose max HP.
pub(in crate::run) const WINDING_HALLS:Body=Body{
    construct:|cx|{
        let (damage,heal)=if a15(cx.state){(0.18_f32,0.20_f32)}else{(0.125_f32,0.25_f32)};
        Ok(EventCaptures{values:vec![share(cx.state.max_hp,damage),share(cx.state.max_hp,heal),share(cx.state.max_hp,0.05_f32)],..Default::default()})
    },
    rows:|_,_,stage|if stage==0{vec![(0,None),(1,None),(2,None)]}else{vec![]},
    choose:|_,c,stage,option|{
        if stage!=0||c.values.len()<3{return Err(RunError::InvalidAction)}
        Ok(match option{
            0=>vec![Outcome::Damage(c.values[0]),Outcome::ObtainCard{id:CardId::Madness,upgrades:0},Outcome::ObtainCard{id:CardId::Madness,upgrades:0},Outcome::NextStage(DONE)],
            1=>vec![Outcome::Heal(c.values[1]),Outcome::ObtainCurse(CardId::Writhe),Outcome::NextStage(DONE)],
            2=>vec![Outcome::LoseMaxHp(c.values[2]),Outcome::NextStage(DONE)],
            _=>return Err(RunError::InvalidAction),
        })
    },
    ..PLAIN
};

/// The bodies landed in this file by M28f (dispatch from `events::body`).
pub(super) fn body(event:EventId)->Option<&'static Body>{
    match event{
        EventId::MoaiHead=>Some(&MOAI_HEAD),
        EventId::MysteriousSphere=>Some(&MYSTERIOUS_SPHERE),
        EventId::SensoryStone=>Some(&SENSORY_STONE),
        EventId::WindingHalls=>Some(&WINDING_HALLS),
        _=>None,
    }
}
