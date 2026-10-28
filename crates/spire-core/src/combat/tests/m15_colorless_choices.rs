use super::super::*;
use super::super::cards::ColorlessEffect;
use crate::combat::model::OwnedCard;
use crate::content::{CardColor, CardId, CardRarity, CardType, Catalog, Character};

#[derive(Clone, Copy)]
enum Zone { Hand, Draw, Discard, Exhaust }

fn id(token: &str) -> CardId {
    Catalog::standard().cards().iter().find(|row| row.id.as_str() == token)
        .unwrap_or_else(|| panic!("missing Colorless choice card token {token}")).id
}

fn fight(token: &str, upgrades: u32) -> Combat {
    let mut combat = super::full_runtime::entered_combat(3, 3, |card| {
        *card = card_state::permanent(id(token), card.instance, upgrades);
    });
    combat.state.character = characters::CharacterState::for_character(Character::Silent);
    combat.state.enemies[0].hp = 500;
    combat.state.enemies[0].max_hp = 500;
    combat
}

fn add(combat: &mut Combat, card_id: CardId, instance: u64, zone: Zone) {
    let card = card_state::permanent(card_id, CardInstanceId(instance), 0);
    let handle = card_state::handle(card.instance);
    assert!(combat.state.card_ids_issued.insert(handle), "duplicate test card identity {instance}");
    let owned = OwnedCard::Full(card);
    match zone {
        Zone::Hand => combat.state.piles.hand.push(owned),
        Zone::Draw => combat.state.piles.draw.push(owned),
        Zone::Discard => combat.state.piles.discard.push(owned),
        Zone::Exhaust => combat.state.piles.exhaust.push(owned),
    }
}

fn live(combat: &Combat, instance: u64) -> &LiveCard {
    let handle = card_state::handle(CardInstanceId(instance));
    combat.state.piles.all(combat.state.in_play.as_ref())
        .chain(combat.state.inactive_cards.iter())
        .chain(combat.state.queued_autoplay.values())
        .find(|owned| owned.handle() == handle).and_then(OwnedCard::full)
        .unwrap_or_else(|| panic!("missing card instance {instance}"))
}

fn set_hand_cost(combat: &mut Combat, instance: u64, combat_cost: i32, turn_cost: i32) {
    let handle = card_state::handle(CardInstanceId(instance));
    let card = combat.state.piles.hand.iter_mut().find(|owned| owned.handle() == handle)
        .and_then(OwnedCard::full_mut).unwrap_or_else(|| panic!("missing hand card {instance}"));
    card.local.mutable_combat_cost = Some(combat_cost);
    card.local.turn_cost = Some(turn_cost);
}

fn play(combat: &mut Combat) {
    combat.step_full(&CombatAction::PlayCard { instance: 3, target: None }).unwrap();
}

fn picked(selection: &PendingSelection, ordinal: usize) -> u64 {
    selection.candidates[ordinal].as_str().parse().unwrap()
}

#[test]
fn m15_colorless_choices_register_all_nine_source_kinds() {
    let expected = [
        ("apotheosis", CardRarity::Rare),
        ("enlightenment", CardRarity::Uncommon),
        ("forethought", CardRarity::Uncommon),
        ("madness", CardRarity::Uncommon),
        ("purity", CardRarity::Uncommon),
        ("secret_technique", CardRarity::Rare),
        ("secret_weapon", CardRarity::Rare),
        ("thinking_ahead", CardRarity::Rare),
        ("violence", CardRarity::Rare),
    ];
    for (token, rarity) in expected {
        let definition = Catalog::standard().cards().iter()
            .find(|row| row.id.as_str() == token)
            .unwrap_or_else(|| panic!("missing Colorless choice card token {token}"));
        assert_eq!(definition.color, CardColor::Colorless, "{token} color");
        assert_eq!(definition.card_type, CardType::Skill, "{token} type");
        assert_eq!(definition.rarity, rarity, "{token} rarity");
        assert!(!definition.definition_only, "{token} is a supported M15 card");
    }
}

