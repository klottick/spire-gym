use super::super::*;
use super::full_runtime::entered_combat;
use crate::{combat::model::{OwnedCard,PlayContext},content::{CardId,Catalog,CardRarity,CardType}};

fn combat_with(id:CardId,upgraded:bool)->Combat {
    entered_combat(3,3,|card|*card=card_state::permanent(id,card.instance,u32::from(upgraded)))
}
fn armaments_sentinel_combat()->Combat{
    let mut combat=entered_combat(3,3,|_|{});
    for owned in &mut combat.state.piles.hand {
        if owned.handle().as_str()=="1"{*owned=OwnedCard::Full(card_state::permanent(CardId::Armaments,CardInstanceId(1),0));}
        if owned.handle().as_str()=="2"{*owned=OwnedCard::Full(card_state::permanent(CardId::Sentinel,CardInstanceId(2),0));}
    }
    combat
}
fn instance(c:&Combat,id:CardId)->u64 {
    c.state.piles.all(c.state.in_play.as_ref()).filter_map(OwnedCard::full)
        .find(|card|card.id==id).unwrap().instance.0
}
fn play(c:&mut Combat,id:CardId,target:Option<TargetId>){
    c.step_full(&CombatAction::PlayCard{instance:instance(c,id),target}).unwrap();
}
fn power(c:&mut Combat,id:&'static str,amount:i32){
    c.state.player_full_powers.push(FullPowerEntry{id:FullPowerId(id),state:PowerState{amount,skip_first:false,monster_source_grace:false}});
}

#[test]
fn m10_exhaust_cards_have_real_catalog_classification(){
    let catalog=Catalog::standard();
    for (id,kind,rarity) in [(CardId::Corruption,CardType::Power,CardRarity::Rare),
        (CardId::DarkEmbrace,CardType::Power,CardRarity::Uncommon),
        (CardId::FeelNoPain,CardType::Power,CardRarity::Uncommon),
        (CardId::Sentinel,CardType::Skill,CardRarity::Uncommon)] {
        let def=catalog.card(id).unwrap(); assert_eq!(def.card_type,kind); assert_eq!(def.rarity,rarity);
        assert!(!def.definition_only);
    }
}

#[test]
fn m10_exhaust_cards_publish_base_and_upgrade_costs_without_intrinsic_exhaust(){
    for (index,(id,base,upgraded)) in [(CardId::Corruption,3,2),(CardId::DarkEmbrace,2,1),
        (CardId::FeelNoPain,1,1),(CardId::Sentinel,1,1)].into_iter().enumerate(){
        let c=combat_with(id,false);let live=c.state.piles.hand.iter().filter_map(OwnedCard::full).find(|x|x.id==id).unwrap();
        assert_eq!(cards::visible_cost(&c.state,live),Some(base));assert!(!live.local.exhaust_on_finish);
        let c=combat_with(id,true);let live=c.state.piles.hand.iter().filter_map(OwnedCard::full).find(|x|x.id==id).unwrap();
        assert_eq!(cards::visible_cost(&c.state,live),Some(upgraded));assert!(!live.local.exhaust_on_finish);
        let generated=card_state::generated(id,CardInstanceId(900+index as u64),1);
        assert!(generated.is_some(),"generated constructor must represent {id:?}");
        let generated=generated.unwrap();assert!(!generated.local.exhaust_on_finish);
        assert_eq!(cards::ironclad::rules::public_values(&generated).unwrap().exhaust_on_finish,false);
        assert!(cards::ironclad::rules::public_values(live).is_some(),"public values missing for {id:?}");
        let unupgraded=card_state::generated(id,CardInstanceId(950+index as u64),0).unwrap();
        assert_eq!(cards::visible_cost(&c.state,&unupgraded),Some(base),"generated base cost {id:?}");
        assert_eq!(cards::visible_cost(&c.state,&generated),Some(upgraded),"generated upgraded cost {id:?}");
    }
}

