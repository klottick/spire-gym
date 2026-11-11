//! M35 R8: native `PublicSearch::step_many` agreement, privacy, fault and preflight corpus.
use super::m34s_transplant::{standard_combat_session, standard_entry};
use super::standard_traversal::{short_creation, simple_action, standard_session, CHARACTERS};
use crate::*;
use serde_json::{json, Value};
use spire_core::content::{Catalog, Character};

const SEED: u64 = 0x35_10_8;
const WIDE: SearchLimits = SearchLimits { max_live_branches:64,max_branches:1_000,max_steps_per_branch:5_000,max_total_steps:1_000_000 };
fn standalone(character: Character, ascension: u8, encounter: &str, seed: u64) -> (CombatTrainingSession, CombatPolicyBoundary) {
    let mut s=standard_combat_session(vec![standard_entry("e",character,ascension,encounter,None)]);
    let d=s.reset(Some(seed),None).unwrap(); (s,d)
}
/// M34s P1 natural pair: different true worlds with the same public root, from the 7-Strikes Guardian census.
fn natural_pair(character: Character) -> (PublicRoot, PublicRoot) {
    let strike=Catalog::standard().starter_deck(character).unwrap()[0];
    let mut entry=standard_entry("e",character,0,"spheric_guardian",None);
    entry["deck"]=json!(vec![json!({"card":strike.as_str(),"upgrades":0});7]); entry["relics"]=json!([]);
    let mut seen=std::collections::BTreeMap::<[u8;32],(PublicRoot,spire_core::run::Run)>::new();
    for seed in 0..400u64 {
        let mut session=standard_combat_session(vec![entry.clone()]); let _=session.reset(Some(seed),None).unwrap();
        let root=session.public_root().unwrap(); let world=session.run_for_test().unwrap().clone(); let digest=root.digest();
        if let Some((prior,prior_world))=seen.get(&digest) { if prior==&root && prior_world!=&world { return (prior.clone(),root); } }
        else { seen.insert(digest,(root,world)); }
    }
    panic!("M34s natural-pair census found no public-equivalent distinct worlds");
}
fn limits(live: usize, branches: u64, each: u64, total: u64) -> SearchLimits {
    SearchLimits { max_live_branches: live, max_branches: branches, max_steps_per_branch: each, max_total_steps: total }
}
fn search(root: PublicRoot, workers: usize, l: SearchLimits) -> PublicSearch {
    PublicSearch::new(root, SearchConfig { research_seed: SEED, limits: l }).unwrap().with_workers(workers).unwrap()
}
fn run_root(character: Character, ascension: u8, seed: u8) -> PublicRoot {
    let (mut s, _) = standard_session(character, ascension, seed, short_creation());
    let mut b = s.decision().unwrap();
    while s.run_for_test().unwrap().decision().unwrap().combat.is_none() {
        b = s.step(b.token, RunActionIndex(simple_action(&b.observation) as i64)).unwrap().boundary;
    }
    s.public_root().unwrap()
}
fn roots() -> Vec<(String, PublicRoot)> {
    let mut out = vec![];
    for (ci, c) in CHARACTERS.into_iter().enumerate() { for (ai, a) in [0u8, 20].into_iter().enumerate() {
        let seed = 0x40 + (ci * 2 + ai) as u8;
        out.push((format!("{c:?} A{a} Standard Run"), run_root(c, a, seed)));
        let encounter = if (ci + ai) % 2 == 0 { "jaw_worm" } else { "cultist" };
        let (s, _) = standalone(c, a, encounter, u64::from(seed));
        out.push((format!("{c:?} A{a} {encounter}"), s.public_root().unwrap()));
    }}
    out
}
fn branch_set(s: &mut PublicSearch) -> Vec<BranchId> {
    let mut b = (0..8).map(|k| s.branch(Some(k)).unwrap()).collect::<Vec<_>>();
    b.push(s.branch(Some(3)).unwrap());
    b.push(s.fork(b[0]).unwrap());
    b.push(s.fork(b[1]).unwrap());
    b
}
fn serial_token(t: RunDecisionToken) -> RunDecisionToken { RunDecisionToken([0, t.0[1]]) }
fn normalize(o: BranchStepOutcome) -> BranchStepOutcome {
    match o { BranchStepOutcome::Stepped(mut s) => { s.token = serial_token(s.token); BranchStepOutcome::Stepped(s) }, x => x }
}
fn normalize_decision(mut d: BranchBoundary) -> BranchBoundary { d.token = serial_token(d.token); d }
fn same_state(a: &PublicSearch, ab: &[BranchId], b: &PublicSearch, bb: &[BranchId]) {
    assert_eq!(a.counters(), b.counters());
    for (x, y) in ab.iter().zip(bb) {
        assert_eq!(a.actions(*x).unwrap(), b.actions(*y).unwrap());
        assert_eq!(normalize_decision(a.decision(*x).unwrap()), normalize_decision(b.decision(*y).unwrap()));
    }
}
fn next_legal(s: &mut PublicSearch, b: BranchId, step: u64, k: u64, view: BranchView) -> (usize, BranchStep) {
    let d = s.decision(b).unwrap();
    let first = crate::tests::m35_r2::r1(k, step, &d.observation.action_mask);
    let mut legal = d.observation.action_mask.iter().enumerate().filter_map(|(i, m)| (*m != 0).then_some(i)).collect::<Vec<_>>();
    legal.sort_unstable();
    let start = legal.iter().position(|i| *i == first).unwrap();
    for off in 0..legal.len() {
        let i = legal[(start + off) % legal.len()];
        match s.step(b, d.token, RunActionIndex(i as i64), view) {
            Ok(r) => return (i, r),
            Err(RunSessionError::UnsupportedBoundary(crate::HIDDEN_ORDER_PICK)) => continue,
            Err(e) => panic!("R1 step failed: {e:?}"),
        }
    }
    panic!("no legal action escaped hidden-order refusal")
}
fn live(d: &BranchBoundary) -> bool { d.info["end_reason"] != "step_limit" && d.observation.action_mask.iter().any(|m| *m != 0) }
fn aggressive_index(o: &RunObservation) -> usize {
    let g=o.group("actions").unwrap(); let cols=*g.shape.last().unwrap();
    let verb=|i:usize| { let n=g.values[i*cols]; crate::standard_vocab_manifest()["families"]["verb"].as_array().unwrap().get(usize::try_from(n).unwrap()).and_then(Value::as_str).unwrap_or("") };
    let legal=o.action_mask.iter().enumerate().filter_map(|(i,m)|(*m!=0).then_some(i)).collect::<Vec<_>>();
    legal.iter().copied().find(|i|verb(*i)=="play_card").or_else(||legal.iter().copied().find(|i|verb(*i)=="end_turn")).unwrap_or(legal[0])
}
fn req(s: &PublicSearch, b: BranchId, index: usize) -> BranchStepRequest {
    BranchStepRequest { branch: b, token: s.decision(b).unwrap().token, index: RunActionIndex(index as i64) }
}

