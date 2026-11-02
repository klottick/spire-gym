//! Act 1 ordinary event bodies (spec m28c/spec/core.md §4).
use super::*;
use super::super::event_exec::{self, RelicChoice, RewardSpec};
use super::super::event_frame::DONE;
use crate::content::{CardType, Catalog, EncounterId, EventId};

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
    ..PLAIN
};

const GOLD_TOKEN:u32=0;
const NOTHING_TOKEN:u32=1;
const RELIC_TOKEN:u32=2;

/// Dead Adventurer (event-neow research :258; W rev 68427). Construction fixes a uniform enemy
/// and a once-shuffled GOLD/NOTHING/RELIC order; each safe Search consumes one token, while an
/// ambush keeps all remaining token rewards in its combat reward rows.
pub(in crate::run) const DEAD_ADVENTURER:Body=Body{
    construct:|cx|{
        let encounter=match cx.draw(0xDA001,3)?{
            0=>EncounterId::ThreeSentries,
            1=>EncounterId::GremlinNob,
            _=>EncounterId::LagavulinEvent,
        };
        let mut tokens=[GOLD_TOKEN,NOTHING_TOKEN,RELIC_TOKEN];
        for i in (1..tokens.len()).rev(){
            let j=cx.draw(0xDA100+i as u64,i+1)?;
            tokens.swap(i,j);
        }
        Ok(EventCaptures{values:tokens.to_vec(),encounter:Some(encounter),..Default::default()})
    },
    rows:|_,_,stage|if stage<3{vec![(0,None),(1,None)]}else{vec![(1,None)]},
    choose:|state,c,stage,option|Ok(match option{
        0 if stage<3=>{
            let encounter=c.encounter.ok_or(RunError::InvalidAction)?;
            let remaining=c.values.get(usize::from(stage)..).ok_or(RunError::InvalidAction)?;
            let has_gold=remaining.contains(&GOLD_TOKEN);
            let has_relic=remaining.contains(&RELIC_TOKEN);
            let ambush=vec![Outcome::Combat{
                encounter,
                rewards:Some(RewardSpec{
                    gold:if has_gold{30}else{0},
                    gold_range:Some((25,35)),
                    relics:if has_relic{vec![RelicChoice::RandomTier]}else{vec![]},
                }),
                next_stage:DONE,
                elite:true,
            }];
            let token=*c.values.get(usize::from(stage)).ok_or(RunError::InvalidAction)?;
            let mut safe=match token{
                GOLD_TOKEN=>vec![Outcome::GainGold(30)],
                NOTHING_TOKEN=>vec![],
                RELIC_TOKEN=>vec![Outcome::ObtainRelic(RelicChoice::RandomTier)],
                _=>return Err(RunError::InvalidAction),
            };
            safe.push(Outcome::NextStage(if stage==2{DONE}else{stage+1}));
            // Fight threshold 25/50/75, or 35/60/85 at A15 (DeadAdventurer constructor, randomReward).
            vec![Outcome::Roll{bound:100,arms:vec![((if a15(state){35}else{25})+25*u32::from(stage),ambush),(100,safe)]}]
        }
        1=>vec![Outcome::Leave],
        _=>return Err(RunError::InvalidAction),
    }),
    ..PLAIN
};

