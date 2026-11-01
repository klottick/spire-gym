//! M26 Task M3 Run mutations, transform pools, persistence, and diagnostics.
//! Literals are sourced from the M26 design, Task M3 plan, and seam map cited below.
use super::super::{diagnostics, mutations};
use super::super::*;
use crate::content::{self, CardId, CardRarity, CardType, Catalog, Character, EncounterId, NoteStorage};
use crate::run::CardInstance;
use std::collections::{BTreeMap, BTreeSet};

use CardId::*;

fn card(instance: u64, id: CardId, upgrades: u32, misc: i32) -> CardInstance {
    CardInstance { instance, id, upgrades, misc }
}

fn supplied(deck: Vec<CardInstance>) -> AuthoredRunStart {
    AuthoredRunStart {
        hp: 80,
        max_hp: 80,
        gold: 99,
        deck,
        relics: vec![],
        potions: vec![None; 3],
        relic_counters: Default::default(),
    }
}

fn complete_run(character: Character, deck: Vec<CardInstance>) -> Run {
    diagnostics::complete_run(character, supplied(deck))
        .unwrap_or_else(|error| panic!("complete {character:?} Run: {error:?}"))
}

fn deck_card(run: &Run, instance: u64) -> &CardInstance {
    run.state.deck.iter().find(|card| card.instance == instance)
        .unwrap_or_else(|| panic!("permanent card instance {instance} is absent"))
}

fn state_bytes(run: &Run) -> String {
    format!("{:?}", run.state)
}

#[test]
fn m26_mutation_obtain_upgrade_remove() {
    // Design §4.4 and §5.1: content::complete enumerates the 352 executable permanent kinds.
    let kinds = Catalog::standard().cards().iter()
        .filter(|definition| content::complete::supports_permanent(definition.id))
        .map(|definition| definition.id)
        .collect::<Vec<_>>();
    assert_eq!(kinds.len(), 352, "complete permanent union, design §4.4");

    for character in [Character::Ironclad, Character::Silent, Character::Defect, Character::Watcher] {
        let mut run = complete_run(character, vec![card(1, Strike, 0, 0)]);
        let mut removed_count = 0;
        let mut nonremovable = BTreeSet::new();
        for id in kinds.iter().copied() {
            let before_len = run.state.deck.len();
            let expected_misc = match id {
                GeneticAlgorithm => 1,
                RitualDagger => 15,
                _ => 0,
            };
            let instance = mutations::obtain_card(&mut run.state, id, 0)
                .unwrap_or_else(|error| panic!("obtain {id:?}: {error:?}"));
            let obtained = deck_card(&run, instance).clone();
            assert_eq!(obtained, card(instance, id, 0, expected_misc), "obtain {id:?}; design §8.1");
            assert_eq!(run.state.deck.len(), before_len + 1, "obtain grows deck once for {id:?}");

            // Catalog type is the independent upgradeability expectation; do not derive it from
            // the can_upgrade function under test (design §8.1–8.2 and catalog rows).
            let definition = Catalog::standard().card(id).expect("enumerated catalog row");
            let catalog_says_upgradeable = matches!(definition.card_type,
                CardType::Attack | CardType::Skill | CardType::Power);
            let at = run.state.deck.iter().position(|owned| owned.instance == instance).unwrap();
            assert_eq!(mutations::can_upgrade(&run.state, &run.state.deck[at]), catalog_says_upgradeable,
                "catalog upgradeability for {id:?}");
            if catalog_says_upgradeable {
                mutations::upgrade_card(&mut run.state, instance)
                    .unwrap_or_else(|error| panic!("upgrade {id:?}: {error:?}"));
                assert_eq!(deck_card(&run, instance).upgrades, 1, "one shared upgrade for {id:?}");
                assert_eq!(deck_card(&run, instance).misc, expected_misc, "upgrade preserves misc for {id:?}");
            } else {
                let before = state_bytes(&run);
                assert!(mutations::upgrade_card(&mut run.state, instance).is_err(), "{id:?} does not upgrade");
                assert_eq!(state_bytes(&run), before, "failed upgrade of {id:?} is atomic");
            }

            if id == SearingBlow {
                let extra = mutations::obtain_card(&mut run.state, id, 3).unwrap();
                mutations::upgrade_card(&mut run.state, extra).unwrap();
                mutations::upgrade_card(&mut run.state, extra).unwrap();
                assert_eq!(deck_card(&run, extra).upgrades, 5,
                    "Searing Blow+3 upgrades twice more without a new bound; seam map §5");
                mutations::remove_card(&mut run.state, extra).unwrap();
            }

            let catalog_says_removable = definition.note_storage == NoteStorage::RemovablePermanent;
            if catalog_says_removable {
                let before_max_hp = run.state.max_hp;
                let before_len = run.state.deck.len();
                let removed = mutations::remove_card(&mut run.state, instance)
                    .unwrap_or_else(|error| panic!("remove {id:?}: {error:?}"));
                assert_eq!(removed, deck_card_from_value(&obtained, catalog_says_upgradeable));
                assert_eq!(run.state.deck.len(), before_len - 1, "remove shrinks deck once for {id:?}");
                removed_count += 1;
                if id == Parasite {
                    assert_eq!(before_max_hp - run.state.max_hp, 3, "Parasite removal loses 3 max HP; design §4.3, §8.2");
                } else {
                    assert_eq!(run.state.max_hp, before_max_hp, "non-Parasite removal keeps max HP");
                }
            } else {
                nonremovable.insert(id);
                let before = state_bytes(&run);
                assert!(mutations::remove_card(&mut run.state, instance).is_err(), "nonremovable {id:?}");
                assert_eq!(state_bytes(&run), before, "failed removal of {id:?} is atomic");
            }
        }
        assert_eq!(removed_count, 349, "design §7.2 NoteForYourself / §8.2");
        assert_eq!(nonremovable, BTreeSet::from([CurseOfTheBell, Necronomicurse, AscendersBane]),
            "the three nonremovable curses; design §4.3");
    }
}

