//! Fixed, default-off M23 supplied starts: the design-pinned conditional Act2 boss
//! series and the six plan Task3 played fixtures. Each start is placed one map edge
//! before a real captured Act2 room; the first real Move is disclosed bootstrap.
//! The completed (1,2,17) frame is a constructed predecessor, never an earned Act1.
//! No arbitrary resources, powers, snapshot importer or normal supplied-run parser.
use super::super::*;
use crate::{combat::rng::Partition,content::{CardId,Character,EncounterId,PotionId,RelicId}};

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum M23DiagnosticBoss{Champ,BronzeAutomaton,Collector}
impl M23DiagnosticBoss{
    pub const ALL:[Self;3]=[Self::Champ,Self::BronzeAutomaton,Self::Collector];
    pub fn encounter(self)->EncounterId{match self{Self::Champ=>EncounterId::Champ,Self::BronzeAutomaton=>EncounterId::BronzeAutomaton,Self::Collector=>EncounterId::Collector}}
}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum M23PlayedFixture{FlightThreeByrds,PricesSnecko,PowerScryChosenAndByrds,WoundsSlavers,WoundsBookOfStabbing,StasisBronzeAutomaton}
impl M23PlayedFixture{
    pub const ALL:[Self;6]=[Self::FlightThreeByrds,Self::PricesSnecko,Self::PowerScryChosenAndByrds,Self::WoundsSlavers,Self::WoundsBookOfStabbing,Self::StasisBronzeAutomaton];
    pub fn name(self)->&'static str{match self{Self::FlightThreeByrds=>"flight-three-byrds",Self::PricesSnecko=>"prices-snecko",Self::PowerScryChosenAndByrds=>"power-scry-chosen-and-byrds",Self::WoundsSlavers=>"wounds-slavers",Self::WoundsBookOfStabbing=>"wounds-book-of-stabbing",Self::StasisBronzeAutomaton=>"stasis-bronze-automaton"}}
    pub fn encounter(self)->EncounterId{match self{Self::FlightThreeByrds=>EncounterId::ThreeByrds,Self::PricesSnecko=>EncounterId::Snecko,Self::PowerScryChosenAndByrds=>EncounterId::ChosenAndByrds,Self::WoundsSlavers=>EncounterId::Slavers,Self::WoundsBookOfStabbing=>EncounterId::BookOfStabbing,Self::StasisBronzeAutomaton=>EncounterId::BronzeAutomaton}}
    /// Exact plan Task3 deck order; `true` marks a once-upgraded instance.
    pub fn deck(self)->&'static [(CardId,bool)]{match self{
        Self::FlightThreeByrds=>&[(CardId::Eruption,true),(CardId::Vigilance,false),(CardId::EmptyFist,false),(CardId::FlurryOfBlows,false),(CardId::Prostrate,false)],
        Self::PricesSnecko=>&[(CardId::SandsOfTime,false),(CardId::Establishment,true),(CardId::Meditate,false),(CardId::WindmillStrike,false),(CardId::Vigilance,false)],
        Self::PowerScryChosenAndByrds=>&[(CardId::Foresight,true),(CardId::ThirdEye,false),(CardId::Weave,false),(CardId::Nirvana,false),(CardId::CutThroughFate,false)],
        Self::WoundsSlavers|Self::WoundsBookOfStabbing=>&[(CardId::Vigilance,false),(CardId::Meditate,false),(CardId::Perseverance,false),(CardId::Prostrate,false),(CardId::DefendPurple,false)],
        Self::StasisBronzeAutomaton=>&[(CardId::WindmillStrike,false),(CardId::Perseverance,false),(CardId::SandsOfTime,false),(CardId::Establishment,true),(CardId::BattleHymn,true)],
    }}
    pub fn relics(self)->&'static [RelicId]{match self{Self::FlightThreeByrds=>&[RelicId::PureWater,RelicId::VioletLotus],Self::PowerScryChosenAndByrds=>&[RelicId::GoldenEye],_=>&[RelicId::PureWater]}}
    pub fn potions(self)->[Option<PotionId>;3]{if self==Self::FlightThreeByrds{[Some(PotionId::StancePotion),Some(PotionId::Ambrosia),None]}else{[None;3]}}
}

