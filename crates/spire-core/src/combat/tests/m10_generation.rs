use super::super::*;
use super::super::model::OwnedCard;
use crate::{content::{CardId,CardRarity,CardType},combat::{card_state,tests::full_runtime}};

fn real_infernal(upgrades:u32)->Combat{
    full_runtime::entered_combat(1,1,|card|*card=card_state::permanent(CardId::InfernalBlade,card.instance,upgrades))
}

fn stage_real_play(combat:&mut Combat,action:&CombatAction)->Vec<Effect>{
    let prepared=cards::prepare(combat,action).expect("real legal play prepares");
    let handle=card_state::handle(prepared.instance);
    let source=Source::Card(handle.clone());
    let card=combat.state.piles.hand.remove(prepared.hand_index);
    combat.state.in_play=Some(card);
    combat.state.player.energy=prepared.remaining_energy;
    combat.state.phase=Phase::PlayerAction;
    combat.state.play_context=Some(model::PlayContext{card:handle,target:prepared.target.clone(),
        energy_on_use:prepared.captured_energy,paid:prepared.paid,disposition:model::FinishDisposition::Natural});
    let mut work=vec![Effect{source:source.clone(),target:None,kind:EffectKind::Hook(Hook::CardPlayed)}];
    work.extend(prepared.effects);
    work.push(Effect{source,target:None,kind:EffectKind::FinishCard});
    work.push(Effect{source:Source::None,target:None,kind:EffectKind::Decision});
    combat.state.pending.extend(work.iter().cloned());
    work
}

fn infernal_action(combat:&Combat)->CombatAction{
    CombatAction::PlayCard{instance:combat.state.piles.hand.iter().filter_map(OwnedCard::full)
        .find(|card|card.id==CardId::InfernalBlade).expect("actual Infernal Blade in hand").instance.0,target:None}
}

#[test]
fn m10_infernal_blade_generates_a_real_temporary_zero_cost_attack(){
    let mut combat=real_infernal(0);
    let parent=combat.state.piles.hand.iter().find_map(|owned|owned.full().filter(|card|card.id==CardId::InfernalBlade).map(|card|card.instance.0)).unwrap();
    let definition=crate::content::Catalog::standard().card(CardId::InfernalBlade).unwrap();
    assert_eq!((definition.rarity,definition.card_type),(CardRarity::Uncommon,CardType::Skill));
    let source_rng=combat.state.card_rng.clone();
    let unrelated_rng=combat.state.rng.clone();
    combat.step_full(&CombatAction::PlayCard{instance:parent,target:None}).expect("Infernal Blade must dispatch a real generated-card effect");
    assert_ne!(combat.state.card_rng,source_rng,"Infernal Blade consumes only its owned card-generation stream");
    assert_eq!(combat.state.rng,unrelated_rng,"generation must not advance shuffle/enemy streams");
    assert!(engine::validate_runtime(&combat.state).is_ok());
    let generated=combat.state.piles.hand.iter().chain(&combat.state.piles.discard).filter_map(OwnedCard::full)
        .find(|card|card.instance.0!=parent&&crate::content::Catalog::standard().card(card.id).is_some_and(|row|row.color==crate::content::CardColor::Red&&row.card_type==CardType::Attack&&row.rarity!=CardRarity::Basic))
        .expect("generated Attack is preserved in hand or overflow discard");
    assert_eq!(combat.public_full().unwrap().hand.iter().find(|view|view.instance.0==generated.instance.0).unwrap().effective_cost,Some(0));
    assert_eq!(generated.upgrades,0,"generation creates the unupgraded eligible attack");
    assert!(!generated.local.free_play,"turn discount is not a free-play flag");
    assert!(!generated.local.purge);
    assert!(combat.state.piles.exhaust.iter().any(|owned|owned.full().is_some_and(|card|card.instance.0==parent)),"source exhausts");
    let generated_id=generated.instance.0;let generated_card_id=generated.id;
    let enemy_hp_before=combat.state.enemies.iter().map(|enemy|enemy.hp).sum::<i32>();
    let play=combat.legal_actions_full().unwrap().into_iter().find(|action|matches!(action,CombatAction::PlayCard{instance,target,..} if *instance==generated_id&&target.is_some())).expect("generated ordinary Attack must be playable with an enemy target");
    combat.step_full(&play).expect("sampled Infernal Blade result executes its real card body");
    let enemy_hp_after=combat.state.enemies.iter().map(|enemy|enemy.hp).sum::<i32>();
    assert!(enemy_hp_after<enemy_hp_before,"generated {:?} must execute its actual attack outcome",generated_card_id);
}