#[test]
fn m10_exhaust_public_values_publish_only_source_qualified_magic_and_effects(){
    for (id,base_magic,up_magic) in [(CardId::Corruption,None,None),(CardId::DarkEmbrace,None,None),
        (CardId::FeelNoPain,Some(3),Some(4)),(CardId::Sentinel,Some(2),Some(3))]{
        for upgraded in [false,true]{
            let card=card_state::permanent(id,CardInstanceId(800+u64::from(upgraded)),u32::from(upgraded));
            let values=cards::ironclad::rules::public_values(&card).unwrap();
            assert_eq!(values.magic_number,if upgraded{up_magic}else{base_magic},"{id:?} upgraded={upgraded}");
            assert!(!values.exhaust_on_finish,"printed exhaust-trigger behavior is not intrinsic disposition");
        }
    }
    let sentinel=card_state::permanent(CardId::Sentinel,CardInstanceId(899),1);
    assert_eq!(cards::ironclad::rules::public_values(&sentinel).unwrap().base_block,Some(8));
    assert_eq!(cards::ironclad::rules::public_values(&card_state::permanent(CardId::Sentinel,CardInstanceId(898),0)).unwrap().base_block,Some(5));
}

#[test]
fn m10_exhaust_generated_permanent_and_stat_copies_retain_costs_and_played_power_identity(){
    let combat=combat_with(CardId::DarkEmbrace,true);
    let source=card_state::generated(CardId::DarkEmbrace,CardInstanceId(901),1).unwrap();
    assert_eq!(cards::visible_cost(&combat.state,&source),Some(1));
    assert!(!source.local.exhaust_on_finish);
    let stat=source.stat_equivalent_copy(CardInstanceId(902));
    assert_eq!(stat.local.exhaust_on_finish,false);
    assert_eq!(stat.upgrades,1);
    let clone=source.copy_as(CardInstanceId(903));
    assert_eq!(clone.local,source.local,"copy_as keeps adapter-compatible local state");
    let permanent=card_state::permanent(CardId::Sentinel,CardInstanceId(904),0);
    assert_eq!(card_state::generated(CardId::Sentinel,CardInstanceId(905),0).unwrap().local.exhaust_on_finish,false);
    assert_eq!(permanent.local.exhaust_on_finish,false);
}

#[test]
fn m10_exhaust_stat_equivalent_and_adapter_clones_play_as_one_owned_power(){
    for (instance,copy) in [(71,card_state::permanent(CardId::FeelNoPain,CardInstanceId(71),0)
        .stat_equivalent_copy(CardInstanceId(71))),
        (72,card_state::permanent(CardId::DarkEmbrace,CardInstanceId(72),1).copy_as(CardInstanceId(72)))]{
        let mut c=combat_with(CardId::Strike,false);
        c.state.card_ids_issued.insert(card_state::handle(CardInstanceId(instance)));
        c.state.piles.hand.push(OwnedCard::Full(copy));
        c.step_full(&CombatAction::PlayCard{instance,target:None}).unwrap();
        let id=c.state.played_powers.iter().filter_map(OwnedCard::full).filter(|card|card.instance.0==instance).count();
        assert_eq!(id,1);
        assert_eq!(c.state.player_full_powers.iter().filter(|entry|
            (instance==71&&entry.id.0=="feel_no_pain")||(instance==72&&entry.id.0=="dark_embrace")).count(),1);
        assert!(engine::validate_runtime(&c.state).is_ok());
    }
}

#[test]
fn m10_exhaust_rejects_mismatched_play_context_owner(){
    let mut c=combat_with(CardId::Strike,false);
    let owned=c.state.piles.hand.pop().unwrap();let handle=owned.handle();
    c.state.in_play=Some(owned);c.state.play_context=Some(PlayContext{card:card_state::handle(CardInstanceId(999)),target:None,
        energy_on_use:1,paid:1,disposition:super::super::model::FinishDisposition::Exhaust});
    c.state.phase=Phase::PlayerAction;
    c.state.pending.push_back(Effect{source:Source::Card(handle),target:None,kind:EffectKind::FinishCard});
    assert_eq!(engine::validate_runtime(&c.state),Err(InputError::Invariant("autoplay active owner/context".into())));
}

