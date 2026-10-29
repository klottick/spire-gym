use super::super::*;
use crate::content::{CardId,Character,CardColor,CardRarity,CardType,Catalog};
#[test]
fn m20_defect_generators_complete_support_and_capture(){
    for (pool,count,kind) in [(GenerationPool::Attack,23,Some(CardType::Attack)),
        (GenerationPool::Skill,34,Some(CardType::Skill)),(GenerationPool::Power,13,Some(CardType::Power)),
        (GenerationPool::Any,70,None)] {
        let ids=generation::pool(Some(Character::Defect),pool).expect("complete actual blue family");
        assert_eq!(ids.len(),count);assert_eq!(ids.iter().collect::<std::collections::BTreeSet<_>>().len(),count);
        for id in ids {
            let row=Catalog::standard().card(id).unwrap();assert_eq!(row.color,CardColor::Blue);
            assert!(!matches!(row.rarity,CardRarity::Basic));assert_ne!(id.as_str(),"self_repair");
            if let Some(kind)=kind{assert_eq!(row.card_type,kind);}
            for up in 0..=1{assert!(card_state::generated(id,CardInstanceId(0),up).is_some());}
        }
    }
    assert_eq!(generation::pool(Some(Character::Defect),GenerationPool::Colorless).unwrap().len(),34);
    assert!(generation::pool(Some(Character::Ironclad),GenerationPool::Any).is_err(),"no unsupported actual-character substitution");
    for (pool,count) in [(GenerationPool::Attack,26),(GenerationPool::Skill,33),(GenerationPool::Power,11),(GenerationPool::Any,70)] {
        assert_eq!(generation::pool(Some(Character::Silent),pool).unwrap().len(),count,"accepted Silent domain remains unchanged");
    }
}

