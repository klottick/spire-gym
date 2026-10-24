use super::super::*;
use super::super::model::OwnedCard;
use crate::{combat::{card_state,tests::full_runtime},content::CardId};

fn with_task7_card(id:CardId,upgraded:bool)->Combat{
    full_runtime::entered_combat(3,3,|card|{
        *card=card_state::permanent(id,card.instance,u32::from(upgraded));
    })
}

#[test]
fn m10_havoc_executes_real_top_card_and_imposes_exhaust(){
    let mut combat=with_task7_card(CardId::Havoc,false);
    assert_eq!(crate::content::Catalog::standard().card(CardId::Havoc).unwrap().rarity,crate::content::CardRarity::Common);
    let hand_index=combat.state.piles.hand.iter().position(|owned|owned.handle()!=card_state::handle(CardInstanceId(3))).unwrap();
    let mut child=combat.state.piles.hand.remove(hand_index);
    child.full_mut().unwrap().id=CardId::Strike;
    let child_id=child.handle().as_str().parse::<u64>().unwrap();
    combat.state.piles.draw.clear();combat.state.piles.draw.push(child);
    let before=combat.state.enemies[0].hp;
    let card_rng=combat.state.card_rng.clone();
    let actions=combat.legal_actions_full().unwrap();
    assert!(actions.iter().any(|action|matches!(action,CombatAction::PlayCard{instance:3,..})));
    assert_eq!(combat.state.card_rng,card_rng,"read-only legal-action preparation does not sample Havoc's random target");
    let result=combat.step_full(&CombatAction::PlayCard{instance:3,target:None});
    assert!(result.is_ok(),"Havoc must enter the actual top-card autoplay path: {result:?}");
    assert_eq!(combat.state.enemies[0].hp,before-6,"Havoc plays the real extracted top card");
    assert!(combat.state.piles.exhaust.iter().any(|owned|owned.handle()==card_state::handle(CardInstanceId(child_id))),"Havoc exhausts the actually played child");
    assert!(combat.state.piles.discard.iter().any(|owned|owned.handle()==card_state::handle(CardInstanceId(3))),"Havoc parent retains its own disposition");
}

#[test]
fn m10_corruption_exhaust_capture_survives_real_havoc_deferred_skill_finish(){
    let mut combat=with_task7_card(CardId::Havoc,false);
    combat.state.player_full_powers.push(super::super::model::FullPowerEntry{id:super::super::FullPowerId("corruption"),
        state:super::super::PowerState{amount:1,skip_first:false,monster_source_grace:false}});
    let child_index=combat.state.piles.hand.iter().position(|owned|owned.handle()!=card_state::handle(CardInstanceId(3))).unwrap();
    let mut child=combat.state.piles.hand.remove(child_index);let child_id=child.handle();
    let child_card=child.full_mut().unwrap();child_card.id=CardId::Defend;child_card.local.exhaust_on_finish=false;
    combat.state.piles.draw.clear();combat.state.piles.draw.push(child);
    assert!(engine::validate_runtime(&combat.state).is_ok());
    combat.step_full(&CombatAction::PlayCard{instance:3,target:None}).unwrap();
    assert!(combat.state.piles.exhaust.iter().any(|owned|owned.handle()==child_id),"Havoc deferred frame retains captured Corruption disposition");
    assert!(combat.state.piles.exhaust.iter().any(|owned|owned.handle()==card_state::handle(CardInstanceId(3))),
        "Corruption independently exhausts the Skill parent and Havoc always exhausts its child");
    assert_eq!(combat.state.hook_events.iter().filter(|event|event.hook==Hook::CardExhausted&&event.source==Source::Card(child_id.clone())).count(),1);
    assert_eq!(combat.state.hook_events.iter().filter(|event|event.hook==Hook::CardExhausted&&event.source==Source::Card(card_state::handle(CardInstanceId(3)))).count(),1);
    assert_eq!(combat.state.piles.exhaust.iter().find(|owned|owned.handle()==card_state::handle(CardInstanceId(3)))
        .unwrap().full().unwrap().local.exhaust_on_finish,false,"Corruption captures Havoc's use disposition without mutating the parent card's intrinsic flag");
    assert!(!combat.state.piles.exhaust.iter().find(|owned|owned.handle()==child_id).unwrap().full().unwrap().local.exhaust_on_finish,
        "Havoc captures child exhaust as a per-use disposition without mutating intrinsic card data");
    assert!(engine::validate_runtime(&combat.state).is_ok());
}

#[test]
fn m10_nested_havoc_captures_each_accepted_target_once(){
    let mut combat=with_task7_card(CardId::Havoc,false);
    let mut second=combat.state.enemies[0].clone();
    second.instance_id=EnemyHandle::parse("enemy-2").unwrap();
    let old=second.full_ai_rng.as_ref().unwrap();
    let id=super::super::rng::StreamId::new(super::super::rng::Domain::RunCombatEnemyAi,
        vec!["run-combat-1".into(),"enemy-scope-2".into()]).unwrap();
    second.full_ai_rng=Some(super::super::rng::StreamState::new(id,*old.key(),old.word_index()).unwrap());
    let second_id=second.instance_id.clone();combat.state.actor_slots.insert(second_id,2);combat.state.enemies.push(second);
    let nested=OwnedCard::Full(card_state::permanent(CardId::Havoc,CardInstanceId(77),0));
    let strike=OwnedCard::Full(card_state::permanent(CardId::Strike,CardInstanceId(78),0));
    combat.state.card_ids_issued.insert(nested.handle());combat.state.card_ids_issued.insert(strike.handle());
    combat.state.piles.draw.clear();combat.state.piles.draw.push(strike);combat.state.piles.draw.push(nested);
    let mut expected=combat.state.card_rng.clone().unwrap();
    for _ in 0..2{let (_,next)=super::super::rng::uniform_below(&expected,2).unwrap();expected=next;}
    let before=combat.state.enemies.iter().map(|enemy|enemy.hp).collect::<Vec<_>>();
    combat.step_full(&CombatAction::PlayCard{instance:3,target:None}).unwrap();
    assert_eq!(combat.state.card_rng.as_ref(),Some(&expected),"manual Havoc and the accepted Havoc child each capture once; prepare/readiness do not sample");
    assert_eq!(combat.state.enemies.iter().map(|enemy|enemy.hp).sum::<i32>(),before.iter().sum::<i32>()-6,
        "nested Havoc preserves the second captured target for its Strike grandchild");
    assert!(combat.state.piles.exhaust.iter().any(|owned|owned.handle()==card_state::handle(CardInstanceId(77))));
    assert!(combat.state.piles.exhaust.iter().any(|owned|owned.handle()==card_state::handle(CardInstanceId(78))));
    assert!(engine::validate_runtime(&combat.state).is_ok());
}

