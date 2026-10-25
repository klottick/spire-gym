use super::super::*;
use super::super::model::OwnedCard;

fn with_card(id:crate::content::CardId,upgraded:bool)->Combat{
    super::full_runtime::entered_combat(3,3,|card|*card=card_state::permanent(id,card.instance,u32::from(upgraded)))
}

fn add_draw(c:&mut Combat,count:u64){
    for offset in 0..count{
        let card=card_state::permanent(crate::content::CardId::Strike,CardInstanceId(90+offset),0);
        let handle=card_state::handle(card.instance);
        c.state.card_ids_issued.insert(handle);
        c.state.piles.draw.push(OwnedCard::Full(card));
    }
}

#[test]
fn m10_selection_card_public_values_and_finish_flags_cover_both_upgrade_states(){
    use crate::content::CardId::*;
    assert_eq!(crate::content::Catalog::standard().card(Warcry).unwrap().rarity,crate::content::CardRarity::Common);
    for (id,cost,damage,block,magic,exhaust) in [
        (TrueGrit,1,None,Some(7),Some(1),false),(Headbutt,1,Some(9),None,None,false),
        (Warcry,0,None,None,Some(1),true),(BattleTrance,0,None,None,Some(3),false),(Exhume,1,None,None,None,true)]{
        let c=with_card(id,false);let card=c.public_full().unwrap().hand.into_iter().find(|card|card.instance.0==3).unwrap();
        assert_eq!(card.effective_cost,Some(cost),"{id:?}");
        let values=card.values.unwrap();
        assert_eq!((values.base_damage,values.base_block,values.magic_number,values.exhaust_on_finish),(damage,block,magic,exhaust),"{id:?}");
        let up_cost=if id==TrueGrit||id==Headbutt{1}else if id==Exhume{0}else{0};
        let c=with_card(id,true);let card=c.public_full().unwrap().hand.into_iter().find(|card|card.instance.0==3).unwrap();
        assert_eq!(card.effective_cost,Some(up_cost),"upgraded {id:?}");
        let values=card.values.unwrap();
        match id{TrueGrit=>assert_eq!(values.base_block,Some(9)),Headbutt=>assert_eq!(values.base_damage,Some(12)),
            Warcry=>assert_eq!(values.magic_number,Some(2)),BattleTrance=>assert_eq!(values.magic_number,Some(4)),
            Exhume=>assert_eq!(values.magic_number,None),_=>unreachable!()}
    }
}

#[test]
fn m10_true_grit_upgraded_offers_owned_hand_card_and_exhausts_only_confirmed_choice(){
    let mut c=with_card(crate::content::CardId::TrueGrit,true);
    let before=c.state.piles.hand.iter().map(OwnedCard::handle).collect::<Vec<_>>();
    c.step_full(&CombatAction::PlayCard{instance:3,target:None}).unwrap();
    assert_eq!(c.state.player.block,9);
    let selection=c.state.pending_selection.as_ref().unwrap();
    assert_eq!(selection.source_zone,SelectionSourceZone::Hand);
    assert_eq!(selection.maximum,1);
    assert!(selection.candidates.iter().all(|id|before.contains(id)&&id.as_str()!="3"));
    let chosen=selection.candidates[0].as_str().parse().unwrap();
    c.step_full(&CombatAction::SelectCard{instance:chosen}).unwrap();
    assert!(c.state.piles.exhaust.iter().any(|card|card.handle().as_str()==chosen.to_string()));
    assert!(c.state.piles.hand.iter().all(|card|card.handle().as_str()!=chosen.to_string()));
}

#[test]
fn m10_true_grit_choice_reads_hand_after_preceding_block_work(){
    let mut c=with_card(crate::content::CardId::TrueGrit,true);
    let initial_discard=c.state.piles.hand.iter().position(|card|card.handle().as_str()!="3").unwrap();
    c.state.piles.discard.push(c.state.piles.hand.remove(initial_discard));
    let action=CombatAction::PlayCard{instance:3,target:None};
    let prepared=super::super::cards::prepare(&c,&action).unwrap();
    let parent=super::super::card_state::handle(prepared.instance);
    let source=Source::Card(parent.clone());
    let parent_card=c.state.piles.hand.remove(prepared.hand_index);
    c.state.in_play=Some(parent_card);c.state.phase=Phase::PlayerAction;
    let block=prepared.effects[0].clone();
    super::super::effects::apply_effect(&mut c.state,&block).unwrap();
    let old=c.state.piles.hand.remove(0);let old_id=old.handle();
    c.state.piles.exhaust.push(old);
    let replacement=c.state.piles.discard.pop().unwrap();
    let new_id=replacement.handle();
    c.state.piles.hand.push(replacement);
    let select=prepared.effects.iter().find(|effect|matches!(effect.kind,EffectKind::Ironclad(IroncladEffect::ChooseTrueGritCard))).unwrap().clone();
    assert_eq!(select.source,source);
    super::super::effects::apply_effect(&mut c.state,&select).unwrap();
    let eligible=c.state.piles.hand.iter().map(OwnedCard::handle).filter(|id|*id!=parent).collect::<Vec<_>>();
    assert_ne!(old_id,new_id);
    assert_eq!(c.state.pending_selection.as_ref().unwrap().candidates,eligible,
        "the queued choice must use the hand that exists after preceding work");
}

