//! M25 Watcher A0 Acts 1–4 and Heart on the shared Run owners. Supplied map positions,
//! visit counters and Lament HP establish owner behavior only; they are never a
//! canonical traversal or normal-completion witness.
use super::super::*;
use crate::{combat::{CombatAction,rng::Partition},content::{Character,EncounterId,EventId,RelicId}};
use super::m17_defect_act1::{choose,diagnostic_room,settle_lament};

fn history(a:EncounterId,b:EncounterId,c:EncounterId)->CreationProfile{
    let mut p=CreationProfile::experienced();p.neow_eligible=false;
    p.seen_bosses[0]=match a{EncounterId::Guardian=>vec!["Hexaghost","SlimeBoss"],EncounterId::Hexaghost=>vec!["Guardian","SlimeBoss"],EncounterId::SlimeBoss=>vec!["Guardian","Hexaghost"],_=>unreachable!()};
    p.seen_bosses[1]=match b{EncounterId::Champ=>vec!["Automaton","Collector"],EncounterId::BronzeAutomaton=>vec!["Champ","Collector"],EncounterId::Collector=>vec!["Champ","Automaton"],_=>unreachable!()};
    p.seen_bosses[2]=match c{EncounterId::AwakenedOne=>vec!["TimeEater","DonuAndDeca"],EncounterId::TimeEater=>vec!["AwakenedOne","DonuAndDeca"],EncounterId::DonuAndDeca=>vec!["AwakenedOne","TimeEater"],_=>unreachable!()};p
}
fn watcher(p:CreationProfile,root:[u8;32],scenario:RoomScenario)->Result<Run,RunError>{Run::new_room_scenario(RunConfig{character:Character::Watcher,ascension:0},p,RunRng::new(root,Partition::Test,0),scenario)}
/// Matched inputs: identical creation history, root, partition and ordinal.
fn pair(c:EncounterId,root:[u8;32])->(Run,Run){
    let p=history(EncounterId::Guardian,EncounterId::Champ,c);(watcher(p.clone(),root,RoomScenario::M24WatcherAct3).unwrap(),watcher(p,root,RoomScenario::M25WatcherHeart).unwrap())
}
/// Allowed differences before the Act4 door (design §4.2): the scenario tag and, inside an
/// active combat, the item-context profile flag (`watcher_act3` against `watcher_heart`).
/// The combat is compared through its full `Debug` text with exactly that flag normalized, and the public
/// decision with the M25-only `previous_card_type` checked against its owner and then masked.
fn assert_prefix(old:&Run,new:&Run,context:&str){
    let (mut a,mut b)=(old.clone(),new.clone());a.state.room_loop.as_mut().unwrap().scenario=RoomScenario::M25WatcherHeart;
    let combat=|r:&mut Run|r.state.combat.take().map(|c|format!("{c:?}"));
    let (old_combat,new_combat)=(combat(&mut a),combat(&mut b));
    assert_eq!(a,b,"{context}");
    assert_eq!(old_combat.map(|s|s.replace("watcher_act3: true","watcher_heart: true")),new_combat,"{context}: combat with the profile flag normalized");
    match (old.decision(),new.decision()){
        (Ok(mut d),Ok(mut e))=>{assert_eq!((d.content_profile,e.content_profile),(Some(RoomScenario::M24WatcherAct3),Some(RoomScenario::M25WatcherHeart)));d.content_profile=e.content_profile;
            // M25 additionally publishes the previous card type (its own owner); M24 never does.
            if let Some(view)=e.combat.as_mut(){assert_eq!(view.previous_card_type,new.state.combat.as_ref().and_then(|c|c.previous_card_type()),"{context}: M25 previous card type");view.previous_card_type=None;}
            assert_eq!(d,e,"{context}: public decision")}
        (d,e)=>assert_eq!(d,e,"{context}: decision error"),
    }
}
fn rows(r:&Run)->Vec<(RoomChoiceKind,Option<u32>,RoomChoiceStatus)>{r.decision().unwrap().room.unwrap().choices.iter().map(|c|(c.kind.clone(),c.price,c.status)).collect()}
fn room_kind(r:&Run)->Option<RoomViewKind>{r.decision().ok().and_then(|d|d.room).map(|room|room.kind)}
/// Supplied Lament path through a real boss source and its transition.
fn advance(r:&mut Run){
    diagnostic_room(r,MapNodeKind::Boss);settle_lament(r);
    choose(r,RoomChoiceKind::Proceed);choose(r,RoomChoiceKind::Proceed);choose(r,RoomChoiceKind::ConfirmSkip);
}
/// Both profiles driven identically to the start of `act` (1, 2 or 3) on the Lament path.
fn at_act(c:EncounterId,root:[u8;32],act:u8)->(Run,Run){
    let (mut old,mut new)=pair(c,root);
    for r in [&mut old,&mut new]{
        choose(r,RoomChoiceKind::NeowLament);
        if act>=2{advance(r);}
        if act>=3{r.state.relic_counters.insert(RelicId::NeowsLament,3);advance(r);}
        assert_eq!(r.state.act,act);assert_eq!(room_kind(r),Some(RoomViewKind::Map));
    }
    assert_prefix(&old,&new,&format!("Act{act} start"));(old,new)
}
fn both(old:&mut Run,new:&mut Run,f:impl Fn(&mut Run)){f(old);f(new);}

#[test]
fn m25_rejects_unsupported_creation(){
    for (character,ascension,full) in [(Character::Silent,0,false),(Character::Defect,0,false),(Character::Ironclad,0,false),(Character::Watcher,1,false),(Character::Watcher,0,true)]{
        let mut p=history(EncounterId::Guardian,EncounterId::Champ,EncounterId::AwakenedOne);p.neow_eligible=full;
        assert_eq!(Run::new_room_scenario(RunConfig{character,ascension},p,RunRng::new([7;32],Partition::Test,0),RoomScenario::M25WatcherHeart),Err(RunError::Unsupported("M25 requires Watcher A0 short Neow")),"{character:?} A{ascension} full Neow {full}");
    }
    for unlocked in [false,true]{
        let mut p=history(EncounterId::Guardian,EncounterId::Champ,EncounterId::AwakenedOne);if unlocked{p.act4_enabled=false}else{p.fully_unlocked=false}
        assert_eq!(watcher(p,[7;32],RoomScenario::M25WatcherHeart),Err(RunError::Unsupported("profile must be fully unlocked with Act 4 enabled")),"existing Run::new owner, same as M22–M24");
    }
    // Canonical resources from the existing Run::new/Catalog path (design §4.1).
    let r=watcher(history(EncounterId::Guardian,EncounterId::Champ,EncounterId::AwakenedOne),[7;32],RoomScenario::M25WatcherHeart).unwrap();
    assert_eq!((r.state.hp,r.state.max_hp,r.state.gold,r.state.act4_enabled,r.state.keys),(72,72,99,true,[false;3]));
    assert_eq!(r.state.relics,vec![RelicId::PureWater]);assert_eq!(r.state.potions,vec![None;3]);
    let mut deck=r.state.deck.iter().map(|c|(c.id,c.upgrades)).collect::<Vec<_>>();deck.sort_by_key(|(id,_)|format!("{id:?}"));
    use crate::content::CardId::{DefendPurple,Eruption,StrikePurple,Vigilance};
    assert_eq!(deck,vec![(DefendPurple,0),(DefendPurple,0),(DefendPurple,0),(DefendPurple,0),(Eruption,0),(StrikePurple,0),(StrikePurple,0),(StrikePurple,0),(StrikePurple,0),(Vigilance,0)]);
    let mut instances=r.state.deck.iter().map(|c|c.instance).collect::<Vec<_>>();instances.sort_unstable();instances.dedup();assert_eq!(instances.len(),10,"ten distinct permanent instances");
    assert_eq!(r.decision().unwrap().content_profile,Some(RoomScenario::M25WatcherHeart));
}