fn deck_card_from_value(card: &CardInstance, upgraded: bool) -> CardInstance {
    CardInstance { upgrades: u32::from(upgraded), ..card.clone() }
}

#[test]
fn m26_mutation_rejections_are_atomic() {
    // Review Focus 1: compare byte-for-byte RunState Debug, including deck, HP, max HP and RNG.
    for id in [CurseOfTheBell, Necronomicurse, AscendersBane] {
        for transform in [false, true] {
            let mut run = complete_run(Character::Ironclad, vec![card(1, id, 0, 0)]);
            let before = state_bytes(&run);
            let result = if transform {
                mutations::transform_card(&mut run.state, 1, false, &[41, 1]).map(|_| ())
            } else {
                mutations::remove_card(&mut run.state, 1).map(|_| ())
            };
            assert!(result.is_err(), "{id:?} {transform:?} rejection");
            assert_eq!(state_bytes(&run), before, "{id:?} rejection leaves the whole Run unchanged");
        }
    }

    let mut upgraded_curse = complete_run(Character::Ironclad, vec![card(1, Strike, 0, 0)]);
    let before = state_bytes(&upgraded_curse);
    assert!(mutations::obtain_card(&mut upgraded_curse.state, Regret, 1).is_err(),
        "Regret+1 is rejected; design §8.1");
    assert_eq!(state_bytes(&upgraded_curse), before, "upgraded curse rejection is atomic");

    let mut second_upgrade = complete_run(Character::Ironclad, vec![card(1, Strike, 0, 0)]);
    mutations::upgrade_card(&mut second_upgrade.state, 1).unwrap();
    let before = state_bytes(&second_upgrade);
    assert!(mutations::upgrade_card(&mut second_upgrade.state, 1).is_err(), "ordinary card cannot upgrade twice");
    assert_eq!(state_bytes(&second_upgrade), before, "second ordinary upgrade rejection is atomic");

    let mut full = complete_run(Character::Ironclad, vec![card(1, Strike, 0, 0)]);
    while full.state.deck.len() < 999 {
        mutations::obtain_card(&mut full.state, Strike, 0).unwrap();
    }
    assert_eq!(full.state.deck.len(), 999, "Run deck capacity from design §8.1");
    let before = state_bytes(&full);
    assert!(mutations::obtain_card(&mut full.state, Strike, 0).is_err(), "obtain at 999 cards rejects");
    assert_eq!(state_bytes(&full), before, "deck-cap rejection is atomic");
}

