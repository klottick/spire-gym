//! M18 Act2 interaction diagnostics over real shared encounter setup and AI.
//! Decks, inventory, and selected orb/power checkpoints are explicit fixtures;
//! enemy setup, HP, and encounter AI remain sourced from their shared owners.
use super::super::*;
use crate::{
    combat::{characters::{CharacterEffect, OrbPassiveGroup}, model::OwnedCard,
        observe::{DamageCategory, ResolvedEffect, ResolvedEffectKind}},
    content::{CardId, Character, EncounterId, EnemyId, PotionId, RelicId},
    items::CombatItemContext,
};

fn items(relics: Vec<RelicId>, potions: Vec<Option<PotionId>>) -> CombatItemContext {
    CombatItemContext::new(relics, potions)
}

fn enter_defect(encounter: EncounterId, cards: &[(CardId, u32)], opening_draw: usize,
    max_energy: i32, inventory: CombatItemContext) -> Combat {
    // The existing helper supplies diagnostic player HP80 (not canonical75).
    // Enemy stats/powers/AI still come from actual shared Act2 setup.
    let mut entry = super::m6_ordinary::entry(encounter);
    entry.setup.character = Character::Defect;
    entry.setup.player.energy = max_energy;
    entry.setup.player.max_energy = max_energy;
    entry.setup.player.draw_per_turn = cards.len().clamp(1, 10) as i32;
    entry.cards = cards.iter().enumerate().map(|(index, (id, upgrades))|
        permanent_live_card(*id, CardInstanceId(index as u64 + 1), *upgrades)).collect();
    entry.opening_draw = opening_draw;
    let mut inventory = Some(inventory);
    let combat = Combat::enter_with_item_context(entry, &mut inventory).unwrap();
    assert!(inventory.is_none(), "entry transfers the diagnostic inventory once");
    combat
}

fn no_items() -> CombatItemContext { items(vec![], vec![None; 3]) }

fn power(id: &'static str, amount: i32) -> FullPowerEntry {
    FullPowerEntry { id: FullPowerId(id), state: PowerState {
        amount, skip_first: false, monster_source_grace: false,
    }}
}

fn amount(combat: &Combat, owner: &EnemyHandle, id: &str) -> i32 {
    combat.state.enemies.iter().find(|enemy| &enemy.instance_id == owner)
        .unwrap().full_powers.iter().find(|power| power.id.0 == id)
        .map_or(0, |power| power.state.amount)
}

fn enemy_hp(combat: &Combat, owner: &EnemyHandle) -> i32 {
    combat.state.enemies.iter().find(|enemy| &enemy.instance_id == owner).unwrap().hp
}

fn enemy_id(combat: &Combat, owner: &EnemyHandle) -> EnemyId {
    let enemy = combat.state.enemies.iter().find(|enemy| &enemy.instance_id == owner).unwrap();
    match enemy.enemy_id { ActorEnemyId::Full(id) => id, _ => panic!("full encounter actor expected") }
}

fn character(combat: &mut Combat, action: CharacterEffect) {
    combat.enqueue_native_effects(vec![Effect {
        source: Source::Actor(Actor::Player), target: Some(Actor::Player),
        kind: EffectKind::Character(action),
    }], None).unwrap();
}

fn hit(combat: &mut Combat, owner: &EnemyHandle, damage: i32) {
    combat.enqueue_native_effects(vec![Effect {
        source: Source::Actor(Actor::Player), target: Some(Actor::Enemy(owner.clone())),
        kind: EffectKind::Damage(DamagePayload::Fixed(damage)),
    }], None).unwrap();
}

fn actor_move(combat: &mut Combat, slot: usize, kind: EffectKind) {
    let owner = combat.state.enemies[slot].instance_id.clone();
    let actor = Actor::Enemy(owner);
    combat.enqueue_native_effects(vec![Effect {
        source: Source::Actor(actor.clone()), target: Some(actor), kind,
    }], None).unwrap();
}