#[test]
fn m10_exhaust_corruption_makes_skill_free_and_exhausts_after_real_use(){
    let mut c=combat_with(CardId::Corruption,false); play(&mut c,CardId::Corruption,None);
    assert!(c.state.played_powers.iter().any(|x|x.full().is_some_and(|x|x.id==CardId::Corruption)));
    assert!(!c.state.piles.exhaust.iter().any(|x|x.full().is_some_and(|x|x.id==CardId::Corruption)));
    assert_eq!(cards::visible_cost(&c.state,c.state.piles.hand.iter().filter_map(OwnedCard::full).find(|x|x.id==CardId::Defend).unwrap()),Some(0));
    play(&mut c,CardId::Defend,None);
    assert!(c.state.piles.exhaust.iter().any(|x|x.full().is_some_and(|x|x.id==CardId::Defend)));
    assert_eq!(c.state.piles.exhaust.iter().filter(|x|x.full().is_some_and(|x|x.id==CardId::Defend)).count(),1);
    assert!(!c.state.piles.exhaust.iter().find(|owned|owned.full().is_some_and(|card|card.id==CardId::Defend))
        .unwrap().full().unwrap().local.exhaust_on_finish,"Corruption's accepted per-use disposition does not become an intrinsic card flag");
}

#[test]
fn m10_exhaust_corruption_captures_the_accepted_skill_use_without_mutating_intrinsic_flags(){
    let mut c=combat_with(CardId::Corruption,false);play(&mut c,CardId::Corruption,None);
    let skill=instance(&c,CardId::Defend);
    let action=CombatAction::PlayCard{instance:skill,target:None};
    let prepared=cards::prepare(&c,&action).unwrap();
    let handle=card_state::handle(prepared.instance);let source=Source::Card(handle.clone());
    let card=c.state.piles.hand.remove(prepared.hand_index);c.state.in_play=Some(card);
    c.state.player.energy=prepared.remaining_energy;c.state.phase=Phase::PlayerAction;
    c.state.play_context=Some(PlayContext{card:handle,target:prepared.target,energy_on_use:prepared.captured_energy,paid:prepared.paid,
        disposition:super::super::model::FinishDisposition::Exhaust});
    c.state.pending.extend(prepared.effects);c.state.pending.push_back(Effect{source,target:None,kind:EffectKind::FinishCard});
    c.state.pending.push_back(Effect{source:Source::None,target:None,kind:EffectKind::Decision});
    c.state.player_full_powers.retain(|power|power.id.0!="corruption");
    engine::resolve(&mut c.state);
    assert!(c.state.piles.exhaust.iter().any(|x|x.handle()==card_state::handle(CardInstanceId(skill))));
    assert!(!c.state.piles.hand.iter().chain(&c.state.piles.discard).filter_map(OwnedCard::full)
        .any(|x|x.id==CardId::Defend&&x.local.exhaust_on_finish));
}

#[test]
fn m10_exhaust_corruption_does_not_turn_x_payment_or_invalid_negative_cost_into_free_play(){
    let mut c=combat_with(CardId::Corruption,false);play(&mut c,CardId::Corruption,None);
    c.state.player.energy=2;
    let id=instance(&c,CardId::Defend);
    c.state.piles.hand.iter_mut().find(|owned|owned.handle()==card_state::handle(CardInstanceId(id))).unwrap().full_mut().unwrap().local.x_cost=true;
    let prepared=cards::prepare(&c,&CombatAction::PlayCard{instance:id,target:None}).unwrap();
    assert_eq!((prepared.paid,prepared.remaining_energy),(2,0),"adapter X Skill sentinel is explicitly outside sourced Corruption composition; this native boundary preserves ordinary X payment");
    play(&mut c,CardId::Defend,None);assert_eq!(c.state.player.energy,0);

    let mut c=combat_with(CardId::Corruption,false);play(&mut c,CardId::Corruption,None);
    let id=instance(&c,CardId::Defend);
    c.state.piles.hand.iter_mut().find(|owned|owned.handle()==card_state::handle(CardInstanceId(id))).unwrap().full_mut().unwrap().local.temporary_cost=-2;
    let action=CombatAction::PlayCard{instance:id,target:None};
    assert!(cards::prepare(&c,&action).is_err());
}

