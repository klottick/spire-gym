use super::super::*;
use crate::content::{CardId, Catalog, Character};
use super::full_runtime::entered_combat;
use model::OwnedCard;

#[derive(Clone, Copy)]
enum Zone { Hand, Draw, Discard, Exhaust }

fn id(token: &str) -> CardId {
    Catalog::standard().cards().iter().find(|row| row.id.as_str() == token)
        .unwrap_or_else(|| panic!("missing executable green card {token}")).id
}

fn fight(token: &str, upgrades: u32) -> Combat {
    let mut combat = entered_combat(12, 12, |card| {
        *card = permanent_live_card(id(token), card.instance, upgrades);
    });
    combat.state.character = characters::CharacterState::for_character(Character::Silent);
    combat.state.enemies[0].hp = 500;
    combat.state.enemies[0].max_hp = 500;
    combat
}

fn add(combat: &mut Combat, token: &str, instance: u64, zone: Zone) {
    let card = permanent_live_card(id(token), CardInstanceId(instance), 0);
    assert!(combat.state.card_ids_issued.insert(card_state::handle(card.instance)));
    let owned = OwnedCard::Full(card);
    match zone {
        Zone::Hand => combat.state.piles.hand.push(owned),
        Zone::Draw => combat.state.piles.draw.push(owned),
        Zone::Discard => combat.state.piles.discard.push(owned),
        Zone::Exhaust => combat.state.piles.exhaust.push(owned),
    }
}

fn live(combat: &Combat, instance: u64) -> &LiveCard {
    let handle = CardHandle::parse(&instance.to_string()).unwrap();
    combat.state.piles.all(combat.state.in_play.as_ref())
        .chain(combat.state.inactive_cards.iter())
        .chain(combat.state.queued_autoplay.values())
        .find(|card| card.handle() == handle).and_then(OwnedCard::full)
        .unwrap_or_else(|| panic!("missing card instance {instance}"))
}

fn play(combat: &mut Combat, instance: u64, target: bool) {
    combat.step_full(&CombatAction::PlayCard {
        instance, target: target.then_some(TargetId(1)),
    }).unwrap();
}

fn move_to_hand(combat: &mut Combat, instance: u64) {
    let handle=CardHandle::parse(&instance.to_string()).unwrap();
    if combat.state.piles.hand.iter().any(|card|card.handle()==handle){return}
    if let Some(index)=combat.state.piles.draw.iter().position(|card|card.handle()==handle){
        let card=combat.state.piles.draw.remove(index);combat.state.piles.hand.push(card);return
    }
    if let Some(index)=combat.state.piles.discard.iter().position(|card|card.handle()==handle){
        let card=combat.state.piles.discard.remove(index);combat.state.piles.hand.push(card);return
    }
    if let Some(index)=combat.state.piles.exhaust.iter().position(|card|card.handle()==handle){
        let card=combat.state.piles.exhaust.remove(index);combat.state.piles.hand.push(card);return
    }
    panic!("missing card instance {instance}");
}

fn hp_loss(amount: i32) -> Effect {
    Effect { source: Source::Actor(Actor::Player), target: Some(Actor::Player),
        kind: EffectKind::Damage(DamagePayload::HpLoss(amount)) }
}

