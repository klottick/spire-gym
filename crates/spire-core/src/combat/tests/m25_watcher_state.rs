//! M25 state, history, selection and fatal-upgrade cards (plan Task C3).
//! Expected damage/Block/threshold values are literals from design §5.1 and
//! docs/research/2026-10-19-act1-content-watcher.md rows 77, 84, 90, 118, 120,
//! 125-126, 146-150, 509-523; mechanics are cross-checked against
//! data/cards/implementation-notes.json OMNISCIENCE and LESSON_LEARNED and
//! data/cards/research-annotations.json LESSON_LEARNED. All plays use step_full.
use super::super::*;
use crate::{content::{CardId,Character,EncounterId},items::CombatItemContext};
use super::m8_shapes::entry;
use CardId::*;

fn handle(id:u64)->CardHandle{card_state::handle(CardInstanceId(id))}
fn watcher(mut e:CombatEntry,deck:&[(CardId,u32)])->Combat{
    e.setup.character=Character::Watcher;
    e.cards=deck.iter().enumerate().map(|(i,(id,up))|m25_watcher_live_card(*id,CardInstanceId(i as u64+1),*up).unwrap()).collect();
    e.opening_draw=deck.len().min(10);e.setup.player.energy=9;e.setup.player.max_energy=9;e.setup.player.draw_per_turn=1;
    let mut items=Some(CombatItemContext::new(vec![],vec![None;3]).with_m25_profile());
    let c=Combat::enter_with_item_context(e,&mut items).unwrap();assert!(items.is_none());c
}
fn heart_entry()->CombatEntry{
    let context=enemies::SetupContext{player:PlayerState{hp:300,max_hp:300,block:0,energy:3,max_energy:3,draw_per_turn:1,powers:vec![]},
        player_powers:vec![],character:Character::Watcher,cards:vec![permanent_live_card(DefendPurple,CardInstanceId(1),0)],opening_draw:1,encounter_visit:0};
    enemies::setup(EncounterId::CorruptHeart,4,0,&mut crate::run::RunRng::new([29;32],rng::Partition::Test,0),context).unwrap()
}
fn play(c:&mut Combat,id:u64,target:Option<u64>){
    c.step_full(&CombatAction::PlayCard{instance:id,target:target.map(TargetId)})
        .unwrap_or_else(|e|panic!("play {id}: {e:?} legal={:?} fault={:?}",c.legal_actions_full(),c.fault_record()));
}
fn place(c:&mut Combat,id:u64,zone:SelectionSourceZone){
    let h=handle(id);let take=|p:&mut Vec<model::OwnedCard>|p.iter().position(|o|o.handle()==h).map(|i|p.remove(i));
    let owned=take(&mut c.state.piles.hand).or_else(||take(&mut c.state.piles.draw)).or_else(||take(&mut c.state.piles.discard)).unwrap();
    match zone{SelectionSourceZone::Hand=>c.state.piles.hand.push(owned),SelectionSourceZone::Draw=>c.state.piles.draw.push(owned),SelectionSourceZone::Discard=>c.state.piles.discard.push(owned),_=>unreachable!()}
    piles::refresh_hand_values(&mut c.state);
}
fn power_amount(c:&Combat,slot:usize,id:&str)->i32{c.state.enemies[slot].full_powers.iter().find(|p|p.id.0==id).map_or(0,|p|p.state.amount)}
fn hook_count(c:&Combat,hook:Hook)->usize{c.state.hook_events.iter().filter(|e|e.hook==hook).count()}
fn face(up:u32,base:i32,upgraded:i32)->i32{if up==0{base}else{upgraded}}

