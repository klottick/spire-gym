use super::super::*;
use super::full_runtime::entered_combat;
use crate::{combat::model::{OwnedCard,PlayContext},content::{CardId,Catalog,CardRarity}};

fn combat_with(id:CardId,upgraded:bool)->Combat {
    entered_combat(3,3,|card|*card=card_state::permanent(id,card.instance,u32::from(upgraded)))
}
fn instance(c:&Combat,id:CardId)->u64 {
    c.state.piles.all(c.state.in_play.as_ref()).filter_map(OwnedCard::full)
        .find(|card|card.id==id).unwrap().instance.0
}
fn play(c:&mut Combat,id:CardId,target:Option<TargetId>){
    c.step_full(&CombatAction::PlayCard{instance:instance(c,id),target}).unwrap();
}
fn card(c:&Combat,id:CardId)->&LiveCard {
    c.state.piles.all(c.state.in_play.as_ref()).filter_map(OwnedCard::full).find(|card|card.id==id).unwrap()
}

fn register(c:&mut Combat,instance:u64){c.state.card_ids_issued.insert(card_state::handle(CardInstanceId(instance)));}

fn commit_prepared_without_draining(c:&mut Combat,action:&CombatAction)->Vec<Effect>{
    let prepared=cards::prepare(c,action).unwrap();
    let handle=card_state::handle(prepared.instance);
    let source=Source::Card(handle.clone());
    let card=c.state.piles.hand.remove(prepared.hand_index);
    c.state.in_play=Some(card);
    c.state.player.energy=prepared.remaining_energy;
    c.state.phase=Phase::PlayerAction;
    c.state.play_context=Some(PlayContext{card:handle,target:prepared.target,energy_on_use:prepared.captured_energy,paid:prepared.paid,
        disposition:super::super::model::FinishDisposition::Natural});
    let effects=prepared.effects;
    c.state.pending.extend(effects.iter().cloned());
    c.state.pending.push_back(Effect{source:source.clone(),target:None,kind:EffectKind::FinishCard});
    c.state.pending.push_back(Effect{source:Source::None,target:None,kind:EffectKind::Decision});
    effects
}

#[test]
fn m10_growth_flex_applies_gain_then_artifact_blockable_expiry_loss(){
    for (upgraded,gain) in [(false,2),(true,4)] {
        let mut c=combat_with(CardId::Flex,upgraded);
        assert_eq!(Catalog::standard().card(CardId::Flex).unwrap().rarity,CardRarity::Common);
        play(&mut c,CardId::Flex,None);
        assert_eq!(c.state.player_full_powers.iter().find(|p|p.id.0=="strength").unwrap().state.amount,gain);
        assert_eq!(c.state.player_full_powers.iter().find(|p|p.id.0=="lose_strength").unwrap().state.amount,gain);
        c.state.player_full_powers.push(FullPowerEntry{id:FullPowerId("artifact"),state:PowerState{amount:1,skip_first:false,monster_source_grace:false}});
        c.step_full(&CombatAction::EndTurn).unwrap();
        assert_eq!(c.state.player_full_powers.iter().find(|p|p.id.0=="strength").unwrap().state.amount,gain,
            "Artifact at expiry blocks only the negative Strength application");
        assert!(!c.state.player_full_powers.iter().any(|p|p.id.0=="lose_strength"));
    }
}

#[test]
fn m10_growth_flex_use_artifact_blocks_only_the_loss_power(){
    let mut c=combat_with(CardId::Flex,false);
    c.state.player_full_powers.push(FullPowerEntry{id:FullPowerId("artifact"),state:PowerState{amount:1,skip_first:false,monster_source_grace:false}});
    play(&mut c,CardId::Flex,None);
    assert_eq!(c.state.player_full_powers.iter().find(|p|p.id.0=="strength").unwrap().state.amount,2);
    assert!(!c.state.player_full_powers.iter().any(|p|p.id.0=="lose_strength"));
    assert!(!c.state.player_full_powers.iter().any(|p|p.id.0=="artifact"));
}

#[test]
fn m10_growth_flex_expires_the_unblocked_temporary_strength(){
    let mut c=combat_with(CardId::Flex,false);play(&mut c,CardId::Flex,None);
    c.step_full(&CombatAction::EndTurn).unwrap();
    assert!(!c.state.player_full_powers.iter().any(|p|p.id.0=="strength"));
    assert!(!c.state.player_full_powers.iter().any(|p|p.id.0=="lose_strength"));
}

#[test]
fn m10_growth_limit_break_doubles_signed_strength_and_only_base_exhausts(){
    for (upgraded,start,want,exhaust) in [(false,3,6,true),(true,-2,-4,false)] {
        let mut c=combat_with(CardId::LimitBreak,upgraded);
        c.state.player_full_powers.push(FullPowerEntry{id:FullPowerId("strength"),state:PowerState{amount:start,skip_first:false,monster_source_grace:false}});
        play(&mut c,CardId::LimitBreak,None);
        assert_eq!(c.state.player_full_powers.iter().find(|p|p.id.0=="strength").unwrap().state.amount,want);
        assert_eq!(c.state.piles.exhaust.iter().any(|o|o.full().is_some_and(|x|x.id==CardId::LimitBreak)),exhaust);
    }
}

#[test]
fn m10_growth_limit_break_negative_strength_is_an_artifact_blockable_debuff(){
    let mut c=combat_with(CardId::LimitBreak,true);
    c.state.player_full_powers.extend([
        FullPowerEntry{id:FullPowerId("strength"),state:PowerState{amount:-2,skip_first:false,monster_source_grace:false}},
        FullPowerEntry{id:FullPowerId("artifact"),state:PowerState{amount:1,skip_first:false,monster_source_grace:false}},
    ]);
    play(&mut c,CardId::LimitBreak,None);
    assert_eq!(c.state.player_full_powers.iter().find(|p|p.id.0=="strength").unwrap().state.amount,-2);
    assert!(!c.state.player_full_powers.iter().any(|p|p.id.0=="artifact"));
}

