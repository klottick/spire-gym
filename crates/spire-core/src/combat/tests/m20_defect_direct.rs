use super::super::*;
use crate::combat::cards::DefectEffect;
use crate::content::{CardCost, CardId, CardKeywords, CardTag, CardTarget, Catalog};
use super::full_runtime::entered_combat;

#[derive(Clone, Copy)]
struct DirectRow {
    id: CardId,
    cost: [i32; 2],
    target: CardTarget,
    keywords: [CardKeywords; 2],
    damage: Option<[i32; 2]>,
    block: Option<[i32; 2]>,
    magic: Option<[i32; 2]>,
}

fn row(
    id: CardId, cost: [i32; 2], target: CardTarget, keywords: [CardKeywords; 2],
    damage: Option<[i32; 2]>, block: Option<[i32; 2]>, magic: Option<[i32; 2]>,
) -> DirectRow { DirectRow { id, cost, target, keywords, damage, block, magic } }

fn fight(id: CardId, upgrades: u32) -> Combat {
    let mut combat = entered_combat(9, 9, |card| {
        *card = permanent_live_card(id, card.instance, upgrades);
    });
    combat.state.enemies[0].hp = 500;
    combat.state.enemies[0].max_hp = 500;
    combat.state.silent.source_character = Some(crate::content::Character::Defect);
    combat.state.character = characters::CharacterState::for_character(crate::content::Character::Defect);
    combat
}

fn card_view_at(combat: &Combat, instance: u64) -> LiveCardView {
    combat.public_full().unwrap().hand.into_iter().find(|card| card.instance.0 == instance)
        .expect("test card remains in the hand before use")
}

fn card_view(combat: &Combat) -> LiveCardView { card_view_at(combat, 3) }

fn ensure_draw_cards(combat: &mut Combat, count: usize) {
    while combat.state.piles.draw.len() < count {
        let instance = CardInstanceId(1000 + combat.state.card_ids_issued.len() as u64);
        let card = permanent_live_card(CardId::DefendBlue, instance, 0);
        assert!(combat.state.card_ids_issued.insert(super::super::card_state::handle(instance)));
        combat.state.piles.draw.push(model::OwnedCard::Full(card));
    }
}

fn play_at(combat: &mut Combat, instance: u64, id: CardId, target: CardTarget) {
    let target = (target == CardTarget::Enemy).then_some(TargetId(1));
    combat.step_full(&CombatAction::PlayCard { instance, target })
        .unwrap_or_else(|error| panic!("{id:?}: {error:?}"));
    assert!(combat.fault_record().is_none(), "{id:?} must resolve without a retained fault");
}

fn play(combat: &mut Combat, id: CardId, target: CardTarget) { play_at(combat, 3, id, target); }

fn replace_hand_card(combat: &mut Combat, instance: u64, id: CardId, upgrades: u32) {
    let owned = combat.state.piles.hand.iter_mut().find(|owned| owned.full().is_some_and(|card| card.instance.0 == instance))
        .expect("replacement card owner in hand");
    *owned = model::OwnedCard::Full(permanent_live_card(id, CardInstanceId(instance), upgrades));
}

fn add_active_enemy(combat: &mut Combat, handle: &str, hp: i32) {
    let mut enemy = combat.state.enemies[0].clone();
    enemy.instance_id = EnemyHandle::parse(handle).unwrap();
    enemy.hp = hp;
    enemy.max_hp = hp;
    enemy.block = 0;
    let slot = combat.state.actor_slots.len() as u8;
    combat.state.actor_slots.insert(enemy.instance_id.clone(), slot);
    combat.state.enemies.push(enemy);
}

