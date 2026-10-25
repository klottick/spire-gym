#[path="common/native.rs"] mod native;
use spire_core::{run::*,combat::*,content::PotionId};
use spire_report::*;
use serde_json::Value;

fn align_public_deck(decision: &mut PublicDecision, cards: &[LiveCard]) {
    decision.objects = cards.iter().map(|card| PublicObject {
        instance: card.instance.0,
        id: card.id,
        upgrades: card.upgrades,
    }).collect();
}

#[test]
fn run_snapshot_current_owner() {
    let mut run=native::lethal();
    run.step(&GameAction::Combat(CombatAction::UsePotion{slot:0,target:Some(TargetId(1))})).unwrap();
    let d=run.decision().unwrap();let c=d.combat.as_ref().unwrap();
    let state=run_snapshot(&d,run.public_map().as_ref()).unwrap();
    assert_eq!(state["player"]["hp"],c.player.hp);assert_eq!(state["player"]["energy"],c.player.energy);
    assert_eq!(state["inventory"]["potions"].as_array().unwrap().len(),3);assert!(state["inventory"]["potions"][0].is_null());
    assert_eq!(state["combat"]["hand"][0]["origin"]["instance"],"1");
    assert_eq!(state["combat"]["hand"][0]["currentCost"],c.hand[0].effective_cost.unwrap());
}
#[test]
fn run_option_namespace() {
    let mut d=native::m4::run(7).decision().unwrap();
    let use_action=GameAction::UsePotion{slot:0,target:None};let discard=GameAction::DiscardPotion{slot:0};
    d.player.potions[0]=Some(PotionId::BloodPotion);
    d.candidates=vec![Candidate{id:ChoiceId(42),action:use_action.clone(),label:"Use",neow_offer:None,object:None},Candidate{id:ChoiceId(42),action:discard.clone(),label:"Discard",neow_offer:None,object:None}];
    let a=describe_run_action(&d,&use_action,"combat-0").unwrap();let b=describe_run_action(&d,&discard,"combat-0").unwrap();
    assert_ne!(a["optionKey"],b["optionKey"]);
    assert_eq!(a["preActionOffer"]["potionId"],"blood_potion");
}
#[test]
fn run_target_lifetimes() {
    let mut d=native::lethal().decision().unwrap();let c=d.combat.as_mut().unwrap();
    let mut dead=c.enemies[0].clone();dead.hp=0;dead.instance_id=EnemyHandle::parse("dead-first").unwrap();c.enemies.insert(0,dead);
    c.targets=vec![(TargetId(99),false),(TargetId(73),true)];
    let action=GameAction::Combat(CombatAction::UsePotion{slot:0,target:Some(TargetId(73))});
    d.candidates=vec![Candidate{id:ChoiceId(0),action:action.clone(),label:"Potion",neow_offer:None,object:None}];
    let described=describe_run_action(&d,&action,"combat-9").unwrap();
    assert!(described["targets"][0].as_str().unwrap().ends_with(d.combat.as_ref().unwrap().enemies[1].instance_id.as_str()));
    d.combat.as_mut().unwrap().targets.pop();assert!(describe_run_action(&d,&action,"combat-9").is_err());
}

#[test]
fn run_snapshot_keeps_half_dead_distinct_from_permanent_defeat() {
    let mut d=native::lethal().decision().unwrap();d.candidates.clear();let c=d.combat.as_mut().unwrap();
    c.enemies[0].hp=0;c.enemies[0].half_dead=true;c.targets[0].1=false;
    let state=run_snapshot(&d,None).unwrap();
    assert_eq!(state["combat"]["enemies"][0]["lifeState"],"half_dead");
    assert_eq!(state["combat"]["enemies"][0]["targetEligible"],false);
}