#[test]
fn m10_havoc_executes_power_child_and_empty_piles_are_a_noop(){
    let mut power=with_task7_card(CardId::Havoc,false);
    let index=power.state.piles.hand.iter().position(|owned|owned.handle()==card_state::handle(CardInstanceId(3))).unwrap();
    let havoc=power.state.piles.hand.remove(index);
    let child=OwnedCard::Full(card_state::permanent(CardId::Inflame,CardInstanceId(77),0));
    power.state.card_ids_issued.insert(child.handle());power.state.piles.draw.push(child.clone());power.state.piles.hand.push(havoc);
    power.step_full(&CombatAction::PlayCard{instance:3,target:None}).unwrap();
    assert_eq!(power.state.player_full_powers.iter().find(|power|power.id.0=="strength").unwrap().state.amount,2);
    assert!(power.state.played_powers.iter().any(|owned|owned.handle()==child.handle()));
    assert!(power.state.piles.discard.iter().any(|owned|owned.full().is_some_and(|card|card.id==CardId::Havoc)));
    engine::validate_runtime(&power.state).unwrap();

    let mut empty=with_task7_card(CardId::Havoc,false);
    empty.state.piles.draw.clear();empty.state.piles.discard.clear();
    empty.step_full(&CombatAction::PlayCard{instance:3,target:None}).unwrap();
    assert!(empty.state.in_play.is_none()&&empty.state.play_frames.is_empty());
    assert!(empty.state.piles.discard.iter().any(|owned|owned.full().is_some_and(|card|card.instance.0==3&&card.id==CardId::Havoc)));
    assert!(engine::validate_runtime(&empty.state).is_ok());
}

#[test]
fn m10_double_tap_repeats_one_attack_for_free_without_recursion(){
    let mut combat=with_task7_card(CardId::DoubleTap,false);
    assert_eq!(crate::content::Catalog::standard().card(CardId::DoubleTap).unwrap().rarity,crate::content::CardRarity::Rare);
    combat.state.enemies[0].full_powers.push(FullPowerEntry{id:FullPowerId("curiosity"),state:PowerState{amount:1,skip_first:false,monster_source_grace:false}});
    let result=combat.step_full(&CombatAction::PlayCard{instance:3,target:None});
    assert!(result.is_ok(),"Double Tap must enter the actual power/play path: {result:?}");
    assert!(combat.state.piles.discard.iter().any(|owned|owned.full().is_some_and(|card|card.id==CardId::DoubleTap)));
    assert!(!combat.state.played_powers.iter().any(|owned|owned.full().is_some_and(|card|card.id==CardId::DoubleTap)));
    assert!(!combat.state.enemies[0].full_powers.iter().any(|power|power.id.0=="strength"),"Double Tap is a Skill and must not invoke Power after-use Curiosity");
    let strike=combat.state.piles.hand.iter().filter_map(OwnedCard::full).find(|card|card.id==CardId::Strike).unwrap().instance.0;
    let hp=combat.state.enemies[0].hp;
    combat.step_full(&CombatAction::PlayCard{instance:strike,target:Some(TargetId(1))}).unwrap();
    assert_eq!(combat.state.enemies[0].hp,hp-12,"one original Attack and one free repeat both resolve");
}

#[test]
fn m10_double_tap_repeat_does_not_retarget_after_original_kills_target_red(){
    let mut combat=with_task7_card(CardId::DoubleTap,false);
    combat.state.enemies[0].hp=6;
    let mut second=combat.state.enemies[0].clone();second.instance_id=EnemyHandle::parse("enemy-2").unwrap();second.hp=60;second.max_hp=60;
    let old=second.full_ai_rng.as_ref().unwrap();let id=super::super::rng::StreamId::new(super::super::rng::Domain::RunCombatEnemyAi,
        vec!["run-combat-1".into(),"enemy-scope-2".into()]).unwrap();
    second.full_ai_rng=Some(super::super::rng::StreamState::new(id,*old.key(),old.word_index()).unwrap());
    let second_id=second.instance_id.clone();combat.state.actor_slots.insert(second_id.clone(),2);combat.state.enemies.push(second);
    combat.step_full(&CombatAction::PlayCard{instance:3,target:None}).unwrap();
    let strike=combat.state.piles.hand.iter().filter_map(OwnedCard::full).find(|card|card.id==CardId::Strike).unwrap().instance.0;
    let result=combat.step_full(&CombatAction::PlayCard{instance:strike,target:Some(TargetId(1))});
    assert!(result.is_ok(),"repeat dequeue with a dead captured target follows ordinary failed-use disposition, not a fault: {result:?}");
    assert_eq!(combat.state.outcome,None);
    assert_eq!(combat.state.enemies.iter().find(|enemy|enemy.instance_id==second_id).unwrap().hp,60,"repeat preserves the original target and never retargets the surviving enemy");
    assert!(combat.state.fault.is_none());
    assert!(combat.state.inactive_cards.iter().filter_map(OwnedCard::full).any(|card|card.copy_provenance==card_state::CopyProvenance::SameInstanceFamily),"purged queued repeat has its own disposition");
    assert!(engine::validate_runtime(&combat.state).is_ok());
}

#[test]
fn m10_whirlwind_accepts_x_zero_and_still_finishes_parent(){
    let mut combat=with_task7_card(CardId::Whirlwind,false);
    combat.state.player.energy=0;
    assert_eq!(crate::content::Catalog::standard().card(CardId::Whirlwind).unwrap().rarity,crate::content::CardRarity::Uncommon);
    let result=combat.step_full(&CombatAction::PlayCard{instance:3,target:None});
    assert!(result.is_ok(),"Whirlwind must enter the actual X-action path: {result:?}");
    assert!(combat.state.in_play.is_none());
    assert_eq!(combat.state.enemies[0].hp,48,"zero X produces no damage");
}

#[test]
fn m10_whirlwind_fails_closed_for_unrepresented_chemical_x_at_actual_use_red(){
    for energy in [0,1]{
        let mut combat=with_task7_card(CardId::Whirlwind,false);
        combat.state.player.energy=energy;
        combat.state.item_context=Some(crate::items::CombatItemContext::new(vec![crate::content::RelicId::Unknown("chemical_x")],vec![]));
        let result=combat.step_full(&CombatAction::PlayCard{instance:3,target:None});
        assert!(result.is_err(),"unsupported modifier must fail closed when it would alter X={energy}");
        assert_eq!(combat.state.player.energy,energy,"fail-closed use preserves energy");
        assert!(combat.state.piles.hand.iter().any(|owned|owned.handle()==card_state::handle(CardInstanceId(3))),"fail-closed use leaves the card in hand");
        assert!(combat.state.fault.as_ref().is_some_and(|fault|fault.kind==FaultKind::Unsupported),"unsupported modifier is an honest retained unsupported fault");
    }
}

