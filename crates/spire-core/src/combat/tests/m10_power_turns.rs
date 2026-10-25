use super::super::*;
use super::full_runtime::entered_combat;
use crate::{content::{CardId,Catalog,CardColor,CardRarity,CardType},combat::model::OwnedCard};

fn combat_with(id:CardId,upgraded:bool)->Combat{
    entered_combat(3,3,|card|*card=card_state::permanent(id,card.instance,u32::from(upgraded)))
}
fn play(c:&mut Combat,id:CardId){
    let instance=c.state.piles.hand.iter().filter_map(OwnedCard::full).find(|card|card.id==id).unwrap().instance.0;
    c.step_full(&CombatAction::PlayCard{instance,target:None}).unwrap();
}
fn play_instance(c:&mut Combat,instance:u64){c.step_full(&CombatAction::PlayCard{instance,target:None}).unwrap();}
fn amount(c:&Combat,id:&str)->i32{
    c.state.player_full_powers.iter().find(|power|power.id.0==id).map_or(0,|power|power.state.amount)
}
fn add_draw_cards(c:&mut Combat,count:u64){
    for instance in 100..100+count{
        let card=card_state::permanent(CardId::Defend,CardInstanceId(instance),0);
        c.state.card_ids_issued.insert(card_state::handle(card.instance));
        c.state.piles.draw.push(OwnedCard::Full(card));
    }
}
fn start_next_turn(c:&mut Combat){
    turns::schedule_next_player_turn(&mut c.state).unwrap();
    engine::resolve(&mut c.state);
    assert!(c.state.fault.is_none(),"{:?}",c.fault_record());
}
fn full_power(id:&'static str,amount:i32)->FullPowerEntry{
    FullPowerEntry{id:FullPowerId(id),state:PowerState{amount,skip_first:false,monster_source_grace:false}}
}

fn full_deck_with_brutality(upgraded:bool)->CombatEntry{
    let mut entry=super::m8_shapes::entry(crate::content::EncounterId::OrbWalker);
    entry.cards=(1..=8).map(|n|card_state::permanent(
        if n==1{CardId::Brutality}else{CardId::Defend},CardInstanceId(n),u32::from(n==1&&upgraded))).collect();
    entry.opening_draw=1;
    entry
}
fn full_deck_with_innate_brutalities(innate_count:u64,deck_size:u64,opening_draw:usize)->CombatEntry{
    let mut entry=super::m8_shapes::entry(crate::content::EncounterId::OrbWalker);
    entry.cards=(1..=deck_size).map(|n|card_state::permanent(
        if n<=innate_count{CardId::Brutality}else{CardId::Defend},CardInstanceId(n),u32::from(n<=innate_count))).collect();
    entry.opening_draw=opening_draw;
    entry
}
fn time_eater_entry(cards:Vec<LiveCard>,time_warp:i32)->CombatEntry{
    let mut entry=super::m8_shapes::entry(crate::content::EncounterId::TimeEater);
    entry.cards=cards;entry.opening_draw=0;
    entry.setup.enemies[0].powers.iter_mut().find(|power|power.id.0=="time_warp").unwrap().state.amount=time_warp;
    entry.setup.enemies[0].intent=FullIntentId::from_name("time_ripple");
    entry
}
fn move_draw_to_hand(c:&mut Combat,instance:u64){
    let index=c.state.piles.draw.iter().position(|owned|owned.full().is_some_and(|card|card.instance.0==instance)).unwrap();
    let card=c.state.piles.draw.remove(index);c.state.public.revealed_order.push(card.handle());c.state.piles.hand.push(card);
}
fn enemy_amount(c:&Combat,slot:usize,id:&str)->i32{c.state.enemies[slot].full_powers.iter().find(|power|power.id.0==id).map_or(0,|power|power.state.amount)}

#[test]
fn m10_power_turns_catalog_rows_are_executable_ironclad_powers(){
    for (id,rarity) in [(CardId::Barricade,CardRarity::Rare),(CardId::Berserk,CardRarity::Rare),
        (CardId::Brutality,CardRarity::Rare),(CardId::DemonForm,CardRarity::Rare),
        (CardId::Metallicize,CardRarity::Uncommon),(CardId::Combust,CardRarity::Uncommon)]{
        let row=Catalog::standard().card(id).unwrap();
        assert_eq!((row.color,row.rarity,row.card_type,row.definition_only),
            (CardColor::Red,rarity,CardType::Power,false),"{id:?}");
    }
}

