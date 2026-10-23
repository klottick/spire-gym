//! Bounded menu/public-map-only caller; shared combat policy is not a game resolver.
#![allow(dead_code)]
#[path="../m6_support/mod.rs"] mod previous;
pub use previous::{Step, PrefixReport as Act2Report};
use spire_core::{combat::{CombatAction,CombatPublic,rng::Partition},content::{CardId,Character,EncounterId,EventId,RelicId},run::*,run::map::PublicMap};
use std::collections::BTreeMap;

pub fn start(seed:u64,seen:Vec<&'static str>)->Result<Run,String>{
    let mut root=[0;32];root[..8].copy_from_slice(&seed.to_le_bytes());
    let mut profile=CreationProfile::experienced();profile.neow_eligible=false;
    profile.seen_bosses[0]=vec!["Hexaghost","SlimeBoss"];profile.seen_bosses[1]=seen;
    Run::new_room_scenario(RunConfig{character:Character::Ironclad,ascension:0},profile,RunRng::new(root,Partition::Test,0),RoomScenario::M7Act2Captured).map_err(|e|format!("{e:?}"))
}
pub fn complete(seed:u64,seen:Vec<&'static str>,budget:usize)->Result<Act2Report,String>{rollout(start(seed,seen)?,budget,false)}
pub fn death(seed:u64,budget:usize)->Result<Act2Report,String>{rollout(start(seed,vec!["Automaton","Collector"])?,budget,true)}

pub fn combat_action(c:&CombatPublic,legal:&[CombatAction])->CombatAction {
    let shared=previous::combat_action(c,legal);
    let alive=c.enemies.iter().filter(|e|e.hp>0&&!e.replaced&&!e.escaped).collect::<Vec<_>>();
    if alive.len()!=1||alive[0].powers.iter().any(|p|p.id.0=="sharp_hide"&&p.amount>0){return shared}
    let card_for=|a:&CombatAction|match a{CombatAction::PlayCard{instance,..}=>c.hand.iter().find(|card|card.instance.0==*instance),_=>None};
    let Some(selected)=card_for(&shared)else{return shared}; // Preserve potion/selection work.
    if selected.id==CardId::Disarm{return shared}
    // Complete free attacks before paying to draw: the real Anger copy is then
    // in discard if that draw reaches a reshuffle. No hidden pile inspection.
    if let Some(free)=legal.iter().find(|a|card_for(a).is_some_and(|card|card.id==CardId::Anger&&card.effective_cost==Some(0))){return free.clone()}
    let Some(big)=legal.iter().find(|a|card_for(a).is_some_and(|card|card.id==CardId::Bludgeon))else{return shared};
    let Some(cost)=card_for(big).and_then(|card|card.effective_cost)else{return shared};
    if matches!(selected.id,CardId::Disarm|CardId::Offering|CardId::Impervious){return shared}
    // A public priority heuristic, not a second damage/turn resolver. Keep lethal
    // survival work; otherwise reserve one spare-energy defense then the big hit.
    let hits=match alive[0].intent.as_str(){"champ_execute"|"automaton_flail"|"sphere_slam"|"twin_slam"|"poke"=>2,"book_multi"=>c.turn as i32+1,"byrd_peck"=>5,_=>1};
    let danger=alive[0].intent_damage.unwrap_or(0)*hits-c.player.block>=c.player.hp;
    if danger&&(c.player.energy<=cost||matches!(selected.id,CardId::Defend|CardId::ShrugItOff|CardId::IronWave)){return shared}
    if c.player.block==0&&c.player.energy>cost {
        for id in [CardId::ShrugItOff,CardId::Defend,CardId::IronWave] {
            if let Some(defend)=legal.iter().find(|a|card_for(a).is_some_and(|card|card.id==id&&card.effective_cost.is_some_and(|n|n<=c.player.energy-cost))){return defend.clone()}
        }
    }
    big.clone()
}

fn rollout(mut run:Run,budget:usize,die:bool)->Result<Act2Report,String>{
    let mut trace=vec![];let mut combats=vec![];let mut events=vec![];let mut act1_elite=false;let mut act2_event=false;
    for _ in 0..budget.min(4096){
        let d=run.decision().map_err(|e|format!("{e:?}"))?;
        if d.terminal.is_some()||d.room.as_ref().is_some_and(|r|r.kind==RoomViewKind::Act3Ready){break}
        let action=if let Some(c)=&d.combat{
            let legal=d.candidates.iter().filter_map(|a|match &a.action{GameAction::Combat(a)=>Some(a.clone()),_=>None}).collect::<Vec<_>>();
            GameAction::Combat(if die{CombatAction::EndTurn}else{combat_action(c,&legal)})
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
                    let map=run.public_map().ok_or("missing map")?;
                    let ordinary=combats.iter().filter(|(act,c):&&(u8,CombatRecord)|*act==2&&matches!(c.encounter,EncounterId::SphericGuardian|EncounterId::ThreeByrds|EncounterId::CenturionAndHealer|EncounterId::Snecko|EncounterId::ChosenAndByrds|EncounterId::ThreeCultists)).count().min(2) as u8;
                    let elite=combats.iter().any(|(act,c)|*act==2&&matches!(c.encounter,EncounterId::GremlinLeader|EncounterId::Slavers|EncounterId::BookOfStabbing));
                    let mut memo=BTreeMap::new();
                    d.candidates.iter().filter_map(|c|match c.action{GameAction::Move(id)=>path_score(&map,id,ordinary,if d.act==1{act1_elite}else{elite},act2_event,&mut memo).map(|score|(score,id,c.action.clone())),_=>None}).max_by_key(|(score,id,_)|(*score,std::cmp::Reverse(*id))).map(|(_,id,a)|{if d.act==1&&map.nodes.iter().any(|n|n.id==id&&n.kind==MapNodeKind::Elite){act1_elite=true;}a})
                },
                RoomViewKind::Rewards=>room.choices.iter().find(|c|c.status==RoomChoiceStatus::Offered&&matches!(c.kind,RoomChoiceKind::Gold(_)|RoomChoiceKind::Relic(_))).map(|c|GameAction::Choose(c.id)).or_else(||offered(&RoomChoiceKind::CardReward)).or_else(||room.choices.iter().find(|c|c.status==RoomChoiceStatus::Offered&&matches!(c.kind,RoomChoiceKind::Potion(_))).map(|c|GameAction::Choose(c.id))).or_else(||offered(&RoomChoiceKind::Proceed)),
                RoomViewKind::CardReward=>{
                    [CardId::Bludgeon,CardId::Impervious,CardId::Offering,CardId::Disarm,CardId::Clothesline,CardId::Cleave,CardId::Anger,CardId::ShrugItOff,CardId::PommelStrike,CardId::TwinStrike,CardId::Thunderclap,CardId::IronWave].into_iter().filter(|id|d.objects.iter().filter(|c|c.id==*id).count()<match id{CardId::ShrugItOff=>3,CardId::Anger=>2,_=>1}).find_map(|id|offered(&RoomChoiceKind::Card{id,upgrades:0})).or_else(||offered(&RoomChoiceKind::Skip))
                },
                RoomViewKind::Shop=>[CardId::IronWave,CardId::Clothesline].into_iter().filter(|id|d.objects.iter().filter(|c|c.id==*id).count()<2).find_map(|id|offered(&RoomChoiceKind::Card{id,upgrades:0})).or_else(||room.choices.iter().find(|c|c.status==RoomChoiceStatus::Offered&&matches!(c.kind,RoomChoiceKind::Potion(_))).map(|c|GameAction::Choose(c.id))).or_else(||offered(&RoomChoiceKind::Leave)),
                RoomViewKind::Event(EventId::UpgradeShrine)=>offered(&RoomChoiceKind::ShrineUpgrade).or_else(||offered(&RoomChoiceKind::Leave)),
                RoomViewKind::Event(EventId::Purifier)=>offered(&RoomChoiceKind::ShrineRemove).or_else(||offered(&RoomChoiceKind::Leave)),
                RoomViewKind::Event(EventId::AncientWriting)=>offered(&RoomChoiceKind::AncientSimplicity).or_else(||offered(&RoomChoiceKind::Leave)),
                RoomViewKind::Event(EventId::Joust)=>offered(&RoomChoiceKind::JoustMurderer).or_else(||offered(&RoomChoiceKind::JoustContinue)).or_else(||room.choices.iter().find(|c|c.status==RoomChoiceStatus::Offered&&matches!(c.kind,RoomChoiceKind::JoustResult{..})).map(|c|GameAction::Choose(c.id))).or_else(||offered(&RoomChoiceKind::Leave)),
                RoomViewKind::Campfire=>if d.player.hp+15>=d.player.max_hp{offered(&RoomChoiceKind::Smith).or_else(||room.choices.iter().find(|c|c.status==RoomChoiceStatus::Offered&&matches!(c.kind,RoomChoiceKind::Rest{..})).map(|c|GameAction::Choose(c.id)))}else{room.choices.iter().find(|c|c.status==RoomChoiceStatus::Offered&&matches!(c.kind,RoomChoiceKind::Rest{..})).map(|c|GameAction::Choose(c.id))},
                RoomViewKind::Treasure=>offered(&RoomChoiceKind::OpenChest).or_else(||room.choices.iter().find(|c|c.status==RoomChoiceStatus::Offered&&matches!(c.kind,RoomChoiceKind::Relic(_))).map(|c|GameAction::Choose(c.id))).or_else(||offered(&RoomChoiceKind::Proceed)),
                RoomViewKind::BossTreasure=>offered(&RoomChoiceKind::OpenChest),
                RoomViewKind::BossRelicChoice=>offered(&RoomChoiceKind::Relic(if d.act==1{RelicId::SlaversCollar}else{RelicId::BlackBlood})).or_else(||offered(&RoomChoiceKind::Relic(RelicId::EmptyCage))).or_else(||offered(&RoomChoiceKind::Skip)),
                RoomViewKind::BossRelicChild=>offered(&RoomChoiceKind::Proceed),
                RoomViewKind::ProceedWithoutRelic=>offered(&RoomChoiceKind::ConfirmSkip),
                _=>None,
            }.ok_or_else(||format!("no public policy at act{} floor{} {:?}, HP{}",d.act,d.floor,room.kind,d.player.hp))?
        };
        if !d.candidates.iter().any(|c|c.action==action){return Err(format!("unoffered action {action:?}"))}
        let t=run.step(&action).map_err(|e|format!("act{} floor{} {action:?}: {e:?}",d.act,d.floor))?;
        let after=run.decision().map_err(|e|format!("{e:?}"))?;
        if let Some(record)=&after.last_combat{if combats.last().map_or(true,|(_,last):&(u8,CombatRecord)|last.visit!=record.visit){combats.push((after.act,record.clone()));}}
        if let Some(RoomView{kind:RoomViewKind::Event(id),..})=&after.room{events.push(*id);if after.act==2{act2_event=true;}}
        trace.push(Step{action,reward:t.reward,decision:after,map:run.public_map()});
    }
    Ok(Act2Report{trace,final_decision:run.decision().map_err(|e|format!("{e:?}"))?,final_map:run.public_map(),combats,events})
}

