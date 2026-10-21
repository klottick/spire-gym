use std::sync::Arc;
use crate::*;
use spire_core::combat::{CardKind, EnemyKind};
use spire_core::combat::rng::Partition;

fn seed() -> IntegerSeed { IntegerSeed::from_be_bytes(&[7]).unwrap() }
fn request() -> ResetRequest { ResetRequest { seed:Some(seed()), scenario_id:None } }
fn config(batch_size: usize, autoreset: AutoReset) -> PoolConfig {
    PoolConfig { batch_size, workers:1, partition:Partition::Train, autoreset, record_replay:false }
}
fn win_template() -> ScenarioTemplate {
    let mut t = super::support::template();
    t.player_hp = 40;
    t.energy_per_turn = 10;
    t.hand = vec![CardSpec { kind:CardKind::Strike, upgraded:true };10];
    t
}

// Catches stale menu mapping, premature cleanup, synthetic reset and reward mistakes.
#[test]
fn manual_pool_matches_core_and_requires_reset() {
    let catalog = Arc::new(ScenarioCatalog::new(vec![win_template()]).unwrap());
    let pool = CombatPool::new(catalog.clone(), config(1,AutoReset::Manual)).unwrap();
    assert!(matches!(pool.step_batch(&[0], &[[0,0]]), Err(EnvError::State { reason:StateReason::Uninitialized, .. })));
    let reset = pool.reset_batch(&[0], &[request()]).unwrap();
    assert_eq!(reset.decisions.infos[0].episode_id,"episode-0");
    assert_eq!(reset.decisions.infos[0].episode_steps,0);
    assert_ne!(reset.decisions.tokens[0],[0,0]);
    let prepared = crate::episode::EpisodeSequence::default().prepare_reset(Some(&seed()),None,&catalog,Partition::Train).unwrap();
    let mut core = prepared.job.instantiate().unwrap();
    let mut knowledge = crate::knowledge::PublicKnowledge::from_template(prepared.job.template());
    let mut menu = crate::actions::CandidateMenu::from_combat(&core).unwrap();
    let mut row = crate::observation::encode(&core,&knowledge,&menu,&prepared.job.template().context).unwrap();
    assert_eq!(reset.decisions.observations,ObservationBatch::from_rows(&[row.clone()]).unwrap());
    let mut tokens = reset.decisions.tokens;
    for steps in 1..=6 {
        let action = menu.resolve(1).unwrap().clone();
        core.step(&action.core).unwrap();
        knowledge.observe_completed(&row,&action,&core.public_projection().unwrap()).unwrap();
        menu = crate::actions::CandidateMenu::from_combat(&core).unwrap();
        row = crate::observation::encode(&core,&knowledge,&menu,&prepared.job.template().context).unwrap();
        let result = pool.step_batch(&[1],&tokens).unwrap();
        assert_eq!(result.current.observations,ObservationBatch::from_rows(&[row.clone()]).unwrap());
        assert_eq!(pool.state.lock().unwrap().slots[0].live.as_ref().unwrap().combat.snapshot(), core.snapshot());
        assert_eq!(result.current.infos[0].episode_steps,steps);
        assert_eq!(result.truncated,[false]);
        if steps < 6 { assert_eq!(result.rewards,[0.0]); assert_eq!(result.terminated,[false]); }
        else {
            assert_eq!(result.rewards,[1.0]); assert_eq!(result.terminated,[true]);
            assert_eq!(result.current.tokens,[[0,0]]);
            assert_eq!(result.finals,result.current);
            assert_eq!(result.current.infos[0].hp_before_healing,Some(40));
            assert_eq!(result.current.infos[0].hp_after_cleanup,Some(46));
            assert_eq!(row.action_mask,[0;32]);
        }
        tokens = result.current.tokens;
    }
    assert!(matches!(pool.step_batch(&[0],&tokens),Err(EnvError::State { reason:StateReason::Terminal,.. })));
    let next = pool.reset_batch(&[0],&[ResetRequest::default()]).unwrap();
    assert_eq!(next.decisions.infos[0].episode_id,"episode-1");
    let abandoned = pool.reset_batch(&[0],&[ResetRequest::default()]).unwrap();
    assert_eq!(abandoned.decisions.infos[0].episode_id,"episode-2");

    let mut death = win_template(); death.id = "death".into(); death.encounter = EnemyKind::JawWorm;
    death.enemy_hp = 42; death.player_hp = 1;
    let p = CombatPool::new(Arc::new(ScenarioCatalog::new(vec![death]).unwrap()),config(1,AutoReset::Manual)).unwrap();
    let r = p.reset_batch(&[0],&[request()]).unwrap();
    let terminal = p.step_batch(&[0],&r.decisions.tokens).unwrap();
    assert_eq!(terminal.rewards,[-1.0]);
    assert_eq!(terminal.current.infos[0].hp_before_healing,Some(0));
    assert_eq!(terminal.current.infos[0].hp_after_cleanup,Some(0));
    assert_eq!(terminal.current.tokens[0],[0,0]);
}

