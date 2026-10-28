use spire_core::{
    combat::{
        enemies::{self, SetupContext}, rng::Partition, Combat, CombatAction, CombatCardRef,
        Outcome, PlayerState, permanent_live_card, CardInstanceId,
    },
    content::{CardId, Character, EncounterId, RelicId},
    run::{
        AuthoredRunStart, CardInstance, CreationProfile, GameAction, PublicDecision, Run,
        RunConfig, RunRng,
    },
};
use std::collections::BTreeMap;

const ACTION_BUDGET_MAX: usize = 8192;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Options {
    pub(crate) seed: u64,
    pub(crate) budget: usize,
    pub(crate) scenario: String,}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Report {
    label: String,
    actions: usize,
    outcome: Outcome,
    witness: String,
}

fn rng(seed: u64) -> RunRng {
    let mut root = [0; 32];
    root[..8].copy_from_slice(&seed.to_le_bytes());
    RunRng::new(root, Partition::Test, 0)
}

fn value<I>(args: &mut I, flag: &str) -> Result<String, String>
where
    I: Iterator<Item = String>,
{
    args.next().ok_or_else(|| format!("missing value for {flag}"))
}

pub(crate) fn parse_args<I>(args: I) -> Result<Options, String>
where
    I: IntoIterator<Item = String>,
{
    let mut seed = None;
    let mut budget = ACTION_BUDGET_MAX;
    let mut scenario = "all".to_owned();
    let mut args = args.into_iter();
    while let Some(flag) = args.next() {
        match flag.as_str() {
            "--seed" => {
                seed = Some(value(&mut args, "--seed")?.parse::<u64>().map_err(|_| "seed must be a u64")?);
            }
            "--budget" => {
                budget = value(&mut args, "--budget")?.parse::<usize>().map_err(|_| "budget must be an integer")?;
            }
            "--scenario" => scenario = value(&mut args, "--scenario")?,
            _ => return Err(format!("unknown argument {flag}")),
        }
    }
    let seed = seed.ok_or_else(|| "missing --seed <u64>".to_owned())?;
    if !(1..=ACTION_BUDGET_MAX).contains(&budget) {
        return Err(format!("budget must be in 1..={ACTION_BUDGET_MAX}"));
    }
    if !matches!(scenario.as_str(), "starter" | "supplied" | "all") {
        return Err("scenario must be starter|supplied|all".into());
    }
    Ok(Options { seed, budget, scenario })
}

fn authored_start(decision: &PublicDecision) -> Result<AuthoredRunStart, String> {
    if decision.player.relic_counters.len() != 0
        || decision.player.character != Character::Silent
        || decision.player.hp != 70
        || decision.player.max_hp != 70
        || decision.player.gold != 99
        || decision.player.relics != [RelicId::RingOfTheSnake]
        || decision.player.potions != [None, None, None]
        || decision.player.keys != [false; 3]
        || decision.objects.len() != 12
    {
        return Err("Run::new did not expose the expected canonical Silent A0 resources".into());
    }
    Ok(AuthoredRunStart {
        hp: decision.player.hp,
        max_hp: decision.player.max_hp,
        gold: decision.player.gold,
        deck: decision.objects.iter().map(|card| CardInstance { misc:0,
            instance: card.instance,
            id: card.id,
            upgrades: card.upgrades,
        }).collect(),
        relics: decision.player.relics.clone(),
        potions: decision.player.potions.clone(),
        relic_counters: BTreeMap::new(),
    })
}

fn starter(seed: u64, budget: usize) -> Result<Report, String> {
    let config = RunConfig { character: Character::Silent, ascension: 0 };
    let normal_start = Run::new(config, CreationProfile::experienced(), rng(seed))
        .map_err(|error| format!("Run::new: {error:?}"))?;
    let decision = normal_start.decision().map_err(|error| format!("Neow decision: {error:?}"))?;
    if decision.offers.is_empty() || decision.combat.is_some() {
        return Err("expected an unselected public Neow decision before authored combat".into());
    }
    let start = authored_start(&decision)?;
    let mut run = Run::from_authored_combat(config, start, rng(seed))
        .map_err(|error| format!("authored Silent bridge: {error:?}"))?;
    run.enter_combat(EncounterId::JawWorm)
        .map_err(|error| format!("enter authored Jaw Worm combat: {error:?}"))?;

    let mut actions = 0usize;
    let record = loop {
        let decision = run.decision().map_err(|error| format!("starter decision: {error:?}"))?;
        if let Some(record) = decision.last_combat {
            break record;
        }
        if actions >= budget {
            return Err(format!("starter budget exhausted after {actions} accepted actions"));
        }
        let action = decision.candidates.first().ok_or_else(|| "starter combat has no public legal action".to_owned())?.action.clone();
        if !matches!(action, GameAction::Combat(_)) {
            return Err("starter caller received a non-combat action while combat was active".into());
        }
        run.step(&action).map_err(|error| format!("starter native action {actions}: {error:?}"))?;
        actions += 1;
    };

    Ok(Report {
        label: "starter-authored-JawWorm".into(),
        actions,
        outcome: record.outcome,
        witness: "canonical Run::new Silent resources; authored combat bypassed Neow selection, map generation, and Act 1 traversal".into(),
    })
}

