//! Silent/common card payloads using the shared native resolver.
use super::*;

mod poison;
mod discard;
mod shiv;
mod retain;
mod copy;
mod direct;
mod history;
mod powers;
mod state;
mod costs;
mod x_repeat;
pub use state::{SilentEffect, SilentState, DrawnCard};
pub(super) use state::validate_work;
pub(super) use history::apply;

pub(super) fn increment_discard(state:&mut State,notify_cards:bool)->Result<(),crate::combat::InputError>{
    state::increment_discard(state,notify_cards)
}
pub(super) fn actual_draw(state:&mut State,owner:&crate::combat::CardHandle)->Result<(),crate::combat::EffectError>{
    Ok(costs::actual_draw(state,owner)?)
}
pub(super) fn setup_move_card(state:&mut State,owned:crate::combat::model::OwnedCard,handle:&crate::combat::CardHandle)
    ->Result<(),crate::combat::InputError>{costs::setup_move_card(state,owned,handle)}

pub(super) fn upgrade_local(card:&mut LiveCard)->Result<(),crate::run::RunError>{costs::upgrade_local(card)}

pub(super) fn starting_cost(id:crate::content::CardId,upgrades:u32)->Option<i32>{costs::starting_cost(id,upgrades)}
pub(super) fn before_make_copy(source:&mut LiveCard,history:super::CopyHistory)->Result<(),crate::combat::InputError>{
    costs::before_make_copy(source,history)
}
pub(super) fn player_hp_lost(state:&mut State)->Result<(),crate::combat::InputError>{costs::player_hp_lost(state)}

pub(super) fn requires_draw_history(id:crate::content::CardId)->bool {
    use crate::content::CardId::*;
    matches!(id,Skewer|Burst|Doppelganger|Malaise|StormOfSteel|Acrobatics|Prepared|SneakyStrike|AllOutAttack|CalculatedGamble|Concentrate|EscapePlan|Expertise|Finisher|Flechettes|GrandFinale|Unload|DodgeAndRoll|FlyingKnee|Outmaneuver|Blur|Caltrops|Choke|Footwork|InfiniteBlades|Predator|AThousandCuts|AfterImage|Envenom|PhantasmalKiller|ToolsOfTheTrade|WraithForm|Bane|BouncingFlask|PiercingWail|CripplingCloud|HeelHook|Terror|CorpseExplosion|MasterfulStab|Eviscerate|Setup|BulletTime|EndlessAgony|GlassKnife|DaggerSpray|Slice|SuckerPunch|Backstab|Dash|LegSweep|RiddleWithHoles|Adrenaline|DieDieDie)
}

pub(super) fn manual_discard(state: &mut State, card: &LiveCard) -> Result<bool, crate::combat::InputError> {
    discard::manual_discard(state, card)
}

pub(super) fn apply_catalyst(state: &mut State, effect: &Effect) -> Result<(), super::super::EffectError> {
    poison::apply_catalyst(state, effect)
}

pub(super) fn payload(state: &State, card: &LiveCard, target: Option<u64>)
    -> Result<Option<(i32, Vec<(Option<PayloadTarget>, EffectKind)>)>, crate::run::RunError>
{
    if let Some(result) = powers::payload(state, card, target)? { return Ok(Some(result)); }
    if let Some(result) = history::payload(state, card, target)? { return Ok(Some(result)); }
    if let Some(result) = direct::payload(state, card, target)? { return Ok(Some(result)); }
    if let Some(result) = costs::payload(state, card, target)? { return Ok(Some(result)); }
    if let Some(result) = x_repeat::payload(state, card, target)? { return Ok(Some(result)); }
    if let Some(result) = poison::payload(state, card, target)? { return Ok(Some(result)); }
    if let Some(result) = shiv::payload(state, card, target)? { return Ok(Some(result)); }
    if let Some(result) = copy::payload(card, target)? { return Ok(Some(result)); }
    use crate::content::CardId::*;
    let upgraded = card.upgrades > 0;
    let result = match card.id {
        Backflip => (1, vec![
            (Some(PayloadTarget::Player), EffectKind::GainBlock(modified_block(state, if upgraded { 8 } else { 5 })?)),
            (None, EffectKind::Draw(2)),
        ]),
        Deflect => (0, vec![(Some(PayloadTarget::Player), EffectKind::GainBlock(modified_block(state, if upgraded { 7 } else { 4 })?))]),
        QuickSlash | DaggerThrow => {
            let target_id = target.ok_or(crate::run::RunError::InvalidAction)?;
            let base = match (card.id, upgraded) {
                (QuickSlash, false) => 8, (QuickSlash, true) => 12,
                (DaggerThrow, false) => 9, (DaggerThrow, true) => 12,
                _ => unreachable!(),
            };
            let damage = super::modified_damage(state, target_id, base)?;
            let enemy = state.enemies.get((target_id - 1) as usize).unwrap().instance_id.clone();
            let mut effects = vec![(Some(PayloadTarget::Enemy(enemy)), EffectKind::Damage(DamagePayload::Fixed(damage)))];
            if card.id == QuickSlash {
                effects.push((None, EffectKind::Draw(1)));
            } else {
                effects.push((None, EffectKind::Draw(1)));
                effects.push((None, EffectKind::SelectHandCardsAfterDraw { required: 1,
                    disposition: super::super::SelectionDisposition::Discard,
                    empty: super::super::EmptySelection::Continue }));
            }
            (1, effects)
        }
        CloakAndDagger => {
            let mut effects = vec![(Some(PayloadTarget::Player), EffectKind::GainBlock(modified_block(state, 6)?)),
                (Some(PayloadTarget::Player), EffectKind::CreateCard { id: crate::content::CardId::Shiv,
                    destination: crate::combat::CardDestination::Hand, upgrades: 0 })];
            if upgraded {
                effects.push((Some(PayloadTarget::Player), EffectKind::CreateCard { id: crate::content::CardId::Shiv,
                    destination: crate::combat::CardDestination::Hand, upgrades: 0 }));
            }
            (1, effects)
        }
        WellLaidPlans => (1, vec![(Some(PayloadTarget::Player),EffectKind::ApplyFullPower {
            id:FullPowerId("retain_cards"),amount:if upgraded{2}else{1},
        })]),
        _ => return Ok(None),
    };
    Ok(Some(result))
}