fn transform_support(character: Character, input: CardId, expected_size: usize) -> Vec<CardId> {
    let support = content::pools::cards(content::pools::CardPool::Transform { character, input })
        .unwrap_or_else(|error| panic!("Transform({character:?}, {input:?}): {error:?}"));
    assert_eq!(support.len(), expected_size,
        "literal Transform support size from design §7.2 for {character:?}/{input:?}");
    support
}

fn transform_once(character: Character, input: CardId, auto_upgrade: bool, key: &[u64]) -> CardInstance {
    let mut run = complete_run(character, vec![card(1, input, 0, 0)]);
    let instance = mutations::transform_card(&mut run.state, 1, auto_upgrade, key)
        .unwrap_or_else(|error| panic!("transform {input:?}: {error:?}"));
    deck_card(&run, instance).clone()
}

#[test]
fn m26_mutation_transform() {
    // Design §7.2: Curse 10−1=9; ordinary colourless 35−1=34; ordinary
    // Ironclad 72−1=71; other character 71−1=70. Strike is Basic and is not
    // a member of Ordinary, so Ironclad keeps all 72. These are hand-derived sizes.
    let curse = transform_support(Character::Ironclad, Regret, 9);
    let colourless = transform_support(Character::Ironclad, FlashOfSteel, 34);
    let ironclad = transform_support(Character::Ironclad, Anger, 71);
    let silent = transform_support(Character::Silent, DaggerSpray, 70);
    let strike_basic = transform_support(Character::Ironclad, Strike, 72);
    let off_colour = transform_support(Character::Silent, Inflame, 71);

    for (input, support) in [
        (Regret, curse.as_slice()),
        (FlashOfSteel, colourless.as_slice()),
        (Anger, ironclad.as_slice()),
        (DaggerSpray, silent.as_slice()),
        (Strike, strike_basic.as_slice()),
        (Inflame, off_colour.as_slice()),
    ] {
        let character = if input == DaggerSpray || input == Inflame { Character::Silent } else { Character::Ironclad };
        let output = transform_once(character, input, false, &[26, 1]);
        assert!(support.contains(&output.id), "{input:?} transforms inside its sized support");
        assert_eq!(output.upgrades, 0, "transform output is fresh for {input:?}");
        assert_eq!(output.misc, 0, "transform output has base misc for {input:?}");
    }

    let upgraded = transform_once(Character::Ironclad, Anger, true, &[26, 2]);
    let definition = Catalog::standard().card(upgraded.id).unwrap();
    assert!(matches!(definition.card_type, CardType::Attack | CardType::Skill | CardType::Power));
    assert_eq!(upgraded.upgrades, 1, "auto_upgrade upgrades an upgradeable output once; design §8.3");

    // Every one of the literal 71 Ironclad support members appears over 2,000
    // transforms of the same Anger input under distinct semantic keys.
    let mut run = complete_run(Character::Ironclad, vec![card(1, Strike, 0, 0)]);
    let mut seen = BTreeMap::<CardId, usize>::new();
    for key in 0..2_000u64 {
        let input = mutations::obtain_card(&mut run.state, Anger, 0).unwrap();
        let output = mutations::transform_card(&mut run.state, input, false, &[26, key])
            .unwrap_or_else(|error| panic!("Anger transform key {key}: {error:?}"));
        let generated = deck_card(&run, output);
        assert!(ironclad.contains(&generated.id), "Anger transform key {key} support");
        assert_eq!(generated.upgrades, 0, "fresh transform output key {key}");
        assert_eq!(generated.misc, 0, "ordinary output base misc key {key}");
        *seen.entry(generated.id).or_default() += 1;
        mutations::remove_card(&mut run.state, output).unwrap();
    }
    for id in &ironclad {
        assert!(seen.get(id).copied().unwrap_or(0) > 0, "support member {id:?} appears");
    }
    assert_eq!(seen.len(), 71, "no output outside the 71-member support");

    let mut parasite = complete_run(Character::Ironclad, vec![card(1, Parasite, 0, 0)]);
    let before_max_hp = parasite.state.max_hp;
    let parasite_instance = mutations::transform_card(&mut parasite.state, 1, false, &[26, 3]).unwrap();
    assert_eq!(before_max_hp - parasite.state.max_hp, 3, "Parasite transform loses 3 max HP; design §4.3, §8.3");
    assert_ne!(deck_card(&parasite, parasite_instance).id, Parasite);

    let transformed_red = transform_once(Character::Silent, Inflame, false, &[26, 4]);
    assert_eq!(Catalog::standard().card(transformed_red.id).unwrap().color, content::CardColor::Green,
        "off-colour red input transforms into the player's green pool");
}

