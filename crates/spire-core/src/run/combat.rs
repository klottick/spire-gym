//! Thin persistent run/combat ownership bridge over the shared resolver.
use super::*;
use crate::{combat::{Combat, CombatEntry, CombatEntryFailure, PlayerState, CardInstanceId,
    enemies::{self, SetupContext}}, content::{Character, CardId, EncounterId},
    items::{CombatItemContext, PotionRoomPhase}};
use std::collections::BTreeSet;
use crate::combat::{CombatAction, CombatExit, CombatCardRef, Outcome};

/// All allocation and membership checks precede any permanent Run mutation. The
/// source journal remains readable after a native fault and is never consumed.
fn prepare_gains(state:&RunState)->Result<Option<(Vec<CardInstance>,u64,u64)>,RunError>{
    let active=state.active_combat.as_ref().ok_or(RunError::InvalidAction)?;
    let journal=state.combat.as_ref().ok_or(RunError::InvalidAction)?.permanent_gains();
    if journal.is_empty()&&active.applied_gain_ordinal==0&&active.acquired_instances.is_empty(){return Ok(None)}
    let applied=usize::try_from(active.applied_gain_ordinal).map_err(|_|RunError::InvalidAction)?;
    if applied>journal.len()||active.acquired_instances.len()!=applied||!state.relics.is_empty()||!state.potions.is_empty()||!state.relic_counters.is_empty(){return Err(RunError::InvalidAction)}
    for (i,gain) in journal.iter().enumerate(){if gain.ordinal!=i as u64+1||gain.id!=CardId::Parasite||gain.upgrades!=0{return Err(RunError::InvalidAction)}}
    let mut existing=BTreeSet::new();if state.deck.iter().any(|c|c.instance==0||!existing.insert(c.instance)){return Err(RunError::InvalidAction)}
    let mut mapped=BTreeSet::new();for (gain,id) in journal[..applied].iter().zip(&active.acquired_instances){if !mapped.insert(*id)||!state.deck.iter().any(|c|c.instance==*id&&c.id==gain.id&&c.upgrades==gain.upgrades){return Err(RunError::InvalidAction)}}
    if applied==journal.len(){return Ok(None)}
    let pending=&journal[applied..];if state.deck.len().checked_add(pending.len()).is_none_or(|n|n>999){return Err(RunError::Unsupported("permanent deck capacity exhausted"))}
    let count=u64::try_from(pending.len()).map_err(|_|RunError::InvalidAction)?;
    let next=state.next_instance.checked_add(count).ok_or(RunError::Unsupported("permanent identity exhausted"))?;
    let mut cards=Vec::with_capacity(pending.len());for (i,gain) in pending.iter().enumerate(){let instance=state.next_instance+i as u64;if instance==0||existing.contains(&instance){return Err(RunError::InvalidAction)}cards.push(CardInstance{ misc:0,instance,id:gain.id,upgrades:gain.upgrades});}
    Ok(Some((cards,next,journal.last().unwrap().ordinal)))
}

/// Absolute action-time misc updates; retries never replay a growth delta.
fn prepare_card_mutations(state:&RunState)->Result<Option<(Vec<(u64,i32)>,u64)>,RunError>{
    let active=state.active_combat.as_ref().ok_or(RunError::InvalidAction)?;
    let combat=state.combat.as_ref().ok_or(RunError::InvalidAction)?;
    let journal=combat.permanent_card_mutations();
    let applied=usize::try_from(active.applied_mutation_ordinal).map_err(|_|RunError::InvalidAction)?;
    if applied>journal.len(){return Err(RunError::InvalidAction)}
    if journal.is_empty(){return Ok(None)}
    combat.validate_permanent_mutations_for_run().map_err(|_|RunError::InvalidAction)?;
    let mut deck_ids=BTreeSet::new();if state.deck.iter().any(|c|c.instance==0||!deck_ids.insert(c.instance)){return Err(RunError::InvalidAction)}
    let mut previous=std::collections::BTreeMap::new();let mut next=std::collections::BTreeMap::new();
    for (index,mutation) in journal.iter().enumerate(){
        if mutation.ordinal!=index as u64+1||mutation.id!=CardId::GeneticAlgorithm
            ||!state.deck.iter().any(|card|card.instance==mutation.instance.0&&card.id==mutation.id){return Err(RunError::InvalidAction)}
        if index<applied {previous.insert(mutation.instance.0,mutation.misc);}
        else {next.insert(mutation.instance.0,mutation.misc);}
    }
    if previous.iter().any(|(instance,misc)|!state.deck.iter().any(|card|card.instance==*instance&&card.misc==*misc)){return Err(RunError::InvalidAction)}
    if applied==journal.len(){return Ok(None)}
    Ok(Some((next.into_iter().collect(),journal.last().unwrap().ordinal)))
}

