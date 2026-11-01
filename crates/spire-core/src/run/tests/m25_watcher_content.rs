//! M25 Run-side content, settlement, relic and event ownership.
use super::super::*;
use crate::{
    combat::{Actor, CombatAction, DamagePayload, Effect, EffectKind, Outcome, Source, TargetId, rng::Partition},
    content::{CardId, CardType, Character, EncounterId, EventId, RelicId},
    items::{CombatItemContext, CombatStartupProfile},
};
use super::m17_defect_act1::choose as choose_room;

fn history() -> CreationProfile {
    let mut profile = CreationProfile::experienced();
    profile.neow_eligible = false;
    profile.seen_bosses[0] = vec!["Hexaghost", "SlimeBoss"];
    profile.seen_bosses[1] = vec!["Automaton", "Collector"];
    profile.seen_bosses[2] = vec!["TimeEater", "DonuAndDeca"];
    profile
}

fn m25_run_with_root(root: [u8; 32]) -> Run {
    Run::new_room_scenario(
        RunConfig { character: Character::Watcher, ascension: 0 },
        history(),
        RunRng::new(root, Partition::Test, 0),
        RoomScenario::M25WatcherHeart,
    ).unwrap()
}

fn m25_run() -> Run { m25_run_with_root([7; 32]) }

fn mapped() -> Run {
    let mut run = m25_run();
    choose_room(&mut run, RoomChoiceKind::NeowLament);
    run
}

fn mapped_with_root(root: [u8; 32]) -> Run {
    let mut run = m25_run_with_root(root);
    choose_room(&mut run, RoomChoiceKind::NeowLament);
    run
}

fn anchor_map(run: &mut Run) -> NodeId {
    let map = run.state.map.as_mut().expect("M25 map after short Neow");
    let node = map.nodes[0].id;
    map.current = Some(node);
    node
}

fn install_event(run: &mut Run, event: EventId, act: u8) {
    let node = anchor_map(run);
    if act == 3 {
        prepare_act3(run);
    } else if act == 2 {
        run.state.act = 2;
        let room = run.state.room_loop.as_mut().unwrap();
        room.act1 = None;
        room.act2 = Some(Default::default());
        room.act3 = None;
    }
    run.state.act = act;
    run.state.floor = match act { 1 => 2, 2 => 20, _ => 36 };
    run.state.room = "Map";
    run.state.map.as_mut().unwrap().act = act;
    if act < 3 {
        let mut next = run.next_choice;
        let screen = rooms::events::screen(&mut next, event).unwrap();
        run.next_choice = next;
        run.state.room_loop.as_mut().unwrap().screen = screen;
    } else {
        let mut next = run.next_choice;
        let prepared = rooms::events::act3::prepare(run, event, node, 0, &mut next).unwrap();
        run.state.rng = prepared.rng;
        run.next_choice = next;
        let room = run.state.room_loop.as_mut().unwrap();
        room.screen = prepared.screen;
        room.act3_event = Some(prepared.frame);
    }
}

fn install_smith(run: &mut Run) {
    anchor_map(run);
    let mut next = run.next_choice;
    let heal = (run.state.max_hp as f32 * 0.3_f32) as u32;
    let screen = rooms::view(&mut next, RoomViewKind::Campfire, vec![
        (RoomChoiceKind::Rest { heal }, None), (RoomChoiceKind::Smith, None), (RoomChoiceKind::Recall, None),
    ]).unwrap();
    run.next_choice = next;
    run.state.room_loop.as_mut().unwrap().screen = screen;
}

fn select_instance(run: &mut Run, instance: u64) {
    let action = run.decision().unwrap().candidates.into_iter()
        .find(|candidate| candidate.object.as_ref().is_some_and(|object| object.instance == instance))
        .unwrap_or_else(|| panic!("no physical selection candidate for instance {instance}"))
        .action;
    run.step(&action).unwrap();
}

fn install_deck(run: &mut Run, cards: &[(CardId, u32)]) -> Vec<u64> {
    let old = run.state.deck.iter().map(|card| card.instance).collect::<Vec<_>>();
    for instance in old { mutations::remove_card(&mut run.state, instance).unwrap(); }
    cards.iter().map(|(id, upgrades)| mutations::obtain_card(&mut run.state, *id, *upgrades).unwrap()).collect()
}

fn prepare_act3(run: &mut Run) {
    if run.state.act == 3 { return; }
    run.state.act = 3;
    run.state.floor = 34;
    if let Some(map) = run.state.map.as_mut() { map.act = 3; }
    let room = run.state.room_loop.as_mut().unwrap();
    room.act1 = None;
    room.act2 = None;
    room.act3 = Some(Default::default());
}

fn enter_owned(run: &mut Run, encounter: EncounterId) {
    if matches!(encounter, EncounterId::OrbWalker | EncounterId::ThreeShapes | EncounterId::FourShapes
        | EncounterId::ThreeDarklings | EncounterId::WrithingMass | EncounterId::Reptomancer | EncounterId::AwakenedOne) {
        prepare_act3(run);
    }
    let kind = match encounter {
        EncounterId::Reptomancer => MapNodeKind::Elite,
        EncounterId::AwakenedOne => MapNodeKind::Boss,
        _ => MapNodeKind::Monster,
    };
    run.enter_owned(encounter, CombatOrigin::Training { kind }).unwrap();
}