#[test]
fn m10_growth_limit_break_without_strength_leaves_no_persistent_strength(){
    let mut c=combat_with(CardId::LimitBreak,true);
    c.state.player_full_powers.push(FullPowerEntry{id:FullPowerId("artifact"),state:PowerState{amount:1,skip_first:false,monster_source_grace:false}});
    play(&mut c,CardId::LimitBreak,None);
    assert!(!c.state.player_full_powers.iter().any(|p|p.id.0=="strength"));
    assert_eq!(c.state.player_full_powers.iter().find(|p|p.id.0=="artifact").unwrap().state.amount,1,
        "with no existing Strength power, Limit Break is a no-op and does not create a zero application");
    assert!(c.state.piles.discard.iter().any(|owned|owned.full().is_some_and(|x|x.id==CardId::LimitBreak)));
}

#[test]
fn m10_growth_limit_break_reads_signed_strength_at_queued_dispatch(){
    let mut c=combat_with(CardId::LimitBreak,true);
    c.state.player_full_powers.push(FullPowerEntry{id:FullPowerId("strength"),state:PowerState{amount:2,skip_first:false,monster_source_grace:false}});
    let action=CombatAction::PlayCard{instance:instance(&c,CardId::LimitBreak),target:None};
    let prepared_effects=commit_prepared_without_draining(&mut c,&action);
    assert_eq!(prepared_effects.len(),1,"Limit Break creates one native action regardless of the stat captured by its read-only preparation");
    c.state.pending.push_front(Effect{source:Source::None,target:Some(Actor::Player),kind:EffectKind::ApplyFullPower{id:FullPowerId("strength"),amount:-3}});
    assert_eq!(engine::resolve(&mut c.state).status,TransitionStatus::Decision);
    assert_eq!(c.state.player_full_powers.iter().find(|p|p.id.0=="strength").unwrap().state.amount,-2,
        "the queued operation doubles dispatch-time Strength -1, not preparation-time Strength 2");
}

#[test]
fn m10_growth_blood_for_blood_cost_tracks_positive_hp_loss_events_not_hp_quantity(){
    let mut c=combat_with(CardId::BloodForBlood,false);
    let id=instance(&c,CardId::BloodForBlood);
    let source=card(&c,CardId::BloodForBlood).clone();
    c.state.piles.draw.push(OwnedCard::Full(source.copy_as(CardInstanceId(801))));
    c.state.piles.discard.push(OwnedCard::Full(source.copy_as(CardInstanceId(802))));
    c.state.piles.exhaust.push(OwnedCard::Full(source.copy_as(CardInstanceId(803))));
    c.state.inactive_cards.push(OwnedCard::Full(source.copy_as(CardInstanceId(804))));
    for id in [801,802,803,804]{register(&mut c,id)}
    engine::validate_runtime(&c.state).unwrap();
    let hp=c.state.player.hp;
    c.enqueue_native_effects(vec![Effect{source:Source::None,target:Some(Actor::Player),kind:EffectKind::Damage(DamagePayload::HpLoss(3))}],None).unwrap();
    assert_eq!(c.state.player.hp,hp-3);
    assert_eq!(card_state::effective_cost(card(&c,CardId::BloodForBlood),4).unwrap(),3);
    for zone in [&c.state.piles.hand,&c.state.piles.draw,&c.state.piles.discard]{assert!(zone.iter().filter_map(OwnedCard::full).filter(|x|x.id==CardId::BloodForBlood).all(|x|card_state::effective_cost(x,4).unwrap()==3));}
    assert_eq!(card_state::effective_cost(c.state.piles.exhaust[0].full().unwrap(),4).unwrap(),4,"exhaust is not notified");
    assert_eq!(card_state::effective_cost(c.state.inactive_cards[0].full().unwrap(),4).unwrap(),4,"inactive census custody is not a callback zone");
    c.enqueue_native_effects(vec![Effect{source:Source::None,target:Some(Actor::Player),kind:EffectKind::Damage(DamagePayload::HpLoss(0))}],None).unwrap();
    assert_eq!(card_state::effective_cost(card(&c,CardId::BloodForBlood),4).unwrap(),3);
    c.state.player.block=5;let hp_after_loss=c.state.player.hp;
    c.enqueue_native_effects(vec![Effect{source:Source::None,target:Some(Actor::Player),kind:EffectKind::Damage(DamagePayload::Fixed(3))}],None).unwrap();
    assert_eq!(c.state.player.hp,hp_after_loss,"fully blocked damage does not commit a positive HP-loss event");
    assert_eq!(card_state::effective_cost(card(&c,CardId::BloodForBlood),4).unwrap(),3);
    play(&mut c,CardId::BloodForBlood,Some(TargetId(1)));
    assert_eq!(card_state::effective_cost(card(&c,CardId::BloodForBlood),4).unwrap(),3);
    c.enqueue_native_effects(vec![Effect{source:Source::None,target:Some(Actor::Player),kind:EffectKind::Damage(DamagePayload::HpLoss(1))}],None).unwrap();
    assert_eq!(card_state::effective_cost(card(&c,CardId::BloodForBlood),4).unwrap(),2);
    for _ in 0..3{c.enqueue_native_effects(vec![Effect{source:Source::None,target:Some(Actor::Player),kind:EffectKind::Damage(DamagePayload::HpLoss(1))}],None).unwrap();}
    assert_eq!(card_state::effective_cost(card(&c,CardId::BloodForBlood),4).unwrap(),0,"combat cost lower-clamps at zero");
    assert!(c.state.piles.all(c.state.in_play.as_ref()).any(|owned|owned.full().is_some_and(|x|x.instance.0==id&&x.id==CardId::BloodForBlood)),"cost changes preserve the exact source instance");
    engine::validate_runtime(&c.state).unwrap();
}