#[test]
fn m20_defect_direct_base_plus_and_flags() {
    use CardId::*;
    use CardKeywords as K;
    use CardTarget as T;
    let rows = [
        row(BeamCell, [0,0], T::Enemy, [K::NONE,K::NONE], Some([3,4]), None, Some([1,2])),
        row(ChargeBattery, [1,1], T::SelfTarget, [K::NONE,K::NONE], None, Some([7,10]), None),
        row(GoForTheEyes, [0,0], T::Enemy, [K::NONE,K::NONE], Some([3,4]), None, Some([1,2])),
        row(Leap, [1,1], T::SelfTarget, [K::NONE,K::NONE], None, Some([9,12]), None),
        row(Stack, [1,1], T::SelfTarget, [K::NONE,K::NONE], None, Some([0,3]), None),
        row(SweepingBeam, [1,1], T::AllEnemies, [K::NONE,K::NONE], Some([6,9]), None, Some([1,1])),
        row(Turbo, [0,0], T::SelfTarget, [K::NONE,K::NONE], None, None, Some([2,3])),
        row(Aggregate, [1,1], T::SelfTarget, [K::NONE,K::NONE], None, None, None),
        row(AutoShields, [1,1], T::SelfTarget, [K::NONE,K::NONE], None, Some([11,15]), None),
        row(Blizzard, [1,1], T::AllEnemies, [K::NONE,K::NONE], Some([0,0]), None, None),
        row(BootSequence, [0,0], T::SelfTarget, [K::EXHAUST_INNATE,K::EXHAUST_INNATE], None, Some([10,13]), None),
        row(Chaos, [1,1], T::SelfTarget, [K::NONE,K::NONE], None, None, Some([1,2])),
        row(Chill, [0,0], T::SelfTarget, [K::EXHAUST,K::EXHAUST_INNATE], None, None, None),
        row(DoomAndGloom, [2,2], T::AllEnemies, [K::NONE,K::NONE], Some([10,14]), None, None),
        row(DoubleEnergy, [1,0], T::SelfTarget, [K::EXHAUST,K::EXHAUST], None, None, None),
        row(Glacier, [2,2], T::SelfTarget, [K::NONE,K::NONE], None, Some([7,10]), Some([2,2])),
        row(Melter, [1,1], T::Enemy, [K::NONE,K::NONE], Some([10,14]), None, None),
        row(Overclock, [0,0], T::SelfTarget, [K::NONE,K::NONE], None, None, Some([2,3])),
        row(Reprogram, [1,1], T::None, [K::NONE,K::NONE], None, None, None),
        row(RipAndTear, [1,1], T::AllEnemies, [K::NONE,K::NONE], Some([7,9]), None, None),
        row(Skim, [1,1], T::None, [K::NONE,K::NONE], None, None, Some([3,4])),
        row(Sunder, [3,3], T::Enemy, [K::NONE,K::NONE], Some([24,32]), None, None),
        row(CoreSurge, [1,1], T::Enemy, [K::EXHAUST,K::EXHAUST], Some([11,15]), None, None),
        row(Hyperbeam, [2,2], T::AllEnemies, [K::NONE,K::NONE], Some([26,34]), None, None),
        row(MeteorStrike, [5,5], T::Enemy, [K::NONE,K::NONE], Some([24,30]), None, None),
        row(Rainbow, [2,2], T::SelfTarget, [K::EXHAUST,K::NONE], None, None, None),
        row(ThunderStrike, [3,3], T::AllEnemies, [K::NONE,K::NONE], Some([7,9]), None, Some([0,0])),
    ];
    assert_eq!(rows.len(), 27);
    assert_eq!(Catalog::standard().card(MeteorStrike).unwrap().blue_metadata.unwrap().tags, &[CardTag::Strike]);
    assert_eq!(Catalog::standard().card(ThunderStrike).unwrap().blue_metadata.unwrap().tags, &[CardTag::Strike]);

    for expected in rows {
        let definition = Catalog::standard().card(expected.id).unwrap();
        let metadata = definition.blue_metadata.unwrap();
        for up in 0..=1 {
            let index = up as usize;
            let face = if up == 0 { metadata.base } else { metadata.upgraded };
            assert_eq!(face.cost, CardCost::Energy(expected.cost[index] as u8), "{:?}+{up} cost metadata", expected.id);
            assert_eq!(face.target, expected.target, "{:?}+{up} target metadata", expected.id);
            assert_eq!(face.keywords, expected.keywords[index], "{:?}+{up} keyword metadata", expected.id);

            let mut combat = fight(expected.id, up);
            let before_enemy_hp = combat.state.enemies[0].hp;
            let before_block = combat.state.player.block;
            let view = card_view(&combat);
            assert_eq!(view.effective_cost, Some(expected.cost[index]), "{:?}+{up} public cost", expected.id);
            let values = view.values.unwrap();
            if let Some(pair) = expected.damage { assert_eq!(values.base_damage, Some(pair[index]), "{:?}+{up} public damage", expected.id); }
            if let Some(pair) = expected.block { assert_eq!(values.base_block, Some(pair[index]), "{:?}+{up} public Block", expected.id); }
            if let Some(pair) = expected.magic { assert_eq!(values.magic_number, Some(pair[index]), "{:?}+{up} public magic number", expected.id); }
            assert_eq!(values.exhaust_on_finish, face.keywords.exhaust, "{:?}+{up} Exhaust", expected.id);
            assert_eq!(values.innate, face.keywords.innate, "{:?}+{up} Innate", expected.id);
            assert_eq!(values.ethereal, face.keywords.ethereal, "{:?}+{up} Ethereal", expected.id);

            play(&mut combat, expected.id, expected.target);
            let hp_loss = before_enemy_hp - combat.state.enemies[0].hp;
            let expected_loss = match expected.id {
                BeamCell|GoForTheEyes|Melter|Sunder|CoreSurge|MeteorStrike => expected.damage.unwrap()[index],
                SweepingBeam|DoomAndGloom|Hyperbeam|Blizzard => expected.damage.unwrap()[index],
                RipAndTear => expected.damage.unwrap()[index] * 2,
                _ => 0,
            };
            assert_eq!(hp_loss, expected_loss, "{:?}+{up} resolved damage", expected.id);
            if let Some(pair) = expected.block {
                if expected.id != Stack { assert_eq!(combat.state.player.block - before_block, pair[index], "{:?}+{up} resolved Block", expected.id); }
            }
            let owned = combat.state.piles.exhaust.iter().chain(&combat.state.piles.discard)
                .find(|owned| owned.full().is_some_and(|card| card.instance.0 == 3))
                .expect("played direct card reaches its shared destination");
            assert_eq!(owned.full().unwrap().local.exhaust_on_finish, face.keywords.exhaust, "{:?}+{up} final disposition", expected.id);
        }
    }
}

