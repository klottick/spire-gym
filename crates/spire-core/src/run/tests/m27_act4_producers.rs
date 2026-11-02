//! M27 complete-admission Act 4 relic producers and diagnostics.
use super::super::*;
use crate::{
    combat::{Actor, CombatAction, DamagePayload, Effect, EffectKind, Outcome, Source},
    content::{Character, EncounterId, RelicId, RelicTier},
    items::{dispatch::RunItemEvent, relic_pools::{self, RelicPools}, run_laws::RewardRoom},
    run::{diagnostics::{self, CompleteItemSupply, M25Diagnostic}},
};
use std::collections::BTreeMap;

fn supply(relics: Vec<RelicId>) -> CompleteItemSupply {
    CompleteItemSupply { relics, potions: vec![None; 3], counters: BTreeMap::new() }
}

fn move_to_floor(run: &mut Run, floor: u32) {
    let target = run.state.map.as_ref().unwrap().nodes.iter().find(|node| node.floor == floor).unwrap().id;
    run.step(&GameAction::Move(target)).unwrap();
}

fn choose_kind(run: &mut Run, predicate: impl Fn(&RoomChoiceKind) -> bool) {
    let id = run.decision().unwrap().room.unwrap().choices.into_iter()
        .find(|row| row.status == RoomChoiceStatus::Offered && predicate(&row.kind))
        .unwrap_or_else(|| panic!("offered room choice matching predicate")).id;
    run.step(&GameAction::Choose(id)).unwrap();
}

fn enter_shop(run: &mut Run) {
    move_to_floor(run, 51);
    assert_eq!(run.decision().unwrap().room.unwrap().kind, RoomViewKind::Campfire);
    choose_kind(run, |kind| matches!(kind, RoomChoiceKind::Rest { .. }));
    move_to_floor(run, 52);
    assert_eq!(run.decision().unwrap().room.unwrap().kind, RoomViewKind::Shop);
}

fn elite_pools() -> RelicPools {
    relic_pools::explicit_for_test(
        vec![RelicId::Anchor, RelicId::BloodVial],
        vec![RelicId::Kunai],
        vec![RelicId::Mango],
        vec![RelicId::MembershipCard],
        vec![],
    )
}

fn defeat_current_combat(run: &mut Run) {
    let ids = run.state.combat.as_ref().unwrap().public_full().unwrap().enemies.into_iter()
        .filter(|enemy| enemy.hp > 0).map(|enemy| enemy.instance_id).collect::<Vec<_>>();
    for id in ids {
        run.state.combat.as_mut().unwrap().enqueue_native_effects(vec![Effect {
            source: Source::Actor(Actor::Player), target: Some(Actor::Enemy(id)),
            kind: EffectKind::Damage(DamagePayload::Fixed(10_000)),
        }], None).unwrap();
    }
    assert_eq!(run.state.combat.as_ref().unwrap().public_full().unwrap().outcome, Some(Outcome::Victory));
}

fn enter_elite(run: &mut Run) {
    enter_shop(run);
    choose_kind(run, |kind| *kind == RoomChoiceKind::Leave);
    move_to_floor(run, 53);
}

fn play_strike(run: &mut Run) {
    let decision=run.decision().unwrap();
    let instance=decision.combat.as_ref().unwrap().hand.iter().find(|card|card.id==crate::content::CardId::Strike)
        .unwrap_or_else(||panic!("Strike is in the current hand: {:?}",decision.combat.as_ref().unwrap().hand)).instance.0;
    let action=decision.candidates.into_iter().find(|candidate|matches!(candidate.action,
        GameAction::Combat(CombatAction::PlayCard{instance:found,..}) if found==instance))
        .unwrap_or_else(||panic!("Strike {instance} has an offered play action")).action;
    run.step(&action).unwrap_or_else(|error|panic!("play Strike: {error:?}"));
}