#[test]
fn m15_silent_cost_copy_keeps_instance_location_and_writer_order() {
    let mut masterful = fight("masterful_stab", 0);
    let view = masterful.public_full().unwrap();
    let played = view.hand.iter().find(|card| card.instance.0 == 3).unwrap();
    assert_eq!(played.effective_cost, Some(0));
    assert_eq!(played.values.as_ref().unwrap().base_damage, Some(12));
    add(&mut masterful, "masterful_stab", 100, Zone::Hand);
    add(&mut masterful, "masterful_stab", 101, Zone::Draw);
    add(&mut masterful, "masterful_stab", 102, Zone::Discard);
    add(&mut masterful, "masterful_stab", 103, Zone::Exhaust);
    masterful.state.enemies[0].enemy_id = model::ActorEnemyId::Full(crate::content::EnemyId::Spiker);
    masterful.state.enemies[0].full_powers.push(FullPowerEntry {
        id: FullPowerId("thorns"), state: PowerState { amount: 3, skip_first: false, monster_source_grace: false },
    });
    play(&mut masterful, 3, true);
    assert_eq!(live(&masterful, 3).local.mutable_combat_cost, Some(0), "the playing object is outside the callback locations");
    for instance in [100, 101, 102] {
        assert_eq!(live(&masterful, instance).local.mutable_combat_cost, Some(1), "positive HP-loss event visits hand, draw and discard once");
    }
    assert_eq!(live(&masterful, 103).local.mutable_combat_cost, Some(0), "exhaust is not visited");

    masterful.state.player.block = 2;
    masterful.enqueue_native_effects(vec![Effect { source: Source::None, target: Some(Actor::Player),
        kind: EffectKind::Damage(DamagePayload::Fixed(2)) }], None).unwrap();
    assert_eq!(live(&masterful, 100).local.mutable_combat_cost, Some(1), "fully prevented loss does not notify");
    masterful.enqueue_native_effects(vec![hp_loss(5)], None).unwrap();
    for instance in [100, 101, 102] {
        assert_eq!(live(&masterful, instance).local.mutable_combat_cost, Some(2), "five lost HP is one notification");
    }
    assert_eq!(live(&masterful, 3).local.mutable_combat_cost, Some(1), "the played card joins the discard callback locations only after its use");
    assert_eq!(live(&masterful, 103).local.mutable_combat_cost, Some(0));
    let index=masterful.state.piles.exhaust.iter().position(|card|card.handle().as_str()=="103").unwrap();
    let exhausted=masterful.state.piles.exhaust.remove(index);
    masterful.state.piles.hand.push(exhausted);
    assert_eq!(live(&masterful,103).local.mutable_combat_cost,Some(0),"returning from exhaust does not catch up missed damage events");

    let mut eviscerate = fight("eviscerate", 0);
    let view = eviscerate.public_full().unwrap();
    let played = view.hand.iter().find(|card| card.instance.0 == 3).unwrap();
    assert_eq!(played.effective_cost, Some(3));
    assert_eq!(played.values.as_ref().unwrap().base_damage, Some(7));
    add(&mut eviscerate, "eviscerate", 200, Zone::Hand);
    add(&mut eviscerate, "eviscerate", 201, Zone::Draw);
    add(&mut eviscerate, "eviscerate", 202, Zone::Discard);
    for victim in [210, 211] {
        add(&mut eviscerate, "wound", victim, Zone::Hand);
        let handle = CardHandle::parse(&victim.to_string()).unwrap();
        let index = eviscerate.state.piles.hand.iter().position(|card| card.handle() == handle).unwrap();
        let card = eviscerate.state.piles.hand.remove(index);
        piles::discard_selected_card(&mut eviscerate.state, card, true).unwrap();
    }
    assert_eq!(eviscerate.state.silent.discarded_this_turn, 2);
    for instance in [3, 200, 201, 202] {
        assert_eq!(live(&eviscerate, instance).local.turn_cost, Some(1), "each manual discard notifies each extant pile instance");
    }
    let draw_index = eviscerate.state.piles.draw.iter().position(|card| card.handle().as_str() == "201").unwrap();
    eviscerate.state.piles.draw[draw_index].full_mut().unwrap().local.turn_cost = Some(3);
    eviscerate.enqueue_native_effects(vec![Effect { source: Source::None, target: Some(Actor::Player),
        kind: EffectKind::Draw(1) }], None).unwrap();
    assert_eq!(live(&eviscerate, 201).local.turn_cost, Some(1), "actual draw rebases from combat cost minus the current turn discard count");
}