fn supplied_combat(seed: u64, ids: &[CardId]) -> Result<Combat, String> {
    let mut setup_rng = rng(seed);
    let cards = ids.iter().enumerate().map(|(index, id)| {
        permanent_live_card(*id, CardInstanceId(index as u64 + 1), 0)
    }).collect::<Vec<_>>();
    let entry = enemies::setup(EncounterId::JawWorm, 1, 0, &mut setup_rng, SetupContext {
        player: PlayerState {
            hp: 70,
            max_hp: 70,
            block: 0,
            energy: 3,
            max_energy: 3,
            draw_per_turn: 5,
            powers: vec![],
        },
        player_powers: vec![],
        character: Character::Silent,
        cards,
        opening_draw: ids.len(),
        encounter_visit: 0,
    }).map_err(|error| format!("supplied Jaw Worm setup: {error:?}"))?;
    let combat = Combat::enter(entry).map_err(|error| format!("supplied combat entry: {error:?}"))?;
    let public = combat.public_full().map_err(|error| format!("supplied opening decision: {error:?}"))?;
    if public.character.orbs.len() != 0 || public.hand.len() != ids.len() {
        return Err("supplied fixture did not enter as a complete Silent hand".into());
    }
    for id in ids {
        if !public.hand.iter().any(|card| card.id == *id) {
            return Err(format!("supplied opening hand is missing {id:?}"));
        }
    }
    Ok(combat)
}

fn accept_combat_action(combat: &mut Combat, action: CombatAction, actions: &mut usize, budget: usize) -> Result<(), String> {
    if *actions >= budget {
        return Err(format!("supplied action budget exhausted after {} accepted actions", *actions));
    }
    let legal = combat.legal_actions_full().map_err(|error| format!("legal menu: {error:?}"))?;
    if !legal.contains(&action) {
        return Err(format!("requested supplied action is absent from the current legal menu: {action:?}"));
    }
    combat.step_full(&action).map_err(|error| format!("supplied action {actions}: {error:?}"))?;
    *actions += 1;
    Ok(())
}

fn play_card(combat: &mut Combat, id: CardId, actions: &mut usize, budget: usize) -> Result<u64, String> {
    let public = combat.public_full().map_err(|error| format!("play-card observation: {error:?}"))?;
    let instance = public.hand.iter().find(|card| card.id == id)
        .ok_or_else(|| format!("requested {id:?} is absent from the current public hand"))?.instance.0;
    let action = combat.legal_actions_full().map_err(|error| format!("play-card legal menu: {error:?}"))?
        .into_iter().find(|action| matches!(action, CombatAction::PlayCard { instance: offered, .. } if *offered == instance))
        .ok_or_else(|| format!("{id:?} instance {instance} has no current legal play action"))?;
    accept_combat_action(combat, action, actions, budget)?;
    Ok(instance)
}

fn select_captured_card(combat: &mut Combat, id: CardId, actions: &mut usize, budget: usize) -> Result<u64, String> {
    let public = combat.public_full().map_err(|error| format!("selection observation: {error:?}"))?;
    let selection = public.selection.as_ref().ok_or_else(|| "the public combat has no active selection".to_owned())?;
    let instance = selection.candidates.iter().find(|card| card.id == id)
        .ok_or_else(|| format!("{id:?} is not an actual captured selection candidate"))?.instance.0;
    let action = CombatAction::SelectCard { instance };
    if !combat.legal_actions_full().map_err(|error| format!("selection legal menu: {error:?}"))?.contains(&action) {
        return Err(format!("captured {id:?} instance {instance} is absent from the current legal menu"));
    }
    accept_combat_action(combat, action, actions, budget)?;
    Ok(instance)
}

