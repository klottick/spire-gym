use super::super::*;
use super::super::rules::*;
use super::super::rng::{self, Domain, RngError, StreamId, StreamState};
use super::support::base_scenario;
use MoveId::*;

#[test]
fn six_card_rows() {
    for (kind, upgraded, row) in [
        (CardKind::Strike, false, (1, 6, 0, 0)),
        (CardKind::Strike, true, (1, 9, 0, 0)),
        (CardKind::Defend, false, (1, 0, 5, 0)),
        (CardKind::Defend, true, (1, 0, 8, 0)),
        (CardKind::Bash, false, (2, 8, 0, 2)),
        (CardKind::Bash, true, (2, 10, 0, 3)),
    ] {
        let s = card_stats(kind, upgraded);
        assert_eq!((s.cost, s.damage, s.block, s.vulnerable), row);
    }
}

#[test]
fn bounded_arithmetic_floors_caps_and_absorption() {
    for (base, strength, vulnerable, want) in [
        (9, 0, true, 13), (6, 999, false, 1005), (11, 999, true, 1515),
        (0, 0, true, 0), (1, 0, true, 1), (7, 2, true, 13),
    ] { assert_eq!(normal_damage(base, strength, vulnerable), Ok(want)); }
    assert_eq!(block_after_add(998, 8), Ok(999));
    assert_eq!(block_after_add(0, 0), Ok(0));
    assert_eq!(block_after_add(999, 8), Ok(999));
    for (hp, block, damage, want) in [
        (40, 5, 11, (34, 0)), (5, 0, 11, (0, 0)),
        (40, 20, 11, (40, 9)), (0, 999, 1515, (0, 0)),
        (999, 999, 0, (999, 999)),
    ] { assert_eq!(hp_and_block_after_damage(hp, block, damage), Ok(want)); }
    assert_eq!(clamped_strength(998, 3), Ok(999));
    assert_eq!(clamped_strength(999, 3), Ok(999));
    assert_eq!(clamped_strength(0, 0), Ok(0));
}

#[test]
fn arithmetic_rejects_outside_bounded_surface() {
    for bad in [i32::MIN, -1, 12, i32::MAX] {
        assert!(normal_damage(bad, 0, false).is_err());
    }
    for bad in [i32::MIN, -1, 1000, i32::MAX] {
        assert!(normal_damage(6, bad, true).is_err());
        assert!(block_after_add(bad, 5).is_err());
        assert!(hp_and_block_after_damage(bad, 0, 0).is_err());
        assert!(hp_and_block_after_damage(40, bad, 0).is_err());
        assert!(clamped_strength(bad, 3).is_err());
    }
    for bad in [-1, 9, i32::MAX] { assert!(block_after_add(0, bad).is_err()); }
    for bad in [-1, 1516, i32::MAX] {
        assert!(hp_and_block_after_damage(40, 0, bad).is_err());
    }
    for bad in [-1, 4, i32::MAX] { assert!(clamped_strength(0, bad).is_err()); }
}

#[test]
fn exact_history_laws() {
    // Independent acceptance rows; denominator is divisible by all fallback denominators.
    let common = 100_u128 * 8388608;
    let cases: &[(&[MoveId], [(u128, u128); 3])] = &[
        (&[], [(1, 1), (0, 1), (0, 1)]),
        (&[Chomp], [(0, 1), (131, 320), (189, 320)]),
        (&[Chomp, Bellow], [(73349989, 167772160), (94422171, 167772160), (0, 1)]),
        (&[Chomp, Thrash], [(1, 4), (3, 10), (9, 20)]),
        (&[Chomp, Thrash, Thrash], [(29955719, 83886080), (0, 1), (53930361, 83886080)]),
    ];
    for (history, expected) in cases {
        let mut weights = [0_u128; 3];
        for roll in 0..100 {
            let ratio = jaw_worm_fallback(history, roll).unwrap();
            let branches = match ratio {
                None => vec![(None, common / 100)],
                Some(p) => vec![
                    (Some(true), (common / 100 / p.denominator) * p.numerator),
                    (Some(false), (common / 100 / p.denominator) * (p.denominator - p.numerator)),
                ],
            };
            for (coin, weight) in branches {
                let index = match jaw_worm_move(history, roll, coin).unwrap() {
                    Chomp => 0, Thrash => 1, Bellow => 2, _ => panic!("wrong enemy move"),
                };
                weights[index] += weight;
            }
        }
        assert_eq!(weights.iter().sum::<u128>(), common);
        for i in 0..3 {
            assert_eq!(weights[i] * expected[i].1, expected[i].0 * common, "{history:?}, column {i}");
        }
    }
}