#[test]
fn m10_upgraded_true_grit_empty_hand_skips_choice_after_block(){
    let mut c=with_card(crate::content::CardId::TrueGrit,true);
    c.state.piles.hand.retain(|card|card.handle().as_str()=="3");
    c.step_full(&CombatAction::PlayCard{instance:3,target:None}).unwrap();
    assert_eq!(c.state.player.block,9);
    assert!(c.state.pending_selection.is_none());
    assert!(c.state.piles.discard.iter().any(|card|card.handle().as_str()=="3"));
}

#[test]
fn m10_headbutt_choice_reads_discard_after_its_hit(){
    let mut c=with_card(crate::content::CardId::Headbutt,false);
    add_draw(&mut c,2);
    let initial=c.state.piles.hand.iter().position(|card|card.handle().as_str()!="3").unwrap();
    c.state.piles.discard.push(c.state.piles.hand.remove(initial));
    let action=CombatAction::PlayCard{instance:3,target:Some(TargetId(1))};
    c.state.enemies[0].hp=80;
    let prepared=super::super::cards::prepare(&c,&action).unwrap();
    let parent=super::super::card_state::handle(prepared.instance);
    let parent_card=c.state.piles.hand.remove(prepared.hand_index);
    c.state.in_play=Some(parent_card);c.state.phase=Phase::PlayerAction;
    super::super::effects::apply_effect(&mut c.state,&prepared.effects[0]).unwrap();
    assert!(!super::super::enemies::act2::combat_victory_ready(&c.state),"test must exercise a nonterminal hit");
    assert_eq!(c.state.outcome,None,"test must not have a queued terminal outcome");
    let old=c.state.piles.discard.remove(0);let old_id=old.handle();c.state.piles.exhaust.push(old);
    let replacement=c.state.piles.hand.remove(0);let new_id=replacement.handle();c.state.piles.discard.push(replacement);
    let second=c.state.piles.draw.pop().unwrap();c.state.piles.discard.push(second);
    let choose=prepared.effects.iter().find(|effect|matches!(effect.kind,EffectKind::Ironclad(IroncladEffect::ChooseHeadbuttCard)|EffectKind::SelectCards{..})).unwrap().clone();
    super::super::effects::apply_effect(&mut c.state,&choose).unwrap();
    assert_ne!(old_id,new_id);
    assert_eq!(c.state.pending_selection.as_ref().unwrap().candidates,
        c.state.piles.discard.iter().map(OwnedCard::handle).collect::<Vec<_>>());
    assert_eq!(c.state.pending_selection.as_ref().unwrap().parent,SelectionParent::Card(parent));
}

#[test]
fn m10_exhume_choice_reads_exhaust_after_its_queued_action(){
    let mut c=with_card(crate::content::CardId::Exhume,false);
    let old=card_state::permanent(crate::content::CardId::Strike,CardInstanceId(94),0);
    let old_id=card_state::handle(old.instance);c.state.card_ids_issued.insert(old_id.clone());c.state.piles.exhaust.push(OwnedCard::Full(old));
    let action=CombatAction::PlayCard{instance:3,target:None};
    let prepared=super::super::cards::prepare(&c,&action).unwrap();
    let parent=super::super::card_state::handle(prepared.instance);
    let parent_card=c.state.piles.hand.remove(prepared.hand_index);
    c.state.in_play=Some(parent_card);c.state.phase=Phase::PlayerAction;
    let removed=c.state.piles.exhaust.pop().unwrap();c.state.piles.discard.push(removed);
    let current=card_state::permanent(crate::content::CardId::Defend,CardInstanceId(95),0);
    let current_id=card_state::handle(current.instance);c.state.card_ids_issued.insert(current_id.clone());c.state.piles.exhaust.push(OwnedCard::Full(current));
    let other=card_state::permanent(crate::content::CardId::Strike,CardInstanceId(96),0);
    let other_id=card_state::handle(other.instance);c.state.card_ids_issued.insert(other_id);c.state.piles.exhaust.push(OwnedCard::Full(other));
    let choose=prepared.effects.iter().find(|effect|matches!(effect.kind,EffectKind::Ironclad(IroncladEffect::ChooseExhumeCard)|EffectKind::SelectCards{..})).unwrap().clone();
    super::super::effects::apply_effect(&mut c.state,&choose).unwrap();
    assert_ne!(old_id,current_id);
    assert_eq!(c.state.pending_selection.as_ref().unwrap().candidates,vec![current_id,card_state::handle(CardInstanceId(96))]);
    assert_eq!(c.state.pending_selection.as_ref().unwrap().parent,SelectionParent::Card(parent));
}

#[test]
fn m10_exhume_does_not_offer_return_when_hand_is_full_at_choice_dispatch(){
    let mut c=with_card(crate::content::CardId::Exhume,false);
    let returning=card_state::permanent(crate::content::CardId::Strike,CardInstanceId(97),0);
    let returning_id=card_state::handle(returning.instance);c.state.card_ids_issued.insert(returning_id.clone());c.state.piles.exhaust.push(OwnedCard::Full(returning));
    let action=CombatAction::PlayCard{instance:3,target:None};
    let prepared=super::super::cards::prepare(&c,&action).unwrap();
    let parent=super::super::card_state::handle(prepared.instance);
    c.state.in_play=Some(c.state.piles.hand.remove(prepared.hand_index));c.state.phase=Phase::PlayerAction;
    while c.state.piles.hand.len()<10{
        let card=card_state::permanent(crate::content::CardId::Defend,CardInstanceId(200+c.state.piles.hand.len() as u64),0);
        let id=card_state::handle(card.instance);c.state.card_ids_issued.insert(id);c.state.piles.hand.push(OwnedCard::Full(card));
    }
    let choose=prepared.effects.iter().find(|effect|matches!(effect.kind,EffectKind::Ironclad(IroncladEffect::ChooseExhumeCard))).unwrap().clone();
    assert_eq!(choose.source,Source::Card(parent));
    super::super::effects::apply_effect(&mut c.state,&choose).unwrap();
    assert!(c.state.pending_selection.is_none());
    assert!(c.state.piles.exhaust.iter().any(|card|card.handle()==returning_id));
}