#[test]
fn step_many_matches_sequential_steps() {
    let mut count = 0;
    for (label, root) in roots() {
        let l = limits(16, 16, 600, 40_000);
        let (mut seq, mut one, mut four) = (search(root.clone(), 1, l), search(root.clone(), 1, l), search(root, 4, l));
        let (bs, bo, bf) = (branch_set(&mut seq), branch_set(&mut one), branch_set(&mut four));
        let mut done = vec![false; bs.len()];
        for round in 0..600u64 {
            if done.iter().all(|x| *x) { break; }
            let mut requests = vec![];
            let mut expected = vec![None; bs.len()];
            for i in 0..bs.len() { if !done[i] {
                let d = seq.decision(bs[i]).unwrap();
                if !live(&d) { done[i] = true; continue; }
                let view = if round % 10 == 0 { BranchView::Full } else { BranchView::Mask };
                let (action, result) = next_legal(&mut seq, bs[i], round, if i == 8 { 3 } else if i >= 9 { (i - 9) as u64 } else { i as u64 }, view);
                expected[i] = Some(normalize(BranchStepOutcome::Stepped(result)));
                let info = seq.decision(bs[i]).unwrap();
                if !live(&info) { done[i] = true; }
                requests.push((i, action));
            }}
            if requests.is_empty() { continue; }
            // Rotate/reverse the input order each round, then vary call grouping 1, 3, all.
            if round % 2 == 0 { requests.reverse(); }
            else { let n = requests.len(); requests.rotate_left((round as usize) % n); }
            for (arena, branches) in [(&mut one, &bo), (&mut four, &bf)] {
                let chunk = match round % 3 { 0 => 1, 1 => 3, _ => requests.len().max(1) };
                for part in requests.chunks(chunk) {
                    let batch = part.iter().map(|(i, index)| req(arena, branches[*i], *index)).collect::<Vec<_>>();
                    let got = arena.step_many(&batch, if round % 10 == 0 { BranchView::Full } else { BranchView::Mask }).unwrap();
                    for ((i, _), outcome) in part.iter().zip(got) {
                        // The sequential lane uses Full every tenth step too; compare masks separately when its step was Mask.
                        let want = expected[*i].as_ref().unwrap();
                        let got = normalize(outcome);
                        if round % 10 == 0 { assert_eq!(&got, want, "{label} branch {i} round {round}"); }
                        else if let (BranchStepOutcome::Stepped(g), BranchStepOutcome::Stepped(w)) = (&got, want) {
                            assert_eq!((&g.action_mask, g.reward, g.terminated, g.truncated, &g.info), (&w.action_mask, w.reward, w.terminated, w.truncated, &w.info), "{label} branch {i} round {round}");
                            assert!(g.observation.is_none());
                        } else { assert_eq!(&got, want, "{label} branch {i} round {round}"); }
                    }
                }
            }
            // All three states are checked after every round, including decisions and counters.
            same_state(&seq, &bs, &one, &bo); same_state(&seq, &bs, &four, &bf);
        }
        assert!(done.iter().all(|x| *x), "{label}: every branch terminal or truncated");
        count += bs.len();
    }
    assert_eq!(count, 176);
}