pub(super) fn rewrite_accuracy_shivs(state: &mut State, amount: i32) {
    shiv::rewrite_existing(state, amount);
}

pub(super) fn rewrite_accuracy_hand_shivs(state: &mut State, amount: i32) {
    shiv::rewrite_hand(state, amount);
}

pub(super) fn apply_retain_selection(state: &mut State, effect: &Effect, limit: i32)
    -> Result<(), super::super::EffectError> {
    retain::begin(state, effect, limit)
}

pub(super) fn capture_nightmare_template(state:&mut State,owner:crate::combat::CardHandle,
    source:&LiveCard)->Result<(),crate::combat::InputError>{copy::capture(state,owner,source)}
pub(super) fn capture_nightmare_template_from_owner(state:&mut State,owner:crate::combat::CardHandle,
    source:&crate::combat::CardHandle)->Result<(),crate::combat::InputError>{copy::capture_from_owner(state,owner,source)}

pub(super) fn nightmare_pre_draw(state:&State)->Vec<Effect>{copy::pre_draw(state)}

pub(super) fn apply_nightmare_copy(state:&mut State,effect:&Effect,owner:&crate::combat::CardHandle,installation_id:u64)
    ->Result<(),super::super::EffectError>{copy::deliver(state,effect,owner,installation_id)}

pub(super) fn validate_nightmare_templates(state:&State)->Result<(),crate::combat::InputError>{
    copy::validate_pending(state)
}