#[test]
fn m15_silent_setup_and_bullet_time_have_distinct_cost_lifetimes() {
    let mut automatic=fight("setup",0);
    let first=automatic.state.piles.hand.iter().position(|card|card.handle().as_str()=="1").unwrap();
    let removed=automatic.state.piles.hand.remove(first);
    automatic.state.inactive_cards.push(removed);
    play(&mut automatic,3,false);
    assert!(automatic.state.pending_selection.is_none(),"one remaining hand card is moved automatically");
    assert_eq!(automatic.state.piles.draw.last().unwrap().handle().as_str(),"2");
    assert!(automatic.state.piles.draw.last().unwrap().full().unwrap().local.free_to_play_once);

    let mut empty=fight("setup",0);
    for card in std::mem::take(&mut empty.state.piles.hand){
        if card.handle().as_str()=="3"{empty.state.piles.hand.push(card)}else{empty.state.inactive_cards.push(card)}
    }
    play(&mut empty,3,false);
    assert!(empty.state.pending_selection.is_none(),"empty hand makes Setup a no-op");
    assert_eq!(empty.state.piles.discard.iter().filter(|card|card.handle().as_str()=="3").count(),1);

    let mut setup = fight("setup", 0);
    add(&mut setup, "eviscerate", 120, Zone::Hand);
    {
        let card = setup.state.piles.hand.iter_mut().find(|card| card.handle().as_str() == "120").unwrap().full_mut().unwrap();
        card.local.turn_cost = Some(1);
        card.local.mutable_combat_cost = Some(3);
    }
    play(&mut setup, 3, false);
    assert_eq!(setup.public_full().unwrap().selection.as_ref().unwrap().minimum, 1);
    setup.step_full(&CombatAction::SelectCard { instance: 120 }).unwrap();
    let moved = setup.state.piles.draw.last().unwrap();
    assert_eq!(moved.handle().as_str(), "120", "Setup topdecks the same owned instance");
    let selected = moved.full().unwrap();
    assert!(selected.local.free_to_play_once);
    assert!(!selected.local.free_play,"Setup state does not reuse the autoplay free_play marker");
    assert!(setup.public_full().unwrap().known_draw.iter().find(|known|known.card.instance.0==120)
        .unwrap().card.values.as_ref().unwrap().free_play,"the public value reports Setup's active one-use waiver");
    assert_eq!(selected.local.mutable_combat_cost, Some(3));
    assert_eq!(selected.local.turn_cost, Some(1), "the free flag does not rewrite either numeric cost");
    assert_eq!(setup.state.piles.discard.iter().filter(|card| card.handle().as_str() == "3").count(), 1);
    assert!(setup.state.play_frames.is_empty(), "the selection resumes its parent once");
    setup.step_full(&CombatAction::EndTurn).unwrap();
    assert_eq!(live(&setup,120).local.turn_cost,Some(3),"turn cost resets to combat cost at the player-turn boundary");
    assert!(live(&setup,120).local.free_to_play_once,"Setup's one-use flag survives the turn reset");
    move_to_hand(&mut setup,120);
    setup.state.player.energy=0;
    play(&mut setup,120,true);
    assert_eq!(setup.state.player.energy,0,"Setup waives payment after the ordinary turn-cost reset");
    assert!(!live(&setup,120).local.free_to_play_once,"the Setup waiver clears after its card-use callbacks");

    let mut zero_cost = fight("setup", 0);
    add(&mut zero_cost, "masterful_stab", 121, Zone::Hand);
    {
        let card = zero_cost.state.piles.hand.iter_mut().find(|card| card.handle().as_str() == "121").unwrap().full_mut().unwrap();
        card.local.mutable_combat_cost = Some(0);
        card.local.turn_cost = Some(2);
    }
    play(&mut zero_cost, 3, false);
    zero_cost.step_full(&CombatAction::SelectCard { instance: 121 }).unwrap();
    let selected = live(&zero_cost, 121);
    assert!(!selected.local.free_to_play_once, "zero combat cost with a positive turn price does not gain a Setup flag");
    assert_eq!(selected.local.mutable_combat_cost, Some(0));
    assert_eq!(selected.local.turn_cost, Some(2));

    let mut existing_flag = fight("setup", 0);
    add(&mut existing_flag,"masterful_stab",124,Zone::Hand);
    {
        let card=existing_flag.state.piles.hand.iter_mut().find(|card|card.handle().as_str()=="124").unwrap().full_mut().unwrap();
        card.local.mutable_combat_cost=Some(0);card.local.turn_cost=Some(2);card.local.free_to_play_once=true;
    }
    play(&mut existing_flag,3,false);
    existing_flag.step_full(&CombatAction::SelectCard{instance:124}).unwrap();
    assert!(live(&existing_flag,124).local.free_to_play_once,"a failed guard preserves an existing Setup flag");
    assert_eq!(live(&existing_flag,124).local.turn_cost,Some(2));

    let mut sentinel = fight("setup", 0);
    add(&mut sentinel, "whirlwind", 122, Zone::Hand);
    {
        let card = sentinel.state.piles.hand.iter_mut().find(|card| card.handle().as_str() == "122").unwrap().full_mut().unwrap();
        card.local.mutable_combat_cost = Some(-1);
        card.local.turn_cost = Some(-1);
        card.local.x_cost = true;
    }
    play(&mut sentinel, 3, false);
    sentinel.step_full(&CombatAction::SelectCard { instance: 122 }).unwrap();
    assert!(!live(&sentinel, 122).local.free_to_play_once, "X remains ineligible for Setup's one-play flag");

    let mut unplayable=fight("setup",0);
    add(&mut unplayable,"reflex",123,Zone::Hand);
    {
        let card=unplayable.state.piles.hand.iter_mut().find(|card|card.handle().as_str()=="123").unwrap().full_mut().unwrap();
        card.local.mutable_combat_cost=Some(-2);card.local.turn_cost=Some(1);
    }
    play(&mut unplayable,3,false);
    unplayable.step_full(&CombatAction::SelectCard{instance:123}).unwrap();
    assert!(!live(&unplayable,123).local.free_to_play_once,"unplayable sentinel with positive turn price is still ineligible");

    let mut bullet = fight("bullet_time", 0);
    add(&mut bullet, "eviscerate", 130, Zone::Hand);
    add(&mut bullet, "masterful_stab", 131, Zone::Hand);
    add(&mut bullet, "whirlwind", 132, Zone::Hand);
    add(&mut bullet, "reflex", 133, Zone::Hand);
    for (instance, combat_cost, turn_cost, x_cost) in [
        (130, 3, 2, false), (131, 0, 2, false), (132, -1, -1, true), (133, -2, -2, false),
    ] {
        let card = bullet.state.piles.hand.iter_mut().find(|card| card.handle().as_str() == instance.to_string()).unwrap().full_mut().unwrap();
        card.local.mutable_combat_cost = Some(combat_cost);
        card.local.turn_cost = Some(turn_cost);
        card.local.x_cost = x_cost;
    }
    play(&mut bullet, 3, false);
    for (instance, expected_cost) in [(130, 3), (131, 0), (132, -1), (133, -2)] {
        let card = live(&bullet, instance);
        assert_eq!(card.local.mutable_combat_cost, Some(expected_cost));
        assert_eq!(card.local.turn_cost, Some(if instance < 132 { 0 } else { expected_cost }));
        assert!(!card.local.free_to_play_once, "Bullet Time does not grant Setup's one-play flag");
    }
    assert!(bullet.state.player_full_powers.iter().any(|power| power.id.0 == "no_draw" && power.state.amount > 0));
    let hand_before = bullet.state.piles.hand.len();
    bullet.enqueue_native_effects(vec![Effect { source: Source::None, target: Some(Actor::Player),
        kind: EffectKind::Draw(1) }], None).unwrap();
    assert_eq!(bullet.state.piles.hand.len(), hand_before, "No Draw blocks ordinary draw requests");
    bullet.enqueue_native_effects(vec![Effect { source: Source::None, target: Some(Actor::Player),
        kind: EffectKind::CreateCard { id: id("shiv"), destination: CardDestination::Hand, upgrades: 0 } }], None).unwrap();
    assert_eq!(bullet.state.piles.hand.iter().filter(|card| card.full().is_some_and(|live| live.id == id("shiv"))).count(), 1,
        "No Draw does not prevent non-draw card creation");
    assert_eq!(bullet.state.piles.discard.iter().filter(|card| card.handle().as_str() == "3").count(), 1);
    bullet.step_full(&CombatAction::EndTurn).unwrap();
    for (instance,combat_cost) in [(130,3),(131,0)]{
        assert_eq!(live(&bullet,instance).local.turn_cost,Some(combat_cost),"Bullet Time turn price expires without changing combat cost");
    }
    assert!(!bullet.state.player_full_powers.iter().any(|power|power.id.0=="no_draw"&&power.state.amount>0));
}