#[test]
fn m10_task7_literal_costs_and_public_values(){
    let expected=[(CardId::Havoc,1,0,None,None),(CardId::DoubleTap,1,1,Some(1),Some(2)),
        (CardId::Whirlwind,0,0,Some(5),Some(8))];
    let catalog=crate::content::Catalog::standard();
    for (id,rarity,card_type) in [(CardId::Havoc,crate::content::CardRarity::Common,crate::content::CardType::Skill),
        (CardId::DoubleTap,crate::content::CardRarity::Rare,crate::content::CardType::Skill),
        (CardId::Whirlwind,crate::content::CardRarity::Uncommon,crate::content::CardType::Attack)]{
        let definition=catalog.card(id).unwrap();
        assert_eq!((definition.rarity,definition.card_type,definition.definition_only),(rarity,card_type,false),"literal supported catalog row for {id:?}");
    }
    for (id,cost,upgraded_cost,damage,upgraded_damage) in expected{
        for (upgrades,want_cost,want_damage) in [(0,cost,damage),(1,upgraded_cost,upgraded_damage)]{
            let mut combat=with_task7_card(id,false);
            {let card=combat.state.piles.hand.iter_mut().filter_map(OwnedCard::full_mut)
                .find(|card|card.instance.0==3).unwrap();
            card.id=id;card.upgrades=upgrades;
            }
            let card=combat.state.piles.hand.iter().filter_map(OwnedCard::full)
                .find(|card|card.instance.0==3).unwrap();
            assert_eq!(super::super::cards::base_cost(&combat.state,card),Some(want_cost),"base cost for {id:?} upgrades={upgrades}");
            let values=super::super::cards::ironclad::rules::public_values(card).unwrap();
            if id==CardId::Whirlwind{
                assert_eq!(values.base_damage,want_damage);
                assert_eq!(values.cost_kind,CardCostKind::X);
            }else if id==CardId::DoubleTap{
                assert_eq!(values.magic_number,want_damage,"Double Tap charges are public magic number for both upgrade states");
            }
        }
    }
}

#[test]
fn m10_dual_wield_eligibility_uses_double_tap_skill_literal(){
    let mut combat=with_task7_card(CardId::DualWield,false);
    combat.state.piles.hand.iter_mut().filter_map(OwnedCard::full_mut).find(|card|card.instance.0==1).unwrap().id=CardId::Strike;
    combat.state.piles.hand.iter_mut().filter_map(OwnedCard::full_mut).find(|card|card.instance.0==2).unwrap().id=CardId::Inflame;
    combat.state.card_ids_issued.insert(card_state::handle(CardInstanceId(80)));
    combat.state.piles.hand.push(OwnedCard::Full(card_state::permanent(CardId::DoubleTap,CardInstanceId(80),0)));
    assert_eq!(crate::content::Catalog::standard().card(CardId::DoubleTap).unwrap().card_type,crate::content::CardType::Skill);
    combat.step_full(&CombatAction::PlayCard{instance:3,target:None}).unwrap();
    assert_eq!(combat.state.phase,Phase::Selection);
    assert!(!combat.state.pending_selection.as_ref().unwrap().candidates.contains(&card_state::handle(CardInstanceId(80))),
        "Rare Double Tap Skill must not be offered by Dual Wield's Attack/Power choice");
}

#[test]
fn m10_whirlwind_x_cost_survives_fresh_generated_and_stat_equivalent_constructors_red(){
    let permanent=card_state::permanent(CardId::Whirlwind,CardInstanceId(90),0);
    assert!(permanent.local.x_cost,"permanent X card descriptor must carry X status");
    let generated=card_state::generated(CardId::Whirlwind,CardInstanceId(91),0).expect("Whirlwind descriptor supports fresh constructor");
    assert!(generated.local.x_cost,"generated X card descriptor must carry X status");
    let copied=permanent.stat_equivalent_copy(CardInstanceId(92));
    assert!(copied.local.x_cost,"stat-equivalent copy reconstructs X status from card identity");
}

#[test]
fn m10_havoc_unplayable_clash_is_suppressed_and_exhausted_without_fault_red(){
    let mut combat=with_task7_card(CardId::Havoc,false);
    combat.state.player_full_powers.push(FullPowerEntry{id:FullPowerId("feel_no_pain"),state:PowerState{amount:3,skip_first:false,monster_source_grace:false}});
    let child_index=combat.state.piles.hand.iter().position(|owned|owned.handle()!=card_state::handle(CardInstanceId(3))).unwrap();
    let mut child=combat.state.piles.hand.remove(child_index);
    let child_id=child.handle();let child_instance=child.full().unwrap().instance;
    child.full_mut().unwrap().id=CardId::Clash;
    for held in &mut combat.state.piles.hand{if held.handle()!=card_state::handle(CardInstanceId(3)){held.full_mut().unwrap().id=CardId::Defend;}}
    combat.state.piles.draw.clear();combat.state.piles.draw.push(child);
    assert!(combat.state.piles.hand.iter().filter_map(OwnedCard::full).any(|card|crate::content::Catalog::standard().card(card.id).unwrap().card_type==crate::content::CardType::Skill));
    let hp=combat.state.enemies[0].hp;
    let result=combat.step_full(&CombatAction::PlayCard{instance:3,target:None});
    assert!(result.is_ok(),"supported-but-unplayable autoplay child follows suppressed-use cleanup, not unsupported fault: {result:?}");
    assert_eq!(combat.state.enemies[0].hp,hp);
    assert!(combat.state.piles.exhaust.iter().any(|owned|owned.handle()==child_id),"child disposition exhaust={} discard={} inactive={} draw={} queued={} in_play={:?} frames={} fault={:?}",combat.state.piles.exhaust.iter().any(|owned|owned.handle()==child_id),combat.state.piles.discard.iter().any(|owned|owned.handle()==child_id),combat.state.inactive_cards.iter().any(|owned|owned.handle()==child_id),combat.state.piles.draw.iter().any(|owned|owned.handle()==child_id),combat.state.queued_autoplay.contains_key(&child_id),combat.state.in_play.as_ref().map(OwnedCard::handle),combat.state.play_frames.len(),combat.state.fault);
    assert_eq!(combat.state.player.block,3,"suppressed Havoc child still calls the normal exhaust callback");
    assert_eq!(combat.state.hook_events.iter().filter(|event|event.hook==Hook::CardExhausted&&event.source==Source::Card(child_id.clone())).count(),1);
    assert!(combat.state.piles.discard.iter().any(|owned|owned.full().is_some_and(|card|card.instance==CardInstanceId(3))));
    assert!(!combat.state.hook_events.iter().any(|event|event.hook==Hook::CardPlayed&&event.source==Source::Card(child_id.clone())));
    assert!(engine::validate_runtime(&combat.state).is_ok());
    assert!(combat.state.card_ids_issued.contains(&card_state::handle(child_instance)));
}

