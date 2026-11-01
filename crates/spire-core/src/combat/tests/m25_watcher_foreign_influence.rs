//! M25 Foreign Influence closure on the shared generated-card and card-offer owners.
use super::super::*;
use crate::{
    combat::model::OwnedCard,
    content::{CardColor, CardId, CardRarity, CardType, Catalog, Character, EncounterId},
    items::CombatItemContext,
};
use std::collections::BTreeSet;

use CardId::*;

const SUPPORT: &[(CardColor, CardRarity, &[CardId])] = &[
    (CardColor::Purple, CardRarity::Common, &[BowlingBash, Consecrate, CrushJoints, CutThroughFate, EmptyFist, FlurryOfBlows, FlyingSleeves, FollowUp, JustLucky, SashWhip]),
    (CardColor::Purple, CardRarity::Uncommon, &[CarveReality, Conclude, FearNoEvil, ReachHeaven, SandsOfTime, SignatureMove, TalkToTheHand, Tantrum, Wallop, Weave, WheelKick, WindmillStrike]),
    (CardColor::Purple, CardRarity::Rare, &[Brilliance, Ragnarok]),
    (CardColor::Red, CardRarity::Common, &[Anger, BodySlam, Clash, Cleave, Clothesline, Headbutt, HeavyBlade, IronWave, PerfectedStrike, PommelStrike, SwordBoomerang, Thunderclap, TwinStrike, WildStrike]),
    (CardColor::Red, CardRarity::Uncommon, &[BloodForBlood, Carnage, Dropkick, Hemokinesis, Pummel, Rampage, RecklessCharge, SearingBlow, SeverSoul, Uppercut, Whirlwind]),
    (CardColor::Red, CardRarity::Rare, &[Bludgeon, FiendFire, Immolate]),
    (CardColor::Green, CardRarity::Common, &[Bane, DaggerSpray, DaggerThrow, FlyingKnee, PoisonedStab, QuickSlash, Slice, SneakyStrike, SuckerPunch]),
    (CardColor::Green, CardRarity::Uncommon, &[AllOutAttack, Backstab, Choke, Dash, EndlessAgony, Eviscerate, Finisher, Flechettes, HeelHook, MasterfulStab, Predator, RiddleWithHoles, Skewer]),
    (CardColor::Green, CardRarity::Rare, &[DieDieDie, GlassKnife, GrandFinale, Unload]),
    (CardColor::Blue, CardRarity::Common, &[BallLightning, Barrage, BeamCell, Claw, ColdSnap, CompileDriver, GoForTheEyes, Rebound, Streamline, SweepingBeam]),
    (CardColor::Blue, CardRarity::Uncommon, &[Blizzard, Bullseye, DoomAndGloom, FTL, Melter, RipAndTear, Scrape, Sunder]),
    (CardColor::Blue, CardRarity::Rare, &[AllForOne, CoreSurge, Hyperbeam, MeteorStrike, ThunderStrike]),
    (CardColor::Colorless, CardRarity::Uncommon, &[DramaticEntrance, FlashOfSteel, MindBlast, SwiftStrike]),
    (CardColor::Colorless, CardRarity::Rare, &[HandOfGreed]),
];

#[derive(Clone, Copy)]
struct PrimaryFace { id: CardId, base_damage: i32, upgraded_damage: i32, exhaust: bool }