#[test]
fn m15_colorless_choices_publish_cost_magic_and_exhaust_for_both_upgrade_levels() {
    let expected = [
        ("apotheosis", CardRarity::Rare, 2, 1, true, true, None, None),
        ("enlightenment", CardRarity::Uncommon, 0, 0, false, false, None, None),
        ("forethought", CardRarity::Uncommon, 0, 0, false, false, None, None),
        ("madness", CardRarity::Uncommon, 1, 0, true, true, None, None),
        ("purity", CardRarity::Uncommon, 0, 0, true, true, Some(3), Some(5)),
        ("secret_technique", CardRarity::Rare, 0, 0, true, false, None, None),
        ("secret_weapon", CardRarity::Rare, 0, 0, true, false, None, None),
        ("thinking_ahead", CardRarity::Rare, 0, 0, true, false, Some(2), Some(2)),
        ("violence", CardRarity::Rare, 0, 0, true, true, Some(3), Some(4)),
    ];
    for (index, (token, rarity, base_cost, upgraded_cost, base_exhaust, upgraded_exhaust,
        base_magic, upgraded_magic)) in expected.into_iter().enumerate() {
        let definition = Catalog::standard().card(id(token)).unwrap();
        assert_eq!(definition.rarity, rarity, "{token} rarity");
        for (upgrades, cost, exhaust, magic) in [
            (0, base_cost, base_exhaust, base_magic),
            (1, upgraded_cost, upgraded_exhaust, upgraded_magic),
        ] {
            let card = card_state::permanent(definition.id, CardInstanceId(500 + index as u64), upgrades);
            assert_eq!(cards::starting_cost(card.id, upgrades), Some(cost), "{token}+{upgrades} cost");
            assert_eq!(card.local.exhaust_on_finish, exhaust, "{token}+{upgrades} permanent exhaust");
            let values = cards::public_values(&card).unwrap();
            assert_eq!(values.magic_number, magic, "{token}+{upgrades} magic");
            assert_eq!(values.cost_kind, CardCostKind::Fixed, "{token}+{upgrades} cost kind");
            let generated = card_state::generated(card.id, CardInstanceId(600 + index as u64), upgrades).unwrap();
            assert_eq!(generated.local.exhaust_on_finish, exhaust, "{token}+{upgrades} generated exhaust");
            assert_eq!(cards::public_values(&generated).unwrap().magic_number, magic, "{token}+{upgrades} generated magic");
        }
    }
}

#[test]
fn m15_colorless_choices_apotheosis_temporarily_upgrades_each_owned_pile_only() {
    let mut combat = fight("apotheosis", 0);
    add(&mut combat, CardId::StrikeGreen, 100, Zone::Hand);
    add(&mut combat, CardId::DefendGreen, 101, Zone::Draw);
    add(&mut combat, CardId::Survivor, 102, Zone::Discard);
    add(&mut combat, CardId::Bash, 103, Zone::Exhaust);
    add(&mut combat, CardId::AscendersBane, 104, Zone::Discard);

    play(&mut combat);

    for instance in [1, 2, 100, 101, 102, 103] {
        assert_eq!((live(&combat, instance).upgrades, live(&combat, instance).local.temporary_upgrades),
            (1, 1), "owned upgradable instance {instance} gets one combat upgrade");
    }
    assert_eq!(live(&combat, 104).upgrades, 0, "a non-upgradable curse is untouched");
    assert_eq!(live(&combat, 3).upgrades, 0, "Apotheosis does not upgrade its in-play parent");
    assert_eq!(combat.state.piles.exhaust.iter().filter(|owned| owned.handle().as_str() == "3").count(), 1,
        "the parent exhausts exactly once after its all-pile work");
}

#[test]
fn m15_colorless_choices_enlightenment_separates_turn_and_combat_cost_writers() {
    let mut base = fight("enlightenment", 0);
    set_hand_cost(&mut base, 1, 3, 3);
    play(&mut base);
    assert_eq!((live(&base, 1).local.mutable_combat_cost, live(&base, 1).local.turn_cost), (Some(3), Some(1)));
    base.step_full(&CombatAction::EndTurn).unwrap();
    assert_eq!(live(&base, 1).local.turn_cost, Some(3), "base turn-only reduction expires at the next turn");

    let mut upgraded = fight("enlightenment", 1);
    set_hand_cost(&mut upgraded, 1, 3, 0);
    play(&mut upgraded);
    assert_eq!((live(&upgraded, 1).local.mutable_combat_cost, live(&upgraded, 1).local.turn_cost), (Some(1), Some(0)),
        "the upgraded combat-cost predicate is independent from the current turn predicate");
    upgraded.step_full(&CombatAction::EndTurn).unwrap();
    assert_eq!(live(&upgraded, 1).local.turn_cost, Some(1), "combat reduction remains after turn reset");
}

