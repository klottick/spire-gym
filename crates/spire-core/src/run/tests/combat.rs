use crate::{combat::*, content::{CardId, EncounterId, RelicId}, run::*};
use crate::run::CardInstance;
mod support {
    use crate::{combat::rng::Partition,content::{CardId,Character,PotionId,RelicId},run::*};
    pub fn config() -> RunConfig { RunConfig {character:Character::Ironclad,ascension:0} }
    pub fn rng(seed:u64) -> RunRng { let mut key=[0;32]; key[..8].copy_from_slice(&seed.to_le_bytes()); RunRng::new(key,Partition::Test,0) }
    pub fn start() -> AuthoredRunStart {
        AuthoredRunStart {hp:40,max_hp:80,gold:123,
            deck:[CardId::Strike,CardId::Bash,CardId::Anger,CardId::Bloodletting,CardId::Disarm].into_iter()
                .enumerate().map(|(i,id)| CardInstance {instance:i as u64+1,id,upgrades:u32::from(i<2)}).collect(),
            relics:vec![RelicId::BurningBlood,RelicId::BloodVial,RelicId::Lantern,RelicId::Anchor],
            potions:vec![Some(PotionId::StrengthPotion),Some(PotionId::FirePotion),Some(PotionId::BloodPotion)],relic_counters:Default::default()}
    }
}

#[test]
fn real_entry_moves_inventory_and_runs_startup_once() {
    let rng=support::rng(7);
    let mut run=Run::from_authored_combat(support::config(),support::start(),rng.clone()).unwrap();
    assert_eq!(run.state().rng,rng,"authored construction doesn't sample Neow");
    run.enter_combat(EncounterId::JawWorm).unwrap();
    assert!(run.state().relics.is_empty() && run.state().potions.is_empty());
    let public=run.state().combat.as_ref().unwrap().public_full().unwrap();
    assert_eq!((public.player.hp,public.player.energy,public.player.max_energy,public.player.block),(42,4,3,10));
    assert_eq!(run.state().combat_progress,CombatProgress {entered:1,settled:0});
    let before=run.clone();
    assert_eq!(run.state().combat.as_ref().unwrap().public_full().unwrap(),public);
    assert_eq!(run,before);
}

#[test]
fn startup_fault_keeps_transferred_owner_and_cannot_replay() {
    let mut start=support::start();
    start.relics=vec![RelicId::PureWater];
    start.deck=(1..=999).map(|instance| CardInstance {instance,id:CardId::Strike,upgrades:0}).collect();
    let mut run=Run::from_authored_combat(support::config(),start,support::rng(7)).unwrap();
    let error=run.enter_combat(EncounterId::JawWorm).unwrap_err();
    assert!(matches!(&error,RunError::Faulted(_)));
    assert_eq!(run.state().combat.as_ref().unwrap().item_context().unwrap().relics(),&[RelicId::PureWater]);
    assert!(run.state().relics.is_empty());
    assert_eq!(run.state().combat_progress,CombatProgress {entered:1,settled:0});
    let before=run.clone();
    assert_eq!(run.enter_combat(EncounterId::JawWorm),Err(error.clone()));
    assert_eq!(run.choose_combat(CombatAction::EndTurn),Err(error));
    assert_eq!(run,before);
}

