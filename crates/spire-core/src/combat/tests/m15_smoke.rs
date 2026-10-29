use super::super::*;
use crate::content::{Character,CardId,EncounterId,RelicId};
fn smoke()->crate::content::PotionId{crate::content::PotionId::SmokeBomb}
fn fight(encounter:EncounterId,relics:Vec<RelicId>,elite:bool)->Combat{
    let act=enemies::A0_ENCOUNTER_SETUPS.iter().find(|(id,_,_)|*id==encounter).unwrap().1;
    let entry=enemies::setup(encounter,act,0,&mut crate::run::RunRng::new([43;32],rng::Partition::Test,0),enemies::SetupContext{
        player:PlayerState{hp:300,max_hp:300,block:0,energy:3,max_energy:3,draw_per_turn:1,powers:vec![]},player_powers:vec![],character:Character::Silent,
        cards:vec![permanent_live_card(CardId::DefendGreen,CardInstanceId(1),0)],opening_draw:1,encounter_visit:0,
    }).unwrap();
    let context=crate::items::CombatItemContext::new(relics,vec![Some(smoke())]).with_battle_context(crate::items::NativeBattleContext{elite_trigger:elite,boss:false,burning:None,m15:true,m20:false});
    Combat::enter_with_item_context(entry,&mut Some(context)).unwrap()
}
#[test]
fn m15_smoke_consumes_real_slot_and_ends_without_defeating_monsters(){
    let mut c=fight(EncounterId::ThreeShapes,vec![],false);let hp=c.state.enemies.iter().map(|e|e.hp).collect::<Vec<_>>();
    c.step_full(&CombatAction::UsePotion{slot:0,target:None}).unwrap();
    assert_eq!(format!("{:?}",c.state.outcome),"Some(Escaped)");
    assert_eq!(c.state.enemies.iter().map(|e|e.hp).collect::<Vec<_>>(),hp);
    assert_eq!(c.item_context().unwrap().potions()[0],None);assert_eq!(c.state.phase,Phase::Terminal);assert!(c.state.pending.is_empty());
    engine::validate_runtime(&c.state).unwrap();
    assert!(c.exit().is_ok());let frozen=c.snapshot();assert!(c.step_full(&CombatAction::EndTurn).is_err());assert_eq!(c.snapshot(),frozen);
}
#[test]
fn m15_smoke_rejects_boss_and_any_retained_back_attack_owner_atomically(){
    let mut c=fight(EncounterId::TimeEater,vec![],false);let frozen=c.snapshot();assert!(c.step_full(&CombatAction::UsePotion{slot:0,target:None}).is_err());assert_eq!(c.snapshot(),frozen);
    let mut c=fight(EncounterId::ThreeShapes,vec![],false);
    c.state.enemies[1].full_powers.push(FullPowerEntry{id:FullPowerId("back_attack"),state:PowerState{amount:1,skip_first:false,monster_source_grace:false}});
    let frozen=c.snapshot();assert!(c.step_full(&CombatAction::UsePotion{slot:0,target:None}).is_err());assert_eq!(c.snapshot(),frozen);
}
#[test]
fn m15_smoke_runs_shared_collar_cleanup_without_requiring_enemy_death(){
    let mut c=fight(EncounterId::ThreeShapes,vec![RelicId::SlaversCollar],true);assert_eq!(c.state.player.max_energy,4);
    c.step_full(&CombatAction::UsePotion{slot:0,target:None}).unwrap();assert_eq!(c.state.player.max_energy,3);
    engine::validate_runtime(&c.state).unwrap();
    assert!(c.state.hook_events.iter().any(|event|event.hook==Hook::CombatSettled));
}

#[test]
fn m15_smoke_keeps_committed_thief_debit_without_fabricating_recovery(){
    let mut c=fight(EncounterId::Looter,vec![],false);
    enemies::act1::thieves::initialize_gold(&mut c.state,100).unwrap();
    c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(c.current_combat_gold(),Some(85));
    c.step_full(&CombatAction::UsePotion{slot:0,target:None}).unwrap();
    assert_eq!(c.current_combat_gold(),Some(85));assert!(c.stolen_gold_rewards().is_empty());engine::validate_runtime(&c.state).unwrap();assert!(c.exit().is_ok());
}
#[test]
fn m15_smoke_scan_includes_dead_monsters_retaining_back_attack(){
    let mut c=fight(EncounterId::ThreeShapes,vec![],false);
    let target=c.state.enemies[0].instance_id.clone();
    c.state.enemies[0].full_powers.push(FullPowerEntry{id:FullPowerId("back_attack"),state:PowerState{amount:1,skip_first:false,monster_source_grace:false}});
    c.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Enemy(target)),kind:EffectKind::Damage(DamagePayload::Fixed(999))}],None).unwrap();
    assert_eq!(c.state.enemies[0].hp,0);let frozen=c.snapshot();
    assert!(c.step_full(&CombatAction::UsePotion{slot:0,target:None}).is_err());assert_eq!(c.snapshot(),frozen);
}
