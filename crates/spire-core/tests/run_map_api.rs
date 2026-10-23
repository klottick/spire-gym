use spire_core::{
    combat::rng::Partition,
    run::{map, RunRng},
};

#[test]
fn external_map_api_generates_sanitized_eligible_and_ineligible_maps() {
    let mut eligible_rng = RunRng::new([41; 32], Partition::Test, 0);
    let eligible = map::generate(
        1,
        &mut eligible_rng,
        map::MapEligibility::new(true, false),
    )
    .unwrap();
    let visible = eligible.public_view();
    assert_eq!(visible.act, 1);
    assert!(visible.nodes.iter().any(|node| node.kind == spire_core::run::MapNodeKind::Elite));
    assert!(visible.nodes.iter().any(|node| node.burning == Some(true)));
    assert!(visible.nodes.iter().all(|node| {
        node.edges.iter().all(|edge| visible.nodes.iter().any(|child| child.id == *edge))
    }));

    for eligibility in [
        map::MapEligibility::new(false, false),
        map::MapEligibility::new(true, true),
    ] {
        let mut rng = RunRng::new([41; 32], Partition::Test, 0);
        let generated = map::generate(1, &mut rng, eligibility).unwrap();
        let public = generated.public_view();
        assert!(public.nodes.iter().all(|node| node.burning != Some(true)));
    }
}

#[test]
fn external_map_debug_is_the_sanitized_public_projection() {
    let mut rng = RunRng::new([42; 32], Partition::Test, 0);
    let generated = map::generate(1, &mut rng, map::MapEligibility::new(true, false)).unwrap();

    assert_eq!(format!("{generated:?}"), format!("{:?}", generated.public_view()));
}