use model::OwnedCard;
use generation::{GenerationWork,GenerationCost};
fn fight(id:CardId)->Combat{
    let mut c=super::full_runtime::entered_combat(9,9,|card|*card=permanent_live_card(id,card.instance,0));
    c.state.character=characters::CharacterState::for_character(Character::Defect);c.state.silent.source_character=Some(Character::Defect);
    c.state.enemies[0].hp=500;c.state.enemies[0].max_hp=500;c
}
fn local_cards(c:&Combat)->Vec<&LiveCard>{c.state.piles.hand.iter().chain(&c.state.piles.discard).chain(&c.state.piles.draw).filter_map(OwnedCard::full).filter(|card|matches!(card.origin,CombatCardRef::CombatLocal(_))).collect()}
#[test]
fn m20_defect_generators_white_noise_real_capture_and_cost_layers(){
    let mut c=fight(CardId::WhiteNoise);let before=c.state.card_rng.clone();
    c.step_full(&CombatAction::PlayCard{instance:3,target:None}).unwrap();
    assert_ne!(c.state.card_rng,before);let cards=local_cards(&c);assert_eq!(cards.len(),1);
    let generated=cards[0];assert_eq!(Catalog::standard().card(generated.id).unwrap().card_type,CardType::Power);
    assert_ne!(generated.id,CardId::SelfRepair);assert_eq!(generated.local.turn_cost,Some(0));
    assert!(!generated.local.free_to_play_once&&!generated.local.free_play);assert_eq!(generated.upgrades,0);
    assert_eq!(card_state::combat_price(generated,cards::starting_cost(generated.id,0).unwrap()).unwrap(),cards::starting_cost(generated.id,0).unwrap(),"turn zero leaves ordinary combat cost");
    assert!(c.state.piles.exhaust.iter().any(|card|card.handle().as_str()=="3"));
}
#[test]
fn m20_defect_generators_pre_draw_callbacks_capture_then_deliver_and_keep_distinct_guards(){
    for id in [CardId::HelloWorld,CardId::CreativeAI]{
        let mut c=fight(id);c.step_full(&CombatAction::PlayCard{instance:3,target:None}).unwrap();
        let mut callbacks=vec![];let before=c.state.card_rng.clone();let issued=c.state.card_ids_issued.clone();
        generation::blue_start_turn(&mut c.state,Hook::TurnStartedPreDraw,Some(&Actor::Player),&mut callbacks).unwrap();
        assert_ne!(c.state.card_rng,before);assert_eq!(c.state.card_ids_issued,issued,"callback only captures; no child delivered yet");assert_eq!(callbacks.len(),1);
        let EffectKind::Generate(work)=&callbacks[0].kind else{panic!("captured request")};assert!(work.captured);assert_eq!(work.cards.len(),1);
        assert_eq!(work.cost,GenerationCost::Ordinary);assert_eq!(work.pool,if id==CardId::HelloWorld{GenerationPool::Common}else{GenerationPool::Power});
        c.enqueue_native_effects(callbacks,None).unwrap();let generated=local_cards(&c);assert_eq!(generated.len(),1);assert!(!generated[0].local.free_to_play_once);
        assert_eq!(generated[0].local.turn_cost,cards::starting_cost(generated[0].id,0));
        if id==CardId::HelloWorld{assert_eq!(Catalog::standard().card(generated[0].id).unwrap().rarity,CardRarity::Common);}
        else{assert_eq!(Catalog::standard().card(generated[0].id).unwrap().card_type,CardType::Power);}
        c.state.enemies[0].hp=0;c.state.enemies[0].life=ActorLife::Replaced;let before=c.state.card_rng.clone();let mut work=vec![];
        generation::blue_start_turn(&mut c.state,Hook::TurnStartedPreDraw,Some(&Actor::Player),&mut work).unwrap();
        assert_eq!(work.len(),usize::from(id==CardId::CreativeAI));assert_eq!(c.state.card_rng==before,id==CardId::HelloWorld);
    }
}
#[test]
fn m20_defect_generators_callback_rng_fault_preserves_already_captured_prefix(){
    let mut c=fight(CardId::HelloWorld);c.step_full(&CombatAction::PlayCard{instance:3,target:None}).unwrap();
    c.state.player_full_powers.iter_mut().find(|power|power.id.0=="hello_world").unwrap().state.amount=2;
    let stream=c.state.card_rng.as_ref().unwrap();c.state.card_rng=Some(rng::StreamState::new(stream.id().clone(),*stream.key(),(1u128<<64)-1).unwrap());
    let mut callbacks=vec![];let err=generation::blue_start_turn(&mut c.state,Hook::TurnStartedPreDraw,Some(&Actor::Player),&mut callbacks).unwrap_err();
    assert!(matches!(err,EffectError::Random(_)));assert!(callbacks.is_empty());
    let first=c.state.pending.front().expect("first captured callback retained");assert!(matches!(&first.kind,EffectKind::Generate(work) if work.captured&&work.cards.len()==1));
    assert!(matches!(&c.state.current.as_ref().unwrap().kind,EffectKind::Generate(work) if !work.captured&&work.cards.is_empty()));
    assert_eq!(c.state.card_rng.as_ref().unwrap().word_index(),1u128<<64);
}
#[test]
fn m20_defect_generators_creation_upgrade_uses_same_template_checkpoint(){
    let c=fight(CardId::WhiteNoise);
    let work=GenerationWork{pool:GenerationPool::Power,count:1,unique:false,destination:CardDestination::Hand,cost:GenerationCost::TurnZero,upgraded:true,offer:None,captured:true,cards:vec![CardId::CreativeAI],next:0};
    let template=generation::creation_template(&c.state,&work,CardId::CreativeAI,true).unwrap();
    assert_eq!(template.upgrades,1);assert_eq!(template.local.mutable_combat_cost,Some(2));assert_eq!(template.local.turn_cost,Some(0));
    assert!(!template.local.free_to_play_once);assert_eq!(template.instance,CardInstanceId(0));
    let ga=card_state::generated_with_history(CardId::GeneticAlgorithm,CardInstanceId(5),1,card_state::CardConstructionHistory::from_state(&c.state)).unwrap();
    assert_eq!(ga.misc,1,"fresh GA never imports permanent growth");
}