#[test]
fn m10_growth_blood_for_blood_event_history_overflow_faults_before_hp_commit(){
    let mut c=combat_with(CardId::BloodForBlood,false);
    c.state.damage_events_this_combat=u32::MAX;
    let hp=c.state.player.hp;
    let result=c.enqueue_native_effects(vec![Effect{source:Source::None,target:Some(Actor::Player),kind:EffectKind::Damage(DamagePayload::HpLoss(1))}],None);
    assert!(result.is_err());
    assert_eq!(c.state.player.hp,hp,"counter overflow must be rejected before committing player damage");
    assert_eq!(c.state.damage_events_this_combat,u32::MAX);
    assert!(c.state.fault.is_some());
}

#[test]
fn m10_growth_blood_for_blood_cost_state_survives_snapshot_comparison_and_observation(){
    let mut c=combat_with(CardId::BloodForBlood,false);
    c.enqueue_native_effects(vec![Effect{source:Source::None,target:Some(Actor::Player),kind:EffectKind::Damage(DamagePayload::HpLoss(1))}],None).unwrap();
    let snapshot=c.snapshot();let restored=c.clone();
    assert_eq!(super::super::snapshot::compare(&snapshot,&restored.snapshot()),None);
    let view=restored.public_full().unwrap();
    assert_eq!(view.hand.iter().find(|card|card.id==CardId::BloodForBlood).unwrap().effective_cost,Some(3));
    assert_eq!(view,restored.public_full().unwrap(),"reading the mutated cost state is pure");
}

#[test]
fn m10_growth_blood_for_blood_fresh_kind_catches_up_actual_combat_loss_history(){
    let mut c=combat_with(CardId::BloodForBlood,false);
    c.enqueue_native_effects(vec![Effect{source:Source::None,target:Some(Actor::Player),kind:EffectKind::Damage(DamagePayload::HpLoss(2))}],None).unwrap();
    let fresh=card_state::generated_with_combat_history(CardId::BloodForBlood,CardInstanceId(777),0,c.state.damage_events_this_combat).unwrap();
    assert_eq!(card_state::effective_cost(&fresh,4).unwrap(),3,"fresh constructor catches up each actual positive HP-loss event exactly once");
    let retained=card(&c,CardId::BloodForBlood).stat_equivalent_copy(CardInstanceId(778));
    assert_eq!(card_state::effective_cost(&retained,4).unwrap(),3,"stat-equivalent copies preserve source cost instead of replaying combat history");
    assert_eq!(retained.local.combat_cost_reduction,1);
    let family=card(&c,CardId::BloodForBlood).same_instance_family_copy(CardInstanceId(779));
    assert_eq!(family.local.combat_cost_reduction,1,"same-family copies preserve live cost without replaying history");
}

#[test]
fn m10_growth_blood_for_blood_cost_discount_composes_with_upgrade_base_cost(){
    let mut live=card_state::permanent(CardId::BloodForBlood,CardInstanceId(778),0);
    card_state::reduce_blood_for_blood_cost(&mut live);
    assert_eq!(card_state::effective_cost(&live,4).unwrap(),3);
    cards::ironclad::rules::upgrade(&mut live).unwrap();
    assert_eq!(card_state::effective_cost(&live,3).unwrap(),2,
        "upgrading from combat cost 4 to 3 preserves its separate one-point combat discount");
}

#[test]
fn m10_growth_blood_for_blood_cost_clamp_does_not_change_other_card_negative_cost_rejection(){
    let mut strike=card_state::permanent(CardId::Strike,CardInstanceId(777),0);
    strike.local.temporary_cost=-1;
    assert!(card_state::effective_cost(&strike,0).is_err(),
        "legacy non-BfB negative effective-cost adapter contract remains an invariant error");
    let mut bfb=card_state::permanent(CardId::BloodForBlood,CardInstanceId(778),0);
    bfb.local.temporary_cost=-99;
    assert_eq!(card_state::effective_cost(&bfb,4).unwrap(),0,
        "Blood for Blood's source cost adjustment floors its own current cost at zero");
}

#[test]
fn m10_growth_blood_for_blood_floor_hp_loss_is_noop_for_fresh_confused_price_and_copies(){
    let mut bfb=card_state::permanent(CardId::BloodForBlood,CardInstanceId(779),0);
    bfb.local.combat_cost_reduction=4;
    bfb.local.randomized_cost=Some(2);
    bfb.local.temporary_cost=0;
    bfb.local.exhaust_on_finish=false;
    let mut copy=bfb.stat_equivalent_copy(CardInstanceId(780));
    card_state::reduce_blood_for_blood_cost(&mut bfb);
    assert_eq!(bfb.local.combat_cost_reduction,4,"ordinary cost update at base-cost floor is a no-op");
    assert_eq!(bfb.local.randomized_cost,Some(2),"fresh Confused price survives a floor no-op");
    assert_eq!(card_state::effective_cost(&bfb,4).unwrap(),2);
    assert!(!bfb.local.exhaust_on_finish);
    assert_eq!((copy.local.combat_cost_reduction,copy.local.randomized_cost),(4,Some(2)),
        "a preexisting stat-equivalent copy remains an immutable snapshot of its own cost state");
    // A cost-state copy made after the no-op must retain the unchanged fields.
    copy=bfb.stat_equivalent_copy(CardInstanceId(781));
    assert_eq!((copy.local.combat_cost_reduction,copy.local.randomized_cost,copy.local.temporary_cost),(4,Some(2),0));
}