#[test]
fn m10_infernal_base_and_upgraded_source_play_costs_and_generate_exactly_one_attack(){
    for (upgrades,paid) in [(0,1),(1,0)]{
        let mut combat=real_infernal(upgrades);
        assert!(engine::validate_runtime(&combat.state).is_ok());
        combat.step_full(&infernal_action(&combat)).expect("actual base/upgraded card play");
        assert_eq!(combat.state.player.energy,1-paid,"upgrading changes source payment only");
        let generated=combat.state.piles.hand.iter().chain(&combat.state.piles.discard).filter_map(OwnedCard::full)
            .filter(|card|card.copy_provenance==card_state::CopyProvenance::Generated
                &&crate::run::rooms::m10_content::infernal_candidates().contains(&card.id)).collect::<Vec<_>>();
        assert_eq!(generated.len(),1,"both source upgrade states generate exactly one eligible Attack");
        assert_eq!(generated[0].upgrades,0);
        assert!(combat.state.piles.exhaust.iter().any(|owned|owned.full().is_some_and(|card|card.id==CardId::InfernalBlade)));
        assert!(engine::validate_runtime(&combat.state).is_ok());
    }
}

#[test]
fn m10_infernal_generated_attack_overflows_after_a_real_preceding_draw_fills_hand(){
    let mut combat=real_infernal(0);
    for instance in 100..108{
        let card=card_state::generated(CardId::Anger,CardInstanceId(instance),0).unwrap();
        combat.state.card_ids_issued.insert(card_state::handle(card.instance));
        combat.state.piles.draw.push(OwnedCard::Full(card));
    }
    assert!(engine::validate_runtime(&combat.state).is_ok(),"the owned draw pile and hand are valid before the card action");
    let action=infernal_action(&combat);
    let mut work=stage_real_play(&mut combat,&action);
    let generator=work.iter().position(|effect|matches!(effect.kind,
        EffectKind::Ironclad(super::super::cards::IroncladEffect::InfernalBladeGenerate))).unwrap();
    work.insert(generator,Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::Draw(8)});
    combat.state.pending.clear();combat.state.pending.extend(work);
    assert!(engine::validate_runtime(&combat.state).is_ok(),"actual Infernal owner plus its accepted draw/finish continuation is valid");
    combat.state.resolved_effects=0;
    let transition=engine::resolve(&mut combat.state);
    assert_eq!(transition.status,TransitionStatus::Decision);
    assert!(combat.state.fault.is_none());
    assert_eq!(combat.state.piles.hand.len(),10,"the current Infernal source left hand before its preceding draw filled the hand");
    let generated=combat.state.piles.discard.iter().filter_map(OwnedCard::full)
        .find(|card|card.instance.0==108).expect("new eligible Attack overflows to discard");
    assert!(crate::run::rooms::m10_content::infernal_candidates().contains(&generated.id));
    assert!(combat.state.piles.exhaust.iter().any(|owned|owned.full().is_some_and(|card|card.id==CardId::InfernalBlade)));
    assert!(engine::validate_runtime(&combat.state).is_ok());
}

#[test]
fn m10_infernal_generated_identity_has_real_constructor_and_public_values(){
    use crate::combat::CombatCardRef;
    let card=card_state::generated(CardId::InfernalBlade,CardInstanceId(700),0)
        .expect("the actual generated Infernal Blade has an owned constructor");
    assert_eq!(card.origin,CombatCardRef::CombatLocal(700));
    assert!(card.local.exhaust_on_finish);
    let values=cards::ironclad::rules::public_values(&card)
        .expect("generated Infernal Blade exposes its real public stats");
    assert_eq!(values.magic_number,None,"the one generated card is not a card-text magic number");
    assert!(values.exhaust_on_finish);
    let upgraded=card_state::generated(CardId::InfernalBlade,CardInstanceId(701),1).unwrap();
    assert_eq!(cards::ironclad::rules::public_values(&upgraded).unwrap().magic_number,None,
        "upgrading changes payment, not the generated-card count or displayed magic");
}