/// Literal attack rows from research red/colourless §2–3, Silent §2–3 and
/// Defect §2–3. Damage totals below use the one-enemy fixture; multi-hit counts,
/// dynamic inputs, and the zero-orb Watcher cases are named in the test setup.
const PRIMARY: &[PrimaryFace] = &[
    PrimaryFace{id:Anger,base_damage:6,upgraded_damage:8,exhaust:false},
    PrimaryFace{id:BodySlam,base_damage:7,upgraded_damage:7,exhaust:false},
    PrimaryFace{id:Clash,base_damage:14,upgraded_damage:18,exhaust:false},
    PrimaryFace{id:Cleave,base_damage:8,upgraded_damage:11,exhaust:false},
    PrimaryFace{id:Clothesline,base_damage:12,upgraded_damage:14,exhaust:false},
    PrimaryFace{id:Headbutt,base_damage:9,upgraded_damage:12,exhaust:false},
    PrimaryFace{id:HeavyBlade,base_damage:14,upgraded_damage:14,exhaust:false},
    PrimaryFace{id:IronWave,base_damage:5,upgraded_damage:7,exhaust:false},
    PrimaryFace{id:PerfectedStrike,base_damage:8,upgraded_damage:9,exhaust:false},
    PrimaryFace{id:PommelStrike,base_damage:9,upgraded_damage:10,exhaust:false},
    PrimaryFace{id:SwordBoomerang,base_damage:9,upgraded_damage:12,exhaust:false},
    PrimaryFace{id:Thunderclap,base_damage:4,upgraded_damage:7,exhaust:false},
    PrimaryFace{id:TwinStrike,base_damage:10,upgraded_damage:14,exhaust:false},
    PrimaryFace{id:WildStrike,base_damage:12,upgraded_damage:17,exhaust:false},
    PrimaryFace{id:BloodForBlood,base_damage:18,upgraded_damage:22,exhaust:false},
    PrimaryFace{id:Carnage,base_damage:20,upgraded_damage:28,exhaust:false},
    PrimaryFace{id:Dropkick,base_damage:5,upgraded_damage:8,exhaust:false},
    PrimaryFace{id:Hemokinesis,base_damage:15,upgraded_damage:20,exhaust:false},
    PrimaryFace{id:Pummel,base_damage:8,upgraded_damage:10,exhaust:true},
    PrimaryFace{id:Rampage,base_damage:8,upgraded_damage:8,exhaust:false},
    PrimaryFace{id:RecklessCharge,base_damage:7,upgraded_damage:10,exhaust:false},
    // The accepted fresh-kind constructor carries no prior Searing Blow upgrade
    // history (M10); the actual Master Reality hand route below applies one real
    // upgrade and asserts the 16-damage face from the research row.
    PrimaryFace{id:SearingBlow,base_damage:12,upgraded_damage:12,exhaust:false},
    PrimaryFace{id:SeverSoul,base_damage:16,upgraded_damage:22,exhaust:false},
    PrimaryFace{id:Uppercut,base_damage:13,upgraded_damage:13,exhaust:false},
    PrimaryFace{id:Whirlwind,base_damage:45,upgraded_damage:72,exhaust:false},
    PrimaryFace{id:Bludgeon,base_damage:32,upgraded_damage:42,exhaust:false},
    PrimaryFace{id:FiendFire,base_damage:7,upgraded_damage:10,exhaust:true},
    PrimaryFace{id:Immolate,base_damage:21,upgraded_damage:28,exhaust:false},
    PrimaryFace{id:Bane,base_damage:7,upgraded_damage:10,exhaust:false},
    PrimaryFace{id:DaggerSpray,base_damage:8,upgraded_damage:12,exhaust:false},
    PrimaryFace{id:DaggerThrow,base_damage:9,upgraded_damage:12,exhaust:false},
    PrimaryFace{id:FlyingKnee,base_damage:8,upgraded_damage:11,exhaust:false},
    PrimaryFace{id:PoisonedStab,base_damage:6,upgraded_damage:8,exhaust:false},
    PrimaryFace{id:QuickSlash,base_damage:8,upgraded_damage:12,exhaust:false},
    PrimaryFace{id:Slice,base_damage:6,upgraded_damage:9,exhaust:false},
    PrimaryFace{id:SneakyStrike,base_damage:12,upgraded_damage:16,exhaust:false},
    PrimaryFace{id:SuckerPunch,base_damage:7,upgraded_damage:9,exhaust:false},
    PrimaryFace{id:AllOutAttack,base_damage:10,upgraded_damage:14,exhaust:false},
    PrimaryFace{id:Backstab,base_damage:11,upgraded_damage:15,exhaust:true},
    PrimaryFace{id:Choke,base_damage:12,upgraded_damage:12,exhaust:false},
    PrimaryFace{id:Dash,base_damage:10,upgraded_damage:13,exhaust:false},
    PrimaryFace{id:EndlessAgony,base_damage:4,upgraded_damage:6,exhaust:true},
    PrimaryFace{id:Eviscerate,base_damage:21,upgraded_damage:27,exhaust:false},
    PrimaryFace{id:Finisher,base_damage:6,upgraded_damage:8,exhaust:false},
    PrimaryFace{id:Flechettes,base_damage:8,upgraded_damage:12,exhaust:false},
    PrimaryFace{id:HeelHook,base_damage:5,upgraded_damage:8,exhaust:false},
    PrimaryFace{id:MasterfulStab,base_damage:12,upgraded_damage:16,exhaust:false},
    PrimaryFace{id:Predator,base_damage:15,upgraded_damage:20,exhaust:false},
    PrimaryFace{id:RiddleWithHoles,base_damage:15,upgraded_damage:20,exhaust:false},
    PrimaryFace{id:Skewer,base_damage:63,upgraded_damage:90,exhaust:false},
    PrimaryFace{id:DieDieDie,base_damage:13,upgraded_damage:17,exhaust:true},
    PrimaryFace{id:GlassKnife,base_damage:16,upgraded_damage:24,exhaust:false},
    PrimaryFace{id:GrandFinale,base_damage:50,upgraded_damage:60,exhaust:false},
    PrimaryFace{id:Unload,base_damage:14,upgraded_damage:18,exhaust:false},
    PrimaryFace{id:BallLightning,base_damage:7,upgraded_damage:10,exhaust:false},
    PrimaryFace{id:Barrage,base_damage:0,upgraded_damage:0,exhaust:false},
    PrimaryFace{id:BeamCell,base_damage:3,upgraded_damage:4,exhaust:false},
    PrimaryFace{id:Claw,base_damage:3,upgraded_damage:5,exhaust:false},
    PrimaryFace{id:ColdSnap,base_damage:6,upgraded_damage:9,exhaust:false},
    PrimaryFace{id:CompileDriver,base_damage:7,upgraded_damage:10,exhaust:false},
    PrimaryFace{id:GoForTheEyes,base_damage:3,upgraded_damage:4,exhaust:false},
    PrimaryFace{id:Rebound,base_damage:9,upgraded_damage:12,exhaust:false},
    PrimaryFace{id:Streamline,base_damage:15,upgraded_damage:20,exhaust:false},
    PrimaryFace{id:SweepingBeam,base_damage:6,upgraded_damage:9,exhaust:false},
    PrimaryFace{id:Blizzard,base_damage:0,upgraded_damage:0,exhaust:false},
    PrimaryFace{id:Bullseye,base_damage:8,upgraded_damage:11,exhaust:false},
    PrimaryFace{id:DoomAndGloom,base_damage:10,upgraded_damage:14,exhaust:false},
    PrimaryFace{id:FTL,base_damage:5,upgraded_damage:6,exhaust:false},
    PrimaryFace{id:Melter,base_damage:10,upgraded_damage:14,exhaust:false},
    PrimaryFace{id:RipAndTear,base_damage:14,upgraded_damage:18,exhaust:false},
    PrimaryFace{id:Scrape,base_damage:7,upgraded_damage:10,exhaust:false},
    PrimaryFace{id:Sunder,base_damage:24,upgraded_damage:32,exhaust:false},
    PrimaryFace{id:AllForOne,base_damage:10,upgraded_damage:14,exhaust:false},
    PrimaryFace{id:CoreSurge,base_damage:11,upgraded_damage:15,exhaust:true},
    PrimaryFace{id:Hyperbeam,base_damage:26,upgraded_damage:34,exhaust:false},
    PrimaryFace{id:MeteorStrike,base_damage:24,upgraded_damage:30,exhaust:false},
    PrimaryFace{id:ThunderStrike,base_damage:0,upgraded_damage:0,exhaust:false},
    PrimaryFace{id:DramaticEntrance,base_damage:8,upgraded_damage:12,exhaust:true},
    PrimaryFace{id:FlashOfSteel,base_damage:3,upgraded_damage:6,exhaust:false},
    PrimaryFace{id:MindBlast,base_damage:4,upgraded_damage:4,exhaust:false},
    PrimaryFace{id:SwiftStrike,base_damage:7,upgraded_damage:10,exhaust:false},
    PrimaryFace{id:HandOfGreed,base_damage:20,upgraded_damage:25,exhaust:false},
];

