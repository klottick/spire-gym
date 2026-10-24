//! Native Ironclad potion actions and their owned continuations.
use super::super::*;
use crate::{content::{CardId, Character, EncounterId, PotionId, RelicId}, items::CombatItemContext};

fn entry() -> CombatEntry {
    let mut e=super::m8_shapes::entry(EncounterId::OrbWalker);
    e.cards=(1..=12).map(|id|permanent_live_card(CardId::Defend,CardInstanceId(id),0)).collect();
    e.opening_draw=5;e.setup.player.draw_per_turn=5;e
}
fn full_hand_entry() -> CombatEntry {
    let mut e=entry();
    e.opening_draw=10;e.setup.player.draw_per_turn=10;
    e
}
fn enter(potion:PotionId,relics:&[RelicId])->Combat {
    let mut inventory=CombatItemContext::new(relics.to_vec(),vec![Some(potion),None,None]);
    Combat::enter_with_item_context(entry(),&mut Some(std::mem::replace(&mut inventory,CombatItemContext::default()))).unwrap()
}

#[test]
fn m10_ironclad_potions_elixir_uses_owned_optional_selection_and_shared_exhaust() {
    let mut c=enter(PotionId::Elixir,&[RelicId::CharonsAshes]);
    assert!(engine::validate_runtime(&c.state).is_ok(),"{:?}",engine::validate_runtime(&c.state));
    c.step_full(&CombatAction::UsePotion{slot:0,target:None}).unwrap();
    let selection=c.state.pending_selection.as_ref().expect("Elixir opens its owned selector");
    assert_eq!(selection.parent,SelectionParent::Potion{slot:0,id:PotionId::Elixir});
    assert_eq!(selection.minimum,0);
    assert_eq!(selection.maximum,5);
    let enemy_hp_before=c.state.enemies[0].hp;
    assert_eq!(c.item_context().unwrap().potions()[0],None,"the consumed slot stays consumed while selection is pending");
    assert!(c.state.in_play.is_none(),"potion selection has no fake card source");
    let candidates=c.state.pending_selection.as_ref().unwrap().candidates.iter().map(|id|id.as_str().parse::<u64>().unwrap()).collect::<Vec<_>>();
    for instance in candidates { c.step_full(&CombatAction::SelectCard{instance}).unwrap(); }
    assert_eq!(c.state.pending_selection.as_ref().unwrap().selected.len(),5,"reaching the maximum still waits for explicit confirmation");
    c.step_full(&CombatAction::ConfirmSelection).unwrap();
    assert!(c.state.pending_selection.is_none());
    assert_eq!(c.state.piles.exhaust.len(),5);
    assert_eq!(c.state.enemies[0].hp,enemy_hp_before-15,"Ashes listens to each real shared exhaust");
    assert!(engine::validate_runtime(&c.state).is_ok(),"{:?}",engine::validate_runtime(&c.state));
}

#[test]
fn m10_ironclad_potions_elixir_zero_pick_confirms_and_pending_use_is_atomic() {
    let mut c=enter(PotionId::Elixir,&[]);
    c.step_full(&CombatAction::UsePotion{slot:0,target:None}).unwrap();
    let pending=c.snapshot();
    assert_eq!(c.step_full(&CombatAction::UsePotion{slot:1,target:None}),Err(crate::run::RunError::InvalidAction));
    assert_eq!(crate::combat::snapshot::compare(&pending,&c.snapshot()),None,"no second potion can replace the pending owner");
    assert_eq!(c.step_full(&CombatAction::UsePotion{slot:0,target:Some(TargetId(1))}),Err(crate::run::RunError::InvalidAction));
    assert_eq!(crate::combat::snapshot::compare(&pending,&c.snapshot()),None,"Elixir does not accept an unrelated target");
    assert_eq!(c.step_full(&CombatAction::SelectCard{instance:u64::MAX}),Err(crate::run::RunError::InvalidAction));
    assert_eq!(crate::combat::snapshot::compare(&pending,&c.snapshot()),None,"unoffered card identity does not mutate selection");
    c.step_full(&CombatAction::ConfirmSelection).unwrap();
    assert_eq!(c.step_full(&CombatAction::ConfirmSelection),Err(crate::run::RunError::InvalidAction),"a confirmed potion selection cannot be replayed");
    assert!(c.state.piles.exhaust.is_empty(),"zero selected cards is a valid confirmed use");
    assert!(c.item_context().unwrap().potions()[0].is_none(),"the original belt slot remains consumed");
    assert!(engine::validate_runtime(&c.state).is_ok());
}