#[test]
fn m10_havoc_entangled_attack_is_suppressed_without_using_card_red(){
    let mut combat=with_task7_card(CardId::Havoc,false);
    let child_index=combat.state.piles.hand.iter().position(|owned|owned.handle()!=card_state::handle(CardInstanceId(3))).unwrap();
    let mut child=combat.state.piles.hand.remove(child_index);
    child.full_mut().unwrap().id=CardId::Strike;let child_handle=child.handle();
    combat.state.piles.draw.push(child);
    combat.state.player_full_powers.push(FullPowerEntry{id:FullPowerId("entangled"),state:PowerState{amount:1,skip_first:false,monster_source_grace:false}});
    let hp=combat.state.enemies[0].hp;
    let result=combat.step_full(&CombatAction::PlayCard{instance:3,target:None});
    assert!(result.is_ok(),"Entangled is ordinary canUse failure for autoplay child: {result:?}");
    assert_eq!(combat.state.enemies[0].hp,hp);
    assert!(combat.state.piles.exhaust.iter().any(|owned|owned.handle()==child_handle));
    assert!(!combat.state.hook_events.iter().any(|event|event.hook==Hook::CardPlayed&&event.source==Source::Card(child_handle.clone())));
}

#[test]
fn m10_havoc_unplayable_curse_has_no_payload_hook_or_end_turn_trigger_red(){
    let mut combat=with_task7_card(CardId::Havoc,false);
    let child=OwnedCard::Full(card_state::permanent(CardId::Regret,CardInstanceId(77),0));let child_handle=child.handle();
    combat.state.card_ids_issued.insert(child_handle.clone());combat.state.piles.draw.push(child);
    let hp=combat.state.player.hp;
    combat.step_full(&CombatAction::PlayCard{instance:3,target:None}).unwrap();
    assert_eq!(combat.state.player.hp,hp,"Curse is not played and emits no end-turn payload");
    assert!(combat.state.piles.exhaust.iter().any(|owned|owned.handle()==child_handle));
    assert!(!combat.state.hook_events.iter().any(|event|event.hook==Hook::CardPlayed&&event.source==Source::Card(child_handle.clone())));
}

#[test]
fn m10_havoc_unsupported_nested_child_fault_keeps_parent_context_and_child_owner_red(){
    let mut combat=with_task7_card(CardId::Havoc,false);
    let havoc=combat.state.piles.hand.iter().find(|owned|owned.handle()==card_state::handle(CardInstanceId(3))).unwrap().clone();
    let child=OwnedCard::Full(card_state::permanent(CardId::DeadlyPoison,CardInstanceId(78),0));let child_handle=child.handle();
    combat.state.card_ids_issued.insert(child_handle.clone());combat.state.piles.draw.clear();combat.state.piles.draw.push(child);
    assert!(crate::content::Catalog::standard().card(CardId::DeadlyPoison).unwrap().definition_only,"fixture is a registered definition-only card with no M10 payload handler");
    let (_,expected_rng)=super::super::rng::uniform_below(combat.state.card_rng.as_ref().unwrap(),1).unwrap();
    let result=combat.step_full(&CombatAction::PlayCard{instance:3,target:None});
    assert!(result.is_err(),"a genuinely unsupported child handler remains an honest retained fault");
    assert!(combat.state.fault.as_ref().is_some_and(|fault|fault.kind==FaultKind::Unsupported));
    let frame=combat.state.play_frames.last().expect("completed parent is retained in the active child frame before child preparation");
    assert_eq!(frame.parent.as_ref().unwrap().handle(),card_state::handle(CardInstanceId(3)),"parent remains owned while child preparation faults");
    assert_eq!(frame.parent_context.as_ref().unwrap().card,card_state::handle(CardInstanceId(3)),"parent play context remains owned by the frame");
    assert_eq!(frame.child,child_handle);
    assert!(combat.state.queued_autoplay.contains_key(&child_handle),"child remains in queued ownership, not dropped into a local variable");
    assert_eq!(combat.state.card_rng.as_ref(),Some(&expected_rng),"an unsupported extracted child is never accepted as a second Havoc use and spends no second target draw");
    assert!(combat.state.card_ids_issued.contains(&child_handle));
    assert!(!combat.state.piles.all(combat.state.in_play.as_ref()).any(|owned|owned.handle()==havoc.handle()),"parent has exactly one owner in the frame");
    assert!(engine::validate_runtime(&combat.state).is_ok(),"a well-owned deferred child is a valid retained fault state");
    let mut dangling=combat.state.clone();dangling.play_frames.last_mut().unwrap().child=card_state::handle(CardInstanceId(999));
    assert!(engine::validate_runtime(&dangling).is_err(),"fault diagnostic exemption must not admit dangling play-frame references");
}

#[test]
fn m10_whirlwind_spends_at_action_checkpoint_before_queued_hit_red(){
    let mut combat=with_task7_card(CardId::Whirlwind,false);
    combat.state.player.energy=1;
    let enemy=combat.state.enemies[0].instance_id.clone();let hp=combat.state.enemies[0].hp;
    let source=Source::Card(card_state::handle(CardInstanceId(3)));
    combat.state.pending.push_front(Effect{source:source.clone(),target:None,kind:EffectKind::Ironclad(IroncladEffect::WhirlwindRound{remaining:2,hits:vec![(enemy.clone(),5)],spend_current:true})});
    engine::dispatch_one(&mut combat.state);
    assert_eq!(combat.state.player.energy,0,"two captured hit rounds are independent of the one current energy spent at Whirlwind's action checkpoint");
    assert_eq!(combat.state.enemies[0].hp,hp,"damage is still queued when energy payment commits");
    assert!(matches!(combat.state.pending.front().unwrap().kind,EffectKind::Damage(DamagePayload::Fixed(5))));
}

#[test]
fn m10_whirlwind_lethals_first_thorns_target_after_checkpoint_payment_red(){
    let mut entry=super::m8_shapes::single(crate::content::EnemyId::Spiker);
    entry.cards[0]=permanent_live_card(CardId::Whirlwind,CardInstanceId(3),0);
    let mut combat=Combat::enter(entry).unwrap();
    combat.state.player.energy=2;combat.state.enemies[0].hp=5;combat.state.enemies[0].block=0;
    let player_hp=combat.state.player.hp;
    assert!(engine::validate_runtime(&combat.state).is_ok());
    combat.step_full(&CombatAction::PlayCard{instance:3,target:None}).unwrap();
    assert_eq!(combat.state.enemies[0].hp,0,"first captured Whirlwind hit is lethal");
    assert_eq!(combat.state.player.hp,player_hp-3,"Spiker's real THORNS retaliates on the lethal hit");
    assert_eq!(combat.state.player.energy,0,"energy committed at the action checkpoint survives lethal retaliation");
    assert!(engine::validate_runtime(&combat.state).is_ok());
}

