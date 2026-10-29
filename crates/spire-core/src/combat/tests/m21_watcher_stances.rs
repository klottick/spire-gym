use super::super::*;
use crate::content::{CardColor,Catalog};

// A missing constructor or a constructor that ignores the source base/+ attack
// and block fields breaks the resulting physical card, not just its token list.
#[test]
fn m21_watcher_admission_and_pairs() {
    let rows = [
        ("flurry_of_blows", Some([4,6]), None),
        ("brilliance", Some([12,16]), None),
        ("windmill_strike", Some([7,10]), None),
        ("perseverance", None, Some([5,7])),
        ("sands_of_time", Some([20,26]), None),
        ("carve_reality", Some([6,10]), None),
        ("deceive_reality", None, Some([4,7])),
    ];
    for (token,damage,block) in rows {
        let row=Catalog::standard().cards().iter().find(|row|row.id.as_str()==token)
            .unwrap_or_else(||panic!("missing admitted Watcher card {token}"));
        assert_eq!(row.color,CardColor::Purple);
        for up in 0..=1 {
            let card=permanent_live_card(row.id,CardInstanceId(17),up);
            assert_eq!(card.local.base_damage,damage.map(|v|v[up as usize]),"{token}/{up}");
            assert_eq!(card.local.base_block,block.map(|v|v[up as usize]),"{token}/{up}");
        }
    }
}

fn watcher_fight()->Combat {
    let mut c=super::full_runtime::entered_combat(3,3, |_|{});
    c.state.silent.source_character=Some(crate::content::Character::Watcher);
    c
}
fn apply(c:&mut Combat,kind:EffectKind) {
    c.state.pending.push_back(Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind});
    let result=engine::resolve(&mut c.state);
    assert_ne!(result.status,TransitionStatus::Fault,"{:?}",c.fault_record());
}
#[test]
fn m21_watcher_stance_requests_are_not_transitions() {
    let mut c=watcher_fight();
    c.state.character.stance=Stance::Calm;
    c.state.player_full_powers.push(FullPowerEntry{id:FullPowerId("mental_fortress"),state:PowerState{amount:4,skip_first:false,monster_source_grace:false}});
    apply(&mut c,EffectKind::Character(CharacterEffect::Stance(Stance::Calm)));
    assert_eq!((c.state.player.energy,c.state.player.block),(3,0));
    apply(&mut c,EffectKind::Character(CharacterEffect::Stance(Stance::Neutral)));
    assert_eq!((c.state.player.energy,c.state.player.block),(5,4));
}
#[test]
fn m21_watcher_mantra_threshold_subtracts_once() {
    let mut c=watcher_fight();
    apply(&mut c,EffectKind::ApplyFullPower{id:FullPowerId("mantra"),amount:9});
    apply(&mut c,EffectKind::ApplyFullPower{id:FullPowerId("mantra"),amount:23});
    assert_eq!(c.state.player_full_powers.iter().find(|p|p.id.0=="mantra").unwrap().state.amount,22);
}
#[test]
fn m21_watcher_stance_divinity_uses_one_final_floor() {
    let mut c=watcher_fight();
    c.state.character.stance=Stance::Divinity;
    c.state.player_full_powers.push(FullPowerEntry{id:FullPowerId("weak"),state:PowerState{amount:1,skip_first:false,monster_source_grace:false}});
    c.state.enemies[0].full_powers.push(FullPowerEntry{id:FullPowerId("vulnerable"),state:PowerState{amount:1,skip_first:false,monster_source_grace:false}});
    let n=cards::normal_damage(&c.state,7,0,true,&c.state.enemies[0]).unwrap();
    assert_eq!(n,23);
}
#[test]
fn m21_watcher_mantra_prostrate_plays_real_card() {
    let mut c=super::full_runtime::entered_combat(3,3,|card|*card=permanent_live_card(crate::content::CardId::Prostrate,card.instance,1));
    c.state.silent.source_character=Some(crate::content::Character::Watcher);
    c.step_full(&CombatAction::PlayCard{instance:3,target:None}).unwrap();
    assert_eq!(c.state.player_full_powers.iter().find(|p|p.id.0=="mantra").unwrap().state.amount,3);
    assert_eq!((c.state.player.energy,c.state.player.block),(3,4));
}

