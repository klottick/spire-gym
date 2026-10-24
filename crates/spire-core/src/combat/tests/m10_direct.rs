use super::super::*;
use super::full_runtime::entered_combat;
use crate::content::{CardId,Catalog};
use crate::combat::model::OwnedCard;

fn with_card(id:CardId,upgraded:bool)->Combat{
    entered_combat(3,3,|card|*card=permanent_live_card(id,card.instance,u32::from(upgraded)))
}
fn play(c:&mut Combat,target:Option<TargetId>)->Result<(),crate::run::RunError>{
    c.step_full(&CombatAction::PlayCard{instance:3,target})
}
fn card(c:&Combat,id:CardId)->&LiveCard{c.state.piles.hand.iter().filter_map(OwnedCard::full).chain(c.state.in_play.iter().filter_map(OwnedCard::full)).find(|card|card.id==id).unwrap()}

#[test]
fn m10_direct_body_slam_uses_current_block_and_upgrade_cost(){
    for (upgraded,damage,cost) in [(false,17,1),(true,17,0)]{
        let mut c=with_card(CardId::BodySlam,upgraded);c.state.player.block=17;c.state.enemies[0].block=0;
        assert!(engine::validate_runtime(&c.state).is_ok());
        let prep=cards::prepare(&c,&CombatAction::PlayCard{instance:3,target:Some(TargetId(1))}).unwrap();
        assert_eq!(prep.remaining_energy,3-cost);
        assert_eq!(prep.effects.iter().find_map(|e|if let EffectKind::Damage(DamagePayload::Fixed(n))=e.kind{Some(n)}else{None}),Some(damage));
        let before=c.public_full().unwrap().enemies[0].hp;play(&mut c,Some(TargetId(1))).unwrap();
        assert_eq!(before-c.public_full().unwrap().enemies[0].hp,damage);
        assert!(c.state.piles.discard.iter().any(|c|c.full().is_some_and(|card|card.id==CardId::BodySlam)));
    }
}

#[test]
fn m10_direct_carnage_is_ethereal_and_hits_base_and_upgraded(){
    for (upgraded,damage) in [(false,20),(true,28)]{
        let mut c=with_card(CardId::Carnage,upgraded);
        assert!(engine::validate_runtime(&c.state).is_ok());
        assert!(cards::ironclad::rules::public_values(card(&c,CardId::Carnage)).unwrap().ethereal);
        c.step_full(&CombatAction::EndTurn).unwrap();
        assert!(c.state.piles.exhaust.iter().any(|c|c.full().is_some_and(|card|card.id==CardId::Carnage)));
        let mut c=with_card(CardId::Carnage,upgraded);c.state.enemies[0].block=0;
        let before=c.public_full().unwrap().enemies[0].hp;play(&mut c,Some(TargetId(1))).unwrap();
        assert_eq!(before-c.public_full().unwrap().enemies[0].hp,damage);
        assert!(c.state.piles.discard.iter().any(|c|c.full().is_some_and(|card|card.id==CardId::Carnage)));
    }
}

#[test]
fn m10_direct_clash_requires_an_all_attack_hand(){
    let mut invalid=with_card(CardId::Clash,false);
    assert!(engine::validate_runtime(&invalid.state).is_ok());
    let before=invalid.clone();assert_eq!(play(&mut invalid,Some(TargetId(1))),Err(crate::run::RunError::InvalidAction));assert_eq!(invalid,before);
    for (upgraded,damage) in [(false,14),(true,18)]{
        let mut c=with_card(CardId::Clash,upgraded);c.state.enemies[0].block=0;
        for held in &mut c.state.piles.hand{if held.handle().as_str()!="3"{if let Some(card)=held.full_mut(){card.id=CardId::Strike;}}}
        assert!(engine::validate_runtime(&c.state).is_ok());
        let before=c.public_full().unwrap().enemies[0].hp;play(&mut c,Some(TargetId(1))).unwrap();
        assert_eq!(before-c.public_full().unwrap().enemies[0].hp,damage);
    }
}

