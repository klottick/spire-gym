use super::super::*;
use super::full_runtime::entered_combat;

fn defect() -> Combat {
    let mut combat=entered_combat(3,3, |_|{});
    combat.state.character=CharacterState::for_character(crate::content::Character::Defect);
    combat
}

#[test]
fn m16_orbs_focus_empty_capacity_and_bounds() {
    for slots in [0,3] {
        let mut combat=defect();combat.state.character.orb_slots=vec![None;slots];
        super::super::characters::apply_focus(&mut combat.state,2).unwrap();
        assert_eq!(combat.state.character.focus,2);
        super::super::characters::apply_focus(&mut combat.state,-3).unwrap();
        assert_eq!(combat.state.character.focus,-1);
        super::super::characters::apply_focus(&mut combat.state,2000).unwrap();
        assert_eq!(combat.state.character.focus,999);
    }
}

#[test]
fn m16_orbs_dark_identity_and_target() {
    let mut combat=defect();
    combat.state.character.orb_slots=vec![Some(OrbKind::Dark.into()),None,None];
    super::super::characters::apply(&mut combat.state,&Source::Actor(Actor::Player),CharacterEffect::EndTurn).unwrap();
    assert_eq!(combat.state.character.orb_slots[0].unwrap().dark_damage,Some(12));
    combat.state.character.focus=-10;
    super::super::characters::apply(&mut combat.state,&Source::Actor(Actor::Player),CharacterEffect::EndTurn).unwrap();
    assert_eq!(combat.state.character.orb_slots[0].unwrap().dark_damage,Some(12));
}

#[test]
fn m16_orbs_sparse_front_does_not_search_ahead() {
    let mut combat=defect();
    combat.state.character.orb_slots=vec![None,Some(OrbKind::Frost.into()),None];
    let before=combat.state.character.clone();
    super::super::characters::apply(&mut combat.state,&Source::Actor(Actor::Player),CharacterEffect::EvokeFront{remove:true}).unwrap();
    assert_eq!(combat.state.character,before);
    assert!(combat.state.pending.is_empty());
}

fn play_card(id:crate::content::CardId,upgrades:u32,slots:usize) -> Combat {
    let mut combat=defect();
    combat.state.character.orb_slots=vec![None;slots];
    let instance=combat.state.piles.hand[0].full().unwrap().instance;
    *combat.state.piles.hand[0].full_mut().unwrap()=permanent_live_card(id,instance,upgrades);
    combat.step_full(&CombatAction::PlayCard{instance:instance.0,target:None}).unwrap();
    combat
}
#[test]
fn m16_orbs_real_card_producers_base_and_upgraded() {
    use crate::content::CardId::*;
    for (up,focus,slots) in [(0,2,5),(1,3,6)] {
        let consumed=play_card(Consume,up,0);
        assert_eq!((consumed.state.character.focus,consumed.state.character.orb_slots.len()),(focus,0));
        let capacity=play_card(Capacitor,up,3);
        assert_eq!(capacity.state.character.orb_slots.len(),slots);
        let darkness=play_card(Darkness,up,3);
        assert_eq!(darkness.state.character.orb_slots[0].unwrap().dark_damage,Some(if up>0{12}else{6}));
        let fusion=play_card(Fusion,up,3);
        assert_eq!(fusion.state.character.orb_slots[0].unwrap().kind,OrbKind::Plasma);
    }
}