/// Visible quantities for the finite supported Silent slice; execution stays shared.
pub(super) fn public_values(card:&LiveCard)->Option<CardPublicValues>{
    use crate::content::CardId::*;
    let up=card.upgrades>0;
    let (damage,block,magic)=match card.id{
        Neutralize=>(Some(if up{4}else{3}),None,Some(if up{2}else{1})),
        Survivor=>(None,Some(if up{11}else{8}),Some(1)),
        PoisonedStab=>(Some(if up{8}else{6}),None,Some(if up{4}else{3})),
        Backflip=>(None,Some(if up{8}else{5}),Some(2)),
        BladeDance=>(None,None,Some(if up{4}else{3})),
        DeadlyPoison=>(None,None,Some(if up{7}else{5})),
        CloakAndDagger=>(None,Some(6),Some(if up{2}else{1})),
        DaggerThrow=>(Some(if up{12}else{9}),None,Some(1)),
        NoxiousFumes=>(None,None,Some(if up{3}else{2})),
        Accuracy=>(None,None,Some(if up{6}else{4})),
        WellLaidPlans=>(None,None,Some(if up{2}else{1})),
        QuickSlash=>(Some(if up{12}else{8}),None,Some(1)),
        Reflex=>(None,None,Some(if up{3}else{2})),
        Catalyst=>(None,None,Some(if up{3}else{2})),
        Deflect=>(None,Some(if up{7}else{4}),None),
        Tactician=>(None,None,Some(if up{2}else{1})),
        Nightmare=>(None,None,Some(3)),
        Shiv=>(Some(card.local.base_damage.unwrap_or(if up{6}else{4})),None,None),
        DaggerSpray=>(Some(if up{6}else{4}),None,Some(2)),
        Slice=>(Some(if up{9}else{6}),None,None),
        SuckerPunch=>(Some(if up{9}else{7}),None,Some(if up{2}else{1})),
        Backstab=>(Some(if up{15}else{11}),None,None),
        Dash=>(Some(if up{13}else{10}),Some(if up{13}else{10}),None),
        LegSweep=>(None,Some(if up{14}else{11}),Some(if up{3}else{2})),
        RiddleWithHoles=>(Some(if up{4}else{3}),None,Some(5)),
        Adrenaline=>(None,None,Some(if up{2}else{1})),
        DieDieDie=>(Some(if up{17}else{13}),None,None),
        Acrobatics=>(None,None,Some(if up{4}else{3})),
        Prepared=>(None,None,Some(if up{2}else{1})),
        SneakyStrike=>(Some(if up{16}else{12}),None,None),
        AllOutAttack=>(Some(if up{14}else{10}),None,None),
        CalculatedGamble=>(None,None,None),
        Concentrate=>(None,None,Some(if up{2}else{3})),
        EscapePlan=>(None,Some(if up{5}else{3}),None),
        Expertise=>(None,None,Some(if up{7}else{6})),
        Finisher=>(Some(if up{8}else{6}),None,None),
        Flechettes=>(Some(if up{6}else{4}),None,None),
        GrandFinale=>(Some(if up{60}else{50}),None,None),
        Unload=>(Some(if up{18}else{14}),None,None),
        DodgeAndRoll=>(None,Some(if up{6}else{4}),None),
        FlyingKnee=>(Some(if up{11}else{8}),None,Some(1)),
        Outmaneuver=>(None,None,Some(if up{3}else{2})),
        Blur=>(None,Some(if up{8}else{5}),Some(1)),
        Caltrops=>(None,None,Some(if up{5}else{3})),
        Choke=>(Some(12),None,Some(if up{5}else{3})),
        Footwork=>(None,None,Some(if up{3}else{2})),
        InfiniteBlades=>(None,None,Some(1)),
        Predator=>(Some(if up{20}else{15}),None,Some(2)),
        AThousandCuts=>(None,None,Some(if up{2}else{1})),
        AfterImage=>(None,Some(1),None),
        Envenom|PhantasmalKiller|ToolsOfTheTrade=>(None,None,Some(1)),
        WraithForm=>(None,None,Some(if up{3}else{2})),
        Bane=>(Some(if up{10}else{7}),None,None),
        BouncingFlask=>(None,None,Some(if up{4}else{3})),
        PiercingWail=>(None,None,Some(if up{8}else{6})),
        CripplingCloud=>(None,None,Some(if up{7}else{4})),
        HeelHook=>(Some(if up{8}else{5}),None,None),
        Terror=>(None,None,Some(99)),
        CorpseExplosion=>(None,None,Some(if up{9}else{6})),
        MasterfulStab=>(Some(card.local.base_damage.unwrap_or(if up{16}else{12})),None,None),
        Eviscerate=>(Some(card.local.base_damage.unwrap_or(if up{9}else{7})),None,None),
        Setup|BulletTime=>(None,None,None),
        EndlessAgony=>(Some(card.local.base_damage.unwrap_or(if up{6}else{4})),None,None),
        GlassKnife=>(Some(card.local.base_damage.unwrap_or(if up{12}else{8})),None,None),
        Skewer=>(Some(card.local.base_damage.unwrap_or(if up{10}else{7})),None,None),
        Burst=>(None,None,Some(if up{2}else{1})),
        Doppelganger|Malaise|StormOfSteel=>(None,None,None),
        _=>return None,
    };
    Some(CardPublicValues{base_damage:damage,base_block:block,magic_number:magic,damage_bonus:None,
        cost_kind:if matches!(card.id,Reflex|Tactician){CardCostKind::Unplayable}
            else if matches!(card.id,Skewer|Doppelganger|Malaise){CardCostKind::X}else{CardCostKind::Fixed},
        free_play:card.local.free_play||card.local.free_to_play_once,exhaust_on_finish:card.local.exhaust_on_finish,
        ethereal:card.local.ethereal,innate:card.local.innate,temporary_cost:card.local.temporary_cost})
}

pub(super) fn power_hook(state:&mut State,hook:Hook,target:Option<&Actor>,work:&mut Vec<Effect>)->Result<(),EffectError>{powers::hook(state,hook,target,work)}
pub(super) fn power_on_use(state:&mut State)->Result<(),EffectError>{powers::card_on_use(state)}
pub(super) fn power_after_played(state:&mut State){powers::card_after_played(state)}
pub(super) fn power_after_damage(state:&mut State,result:&crate::combat::model::DamageResult){powers::after_damage(state,result)}

pub(super) fn capture_targets(state:&mut State,effects:&mut [Effect])->Result<(),EffectError>{poison::capture_targets(state,effects)}
pub(super) fn poison_death_work(state:&State,target:Option<&Actor>)->Result<Vec<Effect>,InputError>{poison::death_work(state,target)}
