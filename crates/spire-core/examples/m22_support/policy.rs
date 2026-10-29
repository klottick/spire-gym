//! Pure M22 choices from the current public menu and accepted public progress.
use std::collections::BTreeSet;
use spire_core::{combat::{CombatAction,SelectionOperation,Stance},content::RelicId,run::{GameAction,PublicDecision,RoomChoiceKind,RoomChoiceStatus,RoomViewKind},run::map::PublicMap};
#[path="../shared/public_routes.rs"]mod routes;
#[derive(Clone,Debug,Default,PartialEq,Eq)]
pub struct PublicCallerProgress{pub ordinary_reached:bool,pub elite_reached:bool,pub purchased_shops:BTreeSet<(u8,u32)>}
pub fn choose(d:&PublicDecision,map:Option<&PublicMap>,progress:&PublicCallerProgress)->Result<GameAction,String>{
    let offered=|a:&GameAction|d.candidates.iter().any(|c|&c.action==a);
    if let Some(action)=d.candidates.iter().filter(|c|matches!(c.action,GameAction::UsePotion{..}|GameAction::Combat(CombatAction::UsePotion{..}))).min_by_key(|c|match c.action{GameAction::UsePotion{slot,..}=>slot as usize,GameAction::Combat(CombatAction::UsePotion{slot,..})=>slot,_=>unreachable!()}).map(|c|c.action.clone()){return Ok(action)}
    if let Some(c)=&d.combat{
        let calm=GameAction::Combat(CombatAction::ChooseStance{stance:Stance::Calm});if c.stance_choice.is_some(){return offered(&calm).then_some(calm).ok_or_else(||"missing public Calm choice".into())}
        if let Some(selection)=&c.selection{
            let confirm=GameAction::Combat(CombatAction::ConfirmSelection);
            if matches!(selection.operation,SelectionOperation::Scry|SelectionOperation::MeditateReturn)&&offered(&confirm){return Ok(confirm)}
            return d.candidates.iter().find(|c|matches!(c.action,GameAction::Combat(CombatAction::SelectCard{..}))).map(|c|c.action.clone()).ok_or_else(||"missing public selection action".into())
        }
        return d.candidates.iter().find(|c|matches!(c.action,GameAction::Combat(CombatAction::PlayCard{..}))).or_else(||d.candidates.iter().find(|c|c.action==GameAction::Combat(CombatAction::EndTurn))).map(|c|c.action.clone()).ok_or_else(||"missing public combat action".into())
    }
    if let Some(c)=d.candidates.iter().find(|c|matches!(c.action,GameAction::Select(_))){return Ok(c.action.clone())}
    let room=d.room.as_ref().ok_or("missing public room")?;
    if room.kind==RoomViewKind::Map{return routes::action(d,map.ok_or("missing public map")?,progress.elite_reached,progress.ordinary_reached)}
    let choice=|wanted:&dyn Fn(&RoomChoiceKind)->bool|room.choices.iter().find(|c|c.status==RoomChoiceStatus::Offered&&c.price.is_none_or(|price|price<=d.player.gold)&&wanted(&c.kind)&&offered(&GameAction::Choose(c.id))).map(|c|GameAction::Choose(c.id));
    let action=match room.kind{
        RoomViewKind::Neow=>choice(&|k|matches!(k,RoomChoiceKind::NeowMaxHp{amount:7})),
        RoomViewKind::CardReward=>choice(&|k|matches!(k,RoomChoiceKind::Card{..})).or_else(||choice(&|k|*k==RoomChoiceKind::Skip)),
        RoomViewKind::Shop=>if progress.purchased_shops.contains(&(d.act,d.floor)){choice(&|k|*k==RoomChoiceKind::Leave)}else{choice(&|k|matches!(k,RoomChoiceKind::Card{..})).or_else(||choice(&|k|*k==RoomChoiceKind::Leave))},
        RoomViewKind::Campfire=>{let kind=if u64::from(d.player.hp)*2<u64::from(d.player.max_hp){choice(&|k|matches!(k,RoomChoiceKind::Rest{..}))}else{choice(&|k|*k==RoomChoiceKind::Smith)};kind.or_else(||choice(&|k|*k==RoomChoiceKind::Proceed))},
        RoomViewKind::BossRelicChoice=>[RelicId::HolyWater,RelicId::VioletLotus,RelicId::SacredBark,RelicId::EmptyCage,RelicId::SlaversCollar].into_iter().find_map(|id|choice(&|k|*k==RoomChoiceKind::Relic(id))).or_else(||choice(&|k|*k==RoomChoiceKind::Skip)),
        RoomViewKind::Treasure=>choice(&|k|matches!(k,RoomChoiceKind::Relic(_))).or_else(||choice(&|k|*k==RoomChoiceKind::Sapphire)).or_else(||choice(&|k|*k==RoomChoiceKind::OpenChest)).or_else(||choice(&|k|*k==RoomChoiceKind::Proceed)),
        RoomViewKind::ProceedWithoutRelic=>choice(&|k|*k==RoomChoiceKind::ConfirmSkip),
        _=>choice(&|k|!matches!(k,RoomChoiceKind::Skip|RoomChoiceKind::Proceed|RoomChoiceKind::Leave|RoomChoiceKind::CancelSkip)).or_else(||choice(&|k|matches!(k,RoomChoiceKind::Proceed|RoomChoiceKind::Leave|RoomChoiceKind::Skip))),
    };action.ok_or_else(||format!("no prescribed public action in {:?}",room.kind))
}
