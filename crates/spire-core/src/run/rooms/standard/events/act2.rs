//! Act 2 ordinary event bodies (spec m28c/spec/core.md §4; M28e plan-check m28e/plan-check-v1).
use super::*;
use super::super::event_exec::{RelicChoice, RewardSpec};
use crate::content::{EncounterId, RelicTier};
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
    ..PLAIN
};

/// `ceil(binary32(M × f))` max-HP loss, kept below M (Ghosts, Vampires: `MathUtils.ceil`, then
/// `maxHealth − 1` if it would take all of it), captured at construction.
fn max_hp_cost(max_hp:u32,f:f32)->u32{
    let loss=((max_hp as f32)*f).ceil() as u32;
    if loss>=max_hp{max_hp.saturating_sub(1)}else{loss}
}

/// Council of Ghosts (acts23 research :191; W rev; spec m28e batch2). Accept loses the captured
/// `ceil(.5M)` max HP, then obtains five Apparitions (three from A15); Refuse leaves.
pub(in crate::run) const GHOSTS:Body=Body{
    construct:|cx|Ok(EventCaptures{values:vec![max_hp_cost(cx.state.max_hp,0.5)],..Default::default()}),
    rows:|_,_,stage|if stage==0{vec![(0,None),(1,None)]}else{vec![]},
    choose:|state,c,stage,option|{
        if stage!=0{return Err(RunError::InvalidAction)}
        Ok(match option{
            0=>{
                let mut list=vec![Outcome::LoseMaxHp(c.values[0])];
                list.extend((0..if a15(state){3}else{5}).map(|_|Outcome::ObtainCard{id:CardId::Apparition,upgrades:0}));
                list.push(Outcome::NextStage(DONE));list
            }
            1=>vec![Outcome::Leave],
            _=>return Err(RunError::InvalidAction),
        })
    },
    params:Some(|s,c,stage,option|super::params(EventId::Ghosts,s,c,stage,option)),
    ..PLAIN
};

/// Vampires (acts23 research :196; G1). Accept loses the captured `ceil(.3M)` max HP; Give Blood
/// Vial (offered while owned) loses it instead; both then remove every starter Strike and obtain
/// five Bites. Refuse leaves.
pub(in crate::run) const VAMPIRES:Body=Body{
    construct:|cx|Ok(EventCaptures{values:vec![max_hp_cost(cx.state.max_hp,0.3)],..Default::default()}),
    rows:|state,_,stage|if stage!=0{vec![]}else if state.relics.contains(&RelicId::BloodVial){vec![(0,None),(1,None),(2,None)]}else{vec![(0,None),(2,None)]},
    choose:|state,c,stage,option|{
        if stage!=0{return Err(RunError::InvalidAction)}
        let cost=match option{
            0=>Outcome::LoseMaxHp(c.values[0]),
            1 if state.relics.contains(&RelicId::BloodVial)=>Outcome::LoseRelic(RelicId::BloodVial),
            2=>return Ok(vec![Outcome::Leave]),
            _=>return Err(RunError::InvalidAction),
        };
        // PM check m28e/source-check-mechanics-c-v1: removals precede the five queued Bite obtains.
        let mut list=vec![cost,Outcome::RemoveTagged(crate::content::CardTag::StarterStrike)];
        list.extend((0..5).map(|_|Outcome::ObtainCard{id:CardId::Bite,upgrades:0}));
        list.push(Outcome::NextStage(DONE));
        Ok(list)
    },
    params:Some(|s,c,stage,option|super::params(EventId::Vampires,s,c,stage,option)),
    ..PLAIN
};

/// The Mausoleum (acts23 research :195; PM check m28e/source-check-mechanics-c-v1). Open draws the
/// Writhe boolean (forced true from A15, the draw kept), queues that effect before selecting a
/// screenless relic of an ordinary tier roll; Leave.
pub(in crate::run) const MAUSOLEUM:Body=Body{
    rows:|_,_,stage|if stage==0{vec![(0,None),(1,None)]}else{vec![]},
    choose:|state,_,stage,option|{
        if stage!=0{return Err(RunError::InvalidAction)}
        Ok(match option{
            0=>{
                let writhe=vec![Outcome::ObtainCurse(CardId::Writhe),Outcome::ObtainRelic(RelicChoice::RandomTier),Outcome::NextStage(DONE)];
                let clean=if a15(state){writhe.clone()}else{vec![Outcome::ObtainRelic(RelicChoice::RandomTier),Outcome::NextStage(DONE)]};
                vec![Outcome::Roll{bound:2,arms:vec![(1,writhe),(2,clean)]}]
            }
            1=>vec![Outcome::Leave],
            _=>return Err(RunError::InvalidAction),
        })
    },
    params:Some(|s,c,stage,option|super::params(EventId::Mausoleum,s,c,stage,option)),
    ..PLAIN
};

