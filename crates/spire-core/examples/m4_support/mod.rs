//! Public-only caller policy. No resource injection or game-rule implementation.
use spire_core::{combat::{rng::Partition,CombatAction},content::{Character,CardId,PotionId,RelicId,EventId},run::*};
pub fn run(seed:u64)->Run {
    let mut key=[0;32];key[..8].copy_from_slice(&seed.to_le_bytes());let mut profile=CreationProfile::experienced();profile.neow_eligible=false;
    Run::new_room_scenario(RunConfig{character:Character::Ironclad,ascension:0},profile,RunRng::new(key,Partition::Test,0),RoomScenario::M4Act1Prefix).unwrap()
}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Scenario {Route,Purge,Keys,Death}
pub struct Caller {limit:usize,pub trace:Vec<(GameAction,RunTransition,PublicDecision)>}
impl Caller {
    pub fn new(limit:usize)->Self{Self{limit:limit.min(512),trace:vec![]}}
    pub fn accepted(&self)->usize{self.trace.len()}
}
pub fn rollout(run:&mut Run,caller:&mut Caller,scenario:Scenario)->Result<(),String>{
    let mut purge_cancelled=false;
    loop {
        let d=run.decision().map_err(|e|format!("{e:?}"))?;
        if d.terminal.is_some()||d.room.as_ref().is_some_and(|v|v.kind==RoomViewKind::CheckpointComplete){return Ok(())}
        if caller.accepted()>=caller.limit{return Err(format!("unfinished scenario: action budget exhausted after {} accepted actions",caller.accepted()))}
        let action=if d.combat.is_some(){
            if scenario==Scenario::Death{GameAction::Combat(CombatAction::EndTurn)}else{combat_action(&d)?}
        }else if d.selection.is_some(){
            if scenario==Scenario::Purge&&d.room.as_ref().unwrap().kind==RoomViewKind::Shop&&!purge_cancelled {
                purge_cancelled=true;GameAction::Cancel
            }else{
                let upgrade=matches!(d.room.as_ref().unwrap().kind,RoomViewKind::Event(EventId::UpgradeShrine)|RoomViewKind::Campfire);
                d.candidates.iter().find(|c|c.object.as_ref().is_some_and(|o|o.id==if upgrade{CardId::Bash}else{CardId::Strike}))
                    .or_else(||d.candidates.iter().find(|c|c.object.is_some())).ok_or("no eligible public selection")?.action.clone()
            }
        }else{
            let room=d.room.as_ref().ok_or("missing room view")?;
            let offered=|kind:&RoomChoiceKind|room.choices.iter().find(|c|&c.kind==kind&&c.status==RoomChoiceStatus::Offered).map(|c|GameAction::Choose(c.id));
            match room.kind {
                RoomViewKind::Neow=>offered(&if matches!(scenario,Scenario::Keys|Scenario::Purge){RoomChoiceKind::NeowLament}else{RoomChoiceKind::NeowMaxHp{amount:8}}),
                RoomViewKind::Map=>d.candidates.iter().find(|c|matches!(c.action,GameAction::Move(id) if d.floor!=0||id==NodeId(if scenario==Scenario::Death{4}else{2}))).map(|c|c.action.clone()),
                RoomViewKind::Rewards=>room.choices.iter().find(|c|c.status==RoomChoiceStatus::Offered&&matches!(c.kind,RoomChoiceKind::Gold(_)|RoomChoiceKind::CardReward|RoomChoiceKind::Potion(_))).map(|c|GameAction::Choose(c.id)).or_else(||offered(&RoomChoiceKind::Proceed)),
                RoomViewKind::CardReward=>offered(&RoomChoiceKind::Card{id:CardId::ShrugItOff,upgrades:0}),
                RoomViewKind::Shop=>{
                    if d.floor==3&&scenario==Scenario::Purge{offered(&RoomChoiceKind::Purge).or_else(||offered(&RoomChoiceKind::Leave))}
                    else if d.floor==3&&scenario==Scenario::Route{offered(&RoomChoiceKind::Card{id:CardId::IronWave,upgrades:0}).or_else(||offered(&RoomChoiceKind::Potion(PotionId::FirePotion))).or_else(||offered(&RoomChoiceKind::Leave))}
                    else{offered(&RoomChoiceKind::Leave)}
                }
                RoomViewKind::Event(EventId::UpgradeShrine)=>offered(&RoomChoiceKind::ShrineUpgrade).or_else(||offered(&RoomChoiceKind::Leave)),
                RoomViewKind::Event(EventId::Purifier)=>offered(&RoomChoiceKind::ShrineRemove),
                RoomViewKind::Campfire=>if scenario==Scenario::Keys{offered(&RoomChoiceKind::Recall)}else{room.choices.iter().find(|c|matches!(c.kind,RoomChoiceKind::Rest{..})).map(|c|GameAction::Choose(c.id))},
                RoomViewKind::Treasure=>offered(&RoomChoiceKind::OpenChest).or_else(||offered(&if scenario==Scenario::Keys{RoomChoiceKind::Sapphire}else{RoomChoiceKind::Relic(RelicId::Strawberry)})).or_else(||offered(&RoomChoiceKind::Proceed)),
                _=>None,
            }.ok_or_else(||format!("no caller policy for {:?} at floor {}",room.kind,d.floor))?
        };
        if !d.candidates.iter().any(|c|c.action==action){return Err(format!("policy chose unoffered action {action:?}"))}
        let t=run.step(&action).map_err(|e|format!("{action:?}: {e:?}"))?;
        caller.trace.push((action,t,run.decision().map_err(|e|format!("{e:?}"))?));
    }
}
fn combat_action(d:&PublicDecision)->Result<GameAction,String>{
    let c=d.combat.as_ref().unwrap();
    // Public scripted policy only: spend finite Fire, play useful cards, then end.
    if let Some(action)=d.candidates.iter().find(|a|matches!(a.action,GameAction::Combat(CombatAction::UsePotion{slot,..}) if d.player.potions[slot]==Some(PotionId::FirePotion))){return Ok(action.action.clone())}
    for id in [CardId::Bash,CardId::ShrugItOff,CardId::Anger,CardId::IronWave,CardId::Strike,CardId::Defend] {
        if let Some(action)=d.candidates.iter().find(|a|matches!(a.action,GameAction::Combat(CombatAction::PlayCard{instance,..}) if c.hand.iter().any(|card|card.instance.0==instance&&card.id==id))){return Ok(action.action.clone())}
    }Ok(GameAction::Combat(CombatAction::EndTurn))
}