#[test]
fn m20_defect_direct_capture_times_and_successful_channels() {
    for (up, size, expected_gain) in [(0,0,0),(0,3,0),(0,4,1),(0,8,2),(1,0,0),(1,3,1),(1,4,1),(1,8,2)] {
        let mut combat = fight(CardId::Aggregate, up);
        ensure_draw_cards(&mut combat, 8);
        let card = combat.state.piles.hand.iter().find(|owned| owned.full().is_some_and(|card| card.instance.0 == 3)).unwrap().clone();
        let mut other_cards = combat.state.piles.hand.drain(..).chain(combat.state.piles.draw.drain(..))
            .chain(combat.state.piles.discard.drain(..)).filter(|owned| owned.handle() != card.handle()).collect::<Vec<_>>();
        combat.state.piles.hand.push(card);
        combat.state.piles.draw.clear();
        combat.state.piles.discard.clear();
        for _ in 0..size { if !other_cards.is_empty() { combat.state.piles.draw.push(other_cards.remove(0)); } }
        combat.state.piles.discard.extend(other_cards);
        assert_eq!(combat.state.piles.draw.len(), size, "Aggregate fixture draw size before use");
        play(&mut combat, CardId::Aggregate, CardTarget::SelfTarget);
        assert_eq!(combat.state.player.energy, 9 - 1 + expected_gain,
            "Aggregate+{up}, draw {size}, pile after use {}", combat.state.piles.draw.len());
    }

    for (up, count, expected) in [(0,0,0),(0,2,2),(1,0,3),(1,2,5)] {
        let mut combat = fight(CardId::Stack, up);
        let card = combat.state.piles.hand.iter().find(|owned| owned.full().is_some_and(|card| card.instance.0 == 3)).unwrap().clone();
        let mut others = combat.state.piles.hand.drain(..).chain(combat.state.piles.draw.drain(..))
            .chain(combat.state.piles.discard.drain(..)).filter(|owned| owned.handle() != card.handle()).collect::<Vec<_>>();
        combat.state.piles.hand.push(card);
        combat.state.piles.discard.extend(others.drain(..count));
        super::super::piles::refresh_hand_values(&mut combat.state);
        assert_eq!(card_view(&combat).values.unwrap().base_block, Some(expected));
        play(&mut combat, CardId::Stack, CardTarget::SelfTarget);
        assert_eq!(combat.state.player.block, expected);
    }

    for (up, initial_block, expected) in [(0,0,11),(0,1,1),(1,0,15),(1,1,1)] {
        let mut combat = fight(CardId::AutoShields, up);
        combat.state.player.block = initial_block;
        play(&mut combat, CardId::AutoShields, CardTarget::SelfTarget);
        assert_eq!(combat.state.player.block, expected, "Auto-Shields+{up} at block {initial_block}");
    }

    for (id, up, slots, expected) in [
        (CardId::Glacier,0,3,2),(CardId::Glacier,1,3,2),
        (CardId::Glacier,0,0,0),(CardId::Chill,0,3,2),(CardId::Chill,1,3,2),
    ] {
        let mut combat = fight(id, up);
        combat.state.character.orb_slots.resize(slots, None);
        if id == CardId::Chill { add_active_enemy(&mut combat,"enemy-2",500); }
        play(&mut combat, id, CardTarget::SelfTarget);
        assert_eq!(combat.state.defect.frost_channeled, expected, "{id:?}+{up} history with {slots} slots");
        assert_eq!(combat.state.character.orb_slots.iter().flatten().filter(|orb| orb.kind == characters::OrbKind::Frost).count(), expected as usize);
    }

    for (glacier_up,blizzard_up,damage) in [(0,0,4),(1,0,4),(0,1,6),(1,1,6)] {
        let mut combat = fight(CardId::Glacier,glacier_up);
        replace_hand_card(&mut combat,1,CardId::Blizzard,blizzard_up);
        let before = combat.state.enemies[0].hp;
        play(&mut combat,CardId::Glacier,CardTarget::SelfTarget);
        assert_eq!(combat.state.defect.frost_channeled,2);
        assert_eq!(card_view_at(&combat,1).values.unwrap().base_damage,Some(damage),
            "successful Glacier installs refresh the live Blizzard value");
        play_at(&mut combat,1,CardId::Blizzard,CardTarget::AllEnemies);
        assert_eq!(before-combat.state.enemies[0].hp,damage,"Blizzard captures the refreshed Frost count at use");
    }

    for up in 0..=1 {
        let mut combat = fight(CardId::ThunderStrike,up);
        replace_hand_card(&mut combat,1,CardId::BallLightning,0);
        let before = combat.state.enemies[0].hp;
        play_at(&mut combat,1,CardId::BallLightning,CardTarget::Enemy);
        assert_eq!(combat.state.defect.lightning_channeled,1);
        assert_eq!(card_view(&combat).values.unwrap().magic_number,Some(1),
            "a successful Lightning installation refreshes Thunder Strike's public magic value");
        play(&mut combat,CardId::ThunderStrike,CardTarget::AllEnemies);
        assert_eq!(before-combat.state.enemies[0].hp,7+if up==0{7}else{9});
        assert_eq!(combat.state.card_rng.as_ref().unwrap().word_index(),1,
            "source random(0,0) consumes one card RNG word for the singleton target");
    }
}