/// The Nest (acts23 research :193). Smash and Grab gains 99 gold (50 from A15); Stay in Line takes 6
/// damage, then obtains Ritual Dagger.
pub(in crate::run) const NEST:Body=Body{
    rows:|_,_,stage|if stage==0{vec![(0,None),(1,None)]}else{vec![]},
    choose:|state,_,stage,option|{
        if stage!=0{return Err(RunError::InvalidAction)}
        Ok(match option{
            0=>vec![Outcome::GainGold(if a15(state){50}else{99}),Outcome::NextStage(DONE)],
            // PM check m28e/source-check-mechanics-c-v1: null-owner damage is NORMAL.
            1=>vec![Outcome::Damage(6),Outcome::ObtainCard{id:CardId::RitualDagger,upgrades:0},Outcome::NextStage(DONE)],
            _=>return Err(RunError::InvalidAction),
        })
    },
    params:Some(|s,c,stage,option|super::params(EventId::Nest,s,c,stage,option)),
    ..PLAIN
};

/// Beggar: pay 75 gold, then select one purgeable card; an empty purge group pays and completes
/// the event (SOFTLOCK-001). Leave is free.
pub(in crate::run) const BEGGAR:Body=Body{
    rows:|_,_,stage|if stage==0{vec![(0,Some(75)),(1,None)]}else{vec![]},
    choose:|state,_,stage,option|{
        if stage!=0{return Err(RunError::InvalidAction)}
        match option{
            0=>{
                let mut outcomes=vec![Outcome::LoseGold(75)];
                if !super::super::event_exec::select_candidates(state,EventSelectOp::Remove).is_empty(){
                    outcomes.push(Outcome::Select{op:EventSelectOp::Remove,count:1,skippable:false});
                }
                outcomes.push(Outcome::NextStage(DONE));
                Ok(outcomes)
            }
            1=>Ok(vec![Outcome::Leave]),
            _=>Err(RunError::InvalidAction),
        }
    },
    ..PLAIN
};

/// Cursed Tome: costs for pages 1–3 are separate stages; final Take cost is captured on entry.
pub(in crate::run) const CURSED_TOME:Body=Body{
    construct:|cx|Ok(EventCaptures{values:vec![if a15(cx.state){15}else{10}],..Default::default()}),
    rows:|_,_,stage|match stage{
        0=>vec![(0,None),(1,None)],
        1..=3=>vec![(0,None)],
        4=>vec![(0,None),(1,None)],
        _=>vec![],
    },
    choose:|state,c,stage,option|{
        match (stage,option){
            (0,0)=>Ok(vec![Outcome::NextStage(1)]),
            (0,1)=>Ok(vec![Outcome::Leave]),
            (1..=3,0)=>Ok(vec![Outcome::LoseHp(u32::from(stage)),Outcome::NextStage(stage+1)]),
            (4,0)=>{
                let missing=[RelicId::Necronomicon,RelicId::Enchiridion,RelicId::NilrysCodex]
                    .into_iter().filter(|id|!state.relics.contains(id)).collect::<Vec<_>>();
                // PM check m28e/source-check-mechanics-c-v1: the relic is a skippable reward;
                // Proceed finishes the room.
                if missing.is_empty(){
                    Ok(vec![Outcome::LoseHp(c.values[0]),Outcome::Rewards{potions:0,relics:vec![RelicChoice::Specific(RelicId::Circlet)]}])
                }else{
                    let arms=missing.iter().enumerate().map(|(index,id)|
                        (index as u32+1,vec![Outcome::Rewards{potions:0,relics:vec![RelicChoice::Specific(*id)]}])).collect();
                    Ok(vec![Outcome::LoseHp(c.values[0]),Outcome::Roll{bound:missing.len() as u32,arms}])
                }
            }
            (4,1)=>Ok(vec![Outcome::LoseHp(3),Outcome::NextStage(DONE)]),
            _=>Err(RunError::InvalidAction),
        }
    },
    params:Some(|s,c,stage,option|super::params(EventId::CursedTome,s,c,stage,option)),
    ..PLAIN
};

