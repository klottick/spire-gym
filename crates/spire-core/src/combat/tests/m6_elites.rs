use super::super::*;
use crate::content::{CardId,EncounterId,PotionId};
use super::m5_elites::hit;

fn entry(id:EncounterId)->CombatEntry{let mut e=super::m6_ordinary::entry(id);e.cards=vec![permanent_live_card(CardId::Strike,CardInstanceId(1),0),permanent_live_card(CardId::Defend,CardInstanceId(2),0)];e.opening_draw=2;e.setup.player.draw_per_turn=2;e}
fn cursor(c:&mut Combat,slot:usize,index:u128){let old=c.state.enemies[slot].full_ai_rng.as_ref().unwrap();c.state.enemies[slot].full_ai_rng=Some(rng::StreamState::new(old.id().clone(),[0;32],index).unwrap());}
fn roll(c:&mut Combat,slot:usize){let actor=Actor::Enemy(c.state.enemies[slot].instance_id.clone());c.enqueue_native_effects(vec![Effect{source:Source::Actor(actor.clone()),target:Some(actor),kind:EffectKind::RollMove}],None).unwrap();}
fn act(c:&mut Combat,slot:usize){let actor=Actor::Enemy(c.state.enemies[slot].instance_id.clone());c.enqueue_native_effects(vec![Effect{source:Source::Actor(actor.clone()),target:Some(actor),kind:EffectKind::EnemyMove}],None).unwrap();}
fn wounds(c:&Combat)->Vec<CardInstanceId>{c.state.piles.all(c.state.in_play.as_ref()).filter_map(|card|card.full().filter(|card|card.id==CardId::Wound).map(|c|c.instance)).collect()}
fn apply(c:&mut Combat,id:&'static str,n:i32){c.enqueue_native_effects(vec![Effect{source:Source::None,target:Some(Actor::Player),kind:EffectKind::ApplyFullPower{id:FullPowerId(id),amount:n}}],None).unwrap();}

#[test]
fn m6_slavers_actors_keep_independent_histories_and_entangle() {
    let mut c=Combat::enter(entry(EncounterId::Slavers)).unwrap();
    assert_eq!(c.state.enemies.iter().map(|e|e.enemy_id.as_str()).collect::<Vec<_>>(),vec!["slaver_blue","taskmaster","slaver_red"]);
    for (slot,lo,hi) in [(0,46,50),(1,54,60),(2,46,50)]{assert!((lo..=hi).contains(&c.state.enemies[slot].hp));}
    assert_ne!(c.state.enemies[0].full_ai_rng.as_ref().unwrap().id(),c.state.enemies[2].full_ai_rng.as_ref().unwrap().id());
    cursor(&mut c,0,0);roll(&mut c,0);assert_eq!(c.state.enemies[0].intent_id.as_str(),"slaver_rake");cursor(&mut c,2,0);apply(&mut c,"artifact",1);
    c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!((c.state.player.hp,wounds(&c).len()),(53,1));assert!(!c.state.player_full_powers.iter().any(|p|matches!(p.id.0,"weak"|"artifact")));
    hit(&mut c,0,60);let history=c.state.enemies[0].move_history.clone();let other=c.state.enemies[1].move_history.len();assert_eq!(c.state.enemies[2].intent_id.as_str(),"slaver_scrape");
    c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!((c.state.player.hp,wounds(&c).len()),(38,2));assert_eq!(c.state.enemies[0].move_history,history);assert_eq!(c.state.enemies[1].move_history.len(),other+1);
}

#[test]
fn m6_entangle_is_once_only_artifact_blockable_and_restricts_only_attacks() {
    for artifact in [false,true]{
        let mut items=Some(crate::items::CombatItemContext::new(vec![],vec![Some(PotionId::FirePotion)]));let mut c=Combat::enter_with_item_context(entry(EncounterId::Slavers),&mut items).unwrap();
        if artifact{apply(&mut c,"artifact",1)}cursor(&mut c,2,33);roll(&mut c,2);assert_eq!(c.state.enemies[2].intent_id.as_str(),"slaver_entangle");act(&mut c,2);
        assert_eq!(c.state.player_full_powers.iter().any(|p|p.id.0=="entangled"),!artifact);
        let attack=CombatAction::PlayCard{instance:1,target:Some(TargetId(1))};let skill=CombatAction::PlayCard{instance:2,target:None};let potion=CombatAction::UsePotion{slot:0,target:Some(TargetId(1))};
        let actions=c.legal_actions_full().unwrap();assert_eq!(actions.contains(&attack),artifact);assert!(actions.contains(&skill)&&actions.contains(&potion));
        if !artifact{let before=c.snapshot();assert!(c.step_full(&attack).is_err());assert_eq!(c.snapshot(),before);}
        c.step_full(&skill).unwrap();assert_eq!((c.state.player.block,c.state.player.energy),(5,2));let hp=c.state.enemies[0].hp;c.step_full(&potion).unwrap();assert_eq!(c.state.enemies[0].hp,hp-20);
        cursor(&mut c,2,33);roll(&mut c,2);assert_ne!(c.state.enemies[2].intent_id.as_str(),"slaver_entangle","executed Entangle is spent even if Artifact blocked it");
        c.state.pending.push_back(Effect{source:Source::None,target:None,kind:EffectKind::EndPlayerTurn});
        for _ in 0..20{if c.state.pending.front().is_some_and(|e|e.kind==EffectKind::MonsterStart){break}engine::dispatch_one(&mut c.state);assert!(c.state.fault.is_none());}
        assert!(c.state.pending.front().is_some_and(|e|e.kind==EffectKind::MonsterStart));assert!(!c.state.player_full_powers.iter().any(|p|p.id.0=="entangled"),"player-end removal resolves BEFORE monster-start, not at the post-enemy round hook");
        engine::resolve(&mut c.state);assert!(c.state.fault.is_none());
    }
}

#[test]
fn m6_book_counts_multistab_uses_and_hp_damage_wounds() {
    let mut e=entry(EncounterId::BookOfStabbing);e.setup.player.hp=200;e.setup.player.max_hp=200;let mut c=Combat::enter(e).unwrap();assert!((160..=164).contains(&c.state.enemies[0].hp));assert_eq!(c.state.enemies[0].intent_id.as_str(),"book_multi");cursor(&mut c,0,0);
    for (hp,count,next) in [(188,2,"book_multi"),(170,5,"book_big"),(149,6,"book_multi"),(125,10,"book_multi")] {
        c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!((c.state.player.hp,wounds(&c).len(),c.state.enemies[0].intent_id.as_str()),(hp,count,next));
    }
    let cards=c.state.piles.all(c.state.in_play.as_ref()).filter_map(|c|c.full().filter(|c|c.id==CardId::Wound)).collect::<Vec<_>>();let instances=cards.iter().map(|c|c.instance).collect::<std::collections::BTreeSet<_>>();assert_eq!(instances.len(),10);assert!(cards.iter().all(|c|matches!(c.origin,CombatCardRef::CombatLocal(_))));
}

#[test]
fn m6_book_block_boundary_and_taskmaster_status_do_not_share_a_trigger() {
    for (block,loss,want) in [(6,0,0),(5,1,1),(0,6,1)] {
        let mut c=Combat::enter(entry(EncounterId::BookOfStabbing)).unwrap();c.state.player.block=block;let source=Source::Actor(Actor::Enemy(c.state.enemies[0].instance_id.clone()));
        c.enqueue_native_effects(vec![Effect{source,target:Some(Actor::Player),kind:EffectKind::Damage(DamagePayload::Fixed(6))}],None).unwrap();assert_eq!((c.state.player.hp,wounds(&c).len()),(80-loss,want));
    }
    let mut e=entry(EncounterId::Slavers);let taskmaster=e.setup.enemies.remove(1);e.setup.enemies=vec![taskmaster];let mut c=Combat::enter(e).unwrap();c.state.player.block=7;
    act(&mut c,0);assert_eq!((c.state.player.hp,c.state.player.block,wounds(&c).len()),(80,0,1));assert!(c.state.piles.discard.iter().any(|card|card.full().is_some_and(|c|c.id==CardId::Wound)));
}

#[test]
fn m6_painful_stabs_uses_inflictor_and_qualified_damage_types() {
    let mut c=Combat::enter(entry(EncounterId::BookOfStabbing)).unwrap();let book=Actor::Enemy(c.state.enemies[0].instance_id.clone());
    c.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Player),target:Some(book.clone()),kind:EffectKind::Damage(DamagePayload::Thorns(2))},Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::Damage(DamagePayload::HpLoss(1))}],None).unwrap();assert!(wounds(&c).is_empty());
    let before=c.snapshot();assert!(c.enqueue_native_effects(vec![Effect{source:Source::Actor(book.clone()),target:Some(Actor::Player),kind:EffectKind::Damage(DamagePayload::Thorns(2))}],None).is_err());assert_eq!(c.snapshot(),before,"Book-owned THORNS is not an admitted producer");
    // Qualified source excludes THORNS, not HP_LOSS; an owner-attributed HP_LOSS callback is real.
    c.enqueue_native_effects(vec![Effect{source:Source::Actor(book),target:Some(Actor::Player),kind:EffectKind::Damage(DamagePayload::HpLoss(1))}],None).unwrap();assert_eq!(wounds(&c).len(),1);
}

