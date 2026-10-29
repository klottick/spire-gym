use super::super::*;
use crate::{combat::{self,CardInstanceId,Combat},content::{CardId,Character,EncounterId,RelicId},items::{CombatItemContext,CombatStartupProfile}};
fn rng()->RunRng{RunRng::new([7;32],combat::rng::Partition::Test,0)}
#[test]
fn m21_watcher_entry_prepares_canonical_resources_without_progression() {
    let run=Run::prepare_m21_watcher_combat(M21WatcherCombatStart::Canonical,EncounterId::JawWorm,rng()).unwrap();
    let original=Run::new(RunConfig{character:Character::Watcher,ascension:0},CreationProfile::experienced(),rng()).unwrap();
    assert_eq!((run.state.hp,run.state.max_hp,run.state.gold),(72,72,99));
    assert_eq!(run.state.deck,original.state.deck);assert_eq!(run.state.relics,vec![RelicId::PureWater]);
    assert_eq!(run.state.potions,vec![None;3]);assert_eq!(run.state.keys,[false;3]);
    assert!(run.state.combat.is_none());assert!(run.state.selected_neow.is_none());
}
#[test]
fn m21_watcher_startup_pure_water_predraw_preserves_legacy_overflow() {
    let make=||{
        let mut random=rng();
        combat::enemies::setup(EncounterId::JawWorm,1,0,&mut random,combat::enemies::SetupContext{
            player:combat::PlayerState{hp:72,max_hp:72,block:0,energy:3,max_energy:3,draw_per_turn:5,powers:vec![]},
            player_powers:vec![],character:Character::Watcher,
            cards:(1..=10).map(|id|combat::permanent_live_card(CardId::DefendPurple,CardInstanceId(id),0)).collect(),
            opening_draw:10,encounter_visit:0,
        }).unwrap()
    };
    let mut items=Some(CombatItemContext::new(vec![RelicId::PureWater],vec![None;3]).with_startup_profile(CombatStartupProfile::M21Watcher));
    let c=Combat::enter_with_item_context(make(),&mut items).unwrap();
    let view=c.public_full().unwrap();assert_eq!(view.hand.len(),10);assert_eq!(view.draw_count,1);
    assert!(view.hand.iter().any(|card|card.id==CardId::Miracle));assert!(view.discard.is_empty());
    let mut legacy=Some(CombatItemContext::new(vec![RelicId::PureWater],vec![None;3]));
    let legacy=Combat::enter_with_item_context(make(),&mut legacy).unwrap().public_full().unwrap();
    assert_eq!(legacy.draw_count,0);assert!(legacy.hand.iter().all(|card|card.id==CardId::DefendPurple));
    assert_eq!(legacy.discard.iter().filter(|card|card.id==CardId::Miracle).count(),1);
}
#[test]
fn m21_watcher_items_teardrop_opens_calm_before_any_action(){
    let resources=AuthoredRunStart{hp:72,max_hp:72,gold:99,deck:vec![CardInstance{instance:31,id:CardId::DefendPurple,upgrades:0,misc:0}],relics:vec![RelicId::TeardropLocket],potions:vec![None;3],relic_counters:Default::default()};
    let mut run=Run::prepare_m21_watcher_combat(M21WatcherCombatStart::Supplied(resources),EncounterId::Cultist,rng()).unwrap();
    run.enter_training_combat(EncounterId::Cultist).unwrap();assert_eq!(run.decision().unwrap().combat.unwrap().character.stance,combat::Stance::Calm);
}
#[test]
fn m21_watcher_items_acquired_bark_reinitializes_held_potion_metadata_at_owned_entry(){
    use crate::content::PotionId;
    let resources=AuthoredRunStart{hp:72,max_hp:72,gold:99,deck:vec![CardInstance{instance:31,id:CardId::DefendPurple,upgrades:0,misc:0}],relics:vec![],potions:vec![Some(PotionId::StancePotion),Some(PotionId::Ambrosia),Some(PotionId::BottledMiracle)],relic_counters:Default::default()};
    let mut run=Run::prepare_m21_watcher_combat(M21WatcherCombatStart::Supplied(resources),EncounterId::Cultist,rng()).unwrap();
    crate::items::acquire_relic(&mut run.state,RelicId::SacredBark).unwrap();run.enter_training_combat(EncounterId::Cultist).unwrap();
    let items=run.state.combat.as_ref().unwrap().item_context().unwrap();
    assert_eq!((items.potion_potency(0),items.potion_potency(1),items.potion_potency(2)),(Some(0),Some(4),Some(4)));
}
fn supplied()->AuthoredRunStart{AuthoredRunStart{hp:72,max_hp:72,gold:99,deck:vec![CardInstance{instance:31,id:CardId::DefendPurple,upgrades:0,misc:0}],relics:vec![],potions:vec![None;3],relic_counters:Default::default()}}
#[test]
fn m21_watcher_entry_rejects_invalid_owned_resources_before_effects(){
    use crate::content::PotionId;
    let mut bad=Vec::new();
    let mut r=supplied();r.deck[0].id=CardId::Miracle;bad.push(r);
    let mut r=supplied();r.deck[0].id=CardId::Strike;bad.push(r);
    let mut r=supplied();r.deck[0].instance=0;bad.push(r);
    let mut r=supplied();r.deck.push(r.deck[0].clone());bad.push(r);
    let mut r=supplied();r.deck[0].upgrades=2;bad.push(r);
    let mut r=supplied();r.deck[0].misc=1;bad.push(r);
    let mut r=supplied();r.relics=vec![RelicId::PureWater;2];bad.push(r);
    let mut r=supplied();r.relics=vec![RelicId::CrackedCore];bad.push(r);
    let mut r=supplied();r.potions[0]=Some(PotionId::FirePotion);bad.push(r);
    let mut r=supplied();r.relic_counters.insert(RelicId::PureWater,1);bad.push(r);
    let mut r=supplied();r.potions.pop();bad.push(r);
    let mut r=supplied();r.hp=73;bad.push(r);
    let mut r=supplied();r.hp=0;bad.push(r);
    for resources in bad{assert!(Run::prepare_m21_watcher_combat(M21WatcherCombatStart::Supplied(resources),EncounterId::Cultist,rng()).is_err());}
}
#[test]
fn m21_watcher_entry_all57_use_existing_setup_and_inventory_owner(){
    assert_eq!(TRAINING_ENCOUNTERS_V2.len(),57);
    for (encounter,act,kind) in TRAINING_ENCOUNTERS_V2{
        assert_eq!(m21_watcher_combat_encounter_metadata(*encounter),Some((*act,*kind)));
        let mut run=Run::prepare_m21_watcher_combat(M21WatcherCombatStart::Supplied(supplied()),*encounter,rng()).unwrap();
        run.enter_training_combat(*encounter).unwrap_or_else(|e|panic!("{}: {e:?}",encounter.as_str()));
        assert!(run.state.relics.is_empty()&&run.state.potions.is_empty());
        let c=run.decision().unwrap().combat.unwrap();assert_eq!(c.mantra_gained,Some(0));assert!(c.character.orbs.is_empty());
        assert_eq!(run.state.combat_progress,CombatProgress{entered:1,settled:0});
    }
}
#[test]
fn m21_watcher_entry_settlement_census_discards_children_and_second_fight_resets(){
    use crate::combat::{CombatAction,Stance};
    let mut r=supplied();r.deck=vec![CardInstance{instance:31,id:CardId::MentalFortress,upgrades:0,misc:0},CardInstance{instance:32,id:CardId::Prostrate,upgrades:0,misc:0}];r.relics=vec![RelicId::PureWater];
    let mut run=Run::prepare_m21_watcher_combat(M21WatcherCombatStart::Supplied(r.clone()),EncounterId::JawWorm,rng()).unwrap();run.enter_training_combat(EncounterId::JawWorm).unwrap();
    run.step(&GameAction::Combat(CombatAction::PlayCard{instance:31,target:None})).unwrap();run.step(&GameAction::Combat(CombatAction::PlayCard{instance:32,target:None})).unwrap();
    assert_eq!(run.decision().unwrap().combat.unwrap().mantra_gained,Some(2));
    // Supplied diagnostic enemy-lethal packet exercises shared Victory/exit/settlement;
    // it is not a prescribed public-policy demonstration or canonical victory.
    super::m9_boss::hit(&mut run,0,999);
    run.settle_combat().unwrap();assert_eq!(run.state.deck,r.deck);assert_eq!(run.state.relics,r.relics);
    assert_eq!(run.state.combat_progress,CombatProgress{entered:1,settled:1});let settled=run.clone();assert!(run.choose_combat(CombatAction::EndTurn).is_err());assert_eq!(run,settled);
    run.enter_combat(EncounterId::JawWorm).unwrap();let view=run.decision().unwrap().combat.unwrap();
    assert_eq!((view.character.stance,view.mantra_gained),(Stance::Neutral,Some(0)));
    assert_eq!(view.hand.iter().filter(|c|c.id==CardId::Miracle).count(),1);assert_eq!(run.state.combat_progress.entered,2);
}