type RouteMemo=BTreeMap<(NodeId,u8,bool,bool),Option<i32>>;
fn path_score(map:&PublicMap,id:NodeId,ordinary:u8,elite:bool,event:bool,memo:&mut RouteMemo)->Option<i32>{
    let key=(id,ordinary,elite,event);if let Some(score)=memo.get(&key){return *score}let n=map.nodes.iter().find(|n|n.id==id)?;
    let ordinary=if map.act==2&&n.kind==MapNodeKind::Monster{(ordinary+1).min(2)}else{ordinary};
    let elite=elite||n.kind==MapNodeKind::Elite;let event=event||map.act==2&&n.kind==MapNodeKind::Question;
    let result=if n.kind==MapNodeKind::Boss{(elite&&(map.act==1||ordinary>=2&&event)).then_some(0)}else if map.act==2&&n.floor==26&&!(ordinary>=2&&elite&&event){None}else{
        let weight=match n.kind{MapNodeKind::Rest=>40,MapNodeKind::Question=>22,MapNodeKind::Shop=>12,MapNodeKind::Treasure=>25,MapNodeKind::Elite=> -25,_=>8};
        n.edges.iter().filter_map(|next|path_score(map,*next,ordinary,elite,event,memo)).max().map(|score|score+weight)
    };memo.insert(key,result);result
}
