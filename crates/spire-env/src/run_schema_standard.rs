//! M34b Standard profile descriptor: the checked-in append-only vocabulary manifest, the shared power and
//! intent order, engine-bound capacities and domains, and the event params semantics. Layout extends M25.
use crate::{run_schema as v1, run_schema_m10 as m10, run_schema_m25 as m25, RunCapacities, RunObservationSpec, RunSessionError};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use spire_core::{
    combat::{public_ids::{EVENT_INTENTS, PUBLIC_INTENTS}, intents::IntentCategory, SelectionDisposition, SelectionOperation, SelectionView},
    content::{CardId, CardType, EncounterId, EnemyId, EventId, PotionId, RelicId},
    run::{EventSelectOp, NeowDrawback, NeowReward, NeowSelectOp, RoomChoiceKind, RoomScenario, RoomSelectionOperation, RoomViewKind, PARAM_SEMANTICS},
};
use std::sync::OnceLock;

pub const RUN_OBSERVATION_STANDARD_SCHEMA_ID: &str = "spire-gym-standard-run-observation-v1";
pub const STANDARD_VOCAB_MANIFEST_ID: &str = "standard-vocab-v1";
pub const STANDARD_CAPACITY_PRESET_ID: &str = "standard-default-v1";
const DOMAIN: &[u8] = b"spire-gym/run-schema/standard/v1\0";
const MANIFEST_JSON: &str = include_str!("standard_vocab_v1.json");
const EVENT_CONSTANTS_JSON: &str = include_str!("standard_event_constants_v1.json");

impl RunCapacities {
    /// `standard-default-v1`: every row capacity covers its engine bound (M34a lane B, M34b Task 0).
    /// Enemy rows hold present actors only (retired actors are not published): active ≤ 9 is engine-enforced.
    pub fn standard_default() -> Self {
        Self { map_nodes: 107, map_edges: 742, permanent_cards: 1024, combat_cards: 1024, enemies: 12, powers: 2048,
            relics: 256, potions: 16, offers: 512, selection: 1024, orbs: 10, actions: 2048 }
    }
}

/// The parsed `standard-vocab-v1` manifest (`manifest_id`, `open_families`, `families`).
pub fn standard_vocab_manifest() -> &'static Value {
    static M: OnceLock<Value> = OnceLock::new();
    M.get_or_init(|| serde_json::from_str(MANIFEST_JSON).expect("checked-in Standard vocabulary manifest"))
}

/// The Standard intent family: the shared order, then the event intents (review r1: later shared ids append after these).
pub(crate) fn intent_family() -> impl Iterator<Item = &'static str> { PUBLIC_INTENTS.iter().chain(EVENT_INTENTS).copied() }

fn vocabularies() -> Value {
    let mut vocab = standard_vocab_manifest()["families"].clone();
    vocab["power"] = crate::public_vocab::powers();
    vocab["intent"] = json!(std::iter::once("padding").chain(intent_family()).collect::<Vec<_>>());
    vocab
}
fn vocab() -> &'static Value { static V: OnceLock<Value> = OnceLock::new(); V.get_or_init(vocabularies) }

fn named(key: &str, name: &str) -> Result<i64, RunSessionError> {
    vocab()[key].as_array()
        .and_then(|rows| rows.iter().position(|row| row.as_str() == Some(name)))
        .filter(|index| *index > 0 || !standard_vocab_manifest()["open_families"].as_array().is_some_and(|f| f.iter().any(|k| k == key)))
        .map(|index| index as i64)
        .ok_or(RunSessionError::Input("unregistered Standard public identity"))
}