fn collector_with_summons(cards: &[(CardId, u32)], opening_draw: usize) -> Combat {
    let mut combat = enter_defect(EncounterId::Collector, cards, opening_draw, 3, no_items());
    combat.step_full(&CombatAction::EndTurn).unwrap(); // real initial Collector spawn turn
    assert_eq!(combat.state.enemies.iter().filter(|enemy| enemy_id(&combat, &enemy.instance_id) == EnemyId::TorchHead).count(), 2);
    combat
}

fn damage_to_player(effects: &[ResolvedEffect]) -> Vec<(i32, i32, i32)> {
    effects.iter().filter_map(|event| {
        if event.target != Some(Actor::Player) { return None; }
        match &event.kind {
            ResolvedEffectKind::Damage { incoming, absorbed_block, hp_lost, .. } =>
                Some((*incoming, *absorbed_block, *hp_lost)),
            _ => None,
        }
    }).collect()
}

#[test]
fn m18_lightning_respects_artifact_block_and_bypasses_flight() {
    // Real Spheric Guardian setup; the orb is a diagnostic player checkpoint.
    let mut sphere = enter_defect(EncounterId::SphericGuardian, &[(CardId::StrikeBlue, 0)], 0, 3, no_items());
    let sphere_id = sphere.state.enemies[0].instance_id.clone();
    assert_eq!((sphere.state.enemies[0].block, amount(&sphere, &sphere_id, "artifact")), (40, 3));
    character(&mut sphere, CharacterEffect::Channel(OrbKind::Lightning));
    character(&mut sphere, CharacterEffect::PassiveGroup(OrbPassiveGroup::NaturalEnd));
    assert_eq!((sphere.state.enemies[0].block, amount(&sphere, &sphere_id, "artifact")), (37, 3));

    // Data Disk's real combat-start hook owns +1 Focus: Lightning 4/9 and Frost 3/6.
    let mut disk = enter_defect(EncounterId::SphericGuardian, &[(CardId::StrikeBlue, 0)], 0, 3,
        items(vec![RelicId::DataDisk], vec![None; 3]));
    let disk_sphere = disk.state.enemies[0].instance_id.clone();
    assert_eq!(disk.state.character.focus, 1);
    character(&mut disk, CharacterEffect::Channel(OrbKind::Lightning));
    character(&mut disk, CharacterEffect::PassiveGroup(OrbPassiveGroup::NaturalEnd));
    assert_eq!(disk.state.enemies[0].block, 36);
    character(&mut disk, CharacterEffect::EvokeFront { remove: true });
    assert_eq!((disk.state.enemies[0].block, amount(&disk, &disk_sphere, "artifact")), (27, 3));
    character(&mut disk, CharacterEffect::Channel(OrbKind::Frost));
    character(&mut disk, CharacterEffect::PassiveGroup(OrbPassiveGroup::NaturalEnd));
    assert_eq!(disk.state.player.block, 3);
    character(&mut disk, CharacterEffect::EvokeFront { remove: true });
    assert_eq!(disk.state.player.block, 9);

    // Real Automaton Artifact remains untouched by its typed Orb damage lane.
    let mut automaton = enter_defect(EncounterId::BronzeAutomaton, &[(CardId::StrikeBlue, 0)], 0, 3, no_items());
    let boss = automaton.state.enemies.iter().find(|enemy| enemy_id(&automaton, &enemy.instance_id) == EnemyId::BronzeAutomaton).unwrap().instance_id.clone();
    let hp = enemy_hp(&automaton, &boss);
    automaton.enqueue_native_effects(vec![Effect {
        source: Source::Actor(Actor::Player), target: Some(Actor::Enemy(boss.clone())),
        kind: EffectKind::Damage(DamagePayload::Orb(3)),
    }], None).unwrap();
    assert_eq!((enemy_hp(&automaton, &boss), amount(&automaton, &boss, "artifact")), (hp - 3, 3));

    // Actual Byrd AI reaches Flight3 after its real opening, headbutt, and airborne turns.
    let mut birds = enter_defect(EncounterId::ThreeByrds, &[(CardId::Barrage, 0)], 1, 3, no_items());
    for _ in 0..3 { birds.step_full(&CombatAction::EndTurn).unwrap(); }
    let byrd_ids = birds.state.enemies.iter().filter(|enemy| enemy_id(&birds, &enemy.instance_id) == EnemyId::Byrd)
        .map(|enemy| enemy.instance_id.clone()).collect::<Vec<_>>();
    assert_eq!(byrd_ids.len(), 3);
    assert!(byrd_ids.iter().all(|owner| amount(&birds, owner, "flight") == 3));
    let before = byrd_ids.iter().map(|owner| enemy_hp(&birds, owner)).collect::<Vec<_>>();
    // Explicit player power/orb checkpoint isolates the already accepted Lightning lanes.
    birds.state.player_full_powers.push(power("electrodynamics", 1));
    birds.state.character.orb_slots = vec![Some(OrbKind::Lightning.into()), None, None];
    character(&mut birds, CharacterEffect::PassiveGroup(OrbPassiveGroup::NaturalEnd));
    assert_eq!(byrd_ids.iter().map(|owner| enemy_hp(&birds, owner)).collect::<Vec<_>>(), before.iter().map(|hp| hp - 3).collect::<Vec<_>>());
    assert!(byrd_ids.iter().all(|owner| amount(&birds, owner, "flight") == 3));
    character(&mut birds, CharacterEffect::EvokeFront { remove: false });
    assert_eq!(byrd_ids.iter().map(|owner| enemy_hp(&birds, owner)).collect::<Vec<_>>(), before.iter().map(|hp| hp - 11).collect::<Vec<_>>());
    assert!(byrd_ids.iter().all(|owner| amount(&birds, owner, "flight") == 3));
    let first_before = enemy_hp(&birds, &byrd_ids[0]);
    let barrage = birds.state.piles.hand.iter().find_map(OwnedCard::full).filter(|card| card.id == CardId::Barrage).unwrap().instance;
    birds.step_full(&CombatAction::PlayCard { instance: barrage.0, target: Some(TargetId(1)) }).unwrap();
    assert_eq!((enemy_hp(&birds, &byrd_ids[0]), amount(&birds, &byrd_ids[0], "flight")), (first_before - 2, 2));
}

