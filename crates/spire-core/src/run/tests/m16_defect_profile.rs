use crate::{run::*,content::{CardId,Character,EncounterId,RelicId},combat::rng::Partition};
#[test]
fn m16_defect_profile_canonical_and_supplied_admission(){
    let rng=RunRng::new([7;32],Partition::Test,0);
    let mut run=Run::from_m16_defect_combat(M16DefectCombatStart::Canonical,rng.clone()).unwrap();
    let d=run.decision().unwrap();
    assert_eq!((d.player.character,d.player.hp,d.player.max_hp,d.player.gold),(Character::Defect,75,75,99));
    assert_eq!(d.player.relics,vec![RelicId::CrackedCore]);assert_eq!(d.player.potions,vec![None;3]);assert_eq!(d.player.keys,[false;3]);
    assert_eq!(d.objects.iter().filter(|c|c.id==CardId::StrikeBlue).count(),4);
    assert_eq!(d.objects.iter().filter(|c|c.id==CardId::DefendBlue).count(),4);
    assert_eq!(d.objects.len(),10);assert!(d.content_profile.is_none());
    run.enter_combat(EncounterId::Cultist).unwrap();
    assert_eq!(run.decision().unwrap().combat.unwrap().character.orbs,vec![Some(crate::combat::OrbKind::Lightning),None,None]);
    let start=AuthoredRunStart{hp:75,max_hp:75,gold:99,deck:vec![CardInstance{id:CardId::GeneticAlgorithm,instance:1,upgrades:0,misc:1}],relics:vec![],potions:vec![None;3],relic_counters:Default::default()};
    assert!(Run::from_m16_defect_combat(M16DefectCombatStart::Supplied(start.clone()),rng.clone()).is_ok());
    for bad in [CardId::Unknown("unsupported"),CardId::Leap,CardId::Strike] {
        let mut bad_start=start.clone();bad_start.deck[0].id=bad;
        assert!(Run::from_m16_defect_combat(M16DefectCombatStart::Supplied(bad_start),rng.clone()).is_err());
    }
    let mut duplicate=start.clone();duplicate.deck.push(duplicate.deck[0].clone());
    assert!(Run::from_m16_defect_combat(M16DefectCombatStart::Supplied(duplicate),rng).is_err());
}

#[test]
fn m16_defect_profile_frozen_act1_metadata_and_real_elite_context(){
    let rng=RunRng::new([7;32],Partition::Test,0);
    assert_eq!(crate::combat::enemies::A0_ENCOUNTER_SETUPS[..M16_DEFECT_COMBAT_SETUP_PREFIX_LEN].iter().filter(|(_,act,_)|*act==1).count(),20);
    let mut run=Run::from_m16_defect_combat(M16DefectCombatStart::Canonical,rng.clone()).unwrap();
    run.state.relics.push(RelicId::SlaversCollar);
    run.enter_combat(EncounterId::Lagavulin).unwrap();
    let c=run.decision().unwrap().combat.unwrap();
    assert_eq!(c.enemies[0].enemy_id,crate::content::EnemyId::Lagavulin);
    assert_eq!(c.player.max_energy,4,"the real shared Elite kind activates Collar");
    assert_eq!(c.enemies[0].intent.as_str(),"sleep");
    for encounter in [EncounterId::SphericGuardian,EncounterId::ThreeDarklings,EncounterId::CorruptHeart,EncounterId::Unknown("future")]{
        let mut bad=Run::from_m16_defect_combat(M16DefectCombatStart::Canonical,rng.clone()).unwrap();let frozen=bad.clone();
        assert!(bad.enter_combat(encounter).is_err());assert_eq!(bad,frozen,"rejected setup changed resources/RNG");
    }
    for (act,ascension) in [(2,0),(1,1)]{
        let mut bad=Run::from_m16_defect_combat(M16DefectCombatStart::Canonical,rng.clone()).unwrap();bad.state.act=act;bad.state.ascension=ascension;let frozen=bad.clone();
        assert!(bad.enter_combat(EncounterId::Cultist).is_err());assert_eq!(bad,frozen);
    }
}
