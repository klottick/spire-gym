//! Fixed, default-off M25 constructed stages (design §12.7). Constructed sources (a
//! constructed Act3 boss Victory record, supplied keys) are never played receipts, and
//! callers must label them so. This module uses no M24 diagnostics helper. It has no
//! arbitrary resources, no snapshot importer and no public supplied-run API.
use super::super::*;
use crate::{combat::{Actor,CombatAction,DamagePayload,Effect,EffectKind,Outcome,Source,rng::Partition},content::{CardId,Character,EncounterId,PotionId,RelicId}};
use super::m23::{M23DiagnosticBoss,boss_source_for,resources};

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum M25DiagnosticBoss{AwakenedOne,TimeEater,DonuAndDeca}
impl M25DiagnosticBoss{
    pub const ALL:[Self;3]=[Self::AwakenedOne,Self::TimeEater,Self::DonuAndDeca];
    pub fn name(self)->&'static str{match self{Self::AwakenedOne=>"awakened-one",Self::TimeEater=>"time-eater",Self::DonuAndDeca=>"donu-and-deca"}}
    pub fn encounter(self)->EncounterId{match self{Self::AwakenedOne=>EncounterId::AwakenedOne,Self::TimeEater=>EncounterId::TimeEater,Self::DonuAndDeca=>EncounterId::DonuAndDeca}}
}

fn random()->RunRng{let mut root=[0;32];root[..8].copy_from_slice(&7u64.to_le_bytes());RunRng::new(root,Partition::Test,0)}
/// Guardian and Champ histories, plus the given Act3 boss's history.
fn creation(act3:EncounterId)->CreationProfile{
    let mut p=CreationProfile::experienced();p.neow_eligible=false;
    p.seen_bosses[0]=vec!["Hexaghost","SlimeBoss"];p.seen_bosses[1]=vec!["Automaton","Collector"];
    p.seen_bosses[2]=match act3{EncounterId::TimeEater=>vec!["AwakenedOne","DonuAndDeca"],EncounterId::DonuAndDeca=>vec!["AwakenedOne","TimeEater"],_=>vec!["TimeEater","DonuAndDeca"]};p
}

/// The Act3 boss Victory source (design §12.7): a Watcher A0 Run of `scenario` (M25, or M24
/// for the prefix check), with resources checked under `admission` (canonical when `None`).
/// It sets exactly the state the real ending owner reads; only that owner runs afterwards.
pub(super) fn act3_victory_source(scenario:RoomScenario,boss:M25DiagnosticBoss,keys:[bool;3],resources:Option<AuthoredRunStart>,admission:WatcherAdmission)->Result<Run,RunError>{
    if !matches!(scenario,RoomScenario::M24WatcherAct3|RoomScenario::M25WatcherHeart){return Err(RunError::InvalidAction)}
    let config=RunConfig{character:Character::Watcher,ascension:0};let rng=random();
    let template=Run::new_room_scenario(config,creation(boss.encounter()),rng.clone(),scenario)?;
    let start=resources.unwrap_or_else(||{let s=&template.state;AuthoredRunStart{hp:s.hp,max_hp:s.max_hp,gold:s.gold,deck:s.deck.clone(),relics:s.relics.clone(),potions:s.potions.clone(),relic_counters:s.relic_counters.clone()}});
    M21WatcherCombatStart::Supplied(start.clone()).normalized_for(admission)?;
    let next=start.deck.iter().map(|c|c.instance).max().unwrap_or(0).checked_add(1).ok_or(RunError::Unsupported("permanent identity exhausted"))?;
    let mut r=Run::from_validated_combat_resources(config,start,rng,next);
    r.state.authored_combat=false;r.state.room_loop=template.state.room_loop;r.state.seen_bosses=template.state.seen_bosses;
    let encounter=boss.encounter();let node=NodeId(0xF250_0050);
    let record=CombatRecord{visit:0,encounter,outcome:Outcome::Victory};
    r.state.act=3;r.state.floor=50;r.state.room="M25ConstructedAct3Source";r.state.keys=keys;
    r.state.map=Some(RunMap{act:3,boss_icon:Some(encounter),current:Some(node),nodes:vec![MapNode{id:node,floor:50,label:"Boss",edges:vec![],x:0,y:15,kind:MapNodeKind::Boss,hidden_kind:None,visited:true,revealed:true,encounter:Some(encounter),burning:false}],..RunMap::default()});
    r.state.combat_progress=CombatProgress{entered:1,settled:1};r.state.last_combat=Some(record.clone());
    r.state.history.events.push(CommittedEvent::CombatFinished(record.clone()));r.state.history.rooms.push(VisitedRoom{act:3,floor:50,label:"M25ConstructedAct3Source"});
    let screen=rooms::view(&mut r.next_choice,RoomViewKind::Act3BossComplete,vec![(RoomChoiceKind::Proceed,None)])?;
    let room=r.state.room_loop.as_mut().ok_or(RunError::InvalidAction)?;
    room.act1=None;room.act2=None;room.act3=Some(Default::default());room.act3_boss_order=Some(vec![encounter]);
    room.transition=Some(ActTransition{from:2,to:3,floor:34,reset_applied:true,heal_applied:true,initialized:true});
    room.screen=screen;room.act3_end=Some(rooms::act3_end::Act3EndFrame{node,record,phase:rooms::act3_end::Act3EndPhase::AwaitingSource,accepted:None});
    r.candidates.clear();Ok(r)
}