type SavedSlot = (crate::episode::EpisodeSequence, Option<(spire_core::combat::Snapshot,
    ObservationRow, crate::actions::CandidateMenu, crate::knowledge::PublicKnowledge, [u64;2], u64)>);
fn saved(pool: &CombatPool) -> (Vec<SavedSlot>, Option<u64>) {
    let state = pool.state.lock().unwrap();
    (state.slots.iter().map(|s| (s.sequence.clone(),s.live.as_ref().map(|l|
        (l.combat.snapshot(),l.row.clone(),l.menu.clone(),l.knowledge.clone(),l.token.into(),l.steps)))).collect(),state.serials.next())
}

// Catches dispatch before late-row validation and partial reset/counter installation.
#[test]
fn invalid_batch_and_reset_are_atomic() {
    let catalog = Arc::new(ScenarioCatalog::starter());
    let pool = CombatPool::new(catalog.clone(),config(3,AutoReset::Manual)).unwrap();
    let requests = vec![request();3];
    let original = requests.clone();
    let reset = pool.reset_batch(&[0,1,2],&requests).unwrap();
    assert_eq!(requests,original);
    let before = saved(&pool);
    let repeated=pool.read_rows(&[2,0,2]).unwrap();
    assert_eq!(repeated.slot_indices,[2,0,2]);
    assert_eq!(repeated.tokens,[reset.decisions.tokens[2],reset.decisions.tokens[0],reset.decisions.tokens[2]]);
    for actions in [vec![0,0,-1],vec![0,0,32],vec![0,0,31],vec![0,0],vec![0,0,0,0]] {
        assert!(pool.step_batch(&actions,&reset.decisions.tokens).is_err());
        assert_eq!(saved(&pool),before);
    }
    for tokens in [vec![[0,0];3],{let mut t=reset.decisions.tokens.clone();t[2]=[0,0];t},reset.decisions.tokens[..2].to_vec(),{
        let foreign = CombatPool::new(catalog,config(3,AutoReset::Manual)).unwrap();
        foreign.reset_batch(&[0,1,2],&requests).unwrap().decisions.tokens
    }] {
        assert!(pool.step_batch(&[0;3],&tokens).is_err()); assert_eq!(saved(&pool),before);
    }
    for indices in [vec![0,0,2],vec![0,1,3]] {
        assert!(pool.reset_batch(&indices,&requests).is_err()); assert_eq!(saved(&pool),before);
    }
    assert!(pool.reset_batch(&[0,1,2],&requests[..2]).is_err());
    assert!(expand_seeds(SeedBatch::PerSlot(vec![None;2]),3).is_err());
    let mut malformed = requests.clone(); malformed[2].scenario_id = Some("missing".into());
    assert!(pool.reset_batch(&[0,1,2],&malformed).is_err()); assert_eq!(saved(&pool),before);
    assert_eq!(malformed[2].scenario_id.as_deref(),Some("missing"));
    let replay = pool.reset_batch(&[0,1,2],&requests).unwrap();
    assert_eq!(replay.decisions.observations,reset.decisions.observations);
    let replay_state = saved(&pool);
    assert!(pool.step_batch(&[0;3],&reset.decisions.tokens).is_err()); assert_eq!(saved(&pool),replay_state);
    let stepped = pool.step_batch(&[0;3],&replay.decisions.tokens).unwrap();
    let after = saved(&pool);
    assert!(pool.step_batch(&[0;3],&replay.decisions.tokens).is_err()); assert_eq!(saved(&pool),after);
    assert_ne!(stepped.current.tokens,replay.decisions.tokens);
    pool.state.lock().unwrap().serials=crate::identity::CheckedCounter::new(u64::MAX-1);
    let before=saved(&pool);
    assert!(pool.step_batch(&[0;3],&stepped.current.tokens).is_err());assert_eq!(saved(&pool),before);
    assert!(pool.reset_batch(&[0,1,2],&requests).is_err());assert_eq!(saved(&pool),before);
}