#[test]
fn newer_public_room_variants_are_named_without_inventing_completion() {
    let mut d=native::m4::run(7).decision().unwrap();d.candidates.clear();
    for (kind,want) in [(RoomViewKind::Act3Ready,"act3_ready"),(RoomViewKind::Act3PrefixComplete,"act3_prefix_complete")] {
        d.room=Some(RoomView{kind,choices:vec![]});let state=run_snapshot(&d,None).unwrap();
        assert_eq!(state["screen"]["kind"],want);assert!(state["terminal"].is_null());
    }
    d.room=Some(RoomView{kind:RoomViewKind::Event(spire_core::content::EventId::Falling),choices:vec![
        RoomChoiceView{id:ChoiceId(1),kind:RoomChoiceKind::FallingLose{card_type:spire_core::content::CardType::Attack,instance:Some(42)},status:RoomChoiceStatus::Offered,price:None},
        RoomChoiceView{id:ChoiceId(2),kind:RoomChoiceKind::TombGold,status:RoomChoiceStatus::Offered,price:None},
        RoomChoiceView{id:ChoiceId(3),kind:RoomChoiceKind::TombOfferGold,status:RoomChoiceStatus::Disabled,price:Some(99)},
    ]});
    let state=run_snapshot(&d,None).unwrap();let options=state["screen"]["options"].as_array().unwrap();
    assert_eq!(options[0]["data"]["instance"],"42");assert_eq!(options[0]["data"]["cardType"],"attack");
    assert_eq!(options[1]["label"],"Take gold");assert_eq!(options[2]["label"],"Offer gold");
    assert_eq!(options[2]["status"],"disabled");assert_eq!(options[2]["data"]["price"],99);
}
#[test]
fn run_public_map() {
    let mut run=native::m4::run(7);
    let action=run.decision().unwrap().candidates[0].action.clone();run.step(&action).unwrap();
    let map=run.public_map().unwrap();
    let state=run_snapshot(&run.decision().unwrap(),Some(&map)).unwrap();
    assert_eq!(state["map"]["nodes"].as_array().unwrap().len(),map.nodes.len());
    assert!(state["map"]["edges"].is_array());
    for n in state["map"]["nodes"].as_array().unwrap() {if n["kind"]=="?" {assert_eq!(n["resolvedRoom"],Value::Null);}}
    assert!(!state.to_string().contains("encounter_history"));
}

#[test]
fn m9_report_rooms_do_not_invent_a_terminal_or_next_act() {
    // Public display diagnostics, not a supplied-state victory or reset API.
    let mut d=native::m4::run(7).decision().unwrap();d.candidates.clear();
    d.act=3;d.floor=50;let hp=d.player.hp;
    for (kind,want) in [(RoomViewKind::Act3BossComplete,"act3_boss_complete"),
        (RoomViewKind::Act3Ending,"act3_ending"),(RoomViewKind::Act4Ready,"act4_ready")] {
        d.room=Some(RoomView{kind,choices:vec![]});let state=run_snapshot(&d,None).unwrap();
        assert_eq!(state["screen"]["kind"],want);assert!(state["terminal"].is_null());
        assert_eq!(state["player"]["hp"],hp);assert!(state["screen"]["options"].as_array().unwrap().is_empty());
    }
    d.terminal=Some(RunTerminal::Act3NoKeys);
    assert_eq!(run_snapshot(&d,None).unwrap()["terminal"],"act3_no_keys");
}

#[test]
fn m9_report_heart_choices_preserve_native_stage_and_action() {
    let mut d=native::m4::run(7).decision().unwrap();
    for stage in 1..=4 {
        let action=GameAction::Choose(ChoiceId(42));
        d.candidates=vec![Candidate{id:ChoiceId(42),action:action.clone(),label:"Continue",object:None,neow_offer:None}];
        d.room=Some(RoomView{kind:RoomViewKind::Act3Ending,choices:vec![RoomChoiceView{
            id:ChoiceId(42),kind:RoomChoiceKind::Act3Continue{stage},status:RoomChoiceStatus::Offered,price:None,
        }]});
        let snapshot=run_snapshot(&d,None).unwrap();let row=&snapshot["screen"]["options"][0];
        let described=describe_run_action(&d,&action,"combat-0").unwrap();
        assert_eq!(row["label"],"Continue");assert_eq!(row["status"],"offered");
        assert_eq!(row["data"]["stage"],stage);assert_eq!(row["data"]["choiceId"],"42");
        assert_eq!(described["optionKey"],row["key"]);assert_eq!(described["preActionOffer"],row["data"]);
        assert!(snapshot["terminal"].is_null());
    }
}

