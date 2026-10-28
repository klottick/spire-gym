use super::super::*;
use crate::content::{Catalog,CardId,Character,CardType};
use model::OwnedCard;

fn id(token:&str)->CardId{Catalog::standard().cards().iter().find(|row|row.id.as_str()==token).unwrap_or_else(||panic!("missing generated kind {token}")).id}
fn entry(token:&str,upgrades:u32)->CombatEntry{
    let mut entry=super::m8_shapes::entry(crate::content::EncounterId::ThreeShapes);
    entry.setup.character=Character::Silent;entry.setup.player.energy=9;entry.setup.player.max_energy=9;
    entry.cards=vec![permanent_live_card(CardId::DefendGreen,CardInstanceId(1),0),permanent_live_card(CardId::DefendGreen,CardInstanceId(2),0),permanent_live_card(id(token),CardInstanceId(3),upgrades)];
    entry.opening_draw=3;entry
}
fn fight(token:&str,upgrades:u32)->Combat{Combat::enter(entry(token,upgrades)).unwrap()}
#[test]
fn m15_generation_distraction_uses_complete_skill_support_and_real_temporary_cards(){
    for upgrades in [0,1]{
        let mut c=fight("distraction",upgrades);
        let before=c.state.card_rng.clone();
        c.step_full(&CombatAction::PlayCard{instance:3,target:None}).unwrap();
        assert_ne!(c.state.card_rng,before);
        let generated=c.state.piles.hand.iter().filter_map(OwnedCard::full).find(|card|matches!(card.origin,CombatCardRef::CombatLocal(_))).unwrap();
        assert_eq!(Catalog::standard().card(generated.id).unwrap().card_type,CardType::Skill);
        if !generated.local.x_cost&&!matches!(generated.id,CardId::Reflex|CardId::Tactician){assert_eq!(generated.local.turn_cost,Some(0));}
        assert!(c.state.piles.exhaust.iter().any(|card|card.handle().as_str()=="3"));
    }
}
#[test]
fn m15_generation_discovery_suspends_real_parent_with_mandatory_offer(){
    let mut c=fight("discovery",0);
    c.step_full(&CombatAction::PlayCard{instance:3,target:None}).unwrap();
    let v=c.public_full().unwrap();let offer=v.card_offer.unwrap();
    assert_eq!(offer.cards.len(),3);assert!(!offer.optional);
    assert_eq!(c.state.in_play.as_ref().unwrap().handle().as_str(),"3");
    let snap=c.snapshot();assert!(c.step_full(&CombatAction::SkipCardOffer).is_err());assert_eq!(c.snapshot(),snap);
    c.step_full(&CombatAction::ChooseCardOffer{index:0}).unwrap();
    assert!(c.state.in_play.is_none());assert_eq!(c.state.phase,Phase::Decision);
    let generated=c.state.piles.hand.iter().filter_map(OwnedCard::full).find(|card|matches!(card.origin,CombatCardRef::CombatLocal(_))).unwrap();
    assert_eq!(generated.id,offer.cards[0]);
}