#[test]
fn m10_infernal_validation_preflights_native_submission_without_mutation(){
    use crate::combat::{Effect,EffectKind,Source};
    let mut combat=full_runtime::entered_combat(1,1,|card|*card=card_state::permanent(CardId::InfernalBlade,card.instance,0));
    assert!(engine::validate_runtime(&combat.state).is_ok());
    let before=combat.snapshot();
    let effect=Effect{source:Source::None,target:None,
        kind:EffectKind::Ironclad(super::super::cards::IroncladEffect::InfernalBladeGenerate)};
    assert!(combat.enqueue_native_effects(vec![effect],None).is_err(),
        "an orphan generator cannot pass the public ready-command preflight");
    assert_eq!(combat.snapshot(),before,"rejected work leaves the ready combat checkpoint unchanged");
    assert!(engine::validate_runtime(&combat.state).is_ok());
}

#[test]
fn m10_infernal_validation_accepts_only_the_actual_inflight_card_owner(){
    let mut combat=real_infernal(0);
    let action=infernal_action(&combat);
    let work=stage_real_play(&mut combat,&action);
    assert!(engine::validate_runtime(&combat.state).is_ok(),"the in-flight player action is a legitimate checkpoint");
    let generator=work.iter().find(|effect|matches!(effect.kind,
        EffectKind::Ironclad(super::super::cards::IroncladEffect::InfernalBladeGenerate))).unwrap();
    assert!(engine::validate_effect(generator,&combat.state).is_ok(),"the actual Infernal Blade producer is admitted");
    let snapshot=combat.snapshot();
    assert_eq!(snapshot.schema_version,3,"actual M10 state is held in the full-runtime in-memory snapshot schema");
    assert!(snapshot.state.pending.iter().any(|effect|matches!(effect.kind,
        EffectKind::Ironclad(super::super::cards::IroncladEffect::InfernalBladeGenerate))));
    assert_eq!(combat.try_snapshot_v2(),Err(RestoreError::Compatibility),"the old restorable schema refuses this new full-runtime continuation");
    let legacy=Combat::new(&super::whole_combat::scenario(EnemyKind::Cultist)).unwrap();
    let legacy_checkpoint=legacy.try_snapshot_v2().expect("old M9 checkpoint remains representable");
    assert_eq!(Combat::restore(&legacy_checkpoint).unwrap().snapshot(),legacy_checkpoint,
        "ordinary M9 snapshot remains restorable through its actual legacy codec");
    let wrong_source=Effect{source:Source::None,target:None,kind:generator.kind.clone()};
    let wrong_target=Effect{source:generator.source.clone(),target:Some(Actor::Player),kind:generator.kind.clone()};
    assert!(engine::validate_effect(&wrong_source,&combat.state).is_err());
    assert!(engine::validate_effect(&wrong_target,&combat.state).is_err());

    let mut non_infernal=full_runtime::entered_combat(1,1,|_|{});
    let strike=non_infernal.state.piles.hand.iter().filter_map(OwnedCard::full).find(|card|card.id==CardId::Strike).unwrap().instance.0;
    let work=stage_real_play(&mut non_infernal,&CombatAction::PlayCard{instance:strike,target:Some(TargetId(1))});
    assert!(engine::validate_runtime(&non_infernal.state).is_ok(),"negative control starts from a valid actual Strike action");
    let wrong=Effect{source:work[0].source.clone(),target:None,kind:generator.kind.clone()};
    assert!(engine::validate_effect(&wrong,&non_infernal.state).is_err(),"another valid in-play card cannot own Infernal generation");
}

#[test]
fn m10_infernal_support_has_generated_owners_for_all_28_attacks(){
    for (index,id) in crate::run::rooms::m10_content::infernal_candidates().iter().copied().enumerate(){
        let card=card_state::generated(id,CardInstanceId(800+index as u64),0)
            .unwrap_or_else(||panic!("{id:?} is in the captured Infernal support but has no generated owner"));
        assert_eq!(card.id,id);
        assert_eq!(card.origin,crate::combat::CombatCardRef::CombatLocal(card.instance.0));
    }
    let bfb=card_state::generated_with_combat_history(CardId::BloodForBlood,CardInstanceId(900),0,3).unwrap();
    assert_eq!(card_state::effective_cost(&bfb,4).unwrap(),1,"generated Blood for Blood retains the already-observed combat damage reductions");
    let whirlwind=card_state::generated(CardId::Whirlwind,CardInstanceId(901),1).unwrap();
    assert!(whirlwind.local.x_cost,"generated Whirlwind preserves native X-cost payment");
    assert_eq!(whirlwind.local.temporary_cost,0,"turn-zero treatment must not encode a fixed cost discount on an X card");
}

