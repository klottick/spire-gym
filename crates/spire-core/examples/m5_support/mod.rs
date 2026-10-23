//! Public observation/menu-only example caller; no simulator clones or hidden state.
use std::collections::BTreeMap;
use spire_core::{combat::{rng::Partition,CombatAction,TargetId},content::{Character,CardId,PotionId,RelicId,EventId},run::*};
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Scenario {Guardian,Hexaghost,Slime,Death}
pub fn run(seed:u64,scenario:Scenario)->Run{
    let mut key=[0;32];key[..8].copy_from_slice(&seed.to_le_bytes());let mut profile=CreationProfile::experienced();profile.neow_eligible=false;
    profile.seen_bosses[0]=match scenario{Scenario::Guardian|Scenario::Death=>vec!["Hexaghost","SlimeBoss"],Scenario::Hexaghost=>vec!["Guardian","SlimeBoss"],Scenario::Slime=>vec!["Guardian","Hexaghost"]};
    Run::new_room_scenario(RunConfig{character:Character::Ironclad,ascension:0},profile,RunRng::new(key,Partition::Test,0),RoomScenario::M5Act1Captured).unwrap()
}
pub struct Caller {limit:usize,pub elites:usize,goal_met:bool,pub trace:Vec<(GameAction,RunTransition,PublicDecision)>}
impl Caller {pub fn new(limit:usize)->Self{Self{limit:limit.min(4096),elites:0,goal_met:false,trace:vec![]}}pub fn accepted(&self)->usize{self.trace.len()}}