#[test]
fn m10_double_tap_two_charges_decrement_at_card_played_checkpoint_red(){
    let mut combat=with_task7_card(CardId::DoubleTap,true);
    combat.step_full(&CombatAction::PlayCard{instance:3,target:None}).unwrap();
    combat.state.player_full_powers.iter_mut().find(|power|power.id.0=="double_tap").unwrap().state.amount=2;
    let strike_index=combat.state.piles.hand.iter().position(|owned|owned.full().is_some_and(|card|card.id==CardId::Strike)).unwrap();
    let strike=combat.state.piles.hand.remove(strike_index);let handle=strike.handle();
    combat.state.in_play=Some(strike);
    let target=Actor::Enemy(combat.state.enemies[0].instance_id.clone());
    combat.state.play_context=Some(super::super::model::PlayContext{card:handle.clone(),target:Some(target.clone()),energy_on_use:2,paid:1,
        disposition:super::super::model::FinishDisposition::Natural});
    combat.state.pending.push_back(Effect{source:Source::Card(handle.clone()),target:None,kind:EffectKind::FinishCard});
    let work:Vec<Effect>=Vec::new();
    super::super::hooks::dispatch(&mut combat.state,Hook::CardPlayed,&Source::Card(handle),Some(&target)).unwrap();
    let immediate=combat.state.player_full_powers.iter().find(|power|power.id.0=="double_tap").unwrap().state.amount;
    assert_eq!(immediate,1,"Double Tap decrements synchronously at CardPlayed before queued repeat work");
    assert!(work.is_empty(),"charge decrement is synchronous; repeat is attached to the parent's FinishCard queue");
    assert!(matches!(combat.state.pending.get(1).map(|effect|&effect.kind),Some(EffectKind::Ironclad(IroncladEffect::DoubleTapRepeat{..}))));
}

#[test]
fn m10_double_tap_headbutt_repeat_owns_parent_through_nested_selection_red(){
    let mut combat=with_task7_card(CardId::DoubleTap,false);
    combat.state.player.energy=3;
    combat.state.card_ids_issued.insert(card_state::handle(CardInstanceId(80)));
    combat.state.piles.hand.push(OwnedCard::Full(card_state::permanent(CardId::Headbutt,CardInstanceId(80),0)));
    combat.state.card_ids_issued.insert(card_state::handle(CardInstanceId(81)));
    combat.state.piles.discard.push(OwnedCard::Full(card_state::permanent(CardId::Strike,CardInstanceId(81),0)));
    combat.state.card_ids_issued.insert(card_state::handle(CardInstanceId(90)));
    combat.state.piles.discard.push(OwnedCard::Full(card_state::permanent(CardId::Strike,CardInstanceId(90),0)));
    combat.step_full(&CombatAction::PlayCard{instance:3,target:None}).unwrap();
    combat.step_full(&CombatAction::PlayCard{instance:80,target:Some(TargetId(1))}).unwrap();
    assert_eq!(combat.state.phase,Phase::Selection);
    let first=combat.state.pending_selection.as_ref().unwrap().candidates[0].as_str().parse().unwrap();
    combat.step_full(&CombatAction::SelectCard{instance:first}).unwrap();
    assert_eq!(combat.state.phase,Phase::Selection,"Double Tap's Headbutt repeat must execute its own source selection: fault={:?} hp={} inactive={:?} in_play={:?} frames={} pending={:?}",combat.state.fault,combat.state.enemies[0].hp,combat.state.inactive_cards.iter().map(OwnedCard::handle).collect::<Vec<_>>(),combat.state.in_play.as_ref().map(OwnedCard::handle),combat.state.play_frames.len(),combat.state.pending);
    let frame=combat.state.play_frames.last().expect("repeat retains its original parent frame");
    assert_eq!(frame.parent_handle,card_state::handle(CardInstanceId(80)));
    assert_ne!(frame.child,card_state::handle(CardInstanceId(80)),"repeat is a distinct issued card instance");
    let playing=combat.public_full().unwrap().playing_cards;
    assert_eq!(playing.iter().map(|card|card.id).collect::<Vec<_>>(),vec![CardId::Headbutt,CardId::Headbutt]);
    assert_ne!(playing[0].instance,playing[1].instance,"public playing owners expose parent and started repeat as distinct issued instances");
    engine::validate_runtime(&combat.state).unwrap();
}

#[test]
fn m10_headbutt_repeat_budget_fault_retains_accepted_selection_owners_red(){
    let mut combat=with_task7_card(CardId::DoubleTap,false);
    combat.state.player.energy=3;
    for (id,card,zone) in [(80,CardId::Headbutt,0),(81,CardId::Strike,2),(90,CardId::Strike,2)]{
        let owned=OwnedCard::Full(card_state::permanent(card,CardInstanceId(id),0));
        combat.state.card_ids_issued.insert(owned.handle());
        match zone{0=>combat.state.piles.hand.push(owned),_=>combat.state.piles.discard.push(owned)}
    }
    combat.step_full(&CombatAction::PlayCard{instance:3,target:None}).unwrap();
    combat.step_full(&CombatAction::PlayCard{instance:80,target:Some(TargetId(1))}).unwrap();
    assert_eq!(combat.state.phase,Phase::Selection,"original Headbutt opens its actual discard selection");
    let first=combat.state.pending_selection.as_ref().unwrap().candidates[0].as_str().parse().unwrap();
    combat.step_full(&CombatAction::SelectCard{instance:first}).unwrap();
    assert_eq!(combat.state.phase,Phase::Selection,"repeated Headbutt has offered and staged its own selection");
    let selection=combat.state.pending_selection.as_ref().unwrap().clone();
    let frame=combat.state.play_frames.clone();let in_play=combat.state.in_play.clone();
    combat.state.resolved_effects=99_999; // valid immediately before accepted selection commit
    assert!(engine::validate_runtime(&combat.state).is_ok());
    let chosen=selection.candidates[0].as_str().parse().unwrap();
    let result=combat.step_full(&CombatAction::SelectCard{instance:chosen});
    assert!(result.is_err(),"accepted move hits the existing 100,000-dispatch budget guard");
    assert!(combat.state.fault.as_ref().is_some_and(|fault|fault.kind==FaultKind::EffectGuard));
    assert_eq!(combat.state.fault_selection.as_ref().unwrap().selected,vec![selection.candidates[0].clone()]);
    assert_eq!(combat.state.play_frames,frame,"fault keeps every parent/child continuation owner");
    assert_eq!(combat.state.in_play,in_play,"started repeat remains owned after the accepted move faults");
    assert_eq!(combat.state.piles.draw.last().map(OwnedCard::handle),Some(selection.candidates[0].clone()),"accepted Headbutt selection remains at its committed topdeck destination");
    assert_eq!(combat.state.public.known_draw.first(),Some(&selection.candidates[0]));
    assert_eq!(combat.state.pending_selection,None);
    let frozen=combat.state.clone();assert!(combat.step_full(&CombatAction::SelectCard{instance:chosen}).is_err());assert_eq!(combat.state,frozen,"retained fault cannot retry or mutate the selection");
}