fn prepare_settlement(state:&RunState) -> Result<(CombatExit,CombatRecord,u64,Option<(u8,u64)>),RunError> {
    let active=state.active_combat.as_ref().ok_or(RunError::InvalidAction)?;
    let settled=state.combat_progress.settled.checked_add(1).ok_or(RunError::Unsupported("settlement count exhausted"))?;
    if settled!=state.combat_progress.entered || active.visit!=state.combat_progress.settled
        || !state.relics.is_empty() || !state.potions.is_empty() || !state.relic_counters.is_empty() { return Err(RunError::InvalidAction); }
    let exit=state.combat.as_ref().ok_or(RunError::InvalidAction)?.exit()?;
    if exit.max_hp<=0 || exit.max_hp>999 || exit.hp<0 || exit.hp>exit.max_hp
        || (exit.hp==0)!=(exit.outcome==Outcome::Death) || exit.item_context.is_none() {
        return Err(RunError::InvalidAction);
    }
    let mut seen=BTreeSet::new();
    for card in exit.hand.iter().chain(&exit.draw).chain(&exit.discard).chain(&exit.exhaust).chain(exit.in_play.iter()).chain(&exit.play_frames).chain(&exit.queued_autoplay).chain(&exit.stasis).chain(&exit.played_powers).chain(&exit.inactive_cards) {
        if let CombatCardRef::Permanent(id)=card.origin {
            if card.instance!=id || active.acquired_instances.contains(&id.0) || !seen.insert(id.0) || !state.deck.iter().any(|original|
                original.instance==id.0 && original.id==card.id && original.misc==card.misc && Some(original.upgrades)==card.upgrades.checked_sub(card.local.temporary_upgrades)) {
                return Err(RunError::Unsupported("permanent card exit mismatch"));
            }
        }
    }
    let originals=state.deck.iter().filter(|c|!active.acquired_instances.contains(&c.instance)).map(|c|c.instance).collect::<BTreeSet<_>>();
    if seen!=originals { return Err(RunError::Unsupported("permanent card missing from exit")); }
    let record=CombatRecord {visit:active.visit,encounter:active.encounter,outcome:exit.outcome};
    let local_visits=if super::rooms::uses_act4(state)&&state.act==4{Some((4,state.room_loop.as_ref().and_then(|r|r.act4.as_ref()).ok_or(RunError::InvalidAction)?.combat_visits.checked_add(1).ok_or(RunError::Unsupported("Act4 settled visit exhausted"))?))}else if super::rooms::uses_captured_act3(state)&&state.act==3{Some((3,state.room_loop.as_ref().and_then(|r|r.act3.as_ref()).ok_or(RunError::InvalidAction)?.combat_visits.checked_add(1).ok_or(RunError::Unsupported("Act3 settled visit exhausted"))?))}else if super::rooms::uses_captured_act2(state)&&state.act==2{Some((2,state.room_loop.as_ref().and_then(|r|r.act2.as_ref()).ok_or(RunError::InvalidAction)?.combat_visits.checked_add(1).ok_or(RunError::Unsupported("Act2 settled visit exhausted"))?))}else if super::rooms::is_m20(state)&&state.act==1{Some((1,state.room_loop.as_ref().and_then(|r|r.act1.as_ref()).ok_or(RunError::InvalidAction)?.combat_visits.checked_add(1).ok_or(RunError::Unsupported("M20 Act1 settled visit exhausted"))?))}else{None};
    Ok((exit,record,settled,local_visits))
}

