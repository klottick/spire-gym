use super::super::*;
use super::full_runtime::entered_combat;
use crate::combat::model::{ExhaustBatchFilter,ExhaustBatchFinish,ExhaustBatch,FinishDisposition,OwnedCard,PlayContext};
use crate::content::{CardId,CardRarity,CardType,Catalog};

fn combat_with_action_card(id:CardId,upgrades:u32)->Combat{
    let mut combat=entered_combat(3,3,|_|{});
    let card=combat.state.piles.hand.iter_mut().find(|owned|owned.handle()==card_state::handle(CardInstanceId(1))).unwrap();
    *card=OwnedCard::Full(card_state::permanent(id,CardInstanceId(1),upgrades));
    assert!(engine::validate_runtime(&combat.state).is_ok(),"canonical live actors and issued card IDs validate before action");
    combat
}

fn replace_card(combat:&mut Combat,instance:u64,id:CardId,upgrades:u32){
    let owned=combat.state.piles.hand.iter_mut().find(|owned|owned.handle()==card_state::handle(CardInstanceId(instance))).unwrap();
    *owned=OwnedCard::Full(card_state::permanent(id,CardInstanceId(instance),upgrades));
}

fn add_hand(combat:&mut Combat,instance:u64,id:CardId){
    let handle=card_state::handle(CardInstanceId(instance));
    assert!(combat.state.card_ids_issued.insert(handle));
    combat.state.piles.hand.push(OwnedCard::Full(card_state::permanent(id,CardInstanceId(instance),0)));
}

fn add_power(combat:&mut Combat,id:&'static str,amount:i32){
    combat.state.player_full_powers.push(FullPowerEntry{id:FullPowerId(id),state:PowerState{amount,skip_first:false,monster_source_grace:false}});
}

fn instance(combat:&Combat,id:CardId)->u64{
    combat.state.piles.hand.iter().filter_map(OwnedCard::full).find(|card|card.id==id).unwrap().instance.0
}

#[test]
fn m10_exhaust_actions_have_literal_catalog_rows_costs_and_properties(){
    let catalog=Catalog::standard();
    for (id,kind,rarity,cost,damage,block,exhaust) in [
        (CardId::SecondWind,CardType::Skill,CardRarity::Uncommon,1,None,Some(5),false),
        (CardId::SeverSoul,CardType::Attack,CardRarity::Uncommon,2,Some(16),None,false),
        (CardId::FiendFire,CardType::Attack,CardRarity::Rare,2,Some(7),None,true),
    ]{
        let definition=catalog.card(id).expect("catalog row");
        assert_eq!((definition.card_type,definition.rarity),(kind,rarity));
        assert!(!definition.definition_only);
        let normal=card_state::permanent(id,CardInstanceId(991),0);
        let upgraded=card_state::permanent(id,CardInstanceId(992),1);
        assert_eq!(normal.local.exhaust_on_finish,exhaust,"intrinsic disposition for {id:?}");
        assert_eq!(upgraded.local.exhaust_on_finish,exhaust,"upgraded intrinsic disposition for {id:?}");
        let generated=card_state::generated(id,CardInstanceId(993),1).expect("generated/stat-equivalent constructor supports actual card flags");
        assert_eq!(generated.local.exhaust_on_finish,exhaust,"generated intrinsic disposition for {id:?}");
        let adapter=generated.copy_as(CardInstanceId(994));
        assert_eq!(adapter.local.exhaust_on_finish,exhaust,"native adapter clone preserves intrinsic disposition");
        let combat=combat_with_action_card(id,0);
        let live=combat.state.piles.hand.iter().filter_map(OwnedCard::full).find(|card|card.id==id).unwrap();
        assert_eq!(cards::visible_cost(&combat.state,live),Some(cost),"printed cost for {id:?}");
        let values=cards::ironclad::rules::public_values(live).unwrap();
        assert_eq!((values.base_damage,values.base_block,values.exhaust_on_finish),(damage,block,exhaust));
        let combat=combat_with_action_card(id,1);
        let live=combat.state.piles.hand.iter().filter_map(OwnedCard::full).find(|card|card.id==id).unwrap();
        let values=cards::ironclad::rules::public_values(live).unwrap();
        assert_eq!(cards::visible_cost(&combat.state,live),Some(cost));
        assert_eq!(values.exhaust_on_finish,exhaust);
        assert_eq!(values.base_damage,match id{CardId::SecondWind=>None,CardId::SeverSoul=>Some(22),CardId::FiendFire=>Some(10),_=>unreachable!()});
        assert_eq!(values.base_block,if id==CardId::SecondWind{Some(7)}else{None});
    }
}