#[test]
fn m10_growth_blood_for_blood_upgrade_uses_native_turn_cost_offsets(){
    for (reduction,turn_cost,want_reduction,want_turn_cost) in [(4,2,3,1),(3,2,3,1),(2,2,2,1),(4,0,3,0)] {
        let mut bfb=card_state::permanent(CardId::BloodForBlood,CardInstanceId(782),0);
        bfb.local.combat_cost_reduction=reduction;
        bfb.local.randomized_cost=Some(turn_cost);
        super::super::cards::ironclad::rules::upgrade(&mut bfb).unwrap();
        assert_eq!(bfb.upgrades,1);
        assert_eq!(bfb.local.combat_cost_reduction,want_reduction,
            "native BfB upgrade re-encodes the floored base reduction ({reduction},{turn_cost})");
        assert_eq!(bfb.local.randomized_cost,Some(want_turn_cost),
            "the actual current-turn price follows the BfB upgrade cost helper ({reduction},{turn_cost})");
        assert_eq!(card_state::effective_cost(&bfb,3).unwrap(),want_turn_cost);
        assert_eq!(super::super::cards::ironclad::rules::public_values(&bfb).unwrap().base_damage,Some(22));
        assert_eq!(bfb.local.temporary_upgrades,1,"the upgrade remains combat-local");
    }
}

#[test]
fn m10_growth_blood_for_blood_upgrade_decrements_actual_turn_cost_across_representations(){
    for (randomized,temporary,expected_randomized,expected_temporary,turn_cost) in [
        (None,2,None,1,2),(Some(2),0,Some(1),0,2),(Some(0),2,Some(0),1,2),
        (None,0,None,0,0),(Some(0),0,Some(0),0,0),
    ] {
        let mut bfb=card_state::permanent(CardId::BloodForBlood,CardInstanceId(783),0);
        bfb.local.combat_cost_reduction=4;
        bfb.local.randomized_cost=randomized;
        bfb.local.temporary_cost=temporary;
        assert_eq!(card_state::effective_cost(&bfb,4).unwrap(),turn_cost,
            "test setup must start from its source-observable current turn price");
        super::super::cards::ironclad::rules::upgrade(&mut bfb).unwrap();
        assert_eq!(bfb.local.randomized_cost,expected_randomized);
        assert_eq!(bfb.local.temporary_cost,expected_temporary);
        assert_eq!(card_state::effective_cost(&bfb,3).unwrap(),turn_cost.saturating_sub(1).max(0));
    }
}

#[test]
fn m10_growth_blood_for_blood_floor_hp_loss_preserves_every_turn_cost_representation(){
    for (randomized,temporary) in [(None,2),(Some(2),0),(Some(0),2)] {
        let mut bfb=card_state::permanent(CardId::BloodForBlood,CardInstanceId(784),0);
        bfb.local.combat_cost_reduction=4;
        bfb.local.randomized_cost=randomized;
        bfb.local.temporary_cost=temporary;
        assert_eq!(card_state::effective_cost(&bfb,4).unwrap(),2);
        card_state::reduce_blood_for_blood_cost(&mut bfb);
        assert_eq!(card_state::effective_cost(&bfb,4).unwrap(),2);
        assert_eq!((bfb.local.combat_cost_reduction,bfb.local.randomized_cost,bfb.local.temporary_cost),(4,randomized,temporary));
    }
}

#[test]
fn m10_growth_armaments_upgrade_of_discounted_blood_for_blood_preserves_runtime_validity_and_floor_view(){
    let mut c=combat_with(CardId::BloodForBlood,false);
    let armaments=c.state.piles.hand.iter_mut().find(|owned|owned.handle().as_str()=="2").unwrap();
    *armaments=OwnedCard::Full(card_state::permanent(CardId::Armaments,CardInstanceId(2),1));
    engine::validate_runtime(&c.state).unwrap();
    for _ in 0..4 {
        c.enqueue_native_effects(vec![Effect{source:Source::None,target:Some(Actor::Player),
            kind:EffectKind::Damage(DamagePayload::HpLoss(1))}],None).unwrap();
    }
    let at_floor=c.public_full().unwrap().hand.into_iter().find(|card|card.id==CardId::BloodForBlood).unwrap();
    assert_eq!(at_floor.effective_cost,Some(0));
    c.enqueue_native_effects(vec![Effect{source:Source::None,target:Some(Actor::Player),
        kind:EffectKind::Damage(DamagePayload::HpLoss(1))}],None).unwrap();
    let after_floor=c.public_full().unwrap().hand.into_iter().find(|card|card.id==CardId::BloodForBlood).unwrap();
    assert_eq!(after_floor,at_floor,"no-op at the cost floor preserves the public owned-card view and flags");
    assert_eq!(c.state.player.energy,3);
    c.step_full(&CombatAction::PlayCard{instance:2,target:None}).unwrap();
    let upgraded=c.public_full().unwrap().hand.iter().find(|card|card.id==CardId::BloodForBlood).unwrap().clone();
    assert_eq!(upgraded.upgrades,1);
    assert_eq!(upgraded.effective_cost,Some(0));
    assert_eq!(upgraded.values.as_ref().unwrap().base_damage,Some(22));
    assert!(c.fault_record().is_none());
    engine::validate_runtime(&c.state).expect("Armaments+ must leave the owned BfB runtime cost state valid");
}

#[test]
fn m10_growth_blood_for_blood_actual_create_card_path_catches_up_history(){
    let mut c=combat_with(CardId::BloodForBlood,false);
    c.enqueue_native_effects(vec![Effect{source:Source::None,target:Some(Actor::Player),kind:EffectKind::Damage(DamagePayload::HpLoss(1))}],None).unwrap();
    effects::apply_effect(&mut c.state,&Effect{source:Source::None,target:Some(Actor::Player),kind:EffectKind::CreateCard{
        id:CardId::BloodForBlood,destination:CardDestination::Discard,upgrades:0
    }}).unwrap();
    let generated=c.state.piles.discard.iter().filter_map(OwnedCard::full).find(|x|x.id==CardId::BloodForBlood).unwrap();
    assert_eq!(card_state::effective_cost(generated,4).unwrap(),3,
        "the existing native CreateCard constructor applies prior positive combat-loss history once");
    engine::validate_runtime(&c.state).unwrap();
}