#[test]
fn m10_upgraded_double_tap_has_two_independent_nonrecursive_purged_repeats(){
    let mut combat=with_task7_card(CardId::DoubleTap,true);
    combat.state.player.energy=4;
    let mut ids=combat.state.piles.hand.iter().filter_map(OwnedCard::full).filter(|card|card.instance.0!=3).map(|card|card.instance.0).collect::<Vec<_>>();
    for instance in 80..=80{
        combat.state.card_ids_issued.insert(card_state::handle(CardInstanceId(instance)));
        combat.state.piles.hand.push(OwnedCard::Full(card_state::permanent(CardId::Strike,CardInstanceId(instance),0)));
        ids.push(instance);
    }
    for instance in &ids{combat.state.piles.hand.iter_mut().filter_map(OwnedCard::full_mut).find(|card|card.instance.0==*instance).unwrap().id=CardId::Strike;}
    let families=ids.iter().take(3).map(|instance|(*instance,*instance)).collect::<Vec<_>>();
    combat.step_full(&CombatAction::PlayCard{instance:3,target:None}).unwrap();
    for instance in ids.iter().take(3){combat.step_full(&CombatAction::PlayCard{instance:*instance,target:Some(TargetId(1))}).unwrap();}
    assert_eq!(combat.state.enemies[0].hp,48-30,"two originals repeat once each; the third is not doubled");
    assert!(!combat.state.player_full_powers.iter().any(|power|power.id.0=="double_tap"));
    let repeats=combat.state.inactive_cards.iter().filter_map(OwnedCard::full).filter(|card|card.copy_provenance==card_state::CopyProvenance::SameInstanceFamily).collect::<Vec<_>>();
    assert_eq!(repeats.len(),2);
    assert!(repeats.iter().all(|copy|copy.local.free_play&&copy.local.purge&&!copy.local.exhaust_on_finish));
    assert!(repeats.iter().all(|copy|!combat.state.hook_events.iter().any(|event|
        event.hook==Hook::CardExhausted&&event.source==Source::Card(card_state::handle(copy.instance)))),
        "purged repeat copies do not enter exhaust callbacks");
    assert_eq!(repeats.iter().map(|copy|copy.copy_family).collect::<std::collections::BTreeSet<_>>(),families.into_iter().take(2).map(|(_,family)|family).collect());
    assert!(combat.state.piles.discard.iter().any(|owned|owned.full().is_some_and(|card|card.id==CardId::DoubleTap&&card.upgrades==1)));
    assert!(!combat.state.played_powers.iter().any(|owned|owned.full().is_some_and(|card|card.id==CardId::DoubleTap)));
    assert!(engine::validate_runtime(&combat.state).is_ok());
}

#[test]
fn m10_unused_double_tap_charge_expires_at_its_owning_player_end_turn(){
    let mut combat=with_task7_card(CardId::DoubleTap,false);
    combat.step_full(&CombatAction::PlayCard{instance:3,target:None}).unwrap();
    assert_eq!(combat.state.player_full_powers.iter().find(|power|power.id.0=="double_tap").unwrap().state.amount,1);
    combat.step_full(&CombatAction::EndTurn).unwrap();
    assert!(!combat.state.player_full_powers.iter().any(|power|power.id.0=="double_tap"));
    assert_eq!(combat.state.turn,2);
    assert!(engine::validate_runtime(&combat.state).is_ok());
}

#[test]
fn m10_free_whirlwind_uses_captured_x_without_spending_current_energy(){
    let mut combat=with_task7_card(CardId::Whirlwind,false);
    combat.state.player.energy=2;
    combat.state.piles.hand.iter_mut().filter_map(OwnedCard::full_mut).find(|card|card.instance.0==3).unwrap().local.free_play=true;
    combat.step_full(&CombatAction::PlayCard{instance:3,target:None}).unwrap();
    assert_eq!(combat.state.enemies[0].hp,38,"X=2 produces two five-damage rounds");
    assert_eq!(combat.state.player.energy,2,"free X action does not spend energy");
    assert!(engine::validate_runtime(&combat.state).is_ok());
}

#[test]
fn m10_whirlwind_repeats_each_living_actor_with_per_target_weak_and_vulnerable_math(){
    let mut combat=with_task7_card(CardId::Whirlwind,false);
    let mut second=combat.state.enemies[0].clone();
    second.instance_id=EnemyHandle::parse("enemy-2").unwrap();second.hp=60;second.max_hp=60;
    let old=second.full_ai_rng.as_ref().unwrap();
    let id=super::super::rng::StreamId::new(super::super::rng::Domain::RunCombatEnemyAi,
        vec!["run-combat-1".into(),"enemy-scope-2".into()]).unwrap();
    second.full_ai_rng=Some(super::super::rng::StreamState::new(id,*old.key(),old.word_index()).unwrap());
    second.full_powers.push(FullPowerEntry{id:FullPowerId("vulnerable"),state:PowerState{amount:1,skip_first:false,monster_source_grace:false}});
    let second_id=second.instance_id.clone();combat.state.actor_slots.insert(second_id.clone(),2);combat.state.enemies.push(second);
    combat.state.player_full_powers.push(FullPowerEntry{id:FullPowerId("weak"),state:PowerState{amount:1,skip_first:false,monster_source_grace:false}});
    combat.state.player.energy=2;
    assert!(engine::validate_runtime(&combat.state).is_ok());
    assert!(combat.state.enemies.iter().find(|enemy|enemy.instance_id==second_id).unwrap().full_powers.iter().any(|power|power.id.0=="vulnerable"&&power.state.amount==1),"fixture retains the intended target-owned Vulnerable before resolution");
    let first_hp=combat.state.enemies[0].hp;
    combat.step_full(&CombatAction::PlayCard{instance:3,target:None}).unwrap();
    assert_eq!(combat.state.enemies[0].hp,first_hp-6,"two five-damage rounds reduced by player Weak");
    assert_eq!(combat.state.enemies.iter().find(|enemy|enemy.instance_id==second_id).unwrap().hp,50,"player Weak then target Vulnerable yields five per hit in native modifier order");
    assert_eq!(combat.state.player.energy,0);
    assert!(engine::validate_runtime(&combat.state).is_ok());
}