fn force_victory(run: &mut Run) -> crate::combat::CombatExit {
    for _ in 0..16 {
        let public = run.state.combat.as_ref().unwrap().public_full().unwrap();
        if public.outcome == Some(Outcome::Victory) { break; }
        let alive = public.enemies.iter().enumerate().filter_map(|(slot, enemy)| (enemy.hp > 0).then_some(slot)).collect::<Vec<_>>();
        if alive.is_empty() { break; }
        for slot in alive {
            super::m9_boss::hit(run, slot, 9999);
            if run.state.combat.as_ref().unwrap().public_full().unwrap().outcome == Some(Outcome::Victory) { break; }
        }
    }
    assert_eq!(run.state.combat.as_ref().unwrap().public_full().unwrap().outcome, Some(Outcome::Victory));
    let exit = run.state.combat.as_ref().unwrap().exit().unwrap();
    run.settle_combat().unwrap();
    exit
}

fn force_death(run: &mut Run) {
    let hp = run.state.combat.as_ref().unwrap().public_full().unwrap().player.hp;
    run.state.combat.as_mut().unwrap().enqueue_native_effects(vec![Effect {
        source: Source::None,
        target: Some(Actor::Player),
        kind: EffectKind::Damage(DamagePayload::HpLoss(hp)),
    }], None).unwrap();
    assert_eq!(run.state.combat.as_ref().unwrap().public_full().unwrap().outcome, Some(Outcome::Death));
    run.settle_combat().unwrap();
}

fn gain_parasite(run: &mut Run) -> u64 {
    enter_owned(run, EncounterId::WrithingMass);
    let before = run.state.deck.iter().filter(|card| card.id == CardId::Parasite).count();
    crate::combat::tests::m19_defect_interactions::implant_checkpoint(run.state.combat.as_mut().unwrap());
    run.choose_combat(CombatAction::EndTurn).unwrap();
    let journal = run.state.combat.as_ref().unwrap().permanent_gains();
    assert_eq!(journal.len(), 1);
    assert_eq!(journal[0].id, CardId::Parasite);
    assert_eq!(run.state.active_combat.as_ref().unwrap().applied_gain_ordinal, 1);
    assert_eq!(run.state.deck.iter().filter(|card| card.id == CardId::Parasite).count(), before + 1);
    let instance = run.state.deck.iter().find(|card| card.id == CardId::Parasite).unwrap().instance;
    force_victory(run);
    instance
}

fn play_card(run: &mut Run, instance: u64, target: Option<TargetId>) {
    run.choose_combat(CombatAction::PlayCard { instance, target }).unwrap();
}

fn lower_enemy_to(run: &mut Run, slot: usize, hp: i32) {
    let current = run.state.combat.as_ref().unwrap().public_full().unwrap().enemies[slot].hp;
    if current > hp { super::m9_boss::hit(run, slot, current - hp); }
}

fn play_lesson_fatal(run: &mut Run, lesson: u64, slot: usize) {
    lower_enemy_to(run, slot, 10);
    play_card(run, lesson, Some(TargetId(slot as u64 + 1)));
}

fn arm_fault_once(run: &Run, expected_deck: &[(u64, u32)]) {
    let mut faulted = run.clone();
    crate::combat::tests::m19_defect_interactions::arm_later_dispatch_fault(faulted.state.combat.as_mut().unwrap());
    let error = faulted.choose_combat(CombatAction::EndTurn).unwrap_err();
    assert!(matches!(error, RunError::Faulted(_)));
    assert_eq!(faulted.state.deck.iter().filter(|card| expected_deck.iter().any(|(id, _)| *id == card.instance))
        .map(|card| (card.instance, card.upgrades)).collect::<Vec<_>>(), expected_deck);
    let frozen = faulted.clone();
    assert_eq!(faulted.choose_combat(CombatAction::EndTurn), Err(error));
    assert_eq!(faulted, frozen);
}

fn arm_fault_gold(run: &Run, expected_gold: u32) {
    let mut faulted = run.clone();
    crate::combat::tests::m19_defect_interactions::arm_later_dispatch_fault(faulted.state.combat.as_mut().unwrap());
    let error = faulted.choose_combat(CombatAction::EndTurn).unwrap_err();
    assert!(matches!(error, RunError::Faulted(_)));
    assert_eq!(faulted.state.gold, expected_gold);
    let frozen = faulted.clone();
    assert_eq!(faulted.choose_combat(CombatAction::EndTurn), Err(error));
    assert_eq!(faulted, frozen);
}

fn combat_rng_text(run: &Run) -> String {
    let debug = format!("{:?}", run.state.combat.as_ref().unwrap().snapshot());
    let start = debug.find("rng: CombatRng {").expect("combat RNG in snapshot") + "rng: ".len();
    let mut depth = 0usize;
    for (offset, ch) in debug[start..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 { return debug[start..start + offset + 1].to_owned(); }
            }
            _ => (),
        }
    }
    panic!("unterminated CombatRng snapshot")
}