pub(crate) fn card(id: CardId) -> Result<i64, RunSessionError> { if matches!(id, CardId::Unknown(_)) { return Err(RunSessionError::Input("unknown Standard card")) } named("card", id.as_str()) }
pub(crate) fn relic(id: RelicId) -> Result<i64, RunSessionError> { if matches!(id, RelicId::Unknown(_)) { return Err(RunSessionError::Input("unknown Standard relic")) } named("relic", id.as_str()) }
pub(crate) fn potion(id: PotionId) -> Result<i64, RunSessionError> { if matches!(id, PotionId::Unknown(_)) { return Err(RunSessionError::Input("unknown Standard potion")) } named("potion", id.as_str()) }
pub(crate) fn enemy(id: EnemyId) -> Result<i64, RunSessionError> { if matches!(id, EnemyId::Unknown(_)) { return Err(RunSessionError::Input("unknown Standard enemy")) } named("enemy", id.as_str()) }
pub(crate) fn encounter(id: EncounterId) -> Result<i64, RunSessionError> { if matches!(id, EncounterId::Unknown(_)) { return Err(RunSessionError::Input("unknown Standard encounter")) } named("encounter", id.as_str()) }
pub(crate) fn event(id: EventId) -> Result<i64, RunSessionError> { if matches!(id, EventId::Unknown(_)) { return Err(RunSessionError::Input("unknown Standard event")) } named("event", id.as_str()) }
pub(crate) fn zone(name: &str) -> Result<i64, RunSessionError> { named("zone", name) }
pub(crate) fn room(kind: RoomViewKind) -> Result<i64, RunSessionError> { named("room", room_name(kind)) }
pub(crate) fn choice(kind: &RoomChoiceKind) -> Result<i64, RunSessionError> { named("choice", choice_name(kind)) }
pub(crate) fn verb(name: &str) -> Result<i64, RunSessionError> { named("verb", name) }
pub(crate) fn intent(name: &str) -> Result<i64, RunSessionError> {
    intent_family().position(|n| n == name).map(|i| i as i64 + 1).ok_or(RunSessionError::Input("unregistered public intent"))
}
pub(crate) fn intent_category(category: IntentCategory) -> Result<i64, RunSessionError> { named("intent_category", category.as_str()) }
pub(crate) fn profile(profile: RoomScenario) -> Result<i64, RunSessionError> {
    if profile == RoomScenario::Standard { named("profile", "Standard") } else { Err(RunSessionError::Input("Standard encoder requires the Standard profile")) }
}
pub(crate) fn operation(selection: &SelectionView) -> Result<i64, RunSessionError> { named("selection_operation", combat_operation_name(selection.operation)?) }
pub(crate) fn run_operation(operation: Option<RoomSelectionOperation>) -> Result<i64, RunSessionError> { named("selection_operation", run_operation_name(operation)) }