// Catches final/replacement identity mixups, action reuse, and nonselected reset mutation.
#[test]
fn same_step_final_rows_and_partial_reset() {
    let mut death = win_template(); death.id = "death".into(); death.encounter = EnemyKind::JawWorm;
    death.player_hp = 1; death.enemy_hp = 42;
    let mut survivor = win_template(); survivor.id="survivor".into(); survivor.player_hp=999; survivor.player_max_hp=999;
    let pool = CombatPool::new(Arc::new(ScenarioCatalog::new(vec![win_template(),death,survivor]).unwrap()),config(3,AutoReset::SameStep)).unwrap();
    let r = pool.reset_batch(&[0,1,2],&[request(),request(),ResetRequest { seed:Some(seed()),scenario_id:Some("survivor".into()) }]).unwrap();
    let mut tokens = r.decisions.tokens;
    for step in 1..=6 {
        let r = pool.step_batch(&[1,1,0],&tokens).unwrap();
        assert_eq!(r.truncated,[false;3]);
        assert_eq!(r.terminated,if step == 6 { vec![true,true,false] } else { vec![false;3] });
        assert_eq!(r.finals.slot_indices,if step == 6 { vec![0,1] } else { vec![] });
        if step == 6 {
            assert_eq!(r.rewards,[1.0,1.0,0.0]);
            assert_eq!(r.finals.infos.iter().map(|i|i.episode_steps).collect::<Vec<_>>(),[6,6]);
            assert_eq!(r.finals.tokens,[[0,0];2]);
            assert!(r.finals.infos.iter().all(|i|i.episode_id=="episode-0" && i.scenario_id=="authored"));
            let values=|key| &r.finals.observations.tensors.iter().find(|t|t.key==key).unwrap().values;
            assert_eq!(values("outcome"),&TensorValues::I32(vec![1,1]));
            assert_eq!(values("in_play_counts"),&TensorValues::I32(vec![0,1,0,0,0,0,0,1,0,0,0,0]));
            assert_eq!(values("player"),&TensorValues::I32(vec![46,80,0,4,10,5,1,46,80,0,4,10,5,1]));
            for i in 0..2 {
                assert_eq!(r.current.infos[i].episode_id,"episode-1");
                assert_eq!(r.current.infos[i].episode_steps,0);
                let state = pool.state.lock().unwrap(); let l = state.slots[i].live.as_ref().unwrap();
                assert_eq!(l.row.in_play_counts,[[0;2];3]); assert_eq!(l.row.enemy_history_mask,[[0;2]]);
            }
            assert_eq!(r.current.infos[2].episode_steps,6);
        } else {
            assert_eq!(r.finals.observations.rows,0);
            assert!(r.finals.observations.tensors.iter().all(|t| match &t.values {
                TensorValues::I32(v)=>v.is_empty(),TensorValues::I8(v)=>v.is_empty() }));
        }
        tokens = r.current.tokens;
    }
    // Replacement is the low-HP Jaw Worm; both die without changing the third episode.
    let dead = pool.step_batch(&[0;3],&tokens).unwrap();
    assert_eq!(dead.rewards,[-1.0,-1.0,0.0]);
    assert_eq!(dead.finals.infos[0].outcome,2);
    assert_eq!(dead.finals.infos[0].hp_after_cleanup,Some(0));
    let before = saved(&pool);
    let selected = pool.reset_batch(&[2,0],&[ResetRequest { seed:None,scenario_id:Some("death".into()) },request()]).unwrap();
    assert_eq!(selected.decisions.slot_indices,[2,0]);
    assert_eq!(selected.decisions.infos[0].episode_id,"episode-1");
    assert_eq!(selected.decisions.infos[0].scenario_id,"death");
    assert_eq!(selected.decisions.infos[1].episode_id,"episode-0");
    assert_eq!(selected.effective_seeds,[None,Some(seed())]);
    assert!(selected.decisions.tokens[1][1] < selected.decisions.tokens[0][1]);
    assert_eq!(saved(&pool).0[1],before.0[1]);
    let all = CombatPool::new(Arc::new(ScenarioCatalog::new(vec![{
        let mut t = win_template(); t.encounter=EnemyKind::JawWorm; t.enemy_hp=42;t.player_hp=1;t
    }]).unwrap()),config(3,AutoReset::SameStep)).unwrap();
    let r = all.reset_batch(&[0,1,2],&vec![request();3]).unwrap();
    let r = all.step_batch(&[0;3],&r.decisions.tokens).unwrap();
    assert_eq!(r.terminated,[true;3]);assert_eq!(r.rewards,[-1.0;3]);assert_eq!(r.finals.slot_indices,[0,1,2]);
    assert!(r.current.infos.iter().all(|i|i.episode_steps==0 && i.episode_id=="episode-1"));
}