struct Script {
    bucket: Result<u8, RngError>,
    coin: Result<bool, RngError>,
    buckets: usize,
    coins: Vec<Ratio>,
}
impl Script {
    fn new(roll: u8, coin: bool) -> Self {
        Self { bucket: Ok(roll), coin: Ok(coin), buckets: 0, coins: vec![] }
    }
}
impl SelectionDraws for Script {
    fn bucket(&mut self) -> Result<u8, RngError> {
        self.buckets += 1;
        assert_eq!(self.buckets, 1);
        self.bucket.clone()
    }
    fn coin(&mut self, probability: Ratio) -> Result<bool, RngError> {
        self.coins.push(probability);
        assert_eq!(self.coins.len(), 1);
        self.coin.clone()
    }
}

#[test]
fn only_restricted_buckets_request_fallback() {
    let histories: &[&[MoveId]] = &[&[Chomp], &[Chomp, Bellow], &[Chomp, Thrash], &[Chomp, Thrash, Thrash]];
    for history in histories {
        for roll in 0..100 {
            // These predicates and literal ratios are independent of the selection helpers.
            let restricted = match (*history, roll) {
                ([Chomp], 0..=24) => Some((9, 16, Bellow, Thrash)),
                ([Chomp, Bellow], 55..=99) => Some((3489661, 8388608, Chomp, Thrash)),
                ([Chomp, Thrash, Thrash], 25..=54) => Some((2994733, 8388608, Chomp, Bellow)),
                _ => None,
            };
            for coin in [false, true] {
                let mut draws = Script::new(roll, coin);
                let got = select(EnemyKind::JawWorm, history, &mut draws).unwrap();
                let ordinary = if roll < 25 { Chomp } else if roll < 55 { Thrash } else { Bellow };
                let (expected, ratios) = match restricted {
                    Some((n, d, yes, no)) => (if coin { yes } else { no }, vec![Ratio { numerator: n, denominator: d }]),
                    None => (ordinary, vec![]),
                };
                assert_eq!(got, expected, "{history:?}, bucket {roll}, coin {coin}");
                assert_eq!(draws.buckets, 1);
                assert_eq!(draws.coins, ratios);
            }
        }
    }
    for (kind, history, want) in [
        (EnemyKind::JawWorm, vec![], Chomp), (EnemyKind::Cultist, vec![], Incantation),
        (EnemyKind::Cultist, vec![Incantation], Attack),
        (EnemyKind::Cultist, vec![Incantation, Attack, Attack], Attack),
    ] {
        let mut draws = Script::new(100, false);
        assert_eq!(select(kind, &history, &mut draws), Ok(want));
        assert_eq!((draws.buckets, draws.coins.len()), (0, 0));
    }
}

#[test]
fn fixed_project_selection() {
    let stream = base_scenario().rng.enemy_ai;
    assert_eq!(rng::uniform_below(&stream, 100).unwrap().0, 29);
    let history = vec![Chomp];
    let (chosen, next) = choose_enemy_move(EnemyKind::JawWorm, &history, &stream).unwrap();
    assert_eq!(chosen, Thrash);
    assert_eq!(next.word_index(), 1);
    assert_eq!(stream.word_index(), 0);
    assert_eq!(history, [Chomp]);
    for (kind, history, want) in [
        (EnemyKind::JawWorm, vec![], Chomp),
        (EnemyKind::Cultist, vec![Incantation, Attack], Attack),
    ] {
        assert_eq!(choose_enemy_move(kind, &history, &stream), Ok((want, stream.clone())));
    }
}

