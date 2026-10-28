//! Supplied public-decision tests for the finite M14 acquisition policy.
//!
//! These call the shared parent dispatcher with an M14 Caller. They do not
//! instantiate a Run, route a seed, or claim an outcome.
use super::*;
use spire_core::{
    content::{CardType, EventId},
    run::{
        Candidate, ChoiceId, CombatProgress, GameAction, PublicDecision, PublicObject,
        PublicPlayer, PublicSelection, RoomChoiceStatus, RoomChoiceView, RoomView,
        RoomViewKind, RunTerminal, SelectionId,
    },
};

type Choice = (u64, RoomChoiceKind, Option<u32>, RoomChoiceStatus);

fn decision(
    room_kind: RoomViewKind,
    cards: &[(u64, CardId, u32)],
    choices: &[Choice],
    gold: u32,
    potions: Vec<Option<PotionId>>,
) -> PublicDecision {
    let room_choices = choices
        .iter()
        .map(|(id, kind, price, status)| RoomChoiceView {
            id: ChoiceId(*id),
            kind: kind.clone(),
            price: *price,
            status: *status,
        })
        .collect::<Vec<_>>();
    let candidates = choices
        .iter()
        .filter(|(_, _, _, status)| *status == RoomChoiceStatus::Offered)
        .map(|(id, _, _, _)| Candidate {
            id: ChoiceId(*id),
            action: GameAction::Choose(ChoiceId(*id)),
            label: "supplied offered row",
            neow_offer: None,
            object: None,
        })
        .collect();
    PublicDecision {
        content_profile: Some(RoomScenario::M14SilentAct3),
        room: Some(RoomView { kind: room_kind, choices: room_choices }),
        reward_total: 0,
        act: 3,
        floor: 40,
        player: PublicPlayer {
            relic_counters: Vec::new(),
            character: Character::Silent,
            hp: 50,
            max_hp: 70,
            gold,
            relics: Vec::new(),
            potions,
            keys: [false; 3],
        },
        combat: None,
        combat_progress: CombatProgress::default(),
        last_combat: None,
        objects: cards
            .iter()
            .map(|(instance, id, upgrades)| PublicObject { misc:0, instance: *instance, id: *id, upgrades: *upgrades })
            .collect(),
        candidates,
        offers: Vec::new(),
        selection: None,
        terminal: Option::<RunTerminal>::None,
    }
}

fn decide(decision: &PublicDecision) -> GameAction {
    action_for(decision, None, &Caller::new_m14(M14_ACTION_LIMIT_MAX).unwrap()).unwrap()
}

fn offered(id: u64, kind: RoomChoiceKind, price: Option<u32>) -> Choice {
    (id, kind, price, RoomChoiceStatus::Offered)
}

fn select_cards(mut decision: PublicDecision, instances: &[u64], required: usize) -> PublicDecision {
    let eligible = decision.objects
        .iter()
        .filter(|card| instances.contains(&card.instance))
        .cloned()
        .collect::<Vec<_>>();
    decision.candidates = eligible
        .iter()
        .map(|card| Candidate {
            id: ChoiceId(card.instance),
            action: GameAction::Select(SelectionId(card.instance)),
            label: "supplied legal selection",
            neow_offer: None,
            object: Some(card.clone()),
        })
        .collect();
    decision.selection = Some(PublicSelection { required, selected_count: 0, eligible });
    decision
}

fn instances(cards: &[(CardId, u32)]) -> Vec<(u64, CardId, u32)> {
    cards.iter().enumerate().map(|(index, (id, upgrades))| (index as u64 + 1, *id, *upgrades)).collect()
}

#[test]
fn m14_revision_reward_prefers_noxious_fumes_when_absent() {
    let cards = instances(&[
        (CardId::StrikeGreen, 0), (CardId::StrikeGreen, 0), (CardId::StrikeGreen, 0),
        (CardId::StrikeGreen, 0), (CardId::StrikeGreen, 0),
        (CardId::DefendGreen, 0), (CardId::DefendGreen, 0), (CardId::DefendGreen, 0),
        (CardId::DefendGreen, 0), (CardId::DefendGreen, 0), (CardId::Survivor, 0),
        (CardId::Neutralize, 0),
    ]);
    let decision = decision(RoomViewKind::CardReward, &cards, &[
        offered(1, RoomChoiceKind::Card { id: CardId::Backflip, upgrades: 0 }, None),
        offered(2, RoomChoiceKind::Card { id: CardId::NoxiousFumes, upgrades: 0 }, None),
    ], 99, vec![None; 3]);
    assert_eq!(decide(&decision), GameAction::Choose(ChoiceId(2)));
}