#[test]
fn m20_defect_generators_all_75_blue_base_plus_dispatch_and_shared_temporary_upgrade(){
    use crate::items::{CombatItemContext,NativeBattleContext};
    for id in crate::content::defect::M20_DEFECT_CARDS{for up in 0..=1{
        let mut e=super::m8_shapes::entry(crate::content::EncounterId::ThreeShapes);e.setup.character=Character::Defect;e.setup.player.energy=9;e.setup.player.max_energy=9;
        e.cards=vec![permanent_live_card(CardId::DefendBlue,CardInstanceId(1),0),permanent_live_card(CardId::DefendBlue,CardInstanceId(2),0),permanent_live_card(*id,CardInstanceId(3),up)];e.opening_draw=3;
        let mut owner=Some(CombatItemContext::new(vec![],vec![None;3]).with_battle_context(NativeBattleContext{m20:true,..Default::default()}));
        let mut c=Combat::enter_with_item_context(e,&mut owner).unwrap();
        let before=c.snapshot();let actions=c.legal_actions_full().unwrap();assert_eq!(c.snapshot(),before,"{id:?}+{up} listing does not mutate");
        let action=actions.into_iter().find(|action|matches!(action,CombatAction::PlayCard{instance:3,..})).unwrap_or_else(||panic!("no actual blue play {id:?}+{up}"));
        c.step_full(&action).unwrap_or_else(|error|panic!("actual blue {id:?}+{up}: {error:?}"));
        for _ in 0..8{
            if c.state.phase!=Phase::Selection{break}
            let actions=c.legal_actions_full().unwrap();let action=actions.iter().find(|action|matches!(action,CombatAction::ConfirmSelection)).or_else(||actions.iter().find(|action|matches!(action,CombatAction::SelectCard{..}|CombatAction::ChooseCardOffer{..}))).expect("finite mandatory choice");
            c.step_full(action).unwrap_or_else(|error|panic!("blue choice {id:?}+{up}: {error:?}"));
        }
        assert_ne!(c.state.phase,Phase::Selection);assert!(c.state.fault.is_none(),"{id:?}+{up}");engine::validate_runtime(&c.state).unwrap_or_else(|error|panic!("blue census {id:?}+{up}: {error:?}"));
        // The real upgrade owner keeps original misc while changing combat face.
        let mut card=card_state::generated(*id,CardInstanceId(900),0).unwrap();let misc=card.misc;
        cards::ironclad::rules::upgrade(&mut card).unwrap();assert_eq!(card.upgrades,1);assert_eq!(card.misc,misc,"temporary upgrade cannot invent permanent growth");
    }}
}
#[test]
fn m20_defect_generators_hello_basically_dead_guard_keeps_halfdead_non_dying_actor(){
    let mut c=fight(CardId::HelloWorld);c.step_full(&CombatAction::PlayCard{instance:3,target:None}).unwrap();
    c.state.enemies[0].hp=0;c.state.enemies[0].life=ActorLife::HalfDead;let mut callbacks=vec![];
    generation::blue_start_turn(&mut c.state,Hook::TurnStartedPreDraw,Some(&Actor::Player),&mut callbacks).unwrap();
    assert_eq!(callbacks.len(),1,"source MonsterGroup guard tests dying/escaping, not HalfDead or HP alone");
}
#[test]
fn m20_defect_generators_all_l34_base_plus_use_actual_defect_dependency_paths(){
    use crate::items::{CombatItemContext,NativeBattleContext};
    let ids=generation::pool(Some(Character::Defect),GenerationPool::Colorless).unwrap();assert_eq!(ids.len(),34);
    for id in ids{for up in 0..=1{
        let mut e=super::m8_shapes::entry(crate::content::EncounterId::ThreeShapes);e.setup.character=Character::Defect;e.setup.player.energy=9;e.setup.player.max_energy=9;
        e.cards=vec![permanent_live_card(CardId::DefendBlue,CardInstanceId(1),0),permanent_live_card(CardId::DefendBlue,CardInstanceId(2),0),card_state::generated(id,CardInstanceId(3),up).unwrap()];e.opening_draw=3;
        let mut owner=Some(CombatItemContext::new(vec![],vec![None;3]).with_battle_context(NativeBattleContext{m20:true,..Default::default()}));let mut c=Combat::enter_with_item_context(e,&mut owner).unwrap();
        for (n,kind)in [(4,CardId::DefendBlue),(5,CardId::StrikeBlue)]{let card=permanent_live_card(kind,CardInstanceId(n),0);assert!(c.state.card_ids_issued.insert(card_state::handle(card.instance)));c.state.piles.draw.push(OwnedCard::Full(card));}
        let action=c.legal_actions_full().unwrap().into_iter().find(|a|matches!(a,CombatAction::PlayCard{instance:3,..})).unwrap_or_else(||panic!("L34 actualDefect {id:?}+{up} missing"));
        c.step_full(&action).unwrap_or_else(|error|panic!("L34 actualDefect {id:?}+{up}: {error:?}"));
        for _ in 0..12{if c.state.phase!=Phase::Selection{break}let actions=c.legal_actions_full().unwrap();let action=actions.iter().find(|a|matches!(a,CombatAction::ConfirmSelection)).or_else(||actions.iter().find(|a|matches!(a,CombatAction::SelectCard{..}|CombatAction::ChooseCardOffer{..}))).unwrap();c.step_full(action).unwrap();}
        assert_ne!(c.state.phase,Phase::Selection);assert!(c.state.fault.is_none(),"L34 {id:?}+{up}");engine::validate_runtime(&c.state).unwrap_or_else(|error|panic!("L34 {id:?}+{up} census: {error:?}"));
        if matches!(id,CardId::Discovery|CardId::Chrysalis|CardId::Metamorphosis){
            for card in local_cards(&c).into_iter().filter(|card|card.instance.0!=3){assert_eq!(Catalog::standard().card(card.id).unwrap().color,CardColor::Blue,"nested actual-character request never substitutes Silent");assert_ne!(card.id,CardId::SelfRepair);}
        }
    }}
}
#[test]
fn m20_defect_generators_magnetism_m20_captures_in_callback_and_preserves_legacy_checkpoint(){
    let mut c=fight(CardId::Magnetism);c.state.item_context=Some(crate::items::CombatItemContext::new(vec![],vec![None;3]).with_m20_profile());c.step_full(&CombatAction::PlayCard{instance:3,target:None}).unwrap();
    for halfdead in [false,true]{
        let mut state=c.state.clone();if halfdead{state.enemies[0].hp=0;state.enemies[0].life=ActorLife::HalfDead;}
        let rng=state.card_rng.clone();let issued=state.card_ids_issued.clone();let mut work=vec![];
        generation::start_turn(&state,Hook::TurnStartedPreDraw,Some(&Actor::Player),&mut work);generation::blue_start_turn(&mut state,Hook::TurnStartedPreDraw,Some(&Actor::Player),&mut work).unwrap();
        assert_eq!(work.len(),1);assert_ne!(state.card_rng,rng,"M20 Magnetism samples during source callback");assert_eq!(state.card_ids_issued,issued,"delivery remains later");assert!(matches!(&work[0].kind,EffectKind::Generate(w) if w.captured&&w.pool==GenerationPool::Colorless&&w.cards.len()==1));
    }
    let mut old=c.state.clone();old.silent.source_character=Some(Character::Silent);old.item_context=Some(crate::items::CombatItemContext::new(vec![],vec![None;3]).with_m15_profile());let rng=old.card_rng.clone();let mut work=vec![];generation::start_turn(&old,Hook::TurnStartedPreDraw,Some(&Actor::Player),&mut work);generation::blue_start_turn(&mut old,Hook::TurnStartedPreDraw,Some(&Actor::Player),&mut work).unwrap();assert_eq!(old.card_rng,rng);assert!(matches!(&work[0].kind,EffectKind::Generate(w) if !w.captured),"frozen M15 callback bytes retain their checkpoint");
}
#[test]
fn m20_defect_generators_magnetism_callback_fault_keeps_captured_prefix(){
    let mut c=fight(CardId::Magnetism);c.state.item_context=Some(crate::items::CombatItemContext::new(vec![],vec![None;3]).with_m20_profile());c.step_full(&CombatAction::PlayCard{instance:3,target:None}).unwrap();
    c.state.player_full_powers.iter_mut().find(|p|p.id.0=="magnetism").unwrap().state.amount=2;let stream=c.state.card_rng.as_ref().unwrap();c.state.card_rng=Some(rng::StreamState::new(stream.id().clone(),*stream.key(),(1u128<<64)-1).unwrap());
    let mut work=vec![];generation::start_turn(&c.state,Hook::TurnStartedPreDraw,Some(&Actor::Player),&mut work);assert!(work.is_empty());
    assert!(matches!(generation::blue_start_turn(&mut c.state,Hook::TurnStartedPreDraw,Some(&Actor::Player),&mut work),Err(EffectError::Random(_))));assert!(work.is_empty());
    assert!(matches!(&c.state.pending.front().unwrap().kind,EffectKind::Generate(w) if w.captured&&w.cards.len()==1));assert!(matches!(&c.state.current.as_ref().unwrap().kind,EffectKind::Generate(w) if !w.captured&&w.cards.is_empty()));assert_eq!(c.state.card_rng.as_ref().unwrap().word_index(),1u128<<64);
}
#[test]
fn m20_defect_generators_bark_second_blue_offer_delivery_fault_keeps_first_and_cursor(){
    use crate::{content::{PotionId,RelicId},items::{CombatItemContext,NativeBattleContext}};
    let mut e=super::m8_shapes::entry(crate::content::EncounterId::ThreeShapes);e.setup.character=Character::Defect;e.cards=vec![permanent_live_card(CardId::DefendBlue,CardInstanceId(1),0)];e.opening_draw=1;
    let mut owner=Some(CombatItemContext::new(vec![RelicId::SacredBark],vec![Some(PotionId::PowerPotion),None,None]).with_battle_context(NativeBattleContext{m20:true,..Default::default()}));let mut c=Combat::enter_with_item_context(e,&mut owner).unwrap();
    c.step_full(&CombatAction::UsePotion{slot:0,target:None}).unwrap();let offer=c.public_full().unwrap().card_offer.unwrap();assert!(offer.optional);let rng=c.state.card_rng.clone();
    for n in 2..=998{let card=permanent_live_card(CardId::DefendBlue,CardInstanceId(n),0);assert!(c.state.card_ids_issued.insert(card_state::handle(card.instance)));c.state.piles.discard.push(OwnedCard::Full(card));}
    assert!(c.step_full(&CombatAction::ChooseCardOffer{index:0}).is_err());assert_eq!(c.item_context().unwrap().potions()[0],None);assert_eq!(c.state.card_rng,rng);
    let EffectKind::DeliverCardOffer{id,copies,next,..}=&c.state.current.as_ref().unwrap().kind else{panic!("retained chosen template and delivery cursor")};assert_eq!((*id,*copies,*next),(offer.cards[0],2,1));
    let copies=c.state.piles.hand.iter().filter_map(OwnedCard::full).filter(|card|matches!(card.origin,CombatCardRef::CombatLocal(_))).collect::<Vec<_>>();assert_eq!(copies.len(),1);assert_eq!(copies[0].id,offer.cards[0]);assert_eq!(copies[0].local.turn_cost,Some(0));assert_eq!(c.state.card_ids_issued.len(),999);
    let frozen=c.snapshot();assert!(c.step_full(&CombatAction::ChooseCardOffer{index:0}).is_err());assert_eq!(c.snapshot(),frozen);
}