#[test]
fn m10_true_grit_base_randomly_exhausts_one_other_hand_instance_with_scoped_rng(){
    let mut c=with_card(crate::content::CardId::TrueGrit,false);
    let eligible=c.state.piles.hand.iter().filter(|card|card.handle().as_str()!="3").count();
    let before=c.state.card_rng.clone();
    c.step_full(&CombatAction::PlayCard{instance:3,target:None}).unwrap();
    let exhausted=c.state.piles.exhaust.iter().filter(|card|card.handle().as_str()!="3").count();
    assert_eq!(exhausted,1);
    assert_eq!(c.state.piles.hand.iter().filter(|card|card.handle().as_str()!="3").count(),eligible-1);
    if eligible<=1{assert_eq!(c.state.card_rng,before,"singleton choice does not advance RNG");}
    else{assert_ne!(c.state.card_rng,before,"multi-choice uses only the existing scoped card RNG");}
}

#[test]
fn m10_true_grit_single_eligible_card_uses_no_card_rng_word(){
    let mut c=with_card(crate::content::CardId::TrueGrit,false);
    let selected=c.state.piles.hand.iter().find(|card|card.handle().as_str()!="3").unwrap().clone();
    c.state.piles.hand.retain(|card|card.handle().as_str()=="3"||card.handle()==selected.handle());
    let rng=c.state.card_rng.clone();
    c.step_full(&CombatAction::PlayCard{instance:3,target:None}).unwrap();
    assert_eq!(c.state.card_rng,rng,"a singleton random exhaust does not sample");
    assert!(c.state.piles.exhaust.iter().any(|card|card.handle()==selected.handle()));
}

#[test]
fn m10_headbutt_hits_then_returns_exact_discard_instance_to_known_draw_top(){
    let mut c=with_card(crate::content::CardId::Headbutt,false);
    let returned=c.state.piles.hand.iter().find(|owned|owned.handle().as_str()!="3").unwrap().clone();let id=returned.handle();
    let at=c.state.piles.hand.iter().position(|owned|owned.handle()==id).unwrap();c.state.piles.discard.push(c.state.piles.hand.remove(at));
    let hp=c.state.enemies[0].hp;
    c.step_full(&CombatAction::PlayCard{instance:3,target:Some(TargetId(1))}).unwrap();
    assert_eq!(c.state.enemies[0].hp,hp-9);
    assert!(c.state.pending_selection.is_none(),"a singleton retrieval is forced, not a chooser");
    assert_eq!(c.state.piles.draw.last().unwrap().handle(),id);
    assert_eq!(c.state.public.known_draw.first(),Some(&id));
}

#[test]
fn m10_headbutt_with_empty_discard_skips_retrieval(){
    let mut c=with_card(crate::content::CardId::Headbutt,false);c.state.piles.discard.clear();
    c.step_full(&CombatAction::PlayCard{instance:3,target:Some(TargetId(1))}).unwrap();
    assert!(c.state.pending_selection.is_none());
    assert!(c.state.piles.discard.iter().any(|card|card.handle().as_str()=="3"));
}

#[test]
fn m10_headbutt_multiple_discard_choices_preserve_only_entitled_known_prefix(){
    let mut c=with_card(crate::content::CardId::Headbutt,true);
    add_draw(&mut c,2);
    for _ in 0..2{
        let at=c.state.piles.hand.iter().position(|card|card.handle().as_str()!="3").unwrap();
        c.state.piles.discard.push(c.state.piles.hand.remove(at));
    }
    let eligible=c.state.piles.discard.iter().map(OwnedCard::handle).collect::<Vec<_>>();
    let prior_known=c.state.piles.draw.last().unwrap().handle();c.state.public.known_draw=vec![prior_known.clone()];
    c.step_full(&CombatAction::PlayCard{instance:3,target:Some(TargetId(1))}).unwrap();
    let selection=c.state.pending_selection.as_ref().unwrap();
    assert_eq!(selection.candidates,eligible);
    assert_eq!(selection.operation,SelectionOperation::Topdeck);
    let chosen=eligible[0].as_str().parse().unwrap();c.step_full(&CombatAction::SelectCard{instance:chosen}).unwrap();
    assert_eq!(c.state.public.known_draw,vec![eligible[0].clone(),prior_known]);
    assert!(c.state.piles.discard.iter().any(|card|card.handle()==eligible[1]),"unselected discard remains owned in its pile");
    assert!(!c.state.piles.discard.iter().any(|card|card.handle()==eligible[0]));
}