#[test]
fn m20_defect_direct_resource_status_and_random_fault_prefixes() {
    for (up, cost, energy) in [(0,1,16),(1,0,18)] {
        let mut combat = fight(CardId::DoubleEnergy, up);
        play(&mut combat, CardId::DoubleEnergy, CardTarget::SelfTarget);
        assert_eq!(combat.state.player.energy, energy, "Double Energy pays {cost} before doubling");
    }

    for (up, damage, block) in [(0,10,8),(1,14,8)] {
        let mut combat = fight(CardId::Melter, up);
        combat.state.enemies[0].block = block;
        let spiker = crate::content::EnemyId::Spiker;
        combat.state.enemies[0].enemy_id = model::ActorEnemyId::Full(spiker);
        combat.state.enemies[0].full_powers.push(FullPowerEntry {
            id: FullPowerId("thorns"), state: PowerState { amount: 3, skip_first: false, monster_source_grace: false },
        });
        let hp = combat.state.player.hp;
        play(&mut combat, CardId::Melter, CardTarget::Enemy);
        assert_eq!(combat.state.enemies[0].block, 0);
        assert_eq!(combat.state.enemies[0].hp, 500 - damage);
        assert_eq!(combat.state.player.hp, hp - 3, "block removal has no separate damage/thorns callback");
    }

    for up in 0..=1 {
        let mut combat = fight(CardId::Turbo, up);
        let before = combat.state.player.energy;
        play(&mut combat, CardId::Turbo, CardTarget::SelfTarget);
        assert_eq!(combat.state.player.energy, before + 2 + up as i32);
        assert!(combat.state.piles.discard.iter().any(|owned| owned.full().is_some_and(|card| card.id == CardId::Void && card.upgrades == 0)),
            "Turbo creates a base Void in discard");
    }
    let mut overclock = fight(CardId::Overclock, 1);
    play(&mut overclock, CardId::Overclock, CardTarget::SelfTarget);
    assert!(overclock.state.piles.discard.iter().any(|owned| owned.full().is_some_and(|card| card.id == CardId::Burn && card.upgrades == 0)),
        "Overclock+ still creates a base Burn");

    for (up, damage) in [(0,7),(1,9)] {
        let mut combat = fight(CardId::RipAndTear, up);
        let mut second = combat.state.enemies[0].clone();
        second.instance_id = EnemyHandle::parse("enemy-2").unwrap();
        second.hp = 500;
        second.max_hp = 500;
        combat.state.actor_slots.insert(second.instance_id.clone(),1);
        combat.state.enemies.push(second);
        let stream = combat.state.card_rng.as_ref().unwrap();
        combat.state.card_rng = Some(rng::StreamState::new(stream.id().clone(), [0;32], (1u128 << 64) - 1).unwrap());
        let total_hp = combat.state.enemies.iter().map(|enemy| enemy.hp).sum::<i32>();
        assert!(combat.step_full(&CombatAction::PlayCard { instance:3, target:None }).is_err());
        assert_eq!(total_hp - combat.state.enemies.iter().map(|enemy| enemy.hp).sum::<i32>(), damage,
            "first random hit commits before the second selection faults");
        assert_eq!(combat.state.player.energy, 8);
        assert_eq!(combat.state.fault.as_ref().unwrap().kind, FaultKind::RngExhausted,
            "full retained fault: {:?}", combat.state.fault.as_ref().unwrap());
        assert!(combat.state.current.as_ref().is_some_and(|effect|
            matches!(effect.kind, EffectKind::Defect(_)) && format!("{:?}", effect.kind).contains("remaining: 1")));
        let snapshot = combat.snapshot();
        assert!(combat.step_full(&CombatAction::EndTurn).is_err());
        assert_eq!(combat.snapshot(), snapshot, "retry cannot replay a committed hit or selection");
    }
}