#[test]
fn step_many_no_cross_branch_leakage() {
    let (root,twin_root) = natural_pair(Character::Ironclad); let l = limits(8, 16, 300, 5_000);
    let mut together = search(root.clone(), 4, l); let parent = together.branch(Some(0)).unwrap();
    let duplicate = together.branch(Some(0)).unwrap(); let other_k = together.branch(Some(1)).unwrap(); let fork = together.fork(parent).unwrap();
    let mut alone = search(root.clone(), 4, l); let solo = alone.branch(Some(0)).unwrap();
    let mut alone_trace=vec![];
    for round in 0..300u64 {
        let d = together.decision(parent).unwrap(); if !live(&d) { break; }
        let mut indices = [parent, duplicate, other_k, fork].map(|b| {
            let d = together.decision(b).unwrap();
            crate::tests::m35_r2::r1(if b == other_k { 1 } else { 0 }, round, &d.observation.action_mask)
        });
        loop {
            let branches = [parent, duplicate, other_k, fork];
            let reqs = branches.iter().enumerate().map(|(i, branch)| BranchStepRequest { branch:*branch, token:together.decision(*branch).unwrap().token, index:RunActionIndex(indices[i] as i64) }).collect::<Vec<_>>();
            match together.step_many(&reqs, BranchView::Mask) {
                Ok(outcomes) => {
                    let first = normalize(outcomes[0].clone()); assert_eq!(first, normalize(outcomes[1].clone())); assert_eq!(first, normalize(outcomes[3].clone()));
                    let sd = alone.decision(solo).unwrap();
                    assert_eq!(sd.observation.action_mask, d.observation.action_mask);
                    let mut solo_index = crate::tests::m35_r2::r1(0, round, &sd.observation.action_mask);
                    let solo_legal = sd.observation.action_mask.iter().enumerate().filter_map(|(i,m)|(*m!=0).then_some(i)).collect::<Vec<_>>();
                    let solo_outcome = loop { match alone.step_many(&[BranchStepRequest { branch:solo, token:sd.token, index:RunActionIndex(solo_index as i64) }], BranchView::Mask) {
                        Ok(v) => break v[0].clone(),
                        Err(RunSessionError::UnsupportedBoundary(crate::HIDDEN_ORDER_PICK)) => { let pos=solo_legal.iter().position(|j|*j==solo_index).unwrap(); solo_index=solo_legal[(pos+1)%solo_legal.len()]; },
                        Err(e) => panic!("solo step_many failed: {e:?}"),
                    }};
                    assert_eq!(normalize(solo_outcome.clone()), normalize(first.clone()), "alone and batched k=0 outcomes");
                    let BranchStepOutcome::Stepped(so)=solo_outcome else { panic!("unexpected solo fault") };
                    alone_trace.push((solo_index,so.action_mask,so.reward,so.terminated,so.truncated,so.info));
                    break
                }
                Err(RunSessionError::UnsupportedBoundary(crate::HIDDEN_ORDER_PICK)) => {
                    for (i, branch) in [parent, duplicate, other_k, fork].into_iter().enumerate() {
                        let d = together.decision(branch).unwrap();
                        let legal = d.observation.action_mask.iter().enumerate().filter_map(|(j,m)|(*m!=0).then_some(j)).collect::<Vec<_>>();
                        let pos = legal.iter().position(|j| *j == indices[i]).unwrap();
                        indices[i] = legal[(pos + 1) % legal.len()];
                    }
                }
                Err(e)=>panic!("{e:?}"),
            }
        }
        assert_eq!(together.actions(parent).unwrap(), together.actions(duplicate).unwrap());
        assert_eq!(together.actions(parent).unwrap(), together.actions(fork).unwrap());
        assert_eq!(together.actions(other_k).unwrap(), together.actions(other_k).unwrap());
    }
    // A cross-root pair with identical public roots, stepped independently on two OS threads.
    assert_eq!(root, twin_root);
    let (p1, p2) = (search(root, 4, l), search(twin_root, 4, l));
    std::thread::scope(|scope| {
        let h1 = scope.spawn(move || { let mut s = p1; let b = s.branch(Some(0)).unwrap(); let mut trace = vec![]; for i in 0..300 { if !live(&s.decision(b).unwrap()) { break } let d=s.decision(b).unwrap(); let mut idx=crate::tests::m35_r2::r1(0,i,&d.observation.action_mask); let legal=d.observation.action_mask.iter().enumerate().filter_map(|(j,m)|(*m!=0).then_some(j)).collect::<Vec<_>>(); loop {match s.step_many(&[BranchStepRequest{branch:b,token:d.token,index:RunActionIndex(idx as i64)}],BranchView::Mask){Ok(o)=>{let BranchStepOutcome::Stepped(r)=&o[0] else{panic!("fault")};trace.push((idx,r.action_mask.clone(),r.reward,r.terminated,r.truncated,r.info.clone()));break},Err(RunSessionError::UnsupportedBoundary(crate::HIDDEN_ORDER_PICK))=>idx=legal.iter().copied().find(|j|*j>idx).unwrap_or(legal[0]),Err(e)=>panic!("{e:?}")}} } (s, b, trace) });
        let h2 = scope.spawn(move || { let mut s = p2; let b = s.branch(Some(0)).unwrap(); let mut trace = vec![]; for i in 0..300 { if !live(&s.decision(b).unwrap()) { break } let d = s.decision(b).unwrap(); let mut idx=crate::tests::m35_r2::r1(0,i,&d.observation.action_mask); let legal=d.observation.action_mask.iter().enumerate().filter_map(|(j,m)|(*m!=0).then_some(j)).collect::<Vec<_>>(); loop {match s.step_many(&[BranchStepRequest{branch:b,token:d.token,index:RunActionIndex(idx as i64)}],BranchView::Mask){Ok(o)=>{let BranchStepOutcome::Stepped(r)=&o[0] else{panic!("fault")};trace.push((idx,r.action_mask.clone(),r.reward,r.terminated,r.truncated,r.info.clone()));break},Err(RunSessionError::UnsupportedBoundary(crate::HIDDEN_ORDER_PICK))=>idx=legal.iter().copied().find(|j|*j>idx).unwrap_or(legal[0]),Err(e)=>panic!("{e:?}")}} } (s, b, trace) });
        let (s1, b1, t1) = h1.join().unwrap(); let (s2, b2, t2) = h2.join().unwrap(); assert_eq!(t1, t2); assert_eq!(t1, alone_trace); same_state(&s1, &[b1], &s2, &[b2]);
    });
}