/// Golden Idol (event-neow research :259; W rev 68544). Take substitutes Circlet for an owned
/// Golden Idol, then opens the required Injury, captured NORMAL-damage, or captured max-HP cost.
pub(in crate::run) const GOLDEN_IDOL:Body=Body{
    // T(.25M) and max(1,T(.08M)); T(.35M) and max(1,T(.10M)) at A15 (GoldenIdolEvent constructor).
    construct:|cx|{
        let (damage,max_hp)=if a15(cx.state){(0.35_f32,0.10_f32)}else{(0.25_f32,0.08_f32)};
        Ok(EventCaptures{values:vec![
            ((cx.state.max_hp as f32)*damage) as u32,
            (((cx.state.max_hp as f32)*max_hp) as u32).max(1),
        ],..Default::default()})
    },
    rows:|_,_,stage|match stage{
        0=>vec![(0,None),(1,None)],
        1=>vec![(0,None),(1,None),(2,None)],
        _=>vec![],
    },
    choose:|state,c,stage,option|Ok(match (stage,option){
        (0,0)=>vec![Outcome::ObtainRelic(RelicChoice::Specific(if state.relics.contains(&RelicId::GoldenIdol){RelicId::Circlet}else{RelicId::GoldenIdol})),Outcome::NextStage(1)],
        (0,1)=>vec![Outcome::Leave],
        (1,0)=>vec![Outcome::ObtainCurse(CardId::Injury),Outcome::NextStage(DONE)],
        (1,1)=>vec![Outcome::Damage(c.values[0]),Outcome::NextStage(DONE)],
        (1,2)=>vec![Outcome::LoseMaxHp(c.values[1]),Outcome::NextStage(DONE)],
        _=>return Err(RunError::InvalidAction),
    }),
    ..PLAIN
};

/// Hypnotizing Colored Mushrooms (event-neow research :260; W rev 68570). The forced Continue
/// before the Fungi fight is collapsed into Fight; Eat heals from choice-time max HP.
pub(in crate::run) const MUSHROOMS:Body=Body{
    rows:|_,_,_|vec![(0,None),(1,None)],
    choose:|state,_,_,option|Ok(match option{
        0=>vec![Outcome::Combat{
            encounter:EncounterId::ThreeFungiBeasts,
            rewards:Some(RewardSpec{
                gold:0,
                gold_range:Some((20,30)),
                relics:vec![RelicChoice::Specific(if state.relics.contains(&RelicId::OddMushroom){RelicId::Circlet}else{RelicId::OddMushroom})],
            }),
            next_stage:DONE,
            elite:false,
        }],
        1=>vec![Outcome::Heal(((state.max_hp as f32)*0.25_f32) as u32),Outcome::ObtainCurse(CardId::Parasite),Outcome::NextStage(DONE)],
        _=>return Err(RunError::InvalidAction),
    }),
    ..PLAIN
};

/// Living Wall (event-neow research :261; W rev 68610). All three branches skip their selection
/// when P is empty; Grow is disabled when U1 is empty.
pub(in crate::run) const LIVING_WALL:Body=Body{
    rows:|_,_,_|vec![(0,None),(1,None),(2,None)],
    choose:|state,_,_,option|Ok(match option{
        0=>if event_exec::select_candidates(state,EventSelectOp::Remove).is_empty(){
            vec![Outcome::NextStage(DONE)]
        }else{
            vec![Outcome::Select{op:EventSelectOp::Remove,count:1,skippable:false},Outcome::NextStage(DONE)]
        },
        1=>if event_exec::select_candidates(state,EventSelectOp::Transform).is_empty(){
            vec![Outcome::NextStage(DONE)]
        }else{
            vec![Outcome::Select{op:EventSelectOp::Transform,count:1,skippable:false},Outcome::NextStage(DONE)]
        },
        2=>{
            if event_exec::select_candidates(state,EventSelectOp::Upgrade).is_empty(){return Err(RunError::InvalidAction)}
            if event_exec::select_candidates(state,EventSelectOp::Remove).is_empty(){
                vec![Outcome::NextStage(DONE)]
            }else{
                vec![Outcome::Select{op:EventSelectOp::Upgrade,count:1,skippable:false},Outcome::NextStage(DONE)]
            }
        }
        _=>return Err(RunError::InvalidAction),
    }),
    ..PLAIN
};

