//! Literal fixtures catch framing, endian, consumption and sampling regressions.
use super::super::rng::*;
use serde_json::Value;
use std::collections::BTreeSet;

const END: u128 = 1_u128 << 64;

fn fixture() -> Value {
    serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/python/fixtures/combat/project_rng_v1.json"
    )))
    .unwrap()
}

fn text(v: &Value) -> &str { v.as_str().unwrap() }
fn hex(s: &str) -> Vec<u8> {
    (0..s.len()).step_by(2).map(|i| u8::from_str_radix(&s[i..i+2], 16).unwrap()).collect()
}
fn domain(s: &str) -> Domain {
    match s {
        "combat.shuffle" => Domain::CombatShuffle,
        "combat.enemy.hp" => Domain::CombatEnemyHp,
        "combat.enemy.ai" => Domain::CombatEnemyAi,
        _ => panic!("unknown fixture domain"),
    }
}
fn partition(s: &str) -> Partition {
    match s {
        "train" => Partition::Train,
        "selection" => Partition::Selection,
        "final" => Partition::Final,
        "test" => Partition::Test,
        _ => panic!("unknown fixture partition"),
    }
}
fn state() -> StreamState {
    make_stream(&[0;32], Partition::Test, "episode", &StreamId::new(Domain::CombatShuffle, vec!["combat".into()]).unwrap()).unwrap()
}
fn at(s: &StreamState, p: u128) -> StreamState {
    StreamState::new(s.id().clone(), *s.key(), p).unwrap()
}

#[test]
fn fixture_known_answers() {
    let f = fixture();
    assert_eq!(RNG_CONTRACT_ID, text(&f["contract_id"]));
    for case in f["primitive_cases"].as_array().unwrap() {
        assert_eq!(hmac_sha256(&hex(text(&case["key_hex"])), &hex(text(&case["message_hex"]))).as_slice(), hex(text(&case["digest_hex"])));
    }
    let root: [u8;32] = hex(text(&f["root_seed_hex"])).try_into().unwrap();
    let mut shuffle_stream = None;
    for row in f["streams"].as_array().unwrap() {
        let scope = row["scope"].as_array().unwrap().iter().map(|s| text(s).to_owned()).collect();
        let id = StreamId::new(domain(text(&row["domain"])), scope).unwrap();
        let p = partition(text(&row["partition"]));
        let s = make_stream(&root, p, text(&row["episode_id"]), &id).unwrap();
        assert_eq!(s.id(), &id);
        assert_eq!(s.word_index(), 0);
        assert_eq!(s.key().as_slice(), hex(text(&row["key_hex"])));
        // Independently assemble the documented frame and check its fixed bytes.
        let mut fields = vec![p.as_str(), text(&row["episode_id"]), id.domain().as_str()];
        fields.extend(id.scope().iter().map(String::as_str));
        let mut frame = (fields.len() as u32).to_be_bytes().to_vec();
        for field in fields {
            frame.extend((field.len() as u32).to_be_bytes());
            frame.extend(field.as_bytes());
        }
        assert_eq!(frame, hex(text(&row["frame_hex"])));
        let mut message = b"spire-lab/rng/v1/stream\0".to_vec();
        message.extend(frame);
        assert_eq!(hmac_sha256(&root, &message), *s.key());
        for (index, digest) in row["blocks"].as_object().unwrap() {
            let mut message = b"spire-lab/rng/v1/block\0".to_vec();
            message.extend(index.parse::<u64>().unwrap().to_be_bytes());
            let expected = hex(text(digest));
            assert_eq!(hmac_sha256(s.key(), &message).as_slice(), expected);
            for lane in 0..4 {
                let cursor = index.parse::<u128>().unwrap()*4 + lane;
                let (word, next) = draw_u64(&at(&s, cursor)).unwrap();
                assert_eq!(word.to_be_bytes().as_slice(), &expected[lane as usize*8..lane as usize*8+8]);
                assert_eq!(next.word_index(), cursor+1);
            }
        }
        for word in row["words"].as_array().unwrap() {
            let p = text(&word["index"]).parse().unwrap();
            assert_eq!(draw_u64(&at(&s,p)).unwrap().0, u64::from_str_radix(text(&word["hex"]),16).unwrap());
        }
        if text(&row["id"]) == text(&f["integration"]["stream_id"]) { shuffle_stream = Some(s); }
    }
    let s = shuffle_stream.unwrap();
    let (a,s) = uniform_below(&s,10).unwrap();
    let (b,s) = bernoulli(&s,2,5).unwrap();
    let (c,s) = weighted_index(&s,&[0,2,0,3]).unwrap();
    let (d,s) = shuffle(&s,&["a","b","c"]).unwrap();
    assert_eq!(serde_json::json!([a,b,c,d]), f["integration"]["values"]);
    assert_eq!(s.word_index(), text(&f["integration"]["next_word_index"]).parse::<u128>().unwrap());
}

