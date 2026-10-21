use crate::*;
use crate::actions::CandidateMenu;
use crate::knowledge::PublicKnowledge;
use crate::observation::encode;
use crate::episode::EpisodeSequence;
use spire_core::combat::*;
use spire_core::combat::rng::Partition;

fn job(catalog: &ScenarioCatalog, id: Option<&str>) -> EpisodeJob {
    EpisodeSequence::default().prepare_reset(Some(&IntegerSeed::from_be_bytes(&[1]).unwrap()),
        id, catalog, Partition::Train).unwrap().job
}

// Validate space bounds from the independent fixture, plus cross-array invariants
// that a Box/MultiBinary space alone cannot express.
fn assert_row_invariants(row: &ObservationRow) {
    let fixture: serde_json::Value = serde_json::from_str(include_str!("fixtures/observation_spec_v1.json")).unwrap();
    let batch = ObservationBatch::from_rows(std::slice::from_ref(row)).unwrap();
    for (tensor,d) in batch.tensors.iter().zip(fixture["descriptor"]["tensors"].as_array().unwrap()) {
        let values: Vec<i64> = match &tensor.values {
            TensorValues::I32(v) => v.iter().map(|&x| i64::from(x)).collect(),
            TensorValues::I8(v) => v.iter().map(|&x| i64::from(x)).collect(),
        };
        for (i,&value) in values.iter().enumerate() {
            let high = if let Some(bounds) = d["high"].as_array() { bounds[i % bounds.len()].as_i64().unwrap() }
                else { d["high"].as_i64().unwrap() };
            assert!(value >= d["low"].as_i64().unwrap() && value <= high,"{}[{i}]={value}",tensor.key);
        }
    }
    let mut hand = [[0;2];3];
    for i in 0..10 {
        if row.hand_mask[i] == 1 { hand[row.hand_kind[i] as usize-1][row.hand_upgraded[i] as usize] += 1; }
        else { assert_eq!(row.hand_kind[i],0); assert_eq!(row.hand_features[i],[0;4]); assert_eq!(row.hand_playable[i],0); }
        let listed = (0..32).any(|a| row.action_mask[a] == 1 && row.action_source[a] == i as i32);
        assert_eq!(row.hand_playable[i], i8::from(listed));
    }
    for kind in 0..3 { for upgrade in 0..2 {
        assert_eq!(row.deck_counts[kind][upgrade], hand[kind][upgrade] + row.in_play_counts[kind][upgrade]
            + row.pile_counts.iter().map(|p| p[kind][upgrade]).sum::<i32>());
    } }
    for i in 0..2 { if row.enemy_history_mask[0][i] == 0 { assert_eq!(row.enemy_history[0][i],0); } }
    if row.enemy_damage_mask[0] == 0 { assert_eq!(row.enemy[0][3],0); }
    for i in 0..32 {
        if row.action_mask[i] == 0 {
            assert_eq!(row.action_kind[i],0); assert_eq!(row.action_source[i],-1);
            assert_eq!(row.action_target[i],-1); assert_eq!(row.action_features[i],[0;4]);
        }
    }
    if row.outcome[0] != 0 { assert_eq!(row.action_mask,[0;32]); assert_eq!(row.hand_playable,[0;10]); }
}

