//! Test-only constructed pile checkpoints, executing real Automaton/Orb moves.
use super::super::*;
pub(crate) fn capture_learned_genetic(c:&mut Combat,i:u64){
    assert_eq!(c.state.enemies[0].enemy_id,ActorEnemyId::Full(crate::content::EnemyId::BronzeAutomaton));
    let card=c.state.piles.hand.remove(c.state.piles.hand.iter().position(|v|v.full().unwrap().instance.0==i).unwrap());c.state.piles.draw.push(card);
    let boss=Actor::Enemy(c.state.enemies[0].instance_id.clone());c.enqueue_native_effects(vec![Effect{source:Source::Actor(boss.clone()),target:Some(boss),kind:EffectKind::EnemyMove}],None).unwrap();
    assert_eq!(c.state.enemies.len(),3);super::m6_stasis::capture(c,1);assert!(c.state.stasis.values().any(|v|v.full().unwrap().instance.0==i));
}
pub(crate) fn return_learned_genetic(c:&mut Combat,full:bool){
    if full{for _ in 0..10{c.enqueue_native_effects(vec![Effect{source:Source::None,target:Some(Actor::Player),kind:EffectKind::CreateCard{id:crate::content::CardId::Wound,destination:CardDestination::Hand,upgrades:0}}],None).unwrap();}}
    let orb=c.state.enemies[1].instance_id.clone();let hp=c.state.enemies[1].hp;
    c.enqueue_native_effects(vec![Effect{source:Source::None,target:Some(Actor::Enemy(orb)),kind:EffectKind::Damage(DamagePayload::HpLoss(hp))}],None).unwrap();assert!(c.state.stasis.is_empty());
}
pub(crate) fn capture_returned_genetic(c:&mut Combat,i:u64){
    // Empty draw, real discard fallback; the second actual Orb remains present.
    assert!(c.state.piles.discard.iter().any(|v|v.full().unwrap().instance.0==i));super::m6_stasis::capture(c,2);assert!(c.state.stasis.values().any(|v|v.full().unwrap().instance.0==i));
}
pub(crate) fn fresh_genetic_copy(c:&mut Combat,i:u64)->u64{
    let original=c.state.piles.hand.iter().find(|v|v.full().unwrap().instance.0==i).unwrap().full().unwrap().clone();
    super::super::piles::insert_generated(&mut c.state,original.stat_equivalent_copy(CardInstanceId(101)),CardDestination::Hand).unwrap();101
}
pub(crate) fn family_genetic_copy(c:&mut Combat,i:u64)->u64{
    let copy=super::super::piles::make_same_instance_family_copy_from_owner(&mut c.state,&super::super::card_state::handle(CardInstanceId(i)),CardInstanceId(102)).unwrap();super::super::piles::insert_generated(&mut c.state,copy,CardDestination::Hand).unwrap();102
}
pub(crate) fn acquired_item_scheduler(c:&mut Combat,blocked:bool){
    use crate::content::RelicId;
    let items=c.state.item_context.as_ref().unwrap();for id in [RelicId::DataDisk,RelicId::GoldPlatedCables,RelicId::EmotionChip]{assert!(items.owns_relic(id));}
    assert_eq!(c.state.character.focus,3);assert_eq!(c.state.character.orb_slots.len(),5);
    // Explicit supplied layout isolates timing in the real shared Act2 fight.
    c.state.character.orb_slots=vec![Some(OrbKind::Dark.into()),Some(OrbKind::Plasma.into()),Some(OrbKind::Frost.into()),None,None];
    if blocked{c.enqueue_native_effects(vec![Effect{source:Source::None,target:Some(Actor::Player),kind:EffectKind::GainBlock(1)}],None).unwrap();}
    let hp=c.state.player.hp;let enemy=Actor::Enemy(c.state.enemies[0].instance_id.clone());
    c.enqueue_native_effects(vec![Effect{source:Source::Actor(enemy),target:Some(Actor::Player),kind:EffectKind::Damage(DamagePayload::Fixed(1))}],None).unwrap();
    assert_eq!(hp-c.state.player.hp,if blocked{0}else{1});assert_eq!(c.state.item_context.as_ref().unwrap().emotion_chip_armed,!blocked);
    assert_eq!(c.state.character.orb_slots.iter().flatten().filter(|o|o.kind==OrbKind::Lightning).count(),if blocked{0}else{1});
    super::super::turns::schedule_next_player_turn(&mut c.state).unwrap();assert_eq!(super::super::engine::resolve(&mut c.state).status,TransitionStatus::Decision);
    assert_eq!(c.state.character.orb_slots[0].unwrap().dark_damage,Some(if blocked{15}else{33}),"Loop invokes front once; only Chip's whole group adds Cables' extra front");
    assert_eq!(c.state.player.energy,if blocked{4}else{5},"natural Plasma and optional Chip, independent of Focus3");
    assert!(!c.state.item_context.as_ref().unwrap().emotion_chip_armed);assert_eq!(c.state.character.focus,3,"DataDisk never reapplies during a subsequent turn");
}
pub(crate) fn cleanup_preserves_accepted_player_work(c:&mut Combat,i:u64){
    let player=Some(Actor::Player);let hp=c.state.player.hp;
    c.enqueue_native_effects(vec![Effect{source:Source::None,target:player.clone(),kind:EffectKind::Damage(DamagePayload::HpLoss(8))}],None).unwrap();
    let deaths=c.state.hook_events.iter().filter(|e|e.hook==Hook::Death).count();
    c.enqueue_native_effects(vec![
        Effect{source:Source::Actor(Actor::Player),target:player.clone(),kind:EffectKind::Character(CharacterEffect::EndTurn)},
        Effect{source:Source::None,target:player.clone(),kind:EffectKind::Damage(DamagePayload::HpLoss(1))},
        Effect{source:Source::None,target:player,kind:EffectKind::Heal(2)},
        Effect{source:Source::None,target:None,kind:EffectKind::MonsterStart},
    ],None).unwrap();
    assert_eq!(c.state.player.hp,hp-7,"accepted HP loss and heal resolve before native scheduler cleanup");assert_eq!(c.state.outcome,Some(Outcome::Victory));
    assert_eq!(c.state.hook_events.iter().filter(|e|e.hook==Hook::Death).count()-deaths,3);assert_eq!(c.state.hook_events.iter().filter(|e|e.hook==Hook::CombatSettled).count(),1);
    let exit=c.exit().unwrap();assert!(exit.stasis.is_empty());assert_eq!(exit.hand.iter().filter(|v|v.instance.0==i&&v.misc==3&&v.origin==CombatCardRef::Permanent(CardInstanceId(i))).count(),1);c.validate_permanent_mutations_for_run().unwrap();
}