#[test]
fn m21_watcher_finite_attacks_pay_and_hit_in_both_faces() {
    use crate::content::CardId::*;
    for (id,cost,damage) in [(FlurryOfBlows,0,[4,6]),(Weave,0,[4,6]),(Brilliance,1,[12,16]),(WindmillStrike,2,[7,10]),(SandsOfTime,4,[20,26])] {
        for up in 0..=1 {
            let mut c=super::full_runtime::entered_combat(9,9,|card|*card=permanent_live_card(id,card.instance,up));
            c.state.silent.source_character=Some(crate::content::Character::Watcher);
            c.state.enemies[0].hp=500;c.state.enemies[0].max_hp=500;
            c.step_full(&CombatAction::PlayCard{instance:3,target:Some(TargetId(1))}).unwrap_or_else(|e|panic!("{id:?}/{up}: {e:?}"));
            assert_eq!((c.state.enemies[0].hp,c.state.player.energy),(500-damage[up as usize],9-cost),"{id:?}/{up}");
        }
    }
}
#[test]
fn m21_watcher_powers_install_exact_amount_and_retire_physical_card() {
    use crate::content::CardId::*;
    for (id,key,cost,amount) in [(Devotion,"devotion",[1,1],[2,3]),(Foresight,"foresight",[1,1],[3,4]),(Nirvana,"nirvana",[1,1],[3,4]),(Establishment,"establishment",[1,1],[1,1]),(MasterReality,"master_reality",[1,0],[-1,-1]),(BattleHymn,"battle_hymn",[1,1],[1,1]),(MentalFortress,"mental_fortress",[1,1],[4,6]),(Rushdown,"rushdown",[1,0],[2,2]),(LikeWater,"like_water",[1,1],[5,7])] {
        for up in 0..=1 {
            let mut c=super::full_runtime::entered_combat(3,3,|card|*card=permanent_live_card(id,card.instance,up));
            c.state.silent.source_character=Some(crate::content::Character::Watcher);
            c.step_full(&CombatAction::PlayCard{instance:3,target:None}).unwrap_or_else(|e|panic!("{id:?}/{up}: {e:?}"));
            assert_eq!(c.state.player_full_powers.iter().find(|p|p.id.0==key).unwrap().state.amount,amount[up as usize]);
            assert_eq!(c.state.player.energy,3-cost[up as usize]);
            assert_eq!(c.state.played_powers.iter().filter(|c|c.full().is_some_and(|c|c.instance.0==3)).count(),1);
        }
    }
}
#[test]
fn m21_watcher_stance_fault_preserves_first_energy_and_queued_siblings(){
    let mut c=watcher_fight();c.state.character.stance=Stance::Calm;
    c.state.item_context=Some(crate::items::CombatItemContext::new(vec![crate::content::RelicId::VioletLotus],vec![None;3]).with_startup_profile(crate::items::CombatStartupProfile::M21Watcher));
    c.state.resolved_effects=99997;
    c.state.pending.push_back(Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::Character(CharacterEffect::Stance(Stance::Divinity))});
    assert_eq!(engine::resolve(&mut c.state).status,TransitionStatus::Fault);
    assert_eq!((c.state.character.stance,c.state.player.energy),(Stance::Divinity,4));
    assert!(matches!(c.state.current.as_ref().map(|e|&e.kind),Some(EffectKind::Watcher(cards::watcher::WatcherEffect::StanceEnergy{reason:cards::watcher::StanceEnergy::CalmExit}))));
    assert!(c.state.pending.iter().any(|e|matches!(e.kind,EffectKind::Watcher(cards::watcher::WatcherEffect::StanceEnergy{reason:cards::watcher::StanceEnergy::DivinityEntry}))));
    let before=c.clone();assert!(c.public_full().is_err());assert!(c.step_full(&CombatAction::EndTurn).is_err());assert_eq!(c,before);
}
#[test]
fn m21_watcher_mantra_history_survives_turns_and_distinct_first_install(){
    let mut c=watcher_fight();
    apply(&mut c,EffectKind::ApplyFullPower{id:FullPowerId("mantra"),amount:12});
    assert_eq!(c.state.character.stance,Stance::Neutral);
    assert_eq!(c.state.player_full_powers.iter().find(|p|p.id.0=="mantra").unwrap().state.amount,12);
    apply(&mut c,EffectKind::Watcher(cards::watcher::WatcherEffect::GainMantra{amount:3}));
    assert_eq!(c.state.character.mantra_gained,3);assert_eq!(c.state.character.stance,Stance::Divinity);
    c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(c.state.character.mantra_gained,3);assert_eq!(c.state.character.stance,Stance::Neutral);
}
#[test]
fn m21_watcher_powers_master_reality_keeps_source_nonstacking_sentinel(){
    let mut c=super::full_runtime::entered_combat(3,3,|card|*card=permanent_live_card(crate::content::CardId::MasterReality,card.instance,0));c.state.silent.source_character=Some(crate::content::Character::Watcher);
    c.step_full(&CombatAction::PlayCard{instance:3,target:None}).unwrap();
    assert_eq!(c.state.player_full_powers.iter().find(|p|p.id.0=="master_reality").unwrap().state.amount,-1);
}
#[test]
fn m21_watcher_powers_like_water_clamps_without_limiting_other_stacks(){
    let mut c=watcher_fight();
    for (name,initial,addition,expected) in [("like_water",998,7,999),("nirvana",998,4,1002),("devotion",998,3,1001),("mental_fortress",998,6,1004),("foresight",998,4,1002),("battle_hymn",998,1,999),("establishment",998,1,999),("rushdown",998,2,1000)]{
        c.state.player_full_powers.push(FullPowerEntry{id:FullPowerId(name),state:PowerState{amount:initial,skip_first:false,monster_source_grace:false}});
        apply(&mut c,EffectKind::ApplyFullPower{id:FullPowerId(name),amount:addition});
        assert_eq!(c.state.player_full_powers.iter().find(|p|p.id.0==name).unwrap().state.amount,expected);
    }
}
#[test]
fn m21_watcher_stance_wide_retained_damage_is_not_old_administrative_cap(){
    let mut c=super::full_runtime::entered_combat(3,3,|card|{*card=m21_watcher_live_card(crate::content::CardId::WindmillStrike,card.instance,0).unwrap();card.local.base_damage=Some(40_000);});
    c.state.silent.source_character=Some(crate::content::Character::Watcher);
    c.step_full(&CombatAction::PlayCard{instance:3,target:Some(TargetId(1))}).unwrap();assert_eq!(c.public_full().unwrap().outcome,Some(Outcome::Victory));
}
#[test]
fn m21_watcher_mantra_devotion_distinguishes_no_power_from_existing_remainder(){
    for existing in [false,true]{let mut c=watcher_fight();c.state.player_full_powers.push(FullPowerEntry{id:FullPowerId("devotion"),state:PowerState{amount:10,skip_first:false,monster_source_grace:false}});if existing{c.state.player_full_powers.push(FullPowerEntry{id:FullPowerId("mantra"),state:PowerState{amount:9,skip_first:false,monster_source_grace:false}});}
        c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(c.state.character.stance,Stance::Divinity);assert_eq!(c.state.character.mantra_gained,if existing{10}else{0});assert_eq!(c.state.player_full_powers.iter().find(|p|p.id.0=="mantra").map(|p|p.state.amount),if existing{Some(9)}else{None});
    }
}
#[test]
fn m21_watcher_stance_blocked_request_keeps_calm_and_all_resources(){let mut c=watcher_fight();c.state.character.stance=Stance::Calm;for name in ["cannot_change_stance","mental_fortress"]{c.state.player_full_powers.push(FullPowerEntry{id:FullPowerId(name),state:PowerState{amount:1,skip_first:false,monster_source_grace:false}});}apply(&mut c,EffectKind::Character(CharacterEffect::Stance(Stance::Wrath)));assert_eq!((c.state.character.stance,c.state.player.energy,c.state.player.block),(Stance::Calm,3,0));}
#[test]
fn m21_watcher_stance_normal_receive_only_wrath_and_direct_modes_unchanged(){
    let mut base=watcher_fight();effects::refresh_enemy_values(&mut base.state).unwrap();
    let neutral=base.state.enemies[0].full_damage_entries.clone();assert!(!neutral.is_empty());
    for stance in [Stance::Neutral,Stance::Calm,Stance::Wrath,Stance::Divinity]{
        base.state.character.stance=stance;effects::refresh_enemy_values(&mut base.state).unwrap();
        for (id,entry) in &base.state.enemies[0].full_damage_entries{assert_eq!(entry.output,neutral[id].output*if stance==Stance::Wrath{2}else{1});}
        let mut hp_loss=base.clone();let hp=hp_loss.state.player.hp;
        hp_loss.state.pending.push_back(Effect{source:Source::None,target:Some(Actor::Player),kind:EffectKind::Damage(DamagePayload::HpLoss(3))});
        assert_ne!(engine::resolve(&mut hp_loss.state).status,TransitionStatus::Fault);assert_eq!(hp_loss.state.player.hp,hp-3);
        let mut thorns=base.clone();let hp=thorns.state.enemies[0].hp;let enemy=thorns.state.enemies[0].instance_id.clone();
        thorns.state.pending.push_back(Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Enemy(enemy)),kind:EffectKind::Damage(DamagePayload::Thorns(3))});
        assert_ne!(engine::resolve(&mut thorns.state).status,TransitionStatus::Fault);assert_eq!(thorns.state.enemies[0].hp,hp-3);
    }
}
#[test]
fn m21_watcher_mantra_int32_history_exhaustion_is_administrative_fault(){
    let mut c=super::full_runtime::entered_combat(3,3,|card|*card=m21_watcher_live_card(crate::content::CardId::Prostrate,card.instance,0).unwrap());c.state.silent.source_character=Some(crate::content::Character::Watcher);c.state.character.mantra_gained=i32::MAX;
    assert!(matches!(c.step_full(&CombatAction::PlayCard{instance:3,target:None}),Err(crate::run::RunError::Faulted(_))));assert_eq!(c.state.character.mantra_gained,i32::MAX);assert_eq!(c.state.in_play.as_ref().unwrap().handle().as_str(),"3");
}
#[test]
fn m21_watcher_stance_nonrepresentable_final_float_is_an_administrative_error(){
    for (id,damage,block,target) in [(crate::content::CardId::WindmillStrike,Some(1_000_000_000),None,Some(TargetId(1))),(crate::content::CardId::Perseverance,None,Some(i32::MAX),None)]{
        let mut c=super::full_runtime::entered_combat(3,3,|card|{*card=m21_watcher_live_card(id,card.instance,0).unwrap();card.local.base_damage=damage;card.local.base_block=block;});c.state.silent.source_character=Some(crate::content::Character::Watcher);c.state.character.stance=Stance::Divinity;
        assert!(c.step_full(&CombatAction::PlayCard{instance:3,target}).is_err(),"nonrepresentable source float must not saturate into fabricated gameplay");assert!(c.state.outcome.is_none());
    }
}
