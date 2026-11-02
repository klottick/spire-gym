//! Default-off fixed pair starts and full-state access for M27 frozen replay.
//! Pair inputs use a named finite-union profile and ordinary authored combat entry.
use crate::{
    combat::{self, Combat, CombatEntryFailure, PlayerState},
    combat::rng::Partition,
    content::{CardId, Character, EncounterId, RelicId},
    items::{CombatItemContext, CombatStartupProfile},
    run::{CardAdmission, CardInstance, CombatProgress, CombatRecord, CompleteCombatStart, GameAction, ItemAdmission, MapNode, MapNodeKind, Run, RunAdmission, RunError, RunHistory, RunMap, RunRng, VisitedRoom},
};

/// Complete relic, potion and counter resources for the constructed Act 4 diagnostic.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompleteItemSupply {
    pub relics: Vec<RelicId>,
    pub potions: Vec<Option<crate::content::PotionId>>,
    pub counters: std::collections::BTreeMap<RelicId, i32>,
}

/// Construct the selected M26 Heart scenario at the Act 4 map with complete item admission.
/// The input inventory is normalized and validated before the template Run is created. The
/// prior Act 3 victory and keys are constructed fixture provenance; the Act 4 door itself is
/// committed by the shared Run owner.
pub fn complete_act4_run(character: Character, supply: CompleteItemSupply) -> Result<Run, RunError> {
    let admission = RunAdmission::new(CardAdmission::Complete, ItemAdmission::Complete)?;
    let starter = crate::content::Catalog::standard().starter_deck(character)
        .map_err(|_| RunError::Registry("starter deck missing for complete Act 4 diagnostic"))?;
    let deck = starter.iter().enumerate().map(|(index, id)| CardInstance {
        id: *id, instance: index as u64 + 1, upgrades: 0, misc: 0,
    }).collect::<Vec<_>>();
    let supplied = crate::run::AuthoredRunStart {
        hp: 80, max_hp: 80, gold: 999, deck: deck.clone(), relics: supply.relics,
        potions: supply.potions, relic_counters: supply.counters,
    };
    let CompleteCombatStart::Supplied(resources) = CompleteCombatStart::Supplied(supplied)
        .normalized_for(character, admission.items)?;

    // M26 owns the character-to-Heart-scenario mapping and the complete-card template path.
    let template = crate::run::AuthoredRunStart {
        hp: 80, max_hp: 80, gold: 999, deck, relics: vec![], potions: vec![None; 3],
        relic_counters: Default::default(),
    };
    let mut run = super::complete_run(character, template)?;
    run.state.hp = resources.hp;
    run.state.max_hp = resources.max_hp;
    run.state.gold = resources.gold;
    run.state.deck = resources.deck;
    run.state.relics = resources.relics;
    run.state.potions = resources.potions;
    run.state.relic_counters = resources.relic_counters;
    run.state.next_instance = run.state.deck.iter().map(|card| card.instance).max().unwrap() + 1;
    run.state.card_admission = admission.cards;
    run.state.item_admission = admission.items;
    run.state.relic_pools = Some(crate::items::relic_pools::build(&run.state, &[])?);

    let node = crate::run::NodeId(0xF270_0050);
    let record = CombatRecord { visit: 0, encounter: EncounterId::AwakenedOne, outcome: crate::combat::Outcome::Victory };
    run.state.act = 3;
    run.state.floor = 50;
    run.state.room = "A23SuppliedAct3Source";
    run.state.act4_enabled = true;
    run.state.keys = [true; 3];
    run.state.combat_progress = CombatProgress { entered: 1, settled: 1 };
    run.state.last_combat = Some(record.clone());
    run.state.history = RunHistory {
        rooms: vec![VisitedRoom { act: 3, floor: 50, label: "A23SuppliedAct3Source" }],
        events: vec![crate::run::CommittedEvent::CombatFinished(record.clone())],
    };
    run.state.reward_ledger = Default::default();
    run.state.map = Some(RunMap {
        act: 3, boss_icon: Some(EncounterId::AwakenedOne), current: Some(node),
        nodes: vec![MapNode {
            id: node, floor: 50, label: "Boss", edges: vec![], x: 0, y: 15,
            kind: MapNodeKind::Boss, hidden_kind: None, visited: true, revealed: true,
            encounter: Some(EncounterId::AwakenedOne), burning: false,
        }],
        ..RunMap::default()
    });
    let mut next = run.next_choice;
    let screen = crate::run::rooms::view(&mut next, crate::run::RoomViewKind::Act3Ending,
        vec![(crate::run::RoomChoiceKind::Act3Continue { stage: 4 }, None)])?;
    let choice = screen.choices[0].id;
    let room = run.state.room_loop.as_mut().ok_or(RunError::InvalidAction)?;
    room.act1 = None; room.act2 = None; room.act3 = None; room.act4 = None;
    room.act3_end = Some(crate::run::rooms::act3_end::Act3EndFrame {
        node, record, phase: crate::run::rooms::act3_end::Act3EndPhase::Dialog(4), accepted: None,
    });
    room.screen = screen;
    run.next_choice = next;
    run.candidates.clear();
    run.step(&GameAction::Choose(choice))?;
    Ok(run)
}