fn random()->RunRng{let mut root=[0;32];root[..8].copy_from_slice(&7u64.to_le_bytes());RunRng::new(root,Partition::Test,0)}
fn creation(act2:EncounterId)->CreationProfile{
    let mut p=CreationProfile::experienced();p.neow_eligible=false;p.seen_bosses[0]=vec!["Hexaghost","SlimeBoss"];
    p.seen_bosses[1]=match act2{EncounterId::BronzeAutomaton=>vec!["Champ","Collector"],EncounterId::Collector=>vec!["Champ","Automaton"],_=>vec!["Automaton","Collector"]};p
}
fn resources(deck:impl IntoIterator<Item=(CardId,bool)>,relics:&[RelicId],potions:[Option<PotionId>;3])->AuthoredRunStart{
    AuthoredRunStart{hp:72,max_hp:72,gold:99,deck:deck.into_iter().enumerate().map(|(i,(id,up))|CardInstance{instance:i as u64+1,id,upgrades:u32::from(up),misc:0}).collect(),relics:relics.to_vec(),potions:potions.to_vec(),relic_counters:Default::default()}
}
/// One edge before the actual captured room selected by shared act2::capture_room.
fn supplied(start:AuthoredRunStart,encounter:EncounterId,boss:EncounterId)->Result<Run,RunError>{
    let config=RunConfig{character:Character::Watcher,ascension:0};let rng=random();
    let template=Run::new_room_scenario(config,creation(boss),rng.clone(),RoomScenario::M23WatcherAct2)?;
    // Reuse accepted complete resource validation, keeping the pinned ID/order bytes.
    M21WatcherCombatStart::Supplied(start.clone()).normalized()?;
    let next=start.deck.len() as u64+1;
    let mut r=Run::from_validated_combat_resources(config,start,rng,next);r.state.authored_combat=false;r.state.room_loop=template.state.room_loop;
    r.state.seen_bosses=template.state.seen_bosses;
    let (kind,history,floor)=match encounter{
        EncounterId::ThreeByrds=>(MapNodeKind::Monster,vec![EncounterId::SphericGuardian],20),
        EncounterId::Snecko=>(MapNodeKind::Monster,vec![EncounterId::SphericGuardian,EncounterId::ThreeByrds,EncounterId::CenturionAndHealer],20),
        EncounterId::ChosenAndByrds=>(MapNodeKind::Monster,vec![EncounterId::SphericGuardian,EncounterId::ThreeByrds,EncounterId::CenturionAndHealer,EncounterId::Snecko],20),
        EncounterId::Slavers|EncounterId::BookOfStabbing|EncounterId::GremlinLeader=>(MapNodeKind::Elite,vec![],20),
        EncounterId::Champ|EncounterId::BronzeAutomaton|EncounterId::Collector=>(MapNodeKind::Boss,vec![],33),
        _=>return Err(RunError::InvalidAction),
    };
    let previous=NodeId(0xF230_0000+u64::from(floor)-1);let node=NodeId(0xF230_0000+u64::from(floor));
    r.state.act=2;r.state.floor=floor-1;r.state.room="M23SuppliedAct2Entry";
    r.state.map=Some(RunMap{act:2,boss_icon:Some(boss),current:Some(previous),regular_encounter_history:history,
        nodes:vec![MapNode{id:previous,floor:floor-1,label:"Rest",edges:vec![node],x:0,y:0,kind:MapNodeKind::Rest,hidden_kind:None,visited:true,revealed:true,encounter:None,burning:false},
                   MapNode{id:node,floor,label:kind.label(),edges:vec![],x:0,y:1,kind,hidden_kind:None,visited:false,revealed:true,encounter:None,burning:false}],..RunMap::default()});
    let room=r.state.room_loop.as_mut().unwrap();room.act1=None;room.act2=Some(Default::default());room.act2_boss_order=Some(vec![boss]);
    // Constructed completed predecessor frame so the actual (2,3,34) owner can run.
    room.transition=Some(ActTransition{from:1,to:2,floor:17,reset_applied:true,heal_applied:true,initialized:true});
    room.screen=RoomView{kind:RoomViewKind::Map,choices:vec![]};r.candidates.clear();
    if rooms::act2::capture_room(&r.state,node)?.encounter!=Some(encounter){return Err(RunError::InvalidAction)}
    Ok(r)
}
/// Design-pinned conditional series: 72/72HP, 99 gold, four each Eruption+/Vigilance+/EmptyFist+/CarveReality+
/// (IDs1..16), PureWater/VioletLotus/Vajra/OddlySmoothStone, three empty slots.
pub fn m23_supplied_boss(boss:M23DiagnosticBoss)->Result<Run,RunError>{
    let deck=[CardId::Eruption,CardId::Vigilance,CardId::EmptyFist,CardId::CarveReality].into_iter().flat_map(|id|[(id,true);4]);
    supplied(resources(deck,&[RelicId::PureWater,RelicId::VioletLotus,RelicId::Vajra,RelicId::OddlySmoothStone],[None;3]),boss.encounter(),boss.encounter())
}
/// Plan Task3 fixture: real full-HP enemies via shared setup at the first real Move.
pub fn m23_played_fixture(f:M23PlayedFixture)->Result<Run,RunError>{
    let boss=match f.encounter(){EncounterId::Slavers|EncounterId::BronzeAutomaton=>EncounterId::BronzeAutomaton,EncounterId::BookOfStabbing=>EncounterId::Collector,_=>EncounterId::Champ};
    supplied(resources(f.deck().iter().copied(),f.relics(),f.potions()),f.encounter(),boss)
}