#[test]
fn m25_state_previous_card_type(){
    for up in [0,1]{
        let mut c=watcher(entry(EncounterId::OrbWalker),&[(StrikePurple,0),(FollowUp,up),(SashWhip,up)]);
        assert_eq!(c.previous_card_type(),None,"history is empty at combat start");
        let before=c.state.enemies[0].hp;play(&mut c,1,Some(1));assert_eq!(before-c.state.enemies[0].hp,6);
        let before=c.state.enemies[0].hp;play(&mut c,2,Some(1));
        assert_eq!((before-c.state.enemies[0].hp,c.state.player.energy),(face(up,7,11),8),"FollowUp after Attack refunds one energy");
        let before=c.state.enemies[0].hp;play(&mut c,3,Some(1));
        assert_eq!(before-c.state.enemies[0].hp,face(up,8,10));assert_eq!(power_amount(&c,0,"weak"),face(up,1,2),"SashWhip after Attack");

        let mut c=watcher(entry(EncounterId::OrbWalker),&[(DefendPurple,0),(CrushJoints,up),(DefendPurple,0),(Sanctity,up),(DefendPurple,0),(StrikePurple,0)]);
        place(&mut c,5,SelectionSourceZone::Draw);place(&mut c,6,SelectionSourceZone::Draw);
        play(&mut c,1,None);let before=c.state.enemies[0].hp;play(&mut c,2,Some(1));
        assert_eq!(before-c.state.enemies[0].hp,face(up,8,10));assert_eq!(power_amount(&c,0,"vulnerable"),face(up,1,2),"CrushJoints after Skill");
        play(&mut c,3,None);play(&mut c,4,None);
        assert_eq!(c.previous_card_type(),Some(crate::content::CardType::Skill));
        assert_eq!((c.state.player.block,c.state.piles.hand.len(),c.state.piles.draw.len()),(10+face(up,6,9),2,0),"Sanctity after Skill draws two");
    }

    // Combat-start history is empty; a prior Power is neither an Attack nor a Skill.
    for id in [FollowUp,SashWhip,CrushJoints,Sanctity]{
        let up=0;let mut c=watcher(entry(EncounterId::OrbWalker),&[(id,up),(DefendPurple,0),(DefendPurple,0)]);
        place(&mut c,2,SelectionSourceZone::Draw);place(&mut c,3,SelectionSourceZone::Draw);
        let hp=c.state.enemies[0].hp;let energy=c.state.player.energy;
        let action=if matches!(id,FollowUp|SashWhip|CrushJoints){Some(1)}else{None};
        play(&mut c,1,action);assert_eq!(c.state.player.energy,energy-1);
        assert_eq!(c.state.enemies[0].hp,hp-if id==FollowUp{7}else if id==SashWhip{8}else if id==CrushJoints{8}else{0});
        assert_eq!(power_amount(&c,0,"weak"),0);assert_eq!(power_amount(&c,0,"vulnerable"),0);
        assert_eq!(c.state.piles.draw.len(),2,"Sanctity at combat start draws nothing");

        let mut c=watcher(entry(EncounterId::OrbWalker),&[(Devotion,0),(id,0),(DefendPurple,0)]);
        place(&mut c,3,SelectionSourceZone::Draw);play(&mut c,1,None);let energy=c.state.player.energy;let hp=c.state.enemies[0].hp;
        play(&mut c,2,if matches!(id,FollowUp|SashWhip|CrushJoints){Some(1)}else{None});
        assert_eq!(c.state.player.energy,energy-1,"{id:?} after Power has no FollowUp refund");
        assert_eq!(power_amount(&c,0,"weak"),0);assert_eq!(power_amount(&c,0,"vulnerable"),0);
        assert_eq!(c.state.piles.draw.len(),1,"Sanctity after Power draws nothing");
        assert_eq!(c.state.enemies[0].hp,hp-if id==FollowUp{7}else if id==SashWhip{8}else if id==CrushJoints{8}else{0});
    }

    // A generated Smite play enters shared play history: Defend -> Carve Reality
    // (Attack) -> created Smite (Attack) -> FollowUp must refund one energy.
    let mut c=watcher(entry(EncounterId::OrbWalker),&[(DefendPurple,0),(CarveReality,0),(FollowUp,0)]);
    play(&mut c,1,None);play(&mut c,2,Some(1));
    let smite=c.state.piles.hand.iter().find(|card|card.full().is_some_and(|live|live.id==Smite)).unwrap().handle().as_str().parse::<u64>().unwrap();
    play(&mut c,smite,Some(1));let energy=c.state.player.energy;play(&mut c,3,Some(1));assert_eq!(c.state.player.energy,energy,"created Attack is the previous card type");
}