// Catches wrong pile accounting, hand/menu ordering, padding and published damage.
#[test]
fn starter_encoding_exact_rows() {
    for (id, kind, intent, damage, mask) in [
        ("starter-cultist-a0-v1", 1, 1, 0, 0),
        ("starter-jaw-worm-a0-v1", 2, 3, 11, 1),
    ] {
        let j = job(&ScenarioCatalog::starter(), Some(id));
        let c = j.instantiate().unwrap();
        let k = PublicKnowledge::from_template(j.template());
        let menu = CandidateMenu::from_combat(&c).unwrap();
        let row = encode(&c, &k, &menu, &j.template().context).unwrap();
        assert_row_invariants(&row);
        assert_eq!(row.player, [80,80,0,3,3,5,1]);
        assert_eq!(row.player_powers, [0;3]);
        assert_eq!(row.hand_kind, [1,2,1,2,3,0,0,0,0,0]);
        assert_eq!(row.hand_upgraded, [0;10]);
        assert_eq!(&row.hand_features[..5], &[[1,6,0,0],[1,0,5,0],[1,6,0,0],[1,0,5,0],[2,8,0,2]]);
        assert_eq!(&row.hand_features[5..], &[[0;4];5]);
        assert_eq!(row.hand_mask, [1,1,1,1,1,0,0,0,0,0]);
        assert_eq!(row.hand_playable, row.hand_mask);
        assert_eq!(row.deck_counts, [[5,0],[4,0],[1,0]]);
        assert_eq!(row.pile_counts, [[[3,0],[2,0],[0,0]], [[0;2];3], [[0;2];3]]);
        assert_eq!(row.in_play_counts, [[0;2];3]);
        assert_eq!(row.context, [1,0,0,0]);
        assert_eq!(row.context_mask, [1,1,0,0]);
        assert_eq!(row.enemy_kind, [kind]);
        assert_eq!(row.enemy_intent, [intent]);
        assert_eq!(row.enemy[0][3], damage);
        assert_eq!(row.enemy_damage_mask, [mask]);
        assert_eq!(row.enemy_history, [[0;2]]);
        assert_eq!(row.enemy_history_mask, [[0;2]]);
        assert_eq!(row.relic_presence, [1]);
        assert_eq!(row.outcome, [0]);
        assert_eq!(&row.action_kind[..6], &[1,2,2,2,2,2]);
        assert_eq!(&row.action_source[..6], &[-1,0,1,2,3,4]);
        assert_eq!(&row.action_target[..6], &[-1,0,-1,0,-1,0]);
        assert_eq!(&row.action_mask[..6], &[1;6]);
        assert_eq!(&row.action_mask[6..], &[0;26]);
        assert_eq!(&row.action_source[6..], &[-1;26]);
        assert_eq!(&row.action_target[6..], &[-1;26]);
        assert_eq!(&row.action_kind[6..], &[0;26]);
        assert_eq!(row.action_features[0], [0;4]);
        assert_eq!(&row.action_features[1..6], &row.hand_features[..5]);
        assert_eq!(&row.action_features[6..], &[[0;4];26]);
        assert_eq!(menu.resolve(0).unwrap().core, Action::EndTurn);
        for bad in [-1,6,31,32,i64::MAX] { assert!(menu.resolve(bad).is_err()); }
        let mut actions: Vec<_> = (0..6).map(|i| menu.resolve(i).unwrap().core.clone()).collect();
        actions.sort_by_key(|a| format!("{a:?}"));
        let mut legal = c.legal_actions().unwrap();
        legal.sort_by_key(|a| format!("{a:?}"));
        assert_eq!(actions, legal);
    }
}