#[test]
fn m15_colorless_choices_madness_prefers_turn_cost_and_keeps_combat_mutation_through_reset_and_copy() {
    let mut combat = fight("madness", 0);
    set_hand_cost(&mut combat, 1, 0, 2);
    set_hand_cost(&mut combat, 2, 3, 0);
    play(&mut combat);
    assert_eq!((live(&combat, 1).local.mutable_combat_cost, live(&combat, 1).local.turn_cost), (Some(0), Some(0)),
        "a positive turn-price candidate wins before the combat-price fallback");
    assert_eq!((live(&combat, 2).local.mutable_combat_cost, live(&combat, 2).local.turn_cost), (Some(3), Some(0)));
    let copy = live(&combat, 1).stat_equivalent_copy(CardInstanceId(700));
    assert_eq!((copy.local.mutable_combat_cost, copy.local.turn_cost), (Some(0), Some(0)),
        "a stat-equivalent copy preserves the actual Madness cost writers");
    combat.step_full(&CombatAction::EndTurn).unwrap();
    assert_eq!((live(&combat, 1).local.mutable_combat_cost, live(&combat, 1).local.turn_cost), (Some(0), Some(0)),
        "turn reset restores from the persistent combat price");

    let mut fallback = fight("madness", 1);
    set_hand_cost(&mut fallback, 1, 0, 0);
    set_hand_cost(&mut fallback, 2, 3, 0);
    play(&mut fallback);
    assert_eq!((live(&fallback, 2).local.mutable_combat_cost, live(&fallback, 2).local.turn_cost), (Some(0), Some(0)),
        "when no positive turn cost exists, the positive combat-price set is used");

    let mut random = fight("madness", 0);
    set_hand_cost(&mut random, 1, 0, 2);
    set_hand_cost(&mut random, 2, 0, 3);
    let candidate_order = random.state.piles.hand.iter().filter_map(OwnedCard::full)
        .filter(|card| card.instance.0 != 3 && card.local.turn_cost.is_some_and(|cost|cost>0))
        .map(|card|card.instance.0).collect::<Vec<_>>();
    let (index, expected_stream) = super::super::rng::uniform_below(random.state.card_rng.as_ref().unwrap(), 2).unwrap();
    assert_eq!(candidate_order.len(), 2);
    let expected = candidate_order[index as usize];
    play(&mut random);
    assert_eq!(random.state.card_rng.as_ref(), Some(&expected_stream), "Madness samples the preferred eligible set once");
    assert_eq!((live(&random, expected).local.mutable_combat_cost, live(&random, expected).local.turn_cost), (Some(0), Some(0)));
    let other = if expected == 1 { 2 } else { 1 };
    assert_eq!(live(&random, other).local.turn_cost, Some(if other == 1 { 2 } else { 3 }),
        "only the random eligible instance receives the permanent-combat cost mutation");
}

#[test]
fn m15_colorless_choices_purity_accepts_zero_selection_and_runs_real_exhaust_hooks() {
    let mut empty = fight("purity", 0);
    play(&mut empty);
    let selection = empty.state.pending_selection.as_ref().unwrap();
    assert_eq!((selection.minimum, selection.maximum), (0, 2));
    assert_eq!(selection.operation, SelectionOperation::Move(SelectionDisposition::Exhaust));
    empty.step_full(&CombatAction::ConfirmSelection).unwrap();
    assert!(empty.state.pending_selection.is_none());
    assert_eq!(empty.state.piles.exhaust.iter().filter(|owned| owned.handle().as_str() == "3").count(), 1,
        "confirming zero choices resumes Purity cleanup once");
    assert!(empty.state.piles.exhaust.iter().all(|owned| owned.handle().as_str() == "3"),
        "no selected hand card moves when the optional set is empty");

    let mut selected = fight("purity", 1);
    selected.state.player_full_powers.push(FullPowerEntry { id: FullPowerId("feel_no_pain"),
        state: PowerState { amount: 4, skip_first: false, monster_source_grace: false } });
    play(&mut selected);
    let candidates = selected.state.pending_selection.as_ref().unwrap().candidates.clone();
    selected.step_full(&CombatAction::SelectCard { instance: candidates[0].as_str().parse().unwrap() }).unwrap();
    selected.step_full(&CombatAction::SelectCard { instance: candidates[1].as_str().parse().unwrap() }).unwrap();
    selected.step_full(&CombatAction::ConfirmSelection).unwrap();
    for candidate in candidates {
        assert!(selected.state.piles.exhaust.iter().any(|owned| owned.handle() == candidate),
            "selected card reaches the real exhaust pile");
    }
    assert_eq!(selected.state.player.block, 12, "Feel No Pain observes two selected exhausts and Purity's own exhaust");
    assert_eq!(selected.state.piles.exhaust.iter().filter(|owned| owned.handle().as_str() == "3").count(), 1,
        "the selected continuation resumes the parent exactly once");
}

