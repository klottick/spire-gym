//! M25 Watcher × Act4 and generated-card × Act3 compositions (Task C7).
//! Fixtures use the real `enemies::setup` entry, M25 Watcher cards/item context,
//! and legal `step_full` actions. Literal values cite the Watcher research,
//! M10 Act4/Heart research, implementation notes, and relic records.
use super::super::*;
use crate::{
    content::{CardId, Character, EncounterId, PotionId, RelicId},
    items::CombatItemContext,
    run::RunRng,
};
use CardId::*;

fn entry(encounter: EncounterId, act: u8) -> CombatEntry {
    enemies::setup(
        encounter,
        act,
        0,
        &mut RunRng::new([0xC7; 32], rng::Partition::Test, 0),
        enemies::SetupContext {
            player: PlayerState { hp: 300, max_hp: 300, block: 0, energy: 9, max_energy: 9, draw_per_turn: 5, powers: vec![] },
            player_powers: vec![],
            character: Character::Watcher,
            cards: vec![permanent_live_card(StrikePurple, CardInstanceId(1), 0)],
            opening_draw: 1,
            encounter_visit: 0,
        },
    ).expect("qualified real encounter setup")
}

fn watcher_with(
    mut e: CombatEntry,
    deck: &[(CardId, u32)],
    opening_draw: usize,
    energy: i32,
    draw_per_turn: i32,
    relics: Vec<RelicId>,
    potions: Vec<Option<PotionId>>,
) -> Combat {
    e.setup.character = Character::Watcher;
    e.setup.player.energy = energy;
    e.setup.player.max_energy = energy;
    e.setup.player.draw_per_turn = draw_per_turn;
    e.cards = deck.iter().enumerate().map(|(i, (id, upgrades))|
        m25_watcher_live_card(*id, CardInstanceId(i as u64 + 1), *upgrades).unwrap()).collect();
    e.opening_draw = opening_draw.min(deck.len());
    let mut items = Some(CombatItemContext::new(relics, potions).with_m25_profile());
    let combat = Combat::enter_with_item_context(e, &mut items).unwrap();
    assert!(items.is_none(), "the shared item-context owner consumes M25 admission");
    combat
}

fn watcher(encounter: EncounterId, act: u8, deck: &[(CardId, u32)], energy: i32) -> Combat {
    watcher_with(entry(encounter, act), deck, deck.len(), energy, 5, vec![], vec![None; 3])
}

fn heart(deck: &[(CardId, u32)], energy: i32) -> Combat {
    watcher(EncounterId::CorruptHeart, 4, deck, energy)
}

fn play(c: &mut Combat, instance: u64, target: Option<u64>) {
    c.step_full(&CombatAction::PlayCard { instance, target: target.map(TargetId) })
        .unwrap_or_else(|error| panic!("legal play {instance} failed: {error:?}; actions={:?}", c.legal_actions_full()));
}

fn card_instance(c: &Combat, id: CardId) -> u64 {
    [&c.state.piles.hand, &c.state.piles.draw, &c.state.piles.discard, &c.state.piles.exhaust]
        .into_iter().flat_map(|pile| pile.iter()).filter_map(model::OwnedCard::full)
        .find(|card| card.id == id).expect("fixture card exists").instance.0
}

fn play_card(c: &mut Combat, id: CardId, target: Option<u64>) {
    let instance = card_instance(c, id);
    if !c.state.piles.hand.iter().any(|card| card.handle() == card_state::handle(CardInstanceId(instance))) {
        move_card(c, instance, SelectionSourceZone::Hand);
    }
    play(c, instance, target);
}

fn end(c: &mut Combat) {
    c.step_full(&CombatAction::EndTurn).unwrap();
}

fn power(c: &Combat, slot: usize, id: &str) -> i32 {
    c.state.enemies[slot].full_powers.iter().find(|p| p.id.0 == id).map_or(0, |p| p.state.amount)
}

fn player_power(c: &Combat, id: &str) -> i32 {
    c.state.player_full_powers.iter().find(|p| p.id.0 == id).map_or(0, |p| p.state.amount)
}

fn full_power(id: &'static str, amount: i32) -> FullPowerEntry {
    FullPowerEntry { id: FullPowerId(id), state: PowerState { amount, skip_first: false, monster_source_grace: false } }
}

fn hook_count(c: &Combat, hook: Hook) -> usize {
    c.state.hook_events.iter().filter(|event| event.hook == hook).count()
}

fn add_generated(c: &mut Combat, id: CardId, instance: u64, upgrades: u32) {
    let card = if crate::content::watcher::supports_m25_generated(id) {
        m25_watcher_live_card(id, CardInstanceId(instance), upgrades).expect("M25 generated child")
    } else {
        card_state::generated(id, CardInstanceId(instance), upgrades).expect("generated status")
    };
    c.state.card_ids_issued.insert(card_state::handle(card.instance));
    c.state.piles.hand.push(model::OwnedCard::Full(card));
    piles::refresh_hand_values(&mut c.state);
}

fn move_card(c: &mut Combat, instance: u64, zone: SelectionSourceZone) {
    let handle = card_state::handle(CardInstanceId(instance));
    let take = |pile: &mut Vec<model::OwnedCard>| pile.iter().position(|card| card.handle() == handle).map(|i| pile.remove(i));
    let card = take(&mut c.state.piles.hand)
        .or_else(|| take(&mut c.state.piles.draw))
        .or_else(|| take(&mut c.state.piles.discard))
        .or_else(|| take(&mut c.state.piles.exhaust))
        .expect("fixture card custody");
    match zone {
        SelectionSourceZone::Hand => c.state.piles.hand.push(card),
        SelectionSourceZone::Draw => c.state.piles.draw.push(card),
        SelectionSourceZone::Discard => c.state.piles.discard.push(card),
        _ => unreachable!("only visible combat piles are arranged here"),
    }
    piles::refresh_hand_values(&mut c.state);
}

fn set_pair_intent(c: &mut Combat, slot: usize, name: &'static str) {
    let history = match (slot, name) {
        (0, "shield_smash") => vec!["shield_bash", "shield_fortify", "shield_smash"],
        (0, "shield_fortify") => vec!["shield_fortify"],
        (0, _) => vec![name],
        (1, "spear_strength") => vec!["spear_burn_strike", "spear_skewer", "spear_strength"],
        (1, "spear_skewer") => vec!["spear_burn_strike", "spear_skewer"],
        (1, _) => vec![name],
        _ => unreachable!(),
    }.into_iter().map(|move_id| ActorIntentId::Full(FullIntentId::from_name(move_id))).collect::<Vec<_>>();
    let selected = history.last().unwrap().clone();
    c.state.enemies[slot].move_history = history;
    c.state.enemies[slot].intent_id = selected.clone();
    c.state.enemies[slot].selected_move = selected;
    let count = c.state.enemies[slot].move_history.len() as u32;
    c.state.enemies[slot].full_memory = Some(if slot == 0 {
        EnemyMemory::SpireShield { selections: count }
    } else {
        EnemyMemory::SpireSpear { selections: count }
    });
    effects::refresh_enemy_values(&mut c.state).unwrap();
}