// Catches descriptor/table drift, lossy float narrowing, shape growth and buffer aliasing.
#[test]
fn schema_descriptor_and_spaces_data_are_exact() {
    use sha2::{Digest, Sha256};
    let bytes = include_bytes!("fixtures/observation_spec_v1.json");
    assert_eq!(super::support::hex(&Sha256::digest(bytes)),
        "f1512be59209e5e937cb8729df96727c0002bc9daae34121178748e299be43d5");
    let fixture: serde_json::Value = serde_json::from_slice(bytes).unwrap();
    let spec = observation_spec();
    assert_eq!(spec.descriptor(), fixture["descriptor"]);
    let canonical = canonical_json(&fixture["descriptor"]).unwrap();
    assert_eq!(canonical, fixture["canonical_json"].as_str().unwrap().as_bytes());
    let mut hash = Sha256::new(); hash.update(b"spire-lab/env-schema/v1\0"); hash.update(&canonical);
    assert_eq!(super::support::hex(&hash.finalize()), fixture["fingerprint"].as_str().unwrap());
    assert_eq!(super::support::hex(&spec.fingerprint()), "8fd66d212259d2bdd27139988a772e1a6c223f8d65524f38c74174a0d0843357");
    let mut detached = spec.descriptor(); detached["schema_id"] = serde_json::json!("tampered");
    assert_eq!(spec.descriptor(), fixture["descriptor"]);

    let mut t = super::support::template();
    t.hand = (0..10).map(|i| CardSpec { kind: [CardKind::Strike,CardKind::Defend,CardKind::Bash][i%3], upgraded:true }).collect();
    t.draw = vec![CardSpec { kind:CardKind::Defend, upgraded:false };989];
    t.energy_per_turn = 10;
    let j = job(&ScenarioCatalog::new(vec![t]).unwrap(), None);
    let c = j.instantiate().unwrap();
    let k = PublicKnowledge::from_template(j.template());
    let menu = CandidateMenu::from_combat(&c).unwrap();
    let row = encode(&c,&k,&menu,&j.template().context).unwrap();
    assert_eq!(row.hand_mask, [1;10]);
    assert_eq!(row.hand_upgraded, [1;10]);
    assert_eq!(&row.hand_features[..3], &[[1,9,0,0],[1,0,8,0],[2,10,0,3]]);
    assert_eq!(&row.action_features[1..4], &[[1,9,0,0],[1,0,8,0],[2,10,0,3]]);
    assert_row_invariants(&row);
    assert_eq!(row.deck_counts, [[0,4],[989,3],[0,3]]);
    assert_eq!(row.pile_counts[0], [[0,0],[989,0],[0,0]]);
    assert_eq!(&row.action_mask[..11], &[1;11]);
    assert_eq!(&row.action_mask[11..], &[0;21]);
    let mut p = c.public_projection().unwrap();
    p.turn = i32::MAX as u32;
    p.player.hp = i32::MAX; p.player.max_hp = i32::MAX; p.player.block = i32::MAX;
    let upper = crate::observation::encode_projected(&p,&k,&menu,&j.template().context).unwrap();
    assert_eq!(upper.player[6], i32::MAX);
    assert_row_invariants(&upper);
    let mut batch = ObservationBatch::from_rows(&[row.clone(),upper.clone()]).unwrap();
    let retained = batch.clone();
    let table = fixture["descriptor"]["tensors"].as_array().unwrap();
    assert_eq!(batch.rows, 2);
    assert_eq!(batch.tensors.len(), 26);
    for (tensor, d) in batch.tensors.iter().zip(table) {
        assert_eq!(tensor.key, d["key"].as_str().unwrap());
        let expected: Vec<_> = std::iter::once(2).chain(d["shape"].as_array().unwrap().iter().map(|n| n.as_u64().unwrap() as usize)).collect();
        assert_eq!(tensor.shape, expected);
        let count: usize = tensor.shape.iter().product();
        match &tensor.values {
            TensorValues::I32(v) => { assert_eq!(d["dtype"], "int32"); assert_eq!(v.len(),count); },
            TensorValues::I8(v) => { assert_eq!(d["dtype"], "int8"); assert_eq!(v.len(),count); assert!(v.iter().all(|&x| x==0 || x==1)); },
        }
    }
    match &batch.tensors[0].values { TensorValues::I32(v) => assert_eq!(v, &[row.player,upper.player].concat()), _ => panic!("dtype") }
    if let TensorValues::I32(v) = &mut batch.tensors[0].values { v[0] = 123; }
    assert_ne!(batch, retained);
    assert_eq!(ObservationBatch::from_rows(&[row.clone(),upper]).unwrap(), retained);
    let empty = ObservationBatch::from_rows(&[]).unwrap();
    assert_eq!(empty.rows,0);
    for t in empty.tensors { assert_eq!(t.shape[0],0); match t.values { TensorValues::I32(v)=>assert!(v.is_empty()),TensorValues::I8(v)=>assert!(v.is_empty()) } }
    p.turn += 1;
    assert!(crate::observation::encode_projected(&p,&k,&menu,&j.template().context).is_err());
    p.turn = 1;
    let overhand = vec![p.hand[0].clone();11]; p.hand = &overhand;
    assert!(matches!(crate::observation::encode_projected(&p,&k,&menu,&j.template().context), Err(EnvError::Input {reason:InputReason::Capacity,..})));
    assert!(crate::observation::capacity(10,2,0).is_err());
    assert!(crate::observation::capacity(10,1,33).is_err());
    let mut bad_k = k.clone(); bad_k.deck[0][1] = 0;
    assert!(encode(&c,&bad_k,&menu,&j.template().context).is_err());
    let mut bad_k = k.clone(); bad_k.deck[0][1] += 1;
    assert!(encode(&c,&bad_k,&menu,&j.template().context).is_err());
}