#[test]
fn m15_silent_cost_copy_preserves_existing_combat_card_debug_shape() {
    assert_eq!(format!("{:?}",card_state::CombatCardData::default()),
        "CombatCardData { base_damage: None, temporary_upgrades: 0, randomized_cost: None, combat_cost_reduction: 0, temporary_cost: 0, infernal_blade_cost_this_turn: false, free_play: false, x_cost: false, exhaust_on_finish: false, ethereal: false, innate: false, purge: false }");
}

#[test]
fn m15_silent_copy_generation_is_not_draw_or_alias() {
    let mut created = fight("setup", 0);
    created.enqueue_native_effects(vec![Effect { source: Source::None, target: Some(Actor::Player),
        kind: EffectKind::CreateCard { id: id("endless_agony"), destination: CardDestination::Hand, upgrades: 1 } }], None).unwrap();
    assert_eq!(created.state.piles.hand.iter().filter(|card| card.full().is_some_and(|live| live.id == id("endless_agony"))).count(), 1,
        "creating Endless Agony does not invoke its actual-draw copy hook");

    let mut drawn = fight("setup", 0);
    drawn.state.silent.history_enabled = true;
    drawn.state.silent.drawn_cards.clear();
    add(&mut drawn, "endless_agony", 140, Zone::Draw);
    let source = drawn.state.piles.draw.iter_mut().find(|card| card.handle().as_str() == "140").unwrap().full_mut().unwrap();
    source.upgrades = 1;
    source.local.base_damage = Some(6);
    source.local.exhaust_on_finish = true;
    drawn.enqueue_native_effects(vec![Effect { source: Source::None, target: Some(Actor::Player),
        kind: EffectKind::Draw(1) }], None).unwrap();
    assert_eq!(drawn.state.piles.hand.iter().filter(|card| card.full().is_some_and(|live| live.id == id("endless_agony"))).count(), 2);
    assert_eq!(drawn.state.silent.drawn_cards, vec![id("endless_agony")], "the generated copy is not a second actual draw");
    let copy = drawn.state.piles.hand.iter().filter_map(OwnedCard::full).filter(|live| live.id == id("endless_agony") && live.instance.0 != 140).next().unwrap();
    assert_eq!(copy.upgrades, 1);
    assert_eq!(copy.local.base_damage, Some(6));
    assert_ne!(copy.copy_family, live(&drawn, 140).copy_family, "stat-equivalent temporary copies own a fresh family");

    let mut overflow = fight("setup", 0);
    overflow.state.silent.history_enabled = true;
    overflow.state.silent.drawn_cards.clear();
    overflow.state.inactive_cards.extend(std::mem::take(&mut overflow.state.piles.hand));
    for instance in 150..159 { add(&mut overflow, "defend_g", instance, Zone::Hand); }
    add(&mut overflow, "endless_agony", 160, Zone::Draw);
    overflow.enqueue_native_effects(vec![Effect { source: Source::None, target: Some(Actor::Player),
        kind: EffectKind::Draw(1) }], None).unwrap();
    assert_eq!(overflow.state.piles.hand.len(), 10);
    assert!(overflow.state.piles.discard.iter().any(|card| card.full().is_some_and(|live| live.id == id("endless_agony") && live.instance.0 != 160)),
        "the generated copy is retained through the ordinary hand-overflow discard path");
    assert_eq!(overflow.state.silent.drawn_cards, vec![id("endless_agony")]);

    let mut glass = fight("glass_knife", 0);
    let opening = glass.public_full().unwrap().hand.into_iter().find(|card| card.instance.0 == 3).unwrap();
    assert_eq!(opening.effective_cost, Some(1));
    assert_eq!(opening.values.unwrap().base_damage, Some(8));
    let source = live(&glass, 3).clone();
    let family_copy = source.same_instance_family_copy(CardInstanceId(170));
    assert!(glass.state.card_ids_issued.insert(card_state::handle(family_copy.instance)));
    glass.state.piles.hand.push(OwnedCard::Full(family_copy));
    let fresh_copy = source.stat_equivalent_copy(CardInstanceId(171));
    assert!(glass.state.card_ids_issued.insert(card_state::handle(fresh_copy.instance)));
    glass.state.piles.hand.push(OwnedCard::Full(fresh_copy));
    glass.state.player.energy = 20;
    for (round, damage) in [8, 6, 4, 2, 0].into_iter().enumerate() {
        let before = glass.state.enemies[0].hp;
        play(&mut glass, 3, true);
        assert_eq!(glass.state.enemies[0].hp, before - damage * 2, "both payload hits use pre-mutation damage for family step {round}");
        assert_eq!(live(&glass, 3).local.base_damage, Some((8 - 2 * (round as i32 + 1)).max(0)));
        assert_eq!(live(&glass, 170).local.base_damage, Some((8 - 2 * (round as i32 + 1)).max(0)));
        assert_eq!(live(&glass, 171).local.base_damage, Some(8), "fresh-family copy is independent");
        if round < 4 {
            let index = glass.state.piles.discard.iter().position(|card| card.handle().as_str() == "3").unwrap();
            let card = glass.state.piles.discard.remove(index);
            glass.state.piles.hand.push(card);
            glass.state.player.energy = 20;
        }
    }

    let mut queued=fight("glass_knife",0);
    queued.state.player_full_powers.push(FullPowerEntry{id:FullPowerId("double_tap"),
        state:PowerState{amount:1,skip_first:false,monster_source_grace:false}});
    play(&mut queued,3,true);
    assert_eq!(queued.state.enemies[0].hp,472,"the queued same-family repeat sees Glass Knife's prior -2");
    assert_eq!(live(&queued,3).local.base_damage,Some(4),"the suspended parent is updated during the child's later family mutation");
    let repeated=queued.state.inactive_cards.iter().filter_map(OwnedCard::full).find(|card|card.id==id("glass_knife")).unwrap();
    assert_eq!(repeated.local.base_damage,Some(4),"the queued purge child remains a family owner during mutation");
    assert_eq!(repeated.copy_family,live(&queued,3).copy_family);
}

