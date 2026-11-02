//! M27 Main seam reachability checks (A11).
use super::m27_support::complete_run;
use crate::combat::{Actor,CombatAction,DamagePayload,Effect,EffectKind,FullPowerId,GenerationCost,GenerationPool,GenerationWork,RelicEffect,Source,TargetId};
use crate::combat::tests::m27_state as combat_state;
use crate::content::{Character,EncounterId,RelicId,PotionId,CardType};
use crate::items::{dispatch::{self,ItemPhase},laws};
use crate::run::Run;
use std::collections::BTreeMap;

fn combat(run:&mut Run)->&mut crate::combat::Combat{
    run.state_mut_for_test().combat.as_mut().expect("complete Run has active combat")
}

#[test]
fn m27_seams_reached_and_laws_reach_their_owners(){
    let _=dispatch::take_phase_log();let _=laws::take_law_log();
    let mut run=complete_run(Character::Ironclad,vec![RelicId::PenNib,RelicId::ToyOrnithopter],
        vec![Some(PotionId::BlockPotion),None,None],BTreeMap::new(),EncounterId::Cultist);
    let entry_facts=dispatch::take_phase_fact_log();
    for phase in [ItemPhase::CombatStartPreDraw,ItemPhase::CombatStart]{
        assert!(entry_facts.iter().any(|(actual,_,elite,boss,curses)|*actual==phase&&!*elite&&!*boss&&*curses==0),
            "combat entry facts were not populated for {phase:?}: {entry_facts:?}");
    }
    let entry_phases=dispatch::take_phase_log();
    assert_eq!(entry_phases.iter().filter(|phase|**phase==ItemPhase::AfterOpeningDraw).count(),1);
    assert!(!entry_phases.contains(&ItemPhase::AfterDraw));

    {
        let combat=combat(&mut run);
        combat_state::set_player_hp(combat, 60);
        combat.public_full().unwrap();
        combat.step_full(&CombatAction::UsePotion{slot:0,target:None}).unwrap();
        let target=combat.public_full().unwrap().enemies[0].instance_id.clone();
        {
            combat_state::set_player_block(combat, 0);
            combat_state::set_enemy_block(combat, 0, 5);
        }
        combat.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Player),
            target:Some(Actor::Enemy(target.clone())),kind:EffectKind::Damage(DamagePayload::Fixed(5))}],None).unwrap();
        combat.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),
            kind:EffectKind::ApplyFullPower{id:FullPowerId("strength"),amount:1}}],None).unwrap();
        combat.enqueue_native_effects(vec![Effect{source:Source::Actor(Actor::Enemy(target.clone())),target:Some(Actor::Player),
            kind:EffectKind::Damage(DamagePayload::Fixed(2))}],None).unwrap();
        combat.enqueue_native_effects(vec![Effect{source:Source::None,target:Some(Actor::Player),
            kind:EffectKind::Damage(DamagePayload::HpLoss(1))}],None).unwrap();
        combat.enqueue_native_effects(vec![Effect{source:Source::None,target:Some(Actor::Player),
            kind:EffectKind::Heal(1)}],None).unwrap();
        combat.legal_actions_full().unwrap();

        loop {
            let hand=combat.public_full().unwrap().hand;
            let Some(card)=hand.first() else{break};
            let handle=card.instance;
            let definition=crate::content::Catalog::standard().card(card.id).unwrap();
            let target=if definition.card_type==CardType::Attack{Some(TargetId(1))}else{None};
            {
                combat_state::set_player_energy(combat, 99);
            }
            assert!(combat.exhaust_card_for_test(handle));
            combat.step_full(&CombatAction::PlayCard{instance:handle.0,target}).unwrap();
        }
        combat_state::set_player_block(combat, 3);
        combat.step_full(&CombatAction::EndTurn).unwrap();
        combat.public_full().unwrap();
        combat_state::set_player_hp(combat, 1);
        combat.enqueue_native_effects(vec![Effect{source:Source::None,target:Some(Actor::Player),
            kind:EffectKind::Damage(DamagePayload::HpLoss(1))}],None).unwrap();
    }

    let phases=dispatch::take_phase_log();
    for phase in [ItemPhase::PotionUsed,ItemPhase::BlockBroken,ItemPhase::PowerApplied,
        ItemPhase::HandEmptied,ItemPhase::AfterDraw]{assert!(phases.contains(&phase),"missing {phase:?}: {phases:?}");}
    let laws=laws::take_law_log();
    for law in ["outgoing_damage","unblocked_outgoing_damage","incoming_damage","hp_loss",
        "block_decay","would_die","heal_amount","play_allowed","repeat_play","would_exhaust",
        "player_vulnerable_factor","public_view"] {
        assert!(laws.contains(&law),"missing law {law}: {laws:?}");
    }
    let runtime_facts=dispatch::take_phase_fact_log();
    assert!(runtime_facts.iter().any(|(phase,card,_,_,_)|*phase==ItemPhase::CardPlayed&&card.is_some_and(|fact|fact.id==crate::content::CardId::Bash&&fact.cost_paid>0)),
        "played card facts retain identity and paid cost: {runtime_facts:?}");
    assert!(runtime_facts.iter().any(|(phase,card,_,_,_)|*phase==ItemPhase::AfterUse&&card.is_some()),
        "after-use facts retain the live card owner: {runtime_facts:?}");

    let _=dispatch::take_phase_log();
    let mut summons=complete_run(Character::Ironclad,vec![RelicId::PenNib],vec![None;3],BTreeMap::new(),EncounterId::Collector);
    combat(&mut summons).step_full(&CombatAction::EndTurn).unwrap();
    assert!(dispatch::take_phase_log().contains(&ItemPhase::EnemySpawned),"Collector adds a minion during combat");
}

