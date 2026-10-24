use super::super::*;
use crate::content::CardId;
use crate::combat::model::OwnedCard;
use super::full_runtime::entered_combat;

fn with_card(id:CardId,upgraded:bool)->Combat{
    entered_combat(3,3,|card|*card=permanent_live_card(id,card.instance,u32::from(upgraded)))
}

fn canonical_combat(encounter:crate::content::EncounterId,id:CardId)->Combat{
    let mut entry=super::m8_shapes::entry(encounter);
    entry.cards=vec![permanent_live_card(id,CardInstanceId(1),0)];
    entry.opening_draw=1;
    let combat=Combat::enter(entry).unwrap();
    assert!(engine::validate_runtime(&combat.state).is_ok(),"canonical encounter ownership must validate before play");
    combat
}

fn shaped_reaper_then_writhing()->Combat{
    let mut entry=super::m8_shapes::entry(crate::content::EncounterId::ThreeShapes);
    let writhing_setup=super::m8_shapes::entry(crate::content::EncounterId::WrithingMass);
    let writhing_source=&writhing_setup.setup.enemies[0];
    // A source-supported mixed actor diagnostic retains this real formation
    // slot and derives Writhing's real species-qualified owner, AI stream,
    // memory, and first move through the native constructor contracts.
    let actor=&mut entry.setup.enemies[1];
    actor.enemy_id=crate::content::EnemyId::WrithingMass;
    actor.profile=crate::combat::enemies::EnemyProfile::Standard;
    let owner=format!("{}-0-1",crate::content::EnemyId::WrithingMass.as_str());
    actor.instance_id=crate::combat::EnemyHandle::parse(&owner).unwrap();
    let run_rng=crate::run::RunRng::new([41;32],crate::combat::rng::Partition::Test,0);
    actor.ai_rng=run_rng.derive_combat_stream(crate::combat::rng::Domain::RunCombatEnemyAi,3,
        crate::content::EncounterId::ThreeShapes.as_str(),0,Some(1),&owner).unwrap();
    actor.hp=160;actor.max_hp=160;actor.powers=writhing_source.powers.clone();
    let memory=crate::combat::enemies::EnemyMemory::for_enemy(actor.enemy_id).unwrap();
    let selected=crate::combat::enemies::select_move(actor.enemy_id,&[],&memory,&actor.ai_rng).unwrap();
    actor.intent=selected.intent;actor.initial_memory=Some(selected.memory);actor.ai_rng=selected.stream;
    entry.cards=vec![permanent_live_card(CardId::Reaper,CardInstanceId(1),0)];
    entry.opening_draw=1;
    let mut combat=Combat::enter(entry).unwrap();
    combat.state.enemies[0].hp=40;
    combat.state.enemies[0].block=0;
    let mass=&mut combat.state.enemies[1];
    mass.hp=160;mass.max_hp=160;
    mass.full_powers.iter_mut().find(|power|power.id.0=="malleable").expect("native Writhing owns Malleable").state.amount=999;
    assert!(engine::validate_runtime(&combat.state).is_ok(),"both canonical owners, slots, memories, and stream identities validate before the action");
    combat
}

#[test]
fn m10_fatal_feed_lethal_hit_commits_max_hp_then_ordinary_heal(){
    let mut c=with_card(CardId::Feed,false);
    let enemy=Actor::Enemy(c.state.enemies[0].instance_id.clone());
    c.state.enemies[0].hp=1;
    c.state.player.hp=20;
    c.step_full(&CombatAction::PlayCard{instance:3,target:Some(TargetId(1))}).unwrap();
    assert_eq!(c.state.player.max_hp,53);
    assert_eq!(c.state.player.hp,23);
    assert_eq!(c.state.enemies[0].hp,0);
    assert!(c.state.outcome.is_some());
    assert!(c.state.in_play.as_ref().and_then(OwnedCard::full).is_some_and(|card|card.id==CardId::Feed&&card.local.exhaust_on_finish));
    assert!(c.state.hook_events.iter().any(|event|event.target==Some(enemy.clone())));
}