fn set_heart_cursor(c: &mut Combat, selection_count: u32, buff_count: u32) {
    // Same typed history/memory arrangement as the accepted M10 Heart fixture;
    // the refresh keeps derived enemy values synchronized with the selected move.
    let first_attack = (selection_count > 0).then_some(true);
    let mut history = vec![FullIntentId::HeartDebilitate];
    for selected in 0..selection_count {
        history.push(match selected % 3 {
            0 => FullIntentId::HeartBloodShots,
            1 => FullIntentId::HeartEcho,
            _ => FullIntentId::HeartBuff,
        });
    }
    let selected = *history.last().unwrap();
    let enemy = &mut c.state.enemies[0];
    enemy.intent_id = ActorIntentId::Full(selected);
    enemy.selected_move = enemy.intent_id.clone();
    enemy.move_history = history.into_iter().map(ActorIntentId::Full).collect();
    enemy.full_memory = Some(EnemyMemory::CorruptHeart {
        selection_count,
        first_attack,
        buff_count,
        last_buff_selection: if buff_count > 0 { Some(buff_count * 3 - 1) } else { None },
    });
    effects::refresh_enemy_values(&mut c.state).unwrap();
}

#[test]
fn m25_t01_beat_of_death_counts_watcher_plays() {
    // Heart is A0/750 HP with Beat of Death 1; each CardAfterUse queues one
    // THORNS packet. Source: docs/research/2026-10-24-m10-act4-and-ironclad.md:174-179;
    // children and play effects: docs/research/2026-10-19-act1-content-watcher.md:164-200.
    for (id, target) in [
        (Miracle, None), (Insight, None), (Smite, Some(1)), (Safety, None),
        (ThroughViolence, Some(1)), (Expunger, Some(1)),
    ] {
        let mut c = heart(&[(DefendPurple, 0)], 9);
        add_generated(&mut c, id, 20, 0);
        if id == Expunger {
            c.state.piles.hand.iter_mut().filter_map(model::OwnedCard::full_mut)
                .find(|card| card.instance.0 == 20).unwrap().misc = 3;
        }
        let hp = c.state.player.hp;
        play(&mut c, 20, target);
        if id == Safety {
            assert_eq!(c.state.player.hp, hp, "Safety's Block absorbs its one Beat of Death");
            assert_eq!(c.state.player.block, 11, "Safety's 12 Block is reduced by one Beat packet");
        } else {
            assert_eq!(c.state.player.hp, hp - 1, "one Beat for generated card {id:?}");
        }
    }

    // A real Foreign Influence use and its selected fresh output each own one play.
    let mut fi = heart(&[(ForeignInfluence, 0)], 9);
    play_card(&mut fi, ForeignInfluence, None);
    fi.state.silent.card_offer.as_mut().unwrap().cards = vec![SwiftStrike, Anger, BodySlam];
    fi.step_full(&CombatAction::ChooseCardOffer { index: 0 }).unwrap();
    let output = fi.state.piles.hand.iter().filter_map(model::OwnedCard::full)
        .find(|card| card.id == SwiftStrike).unwrap().instance.0;
    let hp = fi.state.player.hp;
    play(&mut fi, output, Some(1));
    assert_eq!(fi.state.player.hp, hp - 1, "Foreign Influence output is a second actual play");
    assert_eq!(hook_count(&fi, Hook::CardPlayed), 2);

    // Omniscience itself and each of its two queued plays count; its choice action does not.
    let mut omni = heart(&[(Omniscience, 0), (StrikePurple, 0)], 9);
    move_card(&mut omni, 2, SelectionSourceZone::Draw);
    let before = omni.state.player.hp;
    play_card(&mut omni, Omniscience, None);
    omni.step_full(&CombatAction::SelectCard { instance: 2 }).unwrap();
    assert_eq!(omni.state.player.hp, before - 3);
    assert_eq!(hook_count(&omni, Hook::CardPlayed), 3);

    // Scry, potions and stance changes are not extra plays. Flurry and Tantrum each
    // count once; returning Flurry does not add another play.
    let mut scry = heart(&[(CutThroughFate, 0), (StrikePurple, 0)], 9);
    move_card(&mut scry, 2, SelectionSourceZone::Draw);
    let hp = scry.state.player.hp;
    play_card(&mut scry, CutThroughFate, Some(1));
    assert_eq!(scry.state.player.hp, hp, "Beat waits for Cut Through Fate's Scry continuation to finish");
    scry.step_full(&CombatAction::ConfirmSelection).unwrap();
    assert_eq!(scry.state.player.hp, hp - 1, "Scry continuation is not a CardAfterUse");

    let mut potion = watcher_with(entry(EncounterId::CorruptHeart, 4), &[(DefendPurple, 0)], 1, 9, 5,
        vec![], vec![Some(PotionId::StancePotion), None, None]);
    let hp = potion.state.player.hp;
    potion.step_full(&CombatAction::UsePotion { slot: 0, target: None }).unwrap();
    potion.step_full(&CombatAction::ChooseStance { stance: Stance::Calm }).unwrap();
    assert_eq!(potion.state.player.hp, hp, "a potion and its stance choice do not trigger Beat of Death");
    assert_eq!(hook_count(&potion, Hook::CardPlayed), 0);

    let mut tantrum = heart(&[(FlurryOfBlows, 0), (Tantrum, 0)], 9);
    let hp = tantrum.state.player.hp;
    play_card(&mut tantrum, FlurryOfBlows, Some(1));
    play_card(&mut tantrum, Tantrum, Some(1));
    assert_eq!(tantrum.state.player.hp, hp - 2, "Tantrum's stance change and Flurry return are not extra plays");
    assert!(tantrum.state.piles.hand.iter().any(|card| card.full().is_some_and(|live| live.id == FlurryOfBlows)));
    assert_eq!(hook_count(&tantrum, Hook::CardPlayed), 2);

    // Scrawl draws Deus Ex Machina: its draw callback creates Miracles without a play callback.
    let mut deus = heart(&[(Scrawl, 0), (DefendPurple, 0)], 9);
    let card = m25_watcher_live_card(DeusExMachina, CardInstanceId(20), 0).unwrap();
    deus.state.card_ids_issued.insert(card_state::handle(CardInstanceId(20)));
    deus.state.piles.draw.push(model::OwnedCard::Full(card));
    let hp = deus.state.player.hp;
    play_card(&mut deus, Scrawl, None);
    assert_eq!(deus.state.player.hp, hp - 1);
    assert_eq!(deus.state.piles.hand.iter().filter_map(model::OwnedCard::full).filter(|card| card.id == Miracle).count(), 2);
    assert_eq!(hook_count(&deus, Hook::CardPlayed), 1, "drawing Deus Ex Machina is not another play");

    // A lethal Beat shares Death's terminal cancellation: Echo Form's repeated Tantrum never starts.
    let mut lethal = heart(&[(Tantrum, 0)], 9);
    lethal.state.player.hp = 1;
    lethal.state.player_full_powers.push(full_power("echo_form", 1));
    play_card(&mut lethal, Tantrum, Some(1));
    assert_eq!(lethal.state.outcome, Some(Outcome::Death));
    assert_eq!(hook_count(&lethal, Hook::CardPlayed), 1);
    assert!(engine::validate_runtime(&lethal.state).is_ok());
}

