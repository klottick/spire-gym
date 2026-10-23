//! Authored M2 combat checkpoints, not a normal run or complete content pool.
mod m2_support;
use m2_support::*;
use spire_core::{combat::*, content::{CardId, EnemyId, PotionId, RelicId}, items::CombatItemContext};

fn start(entry: CombatEntry, relics: Vec<RelicId>, potions: Vec<Option<PotionId>>) -> Result<Combat, String> {
    let mut owner = Some(CombatItemContext::new(relics, potions));
    Combat::enter_with_item_context(entry, &mut owner).map_err(|error| format!("{error:?}"))
}

fn show(combat: &Combat) -> Result<(), String> {
    let view = combat.public_full().map_err(|e| format!("{e:?}"))?;
    println!("turn={} player={:?} enemies={:?}", view.turn, view.player, view.enemies);
    println!("  hand={:?} draw_count={} discard={} exhaust={} inventory={:?} selection={:?} outcome={:?}",
        view.hand.iter().map(|c| (c.instance.0, c.id.as_str())).collect::<Vec<_>>(),
        view.draw_count, view.discard.len(), view.exhaust.len(), view.inventory, view.selection, view.outcome);
    Ok(())
}

fn take(script: &mut Script, combat: &mut Combat, action: CombatAction) -> Result<(), String> {
    show(combat)?;
    println!("  menu={:?} action={action:?}", combat.legal_actions_full().map_err(|e| format!("{e:?}"))?);
    script.step(combat, action).map_err(|e| e.to_string())?;
    show(combat)
}

fn play(script: &mut Script, combat: &mut Combat, id: CardId) -> Result<(), String> {
    let action = card_action(combat, id).map_err(|e| format!("{e:?}"))?;
    take(script, combat, action)
}

fn potion(script: &mut Script, combat: &mut Combat, slot: usize, target: Option<TargetId>) -> Result<(), String> {
    take(script, combat, CombatAction::UsePotion { slot, target })
}

fn powers(seed: u64) -> Result<(), String> {
    let mut script = Script::new(64);
    println!("powers: real Strength/Dexterity/Ancient potions, Strike, card Block and exhausted Disarm");
    let mut combat = start(cards_entry(&[CardId::Strike, CardId::Defend, CardId::Disarm], seed), vec![],
        vec![Some(PotionId::StrengthPotion), Some(PotionId::DexterityPotion), Some(PotionId::AncientPotion)])?;
    for slot in 0..3 { potion(&mut script, &mut combat, slot, None)?; }
    for id in [CardId::Strike, CardId::Defend, CardId::Disarm] { play(&mut script, &mut combat, id)?; }

    for protected in [false, true] {
        println!("powers: native Spike Lick Frail, Dexterity card Block, Ancient protection={protected}");
        let mut combat = start(fixture(EnemyId::SpikeSlimeM, seed), vec![],
            vec![Some(PotionId::DexterityPotion), Some(PotionId::AncientPotion)])?;
        potion(&mut script, &mut combat, 0, None)?;
        if protected { potion(&mut script, &mut combat, 1, None)?; }
        take(&mut script, &mut combat, CombatAction::EndTurn)?;
        play(&mut script, &mut combat, CardId::Defend)?;
        take(&mut script, &mut combat, CombatAction::EndTurn)?;
    }
    println!("powers: Ancient protects against actual Louse SpitWeb");
    let mut combat = start(fixture(EnemyId::LouseGreen, seed), vec![], vec![Some(PotionId::AncientPotion)])?;
    potion(&mut script, &mut combat, 0, None)?;
    while combat.public_full().map_err(|e| format!("{e:?}"))?.enemies[0].intent != FullIntentId::SpitWeb {
        take(&mut script, &mut combat, CombatAction::EndTurn)?;
    }
    take(&mut script, &mut combat, CombatAction::EndTurn)?;
    println!("scenario=powers checkpoints complete; accepted_actions={}", script.accepted());
    Ok(())
}

fn selection(seed: u64) -> Result<(), String> {
    let mut script = Script::new(64);
    println!("selection: exact Defend sibling exhausted, two actual discard cards recycled/drawn");
    let mut combat = start(cards_entry(&[CardId::Strike, CardId::Strike, CardId::Defend,
        CardId::Defend, CardId::BurningPact, CardId::PommelStrike], seed), vec![], vec![])?;
    for action in [CombatAction::PlayCard { instance: 3, target: None },
        CombatAction::PlayCard { instance: 1, target: Some(TargetId(1)) },
        CombatAction::PlayCard { instance: 5, target: None }, CombatAction::SelectCard { instance: 4 },
        CombatAction::EndTurn] {
        take(&mut script, &mut combat, action)?;
    }
    println!("scenario=selection checkpoints complete; accepted_actions={}", script.accepted());
    Ok(())
}

fn damage(seed: u64) -> Result<(), String> {
    let mut script = Script::new(64);
    println!("damage: NORMAL Chomp against actual Defend Block");
    let mut combat = start(fixture(EnemyId::JawWorm, seed), vec![], vec![])?;
    play(&mut script, &mut combat, CardId::Defend)?;
    take(&mut script, &mut combat, CombatAction::EndTurn)?;

    println!("damage: Fire THORNS against actual enemy Block, with Strength and enemy Vulnerable");
    let mut combat = start(fixture(EnemyId::JawWorm, seed), vec![],
        vec![Some(PotionId::StrengthPotion), Some(PotionId::FearPotion), Some(PotionId::FirePotion)])?;
    while combat.public_full().map_err(|e| format!("{e:?}"))?.enemies[0].block == 0 {
        take(&mut script, &mut combat, CombatAction::EndTurn)?;
    }
    potion(&mut script, &mut combat, 0, None)?;
    potion(&mut script, &mut combat, 1, Some(TargetId(1)))?;
    potion(&mut script, &mut combat, 2, Some(TargetId(1)))?;

    for hp in [20, 3] {
        println!("damage: Bloodletting HP_LOSS before energy, supplied initial hp={hp}");
        let mut entry = cards_entry(&[CardId::Defend, CardId::Bloodletting], seed);
        entry.setup.player.hp = hp;
        let mut combat = start(entry, vec![], vec![])?;
        play(&mut script, &mut combat, CardId::Defend)?;
        play(&mut script, &mut combat, CardId::Bloodletting)?;
    }
    println!("scenario=damage checkpoints complete; accepted_actions={}", script.accepted());
    Ok(())
}

fn main() -> Result<(), String> {
    let mut scenario = "all".to_owned();
    let mut seed = 7_u64;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--scenario" => scenario = args.next().ok_or("--scenario requires a value")?,
            "--seed" => seed = args.next().ok_or("--seed requires a u64")?.parse().map_err(|_| "--seed requires a u64")?,
            "--help" | "-h" => {
                println!("m2_combat --scenario all|powers|selection|damage --seed <u64> (default7)");
                return Ok(());
            }
            _ => return Err(format!("unknown argument {arg}")),
        }
    }
    if !["all", "powers", "selection", "damage"].contains(&scenario.as_str()) { return Err(format!("unknown scenario {scenario}")); }
    println!("M2 authored Ironclad A0 combat checkpoints, seed={seed}; not normal runs or complete pools");
    if scenario == "all" || scenario == "powers" { powers(seed)?; }
    if scenario == "all" || scenario == "selection" { selection(seed)?; }
    if scenario == "all" || scenario == "damage" { damage(seed)?; }
    Ok(())
}
