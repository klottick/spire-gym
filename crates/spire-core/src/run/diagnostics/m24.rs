//! Fixed, default-off M24 supplied starts and constructed stages. Supplied resources
//! are unearned and labelled; constructed sources (a completed predecessor frame, a
//! constructed boss Victory record) are never played receipts. Each supplied start sits
//! one map edge before a real captured room; the first real Move is disclosed bootstrap.
//! No arbitrary resources, snapshot importer or public supplied-run API.
use super::super::*;
use crate::{combat::{CombatAction,Outcome,rng::Partition},content::{CardId,Character,EncounterId,PotionId,RelicId}};
use super::m23::{M23DiagnosticBoss,boss_source,resources};

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum M24DiagnosticBoss{AwakenedOne,TimeEater,DonuAndDeca}
impl M24DiagnosticBoss{
    pub const ALL:[Self;3]=[Self::AwakenedOne,Self::TimeEater,Self::DonuAndDeca];
    pub fn name(self)->&'static str{match self{Self::AwakenedOne=>"awakened-one",Self::TimeEater=>"time-eater",Self::DonuAndDeca=>"donu-and-deca"}}
    pub fn encounter(self)->EncounterId{match self{Self::AwakenedOne=>EncounterId::AwakenedOne,Self::TimeEater=>EncounterId::TimeEater,Self::DonuAndDeca=>EncounterId::DonuAndDeca}}
    /// Review N1: the supplied traversal's Act2 boss is the seed-7 canonical pairing.
    pub fn traversal_act2(self)->M23DiagnosticBoss{match self{Self::AwakenedOne=>M23DiagnosticBoss::Champ,Self::TimeEater=>M23DiagnosticBoss::BronzeAutomaton,Self::DonuAndDeca=>M23DiagnosticBoss::Collector}}
}