/// Scrap Ooze (event-neow research :262 BC; W rev 68770). Each Reach takes ownerless NORMAL
/// damage before its inclusive roll; stage counts failed Reaches and n=8 always succeeds.
pub(in crate::run) const SCRAP_OOZE:Body=Body{
    rows:|_,_,_|vec![(0,None),(1,None)],
    choose:|state,_,stage,option|Ok(match option{
        0=>{
            let fail_upper=74_u32.saturating_sub(10*u32::from(stage));
            // First Reach deals 3, or 5 at A15 (ScrapOoze constructor); each failure adds 1.
            vec![Outcome::Damage((if a15(state){5}else{3})+u32::from(stage)),Outcome::Roll{
                bound:100,
                arms:vec![
                    (fail_upper,vec![Outcome::NextStage(stage.saturating_add(1))]),
                    (100,vec![Outcome::ObtainRelic(RelicChoice::RandomTier),Outcome::NextStage(DONE)]),
                ],
            }]
        }
        1=>vec![Outcome::Leave],
        _=>return Err(RunError::InvalidAction),
    }),
    ..PLAIN
};

/// Shining Light (event-neow research :263; W rev 68790). Capture J(.20M) at construction,
/// then pass player-owned NORMAL damage before shuffling/upgrading up to two U1 instances.
pub(in crate::run) const SHINING_LIGHT:Body=Body{
    // J(.20M), or J(.30M) at A15 (ShiningLight constructor).
    construct:|cx|Ok(EventCaptures{values:vec![crate::run::rooms::shop::libgdx_round((cx.state.max_hp as f32)*if a15(cx.state){0.30_f32}else{0.20_f32})],..Default::default()}),
    rows:|_,_,_|vec![(0,None),(1,None)],
    choose:|state,c,_,option|Ok(match option{
        0=>{
            if event_exec::select_candidates(state,EventSelectOp::Upgrade).is_empty(){return Err(RunError::InvalidAction)}
            vec![Outcome::PlayerDamage(c.values[0]),Outcome::UpgradeRandom(2),Outcome::NextStage(DONE)]
        }
        1=>vec![Outcome::Leave],
        _=>return Err(RunError::InvalidAction),
    }),
    ..PLAIN
};

/// The Cleric (event-neow research :264; W rev 68864). The captured heal is paid for with 35;
/// Purify requires 50 and charges only when a purgeable card exists.
pub(in crate::run) const CLERIC:Body=Body{
    construct:|cx|Ok(EventCaptures{values:vec![((cx.state.max_hp as f32)*0.25_f32) as u32],..Default::default()}),
    // Purify costs 50, or 75 at A15 (Cleric constructor); the row shows its price either way.
    rows:|state,_,_|vec![(0,Some(35)),(1,Some(cleric_purify(state))),(2,None)],
    choose:|state,c,_,option|Ok(match option{
        0=>vec![Outcome::LoseGold(35),Outcome::Heal(c.values[0]),Outcome::NextStage(DONE)],
        1=>{
            if state.gold<cleric_purify(state){return Err(RunError::InvalidAction)}
            if event_exec::select_candidates(state,EventSelectOp::Remove).is_empty(){
                vec![Outcome::NextStage(DONE)]
            }else{
                vec![Outcome::LoseGold(cleric_purify(state)),Outcome::Select{op:EventSelectOp::Remove,count:1,skippable:false},Outcome::NextStage(DONE)]
            }
        }
        2=>vec![Outcome::Leave],
        _=>return Err(RunError::InvalidAction),
    }),
    ..PLAIN
};

fn cleric_purify(state:&RunState)->u32{if a15(state){75}else{50}}

/// The Ssssserpent (event-neow research :265; W rev 68877). D1 collapses its forced confirmation:
/// Agree schedules fixed Doubt before requesting 175 gold.
pub(in crate::run) const SSSSSERPENT:Body=Body{
    rows:|_,_,_|vec![(0,None),(1,None)],
    // 175 gold, or 150 at A15 (Sssserpent constructor).
    choose:|state,_,_,option|Ok(match option{
        0=>vec![Outcome::ObtainCurse(CardId::Doubt),Outcome::GainGold(if a15(state){150}else{175}),Outcome::NextStage(DONE)],
        1=>vec![Outcome::Leave],
        _=>return Err(RunError::InvalidAction),
    }),
    ..PLAIN
};