/// Drug Dealer: J.A.X., a two-card transform selection, or Mutagenic Strength/Circlet.
pub(in crate::run) const DRUG_DEALER:Body=Body{
    rows:|_,_,stage|if stage==0{vec![(0,None),(1,None),(2,None)]}else{vec![]},
    choose:|state,_,stage,option|{
        if stage!=0{return Err(RunError::InvalidAction)}
        match option{
            0=>Ok(vec![Outcome::ObtainCard{id:CardId::Jax,upgrades:0},Outcome::NextStage(DONE)]),
            1=>Ok(vec![Outcome::Select{op:EventSelectOp::Transform,count:2,skippable:false},Outcome::NextStage(DONE)]),
            2=>Ok(vec![Outcome::ObtainRelic(RelicChoice::Specific(if state.relics.contains(&RelicId::MutagenicStrength){RelicId::Circlet}else{RelicId::MutagenicStrength})),Outcome::NextStage(DONE)]),
            _=>Err(RunError::InvalidAction),
        }
    },
    params:Some(|s,c,stage,option|super::params(EventId::DrugDealer,s,c,stage,option)),
    ..PLAIN
};

/// Forgotten Altar: the Idol cost is captured by the entry relic state; the sacrifice damage is
/// rounded from the pre-gain maximum HP at construction.
pub(in crate::run) const FORGOTTEN_ALTAR:Body=Body{
    construct:|cx|Ok(EventCaptures{values:vec![crate::run::rooms::shop::libgdx_round((cx.state.max_hp as f32)*if a15(cx.state){0.35_f32}else{0.25_f32})],..Default::default()}),
    rows:|_,_,stage|if stage==0{vec![(0,None),(1,None),(2,None)]}else{vec![]},
    choose:|state,c,stage,option|{
        if stage!=0{return Err(RunError::InvalidAction)}
        match option{
            0 if state.relics.contains(&RelicId::GoldenIdol)=>{
                if state.relics.contains(&RelicId::BloodyIdol){
                    Ok(vec![Outcome::ObtainRelic(RelicChoice::Specific(RelicId::Circlet)),Outcome::NextStage(DONE)])
                }else{
                    // PM check m28e/source-check-mechanics-c-v1 item 3: Golden Idol's slot, no equip effect.
                    Ok(vec![Outcome::ReplaceRelic{old:RelicId::GoldenIdol,new:RelicId::BloodyIdol},Outcome::NextStage(DONE)])
                }
            }
            1=>{
                // PM check m28e/source-check-mechanics-c-v1: null-owner damage is NORMAL.
                Ok(vec![Outcome::GainMaxHp(5),Outcome::Damage(c.values[0]),Outcome::NextStage(DONE)])
            }
            2=>Ok(vec![Outcome::ObtainCurse(CardId::Decay),Outcome::NextStage(DONE)]),
            _=>Err(RunError::InvalidAction),
        }
    },
    params:Some(|s,c,stage,option|super::params(EventId::ForgottenAltar,s,c,stage,option)),
    ..PLAIN
};

/// The Library: Sleep uses the captured rounded heal; Read generates its 20-card board at choice.
pub(in crate::run) const LIBRARY:Body=Body{
    construct:|cx|Ok(EventCaptures{values:vec![crate::run::rooms::shop::libgdx_round((cx.state.max_hp as f32)*if a15(cx.state){0.20_f32}else{0.33_f32})],..Default::default()}),
    rows:|_,c,stage|match stage{
        0=>vec![(0,None),(1,None)],
        1=>(0..c.card_ids.len() as u8).map(|option|(option,None)).collect(),
        _=>vec![],
    },
    choose:|_,c,stage,option|match (stage,option){
        (0,0)=>Ok(vec![Outcome::CardBoard{count:20},Outcome::NextStage(1)]),
        (0,1)=>Ok(vec![Outcome::Heal(c.values[0]),Outcome::NextStage(DONE)]),
        (1,position)=>{
            let id=*c.card_ids.get(usize::from(position)).ok_or(RunError::InvalidAction)?;
            // PM check m28e/source-check-mechanics-c-v1: the Read grid has no Cancel button.
            Ok(vec![Outcome::ObtainCard{id,upgrades:0},Outcome::NextStage(DONE)])
        }
        _=>Err(RunError::InvalidAction),
    },
    card:Some(|c,stage,option|if stage==1{c.card_ids.get(usize::from(option)).copied()}else{None}),
    card_stage:Some(|stage|stage==1),
    params:Some(|s,c,stage,option|super::params(EventId::Library,s,c,stage,option)),
    ..PLAIN
};

