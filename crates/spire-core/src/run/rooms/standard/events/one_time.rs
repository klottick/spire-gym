//! One-time event bodies (spec m28d/spec/batch1.md): run-wide, never replenished (design §5.2).
use super::*;
use super::super::event_exec::RelicChoice;
use super::super::event_frame::DONE;

/// Accursed Blacksmith (event-neow research :284; W rev 68677). Forge upgrades one U1 card;
/// Rummage obtains Pain followed by fixed Warped Tongs; Leave exits.
pub(in crate::run) const ACCURSED_BLACKSMITH:Body=Body{
    rows:|_,_,stage|if stage==0{vec![(0,None),(1,None),(2,None)]}else{vec![]},
    choose:|state,_,stage,option|{
        if stage!=0{return Err(RunError::InvalidAction)}
        match option{
            0 if super::super::event_exec::select_candidates(state,EventSelectOp::Upgrade).is_empty()=>Err(RunError::InvalidAction),
            0=>Ok(vec![Outcome::Select{op:EventSelectOp::Upgrade,count:1,skippable:false},Outcome::NextStage(DONE)]),
            1=>Ok(vec![Outcome::ObtainCurse(CardId::Pain),Outcome::ObtainRelic(RelicChoice::Specific(RelicId::WarpedTongs)),Outcome::NextStage(DONE)]),
            2=>Ok(vec![Outcome::Leave]),
            _=>Err(RunError::InvalidAction),
        }
    },
    ..PLAIN
};

/// Bonfire Spirits (event-neow research :285; W rev 68333). Choose one purgeable card; its catalog
/// rarity determines the reward, and the chosen instance is removed after that reward.
pub(in crate::run) const BONFIRE_SPIRITS:Body=Body{
    rows:|_,_,stage|if stage==0{vec![(0,None)]}else{vec![]},
    choose:|state,_,stage,option|{
        if stage!=0||option!=0{return Err(RunError::InvalidAction)}
        if super::super::event_exec::select_candidates(state,EventSelectOp::Choose).is_empty(){
            Ok(vec![Outcome::NextStage(DONE)])
        }else{
            Ok(vec![Outcome::Select{op:EventSelectOp::Choose,count:1,skippable:false},Outcome::NextStage(DONE)])
        }
    },
    selected:Some(|state,_,stage,cards|{
        if stage!=0||cards.len()!=1{return Err(RunError::InvalidAction)}
        let card=&cards[0];
        let rarity=crate::content::Catalog::standard().card(card.id).ok_or(RunError::InvalidAction)?.rarity;
        let mut outcomes=match rarity{
            crate::content::CardRarity::Basic=>vec![],
            crate::content::CardRarity::Common|crate::content::CardRarity::Special=>vec![Outcome::Heal(5)],
            crate::content::CardRarity::Uncommon=>vec![Outcome::Heal(state.max_hp)],
            crate::content::CardRarity::Rare=>vec![
                Outcome::GainMaxHp(10),
                Outcome::Heal(state.max_hp.checked_add(10).ok_or(RunError::InvalidAction)?),
            ],
            crate::content::CardRarity::Curse=>vec![Outcome::ObtainRelic(RelicChoice::Specific(
                if state.relics.contains(&RelicId::SpiritPoop){RelicId::Circlet}else{RelicId::SpiritPoop}
            ))],
            _=>return Err(RunError::InvalidAction),
        };
        outcomes.push(Outcome::RemoveSpecific(card.instance));
        Ok(outcomes)
    }),
    ..PLAIN
};

/// Face Trader (event-neow research :286; W rev 68497). Capture NORMAL damage from entry max HP.
/// Trade draws one face the player does not own when it is chosen (`getRandomFace` in
/// `FaceTrader.buttonEffect`, PM ruling on review-v1 source check), falling back to Circlet.
pub(in crate::run) const FACE_TRADER:Body=Body{
    construct:|cx|{
        let damage=(cx.state.max_hp/10).max(1);
        Ok(EventCaptures{values:vec![damage],..Default::default()})
    },
    rows:|_,_,stage|if stage==0{vec![(0,None),(1,None),(2,None)]}else{vec![]},
    choose:|state,c,stage,option|{
        if stage!=0{return Err(RunError::InvalidAction)}
        match option{
            // Touch gives 75 gold, or 50 at A15 (FaceTrader constructor).
            0=>Ok(vec![Outcome::GainGold(if super::a15(state){50}else{75}),Outcome::Damage(c.values[0]),Outcome::NextStage(DONE)]),
            1=>{
                let missing=[RelicId::CultistHeadpiece,RelicId::FaceOfCleric,RelicId::GremlinVisage,RelicId::NlothsHungryFace,RelicId::SsserpentHead]
                    .into_iter().filter(|id|!state.relics.contains(id)).collect::<Vec<_>>();
                if missing.is_empty(){return Ok(vec![Outcome::ObtainRelic(RelicChoice::Specific(RelicId::Circlet)),Outcome::NextStage(DONE)])}
                Ok(vec![Outcome::Roll{bound:missing.len() as u32,arms:missing.iter().enumerate().map(|(i,face)|
                    (i as u32+1,vec![Outcome::ObtainRelic(RelicChoice::Specific(*face)),Outcome::NextStage(DONE)])).collect()}])
            }
            2=>Ok(vec![Outcome::Leave]),
            _=>Err(RunError::InvalidAction),
        }
    },
    ..PLAIN
};

