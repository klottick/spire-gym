//! Public-only normal-start caller, distinct from supplied late-state diagnostics.
#![allow(dead_code)]
#[path="../m8_support/mod.rs"] mod previous;
pub use previous::Step;
use spire_core::{combat::{CombatAction,rng::Partition},content::{CardId,Character,EncounterId,EventId,RelicId},run::*,run::map::PublicMap};
use std::collections::BTreeMap;

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum M9BossHistory { AwakenedUnseen,TimeEaterUnseen,DonuDecaUnseen }
impl M9BossHistory{
    pub fn seen(self)->Vec<&'static str>{match self{Self::AwakenedUnseen=>vec!["TimeEater","DonuAndDeca"],Self::TimeEaterUnseen=>vec!["AwakenedOne","DonuAndDeca"],Self::DonuDecaUnseen=>vec!["AwakenedOne","TimeEater"]}}
    pub fn boss(self)->EncounterId{match self{Self::AwakenedUnseen=>EncounterId::AwakenedOne,Self::TimeEaterUnseen=>EncounterId::TimeEater,Self::DonuDecaUnseen=>EncounterId::DonuAndDeca}}
}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum M9Completion { Act3NoKeys,Death,EmptyDeck,Act4Ready,Unfinished }
pub struct M9Report{pub trace:Vec<Step>,pub final_decision:PublicDecision,pub final_map:Option<PublicMap>,pub combats:Vec<(u8,CombatRecord)>,pub events:Vec<EventId>,pub completion:M9Completion}
pub fn start(seed:u64,history:M9BossHistory)->Result<Run,String>{
    let mut root=[0;32];root[..8].copy_from_slice(&seed.to_le_bytes());
    let mut profile=CreationProfile::experienced();profile.neow_eligible=false;
    profile.seen_bosses[0]=vec!["Hexaghost","SlimeBoss"];profile.seen_bosses[1]=vec!["Champ","Collector"];profile.seen_bosses[2]=history.seen();
    Run::new_room_scenario(RunConfig{character:Character::Ironclad,ascension:0},profile,RunRng::new(root,Partition::Test,0),RoomScenario::M9IroncladAct3).map_err(|e|format!("{e:?}"))
}
pub fn complete(seed:u64,history:M9BossHistory,budget:usize)->Result<M9Report,String>{
    rollout(start(seed,history)?,budget,false)
}
pub fn death(seed:u64,budget:usize)->Result<M9Report,String>{rollout(start(seed,M9BossHistory::AwakenedUnseen)?,budget,true)}
pub fn classify(d:&PublicDecision)->M9Completion{match d.terminal{
    Some(RunTerminal::Act3NoKeys)=>M9Completion::Act3NoKeys,Some(RunTerminal::Death)=>M9Completion::Death,
    Some(RunTerminal::EmptyDeck)=>M9Completion::EmptyDeck,_=>if d.room.as_ref().is_some_and(|r|r.kind==RoomViewKind::Act4Ready){M9Completion::Act4Ready}else{M9Completion::Unfinished},
}}
fn rollout(mut run:Run,budget:usize,die:bool)->Result<M9Report,String>{
    if budget>8192{return Err("budget must be in 0..8192".into())}
    let mut trace=vec![];let mut combats=vec![];let mut events=vec![];let mut act1_elite=false;let mut act2_event=false;
    for _ in 0..budget{
        let d=run.decision().map_err(|e|format!("{e:?}"))?;if classify(&d)!=M9Completion::Unfinished{break}
        let action=if let Some(c)=&d.combat{
            let legal=d.candidates.iter().filter_map(|a|match &a.action{GameAction::Combat(a)=>Some(a.clone()),_=>None}).collect::<Vec<_>>();
            GameAction::Combat(if die{CombatAction::EndTurn}else{previous::combat_action(c,&legal)})
        }else if d.selection.is_some(){
            let upgrade=matches!(d.room.as_ref().unwrap().kind,RoomViewKind::Campfire|RoomViewKind::Event(EventId::UpgradeShrine));
            let priorities=if upgrade{vec![CardId::Bludgeon,CardId::Bash,CardId::Disarm,CardId::Clothesline,CardId::Anger,CardId::Cleave,CardId::ShrugItOff,CardId::IronWave]}else{vec![CardId::Strike,CardId::Defend]};
            priorities.into_iter().find_map(|id|d.candidates.iter().find(|c|c.object.as_ref().is_some_and(|o|o.id==id))).or_else(||d.candidates.iter().find(|c|c.object.is_some())).ok_or("missing public selection")?.action.clone()
        }else{
            let room=d.room.as_ref().ok_or("missing room")?;
            let offered=|kind:&RoomChoiceKind|room.choices.iter().find(|c|c.status==RoomChoiceStatus::Offered&&&c.kind==kind).map(|c|GameAction::Choose(c.id));
            match room.kind{
                RoomViewKind::Neow=>offered(&RoomChoiceKind::NeowMaxHp{amount:8}),
                RoomViewKind::Map=>{
                    let map=run.public_map().ok_or("missing public map")?;
                    let ordinary=combats.iter().filter(|(act,c):&&(u8,CombatRecord)|*act==2&&matches!(c.encounter,EncounterId::SphericGuardian|EncounterId::ThreeByrds|EncounterId::CenturionAndHealer|EncounterId::Snecko|EncounterId::ChosenAndByrds|EncounterId::ThreeCultists)).count().min(2) as u8;
                    let elite=combats.iter().any(|(act,c)|*act==2&&matches!(c.encounter,EncounterId::GremlinLeader|EncounterId::Slavers|EncounterId::BookOfStabbing));let mut memo=BTreeMap::new();
                    d.candidates.iter().filter_map(|c|match c.action{GameAction::Move(id)=>path_score(&map,id,ordinary,if d.act==1{act1_elite}else{elite},act2_event,&mut memo).map(|score|(score,id,c.action.clone())),_=>None}).max_by_key(|(score,id,_)|(*score,std::cmp::Reverse(*id))).map(|(_,id,a)|{if d.act==1&&map.nodes.iter().any(|n|n.id==id&&n.kind==MapNodeKind::Elite){act1_elite=true;}a})
                },
                RoomViewKind::Rewards=>room.choices.iter().find(|c|c.status==RoomChoiceStatus::Offered&&matches!(c.kind,RoomChoiceKind::Gold(_)|RoomChoiceKind::Relic(_))).map(|c|GameAction::Choose(c.id)).or_else(||offered(&RoomChoiceKind::CardReward)).or_else(||room.choices.iter().find(|c|c.status==RoomChoiceStatus::Offered&&matches!(c.kind,RoomChoiceKind::Potion(_))).map(|c|GameAction::Choose(c.id))).or_else(||offered(&RoomChoiceKind::Proceed)),
                RoomViewKind::CardReward=>[CardId::Bludgeon,CardId::Impervious,CardId::Offering,CardId::Inflame,CardId::Disarm,CardId::Clothesline,CardId::Cleave,CardId::Anger,CardId::ShrugItOff,CardId::PommelStrike,CardId::TwinStrike,CardId::Thunderclap,CardId::IronWave].into_iter().filter(|id|d.objects.iter().filter(|c|c.id==*id).count()<match id{CardId::ShrugItOff=>3,CardId::Anger=>2,_=>1}).find_map(|id|offered(&RoomChoiceKind::Card{id,upgrades:0})).or_else(||offered(&RoomChoiceKind::Skip)),
                RoomViewKind::Shop=>[CardId::IronWave,CardId::Clothesline].into_iter().filter(|id|d.objects.iter().filter(|c|c.id==*id).count()<2).find_map(|id|offered(&RoomChoiceKind::Card{id,upgrades:0})).or_else(||room.choices.iter().find(|c|c.status==RoomChoiceStatus::Offered&&matches!(c.kind,RoomChoiceKind::Potion(_))).map(|c|GameAction::Choose(c.id))).or_else(||offered(&RoomChoiceKind::Leave)),
                RoomViewKind::Event(EventId::UpgradeShrine)=>offered(&RoomChoiceKind::ShrineUpgrade).or_else(||offered(&RoomChoiceKind::Leave)),
                RoomViewKind::Event(EventId::Purifier)=>offered(&RoomChoiceKind::ShrineRemove).or_else(||offered(&RoomChoiceKind::Leave)),
                RoomViewKind::Event(EventId::AncientWriting)=>offered(&RoomChoiceKind::AncientSimplicity).or_else(||offered(&RoomChoiceKind::Leave)),
                RoomViewKind::Event(EventId::Joust)=>offered(&RoomChoiceKind::JoustMurderer).or_else(||offered(&RoomChoiceKind::JoustContinue)).or_else(||room.choices.iter().find(|c|c.status==RoomChoiceStatus::Offered&&matches!(c.kind,RoomChoiceKind::JoustResult{..})).map(|c|GameAction::Choose(c.id))).or_else(||offered(&RoomChoiceKind::Leave)),
                RoomViewKind::Event(EventId::Falling)=>room.choices.iter().filter(|c|c.status==RoomChoiceStatus::Offered).filter_map(|c|match c.kind{RoomChoiceKind::FallingLose{instance:Some(instance),..}=>d.objects.iter().find(|card|card.instance==instance).map(|card|((match card.id{CardId::Strike=>100,CardId::Defend=>90,CardId::Anger=>70,CardId::IronWave=>60,CardId::Bludgeon=> -10,_=>0},std::cmp::Reverse(instance)),GameAction::Choose(c.id))),_=>None}).max_by_key(|(score,_)|*score).map(|(_,a)|a).or_else(||offered(&RoomChoiceKind::Leave)),
                RoomViewKind::Event(EventId::TombOfLordRedMask)=>offered(&RoomChoiceKind::TombGold).or_else(||offered(&RoomChoiceKind::TombOfferGold)).or_else(||offered(&RoomChoiceKind::Leave)),
                RoomViewKind::Campfire=>if d.player.hp+15>=d.player.max_hp{offered(&RoomChoiceKind::Smith).or_else(||room.choices.iter().find(|c|c.status==RoomChoiceStatus::Offered&&matches!(c.kind,RoomChoiceKind::Rest{..})).map(|c|GameAction::Choose(c.id)))}else{room.choices.iter().find(|c|c.status==RoomChoiceStatus::Offered&&matches!(c.kind,RoomChoiceKind::Rest{..})).map(|c|GameAction::Choose(c.id))},
                RoomViewKind::Treasure=>offered(&RoomChoiceKind::OpenChest).or_else(||room.choices.iter().find(|c|c.status==RoomChoiceStatus::Offered&&matches!(c.kind,RoomChoiceKind::Relic(_))).map(|c|GameAction::Choose(c.id))).or_else(||offered(&RoomChoiceKind::Proceed)),
                RoomViewKind::BossTreasure=>offered(&RoomChoiceKind::OpenChest),
                RoomViewKind::BossRelicChoice=>offered(&RoomChoiceKind::Relic(if d.act==1{RelicId::SlaversCollar}else{RelicId::BlackBlood})).or_else(||offered(&RoomChoiceKind::Relic(RelicId::EmptyCage))).or_else(||offered(&RoomChoiceKind::Skip)),
                RoomViewKind::BossRelicChild|RoomViewKind::Act3BossComplete=>offered(&RoomChoiceKind::Proceed),
                RoomViewKind::ProceedWithoutRelic=>offered(&RoomChoiceKind::ConfirmSkip),
                RoomViewKind::Act3Ending=>room.choices.iter().find(|c|c.status==RoomChoiceStatus::Offered&&matches!(c.kind,RoomChoiceKind::Act3Continue{..})).map(|c|GameAction::Choose(c.id)),
                _=>None,
            }.ok_or_else(||format!("no public policy at act{} floor{} {:?}, HP{}",d.act,d.floor,room.kind,d.player.hp))?
        };
        if !d.candidates.iter().any(|c|c.action==action){return Err(format!("unoffered action {action:?}"))}
        let t=run.step(&action).map_err(|e|format!("act{} floor{} {action:?}: {e:?}",d.act,d.floor))?;let after=run.decision().map_err(|e|format!("{e:?}"))?;
        if let Some(record)=&after.last_combat{if combats.last().is_none_or(|(_,last):&(u8,CombatRecord)|last.visit!=record.visit){combats.push((after.act,record.clone()));}}
        if let Some(RoomView{kind:RoomViewKind::Event(id),..})=&after.room{events.push(*id);if after.act==2{act2_event=true;}}
        trace.push(Step{action,reward:t.reward,decision:after,map:run.public_map()});
    }
    let final_decision=run.decision().map_err(|e|format!("{e:?}"))?;let completion=classify(&final_decision);
    Ok(M9Report{trace,final_decision,completion,final_map:run.public_map(),combats,events})
}