#[test]
fn m10_power_turns_card_costs_values_and_upgrade_flags_are_literal(){
    let cases=[(CardId::Barricade,3,2,None,None),(CardId::Berserk,0,0,Some(2),Some(1)),
        (CardId::Brutality,0,0,None,None),(CardId::DemonForm,3,3,Some(2),Some(3)),
        (CardId::Metallicize,1,1,Some(3),Some(4)),(CardId::Combust,1,1,Some(5),Some(7))];
    for (id,cost,up_cost,magic,up_magic) in cases{
        for upgraded in [false,true]{
            let c=combat_with(id,upgraded);
            let card=card_state::permanent(id,CardInstanceId(91),u32::from(upgraded));
            assert_eq!(cards::base_cost(&c.state,&card),Some(if upgraded{up_cost}else{cost}),"{id:?}");
            let values=cards::ironclad::rules::public_values(&card).unwrap();
            assert_eq!(values.magic_number,if upgraded{up_magic}else{magic},"{id:?}");
            assert_eq!(values.innate,id==CardId::Brutality&&upgraded,"{id:?}");
        }
    }
}

#[test]
fn m10_power_turns_innate_uses_only_opening_entitlement_without_revealing_deck(){
    let upgraded=Combat::enter(full_deck_with_brutality(true)).unwrap();
    assert!(engine::validate_runtime(&upgraded.state).is_ok());
    let public=upgraded.public_full().unwrap();
    assert_eq!(public.hand.iter().map(|card|card.id).collect::<Vec<_>>(),[CardId::Brutality]);
    assert_eq!(public.draw_count,7);
    assert!(public.known_draw.is_empty());
    assert_eq!(upgraded.state.public.revealed_order.len(),1,"only opening hand identity is revealed");
    let ordinary=Combat::enter(full_deck_with_brutality(false)).unwrap();
    assert!(engine::validate_runtime(&ordinary.state).is_ok());
    assert_eq!(ordinary.public_full().unwrap().draw_count,7);
    assert_eq!(ordinary.state.public.revealed_order.len(),1);
    assert_ne!(ordinary.public_full().unwrap().hand[0].id,CardId::Brutality,
        "same shuffle stream without Innate leaves the card outside ordinary opening draw");
}

#[test]
fn m10_power_turns_multiple_innates_exceed_normal_draw_but_remain_within_hand_cap(){
    let two=Combat::enter(full_deck_with_innate_brutalities(2,6,1)).unwrap();
    assert_eq!(two.state.piles.hand.iter().filter(|card|card.full().unwrap().id==CardId::Brutality).count(),2,
        "all Innate copies are entitled even above the ordinary opening draw");
    assert_eq!(two.state.piles.draw.len(),4);
    assert!(engine::validate_runtime(&two.state).is_ok());

    let capped=Combat::enter(full_deck_with_innate_brutalities(12,12,5)).unwrap();
    assert_eq!(capped.state.piles.hand.len(),10,"the existing hand cap remains authoritative");
    assert_eq!(capped.state.piles.draw.len()+capped.state.piles.hand.len(),12,"over-cap cards remain owned in draw");
    assert!(capped.state.piles.hand.iter().all(|card|card.full().unwrap().id==CardId::Brutality));
    assert!(engine::validate_runtime(&capped.state).is_ok());
}

#[test]
fn m10_power_turns_generated_power_constructors_cover_all_six_supported_ids(){
    let cases=[(CardId::Barricade,3,2,None,None),(CardId::Berserk,0,0,Some(2),Some(1)),
        (CardId::Brutality,0,0,None,None),(CardId::DemonForm,3,3,Some(2),Some(3)),
        (CardId::Metallicize,1,1,Some(3),Some(4)),(CardId::Combust,1,1,Some(5),Some(7))];
    for (index,(id,cost,up_cost,magic,up_magic)) in cases.into_iter().enumerate(){
        for upgrades in [0,1]{
            let card=card_state::generated(id,CardInstanceId(400+index as u64*2+u64::from(upgrades)),upgrades)
                .unwrap_or_else(||panic!("missing Task10 generated constructor for {id:?}"));
            assert_eq!(card.id,id);
            assert!(!card.local.exhaust_on_finish,"Power disposition remains non-exhausting: {id:?}");
            assert_eq!(card.local.innate,id==CardId::Brutality&&upgrades>0);
            assert_eq!(cards::base_cost(&combat_with(id,upgrades>0).state,&card),Some(if upgrades>0{up_cost}else{cost}),"{id:?}");
            assert_eq!(cards::ironclad::rules::public_values(&card).unwrap().magic_number,
                if upgrades>0{up_magic}else{magic},"{id:?}");
        }
    }
}