fn end_turn(combat: &mut Combat, actions: &mut usize, budget: usize) -> Result<(), String> {
    accept_combat_action(combat, CombatAction::EndTurn, actions, budget)
}

fn finish_supplied(mut combat: Combat, label: &str, mut actions: usize, budget: usize, witness: String) -> Result<Report, String> {
    loop {
        let public = combat.public_full().map_err(|error| format!("{label} outcome observation: {error:?}"))?;
        if let Some(outcome) = public.outcome {
            return Ok(Report { label: format!("supplied-{label}"), actions, outcome, witness });
        }
        if actions >= budget {
            return Err(format!("supplied {label} budget exhausted after {actions} accepted actions"));
        }
        let action = combat.legal_actions_full().map_err(|error| format!("{label} legal menu: {error:?}"))?
            .into_iter().next().ok_or_else(|| format!("supplied {label} combat has no public legal action"))?;
        combat.step_full(&action).map_err(|error| format!("supplied {label} native action {actions}: {error:?}"))?;
        actions += 1;
    }
}

fn supplied_poison(seed: u64, budget: usize) -> Result<Report, String> {
    let mut combat = supplied_combat(seed, &[
        CardId::NoxiousFumes, CardId::DeadlyPoison, CardId::PoisonedStab, CardId::Catalyst,
    ])?;
    let mut actions = 0;
    play_card(&mut combat, CardId::NoxiousFumes, &mut actions, budget)?;
    play_card(&mut combat, CardId::DeadlyPoison, &mut actions, budget)?;
    end_turn(&mut combat, &mut actions, budget)?;
    let public = combat.public_full().map_err(|error| format!("Poison checkpoint: {error:?}"))?;
    let poison = public.enemies[0].powers.iter().find(|power| power.id.0 == "poison")
        .map(|power| power.amount);
    if poison != Some(6) {
        return Err(format!("Poison checkpoint expected 6 after enemy tick and Noxious Fumes, got {poison:?}"));
    }
    play_card(&mut combat, CardId::PoisonedStab, &mut actions, budget)?;
    play_card(&mut combat, CardId::Catalyst, &mut actions, budget)?;
    let final_poison = combat.public_full().map_err(|error| format!("Catalyst checkpoint: {error:?}"))?
        .enemies[0].powers.iter().find(|power| power.id.0 == "poison").map(|power| power.amount);
    if final_poison != Some(18) {
        return Err(format!("Poisoned Stab plus base Catalyst expected Poison 18, got {final_poison:?}"));
    }
    finish_supplied(combat, "poison", actions, budget,
        "Noxious Fumes post-draw application and Poisoned Stab/Catalyst produced public Poison 18".into())
}

fn supplied_discard(seed: u64, budget: usize) -> Result<Report, String> {
    let mut combat = supplied_combat(seed, &[
        CardId::DaggerThrow, CardId::Survivor, CardId::Reflex,
        CardId::Tactician, CardId::Strike, CardId::Defend,
    ])?;
    let mut actions = 0;
    play_card(&mut combat, CardId::DaggerThrow, &mut actions, budget)?;
    let before_reflex_hand = combat.public_full().map_err(|error| format!("Reflex selector observation: {error:?}"))?.hand.len();
    let reflex = select_captured_card(&mut combat, CardId::Reflex, &mut actions, budget)?;
    let after_reflex = combat.public_full().map_err(|error| format!("Reflex reaction observation: {error:?}"))?;
    let expected_reflex_hand = before_reflex_hand.checked_sub(1).and_then(|count| count.checked_add(2))
        .ok_or_else(|| format!("Reflex hand-count arithmetic is invalid for {before_reflex_hand} cards"))?;
    if after_reflex.hand.len() != expected_reflex_hand {
        return Err(format!("selecting captured Reflex instance {reflex} should remove one card and draw two: hand {before_reflex_hand}->{}, expected {expected_reflex_hand}", after_reflex.hand.len()));
    }
    play_card(&mut combat, CardId::Survivor, &mut actions, budget)?;
    let after_survivor = combat.public_full().map_err(|error| format!("Survivor selection observation: {error:?}"))?;
    if after_survivor.player.block < 8 {
        return Err(format!("Survivor should grant base Block before discard; got {}", after_survivor.player.block));
    }
    let energy_before_tactician = after_survivor.player.energy;
    select_captured_card(&mut combat, CardId::Tactician, &mut actions, budget)?;
    let energy_after = combat.public_full().map_err(|error| format!("Tactician observation: {error:?}"))?.player.energy;
    if energy_after != energy_before_tactician + 1 {
        return Err(format!("Tactician base discard should add one energy: before {energy_before_tactician}, after {energy_after}"));
    }
    finish_supplied(combat, "discard", actions, budget,
        format!("Dagger Throw selected captured Reflex instance {reflex}, public hand {before_reflex_hand}→{} (-1+2 draw); Survivor selected captured Tactician, Block {} and energy {}→{}", after_reflex.hand.len(), after_survivor.player.block, energy_before_tactician, energy_after))
}