fn watcher(deck: &[(CardId, u32)]) -> Combat {
    let mut entry = super::m8_shapes::entry(EncounterId::OrbWalker);
    entry.setup.character = Character::Watcher;
    entry.cards = deck.iter().enumerate()
        .map(|(index, (id, upgrades))| m25_watcher_live_card(*id, CardInstanceId(index as u64 + 1), *upgrades).unwrap())
        .collect();
    entry.opening_draw = deck.len().min(10);
    entry.setup.player.energy = 9;
    entry.setup.player.max_energy = 9;
    let mut items = Some(CombatItemContext::new(vec![], vec![None; 3]).with_m25_profile());
    let combat = Combat::enter_with_item_context(entry, &mut items).unwrap();
    assert!(items.is_none());
    combat
}

fn m25_output(combat: &Combat, id: CardId, upgrades: u32) -> crate::combat::card_state::LiveCard {
    let template = super::super::cards::watcher::foreign_output_template(&combat.state, id, upgrades).unwrap();
    template.stat_equivalent_copy(CardInstanceId(1))
}

fn watcher_filler(combat: &mut Combat, instance: u64) -> OwnedCard {
    let card = m25_watcher_live_card(DefendPurple, CardInstanceId(instance), 0).unwrap();
    combat.state.card_ids_issued.insert(card_state::handle(card.instance));
    OwnedCard::Full(card)
}