#[test]
fn m10_fatal_feed_reuses_flower_and_bloom_ordinary_heal_hooks(){
    let mut rng=crate::run::RunRng::new([4;32],crate::combat::rng::Partition::Test,0);
    let mut map=crate::run::map::generate(1,&mut rng,crate::run::map::MapEligibility::new(true,false)).unwrap();
    map.current=Some(map.nodes[0].id);
    for (relics,expected_hp) in [
        (vec![crate::content::RelicId::MagicFlower],25),
        (vec![crate::content::RelicId::MarkOfBloom],20),
    ] {
        let mut c=with_card(CardId::Feed,false);
        c.state.enemies[0].hp=1;c.state.player.hp=20;
        c.state.item_context=Some(crate::items::CombatItemContext::new(relics,vec![])
            .with_native_room(crate::items::PotionRoomPhase::Combat,Some(&map)));
        c.step_full(&CombatAction::PlayCard{instance:3,target:Some(TargetId(1))}).unwrap();
        assert_eq!((c.state.player.max_hp,c.state.player.hp),(53,expected_hp));
    }
    for (relics,expected_hp) in [
        (vec![crate::content::RelicId::MagicFlower],38),
        (vec![crate::content::RelicId::MarkOfBloom],20),
    ] {
        let mut c=canonical_combat(crate::content::EncounterId::ThreeDarklings,CardId::Reaper);
        c.state.player.hp=20;
        c.state.item_context=Some(crate::items::CombatItemContext::new(relics,vec![])
            .with_native_room(crate::items::PotionRoomPhase::Combat,Some(&map)));
        c.step_full(&CombatAction::PlayCard{instance:1,target:None}).unwrap();
        assert_eq!(c.state.player.hp,expected_hp,"Reaper heal uses the same ordinary-heal relic owners as Feed");
        assert!(engine::validate_runtime(&c.state).is_ok(),"canonical actor and issued-ID ownership survives Reaper");
    }
}

#[test]
fn m10_fatal_feed_commits_max_hp_and_ordinary_heal_before_actual_spiker_thorns(){
    let mut c=canonical_combat(crate::content::EncounterId::ThreeShapes,CardId::Feed);
    let slot=c.state.enemies.iter().position(|enemy|enemy.enemy_id==ActorEnemyId::Full(crate::content::EnemyId::Spiker)).unwrap();
    let spiker=&mut c.state.enemies[slot];
    assert!(spiker.full_powers.iter().any(|power|power.id.0=="thorns"&&power.state.amount==3),"actual canonical Spiker setup owns its source Thorns");
    spiker.hp=10;
    c.state.player.hp=2;
    assert!(engine::validate_runtime(&c.state).is_ok());
    c.step_full(&CombatAction::PlayCard{instance:1,target:Some(TargetId(slot as u64+1))}).unwrap();
    assert_eq!(c.state.player.max_hp,303);
    assert_eq!(c.state.player.hp,2,"Feed's +3 ordinary heal raises HP 2→5 before the queued 3-THORNS retaliation");
    assert_eq!(c.state.outcome,None,"the queued retaliation does not manufacture an earlier Death");
    assert!(engine::validate_runtime(&c.state).is_ok());
}

#[test]
fn m10_fatal_upgraded_feed_uses_twelve_damage_and_four_max_hp(){
    let mut c=with_card(CardId::Feed,true);
    c.state.enemies[0].hp=11;
    c.state.player.hp=20;
    c.step_full(&CombatAction::PlayCard{instance:3,target:Some(TargetId(1))}).unwrap();
    assert_eq!(c.state.player.max_hp,54);
    assert_eq!(c.state.player.hp,24);
    assert_eq!(c.state.enemies[0].hp,0);
}

#[test]
fn m10_fatal_feed_minion_death_does_not_earn_the_max_hp_gain(){
    let mut c=with_card(CardId::Feed,false);
    c.state.enemies[0].hp=1;
    c.state.enemies[0].full_powers.push(FullPowerEntry{id:FullPowerId("minion"),state:PowerState{amount:1,skip_first:false,monster_source_grace:false}});
    let before=(c.state.player.hp,c.state.player.max_hp);
    c.step_full(&CombatAction::PlayCard{instance:3,target:Some(TargetId(1))}).unwrap();
    assert_eq!((c.state.player.hp,c.state.player.max_hp),before);
}

#[test]
fn m10_fatal_feed_does_not_reward_a_darkling_half_death(){
    let mut entry=super::m8_shapes::entry(crate::content::EncounterId::ThreeDarklings);
    entry.cards=vec![permanent_live_card(CardId::Feed,CardInstanceId(1),0)];
    entry.opening_draw=1;
    let mut c=Combat::enter(entry).unwrap();
    c.state.enemies[0].hp=1;
    let max_hp=c.state.player.max_hp;
    c.step_full(&CombatAction::PlayCard{instance:1,target:Some(TargetId(1))}).unwrap();
    assert_eq!(c.state.enemies[0].life,ActorLife::HalfDead);
    assert_eq!(c.state.player.max_hp,max_hp,"half-death is excluded by the actual post-damage Fatal predicate");
}