/// Fountain of Cleansing (event-neow research :287; W rev 68868). Remove qualifying curses in
/// reverse deck order, excluding the three protected identities and cards marked by Flame/Lightning.
pub(in crate::run) const FOUNTAIN_OF_CLEANSING:Body=Body{
    rows:|_,_,stage|if stage==0{vec![(0,None),(1,None)]}else{vec![]},
    choose:|state,_,stage,option|{
        if stage!=0{return Err(RunError::InvalidAction)}
        match option{
            0=>{
                let mut outcomes=Vec::new();
                for card in state.deck.iter().rev(){
                    let is_curse=crate::content::Catalog::standard().card(card.id).is_some_and(|definition|definition.rarity==crate::content::CardRarity::Curse);
                    let protected=matches!(card.id,CardId::AscendersBane|CardId::CurseOfTheBell|CardId::Necronomicurse);
                    let bottled=state.relic_cards.get(&RelicId::BottledFlame)==Some(&card.instance)
                        ||state.relic_cards.get(&RelicId::BottledLightning)==Some(&card.instance);
                    if is_curse&&!protected&&!bottled{outcomes.push(Outcome::RemoveSpecific(card.instance));}
                }
                outcomes.push(Outcome::NextStage(DONE));
                Ok(outcomes)
            },
            1=>Ok(vec![Outcome::Leave]),
            _=>Err(RunError::InvalidAction),
        }
    },
    ..PLAIN
};

/// Lab (event-neow research :288; W rev 68599). Continue constructs three flat, character-pool
/// potion rows on the shared non-combat reward screen; Proceed finishes directly to the map.
pub(in crate::run) const LAB:Body=Body{
    rows:|_,_,stage|if stage==0{vec![(0,None)]}else{vec![]},
    choose:|state,_,stage,option|if stage==0&&option==0{
        // Three potions, or two at A15 (Lab.buttonEffect).
        Ok(vec![Outcome::Rewards{potions:if super::a15(state){2}else{3},relics:vec![]}])
    }else{Err(RunError::InvalidAction)},
    ..PLAIN
};

/// Note For Yourself (event-neow research :289; W rev 68274). Directly obtain the stored profile
/// card, then let the player choose and store any purgeable instance (including the retrieved card).
pub(in crate::run) const NOTE_FOR_YOURSELF:Body=Body{
    rows:|_,_,stage|if stage==0{vec![(0,None),(1,None)]}else{vec![]},
    choose:|_,_,stage,option|{
        if stage!=0{return Err(RunError::InvalidAction)}
        match option{
            0=>Ok(vec![Outcome::ObtainStoredNote,Outcome::Select{op:EventSelectOp::Choose,count:1,skippable:false},Outcome::NextStage(DONE)]),
            1=>Ok(vec![Outcome::Leave]),
            _=>Err(RunError::InvalidAction),
        }
    },
    selected:Some(|_,_,stage,cards|{
        if stage!=0||cards.len()!=1{return Err(RunError::InvalidAction)}
        Ok(vec![Outcome::StoreNote(cards[0].instance),Outcome::RemoveSpecific(cards[0].instance)])
    }),
    ..PLAIN
};