#[test]
fn m10_direct_hemokinesis_pays_hp_loss_before_damage(){
    for (upgraded,damage) in [(false,15),(true,20)]{
        let mut c=with_card(CardId::Hemokinesis,upgraded);let hp=c.state.player.hp;c.state.enemies[0].block=0;
        assert!(engine::validate_runtime(&c.state).is_ok());
        let prep=cards::prepare(&c,&CombatAction::PlayCard{instance:3,target:Some(TargetId(1))}).unwrap();
        assert_eq!(prep.effects[0].kind,EffectKind::Damage(DamagePayload::HpLoss(2)));
        assert_eq!(prep.effects[1].kind,EffectKind::Damage(DamagePayload::Fixed(damage)));
        let before=c.public_full().unwrap().enemies[0].hp;play(&mut c,Some(TargetId(1))).unwrap();
        assert_eq!(c.state.player.hp,hp-2);assert_eq!(before-c.public_full().unwrap().enemies[0].hp,damage);
    }
}

#[test]
fn m10_direct_pummel_hits_four_or_five_times_and_exhausts(){
    for (upgraded,hits) in [(false,4),(true,5)]{
        let mut c=with_card(CardId::Pummel,upgraded);c.state.enemies[0].block=0;let before=c.public_full().unwrap().enemies[0].hp;
        assert!(engine::validate_runtime(&c.state).is_ok());play(&mut c,Some(TargetId(1))).unwrap();
        assert_eq!(before-c.public_full().unwrap().enemies[0].hp,2*hits,"Pummel deals 2 damage per hit, four/five hits");
        assert!(c.state.piles.exhaust.iter().any(|c|c.full().is_some_and(|card|card.id==CardId::Pummel)));
    }
}

#[test]
fn m10_direct_pummel_keeps_second_hit_and_finish_when_first_hit_kills_target(){
    let mut entry=super::m8_shapes::entry(crate::content::EncounterId::ThreeShapes);
    entry.cards[0]=permanent_live_card(CardId::Pummel,CardInstanceId(1),0);
    let mut c=Combat::enter(entry).unwrap();c.state.enemies[0].block=0;c.state.enemies[0].hp=2;
    for enemy in c.state.enemies.iter_mut().skip(1){enemy.block=0;}
    let other_hp=c.state.enemies.iter().skip(1).map(|enemy|enemy.hp).collect::<Vec<_>>();
    assert!(engine::validate_runtime(&c.state).is_ok());
    c.step_full(&CombatAction::PlayCard{instance:1,target:Some(TargetId(1))}).unwrap();
    assert_eq!(c.public_full().unwrap().enemies[0].hp,0);
    assert_eq!(c.state.enemies.iter().skip(1).map(|enemy|enemy.hp).collect::<Vec<_>>(),other_hp,"remaining Pummel hits remain fixed on the original dead target; they do not retarget");
    assert_eq!(c.public_full().unwrap().outcome,None);
    assert!(c.state.piles.exhaust.iter().any(|owned|owned.full().is_some_and(|card|card.id==CardId::Pummel)),"FinishCard retains ordinary exhaust after the fixed-target attack chain");
    assert!(engine::validate_runtime(&c.state).is_ok());
}

#[test]
fn m10_direct_hemokinesis_death_prunes_later_hit_and_retains_played_parent(){
    let mut c=with_card(CardId::Hemokinesis,false);c.state.player.hp=1;c.state.enemies[0].block=0;
    let enemy_hp=c.state.enemies[0].hp;
    assert!(engine::validate_runtime(&c.state).is_ok());
    play(&mut c,Some(TargetId(1))).unwrap();
    assert_eq!(c.public_full().unwrap().outcome,Some(Outcome::Death));
    assert_eq!(c.public_full().unwrap().enemies[0].hp,enemy_hp,"later attack is pruned after payment Death");
    assert_eq!(c.state.in_play.as_ref().unwrap().full().unwrap().id,CardId::Hemokinesis,"terminal damage keeps parent custody");
    assert!(engine::validate_runtime(&c.state).is_ok());
}

#[test]
fn m10_direct_pummel_applies_enemy_thorns_once_per_hit(){
    let mut entry=super::m8_shapes::single(crate::content::EnemyId::Spiker);
    entry.cards[0]=permanent_live_card(CardId::Pummel,CardInstanceId(1),0);
    let mut c=Combat::enter(entry).unwrap();c.state.enemies[0].block=0;
    let hp=c.state.player.hp;
    assert!(engine::validate_runtime(&c.state).is_ok());
    c.step_full(&CombatAction::PlayCard{instance:1,target:Some(TargetId(1))}).unwrap();
    assert_eq!(c.state.player.hp,hp-12,"enemy THORNS retaliates separately for all four Pummel hits");
}