#[test]
fn m15_silent_eviscerate_make_copy_mutates_the_source_before_capture() {
    let mut nightmare = fight("nightmare", 0);
    add(&mut nightmare, "eviscerate", 180, Zone::Hand);
    {
        let source = nightmare.state.piles.hand.iter_mut().find(|card| card.handle().as_str() == "180").unwrap().full_mut().unwrap();
        source.local.mutable_combat_cost = Some(3);
        source.local.turn_cost = Some(0);
    }
    nightmare.state.silent.discarded_this_turn = 1;
    play(&mut nightmare, 3, false);
    nightmare.step_full(&CombatAction::SelectCard { instance: 180 }).unwrap();
    assert_eq!(live(&nightmare, 180).local.turn_cost, Some(2), "makeCopy applies the discard-history setter to its actual source");
    assert_eq!(nightmare.state.nightmare_templates[0].template.local.turn_cost, Some(3), "Nightmare then resets the captured template to combat cost");

    let source = live(&nightmare, 180);
    let stat_equivalent = source.stat_equivalent_copy(CardInstanceId(181));
    let fresh = card_state::generated(id("eviscerate"), CardInstanceId(182), 0).unwrap();
    assert_eq!(stat_equivalent.local.turn_cost, Some(2));
    assert_eq!(fresh.local.turn_cost, Some(3));
    assert_eq!(live(&nightmare, 180).local.turn_cost, Some(2));

    let mut repeat=fight("eviscerate",0);
    repeat.state.silent.discarded_this_turn=1;
    repeat.state.player_full_powers.push(FullPowerEntry{id:FullPowerId("double_tap"),
        state:PowerState{amount:1,skip_first:false,monster_source_grace:false}});
    play(&mut repeat,3,true);
    assert_eq!(live(&repeat,3).local.turn_cost,Some(2),"Double Tap's actual same-family copy invokes Eviscerate's source hook");
    let repeated=repeat.state.inactive_cards.iter().filter_map(OwnedCard::full).find(|card|card.id==id("eviscerate")).unwrap();
    assert_eq!(repeated.local.turn_cost,Some(2));
    assert_eq!(repeated.copy_family,live(&repeat,3).copy_family);
}

#[test]
fn m15_silent_cost_overflow_faults_instead_of_saturating() {
    let mut combat_overflow=fight("masterful_stab",0);
    add(&mut combat_overflow,"masterful_stab",190,Zone::Hand);
    {
        let card=combat_overflow.state.piles.hand.iter_mut().find(|owned|owned.handle().as_str()=="190").unwrap().full_mut().unwrap();
        card.local.mutable_combat_cost=Some(i32::MAX);
        card.local.turn_cost=Some(1);
    }
    let combat_result=combat_overflow.enqueue_native_effects(vec![hp_loss(1)],None);

    let mut turn_overflow=fight("masterful_stab",0);
    add(&mut turn_overflow,"masterful_stab",191,Zone::Hand);
    {
        let card=turn_overflow.state.piles.hand.iter_mut().find(|owned|owned.handle().as_str()=="191").unwrap().full_mut().unwrap();
        card.local.mutable_combat_cost=Some(i32::MAX-1);
        card.local.turn_cost=Some(i32::MAX);
    }
    let turn_result=turn_overflow.enqueue_native_effects(vec![hp_loss(1)],None);

    assert!(matches!(combat_result,Err(crate::run::RunError::Faulted(_))),"Masterful Stab combat-cost overflow is an explicit native fault");
    assert!(matches!(turn_result,Err(crate::run::RunError::Faulted(_))),"Masterful Stab turn-cost overflow is an explicit native fault");
    assert_eq!(live(&combat_overflow,190).local.mutable_combat_cost,Some(i32::MAX));
    assert_eq!(live(&turn_overflow,191).local.mutable_combat_cost,Some(i32::MAX-1),"failed arithmetic does not commit the cost writer");
    assert_eq!(live(&turn_overflow,191).local.turn_cost,Some(i32::MAX));
}

#[test]
fn m15_bullet_time_turn_price_overflow_faults_instead_of_saturating() {
    let mut combat=fight("bullet_time",0);
    for card in std::mem::take(&mut combat.state.piles.hand) {
        if card.handle().as_str()=="3" { combat.state.piles.hand.push(card); }
        else { combat.state.inactive_cards.push(card); }
    }
    add(&mut combat,"eviscerate",200,Zone::Hand);
    {
        let card=combat.state.piles.hand.iter_mut().find(|owned|owned.handle().as_str()=="200").unwrap().full_mut().unwrap();
        card.local.mutable_combat_cost=Some(3);
        card.local.turn_cost=None;
        card.local.temporary_cost=i32::MAX;
    }
    let result=combat.step_full(&CombatAction::PlayCard{instance:3,target:None});
    assert!(matches!(result,Err(crate::run::RunError::Faulted(_))),"Bullet Time must propagate checked shared price arithmetic as a native fault");
    assert_eq!(live(&combat,200).local.turn_cost,None,"overflow must not rewrite the affected price");
}