#[test]
fn m26_mutation_duplicate_and_note() {
    // Design §8.4, §8.5; F5 keeps Searing Blow's full u32 count and F6 copies exact misc.
    let rows = [
        card(7, SearingBlow, 4, 0),
        card(7, RitualDagger, 0, 21),
        card(7, GeneticAlgorithm, 0, 4),
    ];
    for source in rows {
        let mut run = complete_run(Character::Ironclad, vec![card(1, Strike, 0, 0)]);
        // The M26 diagnostics start validates obtainable misc, so these historical deck values
        // are installed directly in the crate-internal fixture as allowed by the contract.
        run.state.deck = vec![source.clone()];
        run.state.next_instance = source.instance + 1;
        let copy_instance = mutations::duplicate_card(&mut run.state, source.instance)
            .unwrap_or_else(|error| panic!("duplicate {:?}: {error:?}", source.id));
        let copy = deck_card(&run, copy_instance);
        assert_ne!(copy.instance, source.instance, "duplicate has a fresh physical instance");
        assert_eq!((copy.id, copy.upgrades, copy.misc), (source.id, source.upgrades, source.misc),
            "duplicate preserves id/upgrades/misc for {:?}", source.id);
    }

    let note_pool = content::pools::cards(content::pools::CardPool::NoteForYourself).unwrap();
    assert_eq!(note_pool.len(), 349, "design §7.2 NoteForYourself support");
    let mut run = complete_run(Character::Ironclad, vec![card(1, Strike, 0, 0)]);
    let searing = mutations::obtain_stored_card(&mut run.state, SearingBlow, 4).unwrap();
    assert_eq!(deck_card(&run, searing), &card(searing, SearingBlow, 4, 0), "stored Searing Blow preserves +4 and base misc");
    let dagger = mutations::obtain_stored_card(&mut run.state, RitualDagger, 1).unwrap();
    assert_eq!(deck_card(&run, dagger), &card(dagger, RitualDagger, 1, 15), "stored Ritual Dagger starts at base misc 15");

    for id in [Miracle, Wound] {
        let before = state_bytes(&run);
        assert!(mutations::obtain_stored_card(&mut run.state, id, 0).is_err(), "NoteForYourself rejects {id:?}");
        assert_eq!(state_bytes(&run), before, "failed note retrieval of {id:?} is atomic");
    }
}