impl M25DiagnosticBoss{
    /// The supplied traversal's Act2 boss: M24's seed-7 canonical pairing (M24 review N1).
    pub fn traversal_act2(self)->M23DiagnosticBoss{match self{Self::AwakenedOne=>M23DiagnosticBoss::Champ,Self::TimeEater=>M23DiagnosticBoss::BronzeAutomaton,Self::DonuAndDeca=>M23DiagnosticBoss::Collector}}
}

/// Design §12.4 strong loadout v1 and §12.5 coverage loadouts (permanent IDs in this order;
/// `true` marks a once-upgraded instance). Resources are supplied and unearned.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum M25DiagnosticLoadout{CoverageA,CoverageB,CoverageC,Strong}
impl M25DiagnosticLoadout{
    pub const ALL:[Self;4]=[Self::CoverageA,Self::CoverageB,Self::CoverageC,Self::Strong];
    pub fn name(self)->&'static str{match self{Self::CoverageA=>"coverage-a",Self::CoverageB=>"coverage-b",Self::CoverageC=>"coverage-c",Self::Strong=>"strong"}}
    /// The Act3 source boss of the supplied Act4 case.
    pub fn act3(self)->M25DiagnosticBoss{match self{Self::CoverageA|Self::Strong=>M25DiagnosticBoss::AwakenedOne,Self::CoverageB=>M25DiagnosticBoss::TimeEater,Self::CoverageC=>M25DiagnosticBoss::DonuAndDeca}}
    pub fn deck(self)->&'static [(CardId,bool)]{use CardId::*;match self{
        Self::CoverageA=>&[(Eruption,false),(Vigilance,false),(MentalFortress,true),(Rushdown,false),(BowlingBash,false),(Consecrate,true),(Crescendo,false),(CrushJoints,true),(FlyingSleeves,false),(FollowUp,true),
            (SashWhip,false),(Tantrum,true),(Indignation,false),(FearNoEvil,true),(SignatureMove,false),(WheelKick,true),(Wallop,false),(Ragnarok,true),(Judgment,false),(Conclude,true)],
        Self::CoverageB=>&[(Vigilance,false),(ThirdEye,false),(LikeWater,true),(EmptyBody,false),(Halt,true),(Protect,false),(Tranquility,true),(Evaluate,false),(Pray,true),(Sanctity,false),
            (Swivel,true),(InnerPeace,false),(EmptyMind,true),(WaveOfTheHand,false),(SpiritShield,true),(Scrawl,false),(PressurePoints,true),(TalkToTheHand,false),(Study,true)],
        Self::CoverageC=>&[(Eruption,false),(Vigilance,false),(MasterReality,true),(BattleHymn,false),(Collect,false),(Fasting,true),(SimmeringFury,false),(WreathOfFlame,true),(ReachHeaven,false),
            (ForeignInfluence,true),(Alpha,false),(ConjureBlade,true),(DeusExMachina,false),(DevaForm,true),(LessonLearned,false),(Omniscience,true),(Vault,false),(Wish,true)],
        Self::Strong=>&[(PressurePoints,true);10],
    }}
    pub fn relics(self)->&'static [RelicId]{use RelicId::*;match self{
        Self::CoverageA=>&[PureWater,VioletLotus,Duality,Damaru],Self::CoverageB=>&[PureWater,TeardropLocket,GoldenEye,CloakClasp,Melange],
        Self::CoverageC=>&[PureWater,Vajra,OddlySmoothStone],Self::Strong=>&[PureWater]}}
    pub fn potions(self)->[Option<PotionId>;3]{use PotionId::*;match self{
        Self::CoverageA=>[Some(StancePotion),Some(BlockPotion),Some(AncientPotion)],Self::CoverageB=>[Some(BlockPotion),Some(DexterityPotion),Some(BottledMiracle)],
        Self::CoverageC=>[Some(EnergyPotion),Some(Ambrosia),Some(StancePotion)],Self::Strong=>[None;3]}}
    pub fn hp(self)->u32{if self==Self::Strong{200}else{100}}
    pub fn resources(self)->AuthoredRunStart{let mut r=resources(self.deck().iter().copied(),self.relics(),self.potions());r.hp=self.hp();r.max_hp=self.hp();r}
}