#[test]
fn m14_revision_short_defense_uses_backflip_before_general_reward_order() {
    let mut owned = vec![(CardId::StrikeGreen, 0); 8];
    owned.extend([(CardId::DefendGreen, 0), (CardId::Survivor, 0)]);
    let cards = instances(&owned);
    let decision = decision(RoomViewKind::CardReward, &cards, &[
        offered(1, RoomChoiceKind::Card { id: CardId::BladeDance, upgrades: 0 }, None),
        offered(2, RoomChoiceKind::Card { id: CardId::Backflip, upgrades: 0 }, None),
    ], 99, vec![None; 3]);
    assert_eq!(decide(&decision), GameAction::Choose(ChoiceId(2)));
}

#[test]
fn m14_revision_defense_ratio_exact_boundary_uses_general_reward_order() {
    let mut owned = vec![(CardId::StrikeGreen, 0); 8];
    owned.extend([(CardId::DefendGreen, 0); 4]);
    let cards = instances(&owned);
    let decision = decision(RoomViewKind::CardReward, &cards, &[
        offered(1, RoomChoiceKind::Card { id: CardId::Backflip, upgrades: 0 }, None),
        offered(2, RoomChoiceKind::Card { id: CardId::BladeDance, upgrades: 0 }, None),
    ], 99, vec![None; 3]);
    assert_eq!(decide(&decision), GameAction::Choose(ChoiceId(2)), "3*D == N is not defense-short");
}

#[test]
fn m14_revision_upgraded_copies_count_toward_caps_and_exhaustion_skips() {
    let owned = [
        (CardId::Backflip, 1), (CardId::Backflip, 0), (CardId::Backflip, 1),
        (CardId::BladeDance, 1), (CardId::BladeDance, 0),
        (CardId::CloakAndDagger, 1), (CardId::CloakAndDagger, 0),
        (CardId::DeadlyPoison, 1), (CardId::DeadlyPoison, 0),
        (CardId::PoisonedStab, 1), (CardId::PoisonedStab, 0),
        (CardId::Deflect, 1), (CardId::Deflect, 0),
        (CardId::DaggerThrow, 1), (CardId::DaggerThrow, 0),
        (CardId::NoxiousFumes, 1), (CardId::Accuracy, 0),
        (CardId::WellLaidPlans, 0), (CardId::QuickSlash, 0),
        (CardId::Catalyst, 0), (CardId::Reflex, 0), (CardId::Tactician, 0),
    ];
    let cards = instances(&owned);
    let decision = decision(RoomViewKind::CardReward, &cards, &[
        offered(1, RoomChoiceKind::Card { id: CardId::NoxiousFumes, upgrades: 0 }, None),
        offered(2, RoomChoiceKind::Card { id: CardId::Accuracy, upgrades: 0 }, None),
        offered(3, RoomChoiceKind::Card { id: CardId::Nightmare, upgrades: 0 }, None),
        offered(4, RoomChoiceKind::Skip, None),
    ], 99, vec![None; 3]);
    assert_eq!(decide(&decision), GameAction::Choose(ChoiceId(4)));
}

#[test]
fn m14_revision_conditional_cards_require_public_synergies() {
    let cards = instances(&[(CardId::StrikeGreen, 0), (CardId::DefendGreen, 0)]);
    for card in [CardId::Accuracy, CardId::Catalyst, CardId::Reflex, CardId::Tactician] {
        let decision = decision(RoomViewKind::CardReward, &cards, &[
            offered(1, RoomChoiceKind::Card { id: card, upgrades: 0 }, None),
            offered(2, RoomChoiceKind::Skip, None),
        ], 99, vec![None; 3]);
        assert_eq!(decide(&decision), GameAction::Choose(ChoiceId(2)), "inadmissible {card:?} must be skipped");
    }
}