#[test]
fn m25_content_every_card_obtain_upgrade_remove_next_combat() {
    use crate::content::watcher::M25_WATCHER_CARDS;
    assert_eq!(M25_WATCHER_CARDS.len(), 75, "the admitted permanent Watcher roster is literally 75 kinds");
    assert_eq!(M25_WATCHER_CARDS.iter().collect::<std::collections::BTreeSet<_>>().len(), 75);
    for &id in M25_WATCHER_CARDS {
        let mut run = mapped();
        let instances = install_deck(&mut run, &[(id, 0), (id, 0)]);
        let smith_card = instances[0];
        let shrine_card = instances[1];

        install_smith(&mut run);
        choose_room(&mut run, RoomChoiceKind::Smith);
        select_instance(&mut run, smith_card);
        assert_eq!(run.state.deck.iter().find(|card| card.instance == smith_card).unwrap().upgrades, 1, "Smith {id:?}");

        install_event(&mut run, EventId::UpgradeShrine, 1);
        let shrine = run.decision().unwrap().room.unwrap().choices.into_iter()
            .find(|choice| choice.kind == RoomChoiceKind::ShrineUpgrade).unwrap();
        assert_eq!(shrine.status, RoomChoiceStatus::Offered, "UpgradeShrine offers the physical {id:?} instance {shrine_card}");
        run.step(&GameAction::Choose(shrine.id)).unwrap();
        select_instance(&mut run, shrine_card);
        assert_eq!(run.state.deck.iter().find(|card| card.instance == shrine_card).unwrap().upgrades, 1, "UpgradeShrine {id:?}");

        enter_owned(&mut run, EncounterId::Cultist);
        let exit = force_victory(&mut run);
        let piles = exit.hand.iter().chain(&exit.draw).chain(&exit.discard).chain(&exit.exhaust)
            .chain(exit.in_play.iter()).chain(&exit.play_frames).chain(&exit.queued_autoplay)
            .chain(&exit.stasis).chain(&exit.played_powers).chain(&exit.inactive_cards);
        for instance in [smith_card, shrine_card] {
            assert!(piles.clone().any(|card| card.instance.0 == instance && card.id == id && card.upgrades == 1),
                "the next combat piles contain the upgraded physical {id:?} instance {instance}");
        }

        for instance in instances {
            install_event(&mut run, EventId::Purifier, 1);
            choose_room(&mut run, RoomChoiceKind::ShrineRemove);
            select_instance(&mut run, instance);
            assert!(!run.state.deck.iter().any(|card| card.instance == instance), "Purifier removes only {id:?} instance {instance}");
        }
        assert_eq!(run.state.terminal, Some(RunTerminal::EmptyDeck), "the second Purifier removal empties this supplied deck");
    }

    let mut run = mapped();
    let parasite = gain_parasite(&mut run);
    assert!(!mutations::can_upgrade(&run.state, run.state.deck.iter().find(|card| card.instance == parasite).unwrap()));
    install_smith(&mut run);
    choose_room(&mut run, RoomChoiceKind::Smith);
    assert!(!run.state.pending.as_ref().unwrap().selection.candidates.iter().any(|candidate| candidate.card.instance == parasite));
    run.step(&GameAction::Cancel).unwrap();
    install_event(&mut run, EventId::UpgradeShrine, 1);
    choose_room(&mut run, RoomChoiceKind::ShrineUpgrade);
    assert!(!run.state.pending.as_ref().unwrap().selection.candidates.iter().any(|candidate| candidate.card.instance == parasite));
    let ordinary = run.state.pending.as_ref().unwrap().selection.candidates[0].card.instance;
    select_instance(&mut run, ordinary);
    let max_hp = run.state.max_hp;
    install_event(&mut run, EventId::Purifier, 1);
    choose_room(&mut run, RoomChoiceKind::ShrineRemove);
    select_instance(&mut run, parasite);
    assert_eq!(run.state.max_hp, max_hp - 3, "Parasite removal costs literal three maximum HP");
    let frozen = run.clone();
    assert!(mutations::remove_card(&mut run.state, parasite).is_err());
    assert_eq!(run, frozen, "Parasite max-HP loss cannot be charged twice");
}

#[test]
fn m25_content_lesson_learned_and_wish_persist_once() {
    for death in [false, true] {
        let mut run = mapped();
        let instances = install_deck(&mut run, &[(CardId::LessonLearned, 1), (CardId::DefendPurple, 0)]);
        let (lesson, extra) = (instances[0], instances[1]);
        enter_owned(&mut run, EncounterId::ThreeShapes);
        play_lesson_fatal(&mut run, lesson, 0);
        assert_eq!(run.state.deck.iter().find(|card| card.instance == extra).unwrap().upgrades, 1);
        assert_eq!(run.state.active_combat.as_ref().unwrap().applied_upgrade_ordinal, 1);
        arm_fault_once(&run, &[(extra, 1)]);
        if !death { force_victory(&mut run); } else { force_death(&mut run); }
        assert_eq!(run.state.deck.iter().find(|card| card.instance == extra).unwrap().upgrades, 1, "Lesson Learned settles once on {death:?}");
        assert_eq!(run.state.last_combat.as_ref().unwrap().outcome, if death { Outcome::Death } else { Outcome::Victory });
        assert_eq!(run.state.terminal, if death { Some(RunTerminal::Death) } else { None });
    }

    for (upgrades, gold_gain) in [(0, 25), (1, 30)] {
        for death in [false, true] {
            let mut run = mapped();
            install_deck(&mut run, &[(CardId::Wish, upgrades)]);
            enter_owned(&mut run, EncounterId::ThreeShapes);
            let wish = run.state.deck[0].instance;
            play_card(&mut run, wish, None);
            run.choose_combat(CombatAction::ChooseCardOffer { index: 1 }).unwrap();
            let expected_gold = 99 + gold_gain;
            assert_eq!(run.state.gold, expected_gold, "FameAndFortune +{gold_gain} synchronizes into Run immediately");
            arm_fault_gold(&run, expected_gold);
            if death { force_death(&mut run); } else { force_victory(&mut run); }
            assert_eq!(run.state.gold, expected_gold, "FameAndFortune settles once on {death:?}");
        }
    }

    for death in [false, true] {
        let (mut run, greed) = (0u16..=255).find_map(|seed| {
            let mut candidate = mapped_with_root([seed as u8; 32]);
            install_deck(&mut candidate, &[(CardId::ForeignInfluence, 0)]);
            enter_owned(&mut candidate, EncounterId::ThreeShapes);
            let foreign = candidate.state.deck[0].instance;
            play_card(&mut candidate, foreign, None);
            let offer = candidate.decision().unwrap().combat.unwrap().card_offer.unwrap();
            let choice = offer.cards.iter().position(|id| *id == CardId::HandOfGreed)?;
            candidate.choose_combat(CombatAction::ChooseCardOffer { index: choice as u32 }).unwrap();
            let greed = candidate.decision().unwrap().combat.unwrap().hand.iter()
                .find(|card| card.id == CardId::HandOfGreed)?.instance.0;
            Some((candidate, greed))
        }).expect("the fixed 256-seed fixture set offers Hand of Greed from Foreign Influence");
        let hand = run.decision().unwrap().combat.unwrap().hand;
        assert!(hand.iter().any(|card| card.instance.0 == greed));
        lower_enemy_to(&mut run, 0, 20);
        play_card(&mut run, greed, Some(TargetId(1)));
        assert_eq!(run.state.gold, 119, "fatal base Hand of Greed from Foreign Influence grants literal +20");
        arm_fault_gold(&run, 119);
        if death { force_death(&mut run); } else { force_victory(&mut run); }
        assert_eq!(run.state.gold, 119, "Foreign Influence Hand of Greed settles once on {death:?}");
    }
}

