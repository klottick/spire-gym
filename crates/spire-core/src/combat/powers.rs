//! Finite full-mode power operations on the existing actor-owned storage lanes.
use super::*;
use super::model::State;

pub(super) fn on_card_exhausted(state:&State,source:&super::Source)->Vec<super::Effect>{
    if !state.full_runtime{return Vec::new()}
    let monsters_alive=state.enemies.iter().any(|enemy|enemy.life==super::ActorLife::Active&&enemy.hp>0);
    state.player_full_powers.iter().filter_map(|power|{
        if power.state.amount<=0{return None}
        let kind=match power.id.0{
            "dark_embrace" if monsters_alive=>super::EffectKind::Draw(power.state.amount),
            "feel_no_pain"=>super::EffectKind::GainBlock(power.state.amount),
            _=>return None,
        };
        Some(super::Effect{source:source.clone(),target:Some(super::Actor::Player),kind})
    }).collect()
}

/// Finite source-qualified ordinary cleanse categories. Shackled has a separate
/// unconditional removal at the named Champ/Awakened phase transitions.
pub(super) fn cleanse_debuff(power:&FullPowerEntry)->Option<bool>{Some(match power.id.0{
     "weak"|"vulnerable"|"frail"|"hex"|"confused"|"entangled"|"poison"|"draw_reduction"|"no_draw"|"lose_strength"|"lose_dexterity"|"wraith_form"|"choked"|"corpse_explosion"=>true,
    "strength"|"dexterity"=>power.state.amount<=0,
    "thievery"|"plated_armor"|"spore_cloud"|"metallicize"|"artifact"|"ritual"|"enrage"|"regeneration"|"barricade"|"angry"|"flight"|"minion"|"painful_stabs"|"curl_up"|"split"|"mode_shift"|"sharp_hide"|"shackled"|"regenerate"|"curiosity"|"unawakened"|"time_warp"|"double_tap"|"burst"|"mayhem"|"panache"|"sadistic_nature"|"magnetism"=>false,
    _=>return None,
})}

fn lanes<'a>(state: &'a mut State, actor: &Actor) -> (&'a mut Vec<PowerEntry>, &'a mut Vec<FullPowerEntry>) {
    match actor {
        Actor::Player => (&mut state.player.powers, &mut state.player_full_powers),
        Actor::Enemy(id) => {
            let enemy = state.enemies.iter_mut().find(|e| &e.instance_id == id).expect("validated actor");
            (&mut enemy.powers, &mut enemy.full_powers)
        }
    }
}

fn amount(state: &State, actor: &Actor, id: FullPowerId) -> Result<i32, InputError> {
    if *actor==Actor::Player&&id.0=="focus"{return Ok(state.character.focus)}
    let (legacy, full) = match actor {
        Actor::Player => (&state.player.powers, &state.player_full_powers),
        Actor::Enemy(owner) => {
            let enemy = state.enemies.iter().find(|e| &e.instance_id == owner).expect("validated actor");
            (&enemy.powers, &enemy.full_powers)
        }
    };
    legacy.iter().filter(|p| p.id.as_str() == id.0).map(|p| p.state.amount)
        .chain(full.iter().filter(|p| p.id == id).map(|p| p.state.amount))
        .try_fold(0_i32, |a, b| a.checked_add(b))
        .ok_or_else(|| InputError::Invariant("power sum overflow".into()))
}

fn remove(state: &mut State, actor: &Actor, id: FullPowerId) {
    if *actor==Actor::Player&&id.0=="focus"{state.character.focus=0;}
    let (legacy, full) = lanes(state, actor);
    legacy.retain(|p| p.id.as_str() != id.0);
    full.retain(|p| p.id != id);
}

