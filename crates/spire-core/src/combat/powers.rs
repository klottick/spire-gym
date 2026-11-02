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
     "weak"|"vulnerable"|"frail"|"hex"|"confused"|"entangled"|"poison"|"draw_reduction"|"no_draw"|"lose_strength"|"lose_dexterity"|"wraith_form"|"choked"|"corpse_explosion"|"mark"|"block_return"|"energy_down"=>true,
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

/// The source's just-applied flag (`isSourceMonster` in Weak, Vulnerable and Frail): the new
/// debuff skips its first end-of-round decrement. Monsters pass it, and so do the cards that
/// play themselves at the end of the player's turn (Doubt, Shame), keyed on that card property.
fn just_applied(state: &State, effect: &Effect) -> bool {
    match &effect.source {
        Source::Actor(Actor::Enemy(_)) => true,
        Source::Card(handle) => state.piles.hand.iter().any(|owned| owned.handle() == *handle
            && owned.full().is_some_and(|live| crate::content::plays_itself_at_end_of_turn(live.id))),
        _ => false,
    }
}

fn set(state: &mut State, effect: &Effect, id: FullPowerId, value: i32) {
    let grace = just_applied(state, effect);
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
            monster_source_grace: legacy_id == PowerId::Vulnerable && grace,
        }});
    } else {
        full.push(FullPowerEntry { id, state: PowerState { amount: value,
            skip_first: (matches!(id.0,"ritual"|"draw_reduction")
                && !matches!(effect.source,Source::Potion{id:crate::content::PotionId::CultistPotion,..}))
                ||id.0=="intangible"&&matches!(actor,Actor::Enemy(_)),
            monster_source_grace: matches!(id.0, "weak" | "vulnerable" | "frail") && grace,
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

/// Character-neutral Block-gain listeners. The committed Block owner calls
/// this once for the incoming positive amount, so zero-gain paths stay quiet.
pub(super) fn player_block_gained(state:&mut State,target:&Actor,amount:i32){
    if !state.full_runtime||*target!=Actor::Player||amount<=0{return}
    let Some(weak)=state.player_full_powers.iter().find(|power|power.id.0=="wave_of_the_hand")
        .map(|power|power.state.amount).filter(|amount|*amount>0) else{return};
    let effects=state.enemies.iter().filter(|enemy|enemy.life==ActorLife::Active&&enemy.hp>0)
        .map(|enemy|Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Enemy(enemy.instance_id.clone())),
            kind:EffectKind::ApplyFullPower{id:FullPowerId("weak"),amount:weak}}).collect::<Vec<_>>();
    for effect in effects.into_iter().rev(){state.pending.push_front(effect);}
}

/// Block Return observes every committed NORMAL Attack hit, even when Block,
/// zero damage, or Invincible leaves no HP loss. Other damage types are excluded.
pub(super) fn enemy_block_return_after_damage(state:&mut State,result:&super::model::DamageResult){
    if !state.full_runtime||!result.normal{return}
    let (Actor::Enemy(owner),Source::Card(card_owner))=(&result.target,&result.source) else{return};
    let attack=state.in_play.as_ref().filter(|card|card.handle()==*card_owner)
        .and_then(super::model::OwnedCard::full).is_some_and(|card|
            crate::content::Catalog::standard().card(card.id)
                .is_some_and(|definition|definition.card_type==crate::content::CardType::Attack));
    if !attack{return}
    let Some(amount)=state.enemies.iter().find(|enemy|&enemy.instance_id==owner)
        .and_then(|enemy|enemy.full_powers.iter().find(|power|power.id.0=="block_return"))
        .map(|power|power.state.amount).filter(|amount|*amount>0) else{return};
    state.pending.push_front(Effect{source:result.source.clone(),target:Some(Actor::Player),kind:EffectKind::GainBlock(amount)});
}

/// Only ordinary supported applications spend Artifact. Reductions/removals
/// reach this same owner without being reclassified as fresh applications.
pub(super) fn apply_effect(state: &mut State, effect: &Effect) -> Result<(), InputError> {
    let application=match effect.kind {
        EffectKind::ApplyFullPower{id,amount}=>(id,amount),
        EffectKind::ApplyPower{id,amount}=>(FullPowerId(id.as_str()),amount),
        _=>return apply_effect_inner(state,effect),
    };
    let target=effect.target.as_ref().ok_or_else(||InputError::Invariant("power target absent at M27 phase".into()))?;
    let before=amount(state,target,application.0)?;
    apply_effect_inner(state,effect)?;
    let after=amount(state,target,application.0)?;
    let applied=after.saturating_sub(before);
    if state.full_runtime&&applied!=0 {
        // M27 seam: power-applied
        let work=if let Some(items)=state.item_context.as_mut(){
            let mut facts=crate::items::dispatch::PhaseFacts::new(None,state.player.hp,state.player.max_hp,state.player.block);
            facts.target=Some(target);facts.source=Some(&effect.source);facts.power=Some((application.0.0,applied));
            Some(crate::items::dispatch::combat_phase(items,crate::items::dispatch::ItemPhase::PowerApplied,&facts)?)
        }else{None};
        if let Some(work)=work {
            if work.direct_strength!=0{add_direct_strength(state,work.direct_strength)?;}
            match super::effects::queue_relic_work(state,work){
                Ok(())=>(),
                Err(super::EffectError::Invalid(error))=>return Err(error),
                Err(super::EffectError::Random(_))=>return Err(InputError::Invariant("relic phase validation unexpectedly requested randomness".into())),
            }
        }
    }
    Ok(())
}

fn apply_effect_inner(state: &mut State, effect: &Effect) -> Result<(), InputError> {
    let (id, mut delta, application) = match effect.kind {
        EffectKind::ApplyFullPower { id, amount } => (id, amount, true),
        EffectKind::ApplyPower { id, amount } => (FullPowerId(id.as_str()), amount, true),
        EffectKind::ReduceFullPower { id, amount } => (id, -amount, false),
        EffectKind::ReducePower { id, amount } => (FullPowerId(id.as_str()), -amount, false),
        EffectKind::RemoveFullPower(id) => {
            let target=effect.target.as_ref().unwrap();
            if matches!(target,Actor::Player)&&id.0=="combust"{state.combust=None;}
            if matches!(target,Actor::Player)&&id.0=="deva_form"{state.watcher.deva_energy_gain=0;}
            let flight=id.0=="flight"&&match target{Actor::Enemy(owner)=>state.enemies.iter().any(|e|&e.instance_id==owner&&e.full_powers.iter().any(|p|p.id==id)),_=>false};
            let plated=id.0=="plated_armor"&&match target{Actor::Enemy(owner)=>state.enemies.iter().any(|enemy|&enemy.instance_id==owner&&enemy.full_powers.iter().any(|power|power.id==id)),_=>false};
            if *target==Actor::Player&&id.0=="draw"{state.player.draw_per_turn=state.player.draw_per_turn.wrapping_sub(amount(state,target,id)?);}
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
            if matches!(target,Actor::Player)&&id.as_str()=="deva_form"{state.watcher.deva_energy_gain=0;}
            let full_id=FullPowerId(id.as_str());
            remove(state, target, full_id);
            super::cards::colorless_power_removed(state,target,full_id);
            return super::effects::refresh_enemy_values(state);
        }
        _ => return Err(InputError::Invariant("power owner received non-power work".into())),
    };
    let actor = effect.target.as_ref().unwrap();
    // ApplyPowerAction ends at once when its target already has No Draw: no stack,
    // no Artifact, no apply-time callbacks (source-check-nodraw-slime-v1).
    if application && id.0=="no_draw" && amount(state, actor, id)? != 0 { return Ok(()); }
    let alive = match actor {
        Actor::Player => state.player.hp > 0,
        Actor::Enemy(owner) => state.enemies.iter().any(|e| &e.instance_id == owner
            && e.life == ActorLife::Active && e.hp > 0),
    };
    if application && !alive { return Ok(()); }
    let debuff = (matches!(id.0, "weak" | "vulnerable" | "frail" | "hex" | "confused" | "entangled" | "shackled" | "corpse_explosion" | "choked" | "constricted" | "poison" | "draw_reduction"|"no_draw"|"lose_strength"|"lose_dexterity"|"bias"|"lock_on"|"mark"|"block_return"|"energy_down") && delta > 0)
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
    if application && state.full_runtime {
        if let Some(items) = state.item_context.as_ref() {
            // M27 seam: power-attempt follows source callbacks and gates before Artifact.
            if !crate::items::laws::power_attempt(items, actor, id.0, delta) {
                return Ok(());
            }
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
    let deva_gain_update=if application&&*actor==Actor::Player&&id.0=="deva_form"{
        if !super::cards::watcher::cards_active(state)
            ||!state.item_context.as_ref().is_some_and(|items|items.m25_cards())
            ||delta<=0{return Err(InputError::Invariant("Deva Form application lacks its M25 owner or positive stack".into()))}
        let current=state.watcher.deva_energy_gain;
        if previous==0{
            if current!=0{return Err(InputError::Invariant("new Deva Form has stale recharge state".into()))}
            Some(1)
        }else{
            if current<previous{return Err(InputError::Invariant("Deva Form recharge state is below its growth step".into()))}
            Some(current.checked_add(1).ok_or_else(||InputError::Unsupported("Deva Form next energy gain exhausted".into()))?)
        }
    }else{None};
    if *actor==Actor::Player&&id.0=="master_reality"&&super::cards::watcher::cards_active(state){
        if application&&previous==0{set(state,effect,id,-1);}
        return Ok(())
    }
    if application&&*actor==Actor::Player&&id.0=="mantra"&&super::cards::watcher::cards_active(state){
        let existed=state.player_full_powers.iter().any(|p|p.id==id);
        let mut next=if existed&&previous!=-1{previous.checked_add(delta).ok_or_else(||InputError::Unsupported("Watcher Mantra amount int32 exhausted".into()))?}else{delta};
        let threshold=existed&&next>=10;
        if threshold{next=next.wrapping_sub(10);}
        set(state,effect,id,next);
        if threshold{super::enemies::act2::bottom(state,Effect{source:effect.source.clone(),target:Some(Actor::Player),kind:EffectKind::Character(super::CharacterEffect::Stance(super::Stance::Divinity))});}
        return super::effects::refresh_enemy_values(state);
    }

    if application&&id.0=="rebound"&&previous==0&&*actor==Actor::Player{state.defect.rebound_just_applied=true;}
    if !application && previous == 0 { return Ok(()); }
    let sum = previous.checked_add(delta).ok_or_else(|| InputError::Invariant("power amount overflow".into()))?;
    let next = match id.0 {
        "like_water" if super::cards::watcher::cards_active(state)=>sum.clamp(0,999),
        _ if super::cards::watcher::cards_active(state)&&super::cards::watcher::power_id(id)=>sum.max(0),
        "strength" => sum.clamp(-999, 999),
        // DoubleDamagePower constructs at one; ApplyPower's application
        // amount is the delta only when an existing matching power stacks.
        "double_damage" if application&&previous==0=>1,
        "energized"|"energized_blue"=>sum.clamp(0,999),
        "wraith_form"=>sum.min(0),
        "next_turn_block"|"draw_card_next_turn"|"blur"|"choked"|"infinite_blades"|"after_image"|"a_thousand_cuts"|"envenom"|"phantasmal"|"double_damage"|"intangible_player"|"tools_of_the_trade"|"burst"=>sum.max(0),
        "buffer"|"repair"|"draw"|"hello_world"|"creative_ai"|"equilibrium"|"rebound"|"bias"|"loop"|"electrodynamics"|"static_discharge"|"storm"|"heatsinks"|"amplify"|"echo_form"|"lock_on"=>sum.max(0),
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
    if let Some(next_gain)=deva_gain_update{state.watcher.deva_energy_gain=next_gain;}
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