#[test]
fn m25_state_signature_wallop_tantrum(){
    for up in [0,1]{
        let mut c=watcher(entry(EncounterId::OrbWalker),&[(SignatureMove,up),(SandsOfTime,0),(DefendPurple,0)]);c.state.player.energy=2;
        let signature=CombatAction::PlayCard{instance:1,target:Some(TargetId(1))};
        assert!(!c.legal_actions_full().unwrap().contains(&signature),"an unplayable 4-cost Attack still blocks Signature Move");
        place(&mut c,2,SelectionSourceZone::Draw);assert!(c.legal_actions_full().unwrap().contains(&signature),"Signature Move is legal as the only Attack");
        let before=c.state.enemies[0].hp;play(&mut c,1,Some(1));assert_eq!(before-c.state.enemies[0].hp,face(up,30,40));

        let mut c=watcher(heart_entry(),&[(Wallop,up)]);c.state.enemies[0].hp=100;
        c.state.enemies[0].full_powers.iter_mut().find(|p|p.id.0=="invincible").unwrap().state.amount=2;
        c.state.enemies[0].full_powers.iter_mut().find(|p|p.id.0=="beat_of_death").unwrap().state.amount=0;
        play(&mut c,1,Some(1));
        assert_eq!((c.state.enemies[0].hp,c.state.player.block,power_amount(&c,0,"invincible")),(98,2,0),"Wallop reads actual HP loss after Invincible");

        let mut c=watcher(heart_entry(),&[(Wallop,up)]);c.state.enemies[0].hp=100;c.state.enemies[0].block=face(up,9,12);
        let quota=power_amount(&c,0,"invincible");play(&mut c,1,Some(1));
        assert_eq!((c.state.enemies[0].hp,c.state.player.block,power_amount(&c,0,"invincible")),(100,0,quota),"full Block yields zero Wallop Block");

        let mut c=watcher(entry(EncounterId::ThreeDarklings),&[(Wallop,up)]);c.state.enemies[0].hp=5;c.state.enemies[0].block=3;
        play(&mut c,1,Some(1));assert_eq!((c.state.enemies[0].hp,c.state.player.block,c.state.enemies[0].life),(0,5,ActorLife::HalfDead),"overkill Wallop Block is capped to HP actually lost");

        // Disclosed project decision (contract Binding 9): unresolved W-RNG
        // uses the shared ShuffledDraw insertion law and card-random stream
        // already used by piles::insert_generated (watcher research rows 125, 581).
        let mut c=watcher(entry(EncounterId::OrbWalker),&[(Tantrum,up),(DefendPurple,0),(DefendPurple,0)]);
        place(&mut c,2,SelectionSourceZone::Draw);place(&mut c,3,SelectionSourceZone::Draw);
        let card_rng=c.state.card_rng.clone();let before=c.state.enemies[0].hp;let hit_events=hook_count(&c,Hook::Damage);
        play(&mut c,1,Some(1));
        assert_eq!((before-c.state.enemies[0].hp,hook_count(&c,Hook::Damage)-hit_events,c.state.character.stance),(9+if up>0{3}else{0},3+if up>0{1}else{0},Stance::Wrath));
        assert!(c.state.piles.draw.iter().any(|card|card.handle()==handle(1)),"Tantrum returns the same instance to draw");
        assert!(!c.state.piles.discard.iter().any(|card|card.handle()==handle(1)));assert_ne!(c.state.card_rng,card_rng,"random draw-pile insertion uses card RNG");
    }
}