fn supplied_shiv(seed: u64, budget: usize) -> Result<Report, String> {
    let mut combat = supplied_combat(seed, &[
        CardId::Accuracy, CardId::BladeDance, CardId::Neutralize, CardId::Strike, CardId::Defend,
    ])?;
    let mut actions = 0;
    play_card(&mut combat, CardId::Accuracy, &mut actions, budget)?;
    play_card(&mut combat, CardId::BladeDance, &mut actions, budget)?;
    let public = combat.public_full().map_err(|error| format!("generated Shiv observation: {error:?}"))?;
    let shiv = public.hand.iter().find(|card| card.id == CardId::Shiv && matches!(card.origin, CombatCardRef::CombatLocal(_)))
        .ok_or_else(|| "Blade Dance did not expose an actual generated Shiv instance in hand".to_owned())?;
    let instance = shiv.instance.0;
    let expected_damage = 8;
    let enemy_before = public.enemies[0].hp + public.enemies[0].block;
    let action = combat.legal_actions_full().map_err(|error| format!("Shiv legal menu: {error:?}"))?
        .into_iter().find(|action| matches!(action, CombatAction::PlayCard { instance: offered, .. } if *offered == instance))
        .ok_or_else(|| format!("generated Shiv {instance} has no current legal targeted action"))?;
    accept_combat_action(&mut combat, action, &mut actions, budget)?;
    let after = combat.public_full().map_err(|error| format!("played Shiv observation: {error:?}"))?;
    let enemy_after = after.enemies[0].hp + after.enemies[0].block;
    if enemy_before - enemy_after != expected_damage || !after.exhaust.iter().any(|card| card.instance.0 == instance && card.id == CardId::Shiv) {
        return Err(format!("played Shiv {instance} should deal {expected_damage} and exhaust; combined HP/Block {enemy_before}->{enemy_after}"));
    }
    finish_supplied(combat, "shiv", actions, budget,
        format!("Blade Dance generated Shiv instance {instance}; Accuracy raised its damage to 8, it dealt 8 and exhausted"))
}

fn supplied_retain(seed: u64, budget: usize) -> Result<Report, String> {
    let mut combat = supplied_combat(seed, &[
        CardId::WellLaidPlans, CardId::Strike, CardId::Defend, CardId::Neutralize, CardId::DaggerThrow,
    ])?;
    let mut actions = 0;
    play_card(&mut combat, CardId::WellLaidPlans, &mut actions, budget)?;
    end_turn(&mut combat, &mut actions, budget)?;
    let selected = select_captured_card(&mut combat, CardId::Strike, &mut actions, budget)?;
    let public = combat.public_full().map_err(|error| format!("retain confirmation observation: {error:?}"))?;
    if public.selection.is_none() {
        return Err("Well-Laid Plans selection resolved before its legal confirmation step".into());
    }
    accept_combat_action(&mut combat, CombatAction::ConfirmSelection, &mut actions, budget)?;
    let retained = combat.public_full().map_err(|error| format!("retained card observation: {error:?}"))?;
    if !retained.hand.iter().any(|card| card.instance.0 == selected && card.id == CardId::Strike) {
        return Err(format!("captured Strike instance {selected} was not retained after end turn"));
    }
    if retained.turn != 2 {
        return Err(format!("retained card should be observed on turn 2, got turn {}", retained.turn));
    }
    finish_supplied(combat, "retain", actions, budget,
        format!("Well-Laid Plans retained the same Strike instance {selected} in hand on turn {}", retained.turn))
}