/// The same fixed test rule as M24's played-prefix test (not a policy under evaluation):
/// Lament at Neow, confirm any selection, otherwise the first stance/selection or legal card
/// then EndTurn, and never a potion.
fn prefix_action(d:&PublicDecision)->GameAction{
    let has=|a:&GameAction|d.candidates.iter().any(|c|&c.action==a);
    let confirm=GameAction::Combat(CombatAction::ConfirmSelection);if has(&confirm){return confirm}
    if d.combat.is_some(){
        let combat=|f:&dyn Fn(&CombatAction)->bool|d.candidates.iter().find(|c|matches!(&c.action,GameAction::Combat(a) if f(a))).map(|c|c.action.clone());
        return combat(&|a|matches!(a,CombatAction::ChooseStance{..}|CombatAction::SelectCard{..})).or_else(||combat(&|a|matches!(a,CombatAction::PlayCard{..})))
            .unwrap_or(GameAction::Combat(CombatAction::EndTurn))
    }
    if let Some(room)=d.room.as_ref().filter(|r|r.kind==RoomViewKind::Neow){
        if let Some(row)=room.choices.iter().find(|c|c.kind==RoomChoiceKind::NeowLament){return GameAction::Choose(row.id)}
    }
    d.candidates.iter().find(|c|!matches!(c.action,GameAction::UsePotion{..}|GameAction::DiscardPotion{..}|GameAction::Combat(_))).or_else(||d.candidates.first()).map(|c|c.action.clone()).expect("M24 published a legal candidate")
}

#[test]
fn m25_start_and_prefix_match_m24_played(){
    // Design §4.2 check 1: matched history, played rooms; each action is taken from M24's
    // own legal candidates by one fixed rule and applied identically to M25.
    for c in [EncounterId::AwakenedOne,EncounterId::TimeEater,EncounterId::DonuAndDeca]{
        let (mut old,mut new)=pair(c,[7;32]);assert_prefix(&old,&new,"initial");
        let mut steps=0;let mut door=false;
        while steps<4096{
            let d=old.decision().unwrap();
            if d.terminal.is_some()||d.room.as_ref().is_some_and(|r|r.kind==RoomViewKind::Act4Ready){break}
            let action=prefix_action(&d);
            let a=old.step(&action);let b=new.step(&action);assert_eq!(a,b,"{c:?} step {steps}: {action:?}");steps+=1;
            if room_kind(&old)==Some(RoomViewKind::Act4Ready){
                // The first gameplay-facing difference: M25 shows the fourth ending stage.
                let screen=new.decision().unwrap().room.unwrap();assert_eq!(screen.kind,RoomViewKind::Act3Ending);
                assert_eq!(screen.choices.iter().map(|c|c.kind.clone()).collect::<Vec<_>>(),vec![RoomChoiceKind::Act3Continue{stage:4}]);door=true;break
            }
            assert_prefix(&old,&new,&format!("{c:?} after step {steps}: {action:?}"));
        }
        eprintln!("m25 played prefix vs M24 ({c:?}): {steps} matched steps, endpoint act {} floor {} room {:?} terminal {:?} door {door}",old.state.act,old.state.floor,room_kind(&old),old.state.terminal);
        assert!(steps>0);
    }
}

fn question(r:&mut Run,visits:usize){r.state.room_loop.as_mut().unwrap().act3.as_mut().unwrap().question_visits=visits;diagnostic_room(r,MapNodeKind::Question);}

#[test]
fn m25_act3_owner_rows_match_m24(){
    // Design §4.2 check 2: equal constructed Act3 states, real room entries in both profiles.
    let scope=|r:&Run|rooms::m23_content::relic_capture_scope(&r.state);
    let (old,new)=at_act(EncounterId::AwakenedOne,[19;32],2);
    assert_eq!((scope(&old),scope(&new)),(RunRngScope::M23RelicCapture,RunRngScope::M23RelicCapture),"Act2 scope run.m23.relic.capture in both");
    let (mut old,mut new)=at_act(EncounterId::AwakenedOne,[19;32],3);
    assert_eq!((scope(&old),scope(&new)),(RunRngScope::M24RelicCapture,RunRngScope::M24RelicCapture),"Act3 scope run.m24.relic.capture in both");
    both(&mut old,&mut new,|r|{r.state.gold=999;diagnostic_room(r,MapNodeKind::Shop);});
    assert_prefix(&old,&new,"first Act3 merchant (counter 0)");assert_eq!(new.state.room_loop.as_ref().unwrap().act3.as_ref().unwrap().shop_visits,1);
    both(&mut old,&mut new,|r|{choose(r,RoomChoiceKind::Leave);diagnostic_room(r,MapNodeKind::Shop);});
    assert_prefix(&old,&new,"later Act3 merchant (counter 1)");assert_eq!(rows(&old),rows(&new));
    both(&mut old,&mut new,|r|{choose(r,RoomChoiceKind::Leave);r.state.combat_progress=CombatProgress{entered:9,settled:9};diagnostic_room(r,MapNodeKind::Elite);});
    assert_prefix(&old,&new,"Act3 elite entry");
    both(&mut old,&mut new,settle_lament);
    assert_prefix(&old,&new,"Act3 elite reward");assert_eq!(old.state.rng,new.state.rng);
    let relics=|r:&Run|rows(r).into_iter().filter(|(k,_,_)|matches!(k,RoomChoiceKind::Relic(_))).collect::<Vec<_>>();
    assert_eq!(relics(&new).len(),1,"Act3 elite captured one relic");assert_eq!(relics(&old),relics(&new));
    // Floor-43 chest.
    let (mut old,mut new)=at_act(EncounterId::AwakenedOne,[19;32],3);
    both(&mut old,&mut new,|r|diagnostic_room(r,MapNodeKind::Treasure));assert_eq!(new.state.floor,43);assert_prefix(&old,&new,"floor-43 chest");
    both(&mut old,&mut new,|r|{choose(r,RoomChoiceKind::OpenChest);});assert_prefix(&old,&new,"floor-43 chest opened");assert_eq!(rows(&old),rows(&new));
    // Falling: the captured instance and type rows.
    let (mut old,mut new)=at_act(EncounterId::AwakenedOne,[19;32],3);
    both(&mut old,&mut new,|r|question(r,0));assert_eq!(room_kind(&new),Some(RoomViewKind::Event(EventId::Falling)));assert_prefix(&old,&new,"Falling");
    let lose=rows(&new).into_iter().find_map(|(k,_,s)|(matches!(k,RoomChoiceKind::FallingLose{..})&&s==RoomChoiceStatus::Offered).then_some(k)).unwrap();
    both(&mut old,&mut new,|r|{choose(r,lose.clone());});assert_prefix(&old,&new,"Falling resolved");
    // Tomb of Lord Red Mask.
    let (mut old,mut new)=at_act(EncounterId::AwakenedOne,[19;32],3);
    both(&mut old,&mut new,|r|{r.state.gold=123;question(r,1);});assert_eq!(room_kind(&new),Some(RoomViewKind::Event(EventId::TombOfLordRedMask)));assert_prefix(&old,&new,"Tomb");
    both(&mut old,&mut new,|r|{choose(r,RoomChoiceKind::TombOfferGold);});assert_prefix(&old,&new,"Tomb offer");assert!(new.state.relics.contains(&RelicId::RedMask));
}