#[test]
fn m25_state_judgment_and_omniscience(){
    for (up,hp,kill) in [(0,30,true),(0,31,false),(1,40,true),(1,41,false)]{
        let mut c=watcher(entry(EncounterId::OrbWalker),&[(Judgment,up)]);c.state.enemies[0].hp=hp;let damage=hook_count(&c,Hook::Damage);
        play(&mut c,1,Some(1));assert_eq!(c.state.enemies[0].hp,if kill{0}else{hp});
        assert_eq!(hook_count(&c,Hook::Damage),damage,"Judgment requests Instant Kill, not damage");
        assert_eq!(c.state.outcome,if kill{Some(Outcome::Victory)}else{None});
    }
    let mut heart=watcher(heart_entry(),&[(Judgment,0)]);heart.state.enemies[0].hp=30;let invincible=power_amount(&heart,0,"invincible");
    play(&mut heart,1,Some(1));assert_eq!((heart.state.enemies[0].hp,power_amount(&heart,0,"invincible")),(0,invincible),"Judgment bypasses Invincible without spending it");

    let mut darklings=watcher(entry(EncounterId::ThreeDarklings),&[(Judgment,0)]);darklings.state.enemies[0].hp=30;play(&mut darklings,1,Some(1));
    assert_eq!((darklings.state.enemies[0].hp,darklings.state.enemies[0].life,darklings.state.outcome),(0,ActorLife::HalfDead,None),"Darkling lethal owner keeps half-death");
    let mut awakened=watcher(entry(EncounterId::AwakenedOne),&[(Judgment,0)]);awakened.state.enemies[2].hp=30;play(&mut awakened,1,Some(3));
    assert_eq!((awakened.state.enemies[2].hp,awakened.state.enemies[2].life,awakened.state.enemies[2].intent_id.as_str()),(0,ActorLife::HalfDead,"awa_rebirth"),"Awakened phase one enters rebirth");
    let mut minion=watcher(entry(EncounterId::Reptomancer),&[(Judgment,0)]);
    let slot=minion.state.enemies.iter().position(|e|e.full_powers.iter().any(|p|p.id.0=="minion")).unwrap();
    minion.state.enemies[slot].hp=20;play(&mut minion,1,Some(slot as u64+1));
    assert_eq!((minion.state.enemies[slot].hp,minion.state.enemies[slot].life,minion.state.outcome),(0,ActorLife::Active,None),"minion death uses shared lethal owner");
    assert!(hook_count(&minion,Hook::Death)>0);

    // Disclosed project decision (contract Binding 9): W-CHOICE leaves nested
    // Omniscience order open; reuse Seek's (kind, instance) order, not draw order
    // (defect/state.rs:94-99; watcher research:578).
    // Its selected original and stat-equivalent copy both use shared autoplay.
    let mut c=watcher(entry(EncounterId::ThreeDarklings),&[(Omniscience,0),(StrikePurple,0),(FollowUp,0),(DefendPurple,0),(DefendPurple,0)]);
    place(&mut c,2,SelectionSourceZone::Draw);place(&mut c,4,SelectionSourceZone::Discard);place(&mut c,5,SelectionSourceZone::Draw);
    play(&mut c,1,None);let selection=c.public_full().unwrap().selection.unwrap();
    assert_eq!((selection.operation,selection.candidates.iter().map(|card|card.instance.0).collect::<Vec<_>>()),(SelectionOperation::Omniscience,vec![5,2]));
    assert!(c.state.public.known_draw.is_empty(),"selection does not publish draw order");
    c.step_full(&CombatAction::SelectCard{instance:2}).unwrap();
    let loss=c.state.enemies.iter().map(|e|e.max_hp-e.hp).sum::<i32>();
    assert_eq!((loss,c.state.player.energy),(12,5),"Strike is played twice free after Omniscience cost");
    assert!(c.state.piles.exhaust.iter().any(|card|card.handle()==handle(2)));
    assert!(c.state.inactive_cards.iter().any(|card|card.full().is_some_and(|live|live.id==StrikePurple)));
    assert_eq!(hook_count(&c,Hook::CardPlayed),3,"Omniscience and both autoplayed Attack plays enter history");
    play(&mut c,3,Some(1));assert_eq!(c.state.player.energy,5,"the last autoplayed Attack enables FollowUp");

    // Disclosed project decision (contract Binding 9): implementation-notes
    // CLASH (data/cards/implementation-notes.json:31-33) says an unplayable child
    // unplayable queued child is suppressed by the shared autoplay owner, then
    // disposed under its Omniscience-imposed Exhaust disposition.
    let mut clash=watcher(entry(EncounterId::OrbWalker),&[(Omniscience,0),(SignatureMove,0),(StrikePurple,0)]);
    place(&mut clash,2,SelectionSourceZone::Draw);play(&mut clash,1,None);
    clash.step_full(&CombatAction::SelectCard{instance:2}).unwrap();
    assert!(clash.state.piles.exhaust.iter().any(|card|card.handle()==handle(2)));
    assert_eq!(hook_count(&clash,Hook::CardPlayed),1,"the unplayable child does not fire CardPlayed or the repeat");
    assert!(!clash.state.inactive_cards.iter().any(|card|card.full().is_some_and(|live|live.id==SignatureMove)),"no purge copy is made for an unplayable selection");

    // OMNISCIENCE notes that a selected turn-ending card interrupts its repeat.
    // Meditate reaches the shared request path while keeping this case in the finite M21 set.
    let mut ending=watcher(entry(EncounterId::OrbWalker),&[(Omniscience,0),(Meditate,0),(DefendPurple,0)]);
    place(&mut ending,2,SelectionSourceZone::Draw);place(&mut ending,3,SelectionSourceZone::Discard);play(&mut ending,1,None);
    ending.step_full(&CombatAction::SelectCard{instance:2}).unwrap();
    assert_eq!(hook_count(&ending,Hook::CardPlayed),2,"the end-turn request cancels the queued Omniscience repeat");
    assert!(ending.state.piles.exhaust.iter().any(|card|card.handle()==handle(2)));
}