#[test]
fn m26_mutation_pool_draws() {
    use CardRarity::Rare;
    // Purple Rare literals transcribed from the catalog rows in content/cards.rs.
    let purple_rares = [
        Brilliance, Devotion, Establishment, MasterReality, LessonLearned, Ragnarok,
        Alpha, Blasphemy, ConjureBlade, DeusExMachina, Judgment, Omniscience,
        Scrawl, SpiritShield, Vault, Wish, DevaForm,
    ];
    assert_eq!(purple_rares.len(), 17, "design §7.2 Purple Rare support");
    let expected = purple_rares.into_iter().collect::<BTreeSet<_>>();

    let mut run = complete_run(Character::Watcher, vec![card(1, StrikePurple, 0, 0)]);
    let before_rng = format!("{:?}", run.state.rng);
    let cards = mutations::draw_distinct(
        &mut run.state,
        content::pools::CardPool::Ordinary { character: Character::Watcher, rarity: Some(Rare), card_type: None },
        3,
        &[26, 5],
    ).unwrap_or_else(|error| panic!("purple Rare draw: {error:?}"));
    assert_eq!(cards.len(), 3);
    assert_eq!(cards.iter().copied().collect::<BTreeSet<_>>().len(), 3, "sample without replacement");
    assert!(cards.iter().all(|id| expected.contains(id)), "draws are in the literal 17 purple Rare catalog cards");
    assert_ne!(format!("{:?}", run.state.rng), before_rng, "successful draw commits staged RNG");

    for pool in [
        content::pools::CardPool::Colourless { rarity: Some(CardRarity::Common) },
        content::pools::CardPool::Ordinary { character: Character::Watcher, rarity: Some(CardRarity::Basic), card_type: None },
    ] {
        let mut run = complete_run(Character::Watcher, vec![card(1, StrikePurple, 0, 0)]);
        let before_rng = format!("{:?}", run.state.rng);
        assert!(mutations::draw_distinct(&mut run.state, pool, 1, &[26, 6]).is_err(),
            "incomplete pool {pool:?} rejects");
        assert_eq!(format!("{:?}", run.state.rng), before_rng, "failed pool draw commits no RNG");
    }

    // Design §4.3 / F3: flat random curse support has exactly these ten removable curses.
    let curses = [Clumsy, Decay, Doubt, Injury, Normality, Pain, Parasite, Regret, Shame, Writhe];
    let mut run = complete_run(Character::Ironclad, vec![card(1, Strike, 0, 0)]);
    let curse = mutations::draw_card(&mut run.state, content::pools::CardPool::Curse, &[26, 7]).unwrap();
    assert!(curses.contains(&curse), "random curse draw is one of the ten literal F3 members");
}

fn complete_combat(character: Character, encounter: EncounterId, deck: Vec<CardInstance>) -> Run {
    diagnostics::complete_combat(character, supplied(deck), encounter)
        .unwrap_or_else(|error| panic!("complete {character:?}/{encounter:?} combat: {error:?}"))
}

fn set_enemy_hp(run: &mut Run, target_index: usize, hp: i32) {
    let combat = run.state.combat.as_mut().expect("complete_combat enters combat");
    let view = combat.public_full().unwrap();
    let enemy = &view.enemies[target_index];
    assert!(enemy.hp > hp, "fixture lowers an enemy to the independently chosen Fatal threshold");
    combat.enqueue_native_effects(vec![crate::combat::Effect {
        source: crate::combat::Source::None,
        target: Some(crate::combat::Actor::Enemy(enemy.instance_id.clone())),
        kind: crate::combat::EffectKind::Damage(crate::combat::DamagePayload::HpLoss(enemy.hp - hp)),
    }], None).unwrap_or_else(|error| panic!("lower enemy to {hp} HP: {error:?}"));
    assert_eq!(combat.public_full().unwrap().enemies[target_index].hp, hp);
}