#[test]
fn m6_wound_fault_preserves_prior_hit_and_first_card_frontier_without_retry() {
    let mut c=Combat::enter(entry(EncounterId::BookOfStabbing)).unwrap();let source=Source::Actor(Actor::Enemy(c.state.enemies[0].instance_id.clone()));
    c.state.pending.extend((0..2).map(|_|Effect{source:source.clone(),target:Some(Actor::Player),kind:EffectKind::Damage(DamagePayload::Fixed(6))}));
    for _ in 0..16{if wounds(&c).len()==1&&c.state.pending.front().is_some_and(|e|matches!(e.kind,EffectKind::CreateCard{id:CardId::Wound,..})){break}engine::dispatch_one(&mut c.state);assert!(c.state.fault.is_none());}
    assert_eq!((c.state.player.hp,c.state.card_ids_issued.len(),wounds(&c).len()),(68,3,1));c.state.resolved_effects=99_999;engine::dispatch_one(&mut c.state);assert_eq!(c.state.fault.as_ref().unwrap().kind,FaultKind::EffectGuard);
    let current=c.state.current.as_ref().unwrap();assert_eq!(current.source,source);assert!(matches!(current.kind,EffectKind::CreateCard{id:CardId::Wound,destination:CardDestination::Discard,..}));
    let before=c.snapshot();assert!(c.step_full(&CombatAction::EndTurn).is_err());assert_eq!(c.snapshot(),before);assert_eq!(wounds(&c).len(),1);
}

