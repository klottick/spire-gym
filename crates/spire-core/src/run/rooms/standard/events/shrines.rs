//! Shrine event bodies (spec m28c/spec/core.md §4).
use super::*;
use super::super::event_exec::RelicChoice;
use super::super::event_frame::DONE;

/// Golden Shrine (event-neow research :273; W rev 68546): Pray +100 gold; Desecrate +275 gold and
/// Regret; Leave.
pub(in crate::run) const GOLDEN_SHRINE:Body=Body{
    construct:|_|Ok(EventCaptures::default()),
    rows:|_,_,_|vec![(0,None),(1,None),(2,None)],
    // Pray gives 100, or 50 at A15 (GoldShrine constructor); Desecrate is 275 at every ascension.
    choose:|state,_,_,option|Ok(match option{
        0=>vec![Outcome::GainGold(if a15(state){50}else{100}),Outcome::NextStage(DONE)],
        1=>vec![Outcome::GainGold(275),Outcome::ObtainCurse(CardId::Regret),Outcome::NextStage(DONE)],
        2=>vec![Outcome::Leave],
        _=>return Err(RunError::InvalidAction),
    }),
    params:Some(|s,c,stage,option|super::params(EventId::GoldenShrine,s,c,stage,option)),
    ..PLAIN
};

fn pool_card(cx:&mut BodyContext,tag:u64,pool:crate::content::pools::CardPool)->Result<CardId,RunError>{
    let cards=crate::content::pools::cards(pool).map_err(|_|RunError::Unsupported("event card pool is incomplete"))?;
    let index=cx.draw(tag,cards.len())?;
    cards.get(index).copied().ok_or(RunError::InvalidAction)
}

/// Match and Keep (event-neow research :274; W rev 68628). The six identities are paired and
/// shuffled at construction; E11 fixes the character starter-event card, while successful matches
/// make one fresh obtain so Omamori and egg hooks run through the shared owner.
pub(in crate::run) const MATCH_AND_KEEP:Body=Body{
    construct:|cx|{
        use crate::content::{CardRarity,Character};
        use crate::content::pools::CardPool;
        let character=cx.state.character;
        let rare=pool_card(cx,1,CardPool::Ordinary{character,rarity:Some(CardRarity::Rare),card_type:None})?;
        let uncommon=pool_card(cx,2,CardPool::Ordinary{character,rarity:Some(CardRarity::Uncommon),card_type:None})?;
        let common=pool_card(cx,3,CardPool::Ordinary{character,rarity:Some(CardRarity::Common),card_type:None})?;
        // At A15 a second random curse replaces the Uncommon colourless card (initializeCards).
        let fourth=if super::a15(cx.state){pool_card(cx,4,CardPool::Curse)?}else{pool_card(cx,4,CardPool::Colourless{rarity:Some(CardRarity::Uncommon)})?};
        let curse=pool_card(cx,5,CardPool::Curse)?;
        let starter=match character{
            Character::Ironclad=>CardId::Bash,
            Character::Silent=>CardId::Neutralize,
            Character::Defect=>CardId::Zap,
            Character::Watcher=>CardId::Eruption,
        };
        let mut card_ids=vec![rare,rare,uncommon,uncommon,common,common,fourth,fourth,curse,curse,starter,starter];
        for i in (1..card_ids.len()).rev(){let j=cx.draw(100+i as u64,i+1)?;card_ids.swap(i,j);}
        Ok(EventCaptures{values:vec![0,0,0],card_ids,..Default::default()})
    },
    rows:|_,c,stage|{
        if stage>=5||c.values.len()<3{return vec![]}
        let removed=c.values[0];
        (0..c.card_ids.len().min(12)).filter_map(|position|{
            let bit=1u32<<position;
            // The attempt's first card stays published face up (Disabled: `choose` rejects it) while
            // the second is chosen (D2; review-v1 I1).
            (removed&bit==0).then_some((position as u8,None))
        }).collect()
    },
    choose:|_,c,stage,option|{
        if stage>=5||c.values.len()<3||option>=12{return Err(RunError::InvalidAction)}
        let position=u32::from(option);
        let bit=1u32<<position;
        let removed=c.values[0];
        let first=c.values[1];
        let revealed=c.values[2];
        if removed&bit!=0||first==position+1{return Err(RunError::InvalidAction)}
        if first==0{
            return Ok(vec![
                Outcome::SetValue{slot:1,value:position+1},
                Outcome::SetValue{slot:2,value:revealed|bit},
                Outcome::NextStage(stage),
            ])
        }
        let first_position=first-1;
        let first_bit=1u32<<first_position;
        let matches=c.card_ids.get(first_position as usize)==c.card_ids.get(position as usize);
        let mut outcomes=Vec::new();
        if matches{
            let id=*c.card_ids.get(position as usize).ok_or(RunError::InvalidAction)?;
            outcomes.push(Outcome::ObtainCard{id,upgrades:0});
        }
        outcomes.extend([
            Outcome::SetValue{slot:0,value:if matches{removed|first_bit|bit}else{removed}},
            Outcome::SetValue{slot:1,value:0},
            Outcome::SetValue{slot:2,value:revealed|first_bit|bit},
        ]);
        let next=if stage+1>=5{DONE}else{stage+1};
        outcomes.push(Outcome::NextStage(next));
        Ok(outcomes)
    },
    card:Some(|c,_,option|{
        let position=usize::from(option);
        if position>=c.card_ids.len()||position>=12||c.values.len()<3{return None}
        (c.values[2]&(1u32<<position)!=0).then(||c.card_ids[position])
    }),
    card_stage:Some(|stage|stage<5),
    shows:None,
    params:None,
    selected:None,
};