/// Design §11 Act3 loadout v2, permanent IDs 1..18 in this order, all upgraded once.
pub const LOADOUT_V2_DECK:[CardId;18]=[
    CardId::MentalFortress,CardId::MentalFortress,CardId::LikeWater,CardId::BattleHymn,CardId::Devotion,
    CardId::Eruption,CardId::Eruption,CardId::Eruption,CardId::EmptyFist,CardId::EmptyFist,CardId::EmptyFist,
    CardId::CarveReality,CardId::WindmillStrike,CardId::Vigilance,CardId::Vigilance,CardId::Vigilance,CardId::Prostrate,CardId::ThirdEye,
];
pub const LOADOUT_V2_RELICS:[RelicId;6]=[RelicId::PureWater,RelicId::VioletLotus,RelicId::TeardropLocket,RelicId::GoldenEye,RelicId::Vajra,RelicId::OddlySmoothStone];
pub const LOADOUT_V2_POTIONS:[Option<PotionId>;3]=[Some(PotionId::BlockPotion),Some(PotionId::DexterityPotion),Some(PotionId::AncientPotion)];
pub const LOADOUT_V2_HP:u32=100;
fn loadout_v2()->AuthoredRunStart{let mut r=resources(LOADOUT_V2_DECK.into_iter().map(|id|(id,true)),&LOADOUT_V2_RELICS,LOADOUT_V2_POTIONS);r.hp=LOADOUT_V2_HP;r.max_hp=LOADOUT_V2_HP;r}

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum M24PlayedFixture{SlowGiantHead,IntangibleNemesis,DaggersReptomancer,ImplantWrithingMass,HalfDeadDarklings,BurnOrbWalker,ConstrictSpireGrowth,ThornsFourShapes,FadingTransient}
impl M24PlayedFixture{
    pub const ALL:[Self;9]=[Self::SlowGiantHead,Self::IntangibleNemesis,Self::DaggersReptomancer,Self::ImplantWrithingMass,Self::HalfDeadDarklings,Self::BurnOrbWalker,Self::ConstrictSpireGrowth,Self::ThornsFourShapes,Self::FadingTransient];
    pub fn name(self)->&'static str{match self{Self::SlowGiantHead=>"slow-giant-head",Self::IntangibleNemesis=>"intangible-nemesis",Self::DaggersReptomancer=>"daggers-reptomancer",Self::ImplantWrithingMass=>"implant-writhing-mass",Self::HalfDeadDarklings=>"half-dead-darklings",Self::BurnOrbWalker=>"burn-orb-walker",Self::ConstrictSpireGrowth=>"constrict-spire-growth",Self::ThornsFourShapes=>"thorns-four-shapes",Self::FadingTransient=>"fading-transient"}}
    pub fn encounter(self)->EncounterId{match self{Self::SlowGiantHead=>EncounterId::GiantHead,Self::IntangibleNemesis=>EncounterId::Nemesis,Self::DaggersReptomancer=>EncounterId::Reptomancer,Self::ImplantWrithingMass=>EncounterId::WrithingMass,Self::HalfDeadDarklings=>EncounterId::ThreeDarklings,Self::BurnOrbWalker=>EncounterId::OrbWalker,Self::ConstrictSpireGrowth=>EncounterId::SpireGrowth,Self::ThornsFourShapes=>EncounterId::FourShapes,Self::FadingTransient=>EncounterId::Transient}}
    /// Exact design §11 deck order; `true` marks a once-upgraded instance.
    pub fn deck(self)->&'static [(CardId,bool)]{use CardId::*;match self{
        Self::SlowGiantHead=>&[(FlurryOfBlows,false),(Eruption,false),(WindmillStrike,false),(Vigilance,false),(EmptyFist,false)],
        Self::IntangibleNemesis=>&[(FlurryOfBlows,false),(Eruption,true),(CarveReality,false),(ThirdEye,false),(Vigilance,false)],
        Self::DaggersReptomancer=>&[(EmptyFist,false),(StrikePurple,false),(StrikePurple,false),(Eruption,false),(Vigilance,false)],
        Self::ImplantWrithingMass=>&[(FlurryOfBlows,false),(WindmillStrike,false),(Meditate,false),(Vigilance,false),(Eruption,false)],
        Self::HalfDeadDarklings=>&[(Eruption,true),(EmptyFist,false),(StrikePurple,false),(StrikePurple,false),(Vigilance,false)],
        Self::BurnOrbWalker=>&[(ThirdEye,true),(Foresight,false),(Eruption,false),(Vigilance,false),(Weave,false)],
        Self::ConstrictSpireGrowth=>&[(Eruption,false),(Vigilance,false),(Prostrate,false),(LikeWater,false),(StrikePurple,false)],
        Self::ThornsFourShapes=>&[(FlurryOfBlows,false),(WindmillStrike,false),(Eruption,false),(Vigilance,false),(EmptyFist,false)],
        Self::FadingTransient=>&[(Eruption,true),(WindmillStrike,false),(Vigilance,false),(Perseverance,false),(Prostrate,false)],
    }}
    pub fn relics(self)->&'static [RelicId]{if self==Self::IntangibleNemesis{&[RelicId::PureWater,RelicId::VioletLotus]}else{&[RelicId::PureWater]}}
}

