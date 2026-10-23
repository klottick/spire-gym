use std::collections::BTreeMap;
use crate::{combat::command::CombatAction, content::{Catalog, Character, CardId}};
use super::*;

#[derive(Clone, PartialEq, Eq)] pub struct Run { state:RunState, candidates:Vec<Candidate>, next_choice:u64 }
impl Run {
    pub fn new(config:RunConfig,profile:CreationProfile,rng:RunRng)->Result<Self,RunError>{
        if config.ascension>20{return Err(RunError::InvalidAscension(config.ascension));}
        if !profile.fully_unlocked||!profile.act4_enabled{return Err(RunError::Unsupported("profile must be fully unlocked with Act 4 enabled"));}
        let catalog=Catalog::standard();
        let starter=catalog.starter_deck(config.character).map_err(|_|RunError::Registry("starter deck missing"))?;
        let starter_relic=catalog.starter_relic(config.character).ok_or(RunError::Registry("starter relic missing"))?;
        let(base_hp,base_gold):(u32,u32)=match config.character{Character::Ironclad=>(80,99),Character::Silent=>(70,99),Character::Defect=>(75,99),Character::Watcher=>(72,99)};
        let max_hp=base_hp.saturating_sub(if config.ascension>=14{if config.character==Character::Ironclad{5}else{4}}else{0});
        // A6 uses nearest-integer 90% HP (half up for these positive values); A14
        // max-HP reduction is applied first as established by bounded setup facts.
        let hp=if config.ascension>=6{(max_hp*9+5)/10}else{max_hp};
        let mut deck=Vec::new();let mut id=1;
        for card in starter{deck.push(CardInstance{id:*card,instance:id,upgrades:0});id+=1;}
        if config.ascension>=10{deck.push(CardInstance{id:CardId::AscendersBane,instance:id,upgrades:0});}
        let state=RunState{character:config.character,ascension:config.ascension,hp,max_hp,gold:base_gold,deck,relics:vec![starter_relic],
            potions:vec![None;if config.ascension>=11{2}else{3}],keys:[false;3],act:1,floor:0,room:"Neow",act4_enabled:profile.act4_enabled,
            neow_eligible:profile.neow_eligible,seen_bosses:profile.seen_bosses,stored_note:profile.stored_note,terminal:None,map:None,
            history:RunHistory::default(),relic_counters:BTreeMap::new(),combat:None,selected_neow:None,fault:None,last_selection:None,
            last_resumed_parent:None,rng,pending:None,next_instance:id+1,resume_count:0};
        let mut run=Self{state,candidates:Vec::new(),next_choice:0};run.make_initial_neow()?;Ok(run)
    }
    fn next_choice(&mut self)->ChoiceId{let id=ChoiceId(self.next_choice);self.next_choice+=1;id}
    fn offer(&mut self,label:&'static str,offer:NeowOffer){let id=self.next_choice();self.candidates.push(Candidate{id,action:GameAction::Choose(id),label,neow_offer:Some(offer),object:None});}
    fn make_initial_neow(&mut self)->Result<(),RunError>{
        if self.state.neow_eligible{
            let r0=[NeowReward::ThreeCards,NeowReward::RandomRare,NeowReward::RemoveOne,NeowReward::UpgradeOne,NeowReward::TransformOne,NeowReward::ThreeUncommonColorless];
            let r1=[NeowReward::ThreePotions,NeowReward::CommonRelic,NeowReward::MaxHp,NeowReward::Lament,NeowReward::Gold];
            let cat0=self.state.rng.bounded(RunRngScope::NeowCategory0,r0.len() as u64);
            self.offer("Neow category 0",NeowOffer{category:0,reward:r0[cat0],drawback:None});
            let cat1=self.state.rng.bounded(RunRngScope::NeowCategory1,r1.len() as u64);
            self.offer("Neow category 1",NeowOffer{category:1,reward:r1[cat1],drawback:None});
            let drawback=[NeowDrawback::MaxHpLoss,NeowDrawback::NoGold,NeowDrawback::RandomCurse,NeowDrawback::CurrentHpLoss][self.state.rng.bounded(RunRngScope::NeowDrawback,4)];
            let rewards:&[NeowReward]=match drawback{
                NeowDrawback::MaxHpLoss=>&[NeowReward::RareColorless,NeowReward::RareRelic,NeowReward::ThreeRareCards,NeowReward::TransformTwo,NeowReward::RemoveTwo,NeowReward::Gold250],
                NeowDrawback::NoGold=>&[NeowReward::RareColorless,NeowReward::RareRelic,NeowReward::ThreeRareCards,NeowReward::TransformTwo,NeowReward::RemoveTwo,NeowReward::MaxHpTwice],
                NeowDrawback::RandomCurse=>&[NeowReward::RareColorless,NeowReward::RareRelic,NeowReward::ThreeRareCards,NeowReward::TransformTwo,NeowReward::Gold250,NeowReward::MaxHpTwice],
                NeowDrawback::CurrentHpLoss=>&[NeowReward::RareColorless,NeowReward::RareRelic,NeowReward::ThreeRareCards,NeowReward::TransformTwo,NeowReward::RemoveTwo,NeowReward::Gold250,NeowReward::MaxHpTwice],
            };
            let cat2=self.state.rng.bounded(RunRngScope::NeowCategory2,rewards.len() as u64);
            let reward=rewards[cat2];
            self.offer("Neow category 2",NeowOffer{category:2,reward,drawback:Some(drawback)});
            self.offer("Neow category 3",NeowOffer{category:3,reward:NeowReward::BossSwap,drawback:None});
        }else{
            self.offer("Neow short blessing",NeowOffer{category:0,reward:NeowReward::Lament,drawback:None});
            self.offer("Neow short blessing",NeowOffer{category:0,reward:NeowReward::MaxHp,drawback:None});
        }
        if self.candidates.is_empty(){return Err(RunError::Unsupported("empty Neow offer"));}Ok(())
    }
    pub(crate) fn state(&self)->&RunState{&self.state}
    /// Ordinary public map projection excludes hidden outcomes, encounter IDs,
    /// replay counters and internal scheduling history.
    pub fn public_map(&self)->Option<super::map::PublicMap>{self.state.map.as_ref().map(super::map::public_view)}
    #[cfg(test)] pub(crate) fn state_mut_for_test(&mut self)->&mut RunState{&mut self.state}
    pub fn decision(&self)->Result<PublicDecision,RunError>{
        if let Some(fault)=&self.state.fault{return Err(RunError::Faulted(fault.clone()));}
        let objects=self.state.deck.iter().map(|c|PublicObject{instance:c.instance,id:c.id,upgrades:c.upgrades}).collect();
        let candidates=if self.state.terminal.is_some(){vec![]}else{self.candidates.clone()};
        let offers=candidates.iter().filter_map(|c|c.neow_offer.clone()).collect();
        let selection=self.state.pending.as_ref().map(|p|PublicSelection{required:p.selection.required,selected_count:p.selection.selected.len(),eligible:p.selection.candidates.iter().map(|c|PublicObject{instance:c.card.instance,id:c.card.id,upgrades:c.card.upgrades}).collect()});
        Ok(PublicDecision{act:self.state.act,floor:self.state.floor,player:PublicPlayer{character:self.state.character,hp:self.state.hp,max_hp:self.state.max_hp,gold:self.state.gold,relics:self.state.relics.clone(),potions:self.state.potions.clone(),keys:self.state.keys},objects,candidates,offers,selection,terminal:self.state.terminal})
    }
    pub fn step(&mut self,action:&GameAction)->Result<RunTransition,RunError>{
        if let Some(fault)=&self.state.fault{return Err(RunError::Faulted(fault.clone()));}
        if self.state.terminal.is_some(){return Err(RunError::Terminal);}
        if !self.candidates.iter().any(|c|&c.action==action){return Err(RunError::InvalidAction);}
        match action{
            GameAction::Choose(id)=>{
                let offer=self.candidates.iter().find(|c|c.id==*id).and_then(|c|c.neow_offer.clone()).ok_or(RunError::InvalidAction)?;
                // Selection is the commit point. Task10 owns the consequence; automatic
                // coordinator dispatch faults here without adding a fake continue action.
                self.state.selected_neow=Some(offer);self.candidates.clear();
                Err(self.retain_fault(RunFaultKind::UnsupportedBoundary("Task10 Neow consequence"),"run.neow.consequence"))
            }
            GameAction::Select(id)=>self.select(*id),
            GameAction::Confirm=>self.drive_automatic(),
            _=>Err(self.retain_fault(RunFaultKind::UnsupportedBoundary("unwired run action"),"run.step.unsupported")),
        }
    }
    fn select(&mut self,id:SelectionId)->Result<RunTransition,RunError>{
        let pending=self.state.pending.as_ref().ok_or(RunError::InvalidAction)?;
        let captured=pending.selection.candidates.iter().find(|c|c.id==id).cloned().ok_or(RunError::InvalidAction)?;
        if !self.state.deck.iter().any(|card|card==&captured.card){return Err(RunError::InvalidAction);}
        let pending=self.state.pending.as_mut().ok_or(RunError::InvalidAction)?;
        let index=pending.selection.candidates.iter().position(|c|c.id==id).ok_or(RunError::InvalidAction)?;
        let captured=pending.selection.candidates.remove(index);
        pending.selection.selected.push(captured.card);
        if pending.selection.selected.len()>=pending.selection.required||pending.selection.candidates.is_empty(){
            if pending.selection.candidates.is_empty()&&pending.selection.selected.len()<pending.selection.required{
                match pending.selection.empty_rule{
                    EmptySelectionRule::ResumeParent=>self.resume_parent()?,
                    EmptySelectionRule::Fault=>return Err(self.retain_fault(RunFaultKind::Invariant("selection exhausted before required count"),"run.selection.exhausted")),
                }
            }else{self.resume_parent()?;}
        }else{self.publish_selection_candidates();}
        Ok(RunTransition{terminal:None,committed_events:vec![]})
    }
    fn publish_selection_candidates(&mut self){
        let selection_ids=self.state.pending.as_ref().map(|p|p.selection.candidates.iter().map(|x|x.id).collect::<Vec<_>>()).unwrap_or_default();
        self.candidates.clear();for selection_id in selection_ids{
            let id=self.next_choice();
            let captured=self.state.pending.as_ref().and_then(|p|p.selection.candidates.iter().find(|c|c.id==selection_id)).unwrap();
            let card=&captured.card;
            self.candidates.push(Candidate{id,action:GameAction::Select(selection_id),label:"Select captured instance",neow_offer:None,object:Some(PublicObject{instance:card.instance,id:card.id,upgrades:card.upgrades})});
        }
    }
    fn resume_parent(&mut self)->Result<(),RunError>{
        let Some(mut pending)=self.state.pending.take()else{return Err(RunError::InvalidAction)};
        if pending.resumed{return Err(RunError::InvalidAction)}
        pending.resumed=true;
        self.state.last_selection=Some(pending.selection.selected);
        let (parent_id,target)=match pending.parent{
            ContinuationTarget::ResumeDecision(parent)=>(parent.id,Some(parent)),
            ContinuationTarget::NeowConsequence{parent,offer}=>(parent.id,{self.state.selected_neow=Some(offer);None}),
        };
        self.state.last_resumed_parent=Some(parent_id);
        self.state.resume_count=self.state.resume_count.saturating_add(1);
        if let Some(parent)=target{self.candidates=vec![parent];Ok(())}
        else{self.candidates.clear();Err(self.retain_fault(RunFaultKind::UnsupportedBoundary("Task10 Neow consequence"),"run.neow.consequence"))}
    }
    fn drive_automatic(&mut self)->Result<RunTransition,RunError>{
        let Some(pending)=&self.state.pending else{return Err(RunError::InvalidAction)};
        if !pending.selection.candidates.is_empty()&&pending.selection.selected.len()<pending.selection.required{return Err(RunError::InvalidAction)}
        if pending.selection.selected.len()<pending.selection.required{
            match pending.selection.empty_rule{
                EmptySelectionRule::ResumeParent=>self.resume_parent()?,
                EmptySelectionRule::Fault=>return Err(self.retain_fault(RunFaultKind::Invariant("required selection has no eligible objects"),"run.selection.empty")),
            }
        }else{
            self.resume_parent()?;
        }
        Ok(RunTransition{terminal:None,committed_events:vec![]})
    }
    fn retain_fault(&mut self,kind:RunFaultKind,checkpoint:&'static str)->RunError{
        if self.state.fault.is_none(){self.state.fault=Some(RunFault{kind,checkpoint,committed_events:self.state.history.events.clone()});}
        RunError::Faulted(self.state.fault.clone().unwrap())
    }
    pub(crate) fn begin_selection(&mut self,parent:ContinuationTarget,candidates:Vec<SelectionCandidate>,required:usize,empty_rule:EmptySelectionRule)->Result<(),RunError>{
        if let Some(fault)=&self.state.fault{return Err(RunError::Faulted(fault.clone()));}
        if self.state.terminal.is_some(){return Err(RunError::Terminal);}
        if self.state.pending.is_some(){return Err(RunError::InvalidAction)}
        if candidates.iter().any(|c|!self.state.deck.iter().any(|card|card==&c.card)){return Err(RunError::InvalidAction)}
        self.state.pending=Some(PendingContinuation{parent,selection:PendingSelection{candidates,required,selected:vec![],empty_rule},resumed:false});
        if self.state.pending.as_ref().is_some_and(|p|p.selection.candidates.is_empty()||p.selection.required==0){self.drive_automatic().map(|_|())}
        else{self.publish_selection_candidates();Ok(())}
    }
    pub(crate) fn commit_event(&mut self,event:CommittedEvent)->Result<(),RunError>{
        if let Some(fault)=&self.state.fault{return Err(RunError::Faulted(fault.clone()));}
        if self.state.terminal.is_some(){return Err(RunError::Terminal);}
        self.state.history.events.push(event);Ok(())
    }
    pub(crate) fn commit_terminal(&mut self,terminal:RunTerminal)->Result<RunTransition,RunError>{
        if let Some(fault)=&self.state.fault{return Err(RunError::Faulted(fault.clone()));}
        if self.state.terminal.is_some(){return Err(RunError::Terminal);}
        self.state.terminal=Some(terminal);self.candidates.clear();
        let event=CommittedEvent::Ended(terminal);self.state.history.events.push(event.clone());
        Ok(RunTransition{terminal:Some(terminal),committed_events:vec![event]})
    }
    pub fn remove_card(&mut self,id:u64)->Result<CardInstance,RunError>{
        if let Some(fault)=&self.state.fault{return Err(RunError::Faulted(fault.clone()));}
        if self.state.terminal.is_some(){return Err(RunError::Terminal);}
        let at=self.state.deck.iter().position(|c|c.instance==id).ok_or(RunError::InvalidAction)?;Ok(self.state.deck.remove(at))
    }
    pub fn choose_combat(&mut self,_action:CombatAction)->Result<RunTransition,RunError>{
        if let Some(fault)=&self.state.fault{return Err(RunError::Faulted(fault.clone()));}
        if self.state.terminal.is_some(){return Err(RunError::Terminal);}
        Err(self.retain_fault(RunFaultKind::UnsupportedBoundary("Task3 combat bridge"),"run.combat"))
    }

    #[cfg(test)] pub(crate) fn install_pending_selection_for_test(&mut self,parent:Candidate,instances:Vec<u64>,required:usize,empty_rule:EmptySelectionRule){
        let candidates=instances.into_iter().enumerate().map(|(i,instance)|SelectionCandidate{id:SelectionId(i as u64),card:self.state.deck.iter().find(|c|c.instance==instance).unwrap().clone()}).collect();
        self.begin_selection(ContinuationTarget::ResumeDecision(parent),candidates,required,empty_rule).unwrap();
    }
    #[cfg(test)] pub(crate) fn install_neow_selection_for_test(&mut self,offer:NeowOffer,instances:Vec<u64>){
        let parent=self.candidates[0].clone();
        let candidates=instances.into_iter().enumerate().map(|(i,instance)|SelectionCandidate{id:SelectionId(i as u64),card:self.state.deck.iter().find(|c|c.instance==instance).unwrap().clone()}).collect();
        self.begin_selection(ContinuationTarget::NeowConsequence{parent,offer},candidates,1,EmptySelectionRule::Fault).unwrap();
    }
    #[cfg(test)] pub(crate) fn install_empty_selection_for_test(&mut self,parent:Candidate){
        self.state.pending=Some(PendingContinuation{parent:ContinuationTarget::ResumeDecision(parent),selection:PendingSelection{candidates:vec![],required:1,selected:vec![],empty_rule:EmptySelectionRule::ResumeParent},resumed:false});
        self.candidates=vec![Candidate{id:ChoiceId(u64::MAX),action:GameAction::Confirm,label:"automatic boundary",neow_offer:None,object:None}];
    }
    #[cfg(test)] pub(crate) fn install_empty_fault_selection_for_test(&mut self,parent:Candidate){
        self.state.pending=Some(PendingContinuation{parent:ContinuationTarget::ResumeDecision(parent),selection:PendingSelection{candidates:vec![],required:1,selected:vec![],empty_rule:EmptySelectionRule::Fault},resumed:false});
        self.candidates=vec![Candidate{id:ChoiceId(u64::MAX),action:GameAction::Confirm,label:"automatic boundary",neow_offer:None,object:None}];
    }
    #[cfg(test)] pub(crate) fn record_event_for_test(&mut self,event:CommittedEvent){self.commit_event(event).unwrap();}
    #[cfg(test)] pub(crate) fn install_terminal_for_test(&mut self,terminal:RunTerminal){self.commit_terminal(terminal).unwrap();}
    #[cfg(test)] pub(crate) fn install_fault_for_test(&mut self,fault:RunFault){self.state.fault=Some(fault);self.candidates.clear();}
}