#[test]
fn m10_exhaust_action_values_survive_real_upgraded_armaments(){
    let mut entry=super::m5_elites::entry(crate::content::EncounterId::Lagavulin);
    entry.cards=vec![permanent_live_card(CardId::Armaments,CardInstanceId(1),1),
        permanent_live_card(CardId::SecondWind,CardInstanceId(2),0),
        permanent_live_card(CardId::SeverSoul,CardInstanceId(3),0),
        permanent_live_card(CardId::FiendFire,CardInstanceId(4),0)];
    entry.opening_draw=4;
    let mut combat=Combat::enter(entry).unwrap();
    combat.step_full(&CombatAction::PlayCard{instance:1,target:None}).unwrap();
    for (id,expected_damage,expected_block,expected_exhaust) in [
        (CardId::SecondWind,None,Some(7),false),(CardId::SeverSoul,Some(22),None,false),(CardId::FiendFire,Some(10),None,true)]{
        let card=combat.state.piles.hand.iter().filter_map(OwnedCard::full).find(|card|card.id==id).unwrap();
        assert_eq!(card.upgrades,1,"actual Armaments upgrades {id:?}");
        let values=cards::ironclad::rules::public_values(card).unwrap();
        assert_eq!((values.base_damage,values.base_block,values.exhaust_on_finish),(expected_damage,expected_block,expected_exhaust));
        assert_eq!(card.local.exhaust_on_finish,expected_exhaust,"Armaments does not mutate intrinsic disposition for {id:?}");
    }
}

#[test]
fn m10_second_wind_exhausts_non_attacks_then_grants_per_card_block(){
    let mut combat=combat_with_action_card(CardId::SecondWind,0);
    replace_card(&mut combat,3,CardId::Strike,0);
    add_hand(&mut combat,4,CardId::Inflame);
    add_hand(&mut combat,5,CardId::Dazed);
    add_hand(&mut combat,6,CardId::Parasite);
    add_hand(&mut combat,7,CardId::Sentinel);
    for (id,card) in [(8,CardId::Anger),(9,CardId::Clash),(10,CardId::BodySlam)]{add_hand(&mut combat,id,card)}
    for id in 12..17{let handle=card_state::handle(CardInstanceId(id));assert!(combat.state.card_ids_issued.insert(handle));combat.state.piles.draw.push(OwnedCard::Full(card_state::permanent(CardId::Defend,CardInstanceId(id),0)));}
    add_power(&mut combat,"feel_no_pain",3);add_power(&mut combat,"dark_embrace",1);
    engine::validate_runtime(&combat.state).unwrap();
    let result=combat.step_full_observed(&CombatAction::PlayCard{instance:instance(&combat,CardId::SecondWind),target:None});
    result.result.unwrap();
    let exhausted_order=result.effects.iter().filter_map(|event|match &event.kind{
        ResolvedEffectKind::CardMove{card,after:PublicZone::Exhaust,..} if card.as_str()!="1"=>Some(card.as_str().to_owned()),_=>None}).collect::<Vec<_>>();
    assert_eq!(exhausted_order,vec!["7","6","5","4","2"],"captured refs resolve in reverse hand order because source action uses addToTop");
    for id in [2,4,5,6,7]{assert_eq!(combat.state.piles.exhaust.iter().filter(|card|card.handle()==card_state::handle(CardInstanceId(id))).count(),1)}
    for id in [3,8,9,10]{assert!(!combat.state.piles.exhaust.iter().any(|card|card.handle()==card_state::handle(CardInstanceId(id))))}
    assert_eq!(combat.state.player.block,40,"five actual exhausts each grant printed Block5 and Feel No Pain3");
    assert_eq!(combat.state.player.energy,4,"Sentinel's per-instance exhaust callback resolves once");
    assert_eq!(combat.state.hook_events.iter().filter(|event|event.hook==Hook::CardExhausted).count(),5);
    assert_eq!(result.effects.iter().filter(|event|matches!(event.kind,ResolvedEffectKind::CardMove{after:PublicZone::Exhaust,..})).count(),5);
    assert_eq!(combat.state.piles.hand.len(),9,"draw callbacks refill the slots freed by playing Second Wind from a full ten-card hand");
    for id in 12..17{assert!(combat.state.piles.hand.iter().any(|card|card.handle()==card_state::handle(CardInstanceId(id))),"drawn card {id} remains outside the captured snapshot");}
}