#[test]
fn m15_eviscerate_manual_discard_preserves_negative_turn_sentinel() {
    let mut combat=fight("eviscerate",0);
    add(&mut combat,"eviscerate",192,Zone::Hand);
    add(&mut combat,"wound",193,Zone::Hand);
    {
        let source=combat.state.piles.hand.iter_mut().find(|owned|owned.handle().as_str()=="192").unwrap().full_mut().unwrap();
        source.local.mutable_combat_cost=Some(3);
        source.local.turn_cost=Some(-1);
    }
    let index=combat.state.piles.hand.iter().position(|owned|owned.handle().as_str()=="193").unwrap();
    let wound=combat.state.piles.hand.remove(index);
    piles::discard_selected_card(&mut combat.state,wound,true).unwrap();
    let eviscerate=live(&combat,192);
    assert_eq!(eviscerate.local.turn_cost,Some(-1),"manual discard's guarded setter leaves negative sentinels intact");
    assert!(card_state::effective_cost(eviscerate,3).is_err(),"the sentinel does not become ordinarily playable");
}

#[test]
fn m15_eviscerate_actual_draw_preserves_negative_turn_sentinel() {
    let mut combat=fight("eviscerate",0);
    add(&mut combat,"eviscerate",194,Zone::Draw);
    {
        let source=combat.state.piles.draw.last_mut().unwrap().full_mut().unwrap();
        source.local.mutable_combat_cost=Some(3);
        source.local.turn_cost=Some(-2);
    }
    combat.enqueue_native_effects(vec![Effect{source:Source::None,target:Some(Actor::Player),kind:EffectKind::Draw(1)}],None).unwrap();
    let eviscerate=live(&combat,194);
    assert_eq!(eviscerate.local.turn_cost,Some(-2),"actual draw's guarded setter leaves negative sentinels intact");
    assert!(card_state::effective_cost(eviscerate,3).is_err(),"the sentinel does not become ordinarily playable");
}

#[test]
fn m15_eviscerate_source_copy_preserves_negative_turn_sentinel() {
    let mut combat=fight("eviscerate",0);
    add(&mut combat,"eviscerate",195,Zone::Hand);
    {
        let source=combat.state.piles.hand.last_mut().unwrap().full_mut().unwrap();
        source.local.mutable_combat_cost=Some(3);
        source.local.turn_cost=Some(-1);
    }
    combat.state.silent.discarded_this_turn=1;
    let source=CardHandle::parse("195").unwrap();
    let copy=piles::make_stat_equivalent_copy_from_owner(&mut combat.state,&source,CardInstanceId(196)).unwrap();
    assert_eq!(live(&combat,195).local.turn_cost,Some(-1),"makeCopy's guarded setter leaves the source sentinel intact");
    assert_eq!(copy.local.turn_cost,Some(-1),"stat copy captures the still-unplayable sentinel");
    assert!(card_state::effective_cost(&copy,3).is_err(),"the copied sentinel does not become ordinarily playable");
}

#[test]
fn m15_eviscerate_actual_draw_rejects_counter_outside_signed_native_domain() {
    let mut combat=fight("eviscerate",0);
    add(&mut combat,"eviscerate",197,Zone::Draw);
    combat.state.silent.discarded_this_turn=i32::MAX as u32+1;
    let result=combat.enqueue_native_effects(vec![Effect{source:Source::None,target:Some(Actor::Player),kind:EffectKind::Draw(1)}],None);
    assert!(result.is_err(),"actual-draw cost reconciliation rejects a forged unsigned count outside the source's signed domain");
    assert_eq!(live(&combat,197).local.turn_cost,Some(3),"rejected reconciliation does not clamp the count and rewrite the price");
}

#[test]
fn m15_eviscerate_source_copy_rejects_counter_outside_signed_native_domain() {
    let mut combat=fight("eviscerate",0);
    add(&mut combat,"eviscerate",198,Zone::Hand);
    combat.state.silent.discarded_this_turn=i32::MAX as u32+1;
    let source=CardHandle::parse("198").unwrap();
    let result=piles::make_stat_equivalent_copy_from_owner(&mut combat.state,&source,CardInstanceId(199));
    assert!(result.is_err(),"the source makeCopy hook rejects a forged unsigned count outside the source's signed domain");
    assert_eq!(live(&combat,198).local.turn_cost,Some(3),"rejected reconciliation leaves the live source unchanged");
}

#[test]
fn m15_silent_debug_emits_nondefault_cost_fields() {
    let mut data=card_state::CombatCardData::default();
    data.mutable_combat_cost=Some(3);
    data.turn_cost=Some(-1);
    data.free_to_play_once=true;
    let debug=format!("{data:?}");
    assert!(debug.contains("mutable_combat_cost: Some(3)"),"nondefault combat cost is visible in Debug");
    assert!(debug.contains("turn_cost: Some(-1)"),"nondefault turn sentinel is visible in Debug");
    assert!(debug.contains("free_to_play_once: true"),"nondefault Setup state is visible in Debug");
}