#[test]
fn m10_ironclad_potions_elixir_rejects_duplicate_selection_and_end_turn_atomically() {
    let mut c=enter(PotionId::Elixir,&[]);
    c.step_full(&CombatAction::UsePotion{slot:0,target:None}).unwrap();
    let instance=c.state.pending_selection.as_ref().unwrap().candidates[0].as_str().parse::<u64>().unwrap();
    c.step_full(&CombatAction::SelectCard{instance}).unwrap();
    let staged=c.snapshot();
    assert_eq!(c.step_full(&CombatAction::SelectCard{instance}),Err(crate::run::RunError::InvalidAction));
    assert_eq!(crate::combat::snapshot::compare(&staged,&c.snapshot()),None,"a selected owner cannot be accepted twice");
    assert_eq!(c.step_full(&CombatAction::EndTurn),Err(crate::run::RunError::InvalidAction));
    assert_eq!(crate::combat::snapshot::compare(&staged,&c.snapshot()),None,"the pending potion continuation blocks end turn without mutation");
    c.step_full(&CombatAction::ConfirmSelection).unwrap();
    assert_eq!(c.state.piles.exhaust.iter().filter(|card|card.handle().as_str()==instance.to_string()).count(),1);
    assert!(engine::validate_runtime(&c.state).is_ok());
}

#[test]
fn m10_ironclad_potions_raw_elixir_selection_without_consumed_inventory_is_atomic() {
    let mut c=enter(PotionId::Elixir,&[]);
    let before=c.snapshot();
    let effect=Effect{source:Source::Potion{slot:0,id:PotionId::Elixir},target:Some(Actor::Player),
        kind:EffectKind::SelectHandCardsAfterDraw{required:0,disposition:SelectionDisposition::Exhaust,empty:EmptySelection::Continue}};
    assert_eq!(c.enqueue_native_effects(vec![effect],None),Err(crate::run::RunError::InvalidAction));
    assert_eq!(crate::combat::snapshot::compare(&before,&c.snapshot()),None,"a producer without the consumed-slot owner cannot leave a selector behind");
    assert!(engine::validate_runtime(&c.state).is_ok());
}

#[test]
fn m10_ironclad_potions_elixir_selection_validator_is_finite_and_full_runtime_only() {
    let c=enter(PotionId::Elixir,&[]);
    let valid=Effect{source:Source::Potion{slot:0,id:PotionId::Elixir},target:Some(Actor::Player),
        kind:EffectKind::SelectHandCardsAfterDraw{required:0,disposition:SelectionDisposition::Exhaust,empty:EmptySelection::Continue}};
    engine::validate_effect(&valid,&c.state).expect("the actual offered Elixir has the one finite selection shape");
    let mut malformed=vec![valid.clone()];
    malformed.push(Effect{kind:EffectKind::SelectHandCardsAfterDraw{required:1,disposition:SelectionDisposition::Exhaust,empty:EmptySelection::Continue},..valid.clone()});
    malformed.push(Effect{kind:EffectKind::SelectHandCardsAfterDraw{required:0,disposition:SelectionDisposition::Discard,empty:EmptySelection::Continue},..valid.clone()});
    malformed.push(Effect{target:None,..valid.clone()});
    malformed.push(Effect{target:Some(Actor::Enemy(c.state.enemies[0].instance_id.clone())),..valid.clone()});
    malformed.push(Effect{source:Source::None,..valid.clone()});
    malformed.push(Effect{source:Source::Potion{slot:3,id:PotionId::Elixir},..valid.clone()});
    malformed.push(Effect{source:Source::Potion{slot:0,id:PotionId::HeartOfIron},..valid.clone()});
    for (index,effect) in malformed.into_iter().enumerate().skip(1){
        let before=c.snapshot();
        assert!(engine::validate_effect(&effect,&c.state).is_err(),"malformed Elixir selection #{index} was admitted");
        assert_eq!(crate::combat::snapshot::compare(&before,&c.snapshot()),None,"validation is read-only");
    }

    let mut legacy=Combat::new(&super::whole_combat::scenario(EnemyKind::Cultist)).unwrap();
    let before=legacy.snapshot();
    let legacy_effect=Effect{source:Source::Potion{slot:0,id:PotionId::Elixir},target:Some(Actor::Player),
        kind:EffectKind::SelectHandCardsAfterDraw{required:0,disposition:SelectionDisposition::Exhaust,empty:EmptySelection::Continue}};
    assert!(engine::validate_effect(&legacy_effect,&legacy.state).is_err());
    assert_eq!(legacy.enqueue_native_effects(vec![legacy_effect],None),Err(crate::run::RunError::InvalidAction));
    assert_eq!(crate::combat::snapshot::compare(&before,&legacy.snapshot()),None,"legacy combat cannot admit the full-runtime potion selection variant");
}