#[test]
fn m27_addendum_barriers_relic_sources_effects_and_generation(){
    let mut barrier_run=complete_run(Character::Ironclad,vec![RelicId::PenNib],vec![None;3],BTreeMap::new(),EncounterId::Cultist);
    {
        let combat=combat(&mut barrier_run);
        let decision=Effect{source:Source::None,target:None,kind:EffectKind::Decision};
        combat_state::push_pending(combat, decision.clone());
        combat.queue_relic_work_for_test(crate::items::RelicHookWork{top:vec![],bottom:vec![Effect{
            source:Source::Relic(RelicId::PenNib),target:Some(Actor::Player),kind:EffectKind::Heal(1)}],direct_strength:0}).unwrap();
        let pending = combat_state::pending_effects(combat);
        assert!(matches!(pending.first().map(|effect|&effect.kind),Some(EffectKind::Heal(1))));
        assert_eq!(pending.last(),Some(&decision),"complete-item bottom work precedes the pending Decision");
    }

    let mut unowned=complete_run(Character::Ironclad,vec![RelicId::PenNib],vec![None;3],BTreeMap::new(),EncounterId::Cultist);
    let rejected=Effect{source:Source::Relic(RelicId::Enchiridion),target:None,
        kind:EffectKind::Relic(RelicEffect::CapEnemyHp{percent:50})};
    assert!(combat(&mut unowned).enqueue_native_effects(vec![rejected.clone()],None).is_err(),"unowned relic provenance rejects");

    let mut incomplete=complete_run(Character::Ironclad,vec![RelicId::PenNib],vec![None;3],BTreeMap::new(),EncounterId::Cultist);
    combat_state::set_item_context(combat(&mut incomplete), Some(crate::items::CombatItemContext::new(
        vec![RelicId::PenNib],vec![None;3])));
    let rejected_owned=Effect{source:Source::Relic(RelicId::PenNib),target:None,
        kind:EffectKind::Relic(RelicEffect::CapEnemyHp{percent:50})};
    assert!(combat(&mut incomplete).enqueue_native_effects(vec![rejected_owned],None).is_err(),"incomplete item context rejects relic provenance");

    let mut relic_run=complete_run(Character::Ironclad,vec![RelicId::PenNib],vec![None;3],BTreeMap::new(),EncounterId::Cultist);
    {
        let combat=combat(&mut relic_run);
        let before=combat.public_full().unwrap().hand.iter().map(|card|(card.instance,card.upgrades)).collect::<Vec<_>>();
        combat.enqueue_native_effects(vec![Effect{source:Source::Relic(RelicId::PenNib),target:None,
            kind:EffectKind::Relic(RelicEffect::UpgradeRandomHandCard)}],None).unwrap();
        let after=combat.public_full().unwrap().hand.iter().map(|card|(card.instance,card.upgrades)).collect::<Vec<_>>();
        assert_eq!(before.iter().filter(|(instance,upgrades)|after.iter().any(|(next,next_upgrades)|next==instance&&next_upgrades>upgrades)).count(),1,
            "random upgrade changes one eligible combat hand instance");
        let max_hp = combat_state::enemy_max_hp(combat, 0);
        combat_state::set_enemy_hp(combat, 0, max_hp);
        combat.enqueue_native_effects(vec![Effect{source:Source::Relic(RelicId::PenNib),target:None,
            kind:EffectKind::Relic(RelicEffect::CapEnemyHp{percent:50})}],None).unwrap();
        assert_eq!(combat_state::enemy_hp(combat, 0),max_hp*50/100,"HP cap floors the enemy's maximum-HP percentage");
    }

    let mut enchiridion=complete_run(Character::Ironclad,vec![RelicId::Enchiridion],vec![None;3],BTreeMap::new(),EncounterId::Cultist);
    let generated=combat(&mut enchiridion).enqueue_native_effects(vec![Effect{source:Source::Relic(RelicId::Enchiridion),target:Some(Actor::Player),
        kind:EffectKind::Generate(GenerationWork{pool:GenerationPool::Power,count:1,unique:false,destination:crate::combat::CardDestination::Hand,
            cost:GenerationCost::TurnZero,upgraded:false,offer:None,captured:false,cards:vec![],next:0})}],None);
    generated.unwrap();
    assert!(combat(&mut enchiridion).public_full().unwrap().hand.iter().any(|card|
        crate::content::Catalog::standard().card(card.id).is_some_and(|definition|definition.card_type==CardType::Power)
            &&card.effective_cost==Some(0)),"Enchiridion creates a Power with this-turn cost zero");
}