#[test]
fn m27_diagnostics_complete_act4_run() {
    for character in [Character::Ironclad, Character::Silent, Character::Defect, Character::Watcher] {
        let mut supplied = supply(vec![RelicId::PenNib]);
        supplied.potions[0] = Some(crate::content::PotionId::FirePotion);
        supplied.counters.insert(RelicId::PenNib, 7);
        let mut run = diagnostics::complete_act4_run(character, supplied)
            .unwrap_or_else(|error| panic!("complete {character:?} Act 4 diagnostic: {error:?}"));
        assert_eq!((run.state.character, run.state.act, run.state.floor), (character, 4, 50));
        assert_eq!(run.state.card_admission, CardAdmission::Complete);
        assert_eq!(run.state.item_admission, ItemAdmission::Complete);
        assert!(run.state.relic_pools.is_some());
        assert_eq!(run.state.map.as_ref().unwrap().nodes.iter().map(|node| node.floor).collect::<Vec<_>>(), vec![51, 52, 53, 54]);
        move_to_floor(&mut run, 51);
        assert_eq!(run.state.floor, 51, "{character:?} reaches the floor-51 campfire");
        assert_eq!(run.decision().unwrap().room.unwrap().kind, RoomViewKind::Campfire);
        assert_eq!(run.state.relics, vec![RelicId::PenNib]);
        assert_eq!(run.state.potions, vec![Some(crate::content::PotionId::FirePotion), None, None]);
        assert_eq!(run.state.relic_counters.get(&RelicId::PenNib), Some(&7));
    }
    assert_eq!(diagnostics::complete_act4_run(Character::Ironclad, supply(vec![RelicId::RingOfTheSnake])).unwrap_err(), RunError::InvalidAction,
        "an off-class supplied relic is rejected before the constructor returns a Run");
}

#[test]
fn m27_act4_relic_producers_under_complete() {
    for character in [Character::Ironclad, Character::Silent, Character::Defect, Character::Watcher] {
        let mut run = diagnostics::complete_act4_run(character, supply(vec![])).unwrap();
        run.state.relic_pools = Some(elite_pools());
        enter_shop(&mut run);
        let shop = run.decision().unwrap().room.unwrap();
        let relic_rows = shop.choices.iter().filter(|row| matches!(row.kind, RoomChoiceKind::Relic(_))).collect::<Vec<_>>();
        assert_eq!(relic_rows.len(), 1, "{character:?} complete Act 4 shop has one relic row");
        assert_eq!(relic_rows[0].kind, RoomChoiceKind::Relic(RelicId::MembershipCard), "Shop-tier source selects its remaining eligible back row");
        assert_eq!(relic_rows[0].price, Some(150), "Shop-tier base price");
        assert!(!run.state.relic_pools.as_ref().unwrap().shop.contains(&RelicId::MembershipCard), "the offered row is consumed");

        let mut courier = diagnostics::complete_act4_run(character, supply(vec![RelicId::TheCourier])).unwrap();
        courier.state.relic_pools = Some(elite_pools());
        enter_shop(&mut courier);
        let initial = courier.decision().unwrap().room.unwrap().choices.into_iter()
            .find(|row| row.status == RoomChoiceStatus::Offered && matches!(row.kind, RoomChoiceKind::Relic(_))).unwrap();
        courier.step(&GameAction::Choose(initial.id)).unwrap();
        let replacement = courier.decision().unwrap().room.unwrap().choices.into_iter()
            .find(|row| row.status == RoomChoiceStatus::Offered && matches!(row.kind, RoomChoiceKind::Relic(_)))
            .unwrap_or_else(|| panic!("{character:?} The Courier adds an offered relic replacement"));
        let RoomChoiceKind::Relic(replacement_id) = replacement.kind else { unreachable!() };
        assert!(![RelicId::OldCoin, RelicId::SmilingMask, RelicId::MawBank, RelicId::TheCourier].contains(&replacement_id));
        // Main-4 amendment: the first purchase is Membership Card, so the replacement takes Courier ×0.8 then
        // Membership ×0.5, separately rounded (economy research §3.3 relic replacement price): 150/250/300 → 60/100/120.
        let price = match crate::content::Catalog::standard().relic_tier(replacement_id).unwrap() {
            RelicTier::Common => 60, RelicTier::Uncommon => 100, RelicTier::Rare => 120,
            tier => panic!("merchant replacement has a non-ordinary tier: {tier:?}"),
        };
        assert_eq!(replacement.price, Some(price), "replacement uses its merchant tier base after both owned discounts");
        assert!(!courier.state.relic_pools.as_ref().unwrap().common.contains(&replacement_id)
            && !courier.state.relic_pools.as_ref().unwrap().uncommon.contains(&replacement_id)
            && !courier.state.relic_pools.as_ref().unwrap().rare.contains(&replacement_id), "replacement is consumed");

        choose_kind(&mut run, |kind| *kind == RoomChoiceKind::Leave);
        run.state.relic_pools = Some(elite_pools());
        move_to_floor(&mut run, 53);
        assert_eq!(run.state.active_combat.as_ref().unwrap().encounter, EncounterId::ShieldAndSpear);
        defeat_current_combat(&mut run);
        let active = run.state.active_combat.as_ref().unwrap();
        let record = CombatRecord { visit: active.visit, encounter: active.encounter, outcome: Outcome::Victory };
        let building = RunItemEvent::RewardsBuilding { room: RewardRoom::Elite, card_choices: 3,
            extra_card_rewards: 0, gold_percent: 0, extra_elite_relics: 1, include_potion: true, singing_bowl: false };
        let prepared = rooms::rewards::prepare_combat_with_building(&run, &record, building).unwrap().unwrap();
        let preview_relics = prepared.screen.choices.iter().filter_map(|row| match row.kind {
            RoomChoiceKind::Relic(id) => Some(id), _ => None,
        }).collect::<Vec<_>>();
        assert_eq!(preview_relics.len(), 2, "one elite relic plus one injected Black Star producer row");
        assert!(![RelicId::Girya, RelicId::PeacePipe, RelicId::Shovel].contains(&preview_relics[1]));
        let main_tier = crate::content::Catalog::standard().relic_tier(preview_relics[0]).unwrap();
        let expected_front = match main_tier {
            RelicTier::Common => RelicId::Anchor, RelicTier::Uncommon => RelicId::Kunai, RelicTier::Rare => RelicId::Mango,
            tier => panic!("ordinary elite roll has non-ordinary tier {tier:?}"),
        };
        assert_eq!(preview_relics[0], expected_front, "elite relic is selected from its rolled tier front");
        run.settle_combat().unwrap();
        assert!(run.state.relic_pools.as_ref().unwrap().common.iter().chain(&run.state.relic_pools.as_ref().unwrap().uncommon)
            .chain(&run.state.relic_pools.as_ref().unwrap().rare).all(|id| *id != preview_relics[0]), "elite relic is consumed on settlement");
    }
}