#[test]
fn m10_sever_soul_exhausts_non_attacks_before_targeted_damage(){
    let mut combat=combat_with_action_card(CardId::SeverSoul,0);
    let before=combat.state.enemies[0].hp;
    replace_card(&mut combat,3,CardId::Strike,0);add_hand(&mut combat,4,CardId::Inflame);add_hand(&mut combat,5,CardId::Dazed);
    let result=combat.step_full_observed(&CombatAction::PlayCard{instance:instance(&combat,CardId::SeverSoul),target:Some(TargetId(1))});
    result.result.unwrap();
    let exhausted_order=result.effects.iter().filter_map(|event|match &event.kind{
        ResolvedEffectKind::CardMove{card,after:PublicZone::Exhaust,..} if card.as_str()!="2"=>Some(card.as_str().to_owned()),_=>None}).collect::<Vec<_>>();
    assert_eq!(exhausted_order,vec!["5","4"],"ExhaustAllNonAttackAction's top-stack insertion reverses its captured refs");
    assert_eq!(combat.state.enemies[0].hp,before-16);
    for id in [2,4,5]{assert!(combat.state.piles.exhaust.iter().any(|card|card.handle()==card_state::handle(CardInstanceId(id))))}
    let last_exhaust=result.effects.iter().rposition(|event|matches!(event.kind,ResolvedEffectKind::CardMove{after:PublicZone::Exhaust,..})).unwrap();
    let hit=result.effects.iter().position(|event|matches!(event.kind,ResolvedEffectKind::Damage{incoming:16,..})).unwrap();
    assert!(last_exhaust<hit,"all non-Attack moves and callbacks precede Sever Soul's hit");
}

#[test]
fn m10_fiend_fire_exhausts_each_captured_hand_card_hits_and_exhausts_source(){
    let mut combat=combat_with_action_card(CardId::FiendFire,0);
    replace_card(&mut combat,2,CardId::Inflame,0);replace_card(&mut combat,3,CardId::Dazed,0);
    for (id,card) in [(4,CardId::Parasite),(5,CardId::Anger),(6,CardId::Clash),(7,CardId::BodySlam),(8,CardId::PommelStrike),(9,CardId::Defend),(10,CardId::Wound)]{add_hand(&mut combat,id,card)}
    for id in 12..22{let handle=card_state::handle(CardInstanceId(id));assert!(combat.state.card_ids_issued.insert(handle));combat.state.piles.draw.push(OwnedCard::Full(card_state::permanent(CardId::Defend,CardInstanceId(id),0)));}
    combat.state.enemies[0].hp=200;combat.state.enemies[0].max_hp=200;
    let random_before=combat.state.card_rng.clone();add_power(&mut combat,"feel_no_pain",2);add_power(&mut combat,"dark_embrace",1);
    assert_eq!(combat.state.piles.hand.len(),10,"legal maximum hand before the card is played");
    engine::validate_runtime(&combat.state).expect("complete Fiend Fire fixture validates before play");
    let before=combat.state.enemies[0].hp;
    let result=combat.step_full_observed(&CombatAction::PlayCard{instance:instance(&combat,CardId::FiendFire),target:Some(TargetId(1))});
    result.result.unwrap();
    assert_eq!(combat.state.enemies[0].hp,before-63,"the nine eligible cards in a legal ten-card hand each produce one seven damage hit");
    assert!(combat.state.piles.exhaust.iter().any(|card|card.handle()==card_state::handle(CardInstanceId(1))),"Fiend Fire itself exhausts");
    for id in 2..=10{assert_eq!(combat.state.piles.exhaust.iter().filter(|card|card.handle()==card_state::handle(CardInstanceId(id))).count(),1)}
    assert_eq!(result.effects.iter().filter(|event|matches!(event.kind,ResolvedEffectKind::Damage{incoming:7,..})).count(),9);
    let last_hand_exhaust=result.effects.iter().rposition(|event|matches!(&event.kind,ResolvedEffectKind::CardMove{card,after:PublicZone::Exhaust,..} if card.as_str()!="1")).unwrap();
    let first_hit=result.effects.iter().position(|event|matches!(event.kind,ResolvedEffectKind::Damage{incoming:7,..})).unwrap();
    assert!(last_hand_exhaust<first_hit,"captured hand exhausts and their callbacks precede the Fiend Fire hit batch");
    let source_exhaust=result.effects.iter().position(|event|matches!(&event.kind,ResolvedEffectKind::CardMove{card,after:PublicZone::Exhaust,..} if card.as_str()=="1")).unwrap();
    assert!(source_exhaust>first_hit,"Fiend Fire's intrinsic source exhaust is separate and follows its hits");
    let last_hit=result.effects.iter().rposition(|event|matches!(event.kind,ResolvedEffectKind::Damage{incoming:7,..})).unwrap();
    let first_refill=result.effects.iter().position(|event|matches!(event.kind,ResolvedEffectKind::CardMove{after:PublicZone::Hand,..})).unwrap();
    assert!(last_hit<first_refill,"on-exhaust draw children run after all already queued Fiend Fire exhausts and hits");
    assert!(combat.state.card_rng.as_ref().unwrap().word_index()>random_before.as_ref().unwrap().word_index(),"multiple captured candidates use the scoped card stream");
    assert_eq!(combat.state.hook_events.iter().filter(|event|event.hook==Hook::CardExhausted).count(),10,"nine hand cards plus the source's own intrinsic exhaust");
    assert_eq!(combat.state.piles.hand.len(),10,"Dark Embrace fills the hand while the captured exhaust set stays bounded");
    for id in 12..22{assert!(combat.state.piles.hand.iter().any(|card|card.handle()==card_state::handle(CardInstanceId(id))),"callback draw {id} is not added to Fiend Fire's captured set");}
    assert!(engine::validate_runtime(&combat.state).is_ok(),"complete Fiend Fire outcome validates");
}

