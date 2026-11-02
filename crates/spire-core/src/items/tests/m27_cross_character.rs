//! M27 plan Task X1, lane X, key m27_cross_character.
//! Cross-character deltas use public_full / Run::decision projections. The four Heart-context
//! complete-item inertness proof is m27_capability_overlay_inert in m27_inert.rs:55.

use crate::{
    combat::{self, Combat, CombatAction, CombatPublic, FullPowerView, LiveCardView, TargetId},
    content::{
        self, complete, CardColor, CardId, CardRarity, CardType, Character, EncounterId, PotionId,
        RelicId,
    },
    items::{self, CombatItemContext, NativeBattleContext},
    run::{self, GameAction, Run, RunRng},
};
use std::collections::BTreeMap;

const CHARACTERS: [Character; 4] = [
    Character::Ironclad,
    Character::Silent,
    Character::Defect,
    Character::Watcher,
];

/// §4.1's literal shared count is 139; IDs come from each character's complete catalog union.
fn shared_relics() -> Vec<RelicId> {
    let catalog = content::Catalog::standard();
    let ids = complete::relic_union(Character::Ironclad)
        .into_iter()
        .filter(|id| CHARACTERS.iter().all(|character| {
            catalog.relic_eligible_for(*id, *character) == Some(true)
        }))
        .collect::<Vec<_>>();
    assert_eq!(ids.len(), 139, "design §4.1: 139 shared relics");
    ids
}

/// §4.3's literal shared count is 30; IDs come from each character's complete catalog union.
fn shared_potions() -> Vec<PotionId> {
    let catalog = content::Catalog::standard();
    let ids = complete::potion_union(Character::Ironclad)
        .into_iter()
        .filter(|id| CHARACTERS.iter().all(|character| {
            catalog.potion_eligible_for(*id, *character) == Some(true)
        }))
        .collect::<Vec<_>>();
    assert_eq!(ids.len(), 30, "design §4.3: 30 shared potions");
    ids
}

/// §15.3 / M26 seam-map §6 basis: Jaw Worm setup HP, 80/80, 10 energy, five red Defends,
/// opening draw 1, and only the one row item. The complete flags admit the stripped cards/items.
fn matrix_combat(
    character: Character,
    relic: Option<RelicId>,
    potion: Option<PotionId>,
    encounter: EncounterId,
    energy: i32,
    max_energy: i32,
    cards: &[CardId],
    opening_draw: usize,
) -> Combat {
    matrix_combat_with_battle(character, relic, potion, encounter, energy, max_energy, cards, opening_draw, None)
}