#[test]
fn m25_t02_invincible_allowance() {
    // Literal A0 quota/reset 300 and no damage-type exclusion: M10 source review
    // docs/research/2026-10-24-m10-act4-and-ironclad.md:154-179. Watcher values:
    // research docs/research/2026-10-19-act1-content-watcher.md:83,87,150,169-171.
    let mut c = heart(&[
        (Tantrum, 0), (Worship, 0), (Ragnarok, 0), (FlyingSleeves, 0),
        (PressurePoints, 0), (Omega, 0),
    ], 20);
    c.state.character.stance = Stance::Wrath;
    c.state.player_full_powers.push(full_power("mantra", 5));
    assert_eq!(power(&c, 0, "invincible"), 300);
    play_card(&mut c, Tantrum, Some(1));
    assert_eq!(power(&c, 0, "invincible"), 282, "Wrath Tantrum is three separate 6-damage hits");
    play_card(&mut c, Worship, None);
    assert_eq!(c.state.character.stance, Stance::Divinity);
    play_card(&mut c, Ragnarok, None);
    assert_eq!(power(&c, 0, "invincible"), 207, "Divinity multiplies Ragnarok's five 5-damage hits");
    play_card(&mut c, FlyingSleeves, Some(1));
    assert_eq!(power(&c, 0, "invincible"), 183, "two Divinity Flying Sleeves hits spend the same pool");
    play_card(&mut c, PressurePoints, Some(1));
    assert_eq!(power(&c, 0, "invincible"), 175, "Pressure Points Mark HP loss spends the same pool");
    play_card(&mut c, Omega, None);
    end(&mut c);
    assert_eq!(power(&c, 0, "invincible"), 300, "the allowance resets at the Heart's shared monster-start checkpoint");

    // F9: Judgment is an instant-kill threshold, not damage, and does not spend Invincible.
    let mut judgment = heart(&[(Judgment, 0)], 9);
    judgment.state.enemies[0].hp = 30;
    play_card(&mut judgment, Judgment, Some(1));
    assert_eq!((judgment.state.enemies[0].hp, power(&judgment, 0, "invincible"), judgment.state.outcome),
        (0, 300, Some(Outcome::Victory)));
}

#[test]
fn m25_t03_debilitate_statuses_draw_and_scry() {
    // The Heart's first selected Debilitate adds exactly Dazed/Slimed/Wound/Burn/Void
    // through ShuffledDraw; see docs/research/2026-10-24-m10-act4-and-ironclad.md:147-153.
    let mut c = watcher_with(entry(EncounterId::CorruptHeart, 4),
        &[(ThirdEye, 0), (Meditate, 0), (Scrawl, 0)], 3, 9, 5, vec![], vec![None; 3]);
    c.state.player_full_powers.push(full_power("no_draw", 1));
    set_heart_cursor(&mut c, 0, 0);
    end(&mut c);
    for id in [Dazed, Slimed, Wound, Burn, Void] {
        assert_eq!(piles::owned_full_cards(&c.state).filter(|card| card.id == id).count(), 1,
            "Heart Debilitate inserts one {id:?}");
    }
    let wound = piles::owned_full_cards(&c.state).find(|card| card.id == Wound).unwrap().instance.0;
    move_card(&mut c, wound, SelectionSourceZone::Draw);
    for id in [Dazed, Slimed, Burn, Void] {
        let instance = piles::owned_full_cards(&c.state).find(|card| card.id == id).unwrap().instance.0;
        move_card(&mut c, instance, SelectionSourceZone::Draw);
    }
    // Scry exposes the top three in the real selection window; it moves no card until confirmed.
    move_card(&mut c, 1, SelectionSourceZone::Hand);
    let wound_card = c.state.piles.draw.iter().position(|card| card.handle() == card_state::handle(CardInstanceId(wound))).unwrap();
    let top = c.state.piles.draw.remove(wound_card);
    c.state.piles.draw.push(top);
    let energy = c.state.player.energy;
    play_card(&mut c, ThirdEye, None);
    let selection = c.public_full().unwrap().selection.unwrap();
    assert_eq!(selection.operation, SelectionOperation::Scry);
    assert_eq!(selection.candidates.len(), 3);
    assert!(selection.candidates.iter().any(|card| card.instance.0 == wound));
    assert!(c.state.piles.draw.iter().any(|card| card.handle() == card_state::handle(CardInstanceId(wound))));
    assert_eq!(c.state.player.energy, energy - 1, "Third Eye costs one; revealing Void through Scry costs no extra Energy");
    c.step_full(&CombatAction::SelectCard { instance: wound }).unwrap();
    c.step_full(&CombatAction::ConfirmSelection).unwrap();
    assert!(c.state.piles.discard.iter().any(|card| card.handle() == card_state::handle(CardInstanceId(wound))));

    // Meditate can retrieve that Scry-discarded Status and retain the exact instance.
    move_card(&mut c, 2, SelectionSourceZone::Hand);
    play_card(&mut c, Meditate, None);
    c.step_full(&CombatAction::SelectCard { instance: wound }).unwrap();
    c.step_full(&CombatAction::ConfirmSelection).unwrap();
    assert!(c.state.piles.hand.iter().any(|card| card.handle() == card_state::handle(CardInstanceId(wound))),
        "Meditate's selected Wound survives the end turn in the same hand instance");
    assert_eq!(c.state.turn, 3, "Meditate requests its end turn after the current one");

    // Scrawl fills from the remaining Heart-created status pile; actual Void draw loses one Energy.
    move_card(&mut c, 3, SelectionSourceZone::Hand);
    play_card(&mut c, Scrawl, None);
    assert!(c.state.piles.hand.iter().filter_map(model::OwnedCard::full)
        .any(|card| matches!(card.id, Dazed | Slimed | Burn | Wound)));

    // Calm exit +2, Violet Lotus +1, Divinity entry +3, Scrawl cost 1 and
    // actual Void draw −1: 9−2+2+1−2+3−1−1=9.
    let mut energy = watcher_with(entry(EncounterId::CorruptHeart, 4),
        &[(Vigilance, 0), (Worship, 0), (Scrawl, 0)], 3, 9, 5,
        vec![RelicId::VioletLotus], vec![None; 3]);
    let void = card_state::generated(Void, CardInstanceId(20), 0).unwrap();
    energy.state.card_ids_issued.insert(card_state::handle(CardInstanceId(20)));
    energy.state.piles.draw.push(model::OwnedCard::Full(void));
    energy.state.player_full_powers.push(full_power("mantra", 5));
    play_card(&mut energy, Vigilance, None);
    assert_eq!(energy.state.player.energy, 7);
    play_card(&mut energy, Worship, None);
    assert_eq!(energy.state.character.stance, Stance::Divinity);
    assert_eq!(energy.state.player.energy, 11);
    play_card(&mut energy, Scrawl, None);
    assert_eq!(energy.state.player.energy, 9, "Scrawl and the drawn Void each spend one after stance and relic gains");
    assert!(energy.state.piles.hand.iter().any(|card| card.handle() == card_state::handle(CardInstanceId(20))),
        "Void remains in hand after its draw-time Energy loss");
}

