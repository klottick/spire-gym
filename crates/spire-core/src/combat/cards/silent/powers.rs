//! Delayed green resources and typed lifecycle callbacks on the shared queue.
use super::*;
use crate::content::CardId;

pub(super) fn payload(state:&State,card:&LiveCard,target:Option<u64>)
    ->Result<Option<(i32,Vec<(Option<PayloadTarget>,EffectKind)>)>,crate::run::RunError>{
    use CardId::*;
    if !matches!(card.id,DodgeAndRoll|FlyingKnee|Outmaneuver|Blur|Caltrops|Choke|Footwork|InfiniteBlades|Predator|AThousandCuts|AfterImage|Envenom|PhantasmalKiller|ToolsOfTheTrade|WraithForm|Burst){return Ok(None)}
    let up=card.upgrades>0;
    if matches!(card.id,FlyingKnee|Choke|Predator)!=target.is_some(){return Err(crate::run::RunError::InvalidAction)}
    let mut effects=Vec::new();
    let player_power=|name,amount|(Some(PayloadTarget::Player),EffectKind::ApplyFullPower{id:FullPowerId(name),amount});
    let cost=match card.id {
        DodgeAndRoll=>{
            let block=modified_block(state,if up{6}else{4})?;
            effects.push((Some(PayloadTarget::Player),EffectKind::GainBlock(block)));
            if block>0{effects.push(player_power("next_turn_block",block));}1
        }
        Blur=>{
            effects.push((Some(PayloadTarget::Player),EffectKind::GainBlock(modified_block(state,if up{8}else{5})?)));
            effects.push(player_power("blur",1));1
        }
        FlyingKnee|Choke|Predator=>{
            let damage=match card.id{FlyingKnee=>if up{11}else{8},Predator=>if up{20}else{15},_=>12};
            effects.push((Some(PayloadTarget::SelectedEnemy),EffectKind::Damage(DamagePayload::Fixed(super::super::modified_damage(state,target.unwrap(),damage)?))));
            match card.id{
                FlyingKnee=>effects.push(player_power("energized",1)),
                Predator=>effects.push(player_power("draw_card_next_turn",2)),
                _=>effects.push((Some(PayloadTarget::SelectedEnemy),EffectKind::ApplyFullPower{id:FullPowerId("choked"),amount:if up{5}else{3}})),
            }
            if card.id==FlyingKnee{1}else{2}
        }
        Outmaneuver=>{effects.push(player_power("energized",if up{3}else{2}));1}
        Footwork=>{effects.push(player_power("dexterity",if up{3}else{2}));1}
        Caltrops=>{effects.push(player_power("thorns",if up{5}else{3}));1}
        InfiniteBlades=>{effects.push(player_power("infinite_blades",1));1}
        AThousandCuts=>{effects.push(player_power("a_thousand_cuts",if up{2}else{1}));2}
        AfterImage=>{effects.push(player_power("after_image",1));1}
        Envenom=>{effects.push(player_power("envenom",1));if up{1}else{2}}
        PhantasmalKiller=>{effects.push(player_power("phantasmal",1));if up{0}else{1}}
        ToolsOfTheTrade=>{effects.push(player_power("tools_of_the_trade",1));if up{0}else{1}}
        WraithForm=>{
            effects.push(player_power("intangible_player",if up{3}else{2}));
            effects.push(player_power("wraith_form",-1));3
        }
        Burst=>{effects.push(player_power("burst",if up{2}else{1}));1}
        _=>unreachable!(),
    };
    Ok(Some((cost,effects)))
}