#[test]
fn m14_revision_conditional_cards_accept_exact_threshold_and_reject_one_below() {
    let cases: [(CardId, Vec<(CardId, u32)>, Vec<(CardId, u32)>); 4] = [
        (
            CardId::Accuracy,
            vec![
                (CardId::BladeDance, 0), (CardId::CloakAndDagger, 0),
                (CardId::DefendGreen, 0), (CardId::DefendGreen, 0),
            ],
            vec![
                (CardId::BladeDance, 0),
                (CardId::DefendGreen, 0), (CardId::DefendGreen, 0),
            ],
        ),
        (
            CardId::Catalyst,
            vec![
                (CardId::DeadlyPoison, 0), (CardId::PoisonedStab, 0),
                (CardId::DefendGreen, 0), (CardId::DefendGreen, 0),
            ],
            vec![
                (CardId::DeadlyPoison, 0),
                (CardId::DefendGreen, 0), (CardId::DefendGreen, 0),
            ],
        ),
        (
            CardId::Reflex,
            vec![
                (CardId::Survivor, 0), (CardId::DaggerThrow, 0), (CardId::DaggerThrow, 0),
                (CardId::DefendGreen, 0), (CardId::DefendGreen, 0),
            ],
            vec![
                (CardId::Survivor, 0), (CardId::DaggerThrow, 0),
                (CardId::DefendGreen, 0), (CardId::DefendGreen, 0),
            ],
        ),
        (
            CardId::Tactician,
            vec![
                (CardId::Survivor, 0), (CardId::DaggerThrow, 0), (CardId::DaggerThrow, 0),
                (CardId::DefendGreen, 0), (CardId::DefendGreen, 0),
            ],
            vec![
                (CardId::Survivor, 0), (CardId::DaggerThrow, 0),
                (CardId::DefendGreen, 0), (CardId::DefendGreen, 0),
            ],
        ),
    ];

    for (card, enough, one_below) in cases {
        let enough_cards = instances(&enough);
        let accepted = decision(RoomViewKind::CardReward, &enough_cards, &[
            offered(1, RoomChoiceKind::Card { id: card, upgrades: 0 }, None),
            offered(2, RoomChoiceKind::Skip, None),
        ], 99, vec![None; 3]);
        assert_eq!(decide(&accepted), GameAction::Choose(ChoiceId(1)), "accept exact synergy threshold for {card:?}");

        let one_below_cards = instances(&one_below);
        let rejected = decision(RoomViewKind::CardReward, &one_below_cards, &[
            offered(1, RoomChoiceKind::Card { id: card, upgrades: 0 }, None),
            offered(2, RoomChoiceKind::Skip, None),
        ], 99, vec![None; 3]);
        assert_eq!(decide(&rejected), GameAction::Choose(ChoiceId(2)), "reject one below synergy threshold for {card:?}");
    }
}

#[test]
fn m14_revision_reward_ties_use_the_smallest_offered_choice_id() {
    let cards = instances(&[(CardId::StrikeGreen, 0), (CardId::DefendGreen, 0)]);
    let decision = decision(RoomViewKind::CardReward, &cards, &[
        offered(9, RoomChoiceKind::Card { id: CardId::NoxiousFumes, upgrades: 0 }, None),
        offered(2, RoomChoiceKind::Card { id: CardId::NoxiousFumes, upgrades: 0 }, None),
    ], 99, vec![None; 3]);
    assert_eq!(decide(&decision), GameAction::Choose(ChoiceId(2)));
}

#[test]
fn m14_revision_shop_uses_affordable_belt_fitting_potions_before_other_cards() {
    let cards = instances(&[(CardId::StrikeGreen, 0), (CardId::DefendGreen, 0)]);
    let decision = decision(RoomViewKind::Shop, &cards, &[
        offered(1, RoomChoiceKind::Card { id: CardId::QuickSlash, upgrades: 0 }, Some(25)),
        offered(2, RoomChoiceKind::Potion(PotionId::DexterityPotion), Some(60)),
        offered(3, RoomChoiceKind::Potion(PotionId::FirePotion), Some(50)),
    ], 50, vec![None, Some(PotionId::WeakPotion), Some(PotionId::StrengthPotion)]);
    assert_eq!(decide(&decision), GameAction::Choose(ChoiceId(3)), "ignore unaffordable Dex and buy fitting Fire");
}

#[test]
fn m14_revision_shop_skips_full_belt_and_owned_relics() {
    let cards = instances(&[(CardId::StrikeGreen, 0), (CardId::DefendGreen, 0)]);
    let mut decision = decision(RoomViewKind::Shop, &cards, &[
        offered(1, RoomChoiceKind::Potion(PotionId::DexterityPotion), Some(50)),
        offered(2, RoomChoiceKind::Relic(RelicId::SneckoSkull), Some(150)),
        offered(3, RoomChoiceKind::Relic(RelicId::BagOfPreparation), Some(150)),
        offered(4, RoomChoiceKind::Card { id: CardId::QuickSlash, upgrades: 0 }, Some(50)),
    ], 150, vec![Some(PotionId::FirePotion), Some(PotionId::WeakPotion), Some(PotionId::StrengthPotion)]);
    decision.player.relics = vec![RelicId::SneckoSkull];
    assert_eq!(decide(&decision), GameAction::Choose(ChoiceId(3)));
}