#[test]
fn m10_nonattack_exhaust_actions_without_candidates_keep_their_action_specific_finish(){
    let mut second=combat_with_action_card(CardId::SecondWind,0);replace_card(&mut second,2,CardId::Strike,0);replace_card(&mut second,3,CardId::Bash,0);
    second.step_full(&CombatAction::PlayCard{instance:instance(&second,CardId::SecondWind),target:None}).unwrap();
    assert_eq!(second.state.player.block,0);assert!(second.state.piles.exhaust.is_empty());
    let mut sever=combat_with_action_card(CardId::SeverSoul,0);replace_card(&mut sever,2,CardId::Strike,0);replace_card(&mut sever,3,CardId::Bash,0);
    let before=sever.state.enemies[0].hp;
    sever.step_full(&CombatAction::PlayCard{instance:instance(&sever,CardId::SeverSoul),target:Some(TargetId(1))}).unwrap();
    assert_eq!(sever.state.enemies[0].hp,before-16,"Sever Soul still hits when no hand card is eligible");
}

#[test]
fn m10_fiend_fire_empty_hand_has_no_hand_hits_but_still_exhausts_source(){
    let mut combat=combat_with_action_card(CardId::FiendFire,0);
    replace_card(&mut combat,2,CardId::Strike,0);replace_card(&mut combat,3,CardId::Bash,0);
    for id in [2_u64,3]{
        let index=combat.state.piles.hand.iter().position(|card|card.handle()==card_state::handle(CardInstanceId(id))).unwrap();
        let card=combat.state.piles.hand.remove(index);combat.state.piles.discard.push(card);
    }
    engine::validate_runtime(&combat.state).expect("valid zero-eligible fixture before action");
    let before=combat.state.enemies[0].hp;
    let result=combat.step_full_observed(&CombatAction::PlayCard{instance:1,target:Some(TargetId(1))});
    result.result.unwrap();
    assert_eq!(combat.state.enemies[0].hp,before);
    assert_eq!(result.effects.iter().filter(|event|matches!(event.kind,ResolvedEffectKind::Damage{incoming:7,..})).count(),0);
    assert_eq!(combat.state.piles.exhaust.iter().filter(|card|card.handle()==card_state::handle(CardInstanceId(1))).count(),1);
    assert_eq!(combat.state.piles.exhaust.len(),1,"no hand card is fabricated into the batch");
    assert!(engine::validate_runtime(&combat.state).is_ok());
}

#[test]
fn m10_exhaust_action_target_shapes_reject_atomically(){
    for (id,target) in [(CardId::SecondWind,Some(TargetId(1))),(CardId::SeverSoul,None),(CardId::FiendFire,None),
        (CardId::SeverSoul,Some(TargetId(99))),(CardId::FiendFire,Some(TargetId(99)))]{
        let mut combat=combat_with_action_card(id,0);let before=combat.state.clone();
        let action=CombatAction::PlayCard{instance:instance(&combat,id),target};
        assert!(combat.step_full(&action).is_err(),"invalid target shape for {id:?}");
        assert_eq!(combat.state,before,"invalid targeting must preflight atomically for {id:?}");
    }
}

#[test]
fn m10_fiend_fire_hits_stop_at_actual_minion_death_but_finish_hand_exhausts(){
    let mut entry=super::m6_ordinary::entry(crate::content::EncounterId::GremlinLeader);
    entry.cards=(1..=10).map(|id|permanent_live_card(if id==1{CardId::FiendFire}else{CardId::Defend},CardInstanceId(id),0)).collect();
    entry.opening_draw=10;
    let mut combat=Combat::enter(entry).unwrap();
    let minion=combat.state.enemies.iter().position(|enemy|enemy.full_powers.iter().any(|power|power.id.0=="minion")).unwrap();
    combat.state.enemies[minion].hp=10;
    engine::validate_runtime(&combat.state).unwrap();
    let target=TargetId((minion+1) as u64);
    let result=combat.step_full_observed(&CombatAction::PlayCard{instance:1,target:Some(target)});
    result.result.unwrap();
    assert_eq!(combat.state.enemies[minion].hp,0);
    assert_eq!(combat.state.enemies[minion].hp,0,"normal death is represented by zero HP and the ordinary Death hook");
    assert!(combat.state.hook_events.iter().any(|event|event.hook==Hook::Death&&event.target==Some(Actor::Enemy(combat.state.enemies[minion].instance_id.clone()))));
    assert_eq!(result.effects.iter().filter(|event|matches!(event.kind,ResolvedEffectKind::Damage{incoming:7,..})).count(),2,
        "the scheduled per-instance attacks stop once normal Minion death commits");
    for id in 2..=10{assert!(combat.state.piles.exhaust.iter().any(|card|card.handle()==card_state::handle(CardInstanceId(id))),"captured card {id} exhausts before damage");}
    assert_eq!(combat.state.outcome,None,"the Gremlin Leader and other encounter actors remain live");
}

