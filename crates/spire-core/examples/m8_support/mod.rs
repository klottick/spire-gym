//! Public-only normal-start caller. Supplied diagnostics are separate.
#![allow(dead_code)]
#[path="../m7_support/mod.rs"] mod previous;
pub use previous::{Step,Act2Report as Act3Report};
use spire_core::{combat::{CombatAction,CombatPublic,rng::Partition},content::{CardId,CardType,Character,EncounterId,EventId,RelicId},run::*,run::map::PublicMap};
use std::collections::BTreeMap;

pub fn start(seed:u64,seen_act3:Vec<&'static str>)->Result<Run,String>{
    let mut root=[0;32];root[..8].copy_from_slice(&seed.to_le_bytes());let mut profile=CreationProfile::experienced();profile.neow_eligible=false;profile.seen_bosses[0]=vec!["Hexaghost","SlimeBoss"];profile.seen_bosses[1]=vec!["Champ","Collector"];profile.seen_bosses[2]=seen_act3;
    Run::new_room_scenario(RunConfig{character:Character::Ironclad,ascension:0},profile,RunRng::new(root,Partition::Test,0),RoomScenario::M8Act3Prefix).map_err(|e|format!("{e:?}"))
}
pub fn prefix(seed:u64,seen:Vec<&'static str>,budget:usize)->Result<Act3Report,String>{rollout(start(seed,seen)?,budget,false)}
pub fn death(seed:u64,budget:usize)->Result<Act3Report,String>{rollout(start(seed,vec![])?,budget,true)}

pub struct DiagnosticReport{pub encounter:EncounterId,pub actions:usize,pub view:CombatPublic,pub saw_half_dead:bool,pub saw_darkling_return:bool}
pub fn diagnostics(seed:u64,budget:usize)->Result<Vec<DiagnosticReport>,String>{
    use spire_core::combat::{Combat,CardInstanceId,PlayerState,FullPowerEntry,FullPowerId,PowerState,FullIntentId,TargetId,permanent_live_card,enemies::{self,SetupContext}};
    let mut root=[0;32];root[..8].copy_from_slice(&seed.to_le_bytes());let mut rng=RunRng::new(root,Partition::Test,0);let mut reports=vec![];
    for (visit,encounter) in [EncounterId::OrbWalker,EncounterId::ThreeShapes,EncounterId::FourShapes,EncounterId::SphereAndTwoShapes,EncounterId::ThreeDarklings,EncounterId::SpireGrowth,EncounterId::Transient,EncounterId::Maw,EncounterId::WrithingMass,EncounterId::JawWormHorde,EncounterId::GiantHead,EncounterId::Nemesis,EncounterId::Reptomancer,EncounterId::AwakenedOne,EncounterId::TimeEater,EncounterId::DonuAndDeca].into_iter().enumerate(){
        // Explicit diagnostic resources, not normal-start values or tuned enemy
        // stats: the same A0 native bodies run with a roomy supplied player/deck.
        let cards=[CardId::Bludgeon,CardId::Bludgeon,CardId::Bludgeon,CardId::Inflame,CardId::Disarm,CardId::Impervious,CardId::Impervious,CardId::Offering,CardId::Anger,CardId::ShrugItOff,CardId::ShrugItOff,CardId::Cleave].into_iter().enumerate().map(|(i,id)|permanent_live_card(id,CardInstanceId(i as u64+1),1)).collect();
        let mut entry=enemies::setup(encounter,3,0,&mut rng,SetupContext{player:PlayerState{hp:999,max_hp:999,block:0,energy:4,max_energy:4,draw_per_turn:5,powers:vec![]},player_powers:vec![FullPowerEntry{id:FullPowerId("strength"),state:PowerState{amount:20,skip_first:false,monster_source_grace:false}}],character:Character::Ironclad,cards,opening_draw:10,encounter_visit:visit as u64}).map_err(|e|format!("{e:?}"))?;
        if encounter==EncounterId::TimeEater{entry.setup.enemies[0].powers.iter_mut().find(|p|p.id.0=="time_warp").unwrap().state.amount=11;}
        if encounter==EncounterId::WrithingMass{entry.setup.enemies[0].intent=FullIntentId::from_name("writhing_parasite");}
        let mut c=Combat::enter(entry).map_err(|e|format!("{e:?}"))?;let mut actions=0;let mut saw_half_dead=false;let mut saw_darkling_return=false;let mut waiting=0;let mut darkling_waited=false;
        for _ in 0..budget.min(8192){
            let view=c.public_full().map_err(|e|format!("{e:?}"))?;saw_half_dead|=view.enemies.iter().any(|e|e.half_dead);
            if encounter==EncounterId::ThreeDarklings&&view.enemies[0].half_dead&&!darkling_waited{waiting=2;darkling_waited=true;}
            if encounter==EncounterId::ThreeDarklings&&darkling_waited&&waiting==0&&view.enemies[0].hp>0{saw_darkling_return=true;}
            if view.outcome.is_some(){break}let legal=c.legal_actions_full().map_err(|e|format!("{e:?}"))?;
            let action=if waiting>0{waiting-=1;CombatAction::EndTurn}
                else if encounter==EncounterId::ThreeDarklings&&!darkling_waited{legal.iter().find(|a|matches!(a,CombatAction::PlayCard{instance,target:Some(TargetId(1))} if view.hand.iter().any(|card|card.instance.0==*instance&&card.id==CardId::Bludgeon))).cloned().unwrap_or_else(||combat_action(&view,&legal))}
                else if encounter==EncounterId::Transient||view.turn==1&&matches!(encounter,EncounterId::OrbWalker|EncounterId::WrithingMass|EncounterId::Reptomancer){CombatAction::EndTurn}
                else{combat_action(&view,&legal)};
            c.step_full(&action).map_err(|e|format!("supplied {encounter:?} {action:?}: {e:?}"))?;actions+=1;
        }
        let view=c.public_full().map_err(|e|format!("{e:?}"))?;if view.outcome.is_none(){return Err(format!("unfinished supplied diagnostic {encounter:?}, actions{actions}"))}
        reports.push(DiagnosticReport{encounter,actions,view,saw_half_dead,saw_darkling_return});
    }Ok(reports)
}

pub fn combat_action(c:&CombatPublic,legal:&[CombatAction])->CombatAction{
    let shared=previous::combat_action(c,legal);
    // Preserve inherited potion/selection/defense priorities. The newly offered
    // Power can precede ordinary attacks or use otherwise unspent end-turn energy.
    let ordinary_attack=matches!(&shared,CombatAction::PlayCard{instance,..} if c.hand.iter().any(|card|card.instance.0==*instance&&spire_core::content::Catalog::standard().card(card.id).is_some_and(|d|d.card_type==CardType::Attack)));
    if ordinary_attack||shared==CombatAction::EndTurn{if let Some(a)=legal.iter().find(|a|matches!(a,CombatAction::PlayCard{instance,..} if c.hand.iter().any(|card|card.instance.0==*instance&&card.id==CardId::Inflame))){return a.clone()}}
    shared
}

fn rollout(mut run:Run,budget:usize,die:bool)->Result<Act3Report,String>{
    let mut trace=vec![];let mut combats=vec![];let mut events=vec![];let mut act1_elite=false;let mut act2_event=false;let mut act3_event=false;
    for _ in 0..budget.min(8192){
        let d=run.decision().map_err(|e|format!("{e:?}"))?;if d.terminal.is_some()||d.room.as_ref().is_some_and(|r|r.kind==RoomViewKind::Act3PrefixComplete){break}
        let action=if let Some(c)=&d.combat{let legal=d.candidates.iter().filter_map(|a|match &a.action{GameAction::Combat(a)=>Some(a.clone()),_=>None}).collect::<Vec<_>>();GameAction::Combat(if die{CombatAction::EndTurn}else{combat_action(c,&legal)})}
        else if d.selection.is_some(){
            let upgrade=matches!(d.room.as_ref().unwrap().kind,RoomViewKind::Campfire|RoomViewKind::Event(EventId::UpgradeShrine));let priorities=if upgrade{vec![CardId::Bludgeon,CardId::Bash,CardId::Disarm,CardId::Clothesline,CardId::Anger,CardId::Cleave,CardId::ShrugItOff,CardId::IronWave]}else{vec![CardId::Strike,CardId::Defend]};
            priorities.into_iter().find_map(|id|d.candidates.iter().find(|c|c.object.as_ref().is_some_and(|o|o.id==id))).or_else(||d.candidates.iter().find(|c|c.object.is_some())).ok_or("missing public selection")?.action.clone()
        }else{
            let room=d.room.as_ref().ok_or("missing room")?;let offered=|kind:&RoomChoiceKind|room.choices.iter().find(|c|c.status==RoomChoiceStatus::Offered&&&c.kind==kind).map(|c|GameAction::Choose(c.id));
            match room.kind{
                RoomViewKind::Neow=>offered(&RoomChoiceKind::NeowMaxHp{amount:8}),
                RoomViewKind::Map=>{
                    let map=run.public_map().ok_or("missing map")?;
                    let ordinary=combats.iter().filter(|(act,c):&&(u8,CombatRecord)|*act==d.act&&match d.act{2=>matches!(c.encounter,EncounterId::SphericGuardian|EncounterId::ThreeByrds|EncounterId::CenturionAndHealer|EncounterId::Snecko|EncounterId::ChosenAndByrds|EncounterId::ThreeCultists),3=>matches!(c.encounter,EncounterId::OrbWalker|EncounterId::ThreeShapes|EncounterId::SpireGrowth|EncounterId::Transient|EncounterId::FourShapes|EncounterId::Maw|EncounterId::SphereAndTwoShapes|EncounterId::JawWormHorde|EncounterId::ThreeDarklings|EncounterId::WrithingMass),_=>false}).count().min(2) as u8;
                    let elite=combats.iter().any(|(act,c)|*act==d.act&&matches!(c.encounter,EncounterId::GremlinLeader|EncounterId::Slavers|EncounterId::BookOfStabbing|EncounterId::GiantHead|EncounterId::Nemesis|EncounterId::Reptomancer));
                    let mut memo=BTreeMap::new();d.candidates.iter().filter_map(|c|match c.action{GameAction::Move(id)=>(if d.act==3{path_score(&map,id,ordinary,elite,act3_event,&mut memo)}else{previous::path_score(&map,id,ordinary,if d.act==1{act1_elite}else{elite},act2_event,&mut memo)}).map(|score|(score,id,c.action.clone())),_=>None}).max_by_key(|(score,id,_)|(*score,std::cmp::Reverse(*id))).map(|(_,id,a)|{if d.act==1&&map.nodes.iter().any(|n|n.id==id&&n.kind==MapNodeKind::Elite){act1_elite=true;}a})
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
                RoomViewKind::BossRelicChild=>offered(&RoomChoiceKind::Proceed),RoomViewKind::ProceedWithoutRelic=>offered(&RoomChoiceKind::ConfirmSkip),_=>None,
            }.ok_or_else(||format!("no public policy at act{} floor{} {:?}, HP{}",d.act,d.floor,room.kind,d.player.hp))?
        };
        if !d.candidates.iter().any(|c|c.action==action){return Err(format!("unoffered action {action:?}"))}
        let t=run.step(&action).map_err(|e|format!("act{} floor{} {action:?}: {e:?}",d.act,d.floor))?;let after=run.decision().map_err(|e|format!("{e:?}"))?;
        if let Some(record)=&after.last_combat{if combats.last().is_none_or(|(_,last):&(u8,CombatRecord)|last.visit!=record.visit){combats.push((after.act,record.clone()));}}
        if let Some(RoomView{kind:RoomViewKind::Event(id),..})=&after.room{events.push(*id);if after.act==2{act2_event=true;}if after.act==3{act3_event=true;}}
        trace.push(Step{action,reward:t.reward,decision:after,map:run.public_map()});
    }
    Ok(Act3Report{trace,final_decision:run.decision().map_err(|e|format!("{e:?}"))?,final_map:run.public_map(),combats,events})
}

type RouteMemo=BTreeMap<(NodeId,u8,bool,bool),Option<i32>>;
fn path_score(map:&PublicMap,id:NodeId,ordinary:u8,elite:bool,event:bool,memo:&mut RouteMemo)->Option<i32>{
    let key=(id,ordinary,elite,event);if let Some(score)=memo.get(&key){return *score}let n=map.nodes.iter().find(|n|n.id==id)?;
    let ordinary=if n.kind==MapNodeKind::Monster{(ordinary+1).min(2)}else{ordinary};let elite=elite||n.kind==MapNodeKind::Elite;let event=event||n.kind==MapNodeKind::Question;
    let result=if n.floor==43&&n.kind==MapNodeKind::Treasure{(ordinary>=2&&elite&&event).then_some(0)}else if n.floor>43{None}else{let weight=match n.kind{MapNodeKind::Rest=>40,MapNodeKind::Question=>22,MapNodeKind::Shop=>12,MapNodeKind::Treasure=>25,MapNodeKind::Elite=> -25,_=>8};n.edges.iter().filter_map(|next|path_score(map,*next,ordinary,elite,event,memo)).max().map(|score|score+weight)};memo.insert(key,result);result
}
