//! M34s-a: the displayed intent applies the player's Intangible cap (original `calculateDamage`
//! final-receive) and truncates Back Attack before the floor (R-a2/R-a3); the gameplay damage entries and the damage
//! applied are unchanged.
use super::super::*;
use crate::{content::{CardId,Character,EncounterId,PotionId},run::RunRng,items::CombatItemContext};
fn entered(character:Character,encounter:EncounterId,cards:&[CardId])->Combat{
    let entry=enemies::setup(encounter,if matches!(encounter,EncounterId::CorruptHeart|EncounterId::ShieldAndSpear){4}else{1},0,&mut RunRng::new([81;32],rng::Partition::Test,0),enemies::SetupContext{
        player:PlayerState{hp:999,max_hp:999,block:0,energy:9,max_energy:9,draw_per_turn:5,powers:vec![]},player_powers:vec![],character,
        cards:cards.iter().enumerate().map(|(index,id)|complete_live_card(*id,CardInstanceId(index as u64+1),0).unwrap()).collect(),opening_draw:cards.len(),encounter_visit:0}).unwrap();
    let mut items=Some(CombatItemContext::new(vec![],vec![(character==Character::Silent).then_some(PotionId::GhostInAJar),None,None]).with_complete_cards().with_complete_items());
    Combat::enter_with_item_context(entry,&mut items).unwrap()
}
fn intangible(c:&Combat)->i32{c.state.player_full_powers.iter().find(|p|p.id.0=="intangible_player").map_or(0,|p|p.state.amount)}
fn first_attack(c:&mut Combat){
    for _ in 0..4{if c.state.enemies[0].intent_damage.is_some_and(|d|d>1){return}c.step_full(&CombatAction::EndTurn).unwrap();}
    panic!("no attack intent above 1 within four turns");
}
#[test]
fn intent_display_applies_player_intangible_cap(){
    for (character,card) in [(Character::Ironclad,CardId::Defend),(Character::Silent,CardId::Deflect),(Character::Defect,CardId::DefendBlue),(Character::Watcher,CardId::DefendPurple)]{
        for encounter in [EncounterId::CorruptHeart,EncounterId::JawWorm]{
            let mut c=entered(character,encounter,&[card,card,card,card,card]);first_attack(&mut c);
            let raw=c.state.enemies[0].intent_damage.unwrap();
            assert_eq!(c.public_full().unwrap().enemies[0].intent_damage,Some(raw),"{character:?} {encounter:?}: no Intangible, no cap");
            if character==Character::Silent{c.step_full(&CombatAction::UsePotion{slot:0,target:None}).unwrap();}
            else{c.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::ApplyFullPower{id:FullPowerId("intangible_player"),amount:1}}],None).unwrap();}
            assert!(intangible(&c)>0);
            assert_eq!(c.state.enemies[0].intent_damage,Some(raw),"{character:?} {encounter:?}: the gameplay output is unchanged");
            let view=c.public_full().unwrap();
            assert_eq!(view.enemies[0].intent_damage,Some(1),"{character:?} {encounter:?}: the displayed intent is capped at 1");
            assert_eq!(view.enemies[0].intent_hits,c.public_full().unwrap().enemies[0].intent_hits);
            let hits=view.enemies[0].intent_hits.unwrap_or(1) as i32;
            let before=c.state.player.hp;c.state.player.block=0;let owner=c.state.enemies[0].instance_id.clone();
            enemies::act(&mut c.state,&owner).unwrap();super::super::engine::resolve(&mut c.state);
            assert_eq!(c.state.player.hp,before-hits,"{character:?} {encounter:?}: each hit still applies the cap once");
            assert!(c.fault_record().is_none());
        }
    }
}
/// R-a2/R-a3: the original displays Back Attack as `(int)(tmp*1.5f)` on the un-floored value (Spear behind the player,
/// Weak 5 x 0.75 = 3.75 shows 5), while the dealt damage floors first (3, then x1.5 = 4). The view shows the original's
/// value; the gameplay output and the damage applied stay 4. Intangible then caps the display at 1.
#[test]
fn intent_display_back_attack_truncates_before_the_floor(){
    for character in Character::ALL{
        let mut c=entered(character,EncounterId::ShieldAndSpear,&[CardId::Strike;5]);
        let spear=c.state.enemies.iter().position(|e|e.enemy_id==ActorEnemyId::Full(crate::content::EnemyId::SpireSpear)).unwrap();
        for _ in 0..4{if c.state.enemies[spear].intent_damage.is_some(){break}c.step_full(&CombatAction::EndTurn).unwrap();}
        let owner=c.state.enemies[spear].instance_id.clone();
        assert!(effects::back_attack_active(&c.state,&c.state.enemies[spear]),"{character:?}: the Spear starts behind the player");
        let raw=c.state.enemies[spear].intent_damage.expect("a Spear attack intent within four turns");
        let base=match &c.state.enemies[spear].intent_id{ActorIntentId::Full(id)=>c.state.enemies[spear].full_damage_entries[id].base,_=>unreachable!()};
        assert_eq!(c.public_full().unwrap().enemies[spear].intent_damage,Some(raw),"{character:?}: an integral value shows the gameplay output");
        // The Spear's Artifact blocks the first debuff.
        for _ in 0..2{c.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Enemy(owner.clone())),kind:EffectKind::ApplyFullPower{id:FullPowerId("weak"),amount:2}}],None).unwrap();}
        assert!(c.state.enemies[spear].full_powers.iter().any(|p|p.id.0=="weak"),"{character:?}: the Spear is Weak");
        let output=c.state.enemies[spear].intent_damage.unwrap();
        assert_eq!(output,((base as f32*0.75).floor()*1.5) as i32,"{character:?}: gameplay floors, then Back Attack");
        let shown=(base as f32*0.75*1.5) as i32;
        assert_ne!(shown,output,"{character:?}: the fixture is a fractional case (base {base})");
        let view=c.public_full().unwrap();
        assert_eq!(view.enemies[spear].intent_damage,Some(shown),"{character:?}: the display truncates Back Attack before the floor");
        let hits=view.enemies[spear].intent_hits.unwrap_or(1) as i32;
        let before=c.state.player.hp;c.state.player.block=0;
        enemies::act(&mut c.state,&owner).unwrap();super::super::engine::resolve(&mut c.state);
        assert_eq!(before-c.state.player.hp,hits*output,"{character:?}: dealt damage is the gameplay output");
        assert!(c.fault_record().is_none());
    }
    let mut c=entered(Character::Ironclad,EncounterId::ShieldAndSpear,&[CardId::Strike;5]);
    let spear=c.state.enemies.iter().position(|e|e.enemy_id==ActorEnemyId::Full(crate::content::EnemyId::SpireSpear)).unwrap();
    for _ in 0..4{if c.state.enemies[spear].intent_damage.is_some(){break}c.step_full(&CombatAction::EndTurn).unwrap();}
    c.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::ApplyFullPower{id:FullPowerId("intangible_player"),amount:1}}],None).unwrap();
    assert_eq!(c.public_full().unwrap().enemies[spear].intent_damage,Some(1),"Intangible caps the Back Attack display");
}