#[test]
fn step_many_privacy_oracle_p3_p4() {
    let (root,twin_root)=natural_pair(Character::Silent);
    let (different_session, _) = standalone(Character::Silent, 0, "cultist", 95);
    let different_root = different_session.public_root().unwrap();
    assert_ne!(root.digest(), different_root.digest());
    let l = limits(16, 32, 300, 8_000); let mut x = search(root.clone(), 4, l); let mut y = search(twin_root, 4, l); let mut z=search(root.clone(),1,l);
    let (a, b, c) = (branch_set(&mut x), branch_set(&mut y), branch_set(&mut z));
    let research = spire_core::run::research_root(SEED);
    let keys = (0..a.len()).map(|i| {
        let k = if i == 8 { 3 } else if i >= 9 { (i - 9) as u64 } else { i as u64 };
        spire_core::run::research_sample_key(&research, &root.digest(), k)
    }).collect::<Vec<_>>();
    for i in 0..a.len() { let k = if i == 8 { 3 } else if i >= 9 { (i - 9) as u64 } else { i as u64 };
        assert_eq!(x.origin_for_test(a[i]), keys[i]); assert_eq!(y.origin_for_test(b[i]), keys[i]);
        assert_ne!(keys[i], spire_core::run::research_sample_key(&research, &different_root.digest(), k));
    }
    for i in 0..a.len() { for j in (i+1)..a.len() {
        let ki = if i == 8 { 3 } else if i >= 9 { (i - 9) as u64 } else { i as u64 };
        let kj = if j == 8 { 3 } else if j >= 9 { (j - 9) as u64 } else { j as u64 };
        if ki != kj { assert_ne!(keys[i], keys[j], "distinct samples {ki} and {kj} have distinct keys"); }
    }}
    assert_eq!(x.origin_for_test(a[8]), x.origin_for_test(a[3]));
    assert_eq!(x.origin_for_test(a[9]), x.origin_for_test(a[0])); assert_eq!(x.origin_for_test(a[10]), x.origin_for_test(a[1]));
    let mut done = vec![false; a.len()];
    for step in 0..300 {
        for i in 0..a.len() { if !done[i] { let k = if i == 8 { 3 } else if i >= 9 { (i - 9) as u64 } else { i as u64 }; let view = if step % 10 == 0 { BranchView::Full } else { BranchView::Mask };
            let d = x.decision(a[i]).unwrap(); if !live(&d) { done[i] = true; continue }
            let run_batch = |search: &mut PublicSearch, branch: BranchId, mut action: usize, view| {
                let d = search.decision(branch).unwrap(); let legal = d.observation.action_mask.iter().enumerate().filter_map(|(j,m)|(*m!=0).then_some(j)).collect::<Vec<_>>();
                loop { match search.step_many(&[BranchStepRequest{branch,token:d.token,index:RunActionIndex(action as i64)}],view) {
                    Ok(v) => break v[0].clone(), Err(RunSessionError::UnsupportedBoundary(crate::HIDDEN_ORDER_PICK)) => { let pos=legal.iter().position(|j|*j==action).unwrap(); action=legal[(pos+1)%legal.len()]; }, Err(e) => panic!("{e:?}")
                }}
            };
            let rx = run_batch(&mut x, a[i], crate::tests::m35_r2::r1(k, step, &d.observation.action_mask), view);
            let d2 = y.decision(b[i]).unwrap(); let ry = run_batch(&mut y, b[i], crate::tests::m35_r2::r1(k, step, &d2.observation.action_mask), view);
            let d3 = z.decision(c[i]).unwrap(); let initial = crate::tests::m35_r2::r1(k, step, &d3.observation.action_mask);
            let mut action=initial; let legal=d3.observation.action_mask.iter().enumerate().filter_map(|(j,m)|(*m!=0).then_some(j)).collect::<Vec<_>>();
            let rz=loop { match z.step(c[i],d3.token,RunActionIndex(action as i64),view) { Ok(r)=>break BranchStepOutcome::Stepped(r), Err(RunSessionError::UnsupportedBoundary(crate::HIDDEN_ORDER_PICK))=>{let pos=legal.iter().position(|j|*j==action).unwrap();action=legal[(pos+1)%legal.len()];},Err(e)=>panic!("{e:?}")} };
            assert_eq!(normalize(rx.clone()), normalize(ry), "P3 paired-root outcome at sample {k}, step {step}");
            assert_eq!(normalize(rx), normalize(rz), "batched/sequential outcome at sample {k}, step {step}");
            done[i] = !live(&x.decision(a[i]).unwrap());
        }}
        if done.iter().all(|z| *z) { break }
    }
    for i in 0..a.len() { assert_eq!(x.actions(a[i]).unwrap(), y.actions(b[i]).unwrap()); assert_eq!(normalize_decision(x.decision(a[i]).unwrap()), normalize_decision(y.decision(b[i]).unwrap()));
        assert_eq!(x.actions(a[i]).unwrap(),z.actions(c[i]).unwrap()); assert_eq!(normalize_decision(x.decision(a[i]).unwrap()),normalize_decision(z.decision(c[i]).unwrap()));
        assert_eq!(x.origin_for_test(a[i]), keys[i]); assert_eq!(y.origin_for_test(b[i]), keys[i]); assert_eq!(z.origin_for_test(c[i]), keys[i]); }
}