#[test]
fn m10_random_spot_insertion_preserves_entitled_old_top_card(){
    let mut c=with_card(crate::content::CardId::Strike,false);add_draw(&mut c,2);
    let old_top=c.state.piles.draw.last().unwrap().handle();c.state.public.known_draw=vec![old_top.clone()];
    c.enqueue_native_effects(vec![Effect{source:Source::None,target:Some(Actor::Player),
        kind:EffectKind::CreateCard{id:crate::content::CardId::Dazed,destination:CardDestination::ShuffledDraw,upgrades:0}}],None).unwrap();
    assert_eq!(c.state.piles.draw.last().unwrap().handle(),old_top);
    assert_eq!(c.state.public.known_draw,vec![old_top],"insertion below the known top cannot erase that guaranteed position");
}

#[test]
fn m10_public_known_draw_rows_keep_entitled_top_and_bottom_prefixes(){
    let mut c=with_card(crate::content::CardId::Strike,false);add_draw(&mut c,3);
    let top=c.state.piles.draw.last().unwrap().handle();let bottom=c.state.piles.draw.first().unwrap().handle();
    c.state.public.known_draw=vec![top.clone()];c.state.public.known_draw_bottom=vec![bottom.clone()];
    let new_bottom=Effect{source:Source::None,target:Some(Actor::Player),kind:EffectKind::CreateCard{
        id:crate::content::CardId::Dazed,destination:CardDestination::BottomDraw,upgrades:0}};
    c.enqueue_native_effects(vec![new_bottom],None).unwrap();
    let known=c.public_full().unwrap().known_draw;
    assert_eq!(known.len(),3);
    assert_eq!((known[0].card.instance,known[0].from_top,known[0].offset),(CardInstanceId(top.as_str().parse().unwrap()),true,0));
    assert_eq!((known[1].card.instance,known[1].from_top,known[1].offset),(CardInstanceId(c.state.piles.draw[0].handle().as_str().parse().unwrap()),false,0));
    assert_eq!((known[2].card.instance,known[2].from_top,known[2].offset),(CardInstanceId(bottom.as_str().parse().unwrap()),false,1));

    let mut empty=with_card(crate::content::CardId::Strike,false);empty.state.piles.draw.clear();
    empty.enqueue_native_effects(vec![Effect{source:Source::None,target:Some(Actor::Player),kind:EffectKind::CreateCard{
        id:crate::content::CardId::Dazed,destination:CardDestination::BottomDraw,upgrades:0}}],None).unwrap();
    let only=empty.public_full().unwrap().known_draw;
    assert_eq!(only.len(),2,"the sole card is entitled as both top and bottom");
    assert_eq!((only[0].from_top,only[0].offset),(true,0));
    assert_eq!((only[1].from_top,only[1].offset),(false,0));
    assert_eq!(only[0].card.instance,only[1].card.instance);
}

#[test]
fn m10_headbutt_suppresses_selection_when_its_hit_ends_combat(){
    let mut c=with_card(crate::content::CardId::Headbutt,false);
    c.state.enemies[0].hp=9;
    let at=c.state.piles.hand.iter().position(|card|card.handle().as_str()!="3").unwrap();
    let existing=c.state.piles.hand.remove(at);let id=existing.handle();c.state.piles.discard.push(existing);
    c.step_full(&CombatAction::PlayCard{instance:3,target:Some(TargetId(1))}).unwrap();
    assert!(c.state.pending_selection.is_none());
    assert!(c.state.piles.discard.iter().any(|card|card.handle()==id));
    let parent=card_state::handle(CardInstanceId(3));
    assert_eq!(c.state.piles.all(c.state.in_play.as_ref()).filter(|card|card.handle()==parent).count(),1);
}

#[test]
fn m10_warcry_draws_then_required_one_topdeck_auto_completes_and_exhausts_parent(){
    let mut c=with_card(crate::content::CardId::Warcry,false);
    c.step_full(&CombatAction::PlayCard{instance:3,target:None}).unwrap();
    let parent=c.state.in_play.as_ref().unwrap().handle();
    assert_eq!(c.state.pending_selection.as_ref().unwrap().minimum,1);
    assert_eq!(c.state.pending_selection.as_ref().unwrap().operation,SelectionOperation::Topdeck);
    assert!(c.state.piles.hand.len()>0);
    let choice=c.state.pending_selection.as_ref().unwrap().candidates[0].clone();
    assert!(!c.legal_actions_full().unwrap().contains(&CombatAction::ConfirmSelection));
    c.step_full(&CombatAction::SelectCard{instance:choice.as_str().parse().unwrap()}).unwrap();
    assert_ne!(c.state.phase,Phase::Selection,"one required choice auto-completes without ConfirmSelection");
    assert_eq!(c.state.public.known_draw.first(),Some(&choice));
    assert!(c.state.piles.exhaust.iter().any(|card|card.handle()==parent));
}

#[test]
fn m10_warcry_skips_selection_when_draw_and_resulting_hand_are_empty(){
    let mut c=with_card(crate::content::CardId::Warcry,false);
    let rest=std::mem::take(&mut c.state.piles.hand);
    c.state.piles.hand.push(rest.into_iter().find(|card|card.handle().as_str()=="3").unwrap());
    c.state.piles.draw.clear();c.state.piles.discard.clear();
    c.step_full(&CombatAction::PlayCard{instance:3,target:None}).unwrap();
    assert!(c.state.pending_selection.is_none());
    assert!(c.state.piles.exhaust.iter().any(|card|card.handle().as_str()=="3"));
}