#[test]
fn m10_exhaust_dark_embrace_draws_once_for_each_actual_exhaust(){
    let mut c=combat_with(CardId::DarkEmbrace,false);play(&mut c,CardId::DarkEmbrace,None);
    assert!(c.state.played_powers.iter().any(|owned|owned.full().is_some_and(|card|card.id==CardId::DarkEmbrace)));
    assert_eq!(c.state.player_full_powers.iter().find(|entry|entry.id.0=="dark_embrace").unwrap().state.amount,1);
    let draw=card_state::permanent(CardId::Strike,CardInstanceId(100),0);
    c.state.card_ids_issued.insert(card_state::handle(draw.instance)); c.state.piles.draw.push(OwnedCard::Full(draw));
    let before=c.state.piles.hand.len();
    let card=c.state.piles.hand.pop().unwrap(); piles::exhaust_owned_card(&mut c.state,card).unwrap(); engine::resolve(&mut c.state);
    assert_eq!(c.state.piles.hand.len(),before);
    assert_eq!(c.state.hook_events.iter().filter(|e|e.hook==Hook::CardExhausted).count(),1);

    let mut upgraded=combat_with(CardId::DarkEmbrace,true);play(&mut upgraded,CardId::DarkEmbrace,None);
    assert_eq!(upgraded.state.player_full_powers.iter().find(|entry|entry.id.0=="dark_embrace").unwrap().state.amount,1,
        "Dark Embrace upgrade lowers cost only; its source-qualified draw payload remains one");
}

#[test]
fn m10_exhaust_dark_embrace_draw_uses_real_discard_refill_and_recycle(){
    let mut c=combat_with(CardId::DarkEmbrace,false);play(&mut c,CardId::DarkEmbrace,None);
    let held=c.state.piles.hand.pop().unwrap();let source=held.handle();
    let recycle=c.state.piles.hand.pop().unwrap();let recycle_id=recycle.handle();
    c.state.piles.discard.push(recycle);
    assert!(c.state.piles.draw.is_empty());
    piles::exhaust_owned_card(&mut c.state,held).unwrap();
    c.state.phase=Phase::PlayerAction;
    engine::resolve(&mut c.state);
    assert!(c.state.piles.hand.iter().any(|owned|owned.handle()==recycle_id),"real Draw refills from discard after exhaustion");
    assert!(c.state.piles.exhaust.iter().any(|owned|owned.handle()==source));
    assert_eq!(c.state.hook_events.iter().filter(|event|event.hook==Hook::CardExhausted&&event.source==Source::Card(source.clone())).count(),1);
    assert!(engine::validate_runtime(&c.state).is_ok());
}