fn prepare_entry(state:&RunState, encounter:EncounterId,origin:CombatOrigin) -> Result<(CombatEntry,RunRng,crate::items::NativeBattleContext),RunError> {
    let m16=state.authored_combat&&state.character==Character::Defect;
    let m16_metadata=if m16 {
        if state.ascension!=0||state.act!=1||origin!=CombatOrigin::Authored{return Err(RunError::Unsupported("M16 authored combat requires A0 Act1 context"))}
        Some(m16_defect_combat_encounter_metadata(encounter).ok_or(RunError::Unsupported("encounter outside frozen M16 Act1 setup prefix"))?)
    }else{None};
    let m5=super::rooms::uses_captured_act1(state)&&state.act==1&&matches!(encounter,EncounterId::Cultist|EncounterId::JawWorm|EncounterId::TwoLouse|EncounterId::SmallSlimes
        |EncounterId::LargeSlime|EncounterId::ThreeLouse|EncounterId::LotsOfSlimes|EncounterId::GremlinNob|EncounterId::Lagavulin|EncounterId::ThreeSentries
        |EncounterId::Guardian|EncounterId::Hexaghost|EncounterId::SlimeBoss);
    let m6=super::rooms::uses_captured_act2(state)&&state.act==2&&matches!(encounter,EncounterId::SphericGuardian|EncounterId::ThreeByrds|EncounterId::CenturionAndHealer|EncounterId::Snecko|EncounterId::ChosenAndByrds|EncounterId::ThreeCultists|EncounterId::GremlinLeader|EncounterId::Slavers|EncounterId::BookOfStabbing|EncounterId::Champ|EncounterId::Collector|EncounterId::BronzeAutomaton);
    let m8=super::rooms::uses_captured_act3(state)&&state.act==3&&matches!(encounter,EncounterId::OrbWalker|EncounterId::ThreeShapes|EncounterId::SpireGrowth|EncounterId::Transient|EncounterId::FourShapes|EncounterId::Maw|EncounterId::SphereAndTwoShapes|EncounterId::JawWormHorde|EncounterId::ThreeDarklings|EncounterId::WrithingMass|EncounterId::GiantHead|EncounterId::Nemesis|EncounterId::Reptomancer);
    let captured_act3_boss=super::rooms::uses_full_act3(state)&&state.act==3&&matches!(encounter,EncounterId::AwakenedOne|EncounterId::TimeEater|EncounterId::DonuAndDeca)&&matches!(origin,CombatOrigin::MapRoom{kind:MapNodeKind::Boss,..}|CombatOrigin::Training{kind:MapNodeKind::Boss});
    let captured_act4=super::rooms::uses_act4(state)&&state.act==4&&match (encounter,origin){
        (EncounterId::ShieldAndSpear,CombatOrigin::MapRoom{node,kind:MapNodeKind::Elite})=>state.map.as_ref().is_some_and(|map|map.current==Some(node)&&map.nodes.iter().any(|n|n.id==node&&n.floor==53&&n.visited&&n.encounter==Some(encounter)&&!n.burning)),
        (EncounterId::CorruptHeart,CombatOrigin::MapRoom{node,kind:MapNodeKind::Boss})=>state.map.as_ref().is_some_and(|map|map.current==Some(node)&&map.nodes.iter().any(|n|n.id==node&&n.floor==54&&n.visited&&n.encounter==Some(encounter))),
        _=>false,
    };
    if !m16&&!matches!(origin,CombatOrigin::Training{..})&&!m5&&!m6&&!m8&&!captured_act3_boss&&!captured_act4&&!matches!(encounter,EncounterId::JawWorm|EncounterId::Cultist) && !(state.room_loop.is_some()&&encounter==EncounterId::TwoLouse) {
        return Err(RunError::Unsupported("encounter outside authored M3 profile"));
    }
    let mut rng=state.rng.clone();
    let mut battle=crate::items::NativeBattleContext{elite_trigger:m16_metadata.is_some_and(|(_,kind)|kind==enemies::EncounterKind::Elite)||matches!(origin,CombatOrigin::MapRoom{kind:MapNodeKind::Elite,..}|CombatOrigin::Training{kind:MapNodeKind::Elite}),boss:m16_metadata.is_some_and(|(_,kind)|kind==enemies::EncounterKind::Boss)||matches!(origin,CombatOrigin::MapRoom{kind:MapNodeKind::Boss,..}|CombatOrigin::Training{kind:MapNodeKind::Boss}),burning:None,m15:super::rooms::is_m15(state),m20:super::rooms::is_m20(state)};
    let mut entry=enemies::setup(encounter,m16_metadata.map_or(state.act,|(act,_)|act),state.ascension,&mut rng,SetupContext {
        player:PlayerState {hp:state.hp as i32,max_hp:state.max_hp as i32,block:0,
            energy:3,max_energy:3,draw_per_turn:5,powers:vec![]},player_powers:vec![],
        character:state.character,cards:state.deck.iter().map(|card|
            crate::combat::permanent_live_card_with_misc(card.id,CardInstanceId(card.instance),card.upgrades,card.misc)).collect(),
        opening_draw:state.deck.len().min(5),encounter_visit:state.combat_progress.entered,
    })?;
    if m5||m6||m8 {
        if let Some(node)=state.map.as_ref().and_then(|map|map.current.and_then(|id|map.nodes.iter().find(|n|n.id==id)))
            .filter(|node|battle.elite_trigger&&node.burning) {
            use enemies::BurningBuff;
            let buff=match rng.bounded_scoped(RunRngScope::BurningEliteBuff,&[state.act as u64,node.id.0,state.combat_progress.entered],4) {
                0=>BurningBuff::Strength,1=>BurningBuff::MaxHp,2=>BurningBuff::Metallicize,_=>BurningBuff::Regeneration,
            };
            enemies::apply_burning_buff(&mut entry,buff)?;
            battle.burning=Some(buff);
        }
    }
    Ok((entry,rng,battle))
}