#[test]
fn step_many_distribution_p5_workers4() {
    let (s, _) = standalone(Character::Defect, 0, "jaw_worm", 21); let root = s.public_root().unwrap(); let l = limits(64, 64, 20, 1_000);
    let (mut seq, mut batched) = (search(root.clone(), 1, l), search(root, 4, l));
    let (a, b) = ((0..64).map(|k| seq.branch(Some(k)).unwrap()).collect::<Vec<_>>(), (0..64).map(|k| batched.branch(Some(k)).unwrap()).collect::<Vec<_>>());
    let mut traces = vec![vec![]; 64];
    let mut batch_traces = vec![vec![];64];
    for step in 0..3u64 {
        let mut reqs = vec![];
        for k in 0..64 { let da = seq.decision(a[k]).unwrap(); let mask = &da.observation.action_mask; let mut idx = crate::tests::m35_r2::r1(k as u64, step, mask); let legal = mask.iter().enumerate().filter_map(|(i,m)|(*m!=0).then_some(i)).collect::<Vec<_>>();
            let ra = loop { match seq.step(a[k], da.token, RunActionIndex(idx as i64), BranchView::Mask) { Ok(r)=>break r, Err(RunSessionError::UnsupportedBoundary(crate::HIDDEN_ORDER_PICK))=>idx=legal.iter().copied().find(|i|*i>idx).unwrap_or(legal[0]), Err(e)=>panic!("{e:?}") } };
            traces[k].push((ra.action_mask,ra.reward,ra.terminated,ra.truncated,ra.info)); reqs.push(BranchStepRequest{branch:b[k],token:batched.decision(b[k]).unwrap().token,index:RunActionIndex(idx as i64)});
        }
        let got = batched.step_many(&reqs, BranchView::Mask).unwrap();
        for k in 0..64 { let BranchStepOutcome::Stepped(r) = &got[k] else { panic!("unexpected fault") }; let row=(r.action_mask.clone(),r.reward,r.terminated,r.truncated,r.info.clone()); assert_eq!((&row.0,row.1,row.2,row.3,&row.4), (&traces[k][step as usize].0,traces[k][step as usize].1,traces[k][step as usize].2,traces[k][step as usize].3,&traces[k][step as usize].4)); batch_traces[k].push(row); }
    }
    let mut sequential = traces.iter().map(|t|serde_json::to_string(t).unwrap()).collect::<Vec<_>>(); sequential.sort();
    let mut batched = batch_traces.iter().map(|t|serde_json::to_string(t).unwrap()).collect::<Vec<_>>(); batched.sort(); assert_eq!(sequential,batched);
}

#[test]
fn step_many_output_channels_p7() {
    const RESEARCH: u64 = 0x5EED_1234_ABCD_9876; const SAMPLE: u64 = 987_654_321_123; const LIVE: u64 = 0x0DDB_A11C_0FFE_E123;
    let (s, _) = standalone(Character::Ironclad, 20, "two_louse", LIVE); let root = s.public_root().unwrap();
    let mut q = PublicSearch::new(root.clone(), SearchConfig { research_seed: RESEARCH, limits: WIDE }).unwrap().with_workers(4).unwrap(); let id = q.branch(Some(SAMPLE)).unwrap(); let full_id = q.branch(Some(SAMPLE)).unwrap(); let d = q.decision(id).unwrap();
    let key = spire_core::run::research_sample_key(&spire_core::run::research_root(RESEARCH), &root.digest(), SAMPLE);
    let forbidden = [RESEARCH.to_string(), format!("{RESEARCH:x}"), SAMPLE.to_string(), format!("{SAMPLE:x}"), LIVE.to_string(), format!("{LIVE:x}"), crate::combat_policy::hex(&key)];
    let legal=d.observation.action_mask.iter().enumerate().filter_map(|(i,m)|(*m!=0).then_some(i)).collect::<Vec<_>>();
    let mut action=crate::tests::m35_r2::r1(SAMPLE,0,&d.observation.action_mask);
    let out=loop {match q.step_many(&[BranchStepRequest{branch:id,token:d.token,index:RunActionIndex(action as i64)}],BranchView::Mask){Ok(v)=>break v,Err(RunSessionError::UnsupportedBoundary(crate::HIDDEN_ORDER_PICK))=>action=legal.iter().copied().find(|i|*i>action).unwrap_or(legal[0]),Err(e)=>panic!("{e:?}")}};
    let fd=q.decision(full_id).unwrap(); let full_legal=fd.observation.action_mask.iter().enumerate().filter_map(|(i,m)|(*m!=0).then_some(i)).collect::<Vec<_>>(); let mut full_action=crate::tests::m35_r2::r1(SAMPLE,0,&fd.observation.action_mask);
    let full_out=loop {match q.step_many(&[BranchStepRequest{branch:full_id,token:fd.token,index:RunActionIndex(full_action as i64)}],BranchView::Full){Ok(v)=>break v,Err(RunSessionError::UnsupportedBoundary(crate::HIDDEN_ORDER_PICK))=>{full_action=full_legal.iter().copied().find(|i|*i>full_action).unwrap_or(full_legal[0]);},Err(e)=>panic!("{e:?}")}};
    let bad=q.step_many(&[BranchStepRequest{branch:BranchId(u64::MAX),token:d.token,index:RunActionIndex(0)}],BranchView::Mask);
    let spec=q.spec(); let counters=q.counters(); let empty=q.step_many(&[],BranchView::Mask);
    let text = format!("{:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?}", out, full_out, bad.as_ref().err(), q, spec, counters, empty, root);
    for f in forbidden { assert!(!text.contains(&f), "hidden value {f} in {text}"); }
}