#[test]
fn m25_t04_painful_stabs_and_block_return() {
    // Heart Blood Shots are 12 normal hits of 2; Painful Stabs adds one Wound only
    // after each HP-losing hit. F11 binds Block Return per committed player Attack hit;
    // docs/research/2026-10-24-m10-act4-and-ironclad.md:147-156 and design §5.9.
    let deck = [(DefendPurple, 0); 20];
    let mut pain = watcher_with(entry(EncounterId::CorruptHeart, 4), &deck, 10, 9, 5, vec![], vec![None; 3]);
    pain.state.enemies[0].full_powers.push(full_power("painful_stabs", 1));
    pain.state.player.block = 2;
    set_heart_cursor(&mut pain, 1, 0);
    let pain_turn = pain.step_full_observed(&CombatAction::EndTurn);
    pain_turn.result.unwrap();
    assert_eq!(pain.state.player.hp, 278, "the first two-point hit is fully blocked; eleven later hits lose HP");
    assert_eq!(power(&pain, 0, "painful_stabs"), 1, "the fixture retains Heart Painful Stabs through Blood Shots");
    assert_eq!(piles::owned_full_cards(&pain.state).filter(|card| card.id == Wound).count(), 11,
        "each of the eleven HP-losing normal hits creates one Wound; resolved={:?}",
        pain_turn.effects.iter().filter(|effect| matches!(effect.kind, observe::ResolvedEffectKind::Damage { .. })).collect::<Vec<_>>());
    assert_eq!(pain.state.piles.discard.iter().filter_map(model::OwnedCard::full).filter(|card| card.id == Wound).count(), 11);

    let mut block_return = heart(&[(TalkToTheHand, 0), (Tantrum, 0)], 9);
    block_return.state.enemies[0].full_powers.iter_mut().find(|power| power.id.0 == "beat_of_death").unwrap().state.amount = 0;
    play_card(&mut block_return, TalkToTheHand, Some(1));
    block_return.state.enemies[0].full_powers.iter_mut().find(|power| power.id.0 == "invincible").unwrap().state.amount = 1;
    play_card(&mut block_return, Tantrum, Some(1));
    assert_eq!((block_return.state.player.block, power(&block_return, 0, "invincible")), (6, 0),
        "three Tantrum hit callbacks each return two Block, including Invincible-capped hits");
}

#[test]
fn m25_t05_heart_artifact_and_watcher_debuffs() {
    // First Heart buff installs Artifact 2; Watcher debuffs spend each charge in
    // card order. Values and classifications: Heart source findings above and
    // docs/research/2026-10-19-act1-content-watcher.md:87,108,124,132,139.
    let mut c = heart(&[
        (StrikePurple, 0), (SashWhip, 0), (DefendPurple, 0), (CrushJoints, 0),
        (Tantrum, 0), (Indignation, 0), (WaveOfTheHand, 0), (Evaluate, 0), (PressurePoints, 0),
    ], 20);
    set_heart_cursor(&mut c, 3, 0);
    end(&mut c);
    assert_eq!(power(&c, 0, "artifact"), 2);
    play_card(&mut c, StrikePurple, Some(1));
    play_card(&mut c, SashWhip, Some(1));
    assert_eq!((power(&c, 0, "artifact"), power(&c, 0, "weak")), (1, 0), "Sash Whip's Weak consumes charge one");
    play_card(&mut c, DefendPurple, None);
    play_card(&mut c, CrushJoints, Some(1));
    assert_eq!((power(&c, 0, "artifact"), power(&c, 0, "vulnerable")), (0, 0), "Crush Joints consumes charge two");
    play_card(&mut c, Tantrum, Some(1));
    play_card(&mut c, Indignation, None);
    assert_eq!(power(&c, 0, "vulnerable"), 3, "Indignation in Wrath applies after Artifact is spent");
    play_card(&mut c, WaveOfTheHand, None);
    play_card(&mut c, Evaluate, None);
    assert_eq!(power(&c, 0, "weak"), 1, "Wave of the Hand applies Weak on a positive Block gain");
    play_card(&mut c, PressurePoints, Some(1));
    assert_eq!(power(&c, 0, "mark"), 8, "Pressure Points Mark applies after Artifact is spent");

    let mut blasphemy = heart(&[(Blasphemy, 0)], 9);
    blasphemy.state.player_full_powers.push(full_power("artifact", 1));
    play_card(&mut blasphemy, Blasphemy, None);
    assert_eq!((player_power(&blasphemy, "blasphemer"), player_power(&blasphemy, "artifact")), (1, 1),
        "Blasphemer is a positive BUFF and does not spend player Artifact");
}