#[test]
fn m20_defect_direct_chaos_capture_keeps_sampled_prefix_on_later_rng_fault() {
    let mut combat = fight(CardId::Chaos, 1);
    combat.state.character.orb_slots.clear();
    let stream = combat.state.card_rng.as_ref().unwrap();
    combat.state.card_rng = Some(rng::StreamState::new(stream.id().clone(), [0;32], (1u128 << 64) - 1).unwrap());
    assert!(combat.step_full(&CombatAction::PlayCard { instance:3, target:None }).is_err());
    assert_eq!(combat.state.fault.as_ref().unwrap().kind, FaultKind::RngExhausted);
    assert_eq!(combat.state.player.energy, 8, "payment remains committed before use capture");
    assert_eq!(combat.state.character.orb_slots.len(), 0);
    assert_eq!(combat.state.defect.frost_channeled, 0);
    assert_eq!(combat.state.defect.lightning_channeled, 0);
    let current = combat.state.current.as_ref().expect("failed captured use remains current");
    let EffectKind::Defect(DefectEffect::Direct(work)) = &current.kind else { panic!("Chaos capture remains typed") };
    let captured = format!("{work:?}");
    assert!(captured.contains("requested: 2"));
    let prefix = captured.split("orbs: [").nth(1).unwrap().split(']').next().unwrap();
    assert_eq!(prefix.split(',').count(), 1, "first independently sampled orb is retained");
    assert_eq!(combat.state.card_rng.as_ref().unwrap().word_index(), 1u128 << 64);
    let snapshot = combat.snapshot();
    assert!(combat.step_full(&CombatAction::EndTurn).is_err());
    assert_eq!(combat.snapshot(), snapshot, "retry cannot resample or channel the accepted prefix");
}

#[test]
fn m20_defect_direct_chaos_samples_at_use_even_without_orb_slots() {
    use characters::OrbKind::*;
    for up in 0..=1 {
        let mut combat = fight(CardId::Chaos,up);
        let before = combat.state.card_rng.as_ref().unwrap().clone();
        let count = if up==0 {1} else {2};
        let mut expected_stream = before;
        let mut expected_orbs = Vec::new();
        for _ in 0..count {
            let (value,next)=rng::uniform_below(&expected_stream,4).unwrap();
            expected_orbs.push(match value {0=>Dark,1=>Frost,2=>Lightning,_=>Plasma});
            expected_stream=next;
        }
        play(&mut combat,CardId::Chaos,CardTarget::SelfTarget);
        assert_eq!(combat.state.card_rng.as_ref(),Some(&expected_stream));
        assert_eq!(combat.state.character.orb_slots.iter().filter_map(|slot|slot.map(|orb|orb.kind)).collect::<Vec<_>>(),expected_orbs);
    }

    let mut no_slots = fight(CardId::Chaos,1);
    no_slots.state.character.orb_slots.clear();
    let before=no_slots.state.card_rng.as_ref().unwrap().clone();
    let (_,one)=rng::uniform_below(&before,4).unwrap();
    let (_,two)=rng::uniform_below(&one,4).unwrap();
    play(&mut no_slots,CardId::Chaos,CardTarget::SelfTarget);
    assert_eq!(no_slots.state.card_rng.as_ref(),Some(&two),"zero capacity does not skip use-time sampling");
    assert!(no_slots.state.character.orb_slots.is_empty());
    assert_eq!((no_slots.state.defect.frost_channeled,no_slots.state.defect.lightning_channeled),(0,0));
}