#[test]
fn m10_growth_blood_for_blood_reduction_updates_randomized_current_cost_and_preserves_free_flag(){
    let mut c=combat_with(CardId::BloodForBlood,false);
    let mut alternate=card(&c,CardId::BloodForBlood).copy_as(CardInstanceId(807));
    alternate.local.randomized_cost=Some(2);alternate.local.free_play=true;
    c.state.piles.draw.push(OwnedCard::Full(alternate));
    register(&mut c,807);engine::validate_runtime(&c.state).unwrap();
    c.enqueue_native_effects(vec![Effect{source:Source::None,target:Some(Actor::Player),kind:EffectKind::Damage(DamagePayload::HpLoss(1))}],None).unwrap();
    let changed=c.state.piles.draw[0].full().unwrap();
    assert_eq!(changed.local.randomized_cost,Some(1),"positive HP loss mutates the current Confused-like price once");assert!(changed.local.free_play);
    assert_eq!(changed.local.temporary_cost,0,"persistent combat reduction is distinct from temporary cost offset");
    let mut visible=changed.clone();visible.local.free_play=false;
    assert_eq!(card_state::effective_cost(&visible,4).unwrap(),1);
    engine::validate_runtime(&c.state).unwrap();
}

#[test]
fn m10_growth_blood_for_blood_reduces_persistent_cost_and_preserves_turn_offset(){
    for (randomized,temporary,want) in [(Some(6),0,5),(Some(6),2,7),(Some(0),0,0),(Some(0),-1,0),(None,2,5)] {
        let mut c=combat_with(CardId::BloodForBlood,false);
        let id=instance(&c,CardId::BloodForBlood);
        let live=c.state.piles.hand.iter_mut().filter_map(OwnedCard::full_mut).find(|x|x.instance.0==id).unwrap();
        live.local.randomized_cost=randomized;live.local.temporary_cost=temporary;
        c.enqueue_native_effects(vec![Effect{source:Source::None,target:Some(Actor::Player),kind:EffectKind::Damage(DamagePayload::HpLoss(1))}],None).unwrap();
        let reduced=card(&c,CardId::BloodForBlood);
        assert_eq!(reduced.local.randomized_cost,randomized.map(|cost|cost.saturating_sub(1).max(0)));
        assert_eq!(card_state::effective_cost(reduced,4).unwrap(),want,
            "persistent combat reduction composes with randomized/temporary turn cost");
        if randomized==Some(6){
            let live=c.state.piles.hand.iter_mut().filter_map(OwnedCard::full_mut).find(|x|x.id==CardId::BloodForBlood).unwrap();
            live.local.randomized_cost=None;live.local.temporary_cost=0;
            assert_eq!(card_state::effective_cost(live,4).unwrap(),3,"clearing a prior turn's randomized price retains combat cost reduction");
        }
    }
}

#[test]
fn m10_growth_blood_for_blood_hp_loss_updates_actual_turn_cost_across_representations(){
    for (reduction,randomized,temporary,base,initial,want) in [
        (0,None,-2,4,2,1),(0,Some(2),0,4,2,1),(0,Some(0),2,4,2,1),
        (0,None,2,4,6,5),(0,Some(4),2,4,6,5),
        (4,None,2,4,2,2),(4,Some(2),0,4,2,2),(4,Some(0),2,4,2,2),
    ] {
        let mut c=combat_with(CardId::BloodForBlood,false);
        let id=instance(&c,CardId::BloodForBlood);
        {
            let owned=c.state.piles.hand.iter_mut().filter_map(OwnedCard::full_mut)
                .find(|card|card.instance.0==id).unwrap();
            owned.local.combat_cost_reduction=reduction;
            owned.local.randomized_cost=randomized;
            owned.local.temporary_cost=temporary;
        }
        engine::validate_runtime(&c.state).expect("issued owned BfB cost representation is valid before the callback");
        assert_eq!(card_state::effective_cost(card(&c,CardId::BloodForBlood),base).unwrap(),initial);
        c.enqueue_native_effects(vec![Effect{source:Source::None,target:Some(Actor::Player),
            kind:EffectKind::Damage(DamagePayload::HpLoss(1))}],None).unwrap();
        let reduced=card(&c,CardId::BloodForBlood);
        assert_eq!(card_state::effective_cost(reduced,base).unwrap(),want,
            "ordinary cost update changes the actual turn price for ({reduction},{randomized:?},{temporary})");
        engine::validate_runtime(&c.state).expect("callback leaves owned cost state valid");
    }
}

#[test]
fn m10_growth_blood_for_blood_counts_actual_enemy_thorns_hp_loss(){
    let mut entry=super::m8_shapes::single(crate::content::EnemyId::Spiker);
    entry.cards[0]=card_state::permanent(CardId::BloodForBlood,CardInstanceId(88),0);
    entry.opening_draw=1;entry.setup.player.energy=4;entry.setup.player.max_energy=4;
    let mut c=Combat::enter(entry).unwrap();c.state.enemies[0].block=0;
    c.state.enemies[0].hp=c.state.enemies[0].max_hp;
    c.state.player_full_powers.push(FullPowerEntry{id:FullPowerId("double_tap"),state:PowerState{amount:1,skip_first:false,monster_source_grace:false}});
    play(&mut c,CardId::BloodForBlood,Some(TargetId(1)));
    assert_eq!(c.state.player.hp,294,"both Spiker retaliations cause actual positive THORNS HP loss");
    assert_eq!(card_state::effective_cost(card(&c,CardId::BloodForBlood),4).unwrap(),4,
        "parent held in a live play frame and current repeated card are excluded from hand/draw/discard callbacks");
    assert!(c.state.inactive_cards.iter().filter_map(OwnedCard::full).any(|card|card.id==CardId::BloodForBlood
        &&card.copy_provenance==card_state::CopyProvenance::SameInstanceFamily
        &&card.local.combat_cost_reduction==0),"the completed queued limbo child remains unmodified in inactive custody: {:?}",c.state.inactive_cards);
    engine::validate_runtime(&c.state).unwrap();
}