fn kill_with_ritual_dagger(run: &mut Run, target_index: usize) {
    run.choose_combat(crate::combat::CombatAction::PlayCard {
        instance: 1,
        target: Some(crate::combat::TargetId(target_index as u64 + 1)),
    }).unwrap_or_else(|error| panic!("Ritual Dagger Fatal at target {target_index}: {error:?}"));
}

fn dagger_view(run: &Run, instance: u64) -> crate::combat::LiveCardView {
    let combat = run.state.combat.as_ref().expect("Ritual Dagger combat");
    let view = combat.public_full().unwrap();
    view.exhaust.into_iter().chain(view.discard).chain(view.hand).chain(view.playing_cards)
        .find(|card| card.instance.0 == instance && card.id == RitualDagger)
        .unwrap_or_else(|| panic!("Ritual Dagger instance {instance} remains visible"))
}

#[test]
fn m26_ritual_dagger_growth_exclusions_and_persistence() {
    // Design §8.6: Ritual Dagger growth excludes minions, Awakened One's first phase,
    // and a non-final Darkling; all three encounters are in TRAINING_ENCOUNTERS_V2.
    for (encounter, target) in [
        (EncounterId::Reptomancer, 0usize),
        (EncounterId::AwakenedOne, 2usize),
        (EncounterId::ThreeDarklings, 0usize),
    ] {
        let mut run = complete_combat(Character::Ironclad, encounter, vec![card(1, RitualDagger, 0, 15)]);
        let view = run.state.combat.as_ref().unwrap().public_full().unwrap();
        if encounter == EncounterId::Reptomancer {
            assert_eq!(view.enemies[target].enemy_id, crate::content::EnemyId::Dagger,
                "Reptomancer's first actor is a Dagger minion");
        }
        if encounter == EncounterId::AwakenedOne {
            assert_eq!(view.enemies[target].enemy_id, crate::content::EnemyId::AwakenedOne,
                "target is Awakened One's first phase");
        }
        if encounter == EncounterId::ThreeDarklings {
            assert!(view.enemies.iter().skip(1).all(|enemy| enemy.hp > 0),
                "the targeted Darkling is not the final live Darkling");
        }
        set_enemy_hp(&mut run, target, 15);
        kill_with_ritual_dagger(&mut run, target);
        let combat = run.state.combat.as_ref().unwrap();
        assert!(combat.permanent_card_mutations().is_empty(), "excluded Fatal {encounter:?} has no growth journal");
        assert_eq!(dagger_view(&run, 1).misc, 15, "excluded Fatal {encounter:?} does not grow misc");
        assert!(combat.fault_record().is_none(), "excluded Fatal {encounter:?} is not a fault");
    }

    // Design §4.2/§8.6 and RitualDagger card data: base Fatal +3; upgraded Fatal +5.
    for (upgrades, growth) in [(0, 3), (1, 5)] {
        // Main fix: Three Cultists keeps the combat live after one Fatal (a sole-enemy kill
        // ends the combat and `choose_combat` settles it at once).
        let mut run = complete_combat(Character::Ironclad, EncounterId::ThreeCultists,
            vec![card(1, RitualDagger, upgrades, 15)]);
        set_enemy_hp(&mut run, 0, 15);
        kill_with_ritual_dagger(&mut run, 0);
        let combat = run.state.combat.as_ref().unwrap();
        assert_eq!(dagger_view(&run, 1).misc, 15 + growth, "played Ritual Dagger growth for +{upgrades}");
        let mutations = combat.permanent_card_mutations();
        assert_eq!(mutations.len(), 1, "one permanent journal entry");
        assert_eq!((mutations[0].instance.0, mutations[0].id, mutations[0].misc),
            (1, RitualDagger, 15 + growth), "Run-visible mutation is tied to the played instance");
    }

    // Real Dual Wield creates a stat-equivalent Ritual Dagger copy in another family.
    let mut dual_wield = complete_combat(Character::Ironclad, EncounterId::ThreeCultists,
        // Main fix: Ritual Dagger is the sole Attack/Power in hand, so Dual Wield copies it and the
        // original stays in hand (with two candidates the selected original is retired and copy+1
        // copies are made: accepted m10_dual_wield_multi_choice_retires_source_...).
        vec![card(1, RitualDagger, 0, 15), card(2, DualWield, 0, 0), card(3, Defend, 0, 0)]);
    dual_wield.choose_combat(crate::combat::CombatAction::PlayCard { instance: 2, target: None }).unwrap();
    let copy_instance = dual_wield.state.combat.as_ref().unwrap().public_full().unwrap().hand.iter()
        .find(|card| card.id == RitualDagger && card.instance.0 != 1)
        .expect("Dual Wield creates a second Ritual Dagger in hand").instance.0;
    set_enemy_hp(&mut dual_wield, 0, 15);
    kill_with_ritual_dagger(&mut dual_wield, 0);
    assert_eq!(dagger_view(&dual_wield, 1).misc, 18, "Fatal grows the original Ritual Dagger");
    assert_eq!(dagger_view(&dual_wield, copy_instance).misc, 15,
        "Dual Wield's stat-equivalent copy does not grow with the original");

    // Victory applies the journal exactly once at the Run settlement owner; `choose_combat`
    // settles a combat whose outcome is set (Main fix: the original called settle_combat again).
    let mut victory = complete_combat(Character::Ironclad, EncounterId::JawWorm,
        vec![card(1, RitualDagger, 0, 15)]);
    assert_eq!(victory.state.deck[0].misc, 15, "growth is not applied before the Fatal");
    set_enemy_hp(&mut victory, 0, 15);
    kill_with_ritual_dagger(&mut victory, 0);
    assert_eq!(victory.state.last_combat.as_ref().map(|r| r.outcome), Some(crate::combat::Outcome::Victory));
    assert_eq!(victory.state.deck[0].misc, 18, "Victory persists 15+3 exactly once; design §8.6");
    assert_eq!(victory.state.combat_progress.settled, 1, "one settlement");
    assert!(victory.state.combat.is_none());
    assert!(victory.state.fault.is_none());

    // A normal Death after the accepted Fatal keeps the Run healthy and does not settle growth.
    let mut death = complete_combat(Character::Ironclad, EncounterId::Reptomancer,
        vec![card(1, RitualDagger, 0, 15)]);
    let dagger_target = death.state.combat.as_ref().unwrap().public_full().unwrap().enemies.iter()
        .position(|enemy| enemy.enemy_id == crate::content::EnemyId::Dagger).unwrap();
    set_enemy_hp(&mut death, dagger_target, 15);
    kill_with_ritual_dagger(&mut death, dagger_target);
    assert_eq!(death.state.combat.as_ref().unwrap().public_full().unwrap().outcome, None,
        "Reptomancer remains active after the Fatal, allowing a subsequent Death in the same combat");
    let combat = death.state.combat.as_mut().unwrap();
    combat.enqueue_native_effects(vec![crate::combat::Effect {
        source: crate::combat::Source::None,
        target: Some(crate::combat::Actor::Player),
        kind: crate::combat::EffectKind::Damage(crate::combat::DamagePayload::HpLoss(999)),
    }], None).unwrap();
    assert_eq!(death.state.combat.as_ref().unwrap().public_full().unwrap().outcome, Some(crate::combat::Outcome::Death));
    death.settle_combat().unwrap_or_else(|error| panic!("post-growth Death settlement: {error:?}"));
    assert!(death.state.fault.is_none(), "Death after Ritual Dagger growth is not a Run fault");
    assert_eq!(death.state.deck[0].misc, 15, "Death does not persist the Victory-only mutation");
}

