use super::super::*;
use super::full_runtime::entered_combat;

fn with_card(id:crate::content::CardId,upgrades:u32)->(Combat,u64){
    let mut combat=entered_combat(3,3, |_|{});
    let instance=combat.state.piles.hand[0].full().unwrap().instance;
    *combat.state.piles.hand[0].full_mut().unwrap()=permanent_live_card(id,instance,upgrades);
    combat.state.character=CharacterState::for_character(crate::content::Character::Defect);
    (combat,instance.0)
}
#[test]
fn m16_mutations_genetic_capture_and_copy_family(){
    for up in [0,1]{
        let (mut combat,instance)=with_card(crate::content::CardId::GeneticAlgorithm,up);
        combat.step_full(&CombatAction::PlayCard{instance,target:None}).unwrap();
        assert_eq!(combat.state.player.block,1);
        let learned=combat.state.piles.exhaust.iter().find_map(|c|c.full().filter(|c|c.instance.0==instance)).unwrap();
        assert_eq!(learned.misc,if up>0{4}else{3});
    }
}
#[test]
fn m16_mutations_claw_wear_streamline_reset(){
    use crate::content::CardId::*;
    for up in [0,1]{
        let (mut claw,instance)=with_card(Claw,up);let hp=claw.state.enemies[0].hp;
        claw.step_full(&CombatAction::PlayCard{instance,target:Some(TargetId(1))}).unwrap();
        assert_eq!(hp-claw.state.enemies[0].hp,if up>0{5}else{3});
        assert_eq!(claw.state.piles.discard.last().unwrap().full().unwrap().local.base_damage,Some(if up>0{7}else{5}));
        let (mut steam,instance)=with_card(SteamBarrier,up);
        steam.step_full(&CombatAction::PlayCard{instance,target:None}).unwrap();
        assert_eq!(steam.state.player.block,if up>0{8}else{6});
        assert_eq!(steam.state.piles.discard.last().unwrap().full().unwrap().local.base_block,Some(if up>0{7}else{5}));
        let (mut stream,instance)=with_card(Streamline,up);
        stream.step_full(&CombatAction::PlayCard{instance,target:Some(TargetId(1))}).unwrap();
        assert_eq!(stream.state.piles.discard.last().unwrap().full().unwrap().local.mutable_combat_cost,Some(1));
    }
}

#[test]
fn m16_mutations_genetic_same_family_repeat_and_fresh_copy(){
    for up in [0,1]{
        let (mut combat,instance)=with_card(crate::content::CardId::GeneticAlgorithm,up);
        combat.state.player_full_powers.push(FullPowerEntry{id:FullPowerId("echo_form"),state:PowerState{amount:1,skip_first:false,monster_source_grace:false}});
        combat.step_full(&CombatAction::PlayCard{instance,target:None}).unwrap();
        assert_eq!(combat.state.player.block,if up>0{5}else{4});
        assert_eq!(combat.permanent_card_mutations().len(),2);
        assert_eq!(combat.permanent_card_mutations()[1].misc,if up>0{7}else{5});
        let (mut fresh,_)=with_card(crate::content::CardId::GeneticAlgorithm,up);
        let original=fresh.state.piles.hand[0].full().unwrap().clone();
        let copied=original.stat_equivalent_copy(CardInstanceId(10));
        super::super::piles::insert_generated(&mut fresh.state,copied,CardDestination::Hand).unwrap();
        fresh.step_full(&CombatAction::PlayCard{instance:10,target:None}).unwrap();
        assert!(fresh.permanent_card_mutations().is_empty());
        assert_eq!(fresh.state.piles.hand[0].full().unwrap().misc,1);
    }
}

#[test]
fn m16_mutations_signed_raw_fields_are_floored_after_stats(){
    let (mut steam,instance)=with_card(crate::content::CardId::SteamBarrier,0);
    steam.state.piles.hand[0].full_mut().unwrap().local.base_block=Some(-2);
    steam.state.player_full_powers.push(FullPowerEntry{id:FullPowerId("dexterity"),state:PowerState{amount:5,skip_first:false,monster_source_grace:false}});
    steam.step_full(&CombatAction::PlayCard{instance,target:None}).unwrap();
    assert_eq!(steam.state.player.block,3,"-2 raw Block +5 Dex, then floor");
    assert_eq!(steam.state.piles.discard.last().unwrap().full().unwrap().local.base_block,Some(-3));
    let (mut genetic,instance)=with_card(crate::content::CardId::GeneticAlgorithm,0);genetic.state.piles.hand[0].full_mut().unwrap().misc=-2;
    genetic.state.player_full_powers.push(FullPowerEntry{id:FullPowerId("dexterity"),state:PowerState{amount:5,skip_first:false,monster_source_grace:false}});
    genetic.step_full(&CombatAction::PlayCard{instance,target:None}).unwrap();assert_eq!(genetic.state.player.block,3);
    assert_eq!(genetic.permanent_card_mutations()[0].misc,0);
    let (mut claw,instance)=with_card(crate::content::CardId::Claw,0);claw.state.piles.hand[0].full_mut().unwrap().local.base_damage=Some(-2);
    claw.state.player_full_powers.push(FullPowerEntry{id:FullPowerId("strength"),state:PowerState{amount:5,skip_first:false,monster_source_grace:false}});
    let hp=claw.state.enemies[0].hp;claw.step_full(&CombatAction::PlayCard{instance,target:Some(TargetId(1))}).unwrap();assert_eq!(hp-claw.state.enemies[0].hp,3);
}