#[test]
fn m25_state_lesson_learned_requests_permanent_upgrade(){
    // Disclosed project decision (contract Binding 9): the source names misc
    // RNG but no separate native stream; use RunCombatCardRandom, which owns
    // native random card choices. Source lines: watcher research 516-523 and
    // implementation-notes LESSON_LEARNED. The in-combat copy remains unchanged
    // because both sources say the permanent deck is the upgrade target.
    let mut fatal=watcher(entry(EncounterId::ThreeShapes),&[(LessonLearned,0),(DefendPurple,0)]);
    fatal.state.enemies[0].hp=10;let card_rng=fatal.state.card_rng.clone();play(&mut fatal,1,Some(1));
    assert_eq!(fatal.state.enemies[0].hp,0);let journal=fatal.permanent_card_upgrades();assert_eq!(journal.len(),1);
    assert_eq!(journal[0].ordinal,1);assert!(matches!((journal[0].instance.0,journal[0].id),(1,LessonLearned)|(2,DefendPurple)));
    assert_ne!(fatal.state.card_rng,card_rng,"eligible choice uses the misc/card-random stream");
    assert_eq!(fatal.state.piles.exhaust.iter().find_map(model::OwnedCard::full).filter(|card|card.id==LessonLearned).unwrap().upgrades,0,
        "the combat copy remains unchanged; the journal targets the permanent deck instance");

    let mut nonfatal=watcher(entry(EncounterId::OrbWalker),&[(LessonLearned,0),(DefendPurple,0)]);nonfatal.state.enemies[0].hp=100;
    let stream=nonfatal.state.card_rng.clone();play(&mut nonfatal,1,Some(1));assert!(nonfatal.permanent_card_upgrades().is_empty());assert_eq!(nonfatal.state.card_rng,stream);

    let mut none=watcher(entry(EncounterId::ThreeShapes),&[(LessonLearned,1),(DefendPurple,1)]);none.state.enemies[0].hp=13;
    let stream=none.state.card_rng.clone();play(&mut none,1,Some(1));assert!(none.permanent_card_upgrades().is_empty());assert_eq!(none.state.card_rng,stream,"no eligible permanent instance makes no RNG request");

    let mut minion=watcher(entry(EncounterId::Reptomancer),&[(LessonLearned,0)]);
    let slot=minion.state.enemies.iter().position(|e|e.full_powers.iter().any(|p|p.id.0=="minion")).unwrap();minion.state.enemies[slot].hp=10;
    play(&mut minion,1,Some(slot as u64+1));assert!(minion.permanent_card_upgrades().is_empty(),"minion kill excluded");
    let mut darkling=watcher(entry(EncounterId::ThreeDarklings),&[(LessonLearned,0)]);darkling.state.enemies[0].hp=10;play(&mut darkling,1,Some(1));
    assert_eq!(darkling.state.enemies[0].life,ActorLife::HalfDead);assert!(darkling.permanent_card_upgrades().is_empty(),"half-dead Darkling excluded");
    let mut awakened=watcher(entry(EncounterId::AwakenedOne),&[(LessonLearned,0)]);awakened.state.enemies[2].hp=10;play(&mut awakened,1,Some(3));
    assert_eq!(awakened.state.enemies[2].life,ActorLife::HalfDead);assert!(awakened.permanent_card_upgrades().is_empty(),"Awakened phase one excluded");
}