#[test]
fn m15_generation_complete_supports_preserve_source_eligibility(){
    for (kind,n) in [(GenerationPool::Attack,26),(GenerationPool::Skill,33),(GenerationPool::Power,11),(GenerationPool::Any,70),(GenerationPool::Colorless,34)]{
        let support=generation::pool(Some(Character::Silent),kind).unwrap();assert_eq!(support.len(),n);
        assert!(!support.contains(&id("alchemize")));assert!(!support.contains(&CardId::StrikeGreen));
        for id in support {assert!(card_state::generated(id,CardInstanceId(0),0).is_some());}
    }
    let skills=generation::pool(Some(Character::Silent),GenerationPool::Skill).unwrap();
    for token in ["distraction","reflex","tactician","doppelganger","malaise"]{assert!(skills.contains(&id(token)));}
    assert!(generation::pool(Some(Character::Ironclad),GenerationPool::Any).is_err());
}
#[test]
fn m15_generation_chrysalis_and_metamorphosis_insert_all_samples_without_drawing(){
    for (token,kind) in [("chrysalis",CardType::Skill),("metamorphosis",CardType::Attack)]{
        for (upgrades,count) in [(0,3),(1,5)]{
            let mut c=fight(token,upgrades);let drawn=c.state.silent.drawn_cards.clone();
            c.step_full(&CombatAction::PlayCard{instance:3,target:None}).unwrap();
            let generated=c.state.piles.draw.iter().filter_map(OwnedCard::full).filter(|card|matches!(card.origin,CombatCardRef::CombatLocal(_))).collect::<Vec<_>>();
            assert_eq!(generated.len(),count);assert_eq!(c.state.silent.drawn_cards,drawn);
            for card in generated{assert_eq!(Catalog::standard().card(card.id).unwrap().card_type,kind);assert_eq!(card.upgrades,0);
                if !card.local.x_cost&&!matches!(card.id,CardId::Reflex|CardId::Tactician){assert_eq!(card.local.mutable_combat_cost,Some(0));assert_eq!(card.local.turn_cost,Some(0));}}
        }
    }
}
#[test]
fn m15_generation_jack_and_transmutation_make_real_colorless_cards(){
    for (token,upgrades,count) in [("jack_of_all_trades",0,1),("jack_of_all_trades",1,2),("transmutation",0,9),("transmutation",1,9)]{
        let mut c=fight(token,upgrades);c.step_full(&CombatAction::PlayCard{instance:3,target:None}).unwrap();
        let generated=c.state.piles.hand.iter().chain(c.state.piles.discard.iter()).filter_map(OwnedCard::full).filter(|card|matches!(card.origin,CombatCardRef::CombatLocal(_))).collect::<Vec<_>>();
        assert_eq!(generated.len(),count);
        for card in generated{assert_eq!(Catalog::standard().card(card.id).unwrap().color,crate::content::CardColor::Colorless);
            if token=="transmutation"{assert_eq!(card.upgrades,upgrades);if !card.local.x_cost{assert_eq!(card.local.turn_cost,Some(0));}}
        }
        assert_eq!(c.state.player.energy,if token=="transmutation"{0}else{9});
        assert!(c.state.piles.exhaust.iter().any(|card|card.handle().as_str()=="3"));
    }
}
#[test]
fn m15_generation_magnetism_is_an_actual_pre_draw_power(){
    let mut c=fight("magnetism",1);c.step_full(&CombatAction::PlayCard{instance:3,target:None}).unwrap();
    c.step_full(&CombatAction::EndTurn).unwrap();
    assert!(c.state.piles.hand.iter().filter_map(OwnedCard::full).any(|card|matches!(card.origin,CombatCardRef::CombatLocal(_))));
}
#[test]
fn m15_generation_live_upgrades_keep_source_cost_and_exhaust_changes(){
    for (token,cost,exhaust) in [("distraction",0,true),("magnetism",1,false),("discovery",1,false)]{
        let mut card=permanent_live_card(id(token),CardInstanceId(71),0);cards::ironclad::rules::upgrade(&mut card).unwrap();
        assert_eq!(card.local.mutable_combat_cost,Some(cost),"{token}");assert_eq!(card.local.exhaust_on_finish,exhaust,"{token}");
    }
}