pub(crate) fn room_name(kind: RoomViewKind) -> &'static str {
    match kind {
        RoomViewKind::Neow => "neow", RoomViewKind::Map => "map", RoomViewKind::Combat => "combat", RoomViewKind::Rewards => "rewards",
        RoomViewKind::CardReward => "card_reward", RoomViewKind::Shop => "shop", RoomViewKind::Campfire => "campfire", RoomViewKind::Event(_) => "event",
        RoomViewKind::Treasure => "treasure", RoomViewKind::CheckpointComplete => "checkpoint_complete", RoomViewKind::BossTreasure => "boss_treasure",
        RoomViewKind::BossRelicChoice => "boss_relic_choice", RoomViewKind::BossRelicChild => "boss_relic_child",
        RoomViewKind::ProceedWithoutRelic => "proceed_without_relic", RoomViewKind::Act2Ready => "act2_ready",
        RoomViewKind::Act2PrefixComplete => "act2_prefix_complete", RoomViewKind::Act3Ready => "act3_ready",
        RoomViewKind::Act3PrefixComplete => "act3_prefix_complete", RoomViewKind::Act3BossComplete => "act3_boss_complete",
        RoomViewKind::Act3Ending => "act3_ending", RoomViewKind::Act4Ready => "act4_ready", RoomViewKind::HeartVictory => "heart_victory",
        RoomViewKind::TrueVictory => "true_victory",
    }
}
pub(crate) fn choice_name(kind: &RoomChoiceKind) -> &'static str {
    use RoomChoiceKind as K;
    match kind {
        K::NeowMaxHp { .. } => "neow_max_hp", K::NeowLament => "neow_lament", K::Gold(_) => "gold", K::CardReward => "card_reward",
        K::Card { .. } => "card", K::Potion(_) => "potion", K::Relic(_) => "relic", K::Sapphire => "sapphire", K::Emerald => "emerald",
        K::OpenChest => "open_chest", K::Rest { .. } => "rest", K::Smith => "smith", K::Recall => "recall", K::Lift => "lift", K::Toke => "toke",
        K::Dig => "dig", K::SingingBowl => "singing_bowl", K::Purge => "purge", K::ShrineUpgrade => "shrine_upgrade", K::ShrineRemove => "shrine_remove",
        K::AncientElegance => "ancient_elegance", K::AncientSimplicity => "ancient_simplicity", K::JoustMurderer => "joust_murderer",
        K::JoustOwner => "joust_owner", K::JoustContinue => "joust_continue", K::JoustResult { .. } => "joust_result",
        K::FallingLose { .. } => "falling_lose", K::TombGold => "tomb_gold", K::TombOfferGold => "tomb_offer_gold", K::Leave => "leave",
        K::Proceed => "proceed", K::Skip => "skip", K::ConfirmSkip => "confirm_skip", K::CancelSkip => "cancel_skip",
        K::Act3Continue { .. } => "act3_continue", K::NeowBlessing { .. } => "neow_blessing", K::EventOption { .. } => "event_option",
        K::EventCard { .. } => "event_card", K::EventPotion { .. } => "event_potion", K::EventRelic { .. } => "event_relic",
    }
}
fn combat_operation_name(operation: SelectionOperation) -> Result<&'static str, RunSessionError> {
    use SelectionOperation as O;
    Ok(match operation {
        O::Scry => "scry", O::MeditateReturn => "meditate_return", O::HologramRetrieve => "hologram_retrieve", O::SeekRetrieve => "seek_retrieve",
        O::Recycle => "recycle", O::Move(SelectionDisposition::Hand) => "move_hand", O::Move(SelectionDisposition::Discard) => "move_discard",
        O::Move(SelectionDisposition::Exhaust) => "move_exhaust", O::Move(SelectionDisposition::DrawTop) => "move_draw_top", O::Upgrade => "upgrade",
        O::Topdeck => "topdeck", O::ReturnFromExhaust => "return_from_exhaust", O::CopyToHand { copies: 1 } => "copy_to_hand_1",
        O::CopyToHand { copies: 2 } => "copy_to_hand_2", O::CopyToHand { copies: 3 } => "copy_to_hand_3",
        O::CopyToHand { .. } => return Err(RunSessionError::Input("unsupported public selection copy count")),
        O::Retain => "retain", O::NightmareCapture => "nightmare_capture", O::Setup => "setup", O::Bottomdeck => "bottomdeck",
        O::RetrieveMemories => "retrieve_memories", O::Omniscience => "omniscience",
    })
}
fn run_operation_name(operation: Option<RoomSelectionOperation>) -> &'static str {
    use RoomSelectionOperation as R;
    match operation {
        None => "select_permanent", Some(R::ShopPurge { .. }) => "shop_purge", Some(R::Smith) => "smith", Some(R::ShrineUpgrade) => "shrine_upgrade",
        Some(R::ShrineRemove) => "shrine_remove", Some(R::AncientElegance) => "ancient_elegance", Some(R::EmptyCage) => "empty_cage",
        Some(R::RelicEquip { .. }) => "relic_equip", Some(R::CampfireToke) => "campfire_toke",
        Some(R::NeowSelect { op, .. }) => match op { NeowSelectOp::Remove => "neow_remove", NeowSelectOp::Upgrade => "neow_upgrade", NeowSelectOp::Transform => "neow_transform" },
        Some(R::EventSelect { op, .. }) => match op { EventSelectOp::Remove => "event_remove", EventSelectOp::Upgrade => "event_upgrade",
            EventSelectOp::Transform => "event_transform", EventSelectOp::Duplicate => "event_duplicate", EventSelectOp::Choose => "event_choose" },
    }
}

