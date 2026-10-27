use spire_core::{combat::{CombatAction,Outcome,rng::Partition},content::*,run::*};

fn entry(encounter:EncounterId)->CombatTrainingEntry{CombatTrainingEntry{encounter,hp:1,max_hp:80,
    deck:vec![(CardId::Strike,0);5],relics:vec![],potions:vec![None;3],neows_lament_remaining:None}}

#[test]
fn all_current_standard_encounters_construct_real_actors_and_reach_a_natural_endpoint(){
    let ids=[EncounterId::RedSlaver,EncounterId::Looter,EncounterId::GremlinGang,EncounterId::ExordiumThugs,
        EncounterId::ExordiumWildlife,EncounterId::Chosen,EncounterId::ShelledParasite,EncounterId::TwoThieves,
        EncounterId::SentryAndSphere,EncounterId::SnakePlant,EncounterId::CultistAndChosen,EncounterId::ShelledParasiteAndFungi];
    let mut errors=Vec::new();
    for id in ids {
        let mut run=match Run::prepare_combat_training_v2(entry(id),RunRng::new([131;32],Partition::Test,0)){
            Ok(run)=>run,Err(error)=>{errors.push(format!("{id:?}: {error:?}"));continue}
        };
        if let Err(error)=run.enter_training_combat(id){errors.push(format!("{id:?}: {error:?}"));continue}
        assert!(!run.decision().unwrap().combat.unwrap().enemies.is_empty());
        for _ in 0..100 {
            if run.combat_checkpoint().is_some(){break}
            run.choose_combat(CombatAction::EndTurn).unwrap_or_else(|error|panic!("{id:?}: {error:?}"));
        }
        assert!(matches!(run.combat_checkpoint().unwrap_or_else(||panic!("{id:?}: no endpoint")).record.outcome,Outcome::Death|Outcome::Victory));
    }
    assert!(errors.is_empty(),"current standard setup gaps: {errors:?}");
}
