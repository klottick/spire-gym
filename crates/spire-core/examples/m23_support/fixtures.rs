//! Plan Task3 fixed played-fixture scripts over the current public menu only.
//! No draw/AI search, no hidden state, no outcome tuning; a miss stays missing.
use spire_core::{combat::{CombatAction,SelectionOperation},content::{CardId,CardType,Catalog},run::{GameAction,PublicDecision}};
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Script{Flight,Prices,PowerScry,Wounds,Stasis}
/// Public facts from earlier accepted decisions of the same fixture.
#[derive(Clone,Debug,Default,PartialEq,Eq)]
pub struct ScriptState{pub confused_seen:bool,pub stasis_seen:bool,pub scry_opening:Option<Vec<u64>>,pub scry_steps:usize}
fn allowed(script:Script,id:CardId,s:&ScriptState)->bool{
    let kind=Catalog::standard().card(id).map(|c|c.card_type);
    match script{
        Script::Flight|Script::Wounds=>true,
        // Establish power/Calm, then EndTurn until an actual Confused draw.
        Script::Prices=>s.confused_seen||matches!(id,CardId::Establishment|CardId::Vigilance),
        // Offered powers/skills only.
        Script::PowerScry=>matches!(kind,Some(CardType::Power|CardType::Skill)),
        // Establish powers (retain the rest) until real Orbs/Stasis appear.
        Script::Stasis=>s.stasis_seen||kind==Some(CardType::Power),
    }
}
pub fn script_action(script:Script,order:&[CardId],d:&PublicDecision,s:&ScriptState)->Result<GameAction,String>{
    let offered=|a:&GameAction|d.candidates.iter().any(|c|&c.action==a);
    let c=d.combat.as_ref().ok_or(SETTLED)?;
    if let Some(choice)=&c.stance_choice{
        return choice.options.iter().map(|stance|GameAction::Combat(CombatAction::ChooseStance{stance:*stance})).find(|a|offered(a)).ok_or_else(||"missing public stance row".into())
    }
    let confirm=GameAction::Combat(CombatAction::ConfirmSelection);
    let first_select=||d.candidates.iter().find(|x|matches!(x.action,GameAction::Combat(CombatAction::SelectCard{..}))).map(|x|x.action.clone());
    if let Some(sel)=&c.selection{
        if sel.operation==SelectionOperation::Scry&&script==Script::PowerScry{
            // Select first two (if available), deselect first, reselect first, confirm; otherwise confirm0.
            let open=s.scry_opening.clone().unwrap_or_else(||sel.candidates.iter().map(|x|x.instance.0).collect());
            let mut plan=Vec::new();if let Some(a)=open.first().copied(){plan.push(CombatAction::SelectCard{instance:a});if let Some(b)=open.get(1).copied(){plan.push(CombatAction::SelectCard{instance:b});}plan.extend([CombatAction::DeselectCard{instance:a},CombatAction::SelectCard{instance:a}]);}
            plan.push(CombatAction::ConfirmSelection);
            if let Some(next)=plan.get(s.scry_steps).cloned().map(GameAction::Combat).filter(|a|offered(a)){return Ok(next)}
            return offered(&confirm).then_some(confirm).ok_or_else(||"scripted Scry step unavailable".into())
        }
        if sel.operation==SelectionOperation::MeditateReturn&&sel.selected.is_empty(){if let Some(a)=first_select(){return Ok(a)}}
        if offered(&confirm){return Ok(confirm)}
        return first_select().ok_or_else(||"missing public selection action".into())
    }
    if script==Script::Flight{
        if let Some(a)=d.candidates.iter().filter(|x|matches!(x.action,GameAction::Combat(CombatAction::UsePotion{..}))).min_by_key(|x|match x.action{GameAction::Combat(CombatAction::UsePotion{slot,..})=>slot,_=>usize::MAX}).map(|x|x.action.clone()){return Ok(a)}
    }
    for id in order{
        if !allowed(script,*id,s){continue}
        if let Some(a)=d.candidates.iter().find(|x|matches!(x.action,GameAction::Combat(CombatAction::PlayCard{instance,..}) if c.hand.iter().any(|h|h.instance.0==instance&&h.id==*id))){return Ok(a.action.clone())}
    }
    let end=GameAction::Combat(CombatAction::EndTurn);offered(&end).then_some(end).ok_or_else(||"missing public EndTurn".into())
}
/// The fixture script's signal at the first non-combat decision after settlement.
pub const SETTLED:&str="fixture combat settled";
/// Runner exit mapping: the settled signal after a played Victory is a successful
/// fixture completion, not an error; Death, budget and other errors are unchanged.
pub fn settled_victory(error:&str,last_combat_victory:bool)->bool{error==SETTLED&&last_combat_victory}