#[test]
fn m10_ironclad_potions_empty_hand_use_consumes_without_opening_selection() {
    let mut c=enter(PotionId::Elixir,&[]);
    let hand=std::mem::take(&mut c.state.piles.hand);
    c.state.piles.discard.extend(hand);
    engine::validate_runtime(&c.state).expect("all issued hand owners remain in a valid discard pile");
    c.step_full(&CombatAction::UsePotion{slot:0,target:None}).unwrap();
    assert_eq!(c.state.phase,Phase::Decision);
    assert!(c.state.pending_selection.is_none());
    assert!(c.state.pending.is_empty());
    assert!(c.state.in_play.is_none());
    assert_eq!(c.item_context().unwrap().potions()[0],None);
    assert!(!c.legal_actions_full().unwrap().contains(&CombatAction::ConfirmSelection));
    assert!(engine::validate_runtime(&c.state).is_ok());
}

#[test]
fn m10_ironclad_potions_elixir_full_ten_card_hand_requires_explicit_confirmation() {
    let mut c=Combat::enter_with_item_context(full_hand_entry(),&mut Some(CombatItemContext::new(vec![],vec![Some(PotionId::Elixir),None,None]))).unwrap();
    assert_eq!(c.state.piles.hand.len(),10);
    c.step_full(&CombatAction::UsePotion{slot:0,target:None}).unwrap();
    for handle in c.state.pending_selection.as_ref().unwrap().candidates.clone(){
        let instance=handle.as_str().parse::<u64>().unwrap();
        c.step_full(&CombatAction::SelectCard{instance}).unwrap();
    }
    let selection=c.state.pending_selection.as_ref().unwrap();
    assert_eq!((selection.selected.len(),selection.maximum),(10,10));
    assert!(c.state.pending_selection.is_some(),"maximum selection does not implicitly confirm");
    c.step_full(&CombatAction::ConfirmSelection).unwrap();
    assert_eq!(c.state.piles.exhaust.len(),10);
    assert!(c.state.pending_selection.is_none());
    assert!(engine::validate_runtime(&c.state).is_ok());
}

