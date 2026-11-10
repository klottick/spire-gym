//! M34s-a a6 measurement (ignored; release): Standard encode time per published boundary with and without the draw rows
//! (the same decision with `draw_unknown` cleared) on the supplied Heart routes, 4 characters × {A0, A20}. Not a gate.
use super::standard_traversal::{needs_supplied_wound, short_creation, simple_action, standard_session, CHARACTERS};
use crate::*;
use spire_core::run::diagnostics::supplied_wound_enemies;
use std::time::Instant;

#[test]
#[ignore = "measurement; run with --release --ignored --nocapture"]
fn m34s_encode_cost_with_and_without_draw_rows() {
    let caps = RunCapacities::standard_default();
    for (ci, character) in CHARACTERS.into_iter().enumerate() {
        for ascension in [0u8, 20] {
            let (mut session, _) = standard_session(character, ascension, 0xB0 + ci as u8 + if ascension == 20 { 8 } else { 0 }, short_creation());
            let mut boundary = session.decision().unwrap();
            let (mut with_rows, mut without, mut draw_rows) = (vec![], vec![], vec![]);
            while boundary.terminal.is_none() && with_rows.len() < 12_000 {
                let r = session.reproduction().unwrap();
                let d = r.latest_decision.unwrap().unwrap();
                if d.combat.is_some() {
                    let context = || crate::run_observation::StandardContext { ascension, run_selection: None };
                    let mut stripped = d.clone();
                    let n = stripped.combat.as_ref().map_or(0, |c| c.draw_unknown.len());
                    // Without draw rows: the same view with the unknown draw cards removed (draw_count kept consistent).
                    let c = stripped.combat.as_mut().unwrap(); c.draw_unknown.clear(); c.draw_count -= n;
                    // Alternate which encoding runs first, so neither carries the cold-cache cost.
                    for first_with in [with_rows.len() % 2 == 0, with_rows.len() % 2 != 0] {
                        let (decision, sink) = if first_with { (&d, &mut with_rows) } else { (&stripped, &mut without) };
                        let start = Instant::now();
                        let encoded = crate::run_observation::encode_run_standard(decision, r.latest_map.as_ref(), &caps, context());
                        sink.push(start.elapsed().as_nanos() as u64);
                        assert!(encoded.is_ok());
                    }
                    draw_rows.push(n);
                }
                if needs_supplied_wound(&boundary.observation) {
                    match session.apply_supplied(supplied_wound_enemies) { Ok(next) => boundary = next, Err(_) => break }
                }
                let index = simple_action(&boundary.observation);
                boundary = session.step(boundary.token, RunActionIndex(index as i64)).unwrap().boundary;
            }
            let median = |v: &mut Vec<u64>| { v.sort_unstable(); v.get(v.len() / 2).copied().unwrap_or(0) };
            let p99 = |v: &[u64]| v.get(v.len().saturating_sub(1) * 99 / 100).copied().unwrap_or(0);
            let (mw, mo) = (median(&mut with_rows), median(&mut without));
            println!("m34s_measure {character:?} A{ascension}: combat boundaries {}, encode ns median with draw rows {mw} / without {mo} (p99 {} / {}), draw rows mean {:.1} max {}",
                with_rows.len(), p99(&with_rows), p99(&without), draw_rows.iter().sum::<usize>() as f64 / draw_rows.len().max(1) as f64, draw_rows.iter().max().unwrap_or(&0));
        }
    }
}