#[test]
fn m20_defect_direct_ordered_powers_channels_and_sunder_predicate() {
    for up in 0..=1 {
        let mut reprogram=fight(CardId::Reprogram,up);
        play(&mut reprogram,CardId::Reprogram,CardTarget::None);
        let amount=1+up as i32;
        assert_eq!(reprogram.state.character.focus,-amount);
        assert_eq!(reprogram.state.player_full_powers.iter().find(|power|power.id.0=="strength").unwrap().state.amount,amount);
        assert_eq!(reprogram.state.player_full_powers.iter().find(|power|power.id.0=="dexterity").unwrap().state.amount,amount);

        let mut core=fight(CardId::CoreSurge,up);
        play(&mut core,CardId::CoreSurge,CardTarget::Enemy);
        assert_eq!(core.state.player_full_powers.iter().find(|power|power.id.0=="artifact").unwrap().state.amount,1);

        let mut hyper=fight(CardId::Hyperbeam,up);
        let before=hyper.state.enemies[0].hp;
        play(&mut hyper,CardId::Hyperbeam,CardTarget::AllEnemies);
        assert_eq!(before-hyper.state.enemies[0].hp,if up==0{26}else{34});
        assert_eq!(hyper.state.character.focus,-3);
    }

    let mut meteor=fight(CardId::MeteorStrike,0);
    let before=meteor.state.enemies[0].hp;
    play(&mut meteor,CardId::MeteorStrike,CardTarget::Enemy);
    assert_eq!(before-meteor.state.enemies[0].hp,24);
    assert_eq!(meteor.state.character.orb_slots.iter().flatten().map(|orb|orb.kind).collect::<Vec<_>>(),vec![characters::OrbKind::Plasma;3]);
    assert_eq!((meteor.state.defect.frost_channeled,meteor.state.defect.lightning_channeled),(0,0));

    let mut rainbow=fight(CardId::Rainbow,0);
    play(&mut rainbow,CardId::Rainbow,CardTarget::SelfTarget);
    assert_eq!(rainbow.state.character.orb_slots.iter().flatten().map(|orb|orb.kind).collect::<Vec<_>>(),
        vec![characters::OrbKind::Lightning,characters::OrbKind::Frost,characters::OrbKind::Dark]);
    assert_eq!((rainbow.state.defect.lightning_channeled,rainbow.state.defect.frost_channeled),(1,1));
    assert!(rainbow.state.piles.exhaust.iter().any(|owned|owned.full().is_some_and(|card|card.id==CardId::Rainbow&&card.local.exhaust_on_finish)));

    for (up,initial_hp,expected_energy,remaining_hp) in [(0,20,9,0),(0,30,6,6),(1,20,9,0),(1,40,6,8)] {
        let mut sunder=fight(CardId::Sunder,up);
        sunder.state.enemies[0].hp=initial_hp;
        add_active_enemy(&mut sunder,"enemy-2",500);
        play(&mut sunder,CardId::Sunder,CardTarget::Enemy);
        assert_eq!(sunder.state.enemies[0].hp,remaining_hp,"Sunder+{up}, starting HP {initial_hp}");
        assert_eq!(sunder.state.player.energy,expected_energy,"Sunder earns energy only for its direct HP<=0 predicate");
    }
}