#[test]
fn m26_diagnostics_constructors() {
    let characters = [Character::Ironclad, Character::Silent, Character::Defect, Character::Watcher];
    for character in characters {
        let run = diagnostics::complete_run(character, supplied(vec![card(1, Strike, 0, 0)]))
            .unwrap_or_else(|error| panic!("complete_run {character:?}: {error:?}"));
        assert_eq!(run.state.card_admission, CardAdmission::Complete, "complete_run admission {character:?}");
        assert_eq!(run.state.combat_progress.entered, 0, "complete_run has accepted no combat actions");
        assert!(run.state.combat.is_none() && run.state.active_combat.is_none());
        assert!(run.state.history.events.is_empty(), "complete_run has accepted no Run actions");
        assert!(run.state.fault.is_none());

        let combat = diagnostics::complete_combat(character, supplied(vec![card(1, Strike, 0, 0)]), EncounterId::Cultist)
            .unwrap_or_else(|error| panic!("complete_combat {character:?}: {error:?}"));
        assert_eq!(combat.state.card_admission, CardAdmission::Complete, "complete_combat admission {character:?}");
        assert_eq!(combat.state.combat_progress.entered, 1);
        let view = combat.state.combat.as_ref().expect("complete_combat enters one combat").public_full().unwrap();
        assert_eq!(view.turn, 1, "complete_combat accepted no player turn for {character:?}");
        assert!(view.selection.is_none() && view.playing_cards.is_empty(), "no combat continuation/action for {character:?}");
        assert!(combat.state.combat.as_ref().unwrap().fault_record().is_none());
        assert!(combat.state.fault.is_none());
    }

    // Pride has no native CardId in this snapshot; Unknown("pride") is the rejected
    // sentinel for design §4.4's explicit Pride exclusion.
    let invalid = [
        card(1, CardId::Unknown("pride"), 0, 0),
        card(1, Wound, 0, 0),
        card(1, FameAndFortune, 0, 0),
        card(1, Regret, 1, 0),
        card(1, RitualDagger, 0, 14),
    ];
    for character in characters {
        for bad in invalid.iter().cloned() {
            let start = supplied(vec![bad.clone()]);
            assert!(diagnostics::complete_run(character, start.clone()).is_err(),
                "complete_run rejects before returning {character:?} Run: {bad:?}");
            assert!(diagnostics::complete_combat(character, start, EncounterId::Cultist).is_err(),
                "complete_combat rejects before returning {character:?} Run: {bad:?}");
        }
    }
}

