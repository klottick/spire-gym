use crate::content::{CardId, PotionId, RelicId};
use crate::run::*;

#[derive(Clone, Copy, Debug)]
pub(crate) enum Loadout { Silent, Searing }

pub(crate) const RELICS: [RelicId; 9] = [
    RelicId::RingOfTheSnake, RelicId::Mango, RelicId::SneckoSkull,
    RelicId::PaperKrane, RelicId::ToughBandages, RelicId::BagOfPreparation,
    RelicId::Anchor, RelicId::OddlySmoothStone, RelicId::SlaversCollar,
];
pub(crate) const SILENT_DECK: [CardId; 18] = [
    CardId::Backflip, CardId::Backflip, CardId::Backflip,
    CardId::Deflect, CardId::Deflect, CardId::Deflect,
    CardId::Footwork, CardId::Footwork, CardId::NoxiousFumes,
    CardId::NoxiousFumes, CardId::DeadlyPoison, CardId::DeadlyPoison,
    CardId::DeadlyPoison, CardId::Catalyst, CardId::Catalyst,
    CardId::Catalyst, CardId::Neutralize, CardId::Survivor,
];

// Shared source setup for the retained M15 fixture and the default-off replay bridge.
fn supplied_deck(loadout: Loadout) -> Result<Vec<CardInstance>, String> {
    let ids = match loadout {
        Loadout::Silent => SILENT_DECK.to_vec(),
        Loadout::Searing => vec![CardId::SearingBlow; 10],
    };
    ids.into_iter().enumerate().map(|(index, id)| {
        let instance = index as u64 + 1;
        let mut live = crate::combat::permanent_live_card(id, crate::combat::CardInstanceId(instance), 0);
        let count = match loadout { Loadout::Silent => 1, Loadout::Searing => 15 };
        for _ in 0..count {
            crate::combat::cards::ironclad::rules::upgrade(&mut live)
                .map_err(|error| format!("supplied actual upgrade: {error:?}"))?;
        }
        Ok(CardInstance { misc: 0, instance, id, upgrades: live.upgrades })
    }).collect()
}

pub(crate) fn prepare(loadout: Loadout, mut run: Run) -> Result<Run, String> {
    use crate::{combat::Outcome, content::EncounterId};
    use crate::run::rooms::{act3_end::{Act3EndFrame, Act3EndPhase}, view};
    // All privileged resource/provenance assignments are initialization, before
    // the supplied floor50 boundary. No state patch occurs after it.
    run.state.deck = supplied_deck(loadout)?;
    run.state.next_instance = 100;
    for id in RELICS.into_iter().skip(1) {
        crate::items::acquire_relic(&mut run.state, id)
            .map_err(|error| format!("supplied relic {id:?}: {error:?}"))?;
    }
    if matches!(loadout, Loadout::Searing) { run.state.hp = 250; run.state.max_hp = 250; }
    run.state.gold = 200;
    run.state.potions = vec![Some(PotionId::WeakPotion), Some(PotionId::FirePotion), Some(PotionId::StrengthPotion)];
    run.state.keys = [true; 3];
    run.state.act4_enabled = true;
    run.state.act = 3;
    run.state.floor = 50;
    run.state.room = "SuppliedAct3Source";
    // Synthetic prior Victory and supplied keys are declared fixture input,
    // never a played/earned prior win. Only actual Act4 fights follow this source.
    let node = NodeId(0xF150_0050);
    let source = CombatRecord { visit: 0, encounter: EncounterId::AwakenedOne, outcome: Outcome::Victory };
    run.state.combat_progress = CombatProgress { entered: 1, settled: 1 };
    run.state.last_combat = Some(source.clone());
    run.state.history = RunHistory {
        rooms: vec![VisitedRoom { act: 3, floor: 50, label: "SuppliedAct3Source" }],
        events: vec![CommittedEvent::CombatFinished(source.clone())],
    };
    run.state.reward_ledger = Default::default();
    run.state.map = Some(RunMap {
        act: 3,
        boss_icon: Some(EncounterId::AwakenedOne),
        current: Some(node),
        nodes: vec![MapNode {
            id: node, floor: 50, label: "Boss", edges: vec![], x: 0, y: 15,
            kind: MapNodeKind::Boss, hidden_kind: None, visited: true,
            revealed: true, encounter: Some(EncounterId::AwakenedOne), burning: false,
        }],
        ..RunMap::default()
    });
    let mut next = run.next_choice;
    let screen = view(&mut next, RoomViewKind::Act3Ending, vec![(RoomChoiceKind::Act3Continue { stage: 4 }, None)])
        .map_err(|error| format!("supplied door: {error:?}"))?;
    let choice = screen.choices[0].id;
    let room = run.state.room_loop.as_mut().ok_or("supplied room owner absent")?;
    room.act1 = None;
    room.act2 = None;
    room.act3 = None;
    room.act3_end = Some(Act3EndFrame { node, record: source, phase: Act3EndPhase::Dialog(4), accepted: None });
    room.screen = screen;
    room.m10_elite_relic_cursor = 0;
    run.next_choice = next;
    run.candidates.clear();
    run.step(&GameAction::Choose(choice)).map_err(|error| format!("shared supplied Act4 transition: {error:?}"))?;
    Ok(run)
}