/// Exhaustive engine enumerations for the families without a content id registry (manifest tests).
pub fn standard_registry_names(family: &str) -> Option<Vec<&'static str>> {
    let mut names: Vec<&'static str> = match family {
        "room" => [RoomViewKind::Neow, RoomViewKind::Map, RoomViewKind::Combat, RoomViewKind::Rewards, RoomViewKind::CardReward, RoomViewKind::Shop,
            RoomViewKind::Campfire, RoomViewKind::Event(EventId::Neow), RoomViewKind::Treasure, RoomViewKind::CheckpointComplete, RoomViewKind::BossTreasure,
            RoomViewKind::BossRelicChoice, RoomViewKind::BossRelicChild, RoomViewKind::ProceedWithoutRelic, RoomViewKind::Act2Ready,
            RoomViewKind::Act2PrefixComplete, RoomViewKind::Act3Ready, RoomViewKind::Act3PrefixComplete, RoomViewKind::Act3BossComplete,
            RoomViewKind::Act3Ending, RoomViewKind::Act4Ready, RoomViewKind::HeartVictory, RoomViewKind::TrueVictory].into_iter().map(room_name).collect(),
        "choice" => {
            use RoomChoiceKind as K;
            [K::NeowMaxHp { amount: 0 }, K::NeowLament, K::Gold(0), K::CardReward, K::Card { id: CardId::Strike, upgrades: 0 }, K::Potion(PotionId::FirePotion),
                K::Relic(RelicId::Vajra), K::Sapphire, K::Emerald, K::OpenChest, K::Rest { heal: 0 }, K::Smith, K::Recall, K::Lift, K::Toke, K::Dig,
                K::SingingBowl, K::Purge, K::ShrineUpgrade, K::ShrineRemove, K::AncientElegance, K::AncientSimplicity, K::JoustMurderer, K::JoustOwner,
                K::JoustContinue, K::JoustResult { owner_wins: false }, K::FallingLose { card_type: CardType::Attack, instance: None }, K::TombGold,
                K::TombOfferGold, K::Leave, K::Proceed, K::Skip, K::ConfirmSkip, K::CancelSkip, K::Act3Continue { stage: 0 },
                K::NeowBlessing { category: 0, reward: NeowReward::Gold, drawback: None }, K::EventOption { stage: 0, option: 0 },
                K::EventCard { stage: 0, option: 0, card: None }, K::EventPotion { stage: 0, option: 0, potion: None },
                K::EventRelic { stage: 0, option: 0, relic: None }].iter().map(choice_name).collect()
        }
        "selection_operation" => {
            use SelectionOperation as O;
            let mut combat = [O::Scry, O::MeditateReturn, O::HologramRetrieve, O::SeekRetrieve, O::Recycle, O::Move(SelectionDisposition::Hand),
                O::Move(SelectionDisposition::Discard), O::Move(SelectionDisposition::Exhaust), O::Move(SelectionDisposition::DrawTop), O::Upgrade,
                O::Topdeck, O::ReturnFromExhaust, O::CopyToHand { copies: 1 }, O::CopyToHand { copies: 2 }, O::CopyToHand { copies: 3 }, O::Retain,
                O::NightmareCapture, O::Setup, O::Bottomdeck, O::RetrieveMemories, O::Omniscience]
                .into_iter().map(|o| combat_operation_name(o).expect("listed operation")).collect::<Vec<_>>();
            use RoomSelectionOperation as R;
            combat.extend([None, Some(R::ShopPurge { price: 0 }), Some(R::Smith), Some(R::ShrineUpgrade), Some(R::ShrineRemove), Some(R::AncientElegance),
                Some(R::EmptyCage), Some(R::RelicEquip { relic: RelicId::Vajra }), Some(R::CampfireToke),
                Some(R::NeowSelect { op: NeowSelectOp::Remove, count: 1 }), Some(R::NeowSelect { op: NeowSelectOp::Upgrade, count: 1 }),
                Some(R::NeowSelect { op: NeowSelectOp::Transform, count: 1 })]
                .into_iter().chain([EventSelectOp::Remove, EventSelectOp::Upgrade, EventSelectOp::Transform, EventSelectOp::Duplicate, EventSelectOp::Choose]
                    .map(|op| Some(R::EventSelect { event: EventId::Neow, stage: 0, op, count: 1 })))
                .map(run_operation_name));
            combat
        }
        "intent_category" => IntentCategory::ALL.iter().map(|c| c.as_str()).collect(),
        _ => return None,
    };
    names.sort_unstable();
    names.dedup();
    Some(names)
}

/// Standard columns beyond the M25 layout (one owner; strings built once).
struct Layout { enemies_fields: String, enemies_units: String, offers_fields: String, offers_units: String, cards_fields: String, cards_units: String }
fn layout() -> &'static Layout {
    static L: OnceLock<Layout> = OnceLock::new();
    L.get_or_init(|| {
        let base = m25::groups(&RunCapacities::m10_default());
        let find = |key: &str| base.iter().find(|g| g.key == key).map(|g| (g.fields, g.units)).expect("M25 group");
        let (ef, eu) = find("enemies");
        let (of, ou) = find("offers");
        let (cf, cu) = find("combat_cards");
        Layout { enemies_fields: format!("{ef},intent_category"), enemies_units: format!("{eu},category"),
            offers_fields: format!("{of},event_stage,event_option,param0,param1,param2"), offers_units: format!("{ou},stage,option,display_value,display_value,display_value"),
            // M34s-a card lifecycle (a0(iv) follow-up): `core::combat::CardLifecycleView`, semantics `card_lifecycle`.
            cards_fields: format!("{cf},{CARD_LIFECYCLE_FIELDS}"), cards_units: format!("{cu},flag,flag,energy,energy,energy,flag") }
    })
}
/// The Standard card lifecycle columns appended to `combat_cards` (after the M25 columns).
pub(crate) const CARD_LIFECYCLE_FIELDS: &str = "autoplay_free,free_play_once,rolled_cost,modified_combat_cost,hp_loss_discount,cost_zero_this_turn";
/// Offer column indices of the Standard additions (after the 16 shared offer columns).
pub(crate) const OFFER_EVENT_STAGE: usize = 16;
pub(crate) const OFFER_PARAMS: usize = 18;