#[test]
fn m15_generation_potion_offers_are_real_belt_owned_choices_with_source_skip_rules(){
    use crate::content::{PotionId,RelicId};
    for potion in [PotionId::AttackPotion,PotionId::SkillPotion,PotionId::PowerPotion,PotionId::ColorlessPotion]{
        for bark in [false,true]{
            let relics=if bark{vec![RelicId::SacredBark]}else{vec![]};
            let mut context=Some(crate::items::CombatItemContext::new(relics,vec![Some(potion)]).with_m15_profile());
            let mut c=Combat::enter_with_item_context(entry("distraction",0),&mut context).unwrap();
            let rng=c.state.card_rng.clone();let snapshot=c.snapshot();let legal=c.legal_actions_full().unwrap();
            assert!(legal.contains(&CombatAction::UsePotion{slot:0,target:None}));assert_eq!(c.snapshot(),snapshot);assert_eq!(c.state.card_rng,rng);
            c.step_full(&CombatAction::UsePotion{slot:0,target:None}).unwrap();assert_eq!(c.item_context().unwrap().potions()[0],None);
            let offer=c.public_full().unwrap().card_offer.unwrap();assert_eq!(offer.cards.len(),3);
            assert_eq!(offer.optional,potion!=PotionId::ColorlessPotion);
            for id in &offer.cards{let row=Catalog::standard().card(*id).unwrap();match potion{
                PotionId::AttackPotion=>assert_eq!(row.card_type,CardType::Attack),PotionId::SkillPotion=>assert_eq!(row.card_type,CardType::Skill),PotionId::PowerPotion=>assert_eq!(row.card_type,CardType::Power),_=>assert_eq!(row.color,crate::content::CardColor::Colorless)}}
            c.step_full(&CombatAction::ChooseCardOffer{index:1}).unwrap();
            let cards=c.state.piles.hand.iter().filter_map(OwnedCard::full).filter(|card|matches!(card.origin,CombatCardRef::CombatLocal(_))).collect::<Vec<_>>();
            assert_eq!(cards.len(),if bark{2}else{1});assert!(cards.iter().all(|card|card.id==offer.cards[1]));
        }
    }
}
#[test]
fn m15_generation_capture_exhaustion_retains_payment_parent_source_and_rng_without_retry(){
    let mut c=fight("distraction",0);let stream=c.state.card_rng.as_ref().unwrap();
    c.state.card_rng=Some(rng::StreamState::new(stream.id().clone(),*stream.key(),1u128<<64).unwrap());
    assert!(c.step_full(&CombatAction::PlayCard{instance:3,target:None}).is_err());
    assert_eq!(c.state.player.energy,8);assert_eq!(c.state.in_play.as_ref().unwrap().handle().as_str(),"3");
    assert!(matches!(c.state.current.as_ref().unwrap().kind,EffectKind::Generate(_)));
    assert_eq!(c.state.current.as_ref().unwrap().source,Source::Card(card_state::handle(CardInstanceId(3))));
    let frozen=c.snapshot();assert!(c.step_full(&CombatAction::EndTurn).is_err());assert_eq!(c.snapshot(),frozen);
}

#[test]
fn m15_generation_typed_skip_retains_consumed_slot_and_never_rerolls(){
    use crate::content::PotionId;
    for potion in [PotionId::AttackPotion,PotionId::SkillPotion,PotionId::PowerPotion]{
        let mut context=Some(crate::items::CombatItemContext::new(vec![],vec![Some(potion)]).with_m15_profile());
        let mut c=Combat::enter_with_item_context(entry("distraction",0),&mut context).unwrap();
        c.step_full(&CombatAction::UsePotion{slot:0,target:None}).unwrap();
        let rng=c.state.card_rng.clone();let count=c.state.card_ids_issued.len();
        assert!(c.legal_actions_full().unwrap().contains(&CombatAction::SkipCardOffer));
        c.step_full(&CombatAction::SkipCardOffer).unwrap();
        assert_eq!(c.state.phase,Phase::Decision);assert_eq!(c.state.card_rng,rng);assert_eq!(c.state.card_ids_issued.len(),count);
        assert_eq!(c.item_context().unwrap().potions()[0],None);
        let frozen=c.snapshot();assert!(c.step_full(&CombatAction::SkipCardOffer).is_err());assert_eq!(c.snapshot(),frozen);
    }
}
#[test]
fn m15_generation_source_support_tokens_and_order_match_retained_constructor_facts(){
    let expected=[(GenerationPool::Attack,"bane,dagger_spray,dagger_throw,flying_knee,poisoned_stab,quick_slash,slice,sneaky_strike,sucker_punch,all_out_attack,backstab,choke,dash,endless_agony,eviscerate,finisher,flechettes,heel_hook,masterful_stab,predator,riddle_with_holes,skewer,die_die_die,glass_knife,grand_finale,unload"),
    (GenerationPool::Skill,"acrobatics,backflip,blade_dance,cloak_and_dagger,deadly_poison,deflect,dodge_and_roll,outmaneuver,piercing_wail,prepared,blur,bouncing_flask,calculated_gamble,catalyst,concentrate,crippling_cloud,distraction,escape_plan,expertise,leg_sweep,reflex,setup,tactician,terror,adrenaline,bullet_time,burst,corpse_explosion,doppelganger,malaise,nightmare,phantasmal_killer,storm_of_steel"),
    (GenerationPool::Power,"accuracy,caltrops,footwork,infinite_blades,noxious_fumes,well_laid_plans,a_thousand_cuts,after_image,envenom,tools_of_the_trade,wraith_form")];
    for (kind,tokens) in expected{let actual=generation::pool(Some(Character::Silent),kind).unwrap();assert_eq!(actual.iter().map(|id|id.as_str()).collect::<Vec<_>>(),tokens.split(',').collect::<Vec<_>>());}
}