#[test]
fn m10_exhaust_dark_embrace_respects_no_draw_and_hand_cap(){
    for mode in [0,1,2]{ // No Draw, hand already full, and nine-to-ten control.
        let mut c=combat_with(CardId::DarkEmbrace,false);
        play(&mut c,CardId::DarkEmbrace,None);
        let exhaust=card_state::permanent(CardId::Strike,CardInstanceId(100),0);
        let marker=card_state::permanent(CardId::Defend,CardInstanceId(101),0);
        for card in [&exhaust,&marker]{c.state.card_ids_issued.insert(card_state::handle(card.instance));}
        c.state.piles.discard.push(OwnedCard::Full(exhaust));c.state.piles.draw.push(OwnedCard::Full(marker));
        if mode==0{power(&mut c,"no_draw",1)}else{let target=if mode==1{10}else{9};while c.state.piles.hand.len()<target{
            let id=200+c.state.piles.hand.len() as u64;
            let filler=card_state::permanent(CardId::Strike,CardInstanceId(id),0);
            c.state.card_ids_issued.insert(card_state::handle(filler.instance));c.state.piles.hand.push(OwnedCard::Full(filler));
        }}
        let before=c.state.piles.hand.len();let held=c.state.piles.discard.pop().unwrap();
        piles::exhaust_owned_card(&mut c.state,held).unwrap();engine::resolve(&mut c.state);
        assert!(c.state.piles.hand.len()<=10);
        if mode==0||mode==1{assert_eq!(c.state.piles.hand.len(),before);assert_eq!(c.state.piles.draw.first().map(OwnedCard::handle),Some(card_state::handle(CardInstanceId(101))))}
        else{assert_eq!(c.state.piles.hand.len(),10);assert_eq!(c.state.piles.hand.last().unwrap().handle(),card_state::handle(CardInstanceId(101)))}
    }
}

#[test]
fn m10_exhaust_turn_end_status_and_ethereal_paths_share_the_real_callback(){
    for id in [CardId::Dazed,CardId::Defend]{
        let mut c=combat_with(CardId::Sentinel,false);power(&mut c,"feel_no_pain",3);
        let mut live=if id==CardId::Dazed{card_state::generated(id,CardInstanceId(100),0).unwrap()}
            else{card_state::permanent(id,CardInstanceId(100),0)};
        if id==CardId::Dazed{assert_eq!(live.origin,crate::combat::CombatCardRef::CombatLocal(100));}
        if id==CardId::Defend{live.local.ethereal=true;}
        c.state.card_ids_issued.insert(card_state::handle(live.instance));let owner=card_state::handle(live.instance);
        c.state.piles.hand.push(OwnedCard::Full(live));
        let before=c.state.player.block;
        piles::discard_card(&mut c.state,&owner,true).unwrap();engine::resolve(&mut c.state);
        assert!(c.state.piles.exhaust.iter().any(|owned|owned.handle()==owner));
        assert_eq!(c.state.player.block,before+3);
        assert_eq!(c.state.hook_events.iter().filter(|event|event.hook==Hook::CardExhausted&&event.source==Source::Card(owner.clone())).count(),1);
    }
}

#[test]
fn m10_exhaust_dispatch_order_follows_player_powers_then_card_trigger(){
    for dark_first in [true,false]{
        let mut c=combat_with(CardId::Sentinel,false);play(&mut c,CardId::Sentinel,None);
        if dark_first{power(&mut c,"dark_embrace",1);power(&mut c,"feel_no_pain",3)}
        else{power(&mut c,"feel_no_pain",3);power(&mut c,"dark_embrace",1)}
        let index=c.state.piles.discard.iter().position(|owned|owned.full().is_some_and(|card|card.id==CardId::Sentinel)).unwrap();
        let held=c.state.piles.discard.remove(index);let source=Source::Card(held.handle());
        piles::exhaust_owned_card(&mut c.state,held).unwrap();c.state.phase=Phase::PlayerAction;
        engine::dispatch_one(&mut c.state);
        let expected=if dark_first{vec![EffectKind::Draw(1),EffectKind::GainBlock(3),EffectKind::GainEnergy(2)]}
            else{vec![EffectKind::GainBlock(3),EffectKind::Draw(1),EffectKind::GainEnergy(2)]};
        assert_eq!(c.state.pending.iter().take(3).map(|effect|effect.kind.clone()).collect::<Vec<_>>(),expected);
        assert!(c.state.pending.iter().take(3).all(|effect|effect.source==source));
        engine::validate_runtime(&c.state).expect("ordered exhaust hook checkpoint must be a valid runtime boundary");
    }
}