/// Constructed M23 owner stages for native fault/cutoff tests, mirroring M22's
/// fixed set on the Act2 boss source (33/34) and the (2,3,34) transition.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum M23Diagnostic{
    BossRewards{boss:M23DiagnosticBoss},BossRelicMenu{boss:M23DiagnosticBoss},
    BossChild{boss:M23DiagnosticBoss,branch:super::M22DiagnosticBranch},
    Act3Ready{boss:M23DiagnosticBoss,branch:super::M22DiagnosticBranch},
    ScryChoice,MeditateChoice,StanceChoice,BeforeStartupFault,
    TransitionResetFault,TransitionHealFault,
}
fn canonical(boss:EncounterId)->Result<Run,RunError>{Run::new_room_scenario(RunConfig{character:Character::Watcher,ascension:0},creation(boss),random(),RoomScenario::M23WatcherAct2)}
/// Explicitly constructed Act2 boss source (completed (1,2,17) predecessor,
/// visit0 Victory record), never a played receipt or earned Act1.
fn boss_source(boss:M23DiagnosticBoss)->Result<Run,RunError>{
    let mut r=canonical(boss.encounter())?;super::m22::choose(&mut r,RoomChoiceKind::NeowMaxHp{amount:7})?;
    let node=NodeId(0xF231_0033);r.state.act=2;r.state.floor=33;r.state.room="M23ConstructedBossSource";
    r.state.map=Some(RunMap{act:2,boss_icon:Some(boss.encounter()),current:Some(node),nodes:vec![MapNode{id:node,floor:33,label:"Boss",edges:vec![],x:0,y:15,kind:MapNodeKind::Boss,hidden_kind:None,visited:true,revealed:true,encounter:Some(boss.encounter()),burning:false}],..RunMap::default()});
    let record=CombatRecord{visit:0,encounter:boss.encounter(),outcome:crate::combat::Outcome::Victory};
    r.state.combat_progress=CombatProgress{entered:1,settled:1};r.state.last_combat=Some(record.clone());
    r.state.history.events.push(CommittedEvent::CombatFinished(record));r.state.history.rooms.push(VisitedRoom{act:2,floor:33,label:"M23ConstructedBossSource"});
    let room=r.state.room_loop.as_mut().unwrap();room.act1=None;room.act2=Some(Default::default());room.act2_boss_order=Some(vec![]);
    room.transition=Some(ActTransition{from:1,to:2,floor:17,reset_applied:true,heal_applied:true,initialized:true});
    let screen=rooms::view(&mut r.next_choice,RoomViewKind::Rewards,vec![(RoomChoiceKind::CardReward,None),(RoomChoiceKind::Proceed,None)])?;
    let mut cards=rooms::m22_content::reward_cards(true,0).into_iter().map(|id|(RoomChoiceKind::Card{id,upgrades:0},None)).collect::<Vec<_>>();cards.push((RoomChoiceKind::Skip,None));
    let captured=rooms::rewards::CapturedRewards{cards:rooms::view(&mut r.next_choice,RoomViewKind::CardReward,cards)?,return_to:rooms::rewards::RewardReturn::BossTreasure};
    let room=r.state.room_loop.as_mut().unwrap();room.screen=screen;room.active_rewards=Some(captured);r.candidates.clear();Ok(r)
}
fn boss_menu(boss:M23DiagnosticBoss)->Result<Run,RunError>{let mut r=boss_source(boss)?;super::m22::choose(&mut r,RoomChoiceKind::Proceed)?;super::m22::choose(&mut r,RoomChoiceKind::OpenChest)?;Ok(r)}
fn boss_child(boss:M23DiagnosticBoss,branch:super::M22DiagnosticBranch)->Result<Run,RunError>{
    let mut r=boss_menu(boss)?;r.state.hp=31;r.state.potions[0]=Some(PotionId::BottledMiracle);
    if let Some(id)=branch.relic(){super::m22::choose(&mut r,RoomChoiceKind::Relic(id))?;}else{super::m22::choose(&mut r,RoomChoiceKind::Skip)?;}
    Ok(r)
}
fn combat_choice(fixture:M23Diagnostic)->Result<Run,RunError>{
    let mut r=canonical(EncounterId::Champ)?;super::m22::choose(&mut r,RoomChoiceKind::NeowMaxHp{amount:7})?;
    if fixture==M23Diagnostic::ScryChoice{r.state.deck.iter_mut().for_each(|c|c.id=CardId::ThirdEye);}
    if fixture==M23Diagnostic::MeditateChoice{r.state.deck.iter_mut().for_each(|c|c.id=CardId::Meditate);}
    if fixture==M23Diagnostic::StanceChoice{r.state.potions[0]=Some(PotionId::StancePotion);}
    if fixture==M23Diagnostic::BeforeStartupFault{
        crate::items::acquire_relic(&mut r.state,RelicId::HolyWater)?;
        r.state.deck.last_mut().unwrap().instance=u64::MAX-1;r.state.next_instance=u64::MAX;
        return Ok(r)
    }
    let id=r.decision()?.candidates.iter().find_map(|c|if let GameAction::Move(id)=c.action{Some(id)}else{None}).ok_or(RunError::InvalidAction)?;
    r.step(&GameAction::Move(id))?;
    match fixture{
        M23Diagnostic::ScryChoice=>super::m22::play_kind(&mut r,CardId::ThirdEye)?,
        M23Diagnostic::MeditateChoice=>{r.step(&GameAction::Combat(crate::combat::CombatAction::EndTurn))?;super::m22::play_kind(&mut r,CardId::Meditate)?;},
        M23Diagnostic::StanceChoice=>{r.step(&GameAction::Combat(crate::combat::CombatAction::UsePotion{slot:0,target:None}))?;},
        _=>return Err(RunError::InvalidAction),
    }Ok(r)
}
/// Finite typed constructed stages only; callers must label the provenance.
pub fn m23_diagnostic(fixture:M23Diagnostic)->Result<Run,RunError>{match fixture{
    M23Diagnostic::BossRewards{boss}=>boss_source(boss),M23Diagnostic::BossRelicMenu{boss}=>boss_menu(boss),
    M23Diagnostic::BossChild{boss,branch}=>boss_child(boss,branch),
    M23Diagnostic::Act3Ready{boss,branch}=>{let mut r=boss_child(boss,branch)?;
        while r.state.pending.is_some(){let id=r.decision()?.candidates.iter().find_map(|c|if let GameAction::Select(id)=c.action{Some(id)}else{None}).ok_or(RunError::InvalidAction)?;r.step(&GameAction::Select(id))?;}
        super::m22::choose(&mut r,if branch==super::M22DiagnosticBranch::Skip{RoomChoiceKind::ConfirmSkip}else{RoomChoiceKind::Proceed})?;Ok(r)},
    M23Diagnostic::TransitionResetFault|M23Diagnostic::TransitionHealFault=>{
        let mut r=boss_child(M23DiagnosticBoss::Champ,super::M22DiagnosticBranch::HolyWater)?;
        r.state.room_loop.as_mut().unwrap().transition=Some(ActTransition{from:2,to:if fixture==M23Diagnostic::TransitionResetFault{3}else{4},floor:34,reset_applied:false,heal_applied:false,initialized:false});
        if fixture==M23Diagnostic::TransitionResetFault{r.state.max_hp=0;}
        let result=rooms::transition::drive(&mut r);if !matches!(result,Err(RunError::Faulted(_))){return Err(RunError::InvalidAction)}Ok(r)
    },other=>combat_choice(other),
}}

