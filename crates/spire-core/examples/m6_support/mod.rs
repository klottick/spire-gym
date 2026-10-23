#![allow(dead_code)]
use spire_core::{combat::{CombatAction,CombatExit,CombatPublic},content::{EncounterId,EventId},run::{GameAction,PublicDecision,map::PublicMap,Run}};
use std::collections::BTreeMap;
use spire_core::{combat::{Combat,TargetId,PlayerState,CardInstanceId,permanent_live_card,rng::Partition,enemies::{self,SetupContext}},content::{CardId,Character,PotionId,RelicId},run::*};
pub struct Step {pub action:GameAction,pub reward:i32,pub decision:PublicDecision,pub map:Option<PublicMap>}
pub struct PrefixReport {pub trace:Vec<Step>,pub final_decision:PublicDecision,pub final_map:Option<PublicMap>,pub combats:Vec<(u8,spire_core::run::CombatRecord)>,pub events:Vec<EventId>}
pub struct Diagnostic {pub encounter:EncounterId,pub actions:Vec<CombatAction>,pub view:CombatPublic,pub exit:CombatExit}
fn rng(seed:u64)->RunRng{let mut key=[0;32];key[..8].copy_from_slice(&seed.to_le_bytes());RunRng::new(key,Partition::Test,0)}
pub fn start(seed:u64,seen:Vec<&'static str>)->Result<Run,String>{
    let mut p=CreationProfile::experienced();p.neow_eligible=false;p.seen_bosses[0]=vec!["Hexaghost","SlimeBoss"];p.seen_bosses[1]=seen;
    Run::new_room_scenario(RunConfig{character:Character::Ironclad,ascension:0},p,rng(seed),RoomScenario::M6Act2Prefix).map_err(|e|format!("{e:?}"))
}
pub fn prefix(seed:u64,seen:Vec<&'static str>,budget:usize)->Result<PrefixReport,String>{rollout(start(seed,seen)?,budget,false)}
pub fn death(seed:u64,budget:usize)->Result<PrefixReport,String>{rollout(start(seed,vec!["Automaton","Collector"])?,budget,true)}

fn rollout(mut run:Run,budget:usize,die:bool)->Result<PrefixReport,String>{
    let mut trace=vec![];let mut combats=vec![];let mut events=vec![];let mut act1_elite=false;let mut act2_event=false;
    for _ in 0..budget.min(4096){
        let d=run.decision().map_err(|e|format!("{e:?}"))?;if d.terminal.is_some()||d.room.as_ref().is_some_and(|r|r.kind==RoomViewKind::Act2PrefixComplete){break}
        let action=if let Some(c)=&d.combat{
            let legal=d.candidates.iter().filter_map(|a|match &a.action{GameAction::Combat(a)=>Some(a.clone()),_=>None}).collect::<Vec<_>>();GameAction::Combat(if die{CombatAction::EndTurn}else{combat_action(c,&legal)})
        }else if d.selection.is_some(){
            let upgrade=matches!(d.room.as_ref().unwrap().kind,RoomViewKind::Campfire|RoomViewKind::Event(EventId::UpgradeShrine));let priorities=if upgrade{vec![CardId::Bash,CardId::Disarm,CardId::Clothesline,CardId::Anger,CardId::Cleave,CardId::ShrugItOff,CardId::IronWave]}else{vec![CardId::Strike,CardId::Defend]};
            priorities.into_iter().find_map(|id|d.candidates.iter().find(|c|c.object.as_ref().is_some_and(|o|o.id==id))).or_else(||d.candidates.iter().find(|c|c.object.is_some())).ok_or("missing public captured selection")?.action.clone()
        }else{
            let room=d.room.as_ref().ok_or("missing room")?;let offered=|kind:&RoomChoiceKind|room.choices.iter().find(|c|c.status==RoomChoiceStatus::Offered&&&c.kind==kind).map(|c|GameAction::Choose(c.id));
            match room.kind{
                RoomViewKind::Neow=>offered(&RoomChoiceKind::NeowMaxHp{amount:8}),
                RoomViewKind::Map=>{
                    let map=run.public_map().ok_or("missing public map")?;let ordinary=combats.iter().filter(|(act,r):&&(u8,CombatRecord)|*act==2&&matches!(r.encounter,EncounterId::SphericGuardian|EncounterId::ThreeByrds|EncounterId::CenturionAndHealer|EncounterId::Snecko|EncounterId::ChosenAndByrds|EncounterId::ThreeCultists)).count().min(2) as u8;
                    let elite=combats.iter().any(|(act,r)|*act==2&&matches!(r.encounter,EncounterId::GremlinLeader|EncounterId::Slavers|EncounterId::BookOfStabbing));let mut memo=BTreeMap::new();
                    let selected=d.candidates.iter().filter_map(|c|match c.action{GameAction::Move(id)=>path_score(&map,id,ordinary,if d.act==1{act1_elite}else{elite},act2_event,&mut memo).map(|score|(score,id,c.action.clone())),_=>None}).max_by_key(|(score,id,_)|(*score,std::cmp::Reverse(*id)));
                    selected.map(|(_,id,action)|{if d.act==1&&map.nodes.iter().any(|n|n.id==id&&n.kind==MapNodeKind::Elite){act1_elite=true;}action})
                },
                RoomViewKind::Rewards=>room.choices.iter().find(|c|c.status==RoomChoiceStatus::Offered&&matches!(c.kind,RoomChoiceKind::Gold(_)|RoomChoiceKind::Relic(_))).map(|c|GameAction::Choose(c.id)).or_else(||offered(&RoomChoiceKind::CardReward)).or_else(||room.choices.iter().find(|c|c.status==RoomChoiceStatus::Offered&&matches!(c.kind,RoomChoiceKind::Potion(_))).map(|c|GameAction::Choose(c.id))).or_else(||offered(&RoomChoiceKind::Proceed)),
                RoomViewKind::CardReward=>{
                    let priority=[CardId::Bludgeon,CardId::Impervious,CardId::Offering,CardId::Disarm,CardId::Clothesline,CardId::Cleave,CardId::Anger,CardId::ShrugItOff,CardId::PommelStrike,CardId::TwinStrike,CardId::Thunderclap,CardId::IronWave];
                    priority.into_iter().filter(|id|d.objects.iter().filter(|c|c.id==*id).count()<match id{CardId::ShrugItOff=>3,CardId::Anger=>2,_=>1}).find_map(|id|offered(&RoomChoiceKind::Card{id,upgrades:0})).or_else(||offered(&RoomChoiceKind::Skip))
                },
                RoomViewKind::Shop=>[CardId::IronWave,CardId::Clothesline].into_iter().filter(|id|d.objects.iter().filter(|c|c.id==*id).count()<2).find_map(|id|offered(&RoomChoiceKind::Card{id,upgrades:0})).or_else(||room.choices.iter().find(|c|c.status==RoomChoiceStatus::Offered&&matches!(c.kind,RoomChoiceKind::Potion(_))).map(|c|GameAction::Choose(c.id))).or_else(||offered(&RoomChoiceKind::Leave)),
                RoomViewKind::Event(EventId::UpgradeShrine)=>offered(&RoomChoiceKind::ShrineUpgrade).or_else(||offered(&RoomChoiceKind::Leave)),
                RoomViewKind::Event(EventId::Purifier)=>offered(&RoomChoiceKind::ShrineRemove).or_else(||offered(&RoomChoiceKind::Leave)),
                RoomViewKind::Event(EventId::AncientWriting)=>offered(&RoomChoiceKind::AncientSimplicity).or_else(||offered(&RoomChoiceKind::Leave)),
                RoomViewKind::Event(EventId::Joust)=>offered(&RoomChoiceKind::JoustMurderer).or_else(||offered(&RoomChoiceKind::JoustContinue)).or_else(||room.choices.iter().find(|c|c.status==RoomChoiceStatus::Offered&&matches!(c.kind,RoomChoiceKind::JoustResult{..})).map(|c|GameAction::Choose(c.id))).or_else(||offered(&RoomChoiceKind::Leave)),
                RoomViewKind::Campfire=>if d.player.hp+15>=d.player.max_hp{offered(&RoomChoiceKind::Smith).or_else(||room.choices.iter().find(|c|c.status==RoomChoiceStatus::Offered&&matches!(c.kind,RoomChoiceKind::Rest{..})).map(|c|GameAction::Choose(c.id)))}else{room.choices.iter().find(|c|c.status==RoomChoiceStatus::Offered&&matches!(c.kind,RoomChoiceKind::Rest{..})).map(|c|GameAction::Choose(c.id))},
                RoomViewKind::Treasure=>offered(&RoomChoiceKind::OpenChest).or_else(||room.choices.iter().find(|c|c.status==RoomChoiceStatus::Offered&&matches!(c.kind,RoomChoiceKind::Relic(_))).map(|c|GameAction::Choose(c.id))).or_else(||offered(&RoomChoiceKind::Proceed)),
                RoomViewKind::BossTreasure=>offered(&RoomChoiceKind::OpenChest),RoomViewKind::BossRelicChoice=>offered(&RoomChoiceKind::Relic(RelicId::SlaversCollar)),RoomViewKind::BossRelicChild=>offered(&RoomChoiceKind::Proceed),RoomViewKind::ProceedWithoutRelic=>offered(&RoomChoiceKind::ConfirmSkip),_=>None,
            }.ok_or_else(||format!("no public route/policy at act{} floor{} {:?}, hp{}",d.act,d.floor,room.kind,d.player.hp))?
        };
        if !d.candidates.iter().any(|c|c.action==action){return Err(format!("unoffered policy action {action:?}"))}
        let t=run.step(&action).map_err(|e|format!("floor{} {action:?}: {e:?}; prior public combat {:?}",d.floor,d.combat))?;let after=run.decision().map_err(|e|format!("{e:?}"))?;
        if let Some(record)=&after.last_combat{if combats.last().map_or(true,|(_,last):&(u8,CombatRecord)|last.visit!=record.visit){combats.push((after.act,record.clone()));}}
        if let Some(RoomView{kind:RoomViewKind::Event(id),..})=&after.room{events.push(*id);if after.act==2{act2_event=true;}}
        trace.push(Step{action,reward:t.reward,decision:after,map:run.public_map()});
    }
    Ok(PrefixReport{trace,final_decision:run.decision().map_err(|e|format!("{e:?}"))?,final_map:run.public_map(),combats,events})
}

type RouteMemo=BTreeMap<(NodeId,u8,bool,bool),Option<i32>>;
fn path_score(map:&PublicMap,id:NodeId,ordinary:u8,elite:bool,event:bool,memo:&mut RouteMemo)->Option<i32>{
    let key=(id,ordinary,elite,event);if let Some(score)=memo.get(&key){return *score}let n=map.nodes.iter().find(|n|n.id==id)?;
    let ordinary=if map.act==2&&n.kind==MapNodeKind::Monster{(ordinary+1).min(2)}else{ordinary};let elite=elite||n.kind==MapNodeKind::Elite;let event=event||map.act==2&&n.kind==MapNodeKind::Question;
    let result=if map.act==1&&n.kind==MapNodeKind::Boss{elite.then_some(0)}else if map.act==2&&n.floor==26{(ordinary>=2&&elite&&event).then_some(0)}else if map.act==2&&n.floor>26{None}else{
        let weight=match n.kind{MapNodeKind::Rest=>40,MapNodeKind::Question=>22,MapNodeKind::Shop=>12,MapNodeKind::Treasure=>25,MapNodeKind::Elite=> -25,_=>8};n.edges.iter().filter_map(|next|path_score(map,*next,ordinary,elite,event,memo)).max().map(|score|score+weight)
    };memo.insert(key,result);result
}

pub fn combat_action(c:&CombatPublic,legal:&[CombatAction])->CombatAction{
    let alive=c.enemies.iter().enumerate().filter(|(_,e)|e.hp>0&&!e.replaced&&!e.escaped).collect::<Vec<_>>();
    let power=|i:usize,id:&str|c.enemies[i].powers.iter().find(|p|p.id.0==id).map_or(0,|p|p.amount);
    // A heuristic over public per-hit outputs, not a second game resolver. Book's
    // unexposed growing count is conservatively bounded by public turn number.
    let threat=|e:&spire_core::combat::FullEnemyView|e.intent_damage.unwrap_or(0)*match e.intent.as_str(){"divider"|"inferno"=>6,"whirlwind"=>4,"byrd_peck"=>5,"book_multi"=>c.turn as i32+1,"tackle"|"twin_slam"|"poke"|"automaton_flail"|"champ_execute"|"sphere_slam"=>2,"leader_stab"|"centurion_fury"=>3,_=>1};
    let incoming=alive.iter().map(|(_,e)|threat(e)).sum::<i32>();let sharp=alive.iter().map(|(i,_)|power(*i,"sharp_hide")).sum::<i32>();let severe=incoming-c.player.block>=(c.player.hp/3).max(1);
    let leader=alive.iter().find(|(_,e)|e.enemy_id==spire_core::content::EnemyId::GremlinLeader);
    // On dangerous boards, current attackers outrank more nonlethal Leader damage.
    // Otherwise end the summon cycle through the visible Leader itself.
    let target=(if severe&&leader.is_some(){alive.iter().filter(|(_,e)|e.enemy_id!=spire_core::content::EnemyId::GremlinLeader&&threat(e)>0).min_by_key(|(_,e)|e.hp+e.block)}else{None}).or(leader).or_else(||alive.iter().min_by_key(|(_,e)|e.hp+e.block)).map(|(i,_)|TargetId(*i as u64+1));
    let weak_target=alive.iter().filter(|(i,e)|power(*i,"weak")==0&&threat(e)>0).max_by_key(|(_,e)|threat(e)).map(|(i,_)|TargetId(*i as u64+1));
    let play=|id:CardId|{let wanted=if id==CardId::Disarm{alive.iter().max_by_key(|(_,e)|e.max_hp).map(|(i,_)|TargetId(*i as u64+1))}else if id==CardId::Clothesline{weak_target.or(target)}else{target};legal.iter().filter(|a|matches!(a,CombatAction::PlayCard{instance,..}if c.hand.iter().any(|card|card.instance.0==*instance&&card.id==id))).min_by_key(|a|match a{CombatAction::PlayCard{target:t,..}=>i32::from(t.is_some()&&*t!=wanted),_=>0}).cloned()};
    let big=alive.iter().any(|(_,e)|e.hp>20);if let Some(items)=&c.inventory{for (slot,potion) in items.potions.iter().enumerate(){let use_now=match potion{Some(PotionId::StrengthPotion)=>big,Some(PotionId::FirePotion)=>big||alive.len()>1&&alive.iter().any(|(_,e)|e.hp>6),Some(PotionId::WeakPotion)=>incoming>=8&&weak_target.is_some(),_=>false};if use_now{let wanted=if *potion==Some(PotionId::WeakPotion){weak_target}else{target};if let Some(a)=legal.iter().filter(|a|matches!(a,CombatAction::UsePotion{slot:s,..}if *s==slot)).min_by_key(|a|match a{CombatAction::UsePotion{target:t,..}=>i32::from(t.is_some()&&*t!=wanted),_=>0}){return a.clone()}}}}
    if alive.iter().any(|(_,e)|e.enemy_id==spire_core::content::EnemyId::GremlinLeader){
        // Take an already-visible cheap add kill before spending the turn on
        // defense/Leader damage; otherwise a six-HP Fat keeps scaling and applying
        // Weak every round. No future summon/AI state is consulted.
        for id in [CardId::Anger,CardId::Strike,CardId::TwinStrike]{if let Some(a)=legal.iter().find(|a|matches!(a,CombatAction::PlayCard{instance,target:Some(t)}if c.hand.iter().any(|card|card.instance.0==*instance&&card.id==id)&&alive.iter().any(|(i,e)|TargetId(*i as u64+1)==*t&&e.enemy_id!=spire_core::content::EnemyId::GremlinLeader&&e.hp<=6&&e.block==0))){return a.clone()}}
    }
    if sharp==0&&alive.len()==1&&alive[0].1.hp+alive[0].1.block<=6{for id in [CardId::Anger,CardId::Strike,CardId::TwinStrike]{if let Some(a)=play(id){return a}}}
    if big{if let Some(a)=play(CardId::Disarm){return a}}if alive.iter().any(|(i,e)|power(*i,"weak")==0&&e.intent_damage.is_some()){if let Some(a)=play(CardId::Clothesline){return a}}
    if c.hand.len()<8&&c.player.hp>6{if let Some(a)=play(CardId::Offering){return a}}
    if incoming+sharp*2>c.player.block{for id in [CardId::Impervious,CardId::ShrugItOff,CardId::IronWave,CardId::Defend]{if let Some(a)=play(id){return a}}}
    if alive.len()>1{if let Some(a)=play(CardId::Cleave){return a}}if alive.iter().any(|(i,e)|power(*i,"vulnerable")==0&&e.hp>20){if let Some(a)=play(CardId::Bash){return a}}
    for id in [CardId::Offering,CardId::Bludgeon,CardId::PommelStrike,CardId::Anger,CardId::TwinStrike,CardId::IronWave,CardId::Thunderclap,CardId::Clothesline,CardId::Strike,CardId::Cleave,CardId::ShrugItOff]{if let Some(a)=play(id){return a}}CombatAction::EndTurn
}

pub fn encounters(seed:u64,budget:usize)->Result<Vec<Diagnostic>,String>{
    let mut results=vec![];for (visit,encounter) in [EncounterId::GremlinLeader,EncounterId::Slavers,EncounterId::BookOfStabbing,EncounterId::Champ,EncounterId::Collector,EncounterId::BronzeAutomaton].into_iter().enumerate(){
        let ids=[CardId::Bludgeon,CardId::Bludgeon,CardId::Bash,CardId::Clothesline,CardId::Disarm,CardId::Impervious,CardId::Offering,CardId::Anger,CardId::Cleave,CardId::ShrugItOff,CardId::ShrugItOff,CardId::IronWave];let cards=ids.into_iter().enumerate().map(|(i,id)|permanent_live_card(id,CardInstanceId(i as u64+1),1)).collect();
        let entry=enemies::setup(encounter,2,0,&mut rng(seed),SetupContext{player:PlayerState{hp:300,max_hp:300,block:0,energy:3,max_energy:3,draw_per_turn:5,powers:vec![]},player_powers:vec![],character:Character::Ironclad,cards,opening_draw:5,encounter_visit:visit as u64}).map_err(|e|format!("{e:?}"))?;
        let mut c=Combat::enter(entry).map_err(|e|format!("{e:?}"))?;let mut actions=vec![];for _ in 0..budget.min(4096){let view=c.public_full().map_err(|e|format!("{e:?}"))?;if view.outcome.is_some(){break}let legal=c.legal_actions_full().map_err(|e|format!("{e:?}"))?;let action=combat_action(&view,&legal);c.step_full(&action).map_err(|e|format!("{encounter:?} {action:?}: {e:?}"))?;actions.push(action);}
        let view=c.public_full().map_err(|e|format!("{e:?}"))?;if view.outcome.is_none(){return Err(format!("unfinished diagnostic {encounter:?}, budget{}",actions.len()))}let exit=c.exit().map_err(|e|format!("{e:?}"))?;results.push(Diagnostic{encounter,actions,view,exit});
    }Ok(results)
}