#[test]
fn m25_content_lesson_learned_no_eligible_card() {
    for parasite_only in [false, true] {
        let mut run = mapped();
        let parasite = if parasite_only { Some(gain_parasite(&mut run)) } else { None };
        let keep = parasite.into_iter().collect::<Vec<_>>();
        let discard = run.state.deck.iter().map(|card| card.instance).filter(|id| !keep.contains(id)).collect::<Vec<_>>();
        for instance in discard { mutations::remove_card(&mut run.state, instance).unwrap(); }
        let lesson = mutations::obtain_card(&mut run.state, CardId::LessonLearned, 1).unwrap();
        enter_owned(&mut run, EncounterId::ThreeShapes);
        let run_rng = run.state.rng.clone();
        let card_rng = combat_rng_text(&run);
        let control = run.clone();
        play_lesson_fatal(&mut run, lesson, 0);
        assert!(run.state.combat.as_ref().unwrap().permanent_card_upgrades().is_empty());
        assert_eq!(run.state.active_combat.as_ref().unwrap().applied_upgrade_ordinal, 0);
        assert_eq!(run.state.rng, run_rng);
        assert_eq!(combat_rng_text(&run), card_rng);
        assert_eq!(run.state.rng, control.state.rng, "control Run stream unchanged");
        assert_eq!(combat_rng_text(&run), combat_rng_text(&control),
            "control combat card-random stream unchanged");
        assert_eq!(run.state.deck.iter().find(|card| card.instance == lesson).unwrap().upgrades, 1);
        if let Some(instance) = parasite { assert_eq!(run.state.deck.iter().find(|card| card.instance == instance).unwrap().upgrades, 0); }
    }

    for (encounter, slot_kind) in [
        (EncounterId::Reptomancer, "minion"),
        (EncounterId::ThreeDarklings, "darkling"),
        (EncounterId::AwakenedOne, "awakened"),
    ] {
        let mut run = mapped();
        let lesson = install_deck(&mut run, &[(CardId::LessonLearned, 0), (CardId::DefendPurple, 0)])[0];
        enter_owned(&mut run, encounter);
        let slot = match slot_kind {
            "minion" => run.decision().unwrap().combat.unwrap().enemies.iter().position(|enemy| enemy.powers.iter().any(|power| power.id.0 == "minion")).unwrap(),
            "darkling" => 0,
            _ => 2,
        };
        lower_enemy_to(&mut run, slot, 10);
        play_card(&mut run, lesson, Some(TargetId(slot as u64 + 1)));
        assert!(run.state.combat.as_ref().unwrap().permanent_card_upgrades().is_empty(), "{slot_kind} kill is not fatal to the combat");
        if slot_kind == "darkling" || slot_kind == "awakened" {
            assert!(run.decision().unwrap().combat.unwrap().enemies[slot].half_dead);
        }
    }
}