#[test]
fn m27_membership_card_purchase_discounts_current_shop() {
    // Main-4: buying Membership Card runs its 0.5 pass over every current priced row at once (rounded), and the
    // removal price is recomputed from its base (economy research §3.2: base 75 -> 38 with Membership Card).
    let mut run = diagnostics::complete_act4_run(Character::Ironclad, supply(vec![])).unwrap();
    run.state.relic_pools = Some(elite_pools());
    enter_shop(&mut run);
    let before = run.decision().unwrap().room.unwrap().choices;
    let membership = before.iter().find(|row| row.kind == RoomChoiceKind::Relic(RelicId::MembershipCard)).unwrap().id;
    assert_eq!(before.iter().find(|row| row.kind == RoomChoiceKind::Purge).unwrap().price, Some(75));
    run.step(&GameAction::Choose(membership)).unwrap();
    let after = run.decision().unwrap().room.unwrap().choices;
    assert_eq!(after.iter().find(|row| row.kind == RoomChoiceKind::Purge).unwrap().price, Some(38));
    for row in before.iter().filter(|row| matches!(row.kind, RoomChoiceKind::Card { .. } | RoomChoiceKind::Potion(_))) {
        let old = row.price.unwrap();
        let new = after.iter().find(|next| next.id == row.id).unwrap().price.unwrap();
        assert_eq!(new, (old + 1) / 2, "{:?}: half of {old}, rounded half up", row.kind);
    }
}

fn assert_no_curse(cards: &[crate::content::CardId], context: &str) {
    for id in cards {
        let card = crate::content::Catalog::standard().card(*id).unwrap();
        assert_ne!(card.card_type, crate::content::CardType::Curse, "{context}: shop offers Curse {id:?}");
    }
}
fn shop_card_ids(run: &Run) -> Vec<crate::content::CardId> {
    run.decision().unwrap().room.unwrap().choices.iter().filter_map(|row| match row.kind {
        RoomChoiceKind::Card { id, .. } => Some(id), _ => None,
    }).collect()
}