#[test]
fn m10_report_projects_card_values_facing_and_qualified_intent_hits() {
    use spire_core::{combat::{self, Combat},content::{CardId,Character,EncounterId,RelicId},items::CombatItemContext};
    let mut profile=CreationProfile::experienced();profile.neow_eligible=false;
    let run=Run::new_room_scenario(RunConfig{character:Character::Ironclad,ascension:0},profile,
        RunRng::new([91;32],combat::rng::Partition::Test,0),RoomScenario::M10IroncladHeart).unwrap();
    let mut decision=run.decision().unwrap();
    let cards=(1..=10).map(|id|combat::permanent_live_card(CardId::Strike,combat::CardInstanceId(id),0)).collect::<Vec<_>>();
    align_public_deck(&mut decision,&cards);
    // Reuse the existing native Shield/Spear characterization seed; do not
    // replace the intents produced and published by the encounter engine.
    let setup=combat::enemies::setup(EncounterId::ShieldAndSpear,4,0,&mut RunRng::new([19;32],combat::rng::Partition::Test,0),combat::enemies::SetupContext{
        player:combat::PlayerState{hp:80,max_hp:80,block:0,energy:3,max_energy:3,draw_per_turn:5,powers:vec![]},player_powers:vec![],character:Character::Ironclad,
        cards,opening_draw:5,encounter_visit:0}).unwrap();
    let mut items=Some(CombatItemContext::new(vec![RelicId::BurningBlood],vec![None,None,None]));
    let combat=Combat::enter_with_item_context(setup,&mut items).unwrap();
    let public=combat.public_full().unwrap();
    assert_eq!(public.enemies.iter().map(|enemy|enemy.intent.as_str()).collect::<Vec<_>>(),["shield_fortify","spear_burn_strike"]);
    decision.combat=Some(public);
    decision.room=None;decision.candidates.clear();

    let state=run_snapshot(&decision,None).unwrap();
    let strike=&state["combat"]["hand"][0];
    assert_eq!(strike["baseDamage"],6);
    assert_eq!(strike["baseBlock"],Value::Null,"non-applicable values remain null, not known zero");
    assert_eq!(strike["costKind"],"fixed");
    assert_eq!(state["combat"]["facing"]["player"],"left");
    assert_eq!(state["combat"]["facing"]["surrounded"],true);
    assert_eq!(state["combat"]["enemies"][0]["side"],"left");
    assert_eq!(state["combat"]["enemies"][0]["intent"]["hitCount"],Value::Null,"the actual Fortify intent has no qualified attack hit count");
    assert_eq!(state["combat"]["enemies"][1]["side"],"right");
    assert_eq!(state["combat"]["enemies"][1]["intent"]["hitCount"],2);
    assert_eq!(state["combat"]["facing"]["enemies"][1]["backAttack"],true);
}

fn m10_headbutt_draw_snapshot(opening_draw: usize) -> Value {
    use spire_core::{combat::{self, Combat, CombatAction}, content::{CardId, Character, EncounterId, RelicId}, items::CombatItemContext};
    let mut profile=CreationProfile::experienced();profile.neow_eligible=false;
    let run=Run::new_room_scenario(RunConfig{character:Character::Ironclad,ascension:0},profile,
        RunRng::new([97;32],combat::rng::Partition::Test,0),RoomScenario::M10IroncladHeart).unwrap();
    let mut decision=run.decision().unwrap();
    let cards=std::iter::once(combat::permanent_live_card(CardId::Headbutt,combat::CardInstanceId(1),0))
        .chain((2..=6).map(|instance|combat::permanent_live_card(CardId::Strike,combat::CardInstanceId(instance),0))).collect::<Vec<_>>();
    align_public_deck(&mut decision,&cards);
    let setup=combat::enemies::setup(EncounterId::CorruptHeart,4,0,&mut RunRng::new([98;32],combat::rng::Partition::Test,0),combat::enemies::SetupContext{
        player:combat::PlayerState{hp:80,max_hp:80,block:0,energy:3,max_energy:3,draw_per_turn:5,powers:vec![]},player_powers:vec![],character:Character::Ironclad,
        cards,opening_draw,encounter_visit:0}).unwrap();
    let mut items=Some(CombatItemContext::new(vec![RelicId::BurningBlood],vec![None,None,None]));
    let mut native=Combat::enter_with_item_context(setup,&mut items).unwrap();
    let hand=native.public_full().unwrap().hand;
    let headbutt=hand.iter().find(|card|card.id==CardId::Headbutt).expect("Headbutt is in the actual opening hand").instance.0;
    let strike=hand.iter().find(|card|card.id==CardId::Strike).expect("a real Strike can seed Headbutt's discard choice").instance.0;
    native.step_full(&CombatAction::PlayCard{instance:strike,target:Some(TargetId(1))}).unwrap();
    native.step_full(&CombatAction::PlayCard{instance:headbutt,target:Some(TargetId(1))}).unwrap();
    let public=native.public_full().unwrap();
    decision.combat=Some(public);
    decision.room=None;decision.candidates.clear();decision.offers.clear();decision.selection=None;
    run_snapshot(&decision,None).unwrap()
}