fn matrix_combat_with_battle(
    character: Character,
    relic: Option<RelicId>,
    potion: Option<PotionId>,
    encounter: EncounterId,
    energy: i32,
    max_energy: i32,
    cards: &[CardId],
    opening_draw: usize,
    battle: Option<NativeBattleContext>,
) -> Combat {
    let mut rng = RunRng::new([41; 32], combat::rng::Partition::Test, 0);
    let cards = cards.iter().enumerate()
        .map(|(index, id)| combat::complete_live_card(*id, combat::CardInstanceId(index as u64 + 1), 0)
            .unwrap_or_else(|error| panic!("complete {character:?} {id:?}: {error:?}")))
        .collect();
    let entry = combat::enemies::setup(encounter, 1, 0, &mut rng,
        combat::enemies::SetupContext {
            player: combat::PlayerState {
                hp: 80, max_hp: 80, block: 0, energy, max_energy, draw_per_turn: 5, powers: vec![],
            },
            player_powers: vec![], character, cards, opening_draw, encounter_visit: 0,
        })
        .unwrap_or_else(|error| panic!("Jaw Worm setup {character:?}: {error:?}"));
    // Potion Belt is an already-equipped Run item; its complete public context has five slots (design §8.1 / G3).
    let mut potions = vec![None; if relic == Some(RelicId::PotionBelt) { 5 } else { 3 }];
    potions[0] = potion;
    let mut context = CombatItemContext::new(relic.into_iter().collect(), potions)
        .with_complete_cards().with_complete_items();
    if let Some(battle) = battle { context = context.with_battle_context(battle); }
    let mut owner = Some(context);
    Combat::enter_with_item_context(entry, &mut owner)
        .unwrap_or_else(|error| panic!("complete Jaw Worm entry {character:?} {relic:?} {potion:?}: {error:?}"))
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct CombatDelta {
    hp: i32,
    max_hp: i32,
    block: i32,
    energy: i32,
    max_energy: i32,
    powers: BTreeMap<String, i32>,
    enemies: Vec<(String, i32, i32, BTreeMap<String, i32>)>,
    hand: BTreeMap<(CardId, u32), i32>,
    draw_count: i32,
    discard: BTreeMap<(CardId, u32), i32>,
    exhaust: BTreeMap<(CardId, u32), i32>,
    relic_item_delta: BTreeMap<RelicId, i32>,
    counter_item_delta: BTreeMap<RelicId, i32>,
    potion_slots: Vec<(usize, Option<PotionId>, Option<PotionId>)>,
    orb_slot_count: i32,
    orb_changes: Vec<(usize, Option<combat::OrbKind>, Option<combat::OrbKind>)>,
    orb_dark_damage: Vec<(usize, Option<i32>, Option<i32>)>,
    focus: i32,
    stance: (combat::Stance, combat::Stance),
    offers: Vec<Vec<CardId>>,
    outcome: Option<combat::Outcome>,
}

fn power_counts(powers: &[FullPowerView]) -> BTreeMap<String, i32> {
    powers.iter().map(|power| (power.id.0.to_owned(), power.amount)).collect()
}

fn card_counts(cards: &[LiveCardView]) -> BTreeMap<(CardId, u32), i32> {
    let mut counts = BTreeMap::new();
    for card in cards { *counts.entry((card.id, card.upgrades)).or_insert(0) += 1; }
    counts
}

fn map_delta<K: Ord + Clone>(before: &BTreeMap<K, i32>, after: &BTreeMap<K, i32>) -> BTreeMap<K, i32> {
    let mut keys = before.keys().chain(after.keys()).cloned().collect::<std::collections::BTreeSet<_>>();
    std::mem::take(&mut keys).into_iter().filter_map(|key| {
        let change = after.get(&key).copied().unwrap_or(0) - before.get(&key).copied().unwrap_or(0);
        (change != 0).then_some((key, change))
    }).collect()
}

fn relic_counts(inventory: &combat::CombatInventoryView) -> BTreeMap<RelicId, i32> {
    let mut counts = BTreeMap::new();
    for relic in &inventory.relics { *counts.entry(*relic).or_insert(0) += 1; }
    counts
}

fn relic_counter_counts(inventory: &combat::CombatInventoryView) -> BTreeMap<RelicId, i32> {
    inventory.relic_counters.iter().map(|counter| (counter.relic, counter.value)).collect()
}

fn public_delta(before: &CombatPublic, after: &CombatPublic, offers: Vec<Vec<CardId>>) -> CombatDelta {
    let before_inventory = before.inventory.as_ref().expect("complete public inventory before");
    let after_inventory = after.inventory.as_ref().expect("complete public inventory after");
    let mut enemies = before.enemies.iter().zip(&after.enemies).map(|(old, new)| (
        format!("{:?}", new.enemy_id), new.hp - old.hp, new.block - old.block,
        map_delta(&power_counts(&old.powers), &power_counts(&new.powers)),
    )).collect::<Vec<_>>();
    enemies.sort_by(|left, right| left.0.cmp(&right.0));
    let max_orbs = before.character.orbs.len().max(after.character.orbs.len());
    let orb_changes = (0..max_orbs).filter_map(|index| {
        let old = before.character.orbs.get(index).copied().flatten();
        let new = after.character.orbs.get(index).copied().flatten();
        (old != new).then_some((index, old, new))
    }).collect();
    let max_dark = before.character.orb_dark_damage.len().max(after.character.orb_dark_damage.len());
    let orb_dark_damage = (0..max_dark).filter_map(|index| {
        let old = before.character.orb_dark_damage.get(index).copied().flatten();
        let new = after.character.orb_dark_damage.get(index).copied().flatten();
        (old != new).then_some((index, old, new))
    }).collect();
    let potion_slots = (0..before_inventory.potions.len().max(after_inventory.potions.len()))
        .filter_map(|index| {
            let old = before_inventory.potions.get(index).copied().flatten();
            let new = after_inventory.potions.get(index).copied().flatten();
            (old != new).then_some((index, old, new))
        }).collect();
    CombatDelta {
        hp: after.player.hp - before.player.hp,
        max_hp: after.player.max_hp - before.player.max_hp,
        block: after.player.block - before.player.block,
        energy: after.player.energy - before.player.energy,
        max_energy: after.player.max_energy - before.player.max_energy,
        powers: map_delta(&power_counts(&before.player.powers), &power_counts(&after.player.powers)),
        enemies,
        hand: map_delta(&card_counts(&before.hand), &card_counts(&after.hand)),
        draw_count: after.draw_count as i32 - before.draw_count as i32,
        discard: map_delta(&card_counts(&before.discard), &card_counts(&after.discard)),
        exhaust: map_delta(&card_counts(&before.exhaust), &card_counts(&after.exhaust)),
        relic_item_delta: map_delta(&relic_counts(before_inventory), &relic_counts(after_inventory)),
        counter_item_delta: map_delta(&relic_counter_counts(before_inventory), &relic_counter_counts(after_inventory)),
        potion_slots,
        orb_slot_count: after.character.orbs.len() as i32 - before.character.orbs.len() as i32,
        orb_changes,
        orb_dark_damage,
        focus: after.character.focus - before.character.focus,
        stance: (before.character.stance, after.character.stance),
        offers,
        outcome: after.outcome,
    }
}

fn resolve_public_choices(combat: &mut Combat, offers: &mut Vec<Vec<CardId>>, row: &str) {
    for _ in 0..32 {
        let public = combat.public_full().unwrap_or_else(|error| panic!("{row} public selection: {error:?}"));
        if let Some(offer) = public.card_offer {
            offers.push(offer.cards);
            combat.step_full(&CombatAction::ChooseCardOffer { index: 0 })
                .unwrap_or_else(|error| panic!("{row} choose first generated card: {error:?}"));
            continue;
        }
        if let Some(selection) = public.selection {
            let actions = combat.legal_actions_full().unwrap_or_else(|error| panic!("{row} selection actions: {error:?}"));
            let action = if selection.selected.len() >= selection.required {
                actions.into_iter().find(|action| matches!(action, CombatAction::ConfirmSelection))
            } else {
                actions.into_iter().find(|action| matches!(action, CombatAction::SelectCard { .. }))
            }.unwrap_or_else(|| panic!("{row} no public continuation for selection {selection:?}"));
            combat.step_full(&action).unwrap_or_else(|error| panic!("{row} selection {action:?}: {error:?}"));
            continue;
        }
        if let Some(choice) = public.stance_choice {
            combat.step_full(&CombatAction::ChooseStance { stance: choice.options[0] })
                .unwrap_or_else(|error| panic!("{row} choose first public stance: {error:?}"));
            continue;
        }
        return;
    }
    panic!("{row} public choices exceeded 32 continuations");
}

struct CombatResult { public: CombatPublic, offers: Vec<Vec<CardId>> }

fn play_matrix_script(
    character: Character,
    relic: Option<RelicId>,
    potion: Option<PotionId>,
    encounter: EncounterId,
    energy: i32,
    max_energy: i32,
    cards: &[CardId],
    opening_draw: usize,
    row: &str,
) -> CombatResult {
    let mut combat = matrix_combat(character, relic, potion, encounter, energy, max_energy, cards, opening_draw);
    let mut offers = Vec::new();
    resolve_public_choices(&mut combat, &mut offers, row);
    let actions = combat.legal_actions_full().unwrap_or_else(|error| panic!("{row} public actions: {error:?}"));
    if let Some(action) = actions.into_iter().find(|action| matches!(action, CombatAction::PlayCard { .. })) {
        combat.step_full(&action).unwrap_or_else(|error| panic!("{row} play first legal card {action:?}: {error:?}"));
        resolve_public_choices(&mut combat, &mut offers, row);
    }
    if combat.public_full().ok().and_then(|public| public.outcome).is_none() {
        let actions = combat.legal_actions_full().unwrap_or_else(|error| panic!("{row} potion actions: {error:?}"));
        if let Some(action) = actions.into_iter().find(|action| matches!(action,
            CombatAction::UsePotion { slot: 0, .. })) {
            combat.step_full(&action).unwrap_or_else(|error| panic!("{row} use potion {action:?}: {error:?}"));
            resolve_public_choices(&mut combat, &mut offers, row);
        }
    }
    if combat.public_full().ok().and_then(|public| public.outcome).is_none() {
        combat.step_full(&CombatAction::EndTurn)
            .unwrap_or_else(|error| panic!("{row} end player turn: {error:?}"));
        resolve_public_choices(&mut combat, &mut offers, row);
    }
    let public = combat.public_full().unwrap_or_else(|error| panic!("{row} final public view: {error:?}"));
    CombatResult { public, offers }
}

fn combat_item_delta(character: Character, relic: Option<RelicId>, potion: Option<PotionId>, row: &str) -> CombatDelta {
    // Compare the public result with the same character, seed, deck and script without the row item.
    let cards = [CardId::Defend; 5];
    let baseline = play_matrix_script(character, None, None, EncounterId::JawWorm, 10, 10, &cards, 1, row);
    let with_item = play_matrix_script(character, relic, potion, EncounterId::JawWorm, 10, 10, &cards, 1, row);
    public_delta(&baseline.public, &with_item.public, with_item.offers)
}

fn card_counts_in_run(objects: &[run::PublicObject]) -> BTreeMap<(CardId, u32), i32> {
    let mut counts = BTreeMap::new();
    for card in objects { *counts.entry((card.id, card.upgrades)).or_insert(0) += 1; }
    counts
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct RunDelta {
    hp: i64,
    max_hp: i64,
    gold: i64,
    relic_item_delta: BTreeMap<RelicId, i32>,
    counter_item_delta: BTreeMap<RelicId, i32>,
    potion_slots: Vec<(usize, Option<PotionId>, Option<PotionId>)>,
    deck: BTreeMap<(CardId, u32), i32>,
    rule_results: Vec<String>,
}

fn run_relic_counts(player: &run::PublicPlayer) -> BTreeMap<RelicId, i32> {
    let mut counts = BTreeMap::new();
    for relic in &player.relics { *counts.entry(*relic).or_insert(0) += 1; }
    counts
}

fn run_counter_counts(player: &run::PublicPlayer) -> BTreeMap<RelicId, i32> {
    player.relic_counters.iter().map(|counter| (counter.relic, counter.value)).collect()
}

fn run_public_delta(before: &run::PublicDecision, after: &run::PublicDecision, rule_results: Vec<String>) -> RunDelta {
    let potion_slots = (0..before.player.potions.len().max(after.player.potions.len()))
        .filter_map(|index| {
            let old = before.player.potions.get(index).copied().flatten();
            let new = after.player.potions.get(index).copied().flatten();
            (old != new).then_some((index, old, new))
        }).collect();
    RunDelta {
        hp: i64::from(after.player.hp) - i64::from(before.player.hp),
        max_hp: i64::from(after.player.max_hp) - i64::from(before.player.max_hp),
        gold: i64::from(after.player.gold) - i64::from(before.player.gold),
        relic_item_delta: map_delta(&run_relic_counts(&before.player), &run_relic_counts(&after.player)),
        counter_item_delta: map_delta(&run_counter_counts(&before.player), &run_counter_counts(&after.player)),
        potion_slots,
        deck: map_delta(&card_counts_in_run(&before.objects), &card_counts_in_run(&after.objects)),
        rule_results,
    }
}

fn dispatch_run_event(run: &mut Run, event: &mut items::dispatch::RunItemEvent) {
    items::dispatch::run_event(run.state_mut_for_test(), event)
        .unwrap_or_else(|error| panic!("Run item event {event:?}: {error:?}"));
}

fn dispatch_and_record(run: &mut Run, mut event: items::dispatch::RunItemEvent, results: &mut Vec<String>) {
    dispatch_run_event(run, &mut event);
    results.push(format!("{event:?}"));
}

/// A small m27_support complete Run with one owned item; all measured values come from Run::decision.
fn run_event_delta(character: Character, relic: Option<RelicId>, row: &str) -> RunDelta {
    use items::{dispatch::RunItemEvent, run_laws::RewardRoom};
    let counters = BTreeMap::new();
    let mut run = items::tests::m27_support::complete_run(
        character, relic.into_iter().collect(), vec![None; 3], counters, EncounterId::Cultist,
    );
    for _ in 0..128 {
        let decision = run.decision().unwrap_or_else(|error| panic!("{row} Run public view: {error:?}"));
        if decision.combat.is_none() { break; }
        let action = decision.candidates.iter().find(|candidate| matches!(candidate.action,
                GameAction::Combat(CombatAction::PlayCard { .. })))
            .or_else(|| decision.candidates.iter().find(|candidate| matches!(candidate.action,
                GameAction::Combat(CombatAction::EndTurn))))
            .or_else(|| decision.candidates.first())
            .unwrap_or_else(|| panic!("{row} no public legal Run combat action"))
            .action.clone();
        run.step(&action).unwrap_or_else(|error| panic!("{row} public action {action:?}: {error:?}"));
    }
    let before = run.decision().unwrap_or_else(|error| panic!("{row} settled Run public view: {error:?}"));
    assert!(before.combat.is_none(), "{row} support combat settled through public actions");
    // Lowering HP makes the public heal delta observable; this is fixture construction only.
    run.state_mut_for_test().hp = 40;
    let before = run.decision().expect("Run public view after fixture HP setup");
    let mut results = Vec::new();
    dispatch_and_record(&mut run, RunItemEvent::FloorEntered { floor: 2 }, &mut results);
    dispatch_and_record(&mut run, RunItemEvent::RoomEntered { kind: run::RoomViewKind::Campfire }, &mut results);
    dispatch_and_record(&mut run, RunItemEvent::RoomEntered { kind: run::RoomViewKind::Shop }, &mut results);
    dispatch_and_record(&mut run, RunItemEvent::RoomEntered { kind: run::RoomViewKind::Event(content::EventId::Unknown("x1-matrix")) }, &mut results);
    dispatch_and_record(&mut run, RunItemEvent::CampfireOptions { options: vec![
        run::RoomChoiceKind::Rest { heal: 20 }, run::RoomChoiceKind::Smith,
    ] }, &mut results);
    let mut rest_heal = RunItemEvent::RestHeal { amount: 20 };
    dispatch_run_event(&mut run, &mut rest_heal);
    let amount = match rest_heal { RunItemEvent::RestHeal { amount } => amount, _ => unreachable!() };
    run::mutations::heal(run.state_mut_for_test(), amount).unwrap_or_else(|error| panic!("{row} Run rest heal: {error:?}"));
    results.push(format!("{rest_heal:?}"));
    dispatch_and_record(&mut run, RunItemEvent::Rested { card_reward: false }, &mut results);
    dispatch_and_record(&mut run, RunItemEvent::ShopEntered, &mut results);
    dispatch_and_record(&mut run, RunItemEvent::ShopPrice { price: 150 }, &mut results);
    dispatch_and_record(&mut run, RunItemEvent::ShopPurchase, &mut results);
    dispatch_and_record(&mut run, RunItemEvent::RemovalPrice { price: 75 }, &mut results);
    dispatch_and_record(&mut run, RunItemEvent::RewardsBuilding {
        room: RewardRoom::Ordinary { enemy_escaped: false }, card_choices: 3,
        extra_card_rewards: 0, gold_percent: 0, extra_elite_relics: 0,
        include_potion: false, singing_bowl: false,
    }, &mut results);
    dispatch_and_record(&mut run, RunItemEvent::RewardsBuilding {
        room: RewardRoom::Elite, card_choices: 3, extra_card_rewards: 0,
        gold_percent: 0, extra_elite_relics: 0, include_potion: false, singing_bowl: false,
    }, &mut results);
    dispatch_and_record(&mut run, RunItemEvent::ChestOpened { boss: false }, &mut results);
    run::mutations::obtain_card(run.state_mut_for_test(), CardId::Regret, 0)
        .unwrap_or_else(|error| panic!("{row} public curse obtain: {error:?}"));
    run::mutations::gain_gold(run.state_mut_for_test(), 10)
        .unwrap_or_else(|error| panic!("{row} public gold gain: {error:?}"));
    dispatch_and_record(&mut run, RunItemEvent::CombatSettled { outcome: combat::Outcome::Victory }, &mut results);
    let after = run.decision().unwrap_or_else(|error| panic!("{row} final Run public view: {error:?}"));
    run_public_delta(&before, &after, results)
}

fn run_owner_relics() -> &'static [RelicId] { items::relic_effects::run::RELICS }

fn character_card_color(character: Character) -> CardColor {
    // Literal class colors from M26 content/pools.rs:57.
    match character {
        Character::Ironclad => CardColor::Red,
        Character::Silent => CardColor::Green,
        Character::Defect => CardColor::Blue,
        Character::Watcher => CardColor::Purple,
    }
}

/// M26 content/pools.rs:20 and 30-31 define ordinary class-color pools and non-healing combat pools.
fn assert_character_card(
    character: Character,
    item: &str,
    id: CardId,
    want: Option<CardType>,
    combat_pool: bool,
) {
    let row = content::Catalog::standard().card(id)
        .unwrap_or_else(|| panic!("{item} / {character:?}: generated card {id:?} is in the public catalog"));
    assert_eq!(row.color, character_card_color(character),
        "{item} / {character:?}: {id:?} must be from that character's literal color pool");
    if let Some(want) = want {
        assert_eq!(row.card_type, want, "{item} / {character:?}: {id:?} has the requested pool type");
    }
    assert!(matches!(row.rarity, CardRarity::Common | CardRarity::Uncommon | CardRarity::Rare),
        "{item} / {character:?}: {id:?} is in the literal ordinary pool");
    if combat_pool {
        assert_ne!(id, CardId::SelfRepair,
            "{item} / {character:?}: M26 combat pools exclude healing Self Repair");
    }
}

/// §4.3 / Appendix B literal pool: 30 shared IDs plus only the owning character's three class IDs.
const SHARED_POTION_POOL: [PotionId; 30] = [
    PotionId::FirePotion, PotionId::FruitJuice, PotionId::BlockPotion, PotionId::EnergyPotion,
    PotionId::StrengthPotion, PotionId::SwiftPotion, PotionId::WeakPotion, PotionId::FearPotion,
    PotionId::DexterityPotion, PotionId::AncientPotion, PotionId::SmokeBomb, PotionId::AttackPotion,
    PotionId::SkillPotion, PotionId::PowerPotion, PotionId::ColorlessPotion, PotionId::FlexPotion,
    PotionId::SpeedPotion, PotionId::RegenPotion, PotionId::LiquidBronze, PotionId::EssenceOfSteel,
    PotionId::CultistPotion, PotionId::BlessingOfTheForge, PotionId::SneckoOil, PotionId::FairyPotion,
    PotionId::ExplosivePotion, PotionId::GamblersBrew, PotionId::LiquidMemories,
    PotionId::DuplicationPotion, PotionId::DistilledChaos, PotionId::EntropicBrew,
];

/// The four class triplets are literal §4.3 / Appendix B potion-union entries.
fn class_potions(character: Character) -> [PotionId; 3] {
    match character {
        Character::Ironclad => [PotionId::BloodPotion, PotionId::Elixir, PotionId::HeartOfIron],
        Character::Silent => [PotionId::PoisonPotion, PotionId::CunningPotion, PotionId::GhostInAJar],
        Character::Defect => [PotionId::FocusPotion, PotionId::PotionOfCapacity, PotionId::EssenceOfDarkness],
        Character::Watcher => [PotionId::BottledMiracle, PotionId::StancePotion, PotionId::Ambrosia],
    }
}

/// A complete Run that can exercise equip offers through public room choices.
/// Potion Belt supplies two extra slots only for the five Cauldron offers (design §8.1 / G3).
fn equip_offer_run(character: Character, relic: RelicId) -> Run {
    let cauldron = relic == RelicId::Cauldron;
    let owned = if cauldron { vec![RelicId::PotionBelt] } else { vec![] };
    let mut potions = vec![Some(PotionId::FirePotion); 3];
    if cauldron { potions.resize(5, None); }
    let mut run = items::tests::m27_support::complete_run(
        character, owned, potions, BTreeMap::new(), EncounterId::Cultist,
    );
    // Three public Fire Potion uses settle the support Cultist before the equip screen is installed.
    for slot in 0..3 {
        run.step(&GameAction::Combat(CombatAction::UsePotion {
            slot, target: Some(combat::TargetId(1)),
        })).unwrap_or_else(|error| panic!("{relic:?} / {character:?}: support potion {slot}: {error:?}"));
    }
    assert!(run.decision().expect("settled support Run public view").combat.is_none());

    let node = run::NodeId(1);
    let state = run.state_mut_for_test();
    state.act = 1;
    state.floor = 1;
    state.map = Some(run::RunMap {
        act: 1,
        current: Some(node),
        nodes: vec![run::MapNode {
            id: node, floor: 1, label: "x1 equip fixture", edges: vec![], x: 0, y: 0,
            kind: run::MapNodeKind::Monster, hidden_kind: None, visited: true,
            revealed: true, encounter: None, burning: false,
        }],
        ..run::RunMap::default()
    });
    state.room_loop = Some(run::rooms::RoomLoop {
        scenario: run::RoomScenario::M5Act1Captured,
        act1: None, act2: None, act3: None, act4: None, m10_elite_relic_cursor: 0,
        screen: run::RoomView {
            kind: run::RoomViewKind::Rewards,
            choices: vec![
                run::RoomChoiceView { id: run::ChoiceId(100), kind: run::RoomChoiceKind::Relic(relic),
                    price: None, status: run::RoomChoiceStatus::Offered },
                run::RoomChoiceView { id: run::ChoiceId(101), kind: run::RoomChoiceKind::Proceed,
                    price: None, status: run::RoomChoiceStatus::Offered },
            ],
        },
        purge_cost: 75, reward_parent: None, active_rewards: None, boss: None, transition: None,
        act2_boss_order: None, act3_boss_order: None, joust: None, joust_visits: 0,
        act3_event: None, act3_end: None, standard_event: None,
    });
    let public = run.decision().expect("public equip fixture choices");
    let choice = public.room.unwrap().choices.into_iter().find(|choice|
        choice.kind == run::RoomChoiceKind::Relic(relic)
            && choice.status == run::RoomChoiceStatus::Offered)
        .unwrap_or_else(|| panic!("{relic:?} / {character:?}: public relic obtain choice"));
    run.step(&GameAction::Choose(choice.id))
        .unwrap_or_else(|error| panic!("{relic:?} / {character:?}: public relic obtain: {error:?}"));
    assert!(run.decision().expect("public post-obtain Run view").player.relics.contains(&relic),
        "{relic:?} / {character:?}: the item is equipped through the public Run owner");
    run
}

fn public_room(run: &Run, item: &str) -> run::RoomView {
    run.decision().unwrap_or_else(|error| panic!("{item} public Run decision: {error:?}"))
        .room.unwrap_or_else(|| panic!("{item} public Run room exists"))
}

fn choose_public_room(run: &mut Run, item: &str, wanted: impl Fn(&run::RoomChoiceKind) -> bool) {
    let choice = public_room(run, item).choices.into_iter().find(|choice|
        choice.status == run::RoomChoiceStatus::Offered && wanted(&choice.kind))
        .unwrap_or_else(|| panic!("{item}: public offered choice exists"));
    run.step(&GameAction::Choose(choice.id))
        .unwrap_or_else(|error| panic!("{item}: public choice {:?}: {error:?}", choice.kind));
}

fn assert_character_potion(character: Character, item: &str, id: PotionId) {
    let pool = class_potions(character);
    assert!(SHARED_POTION_POOL.contains(&id) || pool.contains(&id),
        "{item} / {character:?}: {id:?} is in that character's literal 33-potion pool");
    for other in CHARACTERS {
        if other != character {
            assert!(!class_potions(other).contains(&id),
                "{item} / {character:?}: {id:?} never comes from {other:?}'s class potion pool");
        }
    }
}

fn relic_has_class_pool(id: RelicId) -> bool {
    matches!(id, RelicId::DeadBranch | RelicId::Enchiridion | RelicId::NilrysCodex
        | RelicId::Toolbox | RelicId::Orrery | RelicId::TinyHouse)
}

fn potion_has_class_card_pool(id: PotionId) -> bool {
    matches!(id, PotionId::AttackPotion | PotionId::SkillPotion | PotionId::PowerPotion)
}

fn potion_has_class_potion_pool(id: PotionId) -> bool { id == PotionId::EntropicBrew }

fn character_sensitive_relic(id: RelicId) -> bool {
    relic_has_class_pool(id) || matches!(id, RelicId::PrismaticShard | RelicId::Cauldron | RelicId::WhiteBeastStatue)
}

#[test]
fn m27_cross_character_shared_relic_deltas() {
    for relic in shared_relics() {
        for character in CHARACTERS.into_iter().skip(1) {
            if character_sensitive_relic(relic) { continue; }
            let expected = combat_item_delta(Character::Ironclad, Some(relic), None, &format!("{relic:?}"));
            let actual = combat_item_delta(character, Some(relic), None, &format!("{relic:?}"));
            assert_eq!(actual, expected, "{relic:?} / {character:?}: shared relic public delta equals Ironclad");
            if run_owner_relics().contains(&relic) {
                let expected_run = run_event_delta(Character::Ironclad, Some(relic), &format!("{relic:?} Run"));
                let actual_run = run_event_delta(character, Some(relic), &format!("{relic:?} Run"));
                assert_eq!(actual_run, expected_run,
                    "{relic:?} / {character:?}: shared Run-owner public delta equals Ironclad");
            }
        }
    }
}

#[test]
fn m27_cross_character_shared_potion_deltas() {
    for potion in shared_potions() {
        for character in CHARACTERS.into_iter().skip(1) {
            if potion_has_class_card_pool(potion) || potion_has_class_potion_pool(potion) { continue; }
            let expected = combat_item_delta(Character::Ironclad, None, Some(potion), &format!("{potion:?}"));
            let actual = combat_item_delta(character, None, Some(potion), &format!("{potion:?}"));
            assert_eq!(actual, expected, "{potion:?} / {character:?}: shared potion public delta equals Ironclad");
        }
    }
}

#[test]
fn m27_cross_character_energy_recharge_literals() {
    // Design §12 G1 and Review Focus 4: each unconditional boss-energy relic and Sozu adds +1 master energy.
    const ENERGY_RELICS: [RelicId; 9] = [
        RelicId::Sozu, RelicId::BustedCrown, RelicId::CoffeeDripper, RelicId::CursedKey,
        RelicId::Ectoplasm, RelicId::FusionHammer, RelicId::PhilosophersStone,
        RelicId::RunicDome, RelicId::VelvetChoker,
    ];
    for character in CHARACTERS {
        for relic in ENERGY_RELICS {
            let combat = matrix_combat(character, Some(relic), None, EncounterId::JawWorm, 10, 10,
                &[CardId::Defend; 5], 1);
            let public = combat.public_full().unwrap();
            assert_eq!((public.player.energy, public.player.max_energy), (11, 11),
                "{relic:?} / {character:?}: one master energy and one recharge");
            let after = play_matrix_script(character, Some(relic), None, EncounterId::JawWorm, 10, 10,
                &[CardId::Defend; 5], 1, "energy recharge").public;
            assert_eq!((after.player.energy, after.player.max_energy), (11, 11),
                "{relic:?} / {character:?}: next turn recharges to its 11-energy maximum");
        }
        let collar = matrix_combat(character, Some(RelicId::SlaversCollar), None, EncounterId::JawWorm,
            10, 10, &[CardId::Defend; 5], 1).public_full().unwrap();
        assert_eq!((collar.player.energy, collar.player.max_energy), (10, 10),
            "SlaversCollar / {character:?}: Jaw Worm is not an elite or boss combat");
        // Slaver's Collar adds +1 only when Run entry marks an elite or boss (accepted law
        // m27_energy_law.rs:104-106; Run maps the origin at run/combat.rs:125).
        let collar_battle = NativeBattleContext {
            elite_trigger: false, boss: true, burning: None, m15: false, m20: false,
        };
        let collar = matrix_combat_with_battle(character, Some(RelicId::SlaversCollar), None,
            EncounterId::Guardian, 10, 10, &[CardId::Defend; 5], 1, Some(collar_battle))
            .public_full().unwrap();
        assert_eq!((collar.player.energy, collar.player.max_energy), (11, 11),
            "SlaversCollar / {character:?}: boss combat adds one master energy");

        let mut ice_cream = matrix_combat(character, Some(RelicId::IceCream), None,
            EncounterId::JawWorm, 10, 10, &[CardId::Defend; 5], 1);
        let initial = ice_cream.public_full().unwrap();
        assert_eq!((initial.player.energy, initial.player.max_energy), (10, 10),
            "IceCream / {character:?}: no initial master-energy bonus");
        let first = ice_cream.legal_actions_full().unwrap().into_iter()
            .find(|action| matches!(action, CombatAction::PlayCard { .. })).expect("first Defend is legal");
        ice_cream.step_full(&first).unwrap();
        ice_cream.step_full(&CombatAction::EndTurn).unwrap();
        resolve_public_choices(&mut ice_cream, &mut Vec::new(), "IceCream recharge");
        let next = ice_cream.public_full().unwrap();
        assert_eq!((next.player.energy, next.player.max_energy), (19, 10),
            "IceCream / {character:?}: nine unspent energy is added to the next 10");

        let mut capped = matrix_combat(character, Some(RelicId::IceCream), None,
            EncounterId::JawWorm, 999, 999, &[CardId::Defend; 5], 1);
        let first = capped.legal_actions_full().unwrap().into_iter()
            .find(|action| matches!(action, CombatAction::PlayCard { .. })).expect("capped Defend is legal");
        capped.step_full(&first).unwrap();
        capped.step_full(&CombatAction::EndTurn).unwrap();
        resolve_public_choices(&mut capped, &mut Vec::new(), "IceCream 999 cap");
        let capped = capped.public_full().unwrap();
        assert_eq!((capped.player.energy, capped.player.max_energy), (999, 999),
            "IceCream / {character:?}: carried energy uses the documented 999 cap");
    }
}

#[test]
fn m27_cross_character_prismatic_shard_orb_slots() {
    // Design Appendix A, PrismaticShard: one master slot if the character has none; Defect retains its three.
    for (character, expected) in [
        (Character::Ironclad, 1), (Character::Silent, 1),
        (Character::Defect, 3), (Character::Watcher, 1),
    ] {
        let public = matrix_combat(character, Some(RelicId::PrismaticShard), None,
            EncounterId::JawWorm, 10, 10, &[CardId::Defend; 5], 1).public_full().unwrap();
        assert_eq!(public.character.orbs.len(), expected,
            "PrismaticShard / {character:?}: literal master orb-slot count");
    }
}

#[test]
fn m27_cross_character_card_generator_pools() {
    // M26 pools.rs: Combat Any/Power are the player's color, Common→Rare, excluding HEALING;
    // F14 rulings specify Dead Branch, Enchiridion and Nilry, and Toolbox is ordinary colorless.
    for character in CHARACTERS {
        let enchiridion = matrix_combat(character, Some(RelicId::Enchiridion), None,
            EncounterId::JawWorm, 10, 10, &[CardId::Defend; 5], 1).public_full().unwrap();
        let generated = enchiridion.hand.iter().find(|card| card.id != CardId::Defend)
            .unwrap_or_else(|| panic!("Enchiridion / {character:?}: generated Power is in public hand"));
        assert_character_card(character, "Enchiridion", generated.id, Some(CardType::Power), true);

        let toolbox = matrix_combat(character, Some(RelicId::Toolbox), None,
            EncounterId::JawWorm, 10, 10, &[CardId::Defend; 5], 1);
        let offer = toolbox.public_full().unwrap().card_offer
            .unwrap_or_else(|| panic!("Toolbox / {character:?}: three public colorless offers before draw"));
        assert_eq!(offer.cards.len(), 3, "Toolbox / {character:?}: three choices");
        let mut unique = offer.cards.clone(); unique.sort_by_key(|id| id.as_str()); unique.dedup();
        assert_eq!(unique.len(), 3, "Toolbox / {character:?}: three distinct ids");
        for id in &offer.cards {
            let row = content::Catalog::standard().card(*id).expect("Toolbox card is catalogued");
            assert_eq!(row.color, CardColor::Colorless, "Toolbox / {character:?}: {id:?} is literal colorless");
            assert!(matches!(row.rarity, CardRarity::Common | CardRarity::Uncommon | CardRarity::Rare));
            assert_ne!(*id, CardId::BandageUp, "M26 colorless combat pool excludes healing Bandage Up");
        }

        let codex = play_matrix_script(character, Some(RelicId::NilrysCodex), None,
            EncounterId::JawWorm, 10, 10, &[CardId::Defend; 5], 1, "NilrysCodex");
        let offer = codex.offers.first().unwrap_or_else(||
            panic!("NilrysCodex / {character:?}: end-turn public card offer"));
        assert_eq!(offer.len(), 3, "NilrysCodex / {character:?}: three random cards");
        let mut unique = offer.clone(); unique.sort_by_key(|id| id.as_str()); unique.dedup();
        assert_eq!(unique.len(), 3, "NilrysCodex / {character:?}: distinct cards");
        for id in offer { assert_character_card(character, "NilrysCodex", *id, None, true); }

        // Pummel is the supplied exhaust trigger; Jaw Worm survives the one attack (design §15.3).
        let mut dead_branch = matrix_combat(character, Some(RelicId::DeadBranch), None,
            EncounterId::JawWorm, 10, 10,
            &[CardId::Pummel, CardId::Defend, CardId::Defend, CardId::Defend, CardId::Defend], 5);
        let pummel = dead_branch.public_full().unwrap().hand.iter().find(|card| card.id == CardId::Pummel)
            .unwrap_or_else(|| panic!("DeadBranch / {character:?}: supplied Pummel is in the public hand")).instance.0;
        let pummel_action = dead_branch.legal_actions_full().unwrap().into_iter()
            .find(|action| matches!(action, CombatAction::PlayCard { instance, .. } if *instance == pummel))
            .unwrap_or_else(|| panic!("DeadBranch / {character:?}: supplied Pummel is legal"));
        dead_branch.step_full(&pummel_action)
            .unwrap_or_else(|error| panic!("DeadBranch / {character:?}: play Pummel: {error:?}"));
        let public = dead_branch.public_full().unwrap();
        let generated = public.hand.iter().find(|card| card.id != CardId::Defend)
            .unwrap_or_else(|| panic!("DeadBranch / {character:?}: generated card is in public hand"));
        assert_character_card(character, "DeadBranch", generated.id, None, true);
    }
}

#[test]
fn m27_cross_character_run_card_generator_pools() {
    // Design §7.1 and accepted selection tests: Orrery makes five offers; Tiny House makes one.
    // Equip offers use public Run decisions, and their literal class pool is M26 content/pools.rs.
    for character in CHARACTERS {
        let mut orrery = equip_offer_run(character, RelicId::Orrery);
        let reward_count = public_room(&orrery, "Orrery").choices.iter()
            .filter(|choice| choice.kind == run::RoomChoiceKind::CardReward).count();
        assert_eq!(reward_count, 5, "Orrery / {character:?}: five separate public card rewards");
        for _ in 0..5 {
            choose_public_room(&mut orrery, "Orrery", |kind| *kind == run::RoomChoiceKind::CardReward);
            let child = public_room(&orrery, "Orrery");
            assert_eq!(child.kind, run::RoomViewKind::CardReward, "Orrery / {character:?}: public card reward");
            let cards = child.choices.iter().filter_map(|choice| match choice.kind {
                run::RoomChoiceKind::Card { id, .. } => Some(id), _ => None,
            }).collect::<Vec<_>>();
            assert!(!cards.is_empty(), "Orrery / {character:?}: generated public card choices");
            for id in cards { assert_character_card(character, "Orrery", id, None, false); }
            choose_public_room(&mut orrery, "Orrery", |kind| *kind == run::RoomChoiceKind::Skip);
        }

        let mut tiny_house = equip_offer_run(character, RelicId::TinyHouse);
        assert_eq!(public_room(&tiny_house, "TinyHouse").choices.iter()
            .filter(|choice| choice.kind == run::RoomChoiceKind::CardReward).count(), 1,
            "TinyHouse / {character:?}: one ordinary public card reward");
        choose_public_room(&mut tiny_house, "TinyHouse", |kind| *kind == run::RoomChoiceKind::CardReward);
        let child = public_room(&tiny_house, "TinyHouse");
        let cards = child.choices.iter().filter_map(|choice| match choice.kind {
            run::RoomChoiceKind::Card { id, .. } => Some(id), _ => None,
        }).collect::<Vec<_>>();
        assert!(!cards.is_empty(), "TinyHouse / {character:?}: generated public card choices");
        for id in cards { assert_character_card(character, "TinyHouse", id, None, false); }
        choose_public_room(&mut tiny_house, "TinyHouse", |kind| *kind == run::RoomChoiceKind::Skip);
        let potion = public_room(&tiny_house, "TinyHouse").choices.into_iter().find_map(|choice|
            if choice.status == run::RoomChoiceStatus::Offered {
                if let run::RoomChoiceKind::Potion(id) = choice.kind { Some(id) } else { None }
            } else { None })
            .unwrap_or_else(|| panic!("TinyHouse / {character:?}: public flat-pool potion offer"));
        assert_character_potion(character, "TinyHouse", potion);
    }
}

#[test]
fn m27_cross_character_cauldron_potion_offer_pools() {
    // Design §7.1 and accepted m27_relic_selections.rs:378-396: five flat offers from the
    // literal 33-potion character union. Potion Belt supplies five public claim slots.
    for character in CHARACTERS {
        let cauldron = equip_offer_run(character, RelicId::Cauldron);
        let room = public_room(&cauldron, "Cauldron");
        let potions = room.choices.iter().filter_map(|choice| match choice.kind {
            run::RoomChoiceKind::Potion(id) => Some(id), _ => None,
        }).collect::<Vec<_>>();
        assert_eq!(potions.len(), 5, "Cauldron / {character:?}: five generated potion offers");
        for id in potions { assert_character_potion(character, "Cauldron", id); }
    }
}

#[test]
fn m27_cross_character_white_beast_statue_potion_offer_pool() {
    // Design §8.5: White Beast Statue makes the shared rarity-first reward appearance 100%.
    // The actual offer id returned by the reward owner must be in Appendix B's literal class union.
    for character in CHARACTERS {
        let mut run = items::tests::m27_support::complete_run(
            character, vec![RelicId::WhiteBeastStatue], vec![None; 3], BTreeMap::new(), EncounterId::Cultist,
        );
        assert!(run.decision().expect("WhiteBeastStatue Run public view")
            .player.relics.contains(&RelicId::WhiteBeastStatue));
        let offer = items::potion_offers::combat_appearance(
            run.state_mut_for_test(),
            items::potion_offers::RewardRoom::Ordinary { enemy_escaped: false },
            0,
            &[1, 1, 0, 1],
        ).unwrap_or_else(|error| panic!("WhiteBeastStatue / {character:?}: reward offer: {error:?}"))
            .unwrap_or_else(|| panic!("WhiteBeastStatue / {character:?}: 100% offer is present"));
        assert_character_potion(character, "WhiteBeastStatue", offer);
    }
}

#[test]
fn m27_cross_character_potion_generator_pools() {
    // Literal class-potion rows from design §4.3 / Appendix B; never derive expected membership
    // from potion_generation::candidates, which is the implementation under test.
    for character in CHARACTERS {
        let mut brew = matrix_combat(character, None, Some(PotionId::EntropicBrew),
            EncounterId::JawWorm, 10, 10, &[CardId::Defend; 5], 1);
        let actions = brew.legal_actions_full().unwrap();
        let action = actions.into_iter().find(|action| matches!(action,
            CombatAction::UsePotion { slot: 0, .. })).expect("Entropic Brew is a legal combat potion");
        brew.step_full(&action).unwrap_or_else(|error| panic!("EntropicBrew / {character:?}: {error:?}"));
        resolve_public_choices(&mut brew, &mut Vec::new(), "EntropicBrew");
        let public = brew.public_full().unwrap();
        let generated = public.inventory.as_ref().unwrap().potions.iter().flatten()
            .copied().collect::<Vec<_>>();
        assert!(!generated.is_empty(), "EntropicBrew / {character:?}: fills a public empty potion slot");
        for id in generated { assert_character_potion(character, "EntropicBrew", id); }

        // Class-card potions are character-sensitive within the shared potion union; M26
        // Combat pools are the owning class color, ordinary rarity, and the requested type.
        for (potion_id, item, card_type) in [
            (PotionId::AttackPotion, "AttackPotion", CardType::Attack),
            (PotionId::SkillPotion, "SkillPotion", CardType::Skill),
            (PotionId::PowerPotion, "PowerPotion", CardType::Power),
        ] {
            let mut potion = matrix_combat(character, None, Some(potion_id),
                EncounterId::JawWorm, 10, 10, &[CardId::Defend; 5], 1);
            let action = potion.legal_actions_full().unwrap().into_iter()
                .find(|action| matches!(action, CombatAction::UsePotion { slot: 0, .. }))
                .unwrap_or_else(|| panic!("{item} / {character:?}: potion is a legal combat action"));
            potion.step_full(&action).unwrap_or_else(|error| panic!("{item} / {character:?}: {error:?}"));
            let offer = potion.public_full().unwrap().card_offer
                .unwrap_or_else(|| panic!("{item} / {character:?}: public generated card choices"));
            assert_eq!(offer.cards.len(), 3, "{item} / {character:?}: three options");
            for id in offer.cards {
                assert_character_card(character, item, id, Some(card_type), true);
            }
        }
    }
}