#[test]
fn m25_t06_shield_spear_facing() {
    // A0 formation is Shield-left 110 HP, Spear-right 160 HP; manual targeting
    // sets facing, ALL_ENEMY/random/power effects do not. Sources: M10 Act4
    // research:108-135 and data/enemies/enemies.json:7308-7430,7445-7565.
    let mut c = watcher(EncounterId::ShieldAndSpear, 4,
        &[(StrikePurple, 0), (Consecrate, 0), (Ragnarok, 0), (PressurePoints, 0)], 20);
    assert_eq!(c.state.enemies.iter().map(|enemy| enemy.hp).collect::<Vec<_>>(), [110, 160]);
    play_card(&mut c, StrikePurple, Some(2));
    assert_eq!(c.state.facing, Some(PlayerFacing::Right));
    let before = c.state.enemies.iter().map(|enemy| enemy.hp).collect::<Vec<_>>();
    play_card(&mut c, Consecrate, None);
    assert_eq!((before[0] - c.state.enemies[0].hp, before[1] - c.state.enemies[1].hp), (5, 5));
    assert_eq!(c.state.facing, Some(PlayerFacing::Right), "Consecrate's ALL_ENEMY path does not write facing");
    let before = c.state.enemies.iter().map(|enemy| enemy.hp).sum::<i32>();
    play_card(&mut c, Ragnarok, None);
    assert_eq!(before - c.state.enemies.iter().map(|enemy| enemy.hp).sum::<i32>(), 25,
        "five separate random Ragnarok hits total five base five-damage packets");
    assert_eq!(c.state.facing, Some(PlayerFacing::Right), "random attack targets do not turn the player");

    // The targeted Pressure Points selection faces Shield; its all-target Mark trigger keeps it.
    c.state.enemies[1].full_powers.push(full_power("mark", 6));
    let left_before = c.state.enemies[0].hp;
    let right_before = c.state.enemies[1].hp;
    play_card(&mut c, PressurePoints, Some(1));
    assert_eq!(c.state.facing, Some(PlayerFacing::Left));
    assert_eq!((left_before - c.state.enemies[0].hp, right_before - c.state.enemies[1].hp), (0, 6),
        "Shield Artifact blocks the new Mark while the existing Spear Mark still triggers");

    // Omega and Conclude's all-target paths leave the previous facing untouched.
    let mut omega = watcher(EncounterId::ShieldAndSpear, 4, &[(DefendPurple, 0)], 9);
    omega.state.facing = Some(PlayerFacing::Right);
    omega.state.player_full_powers.push(full_power("omega", 1));
    set_pair_intent(&mut omega, 0, "shield_fortify");
    set_pair_intent(&mut omega, 1, "spear_strength");
    end(&mut omega);
    assert_eq!(omega.state.facing, Some(PlayerFacing::Right));

    let mut conclude = watcher(EncounterId::ShieldAndSpear, 4, &[(Conclude, 0)], 9);
    conclude.state.facing = Some(PlayerFacing::Right);
    set_pair_intent(&mut conclude, 0, "shield_fortify");
    set_pair_intent(&mut conclude, 1, "spear_strength");
    play_card(&mut conclude, Conclude, None);
    assert_eq!(conclude.state.facing, Some(PlayerFacing::Right));

    let mut death = watcher(EncounterId::ShieldAndSpear, 4, &[(StrikePurple, 0)], 9);
    death.state.enemies[0].hp = 6;
    play_card(&mut death, StrikePurple, Some(1));
    assert_eq!((death.state.enemies[0].hp, death.state.facing, player_power(&death, "surrounded"), power(&death, 1, "back_attack")),
        (0, Some(PlayerFacing::Right), 0, 0), "partner death faces the survivor and removes Surrounded/Back Attack");
}

#[test]
fn m25_t07_shield_bash_and_watcher_strength() {
    // Named Shield Bash without active Orbs applies Strength−1; Fasting and
    // Become Almighty use their shared positive-Strength owner. Source:
    // docs/research/2026-10-24-m10-act4-and-ironclad.md:125-135 and
    // docs/research/2026-10-19-act1-content-watcher.md:104-106; Wish values :190-200.
    let mut c = watcher(EncounterId::ShieldAndSpear, 4, &[(Fasting, 0), (Wish, 0)], 9);
    set_pair_intent(&mut c, 0, "shield_bash");
    set_pair_intent(&mut c, 1, "spear_strength");
    end(&mut c);
    assert_eq!(player_power(&c, "strength"), -1);
    play_card(&mut c, Fasting, None);
    assert_eq!(player_power(&c, "strength"), 2);
    play_card(&mut c, Wish, None);
    c.step_full(&CombatAction::ChooseCardOffer { index: 0 }).unwrap();
    assert_eq!(player_power(&c, "strength"), 5, "Become Almighty composes with Shield Bash and Fasting");

    // A Heart buff clears only its own negative Strength; the player's debuff stays.
    let mut heart = heart(&[(DefendPurple, 0)], 9);
    heart.state.enemies[0].full_powers.push(full_power("strength", -4));
    heart.state.player_full_powers.push(full_power("strength", -4));
    effects::refresh_enemy_values(&mut heart.state).unwrap();
    set_heart_cursor(&mut heart, 3, 0);
    end(&mut heart);
    assert_eq!((power(&heart, 0, "strength"), player_power(&heart, "strength")), (2, -4));
}