fn set(state: &mut State, effect: &Effect, id: FullPowerId, value: i32) {
    let actor = effect.target.as_ref().expect("validated power target");
    let (legacy, full) = lanes(state, actor);
    if let Some(existing) = legacy.iter_mut().find(|p| p.id.as_str() == id.0) {
        existing.state.amount = value;
        full.retain(|p| p.id != id);
    } else if let Some(existing) = full.iter_mut().find(|p| p.id == id) {
        existing.state.amount = value;
    } else if let EffectKind::ApplyPower { id: legacy_id, .. } = effect.kind
        && legacy_id.as_str() == id.0 {
        legacy.push(PowerEntry { id: legacy_id, state: PowerState { amount: value,
            skip_first: legacy_id == PowerId::Ritual,
            monster_source_grace: legacy_id == PowerId::Vulnerable
                && matches!(effect.source, Source::Actor(Actor::Enemy(_))),
        }});
    } else {
        full.push(FullPowerEntry { id, state: PowerState { amount: value,
            skip_first: (matches!(id.0,"ritual"|"draw_reduction")
                && !matches!(effect.source,Source::Potion{id:crate::content::PotionId::CultistPotion,..}))
                ||id.0=="intangible"&&matches!(actor,Actor::Enemy(_)),
            monster_source_grace: matches!(id.0, "weak" | "vulnerable" | "frail")
                && matches!(effect.source, Source::Actor(Actor::Enemy(_))),
        }});
    }
    if value == 0 {
        state.pending.push_front(Effect { source: effect.source.clone(), target: effect.target.clone(),
            kind: EffectKind::RemoveFullPower(id) });
    }
}

/// The one source-qualified direct `AbstractCreature.addPower` adapter used by
/// already-bloodied Red Skull at combat start. Unlike ApplyPowerAction this
/// path does not consult Artifact; all threshold callbacks remain queued.
pub(super) fn add_direct_strength(state:&mut State,delta:i32)->Result<(),InputError>{
    let actor=Actor::Player;let id=FullPowerId("strength");
    let previous=amount(state,&actor,id)?;
    let next=previous.checked_add(delta).ok_or_else(||InputError::Invariant("direct Strength overflow".into()))?.clamp(-999,999);
    if let Some(power)=state.player_full_powers.iter_mut().find(|power|power.id==id){power.state.amount=next;}
    else if let Some(power)=state.player.powers.iter_mut().find(|power|power.id.as_str()==id.0){power.state.amount=next;}
    else{state.player_full_powers.push(FullPowerEntry{id,state:PowerState{amount:next,skip_first:false,monster_source_grace:false}});}
    if next==0{state.pending.push_front(Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind:EffectKind::RemoveFullPower(id)});}
    super::effects::refresh_enemy_values(state)
}

/// Owner-qualified player Thorns callback, shared by real incoming hit producers.
pub(super) fn player_thorns_after_damage(state:&mut State,result:&super::model::DamageResult){
    if result.target!=Actor::Player||result.thorns||result.hp_loss{return}
    let Source::Actor(Actor::Enemy(attacker))=&result.source else{return};
    let thorns=state.player_full_powers.iter().find(|p|p.id.0=="thorns").map_or(0,|p|p.state.amount);
    if thorns<=0||!state.enemies.iter().any(|e|&e.instance_id==attacker&&e.life==ActorLife::Active&&e.hp>0){return}
    // onAttacked is per qualifying hit, including zero/fully blocked NORMAL.
    // Retaliation is typed THORNS and cannot recursively trigger this power.
    state.pending.push_front(Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Enemy(attacker.clone())),kind:EffectKind::Damage(DamagePayload::Thorns(thorns))});
}