#[test]
fn m15_colorless_choices_forethought_preserves_order_optionality_and_mutable_cost_predicate() {
    let mut base = fight("forethought", 0);
    set_hand_cost(&mut base, 1, 2, 0);
    play(&mut base);
    let selection = base.state.pending_selection.as_ref().unwrap();
    assert_eq!((selection.minimum, selection.maximum), (1, 1));
    assert_eq!(selection.operation, SelectionOperation::Bottomdeck);
    base.step_full(&CombatAction::SelectCard { instance: 1 }).unwrap();
    assert_eq!(base.state.piles.draw.first().unwrap().handle().as_str(), "1");
    assert!(live(&base, 1).local.free_to_play_once, "positive mutable combat cost grants the one-play flag despite turn cost zero");

    let mut upgraded = fight("forethought", 1);
    set_hand_cost(&mut upgraded, 1, 2, 0);
    set_hand_cost(&mut upgraded, 2, 0, 1);
    play(&mut upgraded);
    let selection = upgraded.state.pending_selection.as_ref().unwrap();
    assert_eq!((selection.minimum, selection.maximum), (0, 2));
    upgraded.step_full(&CombatAction::SelectCard { instance: 1 }).unwrap();
    upgraded.step_full(&CombatAction::SelectCard { instance: 2 }).unwrap();
    upgraded.step_full(&CombatAction::ConfirmSelection).unwrap();
    assert_eq!(upgraded.state.piles.draw.iter().take(2).map(OwnedCard::handle).collect::<Vec<_>>(),
        vec![card_state::handle(CardInstanceId(2)), card_state::handle(CardInstanceId(1))],
        "cards selected A then B are drawn A then B after the original draw pile");
    assert_eq!(upgraded.state.public.known_draw_bottom,
        vec![card_state::handle(CardInstanceId(2)), card_state::handle(CardInstanceId(1))]);
    assert!(live(&upgraded, 1).local.free_to_play_once);
    assert!(!live(&upgraded, 2).local.free_to_play_once,
        "combat-cost zero does not newly receive the flag even with positive turn cost");

    let mut skipped = fight("forethought", 1);
    play(&mut skipped);
    skipped.step_full(&CombatAction::ConfirmSelection).unwrap();
    assert!(skipped.state.piles.hand.iter().any(|owned| owned.handle().as_str() == "1"));
    assert!(skipped.state.piles.hand.iter().any(|owned| owned.handle().as_str() == "2"));
    assert_eq!(skipped.state.piles.discard.iter().filter(|owned| owned.handle().as_str() == "3").count(), 1,
        "confirming no upgraded choices still finishes the parent once");
}