#[test]
fn m25_content_relic_hooks_across_combats() {
    use crate::combat::ResolvedEffectKind;
    let mut run = mapped();
    run.state.relics.retain(|relic| *relic != RelicId::NeowsLament);
    run.state.relic_counters.remove(&RelicId::NeowsLament);
    install_deck(&mut run, &[(CardId::Weave, 0), (CardId::StrikePurple, 0), (CardId::DefendPurple, 0),
        (CardId::DefendPurple, 0), (CardId::DefendPurple, 0), (CardId::DefendPurple, 0), (CardId::DefendPurple, 0),
        (CardId::DefendPurple, 0), (CardId::DefendPurple, 0), (CardId::DefendPurple, 0)]);
    for relic in [RelicId::Damaru, RelicId::Duality, RelicId::CloakClasp, RelicId::Melange] {
        crate::items::acquire_relic(&mut run.state, relic).unwrap();
    }
    run.state.hp = 999;
    run.state.max_hp = 999;
    let held = run.state.relics.clone();
    let counters = run.state.relic_counters.clone();
    let deck = run.state.deck.clone();
    let gold = run.state.gold;

    for visit in 1..=2 {
        enter_owned(&mut run, EncounterId::ThreeShapes);
        let start = run.decision().unwrap().combat.unwrap();
        assert_eq!(start.player.powers.iter().find(|power| power.id.0 == "mantra").map(|power| power.amount), Some(1),
            "Damaru grants one Mantra on combat {visit} entry");

        let mut duality_fired = false;
        let mut clasp_fired = false;
        let mut melange_fired = false;
        for _ in 0..8 {
            let public = run.decision().unwrap().combat.unwrap();
            if !duality_fired {
                if let Some(strike) = public.hand.iter().find(|card| card.id == CardId::StrikePurple) {
                    run.step_observed(&GameAction::Combat(CombatAction::PlayCard {
                        instance: strike.instance.0, target: Some(TargetId(1)),
                    })).result.unwrap();
                    let after = run.decision().unwrap().combat.unwrap();
                    assert_eq!(after.player.powers.iter().find(|power| power.id.0 == "dexterity").map(|power| power.amount), Some(1),
                        "Duality grants one temporary Dexterity on combat {visit}");
                    assert_eq!(after.player.powers.iter().find(|power| power.id.0 == "lose_dexterity").map(|power| power.amount), Some(1));
                    duality_fired = true;
                    continue;
                }
            }
            if public.selection.is_some() {
                let selection = public.selection.as_ref().unwrap();
                assert_eq!(selection.candidates.len(), 3, "Melange Scry 3 on combat {visit}");
                let instance = selection.candidates[0].instance.0;
                run.choose_combat(CombatAction::SelectCard { instance }).unwrap();
                run.choose_combat(CombatAction::ConfirmSelection).unwrap();
                melange_fired = true;
                break;
            }
            let hand_size = public.hand.len();
            let observed = run.step_observed(&GameAction::Combat(CombatAction::EndTurn));
            observed.result.unwrap();
            if !clasp_fired {
                assert!(observed.effects.iter().any(|effect| effect.target == Some(Actor::Player)
                    && matches!(effect.kind, ResolvedEffectKind::Block { before, after } if after - before == hand_size as i32)),
                    "Cloak Clasp adds one Block per hand card before cleanup on combat {visit}");
                clasp_fired = true;
            }
        }
        assert!(duality_fired, "the Watcher Attack entered from the permanent deck and triggered Duality on combat {visit}");
        assert!(clasp_fired, "Cloak Clasp fired on combat {visit}");
        assert!(melange_fired, "Melange fired after the real draw-pile reshuffle on combat {visit}");

        force_victory(&mut run);
        assert!(run.state.combat.is_none() && run.state.active_combat.is_none(), "combat {visit} state is released at settlement");
        assert_eq!(run.state.relic_counters, counters, "combat {visit} relic counters return unchanged");
        assert_eq!(run.state.relics, held);
        assert_eq!(run.state.deck, deck);
        assert_eq!(run.state.gold, gold);
        assert_eq!(run.state.combat_progress.settled, visit);
    }
}