fn step(c: &mut Combat, k: &mut PublicKnowledge, context: &PublicContext, index: i64) -> ObservationRow {
    let menu = CandidateMenu::from_combat(c).unwrap();
    let previous = encode(c,k,&menu,context).unwrap();
    let actions: Vec<_> = menu.rows().map(|(a,_,_)| a.core.clone()).collect();
    let legal = c.legal_actions().unwrap();
    assert_eq!(actions.len(),legal.len());
    assert!(actions.iter().all(|a| legal.contains(a)));
    let accepted = menu.resolve(index).unwrap();
    assert_ne!(c.step(&accepted.core).unwrap().status, TransitionStatus::Fault);
    k.observe_completed(&previous,accepted,&c.public_projection().unwrap()).unwrap();
    let row = encode(c,k,&CandidateMenu::from_combat(c).unwrap(),context).unwrap();
    assert_row_invariants(&row);
    row
}

// Catches card-shift/repeat suppression, killing-card invention and hidden-handle leakage.
#[test]
fn history_accounting_and_noninterference() {
    let j = job(&ScenarioCatalog::starter(), None);
    let mut c = j.instantiate().unwrap();
    let mut k = PublicKnowledge::from_template(j.template());
    let context = &j.template().context;
    let card = step(&mut c,&mut k,context,1);
    assert_eq!(card.enemy_history, [[0,0]]);
    assert_eq!(card.enemy_history_mask, [[0,0]]);
    let low_energy = step(&mut c,&mut k,context,2);
    assert_eq!(low_energy.hand_kind,[2,2,3,0,0,0,0,0,0,0]);
    assert_eq!(low_energy.hand_playable,[1,1,0,0,0,0,0,0,0,0]);
    let spent = step(&mut c,&mut k,context,1);
    assert_eq!(spent.hand_playable,[0;10]);
    assert_eq!(spent.action_mask[0],1);
    assert_eq!(&spent.action_mask[1..],&[0;31]);
    let first = step(&mut c,&mut k,context,0);
    assert_eq!(first.enemy_intent,[2]);
    assert_eq!(first.enemy_history, [[1,0]]);
    assert_eq!(first.enemy_history_mask, [[1,0]]);
    let second = step(&mut c,&mut k,context,0);
    assert_eq!(second.enemy_history, [[2,1]]);
    let third = step(&mut c,&mut k,context,0);
    assert_eq!(third.enemy_history, [[2,2]]);
    assert_eq!(third.enemy_history_mask, [[1,1]]);
    let fresh = j.instantiate().unwrap();
    let reset = encode(&fresh,&PublicKnowledge::from_template(j.template()),&CandidateMenu::from_combat(&fresh).unwrap(),context).unwrap();
    assert_eq!(reset.enemy_history, [[0,0]]);
    assert_eq!(reset.enemy_history_mask, [[0,0]]);

    let mut t = super::support::template();
    t.hand = vec![CardSpec {kind:CardKind::Strike,upgraded:true};10]; t.energy_per_turn=10;
    let victory = job(&ScenarioCatalog::new(vec![t]).unwrap(),None);
    let mut c = victory.instantiate().unwrap();
    let mut k = PublicKnowledge::from_template(victory.template());
    let mut terminal = reset.clone();
    for _ in 0..6 { terminal = step(&mut c,&mut k,context,1); }
    assert_eq!(terminal.outcome,[1]);
    assert_eq!(terminal.in_play_counts,[[0,1],[0,0],[0,0]]);
    assert_eq!(terminal.pile_counts,[[[0;2];3],[[0,5],[0,0],[0,0]],[[0;2];3]]);
    assert_eq!(terminal.hand_mask,[1,1,1,1,0,0,0,0,0,0]);
    assert_eq!(terminal.hand_playable,[0;10]);
    assert_eq!(terminal.action_mask,[0;32]);
    assert_eq!(terminal.action_source,[-1;32]);
    assert!(CandidateMenu::from_combat(&c).unwrap().resolve(0).is_err());
    assert_eq!(terminal.enemy_history,[[0,0]]);

    let mut death_t = j.template().clone(); death_t.player_hp=1; death_t.encounter=EnemyKind::JawWorm; death_t.enemy_hp=42;
    let death = job(&ScenarioCatalog::new(vec![death_t]).unwrap(),None);
    let mut c = death.instantiate().unwrap(); let mut k = PublicKnowledge::from_template(death.template());
    let terminal = step(&mut c,&mut k,context,0);
    assert_eq!(terminal.outcome,[2]); assert_eq!(terminal.in_play_counts,[[0;2];3]);
    assert_eq!(terminal.enemy_intent,[3]); assert_eq!(terminal.enemy_history,[[0,0]]);
    assert_eq!(terminal.enemy_history_mask,[[0,0]]); assert_eq!(terminal.action_mask,[0;32]);

    let mut death_t = j.template().clone(); death_t.player_hp=1;
    let death = job(&ScenarioCatalog::new(vec![death_t]).unwrap(),None);
    let mut c = death.instantiate().unwrap(); let mut k = PublicKnowledge::from_template(death.template());
    let first = step(&mut c,&mut k,context,0);
    assert_eq!(first.enemy_history,[[1,0]]);
    let terminal = step(&mut c,&mut k,context,0);
    assert_eq!(terminal.outcome,[2]); assert_eq!(terminal.enemy_intent,[2]);
    assert_eq!(terminal.enemy_history,[[1,0]]); assert_eq!(terminal.enemy_history_mask,[[1,0]]);
    assert_eq!(terminal.in_play_counts,[[0;2];3]);

    let mut scenario = j.scenario().unwrap();
    scenario.piles.draw.reverse();
    use spire_core::combat::rng::{StreamId,StreamState,Domain};
    scenario.rng.shuffle = StreamState::new(scenario.rng.shuffle.id().clone(),[123;32],17).unwrap();
    let renamed = EnemyHandle::parse("arbitrary-enemy").unwrap();
    scenario.enemies[0].instance_id = renamed.clone(); scenario.rng.enemy_instance_id = renamed.clone();
    scenario.rng.enemy_ai = StreamState::new(StreamId::new(Domain::CombatEnemyAi,vec![j.template().id.clone(),renamed.as_str().into()]).unwrap(),[19;32],31).unwrap();
    for (i, card) in scenario.piles.hand.iter_mut().chain(&mut scenario.piles.draw).enumerate() {
        card.instance_id = CardHandle::parse(&format!("-{}",100-i*7)).unwrap();
    }
    let altered = Combat::new(&scenario).unwrap();
    let knowledge = PublicKnowledge::from_template(j.template());
    assert_eq!(encode(&altered,&knowledge,&CandidateMenu::from_combat(&altered).unwrap(),context).unwrap(),reset);
    // Same scenario ID/root/ordinal: only asserted public context changes.
    let mut t = j.template().clone(); t.context = PublicContext {act:Some(1),next_encounter:Some(EnemyKind::JawWorm)};
    let contextual = job(&ScenarioCatalog::new(vec![t]).unwrap(),None);
    assert_eq!(j.root(),contextual.root()); assert_eq!(j.ordinal(),contextual.ordinal());
    assert_eq!(j.template().id,contextual.template().id);
    let c = contextual.instantiate().unwrap();
    let mut row = encode(&c,&knowledge,&CandidateMenu::from_combat(&c).unwrap(),&contextual.template().context).unwrap();
    assert_eq!(row.context,[1,0,1,2]); assert_eq!(row.context_mask,[1;4]);
    row.context=reset.context; row.context_mask=reset.context_mask; assert_eq!(row,reset);
}