#[test]
fn m10_power_turns_stat_equivalent_copy_rederives_innate_but_adapter_clone_preserves_it(){
    let mut forged=card_state::permanent(CardId::Brutality,CardInstanceId(71),0);
    forged.local.innate=true;
    assert!(forged.copy_as(CardInstanceId(72)).local.innate,"public native adapter clone keeps cloned state");
    assert!(!forged.stat_equivalent_copy(CardInstanceId(73)).local.innate,
        "stat-equivalent copy derives intrinsic Innate from card identity/upgrades");
    assert!(!forged.same_instance_family_copy(CardInstanceId(74)).local.innate);
    let upgraded=card_state::permanent(CardId::Brutality,CardInstanceId(75),1);
    assert!(upgraded.stat_equivalent_copy(CardInstanceId(76)).local.innate,
        "actual upgraded source retains its intrinsic opening entitlement");
}

#[test]
fn m10_power_turns_armaments_upgrade_and_dual_wielded_power_play_once_into_census(){
    let mut entry=super::m8_shapes::entry(crate::content::EncounterId::OrbWalker);
    entry.cards=[CardId::Armaments,CardId::Brutality,CardId::DualWield,CardId::Defend].into_iter()
        .enumerate().map(|(index,id)|card_state::permanent(id,CardInstanceId(index as u64+1),0)).collect();
    entry.opening_draw=4;
    let mut c=Combat::enter(entry).unwrap();
    assert!(engine::validate_runtime(&c.state).is_ok());
    c.step_full(&CombatAction::PlayCard{instance:1,target:None}).unwrap();
    c.step_full(&CombatAction::SelectCard{instance:2}).unwrap();
    assert!(c.state.piles.hand.iter().any(|owned|owned.full().is_some_and(|card|card.id==CardId::Brutality&&card.upgrades==1&&card.local.innate)));
    assert!(engine::validate_runtime(&c.state).is_ok());

    c.step_full(&CombatAction::PlayCard{instance:3,target:None}).unwrap();
    if c.state.phase==Phase::Selection{c.step_full(&CombatAction::SelectCard{instance:2}).unwrap();}
    assert!(engine::validate_runtime(&c.state).is_ok());
    let copied=c.state.piles.hand.iter().filter_map(OwnedCard::full)
        .find(|card|card.id==CardId::Brutality&&card.copy_provenance==card_state::CopyProvenance::StatEquivalent)
        .expect("Dual Wield's actual child is a stat-equivalent Power copy").instance.0;
    assert!(c.state.piles.hand.iter().any(|owned|owned.full().is_some_and(|card|card.instance==CardInstanceId(2))),
        "the actual one-candidate Dual Wield path retains its source Power");
    assert!(c.state.piles.hand.iter().any(|owned|owned.full().is_some_and(|card|card.instance.0==copied&&card.upgrades==1&&card.local.innate)));
    c.step_full(&CombatAction::PlayCard{instance:copied,target:None}).unwrap();
    assert_eq!(amount(&c,"brutality"),1);
    assert_eq!(c.public_full().unwrap().played_powers.iter().filter(|card|card.id==CardId::Brutality).count(),1);
    assert!(engine::validate_runtime(&c.state).is_ok());
}

#[test]
fn m10_power_turns_barricade_play_registers_actual_power(){
    let mut c=combat_with(CardId::Barricade,false);play(&mut c,CardId::Barricade);
    assert_eq!(amount(&c,"barricade"),1);
    assert_eq!(c.public_full().unwrap().played_powers.iter().filter(|p|p.id==CardId::Barricade).count(),1);
    c.state.player.block=7;start_next_turn(&mut c);
    assert_eq!(c.state.player.block,7,"Barricade guards only the player's ordinary start reset");
}

#[test]
fn m10_power_turns_barricade_only_guards_start_reset_and_does_not_prevent_block_loss(){
    let mut c=combat_with(CardId::Barricade,false);play(&mut c,CardId::Barricade);c.state.player.block=7;
    let enemy=Actor::Enemy(c.state.enemies[0].instance_id.clone());
    c.enqueue_native_effects(vec![Effect{source:Source::Actor(enemy),target:Some(Actor::Player),
        kind:EffectKind::Damage(DamagePayload::Fixed(2))}],None).unwrap();
    assert_eq!(c.state.player.block,5,"ordinary incoming damage still consumes Block under Barricade");
    start_next_turn(&mut c);assert_eq!(c.state.player.block,5,"only the ordinary player-start reset is guarded");

    let mut control=combat_with(CardId::Defend,false);control.state.player.block=7;
    start_next_turn(&mut control);assert_eq!(control.state.player.block,0,"without Barricade ordinary player-start loss applies");
}

#[test]
fn m10_power_turns_berserk_applies_vulnerable_and_delayed_energy_power(){
    let mut c=combat_with(CardId::Berserk,false);play(&mut c,CardId::Berserk);
    assert_eq!(amount(&c,"vulnerable"),2);
    assert_eq!(amount(&c,"berserk"),1);
    assert_eq!(c.state.player.energy,3,"no energy on application");
    start_next_turn(&mut c);
    assert_eq!(c.state.player.energy,4,"one delayed energy at the ordinary player start hook");
}