#[test]
fn m25_events_have_watcher_semantics() {
    use crate::{combat::CardInstanceId, content::Catalog};

    let mut lament = m25_run();
    assert_eq!(lament.decision().unwrap().room.unwrap().kind, RoomViewKind::Neow);
    choose_room(&mut lament, RoomChoiceKind::NeowLament);
    assert!(lament.state.relics.contains(&RelicId::NeowsLament));
    let mut max_hp = m25_run();
    choose_room(&mut max_hp, RoomChoiceKind::NeowMaxHp { amount: 7 });
    assert_eq!((max_hp.state.hp, max_hp.state.max_hp), (79, 79), "short Neow raises Watcher max HP by literal 7");

    let shrine_rows = [
        (CardId::Crescendo, 1, 0), (CardId::Tranquility, 1, 0), (CardId::Scrawl, 1, 0),
        (CardId::Study, 2, 1), (CardId::Vault, 3, 2), (CardId::Omniscience, 4, 3), (CardId::Alpha, 1, 1),
    ];
    for (id, base_cost, upgraded_cost) in shrine_rows {
        let mut run = mapped();
        let instance = install_deck(&mut run, &[(id, 0)])[0];
        assert_eq!(crate::combat::m25_watcher_live_card(id, CardInstanceId(instance), 0).unwrap().local.turn_cost, Some(base_cost));
        install_event(&mut run, EventId::UpgradeShrine, 1);
        choose_room(&mut run, RoomChoiceKind::ShrineUpgrade);
        select_instance(&mut run, instance);
        assert_eq!(run.state.deck.iter().find(|card| card.instance == instance).unwrap().upgrades, 1, "Shrine upgrades {id:?}");
        enter_owned(&mut run, EncounterId::Cultist);
        let live = run.decision().unwrap().combat.unwrap().hand.into_iter().find(|card| card.instance.0 == instance).unwrap();
        assert_eq!(live.effective_cost, Some(upgraded_cost), "Shrine applies {id:?} cost change in the next M25 combat");
        if id == CardId::Alpha { assert!(live.values.unwrap().innate, "upgraded Alpha is Innate"); }
    }

    let mut locked = mapped();
    let parasite = gain_parasite(&mut locked);
    let already_upgraded = mutations::obtain_card(&mut locked.state, CardId::Crescendo, 1).unwrap();
    install_event(&mut locked, EventId::UpgradeShrine, 1);
    choose_room(&mut locked, RoomChoiceKind::ShrineUpgrade);
    assert!(!locked.state.pending.as_ref().unwrap().selection.candidates.iter()
        .any(|candidate| candidate.card.instance == parasite || candidate.card.instance == already_upgraded),
        "Parasite and already-upgraded cards are not Shrine options");

    let mut simplicity = mapped();
    let nonstarter = mutations::obtain_card(&mut simplicity.state, CardId::SandsOfTime, 0).unwrap();
    install_event(&mut simplicity, EventId::AncientWriting, 2);
    choose_room(&mut simplicity, RoomChoiceKind::AncientSimplicity);
    for card in &simplicity.state.deck {
        let expected = u32::from(matches!(card.id, CardId::StrikePurple | CardId::DefendPurple));
        assert_eq!(card.upgrades, expected, "Simplicity upgrades Watcher starter only: {:?}", card.id);
    }
    assert_eq!(simplicity.state.deck.iter().find(|card| card.instance == nonstarter).unwrap().upgrades, 0,
        "Simplicity excludes nonstarter M25 cards");
    let mut elegance = mapped();
    let first = mutations::obtain_card(&mut elegance.state, CardId::StrikePurple, 0).unwrap();
    let second = mutations::obtain_card(&mut elegance.state, CardId::StrikePurple, 0).unwrap();
    install_event(&mut elegance, EventId::AncientWriting, 2);
    choose_room(&mut elegance, RoomChoiceKind::AncientElegance);
    select_instance(&mut elegance, second);
    assert!(elegance.state.deck.iter().any(|card| card.instance == first));
    assert!(!elegance.state.deck.iter().any(|card| card.instance == second), "Elegance removes the selected physical copy");

    let mut joust = mapped();
    for id in crate::content::watcher::M25_WATCHER_CARDS {
        mutations::obtain_card(&mut joust.state, *id, 0).unwrap();
    }
    assert!(crate::content::watcher::M25_WATCHER_CARDS.iter().all(|id| joust.state.deck.iter().any(|card| card.id == *id)));
    let deck = joust.state.deck.clone();
    joust.state.gold = 99;
    install_event(&mut joust, EventId::Joust, 2);
    let rng_before = joust.state.rng.clone();
    choose_room(&mut joust, RoomChoiceKind::JoustMurderer);
    assert_eq!(joust.state.gold, 49, "Joust charges its literal 50-gold payment");
    assert_eq!(joust.state.rng, rng_before, "payment does not reveal or reroll the outcome");
    choose_room(&mut joust, RoomChoiceKind::JoustContinue);
    let result = joust.decision().unwrap().room.unwrap().choices[0].kind.clone();
    let RoomChoiceKind::JoustResult { owner_wins } = result else { panic!("Joust reveals its captured result") };
    choose_room(&mut joust, result);
    assert_eq!(joust.state.gold, if owner_wins { 49 } else { 149 }, "Murderer bet payout uses the revealed outcome");
    assert_eq!(joust.state.deck, deck, "Joust leaves the complete Watcher roster unchanged");

    let mut complete = mapped();
    let parasite_instance = gain_parasite(&mut complete);
    for id in crate::content::watcher::M25_WATCHER_CARDS {
        mutations::obtain_card(&mut complete.state, *id, 0).unwrap();
    }
    install_event(&mut complete, EventId::Falling, 3);
    let rows = complete.decision().unwrap().room.unwrap().choices;
    assert_eq!(rows.iter().filter(|row| matches!(row.kind, RoomChoiceKind::FallingLose { instance: Some(_), .. })).count(), 3);
    assert!(rows.iter().all(|row| !matches!(row.kind, RoomChoiceKind::FallingLose { instance: Some(id), .. } if id == parasite_instance)),
        "Falling never captures the journal-created Parasite");
    for card_type in [CardType::Skill, CardType::Power, CardType::Attack] {
        let row = rows.iter().find(|row| matches!(row.kind, RoomChoiceKind::FallingLose { card_type: found, .. } if found == card_type)).unwrap();
        let RoomChoiceKind::FallingLose { instance: Some(instance), .. } = row.kind else { panic!("complete roster has each Falling type") };
        let physical = complete.state.deck.iter().find(|card| card.instance == instance).unwrap();
        assert_eq!(Catalog::standard().card(physical.id).unwrap().card_type, card_type);
    }
    let attack = rows.iter().find(|row| matches!(row.kind, RoomChoiceKind::FallingLose { card_type: CardType::Attack, .. })).unwrap();
    let RoomChoiceKind::FallingLose { instance: Some(lost), .. } = attack.kind else { unreachable!() };
    let before = complete.state.deck.clone();
    complete.step(&GameAction::Choose(attack.id)).unwrap();
    assert_eq!(complete.state.deck, before.into_iter().filter(|card| card.instance != lost).collect::<Vec<_>>(),
        "Falling removes the exact captured Attack instance");

    let mut parasite_pair = mapped();
    let parasite = gain_parasite(&mut parasite_pair);
    let discard = parasite_pair.state.deck.iter().map(|card| card.instance).filter(|id| *id != parasite).collect::<Vec<_>>();
    for instance in discard { mutations::remove_card(&mut parasite_pair.state, instance).unwrap(); }
    let x = mutations::obtain_card(&mut parasite_pair.state, CardId::StrikePurple, 0).unwrap();
    install_event(&mut parasite_pair, EventId::Falling, 3);
    let row = parasite_pair.decision().unwrap().room.unwrap().choices.into_iter()
        .find(|row| row.kind == RoomChoiceKind::FallingLose { card_type: CardType::Attack, instance: Some(x) }).unwrap();
    parasite_pair.step(&GameAction::Choose(row.id)).unwrap();
    assert_eq!(parasite_pair.state.terminal, None, "[Parasite, X] Falling removes X and Run continues");
    assert_eq!(parasite_pair.state.deck.iter().map(|card| card.instance).collect::<Vec<_>>(), vec![parasite]);

    let mut last = mapped();
    let x = install_deck(&mut last, &[(CardId::Vigilance, 0)])[0];
    install_event(&mut last, EventId::Falling, 3);
    let row = last.decision().unwrap().room.unwrap().choices.into_iter()
        .find(|row| row.kind == RoomChoiceKind::FallingLose { card_type: CardType::Skill, instance: Some(x) }).unwrap();
    assert_eq!(last.step(&GameAction::Choose(row.id)).unwrap().terminal, Some(RunTerminal::EmptyDeck));
    assert!(last.state.deck.is_empty(), "[X] Falling ends with EmptyDeck");

    for (has_mask, gold) in [(false, 0), (false, 99), (true, 0), (true, 99)] {
        let mut tomb = mapped();
        tomb.state.gold = gold;
        if has_mask { crate::items::acquire_relic(&mut tomb.state, RelicId::RedMask).unwrap(); }
        install_event(&mut tomb, EventId::TombOfLordRedMask, 3);
        let rng = tomb.state.rng.clone();
        choose_room(&mut tomb, if has_mask { RoomChoiceKind::TombGold } else { RoomChoiceKind::TombOfferGold });
        assert_eq!(tomb.state.gold, if has_mask { gold + 222 } else { 0 }, "Tomb gold {gold} mask {has_mask}");
        assert_eq!(tomb.state.relics.iter().filter(|relic| **relic == RelicId::RedMask).count(), 1);
        assert_eq!(tomb.state.rng, rng, "Tomb does not request RNG");
    }

    let mut purifier = mapped();
    let x = install_deck(&mut purifier, &[(CardId::StrikePurple, 0)])[0];
    install_event(&mut purifier, EventId::Purifier, 1);
    choose_room(&mut purifier, RoomChoiceKind::ShrineRemove);
    select_instance(&mut purifier, x);
    assert_eq!(purifier.state.terminal, Some(RunTerminal::EmptyDeck), "Purifier removal of the last card ends EmptyDeck");
}