/// The bodies landed in this file by M28d (dispatch from `events::body`).
pub(super) fn body(event:EventId)->Option<&'static Body>{
    match event{
        EventId::AccursedBlacksmith=>Some(&ACCURSED_BLACKSMITH),
        EventId::BonfireSpirits=>Some(&BONFIRE_SPIRITS),
        EventId::FaceTrader=>Some(&FACE_TRADER),
        EventId::FountainOfCleansing=>Some(&FOUNTAIN_OF_CLEANSING),
        EventId::Lab=>Some(&LAB),
        EventId::NoteForYourself=>Some(&NOTE_FOR_YOURSELF),
        EventId::WeMeetAgain=>Some(&WE_MEET_AGAIN),
        EventId::WomanInBlue=>Some(&WOMAN_IN_BLUE),
        _=>None,
    }
}

/// We Meet Again! (event-neow research :290; PM checks m28e/source-check-open-a-v1, -b-v1 item 11).
/// Construction, in source order: a uniform held potion slot, the gold amount (none below 50 gold;
/// else uniform 50..=min(150, G)), a uniform non-Basic non-Curse deck card; an empty list draws
/// nothing and disables its row. Each gift then obtains a screenless relic of an ordinary tier
/// roll; Attack leaves. The buttons name the preselected potion, gold amount (the row price) and
/// card (D2). Potion use and discard are locked from construction until the next room (frame).
pub(in crate::run) const WE_MEET_AGAIN:Body=Body{
    construct:|cx|{
        let held:Vec<u32>=cx.state.potions.iter().enumerate().filter(|(_,p)|p.is_some()).map(|(i,_)|i as u32).collect();
        let potion=if held.is_empty(){0}else{held[cx.draw(1,held.len())?]+1};
        let gold=if cx.state.gold<50{0}else{50+cx.draw(2,(cx.state.gold.min(150)-49) as usize)? as u32};
        let catalog=crate::content::Catalog::standard();
        let gifts:Vec<u64>=cx.state.deck.iter().filter(|c|catalog.card(c.id).is_some_and(|d|d.rarity!=crate::content::CardRarity::Basic&&d.card_type!=crate::content::CardType::Curse)).map(|c|c.instance).collect();
        let cards=if gifts.is_empty(){vec![]}else{vec![gifts[cx.draw(3,gifts.len())?]]};
        Ok(EventCaptures{values:vec![potion,gold],cards,..Default::default()})
    },
    rows:|_,c,stage|if stage==0{vec![(0,None),(1,(c.values[1]>0).then_some(c.values[1])),(2,None),(3,None)]}else{vec![]},
    shows:Some(|state,c,stage,option|match (stage,option){
        (0,0)=>super::Shows::Potion(c.values[0].checked_sub(1).and_then(|slot|state.potions.get(slot as usize).copied().flatten())),
        (0,2)=>super::Shows::Card(c.cards.first().copied()),
        _=>super::Shows::Nothing,
    }),
    choose:|_,c,stage,option|{
        if stage!=0{return Err(RunError::InvalidAction)}
        // PM check m28e/source-check-mechanics-c-v1: spawnRelicAndObtain is screenless.
        let gift=match option{
            0 if c.values[0]>0=>Outcome::LosePotion((c.values[0]-1) as u8),
            1 if c.values[1]>0=>Outcome::LoseGold(c.values[1]),
            2 if !c.cards.is_empty()=>Outcome::RemoveSpecific(c.cards[0]),
            3=>return Ok(vec![Outcome::Leave]),
            _=>return Err(RunError::InvalidAction),
        };
        Ok(vec![gift,Outcome::ObtainRelic(RelicChoice::RandomTier),Outcome::NextStage(DONE)])
    },
    ..PLAIN
};

/// The Woman in Blue: buy one, two, or three flat character-pool potions, or leave. At A15+,
/// Leave reads the current maximum HP and requests five-percent HP_LOSS.
pub(in crate::run) const WOMAN_IN_BLUE:Body=Body{
    rows:|_,_,stage|if stage==0{vec![(0,Some(20)),(1,Some(30)),(2,Some(40)),(3,None)]}else{vec![]},
    choose:|state,_,stage,option|{
        if stage!=0{return Err(RunError::InvalidAction)}
        match option{
            0..=2=>{
                let price=[20,30,40][usize::from(option)];
                // PM check m28e/source-check-mechanics-c-v1: Proceed finishes the room.
                Ok(vec![Outcome::LoseGold(price),Outcome::Rewards{potions:option+1,relics:vec![]}])
            }
            3 if a15(state)=>Ok(vec![Outcome::LoseHp(((state.max_hp as f32)*0.05_f32).ceil() as u32),Outcome::Leave]),
            3=>Ok(vec![Outcome::Leave]),
            _=>Err(RunError::InvalidAction),
        }
    },
    ..PLAIN
};