#[test]
fn m10_warcry_recycles_discard_before_required_current_hand_choice(){
    let mut c=with_card(crate::content::CardId::Warcry,false);
    let at=c.state.piles.hand.iter().position(|card|card.handle().as_str()!="3").unwrap();
    let recycled=c.state.piles.hand.remove(at);let id=recycled.handle();c.state.piles.discard.push(recycled);
    c.state.piles.draw.clear();
    c.step_full(&CombatAction::PlayCard{instance:3,target:None}).unwrap();
    assert!(c.state.piles.hand.iter().any(|card|card.handle()==id));
    assert!(c.state.pending_selection.as_ref().unwrap().candidates.contains(&id));
    assert_eq!(c.state.pending_selection.as_ref().unwrap().operation,SelectionOperation::Topdeck);
    let choice=id.as_str().parse().unwrap();c.step_full(&CombatAction::SelectCard{instance:choice}).unwrap();
    assert_eq!(c.state.public.known_draw.first(),Some(&id));
}

#[test]
fn m10_battle_trance_draws_before_no_draw_and_suppresses_later_draw_request(){
    let mut c=with_card(crate::content::CardId::BattleTrance,false);
    add_draw(&mut c,4);
    let initial=c.state.piles.hand.len();
    c.step_full(&CombatAction::PlayCard{instance:3,target:None}).unwrap();
    assert_eq!(c.state.piles.hand.len(),initial-1+3);
    assert!(c.state.player_full_powers.iter().any(|p|p.id.0=="no_draw"));
    let hand=c.state.piles.hand.len();
    c.enqueue_native_effects(vec![Effect{source:Source::None,target:Some(Actor::Player),kind:EffectKind::Draw(2)}],None).unwrap();
    assert_eq!(c.state.piles.hand.len(),hand);
}

#[test]
fn m10_battle_trance_respects_preexisting_no_draw(){
    let mut c=with_card(crate::content::CardId::BattleTrance,false);add_draw(&mut c,4);
    c.state.player_full_powers.push(FullPowerEntry{id:FullPowerId("no_draw"),state:PowerState{amount:1,skip_first:false,monster_source_grace:false}});
    let initial=c.state.piles.hand.len();
    c.step_full(&CombatAction::PlayCard{instance:3,target:None}).unwrap();
    assert_eq!(c.state.piles.hand.len(),initial-1,"a pre-existing No Draw suppresses Battle Trance's attempted draw");
    assert!(c.state.player_full_powers.iter().any(|p|p.id.0=="no_draw"&&p.state.amount>0));
}

#[test]
fn m10_battle_trance_can_be_drawn_and_played_again_after_not_exhausting(){
    let mut c=with_card(crate::content::CardId::BattleTrance,false);add_draw(&mut c,4);
    c.step_full(&CombatAction::PlayCard{instance:3,target:None}).unwrap();
    assert!(c.state.piles.discard.iter().any(|card|card.handle().as_str()=="3"));
    assert!(c.state.piles.exhaust.iter().all(|card|card.handle().as_str()!="3"));
    let at=c.state.piles.discard.iter().position(|card|card.handle().as_str()=="3").unwrap();
    c.state.piles.hand.push(c.state.piles.discard.remove(at));
    c.step_full(&CombatAction::PlayCard{instance:3,target:None}).unwrap();
    assert!(c.state.piles.discard.iter().any(|card|card.handle().as_str()=="3"));
    assert!(c.state.piles.exhaust.iter().all(|card|card.handle().as_str()!="3"));
}

#[test]
fn m10_battle_trance_artifact_resists_no_draw_after_the_card_draws(){
    let mut c=with_card(crate::content::CardId::BattleTrance,false);
    add_draw(&mut c,4);
    c.state.player_full_powers.push(FullPowerEntry{id:FullPowerId("artifact"),state:PowerState{amount:1,skip_first:false,monster_source_grace:false}});
    let initial=c.state.piles.hand.len();
    c.step_full(&CombatAction::PlayCard{instance:3,target:None}).unwrap();
    assert_eq!(c.state.piles.hand.len(),initial-1+3);
    assert!(!c.state.player_full_powers.iter().any(|p|p.id.0=="no_draw"));
    assert_eq!(c.state.player_full_powers.iter().find(|p|p.id.0=="artifact").map_or(0,|p|p.state.amount),0);
}

#[test]
fn m10_battle_trance_no_draw_expires_at_player_turn_boundary(){
    let mut c=with_card(crate::content::CardId::BattleTrance,false);add_draw(&mut c,5);
    c.step_full(&CombatAction::PlayCard{instance:3,target:None}).unwrap();
    assert!(c.state.player_full_powers.iter().any(|p|p.id.0=="no_draw"&&p.state.amount>0));
    c.state.pending.push_back(Effect{source:Source::None,target:None,kind:EffectKind::EndPlayerTurn});
    for _ in 0..100{
        if c.state.pending.front().is_some_and(|effect|effect.kind==EffectKind::MonsterStart){break}
        super::super::engine::dispatch_one(&mut c.state);
        assert!(c.state.fault.is_none());
    }
    assert!(c.state.pending.front().is_some_and(|effect|effect.kind==EffectKind::MonsterStart),"bounded end-turn checkpoint reached");
    assert!(!c.state.player_full_powers.iter().any(|p|p.id.0=="no_draw"&&p.state.amount>0));
    for _ in 0..1000{
        if c.state.phase==Phase::Decision&&c.state.pending.is_empty()&&c.state.current.is_none(){break}
        super::super::engine::dispatch_one(&mut c.state);assert!(c.state.fault.is_none());
    }
    assert_eq!(c.state.phase,Phase::Decision,"bounded next-player checkpoint reached");
    let before=c.state.piles.hand.len();
    c.enqueue_native_effects(vec![Effect{source:Source::None,target:Some(Actor::Player),kind:EffectKind::Draw(1)}],None).unwrap();
    assert_eq!(c.state.piles.hand.len(),before+1,"normal draw works on the next player turn");
}