impl Run {
    pub(super) fn sync_permanent_gains(&mut self)->Result<(),RunError>{
        // Feed commits maxHP at its native Fatal checkpoint. Publish that same
        // resource owner after each attempted action, including an action that
        // later faults, without replaying its already-committed heal.
        if let Some(combat)=self.state.combat.as_ref(){
            let max_hp=combat.max_hp_for_run();
            let Ok(max_hp)=u32::try_from(max_hp) else {
                return Err(self.retain_fault(RunFaultKind::Invariant("combat max HP is outside Run bounds"),"run.combat.max-hp"));
            };
            if max_hp>999 {
                return Err(self.retain_fault(RunFaultKind::Invariant("combat max HP is outside Run bounds"),"run.combat.max-hp"));
            }
            self.state.max_hp=max_hp;
            if let Some(gold)=combat.current_combat_gold(){self.state.gold=gold;}
        }
        let prepared=match prepare_gains(&self.state){Ok(p)=>p,Err(_)=>return Err(self.retain_fault(RunFaultKind::Invariant("permanent gain synchronization preflight failed"),"run.combat.permanent-gains"))};
        let mutations=match prepare_card_mutations(&self.state){Ok(p)=>p,Err(_)=>return Err(self.retain_fault(RunFaultKind::Invariant("permanent mutation synchronization preflight failed"),"run.combat.permanent-mutations"))};
        if let Some((cards,next,ordinal))=prepared{
            let ids=cards.iter().map(|c|c.instance).collect::<Vec<_>>();self.state.deck.extend(cards);self.state.next_instance=next;
            let active=self.state.active_combat.as_mut().unwrap();active.acquired_instances.extend(ids);active.applied_gain_ordinal=ordinal;
        }
        if let Some((values,ordinal))=mutations {
            for (instance,misc) in values {self.state.deck.iter_mut().find(|card|card.instance==instance).expect("preflight permanent owner").misc=misc;}
            self.state.active_combat.as_mut().unwrap().applied_mutation_ordinal=ordinal;
        }Ok(())
    }
    pub(super) fn combat_decision(&self) -> Result<PublicDecision,RunError> {
        let owner=self.state.combat.as_ref().ok_or(RunError::InvalidAction)?;
        let combat=owner.public_full()?;
        let inventory=combat.inventory.as_ref().ok_or(RunError::Unsupported("active combat missing inventory"))?;
        let candidates=owner.legal_actions_full()?.into_iter().enumerate().map(|(i,action)| Candidate {
            id:ChoiceId(i as u64),action:GameAction::Combat(action),label:"Combat action",neow_offer:None,object:None,
        }).collect();
        let player=PublicPlayer {relic_counters:inventory.relic_counters.clone(),character:self.state.character,hp:combat.player.hp as u32,max_hp:combat.player.max_hp as u32,
            gold:self.state.gold,relics:inventory.relics.clone(),potions:inventory.potions.clone(),keys:self.state.keys};
        Ok(PublicDecision {content_profile:self.state.room_loop.as_ref().map(|r|r.scenario),room:self.state.room_loop.as_ref().map(|_|RoomView{kind:RoomViewKind::Combat,choices:vec![]}),reward_total:self.state.reward_ledger.total,act:self.state.act,floor:self.state.floor,player,objects:self.state.deck.iter().map(|c|
            PublicObject { misc:c.misc,instance:c.instance,id:c.id,upgrades:c.upgrades}).collect(),candidates,offers:vec![],selection:None,
            terminal:self.state.terminal,combat:Some(combat),combat_progress:self.state.combat_progress,last_combat:self.state.last_combat.clone()})
    }