#[test]
fn m15_colorless_choices_secret_searches_are_typed_forced_when_single_and_overflow_to_discard() {
    let invalid = fight("secret_technique", 0);
    let before = invalid.clone();
    assert!(matches!(cards::prepare(&invalid, &CombatAction::PlayCard { instance: 3, target: None }),
        Err(crate::run::RunError::InvalidAction)), "Secret Technique cannot be used without a draw-pile Skill");
    assert_eq!(invalid, before, "unavailable type-filtered search is atomic");

    let mut technique = fight("secret_technique", 0);
    add(&mut technique, CardId::StrikeGreen, 100, Zone::Draw);
    add(&mut technique, CardId::DefendGreen, 101, Zone::Draw);
    add(&mut technique, CardId::Survivor, 102, Zone::Draw);
    play(&mut technique);
    let candidates = technique.state.pending_selection.as_ref().unwrap().candidates.clone();
    assert_eq!(candidates.iter().map(|handle| live(&technique, handle.as_str().parse().unwrap()).id).collect::<Vec<_>>(),
        vec![CardId::DefendGreen, CardId::Survivor], "Secret Technique exposes only Skill instances in draw order");
    technique.step_full(&CombatAction::SelectCard { instance: candidates[0].as_str().parse().unwrap() }).unwrap();
    assert!(technique.state.piles.hand.iter().any(|owned| owned.handle() == candidates[0]));
    assert!(!technique.state.piles.draw.iter().any(|owned| owned.handle() == candidates[0]));

    let mut weapon = fight("secret_weapon", 1);
    add(&mut weapon, CardId::DefendGreen, 110, Zone::Draw);
    add(&mut weapon, CardId::StrikeGreen, 111, Zone::Draw);
    play(&mut weapon);
    assert!(weapon.state.pending_selection.is_none(), "the single matching Attack is moved without a prompt");
    assert!(weapon.state.piles.hand.iter().any(|owned| owned.handle().as_str() == "111"));
    assert!(!weapon.state.piles.hand.iter().any(|owned| owned.handle().as_str() == "110"));

    let mut full = fight("secret_technique", 0);
    let owner_handle = card_state::handle(CardInstanceId(3));
    let owner_index = full.state.piles.hand.iter().position(|owned| owned.handle() == owner_handle).unwrap();
    full.state.in_play = Some(full.state.piles.hand.remove(owner_index));
    for instance in 120..128 { add(&mut full, CardId::StrikeGreen, instance, Zone::Hand); }
    add(&mut full, CardId::DefendGreen, 200, Zone::Draw);
    add(&mut full, CardId::Survivor, 201, Zone::Draw);
    full.state.phase = Phase::PlayerAction;
    full.state.pending.push_back(Effect { source: Source::Card(owner_handle.clone()), target: None, kind: EffectKind::FinishCard });
    full.state.pending.push_back(Effect { source: Source::None, target: None, kind: EffectKind::Decision });
    let work = ColorlessEffect::SecretRetrieve { card_type: CardType::Skill };
    let effect = Effect { source: Source::Card(owner_handle), target: None, kind: EffectKind::Colorless(work.clone()) };
    cards::validate_colorless_work(&full.state, &effect, &work).unwrap();
    cards::apply_colorless(&mut full.state, &effect, &work).unwrap();
    assert_eq!(full.state.pending_selection.as_ref().unwrap().disposition, SelectionDisposition::Discard,
        "a full hand redirects Secret retrieval to discard");
    let selected = picked(full.state.pending_selection.as_ref().unwrap(), 0);
    full.step_full(&CombatAction::SelectCard { instance: selected }).unwrap();
    assert!(full.state.piles.discard.iter().any(|owned| owned.handle().as_str() == selected.to_string()));
    assert!(!full.state.piles.hand.iter().any(|owned| owned.handle().as_str() == selected.to_string()));
}

#[test]
fn m15_colorless_choices_thinking_ahead_draws_then_uses_the_shared_topdeck_selection() {
    let mut combat = fight("thinking_ahead", 0);
    add(&mut combat, CardId::StrikeGreen, 100, Zone::Draw);
    add(&mut combat, CardId::DefendGreen, 101, Zone::Draw);
    play(&mut combat);
    let selection = combat.state.pending_selection.as_ref().unwrap();
    assert_eq!(selection.operation, SelectionOperation::Topdeck);
    assert!(selection.candidates.iter().any(|handle| handle.as_str() == "100"));
    combat.step_full(&CombatAction::SelectCard { instance: 100 }).unwrap();
    assert_eq!(combat.state.piles.draw.last().unwrap().handle().as_str(), "100", "the selected card becomes actual draw top");
    assert!(combat.state.piles.exhaust.iter().any(|owned| owned.handle().as_str() == "3"),
        "base Thinking Ahead exhausts after its draw and selection");
}