#[test]
fn m10_exhume_returns_existing_non_exhume_instance_without_reconstruction(){
    let mut c=with_card(crate::content::CardId::Exhume,false);
    let returning=c.state.piles.hand.iter().find(|owned|owned.handle().as_str()!="3").unwrap().clone();let id=returning.handle();
    let at=c.state.piles.hand.iter().position(|owned|owned.handle()==id).unwrap();c.state.piles.exhaust.push(c.state.piles.hand.remove(at));
    let draw_hooks=c.state.hook_events.iter().filter(|event|event.hook==Hook::CardDrawn).count();
    c.step_full(&CombatAction::PlayCard{instance:3,target:None}).unwrap();
    assert!(c.state.pending_selection.is_none(),"a singleton Exhume return is forced, not a chooser");
    assert!(c.state.piles.hand.iter().any(|card|card.handle()==id));
    assert!(c.state.piles.exhaust.iter().all(|card|card.handle()!=id));
    assert_eq!(c.state.hook_events.iter().filter(|event|event.hook==Hook::CardDrawn).count(),draw_hooks,
        "returning the owned instance invokes no draw callback");
}

#[test]
fn m10_exhume_skips_empty_eligible_pile_and_keeps_parent_finish(){
    let mut c=with_card(crate::content::CardId::Exhume,false);
    c.state.piles.exhaust.clear();
    c.step_full(&CombatAction::PlayCard{instance:3,target:None}).unwrap();
    assert!(c.state.pending_selection.is_none());
    assert!(c.state.piles.exhaust.iter().any(|card|card.handle().as_str()=="3"));
}

#[test]
fn m10_exhume_with_only_exhume_cards_skips_selection(){
    let mut c=with_card(crate::content::CardId::Exhume,false);
    let exhausted=card_state::permanent(crate::content::CardId::Exhume,CardInstanceId(98),0);
    let id=card_state::handle(exhausted.instance);c.state.card_ids_issued.insert(id.clone());c.state.piles.exhaust.push(OwnedCard::Full(exhausted));
    c.step_full(&CombatAction::PlayCard{instance:3,target:None}).unwrap();
    assert!(c.state.pending_selection.is_none());
    assert!(c.state.piles.exhaust.iter().any(|card|card.handle()==id));
    assert!(c.state.piles.exhaust.iter().any(|card|card.handle().as_str()=="3"));
}

#[test]
fn m10_exhume_rejects_exhume_candidates_and_preserves_same_owned_instance_stats(){
    let mut c=with_card(crate::content::CardId::Exhume,true);
    let excluded=card_state::permanent(crate::content::CardId::Exhume,CardInstanceId(91),1);
    let excluded_id=card_state::handle(excluded.instance);
    c.state.card_ids_issued.insert(excluded_id.clone());c.state.piles.exhaust.push(OwnedCard::Full(excluded));
    let mut returning=card_state::permanent(crate::content::CardId::Strike,CardInstanceId(92),0);
    returning.local.randomized_cost=Some(2);
    let id=card_state::handle(returning.instance);c.state.card_ids_issued.insert(id.clone());c.state.piles.exhaust.push(OwnedCard::Full(returning));
    let second=card_state::permanent(crate::content::CardId::Defend,CardInstanceId(93),0);
    let second_id=card_state::handle(second.instance);c.state.card_ids_issued.insert(second_id);c.state.piles.exhaust.push(OwnedCard::Full(second));
    c.step_full(&CombatAction::PlayCard{instance:3,target:None}).unwrap();
    let selection=c.state.pending_selection.as_ref().unwrap();
    assert_eq!(selection.operation,SelectionOperation::ReturnFromExhaust);
    assert!(!selection.candidates.contains(&excluded_id));
    // The other legal card is returned by identity, retaining its local state.
    let chosen=selection.candidates[0].as_str().parse().unwrap();
    c.step_full(&CombatAction::SelectCard{instance:chosen}).unwrap();
    let returned=c.state.piles.hand.iter().find(|card|card.handle().as_str()==chosen.to_string()).unwrap().full().unwrap();
    assert_eq!(returned.local.randomized_cost,Some(2));
}