fn supplied(cards: &[(CardId, u32)], relics: &[RelicId]) -> AuthoredRunStart {
    AuthoredRunStart {
        hp: 72, max_hp: 72, gold: 99,
        deck: cards.iter().enumerate().map(|(index, (id, upgrades))| CardInstance {
            misc: 0, instance: index as u64 + 1, id: *id, upgrades: *upgrades,
        }).collect(),
        relics: relics.to_vec(), potions: vec![None; 3], relic_counters: Default::default(),
    }
}

#[test]
fn m25_content_old_profiles_reject_new_content() {
    let cards = crate::run::M25_WATCHER_HEART_CARDS.iter().copied()
        .filter(|id| !crate::run::M24_WATCHER_ACT3_CARDS.contains(id)).collect::<Vec<_>>();
    let relics = crate::run::M25_WATCHER_HEART_RELICS.iter().copied()
        .filter(|id| !crate::run::M24_WATCHER_ACT3_RELICS.contains(id)).collect::<Vec<_>>();
    assert_eq!(cards.len(), 47, "M25-only cards are the 47 additions to M24");
    assert_eq!(relics, [RelicId::Damaru, RelicId::Duality, RelicId::CloakClasp, RelicId::Melange]);
    for id in &cards {
        assert!(M21WatcherCombatStart::Supplied(supplied(&[(*id, 0)], &[RelicId::PureWater])).normalized().is_err(),
            "M21 admission rejects {id:?}");
    }
    for id in &relics {
        assert!(M21WatcherCombatStart::Supplied(supplied(&[(CardId::Vigilance, 0)], &[*id])).normalized().is_err(),
            "M21 admission rejects {id:?}");
    }

    let scenarios = [RoomScenario::M22WatcherAct1, RoomScenario::M23WatcherAct2, RoomScenario::M24WatcherAct3];
    let mut old_runs = scenarios.map(|scenario| Run::new_room_scenario(
        RunConfig { character: Character::Watcher, ascension: 0 }, history(),
        RunRng::new([53; 32], Partition::Test, 0), scenario,
    ).unwrap());
    for (scenario, base) in scenarios.into_iter().zip(old_runs.iter_mut()) {
        for id in &cards {
            let before = base.clone();
            assert!(mutations::obtain_card(&mut base.state, *id, 0).is_err(), "{scenario:?} admission rejects {id:?}");
            assert_eq!(*base, before, "rejected {id:?} acquisition commits nothing");
            let mut entry = before.clone();
            let instance = entry.state.next_instance;
            entry.state.next_instance += 1;
            entry.state.deck.push(CardInstance { misc: 0, instance, id: *id, upgrades: 0 });
            assert!(entry.enter_owned(EncounterId::Cultist, CombatOrigin::Training { kind: MapNodeKind::Monster }).is_err(),
                "{scenario:?} combat entry rejects {id:?}");
            assert!(entry.state.combat.is_none() && entry.state.active_combat.is_none());
        }
        for relic in &relics {
            let mut entry = base.clone();
            entry.state.relics.push(*relic);
            assert!(entry.enter_owned(EncounterId::Cultist, CombatOrigin::Training { kind: MapNodeKind::Monster }).is_err(),
                "{scenario:?} combat entry rejects {relic:?}");
            assert!(entry.state.combat.is_none() && entry.state.active_combat.is_none());
        }
    }
    let startup = |relics: Vec<RelicId>| CombatItemContext::new(relics, vec![None; 3])
        .with_startup_profile(CombatStartupProfile::M21Watcher);
    for relic in relics {
        for context in [
            startup(vec![relic]),
            CombatItemContext::new(vec![relic], vec![None; 3]).with_m22_profile(),
            CombatItemContext::new(vec![relic], vec![None; 3]).with_m22_profile(),
            CombatItemContext::new(vec![relic], vec![None; 3]).with_m24_profile(),
        ] {
            assert!(crate::items::validate_combat_inventory(&context, Character::Watcher).is_err(),
                "older inventory validator rejects {relic:?}");
        }
    }
}

