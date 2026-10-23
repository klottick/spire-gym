use crate::{combat::rng::Partition, run::{RunRng, RunRngScope}};

#[test]
fn semantic_scope_draws_are_independent_and_reproducible() {
    let mut left=RunRng::new([9;32],Partition::Test,2);
    let first=left.draw(RunRngScope::NeowCategory0);
    let second=left.draw(RunRngScope::NeowCategory0);
    let mut interleaved=RunRng::new([9;32],Partition::Test,2);
    let first_other=interleaved.draw(RunRngScope::NeowCategory0);
    let _unrelated=interleaved.draw(RunRngScope::NeowCategory1);
    let second_other=interleaved.draw(RunRngScope::NeowCategory0);
    assert_eq!((first,second),(first_other,second_other));
    let mut replay=RunRng::new([9;32],Partition::Test,2);
    assert_eq!(first,replay.draw(RunRngScope::NeowCategory0));
    assert_eq!(second,replay.draw(RunRngScope::NeowCategory0));
    assert!(RunRngScope::parse("run.neow.unregistered").is_err());
}

#[test]
fn map_rng_semantic_payloads_are_independent_of_interleaving_act_and_draft() {
    let mut baseline=RunRng::new([19;32],Partition::Test,4);
    let first=baseline.draw_scoped(RunRngScope::MapTopology,&[1,0,0]);
    let mut interleaved=RunRng::new([19;32],Partition::Test,4);
    let first_again=interleaved.draw_scoped(RunRngScope::MapTopology,&[1,0,0]);
    let _other_site=interleaved.draw_scoped(RunRngScope::MapRoomBucket,&[1,0]);
    let _other_act=interleaved.draw_scoped(RunRngScope::MapTopology,&[2,0,0]);
    let _other_draft=interleaved.draw_scoped(RunRngScope::MapTopology,&[1,1,0]);
    assert_eq!(first,first_again);
    let repeated=interleaved.draw_scoped(RunRngScope::MapTopology,&[1,0,0]);
    let mut replay=RunRng::new([19;32],Partition::Test,4);
    let replay_first=replay.draw_scoped(RunRngScope::MapTopology,&[1,0,0]);
    let replay_second=replay.draw_scoped(RunRngScope::MapTopology,&[1,0,0]);
    assert_eq!((first,repeated),(replay_first,replay_second));
}