fn random()->RunRng{seeded(7)}
fn seeded(seed:u64)->RunRng{let mut root=[0;32];root[..8].copy_from_slice(&seed.to_le_bytes());RunRng::new(root,Partition::Test,0)}
fn creation(act3:EncounterId)->CreationProfile{super::m23::creation(EncounterId::Champ,Some(act3))}
/// Validated supplied resources on the M24 scenario template (no Neow, no map yet).
fn template(start:AuthoredRunStart,act3:EncounterId)->Result<Run,RunError>{template_at(start,act3,7)}
fn template_at(start:AuthoredRunStart,act3:EncounterId,seed:u64)->Result<Run,RunError>{
    let config=RunConfig{character:Character::Watcher,ascension:0};let rng=seeded(seed);
    let template=Run::new_room_scenario(config,creation(act3),rng.clone(),RoomScenario::M24WatcherAct3)?;
    M21WatcherCombatStart::Supplied(start.clone()).normalized()?;
    let next=start.deck.len() as u64+1;let mut r=Run::from_validated_combat_resources(config,start,rng,next);
    r.state.authored_combat=false;r.state.room_loop=template.state.room_loop;r.state.seen_bosses=template.state.seen_bosses;Ok(r)
}
/// One edge before a real captured Act3 room, after a constructed completed (2,3,34) frame.
fn act3_edge(mut r:Run,kind:MapNodeKind,floor:u32,icon:EncounterId,history:Vec<EncounterId>)->Result<(Run,NodeId),RunError>{
    let previous=NodeId(0xF240_0000+u64::from(floor)-1);let node=NodeId(0xF240_0000+u64::from(floor));
    r.state.act=3;r.state.floor=floor-1;r.state.room="M24SuppliedAct3Entry";
    r.state.map=Some(RunMap{act:3,boss_icon:Some(icon),current:Some(previous),regular_encounter_history:history,
        nodes:vec![MapNode{id:previous,floor:floor-1,label:"Rest",edges:vec![node],x:0,y:0,kind:MapNodeKind::Rest,hidden_kind:None,visited:true,revealed:true,encounter:None,burning:false},
                   MapNode{id:node,floor,label:kind.label(),edges:vec![],x:0,y:1,kind,hidden_kind:None,visited:false,revealed:true,encounter:None,burning:false}],..RunMap::default()});
    let room=r.state.room_loop.as_mut().unwrap();room.act1=None;room.act2=None;room.act3=Some(Default::default());room.act3_boss_order=Some(vec![icon]);
    room.transition=Some(ActTransition{from:2,to:3,floor:34,reset_applied:true,heal_applied:true,initialized:true});
    room.screen=RoomView{kind:RoomViewKind::Map,choices:vec![]};r.candidates.clear();Ok((r,node))
}

/// Supplied Act3 traversal (design §11): the constructed M23 Act2 boss source (completed
/// (1,2,17) frame, Act2 boss Victory record, boss card reward) on the M24 scenario with
/// loadout v2; every later room, the (2,3,34) transition and Act3 are played.
pub fn m24_supplied_traversal(boss:M24DiagnosticBoss)->Result<Run,RunError>{
    boss_source(boss.traversal_act2(),RoomScenario::M24WatcherAct3,Some(boss.encounter()),Some(loadout_v2()))
}
/// Supplied full-HP Act3 boss (design §11): floor-49 Rest, one edge to the real boss50.
pub fn m24_supplied_boss(boss:M24DiagnosticBoss,keys:[bool;3])->Result<Run,RunError>{m24_supplied_boss_at(boss,keys,7)}
/// Design §11 runs the supplied bosses at seeds 7 (no keys) and 11 (all keys); the seed
/// sets the canonical Run RNG root exactly as for normal starts.
pub fn m24_supplied_boss_at(boss:M24DiagnosticBoss,keys:[bool;3],seed:u64)->Result<Run,RunError>{
    let (mut r,node)=act3_edge(template_at(loadout_v2(),boss.encounter(),seed)?,MapNodeKind::Boss,50,boss.encounter(),vec![])?;r.state.keys=keys;
    if rooms::act3::capture_room(&r.state,node)?.encounter!=Some(boss.encounter()){return Err(RunError::InvalidAction)}Ok(r)
}
/// Supplied interaction fixture (design §11): 72/72 HP, 99 gold, empty belt, no keys.
pub fn m24_played_fixture(f:M24PlayedFixture)->Result<Run,RunError>{
    use EncounterId as E;let cycle=[E::SpireGrowth,E::Transient,E::FourShapes,E::Maw,E::SphereAndTwoShapes,E::JawWormHorde,E::ThreeDarklings,E::WrithingMass];
    let ordinary=|k:usize|{let mut h=vec![E::OrbWalker,E::ThreeShapes];h.extend_from_slice(&cycle[..k]);h};
    let (kind,icon,history)=match f.encounter(){
        E::GiantHead=>(MapNodeKind::Elite,E::AwakenedOne,vec![]),E::Nemesis=>(MapNodeKind::Elite,E::TimeEater,vec![]),E::Reptomancer=>(MapNodeKind::Elite,E::DonuAndDeca,vec![]),
        E::OrbWalker=>(MapNodeKind::Monster,E::AwakenedOne,vec![]),
        id=>(MapNodeKind::Monster,E::AwakenedOne,ordinary(cycle.iter().position(|c|*c==id).ok_or(RunError::InvalidAction)?)),
    };
    let start=resources(f.deck().iter().copied(),f.relics(),[None;3]);
    let (r,node)=act3_edge(template(start,icon)?,kind,40,icon,history)?;
    if rooms::act3::capture_room(&r.state,node)?.encounter!=Some(f.encounter()){return Err(RunError::InvalidAction)}Ok(r)
}

