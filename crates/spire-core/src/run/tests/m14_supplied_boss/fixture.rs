use super::*;
use crate::{
    items::{self, CombatItemContext},
    run::{map, GameAction, MapNodeKind, RoomView, RoomViewKind},
};

pub(super) struct Prepared {
    pub(super) run: Run,
    pub(super) entry: Option<RunTransition>,
}

pub(super) fn prepare(seed: u64, scenario: frozen::Scenario) -> Result<Prepared, String> {
    let mut run = frozen::make_run(seed, scenario)?;

    let starter_instances = run.state.deck.iter().map(|card| card.instance).collect::<Vec<_>>();
    for id in DECK {
        if !crate::run::M14_SILENT_ACT3_CARDS.contains(&id) {
            return Err(format!("supplied card is outside the M14 finite pool: {id:?}"));
        }
        crate::run::mutations::obtain_card(&mut run.state, id, 1)
            .map_err(|error| format!("obtain upgraded supplied card {id:?}: {error:?}"))?;
    }
    for instance in starter_instances {
        crate::run::mutations::remove_card(&mut run.state, instance)
            .map_err(|error| format!("remove canonical starter card: {error:?}"))?;
    }

    for id in ACQUIRE_RELICS {
        if !crate::run::M14_SILENT_ACT3_RELICS.contains(&id) {
            return Err(format!("supplied relic is outside the M14 finite pool: {id:?}"));
        }
        items::acquire_relic(&mut run.state, id)
            .map_err(|error| format!("acquire supplied relic {id:?}: {error:?}"))?;
    }
    if POTIONS.len() != 3 || POTIONS.iter().flatten().any(|id| !crate::run::M14_SILENT_ACT3_POTIONS.contains(id)) {
        return Err("supplied potion belt is outside the M14 finite inventory".into());
    }
    run.state.potions = POTIONS.to_vec();
    let inventory = CombatItemContext::new(run.state.relics.clone(), run.state.potions.clone());
    items::validate_combat_inventory(&inventory)
        .map_err(|error| format!("validate supplied combat inventory: {error:?}"))?;

    let eligibility = map::MapEligibility::from_run(&run.state);
    let mut act3_map = map::generate(3, &mut run.state.rng, eligibility)
        .map_err(|error| format!("generate supplied Act3 map: {error:?}"))?;
    for node in &mut act3_map.nodes {
        node.floor += 34;
    }
    act3_map.boss_icon = Some(scenario.act3_boss);
    let boss_node = act3_map.nodes.iter().find(|node| {
        node.floor == 50 && node.kind == MapNodeKind::Boss
    }).map(|node| node.id).ok_or("generated Act3 map has no floor-50 boss node")?;
    let predecessor = act3_map.nodes.iter_mut().find(|node| {
        node.floor == 49 && node.edges.contains(&boss_node)
    }).ok_or("generated Act3 map has no floor-49 boss predecessor")?;
    let predecessor_id = predecessor.id;
    predecessor.visited = true;
    predecessor.revealed = true;
    act3_map.current = Some(predecessor_id);

    run.state.act = 3;
    run.state.floor = 49;
    run.state.room = "Map";
    run.state.map = Some(act3_map);
    let room = run.state.room_loop.as_mut().ok_or("M14 run lost its room frame")?;
    room.act1 = None;
    room.act2 = None;
    room.act3 = Some(Default::default());
    room.act3_boss_order = Some(vec![scenario.act3_boss]);
    room.screen = RoomView { kind: RoomViewKind::Map, choices: vec![] };

    let entry = run.step(&GameAction::Move(boss_node))
        .map_err(|error| format!("enter supplied Act3 boss through Run::step: {error:?}"))?;
    Ok(Prepared { run, entry: Some(entry) })
}