#[test]
fn m25_t08_spear_burns_retain_and_wrath() {
    // Spire Spear Burn Strike is 5×2 and creates two Burns in discard; BURN says
    // its end-turn damage is THORNS and discards despite Retain:
    // docs/research/2026-10-24-m10-act4-and-ironclad.md:108-110;
    // data/cards/implementation-notes.json:27-29.
    let deck = [(DefendPurple, 0); 20];
    let mut strike = watcher_with(entry(EncounterId::ShieldAndSpear, 4), &deck, 10, 9, 5, vec![], vec![None; 3]);
    strike.state.facing = Some(PlayerFacing::Right);
    set_pair_intent(&mut strike, 0, "shield_fortify");
    set_pair_intent(&mut strike, 1, "spear_burn_strike");
    end(&mut strike);
    assert_eq!(strike.state.player.hp, 290);
    assert_eq!(piles::owned_full_cards(&strike.state).filter(|card| card.id == Burn).count(), 2,
        "Spear Burn Strike selected and executed; full piles={:?}/{:?}/{:?}/{:?}",
        strike.state.piles.hand, strike.state.piles.draw, strike.state.piles.discard, strike.state.piles.exhaust);
    assert_eq!(strike.state.piles.discard.iter().filter_map(model::OwnedCard::full).filter(|card| card.id == Burn).count(), 2,
        "Burn Strike's two Discard children; piles={:?}/{:?}/{:?}/{:?}",
        strike.state.piles.hand, strike.state.piles.draw, strike.state.piles.discard, strike.state.piles.exhaust);

    let burn_deck = [(DefendPurple, 0); 20];
    let mut burn = watcher_with(entry(EncounterId::ShieldAndSpear, 4), &burn_deck, 10, 9, 5,
        vec![], vec![None; 3]);
    add_generated(&mut burn, Burn, 100, 0);
    burn.state.piles.hand.iter_mut().filter_map(model::OwnedCard::full_mut)
        .find(|card| card.instance.0 == 100).unwrap().local.retain = true;
    burn.state.character.stance = Stance::Wrath;
    set_pair_intent(&mut burn, 0, "shield_fortify");
    set_pair_intent(&mut burn, 1, "spear_strength");
    let ended = burn.step_full_observed(&CombatAction::EndTurn);
    ended.result.unwrap();
    assert_eq!(burn.state.player.hp, 298, "Wrath does not double Burn's self-directed THORNS damage");
    assert!(ended.effects.iter().any(|effect| matches!(effect.kind,
        observe::ResolvedEffectKind::Damage { category: observe::DamageCategory::Thorns, incoming: 2, .. })));
    assert!(burn.state.piles.discard.iter().any(|card| card.handle() == card_state::handle(CardInstanceId(100))));
    assert!(!burn.state.piles.hand.iter().any(|card| card.handle() == card_state::handle(CardInstanceId(100))),
        "Burn discards itself even though the fixture marks it Retain");

    // The shared cleanup also outranks Defect's temporary Equilibrium retention.
    // This reaches the generic DiscardHand branch, beyond Watcher's retained-card owner.
    let mut defect_entry = entry(EncounterId::ShieldAndSpear, 4);
    defect_entry.setup.character = Character::Defect;
    defect_entry.cards = std::iter::once(permanent_live_card(Equilibrium, CardInstanceId(1), 0))
        .chain((2..=20).map(|id| permanent_live_card(Defend, CardInstanceId(id), 0))).collect();
    defect_entry.opening_draw = 10;
    let mut defect = Combat::enter(defect_entry).expect("qualified Defect encounter setup");
    add_generated(&mut defect, Burn, 100, 0);
    play_card(&mut defect, Equilibrium, None);
    defect.state.player.block = 0;
    defect.state.facing = Some(PlayerFacing::Right);
    set_pair_intent(&mut defect, 0, "shield_fortify");
    set_pair_intent(&mut defect, 1, "spear_strength");
    end(&mut defect);
    assert_eq!(defect.state.player.hp, 298, "the shared Burn tick runs before generic hand cleanup");
    assert!(defect.state.piles.discard.iter().any(|card| card.handle() == card_state::handle(CardInstanceId(100))),
        "Burn is forced to discard despite the live Equilibrium temporary-retain owner");
    assert!(!defect.state.piles.hand.iter().any(|card| card.handle() == card_state::handle(CardInstanceId(100))));
    assert!(engine::validate_runtime(&defect.state).is_ok());
}

#[test]
fn m25_t09_vault_against_act4() {
    // Vault's F2 binding runs player end/start effects while skipping enemy and
    // end-of-round work; Heart Invincible resets only at MonsterStart. Sources:
    // data/cards/implementation-notes.json:103-105 and C4/A4 report F2.
    let mut c = heart(&[(Study, 0), (Omega, 0), (LikeWater, 0), (Vault, 0)], 12);
    add_generated(&mut c, Burn, 20, 0);
    c.state.piles.hand.iter_mut().filter_map(model::OwnedCard::full_mut)
        .find(|card| card.instance.0 == 20).unwrap().local.retain = true;
    c.state.character.stance = Stance::Calm;
    assert_eq!(piles::owned_full_cards(&c.state).filter(|card| card.id == Burn).count(), 1);
    play_card(&mut c, Study, None);
    play_card(&mut c, Omega, None);
    play_card(&mut c, LikeWater, None);
    let intent = c.state.enemies[0].intent_id.clone();
    let history = c.state.enemies[0].move_history.clone();
    let player_end_count = hook_count(&c, Hook::PlayerEndTurn);
    let hp_before_vault = c.state.player.hp;
    let vault_instance = card_instance(&c, Vault);
    if !c.state.piles.hand.iter().any(|card| card.handle() == card_state::handle(CardInstanceId(vault_instance))) {
        move_card(&mut c, vault_instance, SelectionSourceZone::Hand);
    }
    let vault_result = c.step_full_observed(&CombatAction::PlayCard { instance: vault_instance, target: None });
    vault_result.result.unwrap();
    assert_eq!(c.state.turn, 2, "Vault starts the extra player turn");
    assert_eq!(c.state.enemies[0].intent_id, intent);
    assert_eq!(c.state.enemies[0].move_history, history);
    assert_eq!(power(&c, 0, "invincible"), 250, "Omega spends 50 allowance; Vault skips the reset");
    assert!(vault_result.effects.iter().any(|effect| matches!(effect.kind,
        observe::ResolvedEffectKind::Block { before: 0, after: 5 })),
        "Like Water resolves its five Block before the extra player start clears it");
    let burn_ticks = vault_result.effects.iter().filter(|effect| matches!(effect.kind,
        observe::ResolvedEffectKind::Damage { category: observe::DamageCategory::Thorns, incoming: 2, .. })).count();
    assert_eq!(burn_ticks, 1, "one held Burn resolves once through Vault: {:?}", vault_result.effects);
    assert!(vault_result.effects.iter().any(|effect| matches!(effect.kind,
        observe::ResolvedEffectKind::Damage { category: observe::DamageCategory::Thorns, incoming: 2, absorbed_block: 2, hp_lost: 0, .. })),
        "Like Water's end-turn Block absorbs the held Burn");
    assert_eq!(c.state.player.hp, hp_before_vault - 1, "Vault's CardAfterUse Beat resolves before the end-turn Block");
    assert_eq!(hook_count(&c, Hook::PlayerEndTurn), player_end_count + 1);
    assert_eq!(piles::owned_full_cards(&c.state).filter(|card| card.id == Insight).count(), 1,
        "Study creates one base Insight during the skipped enemy round");
    assert!(engine::validate_runtime(&c.state).is_ok());

    // BLOCKED on R2: the assertion is retained in an uninvoked closure until the
    // admitted CloakClasp item can enter combat in this tree (C6/a6 request R2).
    let blocked_clasp_case = || {
        let mut clasp = watcher_with(entry(EncounterId::CorruptHeart, 4),
            &[(LikeWater, 0), (Vault, 0), (DefendPurple, 0)], 3, 9, 5,
            vec![RelicId::CloakClasp], vec![None; 3]);
        play_card(&mut clasp, LikeWater, None);
        play_card(&mut clasp, Vault, None);
        play_card(&mut clasp, DefendPurple, None);
        assert_eq!(clasp.state.player.block, 6, "Like Water 5 plus Cloak Clasp 1 for the retained hand");
    };
    let _ = blocked_clasp_case;
}