pub(crate) fn groups(caps: &RunCapacities) -> Vec<v1::GroupSpec> {
    let l = layout();
    let mut groups = m25::groups(caps);
    for g in &mut groups {
        match g.key {
            "enemies" => { g.fields = &l.enemies_fields; g.units = &l.enemies_units; }
            "offers" => { g.fields = &l.offers_fields; g.units = &l.offers_units; }
            "combat_cards" => { g.fields = &l.cards_fields; g.units = &l.cards_units; }
            _ => {}
        }
    }
    groups.push(v1::GroupSpec { key: "room_params", rows: None, fields: "stage0,stage1,stage2,stage3", units: "display_value,display_value,display_value,display_value", references: &[] });
    groups
}

pub(crate) fn validate(caps: &RunCapacities) -> Result<(), RunSessionError> { m10::validate_groups(&groups(caps), caps) }

/// Engine-bound column domains (M34a lane B domain table; M34b Task 0). Columns not listed keep the shared domain.
fn standard_domain(group: &str, field: &str) -> Option<(i64, i64, Option<&'static str>)> {
    Some(match (group, field) {
        ("context", "ascension") => (0, i64::from(spire_core::run::STANDARD_MAX_ASCENSION), None),
        // M34s-a draw rows carry zone_position -1 (position unknown; semantics draw_rows).
        ("combat_cards", "zone_position") => (-1, i64::MAX, None),
        ("enemies", "intent_kind") => (0, intent_family().count() as i64, Some("intent")),
        ("enemies", "intent_category") => (0, IntentCategory::ALL.len() as i64, Some("intent_category")),
        ("enemies", "intent_hits") => (0, i64::from(i32::MAX), None),
        ("combat_state", "previous_card_type") => (0, vocab()["card_type"].as_array().map_or(0, |v| v.len() as i64 - 1), Some("card_type")),
        // Standard card offers carry plain card ids (Wish choices are cards in the Standard vocabulary).
        ("combat_card_offers", "card_kind_or_wish_choice") => (0, vocab()["card"].as_array().map_or(0, |v| v.len() as i64 - 1), Some("card")),
        ("combat_state", "heart_cap") => (0, i64::from(spire_core::combat::enemies::ascension::heart_invincible(0)), None),
        ("offers", "event_stage") | ("offers", "event_option") => (0, i64::from(u8::MAX), None),
        ("offers", "param0" | "param1" | "param2") | ("room_params", _) => (i64::from(i32::MIN), i64::from(i32::MAX), None),
        _ => return None,
    })
}