#[test]
fn m10_havoc_autoplayed_whirlwind_uses_minus_one_current_energy_fallback(){
    let mut combat=with_task7_card(CardId::Havoc,false);
    let index=combat.state.piles.hand.iter().position(|owned|owned.handle()==card_state::handle(CardInstanceId(3))).unwrap();
    let havoc=combat.state.piles.hand.remove(index);
    let child=OwnedCard::Full(card_state::permanent(CardId::Whirlwind,CardInstanceId(77),0));
    combat.state.card_ids_issued.insert(child.handle());combat.state.piles.draw.push(child);combat.state.piles.hand.push(havoc);
    let before=combat.state.enemies[0].hp;
    combat.step_full(&CombatAction::PlayCard{instance:3,target:None}).unwrap();
    assert_eq!(combat.state.enemies[0].hp,before-10,"Havoc child resolves X=-1 against current energy after Havoc payment");
    assert_eq!(combat.state.player.energy,2,"free child does not spend fallback X");
    assert!(engine::validate_runtime(&combat.state).is_ok());
}

#[test]
fn m10_twelfth_card_time_warp_cancels_queued_repeat_but_finishes_original(){
    let mut entry=super::m8_shapes::entry(crate::content::EncounterId::TimeEater);
    entry.cards=vec![permanent_live_card(CardId::Strike,CardInstanceId(1),0)];entry.opening_draw=0;
    entry.setup.enemies[0].powers.iter_mut().find(|power|power.id.0=="time_warp").unwrap().state.amount=11;
    let mut combat=Combat::enter(entry).unwrap();
    let strike=combat.state.piles.draw.pop().unwrap();combat.state.piles.hand.push(strike);
    combat.state.player_full_powers.push(FullPowerEntry{id:FullPowerId("double_tap"),state:PowerState{amount:1,skip_first:false,monster_source_grace:false}});
    let hp=combat.state.enemies[0].hp;
    combat.step_full(&CombatAction::PlayCard{instance:1,target:Some(TargetId(1))}).unwrap();
    assert_eq!(combat.state.enemies[0].hp,hp-6,"Twelfth-card barrier cancels the not-yet-started repeat payload");
    assert_eq!(combat.state.turn,2);
    assert!(combat.state.in_play.is_none()&&combat.state.play_frames.is_empty());
    assert!(combat.state.piles.all(None).any(|owned|owned.handle()==card_state::handle(CardInstanceId(1))),"original card's normal disposition remains owned through the early turn transition");
    assert!(engine::validate_runtime(&combat.state).is_ok());
}

#[test]
fn m10_twelfth_card_time_warp_cancels_havoc_payload_but_keeps_exhaust_cleanup(){
    let mut entry=super::m8_shapes::entry(crate::content::EncounterId::TimeEater);
    entry.cards=vec![permanent_live_card(CardId::Havoc,CardInstanceId(1),0),permanent_live_card(CardId::Strike,CardInstanceId(2),0)];entry.opening_draw=0;
    entry.setup.enemies[0].powers.iter_mut().find(|power|power.id.0=="time_warp").unwrap().state.amount=11;
    let mut combat=Combat::enter(entry).unwrap();
    combat.state.player.draw_per_turn=1;
    combat.state.player_full_powers.push(super::super::model::FullPowerEntry{id:super::super::FullPowerId("dark_embrace"),
        state:super::super::PowerState{amount:1,skip_first:false,monster_source_grace:false}});
    let havoc=combat.state.piles.draw.iter().position(|owned|owned.handle()==card_state::handle(CardInstanceId(1))).unwrap();
    let card=combat.state.piles.draw.remove(havoc);combat.state.piles.hand.push(card);
    let child_at=combat.state.piles.draw.iter().position(|owned|owned.handle()==card_state::handle(CardInstanceId(2))).unwrap();
    let child=combat.state.piles.draw.remove(child_at);
    let marker=OwnedCard::Full(card_state::permanent(CardId::Defend,CardInstanceId(3),0));
    combat.state.card_ids_issued.insert(marker.handle());combat.state.piles.draw.push(marker);combat.state.piles.draw.push(child);
    let before=combat.state.enemies[0].hp;
    let result=combat.step_full(&CombatAction::PlayCard{instance:1,target:None});
    assert!(result.is_ok(),"Time Warp inside Havoc must preserve nested owner cleanup: {result:?}");
    assert_eq!(combat.state.outcome,None);
    assert_eq!(combat.state.turn,2);
    assert_eq!(combat.state.enemies[0].hp,before,"12th-card Time Warp suppresses queued Havoc child payload");
    assert!(combat.state.piles.exhaust.iter().any(|owned|owned.handle()==card_state::handle(CardInstanceId(2))),"Havoc child still exhausts after its payload is canceled");
    assert!(combat.state.piles.hand.iter().chain(&combat.state.piles.discard)
        .any(|owned|owned.handle()==card_state::handle(CardInstanceId(3))),
        "the issued card drawn by Dark Embrace remains in ordinary player custody across the turn boundary");
    assert_eq!(combat.state.hook_events.iter().filter(|event|event.hook==Hook::CardExhausted&&event.source==Source::Card(card_state::handle(CardInstanceId(2)))).count(),1);
    assert!(combat.state.piles.all(None).any(|owned|owned.handle()==card_state::handle(CardInstanceId(1))),"Havoc parent completes normal disposition before forced end-turn");
    assert!(combat.state.play_frames.is_empty()&&combat.state.in_play.is_none());
    engine::validate_runtime(&combat.state).expect("Time Warp exhaust callback must leave a valid final runtime boundary");
}

#[test]
fn m10_whirlwind_real_x_payment_and_per_target_hits_red(){
    let mut combat=with_task7_card(CardId::Whirlwind,false);
    combat.state.player.energy=2;
    assert!(combat.state.piles.hand.iter().filter_map(OwnedCard::full)
        .find(|card|card.instance.0==3).unwrap().local.x_cost,"the real permanent card constructor supplies X-cost status");
    let before=combat.state.enemies.iter().map(|enemy|enemy.hp).collect::<Vec<_>>();
    let result=combat.step_full(&CombatAction::PlayCard{instance:3,target:None});
    assert!(result.is_ok(),"Whirlwind's X action resolves through the real combat queue: {result:?}");
    assert_eq!(combat.state.player.energy,0);
    assert_eq!(combat.state.enemies.iter().map(|enemy|enemy.hp).collect::<Vec<_>>(),
        before.into_iter().map(|hp|hp-10).collect::<Vec<_>>());
    assert!(engine::validate_runtime(&combat.state).is_ok());
}