/// Design §12.6 interaction fixtures: 72/72 HP, 99 gold, three empty potion slots, all keys.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum M25PlayedFixture{FacingShieldSpear,BurnsShieldSpear,GenerationShieldSpear,PowersShieldSpear,BeatHeart,InvincibleHeart,StatusesHeart,TurnControlHeart}
impl M25PlayedFixture{
    pub const ALL:[Self;8]=[Self::FacingShieldSpear,Self::BurnsShieldSpear,Self::GenerationShieldSpear,Self::PowersShieldSpear,Self::BeatHeart,Self::InvincibleHeart,Self::StatusesHeart,Self::TurnControlHeart];
    pub fn name(self)->&'static str{match self{Self::FacingShieldSpear=>"x1-facing-shield-spear",Self::BurnsShieldSpear=>"x2-burns-shield-spear",Self::GenerationShieldSpear=>"x3-generation-shield-spear",
        Self::PowersShieldSpear=>"x4-powers-shield-spear",Self::BeatHeart=>"x5-beat-heart",Self::InvincibleHeart=>"x6-invincible-heart",Self::StatusesHeart=>"x7-statuses-heart",Self::TurnControlHeart=>"x8-turn-control-heart"}}
    pub fn encounter(self)->EncounterId{match self{Self::FacingShieldSpear|Self::BurnsShieldSpear|Self::GenerationShieldSpear|Self::PowersShieldSpear=>EncounterId::ShieldAndSpear,_=>EncounterId::CorruptHeart}}
    pub fn deck(self)->&'static [(CardId,bool)]{use CardId::*;match self{
        Self::FacingShieldSpear=>&[(BowlingBash,false),(Consecrate,true),(Ragnarok,false),(TalkToTheHand,false),(Wallop,false)],
        Self::BurnsShieldSpear=>&[(Tantrum,false),(Indignation,false),(Crescendo,false),(FlyingSleeves,false),(Halt,true)],
        Self::GenerationShieldSpear=>&[(Alpha,true),(ConjureBlade,false),(ReachHeaven,false),(ForeignInfluence,true),(Study,true)],
        Self::PowersShieldSpear=>&[(DevaForm,true),(Fasting,false),(SimmeringFury,false),(WaveOfTheHand,false),(Collect,false)],
        Self::BeatHeart=>&[(FollowUp,false),(SashWhip,false),(CrushJoints,false),(Sanctity,false),(Evaluate,false)],
        Self::InvincibleHeart=>&[(PressurePoints,true),(Judgment,false),(SignatureMove,false),(WheelKick,false),(Eruption,true)],
        Self::StatusesHeart=>&[(InnerPeace,false),(EmptyMind,false),(Pray,false),(Scrawl,false),(ThirdEye,true)],
        Self::TurnControlHeart=>&[(Vault,false),(Conclude,false),(Blasphemy,false),(Swivel,false),(WreathOfFlame,false)],
    }}
    pub fn relics(self)->&'static [RelicId]{use RelicId::*;match self{Self::PowersShieldSpear=>&[PureWater,Damaru,Duality,CloakClasp],Self::StatusesHeart=>&[PureWater,Melange,GoldenEye],_=>&[PureWater]}}
}

/// Supplied traversal (design §12.4): the constructed M23 Act2 boss source on the M25 scenario
/// with the boss's Act3 history, no keys and strong loadout v1 under the M25 admission.
pub fn m25_supplied_traversal(boss:M25DiagnosticBoss)->Result<Run,RunError>{
    boss_source_for(boss.traversal_act2(),RoomScenario::M25WatcherHeart,Some(boss.encounter()),Some(M25DiagnosticLoadout::Strong.resources()),WatcherAdmission::M25)
}
/// Supplied Act4 and Heart (design §12.5): the constructed Act3 boss Victory source with all
/// three (supplied, unearned) keys and the loadout's resources.
pub fn m25_supplied_act4(loadout:M25DiagnosticLoadout)->Result<Run,RunError>{
    act3_victory_source(RoomScenario::M25WatcherHeart,loadout.act3(),[true;3],Some(loadout.resources()),WatcherAdmission::M25)
}

fn route(r:&Run,floor:u32)->Result<NodeId,RunError>{r.state.map.as_ref().filter(|m|m.act==4).and_then(|m|m.nodes.iter().find(|n|n.floor==floor)).map(|n|n.id).ok_or(RunError::InvalidAction)}
/// Proceed, the three dialog stages and the real door, all through the shared owners.
fn through_door(mut r:Run)->Result<Run,RunError>{
    super::m22::choose(&mut r,RoomChoiceKind::Proceed)?;for stage in 1..=4{super::m22::choose(&mut r,RoomChoiceKind::Act3Continue{stage})?;}
    if r.state.act!=4{return Err(RunError::InvalidAction)}Ok(r)
}
/// Constructed Act4 position one edge before `floor`; before the Heart, also a constructed
/// recorded Shield and Spear Victory with its rewards closed.
fn act4_edge(r:&mut Run,floor:u32)->Result<(),RunError>{
    if !(51..=54).contains(&floor){return Err(RunError::InvalidAction)}
    if floor>51{
        let previous=route(r,floor-1)?;let map=r.state.map.as_mut().ok_or(RunError::InvalidAction)?;
        for n in &mut map.nodes{if n.floor<floor{n.visited=true;}}map.current=Some(previous);r.state.floor=floor-1;
    }
    if floor==54{
        let settled=r.state.combat_progress.settled;let record=CombatRecord{visit:settled,encounter:EncounterId::ShieldAndSpear,outcome:Outcome::Victory};
        let elite=route(r,53)?;r.state.map.as_mut().unwrap().nodes.iter_mut().find(|n|n.id==elite).unwrap().encounter=Some(EncounterId::ShieldAndSpear);
        r.state.combat_progress=CombatProgress{entered:settled+1,settled:settled+1};r.state.history.events.push(CommittedEvent::CombatFinished(record.clone()));
        let progress=r.state.room_loop.as_mut().and_then(|room|room.act4.as_mut()).ok_or(RunError::InvalidAction)?;
        progress.elite_record=Some(record);progress.combat_visits=1;progress.elite_rewards_closed=true;
    }
    Ok(())
}
/// Design §12.6 fixture: the real door from `act3_victory_source`, then only the Act4 position.
pub fn m25_played_fixture(f:M25PlayedFixture)->Result<Run,RunError>{
    let start=resources(f.deck().iter().copied(),f.relics(),[None;3]);
    let mut r=through_door(act3_victory_source(RoomScenario::M25WatcherHeart,M25DiagnosticBoss::AwakenedOne,[true;3],Some(start),WatcherAdmission::M25)?)?;
    act4_edge(&mut r,if f.encounter()==EncounterId::ShieldAndSpear{53}else{54})?;Ok(r)
}

