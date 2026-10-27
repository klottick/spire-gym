use spire_core::{combat::{rng::Partition,Outcome},content::*,run::*};
fn entry(encounter:EncounterId)->CombatTrainingEntry{CombatTrainingEntry{encounter,hp:50,max_hp:80,
    deck:vec![(CardId::Immolate,1),(CardId::Armaments,1),(CardId::Whirlwind,0),(CardId::Defend,0),(CardId::Strike,0),(CardId::Warcry,0)],
    relics:vec![RelicId::BurningBlood,RelicId::Anchor],potions:vec![Some(PotionId::Elixir),None,None],neows_lament_remaining:None}}
#[test]
fn training_full_entry_is_seeded_and_owns_broad_deck_startup(){
    let e=entry(EncounterId::ThreeSentries);let mut permuted=e.clone();permuted.deck.reverse();permuted.relics.reverse();
    let rng=||RunRng::new([6;32],Partition::Test,0);
    let mut a=Run::prepare_combat_training(e,rng()).unwrap();let mut b=Run::prepare_combat_training(permuted,rng()).unwrap();
    a.enter_training_combat(EncounterId::ThreeSentries).unwrap();b.enter_training_combat(EncounterId::ThreeSentries).unwrap();
    assert_eq!(a.decision(),b.decision());let d=a.decision().unwrap();let c=d.combat.unwrap();
    assert_eq!(c.enemies.len(),3);assert_eq!(c.player.block,10);assert_eq!(c.hand.len(),5);
    assert_eq!(d.objects.len(),6);assert!(d.candidates.iter().any(|c|matches!(c.action,GameAction::Combat(_))));
}
#[test]
fn training_act_and_boss_context_are_owned(){
    let mut e=entry(EncounterId::Champ);e.relics=vec![RelicId::SlaversCollar];
    let mut r=Run::prepare_combat_training(e,RunRng::new([7;32],Partition::Test,0)).unwrap();r.enter_training_combat(EncounterId::Champ).unwrap();
    let d=r.decision().unwrap();assert_eq!(d.act,2);assert_eq!(d.combat.unwrap().player.max_energy,4);
}
#[test]
fn training_rejects_invalid_public_resources_and_preserves_authored_limits(){
    let mut e=entry(EncounterId::Cultist);e.deck[0].1=2;assert!(Run::prepare_combat_training(e,RunRng::new([0;32],Partition::Test,0)).is_err());
    let mut e=entry(EncounterId::Cultist);e.deck[0].0=CardId::Neutralize;assert!(Run::prepare_combat_training(e,RunRng::new([0;32],Partition::Test,0)).is_err());
    let old=AuthoredRunStart{hp:50,max_hp:80,gold:0,deck:vec![CardInstance{id:CardId::Immolate,instance:1,upgrades:0}],relics:vec![],potions:vec![None;3],relic_counters:Default::default()};
    assert!(Run::from_authored_combat(RunConfig{character:Character::Ironclad,ascension:0},old,RunRng::new([0;32],Partition::Test,0)).is_err());
}
#[test]
fn training_victory_settles_once_with_cleanup_and_permanent_resources(){
    let mut e=entry(EncounterId::Cultist);e.deck=vec![(CardId::Immolate,1);5];e.potions=vec![None;3];
    let mut r=Run::prepare_combat_training(e,RunRng::new([7;32],Partition::Test,0)).unwrap();r.enter_training_combat(EncounterId::Cultist).unwrap();
    for _ in 0..100 {let d=r.decision().unwrap();if d.combat.is_none(){break} let a=d.candidates.iter().find(|c|matches!(c.action,GameAction::Combat(spire_core::combat::CombatAction::PlayCard{..}))).unwrap_or_else(||d.candidates.last().unwrap()).action.clone();r.step(&a).unwrap();}
    let cp=r.combat_checkpoint().unwrap();assert_eq!(cp.record.outcome,Outcome::Victory);assert_eq!(cp.hp_after_cleanup,cp.hp_before_healing+6);
    let d=r.decision().unwrap();assert!(d.combat.is_none());assert_eq!(d.player.hp,cp.hp_after_cleanup);assert_eq!(d.objects.len(),5);assert_eq!(d.combat_progress.settled,1);
}
#[test]
fn training_every_advertised_encounter_enters_and_settles_actual_death_or_victory(){
    assert_eq!(TRAINING_ENCOUNTERS.len(),43);
    for (encounter,act,_) in TRAINING_ENCOUNTERS {
        let mut e=entry(*encounter);e.hp=1;e.deck=vec![(CardId::Strike,0);5];e.relics=vec![];e.potions=vec![None;3];
        let mut r=Run::prepare_combat_training(e,RunRng::new([11;32],Partition::Test,0)).unwrap();
        r.enter_training_combat(*encounter).unwrap_or_else(|e|panic!("{encounter:?}: {e:?}"));
        assert_eq!(r.decision().unwrap().act,*act);
        for _ in 0..100 {if r.decision().unwrap().combat.is_none(){break}r.choose_combat(spire_core::combat::CombatAction::EndTurn).unwrap_or_else(|e|panic!("{encounter:?}: {e:?}"));}
        let cp=r.combat_checkpoint().unwrap_or_else(||panic!("{encounter:?} did not end"));
        assert!(matches!(cp.record.outcome,Outcome::Death|Outcome::Victory));
        assert_eq!(r.decision().unwrap().player.hp,cp.hp_after_cleanup);
    }
}
#[test]
fn training_all_red_base_and_upgrade_constructors_support_actual_play_and_settlement(){
    assert_eq!(TRAINING_CARDS.len(),75);
    for card in TRAINING_CARDS {for upgrade in [0,1] {
        let mut e=entry(EncounterId::Cultist);e.hp=80;e.deck=vec![(*card,upgrade);5];e.relics=vec![RelicId::MarkOfPain];e.potions=vec![None;3];
        let mut r=Run::prepare_combat_training(e,RunRng::new([11;32],Partition::Test,0)).unwrap();r.enter_training_combat(EncounterId::Cultist).unwrap();
        let d=r.decision().unwrap();let action=d.candidates.iter().find(|c|matches!(c.action,GameAction::Combat(spire_core::combat::CombatAction::PlayCard{..}))).unwrap_or_else(||panic!("{card:?}+{upgrade} not playable")).action.clone();
        r.step(&action).unwrap_or_else(|e|panic!("{card:?}+{upgrade}: {e:?}"));
        for _ in 0..100 {let d=r.decision().unwrap();let Some(combat)=d.combat else{break};let action=if combat.selection.is_some(){d.candidates[0].action.clone()}else{GameAction::Combat(spire_core::combat::CombatAction::EndTurn)};r.step(&action).unwrap_or_else(|e|panic!("{card:?}+{upgrade}: {e:?}"));}
        assert!(r.combat_checkpoint().is_some(),"{card:?}+{upgrade} never ended");
    }}
}
#[test]
fn training_cleanup_hp_checkpoint_handles_flower_bloom_cap_and_no_acquisition_replay(){
    for (extra,hp,max_hp,expected) in [(vec![RelicId::MagicFlower],50,80,59),(vec![RelicId::MarkOfBloom],50,80,50),(vec![RelicId::MagicFlower],78,80,80),(vec![RelicId::Strawberry,RelicId::EmptyCage],50,80,56)] {
        let mut e=entry(EncounterId::Cultist);e.hp=hp;e.max_hp=max_hp;e.deck=vec![(CardId::Immolate,1);5];e.potions=vec![None;3];e.relics=vec![RelicId::BurningBlood];e.relics.extend(extra);
        let mut r=Run::prepare_combat_training(e,RunRng::new([7;32],Partition::Test,0)).unwrap();r.enter_training_combat(EncounterId::Cultist).unwrap();
        assert_eq!(r.decision().unwrap().player.max_hp,max_hp);assert_eq!(r.decision().unwrap().objects.len(),5);
        for _ in 0..10 {let d=r.decision().unwrap();if d.combat.is_none(){break}r.step(&d.candidates[0].action).unwrap();}
        let cp=r.combat_checkpoint().unwrap();assert_eq!(cp.hp_before_healing,hp);assert_eq!(cp.hp_after_cleanup,expected);
    }
}

