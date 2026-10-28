use super::super::*;
use crate::content::{CardId, Catalog};
use super::full_runtime::entered_combat;

fn id(token: &str) -> CardId {
    Catalog::standard().cards().iter().find(|row| row.id.as_str() == token)
        .unwrap_or_else(|| panic!("missing executable green card {token}")).id
}

fn combat(token: &str, upgrades: u32) -> Combat {
    let mut combat = entered_combat(3, 3, |card| {
        *card = permanent_live_card(id(token), card.instance, upgrades);
    });
    combat.state.enemies[0].hp = 500;
    combat.state.enemies[0].max_hp = 500;
    combat
}

#[test]
fn m15_direct_pairs_use_shared_damage_and_block() {
    // Removing an individual hit, changing payment or merging hits must fail.
    for (token, damage, hits, block, cost) in [
        ("slice", [6, 9], 1, [0, 0], 0),
        ("dash", [10, 13], 1, [10, 13], 2),
        ("sucker_punch", [7, 9], 1, [0, 0], 1),
        ("backstab", [11, 15], 1, [0, 0], 0),
        ("riddle_with_holes", [3, 4], 5, [0, 0], 2),
    ] {
        for up in 0..=1 {
            let mut fight = combat(token, up);
            fight.state.enemies[0].block = 2;
            let card = fight.public_full().unwrap().hand.into_iter().find(|c|c.instance.0==3).unwrap();
            assert_eq!(card.effective_cost, Some(cost));
            let values = card.values.unwrap();
            assert_eq!(values.base_damage, Some(damage[up as usize]));
            assert_eq!(values.innate, token=="backstab");
            fight.step_full(&CombatAction::PlayCard{instance:3,target:Some(TargetId(1))}).unwrap();
            let view=fight.public_full().unwrap();
            assert_eq!(view.enemies[0].hp, 502-damage[up as usize]*hits, "{token}+{up}");
            assert_eq!(view.player.block,block[up as usize]);
            assert_eq!(view.player.energy,3-cost);
            assert_eq!(view.exhaust.iter().filter(|c|c.instance.0==3).count(),usize::from(token=="backstab"));
            if token=="sucker_punch" {
                assert_eq!(view.enemies[0].powers.iter().find(|p|p.id.0=="weak").unwrap().amount,1+up as i32);
            }
        }
    }
}

#[test]
fn m15_direct_aoe_hits_and_exhaust_have_distinct_recipients() {
    for (token,damage,hits) in [("dagger_spray",[4,6],2),("die_die_die",[13,17],1)] {
        for up in 0..=1 {
            let mut fight=combat(token,up);
            let mut second=fight.state.enemies[0].clone();
            second.instance_id=EnemyHandle::parse("enemy-2").unwrap();
            second.block=3;
            fight.state.enemies.push(second);
            let before=fight.state.clone();
            assert_eq!(fight.step_full(&CombatAction::PlayCard{instance:3,target:Some(TargetId(1))}),Err(crate::run::RunError::InvalidAction));
            assert_eq!(fight.state,before);
            fight.step_full(&CombatAction::PlayCard{instance:3,target:None}).unwrap();
            let view=fight.public_full().unwrap();
            assert_eq!(view.enemies[0].hp,500-damage[up as usize]*hits);
            assert_eq!(view.enemies[1].hp,503-damage[up as usize]*hits);
            assert_eq!(view.player.energy,2);
            assert_eq!(view.exhaust.iter().filter(|c|c.instance.0==3).count(),usize::from(token=="die_die_die"));
        }
    }
}