// Catches wrapping metadata identities, unchecked accepted counts and mutable output aliases.
#[test]
fn owned_rows_and_checked_identity_boundaries() {
    use crate::identity::{CheckedCounter, PoolIdAllocator};
    let ids = PoolIdAllocator::new(u64::MAX);
    assert_eq!(ids.allocate().unwrap(),u64::MAX); assert!(ids.allocate().is_err());
    let mut counter = CheckedCounter::new(u64::MAX);
    assert!(counter.check(2).is_err());assert_eq!(counter.next(),Some(u64::MAX));
    assert_eq!(counter.take().unwrap(),u64::MAX); assert!(counter.take().is_err());
    let mut template = win_template();
    let catalog = Arc::new(ScenarioCatalog::new(vec![template.clone()]).unwrap());
    template.hand.clear();
    let pool = CombatPool::new(catalog,config(1,AutoReset::SameStep)).unwrap();
    let r = pool.reset_batch(&[0],&[request()]).unwrap();let retained = r.clone();
    let mut changed = r.clone(); changed.decisions.infos[0].episode_id.clear();changed.decisions.tokens[0]=[0,0];
    if let TensorValues::I32(v)=&mut changed.decisions.observations.tensors[0].values {v[0]=-9;}
    assert_eq!(pool.read_rows(&[0]).unwrap(),retained.decisions);
    {
        let mut state = pool.state.lock().unwrap();state.serials=CheckedCounter::new(u64::MAX);
        state.slots[0].live.as_mut().unwrap().steps=u64::MAX;
    }
    let before=saved(&pool);
    assert!(pool.step_batch(&[1],&r.decisions.tokens).is_err());assert_eq!(saved(&pool),before);
    {
        let mut s=pool.state.lock().unwrap();s.slots[0].live.as_mut().unwrap().steps=0;
        s.serials=CheckedCounter::exhausted();
    }
    let before=saved(&pool);
    assert!(pool.step_batch(&[1],&r.decisions.tokens).is_err());assert_eq!(saved(&pool),before);
    assert!(pool.reset_batch(&[0],&[request()]).is_err());assert_eq!(saved(&pool),before);
    {
        let mut s=pool.state.lock().unwrap();s.serials=CheckedCounter::new(100);
        let staged=crate::episode::exhaustion_test_sequence(&seed()).prepare_reset(None,None,&ScenarioCatalog::starter(),Partition::Train).unwrap();
        s.slots[0].sequence=staged.sequence;
    }
    let before=saved(&pool);
    assert!(pool.step_batch(&[1],&r.decisions.tokens).is_err());assert_eq!(saved(&pool),before);
    assert!(pool.reset_batch(&[0],&[ResetRequest::default()]).is_err());assert_eq!(saved(&pool),before);
    pool.reset_batch(&[0],&[request()]).unwrap();
    pool.close().unwrap();pool.close().unwrap();
    assert_eq!(r,retained);
    assert!(matches!(pool.read_rows(&[0]),Err(EnvError::State { reason:StateReason::Closed,.. })));
    assert!(pool.reset_batch(&[0],&[request()]).is_err());assert!(pool.step_batch(&[1],&r.decisions.tokens).is_err());
    for (size,workers,replay) in [(0,1,false),(1,0,false)] {
        let mut c=config(size,AutoReset::Manual);c.workers=workers;c.record_replay=replay;
        assert!(CombatPool::new(Arc::new(ScenarioCatalog::starter()),c).is_err());
    }
}