#[test]
fn m25_pre_door_act4_sites_are_inert(){
    // Design §4.3: M25 joins uses_act4; before the door each site is inert for Watcher.
    for act in 1..=3u8{
        let (mut old,mut new)=at_act(EncounterId::AwakenedOne,[23;32],act);
        both(&mut old,&mut new,|r|{r.state.gold=999;diagnostic_room(r,MapNodeKind::Shop);});
        assert_eq!(old.state.rng,new.state.rng,"Act{act} merchant commits an equal RNG");assert_prefix(&old,&new,&format!("Act{act} merchant"));
    }
    // A Watcher chest uses the Watcher branch (checked before the uses_act4 branch) in both.
    let (mut old,mut new)=at_act(EncounterId::AwakenedOne,[23;32],1);
    both(&mut old,&mut new,|r|{diagnostic_room(r,MapNodeKind::Treasure);choose(r,RoomChoiceKind::OpenChest);});
    assert_prefix(&old,&new,"Act1 Watcher chest");
    // Campfire with Ruby held: public Recall Disabled and rejected in both. The stored,
    // unpublished row status is the only internal difference (M25 stores Disabled).
    for act in [1u8,3]{
        let (mut old,mut new)=at_act(EncounterId::AwakenedOne,[23;32],act);
        both(&mut old,&mut new,|r|{r.state.keys[0]=true;diagnostic_room(r,MapNodeKind::Rest);});
        let recall=|r:&Run|r.state.room_loop.as_ref().unwrap().screen.choices.iter().find(|c|c.kind==RoomChoiceKind::Recall).unwrap().clone();
        assert_eq!((recall(&old).status,recall(&new).status),(RoomChoiceStatus::Offered,RoomChoiceStatus::Disabled),"stored status");
        for r in [&old,&new]{assert_eq!(rows(r).iter().find(|(k,_,_)|*k==RoomChoiceKind::Recall).unwrap().2,RoomChoiceStatus::Disabled,"public Recall Disabled");}
        let id=recall(&new).id;assert_eq!(id,recall(&old).id);
        let (before_old,before_new)=(old.clone(),new.clone());
        assert_eq!(old.step(&GameAction::Choose(id)),Err(RunError::InvalidAction));assert_eq!(new.step(&GameAction::Choose(id)),Err(RunError::InvalidAction));
        assert_eq!((&old,&new),(&before_old,&before_new),"rejection commits nothing");
        let mut normalized=old.clone();normalized.state.room_loop.as_mut().unwrap().screen.choices.iter_mut().find(|c|c.kind==RoomChoiceKind::Recall).unwrap().status=RoomChoiceStatus::Disabled;
        assert_prefix(&normalized,&new,&format!("Act{act} campfire with Ruby, stored Recall status normalized"));
    }
}

#[test]
fn m25_combat_context_selects_m25_profile(){
    let context=|r:&Run|r.state.combat.as_ref().unwrap().item_context().unwrap().clone();
    let (mut old,mut new)=at_act(EncounterId::AwakenedOne,[29;32],1);
    both(&mut old,&mut new,|r|diagnostic_room(r,MapNodeKind::Monster));
    let (m24,m25)=(context(&old),context(&new));
    assert!(m25.m25_profile()&&m25.m22_profile()&&m25.m21_profile(),"M25 combat enters with_m25_profile, built on with_m22_profile");
    assert!(!m24.m25_profile());
    let (m24_text,m25_text)=(format!("{m24:?}"),format!("{m25:?}"));
    assert!(m25_text.contains("watcher_heart: true")&&!m25_text.contains("watcher_act3"),"M24's flag is false under M25: {m25_text}");
    assert!(m24_text.contains("watcher_act3: true")&&!m24_text.contains("watcher_heart"),"M24 context Debug text unchanged: {m24_text}");
    assert_eq!(m24_text.replace("watcher_act3: true","watcher_heart: true"),m25_text,"the flag is the only context difference");
    for scenario in [RoomScenario::M22WatcherAct1,RoomScenario::M23WatcherAct2]{
        let mut r=watcher(history(EncounterId::Guardian,EncounterId::Champ,EncounterId::AwakenedOne),[29;32],scenario).unwrap();
        choose(&mut r,RoomChoiceKind::NeowLament);diagnostic_room(&mut r,MapNodeKind::Monster);
        let c=context(&r);let text=format!("{c:?}");
        assert!(c.m22_profile()&&!c.m25_profile(),"{scenario:?} keeps with_m22_profile");
        assert!(text.contains("watcher_progression: true")&&!text.contains("watcher_act3")&&!text.contains("watcher_heart"),"{scenario:?} Debug unchanged: {text}");
        assert_eq!(m24_text.replace(", watcher_act3: true",""),text,"{scenario:?} context equals M24's without the M24 flag");
    }
}

// ---- Task M2: Act4 door, route, Heart Proceed, keys and labels ----