#[test]
fn m27_seams_inert_without_items(){
    let mut run=complete_run(Character::Ironclad,vec![],vec![None;3],BTreeMap::new(),EncounterId::Cultist);
    let mut baseline = combat_state::without_item_context(combat(&mut run));
    let assert_same=|with_context:&mut crate::combat::Combat,without_context:&mut crate::combat::Combat|{
        assert_eq!(combat_state::debug_without_item_context(with_context),combat_state::debug_without_item_context(without_context));
        let mut with_items=with_context.public_full().unwrap();with_items.inventory=None;
        let mut without_items=without_context.public_full().unwrap();without_items.inventory=None;
        assert_eq!(with_items,without_items);
    };
    {
        let with_context=combat(&mut run);
        assert_same(with_context,&mut baseline);
        let target=with_context.public_full().unwrap().enemies[0].instance_id.clone();
        combat_state::set_enemy_block(with_context, 0, 5);
        combat_state::set_enemy_block(&mut baseline, 0, 5);
        let actions=[
            Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Enemy(target)),kind:EffectKind::Damage(DamagePayload::Fixed(5))},
            Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::ApplyFullPower{id:FullPowerId("strength"),amount:1}},
            Effect{source:Source::Actor(Actor::Enemy(combat_state::enemy_instance_id(with_context, 0))),target:Some(Actor::Player),kind:EffectKind::Damage(DamagePayload::Fixed(2))},
            Effect{source:Source::None,target:Some(Actor::Player),kind:EffectKind::Heal(1)},
        ];
        for effect in actions{
            with_context.enqueue_native_effects(vec![effect.clone()],None).unwrap();
            baseline.enqueue_native_effects(vec![effect],None).unwrap();
            assert_same(with_context,&mut baseline);
        }
        with_context.legal_actions_full().unwrap();baseline.legal_actions_full().unwrap();
        assert_same(with_context,&mut baseline);

        loop{
            let hand=with_context.public_full().unwrap().hand;
            let Some(card)=hand.first() else{break};
            let handle=card.instance;let definition=crate::content::Catalog::standard().card(card.id).unwrap();
            let target=if definition.card_type==CardType::Attack{Some(TargetId(1))}else{None};
            for combat in [&mut *with_context,&mut baseline]{
                combat_state::set_player_energy(combat, 99);
                assert!(combat.exhaust_card_for_test(handle));
                combat.step_full(&CombatAction::PlayCard{instance:handle.0,target}).unwrap();
            }
            assert_same(with_context,&mut baseline);
        }
        with_context.step_full(&CombatAction::EndTurn).unwrap();
        baseline.step_full(&CombatAction::EndTurn).unwrap();
        assert_same(with_context,&mut baseline);
        combat_state::set_player_hp(with_context, 1);
        combat_state::set_player_hp(&mut baseline, 1);
        let lethal=Effect{source:Source::None,target:Some(Actor::Player),kind:EffectKind::Damage(DamagePayload::HpLoss(1))};
        with_context.enqueue_native_effects(vec![lethal.clone()],None).unwrap();
        baseline.enqueue_native_effects(vec![lethal],None).unwrap();
        assert_same(with_context,&mut baseline);
    }
    let mut summon_run=complete_run(Character::Ironclad,vec![],vec![None;3],BTreeMap::new(),EncounterId::Collector);
    let mut summon_baseline = combat_state::without_item_context(combat(&mut summon_run));
    assert_same(combat(&mut summon_run),&mut summon_baseline);
    combat(&mut summon_run).step_full(&CombatAction::EndTurn).unwrap();
    summon_baseline.step_full(&CombatAction::EndTurn).unwrap();
    assert_same(combat(&mut summon_run),&mut summon_baseline);
}
