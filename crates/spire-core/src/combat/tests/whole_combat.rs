//! Authored whole-combat acceptance, not original-game evidence.
use super::super::*;
use super::super::{engine, rng};
use super::support::base_scenario;
use super::differential::encode_checkpoint;

pub(super) fn scenario(enemy: EnemyKind) -> Scenario {
    let mut s = base_scenario(); s.player.draw_per_turn = 3; s.encounter_id = enemy;
    let e = &mut s.enemies[0]; e.enemy_id = enemy;
    let (hp, first) = if enemy == EnemyKind::Cultist { (48, MoveId::Incantation) } else { (40, MoveId::Chomp) };
    e.hp = hp; e.max_hp = hp; e.intent_id = first; e.move_history = vec![first];
    s.piles.hand = [CardKind::Bash, CardKind::Strike, CardKind::Defend].into_iter().enumerate()
        .map(|(i, card_id)| CardInstance { instance_id: CardHandle::parse(&(i+1).to_string()).unwrap(), card_id, upgraded: true }).collect(); s
}
fn play(id: &str) -> Action { Action::PlayCard { card_instance_id: CardHandle::parse(id).unwrap(), target_id: Some(EnemyHandle::parse("enemy-1").unwrap()) } }
fn actions() -> Vec<Action> { vec![play("1"), play("2"), Action::EndTurn, play("1"), play("2")] }
fn ownership(c: &Combat, s: &Scenario) {
    let expected: std::collections::BTreeMap<_,_> = s.piles.hand.iter().chain(&s.piles.draw).map(|c| (c.instance_id.clone(), c.clone())).collect();
    let cards: Vec<_> = c.state.piles.hand.iter().chain(&c.state.piles.draw).chain(&c.state.piles.discard).chain(&c.state.piles.exhaust).chain(c.state.in_play.iter()).collect();
    assert_eq!(cards.len(), expected.len());
    assert_eq!(cards.into_iter().map(|c| (c.instance_id.clone(), c.clone())).collect::<std::collections::BTreeMap<_,_>>(), expected);
    assert_eq!(c.state.card_ids_issued, expected.keys().cloned().collect());
}
#[test]
fn two_turn_victories() {
    for (enemy, hp1, hp2, before, after, ai) in [(EnemyKind::Cultist,25,10,40,46,0),(EnemyKind::JawWorm,17,2,29,35,1)] {
        let s = scenario(enemy); let mut c = Combat::new(&s).unwrap();
        for a in &actions()[..2] { c.step(a).unwrap(); } assert_eq!(c.state.enemies[0].hp, hp1);
        c.step(&Action::EndTurn).unwrap();
        assert_eq!(c.state.piles.hand.iter().map(|c| c.instance_id.as_str()).collect::<Vec<_>>(), ["1","3","2"]);
        assert_eq!((c.state.rng.shuffle.word_index(),c.state.rng.enemy_ai.word_index()),(2,ai));
        c.step(&play("1")).unwrap(); assert_eq!(c.state.enemies[0].hp,hp2);
        assert_eq!(c.step(&play("2")).unwrap().status,TransitionStatus::Terminal);
        assert_eq!((c.state.hp_before_healing,c.state.hp_after_cleanup),(Some(before),Some(after)));
        assert_eq!(c.state.outcome,Some(Outcome::Victory)); assert!(c.legal_actions().unwrap().is_empty()); ownership(&c,&s);
    }
}
#[test]
fn cultist_death() {
    let mut s = scenario(EnemyKind::Cultist); s.player.hp=14; s.player.max_hp=20; s.player.draw_per_turn=1;
    s.piles.hand=vec![CardInstance{instance_id:CardHandle::parse("1").unwrap(),card_id:CardKind::Defend,upgraded:false}];
    let mut c=Combat::new(&s).unwrap();
    for hp in [14,8,0] { c.step(&Action::EndTurn).unwrap(); assert_eq!(c.state.player.hp,hp); assert_eq!(c.state.rng.enemy_ai.word_index(),0); ownership(&c,&s); }
    assert_eq!(c.state.outcome,Some(Outcome::Death)); assert_eq!((c.state.hp_before_healing,c.state.hp_after_cleanup),(Some(0),Some(0)));
}
#[test]
fn jaw_death_stops_rng() {
    let mut s=scenario(EnemyKind::JawWorm); s.player.hp=10;
    s.rng.enemy_ai=rng::StreamState::new(s.rng.enemy_ai.id().clone(),*s.rng.enemy_ai.key(),1u128<<64).unwrap();
    let mut c=Combat::new(&s).unwrap(); c.step(&Action::EndTurn).unwrap();
    assert_eq!(c.state.outcome,Some(Outcome::Death)); assert_eq!(c.state.rng,s.rng);
    assert_eq!((c.state.hp_before_healing,c.state.hp_after_cleanup),(Some(0),Some(0)));
    assert_eq!(c.state.enemies[0].move_history,[MoveId::Chomp]); ownership(&c,&s);
}
#[test]
fn whole_combat_safe_points_and_independent_snapshot_baselines() {
    for enemy in [EnemyKind::Cultist,EnemyKind::JawWorm] {
        let s=scenario(enemy); let mut c=Combat::new(&s).unwrap(); let mut saved=vec![];
        for action in actions() {
            let capture=c.snapshot(); let baseline=encode_checkpoint(&c);
            let mut direct=Combat::restore(&capture).unwrap(); direct.step(&action).unwrap();
            // A test-local scheduler uses the independently specified card payloads.
            c.state.phase=Phase::PlayerAction; c.state.resolved_effects=0;
            match &action {
                Action::EndTurn => c.state.pending.push_back(Effect{source:Source::None,target:None,kind:EffectKind::EndPlayerTurn}),
                Action::PlayCard{card_instance_id,..} => {
                    let i=c.state.piles.hand.iter().position(|c| &c.instance_id==card_instance_id).unwrap();
                    let card=c.state.piles.hand.remove(i); let bash=card.card_id==CardKind::Bash;
                    c.state.player.energy-=if bash {2} else {1}; c.state.in_play=Some(card);
                    let target=Some(Actor::Enemy(c.state.enemies[0].instance_id.clone()));
                    let vulnerable=c.state.enemies[0].powers.iter().any(|p|p.id==PowerId::Vulnerable);
                    let base=if bash {10} else {9}; let amount=if vulnerable {base*3/2} else {base};
                    let source=Source::Card(card_instance_id.clone());
                    c.state.pending.push_back(Effect{source:source.clone(),target:target.clone(),kind:EffectKind::Damage(DamagePayload::Fixed(amount))});
                    if bash {c.state.pending.push_back(Effect{source:source.clone(),target,kind:EffectKind::ApplyPower{id:PowerId::Vulnerable,amount:3}});}
                    c.state.pending.push_back(Effect{source,target:None,kind:EffectKind::FinishCard});
                    c.state.pending.push_back(Effect{source:Source::None,target:None,kind:EffectKind::Decision});
                }
            }
            saved.push((capture,baseline));
            while !c.state.pending.is_empty() {
                engine::dispatch_one(&mut c.state); assert!(c.state.fault.is_none()); ownership(&c,&s);
                let capture=c.snapshot(); let baseline=encode_checkpoint(&c);
                let mut replay=Combat::restore(&capture).unwrap(); engine::resolve(&mut replay.state);
                assert_eq!(encode_checkpoint(&replay),encode_checkpoint(&direct)); saved.push((capture,baseline));
            }
            assert_eq!(encode_checkpoint(&c),encode_checkpoint(&direct));
        }
        for (capture,baseline) in saved { assert_eq!(encode_checkpoint(&Combat::restore(&capture).unwrap()),baseline); }
    }
}
#[test]
fn forty_project_legal_walks() {
    for enemy in [EnemyKind::Cultist,EnemyKind::JawWorm] { for episode in 0..20 {
        let mut s=scenario(enemy); s.player.draw_per_turn=5;
        for stream in [&mut s.rng.shuffle,&mut s.rng.enemy_ai] { *stream=rng::make_stream(&[0;32],rng::Partition::Test,&format!("combat-vertical-{episode}"),stream.id()).unwrap(); }
        let mut cards:Vec<_>=(0..9).map(|i|CardInstance{instance_id:CardHandle::parse(&(i+1).to_string()).unwrap(),card_id:[CardKind::Bash,CardKind::Strike,CardKind::Defend][i%3],upgraded:(i+episode)%2!=0}).collect();
        s.piles.draw=cards.split_off(3); s.piles.hand=cards;
        let mut c=Combat::new(&s).unwrap(); let mut replay=Combat::restore(&c.snapshot()).unwrap();
        for step in 0..200 {
            let choices=c.legal_actions().unwrap(); let action=&choices[(episode+step)%choices.len()];
            assert_eq!(c.step(action),replay.step(action)); assert!(c.state.fault.is_none()); ownership(&c,&s); ownership(&replay,&s);
            assert_eq!(encode_checkpoint(&c),encode_checkpoint(&replay)); let capture=c.snapshot(); let baseline=encode_checkpoint(&c);
            replay=Combat::restore(&capture).unwrap(); assert_eq!(encode_checkpoint(&replay),baseline);
            if c.state.outcome.is_some() {break;} assert!(step<199,"walk budget exhausted");
        }
    }}
}