type RouteMemo=BTreeMap<(NodeId,u8,bool,bool),Option<i32>>;
fn path_score(map:&PublicMap,id:NodeId,ordinary:u8,elite:bool,event:bool,memo:&mut RouteMemo)->Option<i32>{
    let key=(id,ordinary,elite,event);if let Some(score)=memo.get(&key){return *score}let n=map.nodes.iter().find(|n|n.id==id)?;
    let ordinary=if map.act==2&&n.kind==MapNodeKind::Monster{(ordinary+1).min(2)}else{ordinary};
    let elite=elite||n.kind==MapNodeKind::Elite;let event=event||map.act==2&&n.kind==MapNodeKind::Question;
    // Earlier-act coverage matches the accepted caller. Act3 has no artificial
    // elite quota/cutoff: use the same visible-room survival weights to boss50.
    let result=if n.kind==MapNodeKind::Boss{(map.act==3||elite&&(map.act==1||ordinary>=2&&event)).then_some(0)}else if map.act==2&&n.floor==26&&!(ordinary>=2&&elite&&event){None}else{
        let weight=match n.kind{MapNodeKind::Rest=>40,MapNodeKind::Question=>22,MapNodeKind::Shop=>12,MapNodeKind::Treasure=>25,MapNodeKind::Elite=> -25,_=>8};
        n.edges.iter().filter_map(|next|path_score(map,*next,ordinary,elite,event,memo)).max().map(|score|score+weight)
    };memo.insert(key,result);result
}