fn m25(c:EncounterId)->Run{watcher(history(EncounterId::Guardian,EncounterId::Champ,c),[7;32],RoomScenario::M25WatcherHeart).unwrap()}
/// Supplied Lament path through real Act1 and Act2 boss sources and both transitions.
fn m25_act3(c:EncounterId)->Run{
    let mut r=m25(c);choose(&mut r,RoomChoiceKind::NeowLament);advance(&mut r);r.state.relic_counters.insert(RelicId::NeowsLament,3);advance(&mut r);
    assert_eq!((r.state.act,r.state.floor),(3,34));r
}
/// Real floor-50 boss entry, then a supplied lethal frontier (M19/M24 pattern) and settlement.
fn boss_victory(c:EncounterId,keys:[bool;3])->Run{
    let mut r=m25_act3(c);r.state.relics.retain(|id|*id!=RelicId::NeowsLament);r.state.relic_counters.remove(&RelicId::NeowsLament);r.state.keys=keys;
    diagnostic_room(&mut r,MapNodeKind::Boss);assert_eq!(r.state.active_combat.as_ref().unwrap().encounter,c);
    for _ in 0..8{
        let alive=r.state.combat.as_ref().unwrap().public_full().unwrap().enemies.iter().enumerate().filter(|(_,e)|e.hp>0).map(|(i,_)|i).collect::<Vec<_>>();
        for slot in alive{super::m9_boss::hit(&mut r,slot,9999);if r.state.combat.as_ref().unwrap().public_full().unwrap().outcome.is_some(){break}}
        if r.state.combat.as_ref().unwrap().public_full().unwrap().outcome.is_some(){break}
        r.step(&GameAction::Combat(CombatAction::EndTurn)).unwrap();
    }
    r.settle_combat().unwrap();r
}
/// Stage 4 with supplied keys (labelled unearned), one choice before the door.
fn door(c:EncounterId)->Run{
    let mut r=boss_victory(c,[true;3]);choose(&mut r,RoomChoiceKind::Proceed);for stage in 1..=3{choose(&mut r,RoomChoiceKind::Act3Continue{stage});}r
}
fn stage4(r:&Run)->ChoiceId{r.decision().unwrap().room.unwrap().choices.iter().find(|c|c.kind==RoomChoiceKind::Act3Continue{stage:4}).unwrap().id}
fn act4(c:EncounterId)->Run{let mut r=door(c);let id=stage4(&r);r.step(&GameAction::Choose(id)).unwrap();assert_eq!(r.state.act,4);r}
fn route(r:&Run,floor:u32)->NodeId{r.state.map.as_ref().unwrap().nodes.iter().find(|n|n.floor==floor).unwrap().id}
/// Supplied Act4 map position (disclosed): every route node up to `floor` visited, current at `floor`.
fn place(r:&mut Run,floor:u32){
    let map=r.state.map.as_mut().unwrap();for n in &mut map.nodes{if n.floor<=floor{n.visited=true;}}
    map.current=Some(map.nodes.iter().find(|n|n.floor==floor).unwrap().id);r.state.floor=floor;
}
/// Constructed Shield and Spear Victory record with the given reward-screen state.
fn constructed_elite(r:&mut Run,closed:bool){
    let settled=r.state.combat_progress.settled;let record=CombatRecord{visit:settled,encounter:EncounterId::ShieldAndSpear,outcome:crate::combat::Outcome::Victory};
    r.state.combat_progress=CombatProgress{entered:settled+1,settled:settled+1};r.state.history.events.push(CommittedEvent::CombatFinished(record.clone()));
    let elite=route(r,53);r.state.map.as_mut().unwrap().nodes.iter_mut().find(|n|n.id==elite).unwrap().encounter=Some(EncounterId::ShieldAndSpear);
    let p=r.state.room_loop.as_mut().unwrap().act4.as_mut().unwrap();p.elite_record=Some(record);p.combat_visits=1;p.elite_rewards_closed=closed;
}
fn outcome(r:&Run)->Option<crate::combat::Outcome>{r.state.combat.as_ref().unwrap().public_full().unwrap().outcome}
fn kill_heart(r:&mut Run){
    for _ in 0..6{super::m9_boss::hit(r,0,1_000);if outcome(r)==Some(crate::combat::Outcome::Victory){return}r.step(&GameAction::Combat(CombatAction::EndTurn)).unwrap();}
    panic!("supplied lethal frontier did not kill the Heart");
}
fn heart(c:EncounterId)->Run{
    let mut r=act4(c);place(&mut r,53);constructed_elite(&mut r,true);r.state.max_hp=999;r.state.hp=999;
    r.step(&GameAction::Move(route(&r,54))).unwrap();assert_eq!(r.state.active_combat.as_ref().unwrap().encounter,EncounterId::CorruptHeart);r
}

#[test]
fn m25_door_commits_once(){
    for c in [EncounterId::AwakenedOne,EncounterId::TimeEater,EncounterId::DonuAndDeca]{
        let mut r=door(c);r.state.hp=10;let events=r.state.history.events.len();let total=r.decision().unwrap().reward_total;let id=stage4(&r);
        let t=r.step(&GameAction::Choose(id)).unwrap();
        assert_eq!(t,RunTransition{reward:0,terminal:None,committed_events:vec![CommittedEvent::ActAdvanced{from:3,to:4}]},"{c:?}");
        assert_eq!(&r.state.history.events[events..],&[CommittedEvent::ActAdvanced{from:3,to:4}]);assert_eq!(r.decision().unwrap().reward_total,total,"transition reward 0");
        assert_eq!((r.state.act,r.state.hp,r.state.room),(4,r.state.max_hp,"Map"),"healed to max HP once");
        let room=r.state.room_loop.as_ref().unwrap();assert!(room.act1.is_none()&&room.act2.is_none()&&room.act3.is_none()&&room.act3_end.is_none()&&room.act3_event.is_none(),"Act1–3 progress cleared");
        assert_eq!(room.act4.as_ref().unwrap().phase,rooms::act4::Act4TransitionPhase::Published);
        let map=r.state.map.as_ref().unwrap();assert_eq!((map.act,map.boss_icon,map.current),(4,Some(EncounterId::CorruptHeart),None));
        assert_eq!(map.nodes.iter().map(|n|(n.floor,n.kind,n.burning,n.edges.len())).collect::<Vec<_>>(),vec![(51,MapNodeKind::Rest,false,1),(52,MapNodeKind::Shop,false,1),(53,MapNodeKind::Elite,false,1),(54,MapNodeKind::Boss,false,0)],"fixed map, no burning node");
        assert_eq!(r.decision().unwrap().room.unwrap().kind,RoomViewKind::Map);
        let before=r.clone();assert_eq!(r.step(&GameAction::Choose(id)),Err(RunError::InvalidAction),"repeated door choice");assert_eq!(r,before,"changes nothing");
    }
    // The heal goes through the ordinary healing owner: Mark of the Bloom blocks it.
    let mut r=door(EncounterId::AwakenedOne);r.state.hp=10;crate::items::acquire_relic(&mut r.state,RelicId::MarkOfBloom).unwrap();
    let id=stage4(&r);r.step(&GameAction::Choose(id)).unwrap();assert_eq!((r.state.act,r.state.hp),(4,10));
    // A stale stage-4 choice (from an older screen) changes nothing.
    let mut r=door(EncounterId::TimeEater);let id=stage4(&r);let before=r.clone();
    assert_eq!(r.step(&GameAction::Choose(ChoiceId(id.0+1000))),Err(RunError::InvalidAction));assert_eq!(r,before);
    // Forced post-accept failure (constructed: invalid HP between preparation and commit) is
    // retained once under the M25 label; nothing heals and nothing commits twice.
    let prepared=rooms::act4::prepare_transition(&r).unwrap();r.state.hp=r.state.max_hp+1;let hp=r.state.hp;
    rooms::act4::commit_transition(&mut r,prepared);
    let fault=r.state.fault.clone().unwrap();assert_eq!(fault.checkpoint,"run.m25.act4.transition");assert_eq!(r.state.hp,hp,"no heal");
    let before=r.clone();assert_eq!(r.step(&GameAction::Choose(id)),Err(RunError::Faulted(fault.clone())));assert_eq!(r,before,"no second heal or commit");
    assert!(!r.state.history.events.contains(&CommittedEvent::ActAdvanced{from:3,to:4}));
}

