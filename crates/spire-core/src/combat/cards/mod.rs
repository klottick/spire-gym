//! Native handlers for the card families implemented in this module.
use super::*;
use super::model::State;
pub(crate) mod ironclad;
pub use ironclad::IroncladEffect;
mod shared;
pub(crate) mod watcher;
mod silent;
mod colorless;
pub(crate) mod defect;
pub(crate) mod curses;
pub use defect::{DefectEffect,DefectState};
pub use silent::{SilentEffect, SilentState, DrawnCard};
pub use colorless::ColorlessEffect;
pub(super) fn requires_draw_history(id:crate::content::CardId)->bool{crate::content::defect::supports_m20_generated(id)||silent::requires_draw_history(id)||colorless::requires_draw_history(id)||super::generation::starting_cost(id,0).is_some()}
pub(super) fn apply_silent(state:&mut State,effect:&Effect,work:&SilentEffect)->Result<(),EffectError>{silent::apply(state,effect,work)}
pub(super) fn validate_silent_work(state:&State,effect:&Effect,work:&SilentEffect)->Result<(),InputError>{silent::validate_work(state,effect,work)}
pub(super) fn apply_colorless(state:&mut State,effect:&Effect,work:&ColorlessEffect)->Result<(),EffectError>{colorless::apply(state,effect,work)}
pub(super) fn validate_colorless_work(state:&State,effect:&Effect,work:&ColorlessEffect)->Result<(),InputError>{colorless::validate_work(state,effect,work)}
pub(super) fn capture_colorless_draw_value(card:&mut LiveCard,draw_count:usize){colorless::capture_draw_value(card,draw_count)}
pub(super) fn increment_discard(state:&mut State,notify_cards:bool)->Result<(),InputError>{silent::increment_discard(state,notify_cards)}
pub(super) fn actual_draw(state:&mut State,owner:&super::CardHandle)->Result<(),EffectError>{
    silent::actual_draw(state,owner)?;
    watcher::actual_draw(state,owner)
}
pub(super) fn starting_cost(id:crate::content::CardId,upgrades:u32)->Option<i32>{watcher::starting_cost(id,upgrades).or_else(||ironclad::starting_cost(id)).or_else(||defect::starting_cost(id,upgrades).or_else(||super::generation::starting_cost(id,upgrades)).or_else(||colorless::starting_cost(id,upgrades)).or_else(||silent::starting_cost(id,upgrades)))}
pub(super) fn silent_player_hp_lost(state:&mut State)->Result<(),EffectError>{Ok(silent::player_hp_lost(state)?)}

#[derive(Clone,Copy,Debug)]
pub(super) struct CopyHistory { pub full_runtime:bool, pub discarded_this_turn:u32 }

pub(super) fn copy_history(state:&State)->CopyHistory{
    CopyHistory{full_runtime:state.full_runtime,discarded_this_turn:state.silent.discarded_this_turn}
}

pub(super) fn make_stat_equivalent_copy(source:&mut super::LiveCard,instance:super::CardInstanceId,history:CopyHistory)
    ->Result<super::LiveCard,InputError>{
    silent::before_make_copy(source,history)?;
    Ok(source.stat_equivalent_copy(instance))
}

pub(super) fn make_same_instance_family_copy(source:&mut super::LiveCard,instance:super::CardInstanceId,history:CopyHistory)
    ->Result<super::LiveCard,InputError>{
    let family=source.copy_family;
    let mut copy=make_stat_equivalent_copy(source,instance,history)?;
    copy.copy_family=family;
    copy.copy_provenance=super::card_state::CopyProvenance::SameInstanceFamily;
    Ok(copy)
}

pub(super) fn apply_setup_move(state:&mut State,owned:super::model::OwnedCard,handle:&super::CardHandle)->Result<(),InputError>{
    silent::setup_move_card(state,owned,handle)
}

pub(super) fn apply_forethought_free_play(state:&State,card:&mut LiveCard){
    if !card.local.x_cost&&card.local.mutable_combat_cost.or(card.local.randomized_cost).or_else(||base_cost(state,card)).is_some_and(|cost|cost>0){
        card.local.free_to_play_once=true;
    }
}

pub(super) fn on_card_exhausted(_state:&State,source:&super::Source,card:&super::LiveCard)
    ->Result<Vec<super::Effect>,super::EffectError>{
    match card.id{
        crate::content::CardId::Sentinel=>Ok(vec![ironclad::exhaust::sentinel_trigger(source,card)]),
        _=>Ok(Vec::new()),
    }
}

pub(super) fn apply_silent_catalyst(state:&mut State, effect:&super::Effect)->Result<(),super::EffectError>{
    silent::apply_catalyst(state,effect)
}

pub(super) fn manual_discard(state: &mut State, card: &super::LiveCard) -> Result<bool, crate::combat::InputError> {
    silent::manual_discard(state, card)
}

pub(super) fn rewrite_accuracy_shivs(state: &mut State, amount: i32) {
    silent::rewrite_accuracy_shivs(state, amount);
}

pub(super) fn rewrite_accuracy_hand_shivs(state: &mut State, amount: i32) {
    silent::rewrite_accuracy_hand_shivs(state, amount);
}

pub(super) fn apply_silent_retain_selection(state:&mut State,effect:&super::Effect,limit:i32)
    ->Result<(),super::EffectError>{
    silent::apply_retain_selection(state,effect,limit)
}

pub(super) fn capture_silent_nightmare_template(state:&mut State,owner:super::CardHandle,
    source:&super::LiveCard)->Result<(),super::InputError>{silent::capture_nightmare_template(state,owner,source)}
pub(super) fn capture_silent_nightmare_template_from_owner(state:&mut State,owner:super::CardHandle,
    source:&super::CardHandle)->Result<(),super::InputError>{silent::capture_nightmare_template_from_owner(state,owner,source)}

pub(super) fn silent_nightmare_pre_draw(state:&State,k:usize)->Option<super::Effect>{silent::nightmare_pre_draw(state,k)}

pub(super) fn apply_silent_nightmare_copy(state:&mut State,effect:&super::Effect,owner:&super::CardHandle,installation_id:u64)
    ->Result<(),super::EffectError>{silent::apply_nightmare_copy(state,effect,owner,installation_id)}

