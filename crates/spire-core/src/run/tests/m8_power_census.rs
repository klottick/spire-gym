use super::super::*;
use crate::run::CardInstance;
use crate::{combat::{self,*,rng::Partition},content::{CardId,Character,EncounterId,RelicId}};

#[test]
fn m8_power_zone_preserves_exact_run_instances_with_new_parasite_on_victory_and_death(){
    for death in [false,true]{
        let mut r=Run::from_authored_combat(RunConfig{character:Character::Ironclad,ascension:0},AuthoredRunStart{hp:88,max_hp:88,gold:99,deck:vec![CardInstance{ misc:0,instance:1,id:CardId::Defend,upgrades:0},CardInstance{ misc:0,instance:2,id:CardId::Defend,upgrades:0}],relics:vec![RelicId::BurningBlood],potions:vec![None;3],relic_counters:Default::default()},RunRng::new([41;32],Partition::Test,0)).unwrap();
        // Supplied native census diagnostic; do not widen the old M3 starter
        // profile or represent this deck/encounter as normal-start progression.
        r.state.deck[0].id=CardId::Inflame;
        let mut e=combat::enemies::setup(EncounterId::WrithingMass,3,0,&mut RunRng::new([41;32],Partition::Test,0),combat::enemies::SetupContext{player:PlayerState{hp:88,max_hp:88,block:0,energy:3,max_energy:3,draw_per_turn:1,powers:vec![]},player_powers:vec![],character:Character::Ironclad,cards:r.state.deck.iter().map(|c|permanent_live_card(c.id,CardInstanceId(c.instance),c.upgrades)).collect(),opening_draw:2,encounter_visit:0}).unwrap();e.setup.enemies[0].intent=FullIntentId::from_name("writhing_parasite");
        let mut items=Some(crate::items::CombatItemContext::take_from_run(&mut r.state));let c=Combat::enter_with_item_context(e,&mut items).unwrap();r.state.combat=Some(c);r.state.active_combat=Some(ActiveCombat{ applied_mutation_ordinal:0,visit:0,encounter:EncounterId::WrithingMass,origin:CombatOrigin::Authored,applied_gain_ordinal:0,acquired_instances:vec![]});r.state.combat_progress=CombatProgress{entered:1,settled:0};
        r.choose_combat(CombatAction::PlayCard{instance:1,target:None}).unwrap();r.choose_combat(CombatAction::EndTurn).unwrap();assert_eq!(r.state.deck[2].id,CardId::Parasite);
        let c=r.state.combat.as_mut().unwrap();assert_eq!(c.public_full().unwrap().played_powers[0].instance,CardInstanceId(1));let target=if death{Actor::Player}else{Actor::Enemy(c.public_full().unwrap().enemies[0].instance_id.clone())};c.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Player),target:Some(target),kind:EffectKind::Damage(DamagePayload::Fixed(2000))}],None).unwrap();r.settle_combat().unwrap();
        assert_eq!(r.state.deck,vec![CardInstance{ misc:0,instance:1,id:CardId::Inflame,upgrades:0},CardInstance{ misc:0,instance:2,id:CardId::Defend,upgrades:0},CardInstance{ misc:0,instance:3,id:CardId::Parasite,upgrades:0}]);assert_eq!(r.state.terminal,if death{Some(RunTerminal::Death)}else{None});assert_eq!(r.state.combat_progress.settled,1);
    }
}