#[test]
fn m10_fiend_fire_half_dead_target_accepts_only_the_lethal_hit(){
    let mut entry=super::m8_shapes::entry(crate::content::EncounterId::AwakenedOne);
    entry.setup.enemies[2].hp=5;
    entry.cards=(1..=10).map(|id|permanent_live_card(if id==1{CardId::FiendFire}else{CardId::Defend},CardInstanceId(id),0)).collect();
    entry.opening_draw=10;
    let mut combat=Combat::enter(entry).unwrap();
    let result=combat.step_full_observed(&CombatAction::PlayCard{instance:1,target:Some(TargetId(3))});
    result.result.unwrap();
    assert_eq!((combat.state.enemies[2].hp,combat.state.enemies[2].life),(0,ActorLife::HalfDead));
    assert_eq!(combat.state.outcome,None,"Awakened One retains its Rebirth decision after the lethal Fiend hit");
    assert_eq!(result.effects.iter().filter(|event|matches!(event.kind,ResolvedEffectKind::Damage{incoming:7,..})).count(),1,
        "later per-instance hits do not treat HalfDead as an ordinary attackable corpse");
    assert!(combat.state.piles.exhaust.iter().any(|card|card.handle()==card_state::handle(CardInstanceId(1))));
    assert!(engine::validate_runtime(&combat.state).is_ok());
}

#[test]
fn m10_exhaust_batch_callback_fault_retains_committed_owner_and_continuation(){
    let mut combat=combat_with_action_card(CardId::SecondWind,0);let owner=card_state::handle(CardInstanceId(1));
    let parent_index=combat.state.piles.hand.iter().position(|card|card.handle()==owner).unwrap();
    combat.state.in_play=Some(combat.state.piles.hand.remove(parent_index));
    combat.state.play_context=Some(PlayContext{card:owner.clone(),target:None,energy_on_use:3,paid:1,disposition:FinishDisposition::Natural});
    combat.state.phase=Phase::PlayerAction;
    combat.state.resolved_effects=99_998;
    combat.state.pending.push_back(Effect{source:Source::Card(owner.clone()),target:None,
        kind:EffectKind::ExhaustBatchStart{filter:ExhaustBatchFilter::NonAttack,random:false,
            finish:ExhaustBatchFinish::BlockPerExhaust(5)}});
    combat.state.pending.push_back(Effect{source:Source::Card(owner.clone()),target:None,kind:EffectKind::FinishCard});
    combat.state.pending.push_back(Effect{source:Source::None,target:None,kind:EffectKind::Decision});
    engine::validate_runtime(&combat.state).unwrap();
    engine::resolve(&mut combat.state);
    assert_eq!(combat.state.fault.as_ref().unwrap().kind,FaultKind::EffectGuard);
    assert_eq!(combat.state.fault.as_ref().unwrap().message,"effect dispatch limit");
    assert!(combat.state.piles.exhaust.iter().any(|card|card.handle()==card_state::handle(CardInstanceId(2))));
    assert!(combat.state.piles.hand.iter().any(|card|card.handle()==card_state::handle(CardInstanceId(3))));
    assert!(combat.state.in_play.as_ref().is_some_and(|card|card.handle()==owner));
    assert!(matches!(combat.state.current.as_ref().unwrap().kind,EffectKind::Hook(Hook::CardExhausted)));
    assert!(combat.state.pending.iter().any(|effect|matches!(effect.kind,EffectKind::ExhaustBatchContinue(ExhaustBatch{committed:1,..}))));
    assert_eq!(combat.state.player.block,0,"post-batch effects wait behind every committed exhaust callback");
    let retained=combat.state.clone();engine::resolve(&mut combat.state);assert_eq!(combat.state,retained,"budget-faulted callback and batch cannot retry");
}