#[test]
fn m15_generation_later_delivery_fault_keeps_sampled_prefix_and_first_child(){
    let mut c=fight("jack_of_all_trades",1);
    for instance in 4..=998{c.state.card_ids_issued.insert(card_state::handle(CardInstanceId(instance)));c.state.piles.discard.push(OwnedCard::Full(permanent_live_card(CardId::DefendGreen,CardInstanceId(instance),0)));}
    assert!(c.step_full(&CombatAction::PlayCard{instance:3,target:None}).is_err());
    let EffectKind::Generate(work)=&c.state.current.as_ref().unwrap().kind else{panic!("retained generator current")};
    assert_eq!(work.cards.len(),2);assert_eq!(work.next,1);assert!(work.captured);
    assert_eq!(c.state.piles.hand.iter().filter_map(OwnedCard::full).filter(|card|matches!(card.origin,CombatCardRef::CombatLocal(_))).count(),1);
    let frozen=c.snapshot();assert!(c.step_full(&CombatAction::EndTurn).is_err());assert_eq!(c.snapshot(),frozen);
}

#[test]
fn m15_generation_use_capture_precedes_random_on_use_block_reactions(){
    let mut setup=entry("distraction",0);
    for (id,amount) in [("after_image",1),("juggernaut",5)]{setup.setup.player_powers.push(FullPowerEntry{id:FullPowerId(id),state:PowerState{amount,skip_first:false,monster_source_grace:false}});}
    let mut c=Combat::enter(setup).unwrap();
    let support=generation::pool(Some(Character::Silent),GenerationPool::Skill).unwrap();
    let (index,_)=rng::uniform_below(c.state.card_rng.as_ref().unwrap(),support.len() as u128).unwrap();
    c.step_full(&CombatAction::PlayCard{instance:3,target:None}).unwrap();
    let child=c.state.piles.hand.iter().filter_map(OwnedCard::full).find(|card|matches!(card.origin,CombatCardRef::CombatLocal(_))).unwrap();
    assert_eq!(child.id,support[index as usize]);
}
#[test]
fn m15_generation_mayhem_child_uses_actual_character_and_shared_generator_owner(){
    let mut context=Some(crate::items::CombatItemContext::new(vec![],vec![]).with_m15_profile());
    let mut c=Combat::enter_with_item_context(entry("mayhem",0),&mut context).unwrap();
    c.step_full(&CombatAction::PlayCard{instance:3,target:None}).unwrap();
    let card=permanent_live_card(id("distraction"),CardInstanceId(70),0);
    c.state.card_ids_issued.insert(card_state::handle(card.instance));c.state.piles.draw.push(OwnedCard::Full(card));
    c.step_full(&CombatAction::EndTurn).unwrap();
    assert!(c.state.piles.exhaust.iter().any(|card|card.handle().as_str()=="70"));
    assert!(c.state.piles.hand.iter().chain(c.state.piles.discard.iter()).filter_map(OwnedCard::full).any(|card|matches!(card.origin,CombatCardRef::CombatLocal(_))));
    assert!(c.state.play_frames.is_empty());
}