pub fn rollout(run:&mut Run,caller:&mut Caller,scenario:Scenario)->Result<(),String>{
    let keys=scenario==Scenario::Slime;
    loop {
        let d=run.decision().map_err(|e|format!("{e:?}"))?;
        if d.room.as_ref().is_some_and(|r|r.kind==RoomViewKind::Act2Ready){return Ok(())}
        if d.terminal.is_some(){return if scenario==Scenario::Death&&d.terminal==Some(RunTerminal::Death){Ok(())}else{Err(format!("policy lost at floor {}: {:?}, hp {}, actions {}",d.floor,d.terminal,d.player.hp,caller.accepted()))}}
        if caller.accepted()>=caller.limit{return Err(format!("unfinished scenario: action budget exhausted after {} accepted actions",caller.accepted()))}
        let action=if d.combat.is_some(){if scenario==Scenario::Death{GameAction::Combat(CombatAction::EndTurn)}else{combat_action(&d)?}}
        else if d.selection.is_some(){
            let upgrade=matches!(d.room.as_ref().unwrap().kind,RoomViewKind::Campfire|RoomViewKind::Event(EventId::UpgradeShrine));
            let priorities=if upgrade{vec![CardId::Bash,CardId::Disarm,CardId::Clothesline,CardId::Anger,CardId::Cleave,CardId::ShrugItOff,CardId::IronWave]}else{vec![CardId::Strike,CardId::Defend]};
            priorities.into_iter().find_map(|id|d.candidates.iter().find(|c|c.object.as_ref().is_some_and(|o|o.id==id))).or_else(||d.candidates.iter().find(|c|c.object.is_some())).ok_or("no public captured selection")?.action.clone()
        }else{
            let room=d.room.as_ref().ok_or("missing public room")?;
            let offered=|kind:&RoomChoiceKind|room.choices.iter().find(|c|c.status==RoomChoiceStatus::Offered&&&c.kind==kind).map(|c|GameAction::Choose(c.id));
            match room.kind {
                RoomViewKind::Neow=>offered(&if matches!(scenario,Scenario::Hexaghost|Scenario::Slime){RoomChoiceKind::NeowLament}else{RoomChoiceKind::NeowMaxHp{amount:8}}),
                RoomViewKind::Map=>{
                    let map=run.public_map().ok_or("missing public map")?;let mut memo=BTreeMap::new();
                    let selected=d.candidates.iter().filter_map(|c|match c.action{GameAction::Move(id)=>path_score(&map,id,caller.goal_met,keys,&mut memo).map(|score|(score,id,c.action.clone())),_=>None}).max_by_key(|(score,id,_)|(*score,std::cmp::Reverse(*id)));
                    selected.map(|(_,id,action)|{let node=map.nodes.iter().find(|n|n.id==id).unwrap();if node.kind==MapNodeKind::Elite{caller.elites+=1;if !keys||node.burning==Some(true){caller.goal_met=true;}}action})
                },
                RoomViewKind::Rewards=>room.choices.iter().find(|c|c.status==RoomChoiceStatus::Offered&&matches!(c.kind,RoomChoiceKind::Gold(_)|RoomChoiceKind::Relic(_))).map(|c|GameAction::Choose(c.id))
                    .or_else(||if keys{offered(&RoomChoiceKind::Emerald)}else{None}).or_else(||offered(&RoomChoiceKind::CardReward))
                    .or_else(||room.choices.iter().find(|c|c.status==RoomChoiceStatus::Offered&&matches!(c.kind,RoomChoiceKind::Potion(_))).map(|c|GameAction::Choose(c.id))).or_else(||offered(&RoomChoiceKind::Proceed)),
                RoomViewKind::CardReward=>{
                    let count=|id|d.objects.iter().filter(|c|c.id==id).count();
                    let priority=[CardId::Bludgeon,CardId::Impervious,CardId::Offering,CardId::Disarm,CardId::Clothesline,CardId::Cleave,CardId::Anger,CardId::ShrugItOff,CardId::PommelStrike,CardId::TwinStrike,CardId::Thunderclap,CardId::IronWave];
                    priority.into_iter().filter(|id|count(*id)<match id{CardId::ShrugItOff=>3,CardId::Anger=>2,_=>1}).find_map(|id|offered(&RoomChoiceKind::Card{id,upgrades:0})).or_else(||offered(&RoomChoiceKind::Skip))
                },
                RoomViewKind::Shop=>[CardId::IronWave,CardId::Clothesline].into_iter().filter(|id|d.objects.iter().filter(|c|c.id==*id).count()<2).find_map(|id|offered(&RoomChoiceKind::Card{id,upgrades:0}))
                    .or_else(||room.choices.iter().find(|c|c.status==RoomChoiceStatus::Offered&&matches!(c.kind,RoomChoiceKind::Potion(_))).map(|c|GameAction::Choose(c.id))).or_else(||offered(&RoomChoiceKind::Leave)),
                RoomViewKind::Event(EventId::UpgradeShrine)=>offered(&RoomChoiceKind::ShrineUpgrade).or_else(||offered(&RoomChoiceKind::Leave)),
                RoomViewKind::Event(EventId::Purifier)=>offered(&RoomChoiceKind::ShrineRemove).or_else(||offered(&RoomChoiceKind::Leave)),
                RoomViewKind::Campfire=>{
                    if keys&&!d.player.keys[0]{offered(&RoomChoiceKind::Recall)}
                    else if d.player.hp+15>=d.player.max_hp{offered(&RoomChoiceKind::Smith).or_else(||room.choices.iter().find(|c|matches!(c.kind,RoomChoiceKind::Rest{..})).map(|c|GameAction::Choose(c.id)))}
                    else{room.choices.iter().find(|c|c.status==RoomChoiceStatus::Offered&&matches!(c.kind,RoomChoiceKind::Rest{..})).map(|c|GameAction::Choose(c.id))}
                },
                RoomViewKind::Treasure=>offered(&RoomChoiceKind::OpenChest).or_else(||if keys{offered(&RoomChoiceKind::Sapphire)}else{room.choices.iter().find(|c|c.status==RoomChoiceStatus::Offered&&matches!(c.kind,RoomChoiceKind::Relic(_))).map(|c|GameAction::Choose(c.id))}).or_else(||offered(&RoomChoiceKind::Proceed)),
                RoomViewKind::BossTreasure=>offered(&RoomChoiceKind::OpenChest),
                RoomViewKind::BossRelicChoice=>offered(&RoomChoiceKind::Relic(match scenario{Scenario::Guardian|Scenario::Death=>RelicId::BlackBlood,Scenario::Hexaghost=>RelicId::EmptyCage,Scenario::Slime=>RelicId::SlaversCollar})),
                RoomViewKind::BossRelicChild=>offered(&RoomChoiceKind::Proceed),
                RoomViewKind::ProceedWithoutRelic=>offered(&RoomChoiceKind::ConfirmSkip),
                _=>None,
            }.ok_or_else(||format!("no public caller policy at floor {} {:?}",d.floor,room.kind))?
        };
        if !d.candidates.iter().any(|c|c.action==action){return Err(format!("policy chose unoffered action {action:?}"))}
        let t=run.step(&action).map_err(|e|format!("floor {} {action:?}: {e:?}",d.floor))?;
        caller.trace.push((action,t,run.decision().map_err(|e|format!("{e:?}"))?));
    }
}