#[test]
fn m18_frost_multiattack_and_hp_loss_hooks_resolve_per_hit() {
    // A real first Book MultiStab sees one real Cold Snap Frost passive at Focus0.
    let mut book = enter_defect(EncounterId::BookOfStabbing, &[(CardId::ColdSnap, 0)], 1, 3, no_items());
    assert_eq!(book.state.enemies[0].intent_id.as_str(), "book_multi");
    book.step_full(&CombatAction::PlayCard { instance: 1, target: Some(TargetId(1)) }).unwrap();
    assert_eq!(book.state.character.orb_slots[0].unwrap().kind, OrbKind::Frost);
    let observed = book.step_full_observed(&CombatAction::EndTurn);
    observed.result.unwrap();
    assert_eq!(damage_to_player(&observed.effects), vec![(6, 2, 4), (6, 0, 6)]);
    assert_eq!((book.state.player.hp, book.state.player.block), (70, 0));

    // Diagnostic four-energy hand and Emotion Chip inventory; every Book hit is blocked.
    let mut blocked = enter_defect(EncounterId::BookOfStabbing,
        &[(CardId::StaticDischarge, 0), (CardId::DefendBlue, 0), (CardId::DefendBlue, 0), (CardId::ColdSnap, 0)],
        4, 4, items(vec![RelicId::EmotionChip], vec![None; 3]));
    for action in [
        CombatAction::PlayCard { instance: 1, target: None },
        CombatAction::PlayCard { instance: 2, target: None },
        CombatAction::PlayCard { instance: 3, target: None },
        CombatAction::PlayCard { instance: 4, target: Some(TargetId(1)) },
    ] { blocked.step_full(&action).unwrap(); }
    assert_eq!(blocked.state.player.block, 10);
    assert!(blocked.state.player_full_powers.iter().any(|power| power.id.0 == "static_discharge" && power.state.amount == 1));
    let observed = blocked.step_full_observed(&CombatAction::EndTurn);
    observed.result.unwrap();
    assert_eq!(damage_to_player(&observed.effects), vec![(6, 6, 0), (6, 6, 0)]);
    assert_eq!((blocked.state.player.hp, blocked.state.player.block), (80, 0));
    assert_eq!(blocked.state.character.orb_slots.iter().flatten().map(|orb| orb.kind).collect::<Vec<_>>(), vec![OrbKind::Frost]);
    assert!(!blocked.state.item_context.as_ref().unwrap().emotion_chip_armed);
}