#[test]
fn all_lanes_boundary_and_final_cursor() {
    let s = state();
    for p in [0,1,2,3,4,END-1] {
        let input = at(&s,p);
        let copy = input.clone();
        let result = draw_u64(&input).unwrap();
        assert_eq!(input,copy);
        assert_eq!(result.1.word_index(), p+1);
        assert_eq!(draw_u64(&copy).unwrap(),result);
    }
    let end = at(&s,END);
    assert_eq!(draw_u64(&end),Err(RngError::Exhausted {state:end.clone()}));
    assert_eq!(end.word_index(),END);
    assert!(StreamState::new(s.id().clone(),*s.key(),END+1).is_err());
}

#[test]
fn scripted_rejection_consumption() {
    let mut words = [u64::MAX,7].into_iter();
    let mut count = 0;
    assert_eq!(uniform_from_words(10,&mut || {count+=1; Ok(words.next().unwrap())}).unwrap(),7);
    assert_eq!(count,2);
    let mut cursor = at(&state(),END-1);
    let mut next = || {
        if cursor.word_index() == END { return Err(RngError::Exhausted {state:cursor.clone()}); }
        cursor = at(&cursor,END);
        Ok(u64::MAX)
    };
    assert_eq!(uniform_from_words(10,&mut next),Err(RngError::Exhausted {state:at(&state(),END)}));
    // Real multi-draw exhaustion: force a swap then exhaust on the next swap.
    let last = at(&state(),END-1);
    let items = vec![1, 2, 3];
    let original = items.clone();
    assert_eq!(shuffle(&last,&items),Err(RngError::Exhausted {state:at(&last,END)}));
    assert_eq!(items, original);
    assert_eq!(last.word_index(),END-1);
}

#[test]
fn invalid_and_zero_word_cases() {
    let s = at(&state(),END);
    for b in [0,END+1,u128::MAX] { assert!(matches!(uniform_below(&s,b),Err(RngError::Invalid(_)))); }
    for (n,d) in [(0,0),(2,1),(0,END+1),(END+1,END)] { assert!(matches!(bernoulli(&s,n,d),Err(RngError::Invalid(_)))); }
    for weights in [&[][..],&[0,0],&[END,1],&[u128::MAX,1]] { assert!(matches!(weighted_index(&s,weights),Err(RngError::Invalid(_)))); }
    assert_eq!(uniform_below(&s,1).unwrap(),(0,s.clone()));
    assert_eq!(bernoulli(&s,0,END).unwrap(),(false,s.clone()));
    assert_eq!(bernoulli(&s,END,END).unwrap(),(true,s.clone()));
    assert_eq!(weighted_index(&s,&[0,1,0]).unwrap(),(1,s.clone()));
    assert_eq!(shuffle::<i32>(&s,&[]).unwrap(),(vec![],s.clone()));
    assert_eq!(shuffle(&s,&[7]).unwrap(),(vec![7],s.clone()));
    for bad in ["".to_owned(),"a b".into(),"\n".into(),"é".into(),"\u{7f}".into(),"x".repeat(256)] {
        assert!(StreamId::new(Domain::CombatShuffle,vec![bad.clone()]).is_err());
        assert!(make_stream(&[0;32],Partition::Test,&bad,s.id()).is_err());
    }
    for d in [Domain::CombatShuffle,Domain::CombatEnemyHp,Domain::CombatEnemyAi] {
        for n in 0..4 {
            let expected = if d == Domain::CombatShuffle {1} else {2};
            assert_eq!(StreamId::new(d,vec!["x".into();n]).is_ok(),n==expected);
        }
    }
    let valid = "~".repeat(255);
    assert!(StreamId::new(Domain::CombatShuffle,vec![valid.clone()]).is_ok());
    assert!(make_stream(&[0;32],Partition::Test,&valid,s.id()).is_ok());
    let mut never = || -> Result<u64,RngError> {panic!("validation/endpoint sampled")};
    assert!(uniform_from_words(0,&mut never).is_err());
    assert_eq!(uniform_from_words(1,&mut never).unwrap(),0);
    assert_eq!(shuffle_from_words::<i32>(&[],&mut never).unwrap(),Vec::<i32>::new());
    assert_eq!(shuffle_from_words(&[1],&mut never).unwrap(),vec![1]);
    assert!(weighted_from_ticket(&[1],1).is_err());
}