#[test]
fn m15_silent_cost_copy_live_upgrade_preserves_cost_writers_and_degraded_damage() {
    for (token,before,after) in [("masterful_stab",12,16),("eviscerate",7,9),("endless_agony",4,6),("glass_knife",2,6)] {
        let mut c=fight(token,0);
        let card=c.state.piles.hand.iter_mut().find(|owned|owned.handle().as_str()=="3").unwrap().full_mut().unwrap();
        card.local.base_damage=Some(before);card.local.mutable_combat_cost=Some(2);card.local.turn_cost=Some(1);
        cards::ironclad::rules::upgrade(card).unwrap();
        assert_eq!(card.local.base_damage,Some(after),"{token} upgrades its actual mutable base damage");
        assert_eq!(card.local.mutable_combat_cost,Some(2));assert_eq!(card.local.turn_cost,Some(1));
        assert_eq!(card.local.temporary_upgrades,1);
    }
    for (token,requested) in [("setup",0),("bullet_time",2)] {
        for turn in [-1,0,2] {
            let mut c=fight(token,0);
            let card=c.state.piles.hand.iter_mut().find(|owned|owned.handle().as_str()=="3").unwrap().full_mut().unwrap();
            card.local.mutable_combat_cost=Some(1);card.local.turn_cost=Some(turn);
            card.local.free_to_play_once=true;
            card.local.temporary_cost=5; // An explicit source turn price owns this override.
            cards::ironclad::rules::upgrade(card).unwrap();
            assert_eq!(card.local.mutable_combat_cost,Some(requested));
            assert_eq!(card.local.turn_cost,Some(if turn>0 { (requested+turn-1).max(0) } else { 0 }));
            assert!(card.local.free_to_play_once,"upgradeBaseCost does not erase Setup's independent flag");
            piles::expire_turn_scoped_costs(&mut c.state);
            assert_eq!(live(&c,3).local.turn_cost,Some(requested),"new turn restores the actual upgraded combat cost");
        }
    }
}

#[test]
fn independent_review_endless_agony_capture() {
    let mut c = fight("setup",0);
    c.state.silent.history_enabled=true;
    c.state.player_full_powers.push(FullPowerEntry{id:FullPowerId("confused"),state:PowerState{amount:1,skip_first:false,monster_source_grace:false}});
    add(&mut c,"endless_agony",140,Zone::Draw);
    let source=c.state.piles.draw.last_mut().unwrap().full_mut().unwrap();
    source.local.turn_cost=Some(2);
    c.enqueue_native_effects(vec![Effect{source:Source::None,target:Some(Actor::Player),kind:EffectKind::Draw(1)}],None).unwrap();
    let agonies=c.state.piles.hand.iter().filter_map(OwnedCard::full).filter(|card|card.id==CardId::EndlessAgony).collect::<Vec<_>>();
    let original=agonies.iter().find(|card|card.instance.0==140).unwrap();
    let copy=agonies.iter().find(|card|card.instance.0!=140).unwrap();
    println!("drawn original cost={:?}, copy cost={:?}, copy randomized={:?}",original.local.turn_cost,copy.local.turn_cost,copy.local.randomized_cost);
    assert_eq!(copy.local.turn_cost,Some(2),"draw callback must capture the copy before Confused rewrites original cost");
    assert_eq!(copy.local.randomized_cost,None,"captured copy must retain pre-Confused randomized-cost state");
}

#[test]
fn independent_review_endless_agony_capture_visible_price() {
    let mut c = fight("setup",0);
    c.state.silent.history_enabled=true;
    c.state.player_full_powers.push(FullPowerEntry{id:FullPowerId("confused"),state:PowerState{amount:1,skip_first:false,monster_source_grace:false}});
    add(&mut c,"endless_agony",140,Zone::Draw);
    let source=c.state.piles.draw.last_mut().unwrap().full_mut().unwrap();
    source.local.turn_cost=Some(3);
    c.enqueue_native_effects(vec![Effect{source:Source::None,target:Some(Actor::Player),kind:EffectKind::Draw(1)}],None).unwrap();
    let agonies=c.state.piles.hand.iter().filter_map(OwnedCard::full).filter(|card|card.id==CardId::EndlessAgony).collect::<Vec<_>>();
    let original=agonies.iter().find(|card|card.instance.0==140).unwrap();
    let copy=agonies.iter().find(|card|card.instance.0!=140).unwrap();
    println!("drawn original cost={:?}, copy cost={:?}, copy randomized={:?}",original.local.turn_cost,copy.local.turn_cost,copy.local.randomized_cost);
    assert_eq!(copy.local.turn_cost,Some(3),"draw callback must capture the copy before Confused rewrites original cost");
    assert_eq!(copy.local.randomized_cost,None,"captured copy must retain pre-Confused randomized-cost state");
}