fn reference_proposals(stream: &crate::combat::rng::StreamState, support: &[CardId])
    -> (Vec<CardId>, crate::combat::rng::StreamState, usize)
{
    let mut tiers: [Vec<CardId>; 3] = [Vec::new(), Vec::new(), Vec::new()];
    for id in support {
        let row = Catalog::standard().card(*id).unwrap();
        let tier = match row.rarity { CardRarity::Common => 0, CardRarity::Uncommon => 1, CardRarity::Rare => 2, _ => panic!("ordinary rarity") };
        tiers[tier].push(*id);
    }
    let mut cursor = stream.clone();
    let mut cards = Vec::new();
    let mut proposals = 0;
    while cards.len() < 3 {
        proposals += 1;
        let (rarity, next) = crate::combat::rng::uniform_below(&cursor, 100).unwrap();
        cursor = next;
        let tier = match rarity { 0..=54 => 0, 55..=84 => 1, 85..=99 => 2, _ => unreachable!() };
        let (index, next) = crate::combat::rng::uniform_below(&cursor, tiers[tier].len() as u128).unwrap();
        cursor = next;
        let id = tiers[tier][index as usize];
        if !cards.contains(&id) { cards.push(id); }
    }
    (cards, cursor, proposals)
}

#[test]
fn m25_fi_support_and_proposal_law() {
    let support = crate::content::watcher::foreign_influence_support();
    let expected = SUPPORT.iter().flat_map(|(_, _, ids)| ids.iter().copied()).collect::<BTreeSet<_>>();
    assert_eq!(support.len(), 106);
    assert_eq!(support.iter().copied().collect::<BTreeSet<_>>(), expected);
    let mut rarity_counts = [0usize; 3];
    for (color, rarity, ids) in SUPPORT {
        rarity_counts[match rarity { CardRarity::Common => 0, CardRarity::Uncommon => 1, CardRarity::Rare => 2, _ => panic!("unexpected support rarity") }] += ids.len();
        for id in *ids {
            let row = Catalog::standard().card(*id).expect("support catalog row");
            assert_eq!((row.color, row.rarity, row.card_type), (*color, *rarity, CardType::Attack), "{id:?}");
            assert!(!crate::content::is_healing(*id), "{id:?}");
        }
    }
    assert_eq!(rarity_counts, [43, 48, 15]);
    assert_eq!(support.iter().filter(|id| Catalog::standard().card(**id).is_some_and(|row|row.color != CardColor::Purple)).count(), 82);
    assert!([Feed, Reaper, LessonLearned].iter().all(|id| !support.contains(id)));
    assert!([Feed, Reaper, LessonLearned].iter().all(|id| crate::content::is_healing(*id)));
    for (roll, tier) in [(0, 0), (54, 0), (55, 1), (84, 1), (85, 2), (99, 2)] {
        assert_eq!(super::super::cards::watcher::foreign_influence_rarity_tier(roll).unwrap(), tier, "roll {roll}");
    }

    let stream_id = crate::combat::rng::StreamId::new(crate::combat::rng::Domain::RunCombatCardRandom,
        vec!["m25-fi-script".into()]).unwrap();
    let scripted = (0u8..=255).find_map(|key| {
        let stream = crate::combat::rng::StreamState::new(stream_id.clone(), [key; 32], 0).unwrap();
        let (expected, next, proposals) = reference_proposals(&stream, support);
        (proposals > 3).then_some((stream, expected, next, proposals))
    }).expect("fixed scripted keys include a duplicate proposal");
    let (stream, expected, expected_next, proposal_count) = scripted;
    assert!(proposal_count > 3, "the script exercises a whole-proposal retry");
    let (actual, actual_next) = super::super::cards::watcher::foreign_influence_proposals(&stream, support).unwrap();
    assert_eq!(actual, expected, "rarity is rolled again when a duplicate kind is retried");
    assert_eq!(actual_next, expected_next, "RNG advances over all rarity and kind draws, including retry");
    assert_eq!(actual.iter().collect::<BTreeSet<_>>().len(), 3);

    let mut incomplete = support.to_vec();
    incomplete.retain(|id| Catalog::standard().card(*id).is_some_and(|row|row.rarity != CardRarity::Rare));
    assert!(super::super::cards::watcher::foreign_influence_proposals(&stream, &incomplete).is_err(),
        "a missing rarity tier rejects before proposal RNG");
    let rejected = watcher(&[(ForeignInfluence, 0)]);
    let card = rejected.state.piles.hand[0].full().unwrap().clone();
    let before_energy = rejected.state.player.energy;
    let before_rng = rejected.state.card_rng.clone();
    assert!(super::super::cards::watcher::validate_foreign_influence_payload(&rejected.state, &card, None, &incomplete).is_err());
    assert_eq!(rejected.state.player.energy, before_energy, "incomplete support rejects before payment");
    assert_eq!(rejected.state.card_rng, before_rng, "incomplete support rejects before RNG");

    let mut combat = watcher(&[(ForeignInfluence, 0)]);
    let action = CombatAction::PlayCard { instance: 1, target: None };
    assert!(combat.legal_actions_full().unwrap().contains(&action), "Foreign Influence is playable under M25");
    let before_rng = combat.state.card_rng.clone();
    combat.step_full(&action).unwrap();
    let offer = combat.public_full().unwrap().card_offer.expect("Foreign Influence opens its optional offer");
    assert!(offer.optional);
    assert_eq!(offer.cards.len(), 3);
    assert_ne!(combat.state.card_rng, before_rng, "three proposals sample the combat card-random stream");
    assert_eq!(offer.cards.iter().collect::<BTreeSet<_>>().len(), 3);
    assert!(offer.cards.iter().all(|id|support.contains(id)));
}