#[test]
fn m10_fatal_feed_max_hp_ceiling_fault_keeps_the_committed_hit_without_partial_gain(){
    let mut c=with_card(CardId::Feed,false);
    c.state.player.max_hp=999;c.state.player.hp=999;c.state.enemies[0].hp=1;
    let result=c.step_full(&CombatAction::PlayCard{instance:3,target:Some(TargetId(1))});
    assert!(matches!(result,Err(crate::run::RunError::Faulted(_))));
    assert_eq!(c.state.enemies[0].hp,0,"the shared hit preceded Fatal's checked max-HP mutation");
    assert_eq!(c.state.player.max_hp,999);
}

#[test]
fn m10_fatal_feed_rejects_dead_target_without_committing_payment_or_changes(){
    let mut c=with_card(CardId::Feed,false);
    c.state.enemies[0].hp=0;
    let before=c.clone();
    assert_eq!(c.step_full(&CombatAction::PlayCard{instance:3,target:Some(TargetId(1))}),Err(crate::run::RunError::InvalidAction));
    assert_eq!(c,before);
}

#[test]
fn m10_fatal_feed_exhausts_after_a_nonlethal_hit_without_awarding_growth(){
    let mut c=with_card(CardId::Feed,false);
    c.state.player.hp=20;c.state.enemies[0].hp=40;
    c.step_full(&CombatAction::PlayCard{instance:3,target:Some(TargetId(1))}).unwrap();
    assert_eq!((c.state.player.hp,c.state.player.max_hp),(20,50));
    assert!(c.state.piles.exhaust.iter().filter_map(OwnedCard::full).any(|card|card.id==CardId::Feed));
}

#[test]
fn m10_fatal_cards_keep_source_costs_upgrades_and_target_shapes(){
    for (id,cost) in [(CardId::Feed,1),(CardId::Reaper,2)] {
        for upgraded in [false,true] {
            let mut c=with_card(id,upgraded);
            let target=if id==CardId::Feed{Some(TargetId(1))}else{None};
            let prepared=cards::prepare(&c,&CombatAction::PlayCard{instance:3,target}).unwrap();
            assert_eq!(prepared.paid,cost);
            assert_eq!(prepared.remaining_energy,3-cost);
            let values=cards::ironclad::rules::public_values(c.state.piles.hand.iter().filter_map(OwnedCard::full).find(|card|card.id==id).unwrap()).unwrap();
            assert_eq!(values.magic_number,if id==CardId::Feed{Some(if upgraded{4}else{3})}else{None});
            assert!(values.exhaust_on_finish);
            let wrong=if target.is_some(){None}else{Some(TargetId(1))};
            let before=c.clone();
            assert_eq!(c.step_full(&CombatAction::PlayCard{instance:3,target:wrong}),Err(crate::run::RunError::InvalidAction));
            assert_eq!(c,before,"invalid target shape is atomic");
        }
    }
}

#[test]
fn m10_fatal_reaper_heals_actual_hp_removed_not_block_or_overkill(){
    let mut c=with_card(CardId::Reaper,false);
    c.state.player.hp=20;
    c.state.enemies[0].hp=2;
    c.state.enemies[0].block=2;
    c.step_full(&CombatAction::PlayCard{instance:3,target:None}).unwrap();
    assert_eq!(c.state.enemies[0].hp,0);
    assert_eq!(c.state.player.hp,22);
    assert!(c.state.in_play.as_ref().and_then(OwnedCard::full).is_some_and(|card|card.id==CardId::Reaper&&card.local.exhaust_on_finish));
}

#[test]
fn m10_fatal_reaper_sums_actual_loss_across_all_live_enemies(){
    let mut c=canonical_combat(crate::content::EncounterId::ThreeDarklings,CardId::Reaper);
    c.state.player.hp=20;
    for enemy in &mut c.state.enemies { enemy.block=0; }
    c.state.enemies[0].hp=4;c.state.enemies[0].block=2;
    c.state.enemies[1].hp=10;
    c.state.enemies[2].hp=5;c.state.enemies[2].block=1;
    assert!(engine::validate_runtime(&c.state).is_ok());
    c.step_full(&CombatAction::PlayCard{instance:1,target:None}).unwrap();
    assert_eq!(c.state.player.hp,29,"2+4+3 HP is removed; Block absorbs the first two damage and overkill is excluded");
    assert!(c.state.piles.exhaust.iter().filter_map(OwnedCard::full).any(|card|card.id==CardId::Reaper));
    assert!(engine::validate_runtime(&c.state).is_ok());

    let mut zero=canonical_combat(crate::content::EncounterId::ThreeDarklings,CardId::Reaper);zero.state.player.hp=20;
    for enemy in &mut zero.state.enemies {enemy.block=4;}
    assert!(engine::validate_runtime(&zero.state).is_ok());
    zero.step_full(&CombatAction::PlayCard{instance:1,target:None}).unwrap();
    assert_eq!(zero.state.player.hp,20,"zero actual HP loss produces no heal");
    assert!(engine::validate_runtime(&zero.state).is_ok());
}

