//! Frozen public-only M21 demonstration inputs and caller.
use spire_core::{combat::CombatAction,content::{CardId,EncounterId,PotionId,RelicId},run::*};
use serde_json::{Value,json};
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Scenario{Starter,Stance,Mantra,Scry,Retain,Creation,Potions,Death}
pub const SCENARIOS:[Scenario;8]=[Scenario::Starter,Scenario::Stance,Scenario::Mantra,Scenario::Scry,Scenario::Retain,Scenario::Creation,Scenario::Potions,Scenario::Death];
impl Scenario{
    pub fn name(self)->&'static str{match self{Self::Starter=>"starter",Self::Stance=>"stance",Self::Mantra=>"mantra",Self::Scry=>"scry",Self::Retain=>"retain",Self::Creation=>"creation",Self::Potions=>"potions",Self::Death=>"death"}}
    pub fn parse(s:&str)->Result<Self,String>{SCENARIOS.into_iter().find(|c|c.name()==s).ok_or_else(||"unsupported scenario".into())}
    pub fn encounter(self)->EncounterId{if self==Self::Starter{EncounterId::JawWorm}else{EncounterId::Cultist}}
    pub fn cards(self)->&'static [CardId]{use CardId::*;match self{
        Self::Starter=>&[],Self::Stance=>&[Vigilance,Eruption,EmptyFist,FlurryOfBlows,MentalFortress,Rushdown,LikeWater],
        Self::Mantra=>&[Prostrate,Prostrate,Prostrate,Worship,Brilliance,Devotion,DefendPurple],
        Self::Scry=>&[ThirdEye,CutThroughFate,JustLucky,Weave,Foresight,Nirvana,StrikePurple,DefendPurple],
        Self::Retain=>&[WindmillStrike,Perseverance,SandsOfTime,Establishment,Meditate,Vigilance,DefendPurple],
        Self::Creation=>&[CarveReality,DeceiveReality,MasterReality,BattleHymn,DefendPurple,DefendPurple],
        Self::Potions=>&[DefendPurple,DefendPurple,DefendPurple,DefendPurple,DefendPurple,Eruption,Vigilance,EmptyFist],Self::Death=>&[DefendPurple]}}
    pub fn start(self)->M21WatcherCombatStart{if self==Self::Starter{return M21WatcherCombatStart::Canonical}
        let relics=match self{Self::Stance=>vec![RelicId::PureWater,RelicId::VioletLotus],Self::Mantra|Self::Retain=>vec![RelicId::PureWater],Self::Scry=>vec![RelicId::GoldenEye],Self::Creation=>vec![RelicId::PureWater,RelicId::SacredBark],Self::Potions=>vec![RelicId::TeardropLocket],_=>vec![]};
        let potions=match self{Self::Creation=>vec![Some(PotionId::BottledMiracle),None,None],Self::Potions=>vec![Some(PotionId::StancePotion),Some(PotionId::Ambrosia),Some(PotionId::BottledMiracle)],_=>vec![None;3]};
        M21WatcherCombatStart::Supplied(AuthoredRunStart{hp:if self==Self::Death{1}else{72},max_hp:72,gold:99,deck:self.cards().iter().enumerate().map(|(i,id)|CardInstance{instance:i as u64+1,id:*id,upgrades:0,misc:0}).collect(),relics,potions,relic_counters:Default::default()})
    }
}
pub fn pick(d:&PublicDecision,scenario:Scenario)->Result<CombatAction,String>{
    let offered=d.candidates.iter().filter_map(|c|if let GameAction::Combat(a)=&c.action{Some(a)}else{None}).collect::<Vec<_>>();
    let missing=||"no prescribed public combat candidate".to_owned();
    if scenario==Scenario::Death{return offered.iter().find(|a|matches!(a,CombatAction::EndTurn)).map(|a|(*a).clone()).ok_or_else(missing)}
    if scenario==Scenario::Starter{return offered.first().map(|a|(*a).clone()).ok_or_else(missing)}
    let combat=d.combat.as_ref().ok_or_else(missing)?;
    if combat.stance_choice.is_some(){return offered.iter().find(|a|matches!(a,CombatAction::ChooseStance{stance:spire_core::combat::Stance::Calm})).map(|a|(*a).clone()).ok_or_else(missing)}
    if let Some(selection)=&combat.selection{
        if selection.operation==spire_core::combat::SelectionOperation::Scry{
            for candidate in &selection.candidates{if candidate.id==CardId::Weave{
                if let Some(action)=offered.iter().find(|a|matches!(a,CombatAction::SelectCard{instance} if *instance==candidate.instance.0)){return Ok((*action).clone())}
            }}
        }else if let Some(action)=offered.iter().find(|a|matches!(a,CombatAction::SelectCard{..})){return Ok((*action).clone())}
        return offered.iter().find(|a|matches!(a,CombatAction::ConfirmSelection)).map(|a|(*a).clone()).ok_or_else(missing)
    }
    if let Some(action)=offered.iter().filter(|a|matches!(a,CombatAction::UsePotion{..})).min_by_key(|a|if let CombatAction::UsePotion{slot,..}=a{*slot}else{unreachable!()}){return Ok((*action).clone())}
    for kind in scenario.cards(){for action in &offered{if let CombatAction::PlayCard{instance,..}=action{
        if combat.hand.iter().any(|c|c.instance.0==*instance&&c.id==*kind){return Ok((*action).clone())}
    }}}
    if let Some(action)=offered.iter().find(|a|matches!(a,CombatAction::PlayCard{..})){return Ok((*action).clone())}
    offered.iter().find(|a|matches!(a,CombatAction::EndTurn)).map(|a|(*a).clone()).ok_or_else(missing)
}
/// Output uses the shared native replay verb/field convention. This serializer
/// reads an already-chosen public action; it does not parse or execute rules.
pub fn action_value(action:&CombatAction)->Value{match action{
    CombatAction::PlayCard{instance,target}=>json!({"verb":"combat.play_card","instance":instance,"target":target.map(|t|t.0)}),
    CombatAction::SelectCard{instance}=>json!({"verb":"combat.select_card","instance":instance}),
    CombatAction::DeselectCard{instance}=>json!({"verb":"combat.deselect_card","instance":instance}),
    CombatAction::ChooseStance{stance}=>json!({"verb":"combat.choose_stance","stance":match stance{spire_core::combat::Stance::Neutral=>"neutral",spire_core::combat::Stance::Wrath=>"wrath",spire_core::combat::Stance::Calm=>"calm",spire_core::combat::Stance::Divinity=>"divinity"}}),
    CombatAction::UsePotion{slot,target}=>json!({"verb":"combat.use_potion","slot":slot,"target":target.map(|t|t.0)}),
    CombatAction::EndTurn=>json!({"verb":"combat.end_turn"}),CombatAction::ConfirmSelection=>json!({"verb":"combat.confirm_selection"}),
    CombatAction::ChooseCardOffer{index}=>json!({"verb":"combat.choose_card_offer","index":index}),CombatAction::SkipCardOffer=>json!({"verb":"combat.skip_card_offer"}),
}}