#[test]
fn m18_dark_targets_live_summons_and_preserves_payload() {
    // Actual Collector spawn, with a visible diagnostic source-order tie among its Torch Heads.
    let mut tie = collector_with_summons(&[(CardId::StrikeBlue, 0)], 1);
    let first = tie.state.enemies[1].instance_id.clone();
    let second = tie.state.enemies[2].instance_id.clone();
    let first_hp = enemy_hp(&tie, &first);
    let second_hp = enemy_hp(&tie, &second);
    if first_hp > second_hp { hit(&mut tie, &first, first_hp - second_hp); }
    else if second_hp > first_hp { hit(&mut tie, &second, second_hp - first_hp); }
    let first_tied_hp = enemy_hp(&tie, &first);
    let second_tied_hp = enemy_hp(&tie, &second);
    assert_eq!(first_tied_hp, second_tied_hp);
    tie.state.character.orb_slots = vec![Some(OrbState { kind: OrbKind::Dark, dark_damage: Some(6) }), None, None];
    character(&mut tie, CharacterEffect::EvokeFront { remove: true });
    assert_eq!((enemy_hp(&tie, &first), enemy_hp(&tie, &second)), (first_tied_hp - 6, second_tied_hp));

    // Sparse capacity10 diagnostic; Defragment supplies real Focus and Recursion is a real card.
    let mut payload = collector_with_summons(&[(CardId::Defragment, 0), (CardId::Recursion, 1)], 2);
    let low = payload.state.enemies[1].instance_id.clone();
    let sibling = payload.state.enemies[2].instance_id.clone();
    hit(&mut payload, &low, 10);
    assert!(enemy_hp(&payload, &low) < enemy_hp(&payload, &sibling));
    payload.state.character.orb_slots = vec![
        Some(OrbState { kind: OrbKind::Dark, dark_damage: Some(12) }), None,
        Some(OrbState { kind: OrbKind::Dark, dark_damage: Some(15) }), None,
        Some(OrbState { kind: OrbKind::Dark, dark_damage: Some(18) }), None,
        Some(OrbState { kind: OrbKind::Dark, dark_damage: Some(21) }), None, None, None,
    ];
    payload.step_full(&CombatAction::PlayCard { instance: 1, target: None }).unwrap();
    assert_eq!(payload.state.character.focus, 1);
    payload.step_full(&CombatAction::EndTurn).unwrap();
    assert_eq!(payload.state.character.orb_slots.iter().filter_map(|orb| orb.and_then(|orb| orb.dark_damage)).collect::<Vec<_>>(), vec![19, 22, 25, 28]);
    let before = enemy_hp(&payload, &low);
    let observed = payload.step_full_observed(&CombatAction::PlayCard { instance: 2, target: None });
    observed.result.unwrap();
    assert!(observed.effects.iter().any(|event| event.target == Some(Actor::Enemy(low.clone()))
        && matches!(event.kind, ResolvedEffectKind::Damage { incoming: 19, category: DamageCategory::Orb, .. })));
    assert_eq!(enemy_hp(&payload, &low), before - 19);
    assert_eq!(payload.state.character.focus, 1);
    assert_eq!(payload.state.character.orb_slots, vec![
        Some(OrbState { kind: OrbKind::Dark, dark_damage: Some(19) }),
        Some(OrbState { kind: OrbKind::Dark, dark_damage: Some(22) }), None,
        Some(OrbState { kind: OrbKind::Dark, dark_damage: Some(25) }),
        None, Some(OrbState { kind: OrbKind::Dark, dark_damage: Some(28) }),
        None, None, None, None,
    ]);

    // Real Collector revival replaces the dead handle; Dark must select its live replacement.
    let mut replaced = collector_with_summons(&[(CardId::StrikeBlue, 0)], 1);
    let dead = replaced.state.enemies[1].instance_id.clone();
    let sibling = replaced.state.enemies[2].instance_id.clone();
    let dead_hp = enemy_hp(&replaced, &dead);
    hit(&mut replaced, &dead, dead_hp);
    assert_eq!(enemy_hp(&replaced, &dead), 0);
    let stream = replaced.state.enemies[0].full_ai_rng.as_ref().unwrap();
    replaced.state.enemies[0].full_ai_rng = Some(rng::StreamState::new(stream.id().clone(), [0; 32], 23).unwrap());
    actor_move(&mut replaced, 0, EffectKind::RollMove);
    assert_eq!(replaced.state.enemies[0].intent_id.as_str(), "collector_revive");
    actor_move(&mut replaced, 0, EffectKind::EnemyMove);
    let replacement = replaced.state.enemies[3].instance_id.clone();
    assert_ne!(replacement, dead);
    assert!(matches!(&replaced.state.enemies[0].full_memory,
        Some(EnemyMemory::Collector { slots, .. }) if slots.get(&0) == Some(&replacement)));
    hit(&mut replaced, &replacement, 10);
    let replacement_before = enemy_hp(&replaced, &replacement);
    let sibling_before = enemy_hp(&replaced, &sibling);
    replaced.state.character.orb_slots = vec![Some(OrbState { kind: OrbKind::Dark, dark_damage: Some(6) }), None, None];
    character(&mut replaced, CharacterEffect::EvokeFront { remove: true });
    assert_eq!((enemy_hp(&replaced, &dead), enemy_hp(&replaced, &replacement), enemy_hp(&replaced, &sibling)),
        (0, replacement_before - 6, sibling_before));

    // Shared predecessor control outside M18 admission: the accepted M16
    // constructor excludes HalfDead through inherited isDeadOrEscaped; queued
    // captured-target continuation is tested separately by M19.
    let mut darklings_entry = super::m8_shapes::entry(EncounterId::ThreeDarklings);
    darklings_entry.setup.character = Character::Defect;
    let mut darklings = Combat::enter(darklings_entry).unwrap();
    let half_dead = darklings.state.enemies[0].instance_id.clone();
    let half_dead_hp = enemy_hp(&darklings, &half_dead);
    hit(&mut darklings, &half_dead, half_dead_hp);
    assert_eq!(darklings.state.enemies[0].life, ActorLife::HalfDead);
    let mut expected = darklings.state.enemies.iter().map(|e|e.hp).collect::<Vec<_>>();
    let target=(1..expected.len()).min_by_key(|i|(expected[*i],*i)).unwrap();expected[target]-=12;
    let deaths = darklings.state.hook_events.iter().filter(|event| event.hook == Hook::Death || event.hook == Hook::HalfDeath).count();
    darklings.state.character.orb_slots = vec![Some(OrbState { kind: OrbKind::Dark, dark_damage: Some(12) }), None, None];
    character(&mut darklings, CharacterEffect::EvokeFront { remove: true });
    assert_eq!(darklings.state.enemies[0].life, ActorLife::HalfDead);
    assert_eq!(darklings.state.enemies.iter().map(|e|e.hp).collect::<Vec<_>>(),expected);
    assert_eq!(darklings.state.hook_events.iter().filter(|event| event.hook == Hook::Death || event.hook == Hook::HalfDeath).count(), deaths);
}