#[test]
fn m25_act4_route_and_heart_proceed(){
    let mut r=act4(EncounterId::AwakenedOne);
    // Rest 51: Ruby is held, so Recall is disabled and rejected.
    r.step(&GameAction::Move(route(&r,51))).unwrap();
    let screen=r.decision().unwrap().room.unwrap();assert_eq!(screen.kind,RoomViewKind::Campfire);
    let recall=screen.choices.iter().find(|c|c.kind==RoomChoiceKind::Recall).unwrap();assert_eq!(recall.status,RoomChoiceStatus::Disabled);
    let before=r.clone();assert_eq!(r.step(&GameAction::Choose(recall.id)),Err(RunError::InvalidAction));assert_eq!(r,before);
    let rest=screen.choices.iter().find(|c|matches!(c.kind,RoomChoiceKind::Rest{..})).unwrap().id;r.step(&GameAction::Choose(rest)).unwrap();
    assert_eq!(r.decision().unwrap().room.unwrap().kind,RoomViewKind::Map);
    // The Heart cannot be entered before the Shield and Spear Victory and closed rewards.
    place(&mut r,53);let heart_node=route(&r,54);
    let before=r.clone();assert_eq!(r.step(&GameAction::Move(heart_node)),Err(RunError::InvalidAction),"no elite record");assert_eq!(r,before);
    constructed_elite(&mut r,false);let before=r.clone();assert_eq!(r.step(&GameAction::Move(heart_node)),Err(RunError::InvalidAction),"rewards still open");assert_eq!(r,before);
    // Heart Victory: one Proceed, no reward screen; pending HeartVictory is nonterminal.
    let mut r=heart(EncounterId::AwakenedOne);kill_heart(&mut r);r.settle_combat().unwrap();
    let d=r.decision().unwrap();let room=d.room.clone().unwrap();assert_eq!(room.kind,RoomViewKind::HeartVictory);
    assert_eq!(room.choices.iter().map(|c|c.kind.clone()).collect::<Vec<_>>(),vec![RoomChoiceKind::Proceed]);assert_eq!(d.terminal,None,"budget 0 here is nonterminal");
    assert_eq!(d.candidates.iter().filter(|c|matches!(c.action,GameAction::Choose(_))).count(),1);
    let total=d.reward_total;let proceed=room.choices[0].id;
    let t=r.step(&GameAction::Choose(proceed)).unwrap();
    assert_eq!(t,RunTransition{reward:110,terminal:Some(RunTerminal::HeartVictory),committed_events:vec![CommittedEvent::FloorEntered{act:4,floor:55},CommittedEvent::Ended(RunTerminal::HeartVictory)]});
    let d=r.decision().unwrap();assert_eq!((d.floor,d.terminal,d.reward_total,r.state.room),(55,Some(RunTerminal::HeartVictory),total+110,"TrueVictoryRoom"));
    assert_eq!(d.room.unwrap().kind,RoomViewKind::TrueVictory);assert!(d.candidates.is_empty());
    assert_eq!(r.step(&GameAction::Choose(proceed)),Err(RunError::Terminal),"Proceed commits once");
    // Death in Act 4 commits its own terminal.
    let mut r=act4(EncounterId::TimeEater);place(&mut r,52);r.state.hp=1;r.step(&GameAction::Move(route(&r,53))).unwrap();
    assert_eq!(r.state.active_combat.as_ref().unwrap().encounter,EncounterId::ShieldAndSpear);
    // A combat step that ends the combat settles it through the shared owner.
    let total=r.decision().unwrap().reward_total;
    for _ in 0..8{if r.state.combat.is_none(){break}r.step(&GameAction::Combat(CombatAction::EndTurn)).unwrap();}
    assert_eq!(r.state.last_combat.as_ref().map(|c|(c.encounter,c.outcome)),Some((EncounterId::ShieldAndSpear,crate::combat::Outcome::Death)));
    let d=r.decision().unwrap();assert_eq!((d.terminal,d.reward_total,d.act),(Some(RunTerminal::Death),total-1,4));
    // EmptyDeck in Act 4 commits its own terminal.
    let mut r=act4(EncounterId::DonuAndDeca);r.step(&GameAction::Move(route(&r,51))).unwrap();
    let ids=r.state.deck.iter().map(|c|c.instance).collect::<Vec<_>>();for i in ids{mutations::remove_card(&mut r.state,i).unwrap();}
    let rest=r.decision().unwrap().room.unwrap().choices.iter().find(|c|matches!(c.kind,RoomChoiceKind::Rest{..})).unwrap().id;
    let t=r.step(&GameAction::Choose(rest)).unwrap();assert_eq!(t.terminal,Some(RunTerminal::EmptyDeck));assert_eq!(r.decision().unwrap().terminal,Some(RunTerminal::EmptyDeck));
}

/// Supplied map position one edge before an unvisited node of `kind`; `burning` marks it.
fn supplied_move(r:&mut Run,kind:MapNodeKind,burning:bool){
    let map=r.state.map.as_mut().unwrap();
    let target=map.nodes.iter().find(|n|n.kind==kind&&!n.visited&&map.nodes.iter().any(|p|p.edges.contains(&n.id))).unwrap().id;
    let previous=map.nodes.iter_mut().find(|n|n.edges.contains(&target)).unwrap();previous.visited=true;r.state.floor=previous.floor;map.current=Some(previous.id);
    for n in &mut map.nodes{n.burning=burning&&n.id==target;}
    r.step(&GameAction::Move(target)).unwrap();
}

#[test]
fn m25_keys_earned_through_shared_owners(){
    let key_events=|r:&Run,key:Key|r.state.history.events.iter().filter(|e|**e==CommittedEvent::KeyAcquired(key)).count();
    // Ruby: Recall at a campfire, +5 once; a second Recall is disabled and rejected.
    let mut r=m25_act3(EncounterId::AwakenedOne);diagnostic_room(&mut r,MapNodeKind::Rest);
    let t=choose(&mut r,RoomChoiceKind::Recall);assert_eq!((r.state.keys,t.reward),([true,false,false],5));assert_eq!(key_events(&r,Key::Ruby),1);
    diagnostic_room(&mut r,MapNodeKind::Rest);let row=r.decision().unwrap().room.unwrap().choices.into_iter().find(|c|c.kind==RoomChoiceKind::Recall).unwrap();
    assert_eq!(row.status,RoomChoiceStatus::Disabled);let before=r.clone();assert_eq!(r.step(&GameAction::Choose(row.id)),Err(RunError::InvalidAction));assert_eq!(r,before);
    // Sapphire at the floor-43 chest instead of its relic.
    let mut r=m25_act3(EncounterId::AwakenedOne);diagnostic_room(&mut r,MapNodeKind::Treasure);choose(&mut r,RoomChoiceKind::OpenChest);
    let relics=r.state.relics.clone();let t=choose(&mut r,RoomChoiceKind::Sapphire);
    assert_eq!((r.state.keys,t.reward),([false,true,false],5));assert_eq!(r.state.relics,relics,"Sapphire instead of the relic");assert_eq!(key_events(&r,Key::Sapphire),1);
    assert!(r.decision().ok().and_then(|d|d.room).is_none_or(|room|room.choices.iter().all(|c|!matches!(c.kind,RoomChoiceKind::Relic(_))||c.status!=RoomChoiceStatus::Offered)));
    // Emerald after a burning-elite Victory.
    let mut r=m25_act3(EncounterId::AwakenedOne);r.state.combat_progress=CombatProgress{entered:9,settled:9};supplied_move(&mut r,MapNodeKind::Elite,true);settle_lament(&mut r);
    let t=choose(&mut r,RoomChoiceKind::Emerald);assert_eq!((r.state.keys,t.reward),([false,false,true],5));assert_eq!(key_events(&r,Key::Emerald),1);
    assert_eq!(mutations::acquire_key(&mut r.state,Key::Emerald).map(|_|()),Err(RunError::InvalidAction),"the shared key owner rejects a held key");
}