#[test]
fn m10_exhaust_callback_fault_retains_moved_source_child_parent_and_first_fault(){
    let mut c=combat_with(CardId::Sentinel,false);power(&mut c,"feel_no_pain",3);play(&mut c,CardId::Sentinel,None);
    let parent=c.state.piles.hand.pop().unwrap();let parent_handle=parent.handle();
    c.state.in_play=Some(parent);c.state.play_context=Some(PlayContext{card:parent_handle.clone(),target:None,
        energy_on_use:0,paid:0,disposition:super::super::model::FinishDisposition::Natural});
    let sentinel_index=c.state.piles.discard.iter().position(|owned|owned.full().is_some_and(|card|card.id==CardId::Sentinel)).unwrap();
    let exhausted=c.state.piles.discard.remove(sentinel_index);let exhausted_handle=exhausted.handle();
    piles::exhaust_owned_card(&mut c.state,exhausted).unwrap();
    c.state.phase=Phase::PlayerAction;
    c.state.pending.push_back(Effect{source:Source::Card(parent_handle.clone()),target:None,kind:EffectKind::FinishCard});
    let block_before=c.state.player.block;let energy_before=c.state.player.energy;
    c.state.resolved_effects=99_997; // Hook and Draw child execute; next owned callback child meets the existing guard.
    engine::validate_runtime(&c.state).expect("retained exhaust callback checkpoint must be valid before guard injection");
    engine::dispatch_one(&mut c.state);
    assert!(c.state.current.is_none());
    assert_eq!(c.state.pending.front().unwrap().kind,EffectKind::GainBlock(3));
    engine::dispatch_one(&mut c.state);
    assert_eq!(c.state.player.block,block_before+3,"first callback child effect is retained as earned work");
    assert_eq!(c.state.pending.front().unwrap().kind,EffectKind::GainEnergy(2));
    engine::dispatch_one(&mut c.state);
    assert_eq!(c.state.fault.as_ref().unwrap().kind,FaultKind::EffectGuard);
    assert_eq!(c.state.current.as_ref().unwrap().kind,EffectKind::GainEnergy(2));
    assert_eq!(c.state.player.block,block_before+3,"prior callback child remains committed across the later fault");
    assert_eq!(c.state.player.energy,energy_before,"faulting next callback child is not partially applied");
    assert!(c.state.piles.exhaust.iter().any(|owned|owned.handle()==exhausted_handle));
    assert_eq!(c.state.hook_events.iter().filter(|event|event.hook==Hook::CardExhausted&&event.source==Source::Card(exhausted_handle.clone())).count(),1);
    assert_eq!(c.state.in_play.as_ref().unwrap().handle(),parent_handle);
    assert!(c.state.pending.iter().any(|effect|effect.kind==EffectKind::FinishCard));
    let fault=c.state.fault.clone();let block=c.state.player.block;let pending=c.state.pending.clone();
    let current=c.state.current.clone();let count=c.state.resolved_effects;
    engine::dispatch_one(&mut c.state);
    assert_eq!(c.state.fault,fault);assert_eq!(c.state.player.block,block);assert_eq!(c.state.pending,pending);
    assert_eq!(c.state.current,current);assert_eq!(c.state.resolved_effects,count);
}

#[test]
fn m10_exhaust_true_grit_selection_triggers_only_for_chosen_instance(){
    let mut c=combat_with(CardId::TrueGrit,true);power(&mut c,"feel_no_pain",3);
    c.step_full(&CombatAction::PlayCard{instance:instance(&c,CardId::TrueGrit),target:None}).unwrap();
    let selection=c.state.pending_selection.as_ref().unwrap();let chosen=selection.candidates[0].clone();
    let before=c.state.player.block;
    c.step_full(&CombatAction::SelectCard{instance:chosen.as_str().parse().unwrap()}).unwrap();
    assert_eq!(c.state.player.block,before+3);
    assert!(c.state.piles.exhaust.iter().any(|owned|owned.handle()==chosen));
    assert_eq!(c.state.hook_events.iter().filter(|event|event.hook==Hook::CardExhausted&&event.source==Source::Card(chosen.clone())).count(),1);
}