#[test]
fn m14_revision_shop_leaves_when_every_product_is_ineligible() {
    let cards = instances(&[
        (CardId::QuickSlash, 0), (CardId::Reflex, 0), (CardId::Deflect, 0), (CardId::Deflect, 0),
        (CardId::StrikeGreen, 0), (CardId::DefendGreen, 0),
    ]);
    let mut decision = decision(RoomViewKind::Shop, &cards, &[
        offered(1, RoomChoiceKind::Card { id: CardId::QuickSlash, upgrades: 0 }, Some(1)),
        offered(2, RoomChoiceKind::Card { id: CardId::Reflex, upgrades: 0 }, Some(1)),
        offered(3, RoomChoiceKind::Card { id: CardId::Deflect, upgrades: 0 }, Some(1)),
        offered(4, RoomChoiceKind::Potion(PotionId::FirePotion), Some(1)),
        offered(5, RoomChoiceKind::Relic(RelicId::SneckoSkull), Some(1)),
        offered(6, RoomChoiceKind::Leave, None),
    ], 100, vec![Some(PotionId::FirePotion), Some(PotionId::WeakPotion), Some(PotionId::StrengthPotion)]);
    decision.player.relics = vec![RelicId::SneckoSkull, RelicId::BagOfPreparation];
    assert_eq!(decide(&decision), GameAction::Choose(ChoiceId(6)));
}

#[test]
fn m14_revision_removal_preserves_defense_ratio_when_possible() {
    let cards = instances(&[
        (CardId::StrikeGreen, 0), (CardId::StrikeGreen, 0), (CardId::StrikeGreen, 0),
        (CardId::StrikeGreen, 0), (CardId::StrikeGreen, 0), (CardId::StrikeGreen, 0),
        (CardId::StrikeGreen, 0), (CardId::StrikeGreen, 0),
        (CardId::DefendGreen, 0), (CardId::DefendGreen, 0),
        (CardId::DefendGreen, 0), (CardId::DefendGreen, 0),
    ]);
    let context = decision(RoomViewKind::Event(EventId::Purifier), &cards, &[
        offered(50, RoomChoiceKind::ShrineRemove, None),
    ], 99, vec![None; 3]);
    let selection = select_cards(context, &[9, 1], 1);
    assert_eq!(decide(&selection), GameAction::Select(SelectionId(1)), "remove non-defense to retain 3D >= N");
}

#[test]
fn m14_revision_mandatory_removal_still_chooses_a_legal_card_without_safe_option() {
    let cards = instances(&[
        (CardId::StrikeGreen, 0), (CardId::StrikeGreen, 0), (CardId::StrikeGreen, 0),
        (CardId::StrikeGreen, 0), (CardId::StrikeGreen, 0), (CardId::StrikeGreen, 0),
        (CardId::DefendGreen, 0), (CardId::Backflip, 0),
    ]);
    let context = decision(RoomViewKind::Event(EventId::Purifier), &cards, &[
        offered(50, RoomChoiceKind::ShrineRemove, None),
    ], 99, vec![None; 3]);
    let selection = select_cards(context, &[7, 8], 1);
    assert_eq!(decide(&selection), GameAction::Select(SelectionId(7)), "with no ratio-safe option, choose the lowest-value offered card");
}

#[test]
fn m14_revision_empty_cage_removal_uses_strike_then_lowest_value_and_instance_ties() {
    let cards = instances(&[
        (CardId::StrikeGreen, 0), (CardId::StrikeGreen, 0),
        (CardId::Backflip, 0), (CardId::Backflip, 0),
    ]);
    let context = decision(RoomViewKind::BossRelicChild, &cards, &[
        offered(50, RoomChoiceKind::Relic(RelicId::EmptyCage), None),
    ], 99, vec![None; 3]);
    let selection = select_cards(context, &[1, 2, 3, 4], 2);
    assert_eq!(decide(&selection), GameAction::Select(SelectionId(1)), "prefer the offered StrikeGreen instance");

    let cards = instances(&[
        (CardId::Backflip, 0), (CardId::Backflip, 1), (CardId::NoxiousFumes, 0),
    ]);
    let context = decision(RoomViewKind::BossRelicChild, &cards, &[
        offered(50, RoomChoiceKind::Relic(RelicId::EmptyCage), None),
    ], 99, vec![None; 3]);
    let selection = select_cards(context, &[1, 2, 3], 2);
    assert_eq!(decide(&selection), GameAction::Select(SelectionId(1)), "equal card values use the smaller permanent instance");
}

