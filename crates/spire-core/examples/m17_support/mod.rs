//! Frozen public-only M17 caller. No simulator rules or hidden state inspection.
use spire_core::{combat::{CombatAction,TargetId,rng::Partition},content::{CardId,Character,EncounterId,EventId,RelicId},run::{*,map::PublicMap}};
#[path="../shared/public_routes.rs"]
mod public_routes;
pub const CALLER_POLICY_REVISION:&str="m17-public-caller-v1";
pub const ACTION_LIMIT_MAX:usize=10000;
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Scenario{Guardian,Hexaghost,SlimeBoss,Death}
impl Scenario{
    pub fn encounter(self)->EncounterId{match self{Self::Guardian|Self::Death=>EncounterId::Guardian,Self::Hexaghost=>EncounterId::Hexaghost,Self::SlimeBoss=>EncounterId::SlimeBoss}}
    pub fn label(self)->&'static str{match self{Self::Guardian=>"guardian",Self::Hexaghost=>"hexaghost",Self::SlimeBoss=>"slime-boss",Self::Death=>"death"}}
}
pub const FIXED_CASES:[(u64,Scenario);6]=[(7,Scenario::Guardian),(7,Scenario::Hexaghost),(7,Scenario::SlimeBoss),(11,Scenario::Guardian),(11,Scenario::Hexaghost),(11,Scenario::SlimeBoss)];
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum CallerOutcome{Act2Ready,Death,EmptyDeck,BudgetExhausted}
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct TraceStep{pub decision:PublicDecision,pub map:Option<PublicMap>,pub action:GameAction,pub transition:RunTransition}
#[derive(Clone,Debug)]
pub struct Caller{pub trace:Vec<TraceStep>,pub final_decision:Option<PublicDecision>,pub final_map:Option<PublicMap>,limit:usize,ordinary_reached:bool,elite_reached:bool,potion_visit:Option<u64>}
impl Caller{
    pub fn new(limit:usize)->Self{Self{trace:vec![],final_decision:None,final_map:None,limit,ordinary_reached:false,elite_reached:false,potion_visit:None}}
    pub fn accepted(&self)->usize{self.trace.len()}
}
pub fn make_run(seed:u64,scenario:Scenario)->Result<Run,String>{
    if ![7,11].contains(&seed){return Err("seed must be 7 or 11".into())}
    let mut root=[0;32];root[..8].copy_from_slice(&seed.to_le_bytes());let mut p=CreationProfile::experienced();p.neow_eligible=false;
    p.seen_bosses[0]=match scenario.encounter(){EncounterId::Guardian=>vec!["Hexaghost","SlimeBoss"],EncounterId::Hexaghost=>vec!["Guardian","SlimeBoss"],EncounterId::SlimeBoss=>vec!["Guardian","Hexaghost"],_=>unreachable!()};
    Run::new_room_scenario(RunConfig{character:Character::Defect,ascension:0},p,RunRng::new(root,Partition::Test,0),RoomScenario::M17DefectAct1).map_err(|e|format!("{e:?}"))
}
pub fn rollout(run:&mut Run,caller:&mut Caller,scenario:Scenario)->Result<CallerOutcome,String>{
    if caller.limit>ACTION_LIMIT_MAX{return Err("budget exceeds 10000".into())}
    loop{
        let d=run.decision().map_err(|e|format!("{e:?}"))?;let map=run.public_map();caller.final_decision=Some(d.clone());caller.final_map=map.clone();
        match d.terminal{Some(RunTerminal::Death)=>return Ok(CallerOutcome::Death),Some(RunTerminal::EmptyDeck)=>return Ok(CallerOutcome::EmptyDeck),Some(t)=>return Err(format!("unexpected terminal {t:?}")),None=>()}
        if d.room.as_ref().is_some_and(|r|r.kind==RoomViewKind::Act2Ready){return Ok(CallerOutcome::Act2Ready)}
        if caller.accepted()>=caller.limit{return Ok(CallerOutcome::BudgetExhausted)}
        let action=if scenario==Scenario::Death&&d.combat.is_some(){GameAction::Combat(CombatAction::EndTurn)}else{action_for(&d,map.as_ref(),caller)?};
        if !d.candidates.iter().any(|c|c.action==action){return Err(format!("policy proposed illegal action {action:?}"))}
        let transition=run.step(&action).map_err(|e|format!("action {action:?}: {e:?}"))?;
        // Flags are learned only from this accepted public transition and prior map.
        if let GameAction::Move(id)=action{
            let captured_combat=transition.committed_events.iter().any(|e|matches!(e,CommittedEvent::CombatEntered{..}));
            if captured_combat{if let Some(n)=map.as_ref().and_then(|m|m.nodes.iter().find(|n|n.id==id)){
                if n.kind==MapNodeKind::Elite{caller.elite_reached=true}else if n.kind!=MapNodeKind::Boss{caller.ordinary_reached=true}
            }}
        }
        if matches!(action,GameAction::Combat(CombatAction::UsePotion{..})){caller.potion_visit=Some(d.combat_progress.entered);}
        caller.trace.push(TraceStep{decision:d,map,action,transition});
    }
}
// The approved main priority also orders card acquisitions; no separate reward
// list was specified. Candidate order settles equal kinds and all other ties.
const PRIORITY:&[CardId]=&[
    CardId::Electrodynamics,CardId::Defragment,CardId::BiasedCognition,CardId::Capacitor,CardId::Consume,CardId::Loop,CardId::Storm,CardId::StaticDischarge,CardId::Heatsinks,CardId::GeneticAlgorithm,CardId::Amplify,CardId::Fission,CardId::Fusion,CardId::Darkness,CardId::Dualcast,CardId::Recursion,CardId::BallLightning,CardId::ColdSnap,CardId::Coolheaded,CardId::CompileDriver,CardId::Barrage,CardId::Bullseye,CardId::Claw,CardId::SteamBarrier,CardId::Streamline,CardId::Zap,CardId::StrikeBlue,CardId::DefendBlue,
];
fn choice(d:&PublicDecision,wanted:impl Fn(&RoomChoiceKind)->bool)->Option<GameAction>{
    let r=d.room.as_ref()?.choices.iter().find(|c|c.status==RoomChoiceStatus::Offered&&wanted(&c.kind))?;
    d.candidates.iter().find(|c|c.action==GameAction::Choose(r.id)).map(|c|c.action.clone())
}
fn card_choice(d:&PublicDecision)->Option<GameAction>{PRIORITY.iter().find_map(|wanted|choice(d,|k|matches!(k,RoomChoiceKind::Card{id,..} if id==wanted)))}
fn selection(d:&PublicDecision)->Result<GameAction,String>{
    let upgrade=d.room.as_ref().is_some_and(|r|matches!(r.kind,RoomViewKind::Campfire|RoomViewKind::Event(EventId::UpgradeShrine)));
    let priority=if upgrade{[CardId::Zap,CardId::Dualcast]}else{[CardId::StrikeBlue,CardId::DefendBlue]};
    for id in priority{if let Some(c)=d.candidates.iter().find(|c|matches!(c.action,GameAction::Select(_))&&c.object.as_ref().is_some_and(|o|o.id==id)){return Ok(c.action.clone())}}
    d.candidates.iter().find(|c|matches!(c.action,GameAction::Select(_))).map(|c|c.action.clone()).ok_or("no public selection candidate".into())
}
fn action_for(d:&PublicDecision,map:Option<&PublicMap>,caller:&Caller)->Result<GameAction,String>{
    if d.combat.is_some(){return combat_action(d,caller)}
    if d.selection.is_some(){return selection(d)}
    let room=d.room.as_ref().ok_or("missing room")?;
    let chosen=match room.kind{
        RoomViewKind::Map=>return public_routes::action(d,map.ok_or("missing public map")?,caller.elite_reached,caller.ordinary_reached),
        RoomViewKind::Neow=>choice(d,|k|*k==RoomChoiceKind::NeowLament),
        RoomViewKind::Rewards=>choice(d,|k|matches!(k,RoomChoiceKind::Relic(id) if !d.player.relics.contains(id)))
            .or_else(||choice(d,|k|matches!(k,RoomChoiceKind::Gold(_))))
            .or_else(||choice(d,|k|*k==RoomChoiceKind::CardReward))
            .or_else(||choice(d,|k|matches!(k,RoomChoiceKind::Potion(_))))
            .or_else(||choice(d,|k|*k==RoomChoiceKind::Proceed)),
        RoomViewKind::CardReward=>card_choice(d).or_else(||choice(d,|k|*k==RoomChoiceKind::Skip)),
        RoomViewKind::Shop=>choice(d,|k|matches!(k,RoomChoiceKind::Relic(id) if !d.player.relics.contains(id)))
            .or_else(||card_choice(d)).or_else(||choice(d,|k|matches!(k,RoomChoiceKind::Potion(_))))
            .or_else(||choice(d,|k|*k==RoomChoiceKind::Leave)),
        RoomViewKind::Campfire=>{
            let rest=d.player.max_hp.saturating_sub(d.player.hp)>=15;
            choice(d,|k|if rest{matches!(k,RoomChoiceKind::Rest{..})}else{*k==RoomChoiceKind::Smith})
                .or_else(||choice(d,|k|if rest{*k==RoomChoiceKind::Smith}else{matches!(k,RoomChoiceKind::Rest{..})}))
        },
        RoomViewKind::Event(EventId::UpgradeShrine)=>choice(d,|k|*k==RoomChoiceKind::ShrineUpgrade).or_else(||choice(d,|k|*k==RoomChoiceKind::Leave)),
        RoomViewKind::Event(EventId::Purifier)=>choice(d,|k|*k==RoomChoiceKind::ShrineRemove).or_else(||choice(d,|k|*k==RoomChoiceKind::Leave)),
        RoomViewKind::Treasure|RoomViewKind::BossTreasure=>choice(d,|k|*k==RoomChoiceKind::OpenChest)
            .or_else(||choice(d,|k|matches!(k,RoomChoiceKind::Relic(id) if !d.player.relics.contains(id))))
            .or_else(||choice(d,|k|*k==RoomChoiceKind::Proceed)),
        RoomViewKind::BossRelicChoice=>choice(d,|k|*k==RoomChoiceKind::Relic(RelicId::SlaversCollar))
            .or_else(||choice(d,|k|*k==RoomChoiceKind::Relic(RelicId::EmptyCage)))
            .or_else(||choice(d,|k|*k==RoomChoiceKind::Skip)),
        RoomViewKind::BossRelicChild=>choice(d,|k|*k==RoomChoiceKind::Proceed),
        RoomViewKind::ProceedWithoutRelic=>choice(d,|k|*k==RoomChoiceKind::ConfirmSkip),
        _=>None,
    };
    chosen.or_else(||d.candidates.first().map(|c|c.action.clone())).ok_or_else(||format!("no offered action in {:?}",room.kind))
}
fn combat_action(d:&PublicDecision,caller:&Caller)->Result<GameAction,String>{
    let c=d.combat.as_ref().ok_or("missing combat")?;
    if c.selection.is_some(){
        if let Some(a)=d.candidates.iter().find(|a|a.action==GameAction::Combat(CombatAction::ConfirmSelection)){return Ok(a.action.clone())}
        return d.candidates.iter().find(|a|matches!(a.action,GameAction::Combat(CombatAction::SelectCard{..}))).map(|a|a.action.clone()).ok_or("no mandatory selection".into())
    }
    if let Some(a)=d.candidates.iter().find(|a|a.action==GameAction::Combat(CombatAction::ConfirmSelection)){return Ok(a.action.clone())}
    let target_hp=|target:Option<TargetId>|target.and_then(|t|c.enemies.iter().enumerate().find(|(i,_)|*i as u64+1==t.0).map(|(_,e)|e.hp)).unwrap_or(i32::MIN);
    if caller.potion_visit!=Some(d.combat_progress.entered){
        let first_slot=d.candidates.iter().find_map(|a|if let GameAction::Combat(CombatAction::UsePotion{slot,..})=a.action{Some(slot)}else{None});
        if let Some(a)=d.candidates.iter().filter(|a|matches!(a.action,GameAction::Combat(CombatAction::UsePotion{slot,..}) if Some(slot)==first_slot))
            .min_by_key(|a|match a.action{GameAction::Combat(CombatAction::UsePotion{target,..})=>target_hp(target),_=>unreachable!()}){return Ok(a.action.clone())}
    }
    let play=|id:CardId|d.candidates.iter().filter(|a|matches!(a.action,GameAction::Combat(CombatAction::PlayCard{instance,..}) if c.hand.iter().any(|card|card.instance.0==instance&&card.id==id)))
        .min_by_key(|a|match a.action{GameAction::Combat(CombatAction::PlayCard{target,..})=>target_hp(target),_=>unreachable!()}).map(|a|a.action.clone());
    let incoming=c.enemies.iter().filter_map(|e|e.intent_damage).sum::<i32>();
    if incoming>c.player.block{for id in [CardId::DefendBlue,CardId::SteamBarrier,CardId::GeneticAlgorithm,CardId::ColdSnap,CardId::Coolheaded]{if let Some(a)=play(id){return Ok(a)}}}
    for &id in PRIORITY{if let Some(a)=play(id){return Ok(a)}}
    d.candidates.iter().find(|a|matches!(a.action,GameAction::Combat(CombatAction::PlayCard{..})))
        .or_else(||d.candidates.iter().find(|a|a.action==GameAction::Combat(CombatAction::EndTurn)))
        .map(|a|a.action.clone()).ok_or("no legal combat action".into())
}
#[derive(Debug)]
pub struct Options{pub seed:u64,pub scenarios:Vec<Scenario>,pub limit:usize}
pub fn parse_args(args:impl IntoIterator<Item=String>)->Result<Options,String>{
    let mut seed=7;let mut scenario="all".to_string();let mut limit=ACTION_LIMIT_MAX;let mut it=args.into_iter();
    while let Some(arg)=it.next(){let value=it.next().ok_or("missing argument value")?;match arg.as_str(){"--seed"=>seed=value.parse().map_err(|_|"invalid seed")?,"--scenario"=>scenario=value,"--budget"=>limit=value.parse().map_err(|_|"invalid budget")?,_=>return Err(format!("unknown argument {arg}"))}}
    if ![7,11].contains(&seed){return Err("seed must be 7 or 11".into())}if limit>ACTION_LIMIT_MAX{return Err("budget exceeds 10000".into())}
    let scenarios=match scenario.as_str(){"all"=>vec![Scenario::Guardian,Scenario::Hexaghost,Scenario::SlimeBoss],"guardian"=>vec![Scenario::Guardian],"hexaghost"=>vec![Scenario::Hexaghost],"slime-boss"=>vec![Scenario::SlimeBoss],"death"=>vec![Scenario::Death],_=>return Err("invalid scenario".into())};Ok(Options{seed,scenarios,limit})
}