#[test]
fn m6_slaver_and_book_ai_use_exact_integer_boundaries_and_selected_memory() {
    use crate::content::EnemyId;
    let id=rng::StreamId::new(rng::Domain::RunCombatEnemyAi,vec!["fixture".into(),"elite".into()]).unwrap();let stream=|cursor|rng::StreamState::new(id.clone(),[0;32],cursor).unwrap();
    for (species,memory,history,cursor,want) in [
        (EnemyId::SlaverBlue,EnemyMemory::SlaverBlue,vec![],8,"slaver_rake"),(EnemyId::SlaverBlue,EnemyMemory::SlaverBlue,vec![],51,"slaver_stab"),
        (EnemyId::SlaverBlue,EnemyMemory::SlaverBlue,vec!["slaver_rake","slaver_rake"],8,"slaver_stab"),(EnemyId::SlaverBlue,EnemyMemory::SlaverBlue,vec!["slaver_stab","slaver_stab"],51,"slaver_rake"),
        (EnemyId::SlaverRed,EnemyMemory::SlaverRed{used_entangle:false},vec!["slaver_stab"],271,"slaver_scrape"),(EnemyId::SlaverRed,EnemyMemory::SlaverRed{used_entangle:false},vec!["slaver_stab"],16,"slaver_entangle"),
        (EnemyId::SlaverRed,EnemyMemory::SlaverRed{used_entangle:false},vec!["slaver_stab","slaver_scrape","slaver_scrape"],271,"slaver_stab"),
        (EnemyId::SlaverRed,EnemyMemory::SlaverRed{used_entangle:true},vec!["slaver_stab","slaver_entangle","slaver_stab"],37,"slaver_scrape"),(EnemyId::SlaverRed,EnemyMemory::SlaverRed{used_entangle:true},vec!["slaver_stab","slaver_entangle","slaver_stab"],90,"slaver_stab"),
        (EnemyId::SlaverRed,EnemyMemory::SlaverRed{used_entangle:true},vec!["slaver_stab","slaver_entangle","slaver_stab","slaver_stab"],90,"slaver_scrape"),(EnemyId::SlaverRed,EnemyMemory::SlaverRed{used_entangle:true},vec!["slaver_stab","slaver_entangle","slaver_scrape","slaver_scrape"],37,"slaver_stab")
    ] {
        let history=history.into_iter().map(FullIntentId::from_name).collect::<Vec<_>>();let selected=enemies::select_move(species,&history,&memory,&stream(cursor)).unwrap();assert_eq!(selected.intent.as_str(),want);assert_eq!(selected.memory,memory,"selection alone must not spend Entangle");assert_eq!(selected.stream.word_index(),cursor+1);
    }
    for (history,cursor,before,want,count) in [(vec![],125,1,"book_big",1),(vec![],225,1,"book_multi",2),(vec!["book_big"],125,1,"book_multi",2),(vec!["book_multi","book_multi"],225,3,"book_big",3)] {
        let history=history.into_iter().map(FullIntentId::from_name).collect::<Vec<_>>();let selected=enemies::select_move(EnemyId::BookOfStabbing,&history,&EnemyMemory::BookOfStabbing{stab_count:before},&stream(cursor)).unwrap();assert_eq!(selected.intent.as_str(),want);assert_eq!(selected.memory,EnemyMemory::BookOfStabbing{stab_count:count});
    }
    let exhausted=stream(1_u128<<64);let selected=enemies::select_move(EnemyId::SlaverRed,&[],&EnemyMemory::SlaverRed{used_entangle:false},&exhausted).unwrap();assert_eq!(selected.intent.as_str(),"slaver_stab");assert_eq!(selected.stream,exhausted);
}