#[test]
fn m25_act4_fault_labels_name_m25(){
    use rooms::act4::{Act4FaultSite as S,fault_label};
    let checkpoint=|e:RunError|{let RunError::Faulted(f)=e else{panic!("{e:?}")};f.checkpoint};
    // Encounter: a Status in the permanent deck fails the accepted Act4 encounter entry.
    let mut r=act4(EncounterId::AwakenedOne);place(&mut r,52);let i=r.state.next_instance;r.state.next_instance+=1;r.state.deck.push(CardInstance{instance:i,id:crate::content::CardId::Dazed,upgrades:0,misc:0});
    assert_eq!(checkpoint(r.step(&GameAction::Move(route(&r,53))).unwrap_err()),"run.m25.act4.encounter");
    // Settlement: the accepted Shield and Spear source changes before settlement.
    let mut r=act4(EncounterId::AwakenedOne);place(&mut r,52);r.step(&GameAction::Move(route(&r,53))).unwrap();
    for slot in 0..2{super::m9_boss::hit(&mut r,slot,10_000);}assert_eq!(outcome(&r),Some(crate::combat::Outcome::Victory));
    let node=route(&r,53);r.state.map.as_mut().unwrap().nodes.iter_mut().find(|n|n.id==node).unwrap().burning=true;
    assert_eq!(checkpoint(r.settle_combat().unwrap_err()),"run.m25.act4.settlement");
    // Heart Victory: the Proceed choice ledger is exhausted at settlement.
    let mut r=heart(EncounterId::TimeEater);kill_heart(&mut r);r.next_choice=u64::MAX;
    assert_eq!(checkpoint(r.settle_combat().unwrap_err()),"run.m25.heart.victory");
    // Transition is forced in m25_door_commits_once. Room-entry hook and Heart Proceed
    // publication cannot fail after their preflights; the selector is pinned for them.
    let r=act4(EncounterId::DonuAndDeca);
    assert_eq!([S::Transition,S::Room,S::Encounter,S::Settlement,S::HeartVictory,S::HeartProceed].map(|s|fault_label(&r.state,s)),
        ["run.m25.act4.transition","run.m25.act4.room","run.m25.act4.encounter","run.m25.act4.settlement","run.m25.heart.victory","run.m25.heart.proceed"]);
    // M20 keeps its existing labels, including a forced transition fault.
    let mut m20=super::super::diagnostics::m20_supplied_act4(super::super::diagnostics::M20DiagnosticLoadout::Defect).unwrap();
    assert_eq!([S::Transition,S::Room,S::Encounter,S::Settlement,S::HeartVictory,S::HeartProceed].map(|s|fault_label(&m20.state,s)),
        ["run.act4.transition","run.m10.act4.room","run.m10.act4.encounter","run.m10.act4.settlement","run.m10.heart.victory","run.m10.heart.proceed"]);
    let prepared=rooms::act4::prepare_transition(&m20).unwrap();m20.state.hp=m20.state.max_hp+1;rooms::act4::commit_transition(&mut m20,prepared);
    assert_eq!(m20.state.fault.unwrap().checkpoint,"run.act4.transition");
}

// ---- Task M3: Act4 producers and admission unions ----

use crate::content::{CardId,CardColor,CardRarity,Catalog,PotionId};
use rooms::m25_content as c25;
fn rarity(id:CardId)->CardRarity{Catalog::standard().card(id).unwrap().rarity}
fn supplied_start(deck:&[(CardId,u32)],relics:&[RelicId],potions:[Option<PotionId>;3])->AuthoredRunStart{
    AuthoredRunStart{hp:72,max_hp:72,gold:99,deck:deck.iter().enumerate().map(|(i,(id,up))|CardInstance{instance:i as u64+1,id:*id,upgrades:*up,misc:0}).collect(),relics:relics.to_vec(),potions:potions.to_vec(),relic_counters:Default::default()}
}

#[test]
fn m25_content_unions_are_closed(){
    use crate::content::watcher as w;
    assert_eq!(M25_WATCHER_HEART_CARDS.len(),76);assert_eq!(&M25_WATCHER_HEART_CARDS[..75],w::M25_WATCHER_CARDS);assert_eq!(M25_WATCHER_HEART_CARDS[75],CardId::Parasite);
    let distinct=M25_WATCHER_HEART_CARDS.iter().collect::<std::collections::BTreeSet<_>>();assert_eq!(distinct.len(),76);
    assert_eq!(c25::GENERATED_ONLY.len(),8);assert_eq!(c25::CHOICES,&[CardId::BecomeAlmighty,CardId::FameAndFortune,CardId::LiveForever]);
    assert_eq!(c25::STATUSES,&[CardId::Dazed,CardId::Burn,CardId::Slimed,CardId::Wound,CardId::Void]);
    assert_eq!(M25_WATCHER_HEART_RELICS.len(),23);assert_eq!(&M25_WATCHER_HEART_RELICS[..19],M24_WATCHER_ACT3_RELICS);
    assert_eq!(&M25_WATCHER_HEART_RELICS[19..],&[RelicId::Damaru,RelicId::Duality,RelicId::CloakClasp,RelicId::Melange]);
    assert_eq!(M25_WATCHER_HEART_POTIONS,rooms::m22_content::M22_WATCHER_ACT1_POTIONS);assert_eq!(M25_WATCHER_HEART_POTIONS.len(),7);
    assert_eq!(c25::RELIC_TAXONOMY_ADDITIONS,&[(RelicId::Damaru,"common"),(RelicId::Duality,"uncommon"),(RelicId::CloakClasp,"rare"),(RelicId::Melange,"shop")]);
    // Every M25 card upgrades once, except Parasite; M22–M24 admit nothing new.
    let m25=m25(EncounterId::AwakenedOne);let (m24,m23,m22)=(watcher(history(EncounterId::Guardian,EncounterId::Champ,EncounterId::AwakenedOne),[7;32],RoomScenario::M24WatcherAct3).unwrap(),
        watcher(history(EncounterId::Guardian,EncounterId::Champ,EncounterId::AwakenedOne),[7;32],RoomScenario::M23WatcherAct2).unwrap(),watcher(history(EncounterId::Guardian,EncounterId::Champ,EncounterId::AwakenedOne),[7;32],RoomScenario::M22WatcherAct1).unwrap());
    for id in M25_WATCHER_HEART_CARDS{
        let card=CardInstance{instance:999,id:*id,upgrades:0,misc:0};
        assert!(mutations::supported_in(&m25.state,*id),"{id:?}");assert_eq!(mutations::can_upgrade(&m25.state,&card),*id!=CardId::Parasite,"{id:?} upgrades once");
        assert!(!mutations::can_upgrade(&m25.state,&CardInstance{upgrades:1,..card}),"{id:?} not twice");
        if !M24_WATCHER_ACT3_CARDS.contains(id){for r in [&m24,&m23,&m22]{assert!(!mutations::supported_in(&r.state,*id),"{id:?} not admitted by older profiles");}}
    }
    let validate=|relics:Vec<RelicId>,m25:bool|{let mut counters=std::collections::BTreeMap::new();if relics.contains(&RelicId::NeowsLament){counters.insert(RelicId::NeowsLament,3);}
        let context=crate::items::CombatItemContext::new(relics,vec![None;3]).with_counters(counters);
        crate::items::validate_combat_inventory(&if m25{context.with_m25_profile()}else{context.with_m24_profile()},Character::Watcher)};
    for id in M25_WATCHER_HEART_RELICS{assert_eq!(validate(vec![*id],true),Ok(()),"M25 context accepts {id:?}");}
    for id in c25::NEW_RELICS{assert!(validate(vec![id],false).is_err(),"M24 context still rejects {id:?}");}
    for id in [RelicId::Brimstone,RelicId::RunicCapacitor,RelicId::TwistedFunnel,RelicId::BurningBlood]{assert!(validate(vec![id],true).is_err(),"M25 context rejects {id:?}");}
}