// Catches installing early staged rows when a later first initialization lacks entropy.
#[test]
fn entropy_failure_preserves_mixed_slots() {
    let pool=CombatPool::new(Arc::new(ScenarioCatalog::starter()),config(3,AutoReset::Manual)).unwrap();
    let first=pool.reset_batch(&[0],&[request()]).unwrap();
    assert_eq!(first.decisions.slot_indices,[0]);
    assert!(matches!(pool.step_batch(&[0;3],&[[0,0];3]),Err(EnvError::Input { reason:InputReason::Token,.. })));
    let mut tokens=vec![[0,0];3]; tokens[0]=first.decisions.tokens[0];
    assert!(matches!(pool.step_batch(&[0;3],&tokens),Err(EnvError::State { reason:StateReason::Uninitialized,slot:Some(1) })));
    let before=saved(&pool);
    let mut calls=0;
    let result=pool.reset_with(&[0,1,2],&vec![ResetRequest::default();3],|sequence,request| {
        sequence.prepare_reset_with_entropy(request.seed.as_ref(),request.scenario_id.as_deref(),&pool.catalog,Partition::Train,&mut |bytes| {
            calls+=1;
            if calls==2 { return Err(crate::error::setup("scripted entropy failure")); }
            *bytes=[9;32];Ok(())
        })
    });
    assert!(matches!(result,Err(EnvError::Setup { slot:Some(2),.. })));
    assert_eq!(calls,2);assert_eq!(saved(&pool),before);
    assert_eq!(pool.state.lock().unwrap().status,PoolStatus::Active);
}