#[test]
fn m25_t10_blasphemy_at_act4() {
    // Blasphemy installs a positive BUFF and requests 99,999 HP loss at next
    // player start, subject to the real terminal boundary: watcher research
    // :139 and data/cards/implementation-notes.json:23-25.
    let mut ordinary = heart(&[(Blasphemy, 0), (DefendPurple, 0)], 9);
    play_card(&mut ordinary, Blasphemy, None);
    assert_eq!(player_power(&ordinary, "blasphemer"), 1);
    end(&mut ordinary);
    assert_eq!(ordinary.state.outcome, Some(Outcome::Death));

    let mut vault = heart(&[(Blasphemy, 0), (Vault, 0)], 9);
    play_card(&mut vault, Blasphemy, None);
    play_card(&mut vault, Vault, None);
    assert_eq!(vault.state.outcome, Some(Outcome::Death), "Blasphemer fires at the extra turn start after Vault");

    let mut victory = heart(&[(Blasphemy, 0), (StrikePurple, 0)], 9);
    victory.state.enemies[0].hp = 1;
    play_card(&mut victory, Blasphemy, None);
    play_card(&mut victory, StrikePurple, Some(1));
    assert_eq!(victory.state.outcome, Some(Outcome::Victory));
    assert!(victory.state.player.hp > 0, "same-turn Heart Victory cuts off the later Blasphemy death");
}

#[test]
fn m25_t11_lesson_learned_and_wish_settlement() {
    // Lesson Learned's fatal branch journals one permanent upgrade; Wish's
    // FameAndFortune uses shared combat gold. Combat-only ownership is asserted
    // here; Run persistence belongs to Main (watcher research :516-523 and
    // data/cards/implementation-notes.json:67-69).
    let mut victory = heart(&[(Wish, 0), (LessonLearned, 0), (DefendPurple, 0)], 9);
    victory.state.enemies[0].hp = 10;
    play_card(&mut victory, Wish, None);
    victory.step_full(&CombatAction::ChooseCardOffer { index: 1 }).unwrap();
    assert_eq!(victory.current_combat_gold(), Some(25));
    play_card(&mut victory, LessonLearned, Some(1));
    assert_eq!(victory.state.outcome, Some(Outcome::Victory));
    assert_eq!(victory.permanent_card_upgrades().len(), 1);
    assert_eq!(victory.current_combat_gold(), Some(25), "Victory settlement does not duplicate or clear combat gold");

    let mut death = watcher(EncounterId::ShieldAndSpear, 4,
        &[(Wish, 0), (LessonLearned, 0), (DefendPurple, 0)], 9);
    death.set_thief_gold_for_entry(0).unwrap();
    set_pair_intent(&mut death, 0, "shield_fortify");
    set_pair_intent(&mut death, 1, "spear_skewer");
    play_card(&mut death, Wish, None);
    death.step_full(&CombatAction::ChooseCardOffer { index: 1 }).unwrap_or_else(|error|
        panic!("Shield/Spear Wish choice failed: {error:?}; actions={:?}; offer={:?}",
            death.legal_actions_full(), death.state.silent.card_offer));
    death.state.enemies[0].hp = 10;
    play_card(&mut death, LessonLearned, Some(1));
    assert_eq!(death.state.outcome, None, "one partner death leaves Shield/Spear combat open");
    assert_eq!(death.permanent_card_upgrades().len(), 1);
    death.state.player.hp = 1;
    end(&mut death);
    assert_eq!(death.state.outcome, Some(Outcome::Death));
    assert_eq!(death.permanent_card_upgrades().len(), 1);
    assert_eq!(death.current_combat_gold(), Some(25), "Death settlement preserves the same single Wish award");

    let mut nonfatal = heart(&[(LessonLearned, 0)], 9);
    nonfatal.state.enemies[0].hp = 100;
    play_card(&mut nonfatal, LessonLearned, Some(1));
    assert!(nonfatal.permanent_card_upgrades().is_empty(), "a nonfatal Lesson Learned hit adds no journal entry");
}