#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn m23_supplied_and_played_fixture_starts_are_exact_without_actions(){
        for boss in M23DiagnosticBoss::ALL{
            let r=m23_supplied_boss(boss).unwrap();assert_eq!((r.state.hp,r.state.max_hp,r.state.gold,r.state.act,r.state.floor),(72,72,99,2,32));
            assert_eq!(r.state.relics,vec![RelicId::PureWater,RelicId::VioletLotus,RelicId::Vajra,RelicId::OddlySmoothStone]);assert_eq!(r.state.potions,vec![None;3]);
            for (i,c) in r.state.deck.iter().enumerate(){assert_eq!((c.instance,c.upgrades,c.misc),(i as u64+1,1,0));assert_eq!(c.id,[CardId::Eruption,CardId::Vigilance,CardId::EmptyFist,CardId::CarveReality][i/4]);}
            assert_eq!(r.state.rng,random());assert!(r.state.combat.is_none()&&r.state.last_combat.is_none());assert_eq!(r.decision().unwrap().content_profile,Some(RoomScenario::M23WatcherAct2));
            let moves=r.decision().unwrap().candidates.iter().filter(|c|matches!(c.action,GameAction::Move(_))).count();assert_eq!(moves,1);
        }
        for f in M23PlayedFixture::ALL{
            let r=m23_played_fixture(f).unwrap();assert_eq!((r.state.hp,r.state.max_hp,r.state.gold,r.state.act),(72,72,99,2));
            assert_eq!(r.state.deck.iter().map(|c|(c.id,c.upgrades==1)).collect::<Vec<_>>(),f.deck().to_vec());assert_eq!(r.state.relics,f.relics().to_vec());assert_eq!(r.state.potions,f.potions().to_vec());
            let node=r.public_map().unwrap().nodes.iter().find(|n|!n.visited).unwrap().id;assert_eq!(rooms::act2::capture_room(&r.state,node).unwrap().encounter,Some(f.encounter()));assert_eq!(r.state.rng,random());
        }
    }
    #[test]
    fn m23_constructed_stages_reach_actual_owners_and_cutoff(){
        use super::super::M22DiagnosticBranch as R;
        for boss in M23DiagnosticBoss::ALL{
            let r=m23_diagnostic(M23Diagnostic::BossRelicMenu{boss}).unwrap();let d=r.decision().unwrap();assert_eq!((d.act,d.floor),(2,34));assert_eq!(d.room.unwrap().kind,RoomViewKind::BossRelicChoice);
            for branch in [R::HolyWater,R::VioletLotus,R::SacredBark,R::EmptyCage,R::SlaversCollar,R::Skip]{
                let r=m23_diagnostic(M23Diagnostic::Act3Ready{boss,branch}).unwrap();let d=r.decision().unwrap();
                assert_eq!((d.act,d.floor),(3,34));assert_eq!(d.room.unwrap().kind,RoomViewKind::Act3Ready);assert!(d.candidates.is_empty()&&d.terminal.is_none());assert_eq!(d.player.hp,d.player.max_hp);
            }
        }
        for f in [M23Diagnostic::ScryChoice,M23Diagnostic::MeditateChoice,M23Diagnostic::StanceChoice]{let r=m23_diagnostic(f).unwrap();let c=r.decision().unwrap().combat.unwrap();assert!(c.selection.is_some()||c.stance_choice.is_some(),"{f:?}");}
        let mut r=m23_diagnostic(M23Diagnostic::BeforeStartupFault).unwrap();let mv=r.decision().unwrap().candidates.iter().find(|c|matches!(c.action,GameAction::Move(_))).unwrap().action.clone();assert!(matches!(r.step(&mv),Err(RunError::Faulted(_))));
        for (f,reset,heal) in [(M23Diagnostic::TransitionResetFault,true,false),(M23Diagnostic::TransitionHealFault,true,true)]{
            let r=m23_diagnostic(f).unwrap();let t=r.state.room_loop.as_ref().unwrap().transition.clone().unwrap();assert_eq!((t.from,t.reset_applied,t.heal_applied,t.initialized),(2,reset,heal,false));assert!(r.state.fault.is_some());
        }
    }
}