#[test]
fn m16_mutations_genetic_stasis_copy_death_census(){
    let original=permanent_live_card(crate::content::CardId::GeneticAlgorithm,CardInstanceId(1),0);
    let mut combat=super::m6_stasis::combat(vec![original]);
    combat.state.character=CharacterState::for_character(crate::content::Character::Defect);
    combat.state.silent.source_character=Some(crate::content::Character::Defect);
    super::m6_stasis::capture(&mut combat,1);
    let orb=combat.state.enemies[1].instance_id.clone();
    let source=combat.state.stasis[&orb].handle();
    let copy=super::super::piles::make_same_instance_family_copy_from_owner(&mut combat.state,&source,CardInstanceId(2)).unwrap();
    super::super::piles::insert_generated(&mut combat.state,copy,CardDestination::Hand).unwrap();
    combat.step_full(&CombatAction::PlayCard{instance:2,target:None}).unwrap();
    assert_eq!(combat.state.stasis[&orb].full().unwrap().misc,3);
    assert_eq!(combat.permanent_card_mutations()[0].instance,CardInstanceId(1));
    let hp=combat.state.player.hp;
    combat.enqueue_native_effects(vec![Effect{source:Source::None,target:Some(Actor::Player),kind:EffectKind::Damage(DamagePayload::HpLoss(hp))}],None).unwrap();
    let exit=combat.exit().unwrap();assert_eq!(exit.outcome,Outcome::Death);
    assert_eq!(exit.stasis[0].misc,3);assert_eq!(exit.stasis[0].origin,CombatCardRef::Permanent(CardInstanceId(1)));
    combat.validate_permanent_mutations_for_run().unwrap();
}

#[test]
fn m16_mutations_wear_upgrade_price_branches_and_family_scope(){
    use crate::content::CardId::*;
    let (mut steam,instance)=with_card(SteamBarrier,0);steam.state.piles.hand[0].full_mut().unwrap().local.base_block=Some(-2);
    super::super::cards::ironclad::rules::upgrade(steam.state.piles.hand[0].full_mut().unwrap()).unwrap();
    assert_eq!(steam.state.piles.hand[0].full().unwrap().local.base_block,Some(0));
    steam.step_full(&CombatAction::PlayCard{instance,target:None}).unwrap();assert_eq!(steam.state.player.block,0);
    assert_eq!(steam.state.piles.discard.last().unwrap().full().unwrap().local.base_block,Some(-1));
    for turn in [0,3]{
        let (mut stream,instance)=with_card(Streamline,0);
        let original=stream.state.piles.hand[0].full_mut().unwrap();original.local.randomized_cost=Some(3);original.local.mutable_combat_cost=Some(3);original.local.turn_cost=Some(turn);
        let owner=original.instance;
        let family=super::super::piles::make_same_instance_family_copy_from_owner(&mut stream.state,&super::super::card_state::handle(owner),CardInstanceId(30)).unwrap();
        super::super::piles::insert_generated(&mut stream.state,family,CardDestination::Hand).unwrap();
        let fresh=stream.state.piles.hand[0].full().unwrap().stat_equivalent_copy(CardInstanceId(31));
        super::super::piles::insert_generated(&mut stream.state,fresh,CardDestination::Hand).unwrap();
        stream.step_full(&CombatAction::PlayCard{instance,target:Some(TargetId(1))}).unwrap();
        let family=stream.state.piles.hand.iter().filter_map(|c|c.full()).find(|c|c.instance==CardInstanceId(30)).unwrap();
        assert_eq!(family.local.mutable_combat_cost,Some(2));assert_eq!(family.local.turn_cost,Some(if turn==0{0}else{2}));
        let fresh=stream.state.piles.hand.iter().filter_map(|c|c.full()).find(|c|c.instance==CardInstanceId(31)).unwrap();assert_eq!(fresh.local.mutable_combat_cost,Some(3));assert_eq!(fresh.local.turn_cost,Some(turn));
    }
    let (mut genetic,instance)=with_card(GeneticAlgorithm,0);genetic.state.piles.hand[0].full_mut().unwrap().misc=17;
    super::super::cards::ironclad::rules::upgrade(genetic.state.piles.hand[0].full_mut().unwrap()).unwrap();
    let other=permanent_live_card(GeneticAlgorithm,CardInstanceId(30),0).copy_as(CardInstanceId(30));super::super::piles::insert_generated(&mut genetic.state,other,CardDestination::Hand).unwrap();
    genetic.step_full(&CombatAction::PlayCard{instance,target:None}).unwrap();
    assert_eq!(genetic.state.player.block,17);assert_eq!(genetic.permanent_card_mutations()[0].misc,20);
    assert_eq!(genetic.state.piles.hand.iter().filter_map(|c|c.full()).find(|c|c.instance==CardInstanceId(30)).unwrap().misc,1);
}