#[test]
fn m10_power_turns_berserk_artifact_blocks_only_its_vulnerable_application(){
    let mut c=combat_with(CardId::Berserk,false);c.state.player_full_powers.push(full_power("artifact",1));
    play(&mut c,CardId::Berserk);
    assert_eq!((amount(&c,"vulnerable"),amount(&c,"berserk"),amount(&c,"artifact")),(0,1,0));
    start_next_turn(&mut c);assert_eq!(c.state.player.energy,4);
}

#[test]
fn m10_power_turns_brutality_is_delayed_and_upgraded_card_is_innate(){
    let mut c=combat_with(CardId::Brutality,false);let hp=c.state.player.hp;let hand=c.state.piles.hand.len();
    play(&mut c,CardId::Brutality);
    assert_eq!(amount(&c,"brutality"),1);
    assert_eq!(c.state.player.hp,hp,"no same-turn phantom HP loss");
    assert_eq!(c.state.piles.hand.len(),hand-1,"only the played Power leaves hand; no same-turn extra draw");
    let upgraded=combat_with(CardId::Brutality,true);
    let opening=upgraded.public_full().unwrap();
    assert!(opening.hand.iter().any(|card|card.id==CardId::Brutality),"upgraded Brutality is Innate");
}

#[test]
fn m10_power_turns_brutality_runs_after_normal_draw_then_extra_draw_then_hploss(){
    let mut c=combat_with(CardId::Brutality,false);play(&mut c,CardId::Brutality);
    c.state.piles.discard.append(&mut c.state.piles.hand);add_draw_cards(&mut c,5);c.state.player.draw_per_turn=2;
    let hp=c.state.player.hp;
    turns::schedule_next_player_turn(&mut c.state).unwrap();
    assert_eq!(c.state.pending.front().unwrap().kind,EffectKind::Hook(Hook::TurnStartedPreDraw));
    engine::dispatch_one(&mut c.state);
    assert_eq!(c.state.pending.front().unwrap().kind,EffectKind::Draw(2));
    engine::dispatch_one(&mut c.state);
    while c.state.pending.front().is_some_and(|effect|effect.kind!=EffectKind::Hook(Hook::TurnStarted)){
        engine::dispatch_one(&mut c.state);assert!(c.state.fault.is_none(),"{:?}",c.fault_record());
    }
    assert_eq!(c.state.piles.hand.len(),2,"normal draw completed before the power callback");
    engine::dispatch_one(&mut c.state);
    assert_eq!(c.state.pending.front().unwrap().kind,EffectKind::Draw(1));
    assert_eq!(c.state.pending.get(1).unwrap().kind,EffectKind::Damage(DamagePayload::HpLoss(1)));
    engine::resolve(&mut c.state);
    assert_eq!((c.state.piles.hand.len(),c.state.player.hp),(3,hp-1));
}

#[test]
fn m10_power_turns_brutality_no_draw_and_full_hand_still_pay_only_hp_loss(){
    let mut c=combat_with(CardId::Brutality,false);play(&mut c,CardId::Brutality);
    for instance in 200..208{
        let card=card_state::permanent(CardId::Defend,CardInstanceId(instance),0);
        c.state.card_ids_issued.insert(card_state::handle(card.instance));c.state.piles.hand.push(OwnedCard::Full(card));
    }
    assert_eq!(c.state.piles.hand.len(),10);
    c.state.player_full_powers.push(full_power("no_draw",1));
    let hp=c.state.player.hp;assert!(engine::validate_runtime(&c.state).is_ok());
    start_next_turn(&mut c);
    assert_eq!(c.state.piles.hand.len(),10,"both ordinary and Brutality draws respect No Draw and the cap");
    assert_eq!(c.state.player.hp,hp-1,"the independent post-draw HP_LOSS still runs");
    assert!(engine::validate_runtime(&c.state).is_ok());
}

#[test]
fn m10_power_turns_demon_form_queues_strength_power(){
    let mut c=combat_with(CardId::DemonForm,false);play(&mut c,CardId::DemonForm);
    assert_eq!(amount(&c,"demon_form"),2);
    assert_eq!(amount(&c,"strength"),0,"not immediate on application");
    start_next_turn(&mut c);
    assert_eq!(amount(&c,"strength"),2,"post-draw ApplyPower resolves through the ordinary Strength owner");
}