#[test]
fn m10_growth_blood_for_blood_observation_is_pure_and_exhume_does_not_replay_history(){
    let mut c=combat_with(CardId::Exhume,false);
    let bfb=card_state::permanent(CardId::BloodForBlood,CardInstanceId(88),0);
    c.state.card_ids_issued.insert(card_state::handle(CardInstanceId(88)));
    c.state.piles.exhaust.push(OwnedCard::Full(bfb));
    c.enqueue_native_effects(vec![Effect{source:Source::None,target:Some(Actor::Player),kind:EffectKind::Damage(DamagePayload::HpLoss(2))}],None).unwrap();
    let events=c.state.damage_events_this_combat;
    let first=c.public_full().unwrap();let second=c.public_full().unwrap();
    assert_eq!(first,second);assert_eq!(c.state.damage_events_this_combat,events,"observing cannot fabricate a damage event/cost callback");
    assert_eq!(card_state::effective_cost(c.state.piles.exhaust[0].full().unwrap(),4).unwrap(),4,"exhaust misses historical live callbacks");
    play(&mut c,CardId::Exhume,None);
    let returned=c.state.piles.hand.iter().filter_map(OwnedCard::full).find(|x|x.instance==CardInstanceId(88)).unwrap();
    assert_eq!(card_state::effective_cost(returned,4).unwrap(),4,"Exhume returns the same object without fresh-constructor catch-up");
}

#[test]
fn m10_growth_rampage_grows_same_family_after_its_hit_without_master_mutation(){
    let mut c=combat_with(CardId::Rampage,false);
    c.state.enemies[0].block=0;
    let source=instance(&c,CardId::Rampage);
    let sibling=card(&c,CardId::Rampage).same_instance_family_copy(CardInstanceId(91));
    c.state.piles.discard.push(OwnedCard::Full(sibling));
    let draw_sibling=card(&c,CardId::Rampage).same_instance_family_copy(CardInstanceId(92));
    c.state.piles.draw.push(OwnedCard::Full(draw_sibling));
    let exhaust_sibling=card(&c,CardId::Rampage).same_instance_family_copy(CardInstanceId(93));
    c.state.piles.exhaust.push(OwnedCard::Full(exhaust_sibling));
    let independent=card(&c,CardId::Rampage).copy_as(CardInstanceId(94));
    c.state.piles.hand.push(OwnedCard::Full(independent));
    let inactive=card(&c,CardId::Rampage).same_instance_family_copy(CardInstanceId(95));
    c.state.inactive_cards.push(OwnedCard::Full(inactive));
    for id in [91,92,93,94,95]{register(&mut c,id)}
    engine::validate_runtime(&c.state).unwrap();
    let before=c.public_full().unwrap().enemies[0].hp;
    play(&mut c,CardId::Rampage,Some(TargetId(1)));
    assert_eq!(before-c.public_full().unwrap().enemies[0].hp,8);
    let family=c.state.piles.all(c.state.in_play.as_ref()).filter_map(OwnedCard::full).filter(|x|x.id==CardId::Rampage&&x.copy_family==3).collect::<Vec<_>>();
    assert_eq!(family.len(),4);
    assert!(family.iter().all(|x|x.local.base_damage==Some(13)));
    assert!(c.state.piles.hand.iter().filter_map(OwnedCard::full).any(|x|x.id==CardId::Rampage&&x.local.base_damage==Some(8)),"stat-equivalent independent lineage is not grown");
    assert_eq!(c.state.inactive_cards[0].full().unwrap().local.base_damage,Some(8),"inactive census custody is not a Rampage battle instance");
    assert!(c.state.piles.discard.iter().chain(&c.state.piles.hand).filter_map(OwnedCard::full).any(|x|x.id==CardId::Rampage&&x.instance.0==source));
    engine::validate_runtime(&c.state).unwrap();
}

#[test]
fn m10_growth_rampage_post_hit_growth_survives_spiker_thorns_reaction(){
    let mut entry=super::m8_shapes::single(crate::content::EnemyId::Spiker);
    entry.cards[0]=card_state::permanent(CardId::Rampage,CardInstanceId(88),0);entry.opening_draw=1;
    let mut c=Combat::enter(entry).unwrap();c.state.enemies[0].block=0;
    let hp=c.state.player.hp;
    play(&mut c,CardId::Rampage,Some(TargetId(1)));
    assert!(c.state.player.hp<hp,"Spiker's actual THORNS retaliation resolves through the same accepted attack");
    let grown=c.state.piles.discard.iter().filter_map(OwnedCard::full).find(|card|card.id==CardId::Rampage).unwrap();
    assert_eq!(grown.local.base_damage,Some(13),"the explicit single post-hit operation grows once despite the reaction");
    engine::validate_runtime(&c.state).unwrap();
}

#[test]
fn m10_growth_rampage_double_tap_repeat_uses_the_already_grown_same_instance(){
    let mut c=combat_with(CardId::Rampage,false);
    c.state.player_full_powers.push(FullPowerEntry{id:FullPowerId("double_tap"),state:PowerState{amount:1,skip_first:false,monster_source_grace:false}});
    c.state.enemies[0].block=0;let before=c.state.enemies[0].hp;
    play(&mut c,CardId::Rampage,Some(TargetId(1)));
    assert_eq!(before-c.state.enemies[0].hp,21,"first play hits for 8 and its repeat sees the +5 growth");
    let grown=c.state.piles.discard.iter().filter_map(OwnedCard::full).find(|x|x.id==CardId::Rampage).unwrap();
    assert_eq!(grown.local.base_damage,Some(18),"the accepted repeat also performs Rampage's post-hit growth once");
}