/// Constructed M25 owner stages for core and native tests. Every stage is constructed
/// provenance; callers must label it so and never count it as a played receipt.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum M25Diagnostic{
    /// `Act3BossComplete` after a constructed Act3 boss Victory, canonical resources, given keys.
    DoorSource{boss:M25DiagnosticBoss,keys:[bool;3]},
    /// A post-accept door failure (constructed invalid HP between preparation and commit),
    /// retained once as `run.m25.act4.transition`.
    DoorFault,
    /// After the real door and a constructed position, the real Move into floor 51–54.
    Act4Room{floor:u32},
    /// The `HeartVictory` screen after a real Heart combat ended by a supplied lethal frontier
    /// (constructed 999 max HP).
    HeartVictoryPending,
    /// The Heart Victory Proceed preparation fails at settlement (exhausted choice ledger),
    /// retained once as `run.m25.heart.victory`.
    HeartProceedFault,
    /// Card-behaviour stages (need the content delivery): the offer or selection is open.
    ForeignInfluenceOffer,WishChoice,OmniscienceChoice,
}
fn door_source(boss:M25DiagnosticBoss,keys:[bool;3],start:Option<AuthoredRunStart>)->Result<Run,RunError>{act3_victory_source(RoomScenario::M25WatcherHeart,boss,keys,start,WatcherAdmission::M25)}
fn act4_room(start:Option<AuthoredRunStart>,floor:u32)->Result<Run,RunError>{
    let mut r=through_door(door_source(M25DiagnosticBoss::AwakenedOne,[true;3],start)?)?;act4_edge(&mut r,floor)?;
    if floor==54{r.state.max_hp=999;r.state.hp=999;}
    let target=route(&r,floor)?;r.step(&GameAction::Move(target))?;Ok(r)
}
fn hit_all(r:&mut Run,amount:i32)->Result<(),RunError>{
    let combat=r.state.combat.as_mut().ok_or(RunError::InvalidAction)?;
    let alive=combat.public_full()?.enemies.into_iter().filter(|e|e.hp>0).map(|e|e.instance_id).collect::<Vec<_>>();
    for id in alive{combat.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Enemy(id)),kind:EffectKind::Damage(DamagePayload::Fixed(amount))}],None).map_err(|_|RunError::InvalidAction)?;}
    Ok(())
}
/// A real Heart combat ended by a supplied lethal frontier; the combat is left unsettled.
fn killed_heart()->Result<Run,RunError>{
    let mut r=act4_room(None,54)?;
    for _ in 0..6{
        hit_all(&mut r,1_000)?;if r.state.combat.as_ref().ok_or(RunError::InvalidAction)?.public_full()?.outcome==Some(Outcome::Victory){return Ok(r)}
        r.step(&GameAction::Combat(CombatAction::EndTurn))?;
    }
    Err(RunError::InvalidAction)
}
fn card_choice(card:CardId,upgraded:bool)->Result<Run,RunError>{
    // Omniscience: 5 in hand and 3 in draw, so a chained choice closes within four public steps.
    let copies=if card==CardId::Omniscience{8}else{10};
    let start=resources(std::iter::repeat((card,upgraded)).take(copies),&[RelicId::PureWater],[None;3]);
    let mut r=act4_room(Some(start),53)?;super::m22::play_kind(&mut r,card)?;Ok(r)
}
/// Finite typed constructed stages only; callers must label the provenance.
pub fn m25_diagnostic(d:M25Diagnostic)->Result<Run,RunError>{match d{
    M25Diagnostic::DoorSource{boss,keys}=>door_source(boss,keys,None),
    M25Diagnostic::DoorFault=>{
        let mut r=door_source(M25DiagnosticBoss::TimeEater,[true;3],None)?;super::m22::choose(&mut r,RoomChoiceKind::Proceed)?;
        for stage in 1..=3{super::m22::choose(&mut r,RoomChoiceKind::Act3Continue{stage})?;}
        let prepared=rooms::act4::prepare_transition(&r)?;r.state.hp=r.state.max_hp+1;rooms::act4::commit_transition(&mut r,prepared);
        if r.state.fault.is_none(){return Err(RunError::InvalidAction)}Ok(r)
    },
    M25Diagnostic::Act4Room{floor}=>act4_room(None,floor),
    M25Diagnostic::HeartVictoryPending=>{let mut r=killed_heart()?;r.settle_combat()?;Ok(r)},
    M25Diagnostic::HeartProceedFault=>{
        let mut r=killed_heart()?;r.next_choice=u64::MAX;
        if !matches!(r.settle_combat(),Err(RunError::Faulted(_))){return Err(RunError::InvalidAction)}Ok(r)
    },
    M25Diagnostic::ForeignInfluenceOffer=>card_choice(CardId::ForeignInfluence,false),
    M25Diagnostic::WishChoice=>card_choice(CardId::Wish,false),
    M25Diagnostic::OmniscienceChoice=>card_choice(CardId::Omniscience,true),
}}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::CardId;
    fn choose(r:&mut Run,kind:RoomChoiceKind)->RunTransition{
        let room=r.decision().unwrap().room.unwrap();let row=room.choices.iter().find(|c|c.kind==kind&&c.status==RoomChoiceStatus::Offered).unwrap_or_else(||panic!("expected {kind:?}, got {room:?}"));
        r.step(&GameAction::Choose(row.id)).unwrap()
    }
    fn masks()->impl Iterator<Item=[bool;3]>{(0u8..8).map(|m|[m&1!=0,m&2!=0,m&4!=0])}

    #[test]
    fn m25_act3_victory_source_runs_the_real_ending(){
        for boss in M25DiagnosticBoss::ALL{for keys in masks(){
            let mut r=act3_victory_source(RoomScenario::M25WatcherHeart,boss,keys,None,WatcherAdmission::M21).unwrap();
            let encounter=boss.encounter();let record=CombatRecord{visit:0,encounter,outcome:Outcome::Victory};
            let d=r.decision().unwrap();
            assert_eq!((d.act,d.floor,d.content_profile,d.terminal),(3,50,Some(RoomScenario::M25WatcherHeart),None),"{boss:?} {keys:?}");
            assert_eq!((r.state.hp,r.state.max_hp,r.state.gold,r.state.relics.clone()),(72,72,99,vec![crate::content::RelicId::PureWater]),"canonical resources");
            assert_eq!(r.state.deck.len(),10);assert!(r.state.deck.iter().all(|c|matches!(c.id,CardId::StrikePurple|CardId::DefendPurple|CardId::Eruption|CardId::Vigilance)&&c.upgrades==0));
            let map=r.state.map.as_ref().unwrap();assert_eq!((map.act,map.boss_icon),(3,Some(encounter)));
            let node=map.nodes.iter().find(|n|Some(n.id)==map.current).unwrap();
            assert_eq!((node.floor,node.kind,node.visited,node.encounter),(50,MapNodeKind::Boss,true,Some(encounter)));
            assert_eq!(r.state.combat_progress,CombatProgress{entered:1,settled:1});assert_eq!(r.state.last_combat,Some(record.clone()));
            assert_eq!(r.state.history.events,vec![CommittedEvent::CombatFinished(record.clone())],"one CombatFinished, zero accepted actions");
            assert_eq!(r.state.keys,keys);
            let room=d.room.unwrap();assert_eq!(room.kind,RoomViewKind::Act3BossComplete);
            assert_eq!(room.choices.iter().map(|c|c.kind.clone()).collect::<Vec<_>>(),vec![RoomChoiceKind::Proceed]);
            let frame=r.state.room_loop.as_ref().unwrap().act3_end.clone().unwrap();
            assert_eq!((frame.node,frame.record,frame.phase,frame.accepted),(map.current.unwrap(),record,rooms::act3_end::Act3EndPhase::AwaitingSource,None));
            choose(&mut r,RoomChoiceKind::Proceed);for stage in 1..=3{choose(&mut r,RoomChoiceKind::Act3Continue{stage});}
            let room=r.decision().unwrap().room.unwrap();assert_eq!(room.kind,RoomViewKind::Act3Ending);
            assert_eq!(room.choices.iter().map(|c|c.kind.clone()).collect::<Vec<_>>(),vec![RoomChoiceKind::Act3Continue{stage:4}],"the real ending reaches stage 4");
        }}
        // A supplied start outside the M21 admission set rejects before any Run is returned.
        let mut start=AuthoredRunStart{hp:72,max_hp:72,gold:99,deck:vec![CardInstance{instance:1,id:CardId::Parasite,upgrades:0,misc:0}],relics:vec![crate::content::RelicId::PureWater],potions:vec![None;3],relic_counters:Default::default()};
        assert_eq!(act3_victory_source(RoomScenario::M25WatcherHeart,M25DiagnosticBoss::TimeEater,[true;3],Some(start.clone()),WatcherAdmission::M21).unwrap_err(),RunError::Unsupported("card outside finite M21 permanent profile"));
        start.deck[0].id=CardId::Vigilance;
        let r=act3_victory_source(RoomScenario::M25WatcherHeart,M25DiagnosticBoss::TimeEater,[true;3],Some(start),WatcherAdmission::M21).unwrap();
        assert_eq!(r.state.deck,vec![CardInstance{instance:1,id:CardId::Vigilance,upgrades:0,misc:0}]);assert_eq!(r.state.next_instance,2);
        assert_eq!(act3_victory_source(RoomScenario::M23WatcherAct2,M25DiagnosticBoss::TimeEater,[true;3],None,WatcherAdmission::M21).unwrap_err(),RunError::InvalidAction,"M24 or M25 scenario only");
    }

    #[test]
    fn m25_ending_matches_m24_until_stage_four(){
        // Design §4.2 check 3.
        for boss in M25DiagnosticBoss::ALL{for keys in masks(){
            let mut old=act3_victory_source(RoomScenario::M24WatcherAct3,boss,keys,None,WatcherAdmission::M21).unwrap();
            let mut new=act3_victory_source(RoomScenario::M25WatcherHeart,boss,keys,None,WatcherAdmission::M21).unwrap();
            let tagged=|r:&Run|{let mut t=r.clone();t.state.room_loop.as_mut().unwrap().scenario=RoomScenario::M25WatcherHeart;t};
            assert_eq!(tagged(&old),new,"{boss:?} {keys:?}: sources equal apart from the scenario tag");
            let total=old.decision().unwrap().reward_total;
            for kind in [RoomChoiceKind::Proceed,RoomChoiceKind::Act3Continue{stage:1},RoomChoiceKind::Act3Continue{stage:2}]{
                let (a,b)=(choose(&mut old,kind.clone()),choose(&mut new,kind.clone()));assert_eq!(a,b);assert_eq!(tagged(&old),new,"{boss:?} {keys:?} after {kind:?}");
            }
            let (a,b)=(choose(&mut old,RoomChoiceKind::Act3Continue{stage:3}),choose(&mut new,RoomChoiceKind::Act3Continue{stage:3}));assert_eq!(a,b);
            let (d,e)=(old.decision().unwrap(),new.decision().unwrap());
            if keys==[true;3]{
                assert_eq!(d.room.as_ref().unwrap().kind,RoomViewKind::Act4Ready);assert!(d.candidates.is_empty()&&d.terminal.is_none());
                let room=e.room.unwrap();assert_eq!(room.kind,RoomViewKind::Act3Ending);
                assert_eq!(room.choices.iter().map(|c|c.kind.clone()).collect::<Vec<_>>(),vec![RoomChoiceKind::Act3Continue{stage:4}]);assert_eq!(e.candidates.len(),1);
            }else{
                assert_eq!(tagged(&old),new,"{boss:?} {keys:?}: stage 4 identical without all keys");
                let (a,b)=(choose(&mut old,RoomChoiceKind::Act3Continue{stage:4}),choose(&mut new,RoomChoiceKind::Act3Continue{stage:4}));
                assert_eq!(a,b);assert_eq!(a.terminal,Some(RunTerminal::Act3NoKeys));assert_eq!(a.reward,0);
                for r in [&old,&new]{let d=r.decision().unwrap();assert_eq!((d.terminal,d.reward_total),(Some(RunTerminal::Act3NoKeys),total));
                    assert_eq!(r.state.history.events.iter().filter(|e|**e==CommittedEvent::Ended(RunTerminal::Act3NoKeys)).count(),1);}
                assert_eq!(tagged(&old),new);
            }
        }}
    }

    // ---- Task M4 ----
    use crate::content::{PotionId,RelicId};
    fn strong()->AuthoredRunStart{M25DiagnosticLoadout::Strong.resources()}
    fn act4_with(start:AuthoredRunStart)->Result<Run,RunError>{act3_victory_source(RoomScenario::M25WatcherHeart,M25DiagnosticBoss::AwakenedOne,[true;3],Some(start),WatcherAdmission::M25)}

    #[test]
    fn m25_supplied_validation_uses_m25_unions(){
        for l in M25DiagnosticLoadout::ALL{assert!(m25_supplied_act4(l).is_ok(),"{l:?}");
            if l!=M25DiagnosticLoadout::Strong{assert!(M21WatcherCombatStart::Supplied(l.resources()).normalized().is_err(),"{l:?} needs the M25 admission");}}
        for f in M25PlayedFixture::ALL{assert!(m25_played_fixture(f).is_ok(),"{f:?}");}
        for b in M25DiagnosticBoss::ALL{assert!(m25_supplied_traversal(b).is_ok(),"{b:?}");}
        let bad=|f:&dyn Fn(&mut AuthoredRunStart)|{let mut s=strong();f(&mut s);s};
        assert_eq!(act4_with(bad(&|s|s.deck[0].id=CardId::Bash)).unwrap_err(),RunError::Unsupported("card outside the M25 permanent union"));
        assert_eq!(act4_with(bad(&|s|s.deck[0].upgrades=2)).unwrap_err(),RunError::Unsupported("card outside the M25 permanent union"),"upgrade above 1");
        assert!(act4_with(bad(&|s|s.relics.push(RelicId::Brimstone))).is_err(),"relic outside the union");
        assert!(act4_with(bad(&|s|s.potions[0]=Some(PotionId::FirePotion))).is_err(),"potion outside the union");
        assert_eq!(act4_with(bad(&|s|s.hp=s.max_hp+1)).unwrap_err(),RunError::InvalidAction,"HP above max HP");
        assert_eq!(act4_with(bad(&|s|{s.max_hp=1000;s.hp=1000;})).unwrap_err(),RunError::InvalidAction,"max HP above 999");
        assert_eq!(boss_source_for(M23DiagnosticBoss::Champ,RoomScenario::M25WatcherHeart,Some(EncounterId::AwakenedOne),Some(bad(&|s|s.deck[0].id=CardId::Bash)),WatcherAdmission::M25).unwrap_err(),
            RunError::Unsupported("card outside the M25 permanent union"),"the traversal path validates under the same admission");
    }

    #[test]
    fn m25_m23_and_m24_diagnostics_unchanged_by_admission_threading(){
        use super::super::m23::boss_source;
        let supplied=resources([CardId::Eruption,CardId::Vigilance,CardId::EmptyFist,CardId::CarveReality].into_iter().flat_map(|id|[(id,true);4]),&[RelicId::PureWater,RelicId::VioletLotus],[None;3]);
        for boss in M23DiagnosticBoss::ALL{for scenario in [RoomScenario::M23WatcherAct2,RoomScenario::M24WatcherAct3]{for act3 in [None,Some(EncounterId::TimeEater)]{for start in [None,Some(supplied.clone())]{
            assert_eq!(boss_source(boss,scenario,act3,start.clone()),boss_source_for(boss,scenario,act3,start,WatcherAdmission::M21),"{boss:?} {scenario:?} {act3:?}");
        }}}}
        // The accepted M24 traversal is the M21-admission source with its v2 loadout.
        use super::super::m24::{M24DiagnosticBoss,m24_supplied_traversal,LOADOUT_V2_DECK,LOADOUT_V2_RELICS,LOADOUT_V2_POTIONS,LOADOUT_V2_HP};
        let mut v2=resources(LOADOUT_V2_DECK.into_iter().map(|id|(id,true)),&LOADOUT_V2_RELICS,LOADOUT_V2_POTIONS);v2.hp=LOADOUT_V2_HP;v2.max_hp=LOADOUT_V2_HP;
        for b in M24DiagnosticBoss::ALL{assert_eq!(m24_supplied_traversal(b),boss_source_for(b.traversal_act2(),RoomScenario::M24WatcherAct3,Some(b.encounter()),Some(v2.clone()),WatcherAdmission::M21));}
        assert!(boss_source_for(M23DiagnosticBoss::Champ,RoomScenario::M24WatcherAct3,None,Some(strong()),WatcherAdmission::M21).is_err(),"the M21 admission rejects the M25-only strong loadout");
    }

    fn seed7()->RunRng{random()}
    fn deck(r:&Run)->Vec<(CardId,bool)>{r.state.deck.iter().map(|c|(c.id,c.upgrades==1)).collect()}

    #[test]
    fn m25_supplied_and_constructed_starts_are_exact_without_actions(){
        for l in M25DiagnosticLoadout::ALL{
            let r=m25_supplied_act4(l).unwrap();let d=r.decision().unwrap();
            assert_eq!((r.state.hp,r.state.max_hp,r.state.gold,r.state.keys,d.act,d.floor),(l.hp(),l.hp(),99,[true;3],3,50),"{l:?}");
            assert_eq!(deck(&r),l.deck().to_vec());assert_eq!(r.state.deck.iter().map(|c|c.instance).collect::<Vec<_>>(),(1..=l.deck().len() as u64).collect::<Vec<_>>());
            assert_eq!(r.state.relics,l.relics().to_vec());assert_eq!(r.state.potions,l.potions().to_vec());
            assert_eq!(d.room.unwrap().kind,RoomViewKind::Act3BossComplete);assert_eq!(r.state.history.events.len(),1,"zero accepted actions");assert_eq!(r.state.rng,seed7());
            assert_eq!(l.act3().encounter(),r.state.last_combat.as_ref().unwrap().encounter);
        }
        assert_eq!(M25DiagnosticLoadout::CoverageA.deck().len()+M25DiagnosticLoadout::CoverageB.deck().len()+M25DiagnosticLoadout::CoverageC.deck().len(),57);
        for b in M25DiagnosticBoss::ALL{
            let r=m25_supplied_traversal(b).unwrap();let d=r.decision().unwrap();
            assert_eq!((d.act,d.floor,r.state.keys,r.state.hp,r.state.max_hp),(2,33,[false;3],200,200),"{b:?}");assert_eq!(deck(&r),vec![(CardId::PressurePoints,true);10]);
            assert_eq!(d.room.unwrap().choices.iter().map(|c|c.kind.clone()).collect::<Vec<_>>(),vec![RoomChoiceKind::CardReward,RoomChoiceKind::Proceed]);
            assert_eq!(r.state.seen_bosses,super::super::m23::creation(b.traversal_act2().encounter(),Some(b.encounter())).seen_bosses,"{b:?} Act3 history");
        }
        for f in M25PlayedFixture::ALL{
            let mut r=m25_played_fixture(f).unwrap();let d=r.decision().unwrap();
            assert_eq!((d.act,r.state.hp,r.state.max_hp,r.state.gold,r.state.keys,r.state.potions.clone()),(4,72,72,99,[true;3],vec![None;3]),"{f:?}");
            assert_eq!(deck(&r),f.deck().to_vec());assert_eq!(r.state.relics,f.relics().to_vec());
            let moves=d.candidates.iter().filter_map(|c|if let GameAction::Move(id)=c.action{Some(id)}else{None}).collect::<Vec<_>>();assert_eq!(moves.len(),1,"{f:?}: one Move");
            let floor=r.state.map.as_ref().unwrap().nodes.iter().find(|n|n.id==moves[0]).unwrap().floor;assert_eq!(floor,if f.encounter()==EncounterId::ShieldAndSpear{53}else{54});
            r.step(&GameAction::Move(moves[0])).unwrap();assert_eq!(r.state.active_combat.as_ref().unwrap().encounter,f.encounter(),"{f:?}: the intended encounter");
        }
    }

    #[test]
    fn m25_constructed_stages_reach_actual_owners(){
        for boss in M25DiagnosticBoss::ALL{for keys in masks(){
            let r=m25_diagnostic(M25Diagnostic::DoorSource{boss,keys}).unwrap();assert_eq!((r.decision().unwrap().room.unwrap().kind,r.state.keys),(RoomViewKind::Act3BossComplete,keys));
        }}
        let r=m25_diagnostic(M25Diagnostic::DoorFault).unwrap();assert_eq!(r.state.fault.as_ref().unwrap().checkpoint,"run.m25.act4.transition");
        let r=m25_diagnostic(M25Diagnostic::Act4Room{floor:51}).unwrap();assert_eq!(r.decision().unwrap().room.unwrap().kind,RoomViewKind::Campfire);
        for (floor,encounter) in [(53,EncounterId::ShieldAndSpear),(54,EncounterId::CorruptHeart)]{
            let r=m25_diagnostic(M25Diagnostic::Act4Room{floor}).unwrap();assert_eq!(r.state.active_combat.as_ref().unwrap().encounter,encounter);assert_eq!(r.state.floor,floor);
        }
        for floor in [50,55]{assert_eq!(m25_diagnostic(M25Diagnostic::Act4Room{floor}).unwrap_err(),RunError::InvalidAction);}
        let r=m25_diagnostic(M25Diagnostic::HeartVictoryPending).unwrap();let d=r.decision().unwrap();
        assert_eq!((d.room.as_ref().unwrap().kind,d.terminal),(RoomViewKind::HeartVictory,None));assert_eq!(d.room.unwrap().choices.iter().map(|c|c.kind.clone()).collect::<Vec<_>>(),vec![RoomChoiceKind::Proceed]);
        let r=m25_diagnostic(M25Diagnostic::HeartProceedFault).unwrap();assert_eq!(r.state.fault.as_ref().unwrap().checkpoint,"run.m25.heart.victory");
    }

    #[test]
    fn m25_constructed_card_stages_reach_actual_owners(){
        // Needs the content delivery (HC1): card behaviour and the Act4 merchant sampler.
        let r=m25_diagnostic(M25Diagnostic::Act4Room{floor:52}).unwrap();assert_eq!(r.decision().unwrap().room.unwrap().kind,RoomViewKind::Shop);
        for stage in [M25Diagnostic::ForeignInfluenceOffer,M25Diagnostic::WishChoice]{
            let r=m25_diagnostic(stage).unwrap();let d=r.decision().unwrap();
            assert!(d.candidates.iter().any(|c|matches!(c.action,GameAction::Combat(CombatAction::ChooseCardOffer{..}))),"{stage:?} opens a card offer");
        }
        let r=m25_diagnostic(M25Diagnostic::OmniscienceChoice).unwrap();let d=r.decision().unwrap();
        assert!(d.candidates.iter().any(|c|matches!(c.action,GameAction::Combat(CombatAction::SelectCard{..}))),"Omniscience opens a draw-pile selection");
    }

    #[test]
    fn m25_omniscience_choosing_omniscience_chains_through_public_actions(){
        // Shared autoplay census must count Omniscience continuations while a nested choice is open.
        let mut r=m25_diagnostic(M25Diagnostic::OmniscienceChoice).unwrap();let mut nested=0;
        for _ in 0..64{
            let d=r.decision().unwrap();let c=d.combat.as_ref().unwrap();
            if c.selection.is_none(){
                assert!(d.candidates.iter().any(|x|matches!(x.action,GameAction::Combat(CombatAction::PlayCard{..}|CombatAction::EndTurn)))||c.outcome.is_some());
                assert!(nested>=1,"the selected Omniscience opened its own draw-pile choice");
                assert!(c.exhaust.iter().filter(|card|card.id==CardId::Omniscience).count()>=2,"both resolved Omniscience copies exhaust");
                assert!(r.state.fault.is_none());return;
            }
            let action=d.candidates.iter().find(|x|matches!(x.action,GameAction::Combat(CombatAction::SelectCard{..}))).or_else(||d.candidates.iter().find(|x|x.action==GameAction::Combat(CombatAction::ConfirmSelection))).unwrap().action.clone();
            r.step(&action).unwrap_or_else(|e|panic!("offered {action:?} rejected: {e:?}"));
            if r.decision().unwrap().combat.as_ref().is_some_and(|c|c.selection.is_some()){nested+=1;}
        }
        panic!("Omniscience chain did not return to play");
    }
}