#[test]
fn m10_exhaust_batch_start_payloads_are_rejected_before_becoming_pending(){
    let mut combat=combat_with_action_card(CardId::SecondWind,0);
    let owner=card_state::handle(CardInstanceId(1));
    let parent_index=combat.state.piles.hand.iter().position(|card|card.handle()==owner).unwrap();
    combat.state.in_play=Some(combat.state.piles.hand.remove(parent_index));
    combat.state.play_context=Some(PlayContext{card:owner.clone(),target:None,energy_on_use:3,paid:1,disposition:FinishDisposition::Natural});
    combat.state.phase=Phase::PlayerAction;
    combat.state.pending.push_back(Effect{source:Source::Card(owner.clone()),target:None,kind:EffectKind::FinishCard});
    engine::validate_runtime(&combat.state).expect("valid in-flight play before malformed batch is queued");
    combat.state.pending.push_back(Effect{source:Source::Card(owner.clone()),target:None,
        kind:EffectKind::ExhaustBatchStart{filter:ExhaustBatchFilter::NonAttack,random:false,
            finish:ExhaustBatchFinish::BlockPerExhaust(-1)}});
    assert!(engine::validate_runtime(&combat.state).is_err(),"invalid batch finish values must not enter retained public state");
    let mut malformed=combat.clone();
    malformed.state.pending.pop_back();
    let duplicate=card_state::handle(CardInstanceId(2));
    malformed.state.pending.push_back(Effect{source:Source::Card(owner.clone()),target:None,
        kind:EffectKind::ExhaustBatchContinue(ExhaustBatch{owner,candidates:vec![duplicate.clone(),duplicate],
            filter:ExhaustBatchFilter::NonAttack,random:false,remaining:2,committed:0,
            finish:ExhaustBatchFinish::BlockPerExhaust(5)})});
    assert!(engine::validate_runtime(&malformed.state).is_err(),"duplicate cursor handles are rejected in retained public work");
}

#[test]
fn m10_exhaust_batch_m9_v1_keeps_legacy_positive_and_rejects_new_start_and_continuation(){
    let mut legacy=Combat::new(&super::whole_combat::scenario(crate::combat::EnemyKind::Cultist)).unwrap();
    let old=super::differential::encode_checkpoint(&legacy);
    assert_eq!(old["content_id"],legacy.state.content_id,"existing v1 producer remains usable");
    let owner=card_state::handle(CardInstanceId(1));
    let work=[EffectKind::ExhaustBatchStart{filter:ExhaustBatchFilter::NonAttack,random:false,finish:ExhaustBatchFinish::BlockPerExhaust(5)},
        EffectKind::ExhaustBatchContinue(ExhaustBatch{owner:owner.clone(),filter:ExhaustBatchFilter::AllHand,candidates:vec![],random:true,
            remaining:0,committed:0,finish:ExhaustBatchFinish::DamageAfterBatch{target:Actor::Player,damage:7}})];
    for kind in work{
        legacy.state.current=Some(Effect{source:Source::Card(owner.clone()),target:None,kind});
        let panic=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||super::differential::encode_checkpoint(&legacy)))
            .expect_err("M9 v1 must not silently omit generic M10 exhaust batch work");
        let message=panic.downcast_ref::<String>().map(String::as_str).or_else(||panic.downcast_ref::<&str>().copied()).unwrap_or("");
        assert!(message.contains("M9 v1 cannot encode M10 exhaust batch work"),"specific compatibility rejection: {message}");
    }
}

#[test]
fn m10_fiend_fire_rng_fault_retains_uncommitted_live_count_frontier(){
    let mut combat=combat_with_action_card(CardId::FiendFire,0);
    replace_card(&mut combat,2,CardId::Defend,0);replace_card(&mut combat,3,CardId::Wound,0);
    let stream=combat.state.card_rng.as_ref().unwrap();
    combat.state.card_rng=Some(rng::StreamState::new(stream.id().clone(),*stream.key(),1_u128<<64).unwrap());
    engine::validate_runtime(&combat.state).unwrap();
    let result=combat.step_full_observed(&CombatAction::PlayCard{instance:1,target:Some(TargetId(1))});
    assert!(result.result.is_err(),"the actual random selection must report exhausted scoped entropy");
    assert_eq!(combat.state.piles.hand.len(),2,"no hand owner detaches before the failed first selection");
    assert!(combat.state.pending.iter().any(|effect|matches!(&effect.kind,
        EffectKind::ExhaustBatchContinue(ExhaustBatch{random:true,remaining:2,committed:0,candidates,..}) if candidates.is_empty())),
        "the fixed-count/current-group selector frontier is retained before the first RNG draw");
    assert_eq!(combat.state.card_rng.as_ref().unwrap().word_index(),1_u128<<64);
    let retained=combat.state.clone();engine::resolve(&mut combat.state);assert_eq!(combat.state,retained,"faulted random batch cannot retry");
}