#[test]
fn m10_report_marks_partial_headbutt_prefix_as_not_full_draw_order() {
    let partial=m10_headbutt_draw_snapshot(5);
    let draw=&partial["combat"]["piles"]["draw"];
    assert_eq!(draw["count"],2,"the original hidden draw card remains behind the native Headbutt topdeck");
    assert_eq!(draw["knownCards"].as_array().unwrap().len(),1,"only the publicly topdecked card is listed");
    assert_eq!(draw["knownCards"][0]["fromTop"],true);
    assert_eq!(draw["knownCards"][0]["offset"],0);
    assert_eq!(draw["orderKnown"],false,"a known prefix does not disclose the remaining draw order");

    let complete=m10_headbutt_draw_snapshot(6);
    let draw=&complete["combat"]["piles"]["draw"];
    assert_eq!(draw["count"],1);
    assert_eq!(draw["knownCards"].as_array().unwrap().len(),2,"the sole card is entitled at both known endpoints");
    assert_eq!(draw["knownCards"][0]["fromTop"],true);
    assert_eq!(draw["knownCards"][1]["fromTop"],false);
    assert_eq!(draw["knownCards"][0]["card"]["origin"],draw["knownCards"][1]["card"]["origin"]);
    assert_eq!(draw["orderKnown"],true,"the public top and bottom facts cover the complete one-card order");
}

#[test]
fn m10_report_discloses_confirm_selection_purpose_and_true_victory_room() {
    use spire_core::{combat::{self,Combat,CombatAction},content::{CardId,Character,EncounterId,PotionId,RelicId},items::CombatItemContext};
    let mut profile=CreationProfile::experienced();profile.neow_eligible=false;
    let run=Run::new_room_scenario(RunConfig{character:Character::Ironclad,ascension:0},profile,
        RunRng::new([93;32],combat::rng::Partition::Test,0),RoomScenario::M10IroncladHeart).unwrap();
    let mut decision=run.decision().unwrap();
    let cards=(1..=5).map(|instance|combat::permanent_live_card(CardId::Strike,combat::CardInstanceId(instance),0)).collect::<Vec<_>>();
    align_public_deck(&mut decision,&cards);
    let setup=combat::enemies::setup(EncounterId::CorruptHeart,4,0,&mut RunRng::new([94;32],combat::rng::Partition::Test,0),combat::enemies::SetupContext{
        player:combat::PlayerState{hp:80,max_hp:80,block:0,energy:3,max_energy:3,draw_per_turn:5,powers:vec![]},player_powers:vec![],character:Character::Ironclad,
        cards,opening_draw:5,encounter_visit:0}).unwrap();
    let mut items=Some(CombatItemContext::new(vec![RelicId::BurningBlood],vec![Some(PotionId::Elixir),None,None]));
    let mut native=Combat::enter_with_item_context(setup,&mut items).unwrap();
    native.step_full(&CombatAction::UsePotion{slot:0,target:None}).unwrap();
    let legal=native.legal_actions_full().unwrap();
    let confirm=CombatAction::ConfirmSelection;
    assert!(legal.contains(&confirm),"ConfirmSelection must come from the native optional Elixir menu");
    let public=native.public_full().unwrap();
    let selection=public.selection.as_ref().expect("the owned Elixir use starts a public selector");
    assert_eq!(selection.minimum,0,"the actual Elixir selector permits zero picks");
    assert!(selection.maximum>selection.minimum);
    let selection_maximum=selection.maximum;
    decision.combat=Some(public);
    decision.room=None;
    decision.candidates=legal.into_iter().enumerate().map(|(index,action)|Candidate{
        id:ChoiceId(400+index as u64),action:GameAction::Combat(action),label:"native legal combat action",neow_offer:None,object:None}).collect();
    let action=GameAction::Combat(confirm);
    assert!(decision.candidates.iter().any(|candidate|candidate.action==action),"the described action is in the actual native menu");
    let snapshot=run_snapshot(&decision,None).unwrap();
    assert_eq!(snapshot["combat"]["selection"]["purpose"],"move_exhaust");
    assert_eq!(snapshot["combat"]["selection"]["minimum"],0);
    assert_eq!(snapshot["combat"]["selection"]["maximum"],selection_maximum);
    assert!(snapshot["screen"]["options"].as_array().unwrap().iter().any(|option|option["key"]=="combat/confirm_selection"));
    let described=describe_run_action(&decision,&action,"combat-0").unwrap();
    assert_eq!(described["type"],"confirm_selection");
    assert_eq!(described["selectionPurpose"],"move_exhaust");

    decision.combat=None;decision.candidates.clear();decision.terminal=None;
    for (kind,label) in [(RoomViewKind::HeartVictory,"heart_victory"),(RoomViewKind::TrueVictory,"true_victory")] {
        decision.room=Some(RoomView{kind,choices:vec![]});
        let state=run_snapshot(&decision,None).unwrap();
        assert_eq!(state["screen"]["kind"],label);
        assert!(state["terminal"].is_null(),"a Heart room label alone is not a terminal settlement");
    }
    decision.room=Some(RoomView{kind:RoomViewKind::TrueVictory,choices:vec![]});decision.terminal=Some(RunTerminal::HeartVictory);
    assert_eq!(run_snapshot(&decision,None).unwrap()["terminal"],"heart_victory");
}