#[test]
fn m10_optional_selection_can_confirm_zero_and_retains_consumed_parent(){
    // Internal shared-contract diagnostic only: this retains the supplied PotionId
    // but does not claim an Elixir producer/use implementation.
    let mut c=super::full_runtime::entered_combat(3,3, |_| {});
    let candidates=c.state.piles.hand.iter().take(2).map(OwnedCard::handle).collect::<Vec<_>>();
    c.state.pending_selection=Some(PendingSelection{
        parent:SelectionParent::Potion{slot:0,id:crate::content::PotionId::FirePotion},source_zone:SelectionSourceZone::Hand,
        candidates:candidates.clone(),operation:SelectionOperation::Move(SelectionDisposition::Exhaust),
        selected:vec![],minimum:0,maximum:2,disposition:SelectionDisposition::Exhaust,empty:EmptySelection::Continue,
    });
    c.state.pending.push_back(Effect{source:Source::None,target:None,kind:EffectKind::Decision});
    c.state.phase=Phase::Selection;
    assert!(engine::validate_runtime(&c.state).is_ok());
    assert_eq!(c.public_full().unwrap().selection.unwrap().parent,
        SelectionParent::Potion{slot:0,id:crate::content::PotionId::FirePotion});
    let menu=c.legal_actions_full().unwrap();
    assert!(menu.contains(&CombatAction::ConfirmSelection),"zero is confirmable immediately");
    assert_eq!(menu.iter().filter(|action|matches!(action,CombatAction::SelectCard{..})).count(),2);
    assert_eq!(c.step_full(&CombatAction::EndTurn),Err(crate::run::RunError::InvalidAction));
    let selected=candidates[0].as_str().parse().unwrap();
    c.step_full(&CombatAction::SelectCard{instance:selected}).unwrap();
    assert_eq!(c.state.phase,Phase::Selection,"optional selection does not auto-complete after a pick");
    assert!(c.state.piles.exhaust.is_empty(),"choice is staged until explicit confirmation");
    let stale=candidates[1].as_str().parse().unwrap();
    let stale_card=c.state.piles.hand.iter().position(|card|card.handle()==candidates[1]).map(|i|c.state.piles.hand.remove(i)).unwrap();
    let before_stale=c.state.clone();
    assert_eq!(c.step_full(&CombatAction::SelectCard{instance:stale}),Err(crate::run::RunError::InvalidAction));
    assert_eq!(c.state,before_stale,"stale candidate is rejected atomically");
    c.state.piles.hand.push(stale_card);
    let before=c.state.clone();
    assert_eq!(c.step_full(&CombatAction::SelectCard{instance:selected}),Err(crate::run::RunError::InvalidAction));
    assert_eq!(c.state,before,"duplicate selection is atomic");
    c.step_full(&CombatAction::ConfirmSelection).unwrap();
    assert_eq!(c.state.phase,Phase::Decision);
    assert_eq!(c.state.piles.exhaust.iter().map(OwnedCard::handle).collect::<Vec<_>>(),vec![candidates[0].clone()]);
    assert!(c.state.pending_selection.is_none());
}

#[test]
fn m10_optional_selection_zero_pick_confirmation_preserves_owned_state(){
    // Internal shared-contract characterization only: this supplies the real
    // PotionId parent but is not evidence of an Elixir producer or potion use.
    let mut c=super::full_runtime::entered_combat(3,3, |_| {});
    let candidates=c.state.piles.hand.iter().take(2).map(OwnedCard::handle).collect::<Vec<_>>();
    c.state.pending_selection=Some(PendingSelection{
        parent:SelectionParent::Potion{slot:1,id:crate::content::PotionId::FirePotion},
        source_zone:SelectionSourceZone::Hand,candidates,operation:SelectionOperation::Move(SelectionDisposition::Exhaust),
        selected:vec![],minimum:0,maximum:2,disposition:SelectionDisposition::Exhaust,empty:EmptySelection::Continue,
    });
    c.state.pending.push_back(Effect{source:Source::None,target:None,kind:EffectKind::Decision});
    c.state.phase=Phase::Selection;
    assert!(engine::validate_runtime(&c.state).is_ok());
    assert_eq!(c.public_full().unwrap().selection.unwrap().parent,
        SelectionParent::Potion{slot:1,id:crate::content::PotionId::FirePotion},
        "the pre-boundary keeps the actual potion identity and slot");
    let piles_before=c.state.piles.clone();
    let rng_before=c.state.card_rng.clone();
    let issued_before=c.state.card_ids_issued.clone();
    let energy_before=c.state.player.energy;
    assert!(c.legal_actions_full().unwrap().contains(&CombatAction::ConfirmSelection));

    c.step_full(&CombatAction::ConfirmSelection).unwrap();

    assert!(engine::validate_runtime(&c.state).is_ok());
    assert_eq!(c.state.phase,Phase::Decision);
    assert!(c.state.pending_selection.is_none());
    assert!(c.state.pending.is_empty());
    assert_eq!(c.state.piles,piles_before,"zero picks leave every owned pile unchanged");
    assert_eq!(c.state.card_rng,rng_before,"confirmation consumes no card RNG");
    assert_eq!(c.state.card_ids_issued,issued_before,"confirmation invents no card identity");
    assert_eq!(c.state.player.energy,energy_before,"the diagnostic does not refund or spend energy");
    assert!(c.state.in_play.is_none(),"no fabricated card parent is installed");
    assert!(c.state.fault.is_none());
}

#[test]
fn m10_confirmation_rejects_a_staged_card_that_left_its_source_atomically(){
    let mut c=super::full_runtime::entered_combat(3,3, |_| {});
    let candidates=c.state.piles.hand.iter().take(2).map(OwnedCard::handle).collect::<Vec<_>>();
    c.state.pending_selection=Some(PendingSelection{parent:SelectionParent::Potion{slot:0,id:crate::content::PotionId::FirePotion},source_zone:SelectionSourceZone::Hand,
        candidates:candidates.clone(),operation:SelectionOperation::Move(SelectionDisposition::Exhaust),selected:vec![candidates[0].clone()],minimum:1,maximum:2,
        disposition:SelectionDisposition::Exhaust,empty:EmptySelection::Continue});
    c.state.pending.push_back(Effect{source:Source::None,target:None,kind:EffectKind::Decision});c.state.phase=Phase::Selection;
    let at=c.state.piles.hand.iter().position(|card|card.handle()==candidates[0]).unwrap();c.state.piles.hand.remove(at);
    let before=c.state.clone();
    assert_eq!(c.step_full(&CombatAction::ConfirmSelection),Err(crate::run::RunError::InvalidAction));
    assert_eq!(c.state,before,"stale staged ownership rejects before selection custody or piles mutate");
}