#[test]
fn encounter_setup_metadata_resolves_registered_species_and_existing_rng_keys(){
    use spire_core::combat::enemies::A0_ENCOUNTER_SETUPS;
    let catalog=Catalog::standard();
    let original=["cultist","jaw_worm","two_louse","small_slimes","large_slime","lots_of_slimes","slime_boss","three_cultists","jaw_worm_horde","gremlin_nob","lagavulin","three_sentries","three_louse","guardian","hexaghost","spheric_guardian","three_byrds","chosen_and_byrds","snecko","centurion_and_healer","gremlin_leader","slavers","book_of_stabbing","champ","collector","bronze_automaton","orb_walker","three_shapes","four_shapes","sphere_and_two_shapes","three_darklings","spire_growth","transient","maw","writhing_mass","giant_head","nemesis","reptomancer","awakened_one","time_eater","donu_and_deca","shield_and_spear","corrupt_heart"];
    assert_eq!(TRAINING_ENCOUNTERS.iter().map(|(id,_,_)|id.as_str()).collect::<Vec<_>>(),original);
    assert_eq!(A0_ENCOUNTER_SETUPS[43].0,EncounterId::BlueSlaver);
    assert_eq!(A0_ENCOUNTER_SETUPS[44].0,EncounterId::TwoFungiBeasts);
    for (encounter,_,_) in A0_ENCOUNTER_SETUPS {
        let definition=catalog.encounter(*encounter).unwrap_or_else(||panic!("missing {encounter:?} catalog metadata"));
        assert!(!definition.definition_only,"{encounter:?} is executable");
        for id in definition.enemies {assert!(!catalog.enemy(*id).unwrap_or_else(||panic!("missing {id:?} species metadata")).definition_only);}
    }
}

#[test]
fn training_v2_enters_added_encounters_without_widening_v1(){
    for (id,count) in [(EncounterId::BlueSlaver,1),(EncounterId::TwoFungiBeasts,2)] {
        let e=entry(id);
        assert!(Run::prepare_combat_training(e.clone(),RunRng::new([11;32],Partition::Test,0)).is_err());
        let mut run=Run::prepare_combat_training_v2(e,RunRng::new([11;32],Partition::Test,0)).unwrap();
        run.enter_training_combat(id).unwrap();
        let d=run.decision().unwrap();assert_eq!(d.act,1);assert_eq!(d.combat.as_ref().unwrap().enemies.len(),count);
        run.choose_combat(spire_core::combat::CombatAction::EndTurn).unwrap();
        assert_eq!(run.decision().unwrap().combat_progress.entered,1);
    }
}
