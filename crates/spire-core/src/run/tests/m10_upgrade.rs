use crate::{run::*,content::{CardId,Character,EncounterId,PotionId},combat::{CombatAction,TargetId}};

// Break caught: comparing combat-local upgraded counts to permanent descriptors
// on settlement either rejects a valid fight or persists a temporary upgrade.
#[test]
fn m10_upgrade_combat_settlement_keeps_master_deck_unchanged(){
    let deck=vec![CardInstance{instance:1,id:CardId::SearingBlow,upgrades:0},
        CardInstance{instance:2,id:CardId::Strike,upgrades:0},CardInstance{instance:3,id:CardId::Armaments,upgrades:1}];
    let start=AuthoredRunStart{hp:80,max_hp:80,gold:99,deck:(1..=3).map(|instance|CardInstance{instance,id:CardId::Strike,upgrades:0}).collect(),relics:vec![],
        potions:vec![Some(PotionId::FirePotion),None,None],relic_counters:Default::default()};
    let mut r=Run::from_authored_combat(RunConfig{character:Character::Ironclad,ascension:0},start,
        RunRng::new([7;32],crate::combat::rng::Partition::Test,0)).unwrap();
    // Privileged supplied bridge diagnostic, not a widened M3 reset contract.
    r.state.deck=deck.clone();
    r.enter_combat(EncounterId::JawWorm).unwrap();
    for turn in 0..2 {
        if turn>0{r.choose_combat(CombatAction::EndTurn).unwrap();}
        r.choose_combat(CombatAction::PlayCard{instance:3,target:None}).unwrap();
        r.choose_combat(CombatAction::PlayCard{instance:1,target:Some(TargetId(1))}).unwrap();
        assert_eq!(r.state.deck,deck);
    }
    r.choose_combat(CombatAction::UsePotion{slot:0,target:Some(TargetId(1))}).unwrap();
    assert_eq!(r.state.deck,deck);
    assert!(r.state.combat.is_none());
    assert_eq!(r.state.combat_progress.settled,1);
    r.enter_combat(EncounterId::Cultist).unwrap();
    let view=r.state.combat.as_ref().unwrap().public_full().unwrap();
    assert_eq!(view.hand.iter().find(|c|c.instance.0==1).unwrap().upgrades,0);
}

#[test]
fn m10_upgrade_smith_accepts_repeat_searing_blow_but_not_upgraded_strike(){
    let mut r=super::room_services::room(38);
    for c in &mut r.state.deck{c.upgrades=1;}
    r.state.deck[0].id=CardId::SearingBlow;
    let smith=r.decision().unwrap().room.unwrap().choices.into_iter().find(|c|c.kind==RoomChoiceKind::Smith).unwrap();
    assert_eq!(smith.status,RoomChoiceStatus::Offered);
    r.step(&GameAction::Choose(smith.id)).unwrap();
    let d=r.decision().unwrap();
    assert_eq!(d.selection.as_ref().unwrap().eligible.len(),1);
    let selected=d.candidates.iter().find(|c|c.object.as_ref().is_some_and(|o|o.instance==1)).unwrap().action.clone();
    r.step(&selected).unwrap();
    assert_eq!(r.state.deck[0].upgrades,2);
    assert!(r.state.deck[1..].iter().all(|c|c.upgrades==1));
}

#[test]
fn m10_armaments_upgrades_fully_discounted_blood_for_blood_without_invalidating_cost_state(){
    let initial=AuthoredRunStart{hp:80,max_hp:80,gold:99,
        deck:vec![CardInstance{instance:1,id:CardId::Strike,upgrades:0},CardInstance{instance:2,id:CardId::Strike,upgrades:0}],
        relics:vec![],potions:vec![None;3],relic_counters:Default::default()};
    let mut run=Run::from_authored_combat(RunConfig{character:Character::Ironclad,ascension:0},initial,
        RunRng::new([7;32],crate::combat::rng::Partition::Test,0)).unwrap();
    let permanent_deck=vec![CardInstance{instance:1,id:CardId::BloodForBlood,upgrades:0},
        CardInstance{instance:2,id:CardId::Armaments,upgrades:1}];
    run.state.deck=permanent_deck.clone();
    run.enter_combat(EncounterId::JawWorm).unwrap();

    for _ in 0..16 {
        let decision=run.decision().unwrap();
        let combat=decision.combat.expect("bounded Jaw Worm combat remains active");
        let bfb=combat.hand.iter().find(|card|card.instance.0==1&&card.id==CardId::BloodForBlood);
        let armaments=combat.hand.iter().find(|card|card.instance.0==2&&card.id==CardId::Armaments);
        if bfb.is_some_and(|card|card.effective_cost==Some(0)) && armaments.is_some() {
            assert_eq!(combat.player.energy,3,"the public action has sufficient energy");
            assert_eq!(run.state.deck,permanent_deck,"discount is combat-local before the upgrade");
            run.choose_combat(CombatAction::PlayCard{instance:2,target:None}).unwrap();
            let after=run.decision().unwrap();
            assert!(run.state.fault.is_none(),"a valid Armaments+ action must not retain a runtime fault: {:?}",run.state.fault);
            let after_combat=after.combat.expect("Armaments resolves without ending combat");
            assert!(run.state.combat.as_ref().unwrap().fault_record().is_none(),
                "the actual Combat producer must not retain a validator/runtime fault: {:?}",run.state.combat.as_ref().unwrap().fault_record());
            let upgraded=after_combat.hand.iter().find(|card|card.instance.0==1&&card.id==CardId::BloodForBlood)
                .expect("the discounted card remains owned in hand");
            assert_eq!(upgraded.upgrades,1,"Armaments+ upgrades the sole eligible card in hand");
            assert_eq!(upgraded.effective_cost,Some(0),"persistent combat reduction keeps the upgraded card free");
            assert_eq!(upgraded.values.as_ref().unwrap().base_damage,Some(22));
            assert_eq!(run.state.deck,permanent_deck,"the temporary combat upgrade never mutates the Run master deck");
            return;
        }
        run.choose_combat(CombatAction::EndTurn).unwrap();
    }
    panic!("the real Jaw Worm attacks did not produce a fully discounted Blood for Blood within the bounded 16-turn fixture");
}