#[test]
fn m10_direct_sword_boomerang_selects_each_hit_at_dispatch(){
    for (upgraded,damage) in [(false,9),(true,12)]{
        let mut c=with_card(CardId::SwordBoomerang,upgraded);c.state.enemies[0].block=0;
        let initial=c.state.card_rng.as_ref().unwrap().clone();
        assert!(engine::validate_runtime(&c.state).is_ok());play(&mut c,None).unwrap();
        assert_eq!(c.state.enemies[0].max_hp-c.state.enemies[0].hp,damage);
        assert_eq!(c.state.card_rng.as_ref().unwrap(),&initial,"singleton uniform choice does not advance the scoped native cursor");
    }
}

#[test]
fn m10_direct_sword_boomerang_recalculates_vulnerable_and_retaliates_per_hit(){
    for (upgraded,hits,per_hit) in [(false,3,4),(true,4,4)]{
        let mut entry=super::m8_shapes::single(crate::content::EnemyId::Spiker);
        entry.cards[0]=permanent_live_card(CardId::SwordBoomerang,CardInstanceId(1),u32::from(upgraded));
        let mut c=Combat::enter(entry).unwrap();c.state.enemies[0].block=0;
        c.state.enemies[0].full_powers.push(FullPowerEntry{id:FullPowerId("vulnerable"),state:PowerState{amount:1,skip_first:false,monster_source_grace:false}});
        let hp=c.state.player.hp;let enemy_hp=c.state.enemies[0].hp;
        assert!(engine::validate_runtime(&c.state).is_ok());
        c.step_full(&CombatAction::PlayCard{instance:1,target:None}).unwrap();
        assert_eq!(enemy_hp-c.state.enemies[0].hp,per_hit*hits,"modified NORMAL damage applies to every random hit");
        assert_eq!(c.state.player.hp,hp-3*hits,"Spiker THORNS reacts once per selected hit");
    }
}

#[test]
fn m10_direct_card_target_preflight_rejects_invalid_targets_atomically(){
    // Wild Strike requires one selected living target; Boomerang selects its
    // own target per hit and must reject an externally supplied selection.
    let mut wild=with_card(CardId::WildStrike,false);
    let before=wild.clone();
    assert_eq!(play(&mut wild,None),Err(crate::run::RunError::InvalidAction));
    assert_eq!(wild,before,"missing Wild Strike target must be rejected before any card/state mutation");

    let mut boomerang=with_card(CardId::SwordBoomerang,false);
    let before=boomerang.clone();
    assert_eq!(play(&mut boomerang,Some(TargetId(1))),Err(crate::run::RunError::InvalidAction));
    assert_eq!(boomerang,before,"random-target Boomerang must reject a supplied target atomically");

    for target in [TargetId(99),TargetId(1)] {
        let mut selected=with_card(CardId::WildStrike,false);
        if target==TargetId(1){selected.state.enemies[0].hp=0;}
        let before=selected.clone();
        assert_eq!(play(&mut selected,Some(target)),Err(crate::run::RunError::InvalidAction));
        assert_eq!(selected,before,"unknown/dead selected target must reject atomically: {target:?}");
    }
}

#[test]
fn m10_direct_legal_actions_full_does_not_offer_invalid_target_shapes(){
    let c=with_card(CardId::WildStrike,false);
    let actions=c.legal_actions_full().unwrap();
    assert!(actions.iter().any(|action|matches!(action,CombatAction::PlayCard{instance:3,target:Some(TargetId(1))})));
    assert!(!actions.iter().any(|action|matches!(action,CombatAction::PlayCard{instance:3,target:None})));

    let c=with_card(CardId::SwordBoomerang,false);
    let actions=c.legal_actions_full().unwrap();
    assert!(actions.iter().any(|action|matches!(action,CombatAction::PlayCard{instance:3,target:None})));
    assert!(!actions.iter().any(|action|matches!(action,CombatAction::PlayCard{instance:3,target:Some(_)})));
}