#[test]
fn m26_any_colour_draw_is_uniform_over_the_flat_family() {
    // Disclosed decision F1 (seam map H0 §7): an any-colour draw is one uniform index over the flat
    // rarity family (76/160/84), with no per-colour weighting.
    for (rarity, size) in [(CardRarity::Common, 76), (CardRarity::Uncommon, 160), (CardRarity::Rare, 84)] {
        let pool = content::pools::CardPool::AnyColour { rarity: Some(rarity) };
        let family = content::pools::cards(pool).unwrap();
        assert_eq!(family.len(), size, "AnyColour {rarity:?} flat family");
        let mut colours = BTreeSet::new();
        for key in 0..200u64 {
            let mut run = complete_run(Character::Ironclad, vec![card(1, Strike, 0, 0)]);
            let mut rng = run.state.rng.clone();
            let drawn = mutations::draw_card(&mut run.state, pool, &[26, 1, key]).unwrap();
            let flat = super::super::rooms::m15_content::sample_capture_row_for(&mut rng, &[26, 1, key],
                &[family.as_slice()], RunRngScope::M26CardOffer, "incomplete", "empty").unwrap();
            assert_eq!(vec![drawn], flat, "F1: {rarity:?} key {key} is one flat-family index");
            colours.insert(format!("{:?}", Catalog::standard().card(drawn).unwrap().color));
        }
        let expected_colours = if rarity == CardRarity::Common { 4 } else { 5 };
        assert_eq!(colours.len(), expected_colours, "F1: {rarity:?} draws reach every colour in its family");
    }
}