/// Wing Statue / Golden Wing (event-neow research :266; W rev 69027). Destroy eligibility is
/// captured at construction; its 50–80 gold is drawn when Destroy is chosen (`GoldenWing.buttonEffect`,
/// PM ruling on review-v1 source check). Pray applies SOFTLOCK-001 if P is empty.
pub(in crate::run) const WING_STATUE:Body=Body{
    construct:|cx|{
        let destroy_enabled=cx.state.deck.iter().any(|card|{
            if !Catalog::standard().card(card.id).is_some_and(|definition|definition.card_type==CardType::Attack){return false}
            let live=crate::combat::permanent_live_card_with_misc(card.id,crate::combat::CardInstanceId(card.instance),card.upgrades,card.misc);
            let damage=if card.id==CardId::RitualDagger{
                Some(live.misc)
            }else{
                crate::combat::static_card_face(card.id,card.upgrades).ok().and_then(|face|face.base_damage).or(live.local.base_damage)
            };
            damage.is_some_and(|value|value>=10)
        });
        Ok(EventCaptures{values:vec![u32::from(destroy_enabled)],..Default::default()})
    },
    rows:|_,_,_|vec![(0,None),(1,None),(2,None)],
    choose:|state,c,_,option|Ok(match option{
        0=>{
            let mut outcomes=vec![Outcome::PlayerDamage(7)];
            if event_exec::select_candidates(state,EventSelectOp::Remove).is_empty(){
                outcomes.push(Outcome::NextStage(DONE));
            }else{
                outcomes.push(Outcome::Select{op:EventSelectOp::Remove,count:1,skippable:false});
                outcomes.push(Outcome::NextStage(DONE));
            }
            outcomes
        }
        1=>{
            if c.values.first()!=Some(&1){return Err(RunError::InvalidAction)}
            vec![Outcome::Roll{bound:31,arms:(1..=31).map(|upper|(upper,vec![Outcome::GainGold(49+upper),Outcome::NextStage(DONE)])).collect()}]
        }
        2=>vec![Outcome::Leave],
        _=>return Err(RunError::InvalidAction),
    }),
    ..PLAIN
};

/// World of Goop (event-neow research :267; W rev 69029). Capture min(entry gold, uniform 20–50);
/// Gather applies player-owned NORMAL damage before its 75-gold request.
pub(in crate::run) const WORLD_OF_GOOP:Body=Body{
    construct:|cx|{
        // Leave loses 20–50, or 35–75 at A15 (GoopPuddle constructor), capped at the gold held.
        let rolled=if a15(cx.state){35+cx.draw(0xD1002,41)? as u32}else{20+cx.draw(0xD1002,31)? as u32};
        Ok(EventCaptures{values:vec![cx.state.gold.min(rolled)],..Default::default()})
    },
    rows:|_,_,_|vec![(0,None),(1,None)],
    choose:|_,c,_,option|Ok(match option{
        0=>vec![Outcome::PlayerDamage(11),Outcome::GainGold(75),Outcome::NextStage(DONE)],
        1=>vec![Outcome::LoseGold(c.values[0]),Outcome::NextStage(DONE)],
        _=>return Err(RunError::InvalidAction),
    }),
    ..PLAIN
};

/// Bodies landed in this file by M28d (dispatch from `events::body`). Big Fish remains dispatched
/// directly by `events::body` in `events/mod.rs`.
pub(super) fn body(event:EventId)->Option<&'static Body>{
    match event{
        EventId::DeadAdventurer=>Some(&DEAD_ADVENTURER),
        EventId::GoldenIdol=>Some(&GOLDEN_IDOL),
        EventId::Mushrooms=>Some(&MUSHROOMS),
        EventId::LivingWall=>Some(&LIVING_WALL),
        EventId::ScrapOoze=>Some(&SCRAP_OOZE),
        EventId::ShiningLight=>Some(&SHINING_LIGHT),
        EventId::Cleric=>Some(&CLERIC),
        EventId::Ssssserpent=>Some(&SSSSSERPENT),
        EventId::WingStatue=>Some(&WING_STATUE),
        EventId::WorldOfGoop=>Some(&WORLD_OF_GOOP),
        _=>None,
    }
}