/// Constructed M24 owner stages for core and native tests. Every stage is constructed
/// provenance; callers must label it so and never count it as a played receipt.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum M24Diagnostic{
    /// Act3BossComplete after a constructed boss50 Victory record, with the given keys.
    BossSource{boss:M24DiagnosticBoss,keys:[bool;3]},
    /// The accepted ending source changes after Proceed: the next dialog faults once.
    EndingSourceFault,
    /// Exhausted choice ledger at Act3BossComplete: Proceed rejects before acceptance.
    EndingPreflightReject,
    /// The actual M24 (2,3,34) continuation from the constructed Act2 boss source (Skip).
    Act3Map{boss:M24DiagnosticBoss},
    Falling,Tomb{has_mask:bool},Chest,ScryChoice,MeditateChoice,StanceChoice,
}
fn boss_complete(boss:M24DiagnosticBoss,keys:[bool;3])->Result<Run,RunError>{
    let (mut r,node)=act3_edge(template(loadout_v2(),boss.encounter())?,MapNodeKind::Boss,50,boss.encounter(),vec![])?;r.state.keys=keys;
    let record=CombatRecord{visit:0,encounter:boss.encounter(),outcome:Outcome::Victory};
    r.state.floor=50;r.state.room="M24ConstructedBossSource";
    let map=r.state.map.as_mut().unwrap();map.current=Some(node);let n=map.nodes.iter_mut().find(|n|n.id==node).unwrap();n.visited=true;n.encounter=Some(boss.encounter());
    r.state.combat_progress=CombatProgress{entered:1,settled:1};r.state.last_combat=Some(record.clone());
    r.state.history.events.push(CommittedEvent::CombatFinished(record.clone()));r.state.history.rooms.push(VisitedRoom{act:3,floor:50,label:"M24ConstructedBossSource"});
    let screen=rooms::view(&mut r.next_choice,RoomViewKind::Act3BossComplete,vec![(RoomChoiceKind::Proceed,None)])?;
    let room=r.state.room_loop.as_mut().unwrap();room.screen=screen;room.act3_end=Some(rooms::act3_end::Act3EndFrame{node,record,phase:rooms::act3_end::Act3EndPhase::AwaitingSource,accepted:None});r.candidates.clear();Ok(r)
}
fn room_stage(kind:MapNodeKind,questions:usize,has_mask:bool)->Result<Run,RunError>{
    let (mut r,_)=act3_edge(template(loadout_v2(),EncounterId::AwakenedOne)?,kind,40,EncounterId::AwakenedOne,vec![])?;
    if has_mask{crate::items::acquire_relic(&mut r.state,RelicId::RedMask)?;}
    r.state.room_loop.as_mut().unwrap().act3.as_mut().unwrap().question_visits=questions;
    let id=r.decision()?.candidates.iter().find_map(|c|if let GameAction::Move(id)=c.action{Some(id)}else{None}).ok_or(RunError::InvalidAction)?;r.step(&GameAction::Move(id))?;Ok(r)
}
fn combat_choice(fixture:M24Diagnostic)->Result<Run,RunError>{
    let mut start=loadout_v2();
    if fixture==M24Diagnostic::ScryChoice{start.deck.iter_mut().for_each(|c|c.id=CardId::ThirdEye);}
    if fixture==M24Diagnostic::MeditateChoice{start.deck.iter_mut().for_each(|c|c.id=CardId::Meditate);}
    if fixture==M24Diagnostic::StanceChoice{start.potions=vec![Some(PotionId::StancePotion),None,None];}
    let (mut r,_)=act3_edge(template(start,EncounterId::AwakenedOne)?,MapNodeKind::Monster,40,EncounterId::AwakenedOne,vec![])?;
    let id=r.decision()?.candidates.iter().find_map(|c|if let GameAction::Move(id)=c.action{Some(id)}else{None}).ok_or(RunError::InvalidAction)?;r.step(&GameAction::Move(id))?;
    match fixture{
        M24Diagnostic::ScryChoice=>super::m22::play_kind(&mut r,CardId::ThirdEye)?,
        M24Diagnostic::MeditateChoice=>{r.step(&GameAction::Combat(CombatAction::EndTurn))?;super::m22::play_kind(&mut r,CardId::Meditate)?;},
        M24Diagnostic::StanceChoice=>{r.step(&GameAction::Combat(CombatAction::UsePotion{slot:0,target:None}))?;},
        _=>return Err(RunError::InvalidAction),
    }Ok(r)
}
/// Finite typed constructed stages only; callers must label the provenance.
pub fn m24_diagnostic(fixture:M24Diagnostic)->Result<Run,RunError>{match fixture{
    M24Diagnostic::BossSource{boss,keys}=>boss_complete(boss,keys),
    M24Diagnostic::EndingSourceFault=>{
        let mut r=boss_complete(M24DiagnosticBoss::TimeEater,[false;3])?;super::m22::choose(&mut r,RoomChoiceKind::Proceed)?;
        r.state.map.as_mut().unwrap().current=None;
        let id=r.decision()?.room.ok_or(RunError::InvalidAction)?.choices.first().ok_or(RunError::InvalidAction)?.id;
        if !matches!(r.step(&GameAction::Choose(id)),Err(RunError::Faulted(_))){return Err(RunError::InvalidAction)}Ok(r)
    },
    M24Diagnostic::EndingPreflightReject=>{let mut r=boss_complete(M24DiagnosticBoss::TimeEater,[false;3])?;r.next_choice=u64::MAX;Ok(r)},
    M24Diagnostic::Act3Map{boss}=>{
        let mut r=boss_source(boss.traversal_act2(),RoomScenario::M24WatcherAct3,Some(boss.encounter()),None)?;
        super::m22::choose(&mut r,RoomChoiceKind::Proceed)?;super::m22::choose(&mut r,RoomChoiceKind::OpenChest)?;
        super::m22::choose(&mut r,RoomChoiceKind::Skip)?;super::m22::choose(&mut r,RoomChoiceKind::ConfirmSkip)?;Ok(r)
    },
    M24Diagnostic::Falling=>room_stage(MapNodeKind::Question,0,false),
    M24Diagnostic::Tomb{has_mask}=>room_stage(MapNodeKind::Question,1,has_mask),
    M24Diagnostic::Chest=>room_stage(MapNodeKind::Treasure,0,false),
    other=>combat_choice(other),
}}