/// Expected staged sample: one bounded request per slot over the remaining family, in the
/// M25 card-offer scope (design §6.2), keyed by the semantic key plus the slot.
fn expected_row(rng:&mut RunRng,semantic:&[u64],rarities:&[CardRarity])->Vec<CardId>{
    let mut chosen=vec![];for (slot,r) in rarities.iter().enumerate(){
        let remaining=c25::purple_candidates(*r).iter().copied().filter(|id|!chosen.contains(id)).collect::<Vec<_>>();
        let mut key=semantic.to_vec();key.push(slot as u64);chosen.push(remaining[rng.bounded_scoped_checked(RunRngScope::M25CardOffer,&key,remaining.len() as u64).unwrap() as usize]);
    }chosen
}

#[test]
fn m25_act4_merchant_rows(){
    // Needs HC1: the shared sampler rejects definition-only kinds until their behaviour lands.
    let mut r=act4(EncounterId::AwakenedOne);place(&mut r,51);r.state.gold=999;let node=route(&r,52);
    let mut expected=r.state.rng.clone();let cards=expected_row(&mut expected,&[4,52,node.0],&[CardRarity::Common,CardRarity::Uncommon,CardRarity::Rare]);
    r.step(&GameAction::Move(node)).unwrap();assert_eq!(r.state.rng,expected,"sampled once under run.m25.card.offer and committed");
    let purge=r.state.room_loop.as_ref().unwrap().purge_cost;
    assert_eq!(rows(&r),vec![(RoomChoiceKind::Card{id:cards[0],upgrades:0},Some(50),RoomChoiceStatus::Offered),(RoomChoiceKind::Card{id:cards[1],upgrades:0},Some(75),RoomChoiceStatus::Offered),
        (RoomChoiceKind::Card{id:cards[2],upgrades:0},Some(150),RoomChoiceStatus::Offered),(RoomChoiceKind::Relic(RelicId::Melange),Some(150),RoomChoiceStatus::Offered),
        (RoomChoiceKind::Potion(PotionId::StancePotion),Some(50),RoomChoiceStatus::Offered),(RoomChoiceKind::Purge,Some(purge),RoomChoiceStatus::Offered),(RoomChoiceKind::Leave,None,RoomChoiceStatus::Offered)]);
    assert_eq!(cards.iter().map(|id|rarity(*id)).collect::<Vec<_>>(),vec![CardRarity::Common,CardRarity::Uncommon,CardRarity::Rare]);
    assert!(cards.iter().all(|id|Catalog::standard().card(*id).unwrap().color==CardColor::Purple));
    let (rng,screen)=(r.state.rng.clone(),rows(&r));for _ in 0..3{let _=r.decision().unwrap();}assert_eq!((r.state.rng.clone(),rows(&r)),(rng,screen),"re-reading never resamples");
}

#[test]
fn m25_act4_elite_rewards_and_relic_capture(){
    // Needs HC1 (card child sampling). Constructed: supplied position before floor 53.
    let mut r=act4(EncounterId::TimeEater);place(&mut r,52);let node=route(&r,53);r.step(&GameAction::Move(node)).unwrap();
    let visit=r.state.active_combat.as_ref().unwrap().visit;for slot in 0..2{super::m9_boss::hit(&mut r,slot,10_000);}
    let mut expected=r.state.rng.clone();let gold=25+expected.bounded_scoped(RunRngScope::CombatGoldReward,&[4,node.0,visit,MapNodeKind::Elite as u64],11) as u32;
    let relic=c25::ACT4_ELITE_RELICS[expected.bounded_scoped_checked(RunRngScope::M25RelicCapture,&[4,53,node.0,visit,0],3).unwrap() as usize];
    let cards=expected_row(&mut expected,&[4,53,node.0,visit,MapNodeKind::Elite as u64],&[CardRarity::Common,CardRarity::Uncommon,CardRarity::Uncommon]);
    r.settle_combat().unwrap();assert_eq!(r.state.rng,expected,"gold, relic and card child each requested once");
    assert_eq!(rows(&r).into_iter().map(|(k,_,_)|k).collect::<Vec<_>>(),vec![RoomChoiceKind::Gold(gold),RoomChoiceKind::CardReward,RoomChoiceKind::Potion(rooms::m22_content::combat_potion(visit)),RoomChoiceKind::Relic(relic),RoomChoiceKind::Proceed],"no Emerald row");
    choose(&mut r,RoomChoiceKind::CardReward);
    assert_eq!(rows(&r).into_iter().map(|(k,_,_)|k).collect::<Vec<_>>(),vec![RoomChoiceKind::Card{id:cards[0],upgrades:0},RoomChoiceKind::Card{id:cards[1],upgrades:0},RoomChoiceKind::Card{id:cards[2],upgrades:0},RoomChoiceKind::Skip]);
    assert_eq!(cards.iter().map(|id|rarity(*id)).collect::<Vec<_>>(),vec![CardRarity::Common,CardRarity::Uncommon,CardRarity::Uncommon]);
    // Decline and stale claims never reroll.
    choose(&mut r,RoomChoiceKind::Skip);assert_eq!(r.state.rng,expected);
    let id=r.decision().unwrap().room.unwrap().choices.iter().find(|c|c.kind==RoomChoiceKind::Relic(relic)).unwrap().id;
    r.step(&GameAction::Choose(id)).unwrap();assert!(r.state.relics.contains(&relic));
    let before=r.clone();assert_eq!(r.step(&GameAction::Choose(id)),Err(RunError::InvalidAction),"stale claim");assert_eq!(r,before);assert_eq!(r.state.rng,expected);
}