#[test]
fn m15_direct_energy_draw_block_and_artifact_use_real_effects() {
    for up in 0..=1 {
        let mut adrenaline=combat("adrenaline",up);
        let draw_cards=std::mem::take(&mut adrenaline.state.piles.hand);
        for card in draw_cards {
            if card.full().unwrap().instance.0==3 {adrenaline.state.piles.hand.push(card)}
            else {adrenaline.state.piles.discard.push(card)}
        }
        adrenaline.step_full(&CombatAction::PlayCard{instance:3,target:None}).unwrap();
        let view=adrenaline.public_full().unwrap();
        assert_eq!(view.player.energy,4+up as i32);
        assert_eq!(view.exhaust.iter().filter(|c|c.instance.0==3).count(),1);
        assert_eq!(view.hand.len(),2,"two real cards drawn from reshuffled discard");
        assert!(view.discard.is_empty());

        let mut sweep=combat("leg_sweep",up);
        sweep.step_full(&CombatAction::PlayCard{instance:3,target:Some(TargetId(1))}).unwrap();
        let view=sweep.public_full().unwrap();
        assert_eq!(view.player.block,if up==0{11}else{14});
        assert_eq!(view.enemies[0].powers.iter().find(|p|p.id.0=="weak").unwrap().amount,2+up as i32);
        assert_eq!(view.player.energy,1);

        let mut punch=combat("sucker_punch",up);
        punch.state.enemies[0].full_powers.push(FullPowerEntry{id:FullPowerId("artifact"),state:PowerState{amount:1,skip_first:false,monster_source_grace:false}});
        punch.step_full(&CombatAction::PlayCard{instance:3,target:Some(TargetId(1))}).unwrap();
        let view=punch.public_full().unwrap();
        assert_eq!(view.enemies[0].hp,if up==0{493}else{491});
        assert!(!view.enemies[0].powers.iter().any(|p|matches!(p.id.0,"weak"|"artifact")));
    }
}

#[test]
fn m15_direct_generated_and_innate_use_shared_constructors() {
    for token in ["dagger_spray","slice","sucker_punch","backstab","dash","leg_sweep","riddle_with_holes","adrenaline","die_die_die"] {
        for up in 0..=1 {
            let card=card_state::generated(id(token),CardInstanceId(3),up).expect("generated green constructor");
            let mut fight=combat(token,up);
            *fight.state.piles.hand.iter_mut().find(|c|c.full().unwrap().instance.0==3).unwrap()=model::OwnedCard::Full(card);
            let selected=matches!(token,"slice"|"sucker_punch"|"backstab"|"dash"|"leg_sweep"|"riddle_with_holes");
            fight.step_full(&CombatAction::PlayCard{instance:3,target:selected.then_some(TargetId(1))}).unwrap();
            assert!(fight.fault_record().is_none());
            if token=="backstab" {
                let mut entry=super::m8_shapes::entry(crate::content::EncounterId::OrbWalker);
                entry.cards=(1..=8).map(|n|permanent_live_card(if n==7{id(token)}else{CardId::DefendGreen},CardInstanceId(n),up)).collect();
                entry.opening_draw=1;
                let opened=Combat::enter(entry).unwrap().public_full().unwrap();
                assert_eq!(opened.hand.len(),1);
                assert_eq!(opened.hand[0].id,id("backstab"));
                assert_eq!(opened.draw_count,7);
                assert!(opened.known_draw.is_empty());
            }
        }
    }
}

#[test]
fn m15_direct_per_hit_callbacks_and_dash_block_precede_damage() {
    for (token,hits,block) in [("dash",1,10),("riddle_with_holes",5,0),("dagger_spray",2,0)] {
        let mut fight=combat(token,0);
        let before=fight.state.player.hp;
        fight.state.enemies[0].enemy_id=model::ActorEnemyId::Full(crate::content::EnemyId::Spiker);
        fight.state.enemies[0].full_powers.push(FullPowerEntry{id:FullPowerId("thorns"),state:PowerState{amount:3,skip_first:false,monster_source_grace:false}});
        fight.step_full(&CombatAction::PlayCard{instance:3,target:(token!="dagger_spray").then_some(TargetId(1))}).unwrap();
        let view=fight.public_full().unwrap();
        assert_eq!(view.player.hp,before-(3*hits-block).max(0));
        assert_eq!(view.player.block,(block-3*hits).max(0));
    }
}