/// Frozen action receipt, consumed without policy calls by all verification lanes.
pub fn action_receipt(seed:u64,scenario:Scenario,caller:&Caller,result:&Result<CallerOutcome,String>)->serde_json::Value{
    serde_json::json!({"policy":CALLER_POLICY_REVISION,"seed":seed,"scenario":scenario.label(),"budget":caller.limit,
        "result":format!("{result:?}"),"actions":caller.trace.iter().map(|s|s.decision.candidates.iter().position(|c|c.action==s.action).expect("accepted public candidate")).collect::<Vec<_>>(),
        "rewards":caller.trace.iter().map(|s|s.transition.reward).collect::<Vec<_>>(),
        "transitions":caller.trace.iter().map(|s|format!("{:?}",s.transition)).collect::<Vec<_>>(),
        "decisions":caller.trace.iter().map(|s|format!("{:?}",s.decision)).collect::<Vec<_>>(),
        "maps":caller.trace.iter().map(|s|format!("{:?}",s.map)).collect::<Vec<_>>(),
        "final_decision":format!("{:?}",caller.final_decision),"final_map":format!("{:?}",caller.final_map)})
}
/// Reconstruct the recorded public trace from candidate indices, never another policy.
pub fn replay_receipt(value:&serde_json::Value)->Result<(Run,Caller),String>{
    let seed=value["seed"].as_u64().ok_or("receipt seed")?;
    let scenario=match value["scenario"].as_str().ok_or("receipt scenario")?{"guardian"=>Scenario::Guardian,"hexaghost"=>Scenario::Hexaghost,"slime-boss"=>Scenario::SlimeBoss,"death"=>Scenario::Death,_=>return Err("receipt scenario".into())};
    let mut run=make_run(seed,scenario)?;let mut caller=Caller::new(ACTION_LIMIT_MAX);
    let actions=value["actions"].as_array().ok_or("receipt actions")?;
    for (i,row) in actions.iter().enumerate(){
        let d=run.decision().map_err(|e|format!("{e:?}"))?;let map=run.public_map();
        if value["decisions"][i]!=format!("{d:?}")||value["maps"][i]!=format!("{map:?}"){return Err(format!("receipt public mismatch {i}"))}
        let action=d.candidates.get(row.as_u64().ok_or("receipt action")? as usize).ok_or("receipt illegal index")?.action.clone();
        let transition=run.step(&action).map_err(|e|format!("{e:?}"))?;
        if value["transitions"][i]!=format!("{transition:?}"){return Err(format!("receipt transition mismatch {i}"))}
        caller.trace.push(TraceStep{decision:d,map,action,transition});
    }
    caller.final_decision=Some(run.decision().map_err(|e|format!("{e:?}"))?);caller.final_map=run.public_map();
    if value["final_decision"]!=format!("{:?}",caller.final_decision)||value["final_map"]!=format!("{:?}",caller.final_map){return Err("receipt ending mismatch".into())}
    Ok((run,caller))
}