#[test]
fn m10_report_keeps_dual_wield_copy_purpose_without_synthetic_confirmation() {
    use spire_core::{combat::{self,Combat,CombatAction},content::{CardId,Character,EncounterId,RelicId},items::CombatItemContext};
    let mut profile=CreationProfile::experienced();profile.neow_eligible=false;
    let run=Run::new_room_scenario(RunConfig{character:Character::Ironclad,ascension:0},profile,
        RunRng::new([95;32],combat::rng::Partition::Test,0),RoomScenario::M10IroncladHeart).unwrap();
    let mut decision=run.decision().unwrap();
    let cards=std::iter::once(combat::permanent_live_card(CardId::DualWield,combat::CardInstanceId(1),0))
        .chain((2..=5).map(|instance|combat::permanent_live_card(CardId::Strike,combat::CardInstanceId(instance),0))).collect::<Vec<_>>();
    align_public_deck(&mut decision,&cards);
    let setup=combat::enemies::setup(EncounterId::CorruptHeart,4,0,&mut RunRng::new([96;32],combat::rng::Partition::Test,0),combat::enemies::SetupContext{
        player:combat::PlayerState{hp:80,max_hp:80,block:0,energy:3,max_energy:3,draw_per_turn:5,powers:vec![]},player_powers:vec![],character:Character::Ironclad,
        cards,opening_draw:5,encounter_visit:0}).unwrap();
    let mut items=Some(CombatItemContext::new(vec![RelicId::BurningBlood],vec![None,None,None]));
    let mut native=Combat::enter_with_item_context(setup,&mut items).unwrap();
    let parent=native.public_full().unwrap().hand.iter().find(|card|card.id==CardId::DualWield).unwrap().instance.0;
    native.step_full(&CombatAction::PlayCard{instance:parent,target:None}).unwrap();
    let legal=native.legal_actions_full().unwrap();
    assert!(!legal.contains(&CombatAction::ConfirmSelection),"fixed-size Dual Wield does not offer confirmation");
    let public=native.public_full().unwrap();
    let selection=public.selection.as_ref().expect("Dual Wield starts its real copy selector");
    assert_eq!(selection.operation,spire_core::combat::SelectionOperation::CopyToHand{copies:2});
    assert_eq!((selection.minimum,selection.maximum),(1,1));
    decision.combat=Some(public);
    decision.room=None;
    decision.candidates=legal.into_iter().enumerate().map(|(index,action)|Candidate{
        id:ChoiceId(500+index as u64),action:GameAction::Combat(action),label:"native legal combat action",neow_offer:None,object:None}).collect();
    let snapshot=run_snapshot(&decision,None).unwrap();
    assert_eq!(snapshot["combat"]["selection"]["purpose"],"copy_to_hand_2");
    assert_eq!(snapshot["combat"]["selection"]["minimum"],1);
    assert_eq!(snapshot["combat"]["selection"]["maximum"],1);
}