pub(super) fn validate_silent_nightmare_templates(state:&State)->Result<(),super::InputError>{
    silent::validate_nightmare_templates(state)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DefectContinuation {
    GoForTheEyesWeak { amount: i32 },
    BarrageStart { damage: i32 },
    BarrageNext { damage: i32, remaining: u8 },
    CompileDriverDraw,
}

pub(super) fn apply_defect_continuation(state: &mut State, effect: &Effect,
    continuation: &DefectContinuation) -> Result<(), crate::combat::EffectError> {
    let target = effect.target.as_ref();
    let source = effect.source.clone();
    match continuation {
        DefectContinuation::GoForTheEyesWeak { amount } => {
            let Some(Actor::Enemy(id)) = target else { return Err(crate::combat::InputError::Invariant(
                "Go for the Eyes continuation target missing".into()).into()) };
            let enemy = state.enemies.iter().find(|enemy| &enemy.instance_id == id)
                .ok_or_else(|| crate::combat::InputError::Invariant("Go for the Eyes target disappeared".into()))?;
            let intent_base = match &enemy.intent_id {
                ActorIntentId::Legacy(intent) => enemy.damage_entries.get(intent).map(|entry| entry.base),
                ActorIntentId::Full(intent) => enemy.full_damage_entries.get(intent).map(|entry| entry.base),
            };
            if intent_base.is_some_and(|damage| damage >= 0) {
                state.pending.push_front(Effect { source, target: Some(Actor::Enemy(id.clone())),
                    kind: EffectKind::ApplyFullPower { id: FullPowerId("weak"), amount: *amount } });
            }
        }
        DefectContinuation::BarrageStart { damage } => {
            let Some(Actor::Enemy(id)) = target else { return Err(crate::combat::InputError::Invariant(
                "Barrage continuation target missing".into()).into()) };
            let count = state.character.orb_slots.iter().flatten().count();
            if count > 0 && state.enemies.iter().any(|enemy|
                &enemy.instance_id == id && enemy.life == ActorLife::Active && enemy.hp > 0) {
                if count > 1 {
                    state.pending.push_front(Effect { source: source.clone(), target: Some(Actor::Enemy(id.clone())),
                        kind: EffectKind::CardContinuation(DefectContinuation::BarrageNext {
                            damage: *damage, remaining: (count - 1) as u8 }) });
                }
                state.pending.push_front(Effect { source, target: Some(Actor::Enemy(id.clone())),
                    kind: EffectKind::Damage(DamagePayload::Fixed(*damage)) });
            }
        }
        DefectContinuation::BarrageNext { damage, remaining } => {
            let Some(Actor::Enemy(id)) = target else { return Err(crate::combat::InputError::Invariant(
                "Barrage continuation target missing".into()).into()) };
            if state.enemies.iter().any(|enemy|
                &enemy.instance_id == id && enemy.life == ActorLife::Active && enemy.hp > 0) {
                if *remaining > 1 {
                    state.pending.push_front(Effect { source: source.clone(), target: Some(Actor::Enemy(id.clone())),
                        kind: EffectKind::CardContinuation(DefectContinuation::BarrageNext {
                            damage: *damage, remaining: remaining - 1 }) });
                }
                state.pending.push_front(Effect { source, target: Some(Actor::Enemy(id.clone())),
                    kind: EffectKind::Damage(DamagePayload::Fixed(*damage)) });
            }
        }
        DefectContinuation::CompileDriverDraw => {
            let mut kinds = Vec::new();
            for orb in state.character.orb_slots.iter().flatten() {
                if !kinds.contains(&orb.kind) { kinds.push(orb.kind); }
            }
            let kinds = kinds.len() as i32;
            if kinds > 0 {
                state.pending.push_front(Effect { source, target: None, kind: EffectKind::Draw(kinds) });
            }
        }
    }
    Ok(())
}

pub(super) struct PreparedCard {
    pub hand_index: usize,
    pub instance: CardInstanceId,
    pub captured_energy:i32,
    pub paid:i32,
    pub remaining_energy: i32,
    pub target: Option<Actor>,
    pub effects: Vec<Effect>,
    pub exhaust_on_finish: bool,
}

/// Validate a full-card play and construct its ordered native effects without
/// mutating combat state. The engine owns the single commit and resolver drain.
pub(super) fn prepare(combat: &Combat, action: &CombatAction) -> Result<PreparedCard, crate::run::RunError> {
    let CombatAction::PlayCard { instance, target } = action else {
        return Err(crate::run::RunError::InvalidAction);
    };
    let handle = CardHandle::parse(&instance.to_string())
        .map_err(|_| crate::run::RunError::InvalidAction)?;
    if matches!(target, Some(TargetId(0))) { return Err(crate::run::RunError::InvalidAction); }
    let (hand_index, owned) = combat.state.piles.hand.iter().enumerate()
        .find(|(_, card)| card.handle() == handle)
        .ok_or(crate::run::RunError::InvalidAction)?;
    let card = owned.full().ok_or(crate::run::RunError::InvalidAction)?;
    let exhaust_on_finish=validate_card_use(&combat.state,card,*target)?;
    let captured_energy=combat.state.player.energy;
    let (cost, payloads) = if exhaust_on_finish { (0,Vec::new()) }
        else { payload_with_wrist_blade(&combat.state, card, target.map(|id| id.0))? };
    let attack=crate::content::Catalog::standard().card(card.id).is_some_and(|definition|definition.card_type==crate::content::CardType::Attack);
    // Free Attack (Swivel) makes every Attack free while a positive count remains; the count
    // is spent by the shared CardPlayed owner for any non-purge Attack use (watcher.md:123).
    let free_attack=attack&&!card.local.free_play&&!card.local.free_to_play_once
        &&combat.state.player_full_powers.iter().any(|power|power.id.0=="free_attack"&&power.state.amount>0);
    let mut payment_card=card.clone();
    if free_attack{payment_card.local.free_to_play_once=true;}
    let corruption_skill=ironclad::exhaust::corruption_skill_active(&combat.state)
        &&crate::content::Catalog::standard().card(card.id).is_some_and(|definition|definition.card_type==crate::content::CardType::Skill);
    let payment = if free_attack{
        super::card_state::payment(&payment_card,captured_energy,cost)
    }else if matches!(card.id,crate::content::CardId::Whirlwind|crate::content::CardId::ReinforcedBody|crate::content::CardId::Tempest|crate::content::CardId::MultiCast)&&card.local.x_cost&&!card.local.free_play {
        Ok(super::card_state::Payment{remaining_energy:captured_energy,paid:0})
    }else if corruption_skill&&!card.local.free_play&&!card.local.x_cost{
        super::card_state::effective_cost(card,cost).map_err(|_|crate::run::RunError::InvalidAction)?;
        Ok(super::card_state::Payment{remaining_energy:captured_energy,paid:0})
    }else{super::card_state::payment(card,captured_energy,cost)}
        .map_err(|_| crate::run::RunError::InvalidAction)?;
    let actor = match target {
        Some(TargetId(id)) if *id > 0 => Some(combat.state.enemies.get((*id - 1) as usize)
            .filter(|enemy| enemy.hp > 0).map(|enemy| Actor::Enemy(enemy.instance_id.clone()))
            .ok_or(crate::run::RunError::InvalidAction)?),
        Some(_) => return Err(crate::run::RunError::InvalidAction),
        None => None,
    };
    let source = Source::Card(handle);
    let mut effects = payloads.into_iter().map(|(target, kind)| Effect {
        source: source.clone(), target: target.map(|target| match target {
            PayloadTarget::Player => Actor::Player,
            PayloadTarget::SelectedEnemy => actor.clone().expect("enemy target validated"),
            PayloadTarget::Enemy(id) => Actor::Enemy(id),
        }), kind,
    }).collect::<Vec<_>>();
    Ok(PreparedCard { hand_index, instance: card.instance,captured_energy,paid:payment.paid,
        remaining_energy: payment.remaining_energy, target: actor, effects, exhaust_on_finish })
}

/// Catalog-level Unplayable kinds. This list holds even for a card built
/// without public values; the cost metadata adds any data-driven kind.
fn catalog_unplayable(id:crate::content::CardId)->bool{
    use crate::content::CardId::*;
    matches!(id,Dazed|Burn|Wound|Parasite|Void|Regret|Reflex|Tactician|Clumsy|CurseOfTheBell|Decay|Doubt|Injury|Necronomicurse
        |Normality|Pain|Shame|Writhe|AscendersBane
        // Deus Ex Machina is Unplayable; only its on-draw trigger acts, so one that reaches the hand
        // without a draw (a Stasis return, Discovery, copies) is unplayable, never Unsupported.
        |DeusExMachina)
}

pub(super) fn unplayable_card(card:&LiveCard)->bool{
    catalog_unplayable(card.id)||public_values(card).is_some_and(|values|values.cost_kind==CardCostKind::Unplayable)
}

pub(super) fn suppress_autoplay_card(card:&LiveCard)->bool{
    let card_type=crate::content::Catalog::standard().card(card.id).map(|definition|definition.card_type);
    card_type.is_some_and(|kind|matches!(kind,crate::content::CardType::Status|crate::content::CardType::Curse))
        ||unplayable_card(card)
}

/// Shared accepted-use legality used by both ordinary hand plays and autoplay
/// children. Payload production remains read-only; payment is a separate owner.
pub(super) fn validate_card_use(state:&State,card:&LiveCard,target:Option<TargetId>)->Result<bool,crate::run::RunError>{
    let paid=state.play_context.as_ref().filter(|context|context.card==card_handle(card)).map_or(0,|context|context.paid);
    let rule=state.item_context.as_ref().and_then(|context|{
        let definition=crate::content::Catalog::standard().card(card.id)?;
        // M27 seam: play-allowed; this validator also drives the legal-action mask.
        Some(crate::items::laws::play_allowed(context,crate::items::dispatch::PhaseCard{
            id:card.id,card_type:definition.card_type,cost_paid:paid}))
    });
    if rule.is_some_and(|rule|rule.allow==Some(false)){return Err(crate::run::RunError::InvalidAction)}
    if curses::play_blocked(state){return Err(crate::run::RunError::InvalidAction)}
    if card.id==crate::content::CardId::SignatureMove {
        let owner=card_handle(card);
        if state.piles.hand.iter().any(|owned|owned.handle()!=owner&&owned.full().is_some_and(|live|
            crate::content::Catalog::standard().card(live.id).is_some_and(|definition|definition.card_type==crate::content::CardType::Attack))){
            return Err(crate::run::RunError::InvalidAction)
        }
    }
    if state.player_full_powers.iter().any(|p|p.id.0=="entangled"&&p.state.amount>0)
        &&crate::content::Catalog::standard().card(card.id).is_some_and(|card|card.card_type==crate::content::CardType::Attack){return Err(crate::run::RunError::InvalidAction)}
    let card_type=crate::content::Catalog::standard().card(card.id).map(|definition|definition.card_type);
    let unplayable=unplayable_card(card);
    let exhaust_unplayable=unplayable
        && card_type.is_some_and(|kind|matches!(kind,crate::content::CardType::Curse|crate::content::CardType::Status))
        && state.item_context.as_ref().is_some_and(crate::items::CombatItemContext::complete_items)
        && rule.is_some_and(|rule|rule.allow==Some(true));
    if unplayable&&!exhaust_unplayable{
        return Err(crate::run::RunError::InvalidAction);
    }
    if matches!(card.id, crate::content::CardId::Defend | crate::content::CardId::DefendGreen
        | crate::content::CardId::DefendBlue | crate::content::CardId::DefendPurple
        | crate::content::CardId::Survivor | crate::content::CardId::Zap
        | crate::content::CardId::Dualcast | crate::content::CardId::Vigilance | crate::content::CardId::WellLaidPlans)
        && target.is_some() {
        return Err(crate::run::RunError::InvalidAction);
    }
    if matches!(card.id, crate::content::CardId::Cleave | crate::content::CardId::Thunderclap
        | crate::content::CardId::ShrugItOff | crate::content::CardId::Miracle | crate::content::CardId::Slimed
        | crate::content::CardId::Backflip | crate::content::CardId::Deflect
        | crate::content::CardId::CloakAndDagger | crate::content::CardId::Consecrate)
        && target.is_some() {
        return Err(crate::run::RunError::InvalidAction);
    }
    if matches!(card.id, crate::content::CardId::Coolheaded | crate::content::CardId::Leap
        | crate::content::CardId::EmptyBody | crate::content::CardId::EmptyMind)
        && target.is_some() {
        return Err(crate::run::RunError::InvalidAction);
    }
    if matches!(card.id, crate::content::CardId::SweepingBeam) && target.is_some() {
        return Err(crate::run::RunError::InvalidAction);
    }
    Ok(exhaust_unplayable)
}

/// Public card adapter. Native play validation and commit remain in
/// `Combat::step_full`; that path delegates here to `prepare` exactly once.
pub fn resolve(combat: &mut Combat, action: &CombatAction) -> Result<(), crate::run::RunError> {
    combat.step_full(action)
}

#[derive(Clone)]
pub(in crate::combat) enum PayloadTarget { Player, SelectedEnemy, Enemy(EnemyHandle) }

/// M26 colour dispatcher (design §5.2): under the complete capability a card is
/// initialized by its own colour's existing owner, whatever the player's character.
/// Watcher kinds (purple roster and Watcher children) use the M25 Watcher owner, blue
/// cards the Defect metadata; Red, Green, Colorless, curses and statuses keep the shared
/// constructor defaults. Choice-only options never become physical cards.
pub(crate) fn initialize_complete(card:&mut LiveCard)->Result<(),InputError>{
    if !crate::content::complete::supports_generated(card.id)||!crate::run::complete_upgrades_valid(card.id,card.upgrades){
        return Err(InputError::Unsupported("card outside the complete card union".into()))
    }
    if crate::content::watcher::supports_m25_generated(card.id){return watcher::initialize_m25_base_profile(card)}
    if crate::content::Catalog::standard().card(card.id).is_some_and(|row|row.color==crate::content::CardColor::Blue){defect::initialize(card);}
    Ok(())
}

/// Obtain price from the same read-only payload producer as actual play, not a second table.
pub(super) fn base_cost(state:&State,card:&LiveCard)->Option<i32>{
    if let Some(cost)=starting_cost(card.id,card.upgrades){return Some(cost)}
    if matches!(card.id,crate::content::CardId::Dazed|crate::content::CardId::Burn|crate::content::CardId::Wound|crate::content::CardId::Parasite|crate::content::CardId::Void|crate::content::CardId::Regret|crate::content::CardId::Clumsy|crate::content::CardId::CurseOfTheBell|crate::content::CardId::Decay|crate::content::CardId::Doubt|crate::content::CardId::Injury|crate::content::CardId::Necronomicurse|crate::content::CardId::Normality|crate::content::CardId::Pain|crate::content::CardId::Shame|crate::content::CardId::Writhe|crate::content::CardId::AscendersBane){return None}
    payload(state,card,None).ok().or_else(||state.enemies.iter().enumerate().find_map(|(i,_)|payload(state,card,Some(i as u64+1)).ok())).map(|(cost,_)|cost)
}
pub(super) fn visible_cost(state:&State,card:&LiveCard)->Option<i32>{
    if card.local.x_cost{
        if watcher::cards_active(state)&&crate::content::Catalog::standard().card(card.id).is_some_and(|definition|definition.card_type==crate::content::CardType::Attack)
            &&state.player_full_powers.iter().any(|power|power.id.0=="free_attack"&&power.state.amount>0){return Some(0)}
        return None
    }
    let cost=super::card_state::effective_cost(card,base_cost(state,card)?).ok()?;
    if watcher::cards_active(state)&&!card.local.free_play&&!card.local.free_to_play_once
        &&crate::content::Catalog::standard().card(card.id).is_some_and(|definition|definition.card_type==crate::content::CardType::Attack)
        &&cost>0&&state.player_full_powers.iter().any(|power|power.id.0=="free_attack"&&power.state.amount>0){return Some(0)}
    if ironclad::exhaust::corruption_skill_active(state)&&crate::content::Catalog::standard().card(card.id)
        .is_some_and(|definition|definition.card_type==crate::content::CardType::Skill)&&!card.local.free_play{return Some(0)}
    Some(cost)
}

fn payload(state: &State, card: &LiveCard, target: Option<u64>)
    -> Result<(i32, Vec<(Option<PayloadTarget>, EffectKind)>), crate::run::RunError>
{
    if let Some(result) = watcher::payload(state,card,target)? {return Ok(result)}
    if let Some(result) = defect::payload(state,card,target)? { return Ok(result); }
    if let Some(result) = super::generation::payload(state,card,target)? { return Ok(result); }
    if let Some(result) = colorless::payload(state, card, target)? { return Ok(result); }
    if let Some(result) = ironclad::payload(state, card, target)? { return Ok(result); }
    if let Some(result) = shared::payload(card)? { return Ok(result); }
    if let Some(result) = silent::payload(state, card, target)? { return Ok(result); }
    use crate::content::CardId::*;
    let upgraded = card.upgrades > 0;
    let (cost, effects) = match card.id {
        IronWave => {
            if target.is_none() { return Err(crate::run::RunError::InvalidAction); }
            let block = modified_block(state, if upgraded { 7 } else { 5 })?;
            let damage = modified_damage(state, target.unwrap(), if upgraded { 7 } else { 5 })?;
            (1, vec![ (Some(PayloadTarget::Player), EffectKind::GainBlock(block)),
                (Some(PayloadTarget::SelectedEnemy), EffectKind::Damage(DamagePayload::Fixed(damage))) ])
        }
        Strike | StrikeGreen | StrikeBlue | StrikePurple => {
            if target.is_none() { return Err(crate::run::RunError::InvalidAction); }
            let damage = modified_damage(state, target.unwrap(), if upgraded { 9 } else { 6 })?;
            (1, vec![(Some(PayloadTarget::SelectedEnemy), EffectKind::Damage(DamagePayload::Fixed(damage)))])
        }
        Defend | DefendGreen | DefendBlue | DefendPurple => {
            (1, vec![(Some(PayloadTarget::Player), EffectKind::GainBlock(modified_block(state, if upgraded { 8 } else { 5 })?))])
        }
        Neutralize => {
            if target.is_none() { return Err(crate::run::RunError::InvalidAction); }
            let damage = modified_damage(state, target.unwrap(), if upgraded { 4 } else { 3 })?;
            (0, vec![
                (Some(PayloadTarget::SelectedEnemy), EffectKind::Damage(DamagePayload::Fixed(damage))),
                (Some(PayloadTarget::SelectedEnemy), EffectKind::ApplyFullPower { id: FullPowerId("weak"), amount: if upgraded { 2 } else { 1 } }),
            ])
        }
        Survivor => {
            let candidates = state.piles.hand.iter().filter(|owned| owned.handle() != card_handle(card))
                .map(|owned| owned.handle()).collect::<Vec<_>>();
            (1, vec![
                (Some(PayloadTarget::Player), EffectKind::GainBlock(modified_block(state, if upgraded { 11 } else { 8 })?)),
                (None, EffectKind::SelectCards { candidates, required: 1,
                    disposition: super::SelectionDisposition::Discard,
                    empty: super::EmptySelection::Continue }),
            ])
        }
        Zap => (if upgraded { 0 } else { 1 }, vec![
            (Some(PayloadTarget::Player), EffectKind::Character(super::characters::CharacterEffect::Channel(super::characters::OrbKind::Lightning)))
        ]),
        BallLightning | ColdSnap => {
            let Some(target_id) = target else { return Err(crate::run::RunError::InvalidAction) };
            let damage = modified_damage(state, target_id, match card.id {
                BallLightning => if upgraded { 10 } else { 7 },
                ColdSnap => if upgraded { 9 } else { 6 },
                _ => unreachable!(),
            })?;
            let orb = if card.id == BallLightning {
                super::characters::OrbKind::Lightning
            } else { super::characters::OrbKind::Frost };
            (1, vec![
                (Some(PayloadTarget::SelectedEnemy), EffectKind::Damage(DamagePayload::Fixed(damage))),
                (Some(PayloadTarget::Player), EffectKind::Character(super::characters::CharacterEffect::Channel(orb))),
            ])
        }
        BeamCell | GoForTheEyes => {
            let Some(target_id) = target else { return Err(crate::run::RunError::InvalidAction) };
            let damage = modified_damage(state, target_id, if upgraded { 4 } else { 3 })?;
            let mut effects = vec![(Some(PayloadTarget::SelectedEnemy), EffectKind::Damage(
                DamagePayload::Fixed(damage)))];
            if card.id == BeamCell {
                effects.push((Some(PayloadTarget::SelectedEnemy), EffectKind::ApplyFullPower {
                    id: FullPowerId("vulnerable"), amount: if upgraded { 2 } else { 1 } }));
            } else {
                effects.push((Some(PayloadTarget::SelectedEnemy), EffectKind::CardContinuation(
                    DefectContinuation::GoForTheEyesWeak { amount: if upgraded { 2 } else { 1 } })));
            }
            (0, effects)
        }
        Barrage => {
            let Some(target_id) = target else { return Err(crate::run::RunError::InvalidAction) };
            let damage = modified_damage(state, target_id, if upgraded { 6 } else { 4 })?;
            (1, vec![(Some(PayloadTarget::SelectedEnemy), EffectKind::CardContinuation(
                DefectContinuation::BarrageStart { damage }))])
        }
        CompileDriver => {
            let Some(target_id) = target else { return Err(crate::run::RunError::InvalidAction) };
            let damage = modified_damage(state, target_id, if upgraded { 10 } else { 7 })?;
            (1, vec![
                (Some(PayloadTarget::SelectedEnemy), EffectKind::Damage(DamagePayload::Fixed(damage))),
                (None, EffectKind::CardContinuation(DefectContinuation::CompileDriverDraw)),
            ])
        }
        SweepingBeam => {
            let mut effects = Vec::new();
            for (index, enemy) in state.enemies.iter().enumerate()
                .filter(|(_, enemy)| enemy.life == ActorLife::Active && enemy.hp > 0) {
                let damage = modified_damage(state, index as u64 + 1, if upgraded { 9 } else { 6 })?;
                effects.push((Some(PayloadTarget::Enemy(enemy.instance_id.clone())),
                    EffectKind::Damage(DamagePayload::Fixed(damage))));
            }
            effects.push((None, EffectKind::Draw(1)));
            (1, effects)
        }
        Coolheaded => {
            (1, vec![
                (Some(PayloadTarget::Player), EffectKind::Character(super::characters::CharacterEffect::Channel(super::characters::OrbKind::Frost))),
                (None, EffectKind::Draw(if upgraded { 2 } else { 1 })),
            ])
        }
        Leap => (1, vec![(Some(PayloadTarget::Player), EffectKind::GainBlock(modified_block(state, if upgraded { 12 } else { 9 })?))]),
        Dualcast => (if upgraded { 0 } else { 1 }, vec![
            (Some(PayloadTarget::Player), EffectKind::Character(super::characters::CharacterEffect::EvokeFront { remove: false })),
            (Some(PayloadTarget::Player), EffectKind::Character(super::characters::CharacterEffect::EvokeFront { remove: true })),
        ]),
        Eruption => {
            if target.is_none() { return Err(crate::run::RunError::InvalidAction); }
            let damage = modified_damage(state, target.unwrap(), 9)?;
            (if upgraded { 1 } else { 2 }, vec![
                (Some(PayloadTarget::SelectedEnemy), EffectKind::Damage(DamagePayload::Fixed(damage))),
                (Some(PayloadTarget::Player), EffectKind::Character(super::characters::CharacterEffect::Stance(super::characters::Stance::Wrath))),
            ])
        }
        EmptyFist => {
            let Some(target_id) = target else { return Err(crate::run::RunError::InvalidAction) };
            let damage = modified_damage(state, target_id, if upgraded { 14 } else { 9 })?;
            (1, vec![
                (Some(PayloadTarget::SelectedEnemy), EffectKind::Damage(DamagePayload::Fixed(damage))),
                (Some(PayloadTarget::Player), EffectKind::Character(
                    super::characters::CharacterEffect::Stance(super::characters::Stance::Neutral))),
            ])
        }
        EmptyBody => (1, vec![
            (Some(PayloadTarget::Player), EffectKind::GainBlock(modified_block(state, if upgraded { 10 } else { 7 })?)),
            (Some(PayloadTarget::Player), EffectKind::Character(
                super::characters::CharacterEffect::Stance(super::characters::Stance::Neutral))),
        ]),
        EmptyMind => (1, vec![
            (None, EffectKind::Draw(if upgraded { 3 } else { 2 })),
            (Some(PayloadTarget::Player), EffectKind::Character(
                super::characters::CharacterEffect::Stance(super::characters::Stance::Neutral))),
        ]),
        Consecrate => {
            if target.is_some() { return Err(crate::run::RunError::InvalidAction); }
            let mut effects = Vec::new();
            for (index, enemy) in state.enemies.iter().enumerate().filter(|(_, enemy)| enemy.targetable()) {
                let damage = modified_damage(state, index as u64 + 1, if upgraded { 8 } else { 5 })?;
                effects.push((Some(PayloadTarget::Enemy(enemy.instance_id.clone())),
                    EffectKind::Damage(DamagePayload::Fixed(damage))));
            }
            (0, effects)
        }
        Vigilance => (2, vec![
            (Some(PayloadTarget::Player), EffectKind::GainBlock(modified_block(state, if upgraded { 12 } else { 8 })?)),
            (Some(PayloadTarget::Player), EffectKind::Character(super::characters::CharacterEffect::Stance(super::characters::Stance::Calm))),
        ]),
        _ => return Err(crate::run::RunError::Unsupported("card handler not implemented")),
    };
    Ok((cost, effects))
}

/// Source-stage Wrist Blade modification. The first pure payload pass obtains
/// the live costForTurn; an eligible non-X Attack is recalculated with +4
/// present before the shared NORMAL damage modifiers run.
pub(super) fn payload_with_wrist_blade(state:&State,card:&LiveCard,target:Option<u64>)
    ->Result<(i32,Vec<(Option<PayloadTarget>,EffectKind)>),crate::run::RunError>{
    let attack=crate::content::Catalog::standard().card(card.id)
        .is_some_and(|definition|definition.card_type==crate::content::CardType::Attack);
    let vigor=if watcher::cards_active(state)&&attack{state.player_full_powers.iter()
        .find(|power|power.id.0=="vigor").map_or(0,|power|power.state.amount.max(0))}else{0};
    let eligible=state.item_context.as_ref().is_some_and(|items|
        (items.m15_profile()||items.complete_items())&&items.owns_relic(crate::content::RelicId::WristBlade))
        &&attack&&!card.local.x_cost;
    let complete_items=state.item_context.as_ref().is_some_and(crate::items::CombatItemContext::complete_items);
    let mut projection=if vigor>0||eligible||complete_items{Some(state.clone())}else{None};
    if vigor>0 {
        let projected=projection.as_mut().expect("Vigor projection allocated");
        projected.player_full_powers.push(super::FullPowerEntry{id:super::FullPowerId("strength"),
            state:super::PowerState{amount:vigor,skip_first:false,monster_source_grace:false}});
    }
    let base=payload(projection.as_ref().unwrap_or(state),card,target)?;
    let wrist=eligible&&super::card_state::effective_cost(card,base.0).ok()==Some(0);
    if !wrist {
        if complete_items {
            let projected=projection.get_or_insert_with(||state.clone());
            let definition=crate::content::Catalog::standard().card(card.id).expect("validated live card definition");
            let paid=state.play_context.as_ref().filter(|context|context.card==card_handle(card)).map_or(0,|context|context.paid);
            if let Some(items)=projected.item_context.as_mut(){
                items.set_damage_card_for_projection(crate::items::dispatch::PhaseCard{id:card.id,card_type:definition.card_type,cost_paid:paid});
            }
            return payload(projected,card,target);
        }
        return Ok(base)
    }
    let projected=projection.get_or_insert_with(||state.clone());
    if let Some(items)=projected.item_context.as_mut(){items.set_wrist_blade_bonus(4);}
    if complete_items {
        let definition=crate::content::Catalog::standard().card(card.id).expect("validated live card definition");
        let paid=state.play_context.as_ref().filter(|context|context.card==card_handle(card)).map_or(0,|context|context.paid);
        if let Some(items)=projected.item_context.as_mut(){
            items.set_damage_card_for_projection(crate::items::dispatch::PhaseCard{id:card.id,card_type:definition.card_type,cost_paid:paid});
        }
    }
    let boosted=payload(projected,card,target)?;
    if boosted.0!=base.0{return Err(crate::run::RunError::Unsupported("Wrist Blade payload cost changed during source modification"));}
    Ok(boosted)
}

/// Read-only equivalent of the ordinary card preparation path for an already
/// owned autoplay child. It shares use legality, payload production and target
/// conversion; only energy payment is waived by the caller.
pub(super) fn prepare_autoplay(state:&State,card:&LiveCard,target:Option<&Actor>,energy_on_use:i32,
    source:&CardHandle)->Result<Option<Vec<Effect>>,crate::run::RunError>{
    let mut projection=state.clone();
    let actor=target.cloned();
    projection.play_context=Some(super::model::PlayContext{card:source.clone(),target:actor.clone(),energy_on_use,paid:0,
        disposition:if card.local.purge{super::model::FinishDisposition::Purge}else{super::model::FinishDisposition::Natural}});
    let target_id=actor.as_ref().and_then(|actor|match actor{Actor::Player=>None,Actor::Enemy(owner)=>projection.enemies.iter().position(|enemy|&enemy.instance_id==owner).map(|index|index as u64+1)});
    let (cost,payloads,selected_target)=match payload_with_wrist_blade(&projection,card,None){
        Ok((cost,payloads))=>{validate_card_use(&projection,card,None)?;(cost,payloads,None)}
        Err(crate::run::RunError::InvalidAction)=>{
            let Some(id)=target_id else{return Ok(None)};
            if let Some(Actor::Enemy(owner))=&actor{
                if !projection.enemies.iter().any(|enemy|&enemy.instance_id==owner&&enemy.life==ActorLife::Active&&enemy.hp>0){return Ok(None)}
            }else{return Ok(None)}
            validate_card_use(&projection,card,Some(crate::combat::TargetId(id)))?;
            match payload_with_wrist_blade(&projection,card,Some(id)){
                Ok((cost,payloads))=>(cost,payloads,actor.clone()),
                Err(crate::run::RunError::InvalidAction)=>return Ok(None),
                Err(error)=>return Err(error),
            }
        }
        Err(error)=>return Err(error),
    };
    if cost>0&&!card.local.free_play&&!card.local.free_to_play_once{return Err(crate::run::RunError::InvalidAction)}
    let effects=payloads.into_iter().map(|(target,kind)|Effect{source:Source::Card(source.clone()),target:target.map(|target|match target{
        PayloadTarget::Player=>Actor::Player,
        PayloadTarget::SelectedEnemy=>selected_target.clone().expect("validated selected autoplay target"),
        PayloadTarget::Enemy(owner)=>Actor::Enemy(owner),
    }),kind}).collect();
    Ok(Some(effects))
}

pub(super) fn normal_damage(state: &State, base: i32, strength: i32, weak: bool, enemy:&super::model::EnemyState)
    -> Result<i32, crate::run::RunError>
{
    normal_damage_pipeline(state,base,strength,weak,enemy,false)
}
fn normal_damage_pipeline(state:&State,base:i32,strength:i32,weak:bool,enemy:&super::model::EnemyState,signed:bool)->Result<i32,crate::run::RunError>{
    let item_bonus=state.item_context.as_ref().map_or(0,crate::items::CombatItemContext::wrist_blade_bonus);
    let base=base.checked_add(item_bonus)
        .ok_or(crate::run::RunError::Unsupported("full card damage arithmetic overflow"))?;
    let base=state.item_context.as_ref().filter(|context|context.complete_items())
        .and_then(crate::items::CombatItemContext::damage_card).map_or(base,|card|{
            // M27 seam: outgoing-damage, after Wrist Blade and before Strength.
            crate::items::laws::outgoing_damage(state.item_context.as_ref().expect("item context present"),Some(card),
                &Actor::Enemy(enemy.instance_id.clone()),base)
        });
    if !signed&&(!(0..=3003).contains(&base) || !(-9990..=9990).contains(&strength)) {
        return Err(crate::run::RunError::Unsupported("normal damage outside supported numeric surface"));
    }
    let combined_base = if signed{0}else{base.checked_add(strength)
        .ok_or(crate::run::RunError::Unsupported("full card damage arithmetic overflow"))?};
    if !signed&&combined_base > 10004 {
        return Err(crate::run::RunError::Unsupported("full card damage exceeds supported numeric surface"));
    }
    let mut receive=Vec::new();
    let phrog=state.item_context.as_ref().is_some_and(|items|items.owns_relic(crate::content::RelicId::PaperPhrog));
    if enemy.powers.iter().any(|p|p.id==PowerId::Vulnerable&&p.state.amount>0){receive.push(if phrog{1.75f32}else{1.5f32});}
    for power in &enemy.full_powers{match power.id.0{
        "vulnerable" if power.state.amount>0=>receive.push(if phrog{1.75f32}else{1.5f32}),
        "slow"=>receive.push(1.0f32+0.1f32*power.state.amount as f32),_=>{},
    }}
    // Pen Nib is a separate ×2 atDamageGive multiplier. Applying its exact power-of-two
    // here preserves the shared damage calculator while stacking with double_damage.
    let pen_nib=state.player_full_powers.iter().any(|power|power.id.0=="pen_nib"&&power.state.amount>0);
    if pen_nib { receive.push(2.0f32); }
    // M26 §6.3 S: the stance pipeline runs once a stance or Vigor exists for any character.
    if watcher::active(state)||state.character.stance!=super::characters::Stance::Neutral
        ||state.player_full_powers.iter().any(|p|p.id.0=="vigor"){return rules::full_card_damage_for_stance(if signed{base}else{combined_base.max(0)},if signed{strength}else{0},weak,state.character.stance,
        state.player_full_powers.iter().any(|p|p.id.0=="double_damage"&&p.state.amount>0),receive,signed).map_err(|_|crate::run::RunError::Unsupported("invalid Watcher card damage"))}
    if signed{return rules::full_signed_card_damage(base,strength,weak,state.character.stance==super::characters::Stance::Wrath,
        state.player_full_powers.iter().any(|p|p.id.0=="double_damage"&&p.state.amount>0),receive).map_err(|_|crate::run::RunError::Unsupported("invalid signed card damage"))}
    rules::full_normal_damage_given_received(combined_base.max(0), 0, weak,
        state.character.stance == super::characters::Stance::Wrath, state.player_full_powers.iter().any(|p|p.id.0=="double_damage"&&p.state.amount>0), receive)
        .map_err(|_| crate::run::RunError::Unsupported("invalid card damage"))
}

pub(super) fn normal_potion_damage(state:&State,base:i32,enemy:&super::model::EnemyState)
    ->Result<i32,crate::run::RunError>{
    if !(0..=30_012).contains(&base){return Err(crate::run::RunError::Unsupported("potion damage outside supported numeric surface"))}
    let mut receive=Vec::new();
    let phrog=state.item_context.as_ref().is_some_and(|items|items.owns_relic(crate::content::RelicId::PaperPhrog));
    if enemy.powers.iter().any(|power|power.id==PowerId::Vulnerable&&power.state.amount>0){receive.push(if phrog{1.75f32}else{1.5f32});}
    for power in &enemy.full_powers{match power.id.0{
        "vulnerable" if power.state.amount>0=>receive.push(if phrog{1.75f32}else{1.5f32}),
        "slow"=>receive.push(1.0f32+0.1f32*power.state.amount as f32),_=>(),
    }}
    rules::full_normal_damage_given_received(base,0,false,false,false,receive)
        .map_err(|_|crate::run::RunError::Unsupported("invalid potion damage"))
}

fn modified_damage(state:&State,target:u64,base:i32)->Result<i32,crate::run::RunError>{modified_damage_pipeline(state,target,base,false)}
fn modified_mutable_damage(state:&State,target:u64,base:i32)->Result<i32,crate::run::RunError>{modified_damage_pipeline(state,target,base,true)}
fn modified_damage_pipeline(state:&State,target:u64,base:i32,signed:bool)->Result<i32,crate::run::RunError>{
    let enemy = state.enemies.get((target - 1) as usize)
        .filter(|enemy| enemy.hp > 0).ok_or(crate::run::RunError::InvalidAction)?;
    let strength = state.player.powers.iter().find(|p| p.id == PowerId::Strength)
        .map_or(0, |p| p.state.amount)
        + state.player_full_powers.iter().filter(|p| p.id.0 == "strength").map(|p| p.state.amount).sum::<i32>();
    let weak = state.player_full_powers.iter().any(|p| p.id.0 == "weak" && p.state.amount > 0);
    normal_damage_pipeline(state, base, strength, weak, enemy,signed)
}

fn modified_block(state:&State,base:i32)->Result<i32,crate::run::RunError>{modified_block_pipeline(state,base,false)}
fn modified_mutable_block(state:&State,base:i32)->Result<i32,crate::run::RunError>{modified_block_pipeline(state,base,true)}
fn modified_block_pipeline(state:&State,base:i32,signed:bool)->Result<i32,crate::run::RunError>{
    if state.player_full_powers.iter().any(|power|power.id.0=="no_block"&&power.state.amount>0){return Ok(0)}
    let dexterity = state.player_full_powers.iter().find(|p| p.id.0 == "dexterity").map_or(0, |p| p.state.amount);
    let frail = state.player_full_powers.iter().any(|p| p.id.0 == "frail" && p.state.amount > 0);
    (if watcher::active(state){rules::full_watcher_card_block(base,dexterity,frail)}else if signed{rules::full_signed_card_block(base,dexterity,frail)}else{rules::full_card_block(base, dexterity, frail)})
        .map_err(|_| crate::run::RunError::Unsupported("invalid calculated card Block"))
}

fn card_handle(card: &LiveCard) -> CardHandle {
    super::card_state::handle(card.instance)
}

/// One public card-value projection; established Ironclad values stay exact.
pub(super) fn public_values(card:&LiveCard)->Option<CardPublicValues>{
    watcher::public_values(card).or_else(||defect::public_values(card)).or_else(||super::generation::public_values(card)).or_else(||ironclad::rules::public_values(card)).or_else(||silent::public_values(card)).or_else(||colorless::public_values(card))
        .map(|mut values|{values.free_play=card.local.free_play||card.local.free_to_play_once;values})
}

pub(super) fn colorless_power_hook(state:&mut State,hook:Hook,target:Option<&Actor>,work:&mut Vec<Effect>)->Result<(),EffectError>{colorless::power_hook(state,hook,target,work)}
pub(super) fn colorless_start_power(state:&mut State,hook:Hook,power:&FullPowerEntry,work:&mut Vec<Effect>)->Result<(),EffectError>{colorless::start_power(state,hook,power,work)}
pub(super) fn colorless_power_card_played(state:&mut State,source:&Source)->Result<(),EffectError>{colorless::card_played(state,source)}
pub(super) fn validate_colorless_power_application(state:&State,effect:&Effect,id:FullPowerId,amount:i32)->Result<(),InputError>{colorless::validate_power_application(state,effect,id,amount)}
pub(super) fn validate_colorless_state(state:&State)->Result<(),InputError>{colorless::validate_state(state)}
pub(super) fn colorless_power_applied(state:&mut State,actor:&Actor,id:FullPowerId,application:bool,previous:i32){colorless::power_applied(state,actor,id,application,previous)}
pub(super) fn colorless_power_removed(state:&mut State,actor:&Actor,id:FullPowerId){colorless::power_removed(state,actor,id)}
pub(super) fn silent_power_hook(state:&mut State,hook:Hook,target:Option<&Actor>,work:&mut Vec<Effect>)->Result<(),EffectError>{silent::power_hook(state,hook,target,work)}
pub(super) fn silent_start_power(state:&State,hook:Hook,power:&FullPowerEntry,work:&mut Vec<Effect>){silent::start_power(state,hook,power,work)}
pub(super) fn silent_power_on_use(state:&mut State)->Result<(),EffectError>{silent::power_on_use(state)}
pub(super) fn silent_power_after_played(state:&mut State){silent::power_after_played(state)}
pub(super) fn silent_power_after_damage(state:&mut State,result:&super::model::DamageResult){silent::power_after_damage(state,result)}

pub(super) fn capture_silent_targets(state:&mut State,effects:&mut [Effect])->Result<(),EffectError>{silent::capture_targets(state,effects)?;defect::capture_direct(state,effects)?;super::generation::capture(state,effects)?;watcher::capture(state,effects)?;Ok(())}
pub(super) fn silent_poison_death_work(state:&State,target:Option<&Actor>)->Result<Vec<Effect>,InputError>{silent::poison_death_work(state,target)}

pub(in crate::combat) fn captured_x(state:&State,card:&LiveCard)->Result<i32,crate::run::RunError>{
    let handle = crate::combat::card_state::handle(card.instance);
    let captured = state.play_context.as_ref().filter(|context| context.card == handle)
        .map_or(state.player.energy, |context| {
            if context.energy_on_use == -1 { state.player.energy } else { context.energy_on_use }
        });
    if !(0..=999).contains(&captured) {
        return Err(crate::run::RunError::Unsupported("captured X outside supported energy surface"));
    }
    Ok(captured)
}

pub(super) fn chemical_x_bonus(state:&State)->Result<i32,crate::run::RunError>{
    let Some(items)=state.item_context.as_ref() else{return Ok(0)};
    if items.owns_relic(crate::content::RelicId::ChemicalX){return Ok(2)}
    if items.relics().iter().any(|relic|relic.as_str()=="chemical_x"){
        return Err(crate::run::RunError::Unsupported("Chemical X relic identity is not represented for X action"));
    }
    Ok(0)
}