fn hand_card(combat: &Combat, instance: CardInstanceId) -> &LiveCard {
    combat.state.piles.hand.iter().filter_map(OwnedCard::full).find(|card| card.instance == instance).unwrap()
}

fn owned_card(combat: &Combat, instance: CardInstanceId) -> &LiveCard {
    combat.state.piles.hand.iter().chain(&combat.state.piles.draw).chain(&combat.state.piles.discard)
        .chain(&combat.state.piles.exhaust).chain(&combat.state.played_powers)
        .filter_map(OwnedCard::full).find(|card| card.instance == instance).unwrap()
}

fn play_confused(combat: &mut Combat, instance: CardInstanceId, id: CardId, base_cost: i32,
    energy_gain: i32) -> i32 {
    let card = hand_card(combat, instance);
    assert_eq!(card.id, id);
    let cost = card.local.randomized_cost.expect("actual Confused draw exposes its randomized price");
    assert!((0..=3).contains(&cost));
    // Source ConfusionPower.onCardDraw sets the drawn instance's combat cost.
    assert_eq!(card.local.mutable_combat_cost, Some(cost));
    assert_eq!(crate::combat::cards::base_cost(&combat.state, card), Some(base_cost));
    assert_eq!(card.local.turn_cost, Some(cost));
    let visible = combat.public_full().unwrap().hand.into_iter().find(|card| card.instance == instance).unwrap();
    assert_eq!(visible.effective_cost, Some(cost));
    let action = CombatAction::PlayCard { instance: instance.0, target: None };
    assert!(combat.legal_actions_full().unwrap().contains(&action));
    let energy = combat.state.player.energy;
    combat.step_full(&action).unwrap();
    assert_eq!(combat.state.player.energy, energy - cost + energy_gain);
    assert_eq!(owned_card(combat, instance).local.mutable_combat_cost, Some(cost),
        "Confused price is the combat instance's cost for the rest of combat");
    assert_eq!(crate::combat::cards::base_cost(&combat.state, owned_card(combat, instance)), Some(base_cost),
        "the permanent card's printed base cost is unchanged");
    cost
}