#[test]
fn m10_infernal_rng_exhaustion_retains_the_accepted_parent_and_cursor(){
    let mut combat=real_infernal(0);
    let stream=combat.state.card_rng.as_ref().unwrap();
    combat.state.card_rng=Some(crate::combat::rng::StreamState::new(stream.id().clone(),*stream.key(),1_u128<<64).unwrap());
    let exhausted=combat.state.card_rng.clone();
    assert!(engine::validate_runtime(&combat.state).is_ok(),"sentinel exhaustion state is valid before actual card play");
    assert!(combat.step_full(&infernal_action(&combat)).is_err());
    assert_eq!(combat.state.card_rng,exhausted,"RNG exhaustion is detected before advancing the sentinel frontier");
    assert!(combat.state.current.as_ref().is_some_and(|effect|matches!(effect.kind,EffectKind::Ironclad(super::super::cards::IroncladEffect::InfernalBladeGenerate))));
    assert!(combat.state.fault.is_some());
    assert!(engine::validate_runtime(&combat.state).is_ok(),"the accepted current failure belongs to the valid in-flight Infernal play");
}

#[test]
fn m10_infernal_finite_issuer_failure_retains_sampled_frontier_without_retry(){
    let mut combat=real_infernal(0);
    for id in 10_000..10_996 {
        let card=card_state::generated(CardId::Anger,CardInstanceId(id),0).unwrap();
        combat.state.card_ids_issued.insert(card_state::handle(card.instance));
        combat.state.inactive_cards.push(OwnedCard::Full(card));
    }
    assert_eq!(combat.state.card_ids_issued.len(),999);
    assert!(engine::validate_runtime(&combat.state).is_ok(),"current owners are unchanged; only inactive historical issuance IDs were added");
    let before=combat.state.card_rng.clone();
    assert!(combat.step_full(&infernal_action(&combat)).is_err(),"real Infernal play samples before finite allocation exhaustion");
    assert_ne!(combat.state.card_rng,before,"the accepted sample is captured before the finite issuer boundary");
    assert!(combat.state.current.as_ref().is_some_and(|effect|matches!(effect.kind,EffectKind::Ironclad(super::super::cards::IroncladEffect::InfernalBladeGenerate))));
    assert!(combat.state.fault.is_some());
    assert!(engine::validate_runtime(&combat.state).is_ok(),"the accepted allocation failure is attached to a valid in-flight play");
}

#[test]
fn m10_infernal_blade_turn_discount_expires_at_the_next_player_turn(){
    let mut combat=full_runtime::entered_combat(1,1,|_|{});
    let mut card=card_state::generated(CardId::Bludgeon,CardInstanceId(900),0).unwrap();
    card.local.temporary_cost=-3;
    card.local.infernal_blade_cost_this_turn=true;
    combat.state.card_ids_issued.insert(card_state::handle(card.instance));
    combat.state.piles.draw.push(OwnedCard::Full(card));
    let live=combat.state.piles.draw.last().unwrap().full().unwrap();
    assert_eq!(card_state::effective_cost(live,3).unwrap(),0);
    assert!(engine::validate_runtime(&combat.state).is_ok(),"the generated cost owner is a valid issued card");
    combat.step_full(&CombatAction::EndTurn).expect("actual end-turn and enemy phase reaches the next player turn");
    let restored=combat.state.piles.hand.iter().chain(&combat.state.piles.draw).chain(&combat.state.piles.discard).chain(&combat.state.piles.exhaust)
        .filter_map(OwnedCard::full).find(|card|card.instance==CardInstanceId(900)).expect("generated owner survives the actual draw/discard lifecycle");
    assert_eq!(restored.local.temporary_cost,0);
    assert!(!restored.local.infernal_blade_cost_this_turn);
    assert_eq!(card_state::effective_cost(restored,3).unwrap(),3);
}