#[test]
fn m25_fi_choice_and_delivery() {
    let set_searing_offer = |combat: &mut Combat| {
        let pending = combat.state.silent.card_offer.as_mut().expect("pending Foreign Influence choice");
        pending.cards = vec![SearingBlow, Anger, BodySlam];
    };

    let mut base = watcher(&[(ForeignInfluence, 0)]);
    let energy = base.state.player.energy;
    base.step_full(&CombatAction::PlayCard { instance: 1, target: None }).unwrap();
    let offer = base.public_full().unwrap().card_offer.expect("Foreign Influence offers three cards");
    assert_eq!(offer.cards.len(), 3);
    assert!(offer.optional);
    assert_eq!(base.legal_actions_full().unwrap(), vec![
        CombatAction::ChooseCardOffer { index: 0 },
        CombatAction::ChooseCardOffer { index: 1 },
        CombatAction::ChooseCardOffer { index: 2 },
        CombatAction::SkipCardOffer,
    ]);
    set_searing_offer(&mut base);
    base.step_full(&CombatAction::ChooseCardOffer { index: 0 }).unwrap();
    assert_eq!(base.state.player.energy, energy, "base Foreign Influence costs zero and offers a normally-priced copy");
    assert!(base.state.piles.exhaust.iter().any(|owned| owned.full().is_some_and(|card|card.id == ForeignInfluence)));
    let output = base.state.piles.hand.iter().filter_map(OwnedCard::full)
        .find(|card| card.id == SearingBlow).expect("selected fresh stat-equivalent copy enters hand");
    assert_eq!((output.upgrades, output.local.base_damage, output.local.turn_cost), (0, Some(12), None));
    assert!(matches!(output.origin, CombatCardRef::CombatLocal(_)));
    assert_ne!(output.instance.0, 1);

    let master_reality = FullPowerEntry { id: FullPowerId("master_reality"),
        state: PowerState { amount: -1, skip_first: false, monster_source_grace: false } };
    let mut upgraded = watcher(&[(ForeignInfluence, 1)]);
    upgraded.state.player_full_powers.push(master_reality.clone());
    upgraded.step_full(&CombatAction::PlayCard { instance: 1, target: None }).unwrap();
    set_searing_offer(&mut upgraded);
    upgraded.step_full(&CombatAction::ChooseCardOffer { index: 0 }).unwrap();
    let output = upgraded.state.piles.hand.iter().filter_map(OwnedCard::full)
        .find(|card|card.id==SearingBlow).expect("upgraded Foreign Influence output enters hand");
    assert_eq!((output.upgrades, output.local.base_damage, output.local.turn_cost), (1, Some(16), Some(0)),
        "Master Reality upgrades once and upgraded Foreign Influence assigns turn cost zero");
    assert!(upgraded.state.piles.exhaust.iter().any(|owned|owned.full().is_some_and(|card|card.id==ForeignInfluence)));

    let mut overflow = watcher(&[(ForeignInfluence, 0)]);
    overflow.state.player_full_powers.push(master_reality);
    overflow.step_full(&CombatAction::PlayCard { instance: 1, target: None }).unwrap();
    set_searing_offer(&mut overflow);
    for id in 100..110 {
        let card = m25_watcher_live_card(DefendPurple, CardInstanceId(id), 0).unwrap();
        overflow.state.card_ids_issued.insert(card_state::handle(card.instance));
        overflow.state.piles.hand.push(OwnedCard::Full(card));
    }
    assert_eq!(overflow.state.piles.hand.len(), 10);
    overflow.step_full(&CombatAction::ChooseCardOffer { index: 0 }).unwrap();
    assert_eq!(overflow.state.piles.hand.len(), 10);
    let overflow_copy = overflow.state.piles.discard.iter().filter_map(OwnedCard::full)
        .find(|card|card.id==SearingBlow).expect("full hand routes Foreign Influence output to discard");
    assert_eq!((overflow_copy.upgrades, overflow_copy.local.base_damage), (0, Some(12)),
        "Master Reality does not upgrade a discarded overflow copy");

    let mut skipped = watcher(&[(ForeignInfluence, 0)]);
    skipped.step_full(&CombatAction::PlayCard { instance: 1, target: None }).unwrap();
    skipped.step_full(&CombatAction::SkipCardOffer).unwrap();
    assert!(skipped.state.piles.exhaust.iter().any(|owned|owned.full().is_some_and(|card|card.id==ForeignInfluence)));
    assert!(skipped.state.piles.hand.iter().filter_map(OwnedCard::full).all(|card|card.id!=ForeignInfluence));
}