#[test]
fn settlement_conflict_retains_terminal_owner_and_opaque_counters() {
    let mut start=support::start();
    // Opaque legacy storage marker only, not a claim Burning Blood uses a counter.
    start.relic_counters.insert(RelicId::BurningBlood,7);
    let mut run=Run::from_authored_combat(support::config(),start,support::rng(7)).unwrap();
    run.enter_combat(EncounterId::JawWorm).unwrap();
    for action in [CombatAction::PlayCard {instance:5,target:Some(TargetId(1))},
        CombatAction::PlayCard {instance:4,target:None}, CombatAction::UsePotion {slot:0,target:None},
        CombatAction::PlayCard {instance:3,target:Some(TargetId(1))},
        CombatAction::PlayCard {instance:2,target:Some(TargetId(1))},
        CombatAction::PlayCard {instance:1,target:Some(TargetId(1))}] {run.choose_combat(action).unwrap();}
    // Named competing-inventory fault state: a second owner must never overwrite
    // the real terminal inventory or make an accepted kill retryable.
    run.state_mut_for_test().relics.push(RelicId::Anchor);
    let error=run.choose_combat(CombatAction::UsePotion {slot:1,target:Some(TargetId(1))}).unwrap_err();
    assert!(matches!(&error,RunError::Faulted(_)));
    let combat=run.state().combat.as_ref().unwrap().public_full().unwrap();
    assert_eq!((combat.outcome,combat.player.hp),(Some(Outcome::Victory),45));
    assert_eq!(combat.inventory.unwrap().potions,vec![None,None,Some(crate::content::PotionId::BloodPotion)]);
    assert_eq!(run.state().combat_progress,CombatProgress {entered:1,settled:0});
    assert!(run.state().relic_counters.is_empty());
    assert_eq!(run.state().combat.as_ref().unwrap().item_context().unwrap().counters().get(&RelicId::BurningBlood),Some(&7));
    let before=run.clone();
    assert_eq!(run.choose_combat(CombatAction::EndTurn),Err(error));
    assert_eq!(run,before);
}

#[test]
fn m6_stasis_run_death_settlement_counts_living_orb_card_once() {
    let mut start=support::start();start.deck=vec![CardInstance{instance:1,id:CardId::Disarm,upgrades:1}];start.relics.clear();start.potions=vec![None;3];
    let mut run=Run::from_authored_combat(support::config(),start,support::rng(41)).unwrap();let deck=run.state().deck.clone();
    let entry=enemies::setup(EncounterId::BronzeAutomaton,2,0,&mut RunRng::new([41;32],crate::combat::rng::Partition::Test,0),enemies::SetupContext{
        player:PlayerState{hp:200,max_hp:200,block:0,energy:3,max_energy:3,draw_per_turn:1,powers:vec![]},player_powers:vec![],character:crate::content::Character::Ironclad,
        cards:vec![permanent_live_card(CardId::Disarm,CardInstanceId(1),1)],opening_draw:0,encounter_visit:0,
    }).unwrap();
    let mut items=Some(crate::items::CombatItemContext::take_from_run(run.state_mut_for_test()));let mut combat=Combat::enter_with_item_context(entry,&mut items).unwrap();assert!(items.is_none());
    for _ in 0..5{combat.step_full(&CombatAction::EndTurn).unwrap();if !combat.public_full().unwrap().stasis.is_empty(){break}}
    let public=combat.public_full().unwrap();assert_eq!(public.stasis.len(),1);assert_eq!(public.stasis[0].card.instance,CardInstanceId(1));assert!(public.enemies.iter().any(|e|e.instance_id==public.stasis[0].orb&&e.hp>0));
    combat.enqueue_native_effects(vec![Effect{source:Source::None,target:Some(Actor::Player),kind:EffectKind::Damage(DamagePayload::HpLoss(300))}],None).unwrap();assert_eq!(combat.exit().unwrap().stasis.len(),1);
    let state=run.state_mut_for_test();state.combat=Some(combat);state.active_combat=Some(ActiveCombat{visit:0,encounter:EncounterId::BronzeAutomaton,origin:CombatOrigin::Authored});state.combat_progress=CombatProgress{entered:1,settled:0};
    let result=run.settle_combat().unwrap();assert_eq!(result.terminal,Some(RunTerminal::Death));assert_eq!(run.state().deck,deck);assert_eq!(run.state().last_combat.as_ref().unwrap().outcome,Outcome::Death);assert_eq!(run.state().combat_progress,CombatProgress{entered:1,settled:1});assert!(run.state().combat.is_none());
}