#[test]
fn m15_colorless_choices_violence_moves_random_attacks_without_draw_callbacks() {
    for upgrades in [0, 1] {
        let count = if upgrades == 0 { 3 } else { 4 };
        let mut combat = fight("violence", upgrades);
        combat.state.piles.draw.clear();
        combat.state.public.known_draw.clear();
        combat.state.public.known_draw_bottom.clear();
        combat.state.silent.history_enabled = true;
        combat.state.silent.drawn_cards.clear();
        add(&mut combat, CardId::EndlessAgony, 300, Zone::Draw);
        for index in 1..count {
            let attack = [CardId::StrikeGreen, CardId::Bane, CardId::SwiftStrike][(index as usize - 1) % 3];
            add(&mut combat, attack, 300 + index as u64, Zone::Draw);
        }
        add(&mut combat, CardId::DefendGreen, 310, Zone::Draw);
        let selected = combat.state.piles.draw.iter().filter_map(OwnedCard::full)
            .filter(|card| Catalog::standard().card(card.id).is_some_and(|definition|definition.card_type==CardType::Attack))
            .map(|card| card_state::handle(card.instance)).collect::<Vec<_>>();
        let mut remaining = selected.clone();
        let mut expected_order = Vec::new();
        let mut expected_stream = combat.state.card_rng.clone().unwrap();
        for _ in 0..count {
            let (index, next) = super::super::rng::uniform_below(&expected_stream, remaining.len() as u128).unwrap();
            expected_stream = next;
            expected_order.push(remaining.remove(index as usize));
        }

        play(&mut combat);

        let observed_order = combat.state.piles.hand.iter().map(OwnedCard::handle)
            .filter(|handle| selected.contains(handle)).collect::<Vec<_>>();
        assert_eq!(observed_order, expected_order, "Violence samples live attacks without replacement in card-random order");
        assert_eq!(combat.state.card_rng.as_ref(), Some(&expected_stream));
        assert!(combat.state.piles.draw.iter().any(|owned| owned.handle().as_str() == "310"),
            "non-Attack draw candidates remain in draw");
        assert_eq!(combat.state.piles.all(combat.state.in_play.as_ref()).filter(|owned|
            owned.full().is_some_and(|card|card.id==CardId::EndlessAgony)).count(), 1,
            "moving an Attack without a draw hook does not create an Endless Agony copy");
        assert!(combat.state.silent.drawn_cards.is_empty(), "Violence does not report actual card draws");
    }
}

#[test]
fn m15_colorless_choices_finite_costs_are_not_x_and_pay_their_source_price() {
    for (token,base,up) in [("apotheosis",2,1),("madness",1,0),("purity",0,0),("violence",0,0)] {
        for upgrades in 0..=1 {
            let mut c=fight(token,upgrades);let price=if upgrades>0{up}else{base};
            assert!(!live(&c,3).local.x_cost,"{token} is a fixed-cost Skill after shared constructor integration");
            play(&mut c);
            if c.state.pending_selection.is_some(){c.step_full(&CombatAction::ConfirmSelection).unwrap();}
            assert_eq!(c.state.player.energy,3-price,"{token} pays its one physical card cost");
        }
    }
}

#[test]
fn m15_colorless_choices_snecko_combat_cost_is_not_the_printed_cost() {
    let mut madness=fight("madness",1);
    for instance in [1,2] {
        let card=madness.state.piles.hand.iter_mut().find(|owned|owned.handle().as_str()==instance.to_string()).unwrap().full_mut().unwrap();
        card.local.mutable_combat_cost=None;card.local.randomized_cost=Some(0);card.local.turn_cost=Some(0);
    }
    let rng=madness.state.card_rng.clone();play(&mut madness);
    assert_eq!(madness.state.card_rng,rng,"zero mutable combat and turn costs supply no Madness candidate");
    let mut forethought=fight("forethought",0);
    let card=forethought.state.piles.hand.iter_mut().find(|owned|owned.handle().as_str()=="1").unwrap().full_mut().unwrap();
    card.local.mutable_combat_cost=None;card.local.randomized_cost=Some(0);card.local.turn_cost=Some(2);
    play(&mut forethought);forethought.step_full(&CombatAction::SelectCard{instance:1}).unwrap();
    assert!(!live(&forethought,1).local.free_to_play_once,"Forethought reads Snecko's mutable zero base even when the turn price is positive");
}

#[test]
fn m15_colorless_choices_prices_remain_public_when_retrieval_is_unusable() {
    for token in ["secret_technique","secret_weapon"] {
        let mut c=fight(token,0);
        assert_eq!(c.public_full().unwrap().hand.iter().find(|card|card.instance.0==3).unwrap().effective_cost,Some(0));
        let before=c.snapshot();assert_eq!(c.step_full(&CombatAction::PlayCard{instance:3,target:None}),Err(crate::run::RunError::InvalidAction));assert_eq!(c.snapshot(),before);
    }
}