    pub fn choose_combat(&mut self, action:CombatAction) -> Result<RunTransition,RunError> {
        let observation_owner=std::ptr::from_ref(self) as usize;
        if let Some(fault)=&self.state.fault { return Err(RunError::Faulted(fault.clone())); }
        if self.state.terminal.is_some() { return Err(RunError::Terminal); }
        let combat=self.state.combat.as_mut().ok_or(RunError::InvalidAction)?;
        if !combat.legal_actions_full()?.contains(&action) { return Err(RunError::InvalidAction); }
        let _observation=crate::combat::observe::bind_run(observation_owner,combat);
        let result=combat.step_full(&action);
        if let Err(error)=&result {
            if let RunError::Faulted(fault)=&error {self.state.fault=Some(fault.clone());}
        }
        if result.is_ok() {crate::combat::observe::boundary(combat);}
        self.sync_permanent_gains()?;result?;
        if self.state.combat.as_ref().unwrap().public_full()?.outcome.is_some() { return self.settle_combat(); }
        Ok(RunTransition {reward:0,terminal:None,committed_events:vec![]})
    }

    pub(super) fn settle_combat(&mut self) -> Result<RunTransition,RunError> {
        if let Some(fault)=&self.state.fault { return Err(RunError::Faulted(fault.clone())); }
        self.sync_permanent_gains()?;
        let (mut exit,record,settled,local_visits)=match prepare_settlement(&self.state) {
            Ok(prepared)=>prepared,
            Err(RunError::Faulted(fault))=>{self.state.fault=Some(fault.clone());return Err(RunError::Faulted(fault));}
            Err(_)=>return Err(self.retain_fault(RunFaultKind::Invariant("combat settlement validation failed"),"run.combat.settlement")),
        };
        let act4_node=match super::rooms::act4::validate_combat_source(self,&record){
            Ok(node)=>node,
            Err(_)=>return Err(self.retain_fault(RunFaultKind::Invariant("Act4 combat source preparation failed"),"run.m10.act4.settlement")),
        };
        let heart_victory=if act4_node.is_some()&&record.encounter==EncounterId::CorruptHeart&&record.outcome==Outcome::Victory{
            match super::rooms::act4::prepare_heart_victory(self,&record){
                Ok(prepared)=>prepared,
                Err(_)=>return Err(self.retain_fault(RunFaultKind::Invariant("Heart victory Proceed preparation failed"),"run.m10.heart.victory")),
            }
        }else{None};
        let mut events=vec![CommittedEvent::CombatFinished(record.clone())];
        if exit.outcome==Outcome::Death {events.push(CommittedEvent::Ended(RunTerminal::Death));}
        if super::reward::preflight(&self.state,&events).is_err(){
            return Err(self.retain_fault(RunFaultKind::Invariant("combat reward ledger preflight failed"),"run.combat.settlement"));
        }
        // Final Act3 suppresses the ordinary reward screen. Prepare its exact
        // source/frame before releasing the real combat inventory and census.
        let ending=match super::rooms::act3_end::prepare(self,&record){
            Ok(prepared)=>prepared,
            Err(_)=>return Err(self.retain_fault(RunFaultKind::Invariant("Act3 ending preparation failed"),super::rooms::act3_end::fault_checkpoint(&self.state))),
        };
        let rewards=match if ending.is_some()||heart_victory.is_some(){Ok(None)}else{super::rooms::rewards::prepare_combat(self,&record)}{
            Ok(prepared)=>prepared,
            Err(error)=>{let kind=if super::rooms::is_m20(&self.state){match error{RunError::Unsupported(message)=>RunFaultKind::UnsupportedBoundary(message),_=>RunFaultKind::Invariant("combat reward preparation failed")}}else{RunFaultKind::Invariant("combat reward preparation failed")};return Err(self.retain_fault(kind,"run.combat.rewards"))},
        };
        let checkpoint=match self.state.combat.as_ref().unwrap().public_full(){
            Ok(view)=>match (view.hp_before_healing,view.hp_after_cleanup){
                (Some(before),Some(after)) if before>=0&&after>=0=>CombatCheckpoint{record:record.clone(),hp_before_healing:before as u32,hp_after_cleanup:after as u32,
                    stolen_gold_rewards:self.state.combat.as_ref().unwrap().stolen_gold_rewards(),
                    all_enemies_escaped:self.state.combat.as_ref().unwrap().all_enemies_escaped(),
                    mugged:self.state.combat.as_ref().unwrap().combat_mugged()},
                _=>return Err(self.retain_fault(RunFaultKind::Invariant("combat cleanup checkpoint missing"),"run.combat.checkpoint")),
            },
            Err(_)=>return Err(self.retain_fault(RunFaultKind::Invariant("combat cleanup checkpoint unreadable"),"run.combat.checkpoint")),
        };
        // All conversions/counter/membership/return checks precede this commit.
        // Export is detached; the real terminal owner remains until commit.
        if exit.item_context.take().unwrap().return_to_run(&mut self.state).is_err(){
            return Err(self.retain_fault(RunFaultKind::Invariant("accepted combat item return failed"),"run.combat.item-return"));
        }
        self.state.hp=exit.hp as u32;
        self.state.max_hp=exit.max_hp as u32;
        self.state.combat_progress.settled=settled;
        if let Some((act,visits))=local_visits{let room=self.state.room_loop.as_mut().unwrap();match act{1=>room.act1.as_mut().unwrap().combat_visits=visits,2=>room.act2.as_mut().unwrap().combat_visits=visits,3=>room.act3.as_mut().unwrap().combat_visits=visits,4=>room.act4.as_mut().unwrap().combat_visits=visits,_=>unreachable!()}}
        self.state.last_combat_checkpoint=Some(checkpoint);
        self.state.last_combat=Some(record.clone());
        self.state.active_combat=None;
        self.state.combat=None;
        self.candidates.clear();
        if let Some(prepared)=rewards{super::rooms::rewards::commit_combat(self,prepared);}
        if let Some(prepared)=ending{super::rooms::act3_end::commit(self,prepared);}
        if let Some(node)=act4_node{super::rooms::act4::commit_elite_victory(self,node,record.clone());}
        if let Some(prepared)=heart_victory{super::rooms::act4::commit_heart_victory(self,prepared);}
        if exit.outcome==Outcome::Death {
            self.state.terminal=Some(RunTerminal::Death);
        }
        let reward=super::reward::commit(&mut self.state,&events)?;
        Ok(RunTransition {reward,terminal:self.state.terminal,committed_events:events})
    }