#[test]
fn m14_revision_empty_cage_second_selection_accounts_for_prior_public_selection() {
    // All eleven objects are removable permanent cards in the M14 universe.
    // At the second selection, eligible omits only the already selected
    // QuickSlash, so the public objects/eligible difference identifies it.
    let cards = instances(&[
        (CardId::QuickSlash, 0),
        (CardId::DefendGreen, 0), (CardId::DefendGreen, 0), (CardId::DefendGreen, 0),
        (CardId::Neutralize, 0), (CardId::DeadlyPoison, 0), (CardId::BladeDance, 0),
        (CardId::DaggerThrow, 0), (CardId::Catalyst, 0),
        (CardId::PoisonedStab, 0), (CardId::NoxiousFumes, 0),
    ]);
    let context = decision(RoomViewKind::BossRelicChild, &cards, &[
        offered(50, RoomChoiceKind::Relic(RelicId::EmptyCage), None),
    ], 99, vec![None; 3]);
    let remaining_candidates = (2..=11).collect::<Vec<_>>();
    let mut second_selection = select_cards(context, &remaining_candidates, 2);
    second_selection.selection.as_mut().unwrap().selected_count = 1;

    assert_eq!(
        decide(&second_selection),
        GameAction::Select(SelectionId(5)),
        "after the prior non-defense choice, Neutralize is the lowest-value safe offered removal",
    );
}

#[test]
fn m14_revision_upgrade_selection_uses_preference_then_smallest_instance() {
    let cards = instances(&[
        (CardId::NoxiousFumes, 0), (CardId::Backflip, 0),
        (CardId::Backflip, 0), (CardId::BladeDance, 0),
    ]);
    let context = decision(RoomViewKind::Campfire, &cards, &[
        offered(50, RoomChoiceKind::Smith, None),
    ], 99, vec![None; 3]);
    let selection = select_cards(context, &[1, 2, 3, 4], 1);
    assert_eq!(decide(&selection), GameAction::Select(SelectionId(2)), "Backflip outranks higher frozen value; ties use smallest instance");
}

#[test]
fn m14_revision_falling_removal_preserves_defense_ratio_and_uses_legal_rows() {
    let cards = instances(&[
        (CardId::StrikeGreen, 0), (CardId::StrikeGreen, 0), (CardId::StrikeGreen, 0),
        (CardId::StrikeGreen, 0), (CardId::StrikeGreen, 0), (CardId::StrikeGreen, 0),
        (CardId::StrikeGreen, 0), (CardId::StrikeGreen, 0),
        (CardId::DefendGreen, 0), (CardId::DefendGreen, 0),
        (CardId::DefendGreen, 0), (CardId::DefendGreen, 0),
    ]);
    let decision = decision(RoomViewKind::Event(EventId::Falling), &cards, &[
        offered(3, RoomChoiceKind::FallingLose { card_type: CardType::Skill, instance: Some(9) }, None),
        offered(2, RoomChoiceKind::FallingLose { card_type: CardType::Attack, instance: Some(1) }, None),
        (1, RoomChoiceKind::FallingLose { card_type: CardType::Power, instance: Some(12) }, None, RoomChoiceStatus::Disabled),
    ], 99, vec![None; 3]);
    assert_eq!(decide(&decision), GameAction::Choose(ChoiceId(2)), "choose legal non-defense removal that retains the ratio");
}

#[test]
fn m14_revision_v4_offensive_reward_caps_are_one_including_upgraded_copies() {
    for card in [CardId::BladeDance, CardId::DeadlyPoison, CardId::PoisonedStab] {
        let mut owned = vec![(CardId::StrikeGreen, 0); 4];
        owned.extend([(CardId::DefendGreen, 0); 4]);
        owned.push((card, 1));
        let cards = instances(&owned);
        let decision = decision(RoomViewKind::CardReward, &cards, &[
            offered(1, RoomChoiceKind::Card { id: card, upgrades: 0 }, None),
            offered(2, RoomChoiceKind::Skip, None),
        ], 99, vec![None; 3]);
        assert_eq!(
            decide(&decision),
            GameAction::Choose(ChoiceId(2)),
            "one upgraded copy of {card:?} reaches the new one-copy cap",
        );
    }
}