#[test]
fn m10_growth_rampage_queues_one_post_hit_operation_and_clamps_same_family_damage(){
    let mut c=combat_with(CardId::Rampage,false);
    let action=CombatAction::PlayCard{instance:instance(&c,CardId::Rampage),target:Some(TargetId(1))};
    let prepared=commit_prepared_without_draining(&mut c,&action);
    assert_eq!(prepared.iter().filter(|effect|matches!(effect.kind,EffectKind::Ironclad(IroncladEffect::RampageGrow{..}))).count(),1,
        "Rampage prepares one explicit post-hit family operation, not a generic owner-damage hook");
    c.state.in_play.as_mut().unwrap().full_mut().unwrap().local.base_damage=Some(-3);
    engine::validate_runtime(&c.state).unwrap();
    assert_eq!(engine::resolve(&mut c.state).status,TransitionStatus::Decision);
    let grown=c.state.piles.discard.iter().filter_map(OwnedCard::full).find(|card|card.id==CardId::Rampage).unwrap();
    assert_eq!(grown.local.base_damage,Some(2),"Rampage adds +5 to -3, then applies its native lower clamp");
}

#[test]
fn m10_growth_dropkick_uses_shared_fixed_damage_bounds_for_supported_strength(){
    let mut c=combat_with(CardId::Dropkick,false);
    c.state.player_full_powers.push(FullPowerEntry{id:FullPowerId("strength"),state:PowerState{amount:999,skip_first:false,monster_source_grace:false}});
    c.state.enemies[0].full_powers.push(FullPowerEntry{id:FullPowerId("vulnerable"),state:PowerState{amount:999,skip_first:false,monster_source_grace:false}});
    let action=CombatAction::PlayCard{instance:instance(&c,CardId::Dropkick),target:Some(TargetId(1))};
    let prepared=cards::prepare(&c,&action).unwrap();
    let damage=match prepared.effects[0].kind {EffectKind::Ironclad(IroncladEffect::DropkickDispatch{damage})=>damage,_=>panic!("Dropkick dispatch operation")};
    assert!(damage>999&&damage<=30_012,"canonical shared Fixed damage range, not a Dropkick-specific cap: {damage}");
    // The accepted, owned queue may carry the same canonical payload bounds
    // as any other native NORMAL attack.
    let queued=commit_prepared_without_draining(&mut c,&action);
    assert_eq!(queued,prepared.effects);
    assert!(engine::validate_runtime(&c.state).is_ok(),"legitimate supported Strength/Vulnerable output must validate");
}

#[test]
fn m10_growth_rampage_overflow_keeps_the_post_hit_family_mutation_atomic(){
    let mut c=combat_with(CardId::Rampage,false);
    let source_id=instance(&c,CardId::Rampage);
    let action=CombatAction::PlayCard{instance:source_id,target:Some(TargetId(1))};
    let _=commit_prepared_without_draining(&mut c,&action);
    let mut queued=card(&c,CardId::Rampage).same_instance_family_copy(CardInstanceId(996));
    queued.local.base_damage=Some(i32::MAX);queued.local.free_play=true;queued.local.purge=true;
    let child=OwnedCard::Full(queued);let handle=child.handle();
    c.state.card_ids_issued.insert(handle.clone());c.state.queued_autoplay.insert(handle.clone(),child);
    c.state.pending.insert(2,Effect{source:Source::Card(card_state::handle(CardInstanceId(source_id))),target:None,
        kind:EffectKind::Ironclad(IroncladEffect::DoubleTapRepeat{child:handle.clone(),target:Some(Actor::Enemy(c.state.enemies[0].instance_id.clone())),energy_on_use:3,cancelled:false})});
    engine::validate_runtime(&c.state).unwrap();
    let _=engine::resolve(&mut c.state);
    assert!(c.state.fault.is_some(),"unrepresentable selected Rampage increment retains a native fault");
    assert_eq!(c.state.in_play.as_ref().unwrap().full().unwrap().local.base_damage,Some(8),
        "preflight prevents a partial source-card increment before the queued copy overflow");
    assert_eq!(c.state.queued_autoplay[&handle].full().unwrap().local.base_damage,Some(i32::MAX));
}

#[test]
fn m10_growth_dropkick_missing_target_rejects_atomically(){
    let mut c=combat_with(CardId::Dropkick,false);let before=c.clone();
    assert_eq!(c.step_full(&CombatAction::PlayCard{instance:instance(&c,CardId::Dropkick),target:None}),Err(crate::run::RunError::InvalidAction));
    assert_eq!(c,before);
}

#[test]
fn m10_growth_dropkick_preparation_is_read_only_and_captures_the_attack(){
    let c=combat_with(CardId::Dropkick,false);
    let action=CombatAction::PlayCard{instance:instance(&c,CardId::Dropkick),target:Some(TargetId(1))};
    let before=c.clone();let prepared=cards::prepare(&c,&action).unwrap();
    assert_eq!(c,before,"payload preparation remains read-only");
    assert_eq!(prepared.remaining_energy,2);
    assert!(matches!(prepared.effects[0].kind,EffectKind::Ironclad(IroncladEffect::DropkickDispatch{damage:5})));
    assert_eq!(prepared.effects[0].target,Some(Actor::Enemy(c.state.enemies[0].instance_id.clone())));
}

#[test]
fn m10_growth_dropkick_queued_operation_observes_added_or_removed_vulnerable(){
    for (initial,change,want_reward) in [(false,true,true),(true,false,false)] {
        let mut c=combat_with(CardId::Dropkick,false);
        if initial {c.state.enemies[0].full_powers.push(FullPowerEntry{id:FullPowerId("vulnerable"),state:PowerState{amount:1,skip_first:false,monster_source_grace:false}});}
        let fixture=card_state::permanent(CardId::Strike,CardInstanceId(899),0);
        c.state.card_ids_issued.insert(card_state::handle(fixture.instance));c.state.piles.draw.push(OwnedCard::Full(fixture));
        let action=CombatAction::PlayCard{instance:instance(&c,CardId::Dropkick),target:Some(TargetId(1))};
        let strikes_before=c.state.piles.hand.iter().filter_map(OwnedCard::full).filter(|x|x.id==CardId::Strike).count();
        let prepared=commit_prepared_without_draining(&mut c,&action);
        assert!(prepared.iter().any(|effect|matches!(effect.kind,EffectKind::Ironclad(IroncladEffect::DropkickDispatch{..}))));
        let change_effect=Effect{source:Source::None,target:Some(Actor::Enemy(c.state.enemies[0].instance_id.clone())),kind:if change{
            EffectKind::ApplyFullPower{id:FullPowerId("vulnerable"),amount:1}
        }else{EffectKind::RemoveFullPower(FullPowerId("vulnerable"))}};
        c.state.pending.push_front(change_effect);
        let energy=c.state.player.energy;
        assert_eq!(engine::resolve(&mut c.state).status,TransitionStatus::Decision);
        assert_eq!(c.state.player.energy,energy+i32::from(want_reward));
        assert_eq!(c.state.piles.hand.iter().filter_map(OwnedCard::full).filter(|x|x.id==CardId::Strike).count(),strikes_before+usize::from(want_reward));
        engine::validate_runtime(&c.state).unwrap();
    }
}