#[test]
fn m10_fatal_reaper_completes_all_hits_then_retaliates_before_heal(){
    let mut c=canonical_combat(crate::content::EncounterId::ThreeShapes,CardId::Reaper);
    c.state.player.hp=3;
    for enemy in &mut c.state.enemies { enemy.block=0; }
    let spiker=c.state.enemies.iter().position(|enemy|enemy.enemy_id==ActorEnemyId::Full(crate::content::EnemyId::Spiker)).unwrap();
    assert!(c.state.enemies[spiker].full_powers.iter().any(|power|power.id.0=="thorns"&&power.state.amount==3));
    c.state.enemies[0].hp=2;
    assert!(engine::validate_runtime(&c.state).is_ok());
    c.step_full(&CombatAction::PlayCard{instance:1,target:None}).unwrap();
    assert_eq!(c.state.enemies[0].hp,0);
    assert!(c.state.enemies.iter().enumerate().filter(|(slot,_)|*slot!=0).all(|(_,enemy)|enemy.hp<enemy.max_hp),"every other canonical room-list target was hit before the first queued retaliation");
    assert_eq!(c.state.player.hp,0,"lethal queued THORNS prevents Reaper's later heal from rescuing the player");
    assert_eq!(c.state.outcome,Some(Outcome::Death));
    assert!(engine::validate_runtime(&c.state).is_ok());
}

#[test]
fn m10_fatal_reaper_postcommit_callback_fault_retains_current_hit_receipt(){
    let mut combat=shaped_reaper_then_writhing();
    combat.state.player.hp=20;
    let result=combat.step_full(&CombatAction::PlayCard{instance:1,target:None});
    assert!(matches!(result,Err(crate::run::RunError::Faulted(_))),"checked native Malleable counter overflow faults in the post-hit lifecycle callback");
    assert_eq!(combat.state.enemies[0].hp,36,"the first target's actual four HP loss completed before the later callback fault");
    assert_eq!(combat.state.enemies[0].block,0);
    assert_eq!(combat.state.enemies[1].hp,156,"the callback fault happened after the actual second hit committed");
    let current=combat.state.current.as_ref().expect("real parent retained at the callback checkpoint");
    assert!(matches!(&current.kind,EffectKind::Ironclad(IroncladEffect::ReaperResolve{next:2,healed:8,..})),
        "cursor and both completed actual HP losses, including the faulting hit, must be durable; retained={current:?}");
    assert!(combat.state.pending.iter().any(|queued|matches!(queued.kind,EffectKind::Hook(Hook::Damage))),"already queued damage callback remains owned");
    assert!(!combat.state.pending.iter().any(|queued|matches!(queued.kind,EffectKind::Heal(_))),"a callback fault cannot enqueue Reaper's heal");
    let first_fault=combat.state.fault.as_ref().expect("post-commit callback fault is retained").clone();
    assert!(first_fault.message.contains("Malleable amount exceeds native bounds"));
    assert!(matches!(&current.source,Source::Card(_)),"the original Reaper card source remains the current owner");
    let frozen=combat.clone();
    assert!(combat.step_full(&CombatAction::EndTurn).is_err());
    assert_eq!(combat,frozen,"retained hit/parent and first fault cannot be retried or replaced");
    assert_eq!(combat.state.fault.as_ref(),Some(&first_fault));
}

#[test]
fn m10_reaper_checkpoint_is_rejected_by_m9_v1_while_legacy_checkpoint_remains_encodable(){
    let mut legacy=Combat::new(&super::whole_combat::scenario(crate::combat::EnemyKind::Cultist)).unwrap();
    let legacy_v1=super::differential::encode_checkpoint(&legacy);
    assert_eq!(legacy_v1["content_id"],legacy.state.content_id,"existing M9 v1 checkpoint producer remains usable");
    let target=legacy.state.enemies[0].instance_id.clone();
    legacy.state.current=Some(Effect{source:Source::Card(CardHandle::parse("1").unwrap()),target:None,
        kind:EffectKind::Ironclad(IroncladEffect::ReaperResolve{targets:vec![(target,4)],next:0,healed:0})});
    let panic=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||super::differential::encode_checkpoint(&legacy)))
        .expect_err("M9 v1 must reject actual M10 Reaper work instead of silently encoding it");
    let message=panic.downcast_ref::<String>().map(String::as_str)
        .or_else(||panic.downcast_ref::<&str>().copied()).unwrap_or("");
    assert!(message.contains("M9 v1 cannot encode M10 Ironclad work"),"specific compatibility boundary: {message}");
}