#[test]
fn step_many_fault_isolated_to_entry() {
    let (s, _) = standalone(Character::Silent, 0, "jaw_worm", 22); let root = s.public_root().unwrap(); let l = limits(16, 20, 100, 100);
    let (mut clean, mut faulted) = (search(root.clone(), 4, l), search(root, 4, l));
    let (ca, cb) = (branch_set(&mut clean), branch_set(&mut faulted));
    let fault0 = faulted.decision(cb[0]).unwrap();
    let (accepted, _) = next_legal(&mut clean, ca[0], 0, 0, BranchView::Mask);
    faulted.step(cb[0], fault0.token, RunActionIndex(accepted as i64), BranchView::Mask).unwrap();
    assert_eq!(clean.actions(ca[0]).unwrap(),faulted.actions(cb[0]).unwrap());
    let bad = faulted.decision(cb[0]).unwrap(); let sib = faulted.decision(cb[1]).unwrap();
    spire_core::run::diagnostics::supplied_fault(faulted.branch_run_for_test(cb[0]));
    let a = crate::tests::m35_r2::r1(0, 1, &bad.observation.action_mask) as i64; let b = crate::tests::m35_r2::r1(1, 0, &sib.observation.action_mask) as i64;
    let clean_out = clean.step_many(&[req(&clean,ca[1],b as usize)],BranchView::Mask).unwrap();
    let fault_out = faulted.step_many(&[BranchStepRequest{branch:cb[0],token:bad.token,index:RunActionIndex(a)},BranchStepRequest{branch:cb[1],token:sib.token,index:RunActionIndex(b)}],BranchView::Mask).unwrap();
    assert!(matches!(&fault_out[0],BranchStepOutcome::Fault{error:RunSessionError::DeliveryFault(crate::BRANCH_CORE_FAULT),stage:"step",..}));
    assert_eq!(normalize(fault_out[1].clone()),normalize(clean_out[0].clone())); assert_eq!(faulted.actions(cb[0]).unwrap(),clean.actions(ca[0]).unwrap());
    assert_eq!(faulted.step(cb[0],bad.token,RunActionIndex(a),BranchView::Mask).err(),Some(RunSessionError::BranchFaulted));
    assert_eq!(faulted.step_many(&[BranchStepRequest{branch:cb[0],token:bad.token,index:RunActionIndex(a)}],BranchView::Mask).err(),Some(RunSessionError::BranchFaulted));
}