#[test]
fn m10_ironclad_potions_elixir_selected_group_keeps_action_order_and_real_listeners() {
    let mut e=entry();
    e.cards[0]=permanent_live_card(CardId::Sentinel,CardInstanceId(1),0);
    let mut c=Combat::enter_with_item_context(e,&mut Some(CombatItemContext::new(vec![RelicId::CharonsAshes],vec![Some(PotionId::Elixir),None,None]))).unwrap();
    let sentinel=if let Some(index)=c.state.piles.hand.iter().position(|card|card.full().is_some_and(|live|live.id==CardId::Sentinel)){
        c.state.piles.hand[index].handle()
    }else{
        let index=c.state.piles.draw.iter().position(|card|card.full().is_some_and(|live|live.id==CardId::Sentinel)).expect("issued Sentinel remains in a pile");
        let card=c.state.piles.draw.remove(index);
        let displaced=c.state.piles.hand.remove(0);
        c.state.piles.draw.insert(index,displaced);
        let handle=card.handle();c.state.piles.hand.push(card);handle
    };
    c.state.player_full_powers.push(FullPowerEntry{id:FullPowerId("feel_no_pain"),state:PowerState{amount:3,skip_first:false,monster_source_grace:false}});
    c.state.player_full_powers.push(FullPowerEntry{id:FullPowerId("dark_embrace"),state:PowerState{amount:1,skip_first:false,monster_source_grace:false}});
    engine::validate_runtime(&c.state).unwrap();
    let defend=c.state.piles.hand.iter().find(|card|card.full().is_some_and(|live|live.id==CardId::Defend)).unwrap().handle();
    let eligible=[defend.as_str().parse::<u64>().unwrap(),sentinel.as_str().parse::<u64>().unwrap()];
    c.step_full(&CombatAction::UsePotion{slot:0,target:None}).unwrap();
    for instance in eligible.iter(){c.step_full(&CombatAction::SelectCard{instance:*instance}).unwrap();}
    let observed=c.step_full_observed(&CombatAction::ConfirmSelection);
    observed.result.unwrap();
    let exhausts=c.public_full().unwrap().exhaust.iter().map(|card|card.instance.0).collect::<Vec<_>>();
    assert_eq!(exhausts,eligible,"exhaust custody keeps the user's selected-group order");
    let moves=observed.effects.iter().enumerate().filter_map(|(index,event)|
        matches!(event.kind,ResolvedEffectKind::CardMove{after:PublicZone::Exhaust,..}).then_some(index)).collect::<Vec<_>>();
    let first_child=observed.effects.iter().position(|event|matches!(event.kind,
        ResolvedEffectKind::Damage{category:DamageCategory::Thorns,..}|ResolvedEffectKind::Block{..}
            |ResolvedEffectKind::Energy{..}|ResolvedEffectKind::CardMove{after:PublicZone::Hand,..})).unwrap();
    assert_eq!(moves.len(),2);assert!(moves.iter().all(|index|*index<first_child),"all selected transfers commit before queued listener children");
    let callbacks=observed.effects.iter().filter_map(|event|match event.kind{
        ResolvedEffectKind::Damage{category:DamageCategory::Thorns,..}=>Some("ashes"),
        ResolvedEffectKind::Block{..}=>Some("feel_no_pain"),
        ResolvedEffectKind::CardMove{after:PublicZone::Hand,..}=>Some("dark_embrace_draw"),
        ResolvedEffectKind::Energy{..}=>Some("sentinel_energy"),_=>None,
    }).collect::<Vec<_>>();
    assert_eq!(callbacks,vec!["ashes","feel_no_pain","dark_embrace_draw","ashes","feel_no_pain","dark_embrace_draw","sentinel_energy"],
        "each selected original runs top-queued Ashes before power/card children, then the next selected owner's listeners");
    let ashes=observed.effects.iter().filter(|event|matches!(event.kind,ResolvedEffectKind::Damage{category:DamageCategory::Thorns,..})).collect::<Vec<_>>();
    assert_eq!(ashes.len(),2);
    assert!(ashes.iter().all(|event|event.source==Source::None),"the relic's actual DamageAllEnemies child has null DamageInfo source");
    assert!(engine::validate_runtime(&c.state).is_ok());
}

#[test]
fn m10_ironclad_potions_elixir_rejects_unoffered_slot_without_consuming_owner() {
    let mut c=enter(PotionId::Elixir,&[]);
    let before=c.snapshot();
    assert_eq!(c.step_full(&CombatAction::UsePotion{slot:1,target:None}),Err(crate::run::RunError::InvalidAction));
    assert_eq!(crate::combat::snapshot::compare(&before,&c.snapshot()),None);
    assert!(engine::validate_runtime(&c.state).is_ok());
}

#[test]
fn m10_ironclad_potions_elixir_retains_consumption_and_selected_owner_on_exhaust_fault() {
    let mut c=enter(PotionId::Elixir,&[RelicId::CharonsAshes]);
    c.step_full(&CombatAction::UsePotion{slot:0,target:None}).unwrap();
    let selected=c.state.pending_selection.as_ref().unwrap().candidates.iter().take(2)
        .map(|handle|handle.as_str().parse::<u64>().unwrap()).collect::<Vec<_>>();
    for instance in &selected{c.step_full(&CombatAction::SelectCard{instance:*instance}).unwrap();}
    c.state.resolved_effects=99_998;
    let result=c.step_full(&CombatAction::ConfirmSelection);
    assert!(matches!(result,Err(crate::run::RunError::Faulted(_))));
    assert_eq!(c.state.fault.as_ref().unwrap().kind,FaultKind::EffectGuard);
    assert!(c.state.fault_selection.as_ref().is_some_and(|selection|matches!(selection.parent,SelectionParent::Potion{slot:0,id:PotionId::Elixir})&&selection.selected.len()==2));
    assert_eq!(c.item_context().unwrap().potions()[0],None);
    assert!(selected.iter().all(|instance|c.state.piles.exhaust.iter().any(|card|card.handle().as_str()==instance.to_string())),"both accepted transfers commit before the callback budget faults");
    assert!(c.state.pending.iter().any(|effect|effect.kind==EffectKind::Damage(DamagePayload::Thorns(3)))
        ||c.state.current.as_ref().is_some_and(|effect|effect.kind==EffectKind::Damage(DamagePayload::Thorns(3))),
        "the committed Ashes child remains pending or current behind the first callback fault: pending={:?} current={:?}",c.state.pending,c.state.current);
    assert!(engine::validate_runtime(&c.state).is_ok());
    let frozen=c.snapshot();
    assert!(c.step_full(&CombatAction::ConfirmSelection).is_err());
    assert_eq!(crate::combat::snapshot::compare(&frozen,&c.snapshot()),None,"retained fault cannot retry the consumed potion or callback");
}