#[test]
fn independent_review_endless_agony_opening() {
    let c=fight("endless_agony",0);
    let copies=c.public_full().unwrap().hand.iter().filter(|card|card.id==CardId::EndlessAgony).count();
    println!("Endless Agony opening instances={copies}");
    assert_eq!(copies,2,"opening draw must invoke Endless Agony actual-draw callback once");
}
#[test]
fn independent_review_grand_finale_cost_legality() {
    let mut c=fight("setup",0);
    add(&mut c,"defend_g",130,Zone::Draw);
    add(&mut c,"grand_finale",140,Zone::Draw);
    c.state.player_full_powers.push(FullPowerEntry{id:FullPowerId("confused"),state:PowerState{amount:1,skip_first:false,monster_source_grace:false}});
    c.enqueue_native_effects(vec![Effect{source:Source::None,target:Some(Actor::Player),kind:EffectKind::Draw(1)}],None).unwrap();
    let finale=live(&c,140);
    println!("Grand Finale draw remaining={} local={:?} public={:?}",c.state.piles.draw.len(),finale.local,c.public_full().unwrap().hand.iter().find(|card|card.instance.0==140));
    assert!(finale.local.randomized_cost.is_some(),"Confused randomizes nonnegative card cost even while Grand Finale cannot be played");
}
#[test]
fn independent_review_setup_colorless_free_flag() {
    let mut c=fight("setup",0);
    add(&mut c,"apotheosis",140,Zone::Hand);
    play(&mut c,3,false);
    c.step_full(&CombatAction::SelectCard{instance:140}).unwrap();
    let card=c.public_full().unwrap().known_draw.into_iter().find(|row|row.card.instance.0==140).unwrap().card;
    assert_eq!(card.effective_cost,Some(0));
    println!("Setup Apotheosis local={:?} public values={:?}",live(&c,140).local,card.values);
    assert!(card.values.unwrap().free_play,"public free-play field must include Setup one-play waiver as green cards do");
}


#[test]
fn m15_review_endless_snapshot_retains_source_and_template_on_id_exhaustion() {
    let mut c=fight("setup",0);add(&mut c,"endless_agony",140,Zone::Draw);
    c.state.piles.draw.last_mut().unwrap().full_mut().unwrap().local.turn_cost=Some(3);
    c.state.player_full_powers.push(FullPowerEntry{id:FullPowerId("confused"),state:PowerState{amount:1,skip_first:false,monster_source_grace:false}});
    c.state.card_ids_issued=(1..=999).map(|n|card_state::handle(CardInstanceId(n))).collect();
    let result=c.enqueue_native_effects(vec![Effect{source:Source::None,target:Some(Actor::Player),kind:EffectKind::Draw(1)}],None);
    assert!(matches!(result,Err(crate::run::RunError::Faulted(_))));
    let current=c.state.current.as_ref().unwrap();assert_eq!(current.source,Source::Card(card_state::handle(CardInstanceId(140))));
    let EffectKind::Silent(cards::SilentEffect::EndlessAgonyCopy{card,template})=&current.kind else{panic!("captured draw-copy must retain first-fault custody")};
    assert_eq!(*card,card_state::handle(CardInstanceId(140)));assert_eq!(template.local.turn_cost,Some(3));assert_eq!(template.local.randomized_cost,None);
    assert_eq!(live(&c,140).local.turn_cost,Some(2));assert_eq!(c.state.card_ids_issued.len(),999);
}

#[test]
fn m15_review_grand_finale_unconditional_price_bullet_time_reset_and_legality() {
    let mut c=fight("bullet_time",0);add(&mut c,"grand_finale",140,Zone::Hand);add(&mut c,"defend_g",130,Zone::Draw);
    let view=|c:&Combat|c.public_full().unwrap().hand.into_iter().find(|card|card.instance.0==140).unwrap();
    assert_eq!(view(&c).effective_cost,Some(0));
    let action=CombatAction::PlayCard{instance:140,target:None};assert!(!c.legal_actions_full().unwrap().contains(&action));
    c.state.piles.hand.iter_mut().find(|card|card.handle()==card_state::handle(CardInstanceId(140))).unwrap().full_mut().unwrap().local.turn_cost=Some(2);
    play(&mut c,3,false);assert_eq!(view(&c).effective_cost,Some(0));assert!(!c.legal_actions_full().unwrap().contains(&action));
    c.step_full(&CombatAction::EndTurn).unwrap();assert_eq!(live(&c,140).local.turn_cost,Some(0),"turn reset uses known base0 regardless of earlier play predicate");
}

#[test]
fn m15_review_setup_forethought_publish_waiver_for_every_value_family() {
    for source in ["setup","forethought"] {for target in ["apotheosis","hand_of_greed","mayhem","distraction"] {
        let mut c=fight(source,0);
        let instance=if target=="distraction" {
            let instance=piles::next_generated_instance(&c.state).unwrap();
            let card=card_state::generated_with_combat_history(CardId::Distraction,instance,0,c.state.damage_events_this_combat).unwrap();
            piles::insert_generated(&mut c.state,card,CardDestination::Hand).unwrap();
            instance.0
        }else{add(&mut c,target,140,Zone::Hand);140};
        play(&mut c,3,false);c.step_full(&CombatAction::SelectCard{instance}).unwrap();
        let card=c.public_full().unwrap().known_draw.into_iter().find(|row|row.card.instance.0==instance).unwrap().card;
        assert_eq!(card.effective_cost,Some(0));assert!(card.values.unwrap().free_play,"{source}/{target} carries a one-play waiver");
    }}
    let mut ordinary=fight("setup",0);add(&mut ordinary,"mayhem",140,Zone::Hand);
    ordinary.state.piles.hand.iter_mut().find(|c|c.handle()==card_state::handle(CardInstanceId(140))).unwrap().full_mut().unwrap().local.turn_cost=Some(0);
    let card=ordinary.public_full().unwrap().hand.into_iter().find(|card|card.instance.0==140).unwrap();
    assert_eq!(card.effective_cost,Some(0));assert!(!card.values.unwrap().free_play,"ordinary numeric zero is not a waiver");
}