#[test]
fn m25_fi_every_output_playable_by_watcher() {
    let outputs = crate::content::watcher::foreign_influence_support().iter()
        .copied().filter(|id| Catalog::standard().card(*id).is_some_and(|row|row.color != CardColor::Purple))
        .collect::<Vec<_>>();
    assert_eq!(outputs.len(), 82);
    assert_eq!(PRIMARY.len(), 82);
    assert_eq!(outputs.iter().copied().collect::<BTreeSet<_>>(), PRIMARY.iter().map(|face|face.id).collect());
    let mut off_colour_permanent = card_state::permanent(SearingBlow, CardInstanceId(90), 0);
    assert!(super::super::cards::watcher::initialize_m25_profile(&mut off_colour_permanent).is_err(),
        "an off-colour permanent remains outside M25 Watcher admission");

    for face in PRIMARY {
        for (upgrades, expected_damage) in [(0, face.base_damage), (1, face.upgraded_damage)] {
            let mut combat = watcher(&[(DefendPurple, 0)]);
            combat.state.enemies[0].hp = 500;
            combat.state.enemies[0].max_hp = 500;
            combat.state.player.block = 7;
            match face.id {
                MindBlast => for instance in 100..104 {
                    let card = watcher_filler(&mut combat, instance);
                    combat.state.piles.draw.push(card);
                },
                Flechettes => for instance in 100..102 {
                    let card = watcher_filler(&mut combat, instance);
                    combat.state.piles.hand.push(card);
                },
                FiendFire => {
                    let filler = watcher_filler(&mut combat, 100);
                    combat.state.piles.hand.push(filler);
                },
                Finisher => combat.state.silent.played_this_turn.push(Strike),
                _ => (),
            }
            let card = m25_output(&combat, face.id, upgrades);
            combat.state.piles.hand[0] = OwnedCard::Full(card);
            piles::refresh_hand_values(&mut combat.state);
            let legal = combat.legal_actions_full().unwrap();
            let action = legal.into_iter().find(|action|matches!(action, CombatAction::PlayCard{instance:1,..}))
                .unwrap_or_else(||panic!("{:?}+{upgrades} has no legal Watcher play", face.id));
            let before_hp = combat.state.enemies[0].hp;
            combat.step_full(&action).unwrap_or_else(|error|panic!("{:?}+{upgrades} failed to play: {error:?}", face.id));
            assert!(combat.fault_record().is_none(), "{:?}+{upgrades} resolver fault: {:?}", face.id, combat.fault_record());
            assert_eq!(before_hp-combat.state.enemies[0].hp, expected_damage, "{:?}+{upgrades} research primary value", face.id);
            assert_eq!(combat.state.phase, Phase::Decision, "{:?}+{upgrades} completed its action", face.id);
            assert!(combat.state.in_play.is_none(), "{:?}+{upgrades} left no card in limbo", face.id);
            assert_eq!(combat.state.piles.exhaust.iter().any(|owned|owned.handle().as_str()=="1"), face.exhaust,
                "{:?}+{upgrades} finish destination", face.id);
            assert_eq!(combat.state.piles.discard.iter().any(|owned|owned.handle().as_str()=="1"), !face.exhaust,
                "{:?}+{upgrades} finish destination", face.id);
            if face.id==Hyperbeam { assert_eq!(combat.state.character.focus,-3,"Watcher has zero orbs but still loses Focus"); }
            if matches!(face.id,Barrage|Blizzard|ThunderStrike) {
                assert_eq!(before_hp-combat.state.enemies[0].hp,0,"zero orb/Lightning channels yield zero hits");
            }
        }
    }

    let mut searing = watcher(&[(ForeignInfluence, 0)]);
    searing.state.enemies[0].hp = 500;
    searing.state.enemies[0].max_hp = 500;
    searing.state.player_full_powers.push(FullPowerEntry { id: FullPowerId("master_reality"),
        state: PowerState { amount: -1, skip_first: false, monster_source_grace: false } });
    searing.step_full(&CombatAction::PlayCard { instance: 1, target: None }).unwrap();
    searing.state.silent.card_offer.as_mut().unwrap().cards = vec![SearingBlow, Anger, BodySlam];
    searing.step_full(&CombatAction::ChooseCardOffer { index: 0 }).unwrap();
    let output = searing.state.piles.hand.iter().filter_map(OwnedCard::full).find(|card|card.id==SearingBlow).unwrap().clone();
    assert_eq!((output.upgrades, output.local.base_damage), (1, Some(16)), "Master Reality upgrades the hand copy exactly once");
    searing.step_full(&CombatAction::PlayCard { instance: output.instance.0, target: Some(TargetId(1)) }).unwrap();
    assert_eq!(500-searing.state.enemies[0].hp, 16, "Searing Blow plays through the Ironclad owner at its once-upgraded value");
    let finished = searing.state.piles.discard.iter().filter_map(OwnedCard::full)
        .find(|card|card.instance==output.instance).expect("Searing Blow finishes in discard");
    assert_eq!((finished.upgrades, finished.local.base_damage), (1, Some(16)));
}