#[test]
fn m10_power_turns_metallicize_registers_pre_end_power(){
    let mut c=combat_with(CardId::Metallicize,false);play(&mut c,CardId::Metallicize);
    assert_eq!(amount(&c,"metallicize"),3);
    c.state.pending.push_front(Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::EndPlayerTurn});
    engine::dispatch_one(&mut c.state);
    assert_eq!(c.state.pending.front().unwrap().kind,EffectKind::Hook(Hook::PlayerPreEndTurnCards));
    engine::dispatch_one(&mut c.state);
    assert_eq!(c.state.pending.front().unwrap().kind,EffectKind::GainBlock(3));
    engine::dispatch_one(&mut c.state);
    assert_eq!(c.state.player.block,3,"pre-end-card block is committed before enemy turn");
}

#[test]
fn m10_power_turns_ordinary_end_runs_player_metallicize_before_held_burn_once(){
    let cards=vec![card_state::permanent(CardId::Combust,CardInstanceId(1),0),
        card_state::permanent(CardId::Metallicize,CardInstanceId(2),0),
        card_state::permanent(CardId::Burn,CardInstanceId(3),0)];
    let mut c=Combat::enter(time_eater_entry(cards,0)).unwrap();
    assert!(engine::validate_runtime(&c.state).is_ok());
    move_draw_to_hand(&mut c,1);move_draw_to_hand(&mut c,2);
    assert!(engine::validate_runtime(&c.state).is_ok());
    c.step_full(&CombatAction::PlayCard{instance:1,target:None}).unwrap();
    c.step_full(&CombatAction::PlayCard{instance:2,target:None}).unwrap();
    move_draw_to_hand(&mut c,3);assert!(engine::validate_runtime(&c.state).is_ok());
    let hp=c.state.player.hp;
    let observed=c.step_full_observed(&CombatAction::EndTurn);assert!(observed.result.is_ok());
    assert_eq!(c.state.player.hp,hp-1,"one Combust HP_LOSS; end-turn Burn's 2 THORNS is fully absorbed");
    assert_eq!(observed.effects.iter().filter(|effect|effect.target==Some(Actor::Player)&&matches!(effect.kind,
        ResolvedEffectKind::Damage{category:DamageCategory::HpLoss,incoming:1,..})).count(),1);
    assert_eq!(observed.effects.iter().filter(|effect|effect.source==Source::None&&matches!(effect.kind,
        ResolvedEffectKind::Damage{category:DamageCategory::Thorns,incoming:5,..})).count(),1);
    let block=observed.effects.iter().position(|effect|effect.target==Some(Actor::Player)&&matches!(effect.kind,
        ResolvedEffectKind::Block{before:0,after:3})).unwrap();
    let burn=observed.effects.iter().position(|effect|effect.source==Source::Card(card_state::handle(CardInstanceId(3)))&&matches!(effect.kind,
        ResolvedEffectKind::Damage{category:DamageCategory::Thorns,incoming:2,absorbed_block:2,..})).unwrap();
    assert!(block<burn,"player Metallicize precedes real held-card end-turn work");
    assert_eq!(observed.effects.iter().filter(|effect|effect.target==Some(Actor::Player)&&matches!(effect.kind,
        ResolvedEffectKind::Block{after:3,..})).count(),1);
    assert_eq!(c.state.hook_events.iter().filter(|event|event.hook==Hook::TurnEnded).count(),1);
    assert!(engine::validate_runtime(&c.state).is_ok());
}

#[test]
fn m10_power_turns_combust_keeps_hp_cost_separate_from_damage_amount(){
    let mut c=combat_with(CardId::Combust,false);play(&mut c,CardId::Combust);
    assert_eq!(amount(&c,"combust"),5);
}

#[test]
fn m10_power_turns_combust_mixed_upgrade_stacks_damage_sum_and_copy_count(){
    let mut c=combat_with(CardId::Combust,false);
    let second=c.state.piles.hand.iter().filter_map(OwnedCard::full).find(|card|card.id!=CardId::Combust).unwrap().instance;
    *c.state.piles.hand.iter_mut().filter_map(OwnedCard::full_mut).find(|card|card.instance==second).unwrap()=
        card_state::permanent(CardId::Combust,second,1);
    play(&mut c,CardId::Combust);play(&mut c,CardId::Combust);
    assert_eq!(amount(&c,"combust"),12,"base5 plus upgraded7");
    assert_eq!(c.state.combust.unwrap().copies,2,"HP loss counts copies, not damage");
}