fn generated_play(combat:&mut Combat,id:crate::content::CardId,upgrades:u32,target:Option<TargetId>){
    let instance=super::super::piles::next_generated_instance(&combat.state).unwrap();
    let card=super::super::card_state::generated(id,instance,upgrades).unwrap();
    super::super::piles::insert_generated(&mut combat.state,card,CardDestination::Hand).unwrap();
    combat.step_full(&CombatAction::PlayCard{instance:instance.0,target}).unwrap();
}
#[test]
fn m16_orbs_capacity_and_sparse_barrage(){
    use crate::content::CardId::*;
    let mut combat=entered_combat(20,20, |_|{});combat.state.character=CharacterState::for_character(crate::content::Character::Defect);
    combat.state.enemies[0].max_hp=999;combat.state.enemies[0].hp=999;
    for _ in 0..3{generated_play(&mut combat,Capacitor,1,None);}
    assert_eq!(combat.state.character.orb_slots.len(),10);
    for _ in 0..10{generated_play(&mut combat,Zap,1,None);}
    assert_eq!(combat.state.character.orb_slots.iter().flatten().count(),10);
    generated_play(&mut combat,Barrage,0,Some(TargetId(1)));
    assert_eq!(combat.state.enemies[0].hp,959,"ten occupied slots cause ten separate four-damage hits");
    generated_play(&mut combat,Consume,0,None);assert_eq!(combat.state.character.orb_slots.len(),9);
}
#[test]
fn m16_orbs_plasma_phase_and_focus(){
    use super::super::characters::{invoke_passive,OrbPassivePhase};
    for focus in [-9,9]{
        let mut combat=defect();combat.state.character.focus=focus;
        combat.state.character.orb_slots=vec![Some(OrbKind::Plasma.into()),None,None];combat.state.player.energy=0;
        invoke_passive(&mut combat.state,&Source::Actor(Actor::Player),0,OrbPassivePhase::End).unwrap();assert!(combat.state.pending.is_empty());
        invoke_passive(&mut combat.state,&Source::Actor(Actor::Player),0,OrbPassivePhase::Start).unwrap();
        combat.state.pending.push_back(Effect{source:Source::None,target:None,kind:EffectKind::Decision});
        super::super::engine::resolve(&mut combat.state);assert_eq!(combat.state.player.energy,1);
        generated_play(&mut combat,crate::content::CardId::Recursion,1,None);assert_eq!(combat.state.player.energy,3);
        assert_eq!(combat.state.character.orb_slots[0].unwrap().kind,OrbKind::Plasma);
    }
}
#[test]
fn m16_orbs_recursion_fission_terminal(){
    use crate::content::CardId::*;
    for up in [0,1]{
        let mut combat=defect();combat.state.character.orb_slots=vec![Some(OrbKind::Frost.into()),Some(OrbKind::Plasma.into()),None];
        combat.state.player.energy=0;
        generated_play(&mut combat,Fission,up,None);
        assert_eq!(combat.state.player.energy,if up>0{4}else{2});
        assert_eq!(combat.state.player.block,if up>0{5}else{0});
        assert!(combat.state.character.orb_slots.iter().all(Option::is_none));
        assert!(combat.state.piles.exhaust.iter().any(|c|c.full().is_some_and(|c|c.id==Fission)));
    }
    let mut combat=defect();combat.state.character.orb_slots=vec![Some(OrbState{kind:OrbKind::Dark,dark_damage:Some(2000)}),Some(OrbKind::Plasma.into()),None];
    generated_play(&mut combat,Fission,1,None);
    assert_eq!(combat.state.outcome,Some(Outcome::Victory));
    assert_eq!(combat.state.player.energy,3,"lethal first evoke prunes subsequent Plasma and Fission energy");
}

#[test]
fn m16_orbs_dark_kind_count_target_and_recursion_payload(){
    use crate::content::CardId::*;
    for frost in [false,true]{
        let mut combat=defect();
        combat.state.character.orb_slots=vec![Some(OrbState{kind:OrbKind::Dark,dark_damage:Some(6)}),Some(OrbState{kind:OrbKind::Dark,dark_damage:Some(12)}),if frost{Some(OrbKind::Frost.into())}else{None}];
        for n in 0..3{super::super::piles::insert_generated(&mut combat.state,permanent_live_card(DefendBlue,CardInstanceId(10+n),0).copy_as(CardInstanceId(10+n)),CardDestination::Draw).unwrap();}
        generated_play(&mut combat,CompileDriver,0,Some(TargetId(1)));
        assert_eq!(combat.state.piles.draw.len(),if frost{1}else{2},"different Dark payloads are one kind");
    }
    let mut combat=defect();combat.state.character.orb_slots=vec![Some(OrbState{kind:OrbKind::Dark,dark_damage:Some(12)}),None,None];
    let hp=combat.state.enemies[0].hp;generated_play(&mut combat,Recursion,1,None);
    assert_eq!(combat.state.enemies[0].hp,hp-12);assert_eq!(combat.state.character.orb_slots[0].unwrap().dark_damage,Some(12));
    let mut combat=defect();let base=combat.state.enemies[0].clone();
    combat.state.enemies=(0..3).map(|index|{let mut enemy=base.clone();enemy.instance_id=EnemyHandle::parse(&format!("target-{index}")).unwrap();enemy.hp=if index==0{9}else{4};enemy}).collect();
    combat.state.character.orb_slots=vec![Some(OrbKind::Dark.into())];let before=combat.state.card_rng.clone();
    super::super::characters::apply(&mut combat.state,&Source::Actor(Actor::Player),CharacterEffect::EvokeFront{remove:true}).unwrap();
    assert_eq!(combat.state.pending.front().unwrap().target,Some(Actor::Enemy(EnemyHandle::parse("target-1").unwrap())));
    assert_eq!(combat.state.card_rng,before,"Dark tie selection consumes no RNG");
}