// Catches early-stop fault handling, provenance loss, resumed execution and hidden lock blocking.
#[test]
fn runtime_failures_fault_pool_and_preserve_first_core_fault() {
    use spire_core::combat::{Combat,FaultKind};
    use spire_core::combat::rng::StreamState;
    let pool=CombatPool::new(Arc::new(ScenarioCatalog::starter()),config(3,AutoReset::Manual)).unwrap();
    let reset=pool.reset_batch(&[0,1,2],&vec![ResetRequest { seed:Some(seed()),scenario_id:Some("starter-jaw-worm-a0-v1".into()) };3]).unwrap();
    {
        let mut s=pool.state.lock().unwrap();
        assert!(matches!(pool.read_rows(&[0]),Err(EnvError::State { reason:StateReason::Busy,.. })));
        for i in [0,2] {
            let l=s.slots[i].live.as_mut().unwrap();
            let mut scenario=l.job.scenario().unwrap();
            scenario.rng.enemy_ai=StreamState::new(scenario.rng.enemy_ai.id().clone(),[3;32],(1u128<<64)-1).unwrap();
            l.combat=Combat::new(&scenario).unwrap();
        }
    }
    let faults=match pool.step_batch(&[0;3],&reset.decisions.tokens) { Err(EnvError::Fault { records })=>records,other=>panic!("{other:?}") };
    assert_eq!(faults.iter().map(|f|f.slot).collect::<Vec<_>>(),[Some(0),Some(2)]);
    assert!(faults.iter().all(|f|f.stage==FaultStage::Resolution && f.core_fault.as_ref().unwrap().kind==FaultKind::RngExhausted));
    {
        let s=pool.state.lock().unwrap();assert_eq!(s.status,PoolStatus::Faulted);
        assert_eq!(s.slots[1].live.as_ref().unwrap().steps,1);
        for (f,i) in faults.iter().zip([0,2]) { assert_eq!(f.core_fault,s.slots[i].live.as_ref().unwrap().combat.fault_record()); }
    }
    let before=saved(&pool);
    for result in [pool.step_batch(&[0;3],&reset.decisions.tokens).map(|_|()),pool.reset_batch(&[0],&[request()]).map(|_|()),pool.read_rows(&[0]).map(|_|())] {
        assert!(matches!(result,Err(EnvError::State { reason:StateReason::Faulted,.. })));
    }
    assert_eq!(saved(&pool),before);assert_eq!(pool.state.lock().unwrap().faults,faults);
    pool.close().unwrap();pool.close().unwrap();

    let encoding=CombatPool::new(Arc::new(ScenarioCatalog::starter()),config(1,AutoReset::Manual)).unwrap();
    let r=encoding.reset_batch(&[0],&[request()]).unwrap();
    encoding.state.lock().unwrap().slots[0].live.as_mut().unwrap().knowledge.deck=[[0;2];3];
    let f=match encoding.step_batch(&[1],&r.decisions.tokens) { Err(EnvError::Fault { records })=>records,_=>panic!("expected encoding fault") };
    assert_eq!(f[0].stage,FaultStage::Encoding);assert_eq!(f[0].slot,Some(0));
    assert_eq!(encoding.state.lock().unwrap().slots[0].live.as_ref().unwrap().steps,1);

    let poisoned=CombatPool::new(Arc::new(ScenarioCatalog::starter()),PoolConfig::default()).unwrap();
    let _=std::panic::catch_unwind(|| { let _guard=poisoned.state.lock().unwrap();panic!("test unwind"); });
    assert!(matches!(poisoned.read_rows(&[0]),Err(EnvError::State { reason:StateReason::Faulted,.. })));
    poisoned.close().unwrap();poisoned.close().unwrap();
}