#[test]
fn m10_power_turns_combust_stacks_resolve_copy_hp_loss_and_all_canonical_enemies(){
    let mut entry=super::m8_shapes::entry(crate::content::EncounterId::FourShapes);
    entry.cards=[card_state::permanent(CardId::Combust,CardInstanceId(1),0),
        card_state::permanent(CardId::Combust,CardInstanceId(2),1)].into();entry.opening_draw=0;
    let mut c=Combat::enter(entry).unwrap();
    assert_eq!(c.state.enemies.len(),4,"canonical Four Shapes encounter, not cloned actors");
    move_draw_to_hand(&mut c,1);move_draw_to_hand(&mut c,2);c.state.player.block=999;
    assert!(engine::validate_runtime(&c.state).is_ok());
    c.step_full(&CombatAction::PlayCard{instance:1,target:None}).unwrap();
    c.step_full(&CombatAction::PlayCard{instance:2,target:None}).unwrap();
    assert_eq!((amount(&c,"combust"),c.state.combust.unwrap().copies),(12,2));
    let before=c.state.enemies.iter().map(|enemy|enemy.hp).collect::<Vec<_>>();let hp=c.state.player.hp;
    let observed=c.step_full_observed(&CombatAction::EndTurn);assert!(observed.result.is_ok());
    assert_eq!(c.state.player.hp,hp-2,"the two actual copies produce HP_LOSS2 despite Block999");
    assert_eq!(c.state.enemies.iter().map(|enemy|enemy.hp).collect::<Vec<_>>(),before.iter().map(|value|value-12).collect::<Vec<_>>(),
        "one all-enemy THORNS packet carries sum(5+7), reaching every canonical live actor");
    assert_eq!(observed.effects.iter().filter(|effect|effect.target==Some(Actor::Player)&&matches!(effect.kind,
        ResolvedEffectKind::Damage{category:DamageCategory::HpLoss,incoming:2,..})).count(),1);
    assert_eq!(observed.effects.iter().filter(|effect|effect.source==Source::None&&matches!(effect.kind,ResolvedEffectKind::Damage{category:DamageCategory::Thorns,incoming:12,..})).count(),4);
    assert!(engine::validate_runtime(&c.state).is_ok());
}

#[test]
fn m10_power_turns_combust_skips_canonical_all_down_awakened_group_with_halfdead(){
    let mut entry=super::m8_shapes::entry(crate::content::EncounterId::AwakenedOne);
    entry.cards=vec![card_state::permanent(CardId::Combust,CardInstanceId(1),0)];entry.opening_draw=1;
    let mut c=Combat::enter(entry).unwrap();c.step_full(&CombatAction::PlayCard{instance:1,target:None}).unwrap();
    let owners=c.state.enemies.iter().map(|enemy|enemy.instance_id.clone()).collect::<Vec<_>>();
    let fatal=owners.iter().map(|owner|Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Enemy(owner.clone())),
        kind:EffectKind::Damage(DamagePayload::Fixed(999))}).collect();
    c.enqueue_native_effects(fatal,None).unwrap();
    assert_eq!(c.state.outcome,None);assert!(c.state.enemies.iter().all(|enemy|enemy.hp==0));
    assert!(c.state.enemies.iter().any(|enemy|enemy.life==ActorLife::HalfDead));
    assert!(engine::validate_runtime(&c.state).is_ok());
    let observed=c.step_full_observed(&CombatAction::EndTurn);assert!(observed.result.is_ok());
    assert!(!observed.effects.iter().any(|effect|matches!(effect.kind,ResolvedEffectKind::Damage{category:DamageCategory::HpLoss|DamageCategory::Thorns,..})),
        "basically-dead half-dead group skips both Combust actions");
}

#[test]
fn m10_power_turns_combust_setup_and_actor_scope_fail_closed(){
    let mut entry=super::m8_shapes::entry(crate::content::EncounterId::OrbWalker);
    entry.setup.player_powers.push(full_power("combust",5));
    assert_eq!(Combat::enter(entry),Err(crate::run::RunError::InvalidAction),
        "setup carries no qualified per-copy Combust custody and must reject before resolver work");

    let combat=combat_with(CardId::Combust,false);
    let enemy=Actor::Enemy(combat.state.enemies[0].instance_id.clone());
    assert!(engine::validate_effect(&Effect{source:Source::None,target:Some(enemy),
        kind:EffectKind::ApplyFullPower{id:FullPowerId("combust"),amount:5}},&combat.state).is_err(),
        "typed Combust custody is player-only");

    let mut enemy_entry=super::m8_shapes::entry(crate::content::EncounterId::OrbWalker);
    enemy_entry.setup.enemies[0].powers.push(full_power("combust",5));
    assert_eq!(Combat::enter(enemy_entry),Err(crate::run::RunError::InvalidAction),
        "unowned enemy Combust is rejected at construction, not admitted without player custody");
    let mut invalid_runtime=combat_with(CardId::Defend,false);
    invalid_runtime.state.enemies[0].full_powers.push(full_power("combust",5));
    assert!(engine::validate_runtime(&invalid_runtime.state).is_err(),
        "runtime census also rejects enemy Combust outside player copy custody");
}