fn supplied_nightmare(seed: u64, budget: usize) -> Result<Report, String> {
    let mut combat = supplied_combat(seed, &[CardId::Nightmare, CardId::StrikeGreen, CardId::DefendGreen])?;
    let mut actions = 0;
    let nightmare = play_card(&mut combat, CardId::Nightmare, &mut actions, budget)?;
    let source = select_captured_card(&mut combat, CardId::StrikeGreen, &mut actions, budget)?;
    if combat.legal_actions_full().map_err(|error| format!("Nightmare selection follow-up menu: {error:?}"))?
        .contains(&CombatAction::ConfirmSelection) {
        accept_combat_action(&mut combat, CombatAction::ConfirmSelection, &mut actions, budget)?;
    }
    let captured = combat.public_full().map_err(|error| format!("Nightmare capture observation: {error:?}"))?;
    if !captured.hand.iter().any(|card| card.instance.0 == source && card.id == CardId::StrikeGreen
        && card.origin == CombatCardRef::Permanent(CardInstanceId(source))) {
        return Err(format!("Nightmare selection moved the original permanent Strike instance {source}"));
    }
    if !captured.exhaust.iter().any(|card| card.instance.0 == nightmare && card.id == CardId::Nightmare) {
        return Err(format!("Nightmare parent instance {nightmare} did not exhaust after capture"));
    }
    let immediate_copies = captured.hand.iter().chain(&captured.discard).chain(&captured.exhaust)
        .chain(&captured.playing_cards)
        .filter(|card| card.id == CardId::StrikeGreen && matches!(card.origin, CombatCardRef::CombatLocal(_)))
        .count();
    if immediate_copies != 0 {
        return Err(format!("Nightmare should deliver no immediate copies, found {immediate_copies}"));
    }

    end_turn(&mut combat, &mut actions, budget)?;
    let delivered = combat.public_full().map_err(|error| format!("Nightmare next-turn observation: {error:?}"))?;
    if delivered.turn != 2 {
        return Err(format!("Nightmare copies should be observed on turn 2, got turn {}", delivered.turn));
    }
    let copies = delivered.hand.iter().filter(|card| card.id == CardId::StrikeGreen
        && matches!(card.origin, CombatCardRef::CombatLocal(_))).collect::<Vec<_>>();
    let copy_ids = copies.iter().map(|card| card.instance.0).collect::<std::collections::BTreeSet<_>>();
    if copies.len() != 3 || copy_ids.len() != 3
        || copies.iter().any(|card| card.origin != CombatCardRef::CombatLocal(card.instance.0)) {
        return Err(format!("Nightmare next turn should expose three distinct CombatLocal Strike copies in hand, got {} with {} identities", copies.len(), copy_ids.len()));
    }

    let generated = copies[0].instance.0;
    let before_play = combat.public_full().map_err(|error| format!("generated Strike before play: {error:?}"))?;
    let enemy_before = before_play.enemies[0].hp + before_play.enemies[0].block;
    let action = combat.legal_actions_full().map_err(|error| format!("generated Strike legal menu: {error:?}"))?
        .into_iter().find(|action| matches!(action, CombatAction::PlayCard { instance, .. } if *instance == generated))
        .ok_or_else(|| format!("generated Strike instance {generated} has no current legal play action"))?;
    accept_combat_action(&mut combat, action, &mut actions, budget)?;
    let after_play = combat.public_full().map_err(|error| format!("generated Strike after play: {error:?}"))?;
    let enemy_after = after_play.enemies[0].hp + after_play.enemies[0].block;
    if enemy_before - enemy_after != 6 {
        return Err(format!("generated Strike instance {generated} should deal six: combined HP/Block {enemy_before}->{enemy_after}"));
    }
    finish_supplied(combat, "copy", actions, budget,
        format!("Nightmare captured permanent Strike {source}; parent {nightmare} exhausted with zero immediate copies; turn 2 delivered three distinct CombatLocal StrikeGreen copies, and instance {generated} dealt 6"))
}

fn supplied(seed: u64, budget: usize) -> Result<Vec<Report>, String> {
    [
        supplied_poison(seed, budget),
        supplied_discard(seed, budget),
        supplied_shiv(seed, budget),
        supplied_retain(seed, budget),
        supplied_nightmare(seed, budget),
    ].into_iter().collect()
}

fn execute(options: Options) -> Result<(), String> {
    let mut reports = Vec::new();
    if options.scenario != "supplied" {
        reports.push(starter(options.seed, options.budget)?);
    }
    if options.scenario != "starter" {
        reports.extend(supplied(options.seed, options.budget)?);
    }
    for report in reports {
        println!(
            "seed{} {}: {:?}; actions{}; {}",
            options.seed, report.label, report.outcome, report.actions, report.witness
        );
    }
    println!("Supplied rows are separate native diagnostics, not normal-start card acquisition.");
    Ok(())
}

fn main() {
    let result = parse_args(std::env::args().skip(1)).and_then(execute);
    if let Err(error) = result {
        eprintln!("M11 Silent combat: {error}");
        std::process::exit(2);
    }
}