#[test]
fn m25_fi_perfected_strike_counts_the_source_strike_set() {
    for (character, source_character) in [
        (Character::Watcher, Some(Character::Watcher)),
        (Character::Ironclad, None),
    ] {
        let mut combat = super::full_runtime::entered_combat(3, 3, |_| {});
        combat.state.character = super::super::characters::CharacterState::for_character(character);
        combat.state.silent.source_character = source_character;
        let fixture = [
            (PerfectedStrike, 1),
            (Thunderclap, 2),
            (PommelStrike, 3),
            (TwinStrike, 4),
            (Strike, 5),
        ].map(|(id, instance)| {
            OwnedCard::Full(card_state::permanent(id, CardInstanceId(instance), 0))
        });
        combat.state.piles.hand = vec![fixture[0].clone(), fixture[1].clone()];
        combat.state.piles.draw = vec![fixture[2].clone()];
        combat.state.piles.discard = vec![fixture[3].clone()];
        combat.state.piles.exhaust = vec![fixture[4].clone()];
        combat.state.card_ids_issued = (1..=5).map(|id|card_state::handle(CardInstanceId(id))).collect();
        let action = CombatAction::PlayCard { instance: 1, target: Some(TargetId(1)) };
        let prepared = cards::prepare(&combat, &action).unwrap();
        let expected = 12;
        assert!(matches!(&prepared.effects[0].kind, EffectKind::Damage(DamagePayload::Fixed(value)) if *value == expected),
            "{character:?} Perfected Strike counts 3 source STRIKE kinds outside exhaust");
        combat.step_full(&action).unwrap();
        assert_eq!(combat.state.enemies[0].hp, 48 - expected, "{character:?}");
    }
}