#[test]
fn m10_power_turns_combust_remove_then_reapply_starts_new_typed_stack(){
    let mut c=combat_with(CardId::Combust,false);
    let second=c.state.piles.hand.iter().filter_map(OwnedCard::full).find(|card|card.id!=CardId::Combust).unwrap().instance;
    *c.state.piles.hand.iter_mut().filter_map(OwnedCard::full_mut).find(|card|card.instance==second).unwrap()=
        card_state::permanent(CardId::Combust,second,1);
    play_instance(&mut c,3);assert_eq!(c.state.combust.unwrap().copies,1);
    assert!(c.state.piles.hand.iter().any(|owned|owned.full().is_some_and(|card|card.instance==second&&card.id==CardId::Combust&&card.upgrades==1)));
    c.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),
        kind:EffectKind::RemoveFullPower(FullPowerId("combust"))}],None).unwrap();
    assert!(c.state.combust.is_none());assert_eq!(amount(&c,"combust"),0);
    play_instance(&mut c,second.0);
    assert_eq!((amount(&c,"combust"),c.state.combust.unwrap().copies),(7,1));
    assert!(engine::validate_runtime(&c.state).is_ok());
}

#[test]
fn m10_power_turns_combust_full_reduction_clears_copy_custody_before_reapplication(){
    let mut c=combat_with(CardId::Combust,false);
    let second=c.state.piles.hand.iter().filter_map(OwnedCard::full).find(|card|card.id!=CardId::Combust).unwrap().instance;
    *c.state.piles.hand.iter_mut().filter_map(OwnedCard::full_mut).find(|card|card.instance==second).unwrap()=
        card_state::permanent(CardId::Combust,second,0);
    play_instance(&mut c,3);assert_eq!(c.state.combust.unwrap().copies,1);
    c.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),
        kind:EffectKind::ReduceFullPower{id:FullPowerId("combust"),amount:5}}],None).unwrap();
    assert_eq!(amount(&c,"combust"),0);assert!(c.state.combust.is_none());
    assert!(engine::validate_runtime(&c.state).is_ok());
    play_instance(&mut c,second.0);
    assert_eq!((amount(&c,"combust"),c.state.combust.unwrap().copies),(5,1));
    assert!(engine::validate_runtime(&c.state).is_ok());
}

#[test]
fn m10_power_turns_combust_copy_custody_matches_reachable_5_or_7_amounts(){
    let mut c=combat_with(CardId::Combust,false);play(&mut c,CardId::Combust);
    assert!(engine::validate_runtime(&c.state).is_ok());
    c.state.combust=Some(super::super::model::CombustState{copies:2});
    assert!(engine::validate_runtime(&c.state).is_err(),
        "a 5-point stack cannot represent two supported Combust copies (minimum sum10)");
}

#[test]
fn m10_power_turns_combust_guard_fault_retains_the_unapplied_card_effect_owner(){
    let mut c=combat_with(CardId::Combust,false);
    let instance=c.state.piles.hand.iter().filter_map(OwnedCard::full).find(|card|card.id==CardId::Combust).unwrap().instance;
    let prepared=cards::prepare(&c,&CombatAction::PlayCard{instance:instance.0,target:None}).unwrap();
    let index=c.state.piles.hand.iter().position(|owned|owned.full().is_some_and(|card|card.instance==instance)).unwrap();
    let card=c.state.piles.hand.remove(index);let source=Source::Card(card.handle());
    c.state.in_play=Some(card);c.state.player.energy=prepared.remaining_energy;c.state.phase=Phase::PlayerAction;
    c.state.pending.push_back(Effect{source:source.clone(),target:None,kind:EffectKind::Hook(Hook::CardPlayed)});
    c.state.pending.extend(prepared.effects);
    c.state.pending.push_back(Effect{source,target:None,kind:EffectKind::FinishCard});
    engine::dispatch_one(&mut c.state);
    assert!(c.state.fault.is_none());
    assert!(matches!(c.state.pending.front().map(|effect|&effect.kind),Some(EffectKind::ApplyFullPower{id,..}) if id.0=="combust"));
    c.state.resolved_effects=99_999;
    engine::dispatch_one(&mut c.state);
    assert_eq!(c.state.fault.as_ref().unwrap().kind,FaultKind::EffectGuard);
    assert!(matches!(c.state.current.as_ref().map(|effect|&effect.kind),Some(EffectKind::ApplyFullPower{id,..}) if id.0=="combust"));
    assert_eq!(c.state.current.as_ref().unwrap().source,Source::Card(card_state::handle(instance)));
    assert_eq!(amount(&c,"combust"),0);assert!(c.state.combust.is_none());
    assert!(engine::validate_runtime(&c.state).is_ok(),"retained owner and sidecar state remain coherent");
}