#[test]
fn m10_exhaust_feel_no_pain_gains_block_for_actual_exhaust(){
    let mut c=combat_with(CardId::FeelNoPain,false);play(&mut c,CardId::FeelNoPain,None);
    assert!(c.state.played_powers.iter().any(|owned|owned.full().is_some_and(|card|card.id==CardId::FeelNoPain)));
    assert_eq!(c.state.player_full_powers.iter().find(|entry|entry.id.0=="feel_no_pain").unwrap().state.amount,3);
    let before=c.state.player.block;
    let card=c.state.piles.hand.pop().unwrap(); piles::exhaust_owned_card(&mut c.state,card).unwrap(); engine::resolve(&mut c.state);
    assert_eq!(c.state.player.block,before+3);
}

#[test]
fn m10_exhaust_sentinel_blocks_when_played_and_grants_energy_when_exhausted(){
    let mut c=combat_with(CardId::Sentinel,false); let energy=c.state.player.energy;
    play(&mut c,CardId::Sentinel,None); assert_eq!(c.state.player.block,5); assert_eq!(c.state.player.energy,energy-1);
    let sentinel=c.state.piles.discard.iter().position(|x|x.full().is_some_and(|x|x.id==CardId::Sentinel)).unwrap();
    let card=c.state.piles.discard.remove(sentinel); piles::exhaust_owned_card(&mut c.state,card).unwrap(); engine::resolve(&mut c.state);
    assert_eq!(c.state.player.energy,energy+1);
}

#[test]
fn m10_exhaust_armaments_upgrade_changes_sentinel_exhaust_energy(){
    let mut c=armaments_sentinel_combat();
    let sentinel=instance(&c,CardId::Sentinel);
    play(&mut c,CardId::Armaments,None);
    c.step_full(&CombatAction::SelectCard{instance:sentinel}).unwrap();
    assert_eq!(c.state.piles.hand.iter().filter_map(OwnedCard::full).find(|card|card.instance.0==sentinel).unwrap().upgrades,1);
    play(&mut c,CardId::Sentinel,None);
    let index=c.state.piles.discard.iter().position(|owned|owned.full().is_some_and(|card|card.instance.0==sentinel)).unwrap();
    let card=c.state.piles.discard.remove(index);let energy=c.state.player.energy;
    piles::exhaust_owned_card(&mut c.state,card).unwrap();engine::resolve(&mut c.state);
    assert_eq!(c.state.player.energy,energy+3);
}

#[test]
fn m10_exhaust_upgraded_printed_values_are_preserved(){
    let mut c=combat_with(CardId::Corruption,true);
    assert_eq!(cards::visible_cost(&c.state,c.state.piles.hand.iter().filter_map(OwnedCard::full).find(|x|x.id==CardId::Corruption).unwrap()),Some(2));
    play(&mut c,CardId::Corruption,None);
    let mut c=combat_with(CardId::FeelNoPain,true);play(&mut c,CardId::FeelNoPain,None);
    assert_eq!(c.state.player_full_powers.iter().find(|power|power.id.0=="feel_no_pain").unwrap().state.amount,4);
    let before=c.state.player.block;let held=c.state.piles.hand.pop().unwrap();piles::exhaust_owned_card(&mut c.state,held).unwrap();engine::resolve(&mut c.state);
    assert_eq!(c.state.player.block,before+4);
    let mut c=combat_with(CardId::Sentinel,true);let energy=c.state.player.energy;
    play(&mut c,CardId::Sentinel,None);assert_eq!(c.state.player.block,8);
    let index=c.state.piles.discard.iter().position(|o|o.full().is_some_and(|card|card.id==CardId::Sentinel)).unwrap();
    let held=c.state.piles.discard.remove(index);piles::exhaust_owned_card(&mut c.state,held).unwrap();engine::resolve(&mut c.state);
    assert_eq!(c.state.player.energy,energy+2);
}