#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn m24_supplied_and_constructed_starts_are_exact_without_actions(){
        for boss in M24DiagnosticBoss::ALL{for keys in [[false;3],[true;3]]{
            let r=m24_supplied_boss(boss,keys).unwrap();assert_eq!((r.state.hp,r.state.max_hp,r.state.gold,r.state.act,r.state.floor,r.state.keys),(100,100,99,3,49,keys));
            assert_eq!(r.state.relics,LOADOUT_V2_RELICS.to_vec());assert_eq!(r.state.potions,LOADOUT_V2_POTIONS.to_vec());
            for (i,c) in r.state.deck.iter().enumerate(){assert_eq!((c.instance,c.id,c.upgrades,c.misc),(i as u64+1,LOADOUT_V2_DECK[i],1,0));}
            assert_eq!(r.state.rng,random());assert!(r.state.combat.is_none()&&r.state.last_combat.is_none());
            let d=r.decision().unwrap();assert_eq!(d.content_profile,Some(RoomScenario::M24WatcherAct3));
            assert_eq!(d.candidates.iter().filter(|c|matches!(c.action,GameAction::Move(_))).count(),1);
        }}
        for boss in M24DiagnosticBoss::ALL{
            let r=m24_supplied_traversal(boss).unwrap();assert_eq!((r.state.hp,r.state.max_hp,r.state.gold,r.state.act,r.state.floor),(100,100,99,2,33));
            assert_eq!(r.state.deck.iter().map(|c|(c.id,c.upgrades)).collect::<Vec<_>>(),LOADOUT_V2_DECK.iter().map(|id|(*id,1)).collect::<Vec<_>>());
            assert_eq!(r.decision().unwrap().room.unwrap().kind,RoomViewKind::Rewards);assert_eq!(r.state.map.as_ref().unwrap().boss_icon,Some(boss.traversal_act2().encounter()));
            assert_eq!(r.state.seen_bosses[2].len(),2,"targeted Act3 history");assert!(!r.state.seen_bosses[2].contains(&match boss{M24DiagnosticBoss::AwakenedOne=>"AwakenedOne",M24DiagnosticBoss::TimeEater=>"TimeEater",M24DiagnosticBoss::DonuAndDeca=>"DonuAndDeca"}));
        }
        for f in M24PlayedFixture::ALL{
            let r=m24_played_fixture(f).unwrap();assert_eq!((r.state.hp,r.state.max_hp,r.state.gold,r.state.act,r.state.keys),(72,72,99,3,[false;3]),"{f:?}");
            assert_eq!(r.state.deck.iter().map(|c|(c.id,c.upgrades==1)).collect::<Vec<_>>(),f.deck().to_vec());assert_eq!(r.state.relics,f.relics().to_vec());assert_eq!(r.state.potions,vec![None;3]);
            let node=r.public_map().unwrap().nodes.iter().find(|n|!n.visited).unwrap().id;assert_eq!(rooms::act3::capture_room(&r.state,node).unwrap().encounter,Some(f.encounter()),"{f:?}");assert_eq!(r.state.rng,random());
        }
    }
    #[test]
    fn m24_start_and_prefix_match_m23(){
        // Matched constructor inputs (same creation history incl. Act3, root, partition,
        // ordinal) through the constructed Act2 boss source, boss treasure, every relic
        // branch and the committed (2,3,34) transition. Only the scenario tag differs until
        // the continuation; then M23 publishes Act3Ready and M24 the same successor as Map.
        use super::super::M22DiagnosticBranch as B;
        let tag=|r:&Run|{let mut t=r.clone();t.state.room_loop.as_mut().unwrap().scenario=RoomScenario::M24WatcherAct3;t};
        for boss in M24DiagnosticBoss::ALL{for branch in [B::HolyWater,B::VioletLotus,B::SacredBark,B::EmptyCage,B::SlaversCollar,B::Skip]{
            let mut old=boss_source(boss.traversal_act2(),RoomScenario::M23WatcherAct2,Some(boss.encounter()),None).unwrap();
            let mut new=boss_source(boss.traversal_act2(),RoomScenario::M24WatcherAct3,Some(boss.encounter()),None).unwrap();assert_eq!(tag(&old),new);
            for r in [&mut old,&mut new]{
                super::super::m22::choose(r,RoomChoiceKind::Proceed).unwrap();super::super::m22::choose(r,RoomChoiceKind::OpenChest).unwrap();
                r.state.hp=31;r.state.potions[0]=Some(PotionId::BottledMiracle);
                if let Some(id)=branch.relic(){super::super::m22::choose(r,RoomChoiceKind::Relic(id)).unwrap();}else{super::super::m22::choose(r,RoomChoiceKind::Skip).unwrap();}
                while r.state.pending.is_some(){let id=r.decision().unwrap().candidates.iter().find_map(|c|if let GameAction::Select(id)=c.action{Some(id)}else{None}).unwrap();r.step(&GameAction::Select(id)).unwrap();}
            }
            assert_eq!(tag(&old),new,"{boss:?} {branch:?}: identical boss treasure and child");
            for r in [&mut old,&mut new]{super::super::m22::choose(r,if branch==B::Skip{RoomChoiceKind::ConfirmSkip}else{RoomChoiceKind::Proceed}).unwrap();}
            let a=old.decision().unwrap();let b=new.decision().unwrap();
            assert_eq!(a.room.as_ref().unwrap().kind,RoomViewKind::Act3Ready);assert!(a.candidates.is_empty());
            assert_eq!(b.room.as_ref().unwrap().kind,RoomViewKind::Map);assert!(b.candidates.iter().any(|c|matches!(c.action,GameAction::Move(_))));
            assert_eq!(new.state.room_loop.as_ref().unwrap().act3,Some(Default::default()));
            let mut endpoint=tag(&old);endpoint.state.room="Map";let room=endpoint.state.room_loop.as_mut().unwrap();room.act3=Some(Default::default());room.screen.kind=RoomViewKind::Map;
            assert_eq!(endpoint,new,"{boss:?} {branch:?}: same successor map, boss order, resources, records, flags and RNG");
            assert_eq!(new.public_map().unwrap().boss,Some(boss.encounter()));
        }}
    }
    #[test]
    fn m24_constructed_stages_reach_actual_owners(){
        for boss in M24DiagnosticBoss::ALL{
            let r=m24_diagnostic(M24Diagnostic::BossSource{boss,keys:[true;3]}).unwrap();let d=r.decision().unwrap();assert_eq!((d.act,d.floor),(3,50));assert_eq!(d.room.unwrap().kind,RoomViewKind::Act3BossComplete);
            let r=m24_diagnostic(M24Diagnostic::Act3Map{boss}).unwrap();let d=r.decision().unwrap();assert_eq!((d.act,d.floor),(3,34));assert_eq!(d.room.unwrap().kind,RoomViewKind::Map);
            assert_eq!(r.public_map().unwrap().boss,Some(boss.encounter()));assert!(d.candidates.iter().any(|c|matches!(c.action,GameAction::Move(_))));
        }
        let r=m24_diagnostic(M24Diagnostic::EndingSourceFault).unwrap();assert_eq!(r.state.fault.as_ref().unwrap().checkpoint,"run.m24.act3.end");
        let mut r=m24_diagnostic(M24Diagnostic::EndingPreflightReject).unwrap();let id=r.decision().unwrap().room.unwrap().choices[0].id;let before=r.clone();assert!(r.step(&GameAction::Choose(id)).is_err());assert_eq!(r,before);
        assert_eq!(m24_diagnostic(M24Diagnostic::Falling).unwrap().decision().unwrap().room.unwrap().kind,RoomViewKind::Event(crate::content::EventId::Falling));
        for has_mask in [false,true]{let r=m24_diagnostic(M24Diagnostic::Tomb{has_mask}).unwrap();let rows=r.decision().unwrap().room.unwrap().choices;assert!(rows.iter().any(|c|c.kind==if has_mask{RoomChoiceKind::TombGold}else{RoomChoiceKind::TombOfferGold}));}
        assert_eq!(m24_diagnostic(M24Diagnostic::Chest).unwrap().decision().unwrap().room.unwrap().kind,RoomViewKind::Treasure);
        for f in [M24Diagnostic::ScryChoice,M24Diagnostic::MeditateChoice,M24Diagnostic::StanceChoice]{let r=m24_diagnostic(f).unwrap();let c=r.decision().unwrap().combat.unwrap();assert!(c.selection.is_some()||c.stance_choice.is_some(),"{f:?}");}
    }
}