#[test]
fn m15_generation_every_l34_kind_dispatches_actual_base_and_upgraded_use(){
    for id in generation::pool(Some(Character::Silent),GenerationPool::Colorless).unwrap(){
        for upgrades in [0,1]{
            let mut c=fight(id.as_str(),upgrades);
            // Secret retrieve cards require a matching real draw-pile kind.
            for (kind,instance) in [(CardId::DefendGreen,71),(CardId::StrikeGreen,72)]{let card=permanent_live_card(kind,CardInstanceId(instance),0);c.state.card_ids_issued.insert(card_state::handle(card.instance));c.state.piles.draw.push(OwnedCard::Full(card));}
            piles::refresh_hand_values(&mut c.state);
            let action=c.legal_actions_full().unwrap().into_iter().find(|action|matches!(action,CombatAction::PlayCard{instance:3,..})).unwrap_or_else(||panic!("L34 kind {id:?}+{upgrades} has no actual play handler"));
            c.step_full(&action).unwrap_or_else(|e|panic!("{id:?}+{upgrades}: {e:?}"));
            for _ in 0..12{
                if c.state.phase!=Phase::Selection{break}
                let actions=c.legal_actions_full().unwrap();
                let action=actions.iter().find(|action|matches!(action,CombatAction::ConfirmSelection)).or_else(||actions.iter().find(|action|matches!(action,CombatAction::ChooseCardOffer{..}|CombatAction::SelectCard{..}))).unwrap();
                c.step_full(action).unwrap_or_else(|e|panic!("{id:?}+{upgrades} selection: {e:?}"));
            }
            assert_ne!(c.state.phase,Phase::Selection,"{id:?}+{upgrades} retained unexpected selection");
            assert!(c.state.fault.is_none(),"{id:?}+{upgrades}");
        }
    }
}

#[test]
fn m15_generation_new_shared_potions_do_not_widen_older_inventory_profiles(){
    use crate::content::PotionId;
    for potion in [PotionId::AttackPotion,PotionId::SmokeBomb,PotionId::DuplicationPotion]{
        let mut context=Some(crate::items::CombatItemContext::new(vec![],vec![Some(potion)]));let before=context.clone();
        assert!(matches!(Combat::enter_with_item_context(entry("distraction",0),&mut context),Err(CombatEntryFailure::Rejected(crate::run::RunError::Unsupported(_)))));
        assert_eq!(context,before);
    }
}

#[test]
fn m15_generation_preserves_accepted_default_item_and_old_card_debug_bytes(){
    let context=crate::items::CombatItemContext::new(vec![],vec![]);
    assert_eq!(format!("{context:?}"),"CombatItemContext { battle: NativeBattleContext { elite_trigger: false, boss: false, burning: None }, collar_activated: false, counters: {}, relics: [], potions: [], orichalcum_trigger: false, lantern_armed: false, heal_room: NativeHealRoom { combat: false, current_map_node: false } }");
    let mut c=super::full_runtime::entered_combat(3,3,|_|{});
    c.step_full(&CombatAction::PlayCard{instance:1,target:Some(TargetId(1))}).unwrap();
    assert!(!format!("{c:?}").contains("silent:"));
}

#[test]
fn m15_generation_power_upgrade_cost_writers_preserve_live_turn_delta(){
    for (token,price) in [("tools_of_the_trade",1),("phantasmal_killer",1),("envenom",2)]{
        let mut c=fight(token,0);
        let card=c.state.piles.hand.iter_mut().find(|card|card.handle().as_str()=="3").unwrap().full_mut().unwrap();
        card.local.randomized_cost=Some(2);card.local.turn_cost=Some(3);
        cards::ironclad::rules::upgrade(card).unwrap();
        c.step_full(&CombatAction::PlayCard{instance:3,target:None}).unwrap();
        assert_eq!(c.state.player.energy,9-price,"{token} source upgrade cost plus existing positive turn delta");
    }
}

#[test]
fn m15_generation_alchemize_uses_real_factory_obtain_and_actual_card_payment(){
    use crate::content::{PotionId,RelicId};
    for upgrades in [0,1]{for mode in 0..3{
        let relics=if mode==1{vec![RelicId::Sozu]}else{vec![]};
        let potions=if mode==2{vec![Some(PotionId::StrengthPotion);3]}else{vec![None;3]};
        let mut context=Some(crate::items::CombatItemContext::new(relics,potions.clone()).with_m15_profile());
        let mut c=Combat::enter_with_item_context(entry("alchemize",upgrades),&mut context).unwrap();let rng=c.state.card_rng.clone();
        c.step_full(&CombatAction::PlayCard{instance:3,target:None}).unwrap();
        assert_ne!(c.state.card_rng,rng,"even Sozu/full capacity retains source sampling");
        assert_eq!(c.state.player.energy,if upgrades==0{8}else{9});
        assert!(c.state.piles.exhaust.iter().any(|card|card.handle().as_str()=="3"));
        let after=c.item_context().unwrap().potions();
        if mode==0{assert_eq!(after.iter().flatten().count(),1);assert_ne!(after[0],Some(PotionId::FruitJuice));}
        else{assert_eq!(after,potions.as_slice());}
    }}
}