#[test]
fn m10_growth_dropkick_works_with_double_tap_repeat(){
    let mut c=combat_with(CardId::Dropkick,false);
    c.state.player_full_powers.push(FullPowerEntry{id:FullPowerId("double_tap"),state:PowerState{amount:1,skip_first:false,monster_source_grace:false}});
    c.state.enemies[0].block=0;let before=c.state.enemies[0].hp;
    play(&mut c,CardId::Dropkick,Some(TargetId(1)));
    assert_eq!(before-c.state.enemies[0].hp,10,"the ordinary targeted growth attack remains compatible with the native free repeat route");
    assert!(c.state.piles.discard.iter().any(|owned|owned.full().is_some_and(|x|x.id==CardId::Dropkick)));
}

#[test]
fn m10_growth_dropkick_works_as_havoc_autoplay_child(){
    let mut c=combat_with(CardId::Havoc,false);
    let child=card_state::permanent(CardId::Dropkick,CardInstanceId(99),0);
    c.state.piles.draw.clear();c.state.piles.draw.push(OwnedCard::Full(child));
    c.state.card_ids_issued.insert(card_state::handle(CardInstanceId(99)));
    let before=c.state.enemies[0].hp;
    c.step_full(&CombatAction::PlayCard{instance:instance(&c,CardId::Havoc),target:None}).unwrap();
    assert_eq!(before-c.state.enemies[0].hp,5,"Havoc resolves the actual targeted payload using the accepted captured target");
    assert!(c.state.piles.exhaust.iter().any(|owned|owned.full().is_some_and(|x|x.instance==CardInstanceId(99)&&x.id==CardId::Dropkick)));
}

#[test]
fn m10_growth_dropkick_always_hits_but_rewards_only_if_vulnerable_at_dispatch(){
    for (vulnerable,reward) in [(false,0),(true,1)] {
        let mut c=combat_with(CardId::Dropkick,false);
        if vulnerable {c.state.enemies[0].full_powers.push(FullPowerEntry{id:FullPowerId("vulnerable"),state:PowerState{amount:1,skip_first:false,monster_source_grace:false}});}
        c.state.enemies[0].block=0;
        c.state.piles.draw.push(OwnedCard::Full(card_state::permanent(CardId::Strike,CardInstanceId(99),0)));
        register(&mut c,99);engine::validate_runtime(&c.state).unwrap();
        let hp=c.state.enemies[0].hp;let energy=c.state.player.energy;
        play(&mut c,CardId::Dropkick,Some(TargetId(1)));
        assert_eq!(hp-c.state.enemies[0].hp,if vulnerable{7}else{5});
        assert_eq!(c.state.player.energy,energy-1+reward);
        assert_eq!(c.state.piles.hand.len(),2+reward as usize,"only a qualified Dropkick draws");
        engine::validate_runtime(&c.state).unwrap();
    }
}

#[test]
fn m10_growth_cards_publish_literal_cost_values_and_disposition(){
    let rows=[(CardId::Flex,0,false),(CardId::LimitBreak,1,true),(CardId::BloodForBlood,4,false),(CardId::Rampage,1,false),(CardId::Dropkick,1,false)];
    for (id,cost,exhaust) in rows {
        let c=combat_with(id,false);let live=card(&c,id);
        assert_eq!(cards::base_cost(&c.state,live),Some(cost),"{id:?}");
        assert_eq!(cards::ironclad::rules::public_values(live).unwrap().exhaust_on_finish,exhaust,"{id:?}");
        assert_eq!(Catalog::standard().card(id).unwrap().definition_only,false,"{id:?} has an executable handler, not a definition-only row");
    }
    let upgraded=combat_with(CardId::BloodForBlood,true);
    assert_eq!(cards::base_cost(&upgraded.state,card(&upgraded,CardId::BloodForBlood)),Some(3));
    assert_eq!(Catalog::standard().card(CardId::BloodForBlood).unwrap().rarity,CardRarity::Uncommon);
    let flex=combat_with(CardId::Flex,false);let rampage=combat_with(CardId::Rampage,true);
    assert_eq!(cards::ironclad::rules::public_values(card(&flex,CardId::Flex)).unwrap().magic_number,Some(2));
    assert_eq!(cards::ironclad::rules::public_values(card(&rampage,CardId::Rampage)).unwrap().magic_number,Some(8));
}

#[test]
fn m10_growth_generated_and_stat_copied_limit_break_keep_native_exhaust_flags(){
    for (upgrades,exhaust) in [(0,true),(1,false)] {
        let generated=card_state::generated(CardId::LimitBreak,CardInstanceId(880+u64::from(upgrades)),upgrades).unwrap();
        assert_eq!(generated.local.exhaust_on_finish,exhaust,"fresh generated disposition");
        let copied=generated.stat_equivalent_copy(CardInstanceId(890+u64::from(upgrades)));
        assert_eq!(copied.local.exhaust_on_finish,exhaust,"stat-equivalent copy disposition");
    }
}