#[path = "m14_supplied_boss.rs"]
mod m14_supplied_source;

pub fn m27_m14_supplied_run(
    run: Run,
    act3_boss: crate::content::EncounterId,
) -> Result<Run, String> {
    m14_supplied_source::m27_replay_fixture(run, act3_boss)
}

pub fn m27_m15_supplied_run(run: Run) -> Result<Run, String> {
    super::m15_supplied_act4::prepare(super::m15_supplied_act4::Loadout::Silent, run)
}

/// Per-step private-state capture used only by the M27 frozen replay example.
pub struct M27RunCapture {
    pub run_state_debug: String,
    pub relics: Vec<RelicId>,
    pub combat_state_debug: Option<String>,
    pub public_full_debug: Option<String>,
    pub enemy1_power_order: Vec<String>,
}

pub fn m27_capture_run(run: &Run) -> Result<M27RunCapture, RunError> {
    let (combat_state_debug, public_full_debug, enemy1_power_order) =
        if let Some(combat) = run.state.combat.as_ref() {
            let public = combat.public_full()?;
            let powers = public.enemies.first()
                .map(|enemy| enemy.powers.iter().map(|power| power.id.0.to_owned()).collect())
                .unwrap_or_default();
            (Some(combat.m27_state_debug()), Some(format!("{public:?}")), powers)
        } else {
            (None, None, Vec::new())
        };
    Ok(M27RunCapture {
        run_state_debug: format!("{:?}", run.state),
        relics: run.state.relics.clone(),
        combat_state_debug,
        public_full_debug,
        enemy1_power_order,
    })
}

/// Build a fixed authored combat used by one co-firing pair replay input.
/// `profile` is the finite union label reduced to one of legacy/m15/m20/m21/m22/m24/m25.
pub fn m27_pair_combat(
    character: Character,
    relics: Vec<RelicId>,
    profile: &str,
    hp: i32,
    max_hp: i32,
    seed_byte: u8,
    deck_copies: usize,
    opening_draw: usize,
    encounter: Option<EncounterId>,
    deck_card: Option<CardId>,
    enemy_hp: Option<i32>,
) -> Result<Combat, RunError> {
    let card = deck_card.unwrap_or(match character {
        Character::Ironclad => CardId::Strike,
        Character::Silent => CardId::StrikeGreen,
        Character::Defect => CardId::StrikeBlue,
        Character::Watcher => CardId::StrikePurple,
    });
    let cards = (0..deck_copies)
        .map(|index| combat::permanent_live_card(card, combat::CardInstanceId(index as u64 + 1), 0))
        .collect::<Vec<_>>();
    let mut rng = RunRng::new([seed_byte; 32], Partition::Test, 0);
    let mut entry = combat::enemies::setup(
        encounter.unwrap_or(EncounterId::Cultist),
        1,
        0,
        &mut rng,
        combat::enemies::SetupContext {
            player: PlayerState {
                hp,
                max_hp,
                block: 0,
                energy: 3,
                max_energy: 3,
                draw_per_turn: 5,
                powers: vec![],
            },
            player_powers: vec![],
            character,
            cards,
            opening_draw,
            encounter_visit: 0,
        },
    )?;
    if let Some(hp) = enemy_hp {
        for enemy in &mut entry.setup.enemies {
            enemy.hp = hp;
            enemy.max_hp = hp;
        }
    }
    let items = CombatItemContext::new(relics, vec![None; 3]);
    let items = match profile {
        "legacy" => items,
        "m15" => items.with_m15_profile(),
        "m20" => items.with_m20_profile(),
        "m21" => items.with_startup_profile(CombatStartupProfile::M21Watcher),
        "m22" => items.with_m22_profile(),
        "m24" => items.with_m24_profile(),
        "m25" => items.with_m25_profile(),
        _ => return Err(RunError::Unsupported("unknown M27 replay union profile")),
    };
    let mut owner = Some(items);
    match Combat::enter_with_item_context(entry, &mut owner) {
        Ok(combat) => Ok(combat),
        Err(CombatEntryFailure::Rejected(error)) => Err(error),
        Err(CombatEntryFailure::Faulted { fault, .. }) => Err(RunError::Faulted(fault)),
    }
}