/// The Colosseum (acts23 research :187; PM checks m28e B item 12, C item 7, escape N3). Stage 0's
/// single row starts fight 1 (the source's intro and Continue pages collapse under D1): Blue and Red
/// Slaver with rewards disabled, reopening at stage 1 after a victory or escape. Stage 1: Escape
/// leaves; Fight Nobs is Taskmaster and Gremlin Nob with eliteTrigger, a Rare then an Uncommon relic
/// row taken at fight start, 100 gold and the EventRoom card offer.
pub(in crate::run) const COLOSSEUM:Body=Body{
    rows:|_,_,stage|match stage{0=>vec![(0,None)],1=>vec![(0,None),(1,None)],_=>vec![]},
    choose:|_,_,stage,option|Ok(match (stage,option){
        (0,0)=>vec![Outcome::Combat{encounter:EncounterId::ColosseumSlavers,rewards:None,next_stage:1,elite:false}],
        (1,0)=>vec![Outcome::Leave],
        (1,1)=>vec![Outcome::Combat{encounter:EncounterId::ColosseumNobs,
            rewards:Some(RewardSpec{gold:100,gold_range:None,relics:vec![RelicChoice::Tier(RelicTier::Rare),RelicChoice::Tier(RelicTier::Uncommon)]}),
            next_stage:DONE,elite:true}],
        _=>return Err(RunError::InvalidAction),
    }),
    params:Some(|s,c,stage,option|super::params(EventId::Colosseum,s,c,stage,option)),
    ..PLAIN
};

/// Masked Bandits (acts23 research :192; PM checks m28e A, B items 13a/13b). Pay loses all gold.
/// Fight: Pointy, Romeo, Bear (not an elite); 25–35 gold drawn at the fight choice and Red Mask, or
/// Circlet if owned, as reward rows with the EventRoom card offer.
pub(in crate::run) const MASKED_BANDITS:Body=Body{
    rows:|_,_,stage|if stage==0{vec![(0,None),(1,None)]}else{vec![]},
    choose:|state,_,stage,option|Ok(match (stage,option){
        (0,0)=>vec![Outcome::LoseAllGold,Outcome::NextStage(DONE)],
        (0,1)=>vec![Outcome::Combat{encounter:EncounterId::MaskedBandits,
            rewards:Some(RewardSpec{gold:0,gold_range:Some((25,35)),
                relics:vec![RelicChoice::Specific(if state.relics.contains(&RelicId::RedMask){RelicId::Circlet}else{RelicId::RedMask})]}),
            next_stage:DONE,elite:false}],
        _=>return Err(RunError::InvalidAction),
    }),
    ..PLAIN
};

pub(in crate::run) fn body(event:EventId)->Option<&'static Body>{
    match event{
        EventId::Beggar=>Some(&BEGGAR),
        EventId::CursedTome=>Some(&CURSED_TOME),
        EventId::DrugDealer=>Some(&DRUG_DEALER),
        EventId::ForgottenAltar=>Some(&FORGOTTEN_ALTAR),
        EventId::Ghosts=>Some(&GHOSTS),
        EventId::Colosseum=>Some(&COLOSSEUM),
        EventId::MaskedBandits=>Some(&MASKED_BANDITS),
        EventId::Vampires=>Some(&VAMPIRES),
        EventId::Mausoleum=>Some(&MAUSOLEUM),
        EventId::Nest=>Some(&NEST),
        EventId::Library=>Some(&LIBRARY),
        _=>None,
    }
}