#[test]
fn m27_shop_card_offers_never_contain_a_curse() {
    // User correction (2026-11-01): the merchant sells coloured and colourless cards only; its one curse
    // interaction is removal. Every shop card producer, per character and act, frozen and complete.
    use crate::content::CardRarity::{Common, Rare, Uncommon};
    use crate::run::rooms::{m10_content, m12_content, m15_content, m17_content, m19_content, m20_content, m22_content, m25_content};
    for later in [false, true] {
        assert_no_curse(&m12_content::shop_cards(later), "Silent Acts 1-3 (M12-M14)");
        assert_no_curse(&m17_content::shop_cards(later), "Defect Acts 1-2 (M17-M18)");
        assert_no_curse(&m19_content::shop_cards(later), "Defect Act 3 (M19)");
        assert_no_curse(&m22_content::shop_cards(later), "Watcher Acts 1-3 (M22-M24)");
    }
    assert_no_curse(&[crate::content::CardId::IronWave, crate::content::CardId::Clothesline], "captured Ironclad Acts 1-3");
    let samplers: [(&str, fn(&mut RunRng, &[u64], &[crate::content::CardRarity]) -> Result<Vec<crate::content::CardId>, RunError>); 4] = [
        ("Ironclad M10", m10_content::sample_capture_row), ("Silent M15", m15_content::sample_capture_row),
        ("Defect M20", m20_content::sample_capture_row), ("Watcher M25", m25_content::sample_capture_row),
    ];
    for (name, sample) in samplers {
        let mut rng = RunRng::new([0x27; 32], crate::combat::rng::Partition::Test, 0);
        for act in 1..=4_u64 { for node in 0..64_u64 {
            assert_no_curse(&sample(&mut rng, &[act, 10 + act, node], &[Common, Uncommon, Rare]).unwrap(), name);
        } }
    }
    let frozen = diagnostics::m25_diagnostic(M25Diagnostic::Act4Room { floor: 52 }).unwrap();
    assert_no_curse(&shop_card_ids(&frozen), "frozen M25 Act 4 shop");
    for character in [Character::Ironclad, Character::Silent, Character::Defect, Character::Watcher] {
        let mut run = diagnostics::complete_act4_run(character, supply(vec![RelicId::TheCourier])).unwrap();
        run.state.relic_pools = Some(elite_pools());
        run.state.gold = 999;
        enter_shop(&mut run);
        let cards = shop_card_ids(&run);
        assert!(!cards.is_empty());
        assert_no_curse(&cards, "complete Act 4 shop");
        let first = run.decision().unwrap().room.unwrap().choices.into_iter()
            .find(|row| matches!(row.kind, RoomChoiceKind::Card { .. }) && row.status == RoomChoiceStatus::Offered).unwrap().id;
        run.step(&GameAction::Choose(first)).unwrap();
        assert_no_curse(&shop_card_ids(&run), "complete Act 4 shop after The Courier's card replacement");
    }
}

#[test]
fn m27_act4_frozen_profile_relic_rows_remain_accepted() {
    let shop = diagnostics::m25_diagnostic(M25Diagnostic::Act4Room { floor: 52 }).unwrap();
    let shop_relics = shop.decision().unwrap().room.unwrap().choices.into_iter()
        .filter_map(|row| if let RoomChoiceKind::Relic(id) = row.kind { Some((id, row.price)) } else { None }).collect::<Vec<_>>();
    assert_eq!(shop_relics, vec![(RelicId::Melange, Some(150))]);

    let mut elite = diagnostics::m25_diagnostic(M25Diagnostic::Act4Room { floor: 53 }).unwrap();
    defeat_current_combat(&mut elite);
    elite.settle_combat().unwrap();
    let elite_relics = elite.decision().unwrap().room.unwrap().choices.into_iter()
        .filter_map(|row| if let RoomChoiceKind::Relic(id) = row.kind { Some(id) } else { None }).collect::<Vec<_>>();
    assert_eq!(elite_relics, vec![RelicId::Duality]);
}