#[test]
fn native_conditional_coins_use_exact_project_sampling() {
    // Independently calculated public HMAC words, zero key:
    // cursor 0: bucket 22, ticket 7/16 => Bellow;
    // cursor 1: bucket 79, ticket 538631/8388608 => Chomp;
    // cursor 3: bucket 37, ticket 7772798/8388608 => Bellow.
    let id = StreamId::new(Domain::CombatEnemyAi, vec!["combat".into(), "enemy".into()]).unwrap();
    for (cursor, history, want, next_cursor) in [
        (0, vec![Chomp], Bellow, 2),
        (1, vec![Chomp, Bellow], Chomp, 3),
        (3, vec![Chomp, Thrash, Thrash], Bellow, 5),
    ] {
        let stream = StreamState::new(id.clone(), [0; 32], cursor).unwrap();
        let (got, next) = choose_enemy_move(EnemyKind::JawWorm, &history, &stream).unwrap();
        assert_eq!(got, want);
        assert_eq!(next, StreamState::new(id.clone(), [0; 32], next_cursor).unwrap());
        assert_eq!(stream.word_index(), cursor);
    }
}

#[test]
fn invalid_selection_does_not_sample() {
    for (kind, history) in [
        (EnemyKind::JawWorm, vec![Attack]),
        (EnemyKind::JawWorm, vec![Chomp, Incantation]),
        (EnemyKind::Cultist, vec![Chomp]),
        (EnemyKind::Cultist, vec![Attack]),
        (EnemyKind::Cultist, vec![Incantation, Incantation]),
    ] {
        let mut draws = Script::new(29, true);
        assert!(matches!(select(kind, &history, &mut draws), Err(SelectionError::Invalid(_))));
        assert_eq!((draws.buckets, draws.coins.len()), (0, 0));
    }
    let stream = base_scenario().rng.shuffle;
    assert!(matches!(choose_enemy_move(EnemyKind::JawWorm, &[], &stream), Err(SelectionError::Invalid(_))));
    for history in [&[][..], &[Chomp][..]] {
        assert!(jaw_worm_fallback(history, 100).is_err());
        assert!(jaw_worm_move(history, 100, None).is_err());
    }
    assert!(jaw_worm_fallback(&[Attack], 29).is_err());
    assert!(jaw_worm_move(&[Chomp], 0, None).is_err());
    assert!(jaw_worm_move(&[Chomp], 29, Some(true)).is_err());
    let mut draws = Script::new(100, true);
    assert!(matches!(select(EnemyKind::JawWorm, &[Chomp], &mut draws), Err(SelectionError::Invalid(_))));
    assert_eq!(draws.coins.len(), 0);
}

#[test]
fn fallback_exhaustion_is_not_selection() {
    let stream = base_scenario().rng.enemy_ai;
    let exhausted = StreamState::new(stream.id().clone(), *stream.key(), 1_u128 << 64).unwrap();
    let error = RngError::Exhausted { state: exhausted.clone() };
    let mut draws = Script::new(0, true);
    draws.coin = Err(error.clone());
    assert_eq!(select(EnemyKind::JawWorm, &[Chomp], &mut draws), Err(SelectionError::Random(error.clone())));
    assert_eq!((draws.buckets, draws.coins.len()), (1, 1));
    draws = Script::new(0, true);
    draws.bucket = Err(error.clone());
    assert_eq!(select(EnemyKind::JawWorm, &[Chomp], &mut draws), Err(SelectionError::Random(error.clone())));
    assert_eq!((draws.buckets, draws.coins.len()), (1, 0));
    // Pin a real final-word bucket whose conditional coin cannot be drawn.
    let id = StreamId::new(Domain::CombatEnemyAi, vec!["combat".into(), "enemy".into()]).unwrap();
    let final_word = StreamState::new(id, [0; 32], (1_u128 << 64) - 1).unwrap();
    let (roll, end) = rng::uniform_below(&final_word, 100).unwrap();
    let history = if roll < 25 { vec![Chomp] } else if roll < 55 {
        vec![Chomp, Thrash, Thrash]
    } else { vec![Chomp, Bellow] };
    assert_eq!(end.word_index(), 1_u128 << 64);
    assert_eq!(choose_enemy_move(EnemyKind::JawWorm, &history, &final_word),
        Err(SelectionError::Random(RngError::Exhausted { state: end })));
    assert_eq!(choose_enemy_move(EnemyKind::JawWorm, &[Chomp], &exhausted),
        Err(SelectionError::Random(error)));
}