    pub fn from_authored_combat(config:RunConfig, start:AuthoredRunStart, rng:RunRng) -> Result<Self,RunError> {
        if config.ascension>20 { return Err(RunError::InvalidAscension(config.ascension)); }
        let silent_a0=config.character==Character::Silent&&config.ascension==0;
        let silent_next_instance=if silent_a0 {
            let canonical=Self::new(config,CreationProfile::experienced(),rng.clone())?;
            let state=&canonical.state;
            if start.hp!=state.hp || start.max_hp!=state.max_hp || start.gold!=state.gold
                || start.deck!=state.deck || start.relics!=state.relics || start.potions!=state.potions
                || start.relic_counters!=state.relic_counters {
                return Err(RunError::InvalidAction);
            }
            Some(state.next_instance)
        }else{
            if config.character!=Character::Ironclad || config.ascension!=0 {
                return Err(RunError::Unsupported("authored M3 requires Ironclad A0"));
            }
            None
        };
        if start.hp==0 || start.hp>start.max_hp || start.max_hp>999
            || !(1..=999).contains(&start.deck.len()) || start.potions.len()!=3 {
            return Err(RunError::InvalidAction);
        }
        let mut ids=BTreeSet::new();
        for card in &start.deck {
            if card.instance==0 || !ids.insert(card.instance) || card.upgrades>1 {
                return Err(RunError::InvalidAction);
            }
            if !silent_a0 && !matches!(card.id,CardId::Strike|CardId::Defend|CardId::Bash|CardId::Clothesline
                |CardId::IronWave|CardId::PommelStrike|CardId::ShrugItOff|CardId::Anger
                |CardId::Bloodletting|CardId::BurningPact|CardId::Disarm) {
                return Err(RunError::Unsupported("card outside authored M3 profile"));
            }
        }
        let next_instance=if let Some(next)=silent_next_instance { next } else {
            ids.last().unwrap().checked_add(1).ok_or(RunError::Unsupported("permanent identity exhausted"))?
        };
        let mut relics=BTreeSet::new();
        if start.relics.iter().any(|id| !relics.insert(*id))
            || start.relic_counters.keys().any(|id| !relics.contains(id)) { return Err(RunError::InvalidAction); }
        crate::items::validate_combat_inventory(&CombatItemContext::new(start.relics.clone(),start.potions.clone()).with_counters(start.relic_counters.clone()),config.character)?;
        Ok(Self::from_validated_combat_resources(config,start,rng,next_instance))
    }