fn finish_choices(c:&mut Combat){
    for _ in 0..12{if c.state.phase!=Phase::Selection{return}
        let actions=c.legal_actions_full().unwrap();
        let action=actions.iter().find(|action|matches!(action,CombatAction::ConfirmSelection)).or_else(||actions.iter().find(|action|matches!(action,CombatAction::ChooseCardOffer{..}|CombatAction::SelectCard{..}))).unwrap();
        c.step_full(action).unwrap();
    }
    panic!("diagnostic fixture did not finish its finite choices")
}
#[test]
fn m15_generation_all_75_green_kinds_have_actual_base_and_upgraded_dispatch(){
    let ids=Catalog::standard().cards().iter().filter(|row|row.color==crate::content::CardColor::Green).map(|row|row.id).collect::<Vec<_>>();assert_eq!(ids.len(),75);
    for id in ids{for upgrades in [0,1]{
        assert!(!Catalog::standard().card(id).unwrap().definition_only,"{id:?}");
        let mut context=Some(crate::items::CombatItemContext::new(vec![],vec![None;3]).with_m15_profile());
        let mut c=Combat::enter_with_item_context(entry(id.as_str(),upgrades),&mut context).unwrap();
        let action=c.legal_actions_full().unwrap().into_iter().find(|action|matches!(action,CombatAction::PlayCard{instance:3,..}));
        if matches!(id,CardId::Reflex|CardId::Tactician){assert!(action.is_none());continue}
        c.step_full(&action.unwrap_or_else(||panic!("missing actual play {id:?}+{upgrades}"))).unwrap_or_else(|e|panic!("{id:?}+{upgrades}: {e:?}"));
        finish_choices(&mut c);assert!(c.state.fault.is_none(),"{id:?}+{upgrades}");
    }}
}
#[test]
fn m15_generation_all_33_potion_kinds_use_actual_handlers_and_real_belt_custody(){
    for potion in crate::items::potion_generation::candidates(Character::Silent,false).unwrap(){
        let mut context=Some(crate::items::CombatItemContext::new(vec![],vec![Some(*potion),None,None]).with_m15_profile());
        let mut c=Combat::enter_with_item_context(entry("defend_g",0),&mut context).unwrap();
        let action=c.legal_actions_full().unwrap().into_iter().find(|action|matches!(action,CombatAction::UsePotion{slot:0,..}));
        if *potion==crate::content::PotionId::FairyPotion{assert!(action.is_none());continue}
        c.step_full(&action.unwrap_or_else(||panic!("missing actual potion handler {potion:?}"))).unwrap_or_else(|e|panic!("{potion:?}: {e:?}"));
        finish_choices(&mut c);assert!(c.state.fault.is_none(),"{potion:?}");
        if *potion!=crate::content::PotionId::EntropicBrew{assert_eq!(c.item_context().unwrap().potions()[0],None,"{potion:?}");}
    }
}
#[test]
fn m15_generation_burst_repeats_real_alchemize_through_the_same_factory_and_obtain_owner(){
    let mut context=Some(crate::items::CombatItemContext::new(vec![],vec![None;3]).with_m15_profile());
    let mut c=Combat::enter_with_item_context(entry("burst",0),&mut context).unwrap();
    let card=permanent_live_card(CardId::Alchemize,CardInstanceId(70),0);c.state.card_ids_issued.insert(card_state::handle(card.instance));c.state.piles.hand.push(OwnedCard::Full(card));
    c.step_full(&CombatAction::PlayCard{instance:3,target:None}).unwrap();
    c.step_full(&CombatAction::PlayCard{instance:70,target:None}).unwrap();
    assert_eq!(c.state.player.energy,7);assert_eq!(c.item_context().unwrap().potions().iter().flatten().count(),2);
    assert!(c.state.play_frames.is_empty());assert!(c.state.queued_autoplay.is_empty());
}