#[test]
fn m10_fiend_fire_later_rng_fault_keeps_first_transfer_and_callback_children(){
    let mut combat=combat_with_action_card(CardId::FiendFire,0);
    replace_card(&mut combat,2,CardId::Defend,0);replace_card(&mut combat,3,CardId::Wound,0);add_hand(&mut combat,4,CardId::Inflame);add_hand(&mut combat,5,CardId::Parasite);
    add_power(&mut combat,"feel_no_pain",3);add_power(&mut combat,"dark_embrace",1);
    let stream=combat.state.card_rng.as_ref().unwrap();
    combat.state.card_rng=Some(rng::StreamState::new(stream.id().clone(),[0;32],(1_u128<<64)-2).unwrap());
    engine::validate_runtime(&combat.state).unwrap();
    let result=combat.step_full_observed(&CombatAction::PlayCard{instance:1,target:Some(TargetId(1))});
    assert!(result.result.is_err());
    assert_eq!(combat.state.piles.exhaust.iter().filter(|card|matches!(card.full(),Some(card) if matches!(card.id,CardId::Defend|CardId::Wound|CardId::Inflame|CardId::Parasite))).count(),2,
        "two RNG selections commit owners before the later entropy fault");
    assert_eq!(combat.state.piles.hand.len(),2);
    assert!(combat.state.pending.iter().any(|effect|matches!(&effect.kind,
        EffectKind::ExhaustBatchContinue(ExhaustBatch{random:true,remaining:2,committed:2,candidates,..}) if candidates.is_empty())));
    assert!(combat.state.pending.iter().any(|effect|matches!(effect.kind,EffectKind::GainBlock(3))));
    assert!(combat.state.pending.iter().any(|effect|matches!(effect.kind,EffectKind::Draw(1))));
    assert_eq!(combat.state.card_rng.as_ref().unwrap().word_index(),1_u128<<64);
    let retained=combat.state.clone();engine::resolve(&mut combat.state);assert_eq!(combat.state,retained);
}

#[test]
fn m10_time_eater_twelfth_play_resolves_second_wind_batch_before_forced_end_turn(){
    let mut entry=super::m8_shapes::entry(crate::content::EncounterId::TimeEater);
    entry.cards=vec![permanent_live_card(CardId::SecondWind,CardInstanceId(1),0),
        permanent_live_card(CardId::Defend,CardInstanceId(2),0),permanent_live_card(CardId::Inflame,CardInstanceId(3),0),
        permanent_live_card(CardId::Strike,CardInstanceId(4),0),permanent_live_card(CardId::Defend,CardInstanceId(5),0)];
    entry.opening_draw=0;
    entry.setup.enemies[0].powers.iter_mut().find(|power|power.id.0=="time_warp").unwrap().state.amount=11;
    let mut combat=Combat::enter(entry).unwrap();
    combat.state.player.draw_per_turn=1;
    add_power(&mut combat,"feel_no_pain",2);add_power(&mut combat,"dark_embrace",1);
    for id in 1..=4{
        let handle=card_state::handle(CardInstanceId(id));
        let index=combat.state.piles.draw.iter().position(|card|card.handle()==handle).unwrap();
        let card=combat.state.piles.draw.remove(index);combat.state.public.revealed_order.push(handle);combat.state.piles.hand.push(card);
    }
    engine::validate_runtime(&combat.state).expect("valid canonical Time Eater batch fixture before action");
    let result=combat.step_full_observed(&CombatAction::PlayCard{instance:1,target:None});
    result.result.unwrap();
    assert_eq!(combat.state.turn,2,"actual Time Warp forced the ordinary end-turn after the 12th played card");
    assert_eq!(enemy_power(&combat,0,"time_warp"),0);
    for id in [2_u64,3]{assert_eq!(combat.state.piles.exhaust.iter().filter(|card|card.handle()==card_state::handle(CardInstanceId(id))).count(),1)}
    assert!(combat.state.piles.all(None).any(|card|card.handle()==card_state::handle(CardInstanceId(4))),"Attack is excluded from the captured non-Attack batch");
    assert_eq!(combat.state.hook_events.iter().filter(|event|event.hook==Hook::CardExhausted&&event.source==Source::Card(card_state::handle(CardInstanceId(2)))).count(),1);
    assert_eq!(combat.state.hook_events.iter().filter(|event|event.hook==Hook::CardExhausted&&event.source==Source::Card(card_state::handle(CardInstanceId(3)))).count(),1);
    let player_blocks=result.effects.iter().filter(|event|event.target==Some(Actor::Player)&&matches!(event.kind,ResolvedEffectKind::Block{..})).map(|event|event.kind.clone()).collect::<Vec<_>>();
    assert_eq!(&player_blocks[..4],&[
        ResolvedEffectKind::Block{before:0,after:5},ResolvedEffectKind::Block{before:5,after:10},
        ResolvedEffectKind::Block{before:10,after:12},ResolvedEffectKind::Block{before:12,after:14}],
        "two per-exhaust Second Wind blocks and two deferred Feel No Pain children resolve once each before forced-end reset: {player_blocks:?}");
    assert_eq!(combat.state.piles.exhaust.iter().filter(|card|card.handle()==card_state::handle(CardInstanceId(2))).count(),1);
    assert!(combat.state.piles.all(None).any(|card|card.handle()==card_state::handle(CardInstanceId(1))),"source card remains owned exactly once through turn cleanup");
    assert!(combat.state.in_play.is_none()&&combat.state.pending_time_warp.is_none());
    assert!(engine::validate_runtime(&combat.state).is_ok(),"forced-end boundary retains valid ownership and cleanup");
}