// Catches slot-dependent gameplay identity, cached mapping drift on EndTurn, and export aliasing.
#[test]
fn starter_single_batch_and_core_boundaries_agree() {
    for id in ["starter-cultist-a0-v1","starter-jaw-worm-a0-v1"] {
        let catalog=Arc::new(ScenarioCatalog::starter());
        let mut c=config(3,AutoReset::Manual);c.partition=Partition::Test;
        let batch=CombatPool::new(catalog.clone(),c.clone()).unwrap();c.batch_size=1;
        let single=CombatPool::new(catalog.clone(),c).unwrap();
        let request=ResetRequest { seed:Some(seed()),scenario_id:Some(id.into()) };
        let r=batch.reset_batch(&[2,0,1],&vec![request.clone();3]).unwrap();
        assert_eq!(r.decisions.slot_indices,[2,0,1]);
        let mut one=single.reset_batch(&[0],&[request]).unwrap().decisions;
        let job=crate::episode::EpisodeSequence::default().prepare_reset(Some(&seed()),Some(id),&catalog,Partition::Test).unwrap().job;
        let mut core=job.instantiate().unwrap();
        let mut knowledge=crate::knowledge::PublicKnowledge::from_template(job.template());
        let mut menu=crate::actions::CandidateMenu::from_combat(&core).unwrap();
        let mut row=crate::observation::encode(&core,&knowledge,&menu,&job.template().context).unwrap();
        assert_eq!(row.player,[80,80,0,3,3,5,1]);
        assert_eq!(row.hand_kind,[1,2,1,2,3,0,0,0,0,0]);
        assert_eq!(one.observations,ObservationBatch::from_rows(&[row.clone()]).unwrap());
        let retained_initial=one.clone();
        let mut many=batch.read_rows(&[0,1,2]).unwrap();
        let mut retained_step=None;
        let mut steps=0;
        loop {
            let action_index=if row.action_mask[1]==1 {1} else {0};
            let accepted=menu.resolve(action_index).unwrap().clone();
            core.step(&accepted.core).unwrap();
            knowledge.observe_completed(&row,&accepted,&core.public_projection().unwrap()).unwrap();
            menu=crate::actions::CandidateMenu::from_combat(&core).unwrap();
            row=crate::observation::encode(&core,&knowledge,&menu,&job.template().context).unwrap();
            let a=single.step_batch(&[action_index],&one.tokens).unwrap();
            let b=batch.step_batch(&[action_index;3],&many.tokens).unwrap();
            steps+=1;
            assert!(steps<1000,"fixture should finish, not impose an environment cap");
            assert_eq!(a.current.infos[0].episode_steps,steps);
            assert_eq!(a.current.observations,ObservationBatch::from_rows(&[row.clone()]).unwrap());
            assert_eq!(b.current.observations,ObservationBatch::from_rows(&vec![row.clone();3]).unwrap());
            assert_eq!(b.current.infos,vec![a.current.infos[0].clone();3]);
            assert_eq!(b.rewards,vec![a.rewards[0];3]);
            assert_eq!(b.terminated,vec![a.terminated[0];3]);
            assert_eq!(b.truncated,[false;3]);
            {
                let s=batch.state.lock().unwrap();
                for slot in &s.slots { assert_eq!(slot.live.as_ref().unwrap().combat.snapshot(),core.snapshot()); }
            }
            let terminal=a.terminated[0];
            if retained_step.is_none() {retained_step=Some((a.clone(),a.clone()));}
            one=a.current;many=b.current;
            if terminal {break;}
        }
        single.reset_batch(&[0],&[ResetRequest::default()]).unwrap();single.close().unwrap();batch.close().unwrap();
        assert_eq!(retained_initial.infos[0].episode_steps,0);
        assert_eq!(retained_initial.observations.tensors[0].values,TensorValues::I32(vec![80,80,0,3,3,5,1]));
        let (original,copy)=retained_step.unwrap();assert_eq!(original,copy);
    }
}

// Catches reserving one past the last valid serial or treating checked exhaustion as a cap.
#[test]
fn last_serial_and_accepted_count_are_usable_once() {
    let pool=CombatPool::new(Arc::new(ScenarioCatalog::starter()),PoolConfig::default()).unwrap();
    let reset=pool.reset_batch(&[0],&[request()]).unwrap();
    {
        let mut state=pool.state.lock().unwrap();
        state.serials=crate::identity::CheckedCounter::new(u64::MAX);
        state.slots[0].live.as_mut().unwrap().steps=u64::MAX-1;
    }
    let r=pool.step_batch(&[1],&reset.decisions.tokens).unwrap();
    assert_eq!(r.current.tokens[0][1],u64::MAX);
    assert_eq!(r.current.infos[0].episode_steps,u64::MAX);
    assert_eq!(r.truncated,[false]);assert_eq!(r.terminated,[false]);
    let before=saved(&pool);
    assert!(matches!(pool.step_batch(&[0],&r.current.tokens),Err(EnvError::Setup { .. })));
    assert_eq!(saved(&pool),before);
}