#[test]
fn m10_ironclad_potions_heart_of_iron_applies_metallicize_for_player_checkpoint() {
    let mut c=enter(PotionId::HeartOfIron,&[]);
    c.state.player_full_powers.push(FullPowerEntry{id:FullPowerId("metallicize"),state:PowerState{amount:2,skip_first:false,monster_source_grace:false}});
    c.state.player_full_powers.push(FullPowerEntry{id:FullPowerId("dexterity"),state:PowerState{amount:4,skip_first:false,monster_source_grace:false}});
    c.state.player_full_powers.push(FullPowerEntry{id:FullPowerId("frail"),state:PowerState{amount:1,skip_first:false,monster_source_grace:false}});
    engine::validate_runtime(&c.state).unwrap();
    c.step_full(&CombatAction::UsePotion{slot:0,target:None}).unwrap();
    assert_eq!(c.state.player_full_powers.iter().find(|p|p.id.0=="metallicize").map_or(0,|p|p.state.amount),8);
    assert_eq!(c.state.player.block,0);
    let ended=c.step_full_observed(&CombatAction::EndTurn);
    ended.result.unwrap();
    let player_blocks=ended.effects.iter().filter(|event|event.source==Source::Actor(Actor::Player)
        &&event.target==Some(Actor::Player)&&matches!(event.kind,ResolvedEffectKind::Block{..}))
        .map(|event|event.kind.clone()).collect::<Vec<_>>();
    assert_eq!(player_blocks,vec![ResolvedEffectKind::Block{before:0,after:8}],
        "the real checkpoint stacks native Metallicize as raw Block: neither Dexterity nor Frail changes it, and enemy turn adds no second player gain");
    assert!(engine::validate_runtime(&c.state).is_ok());
}

#[test]
fn m10_ironclad_potions_heart_of_iron_sacred_bark_doubles_the_power_request() {
    let mut c=enter(PotionId::HeartOfIron,&[RelicId::SacredBark]);
    c.state.player_full_powers.push(FullPowerEntry{id:FullPowerId("metallicize"),state:PowerState{amount:2,skip_first:false,monster_source_grace:false}});
    c.state.player_full_powers.push(FullPowerEntry{id:FullPowerId("dexterity"),state:PowerState{amount:4,skip_first:false,monster_source_grace:false}});
    c.state.player_full_powers.push(FullPowerEntry{id:FullPowerId("frail"),state:PowerState{amount:1,skip_first:false,monster_source_grace:false}});
    c.step_full(&CombatAction::UsePotion{slot:0,target:None}).unwrap();
    assert_eq!(c.state.player_full_powers.iter().find(|p|p.id.0=="metallicize").map_or(0,|p|p.state.amount),14);
    let ended=c.step_full_observed(&CombatAction::EndTurn);ended.result.unwrap();
    let player_blocks=ended.effects.iter().filter(|event|event.source==Source::Actor(Actor::Player)
        &&event.target==Some(Actor::Player)&&matches!(event.kind,ResolvedEffectKind::Block{..}))
        .map(|event|event.kind.clone()).collect::<Vec<_>>();
    assert_eq!(player_blocks,vec![ResolvedEffectKind::Block{before:0,after:14}],"Sacred Bark's doubled raw request stacks with Metallicize exactly once");
    assert!(engine::validate_runtime(&c.state).is_ok());
}

#[test]
fn m10_ironclad_potions_catalog_marks_the_three_exclusive_ids_active() {
    for id in [PotionId::BloodPotion,PotionId::Elixir,PotionId::HeartOfIron] {
        assert!(crate::content::Catalog::standard().potion(id).is_some_and(|p|!p.definition_only),"{id:?}");
    }
    assert_eq!(Character::Ironclad,entry().setup.character);
}