#[test]
fn m10_power_turns_combust_hploss_precedes_null_owner_thorns_on_timewarp_end(){
    let mut entry=time_eater_entry(vec![card_state::permanent(CardId::Metallicize,CardInstanceId(1),0),
        card_state::permanent(CardId::Combust,CardInstanceId(2),0)],10);
    entry.setup.enemies[0].powers.push(full_power("metallicize",3));
    let mut c=Combat::enter(entry).unwrap();
    assert!(engine::validate_runtime(&c.state).is_ok());
    move_draw_to_hand(&mut c,1);move_draw_to_hand(&mut c,2);
    assert!(engine::validate_runtime(&c.state).is_ok());
    c.step_full(&CombatAction::PlayCard{instance:1,target:None}).unwrap();
    assert_eq!(amount(&c,"metallicize"),3);
    assert_eq!(enemy_amount(&c,0,"time_warp"),11);
    assert!(engine::validate_runtime(&c.state).is_ok());
    let player_hp=c.state.player.hp;let enemy_hp=c.state.enemies[0].hp;
    let observed=c.step_full_observed(&CombatAction::PlayCard{instance:2,target:None});
    assert!(observed.result.is_ok());
    assert_eq!(c.state.turn,2,"the real eleventh-to-twelfth card barrier forced the ordinary end-turn route");
    assert_eq!(enemy_amount(&c,0,"time_warp"),0);
    assert_eq!(c.state.hook_events.iter().filter(|event|event.hook==Hook::TurnEnded).count(),1);
    assert_eq!((c.state.player.hp,c.state.enemies[0].hp),(player_hp-1,enemy_hp-5));
    let hp_loss=observed.effects.iter().position(|effect|matches!(effect.kind,
        ResolvedEffectKind::Damage{category:DamageCategory::HpLoss,..})).unwrap();
    let enemy_thorns=observed.effects.iter().position(|effect|effect.target==Some(Actor::Enemy(c.state.enemies[0].instance_id.clone()))&&matches!(effect.kind,
        ResolvedEffectKind::Damage{category:DamageCategory::Thorns,incoming:5,..})).unwrap();
    assert!(hp_loss<enemy_thorns,"player HP_LOSS is committed before null-owner enemy THORNS");
    assert_eq!(observed.effects.iter().filter(|effect|effect.target==Some(Actor::Player)&&matches!(effect.kind,
        ResolvedEffectKind::Damage{category:DamageCategory::HpLoss,incoming:1,..})).count(),1);
    assert_eq!(observed.effects.iter().filter(|effect|effect.source==Source::None&&matches!(effect.kind,
        ResolvedEffectKind::Damage{category:DamageCategory::Thorns,incoming:5,..})).count(),1);
    assert_eq!(observed.effects.iter().filter(|effect|effect.target==Some(Actor::Player)&&matches!(effect.kind,
        ResolvedEffectKind::Block{after:3,..})).count(),1,"player Metallicize runs once at its pre-end-card checkpoint");
    assert_eq!(c.state.enemies[0].block,23,"enemy Metallicize remains at its independently qualified round-end checkpoint after Time Ripple Block20");
    assert!(engine::validate_runtime(&c.state).is_ok());
}

#[test]
fn m10_power_turns_combust_lethal_hploss_does_not_precredit_enemy_damage(){
    let entry=time_eater_entry(vec![card_state::permanent(CardId::Combust,CardInstanceId(1),0)],11);
    let mut c=Combat::enter(entry).unwrap();assert!(engine::validate_runtime(&c.state).is_ok());
    move_draw_to_hand(&mut c,1);c.state.player.hp=1;
    assert!(engine::validate_runtime(&c.state).is_ok());
    let enemy_hp=c.state.enemies[0].hp;
    let observed=c.step_full_observed(&CombatAction::PlayCard{instance:1,target:None});
    assert!(observed.result.is_ok());
    assert_eq!(c.state.outcome,Some(Outcome::Death));
    assert_eq!(c.state.enemies[0].hp,enemy_hp,"later Combust THORNS is not pre-credited or run after Death");
    assert!(observed.effects.iter().any(|effect|matches!(effect.kind,ResolvedEffectKind::Damage{category:DamageCategory::HpLoss,..})));
    assert!(!observed.effects.iter().any(|effect|effect.target==Some(Actor::Enemy(c.state.enemies[0].instance_id.clone()))&&matches!(effect.kind,ResolvedEffectKind::Damage{category:DamageCategory::Thorns,..})));
    assert_eq!(c.state.hook_events.iter().filter(|event|event.hook==Hook::TurnEnded).count(),0);
    assert!(engine::validate_runtime(&c.state).is_ok());
}