#[test]
fn m10_direct_sword_boomerang_reselects_after_a_hit_kills_its_target(){
    let mut entry=super::m8_shapes::entry(crate::content::EncounterId::ThreeShapes);
    entry.cards[0]=permanent_live_card(CardId::SwordBoomerang,CardInstanceId(1),0);
    let mut c=Combat::enter(entry).unwrap();
    for enemy in &mut c.state.enemies{enemy.block=0;}
    c.state.enemies[0].hp=3;
    let initial=c.state.card_rng.as_ref().unwrap().clone();
    let cursor=(0..1000).find_map(|n|{
        let stream=crate::combat::rng::StreamState::new(initial.id().clone(),*initial.key(),n).ok()?;
        let (pick,_)=crate::combat::rng::uniform_below(&stream,3).ok()?;
        (pick==0).then_some(stream)
    }).unwrap();
    c.state.card_rng=Some(cursor);
    assert!(engine::validate_runtime(&c.state).is_ok());
    c.step_full(&CombatAction::PlayCard{instance:1,target:None}).unwrap();
    assert_eq!(c.state.enemies[0].hp,0,"first action selected and killed first encounter-list actor");
    assert_eq!(c.state.enemies.iter().skip(1).map(|enemy|enemy.hp).sum::<i32>(),
        c.state.enemies.iter().skip(1).map(|enemy|enemy.max_hp).sum::<i32>()-6,
        "the two remaining hits independently select among current living actors");
}

#[test]
fn m10_direct_sword_boomerang_skips_remaining_hits_when_no_living_target_remains(){
    let mut entry=super::m8_shapes::single(crate::content::EnemyId::Spiker);
    entry.cards[0]=permanent_live_card(CardId::SwordBoomerang,CardInstanceId(1),0);
    let mut c=Combat::enter(entry).unwrap();c.state.enemies[0].block=0;c.state.enemies[0].hp=3;
    let player_hp=c.state.player.hp;
    assert!(engine::validate_runtime(&c.state).is_ok());
    c.step_full(&CombatAction::PlayCard{instance:1,target:None}).unwrap();
    assert_eq!(c.public_full().unwrap().outcome,Some(Outcome::Victory));
    assert_eq!(c.state.enemies[0].hp,0);
    assert_eq!(c.state.player.hp,player_hp-3,"only the first random selector finds a target and its THORNS reaction resolves once");
}

#[test]
fn m10_direct_wild_strike_generates_wound_into_shuffled_draw(){
    for (upgraded,damage) in [(false,12),(true,17)]{
        let mut c=with_card(CardId::WildStrike,upgraded);c.state.enemies[0].block=0;
        assert!(engine::validate_runtime(&c.state).is_ok());play(&mut c,Some(TargetId(1))).unwrap();
        assert_eq!(c.state.enemies[0].max_hp-c.state.enemies[0].hp,damage);
        assert!(c.state.piles.draw.iter().chain(c.state.piles.hand.iter()).any(|owned|owned.full().is_some_and(|card|card.id==CardId::Wound)));
    }
}

#[test]
fn m10_direct_wild_strike_insertion_uses_a_slot_among_existing_draw_cards(){
    for n in 0..=2 {
        let mut c=with_card(CardId::WildStrike,false);c.state.enemies[0].block=0;
        for instance in 100..100+n as u64 {
            let handle=CardHandle::parse(&instance.to_string()).unwrap();c.state.card_ids_issued.insert(handle);
            c.state.piles.draw.push(OwnedCard::Full(permanent_live_card(CardId::Strike,CardInstanceId(instance),0)));
        }
        let stream=c.state.card_rng.as_ref().unwrap().clone();let shuffle=c.state.rng.shuffle.clone();
        let (expected,expected_next)=crate::combat::rng::uniform_below(&stream,n.max(1) as u128).unwrap();
        play(&mut c,Some(TargetId(1))).unwrap();
        let actual=c.state.piles.draw.iter().position(|owned|owned.full().is_some_and(|card|card.id==CardId::Wound)).unwrap();
        assert_eq!(actual,if n==0{0}else{expected as usize},"n={n}: insertion samples exactly n positions; empty is deterministic");
        assert_eq!(c.state.card_rng.as_ref().unwrap(),&expected_next,"n={n}: scoped card-random cursor follows uniform_below(n), or is unchanged for empty");
        assert_eq!(c.state.rng.shuffle,shuffle,"Wild Strike insertion does not consume the distinct opening/combat shuffle stream");
    }
}

#[test]
fn m10_direct_bottom_draw_is_deterministic_and_does_not_use_card_random(){
    let mut c=with_card(CardId::WildStrike,false);
    let initial=c.state.card_rng.as_ref().unwrap().clone();
    c.enqueue_native_effects(vec![Effect{source:Source::Card(CardHandle::parse("3").unwrap()),target:Some(Actor::Player),
        kind:EffectKind::CreateCard{id:CardId::Wound,destination:CardDestination::BottomDraw,upgrades:0}}],None).unwrap();
    assert_eq!(c.state.piles.draw.first().and_then(OwnedCard::full).unwrap().id,CardId::Wound);
    assert_eq!(c.state.card_rng.as_ref().unwrap(),&initial);
}