    /// Shared resource owner after profile-specific validation. Character-specific
    /// entry contracts choose resources; no character copies this state builder,
    /// encounter startup, item custody, resolver or settlement implementation.
    pub(super) fn from_validated_combat_resources(config:RunConfig,start:AuthoredRunStart,rng:RunRng,next_instance:u64)->Self{
        let profile=CreationProfile::experienced();
        Self {state:RunState {
            room_loop:None,reward_ledger:Default::default(),authored_combat:true,training_encounter:None,last_combat_checkpoint:None,combat_progress:CombatProgress::default(),active_combat:None,last_combat:None,
            character:config.character,ascension:config.ascension,hp:start.hp,max_hp:start.max_hp,gold:start.gold,
            deck:start.deck,relics:start.relics,potions:start.potions,relic_counters:start.relic_counters,
            keys:[false;3],act:1,floor:0,room:"AuthoredCombat",act4_enabled:true,neow_eligible:profile.neow_eligible,
            seen_bosses:profile.seen_bosses,stored_note:profile.stored_note,terminal:None,map:None,
            history:RunHistory::default(),combat:None,selected_neow:None,fault:None,last_selection:None,
            last_resumed_parent:None,rng,pending:None,next_instance,resume_count:0,
        },candidates:vec![],next_choice:0}
    }

    pub fn enter_combat(&mut self, encounter:EncounterId) -> Result<RunTransition,RunError> {
        if !self.state.authored_combat {return Err(RunError::InvalidAction)}
        self.enter_owned(encounter,CombatOrigin::Authored)
    }

    pub(super) fn enter_owned(&mut self, encounter:EncounterId, origin:CombatOrigin) -> Result<RunTransition,RunError> {
        if let Some(fault)=&self.state.fault { return Err(RunError::Faulted(fault.clone())); }
        if self.state.terminal.is_some() { return Err(RunError::Terminal); }
        if self.state.combat.is_some() || self.state.active_combat.is_some()
            || self.state.pending.is_some() { return Err(RunError::InvalidAction); }
        let entered=self.state.combat_progress.entered.checked_add(1).ok_or(RunError::Unsupported("combat visit exhausted"))?;
        let (entry,rng,battle)=prepare_entry(&self.state,encounter,origin)?;
        let context=CombatItemContext::take_from_run(&mut self.state)
            .with_native_room(PotionRoomPhase::Combat,self.state.map.as_ref()).with_battle_context(battle);
        let context=if matches!(origin,CombatOrigin::Training{..}){context.with_training_room()}else{context};
        let context=if super::rooms::uses_watcher_content(&self.state){context.with_m22_profile()}else if self.state.authored_combat&&self.state.character==Character::Watcher&&self.state.training_encounter.is_some(){context.with_startup_profile(crate::items::CombatStartupProfile::M21Watcher)}else{context};
        let mut items=Some(context);
        let _observation_entry=crate::combat::observe::permit_entry(std::ptr::from_ref(self) as usize);
        let (mut combat,fault)=match Combat::enter_with_item_context(entry,&mut items) {
            Ok(combat)=>(combat,None),
            Err(CombatEntryFailure::Rejected(error))=> {
                items.expect("rejected entry retained owner").return_to_run(&mut self.state)?;
                return Err(error);
            }
            Err(CombatEntryFailure::Faulted {combat,fault})=>(*combat,Some(fault)),
        };
        let gold_attached=combat.set_thief_gold_for_entry(self.state.gold).is_ok();
        let visit=self.state.combat_progress.entered;
        self.state.rng=rng;
        self.state.combat=Some(combat);
        self.state.active_combat=Some(ActiveCombat { applied_mutation_ordinal:0,visit,encounter,origin,applied_gain_ordinal:0,acquired_instances:vec![]});
        if matches!(origin,CombatOrigin::MapRoom{kind:MapNodeKind::Boss,..}|CombatOrigin::Training{kind:MapNodeKind::Boss}){
            let name=match encounter{EncounterId::Guardian=>Some("Guardian"),EncounterId::Hexaghost=>Some("Hexaghost"),EncounterId::SlimeBoss=>Some("SlimeBoss"),EncounterId::Champ=>Some("Champ"),EncounterId::BronzeAutomaton=>Some("Automaton"),EncounterId::Collector=>Some("Collector"),EncounterId::AwakenedOne=>Some("AwakenedOne"),EncounterId::TimeEater=>Some("TimeEater"),EncounterId::DonuAndDeca=>Some("DonuAndDeca"),_=>None};
            if let Some(name)=name{let seen=&mut self.state.seen_bosses[(self.state.act-1) as usize];if !seen.contains(&name){seen.push(name);}}
        }
        self.state.combat_progress.entered=entered;
        self.candidates.clear();
        let event=CommittedEvent::CombatEntered {visit,encounter};
        let reward=super::reward::commit(&mut self.state,std::slice::from_ref(&event))?;
        if let Some(fault)=fault { self.state.fault=Some(fault.clone()); return Err(RunError::Faulted(fault)); }
        if !gold_attached{return Err(self.retain_fault(RunFaultKind::Invariant("combat gold custody attachment failed"),"run.combat.gold"));}
        Ok(RunTransition {reward,terminal:None,committed_events:vec![event]})
    }
}