pub fn standard_run_observation_spec(caps: &RunCapacities) -> Result<RunObservationSpec, RunSessionError> {
    validate(caps)?;
    let param_semantics = PARAM_SEMANTICS.iter().map(|p| json!({"event":p.event.as_str(),"stage":p.stage,"option":p.option,"index":p.index,"meaning":p.meaning})).collect::<Vec<_>>();
    let event_constants: Value = serde_json::from_str(EVENT_CONSTANTS_JSON).map_err(|_| RunSessionError::Input("invalid Standard event constants"))?;
    let semantics = json!({
        "profile":"Standard: every character, A0–A20, Acts 1–4 and the Heart, all M26–M33 content",
        "vocabulary_manifest":STANDARD_VOCAB_MANIFEST_ID,
        "capacity_preset":STANDARD_CAPACITY_PRESET_ID,
        "vocabulary_order":"open families are append-only from standard-vocab-v1; power = shared PUBLIC_POWERS order; intent = shared PUBLIC_INTENTS order then EVENT_INTENTS; a later shared power or intent appends after these rows",
        "padding":"zero,known0,occupied0","unknown_reference":"minus_one,known0 in occupied rows",
        "enemy_rows":"present actors only (active with hp>0, or half-dead), in engine order; replaced, escaped and defeated actors are not published; logical_slot is the actor's position; a Stasis card whose holder is no longer present keeps zone stasis with stasis_enemy_row unknown until the engine returns it",
        "intent":"intent_kind, intent_damage, intent_hits and intent_category are unknown while intents are hidden (Runic Dome)",
        "intent_hits":"planned damage applications of the selected move (engine-owned; includes AI memory such as Book of Stabbing); unknown for non-attacks",
        "heart_cap":"the present Corrupt Heart's live Invincible cap; unknown without a present Heart",
        "draw_rows":"M34s: every draw-pile card not published as known_draw (or as another row) is a combat_cards row in zone draw with zone_position -1/known0, after all other rows, sorted by the canonical card key (the published row without zone and zone_position, permanent_row last); tied rows are interchangeable and their order carries no information; never physical draw order",
        "card_lifecycle":"combat_cards lifecycle columns, each a fact the player sees or derives from public history and each read by later play: autoplay_free = the card still holds an autoplay waiver (displayed cost 0); free_play_once = a one-use waiver from Forethought or Setup (displayed cost 0); rolled_cost = the cost rolled by Confusion or Snecko Oil (shown when drawn), unknown when none; modified_combat_cost = the card's modified combat cost (cost changes this combat, e.g. Madness, Enlightenment, Force Field), unknown when none; hp_loss_discount = Blood for Blood's discount from HP losses this combat; cost_zero_this_turn = Infernal Blade's cost 0 that ends at end of turn",
        "intent_damage":"displayed damage per hit as the original calculateDamage shows it, including the player's Intangible cap (1)",
        "offers":"choice payloads: amount/card/relic/potion/neow columns as named; event_stage and event_option for event rows; param0..2 are the row's displayed numbers per param_semantics",
        "room_params":"stage0..3: numbers the current room displays (Treasure stage0 = chest size 1/2/3); meanings in param_semantics",
        "param_semantics":param_semantics,
        "event_constants":event_constants,
        "selection_operation":"combat selections name their operation; Run card selections name what they are for (shop_purge, smith, neow_*, event_*), select_permanent otherwise",
        "terminal":"ongoing, death, empty_deck, act3_no_keys and heart_victory; HeartVictory follows the final Proceed; truncated is always false",
        "creation":"the resolved creation profile is configuration and player knowledge: environment_spec, reset info and the record; never a per-step column",
    });
    let mut spec = m10::spec_from_vocabulary(caps, RUN_OBSERVATION_STANDARD_SCHEMA_ID, "Standard", vocab().clone(), semantics, groups(caps), DOMAIN)?;
    // `spec_from_vocabulary` installs the shared intent table; Standard's family also carries the event intents.
    spec.descriptor["vocabularies"]["intent"] = vocab()["intent"].clone();
    for tensor in spec.descriptor["tensors"].as_array_mut().expect("descriptor tensors") {
        let Some(key) = tensor["key"].as_str().map(str::to_owned) else { continue };
        let Some(fields) = tensor["fields"].as_array().cloned() else { continue };
        for (index, field) in fields.iter().enumerate() {
            let Some((low, high, vocabulary)) = standard_domain(&key, field.as_str().unwrap_or("")) else { continue };
            let mut column = json!({"field":field,"unit":tensor["units"][index],"low":low,"high":high});
            if let Some(v) = vocabulary { column["vocabulary"] = json!(v); }
            tensor["columns"][index] = column;
            tensor["low"][index] = json!(low);
            tensor["high"][index] = json!(high);
        }
    }
    // As M21–M25: the stance-choice reference column is -1 (unavailable) in occupied rows.
    crate::run_schema_m21::finalize_stance_descriptor(&mut spec.descriptor);
    let mut hash = Sha256::new();
    hash.update(DOMAIN);
    hash.update(crate::canonical_json(&spec.descriptor).map_err(|_| RunSessionError::Input("invalid Standard descriptor"))?);
    spec.fingerprint = hash.finalize().into();
    Ok(spec)
}

pub(crate) fn neow_reward(reward: NeowReward) -> i64 { v1::reward(reward) }
pub(crate) fn neow_drawback(drawback: Option<NeowDrawback>) -> i64 { drawback.map(v1::drawback).unwrap_or(0) }
