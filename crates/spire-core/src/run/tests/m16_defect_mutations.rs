use crate::{run::*,content::{CardId,EncounterId},combat::{Actor,CombatAction,DamagePayload,Effect,EffectKind,Source,rng::Partition}};
fn supplied(upgrades:u32)->Run{
    Run::from_m16_defect_combat(M16DefectCombatStart::Supplied(AuthoredRunStart{hp:75,max_hp:75,gold:99,
        deck:vec![CardInstance{id:CardId::GeneticAlgorithm,instance:1,upgrades,misc:1}],relics:vec![],potions:vec![None;3],relic_counters:Default::default()}),RunRng::new([7;32],Partition::Test,0)).unwrap()
}
#[test]
fn m16_defect_mutations_consecutive_death_fault_stasis(){
    for up in [0,1]{
        let learned=if up>0{4}else{3};
        let mut run=supplied(up);run.enter_combat(EncounterId::Cultist).unwrap();
        run.choose_combat(CombatAction::PlayCard{instance:1,target:None}).unwrap();
        assert_eq!(run.state.deck[0].misc,learned,"growth syncs at action time");
        let enemy=run.decision().unwrap().combat.unwrap().enemies[0].clone();
        run.state.combat.as_mut().unwrap().enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Enemy(enemy.instance_id)),kind:EffectKind::Damage(DamagePayload::HpLoss(enemy.hp))}],None).unwrap();
        run.settle_combat().unwrap();run.enter_combat(EncounterId::Cultist).unwrap();
        assert_eq!(run.decision().unwrap().combat.unwrap().hand[0].misc,learned);
        run.choose_combat(CombatAction::PlayCard{instance:1,target:None}).unwrap();
        assert_eq!(run.state.deck[0].misc,if up>0{7}else{5});
        let hp=run.decision().unwrap().player.hp as i32;
        run.state.combat.as_mut().unwrap().enqueue_native_effects(vec![Effect{source:Source::None,target:Some(Actor::Player),kind:EffectKind::Damage(DamagePayload::HpLoss(hp))}],None).unwrap();
        run.settle_combat().unwrap();assert_eq!(run.state.terminal,Some(RunTerminal::Death));
        assert_eq!(run.state.deck[0].misc,if up>0{7}else{5});
        let mut faulted=supplied(up);faulted.enter_combat(EncounterId::TwoLouse).unwrap();
        crate::combat::tests::m16_support::exhaust_juggernaut_rng(faulted.state.combat.as_mut().unwrap());
        let result=faulted.choose_combat(CombatAction::PlayCard{instance:1,target:None});
        assert!(matches!(result,Err(RunError::Faulted(_))),"actual injected continuation result: {result:?}");
        assert_eq!(faulted.state.deck[0].misc,learned,"later RNG fault retains accepted growth");
        let frozen=faulted.clone();assert!(faulted.choose_combat(CombatAction::PlayCard{instance:1,target:None}).is_err());assert_eq!(faulted,frozen);
    }
}

#[test]
fn m16_defect_mutations_forged_journal_preflight_is_atomic(){
    for mode in 0..7{
        let mut run=supplied(0);run.enter_combat(EncounterId::Cultist).unwrap();run.choose_combat(CombatAction::PlayCard{instance:1,target:None}).unwrap();
        let deck=run.state.deck.clone();
        crate::combat::tests::m16_support::corrupt_mutation_journal(run.state.combat.as_mut().unwrap(),mode);
        assert!(matches!(run.sync_permanent_gains(),Err(RunError::Faulted(_))),"forged mode {mode}");
        assert_eq!(run.state.deck,deck,"preflight failure changed no permanent owner, mode {mode}");
    }
}

#[test]
fn m16_defect_mutations_real_next_entry_resets_local_fields_only(){
    use crate::content::CardId::*;
    let start=AuthoredRunStart{hp:75,max_hp:75,gold:99,deck:[(GeneticAlgorithm,0),(SteamBarrier,0),(Streamline,0),(Claw,1),(Claw,1)].into_iter().enumerate().map(|(i,(id,upgrades))|CardInstance{id,upgrades,instance:i as u64+1,misc:if id==GeneticAlgorithm{1}else{0}}).collect(),relics:vec![],potions:vec![None;3],relic_counters:Default::default()};
    let mut run=Run::from_m16_defect_combat(M16DefectCombatStart::Supplied(start),RunRng::new([7;32],Partition::Test,0)).unwrap();run.enter_combat(EncounterId::Cultist).unwrap();
    for _ in 0..30{
        let d=run.decision().unwrap();let Some(c)=&d.combat else{break};
        let chosen=[GeneticAlgorithm,SteamBarrier,Streamline,Claw].into_iter().find_map(|id|d.candidates.iter().find_map(|candidate|
            match &candidate.action{GameAction::Combat(action @ CombatAction::PlayCard{instance,..}) if c.hand.iter().any(|card|card.instance.0==*instance&&card.id==id)=>Some(action.clone()),_=>None})).unwrap_or(CombatAction::EndTurn);
        run.choose_combat(chosen).unwrap();
    }
    assert_eq!(run.decision().unwrap().last_combat.unwrap().outcome,crate::combat::Outcome::Victory);
    run.enter_combat(EncounterId::Cultist).unwrap();let c=run.decision().unwrap().combat.unwrap();
    for card in &c.hand{match card.id{GeneticAlgorithm=>assert_eq!(card.misc,3),SteamBarrier=>assert_eq!(card.values.as_ref().unwrap().base_block,Some(6)),Streamline=>assert_eq!(card.effective_cost,Some(2)),Claw=>assert_eq!(card.values.as_ref().unwrap().base_damage,Some(5)),_=>unreachable!()}}
}