fn m25_foreign_influence_flying_knee(master_reality: bool, fasting: bool) -> Run {
    (0u16..=255).find_map(|seed| {
        let mut cards = vec![(CardId::ForeignInfluence, 0)];
        let master_reality_index = if master_reality {
            let index = cards.len();
            cards.push((CardId::MasterReality, 1));
            Some(index)
        } else {
            None
        };
        let fasting_index = if fasting {
            let index = cards.len();
            cards.push((CardId::Fasting, 1));
            Some(index)
        } else {
            None
        };
        let mut run = m25_run_with_root([seed as u8; 32]);
        choose_room(&mut run, RoomChoiceKind::NeowMaxHp { amount: 7 });
        let instances = install_deck(&mut run, &cards);
        enter_owned(&mut run, EncounterId::Cultist);
        if let Some(index) = master_reality_index {
            play_card(&mut run, instances[index], None);
        }
        play_card(&mut run, instances[0], None);
        let offer = run.decision().unwrap().combat.unwrap().card_offer?;
        let choice = offer.cards.iter().position(|id| *id == CardId::FlyingKnee)?;
        run.choose_combat(CombatAction::ChooseCardOffer { index: choice as u32 }).unwrap();
        let knee = run.decision().unwrap().combat.unwrap().hand.into_iter()
            .find(|card| card.id == CardId::FlyingKnee)
            .expect("Foreign Influence delivers the selected Flying Knee into hand");
        assert_eq!(knee.upgrades, if master_reality { 1 } else { 0 }, "Master Reality controls the generated Flying Knee upgrade");
        if let Some(index) = fasting_index {
            play_card(&mut run, instances[index], None);
            let public = run.decision().unwrap().combat.unwrap();
            assert!(public.player.powers.iter().any(|power| power.id.0 == "energy_down" && power.amount == 1),
                "played Fasting contributes literal energy_down 1");
        }
        play_card(&mut run, knee.instance.0, Some(TargetId(1)));
        Some(run)
    }).expect("the fixed 256-seed fixture set offers Flying Knee from Foreign Influence")
}

#[test]
fn m25_watcher_energized_from_foreign_attack_next_turn() {
    for (master_reality, fasting, next_energy, following_energy) in [
        (false, false, 4, 3),
        (true, false, 4, 3),
        (true, true, 3, 2),
    ] {
        let mut run = m25_foreign_influence_flying_knee(master_reality, fasting);
        let decision = run.decision().unwrap();
        assert!(decision.combat.is_some(), "combat must remain active after Flying Knee: {decision:?}");
        let before_recharge = decision.combat.unwrap();
        assert_eq!(before_recharge.turn, 1);
        assert_eq!(before_recharge.player.max_energy, 3, "the finite M25 Run starts at the requested three-energy baseline");
        assert_eq!(before_recharge.previous_card_type, Some(CardType::Attack), "Run entry retains the active M25 Watcher profile");
        assert!(before_recharge.player.powers.iter().any(|power| power.id.0 == "energized" && power.amount == 1),
            "Flying Knee grants literal Energized 1 before recharge");

        run.choose_combat(CombatAction::EndTurn).unwrap();
        let next = run.decision().unwrap().combat.unwrap();
        assert_eq!(next.turn, 2);
        assert_eq!(next.player.energy, next_energy, "next-turn Watcher energy with master_reality={master_reality}, fasting={fasting}");
        assert!(!next.player.powers.iter().any(|power| matches!(power.id.0, "energized" | "energized_blue")),
            "recharge consumes Energized after applying it");

        run.choose_combat(CombatAction::EndTurn).unwrap();
        let following = run.decision().unwrap().combat.unwrap();
        assert_eq!(following.turn, 3);
        assert_eq!(following.player.energy, following_energy,
            "turn after consumed Energized returns to the literal base with energy_down={fasting}");
        assert!(!following.player.powers.iter().any(|power| matches!(power.id.0, "energized" | "energized_blue")),
            "no Energized remains on the following turn");
    }
}

#[test]
fn m25_public_view_previous_card_type() {
    let mut m25 = m25_run();
    let instances = install_deck(&mut m25, &[(CardId::StrikePurple, 0), (CardId::DefendPurple, 0)]);
    enter_owned(&mut m25, EncounterId::Cultist);
    assert_eq!(m25.decision().unwrap().combat.unwrap().previous_card_type, None,
        "M25 Watcher has no previous card before a play");
    play_card(&mut m25, instances[0], Some(TargetId(1)));
    assert_eq!(m25.decision().unwrap().combat.unwrap().previous_card_type, Some(CardType::Attack));
    play_card(&mut m25, instances[1], None);
    assert_eq!(m25.decision().unwrap().combat.unwrap().previous_card_type, Some(CardType::Skill));
    m25.choose_combat(CombatAction::EndTurn).unwrap();
    assert_eq!(m25.decision().unwrap().combat.unwrap().previous_card_type, Some(CardType::Skill),
        "previous card type persists across turns within the combat");

    let mut m24 = Run::new_room_scenario(
        RunConfig { character: Character::Watcher, ascension: 0 },
        history(),
        RunRng::new([59; 32], Partition::Test, 0),
        RoomScenario::M24WatcherAct3,
    ).unwrap();
    let instances = install_deck(&mut m24, &[(CardId::StrikePurple, 0), (CardId::DefendPurple, 0)]);
    enter_owned(&mut m24, EncounterId::Cultist);
    assert_eq!(m24.decision().unwrap().combat.unwrap().previous_card_type, None,
        "M24 Watcher omits previous card type before play");
    play_card(&mut m24, instances[0], Some(TargetId(1)));
    assert_eq!(m24.decision().unwrap().combat.unwrap().previous_card_type, None,
        "M24 Watcher keeps previous card type absent after an Attack");
    play_card(&mut m24, instances[1], None);
    assert_eq!(m24.decision().unwrap().combat.unwrap().previous_card_type, None,
        "M24 Watcher keeps previous card type absent after a Skill");
    m24.choose_combat(CombatAction::EndTurn).unwrap();
    assert_eq!(m24.decision().unwrap().combat.unwrap().previous_card_type, None,
        "M24 Watcher keeps previous card type absent across turns");
}