#[test]
fn m10_havoc_nested_selection_keeps_parent_reference_and_started_child_public(){
    let mut combat=with_task7_card(CardId::Havoc,false);
    let index=combat.state.piles.hand.iter().position(|owned|owned.handle()==card_state::handle(CardInstanceId(3))).unwrap();
    let havoc=combat.state.piles.hand.remove(index);
    let mut child=havoc.clone();
    *child.full_mut().unwrap()=card_state::permanent(CardId::Armaments,CardInstanceId(77),0);
    combat.state.card_ids_issued.insert(child.handle());
    combat.state.piles.draw.push(child);
    combat.state.piles.hand.push(havoc);
    let source_cards=combat.state.piles.hand.iter().filter_map(OwnedCard::full).filter(|card|card.instance.0!=3)
        .map(|card|card.instance.0).collect::<Vec<_>>();
    combat.step_full(&CombatAction::PlayCard{instance:3,target:None}).unwrap();
    assert_eq!(combat.state.phase,Phase::Selection);
    engine::validate_runtime(&combat.state).unwrap();
    let public=combat.public_full().unwrap();
    assert_eq!(public.playing_cards.iter().map(|card|card.id).collect::<Vec<_>>(),vec![CardId::Havoc,CardId::Armaments]);
    assert_eq!(combat.state.play_frames.len(),1);
    let encoded=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||super::differential::encode_checkpoint(&combat)));
    let panic=encoded.expect_err("M9 v1 checkpoint codec rejects active M10 play-frame ownership");
    let message=panic.downcast_ref::<String>().map(String::as_str).or_else(||panic.downcast_ref::<&str>().copied()).unwrap_or("");
    assert!(message.contains("M9 v1 cannot encode an active or retained M10 autoplay play frame"),"specific v1 ownership boundary: {message}");
    assert_eq!(combat.state.play_frames[0].parent_handle,card_state::handle(CardInstanceId(3)));
    assert!(!combat.state.piles.discard.iter().any(|owned|owned.handle()==card_state::handle(CardInstanceId(3))),"suspended parent remains in the retained play frame until its child settles");
    combat.step_full(&CombatAction::SelectCard{instance:source_cards[0]}).unwrap();
    assert!(combat.state.play_frames.is_empty());
    assert!(combat.state.in_play.is_none());
    assert!(combat.state.piles.discard.iter().any(|owned|owned.full().is_some_and(|card|card.instance.0==3&&card.id==CardId::Havoc)));
    assert!(combat.state.piles.exhaust.iter().any(|owned|owned.full().is_some_and(|card|card.instance.0==77&&card.id==CardId::Armaments)));
    assert!(engine::validate_runtime(&combat.state).is_ok());
}

#[test]
fn m10_havoc_recycles_without_draw_callback_and_suppresses_unplayable_status(){
    let mut combat=with_task7_card(CardId::Havoc,false);
    let child=OwnedCard::Full(card_state::permanent(CardId::Dazed,CardInstanceId(77),0));
    combat.state.card_ids_issued.insert(child.handle());combat.state.piles.discard.push(child);
    combat.state.piles.draw.clear();
    combat.state.player_full_powers.push(FullPowerEntry{id:FullPowerId("confused"),state:PowerState{amount:1,skip_first:false,monster_source_grace:false}});
    assert!(engine::validate_runtime(&combat.state).is_ok());
    combat.step_full(&CombatAction::PlayCard{instance:3,target:None}).unwrap();
    assert!(combat.state.piles.exhaust.iter().any(|owned|owned.full().is_some_and(|card|card.instance.0==77&&card.id==CardId::Dazed)));
    assert!(!combat.state.hook_events.iter().any(|event|event.hook==Hook::CardDrawn&&event.source==Source::Card(card_state::handle(CardInstanceId(77)))));
    assert!(!combat.state.hook_events.iter().any(|event|event.hook==Hook::CardPlayed&&event.source==Source::Card(card_state::handle(CardInstanceId(77)))));
    assert_eq!(combat.state.player_full_powers.iter().find(|power|power.id.0=="confused").unwrap().state.amount,1);
    assert!(combat.state.in_play.is_none());
    assert!(engine::validate_runtime(&combat.state).is_ok());
}

#[test]
fn m10_terminal_during_autoplay_exports_both_real_play_owners(){
    let mut combat=with_task7_card(CardId::Havoc,false);
    combat.state.player.hp=6;
    let child=OwnedCard::Full(card_state::permanent(CardId::Offering,CardInstanceId(77),0));
    combat.state.card_ids_issued.insert(child.handle());combat.state.piles.draw.push(child);
    assert!(engine::validate_runtime(&combat.state).is_ok());
    combat.step_full(&CombatAction::PlayCard{instance:3,target:None}).unwrap();
    assert_eq!(combat.state.outcome,Some(Outcome::Death));
    assert_eq!(combat.state.play_frames.len(),1);
    let exit=combat.exit().unwrap();
    assert_eq!(exit.in_play.as_ref().unwrap().instance,CardInstanceId(77));
    assert_eq!(exit.play_frames.iter().map(|card|card.instance).collect::<Vec<_>>(),vec![CardInstanceId(3)]);
    assert!(exit.play_frames.iter().any(|card|card.instance==CardInstanceId(3)),"Havoc parent remains a real owner in the suspended terminal frame");
    let owners=exit.hand.iter().chain(&exit.draw).chain(&exit.discard).chain(&exit.exhaust).chain(exit.in_play.iter()).chain(&exit.play_frames).chain(&exit.queued_autoplay).map(|card|card.instance).collect::<std::collections::BTreeSet<_>>();
    assert_eq!(owners,combat.state.card_ids_issued.iter().map(|handle|CardInstanceId(handle.as_str().parse().unwrap())).collect());
}

#[test]
fn m10_run_exit_census_retains_queued_purged_double_tap_repeat_on_lethal_thorns(){
    let mut entry=super::m8_shapes::single(crate::content::EnemyId::Spiker);
    entry.cards[0]=permanent_live_card(CardId::Strike,CardInstanceId(1),0);
    let mut combat=Combat::enter(entry).unwrap();
    combat.state.player.hp=3;combat.state.player_full_powers.push(FullPowerEntry{id:FullPowerId("double_tap"),state:PowerState{amount:1,skip_first:false,monster_source_grace:false}});
    assert!(engine::validate_runtime(&combat.state).is_ok());
    combat.step_full(&CombatAction::PlayCard{instance:1,target:Some(TargetId(1))}).unwrap();
    assert_eq!(combat.state.outcome,Some(Outcome::Death),"real Spiker THORNS kills after original Strike damage");
    let exit=combat.exit().unwrap();
    assert_eq!(exit.in_play.as_ref().map(|card|card.instance),Some(CardInstanceId(1)));
    assert_eq!(exit.queued_autoplay.len(),1,"unstarted repeat remains a real run-exit owner");
    assert!(exit.queued_autoplay[0].local.purge&&exit.queued_autoplay[0].local.free_play);
    let owners=exit.hand.iter().chain(&exit.draw).chain(&exit.discard).chain(&exit.exhaust).chain(exit.in_play.iter()).chain(&exit.play_frames).chain(&exit.queued_autoplay).map(|card|card.instance).collect::<std::collections::BTreeSet<_>>();
    assert_eq!(owners,combat.state.card_ids_issued.iter().map(|handle|CardInstanceId(handle.as_str().parse().unwrap())).collect());
}
