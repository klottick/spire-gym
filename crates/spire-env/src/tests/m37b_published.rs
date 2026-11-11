//! M37b (spire-4yo.2.3 and .2.6): a session keeps no copy of the observation it returns; `decision()` re-encodes it from
//! the unchanged Run, and record headers reuse the memoised spec fingerprint. Random Standard play, four characters at
//! A0 and A20, recording on: after every reset, step, checkpoint, restore and fork, `decision()` must equal the boundary
//! the call returned (observation bytes, token and terminal), and every record header must carry the spec fingerprint.
use crate::{RunActionIndex, RunSession, StandardRunConfig};
use spire_core::{combat::rng::Partition, content::Character};

const CHARACTERS: [Character; 4] = [Character::Ironclad, Character::Silent, Character::Defect, Character::Watcher];

#[test]
fn m37b_decision_reencodes_the_returned_boundary_and_records_keep_the_fingerprint() {
    let mut checked = 0usize;
    for character in CHARACTERS {
        for ascension in [0, 20] {
            let mut config = StandardRunConfig::new(character, ascension, Partition::Test);
            config.record_replay = true;
            let mut session = RunSession::new_standard(config).unwrap();
            let fingerprint = session.observation_spec().unwrap().fingerprint();
            for episode in 0..3u64 {
                let seed = 0x37B0 + episode;
                let mut boundary = session.reset(Some(seed)).unwrap();
                assert_eq!(session.decision().unwrap(), boundary, "reset {character:?} A{ascension} episode {episode}");
                for step in 0..400u64 {
                    if boundary.terminal.is_some() { break }
                    if step == 25 {
                        let checkpoint = session.exact_checkpoint().unwrap();
                        let fork = RunSession::exact_fork(&checkpoint).unwrap();
                        let forked = fork.decision().unwrap();
                        assert_eq!((&forked.observation, forked.terminal), (&boundary.observation, boundary.terminal), "fork at {step}");
                        let restored = session.exact_restore(&checkpoint).unwrap();
                        assert_eq!((&restored.observation, restored.terminal), (&boundary.observation, boundary.terminal), "restore at {step}");
                        assert_eq!(session.decision().unwrap(), restored, "decision after restore at {step}");
                        boundary = restored;
                    }
                    let action = crate::tests::m35_r2::r1(seed, step, &boundary.observation.action_mask);
                    boundary = session.step(boundary.token, RunActionIndex(action as i64)).unwrap().boundary;
                    assert_eq!(session.decision().unwrap(), boundary, "step {step} {character:?} A{ascension} episode {episode}");
                    checked += 1;
                }
                let record = session.record().unwrap().unwrap();
                assert_eq!(record.observation_fingerprint, fingerprint, "record header fingerprint");
                assert_eq!(session.record().unwrap().unwrap(), record, "record export is repeatable");
            }
        }
    }
    assert!(checked > 1000, "{checked} steps checked");
}