#[test]
fn domain_isolation_and_replay() {
    let mut keys = BTreeSet::new();
    for p in [Partition::Train,Partition::Selection,Partition::Final,Partition::Test] {
        for d in [Domain::CombatShuffle,Domain::CombatEnemyHp,Domain::CombatEnemyAi] {
            let scope = if d == Domain::CombatShuffle {vec!["combat".into()]} else {vec!["combat".into(),"enemy".into()]};
            let id = StreamId::new(d,scope).unwrap();
            let s = make_stream(&[0;32],p,"episode",&id).unwrap();
            assert!(keys.insert(*s.key()));
            assert_eq!(s,make_stream(&[0;32],p,"episode",&id).unwrap());
            assert_ne!(s.key(),make_stream(&[1;32],p,"episode",&id).unwrap().key());
            assert_ne!(s.key(),make_stream(&[0;32],p,"other",&id).unwrap().key());
        }
    }
    let a = state();
    let id = StreamId::new(Domain::CombatEnemyAi,vec!["combat".into(),"enemy".into()]).unwrap();
    let b = make_stream(&[0;32],Partition::Test,"episode",&id).unwrap();
    let other_id = StreamId::new(Domain::CombatEnemyAi,vec!["combat".into(),"other".into()]).unwrap();
    assert_ne!(b.key(),make_stream(&[0;32],Partition::Test,"episode",&other_id).unwrap().key());
    let replay = shuffle(&a,&[1,2,3,4,5]).unwrap();
    let (_,b1) = draw_u64(&b).unwrap();
    let _ = shuffle(&b1,&[1,2,3,4,5,6]).unwrap();
    assert_eq!(shuffle(&a,&[1,2,3,4,5]).unwrap(),replay);
    assert_eq!(b.word_index(),0);
    let left = StreamId::new(Domain::CombatEnemyAi,vec!["ab".into(),"c".into()]).unwrap();
    let right = StreamId::new(Domain::CombatEnemyAi,vec!["a".into(),"bc".into()]).unwrap();
    assert_ne!(make_stream(&[0;32],Partition::Test,"episode",&left).unwrap().key(),make_stream(&[0;32],Partition::Test,"episode",&right).unwrap().key());
}

#[test]
fn weighted_small_cases() {
    for (ticket, expected) in [1, 1, 3, 3, 3].into_iter().enumerate() {
        assert_eq!(weighted_from_ticket(&[0, 2, 0, 3], ticket as u64).unwrap(), expected);
    }
    for a in 0..=3_u128 { for b in 0..=3_u128 { for c in 0..=3_u128 {
        let weights = [a,b,c];
        let mut counts = [0;3];
        for ticket in 0..(a+b+c) as u64 { counts[weighted_from_ticket(&weights,ticket).unwrap()] += 1; }
        assert_eq!(counts,weights);
    } } }
    assert_eq!(weighted_from_ticket(&[0,END,0],u64::MAX).unwrap(),1);
    let s = state();
    assert_eq!(uniform_below(&s,END).unwrap(),draw_u64(&s).unwrap());
    assert_eq!(weighted_index(&s,&[END]).unwrap().1.word_index(),1);
}

#[test]
fn three_item_all_shuffle_paths() {
    let mut permutations = BTreeSet::new();
    let expected = [
        [vec![1, 2, 0], vec![2, 1, 0]],
        [vec![2, 0, 1], vec![0, 2, 1]],
        [vec![1, 0, 2], vec![0, 1, 2]],
    ];
    for first in 0..3 { for second in 0..2 {
        let mut words = [first,second].into_iter();
        let mut count = 0;
        let result = shuffle_from_words(&[0,1,2],&mut || {count+=1; Ok(words.next().unwrap())}).unwrap();
        assert_eq!(count,2);
        assert_eq!(result, expected[first as usize][second as usize]);
        assert_eq!(result.iter().copied().collect::<BTreeSet<_>>(),BTreeSet::from([0,1,2]));
        assert!(permutations.insert(result));
    } }
    assert_eq!(permutations.len(),6);
    let input = vec!["a","a","b"];
    let before = input.clone();
    let s = state();
    let result = shuffle(&s,&input).unwrap();
    assert_eq!(input,before);
    let mut sorted = result.0.clone(); sorted.sort();
    assert_eq!(sorted,vec!["a","a","b"]);
    assert_eq!(result,shuffle(&s,&input).unwrap());
}
