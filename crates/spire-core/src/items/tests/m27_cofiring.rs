//! Per-relic attribution for the H0 co-firing pair ledger (M27 ruling `pair_test`).
use super::*;
use crate::combat::{Actor, EnemyHandle, Hook};
use crate::content::RelicId;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PairPhase {
    CombatStarted,
    HpLoss,
    TurnStartedPreDraw,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PairProfile {
    TrainingIronclad,
    M15Silent,
    M20Defect,
    M25Watcher,
}

#[derive(Clone, Copy, Debug)]
struct PairCase {
    id: &'static str,
    a: RelicId,
    b: RelicId,
    phase: PairPhase,
    profile: PairProfile,
}

macro_rules! pair {
    ($id:literal, $a:ident, $b:ident, $phase:ident, $profile:ident) => {
        PairCase {
            id: $id,
            a: RelicId::$a,
            b: RelicId::$b,
            phase: PairPhase::$phase,
            profile: PairProfile::$profile,
        }
    };
}

// Literal pair keys and phases from task0/a2-order/cofiring-pairs.tsv. The selected profile
// is one admitted union for that row: TRAINING_RELICS, M15 Silent, M20 Defect, or M25 Watcher.
const COFIRING_PAIRS: &[PairCase] = &[
    pair!("P01", RedSkull, NeowsLament, CombatStarted, TrainingIronclad),
    pair!("P02", RedSkull, Vajra, CombatStarted, TrainingIronclad),
    pair!("P03", RedSkull, OddlySmoothStone, CombatStarted, TrainingIronclad),
    pair!("P04", RedSkull, BagOfPreparation, CombatStarted, TrainingIronclad),
    pair!("P05", RedSkull, BronzeScales, CombatStarted, TrainingIronclad),
    pair!("P06", RedSkull, BloodVial, CombatStarted, TrainingIronclad),
    pair!("P07", RedSkull, MarkOfPain, CombatStarted, TrainingIronclad),
    pair!("P08", RedSkull, BagOfMarbles, CombatStarted, TrainingIronclad),
    pair!("P09", RedSkull, RedMask, CombatStarted, TrainingIronclad),
    pair!("P10", RedSkull, Anchor, CombatStarted, TrainingIronclad),
    pair!("P11", NeowsLament, Vajra, CombatStarted, TrainingIronclad),
    pair!("P12", NeowsLament, OddlySmoothStone, CombatStarted, TrainingIronclad),
    pair!("P13", NeowsLament, BagOfPreparation, CombatStarted, TrainingIronclad),
    pair!("P14", NeowsLament, BronzeScales, CombatStarted, TrainingIronclad),
    pair!("P15", NeowsLament, BloodVial, CombatStarted, TrainingIronclad),
    pair!("P16", NeowsLament, MarkOfPain, CombatStarted, TrainingIronclad),
    pair!("P17", NeowsLament, BagOfMarbles, CombatStarted, TrainingIronclad),
    pair!("P18", NeowsLament, RedMask, CombatStarted, TrainingIronclad),
    pair!("P19", NeowsLament, Anchor, CombatStarted, TrainingIronclad),
    pair!("P20", Vajra, MarkOfPain, CombatStarted, TrainingIronclad),
    pair!("P21", OddlySmoothStone, MarkOfPain, CombatStarted, TrainingIronclad),
    pair!("P22", BagOfPreparation, MarkOfPain, CombatStarted, TrainingIronclad),
    pair!("P23", BronzeScales, MarkOfPain, CombatStarted, TrainingIronclad),
    pair!("P24", BloodVial, MarkOfPain, CombatStarted, TrainingIronclad),
    pair!("P25", Vajra, BagOfMarbles, CombatStarted, TrainingIronclad),
    pair!("P26", Vajra, RedMask, CombatStarted, TrainingIronclad),
    pair!("P27", OddlySmoothStone, BagOfMarbles, CombatStarted, TrainingIronclad),
    pair!("P28", OddlySmoothStone, RedMask, CombatStarted, TrainingIronclad),
    pair!("P29", BagOfPreparation, BagOfMarbles, CombatStarted, TrainingIronclad),
    pair!("P30", BagOfPreparation, RedMask, CombatStarted, TrainingIronclad),
    pair!("P31", BronzeScales, BagOfMarbles, CombatStarted, TrainingIronclad),
    pair!("P32", BronzeScales, RedMask, CombatStarted, TrainingIronclad),
    pair!("P33", BloodVial, BagOfMarbles, CombatStarted, TrainingIronclad),
    pair!("P34", BloodVial, RedMask, CombatStarted, TrainingIronclad),
    pair!("P35", Vajra, Anchor, CombatStarted, TrainingIronclad),
    pair!("P36", OddlySmoothStone, Anchor, CombatStarted, TrainingIronclad),
    pair!("P37", BagOfPreparation, Anchor, CombatStarted, TrainingIronclad),
    pair!("P38", BronzeScales, Anchor, CombatStarted, TrainingIronclad),
    pair!("P39", BloodVial, Anchor, CombatStarted, TrainingIronclad),
    pair!("P40", MarkOfPain, BagOfMarbles, CombatStarted, TrainingIronclad),
    pair!("P41", MarkOfPain, RedMask, CombatStarted, TrainingIronclad),
    pair!("P42", MarkOfPain, Anchor, CombatStarted, TrainingIronclad),
    pair!("P43", BagOfMarbles, Anchor, CombatStarted, TrainingIronclad),
    pair!("P44", RedMask, Anchor, CombatStarted, TrainingIronclad),
    pair!("P45", SelfFormingClay, RedSkull, HpLoss, TrainingIronclad),
    pair!("P46", SelfFormingClay, RunicCube, HpLoss, TrainingIronclad),
    pair!("P47", RedSkull, RunicCube, HpLoss, TrainingIronclad),
    pair!("P48", NeowsLament, RingOfTheSnake, CombatStarted, M15Silent),
    pair!("P49", RingOfTheSnake, Anchor, CombatStarted, M15Silent),
    pair!("P50", RingOfTheSnake, BagOfMarbles, CombatStarted, M15Silent),
    pair!("P51", RingOfTheSnake, RedMask, CombatStarted, M15Silent),
    pair!("P52", NeowsLament, TwistedFunnel, CombatStarted, M15Silent),
    pair!("P53", Vajra, TwistedFunnel, CombatStarted, M15Silent),
    pair!("P54", OddlySmoothStone, TwistedFunnel, CombatStarted, M15Silent),
    pair!("P55", BagOfPreparation, TwistedFunnel, CombatStarted, M15Silent),
    pair!("P56", BronzeScales, TwistedFunnel, CombatStarted, M15Silent),
    pair!("P57", BloodVial, TwistedFunnel, CombatStarted, M15Silent),
    pair!("P58", RingOfTheSnake, TwistedFunnel, CombatStarted, M15Silent),
    pair!("P59", BagOfMarbles, TwistedFunnel, CombatStarted, M15Silent),
    pair!("P60", RedMask, TwistedFunnel, CombatStarted, M15Silent),
    pair!("P61", TwistedFunnel, Anchor, CombatStarted, M15Silent),
    pair!("P62", NeowsLament, CrackedCore, CombatStarted, M20Defect),
    pair!("P63", NeowsLament, DataDisk, CombatStarted, M20Defect),
    pair!("P64", CrackedCore, Anchor, CombatStarted, M20Defect),
    pair!("P65", DataDisk, Anchor, CombatStarted, M20Defect),
    pair!("P66", EmotionChip, GoldPlatedCables, TurnStartedPreDraw, M20Defect),
    pair!("P67", CrackedCore, BagOfMarbles, CombatStarted, M20Defect),
    pair!("P68", CrackedCore, RedMask, CombatStarted, M20Defect),
    pair!("P69", DataDisk, BagOfMarbles, CombatStarted, M20Defect),
    pair!("P70", DataDisk, RedMask, CombatStarted, M20Defect),
    pair!("P71", RunicCapacitor, EmotionChip, TurnStartedPreDraw, M20Defect),
    pair!("P72", RunicCapacitor, GoldPlatedCables, TurnStartedPreDraw, M20Defect),
    pair!("P73", Inserter, EmotionChip, TurnStartedPreDraw, M20Defect),
    pair!("P74", Inserter, GoldPlatedCables, TurnStartedPreDraw, M20Defect),
    pair!("P76", NeowsLament, PureWater, CombatStarted, M25Watcher),
    pair!("P77", PureWater, Anchor, CombatStarted, M25Watcher),
    pair!("P78", PureWater, RedMask, CombatStarted, M25Watcher),
    pair!("P79", Damaru, GoldenEye, TurnStartedPreDraw, M25Watcher),
];

// These are the ledger sides with no Effect/RelicHookWork from the named dispatcher
// at the ledger phase. P45/P46 still mutate Clay's pending-Block counter.
const EMPTY_OUTPUT_EXCEPTIONS: &[(&str, RelicId)] = &[
    ("P45", RelicId::SelfFormingClay),
    ("P46", RelicId::SelfFormingClay),
    // Cracked Core channels at atPreBattle (the startup dispatch), not at CombatStarted
    // (source-check-turnstart-hooks-v1).
    ("P62", RelicId::CrackedCore),
    ("P64", RelicId::CrackedCore),
    ("P66", RelicId::GoldPlatedCables),
    ("P67", RelicId::CrackedCore),
    ("P68", RelicId::CrackedCore),
    ("P72", RelicId::GoldPlatedCables),
    ("P74", RelicId::GoldPlatedCables),
    ("P76", RelicId::PureWater),
    ("P77", RelicId::PureWater),
    ("P78", RelicId::PureWater),
    ("P79", RelicId::GoldenEye),
];

#[derive(Clone, Debug)]
struct HookOutcome {
    work: RelicHookWork,
    callback_changed_context: bool,
    context: CombatItemContext,
}

#[derive(Clone, Debug)]
struct HpLossOutcome {
    hp_changed: Vec<crate::combat::Effect>,
    phase: RelicHookWork,
    callback_changed_context: bool,
    context: CombatItemContext,
}

fn context_for(profile: PairProfile, relics: &[RelicId]) -> CombatItemContext {
    let context = CombatItemContext::new(relics.to_vec(), vec![]);
    match profile {
        PairProfile::TrainingIronclad => context,
        PairProfile::M15Silent => context.with_m15_profile(),
        PairProfile::M20Defect => context.with_m20_profile(),
        PairProfile::M25Watcher => context.with_m25_profile(),
    }
}

fn dispatch_hook(context: &mut CombatItemContext, hook: Hook, hp: i32, max_hp: i32) -> RelicHookWork {
    let player = Actor::Player;
    let enemies = [
        EnemyHandle::parse("enemy-1").unwrap(),
        EnemyHandle::parse("enemy-2").unwrap(),
    ];
    let mut facts = dispatch::PhaseFacts::new(Some(hook), hp, max_hp, 0);
    facts.target = Some(&player);
    facts.enemies = &enemies;
    facts.live_enemies = &enemies;
    facts.hand_size = 2;
    facts.has_empty_orb = true;
    relic_phase_effects(context, &facts).expect("co-firing hook dispatch succeeds")
}

fn run_hook(pair: PairCase, relics: &[RelicId]) -> HookOutcome {
    let mut context = context_for(pair.profile, relics);
    let _entry_work = startup_work(&mut context).expect("combat-entry startup dispatch succeeds");

    if pair.phase == PairPhase::TurnStartedPreDraw {
        if relics.contains(&RelicId::Inserter) {
            // The ledger's Inserter rows are the second trigger: the accepted counter was 1.
            context.set_counter(RelicId::Inserter, 1);
        }
        if relics.contains(&RelicId::EmotionChip) {
            // The item phase follows the earlier player HP-loss arm point.
            context.emotion_chip_armed = true;
        }
    }

    let before = context.clone();
    let hook = match pair.phase {
        PairPhase::CombatStarted => Hook::CombatStarted,
        PairPhase::TurnStartedPreDraw => Hook::TurnStartedPreDraw,
        PairPhase::HpLoss => panic!("HP-loss row sent through hook dispatch"),
    };
    let work = dispatch_hook(&mut context, hook, 20, 50);
    HookOutcome {
        work,
        callback_changed_context: context != before,
        context,
    }
}

fn run_hp_loss(pair: PairCase, relics: &[RelicId]) -> HpLossOutcome {
    let mut context = context_for(pair.profile, relics);
    let _entry_work = startup_work(&mut context).expect("combat-entry startup dispatch succeeds");
    // Red Skull starts above half HP, then this damage crosses its threshold.
    let _start_work = dispatch_hook(&mut context, Hook::CombatStarted, 26, 50);
    let before = context.clone();

    let player = Actor::Player;
    let mut facts = dispatch::PhaseFacts::new(Some(Hook::Damage), 21, 50, 0);
    facts.target = Some(&player);
    facts.hp_lost = 5;
    // Match combat/effects.rs: laws::hp_changed runs before the HpLost phase dispatcher.
    let hp_changed = laws::hp_changed(&mut context, facts.hp_lost, facts.hp, facts.max_hp)
        .expect("positive player HP-loss callback succeeds");
    let phase = dispatch::combat_phase(&mut context, dispatch::ItemPhase::HpLost, &facts)
        .expect("HpLost dispatcher succeeds");
    HpLossOutcome {
        hp_changed,
        phase,
        callback_changed_context: context != before,
        context,
    }
}

fn source_top(id: RelicId) -> bool {
    matches!(
        id,
        RelicId::BloodVial
            | RelicId::Lantern
            | RelicId::Orichalcum
            | RelicId::Brimstone
            | RelicId::RunicCube
            | RelicId::CharonsAshes
            | RelicId::AncientTeaSet
            | RelicId::BottledFlame
            | RelicId::BirdFacedUrn
            | RelicId::CoffeeDripper
            | RelicId::CursedKey
            | RelicId::NlothsHungryFace
            // atBattleStart addToTop (source-check-turnstart-hooks-v1).
            | RelicId::Akabeko | RelicId::BronzeScales | RelicId::ClockworkSouvenir | RelicId::DataDisk
            | RelicId::DuVuDoll | RelicId::Girya | RelicId::MutagenicStrength | RelicId::OddlySmoothStone
            | RelicId::Pantograph | RelicId::SlingOfCourage | RelicId::ThreadAndNeedle | RelicId::Vajra
            | RelicId::NeowsLament | RelicId::PreservedInsect | RelicId::PhilosophersStone | RelicId::TeardropLocket
    )
}

fn concatenated<T: Clone>(first: &[T], second: &[T]) -> Vec<T> {
    first.iter().chain(second).cloned().collect()
}

fn assert_dispatch_order(
    pair: PairCase,
    order: &str,
    first: &RelicHookWork,
    second: &RelicHookWork,
    combined: &RelicHookWork,
) {
    let reverse_top = source_top(pair.a) && source_top(pair.b);
    let expected_top = if reverse_top {
        concatenated(&second.top, &first.top)
    } else {
        concatenated(&first.top, &second.top)
    };
    assert_eq!(combined.top, expected_top, "{} [{order}] top", pair.id);
    assert_eq!(combined.bottom, concatenated(&first.bottom, &second.bottom), "{} [{order}] bottom", pair.id);
    assert_eq!(combined.direct_strength, first.direct_strength + second.direct_strength,
        "{} [{order}] direct_strength", pair.id);
}

fn assert_callback_counters(pair: PairCase, order: &str, combined: &CombatItemContext,
    a: &CombatItemContext, b: &CombatItemContext) {
    assert_eq!(combined.counter(pair.a), a.counter(pair.a), "{} [{order}] A counter", pair.id);
    assert_eq!(combined.counter(pair.b), b.counter(pair.b), "{} [{order}] B counter", pair.id);
}

fn has_empty_output_exception(pair: PairCase, relic: RelicId) -> bool {
    EMPTY_OUTPUT_EXCEPTIONS.contains(&(pair.id, relic))
}

fn assert_single_is_attributable(pair: PairCase, side: &str, relic: RelicId,
    has_output: bool, changed_context: bool) {
    if has_empty_output_exception(pair, relic) {
        assert!(!has_output, "{} [{side}] {:?} unexpectedly emitted work at {:?}",
            pair.id, relic, pair.phase);
        if relic == RelicId::SelfFormingClay {
            assert!(changed_context, "{} [{side}] Self-Forming Clay must advance its pending-Block counter", pair.id);
        }
    } else {
        assert!(has_output, "{} [{side}] {:?} emitted no work at {:?}",
            pair.id, relic, pair.phase);
    }
}

#[test]
fn m27_cofiring_pairs_follow_acquisition_order() {
    assert_eq!(COFIRING_PAIRS.len(), 78, "the ledger has 78 pairs; P75 is only a sentinel");
    let mut pair_ids = std::collections::BTreeSet::new();
    for pair in COFIRING_PAIRS {
        assert!(pair_ids.insert(pair.id), "duplicate pair row {}", pair.id);
        assert_ne!(pair.id, "P75", "the none-qualifying sentinel is not a pair");

        let a_ids = [pair.a];
        let b_ids = [pair.b];
        let ab_ids = [pair.a, pair.b];
        let ba_ids = [pair.b, pair.a];

        if pair.phase == PairPhase::HpLoss {
            let a = run_hp_loss(*pair, &a_ids);
            let b = run_hp_loss(*pair, &b_ids);
            let ab = run_hp_loss(*pair, &ab_ids);
            let ba = run_hp_loss(*pair, &ba_ids);
            assert_single_is_attributable(*pair, "A", pair.a,
                !a.hp_changed.is_empty() || !a.phase.is_empty(), a.callback_changed_context);
            assert_single_is_attributable(*pair, "B", pair.b,
                !b.hp_changed.is_empty() || !b.phase.is_empty(), b.callback_changed_context);

            assert_eq!(ab.hp_changed, concatenated(&a.hp_changed, &b.hp_changed),
                "{} [A,B] hp_changed", pair.id);
            assert_eq!(ba.hp_changed, concatenated(&b.hp_changed, &a.hp_changed),
                "{} [B,A] hp_changed", pair.id);
            assert_dispatch_order(*pair, "A,B", &a.phase, &b.phase, &ab.phase);
            assert_dispatch_order(*pair, "B,A", &b.phase, &a.phase, &ba.phase);
            assert_callback_counters(*pair, "A,B", &ab.context, &a.context, &b.context);
            assert_callback_counters(*pair, "B,A", &ba.context, &a.context, &b.context);
        } else {
            let a = run_hook(*pair, &a_ids);
            let b = run_hook(*pair, &b_ids);
            let ab = run_hook(*pair, &ab_ids);
            let ba = run_hook(*pair, &ba_ids);
            assert_single_is_attributable(*pair, "A", pair.a, !a.work.is_empty(), a.callback_changed_context);
            assert_single_is_attributable(*pair, "B", pair.b, !b.work.is_empty(), b.callback_changed_context);
            assert_dispatch_order(*pair, "A,B", &a.work, &b.work, &ab.work);
            assert_dispatch_order(*pair, "B,A", &b.work, &a.work, &ba.work);
            assert_callback_counters(*pair, "A,B", &ab.context, &a.context, &b.context);
            assert_callback_counters(*pair, "B,A", &ba.context, &a.context, &b.context);

            if pair.profile == PairProfile::M25Watcher && pair.b == RelicId::PureWater
                || pair.profile == PairProfile::M25Watcher && pair.a == RelicId::PureWater {
                let mut pure_water = context_for(PairProfile::M25Watcher, &[RelicId::PureWater]);
                let _entry_work = startup_work(&mut pure_water).expect("Watcher startup dispatch succeeds");
                let pre_draw = dispatch_hook(&mut pure_water, Hook::CombatStartedPreDraw, 20, 50);
                assert!(!pre_draw.is_empty(), "{} Pure Water fires at Hook::CombatStartedPreDraw", pair.id);
            }
        }
    }
}