/// Only ordinary supported applications spend Artifact. Reductions/removals
/// reach this same owner without being reclassified as fresh applications.
pub(super) fn apply_effect(state: &mut State, effect: &Effect) -> Result<(), InputError> {
    let (id, mut delta, application) = match effect.kind {
        EffectKind::ApplyFullPower { id, amount } => (id, amount, true),
        EffectKind::ApplyPower { id, amount } => (FullPowerId(id.as_str()), amount, true),
        EffectKind::ReduceFullPower { id, amount } => (id, -amount, false),
        EffectKind::ReducePower { id, amount } => (FullPowerId(id.as_str()), -amount, false),
        EffectKind::RemoveFullPower(id) => {
            let target=effect.target.as_ref().unwrap();
            if matches!(target,Actor::Player)&&id.0=="combust"{state.combust=None;}
            let flight=id.0=="flight"&&match target{Actor::Enemy(owner)=>state.enemies.iter().any(|e|&e.instance_id==owner&&e.full_powers.iter().any(|p|p.id==id)),_=>false};
            let plated=id.0=="plated_armor"&&match target{Actor::Enemy(owner)=>state.enemies.iter().any(|enemy|&enemy.instance_id==owner&&enemy.full_powers.iter().any(|power|power.id==id)),_=>false};
            remove(state, effect.target.as_ref().unwrap(), id);
            super::cards::colorless_power_removed(state,target,id);
            if plated{if let Actor::Enemy(owner)=target{if state.enemies.iter().any(|enemy|&enemy.instance_id==owner&&enemy.enemy_id==ActorEnemyId::Full(crate::content::EnemyId::ShelledParasite)){
                let actor=Actor::Enemy(owner.clone());super::enemies::act2::bottom(state,Effect{source:Source::Actor(actor.clone()),target:Some(actor),kind:EffectKind::ShelledArmorBreak});
            }}}
            if flight{super::enemies::act2::flight::on_removed(state,target);}
            return super::effects::refresh_enemy_values(state);
        }
        EffectKind::RemovePower(id) => {
            let target=effect.target.as_ref().unwrap();
            if matches!(target,Actor::Player)&&id.as_str()=="combust"{state.combust=None;}
            let full_id=FullPowerId(id.as_str());
            remove(state, target, full_id);
            super::cards::colorless_power_removed(state,target,full_id);
            return super::effects::refresh_enemy_values(state);
        }
        _ => return Err(InputError::Invariant("power owner received non-power work".into())),
    };
    let actor = effect.target.as_ref().unwrap();
    let alive = match actor {
        Actor::Player => state.player.hp > 0,
        Actor::Enemy(owner) => state.enemies.iter().any(|e| &e.instance_id == owner
            && e.life == ActorLife::Active && e.hp > 0),
    };
    if application && !alive { return Ok(()); }
    let debuff = (matches!(id.0, "weak" | "vulnerable" | "frail" | "hex" | "confused" | "entangled" | "shackled" | "corpse_explosion" | "choked" | "constricted" | "poison" | "draw_reduction"|"no_draw"|"lose_strength"|"lose_dexterity"|"bias"|"lock_on") && delta > 0)
        ||(id.0=="focus"&&delta<0)
        ||(id.0=="wraith_form"&&delta<0) || (id.0=="strength"&&delta<=0) || (id.0=="dexterity"&&delta<0);
    let artifact = amount(state, actor, FullPowerId("artifact"))?;
    if application&&debuff&&artifact==0&&id.0!="shackled"&&matches!(actor,Actor::Enemy(_))
        &&matches!(effect.source,Source::Actor(Actor::Player)|Source::Card(_)|Source::Potion{..}) {
        let damage=state.player_full_powers.iter().find(|power|power.id.0=="sadistic_nature")
            .map_or(0,|power|power.state.amount);
        if damage>0{
            super::enemies::act2::bottom(state,Effect{source:Source::Actor(Actor::Player),target:Some(actor.clone()),
                kind:EffectKind::Damage(super::DamagePayload::Thorns(damage))});
        }
    }
    if application && debuff && artifact > 0 {
        set(state, effect, FullPowerId("artifact"), artifact - 1);
        return Ok(());
    }
    if application && id.0=="poison" && delta>0
        && state.item_context.as_ref().is_some_and(|items|
            items.owns_relic(crate::content::RelicId::SneckoSkull)) {
        delta=delta.checked_add(1).ok_or_else(||InputError::Unsupported(
            "Snecko Skull Poison amount overflow".into()))?;
    }
    if *actor==Actor::Player&&id.0=="focus" {
        super::characters::apply_focus(state,delta)?;
        return Ok(())
    }
    let previous = amount(state, actor, id)?;
    if !application && previous == 0 { return Ok(()); }
    let sum = previous.checked_add(delta).ok_or_else(|| InputError::Invariant("power amount overflow".into()))?;
    let next = match id.0 {
        "strength" => sum.clamp(-999, 999),
        // DoubleDamagePower constructs at one; ApplyPower's application
        // amount is the delta only when an existing matching power stacks.
        "double_damage" if application&&previous==0=>1,
        "energized"=>sum.clamp(0,999),
        "wraith_form"=>sum.min(0),
        "next_turn_block"|"draw_card_next_turn"|"blur"|"choked"|"infinite_blades"|"after_image"|"a_thousand_cuts"|"envenom"|"phantasmal"|"double_damage"|"intangible_player"|"tools_of_the_trade"|"burst"=>sum.max(0),
        "bias"|"loop"|"electrodynamics"|"static_discharge"|"storm"|"heatsinks"|"amplify"|"echo_form"|"lock_on"=>sum.max(0),
        "mode_shift"=>sum.max(0),
        "confused"|"entangled"=>i32::from(sum>0),
        "dexterity" if (-999..=999).contains(&sum) => sum,
        "dexterity" => return Err(InputError::Unsupported("Dexterity outside supported native surface".into())),
        // A new Poison object caps at 9999. Existing matching-ID stacks add
        // the raw application amount without that constructor clamp.
        "poison" if previous == 0 => sum.clamp(0, 9_999),
        "poison" => sum.max(0),
        _ if sum <= 999 => sum.max(0),
        _ => return Err(InputError::Unsupported("power outside supported native surface".into())),
    };
    // Combust's card source owns one typed copy per positive application. Its
    // 5/7 amount and copy count are preflighted together before either commits.
    let combust_update=if id.0=="combust" {
        if !matches!(actor,Actor::Player){return Err(InputError::Unsupported("Combust is player-owned only".into()))}
        if application&&delta>0 {
            let copies=state.combust.map_or(0,|power|power.copies).checked_add(1)
                .ok_or_else(||InputError::Unsupported("Combust copy count exhausted".into()))?;
            let count=i32::from(copies);let minimum=5*count;
            if next<minimum||next>7*count||(next-minimum)%2!=0 {
                return Err(InputError::Invariant("Combust amount/copy preflight is not a supported 5-or-7 stack sum".into()))
            }
            Some(Some(super::model::CombustState{copies}))
        }else if next==0 {
            Some(None)
        }else {
            let current=state.combust.ok_or_else(||InputError::Invariant("Combust reduction lacks typed copy custody".into()))?;
            let count=i32::from(current.copies);let minimum=5*count;
            if next<minimum||next>7*count||(next-minimum)%2!=0 {
                return Err(InputError::Unsupported("Combust reductions are not source-qualified".into()))
            }
            Some(Some(current))
        }
    }else{None};
    set(state, effect, id, next);
    super::cards::colorless_power_applied(state,actor,id,application,previous);
    if id.0 == "accuracy" && matches!(actor, Actor::Player) {
        super::cards::rewrite_accuracy_shivs(state, next);
    }
    if let Some(update)=combust_update{state.combust=update;}
    if id.0=="vulnerable"&&delta>0&&matches!(effect.source,Source::Actor(Actor::Player)|Source::Card(_))
        &&matches!(actor,Actor::Enemy(_))
        &&state.item_context.as_ref().is_some_and(|items|items.owns_relic(crate::content::RelicId::ChampionsBelt)) {
        let belt_child=Effect{source:effect.source.clone(),target:Some(actor.clone()),kind:EffectKind::ApplyFullPower{id:FullPowerId("weak"),amount:1}};
        let decision=state.pending.iter().position(|pending|pending.kind==EffectKind::Decision).unwrap_or(state.pending.len());
        state.pending.insert(decision,belt_child);
    }
    super::effects::refresh_enemy_values(state)
}