#[test]
fn m10_confirmation_rejects_changed_hand_capacity_atomically(){
    let mut c=super::full_runtime::entered_combat(3,3, |_| {});add_draw(&mut c,12);
    while c.state.piles.hand.len()<9{let index=c.state.piles.draw.len()-1;c.state.piles.hand.push(c.state.piles.draw.remove(index));}
    let candidate=c.state.piles.draw[c.state.piles.draw.len()-2].handle();
    let second=c.state.piles.draw[c.state.piles.draw.len()-3].handle();
    c.state.pending_selection=Some(PendingSelection{parent:SelectionParent::Potion{slot:0,id:crate::content::PotionId::FirePotion},source_zone:SelectionSourceZone::Draw,
        candidates:vec![candidate.clone(),second],operation:SelectionOperation::Move(SelectionDisposition::Hand),selected:vec![candidate],minimum:1,maximum:2,
        disposition:SelectionDisposition::Hand,empty:EmptySelection::Continue});
    c.state.pending.push_back(Effect{source:Source::None,target:None,kind:EffectKind::Decision});c.state.phase=Phase::Selection;
    let index=c.state.piles.draw.len()-1;c.state.piles.hand.push(c.state.piles.draw.remove(index));
    assert_eq!(c.state.piles.hand.len(),10);
    let before=c.state.clone();
    assert_eq!(c.step_full(&CombatAction::ConfirmSelection),Err(crate::run::RunError::InvalidAction));
    assert_eq!(c.state,before,"destination capacity is revalidated before any accepted movement");
}

#[test]
fn m10_multi_pick_from_draw_preflights_full_hand_and_commits_only_legal_prefix(){
    let mut c=super::full_runtime::entered_combat(3,3, |_| {});
    add_draw(&mut c,8);
    while c.state.piles.hand.len()<9{
        let index=c.state.piles.draw.len()-1;
        c.state.piles.hand.push(c.state.piles.draw.remove(index));
    }
    let candidates=c.state.piles.draw.iter().take(2).map(OwnedCard::handle).collect::<Vec<_>>();
    c.state.pending_selection=Some(PendingSelection{parent:SelectionParent::Potion{slot:0,id:crate::content::PotionId::FirePotion},source_zone:SelectionSourceZone::Draw,
        candidates:candidates.clone(),operation:SelectionOperation::Move(SelectionDisposition::Hand),selected:vec![],minimum:1,maximum:2,
        disposition:SelectionDisposition::Hand,empty:EmptySelection::Continue});
    c.state.pending.push_back(Effect{source:Source::None,target:None,kind:EffectKind::Decision});c.state.phase=Phase::Selection;
    let first=candidates[0].as_str().parse().unwrap();let second=candidates[1].as_str().parse().unwrap();
    c.step_full(&CombatAction::SelectCard{instance:first}).unwrap();
    assert!(!c.legal_actions_full().unwrap().contains(&CombatAction::SelectCard{instance:second}));
    assert!(c.legal_actions_full().unwrap().contains(&CombatAction::ConfirmSelection));
    c.step_full(&CombatAction::ConfirmSelection).unwrap();
    assert_eq!(c.state.piles.hand.len(),10);
    assert!(c.state.piles.hand.iter().any(|card|card.handle()==candidates[0]));
    assert!(c.state.piles.draw.iter().any(|card|card.handle()==candidates[1]));
}

#[test]
fn m10_accepted_optional_confirmation_fault_retains_selection_and_does_not_retry(){
    let mut c=super::full_runtime::entered_combat(3,3, |_| {});
    let candidates=c.state.piles.hand.iter().take(2).map(OwnedCard::handle).collect::<Vec<_>>();
    c.state.pending_selection=Some(PendingSelection{parent:SelectionParent::Potion{slot:0,id:crate::content::PotionId::FirePotion},source_zone:SelectionSourceZone::Hand,
        candidates:candidates.clone(),operation:SelectionOperation::Move(SelectionDisposition::Exhaust),selected:vec![],minimum:1,maximum:2,
        disposition:SelectionDisposition::Exhaust,empty:EmptySelection::Continue});
    c.state.pending.push_back(Effect{source:Source::None,target:None,kind:EffectKind::Decision});c.state.phase=Phase::Selection;
    let selected=candidates[0].as_str().parse().unwrap();c.step_full(&CombatAction::SelectCard{instance:selected}).unwrap();
    c.state.resolved_effects=99_999;
    assert!(c.step_full(&CombatAction::ConfirmSelection).is_err());
    assert!(c.state.fault.is_some());assert!(c.state.fault_selection.is_some());
    assert_eq!(c.state.fault_selection.as_ref().unwrap().parent,
        SelectionParent::Potion{slot:0,id:crate::content::PotionId::FirePotion});
    assert!(c.state.piles.exhaust.iter().any(|card|card.handle()==candidates[0]));
    assert!(!c.state.piles.hand.iter().any(|card|card.handle()==candidates[0]));
    let retained=c.state.clone();
    assert!(c.step_full(&CombatAction::ConfirmSelection).is_err());
    assert_eq!(c.state,retained,"accepted confirmation fault is retained and never retried");
}