#[test]
fn m25_state_zero_x_and_empty_pile_edges(){
    let mut empty=watcher(entry(EncounterId::OrbWalker),&[(Omniscience,0)]);play(&mut empty,1,None);
    assert_eq!((empty.state.player.energy,empty.state.pending_selection.is_some(),empty.state.piles.exhaust.iter().any(|card|card.handle()==handle(1))),(5,false,true),
        "empty draw pile skips selection and Omniscience still exhausts");

    let mut x=watcher(entry(EncounterId::OrbWalker),&[(Omniscience,0)]);
    let whirlwind=card_state::generated(Whirlwind,CardInstanceId(2),0).unwrap();
    x.state.card_ids_issued.insert(handle(2));x.state.piles.draw.push(model::OwnedCard::Full(whirlwind));x.state.player.energy=4;
    play(&mut x,1,None);assert!(x.state.pending_selection.is_some());x.step_full(&CombatAction::SelectCard{instance:2}).unwrap();
    assert_eq!(x.state.player.energy,0,"Omniscience X use captures zero remaining energy without spending it");
    assert!(x.state.piles.exhaust.iter().any(|card|card.handle()==handle(2)));
    // Conjure Blade at X=0 still creates one zero-hit Expunger, then exhausts
    // itself as one played Skill. Playing that child is legal and deals no damage.
    let mut conjure=watcher(entry(EncounterId::OrbWalker),&[(ConjureBlade,0)]);
    conjure.state.player.energy=0;
    play(&mut conjure,1,None);
    assert!(conjure.state.piles.exhaust.iter().any(|card|card.handle()==handle(1)),"Conjure Blade exhausts at X=0");
    assert_eq!(conjure.state.played_card_types.len(),1,"Conjure Blade counts as one played card");
    assert_eq!(conjure.previous_card_type(),Some(crate::content::CardType::Skill));
    let expunger=piles::owned_full_cards(&conjure.state).find(|card|card.id==Expunger).expect("zero-X Conjure Blade creates Expunger");
    assert_eq!((expunger.misc,expunger.upgrades),(0,0),"the created Expunger stores zero hits");
    let expunger_instance=expunger.instance.0;
    place(&mut conjure,expunger_instance,SelectionSourceZone::Hand);
    conjure.state.player.energy=1;
    let hp=conjure.state.enemies[0].hp;
    play(&mut conjure,expunger_instance,Some(1));
    assert_eq!(conjure.state.enemies[0].hp,hp,"a zero-hit Expunger deals no damage");
    assert!(conjure.state.piles.discard.iter().any(|card|card.handle()==handle(expunger_instance)),"zero-hit Expunger remains a legal played card");
    assert_eq!(conjure.state.played_card_types.len(),2,"Expunger also counts as one played card");

    // Disclosed project decision (contract Binding 9): when both piles are
    // empty, Scrawl stops at the shared draw owner's available-card boundary
    // (watcher research row 151; design F16 lines 540-541).
    let mut scrawl=watcher(entry(EncounterId::OrbWalker),&[(Scrawl,0)]);play(&mut scrawl,1,None);
    assert_eq!((scrawl.state.piles.hand.len(),scrawl.state.piles.draw.len(),scrawl.state.piles.discard.len(),scrawl.state.player.energy),(0,0,0,8),
        "Scrawl stops when both draw and discard are empty");
    assert!(scrawl.state.piles.exhaust.iter().any(|card|card.handle()==handle(1)));
    // Collect X=0: research §7.2 lines 417-427. Base captures zero and installs
    // nothing; Collect+ adds one future turn, but neither face creates a card now.
    let collect_amount=|combat:&Combat|combat.state.player_full_powers.iter().find(|power|power.id.0=="collect").map_or(0,|power|power.state.amount);
    let mut collect=watcher(entry(EncounterId::OrbWalker),&[(Collect,0)]);collect.state.player.energy=0;
    play(&mut collect,1,None);
    assert_eq!((collect_amount(&collect),collect.state.piles.exhaust.iter().filter(|card|card.handle()==handle(1)).count()),(0,1));
    assert_eq!(piles::owned_full_cards(&collect.state).filter(|card|card.id==Miracle).count(),0);

    let mut collect_plus=watcher(entry(EncounterId::OrbWalker),&[(Collect,1)]);collect_plus.state.player.energy=0;
    play(&mut collect_plus,1,None);
    assert_eq!(collect_amount(&collect_plus),1);assert_eq!(piles::owned_full_cards(&collect_plus.state).filter(|card|card.id==Miracle).count(),0);
    collect_plus.step_full(&CombatAction::EndTurn).unwrap();
    assert_eq!(piles::owned_full_cards(&collect_plus.state).filter(|card|card.id==Miracle&&card.upgrades==1).count(),1);
    assert_eq!(collect_amount(&collect_plus),0);
}