#[test]
fn step_many_preflight_rejects_without_effect() {
    fn fresh() -> (PublicSearch, Vec<BranchId>) { let (s,_) = standalone(Character::Ironclad,0,"jaw_worm",31); let mut q=search(s.public_root().unwrap(),1,limits(16,20,10,100)); let ids=branch_set(&mut q); (q,ids) }
    let (mut q, ids) = fresh(); let d=q.decision(ids[0]).unwrap(); let mut state=q.counters(); let mut ds=ids.iter().map(|b|q.decision(*b).unwrap()).collect::<Vec<_>>();
    let r=req(&q,ids[0],0); assert!(matches!(q.step_many(&[r,r],BranchView::Mask),Err(RunSessionError::Input(_))));
    let (foreign_session,_) = standalone(Character::Watcher,0,"jaw_worm",30); let mut foreign_search=search(foreign_session.public_root().unwrap(),1,limits(2,2,10,100)); let foreign=foreign_search.branch(None).unwrap();
    assert!(ids.iter().all(|b|*b!=foreign),"foreign branch id comes from a distinct search");
    let bad=BranchStepRequest{branch:foreign,token:d.token,index:RunActionIndex(0)}; assert_eq!(q.step_many(&[bad],BranchView::Mask).err(),Some(RunSessionError::Input("unknown branch")));
    assert_eq!(q.step_many(&[BranchStepRequest{branch:ids[0],token:RunDecisionToken([d.token.0[0],99]),index:RunActionIndex(0)}],BranchView::Mask).err(),Some(RunSessionError::StaleToken));
    let d1=q.decision(ids[1]).unwrap(); let stale=BranchStepRequest{branch:ids[1],token:RunDecisionToken([d1.token.0[0],99]),index:RunActionIndex(0)};
    assert_eq!(q.step_many(&[r,stale,bad],BranchView::Mask).err(),Some(RunSessionError::StaleToken),"the first invalid entry wins after a valid prefix");
    assert_eq!(q.step_many(&[bad,stale],BranchView::Mask).err(),Some(RunSessionError::Input("unknown branch")),"the first invalid entry wins");
    assert_eq!(q.counters(),state); for (b, old) in ids.iter().zip(&ds) { assert_eq!(q.decision(*b).unwrap(),*old); }
    q.close(ids[0]).unwrap(); state=q.counters(); ds=ids[1..].iter().map(|b|q.decision(*b).unwrap()).collect();
    assert_eq!(q.step_many(&[BranchStepRequest{branch:ids[0],token:d.token,index:RunActionIndex(0)}],BranchView::Mask).err(),Some(RunSessionError::Closed)); assert_eq!(q.counters(),state); for (b,old) in ids[1..].iter().zip(&ds){assert_eq!(q.decision(*b).unwrap(),*old)}
    // Faulted, terminal, truncated and budget failures each reject the whole call without changing peers.
    let (s,_) = standalone(Character::Silent,0,"jaw_worm",32); let mut f=search(s.public_root().unwrap(),1,limits(4,4,10,100)); let b=f.branch(None).unwrap(); let peer=f.branch(Some(1)).unwrap(); let d=f.decision(b).unwrap(); let pd=f.decision(peer).unwrap(); let idx=d.observation.action_mask.iter().position(|m|*m!=0).unwrap(); spire_core::run::diagnostics::supplied_fault(f.branch_run_for_test(b)); let _=f.step(b,d.token,RunActionIndex(idx as i64),BranchView::Mask); let before=f.counters(); let peer_before=f.decision(peer).unwrap(); assert_eq!(f.step_many(&[BranchStepRequest{branch:b,token:d.token,index:RunActionIndex(idx as i64)},BranchStepRequest{branch:peer,token:pd.token,index:RunActionIndex(pd.observation.action_mask.iter().position(|m|*m!=0).unwrap() as i64)}],BranchView::Mask).err(),Some(RunSessionError::BranchFaulted)); assert_eq!(f.counters(),before); assert_eq!(f.decision(peer).unwrap(),peer_before);
    let (s,_) = standalone(Character::Defect,0,"cultist",34); let mut t=search(s.public_root().unwrap(),1,limits(4,4,1,100)); let x=t.branch(None).unwrap(); let trunc_peer=t.branch(Some(1)).unwrap(); let d=t.decision(x).unwrap(); let pd=t.decision(trunc_peer).unwrap(); let idx=aggressive_index(&d.observation); let r=t.step(x,d.token,RunActionIndex(idx as i64),BranchView::Mask).unwrap(); assert!(r.truncated); let before=t.counters(); let decision=t.decision(x).unwrap(); let peer_before=t.decision(trunc_peer).unwrap(); let pidx=aggressive_index(&peer_before.observation); assert_eq!(t.step_many(&[BranchStepRequest{branch:x,token:decision.token,index:RunActionIndex(0)},BranchStepRequest{branch:trunc_peer,token:pd.token,index:RunActionIndex(pidx as i64)}],BranchView::Mask).err(),Some(RunSessionError::Terminal)); assert_eq!(t.counters(),before); assert_eq!(t.decision(x).unwrap(),decision); assert_eq!(t.decision(trunc_peer).unwrap(),peer_before);
    let (s,_) = standalone(Character::Ironclad,0,"cultist",35); let mut terminal=search(s.public_root().unwrap(),1,limits(4,4,2_000,2_000)); let x=terminal.branch(None).unwrap(); let mut ended=false;
    for _ in 0..2_000 { let d=terminal.decision(x).unwrap(); if !live(&d){ended=true;break} let idx=aggressive_index(&d.observation); match terminal.step(x,d.token,RunActionIndex(idx as i64),BranchView::Mask) { Ok(r) if r.terminated=>{ended=true;break},Ok(_)=>{},Err(RunSessionError::UnsupportedBoundary(crate::HIDDEN_ORDER_PICK))=>{let legal=d.observation.action_mask.iter().enumerate().filter_map(|(i,m)|(*m!=0).then_some(i)).collect::<Vec<_>>(); let mut done=false; for idx in legal {if let Ok(r)=terminal.step(x,d.token,RunActionIndex(idx as i64),BranchView::Mask){done=r.terminated;break}} if done{ended=true;break}},Err(e)=>panic!("{e:?}")} }
    assert!(ended,"cultist branch reaches terminal"); let td=terminal.decision(x).unwrap(); let terminal_peer=terminal.branch(Some(1)).unwrap(); let tpd=terminal.decision(terminal_peer).unwrap(); let before=terminal.counters(); let peer_before=terminal.decision(terminal_peer).unwrap(); assert_eq!(terminal.step_many(&[BranchStepRequest{branch:x,token:td.token,index:RunActionIndex(0)},BranchStepRequest{branch:terminal_peer,token:tpd.token,index:RunActionIndex(aggressive_index(&tpd.observation) as i64)}],BranchView::Mask).err(),Some(RunSessionError::Terminal)); assert_eq!(terminal.counters(),before); assert_eq!(terminal.decision(x).unwrap(),td); assert_eq!(terminal.decision(terminal_peer).unwrap(),peer_before);
    let (s,_) = standalone(Character::Watcher,0,"jaw_worm",36); let mut budget=search(s.public_root().unwrap(),1,limits(4,4,10,3)); let p=budget.branch(None).unwrap(); let peer=budget.branch(Some(1)).unwrap(); let pd=budget.decision(p).unwrap(); let qd=budget.decision(peer).unwrap(); let ix=pd.observation.action_mask.iter().position(|m|*m!=0).unwrap(); let peer_ix=qd.observation.action_mask.iter().position(|m|*m!=0).unwrap(); budget.step(p,pd.token,RunActionIndex(ix as i64),BranchView::Mask).unwrap(); let now=budget.decision(p).unwrap(); budget.step(p,now.token,RunActionIndex(now.observation.action_mask.iter().position(|m|*m!=0).unwrap() as i64),BranchView::Mask).unwrap(); let p_before=budget.decision(p).unwrap(); let peer_before=budget.decision(peer).unwrap(); let before=budget.counters(); assert!(matches!(budget.step_many(&[BranchStepRequest{branch:p,token:p_before.token,index:RunActionIndex(p_before.observation.action_mask.iter().position(|m|*m!=0).unwrap() as i64)},BranchStepRequest{branch:peer,token:peer_before.token,index:RunActionIndex(peer_ix as i64)}],BranchView::Mask),Err(RunSessionError::Budget(_)))); assert_eq!(budget.counters(),before); assert_eq!(budget.decision(p).unwrap(),p_before); assert_eq!(budget.decision(peer).unwrap(),peer_before);
}