#[test]
fn m14_revision_v4_shop_prioritizes_only_affordable_unowned_snecko_skull() {
    let defense_short = instances(&[
        (CardId::StrikeGreen, 0), (CardId::StrikeGreen, 0), (CardId::StrikeGreen, 0),
        (CardId::StrikeGreen, 0), (CardId::StrikeGreen, 0), (CardId::StrikeGreen, 0),
        (CardId::StrikeGreen, 0), (CardId::StrikeGreen, 0), (CardId::DefendGreen, 0),
    ]);
    let balanced = instances(&[
        (CardId::StrikeGreen, 0), (CardId::StrikeGreen, 0),
        (CardId::DefendGreen, 0), (CardId::DefendGreen, 0),
    ]);
    let cases = [
        (&defense_short, 200, vec![], ChoiceId(1)),
        (&balanced, 200, vec![RelicId::SneckoSkull], ChoiceId(3)),
        (&balanced, 100, vec![], ChoiceId(3)),
    ];

    for (cards, gold, relics, expected) in cases {
        let mut decision = decision(RoomViewKind::Shop, cards, &[
            offered(1, RoomChoiceKind::Relic(RelicId::SneckoSkull), Some(150)),
            offered(2, RoomChoiceKind::Card { id: CardId::Backflip, upgrades: 0 }, Some(50)),
            offered(3, RoomChoiceKind::Potion(PotionId::DexterityPotion), Some(60)),
        ], gold, vec![None; 3]);
        decision.player.relics = relics;
        assert_eq!(
            decide(&decision),
            GameAction::Choose(expected),
            "shop gold {gold}, owned relics {:?}",
            decision.player.relics,
        );
    }
}

#[test]
fn m14_revision_v4_shop_chooses_affordable_purge_for_a_ratio_safe_strike() {
    let mut owned = vec![(CardId::StrikeGreen, 0); 8];
    owned.extend([(CardId::DefendGreen, 0); 4]);
    let cards = instances(&owned);
    let decision = decision(RoomViewKind::Shop, &cards, &[
        offered(1, RoomChoiceKind::Purge, Some(75)),
        offered(2, RoomChoiceKind::Potion(PotionId::DexterityPotion), Some(60)),
    ], 100, vec![None; 3]);
    assert_eq!(decide(&decision), GameAction::Choose(ChoiceId(1)));
}

#[test]
fn m14_revision_v4_shop_skips_purge_without_a_ratio_safe_strike() {
    let unsafe_strikes = instances(&[
        (CardId::StrikeGreen, 0), (CardId::StrikeGreen, 0), (CardId::StrikeGreen, 0),
        (CardId::StrikeGreen, 0), (CardId::StrikeGreen, 0), (CardId::StrikeGreen, 0),
        (CardId::StrikeGreen, 0), (CardId::StrikeGreen, 0), (CardId::DefendGreen, 0),
    ]);
    let no_strikes = instances(&[(CardId::DefendGreen, 0); 4]);
    for cards in [&unsafe_strikes, &no_strikes] {
        let decision = decision(RoomViewKind::Shop, cards, &[
            offered(1, RoomChoiceKind::Purge, Some(75)),
            offered(2, RoomChoiceKind::Potion(PotionId::DexterityPotion), Some(60)),
        ], 100, vec![None; 3]);
        assert_eq!(
            decide(&decision),
            GameAction::Choose(ChoiceId(2)),
            "Purge requires an offered StrikeGreen whose removal preserves 3D >= N",
        );
    }
}

#[test]
fn m14_revision_v4_shop_purge_grid_prefers_strike_over_defend() {
    let mut owned = vec![(CardId::StrikeGreen, 0); 8];
    owned.extend([(CardId::DefendGreen, 0); 4]);
    let cards = instances(&owned);
    let context = decision(RoomViewKind::Shop, &cards, &[
        offered(50, RoomChoiceKind::Purge, Some(75)),
    ], 100, vec![None; 3]);
    let selection = select_cards(context, &[1, 9], 1);
    assert_eq!(
        decide(&selection),
        GameAction::Select(SelectionId(1)),
        "ratio-safe StrikeGreen is preferred over the lower card_value DefendGreen",
    );
}