#[test]
fn m10_havoc_second_wind_child_finishes_batch_and_both_play_owners_once(){
    let mut entry=super::m8_shapes::entry(crate::content::EncounterId::AwakenedOne);
    entry.cards=vec![permanent_live_card(CardId::Havoc,CardInstanceId(1),0),
        permanent_live_card(CardId::SecondWind,CardInstanceId(2),0),permanent_live_card(CardId::Defend,CardInstanceId(3),0),
        permanent_live_card(CardId::Inflame,CardInstanceId(4),0)];entry.opening_draw=0;
    let mut combat=Combat::enter(entry).unwrap();
    for id in [1_u64,3,4]{
        let handle=card_state::handle(CardInstanceId(id));let at=combat.state.piles.draw.iter().position(|card|card.handle()==handle).unwrap();
        let card=combat.state.piles.draw.remove(at);combat.state.public.revealed_order.push(handle);combat.state.piles.hand.push(card);
    }
    add_power(&mut combat,"feel_no_pain",2);
    engine::validate_runtime(&combat.state).expect("valid nested-play fixture before Havoc");
    combat.step_full(&CombatAction::PlayCard{instance:1,target:None}).unwrap();
    assert_eq!(combat.state.piles.exhaust.iter().filter(|card|matches!(card.handle().as_str(),"3"|"4")).count(),2);
    for id in [2_u64,3,4]{
        assert_eq!(combat.state.piles.exhaust.iter().filter(|card|card.handle()==card_state::handle(CardInstanceId(id))).count(),1);
        assert_eq!(combat.state.hook_events.iter().filter(|event|event.hook==Hook::CardExhausted&&event.source==Source::Card(card_state::handle(CardInstanceId(id)))).count(),1);
    }
    assert_eq!(combat.state.player.block,16,"two batch blocks and three one-shot Feel No Pain children (including Havoc's child exhaust) resolve once each");
    assert_eq!(combat.state.piles.all(None).filter(|card|card.handle()==card_state::handle(CardInstanceId(1))).count(),1,"Havoc parent retains one owner");
    assert!(engine::validate_runtime(&combat.state).is_ok());
}

#[test]
fn m10_double_tap_sever_soul_repeat_does_not_replay_exhausted_batch_owners(){
    let mut entry=super::m8_shapes::entry(crate::content::EncounterId::TimeEater);
    entry.cards=vec![permanent_live_card(CardId::DoubleTap,CardInstanceId(1),0),permanent_live_card(CardId::SeverSoul,CardInstanceId(2),0),
        permanent_live_card(CardId::Defend,CardInstanceId(3),0),permanent_live_card(CardId::Inflame,CardInstanceId(4),0),
        permanent_live_card(CardId::Strike,CardInstanceId(5),0)];entry.opening_draw=0;
    let mut combat=Combat::enter(entry).unwrap();
    for id in 1..=5{
        let handle=card_state::handle(CardInstanceId(id));let at=combat.state.piles.draw.iter().position(|card|card.handle()==handle).unwrap();
        let card=combat.state.piles.draw.remove(at);combat.state.public.revealed_order.push(handle);combat.state.piles.hand.push(card);
    }
    add_power(&mut combat,"feel_no_pain",2);
    engine::validate_runtime(&combat.state).expect("valid Double Tap continuation fixture before actions");
    combat.step_full(&CombatAction::PlayCard{instance:1,target:None}).unwrap();
    let hp=combat.state.enemies[0].hp;
    combat.step_full(&CombatAction::PlayCard{instance:2,target:Some(TargetId(1))}).unwrap();
    assert_eq!(combat.state.enemies[0].hp,hp-32,"Double Tap repeats Sever Soul's targeted damage exactly once");
    for id in [3_u64,4]{assert_eq!(combat.state.piles.exhaust.iter().filter(|card|card.handle()==card_state::handle(CardInstanceId(id))).count(),1)}
    assert_eq!(combat.state.hook_events.iter().filter(|event|event.hook==Hook::CardExhausted).count(),2,
        "the repeat sees the already-empty eligible batch and cannot duplicate callbacks");
    assert_eq!(combat.state.player.block,4);
    for id in [1_u64,2]{assert_eq!(combat.state.piles.all(None).filter(|card|card.handle()==card_state::handle(CardInstanceId(id))).count(),1)}
    assert!(engine::validate_runtime(&combat.state).is_ok());
}

fn enemy_power(combat:&Combat,slot:usize,id:&str)->i32{
    combat.state.enemies[slot].full_powers.iter().find(|power|power.id.0==id).map_or(0,|power|power.state.amount)
}