fn player_work(kind:EffectKind)->Effect{Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Player),kind}}
fn power_work(name:&'static str,amount:i32)->Effect{player_work(EffectKind::ApplyFullPower{id:FullPowerId(name),amount})}
fn remove(name:&'static str)->Effect{player_work(EffectKind::RemoveFullPower(FullPowerId(name)))}

/// On-use work precedes the existing after-use checkpoint, in callback order.
fn append_use_work(state:&mut State,effect:Effect){
    let at=state.pending.iter().position(|work|work.kind==EffectKind::Hook(Hook::CardAfterUse)).unwrap_or_else(||state.pending.iter().position(|work|work.kind==EffectKind::FinishCard).unwrap_or(state.pending.len()));
    state.pending.insert(at,effect);
}

pub(in crate::combat) fn card_on_use(state:&mut State)->Result<(),EffectError>{
    let amounts=state.player_full_powers.iter().filter(|p|p.id.0=="after_image"&&p.state.amount>0).map(|p|p.state.amount).collect::<Vec<_>>();
    for block in amounts{append_use_work(state,player_work(EffectKind::GainBlock(block)));}
    let targets=state.enemies.iter().filter(|e|e.hp>0&&e.life==ActorLife::Active).filter_map(|e|e.full_powers.iter().find(|p|p.id.0=="choked").map(|p|(e.instance_id.clone(),p.state.amount))).collect::<Vec<_>>();
    for (target,amount) in targets{append_use_work(state,Effect{source:Source::None,target:Some(Actor::Enemy(target)),kind:EffectKind::Damage(DamagePayload::HpLoss(amount))});}
    Ok(())
}

pub(in crate::combat) fn card_after_played(state:&mut State){
    let cuts=state.player_full_powers.iter().find(|p|p.id.0=="a_thousand_cuts").map_or(0,|p|p.state.amount);
    if cuts>0{
        let targets=state.enemies.iter().filter(|e|e.hp>0&&e.life==ActorLife::Active).map(|e|e.instance_id.clone()).collect::<Vec<_>>();
        let at=state.pending.iter().position(|work|work.kind==EffectKind::FinishCard).map_or(state.pending.len(),|index|index+1);
        let work=targets.into_iter().map(|target|Effect{source:Source::Actor(Actor::Player),target:Some(Actor::Enemy(target)),kind:EffectKind::Damage(DamagePayload::Thorns(cuts))}).collect::<Vec<_>>();
        for (offset,effect) in work.into_iter().enumerate(){state.pending.insert(at+offset,effect);}
    }
}

pub(in crate::combat) fn hook(state:&mut State,hook:Hook,target:Option<&Actor>,work:&mut Vec<Effect>)->Result<(),EffectError>{
    let player=target==Some(&Actor::Player);
    let powers=state.player_full_powers.clone();
    if player&&hook==Hook::TurnStartedPreDraw{
        for power in &powers{
            match power.id.0 {
                "next_turn_block"=>{work.push(player_work(EffectKind::GainBlock(power.state.amount)));work.push(remove("next_turn_block"));}
                "infinite_blades"=>{for _ in 0..power.state.amount{work.push(player_work(EffectKind::CreateCard{id:CardId::Shiv,destination:crate::combat::CardDestination::Hand,upgrades:0}));}}
                "phantasmal"=>{work.push(power_work("double_damage",power.state.amount));work.push(player_work(EffectKind::ReduceFullPower{id:power.id,amount:1}));}
                _=>(),
            }
        }
    }
    if player&&hook==Hook::TurnStarted{
        // The source priorities are DrawNextTurn20, Tools25. Preserve source
        // insertion order within each category, independent of install order.
        for power in powers.iter().filter(|p|p.id.0=="draw_card_next_turn"){
            work.push(player_work(EffectKind::Draw(power.state.amount)));work.push(remove("draw_card_next_turn"));
        }
        for power in powers.iter().filter(|p|p.id.0=="tools_of_the_trade"){
            work.push(player_work(EffectKind::Draw(power.state.amount)));
            work.push(player_work(EffectKind::Silent(super::SilentEffect::ToolsDiscard{count:power.state.amount as usize,turn:state.turn})));
        }
    }
    if player&&hook==Hook::PlayerPreEndTurnCards{
        for power in powers.iter().filter(|p|p.id.0=="wraith_form"){work.push(power_work("dexterity",power.state.amount));}
    }
    if hook==Hook::TurnEnded{
        state.silent.retain_block_next_start=powers.iter().any(|p|p.id.0=="blur"&&p.state.amount>0);
        if powers.iter().any(|p|p.id.0=="burst"&&p.state.amount>0){work.push(remove("burst"));}
        for power in powers.iter().filter(|p|matches!(p.id.0,"blur"|"intangible_player"|"double_damage")){
            work.push(player_work(EffectKind::ReduceFullPower{id:power.id,amount:1}));
        }
    }
    if hook==Hook::EnemyTurnStarted{
        if let Some(Actor::Enemy(owner))=target{
            if state.enemies.iter().any(|e|e.instance_id==*owner&&e.full_powers.iter().any(|p|p.id.0=="choked")){
                work.push(Effect{source:Source::Actor(Actor::Enemy(owner.clone())),target:Some(Actor::Enemy(owner.clone())),kind:EffectKind::RemoveFullPower(FullPowerId("choked"))});
            }
        }
    }
    Ok(())
}

pub(in crate::combat) fn after_damage(state:&mut State,result:&crate::combat::model::DamageResult){
    if !result.normal||result.hp_lost<=0||matches!(result.target,Actor::Player){return}
    let amount=state.player_full_powers.iter().find(|p|p.id.0=="envenom").map_or(0,|p|p.state.amount);
    if amount>0&&matches!(result.source,Source::Card(_)|Source::Potion{..}|Source::Actor(Actor::Player)){
        state.pending.push_front(Effect{source:Source::Actor(Actor::Player),target:Some(result.target.clone()),kind:EffectKind::ApplyFullPower{id:FullPowerId("poison"),amount}});
    }
}