#[test]
fn step_many_fork_step_count_inherited() {
    let (s,_) = standalone(Character::Watcher,0,"jaw_worm",44); let root=s.public_root().unwrap(); let l=limits(8,8,4,100);
    let (mut a,mut b)=(search(root.clone(),1,l),search(root,4,l)); let (pa,pb)=(a.branch(None).unwrap(),b.branch(None).unwrap());
    for step in 0..2 { let da=a.decision(pa).unwrap(); let db=b.decision(pb).unwrap(); let mut i=crate::tests::m35_r2::r1(0,step,&da.observation.action_mask); let legal=da.observation.action_mask.iter().enumerate().filter_map(|(i,m)|(*m!=0).then_some(i)).collect::<Vec<_>>(); let ra=loop{match a.step(pa,da.token,RunActionIndex(i as i64),BranchView::Mask){Ok(x)=>break x,Err(RunSessionError::UnsupportedBoundary(crate::HIDDEN_ORDER_PICK))=>i=legal.iter().copied().find(|j|*j>i).unwrap_or(legal[0]),Err(e)=>panic!("{e:?}")}}; let _=ra; let rb=b.step_many(&[BranchStepRequest{branch:pb,token:db.token,index:RunActionIndex(i as i64)}],BranchView::Mask).unwrap(); assert_eq!(normalize(rb[0].clone()),normalize(BranchStepOutcome::Stepped(ra))); }
    let (fa,fb)=(a.fork(pa).unwrap(),b.fork(pb).unwrap());
    for step in 2..4 { let da=a.decision(fa).unwrap(); let db=b.decision(fb).unwrap(); let mut i=crate::tests::m35_r2::r1(0,step,&da.observation.action_mask); let legal=da.observation.action_mask.iter().enumerate().filter_map(|(i,m)|(*m!=0).then_some(i)).collect::<Vec<_>>(); let ra=loop{match a.step(fa,da.token,RunActionIndex(i as i64),BranchView::Mask){Ok(x)=>break x,Err(RunSessionError::UnsupportedBoundary(crate::HIDDEN_ORDER_PICK))=>i=legal.iter().copied().find(|j|*j>i).unwrap_or(legal[0]),Err(e)=>panic!("{e:?}")}}; let rb=b.step_many(&[BranchStepRequest{branch:fb,token:db.token,index:RunActionIndex(i as i64)}],BranchView::Mask); assert_eq!(normalize(rb.unwrap()[0].clone()),normalize(BranchStepOutcome::Stepped(ra))); }
    assert!(a.decision(fa).unwrap().info["end_reason"]=="step_limit"); assert_eq!(b.step_many(&[BranchStepRequest{branch:fb,token:b.decision(fb).unwrap().token,index:RunActionIndex(0)}],BranchView::Mask).err(),Some(RunSessionError::Terminal));
}

#[test]
fn step_many_uses_shared_pool() {
    let _ = crate::workers::Scheduler::shared(1).unwrap(); assert!(matches!(crate::workers::Scheduler::shared(1).unwrap(),crate::workers::Scheduler::Serial));
    assert_eq!(PublicSearch::new({let (s,_)=standalone(Character::Ironclad,0,"jaw_worm",55);s.public_root().unwrap()},SearchConfig{research_seed:1,limits:WIDE}).unwrap().with_workers(0).err(),Some(RunSessionError::Input("search workers must be 1..=256")) /* M36 amendment (spire-dmm M5): was "search workers must be positive" */);
    let x=crate::workers::Scheduler::shared(4).unwrap(); let y=crate::workers::Scheduler::shared(4).unwrap(); match (x,y) { (crate::workers::Scheduler::Parallel(a),crate::workers::Scheduler::Parallel(b))=>assert!(std::sync::Arc::ptr_eq(&a,&b)), _=>panic!("workers=4 uses a shared pool") }
    let (s,_) = standalone(Character::Ironclad,0,"jaw_worm",56); let root=s.public_root().unwrap(); let l=limits(4,4,30,100); let mut serial=search(root.clone(),1,l); let mut worker=search(root,4,l); let a=serial.branch(None).unwrap(); let b=worker.branch(None).unwrap(); let d=serial.decision(a).unwrap(); let idx=crate::tests::m35_r2::r1(0,0,&d.observation.action_mask); let expected=serial.step(a,d.token,RunActionIndex(idx as i64),BranchView::Mask).unwrap(); let db=worker.decision(b).unwrap(); let got=worker.step_many(&[BranchStepRequest{branch:b,token:db.token,index:RunActionIndex(idx as i64)}],BranchView::Mask).unwrap(); assert_eq!(normalize(got[0].clone()),normalize(BranchStepOutcome::Stepped(expected)));
}