#[test]
fn m18_plasma_confused_costs_and_capacity_share_accepted_owners() {
    for focus in [-9, 9] {
        // Diagnostic signed-Focus checkpoints; Snecko's actual first move applies Confused.
        let mut combat = enter_defect(EncounterId::Snecko,
            &[(CardId::Fusion, 0), (CardId::Recursion, 1)], 0, 3, no_items());
        combat.state.character.focus = focus;
        combat.step_full(&CombatAction::EndTurn).unwrap();
        assert!(combat.state.player_full_powers.iter().any(|power| power.id.0 == "confused"));
        play_confused(&mut combat, CardInstanceId(1), CardId::Fusion, 2, 0);
        assert_eq!(combat.state.character.orb_slots[0].unwrap().kind, OrbKind::Plasma);
        combat.step_full(&CombatAction::EndTurn).unwrap();
        assert_eq!(combat.state.player.energy, 4, "real StartTurn Plasma passive adds1 independent of signed Focus");
        let before = combat.state.player.energy;
        let cost = hand_card(&combat, CardInstanceId(2)).local.randomized_cost.unwrap();
        assert!((0..=3).contains(&cost));
        play_confused(&mut combat, CardInstanceId(2), CardId::Recursion, 0, 2);
        assert_eq!(combat.state.player.energy, before - cost + 2, "real Recursion evokes Plasma for2 independent of Focus");
        assert_eq!(combat.state.character.orb_slots[0].unwrap().kind, OrbKind::Plasma,
            "Recursion rechannels the same Plasma owner after its one evoke");
        assert_eq!(combat.state.character.focus, focus);
    }

    // Diagnostic hand and belt; actual Snecko glare draws/randomizes every tested permanent card.
    let mut combat = enter_defect(EncounterId::Snecko,
        &[(CardId::Capacitor, 1), (CardId::Capacitor, 1), (CardId::Consume, 1)], 0, 3,
        items(vec![], vec![Some(PotionId::FocusPotion), Some(PotionId::PotionOfCapacity), None]));
    combat.step_full(&CombatAction::EndTurn).unwrap();
    assert!(combat.state.player_full_powers.iter().any(|power| power.id.0 == "confused"));

    let focus_invalid = CombatAction::UsePotion { slot: 0, target: Some(TargetId(1)) };
    let focus_before = combat.snapshot();
    assert!(combat.step_full(&focus_invalid).is_err());
    assert_eq!(combat.snapshot(), focus_before);
    assert_eq!(combat.item_context().unwrap().potions()[0], Some(PotionId::FocusPotion));
    let use_focus = CombatAction::UsePotion { slot: 0, target: None };
    assert!(combat.legal_actions_full().unwrap().contains(&use_focus));
    combat.step_full(&use_focus).unwrap();
    assert_eq!(combat.state.character.focus, 2);
    assert_eq!(combat.item_context().unwrap().potions()[0], None);
    let after_focus = combat.snapshot();
    assert!(combat.step_full(&use_focus).is_err());
    assert_eq!(combat.snapshot(), after_focus, "an accepted potion cannot be consumed twice");

    let use_capacity = CombatAction::UsePotion { slot: 1, target: None };
    assert!(combat.legal_actions_full().unwrap().contains(&use_capacity));
    combat.step_full(&use_capacity).unwrap();
    assert_eq!(combat.state.character.orb_slots.len(), 5);
    assert_eq!(combat.item_context().unwrap().potions()[1], None);

    play_confused(&mut combat, CardInstanceId(1), CardId::Capacitor, 1, 0);
    assert_eq!(combat.state.character.orb_slots.len(), 8);
    combat.step_full(&CombatAction::EndTurn).unwrap();
    play_confused(&mut combat, CardInstanceId(2), CardId::Capacitor, 1, 0);
    assert_eq!(combat.state.character.orb_slots.len(), 10);
    combat.step_full(&CombatAction::EndTurn).unwrap();
    // Explicit last-slot Plasma checkpoint makes Consume's accepted resize rule observable.
    combat.state.character.orb_slots[9] = Some(OrbKind::Plasma.into());
    let energy = combat.state.player.energy;
    let consume_cost = play_confused(&mut combat, CardInstanceId(3), CardId::Consume, 2, 0);
    assert_eq!(combat.state.player.energy, energy - consume_cost);
    assert_eq!((combat.state.character.focus, combat.state.character.orb_slots.len()), (5, 9));
    assert!(combat.state.character.orb_slots.iter().all(Option::is_none),
        "Consume removes the truncated last slot without evoking its Plasma");
}

#[test]
fn m18_fission_in_actual_snecko_uses_current_orb_and_confused_owners() {
    for focus in [-9, 9] {
        let mut combat = enter_defect(EncounterId::Snecko,
            &[(CardId::Fusion, 1), (CardId::Fission, 1)], 0, 3, no_items());
        combat.state.character.focus = focus; // supplied signed-Focus checkpoint
        combat.step_full(&CombatAction::EndTurn).unwrap();
        play_confused(&mut combat, CardInstanceId(1), CardId::Fusion, 1, 0);
        combat.step_full(&CombatAction::EndTurn).unwrap();
        assert_eq!(combat.state.player.energy, 4);
        // Fission+ evokes Plasma for2, then grants1 energy for the occupied orb.
        // Draw/randomization uses the same Confused owner; no favorable cost is required.
        play_confused(&mut combat, CardInstanceId(2), CardId::Fission, 0, 3);
        assert!(combat.state.character.orb_slots.iter().all(Option::is_none));
        assert_eq!(combat.state.character.focus, focus);
    }
}