// Bounded supplied core compositions, not canonical acquisition or episodes.
// Removing Turbo's shared add-energy cap must fail these real-play controls.
fn i1_energy_fixture(last:CardId,upgrades:u32,spend:Option<(CardId,i32)>,relics:Vec<crate::content::RelicId>,potion:Option<crate::content::PotionId>,m20:bool)->Combat{
    use crate::{content::{Character,EncounterId},items::{CombatItemContext,NativeBattleContext},run::RunRng};
    let doubles=if spend.is_some(){8}else{9};let initial=if spend.is_some(){4}else{3};
    let mut cards=(1..=doubles).map(|i|permanent_live_card(CardId::DoubleEnergy,CardInstanceId(i),1)).collect::<Vec<_>>();
    if let Some((id,_))=spend{cards.push(permanent_live_card(id,CardInstanceId(9),0));}
    cards.push(permanent_live_card(last,CardInstanceId(10),upgrades));
    let entry=enemies::setup(EncounterId::TwoLouse,1,0,&mut RunRng::new([43;32],rng::Partition::Test,0),enemies::SetupContext{
        player:PlayerState{hp:75,max_hp:75,block:0,energy:initial,max_energy:initial,draw_per_turn:5,powers:vec![]},player_powers:vec![],character:Character::Defect,
        cards,opening_draw:10,encounter_visit:0}).unwrap();
    let mut items=Some(CombatItemContext::new(relics,vec![potion,None,None]).with_battle_context(NativeBattleContext{m20,..Default::default()}));
    let mut c=Combat::enter_with_item_context(entry,&mut items).unwrap();assert_eq!(c.state.player.energy,initial);
    for instance in 1..=doubles{c.step_full(&CombatAction::PlayCard{instance,target:None}).unwrap();}
    assert_eq!(c.state.player.energy,999,"real DoubleEnergy+ plays establish the cap without injecting energy");
    if let Some((_,want))=spend{c.step_full(&CombatAction::PlayCard{instance:9,target:None}).unwrap();assert_eq!(c.state.player.energy,want,"actual card payment establishes the below-cap boundary");}
    c
}
fn i1_turbo_cap_case(upgrades:u32,spend:Option<(CardId,i32)>){
    let doubles=if spend.is_some(){8}else{9};let mut c=i1_energy_fixture(CardId::Turbo,upgrades,spend,vec![],None,true);
    assert!(c.legal_actions_full().unwrap().contains(&CombatAction::PlayCard{instance:10,target:None}));
    assert_eq!(c.state.card_ids_issued.len(),10);c.step_full(&CombatAction::PlayCard{instance:10,target:None}).expect("Turbo saturates before continuing to exactly one Void");
    assert_eq!(c.state.player.energy,999);assert_eq!(c.state.phase,Phase::Decision);assert!(c.state.pending.is_empty()&&c.state.current.is_none()&&c.state.in_play.is_none());
    let voids=c.state.piles.discard.iter().filter_map(model::OwnedCard::full).filter(|card|card.id==CardId::Void).collect::<Vec<_>>();assert_eq!(voids.len(),1);assert_eq!((voids[0].instance,voids[0].upgrades),(CardInstanceId(11),0));assert!(matches!(voids[0].origin,CombatCardRef::CombatLocal(_)));
    assert_eq!(c.state.card_ids_issued.len(),11);assert!(c.state.piles.discard.iter().any(|card|card.full().is_some_and(|card|card.id==CardId::Turbo&&card.instance==CardInstanceId(10))));assert_eq!(c.state.piles.exhaust.len(),doubles as usize);assert!(engine::validate_runtime(&c.state).is_ok());
    let before=c.clone();assert!(c.step_full(&CombatAction::PlayCard{instance:10,target:None}).is_err());assert_eq!(c,before,"stale retry issues no second Void or energy gain");
    assert!(c.enqueue_native_effects(vec![Effect{source:Source::Card(super::super::card_state::handle(CardInstanceId(10))),target:Some(Actor::Player),kind:EffectKind::GainEnergy(if upgrades>0{3}else{2})}],None).is_err());assert_eq!(c,before,"retired Turbo is not a fresh capped-energy source");
}
#[test]
fn m20_defect_direct_turbo_base_saturates_and_finishes_void_at_energy_cap(){
    i1_turbo_cap_case(0,None);i1_turbo_cap_case(0,Some((CardId::Glacier,997)));i1_turbo_cap_case(0,Some((CardId::Coolheaded,998)));
}
#[test]
fn m20_defect_direct_turbo_plus_saturates_and_finishes_base_void_at_energy_cap(){
    i1_turbo_cap_case(1,None);i1_turbo_cap_case(1,Some((CardId::ThunderStrike,996)));i1_turbo_cap_case(1,Some((CardId::Glacier,997)));i1_turbo_cap_case(1,Some((CardId::Coolheaded,998)));
}
#[test]
fn m20_defect_direct_sunder_free_refund_saturates_same_energy_owner(){
    let mut c=i1_energy_fixture(CardId::Sunder,0,Some((CardId::Madness,998)),vec![],None,true);
    let action=CombatAction::PlayCard{instance:10,target:Some(TargetId(1))};assert!(c.legal_actions_full().unwrap().contains(&action));assert_eq!(card_view_at(&c,10).effective_cost,Some(0));
    c.step_full(&action).expect("M20 free Sunder's earned3 refund uses the same add-energy cap");assert_eq!(c.state.player.energy,999);assert_eq!(c.state.enemies[0].hp,0);assert!(c.state.enemies[1].hp>0);assert!(c.state.outcome.is_none());assert!(engine::validate_runtime(&c.state).is_ok());assert!(c.state.current.is_none()&&c.state.pending.is_empty());
}
#[test]
fn m20_defect_direct_energy_potion_saturates_same_energy_owner_base_and_bark(){
    use crate::content::{PotionId,RelicId};
    for relics in [vec![],vec![RelicId::SacredBark]]{
        let mut c=i1_energy_fixture(CardId::DefendBlue,0,None,relics,Some(PotionId::EnergyPotion),true);let issued=c.state.card_ids_issued.clone();
        let prepared=crate::items::prepare_combat_potion(&c,0,None,false).expect("actual capped potion is preparable");
        assert_eq!(prepared.effects,vec![Effect{source:Source::Potion{slot:0,id:PotionId::EnergyPotion},target:Some(Actor::Player),kind:EffectKind::GainEnergy(if c.item_context().unwrap().owns_relic(RelicId::SacredBark){4}else{2})}]);assert_eq!(prepared.potions,vec![None;3]);assert_eq!(c.item_context().unwrap().potions()[0],Some(PotionId::EnergyPotion));assert!(c.legal_actions_full().unwrap().contains(&CombatAction::UsePotion{slot:0,target:None}));
        c.step_full(&CombatAction::UsePotion{slot:0,target:None}).expect("M20 EnergyPotion caps before committing exactly one live slot use");assert_eq!(c.state.player.energy,999);assert_eq!(c.item_context().unwrap().potions(),&[None;3]);assert_eq!(c.state.card_ids_issued,issued);assert!(engine::validate_runtime(&c.state).is_ok());let before=c.clone();assert!(c.step_full(&CombatAction::UsePotion{slot:0,target:None}).is_err());assert_eq!(c,before);
    }
}
#[test]
fn m20_defect_direct_energy_cap_preserves_old_profiles_and_rejects_unowned_work(){
    use crate::content::PotionId;
    let mut old=i1_energy_fixture(CardId::Sunder,0,Some((CardId::Madness,998)),vec![],None,false);
    let action=CombatAction::PlayCard{instance:10,target:Some(TargetId(1))};let error=old.step_full(&action).expect_err("old Sunder overflow remains frozen");assert_eq!(old.state.player.energy,998);assert_eq!(old.state.enemies[0].hp,0);assert!(matches!(old.state.current,Some(Effect{kind:EffectKind::GainEnergy(3),..})));let retained=old.clone();assert_eq!(old.step_full(&action),Err(error));assert_eq!(old,retained);
    let mut old=i1_energy_fixture(CardId::DefendBlue,0,None,vec![],Some(PotionId::EnergyPotion),false);let before=old.clone();assert!(old.step_full(&CombatAction::UsePotion{slot:0,target:None}).is_err());assert_eq!(old,before);
    let mut c=i1_energy_fixture(CardId::DefendBlue,0,None,vec![],Some(PotionId::EnergyPotion),true);
    for (source,target,amount)in [(Source::None,Some(Actor::Player),2),(Source::Actor(Actor::Player),Some(Actor::Player),2),(Source::Potion{slot:1,id:PotionId::EnergyPotion},Some(Actor::Player),2),(Source::Potion{slot:0,id:PotionId::WeakPotion},Some(Actor::Player),2),(Source::Potion{slot:0,id:PotionId::EnergyPotion},Some(Actor::Player),3),(Source::Potion{slot:0,id:PotionId::EnergyPotion},Some(Actor::Enemy(c.state.enemies[0].instance_id.clone())),2)]{
        let before=c.clone();assert!(c.enqueue_native_effects(vec![Effect{source,target,kind:EffectKind::GainEnergy(amount)}],None).is_err());assert_eq!(c,before,"forged work cannot mutate before rejection");
    }
    let before=c.clone();assert!(c.enqueue_native_effects(vec![Effect{source:Source::Potion{slot:0,id:PotionId::EnergyPotion},target:Some(Actor::Player),kind:EffectKind::GainEnergy(2)}],None).is_err());assert_eq!(c,before,"an occupied lawful potion source still requires exact consumed-slot capture");
    let mut foreign=old.clone();let before=foreign.clone();assert!(foreign.enqueue_native_effects(vec![Effect{source:Source::Potion{slot:0,id:PotionId::EnergyPotion},target:Some(Actor::Player),kind:EffectKind::GainEnergy(2)}],None).is_err());assert_eq!(foreign,before);
}