fn path_score(map:&spire_core::run::map::PublicMap,id:NodeId,met:bool,keys:bool,memo:&mut BTreeMap<(NodeId,bool),Option<i32>>)->Option<i32>{
    if let Some(score)=memo.get(&(id,met)){return *score}
    let n=map.nodes.iter().find(|n|n.id==id)?;let now=met||(n.kind==MapNodeKind::Elite&&(!keys||n.burning==Some(true)));
    let result=if n.kind==MapNodeKind::Boss{now.then_some(0)}else{
        let weight=match n.kind{MapNodeKind::Rest=>40,MapNodeKind::Question=>22,MapNodeKind::Shop=>12,MapNodeKind::Treasure=>25,MapNodeKind::Elite=> -25,_=>8};
        n.edges.iter().filter_map(|next|path_score(map,*next,now,keys,memo)).max().map(|score|score+weight)
    };memo.insert((id,met),result);result
}

fn combat_action(d:&PublicDecision)->Result<GameAction,String>{
    let c=d.combat.as_ref().unwrap();let alive=c.enemies.iter().enumerate().filter(|(_,e)|e.hp>0&&!e.replaced).collect::<Vec<_>>();
    let target=alive.iter().min_by_key(|(_,e)|e.hp+e.block).map(|(i,_)|TargetId(*i as u64+1));
    let power=|enemy:usize,id:&str|c.enemies[enemy].powers.iter().find(|p|p.id.0==id).map_or(0,|p|p.amount);
    let play=|id:CardId|d.candidates.iter().filter(|a|matches!(a.action,GameAction::Combat(CombatAction::PlayCard{instance,..})if c.hand.iter().any(|card|card.instance.0==instance&&card.id==id))).min_by_key(|a|match a.action{GameAction::Combat(CombatAction::PlayCard{target:t,..})=>i32::from(t.is_some()&&t!=target),_=>0}).map(|a|a.action.clone());
    let big=alive.iter().any(|(_,e)|e.hp>20);
    for (slot,potion) in d.player.potions.iter().enumerate(){
        let use_now=match potion{Some(PotionId::StrengthPotion)=>big,Some(PotionId::FirePotion)=>big||alive.len()>1&&alive.iter().any(|(_,e)|e.hp>6),Some(PotionId::WeakPotion)=>alive.iter().any(|(i,e)|e.intent_damage.unwrap_or(0)>=8&&power(*i,"weak")==0),_=>false};
        if use_now{if let Some(action)=d.candidates.iter().filter(|a|matches!(a.action,GameAction::Combat(CombatAction::UsePotion{slot:s,..})if s==slot)).min_by_key(|a|match a.action{GameAction::Combat(CombatAction::UsePotion{target:t,..})=>i32::from(t.is_some()&&t!=target),_=>0}){return Ok(action.action.clone())}}
    }
    let incoming=alive.iter().map(|(_,e)|e.intent_damage.unwrap_or(0)*match e.intent.as_str(){"divider"|"inferno"=>6,"whirlwind"=>4,"tackle"|"twin_slam"=>2,_=>1}).sum::<i32>();
    let sharp=alive.iter().map(|(i,_)|power(*i,"sharp_hide")).sum::<i32>();
    if sharp==0&&alive.len()==1&&alive[0].1.hp+alive[0].1.block<=6 {for id in [CardId::Anger,CardId::Strike,CardId::TwinStrike]{if let Some(a)=play(id){return Ok(a)}}}
    if big {if let Some(a)=play(CardId::Disarm){return Ok(a)}}
    if alive.iter().any(|(i,e)|power(*i,"weak")==0&&e.intent_damage.is_some()){if let Some(a)=play(CardId::Clothesline){return Ok(a)}}
    let need=incoming+sharp*2>c.player.block;
    if need{for id in [CardId::Impervious,CardId::ShrugItOff,CardId::IronWave,CardId::Defend]{if let Some(a)=play(id){return Ok(a)}}}
    if alive.len()>1{if let Some(a)=play(CardId::Cleave){return Ok(a)}}
    if alive.iter().any(|(i,e)|power(*i,"vulnerable")==0&&e.hp>20){if let Some(a)=play(CardId::Bash){return Ok(a)}}
    for id in [CardId::Offering,CardId::PommelStrike,CardId::Anger,CardId::TwinStrike,CardId::IronWave,CardId::Thunderclap,CardId::Clothesline,CardId::Bludgeon,CardId::Strike,CardId::Cleave,CardId::ShrugItOff]{if let Some(a)=play(id){return Ok(a)}}
    Ok(GameAction::Combat(CombatAction::EndTurn))
}