#[test]
fn m25_act4_producers_exhausted_and_unaffordable(){
    // Review Focus 4. Needs HC1 (sampling).
    let mut r=act4(EncounterId::AwakenedOne);for id in c25::ACT4_ELITE_RELICS{crate::items::acquire_relic(&mut r.state,id).unwrap();}
    place(&mut r,52);let node=route(&r,53);r.step(&GameAction::Move(node)).unwrap();let visit=r.state.active_combat.as_ref().unwrap().visit;
    for slot in 0..2{super::m9_boss::hit(&mut r,slot,10_000);}
    let mut expected=r.state.rng.clone();expected.bounded_scoped(RunRngScope::CombatGoldReward,&[4,node.0,visit,MapNodeKind::Elite as u64],11);
    expected_row(&mut expected,&[4,53,node.0,visit,MapNodeKind::Elite as u64],&[CardRarity::Common,CardRarity::Uncommon,CardRarity::Uncommon]);
    r.settle_combat().unwrap();assert_eq!(r.state.rng,expected,"an exhausted relic pool makes no RNG request");
    let kinds=rows(&r).into_iter().map(|(k,_,_)|k).collect::<Vec<_>>();assert!(!kinds.iter().any(|k|matches!(k,RoomChoiceKind::Relic(_))));assert_eq!(kinds.last(),Some(&RoomChoiceKind::Proceed));
    // Melange owned: the merchant omits it. Zero gold: every priced row is disabled, Leave stays legal.
    let mut r=act4(EncounterId::DonuAndDeca);crate::items::acquire_relic(&mut r.state,RelicId::Melange).unwrap();r.state.gold=0;place(&mut r,51);r.step(&GameAction::Move(route(&r,52))).unwrap();
    let screen=rows(&r);assert!(!screen.iter().any(|(k,_,_)|*k==RoomChoiceKind::Relic(RelicId::Melange)));
    for (k,price,status) in &screen{if price.is_some_and(|p|p>0){assert_eq!(*status,RoomChoiceStatus::Disabled,"{k:?} unaffordable");}}
    assert_eq!(screen.last(),Some(&(RoomChoiceKind::Leave,None,RoomChoiceStatus::Offered)));choose(&mut r,RoomChoiceKind::Leave);assert_eq!(room_kind(&r),Some(RoomViewKind::Map));
}

fn short_common(r:CardRarity)->&'static [CardId]{let f=c25::purple_candidates(r);if r==CardRarity::Common{&f[1..]}else{f}}
fn definition_only_family(_:CardRarity)->&'static [CardId]{
    static F:std::sync::OnceLock<Vec<CardId>>=std::sync::OnceLock::new();
    F.get_or_init(||Catalog::standard().cards().iter().filter(|row|row.definition_only).map(|row|row.id).take(1).collect())
}

#[test]
fn m25_act4_card_families_complete_or_reject(){
    let sizes=[CardRarity::Common,CardRarity::Uncommon,CardRarity::Rare].map(|r|c25::purple_candidates(r).len());assert_eq!(sizes,[19,35,17]);
    let all=[CardRarity::Common,CardRarity::Uncommon,CardRarity::Rare].iter().flat_map(|r|c25::purple_candidates(*r).iter().copied()).collect::<Vec<_>>();
    assert!(all.contains(&CardId::LessonLearned)&&all.contains(&CardId::Wish),"rewards include HEALING-tagged cards");
    assert!(all.iter().all(|id|{let row=Catalog::standard().card(*id).unwrap();row.color==CardColor::Purple&&row.rarity!=CardRarity::Basic}));
    assert!(all.iter().all(|id|M25_WATCHER_HEART_CARDS.contains(id)));
    let mut rng=RunRng::new([7;32],Partition::Test,0);let before=rng.clone();
    assert_eq!(c25::sample_capture_row_with(&mut rng,&[4,52,1],&[CardRarity::Common],short_common),Err(RunError::Unsupported("M25 incomplete Purple rarity family")),"the wrapper's size check");assert_eq!(rng,before);
    assert_eq!(definition_only_family(CardRarity::Common).len(),1,"a definition-only catalog row exists");
    assert_eq!(rooms::m15_content::sample_capture_row_for(&mut rng,&[4,52,1],&[CardRarity::Common],definition_only_family,RunRngScope::M25CardOffer,"incomplete","empty"),Err(RunError::Unsupported("incomplete")),"the shared sampler's check");
    assert_eq!(rng,before,"no RNG request before either rejection");
    assert_eq!(RunRngScope::parse("run.m25.card.offer"),Ok(RunRngScope::M25CardOffer));assert_eq!(RunRngScope::parse("run.m25.relic.capture"),Ok(RunRngScope::M25RelicCapture));
    let r=act4(EncounterId::AwakenedOne);assert_eq!(rooms::m23_content::relic_capture_scope(&r.state),RunRngScope::M25RelicCapture,"M25 scope only in act 4");
}

#[test]
fn m25_admission_profile_reads_the_m25_unions(){
    let deck=M25_WATCHER_HEART_CARDS.iter().enumerate().map(|(i,id)|(*id,if *id==CardId::Parasite{0}else{(i%2) as u32})).collect::<Vec<_>>();
    let relics=M25_WATCHER_HEART_RELICS.iter().copied().filter(|id|*id!=RelicId::HolyWater&&*id!=RelicId::NeowsLament).collect::<Vec<_>>();
    let mut with_holy=relics.iter().copied().filter(|id|*id!=RelicId::PureWater).collect::<Vec<_>>();with_holy.push(RelicId::HolyWater);
    let p=M25_WATCHER_HEART_POTIONS;
    for (relics,potions) in [(relics.clone(),[Some(p[0]),Some(p[1]),Some(p[2])]),(with_holy,[Some(p[3]),Some(p[4]),Some(p[5])]),(vec![RelicId::PureWater],[Some(p[6]),None,None])]{
        assert!(M21WatcherCombatStart::Supplied(supplied_start(&deck,&relics,potions)).normalized_for(WatcherAdmission::M25).is_ok(),"{relics:?} {potions:?}");
    }
    let mut lament=supplied_start(&deck,&[RelicId::PureWater,RelicId::NeowsLament],[None;3]);lament.relic_counters.insert(RelicId::NeowsLament,2);
    assert!(M21WatcherCombatStart::Supplied(lament).normalized_for(WatcherAdmission::M25).is_ok());
    let m25=|d:&[(CardId,u32)],r:&[RelicId],p:[Option<PotionId>;3]|M21WatcherCombatStart::Supplied(supplied_start(d,r,p)).normalized_for(WatcherAdmission::M25);
    assert_eq!(m25(&[(CardId::Bash,0)],&[RelicId::PureWater],[None;3]),Err(RunError::Unsupported("card outside the M25 permanent union")));
    assert_eq!(m25(&[(CardId::Parasite,1)],&[RelicId::PureWater],[None;3]),Err(RunError::Unsupported("card outside the M25 permanent union")),"Parasite never upgrades");
    assert!(m25(&[(CardId::Vigilance,2)],&[RelicId::PureWater],[None;3]).is_err(),"upgrade above 1");
    assert!(m25(&[(CardId::Vigilance,0)],&[RelicId::Brimstone],[None;3]).is_err(),"relic outside the union");
    assert!(m25(&[(CardId::Vigilance,0)],&[RelicId::PureWater],[Some(PotionId::FirePotion),None,None]).is_err(),"potion outside the union");
    // M21 admission keeps rejecting the new cards and relics, exactly as normalized().
    let new_card=crate::content::watcher::M25_WATCHER_CARDS[28];assert!(!crate::content::watcher::M21_WATCHER_CARDS.contains(&new_card));
    for (d,r) in [(vec![(new_card,0)],vec![RelicId::PureWater]),(vec![(CardId::Vigilance,0)],vec![RelicId::Damaru]),(vec![(CardId::Parasite,0)],vec![RelicId::PureWater])]{
        let start=supplied_start(&d,&r,[None;3]);let a=M21WatcherCombatStart::Supplied(start.clone()).normalized();
        assert!(a.is_err(),"{d:?} {r:?}");assert_eq!(M21WatcherCombatStart::Supplied(start).normalized_for(WatcherAdmission::M21),a);
    }
}