#[test]
fn m27_counters_persist_through_every_combat_end() {
    let mut pen_nib=supply(vec![RelicId::PenNib]);
    pen_nib.counters.insert(RelicId::PenNib,9);
    let mut victory=diagnostics::complete_act4_run(Character::Ironclad,pen_nib.clone()).unwrap();
    enter_elite(&mut victory);
    assert_eq!(victory.state.combat.as_ref().unwrap().item_context().unwrap().counter(RelicId::PenNib),9);
    defeat_current_combat(&mut victory);
    victory.settle_combat().unwrap();
    assert_eq!(victory.state.relic_counters.get(&RelicId::PenNib),Some(&9),"Victory returns the persistent counter");
    choose_kind(&mut victory,|kind|*kind==RoomChoiceKind::Proceed);
    move_to_floor(&mut victory,54);
    assert_eq!(victory.state.combat.as_ref().unwrap().item_context().unwrap().counter(RelicId::PenNib),9,
        "the next combat receives the saved counter");
    crate::combat::tests::m27_state::set_player_energy(victory.state.combat.as_mut().unwrap(),999);
    crate::combat::tests::m27_state::set_enemy_block(victory.state.combat.as_mut().unwrap(),0,0);
    let before=victory.state.combat.as_ref().unwrap().public_full().unwrap().enemies[0].hp;
    play_strike(&mut victory);
    let after=victory.state.combat.as_ref().unwrap().public_full().unwrap().enemies[0].hp;
    assert_eq!(before-after,12,"the first Attack in combat two is doubled");
    assert_eq!(victory.state.combat.as_ref().unwrap().item_context().unwrap().counter(RelicId::PenNib),0,
        "the 10th Attack resets Pen Nib");

    let mut death_supply=supply(vec![RelicId::PenNib]);
    death_supply.counters.insert(RelicId::PenNib,9);
    let mut death=diagnostics::complete_act4_run(Character::Ironclad,death_supply).unwrap();
    enter_elite(&mut death);
    let hp=death.state.combat.as_ref().unwrap().public_full().unwrap().player.hp;
    death.state.combat.as_mut().unwrap().enqueue_native_effects(vec![Effect {
        source:Source::None,target:Some(Actor::Player),kind:EffectKind::Damage(DamagePayload::HpLoss(hp)),
    }],None).unwrap();
    assert_eq!(death.state.combat.as_ref().unwrap().public_full().unwrap().outcome,Some(Outcome::Death));
    death.settle_combat().unwrap_or_else(|error|panic!("Death settles without fault: {error:?}"));
    assert_eq!(death.state.terminal,Some(RunTerminal::Death));
    assert_eq!(death.state.fault,None);
    assert_eq!(death.state.combat_progress,CombatProgress{entered:2,settled:2},"the seeded Act3 source and one Act4 death settle once each");
    assert_eq!(death.state.relics,vec![RelicId::PenNib],"Death returns inventory exactly once");
    assert_eq!(death.state.relic_counters.get(&RelicId::PenNib),Some(&9));
    assert!(death.state.combat.is_none()&&death.state.active_combat.is_none());
    let local=supply(vec![RelicId::Kunai,RelicId::CentennialPuzzle]);
    let mut run=diagnostics::complete_act4_run(Character::Ironclad,local).unwrap();
    enter_elite(&mut run);
    // Seed nonzero in-combat values so settlement verifies that only persistent counters return.
    let mut items=run.state.combat.as_ref().unwrap().item_context().unwrap().clone();
    items.set_local(RelicId::Kunai,2);
    items.set_local(RelicId::CentennialPuzzle,1);
    crate::combat::tests::m27_state::set_item_context(run.state.combat.as_mut().unwrap(),Some(items));
    assert_eq!(run.state.combat.as_ref().unwrap().item_context().unwrap().local(RelicId::Kunai),2);
    assert_eq!(run.state.combat.as_ref().unwrap().item_context().unwrap().local(RelicId::CentennialPuzzle),1);
    defeat_current_combat(&mut run);
    run.settle_combat().unwrap();
    assert_eq!(run.state.relics,vec![RelicId::Kunai,RelicId::CentennialPuzzle]);
    assert!(!run.state.relic_counters.contains_key(&RelicId::Kunai));
    assert!(!run.state.relic_counters.contains_key(&RelicId::CentennialPuzzle));
}

#[test]
fn m27_act4_smoke_bomb_blocked_by_back_attack() {
    // Source law: Smoke Bomb is unusable while any monster has Back Attack; Act 4 Shield and Spear always has one,
    // so no Act 4 combat can escape. Escape counter persistence is covered in combat/tests/m27_rescue_escape.rs.
    let mut resources=supply(vec![RelicId::PenNib]);
    resources.counters.insert(RelicId::PenNib,9);
    resources.potions=vec![Some(crate::content::PotionId::SmokeBomb),None,None];
    let mut run=diagnostics::complete_act4_run(Character::Ironclad,resources).unwrap();
    enter_elite(&mut run);
    let frozen=run.state.combat.as_ref().unwrap().snapshot();
    assert!(matches!(run.step(&GameAction::Combat(CombatAction::UsePotion{slot:0,target:None})),Err(RunError::InvalidAction)));
    assert_eq!(run.state.combat.as_ref().unwrap().snapshot(),frozen,"the rejected use keeps the potion and combat state");
    assert_eq!(run.state.combat.as_ref().unwrap().item_context().unwrap().counter(RelicId::PenNib),9);
}