/// Transmogrifier (event-neow research :276; W rev 68900). Transform one purgeable card; the
/// accepted SOFTLOCK-001 completion applies when the source would open an empty transform grid.
pub(in crate::run) const TRANSMOGRIFIER:Body=Body{
    rows:|_,_,stage|if stage==0{vec![(0,None),(1,None)]}else{vec![]},
    choose:|state,_,stage,option|{
        if stage!=0{return Err(RunError::InvalidAction)}
        match option{
            0 if super::super::event_exec::select_candidates(state,EventSelectOp::Transform).is_empty()=>Ok(vec![Outcome::NextStage(DONE)]),
            0=>Ok(vec![Outcome::Select{op:EventSelectOp::Transform,count:1,skippable:false},Outcome::NextStage(DONE)]),
            1=>Ok(vec![Outcome::Leave]),
            _=>Err(RunError::InvalidAction),
        }
    },
    ..PLAIN
};

/// Wheel of Change (event-neow research :278; BC comparator; W rev 69018). Spin reveals a keyed
/// six-arm result; gold commits at reveal, then the forced press applies the effect. D1 collapses
/// the result page after a relic reward's Proceed because it exposes no further decision.
pub(in crate::run) const WHEEL_OF_CHANGE:Body=Body{
    // Slot 0 is the constructor's `setGold()` amount by dungeon (PM static check of GremlinWheelGame;
    // review-v1 C1): Exordium 100, TheCity 200, TheBeyond 300. Slot 1 is unused: Heal(M) and T(.1M)
    // read M at confirmation, after any potion drunk in the reveal window (ER :278 "Others apply on
    // result confirmation"). Slot 2 is the revealed index.
    construct:|cx|{
        let gold=match cx.state.act{1=>100,2=>200,3=>300,_=>return Err(RunError::Unsupported("Wheel of Change outside Acts 1-3"))};
        Ok(EventCaptures{values:vec![gold,0,0],..Default::default()})
    },
    rows:|_,c,stage|match stage{
        0=>vec![(0,None)],
        1=>c.values.get(2).filter(|index|**index<6).map_or_else(Vec::new,|index|vec![(*index as u8,None)]),
        _=>vec![],
    },
    choose:|state,c,stage,option|{
        if stage==0&&option==0{
            return Ok(vec![Outcome::Roll{bound:6,arms:vec![
                (1,vec![Outcome::SetValue{slot:2,value:0},Outcome::GainGold(c.values.first().copied().ok_or(RunError::InvalidAction)?),Outcome::NextStage(1)]),
                (2,vec![Outcome::SetValue{slot:2,value:1},Outcome::NextStage(1)]),
                (3,vec![Outcome::SetValue{slot:2,value:2},Outcome::NextStage(1)]),
                (4,vec![Outcome::SetValue{slot:2,value:3},Outcome::NextStage(1)]),
                (5,vec![Outcome::SetValue{slot:2,value:4},Outcome::NextStage(1)]),
                (6,vec![Outcome::SetValue{slot:2,value:5},Outcome::NextStage(1)]),
            ]}])
        }
        if stage!=1||c.values.get(2).copied()!=Some(u32::from(option)){return Err(RunError::InvalidAction)}
        Ok(match option{
            0=>vec![Outcome::NextStage(DONE)],
            1=>vec![Outcome::Rewards{potions:0,relics:vec![RelicChoice::RandomTier]}],
            2=>vec![Outcome::Heal(state.max_hp),Outcome::NextStage(DONE)],
            3=>vec![Outcome::ObtainCurse(CardId::Decay),Outcome::NextStage(DONE)],
            4=>if super::super::event_exec::select_candidates(state,EventSelectOp::Remove).is_empty(){
                vec![Outcome::NextStage(DONE)]
            }else{
                vec![Outcome::Select{op:EventSelectOp::Remove,count:1,skippable:false},Outcome::NextStage(DONE)]
            },
            // T(.10M), or T(.15M) at A15 (GremlinWheelGame constructor), HP_LOSS.
            5=>vec![Outcome::LoseHp(((state.max_hp as f32)*if a15(state){0.15_f32}else{0.1_f32}) as u32),Outcome::NextStage(DONE)],
            _=>return Err(RunError::InvalidAction),
        })
    },
    params:Some(|s,c,stage,option|super::params(EventId::WheelOfChange,s,c,stage,option)),
    ..PLAIN
};

/// The bodies landed in this file by M28d (dispatch from `events::body`).
pub(super) fn body(event:EventId)->Option<&'static Body>{
    match event{
        EventId::MatchAndKeep=>Some(&MATCH_AND_KEEP),
        EventId::Transmogrifier=>Some(&TRANSMOGRIFIER),
        EventId::WheelOfChange=>Some(&WHEEL_OF_CHANGE),
        _=>None,
    }
}