impl Run {
    pub fn from_m16_defect_combat(start:M16DefectCombatStart,rng:RunRng)->Result<Self,RunError>{
        let config=RunConfig{character:Character::Defect,ascension:0};
        let (resources,label)=match start {
            M16DefectCombatStart::Canonical=>{
                let canonical=Self::new(config,CreationProfile::experienced(),rng.clone())?;
                let state=canonical.state;
                (AuthoredRunStart{hp:state.hp,max_hp:state.max_hp,gold:state.gold,deck:state.deck,relics:state.relics,potions:state.potions,relic_counters:state.relic_counters},"M16CanonicalCombat")
            }
            M16DefectCombatStart::Supplied(resources)=>(resources,"M16SuppliedCombat"),
        };
        if resources.hp==0||resources.hp>resources.max_hp||resources.max_hp>999
            ||!(1..=999).contains(&resources.deck.len())||resources.potions.len()!=3{return Err(RunError::InvalidAction)}
        let mut ids=BTreeSet::new();
        for card in &resources.deck {
            if card.instance==0||!ids.insert(card.instance)||card.upgrades>1{return Err(RunError::InvalidAction)}
            if !crate::content::defect::supports_permanent(card.id){return Err(RunError::Unsupported("card outside finite M16 authored profile"))}
            if card.id!=CardId::GeneticAlgorithm&&card.misc!=0{return Err(RunError::InvalidAction)}
        }
        let next=ids.last().unwrap().checked_add(1).ok_or(RunError::Unsupported("permanent identity exhausted"))?;
        let mut relics=BTreeSet::new();
        if resources.relics.iter().any(|id|!relics.insert(*id))||resources.relic_counters.keys().any(|id|!relics.contains(id)){return Err(RunError::InvalidAction)}
        crate::items::validate_combat_inventory(&CombatItemContext::new(resources.relics.clone(),resources.potions.clone()).with_counters(resources.relic_counters.clone()),Character::Defect)?;
        let mut run=Self::from_validated_combat_resources(config,resources,rng,next);run.state.room=label;Ok(run)
    }
}

/// Admission reuses the frozen prefix of the authoritative setup owner.
pub fn m16_defect_combat_encounter_metadata(encounter:EncounterId)->Option<(u8,enemies::EncounterKind)>{
    enemies::A0_ENCOUNTER_SETUPS.get(..M16_DEFECT_COMBAT_SETUP_PREFIX_LEN)?.iter()
        .find(|(id,act,_)|*id==encounter&&*act==1).map(|(_,act,kind)|(*act,*kind))
}
