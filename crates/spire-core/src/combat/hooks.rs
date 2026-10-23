//! Typed combat lifecycle checkpoints consumed by the shared effect resolver.
use super::{Effect, EffectError, EffectKind, PowerId, PowerEntry, Source};
use super::model::{Actor, State};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hook {
    CardAcquired, CardRemoved, RoomEntered, CombatStarted, TurnStarted, TurnEnded,
    CardPlayed, CardDiscarded, CardExhausted, CardDrawn, Damage, Death, CombatSettled,
    ChestOpened{boss:bool},ChestOpenedAfter{boss:bool},Rested,HalfDeath,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HookEvent { pub hook: Hook, pub source: Source, pub target: Option<Actor> }

/// Execute the finite native behaviors owned by the combat foundation.
/// Producers enqueue this as ordinary effect work; failures leave it as the
/// resolver's current effect and therefore preserve first-fault provenance.
pub(super) fn dispatch(state: &mut State, hook: Hook, source: &Source,target:Option<&Actor>) -> Result<(), EffectError> {
    if !state.full_runtime { return Ok(()); }

    let mut work = Vec::new();
    if hook==Hook::Death{super::enemies::act2::stasis::on_death(state,target);}
    if hook == Hook::CardPlayed {
        work.extend(super::enemies::gremlin_nob_card_played_work(state, source)?);
        super::enemies::guardian::card_played(state,source)?;
        super::enemies::act2::chosen_snecko::card_played(state,source)?;
        super::enemies::act3::awakened::card_played(state,source)?;
    }
    if hook == Hook::TurnEnded {
        round_end_work(&mut state.player.powers, Actor::Player, &mut work);
        round_end_full_work(&mut state.player_full_powers, Actor::Player, &mut work);
        for enemy in state.enemies.iter_mut().filter(|enemy|
            enemy.life == super::ActorLife::Active && enemy.hp > 0) {
            let actor = Actor::Enemy(enemy.instance_id.clone());
            round_end_work(&mut enemy.powers, actor.clone(), &mut work);
            round_end_full_work(&mut enemy.full_powers, actor, &mut work);
        }
    }
    // Hook-produced work is resolved before the card/action continuation.
    for effect in work.into_iter().rev() { state.pending.push_front(effect); }
    Ok(())
}

fn round_end_full_work(powers: &mut [super::FullPowerEntry], actor: Actor, work: &mut Vec<Effect>) {
    for power in powers {
        let kind = match power.id.0 {
            "ritual" if power.state.skip_first => { power.state.skip_first = false; None }
            "ritual" => Some(EffectKind::ApplyFullPower { id: super::FullPowerId("strength"), amount: power.state.amount }),
            "strength_up" if matches!(actor,Actor::Enemy(_))=>Some(EffectKind::ApplyFullPower{id:super::FullPowerId("strength"),amount:power.state.amount}),
            "metallicize"=>Some(EffectKind::GainBlock(power.state.amount)),
            "slow"=>{power.state.amount=0;None},
            "intangible" if matches!(actor,Actor::Enemy(_))&&power.state.skip_first=>{power.state.skip_first=false;None},
            "intangible" if matches!(actor,Actor::Enemy(_))=>Some(EffectKind::ReduceFullPower{id:power.id,amount:1}),
            "draw_reduction" if power.state.skip_first=>{power.state.skip_first=false;None},
            "draw_reduction"=>Some(EffectKind::ReduceFullPower{id:power.id,amount:1}),
            "regeneration" if matches!(actor,Actor::Enemy(_))=>Some(EffectKind::Heal(power.state.amount)),
            "regenerate" if matches!(actor,Actor::Enemy(_))=>Some(EffectKind::Heal(power.state.amount)),
            "vulnerable" | "weak" | "frail" if power.state.monster_source_grace => { power.state.monster_source_grace = false; None }
            "vulnerable" | "weak" | "frail" => Some(EffectKind::ReduceFullPower { id: power.id, amount: 1 }),
            _ => None,
        };
        if let Some(kind) = kind {
            work.push(Effect { source: Source::Actor(actor.clone()), target: Some(actor.clone()), kind });
        }
    }
}

fn round_end_work(powers: &mut [PowerEntry], actor: Actor, work: &mut Vec<Effect>) {
    for power in powers {
        let kind = match power.id {
            PowerId::Ritual if power.state.skip_first => {
                power.state.skip_first = false;
                None
            }
            PowerId::Ritual => Some(EffectKind::ApplyPower { id: PowerId::Strength, amount: power.state.amount }),
            PowerId::Vulnerable if power.state.monster_source_grace => {
                power.state.monster_source_grace = false;
                None
            }
            PowerId::Vulnerable => Some(EffectKind::ReducePower { id: PowerId::Vulnerable, amount: 1 }),
            PowerId::Strength => None,
        };
        if let Some(kind) = kind {
            work.push(Effect { source: Source::Actor(actor.clone()), target: Some(actor.clone()), kind });
        }
    }
}