#[test]
fn m10_direct_uppercut_applies_literal_damage_and_debuffs(){
    for (upgraded,weak,vulnerable) in [(false,1,1),(true,2,2)]{
        let mut c=with_card(CardId::Uppercut,upgraded);c.state.enemies[0].block=0;let before=c.public_full().unwrap().enemies[0].hp;
        assert!(engine::validate_runtime(&c.state).is_ok());play(&mut c,Some(TargetId(1))).unwrap();
        assert_eq!(before-c.public_full().unwrap().enemies[0].hp,13);
        assert!(c.state.enemies[0].full_powers.iter().any(|p|p.id.0=="weak"&&p.state.amount==weak));
        assert!(c.state.enemies[0].full_powers.iter().any(|p|p.id.0=="vulnerable"&&p.state.amount==vulnerable));
    }
}

#[test]
fn m10_direct_immolate_hits_all_living_enemies_and_creates_discard_burn(){
    for (upgraded,damage) in [(false,21),(true,28)]{
        let mut c=with_card(CardId::Immolate,upgraded);c.state.enemies[0].block=0;let before=c.public_full().unwrap().enemies[0].hp;
        assert!(engine::validate_runtime(&c.state).is_ok());play(&mut c,None).unwrap();
        assert_eq!(before-c.public_full().unwrap().enemies[0].hp,damage);
        assert!(c.state.piles.discard.iter().any(|c|c.full().is_some_and(|card|card.id==CardId::Burn)));
    }
}

#[test]
fn m10_direct_cards_publish_real_stats_flags_costs_and_standard_availability(){
    for (id,upgraded,damage,magic,cost,ethereal,exhaust) in [
        (CardId::BodySlam,false,None,None,1,false,false),(CardId::BodySlam,true,None,None,0,false,false),
        (CardId::Carnage,false,Some(20),None,2,true,false),(CardId::Carnage,true,Some(28),None,2,true,false),
        (CardId::Clash,false,Some(14),None,0,false,false),(CardId::Clash,true,Some(18),None,0,false,false),
        (CardId::Hemokinesis,false,Some(15),Some(2),1,false,false),(CardId::Hemokinesis,true,Some(20),Some(2),1,false,false),
        (CardId::Pummel,false,Some(2),Some(4),1,false,true),(CardId::Pummel,true,Some(2),Some(5),1,false,true),
        (CardId::SwordBoomerang,false,Some(3),Some(3),1,false,false),(CardId::SwordBoomerang,true,Some(3),Some(4),1,false,false),
        (CardId::Uppercut,false,Some(13),Some(1),2,false,false),(CardId::Uppercut,true,Some(13),Some(2),2,false,false),
        (CardId::WildStrike,false,Some(12),None,1,false,false),(CardId::WildStrike,true,Some(17),None,1,false,false),
        (CardId::Immolate,false,Some(21),None,2,false,false),(CardId::Immolate,true,Some(28),None,2,false,false),
    ] {
        assert!(Catalog::standard().card(id).is_some(),"{id:?} is an available standard card");
        if id==CardId::SwordBoomerang {assert_eq!(Catalog::standard().card(id).unwrap().rarity,crate::content::CardRarity::Common,"reference row identifies Sword Boomerang as COMMON");}
        let mut c=with_card(id,upgraded);
        if id==CardId::Clash {for held in &mut c.state.piles.hand{if held.full().is_some_and(|live|live.id!=id){if let Some(live)=held.full_mut(){live.id=CardId::Strike;}}}}
        c.state.player.block=17;
        let live=card(&c,id);let values=cards::ironclad::rules::public_values(live).unwrap();
        assert_eq!((values.base_damage,values.magic_number,values.ethereal,values.exhaust_on_finish),(damage,magic,ethereal,exhaust),"public stat/flag mismatch for {id:?} upgraded={upgraded}");
        let target=if matches!(id,CardId::Immolate|CardId::SwordBoomerang){None}else{Some(TargetId(1))};
        let prepared=cards::prepare(&c,&CombatAction::PlayCard{instance:3,target}).unwrap();
        assert_eq!(prepared.remaining_energy,3-cost,"cost mismatch for {id:?} upgraded={upgraded}");
    }
}