#[test]
fn m25_t12_generation_and_master_reality_at_act4() {
    // Generator and destination facts: watcher research :92-94,129-132,144,150-171 and
    // data/cards/implementation-notes.json:3-12,84. Master Reality ("Whenever a card is created during
    // combat, Upgrade it"; wiki Evaluate/Pray/Study/Reach Heaven/Conjure Blade/Alpha) upgrades every
    // draw-pile child. Amended after final review v1 I-1 (pre-review: upgrades == 0 in each draw-pile
    // assertion below; original bytes retained in the M25 evidence fault-I1/original-bytes).
    let mut alpha = heart(&[(MasterReality, 0), (Alpha, 0)], 9);
    play_card(&mut alpha, MasterReality, None);
    play_card(&mut alpha, Alpha, None);
    let beta = alpha.state.piles.draw.iter().filter_map(model::OwnedCard::full).find(|card| card.id == Beta).unwrap();
    assert_eq!(beta.upgrades, 1, "Master Reality upgrades Alpha's draw-pile Beta");
    let beta_id = beta.instance.0;
    move_card(&mut alpha, beta_id, SelectionSourceZone::Hand);
    play(&mut alpha, beta_id, None);
    let omega = alpha.state.piles.draw.iter().filter_map(model::OwnedCard::full).find(|card| card.id == Omega).unwrap();
    assert_eq!(omega.upgrades, 1, "Master Reality upgrades Beta's draw-pile Omega");

    let mut generated = heart(&[(MasterReality, 0), (Evaluate, 0), (Pray, 0), (Study, 0),
        (ReachHeaven, 0), (ConjureBlade, 0)], 20);
    play_card(&mut generated, MasterReality, None);
    play_card(&mut generated, Evaluate, None);
    let insight = generated.state.piles.draw.iter().filter_map(model::OwnedCard::full).find(|card| card.id == Insight).unwrap();
    assert_eq!(insight.upgrades, 1, "Master Reality upgrades Evaluate's draw-pile Insight");
    play_card(&mut generated, Pray, None);
    assert!(generated.state.piles.draw.iter().filter_map(model::OwnedCard::full)
        .filter(|card| card.id == Insight).all(|card| card.upgrades == 1), "Master Reality upgrades Pray's Insight too");
    play_card(&mut generated, Study, None);
    play_card(&mut generated, ReachHeaven, Some(1));
    assert!(generated.state.piles.draw.iter().filter_map(model::OwnedCard::full)
        .any(|card| card.id == ThroughViolence && card.upgrades == 1), "Reach Heaven creates Through Violence+");
    play_card(&mut generated, ConjureBlade, None);
    assert!(generated.state.piles.draw.iter().filter_map(model::OwnedCard::full)
        .any(|card| card.id == Expunger && card.misc == 13 && card.upgrades == 1),
        "Conjure Blade captures remaining X and creates Expunger+ in draw");

    // Study's delayed Insight is a draw-pile child; Master Reality upgrades it there too.
    let mut study = heart(&[(MasterReality, 0), (Study, 0)], 9);
    study.state.player_full_powers.push(full_power("no_draw", 1));
    play_card(&mut study, MasterReality, None);
    play_card(&mut study, Study, None);
    end(&mut study);
    assert!(piles::owned_full_cards(&study.state).any(|card| card.id == Insight && card.upgrades == 1));

    // A real Foreign Influence opens its source offer; bind the delivered output to the
    // documented test-selection fixture, then play fatal Hand of Greed through shared gold.
    let mut fi = heart(&[(ForeignInfluence, 0)], 9);
    fi.set_thief_gold_for_entry(0).unwrap();
    play_card(&mut fi, ForeignInfluence, None);
    fi.state.silent.card_offer.as_mut().unwrap().cards = vec![HandOfGreed, SwiftStrike, Anger];
    fi.step_full(&CombatAction::ChooseCardOffer { index: 0 }).unwrap();
    let greed = fi.state.piles.hand.iter().filter_map(model::OwnedCard::full)
        .find(|card| card.id == HandOfGreed).unwrap().instance.0;
    fi.state.enemies[0].hp = 1;
    play(&mut fi, greed, Some(1));
    assert_eq!(fi.state.outcome, Some(Outcome::Victory));
    assert_eq!(fi.current_combat_gold(), Some(20), "fatal Foreign Influence Hand of Greed gains gold through the shared owner");

    // Foreign Influence has no action-level Master Reality check: its hand helper upgrades the
    // inserted copy, while its overflow helper upgrades only a display copy (M21 source rulings;
    // play-origins research Foreign Influence row). A full hand sends the base offer to discard.
    let mut overflow = heart(&[(MasterReality, 0), (ForeignInfluence, 0), (DefendPurple, 0)], 9);
    play_card(&mut overflow, MasterReality, None);
    play_card(&mut overflow, ForeignInfluence, None);
    overflow.state.silent.card_offer.as_mut().unwrap().cards = vec![HandOfGreed, SwiftStrike, Anger];
    for instance in 100..109 {
        let card = m25_watcher_live_card(DefendPurple, CardInstanceId(instance), 0).unwrap();
        overflow.state.card_ids_issued.insert(card_state::handle(card.instance));
        overflow.state.piles.hand.push(model::OwnedCard::Full(card));
    }
    overflow.step_full(&CombatAction::ChooseCardOffer { index: 0 }).unwrap();
    let offered = overflow.state.piles.discard.iter().filter_map(model::OwnedCard::full).find(|card| card.id == HandOfGreed).unwrap();
    assert_eq!(offered.upgrades, 0, "overflow to discard bypasses Master Reality's hand-entry checkpoint");
}

#[test]
fn m25_master_reality_upgrades_creations_in_every_destination() {
    // The shared creation owner (piles::insert_generated) applies Master Reality once for every
    // character and destination; Curses and Statuses are not upgraded (final review v1 I-1).
    let mut c = heart(&[(MasterReality, 0), (Vigilance, 0)], 9);
    play_card(&mut c, MasterReality, None);
    let create = |c: &mut Combat, id: CardId, destination: CardDestination| {
        c.state.pending.push_back(Effect { source: Source::None, target: Some(Actor::Player),
            kind: EffectKind::CreateCard { id, destination, upgrades: 0 } });
        assert_ne!(engine::resolve(&mut c.state).status, TransitionStatus::Fault, "{:?}", c.fault_record());
    };
    // Kinds admitted by the native CreateCard validator; Blood for Blood, Endless Agony and Shiv are
    // non-Watcher kinds created through the same owner.
    for (id, destination) in [(Miracle, CardDestination::Hand), (BloodForBlood, CardDestination::Discard),
        (EndlessAgony, CardDestination::ShuffledDraw), (Wound, CardDestination::Discard), (Burn, CardDestination::Draw)] {
        create(&mut c, id, destination);
    }
    let upgrades = |c: &Combat, id: CardId| piles::owned_full_cards(&c.state).filter(|card| card.id == id)
        .map(|card| card.upgrades).collect::<Vec<_>>();
    assert_eq!(upgrades(&c, Miracle), vec![1], "hand creation");
    assert_eq!(upgrades(&c, BloodForBlood), vec![1], "discard creation");
    assert_eq!(upgrades(&c, EndlessAgony), vec![1], "draw-pile creation");
    assert_eq!(upgrades(&c, Wound), vec![0], "a Status is never upgraded");
    assert_eq!(upgrades(&c, Burn), vec![0], "a Status is never upgraded");

    // Hand overflow: an ordinary creation for a full hand goes to discard, still upgraded
    // (the creation's own check precedes the overflow helper).
    while c.state.piles.hand.len() < 10 { create(&mut c, Miracle, CardDestination::Hand); }
    create(&mut c, Shiv, CardDestination::Hand);
    assert!(c.state.piles.discard.iter().filter_map(model::OwnedCard::full).any(|card| card.id == Shiv && card.upgrades == 1),
        "overflow creation into discard is upgraded");
    assert!(engine::validate_runtime(&c.state).is_ok());
}

#[test]
fn m25_collect_counts_chemical_x() {
    // Collect is an X card: Chemical X adds 2, as for Conjure Blade (final review v1 m-5). Chemical X is
    // outside the M25 relic union; this pins the shared X law for later profiles.
    for (upgrades, expected) in [(0, 4), (1, 5)] {
        let mut c = watcher_with(entry(EncounterId::CorruptHeart, 4), &[(Collect, upgrades)], 1, 2, 5,
            vec![RelicId::PureWater], vec![None; 3]);
        // Chemical X is outside the M25 startup union, so it is installed after entry.
        c.state.item_context = Some(CombatItemContext::new(vec![RelicId::PureWater, RelicId::ChemicalX], vec![None; 3]).with_m25_profile());
        play_card(&mut c, Collect, None);
        let collect = c.state.player_full_powers.iter().find(|power| power.id.0 == "collect").map(|power| power.state.amount);
        assert_eq!(collect, Some(expected), "Collect{upgrades}: energy 2 + Chemical X 2 + upgrade");
    }
}