#[test]
fn m16_orbs_focus_has_one_public_power_owner(){
    let mut combat=play_card(crate::content::CardId::Defragment,1,0);
    assert_eq!(combat.state.character.focus,2);
    assert_eq!(combat.public_full().unwrap().player.powers.iter().find(|p|p.id.0=="focus").map(|p|p.amount),Some(2));
    assert!(!combat.state.player_full_powers.iter().any(|p|p.id.0=="focus"));
    let remove=Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::RemoveFullPower(FullPowerId("focus"))};
    super::super::effects::apply_effect(&mut combat.state,&remove).unwrap();assert_eq!(combat.state.character.focus,0);
}

#[test]
fn m16_orbs_dark_selects_half_dead_without_duplicate_death(){
    let mut combat=Combat::enter(super::m8_shapes::entry(crate::content::EncounterId::ThreeDarklings)).unwrap();
    combat.state.character=CharacterState::for_character(crate::content::Character::Defect);combat.state.silent.source_character=Some(crate::content::Character::Defect);
    let hp=combat.state.enemies[0].hp;super::m5_elites::hit(&mut combat,0,hp);
    assert_eq!(combat.state.enemies[0].life,ActorLife::HalfDead);
    let others=combat.state.enemies.iter().skip(1).map(|e|e.hp).collect::<Vec<_>>();
    let deaths=combat.state.hook_events.iter().filter(|e|matches!(e.hook,Hook::Death|Hook::HalfDeath)).count();
    combat.state.character.orb_slots=vec![Some(OrbState{kind:OrbKind::Dark,dark_damage:Some(12)}),None,None];
    generated_play(&mut combat,crate::content::CardId::Recursion,1,None);
    assert_eq!(combat.state.enemies[0].life,ActorLife::HalfDead);assert_eq!(combat.state.enemies[0].hp,0);
    assert_eq!(combat.state.enemies.iter().skip(1).map(|e|e.hp).collect::<Vec<_>>(),others);
    assert_eq!(combat.state.hook_events.iter().filter(|e|matches!(e.hook,Hook::Death|Hook::HalfDeath)).count(),deaths);
    assert_eq!(combat.state.character.orb_slots[0].unwrap().dark_damage,Some(12));
}

#[test]
fn m16_orbs_eviction_rng_fault_retains_current_work_and_slot_custody(){
    let mut combat=defect();combat.state.character.orb_slots=vec![Some(OrbKind::Lightning.into());3];
    let instance=combat.state.piles.hand[0].full().unwrap().instance;
    *combat.state.piles.hand[0].full_mut().unwrap()=permanent_live_card(crate::content::CardId::Zap,instance,1);
    let stream=combat.state.card_rng.as_ref().unwrap();combat.state.card_rng=Some(rng::StreamState::new(stream.id().clone(),*stream.key(),1_u128<<64).unwrap());
    assert!(combat.step_full(&CombatAction::PlayCard{instance:instance.0,target:None}).is_err());
    assert_eq!(combat.state.character.orb_slots,vec![Some(OrbKind::Lightning.into()),Some(OrbKind::Lightning.into()),None]);
    assert!(combat.state.pending.iter().any(|e|matches!(e.kind,EffectKind::Character(CharacterEffect::ChannelState{..}))));
    assert!(combat.state.fault.is_some());let frozen=combat.clone();
    assert!(combat.step_full(&CombatAction::EndTurn).is_err());assert_eq!(combat,frozen);
    for orb in [OrbState{kind:OrbKind::Frost,dark_damage:Some(6)},OrbState{kind:OrbKind::Dark,dark_damage:None}]{
        let mut invalid=defect();let before=invalid.state.character.clone();
        assert!(super::super::characters::channel(&mut invalid.state,&Source::Actor(Actor::Player),orb,true).is_err());assert_eq!(invalid.state.character,before);
    }
    let mut invalid=defect();invalid.state.character.orb_slots=vec![None;11];let before=invalid.state.character.clone();
    assert!(super::super::characters::channel(&mut invalid.state,&Source::Actor(Actor::Player),OrbKind::Frost.into(),true).is_err());assert_eq!(invalid.state.character,before);
}

#[test]
fn m16_orbs_public_projection_excludes_hidden_order_and_rng(){
    let mut combat=defect();
    for instance in 20..24{super::super::piles::insert_generated(&mut combat.state,permanent_live_card(crate::content::CardId::Zap,CardInstanceId(instance),0).copy_as(CardInstanceId(instance)),CardDestination::Draw).unwrap();}
    let public=combat.public_full().unwrap();combat.state.piles.draw.reverse();
    let stream=combat.state.card_rng.as_ref().unwrap();combat.state.card_rng=Some(rng::StreamState::new(stream.id().clone(),[43;32],18).unwrap());
    for enemy in &mut combat.state.enemies{if let Some(stream)=&enemy.full_ai_rng{enemy.full_ai_rng=Some(rng::StreamState::new(stream.id().clone(),[91;32],20).unwrap());}}
    assert_eq!(combat.public_full().unwrap(),public);
}
